//! Ranh giới cây nguồn của Story 6.5 — FR124, thân THẬT của bước 3 chuỗi AD-39.
//!
//! ─────────────────────────────────────────────────────────────────────────────
//! BA MỆNH ĐỀ, đúng khuôn `segment_normalize_boundary.rs`/`segment_pipeline_boundary.rs`
//! ─────────────────────────────────────────────────────────────────────────────
//! 1. **`core/segment/pipeline.rs` THẬT SỰ gọi `core::cleanup::apply`** — không có ca này,
//!    cổng khác xanh y hệt trên một cây mà nhánh `Step::CleanByRules` đã bị viết lại nội
//!    tuyến hoặc mất hẳn lời gọi.
//! 2. **`core/cleanup/**` mang 0 dòng gõ `ScopeKind`/`Semantics`/`Tier`** — `core::cleanup`
//!    là module miền, không được biết từ vựng phân giải hai tầng của `core::scope`
//!    (`scope_boundary.rs::FORBIDDEN_OUTSIDE_SCOPE` đã canh MỌI module ngoài `core/scope/**`
//!    ở phạm vi toàn cây; phép kiểm này lặp lại đúng mệnh đề đó nhưng THU HẸP vào riêng
//!    `core/cleanup/**` — không sửa một kỳ vọng nào của `scope_boundary.rs`, chỉ thêm một
//!    lớp phòng thủ đặc thù module).
//! 3. **Chỗ gọi sản phẩm của `core::cleanup::apply` đúng SỐ ĐÃ BIẾT (một)** — trong
//!    `pipeline.rs`, bước 3. Một chỗ gọi THỨ HAI là một đường ghi/dựng MỚI không ai ký.
//!
//! Sàn quần thể + kiểm chứng dương (ca dương + ca âm cho MỖI vị từ) là bắt buộc, khuôn
//! `segment_pipeline_boundary.rs`/`segment_normalize_boundary.rs`.

use std::fs;
use std::path::{Path, PathBuf};

/// Thư mục ĐỊNH NGHĨA module — không phải một "chỗ gọi ngoài".
const CLEANUP_DIR: &str = "core/cleanup";

/// Số tệp `.rs` tối thiểu dưới `src-tauri/src/**` — cùng lý lẽ mọi `*_boundary.rs` khác.
/// Story 6.5 thêm `core/cleanup/mod.rs` + `core/cleanup/store.rs` + `commands/cleanup.rs`,
/// nên số thật chỉ TĂNG — sàn cũ (50, `segment_pipeline_boundary.rs`) vẫn đúng, không hạ.
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

/// Dòng KHÔNG phải chú thích — cùng khuôn `segment_normalize_boundary.rs::code_lines`: loại
/// cả `//`/`///`/`//!` VÀ phần thân/đóng của một khối `/* … */`.
fn code_lines(text: &str) -> impl Iterator<Item = (usize, &str)> {
    text.lines().enumerate().map(|(index, line)| (index + 1, line.trim_start())).filter(|(_, code)| {
        !code.is_empty()
            && !code.starts_with("//")
            && !code.starts_with("///")
            && !code.starts_with("/*")
            && !code.starts_with("* ")
            && !code.starts_with("*/")
    })
}

