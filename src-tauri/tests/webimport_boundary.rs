//! Ranh giới cây nguồn của Story 6.7 + 6.8 + 6.9 — AD-40 (hai nửa `Fetcher`/`Extractor`),
//! AD-15/AD-41 (điểm ra mạng thứ ba, đóng khung), AD-16 (0 chuỗi đánh dấu rời module).
//!
//! ─────────────────────────────────────────────────────────────────────────────
//! SÁU MỆNH ĐỀ, đúng khuôn `cleanup_boundary.rs`/`segment_pipeline_boundary.rs`
//! ─────────────────────────────────────────────────────────────────────────────
//! 1. **`core/webimport/fetcher.rs` mang 0 dòng gõ `dom_smoothie`/`Readability`** — `Fetcher`
//!    không bao giờ phân tích nội dung (AD-40).
//! 2. **`core/webimport/extractor.rs` mang 0 dòng gõ `reqwest`/`TcpStream`/`http://`/
//!    `https://`** — `Extractor` không bao giờ chạm mạng (AD-40), cùng khuôn
//!    `glossary_han_viet_suggestion_contract.rs:479-497`.
//! 3. **`reqwest` xuất hiện trong `src-tauri/src/**` CHỈ ở `core/webimport/` và `core/ai/`**
//!    — điểm ra mạng thứ ba (AD-15) không rò rỉ sang một module thứ tư.
//! 4. **0 dòng nào để `Article::content` (HTML) rời `core/webimport/extractor.rs`** — AD-16
//!    §Rule mục 1/2.
//! 5. 🔵 **THÊM Story 6.8 — `core/webimport/allowlist.rs` mang 0 dòng gõ
//!    `TcpStream`/`blocking::Client`** — `Allowlist::decide` là một phép QUYẾT ĐỊNH thuần
//!    (AD-41: luật "tầng 2 chỉ ảnh" sống trong KIỂU), không bao giờ tự mở một kết nối hay
//!    gửi một yêu cầu — đó là việc của MỘT MÌNH `Fetcher` (mệnh đề chỗ-duy-nhất-cưỡng-chế,
//!    §Always spec 6.8). 🔴 Không dùng chung `NETWORK_TOKENS` của mệnh đề 2: `allowlist.rs`
//!    HỢP LỆ gọi `reqwest::Url::parse` để bóc host từ một chuỗi URL (phân tích CHUỖI, không
//!    một byte nào đi qua dây) — cấm nguyên chữ `reqwest` ở đây sẽ bắt oan chính chỗ gọi hợp
//!    lệ đó. Xem tệp `spec-6-8-allowlist-mang-hai-tang-va-nhat-ky-domain.md` §Code Map:
//!    "cổng hôm nay mù với tệp mới" — mệnh đề này đóng đúng lỗ đó.
//! 6. 🔵 **THÊM Story 6.9 — `core/webimport/extractor.rs` mang ÍT NHẤT một dòng THÂN MÃ THẬT
//!    gọi `dom_query::Document::from`** — mô hình khối giữ/loại đòi một lượt phân tích LẠI
//!    HTML gốc (doc-comment đầu `extractor.rs` mục 2), không chỉ đọc `text_content` phẳng như
//!    trước. 🔴 Quét `code_lines` (đã lọc chú thích) — vòng rà 1 nhắc đúng token này CHỈ trong
//!    doc-comment rồi quên gọi thật, và bốn mệnh đề 1-5 ở trên không canh nổi lỗ đó (chúng
//!    canh RANH GIỚI, không canh "có mô hình khối THẬT").
//!
//! Sàn quần thể + kiểm chứng dương (ca dương + ca âm cho MỖI vị từ) là bắt buộc, khuôn
//! `cleanup_boundary.rs`.

use std::fs;
use std::path::{Path, PathBuf};

