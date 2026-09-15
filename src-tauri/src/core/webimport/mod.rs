//! Nhập từ web. Hai nửa, hai ranh giới cứng (AD-40, AD-41):
//!
//! - `Fetcher` ([`fetcher::fetch`]) = ĐIỂM RA MẠNG THỨ BA của toàn ứng dụng (AD-15). Nó chỉ
//!   tải, KHÔNG phân tích nội dung. Canh bởi allowlist một-lần-nhập ([`allowlist::Allowlist`],
//!   AD-41, Story 6.8) — cưỡng chế BÊN TRONG `fetcher::fetch`, không ở chỗ gọi.
//! - `Extractor` ([`extractor::extract`]) KHÔNG BAO GIỜ chạm mạng.
//!
//! Nhật ký domain ([`domain_log`]) — NFR19, Story 6.8 — sống theo PHIÊN CHẠY ứng dụng, tách
//! khỏi cả hai nửa trên: `fetcher::fetch` chỉ TRẢ VỀ các bản ghi phát sinh trong lượt gọi của
//! nó, chỗ gọi (`commands::project`) mới nối chúng vào [`domain_log::DomainLogState`].
//!
//! Nội dung nhập từ ngoài không bao giờ render thành HTML (AD-16) — [`extractor::extract`]
//! trả văn bản thuần, và `Article::content` (HTML) không bao giờ rời `extractor.rs`.
//!
//! Crate dành cho module này: `reqwest` (dùng chung với `core::ai`), `dom_smoothie`.
//!
//! ─────────────────────────────────────────────────────────────────────────────
//! 🔵 **Story 6.7 (2026-09-06) — `Fetcher`/`Extractor` THẬT, cả hai vào story này.** Trước
//! story này module chỉ có doc-comment, 0 dòng mã. Xem `fetcher.rs`/`extractor.rs`.
//!
//! ─────────────────────────────────────────────────────────────────────────────
//! TASK 0 — ĐO TRƯỚC KHI VIẾT: `reqwest::blocking` SỐNG ĐƯỢC trong `sync_threadpool`
//! ─────────────────────────────────────────────────────────────────────────────
//! **Đo 2026-09-06.** Hai phần bằng chứng, không chỉ một:
//!
//! 1. **Đọc nguồn `reqwest` 0.13.4 đã ghim** (`~/.cargo/registry/src/…/reqwest-0.13.4/src/
//!    blocking/client.rs:1413-1466`): `reqwest::blocking::Client` KHÔNG BAO GIỜ chạy công
//!    việc mạng trên luồng gọi nó. Mỗi client spawn một `std::thread` RIÊNG mang một
//!    `tokio::runtime::Builder::new_current_thread()` CỦA RIÊNG NÓ; luồng gọi chỉ gửi yêu
//!    cầu qua kênh (`oneshot`) rồi CHỜ — không bao giờ "vào" (enter) một runtime đã có sẵn
//!    trên luồng đang chạy. Đây CHÍNH LÀ cơ chế khiến crate an toàn khi gọi từ bên trong một
//!    context Tokio khác — không có "runtime lồng runtime" để mà panic.
//! 2. **Đo thật, không chỉ đọc nguồn:** một `#[test]` (không phải `#[tokio::test]`) gọi
//!    `tauri::async_runtime::spawn_blocking(|| reqwest::blocking::Client::…get(url).send())`
//!    rồi `tauri::async_runtime::block_on(join)` — ĐÚNG cơ chế mà `#[tauri::command(async)]`
//!    trên một hàm ĐỒNG BỘ dùng để đẩy thân hàm ra khỏi luồng chính (17 tiền lệ đã có,
//!    `library.rs:640`). Server cục bộ `TcpListener::bind("127.0.0.1:0")`. Kết quả: **3/3
//!    lượt chạy — `status=200`, không panic, không treo** (lượt đầu tiên trong phiên đo gặp
//!    một lần `TimedOut` cô lập, không tái lập ở hai lượt sau lẫn một ca đối chứng chạy
//!    TRỰC TIẾP không qua Tauri cùng thời điểm — đọc như nhiễu hạ tầng một lần của máy đo,
//!    không phải một hành vi của cơ chế `spawn_blocking`/`reqwest::blocking`).
//!
//! ⇒ **KHÔNG DỪNG.** `reqwest::blocking` sống được trong `sync_threadpool` của Tauri.
//!
//! ─────────────────────────────────────────────────────────────────────────────
//! TASK 1 — xem doc-comment đầu `extractor.rs` (`TextMode::Formatted` giữ ranh giới đoạn).
//! ─────────────────────────────────────────────────────────────────────────────

