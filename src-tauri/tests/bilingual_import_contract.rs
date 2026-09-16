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
    BilingualMismatchWire, BilingualRegroupingWire, PendingImportSourceState, ReviewCauseWire,
    cancel_import_preview, confirm_bilingual_import, preview_bilingual_import,
    stash_pending_import_source,
};
use auratranslate_lib::core::cleanup::{CleanupRule, CleanupRuleKind, CleanupRuleTier};
use auratranslate_lib::core::i18n::MessageKey;
use auratranslate_lib::core::segment::bilingual::{BilingualRegrouping, BilingualRegroupingAction};
use auratranslate_lib::core::segment::chapterpattern::ChapterPattern;
use auratranslate_lib::core::segment::import::import_bilingual_file;
use auratranslate_lib::core::store::Store;

/// Dựng một [`BilingualRegrouping`] `Cuts` bằng cách ECHO NGUYÊN VẸN ảnh chụp của một mismatch
/// TRÊN DÂY — đúng cách webview làm (không tự đoán lại `source_sentences`/`target_line`).
fn cuts_regrouping(m: &BilingualMismatchWire, cuts: Vec<usize>) -> BilingualRegrouping {
    BilingualRegrouping {
        row_number: m.row_number,
        source_sentences: m.source_sentences.clone(),
        target_line: m.target_line.clone(),
        action: BilingualRegroupingAction::Cuts(cuts),
    }
}

fn skip_regrouping(m: &BilingualMismatchWire) -> BilingualRegrouping {
    BilingualRegrouping {
        row_number: m.row_number,
        source_sentences: m.source_sentences.clone(),
        target_line: m.target_line.clone(),
        action: BilingualRegroupingAction::Skip,
    }
}

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
    let preview = preview_bilingual_import(&shape, "en", &[], Some(&pattern), 0, 1, false, &[])
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
        Vec::new(),
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

    let opened = confirm_bilingual_import(&root, &state, "Hai Cau", "en", "", "UTF-8", Vec::new(), None, 0, 1,false, Vec::new())
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

    let opened = confirm_bilingual_import(&root, &state, "Co Doan", "en", "", "UTF-8", Vec::new(), None, 0, 1,false, Vec::new())
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

    let opened = confirm_bilingual_import(&root, &state, "Xuong Dong", "en", "", "UTF-8", Vec::new(), None, 0, 1,false, Vec::new())
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

    let preview = preview_bilingual_import(&shape, "en", &[], None, 0, 1, false, &[]).expect("xem truoc tep hop le phai thanh cong");
    let selected = preview
        .candidates
        .iter()
        .find(|c| c.encoding == preview.selected_encoding)
        .expect("ung vien dang chon phai co mat trong dai");
    assert_eq!(selected.mismatches.len(), 1, "preview phai liet ke dung MOT hang lech cap");
    assert_eq!(selected.mismatches[0].row_number, 1);
    assert_eq!(selected.mismatches[0].source_sentences.len(), 2);
    assert_eq!(selected.mismatches[0].target_line, "Chi mot cau thoi.");

    let state = pending_state();
    stash_pending_import_source(&state, shape, None);
    let err = confirm_bilingual_import(&root, &state, "Lech Cap", "en", "", "UTF-8", Vec::new(), None, 0, 1,false, Vec::new())
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
    let preview = preview_bilingual_import(&shape, "en", &[], None, 0, 1, false, &[]).expect("xem truoc tep hop le phai thanh cong");
    let selected = preview.candidates.iter().find(|c| c.encoding == preview.selected_encoding).unwrap();
    assert_eq!(selected.mismatches.len(), 1);
    assert_eq!(selected.mismatches[0].source_sentences.len(), 1);
    assert_eq!(selected.mismatches[0].target_line, "");

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

    let without_header = preview_bilingual_import(&shape, "en", &[], None, 0, 1, false, &[]).expect("xem truoc tep hop le phai thanh cong");
    let with_header = preview_bilingual_import(&shape, "en", &[], None, 0, 1, true, &[]).expect("xem truoc tep hop le phai thanh cong");
    // Cùng `shape` (byte thô không đổi) — chỉ đổi tham số `has_header` giữa hai lượt gọi, 0
    // lượt đọc đĩa thêm (§I/O Matrix: "rebuilds the preview in memory").
    assert_eq!(without_header.row_count, 2, "khong bo tieu de -- ca hai hang deu la du lieu");
    assert_eq!(with_header.row_count, 1, "bo tieu de -- chi con dung mot hang du lieu");

    let state = pending_state();
    stash_pending_import_source(&state, shape, None);
    let opened = confirm_bilingual_import(&root, &state, "Co Tieu De", "en", "", "UTF-8", Vec::new(), None, 0, 1,true, Vec::new())
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
        Vec::new(),
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
    let preview = preview_bilingual_import(&shape, "zh", &[], None, 0, 1, false, &[]).expect("xem truoc tep GBK hop le phai thanh cong");
    assert!(
        preview.candidates.iter().any(|c| c.encoding == "GBK"),
        "dai nam ung vien phai co GBK (mot trong nam bang ma FR126)"
    );

    let state = pending_state();
    stash_pending_import_source(&state, shape, None);
    let opened = confirm_bilingual_import(&root, &state, "GBK", "zh", "", "GBK", Vec::new(), None, 0, 1, false, Vec::new())
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

    let err = confirm_bilingual_import(&root, &state, "Ho Ngoac", "en", "", "UTF-8", Vec::new(), None, 0, 1,false, Vec::new())
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

    let err = confirm_bilingual_import(&root, &state, "Mot Cot", "en", "", "UTF-8", Vec::new(), None, 0, 1,false, Vec::new())
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

    let err = confirm_bilingual_import(&root, &state, "Huy Bo", "en", "", "UTF-8", Vec::new(), None, 0, 1,false, Vec::new())
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
            skipped_target_sentence_count: 0,
            mismatches: vec![BilingualMismatchWire {
                chapter_index: 0,
                row_number: 1,
                source_sentences: vec!["Hai cau.".to_owned(), "Cau hai nua.".to_owned()],
                target_line: "Chi mot cau.".to_owned(),
                target_sentence_count: 1,
                candidate_positions: vec![4],
                initial_cuts: vec![],
                proposed_cuts: vec![4],
            }],
            chapters: None,
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
            &"skipped_target_sentence_count".to_owned(),
            &"mismatches".to_owned(),
            &"chapters".to_owned(),
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
            &"source_sentences".to_owned(),
            &"target_line".to_owned(),
            &"target_sentence_count".to_owned(),
            &"candidate_positions".to_owned(),
            &"initial_cuts".to_owned(),
            &"proposed_cuts".to_owned(),
        ]),
        "src/config/project.ts::BilingualMismatchWire doc dung tung ten truong nay"
    );
}