/// Số tệp `.rs` tối thiểu dưới `src-tauri/src/**` — cùng lý lẽ mọi `*_boundary.rs` khác.
/// Story 6.7 thêm `core/webimport/{mod,fetcher,extractor}.rs`, nên số thật chỉ TĂNG — sàn cũ
/// (50, `segment_pipeline_boundary.rs`/`cleanup_boundary.rs`) vẫn đúng, không hạ.
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
        // ⚠️ `symlink_metadata`, không `metadata` — một liên kết trỏ về thư mục cha làm đệ
        // quy không dừng (bài học các tệp `*_boundary.rs` khác).
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

/// Dòng KHÔNG phải chú thích — cùng khuôn `cleanup_boundary.rs::code_lines`.
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
///
/// 🔴 **THÊM 2026-09-07 — bản đầu của tệp này QUÊN hàm này, và đó là một khuyết tật thật,
/// không phải một chi tiết văn phong.** Bốn assert thật dưới đây quét `code_lines` TRẦN, nên
/// một khối `#[cfg(test)]` bên trong `fetcher.rs` nhắc `dom_smoothie` (một helper dựng HTML
/// mẫu là đủ), hoặc bên trong `extractor.rs` nhắc `reqwest`, làm cổng ĐỎ OAN — đúng lớp lỗi
/// mà `segment_pipeline_boundary.rs:182` từng mắc và `cleanup_boundary.rs:324` đã vá. Mã sản
/// phẩm là thứ AD-40 ràng buộc; mã test trong cùng tệp thì không.
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

/// `haystack` mang `needle` như một TỪ trọn vẹn — cùng khuôn `cleanup_boundary.rs::contains_word`.
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

/// `code` mang một trong `tokens` như một TỪ trọn vẹn, hoặc như một chuỗi con literal cho các
/// token có ký tự không phải định danh (`http://`/`https://`) — với các token đó,
/// [`contains_word`] không áp dụng được (dấu `:`/`/` không phải ký tự định danh nên MỌI xuất
/// hiện đã là "biên"); dùng `str::contains` trần cho đúng nhóm này.
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
// Mệnh đề 1 — Fetcher mang 0 dòng gõ dom_smoothie/Readability
// ═════════════════════════════════════════════════════════════════════════════════

const CONTENT_PARSING_TOKENS: [&str; 2] = ["dom_smoothie", "Readability"];

#[test]
fn fetcher_carries_zero_lines_naming_a_content_parsing_crate() {
    let text = text_of("core/webimport/fetcher.rs");
    let offenders: Vec<String> = code_lines(text_before_first_cfg_test_line(&text))
        .filter(|(_, code)| line_names_any_forbidden_token(code, &CONTENT_PARSING_TOKENS))
        .map(|(line, code)| format!("core/webimport/fetcher.rs:{line}  {code}"))
        .collect();
    assert!(
        offenders.is_empty(),
        "`Fetcher` không bao giờ phân tích nội dung (AD-40) — tìm thấy dòng gõ \
         dom_smoothie/Readability:\n{}",
        offenders.join("\n")
    );
}

#[test]
fn the_content_parsing_token_check_would_actually_flag_a_seeded_violation_and_ignore_clean_code() {
    assert!(
        line_names_any_forbidden_token(
            "    let r = dom_smoothie::Readability::new(html, Some(url), None);",
            &CONTENT_PARSING_TOKENS
        ),
        "ca DƯƠNG: một dòng gõ dom_smoothie/Readability phải bị vị từ bắt"
    );
    assert!(
        line_names_any_forbidden_token(
            "    let _ = reqwest::blocking::get(\"http://x.test\");",
            &ALLOWLIST_CONNECTION_TOKENS
        ),
        "ca DƯƠNG: `reqwest::blocking::get` là một hàm TỰ DO không đi qua `Client` — chính lỗ \
         mà bản đầu của mệnh đề 5 để hở (đo 2026-09-07, cổng xanh trên một lời gọi mạng thật)"
    );
    assert!(
        !line_names_any_forbidden_token(
            "    let client = reqwest::blocking::Client::builder();",
            &CONTENT_PARSING_TOKENS
        ),
        "ca ÂM: một dòng KHÔNG gõ dom_smoothie/Readability không được bị bắt oan"
    );
}

