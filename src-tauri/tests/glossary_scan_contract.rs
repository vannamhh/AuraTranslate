//! I/O & Edge-Case Matrix của Story 3.5 (quét ứng viên khi nhập tài liệu) — TẦNG THUẦN,
//! không `Store` nào dựng ở tệp này. `core::glossary::scan::scan_candidates` là module LÁ
//! (doc-comment của chính nó) — điều kiện để mọi ca dưới đây chạy TẤT ĐỊNH, không cần một
//! `.atproj` nào.
//!
//! ⚠️ Ca *"Đã có trong Glossary"*/*"Đã từng bị bỏ"* của I/O Matrix chạm SQL
//! (`WHERE NOT EXISTS`/`ON CONFLICT DO NOTHING`) — chúng nằm ở `glossary_contract.rs`
//! (`insert_import_scan_candidates`), không ở đây.
//!
//! ⚠️ Mỗi hàng của I/O Matrix là ĐÚNG MỘT ca, tên hàm là một CÂU khẳng định.

use auratranslate_lib::core::glossary::scan::{CONTEXT_EXAMPLE_CHAR_LIMIT, scan_candidates};
use auratranslate_lib::core::glossary::surnames::COMMON_SURNAMES;
use auratranslate_lib::core::matching::MatchLang;
use auratranslate_lib::core::scope::store::{
    DEFAULT_GLOSSARY_SCAN_THRESHOLD, parse_glossary_scan_threshold,
};

/// Vị từ `is_known` không bao giờ trả `true` — dùng cho mọi ca không cần một từ điển giả.
fn nothing_known(_term: &str) -> bool {
    false
}

// ═════════════════════════════════════════════════════════════════════════════════
// Hàng 1 — Zh, tên lặp
// ═════════════════════════════════════════════════════════════════════════════════

#[test]
fn a_chinese_name_repeated_forty_times_produces_one_row_with_the_right_count_and_context() {
    // 40 câu riêng biệt, mỗi câu chứa `萧炎` đúng một lần -- segment đã tách câu (Story 2.1),
    // không nối `\n`.
    let segments: Vec<String> = (0..40).map(|i| format!("萧炎在第{i}章出现。")).collect();
    let refs: Vec<&str> = segments.iter().map(String::as_str).collect();

    let mut is_known = nothing_known;
    let out = scan_candidates(&refs, MatchLang::Zh, 5, COMMON_SURNAMES, &mut is_known);

    let hit = out
        .iter()
        .find(|c| c.source_term == "萧炎")
        .unwrap_or_else(|| panic!("khong thay `萧炎` trong: {out:?}"));
    assert_eq!(hit.occurrence_count, 40);
    assert_eq!(hit.context_example, "萧炎在第0章出现。");
}

// ═════════════════════════════════════════════════════════════════════════════════
// Hàng 2 — Zh, n-gram lồng
// ═════════════════════════════════════════════════════════════════════════════════

/// `萧炎` 40 lần, MỖI lần đi kèm CÙNG một hậu tố cố định (`萧炎的`) -- tần suất hai chuỗi
/// BẰNG NHAU ⇒ chuỗi dài là rác đuôi, GIỮ chuỗi ngắn, chuỗi dài KHÔNG xuất hiện trong kết
/// quả.
#[test]
fn a_nested_ngram_with_equal_frequency_to_its_substring_is_dropped_as_padding() {
    let segments: Vec<String> = (0..40).map(|i| format!("萧炎的实力在第{i}章提升。")).collect();
    let refs: Vec<&str> = segments.iter().map(String::as_str).collect();

    let mut is_known = nothing_known;
    let out = scan_candidates(&refs, MatchLang::Zh, 5, COMMON_SURNAMES, &mut is_known);

    assert!(
        out.iter().any(|c| c.source_term == "萧炎"),
        "chuoi ngan `萧炎` phai co mat: {out:?}"
    );
    assert!(
        !out.iter().any(|c| c.source_term == "萧炎的"),
        "chuoi dai `萧炎的` phai bi loai -- tan suat bang chuoi con, la rac duoi: {out:?}"
    );
}

