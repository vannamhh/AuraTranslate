//! Ranh giới cây nguồn của Story 6.4 — FR124/FR125, thân THẬT của bước 4 chuỗi AD-39.
//!
//! ─────────────────────────────────────────────────────────────────────────────
//! BA MỆNH ĐỀ, đúng khuôn `segment_pipeline_boundary.rs`/`segment_encoding_boundary.rs`
//! ─────────────────────────────────────────────────────────────────────────────
//! 1. **`core/segment/pipeline.rs` THẬT SỰ gọi `normalize::`** — không có ca này, cổng khác
//!    xanh y hệt trên một cây mà nhánh `Step::NormalizeParagraphsAndWhitespace` đã bị viết
//!    lại nội tuyến hoặc mất hẳn lời gọi (Task list spec 6.4: "GỌI, đừng viết lại nội
//!    tuyến"). Khuôn `segment_boundary.rs::the_pipeline_module_actually_calls_the_splitter`.
//! 2. **`normalize.rs` 0 dòng mang ký tự của bảng kết câu (`ZH_TERMINATORS`)** — module này
//!    không được dựng một bảng thứ hai; chủ vẫn là `split.rs` (`。！？；`) và `regroup.rs`
//!    (chuỗi nối). Đếm CHÍNH XÁC, không chỉ kiểm thành viên (khuôn
//!    `segment_encoding_boundary.rs::chardetng_is_named_in_exactly_one_product_file...`).
//! 3. **Chỗ gọi sản phẩm của `normalize::normalize`/`normalize::normalize_window` đúng SỐ
//!    ĐÃ BIẾT** — 🔵 SỬA (vá vòng rà 1) từ BA lên BỐN: một trong `pipeline.rs` (bước 4, gọi
//!    `normalize::normalize`), ba trong `encoding.rs` (dải ứng viên — nhánh `if
//!    window_truncated` gọi `normalize_window`, nhánh `else` gọi `normalize` thẳng — cộng
//!    `normalized_self_declared`, đường TỰ KHAI, gọi `normalize_window` một lần thứ ba). Một
//!    chỗ gọi THỨ NĂM là một đường ghi/dựng MỚI không ai ký.
//!
//! Sàn quần thể + kiểm chứng dương (ca dương + ca âm cho MỖI vị từ) là bắt buộc, khuôn
//! `segment_pipeline_boundary.rs`/`segment_encoding_boundary.rs`.

use std::fs;
use std::path::{Path, PathBuf};

/// Số tệp `.rs` tối thiểu dưới `src-tauri/src/**` — cùng lý lẽ
/// `segment_pipeline_boundary.rs::SRC_RS_FLOOR`. Story 6.4 thêm `core/segment/normalize.rs`,
/// nên số thật chỉ TĂNG — sàn cũ (50) vẫn đúng, không hạ.
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

/// Dòng KHÔNG phải chú thích — cùng khuôn `segment_encoding_boundary.rs::code_lines`, trừ
/// vế loại dòng trắng (không cần ở đây, không phép kiểm nào dưới đây nhạy với nó).
///
/// 🔵 **SỬA (vá vòng rà 1, mục 5) — thêm ba tiền tố khối `/* */`.** Bản trước chỉ loại `//`,
/// đúng khuôn `segment_encoding_boundary.rs`, nhưng module này cần một bảo đảm CHẶT hơn
/// (Mệnh đề 2 đếm ký tự bảng kết câu) — một dòng tiếp nối của khối `/* … */` (`* …`) hay
/// dòng đóng khối (`*/`) không phải MÃ, cùng luật `segment_boundary.rs::is_comment` đã dùng.
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
/// TRƯỚC dòng đó (không bao gồm nó). Neo THEO DÒNG, không phải `str::find` trên toàn văn
/// bản: một chú thích phía TRÊN nhắc lại chuỗi `"#[cfg(test)]"` (ví dụ để giải thích quy ước
/// này) sẽ làm `str::find` cắt SỚM, và phần "sản phẩm" bị cắt oan mất một phần thân thật —
/// cổng khi đó MÙ đúng mệnh đề nó tuyên bố canh (vá vòng rà 1, mục 5).
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
// Mệnh đề 1 — pipeline.rs THẬT SỰ gọi normalize::
// ═════════════════════════════════════════════════════════════════════════════════