/// 🔴 THÊM (vòng rà đối kháng) — không ca nào của bộ hợp đồng này từng GIẢI MÃ
/// `BilingualRegroupingWire` từ JSON; mọi ca khác dựng nó bằng struct literal Rust, thứ
/// không đi qua serde nên không canh được `#[serde(rename = "cuts"/"skip")]` có khớp đúng
/// hai chuỗi mà `src/config/project.ts::BilingualRegroupingInput.kind` gửi lên hay không.
#[test]
fn bilingual_regrouping_wire_decodes_the_two_kind_strings_the_frontend_sends() {
    let cuts_json = r#"{"row_number":3,"source_sentences":["One.","Two."],"target_line":"Mot hai","kind":"cuts","cuts":[3]}"#;
    let cuts_wire: BilingualRegroupingWire =
        serde_json::from_str(cuts_json).expect("giai ma kind=\"cuts\" phai thanh cong");
    let cuts_core: auratranslate_lib::core::segment::bilingual::BilingualRegrouping = cuts_wire.into();
    assert_eq!(cuts_core.row_number, 3);
    assert_eq!(cuts_core.action, BilingualRegroupingAction::Cuts(vec![3]));

    let skip_json = r#"{"row_number":4,"source_sentences":[],"target_line":"con day","kind":"skip","cuts":[]}"#;
    let skip_wire: BilingualRegroupingWire =
        serde_json::from_str(skip_json).expect("giai ma kind=\"skip\" phai thanh cong");
    let skip_core: auratranslate_lib::core::segment::bilingual::BilingualRegrouping = skip_wire.into();
    assert_eq!(skip_core.row_number, 4);
    assert_eq!(skip_core.action, BilingualRegroupingAction::Skip);
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
    let err = preview_bilingual_import(&shape, "en", &[], None, 0, 1, false, &[])
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
    let err = preview_bilingual_import(&shape, "en", &[], None, 0, 1, false, &[])
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
    let preview = preview_bilingual_import(&shape, "en", std::slice::from_ref(&rule), None, 0, 1, false, &[])
        .expect("xem truoc tep hop le phai thanh cong");
    let selected = preview.candidates.iter().find(|c| c.encoding == preview.selected_encoding).unwrap();
    assert!(selected.mismatches.is_empty(), "luat lam sach khong duoc lam lech cap hang nay");

    let state = pending_state();
    stash_pending_import_source(&state, shape, None);
    let opened = confirm_bilingual_import(&root, &state, "Lam Sach", "en", "", "UTF-8", vec![rule], None, 0, 1, false, Vec::new())
        .expect("xac nhan phai thanh cong");

    let chapters = read_chapters(&opened.store);
    let segments = read_segments(&opened.store, chapters[0].id);
    assert_eq!(segments.len(), 1);
    assert!(!segments[0].source_text.contains("[QC]"), "o NGUON phai qua luat: {:?}", segments[0].source_text);
    assert!(!segments[0].target_text.contains("[QC]"), "o DICH phai qua luat: {:?}", segments[0].target_text);

    cleanup(&root);
}

// ═════════════════════════════════════════════════════════════════════════════════
// Story 6.17 (FR116) — quy nhóm câu đích trong từng hàng lệch cặp, trên ĐÚNG đường sản
// phẩm (`preview_bilingual_import` → `confirm_bilingual_import` → `create_work`). Mỗi hàng
// của §I/O & Edge-Case Matrix có mặt ở đây.
// ═════════════════════════════════════════════════════════════════════════════════

/// "Split target" — Source 2, target 1; một chỗ cắt tại ranh giới từ. Row resolved; 2
/// segment, cả hai văn bản, cờ off/on. Cũng khoá "Proposal shown": đề xuất máy không rỗng và
/// TỰ NÓ giải quyết được hàng (không cần người dùng chỉnh tay).
#[test]
fn split_target_resolves_using_the_machine_proposal_and_writes_two_paired_segments() {
    let root = temp_dir("6-17-split");
    let csv = "He left. She smiled.,He left she smiled\n";
    let path = write_file(&root, "tach.csv", csv.as_bytes());
    let shape = import_bilingual_file(&path).expect("tep hop le phai doc duoc");

    let preview = preview_bilingual_import(&shape, "en", &[], None, 0, 1, false, &[]).expect("xem truoc phai thanh cong");
    let selected = preview.candidates.iter().find(|c| c.encoding == preview.selected_encoding).unwrap();
    assert_eq!(selected.mismatches.len(), 1);
    let m = &selected.mismatches[0];
    assert_eq!(m.source_sentences, vec!["He left.".to_owned(), "She smiled.".to_owned()]);
    assert!(!m.proposed_cuts.is_empty(), "2 vs 1 phai co it nhat mot cho cat de xuat");

    let regrouping = cuts_regrouping(m, m.proposed_cuts.clone());
    let rebuilt = preview_bilingual_import(&shape, "en", &[], None, 0, 1, false, std::slice::from_ref(&regrouping))
        .expect("xem truoc voi quy nhom phai thanh cong");
    let rebuilt_selected = rebuilt.candidates.iter().find(|c| c.encoding == rebuilt.selected_encoding).unwrap();
    assert!(rebuilt_selected.mismatches.is_empty(), "de xuat may phai tu giai quyet duoc hang nay");

    let state = pending_state();
    stash_pending_import_source(&state, shape, None);
    let opened = confirm_bilingual_import(&root, &state, "Tach", "en", "", "UTF-8", Vec::new(), None, 0, 1, false, vec![
        regrouping,
    ])
    .expect("xac nhan voi quy nhom hop le phai thanh cong");

    let chapters = read_chapters(&opened.store);
    let segments = read_segments(&opened.store, chapters[0].id);
    assert_eq!(segments.len(), 2);
    assert_eq!(segments[0].source_text, "He left.");
    assert_eq!(segments[1].source_text, "She smiled.");
    assert!(!segments[0].target_text.is_empty());
    assert!(!segments[1].target_text.is_empty());
    assert_eq!(
        segments.iter().map(|s| s.is_paragraph_end).collect::<Vec<_>>(),
        vec![false, false],
        "hang DUY NHAT cua Chuong -- ca hai cot deu tat (AD-37 ca cuoi Chuong)"
    );
    for s in &segments {
        assert_eq!(s.translation_origin, "bilingual_import");
        assert_eq!(s.status, "draft");
    }

    cleanup(&root);
}

/// "Join target" — Source 2, target 3; bỏ ranh giới t1/t2. 2 segment; mảnh đích đầu = "t1 t2".
#[test]
fn join_target_removes_a_machine_boundary_and_merges_the_first_two_target_sentences() {
    let root = temp_dir("6-17-join");
    let csv = "One. Two.,Mot. Hai. Ba.\n";
    let path = write_file(&root, "gop.csv", csv.as_bytes());
    let shape = import_bilingual_file(&path).expect("tep hop le phai doc duoc");

    let preview = preview_bilingual_import(&shape, "en", &[], None, 0, 1, false, &[]).expect("xem truoc phai thanh cong");
    let selected = preview.candidates.iter().find(|c| c.encoding == preview.selected_encoding).unwrap();
    let m = &selected.mismatches[0];
    assert_eq!(m.initial_cuts.len(), 2, "3 cau may = 2 ranh gioi may");
    // Giu chi CHO CAT THU HAI -- gop "Mot." + "Hai." thanh manh dau.
    let kept_cut = m.initial_cuts[1];

    let state = pending_state();
    stash_pending_import_source(&state, shape, None);
    let regrouping = cuts_regrouping(m, vec![kept_cut]);
    let opened = confirm_bilingual_import(&root, &state, "Gop", "en", "", "UTF-8", Vec::new(), None, 0, 1, false, vec![
        regrouping,
    ])
    .expect("xac nhan voi quy nhom hop le phai thanh cong");

    let chapters = read_chapters(&opened.store);
    let segments = read_segments(&opened.store, chapters[0].id);
    assert_eq!(segments.len(), 2);
    assert_eq!(segments[0].source_text, "One.");
    assert_eq!(segments[0].target_text, "Mot. Hai.");
    assert_eq!(segments[1].source_text, "Two.");
    assert_eq!(segments[1].target_text, "Ba.");

    cleanup(&root);
}

