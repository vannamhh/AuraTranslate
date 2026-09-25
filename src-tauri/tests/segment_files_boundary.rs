//! Ranh giới cây nguồn của Story 6.6b — `PipelineShape::Files` (nhập N tệp cùng lúc).
//!
//! ⚠️ Tệp riêng có chủ ý, đúng khuôn `docx_boundary.rs`/`segment_pipeline_boundary.rs`: đây
//! là phép kiểm **tĩnh trên cây nguồn** (chỗ gọi, vốn từ vựng) cộng MỘT phép kiểm hành vi lúc
//! chạy (mệnh đề 3) mà không tệp `*_contract.rs` nào khác sở hữu — nó là bất biến CỦA CHÍNH
//! seam này (per-unit isolation), không phải hành vi ghi/tra cứu mà `segment_contract.rs`
//! đã sở hữu.
//!
//! ─────────────────────────────────────────────────────────────────────────────
//! BA MỆNH ĐỀ
//! ─────────────────────────────────────────────────────────────────────────────
//! 1. **`PipelineShape::Files(` được XÂY (không phải khớp mẫu) ở đúng MỘT chỗ gọi sản
//!    phẩm** — `core::segment::import::import_files`. Nếu một chỗ gọi thứ hai xuất hiện
//!    (một story sau tự dựng `Files` bằng tay ở `commands/**`), điều đó phá đúng bất biến
//!    §Always spec 6.6b "N files are one pending import" mà `import_files` là cổng vào DUY
//!    NHẤT canh giữ.
//! 2. **`core/segment/**` mang 0 dòng gõ từ vựng `store`/`scope`** (AD-1: module thuần,
//!    không I/O/`ScopeResolver`) — bề mặt MỚI của story này (`split_chapters_step_files`,
//!    `import_files`) là chỗ dễ nhất để một lượt "tiện tay" kéo `core::store`/`core::scope`
//!    vào một module lẽ ra phải thuần.
//! 3. **Một đơn vị `Files` KHÔNG BAO GIỜ mang số của đơn vị khác** — báo cáo làm sạch và số
//!    nối dòng của Chương *i* phải là số ĐO ĐƯỢC trên CHÍNH đơn vị *i*, không phải một số bị
//!    hoán đổi/dùng chung giữa các đơn vị (§Always spec 6.6b).
//!
//! ─────────────────────────────────────────────────────────────────────────────
//! 🔴 SÀN QUẦN THỂ + ĐỐI CHỨNG DƯƠNG LÀ BẮT BUỘC — khuôn `docx_boundary.rs`
//! ─────────────────────────────────────────────────────────────────────────────
//! *"Cây rỗng đọc thành sạch"*: một gốc quét sai làm phép quét khớp 0 tệp, và khi đó phép
//! quét chỗ gọi xanh mà không kiểm gì cả. Mỗi vị từ tĩnh có một đối chứng DƯƠNG (một vi phạm
//! dựng tay phải bị bắt) VÀ một đối chứng ÂM (mã sạch không bị bắt oan).

use std::fs;

use auratranslate_lib::core::cleanup::{CleanupRule, CleanupRuleKind, CleanupRuleTier};
use auratranslate_lib::core::segment::pipeline::{ChapterInput, PipelineInput, PipelineShape, run_import};

#[path = "support/boundary_scan.rs"]
#[allow(dead_code)] // shared module: not every helper is used in this file
mod boundary_scan;
use boundary_scan::{code_lines, is_inside, src_root};

/// Thư mục ĐỊNH NGHĨA cả bộ chạy lẫn hình dạng `PipelineShape::Files` — không phải một "chỗ
/// gọi ngoài" theo nghĩa mệnh đề 1, và là PHẠM VI quét của mệnh đề 2.
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

#[test]
fn pointing_the_scan_root_at_an_empty_directory_yields_zero_files_not_a_silent_green() {
    let empty =
        std::env::temp_dir().join(format!("segment_files_boundary_empty_probe_{}", std::process::id()));
    fs::create_dir_all(&empty).unwrap_or_else(|e| panic!("tạo {}: {e}", empty.display()));
    let files = boundary_scan::rust_sources(&empty);
    assert_eq!(files.len(), 0, "thư mục vừa tạo phải rỗng — nếu không, phép đo bên dưới vô nghĩa");
    let _ = fs::remove_dir_all(&empty);
}

