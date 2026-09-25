//! Contract for `support/boundary_scan.rs`. Every `*_boundary.rs` file relies on these
//! functions; a red case here means every file that switched to this module is blind to
//! whatever that case guards.
#[path = "support/boundary_scan.rs"]
#[allow(dead_code)] // shared module: not every helper is used in this file
mod boundary_scan;

use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};

// ═════════════════════════════════════════════════════════════════════════════════
// I/O matrix row: sibling directory
// ═════════════════════════════════════════════════════════════════════════════════

#[test]
fn a_sibling_directory_sharing_a_name_prefix_is_not_inside() {
    assert!(
        !boundary_scan::is_inside("core/scope_legacy/x.rs", "core/scope"),
        "`core/scope_legacy/**` không phải `core/scope/**` — một tiền tố chuỗi khớp không \
         phải một biên thư mục khớp"
    );
}

#[test]
fn the_directory_itself_and_a_real_child_are_inside() {
    assert!(boundary_scan::is_inside("core/scope", "core/scope"), "chính thư mục phải khớp");
    assert!(
        boundary_scan::is_inside("core/scope/mod.rs", "core/scope"),
        "một tệp thật trong thư mục phải khớp"
    );
    assert!(
        !boundary_scan::is_inside("core/other/mod.rs", "core/scope"),
        "một thư mục không liên quan không được khớp"
    );
}

// ═════════════════════════════════════════════════════════════════════════════════
// I/O matrix row: block comment
// ═════════════════════════════════════════════════════════════════════════════════

#[test]
fn a_forbidden_token_hidden_inside_a_multiline_block_comment_is_not_seen() {
    let text = "fn a() {}\n/* mot chu thich\n   nhac lai forbidden_call() o day\n   qua nhieu dong */\nfn b() {}\n";
    let hits: Vec<String> = boundary_scan::code_lines(text)
        .filter(|(_, code)| code.contains("forbidden_call"))
        .map(|(_, code)| code)
        .collect();
    assert!(
        hits.is_empty(),
        "một token nằm TRONG khối `/* … */` (kể cả trải nhiều dòng) không được coi là mã: {hits:?}"
    );
}

#[test]
fn code_lines_reports_the_real_1_based_line_number_across_a_multiline_block_comment() {
    let text = "fn a() {}\n/* comment\n   spans\n   three lines */\nfn b() { real_call(); }\n";
    let got: Vec<(usize, String)> = boundary_scan::code_lines(text).collect();
    assert_eq!(
        got,
        vec![(1, "fn a() {}".to_string()), (5, "fn b() { real_call(); }".to_string())],
        "số dòng 1-based phải trỏ đúng vị trí THẬT trong `text`, kể cả dòng đứng SAU một khối \
         chú thích nhiều dòng: {got:?}"
    );
}

#[test]
fn nested_block_comments_are_stripped_as_one_unit() {
    let text = "fn a() {}\n/* ngoai /* trong forbidden_call() */ van ngoai */\nfn b() { real_call(); }\n";
    let hits: Vec<String> = boundary_scan::code_lines(text)
        .filter(|(_, code)| code.contains("forbidden_call"))
        .map(|(_, code)| code)
        .collect();
    assert!(hits.is_empty(), "chú thích khối LỒNG phải bị lột trọn, không dừng ở `*/` đầu tiên: {hits:?}");
    let real: Vec<String> = boundary_scan::code_lines(text)
        .filter(|(_, code)| code.contains("real_call"))
        .map(|(_, code)| code)
        .collect();
    assert_eq!(real.len(), 1, "mã THẬT sau khối chú thích lồng vẫn phải được thấy");
}

// ═════════════════════════════════════════════════════════════════════════════════
// I/O matrix row: `/*` inside a string
// ═════════════════════════════════════════════════════════════════════════════════

#[test]
fn a_slash_star_sequence_inside_a_normal_string_does_not_open_a_block_comment() {
    let text = "let pattern = \"src/**/*.rs\"; forbidden_call();\n";
    let hits: Vec<String> = boundary_scan::code_lines(text)
        .filter(|(_, code)| code.contains("forbidden_call"))
        .map(|(_, code)| code)
        .collect();
    assert_eq!(
        hits.len(),
        1,
        "`/*` bên trong một chuỗi thường không được mở một chú thích khối -- lời gọi SAU \
         chuỗi đó phải vẫn được thấy: {hits:?}"
    );
}

// ═════════════════════════════════════════════════════════════════════════════════
// I/O matrix row: raw string and char
// ═════════════════════════════════════════════════════════════════════════════════

