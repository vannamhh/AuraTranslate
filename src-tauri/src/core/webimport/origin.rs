//! Bóc xuất xứ tài liệu từ HTML — FR128/AD-43, Story 6.15. Hàm **thuần**, **0 lời gọi mạng**,
//! sống ở `Extractor` (không `Fetcher`) đúng ranh giới AD-40: bốn trường xuất xứ là một mô
//! hình dựng từ HTML ĐÃ TẢI, không phải một dữ kiện của lượt tải.
//!
//! ─────────────────────────────────────────────────────────────────────────────
//! 🔴 BA TRƯỜNG BÓC ĐƯỢC TỪ THÂN TRANG, MỘT TRƯỜNG KHÔNG BÓC
//! ─────────────────────────────────────────────────────────────────────────────
//! `author`/`site_name`/`published_at` đọc từ chính HTML — mỗi trường có một danh sách nguồn
//! tín hiệu CÓ THỨ TỰ (xem doc-comment [`extract_origin`]); nguồn trượt (thẻ vắng, JSON-LD sai
//! cú pháp) chỉ khiến hàm thử nguồn KẾ TIẾP, không bao giờ ném lỗi.
//!
//! `url` (URL bài gốc) KHÔNG bóc từ HTML — Ice chốt 2026-09-10: cột đó ghi đúng URL YÊU CẦU
//! (link người dùng đã dán), không phải chặng cuối sau chuyển hướng, không đọc
//! `<link rel="canonical">`. Giá trị đó đã có sẵn ở `Flow.labels` (`core::segment::pipeline`)
//! trước khi hàm này được gọi — [`extract_origin`] chỉ ECHO nó lại vào [`ChapterOrigin::url`]
//! (rỗng ⇒ `None`) để bốn trường xuất xứ có một nguồn sự thật DUY NHẤT (một struct), không hai
//! đường ghi rời nhau ở hai module khác nhau. `fetcher.rs` không đổi một dòng vì quyết định
//! này.
//!
//! ─────────────────────────────────────────────────────────────────────────────
//! KHÔNG SUY MỘT TRƯỜNG TỪ TRƯỜNG KHÁC
//! ─────────────────────────────────────────────────────────────────────────────
//! `og:site_name` vắng thì host của URL (`vnexpress.net`) là thứ duy nhất còn lại — nhưng nó
//! không phải TÊN tờ báo, và một giá trị suy ra trông giống hệt một giá trị bóc được. Hàm này
//! không bao giờ đọc host để điền `site_name`.

use dom_query::Document as HtmlDocument;

/// Kết quả một lượt bóc xuất xứ — bốn trường TỰ DO, `None` khi không tìm được.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ChapterOrigin {
    /// Tác giả bài gốc.
    pub author: Option<String>,
    /// Tên báo/website nguồn.
    pub site_name: Option<String>,
    /// URL bài gốc — URL YÊU CẦU, echo lại từ tham số `url` của [`extract_origin`], KHÔNG bóc
    /// từ HTML. Xem doc-comment đầu tệp.
    pub url: Option<String>,
    /// Ngày đăng, cắt còn `YYYY-MM-DD` khi nguồn là một dấu thời gian ISO; giữ nguyên văn khi
    /// nguồn là văn bản tự do (ví dụ thẻ `<time>` không mang `datetime`).
    pub published_at: Option<String>,
}

impl ChapterOrigin {
    /// `true` khi cả bốn trường đều `None` — dùng ở tầng gọi để quyết có đáng chở một
    /// `Some(ChapterOrigin)` hay không (KHÔNG dùng ở đây: hàm này luôn trả một giá trị, tầng
    /// gọi tự quyết định bọc `Option` thế nào).
    pub fn is_empty(&self) -> bool {
        self.author.is_none()
            && self.site_name.is_none()
            && self.url.is_none()
            && self.published_at.is_none()
    }
}