// ═════════════════════════════════════════════════════════════════════════════════
// Mệnh đề 1 — `PipelineShape::Files(` được XÂY ở đúng MỘT chỗ gọi sản phẩm
// ═════════════════════════════════════════════════════════════════════════════════

/// `code` chứa một lượt XÂY `PipelineShape::Files(...)` — KHÁC một PATTERN khớp mẫu trong
/// `match`. Vị từ HẸP ĐÚNG BẰNG HÌNH DẠNG THẬT (cùng mức "hẹp có chủ ý" mà
/// `docx_boundary.rs::line_indexes_a_slice_raw` đã chấp nhận): mọi chỗ khớp mẫu THẬT trong kho
/// hôm nay viết trên MỘT DÒNG dạng `PipelineShape::Files(<tên>) => …` — `=>` LUÔN có mặt
/// trên CHÍNH dòng đó. Một lượt XÂY (`PipelineShape::Files(inputs)` gán vào một trường/biến)
/// không bao giờ mang `=>` trên dòng của nó. Vị từ vì thế là: dòng chứa
/// `PipelineShape::Files(` VÀ KHÔNG chứa `=>` ở bất kỳ đâu trên CHÍNH dòng đó.
fn line_constructs_files_shape(code: &str) -> bool {
    code.contains("PipelineShape::Files(") && !code.contains("=>")
}

#[test]
fn the_files_shape_construction_predicate_would_actually_flag_a_construction_and_ignore_a_match_arm() {
    assert!(
        line_constructs_files_shape("    Ok(FilesImportOutcome { shape: PipelineShape::Files(inputs), docx_sidecar: None, items })"),
        "ca DƯƠNG: một lượt XÂY (gán vào trường) phải bị vị từ bắt"
    );
    assert!(
        line_constructs_files_shape("let shape = PipelineShape::Files(units);"),
        "ca DƯƠNG: một lượt XÂY gán thẳng vào biến phải bị vị từ bắt"
    );
    assert!(
        !line_constructs_files_shape("        PipelineShape::Files(units) => cs.iter().map(chapter_input_page_url).collect(),"),
        "ca ÂM: một PATTERN khớp mẫu trong `match` (mang `=>` trên dòng) không được bị bắt oan"
    );
    assert!(
        !line_constructs_files_shape("        PipelineShape::Files(_) => None,"),
        "ca ÂM: một PATTERN bỏ qua nội dung (`_`) cũng không được bị bắt oan"
    );
    assert!(
        !line_constructs_files_shape("    let x = 1;"),
        "ca ÂM: một dòng không nhắc `PipelineShape::Files` không được bị bắt"
    );
}

#[test]
fn pipeline_shape_files_is_constructed_at_exactly_one_product_call_site() {
    let files = all_rust_sources();

    let mut construction_sites: Vec<String> = Vec::new();
    for (rel, text) in &files {
        let product_only = boundary_scan::without_test_modules(text);
        for (line, code) in code_lines(&product_only) {
            if line_constructs_files_shape(&code) {
                construction_sites.push(format!("{rel}:{line}  {code}"));
            }
        }
    }

    assert_eq!(
        construction_sites.len(),
        1,
        "`PipelineShape::Files(` phải được XÂY ở đúng MỘT chỗ gọi sản phẩm \
         (`core::segment::import::import_files`) — tìm thấy {}:\n{}\n\n\
         N tệp là MỘT lượt nhập đang chờ (§Always spec 6.6b: \"N files are one pending \
         import\") — một chỗ xây thứ hai mở đường cho một seam khác tự dựng hình dạng này mà \
         không đi qua guard rỗng/đuôi tệp/kích thước của `import_files`.",
        construction_sites.len(),
        construction_sites.join("\n")
    );
    assert!(
        construction_sites[0].starts_with("core/segment/import.rs"),
        "chỗ xây DUY NHẤT phải sống ở `core/segment/import.rs` (`import_files`) — tìm thấy ở \
         {}",
        construction_sites[0]
    );
}

// ═════════════════════════════════════════════════════════════════════════════════
// Mệnh đề 2 — `core/segment/**` mang 0 dòng gõ từ vựng store/scope (AD-1)
// ═════════════════════════════════════════════════════════════════════════════════