/// "Still unequal" — Source 2, target pieces 3 sau khi sửa (cắt sai số lượng). Row vẫn bị
/// liệt kê; confirm bị từ chối Ở RUST, 0 Work.
#[test]
fn still_unequal_after_a_bad_cut_count_keeps_the_row_listed_and_confirm_refused() {
    let root = temp_dir("6-17-still-unequal");
    let csv = "One. Two.,Mot. Hai. Ba.\n";
    let path = write_file(&root, "van-lech.csv", csv.as_bytes());
    let shape = import_bilingual_file(&path).expect("tep hop le phai doc duoc");
    let preview = preview_bilingual_import(&shape, "en", &[], None, 0, 1, false, &[]).expect("xem truoc phai thanh cong");
    let selected = preview.candidates.iter().find(|c| c.encoding == preview.selected_encoding).unwrap();
    let m = &selected.mismatches[0];

    // 0 cho cat -- van 3 manh may, khong ve dung 2 nhu nguon can.
    let regrouping = cuts_regrouping(m, Vec::new());
    let state = pending_state();
    stash_pending_import_source(&state, shape, None);
    let err = confirm_bilingual_import(&root, &state, "Van Lech", "en", "", "UTF-8", Vec::new(), None, 0, 1, false, vec![
        regrouping,
    ])
    .expect_err("cat khong dung so manh can phai bi tu choi");
    assert_eq!(err.message_key(), MessageKey::ImportBilingualMismatchedRows);
    assert_eq!(entry_count(&root), 1, "0 Work nao duoc ghi");

    cleanup(&root);
}

/// "Bad cut on wire" — 0 / = độ dài / trùng nhau / vượt cuối, không panic; hàng vẫn bị coi là
/// mismatch (không ghi đè oan lên dữ liệu).
#[test]
fn bad_cuts_on_the_wire_never_panic_and_leave_the_row_a_mismatch() {
    let root = temp_dir("6-17-bad-cut");
    let csv = "He left. She smiled.,He left she smiled\n";
    let path = write_file(&root, "cat-hong.csv", csv.as_bytes());
    let shape = import_bilingual_file(&path).expect("tep hop le phai doc duoc");
    let preview = preview_bilingual_import(&shape, "en", &[], None, 0, 1, false, &[]).expect("xem truoc phai thanh cong");
    let selected = preview.candidates.iter().find(|c| c.encoding == preview.selected_encoding).unwrap();
    let m = selected.mismatches[0].clone();

    for bad in [vec![0usize], vec![1000usize], vec![3usize, 3usize]] {
        let regrouping = cuts_regrouping(&m, bad.clone());
        let rebuilt = preview_bilingual_import(&shape, "en", &[], None, 0, 1, false, std::slice::from_ref(&regrouping))
            .unwrap_or_else(|e| panic!("cho cat hong {bad:?} khong duoc lam ca chuoi panic/tu choi: {e:?}"));
        let rebuilt_selected = rebuilt.candidates.iter().find(|c| c.encoding == rebuilt.selected_encoding).unwrap();
        assert_eq!(rebuilt_selected.mismatches.len(), 1, "cho cat hong {bad:?} phai giu hang lai trong danh sach");
    }

    cleanup(&root);
}

/// "Stale regrouping" — cuts làm trên cặp cột (nguồn, đích) = (0, 1); đảo cột GIỮA lúc xem
/// trước và lúc xác nhận thay đổi (source_cell, target_cell) của hàng dưới CÙNG `row_number`
/// ⇒ quy nhóm cũ không khớp ảnh chụp nữa, hàng bị liệt kê lại, 0 cắt nào bị áp oan.
#[test]
fn a_stale_regrouping_after_a_column_swap_is_dropped_and_the_row_is_listed_again() {
    let root = temp_dir("6-17-stale");
    let csv = "He left. She smiled.,He left she smiled\n";
    let path = write_file(&root, "cu.csv", csv.as_bytes());
    let shape = import_bilingual_file(&path).expect("tep hop le phai doc duoc");
    let preview = preview_bilingual_import(&shape, "en", &[], None, 0, 1, false, &[]).expect("xem truoc phai thanh cong");
    let selected = preview.candidates.iter().find(|c| c.encoding == preview.selected_encoding).unwrap();
    let m = &selected.mismatches[0];
    let regrouping = cuts_regrouping(m, m.proposed_cuts.clone());

    // Doi vai cot (1, 0) -- CUNG quy nhom (anh chup cua cot 0/1 cu) khong con khop hang thuc su
    // dang doc voi vai cot MOI.
    let rebuilt = preview_bilingual_import(&shape, "en", &[], None, 1, 0, false, std::slice::from_ref(&regrouping))
        .expect("xem truoc phai thanh cong");
    let rebuilt_selected = rebuilt.candidates.iter().find(|c| c.encoding == rebuilt.selected_encoding).unwrap();
    assert_eq!(rebuilt_selected.mismatches.len(), 1, "anh chup cu khong duoc ap len hang da doi vai cot");

    cleanup(&root);
}

/// "Unaffected regrouping" — quy nhóm của MỘT hàng không bị xoá bởi việc một hàng KHÁC vẫn
/// còn lệch cặp trong CÙNG lượt xác nhận: hàng A cặp được, hàng B (0 câu nguồn) không có quy
/// nhóm vẫn ở lại danh sách, xác nhận vẫn bị khoá cho tới khi CẢ HAI được giải quyết.
#[test]
fn a_regrouping_for_one_row_does_not_disturb_a_second_untouched_mismatched_row() {
    let root = temp_dir("6-17-unaffected");
    let csv = "He left. She smiled.,He left she smiled\n,Con mo cot nguon day\n";
    let path = write_file(&root, "hai-hang.csv", csv.as_bytes());
    let shape = import_bilingual_file(&path).expect("tep hop le phai doc duoc");
    let preview = preview_bilingual_import(&shape, "en", &[], None, 0, 1, false, &[]).expect("xem truoc phai thanh cong");
    let selected = preview.candidates.iter().find(|c| c.encoding == preview.selected_encoding).unwrap();
    assert_eq!(selected.mismatches.len(), 2, "ca hai hang phai lech cap");
    let row_a = selected.mismatches.iter().find(|m| !m.source_sentences.is_empty()).unwrap();
    let row_b = selected.mismatches.iter().find(|m| m.source_sentences.is_empty()).unwrap();

    let regrouping_a = cuts_regrouping(row_a, row_a.proposed_cuts.clone());
    let rebuilt = preview_bilingual_import(&shape, "en", &[], None, 0, 1, false, std::slice::from_ref(&regrouping_a))
        .expect("xem truoc phai thanh cong");
    let rebuilt_selected = rebuilt.candidates.iter().find(|c| c.encoding == rebuilt.selected_encoding).unwrap();
    assert_eq!(rebuilt_selected.mismatches.len(), 1, "hang A cap duoc, hang B (chua co quy nhom) van con lai");
    assert_eq!(rebuilt_selected.mismatches[0].row_number, row_b.row_number);

    let state = pending_state();
    stash_pending_import_source(&state, shape, None);
    let err = confirm_bilingual_import(&root, &state, "Chua Xong", "en", "", "UTF-8", Vec::new(), None, 0, 1, false, vec![
        regrouping_a,
    ])
    .expect_err("con hang B chua giai quyet thi xac nhan phai bi khoa");
    assert_eq!(err.message_key(), MessageKey::ImportBilingualMismatchedRows);
    assert_eq!(err.params().get("count").map(String::as_str), Some("1"));

    cleanup(&root);
}