// ═════════════════════════════════════════════════════════════════════════════════
// Mệnh đề 2 — Extractor mang 0 dòng gõ reqwest/TcpStream/http://https://
// ═════════════════════════════════════════════════════════════════════════════════

const NETWORK_TOKENS: [&str; 4] = ["reqwest", "TcpStream", "http://", "https://"];

#[test]
fn extractor_carries_zero_lines_naming_a_network_client_or_a_literal_url_scheme() {
    let text = text_of("core/webimport/extractor.rs");
    let offenders: Vec<String> = code_lines(text_before_first_cfg_test_line(&text))
        .filter(|(_, code)| line_names_any_forbidden_token(code, &NETWORK_TOKENS))
        .map(|(line, code)| format!("core/webimport/extractor.rs:{line}  {code}"))
        .collect();
    assert!(
        offenders.is_empty(),
        "`Extractor` không bao giờ chạm mạng (AD-40) — tìm thấy dòng gõ mạng:\n{}",
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
        line_names_any_forbidden_token("    let u = \"http://example.com\";", &NETWORK_TOKENS),
        "ca DƯƠNG: literal `http://` phải bị vị từ bắt"
    );
    assert!(
        !line_names_any_forbidden_token(
            "    let text = article.text_content.to_string();",
            &NETWORK_TOKENS
        ),
        "ca ÂM: một dòng không gõ mạng không được bị bắt oan"
    );
}

// ═════════════════════════════════════════════════════════════════════════════════
// Mệnh đề 3 — `reqwest` chỉ xuất hiện trong core/webimport/ và core/ai/
// ═════════════════════════════════════════════════════════════════════════════════

fn path_is_allowed_for_reqwest(rel: &str) -> bool {
    rel.starts_with("core/webimport/") || rel.starts_with("core/ai/")
}

#[test]
fn reqwest_is_named_only_inside_core_webimport_or_core_ai() {
    let files = all_rust_sources();
    let mut offenders: Vec<String> = Vec::new();
    let mut allowed_hits = 0usize;

    for (rel, text) in &files {
        for (line, code) in code_lines(text_before_first_cfg_test_line(text)) {
            if contains_word(code, "reqwest") {
                if path_is_allowed_for_reqwest(rel) {
                    allowed_hits += 1;
                } else {
                    offenders.push(format!("{rel}:{line}  {code}"));
                }
            }
        }
    }

    assert!(
        allowed_hits > 0,
        "0 dòng gõ `reqwest` trong toàn bộ `core/webimport/`+`core/ai/` — phép kiểm này \
         không kiểm được gì nếu không có ít nhất một chỗ hợp lệ để so sánh"
    );
    assert!(
        offenders.is_empty(),
        "`reqwest` (điểm ra mạng thứ ba, AD-15) chỉ được phép xuất hiện trong \
         `core/webimport/`/`core/ai/` — tìm thấy ở nơi khác:\n{}",
        offenders.join("\n")
    );
}

#[test]
fn the_reqwest_allowlist_predicate_accepts_only_the_two_named_directories() {
    assert!(path_is_allowed_for_reqwest("core/webimport/fetcher.rs"), "ca DƯƠNG: core/webimport/");
    assert!(path_is_allowed_for_reqwest("core/ai/client.rs"), "ca DƯƠNG: core/ai/");
    assert!(
        !path_is_allowed_for_reqwest("core/segment/pipeline.rs"),
        "ca ÂM: một module KHÁC không được cho qua"
    );
    assert!(
        !path_is_allowed_for_reqwest("commands/project.rs"),
        "ca ÂM: `commands::project` gọi XUỐNG `webimport::fetch`, không tự gõ `reqwest`"
    );
}

