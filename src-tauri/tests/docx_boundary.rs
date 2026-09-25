//! Ranh giới cây nguồn của Story 6.12 (`core::docx`) — AD-39 (`.docx` không thừa kế quyền ra
//! mạng của một bộ đọc tệp cục bộ) và Quyết định 3 của spec 6.12 (0 điểm panic, đây là LÝ DO
//! cả story chọn tự đọc thay vì gọi `docx_rs::read_docx`).
//!
//! ─────────────────────────────────────────────────────────────────────────────
//! HAI MỆNH ĐỀ, đúng khuôn `webimport_boundary.rs`
//! ─────────────────────────────────────────────────────────────────────────────
//! 1. **`core/docx/mod.rs` mang 0 dòng gõ `reqwest`/`TcpStream`/`http://`/`https://`** — một
//!    bộ đọc tệp cục bộ không được thừa kế quyền ra mạng (§Always spec 6.12).
//! 2. **`core/docx/mod.rs` mang 0 dòng gõ `unwrap()`/`expect()`/`panic!`/`unreachable!`,
//!    hoặc chỉ số mảng TRẦN (`x[i]`, không qua `.get(i)`)** — đây là mệnh đề mà CẢ Quyết
//!    định 3 (tự đọc thay vì gọi `docx_rs::read_docx`) dựa lên, nên nó phải là một CỔNG CÓ
//!    TEST, không một lời hứa trong doc-comment (§Always spec 6.12).
//!
//! Sàn quần thể + kiểm chứng dương (ca dương + ca âm cho MỖI vị từ) là bắt buộc, khuôn
//! `webimport_boundary.rs`/`cleanup_boundary.rs`.

use std::fs;

#[path = "support/boundary_scan.rs"]
#[allow(dead_code)] // shared module: not every helper is used in this file
mod boundary_scan;
use boundary_scan::{code_lines, src_root};

/// Số tệp `.rs` tối thiểu dưới `src-tauri/src/**` — cùng lý lẽ mọi `*_boundary.rs` khác.
const SRC_RS_FLOOR: usize = 84;

fn all_rust_sources() -> Vec<(String, String)> {
    boundary_scan::rust_sources(&src_root())
}

/// `haystack` mang `needle` như một TỪ trọn vẹn — cùng khuôn `webimport_boundary.rs::contains_word`.
fn contains_word(haystack: &str, needle: &str) -> bool {
    let bytes = haystack.as_bytes();
    let is_ident = |b: u8| b.is_ascii_alphanumeric() || b == b'_';

    let mut search_from = 0usize;
    while let Some(pos) = haystack[search_from..].find(needle) {
        let start = search_from + pos;
        let end = start + needle.len();
        let before_is_boundary = start == 0 || !is_ident(bytes[start - 1]);
        let after_is_boundary = end >= bytes.len() || !is_ident(bytes[end]);
        if before_is_boundary && after_is_boundary {
            return true;
        }
        search_from = start + 1;
    }
    false
}

fn line_names_any_forbidden_token(code: &str, tokens: &[&str]) -> bool {
    tokens.iter().any(|needle| {
        if needle.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') {
            contains_word(code, needle)
        } else {
            code.contains(needle)
        }
    })
}

fn text_of(rel: &str) -> String {
    fs::read_to_string(src_root().join(rel)).unwrap_or_else(|e| panic!("đọc {rel}: {e}"))
}

const DOCX_MODULE: &str = "core/docx/mod.rs";

// ═════════════════════════════════════════════════════════════════════════════════
// Sàn quần thể
// ═════════════════════════════════════════════════════════════════════════════════

#[test]
fn the_scanned_tree_is_large_enough_to_be_real() {
    let files = all_rust_sources();
    boundary_scan::assert_population_floor(
        SRC_RS_FLOOR,
        files.len(),
        "SRC_RS_FLOOR",
        "tệp `.rs` dưới `src-tauri/src/**`",
    );
}

/// **THÊM (Story 6.12)** — `docx_boundary.rs::the_scanned_tree_is_large_enough_to_be_real`
/// quét TOÀN `src-tauri/src/**`, không riêng `core/docx/`. Đối chứng ⑥ của §Verification spec
/// 6.12 ("phép quét có thật không") đòi một phép kiểm trỏ THẲNG vào gốc quét thật của HAI mệnh
/// đề dưới đây — trỏ nó vào một thư mục RỖNG phải cho **0 tệp**, không phải xanh im lặng
/// (bài học `naming_boundary.rs:95-96`, nhắc lại ở doc-comment đầu tệp spec 6.12).
#[test]
fn pointing_the_scan_root_at_an_empty_directory_yields_zero_files_not_a_silent_green() {
    let empty = std::env::temp_dir().join(format!("docx_boundary_empty_probe_{}", std::process::id()));
    fs::create_dir_all(&empty).unwrap_or_else(|e| panic!("tạo {}: {e}", empty.display()));
    let files = boundary_scan::rust_sources(&empty);
    assert_eq!(files.len(), 0, "thư mục vừa tạo phải rỗng — nếu không, phép đo bên dưới vô nghĩa");
    let _ = fs::remove_dir_all(&empty);
}