/// "Bulk accept" — 3 hàng có đề xuất, cộng 1 hàng 1-vs-0 (chỉ giải quyết được bằng Skip): MỘT
/// lượt xác nhận với quy nhóm cho CẢ BỐN hàng phải cặp/giải quyết TRỌN, ghi đúng số segment.
#[test]
fn bulk_accept_resolves_three_proposal_rows_and_a_fourth_skip_only_row_in_one_confirm() {
    let root = temp_dir("6-17-bulk-accept");
    let csv = "A1. A2.,a1 a2\nB1. B2.,b1 b2\nC1. C2.,c1 c2\nHeading Day.,\n";
    let path = write_file(&root, "bulk.csv", csv.as_bytes());
    let shape = import_bilingual_file(&path).expect("tep hop le phai doc duoc");
    let preview = preview_bilingual_import(&shape, "en", &[], None, 0, 1, false, &[]).expect("xem truoc phai thanh cong");
    let selected = preview.candidates.iter().find(|c| c.encoding == preview.selected_encoding).unwrap();
    assert_eq!(selected.mismatches.len(), 4);

    let regroupings: Vec<BilingualRegrouping> = selected
        .mismatches
        .iter()
        .map(|m| {
            if m.target_line.is_empty() {
                skip_regrouping(m)
            } else {
                cuts_regrouping(m, m.proposed_cuts.clone())
            }
        })
        .collect();

    let state = pending_state();
    stash_pending_import_source(&state, shape, None);
    let opened = confirm_bilingual_import(&root, &state, "Bulk", "en", "", "UTF-8", Vec::new(), None, 0, 1, false, regroupings)
        .expect("mot lot xac nhan voi quy nhom cho ca bon hang phai thanh cong");

    let chapters = read_chapters(&opened.store);
    let segments = read_segments(&opened.store, chapters[0].id);
    // 2 + 2 + 2 (ba hang cat duoc) + 1 (hang tieu de, bo qua -- 1 segment chua dich).
    assert_eq!(segments.len(), 7);
    let heading = segments.iter().find(|s| s.source_text == "Heading Day.").expect("segment tieu de phai co mat");
    assert!(heading.target_text.is_empty());
    assert_eq!(heading.translation_origin, "", "segment chua dich khong duoc mang origin bilingual_import");

    cleanup(&root);
}

/// "Skip blank target" — Heading row 1-vs-0, skipped: 1 segment chưa dịch, target rỗng,
/// KHÔNG origin `bilingual_import`; hàng resolved.
#[test]
fn skip_on_a_blank_target_writes_one_untranslated_segment_with_no_bilingual_import_origin() {
    let root = temp_dir("6-17-skip-target");
    let csv = "Chuong Mot,\n";
    let path = write_file(&root, "tieu-de.csv", csv.as_bytes());
    let shape = import_bilingual_file(&path).expect("tep hop le phai doc duoc");
    let preview = preview_bilingual_import(&shape, "en", &[], None, 0, 1, false, &[]).expect("xem truoc phai thanh cong");
    let selected = preview.candidates.iter().find(|c| c.encoding == preview.selected_encoding).unwrap();
    let m = &selected.mismatches[0];
    assert_eq!(m.source_sentences, vec!["Chuong Mot".to_owned()]);
    assert_eq!(m.target_line, "");

    let state = pending_state();
    stash_pending_import_source(&state, shape, None);
    let opened =
        confirm_bilingual_import(&root, &state, "Tieu De", "en", "", "UTF-8", Vec::new(), None, 0, 1, false, vec![
            skip_regrouping(m),
        ])
        .expect("bo qua mot hang 1-vs-0 phai thanh cong");

    let chapters = read_chapters(&opened.store);
    let segments = read_segments(&opened.store, chapters[0].id);
    assert_eq!(segments.len(), 1);
    assert_eq!(segments[0].source_text, "Chuong Mot");
    assert_eq!(segments[0].target_text, "");
    assert_eq!(segments[0].translation_origin, "");
    assert_eq!(segments[0].status, "draft");

    cleanup(&root);
}

/// "Skip blank source" — 0-vs-2 row, skipped: bản dịch bị bỏ, 0 segment từ hàng này.
#[test]
fn skip_on_a_blank_source_drops_the_translation_and_yields_zero_segments() {
    let root = temp_dir("6-17-skip-source");
    let csv = ",Mot cau. Hai cau.\nCon Hang Nay.,Dich hang nay.\n";
    let path = write_file(&root, "khong-nguon.csv", csv.as_bytes());
    let shape = import_bilingual_file(&path).expect("tep hop le phai doc duoc");
    let preview = preview_bilingual_import(&shape, "en", &[], None, 0, 1, false, &[]).expect("xem truoc phai thanh cong");
    let selected = preview.candidates.iter().find(|c| c.encoding == preview.selected_encoding).unwrap();
    assert_eq!(selected.mismatches.len(), 1);
    let m = &selected.mismatches[0];
    assert!(m.source_sentences.is_empty());
    assert_eq!(m.target_line, "Mot cau. Hai cau.");
    // §I/O Matrix "Skip blank source" — "count shown in preview": Rust phai cap DUNG so cau
    // dich se bi bo, khong suy tu `initial_cuts.len() + 1` (suy do sai o dung ca 0 cau dich).
    assert_eq!(m.target_sentence_count, 2);
    // Chua ai danh dau Skip cho hang nao -- tong tren CA ung vien phai la 0.
    assert_eq!(selected.skipped_target_sentence_count, 0);

    // 🔴 THÊM (vòng rà đối kháng) — ap quy nhom Skip roi goi lai preview (cung mot lot "rebuild"
    // ma webview lam): hang bien MAT khoi `mismatches` (da giai quyet), nhung tong tren
    // `BilingualEncodingCandidateWire` phai SONG SOT dung 2 -- day la con so "se bi bo neu
    // xac nhan ngay bay gio", khac han `target_sentence_count` cua mot hang CON lech cap.
    let rebuilt = preview_bilingual_import(&shape, "en", &[], None, 0, 1, false, &[skip_regrouping(m)])
        .expect("xem truoc voi quy nhom Skip phai thanh cong");
    let rebuilt_selected = rebuilt.candidates.iter().find(|c| c.encoding == rebuilt.selected_encoding).unwrap();
    assert!(rebuilt_selected.mismatches.is_empty(), "hang da duoc giai quyet bang Skip");
    assert_eq!(rebuilt_selected.skipped_target_sentence_count, 2);

    let state = pending_state();
    stash_pending_import_source(&state, shape, None);
    let opened =
        confirm_bilingual_import(&root, &state, "Khong Nguon", "en", "", "UTF-8", Vec::new(), None, 0, 1, false, vec![
            skip_regrouping(m),
        ])
        .expect("bo qua mot hang 0-vs-n phai thanh cong");

    let chapters = read_chapters(&opened.store);
    let segments = read_segments(&opened.store, chapters[0].id);
    // 0 segment tu hang bi bo qua + 1 segment tu hang con lai.
    assert_eq!(segments.len(), 1);
    assert_eq!(segments[0].source_text, "Con Hang Nay.");

    cleanup(&root);
}

