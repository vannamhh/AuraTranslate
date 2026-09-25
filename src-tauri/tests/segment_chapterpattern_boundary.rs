//! Ranh giới cây nguồn của Story 6.6 — FR14, mẫu phân tách Chương thân THẬT của bước 5
//! chuỗi AD-39 (`core::segment::chapterpattern`).
//!
//! ─────────────────────────────────────────────────────────────────────────────
//! BA MỆNH ĐỀ, đúng khuôn `cleanup_boundary.rs`/`segment_pipeline_boundary.rs`
//! ─────────────────────────────────────────────────────────────────────────────
//! 1. **`core/segment/pipeline.rs` THẬT SỰ gọi `ChapterPattern::match_starts`** — không có ca
//!    này, cổng khác xanh y hệt trên một cây mà `split_chapters_step` đã bị viết lại nội
//!    tuyến hoặc mất hẳn lời gọi tới module này. ⚠️ Vị từ neo vào `pattern.match_starts(`
//!    (LỜI GỌI THẬT), KHÔNG neo vào chuỗi con lỏng lẻo `"chapterpattern::"` — một dòng `use
//!    super::chapterpattern::ChapterPattern;` (cần cho CHỮ KÝ hàm, ví dụ
//!    `Option<&ChapterPattern>`) đã chứa sẵn chuỗi đó mà không hề gọi gì cả; neo lỏng sẽ làm
//!    cổng XANH GIẢ ngay cả khi thân hàm bị gỡ trọn.
//! 2. **`core/segment/**` mang 0 dòng gõ `Store`/`ScopeKind`** — `core::segment` phải ở lại
//!    THUẦN (`segment_boundary.rs::the_splitter_stays_pure` chỉ canh riêng `split.rs`; phép
//!    kiểm này THU HẸP/SIẾT hơn — quét CẢ THƯ MỤC, để `chapterpattern.rs` mới không lặng lẽ
//!    kéo `Store`/`ScopeKind` vào một module lẽ ra phải thuần tuyệt đối).
//! 3. **Chỗ gọi sản phẩm của `chapterpattern::compile` đúng SỐ ĐÃ BIẾT (một)** — ở
//!    `commands::project::resolve_chapter_pattern` (biên dịch thử mẫu regex TRƯỚC khi chạy
//!    pipeline). `chapterpattern.rs` tự gọi lại `compile` bên trong `match_starts` — đó là
//!    chỗ ĐỊNH NGHĨA/uỷ quyền nội bộ, không phải một "chỗ gọi ngoài" theo nghĩa phép kiểm này.
//!
//! Sàn quần thể + kiểm chứng dương (ca dương + ca âm cho MỖI vị từ) là bắt buộc, khuôn
//! `cleanup_boundary.rs`.
//!
//! **Đối chứng đỏ (Verification của spec 6.6):** `git stash` tệp này rồi `cargo test --locked`
//! phải XANH (mệnh đề mới chưa ai canh); gỡ lời gọi `chapterpattern::` khỏi `pipeline.rs` rồi
//! chạy LẠI tệp này (không `git stash`) phải ĐỎ ở mệnh đề 1.

use std::fs;

#[path = "support/boundary_scan.rs"]
#[allow(dead_code)] // shared module: not every helper is used in this file
mod boundary_scan;
use boundary_scan::{code_lines, is_inside, src_root};

/// Thư mục ĐỊNH NGHĨA module — không phải một "chỗ gọi ngoài" cho mệnh đề 3.
const CHAPTERPATTERN_FILE: &str = "core/segment/chapterpattern.rs";

/// Thư mục phải ở lại THUẦN — mệnh đề 2.
const SEGMENT_DIR: &str = "core/segment";

/// Số tệp `.rs` tối thiểu dưới `src-tauri/src/**` — cùng lý lẽ mọi `*_boundary.rs` khác.
const SRC_RS_FLOOR: usize = 84;

fn all_rust_sources() -> Vec<(String, String)> {
    boundary_scan::rust_sources(&src_root())
}

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

