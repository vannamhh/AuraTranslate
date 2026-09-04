//! Ranh giới cây nguồn của Story 6.3 — FR126, bộ dò bảng mã.
//!
//! ─────────────────────────────────────────────────────────────────────────────
//! HAI MỆNH ĐỀ, đúng khuôn `segment_pipeline_boundary.rs`/`segment_boundary.rs`
//! ─────────────────────────────────────────────────────────────────────────────
//! 1. **`FR126_LABELS` khớp ĐÚNG THỨ TỰ PRD** (`prd.md:355`: *"UTF-8 · GB18030 · GBK · Big5
//!    · UTF-16"*) — `assert_eq!` trên mảng, không chỉ sự có mặt: một hoán vị đổi chỗ hai
//!    nhãn vẫn giữ nguyên tập hợp năm phần tử, nên một phép kiểm chỉ đếm/`contains` sẽ xanh
//!    trên một thứ tự SAI (AC1 spec 6.3).
//! 2. **`chardetng` được nêu tên ở ĐÚNG MỘT tệp sản phẩm**, và tệp đó ở `core/segment/` —
//!    đếm CHÍNH XÁC (khuôn `segment_boundary.rs:324`: "đếm chính xác, không kiểm thành
//!    viên"), cộng đối chứng dương rằng phép quét THẬT SỰ bắt được một chỗ khớp gieo tay
//!    (AC2 spec 6.3).
//!
//! Sàn quần thể + kiểm chứng dương là bắt buộc, khuôn `segment_pipeline_boundary.rs`.

use std::fs;
use std::path::{Path, PathBuf};

use auratranslate_lib::core::segment::encoding::FR126_LABELS;

const WEBIMPORT_DIR: &str = "core/webimport";

/// Số tệp `.rs` tối thiểu dưới `src-tauri/src/**` — cùng lý lẽ
/// `segment_pipeline_boundary.rs::SRC_RS_FLOOR`. Story 6.3 thêm `core/segment/encoding.rs`,
/// nên số thật chỉ TĂNG — sàn cũ (50, ~80,6%) vẫn đúng, không hạ.
const SRC_RS_FLOOR: usize = 50;

fn src_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src")
}

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

fn code_lines(text: &str) -> impl Iterator<Item = (usize, &str)> {
    // 🔴 SỬA (vòng rà đối kháng 2, mục 17) — bản trước KHÔNG loại dòng TRẮNG (chuỗi rỗng sau
    // `trim_start`), nên `webimport_code_lines == 0` xanh chỉ vì tệp đó TÌNH CỜ không có
    // dòng trắng nào: thêm một dòng trắng làm cổng ĐỎ vì lý do SAI (dòng trắng không phải
    // mã), còn XOÁ HẲN tệp lại làm cổng XANH vì lý do SAI (không phải vì 0 dòng mã, mà vì 0
    // tệp nào được quét — xem sàn quần thể THEO THƯ MỤC ngay dưới, đóng vế thứ hai).
    text.lines()
        .enumerate()
        .map(|(index, line)| (index + 1, line.trim_start()))
        .filter(|(_, code)| !code.is_empty() && !code.starts_with("//") && !code.starts_with("///"))
}

/// `code` nêu tên `chardetng` — vị từ THUẦN, dùng bởi CẢ cổng thật lẫn đối chứng dương.
fn line_names_chardetng(code: &str) -> bool {
    code.contains("chardetng")
}

// ═════════════════════════════════════════════════════════════════════════════════
// Sàn quần thể
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
// Mệnh đề 1 — FR126_LABELS khớp đúng thứ tự PRD (prd.md:355)
// ═════════════════════════════════════════════════════════════════════════════════

#[test]
fn fr126_labels_match_the_prd_order_exactly() {
    assert_eq!(
        FR126_LABELS,
        ["UTF-8", "GB18030", "GBK", "Big5", "UTF-16"],
        "FR126_LABELS lệch khỏi thứ tự PRD (prd.md:355: \"UTF-8 · GB18030 · GBK · Big5 · \
         UTF-16\"). Đây là SO SÁNH THỨ TỰ — một hoán vị giữ nguyên tập hợp năm nhãn vẫn phải \
         làm assert_eq này đỏ (AC1 spec 6.3: cổng đọc được sự LỆCH, không chỉ sự TỒN TẠI)."
    );
}