/// "Skip refused" — 2-vs-1 row, skip bị gửi từ dây: hàng vẫn ở lại chưa giải quyết, 0 gì được
/// ghi, một lỗi TYPED (không phải một hàng lặng lẽ nằm lại).
#[test]
fn skip_on_a_row_with_sentences_on_both_sides_is_refused_with_a_typed_error() {
    let root = temp_dir("6-17-skip-refused");
    let csv = "He left. She smiled.,He left she smiled\n";
    let path = write_file(&root, "khong-duoc-bo-qua.csv", csv.as_bytes());
    let shape = import_bilingual_file(&path).expect("tep hop le phai doc duoc");
    let preview = preview_bilingual_import(&shape, "en", &[], None, 0, 1, false, &[]).expect("xem truoc phai thanh cong");
    let selected = preview.candidates.iter().find(|c| c.encoding == preview.selected_encoding).unwrap();
    let m = &selected.mismatches[0];

    let err = preview_bilingual_import(&shape, "en", &[], None, 0, 1, false, &[skip_regrouping(m)])
        .expect_err("bo qua mot hang ca hai phia deu co cau phai bi tu choi TYPED");
    assert_eq!(err.message_key(), MessageKey::ImportBilingualSkipNotAllowed);
    assert_eq!(err.params().get("row").map(String::as_str), Some(m.row_number.to_string()).as_deref());

    let state = pending_state();
    stash_pending_import_source(&state, shape, None);
    let confirm_err =
        confirm_bilingual_import(&root, &state, "Bi Tu Choi", "en", "", "UTF-8", Vec::new(), None, 0, 1, false, vec![
            skip_regrouping(m),
        ])
        .expect_err("cung phai bi tu choi luc xac nhan");
    assert_eq!(confirm_err.message_key(), MessageKey::ImportBilingualSkipNotAllowed);
    assert_eq!(entry_count(&root), 1, "0 Work nao duoc ghi");

    cleanup(&root);
}

/// "Cancel" — quy nhóm chỉ sống trong tham số MỖI LƯỢT gọi (không state phía Rust); huỷ =
/// không gọi `confirm_bilingual_import` nữa, 0 Work ghi xuống, KHÔNG cần một API "clear" riêng
/// (khác state phía webview, xem `bilingualImportPreviewState.ts`). Test này khoá đúng vế
/// RUST: một quy nhóm ĐÃ TÍNH không có tác dụng nếu không đi qua CHÍNH lượt xác nhận.
#[test]
fn a_computed_regrouping_never_writes_unless_the_very_confirm_call_carries_it() {
    let root = temp_dir("6-17-cancel");
    let csv = "He left. She smiled.,He left she smiled\n";
    let path = write_file(&root, "huy.csv", csv.as_bytes());
    let shape = import_bilingual_file(&path).expect("tep hop le phai doc duoc");
    let preview = preview_bilingual_import(&shape, "en", &[], None, 0, 1, false, &[]).expect("xem truoc phai thanh cong");
    let selected = preview.candidates.iter().find(|c| c.encoding == preview.selected_encoding).unwrap();
    let m = &selected.mismatches[0];
    let _regrouping = cuts_regrouping(m, m.proposed_cuts.clone()); // nguoi dung da bam mot cho cat...

    let state = pending_state();
    stash_pending_import_source(&state, shape, None);
    cancel_import_preview(&state); // ...roi huy truoc khi xac nhan.

    let err = confirm_bilingual_import(&root, &state, "Da Huy", "en", "", "UTF-8", Vec::new(), None, 0, 1, false, Vec::new())
        .expect_err("huy phai don sach nguon dang cho -- xac nhan sau do phai bi tu choi");
    assert_eq!(err.message_key(), MessageKey::ImportNoPendingSource);
    assert_eq!(entry_count(&root), 1, "0 .atproj nao duoc ghi sau mot luot huy -- chi con tep .csv vua tao");

    cleanup(&root);
}

/// "Unaffected regrouping" — §I/O Matrix: quy nhom lam o luot CHUA bat tieu de van con hieu luc
/// sau khi bat "hang 1 la tieu de". Hang bi bo la hang 1; `row_number` cua hang da sua khong doi
/// va hai o cua no cung khong doi, nen anh chup van khop va hang van cap duoc -- ca luc xem truoc
/// lan luc ghi xuong dia.
#[test]
fn a_regrouping_survives_a_header_toggle_that_does_not_touch_its_row() {
    let root = temp_dir("6-17-header-toggle");
    // Hang 1 cap 1-1 (mot cau moi ben) nen chi hang 2 lech cap.
    let csv = "Nguon,Dich\nHe left. She smiled.,He left she smiled\n";
    let path = write_file(&root, "bat-tieu-de.csv", csv.as_bytes());
    let shape = import_bilingual_file(&path).expect("tep hop le phai doc duoc");

    let preview = preview_bilingual_import(&shape, "en", &[], None, 0, 1, false, &[]).expect("xem truoc phai thanh cong");
    let selected = preview.candidates.iter().find(|c| c.encoding == preview.selected_encoding).unwrap();
    let m = selected.mismatches.iter().find(|m| m.row_number == 2).expect("hang 2 phai lech cap").clone();
    let regrouping = cuts_regrouping(&m, m.proposed_cuts.clone());

    // BAT tieu de -- hang 1 bi bo TRUOC khi tach Chuong, hang 2 giu nguyen so hang va noi dung.
    let rebuilt = preview_bilingual_import(&shape, "en", &[], None, 0, 1, true, std::slice::from_ref(&regrouping))
        .expect("xem truoc phai thanh cong");
    let rebuilt_selected = rebuilt.candidates.iter().find(|c| c.encoding == rebuilt.selected_encoding).unwrap();
    assert!(
        rebuilt_selected.mismatches.is_empty(),
        "bat tieu de khong dung toi hang 2 thi khong duoc lam mat hieu luc quy nhom cua no"
    );

    let state = pending_state();
    stash_pending_import_source(&state, shape, None);
    let opened =
        confirm_bilingual_import(&root, &state, "Bat Tieu De", "en", "", "UTF-8", Vec::new(), None, 0, 1, true, vec![
            regrouping,
        ])
        .expect("xac nhan phai thanh cong");

    let chapters = read_chapters(&opened.store);
    let segments = read_segments(&opened.store, chapters[0].id);
    assert_eq!(segments.len(), 2, "hang tieu de da bi bo, hang 2 cap thanh dung hai segment");
    assert_eq!(segments[0].source_text, "He left.");
    assert_eq!(segments[1].source_text, "She smiled.");

    cleanup(&root);
}

// ═════════════════════════════════════════════════════════════════════════════════
// AI-4 — hai lượt xác nhận CHỒNG NHAU trên CÙNG một nguồn đang chờ
// ═════════════════════════════════════════════════════════════════════════════════

