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

use auratranslate_lib::commands::project::create_work_from_text;
use auratranslate_lib::commands::segment::{
    read_open_chapter_segments, save_segment_targets, split_chapter_into_segments, SegmentTargetEdit,
    SplitOutcome,
};
use auratranslate_lib::core::i18n::MessageKey;
use auratranslate_lib::core::segment::split::{
    split_source_text, EN_ABBREVIATIONS, LANG_CHINESE, SplitSegment,
};
use auratranslate_lib::core::store::{
    Migration, PINNED_ENTRY_DDL, PROJECT_MIGRATIONS, Store, StoreSpec, Transaction,
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

#[test]
fn the_project_migration_set_reaches_six_through_five_steps() {
    let versions: Vec<u32> = PROJECT_MIGRATIONS.iter().map(|m| m.to_version).collect();

    assert_eq!(
        versions,
        vec![1, 2, 3, 5, 6],
        "bo di tru cua `project.db` phai la 1 -> 2 -> 3 -> 5 -> 6 (4 la so da chay)"
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
    assert_eq!(
        migrated.schema_version(),
        6,
        "buoc 5 VA buoc 6 phai da chay tren mot tep dung o phien ban 4"
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
#[test]
fn a_fresh_project_database_lands_at_version_six_with_a_target_text_column() {
    let root = temp_dir("fresh-at-six");
    let opened = create_work_from_text(&root, "Sau", "zh", "", "一。二。".to_owned())
        .expect("tao tac pham that bai");

    assert_eq!(
        opened.store.schema_version(),
        6,
        "mot `project.db` moi phai dung o phien ban 6 (Story 2.2 them buoc `target_text`)"
    );

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
fn a_project_database_at_version_five_migrates_to_six_and_keeps_every_segment_row() {
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
        .expect("mot `project.db` o phien ban 5 phai mo duoc va di tru len 6");
    assert_eq!(
        migrated.schema_version(),
        6,
        "buoc 6 phai chay tren mot tep dung o phien ban 5"
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
    assert_eq!(err.code(), "project.no_work_open");
    assert_eq!(err.message_key(), MessageKey::ProjectNoWorkOpen);
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
    assert_eq!(err.code(), "project.no_work_open");
    assert_eq!(err.message_key(), MessageKey::ProjectNoWorkOpen);
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
            // Cau 5 ve huu -- nguon du lieu DUY NHAT cua gia tri vach `ornament`, va hom nay
            // khong duong SAN PHAM nao dat duoc no (Story 2.8). Bom bang SQL la cach duy nhat
            // nhin thay nhanh do chay.
            tx.execute(
                "UPDATE segment SET retired_at = '2026-08-12T00:00:00.000Z' WHERE ord = 5",
                [],
            )?;
            Ok(())
        })
        .expect("bom ban dich bang SQL that bai");

    let loaded = read_open_chapter_segments(Some(&opened)).expect("nap segment that bai");
    assert_eq!(loaded.segments.len(), 5, "fixture phai co dung nam cau");

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
            (translations[3].1, true),
        ],
        "ban dich phai di qua TRON duong cua san pham -- ke ca cau RONG (nhanh *khong vach*) \
         va cau da ve huu (nhanh `ornament`)"
    );

    // Co ket doan van la thu DA LUU, khong bi luot `UPDATE` dung toi.
    assert_eq!(
        loaded
            .segments
            .iter()
            .map(|s| s.is_paragraph_end)
            .collect::<Vec<_>>(),
        vec![false, true, false, false, false],
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

/// Chín cột thật của `segment` hôm nay, đọc lại bằng SQL để khẳng định AC14.
///
/// ⚠️ Đọc bằng SQL **thô** chứ không qua `read_open_chapter_segments`, và đó là điều kiện
/// để phép kiểm có nghĩa: lệnh đọc của sản phẩm chỉ trả **sáu** trường, nên nó **không thấy**
/// `created_at`/`updated_at`/`chapter_id` — đúng ba cột mà AC14 nói phải y nguyên hoặc phải
/// đổi. Một phép kiểm đi qua lệnh đọc là một phép kiểm mù với ba cột nó phải canh.
type SegmentRow = (i64, i64, i64, String, i64, Option<String>, String, String, String);

fn read_all_segment_rows(open: &auratranslate_lib::commands::project::OpenWork) -> Vec<SegmentRow> {
    open.store
        .read(|conn| {
            let mut stmt = conn.prepare(
                "SELECT id, chapter_id, ord, source_text, is_paragraph_end, retired_at, \
                 created_at, updated_at, target_text \
                 FROM segment ORDER BY ord",
            )?;
            let rows = stmt.query_map([], |r| {
                Ok((
                    r.get(0)?,
                    r.get(1)?,
                    r.get(2)?,
                    r.get(3)?,
                    r.get(4)?,
                    r.get(5)?,
                    r.get(6)?,
                    r.get(7)?,
                    r.get(8)?,
                ))
            })?;
            rows.collect::<Result<Vec<_>, _>>()
        })
        .expect("doc lai chin cot that bai")
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

    // BAY cot con lai phai y nguyen TUNG BYTE — day la thu cuong che AD-31 hang 1 that su.
    assert_eq!(a.0, b.0, "`id` doi");
    assert_eq!(a.1, b.1, "`chapter_id` doi");
    assert_eq!(a.2, b.2, "`ord` doi");
    assert_eq!(a.3, b.3, "`source_text` doi — AD-4 dong bang ranh gioi");
    assert_eq!(a.4, b.4, "`is_paragraph_end` doi — AD-37 noi do la du lieu DA LUU");
    assert_eq!(a.5, b.5, "`retired_at` doi");
    assert_eq!(a.6, b.6, "`created_at` doi — no la moc TAO, khong phai moc sua");

    // Hai cau KHONG nam trong lo phai y nguyen tron ven, ke ca `updated_at`.
    assert_eq!(&after[1..], &before[1..], "cau ngoai lo bi dung toi");

    drop(opened);
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
    assert_eq!(err.code(), "project.no_work_open");
    assert_eq!(err.message_key(), MessageKey::ProjectNoWorkOpen);
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
