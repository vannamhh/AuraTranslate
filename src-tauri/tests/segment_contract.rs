//! Hành vi lúc chạy của tầng segment — Story 2.1, AC1 tới AC15.
//!
//! ⚠️ Tệp riêng có chủ ý, đúng khuôn `store_contract.rs`/`project_contract.rs` — một tệp,
//! một mối quan tâm. Phép kiểm **tĩnh trên cây nguồn** sống ở `segment_boundary.rs`.
//!
//! ─────────────────────────────────────────────────────────────────────────────
//! BỐN LUẬT CỦA TỆP NÀY — thừa kế nguyên vẹn từ `project_contract.rs`
//! ─────────────────────────────────────────────────────────────────────────────
//! 1. **Mỗi ca một thư mục tạm riêng** (pid + `AtomicU64`). Không thêm `tempfile`.
//! 2. **Drop `Store`/`OpenWork` TRƯỚC khi xoá thư mục** — Windows từ chối xoá tệp đang mở.
//! 3. Không `sleep` dài.
//! 4. Không ca nào treo khi nó trượt.

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use auratranslate_lib::commands::chapter::split_chapter_at_segment;
use auratranslate_lib::commands::lifecycle::set_chapter_status;
use auratranslate_lib::commands::project::create_work_from_text;
use auratranslate_lib::commands::segment::{
    confirm_segment, flush_segment_targets, list_reading_marks, mark_reading_segment,
    merge_segments, read_open_chapter_segments, read_reading_run, read_segment_history,
    restore_segment_version, save_chapter_position, save_segment_targets, set_segment_omitted,
    split_chapter_into_segments, split_segment, unconfirm_edited_segments,
    ReadingFrontierKind, SegmentTargetEdit, SplitOutcome, SEGMENT_STATUS_CONFIRMED,
};
use auratranslate_lib::core::i18n::MessageKey;
use auratranslate_lib::core::segment::split::{
    split_source_text, EN_ABBREVIATIONS, LANG_CHINESE, SplitSegment,
};
use auratranslate_lib::core::store::{
    GLOBAL_MIGRATIONS, Migration, PINNED_ENTRY_DDL, PROJECT_MIGRATIONS, Store, StoreSpec,
    Transaction,
};

static NEXT_DIR: AtomicU64 = AtomicU64::new(0);

/// Một thư mục tạm **của riêng ca này**. Xem luật 1 ở doc-comment đầu tệp.
fn temp_dir(tag: &str) -> PathBuf {
    let n = NEXT_DIR.fetch_add(1, Ordering::Relaxed);
    let dir = std::env::temp_dir().join(format!(
        "auratranslate-segment-{}-{}-{}",
        std::process::id(),
        tag,
        n
    ));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap_or_else(|e| panic!("tao {}: {e}", dir.display()));
    dir
}

// ═════════════════════════════════════════════════════════════════════════════════
// Story 5.13 — marker bền vững khi đọc (`reading_mark`, FR119). §I/O Matrix.
// ═════════════════════════════════════════════════════════════════════════════════

#[test]
fn marking_a_live_reading_segment_is_idempotent_and_the_same_reading_snapshot_reports_it() {
    let root = temp_dir("5-13-mark-idempotent");
    let mut opened = create_work_from_text(
        &root,
        "5.13 danh dau",
        "zh",
        "",
        "Cau mot。Cau hai。".to_owned(),
    )
    .expect("tao Tac pham that bai");
    let loaded = read_open_chapter_segments(Some(&opened)).expect("nap segment");
    let id = loaded.segments[1].id;

    let first = mark_reading_segment(Some(&opened), id).expect("danh dau lan dau");
    let second = mark_reading_segment(Some(&opened), id).expect("danh dau lan hai");
    assert_eq!(first.segment_id, id);
    assert_eq!(first.marked_at, second.marked_at, "bam M lan hai khong duoc tao/toggle marker");
    let rows: i64 = opened
        .store
        .read(|conn| conn.query_row("SELECT COUNT(*) FROM reading_mark", [], |r| r.get(0)))
        .expect("dem marker");
    assert_eq!(rows, 1, "INSERT idempotent phai giu dung mot hang");

    let chapter_id = opened.chapter_id;
    set_chapter_status(Some(&mut opened), chapter_id, "done").expect("dat Chuong done");
    let reading = read_reading_run(Some(&opened)).expect("doc ReadingRun");
    let wire = &reading.chapters[0].paragraphs[0].segments;
    assert!(wire.iter().any(|s| s.id == id && s.is_marked));
    assert!(wire.iter().any(|s| s.id != id && !s.is_marked));

    let dir = opened.dir.clone();
    drop(opened);
    cleanup(&dir);
}

#[test]
fn reading_marks_reject_no_work_and_unknown_ids_without_creating_rows() {
    assert_eq!(mark_reading_segment(None, 1).unwrap_err().code(), "work.none_open");
    assert_eq!(list_reading_marks(None).unwrap_err().code(), "work.none_open");

    let root = temp_dir("5-13-mark-unknown");
    let opened = create_work_from_text(&root, "5.13 la", "zh", "", "Mot cau。".to_owned())
        .expect("tao Tac pham");
    assert_eq!(
        mark_reading_segment(Some(&opened), 999_999).unwrap_err().code(),
        "segment.not_found"
    );
    assert!(list_reading_marks(Some(&opened)).expect("liet ke").is_empty());
    let dir = opened.dir.clone();
    drop(opened);
    cleanup(&dir);
}

#[test]
fn a_dangling_reading_mark_anchor_is_an_error_instead_of_a_silently_short_list() {
    let root = temp_dir("5-13-mark-dangling-anchor");
    let opened = create_work_from_text(&root, "5.13 neo hong", "zh", "", "Mot cau。".to_owned())
        .expect("tao Tac pham");
    let segment_id = read_open_chapter_segments(Some(&opened)).expect("nap segment").segments[0].id;
    mark_reading_segment(Some(&opened), segment_id).expect("danh dau");
    opened
        .store
        .write(move |tx| {
            tx.execute(
                "UPDATE reading_mark SET navigation_segment_id = ?1 WHERE segment_id = ?2",
                [999_999_i64, segment_id],
            )?;
            Ok(())
        })
        .expect("tao neo hong doi chung");

    assert_eq!(
        list_reading_marks(Some(&opened)).unwrap_err().code(),
        "store.read_failed",
        "neo hong khong duoc bien mat nhu mot danh sach hop le ngan hon",
    );

    let dir = opened.dir.clone();
    drop(opened);
    cleanup(&dir);
}

#[test]
fn a_mark_keeps_its_original_identity_while_repeated_regroup_rebases_the_live_anchor() {
    let root = temp_dir("5-13-mark-regroup-repeat");
    let opened = create_work_from_text(
        &root,
        "5.13 regroup",
        "zh",
        "",
        "Cau mot。Cau hai。Cau ba。".to_owned(),
    )
    .expect("tao Tac pham");
    let loaded = read_open_chapter_segments(Some(&opened)).expect("nap segment");
    let original = loaded.segments[1].id;
    mark_reading_segment(Some(&opened), original).expect("danh dau");

    let merged = merge_segments(Some(&opened), original).expect("gop lan mot");
    let first_anchor = merged.new_segments[0].id;
    let after_merge = list_reading_marks(Some(&opened)).expect("liet ke sau gop");
    assert_eq!(after_merge.len(), 1);
    assert_eq!(after_merge[0].segment_id, original, "danh tinh goc khong doi");
    assert_eq!(after_merge[0].navigation_segment_id, first_anchor);
    assert!(after_merge[0].is_retired, "ghi chu retired den tu segment goc");

    let cut = merged.new_segments[0].source_text.chars().count() / 2;
    let split = split_segment(Some(&opened), first_anchor, vec![cut]).expect("tach lan hai");
    let after_split = list_reading_marks(Some(&opened)).expect("liet ke sau tach");
    assert_eq!(after_split[0].segment_id, original);
    assert_eq!(after_split[0].navigation_segment_id, split.new_segments[0].id);
    assert_ne!(after_split[0].navigation_segment_id, first_anchor);

    let dir = opened.dir.clone();
    drop(opened);
    cleanup(&dir);
}

#[test]
fn a_failed_marker_rebase_rolls_back_the_whole_regroup_transaction() {
    let root = temp_dir("5-13-mark-regroup-rollback");
    let opened = create_work_from_text(
        &root,
        "5.13 rollback",
        "zh",
        "",
        "Cau mot。Cau hai。".to_owned(),
    )
    .expect("tao Tac pham");
    let loaded = read_open_chapter_segments(Some(&opened)).expect("nap segment");
    let original = loaded.segments[1].id;
    mark_reading_segment(Some(&opened), original).expect("danh dau");
    opened
        .store
        .write(|tx| {
            tx.execute_batch(
                "CREATE TRIGGER reject_reading_mark_rebase \
                 BEFORE UPDATE OF navigation_segment_id ON reading_mark \
                 BEGIN SELECT RAISE(ABORT, 'reject rebase'); END;",
            )
        })
        .expect("tao trigger doi chung");

    assert!(merge_segments(Some(&opened), original).is_err(), "trigger phai lam regroup truot");
    let after = read_open_chapter_segments(Some(&opened)).expect("nap lai");
    assert_eq!(after.segments.len(), 2, "retire + insert phai rollback cung marker rebase");
    assert!(after.segments.iter().any(|s| s.id == original && s.retired_at.is_none()));
    let mark = list_reading_marks(Some(&opened)).expect("liet ke").remove(0);
    assert_eq!(mark.navigation_segment_id, original);

    let dir = opened.dir.clone();
    drop(opened);
    cleanup(&dir);
}

#[test]
fn a_mark_list_follows_the_live_anchor_to_its_new_chapter_and_never_crosses_work_stores() {
    let root_a = temp_dir("5-13-mark-work-a");
    let root_b = temp_dir("5-13-mark-work-b");
    let mut a = create_work_from_text(
        &root_a,
        "5.13 A",
        "zh",
        "",
        "Cau mot。Cau hai。Cau ba。".to_owned(),
    )
    .expect("tao A");
    let b = create_work_from_text(&root_b, "5.13 B", "zh", "", "Cau khac。".to_owned())
        .expect("tao B");
    let loaded = read_open_chapter_segments(Some(&a)).expect("nap A");
    let moving = loaded.segments[1].id;
    mark_reading_segment(Some(&a), moving).expect("danh dau A");
    assert!(list_reading_marks(Some(&b)).expect("liet ke B").is_empty());

    let old_chapter = a.chapter_id;
    split_chapter_at_segment(Some(&mut a), moving).expect("tach Chuong tai marker");
    let mark = list_reading_marks(Some(&a)).expect("liet ke A").remove(0);
    assert_ne!(mark.chapter_id, old_chapter, "Chuong phai lay tu neo song sau to chuc lai");
    let navigation_segment_id = mark.navigation_segment_id;
    let anchor_chapter: i64 = a
        .store
        .read(move |conn| {
            conn.query_row(
                "SELECT chapter_id FROM segment WHERE id = ?1",
                [navigation_segment_id],
                |row| row.get(0),
            )
        })
        .expect("doc Chuong cua neo song");
    assert_eq!(mark.chapter_id, anchor_chapter);

    let dir_a = a.dir.clone();
    let dir_b = b.dir.clone();
    drop(a);
    drop(b);
    cleanup(&dir_a);
    cleanup(&dir_b);
}

fn cleanup(dir: &Path) {
    let _ = fs::remove_dir_all(dir);
}

/// Chỉ phần văn bản, cho các ca không nói gì về cờ kết đoạn.
fn texts(segments: &[SplitSegment]) -> Vec<&str> {
    segments.iter().map(|s| s.text.as_str()).collect()
}

/// Chỉ phần cờ, cho các ca nói về cờ.
fn flags(segments: &[SplitSegment]) -> Vec<bool> {
    segments.iter().map(|s| s.is_paragraph_end).collect()
}

// ═════════════════════════════════════════════════════════════════════════════
// AC1 — nhánh tiếng Trung tách theo `。！？；`
// ═════════════════════════════════════════════════════════════════════════════

#[test]
fn chinese_splits_on_each_of_the_four_terminators() {
    // Bốn ca, mỗi ca một dấu — `；` là dấu mà UAX #29 KHÔNG coi la ranh gioi cau
    // (do 2026-08-12, `n=1`), va no la ly do bo tach nay duoc viet moi.
    let cases: [(&str, usize); 4] = [
        ("他走了。她笑了。", 2),
        ("真的吗？太好了。", 2),
        ("太好了！他走了。", 2),
        ("他走了；她笑了。", 2),
    ];

    for (input, expected) in cases {
        let got = split_source_text(input, LANG_CHINESE);
        assert_eq!(
            got.len(),
            expected,
            "dau ket cau tieng Trung khong tach: {input:?} -> {:?}",
            texts(&got)
        );
    }

    assert_eq!(
        texts(&split_source_text("他走了；她笑了。", LANG_CHINESE)),
        vec!["他走了；", "她笑了。"],
        "dau `；` phai o LAI voi cau ma no ket"
    );
}

// ═════════════════════════════════════════════════════════════════════════════
// AC2 — nhánh tiếng Anh, bốn luật của Quyết định #5
// ═════════════════════════════════════════════════════════════════════════════

/// Khuôn *"đối chiếu một bảng trong tài liệu từng hàng"* — mỗi hàng của Quyết định #5
/// một ca, kèm ca đối chứng cho thấy dấu chấm THẬT vẫn cắt.
#[test]
fn the_english_abbreviation_rules_match_decision_five_row_by_row() {
    // ⚠️ Cap nhat code review 2026-08-12: bang thieu 5/6 muc cua nhom thu ba, va ca `etc.`
    // duoi day tung mong `n=2` — no xanh nho LUAT DU PHONG (chu HOA theo sau), khong nho
    // bang viet tat, nen no khong kiem thu no tu khai dang kiem. Ice ky them CA NAM muc,
    // dung chu Quyet dinh #5 ⇒ ky vong doi sang `n=1`.
    let cases: [(&str, usize, &str); 13] = [
        // luat 1 — bang viet tat
        ("Mr. Smith went home. He slept.", 2, "danh xung"),
        ("Acme Inc. bought it. Nobody cared.", 2, "Inc."),
        // luat 1 — nam muc them 2026-08-12, moi muc mot ca, chu HOA ngay sau dau cham
        ("We sell books, pens, etc. They are cheap.", 1, "etc."),
        ("Real Madrid vs. Barcelona played.", 1, "vs."),
        ("Use a tool, e.g. Cargo builds it.", 1, "e.g."),
        ("One thing, i.e. The result matters.", 1, "i.e."),
        ("See cf. Chapter Three for more.", 1, "cf."),
        ("Smith et al. Then we checked.", 1, "al."),
        // luat 2 — chu cai dau don
        ("J. R. R. Tolkien wrote books. He died.", 2, "chu cai dau don"),
        // luat 3 — so thap phan
        ("It costs 3.50 dollars. That is fine.", 2, "so thap phan"),
        // luat 4 — dau ba cham
        ("Wait... What now?", 1, "ba cham ASCII"),
        ("Wait… What now?", 1, "ba cham U+2026"),
        // doi chung duong — dau cham THAT van cat
        ("He went home. He slept.", 2, "doi chung duong"),
    ];

    for (input, expected, label) in cases {
        let got = split_source_text(input, "en");
        assert_eq!(
            got.len(),
            expected,
            "luat `{label}` sai tren {input:?} -> {:?}",
            texts(&got)
        );
    }
}

/// Run đúng **hai** dấu chấm — code review 2026-08-12 bắt được ca này chưa có cổng nào.
///
/// Luật 4 của Quyết định #5 nói *"dấu ba chấm"*, nhưng mã khai rộng hơn bằng chữ: **mọi**
/// run từ 2 dấu chấm trở lên không kết câu (`en_run_is_boundary`). `".."` là một lỗi gõ
/// thường gặp, không phải một dấu ba chấm — ca này khoá hành vi đó lại thay vì để nó là
/// một tác dụng phụ không ai tuyên bố.
#[test]
fn a_run_of_exactly_two_periods_does_not_end_a_sentence() {
    assert_eq!(
        texts(&split_source_text("Wait.. What now?", "en")),
        vec!["Wait.. What now?"],
        "run hai dau cham phai di vao than segment, cung luat voi `...`"
    );
}

/// 🔴 Dấu kết câu **mồ côi** ở đầu segment kế tiếp — code review 2026-08-12, đo trước khi vá.
///
/// Trước bản vá, `split_source_text("你好？”！再见。", LANG_CHINESE)` cho
/// `["你好？”", "！再见。"]`: sau ranh giới, `pending_has_letter` về `false`, nên `！` không
/// tự cắt được (luật *"một câu phải có ít nhất một chữ"* chặn) và rơi vào **đầu** segment
/// hai. `TRAILING_CLOSERS` chỉ ngăn hình dạng hỏng ở phía sau ranh giới, không phía trước.
///
/// Ca này đỏ nếu `absorb_tail` thôi lặp xen kẽ dấu đóng ↔ dấu kết câu.
#[test]
fn no_segment_ever_opens_with_an_orphan_terminator() {
    assert_eq!(
        texts(&split_source_text("你好？”！再见。", LANG_CHINESE)),
        vec!["你好？”！", "再见。"],
        "dau ket cau dung sau dau dong phai o lai segment TRUOC"
    );

    // Doi chung — ca thuong gap khong duoc doi hanh vi.
    assert_eq!(
        texts(&split_source_text("他说：“真的吗？”她笑了。", LANG_CHINESE)),
        vec!["他说：“真的吗？”", "她笑了。"],
        "ca dau dong thuong gap phai giu nguyen ket qua"
    );
}

#[test]
fn an_abbreviation_keeps_the_whole_sentence_together() {
    assert_eq!(
        texts(&split_source_text("Mr. Smith went home. He slept.", "en")),
        vec!["Mr. Smith went home.", "He slept."],
        "UAX #29 cat ngay sau `Mr.` (do 2026-08-12, n=3) -- bo tach nay khong duoc phep"
    );
}

/// Bảng viết tắt là một hằng đã **sắp xếp và không trùng**. Một mục chép hai lần đi qua
/// mọi phép kiểm khác mà không ai thấy; đây là chỗ rẻ nhất bắt được nó, và là điều kiện để
/// `binary_search` dùng được thay cho một lượt quét tuyến tính.
#[test]
fn the_abbreviation_table_is_sorted_and_free_of_duplicates() {
    assert!(
        EN_ABBREVIATIONS.len() >= 20,
        "bang viet tat chi con {} muc -- qua nho de la that",
        EN_ABBREVIATIONS.len()
    );

    for pair in EN_ABBREVIATIONS.windows(2) {
        assert!(
            pair[0] < pair[1],
            "bang viet tat phai sap tang dan va khong trung: {:?} dung truoc {:?}",
            pair[0],
            pair[1]
        );
    }

    for entry in EN_ABBREVIATIONS {
        assert!(
            entry.ends_with('.'),
            "muc `{entry}` khong ket thuc bang dau cham -- no khong bao gio khop duoc"
        );
    }
}

// ═════════════════════════════════════════════════════════════════════════════
// AC11 + bổ sung Quyết định #3 — `\r` và `\n` không bao giờ nằm trong một segment
// ═════════════════════════════════════════════════════════════════════════════

/// AC11 nguyên văn: tách `"Câu một.\r\nCâu hai."` ⇒ segment thứ nhất là `"Câu một."`,
/// **không** `"Câu một.\r"`.
#[test]
fn a_carriage_return_never_sticks_to_the_end_of_a_segment() {
    let got = split_source_text("Cau mot.\r\nCau hai.", "en");

    assert_eq!(
        texts(&got),
        vec!["Cau mot.", "Cau hai."],
        "`\\r` cua CRLF dinh vao segment -- xem AC11"
    );
}

/// Đối chứng âm của AC11, mở rộng theo bổ sung của Quyết định #3: **không** segment nào
/// mang một ký tự xuống dòng nào, bất kể đầu vào và bất kể nhánh ngôn ngữ.
///
/// Vì sao mệnh đề rộng hơn AC11: AD-37 nói cờ kết đoạn mô tả *"sau câu này là xuống dòng"*,
/// và Story 8.4/8.6 dựng lại đoạn lúc xuất **chỉ từ cờ đã lưu**. Một `\n` nằm trong thân
/// segment là một ranh giới đoạn mà cờ không nói được — và AD-4 đóng băng nó vĩnh viễn.
#[test]
fn no_segment_ever_carries_a_line_break() {
    let inputs = [
        "第一章 开端\n他走了。",
        "Chapter One\r\nHe went home.",
        "a\rb\rc",
        "他走了。\n\n她笑了。\r\n\r\n第三段",
        "no terminator at all\nsecond line\nthird line",
    ];

    for input in inputs {
        for lang in [LANG_CHINESE, "en"] {
            for seg in split_source_text(input, lang) {
                assert!(
                    !seg.text.contains('\n') && !seg.text.contains('\r'),
                    "segment {:?} (lang={lang}) mang mot ky tu xuong dong -- dau vao {input:?}",
                    seg.text
                );
            }
        }
    }
}

/// Ca mà bổ sung của Quyết định #3 tồn tại để giải: một dòng **không** kết thúc bằng dấu
/// kết câu (tiêu đề chương) đứng trước một câu bình thường.
#[test]
fn a_line_that_does_not_end_in_a_terminator_is_still_its_own_segment() {
    let got = split_source_text("第一章 开端\n他走了。", LANG_CHINESE);

    assert_eq!(texts(&got), vec!["第一章 开端", "他走了。"]);
    assert_eq!(
        flags(&got),
        vec![true, false],
        "tieu de mang co ket doan; segment cuoi luon tat (AC7)"
    );
}

// ═════════════════════════════════════════════════════════════════════════════
// Bốn ca biên của Task 1
// ═════════════════════════════════════════════════════════════════════════════

/// Ca ① — văn bản rỗng hoặc chỉ khoảng trắng ⇒ **0 segment**.
#[test]
fn blank_text_yields_no_segments_at_all() {
    for input in ["", "   ", "\n\n\n", " \r\n \t \r\n ", "\u{3000}\u{3000}"] {
        for lang in [LANG_CHINESE, "en"] {
            assert!(
                split_source_text(input, lang).is_empty(),
                "dau vao trang {input:?} (lang={lang}) phai cho 0 segment"
            );
        }
    }
}

/// Ca ② — văn bản không kết thúc bằng dấu kết câu ⇒ phần đuôi vẫn là **một** segment.
#[test]
fn a_tail_without_a_terminator_is_still_a_segment() {
    assert_eq!(
        texts(&split_source_text("他走了。她还在", LANG_CHINESE)),
        vec!["他走了。", "她还在"]
    );
    assert_eq!(
        texts(&split_source_text("He left. She stayed", "en")),
        vec!["He left.", "She stayed"]
    );
    assert_eq!(texts(&split_source_text("khong dau cham nao", "en")), vec!["khong dau cham nao"]);
}

/// Ca ③ — nhiều dấu kết câu liền nhau ⇒ **một** ranh giới, không segment rỗng.
#[test]
fn a_run_of_terminators_is_one_boundary_not_many() {
    let got = split_source_text("真的吗？？！太好了。", LANG_CHINESE);
    assert_eq!(texts(&got), vec!["真的吗？？！", "太好了。"]);

    let got_en = split_source_text("Really?! He left.", "en");
    assert_eq!(texts(&got_en), vec!["Really?!", "He left."]);
}

/// Ca ④ — **không** segment nào rỗng hoặc chỉ khoảng trắng, bất kể đầu vào.
#[test]
fn no_segment_is_ever_empty_or_blank() {
    let inputs = [
        "",
        "。。。",
        "！！！他走了。",
        "  \n  他走了。  \n  ",
        "...",
        ". . . .",
        "他走了。\n\n\n\n她笑了。",
        "Mr. Mrs. Dr. Smith.",
    ];

    for input in inputs {
        for lang in [LANG_CHINESE, "en"] {
            for seg in split_source_text(input, lang) {
                assert!(
                    !seg.text.trim().is_empty(),
                    "segment rong hoac chi khoang trang tu dau vao {input:?} (lang={lang})"
                );
                assert_eq!(
                    seg.text,
                    seg.text.trim(),
                    "segment {:?} chua bi cat trang hai dau",
                    seg.text
                );
            }
        }
    }
}

// ═════════════════════════════════════════════════════════════════════════════
// AC6 · AC7 — cờ kết đoạn
// ═════════════════════════════════════════════════════════════════════════════

#[test]
fn the_paragraph_flag_is_on_exactly_where_a_line_break_follows() {
    let got = split_source_text("他走了。她笑了。\n第二段。他还在。", LANG_CHINESE);

    assert_eq!(texts(&got), vec!["他走了。", "她笑了。", "第二段。", "他还在。"]);
    assert_eq!(
        flags(&got),
        vec![false, true, false, false],
        "chi segment ngay TRUOC mot xuong dong moi mang co"
    );
}

/// AC7 — segment cuối cùng của một Chương: cờ **tắt, luôn luôn**, kể cả khi văn bản gốc
/// kết thúc bằng một dòng trống.
#[test]
fn the_last_segment_never_ends_a_paragraph() {
    for input in [
        "他走了。",
        "他走了。\n",
        "他走了。\n\n\n",
        "他走了。\r\n\r\n",
        "他走了。   \n   \n   ",
    ] {
        let got = split_source_text(input, LANG_CHINESE);
        let last = got.last().unwrap_or_else(|| panic!("dau vao {input:?} phai cho >=1 segment"));
        assert!(
            !last.is_paragraph_end,
            "segment cuoi cua {input:?} mang co ket doan -- AC7 cam tuyet doi"
        );
    }
}

/// Luật thứ năm — **một câu phải có ít nhất một chữ**. Một dấu chấm sau một chuỗi chỉ gồm
/// chữ số và dấu là một **mốc đánh số**, không phải một câu.
///
/// 🔴 Ca này dựng từ dữ liệu THẬT, không từ một giả định: Chương lớn nhất của Epic 1 là một
/// tài liệu Markdown, và mục lục đánh số của nó cho **26 ranh giới sai trên 99** trước luật
/// này (đo 2026-08-12). Xem doc-comment của `split_source_text`.
#[test]
fn a_numbering_marker_is_not_a_sentence() {
    // Nguyen van mot dong muc luc cua Chuong 01 trong bo Epic 1.
    let got = split_source_text(
        "* 0\\. Triet Ly Nen Tang FreshBrand & Mo Hinh 5 Lop Thuong Hieu (Brand Philosophy)",
        "en",
    );
    assert_eq!(
        got.len(),
        1,
        "moc danh sach markdown bi doc la dau ket cau -> {:?}",
        texts(&got)
    );

    // Ca khong dau `\` — mot danh sach danh so binh thuong.
    assert_eq!(split_source_text("1. He went home.", "en").len(), 1);

    // Luat ap cho CA HAI nhanh.
    assert_eq!(split_source_text("1。他走了。", LANG_CHINESE).len(), 1);

    // 🔴 Doi chung duong — luat KHONG duoc nuot mot cau that chi vi no ngan.
    assert_eq!(split_source_text("He left. She stayed.", "en").len(), 2);
    assert_eq!(split_source_text("他走了。她笑了。", LANG_CHINESE).len(), 2);
}

/// Luật thứ năm áp cho ranh giới **dấu kết câu**, KHÔNG cho ranh giới **xuống dòng** — một
/// dòng không có chữ vẫn là một dòng riêng.
#[test]
fn a_line_without_letters_is_still_its_own_segment() {
    let got = split_source_text("Tieu de\n---\nThan bai.", "en");
    assert_eq!(texts(&got), vec!["Tieu de", "---", "Than bai."]);
}

/// Bổ sung Quyết định #5 — dấu đóng ngay sau dấu kết câu ở LẠI với câu mà nó đóng.
#[test]
fn a_closing_quote_stays_with_the_sentence_it_closes() {
    assert_eq!(
        texts(&split_source_text("他说：“真的吗？”她笑了。", LANG_CHINESE)),
        vec!["他说：“真的吗？”", "她笑了。"],
        "mot `”` mo coi o dau segment sau la hinh dang hong ma AD-4 dong bang vinh vien"
    );
    assert_eq!(
        texts(&split_source_text("He said \"Go home.\" She left.", "en")),
        vec!["He said \"Go home.\"", "She left."]
    );
}

// ═════════════════════════════════════════════════════════════════════════════
// Nhánh ngôn ngữ đến từ `work.source_lang`, KHÔNG đoán từ nội dung
// ═════════════════════════════════════════════════════════════════════════════

#[test]
fn the_language_branch_comes_from_source_lang_not_from_the_content() {
    // Van ban tieng Trung, nhung khai `en` ⇒ nhanh tieng Anh, khong dau `。` nao duoc doc.
    assert_eq!(
        split_source_text("他走了。她笑了。", "en").len(),
        1,
        "nhanh tieng Anh khong duoc tach theo `。` -- neu no tach, ma dang doan ngon ngu"
    );

    // Van ban tieng Anh, nhung khai `zh` ⇒ nhanh tieng Trung, khong dau `.` nao duoc doc.
    assert_eq!(
        split_source_text("He left. She stayed.", LANG_CHINESE).len(),
        1,
        "nhanh tieng Trung khong duoc tach theo `.`"
    );

    // Mot gia tri la bat ky ⇒ nhanh tieng Anh (xem doc-comment cua `split_source_text`).
    assert_eq!(split_source_text("He left. She stayed.", "vi").len(), 2);
}

// ═════════════════════════════════════════════════════════════════════════════
// AC10 — bước di trú số 5, và cổng cấm số 4 quay lại
// ═════════════════════════════════════════════════════════════════════════════

/// 🔴 **Cổng còn thiếu mà `deferred-work.md:1169-1180` ghi nợ.**
///
/// `validate_strictly_increasing` KHÔNG bắt được ca này: `[1, 2, 3, 4]` là một danh sách
/// tăng dần nghiêm ngặt hoàn hảo. Số **4** là một số **đã cháy** — một `project.db` tạo ra
/// giữa hai lượt ký của Story 1.20 mang `user_version = 4` với một lược đồ KHÁC. Tái dùng
/// số đó là hai đường lược đồ cho cùng một số, và chúng rẽ nhau ở máy người dùng.
///
/// Chạy đỏ-rồi-xanh: đổi `to_version: 5` thành `4` trong `schema.rs`, ca này phải ĐỎ.
#[test]
fn the_project_migration_set_never_reuses_the_burned_number_four() {
    let burned: Vec<u32> = PROJECT_MIGRATIONS
        .iter()
        .map(|m| m.to_version)
        .filter(|v| *v == 4)
        .collect();

    assert!(
        burned.is_empty(),
        "`PROJECT_MIGRATIONS` khai lai buoc di tru so 4 -- so do DA CHAY (xem \
         `schema.rs`, vet seo Story 1.20). Buoc ke tiep sau 3 la 5."
    );
}

/// 🔴 **Tiêu đề của một bộ di trú phải nói ĐÚNG cái đích mà mảng của nó chạm tới.**
///
/// Doc-comment của [`PROJECT_MIGRATIONS`] tự khai đây là lý do kỷ luật ấy tồn tại: *"Một
/// dòng tiêu đề nói một số mà bảng hằng ngay dưới nói một số khác là đúng thứ rot mà cả
/// kiến trúc này dựa vào doc-comment để tránh"*. Nó đã sai **hai lần** — bắt bằng mắt ở code
/// review 2026-08-11, rồi lại ở vòng rà Epic 3 2026-08-25 (Story 3.10 bump doc-comment của
/// [`GLOBAL_MIGRATIONS`] cho bước song sinh mà bỏ sót bộ kia, nên suốt ba ngày tiêu đề đọc
/// *"mười ba bước, đích là 14"* trên một mảng 14 mục chạm `to_version` 15).
///
/// ⚠️ **Không cổng nào hiện có bắt được ca này, và đó không phải một lượt quên.**
/// `validate_strictly_increasing` chỉ đọc MẢNG; `a_fresh_database_migrates_up_to_target_and_logs_it`
/// đỏ khi **đích** đổi, tức nó cưỡng chế đúng chiều ngược lại — nó bắt người sửa nhận ra
/// mình vừa đổi lược đồ, rồi để mặc người đó bump con số trong văn xuôi hay không. Hai lần
/// hụt liên tiếp là ngưỡng của dự án cho *"đừng canh bằng kỷ luật của người sửa"*.
///
/// 🔴 Ca này đọc **văn bản nguồn** chứ không chỉ hằng, tức lệch một nhịp với câu ở dòng 1 của
/// tệp (*hành vi lúc chạy*). Nó ở ĐÂY thay vì `store_boundary.rs` vì nó là anh em ruột của
/// [`the_project_migration_set_never_reuses_the_burned_number_four`] ngay trên — cùng canh
/// một doc-comment, cùng canh cùng một vết sẹo, và tách hai ca đó ra hai tệp là dựng đúng
/// thứ *"hai bản chép phải đồng bộ bằng tay"* mà `deferred-work.md` đang ghi nợ ở chỗ khác.
///
/// Chạy đỏ-rồi-xanh: hạ *"đích là phiên bản 15"* trong `schema.rs` về **14**, ca này phải ĐỎ.
#[test]
fn the_migration_doc_headers_state_the_target_their_array_reaches() {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src/core/store/schema.rs");
    let text = fs::read_to_string(&path).unwrap_or_else(|e| panic!("doc {}: {e}", path.display()));

    // San quan the: mot duong dan go sai lam `find` truot va MOI khang dinh duoi day xanh ma
    // khong kiem gi ca -- dung bai hoc "cay rong doc thanh sach" cua `store_boundary.rs`.
    // 40_000 = mot PHAN BA kich thuoc that (do 2026-08-25: 127.262 byte). Con so nay canh
    // "duong dan go sai / tep rong / doc truot", KHONG canh "tep teo di" -- mot san sat kich
    // thuoc that se do o moi luot ai do go bot doc-comment, tuc no se bi ha dan cho het do va
    // chet im lang. Sai so ba lan la co y.
    assert!(
        text.len() > 40_000,
        "doc duoc {} byte tu `schema.rs` -- duoi mot phan ba kich thuoc that (127k, do          2026-08-25), qua nho de la tep that. Duong dan da doi?",
        text.len()
    );

    for (name, anchor, migrations) in [
        (
            "GLOBAL_MIGRATIONS",
            "pub const GLOBAL_MIGRATIONS: &[Migration] = &[",
            GLOBAL_MIGRATIONS,
        ),
        (
            "PROJECT_MIGRATIONS",
            "pub const PROJECT_MIGRATIONS: &[Migration] = &[",
            PROJECT_MIGRATIONS,
        ),
    ] {
        let target = migrations
            .iter()
            .map(|m| m.to_version)
            .max()
            .unwrap_or_else(|| panic!("`{name}` rong -- khong con dich nao de doi chieu"));

        // Doc-comment cua hang la doan van ban NGAY TRUOC dong khai bao no. Cat tai anchor
        // roi lui ve dong `pub const` gan nhat phia truoc de khong nham sang doc-comment cua
        // hang KIA -- hai bo nam trong cung mot tep.
        let decl = text
            .find(anchor)
            .unwrap_or_else(|| panic!("khong tim thay khai bao `{name}` -- da doi ten?"));
        let doc_start = text[..decl]
            .rfind("\npub const ")
            .map_or(0, |i| i + 1);
        let doc = &text[doc_start..decl];

        let claim = format!("đích là phiên bản {target}");
        assert!(
            doc.contains(&claim),
            "doc-comment cua `{name}` KHONG chua cau {claim:?}, trong khi mang cua no cham \
             `to_version` {target}. Tieu de noi mot so ma bang hang ngay duoi noi mot so khac \
             la dung thu rot ma chinh doc-comment do goi ten -- sua VAN XUOI cho no noi that, \
             va them mot khoi `🔵 CAP NHAT <ngay> (Story x.y)` thay vi sua lang le."
        );
    }
}

/// 🔵 **CẬP NHẬT 2026-08-14 (Story 2.5).** Ca này trước đây tên
/// `..._reaches_six_through_five_steps` và khẳng định `[1, 2, 3, 5, 6]`. Bước **7** ra đời
/// cùng máy trạng thái AD-31, nên phép kiểm được **nâng cho nó nói thật về lược đồ mới** —
/// không phải nới cho hết đỏ: nó vẫn khẳng định danh sách **nguyên văn**, kể cả lỗ hổng ở 4.
///
/// 🔵 **CẬP NHẬT 2026-08-15 (Story 2.5c, AC7).** Bước **8** ra đời cùng cột `is_omitted`
/// (FR133). Tên hàm đổi theo — nó là một **câu khẳng định**, nên một cái tên nói "bảy qua
/// sáu bước" trên một bộ tám bước là một câu **sai** mà trình biên dịch không bao giờ báo.
///
/// 🔵 **CẬP NHẬT 2026-08-16 (Story 2.5d, AC5) — và lượt này GỠ HẲN SỐ KHỎI TÊN.**
/// Bước **9** ra đời cùng cột `segment.is_target_paragraph_end` (FR134/AD-46). Đây là lần
/// thứ **ba** cái tên phải đổi vì đúng một lý do, và hai lần trước đều là một lượt sửa thủ
/// công mà **không cổng nào** nhắc: một tên mang số hiệu là một câu khẳng định **sai lại ở
/// mỗi story thêm một bước**, và trình biên dịch không bao giờ báo. Story 2.5c đã gỡ số
/// khỏi **bốn** tên khác vì lý do này nhưng để sót tên này.
/// ⇒ Tên mới **không mang số**. Mệnh đề không đổi một chữ: bậc thang khai ở `schema.rs` là
/// bậc thang mà bộ di trú thật sự trèo, **nguyên văn**, kể cả lỗ hổng ở 4.
///
/// 🔵 **CẬP NHẬT 2026-08-19 (Story 3.1).** Bước **12** ra đời cùng bảng `glossary_entry`
/// (AD-18/AD-36, cùng hằng `GLOSSARY_ENTRY_DDL` với bước 4 của `global.db`). Danh sách
/// **nguyên văn** dưới đây nói thật đúng lý do cái tên không mang số: nó đổi lại ở đây mà
/// không đổi một chữ nào của chính hàm test.
///
/// 🔵 **CẬP NHẬT 2026-08-20 (Story 3.2).** Bước **13** ra đời cùng bảng `glossary_candidate`
/// (AD-20/AD-36, KHÔNG có bước song sinh ở `GLOBAL_MIGRATIONS` — bảng chờ chỉ ở tầng Tác
/// phẩm). Danh sách **nguyên văn** dưới đây lại đổi, hàm test lại không đổi một chữ.
///
/// 🔵 **CẬP NHẬT 2026-08-22 (Story 3.5).** Bước **14** ra đời cùng hai cột
/// `occurrence_count`/`context_example` của `glossary_candidate` (KHÔNG có bước song sinh ở
/// `GLOBAL_MIGRATIONS` — cùng lý do bước 13). Danh sách **nguyên văn** dưới đây lại đổi,
/// hàm test lại không đổi một chữ.
///
/// 🔵 **CẬP NHẬT 2026-08-24 (Story 3.10).** Bước **15** ra đời cùng lượt dựng lại
/// `glossary_entry` để thêm giá trị `term_origin` thứ tư, `file_import` (FR49/NFR9, CÙNG một
/// hằng với bước 5 của `global.db`). Danh sách **nguyên văn** dưới đây lại đổi, hàm test lại
/// không đổi một chữ.
///
/// 🔵 **CẬP NHẬT 2026-08-27 (Story 5.4).** Bước **16** ra đời cùng cột
/// `work.status_override` (FR6, KHÔNG có bước song sinh ở `GLOBAL_MIGRATIONS` — bảng `work`
/// chỉ tồn tại ở `project.db`). Danh sách **nguyên văn** dưới đây lại đổi, hàm test lại
/// không đổi một chữ.
///
/// 🔵 **CẬP NHẬT 2026-08-29 (Story 5.7).** Bước **17** ra đời cùng bảng `chapter_position`
/// (AD-3, KHÔNG có bước song sinh ở `GLOBAL_MIGRATIONS` — bảng đó chỉ tồn tại ở
/// `project.db`). Danh sách **nguyên văn** dưới đây lại đổi, hàm test lại không đổi một chữ.
#[test]
fn the_project_migration_set_matches_the_declared_ladder_step_for_step() {
    let versions: Vec<u32> = PROJECT_MIGRATIONS.iter().map(|m| m.to_version).collect();

    assert_eq!(
        versions,
        vec![1, 2, 3, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18],
        "bo di tru cua `project.db` phai la 1 -> 2 -> 3 -> 5 -> 6 -> 7 -> 8 -> 9 -> 10 -> 11 \
         -> 12 -> 13 -> 14 -> 15 -> 16 -> 17 -> 18 (4 la so da chay)"
    );
}

/// 🔴 **Vết sẹo `user_version = 4` chạy THẬT, không chỉ đọc bằng mắt trong doc-comment.**
///
/// Code review 2026-08-12 bắt được: nâng target của `project.db` từ 3 lên 5 **đổi hành vi
/// trên dữ liệu có thật** — một `project.db` mang `user_version = 4` (tạo giữa hai lượt ký
/// của Story 1.20, mang `pinned_entry` mà `project.db` không còn dùng) trước đây bị
/// `Store::open` **từ chối** bằng `store.schema_too_new` (4 > target 3); sau lượt này nó
/// **mở được** và di trú thẳng lên 5. Mệnh đề đó chỉ được ghi ở doc-comment `schema.rs`, và
/// hai ca ngay trên chỉ đọc **danh sách hằng** `[1, 2, 3, 5]` — không ca nào dàn một tệp
/// `.db` thật ở phiên bản 4 rồi mở nó.
///
/// Fixture dựng từ **ba bước THẬT** của `PROJECT_MIGRATIONS` cộng đúng bước 4 đã bị gỡ —
/// cùng luật với `pinned_contract.rs::an_older_global_database_migrates_up_and_keeps_its_rows`:
/// một fixture chép tay sẽ trôi khỏi sự thật đúng vào ngày một bước cũ được sửa.
#[test]
fn a_project_database_stranded_at_the_burned_version_four_opens_and_migrates_past_it() {
    static OLD_STEPS: [Migration; 4] = [
        PROJECT_MIGRATIONS[0],
        PROJECT_MIGRATIONS[1],
        PROJECT_MIGRATIONS[2],
        // Dung buoc 4 ma ban dau Story 1.20 da them roi go — day la thu tao ra vet seo.
        Migration {
            to_version: 4,
            sql: PINNED_ENTRY_DDL,
        },
    ];

    let dir = temp_dir("stranded-at-four");
    let db = dir.join("project.db");

    let stranded = Store::open(StoreSpec {
        migrations: &OLD_STEPS,
        ..StoreSpec::project(db.clone())
    })
    .expect("dung fixture o phien ban 4");
    assert_eq!(
        stranded.schema_version(),
        4,
        "fixture phai dung o dung phien ban 4 -- neu khong ca nay khong kiem gi ca"
    );
    drop(stranded);

    // Day la dong menh de that: bo di tru THAT mo duoc mot tep o phien ban 4.
    let migrated = Store::open(StoreSpec::project(db)).expect(
        "mot `project.db` o `user_version = 4` phai MO DUOC sau khi target len 5 -- \
         truoc Story 2.1 no bi tu choi bang `store.schema_too_new`",
    );
    // 🔵 CAP NHAT 2026-08-14 (Story 2.5): dich chuyen tu 6 len 7 — buoc 7 ra doi. Menh de
    // cua ca nay KHONG doi mot chu: mot tep mac ket o so DA CHAY phai di qua duoc **moi**
    // buoc con lai, khong dung o buoc dau tien sau no.
    // 🔵 CAP NHAT 2026-08-15 (Story 2.5c): dich 7 → 8 — buoc 8 ra doi. Menh de van khong doi.
    // 🔵 CAP NHAT 2026-08-16 (Story 2.5d): dich 8 → 9 — buoc 9 ra doi. Menh de van khong doi.
    // 🔵 CAP NHAT 2026-08-16 (Story 2.6): dich 9 → 10 — buoc 10 ra doi. Menh de van khong doi.
    // 🔵 CAP NHAT 2026-08-19 (Story 3.1): dich 11 → 12 — buoc 12 (glossary_entry) ra doi.
    // Menh de van khong doi.
    // 🔵 CAP NHAT 2026-08-20 (Story 3.2): dich 12 → 13 — buoc 13 (glossary_candidate) ra doi.
    // Menh de van khong doi.
    // 🔵 CAP NHAT 2026-08-22 (Story 3.5): dich 13 → 14 — buoc 14 (occurrence_count/
    // context_example) ra doi. Menh de van khong doi.
    // 🔵 CAP NHAT 2026-08-24 (Story 3.10): dich 14 → 15 — buoc 15 (dung lai glossary_entry,
    // gia tri term_origin thu tu) ra doi. Menh de van khong doi.
    // 🔵 CAP NHAT 2026-08-27 (Story 5.4): dich 15 → 16 — buoc 16 (work.status_override,
    // FR6) ra doi. Menh de van khong doi.
    assert_eq!(
        migrated.schema_version(),
        18,
        "buoc 5..18 phai da chay tren mot tep dung o phien ban 4"
    );

    let has_segment: i64 = migrated
        .read(|conn| {
            conn.query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type = 'table' AND name = 'segment'",
                [],
                |r| r.get(0),
            )
        })
        .expect("dem bang segment");
    assert_eq!(has_segment, 1, "buoc 5 phai dung bang `segment`");

    // Bang mo coi di theo, va do la mot GHI CHEP chu khong phai mot loi -- xem `schema.rs`.
    let orphan: i64 = migrated
        .read(|conn| {
            conn.query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type = 'table' AND name = 'pinned_entry'",
                [],
                |r| r.get(0),
            )
        })
        .expect("dem bang mo coi");
    assert_eq!(
        orphan, 1,
        "`pinned_entry` mo coi phai O LAI -- di tru khong duoc tu y xoa bang cua nguoi dung"
    );

    drop(migrated);
    cleanup(&dir);
}

// ═════════════════════════════════════════════════════════════════════════════
// Story 2.2 · AC13 · Task 1 — cột `target_text` và bước di trú 6
// ═════════════════════════════════════════════════════════════════════════════

/// Một `project.db` **mới** dừng ở phiên bản **6**, và bảng `segment` mang `target_text`.
///
/// ⚠️ Hai mệnh đề, không một: số phiên bản là PROXY, còn mệnh đề thật là phép đọc
/// `PRAGMA table_info` ngay dưới. Một bước di trú khai đúng số mà chạy sai DDL đi lọt phép
/// kiểm thứ nhất.
/// 🔵 **CẬP NHẬT 2026-08-14 (Story 2.5).** Ca này trước tên
/// `..._lands_at_version_six_with_a_target_text_column` và mở đầu bằng một khẳng định
/// *"phiên bản mới phải là 6"*. Khẳng định đó **đã hết đúng** (đích nay là 7), và nó cũng
/// **không phải chủ đề** của ca này — chủ đề là **hình dạng cột `target_text`**. Mệnh đề
/// *"một `project.db` mới dừng ở đâu"* nay có đúng một chủ:
/// `a_fresh_project_database_lands_at_version_seven_with_a_status_column_and_a_version_table`.
/// ⇒ Gỡ khẳng định trùng thay vì sửa số ở hai chỗ — hai chỗ khai cùng một mệnh đề là hai
/// chỗ sẽ lệch nhau.
#[test]
fn a_fresh_project_database_carries_a_non_null_target_text_column_defaulting_to_empty() {
    let root = temp_dir("fresh-target-text");
    let opened = create_work_from_text(&root, "Sau", "zh", "", "一。二。".to_owned())
        .expect("tao tac pham that bai");

    let (notnull, default_value): (i64, String) = opened
        .store
        .read(|conn| {
            conn.query_row(
                "SELECT \"notnull\", COALESCE(dflt_value, '<NULL>') FROM pragma_table_info('segment') \
                 WHERE name = 'target_text'",
                [],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
        })
        .expect("cot `target_text` phai co mat trong `segment`");

    assert_eq!(
        notnull, 1,
        "`target_text` phai `NOT NULL` -- \"chua dich\" la CHUOI RONG, khong phai mot gia \
         tri vang mat (Story 2.2, Task 1.4)"
    );
    assert_eq!(
        default_value, "''",
        "`target_text` phai mac dinh chuoi rong -- mot `DEFAULT NULL` lam `ADD COLUMN NOT \
         NULL` khong chay duoc tren bang da co du lieu"
    );

    let blanks: i64 = opened
        .store
        .read(|conn| {
            conn.query_row(
                "SELECT COUNT(*) FROM segment WHERE target_text = ''",
                [],
                |r| r.get(0),
            )
        })
        .expect("dem segment chua dich");
    assert_eq!(
        blanks, 2,
        "moi segment vua nhap phai o trang thai \"chua dich\" -- chuoi rong, khong du lieu moi"
    );

    let dir = opened.dir.clone();
    drop(opened);
    cleanup(&dir);
    cleanup(&root);
}

/// 🔴 **Một `project.db` ĐANG ở phiên bản 5 di trú lên 6 mà KHÔNG mất một hàng `segment` nào.**
///
/// Đây là ca duy nhất chạm đúng đường mà dữ liệu thật của Ice sẽ đi: 21 `project.db` đã ở
/// phiên bản 5 kể từ Story 2.1, mang **10.477** hàng `segment`. Hai ca ở trên chỉ dựng tệp
/// mới; một bước 6 viết bằng `DROP TABLE` + `CREATE TABLE` sẽ đi lọt cả hai.
///
/// Fixture dựng từ **bốn bước THẬT** của [`PROJECT_MIGRATIONS`], không chép tay DDL — cùng
/// luật với ca `stranded_at_the_burned_version_four` ngay trên.
#[test]
fn a_project_database_at_version_five_migrates_up_and_keeps_every_segment_row() {
    static STEPS_TO_FIVE: [Migration; 4] = [
        PROJECT_MIGRATIONS[0],
        PROJECT_MIGRATIONS[1],
        PROJECT_MIGRATIONS[2],
        PROJECT_MIGRATIONS[3],
    ];

    let dir = temp_dir("five-to-six");
    let db = dir.join("project.db");

    let old = Store::open(StoreSpec {
        migrations: &STEPS_TO_FIVE,
        ..StoreSpec::project(db.clone())
    })
    .expect("dung fixture o phien ban 5");
    assert_eq!(
        old.schema_version(),
        5,
        "fixture phai dung o dung phien ban 5 -- neu khong ca nay khong kiem gi ca"
    );

    // ⚠️ KHONG chen hang `chapter` nao: `SEGMENT_DDL` co y KHONG khai `FOREIGN KEY` (xem
    // doc-comment cua no), nen ca nay chi can dung ba hang `segment` de do buoc 6.
    old.write(|tx: &Transaction<'_>| {
        for ord in 1..=3i64 {
            tx.execute(
                "INSERT INTO segment (chapter_id, ord, source_text, is_paragraph_end, created_at, updated_at) \
                 VALUES (1, ?1, ?2, 0, '2026-08-12T00:00:00.000Z', '2026-08-12T00:00:00.000Z')",
                (ord, format!("cau {ord}")),
            )?;
        }
        Ok(())
    })
    .expect("bom ba hang segment vao fixture");
    drop(old);

    let migrated = Store::open(StoreSpec::project(db))
        .expect("mot `project.db` o phien ban 5 phai mo duoc va di tru len dich");
    // 🔵 CAP NHAT 2026-08-14 (Story 2.5): dich 6 → 7. Chu de cua ca nay khong doi — no do
    // menh de "buoc 6 la mot `ALTER TABLE`, khong mot `DROP` + `CREATE`" bang cach dem lai
    // ba hang cu; nay no do menh de do cho **ca hai** buoc 6 va 7 cung mot luot.
    // 🔵 CAP NHAT 2026-08-15 (Story 2.5c): dich 7 → 8, va nay la BA buoc mot luot.
    // 🔵 CAP NHAT 2026-08-16 (Story 2.6): dich 9 → 10, nay la NAM buoc mot luot.
    // 🔵 CAP NHAT 2026-08-19 (Story 3.1): dich 11 → 12, nay la SAU buoc mot luot.
    // 🔵 CAP NHAT 2026-08-20 (Story 3.2): dich 12 → 13, nay la BAY buoc mot luot.
    // 🔵 CAP NHAT 2026-08-22 (Story 3.5): dich 13 → 14, nay la TAM buoc mot luot.
    // 🔵 CAP NHAT 2026-08-24 (Story 3.10): dich 14 → 15, nay la CHIN buoc mot luot.
    // 🔵 CAP NHAT 2026-08-27 (Story 5.4): dich 15 → 16 — buoc 16 (work.status_override,
    // FR6) ra doi. Menh de van khong doi.
    assert_eq!(
        migrated.schema_version(),
        18,
        "buoc 6..18 phai chay tren mot tep dung o phien ban 5"
    );

    let rows: Vec<(i64, String, String)> = migrated
        .read(|conn| {
            let mut stmt =
                conn.prepare("SELECT ord, source_text, target_text FROM segment ORDER BY ord")?;
            let mapped = stmt.query_map([], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)))?;
            mapped.collect::<Result<Vec<_>, _>>()
        })
        .expect("doc lai ba hang segment sau di tru");

    assert_eq!(
        rows,
        vec![
            (1, "cau 1".to_owned(), String::new()),
            (2, "cau 2".to_owned(), String::new()),
            (3, "cau 3".to_owned(), String::new()),
        ],
        "buoc 6 phai la mot `ALTER TABLE` -- moi hang `segment` cu phai o lai nguyen ven, \
         va `target_text` cua chung la CHUOI RONG"
    );

    drop(migrated);
    cleanup(&dir);
}

// ═════════════════════════════════════════════════════════════════════════════
// Story 2.5 · AC9 — bước di trú 7: cột `segment.status` + bảng `segment_version`
// ═════════════════════════════════════════════════════════════════════════════

/// Một `project.db` **mới** dừng ở phiên bản đích, mang cột `status` và bảng
/// `segment_version`.
///
/// ⚠️ Ba mệnh đề, không một: số phiên bản là PROXY; mệnh đề thật là hai phép đọc
/// `pragma_table_info` ngay dưới. Một bước di trú khai đúng số mà chạy sai DDL đi lọt
/// phép kiểm thứ nhất — cùng luật ca `..._lands_at_version_six_...` đã đặt.
///
/// 🔵 **CẬP NHẬT 2026-08-15 (Story 2.5c): số hiệu ĐÍCH gỡ khỏi TÊN hàm, giữ trong phép
/// khẳng định.** Tên cũ nói *"lands at version seven"* và nó **sai** ngay khi bước 8 vào —
/// tức mỗi story thêm một bước lại phải đổi tên một ca **không nói về bước của nó**. Số
/// vẫn viết **thẳng** ở `assert_eq!` chứ không dẫn xuất từ `PROJECT_MIGRATIONS`: một phán
/// quyết đọc tham số từ chính thứ nó đang kiểm thì không phán quyết gì cả.
#[test]
fn a_fresh_project_database_lands_at_the_target_with_a_status_column_and_a_version_table() {
    let root = temp_dir("fresh-at-target");
    let opened = create_work_from_text(&root, "Bay", "zh", "", "一。二。".to_owned())
        .expect("tao tac pham that bai");

    // 🔵 CAP NHAT 2026-08-16 (Story 2.6): dich 9 → 10 — buoc 10 (index tren
    // `segment_version`). Menh de cua ca nay KHONG doi mot chu.
    // 🔵 CAP NHAT 2026-08-19 (Story 3.1): dich 11 → 12 — buoc 12 (`glossary_entry`,
    // AD-18/AD-36). Menh de cua ca nay KHONG doi mot chu.
    // 🔵 CAP NHAT 2026-08-20 (Story 3.2): dich 12 → 13 — buoc 13 (`glossary_candidate`,
    // AD-20/AD-36). Menh de cua ca nay KHONG doi mot chu.
    // 🔵 CAP NHAT 2026-08-22 (Story 3.5): dich 13 → 14 — buoc 14 (occurrence_count/
    // context_example). Menh de cua ca nay KHONG doi mot chu.
    // 🔵 CAP NHAT 2026-08-24 (Story 3.10): dich 14 → 15 — buoc 15 (dung lai glossary_entry,
    // gia tri term_origin thu tu). Menh de cua ca nay KHONG doi mot chu.
    // 🔵 CAP NHAT 2026-08-27 (Story 5.4): dich 15 → 16 — buoc 16 (work.status_override, FR6).
    // Menh de cua ca nay KHONG doi mot chu.
    // 🔵 CAP NHAT 2026-08-29 (Story 5.7): dich 16 → 17 — buoc 17 (chapter_position, AD-3).
    // Menh de cua ca nay KHONG doi mot chu.
    assert_eq!(
        opened.store.schema_version(),
        18,
        "mot `project.db` moi phai dung o phien ban 18 (Story 5.13 them reading_mark)"
    );

    let (notnull, default_value): (i64, String) = opened
        .store
        .read(|conn| {
            conn.query_row(
                "SELECT \"notnull\", COALESCE(dflt_value, '<NULL>') FROM pragma_table_info('segment') \
                 WHERE name = 'status'",
                [],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
        })
        .expect("cot `status` phai co mat trong `segment`");

    assert_eq!(
        notnull, 1,
        "`status` phai `NOT NULL` -- mot segment KHONG co trang thai la mot trang thai thu \
         ba khong ai khai, va no se duoc doc thanh \"chua xac nhan\" o cho nay va \"loi\" o cho khac"
    );
    assert_eq!(
        default_value, "'draft'",
        "`status` phai mac dinh `'draft'` -- SQLite doi mot DEFAULT khac NULL cho moi \
         `ADD COLUMN NOT NULL` tren bang da co du lieu (10.477 hang tren du lieu that)"
    );

    // Bang `segment_version` co that, va DUNG bon cot -- Quyet dinh #6 (Ice ky 2026-08-14).
    let cols: Vec<String> = opened
        .store
        .read(|conn| {
            let mut stmt =
                conn.prepare("SELECT name FROM pragma_table_info('segment_version') ORDER BY cid")?;
            let mapped = stmt.query_map([], |r| r.get(0))?;
            mapped.collect::<Result<Vec<_>, _>>()
        })
        .expect("bang `segment_version` phai co mat");

    assert_eq!(
        cols,
        vec!["id", "segment_id", "target_text", "created_at"],
        "`segment_version` phai TOI GIAN dung bon cot -- Story 2.6 doc chung, khong them cot moi"
    );

    // Moi segment vua nhap phai o `'draft'` -- khong mot gia tri moi nao.
    let drafts: i64 = opened
        .store
        .read(|conn| {
            conn.query_row(
                "SELECT COUNT(*) FROM segment WHERE status = 'draft'",
                [],
                |r| r.get(0),
            )
        })
        .expect("dem segment o trang thai draft");
    assert_eq!(drafts, 2, "moi segment vua nhap phai o `'draft'`");

    let dir = opened.dir.clone();
    drop(opened);
    cleanup(&dir);
    cleanup(&root);
}

/// 🔴 **Một `project.db` ĐANG ở phiên bản 6 di trú lên 7 mà KHÔNG mất một hàng nào, và
/// mọi hàng cũ nhận đúng `'draft'`.**
///
/// Đây là ca chạm đúng đường mà dữ liệu thật của Ice sẽ đi: 21 `project.db` đã ở phiên bản
/// 6 kể từ Story 2.2. Ca `..._lands_at_version_seven_...` ở trên chỉ dựng tệp **mới**; một
/// bước 7 viết bằng `DROP TABLE` + `CREATE TABLE` sẽ đi lọt nó.
///
/// Fixture dựng từ **năm bước THẬT** của [`PROJECT_MIGRATIONS`], không chép tay DDL.
#[test]
fn a_project_database_at_version_six_migrates_up_and_every_old_row_becomes_draft() {
    static STEPS_TO_SIX: [Migration; 5] = [
        PROJECT_MIGRATIONS[0],
        PROJECT_MIGRATIONS[1],
        PROJECT_MIGRATIONS[2],
        PROJECT_MIGRATIONS[3],
        PROJECT_MIGRATIONS[4],
    ];

    let dir = temp_dir("six-to-seven");
    let db = dir.join("project.db");

    let old = Store::open(StoreSpec {
        migrations: &STEPS_TO_SIX,
        ..StoreSpec::project(db.clone())
    })
    .expect("dung fixture o phien ban 6");
    assert_eq!(
        old.schema_version(),
        6,
        "fixture phai dung o dung phien ban 6 -- neu khong ca nay khong kiem gi ca"
    );

    old.write(|tx: &Transaction<'_>| {
        for ord in 1..=3i64 {
            tx.execute(
                "INSERT INTO segment (chapter_id, ord, source_text, target_text, is_paragraph_end, \
                 created_at, updated_at) \
                 VALUES (1, ?1, ?2, ?3, 0, '2026-08-13T00:00:00.000Z', '2026-08-13T00:00:00.000Z')",
                (ord, format!("cau {ord}"), format!("ban dich {ord}")),
            )?;
        }
        Ok(())
    })
    .expect("bom ba hang segment vao fixture");
    drop(old);

    let migrated = Store::open(StoreSpec::project(db))
        .expect("mot `project.db` o phien ban 6 phai mo duoc va di tru len dich");
    // 🔵 CAP NHAT 2026-08-15 (Story 2.5c): dich 7 → 8. Chu de cua ca nay khong doi — no do
    // menh de "buoc 7 backfill 'draft'", va buoc 8 chay them mot luot khong dung toi `status`.
    // 🔵 CAP NHAT 2026-08-16 (Story 2.6): dich 9 → 10, va buoc 10 cung khong dung toi `status`.
    // 🔵 CAP NHAT 2026-08-19 (Story 3.1): dich 11 → 12, va buoc 12 cung khong dung toi `status`.
    // 🔵 CAP NHAT 2026-08-20 (Story 3.2): dich 12 → 13, va buoc 13 cung khong dung toi `status`.
    // 🔵 CAP NHAT 2026-08-22 (Story 3.5): dich 13 → 14, va buoc 14 cung khong dung toi `status`.
    // 🔵 CAP NHAT 2026-08-24 (Story 3.10): dich 14 → 15, va buoc 15 cung khong dung toi `status`.
    // 🔵 CAP NHAT 2026-08-27 (Story 5.4): dich 15 → 16 — buoc 16 (work.status_override,
    // FR6) ra doi. Menh de van khong doi.
    assert_eq!(
        migrated.schema_version(),
        18,
        "buoc 7..18 phai chay tren mot tep dung o phien ban 6"
    );

    let rows: Vec<(i64, String, String, String)> = migrated
        .read(|conn| {
            let mut stmt = conn.prepare(
                "SELECT ord, source_text, target_text, status FROM segment ORDER BY ord",
            )?;
            let mapped = stmt.query_map([], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?)))?;
            mapped.collect::<Result<Vec<_>, _>>()
        })
        .expect("doc lai ba hang segment sau di tru");

    assert_eq!(
        rows,
        vec![
            (1, "cau 1".to_owned(), "ban dich 1".to_owned(), "draft".to_owned()),
            (2, "cau 2".to_owned(), "ban dich 2".to_owned(), "draft".to_owned()),
            (3, "cau 3".to_owned(), "ban dich 3".to_owned(), "draft".to_owned()),
        ],
        "buoc 7 phai la mot `ALTER TABLE` -- moi hang cu o lai nguyen ven, VA `status` cua \
         chung la `'draft'`. 🔴 Mot ban dich CU khong duoc tu dong thanh \"da xac nhan\": \
         khong ai ky no, va mot lan ky gia se ghi mot cap TM chua ai duyet o Epic 7"
    );

    // Bang lich su phai RONG sau di tru -- khong mot phien ban mo nao duoc sinh ra.
    let versions: i64 = migrated
        .read(|conn| conn.query_row("SELECT COUNT(*) FROM segment_version", [], |r| r.get(0)))
        .expect("dem hang segment_version");
    assert_eq!(
        versions, 0,
        "buoc di tru KHONG duoc sinh mot `SegmentVersion` nao -- no la DDL, khong phai mot \
         quy tac nghiep vu (Quyet dinh #4 cua Story 2.1)"
    );

    drop(migrated);
    cleanup(&dir);
}

/// **Cột `is_omitted` có mặt trên một `project.db` mới, và có ĐÚNG hình dạng đã ký** —
/// Story 2.5c, AC7 · Quyết định #5 đường (a).
///
/// ⚠️ Số phiên bản là một **PROXY**; mệnh đề thật là phép đọc `pragma_table_info` ngay
/// dưới. Một bước di trú khai đúng số 8 mà chạy sai DDL đi lọt mọi ca đếm phiên bản — cùng
/// luật ca `..._lands_at_the_target_...` đã đặt cho bước 7.
///
/// 🔴 Ca này canh cả **ba** vế mà Quyết định #5(a) chốt, và mỗi vế hỏng theo một kiểu khác:
/// `NOT NULL` *(một segment không có cờ là một trạng thái thứ ba không ai khai)* · `DEFAULT
/// 0` *(thiếu nó thì `ADD COLUMN NOT NULL` không chạy nổi trên bảng đã có dữ liệu)* · và
/// **không `CHECK`** *(một `CHECK` ở đây dựng quy ước thứ hai cho cùng một việc, khác hẳn
/// `status` và `chapter.status`)*.
#[test]
fn a_fresh_project_database_carries_an_is_omitted_column_with_the_shape_ice_signed() {
    let root = temp_dir("fresh-is-omitted");
    let opened = create_work_from_text(&root, "Cat Bo", "zh", "", "一。二。".to_owned())
        .expect("tao tac pham that bai");

    let (col_type, notnull, default_value): (String, i64, String) = opened
        .store
        .read(|conn| {
            conn.query_row(
                "SELECT type, \"notnull\", COALESCE(dflt_value, '<NULL>') \
                 FROM pragma_table_info('segment') WHERE name = 'is_omitted'",
                [],
                |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
            )
        })
        .expect("cot `is_omitted` phai co mat trong `segment`");

    assert_eq!(
        col_type, "INTEGER",
        "`is_omitted` phai la INTEGER -- Quyet dinh #5 duong (a), khuon \
         `is_paragraph_end`. Duong (b) (`omitted_at TEXT`) da bi Ice loai 2026-08-15"
    );
    assert_eq!(
        notnull, 1,
        "`is_omitted` phai `NOT NULL` -- mot segment KHONG co co la mot trang thai thu ba \
         khong ai khai, va no se duoc doc thanh \"khong cat bo\" o cho nay va \"loi\" o cho khac"
    );
    assert_eq!(
        default_value, "0",
        "`is_omitted` phai mac dinh 0 -- SQLite doi mot DEFAULT khac NULL cho moi \
         `ADD COLUMN NOT NULL` tren bang da co du lieu, VA 0 la su that ve moi hang co san"
    );

    // KHONG `CHECK` -- cung khuon `status` va `chapter.status`. `sqlite_master` chi mot
    // cach doc duoc menh de nay: cau DDL cua bang khong duoc mang tu `CHECK`.
    let ddl: String = opened
        .store
        .read(|conn| {
            conn.query_row(
                "SELECT sql FROM sqlite_master WHERE type = 'table' AND name = 'segment'",
                [],
                |r| r.get(0),
            )
        })
        .expect("doc DDL cua bang `segment`");
    assert!(
        !ddl.to_uppercase().contains("CHECK"),
        "bang `segment` mang mot rang buoc `CHECK` -- gia tri hop le cuong che o tang Rust, \
         dung khuon `status` va `chapter.status` (Quyet dinh #5, 2026-08-15). DDL: {ddl}"
    );

    let dir = opened.dir.clone();
    drop(opened);
    cleanup(&dir);
    cleanup(&root);
}

// ═════════════════════════════════════════════════════════════════════════════
// Story 2.6 · AC1 · AC5 — bước 10: index cho đường ĐỌC lịch sử phiên bản
// ═════════════════════════════════════════════════════════════════════════════

/// 🔴 **Index của `segment_version` có đúng hình dạng Ice ký, và bảng KHÔNG mọc thêm ràng
/// buộc nào.**
///
/// Ca này đọc `sqlite_master` chứ không tin hằng DDL: một hằng đúng mà bước di trú quên gắn
/// vào [`PROJECT_MIGRATIONS`] thì hằng vẫn đúng và **đĩa vẫn không có index nào**. Đây là
/// đúng lớp lỗi mà Story 2.5 đã gặp thật ở một hình dạng khác *(cột thêm vào kiểu TypeScript
/// mà quên thêm vào struct và vào `SELECT`)*: **thứ được khai và thứ chạy là hai chuyện.**
///
/// Hai nửa của phép kiểm, cố ý không tách thành hai ca — chúng nói về cùng một câu DDL:
/// ① index tồn tại, đúng tên, đúng bảng, đúng **hai cột theo đúng thứ tự** *(Quyết định #7
///   đường (a): `(segment_id, created_at DESC)`)*;
/// ② `segment_version` **không** mọc thêm `CHECK` và **không** mọc thêm `FOREIGN KEY`.
///
/// 🔴 Vế ② không phải một phép kiểm thừa. Bảng này cố ý **không** có khoá ngoại — AD-5 nói
/// *"về hưu = tombstone"*, không phải một lượt xoá — và đó là thứ làm **AC4 đúng theo cấu
/// trúc**: lịch sử của một segment đã về hưu không đi đâu cả. Một `ON DELETE CASCADE` thêm
/// vào "cho chặt chẽ" ở một story sau sẽ **phá AC4 mà không một ca nào khác đỏ**.
#[test]
fn the_segment_version_index_has_the_shape_ice_signed_and_the_table_grew_no_new_constraint() {
    let root = temp_dir("fresh-version-index");
    let opened = create_work_from_text(&root, "Lich Su", "zh", "", "一。二。".to_owned())
        .expect("tao tac pham that bai");

    // ① Index co mat, dung ten, dung bang -- doc tu `sqlite_master`, khong tu hang DDL.
    let index_sql: String = opened
        .store
        .read(|conn| {
            conn.query_row(
                "SELECT COALESCE(sql, '<NULL>') FROM sqlite_master \
                 WHERE type = 'index' AND name = 'idx_segment_version_segment_created'",
                [],
                |r| r.get(0),
            )
        })
        .expect(
            "index `idx_segment_version_segment_created` phai co mat tren dia -- buoc di tru \
             10 (Story 2.6, Quyet dinh #7 duong (a)). Khong thay no tuc hang DDL dung ma \
             `PROJECT_MIGRATIONS` chua gan buoc vao",
        );

    let normalized = index_sql.to_uppercase();
    assert!(
        normalized.contains("ON SEGMENT_VERSION"),
        "index phai nam tren bang `segment_version`. DDL doc duoc: {index_sql}"
    );

    // Hai cot, DUNG THU TU. Doc qua `pragma_index_info` thay vi so chuoi: thu tu cot la thu
    // ca nay noi ve, va mot phep so chuoi tho se xanh voi `(created_at, segment_id)` neu ai
    // do doi cho hai cot.
    let cols: Vec<String> = opened
        .store
        .read(|conn| {
            let mut stmt = conn.prepare(
                "SELECT name FROM pragma_index_info('idx_segment_version_segment_created') \
                 ORDER BY seqno",
            )?;
            let rows = stmt.query_map([], |r| r.get::<_, String>(0))?;
            rows.collect::<Result<Vec<String>, _>>()
        })
        .expect("doc cac cot cua index that bai");
    assert_eq!(
        cols,
        vec!["segment_id".to_owned(), "created_at".to_owned()],
        "index phai la `(segment_id, created_at)` DUNG THU TU DO -- AC1 loc theo \
         `segment_id` roi sap theo thoi diem, nen `segment_id` phai dung truoc"
    );
    assert!(
        normalized.contains("CREATED_AT DESC"),
        "cot `created_at` phai giam dan (`DESC`) -- AC1 doi \"moi nhat truoc\". DDL: {index_sql}"
    );

    // ② Bang KHONG mang `CHECK`, KHONG mang khoa ngoai.
    let table_ddl: String = opened
        .store
        .read(|conn| {
            conn.query_row(
                "SELECT sql FROM sqlite_master WHERE type = 'table' AND name = 'segment_version'",
                [],
                |r| r.get(0),
            )
        })
        .expect("doc DDL cua bang `segment_version`");
    let table_up = table_ddl.to_uppercase();
    assert!(
        !table_up.contains("CHECK"),
        "bang `segment_version` mang mot rang buoc `CHECK` -- gia tri hop le cuong che o \
         tang Rust, dung khuon `status`, `is_omitted` va `chapter.status`. DDL: {table_ddl}"
    );
    assert!(
        !table_up.contains("FOREIGN KEY") && !table_up.contains("REFERENCES"),
        "bang `segment_version` mang mot khoa ngoai -- no CO Y khong co. AD-5 noi \"ve huu = \
         tombstone\", khong mot luot xoa, va chinh viec khong co khoa ngoai la thu lam AC4 \
         (\"segment da ve huu van tra duoc lich su\") dung THEO CAU TRUC. DDL: {table_ddl}"
    );

    let dir = opened.dir.clone();
    drop(opened);
    cleanup(&dir);
    cleanup(&root);
}

/// 🔴 **Bước 10 chạy trên dữ liệu ĐÃ CÓ và KHÔNG đụng một hàng `segment_version` nào.**
///
/// Khuôn `a_project_database_at_version_eight_backfills_the_target_flag_...`, nhưng mệnh đề
/// **ngược lại**: bước 9 cố ý sửa dữ liệu *(backfill cờ đích)*, còn bước 10 cố ý **không**.
/// Index là một cấu trúc **dẫn xuất** — SQLite dựng nó từ dữ liệu đã có ngay trong câu
/// `CREATE`, và không hàng nào đổi một byte.
///
/// ⚠️ Fixture dựng bằng các bước **THẬT** của [`PROJECT_MIGRATIONS`], không chép tay DDL.
/// Một fixture chép tay là một nguồn sự thật thứ hai cho lược đồ, và nó **trôi khỏi** hằng
/// thật ở đúng story mà hằng thật đổi — tức đúng lúc ca này cần nói thật nhất.
#[test]
fn a_project_database_at_version_nine_gains_the_index_and_no_version_row_is_touched() {
    let dir = temp_dir("v9-gains-index");
    let db = dir.join("project.db");

    // Fixture o phien ban 9: chin buoc THAT tru buoc cuoi.
    static THROUGH_NINE: [Migration; 8] = [
        PROJECT_MIGRATIONS[0],
        PROJECT_MIGRATIONS[1],
        PROJECT_MIGRATIONS[2],
        PROJECT_MIGRATIONS[3],
        PROJECT_MIGRATIONS[4],
        PROJECT_MIGRATIONS[5],
        PROJECT_MIGRATIONS[6],
        PROJECT_MIGRATIONS[7],
    ];

    let old = Store::open(StoreSpec {
        migrations: &THROUGH_NINE,
        ..StoreSpec::project(db.clone())
    })
    .expect("dung fixture o phien ban 9");
    assert_eq!(
        old.schema_version(),
        9,
        "fixture phai dung o 9 -- neu no da la 10 thi ca nay khong do gi ca"
    );

    // Bom bon hang `segment_version` THAT vao fixture, kem thoi diem tang dan.
    old.write(|tx: &Transaction<'_>| {
        tx.execute(
            "INSERT INTO segment (id, chapter_id, ord, source_text, is_paragraph_end, \
             created_at, updated_at) VALUES (1, 1, 1, 'mot', 0, 'x', 'x')",
            [],
        )?;
        for (id, text, at) in [
            (1_i64, "ban dau", "2026-08-16T09:00:00.000Z"),
            (2, "ban hai", "2026-08-16T10:00:00.000Z"),
            (3, "ban ba", "2026-08-16T11:00:00.000Z"),
            (4, "ban bon", "2026-08-16T12:00:00.000Z"),
        ] {
            tx.execute(
                "INSERT INTO segment_version (id, segment_id, target_text, created_at) \
                 VALUES (?1, 1, ?2, ?3)",
                (id, text, at),
            )?;
        }
        Ok(())
    })
    .expect("bom bon hang segment_version vao fixture");

    let before: Vec<(i64, i64, String, String)> = old
        .read(|conn| {
            let mut stmt = conn.prepare(
                "SELECT id, segment_id, target_text, created_at FROM segment_version ORDER BY id",
            )?;
            let rows = stmt.query_map([], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?)))?;
            rows.collect::<Result<Vec<_>, _>>()
        })
        .expect("doc bon hang truoc luot di tru");
    drop(old);

    // Di tru len dich.
    let migrated =
        Store::open(StoreSpec::project(db)).expect("mot `project.db` o phien ban 9 phai mo duoc");
    // 🔵 CAP NHAT 2026-08-19 (Story 3.1): dich 11 → 12 — buoc 12 (`glossary_entry`) ra doi.
    // 🔵 CAP NHAT 2026-08-20 (Story 3.2): dich 12 → 13 — buoc 13 (`glossary_candidate`) ra doi.
    // 🔵 CAP NHAT 2026-08-22 (Story 3.5): dich 13 → 14 — buoc 14 ra doi.
    // 🔵 CAP NHAT 2026-08-24 (Story 3.10): dich 14 → 15 — buoc 15 ra doi.
    // 🔵 CAP NHAT 2026-08-27 (Story 5.4): dich 15 → 16 — buoc 16 (work.status_override,
    // FR6) ra doi. Menh de van khong doi.
    assert_eq!(
        migrated.schema_version(),
        18,
        "buoc 10..18 phai chay tren mot tep dung o phien ban 9"
    );

    // Index co mat SAU luot di tru -- day la nua "buoc 10 that su da chay".
    let index_count: i64 = migrated
        .read(|conn| {
            conn.query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type = 'index' \
                 AND name = 'idx_segment_version_segment_created'",
                [],
                |r| r.get(0),
            )
        })
        .expect("dem index that bai");
    assert_eq!(
        index_count, 1,
        "index phai co mat sau luot di tru tu 9 len 10"
    );

    // Va KHONG hang nao doi mot byte.
    let after: Vec<(i64, i64, String, String)> = migrated
        .read(|conn| {
            let mut stmt = conn.prepare(
                "SELECT id, segment_id, target_text, created_at FROM segment_version ORDER BY id",
            )?;
            let rows = stmt.query_map([], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?)))?;
            rows.collect::<Result<Vec<_>, _>>()
        })
        .expect("doc bon hang sau luot di tru");

    assert_eq!(
        after, before,
        "buoc 10 KHONG duoc dung toi mot hang `segment_version` nao -- no chi dung mot cau \
         truc dan xuat. Khac buoc 9 (backfill co dich), buoc nay thuan DDL"
    );

    cleanup(&dir);
}

/// 🔴 **Một `project.db` ĐANG ở phiên bản 7 di trú lên 8 mà KHÔNG mất một hàng nào, và mọi
/// hàng cũ nhận `is_omitted = 0`** — Story 2.5c, AC7 · Task 1.7.
///
/// Cùng vai với ca `..._six_migrates_to_seven_...` ngay trên, cho bước kế tiếp: dữ liệu
/// thật của Ice *(đo 2026-08-12: **10.477** hàng `segment` từ 21 Chương)* đi **đúng** đường
/// này, và một bước 8 viết bằng `DROP TABLE` + `CREATE TABLE` sẽ đi lọt mọi ca dựng tệp mới.
///
/// 🔴 Giá trị backfill `0` là một **quyết định nghiệp vụ**, không một chi tiết kỹ thuật:
/// *"chưa ai bấm cắt bỏ câu này"* là sự thật về mọi hàng có sẵn. Backfill `1` sẽ **xoá sạch
/// bản dịch của người dùng khỏi mọi đầu ra** trong im lặng — AC5 nói *"ẩn hoàn toàn, không
/// dấu vết"*, nên một cờ đặt nhầm ở đây không biểu hiện thành lỗi, nó biểu hiện thành **văn
/// bản biến mất**.
#[test]
fn a_project_database_at_version_seven_migrates_up_and_no_old_row_is_omitted() {
    static STEPS_TO_SEVEN: [Migration; 6] = [
        PROJECT_MIGRATIONS[0],
        PROJECT_MIGRATIONS[1],
        PROJECT_MIGRATIONS[2],
        PROJECT_MIGRATIONS[3],
        PROJECT_MIGRATIONS[4],
        PROJECT_MIGRATIONS[5],
    ];

    let dir = temp_dir("seven-to-eight");
    let db = dir.join("project.db");

    let old = Store::open(StoreSpec {
        migrations: &STEPS_TO_SEVEN,
        ..StoreSpec::project(db.clone())
    })
    .expect("dung fixture o phien ban 7");
    assert_eq!(
        old.schema_version(),
        7,
        "fixture phai dung o dung phien ban 7 -- neu khong ca nay khong kiem gi ca"
    );

    // Ba hang mang BA trang thai khac nhau — de ca nay cung canh duoc menh de cua AC2:
    // co cat bo la mot TRUC DOC LAP, no khong duoc dung toi `status` cua ai.
    old.write(|tx: &Transaction<'_>| {
        for (ord, status) in [(1i64, "draft"), (2, "confirmed"), (3, "draft")] {
            tx.execute(
                "INSERT INTO segment (chapter_id, ord, source_text, target_text, status, \
                 is_paragraph_end, created_at, updated_at) \
                 VALUES (1, ?1, ?2, ?3, ?4, 0, '2026-08-15T00:00:00.000Z', \
                 '2026-08-15T00:00:00.000Z')",
                (
                    ord,
                    format!("cau {ord}"),
                    format!("ban dich {ord}"),
                    status,
                ),
            )?;
        }
        Ok(())
    })
    .expect("bom ba hang segment vao fixture");
    drop(old);

    let migrated = Store::open(StoreSpec::project(db))
        .expect("mot `project.db` o phien ban 7 phai mo duoc va di tru len dich");
    // 🔵 CAP NHAT 2026-08-16 (Story 2.6): dich 9 → 10 — buoc 10 ra doi.
    // 🔵 CAP NHAT 2026-08-19 (Story 3.1): dich 11 → 12 — buoc 12 ra doi.
    // 🔵 CAP NHAT 2026-08-20 (Story 3.2): dich 12 → 13 — buoc 13 ra doi.
    // 🔵 CAP NHAT 2026-08-22 (Story 3.5): dich 13 → 14 — buoc 14 ra doi.
    // 🔵 CAP NHAT 2026-08-24 (Story 3.10): dich 14 → 15 — buoc 15 ra doi.
    // 🔵 CAP NHAT 2026-08-27 (Story 5.4): dich 15 → 16 — buoc 16 (work.status_override,
    // FR6) ra doi. Menh de van khong doi.
    assert_eq!(
        migrated.schema_version(),
        18,
        "buoc 8..18 phai chay tren mot tep dung o phien ban 7"
    );

    let rows: Vec<(i64, String, String, String, i64)> = migrated
        .read(|conn| {
            let mut stmt = conn.prepare(
                "SELECT ord, source_text, target_text, status, is_omitted FROM segment \
                 ORDER BY ord",
            )?;
            let mapped = stmt.query_map([], |r| {
                Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?, r.get(4)?))
            })?;
            mapped.collect::<Result<Vec<_>, _>>()
        })
        .expect("doc lai ba hang segment sau di tru");

    assert_eq!(
        rows,
        vec![
            (
                1,
                "cau 1".to_owned(),
                "ban dich 1".to_owned(),
                "draft".to_owned(),
                0
            ),
            (
                2,
                "cau 2".to_owned(),
                "ban dich 2".to_owned(),
                "confirmed".to_owned(),
                0
            ),
            (
                3,
                "cau 3".to_owned(),
                "ban dich 3".to_owned(),
                "draft".to_owned(),
                0
            ),
        ],
        "buoc 8 phai la mot `ALTER TABLE` -- moi hang cu o lai nguyen ven, `status` cua \
         chung KHONG doi (AC2: cat bo la mot truc doc lap), va `is_omitted` cua chung la 0"
    );

    drop(migrated);
    cleanup(&dir);
}

/// **Cột `is_target_paragraph_end` có mặt trên một `project.db` mới, và có ĐÚNG hình dạng
/// đã ký** — Story 2.5d, AC5 · Quyết định #5 đường (c) (Ice ký 2026-08-15).
///
/// ⚠️ Số phiên bản là một **PROXY**; mệnh đề thật là phép đọc `pragma_table_info` ngay
/// dưới. Cùng luật mà ca của bước 8 đã đặt: một bước khai đúng số 9 mà chạy sai DDL đi lọt
/// mọi ca đếm phiên bản.
///
/// 🔴 Ba vế của Quyết định #5(c), mỗi vế hỏng theo một kiểu khác: `NOT NULL` *(một segment
/// không có cờ đích là một trạng thái thứ ba không ai khai)* · `DEFAULT 0` *(thiếu nó thì
/// `ADD COLUMN NOT NULL` không chạy nổi trên bảng đã có dữ liệu)* · **không `CHECK`** *(một
/// `CHECK` ở đây dựng quy ước thứ hai cho cùng một việc — `status`, `is_omitted` và
/// `chapter.status` đều cưỡng chế ở tầng Rust)*.
///
/// ⚠️ **Vế `DEFAULT 0` KHÔNG mâu thuẫn với AC2.** `DEFAULT` của SQLite phải là **hằng**,
/// nên nó không diễn đạt được *"bằng cờ nguồn"* — thứ đóng AC2 là câu `UPDATE` chạy **cùng
/// giao dịch**, và ca ngay dưới (`..._backfills_the_target_flag_from_the_source_flag_...`)
/// là chỗ canh nó. `DEFAULT 0` chỉ phục vụ **hàng chèn mới sau này**, và mọi đường chèn
/// đều set cờ tường minh.
#[test]
fn a_fresh_project_database_carries_an_is_target_paragraph_end_column_with_the_shape_ice_signed() {
    let root = temp_dir("fresh-target-para-end");
    let opened = create_work_from_text(&root, "Ngat Doan", "zh", "", "一。二。".to_owned())
        .expect("tao tac pham that bai");

    let (col_type, notnull, default_value): (String, i64, String) = opened
        .store
        .read(|conn| {
            conn.query_row(
                "SELECT type, \"notnull\", COALESCE(dflt_value, '<NULL>') \
                 FROM pragma_table_info('segment') WHERE name = 'is_target_paragraph_end'",
                [],
                |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
            )
        })
        .expect("cot `is_target_paragraph_end` phai co mat trong `segment`");

    assert_eq!(
        col_type, "INTEGER",
        "`is_target_paragraph_end` phai la INTEGER -- khuon `is_paragraph_end` va \
         `is_omitted`. SQLite khong co kieu boolean"
    );
    assert_eq!(
        notnull, 1,
        "`is_target_paragraph_end` phai `NOT NULL` -- mot segment KHONG co co dich la mot \
         trang thai thu ba khong ai khai"
    );
    assert_eq!(
        default_value, "0",
        "`is_target_paragraph_end` phai mac dinh 0 -- SQLite doi mot DEFAULT khac NULL cho \
         moi `ADD COLUMN NOT NULL` tren bang da co du lieu, VA no PHAI la mot hang: \
         `DEFAULT is_paragraph_end` khong ton tai. Ve \"bang co nguon\" cua AC2 do cau \
         `UPDATE` cung giao dich dong"
    );

    let ddl: String = opened
        .store
        .read(|conn| {
            conn.query_row(
                "SELECT sql FROM sqlite_master WHERE type = 'table' AND name = 'segment'",
                [],
                |r| r.get(0),
            )
        })
        .expect("doc DDL cua bang `segment`");
    assert!(
        !ddl.to_uppercase().contains("CHECK"),
        "bang `segment` mang mot rang buoc `CHECK` -- gia tri hop le cuong che o tang Rust, \
         dung khuon `status`, `is_omitted` va `chapter.status`. DDL: {ddl}"
    );

    let dir = opened.dir.clone();
    drop(opened);
    cleanup(&dir);
    cleanup(&root);
}

/// 🔴 **Bước 9 BACKFILL cờ đích BẰNG CỜ NGUỒN — TỪNG HÀNG, không phải "mọi hàng nhận 0"**
/// — Story 2.5d, AC2 · AC5 · Task 2.9.
///
/// ⚠️ **Ca này KHÁC HẲN năm ca backfill trước của kho, và khác ở chỗ dễ đọc lướt qua.**
/// Bước 6, 7 và 8 backfill một **hằng** *(`''`, `'draft'`, `0`)*, nên một ca kiểm chúng chỉ
/// cần khẳng định *"mọi hàng nhận giá trị X"*. Bước 9 backfill một giá trị **theo hàng**,
/// nên một ca viết theo khuôn cũ — *"mọi hàng nhận 0"* — sẽ **XANH trên một bước di trú
/// chạy sai**, vì nó xanh với đúng cái `DEFAULT 0` mà `ALTER TABLE` để lại.
/// ⇒ Fixture ở đây bắt buộc phải có **cả hai** loại hàng, và phép khẳng định phải so **cặp**
/// `(is_paragraph_end, is_target_paragraph_end)` từng hàng một.
///
/// 🔴 Vì sao nó là một quyết định nghiệp vụ chứ không một chi tiết kỹ thuật: **21**
/// `project.db` thật và **10.477** hàng `segment` *(đo 2026-08-12)* chạy bước này. Một
/// `DEFAULT 0` trần nói *"không đoạn nào của bản dịch kết thúc"* — **sai** với mọi Chương đã
/// nhập, và sai theo kiểu không biểu hiện thành lỗi nào: nó biểu hiện thành **bản dịch xuất
/// ra mất hết ngắt đoạn**, ở Epic 8, nhiều tháng sau.
///
/// ⚠️ Fixture dựng từ các bước **THẬT** của [`PROJECT_MIGRATIONS`], không chép tay DDL —
/// một DDL chép tay là một lược đồ thứ hai, và nó sẽ trôi khỏi lược đồ thật trong im lặng.
#[test]
fn a_project_database_at_version_eight_backfills_the_target_flag_from_the_source_flag_row_by_row() {
    static STEPS_TO_EIGHT: [Migration; 7] = [
        PROJECT_MIGRATIONS[0],
        PROJECT_MIGRATIONS[1],
        PROJECT_MIGRATIONS[2],
        PROJECT_MIGRATIONS[3],
        PROJECT_MIGRATIONS[4],
        PROJECT_MIGRATIONS[5],
        PROJECT_MIGRATIONS[6],
    ];

    let dir = temp_dir("eight-to-nine");
    let db = dir.join("project.db");

    let old = Store::open(StoreSpec {
        migrations: &STEPS_TO_EIGHT,
        ..StoreSpec::project(db.clone())
    })
    .expect("dung fixture o phien ban 8");
    assert_eq!(
        old.schema_version(),
        8,
        "fixture phai dung o dung phien ban 8 -- neu khong ca nay khong kiem gi ca"
    );

    // 🔴 CA BON HANG, va bon hang la so TOI THIEU de menh de nay phan biet duoc:
    // hai hang co nguon TAT va hai hang co nguon BAT. Mot fixture chi co hang tat se xanh
    // voi ca mot buoc di tru khong chay cau `UPDATE` nao.
    old.write(|tx: &Transaction<'_>| {
        for (ord, is_para_end) in [(1i64, 0i64), (2, 1), (3, 0), (4, 1)] {
            tx.execute(
                "INSERT INTO segment (chapter_id, ord, source_text, target_text, status, \
                 is_paragraph_end, is_omitted, created_at, updated_at) \
                 VALUES (1, ?1, ?2, ?3, 'draft', ?4, 0, '2026-08-16T00:00:00.000Z', \
                 '2026-08-16T00:00:00.000Z')",
                (
                    ord,
                    format!("cau {ord}"),
                    format!("ban dich {ord}"),
                    is_para_end,
                ),
            )?;
        }
        Ok(())
    })
    .expect("bom bon hang segment vao fixture");
    drop(old);

    let migrated = Store::open(StoreSpec::project(db))
        .expect("mot `project.db` o phien ban 8 phai mo duoc va di tru len dich");
    // 🔵 CAP NHAT 2026-08-16 (Story 2.6): dich 9 → 10. Chu de cua ca nay khong doi — no do
    // menh de "buoc 9 backfill co dich BANG CO NGUON, tung hang"; buoc 10 chay them mot
    // luot va no KHONG dung toi mot hang nao (no chi dung mot index).
    // 🔵 CAP NHAT 2026-08-19 (Story 3.1): dich 11 → 12 — buoc 12 ra doi.
    // 🔵 CAP NHAT 2026-08-20 (Story 3.2): dich 12 → 13 — buoc 13 ra doi.
    // 🔵 CAP NHAT 2026-08-22 (Story 3.5): dich 13 → 14 — buoc 14 ra doi.
    // 🔵 CAP NHAT 2026-08-24 (Story 3.10): dich 14 → 15 — buoc 15 ra doi.
    // 🔵 CAP NHAT 2026-08-27 (Story 5.4): dich 15 → 16 — buoc 16 (work.status_override,
    // FR6) ra doi. Menh de van khong doi.
    assert_eq!(
        migrated.schema_version(),
        18,
        "buoc 9..18 phai chay tren mot tep dung o phien ban 8"
    );

    let rows: Vec<(i64, i64, i64)> = migrated
        .read(|conn| {
            let mut stmt = conn.prepare(
                "SELECT ord, is_paragraph_end, is_target_paragraph_end FROM segment \
                 ORDER BY ord",
            )?;
            let mapped = stmt.query_map([], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)))?;
            mapped.collect::<Result<Vec<_>, _>>()
        })
        .expect("doc lai bon hang segment sau di tru");

    assert_eq!(
        rows,
        vec![(1, 0, 0), (2, 1, 1), (3, 0, 0), (4, 1, 1)],
        "buoc 9 phai backfill `is_target_paragraph_end` = `is_paragraph_end` TUNG HANG \
         (AC2: ban dich soi guong ban goc cho toi khi nguoi dung doi). Neu ket qua doc ra \
         la (1,0,0) (2,1,0) (3,0,0) (4,1,0) thi cau `UPDATE` KHONG chay -- chi con \
         `DEFAULT 0` cua `ALTER TABLE`"
    );

    drop(migrated);
    cleanup(&dir);
}

/// 🔴 **AD-30 — một `project.db` MỚI HƠN ứng dụng bị TỪ CHỐI MỞ, không bao giờ ghi vào.**
///
/// Ca này canh đúng nửa mà bước 7 làm dịch chuyển: trước lượt này target là 6, nên một tệp
/// ở 7 bị từ chối; sau lượt này một tệp ở **8** phải bị từ chối đúng như vậy. Không có ca
/// này thì việc nâng target là một lượt đổi hành vi **không ai đo**.
///
/// 🔵 **CẬP NHẬT 2026-08-15 (Story 2.5c, Task 1.5) — fixture nâng từ 8 lên 9.**
/// 🔴 Đây **không** phải một lượt nới cho hết đỏ; nếu để nguyên, ca này **vẫn xanh** mà
/// **mất hết ý nghĩa**: số 8 nay là một bước **THẬT** của [`PROJECT_MIGRATIONS`], nên một
/// fixture dừng ở 8 không còn mô phỏng *"một bản ứng dụng tương lai"* — nó mô phỏng đúng
/// bản ứng dụng hôm nay, và phép từ chối được khẳng định ở dưới sẽ **không bao giờ chạy**
/// vào nhánh AD-30. Số của fixture phải luôn là `target + 1`.
///
/// 🔵 **CẬP NHẬT 2026-08-16 (Story 2.5d, Task 2.6) — fixture nâng từ 9 lên 10**, và đây là
/// **lượt lặp lại thứ hai** của luật ngay trên. Bước 9 (`is_target_paragraph_end`) nay là
/// một bước THẬT, nên một fixture dừng ở 9 mô phỏng đúng bản hôm nay và ca này sẽ **chết
/// lâm sàng: xanh mà không bao giờ chạm nhánh AD-30**.
/// ⚠️ Tên hằng cũng đổi theo (`STEP_NINE` → `STEP_TEN`) — cùng lý do với tên hàm ở
/// `..._matches_the_declared_ladder_step_for_step`: một cái tên mang số là một câu khẳng
/// định sai lại ở mỗi story thêm một bước.
///
/// 🔵 **CẬP NHẬT 2026-08-16 (Story 2.6, Task 1.5) — fixture nâng từ 10 lên 11**, và đây là
/// **lượt lặp lại thứ BA** của cùng một luật. Bước 10 (`idx_segment_version_segment_created`)
/// nay là một bước THẬT, nên một fixture dừng ở 10 mô phỏng đúng bản hôm nay ⇒ ca này chết
/// lâm sàng: **xanh mà không bao giờ chạm nhánh AD-30**.
/// 🔴 Ba lượt lặp lại là đủ để gọi tên khuôn: **cái tên `STEP_*` và số của nó là hai thứ
/// KHÔNG cổng nào canh**, và cả hai sai lại ở *mỗi* story thêm một bước. Sửa được vì có
/// người đọc doc-comment này, không vì có gì đỏ. ⇒ Nếu một story sau thêm bước 11 thật, đây
/// là chỗ phải sửa **trước** khi tin ca này còn nói gì.
///
/// 🔵 **CẬP NHẬT 2026-08-16 (Story 2.7) — fixture nâng từ 11 lên 12**, và đây là **lượt lặp
/// lại thứ TƯ**. Đúng như dòng ngay trên đã dặn: bước 11
/// (`SEGMENT_TRANSLATION_ORIGIN_DDL`, FR117/AD-47) nay là một bước **thật**, nên một fixture
/// dừng ở 11 mô phỏng đúng bản hôm nay ⇒ ca này **xanh mà không bao giờ chạm nhánh AD-30**.
/// ⚠️ Ba thứ phải đổi **cùng lượt** và không cái nào có cổng canh: **tên hằng** · kích thước
/// mảng `[Migration; N]` · số giả trong `Migration`. Kích thước mảng là thứ duy nhất trong ba
/// cái báo được — bằng một **lỗi biên dịch `E0080`**, không một ca đỏ.
///
/// 🔵 **CẬP NHẬT 2026-08-19 (Story 3.1) — fixture nâng từ 12 lên 13**, và đây là **lượt lặp
/// lại thứ NĂM**. Bước 12 (`GLOSSARY_ENTRY_DDL`, AD-18/AD-36) nay là một bước **thật**, nên
/// một fixture dừng ở 12 mô phỏng đúng bản hôm nay ⇒ ca này sẽ **xanh mà không bao giờ chạm
/// nhánh AD-30**.
///
/// 🔵 **CẬP NHẬT 2026-08-20 (Story 3.2) — fixture nâng từ 13 lên 14**, và đây là **lượt lặp
/// lại thứ SÁU**. Bước 13 (`GLOSSARY_CANDIDATE_DDL`, AD-20/AD-36) nay là một bước **thật**,
/// nên một fixture dừng ở 13 mô phỏng đúng bản hôm nay ⇒ ca này sẽ **xanh mà không bao giờ
/// chạm nhánh AD-30**. `STEP_THIRTEEN` → `STEP_FOURTEEN` (cùng luật đặt tên đã dặn ở trên);
/// mảng lên `[Migration; 13]`; bước giả lên `to_version: 14`.
///
/// 🔵 **CẬP NHẬT 2026-08-22 (Story 3.5) — fixture nâng từ 14 lên 15**, và đây là **lượt lặp
/// lại thứ BẢY**. Bước 14 (`GLOSSARY_CANDIDATE_OCCURRENCE_CONTEXT_DDL`) nay là một bước
/// **thật**, nên một fixture dừng ở 14 mô phỏng đúng bản hôm nay ⇒ ca này sẽ **xanh mà
/// không bao giờ chạm nhánh AD-30**. `STEP_FOURTEEN` → `STEP_FIFTEEN`; mảng lên
/// `[Migration; 14]`; bước giả lên `to_version: 15`.
///
/// 🔵 **CẬP NHẬT 2026-08-24 (Story 3.10) — fixture nâng từ 15 lên 16**, và đây là **lượt lặp
/// lại thứ TÁM**. Bước 15 (`GLOSSARY_ENTRY_ADD_FILE_IMPORT_ORIGIN_DDL`) nay là một bước
/// **thật**, nên một fixture dừng ở 15 mô phỏng đúng bản hôm nay ⇒ ca này sẽ **xanh mà
/// không bao giờ chạm nhánh AD-30**. `STEP_FIFTEEN` → `STEP_SIXTEEN`; mảng lên
/// `[Migration; 15]`; bước giả lên `to_version: 16`.
///
/// 🔵 **CẬP NHẬT 2026-08-27 (Story 5.4) — fixture nâng từ 16 lên 17**, và đây là **lượt lặp
/// lại thứ CHÍN**. Bước 16 (`WORK_STATUS_OVERRIDE_DDL`, FR6) nay là một bước **thật**, nên
/// một fixture dừng ở 16 mô phỏng đúng bản hôm nay ⇒ ca này sẽ **xanh mà không bao giờ chạm
/// nhánh AD-30**. `STEP_SIXTEEN` → `STEP_SEVENTEEN`; mảng lên `[Migration; 16]`; bước giả
/// lên `to_version: 17`.
///
/// 🔵 **CẬP NHẬT 2026-08-29 (Story 5.7) — fixture nâng từ 17 lên 18**, và đây là **lượt lặp
/// lại thứ MƯỜI**. Bước 17 (`CHAPTER_POSITION_DDL`, AD-3) nay là một bước **thật**, nên một
/// fixture dừng ở 17 mô phỏng đúng bản hôm nay ⇒ ca này sẽ **xanh mà không bao giờ chạm
/// nhánh AD-30**. `STEP_SEVENTEEN` → `STEP_EIGHTEEN`; mảng lên `[Migration; 17]`; bước giả
/// lên `to_version: 18`.
///
/// 🔵 **CẬP NHẬT 2026-08-31 (Story 5.13) — fixture nâng từ 18 lên 19.** Bước 18
/// (`READING_MARK_DDL`) nay là bước thật; một fixture dừng ở 18 không còn mới hơn app.
#[test]
fn a_project_database_newer_than_the_app_is_refused_and_never_written_to() {
    static STEP_NINETEEN: [Migration; 18] = [
        PROJECT_MIGRATIONS[0],
        PROJECT_MIGRATIONS[1],
        PROJECT_MIGRATIONS[2],
        PROJECT_MIGRATIONS[3],
        PROJECT_MIGRATIONS[4],
        PROJECT_MIGRATIONS[5],
        PROJECT_MIGRATIONS[6],
        PROJECT_MIGRATIONS[7],
        PROJECT_MIGRATIONS[8],
        PROJECT_MIGRATIONS[9],
        PROJECT_MIGRATIONS[10],
        PROJECT_MIGRATIONS[11],
        PROJECT_MIGRATIONS[12],
        PROJECT_MIGRATIONS[13],
        PROJECT_MIGRATIONS[14],
        PROJECT_MIGRATIONS[15],
        PROJECT_MIGRATIONS[16],
        // Mot buoc 19 GIA — day la "mot ban ung dung tuong lai" nhin tu hom nay.
        Migration {
            to_version: 19,
            sql: "CREATE TABLE tu_tuong_lai (id INTEGER PRIMARY KEY);",
        },
    ];

    let dir = temp_dir("newer-refused");
    let db = dir.join("project.db");

    let future = Store::open(StoreSpec {
        migrations: &STEP_NINETEEN,
        ..StoreSpec::project(db.clone())
    })
    .expect("dung fixture o phien ban 19");
    assert_eq!(future.schema_version(), 19);
    drop(future);

    let before = fs::metadata(&db).expect("doc metadata truoc").len();

    let refused = Store::open(StoreSpec::project(db.clone()));
    let err = refused.err().expect(
        "mot `project.db` o phien ban 19 PHAI bi tu choi mo -- AD-30 noi \"khong bao gio ghi vao\"",
    );
    let ipc: auratranslate_lib::core::i18n::IpcError = err.into();
    assert_eq!(
        ipc.message_key(),
        MessageKey::StoreSchemaTooNew,
        "phep tu choi phai phan biet duoc, khong phai mot loi mo kho chung chung"
    );

    assert_eq!(
        fs::metadata(&db).expect("doc metadata sau").len(),
        before,
        "mot lan tu choi mo KHONG duoc dung toi mot byte nao cua tep"
    );

    cleanup(&dir);
}

// ═════════════════════════════════════════════════════════════════════════════
// AC5 · AC9 — `AUTOINCREMENT` là cơ chế duy nhất, và nó nằm trong DDL
// ═════════════════════════════════════════════════════════════════════════════

/// Bản sao đúng khuôn `project_contract.rs::a_retired_chapter_id_is_never_handed_out_again`.
/// `INTEGER PRIMARY KEY` trần tái dùng rowid lớn nhất vừa xoá; `AUTOINCREMENT` thì không.
#[test]
fn a_retired_segment_id_is_never_handed_out_again() {
    let root = temp_dir("retired-segment-id");

    let opened = create_work_from_text(&root, "Ba Cau", "zh", "", "一。二。三。".to_owned())
        .expect("tao tac pham that bai");
    let store = opened.store;

    let ids: Vec<i64> = store
        .read(|conn| {
            let mut stmt = conn.prepare("SELECT id FROM segment ORDER BY ord")?;
            let rows = stmt.query_map([], |row| row.get::<_, i64>(0))?;
            rows.collect::<Result<Vec<i64>, _>>()
        })
        .expect("doc id segment that bai");
    assert_eq!(ids, vec![1, 2, 3], "ba cau phai cho ba segment id 1..3");

    // Xoa segment CUOI roi chen mot segment moi. Voi `INTEGER PRIMARY KEY` tran, SQLite
    // phat lai dung id vua mat (3). Voi AUTOINCREMENT, id moi phai la 4.
    store
        .write(move |tx: &Transaction<'_>| {
            tx.execute("DELETE FROM segment WHERE ord = 3", [])?;
            tx.execute(
                "INSERT INTO segment (chapter_id, ord, source_text, is_paragraph_end, \
                 created_at, updated_at) VALUES (1, 3, 'bon.', 0, \
                 strftime('%Y-%m-%dT%H:%M:%fZ','now'), strftime('%Y-%m-%dT%H:%M:%fZ','now'))",
                [],
            )?;
            Ok(())
        })
        .expect("job ghi that bai");

    let new_id: i64 = store
        .read(|conn| conn.query_row("SELECT id FROM segment WHERE ord = 3", [], |row| row.get(0)))
        .expect("doc id segment moi that bai");

    assert_ne!(new_id, 3, "id da ve huu (3) khong duoc phat lai -- AC5");
    assert_eq!(new_id, 4, "AUTOINCREMENT phai cap id tang dan nghiem ngat");

    drop(store);
    cleanup(&root);
}

// ═════════════════════════════════════════════════════════════════════════════
// AC3 · AC4 · AC13 — đường nhập ghi segment cùng giao dịch với `chapter`
// ═════════════════════════════════════════════════════════════════════════════

#[test]
fn creating_a_work_lays_down_its_segments_in_the_same_go() {
    let root = temp_dir("create-lays-segments");

    let opened =
        create_work_from_text(&root, "Bon Cau", "zh", "", "一。二。\n三。四。".to_owned())
            .expect("tao tac pham that bai");
    let store = opened.store;

    let rows: Vec<(i64, i64, String, i64)> = store
        .read(|conn| {
            let mut stmt = conn.prepare(
                "SELECT chapter_id, ord, source_text, is_paragraph_end FROM segment ORDER BY ord",
            )?;
            let rows = stmt.query_map([], |row| {
                Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?))
            })?;
            rows.collect::<Result<Vec<_>, _>>()
        })
        .expect("doc segment that bai");

    assert_eq!(rows.len(), 4, "bon cau phai cho bon segment");

    // AC4 — `ord` danh so tu 1, lien tuc, khong lo. Story 2.10 dung tren gia dinh nay.
    let ords: Vec<i64> = rows.iter().map(|r| r.1).collect();
    assert_eq!(ords, vec![1, 2, 3, 4], "`ord` phai danh so tu 1 va lien tuc");

    // Moi segment tro ve dung hang `chapter` vua sinh ra chung.
    let chapter_id: i64 = store
        .read(|conn| conn.query_row("SELECT id FROM chapter ORDER BY ord LIMIT 1", [], |r| r.get(0)))
        .expect("doc chapter_id that bai");
    assert!(
        rows.iter().all(|r| r.0 == chapter_id),
        "moi segment phai tro ve hang `chapter` sinh ra no"
    );

    assert_eq!(
        rows.iter().map(|r| r.2.as_str()).collect::<Vec<_>>(),
        vec!["一。", "二。", "三。", "四。"]
    );
    // AC6 — co ket doan luu XUONG DIA, khong suy ra luc nap.
    assert_eq!(
        rows.iter().map(|r| r.3).collect::<Vec<_>>(),
        vec![0, 1, 0, 0],
        "co ket doan phai la 0/1 tren dia, va segment cuoi luon 0 (AC7)"
    );

    drop(store);
    cleanup(&root);
}

/// AC13 — một lỗi giữa chừng ⇒ **không** hàng `chapter` nào và **không** hàng `segment`
/// nào còn lại. Kiểm bằng chính cơ chế mà hợp đồng dựa vào: một job `Store::write` chèn
/// `chapter` rồi trượt ở lượt chèn `segment`.
#[test]
fn a_failure_midway_leaves_neither_a_chapter_nor_a_segment() {
    let root = temp_dir("midway-failure");

    let opened = create_work_from_text(&root, "Nua Voi", "zh", "", "一。".to_owned())
        .expect("tao tac pham that bai");
    let store = opened.store;

    let before: (i64, i64) = store
        .read(|conn| {
            Ok((
                conn.query_row("SELECT COUNT(*) FROM chapter", [], |r| r.get(0))?,
                conn.query_row("SELECT COUNT(*) FROM segment", [], |r| r.get(0))?,
            ))
        })
        .expect("dem truoc that bai");
    assert_eq!(before, (1, 1));

    // Chen mot Chuong thu hai roi truot o segment cua no (`source_text` NOT NULL).
    let outcome = store.write(move |tx: &Transaction<'_>| {
        tx.execute(
            "INSERT INTO chapter (ord, title, source_text, status, created_at, updated_at) \
             VALUES (2, NULL, 'hai.', 'not_started', \
             strftime('%Y-%m-%dT%H:%M:%fZ','now'), strftime('%Y-%m-%dT%H:%M:%fZ','now'))",
            [],
        )?;
        tx.execute(
            "INSERT INTO segment (chapter_id, ord, source_text, is_paragraph_end, \
             created_at, updated_at) VALUES (2, 1, NULL, 0, \
             strftime('%Y-%m-%dT%H:%M:%fZ','now'), strftime('%Y-%m-%dT%H:%M:%fZ','now'))",
            [],
        )?;
        Ok(())
    });
    assert!(outcome.is_err(), "job phai truot -- `source_text` la NOT NULL");

    let after: (i64, i64) = store
        .read(|conn| {
            Ok((
                conn.query_row("SELECT COUNT(*) FROM chapter", [], |r| r.get(0))?,
                conn.query_row("SELECT COUNT(*) FROM segment", [], |r| r.get(0))?,
            ))
        })
        .expect("dem sau that bai");
    assert_eq!(
        after, before,
        "giao dich phai rollback tron ven -- khong nua Chuong nao o lai (AC13)"
    );

    drop(store);
    cleanup(&root);
}

/// Một Chương **0 segment** (văn bản chỉ khoảng trắng) không được làm đường nhập hỏng —
/// ca ① của Task 1 nói bộ tách trả 0 segment, và Task 4 phải chịu được điều đó.
#[test]
fn a_chapter_with_no_segments_still_creates_a_work() {
    let root = temp_dir("zero-segments");

    let opened = create_work_from_text(&root, "Trang", "zh", "", "   \n  ".to_owned())
        .expect("tao tac pham voi van ban trang phai thanh cong");
    let store = opened.store;

    let count: i64 = store
        .read(|conn| conn.query_row("SELECT COUNT(*) FROM segment", [], |r| r.get(0)))
        .expect("dem segment that bai");
    assert_eq!(count, 0);

    let chapters: i64 = store
        .read(|conn| conn.query_row("SELECT COUNT(*) FROM chapter", [], |r| r.get(0)))
        .expect("dem chapter that bai");
    assert_eq!(chapters, 1, "hang `chapter` van phai ton tai");

    drop(store);
    cleanup(&root);
}

// ═════════════════════════════════════════════════════════════════════════════
// AC8 vế một + AC14 — lệnh tách tường minh
// ═════════════════════════════════════════════════════════════════════════════

#[test]
fn splitting_without_an_open_work_is_refused() {
    let err = split_chapter_into_segments(None, 1).expect_err("phai tu choi khi chua mo Tac pham");
    assert_eq!(err.code(), "work.none_open");
    assert_eq!(err.message_key(), MessageKey::WorkNoneOpen);
}

#[test]
fn splitting_an_unknown_chapter_is_refused() {
    let root = temp_dir("unknown-chapter");

    let opened = create_work_from_text(&root, "Khong Co", "zh", "", "一。".to_owned())
        .expect("tao tac pham that bai");

    let err = split_chapter_into_segments(Some(&opened), 999)
        .expect_err("phai tu choi mot chapter_id khong ton tai");
    assert_eq!(err.code(), "segment.chapter_not_found");
    assert_eq!(err.message_key(), MessageKey::SegmentChapterNotFound);
    assert_eq!(err.params().get("chapter_id").map(String::as_str), Some("999"));

    drop(opened);
    cleanup(&root);
}

/// Lệnh **từ chối** một Chương đã có segment — không ghi đè im lặng (Quyết định #4).
#[test]
fn splitting_a_chapter_that_already_has_segments_is_refused() {
    let root = temp_dir("already-split");

    let opened = create_work_from_text(&root, "Da Tach", "zh", "", "一。二。".to_owned())
        .expect("tao tac pham that bai");

    let err = split_chapter_into_segments(Some(&opened), 1)
        .expect_err("phai tu choi mot Chuong da co segment");
    assert_eq!(err.code(), "segment.already_split");
    assert_eq!(err.message_key(), MessageKey::SegmentAlreadySplit);
    assert_eq!(err.params().get("count").map(String::as_str), Some("2"));

    // Va khong ghi de: hai segment cu van nguyen ven.
    let count: i64 = opened
        .store
        .read(|conn| conn.query_row("SELECT COUNT(*) FROM segment", [], |r| r.get(0)))
        .expect("dem segment that bai");
    assert_eq!(count, 2, "lenh bi tu choi khong duoc cham mot hang nao");

    drop(opened);
    cleanup(&root);
}

/// Đường mà Quyết định #4 tồn tại để phục vụ: một Chương Epic 1 với `segment_count = 0`.
#[test]
fn an_explicit_split_lays_down_segments_for_an_old_chapter() {
    let root = temp_dir("explicit-split");

    let opened = create_work_from_text(&root, "Cu", "zh", "", "一。二。\n三。".to_owned())
        .expect("tao tac pham that bai");

    // Dung lai trang thai Epic 1: mot Chuong co that, KHONG segment nao.
    opened
        .store
        .write(|tx: &Transaction<'_>| {
            tx.execute("DELETE FROM segment", [])?;
            Ok(())
        })
        .expect("don segment that bai");

    let outcome: SplitOutcome =
        split_chapter_into_segments(Some(&opened), 1).expect("tach tuong minh phai thanh cong");
    assert_eq!(outcome.chapter_id, 1);
    assert_eq!(outcome.segment_count, 3);

    let rows: Vec<(i64, String, i64)> = opened
        .store
        .read(|conn| {
            let mut stmt =
                conn.prepare("SELECT ord, source_text, is_paragraph_end FROM segment ORDER BY ord")?;
            let rows = stmt.query_map([], |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)))?;
            rows.collect::<Result<Vec<_>, _>>()
        })
        .expect("doc segment that bai");

    assert_eq!(rows.iter().map(|r| r.0).collect::<Vec<_>>(), vec![1, 2, 3]);
    assert_eq!(
        rows.iter().map(|r| r.1.as_str()).collect::<Vec<_>>(),
        vec!["一。", "二。", "三。"]
    );
    assert_eq!(rows.iter().map(|r| r.2).collect::<Vec<_>>(), vec![0, 1, 0]);

    // 🔴 AC5 — id KHONG bat dau lai tu 1 sau khi ba id dau da ve huu.
    let ids: Vec<i64> = opened
        .store
        .read(|conn| {
            let mut stmt = conn.prepare("SELECT id FROM segment ORDER BY ord")?;
            let rows = stmt.query_map([], |row| row.get::<_, i64>(0))?;
            rows.collect::<Result<Vec<i64>, _>>()
        })
        .expect("doc id that bai");
    assert_eq!(ids, vec![4, 5, 6], "id da ve huu (1..3) khong duoc phat lai");

    drop(opened);
    cleanup(&root);
}

// ═════════════════════════════════════════════════════════════════════════════
// Story 2.2 · AC13 — lệnh nạp segment của Chương đang mở
// ═════════════════════════════════════════════════════════════════════════════

#[test]
fn loading_segments_without_an_open_work_is_refused() {
    let err = read_open_chapter_segments(None).expect_err("phai tu choi khi chua mo Tac pham");
    assert_eq!(err.code(), "work.none_open");
    assert_eq!(err.message_key(), MessageKey::WorkNoneOpen);
}

/// Lượt nạp trả **đúng** các hàng đã ghi, **theo `ord`**, kèm cờ kết đoạn đã lưu.
///
/// 🔴 Ca này canh AD-37 ở đúng chỗ nó dễ trôi nhất: `is_paragraph_end` đi qua dây **từ dữ
/// liệu đã lưu**, không phải một phép suy lúc render. Đổi lệnh thành tự đoán đoạn từ nội
/// dung sẽ làm hàng thứ hai *(cau ket doan giua Chuong)* sai ngay.
#[test]
fn the_open_chapter_hands_back_every_segment_in_reading_order() {
    let root = temp_dir("load-in-order");

    // Ba cau, dau xuong dong sau cau thu hai ⇒ co ket doan bat o dung hang do.
    let opened = create_work_from_text(&root, "Nap Segment", "zh", "", "一。二。\n三。".to_owned())
        .expect("tao tac pham that bai");

    let loaded = read_open_chapter_segments(Some(&opened)).expect("nap segment that bai");

    assert_eq!(
        loaded.segments.iter().map(|s| s.ord).collect::<Vec<_>>(),
        vec![1, 2, 3],
        "`ORDER BY ord` -- thu tu doc la hop dong, khong phai thu tu SQLite tra ve tinh co"
    );
    assert_eq!(
        loaded
            .segments
            .iter()
            .map(|s| s.source_text.as_str())
            .collect::<Vec<_>>(),
        vec!["一。", "二。", "三。"]
    );
    assert_eq!(
        loaded
            .segments
            .iter()
            .map(|s| s.is_paragraph_end)
            .collect::<Vec<_>>(),
        vec![false, true, false],
        "co ket doan phai la thu DA LUU (AD-37), khong phai mot phep suy luc nap"
    );
    assert!(
        loaded.segments.iter().all(|s| s.target_text.is_empty()),
        "moi segment vua nhap phai o trang thai \"chua dich\" -- CHUOI RONG"
    );
    assert!(
        loaded.segments.iter().all(|s| s.retired_at.is_none()),
        "hom nay chua duong nao cho segment ve huu -- do la Story 2.8"
    );

    let chapter_id: i64 = opened
        .store
        .read(|conn| conn.query_row("SELECT id FROM chapter ORDER BY ord LIMIT 1", [], |r| r.get(0)))
        .expect("doc chapter_id that bai");
    assert_eq!(
        loaded.chapter_id, chapter_id,
        "`chapter_id` phai di kem -- webview khong duoc doan no, va mot luot hoi lai keo \
         theo nguyen khoi `source_text`"
    );

    drop(opened);
    cleanup(&root);
}

/// 🔴 **Chương KHÔNG có segment nào cho một danh sách RỖNG, không một lỗi.**
///
/// Đây là trạng thái thật của **25 Chương Epic 1** (`deferred-work.md:542`) cho tới khi ai
/// đó bấm lệnh tách tường minh. Trả lỗi ở đây sẽ làm Panel Editor hiện một câu lỗi cho một
/// Tác phẩm hoàn toàn lành lặn; câu đúng là một trạng thái rỗng CÓ GIẢI THÍCH, và nó thuộc
/// tầng giao diện (UX-DR27), không thuộc tầng lỗi IPC (AD-21).
#[test]
fn a_chapter_with_no_segments_loads_as_an_empty_list_not_an_error() {
    let root = temp_dir("load-empty");

    // Mot Chuong chi co khoang trang cho 0 segment -- ca bien ① cua bo tach.
    let opened = create_work_from_text(&root, "Rong", "zh", "", "   \n  ".to_owned())
        .expect("tao tac pham that bai");

    let loaded = read_open_chapter_segments(Some(&opened)).expect("nap segment that bai");
    assert!(
        loaded.segments.is_empty(),
        "khong segment nao ⇒ danh sach rong, khong mot loi"
    );
    assert!(
        loaded.chapter_id > 0,
        "`chapter_id` van phai co that -- Chuong ton tai, chi la chua tach"
    );

    drop(opened);
    cleanup(&root);
}

/// 🔴 **FIXTURE CỦA TASK 7 — `target_text` THẬT, bơm bằng SQL, đọc lại qua ĐÚNG lệnh IPC.**
///
/// Ice chốt Quyết định #1 đường (b) ngày 2026-08-12, kèm **một điều kiện**: vì bề mặt
/// chỉ-đọc làm mọi `target_text` rỗng, AC2/AC4/AC5 không nghiệm thu được trên dữ liệu thật,
/// nên story phải dựng **một fixture có bản dịch thật** và nghiệm thu trên đó. Ca này là nửa
/// **dữ liệu** của điều kiện đó; nửa **thị giác** ở
/// `_bmad-output/implementation-artifacts/2-2-ban-do-editor.html`, và hai bên dùng **cùng
/// năm câu**.
///
/// ⚠️ Bơm bằng `UPDATE` trên một `project.db` **TẠM** dựng trong thư mục tạm của ca này —
/// **không** mở một `.atproj` nào của người dùng bằng app. Cùng kỷ luật Task 8 của Story 2.1
/// đã giữ, và nay nó còn chặt hơn: lược đồ đã lên target **6**, nên một lượt mở là một lượt
/// **di trú thật trên dữ liệu thật**.
///
/// 🔴 Mệnh đề của ca này **không** phải "SQL chạy được" — nó là: bản dịch đi qua **trọn**
/// đường của sản phẩm *(cột → truy vấn → struct → dây)* mà không rơi mất ở khúc nào. Bốn giá
/// trị vạch chỉ có nghĩa nếu `target_text` tới được webview.
#[test]
fn a_chapter_with_real_translations_round_trips_through_the_load_command() {
    let root = temp_dir("fixture-translations");

    // Nam cau nguon — dung nam cau ma ban do thi giac dung.
    let opened = create_work_from_text(
        &root,
        "Fixture 2.2",
        "zh",
        "",
        "一。二。\n三。四。五。".to_owned(),
    )
    .expect("tao tac pham that bai");

    let translations = [
        (1i64, "Hắn đẩy cánh cửa ấy ra, bước vào giữa bóng tối dày đặc của gian phòng đã bỏ hoang từ lâu lắm rồi."),
        (2, "Gió thổi tới từ cuối hành lang, mang theo mùi gỉ sắt và một thứ gì đó ẩm ướt hơn thế."),
        (3, "Thiếu niên không quay đầu lại."),
        // ord 4 CO Y de rong -- no la nhanh *khong vach* (chua dich) cua AC3.
        (5, "Câu này đã về hưu sau một lượt gộp — lịch sử của nó vẫn tra lại được."),
    ];

    opened
        .store
        .write(move |tx: &Transaction<'_>| {
            for (ord, text) in translations {
                tx.execute(
                    "UPDATE segment SET target_text = ?1 WHERE ord = ?2",
                    (text, ord),
                )?;
            }
            // 🔵 2026-08-17 (Story 2.8) — VAI CUA HANG NAY DA DOI, va no doi NGUOC.
            //
            // Luc viet (Story 2.2), cau nay la "nguon du lieu DUY NHAT cua gia tri vach
            // `ornament`": chua duong san pham nao dat duoc `retired_at`, nen bom bang SQL la
            // cach duy nhat nhin thay nhanh do chay.
            //
            // Story 2.8 dung duong do THAT (gop/tach), roi Ice LAT chu ky #6(b) sau mot luot
            // dung that: hang ve huu **khong** duoc hien trong luoi nua. ⇒ Hang nay nay canh
            // menh de NGUOC LAI: duong nap phai LOC no ra.
            //
            // ⚠️ No **o lai** trong fixture chu khong bi xoa, va do la ca gia tri nhat: mot
            // fixture chi co hang con song khong phan biet duoc "loc dung" voi "khong co gi de
            // loc".
            tx.execute(
                "UPDATE segment SET retired_at = '2026-08-12T00:00:00.000Z' WHERE ord = 5",
                [],
            )?;
            Ok(())
        })
        .expect("bom ban dich bang SQL that bai");

    let loaded = read_open_chapter_segments(Some(&opened)).expect("nap segment that bai");
    assert_eq!(
        loaded.segments.len(),
        4,
        "🔵 2026-08-17 (Story 2.8): fixture co NAM hang tren dia, nhung duong nap tra ve BON \
         -- hang thu nam da ve huu. Mot con so 5 o day nghia la `WHERE retired_at IS NULL` \
         da bi go, va trieu chung o nguoi dung la dung cau Ice bao: \"cau cu van ton tai va \
         so thu tu van chiem, gay roi noi dung\""
    );
    // Va hang do van NAM TREN DIA -- loc khoi luoi KHONG phai xoa. AD-5 + AC4 doi lich su
    // phien ban cua no tra lai duoc, ma mot hang bi xoa thi khong con gi de tra.
    let tren_dia: i64 = opened
        .store
        .read(|conn| conn.query_row("SELECT COUNT(*) FROM segment", [], |r| r.get(0)))
        .expect("dem hang tren dia");
    assert_eq!(
        tren_dia, 5,
        "loc khoi LUOI, khong xoa khoi DIA -- xoa la mat lich su VINH VIEN (AD-5)"
    );

    let got: Vec<(&str, bool)> = loaded
        .segments
        .iter()
        .map(|s| (s.target_text.as_str(), s.retired_at.is_some()))
        .collect();
    assert_eq!(
        got,
        vec![
            (translations[0].1, false),
            (translations[1].1, false),
            (translations[2].1, false),
            ("", false),
        ],
        "ban dich phai di qua TRON duong cua san pham -- ke ca cau RONG (nhanh *khong vach*). \
         🔵 Cau thu nam (da ve huu) KHONG con o day tu Story 2.8: no bi loc o tang Rust. Moi \
         phan tu con lai phai mang `retired_at = None` -- mot `true` lot vao day nghia la bo \
         loc dang de mot hang ve huu di qua"
    );

    // Co ket doan van la thu DA LUU, khong bi luot `UPDATE` dung toi.
    assert_eq!(
        loaded
            .segments
            .iter()
            .map(|s| s.is_paragraph_end)
            .collect::<Vec<_>>(),
        vec![false, true, false, false],
    );

    drop(opened);
    cleanup(&root);
}

// ═════════════════════════════════════════════════════════════════════════════
// Story 2.3 — ĐƯỜNG FLUSH của AD-35: AC4 · AC12 · AC13 · AC14 · AC15 · AC16
// ═════════════════════════════════════════════════════════════════════════════
//
// ⚠️ Mọi ca dưới đây gọi HÀM THUẦN `save_segment_targets`, không vỏ `wire` — cùng khuôn
// mọi ca của Story 2.1/2.2. Đó là thứ nghiệm thu được **mà không cần webview**.

/// **Mười** cột thật của `segment` hôm nay, đọc lại bằng SQL để khẳng định AC14.
///
/// ⚠️ Đọc bằng SQL **thô** chứ không qua `read_open_chapter_segments`, và đó là điều kiện
/// để phép kiểm có nghĩa: lệnh đọc của sản phẩm chỉ trả **sáu** trường, nên nó **không thấy**
/// `created_at`/`updated_at`/`chapter_id` — đúng ba cột mà AC14 nói phải y nguyên hoặc phải
/// đổi. Một phép kiểm đi qua lệnh đọc là một phép kiểm mù với ba cột nó phải canh.
///
/// 🔵 **CẬP NHẬT 2026-08-14 (Story 2.5, AC8 · Task 4.1): chín cột → MƯỜI.** Bước di trú 7
/// thêm `status`, và một phép kiểm *"tám cột kia y nguyên"* đọc chín cột sẽ **bỏ sót đúng
/// cột mới** — tức nó vẫn xanh trong khi `save_segment_targets` bơm `status` vào câu `UPDATE`
/// của nó và phá AD-31 hàng 1.
/// 🔴 Đây là **nâng phép kiểm cho nó nói thật về lược đồ mới**, không phải nới nó cho hết đỏ:
/// mỗi cột thêm vào đây là một cột nữa mà cổng AC8 canh, không phải một cột nữa nó bỏ qua.
///
/// 🔵 **CẬP NHẬT 2026-08-15 (Story 2.5c, AC7): mười cột → MƯỜI MỘT** — bước di trú 8 thêm
/// `is_omitted`. Cùng lý do nguyên văn ở trên, và lần này lưới đã làm đúng việc của nó:
/// `the_raw_column_reader_sees_every_column_...` ngay dưới đỏ **ngay lượt biên dịch đầu
/// tiên** sau khi bước 8 vào, chứ không để cột mới trôi qua trong im lặng.
///
/// 🔵 **CẬP NHẬT 2026-08-16 (Story 2.5d, AC5): mười một cột → MƯỜI HAI** — bước di trú 9
/// thêm `is_target_paragraph_end`. Lưới lại làm đúng việc của nó, **lượt thứ hai liên
/// tiếp**: ca ngay dưới đỏ ngay lượt chạy đầu sau khi bước 9 vào.
/// ⚠️ Và cột này là cột **đáng canh nhất từ trước tới nay** ở đây: `save_segment_targets`
/// chạy mỗi lượt flush, còn cờ đích chỉ đổi bằng một thao tác **rời rạc** của người dùng.
/// Một câu `UPDATE` của flush lỡ chạm vào nó sẽ **hoàn tác quyết định ngắt đoạn** của người
/// dùng mỗi hai giây, và không một lỗi nào được ném.
///
/// 🔵 **CẬP NHẬT 2026-08-16 (Story 2.7, AC6): mười hai cột → MƯỜI BA** — bước di trú 11 thêm
/// `translation_origin`. Lưới lại làm đúng việc của nó, **lượt thứ ba liên tiếp**.
/// ⚠️ Và ở đây nó canh một mệnh đề **mới**, không chỉ lặp lại mệnh đề cũ: AC6 nói xuất xứ ghi
/// *"cùng lúc với chuyển tiếp sang đã xác nhận, **không ở chỗ nào khác**"*. Một câu `UPDATE`
/// của `save_segment_targets` lỡ chạm vào cột này sẽ khai *tôi dịch* cho một câu người dùng
/// chưa ký — và hậu quả **không** dừng ở màn hình: Epic 7 ghi cặp TM theo nhãn đó, `RagInjector`
/// xếp nó lên đầu, và không lần ngược được. Đây đúng nghĩa vế phủ định của AC6, và cổng AC8
/// (`a_flush_touches_exactly_...`) là chỗ nó được canh.
/// 🔴 **MỘT TUPLE STRUCT, KHÔNG một tuple trần — và đó là một trần THẬT của ngôn ngữ, không
/// một lượt đổi cho đẹp.** Story 2.7, 2026-08-16.
///
/// Cột thứ **mười ba** làm bản cũ *(một `type` alias cho tuple trần)* **không biên dịch được
/// nữa**: `std` chỉ `impl` `PartialEq`/`Debug` cho tuple **tới 12 phần tử**, nên mọi
/// `assert_eq!` trên `Vec<SegmentRow>` chết cùng lúc với `E0369` + `E0277`. Đây là một **trần
/// cứng**, không một lượt thiếu `derive` — không cách nào nới nó từ phía kho.
///
/// ⚠️ **Đường sai rẻ ở đây là gộp hai cột vào một tuple lồng cho đủ 12** — nó biên dịch, và
/// nó làm chính cổng `the_raw_column_reader_sees_every_column_...` ngay dưới **đếm sai số
/// cột**. Tức lượt né trần sẽ tắt đúng cái lưới tồn tại để bắt cột mới. Một tuple **struct**
/// giữ nguyên `.0` … `.12` ở mọi chỗ gọi *(không một chỗ dùng nào phải sửa)*, nhận `derive`
/// ở **mọi** arity, và giữ phép đếm cột trung thực.
#[derive(Debug, Clone, PartialEq, Eq)]
struct SegmentRow(
    i64,
    i64,
    i64,
    String,
    i64,
    Option<String>,
    String,
    String,
    String,
    String,
    i64,
    i64,
    String,
);

fn read_all_segment_rows(open: &auratranslate_lib::commands::project::OpenWork) -> Vec<SegmentRow> {
    open.store
        .read(|conn| {
            let mut stmt = conn.prepare(
                "SELECT id, chapter_id, ord, source_text, is_paragraph_end, retired_at, \
                 created_at, updated_at, target_text, status, is_omitted, \
                 is_target_paragraph_end, translation_origin \
                 FROM segment ORDER BY ord",
            )?;
            let rows = stmt.query_map([], |r| {
                Ok(SegmentRow(
                    r.get(0)?,
                    r.get(1)?,
                    r.get(2)?,
                    r.get(3)?,
                    r.get(4)?,
                    r.get(5)?,
                    r.get(6)?,
                    r.get(7)?,
                    r.get(8)?,
                    r.get(9)?,
                    r.get(10)?,
                    r.get(11)?,
                    r.get(12)?,
                ))
            })?;
            rows.collect::<Result<Vec<_>, _>>()
        })
        .expect("doc lai muoi ba cot that bai")
}

/// 🔴 **Cổng tự kiểm: số cột mà [`read_all_segment_rows`] đọc phải bằng số cột THẬT của
/// bảng `segment`.**
///
/// Không có ca này, một bước di trú tương lai thêm cột thứ mười một sẽ để
/// `a_flush_touches_exactly_target_text_and_updated_at_and_nothing_else` **mù với đúng cột
/// mới** — nó vẫn xanh, và mệnh đề *"auto-save chạm đúng hai cột"* âm thầm hết được canh.
/// Đây là lớp lỗi mà chính Story 2.5 vừa suýt tạo ra, ghi thành một lưới thay vì một lời dặn.
#[test]
fn the_raw_column_reader_sees_every_column_the_segment_table_actually_has() {
    let root = temp_dir("column-count");
    let opened = create_work_from_text(&root, "Dem cot", "zh", "", "一。".to_owned())
        .expect("tao tac pham that bai");

    let real: i64 = opened
        .store
        .read(|conn| {
            conn.query_row("SELECT COUNT(*) FROM pragma_table_info('segment')", [], |r| {
                r.get(0)
            })
        })
        .expect("dem cot that bai");

    assert_eq!(
        real, 13,
        "bang `segment` co {real} cot, ma `read_all_segment_rows` doc 13. Mot cot moi PHAI \
         duoc them vao `SegmentRow` CUNG LUOT voi buoc di tru sinh ra no -- neu khong, cong \
         AC8 (`a_flush_touches_exactly_...`) mu voi dung cot do va van xanh"
    );

    let dir = opened.dir.clone();
    drop(opened);
    cleanup(&dir);
    cleanup(&root);
}

fn edit(id: i64, text: &str) -> SegmentTargetEdit {
    SegmentTargetEdit {
        id,
        target_text: text.to_owned(),
    }
}

// ── AC13 — MỘT lô, nhiều segment, MỘT lượt gọi ──────────────────────────────────

#[test]
fn one_flush_writes_every_changed_segment_in_a_single_call() {
    let root = temp_dir("flush-batch");
    let opened = create_work_from_text(&root, "Lo ghi", "zh", "", "一。二。三。四。".to_owned())
        .expect("tao tac pham that bai");

    let before = read_all_segment_rows(&opened);
    assert_eq!(before.len(), 4, "fixture phai co bon cau");
    let ids: Vec<i64> = before.iter().map(|r| r.0).collect();

    // Nguoi dung go xuyen qua BA cau trong mot nhip flush — ca that nhat cua AD-35.
    let outcome = save_segment_targets(
        Some(&opened),
        before[0].1,
        &[
            edit(ids[0], "Cau mot da dich."),
            edit(ids[1], "Cau hai da dich."),
            edit(ids[3], "Cau bon da dich."),
        ],
    )
    .expect("lo ghi that bai");

    assert_eq!(outcome.saved, 3, "ca ba hang phai duoc UPDATE trong MOT luot");
    assert_eq!(outcome.chapter_id, before[0].1);

    let after = read_all_segment_rows(&opened);
    assert_eq!(
        after.iter().map(|r| r.8.as_str()).collect::<Vec<_>>(),
        vec!["Cau mot da dich.", "Cau hai da dich.", "", "Cau bon da dich."],
        "cau THU BA khong nam trong lo, nen ban dich cua no phai con RONG"
    );

    drop(opened);
    cleanup(&root);
}

#[test]
fn an_empty_batch_is_a_valid_no_op_and_opens_no_transaction() {
    // Nhip flush co the bat gap mot luot khong con gi de ghi — do khong phai mot loi, va no
    // KHONG duoc mo mot giao dich rong tren writer noi tiep cua AD-11.
    let root = temp_dir("flush-empty");
    let opened = create_work_from_text(&root, "Lo rong", "zh", "", "一。二。".to_owned())
        .expect("tao tac pham that bai");
    let before = read_all_segment_rows(&opened);

    let outcome = save_segment_targets(Some(&opened), before[0].1, &[]).expect("lo rong bi tu choi");

    assert_eq!(outcome.saved, 0);
    assert_eq!(
        read_all_segment_rows(&opened),
        before,
        "mot lo rong khong duoc dung toi mot byte nao"
    );

    drop(opened);
    cleanup(&root);
}

// ── AC14 — câu `UPDATE` chạm ĐÚNG hai cột ───────────────────────────────────────

#[test]
fn a_flush_touches_exactly_target_text_and_updated_at_and_nothing_else() {
    let root = temp_dir("flush-two-columns");
    let opened = create_work_from_text(&root, "Hai cot", "zh", "", "一。二。\n三。".to_owned())
        .expect("tao tac pham that bai");

    let before = read_all_segment_rows(&opened);
    assert_eq!(before.len(), 3);
    let chapter_id = before[0].1;

    // `created_at` va `updated_at` sinh cung mot luot `strftime` luc INSERT, nen chung bang
    // nhau truoc luot flush. Do la dieu kien de phep kiem duoi day phan biet duoc hai cot.
    assert_eq!(
        before[0].6, before[0].7,
        "truoc luot flush, `created_at` va `updated_at` phai bang nhau"
    );

    save_segment_targets(Some(&opened), chapter_id, &[edit(before[0].0, "Ban dich moi.")])
        .expect("lo ghi that bai");

    let after = read_all_segment_rows(&opened);
    let (b, a) = (&before[0], &after[0]);

    // Hai cot ĐƯỢC phep doi.
    assert_eq!(a.8, "Ban dich moi.", "`target_text` phai doi");
    assert_ne!(
        a.7, b.7,
        "`updated_at` phai doi — va no sinh o tang SQL (`strftime`), khong truyen tu Rust"
    );

    // TAM cot con lai phai y nguyen TUNG BYTE — day la thu cuong che AD-31 hang 1 that su.
    // 🔵 CAP NHAT 2026-08-14 (Story 2.5): BAY → TAM, cot thu tam la `status`.
    assert_eq!(a.0, b.0, "`id` doi");
    assert_eq!(a.1, b.1, "`chapter_id` doi");
    assert_eq!(a.2, b.2, "`ord` doi");
    assert_eq!(a.3, b.3, "`source_text` doi — AD-4 dong bang ranh gioi");
    assert_eq!(a.4, b.4, "`is_paragraph_end` doi — AD-37 noi do la du lieu DA LUU");
    assert_eq!(a.5, b.5, "`retired_at` doi");
    assert_eq!(a.6, b.6, "`created_at` doi — no la moc TAO, khong phai moc sua");
    assert_eq!(
        a.9, b.9,
        "🔴 `status` doi — AD-31 HANG 1 noi auto-save de trang thai NGUYEN. Nhet `status` vao \
         cau `UPDATE` cua `save_segment_targets` la pha dung hang do; phep ha ve `'draft'` cua \
         AD-31 hang 3 song o `unconfirm_edited_segments`, mot ham RIENG"
    );

    // 🔴 LUOI TRON HANG — Story 2.7, 2026-08-16. Va no vao mot khuyet tat CO SAN.
    //
    // ⚠️ Danh sach `assert_eq!` tung cot o tren mu voi BA cot luc phep do nay chay: `.10`
    // (`is_omitted`, buoc 8) · `.11` (`is_target_paragraph_end`, buoc 9) · `.12`
    // (`translation_origin`, buoc 11). Doc-comment cua `SegmentRow` khai rang bo doc duoc nang
    // "CUNG LUOT voi buoc di tru sinh ra no" de cong nay khong mu -- ve do da lam dung; ve
    // con lai, THEM MOT DONG `assert_eq!` vao day, roi ca hai lan. Dung khuon "chu ky thi hanh
    // dung MOT NUA" da lap bon lan o 2.5b va 2.6: nua kho thi lam, nua la mot dong thi rot.
    //
    // ⇒ Phep khang dinh nay khong the muc lai: no dung hang KY VONG bang chinh hang truoc do,
    // thay dung HAI truong ma AC8 cho phep doi, roi so TRON HANG. Mot cot thu muoi bon them
    // vao mai sau di vao ca nay **mien phi** -- khong ai phai nho gi ca.
    //
    // ⚠️ Cac `assert_eq!` tung cot o tren KHONG bi thay the: chung o lai vi CAU THONG BAO cua
    // chung: mot lot do o `.9` phai noi "AD-31 hang 1", khong noi "mot hang khac nhau o dau do".
    let expected = SegmentRow(
        b.0,
        b.1,
        b.2,
        b.3.clone(),
        b.4,
        b.5.clone(),
        b.6.clone(),
        a.7.clone(),
        a.8.clone(),
        b.9.clone(),
        b.10,
        b.11,
        b.12.clone(),
    );
    assert_eq!(
        a, &expected,
        "AC8: mot luot flush duoc phep cham DUNG `target_text` va `updated_at`. Moi cot khac \
         phai y nguyen TUNG BYTE -- ke ca `translation_origin`, thu ma Epic 7 doc de gan nhan \
         mot cap TM: mot cau `UPDATE` cua flush lo cham vao no se khai `toi dich` cho mot cau \
         nguoi dung CHUA ky, va khong mot loi nao duoc nem"
    );

    // Hai cau KHONG nam trong lo phai y nguyen tron ven, ke ca `updated_at`.
    assert_eq!(&after[1..], &before[1..], "cau ngoai lo bi dung toi");

    drop(opened);
    cleanup(&root);
}

// ═════════════════════════════════════════════════════════════════════════════
// Story 2.5 — MÁY TRẠNG THÁI AD-31: xác nhận segment
//
// 🔴 Sáu ca dưới đây phủ **từng hàng** của bảng AD-31 mà story này chạm tới, cộng ba
//    lối từ chối của AC14/Quyết định #7. Bất biến #1 của story (*"bảng AD-31 đúng từng
//    hàng"*) là bất biến **vi phạm được mà không cổng nào đỏ** — đây là lý do chúng tồn tại.
// ═════════════════════════════════════════════════════════════════════════════

/// Đọc `(status, số hàng segment_version)` của một segment — thứ mọi ca dưới đây so.
fn read_state(open: &auratranslate_lib::commands::project::OpenWork, id: i64) -> (String, i64) {
    open.store
        .read(move |conn| {
            let status: String =
                conn.query_row("SELECT status FROM segment WHERE id = ?1", [id], |r| r.get(0))?;
            let versions: i64 = conn.query_row(
                "SELECT COUNT(*) FROM segment_version WHERE segment_id = ?1",
                [id],
                |r| r.get(0),
            )?;
            Ok((status, versions))
        })
        .expect("doc trang thai segment that bai")
}

/// 🔴 **Một `\n` đi TRỌN một vòng đĩa — không chặng nào cắt nó.** Story 2.5d, AC1 · Task 4.3.
///
/// ⚠️ Đây là mệnh đề mà **không** đường nghiệm thu nào khác canh được ở tầng dữ liệu:
/// `vitest` chạy trên fixture chép tay *(nó khẳng định tập chờ nhận `\n`, không khẳng định
/// đĩa nhận)*; bàn đo dừng ở DOM; e2e canh **cả** đường nhưng chỉ chạy tay và không nằm
/// trong CI. Ca này là chỗ duy nhất chạy mỗi lần `cargo test`.
///
/// 🔴 Ba chặng mà một `\n` có thể chết **mà không lỗi nào được ném**, và cả ba đều rẻ để ai
/// đó "cải thiện" sau này: một `trim()` ở đường ghi *(nghe rất hợp lý: "cắt khoảng trắng
/// thừa của người dùng")* · một `replace('\n', " ")` để *"chuẩn hoá"* · một `TEXT` collation
/// làm phẳng. Cả ba cho một chuỗi **trông như thật** đi xuống đĩa, và ngắt đoạn của người
/// dùng biến mất vĩnh viễn.
///
/// ⚠️ Ca này cố ý dùng một chuỗi mang **ba hình dạng biên** cùng lúc: `\n` giữa câu, `\n\n`
/// liền *(một dòng trống)*, và khoảng trắng **bao quanh** một `\n` — thứ mà một `trim()` theo
/// dòng sẽ ăn mất trong khi `trim()` toàn chuỗi thì không.
#[test]
fn a_newline_survives_the_whole_disk_round_trip_untouched() {
    let root = temp_dir("newline-round-trip");
    let opened = create_work_from_text(&root, "Xuong Dong", "zh", "", "一。二。".to_owned())
        .expect("tao tac pham that bai");

    let rows = read_all_segment_rows(&opened);
    let (id, chapter_id) = (rows[0].0, rows[0].1);

    let with_newlines = "Dong mot.\nDong hai.\n\nDong bon co  khoang trang .\n";
    save_segment_targets(Some(&opened), chapter_id, &[edit(id, with_newlines)])
        .expect("lo ghi that bai");

    let back: String = opened
        .store
        .read(move |conn| {
            conn.query_row("SELECT target_text FROM segment WHERE id = ?1", [id], |r| {
                r.get(0)
            })
        })
        .expect("doc lai `target_text` that bai");

    assert_eq!(
        back, with_newlines,
        "`target_text` phai ve NGUYEN VAN tu dia -- khong `trim`, khong `replace`, khong mot \
         luot \"chuan hoa\" nao. Neu chuoi doc ra thieu mot `\\n` hay mot khoang trang, mot \
         chang tren duong ghi da lam phang no, va ngat doan cua nguoi dung mat VINH VIEN"
    );

    // Va lenh doc cua san pham cung phai tra ve dung chuoi do -- mot chang thu hai, doc bang
    // duong ma that su chay o san pham.
    let loaded = read_open_chapter_segments(Some(&opened)).expect("nap lai that bai");
    let target = loaded
        .segments
        .iter()
        .find(|s| s.id == id)
        .expect("khong thay segment vua ghi");
    assert_eq!(
        target.target_text, with_newlines,
        "lenh doc `read_open_chapter_segments` phai mang `\\n` qua day y nguyen"
    );

    let dir = opened.dir.clone();
    drop(opened);
    cleanup(&dir);
    cleanup(&root);
}

/// 🔴 **Một ô CHỈ CÓ xuống dòng vẫn bị `confirm_segment` TỪ CHỐI ký.** Story 2.5d, Task 4.4.
///
/// ⚠️ Ca này canh một hành vi **đã đúng sẵn**, và nó tồn tại vì đúng lý do đó: `confirm_segment`
/// dùng `target_text.trim().is_empty()`, và `str::trim()` cắt theo `char::is_whitespace` của
/// Unicode — trong đó **có** `\n`. Nên một ô mà người dùng chỉ bấm `Enter` vài lần vẫn bị từ
/// chối, **hôm nay**.
///
/// 🔴 Vì sao phải canh: Story 2.5d là story đầu tiên làm `\n` thành một ký tự **hợp lệ và
/// thường gặp** trong `target_text`. Một người sau đọc `trim().is_empty()` rất dễ thấy nó
/// *"quá tay với dữ liệu mới"* và nới thành `is_empty()` — và lượt nới đó đi qua **mọi** cổng.
/// Hậu quả đã ghi sẵn ở doc-comment của `SegmentNothingToConfirm`: một `SegmentVersion` gần
/// như trống vào lịch sử FR101, rồi Epic 7 ghi một cặp TM có vế dịch là khoảng trắng, rồi
/// FR58 điền sẵn khoảng trắng đó ở một Chương sau. Hỏng vĩnh viễn.
#[test]
fn a_cell_holding_only_newlines_is_still_refused_by_confirm() {
    let root = temp_dir("confirm-only-newlines");
    let opened = create_work_from_text(&root, "Chi Xuong Dong", "zh", "", "一。二。".to_owned())
        .expect("tao tac pham that bai");

    let rows = read_all_segment_rows(&opened);
    let (id, chapter_id) = (rows[0].0, rows[0].1);

    for only_whitespace in ["\n", "\n\n\n", " \n \n "] {
        save_segment_targets(Some(&opened), chapter_id, &[edit(id, only_whitespace)])
            .expect("lo ghi that bai");

        let err = confirm_segment(Some(&opened), id, "")
            .err()
            .unwrap_or_else(|| panic!("mot o chi co {only_whitespace:?} PHAI bi tu choi ky"));
        assert_eq!(
            err.message_key(),
            MessageKey::SegmentNothingToConfirm,
            "phep tu choi phai PHAN BIET DUOC -- khong mot loi chung chung"
        );
        assert_eq!(
            read_state(&opened, id),
            ("draft".to_owned(), 0),
            "mot luot tu choi KHONG duoc doi trang thai va KHONG duoc sinh phien ban nao"
        );
    }

    // Doi chung: cung o do, them mot chu, thi ky duoc -- de ca nay khong xanh vi mot ly do
    // khac (vi du `confirm_segment` hong han).
    save_segment_targets(Some(&opened), chapter_id, &[edit(id, "Dong mot.\nDong hai.")])
        .expect("lo ghi that bai");
    confirm_segment(Some(&opened), id, "").expect("mot o CO chu va co `\\n` phai ky duoc");
    assert_eq!(read_state(&opened, id), ("confirmed".to_owned(), 1));

    let dir = opened.dir.clone();
    drop(opened);
    cleanup(&dir);
    cleanup(&root);
}

/// AD-31 hàng 2 — **xác nhận ⇒ `'confirmed'` VÀ đúng MỘT `SegmentVersion`.** (AC1, AC2)
#[test]
fn confirming_a_segment_sets_it_confirmed_and_writes_exactly_one_version() {
    let root = temp_dir("confirm-one");
    let opened = create_work_from_text(&root, "Xac nhan", "zh", "", "一。二。".to_owned())
        .expect("tao tac pham that bai");

    let rows = read_all_segment_rows(&opened);
    let (id, chapter_id) = (rows[0].0, rows[0].1);
    save_segment_targets(Some(&opened), chapter_id, &[edit(id, "Ban dich cua toi.")])
        .expect("lo ghi that bai");

    assert_eq!(read_state(&opened, id), ("draft".to_owned(), 0));

    let outcome = confirm_segment(Some(&opened), id, "").expect("xac nhan that bai");

    assert_eq!(outcome.segment_id, id);
    assert_eq!(outcome.status, "confirmed");
    assert!(outcome.version_created, "luot nay PHAI sinh mot phien ban");
    assert_eq!(read_state(&opened, id), ("confirmed".to_owned(), 1));

    // Phien ban giu **van ban luc ky**, khong mot con tro toi hang segment.
    let saved: String = opened
        .store
        .read(move |conn| {
            conn.query_row(
                "SELECT target_text FROM segment_version WHERE segment_id = ?1",
                [id],
                |r| r.get(0),
            )
        })
        .expect("doc phien ban that bai");
    assert_eq!(
        saved, "Ban dich cua toi.",
        "`SegmentVersion` phai chep van ban TAI THOI DIEM KY -- FR101 khoi phuc ve chinh no"
    );

    // Cau thu hai KHONG bi cham -- mot luot xac nhan cham DUNG mot segment.
    let other = rows[1].0;
    assert_eq!(read_state(&opened, other), ("draft".to_owned(), 0));

    let dir = opened.dir.clone();
    drop(opened);
    cleanup(&dir);
    cleanup(&root);
}

/// AD-31 hàng 1 — **auto-save sau khi xác nhận KHÔNG đổi trạng thái và KHÔNG tạo phiên
/// bản.** (AC4)
///
/// ⚠️ Trước Story 2.5, mệnh đề này chỉ nghiệm thu được bằng một doc-comment: cột `status`
/// và bảng `segment_version` chưa tồn tại, nên một test *"không có `SegmentVersion` nào"*
/// là một test **XANH RỖNG** (`commands/segment.rs`, doc-comment của `save_segment_targets`).
/// Nay hai thứ đó có thật, và ca này là lưới thật.
#[test]
fn an_auto_save_that_changes_nothing_leaves_the_state_machine_untouched() {
    let root = temp_dir("confirm-autosave");
    let opened = create_work_from_text(&root, "Auto save", "zh", "", "一。二。".to_owned())
        .expect("tao tac pham that bai");

    let rows = read_all_segment_rows(&opened);
    let (id, chapter_id) = (rows[0].0, rows[0].1);
    save_segment_targets(Some(&opened), chapter_id, &[edit(id, "Cau da dich.")])
        .expect("lo ghi that bai");
    confirm_segment(Some(&opened), id, "").expect("xac nhan that bai");
    assert_eq!(read_state(&opened, id), ("confirmed".to_owned(), 1));

    // Mot nhip flush mang DUNG van ban da co -- ca thuong nhat cua AD-35 (tran cung 5 giay
    // ban ra khi nguoi dung khong sua gi them).
    save_segment_targets(Some(&opened), chapter_id, &[edit(id, "Cau da dich.")])
        .expect("lo ghi that bai");

    assert_eq!(
        read_state(&opened, id),
        ("confirmed".to_owned(), 1),
        "auto-save KHONG duoc doi trang thai va KHONG duoc tao phien ban thu hai (AD-31 hang 1)"
    );

    let dir = opened.dir.clone();
    drop(opened);
    cleanup(&dir);
    cleanup(&root);
}

/// AD-31 hàng 3 — **sửa văn bản của segment đã xác nhận ⇒ về `'draft'`, KHÔNG tạo phiên
/// bản.** (AC3)
///
/// 🔴 Đây là hố (2) của AD-31 §Prevents: nếu sửa xong mà nó **vẫn** ở *đã xác nhận* thì
/// không lần xác nhận nào nữa xảy ra, và **cặp TM mới không bao giờ được ghi** (Epic 7).
#[test]
fn editing_a_confirmed_segment_returns_it_to_draft_without_writing_a_version() {
    let root = temp_dir("confirm-edit-back");
    let opened = create_work_from_text(&root, "Sua lai", "zh", "", "一。二。".to_owned())
        .expect("tao tac pham that bai");

    let rows = read_all_segment_rows(&opened);
    let (id, chapter_id) = (rows[0].0, rows[0].1);
    save_segment_targets(Some(&opened), chapter_id, &[edit(id, "Ban dau.")]).expect("lo ghi");
    confirm_segment(Some(&opened), id, "").expect("xac nhan that bai");
    assert_eq!(read_state(&opened, id), ("confirmed".to_owned(), 1));

    // Nguoi dung go tiep vao cau da ky -- di qua DUNG duong flush cua san pham.
    flush_segment_targets(Some(&opened), chapter_id, &[edit(id, "Sua lai roi.")])
        .expect("duong flush that bai");

    assert_eq!(
        read_state(&opened, id),
        ("draft".to_owned(), 1),
        "sua van ban cua mot cau da ky PHAI ha no ve `'draft'`, va KHONG tao phien ban thu hai"
    );

    let dir = opened.dir.clone();
    drop(opened);
    cleanup(&dir);
    cleanup(&root);
}

/// 🔴 **THỨ TỰ của đường flush là một mệnh đề nghiệm thu, không một chi tiết cài đặt.**
///
/// `unconfirm_edited_segments` so văn bản **đang có trên đĩa** với văn bản trong lô. Chạy nó
/// **sau** lượt ghi thì hai vế đã bằng nhau ⇒ phép so không phát hiện được gì, và một câu đã
/// ký ở lại `'confirmed'` mang một văn bản khác. Đó là hố (2) của AD-31 §Prevents: không lần
/// xác nhận nào nữa xảy ra ⇒ **cặp TM mới không bao giờ được ghi** (Epic 7), im lặng vĩnh viễn.
///
/// ⚠️ **Ca này ra đời từ một phép đo, không từ một lo lắng.** Bản đầu đặt thứ tự trong vỏ
/// `wire::save_segment_targets`; đảo hai dòng ở đó cho **54/54 xanh**, vì `tests/**` gọi một
/// vỏ cần `AppHandle` không được. Chạy đỏ-rồi-xanh: đảo hai dòng trong
/// `flush_segment_targets`, ca này phải ĐỎ.
#[test]
fn the_flush_path_lowers_the_state_before_it_writes_the_new_text() {
    let root = temp_dir("flush-order");
    let opened = create_work_from_text(&root, "Thu tu", "zh", "", "一。二。".to_owned())
        .expect("tao tac pham that bai");

    let rows = read_all_segment_rows(&opened);
    let (id, chapter_id) = (rows[0].0, rows[0].1);
    save_segment_targets(Some(&opened), chapter_id, &[edit(id, "Van ban cu.")]).expect("lo ghi");
    confirm_segment(Some(&opened), id, "").expect("xac nhan that bai");

    let (lowered, saved) =
        flush_segment_targets(Some(&opened), chapter_id, &[edit(id, "Van ban moi.")])
            .expect("duong flush that bai");

    assert_eq!(
        lowered, 1,
        "duong flush PHAI ha dung mot hang -- neu no chay SAU luot ghi, phep so van ban khong \
         con gi de so va con so nay la 0"
    );
    assert_eq!(saved.saved, 1);
    assert_eq!(read_state(&opened, id), ("draft".to_owned(), 1));

    let dir = opened.dir.clone();
    drop(opened);
    cleanup(&dir);
    cleanup(&root);
}

/// 🔴 **Hợp đồng phụ của AD-31: so bằng VĂN BẢN, cấm cờ *dirty*.** (AC3, AC11)
///
/// Người dùng gõ rồi **hoàn tác về nguyên trạng**. Cờ dirty nói *đã sửa*; so sánh văn bản
/// nói *không đổi*, và so sánh văn bản mới đúng ý nghĩa *"câu này là chữ của ai"*.
/// ⇒ Một lô flush mang **đúng** văn bản đang có KHÔNG được hạ trạng thái.
#[test]
fn a_flush_carrying_identical_text_never_unconfirms_because_the_contract_compares_text() {
    let root = temp_dir("confirm-undo");
    let opened = create_work_from_text(&root, "Hoan tac", "zh", "", "一。二。".to_owned())
        .expect("tao tac pham that bai");

    let rows = read_all_segment_rows(&opened);
    let (id, chapter_id) = (rows[0].0, rows[0].1);
    save_segment_targets(Some(&opened), chapter_id, &[edit(id, "Y nguyen.")]).expect("lo ghi");
    confirm_segment(Some(&opened), id, "").expect("xac nhan that bai");

    let touched = unconfirm_edited_segments(Some(&opened), chapter_id, &[edit(id, "Y nguyen.")])
        .expect("ha trang thai that bai");

    assert_eq!(
        touched, 0,
        "van ban KHONG doi thi KHONG mot hang nao duoc ha trang thai -- day la hop dong phu \
         cua AD-31, va mot co `dirty` se cho ket qua nguoc lai o dung ca nay"
    );
    assert_eq!(read_state(&opened, id), ("confirmed".to_owned(), 1));

    let dir = opened.dir.clone();
    drop(opened);
    cleanup(&dir);
    cleanup(&root);
}

/// AC13 — **xác nhận lại một câu đã xác nhận, văn bản không đổi, là VÔ HẠI.**
///
/// 🔴 Không có mệnh đề này, giữ phím xác nhận sẽ bơm lịch sử đầy bản sao y hệt và FR101
/// thành vô dụng — đúng hố (1) mà AD-31 §Prevents nêu tên.
#[test]
fn confirming_an_already_confirmed_segment_writes_no_second_version_and_no_new_timestamp() {
    let root = temp_dir("confirm-again");
    let opened = create_work_from_text(&root, "Ky lai", "zh", "", "一。二。".to_owned())
        .expect("tao tac pham that bai");

    let rows = read_all_segment_rows(&opened);
    let (id, chapter_id) = (rows[0].0, rows[0].1);
    save_segment_targets(Some(&opened), chapter_id, &[edit(id, "Mot lan thoi.")]).expect("lo ghi");
    confirm_segment(Some(&opened), id, "").expect("xac nhan lan dau that bai");

    let before = read_all_segment_rows(&opened);

    // Giu phim: nam luot xac nhan lien tiep tren cung mot cau.
    for _ in 0..5 {
        let again = confirm_segment(Some(&opened), id, "").expect("xac nhan lai PHAI vo hai");
        assert_eq!(again.status, "confirmed");
        assert!(
            !again.version_created,
            "mot segment DA o dich thi khong chuyen tiep -- khong phien ban nao duoc sinh"
        );
    }

    assert_eq!(read_state(&opened, id), ("confirmed".to_owned(), 1));
    assert_eq!(
        read_all_segment_rows(&opened),
        before,
        "xac nhan lai KHONG duoc dung toi mot byte nao cua hang `segment`, ke ca `updated_at`"
    );

    let dir = opened.dir.clone();
    drop(opened);
    cleanup(&dir);
    cleanup(&root);
}

/// AC14 + Quyết định #7 — **ba lối từ chối, và cả ba PHÂN BIỆT ĐƯỢC.**
///
/// *"Rỗng IM LẶNG bị cấm; rỗng CÓ LÝ DO thì không"* — không lối nào trả *"đã xong"* cho
/// một lượt không ghi gì.
#[test]
fn every_refusal_of_confirm_carries_its_own_message_key_and_writes_nothing() {
    let root = temp_dir("confirm-refusals");
    let opened = create_work_from_text(&root, "Tu choi", "zh", "", "一。二。".to_owned())
        .expect("tao tac pham that bai");

    let rows = read_all_segment_rows(&opened);
    let (id, chapter_id) = (rows[0].0, rows[0].1);
    let retired_id = rows[1].0;

    // ① Chua Tac pham nao mo.
    assert_eq!(
        confirm_segment(None, id, "").expect_err("phai tu choi").message_key(),
        MessageKey::WorkNoneOpen
    );

    // ② `segment.id` khong ton tai.
    assert_eq!(
        confirm_segment(Some(&opened), 9_999_999, "")
            .expect_err("phai tu choi")
            .message_key(),
        MessageKey::SegmentNotFound
    );

    // ③ Cau CHUA DICH (`target_text` rong) -- Quyet dinh #7, Ice ky 2026-08-14.
    assert_eq!(
        confirm_segment(Some(&opened), id, "")
            .expect_err("mot cau chua dich PHAI bi tu choi")
            .message_key(),
        MessageKey::SegmentNothingToConfirm,
        "cho phep se ghi mot cap TM co ve dich RONG o Epic 7, roi FR58 dien san bao rong do"
    );

    // ④ Segment DA VE HUU. ⚠️ Chua duong san pham nao cho segment ve huu (chu: Story 2.8),
    //    nen trang thai nay dung bang SQL TRUC TIEP -- day la mot HANG RAO VIET TRUOC.
    opened
        .store
        .write(move |tx: &Transaction<'_>| {
            tx.execute(
                "UPDATE segment SET target_text = 'Da dich roi.', \
                 retired_at = '2026-08-14T00:00:00.000Z' WHERE id = ?1",
                [retired_id],
            )?;
            Ok(())
        })
        .expect("dung trang thai ve huu that bai");

    assert_eq!(
        confirm_segment(Some(&opened), retired_id, "")
            .expect_err("mot segment da ve huu PHAI bi tu choi")
            .message_key(),
        MessageKey::SegmentRetired
    );

    // Khong lot tu choi nao duoc ghi mot phien ban.
    let total: i64 = opened
        .store
        .read(|conn| conn.query_row("SELECT COUNT(*) FROM segment_version", [], |r| r.get(0)))
        .expect("dem phien ban");
    assert_eq!(total, 0, "bon luot tu choi KHONG duoc sinh mot phien ban nao");
    assert_eq!(read_state(&opened, id), ("draft".to_owned(), 0));

    // Va khong lot nao dung toi Chuong -- `chapter_id` chi de doc cho ro fixture.
    let _ = chapter_id;

    let dir = opened.dir.clone();
    drop(opened);
    cleanup(&dir);
    cleanup(&root);
}

/// 🔴 **LỆNH ĐỌC CỦA SẢN PHẨM PHẢI CHỞ `status` QUA DÂY — và ca này ra đời từ một lỗi THẬT.**
///
/// Bắt được bằng `e2e/specs/editor-confirm-segment.e2e.mjs` ngày 2026-08-14, **sau khi** cả bốn
/// đường nghiệm thu đã xanh: bản đầu của Story 2.5 thêm `status` vào kiểu TypeScript và cho
/// `segmentRuleInputOf` đọc nó, nhưng **quên** thêm vào `ChapterSegment` phía Rust và vào câu
/// `SELECT`. Rust không gửi trường đó ⇒ `segment.status` là `undefined` trong webview ⇒
/// `isConfirmed` **luôn `false`** ⇒ vạch `confirmed` không bao giờ hiện **trên sản phẩm thật**.
///
/// ⚠️ Nó đi lọt **74/74** test frontend, vì fixture vitest dựng `ChapterSegment` **bằng tay** và
/// có cấp `status`. Một fixture chép tay trôi được khỏi sự thật của dây; ca này đi qua **chính
/// lệnh đọc của sản phẩm**, nên nó không trôi được.
///
/// 🔴 Ca này canh **cả hai vế**, và vế thứ hai mới là vế đắt: trường phải **có mặt**, và giá trị
/// của nó phải **đi theo dữ liệu thật** — một hằng `"draft"` viết cứng ở tầng dựng struct sẽ đi
/// lọt một phép kiểm chỉ hỏi *"trường có tồn tại không"*.
#[test]
fn the_load_command_carries_the_status_column_over_the_wire() {
    let root = temp_dir("wire-status");
    let opened = create_work_from_text(&root, "Day", "zh", "", "一。二。".to_owned())
        .expect("tao tac pham that bai");

    let rows = read_all_segment_rows(&opened);
    let (first, second) = (rows[0].0, rows[1].0);
    let chapter_id = rows[0].1;

    // Moi segment moi nhap phai di ra ngoai day o `'draft'`.
    let fresh = read_open_chapter_segments(Some(&opened)).expect("nap segment that bai");
    assert!(
        fresh.segments.iter().all(|s| s.status == "draft"),
        "moi segment vua nhap phai di ra day o `'draft'` -- mot ban dich cu KHONG duoc tu ky"
    );

    // Xac nhan DUNG MOT cau, roi doc lai qua chinh lenh cua san pham.
    save_segment_targets(Some(&opened), chapter_id, &[edit(first, "Da dich.")])
        .expect("lo ghi that bai");
    confirm_segment(Some(&opened), first, "").expect("xac nhan that bai");

    let loaded = read_open_chapter_segments(Some(&opened)).expect("nap lai segment that bai");
    let a = loaded
        .segments
        .iter()
        .find(|s| s.id == first)
        .expect("khong thay segment vua xac nhan");
    let b = loaded
        .segments
        .iter()
        .find(|s| s.id == second)
        .expect("khong thay segment thu hai");

    assert_eq!(
        a.status, "confirmed",
        "cau vua xac nhan phai di ra day o `'confirmed'` -- neu no van la `'draft'`, tang \
         hien thi khong bao gio ve duoc vach `confirmed` va KHONG mot test frontend nao bat duoc"
    );
    assert_eq!(
        b.status, "draft",
        "cau KHONG duoc xac nhan phai o `'draft'` -- neu hai cau cho cung mot gia tri thi \
         truong nay dang la mot hang viet cung, khong phai du lieu that"
    );
}

/// 🔴 **LỆNH ĐỌC CỦA SẢN PHẨM PHẢI CHỞ `is_omitted` QUA DÂY** — Story 2.5c, Task 2.3.
///
/// Ca ngay trên ghi lại một lỗi **đã xảy ra thật**: bản đầu của Story 2.5 thêm `status` vào
/// CSDL và vào kiểu TypeScript nhưng **quên** hai chỗ — struct [`ChapterSegment`] và câu
/// `SELECT` — nên trường đó là `undefined` trong webview trong khi **74/74** test frontend
/// vẫn xanh *(fixture vitest chép tay có sẵn cột)*. Chỉ e2e bắt được.
///
/// 🔴 Story 2.5c thêm **đúng một cột nữa vào đúng đường đó** ⇒ **cùng cái bẫy, cùng vị
/// trí**. Ca này là lưới, viết **trước** khi cột đi vào struct.
///
/// ⚠️ Cờ đặt bằng **SQL trực tiếp**, không qua lệnh của Task 3, và đó là chủ ý: mệnh đề ở
/// đây là *"câu `SELECT` chở cột"*, không phải *"lệnh ghi được cờ"*. Trộn hai mệnh đề vào
/// một ca là dựng một ca đỏ vì hai lý do khác nhau — và ca của lệnh đã có chủ riêng.
///
/// 🔴 Vế đắt là vế thứ hai: giá trị phải **đi theo dữ liệu thật**. Một `is_omitted: false`
/// viết cứng ở tầng dựng struct đi lọt mọi phép kiểm chỉ hỏi *"trường có tồn tại không"* —
/// nên ca này đòi **hai** segment cho **hai** giá trị khác nhau trong **cùng một** lượt nạp.
#[test]
fn the_load_command_carries_the_is_omitted_column_over_the_wire() {
    let root = temp_dir("wire-omitted");
    let opened = create_work_from_text(&root, "Day cat bo", "zh", "", "一。二。".to_owned())
        .expect("tao tac pham that bai");

    let rows = read_all_segment_rows(&opened);
    let (first, second) = (rows[0].0, rows[1].0);

    // Moi segment moi nhap phai di ra ngoai day o "khong cat bo".
    let fresh = read_open_chapter_segments(Some(&opened)).expect("nap segment that bai");
    assert!(
        fresh.segments.iter().all(|s| !s.is_omitted),
        "moi segment vua nhap phai di ra day o `is_omitted = false` -- backfill cua buoc 8 \
         la 0, va mot cau tu nhien bien mat khoi ban dich la lop loi nang nhat cua AC5"
    );

    opened
        .store
        .write(move |tx: &Transaction<'_>| {
            tx.execute("UPDATE segment SET is_omitted = 1 WHERE id = ?1", [first])?;
            Ok(())
        })
        .expect("dat co bang SQL truc tiep that bai");

    let loaded = read_open_chapter_segments(Some(&opened)).expect("nap lai segment that bai");
    let a = loaded
        .segments
        .iter()
        .find(|s| s.id == first)
        .expect("khong thay segment da cat bo");
    let b = loaded
        .segments
        .iter()
        .find(|s| s.id == second)
        .expect("khong thay segment thu hai");

    assert!(
        a.is_omitted,
        "cau da cat bo phai di ra day o `is_omitted = true` -- neu no van `false`, tang hien \
         thi khong bao gio gach ngang duoc no va KHONG mot test frontend nao bat duoc"
    );
    assert!(
        !b.is_omitted,
        "cau KHONG cat bo phai o `false` -- neu hai cau cho cung mot gia tri thi truong nay \
         dang la mot hang viet cung, khong phai du lieu that"
    );

    // AC2 — TRUC DOC LAP: mot luot dat co KHONG duoc dung toi `status` hay `target_text`.
    assert_eq!(
        (a.status.as_str(), a.target_text.as_str()),
        ("draft", ""),
        "cat bo la mot truc doc lap -- no khong duoc cham `status` lan `target_text`"
    );

    let dir = opened.dir.clone();
    drop(opened);
    cleanup(&dir);
    cleanup(&root);
}

/// 🔴 **Cột `is_target_paragraph_end` đi qua dây** — Story 2.5d, AC4 · Task 5.3.
///
/// ⚠️ **Cột thứ BA đi vào đúng đường đã hỏng một lần**, và vụ đó ghi nguyên văn ở
/// `segment.rs:144-153`: Story 2.5 thêm `status` vào kiểu TypeScript nhưng quên struct và
/// quên câu `SELECT` ⇒ `undefined` phía webview, `isConfirmed` **luôn `false` trên sản phẩm
/// thật**, mà **74/74** test frontend vẫn xanh vì fixture vitest chép tay có sẵn cột.
///
/// 🔴 Và cột này là cột **nguy hiểm nhất trong ba**, vì hỏng của nó **không nhìn thấy được ở
/// bề mặt đang có**: AC4 nói *"cần cấu trúc đoạn của bản dịch thì đọc dữ liệu đã lưu"*, mà
/// bề mặt tiêu thụ thật của nó là **đường xuất** (Epic 8) — chưa tồn tại. Một `undefined`
/// im lặng ở đây sẽ nằm chờ tới ngày Epic 8 chạy, rồi biểu hiện thành *"bản dịch xuất ra
/// mất hết ngắt đoạn"* mà không ai lần về được story này.
///
/// ⚠️ Cờ đặt bằng **SQL trực tiếp**: mệnh đề ở đây là *"câu `SELECT` chở cột"*, không phải
/// *"lệnh ghi được cờ"* — ca của lệnh có chủ riêng.
#[test]
fn the_load_command_carries_the_target_paragraph_end_column_over_the_wire() {
    let root = temp_dir("wire-target-para");
    let opened = create_work_from_text(&root, "Day ngat doan", "zh", "", "一。二。".to_owned())
        .expect("tao tac pham that bai");

    let rows = read_all_segment_rows(&opened);
    let (first, second) = (rows[0].0, rows[1].0);

    opened
        .store
        .write(move |tx: &Transaction<'_>| {
            tx.execute(
                "UPDATE segment SET is_target_paragraph_end = 1 WHERE id = ?1",
                [first],
            )?;
            tx.execute(
                "UPDATE segment SET is_target_paragraph_end = 0 WHERE id = ?1",
                [second],
            )?;
            Ok(())
        })
        .expect("dat co bang SQL truc tiep that bai");

    let loaded = read_open_chapter_segments(Some(&opened)).expect("nap lai segment that bai");
    let a = loaded
        .segments
        .iter()
        .find(|s| s.id == first)
        .expect("khong thay segment thu nhat");
    let b = loaded
        .segments
        .iter()
        .find(|s| s.id == second)
        .expect("khong thay segment thu hai");

    // 🔴 HAI segment cho HAI gia tri trong CUNG mot luot nap -- mot `false` viet cung o tang
    // dung struct di lot moi phep kiem chi hoi "truong co ton tai khong".
    assert!(
        a.is_target_paragraph_end,
        "cau co co dich BAT phai di ra day o `true`"
    );
    assert!(
        !b.is_target_paragraph_end,
        "cau co co dich TAT phai di ra day o `false` -- neu hai cau cho cung mot gia tri thi \
         truong nay dang la mot hang viet cung, khong phai du lieu that"
    );

    // 🔴 Va co DICH khong duoc dung toi co NGUON: AD-37 van so huu `is_paragraph_end`.
    assert!(
        !a.is_paragraph_end && !b.is_paragraph_end,
        "mot luot dat co DICH khong duoc doi co NGUON -- hai cot doc lap tu buoc 9"
    );

    let dir = opened.dir.clone();
    drop(opened);
    cleanup(&dir);
    cleanup(&root);
}

/// 🔴 **AC2 — một Chương VỪA NHẬP có cờ đích BẰNG cờ nguồn từng hàng, gồm cả hàng cuối.**
/// Story 2.5d, AC2 · AC3 · Task 6.1.
///
/// ⚠️ **Ca này canh một đường mà bước di trú 9 KHÔNG chạm tới, và đó là cả điểm của nó.**
/// Bước 9 backfill các hàng **đã có trên đĩa**. Một Chương nhập *sau* lượt di trú đi qua
/// `insert_segments`, và ở đó `DEFAULT 0` của `ALTER TABLE` là thứ duy nhất cấp giá trị —
/// tức **mọi Chương mới sẽ có cờ đích tắt hết** trong khi cờ nguồn bật đúng chỗ, nếu câu
/// `INSERT` không set tường minh.
/// 🔴 Đó là ca **thường nhất** của AC2 *(một Chương vừa nhập)*, không một ca biên, và nó
/// **không** được ca backfill ở trên canh: hai ca, hai đường, hai chủ.
///
/// 🔴 **AC3 — ba ca biên của AD-37 áp Y NGUYÊN, và ca này đo vế duy nhất có mã thi hành:**
/// *"segment cuối Chương ⇒ cờ tắt, luôn luôn"* (`split.rs::mark_paragraph_end`). Vì cờ đích
/// **bằng** cờ nguồn lúc nhập, ba ca biên đúng cho cờ đích **theo dẫn xuất** — và mệnh đề đó
/// chỉ đứng chừng nào phép bằng nhau ở đây còn đúng. Ca này là chỗ nó được cưỡng chế.
#[test]
fn a_freshly_imported_chapter_mirrors_the_source_flag_into_the_target_flag_row_by_row() {
    let root = temp_dir("fresh-mirror");
    // Ba doan: hai cau doan mot, mot cau doan hai, hai cau doan ba -- de co ca hang BAT
    // lan hang TAT, va de hang CUOI roi vao ca bien cua AD-37.
    let source = "一。二。\n三。\n四。五。".to_owned();
    let opened = create_work_from_text(&root, "Soi Guong", "zh", "", source)
        .expect("tao tac pham that bai");

    let loaded = read_open_chapter_segments(Some(&opened)).expect("nap segment that bai");
    assert!(
        loaded.segments.len() >= 3,
        "fixture phai co it nhat ba cau -- neu khong ca nay khong phan biet duoc gi"
    );

    // 🔴 Menh de trung tam: TUNG HANG, khong phai "tat ca deu 0" hay "tat ca deu 1".
    let mismatched: Vec<(i64, bool, bool)> = loaded
        .segments
        .iter()
        .filter(|s| s.is_paragraph_end != s.is_target_paragraph_end)
        .map(|s| (s.ord, s.is_paragraph_end, s.is_target_paragraph_end))
        .collect();
    assert!(
        mismatched.is_empty(),
        "mot Chuong vua nhap phai co co dich BANG co nguon tung hang (AC2 -- \"ban dich soi \
         guong ban goc cho toi khi nguoi dung doi\"). Cac hang lech (ord, nguon, dich): \
         {mismatched:?}. Neu MOI hang lech deu co dang (_, true, false) thi cau `INSERT` cua \
         `insert_segments` chua set cot moi va `DEFAULT 0` dang cap gia tri"
    );

    // Doi chung: fixture phai that su co CA HAI loai hang, neu khong phep kiem tren vo nghia.
    assert!(
        loaded.segments.iter().any(|s| s.is_paragraph_end),
        "fixture phai co it nhat mot hang ket doan -- neu khong, \"bang nhau tung hang\" xanh \
         mot cach vo nghia voi ca mot cot toan 0"
    );

    // 🔴 AC3, ca bien co ma thi hanh: cau CUOI Chuong ⇒ co TAT, luon luon -- va vi hai co
    // bang nhau luc nhap, no tat o CA HAI cot.
    let last = loaded
        .segments
        .last()
        .expect("Chuong phai co it nhat mot cau");
    assert!(
        !last.is_paragraph_end && !last.is_target_paragraph_end,
        "cau CUOI Chuong khong bao gio ket doan (AD-37, `split.rs::mark_paragraph_end`), va \
         co dich thua ke dung menh de do qua phep bang nhau luc nhap"
    );

    let dir = opened.dir.clone();
    drop(opened);
    cleanup(&dir);
    cleanup(&root);
}

// ═════════════════════════════════════════════════════════════════════════════
// Story 2.5c — CẮT BỎ CÂU KHỎI BẢN DỊCH (FR133): AC1 · AC2 · AC4
// ═════════════════════════════════════════════════════════════════════════════
//
// ⚠️ Mọi ca dưới đây gọi **hàm thuần** `set_segment_omitted`, không vỏ `wire` — cùng khuôn
// mọi ca của Story 2.1/2.2/2.3/2.5. Đó là thứ nghiệm thu được **mà không cần webview**.

/// 🔴 **AC2 — CẮT BỎ LÀ MỘT TRỤC ĐỘC LẬP: nó KHÔNG chạm `status` lẫn `target_text`.**
///
/// Đây là mệnh đề trung tâm của story, và nó là thứ làm AC4 đúng **mà không một dòng mã
/// khôi phục nào**: nếu lượt cắt bỏ không xoá gì thì lượt bỏ cờ không phải dựng lại gì.
///
/// ⚠️ Ca này cố ý chọn một câu **đã xác nhận** *(`status = 'confirmed'`)* chứ không một câu
/// `'draft'`: một cài đặt sai hạ trạng thái về `'draft'` *("cắt bỏ rồi thì còn xác nhận gì
/// nữa")* đi lọt hoàn toàn nếu câu thử vốn đã là `'draft'`.
#[test]
fn omitting_a_segment_touches_the_flag_and_nothing_else() {
    let root = temp_dir("omit-independent");
    let opened = create_work_from_text(&root, "Truc doc lap", "zh", "", "一。二。".to_owned())
        .expect("tao tac pham that bai");

    let rows = read_all_segment_rows(&opened);
    let (id, chapter_id) = (rows[0].0, rows[0].1);
    save_segment_targets(Some(&opened), chapter_id, &[edit(id, "Da dich va da ky.")])
        .expect("lo ghi that bai");
    confirm_segment(Some(&opened), id, "").expect("xac nhan that bai");

    let before = read_all_segment_rows(&opened);

    let outcome = set_segment_omitted(Some(&opened), id, true).expect("cat bo that bai");
    assert_eq!(outcome.segment_id, id);
    assert!(outcome.is_omitted, "outcome phai noi dung trang thai MOI");

    let after = read_all_segment_rows(&opened);
    assert_eq!(
        after.len(),
        before.len(),
        "mot luot cat bo KHONG duoc them hay bot mot hang `segment` nao"
    );

    for (b, a) in before.iter().zip(after.iter()) {
        if b.0 == id {
            assert_eq!(a.10, 1, "cau vua cat bo phai co `is_omitted = 1`");
            // 🔴 Muoi cot con lai y nguyen -- ke ca `updated_at`. Cat bo KHONG sua mot ky
            // tu van ban nao, va `updated_at` mang nghia "moc sua VAN BAN" (xem
            // `SEGMENT_DDL` va doc-comment cua `confirm_segment`).
            assert_eq!(
                (
                    a.0, a.1, a.2, &a.3, a.4, &a.5, &a.6, &a.7, &a.8, &a.9
                ),
                (
                    b.0, b.1, b.2, &b.3, b.4, &b.5, &b.6, &b.7, &b.8, &b.9
                ),
                "cat bo la mot TRUC DOC LAP -- muoi cot kia phai y nguyen, ke ca `status` \
                 (`'confirmed'` o day), `target_text` va `updated_at`"
            );
        } else {
            assert_eq!(a, b, "luot cat bo KHONG duoc dung toi mot cau nao khac");
        }
    }

    // Va khong mot `SegmentVersion` nao duoc sinh -- cat bo khong phai mot chuyen tiep AD-31.
    let versions: i64 = opened
        .store
        .read(|conn| conn.query_row("SELECT COUNT(*) FROM segment_version", [], |r| r.get(0)))
        .expect("dem hang segment_version");
    assert_eq!(
        versions, 1,
        "chi mot phien ban tu luot XAC NHAN o tren -- cat bo KHONG duoc sinh them cai nao"
    );

    let dir = opened.dir.clone();
    drop(opened);
    cleanup(&dir);
    cleanup(&root);
}

/// 🔴 **AC4 — bỏ cờ ⇒ câu quay về ĐÚNG trạng thái cũ với NỘI DUNG cũ.**
///
/// Ca này đi trọn vòng: ký → cắt bỏ → bỏ cờ, rồi so **từng byte** của hàng với ảnh chụp
/// trước lượt cắt bỏ. Nó là phép đo cho mệnh đề *"không gì bị mất thì không gì phải khôi
/// phục"* — nếu một cài đặt nào đó hạ `status` hay xoá `target_text` lúc cắt bỏ, ca này đỏ
/// **ngay cả khi** lượt bỏ cờ có một đường khôi phục chạy đúng, vì đường đó không thể đoán
/// lại được `'confirmed'`.
#[test]
fn restoring_a_segment_brings_back_the_exact_old_state_and_the_old_text() {
    let root = temp_dir("omit-round-trip");
    let opened = create_work_from_text(&root, "Dao nguoc", "zh", "", "一。二。".to_owned())
        .expect("tao tac pham that bai");

    let rows = read_all_segment_rows(&opened);
    let (id, chapter_id) = (rows[0].0, rows[0].1);
    save_segment_targets(Some(&opened), chapter_id, &[edit(id, "Cau nay da duoc ky.")])
        .expect("lo ghi that bai");
    confirm_segment(Some(&opened), id, "").expect("xac nhan that bai");

    let before = read_all_segment_rows(&opened);

    set_segment_omitted(Some(&opened), id, true).expect("cat bo that bai");
    let restored = set_segment_omitted(Some(&opened), id, false).expect("bo co that bai");
    assert!(!restored.is_omitted, "outcome phai noi dung trang thai MOI");

    assert_eq!(
        read_all_segment_rows(&opened),
        before,
        "sau mot vong cat bo -> bo co, hang `segment` phai khop TUNG BYTE voi truoc do -- \
         AC4 doi \"dung trang thai cu voi noi dung cu\""
    );

    // Va lenh doc cua san pham cung phai noi dieu do.
    let loaded = read_open_chapter_segments(Some(&opened)).expect("nap lai that bai");
    let s = loaded
        .segments
        .iter()
        .find(|s| s.id == id)
        .expect("khong thay segment");
    assert!(!s.is_omitted);
    assert_eq!(s.status, "confirmed");
    assert_eq!(s.target_text, "Cau nay da duoc ky.");

    let dir = opened.dir.clone();
    drop(opened);
    cleanup(&dir);
    cleanup(&root);
}

/// **Đặt cờ về đúng giá trị nó đang có là một no-op** — không ghi một byte nào.
///
/// Cùng luật AC13 của `confirm_segment` và cùng lý do: giữ phím không được sinh ra một lượt
/// ghi thứ hai. Ở đây cái giá thấp hơn *(không có `segment_version` để nhân bản)*, nhưng
/// mệnh đề *"không chạm `updated_at`"* thì y hệt.
#[test]
fn setting_the_omitted_flag_to_the_value_it_already_has_writes_nothing() {
    let root = temp_dir("omit-idempotent");
    let opened = create_work_from_text(&root, "Bap benh", "zh", "", "一。二。".to_owned())
        .expect("tao tac pham that bai");

    let id = read_all_segment_rows(&opened)[0].0;

    // ① Bo co tren mot cau VON KHONG cat bo.
    let untouched = read_all_segment_rows(&opened);
    set_segment_omitted(Some(&opened), id, false).expect("bo co tren mot cau chua cat bo");
    assert_eq!(
        read_all_segment_rows(&opened),
        untouched,
        "bo co mot cau VON khong cat bo phai la mot no-op"
    );

    // ② Cat bo hai lan lien tiep.
    set_segment_omitted(Some(&opened), id, true).expect("cat bo lan dau");
    let after_first = read_all_segment_rows(&opened);
    for _ in 0..5 {
        let again = set_segment_omitted(Some(&opened), id, true).expect("cat bo lai PHAI vo hai");
        assert!(again.is_omitted);
    }
    assert_eq!(
        read_all_segment_rows(&opened),
        after_first,
        "cat bo lai KHONG duoc dung toi mot byte nao, ke ca `updated_at`"
    );

    let dir = opened.dir.clone();
    drop(opened);
    cleanup(&dir);
    cleanup(&root);
}

/// **Ba lối từ chối, và cả ba PHÂN BIỆT ĐƯỢC** — *"Rỗng IM LẶNG bị cấm"*.
///
/// ⚠️ Không khoá `err.segment.*` **mới** nào được thêm cho story này: ba nhánh từ chối ở
/// đây đã có khoá riêng từ Story 2.5 *(`WorkNoneOpen` · `SegmentNotFound` ·
/// `SegmentRetired`)*, và không nhánh nào của cắt bỏ là một lý do từ chối **mới**. Một khoá
/// thứ tư nói cùng một điều là một danh mục đóng bị nới không lý do.
#[test]
fn every_refusal_of_omitting_carries_its_own_message_key_and_writes_nothing() {
    let root = temp_dir("omit-refusals");
    let opened = create_work_from_text(&root, "Tu choi", "zh", "", "一。二。".to_owned())
        .expect("tao tac pham that bai");

    let rows = read_all_segment_rows(&opened);
    let (id, retired_id) = (rows[0].0, rows[1].0);

    // ① Chua Tac pham nao mo.
    assert_eq!(
        set_segment_omitted(None, id, true)
            .expect_err("phai tu choi")
            .message_key(),
        MessageKey::WorkNoneOpen
    );

    // ② `segment.id` khong ton tai.
    assert_eq!(
        set_segment_omitted(Some(&opened), 9_999_999, true)
            .expect_err("phai tu choi")
            .message_key(),
        MessageKey::SegmentNotFound
    );

    // ③ Segment DA VE HUU (AD-5). ⚠️ Chua duong san pham nao cho segment ve huu (chu: Story
    //    2.8), nen trang thai nay dung bang SQL TRUC TIEP -- mot HANG RAO VIET TRUOC, dung
    //    khuon `every_refusal_of_confirm_...` da dat.
    opened
        .store
        .write(move |tx: &Transaction<'_>| {
            tx.execute(
                "UPDATE segment SET retired_at = '2026-08-15T00:00:00.000Z' WHERE id = ?1",
                [retired_id],
            )?;
            Ok(())
        })
        .expect("dung trang thai ve huu that bai");

    let before = read_all_segment_rows(&opened);
    assert_eq!(
        set_segment_omitted(Some(&opened), retired_id, true)
            .expect_err("mot segment DA VE HUU phai bi tu choi")
            .message_key(),
        MessageKey::SegmentRetired
    );
    assert_eq!(
        read_all_segment_rows(&opened),
        before,
        "mot luot bi tu choi KHONG duoc ghi mot byte nao"
    );

    let dir = opened.dir.clone();
    drop(opened);
    cleanup(&dir);
    cleanup(&root);
}

// ── AC5 — CHỐT LỌC cho mọi đầu ra (Quyết định #2 đường (b), Ice ký 2026-08-15) ──
//
// 🔴 ĐỌC ĐÚNG MỨC: hai ca dưới đây khẳng định **cái chốt** lọc đúng. Chúng **KHÔNG** khẳng
// định AC5 đã đóng — Chế độ đọc và bản xuất đều là khung rỗng, nên vế *"không dấu vết,
// không `[…]`, không chỗ trống"* **không có bề mặt nào để nghiệm thu**. Vế đó là 🟡 và có
// chủ ở `deferred-work.md` (Epic 5 · Epic 8). Đừng đọc hai ca này thành "AC5 xong".

/// **Chốt lọc bỏ đúng câu đã cắt bỏ, và giữ nguyên THỨ TỰ của phần còn lại.**
///
/// ⚠️ Vế thứ tự không thừa: bản xuất render theo `ord`, và một phép lọc dựng lại danh sách
/// theo một thứ tự khác cho ra một Chương xáo trộn — một khuyết tật **không** biểu hiện
/// thành lỗi, chỉ thành một bản dịch đọc không hiểu.
#[test]
fn the_output_filter_drops_omitted_segments_and_keeps_the_order_of_the_rest() {
    use auratranslate_lib::core::segment::omit::{count_in_translation, segments_in_translation};

    let root = temp_dir("output-filter");
    let opened = create_work_from_text(&root, "Chot loc", "zh", "", "一。二。三。四。".to_owned())
        .expect("tao tac pham that bai");

    let rows = read_all_segment_rows(&opened);
    assert_eq!(rows.len(), 4, "fixture phai co bon cau");
    let (second, third) = (rows[1].0, rows[2].0);

    // Cat bo cau THU HAI va THU BA — hai cau LIEN TIEP o GIUA, ca kho nhat cho mot phep loc.
    set_segment_omitted(Some(&opened), second, true).expect("cat bo cau hai");
    set_segment_omitted(Some(&opened), third, true).expect("cat bo cau ba");

    let loaded = read_open_chapter_segments(Some(&opened)).expect("nap segment that bai");
    let kept = segments_in_translation(&loaded.segments);

    assert_eq!(
        kept.iter().map(|s| s.id).collect::<Vec<_>>(),
        vec![rows[0].0, rows[3].0],
        "chot loc phai bo DUNG hai cau da cat bo, va giu THU TU cua hai cau con lai"
    );
    assert!(
        kept.iter().all(|s| !s.is_omitted),
        "khong mot cau da cat bo nao duoc lot qua chot"
    );
    assert_eq!(
        count_in_translation(&loaded.segments),
        2,
        "phep dem phai dung CUNG mot vi tu -- hai ban sao cua vi tu se lech nhau im lang"
    );

    let dir = opened.dir.clone();
    drop(opened);
    cleanup(&dir);
    cleanup(&root);
}

/// **Chốt lọc KHÔNG đọc `status` lẫn `target_text`** — nó lọc trên **một** trục.
///
/// 🔴 Đây là AC2 nói ở tầng đầu ra. Một cài đặt "tiện tay" lọc luôn câu chưa dịch *("xuất
/// làm gì một câu rỗng")* là một **quyết định sản phẩm** chưa ai ký, và nó sẽ nuốt mất
/// những câu người dùng cố ý để trống. Ca này khoá vị từ lại ở đúng một vế.
#[test]
fn the_output_filter_looks_at_exactly_one_axis() {
    use auratranslate_lib::core::segment::omit::segments_in_translation;

    let root = temp_dir("filter-one-axis");
    let opened = create_work_from_text(&root, "Mot truc", "zh", "", "一。二。三。".to_owned())
        .expect("tao tac pham that bai");

    let rows = read_all_segment_rows(&opened);
    let (first, chapter_id) = (rows[0].0, rows[0].1);

    // Cau 1: da dich VA da ky. Cau 2 va 3: CHUA DICH (`target_text` rong, `status` 'draft').
    save_segment_targets(Some(&opened), chapter_id, &[edit(first, "Da dich va da ky.")])
        .expect("lo ghi that bai");
    confirm_segment(Some(&opened), first, "").expect("xac nhan that bai");

    let loaded = read_open_chapter_segments(Some(&opened)).expect("nap segment that bai");
    assert_eq!(
        segments_in_translation(&loaded.segments).len(),
        3,
        "khong cau nao bi cat bo ⇒ chot loc phai giu DU ba cau, ke ca hai cau CHUA DICH. \
         Loc them cau rong o day la mot quyet dinh san pham chua ai ky"
    );

    let dir = opened.dir.clone();
    drop(opened);
    cleanup(&dir);
    cleanup(&root);
}

// ═════════════════════════════════════════════════════════════════════════════
// Story 2.5d — AC3: BẢNG BA CA BIÊN CỦA AD-37, ÁP CHO MỘT **CẶP** CỜ
// ═════════════════════════════════════════════════════════════════════════════
//
// 🔴 Ba ca dưới đây canh một hàm mà **hôm nay chưa ai gọi** — Quyết định #6 đường (b), Ice
// ký 2026-08-15. Chủ thi hành là **Story 2.8** (gộp/tách tường minh, `backlog`).
//
// ⚠️ Chúng **không** đóng AC3. AC3 nói *"áp y nguyên"*, và một bảng không có chỗ áp thì
// chưa áp được — vế đó là 🟡 với chủ ghi ở `deferred-work.md`. Thứ ba ca này mua là: ngày
// Story 2.8 tới, bảng đã có **một** nguồn sự thật đã chạy, và một lượt cài chỉ cho cờ nguồn
// sẽ đỏ ở đây thay vì xoá quyết định ngắt đoạn của người dùng trong im lặng.

/// **Ca ① — segment cuối Chương: cả hai cờ tắt, LUÔN LUÔN.**
///
/// 🔴 Vế *"luôn luôn"* đo bằng một cặp cờ mà người dùng **đã đổi** (`source: false,
/// target: true`): nếu hàm chỉ chép cờ cũ khi nó tắt thì ca này đỏ. Đây là ca biên duy nhất
/// không hỏi cờ cũ, và cũng là ca duy nhất **đã có mã thi hành** ở đường nhập.
#[test]
fn the_last_segment_of_a_chapter_ends_no_paragraph_in_either_column() {
    use auratranslate_lib::core::segment::paragraph::{at_end_of_chapter, ParagraphFlags};

    for current in [
        ParagraphFlags::mirrored(false),
        ParagraphFlags::mirrored(true),
        ParagraphFlags {
            source: false,
            target: true,
        },
        ParagraphFlags {
            source: true,
            target: false,
        },
    ] {
        assert_eq!(
            at_end_of_chapter(current),
            ParagraphFlags {
                source: false,
                target: false
            },
            "cau CUOI Chuong khong bao gio ket doan o CA HAI cot -- ke ca khi nguoi dung da \
             tu bat co dich ({current:?}). Khong co gi dung sau no de tach khoi"
        );
    }
}

/// **Ca ② — gộp: hai cờ đi theo câu cuối MỘT CÁCH ĐỘC LẬP.**
///
/// 🔴 Fixture cố ý cho câu cuối một cặp cờ **lệch nhau** (`source: false, target: true`) —
/// đúng hình dạng mà một lượt cài *"cờ đích chắc cũng như cờ nguồn"* sẽ làm hỏng. Một nhóm
/// gộp mà mọi cặp đều soi gương sẽ xanh với **cả** một hàm chỉ chép cờ nguồn.
#[test]
fn merging_takes_both_flags_from_the_last_sentence_independently() {
    use auratranslate_lib::core::segment::paragraph::{merged, ParagraphFlags};

    let group = [
        ParagraphFlags::mirrored(true),
        ParagraphFlags {
            source: true,
            target: false,
        },
        ParagraphFlags {
            source: false,
            target: true,
        },
    ];

    assert_eq!(
        merged(&group),
        Some(ParagraphFlags {
            source: false,
            target: true
        }),
        "cap co sau khi gop phai la cap cua CAU CUOI, tung cot mot -- khong phai mot phep OR, \
         khong phai co nguon nhan doi sang cot dich"
    );

    // 🔴 Nhom RONG ⇒ `None`, khong mot cap co tat bia ra. Mot nhom gop rong la loi cua cho
    // goi, va tra loi cho no la dung lop "rong im lang" ma `project-context.md` cam.
    assert_eq!(
        merged(&[]),
        None,
        "mot nhom gop RONG phai tra `None` -- tra mot cap co tat la bia ra cau tra loi cho \
         mot cau hoi vo nghia"
    );
}

/// **Ca ③ — tách: mảnh cuối giữ cặp cờ, mọi mảnh trước tắt CẢ HAI cột.**
#[test]
fn splitting_keeps_both_flags_on_the_last_piece_and_clears_every_piece_before_it() {
    use auratranslate_lib::core::segment::paragraph::{split_into, ParagraphFlags};

    let current = ParagraphFlags {
        source: true,
        target: false,
    };
    let pieces = split_into(current, 3);

    assert_eq!(
        pieces,
        vec![
            ParagraphFlags {
                source: false,
                target: false
            },
            ParagraphFlags {
                source: false,
                target: false
            },
            current,
        ],
        "manh CUOI giu nguyen cap co; moi manh truoc no tat CA HAI cot -- mot luot tach de \
         co dich nguyen o moi manh se sinh n ranh gioi doan tu cho truoc do chi co mot"
    );

    // Mot manh ⇒ chinh no giu cap co: tach thanh mot la khong tach.
    assert_eq!(split_into(current, 1), vec![current]);
    // 🔴 `0` manh ⇒ danh sach RONG, cung luat khong-bia voi `merged`.
    assert!(split_into(current, 0).is_empty());
}

/// **Phép soi gương lúc nhập có ĐÚNG MỘT tên trong Rust** — AC2.
///
/// ⚠️ Ca này trông tầm thường và nó không tầm thường: `insert_segments` viết phép soi gương
/// thẳng trong câu `INSERT` *(một giá trị vào hai cột)* vì nó ở tầng SQL. Nếu tầng logic
/// cũng viết lại nó bằng tay ở mỗi chỗ gọi thì có **hai** bản sao của cùng một quy tắc, và
/// chúng sẽ lệch nhau vào ngày quy tắc đổi.
#[test]
fn mirroring_at_import_time_gives_the_target_flag_the_source_value() {
    use auratranslate_lib::core::segment::paragraph::ParagraphFlags;

    assert_eq!(
        ParagraphFlags::mirrored(true),
        ParagraphFlags {
            source: true,
            target: true
        }
    );
    assert_eq!(
        ParagraphFlags::mirrored(false),
        ParagraphFlags {
            source: false,
            target: false
        }
    );
}

/// 🔴 **Lệnh đổi cờ đích ghi NGAY, đúng MỘT cột, và không đụng cờ nguồn.**
/// Story 2.5d, AC2 · AC4 · Quyết định #3 đường (c).
#[test]
fn setting_the_target_paragraph_flag_writes_one_column_and_leaves_the_source_flag_alone() {
    use auratranslate_lib::commands::segment::set_segment_paragraph_end;

    let root = temp_dir("set-target-para");
    // "一。二。\n三。" ⇒ cau 2 ket doan o CA HAI cot luc nhap (soi guong).
    let opened = create_work_from_text(&root, "Doi co", "zh", "", "一。二。\n三。".to_owned())
        .expect("tao tac pham that bai");

    let loaded = read_open_chapter_segments(Some(&opened)).expect("nap segment that bai");
    let first = loaded.segments[0].id;
    assert!(
        !loaded.segments[0].is_paragraph_end && !loaded.segments[0].is_target_paragraph_end,
        "cau dau cua fixture phai TAT o ca hai cot -- neu khong ca nay khong do duoc gi"
    );

    // ── BAT co dich cho cau dau: nguoi dung muon ban dich xuong doan som hon ban goc ──
    let out = set_segment_paragraph_end(Some(&opened), first, true).expect("dat co that bai");
    assert_eq!(out.segment_id, first);
    assert!(out.is_target_paragraph_end, "ket qua tra ve trang thai SAU luot goi");

    let rows = read_all_segment_rows(&opened);
    let row = rows.iter().find(|r| r.0 == first).expect("khong thay hang");
    // 🔴 Cot 4 = `is_paragraph_end`, cot 11 = `is_target_paragraph_end` (xem `SegmentRow`).
    assert_eq!(row.11, 1, "co DICH phai bat");
    assert_eq!(
        row.4, 0,
        "co NGUON phai NGUYEN -- AD-37 van so huu no, va AD-46 khai bang chu \"AD-37 khong \
         sua mot chu\""
    );
    // AC2 -- truc doc lap: khong dung toi `status` lan `target_text`.
    assert_eq!(row.9, "draft", "`status` khong duoc doi");
    assert_eq!(row.8, "", "`target_text` khong duoc doi");

    // ── Dat lai DUNG gia tri dang co ⇒ khong ghi mot byte nao (cung luat AC13) ──
    let again = set_segment_paragraph_end(Some(&opened), first, true).expect("dat lai phai vo hai");
    assert!(again.is_target_paragraph_end);

    // ── TAT lai ⇒ ve dung trang thai cu ──
    set_segment_paragraph_end(Some(&opened), first, false).expect("bo co that bai");
    let rows = read_all_segment_rows(&opened);
    assert_eq!(
        rows.iter().find(|r| r.0 == first).expect("khong thay hang").11,
        0,
        "bo co phai dua cot ve 0"
    );

    let dir = opened.dir.clone();
    drop(opened);
    cleanup(&dir);
    cleanup(&root);
}

/// **Mọi phép từ chối của lệnh đổi cờ đích có hình dạng phân biệt được, và KHÔNG ghi gì.**
/// Story 2.5d, Task 9.3.
///
/// ⚠️ Hai nhánh đầu **dùng lại** khoá của `set_segment_omitted` thay vì dựng khoá mới: chúng
/// nói **cùng một** sự thật *("không có câu đó" / "câu đó đã về hưu")*, và hai cách gọi tên cho
/// một sự thật là hai chỗ phải dịch, phải đồng bộ, phải sửa cùng lúc.
///
/// 🔵 **CẬP NHẬT 2026-08-16 (code review) — nhánh ④ THÊM VÀO, kèm một khoá RIÊNG.** Bản đầu
/// của test này viết *"Hai nhánh"*, và lượt rà tìm ra nhánh thứ ba mà nó bỏ sót: **ca ① của
/// AD-37** *(segment cuối Chương → cờ tắt, LUÔN LUÔN)*. Trước lượt vá, hàng rào đó chỉ sống ở
/// đường **nhập** (`split::mark_paragraph_end`), nên một lệnh `Mod+Alt+P` trên câu cuối Chương
/// **bật được** cờ và lưới vẽ một ranh giới đoạn dưới câu cuối cùng. `SegmentEndsChapter` nói
/// một sự thật mà `not_found` lẫn `retired` đều không diễn đạt nổi: câu **tồn tại**, **còn
/// sống**, và vẫn không mang cờ được.
#[test]
fn every_refusal_of_the_target_paragraph_command_carries_its_own_message_key() {
    use auratranslate_lib::commands::segment::set_segment_paragraph_end;

    let root = temp_dir("set-target-para-refuse");
    let opened = create_work_from_text(&root, "Tu choi", "zh", "", "一。二。".to_owned())
        .expect("tao tac pham that bai");

    // ① Chua Tac pham nao mo.
    let err = set_segment_paragraph_end(None, 1, true)
        .err()
        .expect("khong co Tac pham nao mo PHAI bi tu choi");
    assert_eq!(err.message_key(), MessageKey::WorkNoneOpen);

    // ② Segment khong ton tai.
    let err = set_segment_paragraph_end(Some(&opened), 9_999_999, true)
        .err()
        .expect("mot segment la PHAI bi tu choi");
    assert_eq!(err.message_key(), MessageKey::SegmentNotFound);

    // ③ Segment da ve huu (AD-5).
    let rows = read_all_segment_rows(&opened);
    let retired_id = rows[0].0;
    opened
        .store
        .write(move |tx: &Transaction<'_>| {
            tx.execute(
                "UPDATE segment SET retired_at = '2026-08-16T00:00:00.000Z' WHERE id = ?1",
                [retired_id],
            )?;
            Ok(())
        })
        .expect("cho mot segment ve huu that bai");

    let before = read_all_segment_rows(&opened);
    let err = set_segment_paragraph_end(Some(&opened), retired_id, true)
        .err()
        .expect("mot segment DA VE HUU PHAI bi tu choi");
    assert_eq!(err.message_key(), MessageKey::SegmentRetired);
    assert_eq!(
        read_all_segment_rows(&opened),
        before,
        "mot luot tu choi KHONG duoc ghi mot byte nao"
    );

    // ── ④ Segment CUOI Chuong ⇒ khong BAT duoc co, LUON LUON (AC3, ca ① cua AD-37) ──
    //
    // 🔴 Fixture nay co DUNG HAI cau va cau dau vua bi cho ve huu o nhanh ③, nen cau cuoi la
    //    segment con song duy nhat. Do la chu y: phep kiem "co ke tiep khong" hoi
    //    `retired_at IS NULL`, nen mot hang da ve huu KHONG duoc tinh la nguoi ke tiep.
    let last_id = rows[rows.len() - 1].0;
    let before = read_all_segment_rows(&opened);
    let err = set_segment_paragraph_end(Some(&opened), last_id, true)
        .err()
        .expect("cau CUOI Chuong PHAI bi tu choi khi xin BAT co");
    assert_eq!(err.message_key(), MessageKey::SegmentEndsChapter);
    assert_eq!(
        read_all_segment_rows(&opened),
        before,
        "mot luot tu choi KHONG duoc ghi mot byte nao"
    );

    // ── ⑤ Chieu NGUOC LAI di tiep: BO co tren cau cuoi Chuong phai duoc phep ──
    //
    // 🔴 Day la nua thu hai cua quyet dinh, va no quan trong ngang nua thu nhat: neu dia dang
    //    mang `1` o cau cuoi (du lieu co truoc luot va nay, hoac mot lan sua bang SQL) thi day
    //    la duong DUY NHAT sua no ve dung. Tu choi ca hai chieu la khoa cung mot hang sai
    //    VINH VIEN.
    opened
        .store
        .write(move |tx: &Transaction<'_>| {
            tx.execute(
                "UPDATE segment SET is_target_paragraph_end = 1 WHERE id = ?1",
                [last_id],
            )?;
            Ok(())
        })
        .expect("dung mot hang SAI bang SQL that bai");

    let out = set_segment_paragraph_end(Some(&opened), last_id, false)
        .expect("BO co tren cau cuoi Chuong PHAI duoc phep -- day la duong sua duy nhat");
    assert!(!out.is_target_paragraph_end);
    assert_eq!(
        read_all_segment_rows(&opened)
            .iter()
            .find(|r| r.0 == last_id)
            .expect("khong thay hang")
            .11,
        0,
        "hang sai phai duoc sua ve 0"
    );

    let dir = opened.dir.clone();
    drop(opened);
    cleanup(&dir);
    cleanup(&root);
}

// ── AC16 — round-trip: gõ → flush → nạp lại, và ranh giới KHÔNG đổi ─────────────

#[test]
fn typed_text_round_trips_through_the_flush_and_the_load_command() {
    let root = temp_dir("flush-round-trip");
    let opened = create_work_from_text(
        &root,
        "Round trip",
        "zh",
        "",
        "一。二。\n三。四。五。".to_owned(),
    )
    .expect("tao tac pham that bai");

    let before = read_all_segment_rows(&opened);
    assert_eq!(before.len(), 5);
    let chapter_id = before[0].1;
    let ids: Vec<i64> = before.iter().map(|r| r.0).collect();
    let ords: Vec<i64> = before.iter().map(|r| r.2).collect();
    let flags_before: Vec<i64> = before.iter().map(|r| r.4).collect();

    // Chuoi co dau tieng Viet, mot em-dash, va mot chuoi RONG (nguoi dung xoa sach mot cau).
    let typed = [
        (ids[0], "Hắn đẩy cánh cửa ấy ra — bóng tối dày đặc."),
        (ids[1], "Gió thổi tới từ cuối hành lang."),
        (ids[2], ""),
        (ids[4], "Câu cuối cùng, có dấu đầy đủ: ăn, ắt, ệ, ỡ, ự."),
    ];
    let edits: Vec<SegmentTargetEdit> = typed.iter().map(|(id, t)| edit(*id, t)).collect();

    save_segment_targets(Some(&opened), chapter_id, &edits).expect("lo ghi that bai");

    // Nap lai qua DUNG lenh cua san pham.
    let loaded = read_open_chapter_segments(Some(&opened)).expect("nap segment that bai");

    assert_eq!(loaded.segments.len(), 5, "so hang KHONG duoc doi — go khong phai mot luot tach");
    assert_eq!(
        loaded.segments.iter().map(|s| s.id).collect::<Vec<_>>(),
        ids,
        "`id` phai y nguyen — AD-3 cam tai dung id da ve huu"
    );
    assert_eq!(
        loaded.segments.iter().map(|s| s.ord).collect::<Vec<_>>(),
        ords,
        "`ord` phai y nguyen"
    );
    assert_eq!(
        loaded
            .segments
            .iter()
            .map(|s| i64::from(s.is_paragraph_end))
            .collect::<Vec<_>>(),
        flags_before,
        "co ket doan phai y nguyen — AD-37"
    );
    assert_eq!(
        loaded
            .segments
            .iter()
            .map(|s| s.target_text.as_str())
            .collect::<Vec<_>>(),
        vec![typed[0].1, typed[1].1, "", "", typed[3].1],
        "ban dich phai khop TUNG CHUOI, ke ca chuoi rong"
    );
    // Ranh gioi cau (`source_text`) khong doi mot byte.
    assert_eq!(
        loaded
            .segments
            .iter()
            .map(|s| s.source_text.clone())
            .collect::<Vec<_>>(),
        before.iter().map(|r| r.3.clone()).collect::<Vec<_>>(),
        "`source_text` doi — AD-4 dong bang ranh gioi vinh vien"
    );

    drop(opened);
    cleanup(&root);
}

// ── AC13 vế "từ chối TRỌN lô" + ba ca biên (Task 2.9) ──────────────────────────

#[test]
fn saving_without_an_open_work_is_refused() {
    let err = save_segment_targets(None, 1, &[edit(1, "gi cung duoc")])
        .expect_err("ghi ma chua mo Tac pham phai bi tu choi");
    assert_eq!(err.code(), "work.none_open");
    assert_eq!(err.message_key(), MessageKey::WorkNoneOpen);
}

#[test]
fn saving_into_an_unknown_chapter_is_refused() {
    let root = temp_dir("flush-bad-chapter");
    let opened = create_work_from_text(&root, "Chuong la", "zh", "", "一。二。".to_owned())
        .expect("tao tac pham that bai");
    let before = read_all_segment_rows(&opened);

    let err = save_segment_targets(Some(&opened), 9_999, &[edit(before[0].0, "x")])
        .expect_err("chuong khong ton tai phai bi tu choi");

    assert_eq!(err.code(), "segment.chapter_not_found");
    assert_eq!(err.message_key(), MessageKey::SegmentChapterNotFound);
    assert_eq!(
        read_all_segment_rows(&opened),
        before,
        "mot lo bi tu choi khong duoc de lai mot byte nao"
    );

    drop(opened);
    cleanup(&root);
}

#[test]
fn a_batch_with_one_unknown_segment_id_is_refused_whole_and_writes_nothing() {
    // 🔴 Ca DAT NHAT cua lenh nay: mot lo ghi MOT PHAN de lai dung trang thai ma khong ai
    //    quan sat duoc — nguoi dung thay chu tren man hinh, dia giu mot nua, khong dau hieu
    //    nao bao. `Store::write` tra `Err` ⇒ ROLLBACK, va day la phep kiem cua menh de do.
    let root = temp_dir("flush-unknown-id");
    let opened = create_work_from_text(&root, "Id la", "zh", "", "一。二。三。".to_owned())
        .expect("tao tac pham that bai");

    let before = read_all_segment_rows(&opened);
    let chapter_id = before[0].1;

    let err = save_segment_targets(
        Some(&opened),
        chapter_id,
        &[
            edit(before[0].0, "Cau nay HOP LE."),
            edit(before[1].0, "Cau nay cung hop le."),
            edit(9_999_999, "Cau nay khong thuoc Chuong nao."),
        ],
    )
    .expect_err("lo mang mot id la phai bi tu choi TRON");

    assert_eq!(err.code(), "segment.unknown_ids");
    assert_eq!(err.message_key(), MessageKey::SegmentUnknownIds);
    assert_eq!(err.params().get("count").map(String::as_str), Some("1"));
    assert_eq!(err.retryable(), false, "mot id la khong sua duoc bang cach thu lai");

    assert_eq!(
        read_all_segment_rows(&opened),
        before,
        "HAI cau hop le trong lo cung KHONG duoc ghi — tu choi TRON, khong ghi mot phan"
    );

    drop(opened);
    cleanup(&root);
}

#[test]
fn a_segment_id_from_another_chapter_is_refused_and_never_crosses_over() {
    // `WHERE id = ?2 AND chapter_id = ?3` la nua thu hai cua phep kiem id. Khong co ve
    // `AND chapter_id`, mot id thuoc Chuong KHAC se duoc ghi vao — im lang, va lo tinh la
    // day nen khong phep kiem nao do.
    let root_a = temp_dir("flush-cross-a");
    let opened_a = create_work_from_text(&root_a, "Tac pham A", "zh", "", "一。二。".to_owned())
        .expect("tao tac pham A that bai");
    let rows_a = read_all_segment_rows(&opened_a);
    let chapter_a = rows_a[0].1;

    // MOT Chuong thu hai trong CUNG `project.db`, bom bang SQL: Epic 1 tao dung mot Chuong
    // moi Tac pham, nen day la cach duy nhat dung duoc ca nay hom nay (Story 2.11 mang duong
    // san pham cho nhieu Chuong).
    opened_a
        .store
        .write(move |tx: &Transaction<'_>| {
            tx.execute(
                "INSERT INTO chapter (ord, title, source_text, status, created_at, updated_at) \
                 SELECT 2, 'Chuong hai', 'Ba。Bon。', status, created_at, updated_at \
                 FROM chapter WHERE id = ?1",
                [chapter_a],
            )?;
            let other: i64 = tx.query_row("SELECT id FROM chapter WHERE ord = 2", [], |r| r.get(0))?;
            tx.execute(
                "INSERT INTO segment (chapter_id, ord, source_text, is_paragraph_end, created_at, updated_at) \
                 VALUES (?1, 1, 'Ba。', 0, strftime('%Y-%m-%dT%H:%M:%fZ','now'), \
                 strftime('%Y-%m-%dT%H:%M:%fZ','now'))",
                [other],
            )?;
            Ok(())
        })
        .expect("bom Chuong thu hai that bai");

    let all_before = read_all_segment_rows(&opened_a);
    let foreign = all_before
        .iter()
        .find(|r| r.1 != chapter_a)
        .expect("phai co mot segment thuoc Chuong khac");

    let err = save_segment_targets(Some(&opened_a), chapter_a, &[edit(foreign.0, "Ghi lan Chuong.")])
        .expect_err("mot id thuoc Chuong khac phai bi tu choi");

    assert_eq!(err.code(), "segment.unknown_ids");
    assert_eq!(
        read_all_segment_rows(&opened_a),
        all_before,
        "khong hang nao duoc doi — ke ca hang cua Chuong kia"
    );

    drop(opened_a);
    cleanup(&root_a);
}

// ═════════════════════════════════════════════════════════════════════════════════
// Code review 2026-08-14 — HAI LƯỚI CHO HAI KHE HỞ KHÔNG CỔNG NÀO BẮT ĐƯỢC
// ═════════════════════════════════════════════════════════════════════════════════

/// 🔴 **Một lô hỏng phải chết ở giao dịch MỘT, khi chưa một chữ ký nào bị hạ.**
///
/// `flush_segment_targets` chạy hai giao dịch nối tiếp: `unconfirm_edited_segments` commit
/// **trước**, `save_segment_targets` chạy **sau**. Trước lượt rà, phép kiểm *"lô phải ghi
/// đủ"* chỉ có ở giao dịch **thứ hai** — nên một lô mang một id lạ vẫn kịp hạ những câu hợp
/// lệ về `'draft'` rồi mới bị từ chối, và người dùng nhận một lỗi *"lô bị từ chối"* trong khi
/// một chữ ký thật đã âm thầm biến mất.
///
/// ⚠️ Ca này **không** trùng `a_batch_with_one_unknown_segment_id_is_refused_whole_and_writes_nothing`:
/// ca kia gọi thẳng `save_segment_targets` *(một giao dịch)* và so `target_text`; ca này đi
/// đường `flush_segment_targets` *(hai giao dịch)* và so `status`. Đúng chỗ lệch giữa hai hàm
/// là chỗ khuyết tật sống, nên nó cần một ca riêng.
#[test]
fn the_flush_path_refuses_an_unknown_id_before_it_lowers_a_single_signature() {
    let root = temp_dir("flush-unknown-id-no-lower");
    let opened = create_work_from_text(&root, "Ha truoc khi tu choi", "zh", "", "一。二。".to_owned())
        .expect("tao tac pham that bai");

    let rows = read_all_segment_rows(&opened);
    let chapter_id = rows[0].1;
    let signed = rows[0].0;

    // Dung mot chu ky THAT, khong mot co gia: ghi van ban roi xac nhan.
    save_segment_targets(Some(&opened), chapter_id, &[edit(signed, "Ban dich da ky.")])
        .expect("ghi van ban that bai");
    confirm_segment(Some(&opened), signed, "").expect("xac nhan that bai");

    let before = read_all_segment_rows(&opened);
    assert_eq!(
        read_state(&opened, signed),
        ("confirmed".to_owned(), 1),
        "tien de cua ca nay: segment PHAI dang o 'confirmed' voi dung mot version"
    );

    // Lo tron: mot id hop le DA KY mang van ban MOI (du ca ba dieu kien de bi ha), cong mot
    // id la. Truoc ban va, dong dau tien cua `flush_segment_targets` ha `signed` ve 'draft'
    // va COMMIT, roi giao dich hai moi tu choi.
    let err = flush_segment_targets(
        Some(&opened),
        chapter_id,
        &[
            edit(signed, "Ban dich vua sua, khac han ban da ky."),
            edit(9_999_999, "Cau nay khong thuoc Chuong nao."),
        ],
    )
    .expect_err("lo mang mot id la phai bi tu choi TRON");

    assert_eq!(err.code(), "segment.unknown_ids");
    assert_eq!(err.message_key(), MessageKey::SegmentUnknownIds);
    assert_eq!(err.retryable(), false, "mot id la khong sua duoc bang cach thu lai");

    assert_eq!(
        read_state(&opened, signed),
        ("confirmed".to_owned(), 1),
        "🔴 CHU KY PHAI CON NGUYEN — mot lo bi tu choi khong duoc de lai mot byte nao, ke ca \
         mot lan ha trang thai o giao dich dau"
    );
    assert_eq!(
        read_all_segment_rows(&opened),
        before,
        "va khong mot cot nao khac bi cham"
    );

    drop(opened);
    cleanup(&root);
}

/// 🔴 **Quyết định #7 phải bắt cả câu chỉ có khoảng trắng, không riêng chuỗi rỗng.**
///
/// `str::is_empty()` một mình cho `"   "` đi lọt: nó **không** rỗng. Hậu quả đúng bằng hậu quả
/// của ca rỗng thật mà doc-comment của `SegmentNothingToConfirm` mô tả — một `SegmentVersion`
/// gần như trống vào lịch sử FR101, rồi một cặp TM có vế đích là khoảng trắng ở Epic 7, rồi
/// FR58 điền sẵn đúng khoảng trắng đó ở một Chương sau. Hỏng **vĩnh viễn**.
#[test]
fn a_target_of_only_whitespace_is_refused_exactly_like_an_empty_one() {
    let root = temp_dir("confirm-whitespace-only");
    let opened = create_work_from_text(&root, "Toan khoang trang", "zh", "", "一。二。".to_owned())
        .expect("tao tac pham that bai");

    let rows = read_all_segment_rows(&opened);
    let chapter_id = rows[0].1;
    let target = rows[0].0;

    // Ba hinh dang khoang trang, ca ba deu KHONG rong theo `str::is_empty()`.
    // `U+00A0` la ca that: contenteditable de lai no sau mot lan go phim cach o cuoi dong.
    for blank in ["   ", "\t\n ", "\u{00a0}\u{00a0}"] {
        save_segment_targets(Some(&opened), chapter_id, &[edit(target, blank)])
            .expect("ghi van ban that bai");

        let err = confirm_segment(Some(&opened), target, "")
            .expect_err("mot cau chi co khoang trang phai bi tu choi");

        assert_eq!(err.code(), "segment.nothing_to_confirm", "voi {blank:?}");
        assert_eq!(err.message_key(), MessageKey::SegmentNothingToConfirm);
        assert_eq!(
            err.params().get("segment_id").map(String::as_str),
            Some(target.to_string().as_str())
        );

        assert_eq!(
            read_state(&opened, target),
            ("draft".to_owned(), 0),
            "🔴 KHONG mot SegmentVersion nao duoc sinh cho {blank:?}, va trang thai o nguyen"
        );
    }

    drop(opened);
    cleanup(&root);
}

// ═════════════════════════════════════════════════════════════════════════════
// Story 2.6 · FR101 — đường ĐỌC lịch sử phiên bản
// ═════════════════════════════════════════════════════════════════════════════

/// 🔴 **AC1 — mới nhất trước, và giá trị phải ĐI THEO DỮ LIỆU THẬT.**
///
/// Ca này đi qua **chính lệnh đọc của sản phẩm**, đúng khuôn và đúng lý do của
/// `the_load_command_carries_the_status_column_over_the_wire`: Story 2.6 dựng một **struct mới
/// hoàn toàn** ([`SegmentVersionRow`]), tức cùng lớp rủi ro *"khai một đằng, `SELECT` một nẻo"*
/// ở một hình dạng khác. Một trường quên trong `SELECT` cho `undefined` phía webview và **không
/// một test frontend nào bắt được**, vì fixture vitest dựng struct bằng tay.
///
/// 🔴 Ba lượt ký cho **ba** hàng, và ca này khẳng định cả **thứ tự** lẫn **nội dung** — một
/// phép kiểm chỉ đếm số hàng sẽ xanh trên một danh sách sắp ngược.
#[test]
fn the_history_command_returns_every_version_newest_first_with_the_real_text() {
    let root = temp_dir("history-newest-first");
    let opened = create_work_from_text(&root, "Ba Ban", "zh", "", "一。二。".to_owned())
        .expect("tao tac pham that bai");

    let rows = read_all_segment_rows(&opened);
    let first = rows[0].0;
    let chapter_id = rows[0].1;

    // Ba luot ky, moi luot mot van ban khac.
    //
    // 🔴 Thu tu HAI HAM la duong THAT cua san pham, khong mot chi tiet cua ca nay. Chot AC13
    // cua `confirm_segment` tra `Ok(false)` khi ky lai mot cau DA o `'confirmed'`, va
    // `save_segment_targets` CO Y khong dung toi `status` (AD-31 hang 1: auto-save khong ky
    // va khong huy chu ky). Thu ha `'confirmed'` xuong `'draft'` la
    // `unconfirm_edited_segments`, va vo `wire::save_segment_targets` goi no TRUOC -- ha-roi-ghi,
    // vi ghi-roi-ha ma sap o giua thi van ban da doi trong khi segment van `'confirmed'`, tuc
    // khong lan xac nhan nao nua xay ra (ho (2) cua AD-31 §Prevents).
    // ⚠️ Ca nay vi the phai goi CA HAI, dung thu tu do. Mot ca chi goi `save_segment_targets`
    // roi `confirm_segment` se do -- va no do vi CHINH NO sai duong, khong vi san pham hong.
    for text in ["Ban mot.", "Ban hai.", "Ban ba."] {
        unconfirm_edited_segments(Some(&opened), chapter_id, &[edit(first, text)])
            .expect("ha trang thai that bai");
        save_segment_targets(Some(&opened), chapter_id, &[edit(first, text)])
            .expect("lo ghi that bai");
        confirm_segment(Some(&opened), first, "").expect("xac nhan that bai");
    }

    let history = read_segment_history(Some(&opened), first).expect("doc lich su that bai");

    assert_eq!(
        history.len(),
        3,
        "ba luot ky tren ba van ban khac nhau phai cho DUNG ba phien ban"
    );

    let texts: Vec<&str> = history.iter().map(|v| v.target_text.as_str()).collect();
    assert_eq!(
        texts,
        vec!["Ban ba.", "Ban hai.", "Ban mot."],
        "AC1 doi MOI NHAT TRUOC. Mot danh sach sap nguoc van co ba hang va van di lot mot \
         phep kiem chi dem"
    );

    assert!(
        history.iter().all(|v| v.segment_id == first),
        "moi hang phai mang dung `segment_id` da hoi -- neu truong nay khong di theo du lieu \
         that thi mot hang cua segment KHAC se lot vao ma khong ai thay"
    );

    // `created_at` phai la ISO-8601 UTC co mili giay (AC5, ve luu).
    for v in &history {
        assert!(
            v.created_at.ends_with('Z') && v.created_at.contains('T') && v.created_at.len() == 24,
            "`created_at` phai la ISO-8601 UTC co mili giay (`YYYY-MM-DDTHH:MM:SS.sssZ`, 24 \
             ky tu) -- AC5. Doc duoc: {}",
            v.created_at
        );
    }

    let dir = opened.dir.clone();
    drop(opened);
    cleanup(&dir);
    cleanup(&root);
}

/// 🔴 **AC3 — một segment CHƯA TỪNG được xác nhận trả danh sách RỖNG, và cái rỗng đó phải
/// PHÂN BIỆT ĐƯỢC với "không tìm thấy".**
///
/// Đây là §*"Rỗng IM LẶNG bị cấm; rỗng CÓ LÝ DO thì không"* (`project-context.md:473`) ở đúng
/// hình dạng nguy hiểm nhất của nó: một `segment_id` **gõ sai** và một câu **chưa ai ký** cho
/// cùng một kết quả rỗng trong 0,01 ms, không lỗi nào ném ra, và triệu chứng là *"lịch sử không
/// hiện gì"*. Hai nhánh phải rẽ, và ca này là chỗ chứng minh chúng rẽ.
///
/// ⚠️ Vế **giao diện** của AC3 *(trạng thái rỗng phải nói ra cơ chế)* không thuộc ca này — nó
/// thuộc vitest. Tầng này chỉ chịu trách nhiệm làm cái rỗng **phân biệt được**.
#[test]
fn a_segment_never_confirmed_returns_an_empty_history_not_an_error_and_not_a_missing_segment() {
    let root = temp_dir("history-empty-vs-missing");
    let opened = create_work_from_text(&root, "Chua Ky", "zh", "", "一。二。".to_owned())
        .expect("tao tac pham that bai");

    let rows = read_all_segment_rows(&opened);
    let first = rows[0].0;
    let chapter_id = rows[0].1;

    // ① Segment CO THAT, chua tung ky ⇒ RONG, khong loi.
    let empty = read_segment_history(Some(&opened), first)
        .expect("mot segment chua tung ky KHONG duoc la mot loi -- day la rong CO LY DO");
    assert!(
        empty.is_empty(),
        "mot segment chua tung duoc xac nhan phai cho lich su RONG"
    );

    // ② Van con rong sau khi CHI GO ma khong ky -- AD-31 hang 1: auto-save KHONG tao phien ban.
    save_segment_targets(Some(&opened), chapter_id, &[edit(first, "Go ma chua ky.")])
        .expect("lo ghi that bai");
    let still_empty = read_segment_history(Some(&opened), first).expect("doc lich su that bai");
    assert!(
        still_empty.is_empty(),
        "AD-31 hang 1: auto-save KHONG tao mot `SegmentVersion` nao. Neu ca nay do, mot luot \
         go dang sinh phien ban va FR101 se day ban sao sau mot gio go"
    );

    // ③ Segment KHONG ton tai ⇒ mot loi PHAN BIET DUOC, khong mot danh sach rong.
    let missing = read_segment_history(Some(&opened), 999_999);
    let err = missing.err().expect(
        "mot `segment_id` khong ton tai PHAI la mot loi -- neu no cung cho danh sach rong thi \
         mot id go sai va mot cau chua ai ky khong phan biet duoc, va do la RONG IM LANG",
    );
    assert_eq!(
        err.message_key(),
        MessageKey::SegmentNotFound,
        "phep tu choi phai la `segment.not_found`, phan biet duoc"
    );

    let dir = opened.dir.clone();
    drop(opened);
    cleanup(&dir);
    cleanup(&root);
}

/// 🔴 **AC4 — một segment ĐÃ VỀ HƯU vẫn tra được lịch sử, và đường ĐỌC KHÔNG từ chối nó.**
///
/// Ba lệnh **ghi** (`confirm_segment` · `set_segment_omitted` · `set_segment_paragraph_end`)
/// đều trả `MessageKey::SegmentRetired` khi `retired_at` khác `NULL`, và chúng **đúng** — ghi
/// lên một tombstone là sửa lịch sử. Lượt này đọc, nên nó phải đi ngược lại. Ca này tồn tại
/// để một lượt "cho nhất quán" ở story sau **đỏ** thay vì đi lọt.
///
/// ⚠️ Trạng thái về hưu dựng bằng **SQL trực tiếp** — đó là khuôn Story 2.5 đã dùng, và hôm
/// nay nó là đường **duy nhất**: `retired_at` là `None` cho mọi segment, `merge_segment` cho
/// **0 đường mã**, và Story 2.8 *(gộp/tách tường minh)* là `backlog`.
/// 🔴 Vế **bề mặt vào** vì thế **không** đối chứng được ở story này — AC4 đóng một nửa 🟡, ghi
/// nợ có chủ **Story 2.8**. Không tự chấm đạt.
#[test]
fn a_retired_segment_still_hands_back_its_full_history_because_reading_is_not_writing() {
    let root = temp_dir("history-retired");
    let opened = create_work_from_text(&root, "Ve Huu", "zh", "", "一。二。".to_owned())
        .expect("tao tac pham that bai");

    let rows = read_all_segment_rows(&opened);
    let first = rows[0].0;
    let chapter_id = rows[0].1;

    // Hai luot ky TRUOC khi cho no ve huu. Ha-roi-ghi-roi-ky, dung duong cua vo `wire`.
    for text in ["Ban mot.", "Ban hai."] {
        unconfirm_edited_segments(Some(&opened), chapter_id, &[edit(first, text)])
            .expect("ha trang thai that bai");
        save_segment_targets(Some(&opened), chapter_id, &[edit(first, text)])
            .expect("lo ghi that bai");
        confirm_segment(Some(&opened), first, "").expect("xac nhan that bai");
    }

    // Cho ve huu bang SQL truc tiep -- duong DUY NHAT hom nay.
    opened
        .store
        .write(move |tx: &Transaction<'_>| {
            tx.execute(
                "UPDATE segment SET retired_at = '2026-08-16T12:00:00.000Z' WHERE id = ?1",
                [first],
            )?;
            Ok(())
        })
        .expect("cho segment ve huu that bai");

    // Duong GHI tu choi -- day la doi chung, khong phai muc tieu cua ca nay.
    let write_refused = confirm_segment(Some(&opened), first, "");
    assert_eq!(
        write_refused
            .err()
            .expect("mot lenh GHI phai tu choi mot segment da ve huu")
            .message_key(),
        MessageKey::SegmentRetired,
        "duong GHI phai tu choi -- neu ca nay do thi doi chung ben duoi khong con nghia gi"
    );

    // Va duong DOC thi KHONG.
    let history = read_segment_history(Some(&opened), first).expect(
        "AC4: lich su cua mot segment DA VE HUU phai tra lai duoc. Mot phep tu choi o day la \
         mot luot chep hang rao cua ba lenh GHI sang mot duong DOC",
    );
    assert_eq!(
        history.len(),
        2,
        "lich su phai con DU hai phien ban -- ve huu la mot tombstone (AD-5), khong mot luot \
         xoa, va `segment_version` co y KHONG co `ON DELETE CASCADE`"
    );
    assert_eq!(
        history[0].target_text, "Ban hai.",
        "va no van sap moi nhat truoc"
    );

    let dir = opened.dir.clone();
    drop(opened);
    cleanup(&dir);
    cleanup(&root);
}

/// 🔴 **Lịch sử của một segment KHÔNG được lẫn hàng của segment khác.**
///
/// Khuôn `a_segment_id_from_another_chapter_is_refused_and_never_crosses_over`. Mệnh đề mỏng
/// và đúng lý do đó nó cần một ca: mệnh đề `WHERE segment_id = ?1` là **một dòng**, và một
/// lượt sửa truy vấn sau này *(thêm một `JOIN`, đổi một tên cột)* làm nó rơi mà **không** ca
/// nào khác đỏ — cả ba ca trên đều chỉ dùng **một** segment.
#[test]
fn the_history_of_one_segment_never_carries_a_row_belonging_to_another() {
    let root = temp_dir("history-no-crossover");
    let opened = create_work_from_text(&root, "Hai Cau", "zh", "", "一。二。".to_owned())
        .expect("tao tac pham that bai");

    let rows = read_all_segment_rows(&opened);
    let (first, second) = (rows[0].0, rows[1].0);
    let chapter_id = rows[0].1;

    // Cau MOT ky hai lan, cau HAI ky mot lan. Ha-roi-ghi-roi-ky, dung duong cua vo `wire`.
    for text in ["A mot.", "A hai."] {
        unconfirm_edited_segments(Some(&opened), chapter_id, &[edit(first, text)])
            .expect("ha trang thai that bai");
        save_segment_targets(Some(&opened), chapter_id, &[edit(first, text)])
            .expect("lo ghi that bai");
        confirm_segment(Some(&opened), first, "").expect("xac nhan that bai");
    }
    save_segment_targets(Some(&opened), chapter_id, &[edit(second, "B mot.")])
        .expect("lo ghi that bai");
    confirm_segment(Some(&opened), second, "").expect("xac nhan that bai");

    let a = read_segment_history(Some(&opened), first).expect("doc lich su cau mot that bai");
    let b = read_segment_history(Some(&opened), second).expect("doc lich su cau hai that bai");

    assert_eq!(a.len(), 2, "cau mot phai co dung hai phien ban");
    assert_eq!(b.len(), 1, "cau hai phai co dung mot phien ban");
    assert!(
        a.iter().all(|v| v.segment_id == first),
        "lich su cua cau mot mang mot hang cua cau khac"
    );
    assert_eq!(
        b[0].target_text, "B mot.",
        "lich su cua cau hai phai la van ban cua chinh no"
    );

    let dir = opened.dir.clone();
    drop(opened);
    cleanup(&dir);
    cleanup(&root);
}

/// 🔴 **Hai phiên bản trùng ĐÚNG mili giây vẫn có một thứ tự TẤT ĐỊNH — và vế `id DESC` là
/// thứ mua được điều đó.**
///
/// ⚠️ **Đo 2026-08-16, và con số này là lý do ca tồn tại:** mười hai lượt ký liên tiếp trên
/// cùng một segment cho **11 mốc `created_at` khác nhau** — hai lượt rơi trúng cùng một mili
/// giây (`…55.856Z`). `strftime('%Y-%m-%dT%H:%M:%fZ','now')` chính xác tới mili giây, và một
/// lượt ký mất **~1 ms**, nên va chạm không phải một khả năng lý thuyết: nó xảy ra trong vòng
/// mười hai lượt.
///
/// 🔴 SQLite **không bảo đảm** thứ tự của các hàng bằng nhau ở cột sắp. ⇒ Thiếu vế `id DESC`,
/// thứ tự hiển thị của hai hàng đó là **không tất định** — đổi giữa hai lượt đọc, trên cùng
/// một dữ liệu, và AC1 *(mới nhất trước)* nói sai ở đúng cặp hàng đó.
///
/// ⚠️ Ca này dựng va chạm bằng **SQL trực tiếp** thay vì ký thật mười hai lượt. Cố ý: một ca
/// dựa vào việc đồng hồ *tình cờ* va chạm là một ca **chập chờn** — nó xanh giả ở đa số lượt
/// chạy và chỉ đỏ khi máy đủ nhanh. Đã đo: bốn ca đọc kia chạy **8/8 xanh** ngay cả khi vế
/// `id DESC` bị gỡ, tức chúng **không** canh mệnh đề này. Va chạm phải được **dựng**, không
/// được **chờ**.
#[test]
fn two_versions_sharing_a_millisecond_still_come_back_in_a_deterministic_order() {
    let root = temp_dir("history-same-ms");
    let opened = create_work_from_text(&root, "Trung Mili", "zh", "", "一。二。".to_owned())
        .expect("tao tac pham that bai");

    let rows = read_all_segment_rows(&opened);
    let first = rows[0].0;

    // Ba hang CUNG mot moc, id tang dan. `id` la AUTOINCREMENT nen no la thu tu ky THAT.
    opened
        .store
        .write(move |tx: &Transaction<'_>| {
            for (n, text) in ["som nhat", "giua", "muon nhat"].iter().enumerate() {
                tx.execute(
                    "INSERT INTO segment_version (segment_id, target_text, created_at) \
                     VALUES (?1, ?2, '2026-08-16T02:10:55.856Z')",
                    (first, format!("{text} ({n})")),
                )?;
            }
            Ok(())
        })
        .expect("bom ba hang trung moc that bai");

    let history = read_segment_history(Some(&opened), first).expect("doc lich su that bai");

    let texts: Vec<&str> = history.iter().map(|v| v.target_text.as_str()).collect();
    assert_eq!(
        texts,
        vec!["muon nhat (2)", "giua (1)", "som nhat (0)"],
        "ba hang trung DUNG mot mili giay phai ra theo `id` GIAM DAN -- `id` la AUTOINCREMENT \
         nen no la thu tu ky that, va no la thu duy nhat go hoa duoc. Thieu ve `id DESC`, \
         SQLite khong bao dam thu tu cua cac hang bang nhau va AC1 noi sai o dung cap hang do"
    );

    let dir = opened.dir.clone();
    drop(opened);
    cleanup(&dir);
    cleanup(&root);
}

// ═════════════════════════════════════════════════════════════════════════════
// Story 2.6 · AC2 — đường KHÔI PHỤC
// ═════════════════════════════════════════════════════════════════════════════

/// Dựng một segment đã ký `texts.len()` lượt, trả `(segment_id, chapter_id)`.
///
/// Đi qua **đúng đường của sản phẩm**: hạ → ghi → ký. Xem doc-comment của
/// `unconfirm_edited_segments` — vỏ `wire::save_segment_targets` gọi hàm hạ **trước**, và
/// thứ tự đó là một quyết định về mất mát dữ liệu, không một chi tiết.
fn sign_repeatedly(
    opened: &auratranslate_lib::commands::project::OpenWork,
    texts: &[&str],
) -> (i64, i64) {
    let rows = read_all_segment_rows(opened);
    let (id, chapter_id) = (rows[0].0, rows[0].1);
    for text in texts {
        unconfirm_edited_segments(Some(opened), chapter_id, &[edit(id, text)])
            .expect("ha trang thai that bai");
        save_segment_targets(Some(opened), chapter_id, &[edit(id, text)]).expect("lo ghi that bai");
        confirm_segment(Some(opened), id, "").expect("xac nhan that bai");
    }
    (id, chapter_id)
}

/// 🔴 **AC2 — khôi phục đặt lại `target_text` VÀ hạ `status` về `'draft'`, và lịch sử KHÔNG
/// dài thêm một hàng nào (Quyết định #1 đường (a)).**
///
/// Vế thứ hai là vế mang chữ ký. Mockup viết đậm *"Khôi phục là tạo phiên bản mới… đẩy nó lên
/// thành phiên bản thứ sáu"*, còn bảng Rule của AD-31 có đúng **sáu hàng và không hàng nào là
/// "khôi phục"**. Ice ký AD-31. ⇒ Ca này khoá mệnh đề đó lại: nếu một story sau thêm một
/// `INSERT` vào đường khôi phục *(tức hàng thứ bảy của AD-31)*, ca này **đỏ** và buộc lượt đó
/// đi qua thủ tục viết một `AD` mới thay vì một dòng mã tiện tay.
///
/// ⚠️ Lời hứa *"lịch sử chỉ dài thêm"* vẫn giữ — ca này khẳng định luôn nửa đó: **lượt xác
/// nhận kế tiếp** mới sinh hàng thứ tư, do hàng 2 của AD-31.
#[test]
fn restoring_rewrites_the_target_and_drops_the_status_without_growing_the_history() {
    let root = temp_dir("restore-basic");
    let opened = create_work_from_text(&root, "Khoi Phuc", "zh", "", "一。二。".to_owned())
        .expect("tao tac pham that bai");

    let (id, chapter_id) = sign_repeatedly(&opened, &["Ban mot.", "Ban hai.", "Ban ba."]);

    let before = read_segment_history(Some(&opened), id).expect("doc lich su that bai");
    assert_eq!(before.len(), 3, "ba luot ky phai cho ba phien ban");

    // Phien ban CU NHAT (`Ban mot.`) la hang cuoi cua danh sach moi-nhat-truoc.
    let oldest = before.last().expect("phai co mot phien ban cu nhat");
    assert_eq!(oldest.target_text, "Ban mot.");

    let outcome = restore_segment_version(Some(&opened), id, oldest.id, false)
        .expect("khoi phuc that bai");

    assert!(outcome.restored, "luot nay PHAI ghi");
    assert!(
        !outcome.needs_confirmation,
        "van ban hien tai (`Ban ba.`) DA duoc ky nen no co ban sao trong `segment_version` \
         ⇒ khong co gi de mat ⇒ khong hoi lai"
    );
    assert_eq!(
        outcome.status, "draft",
        "AC2 noi thang: trang thai segment ve CHUA XAC NHAN"
    );

    // Doc lai tu DIA qua chinh lenh cua san pham.
    let loaded = read_open_chapter_segments(Some(&opened)).expect("nap lai segment that bai");
    let seg = loaded
        .segments
        .iter()
        .find(|s| s.id == id)
        .expect("khong thay segment vua khoi phuc");
    assert_eq!(
        seg.target_text, "Ban mot.",
        "van ban dich phai quay ve dung noi dung cua phien ban da chon"
    );
    assert_eq!(
        seg.status, "draft",
        "va trang thai tren DIA phai la `'draft'` -- mot `RestoreOutcome` noi `draft` trong \
         khi dia van `confirmed` la dung lop loi \"khai mot dang, ghi mot neo\""
    );

    // 🔴 Lich su KHONG dai them -- chu ky #1(a).
    let after = read_segment_history(Some(&opened), id).expect("doc lai lich su that bai");
    assert_eq!(
        after.len(),
        3,
        "khoi phuc KHONG duoc `INSERT` mot hang `segment_version` nao (Quyet dinh #1 duong \
         (a), Ice ky 2026-08-16). Bang Rule cua AD-31 co dung SAU hang va khong hang nao la \
         \"khoi phuc\"; mot hang thu bay phai di qua thu tuc viet mot `AD` moi"
    );
    assert_eq!(
        after, before,
        "va khong hang cu nao bi dung toi -- \"lich su chi dai them, khong bao gio ngan di\""
    );

    // Va no dai them o LUOT XAC NHAN KE TIEP, dung nhu loi hua cua mockup -- chi muon mot nhip.
    confirm_segment(Some(&opened), id, "").expect("xac nhan lai that bai");
    let grown = read_segment_history(Some(&opened), id).expect("doc lai lich su that bai");
    assert_eq!(
        grown.len(),
        4,
        "phien ban \"thu sau\" cua mockup sinh o luot XAC NHAN ke tiep, do hang 2 cua AD-31 -- \
         loi hua \"lich su chi dai them\" duoc giu, chi muon hon mot nhip"
    );
    assert_eq!(grown[0].target_text, "Ban mot.");
    let _ = chapter_id;

    let dir = opened.dir.clone();
    drop(opened);
    cleanup(&dir);
    cleanup(&root);
}

/// 🔴 **Chốt chống mất bản nháp CHƯA TỪNG ĐƯỢC KÝ — chữ ký #2(a), và không AC nào nêu nó.**
///
/// Một hàng `segment_version` chỉ sinh ở **đúng một chỗ** (trong `confirm_segment`), nên văn
/// bản `'draft'` có nội dung mà chưa ai ký **không có một bản sao nào ở bất cứ đâu**. Khôi
/// phục lên nó xoá vĩnh viễn.
///
/// Ca này khẳng định **ba** mệnh đề, và mệnh đề giữa là mệnh đề đắt:
/// ① lượt gọi đầu (`force = false`) **giữ lại** lượt ghi và mang bản nháp đó ra;
/// ② **không một byte nào** đã được ghi — đĩa còn nguyên;
/// ③ lượt gọi thứ hai (`force = true`) ghi thật.
#[test]
fn restoring_over_an_unsigned_draft_holds_the_write_until_the_caller_confirms() {
    let root = temp_dir("restore-holds");
    let opened = create_work_from_text(&root, "Giu Lai", "zh", "", "一。二。".to_owned())
        .expect("tao tac pham that bai");

    let (id, chapter_id) = sign_repeatedly(&opened, &["Ban da ky."]);

    // Go them mot ban nhap va KHONG ky -- day la van ban khong co ban sao nao.
    unconfirm_edited_segments(Some(&opened), chapter_id, &[edit(id, "Ban nhap chua ai ky.")])
        .expect("ha trang thai that bai");
    save_segment_targets(Some(&opened), chapter_id, &[edit(id, "Ban nhap chua ai ky.")])
        .expect("lo ghi that bai");

    let history = read_segment_history(Some(&opened), id).expect("doc lich su that bai");
    assert_eq!(history.len(), 1, "chi mot luot ky ⇒ mot phien ban");
    let target = &history[0];

    // ① Luot goi dau: GIU LAI.
    let held = restore_segment_version(Some(&opened), id, target.id, false)
        .expect("mot luot giu lai KHONG phai mot loi -- no la mot ket qua");
    assert!(
        held.needs_confirmation,
        "van ban hien tai chua tung duoc ky va khac ban se khoi phuc ⇒ PHAI hoi lai. \
         Chu ky #2(a) cua Ice, 2026-08-16"
    );
    assert!(!held.restored, "va no KHONG duoc ghi mot byte nao");
    assert_eq!(
        held.unsigned_draft.as_deref(),
        Some("Ban nhap chua ai ky."),
        "ban nhap sap mat phai di ra ngoai de webview HIEN NO RA, khong chi noi \"co thu se mat\""
    );

    // ② Dia con NGUYEN.
    let untouched = read_open_chapter_segments(Some(&opened)).expect("nap lai segment that bai");
    let seg = untouched
        .segments
        .iter()
        .find(|s| s.id == id)
        .expect("khong thay segment");
    assert_eq!(
        seg.target_text, "Ban nhap chua ai ky.",
        "mot luot GIU LAI khong duoc dung toi mot byte nao cua dia"
    );

    // ③ Luot goi thu hai voi `force`: ghi that.
    let forced =
        restore_segment_version(Some(&opened), id, target.id, true).expect("khoi phuc that bai");
    assert!(forced.restored, "voi `force` thi no phai ghi");
    assert!(!forced.needs_confirmation);
    let after = read_open_chapter_segments(Some(&opened)).expect("nap lai segment that bai");
    assert_eq!(
        after
            .segments
            .iter()
            .find(|s| s.id == id)
            .expect("khong thay segment")
            .target_text,
        "Ban da ky."
    );

    let dir = opened.dir.clone();
    drop(opened);
    cleanup(&dir);
    cleanup(&root);
}

/// 🔴 **Phép so là "văn bản này có bản sao không", KHÔNG một cờ `dirty` — hợp đồng phụ AD-31.**
///
/// Ca phân biệt hai cách: người dùng gõ một thứ khác rồi **hoàn tác về đúng một bản đã ký**.
/// - Cờ `dirty` nói *"đã sửa"* ⇒ hỏi lại ⇒ **hỏi thừa**, và một hộp thoại hỏi thừa là thứ làm
///   người dùng bấm "đồng ý" theo phản xạ, tức làm chốt thật mất tác dụng.
/// - Phép so văn bản nói *"cái sắp mất có bản sao trong `segment_version`"* ⇒ không hỏi.
///
/// ⚠️ Đây là mệnh đề mà `AND target_text <> ?3` của `unconfirm_edited_segments` đã khoá cho
/// đường xác nhận; ca này khoá nó cho đường khôi phục.
#[test]
fn text_that_still_has_a_copy_in_the_history_does_not_trigger_the_confirmation_hold() {
    let root = temp_dir("restore-has-copy");
    let opened = create_work_from_text(&root, "Co Ban Sao", "zh", "", "一。二。".to_owned())
        .expect("tao tac pham that bai");

    let (id, chapter_id) = sign_repeatedly(&opened, &["Ban mot.", "Ban hai."]);

    // Nguoi dung go ve DUNG `Ban mot.` -- mot van ban DA tung duoc ky, tuc CO ban sao.
    unconfirm_edited_segments(Some(&opened), chapter_id, &[edit(id, "Ban mot.")])
        .expect("ha trang thai that bai");
    save_segment_targets(Some(&opened), chapter_id, &[edit(id, "Ban mot.")])
        .expect("lo ghi that bai");

    let history = read_segment_history(Some(&opened), id).expect("doc lich su that bai");
    let newest = &history[0];
    assert_eq!(newest.target_text, "Ban hai.");

    let outcome =
        restore_segment_version(Some(&opened), id, newest.id, false).expect("khoi phuc that bai");

    assert!(
        !outcome.needs_confirmation,
        "van ban hien tai (`Ban mot.`) CO mot ban sao trong `segment_version` ⇒ khong mat gi \
         ⇒ KHONG duoc hoi lai. Mot co `dirty` se hoi o day, va do la cho hai cach re nhau"
    );
    assert!(outcome.restored, "va no ghi thang");

    let dir = opened.dir.clone();
    drop(opened);
    cleanup(&dir);
    cleanup(&root);
}

/// 🔴 **Khôi phục về ĐÚNG nội dung đang có là VÔ HẠI: không ghi gì, và KHÔNG hạ chữ ký.**
///
/// Khuôn AC13 của `confirm_segment` *(ký lại một câu đã ký ⇒ `Ok(false)`, không hàng mới)*.
/// Vế **không hạ `status`** là vế quan trọng hơn: hạ một chữ ký mà không đổi lấy gì là huỷ
/// công của người dùng — họ phải ký lại một câu chưa hề đổi.
///
/// ⚠️ Và `restored = false` ở đây phải **phân biệt được** với `needs_confirmation = true`:
/// một cái là *"không có gì để làm"*, cái kia là *"đang chờ bạn"*.
#[test]
fn restoring_to_the_text_already_in_place_writes_nothing_and_keeps_the_signature() {
    let root = temp_dir("restore-noop");
    let opened = create_work_from_text(&root, "Vo Hai", "zh", "", "一。二。".to_owned())
        .expect("tao tac pham that bai");

    let (id, _) = sign_repeatedly(&opened, &["Ban mot.", "Ban hai."]);

    // Segment dang o `confirmed` voi `Ban hai.`; phien ban moi nhat cung la `Ban hai.`
    let history = read_segment_history(Some(&opened), id).expect("doc lich su that bai");
    let newest = &history[0];
    assert_eq!(newest.target_text, "Ban hai.");

    let outcome =
        restore_segment_version(Some(&opened), id, newest.id, false).expect("khoi phuc that bai");

    assert!(
        !outcome.restored,
        "khoi phuc ve dung noi dung dang co KHONG duoc ghi mot byte nao -- khuon AC13"
    );
    assert!(
        !outcome.needs_confirmation,
        "va no KHONG phai mot luot cho nguoi dung tra loi -- hai trang thai nay phai phan biet duoc"
    );
    assert_eq!(
        outcome.status, "confirmed",
        "🔴 chu ky cua nguoi dung PHAI duoc giu. Ha `confirmed` xuong `draft` vi ho bam khoi \
         phuc len chinh ban dang dung la huy cong ma khong doi lay gi -- ho phai ky lai mot \
         cau chua he doi"
    );

    let loaded = read_open_chapter_segments(Some(&opened)).expect("nap lai segment that bai");
    assert_eq!(
        loaded
            .segments
            .iter()
            .find(|s| s.id == id)
            .expect("khong thay segment")
            .status,
        "confirmed",
        "va tren DIA no cung phai con `confirmed`"
    );

    let dir = opened.dir.clone();
    drop(opened);
    cleanup(&dir);
    cleanup(&root);
}

/// 🔴 **Một `version_id` THUỘC SEGMENT KHÁC bị từ chối, và KHÔNG ghi gì.**
///
/// Khuôn `a_segment_id_from_another_chapter_is_refused_and_never_crosses_over`. Nếu hàng rào
/// `AND segment_id = ?2` rơi, lượt khôi phục ghi văn bản của **câu khác** vào câu này — hỏng
/// âm thầm, và người dùng không có đường nào lần ra: cả hai đều là câu tiếng Việt hợp lệ.
#[test]
fn a_version_belonging_to_another_segment_is_refused_and_never_crosses_over() {
    let root = temp_dir("restore-crossover");
    let opened = create_work_from_text(&root, "Cheo Nhau", "zh", "", "一。二。".to_owned())
        .expect("tao tac pham that bai");

    let rows = read_all_segment_rows(&opened);
    let (first, second) = (rows[0].0, rows[1].0);
    let chapter_id = rows[0].1;

    for (id, text) in [(first, "Cua cau MOT."), (second, "Cua cau HAI.")] {
        unconfirm_edited_segments(Some(&opened), chapter_id, &[edit(id, text)])
            .expect("ha trang thai that bai");
        save_segment_targets(Some(&opened), chapter_id, &[edit(id, text)])
            .expect("lo ghi that bai");
        confirm_segment(Some(&opened), id, "").expect("xac nhan that bai");
    }

    // Phien ban cua cau HAI, dem ap len cau MOT.
    let b_history = read_segment_history(Some(&opened), second).expect("doc lich su that bai");
    let b_version = b_history[0].id;

    let refused = restore_segment_version(Some(&opened), first, b_version, false);
    let err = refused.err().expect(
        "mot `version_id` thuoc segment KHAC phai bi tu choi -- neu no di lot, luot khoi phuc \
         ghi van ban cua cau khac vao cau nay va khong ai lan ra duoc",
    );
    assert_eq!(
        err.message_key(),
        MessageKey::SegmentNotFound,
        "phep tu choi phai phan biet duoc"
    );

    // Va KHONG ghi gi.
    let loaded = read_open_chapter_segments(Some(&opened)).expect("nap lai segment that bai");
    assert_eq!(
        loaded
            .segments
            .iter()
            .find(|s| s.id == first)
            .expect("khong thay cau mot")
            .target_text,
        "Cua cau MOT.",
        "mot luot tu choi KHONG duoc dung toi mot byte nao"
    );

    // 🔴 Ke ca voi `force` -- `force` chi bo qua CHOT CHONG MAT BAN NHAP, no khong bo qua
    //    hang rao quyen so huu.
    let refused_forced = restore_segment_version(Some(&opened), first, b_version, true);
    assert!(
        refused_forced.is_err(),
        "`force` chi bo qua chot chong mat ban nhap (chu ky #2(a)). No KHONG duoc bo qua hang \
         rao \"phien ban phai thuoc segment nay\" -- gop hai thu do lam mot la bien mot loi \
         xac nhan cua nguoi dung thanh mot giay phep ghi bat ky dau"
    );

    let dir = opened.dir.clone();
    drop(opened);
    cleanup(&dir);
    cleanup(&root);
}

/// 🔴 **Đường KHÔI PHỤC từ chối một segment đã về hưu — ngược với đường ĐỌC.**
///
/// Cặp đôi với `a_retired_segment_still_hands_back_its_full_history_because_reading_is_not_writing`.
/// Hai ca này nói **hai nửa của cùng một mệnh đề**, và chúng phải cùng tồn tại: một lượt
/// "cho nhất quán" ở story sau sẽ làm **một trong hai** đỏ, dù nó đi theo chiều nào.
#[test]
fn restoring_onto_a_retired_segment_is_refused_because_writing_is_not_reading() {
    let root = temp_dir("restore-retired");
    let opened = create_work_from_text(&root, "Ve Huu Ghi", "zh", "", "一。二。".to_owned())
        .expect("tao tac pham that bai");

    let (id, _) = sign_repeatedly(&opened, &["Ban mot.", "Ban hai."]);
    let history = read_segment_history(Some(&opened), id).expect("doc lich su that bai");
    let oldest = history.last().expect("phai co phien ban cu nhat").id;

    opened
        .store
        .write(move |tx: &Transaction<'_>| {
            tx.execute(
                "UPDATE segment SET retired_at = '2026-08-16T12:00:00.000Z' WHERE id = ?1",
                [id],
            )?;
            Ok(())
        })
        .expect("cho segment ve huu that bai");

    let refused = restore_segment_version(Some(&opened), id, oldest, false);
    assert_eq!(
        refused
            .err()
            .expect("khoi phuc len mot tombstone phai bi tu choi")
            .message_key(),
        MessageKey::SegmentRetired,
        "duong KHOI PHUC GHI, nen no tu choi mot segment da ve huu (AD-5). Duong DOC thi \
         nguoc lai -- xem ca `a_retired_segment_still_hands_back_its_full_history_...`"
    );

    // Va lich su van doc duoc -- hai nua cua cung mot menh de, khang dinh canh nhau.
    let still = read_segment_history(Some(&opened), id).expect("AC4: doc van phai duoc");
    assert_eq!(still.len(), 2);

    let dir = opened.dir.clone();
    drop(opened);
    cleanup(&dir);
    cleanup(&root);
}

// ═════════════════════════════════════════════════════════════════════════════
// Story 2.7 — XUẤT XỨ BẢN DỊCH CẤP SEGMENT (FR117 · AD-31 · AD-47)
// ═════════════════════════════════════════════════════════════════════════════
//
// ⚠️ **Vì sao mọi ca dưới đây sống ở đường Rust, không ở vitest** — bảng §Testing của story:
// phép phân xử xuất xứ là một **quy tắc nghiệp vụ** (AD-1) và nó chạy trong một **giao dịch**.
// `happy-dom` không có giao dịch nào để quan sát, và một fixture chép tay ở vitest luôn có sẵn
// mọi trường — đúng lớp lỗi đã cho 74/74 xanh trên một sản phẩm hỏng ở Story 2.5.
//
// 🔴 **Tham số thứ ba của `confirm_segment` là MỐC, và ở mọi ca cũ nó là `""`** — đó là sự
// thật của các fixture đó chứ không một giá trị mồi: `create_work_from_text` tách segment từ
// văn bản **nguồn**, nên bản dịch lúc nạp là chuỗi rỗng.

/// Xuất xứ đang nằm trên đĩa của một segment.
fn read_origin(open: &auratranslate_lib::commands::project::OpenWork, id: i64) -> String {
    open.store
        .read(move |conn| {
            conn.query_row(
                "SELECT translation_origin FROM segment WHERE id = ?1",
                [id],
                |r| r.get(0),
            )
        })
        .expect("doc xuat xu that bai")
}

/// 🔴 **Danh mục xuất xứ là ĐÓNG — AD-47 ⑥.**
///
/// Đây là **cổng** của một lượt nới đặc tả, không một phép kiểm hình thức. AD-47 ⑥ đặt hai
/// điều kiện cho mọi giá trị thêm vào: nó phải khai nó rơi về vế nào của **trục nhị phân
/// FR118**, và vì tập giá trị nằm trên **đĩa người dùng** nên lượt nới là **một bước di trú
/// nữa**. Cả hai điều kiện đó là công việc của một `AD`, không của một dòng mã — nên một giá
/// trị thứ năm lặng lẽ thêm vào phải làm một thứ gì đó **đỏ**, và đây là thứ đó.
///
/// Chạy đỏ-rồi-xanh: thêm một phần tử vào `TRANSLATION_ORIGINS`, ca này phải ĐỎ.
#[test]
fn the_translation_origin_catalogue_matches_ad_47_row_by_row() {
    use auratranslate_lib::commands::segment::TRANSLATION_ORIGINS;

    assert_eq!(
        TRANSLATION_ORIGINS.len(),
        4,
        "danh muc xuat xu phai co DUNG BON phan tu -- ba gia tri cua FR117 cong `''` \
         (chua co ban dich). Mot gia tri thu nam la mot luot NOI FR117: no doi mot `AD` moi \
         VA mot buoc di tru (AD-47 ⑥), khong mot dong ma"
    );
    assert_eq!(
        TRANSLATION_ORIGINS,
        ["", "self", "other", "bilingual_import"],
        "bon gia tri tren DIA, nguyen van. Doi mot cai ten o day la doi du lieu cua moi \
         `.atproj` da ton tai -- tuc mot buoc di tru, khong mot luot doi ten"
    );
}

/// 🔴 **Câu `UPDATE` backfill của bước 11 chép nguyên văn một hằng — và bản sao đó có lưới.**
///
/// `Migration::sql` là `&'static str` và `concat!` chỉ nhận **literal**, nên `'self'` trong
/// [`SEGMENT_TRANSLATION_ORIGIN_DDL`] **không** trỏ về được `TRANSLATION_ORIGIN_SELF`. Bản sao
/// là bắt buộc; thứ **không** bắt buộc là để nó trôi. Đổi hằng mà quên câu SQL cho ra một cột
/// mang hai từ vựng cho cùng một khái niệm, và mọi hàng backfill sẽ rơi ra ngoài danh mục đóng
/// **mà không ca nào khác đỏ** — `is_omitted`/`status` không có lớp lỗi này vì DDL của chúng
/// không nhắc tới giá trị nào ngoài `DEFAULT`.
///
/// Chạy đỏ-rồi-xanh: đổi `TRANSLATION_ORIGIN_SELF` thành `"mine"`, ca này phải ĐỎ.
#[test]
fn the_backfill_literal_matches_the_origin_constant_it_copies() {
    use auratranslate_lib::commands::segment::TRANSLATION_ORIGIN_SELF;
    use auratranslate_lib::core::store::SEGMENT_TRANSLATION_ORIGIN_DDL;

    let expected = format!("translation_origin = '{TRANSLATION_ORIGIN_SELF}'");
    assert!(
        SEGMENT_TRANSLATION_ORIGIN_DDL.contains(&expected),
        "cau backfill cua buoc 11 phai chep DUNG `TRANSLATION_ORIGIN_SELF`. Doc duoc: {SEGMENT_TRANSLATION_ORIGIN_DDL}"
    );
}

/// **AC1 — gõ bản dịch rồi xác nhận ⇒ *tôi dịch*.**
#[test]
fn confirming_text_the_user_typed_records_it_as_their_own() {
    use auratranslate_lib::commands::segment::TRANSLATION_ORIGIN_SELF;

    let root = temp_dir("origin-typed");
    let opened = create_work_from_text(&root, "Go moi", "zh", "", "一。二。".to_owned())
        .expect("tao tac pham that bai");

    let rows = read_all_segment_rows(&opened);
    let (id, chapter_id) = (rows[0].0, rows[0].1);

    // Truoc luot go: chua co ban dich ⇒ chua co xuat xu nao.
    assert_eq!(
        read_origin(&opened, id),
        "",
        "mot segment vua tach ra tu van ban NGUON chua co ban dich, nen no chua co xuat xu"
    );

    save_segment_targets(Some(&opened), chapter_id, &[edit(id, "Chu cua toi.")])
        .expect("lo ghi that bai");
    // Moc luc nap la chuoi rong -- day la mot Chuong vua nhap.
    confirm_segment(Some(&opened), id, "").expect("xac nhan that bai");

    assert_eq!(
        read_origin(&opened, id),
        TRANSLATION_ORIGIN_SELF,
        "van ban KHAC moc luc nap ⇒ chu cua nguoi dung ⇒ `self` (FR117 hang 1)"
    );

    let dir = opened.dir.clone();
    drop(opened);
    cleanup(&dir);
    cleanup(&root);
}

/// **AC2 — sửa một câu đến từ nơi khác rồi xác nhận ⇒ *tôi dịch*.**
///
/// ⚠️ Fixture dựng bằng **SQL trực tiếp**, và đó là một quyết định có tiền lệ *(chữ ký #8(a)
/// của Story 2.6 cho `retired_at`)*: hôm nay **không đường sản phẩm nào** sinh ra một xuất xứ
/// khác mặc định — FR115 là Epic 6, FR58 là Epic 7, AI là Epic 4, FR94 là Epic 8. Một ca đợi
/// đường sản phẩm là một ca không chạy được cho tới Epic 6.
#[test]
fn confirming_an_edit_of_a_sentence_that_came_from_elsewhere_claims_it_as_their_own() {
    use auratranslate_lib::commands::segment::{TRANSLATION_ORIGIN_OTHER, TRANSLATION_ORIGIN_SELF};

    let root = temp_dir("origin-edited");
    let opened = create_work_from_text(&root, "Sua lai", "zh", "", "一。二。".to_owned())
        .expect("tao tac pham that bai");

    let rows = read_all_segment_rows(&opened);
    let (id, chapter_id) = (rows[0].0, rows[0].1);

    // Mot cau "san co": van ban VA xuat xu do mot co che khac dat vao (AD-47 ①(a)+(b)).
    opened
        .store
        .write(move |tx: &Transaction<'_>| {
            tx.execute(
                "UPDATE segment SET target_text = ?1, translation_origin = ?2 WHERE id = ?3",
                ("Nguoi khac dich.", TRANSLATION_ORIGIN_OTHER, id),
            )?;
            Ok(())
        })
        .expect("dung fixture that bai");

    // Nguoi dung SUA no roi xac nhan. Moc la van ban luc nap.
    save_segment_targets(Some(&opened), chapter_id, &[edit(id, "Toi sua lai roi.")])
        .expect("lo ghi that bai");
    confirm_segment(Some(&opened), id, "Nguoi khac dich.").expect("xac nhan that bai");

    assert_eq!(
        read_origin(&opened, id),
        TRANSLATION_ORIGIN_SELF,
        "cau SAU KHI SUA la chu cua nguoi dung -- FR117 hang 2, nguyen van"
    );

    let dir = opened.dir.clone();
    drop(opened);
    cleanup(&dir);
    cleanup(&root);
}

/// **AC3 — duyệt NGUYÊN VĂN một câu sẵn có ⇒ giữ nguyên xuất xứ lúc nạp.**
///
/// 🔴 Đây là ca mà **hình dạng lược đồ** được nghiệm thu, không chỉ phép phân xử: nó đọc một
/// xuất xứ **lúc nạp**, tức trước bất kỳ lượt xác nhận nào — và đó chính là phép đo đã bác
/// đường (c) của Quyết định #1 *(một segment chưa từng ký có **0** hàng `segment_version`, nên
/// một cột chỉ ở bảng đó không biểu diễn được AC này)*.
#[test]
fn reviewing_a_sentence_word_for_word_keeps_the_origin_it_was_loaded_with() {
    use auratranslate_lib::commands::segment::TRANSLATION_ORIGIN_BILINGUAL_IMPORT;

    let root = temp_dir("origin-kept");
    let opened = create_work_from_text(&root, "Duyet nguyen van", "zh", "", "一。二。".to_owned())
        .expect("tao tac pham that bai");

    let id = read_all_segment_rows(&opened)[0].0;

    opened
        .store
        .write(move |tx: &Transaction<'_>| {
            tx.execute(
                "UPDATE segment SET target_text = ?1, translation_origin = ?2 WHERE id = ?3",
                (
                    "Nhap tu tai lieu song ngu.",
                    TRANSLATION_ORIGIN_BILINGUAL_IMPORT,
                    id,
                ),
            )?;
            Ok(())
        })
        .expect("dung fixture that bai");

    // Khong mot lan `save_segment_targets` nao: nguoi dung KHONG go mot ky tu.
    confirm_segment(Some(&opened), id, "Nhap tu tai lieu song ngu.").expect("xac nhan that bai");

    assert_eq!(
        read_origin(&opened, id),
        TRANSLATION_ORIGIN_BILINGUAL_IMPORT,
        "y het moc ⇒ GIU NGUYEN xuat xu nap vao (AD-31 bang xuat xu, hang 2). Ghi `self` o \
         day la khai chu cua nguoi dung cho mot cau ho khong go -- dung lop hong ma FR117 \
         ton tai de chong"
    );
    assert_eq!(read_state(&opened, id), ("confirmed".to_owned(), 1));

    let dir = opened.dir.clone();
    drop(opened);
    cleanup(&dir);
    cleanup(&root);
}

/// **AC5 — gõ rồi HOÀN TÁC về đúng nguyên trạng ⇒ coi như không sửa.**
///
/// 🔴 Đây là ca phân biệt **phép so văn bản** với một **cờ dirty**, tức chính hợp đồng phụ của
/// AD-31. Và nó là ca mà Quyết định #2 đường *"Rust tự so với đĩa"* **bị bác**: đĩa bị ghi đè
/// dần theo từng lượt flush, nên hai lượt `save_segment_targets` dưới đây đều thấy *"khác"* ở
/// thời điểm chúng chạy — chỉ mốc **lúc nạp** mới trả lời đúng.
#[test]
fn typing_and_undoing_back_to_the_mark_counts_as_untouched() {
    use auratranslate_lib::commands::segment::TRANSLATION_ORIGIN_OTHER;

    let root = temp_dir("origin-undo");
    let opened = create_work_from_text(&root, "Hoan tac", "zh", "", "一。二。".to_owned())
        .expect("tao tac pham that bai");

    let rows = read_all_segment_rows(&opened);
    let (id, chapter_id) = (rows[0].0, rows[0].1);

    opened
        .store
        .write(move |tx: &Transaction<'_>| {
            tx.execute(
                "UPDATE segment SET target_text = ?1, translation_origin = ?2 WHERE id = ?3",
                ("Ban goc.", TRANSLATION_ORIGIN_OTHER, id),
            )?;
            Ok(())
        })
        .expect("dung fixture that bai");

    // Go them, roi hoan tac ve dung nguyen trang. Ca hai luot deu di qua dia.
    save_segment_targets(Some(&opened), chapter_id, &[edit(id, "Ban goc. Them chu.")])
        .expect("luot flush thu nhat that bai");
    save_segment_targets(Some(&opened), chapter_id, &[edit(id, "Ban goc.")])
        .expect("luot flush thu hai that bai");

    confirm_segment(Some(&opened), id, "Ban goc.").expect("xac nhan that bai");

    assert_eq!(
        read_origin(&opened, id),
        TRANSLATION_ORIGIN_OTHER,
        "go roi hoan tac ve nguyen trang ⇒ van ban Y HET moc ⇒ KHONG sua. Mot co dirty se noi \
         `da sua` o day, va no sai -- AD-31 §Hop dong phu goi ten dung ca nay"
    );

    let dir = opened.dir.clone();
    drop(opened);
    cleanup(&dir);
    cleanup(&root);
}

/// 🔵 **Code review 2026-08-16 — một `U+00A0` do `contenteditable` để lại KHÔNG phải một lượt
/// sửa.** Chữ ký thứ **mười** của Ice.
///
/// 🔴 Ca này canh đúng nửa mà lượt rà tìm ra là còn hở: nhánh ② của `confirm_segment` đã được
/// một lượt code review TRƯỚC (2026-08-14) đổi từ `is_empty()` sang `trim().is_empty()` vì
/// `contenteditable` để lại ký tự vô hình — nhưng nhánh ④ *(phép so mốc)* vẫn so `String` THÔ,
/// nên cùng một ký tự vô hình đó làm một câu người dùng **chỉ duyệt** mang nhãn *tôi dịch*.
///
/// ⚠️ **Vì sao fixture phải dùng `TRANSLATION_ORIGIN_OTHER`, ghi ra vì đây là chỗ ca này rất
/// dễ thành một ca XANH GIẢ:** với `''` hay `'self'` — hai giá trị **duy nhất** một `.atproj`
/// thật mang hôm nay — hai nhánh của phép phân xử cho **cùng** kết quả `self`, nên ca sẽ xanh
/// bất kể có `trim()` hay không. Chỉ một xuất xứ **phi-`self`** mới phân biệt được hai nhánh.
/// ⇒ Ca này đo một đường **chưa tới được trên sản phẩm**, và nó có mặt vì Epic 4/6/7/8 sẽ mở
/// đường đó ra — đúng vai *"lớp chặn đặt TRƯỚC"* mà doc-comment ở chỗ dùng đã ghi.
///
/// Chạy đỏ-rồi-xanh: bỏ `.trim()` ở một trong hai vế của `segment.rs`, ca này phải ĐỎ.
#[test]
fn a_stray_invisible_space_is_not_an_edit() {
    use auratranslate_lib::commands::segment::TRANSLATION_ORIGIN_OTHER;

    let root = temp_dir("origin-nbsp");
    let opened = create_work_from_text(&root, "Khoang trang vo hinh", "zh", "", "一。二。".to_owned())
        .expect("tao tac pham that bai");

    let rows = read_all_segment_rows(&opened);
    let (id, chapter_id) = (rows[0].0, rows[0].1);

    opened
        .store
        .write(move |tx: &Transaction<'_>| {
            tx.execute(
                "UPDATE segment SET target_text = ?1, translation_origin = ?2 WHERE id = ?3",
                ("Ban goc.", TRANSLATION_ORIGIN_OTHER, id),
            )?;
            Ok(())
        })
        .expect("dung fixture that bai");

    // `U+00A0` (NO-BREAK SPACE) cuoi dong + mot khoang trang thuong. Day la thu `contenteditable`
    // de lai khi tieu diem ra vao mot o, KHONG phai thu nguoi dung go.
    save_segment_targets(Some(&opened), chapter_id, &[edit(id, "Ban goc.\u{00A0} ")])
        .expect("luot flush that bai");

    // Moc van la ban LUC NAP, khong mang ky tu vo hinh nao.
    confirm_segment(Some(&opened), id, "Ban goc.").expect("xac nhan that bai");

    assert_eq!(
        read_origin(&opened, id),
        TRANSLATION_ORIGIN_OTHER,
        "mot `U+00A0` do contenteditable de lai KHONG phai mot luot sua. So THO o day gan nhan \
         `self` cho mot cau nguoi dung chi DUYET, va nhan sai do di vinh vien vao kho TM cua \
         Epic 7 -- `str::trim()` cat theo `char::is_whitespace` cua Unicode nen no phu `U+00A0`"
    );

    let dir = opened.dir.clone();
    drop(opened);
    cleanup(&dir);
    cleanup(&root);
}

/// **AC6 — xác nhận lại một câu ĐÃ ký không ghi một byte nào, kể cả xuất xứ.**
///
/// Nhánh ③ của `confirm_segment` (AC13 của Story 2.5) trả về **trước** nhánh chuyển tiếp, nên
/// mệnh đề *"ghi cùng lúc với chuyển tiếp, không ở chỗ nào khác"* phải đúng cả khi chỗ gọi
/// truyền một mốc **sai**.
#[test]
fn re_confirming_an_already_signed_segment_leaves_the_origin_alone() {
    use auratranslate_lib::commands::segment::TRANSLATION_ORIGIN_OTHER;

    let root = temp_dir("origin-noop");
    let opened = create_work_from_text(&root, "Ky lai", "zh", "", "一。二。".to_owned())
        .expect("tao tac pham that bai");

    let id = read_all_segment_rows(&opened)[0].0;

    opened
        .store
        .write(move |tx: &Transaction<'_>| {
            tx.execute(
                "UPDATE segment SET target_text = ?1, translation_origin = ?2 WHERE id = ?3",
                ("Cua nguoi khac.", TRANSLATION_ORIGIN_OTHER, id),
            )?;
            Ok(())
        })
        .expect("dung fixture that bai");

    confirm_segment(Some(&opened), id, "Cua nguoi khac.").expect("luot ky dau that bai");
    assert_eq!(read_origin(&opened, id), TRANSLATION_ORIGIN_OTHER);

    // Luot thu hai, VA voi mot moc sai han. Nhanh ③ phai chan no truoc khi toi phep phan xu.
    let again = confirm_segment(Some(&opened), id, "mot moc hoan toan khac")
        .expect("xac nhan lai PHAI vo hai");
    assert!(
        !again.version_created,
        "luot thu hai KHONG duoc la mot chuyen tiep"
    );
    assert_eq!(
        read_origin(&opened, id),
        TRANSLATION_ORIGIN_OTHER,
        "AC6: xuat xu ghi DUNG tai chuyen tiep. Khong co chuyen tiep ⇒ khong mot byte nao doi, \
         KE CA khi cho goi truyen mot moc sai"
    );
    assert_eq!(read_state(&opened, id), ("confirmed".to_owned(), 1));

    let dir = opened.dir.clone();
    drop(opened);
    cleanup(&dir);
    cleanup(&root);
}

/// 🔴 **MỘT CÂU ĐÃ KÝ KHÔNG BAO GIỜ ĐƯỢC MANG NHÃN *"chưa có bản dịch"*.**
///
/// ⚠️ **Không AC nào của story nêu ca này, và nó là ca THƯỜNG NHẬT** — tìm ra lúc đọc lại
/// đường ghi, không lúc thi hành đặc tả. Kịch bản, từng bước, mọi bước đều có thật hôm nay:
/// người dùng gõ bản dịch → flush ghi xuống đĩa → **đóng Tác phẩm mà chưa xác nhận** → mở lại.
/// Lúc này mốc lúc nạp **bằng** văn bản trên đĩa *(cả hai là thứ vừa flush)*, còn
/// `translation_origin` vẫn `''`: bước di trú 11 chỉ backfill các hàng `confirmed`, và flush
/// **không** đụng cột này *(AD-47 ① nói rõ flush không phải một lượt ghi không-phải-người-dùng
/// — nó chở đúng bộ đệm gõ)*. ⇒ Xác nhận mà không sửa đi vào nhánh *"y hệt mốc"*.
///
/// Không có nhánh sửa sentinel, kết quả là một hàng **tự mâu thuẫn**: `status = 'confirmed'`
/// với `target_text` có chữ, mà xuất xứ nói *"chưa có bản dịch"*. Và nó hỏng **im lặng** —
/// không cổng nào đỏ, cho tới khi Epic 7 đọc cột đó để gắn nhãn một cặp TM.
///
/// Chạy đỏ-rồi-xanh: gỡ `|| translation_origin.is_empty()` khỏi `confirm_segment`, ca này ĐỎ.
#[test]
fn a_signed_sentence_can_never_be_left_claiming_it_has_no_translation() {
    use auratranslate_lib::commands::segment::TRANSLATION_ORIGIN_SELF;

    let root = temp_dir("origin-sentinel");
    let opened = create_work_from_text(&root, "Ban nhap chua ky", "zh", "", "一。二。".to_owned())
        .expect("tao tac pham that bai");

    let rows = read_all_segment_rows(&opened);
    let (id, chapter_id) = (rows[0].0, rows[0].1);

    // Phien MOT: go, flush, dong Tac pham ma KHONG xac nhan.
    save_segment_targets(Some(&opened), chapter_id, &[edit(id, "Ban nhap cua toi.")])
        .expect("lo ghi that bai");
    assert_eq!(
        read_origin(&opened, id),
        "",
        "flush KHONG dat xuat xu -- no cho bo dem go, khong phai mot luot ghi \
         khong-phai-nguoi-dung (AD-47 ①)"
    );

    // Phien HAI: moc luc nap nay BANG van ban tren dia. Xac nhan ma khong sua mot ky tu.
    confirm_segment(Some(&opened), id, "Ban nhap cua toi.").expect("xac nhan that bai");

    assert_eq!(
        read_origin(&opened, id),
        TRANSLATION_ORIGIN_SELF,
        "mot cau CO ban dich ma xuat xu rong nghia la KHONG luot ghi khong-phai-nguoi-dung \
         nao dat van ban do (AD-47 ①(b) doi hai thu di cung mot thao tac) ⇒ no den tu bo dem \
         go ⇒ `self`. Giu `''` o day la de lai mot hang tu mau thuan tren dia nguoi dung"
    );

    let dir = opened.dir.clone();
    drop(opened);
    cleanup(&dir);
    cleanup(&root);
}

/// 🔴 **`insert_segments` set cột mới TƯỜNG MINH — bài học 2.5d, cột thứ hai liên tiếp.**
///
/// ⚠️ Ca này **xanh cả khi bỏ `?6`** khỏi câu `INSERT` hôm nay, vì giá trị đúng trùng với
/// `DEFAULT ''`. Nó vẫn đáng tồn tại, và lý do phải nói ra chứ không giả vờ: nó khoá **mệnh
/// đề** *"một Chương vừa nhập chưa có xuất xứ"*, thứ mà Epic 6 (FR115) sắp phá — lúc đường
/// nhập song ngữ ra đời, `DEFAULT` thôi là giá trị đúng, và ca này là chỗ lượt đổi đó phải đi
/// qua thay vì trôi.
#[test]
fn a_freshly_imported_chapter_starts_with_no_translation_origin() {
    let root = temp_dir("origin-import");
    let opened = create_work_from_text(&root, "Nhap moi", "zh", "", "一。二。三。".to_owned())
        .expect("tao tac pham that bai");

    let origins: Vec<String> = read_all_segment_rows(&opened)
        .iter()
        .map(|r| r.12.clone())
        .collect();

    assert_eq!(
        origins,
        vec!["".to_owned(), "".to_owned(), "".to_owned()],
        "mot Chuong vua nhap chua co ban dich nao, nen chua cau nao co xuat xu"
    );

    let dir = opened.dir.clone();
    drop(opened);
    cleanup(&dir);
    cleanup(&root);
}

/// 🔴 **Bước 11 backfill theo HÀNG — `confirmed` ⇒ *tôi dịch*, phần còn lại giữ `''`.**
///
/// Khuôn `a_project_database_at_version_eight_backfills_the_target_flag_...` (bước 9), và
/// mệnh đề cùng hạng: một bước DDL+DML phải nghiệm thu được **cả hai vế**, không chỉ vế cột
/// đã có mặt.
///
/// ⚠️ Fixture dựng bằng các bước **THẬT** của `PROJECT_MIGRATIONS`, không chép tay DDL — một
/// fixture chép tay trôi khỏi hằng thật ở đúng story mà hằng thật đổi.
#[test]
fn a_project_database_at_version_ten_backfills_the_origin_only_for_signed_rows() {
    use auratranslate_lib::commands::segment::TRANSLATION_ORIGIN_SELF;

    let dir = temp_dir("v10-backfills-origin");
    let db = dir.join("project.db");

    // Fixture o phien ban 10: muoi buoc THAT tru buoc cuoi.
    static THROUGH_TEN: [Migration; 9] = [
        PROJECT_MIGRATIONS[0],
        PROJECT_MIGRATIONS[1],
        PROJECT_MIGRATIONS[2],
        PROJECT_MIGRATIONS[3],
        PROJECT_MIGRATIONS[4],
        PROJECT_MIGRATIONS[5],
        PROJECT_MIGRATIONS[6],
        PROJECT_MIGRATIONS[7],
        PROJECT_MIGRATIONS[8],
    ];

    let old = Store::open(StoreSpec {
        migrations: &THROUGH_TEN,
        ..StoreSpec::project(db.clone())
    })
    .expect("dung fixture o phien ban 10");
    assert_eq!(
        old.schema_version(),
        10,
        "fixture phai dung o 10 -- neu no da la 11 thi ca nay khong do gi ca"
    );

    // Bon hang THAT, phu ca bon to hop cua (status, target_text).
    old.write(|tx: &Transaction<'_>| {
        for (id, ord, target, status) in [
            (1_i64, 1_i64, "Da ky, co chu.", "confirmed"),
            (2, 2, "Ban nhap, chua ky.", "draft"),
            (3, 3, "", "draft"),
            (4, 4, "Da ky, cau thu hai.", "confirmed"),
        ] {
            tx.execute(
                "INSERT INTO segment (id, chapter_id, ord, source_text, is_paragraph_end, \
                 target_text, status, created_at, updated_at) \
                 VALUES (?1, 1, ?2, 'nguon', 0, ?3, ?4, 'x', 'x')",
                (id, ord, target, status),
            )?;
        }
        Ok(())
    })
    .expect("bom bon hang segment vao fixture");
    drop(old);

    // Di tru len dich.
    let migrated = Store::open(StoreSpec::project(db))
        .expect("mot `project.db` o phien ban 10 phai mo duoc");
    // 🔵 CAP NHAT 2026-08-19 (Story 3.1): dich 11 → 12 — buoc 12 ra doi.
    // 🔵 CAP NHAT 2026-08-20 (Story 3.2): dich 12 → 13 — buoc 13 ra doi.
    // 🔵 CAP NHAT 2026-08-22 (Story 3.5): dich 13 → 14 — buoc 14 ra doi.
    // 🔵 CAP NHAT 2026-08-24 (Story 3.10): dich 14 → 15 — buoc 15 ra doi.
    // 🔵 CAP NHAT 2026-08-27 (Story 5.4): dich 15 → 16 — buoc 16 (work.status_override,
    // FR6) ra doi. Menh de van khong doi.
    assert_eq!(
        migrated.schema_version(),
        18,
        "buoc 11..18 phai chay tren mot tep dung o phien ban 10"
    );

    let after: Vec<(i64, String, String)> = migrated
        .read(|conn| {
            let mut stmt = conn.prepare(
                "SELECT id, status, translation_origin FROM segment ORDER BY ord",
            )?;
            let rows = stmt.query_map([], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)))?;
            rows.collect::<Result<Vec<_>, _>>()
        })
        .expect("doc bon hang sau luot di tru");

    assert_eq!(
        after,
        vec![
            (1, "confirmed".to_owned(), TRANSLATION_ORIGIN_SELF.to_owned()),
            (2, "draft".to_owned(), String::new()),
            (3, "draft".to_owned(), String::new()),
            (4, "confirmed".to_owned(), TRANSLATION_ORIGIN_SELF.to_owned()),
        ],
        "Quyet dinh #6(a): chi hang DA KY nhan `self`. Mot cau CHUA ky chua duoc ai duyet, \
         nen khai bat cu xuat xu nao cho no la mot loi khai ve mot thu chua xay ra. \
         ⚠️ Menh de bien minh cho `confirmed ⇒ self` la MOT PHEP DO ve hom nay -- khong co \
         che nao ngoai nguoi dung dat duoc van ban vao mot segment (FR115 Epic 6, FR58 Epic 7, \
         AI Epic 4, FR94 Epic 8 deu chua ton tai). No se het dung, va no khong het dung LUI \
         VE QUA KHU"
    );

    drop(migrated);
    cleanup(&dir);
}

// ═════════════════════════════════════════════════════════════════════════════════
// STORY 2.8 — GỘP VÀ TÁCH SEGMENT TƯỜNG MINH (FR78 · AD-5 · AD-37 · AD-47 ④)
//
// 🔴 Tám ca dưới đây chạy trên **hàm thuần** `core::segment::regroup`, không chạm đĩa.
// Đó là cả lý do module đó tách khỏi closure `Store::write`: bốn luật ngoài mã (AD-37 ·
// AD-47 ④ · chữ ký #5(a) · #3(b)) phải gọi tới được từ đây.
// ═════════════════════════════════════════════════════════════════════════════════

/// **Chữ ký #3(b)** — dấu nối `source_text` đi theo **ngôn ngữ NGUỒN**.
///
/// 🔴 Ca này là lý do đường (a) bị loại: `" "` cho mọi ngôn ngữ **sai** ở Tác phẩm tiếng
/// Trung, tức ca thường nhất của sản phẩm.
#[test]
fn merging_joins_chinese_sources_without_a_space_and_every_other_language_with_one() {
    use auratranslate_lib::core::segment::paragraph::ParagraphFlags;
    use auratranslate_lib::core::segment::regroup::{merge, SegmentPart};

    let parts = [
        SegmentPart {
            source_text: "他走了。",
            target_text: "",
            flags: ParagraphFlags::mirrored(false),
            is_omitted: false,
            translation_origin: "",
        },
        SegmentPart {
            source_text: "她留下。",
            target_text: "",
            flags: ParagraphFlags::mirrored(false),
            is_omitted: false,
            translation_origin: "",
        },
    ];

    assert_eq!(
        merge(&parts, "zh").expect("nhom hai phan tu phai gop duoc").source_text,
        "他走了。她留下。",
        "hai cau tieng Trung noi nhau KHONG co khoang trang -- day la ve ma duong (a) lam sai"
    );

    // ⚠️ Mọi giá trị khác `"zh"` đi nhánh khoảng trắng, đúng luật mặc định đã khai bằng chữ
    // ở `split.rs::LANG_CHINESE`. Kiểm cả một giá trị KHÔNG có trong FR23 — cột
    // `work.source_lang` nhận chuỗi tự do.
    for lang in ["en", "fr", ""] {
        let parts = [
            SegmentPart {
                source_text: "He left.",
                target_text: "",
                flags: ParagraphFlags::mirrored(false),
                is_omitted: false,
                translation_origin: "",
            },
            SegmentPart {
                source_text: "She stayed.",
                target_text: "",
                flags: ParagraphFlags::mirrored(false),
                is_omitted: false,
                translation_origin: "",
            },
        ];
        assert_eq!(
            merge(&parts, lang).expect("gop duoc").source_text,
            "He left. She stayed.",
            "`source_lang = {lang:?}` phai di nhanh khoang trang -- chi `zh` la ngoai le"
        );
    }
}

/// 🔴 **Một lượt gộp không được đẻ ra một ký tự người dùng chưa từng gõ.**
///
/// Nối `"A"` với `""` bằng `" "` cho `"A "`, và khoảng trắng đó **nằm trên đĩa vĩnh viễn**
/// mà không bề mặt nào lộ ra: `confirm_segment` nay `trim()` hai vế nên phép so mốc vẫn
/// đúng, và không cổng nào đọc `target_text`. Một lỗi im lặng hoàn hảo.
#[test]
fn merging_never_manufactures_whitespace_the_user_never_typed() {
    use auratranslate_lib::core::segment::paragraph::ParagraphFlags;
    use auratranslate_lib::core::segment::regroup::{merge, SegmentPart};

    let da_dich = SegmentPart {
        source_text: "他走了。",
        target_text: "Anh ta di roi.",
        flags: ParagraphFlags::mirrored(false),
        is_omitted: false,
        translation_origin: "self",
    };
    let chua_dich = SegmentPart {
        source_text: "她留下。",
        target_text: "",
        flags: ParagraphFlags::mirrored(false),
        is_omitted: false,
        translation_origin: "",
    };

    assert_eq!(
        merge(&[da_dich, chua_dich], "zh").expect("gop duoc").target_text,
        "Anh ta di roi.",
        "manh RONG bi bo qua -- khong mot khoang trang duoi duoc noi vao"
    );
    assert_eq!(
        merge(&[chua_dich, da_dich], "zh").expect("gop duoc").target_text,
        "Anh ta di roi.",
        "va cung the o chieu nguoc lai -- khong mot khoang trang DAU nao"
    );
    assert_eq!(
        merge(&[chua_dich, chua_dich], "zh").expect("gop duoc").target_text,
        "",
        "hai cau CHUA DICH gop lai van la chua dich -- khong phai mot chuoi mot khoang trang"
    );
    assert_eq!(
        merge(&[da_dich, da_dich], "zh").expect("gop duoc").target_text,
        "Anh ta di roi. Anh ta di roi.",
        "hai ban dich THAT thi van noi bang dung mot khoang trang"
    );
}

/// **Chữ ký #5(a) của Ice** — bất kỳ mảnh nào đã cắt bỏ ⇒ hàng mới đã cắt bỏ.
///
/// ⚠️ Chiều này **ngược** AD-47 ④ ở ca bất đồng, và ca kế tiếp canh chính chỗ đó.
#[test]
fn merging_carries_the_omitted_flag_from_any_piece_not_from_all_of_them() {
    use auratranslate_lib::core::segment::paragraph::ParagraphFlags;
    use auratranslate_lib::core::segment::regroup::{merge, SegmentPart};

    let base = SegmentPart {
        source_text: "x",
        target_text: "",
        flags: ParagraphFlags::mirrored(false),
        is_omitted: false,
        translation_origin: "",
    };
    let da_cat = SegmentPart {
        is_omitted: true,
        ..base
    };

    assert!(
        merge(&[base, da_cat], "zh").expect("gop duoc").is_omitted,
        "MOT manh da cat la du -- chu ky #5(a) chon chieu an toan cho quyet dinh cua nguoi dung"
    );
    assert!(
        merge(&[da_cat, base], "zh").expect("gop duoc").is_omitted,
        "va thu tu khong doi ket qua"
    );
    assert!(
        !merge(&[base, base], "zh").expect("gop duoc").is_omitted,
        "khong manh nao da cat thi hang moi KHONG cat -- luat khong duoc bat co tu hu khong"
    );
}

/// **AD-47 ④, nguyên văn** — đồng ý ⇒ giữ; **bất kỳ** bất đồng nào ⇒ `other`.
///
/// 🔴 Ca cuối canh đúng **cái mất** mà AD-47 ④ ghi sẵn bằng chữ: gộp một câu `""` *(chưa
/// dịch)* với một câu *tôi dịch* **cũng** rơi vào nhánh bất đồng. Ai đọc ca này sau đừng
/// "sửa" nó thành `self` — đó là luật, không phải một lỗi.
#[test]
fn merging_keeps_a_unanimous_origin_and_falls_back_to_other_on_any_disagreement() {
    use auratranslate_lib::core::segment::paragraph::ParagraphFlags;
    use auratranslate_lib::core::segment::regroup::{merge, SegmentPart, ORIGIN_OTHER};

    let part = |origin: &'static str| SegmentPart {
        source_text: "x",
        target_text: "y",
        flags: ParagraphFlags::mirrored(false),
        is_omitted: false,
        translation_origin: origin,
    };

    assert_eq!(
        merge(&[part("self"), part("self")], "zh").expect("gop duoc").translation_origin,
        "self",
        "moi manh cung mot gia tri ⇒ giu nguyen gia tri do (AD-47 ④, ve dong y)"
    );
    assert_eq!(
        merge(&[part(""), part("")], "zh").expect("gop duoc").translation_origin,
        "",
        "hai cau chua dich dong y voi nhau o gia tri rong -- va rong la mot GIA TRI, khong \
         phai mot cho trong de doan lai"
    );
    assert_eq!(
        merge(&[part("self"), part("bilingual_import")], "zh")
            .expect("gop duoc")
            .translation_origin,
        ORIGIN_OTHER,
        "bat ky bat dong nao ⇒ `other` (AD-47 ④, ve bat dong)"
    );
    assert_eq!(
        merge(&[part(""), part("self")], "zh").expect("gop duoc").translation_origin,
        ORIGIN_OTHER,
        "🔴 CAI MAT ma AD-47 ④ da ghi san bang chu: gop mot cau CHUA DICH voi mot cau `self` \
         cung roi vao nhanh bat dong. Day la LUAT, khong phai mot loi -- dung sua ca nay \
         thanh `self`"
    );
}

/// Nhóm **rỗng** ⇒ `None`, ở cả hai luật cùng lúc.
#[test]
fn merging_an_empty_group_invents_nothing() {
    use auratranslate_lib::core::segment::regroup::merge;

    assert_eq!(
        merge(&[], "zh"),
        None,
        "mot nhom gop RONG phai tra `None` -- dung lop \"rong im lang\" ma project-context cam"
    );
}

/// 🔴 **Mảnh SAU của một lượt tách không có bản dịch ⇒ không có xuất xứ để khai.**
///
/// Đây là một **suy dẫn** từ AD-47 ④ cộng mệnh đề mà `insert_segments` đã khai bằng chữ,
/// không một luật mới. Làm ngược lại cho một hàng **tự mâu thuẫn** trên đĩa người dùng:
/// *"tôi đã dịch câu này"* + `target_text` rỗng.
#[test]
fn splitting_gives_the_tail_piece_no_translation_and_therefore_no_origin() {
    use auratranslate_lib::core::segment::paragraph::ParagraphFlags;
    use auratranslate_lib::core::segment::regroup::{split_at, SegmentPart, ORIGIN_NONE};

    let part = SegmentPart {
        source_text: "Mr. Smith den.",
        target_text: "Ong Smith den.",
        flags: ParagraphFlags {
            source: true,
            target: false,
        },
        is_omitted: true,
        translation_origin: "self",
    };

    let pieces = split_at(&part, &[3]).expect("cat o giua chuoi phai ra hai manh");
    assert_eq!(pieces.len(), 2, "tach doi cho dung HAI manh");

    assert_eq!(pieces[0].source_text, "Mr.");
    assert_eq!(pieces[1].source_text, " Smith den.");

    assert_eq!(
        pieces[0].target_text, "Ong Smith den.",
        "manh DAU giu tron ban dich (chu ky #3(b), ve tach)"
    );
    assert_eq!(
        pieces[1].target_text, "",
        "manh SAU khong co ban dich -- khong co phep chieu nao tu cho cat ben nguon sang \
         ban dich (epics.md:2552)"
    );
    assert_eq!(
        pieces[0].translation_origin, "self",
        "manh mang ban dich giu nguyen xuat xu cua no (AD-47 ④, ca tam thuong)"
    );
    assert_eq!(
        pieces[1].translation_origin, ORIGIN_NONE,
        "🔴 manh KHONG co ban dich thi khong co xuat xu nao de khai. Cho no `self` la dung \
         mot hang TU MAU THUAN tren dia: \"toi da dich cau nay\" + \"chua co ban dich\""
    );

    // Cắt bỏ là một **trục độc lập** và nó đi theo **cả hai** mảnh: người dùng đã quyết định
    // câu này không thuộc bản dịch, và một lượt tách không đảo quyết định đó.
    assert!(pieces[0].is_omitted && pieces[1].is_omitted);

    // AD-37 ca ③ — mảnh CUỐI giữ cặp cờ, mảnh trước tắt **cả hai** cột.
    assert_eq!(
        pieces[0].flags,
        ParagraphFlags {
            source: false,
            target: false
        },
        "manh TRUOC tat ca hai cot"
    );
    assert_eq!(
        pieces[1].flags,
        ParagraphFlags {
            source: true,
            target: false
        },
        "manh CUOI giu nguyen CAP co -- tung cot mot, khong mot phep OR"
    );
}

/// 🔴 **Chỗ cắt đếm KÝ TỰ, không byte** — và một chỉ số hỏng không được giết tiến trình.
///
/// `panic = "abort"` biến mọi `panic!` thành cái chết của tiến trình: không unwind, không
/// `Drop`, không cơ hội flush WAL. Đây là chỗ **duy nhất** của story này nhận một chỉ số từ
/// webview, nên nó là chỗ duy nhất phải chịu được một số bất kỳ.
#[test]
fn splitting_counts_characters_not_bytes_and_refuses_a_cut_that_leaves_an_empty_piece() {
    use auratranslate_lib::core::segment::paragraph::ParagraphFlags;
    use auratranslate_lib::core::segment::regroup::{split_at, SegmentPart};

    // "他走了。" = 4 ký tự, **12 byte**. Một chỉ số byte ở đây rơi giữa một ký tự.
    let part = SegmentPart {
        source_text: "他走了。",
        target_text: "",
        flags: ParagraphFlags::mirrored(false),
        is_omitted: false,
        translation_origin: "",
    };
    assert_eq!(part.source_text.len(), 12, "tien de cua ca nay: 12 byte");
    assert_eq!(part.source_text.chars().count(), 4, "va 4 ky tu");

    let pieces = split_at(&part, &[2]).expect("cat sau ky tu thu hai");
    assert_eq!(pieces[0].source_text, "他走");
    assert_eq!(pieces[1].source_text, "了。");

    for cut in [0, 4, 5, 12, usize::MAX] {
        assert_eq!(
            split_at(&part, &[cut]),
            None,
            "cho cat {cut} de lai mot manh RONG (hoac nam ngoai chuoi) ⇒ phai tu choi. Mot \
             hang `segment` khong co van ban nguon la dung thu khong duong ma nao phia sau \
             biet xu ly"
        );
    }
}

/// 🔴 **`n` chỗ cắt cho `n + 1` mảnh, trong MỘT lượt** — AC7 vế *"nhiều mảnh"*, chữ ký của
/// Ice ngày 2026-08-17 sau code review.
///
/// Ca này khoá bốn mệnh đề mà bản hai-mảnh **không phát biểu được**:
/// ① số mảnh đi theo số chỗ cắt; ② mảnh **đầu** giữ trọn bản dịch và xuất xứ, **mọi** mảnh
/// sau rỗng cả hai *(không chỉ mảnh thứ hai)*; ③ AD-37 ca ③ áp cho `n` mảnh — chỉ mảnh
/// **cuối** giữ cặp cờ; ④ thứ tự bấm chuột của người dùng không phải thứ tự trong câu.
#[test]
fn splitting_at_many_cuts_makes_one_piece_more_than_the_cuts_in_a_single_pass() {
    use auratranslate_lib::core::segment::paragraph::ParagraphFlags;
    use auratranslate_lib::core::segment::regroup::{split_at, SegmentPart, ORIGIN_NONE};

    // 12 ky tu: 他走了。她来了。我看见。
    let part = SegmentPart {
        source_text: "他走了。她来了。我看见。",
        target_text: "Han di roi.",
        flags: ParagraphFlags {
            source: true,
            target: true,
        },
        is_omitted: false,
        translation_origin: "self",
    };
    assert_eq!(part.source_text.chars().count(), 12, "tien de cua ca nay");

    // 🔴 CO Y dua vao KHONG SAP XEP: nguoi dung bam cho thu hai TRUOC cho thu nhat, va
    //    khong luat nao buoc ho bam theo thu tu doc.
    let pieces = split_at(&part, &[8, 4]).expect("hai cho cat hop le phai ra BA manh");

    assert_eq!(pieces.len(), 3, "① hai cho cat ⇒ BA manh, khong hai");
    assert_eq!(pieces[0].source_text, "他走了。");
    assert_eq!(pieces[1].source_text, "她来了。");
    assert_eq!(pieces[2].source_text, "我看见。");

    // ② Manh DAU giu tron ban dich va xuat xu; MOI manh sau rong CA HAI.
    assert_eq!(pieces[0].target_text, "Han di roi.");
    assert_eq!(pieces[0].translation_origin, "self");
    for (i, m) in pieces.iter().enumerate().skip(1) {
        assert_eq!(m.target_text, "", "manh {i} khong duoc mang ban dich");
        assert_eq!(
            m.translation_origin, ORIGIN_NONE,
            "manh {i} khong co ban dich ⇒ khong co xuat xu de khai. Mot `self` o day la mot \
             hang TU MAU THUAN tren dia"
        );
    }

    // ③ AD-37 ca ③ voi n = 3: chi manh CUOI giu cap co.
    let tat = ParagraphFlags {
        source: false,
        target: false,
    };
    assert_eq!(pieces[0].flags, tat, "manh dau tat ca hai cot");
    assert_eq!(pieces[1].flags, tat, "manh GIUA cung tat -- ca nay chi ton tai tu n >= 3");
    assert_eq!(
        pieces[2].flags,
        ParagraphFlags {
            source: true,
            target: true
        },
        "chi manh CUOI giu nguyen cap co"
    );

    // ④ Sap xep tai cho goi ⇒ dua vao da sap cho ket qua Y HET.
    assert_eq!(
        split_at(&part, &[4, 8]),
        split_at(&part, &[8, 4]),
        "thu tu bam chuot khong duoc doi ket qua"
    );
}

/// 🔴 **HAI chỗ cắt TRÙNG NHAU ⇒ từ chối** — một biên **chỉ tồn tại từ lượt đa-mảnh**.
///
/// Bản hai-mảnh không phát biểu được ca này: với một chỉ số duy nhất thì *"trùng"* vô nghĩa.
/// Hai chỗ cắt bằng nhau cho một mảnh giữa **rỗng** — cùng lớp *"rỗng im lặng"* mà ba ca biên
/// cũ (`0`, cuối chuỗi, ngoài chuỗi) đã chặn, chỉ khác đường vào.
///
/// ⚠️ Và `cuts` **rỗng** cũng bị từ chối: không có lượt tách nào được yêu cầu, nên trả một
/// mảnh duy nhất bằng chính segment cũ sẽ là **về hưu + tạo mới cho một thao tác rỗng** —
/// một `id` mới, lịch sử rỗng, không một ký tự nào đổi.
#[test]
fn splitting_refuses_duplicate_cuts_and_an_empty_cut_list() {
    use auratranslate_lib::core::segment::paragraph::ParagraphFlags;
    use auratranslate_lib::core::segment::regroup::{split_at, SegmentPart};

    let part = SegmentPart {
        source_text: "他走了。她来了。",
        target_text: "",
        flags: ParagraphFlags::mirrored(false),
        is_omitted: false,
        translation_origin: "",
    };

    assert_eq!(
        split_at(&part, &[]),
        None,
        "khong cho cat nao ⇒ khong mot luot tach nao duoc yeu cau. Ve huu + tao moi cho mot \
         thao tac RONG la mot `id` moi va mot lich su rong, khong mot ky tu nao doi"
    );
    assert_eq!(
        split_at(&part, &[4, 4]),
        None,
        "hai cho cat TRUNG NHAU ⇒ manh giua RONG ⇒ tu choi"
    );
    assert_eq!(
        split_at(&part, &[2, 6, 2]),
        None,
        "trung nhau o bat ky dau trong danh sach, ke ca khi chua sap xep"
    );
    // Doi chung: cung so luong cho cat, khong trung ⇒ di qua.
    assert!(
        split_at(&part, &[2, 6]).is_some(),
        "doi chung -- hai cho cat KHAC nhau thi hop le"
    );
}

/// 🔴 **Hai hằng xuất xứ ở `core::segment::regroup` phải khớp `commands::segment`.**
///
/// `core::**` không được phụ thuộc `commands::**` *(chiều đúng là commands → core, và
/// `tests/segment_boundary.rs` cưỡng chế)*, nên hai chuỗi `"other"` / `""` sống ở **hai**
/// chỗ và **không cổng nào** canh chúng khớp nhau. Ca này là cái canh đó.
#[test]
fn the_two_origin_constants_of_the_pure_layer_match_the_command_layer_verbatim() {
    use auratranslate_lib::commands::segment::{TRANSLATION_ORIGIN_NONE, TRANSLATION_ORIGIN_OTHER};
    use auratranslate_lib::core::segment::regroup::{ORIGIN_NONE, ORIGIN_OTHER};

    assert_eq!(
        ORIGIN_OTHER, TRANSLATION_ORIGIN_OTHER,
        "hai hang phai la CUNG mot chuoi -- mot lech giua chung cho ra mot gia tri xuat xu \
         khong nam trong danh muc dong cua AD-47, va khong cong nao do"
    );
    assert_eq!(
        ORIGIN_NONE, TRANSLATION_ORIGIN_NONE,
        "va cung the voi gia tri \"chua co ban dich\""
    );
}

// ═════════════════════════════════════════════════════════════════════════════════
// STORY 2.8 — ĐƯỜNG SQL THẬT: về hưu + tạo mới, trên một `project.db` thật
// ═════════════════════════════════════════════════════════════════════════════════

/// Đọc thẳng bảng `segment` — thứ mọi ca dưới đây dùng để nhìn **đĩa**, không nhìn qua một
/// lệnh đọc có thể tự lọc.
///
/// 🔴 Đọc **thô** có chủ ý: `read_open_chapter_segments` là một **đường đọc**, và một ca
/// canh đĩa mà đi qua nó sẽ xanh y nguyên vào ngày đường đọc đó bắt đầu lọc.
fn hang_tho(
    opened: &auratranslate_lib::commands::project::OpenWork,
) -> Vec<(i64, i64, String, String, bool, bool, bool, String, bool)> {
    opened
        .store
        .read(|conn| {
            let mut stmt = conn.prepare(
                "SELECT id, ord, source_text, target_text, is_paragraph_end, \
                 is_target_paragraph_end, is_omitted, status, retired_at \
                 FROM segment ORDER BY ord, id",
            )?;
            let rows = stmt.query_map([], |r| {
                let para: i64 = r.get(4)?;
                let tpara: i64 = r.get(5)?;
                let omit: i64 = r.get(6)?;
                let retired: Option<String> = r.get(8)?;
                Ok((
                    r.get(0)?,
                    r.get(1)?,
                    r.get(2)?,
                    r.get(3)?,
                    para != 0,
                    tpara != 0,
                    omit != 0,
                    r.get(7)?,
                    retired.is_some(),
                ))
            })?;
            rows.collect::<Result<Vec<_>, _>>()
        })
        .expect("doc tho bang segment")
}

/// **AC1 · AC3** — gộp hai câu: cả hai **về hưu**, một hàng mới ra đời **chưa xác nhận** với
/// **lịch sử rỗng**.
///
/// 🔴 Ca này cũng canh AD-31 §bảng máy trạng thái hàng *"Về hưu do gộp/tách ⇒ **không** tạo
/// `segment_version`"*. Bản năng khi đọc *"đừng để mất bản dịch"* là chụp một phiên bản
/// trước khi cho về hưu — và lượt chụp đó phá AC3 theo một cách đọc rất giống một tính năng.
#[test]
fn merging_retires_both_rows_and_creates_one_unconfirmed_row_with_an_empty_history() {
    use auratranslate_lib::commands::segment::{confirm_segment, merge_segments, save_segment_targets, SegmentTargetEdit};

    let root = temp_dir("2-8-merge");
    let opened = create_work_from_text(&root, "2.8 gop", "zh", "", "一。二。三。".to_owned())
        .expect("tao tac pham");

    // Hai cau dau CO ban dich va DA XAC NHAN -- de ca nay canh duoc ca menh de "lich su rong"
    // (chung co lich su that truoc khi gop) lan menh de "hang moi CHUA xac nhan".
    for (id, text) in [(1i64, "Mot."), (2, "Hai.")] {
        save_segment_targets(
            Some(&opened),
            1,
            &[SegmentTargetEdit {
                id,
                target_text: text.to_owned(),
            }],
        )
        .expect("ghi ban dich");
        confirm_segment(Some(&opened), id, "").expect("xac nhan");
    }

    let out = merge_segments(Some(&opened), 2).expect("gop cau 2 voi cau lien tren no");
    let ve_huu_ids: Vec<i64> = out.retired.iter().map(|r| r.id).collect();
    assert_eq!(ve_huu_ids, vec![1, 2], "ca HAI hang cu ve huu (AC1)");
    assert!(
        out.retired.iter().all(|r| r.retired_at.is_some()),
        "hai hang tra ve phai MANG `retired_at` that -- webview ve vach `ornament` bang chinh \
         truong do, va mot danh sach `id` tran buoc no BIA ra mot moc thoi gian (AD-1)"
    );
    assert_eq!(out.new_segments.len(), 1, "va dung MOT hang moi ra doi");

    let moi = &out.new_segments[0];
    assert!(moi.id > 2, "hang moi mang mot `id` CHUA TUNG dung (AD-3)");
    assert_eq!(
        moi.status, "draft",
        "AC3 -- segment moi bat dau CHUA XAC NHAN, du ca hai cau cu deu da xac nhan"
    );
    assert_eq!(moi.source_text, "一。二。", "hai cau tieng Trung noi nhau KHONG khoang trang");
    assert_eq!(moi.target_text, "Mot. Hai.", "hai ban dich noi bang dung mot khoang trang");
    assert_eq!(moi.retired_at, None, "hang moi con song");

    // AC3, ve LICH SU RONG -- va no phai dung DU hai cau cu deu co `segment_version` that.
    let history = read_segment_history(Some(&opened), moi.id).expect("doc lich su hang moi");
    assert!(
        history.is_empty(),
        "AC3 -- lich su cua segment moi phai RONG. Mot luot chup `segment_version` truoc khi \
         cho ve huu se pha dung menh de nay, va no doc rat giong mot tinh nang"
    );

    let tho = hang_tho(&opened);
    let ve_huu: Vec<i64> = tho.iter().filter(|r| r.8).map(|r| r.0).collect();
    assert_eq!(ve_huu, vec![1, 2], "dung hai hang mang `retired_at` khac NULL");

    cleanup(&root);
}

/// 🔴 **AC4 trên một segment về hưu THẬT** — không một hàng dựng bằng SQL.
///
/// `deferred-work.md:3675-3683` ghi thẳng khoảng hở này: mọi ca AC4 trước story này bơm
/// `retired_at` bằng SQL trực tiếp, nên chúng canh *"đường đọc không hỏi `retired_at`"* chứ
/// **không** canh *"một lượt gộp thật để lại lịch sử tra được"*. Story 2.8 là lượt đầu tiên
/// đóng được vế đó, và đây là ca đóng nó.
#[test]
fn the_history_of_a_genuinely_retired_segment_still_reads_back_after_a_real_merge() {
    use auratranslate_lib::commands::segment::{confirm_segment, merge_segments, save_segment_targets, SegmentTargetEdit};

    let root = temp_dir("2-8-ac4");
    let opened = create_work_from_text(&root, "2.8 AC4", "zh", "", "一。二。".to_owned())
        .expect("tao tac pham");

    save_segment_targets(
        Some(&opened),
        1,
        &[SegmentTargetEdit {
            id: 1,
            target_text: "Ban dich se song sot qua luot gop.".to_owned(),
        }],
    )
    .expect("ghi ban dich");
    confirm_segment(Some(&opened), 1, "").expect("xac nhan -- day la luot sinh ra hang lich su");

    let truoc = read_segment_history(Some(&opened), 1).expect("doc lich su truoc khi gop");
    assert_eq!(truoc.len(), 1, "tien de: cau 1 co dung mot phien ban truoc luot gop");

    merge_segments(Some(&opened), 2).expect("gop -- cau 1 ve huu THAT sau lenh nay");

    let sau = read_segment_history(Some(&opened), 1).expect("doc lich su SAU khi cau 1 ve huu");
    assert_eq!(
        sau, truoc,
        "AC4 -- lich su phien ban cua mot segment DA VE HUU van tra lai duoc, nguyen ven. \
         Duong DOC va duong GHI tu choi KHAC NHAU: bon lenh ghi tu choi mot segment ve huu, \
         duong doc thi khong duoc phep"
    );

    cleanup(&root);
}

/// **Chữ ký #7(a)** — `ord` đánh lại **liên tục 1..N** cho hàng còn sống; hàng về hưu đậu ở
/// `ord` của **hàng đầu nhóm**.
///
/// 🔴 Vế thứ hai không phải trang trí: AD-5 hứa *"chỗ đánh dấu khi đọc (FR119) trỏ tới
/// segment về hưu vẫn **mở được về đúng vị trí trong Chương**"*. Một hàng về hưu mang `ord`
/// cũ ở cuối dãy sẽ đáp xuống sai chỗ.
#[test]
fn merging_renumbers_the_living_rows_from_one_and_parks_the_retired_ones_at_the_group_head() {
    use auratranslate_lib::commands::segment::merge_segments;

    let root = temp_dir("2-8-ord");
    let opened = create_work_from_text(&root, "2.8 ord", "zh", "", "一。二。三。四。".to_owned())
        .expect("tao tac pham");

    // Gop cau 3 voi cau 2 -- co y KHONG o dau day, de mot phep danh lai chi-tu-cho-cham
    // phan biet duoc voi mot phep danh lai ca Chuong.
    let out = merge_segments(Some(&opened), 3).expect("gop cau 3 voi cau 2");
    let moi_id = out.new_segments[0].id;

    let tho = hang_tho(&opened);
    let song: Vec<(i64, i64)> = tho
        .iter()
        .filter(|r| !r.8)
        .map(|r| (r.0, r.1))
        .collect();
    assert_eq!(
        song,
        vec![(1, 1), (moi_id, 2), (4, 3)],
        "ba hang con song mang `ord` 1..3 LIEN TUC, va hang moi dung dung cho nhom cu tung dung"
    );

    let ve_huu: Vec<(i64, i64)> = tho.iter().filter(|r| r.8).map(|r| (r.0, r.1)).collect();
    assert_eq!(
        ve_huu,
        vec![(2, 2), (3, 2)],
        "hai hang ve huu dau o `ord` cua HANG DAU NHOM (2) -- AD-5 hua mot cho danh dau tro \
         toi chung van mo duoc ve DUNG VI TRI trong Chuong"
    );

    cleanup(&root);
}

/// **AC2 · AC7** — tách một câu: câu cũ về hưu, hai mảnh mới ra đời **đúng thứ tự**, và cờ
/// kết đoạn theo **mảnh cuối** với mọi mảnh trước **tắt cả hai cột**.
#[test]
fn splitting_retires_the_source_row_and_creates_the_pieces_in_reading_order() {
    use auratranslate_lib::commands::segment::{split_segment, set_segment_paragraph_end};

    let root = temp_dir("2-8-split");
    let opened =
        create_work_from_text(&root, "2.8 tach", "en", "", "Mr. Smith came. He left.".to_owned())
            .expect("tao tac pham");

    let truoc = hang_tho(&opened);
    // Bo tach cau tieng Anh cat o "Mr." -- dung ca ma AC ton tai de giai. Ta khong dua vao
    // so segment bo tach sinh ra; ta lay hang DAU va tach no bang tay.
    let dau = truoc[0].0;

    // Cho cau dau mot co dich BAT va co nguon TAT -- hai co LECH nhau, dung hinh dang ma mot
    // luot cai "co dich chac cung nhu co nguon" se lam hong.
    set_segment_paragraph_end(Some(&opened), dau, true).expect("bat co dich");

    let cut = truoc[0].2.chars().count() - 1;
    let out = split_segment(Some(&opened), dau, vec![cut]).expect("tach cau dau");
    assert_eq!(
        out.retired.iter().map(|r| r.id).collect::<Vec<_>>(),
        vec![dau],
        "dung MOT hang cu ve huu"
    );
    assert!(out.retired[0].retired_at.is_some(), "va no mang `retired_at` that");
    assert_eq!(out.new_segments.len(), 2, "va HAI manh ra doi");

    let noi_lai = format!("{}{}", out.new_segments[0].source_text, out.new_segments[1].source_text);
    assert_eq!(
        noi_lai, truoc[0].2,
        "hai manh noi lai phai bang DUNG van ban goc -- khong mot ky tu nao roi, khong mot \
         ky tu nao them"
    );
    assert!(
        out.new_segments[0].ord < out.new_segments[1].ord,
        "hai manh dung DUNG THU TU doc"
    );

    // AD-37 ca ③ -- manh CUOI giu cap co, manh truoc tat CA HAI cot.
    assert!(
        !out.new_segments[0].is_paragraph_end && !out.new_segments[0].is_target_paragraph_end,
        "manh TRUOC tat ca hai cot"
    );
    assert!(
        out.new_segments[1].is_target_paragraph_end,
        "manh CUOI giu co DICH cua cau goc -- mot luot cai chi chep co nguon se xoa mat no"
    );

    assert_eq!(out.new_segments[0].status, "draft");
    assert_eq!(out.new_segments[1].status, "draft");

    cleanup(&root);
}

/// **AD-37 ca ①** — gộp hai câu **CUỐI Chương**: cặp cờ của hàng mới **TẮT cả hai**, luôn luôn.
#[test]
fn merging_the_last_two_sentences_of_a_chapter_ends_no_paragraph_in_either_column() {
    use auratranslate_lib::commands::segment::{merge_segments, set_segment_paragraph_end};

    let root = temp_dir("2-8-cuoi");
    let opened = create_work_from_text(&root, "2.8 cuoi", "zh", "", "一。二。三。".to_owned())
        .expect("tao tac pham");

    // Bat co DICH cua cau cuoi bang SQL -- lenh san pham tu choi cau cuoi Chuong (dung the),
    // nen day la cach duy nhat dung mot hinh dang "co dich BAT o cau cuoi" de doi chung.
    opened
        .store
        .write(|tx: &Transaction<'_>| {
            tx.execute(
                "UPDATE segment SET is_target_paragraph_end = 1, is_paragraph_end = 1 WHERE ord = 3",
                [],
            )?;
            Ok(())
        })
        .expect("bom co bang SQL");
    // Cau 2 thi dat bang duong san pham -- no khong phai cau cuoi.
    set_segment_paragraph_end(Some(&opened), 2, true).expect("bat co dich cau 2");

    let out = merge_segments(Some(&opened), 3).expect("gop hai cau CUOI Chuong");
    let moi = &out.new_segments[0];
    assert!(
        !moi.is_paragraph_end && !moi.is_target_paragraph_end,
        "AD-37 ca ① -- segment CUOI Chuong tat CA HAI co, LUON LUON, ke ca khi nguoi dung \
         da tu bat co dich. Mot doan khong the ket thuc sau cau cuoi cung"
    );

    cleanup(&root);
}

/// 🔴 **Mọi lượt từ chối của gộp/tách PHÂN BIỆT ĐƯỢC** — bốn nhánh, bốn `MessageKey`.
///
/// Cùng luật AC14 của Story 2.5 đã đặt cho `confirm_segment`: một câu chung chung gửi người
/// dùng đi sửa nhầm chỗ.
#[test]
fn every_refusal_of_merge_and_split_carries_its_own_message_key() {
    use auratranslate_lib::commands::segment::{merge_segments, split_segment};

    let root = temp_dir("2-8-tu-choi");
    let opened = create_work_from_text(&root, "2.8 tu choi", "zh", "", "一。二。".to_owned())
        .expect("tao tac pham");

    // ① Chua Tac pham nao mo.
    assert_eq!(
        merge_segments(None, 1).unwrap_err().code(),
        "work.none_open"
    );
    assert_eq!(
        split_segment(None, 1, vec![1]).unwrap_err().code(),
        "work.none_open"
    );

    // ② `segment_id` khong co.
    assert_eq!(
        merge_segments(Some(&opened), 9_999).unwrap_err().code(),
        "segment.not_found"
    );
    assert_eq!(
        split_segment(Some(&opened), 9_999, vec![1]).unwrap_err().code(),
        "segment.not_found"
    );

    // ③ Cau DAU Chuong -- khong co cau nao lien tren no. Mot ca THUONG NHAT, khong mot ca bien.
    assert_eq!(
        merge_segments(Some(&opened), 1).unwrap_err().code(),
        "segment.no_previous",
        "gop o cau dau Chuong phai co mot cau NOI RIENG -- no khong phai \"khong tim thay\" \
         va cung khong phai \"da ve huu\""
    );

    // ④ Cho cat de lai mot manh RONG.
    for cut in [0usize, 2, 50] {
        assert_eq!(
            split_segment(Some(&opened), 1, vec![cut]).unwrap_err().code(),
            "segment.cut_leaves_empty_piece",
            "cho cat {cut} de lai mot manh rong ⇒ tu choi"
        );
    }

    // ⑤ Segment DA VE HUU -- duong GHI tu choi. Dung mot hang ve huu THAT.
    let out = merge_segments(Some(&opened), 2).expect("gop de sinh ra hai hang ve huu that");
    for id in out.retired.iter().map(|r| r.id) {
        assert_eq!(
            merge_segments(Some(&opened), id).unwrap_err().code(),
            "segment.retired"
        );
        assert_eq!(
            split_segment(Some(&opened), id, vec![1]).unwrap_err().code(),
            "segment.retired"
        );
    }

    cleanup(&root);
}

/// 🔴 **Không lượt gộp/tách nào ghi một hàng `segment_version`** — AD-31, và một lượt vi
/// phạm nó **không cổng nào đỏ**.
#[test]
fn neither_merge_nor_split_ever_writes_a_segment_version_row() {
    use auratranslate_lib::commands::segment::{confirm_segment, merge_segments, save_segment_targets, split_segment, SegmentTargetEdit};

    let root = temp_dir("2-8-khong-version");
    let opened = create_work_from_text(&root, "2.8 version", "zh", "", "一。二。三。".to_owned())
        .expect("tao tac pham");

    save_segment_targets(
        Some(&opened),
        1,
        &[SegmentTargetEdit {
            id: 1,
            target_text: "Mot.".to_owned(),
        }],
    )
    .expect("ghi ban dich");
    confirm_segment(Some(&opened), 1, "").expect("xac nhan -- luot DUY NHAT duoc phep sinh version");

    let dem = |opened: &auratranslate_lib::commands::project::OpenWork| -> i64 {
        opened
            .store
            .read(|conn| conn.query_row("SELECT COUNT(*) FROM segment_version", [], |r| r.get(0)))
            .expect("dem segment_version")
    };
    let truoc = dem(&opened);
    assert_eq!(truoc, 1, "tien de: dung mot hang lich su ton tai truoc khi gop/tach");

    let out = merge_segments(Some(&opened), 2).expect("gop");
    assert_eq!(dem(&opened), truoc, "mot luot GOP khong sinh them mot hang lich su nao");

    split_segment(Some(&opened), out.new_segments[0].id, vec![2]).expect("tach");
    assert_eq!(dem(&opened), truoc, "va mot luot TACH cung khong");

    cleanup(&root);
}

/// 🔴 **Sau một lượt gộp/tách THẬT, lưới không còn thấy hàng cũ — nhưng ĐĨA thì còn.**
///
/// Ca này canh đúng thứ Ice gặp khi dùng thật ngày 2026-08-17: *"đã tách ra 2 câu, nhưng câu
/// cũ vẫn tồn tại và số thứ tự vẫn chiếm, gây rối nội dung"*. Nó là lượt **lật** chữ ký #6(b)
/// viết thành một phép khẳng định.
///
/// ⚠️ Ca 2.2 ở trên canh cùng mệnh đề nhưng trên một hàng về hưu **bơm bằng SQL**. Ca này đi
/// **đường sản phẩm** — và hai ca không thừa nhau: một cái canh bộ lọc, cái kia canh rằng
/// đường ghi thật sinh ra đúng thứ bộ lọc đang chờ.
#[test]
fn the_grid_stops_showing_a_row_the_moment_a_real_merge_or_split_retires_it() {
    use auratranslate_lib::commands::segment::{merge_segments, split_segment};

    let root = temp_dir("2-8-loc-luoi");
    let opened = create_work_from_text(&root, "2.8 loc", "zh", "", "一。二。三。".to_owned())
        .expect("tao tac pham");

    let dem_tren_dia = || -> i64 {
        opened
            .store
            .read(|conn| conn.query_row("SELECT COUNT(*) FROM segment", [], |r| r.get(0)))
            .expect("dem hang tren dia")
    };

    assert_eq!(read_open_chapter_segments(Some(&opened)).unwrap().segments.len(), 3);
    assert_eq!(dem_tren_dia(), 3);

    // ── GOP: hai hang ve huu, mot hang moi ⇒ luoi 3 → 2, dia 3 → 4 ─────────────────
    let gop = merge_segments(Some(&opened), 2).expect("gop cau 2 voi cau lien tren");
    let sau_gop = read_open_chapter_segments(Some(&opened)).expect("nap lai");
    assert_eq!(
        sau_gop.segments.len(),
        2,
        "gop hai cau ⇒ luoi con HAI hang, khong BA. Mot con so 3 o day la trieu chung nguyen \
         van cua bao cao 2026-08-17: \"cau cu van ton tai va so thu tu van chiem\""
    );
    assert!(
        sau_gop.segments.iter().all(|s| s.retired_at.is_none()),
        "khong mot hang nao di ra luoi mang `retired_at`"
    );
    assert_eq!(
        dem_tren_dia(),
        4,
        "va DIA thi tang: hai hang cu O LAI (ve huu), mot hang moi them vao. Loc khoi LUOI \
         khong phai xoa khoi DIA -- xoa la mat lich su VINH VIEN (AD-5)"
    );

    // AC4 van dung tren chinh hang vua bi loc khoi luoi.
    for row in &gop.retired {
        assert!(row.retired_at.is_some());
        read_segment_history(Some(&opened), row.id)
            .expect("lich su cua mot segment DA VE HUU van tra lai duoc (AC4)");
    }

    // ── TACH: mot hang ve huu, hai manh moi ⇒ luoi 2 → 3, dia 4 → 6 ────────────────
    let id_moi = gop.new_segments[0].id;
    split_segment(Some(&opened), id_moi, vec![2]).expect("tach hang vua gop");
    let sau_tach = read_open_chapter_segments(Some(&opened)).expect("nap lai");
    assert_eq!(
        sau_tach.segments.len(),
        3,
        "tach mot cau ⇒ luoi co BA hang (hai manh + cau con lai), khong BON"
    );
    assert!(sau_tach.segments.iter().all(|s| s.retired_at.is_none()));
    assert_eq!(dem_tren_dia(), 6, "dia: 4 + 2 manh moi");

    // 🔴 Va `ord` cua nhung hang CON SONG lien tuc 1..N -- day la thu lam "so thu tu" dung
    // tro lai. Luoi danh so bang chi so mang, nhung mot `ord` thung lo tren dia se lo ra o
    // dung ngay dau tien co ai doc no.
    assert_eq!(
        sau_tach.segments.iter().map(|s| s.ord).collect::<Vec<_>>(),
        vec![1, 2, 3],
        "`ord` cua cac hang con song phai LIEN TUC 1..N sau moi luot gop/tach"
    );

    cleanup(&root);
}

// ═══════════════════════════════════════════════════════════════════════════════════
// 🔴 STORY 2.12 · AC6 — LƯỚI TRỌN HÀNG CHO ĐƯỜNG GHI **THỨ HAI**
// ═══════════════════════════════════════════════════════════════════════════════════

/// 🔴 **Mọi cột của một hàng `segment` MỚI do `write_regroup` sinh ra phải được đặt CÓ CHỦ Ý.**
///
/// ## Lỗ đã ĐO ĐƯỢC, không một khả năng lý thuyết
///
/// Bảng `segment` có **đúng hai** chỗ `INSERT` (`grep -rn "INSERT INTO segment" src-tauri/src`):
///   · `commands/segment.rs:132`  — đường **nhập**;
///   · `commands/segment.rs:2234` — `write_regroup`, tức **gộp/tách segment**.
///
/// Cổng-AC8 (`a_flush_touches_exactly_target_text_and_updated_at_and_nothing_else`) canh
/// đường **flush**, và chỉ đường đó. Tám ca `regroup` đã có trong tệp này chạy trên **hàm
/// thuần** `core::segment::regroup` — chúng **không chạm đĩa**, nên chúng không thấy câu
/// `INSERT` một lần nào. ⇒ Đường ghi thứ hai **không cổng nào đỏ** nếu nó thiếu một cột.
/// Ghi ở `2-8-gop-va-tach-segment-tuong-minh.md:508`, mở lại thành AC6 của Story 2.12.
///
/// ## Hình dạng hỏng, và vì sao nó im lặng
///
/// Câu `INSERT` ở `:2234` liệt kê **tên cột tường minh**. Một bước di trú thêm cột thứ mười
/// bốn mà không sửa câu đó vẫn **biên dịch sạch và chạy sạch** — SQLite điền `DEFAULT` cho
/// cột không được nêu. Hàng mới ra đời mang một giá trị **không ai quyết định**.
///
/// ⚠️ Và đó không phải một lo xa: cột `translation_origin` chính là ca ấy. Nếu nó rơi về
/// `DEFAULT` trên đường gộp, mọi câu sinh ra bởi một lượt gộp sẽ khai sai xuất xứ — Epic 7
/// gắn nhãn cặp TM theo cột đó, `RagInjector` xếp nó lên đầu, và **không lần ngược được**.
/// AD-47 gọi gộp/tách đích danh trong danh mục đóng *"phải đặt CẢ HAI: mốc so sánh VÀ cột
/// xuất xứ"*.
///
/// ## 🔴 Vì sao ca này KHÔNG THỂ mục lại — cùng cơ chế mà cổng-AC8 dùng
///
/// Nó dựng một `SegmentRow` **trọn hàng** rồi so bằng `assert_eq!`. Ngày cột thứ mười bốn ra
/// đời, `SegmentRow` **buộc phải** nhận một trường nữa *(ca tự kiểm
/// `the_raw_column_reader_sees_every_column_the_segment_table_actually_has` đỏ ngay lượt chạy
/// đầu nếu không)* — và lúc đó dòng dựng `SegmentRow(...)` dưới đây **không biên dịch được**.
///
/// ⇒ Người thêm cột **buộc phải trả lời** *"một hàng do gộp/tách sinh ra mang giá trị nào ở
/// cột này"*, ngay tại lượt di trú. Không ai phải nhớ gì cả. Đây đúng là thứ mà khuôn *"chữ
/// ký thi hành đúng MỘT NỬA"* — đã lặp **năm** lần trong Epic 2 — không cưỡng chế được bằng
/// một lời dặn trong chú thích.
#[test]
fn a_row_born_from_regroup_has_every_column_set_on_purpose_not_by_default() {
    use auratranslate_lib::commands::segment::{
        confirm_segment, merge_segments, save_segment_targets, SegmentTargetEdit,
        TRANSLATION_ORIGIN_SELF,
    };

    let root = temp_dir("2-12-regroup-cot");
    let opened = create_work_from_text(&root, "2.12 cot regroup", "zh", "", "一。二。".to_owned())
        .expect("tao tac pham");

    // Hai cau dau co ban dich that VA da XAC NHAN, va ca hai ve deu bat buoc:
    //   · co ban dich  ⇒ `target_text` cua hang moi KHONG rong, tuc phep khang dinh duoi kia
    //     phan biet duoc "da ghep that" voi "roi ve DEFAULT chuoi rong";
    //   · da xac nhan  ⇒ `translation_origin` cua ca hai thanh `"self"` (AD-47: xuat xu duoc
    //     dat CUNG LUC voi chuyen tiep sang da xac nhan).
    //
    // 🔴 Ve thu hai la ve DUY NHAT phan biet duoc "cot duoc mang qua CO CHU Y" voi "cot roi
    // ve DEFAULT". Bo no di thi ca hai cau nguon deu mang xuat xu rong, `merged_origin` tra
    // ve chuoi rong, va mot cau `INSERT` da bo sot han cot nay cung cho DUNG chuoi rong ay --
    // ca xanh tren mot san pham dang hong. Do la mot bay DA CAN THAT o luot dung ca nay.
    for (id, text) in [(1i64, "Mot."), (2, "Hai.")] {
        save_segment_targets(
            Some(&opened),
            1,
            &[SegmentTargetEdit {
                id,
                target_text: text.to_owned(),
            }],
        )
        .expect("ghi ban dich");
        confirm_segment(Some(&opened), id, "").expect("xac nhan -- day la cho xuat xu duoc dat");
    }

    let out = merge_segments(Some(&opened), 2).expect("gop cau 2 voi cau lien tren no");
    assert_eq!(out.new_segments.len(), 1, "mot luot gop sinh dung MOT hang moi");
    let moi_id = out.new_segments[0].id;

    let rows = read_all_segment_rows(&opened);
    let moi = rows
        .iter()
        .find(|r| r.0 == moi_id)
        .expect("hang moi phai doc lai duoc bang chinh bo doc muoi ba cot");

    // ── LUOI TRON HANG ────────────────────────────────────────────────────────────
    //
    // Ba cot khong the doan truoc mot cach tat dinh, va CHI ba cot do duoc muon lai tu chinh
    // hang doc duoc: `id` (autoincrement) va hai moc thoi gian. Moi cot con lai viet TRAN.
    //
    // ⚠️ `created_at` va `updated_at` muon lai co chu y, va no KHONG lam phep kiem rong: ca
    // rieng ngay duoi khang dinh chung khac `null` va bang nhau -- mot hang VUA sinh chua the
    // co hai moc lech nhau.
    let expected = SegmentRow(
        moi.0,                          // id — autoincrement
        1,                              // chapter_id — cung Chuong
        1,                              // ord — nhom moi chiem `ord` cua cau DAU nhom
        "一。二。".to_owned(),          // source_text — hai cau goc noi lai, AD-4
        0,                              // is_paragraph_end — cau cuoi nhom khong ket doan
        None,                           // retired_at — hang MOI, chua ve huu
        moi.6.clone(),                  // created_at
        moi.7.clone(),                  // updated_at
        "Mot. Hai.".to_owned(),         // target_text — hai ban dich ghep lai
        "draft".to_owned(),             // 🔴 status — AC3 cua 2.8: hang moi CHUA xac nhan,
                                        // KE CA khi ca hai cau nguon DA xac nhan
        0,                              // is_omitted — khong cat bo
        0,                              // is_target_paragraph_end
        TRANSLATION_ORIGIN_SELF.to_owned(), // 🔴 translation_origin — AD-47, KHONG roi ve DEFAULT
    );
    assert_eq!(
        moi, &expected,
        "AC6: moi cot cua mot hang do `write_regroup` sinh ra phai duoc dat CO CHU Y.\n\
         Mot cot lech o day nghia la cau `INSERT` o `commands/segment.rs:2234` da bo sot no \
         va SQLite dien `DEFAULT` -- mot gia tri khong ai quyet dinh, khong mot loi nao duoc \
         nem, va khong mot cong nao khac do."
    );

    // Hai moc thoi gian: mot hang VUA sinh phai co ca hai, va chung phai bang nhau.
    assert!(!moi.6.is_empty(), "`created_at` rong -- cau INSERT bo sot no");
    assert_eq!(
        moi.6, moi.7,
        "mot hang vua sinh khong the co `created_at` khac `updated_at` -- hai `strftime` cua \
         cung mot cau INSERT"
    );

    let dir = opened.dir.clone();
    drop(opened);
    cleanup(&dir);
    cleanup(&root);
}

// ═════════════════════════════════════════════════════════════════════════════════
// Story 5.7 — vị trí làm việc của Chương (`chapter_position`, AD-3). §I/O Matrix.
// ═════════════════════════════════════════════════════════════════════════════════

/// §I/O Matrix "Chương chưa từng mở" — không hàng `chapter_position` nào ⇒ `caret_segment_id`
/// là segment ĐẦU theo `(ord, id)` (AC5), và một lượt ĐỌC không tự ghi một hàng nào.
#[test]
fn a_chapter_never_worked_on_reports_the_first_segment_as_caret_without_writing_a_position_row() {
    let root = temp_dir("5-7-position-never-worked");
    let opened =
        create_work_from_text(&root, "5.7 chua tung mo", "zh", "", "Cau mot。Cau hai。".to_owned())
            .expect("tao tac pham that bai");

    let loaded = read_open_chapter_segments(Some(&opened)).expect("nap segment that bai");
    assert_eq!(
        loaded.caret_segment_id,
        Some(loaded.segments[0].id),
        "AC5: Chuong chua tung mo phai roi caret vao segment DAU theo (ord, id)"
    );

    let rows: i64 = opened
        .store
        .read(|conn| conn.query_row("SELECT COUNT(*) FROM chapter_position", [], |row| row.get(0)))
        .expect("dem hang chapter_position that bai");
    assert_eq!(rows, 0, "mot luot DOC khong duoc phep tu ghi mot hang chapter_position");

    let dir = opened.dir.clone();
    drop(opened);
    cleanup(&dir);
}

/// §I/O Matrix "Ghi vị trí" + "Chương đã từng làm việc" — ghi rồi đọc lại TRONG CÙNG một
/// phiên phải khớp nguyên văn, và `save_chapter_position` là một UPSERT (ghi lần hai GHI ĐÈ,
/// không cộng dồn).
#[test]
fn a_saved_chapter_position_is_reported_back_as_caret_segment_id_and_a_second_save_overwrites_it() {
    let root = temp_dir("5-7-position-roundtrip");
    let opened = create_work_from_text(
        &root,
        "5.7 vi tri",
        "zh",
        "",
        "Cau mot。Cau hai。Cau ba。".to_owned(),
    )
    .expect("tao tac pham that bai");

    let loaded = read_open_chapter_segments(Some(&opened)).expect("nap segment that bai");
    let second = loaded.segments[1].id;
    let third = loaded.segments[2].id;

    save_chapter_position(Some(&opened), loaded.chapter_id, second).expect("ghi vi tri that bai");
    let after_first_save =
        read_open_chapter_segments(Some(&opened)).expect("nap lai sau lan ghi dau");
    assert_eq!(
        after_first_save.caret_segment_id,
        Some(second),
        "vi tri da luu phai doc lai DUNG NGUYEN VAN"
    );

    // UPSERT — ghi lan hai GHI DE, khong cong don thanh hai hang.
    save_chapter_position(Some(&opened), loaded.chapter_id, third).expect("ghi vi tri lan hai");
    let after_second_save =
        read_open_chapter_segments(Some(&opened)).expect("nap lai sau lan ghi hai");
    assert_eq!(after_second_save.caret_segment_id, Some(third));

    let rows: i64 = opened
        .store
        .read(|conn| conn.query_row("SELECT COUNT(*) FROM chapter_position", [], |row| row.get(0)))
        .expect("dem hang chapter_position that bai");
    assert_eq!(rows, 1, "UPSERT phai giu DUNG MOT hang cho moi Chuong, khong cong don");

    let dir = opened.dir.clone();
    drop(opened);
    cleanup(&dir);
}

/// §I/O Matrix "Vị trí trỏ vào segment ĐÃ VỀ HƯU" — rơi về segment ĐẦU còn sống, kèm chẩn
/// đoán (không được trả một id mà lưới không dựng ô cho).
#[test]
fn a_position_pointing_at_a_retired_segment_falls_back_to_the_first_living_segment() {
    use auratranslate_lib::commands::segment::split_segment;

    let root = temp_dir("5-7-position-retired");
    let opened = create_work_from_text(
        &root,
        "5.7 vi tri ve huu",
        "zh",
        "",
        "Cau dau tien that dai de con tach duoc。Cau hai。Cau ba。".to_owned(),
    )
    .expect("tao tac pham that bai");

    let loaded = read_open_chapter_segments(Some(&opened)).expect("nap segment that bai");
    let first_id = loaded.segments[0].id;
    save_chapter_position(Some(&opened), loaded.chapter_id, first_id).expect("ghi vi tri vao cau dau");

    // Tach cau DAU -- no VE HUU, hai manh moi ra doi o dau danh sach (ord nho nhat).
    let cut = loaded.segments[0].source_text.chars().count() / 2;
    let out = split_segment(Some(&opened), first_id, vec![cut]).expect("tach cau dau that bai");
    assert!(
        out.retired.iter().any(|r| r.id == first_id),
        "cau dau phai VE HUU sau lenh tach"
    );

    let after = read_open_chapter_segments(Some(&opened)).expect("nap lai sau khi tach");
    let first_living = after.segments.first().expect("phai con it nhat mot segment song").id;
    assert_eq!(
        after.caret_segment_id,
        Some(first_living),
        "vi tri tro vao mot segment DA VE HUU phai roi ve segment DAU CON SONG, khong tra ve \
         mot id ma luoi khong con dung o cho"
    );
    assert_ne!(
        after.caret_segment_id,
        Some(first_id),
        "segment da ve huu khong duoc tra ve nguyen id cua no"
    );

    let dir = opened.dir.clone();
    drop(opened);
    cleanup(&dir);
}

/// §I/O Matrix "Chương không có segment nào" — `caret_segment_id = None`, không một id bịa.
#[test]
fn an_empty_chapter_reports_no_caret_segment() {
    let root = temp_dir("5-7-position-empty-chapter");
    let opened = create_work_from_text(&root, "5.7 chuong rong", "en", "", "One.".to_owned())
        .expect("tao tac pham that bai");

    let chapter_id = opened.chapter_id;
    opened
        .store
        .write(move |tx: &Transaction<'_>| {
            tx.execute("DELETE FROM segment WHERE chapter_id = ?1", [chapter_id])?;
            Ok(())
        })
        .expect("xoa segment that bai");

    let loaded = read_open_chapter_segments(Some(&opened)).expect("nap segment that bai");
    assert!(loaded.segments.is_empty());
    assert_eq!(
        loaded.caret_segment_id, None,
        "Chuong khong con segment nao -- caret phai la None, khong mot id bia"
    );

    let dir = opened.dir.clone();
    drop(opened);
    cleanup(&dir);
}

/// §I/O Matrix "Ghi vị trí cho Chương không thuộc kho" — không hàng nào được ghi, lỗi CÓ TÊN
/// tái dùng (`segment.chapter_not_found`).
#[test]
fn saving_a_chapter_position_for_an_unknown_chapter_is_refused_and_writes_nothing() {
    let root = temp_dir("5-7-position-unknown-chapter");
    let opened = create_work_from_text(&root, "5.7 chuong la", "en", "", "One.".to_owned())
        .expect("tao tac pham that bai");

    let bogus_chapter_id = opened.chapter_id + 999;
    let err = save_chapter_position(Some(&opened), bogus_chapter_id, 1)
        .expect_err("chapter_id la phai la mot loi");
    assert_eq!(err.code(), "segment.chapter_not_found");
    assert_eq!(err.message_key(), MessageKey::SegmentChapterNotFound);

    let rows: i64 = opened
        .store
        .read(|conn| conn.query_row("SELECT COUNT(*) FROM chapter_position", [], |row| row.get(0)))
        .expect("dem hang chapter_position that bai");
    assert_eq!(rows, 0, "khong hang nao duoc ghi khi chapter_id sai");

    let dir = opened.dir.clone();
    drop(opened);
    cleanup(&dir);
}

/// Chưa Tác phẩm nào mở ⇒ `save_chapter_position` trả `project.no_work_open` — cùng khoá
/// tái dùng với mọi lệnh Chương/segment khác.
#[test]
fn saving_a_chapter_position_without_an_open_work_reuses_the_named_error() {
    let err =
        save_chapter_position(None, 1, 1).expect_err("chua Tac pham nao mo phai la mot loi");
    assert_eq!(err.code(), "work.none_open");
    assert_eq!(err.message_key(), MessageKey::WorkNoneOpen);
}

// ═════════════════════════════════════════════════════════════════════════════════
// Story 5.8 — đóng mục `deferred` #1 của Story 5.7: `save_chapter_position` nay kiểm CẶP
// `(chapter_id, segment_id)`, không chỉ `chapter_id` tồn tại.
// ═════════════════════════════════════════════════════════════════════════════════

/// §I/O Matrix "Ghi vị trí lệch cặp" — `segment_id` tồn tại thật, nhưng thuộc một Chương
/// KHÁC `chapter_id` được chỉ ⇒ **0 hàng ghi**, `segment.not_found` (đóng mục `deferred` #1
/// của Story 5.7).
#[test]
fn saving_a_position_whose_segment_belongs_to_another_chapter_writes_nothing() {
    let root = temp_dir("5-8-position-cross-chapter");
    let opened = create_work_from_text(&root, "5.8 cap lech", "zh", "", "一。".to_owned())
        .expect("tao tac pham that bai");
    let chapter_a = opened.chapter_id;

    // Chương thứ hai, cùng khuôn `a_segment_id_from_another_chapter_is_refused_and_never_crosses_over`.
    let chapter_b: i64 = opened
        .store
        .write(move |tx: &Transaction<'_>| {
            tx.execute(
                "INSERT INTO chapter (ord, title, source_text, status, created_at, updated_at) \
                 SELECT 2, 'Chuong hai', 'Nhi。', status, created_at, updated_at \
                 FROM chapter WHERE id = ?1",
                [chapter_a],
            )?;
            let other: i64 = tx.query_row("SELECT id FROM chapter WHERE ord = 2", [], |r| r.get(0))?;
            tx.execute(
                "INSERT INTO segment (chapter_id, ord, source_text, is_paragraph_end, created_at, updated_at) \
                 VALUES (?1, 1, 'Nhi。', 0, strftime('%Y-%m-%dT%H:%M:%fZ','now'), \
                 strftime('%Y-%m-%dT%H:%M:%fZ','now'))",
                [other],
            )?;
            Ok(other)
        })
        .expect("bom Chuong thu hai that bai");

    let segment_of_b: i64 = opened
        .store
        .read(move |conn| conn.query_row("SELECT id FROM segment WHERE chapter_id = ?1", [chapter_b], |r| r.get(0)))
        .expect("doc segment cua Chuong B that bai");

    // `chapter_a` CÓ THẬT, `segment_of_b` CÓ THẬT -- nhưng chúng KHÔNG PHẢI một cặp: đúng
    // hình dạng cặp lệch mà lượt tách của Story 5.8 sinh ra được (segment ĐỔI `chapter_id`).
    let err = save_chapter_position(Some(&opened), chapter_a, segment_of_b)
        .expect_err("mot cap (chapter_id, segment_id) lech phai bi tu choi");
    assert_eq!(err.code(), "segment.not_found");
    assert_eq!(err.message_key(), MessageKey::SegmentNotFound);

    let rows: i64 = opened
        .store
        .read(|conn| conn.query_row("SELECT COUNT(*) FROM chapter_position", [], |row| row.get(0)))
        .expect("dem hang chapter_position that bai");
    assert_eq!(rows, 0, "khong hang nao duoc ghi khi cap (chapter_id, segment_id) lech nhau");

    let dir = opened.dir.clone();
    drop(opened);
    cleanup(&dir);
}

/// §I/O Matrix "Vị trí làm việc theo câu qua lượt tách" — `chapter_position(A) → s` và `s`
/// dời sang B ⇒ hàng vị trí dời THEO, cùng câu, đúng Chương. `save_chapter_position` trên cặp
/// CŨ `(A, s)` bị từ chối sau lượt tách vì `s` không còn thuộc A nữa.
#[test]
fn a_split_moves_the_remembered_position_row_to_the_new_chapter() {
    let root = temp_dir("5-8-split-moves-position");
    let mut opened = create_work_from_text(
        &root,
        "5.8 vi tri theo tach",
        "zh",
        "",
        "Cau mot。Cau hai。Cau ba。".to_owned(),
    )
    .expect("tao tac pham that bai");
    let chapter_a = opened.chapter_id;

    let loaded = read_open_chapter_segments(Some(&opened)).expect("nap segment that bai");
    assert_eq!(loaded.segments.len(), 3, "fixture phai co dung ba cau");
    let third = loaded.segments[2].id;

    save_chapter_position(Some(&opened), chapter_a, third).expect("ghi vi tri vao cau ba");

    split_chapter_at_segment(Some(&mut opened), third).expect("tach that bai");

    // Chương MỚI B nhận câu 3 (điểm cắt) -- đọc lại chapter_id thật của nó.
    let chapter_b: i64 = opened
        .store
        .read(move |conn| conn.query_row("SELECT chapter_id FROM segment WHERE id = ?1", [third], |r| r.get(0)))
        .expect("doc lai chapter_id cua cau ba sau tach");
    assert_ne!(chapter_b, chapter_a, "cau ba la diem cat -- phai doi sang Chuong moi");

    let (position_chapter, position_segment): (i64, i64) = opened
        .store
        .read(|conn| conn.query_row("SELECT chapter_id, segment_id FROM chapter_position", [], |row| Ok((row.get(0)?, row.get(1)?))))
        .expect("doc hang chapter_position that bai");
    assert_eq!(position_chapter, chapter_b, "hang vi tri phai doi theo sang Chuong moi");
    assert_eq!(position_segment, third, "van cung mot cau");

    // Ghi lai tren CAP CU (A, cau ba) sau khi da tach phai bi TU CHOI -- cau ba khong con
    // thuoc A nua.
    let err = save_chapter_position(Some(&opened), chapter_a, third)
        .expect_err("cap CU (A, cau ba) sau lot tach phai bi tu choi");
    assert_eq!(err.code(), "segment.not_found");

    let dir = opened.dir.clone();
    drop(opened);
    cleanup(&dir);
}

// ═════════════════════════════════════════════════════════════════════════════════
// 🔴 STORY 5.11/5.12 — `read_reading_run` VÀ `core::segment::reading::paragraphs_in_translation`
// ═════════════════════════════════════════════════════════════════════════════════
// 🔵 SỬA 2026-08-30 (Story 5.12) — khối này trước gọi `read_reading_chapter` (Story 5.11),
// đọc một Chương ĐƠN bất kể `chapter.status`. Đường đó nay là `read_reading_run`, đọc một
// LƯỢT ĐỌC chỉ gồm các Chương `Done` — mọi ca dưới đây SỬA TẠI CHỖ: mỗi fixture đặt Chương
// đang đọc thành `Done` (`set_chapter_status`) trước khi gọi, và đọc `run.chapters[0]` thay
// vì đọc thẳng một `ReadingChapter`. Khối ca MỚI (từ `a_continuous_run_stops_before_...`
// trở xuống) phủ trọn §I/O Matrix của Story 5.12 — mốc biên, giá trị lạ, `is_confirmed`,
// `segment_count`.
//
// Fixture chung cho nhóm gom đoạn: "一。二。\n三。四。\n五。" -- ba đoạn nguồn (2 câu, 2 câu,
// 1 câu), tách bằng `。` ở tầng CÂU và bằng `\n` ở tầng ĐOẠN (`core::segment::split`). AC2
// (Story 2.5d) đảm bảo cờ đích SOI GƯƠNG cờ nguồn lúc nhập, nên năm câu này ra khỏi
// `create_work_from_text` với `is_target_paragraph_end` = `[false, true, false, true, false]`
// -- đúng khuôn I/O Matrix của Story 5.11 ("câu 2 và 4 mang is_target_paragraph_end = 1").

/// Ids theo đúng thứ tự `(ord, id)` -- cùng thứ tự mà `read_reading_run` phải giữ.
fn ordered_ids(open: &auratranslate_lib::commands::project::OpenWork) -> Vec<i64> {
    open.store
        .read(|conn| {
            let mut stmt = conn.prepare("SELECT id FROM segment ORDER BY ord, id")?;
            let rows = stmt.query_map([], |row| row.get::<_, i64>(0))?;
            rows.collect::<Result<Vec<_>, _>>()
        })
        .expect("doc danh sach id segment that bai")
}

fn set_omitted(open: &auratranslate_lib::commands::project::OpenWork, ids: &[i64]) {
    for &id in ids {
        open.store
            .write(move |tx: &Transaction<'_>| {
                tx.execute("UPDATE segment SET is_omitted = 1 WHERE id = ?1", [id])?;
                Ok(())
            })
            .expect("dat is_omitted = 1 that bai");
    }
}

/// **THÊM Story 5.12.** Chương ĐANG MỞ chỉ đi vào dãy đọc khi nó `Done` — mọi fixture của
/// nhóm gom đoạn (kế thừa từ Story 5.11) phải đặt trạng thái này TRƯỚC khi gọi
/// `read_reading_run`. Đi qua `set_chapter_status` — đường DUY NHẤT ghi `chapter.status`
/// (Code Map của story), không một `UPDATE` tay.
fn make_done(open: &mut auratranslate_lib::commands::project::OpenWork, chapter_id: i64) {
    set_chapter_status(Some(open), chapter_id, "done").expect("dat trang thai done that bai");
}

/// **THÊM Story 5.12.** Đặt `chapter.status` bằng SQL TRỰC TIẾP — dùng CHỈ cho ca "trạng
/// thái lạ" (một giá trị ngoài bốn giá trị của `LifecycleStatus`): `set_chapter_status` từ
/// chối một giá trị như vậy ở tầng Rust (§Always), nên không đường sản phẩm nào ghi được nó
/// — đúng cách bộ ca của Story 5.4 đã dựng ca tương tự ("Ca hợp đồng dựng trạng thái đó bằng
/// SQL trực tiếp").
fn set_chapter_status_raw(open: &auratranslate_lib::commands::project::OpenWork, chapter_id: i64, status: &str) {
    let status = status.to_owned();
    open.store
        .write(move |tx: &Transaction<'_>| tx.execute("UPDATE chapter SET status = ?1 WHERE id = ?2", (status, chapter_id)))
        .expect("dat status Chuong bang SQL truc tiep that bai");
}

/// **THÊM Story 5.12.** Đặt `segment.status` bằng SQL TRỰC TIẾP — chỉ dùng để dựng fixture
/// "câu đã xác nhận"/"câu chưa xác nhận" mà không phải lo điều kiện `target_text` không rỗng
/// của `confirm_segment` (AC13/AC14 của Story 2.7, không liên quan gì tới story này).
fn set_segment_status_directly(open: &auratranslate_lib::commands::project::OpenWork, segment_id: i64, status: &str) {
    let status = status.to_owned();
    open.store
        .write(move |tx: &Transaction<'_>| tx.execute("UPDATE segment SET status = ?1 WHERE id = ?2", (status, segment_id)))
        .expect("dat status segment truc tiep that bai");
}

/// **THÊM Story 5.12.** Xoá hẳn một hàng `chapter` bằng SQL TRỰC TIẾP — dựng ca "hàng Chương
/// biến mất" của §I/O Matrix; không đường sản phẩm nào xoá Chương hôm nay.
fn delete_chapter_row_directly(open: &auratranslate_lib::commands::project::OpenWork, chapter_id: i64) {
    open.store
        .write(move |tx: &Transaction<'_>| tx.execute("DELETE FROM chapter WHERE id = ?1", [chapter_id]))
        .expect("xoa hang chapter truc tiep that bai");
}

/// Rút gọn `paragraphs` của MỘT `ReadingChapter` về một dạng dễ so sánh: mỗi đoạn là danh
/// sách các CHỈ SỐ (1-based, theo `ordered_ids`) của các câu còn sống trong nó -- vị trí thật
/// của id, không giá trị `id` tuyệt đối (id là `AUTOINCREMENT`, khác nhau giữa các Tác phẩm).
fn paragraph_shapes(
    ids: &[i64],
    chapter: &auratranslate_lib::commands::segment::ReadingChapter,
) -> Vec<Vec<usize>> {
    chapter
        .paragraphs
        .iter()
        .map(|p| {
            p.segments
                .iter()
                .map(|s| ids.iter().position(|&id| id == s.id).expect("id la thanh vien") + 1)
                .collect()
        })
        .collect()
}

/// **Ca thường nhất** — ba đoạn cắt đúng theo cờ ĐÍCH, không một câu nào bị cắt bỏ.
#[test]
fn a_chapter_with_no_omissions_groups_into_paragraphs_by_the_target_flag() {
    let root = temp_dir("5-11-basic-grouping");
    let mut opened = create_work_from_text(&root, "5.11 Co ban", "zh", "", "一。二。\n三。四。\n五。".to_owned())
        .expect("tao tac pham that bai");
    let chapter_id_to_mark_done = opened.chapter_id;
    make_done(&mut opened, chapter_id_to_mark_done);

    let ids = ordered_ids(&opened);
    assert_eq!(ids.len(), 5, "fixture phai co dung nam cau");

    let run = read_reading_run(Some(&opened)).expect("doc luot doc that bai");
    assert_eq!(run.chapters.len(), 1, "mot Tac pham mot Chuong done ⇒ dung mot Chuong trong day");
    let chapter = &run.chapters[0];
    assert_eq!(chapter.chapter_id, opened.chapter_id);
    assert_eq!(chapter.chapter_ord, 1, "Chuong duy nhat cua mot Tac pham moi tao mang ord = 1");
    assert_eq!(
        paragraph_shapes(&ids, chapter),
        vec![vec![1, 2], vec![3, 4], vec![5]],
        "ba doan phai cat dung theo co dich, khop I/O Matrix cua story"
    );
    assert_eq!(
        run.frontier.kind,
        ReadingFrontierKind::EndOfWork,
        "mot Chuong duy nhat, da done ⇒ het Tac pham, khong Chuong nao chan"
    );

    let dir = opened.dir.clone();
    drop(opened);
    cleanup(&dir);
}

/// **Câu đã cắt bỏ vắng mặt hoàn toàn** — không chỗ trống, không `[…]`, không phần tử rỗng.
#[test]
fn an_omitted_sentence_leaves_no_trace_in_the_reading_paragraphs() {
    let root = temp_dir("5-11-omitted-leaves-no-trace");
    let mut opened = create_work_from_text(&root, "5.11 Cat bo", "zh", "", "一。二。\n三。四。\n五。".to_owned())
        .expect("tao tac pham that bai");
    let chapter_id_to_mark_done = opened.chapter_id;
    make_done(&mut opened, chapter_id_to_mark_done);

    let ids = ordered_ids(&opened);
    // Cau thu ba: giua doan hai, KHONG mang co ket doan -- cat bo no khong dung cham toi
    // ranh gioi doan nao ca.
    set_omitted(&opened, &[ids[2]]);

    let run = read_reading_run(Some(&opened)).expect("doc luot doc that bai");
    let chapter = &run.chapters[0];
    assert_eq!(
        paragraph_shapes(&ids, chapter),
        vec![vec![1, 2], vec![4], vec![5]],
        "cau 3 phai bien mat hoan toan, khong doan rong nao lot ra day"
    );
    for paragraph in &chapter.paragraphs {
        assert!(
            paragraph.segments.iter().all(|s| s.id != ids[2]),
            "id cua cau da cat bo khong duoc xuat hien o bat ky doan nao"
        );
    }

    let dir = opened.dir.clone();
    drop(opened);
    cleanup(&dir);
}

/// **Cờ kết đoạn nằm TRÊN chính câu bị cắt bỏ vẫn giữ được ranh giới** — cờ chuyển cho câu
/// còn sống liền trước; hai đoạn không bị gộp làm một. Đây là ca biên trung tâm của Task 1.
#[test]
fn a_paragraph_end_flag_on_an_omitted_sentence_still_closes_the_paragraph() {
    let root = temp_dir("5-11-flag-transfers-on-omission");
    let mut opened = create_work_from_text(&root, "5.11 Co chuyen", "zh", "", "一。二。\n三。四。\n五。".to_owned())
        .expect("tao tac pham that bai");
    let chapter_id_to_mark_done = opened.chapter_id;
    make_done(&mut opened, chapter_id_to_mark_done);

    let ids = ordered_ids(&opened);
    // Cau thu hai: mang co ket doan CUA CHINH NO (dong doan mot) -- cat bo no.
    set_omitted(&opened, &[ids[1]]);

    let run = read_reading_run(Some(&opened)).expect("doc luot doc that bai");
    let chapter = &run.chapters[0];
    assert_eq!(
        paragraph_shapes(&ids, chapter),
        vec![vec![1], vec![3, 4], vec![5]],
        "doan phai ket sau cau 1 -- co chuyen cho cau con song lien truoc, KHONG gop doan \
         mot va doan hai lam mot"
    );

    let dir = opened.dir.clone();
    drop(opened);
    cleanup(&dir);
}

/// **Cả một đoạn bị cắt bỏ** ⇒ đoạn đó biến mất trọn vẹn, không một đoạn rỗng nào lọt ra dây.
#[test]
fn an_entirely_omitted_paragraph_produces_no_empty_paragraph() {
    let root = temp_dir("5-11-whole-paragraph-omitted");
    let mut opened = create_work_from_text(&root, "5.11 Ca doan", "zh", "", "一。二。\n三。四。\n五。".to_owned())
        .expect("tao tac pham that bai");
    let chapter_id_to_mark_done = opened.chapter_id;
    make_done(&mut opened, chapter_id_to_mark_done);

    let ids = ordered_ids(&opened);
    // Doan hai tron ven: cau ba VA cau bon.
    set_omitted(&opened, &[ids[2], ids[3]]);

    let run = read_reading_run(Some(&opened)).expect("doc luot doc that bai");
    let chapter = &run.chapters[0];
    assert_eq!(
        paragraph_shapes(&ids, chapter),
        vec![vec![1, 2], vec![5]],
        "doan giua phai bien mat TRON VEN -- khong mot doan rong nao dai dien cho no"
    );
    assert!(
        chapter.paragraphs.iter().all(|p| !p.segments.is_empty()),
        "khong doan nao duoc phep rong"
    );

    let dir = opened.dir.clone();
    drop(opened);
    cleanup(&dir);
}

/// **Chưa mở Tác phẩm nào** ⇒ `err.work.none_open` — cùng khoá mà `read_open_chapter_segments`
/// đã dùng, không một khoá thứ hai cho cùng một câu (§Design Notes của story: tái dùng
/// `no_work_open()`).
#[test]
fn reading_a_run_without_an_open_work_fails_with_work_none_open() {
    let err = read_reading_run(None).expect_err("khong Tac pham nao mo phai bi tu choi");
    assert_eq!(err.code(), "work.none_open");
}

/// **Câu CHƯA DỊCH vẫn ở nguyên trong đoạn** — hàng *"Câu chưa dịch"* của I/O Matrix Story 5.11.
///
/// 🔴 Vì sao ca này tồn tại riêng, khi năm ca trên đã kiểm CẤU TRÚC đoạn: cả năm ca ấy so
/// `paragraph_shapes` -- một phép so theo **vị trí**, mù với nội dung. Một lượt cài "bỏ qua câu
/// rỗng cho trang đọc sạch" sẽ đi qua trọn vẹn năm ca đó. `target_text` rỗng nghĩa là *chưa dịch*,
/// **không** phải *đã cắt bỏ*: hai trục độc lập (`commands/segment.rs` §`is_omitted`), và Chế độ
/// đọc không được tự bịa ra nội dung lẫn tự giấu một câu người dùng chưa làm tới.
#[test]
fn an_untranslated_sentence_stays_in_its_paragraph_with_an_empty_string() {
    let root = temp_dir("5-11-untranslated-stays");
    let mut opened = create_work_from_text(&root, "5.11 Chua dich", "zh", "", "一。二。\n三。四。\n五。".to_owned())
        .expect("tao tac pham that bai");
    let chapter_id_to_mark_done = opened.chapter_id;
    make_done(&mut opened, chapter_id_to_mark_done);

    let ids = ordered_ids(&opened);
    // Dich cau 1 va cau 2, DE NGUYEN cau 3 chua dich (chuoi rong) trong cung doan voi cau 4.
    flush_segment_targets(
        Some(&opened),
        opened.chapter_id,
        &[edit(ids[0], "Mot."), edit(ids[1], "Hai."), edit(ids[3], "Bon.")],
    )
    .expect("ghi ban dich that bai");

    let run = read_reading_run(Some(&opened)).expect("doc luot doc that bai");
    let chapter = &run.chapters[0];
    assert_eq!(
        paragraph_shapes(&ids, chapter),
        vec![vec![1, 2], vec![3, 4], vec![5]],
        "cau chua dich KHONG duoc bien mat -- cau truc doan phai y het ca khong cat bo nao"
    );

    let third = chapter
        .paragraphs
        .iter()
        .flat_map(|p| p.segments.iter())
        .find(|s| s.id == ids[2])
        .expect("cau 3 phai co mat tren day");
    assert_eq!(third.target_text, "", "cau chua dich mang chuoi RONG, khong mot noi dung bia ra");

    let fourth = chapter
        .paragraphs
        .iter()
        .flat_map(|p| p.segments.iter())
        .find(|s| s.id == ids[3])
        .expect("cau 4 phai co mat tren day");
    assert_eq!(fourth.target_text, "Bon.", "cau da dich mang dung ban dich da ghi");

    let dir = opened.dir.clone();
    drop(opened);
    cleanup(&dir);
}

// ═════════════════════════════════════════════════════════════════════════════════
// 🔴 STORY 5.12 — MỐC BIÊN (`ReadingFrontier`), GIÁ TRỊ LẠ, `is_confirmed`, `segment_count`
// ═════════════════════════════════════════════════════════════════════════════════
// Mỗi tên hàm dưới đây là MỘT hàng của §I/O & Edge-Case Matrix — xem tên hàng tương ứng
// trong doc-comment kèm theo.

/// §I/O Matrix "Đọc liên tục" — dãy BA Chương `done` liên tiếp, dừng TRƯỚC Chương thứ tư
/// `in_progress`; `frontier.kind = "next-not-done"`, `frontier.chapter` nêu đích danh Chương
/// thứ tư.
///
/// Dựng bốn Chương một-câu-mỗi-Chương bằng BA lượt `split_chapter_at_segment` liên tiếp,
/// dời con trỏ `opened.chapter_id` sang nửa SAU sau mỗi lượt (`split_chapter_at_segment` GIỮ
/// NGUYÊN con trỏ ở nửa TRƯỚC — xem doc-comment của nó — nên phải tự dời để lượt tách kế tiếp
/// nhắm đúng Chương chứa segment cần cắt).
#[test]
fn a_continuous_run_stops_before_a_chapter_that_is_not_yet_done() {
    let root = temp_dir("5-12-continuous-run-stops-at-not-done");
    let mut opened = create_work_from_text(&root, "5.12 Lien tuc", "zh", "", "一。\n二。\n三。\n四。".to_owned())
        .expect("tao tac pham that bai");
    let chapter_a = opened.chapter_id;

    let ids = ordered_ids(&opened);
    assert_eq!(ids.len(), 4, "fixture phai co dung bon cau, mot cau mot Chuong sau khi tach");

    let (id2, id3, id4) = (ids[1], ids[2], ids[3]);

    // Tach lan 1: A = [cau 1], moi = [cau 2, cau 3, cau 4].
    split_chapter_at_segment(Some(&mut opened), id2).expect("tach lan 1 that bai");
    let chapter_b: i64 = opened
        .store
        .read(move |conn| conn.query_row("SELECT chapter_id FROM segment WHERE id = ?1", [id2], |r| r.get(0)))
        .expect("doc chapter_id cua cau 2 sau tach lan 1");
    opened.chapter_id = chapter_b;

    // Tach lan 2: B = [cau 2], moi = [cau 3, cau 4].
    split_chapter_at_segment(Some(&mut opened), id3).expect("tach lan 2 that bai");
    let chapter_c: i64 = opened
        .store
        .read(move |conn| conn.query_row("SELECT chapter_id FROM segment WHERE id = ?1", [id3], |r| r.get(0)))
        .expect("doc chapter_id cua cau 3 sau tach lan 2");
    opened.chapter_id = chapter_c;

    // Tach lan 3: C = [cau 3], moi = [cau 4].
    split_chapter_at_segment(Some(&mut opened), id4).expect("tach lan 3 that bai");
    let chapter_d: i64 = opened
        .store
        .read(move |conn| conn.query_row("SELECT chapter_id FROM segment WHERE id = ?1", [id4], |r| r.get(0)))
        .expect("doc chapter_id cua cau 4 sau tach lan 3");

    make_done(&mut opened, chapter_a);
    make_done(&mut opened, chapter_b);
    make_done(&mut opened, chapter_c);
    set_chapter_status(Some(&mut opened), chapter_d, "in_progress").expect("dat in_progress that bai");

    opened.chapter_id = chapter_a;
    let run = read_reading_run(Some(&opened)).expect("doc luot doc that bai");

    assert_eq!(run.chapters.len(), 3, "dung ba Chuong done duoc dua vao day");
    assert_eq!(
        run.chapters.iter().map(|c| c.chapter_id).collect::<Vec<_>>(),
        vec![chapter_a, chapter_b, chapter_c],
        "day phai theo dung thu tu (ord, id), bat dau TAI Chuong dang mo"
    );
    assert_eq!(run.frontier.kind, ReadingFrontierKind::NextNotDone);
    let frontier_chapter = run.frontier.chapter.as_ref().expect("kind == NextNotDone ⇒ chapter phai Some");
    assert_eq!(frontier_chapter.chapter_id, chapter_d, "moc bien phai neu dich danh Chuong thu tu");
    assert_eq!(frontier_chapter.status, "in_progress");

    let dir = opened.dir.clone();
    drop(opened);
    cleanup(&dir);
}

/// §I/O Matrix "Chạm biên ngay" — Chương đang mở CHƯA `done` ⇒ `chapters` rỗng, mốc biên trỏ
/// vào CHÍNH Chương đang mở.
#[test]
fn opening_a_not_done_chapter_yields_an_empty_run_with_the_frontier_on_itself() {
    let root = temp_dir("5-12-frontier-on-self");
    let mut opened = create_work_from_text(&root, "5.12 Cham bien", "zh", "", "一。二。".to_owned())
        .expect("tao tac pham that bai");
    let chapter_id = opened.chapter_id;
    set_chapter_status(Some(&mut opened), chapter_id, "in_progress").expect("dat in_progress that bai");

    let run = read_reading_run(Some(&opened)).expect("doc luot doc that bai");
    assert!(run.chapters.is_empty(), "Chuong dang mo chua done ⇒ khong Chuong nao vao day");
    assert_eq!(run.frontier.kind, ReadingFrontierKind::NextNotDone);
    let frontier_chapter = run.frontier.chapter.as_ref().expect("kind == NextNotDone ⇒ chapter phai Some");
    assert_eq!(frontier_chapter.chapter_id, chapter_id, "moc bien phai tro vao CHINH Chuong dang mo");
    assert_eq!(frontier_chapter.status, "in_progress");

    let dir = opened.dir.clone();
    drop(opened);
    cleanup(&dir);
}

/// §I/O Matrix "Hết Tác phẩm" — mọi Chương từ Chương đang mở tới cuối đều `done` ⇒
/// `frontier.kind = "end-of-work"`, `frontier.chapter = None`.
#[test]
fn every_remaining_chapter_being_done_yields_an_end_of_work_frontier() {
    let root = temp_dir("5-12-end-of-work");
    let mut opened = create_work_from_text(&root, "5.12 Het tac pham", "zh", "", "一。\n二。".to_owned())
        .expect("tao tac pham that bai");
    let chapter_a = opened.chapter_id;

    let ids = ordered_ids(&opened);
    split_chapter_at_segment(Some(&mut opened), ids[1]).expect("tach that bai");
    let chapter_b: i64 = opened
        .store
        .read(move |conn| conn.query_row("SELECT chapter_id FROM segment WHERE id = ?1", [ids[1]], |r| r.get(0)))
        .expect("doc chapter_id cua cau 2 sau tach");

    make_done(&mut opened, chapter_a);
    make_done(&mut opened, chapter_b);
    opened.chapter_id = chapter_a;

    let run = read_reading_run(Some(&opened)).expect("doc luot doc that bai");
    assert_eq!(run.chapters.len(), 2, "ca hai Chuong done deu phai vao day");
    assert_eq!(run.frontier.kind, ReadingFrontierKind::EndOfWork);
    assert!(run.frontier.chapter.is_none(), "kind == EndOfWork ⇒ chapter phai None");

    let dir = opened.dir.clone();
    drop(opened);
    cleanup(&dir);
}

/// §I/O Matrix "Trạng thái lạ" — Chương kế mang `status = "finished"` (ngoài bốn giá trị của
/// `LifecycleStatus`) ⇒ dãy DỪNG TRƯỚC nó, và `frontier.chapter.status` mang NGUYÊN VĂN giá
/// trị lạ đó — không đoán, không rơi vào một nhãn nào đó.
#[test]
fn an_unknown_status_value_blocks_the_run_and_travels_verbatim_on_the_frontier() {
    let root = temp_dir("5-12-unknown-status");
    let mut opened = create_work_from_text(&root, "5.12 Trang thai la", "zh", "", "一。\n二。".to_owned())
        .expect("tao tac pham that bai");
    let chapter_a = opened.chapter_id;
    make_done(&mut opened, chapter_a);

    let ids = ordered_ids(&opened);
    split_chapter_at_segment(Some(&mut opened), ids[1]).expect("tach that bai");
    let chapter_b: i64 = opened
        .store
        .read(move |conn| conn.query_row("SELECT chapter_id FROM segment WHERE id = ?1", [ids[1]], |r| r.get(0)))
        .expect("doc chapter_id cua cau 2 sau tach");
    // "finished" NGOAI bon gia tri cua LifecycleStatus -- set_chapter_status se tu choi no,
    // nen day la ca DUY NHAT trong khoi nay phai dat status bang SQL truc tiep.
    set_chapter_status_raw(&opened, chapter_b, "finished");
    opened.chapter_id = chapter_a;

    let run = read_reading_run(Some(&opened)).expect("doc luot doc that bai");
    assert_eq!(run.chapters.len(), 1, "day phai dung truoc Chuong mang gia tri la");
    assert_eq!(run.frontier.kind, ReadingFrontierKind::NextNotDone);
    let frontier_chapter = run.frontier.chapter.as_ref().expect("kind == NextNotDone ⇒ chapter phai Some");
    assert_eq!(frontier_chapter.chapter_id, chapter_b);
    assert_eq!(
        frontier_chapter.status, "finished",
        "mot gia tri la phai di ra day NGUYEN VAN, khong duoc doan thanh mot trang thai da biet"
    );

    let dir = opened.dir.clone();
    drop(opened);
    cleanup(&dir);
}

/// §I/O Matrix "Câu chưa xác nhận" — Chương `done`, ba câu `draft` (chưa xác nhận) giữa các
/// câu `confirmed` ⇒ đúng ba câu ấy mang `is_confirmed = false`, các câu còn lại
/// `is_confirmed = true` — dấu hiệu đến từ CHÍNH `segment.status`, không một phép đoán.
#[test]
fn unconfirmed_sentences_in_a_done_chapter_carry_is_confirmed_false() {
    let root = temp_dir("5-12-unconfirmed-sentences");
    let mut opened = create_work_from_text(&root, "5.12 Chua xac nhan", "zh", "", "一。二。三。四。五。".to_owned())
        .expect("tao tac pham that bai");
    let chapter_id_to_mark_done = opened.chapter_id;
    make_done(&mut opened, chapter_id_to_mark_done);

    let ids = ordered_ids(&opened);
    assert_eq!(ids.len(), 5, "fixture phai co dung nam cau");
    // Cau 1, 3, 5 xac nhan; cau 2 va 4 (mac dinh 'draft' luc tao) giu nguyen CHUA xac nhan.
    set_segment_status_directly(&opened, ids[0], SEGMENT_STATUS_CONFIRMED);
    set_segment_status_directly(&opened, ids[2], SEGMENT_STATUS_CONFIRMED);
    set_segment_status_directly(&opened, ids[4], SEGMENT_STATUS_CONFIRMED);

    let run = read_reading_run(Some(&opened)).expect("doc luot doc that bai");
    let all_segments: Vec<_> = run.chapters[0].paragraphs.iter().flat_map(|p| p.segments.iter()).collect();
    assert_eq!(all_segments.len(), 5);
    for segment in &all_segments {
        let should_be_confirmed = segment.id == ids[0] || segment.id == ids[2] || segment.id == ids[4];
        assert_eq!(
            segment.is_confirmed, should_be_confirmed,
            "segment id={} phai mang is_confirmed={should_be_confirmed}",
            segment.id
        );
    }

    let dir = opened.dir.clone();
    drop(opened);
    cleanup(&dir);
}

/// **THÊM (lượt rà 2026-08-30, Bản vá 10)** — hai bộ lọc chạy trên CÙNG một dãy segment
/// (`omit::segments_in_translation` cắt câu đã cắt bỏ; `is_confirmed` đánh dấu câu chưa ký)
/// nhưng trước ca này chỉ được kiểm RỜI NHAU, mỗi ca một fixture riêng. Ca này trộn cả hai
/// trong CÙNG một Chương `done`: câu đã cắt bỏ phải vắng mặt HOÀN TOÀN (đúng AC5/AC6 của
/// Story 5.11), và trong số câu CÒN SỐNG, câu chưa ký phải mang `is_confirmed = false` ĐÚNG
/// CHỖ của nó — không bị lệch vị trí bởi việc mấy câu kia đã biến mất.
#[test]
fn omitted_and_unconfirmed_sentences_coexist_correctly_in_the_same_done_chapter() {
    let root = temp_dir("5-12-omitted-and-unconfirmed-mixed");
    let mut opened = create_work_from_text(&root, "5.12 Tron", "zh", "", "一。二。三。四。五。".to_owned())
        .expect("tao tac pham that bai");
    let chapter_id_to_mark_done = opened.chapter_id;
    make_done(&mut opened, chapter_id_to_mark_done);

    let ids = ordered_ids(&opened);
    assert_eq!(ids.len(), 5, "fixture phai co dung nam cau");
    // Cau 2 va cau 4: cat bo. Cau 1: xac nhan. Cau 3 va cau 5: giu nguyen 'draft' mac dinh.
    set_omitted(&opened, &[ids[1], ids[3]]);
    set_segment_status_directly(&opened, ids[0], SEGMENT_STATUS_CONFIRMED);

    let run = read_reading_run(Some(&opened)).expect("doc luot doc that bai");
    let chapter = &run.chapters[0];
    assert_eq!(chapter.segment_count, 5, "segment_count dem CA cau da cat bo");

    let all_segments: Vec<_> = chapter.paragraphs.iter().flat_map(|p| p.segments.iter()).collect();
    let surviving_ids: Vec<i64> = all_segments.iter().map(|s| s.id).collect();
    assert_eq!(
        surviving_ids,
        vec![ids[0], ids[2], ids[4]],
        "cau da cat bo (2 va 4) phai VANG MAT HOAN TOAN, con lai dung ba cau theo dung thu tu"
    );

    let by_id = |id: i64| all_segments.iter().find(|s| s.id == id).expect("segment con song phai co mat");
    assert!(by_id(ids[0]).is_confirmed, "cau 1 da ky ⇒ is_confirmed = true");
    assert!(!by_id(ids[2]).is_confirmed, "cau 3 van 'draft' ⇒ is_confirmed = false, DUNG CHO cua no");
    assert!(!by_id(ids[4]).is_confirmed, "cau 5 van 'draft' ⇒ is_confirmed = false, DUNG CHO cua no");

    let dir = opened.dir.clone();
    drop(opened);
    cleanup(&dir);
}

/// §I/O Matrix "Xong bằng tay" — Chương đặt `done` THỦ CÔNG trong khi còn ba câu `draft` ⇒
/// hành vi Y HỆT ca trên: không đường mã nào phân biệt *"đã xong bằng tay"* với *"đã xong"*
/// (không một cờ `manually_set` nào được đọc ở `read_reading_run`).
#[test]
fn manually_marking_a_chapter_done_behaves_identically_to_any_other_done_chapter() {
    let root = temp_dir("5-12-manual-done");
    let mut opened = create_work_from_text(&root, "5.12 Xong bang tay", "zh", "", "一。二。三。".to_owned())
        .expect("tao tac pham that bai");

    let ids = ordered_ids(&opened);
    assert_eq!(ids.len(), 3, "fixture phai co dung ba cau, ca ba deu 'draft' mac dinh");

    // Dat done THU CONG trong khi ca ba cau van 'draft' -- khong xac nhan cau nao truoc.
    let chapter_id_to_mark_done = opened.chapter_id;
    make_done(&mut opened, chapter_id_to_mark_done);

    let run = read_reading_run(Some(&opened)).expect("doc luot doc that bai");
    let all_segments: Vec<_> = run.chapters[0].paragraphs.iter().flat_map(|p| p.segments.iter()).collect();
    assert_eq!(all_segments.len(), 3, "ca ba cau phai ra day -- 'done' khong xoa cau nao");
    assert!(
        all_segments.iter().all(|s| !s.is_confirmed),
        "ca ba cau van 'draft' ⇒ ca ba mang is_confirmed = false, dung khuon ca tren"
    );

    let dir = opened.dir.clone();
    drop(opened);
    cleanup(&dir);
}

/// §I/O Matrix "Chương `done` rỗng" — Chương `done`, 0 segment còn sống ⇒ `paragraphs = []`
/// VÀ `segment_count = 0`.
#[test]
fn a_done_chapter_with_no_segments_reads_as_zero_paragraphs_and_zero_segment_count() {
    let root = temp_dir("5-12-empty-chapter");
    let mut opened = create_work_from_text(&root, "5.12 Rong", "zh", "", "   \n  ".to_owned())
        .expect("tao tac pham that bai");
    let chapter_id_to_mark_done = opened.chapter_id;
    make_done(&mut opened, chapter_id_to_mark_done);

    let ids = ordered_ids(&opened);
    assert!(ids.is_empty(), "fixture phai khong co segment nao");

    let run = read_reading_run(Some(&opened)).expect("Chuong rong khong duoc la mot loi");
    let chapter = &run.chapters[0];
    assert_eq!(chapter.chapter_id, opened.chapter_id);
    assert!(chapter.paragraphs.is_empty(), "Chuong rong ⇒ paragraphs rong");
    assert_eq!(chapter.segment_count, 0, "Chuong rong ⇒ segment_count = 0");

    let dir = opened.dir.clone();
    drop(opened);
    cleanup(&dir);
}

/// §I/O Matrix "Mọi câu cắt bỏ" — Chương `done`, mọi segment `is_omitted = 1` ⇒
/// `paragraphs = []` NHƯNG `segment_count > 0` — đây là dữ kiện phân biệt hai ca "rỗng" khác
/// nhau mà `readingState.ts` dùng, không một lệnh IPC phụ.
#[test]
fn a_chapter_where_every_sentence_is_omitted_reads_as_zero_paragraphs_but_positive_segment_count() {
    let root = temp_dir("5-12-all-omitted");
    let mut opened = create_work_from_text(&root, "5.12 Het cat bo", "zh", "", "一。二。\n三。四。".to_owned())
        .expect("tao tac pham that bai");
    let chapter_id_to_mark_done = opened.chapter_id;
    make_done(&mut opened, chapter_id_to_mark_done);

    let ids = ordered_ids(&opened);
    assert_eq!(ids.len(), 4, "fixture phai co dung bon cau");
    set_omitted(&opened, &ids);

    let run = read_reading_run(Some(&opened)).expect("doc luot doc that bai");
    let chapter = &run.chapters[0];
    assert!(
        chapter.paragraphs.is_empty(),
        "moi cau da cat bo ⇒ paragraphs rong, dung khuon Chuong rong nhung tu MOT du kien khac"
    );
    assert_eq!(chapter.segment_count, 4, "segment_count dem CA cau da cat bo -- day la dau hieu phan biet voi Chuong that su rong");

    let dir = opened.dir.clone();
    drop(opened);
    cleanup(&dir);
}

/// §I/O Matrix "Hàng Chương biến mất" — `OpenWork.chapter_id` không còn trong bảng `chapter`
/// ⇒ `segment.chapter_not_found`, không một `store.read_failed` chung chung.
#[test]
fn a_vanished_open_chapter_row_fails_with_chapter_not_found() {
    let root = temp_dir("5-12-vanished-chapter");
    let opened = create_work_from_text(&root, "5.12 Bien mat", "zh", "", "一。".to_owned())
        .expect("tao tac pham that bai");
    let chapter_id = opened.chapter_id;
    delete_chapter_row_directly(&opened, chapter_id);

    let err = read_reading_run(Some(&opened)).expect_err("hang Chuong da bien mat phai bi tu choi");
    assert_eq!(err.code(), "segment.chapter_not_found");

    let dir = opened.dir.clone();
    drop(opened);
    cleanup(&dir);
}