/// 🔴 **Sinh đôi song ngữ của
/// `segment_contract.rs::two_concurrent_confirms_on_the_same_pending_source_produce_exactly_one_work_not_two`**
/// (vòng rà đối kháng 2, mục 14 — nhánh văn xuôi). Nhánh song ngữ mang ĐÚNG hình dạng ấy —
/// `let mut guard = state.lock()` ở `project.rs:3744`, `shape` clone ra từ dưới khoá,
/// `create_work`, rồi `*guard = None` ở `:3772` — nhưng chưa có ca nào canh nó: thu hẹp khoá
/// lại thành "đọc-clone-rồi-thả, sau đó mới `create_work`" đi qua trọn bộ test.
///
/// **Vì sao ca này ra đời đúng lúc này (AI-4).** Trước AI-4 vỏ IPC
/// `wire::confirm_bilingual_import` là `#[tauri::command]` trần, nên tauri chạy nó trên luồng
/// chính và HAI lời gọi không thể chồng nhau — khoá rộng là một bất biến chưa ai kiểm được
/// thiệt hại. AI-4 lật vỏ đó sang `#[tauri::command(async)]` (thân hàm chạy trên một luồng
/// worker tokio), nên hai lời gọi IPC gần như cùng lúc nay CHẠY SONG SONG THẬT, và khoá rộng
/// trở thành thứ duy nhất chắn giữa chúng.
///
/// Ca này gọi thẳng HÀM THUẦN qua hai luồng thật (`std::thread::scope`), không qua webview:
/// đúng khuôn ca văn xuôi, và nó đo được chính mệnh đề về khoá mà `(async)` vừa đưa vào tầm.
///
/// **Đối chứng đỏ đã chạy (2026-09-15):** thu hẹp khoá trong `confirm_bilingual_import`
/// thành đọc-clone-`drop(guard)`-rồi-`create_work` ⇒ ca này ĐỎ (hai lượt cùng `Ok`); trả
/// khoá về hình dạng cũ ⇒ XANH lại.
#[test]
fn two_concurrent_bilingual_confirms_on_the_same_pending_source_produce_exactly_one_work_not_two() {
    let root = temp_dir("bilingual-confirm-race");
    let csv = "Cau mot day.,Cau mot dich.\n";
    let path = write_file(&root, "race.csv", csv.as_bytes());

    let shape = import_bilingual_file(&path).expect("tep hop le phai doc duoc");
    let state = pending_state();
    stash_pending_import_source(&state, shape, None);

    // 🔵 THEM 2026-09-15 (AI-4, vong ra 3) -- truoc do hai `spawn` chay khong co diem hen, nen
    // ket qua cua ca nay phu thuoc vao lich dinh thoi: neu luong 1 kip xong `create_work` va
    // don o nho TRUOC khi luong 2 doc no, mot cay ĐA HONG van cho dung mot `Ok` + mot
    // `import.no_pending_source` -- chinh hai menh de ca nay assert -- va ca nay XANH.
    //
    // ⚠️ **PHEP DO, khong phai mot suy luan.** Do 2026-09-15 tren macOS cua Ice, cay da hong
    // (khoa `PendingImportSourceState` thu hep thanh doc-clone-`drop`-roi-`create_work`):
    //   CO `Barrier`    : 10/10 luot DO
    //   KHONG `Barrier` : 10/10 luot DO
    // Tuc la o day `Barrier` KHONG bien mot luot xanh gia thanh do -- kich ban xanh gia da
    // KHONG tai lap duoc tren may nay. No van duoc giu lai vi no go su phu thuoc ay bang CAU
    // TRUC: 10/10 tren mot macOS nhan roi khong noi gi ve mot runner Windows dang tai nang,
    // noi hai `spawn` de bi noi tiep hon nhieu. Day la bao hiem CO LY DO, khong phai mot
    // khuyet tat da do -- dung trich no nhu bang chung rang ca nay tung xanh gia.
    let gate = std::sync::Barrier::new(2);
    let results = std::thread::scope(|scope| {
        let h1 = scope.spawn(|| {
            gate.wait();
            confirm_bilingual_import(
                &root, &state, "Race A", "en", "", "UTF-8", Vec::new(), None, 0, 1, false,
                Vec::new(),
            )
        });
        let h2 = scope.spawn(|| {
            gate.wait();
            confirm_bilingual_import(
                &root, &state, "Race B", "en", "", "UTF-8", Vec::new(), None, 0, 1, false,
                Vec::new(),
            )
        });
        [h1.join().expect("luong 1 panic"), h2.join().expect("luong 2 panic")]
    });

    let successes = results.iter().filter(|r| r.is_ok()).count();
    let failures: Vec<_> = results.iter().filter(|r| r.is_err()).collect();
    assert_eq!(
        successes, 1,
        "ky vong DUNG MOT luot xac nhan song ngu thanh cong tu MOT nguon dang cho -- hai luot \
         cung Ok nghia la khoa `PendingImportSourceState` da bi thu hep, va MOT nguon vua sinh \
         ra HAI Tac pham. Nhan duoc: {results:?}"
    );
    assert_eq!(
        failures.len(),
        1,
        "luot con lai phai bi tu choi (nguon dang cho da bi don): {results:?}"
    );
    for err in &failures {
        let Err(e) = err else { unreachable!() };
        assert_eq!(
            e.code(),
            "import.no_pending_source",
            "luot thua phai bi tu choi vi HET nguon dang cho, khong phai vi mot ly do khac: {e:?}"
        );
    }

    for r in results {
        if let Ok(opened) = r {
            drop(opened.store);
        }
    }
    cleanup(&root);
}

// ═════════════════════════════════════════════════════════════════════════════════
// Story 6.16b — bộ lọc "cần xem" cho bản xem trước song ngữ (FR132, tầng 4 TÁI DÙNG qua
// `build_chapter_split_preview_wire`, đúng đường sản phẩm `preview_bilingual_import`).
// review.rs::tests và segment_contract.rs đã canh RIÊNG hai hàng ma trận thuần-toán học
// ("một tín hiệu suy biến" — `a_degenerate_iqr_excludes_only_that_signal`) không lặp lại
// ở đây; `classify` không bị đụng (spec §Boundaries: 0 dòng).
// ═════════════════════════════════════════════════════════════════════════════════

/// I/O Matrix "Table parse fails for a candidate" — một ứng viên (KHÔNG phải ứng viên đang
/// chọn) không ra bảng hợp lệ (`TooFewColumns`) không kéo cả lượt xem trước xuống: `chapters`
/// của ĐÚNG ứng viên đó là `None`, ứng viên đang chọn vẫn có khối tách Chương thật.
///
/// Tệp ASCII thuần: bốn ứng viên byte-đơn-vị (UTF-8/GB18030/GBK/Big5) giải mã GIỐNG HỆT nhau
/// (đúng "phép biểu quyết cùng một chuỗi", `encoding.rs` đầu tệp) nên ứng viên đang chọn (tự
/// đoán UTF-8, tin cậy cao) ra bảng hợp lệ hai cột — nhưng UTF-16LE giải mã CÙNG dải byte ASCII
/// đó ra một chuỗi CJK vô nghĩa (`encoding.rs:25`: `b"abcd"` → `扡摣`, không lỗi, không panic)
/// mà không dấu phẩy ASCII nào còn sống sót qua phép ghép cặp byte — đúng một cột, dưới hai.
#[test]
fn a_table_parse_failure_on_a_non_selected_candidate_leaves_its_chapters_none_others_unaffected() {
    let root = temp_dir("candidate-table-parse-fails");
    let csv = "Cau nguon mot,Cau dich mot\nCau nguon hai,Cau dich hai\n";
    let path = write_file(&root, "ascii-hai-cot.csv", csv.as_bytes());
    let shape = import_bilingual_file(&path).expect("tep hop le phai doc duoc");

    let preview = preview_bilingual_import(&shape, "en", &[], None, 0, 1, false, &[])
        .expect("mot ung vien hong KHONG duoc keo ca luot xem truoc xuong khi no khong phai ung vien dang chon");

    assert_eq!(preview.selected_encoding, "UTF-8", "tien de: tep ASCII thuan tu doan UTF-8, tin cay cao");
    let selected = preview
        .candidates
        .iter()
        .find(|c| c.encoding == preview.selected_encoding)
        .expect("ung vien dang chon phai co mat trong dai");
    assert!(selected.chapters.is_some(), "ung vien dang chon van phai co khoi tach Chuong that");

    let utf16 = preview
        .candidates
        .iter()
        .find(|c| c.encoding == "UTF-16LE")
        .expect("dai nam ung vien luon co UTF-16LE");
    assert!(
        utf16.chapters.is_none(),
        "UTF-16LE giai ma ASCII ra chuoi khong con dau phay -- table-parse phai that bai, chapters phai None"
    );

    cleanup(&root);
}