/// AI-7 — luật CẮT chung cho bốn cột `ChapterOrigin` (`origin_author` · `origin_site_name` ·
/// `origin_url` · `origin_published_at`), MỘT khai báo DUY NHẤT cho cả hai hình dạng
/// (`&str` ở [`chapter_origin_trim`], `Option<String>` ở [`chapter_origin_trim_or_none`]) —
/// mọi chỗ trong kho cắt một trong bốn cột này đi qua đây, không tự viết `str::trim()`.
///
/// **KHÔNG PHẢI lớp `White_Space` 25 mã của `GLOSSARY_ENTRY_DDL`** (`store/schema.rs:305-323`
/// · `core/cleanup/store.rs:140` · `core/segment/normalize.rs:88` ·
/// `glossary_contract.rs:287-325`, ghép với `str::trim()` có chủ ý theo `src-tauri/AGENTS.md`).
/// Đây là một lớp KHÁC cho một thực thể KHÁC — cũng 25 mã, nhưng đổi CHỖ: bỏ `U+0085` (NEL,
/// Rust coi là `White_Space`, JS thì không), thêm `U+FEFF` (BOM/zero-width no-break space, JS
/// coi là khoảng trắng khi `.trim()`, Rust thì không). Đừng "đồng bộ" hai lớp này.
///
/// **Đo 2026-09-15, mọi mã `0x0..=0x10FFFF`, cả hai máy JS** (V8 qua `node`, và vì ứng dụng
/// chạy WKWebView chứ không phải `node`, thêm JavaScriptCore qua `osascript -l JavaScript`) —
/// **hai máy JS cho ĐÚNG một tập**, nên nửa `vitest` và nửa ứng dụng đã đóng gói đo CÙNG một
/// luật:
/// ```text
/// Rust str::trim()  (25):  0009 000A 000B 000C 000D 0020 0085 00A0 1680 2000‥200A 2028 2029 202F 205F 3000
/// JS   .trim()      (25):  0009 000A 000B 000C 000D 0020      00A0 1680 2000‥200A 2028 2029 202F 205F 3000 FEFF
/// Difference: JS-only {FEFF} · Rust-only {0085}
/// ```
/// JSC đối chứng, nguyên văn: `85=keep FEFF=TRIM 9=TRIM 20=TRIM A0=TRIM 200B=keep 180E=keep 2028=TRIM`.
///
/// `U+200B` (zero-width space) không bị cắt bởi bên nào — nó là NỘI DUNG trên cả hai máy, một
/// luật khác đã sở hữu nó (`core/glossary/exchange.rs:503 ZERO_WIDTH_CHARS`); hàm này không
/// đụng tới nó.
///
/// **Hệ quả đã nhận, không thiết kế để tránh (D1, spec AI-7):** một ô CHỈ mang `U+0085` không
/// còn thành `NULL` nữa — cả hai nửa (Rust lẫn JS) đồng thuận GIỮ nó làm nội dung, nên không
/// còn một chỗ lệch NGẦM, nhưng ô đó render thành một hộp trống KHÔNG nhãn (đúng triệu chứng
/// Story 6.15 AC4, qua một ký tự khác). Tần suất trong thực tế CHƯA ĐO — ghi nợ ở
/// `deferred-work.md`, không đóng bởi spec này.
fn is_chapter_origin_trim_char(c: char) -> bool {
    (c.is_whitespace() && c != '\u{0085}') || c == '\u{FEFF}'
}

/// Hình dạng `&str` của luật cắt AI-7 — xem doc-comment [`is_chapter_origin_trim_char`].
pub(crate) fn chapter_origin_trim(value: &str) -> &str {
    value.trim_matches(is_chapter_origin_trim_char)
}

/// Hình dạng `Option<String>` của luật cắt AI-7 — cắt hai đầu, rỗng sau khi cắt coi như vắng
/// (`None`) — khuôn dùng CHUNG cho cả bốn cột `ChapterOrigin`. Xem doc-comment
/// [`is_chapter_origin_trim_char`].
pub(crate) fn chapter_origin_trim_or_none(value: &str) -> Option<String> {
    let trimmed = chapter_origin_trim(value);
    if trimmed.is_empty() { None } else { Some(trimmed.to_owned()) }
}

/// Đọc `content` của thẻ `<meta>` khớp CẢ HAI: tên thuộc tính (`"name"` hoặc `"property"`) VÀ
/// giá trị của nó — trả bản ghi ĐẦU TIÊN theo thứ tự tài liệu có `content` khác rỗng sau khi
/// cắt.
fn meta_content(document: &HtmlDocument, attr: &str, value: &str) -> Option<String> {
    let selector = format!("meta[{attr}='{value}']");
    for node in document.select(&selector).nodes() {
        if let Some(content) = node.attr("content") {
            if let Some(v) = chapter_origin_trim_or_none(&content) {
                return Some(v);
            }
        }
    }
    None
}