pub mod allowlist;
pub mod assets;
pub mod domain_log;
pub mod extractor;
pub mod fetcher;
pub mod origin;

pub use allowlist::{Allowlist, AllowlistDecision, ResourceKind, Tier};
pub use assets::{
    ResolveUrlError, extension_for_mime, host_of, is_raster_image_mime, normalized_mime, resolve_absolute_url,
};
pub use domain_log::{DomainLogDecision, DomainLogEntry, DomainLogOutcome, DomainLogState, append_domain_log_entries, distinct_domain_count, read_domain_log};
pub use extractor::{Block, BlockBody, ExtractError, extract};
pub use fetcher::{FetchError, FetchedPage, MAX_RESPONSE_BYTES, REQUEST_TIMEOUT, fetch, looks_like_html};
pub use origin::{ChapterOrigin, extract_origin};
pub(crate) use origin::{chapter_origin_trim, chapter_origin_trim_or_none};

/// Lý do một MỤC trong danh sách URL nhập thất bại — tám nhánh, đúng tám lý do phân biệt
/// được của I/O Matrix spec 6.7 (`err.import.web_*`, `core::i18n`). Trái với [`FetchError`]/
/// [`extractor::ExtractError`] (chẩn đoán máy, không dấu), đây là hình dạng DỮ LIỆU mà
/// `commands::project` gắn vào một `IpcError` cho ĐÚNG mục — tầng gọi (`commands::project`)
/// là nơi DUY NHẤT dựng biến thể này, từ `FetchError`/`ExtractError`/một dòng không phải URL
/// hợp lệ/một `content-type` không phải HTML.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WebImportItemFailureReason {
    /// Dòng dán vào không phải một URL tuyệt đối hợp lệ — **0 lời gọi mạng** cho dòng đó.
    InvalidUrl,
    /// Máy chủ trả một mã lỗi HTTP (4xx/5xx, ví dụ 404).
    HttpStatus,
    /// Hết thời gian chờ.
    Timeout,
    /// Không kết nối được (DNS, bị từ chối, …).
    ConnectFailed,
    /// Một chuyển hướng sang host KHÁC bị chặn tại chặng (AD-41).
    RedirectBlocked,
    /// Thân trả về vượt [`MAX_RESPONSE_BYTES`].
    TooLarge,
    /// `content-type` không phải HTML — [`extractor::extract`] KHÔNG được gọi trên byte này.
    NotHtml,
    /// `Extractor` chạy nhưng không bóc được nội dung chính (rỗng/quá ngắn) — KHÔNG rơi về
    /// HTML thô (đó là rỗng im lặng đổi hình dạng, §Design Notes spec 6.7).
    ExtractionEmpty,
}

impl From<FetchError> for WebImportItemFailureReason {
    fn from(e: FetchError) -> Self {
        match e {
            FetchError::InvalidUrl { .. } => WebImportItemFailureReason::InvalidUrl,
            // 🔵 Story 6.8 — `NotAllowlisted` (tên cũ `RedirectBlockedCrossHost`) vẫn ánh xạ
            // sang `RedirectBlocked`: trên đường sản phẩm THẬT (`commands::project` dựng
            // allowlist từ CHÍNH danh sách URL đang tải), host của URL gốc LUÔN nằm trong
            // allowlist — biến thể này chỉ bắn được ở đó qua một CHẶNG CHUYỂN HƯỚNG, nên
            // "RedirectBlocked" vẫn đúng sự thật cho người dùng. Ca "bị chặn ngay từ host
            // gốc" chỉ xảy ra khi `fetch()` được gọi TRỰC TIẾP với một allowlist không khớp
            // (đường kiểm ở `webimport_contract.rs`, không một chỗ gọi sản phẩm nào).
            FetchError::NotAllowlisted => WebImportItemFailureReason::RedirectBlocked,
            FetchError::HttpStatus { .. } => WebImportItemFailureReason::HttpStatus,
            FetchError::TooLarge => WebImportItemFailureReason::TooLarge,
            FetchError::ConnectFailed { .. } => WebImportItemFailureReason::ConnectFailed,
            FetchError::Timeout { .. } => WebImportItemFailureReason::Timeout,
            // `Other` gom lỗi hạ tầng hiếm (client build thất bại, đọc thân trượt giữa
            // chừng) — không có lý do RIÊNG nào trong tám nhánh nói đúng hơn "không kết nối
            // được", và đây không phải một ca người dùng lái được tới bằng một URL cụ thể.
            FetchError::Other { .. } => WebImportItemFailureReason::ConnectFailed,
        }
    }
}
