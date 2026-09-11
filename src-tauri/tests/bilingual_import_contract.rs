//! Cổng HỢP ĐỒNG của Story 6.16 (FR115, AD-39 · AD-37/46 · AD-47 ③) — nhập tài liệu song
//! ngữ hai cột (`.csv`/`.tsv`), trên ĐÚNG đường sản phẩm (`import_bilingual_file` →
//! `stash_pending_import_source` → `confirm_bilingual_import` → `create_work`), không một
//! hàm chỉ-test nào bọc ngoài. Mỗi hàng của §I/O & Edge-Case Matrix có mặt ở đây.
//!
//! Bốn đối chứng đỏ mà §Verification đòi (record trong §Implementation Notes của spec):
//! ① gỡ `target_text: &segment.target_text` khỏi `insert_bilingual_segments` — segment mất
//!    bản dịch; ② đổi `LifecycleStatus::InProgress` thành `NotStarted` trên nhánh
//!    `chapter.bilingual_segments.is_some()` của `create_work`; ③ cho phép
//!    `normalize::normalize` chạy trên toàn bộ hàng nối lại (xuyên hàng) thay vì mỗi ô riêng
//!    — phá `a_quoted_cell_with_an_internal_line_break_never_sets_a_flag`; ④ gỡ khối
//!    `if !outcome.bilingual_mismatches.is_empty()` khỏi `create_work` — cho phép ghi khi còn
//!    lệch cặp.

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::sync::atomic::{AtomicU64, Ordering};

use auratranslate_lib::commands::project::{
    PendingImportSourceState, cancel_import_preview, confirm_bilingual_import, preview_bilingual_import,
    stash_pending_import_source,
};
use auratranslate_lib::core::cleanup::{CleanupRule, CleanupRuleKind, CleanupRuleTier};
use auratranslate_lib::core::i18n::MessageKey;
use auratranslate_lib::core::segment::chapterpattern::ChapterPattern;
use auratranslate_lib::core::segment::import::import_bilingual_file;
use auratranslate_lib::core::store::Store;

static NEXT_DIR: AtomicU64 = AtomicU64::new(0);

fn temp_dir(tag: &str) -> PathBuf {
    let n = NEXT_DIR.fetch_add(1, Ordering::Relaxed);
    let dir = std::env::temp_dir().join(format!("auratranslate-bilingual-{}-{}-{}", std::process::id(), tag, n));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap_or_else(|e| panic!("tao {}: {e}", dir.display()));
    dir
}

fn cleanup(dir: &Path) {
    let _ = fs::remove_dir_all(dir);
}

fn write_file(dir: &Path, name: &str, content: &[u8]) -> PathBuf {
    let path = dir.join(name);
    fs::write(&path, content).unwrap_or_else(|e| panic!("ghi {}: {e}", path.display()));
    path
}

fn pending_state() -> PendingImportSourceState {
    Mutex::new(None)
}