// ═════════════════════════════════════════════════════════════════════════════════
// Mệnh đề 1 — core/docx/mod.rs mang 0 dòng gõ reqwest/TcpStream/http://https://
// ═════════════════════════════════════════════════════════════════════════════════

const NETWORK_TOKENS: [&str; 4] = ["reqwest", "TcpStream", "http://", "https://"];

#[test]
fn docx_module_carries_zero_lines_naming_a_network_client_or_a_literal_url_scheme() {
    let text = text_of(DOCX_MODULE);
    let offenders: Vec<String> = code_lines(&boundary_scan::without_test_modules(&text))
        .filter(|(_, code)| line_names_any_forbidden_token(code, &NETWORK_TOKENS))
        .map(|(line, code)| format!("{DOCX_MODULE}:{line}  {code}"))
        .collect();
    assert!(
        offenders.is_empty(),
        "một bộ đọc tệp .docx cục bộ không được thừa kế quyền ra mạng (§Always spec 6.12) — \
         tìm thấy dòng gõ mạng:\n{}",
        offenders.join("\n")
    );
}

#[test]
fn the_network_token_check_would_actually_flag_a_seeded_violation_and_ignore_clean_code() {
    assert!(
        line_names_any_forbidden_token("    let c = reqwest::blocking::Client::new();", &NETWORK_TOKENS),
        "ca DƯƠNG: `reqwest` phải bị vị từ bắt"
    );
    assert!(
        line_names_any_forbidden_token("    let u = \"https://example.com/x.docx\";", &NETWORK_TOKENS),
        "ca DƯƠNG: literal `https://` phải bị vị từ bắt"
    );
    assert!(
        !line_names_any_forbidden_token(
            "    let text = std::str::from_utf8(bytes).unwrap_or(\"\");",
            &NETWORK_TOKENS
        ),
        "ca ÂM: một dòng không gõ mạng không được bị bắt oan"
    );
}

// ═════════════════════════════════════════════════════════════════════════════════
// Mệnh đề 2 — core/docx/mod.rs mang 0 điểm panic (0 unwrap/expect/panic!/unreachable!/chỉ
// số mảng trần) — đây là điều kiện mà Quyết định 3 của spec 6.12 dựa lên.
// ═════════════════════════════════════════════════════════════════════════════════

/// Bốn token gọi hàm/macro panic trực tiếp, cộng một vị từ RIÊNG cho chỉ số mảng TRẦN (không
/// một token đơn — `x[i]` không phải một chuỗi con cố định, xem [`line_indexes_a_slice_raw`]).
const PANIC_CALL_TOKENS: [&str; 4] = ["unwrap()", "expect(", "panic!", "unreachable!"];

/// `code` chứa một chỉ số mảng/slice TRẦN — hình dạng `<định danh>[...]` KHÔNG đứng ngay sau
/// một dấu `.` (loại trừ `.get(i)`/phương thức, vốn không dùng cú pháp `[]`) và không phải một
/// KHAI BÁO KIỂU (`&[u8]`, `Vec<u8>`, `[&str; 4]`) — heuristic THUẦN VĂN BẢN, hẹp có chủ ý:
/// nó chỉ cần bắt được đúng hình dạng `blocks[i]`/`bytes[0]` mà `core::docx` THẬT SỰ có thể
/// viết ra, không cần là một trình phân tích Rust đầy đủ (cùng mức "hẹp đúng bằng hình dạng
/// thật" mà `naming_boundary.rs`/`webimport_boundary.rs` đã chấp nhận cho các vị từ của
/// chúng).
///
/// Vị từ: một `[` đứng ngay sau một ký tự ĐỊNH DANH (chữ/số/`_`), và KHÔNG đứng ngay sau `&`
/// hoặc ` ` do một khai báo kiểu mảng/slice tạo ra (`&[u8]`, `: [Foo; 4]`) — phân biệt bằng
/// cách đòi ký tự NGAY TRƯỚC `[` là một ký tự định danh (kiểu mảng luôn có `&`/khoảng trắng/
/// `<` đứng trước `[`, không bao giờ một định danh).
fn line_indexes_a_slice_raw(code: &str) -> bool {
    let bytes = code.as_bytes();
    let is_ident = |b: u8| b.is_ascii_alphanumeric() || b == b'_';
    for (i, &b) in bytes.iter().enumerate() {
        if b != b'[' {
            continue;
        }
        if i == 0 || !is_ident(bytes[i - 1]) {
            continue;
        }
        // Loại trừ `.len()`-style không áp dụng ở đây (không có `[` trong đó); loại trừ khai
        // báo kiểu như `Vec<[u8; 4]>` bằng cách đòi ký tự NGAY SAU `]` khớp không phải là một
        // phần của một khai báo kiểu — bỏ qua tinh chỉnh đó (không quan sát được trong
        // `core/docx/mod.rs`, xem đối chứng dương/âm ngay dưới cho đúng hai hình dạng THẬT sẽ
        // gặp ở tệp này).
        return true;
    }
    false
}