// ═════════════════════════════════════════════════════════════════════════════════
// Mệnh đề 2 — chardetng được nêu tên ở ĐÚNG MỘT tệp sản phẩm, ở core/segment/
// ═════════════════════════════════════════════════════════════════════════════════

#[test]
fn chardetng_is_named_in_exactly_one_product_file_inside_core_segment() {
    let files = all_rust_sources();

    let mut sites: Vec<String> = Vec::new();
    let mut files_with_hits: Vec<String> = Vec::new();
    for (rel, text) in &files {
        let mut hit_here = false;
        for (line, code) in code_lines(text) {
            if line_names_chardetng(code) {
                sites.push(format!("{rel}:{line}  {code}"));
                hit_here = true;
            }
        }
        if hit_here {
            files_with_hits.push(rel.clone());
        }
    }

    assert_eq!(
        files_with_hits.len(),
        1,
        "kỳ vọng ĐÚNG MỘT tệp sản phẩm nêu tên `chardetng`, tìm thấy {}:\n{}",
        files_with_hits.len(),
        sites.join("\n")
    );
    assert_eq!(
        files_with_hits[0], "core/segment/encoding.rs",
        "tệp DUY NHẤT nêu tên `chardetng` phải là `core/segment/encoding.rs` — tìm thấy ở \
         đây thay vì đó: {}",
        files_with_hits[0]
    );

    // `core/webimport/` vẫn 0 dòng mã — AC2 spec 6.3: "core/webimport/ vẫn 0 dòng mã". Bàn
    // đo Story 6.1 (`webimport_probe.rs`) nêu tên `chardetng` nhưng nó là `tests/**`, ngoài
    // phạm vi phép quét này (chỉ `src-tauri/src/**`).
    let webimport_files: Vec<&(String, String)> =
        files.iter().filter(|(rel, _)| rel.starts_with(WEBIMPORT_DIR)).collect();
    // 🔴 SỬA (vòng rà đối kháng 2, mục 17) — sàn quần thể RIÊNG cho thư mục con này. Không
    // có nó, xoá HẲN `core/webimport/` làm `webimport_code_lines` cộng trên MỘT DANH SÁCH
    // RỖNG, ra 0 một cách VÔ NGHĨA — cổng "xanh" mà không còn gì để mà kiểm.
    assert!(
        !webimport_files.is_empty(),
        "`core/webimport/` phải có ÍT NHẤT MỘT tệp (`mod.rs`, stub) — 0 tệp làm phép đếm \
         dòng mã ngay dưới xanh một cách VÔ NGHĨA (cộng trên danh sách rỗng), không phải vì \
         mô-đun đó thật sự 0 dòng mã"
    );
    let webimport_code_lines: usize =
        webimport_files.iter().map(|(_, text)| code_lines(text).count()).sum();
    assert_eq!(
        webimport_code_lines, 0,
        "`core/webimport/` phải vẫn 0 dòng mã (stub, Story 6.2/6.7/6.9) — bộ dò bảng mã sống \
         ở `core/segment/encoding.rs`, không mở `core/webimport/` ra"
    );
}

/// Đối chứng dương: [`line_names_chardetng`] NỔ được trên một dòng vi phạm dựng tay, và
/// KHÔNG nổ oan trên một dòng bình thường không nhắc tới nó — khuôn
/// `segment_pipeline_boundary.rs::the_run_import_call_check_would_actually_flag_a_seeded_violation_and_ignore_clean_code`.
#[test]
fn the_chardetng_name_check_would_actually_flag_a_seeded_violation_and_ignore_clean_code() {
    assert!(
        line_names_chardetng("    let mut detector = chardetng::EncodingDetector::new(x);"),
        "ca DƯƠNG: một dòng nêu tên `chardetng` phải bị vị từ bắt"
    );
    assert!(
        !line_names_chardetng("    let mut detector = encoding_rs::UTF_8;"),
        "ca ÂM: một dòng KHÔNG nhắc `chardetng` không được bị bắt oan"
    );
}