/// I/O Matrix "Ten bilingual Chapters, all three signals measurable" + "One Chapter unusually
/// short" — mười Chương THẬT trên đường sản phẩm, mỗi Chương hai hàng (tiêu đề + nội dung) để
/// `length` (cột NGUỒN của cả hai hàng cộng lại) tản rộng thật, cùng quy ước
/// `review_contract.rs::a_very_short_chapter_among_ten_real_ones_is_flagged_short_length`.
#[test]
fn ten_bilingual_chapters_show_chip_counts_and_flag_the_short_one() {
    let root = temp_dir("chip-short");
    let mut csv = String::new();
    for i in 0..10 {
        let content_source = if i == 9 { "x".repeat(5) } else { "a".repeat(500 + i * 20) };
        csv.push_str(&format!("CHUONG {:02},dich tieu de {i}\n", i + 1));
        csv.push_str(&format!("{content_source},dich noi dung {i}\n"));
    }
    let path = write_file(&root, "muoi-chuong.csv", csv.as_bytes());
    let shape = import_bilingual_file(&path).expect("tep hop le phai doc duoc");
    let pattern = ChapterPattern::literal("CHUONG");

    let preview = preview_bilingual_import(&shape, "en", &[], Some(&pattern), 0, 1, false, &[])
        .expect("xem truoc phai thanh cong");
    let selected = preview
        .candidates
        .iter()
        .find(|c| c.encoding == preview.selected_encoding)
        .expect("ung vien dang chon phai co mat trong dai");
    let chapters_wire = selected.chapters.as_ref().expect("duong co mau phai co khoi tach Chuong");

    assert_eq!(chapters_wire.chapter_count, 10);
    assert!(chapters_wire.any_signal_participated, "10 Chuong voi length tan rong phai co hang rao");
    let last = chapters_wire.chapters.last().expect("Chuong thu muoi");
    assert!(last.needs_review, "Chuong cuc ngan phai la CAN XEM");
    assert!(last.review_causes.contains(&ReviewCauseWire::ShortLength));
    assert_eq!(chapters_wire.needs_review_count, 1);
    assert_eq!(chapters_wire.clean_count, 9);
    assert_eq!(chapters_wire.broken_item_count, 0, "duong song ngu khong co khai niem muc hong");

    cleanup(&root);
}

/// I/O Matrix "Cleanup rules hit only the target column" (Decision 2) — luật làm sạch chỉ
/// khớp ở CỘT ĐÍCH, cột nguồn 0 chỗ khớp; Chương vẫn bị gắn `high_cleanup_matches` vì Decision
/// 2 cộng CẢ HAI cột vào một tổng. Đối chứng cho chính rule đó: nếu tổng bị co về CHỈ cột
/// nguồn (Decision 2 gãy), `cleanup_match_count` của mọi Chương tụt về `Some(0)`, tín hiệu suy
/// biến (IQR = 0), và Chương thứ mười không còn bị gắn cờ — cả hai assert dưới đây đỏ.
#[test]
fn cleanup_matches_only_in_the_target_column_still_flags_the_chapter() {
    let root = temp_dir("cleanup-target-only");
    let mut csv = String::new();
    let counts = [1usize, 2, 3, 4, 5, 6, 7, 8, 9, 200];
    for (i, &count) in counts.iter().enumerate() {
        csv.push_str(&format!("CHUONG {:02},de\n", i + 1));
        let target = "QUANGCAO ".repeat(count);
        csv.push_str(&format!("Noi dung nguon khong doi.,{target}\n"));
    }
    let path = write_file(&root, "cleanup-dich.csv", csv.as_bytes());
    let shape = import_bilingual_file(&path).expect("tep hop le phai doc duoc");
    let pattern = ChapterPattern::literal("CHUONG");
    let rule = CleanupRule {
        tier: CleanupRuleTier::Global,
        id: 1,
        pattern: "QUANGCAO".to_owned(),
        kind: CleanupRuleKind::Literal,
        enabled: true,
    };

    let preview = preview_bilingual_import(&shape, "en", &[rule], Some(&pattern), 0, 1, false, &[])
        .expect("xem truoc phai thanh cong");
    let selected = preview
        .candidates
        .iter()
        .find(|c| c.encoding == preview.selected_encoding)
        .expect("ung vien dang chon phai co mat trong dai");
    let chapters_wire = selected.chapters.as_ref().expect("duong co mau phai co khoi tach Chuong");

    assert_eq!(chapters_wire.chapter_count, 10);
    for (i, chapter) in chapters_wire.chapters.iter().enumerate() {
        assert_eq!(
            chapter.cleanup_match_count,
            Some(counts[i]),
            "Chuong {i}: tong phai bang DUNG so QUANGCAO o cot dich -- cot nguon 0 cho khop"
        );
    }
    let last = chapters_wire.chapters.last().expect("Chuong thu muoi");
    assert!(last.needs_review, "200 cho khop o cot dich phai vuot hang rao");
    assert!(last.review_causes.contains(&ReviewCauseWire::HighCleanupMatches));
    assert!(
        !last.review_causes.contains(&ReviewCauseWire::ShortLength),
        "length khong doi giua cac Chuong -- suy bien, khong duoc gan co"
    );
    assert_eq!(chapters_wire.needs_review_count, 1);
    assert_eq!(chapters_wire.clean_count, 9);

    cleanup(&root);
}

/// I/O Matrix "Normalize joins lines only in the target column" (Decision 2) — dòng bị NỐI
/// chỉ ở CỘT ĐÍCH (cột nguồn nguyên một dòng, 0 lần nối); Chương vẫn bị gắn `high_joined_lines`.
#[test]
fn joined_lines_only_in_the_target_column_still_flags_the_chapter() {
    let root = temp_dir("joined-target-only");
    let mut csv = String::new();
    // So dong moi o dich = i + 2 (2..=10 dong -> noi 1..=9 lan, tan rong that); Chuong thu
    // muoi 51 dong -> noi 50 lan, vuot han chin Chuong con lai.
    let line_counts = [2usize, 3, 4, 5, 6, 7, 8, 9, 10, 51];
    for (i, &n) in line_counts.iter().enumerate() {
        csv.push_str(&format!("CHUONG {:02},de\n", i + 1));
        let lines: Vec<String> = (0..n).map(|k| format!("dong {k}")).collect();
        let target_cell = format!("\"{}\"", lines.join("\n"));
        csv.push_str(&format!("Noi dung nguon khong doi.,{target_cell}\n"));
    }
    let path = write_file(&root, "noi-dong-dich.csv", csv.as_bytes());
    let shape = import_bilingual_file(&path).expect("tep hop le phai doc duoc");
    let pattern = ChapterPattern::literal("CHUONG");

    let preview = preview_bilingual_import(&shape, "en", &[], Some(&pattern), 0, 1, false, &[])
        .expect("xem truoc phai thanh cong");
    let selected = preview
        .candidates
        .iter()
        .find(|c| c.encoding == preview.selected_encoding)
        .expect("ung vien dang chon phai co mat trong dai");
    let chapters_wire = selected.chapters.as_ref().expect("duong co mau phai co khoi tach Chuong");

    assert_eq!(chapters_wire.chapter_count, 10);
    for (i, chapter) in chapters_wire.chapters.iter().enumerate() {
        assert_eq!(
            chapter.joined_line_count_in_chapter,
            Some(line_counts[i] - 1),
            "Chuong {i}: so lan noi phai bang DUNG (so dong - 1) cua cot dich -- cot nguon 0 lan noi"
        );
    }
    let last = chapters_wire.chapters.last().expect("Chuong thu muoi");
    assert!(last.needs_review);
    assert!(last.review_causes.contains(&ReviewCauseWire::HighJoinedLines));
    assert_eq!(chapters_wire.needs_review_count, 1);
    assert_eq!(chapters_wire.clean_count, 9);

    cleanup(&root);
}