/// Cùng chuỗi con, nhưng chuỗi dài xuất hiện Ở MỘT SỐ LƯỢT KHÁC — tần suất KHÁC nhau ⇒ CẢ
/// HAI là chuỗi thật, cả hai giữ lại.
///
/// ⚠️ **Cả HAI chuỗi con của `萧炎的` (`萧炎` VÀ `炎的`) phải LỆCH tần suất với nó**, không
/// chỉ một — `matches_child(drop_last) || matches_child(drop_first)` là một phép HOẶC:
/// khớp một bên là đủ để bị loại. Vì vậy phải thêm occurrence RIÊNG cho `炎的` (không đi
/// kèm `萧`) để nó không tình cờ bằng đúng tần suất của `萧炎的`.
#[test]
fn a_nested_ngram_with_a_different_frequency_from_its_substring_is_kept_alongside_it() {
    let mut segments: Vec<String> = (0..40).map(|i| format!("萧炎在第{i}章出现。")).collect();
    // Thêm MƯỜI câu mang `萧炎的` -- `萧炎` tổng cộng 50 lần (40 đứng một mình + 10 kèm hậu
    // tố `的`), `萧炎的` 10 lần, và `炎的` (drop_first của `萧炎的`) CŨNG 10 lần nếu dừng ở
    // đây -- đúng cái bẫy phải né, xử lý ở khối dưới.
    for i in 0..10 {
        segments.push(format!("萧炎的实力在第{i}章提升。"));
    }
    // NĂM câu nữa mang `炎的` KHÔNG đi kèm `萧` -- đẩy tần suất `炎的` lên 15, lệch khỏi 10
    // của `萧炎的`, để phép so KHÔNG tình cờ khớp qua nhánh `drop_first`.
    for i in 0..5 {
        segments.push(format!("此人脾气炎的很怪第{i}回。"));
    }
    let refs: Vec<&str> = segments.iter().map(String::as_str).collect();

    let mut is_known = nothing_known;
    let out = scan_candidates(&refs, MatchLang::Zh, 5, COMMON_SURNAMES, &mut is_known);

    let short = out
        .iter()
        .find(|c| c.source_term == "萧炎")
        .unwrap_or_else(|| panic!("khong thay `萧炎`: {out:?}"));
    assert_eq!(short.occurrence_count, 50);

    let long = out
        .iter()
        .find(|c| c.source_term == "萧炎的")
        .unwrap_or_else(|| panic!("khong thay `萧炎的` -- tan suat khac ca hai chuoi con, phai giu ca hai: {out:?}"));
    assert_eq!(long.occurrence_count, 10);
}

// ═════════════════════════════════════════════════════════════════════════════════
// Hàng 3 — Zh, có trong từ điển
// ═════════════════════════════════════════════════════════════════════════════════

#[test]
fn a_chinese_term_already_in_the_dictionary_produces_zero_rows() {
    let segments: Vec<String> = (0..80).map(|i| format!("他在{i}天里修炼。")).collect();
    let refs: Vec<&str> = segments.iter().map(String::as_str).collect();

    let mut is_known = |term: &str| term == "修炼";
    let out = scan_candidates(&refs, MatchLang::Zh, 5, COMMON_SURNAMES, &mut is_known);

    assert!(
        !out.iter().any(|c| c.source_term == "修炼"),
        "`修炼` da co trong tu dien -- phai bi loai o buoc tra, khong duoc ra hang: {out:?}"
    );
}

// ═════════════════════════════════════════════════════════════════════════════════
// Hàng 4/5 — Zh, đoán tên người (bảng họ)
// ═════════════════════════════════════════════════════════════════════════════════

/// Chuỗi 2 ký tự, ký tự đầu (`萧`) nằm trong bảng họ, tần suất = ngưỡng − 1 (4 lần, ngưỡng
/// 5) ⇒ VẪN ra một hàng nhờ bảng họ hạ ngưỡng.
#[test]
fn a_two_char_surname_shaped_string_below_threshold_by_one_is_kept_via_the_surname_table() {
    let segments: Vec<String> = (0..4).map(|i| format!("萧风在第{i}章登场。")).collect();
    let refs: Vec<&str> = segments.iter().map(String::as_str).collect();

    let mut is_known = nothing_known;
    let out = scan_candidates(&refs, MatchLang::Zh, 5, COMMON_SURNAMES, &mut is_known);

    let hit = out
        .iter()
        .find(|c| c.source_term == "萧风")
        .unwrap_or_else(|| panic!("`萧风` (ho + 1 ky tu, 4 lan, nguong 5-1=4) phai co mat: {out:?}"));
    assert_eq!(hit.occurrence_count, 4);
}

/// Cùng hình dạng nhưng CHỈ 3 lần (dưới cả ngưỡng đã hạ, `5 - 1 = 4`) ⇒ vẫn 0 hàng — bảng
/// họ NỚI đúng một bậc, không xoá hẳn ngưỡng.
#[test]
fn a_surname_shaped_string_still_below_the_lowered_threshold_produces_zero_rows() {
    let segments: Vec<String> = (0..3).map(|i| format!("萧风在第{i}章登场。")).collect();
    let refs: Vec<&str> = segments.iter().map(String::as_str).collect();

    let mut is_known = nothing_known;
    let out = scan_candidates(&refs, MatchLang::Zh, 5, COMMON_SURNAMES, &mut is_known);

    assert!(
        !out.iter().any(|c| c.source_term == "萧风"),
        "3 lan < nguong da ha (4) -- van phai bi loai: {out:?}"
    );
}

