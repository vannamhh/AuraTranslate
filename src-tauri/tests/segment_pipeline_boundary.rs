//! Ranh giới cây nguồn của Story 6.2 — AD-39: thứ tự pipeline nhập là DỮ LIỆU, và
//! `run_import` là chỗ gọi sản phẩm DUY NHẤT của bộ chạy nhận thứ tự tuỳ ý.
//!
//! ⚠️ Tệp riêng có chủ ý, đúng khuôn `segment_boundary.rs`/`ai_boundary.rs`: đây là phép
//! kiểm **tĩnh trên cây nguồn** (chỗ gọi, thứ tự khai báo); hành vi LÚC CHẠY của chuỗi
//! (AD-39 symptom, N Chương, bỏ qua bước theo hình dạng) sống ở
//! `segment_contract.rs` §Story 6.2.
//!
//! ─────────────────────────────────────────────────────────────────────────────
//! 🔴 HAI MỆNH ĐỀ, VÀ MỖI MỆNH ĐỀ HỎNG BẰNG ĐÚNG MỘT DÒNG MÀ MỌI THỨ KHÁC VẪN XANH
//! ─────────────────────────────────────────────────────────────────────────────
//! 1. **`PIPELINE_ORDER` khớp đúng thứ tự AD-39** (spine `:473-482`) — [`pipeline_order_matches_ad_39_step_by_step`]
//!    so sánh THỨ TỰ, không chỉ SỰ CÓ MẶT: một hoán vị đổi chỗ hai phần tử vẫn giữ nguyên
//!    tập hợp bảy biến thể, nên một phép kiểm chỉ đếm/`contains` sẽ xanh trên một thứ tự
//!    SAI. `assert_eq!` trên mảng làm đúng việc này.
//! 2. **`run_import` là chỗ gọi sản phẩm DUY NHẤT của bộ chạy nhận thứ tự tuỳ ý** — nếu một
//!    chỗ gọi sản phẩm thứ hai xuất hiện (gọi thẳng `run_import_with_order` với một thứ tự
//!    tự chế), cái seam mà [`run_import_with_order`] mở ra cho `tests/**` sẽ thành đường
//!    tắt cho một story sau âm thầm dùng một thứ tự KHÁC `PIPELINE_ORDER` mà không ai ký.
//!
//! ─────────────────────────────────────────────────────────────────────────────
//! 🔴 SÀN QUẦN THỂ + ĐỐI CHỨNG DƯƠNG LÀ BẮT BUỘC — khuôn `ai_boundary.rs:225,302`
//! ─────────────────────────────────────────────────────────────────────────────
//! *"Cây rỗng đọc thành sạch"*: một gốc quét sai làm `walk` khớp 0 tệp, và khi đó phép quét
//! chỗ gọi xanh mà không kiểm gì cả. Đối chứng dương chạy vị từ THẬT trên một chuỗi vi phạm
//! dựng tay — phân biệt *"không ai vi phạm"* (cây sạch, phép quét còn thấy được) với *"không
//! có gì để vi phạm"* (phép quét đã mù).

use std::fs;

use auratranslate_lib::core::segment::pipeline::{PIPELINE_ORDER, Step};

#[path = "support/boundary_scan.rs"]
#[allow(dead_code)] // shared module: not every helper is used in this file
mod boundary_scan;
use boundary_scan::{code_lines, is_inside, src_root};

/// Thư mục ĐỊNH NGHĨA bộ chạy — không phải một "chỗ gọi ngoài".
const SEGMENT_DIR: &str = "core/segment";

/// Số tệp `.rs` tối thiểu dưới `src-tauri/src/**` để phép quét là thật.
const SRC_RS_FLOOR: usize = 84;

/// Mọi tệp `.rs` dưới `src-tauri/src/**`, kèm đường dẫn tương đối kiểu POSIX và nội dung.
fn all_rust_sources() -> Vec<(String, String)> {
    boundary_scan::rust_sources(&src_root())
}



