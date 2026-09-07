//! `Fetcher` — ĐIỂM RA MẠNG THỨ BA của toàn ứng dụng (AD-15), cài đặt DUY NHẤT MÃI MÃI
//! (Story 6.7 §Intent). URL → byte + `content-type`. **0 dòng phân tích nội dung** — không
//! `dom_smoothie`, không `Readability`, không đọc một thẻ HTML nào ở đây
//! (`webimport_boundary.rs` canh mệnh đề này bằng cách quét TĨNH tệp này).
//!
//! ─────────────────────────────────────────────────────────────────────────────
//! BA NĂNG LỰC — ĐÃ ĐO Ở BÀN ĐO 6.1 (`webimport_probe.rs`), KHÔNG ĐÚC LẠI TỪ ĐẦU
//! ─────────────────────────────────────────────────────────────────────────────
//! `reqwest::blocking` (đã ghim, feature `blocking` bật từ Story 6.1) đo đủ ba năng lực mà
//! hàm [`fetch`] cần: chặn chuyển hướng khác host tại chặng (`redirect::Policy::custom`),
//! cắt thân theo dòng chảy qua `impl Read` (không `.bytes()`/`.text()`), và phân loại lỗi
//! mạng qua `is_connect()`/`is_timeout()`. Xem `reqwest-raw.tsv` cho số đo gốc.
//!
//! ─────────────────────────────────────────────────────────────────────────────
//! HAI HẰNG NGƯỠNG — NGUỒN SỐ, KHÔNG ĐÚC TUỲ Ý
//! ─────────────────────────────────────────────────────────────────────────────
//! [`MAX_RESPONSE_BYTES`] và [`REQUEST_TIMEOUT`] đều kèm phép đo tại chỗ khai — xem
//! doc-comment của từng hằng.

use std::io::Read;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

/// Trần byte MỘT phản hồi.
///
/// 🔴 **Đo 2026-09-06** trên chính bảy trang bài báo thật của bàn đo 6.1
/// (`_bmad-output/implementation-artifacts/6-1-ban-do/fixtures/html/a01.html`…`a07.html`,
/// `wc -c`): 153.752–295.434 byte — trang lớn nhất chưa tới 300 KiB. 20 MiB cho biên độ
/// ~70–130× trên trang thật đã đo, và trùng đúng con số "quảng cáo" (`ADVERTISED_LEN`)
/// mà chính `webimport_probe.rs::size_cap_case` dùng làm kịch bản "rõ ràng không nên nạp
/// trọn" — không phải một con số đúc tuỳ ý.
pub const MAX_RESPONSE_BYTES: usize = 20 * 1024 * 1024;

/// Trần thời gian MỘT yêu cầu (kết nối + toàn bộ thân trả về).
///
/// 🔴 **Không đúc mới** — chép nguyên số ĐÃ ĐO của bàn đo 6.1: `webimport_probe.rs` gọi
/// thật bảy trang `epochtimes.com` với `Duration::from_secs(20)` và cả bảy phản hồi đều
/// trong hạn (không ca nào timeout, xem `EXTRACT_SUMMARY` của bàn đo).
pub const REQUEST_TIMEOUT: Duration = Duration::from_secs(20);

/// Cỡ đệm đọc một lượt `Read::read` — 64 KiB, cùng con số `webimport_probe.rs::size_cap_case`
/// đã dùng để đo bất biến `CAP ≤ actually_read ≪ ADVERTISED_LEN`.
const CHUNK_SIZE: usize = 64 * 1024;

/// Trần số chặng CHUYỂN HƯỚNG CÙNG HOST.
///
/// 🔴 **Không đúc mới — chép nguyên trần MẶC ĐỊNH của `reqwest`.** `redirect::Policy::custom`
/// THAY TRỌN chính sách mặc định (doc-comment của chính `Policy::custom`: "the custom variant
/// does not do that for you automatically"), nên trần ~10 chặng biến mất nếu không tự thêm.
/// Nguồn con số 10: `reqwest-0.13.4/src/redirect.rs` — `impl Default for Policy { fn
/// default() -> Policy { Policy::limited(10) } }`, và `Policy::limited` tự so
/// `attempt.previous().len() > max`. Không có trần này, một vòng lặp chuyển hướng CÙNG host
/// (AD-41 không chặn vì host không đổi) chỉ dừng lại nhờ [`REQUEST_TIMEOUT`] (20 s) và bị báo
/// SAI cho người dùng là [`FetchError::Timeout`] — một chẩn đoán sai (vòng rà đối kháng P2).
const MAX_SAME_HOST_REDIRECTS: usize = 10;