fn line_has_a_panic_point(code: &str) -> bool {
    PANIC_CALL_TOKENS.iter().any(|t| code.contains(t)) || line_indexes_a_slice_raw(code)
}

#[test]
fn docx_module_carries_zero_panic_points_in_product_code() {
    let text = text_of(DOCX_MODULE);
    let offenders: Vec<String> = code_lines(&boundary_scan::without_test_modules(&text))
        .filter(|(_, code)| line_has_a_panic_point(code))
        .map(|(line, code)| format!("{DOCX_MODULE}:{line}  {code}"))
        .collect();
    assert!(
        offenders.is_empty(),
        "core::docx phải mang 0 điểm panic (Quyết định 3 spec 6.12: đây LÀ lý do chọn tự đọc \
         thay vì gọi docx_rs::read_docx, panic = \"abort\" giết cả tiến trình trên một .docx \
         hỏng) — tìm thấy:\n{}",
        offenders.join("\n")
    );
}

#[test]
fn the_panic_point_check_would_actually_flag_seeded_violations_and_ignore_clean_code() {
    assert!(line_has_a_panic_point("    let v = data.unwrap();"), "ca DƯƠNG: `.unwrap()` phải bị bắt");
    assert!(
        line_has_a_panic_point("    let v = data.expect(\"khong the xay ra\");"),
        "ca DƯƠNG: `.expect(` phải bị bắt"
    );
    assert!(line_has_a_panic_point("    unreachable!()"), "ca DƯƠNG: `unreachable!` phải bị bắt");
    assert!(line_has_a_panic_point("    panic!(\"x\")"), "ca DƯƠNG: `panic!` phải bị bắt");
    assert!(line_has_a_panic_point("    let b = blocks[i];"), "ca DƯƠNG: chỉ số mảng trần `blocks[i]` phải bị bắt");
    assert!(
        !line_has_a_panic_point("    let b = blocks.get(i);"),
        "ca ÂM: `.get(i)` (không panic trên chỉ số ngoài phạm vi) không được bị bắt oan"
    );
    assert!(
        !line_has_a_panic_point("    fn f(bytes: &[u8]) -> Result<String, DocxError> { Ok(String::new()) }"),
        "ca ÂM: một khai báo KIỂU slice (`&[u8]`) không phải một phép chỉ số, không được bị bắt oan"
    );
    assert!(
        !line_has_a_panic_point("    match reader.read_event().map_err(xml_err)? {"),
        "ca ÂM: một dòng bình thường không mang token nào không được bị bắt oan"
    );
}

/// Đối chứng THẬT cho phép cắt `#[cfg(test)]` — một khối test GIEO các token cấm KHÔNG được
/// làm mệnh đề 2 đỏ oan (cùng khuôn `webimport_boundary.rs`, mục cuối).
#[test]
fn a_forbidden_token_seeded_only_inside_a_cfg_test_block_is_not_counted_against_the_product_code() {
    let seeded = "fn read() {}\n#[cfg(test)]\nmod tests {\n    fn x() { let v = data[0]; let _ = v.unwrap(); }\n}\n";
    let offenders: Vec<String> = code_lines(&boundary_scan::without_test_modules(seeded))
        .filter(|(_, code)| line_has_a_panic_point(code))
        .map(|(_, code)| code)
        .collect();
    assert!(
        offenders.is_empty(),
        "một khối `#[cfg(test)]` gieo `.unwrap()`/chỉ số mảng KHÔNG được làm cổng đỏ oan — mã \
         SẢN PHẨM là thứ Quyết định 3 ràng buộc, không phải mã test cùng tệp. Tìm thấy: {offenders:?}"
    );
    let product = "fn read() { let v = data[0]; }\n";
    assert_eq!(
        code_lines(&boundary_scan::without_test_modules(product)).filter(|(_, code)| line_has_a_panic_point(code)).count(),
        1,
        "cùng token đó ở mã SẢN PHẨM phải bị bắt — phép cắt không được nuốt luôn thứ cần canh"
    );
}