#[test]
fn a_raw_string_containing_an_embedded_quote_does_not_desync_the_comment_scanner() {
    let text = "let a = r#\"one \"embedded\"#; /* forbidden_call() */ end();\n";
    let hits: Vec<String> = boundary_scan::code_lines(text)
        .filter(|(_, code)| code.contains("forbidden_call"))
        .map(|(_, code)| code)
        .collect();
    assert!(
        hits.is_empty(),
        "chuỗi thô mang dấu `\"` nhúng phải đóng ĐÚNG chỗ (`\"#`), để chú thích khối THẬT \
         đứng sau nó vẫn được lột: {hits:?}"
    );
}

#[test]
fn a_quote_char_literal_does_not_desync_the_comment_scanner() {
    let text = "let q = '\"'; /* forbidden_call() */ end();\n";
    let hits: Vec<String> = boundary_scan::code_lines(text)
        .filter(|(_, code)| code.contains("forbidden_call"))
        .map(|(_, code)| code)
        .collect();
    assert!(
        hits.is_empty(),
        "chữ ký tự `'\"'` không được hiểu nhầm thành mở một chuỗi -- chú thích khối THẬT \
         đứng sau nó vẫn phải bị lột: {hits:?}"
    );
}

#[test]
fn a_lifetime_marker_is_not_mistaken_for_an_unterminated_char_literal() {
    let text = "fn f<'a>(x: &'a str) -> &'a str { forbidden_call(); x }\n";
    let hits: Vec<String> = boundary_scan::code_lines(text)
        .filter(|(_, code)| code.contains("forbidden_call"))
        .map(|(_, code)| code)
        .collect();
    assert_eq!(
        hits.len(),
        1,
        "một chỉ định thời hạn (`'a`) không theo sau bởi một dấu `'` đóng thứ hai -- không \
         được nuốt phần thân hàm còn lại: {hits:?}"
    );
}

// ═════════════════════════════════════════════════════════════════════════════════
// I/O matrix row: dotfile / non-UTF-8
// ═════════════════════════════════════════════════════════════════════════════════

static NEXT_DIR: AtomicU64 = AtomicU64::new(0);

fn temp_dir(tag: &str) -> PathBuf {
    let n = NEXT_DIR.fetch_add(1, Ordering::Relaxed);
    let dir = std::env::temp_dir().join(format!(
        "auratranslate-boundary-scan-contract-{}-{}-{}",
        std::process::id(),
        tag,
        n
    ));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap_or_else(|e| panic!("tạo {}: {e}", dir.display()));
    dir
}

#[test]
fn a_dotfile_and_a_non_utf8_file_are_skipped_with_a_named_reason_not_a_panic() {
    let dir = temp_dir("dotfile-and-non-utf8");
    fs::write(dir.join(".DS_Store"), b"\x00\x01\x02\x03").expect("ghi .DS_Store giả");
    fs::write(dir.join("bad_utf8.rs"), [0xFF, 0xFE, 0x00, 0xFF]).expect("ghi tệp non-UTF-8");
    fs::write(dir.join("real.rs"), "fn a() {}\n").expect("ghi tệp thật");

    let (sources, skipped) = boundary_scan::any_sources(&dir);

    assert_eq!(sources.len(), 1, "chỉ đúng một tệp UTF-8 không-dotfile phải đọc được: {sources:?}");
    assert_eq!(sources[0].0, "real.rs");

    let dotfile_skip = skipped.iter().find(|(rel, _)| rel == ".DS_Store");
    assert!(
        matches!(dotfile_skip, Some((_, boundary_scan::Skip::Dotfile))),
        "`.DS_Store` phải bị bỏ qua với lý do Dotfile, không panic: {skipped:?}"
    );
    let non_utf8_skip = skipped.iter().find(|(rel, _)| rel == "bad_utf8.rs");
    assert!(
        matches!(non_utf8_skip, Some((_, boundary_scan::Skip::NonUtf8))),
        "một tệp không phải UTF-8 phải bị bỏ qua với lý do NonUtf8, không panic: {skipped:?}"
    );

    fs::remove_dir_all(&dir).ok();
}

// ═════════════════════════════════════════════════════════════════════════════════
// I/O matrix row: code after a test module
// ═════════════════════════════════════════════════════════════════════════════════

#[test]
fn code_after_the_closing_brace_of_a_cfg_test_module_is_still_scanned() {
    let text = "fn before() {}\n#[cfg(test)]\nmod tests {\n    fn t() { allowed_in_tests(); }\n}\nfn after() { forbidden_call(); }\n";
    let cleaned = boundary_scan::without_test_modules(text);

    let inside_test: Vec<String> = boundary_scan::code_lines(&cleaned)
        .filter(|(_, code)| code.contains("allowed_in_tests"))
        .map(|(_, code)| code)
        .collect();
    assert!(inside_test.is_empty(), "thân của khối test phải bị xoá: {inside_test:?}");

    let after_module: Vec<String> = boundary_scan::code_lines(&cleaned)
        .filter(|(_, code)| code.contains("forbidden_call"))
        .map(|(_, code)| code)
        .collect();
    assert_eq!(
        after_module.len(),
        1,
        "mã sản phẩm SAU khối `#[cfg(test)] mod … {{ … }}` phải vẫn được quét -- đây chính \
         là lỗ mà `text_before_first_cfg_test_line` (hai bản chép cũ) để hở: {after_module:?}"
    );
}