// ═════════════════════════════════════════════════════════════════════════════════
// Mệnh đề 4 — 0 dòng nào để Article::content (HTML) rời core/webimport/extractor.rs
// ═════════════════════════════════════════════════════════════════════════════════

/// `code` đọc trường `.content` của một `Article` — vị từ THUẦN, hẹp có chủ ý (tệp này nhỏ
/// và chỉ có ĐÚNG một kiểu tên `Article` sống trong phạm vi), không phải một trình phân tích
/// cú pháp Rust đầy đủ.
fn line_reads_article_content_field(code: &str) -> bool {
    code.contains("article.content") || code.contains(".content")
}

#[test]
fn extractor_carries_zero_lines_reading_the_article_content_field() {
    let text = text_of("core/webimport/extractor.rs");
    let offenders: Vec<String> = code_lines(text_before_first_cfg_test_line(&text))
        .filter(|(_, code)| line_reads_article_content_field(code))
        .map(|(line, code)| format!("core/webimport/extractor.rs:{line}  {code}"))
        .collect();
    assert!(
        offenders.is_empty(),
        "`Article::content` (HTML) không bao giờ được rời `core/webimport/extractor.rs` \
         (AD-16 §Rule mục 1/2) — tìm thấy dòng đọc trường đó:\n{}",
        offenders.join("\n")
    );
}

#[test]
fn the_article_content_field_check_would_actually_flag_a_seeded_violation_and_ignore_clean_code() {
    assert!(
        line_reads_article_content_field("    let html = article.content.to_string();"),
        "ca DƯƠNG: đọc `article.content` phải bị vị từ bắt"
    );
    assert!(
        !line_reads_article_content_field("    let text = article.text_content.to_string();"),
        "ca ÂM: đọc `article.text_content` (văn bản thuần, không phải HTML) không được bị bắt oan"
    );
}

// ═════════════════════════════════════════════════════════════════════════════════
// Mệnh đề 5 (Story 6.8) — allowlist.rs mang 0 dòng gõ TcpStream/blocking::Client
// ═════════════════════════════════════════════════════════════════════════════════

/// 🔵 **SỬA 2026-09-07 (vòng rà Story 6.8) — `"blocking::Client"` một mình KHÔNG đủ, đo được.**
/// Bản đầu liệt `["TcpStream", "blocking::Client"]`. Gieo `reqwest::blocking::get("http://x.test")`
/// — một hàm TỰ DO, không đi qua một `Client` nào — vào `allowlist.rs` rồi chạy cổng: **14 ca
/// xanh, 0 đỏ**. Cổng mù đúng thứ nó dựng ra để cấm. ⇒ cắt tại `"blocking::"`, thứ phủ CẢ
/// `reqwest::blocking::Client`, `reqwest::blocking::get`, lẫn dạng `use reqwest::blocking;`
/// rồi gõ `blocking::get(...)`. 🔴 Vẫn KHÔNG cấm nguyên chữ `reqwest`: `reqwest::Url::parse`
/// là chỗ gọi HỢP LỆ duy nhất còn lại (phân tích CHUỖI, 0 byte qua dây) — xem ca ÂM bên dưới.
const ALLOWLIST_CONNECTION_TOKENS: [&str; 2] = ["TcpStream", "blocking::"];

#[test]
fn allowlist_carries_zero_lines_naming_a_socket_or_an_http_client_type() {
    let text = text_of("core/webimport/allowlist.rs");
    let offenders: Vec<String> = code_lines(text_before_first_cfg_test_line(&text))
        .filter(|(_, code)| line_names_any_forbidden_token(code, &ALLOWLIST_CONNECTION_TOKENS))
        .map(|(line, code)| format!("core/webimport/allowlist.rs:{line}  {code}"))
        .collect();
    assert!(
        offenders.is_empty(),
        "`Allowlist::decide` là một phép quyết định THUẦN (AD-41) — nó không bao giờ tự mở \
         kết nối hay gửi yêu cầu, việc đó là của MỘT MÌNH `Fetcher`. Tìm thấy dòng gõ \
         TcpStream/blocking::Client:\n{}",
        offenders.join("\n")
    );
}