/// Kết quả một lượt tải THÀNH CÔNG — byte thô cộng `content-type` (nếu máy chủ có khai).
///
/// 🔴 `bytes` là byte THÔ, CHƯA giải mã, CHƯA phân tích — chỗ gọi tự quyết định làm gì với
/// nó (kiểm `content-type`, đưa vào pipeline làm `ChapterInput::RawBytes`).
#[derive(Debug, Clone)]
pub struct FetchedPage {
    pub bytes: Vec<u8>,
    pub content_type: Option<String>,
}

/// Mọi cách [`fetch`] thất bại — phân biệt đủ để `commands::project` dựng đúng lý do hiển
/// thị cho TỪNG mục trong danh sách URL (I/O Matrix spec 6.7).
#[derive(Debug)]
pub enum FetchError {
    /// Chuỗi đưa vào không phải một URL tuyệt đối hợp lệ.
    InvalidUrl { detail: String },
    /// `redirect::Policy::custom` chặn một chuyển hướng sang host KHÁC host của chính URL
    /// gốc — máy chủ đích chưa từng nhận kết nối nào.
    RedirectBlockedCrossHost,
    /// Máy chủ trả một mã lỗi HTTP (4xx/5xx).
    HttpStatus { status: u16 },
    /// Thân trả về vượt [`MAX_RESPONSE_BYTES`] — đọc dừng NGAY, không nạp trọn.
    TooLarge,
    /// Không kết nối được (DNS, refused, …) — `reqwest::Error::is_connect() == true`.
    ConnectFailed { detail: String },
    /// Hết thời gian chờ — `reqwest::Error::is_timeout() == true`.
    Timeout { detail: String },
    /// Lỗi khác (đọc thân trượt giữa chừng, dựng client thất bại, …) — chẩn đoán CHỈ cho
    /// log, không phân loại thêm được nữa.
    Other { detail: String },
}