const STORE_SCOPE_TOKENS: [&str; 6] =
    ["rusqlite", "core::store", "core::scope", "ScopeResolver", "StoreKind", "scope_kinds!"];

/// `haystack` mang `needle` như một TỪ trọn vẹn, hoặc như một chuỗi con khi `needle` không
/// phải một định danh (`::` chẳng hạn) — cùng khuôn `docx_boundary.rs::contains_word`/
/// `line_names_any_forbidden_token`.
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

fn line_names_a_store_or_scope_token(code: &str) -> bool {
    STORE_SCOPE_TOKENS.iter().any(|needle| {
        if needle.chars().all(|c| c.is_ascii_alphanumeric() || c == '_' || c == ':' || c == '!') {
            contains_word(code, needle)
        } else {
            code.contains(needle)
        }
    })
}

#[test]
fn the_store_scope_token_predicate_would_actually_flag_a_seeded_violation_and_ignore_clean_code() {
    assert!(
        line_names_a_store_or_scope_token("    let conn = rusqlite::Connection::open(\":memory:\")?;"),
        "ca DƯƠNG: `rusqlite` phải bị vị từ bắt"
    );
    assert!(
        line_names_a_store_or_scope_token("    use crate::core::store::Store;"),
        "ca DƯƠNG: `core::store` phải bị vị từ bắt"
    );
    assert!(
        line_names_a_store_or_scope_token("    let scope = ScopeResolver::global_only();"),
        "ca DƯƠNG: `ScopeResolver` phải bị vị từ bắt"
    );
    assert!(
        !line_names_a_store_or_scope_token("    let scope_of_the_word = \"khong lien quan\";"),
        "ca ÂM: một định danh chỉ TÌNH CỜ chứa chữ \"scope\" (không phải một token cấm nguyên \
         vẹn) không được bị bắt oan"
    );
    assert!(
        !line_names_a_store_or_scope_token("    let units = flow.units;"),
        "ca ÂM: một dòng bình thường của `core/segment/pipeline.rs` không được bị bắt"
    );
}

#[test]
fn core_segment_carries_zero_lines_naming_store_or_scope_vocabulary() {
    let files = all_rust_sources();
    let mut offenders: Vec<String> = Vec::new();
    for (rel, text) in &files {
        if !is_inside(rel, SEGMENT_DIR) {
            continue;
        }
        let product_only = boundary_scan::without_test_modules(text);
        for (line, code) in code_lines(&product_only) {
            if line_names_a_store_or_scope_token(&code) {
                offenders.push(format!("{rel}:{line}  {code}"));
            }
        }
    }
    assert!(
        offenders.is_empty(),
        "core/segment/** phải thuần (AD-1: không I/O, không `Store`/`ScopeResolver`) — mặt \
         MỚI của story 6.6b (`import_files`/`split_chapters_step_files`) là chỗ dễ nhất để \
         một lượt \"tiện tay\" kéo store/scope vào một module lẽ ra phải thuần. Tìm thấy:\n{}",
        offenders.join("\n")
    );
}

#[test]
fn a_forbidden_token_seeded_only_inside_a_cfg_test_block_is_not_counted_against_the_product_code() {
    let seeded =
        "fn read() {}\n#[cfg(test)]\nmod tests {\n    fn x() { let _ = rusqlite::params![]; }\n}\n";
    let offenders: Vec<String> = code_lines(&boundary_scan::without_test_modules(seeded))
        .filter(|(_, code)| line_names_a_store_or_scope_token(code))
        .map(|(_, code)| code)
        .collect();
    assert!(
        offenders.is_empty(),
        "một khối `#[cfg(test)]` gieo `rusqlite` KHÔNG được làm mệnh đề 2 đỏ oan — mã SẢN \
         PHẨM là thứ AD-1 ràng buộc, không phải mã test cùng tệp. Tìm thấy: {offenders:?}"
    );
    let product = "fn read() { let _ = rusqlite::params![]; }\n";
    assert_eq!(
        code_lines(&boundary_scan::without_test_modules(product))
            .filter(|(_, code)| line_names_a_store_or_scope_token(code))
            .count(),
        1,
        "cùng token đó ở mã SẢN PHẨM phải bị bắt — phép cắt không được nuốt luôn thứ cần canh"
    );
}