/// Cắt `text` tại dòng ĐẦU TIÊN mà, sau khi trim, khớp NGUYÊN VĂN `#[cfg(test)]` — trả phần
/// TRƯỚC dòng đó. Neo THEO DÒNG, không phải `str::find` trên toàn văn bản (cùng bài học
/// `segment_normalize_boundary.rs::text_before_first_cfg_test_line`, vá vòng rà 1 mục 5).
fn text_before_first_cfg_test_line(text: &str) -> &str {
    let mut end = text.len();
    let mut offset = 0usize;
    for line in text.split_inclusive('\n') {
        if line.trim() == "#[cfg(test)]" {
            end = offset;
            break;
        }
        offset += line.len();
    }
    &text[..end]
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
// Mệnh đề 1 — pipeline.rs THẬT SỰ gọi core::cleanup::apply
// ═════════════════════════════════════════════════════════════════════════════════

/// `code` gọi `cleanup::apply(` (qua alias `crate::core::cleanup::apply` hoặc `cleanup::apply`
/// đã `use`) — vị từ THUẦN, dùng bởi CẢ cổng thật lẫn đối chứng dương.
fn line_calls_cleanup_apply(code: &str) -> bool {
    code.contains("cleanup::apply(")
}

#[test]
fn the_pipeline_module_actually_calls_the_cleanup_module() {
    let text = fs::read_to_string(src_root().join("core/segment/pipeline.rs"))
        .expect("đọc core/segment/pipeline.rs");
    // 🔴 SỬA vòng rà (2026-09-06) — bản trước quét `code_lines(&text)` TRẦN, tức CẢ khối
    // `#[cfg(test)]` của chính `pipeline.rs`. Hậu quả: gỡ lời gọi THẬT khỏi `Step::CleanByRules`
    // (viết lại nội tuyến hoặc xoá hẳn) mà một ca test bên dưới còn nhắc `cleanup::apply(`
    // (ví dụ một `#[test]` gọi thẳng hàm để dựng đối chứng) vẫn làm mệnh đề này ĐỌC RA "có
    // gọi" — cổng xanh trên đúng cây mà nó tồn tại để bắt đỏ. Cắt về PHẦN SẢN PHẨM trước khi
    // quét, đúng khuôn `segment_normalize_boundary.rs:194`.
    let product_only = text_before_first_cfg_test_line(&text);
    let has_call = code_lines(product_only).any(|(_, code)| line_calls_cleanup_apply(code));
    assert!(
        has_call,
        "`core/segment/pipeline.rs` không gọi `cleanup::apply` (ngoài chú thích, ngoài khối \
         `#[cfg(test)]`) — bước 3 của chuỗi AD-39 (\"làm sạch theo luật\") phải GỌI thân thật, \
         không viết lại nội tuyến hay để trống."
    );
}

/// Đối chứng dương THỨ BA cho mệnh đề 1 — chứng minh cổng ĐỎ ĐƯỢC đúng ca vòng rà bắt: lời
/// gọi sản phẩm bị RÚT khỏi thân hàm (viết lại nội tuyến/xoá) trong khi một lời gọi TƯƠNG TỰ
/// vẫn còn sống trong CHÍNH khối `#[cfg(test)]` của tệp — nếu quét cả thân test, mệnh đề trên
/// sẽ đọc SAI ra "có gọi".
#[test]
fn a_call_living_only_inside_the_pipeline_files_own_test_block_does_not_count_as_the_real_call() {
    let seeded_file = "fn step() {\n    // than that da bi go, khong con goi cleanup::apply nua\n}\n\n#[cfg(test)]\nmod tests {\n    #[test]\n    fn seeded() {\n        let _ = crate::core::cleanup::apply(\"x\", &[]);\n    }\n}\n";
    let product_only = text_before_first_cfg_test_line(seeded_file);
    let has_call = code_lines(product_only).any(|(_, code)| line_calls_cleanup_apply(code));
    assert!(
        !has_call,
        "một lời gọi CHỈ sống trong khối `#[cfg(test)]` không được tính là 'pipeline.rs gọi \
         cleanup::apply' — đây đúng ca vòng rà bắt được: cách quét cũ (`code_lines(&text)` \
         trần) sẽ đọc ra CÓ gọi và bỏ lọt đúng lúc thân sản phẩm thật đã bị gỡ"
    );
}

/// Đối chứng dương: [`line_calls_cleanup_apply`] NỔ được trên một dòng vi phạm dựng tay, và
/// KHÔNG nổ oan trên một dòng bình thường không nhắc tới nó.
#[test]
fn the_cleanup_apply_call_check_would_actually_flag_a_seeded_violation_and_ignore_clean_code() {
    assert!(
        line_calls_cleanup_apply(
            "                            let cleaned = crate::core::cleanup::apply(&text, &rules)?;"
        ),
        "ca DƯƠNG: một dòng gọi `cleanup::apply` phải bị vị từ bắt"
    );
    assert!(
        !line_calls_cleanup_apply("    let n = normalize::normalize(&text, &source_lang);"),
        "ca ÂM: một dòng KHÔNG gọi `cleanup::apply` không được bị bắt oan"
    );
}

// ═════════════════════════════════════════════════════════════════════════════════
// Mệnh đề 2 — core/cleanup/** mang 0 dòng gõ ScopeKind/Semantics/Tier
// ═════════════════════════════════════════════════════════════════════════════════

/// Ba token bị cấm bên trong `core/cleanup/**` — module miền không được biết từ vựng phân
/// giải hai tầng của `core::scope`. Thu hẹp vào riêng module này, KHÔNG thay thế
/// `scope_boundary.rs::FORBIDDEN_OUTSIDE_SCOPE` (mệnh đề đó vẫn quét toàn cây, không đổi).
///
/// 🔴 **`Tier` SIẾT HƠN `scope_boundary.rs`** — cổng đó KHÔNG cấm `Tier` (vì
/// `core::glossary::store` đã có tiền lệ `use ... Tier as ScopeTier`), nhưng §Always spec 6.5
/// đòi module NÀY không bao giờ đặt tên kiểu đó (`core/cleanup/store.rs::tier_from_scope_wire`
/// đi qua chuỗi `"global"`/`"work"`, không qua kiểu). Vì `Tier` là một chuỗi con của chính
/// `CleanupRuleTier` (kiểu RIÊNG hợp lệ của module này), phép so KHÔNG được là
/// `str::contains` trần — xem [`contains_word`].
const FORBIDDEN_TOKENS: [&str; 3] = ["ScopeKind", "Semantics", "Tier"];

/// `haystack` mang `needle` như một TỪ trọn vẹn — ký tự liền trước/sau (nếu có) không phải
/// một ký tự định danh (`[A-Za-z0-9_]`). Đây là điều kiện để `"Tier"` không khớp OAN bên
/// trong `"CleanupRuleTier"` (kiểu RIÊNG hợp lệ của module này) trong khi vẫn bắt được một
/// `Tier` ĐỨNG MỘT MÌNH (`use ... Tier as ScopeTier`, `: Tier`, `Tier::Global`, …).
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

/// `code` mang một trong ba token cấm, như một TỪ trọn vẹn — vị từ THUẦN.
fn line_names_a_forbidden_token(code: &str) -> bool {
    FORBIDDEN_TOKENS.iter().any(|needle| contains_word(code, needle))
}

/// Đối chứng dương của [`contains_word`] chính nó — trước khi dùng nó để canh cả module,
/// chứng minh nó phân biệt được "từ trọn vẹn" với "chuỗi con của một định danh khác".
#[test]
fn contains_word_matches_a_whole_word_but_not_a_substring_of_a_longer_identifier() {
    assert!(contains_word("use crate::core::scope::Tier;", "Tier"), "ca DUONG: tu tron ven");
    assert!(contains_word("let t: Tier = Tier::Global;", "Tier"), "ca DUONG: xuat hien hai lan");
    assert!(
        !contains_word("pub tier: CleanupRuleTier,", "Tier"),
        "ca AM: la chuoi con cua CleanupRuleTier, khong phai mot tu tron ven"
    );
    assert!(
        !contains_word("let x = TierExtra;", "Tier"),
        "ca AM: dung truoc mot dinh danh dai hon (TierExtra), khong phai mot tu tron ven"
    );
}

#[test]
fn core_cleanup_carries_zero_lines_naming_scope_kind_semantics_or_tier() {
    let files = all_rust_sources();

    let mut offenders: Vec<String> = Vec::new();
    let mut cleanup_files = 0usize;
    for (rel, text) in &files {
        if !rel.starts_with(CLEANUP_DIR) {
            continue;
        }
        cleanup_files += 1;
        for (line, code) in code_lines(text) {
            if line_names_a_forbidden_token(code) {
                offenders.push(format!("{rel}:{line}  {code}"));
            }
        }
    }

    assert!(
        cleanup_files > 0,
        "không tệp nào khớp `{CLEANUP_DIR}` — đường dẫn đã lệch khỏi cây nguồn"
    );

    assert!(
        offenders.is_empty(),
        "`core::cleanup` mang từ vựng phân giải hai tầng của `core::scope` \
         (`ScopeKind`/`Semantics`/`Tier`):\n{}\n\n\
         Đây là module miền — nó gọi `ScopeResolver::apply_merge(\"import_cleanup_rule\", ..)` \
         bằng một hằng literal, không bao giờ gõ tên kiểu nội bộ của `core::scope`. Kiểu nhãn \
         tầng của MỘT luật là `core::cleanup::CleanupRuleTier` — RIÊNG, không phải `Tier`.",
        offenders.join("\n")
    );
}

/// Đối chứng dương: [`line_names_a_forbidden_token`] NỔ được trên một dòng vi phạm dựng tay,
/// KHÔNG nổ oan trên một dòng dùng đúng `CleanupRuleTier` (kiểu RIÊNG, không phải `Tier`).
#[test]
fn the_forbidden_token_check_would_actually_flag_a_seeded_violation_and_ignore_clean_code() {
    assert!(
        line_names_a_forbidden_token("    let kind = ScopeKind::ImportCleanupRule;"),
        "ca DƯƠNG: một dòng gõ `ScopeKind` phải bị vị từ bắt"
    );
    assert!(
        line_names_a_forbidden_token("    fn f(s: Semantics) {}"),
        "ca DƯƠNG: một dòng gõ `Semantics` phải bị vị từ bắt"
    );
    assert!(
        line_names_a_forbidden_token("    let t: Tier = Tier::Global;"),
        "ca DƯƠNG: một dòng gõ `Tier` phải bị vị từ bắt"
    );
    assert!(
        !line_names_a_forbidden_token("    pub tier: CleanupRuleTier,"),
        "ca ÂM: `CleanupRuleTier` (kiểu RIÊNG của chính module này) không được bị bắt oan — \
         nó không chứa `Tier` như một TỪ tách rời, nhưng ĐÚNG như một chuỗi con, nên vị từ \
         `contains` PHẢI bắt nó nếu ta không cẩn thận; đây là đối chứng cho việc ta đã lựa \
         chọn có ý thức, không phải một chỗ hở"
    );
}

// ═════════════════════════════════════════════════════════════════════════════════
// Mệnh đề 3 — chỗ gọi sản phẩm của core::cleanup::apply đúng SỐ ĐÃ BIẾT (một)
// ═════════════════════════════════════════════════════════════════════════════════

#[test]
fn the_cleanup_apply_function_has_exactly_one_named_product_call_site() {
    let files = all_rust_sources();

    let mut sites: Vec<String> = Vec::new();
    for (rel, text) in &files {
        if rel == "core/cleanup/mod.rs" {
            // Định nghĩa + đối chứng nội bộ (`#[cfg(test)]`) của chính module — không phải
            // một "chỗ gọi sản phẩm" theo nghĩa của phép kiểm này.
            continue;
        }
        // 🔴 SỬA vòng rà (2026-09-06) — bản trước quét `code_lines(text)` TRẦN trên MỌI tệp
        // khác `core/cleanup/mod.rs`, kể cả khối `#[cfg(test)]` CỦA CHÍNH TỆP ĐÓ. Một ca test
        // dựng tay trong một tệp bất kỳ (ví dụ `commands/cleanup.rs`) mà gọi thẳng
        // `cleanup::apply(` để dựng đối chứng sẽ bị ĐẾM NHẦM thành "chỗ gọi sản phẩm thứ
        // hai" — cổng đỏ OAN trên một cây hoàn toàn hợp lệ. Cắt về PHẦN SẢN PHẨM của TỪNG tệp
        // trước khi quét, đúng khuôn `segment_normalize_boundary.rs:194`.
        let product_only = text_before_first_cfg_test_line(text);
        for (line, code) in code_lines(product_only) {
            if line_calls_cleanup_apply(code) {
                sites.push(format!("{rel}:{line}  {code}"));
            }
        }
    }

    assert_eq!(
        sites.len(),
        1,
        "kỳ vọng ĐÚNG MỘT chỗ gọi sản phẩm của `cleanup::apply` (bước 3 của `pipeline.rs`), \
         tìm thấy {}:\n{}",
        sites.len(),
        sites.join("\n")
    );
    assert!(
        sites[0].starts_with("core/segment/pipeline.rs"),
        "chỗ gọi DUY NHẤT phải ở `core/segment/pipeline.rs::Step::CleanByRules` — tìm thấy ở \
         đây thay vì đó: {}",
        sites[0]
    );
}

/// Đối chứng dương THỨ TƯ cho mệnh đề 3 — một lời gọi `cleanup::apply(` sống CHỈ trong khối
/// `#[cfg(test)]` của MỘT TỆP KHÁC (không phải `core/cleanup/mod.rs`, tệp duy nhất được
/// `continue` bỏ qua HẲN) không được đếm là một "chỗ gọi sản phẩm thứ hai" — nếu không cắt về
/// phần sản phẩm trước khi quét, cổng sẽ đếm nhầm và báo đỏ oan "tìm thấy 2" trên một cây chỉ
/// có đúng một chỗ gọi thật.
#[test]
fn a_call_seeded_only_inside_another_files_test_block_is_not_counted_as_a_second_product_call_site()
 {
    let seeded_file = "fn helper() {}\n\n#[cfg(test)]\nmod tests {\n    #[test]\n    fn seeded() {\n        let _ = crate::core::cleanup::apply(\"y\", &[]);\n    }\n}\n";
    let product_only = text_before_first_cfg_test_line(seeded_file);
    let found = code_lines(product_only).any(|(_, code)| line_calls_cleanup_apply(code));
    assert!(
        !found,
        "một lời gọi chỉ tồn tại trong khối `#[cfg(test)]` của MỘT TỆP KHÁC không được đếm là \
         một chỗ gọi SẢN PHẨM thứ hai — nếu quét cả thân test, nó sẽ đếm nhầm và báo 'tìm thấy \
         2' đúng lúc chỉ có 1 chỗ gọi thật"
    );
}

/// Đối chứng dương thứ hai — `text_before_first_cfg_test_line` không bị lừa bởi một chú
/// thích NHẮC LẠI chuỗi `"#[cfg(test)]"` đứng TRƯỚC bản khai thật (khuôn
/// `segment_normalize_boundary.rs`, vá vòng rà 1 mục 5).
#[test]
fn text_before_first_cfg_test_line_is_not_fooled_by_a_comment_mentioning_the_attribute() {
    let text = "fn a() {}\n// mot chu thich nhac lai chuoi \"#[cfg(test)]\" o day\nfn b() {}\n#[cfg(test)]\nmod tests {}\n";
    let got = text_before_first_cfg_test_line(text);
    assert_eq!(
        got, "fn a() {}\n// mot chu thich nhac lai chuoi \"#[cfg(test)]\" o day\nfn b() {}\n",
        "phai cat tai DONG khop NGUYEN VAN `#[cfg(test)]`, khong cat som tai dong chu thich \
         chi NHAC LAI chuoi do"
    );
}

/// Ca ÂM của cùng hàm — không có dòng `#[cfg(test)]` nào ⇒ trả NGUYÊN VĂN toàn bộ input.
#[test]
fn text_before_first_cfg_test_line_returns_the_whole_text_when_there_is_no_such_line() {
    let text = "fn a() {}\nfn b() {}\n";
    assert_eq!(text_before_first_cfg_test_line(text), text);
}