#[test]
fn the_allowlist_connection_token_check_would_actually_flag_a_seeded_violation_and_ignore_clean_code() {
    assert!(
        line_names_any_forbidden_token(
            "    let c = reqwest::blocking::Client::builder().build().unwrap();",
            &ALLOWLIST_CONNECTION_TOKENS
        ),
        "ca DƯƠNG: một dòng dựng `blocking::Client` phải bị vị từ bắt"
    );
    assert!(
        line_names_any_forbidden_token(
            "    let s = TcpStream::connect(\"127.0.0.1:0\").unwrap();",
            &ALLOWLIST_CONNECTION_TOKENS
        ),
        "ca DƯƠNG: một dòng gõ `TcpStream` phải bị vị từ bắt"
    );
    assert!(
        !line_names_any_forbidden_token(
            "    reqwest::Url::parse(url).ok().and_then(|u| u.host_str().map(str::to_owned))",
            &ALLOWLIST_CONNECTION_TOKENS
        ),
        "ca ÂM: phân giải host bằng `reqwest::Url::parse` (chuỗi, không kết nối) không được bị \
         bắt oan — đây chính là lý do mệnh đề này KHÔNG dùng chung `NETWORK_TOKENS` của mệnh \
         đề 2 (`reqwest` là một từ khoá quá rộng cho tệp này)"
    );
}

// ═════════════════════════════════════════════════════════════════════════════════
// Mệnh đề 6 (Story 6.9) — `extractor.rs` mang mô hình khối THẬT trong THÂN MÃ, không chỉ
// trong doc-comment
// ═════════════════════════════════════════════════════════════════════════════════
//
// 🔴 **Đo được, không phải một sở thích.** Vòng rà 1 của spec 6.9 dùng đúng token
// `dom_query`/`Vec<Block>` trong DOC-COMMENT đầu tệp để mô tả cơ chế, rồi lại quên gọi
// `dom_query::Document::from` trong THÂN thật — cả `webimport_boundary.rs::the_scanned_tree_is_large_enough_to_be_real`
// lẫn bốn mệnh đề 1-5 ở trên đều KHÔNG canh nổi lỗ đó (chúng canh "không có mạng"/"không đọc
// `.content`", không canh "có mô hình khối THẬT"). Gỡ sạch thân duyệt DOM thứ hai (giữ nguyên
// doc-comment mô tả nó) phải làm mệnh đề dưới đây ĐỎ — đối chứng đỏ ③ của §Verification spec
// 6.9.

/// `code` gọi `dom_query::Document::from(...)` — chữ ký thật của lượt phân tích HTML GỐC lần
/// thứ hai (xem doc-comment đầu tệp mục 2). Vị từ THUẦN, hẹp có chủ ý (đúng khuôn
/// `line_reads_article_content_field` ở trên).
fn line_parses_the_original_document_a_second_time(code: &str) -> bool {
    code.contains("Document::from")
}

#[test]
fn extractor_carries_at_least_one_line_of_real_code_that_reparses_the_original_document() {
    let text = text_of("core/webimport/extractor.rs");
    let hits = code_lines(text_before_first_cfg_test_line(&text))
        .filter(|(_, code)| line_parses_the_original_document_a_second_time(code))
        .count();
    assert!(
        hits > 0,
        "`extractor.rs` phải có ít nhất một dòng THÂN MÃ (không phải doc-comment) gọi \
         `dom_query::Document::from` — mô hình khối (Story 6.9) đòi một lượt phân tích LẠI \
         HTML GỐC, không chỉ đọc `article.text_content` như trước. Vòng rà 1 nhắc đúng token \
         này CHỈ trong doc-comment, để lượt duyệt thân bị gỡ vẫn xanh; mệnh đề này quét \
         `code_lines` (đã lọc chú thích), không quét toàn văn."
    );
}