// ═════════════════════════════════════════════════════════════════════════════════
// Mệnh đề 3 — một đơn vị `Files` KHÔNG BAO GIỜ mang số của đơn vị khác
// ═════════════════════════════════════════════════════════════════════════════════

fn literal_rule(id: i64, pattern: &str) -> CleanupRule {
    CleanupRule {
        tier: CleanupRuleTier::Global,
        id,
        pattern: pattern.to_owned(),
        kind: CleanupRuleKind::Literal,
        enabled: true,
    }
}

/// **Đối chứng đỏ ② của §Verification spec 6.6b** — làm `split_chapters_step` chỉ xử lý đơn
/// vị ĐẦU TIÊN cho hình dạng `Files` (đột biến tay, không chạy được vì hàm sản phẩm không
/// nhận tham số kiểu đó) sẽ làm CHÍNH ca dưới đây đỏ ở phép so `per_rule_counts`/
/// `joined_line_count`, không chỉ đỏ ở SỐ Chương — nếu chỉ đơn vị 0 được xử, Chương 1 (từ đơn
/// vị 1) hoặc sẽ THIẾU hẳn khỏi kết quả HOẶC (nếu một bản vá sai khác giữ nguyên số Chương
/// nhưng LẶP LẠI số của đơn vị 0) sẽ mang ĐÚNG số của đơn vị 0 — cả hai đều làm assert dưới
/// đây đỏ.
#[test]
fn a_files_units_cleanup_and_joined_line_numbers_never_come_from_another_unit() {
    // Đơn vị 1 (Chương 1): CHỈ đơn vị này mang "ZZZ" (luật làm sạch khớp đúng MỘT lần), và
    // hai dòng KHÔNG có dấu kết câu ngăn giữa — nối dòng thật, `joined_lines == 1`.
    let unit1 = ChapterInput::AlreadyText("ZZZ mot\nhai ba.".to_owned());
    // Đơn vị 2 (Chương 2): KHÔNG mang "ZZZ" — 0 lượt khớp; một dòng trống NGĂN hai dòng con
    // (không bao giờ nối qua dòng trống — `normalize.rs::a_blank_line_is_never_joined_across`),
    // `joined_lines == 0` — một số THẬT, KHÁC số của đơn vị 1, để một lượt hoán đổi lộ ra ngay.
    let unit2 = ChapterInput::AlreadyText("khong lien quan\n\nva dong thu hai".to_owned());

    let rules = vec![literal_rule(1, "ZZZ")];
    let input = PipelineInput::default_shaped(PipelineShape::Files(vec![unit1, unit2]), "en")
        .with_cleanup_rules(rules);
    let outcome = run_import(input).expect("hai don vi hop le phai chay qua duoc chuoi bay buoc");

    assert_eq!(outcome.chapters.len(), 2, "N = 2 tep phai cho ra 2 Chuong khi khong co mau phan tach");

    let ch1 = &outcome.chapters[0];
    let ch2 = &outcome.chapters[1];

    let ch1_matches: usize =
        ch1.cleanup_report.as_ref().map(|r| r.per_rule_counts.values().sum()).unwrap_or(0);
    let ch2_matches: usize =
        ch2.cleanup_report.as_ref().map(|r| r.per_rule_counts.values().sum()).unwrap_or(0);
    assert_eq!(
        ch1_matches, 1,
        "Chuong 1 phai mang DUNG bao cao lam sach cua don vi 1 (1 lan khop \"ZZZ\") -- {ch1:?}"
    );
    assert_eq!(
        ch2_matches, 0,
        "Chuong 2 khong duoc mang bao cao lam sach cua don vi 1 -- don vi 2 khong he co \"ZZZ\", \
         mot lan khop o day nghia la so cua don vi 1 da bi ro ri sang -- {ch2:?}"
    );

    assert_eq!(
        ch1.joined_line_count,
        Some(1),
        "Chuong 1 (k == 1, khong bi tach them) phai giu SO THAT cua CHINH don vi 1 -- {ch1:?}"
    );
    assert_eq!(
        ch2.joined_line_count,
        Some(0),
        "Chuong 2 phai mang so noi dong THAT cua CHINH don vi 2 (0 -- dong trong ngan hai \
         dong con, khong bao gio noi qua), khong phai so cua don vi 1 -- {ch2:?}"
    );
}