#[test]
fn text_with_no_cfg_test_line_is_returned_unchanged() {
    let text = "fn a() {}\nfn b() { forbidden_call(); }\n";
    let cleaned = boundary_scan::without_test_modules(text);
    assert_eq!(
        cleaned, text,
        "không có dòng `#[cfg(test)]` nào ⇒ phải trả về NGUYÊN VĂN input -- một phép cắt \
         nhầm ở đây làm một gate quét trúng chuỗi rỗng mà vẫn xanh"
    );
}

#[test]
fn a_comment_mentioning_the_attribute_does_not_fool_the_line_anchor() {
    let text = "fn a() {}\n// mot chu thich nhac lai chuoi \"#[cfg(test)]\" o day\nfn b() { forbidden_call(); }\n";
    let cleaned = boundary_scan::without_test_modules(text);
    let hits: Vec<String> = boundary_scan::code_lines(&cleaned)
        .filter(|(_, code)| code.contains("forbidden_call"))
        .map(|(_, code)| code)
        .collect();
    assert_eq!(
        hits.len(),
        1,
        "một chú thích chỉ NHẮC LẠI chuỗi `\"#[cfg(test)]\"` không được hiểu nhầm là bản khai \
         thật: {hits:?}"
    );
}

#[test]
fn a_cfg_test_module_declared_in_another_file_is_left_in_place() {
    let text = "fn before() {}\n#[cfg(test)]\nmod tests;\nfn after() { forbidden_call(); }\n";
    let cleaned = boundary_scan::without_test_modules(text);
    assert_eq!(
        cleaned, text,
        "`mod NAME;` (thân sống ở TỆP KHÁC) không có gì để xoá ở tệp này -- văn bản phải \
         giữ NGUYÊN VẸN"
    );
    let hits: Vec<String> = boundary_scan::code_lines(&cleaned)
        .filter(|(_, code)| code.contains("forbidden_call"))
        .map(|(_, code)| code)
        .collect();
    assert_eq!(hits.len(), 1, "mã sau `mod NAME;` phải vẫn được quét: {hits:?}");
}

#[test]
#[should_panic(expected = "không theo sau bởi một dòng `mod")]
fn a_cfg_test_not_followed_by_a_mod_line_fails_loudly() {
    let text = "#[cfg(test)]\nfn not_a_module() {}\n";
    boundary_scan::without_test_modules(text);
}

#[test]
#[should_panic(expected = "không cân dấu ngoặc")]
fn an_unbalanced_cfg_test_module_fails_loudly() {
    let text = "#[cfg(test)]\nmod tests {\n    fn t() { if true {\n}\n";
    boundary_scan::without_test_modules(text);
}

// ═════════════════════════════════════════════════════════════════════════════════
// I/O matrix row: floor drifted / tree truncated (population-floor helper)
// ═════════════════════════════════════════════════════════════════════════════════

#[test]
fn a_floor_at_exactly_85_percent_of_live_does_not_go_red() {
    boundary_scan::assert_population_floor(85, 100, "TEST_FLOOR", "tệp giả");
}

#[test]
fn a_floor_at_exactly_80_percent_of_live_does_not_go_red() {
    boundary_scan::assert_population_floor(80, 100, "TEST_FLOOR", "tệp giả");
}

#[test]
#[should_panic(expected = "Cây quá nhỏ để là thật")]
fn live_below_floor_is_the_truncated_tree_case() {
    boundary_scan::assert_population_floor(50, 49, "TEST_FLOOR", "tệp giả");
}

#[test]
#[should_panic(expected = "đã trôi dưới 80%")]
fn a_floor_below_80_percent_of_live_is_the_drifted_case() {
    boundary_scan::assert_population_floor(79, 100, "TEST_FLOOR", "tệp giả");
}

#[test]
fn a_drifted_floor_names_the_constant_and_ceil_0_85_times_live() {
    let result = std::panic::catch_unwind(|| {
        boundary_scan::assert_population_floor(21, 98, "SRC_RS_FLOOR", "tệp `.rs`");
    });
    let err = result.expect_err("sàn 21/98 = 21.4% phải panic");
    let message = err
        .downcast_ref::<String>()
        .cloned()
        .or_else(|| err.downcast_ref::<&str>().map(|s| s.to_string()))
        .unwrap_or_default();
    assert!(
        message.contains("SRC_RS_FLOOR") && message.contains("84"),
        "thông báo lỗi phải nêu tên hằng số VÀ ceil(0.85 × 98) = 84: {message:?}"
    );
}