/// Ký tự đầu trong bảng họ, nhưng chuỗi 4 ký tự (quá dài cho luật nới) ⇒ đi đường ngưỡng
/// ĐẦY ĐỦ — 4 lần (== ngưỡng đã hạ nhưng < ngưỡng đầy đủ 5) phải bị loại.
#[test]
fn a_surname_shaped_prefix_on_a_four_char_string_does_not_get_the_lowered_threshold() {
    let segments: Vec<String> = (0..4).map(|i| format!("萧风雷动在第{i}章登场。")).collect();
    let refs: Vec<&str> = segments.iter().map(String::as_str).collect();

    let mut is_known = nothing_known;
    let out = scan_candidates(&refs, MatchLang::Zh, 5, COMMON_SURNAMES, &mut is_known);

    assert!(
        !out.iter().any(|c| c.source_term == "萧风雷动"),
        "4 ky tu -- bang ho KHONG noi (chi noi cho 2-3 ky tu); 4 lan < nguong day du 5: {out:?}"
    );
}

// ═════════════════════════════════════════════════════════════════════════════════
// Hàng 6/7 — En, cụm hoa
// ═════════════════════════════════════════════════════════════════════════════════

#[test]
fn a_capitalized_phrase_repeated_mid_sentence_produces_one_row() {
    let segments: Vec<String> = (0..12)
        .map(|i| format!("The beast known as Fire Dragon appeared again at hour {i}."))
        .collect();
    let refs: Vec<&str> = segments.iter().map(String::as_str).collect();

    let mut is_known = nothing_known;
    let out = scan_candidates(&refs, MatchLang::En, 5, COMMON_SURNAMES, &mut is_known);

    let hit = out
        .iter()
        .find(|c| c.source_term == "Fire Dragon")
        .unwrap_or_else(|| panic!("khong thay `Fire Dragon`: {out:?}"));
    assert_eq!(hit.occurrence_count, 12);
}

#[test]
fn a_capitalized_word_opening_three_hundred_segments_produces_zero_rows() {
    let segments: Vec<String> = (0..300).map(|i| format!("The hero walked away, tired, sentence {i}.")).collect();
    let refs: Vec<&str> = segments.iter().map(String::as_str).collect();

    let mut is_known = nothing_known;
    let out = scan_candidates(&refs, MatchLang::En, 5, COMMON_SURNAMES, &mut is_known);

    assert!(
        !out.iter().any(|c| c.source_term == "The"),
        "`The` dung dau moi segment -- vi tri dau segment bi loai: {out:?}"
    );
}

// ═════════════════════════════════════════════════════════════════════════════════
// Hàng 8 — Dưới ngưỡng
// ═════════════════════════════════════════════════════════════════════════════════

#[test]
fn a_string_repeated_four_times_at_threshold_five_produces_zero_rows() {
    // 🔴 KHÔNG đứng đầu segment — nếu không, ca này đo nhầm luật "hoa đầu câu" (Hàng 7) chứ
    // không đo luật "dưới ngưỡng" mà tên hàm khai.
    let segments: Vec<String> =
        (0..4).map(|i| format!("A beast called Fire Dragon roared at hour {i}.")).collect();
    let refs: Vec<&str> = segments.iter().map(String::as_str).collect();

    let mut is_known = nothing_known;
    let out = scan_candidates(&refs, MatchLang::En, 5, COMMON_SURNAMES, &mut is_known);

    assert!(
        !out.iter().any(|c| c.source_term == "Fire Dragon"),
        "4 lan < nguong 5: {out:?}"
    );
}

// ═════════════════════════════════════════════════════════════════════════════════
// Hàng 9 — Chương rỗng / toàn khoảng trắng
// ═════════════════════════════════════════════════════════════════════════════════

#[test]
fn an_empty_chapter_produces_zero_candidates() {
    let refs: Vec<&str> = Vec::new();
    let mut is_known = nothing_known;

    let out_zh = scan_candidates(&refs, MatchLang::Zh, 5, COMMON_SURNAMES, &mut is_known);
    assert!(out_zh.is_empty());

    let out_en = scan_candidates(&refs, MatchLang::En, 5, COMMON_SURNAMES, &mut is_known);
    assert!(out_en.is_empty());
}

// ═════════════════════════════════════════════════════════════════════════════════
// Rà ba lớp 2026-08-22 — `context_example` phải mang một TRẦN độ dài, cắt ở biên KÝ TỰ
// ═════════════════════════════════════════════════════════════════════════════════