/// Tải MỘT trang. **0 dòng phân tích nội dung** — chỗ gọi chịu trách nhiệm mọi việc CÒN LẠI
/// (kiểm `content-type`, bóc nội dung).
///
/// Thứ tự: phân giải URL tuyệt đối → dựng client với chính sách chuyển hướng chỉ-cùng-host
/// → gửi yêu cầu → nếu bị CHẶN ở một chuyển hướng thì trả `RedirectBlockedCrossHost` → nếu
/// mã trạng thái là lỗi thì trả `HttpStatus` → đọc thân qua [`Read`] (KHÔNG `.bytes()`/
/// `.text()`) theo khối, dừng NGAY khi vượt [`MAX_RESPONSE_BYTES`].
pub fn fetch(url: &str) -> Result<FetchedPage, FetchError> {
    let parsed = reqwest::Url::parse(url).map_err(|e| FetchError::InvalidUrl { detail: e.to_string() })?;

    // Chỉ host CỦA CHÍNH URL gốc được phép đi tiếp — mọi chuyển hướng sang host khác bị
    // chặn TẠI CHẶNG (AD-41). So sánh HOST, không PORT — khác bàn đo 6.1 (dùng port để giả
    // lập "host khác" trên cùng máy loopback); ở đây đã có host thật để so trực tiếp.
    let origin_host = parsed.host_str().map(str::to_owned);
    // Cờ CHÍNH SÁCH tự đặt khi nó THẬT SỰ chặn một chặng khác host — dùng cờ này (không dùng
    // `status().is_redirection()` một mình) để nhận diện "bị chặn" sau `send()` (P3 vòng rà
    // đối kháng bước 4): một 3xx LÀNH (300 Multiple Choices, 304, hoặc một 3xx không có
    // `Location`) cũng khớp `is_redirection()` dù chưa từng bị chính sách này chạm tới.
    let blocked_cross_host = Arc::new(AtomicBool::new(false));
    let blocked_cross_host_in_policy = Arc::clone(&blocked_cross_host);
    let policy = reqwest::redirect::Policy::custom(move |attempt| {
        if attempt.previous().len() > MAX_SAME_HOST_REDIRECTS {
            return attempt.error("qua tran so chang chuyen huong cung host (P2)");
        }
        if attempt.url().host_str() == origin_host.as_deref() {
            attempt.follow()
        } else {
            blocked_cross_host_in_policy.store(true, Ordering::SeqCst);
            attempt.stop()
        }
    });

    let client = reqwest::blocking::Client::builder()
        .redirect(policy)
        .timeout(REQUEST_TIMEOUT)
        .build()
        .map_err(|e| FetchError::Other { detail: e.to_string() })?;

    let resp = client.get(parsed).send().map_err(classify_send_error)?;

    // `Policy::stop()` KHÔNG biến thành một `Err` — `send()` trả `Ok` mang chính response
    // 3xx đã bị chặn (header `Location` còn nguyên). Chỉ cờ `blocked_cross_host` (đặt bởi
    // CHÍNH policy phía trên) mới xác nhận "đây LÀ chuyển hướng bị chặn" — không phải mọi
    // `status().is_redirection()` (P3 vòng rà đối kháng bước 4).
    if blocked_cross_host.load(Ordering::SeqCst) {
        return Err(FetchError::RedirectBlockedCrossHost);
    }
    // Một 3xx còn lại (không bị chính sách chặn — máy chủ tự trả nó làm phản hồi CUỐI, ví dụ
    // không có `Location`, hoặc 304) không phải nội dung dùng được cho `Extractor` — coi như
    // một mã lỗi HTTP, cùng khuôn 4xx/5xx bên dưới. `error_for_status()` không tự làm việc
    // này (nó chỉ bắt 4xx/5xx), nên kiểm status TRỰC TIẾP trước khi gọi nó.
    if resp.status().is_redirection() {
        return Err(FetchError::HttpStatus { status: resp.status().as_u16() });
    }
    let mut resp = match resp.error_for_status() {
        Ok(r) => r,
        Err(e) => {
            return Err(match e.status() {
                Some(status) => FetchError::HttpStatus { status: status.as_u16() },
                None => classify_send_error(e),
            });
        }
    };

    let content_type = resp
        .headers()
        .get(reqwest::header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .map(str::to_owned);

    let mut buf = [0u8; CHUNK_SIZE];
    let mut out: Vec<u8> = Vec::new();
    loop {
        match resp.read(&mut buf) {
            Ok(0) => break,
            Ok(n) => {
                out.extend_from_slice(&buf[..n]);
                if out.len() > MAX_RESPONSE_BYTES {
                    drop(resp);
                    return Err(FetchError::TooLarge);
                }
            }
            Err(e) => return Err(FetchError::Other { detail: e.to_string() }),
        }
    }

    Ok(FetchedPage { bytes: out, content_type })
}

fn classify_send_error(e: reqwest::Error) -> FetchError {
    if e.is_timeout() {
        FetchError::Timeout { detail: e.to_string() }
    } else if e.is_connect() {
        FetchError::ConnectFailed { detail: e.to_string() }
    } else {
        FetchError::Other { detail: e.to_string() }
    }
}

/// `content-type` có phải HTML hay không — kiểm PROTOCOL METADATA (một header), không phải
/// phân tích NỘI DUNG (`Fetcher` không chạm byte thân để quyết định điều này). `None` (máy
/// chủ không khai) được coi là KHÔNG PHẢI HTML — im lặng đoán "chắc là HTML" khi máy chủ
/// không nói gì là một phỏng đoán, không phải một sự thật đọc được.
///
/// 🔴 **So BẰNG kiểu MIME, không `contains` chuỗi con** (P4 vòng rà đối kháng bước 4) —
/// `contains("text/html")` khớp nhầm `text/htmlfoo` và mọi tham số tình cờ chứa chuỗi đó.
/// Cắt tại dấu `;` ĐẦU TIÊN (bỏ tham số kiểu `charset=...`), `trim`, hạ chữ thường, rồi so
/// bằng chính xác với hai kiểu MIME HTML đã biết.
pub fn looks_like_html(content_type: Option<&str>) -> bool {
    content_type
        .map(|ct| ct.split(';').next().unwrap_or("").trim().to_ascii_lowercase())
        .is_some_and(|ct| ct == "text/html" || ct == "application/xhtml+xml")
}