/// `code` gọi một hàm của module `normalize` — vị từ THUẦN, dùng bởi CẢ cổng thật lẫn đối
/// chứng dương.
fn line_calls_normalize_module(code: &str) -> bool {
    code.contains("normalize::normalize")
}

#[test]
fn the_pipeline_module_actually_calls_the_normalize_module() {
    let text = fs::read_to_string(src_root().join("core/segment/pipeline.rs"))
        .expect("đọc core/segment/pipeline.rs");
    let has_call = code_lines(&text).any(|(_, code)| line_calls_normalize_module(code));
    assert!(
        has_call,
        "`core/segment/pipeline.rs` không gọi `normalize::normalize`/`normalize::normalize_window` \
         (ngoài chú thích) — bước 4 của chuỗi AD-39 (\"chuẩn hoá đoạn & khoảng trắng\") phải \
         GỌI thân thật, không viết lại nội tuyến hay để trống."
    );
}

/// Đối chứng dương: [`line_calls_normalize_module`] NỔ được trên một dòng vi phạm dựng tay,
/// và KHÔNG nổ oan trên một dòng bình thường không nhắc tới nó.
#[test]
fn the_normalize_call_check_would_actually_flag_a_seeded_violation_and_ignore_clean_code() {
    assert!(
        line_calls_normalize_module("    let n = normalize::normalize(&text, &source_lang);"),
        "ca DƯƠNG: một dòng gọi `normalize::normalize` phải bị vị từ bắt"
    );
    assert!(
        line_calls_normalize_module(
            "                            Unit::Decoded(normalize::normalize(&text, &source_lang).text)"
        ),
        "ca DƯƠNG: dòng gọi thật trong nhánh match cũng phải bị bắt"
    );
    assert!(
        !line_calls_normalize_module("    let s = split_source_text(&text, &source_lang);"),
        "ca ÂM: một dòng KHÔNG gọi `normalize::` không được bị bắt oan"
    );
}

// ═════════════════════════════════════════════════════════════════════════════════
// Mệnh đề 2 — normalize.rs 0 dòng mang ký tự bảng kết câu (chủ vẫn là split.rs)
// ═════════════════════════════════════════════════════════════════════════════════

/// Bốn dấu kết câu tiếng Trung (`split.rs::ZH_TERMINATORS`) — chép LẠI Ở ĐÂY chỉ để SO,
/// không để dùng làm luật: nếu `normalize.rs` chứa bất kỳ ký tự nào trong bốn ký tự này,
/// nó đang tự mang một bản sao của bảng, đúng thứ §Always spec 6.4 cấm ("bảng dấu kết câu
/// không được sao chép").
const ZH_TERMINATOR_CHARS: [char; 4] = ['。', '！', '？', '；'];

/// `code` có mang ký tự nào của bảng kết câu tiếng Trung hay không — vị từ THUẦN.
fn line_names_a_terminator_char(code: &str) -> bool {
    code.chars().any(|c| ZH_TERMINATOR_CHARS.contains(&c))
}