/// Đối chứng THẬT cho mệnh đề trên — chép khuôn
/// `a_forbidden_token_seeded_only_inside_a_cfg_test_block_is_not_counted_against_the_product_code`:
/// một chú thích NHẮC token đúng không được tính là "thân mã có thật".
#[test]
fn the_reparse_check_would_actually_flag_a_gutted_body_that_only_mentions_it_in_a_comment() {
    let gutted = "//! dùng `dom_query::Document::from` để duyệt lại HTML gốc\npub fn extract() {}\n";
    let hits = code_lines(text_before_first_cfg_test_line(gutted))
        .filter(|(_, code)| line_parses_the_original_document_a_second_time(code))
        .count();
    assert_eq!(
        hits, 0,
        "ca ÂM: một chú thích NHẮC LẠI `dom_query::Document::from` không được tính là thân mã \
         có thật — nếu tính, mệnh đề chính không đỏ được khi thân bị gỡ mà doc-comment còn"
    );

    let real = "pub fn extract() {\n    let doc = dom_query::Document::from(html);\n}\n";
    let hits_real = code_lines(text_before_first_cfg_test_line(real))
        .filter(|(_, code)| line_parses_the_original_document_a_second_time(code))
        .count();
    assert_eq!(hits_real, 1, "ca DƯƠNG: một dòng mã THẬT gọi `Document::from` phải bị đếm");
}

// ═════════════════════════════════════════════════════════════════════════════════
// Tự-kiểm phép cắt `#[cfg(test)]` — hai ca, bắt buộc đi kèm hàm (khuôn
// `cleanup_boundary.rs:369-388` / `segment_normalize_boundary.rs:231-247`)
// ═════════════════════════════════════════════════════════════════════════════════

/// Đối chứng dương — `text_before_first_cfg_test_line` không bị lừa bởi một chú thích NHẮC LẠI
/// chuỗi `"#[cfg(test)]"` đứng TRƯỚC bản khai thật.
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

/// Đối chứng THẬT cho lượt vá 2026-09-07: một khối `#[cfg(test)]` gieo vào `fetcher.rs` có
/// nhắc `dom_smoothie` KHÔNG được làm Mệnh đề 1 đỏ. Trước lượt vá, ca này đỏ.
#[test]
fn a_forbidden_token_seeded_only_inside_a_cfg_test_block_is_not_counted_against_the_product_code() {
    let seeded = "use reqwest;\nfn fetch() {}\n#[cfg(test)]\nmod tests {\n    use dom_smoothie::Readability;\n}\n";
    let offenders: Vec<&str> = code_lines(text_before_first_cfg_test_line(seeded))
        .filter(|(_, code)| line_names_any_forbidden_token(code, &CONTENT_PARSING_TOKENS))
        .map(|(_, code)| code)
        .collect();
    assert!(
        offenders.is_empty(),
        "mot khoi `#[cfg(test)]` nhac `dom_smoothie` KHONG duoc lam cong do oan — AD-40 rang \
         buoc ma SAN PHAM, khong rang buoc ma test cung tep. Tim thay: {offenders:?}"
    );
    // Ca ÂM đi kèm: cùng token đó ở mã SẢN PHẨM thì PHẢI bị bắt — nếu không, phép cắt ở trên
    // đã nuốt luôn thứ cần canh.
    let product = "use dom_smoothie::Readability;\nfn fetch() {}\n";
    assert_eq!(
        code_lines(text_before_first_cfg_test_line(product))
            .filter(|(_, code)| line_names_any_forbidden_token(code, &CONTENT_PARSING_TOKENS))
            .count(),
        1,
        "cung token do o ma SAN PHAM phai bi bat — phep cat khong duoc nuot thu can canh"
    );
}