/// `code` gọi `run_import(...)` — vị từ THUẦN, dùng bởi CẢ cổng thật lẫn ca gieo vi phạm
/// tổng hợp. Neo bằng `run_import(` (có dấu mở ngoặc) để KHÔNG khớp `run_import_with_order(`
/// — xem [`line_calls_run_import_with_order`] ngay dưới cho vế THỨ HAI, cấm RIÊNG.
fn line_calls_run_import(code: &str) -> bool {
    code.contains("run_import(")
}

/// `code` gọi `run_import_with_order(...)` — vị từ THUẦN thứ hai.
///
/// 🔴 **SỬA (vòng rà đối kháng 2026-09-04, item 1) — bản đầu CHỈ có [`line_calls_run_import`],
/// và doc-comment của `run_import_with_order` (`pipeline.rs`) tuyên bố cấm "một chỗ gọi SẢN
/// PHẨM thứ hai... dù gọi qua tên nào" trong khi mã chỉ canh ĐÚNG MỘT tên.** `run_import_with_order(`
/// KHÔNG chứa chuỗi `run_import(` (ký tự ngay sau `run_import` là `_`, không phải `(`), nên
/// một chỗ gọi sản phẩm MỚI tới thẳng `run_import_with_order` với một thứ tự tự chế (ví dụ
/// một `commands/import_url.rs` đặt `SplitChapters` trước `DecodeEncoding`) đi lọt HOÀN
/// TOÀN qua cổng cũ — đúng lớp lỗi mà cả câu chuyện AD-39 tồn tại để chặn, mở lại ngay tại
/// cổng vào của chính seam này. `run_import_with_order` là seam CÔNG KHAI cho `tests/**`
/// (được phép xuất hiện ở bất kỳ đâu dưới `tests/`), nhưng KHÔNG được xuất hiện trong
/// `src/**` ngoài `core/segment/`.
fn line_calls_run_import_with_order(code: &str) -> bool {
    code.contains("run_import_with_order(")
}

// ═════════════════════════════════════════════════════════════════════════════════
// Sàn quần thể — chạy TRƯỚC mọi phép kiểm khác
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
// Mệnh đề 1 — PIPELINE_ORDER khớp đúng thứ tự AD-39 (spine :473-482)
// ═════════════════════════════════════════════════════════════════════════════════

#[test]
fn pipeline_order_matches_ad_39_step_by_step() {
    assert_eq!(
        PIPELINE_ORDER,
        [
            Step::DecodeEncoding,
            Step::ExtractMainContent,
            Step::CleanByRules,
            Step::NormalizeParagraphsAndWhitespace,
            Step::SplitChapters,
            Step::Preview,
            Step::SplitSegments,
        ],
        "PIPELINE_ORDER lệch khỏi bảy bước AD-39 (spine :473-482). Đây là SO SÁNH THỨ TỰ, \
         không phải kiểm sự có mặt — một hoán vị giữ nguyên tập hợp bảy biến thể vẫn phải \
         làm assert_eq này đỏ, vì đó chính là điều AC2 của spec 6.2 đòi ('cổng đọc được sự \
         lệch chứ không chỉ đọc được sự tồn tại')."
    );
}

// ═════════════════════════════════════════════════════════════════════════════════
// Mệnh đề 2 — run_import là chỗ gọi sản phẩm DUY NHẤT của bộ chạy nhận thứ tự tuỳ ý
// ═════════════════════════════════════════════════════════════════════════════════