// ═════════════════════════════════════════════════════════════════════════════════
// Mệnh đề 1 — pipeline.rs THẬT SỰ gọi chapterpattern::
// ═════════════════════════════════════════════════════════════════════════════════

/// `code` gọi `.match_starts(` — LỜI GỌI THẬT vào thân mẫu phân tách, KHÔNG phải một chuỗi
/// con lỏng lẻo như `"chapterpattern::"` (một dòng `use` cũng chứa chuỗi đó mà không gọi gì —
/// xem doc-comment đầu tệp). Vị từ THUẦN, dùng bởi CẢ cổng thật lẫn đối chứng dương.
fn line_calls_chapterpattern_match_starts(code: &str) -> bool {
    code.contains(".match_starts(")
}

#[test]
fn the_pipeline_module_actually_calls_the_chapterpattern_module() {
    let text = fs::read_to_string(src_root().join("core/segment/pipeline.rs"))
        .expect("đọc core/segment/pipeline.rs");
    let product_only = boundary_scan::without_test_modules(&text);
    let has_call = code_lines(&product_only).any(|(_, code)| line_calls_chapterpattern_match_starts(&code));
    assert!(
        has_call,
        "`core/segment/pipeline.rs` không còn gọi `ChapterPattern::match_starts` (ngoài chú \
         thích, ngoài khối `#[cfg(test)]`) — `split_chapters_step` (bước 5 của chuỗi AD-39) \
         phải GỌI thân thật của mẫu phân tách, không viết lại nội tuyến hay để trống."
    );
}

/// Đối chứng dương: một lời gọi CHỈ sống trong khối `#[cfg(test)]` của CHÍNH `pipeline.rs`
/// không được tính — cùng ca vòng rà mà `cleanup_boundary.rs` đã bắt cho `cleanup::apply`.
#[test]
fn a_call_living_only_inside_the_pipeline_files_own_test_block_does_not_count_as_the_real_call() {
    let seeded_file = "fn step() {\n    // than that da bi go, khong con goi match_starts nua\n}\n\n#[cfg(test)]\nmod tests {\n    #[test]\n    fn seeded() {\n        let _ = super::chapterpattern::ChapterPattern::literal(\"x\").match_starts(\"y\");\n    }\n}\n";
    let product_only = boundary_scan::without_test_modules(seeded_file);
    let has_call = code_lines(&product_only).any(|(_, code)| line_calls_chapterpattern_match_starts(&code));
    assert!(
        !has_call,
        "một lời gọi CHỈ sống trong khối `#[cfg(test)]` không được tính là 'pipeline.rs gọi \
         match_starts' — quét TRẦN sẽ đọc ra CÓ gọi đúng lúc thân sản phẩm thật đã bị gỡ"
    );
}

/// Đối chứng dương: [`line_calls_chapterpattern_match_starts`] NỔ được trên một dòng vi phạm
/// dựng tay, và KHÔNG nổ oan trên một dòng bình thường không nhắc tới nó — kể cả một dòng
/// `use` mang sẵn chuỗi `"chapterpattern::"` mà không gọi gì (đối chứng cho lý do đổi vị từ).
#[test]
fn the_chapterpattern_call_check_would_actually_flag_a_seeded_violation_and_ignore_clean_code() {
    assert!(
        line_calls_chapterpattern_match_starts("    let starts = pattern.match_starts(text)?;"),
        "ca DUONG: mot dong goi .match_starts( phai bi vi tu bat"
    );
    assert!(
        !line_calls_chapterpattern_match_starts("use super::chapterpattern::ChapterPattern;"),
        "ca AM: mot dong `use` mang san chuoi \"chapterpattern::\" nhung KHONG goi gi khong \
         duoc bi bat oan -- day dung ly do vi tu doi tu chuoi con long leo sang loi goi that"
    );
    assert!(
        !line_calls_chapterpattern_match_starts("    let n = normalize::normalize(&text, &source_lang);"),
        "ca AM: mot dong KHONG goi match_starts khong duoc bi bat oan"
    );
}

