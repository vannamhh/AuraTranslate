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
use std::path::{Path, PathBuf};

use auratranslate_lib::core::segment::pipeline::{PIPELINE_ORDER, Step};

/// Thư mục ĐỊNH NGHĨA bộ chạy — không phải một "chỗ gọi ngoài".
const SEGMENT_DIR: &str = "core/segment";

/// Số tệp `.rs` tối thiểu dưới `src-tauri/src/**` để phép quét là thật.
///
/// Số thật lúc dựng (Story 6.2, 2026-09-04, sau khi thêm `core/segment/pipeline.rs`):
/// **62** tệp. Sàn **50** (~80,6%, cùng khuôn 80-85% mà các tệp `*_boundary.rs` khác dùng) —
/// bắt một cây bị cắt mất, không bắt việc thêm tệp mới.
const SRC_RS_FLOOR: usize = 50;

fn src_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src")
}

/// Đường dẫn tương đối, dùng dấu `/` trên cả hai nền tảng — NFR14, cùng bài học
/// `segment_boundary.rs::rel_posix`.
fn rel_posix(root: &Path, file: &Path) -> String {
    file.strip_prefix(root).unwrap_or(file).to_string_lossy().replace('\\', "/")
}

fn walk(dir: &Path, out: &mut Vec<PathBuf>) {
    let entries = fs::read_dir(dir).unwrap_or_else(|e| panic!("đọc {}: {e}", dir.display()));
    for entry in entries {
        let entry = entry.unwrap_or_else(|e| panic!("duyệt {}: {e}", dir.display()));
        let path = entry.path();
        let meta =
            fs::symlink_metadata(&path).unwrap_or_else(|e| panic!("lstat {}: {e}", path.display()));

        // ⚠️ `symlink_metadata`, không `metadata` — cùng bài học các tệp `*_boundary.rs`
        // khác: `metadata` giải symlink, một liên kết trỏ về thư mục cha làm đệ quy không
        // dừng.
        if meta.file_type().is_symlink() {
            continue;
        }
        if meta.is_dir() {
            walk(&path, out);
        } else if path.extension().and_then(|e| e.to_str()) == Some("rs") {
            out.push(path);
        }
    }
}

/// Mọi tệp `.rs` dưới `src-tauri/src/**`, kèm đường dẫn tương đối kiểu POSIX và nội dung.
fn all_rust_sources() -> Vec<(String, String)> {
    let root = src_root();
    let mut files = Vec::new();
    walk(&root, &mut files);
    files.sort();

    files
        .into_iter()
        .map(|file| {
            let rel = rel_posix(&root, &file);
            let text =
                fs::read_to_string(&file).unwrap_or_else(|e| panic!("đọc {}: {e}", file.display()));
            (rel, text)
        })
        .collect()
}

/// Dòng **mã** của một khối văn bản: `(số dòng 1-based, nội dung đã trim đầu)`. Chỉ dòng bắt
/// đầu bằng `//` bị bỏ qua — một doc-comment giải thích một ranh giới không phải một lời gọi
/// vượt qua nó, cùng luật mọi tệp `*_boundary.rs` khác áp.
fn code_lines(text: &str) -> impl Iterator<Item = (usize, &str)> {
    text.lines()
        .enumerate()
        .map(|(index, line)| (index + 1, line.trim_start()))
        .filter(|(_, code)| !code.starts_with("//"))
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
    assert!(
        files.len() >= SRC_RS_FLOOR,
        "chỉ tìm thấy {} tệp `.rs` dưới `src-tauri/src/**` (sàn {SRC_RS_FLOOR}). Cây quá nhỏ \
         để là thật — một danh sách rỗng làm mọi phép kiểm dưới đây xanh mà không kiểm gì cả.",
        files.len()
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
        if rel.starts_with(SEGMENT_DIR) {
            // Cả hai hàm ĐỊNH NGHĨA/uỷ quyền ở đây — không phải một "chỗ gọi ngoài" theo
            // nghĩa của phép kiểm này.
            continue;
        }
        for (line, code) in code_lines(text) {
            if line_calls_run_import_with_order(code) {
                run_import_with_order_sites.push(format!("{rel}:{line}  {code}"));
            }
            if line_calls_run_import(code) {
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
        run_import_sites[0].starts_with("commands/project.rs"),
        "chỗ gọi DUY NHẤT phải ở `commands/project.rs::create_work` — tìm thấy ở đây thay vì \
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
