//! `Extractor` — byte HTML ĐÃ GIẢI MÃ (văn bản, không phải byte thô) → văn bản thuần. **0
//! dòng chạm mạng** — không `reqwest`, không `TcpStream`, không literal `http://`/`https://`
//! ở đây (`webimport_boundary.rs` canh mệnh đề này bằng cách quét TĨNH tệp này).
//!
//! ─────────────────────────────────────────────────────────────────────────────
//! 🔴 `Article::content` (HTML) KHÔNG BAO GIỜ RỜI HÀM NÀY — AD-16 §Rule mục 1/2
//! ─────────────────────────────────────────────────────────────────────────────
//! [`extract`] chỉ đọc `Article::text_content` (văn bản thuần); `Article::content`
//! (`StrTendril`, HTML) không được gán vào biến, không được trả về, không được ghi log —
//! `webimport_boundary.rs` canh "0 dòng nào để `Article::content` rời module" bằng cách quét
//! literal `.content` trong tệp này.
//!
//! ─────────────────────────────────────────────────────────────────────────────
//! VÌ SAO `TextMode::Formatted`, KHÔNG PHẢI MẶC ĐỊNH `Raw`
//! ─────────────────────────────────────────────────────────────────────────────
//! **Task 1 — ĐO TRƯỚC KHI VIẾT (2026-09-06).** `TextMode::Raw` (mặc định của
//! `dom_smoothie::Config`) nối các đoạn liền nhau — mất ranh giới. Đo thật trên bảy trang
//! HTML cache của bàn đo 6.1 (`_bmad-output/implementation-artifacts/6-1-ban-do/fixtures/html/`)
//! với `TextMode::Formatted`: số lần "xuống hai dòng liên tiếp" trong `text_content` khớp
//! (lệch đúng 1, do N đoạn có N-1 ranh giới) với số thẻ `<p` xấp xỉ trong `article.content`
//! trên SÁU trong bảy mẫu (a01: 6↔5, a02: 8↔7, a03: 41↔45, a04: 5↔4, a05: 33↔35, a06: 9↔8);
//! mẫu còn lại (a07) là chính trang KHÔNG PHẢI bài viết mà bàn đo 6.1 đã ghi nhận — số liệu
//! hỗn loạn ở đó là kỳ vọng đúng, không phải một thất bại của phép đo. Kết luận: `Formatted`
//! GIỮ được ranh giới đoạn bằng `"\n\n"` — bước 4 (`normalize::normalize`, luật gộp dòng)
//! và bước 5 (tách Chương) vẫn tính đúng trên cấu trúc dòng. Không DỪNG.

use dom_smoothie::{Config, Readability, TextMode};

/// Mọi cách [`extract`] thất bại — MỘT lý do duy nhất phía người dùng thấy được
/// ("không bóc được nội dung chính", I/O Matrix spec 6.7), `detail` chỉ để chẩn đoán/log.
///
/// 🔴 **Không dùng `Readability::is_probably_readable()` làm điều kiện từ chối.** Bàn đo 6.1
/// (`REPORT.md` §Giới hạn) đã ghi nhận ÍT NHẤT một âm tính giả (mẫu `a04` — một bài viết
/// THẬT bị cờ này chấm sai "không giống bài viết"). Điều kiện đúng là NỘI DUNG THẬT SỰ bóc
/// ra được hay không (`parse()` thành công VÀ `text_content` không rỗng sau khi trim), không
/// phải một cờ suy đoán trước khi bóc.
#[derive(Debug)]
pub struct ExtractError {
    pub detail: String,
}

/// Bóc nội dung chính từ `html` (văn bản ĐÃ GIẢI MÃ — bước 1 AD-39 đứng TRƯỚC bước này
/// trong `PIPELINE_ORDER`, xem `core::segment::pipeline::Step::ExtractMainContent`).
/// `url` dùng để `dom_smoothie` phân giải các đường dẫn TƯƠNG ĐỐI bên trong tài liệu khi
/// tính điểm/cấu trúc — không ảnh hưởng `text_content` trả về, và không bao giờ được trả
/// lại từ hàm này.
pub fn extract(html: &str, url: &str) -> Result<String, ExtractError> {
    // 🔴 `TextMode::Formatted` — xem doc-comment đầu tệp (Task 1). Mọi trường khác giữ mặc
    // định: `max_elements_to_parse = 0` (không trần — trần byte đã có ở `Fetcher`, một trần
    // SỐ PHẦN TỬ thứ hai ở đây không có phép đo nào chống lưng, đừng đúc thêm một hằng số
    // phù thuỷ).
    let config = Config { text_mode: TextMode::Formatted, ..Config::default() };

    let mut readability = Readability::new(html.to_owned(), Some(url), Some(config))
        .map_err(|e| ExtractError { detail: format!("readability_new: {e}") })?;

    let article = readability
        .parse()
        .map_err(|e| ExtractError { detail: format!("readability_parse: {e}") })?;

    // `Article::content` (HTML) KHÔNG được đọc ở đây, cố ý — xem doc-comment đầu tệp.
    let text_content: String = article.text_content.to_string();

    if text_content.trim().is_empty() {
        return Err(ExtractError { detail: "text_content rong sau khi trim".to_owned() });
    }

    Ok(text_content)
}