#[test]
fn a_context_example_longer_than_the_limit_is_truncated_at_a_character_boundary_not_a_byte_one() {
    // Một segment DÀI, KHÔNG dấu kết câu ở giữa (mô phỏng đúng ca sinh ra khoảng hở --
    // `split_source_text` vẫn phát một segment cho một đoạn thiếu dấu kết câu) -- toàn chữ
    // Hán NHIỀU BYTE, để một lượt cắt sai biên byte sẽ panic ngay tại test này.
    let mut long_segment = String::new();
    for i in 0..80 {
        // "萧炎" lặp lại nhiều lần TRONG CHÍNH đoạn dài này -- đủ tần suất qua ngưỡng 5, và
        // đủ dài (mỗi vòng lặp thêm ký tự) để tổng độ dài vượt xa CONTEXT_EXAMPLE_CHAR_LIMIT.
        long_segment.push_str(&format!("萧炎在遥远的第{i}处大陆之上继续修炼着"));
    }
    assert!(
        long_segment.chars().count() > CONTEXT_EXAMPLE_CHAR_LIMIT,
        "fixture phai THAT SU dai hon tran de ca nay co y nghia"
    );

    let refs: Vec<&str> = vec![long_segment.as_str()];
    let mut is_known = nothing_known;
    let out = scan_candidates(&refs, MatchLang::Zh, 5, COMMON_SURNAMES, &mut is_known);

    let hit = out
        .iter()
        .find(|c| c.source_term == "萧炎")
        .unwrap_or_else(|| panic!("khong thay `萧炎` trong: {out:?}"));

    let char_count = hit.context_example.chars().count();
    assert!(
        char_count <= CONTEXT_EXAMPLE_CHAR_LIMIT,
        "context_example phai bi cat ve toi da {CONTEXT_EXAMPLE_CHAR_LIMIT} ky tu, nhan {char_count}"
    );
    assert_eq!(
        char_count, CONTEXT_EXAMPLE_CHAR_LIMIT,
        "segment goc dai hon tran nen ban cat phai dung KHOP tran, khong ngan hon"
    );

    // Đối chứng biên: bản cắt phải là ĐÚNG {CONTEXT_EXAMPLE_CHAR_LIMIT} ký tự ĐẦU của segment
    // gốc -- không phải một chuỗi con nào khác, không mất/thêm ký tự nào ở đầu.
    let expected_prefix: String = long_segment.chars().take(CONTEXT_EXAMPLE_CHAR_LIMIT).collect();
    assert_eq!(hit.context_example, expected_prefix);

    // `String` hợp lệ tự nó đã chứng minh không cắt giữa một ký tự nhiều byte -- Rust không
    // cho tồn tại một `String` không hợp lệ UTF-8. Khẳng định thêm cho rõ ý test.
    assert!(hit.context_example.is_char_boundary(hit.context_example.len()));
}

#[test]
fn a_context_example_shorter_than_the_limit_is_kept_whole() {
    let segments: Vec<String> = (0..6).map(|i| format!("萧炎在第{i}章出现")).collect();
    let refs: Vec<&str> = segments.iter().map(String::as_str).collect();

    let mut is_known = nothing_known;
    let out = scan_candidates(&refs, MatchLang::Zh, 5, COMMON_SURNAMES, &mut is_known);

    let hit = out
        .iter()
        .find(|c| c.source_term == "萧炎")
        .unwrap_or_else(|| panic!("khong thay `萧炎` trong: {out:?}"));
    assert_eq!(
        hit.context_example, "萧炎在第0章出现",
        "segment ngan hon tran phai giu NGUYEN VEN, khong bi cat mot ky tu nao"
    );
}

// ═════════════════════════════════════════════════════════════════════════════════
// Hàng 10 — Ngưỡng cấu hình sai (`core::scope::store::parse_glossary_scan_threshold`)
// ═════════════════════════════════════════════════════════════════════════════════

#[test]
fn a_missing_threshold_key_falls_back_to_the_default() {
    assert_eq!(parse_glossary_scan_threshold(None), DEFAULT_GLOSSARY_SCAN_THRESHOLD);
}

#[test]
fn a_non_numeric_threshold_value_falls_back_to_the_default() {
    assert_eq!(parse_glossary_scan_threshold(Some("abc")), DEFAULT_GLOSSARY_SCAN_THRESHOLD);
}

#[test]
fn a_zero_threshold_value_falls_back_to_the_default() {
    // "0" phan tich DUOC thanh 0u32 nhung bi chan tuong minh -- mot nguong 0 tat het bo loc.
    assert_eq!(parse_glossary_scan_threshold(Some("0")), DEFAULT_GLOSSARY_SCAN_THRESHOLD);
}

#[test]
fn a_negative_threshold_value_falls_back_to_the_default() {
    assert_eq!(parse_glossary_scan_threshold(Some("-3")), DEFAULT_GLOSSARY_SCAN_THRESHOLD);
}

#[test]
fn a_valid_threshold_value_parses_through_unchanged() {
    assert_eq!(parse_glossary_scan_threshold(Some("12")), 12);
}
