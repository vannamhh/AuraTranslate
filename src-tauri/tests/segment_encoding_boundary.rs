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

    // 🔵 **SỬA 2026-09-06 (Story 6.7) — "`core/webimport/` vẫn 0 dòng mã" đã HẾT ĐÚNG.**
    // Mệnh đề gốc (AC2 spec 6.3) khai đúng cho trạng thái *"stub, chưa story nào cần đến
    // module"* — Story 6.7 mở nó ra thật (`Fetcher`/`Extractor`), nên đếm-dòng-bằng-0 giờ chỉ
    // còn ĐÚNG là *"đã ĐÓNG mãi mãi"*, không phải hiện trạng. Mệnh đề THẬT vẫn giữ nguyên ý
    // nghĩa ban đầu của phép kiểm này (Mệnh đề 2 — "`chardetng` ở ĐÚNG MỘT tệp, không rò rỉ
    // sang module khác"): `core/webimport/` (module bóc nội dung + tải mạng CỦA RIÊNG NÓ, AD-
    // 40) không được PHÉP tự dò bảng mã bằng `chardetng` — bộ dò bảng mã CHỈ sống ở
    // `core/segment/encoding.rs`, và điều đó đã được khẳng định ở `files_with_hits[0]` phía
    // trên (đúng MỘT tệp, đúng đường dẫn đó). Giữ lại một khẳng định HẸP hơn, vẫn đúng nghĩa:
    // `core/webimport/` không nêu tên `chardetng` — cách diễn đạt CŨ ("0 dòng mã") kiểm được
    // điều này chỉ vì nó kiểm NHIỀU HƠN mức cần (0 dòng mã nói riêng ⇒ 0 dòng gõ `chardetng`
    // nói chung), và phần "nhiều hơn" đó nay sai với thực tế đã ký (Story 6.7 §Intent).
    let webimport_files: Vec<&(String, String)> =
        files.iter().filter(|(rel, _)| rel.starts_with(WEBIMPORT_DIR)).collect();
    assert!(
        !webimport_files.is_empty(),
        "`core/webimport/` phải có ÍT NHẤT MỘT tệp — 0 tệp làm phép quét ngay dưới xanh một \
         cách VÔ NGHĨA (quét trên danh sách rỗng), không phải vì module đó thật sự sạch"
    );
    let webimport_chardetng_hits: Vec<String> = webimport_files
        .iter()
        .flat_map(|(rel, text)| {
            code_lines(text)
                .filter(|(_, code)| line_names_chardetng(code))
                .map(move |(line, code)| format!("{rel}:{line}  {code}"))
        })
        .collect();
    assert!(
        webimport_chardetng_hits.is_empty(),
        "`core/webimport/` không được nêu tên `chardetng` — bộ dò bảng mã sống DUY NHẤT ở \
         `core/segment/encoding.rs` (đã khẳng định ở assert phía trên), tìm thấy:\n{}",
        webimport_chardetng_hits.join("\n")
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
//
// 🔵 SỬA 2026-09-11 (Story 6.16) — từ BA lên BỐN, chỗ gọi thứ tư CÓ TÊN: `confirm_bilingual_import`
// (đường xác nhận nhập song ngữ, cùng file `commands/project.rs`, cùng vai trò với
// `confirm_import_with_encoding` — "điểm GHI duy nhất" cho đường `.csv`/`.tsv`, không phải
// một đường ghi thứ hai không ai ký).
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
fn create_work_has_exactly_four_named_product_call_sites_all_inside_commands_project() {
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
        4,
        "ky vong DUNG BON cho goi san pham cua `create_work` (create_work_from_text, \
         create_work_from_file, confirm_import_with_encoding, confirm_bilingual_import), \
         tim thay {}:\n{}",
        sites.len(),
        sites.join("\n")
    );
    for site in &sites {
        assert!(
            site.starts_with("commands/project/mod.rs"),
            "chỗ gọi `create_work` phải ở `commands/project/mod.rs` — tìm thấy ở đây thay vì đó: {site}"
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