#[test]
fn normalize_rs_carries_zero_lines_naming_a_sentence_terminator_character() {
    let text =
        fs::read_to_string(src_root().join("core/segment/normalize.rs")).expect("đọc normalize.rs");

    // 🔴 Chỉ quét PHẦN SẢN PHẨM — cắt tại dòng `#[cfg(test)]` (neo THEO DÒNG, xem doc-comment
    // `text_before_first_cfg_test_line`). Mệnh đề này canh mã SẢN PHẨM không tự chép bảng
    // kết câu; các ca `#[cfg(test)]` bên dưới đó CẦN chữ Hán thật kèm `。！？` để làm fixture
    // (kiểm hành vi NỐI DÒNG), và đó không phải một bản sao bảng — đếm luôn phần đó sẽ cho
    // phép kiểm này đỏ oan trên chính bộ test hợp lệ của module.
    let product_only = text_before_first_cfg_test_line(&text);

    let offenders: Vec<String> = code_lines(product_only)
        .filter(|(_, code)| line_names_a_terminator_char(code))
        .map(|(line, code)| format!("normalize.rs:{line}  {code}"))
        .collect();

    assert!(
        offenders.is_empty(),
        "`normalize.rs` mang ký tự của bảng kết câu tiếng Trung — bảng đó có ĐÚNG MỘT chủ \
         (`split.rs::ZH_TERMINATORS`), `normalize.rs` chỉ được GỌI `split::line_ends_a_sentence`, \
         không được tự biết ký tự nào là dấu kết câu:\n{}",
        offenders.join("\n")
    );
}

/// Đối chứng dương: [`line_names_a_terminator_char`] NỔ được trên một dòng vi phạm dựng
/// tay, và KHÔNG nổ oan trên một dòng bình thường (kể cả một dòng tiếng Trung KHÔNG mang
/// dấu kết câu, ví dụ một cái tên biến hay một đoạn văn mẫu không kết câu).
#[test]
fn the_terminator_char_check_would_actually_flag_a_seeded_violation_and_ignore_clean_code() {
    assert!(
        line_names_a_terminator_char("    const OOPS: [char; 1] = ['。'];"),
        "ca DƯƠNG: một dòng chép lại dấu kết câu tiếng Trung phải bị vị từ bắt"
    );
    assert!(
        !line_names_a_terminator_char("    let joined = format!(\"{a}{joiner}{b}\");"),
        "ca ÂM: một dòng không mang ký tự bảng kết câu không được bị bắt oan"
    );
}

/// Đối chứng dương THỨ BA cho `text_before_first_cfg_test_line` — vá vòng rà 1, mục 5.
/// Đây CHÍNH LÀ ca mà lỗi cũ (`str::find` trên toàn văn bản) mù trước: một chú thích NHẮC
/// LẠI chuỗi `"#[cfg(test)]"` (để giải thích quy ước, không phải chính bản khai) đứng TRƯỚC
/// bản khai thật — hàm phải cắt tại BẢN KHAI THẬT (dòng khớp NGUYÊN VĂN sau trim), không
/// cắt sớm tại dòng chú thích.
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

/// Ca ÂM của cùng hàm — không có dòng `#[cfg(test)]` nào ⇒ trả NGUYÊN VĂN toàn bộ input,
/// không cắt oan.
#[test]
fn text_before_first_cfg_test_line_returns_the_whole_text_when_there_is_no_such_line() {
    let text = "fn a() {}\nfn b() {}\n";
    assert_eq!(text_before_first_cfg_test_line(text), text);
}

/// Đối chứng dương THỨ HAI, bổ khuyết cho mệnh đề 2 — `normalize.rs` phải THẬT SỰ *gọi*
/// hai chủ bảng (`split::line_ends_a_sentence`, `regroup::source_joiner`), không chỉ đơn
/// thuần "không tự chép bảng" — 0 lời gọi CŨNG cho mệnh đề trên xanh (một module rỗng cũng
/// 0 dòng mang ký tự bảng), nhưng khi đó module chẳng làm được việc gì.
#[test]
fn normalize_rs_actually_delegates_to_both_table_owners() {
    let text =
        fs::read_to_string(src_root().join("core/segment/normalize.rs")).expect("đọc normalize.rs");
    let calls_split = code_lines(&text).any(|(_, code)| code.contains("line_ends_a_sentence("));
    let calls_regroup = code_lines(&text).any(|(_, code)| code.contains("source_joiner("));
    assert!(calls_split, "normalize.rs phải GỌI split::line_ends_a_sentence, không tự đoán");
    assert!(calls_regroup, "normalize.rs phải GỌI regroup::source_joiner, không tự đoán");
}