/// Mọi khối `<script type="application/ld+json">` phân giải ĐƯỢC thành JSON, theo thứ tự tài
/// liệu — một khối sai cú pháp bị BỎ QUA (không ném), một mảng top-level HOẶC một `"@graph"`
/// lồng bên trong được TRẢI PHẲNG thành từng đối tượng (JSON-LD cho phép cả hai hình dạng).
fn json_ld_objects(document: &HtmlDocument) -> Vec<serde_json::Value> {
    let mut out = Vec::new();
    for node in document.select("script[type='application/ld+json']").nodes() {
        let text = node.text();
        let Ok(value) = serde_json::from_str::<serde_json::Value>(text.trim()) else { continue };
        push_flattened(&mut out, value);
    }
    out
}

/// Trải phẳng một giá trị JSON-LD vào `out` — một mảng top-level, hoặc một `"@graph"` lồng bên
/// trong một object, đều trở thành nhiều đối tượng độc lập; mọi hình dạng khác (object trần,
/// chuỗi, số, …) được đẩy nguyên vẹn.
fn push_flattened(out: &mut Vec<serde_json::Value>, value: serde_json::Value) {
    match value {
        serde_json::Value::Array(items) => {
            for item in items {
                push_flattened(out, item);
            }
        }
        serde_json::Value::Object(ref map) => {
            if let Some(graph) = map.get("@graph").cloned() {
                push_flattened(out, graph);
            }
            out.push(value);
        }
        other => out.push(other),
    }
}

/// Đọc một trường CHUỖI từ một object JSON-LD — giá trị có thể là một chuỗi trần, một object
/// mang `"name"`, hoặc một mảng của MỘT TRONG HAI hình dạng đó (lấy phần tử ĐẦU hợp lệ).
fn json_ld_string_field(obj: &serde_json::Value, key: &str) -> Option<String> {
    let value = obj.get(key)?;
    json_ld_string_or_named(value)
}

fn json_ld_string_or_named(value: &serde_json::Value) -> Option<String> {
    match value {
        serde_json::Value::String(s) => chapter_origin_trim_or_none(s),
        serde_json::Value::Object(map) => {
            map.get("name").and_then(|n| n.as_str()).and_then(chapter_origin_trim_or_none)
        }
        serde_json::Value::Array(items) => items.iter().find_map(json_ld_string_or_named),
        _ => None,
    }
}

/// Dấu thời gian ISO (`2026-09-10T08:00:00Z`, `2026-09-10`, …) cắt còn `YYYY-MM-DD` — vị từ
/// đơn giản có chủ ý: mười ký tự đầu khớp hình dạng `\d{4}-\d{2}-\d{2}`. Không khớp hình dạng
/// đó (văn bản tự do, ví dụ *"3 giờ trước"*) ⇒ giữ NGUYÊN VĂN, đã cắt hai đầu — cột
/// `origin_published_at` là `TEXT` tự do (xem doc-comment `schema::CHAPTER_ORIGIN_DDL`), ép
/// một khuôn ngày lên nó là chặn người dùng ghi thứ họ đọc được trên trang.
fn truncate_iso_date(raw: &str) -> String {
    let bytes = raw.as_bytes();
    if bytes.len() >= 10
        && bytes[..4].iter().all(u8::is_ascii_digit)
        && bytes[4] == b'-'
        && bytes[5..7].iter().all(u8::is_ascii_digit)
        && bytes[7] == b'-'
        && bytes[8..10].iter().all(u8::is_ascii_digit)
    {
        raw[..10].to_owned()
    } else {
        raw.to_owned()
    }
}