// ═════════════════════════════════════════════════════════════════════════════════
// Mệnh đề 2 — core/segment/** mang 0 dòng gõ Store/ScopeKind (thuần tuyệt đối)
// ═════════════════════════════════════════════════════════════════════════════════

/// Hai token bị cấm trong TOÀN `core/segment/**` — module tách văn bản không được biết I/O
/// (`Store`) hay từ vựng phân giải hai tầng (`ScopeKind`). SIẾT hơn
/// `segment_boundary.rs::the_splitter_stays_pure` (chỉ canh `split.rs`): phép kiểm này quét
/// CẢ THƯ MỤC, để một module mới (như `chapterpattern.rs`) không lặng lẽ kéo hai thứ đó vào.
const FORBIDDEN_TOKENS: [&str; 2] = ["Store", "ScopeKind"];

/// `haystack` mang `needle` như một TỪ trọn vẹn — ký tự liền trước/sau (nếu có) không phải
/// một ký tự định danh (`[A-Za-z0-9_]`). Cùng khuôn `cleanup_boundary.rs::contains_word` —
/// điều kiện để `"Store"` không khớp OAN bên trong một định danh dài hơn (`StoreSpec` không
/// xuất hiện trong `core/segment/**` hôm nay, nhưng vị từ vẫn phải đúng theo cấu trúc).
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

/// `code` mang một trong hai token cấm, như một TỪ trọn vẹn — vị từ THUẦN.
fn line_names_a_forbidden_token(code: &str) -> bool {
    FORBIDDEN_TOKENS.iter().any(|needle| contains_word(code, needle))
}

/// Đối chứng dương của [`contains_word`] chính nó.
#[test]
fn contains_word_matches_a_whole_word_but_not_a_substring_of_a_longer_identifier() {
    assert!(contains_word("use crate::core::store::Store;", "Store"), "ca DUONG: tu tron ven");
    assert!(
        !contains_word("let s: StoreSpec = StoreSpec::global(p);", "Store"),
        "ca AM: la chuoi con cua StoreSpec, khong phai mot tu tron ven"
    );
}

#[test]
fn core_segment_carries_zero_lines_naming_store_or_scope_kind() {
    let files = all_rust_sources();

    let mut offenders: Vec<String> = Vec::new();
    let mut segment_files = 0usize;
    for (rel, text) in &files {
        if !is_inside(rel, SEGMENT_DIR) {
            continue;
        }
        segment_files += 1;
        let product_only = boundary_scan::without_test_modules(text);
        for (line, code) in code_lines(&product_only) {
            if line_names_a_forbidden_token(&code) {
                offenders.push(format!("{rel}:{line}  {code}"));
            }
        }
    }

    assert!(
        segment_files > 0,
        "không tệp nào khớp `{SEGMENT_DIR}` — đường dẫn đã lệch khỏi cây nguồn"
    );

    assert!(
        offenders.is_empty(),
        "`core::segment` mang từ vựng của tầng ghi/tầng phân giải hai tầng \
         (`Store`/`ScopeKind`):\n{}\n\n\
         Module này phải ở lại THUẦN — văn bản + luật đã phân giải đi VÀO, không đọc/ghi gì \
         khác (`segment_boundary.rs::the_splitter_stays_pure`).",
        offenders.join("\n")
    );
}

/// Đối chứng dương: [`line_names_a_forbidden_token`] NỔ được trên một dòng vi phạm dựng tay,
/// KHÔNG nổ oan trên một dòng bình thường.
#[test]
fn the_forbidden_token_check_would_actually_flag_a_seeded_violation_and_ignore_clean_code() {
    assert!(
        line_names_a_forbidden_token("    fn f(store: &Store) {}"),
        "ca DƯƠNG: một dòng gõ `Store` phải bị vị từ bắt"
    );
    assert!(
        line_names_a_forbidden_token("    let kind = ScopeKind::ImportCleanupRule;"),
        "ca DƯƠNG: một dòng gõ `ScopeKind` phải bị vị từ bắt"
    );
    assert!(
        !line_names_a_forbidden_token("    pub pattern: String,"),
        "ca ÂM: một dòng bình thường không được bị bắt oan"
    );
}