// ═════════════════════════════════════════════════════════════════════════════════
// Vòng rà đối kháng 2, mục 5 — `create_work` (hàm THUẦN, điểm GHI duy nhất của cả chuỗi
// AD-39) phải có ĐÚNG BA chỗ gọi sản phẩm, TẤT CẢ trong `commands/project.rs`, TẤT CẢ đã
// biết tên: `create_work_from_text`, `create_work_from_file` (hai hàm thuần cũ, không đi
// qua xem trước — chỉ còn sống nhờ `e2e/**` và `tests/**`), và
// `confirm_import_with_encoding` (đường CÓ xem trước, Story 6.3). Một chỗ gọi THỨ TƯ xuất
// hiện ở bất kỳ đâu là một đường ghi MỚI không ai ký — khuôn
// `segment_pipeline_boundary.rs::run_import_is_the_one_product_call_site`.
// ═════════════════════════════════════════════════════════════════════════════════

/// `code` gọi `create_work(...)` — vị từ THUẦN. Neo bằng `create_work(` (có dấu mở ngoặc)
/// để KHÔNG khớp `create_work_from_text(`/`create_work_from_file(` (giữa `create_work` và
/// `(` của hai tên đó là `_from_text`/`_from_file`, không phải `(` trực tiếp) và KHÔNG khớp
/// chính khai báo `pub fn create_work(` — loại riêng ở vị từ đếm bằng cách bỏ dòng chứa
/// `fn create_work(`.
fn line_calls_create_work(code: &str) -> bool {
    code.contains("create_work(") && !code.contains("fn create_work(")
}

#[test]
fn create_work_has_exactly_three_named_product_call_sites_all_inside_commands_project() {
    let files = all_rust_sources();

    let mut sites: Vec<String> = Vec::new();
    for (rel, text) in &files {
        for (line, code) in code_lines(text) {
            if line_calls_create_work(code) {
                sites.push(format!("{rel}:{line}  {code}"));
            }
        }
    }

    assert_eq!(
        sites.len(),
        3,
        "ky vong DUNG BA cho goi san pham cua `create_work` (create_work_from_text, \
         create_work_from_file, confirm_import_with_encoding), tim thay {}:\n{}",
        sites.len(),
        sites.join("\n")
    );
    for site in &sites {
        assert!(
            site.starts_with("commands/project.rs"),
            "chỗ gọi `create_work` phải ở `commands/project.rs` — tìm thấy ở đây thay vì đó: {site}"
        );
    }
}

/// Đối chứng dương — khuôn `the_chardetng_name_check_would_actually_flag_a_seeded_violation_and_ignore_clean_code`.
#[test]
fn the_create_work_call_check_would_actually_flag_a_seeded_violation_and_ignore_clean_code() {
    assert!(
        line_calls_create_work("    let opened = create_work(&root, name, lang, genre, shape, enc)?;"),
        "ca DƯƠNG: một lời gọi `create_work(...)` phải bị vị từ bắt"
    );
    assert!(
        !line_calls_create_work("    create_work_from_text(&root, name, lang, genre, text)?;"),
        "ca ÂM #1: `create_work_from_text(` KHÔNG được tính là gọi `create_work(`"
    );
    assert!(
        !line_calls_create_work("    create_work_from_file(&root, name, lang, genre, path)?;"),
        "ca ÂM #2: `create_work_from_file(` KHÔNG được tính là gọi `create_work(`"
    );
    assert!(
        !line_calls_create_work("pub fn create_work(documents_root: &Path, name: &str) -> Result<OpenWork, IpcError> {"),
        "ca ÂM #3: chính khai báo `fn create_work(` KHÔNG được tính là một chỗ GỌI"
    );
}