/// Số mục con NGAY DƯỚI `documents_root` — `0` nghĩa là 0 byte `.atproj` nào từng ghi xuống
/// (§Boundaries: "0 bytes on disk before confirm").
fn entry_count(dir: &Path) -> usize {
    fs::read_dir(dir).map(|it| it.count()).unwrap_or(0)
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ChapterRow {
    id: i64,
    title: Option<String>,
    status: String,
}

fn read_chapters(store: &Store) -> Vec<ChapterRow> {
    store
        .read(|conn| {
            let mut stmt = conn.prepare("SELECT id, title, status FROM chapter ORDER BY ord")?;
            let rows = stmt.query_map([], |r| {
                Ok(ChapterRow { id: r.get(0)?, title: r.get(1)?, status: r.get(2)? })
            })?;
            rows.collect::<Result<Vec<_>, _>>()
        })
        .expect("doc chapter that bai")
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct SegmentRow {
    source_text: String,
    target_text: String,
    is_paragraph_end: bool,
    is_target_paragraph_end: bool,
    translation_origin: String,
    status: String,
}

fn read_segments(store: &Store, chapter_id: i64) -> Vec<SegmentRow> {
    store
        .read(move |conn| {
            let mut stmt = conn.prepare(
                "SELECT source_text, target_text, is_paragraph_end, is_target_paragraph_end, \
                 translation_origin, status FROM segment WHERE chapter_id = ?1 \
                 AND retired_at IS NULL ORDER BY ord",
            )?;
            let rows = stmt.query_map([chapter_id], |r| {
                Ok(SegmentRow {
                    source_text: r.get(0)?,
                    target_text: r.get(1)?,
                    is_paragraph_end: r.get::<_, i64>(2)? != 0,
                    is_target_paragraph_end: r.get::<_, i64>(3)? != 0,
                    translation_origin: r.get(4)?,
                    status: r.get(5)?,
                })
            })?;
            rows.collect::<Result<Vec<_>, _>>()
        })
        .expect("doc segment that bai")
}

// ═════════════════════════════════════════════════════════════════════════════════
// "Pattern matches 3 rows" — Preview 3 Chapters; confirm ⇒ 1 Work, 3 Chapters in_progress
// ═════════════════════════════════════════════════════════════════════════════════

#[test]
fn a_pattern_matching_three_rows_yields_one_work_with_three_in_progress_chapters() {
    let root = temp_dir("three-chapters");
    let csv = "CHUONG MOT,CHAPTER ONE\n\
               Cau chuyen bat dau.,The story begins.\n\
               CHUONG HAI,CHAPTER TWO\n\
               Tiep tuc thoi.,Continuing now.\n\
               CHUONG BA,CHAPTER THREE\n\
               Ket thuc roi.,It ends now.\n";
    let path = write_file(&root, "ba-chuong.csv", csv.as_bytes());

    let shape = import_bilingual_file(&path).expect("tep hop le phai doc duoc");

    // §I/O Matrix "Pattern matches 3 rows" — vế "Preview 3 Chapters", do TRUOC khi xac nhan.
    let pattern = ChapterPattern::literal("CHUONG");
    let preview = preview_bilingual_import(&shape, "en", &[], Some(&pattern), 0, 1, false)
        .expect("xem truoc tep hop le phai thanh cong");
    let selected = preview
        .candidates
        .iter()
        .find(|c| c.encoding == preview.selected_encoding)
        .expect("ung vien dang chon phai co mat trong dai");
    assert_eq!(selected.chapter_count, 3, "man xem truoc phai hien 3 Chuong nhan ra");
    assert!(selected.mismatches.is_empty());

    let state = pending_state();
    stash_pending_import_source(&state, shape, None);

    assert_eq!(entry_count(&root), 1, "0 byte nao duoc ghi truoc xac nhan (chi tep .csv vua tao)");

    let opened = confirm_bilingual_import(
        &root,
        &state,
        "Ba Chuong",
        "en",
        "",
        "UTF-8",
        Vec::new(),
        Some(ChapterPattern::literal("CHUONG")),
        0,
        1,
        false,
    )
    .expect("xac nhan phai thanh cong");

    let chapters = read_chapters(&opened.store);
    assert_eq!(chapters.len(), 3, "mau khop 3 hang phai cho 3 Chuong");
    assert_eq!(chapters[0].title.as_deref(), Some("CHUONG MOT"));
    assert_eq!(chapters[1].title.as_deref(), Some("CHUONG HAI"));
    assert_eq!(chapters[2].title.as_deref(), Some("CHUONG BA"));
    for c in &chapters {
        assert_eq!(c.status, "in_progress", "Chuong duong song ngu phai InProgress, khong NotStarted");
    }

    cleanup(&root);
}

// ═════════════════════════════════════════════════════════════════════════════════
// "Equal counts" — 2 segment voi ca hai van ban, bilingual_import, draft
// ═════════════════════════════════════════════════════════════════════════════════

#[test]
fn equal_sentence_counts_write_both_texts_with_bilingual_origin_and_draft_status() {
    let root = temp_dir("equal-counts");
    let csv = "Cau mot day. Cau hai day.,Cau mot dich. Cau hai dich.\n";
    let path = write_file(&root, "hai-cau.csv", csv.as_bytes());

    let shape = import_bilingual_file(&path).expect("tep hop le phai doc duoc");
    let state = pending_state();
    stash_pending_import_source(&state, shape, None);

    let opened = confirm_bilingual_import(&root, &state, "Hai Cau", "en", "", "UTF-8", Vec::new(), None, 0, 1,false)
        .expect("xac nhan phai thanh cong");

    let chapters = read_chapters(&opened.store);
    assert_eq!(chapters.len(), 1);
    let segments = read_segments(&opened.store, chapters[0].id);
    assert_eq!(segments.len(), 2, "hai cau nguon <-> hai cau dich phai cho DUNG hai segment");
    for s in &segments {
        assert_eq!(s.translation_origin, "bilingual_import");
        assert_eq!(s.status, "draft");
        assert!(!s.target_text.is_empty(), "segment da cap phai mang target_text that");
    }
    assert_eq!(segments[0].source_text, "Cau mot day.");
    assert_eq!(segments[0].target_text, "Cau mot dich.");
    assert_eq!(segments[1].source_text, "Cau hai day.");
    assert_eq!(segments[1].target_text, "Cau hai dich.");

    cleanup(&root);
}

// ═════════════════════════════════════════════════════════════════════════════════
// "Paragraph flags" — AD-37/AD-46: off, off, on trong MOT hang; Chuong's last segment off
// ═════════════════════════════════════════════════════════════════════════════════

#[test]
fn row_flags_are_off_off_on_within_a_row_and_the_chapters_last_segment_is_always_off() {
    let root = temp_dir("row-flags");
    // Hàng 1 (KHÔNG phải hàng cuối Chương): ba câu mỗi bên -- cờ mong đợi false, false, true.
    // Hàng 2 (hàng CUỐI Chương): một câu mỗi bên -- cờ luôn tắt bất kể vị trí trong hàng.
    let csv = "One. Two. Three.,Mot. Hai. Ba.\nFour.,Bon.\n";
    let path = write_file(&root, "co-doan.csv", csv.as_bytes());

    let shape = import_bilingual_file(&path).expect("tep hop le phai doc duoc");
    let state = pending_state();
    stash_pending_import_source(&state, shape, None);

    let opened = confirm_bilingual_import(&root, &state, "Co Doan", "en", "", "UTF-8", Vec::new(), None, 0, 1,false)
        .expect("xac nhan phai thanh cong");

    let chapters = read_chapters(&opened.store);
    let segments = read_segments(&opened.store, chapters[0].id);
    assert_eq!(segments.len(), 4, "3 cau hang 1 + 1 cau hang 2 = 4 segment");

    let flags: Vec<bool> = segments.iter().map(|s| s.is_paragraph_end).collect();
    assert_eq!(flags, vec![false, false, true, false], "off, off, on trong hang 1; hang 2 (cuoi Chuong) tat");
    let target_flags: Vec<bool> = segments.iter().map(|s| s.is_target_paragraph_end).collect();
    assert_eq!(target_flags, flags, "AD-46 -- co dich MIRROR co nguon luc nhap");

    cleanup(&root);
}

/// §Always — "Line breaks inside a cell never set a flag." Một ô bọc nháy kép mang xuống
/// dòng THẬT bên trong (không phải ranh giới hàng — vẫn CÙNG một hàng logic) không được phép
/// tự bật cờ kết đoạn; chỉ VỊ TRÍ HÀNG mới quyết định cờ trên đường này.
#[test]
fn a_quoted_cell_with_an_internal_line_break_never_sets_a_flag() {
    let root = temp_dir("quoted-linebreak");
    let csv = "\"Dong mot.\nDong hai.\",\"Line one.\nLine two.\"\nCau cuoi.,Last sentence.\n";
    let path = write_file(&root, "xuong-dong.csv", csv.as_bytes());

    let shape = import_bilingual_file(&path).expect("tep hop le phai doc duoc");
    let state = pending_state();
    stash_pending_import_source(&state, shape, None);

    let opened = confirm_bilingual_import(&root, &state, "Xuong Dong", "en", "", "UTF-8", Vec::new(), None, 0, 1,false)
        .expect("xac nhan phai thanh cong");

    let chapters = read_chapters(&opened.store);
    let segments = read_segments(&opened.store, chapters[0].id);
    // Hàng 1 tách theo `\n` THẬT bên trong ô (AD-4/AC11 của split_source_text vẫn cắt xuống
    // dòng thành ranh giới CỨNG) ra hai segment -- nhưng đây là NỘI DUNG của MỘT hàng logic
    // duy nhất, không phải hai hàng file khác nhau; cờ của hai segment đó phải là [false,
    // true] theo VỊ TRÍ HÀNG (hàng 1 không phải hàng cuối Chương ⇒ segment CUỐI của hàng bật),
    // KHÔNG phải suy từ chính dấu `\n` nó chứa.
    assert_eq!(segments.len(), 3, "2 segment tu hang 1 (co xuong dong trong o) + 1 tu hang 2");
    assert_eq!(
        segments.iter().map(|s| s.is_paragraph_end).collect::<Vec<_>>(),
        vec![false, true, false],
        "segment cuoi HANG 1 bat (khong phai hang cuoi Chuong); segment cuoi Chuong (hang 2) tat"
    );

    cleanup(&root);
}

// ═════════════════════════════════════════════════════════════════════════════════
// "Mismatched row" / "Blank target cell" — liet ke, confirm bi khoa Ở RUST
// ═════════════════════════════════════════════════════════════════════════════════

#[test]
fn a_mismatched_row_is_listed_in_preview_and_confirm_is_refused_writing_nothing() {
    let root = temp_dir("mismatch");
    let csv = "Hai cau day. Cau hai nua.,Chi mot cau thoi.\n";
    let path = write_file(&root, "lech.csv", csv.as_bytes());

    let shape = import_bilingual_file(&path).expect("tep hop le phai doc duoc");

    let preview = preview_bilingual_import(&shape, "en", &[], None, 0, 1, false).expect("xem truoc tep hop le phai thanh cong");
    let selected = preview
        .candidates
        .iter()
        .find(|c| c.encoding == preview.selected_encoding)
        .expect("ung vien dang chon phai co mat trong dai");
    assert_eq!(selected.mismatches.len(), 1, "preview phai liet ke dung MOT hang lech cap");
    assert_eq!(selected.mismatches[0].row_number, 1);
    assert_eq!(selected.mismatches[0].source_sentence_count, 2);
    assert_eq!(selected.mismatches[0].target_sentence_count, 1);

    let state = pending_state();
    stash_pending_import_source(&state, shape, None);
    let err = confirm_bilingual_import(&root, &state, "Lech Cap", "en", "", "UTF-8", Vec::new(), None, 0, 1,false)
        .expect_err("con hang lech cap thi xac nhan phai bi tu choi O RUST");
    assert_eq!(err.message_key(), MessageKey::ImportBilingualMismatchedRows);
    assert_eq!(err.params().get("count").map(String::as_str), Some("1"));

    assert_eq!(entry_count(&root), 1, "chi con dung tep .csv vua tao -- 0 .atproj nao duoc ghi");
    cleanup(&root);
}

#[test]
fn a_blank_target_cell_is_a_mismatch_of_one_versus_zero() {
    let root = temp_dir("blank-target");
    let csv = "Tieu de day.,\n";
    let path = write_file(&root, "trong.csv", csv.as_bytes());

    let shape = import_bilingual_file(&path).expect("tep hop le phai doc duoc");
    let preview = preview_bilingual_import(&shape, "en", &[], None, 0, 1, false).expect("xem truoc tep hop le phai thanh cong");
    let selected = preview.candidates.iter().find(|c| c.encoding == preview.selected_encoding).unwrap();
    assert_eq!(selected.mismatches.len(), 1);
    assert_eq!(selected.mismatches[0].source_sentence_count, 1);
    assert_eq!(selected.mismatches[0].target_sentence_count, 0);

    cleanup(&root);
}

// ═════════════════════════════════════════════════════════════════════════════════
// "Header checkbox on" — hang 1 vang mat khoi segment VA counts; toggle KHONG doc lai dia
// ═════════════════════════════════════════════════════════════════════════════════

#[test]
fn the_header_checkbox_drops_row_1_and_toggling_rebuilds_without_rereading_the_file() {
    let root = temp_dir("header");
    let csv = "source,target\nChi mot cau thoi.,Chi mot cau dich.\n";
    let path = write_file(&root, "header.csv", csv.as_bytes());

    let shape = import_bilingual_file(&path).expect("tep hop le phai doc duoc");

    let without_header = preview_bilingual_import(&shape, "en", &[], None, 0, 1, false).expect("xem truoc tep hop le phai thanh cong");
    let with_header = preview_bilingual_import(&shape, "en", &[], None, 0, 1, true).expect("xem truoc tep hop le phai thanh cong");
    // Cùng `shape` (byte thô không đổi) — chỉ đổi tham số `has_header` giữa hai lượt gọi, 0
    // lượt đọc đĩa thêm (§I/O Matrix: "rebuilds the preview in memory").
    assert_eq!(without_header.row_count, 2, "khong bo tieu de -- ca hai hang deu la du lieu");
    assert_eq!(with_header.row_count, 1, "bo tieu de -- chi con dung mot hang du lieu");

    let state = pending_state();
    stash_pending_import_source(&state, shape, None);
    let opened = confirm_bilingual_import(&root, &state, "Co Tieu De", "en", "", "UTF-8", Vec::new(), None, 0, 1,true)
        .expect("xac nhan phai thanh cong");
    let chapters = read_chapters(&opened.store);
    let segments = read_segments(&opened.store, chapters[0].id);
    assert_eq!(segments.len(), 1, "hang tieu de khong duoc di vao segment");
    assert_eq!(segments[0].source_text, "Chi mot cau thoi.");

    cleanup(&root);
}

// ═════════════════════════════════════════════════════════════════════════════════
// "Swap columns" — mau chay tren cot con lai; counts rebuild
// ═════════════════════════════════════════════════════════════════════════════════

#[test]
fn swapping_columns_runs_the_pattern_on_the_newly_chosen_source_column() {
    let root = temp_dir("swap");
    // Cột 0 = vai ĐÍCH (không khớp mẫu), cột 1 = vai NGUỒN sau khi đảo (khớp mẫu "CHUONG").
    let csv = "CHAPTER ONE,CHUONG MOT\nContent here.,Noi dung mot day.\n";
    let path = write_file(&root, "dao-cot.csv", csv.as_bytes());

    let shape = import_bilingual_file(&path).expect("tep hop le phai doc duoc");
    let state = pending_state();
    stash_pending_import_source(&state, shape, None);

    // source_column=1, target_column=0 -- vai đã đảo so với mặc định 0/1.
    let opened = confirm_bilingual_import(
        &root,
        &state,
        "Dao Cot",
        "en",
        "",
        "UTF-8",
        Vec::new(),
        Some(ChapterPattern::literal("CHUONG")),
        1,
        0,
        false,
    )
    .expect("xac nhan phai thanh cong");

    let chapters = read_chapters(&opened.store);
    assert_eq!(chapters.len(), 1, "mau khop hang dau tien cua cot NGUON MOI (cot 1)");
    assert_eq!(chapters[0].title.as_deref(), Some("CHUONG MOT"), "tieu de doc tu cot NGUON MOI");
    let segments = read_segments(&opened.store, chapters[0].id);
    assert_eq!(segments[0].source_text, "CHUONG MOT");
    assert_eq!(segments[0].target_text, "CHAPTER ONE");

    cleanup(&root);
}

// ═════════════════════════════════════════════════════════════════════════════════
// "Encoding change" — re-decoded, re-parsed; vai cot va co tieu de giu nguyen
// ═════════════════════════════════════════════════════════════════════════════════

#[test]
fn choosing_a_non_utf8_encoding_candidate_re_decodes_and_re_parses_the_same_stashed_bytes() {
    let root = temp_dir("gbk");
    let text = "你好。再见。,Xin chao. Tam biet.\n";
    let (encoded, _, had_errors) = encoding_rs::GBK.encode(text);
    assert!(!had_errors, "fixture phai ma hoa GBK sach, khong ky tu nao roi ra ngoai GBK");
    let path = write_file(&root, "gbk.csv", &encoded);

    let shape = import_bilingual_file(&path).expect("tep hop le phai doc duoc");
    let preview = preview_bilingual_import(&shape, "zh", &[], None, 0, 1, false).expect("xem truoc tep GBK hop le phai thanh cong");
    assert!(
        preview.candidates.iter().any(|c| c.encoding == "GBK"),
        "dai nam ung vien phai co GBK (mot trong nam bang ma FR126)"
    );

    let state = pending_state();
    stash_pending_import_source(&state, shape, None);
    let opened = confirm_bilingual_import(&root, &state, "GBK", "zh", "", "GBK", Vec::new(), None, 0, 1, false)
        .expect("xac nhan voi bang ma GBK da chon phai thanh cong");

    let chapters = read_chapters(&opened.store);
    let segments = read_segments(&opened.store, chapters[0].id);
    assert_eq!(segments.len(), 2, "hai cau tieng Trung (dau kết cau 。) khop hai cau dich");
    assert_eq!(segments[0].source_text, "你好。");
    assert_eq!(segments[1].source_text, "再见。");

    cleanup(&root);
}

// ═════════════════════════════════════════════════════════════════════════════════
// "Unterminated quote" — tu choi voi so hang, 0 Work
// ═════════════════════════════════════════════════════════════════════════════════

#[test]
fn an_unterminated_quoted_field_is_refused_with_its_row_number_and_writes_nothing() {
    let root = temp_dir("unterminated");
    let csv = "Hang mot on,Hang mot dich\n\"Hang hai chua dong,Hang hai dich\n";
    let path = write_file(&root, "ho-ngoac.csv", csv.as_bytes());

    let shape = import_bilingual_file(&path).expect("tep hop le phai doc duoc");
    let state = pending_state();
    stash_pending_import_source(&state, shape, None);

    let err = confirm_bilingual_import(&root, &state, "Ho Ngoac", "en", "", "UTF-8", Vec::new(), None, 0, 1,false)
        .expect_err("mot o mo ngoac kep khong dong phai bi tu choi");
    assert_eq!(err.message_key(), MessageKey::ImportBilingualUnterminatedQuotedField);
    assert_eq!(err.params().get("row").map(String::as_str), Some("2"));
    assert_eq!(entry_count(&root), 1, "chi con dung tep vua tao -- 0 .atproj nao duoc ghi");

    cleanup(&root);
}

// ═════════════════════════════════════════════════════════════════════════════════
// "Fewer than 2 columns" — tu choi TRUOC khi co gi de xem truoc
// ═════════════════════════════════════════════════════════════════════════════════

#[test]
fn a_file_with_fewer_than_two_columns_is_refused_before_anything_is_written() {
    let root = temp_dir("one-column");
    let tsv = "ChiMotCot\nHangHai\n";
    let path = write_file(&root, "mot-cot.tsv", tsv.as_bytes());

    let shape = import_bilingual_file(&path).expect("tep hop le phai doc duoc (loi thuoc ve TABLE PARSE, khong phai doc tep)");
    let state = pending_state();
    stash_pending_import_source(&state, shape, None);

    let err = confirm_bilingual_import(&root, &state, "Mot Cot", "en", "", "UTF-8", Vec::new(), None, 0, 1,false)
        .expect_err("tep chi co 1 cot phai bi tu choi");
    assert_eq!(err.message_key(), MessageKey::ImportBilingualTooFewColumns);
    assert_eq!(err.params().get("found").map(String::as_str), Some("1"));
    assert_eq!(entry_count(&root), 1);

    cleanup(&root);
}

// ═════════════════════════════════════════════════════════════════════════════════
// "Cancel" — 0 Work; override state cleared
// ═════════════════════════════════════════════════════════════════════════════════

#[test]
fn cancelling_the_preview_then_confirming_writes_nothing() {
    let root = temp_dir("cancel");
    let csv = "Mot cau thoi.,One sentence only.\n";
    let path = write_file(&root, "huy.csv", csv.as_bytes());

    let shape = import_bilingual_file(&path).expect("tep hop le phai doc duoc");
    let state = pending_state();
    stash_pending_import_source(&state, shape, None);
    cancel_import_preview(&state);

    let err = confirm_bilingual_import(&root, &state, "Huy Bo", "en", "", "UTF-8", Vec::new(), None, 0, 1,false)
        .expect_err("da huy thi xac nhan phai tu choi");
    assert_eq!(err.message_key(), MessageKey::ImportNoPendingSource);
    assert_eq!(entry_count(&root), 1, "chi con dung tep vua tao -- 0 .atproj nao duoc ghi");

    cleanup(&root);
}

// ═════════════════════════════════════════════════════════════════════════════════
// Ranh giới đuôi tệp — đường văn xuôi/song ngữ KHÔNG lấn sân nhau
// ═════════════════════════════════════════════════════════════════════════════════

#[test]
fn the_prose_import_path_still_rejects_csv_and_tsv_extensions() {
    let root = temp_dir("prose-boundary");
    let path = write_file(&root, "khong-phai-van-xuoi.csv", b"a,b\n");
    let err = auratranslate_lib::core::segment::import::import_file(&path)
        .expect_err(".csv khong duoc DUONG VAN XUOI nhan -- pham vi story 6.16 la mot duong RIENG");
    assert_eq!(err, auratranslate_lib::core::segment::import::ImportError::UnsupportedFormat { format: "csv".to_owned() });
    cleanup(&root);
}

/// §I/O Matrix "Prose `.md` — Existing non-bilingual flow — Byte-identical to today". Them o buoc
/// nghiem thu 2026-09-11: truoc ca nay KHONG ca nao nhap mot tep `.md` van xuoi qua
/// `import_file`. Tep co chua mot bang pipe — dung hinh dang ma mot duong song ngu `.md` (da
/// hoan, chu Ice) se doc — van phai di duong van xuoi: `Blob`, Chuong `not_started`, 0 ban dich,
/// 0 xuat xu, chu trong bang nam nguyen o phia NGUON.
#[test]
fn a_prose_md_file_containing_a_pipe_table_still_imports_as_prose_with_no_translation() {
    let root = temp_dir("prose-md");
    let md = "Mot doan van xuoi.\n\n| Cot A | Cot B |\n|---|---|\n| nguon | dich |\n";
    let path = write_file(&root, "van-xuoi.md", md.as_bytes());

    let (shape, sidecar) = auratranslate_lib::core::segment::import::import_file(&path)
        .expect(".md van xuoi phai doc duoc tren duong van xuoi");
    assert!(
        matches!(shape, auratranslate_lib::core::segment::pipeline::PipelineShape::Blob(_)),
        ".md van xuoi phai la Blob, khong phai Bilingual"
    );
    assert!(sidecar.is_none());

    let opened = auratranslate_lib::commands::project::create_work_from_file(&root, "Van Xuoi Md", "en", "", &path)
        .expect("tao Tac pham tu .md van xuoi phai thanh cong");
    let chapters = read_chapters(&opened.store);
    assert_eq!(chapters.len(), 1);
    assert_eq!(chapters[0].status, "not_started", "Chuong van xuoi giu NotStarted -- InProgress chi cho duong song ngu");

    let segments = read_segments(&opened.store, chapters[0].id);
    assert!(!segments.is_empty());
    for s in &segments {
        assert_eq!(s.target_text, "", "duong van xuoi khong bao gio ghi ban dich luc nhap");
        assert_eq!(s.translation_origin, "", "duong van xuoi khong bao gio ghi xuat xu luc nhap");
    }
    let source: String = segments.iter().map(|s| s.source_text.as_str()).collect::<Vec<_>>().join(" ");
    assert!(source.contains("nguon | dich"), "o bang pipe phai nam nguyen o phia NGUON, khong bi tach cot: {source:?}");

    cleanup(&root);
}

// ═════════════════════════════════════════════════════════════════════════════════
// Hình dạng dây — cùng khuôn segment_contract.rs::the_import_encoding_preview_wire_shape_keeps_snake_case_field_names
// ═════════════════════════════════════════════════════════════════════════════════

#[test]
fn the_bilingual_import_encoding_preview_wire_shape_keeps_snake_case_field_names() {
    use auratranslate_lib::commands::project::{
        BilingualEncodingCandidateWire, BilingualImportEncodingPreview, BilingualMismatchWire,
    };

    let preview = BilingualImportEncodingPreview {
        confidence: auratranslate_lib::commands::project::ConfidenceWire::High,
        selected_encoding: "UTF-8".to_owned(),
        candidates: vec![BilingualEncodingCandidateWire {
            label: "UTF-8".to_owned(),
            encoding: "UTF-8".to_owned(),
            preview: Some("a,b".to_owned()),
            row_count: 1,
            chapter_count: 1,
            pair_count: 0,
            mismatches: vec![BilingualMismatchWire {
                chapter_index: 0,
                row_number: 1,
                source_sentence_count: 2,
                target_sentence_count: 1,
            }],
        }],
        sample_rows: vec![vec!["a".to_owned(), "b".to_owned()]],
        row_count: 1,
        column_count: 2,
    };

    let json = serde_json::to_value(&preview).expect("serialize BilingualImportEncodingPreview");
    let object = json.as_object().expect("phai serialize thanh object");
    assert_eq!(
        object.keys().collect::<std::collections::BTreeSet<_>>(),
        std::collections::BTreeSet::from([
            &"confidence".to_owned(),
            &"selected_encoding".to_owned(),
            &"candidates".to_owned(),
            &"sample_rows".to_owned(),
            &"row_count".to_owned(),
            &"column_count".to_owned(),
        ]),
        "src/config/project.ts::BilingualImportEncodingPreview doc dung tung ten truong nay"
    );

    let candidate_json = &json["candidates"][0];
    let candidate_object = candidate_json.as_object().expect("candidate phai serialize thanh object");
    assert_eq!(
        candidate_object.keys().collect::<std::collections::BTreeSet<_>>(),
        std::collections::BTreeSet::from([
            &"label".to_owned(),
            &"encoding".to_owned(),
            &"preview".to_owned(),
            &"row_count".to_owned(),
            &"chapter_count".to_owned(),
            &"pair_count".to_owned(),
            &"mismatches".to_owned(),
        ]),
        "src/config/project.ts::BilingualEncodingCandidateWire doc dung tung ten truong nay"
    );

    let mismatch_json = &candidate_json["mismatches"][0];
    let mismatch_object = mismatch_json.as_object().expect("mismatch phai serialize thanh object");
    assert_eq!(
        mismatch_object.keys().collect::<std::collections::BTreeSet<_>>(),
        std::collections::BTreeSet::from([
            &"chapter_index".to_owned(),
            &"row_number".to_owned(),
            &"source_sentence_count".to_owned(),
            &"target_sentence_count".to_owned(),
        ]),
        "src/config/project.ts::BilingualMismatchWire doc dung tung ten truong nay"
    );
}

/// Trần 100 MB [`MAX_IMPORT_BYTES` phía Rust] — và trần đó phải chặn TRƯỚC KHI đọc, cùng
/// khuôn `project_contract.rs::a_file_past_the_size_ceiling_is_refused_before_a_single_byte_is_written`.
/// 🔴 THÊM (vòng rà đối kháng) — trước ca này, không ca nào của bộ hợp đồng này chạm nhánh
/// `size > MAX_IMPORT_BYTES` của `import_bilingual_file`; xoá guard đó vẫn XANH.
#[test]
fn a_bilingual_file_past_the_size_ceiling_is_refused_before_a_single_byte_is_read() {
    let root = temp_dir("bilingual-too-large");

    // ⚠️ Không ghi 100 MB thật ra đĩa trong một test — dựng một tệp THƯA (sparse): `set_len`
    // khai kích thước mà không cấp phát khối nào.
    let path = root.join("khong-lo.csv");
    let file = fs::File::create(&path).unwrap_or_else(|e| panic!("tao {}: {e}", path.display()));
    file.set_len(100 * 1024 * 1024 + 1).expect("set_len that bai");
    drop(file);

    let err = import_bilingual_file(&path).expect_err("tep vuot tran phai bi tu choi");
    assert!(
        matches!(err, auratranslate_lib::core::segment::import::ImportError::TooLarge { size, limit } if size == 100 * 1024 * 1024 + 1 && limit == 100 * 1024 * 1024),
        "ky vong TooLarge voi dung size/limit, nhan: {err:?}"
    );

    cleanup(&root);
}

#[test]
fn the_bilingual_import_path_only_accepts_csv_and_tsv() {
    let root = temp_dir("bilingual-boundary");
    let path = write_file(&root, "khong-phai-song-ngu.txt", b"noi dung van xuoi");
    let err = import_bilingual_file(&path).expect_err(".txt khong duoc DUONG SONG NGU nhan");
    assert_eq!(
        err,
        auratranslate_lib::core::segment::import::ImportError::BilingualUnsupportedFormat { format: "txt".to_owned() }
    );
    cleanup(&root);
}

// ═════════════════════════════════════════════════════════════════════════════════
// "Fewer than 2 columns" / "Unterminated quote" — TU CHOI O MAN XEM TRUOC, khong chi o xac nhan
// ═════════════════════════════════════════════════════════════════════════════════
//
// 🔵 THEM 2026-09-11 (buoc nghiem thu spec 6.16) — ban dau `preview_bilingual_import` nuot moi
// `Err` cua chuoi bang `.ok()`: man xem truoc hien toan so 0, nut xac nhan BAT, loi chi lo ra
// khi bam xac nhan. Hai ca xac nhan o tren khong do duoc dieu do.

#[test]
fn preview_refuses_a_one_column_file_before_showing_anything() {
    let root = temp_dir("preview-one-column");
    let path = write_file(&root, "mot-cot-xem-truoc.tsv", b"ChiMotCot\nHangHai\n");

    let shape = import_bilingual_file(&path).expect("doc tep khong table-parse, phai thanh cong");
    let err = preview_bilingual_import(&shape, "en", &[], None, 0, 1, false)
        .expect_err("tep mot cot phai bi tu choi NGAY o man xem truoc");
    assert_eq!(err.message_key(), MessageKey::ImportBilingualTooFewColumns);
    assert_eq!(err.params().get("found").map(String::as_str), Some("1"));

    cleanup(&root);
}

#[test]
fn preview_refuses_an_unterminated_quoted_field_with_its_row_number() {
    let root = temp_dir("preview-unterminated");
    let csv = "Hang mot on,Hang mot dich\n\"Hang hai chua dong,Hang hai dich\n";
    let path = write_file(&root, "ho-ngoac-xem-truoc.csv", csv.as_bytes());

    let shape = import_bilingual_file(&path).expect("doc tep khong table-parse, phai thanh cong");
    let err = preview_bilingual_import(&shape, "en", &[], None, 0, 1, false)
        .expect_err("o mo ngoac kep khong dong phai bi tu choi NGAY o man xem truoc");
    assert_eq!(err.message_key(), MessageKey::ImportBilingualUnterminatedQuotedField);
    assert_eq!(err.params().get("row").map(String::as_str), Some("2"));

    cleanup(&root);
}

// ═════════════════════════════════════════════════════════════════════════════════
// §Always — "Cleanup and normalize run per cell, both columns": luat DA BAT phai toi duoc
// ca xem truoc lan xac nhan, tren CA HAI o
// ═════════════════════════════════════════════════════════════════════════════════

#[test]
fn an_enabled_cleanup_rule_runs_on_both_cells_in_preview_and_on_disk() {
    let root = temp_dir("cleanup-both-cells");
    let csv = "Mot cau day [QC].,One sentence here [QC].\n";
    let path = write_file(&root, "lam-sach.csv", csv.as_bytes());
    let rule = CleanupRule {
        tier: CleanupRuleTier::Global,
        id: 1,
        pattern: "[QC]".to_owned(),
        kind: CleanupRuleKind::Literal,
        enabled: true,
    };

    let shape = import_bilingual_file(&path).expect("tep hop le phai doc duoc");
    let preview = preview_bilingual_import(&shape, "en", std::slice::from_ref(&rule), None, 0, 1, false)
        .expect("xem truoc tep hop le phai thanh cong");
    let selected = preview.candidates.iter().find(|c| c.encoding == preview.selected_encoding).unwrap();
    assert!(selected.mismatches.is_empty(), "luat lam sach khong duoc lam lech cap hang nay");

    let state = pending_state();
    stash_pending_import_source(&state, shape, None);
    let opened = confirm_bilingual_import(&root, &state, "Lam Sach", "en", "", "UTF-8", vec![rule], None, 0, 1, false)
        .expect("xac nhan phai thanh cong");

    let chapters = read_chapters(&opened.store);
    let segments = read_segments(&opened.store, chapters[0].id);
    assert_eq!(segments.len(), 1);
    assert!(!segments[0].source_text.contains("[QC]"), "o NGUON phai qua luat: {:?}", segments[0].source_text);
    assert!(!segments[0].target_text.contains("[QC]"), "o DICH phai qua luat: {:?}", segments[0].target_text);

    cleanup(&root);
}