// ═════════════════════════════════════════════════════════════════════════════════
// Mệnh đề 3 — chỗ gọi sản phẩm của chapterpattern::compile đúng SỐ ĐÃ BIẾT (một)
// ═════════════════════════════════════════════════════════════════════════════════

/// `code` gọi `chapterpattern::compile(` — vị từ THUẦN.
fn line_calls_chapterpattern_compile(code: &str) -> bool {
    code.contains("chapterpattern::compile(")
}

#[test]
fn the_chapterpattern_compile_function_has_exactly_one_named_product_call_site() {
    let files = all_rust_sources();

    let mut sites: Vec<String> = Vec::new();
    for (rel, text) in &files {
        if rel == CHAPTERPATTERN_FILE {
            // Định nghĩa + đối chứng nội bộ (`#[cfg(test)]`) của chính module, cộng chỗ
            // `ChapterPattern::match_starts` uỷ quyền cho `compile` NỘI BỘ — không phải một
            // "chỗ gọi ngoài" theo nghĩa của phép kiểm này.
            continue;
        }
        let product_only = boundary_scan::without_test_modules(text);
        for (line, code) in code_lines(&product_only) {
            if line_calls_chapterpattern_compile(&code) {
                sites.push(format!("{rel}:{line}  {code}"));
            }
        }
    }

    assert_eq!(
        sites.len(),
        1,
        "kỳ vọng ĐÚNG MỘT chỗ gọi sản phẩm của `chapterpattern::compile` (biên dịch thử mẫu \
         regex TRƯỚC khi chạy pipeline), tìm thấy {}:\n{}",
        sites.len(),
        sites.join("\n")
    );
    assert!(
        sites[0].starts_with("commands/project/mod.rs"),
        "chỗ gọi DUY NHẤT phải ở `commands/project/mod.rs::resolve_chapter_pattern` — tìm thấy ở \
         đây thay vì đó: {}",
        sites[0]
    );
}

/// Đối chứng dương: một lời gọi sống CHỈ trong khối `#[cfg(test)]` của MỘT TỆP KHÁC (không
/// phải `chapterpattern.rs`, tệp duy nhất được `continue` bỏ qua HẲN) không được đếm là một
/// "chỗ gọi sản phẩm thứ hai".
#[test]
fn a_call_seeded_only_inside_another_files_test_block_is_not_counted_as_a_second_product_call_site()
 {
    let seeded_file = "fn helper() {}\n\n#[cfg(test)]\nmod tests {\n    #[test]\n    fn seeded() {\n        let _ = crate::core::segment::chapterpattern::compile(\"x\");\n    }\n}\n";
    let product_only = boundary_scan::without_test_modules(seeded_file);
    let found = code_lines(&product_only).any(|(_, code)| line_calls_chapterpattern_compile(&code));
    assert!(
        !found,
        "một lời gọi chỉ tồn tại trong khối `#[cfg(test)]` của MỘT TỆP KHÁC không được đếm là \
         một chỗ gọi SẢN PHẨM thứ hai"
    );
}

/// Đối chứng dương: [`line_calls_chapterpattern_compile`] NỔ được trên một dòng vi phạm dựng
/// tay, và KHÔNG nổ oan trên một dòng bình thường.
#[test]
fn the_chapterpattern_compile_call_check_would_actually_flag_a_seeded_violation_and_ignore_clean_code()
 {
    assert!(
        line_calls_chapterpattern_compile(
            "    if let Err(err) = crate::core::segment::chapterpattern::compile(&wire.pattern) {"
        ),
        "ca DƯƠNG: một dòng gọi `chapterpattern::compile` phải bị vị từ bắt"
    );
    assert!(
        !line_calls_chapterpattern_compile("    let n = normalize::normalize(&text, &source_lang);"),
        "ca ÂM: một dòng KHÔNG gọi `chapterpattern::compile` không được bị bắt oan"
    );
}