/// Bóc bốn trường xuất xứ từ `html` (văn bản ĐÃ GIẢI MÃ) cộng `url` (URL YÊU CẦU, đã có sẵn ở
/// `Flow.labels` — xem doc-comment đầu tệp). **0 lời gọi mạng, 0 điểm panic** — mọi nguồn tín
/// hiệu trượt (thẻ vắng, JSON-LD sai cú pháp) chỉ khiến hàm thử nguồn KẾ TIẾP.
///
/// ─────────────────────────────────────────────────────────────────────────────
/// NGUỒN TÍN HIỆU, THEO THỨ TỰ THỬ — MỖI TRƯỜNG MỘT DANH SÁCH RIÊNG
/// ─────────────────────────────────────────────────────────────────────────────
/// - `author`: ① `<meta name="author">` ② `<meta property="article:author">`
///   ③ JSON-LD `author` (chuỗi, hoặc object/mảng mang `name`).
/// - `site_name`: ① `<meta property="og:site_name">` ② JSON-LD `publisher` (chuỗi, hoặc
///   object/mảng mang `name`).
/// - `published_at`: ① `<meta property="article:published_time">` ② `<meta name="date">`
///   ③ JSON-LD `datePublished` ④ thuộc tính `datetime` của thẻ `<time>` ĐẦU TIÊN. Giá trị tìm
///   được đi qua [`truncate_iso_date`].
pub fn extract_origin(html: &str, url: &str) -> ChapterOrigin {
    let document = HtmlDocument::from(html);
    let json_ld = json_ld_objects(&document);

    let author = meta_content(&document, "name", "author")
        .or_else(|| meta_content(&document, "property", "article:author"))
        .or_else(|| json_ld.iter().find_map(|o| json_ld_string_field(o, "author")));

    let site_name = meta_content(&document, "property", "og:site_name")
        .or_else(|| json_ld.iter().find_map(|o| json_ld_string_field(o, "publisher")));

    let published_at = meta_content(&document, "property", "article:published_time")
        .or_else(|| meta_content(&document, "name", "date"))
        .or_else(|| json_ld.iter().find_map(|o| json_ld_string_field(o, "datePublished")))
        .or_else(|| {
            document
                .select("time")
                .nodes()
                .first()
                .and_then(|n| n.attr("datetime"))
                .and_then(|v| chapter_origin_trim_or_none(&v))
        })
        .map(|v| truncate_iso_date(&v));

    ChapterOrigin { author, site_name, url: chapter_origin_trim_or_none(url), published_at }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracts_all_three_html_fields_when_the_page_declares_them() {
        let html = r#"<html><head>
            <meta name="author" content="Nguyen Van A">
            <meta property="og:site_name" content="Bao Thi Du">
            <meta property="article:published_time" content="2026-09-10T08:00:00+07:00">
        </head><body></body></html>"#;
        let origin = extract_origin(html, "https://example.test/bai-viet");
        assert_eq!(origin.author, Some("Nguyen Van A".to_owned()));
        assert_eq!(origin.site_name, Some("Bao Thi Du".to_owned()));
        assert_eq!(origin.published_at, Some("2026-09-10".to_owned()));
        assert_eq!(origin.url, Some("https://example.test/bai-viet".to_owned()));
    }

    #[test]
    fn missing_author_tag_falls_back_to_none_while_other_fields_still_fill() {
        let html = r#"<html><head>
            <meta property="og:site_name" content="Bao Thi Du">
        </head><body></body></html>"#;
        let origin = extract_origin(html, "https://example.test/bai-viet");
        assert_eq!(origin.author, None);
        assert_eq!(origin.site_name, Some("Bao Thi Du".to_owned()));
    }

    #[test]
    fn malformed_json_ld_is_skipped_and_the_next_signal_source_still_fills() {
        let html = r#"<html><head>
            <script type="application/ld+json">{ not valid json </script>
            <meta name="author" content="Nguyen Van A">
        </head><body></body></html>"#;
        let origin = extract_origin(html, "https://example.test/bai-viet");
        assert_eq!(origin.author, Some("Nguyen Van A".to_owned()));
    }

    #[test]
    fn json_ld_author_object_with_name_is_read_when_meta_tags_are_absent() {
        let html = r#"<html><head>
            <script type="application/ld+json">
            {"@context":"https://schema.org","@type":"Article","author":{"@type":"Person","name":"Tran Thi B"},"publisher":{"@type":"Organization","name":"Bao JSON"},"datePublished":"2026-09-01"}
            </script>
        </head><body></body></html>"#;
        let origin = extract_origin(html, "https://example.test/bai-viet");
        assert_eq!(origin.author, Some("Tran Thi B".to_owned()));
        assert_eq!(origin.site_name, Some("Bao JSON".to_owned()));
        assert_eq!(origin.published_at, Some("2026-09-01".to_owned()));
    }

    #[test]
    fn empty_url_becomes_none_not_an_empty_string() {
        let origin = extract_origin("<html></html>", "");
        assert_eq!(origin.url, None);
    }

    #[test]
    fn a_non_iso_published_at_is_kept_verbatim_not_forced_into_a_date_shape() {
        let html = r#"<html><head><meta property="article:published_time" content="3 gio truoc"></head></html>"#;
        let origin = extract_origin(html, "");
        assert_eq!(origin.published_at, Some("3 gio truoc".to_owned()));
    }

    #[test]
    fn whitespace_only_meta_content_counts_as_absent() {
        let html = r#"<html><head><meta name="author" content="   "></head></html>"#;
        let origin = extract_origin(html, "");
        assert_eq!(origin.author, None);
    }

    /// AI-7 — le seam đo được 2026-09-07/2026-09-15: một `<meta>` chỉ mang `U+FEFF` (BOM)
    /// phải đọc như VẮNG, khớp JS `.trim()`, qua ĐÚNG đường `meta_content`.
    #[test]
    fn a_meta_tag_holding_only_a_byte_order_mark_is_read_as_absent() {
        let html = "<html><head><meta name=\"author\" content=\"\u{FEFF}\"></head></html>";
        let origin = extract_origin(html, "");
        assert_eq!(origin.author, None, "meta_content phai coi BOM la vang, dung luat cat cua JS .trim()");
    }

    /// AI-7, D1 (chiều ngược) — một `<meta>` chỉ mang `U+0085` (NEL) phải GIỮ VERBATIM, vì cả
    /// hai máy JS đều KHÔNG cắt NEL bằng `.trim()`. Qua ĐÚNG đường `meta_content`.
    #[test]
    fn a_meta_tag_holding_only_a_next_line_character_is_kept_verbatim() {
        let html = "<html><head><meta name=\"author\" content=\"\u{0085}\"></head></html>";
        let origin = extract_origin(html, "");
        assert_eq!(
            origin.author,
            Some("\u{0085}".to_owned()),
            "meta_content phai GIU NEL lam noi dung, khong cat thanh vang"
        );
    }

    /// AI-7 — cùng seam BOM, nhưng qua đường JSON-LD, cả hai nhánh của `json_ld_string_or_named`:
    /// `author` là một CHUỖI TRẦN (nhánh `Value::String`), `publisher` là một OBJECT mang
    /// `name` (nhánh `Value::Object`). Không `<meta>` nào trong tài liệu — cô lập đúng đường
    /// JSON-LD, không để `meta_content` che lấp seam.
    #[test]
    fn a_json_ld_field_holding_only_a_byte_order_mark_is_read_as_absent_on_both_shapes() {
        let html = "<html><head><script type=\"application/ld+json\">\
             {\"author\":\"\u{FEFF}\",\"publisher\":{\"name\":\"\u{FEFF}\"}}\
             </script></head></html>";
        let origin = extract_origin(html, "");
        assert_eq!(origin.author, None, "nhanh Value::String phai coi BOM la vang");
        assert_eq!(origin.site_name, None, "nhanh Value::Object (qua .name) phai coi BOM la vang");
    }

    /// AI-7, D1 (chiều ngược) — cùng seam NEL, qua ĐÚNG hai nhánh của `json_ld_string_or_named`
    /// như ca BOM ngay trên, không `<meta>` nào để cô lập đường JSON-LD.
    #[test]
    fn a_json_ld_field_holding_only_a_next_line_character_is_kept_verbatim_on_both_shapes() {
        let html = "<html><head><script type=\"application/ld+json\">\
             {\"author\":\"\u{0085}\",\"publisher\":{\"name\":\"\u{0085}\"}}\
             </script></head></html>";
        let origin = extract_origin(html, "");
        assert_eq!(
            origin.author,
            Some("\u{0085}".to_owned()),
            "nhanh Value::String phai GIU NEL lam noi dung"
        );
        assert_eq!(
            origin.site_name,
            Some("\u{0085}".to_owned()),
            "nhanh Value::Object (qua .name) phai GIU NEL lam noi dung"
        );
    }

    #[test]
    fn no_signal_anywhere_yields_an_empty_origin_not_a_panic() {
        let origin = extract_origin("<html><body><p>khong the tag nao</p></body></html>", "");
        assert!(origin.is_empty());
    }

    #[test]
    fn time_tag_datetime_attribute_is_the_last_resort_signal_for_published_at() {
        let html = r#"<html><body><time datetime="2026-08-01T00:00:00Z">1 thang 8</time></body></html>"#;
        let origin = extract_origin(html, "");
        assert_eq!(origin.published_at, Some("2026-08-01".to_owned()));
    }
}
