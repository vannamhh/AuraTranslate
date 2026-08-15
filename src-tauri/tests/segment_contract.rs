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
    confirm_segment, flush_segment_targets, read_open_chapter_segments, save_segment_targets,
    set_segment_omitted, split_chapter_into_segments, unconfirm_edited_segments, SegmentTargetEdit,
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

/// 🔵 **CẬP NHẬT 2026-08-14 (Story 2.5).** Ca này trước đây tên
/// `..._reaches_six_through_five_steps` và khẳng định `[1, 2, 3, 5, 6]`. Bước **7** ra đời
/// cùng máy trạng thái AD-31, nên phép kiểm được **nâng cho nó nói thật về lược đồ mới** —
/// không phải nới cho hết đỏ: nó vẫn khẳng định danh sách **nguyên văn**, kể cả lỗ hổng ở 4.
///
/// 🔵 **CẬP NHẬT 2026-08-15 (Story 2.5c, AC7).** Bước **8** ra đời cùng cột `is_omitted`
/// (FR133). Tên hàm đổi theo — nó là một **câu khẳng định**, nên một cái tên nói "bảy qua
/// sáu bước" trên một bộ tám bước là một câu **sai** mà trình biên dịch không bao giờ báo.
#[test]
fn the_project_migration_set_reaches_eight_through_seven_steps() {
    let versions: Vec<u32> = PROJECT_MIGRATIONS.iter().map(|m| m.to_version).collect();

    assert_eq!(
        versions,
        vec![1, 2, 3, 5, 6, 7, 8],
        "bo di tru cua `project.db` phai la 1 -> 2 -> 3 -> 5 -> 6 -> 7 -> 8 (4 la so da chay)"
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
    assert_eq!(
        migrated.schema_version(),
        8,
        "buoc 5, 6, 7 VA 8 phai da chay tren mot tep dung o phien ban 4"
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
    assert_eq!(
        migrated.schema_version(),
        8,
        "buoc 6, 7 VA 8 phai chay tren mot tep dung o phien ban 5"
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

    assert_eq!(
        opened.store.schema_version(),
        8,
        "mot `project.db` moi phai dung o phien ban 8 (Story 2.5c them cot `is_omitted`)"
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
    assert_eq!(
        migrated.schema_version(),
        8,
        "buoc 7 VA 8 phai chay tren mot tep dung o phien ban 6"
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
        .expect("mot `project.db` o phien ban 7 phai mo duoc va di tru len 8");
    assert_eq!(
        migrated.schema_version(),
        8,
        "buoc 8 phai chay tren mot tep dung o phien ban 7"
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
#[test]
fn a_project_database_newer_than_the_app_is_refused_and_never_written_to() {
    static STEP_NINE: [Migration; 8] = [
        PROJECT_MIGRATIONS[0],
        PROJECT_MIGRATIONS[1],
        PROJECT_MIGRATIONS[2],
        PROJECT_MIGRATIONS[3],
        PROJECT_MIGRATIONS[4],
        PROJECT_MIGRATIONS[5],
        PROJECT_MIGRATIONS[6],
        // Mot buoc 9 GIA — day la "mot ban ung dung tuong lai" nhin tu hom nay.
        Migration {
            to_version: 9,
            sql: "CREATE TABLE tu_tuong_lai (id INTEGER PRIMARY KEY);",
        },
    ];

    let dir = temp_dir("newer-refused");
    let db = dir.join("project.db");

    let future = Store::open(StoreSpec {
        migrations: &STEP_NINE,
        ..StoreSpec::project(db.clone())
    })
    .expect("dung fixture o phien ban 9");
    assert_eq!(future.schema_version(), 9);
    drop(future);

    let before = fs::metadata(&db).expect("doc metadata truoc").len();

    let refused = Store::open(StoreSpec::project(db.clone()));
    let err = refused.err().expect(
        "mot `project.db` o phien ban 9 PHAI bi tu choi mo -- AD-30 noi \"khong bao gio ghi vao\"",
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
type SegmentRow = (
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
);

fn read_all_segment_rows(open: &auratranslate_lib::commands::project::OpenWork) -> Vec<SegmentRow> {
    open.store
        .read(|conn| {
            let mut stmt = conn.prepare(
                "SELECT id, chapter_id, ord, source_text, is_paragraph_end, retired_at, \
                 created_at, updated_at, target_text, status, is_omitted \
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
                    r.get(9)?,
                    r.get(10)?,
                ))
            })?;
            rows.collect::<Result<Vec<_>, _>>()
        })
        .expect("doc lai muoi mot cot that bai")
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
        real, 11,
        "bang `segment` co {real} cot, ma `read_all_segment_rows` doc 11. Mot cot moi PHAI \
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

    let outcome = confirm_segment(Some(&opened), id).expect("xac nhan that bai");

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
    confirm_segment(Some(&opened), id).expect("xac nhan that bai");
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
    confirm_segment(Some(&opened), id).expect("xac nhan that bai");
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
    confirm_segment(Some(&opened), id).expect("xac nhan that bai");

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
    confirm_segment(Some(&opened), id).expect("xac nhan that bai");

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
    confirm_segment(Some(&opened), id).expect("xac nhan lan dau that bai");

    let before = read_all_segment_rows(&opened);

    // Giu phim: nam luot xac nhan lien tiep tren cung mot cau.
    for _ in 0..5 {
        let again = confirm_segment(Some(&opened), id).expect("xac nhan lai PHAI vo hai");
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
        confirm_segment(None, id).expect_err("phai tu choi").message_key(),
        MessageKey::ProjectNoWorkOpen
    );

    // ② `segment.id` khong ton tai.
    assert_eq!(
        confirm_segment(Some(&opened), 9_999_999)
            .expect_err("phai tu choi")
            .message_key(),
        MessageKey::SegmentNotFound
    );

    // ③ Cau CHUA DICH (`target_text` rong) -- Quyet dinh #7, Ice ky 2026-08-14.
    assert_eq!(
        confirm_segment(Some(&opened), id)
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
        confirm_segment(Some(&opened), retired_id)
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
    confirm_segment(Some(&opened), first).expect("xac nhan that bai");

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
    confirm_segment(Some(&opened), id).expect("xac nhan that bai");

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
    confirm_segment(Some(&opened), id).expect("xac nhan that bai");

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
/// đây đã có khoá riêng từ Story 2.5 *(`ProjectNoWorkOpen` · `SegmentNotFound` ·
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
        MessageKey::ProjectNoWorkOpen
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
    confirm_segment(Some(&opened), first).expect("xac nhan that bai");

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
    confirm_segment(Some(&opened), signed).expect("xac nhan that bai");

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

        let err = confirm_segment(Some(&opened), target)
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