/// I/O Matrix "Cleanup count `null` for one Chapter, fence live" + "Chapter missing both
/// optional signals" — một Chương THIẾU HẲN cột đích (hàng cụt cột: bảng khai ba cột nhưng
/// Chương này chỉ có hai) trong khi chín Chương còn lại đủ ba cột VÀ tản rộng thật ở cột đích
/// (hàng rào SỐNG). Chương cụt cột phải là CẦN XEM với `not_measured`, KHÔNG BAO GIỜ sạch, và
/// `not_measured` chỉ xuất hiện ĐÚNG MỘT LẦN dù CẢ HAI tín hiệu cùng thiếu (bất biến thứ hai
/// của `ReviewVerdict`, xem `core::segment::review.rs`).
#[test]
fn a_chapter_missing_the_target_column_entirely_is_not_measured_never_clean() {
    let root = temp_dir("ragged-column");
    let mut csv = String::new();
    for i in 0..9 {
        csv.push_str(&format!("CHUONG {:02},junk,de\n", i + 1));
        let target = "QUANGCAO ".repeat(i + 1);
        csv.push_str(&format!("Noi dung nguon khong doi.,junk,{target}\n"));
    }
    // Chuong thu muoi -- CUT COT: chi hai cot (0, 1), cot dich (chi so 2) khong ton tai o CA
    // HAI hang cua no (tieu de lan noi dung).
    csv.push_str("CHUONG 10,junk\n");
    csv.push_str("Noi dung nguon khong doi.,junk\n");

    let path = write_file(&root, "cut-cot.csv", csv.as_bytes());
    let shape = import_bilingual_file(&path).expect("tep hop le phai doc duoc");
    let pattern = ChapterPattern::literal("CHUONG");
    let rule = CleanupRule {
        tier: CleanupRuleTier::Global,
        id: 1,
        pattern: "QUANGCAO".to_owned(),
        kind: CleanupRuleKind::Literal,
        enabled: true,
    };

    let preview = preview_bilingual_import(&shape, "en", &[rule], Some(&pattern), 0, 2, false, &[])
        .expect("xem truoc phai thanh cong");
    let selected = preview
        .candidates
        .iter()
        .find(|c| c.encoding == preview.selected_encoding)
        .expect("ung vien dang chon phai co mat trong dai");
    let chapters_wire = selected.chapters.as_ref().expect("duong co mau phai co khoi tach Chuong");

    assert_eq!(chapters_wire.chapter_count, 10);
    assert!(chapters_wire.any_signal_participated, "chin Chuong con lai tan rong that o cot dich");
    let last = chapters_wire.chapters.last().expect("Chuong thu muoi");
    assert_eq!(last.cleanup_match_count, None, "Chuong cut cot dich khong do duoc luat lam sach");
    assert_eq!(last.joined_line_count_in_chapter, None, "cung ly do -- cot dich khong ton tai de do");
    assert!(last.needs_review, "gia tri null duoi mot hang rao TON TAI phai la CAN XEM");
    assert_eq!(
        last.review_causes,
        vec![ReviewCauseWire::NotMeasured],
        "NotMeasured phai xuat hien DUNG MOT LAN du hai tin hieu cung thieu"
    );
    for (i, chapter) in chapters_wire.chapters.iter().enumerate().take(9) {
        assert!(!chapter.needs_review, "Chuong {i}: du du lieu, khong duoc gan co oan");
    }
    assert_eq!(chapters_wire.needs_review_count, 1);
    assert_eq!(chapters_wire.clean_count, 9);

    cleanup(&root);
}

/// I/O Matrix "Fewer than four Chapters" — dưới bốn Chương thật, không tín hiệu nào tham gia,
/// và không Chương nào bị phán "cần xem"/"sạch" theo một hàng rào không tồn tại.
#[test]
fn fewer_than_four_bilingual_chapters_makes_no_signal_participate() {
    let root = temp_dir("fewer-than-four");
    let csv = "CHUONG MOT,mot\nCHUONG HAI,hai ba\nCHUONG BA,bon nam sau\n";
    let path = write_file(&root, "ba-chuong-ngan.csv", csv.as_bytes());
    let shape = import_bilingual_file(&path).expect("tep hop le phai doc duoc");
    let pattern = ChapterPattern::literal("CHUONG");

    let preview = preview_bilingual_import(&shape, "en", &[], Some(&pattern), 0, 1, false, &[])
        .expect("xem truoc phai thanh cong");
    let selected = preview
        .candidates
        .iter()
        .find(|c| c.encoding == preview.selected_encoding)
        .expect("ung vien dang chon phai co mat trong dai");
    let chapters_wire = selected.chapters.as_ref().expect("duong co mau phai co khoi tach Chuong");

    assert_eq!(chapters_wire.chapter_count, 3);
    assert!(
        !chapters_wire.any_signal_participated,
        "N = 3 phai bao 'chua du Chuong de so', khong tin hieu nao duoc tham gia"
    );
    for chapter in &chapters_wire.chapters {
        assert!(!chapter.needs_review, "khong tin hieu nao tham gia thi khong Chuong nao bi phan");
    }

    cleanup(&root);
}

/// `broken_item_count` truyền `0` trên đường song ngữ — I/O Matrix nêu tên riêng vì đây là
/// nơi §Always spec 6.16b buộc hằng số đó, khác đường URL (Story 6.10) truyền số mục hỏng
/// thật. Đối chứng: nếu tham số đổi thành khác `0`, `needs_review_count` cộng thêm sai.
#[test]
fn broken_item_count_is_always_zero_on_the_bilingual_path() {
    let root = temp_dir("broken-zero");
    let long = "b".repeat(500);
    let mut csv = String::new();
    for i in 0..6 {
        csv.push_str(&format!("CHUONG {i},de\n"));
        csv.push_str(&format!("{long},dich\n"));
    }
    let path = write_file(&root, "sau-chuong.csv", csv.as_bytes());
    let shape = import_bilingual_file(&path).expect("tep hop le phai doc duoc");
    let pattern = ChapterPattern::literal("CHUONG");

    let preview = preview_bilingual_import(&shape, "en", &[], Some(&pattern), 0, 1, false, &[])
        .expect("xem truoc phai thanh cong");
    let selected = preview
        .candidates
        .iter()
        .find(|c| c.encoding == preview.selected_encoding)
        .expect("ung vien dang chon phai co mat trong dai");
    let chapters_wire = selected.chapters.as_ref().expect("duong co mau phai co khoi tach Chuong");

    assert_eq!(chapters_wire.chapter_count, 6);
    assert_eq!(chapters_wire.broken_item_count, 0, "duong song ngu khong co khai niem muc hong URL");
    assert_eq!(chapters_wire.needs_review_count, 0, "khong tin hieu nao gan co that tren tap nay");
    assert_eq!(chapters_wire.clean_count, 6);

    cleanup(&root);
}