// ═════════════════════════════════════════════════════════════════════════════════
// Mệnh đề 3 — chỗ gọi sản phẩm của normalize::normalize*/ đúng số đã biết
// ═════════════════════════════════════════════════════════════════════════════════

/// `code` gọi `normalize::normalize(` hoặc `normalize::normalize_window(` — vị từ THUẦN
/// dùng để ĐẾM (khác vị từ của Mệnh đề 1, vốn chỉ cần biết CÓ hay KHÔNG).
fn line_calls_a_normalize_function(code: &str) -> bool {
    code.contains("normalize::normalize(") || code.contains("normalize::normalize_window(")
}

#[test]
fn the_normalize_functions_have_exactly_four_named_product_call_sites() {
    let files = all_rust_sources();

    let mut sites: Vec<String> = Vec::new();
    for (rel, text) in &files {
        if rel == "core/segment/normalize.rs" {
            // Định nghĩa + đối chứng nội bộ (`#[cfg(test)]`) của chính module — không phải
            // một "chỗ gọi sản phẩm" theo nghĩa của phép kiểm này.
            continue;
        }
        for (line, code) in code_lines(text) {
            if line_calls_a_normalize_function(code) {
                sites.push(format!("{rel}:{line}  {code}"));
            }
        }
    }

    // 🔵 SỬA (vá vòng rà 1, mục 1) — từ BA lên BỐN: `encoding::normalized_self_declared`
    // (đường TỰ KHAI, 0 ứng viên) thêm MỘT chỗ gọi `normalize_window` mới. Số thật lúc dựng:
    // MỘT trong `pipeline.rs` (bước 4 gọi `normalize::normalize` trên `Unit::Decoded`), BA
    // trong `encoding.rs` — `normalized_candidate` rẽ nhánh `if window_truncated {
    // normalize_window } else { normalize }` (hai lời gọi tên khác nhau) cộng
    // `normalized_self_declared` (một lời gọi `normalize_window` thứ ba, cửa sổ cố định
    // `EVIDENCE_WINDOW_BYTES`).
    assert_eq!(
        sites.len(),
        4,
        "kỳ vọng ĐÚNG 4 chỗ gọi sản phẩm của `normalize::normalize`/`normalize::normalize_window` \
         (một ở `core/segment/pipeline.rs` bước 4, ba ở `core/segment/encoding.rs` — dải ứng \
         viên cộng nhánh tự khai), tìm thấy {}:\n{}",
        sites.len(),
        sites.join("\n")
    );
    for site in &sites {
        assert!(
            site.starts_with("core/segment/pipeline.rs") || site.starts_with("core/segment/encoding.rs"),
            "chỗ gọi phải ở `core/segment/pipeline.rs` hoặc `core/segment/encoding.rs` — tìm thấy: {site}"
        );
    }
}

/// Đối chứng dương: [`line_calls_a_normalize_function`] NỔ được trên cả hai dạng gọi
/// (`normalize`/`normalize_window`), và KHÔNG nổ oan trên một dòng không gọi hàm nào của
/// module (kể cả một dòng chỉ NHẮC TÊN module, ví dụ `use super::normalize;`).
#[test]
fn the_call_count_check_would_actually_flag_both_forms_and_ignore_a_bare_mention() {
    assert!(line_calls_a_normalize_function("    normalize::normalize(&text, lang);"));
    assert!(line_calls_a_normalize_function(
        "    normalize::normalize_window(&text, lang, 4096);"
    ));
    assert!(
        !line_calls_a_normalize_function("use super::normalize;"),
        "ca ÂM: chỉ NHẮC TÊN module (không gọi hàm nào) không được tính là một chỗ gọi"
    );
}