#[test]
fn run_import_is_the_one_product_call_site() {
    let files = all_rust_sources();

    let mut run_import_sites: Vec<String> = Vec::new();
    let mut run_import_with_order_sites: Vec<String> = Vec::new();
    for (rel, text) in &files {
        if is_inside(rel, SEGMENT_DIR) {
            // Cả hai hàm ĐỊNH NGHĨA/uỷ quyền ở đây — không phải một "chỗ gọi ngoài" theo
            // nghĩa của phép kiểm này.
            continue;
        }
        let product_only = boundary_scan::without_test_modules(text);
        for (line, code) in code_lines(&product_only) {
            if line_calls_run_import_with_order(&code) {
                run_import_with_order_sites.push(format!("{rel}:{line}  {code}"));
            }
            if line_calls_run_import(&code) {
                run_import_sites.push(format!("{rel}:{line}  {code}"));
            }
        }
    }

    assert!(
        run_import_with_order_sites.is_empty(),
        "bộ chạy nhận thứ tự tuỳ ý (`run_import_with_order`) bị gọi NGOÀI `core/segment/`:\n{}\n\n\
         Nó là seam mở cho `tests/**`, không phải một đường sản phẩm thứ hai — mọi lệnh sản \
         phẩm phải đi qua `run_import` (luôn khoá `PIPELINE_ORDER`), không được tự chọn thứ tự.",
        run_import_with_order_sites.join("\n")
    );

    assert_eq!(
        run_import_sites.len(),
        1,
        "kỳ vọng ĐÚNG MỘT chỗ gọi sản phẩm của `run_import`, tìm thấy {}:\n{}",
        run_import_sites.len(),
        run_import_sites.join("\n")
    );
    assert!(
        run_import_sites[0].starts_with("commands/project/mod.rs"),
        "chỗ gọi DUY NHẤT phải ở `commands/project/mod.rs::create_work` — tìm thấy ở đây thay vì \
         đó:\n{}",
        run_import_sites[0]
    );
}

/// Đối chứng dương: [`line_calls_run_import`] NỔ được trên một dòng vi phạm dựng tay, và
/// KHÔNG nổ oan trên một dòng `use` (không phải một lời gọi) — khuôn
/// `ai_boundary.rs::the_bare_dependency_check_would_actually_flag_a_seeded_violation_and_ignore_clean_code`.
#[test]
fn the_run_import_call_check_would_actually_flag_a_seeded_violation_and_ignore_clean_code() {
    assert!(
        line_calls_run_import("    let out = crate::core::segment::pipeline::run_import(input)?;"),
        "ca DUONG THAT: mot loi goi `run_import(...)` phai bi vi tu bat"
    );
    assert!(
        !line_calls_run_import("use crate::core::segment::pipeline::run_import;"),
        "ca AM: mot dong `use` dua ten vao pham vi, KHONG phai mot loi goi"
    );
}

/// Đối chứng dương cho vế `run_import_with_order` (item 1, vòng rà đối kháng 2026-09-04):
/// bản đầu của cổng này chỉ bắt `run_import(`, và `run_import_with_order(` không chứa chuỗi
/// đó — một chỗ gọi sản phẩm mới tới thẳng `run_import_with_order` với một thứ tự tự chế đi
/// lọt HOÀN TOÀN. Ca này chứng minh vị từ MỚI thật sự bắt được đúng hình dạng đó, và không
/// bắt oan một lời gọi `run_import(` thường (không `_with_order`).
#[test]
fn the_run_import_with_order_call_check_would_actually_flag_a_seeded_violation_and_ignore_clean_code()
 {
    assert!(
        line_calls_run_import_with_order(
            "    let out = run_import_with_order(&MY_CUSTOM_ORDER, input)?;"
        ),
        "ca DUONG THAT: mot loi goi `run_import_with_order(...)` tu mot duong san pham phai bi \
         vi tu bat -- day dung hinh dang ma mot story sau se viet neu no tu chon thu tu"
    );
    assert!(
        !line_calls_run_import_with_order(
            "use crate::core::segment::pipeline::run_import_with_order;"
        ),
        "ca AM: mot dong `use` dua ten vao pham vi, KHONG phai mot loi goi"
    );
    assert!(
        !line_calls_run_import_with_order(
            "    let out = crate::core::segment::pipeline::run_import(input)?;"
        ),
        "ca AM: mot loi goi `run_import(...)` (KHONG co `_with_order`) khong duoc bi vi tu nay \
         bat oan"
    );
}

/// Đối chứng dương thứ hai: `core/segment/pipeline.rs` thật sự ĐỊNH NGHĨA `run_import` —
/// không có ca này, mệnh đề 2 xanh y hệt trên một cây mà hàm đã bị xoá (0 chỗ gọi sản phẩm
/// là đúng, nhưng vì lý do sai).
///
/// 🔵 **SỬA (vòng rà đối kháng 2026-09-04, item 12)** — bản đầu dùng `text.contains(..)` trên
/// TOÀN VĂN BẢN tệp, nên một lời gọi bị comment hoặc một dòng doc-comment nhắc tới chuỗi đó
/// vẫn giữ ca này xanh. Lọc chú thích qua [`code_lines`], đúng khuôn mọi phép kiểm khác của
/// tệp này.
#[test]
fn the_pipeline_module_actually_defines_run_import() {
    let text = fs::read_to_string(src_root().join("core/segment/pipeline.rs"))
        .expect("đọc core/segment/pipeline.rs thất bại");
    let has_def = code_lines(&text).any(|(_, code)| code.contains("pub fn run_import("));
    assert!(
        has_def,
        "`core/segment/pipeline.rs` không còn định nghĩa `pub fn run_import` (ngoài chú \
         thích) -- bộ chạy sản phẩm đã biến mất"
    );
}

/// **THÊM Story 6.7** — bước 2 (`Step::ExtractMainContent`) nay có THÂN THẬT, cùng khuôn
/// mệnh đề "gọi xuống, đừng chép lại" mà `cleanup_boundary.rs::the_pipeline_module_actually_calls_the_cleanup_module`
/// (bước 3) và `segment_normalize_boundary.rs::the_pipeline_module_actually_calls_the_normalize_module`
/// (bước 4) đã dựng. Cắt về PHẦN SẢN PHẨM trước khi quét (`boundary_scan::without_test_modules`)
/// — cùng lý do hai mệnh đề anh em: một lời gọi CHỈ sống trong khối `#[cfg(test)]` của chính
/// `pipeline.rs` không được tính là "pipeline.rs gọi `webimport::extract`".
fn line_calls_webimport_extract(code: &str) -> bool {
    code.contains("webimport::extract(")
}

#[test]
fn the_pipeline_module_actually_calls_the_webimport_module() {
    let text = fs::read_to_string(src_root().join("core/segment/pipeline.rs"))
        .expect("đọc core/segment/pipeline.rs thất bại");
    let product_only = boundary_scan::without_test_modules(&text);
    let has_call = code_lines(&product_only).any(|(_, code)| line_calls_webimport_extract(&code));
    assert!(
        has_call,
        "`core/segment/pipeline.rs` không gọi `webimport::extract` (ngoài chú thích, ngoài \
         khối `#[cfg(test)]`) — bước 2 của chuỗi AD-39 (\"bóc nội dung chính\") phải GỌI thân \
         thật, không viết lại nội tuyến hay để trống."
    );
}

/// Đối chứng dương — [`line_calls_webimport_extract`] nổ được trên một dòng vi phạm dựng
/// tay, và KHÔNG nổ oan trên một dòng bình thường không nhắc tới nó.
#[test]
fn the_webimport_extract_call_check_would_actually_flag_a_seeded_violation_and_ignore_clean_code() {
    assert!(
        line_calls_webimport_extract(
            "                                let extracted = crate::core::webimport::extract(&html, label)?;"
        ),
        "ca DƯƠNG: một dòng gọi `webimport::extract` phải bị vị từ bắt"
    );
    assert!(
        !line_calls_webimport_extract("    let n = normalize::normalize(&text, &source_lang);"),
        "ca ÂM: một dòng KHÔNG gọi `webimport::extract` không được bị bắt oan"
    );
}

/// Đối chứng dương THỨ HAI — một lời gọi CHỈ sống trong khối `#[cfg(test)]` của CHÍNH tệp
/// không được tính là "pipeline.rs gọi webimport::extract" (cùng bẫy mà `cleanup_boundary.rs`
/// đã bắt cho bước 3).
#[test]
fn a_call_living_only_inside_the_pipeline_files_own_test_block_does_not_count_as_the_webimport_call() {
    let seeded_file = "fn step() {\n    // than that da bi go, khong con goi webimport::extract nua\n}\n\n#[cfg(test)]\nmod tests {\n    #[test]\n    fn seeded() {\n        let _ = crate::core::webimport::extract(\"x\", \"y\");\n    }\n}\n";
    let product_only = boundary_scan::without_test_modules(seeded_file);
    let has_call = code_lines(&product_only).any(|(_, code)| line_calls_webimport_extract(&code));
    assert!(
        !has_call,
        "một lời gọi CHỈ sống trong khối `#[cfg(test)]` không được tính là 'pipeline.rs gọi \
         webimport::extract'"
    );
}
