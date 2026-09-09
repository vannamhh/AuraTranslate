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
//!
//! ─────────────────────────────────────────────────────────────────────────────
//! 🔵 **Story 6.8 (2026-09-07) — AD-41 vào ĐÚNG Ở ĐÂY, không ở chỗ gọi.** `fetch` nay nhận
//! thêm `&Allowlist` và `ResourceKind` (§Always spec 6.8: *"kiểm ở chỗ gọi thì `fetch` vẫn là
//! một cửa mở"*). Host của CHÍNH url gốc bị từ chối TRƯỚC khi mở kết nối (`send()` chưa từng
//! được gọi — server đích nhận đúng 0 kết nối), và MỖI chặng chuyển hướng đi qua CÙNG một
//! quyết định TRƯỚC khi hop đó được nối tới. `FetchError::RedirectBlockedCrossHost`
//! (tên cũ) đổi thành [`FetchError::NotAllowlisted`] — không chỉ đổi tên: mệnh đề nó canh đổi
//! từ *"khác host gốc"* sang *"không có trong allowlist"* (xem §Design Notes spec 6.8, "vì
//! sao chuyển hướng giữa hai host CÙNG tầng 1 được phép").
//!
//! ─────────────────────────────────────────────────────────────────────────────
//! 🔵 **Story 6.8, VÒNG RÀ SAU GIAO — MỘT CLIENT DÙNG CHUNG, KHÔNG MỘT CLIENT MỖI LƯỢT GỌI**
//! ─────────────────────────────────────────────────────────────────────────────
//! **Hồi quy đo được (2026-09-07, do Ice truy):** `webimport_contract.rs` chạy song song
//! (mặc định `cargo test`) mất tính xác định sau bản thi công đầu của story này — một cụm
//! CỐ ĐỊNH 12 ca (đúng những ca chạm mạng thật) đỏ khoảng 2/10 lượt, luôn cùng dạng lỗi
//! `Other { detail: "error sending request for url (...)" }`, không phải `is_connect()`/
//! `is_timeout()`. **Đo bằng cách GỠ, không suy luận:** một ca 404 CÔ LẬP lặp 150 lần tuần
//! tự trong MỘT tiến trình ⇒ 0/150 đỏ; cùng ca chạy TRÊN 16 luồng × 20 lượt bằng NHAU ⇒
//! 0/320 đỏ; nhưng chạy NGUYÊN VẸN cả 28 ca không đổi của `webimport_contract.rs` (bộ THẬT,
//! có ca CPU nặng như `perf_probe_twenty_links…`/`an_oversized_body…` chen cùng lượt) ⇒ tái
//! lập đúng 12 ca đỏ đó, ~1/8 lượt. Khác biệt DUY NHẤT giữa hai phép đo: bộ THẬT trộn việc
//! NẶNG CPU (bóc `dom_smoothie`/`Readability`) với việc TẠO CLIENT — và mỗi lời gọi [`fetch`]
//! trước bản vá này dựng MỘT `reqwest::blocking::Client` MỚI (doc-comment `mod.rs::TASK 0`
//! đã tự đo và GHI RÕ: *"Mỗi client spawn một `std::thread` RIÊNG mang một
//! `tokio::runtime::Builder::new_current_thread()` CỦA RIÊNG NÓ"*). Máy đo có 16 lõi;
//! `cargo test` mặc định chạy tới 16 ca song song, và bộ THẬT có ~20 ca chạm mạng — một đợt
//! khởi động ~16-20 luồng nền + runtime tokio CÙNG LÚC, cạnh tranh CPU với các ca bóc nội
//! dung nặng, là điều kiện DUY NHẤT phép đo hẹp (đồng nhất, không CPU nặng) không tái tạo
//! được. ⇒ **Đây là lỗi ĐƯỜNG SẢN PHẨM** (chi phí một luồng + một runtime MỖI lời gọi
//! `fetch`, không phải một khiếm khuyết của bộ test) — không phải hạ ngưỡng hay thêm retry:
//! [`fetch`] nay dùng ĐÚNG MỘT [`shared_client`] cho suốt tiến trình (`OnceLock`, dựng lười,
//! đúng khuyến cáo chính thức của `reqwest`: *"it is advised that you create one and reuse
//! it"*), và chính sách chuyển hướng đổi từ `redirect::Policy::custom` (đóng gói ở LÚC DỰNG
//! CLIENT, nên không thể dùng chung một client cho nhiều allowlist khác nhau) sang
//! `redirect::Policy::none()` cộng một VÒNG LẶP thủ công bên trong [`fetch`] — allowlist vẫn
//! được hỏi TRƯỚC mỗi hop (kể cả hop đầu), server bị chặn vẫn nhận **0** kết nối, chỉ khác là
//! không còn một closure chạy trên luồng nền của policy nữa (Story 6.7 dựng closure đó chính
//! vì `redirect::Policy::custom` ĐÒI một closure — không phải một lựa chọn kiến trúc độc lập
//! cần giữ). **Đối chứng sau vá:** `cargo test --test webimport_contract` mặc định (song
//! song) 10 lượt liên tiếp — xem log CI/PR cho con số thật, không đúc lại ở đây.

use std::io::Read;
use std::sync::OnceLock;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;

use super::allowlist::{Allowlist, AllowlistDecision, ResourceKind};
use super::domain_log::{DomainLogDecision, DomainLogEntry, DomainLogOutcome, now_epoch_ms};

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

/// Trần số chặng CHUYỂN HƯỚNG.
///
/// 🔴 **Không đúc mới — chép nguyên trần MẶC ĐỊNH của `reqwest`.** Nguồn con số 10:
/// `reqwest-0.13.4/src/redirect.rs` — `impl Default for Policy { fn default() -> Policy {
/// Policy::limited(10) } }`. Không có trần này, một vòng lặp chuyển hướng CÙNG host (AD-41
/// không chặn vì host không đổi) chỉ dừng lại nhờ [`REQUEST_TIMEOUT`] (20 s) và bị báo SAI
/// cho người dùng là [`FetchError::Timeout`] — một chẩn đoán sai (vòng rà đối kháng P2).
///
/// 🔵 **Story 6.8 — tên hằng giữ nguyên, PHẠM VI canh RỘNG hơn.** Trước 6.8, cap này chỉ có
/// ý nghĩa cho vòng lặp CÙNG host (khác host bị chặn ngay hop đầu). Sau 6.8, hai host CÙNG
/// tầng 1 được phép chuyển hướng qua lại nhau (§Design Notes spec 6.8) — một vòng lặp GIỮA
/// hai host cùng allowlist giờ là một ca THẬT cần cap này chặn, không chỉ vòng lặp cùng host
/// nữa.
///
/// 🔵 **Vòng rà sau giao (§ đầu tệp "MỘT CLIENT DÙNG CHUNG") — đếm bằng TAY, không còn qua
/// `Policy::limited`/`attempt.previous().len()`.** [`fetch`] không còn dùng
/// `redirect::Policy::custom` (đóng gói ở LÚC DỰNG CLIENT — không hợp với một client DÙNG
/// CHUNG cho nhiều allowlist khác nhau); vòng lặp thủ công trong [`fetch`] tự đếm số hop đã
/// theo và so với hằng này TRƯỚC khi theo hop kế tiếp — cùng ngưỡng, cơ chế đổi chỗ đứng.
const MAX_REDIRECTS: usize = 10;

/// Cỡ hồ chứa client DÙNG CHUNG — xem [`shared_client`].
///
/// 🔵 **Vòng rà sau giao (2026-09-07) — phép đo dẫn tới con số 8, không đúc tuỳ ý.** MỘT
/// client dùng chung (thử trước) đẩy TOÀN BỘ lời gọi đồng thời qua ĐÚNG MỘT luồng nền/runtime
/// tokio của riêng nó (`reqwest::blocking` luôn single-thread cho một client, `mod.rs::TASK
/// 0`) — đo trên `webimport_contract.rs` (bộ THẬT, ~20 ca chạm mạng, máy 16 lõi): **15/15
/// lượt ĐỎ**, luôn đúng cùng 12 ca, một đợt ~20 lời gọi dồn về một luồng làm nó nghẽn ngay
/// tại lượt khởi động. Một hồ 8 client rải cùng đợt đó ra 8 luồng — đo lại: xem đối chứng ở
/// doc-comment đầu tệp cho con số 10 lượt liên tiếp thật. 8 không phải "đủ lớn cho chắc": nó
/// là `std::thread::available_parallelism()` điển hình của máy phát triển (8 lõi hiệu năng
/// trên máy 16 lõi ảo đã đo) — CHẶN trên bởi chính số luồng CPU thật có thể phục vụ đồng thời,
/// không phải một số phỏng đoán lớn hơn.
const CLIENT_POOL_SIZE: usize = 8;

/// Hồ client DÙNG CHUNG cho suốt tiến trình — dựng LƯỜI, đúng MỘT lần (`OnceLock::get_or_init`
/// tự khoá nếu nhiều luồng gọi đồng thời trước lần dựng đầu, xem doc `OnceLock`), rồi CHIA
/// đều lời gọi qua [`CLIENT_POOL_SIZE`] client bằng một bộ đếm xoay vòng — không một allowlist
/// hay trạng thái NÀO khác đi kèm client (client chỉ là hạ tầng mạng thuần tuý, xem §Design
/// Notes spec 6.8 "allowlist sống đúng một lần nhập": allowlist không hề đụng tới đây). Trả
/// `Result` (không `.expect()`/panic) vì `ClientBuilder::build()` CÓ thể trượt thật trên máy
/// người dùng (backend TLS không khởi tạo được) — một lỗi dựng client vẫn phải đi ra thành
/// `FetchError::Other` như trước 6.8, không được phép giết tiến trình (AD-11/AD-12: panic
/// trên đường BÁO LỖI cuốn theo cả `core::store`, `Cargo.toml` đặt `panic = "abort"`).
/// `OnceLock` không có một `get_or_try_init` ổn định (`once_cell_try` vẫn unstable ở bản
/// Rust đã ghim) nên mỗi `Result` được CHÍNH init closure bắt và lưu lại, không phải lan ra.
///
/// 🔵 **Vòng rà sau giao (2026-09-07) — lý do dùng CHUNG, không dựng mới mỗi lời gọi.** Xem
/// phép đo đầy đủ ở doc-comment đầu tệp ("MỘT CLIENT DÙNG CHUNG, KHÔNG MỘT CLIENT MỖI LƯỢT
/// GỌI") và ở [`CLIENT_POOL_SIZE`] (vì sao MỘT client không đủ). Tóm tắt: mỗi
/// `reqwest::blocking::Client` spawn MỘT luồng nền + MỘT runtime tokio riêng (`mod.rs::TASK
/// 0`) — dựng một client MỚI mỗi lời gọi `fetch` nhân số luồng nền lên theo số lời gọi ĐANG
/// chạy đồng thời, và dưới tải CPU thật (nhiều `fetch` cộng nhiều lượt bóc nội dung cùng lúc)
/// đợt khởi động luồng đó THỈNH THOẢNG khiến `send()` trượt với một lỗi mạng chung chung
/// (`Other`, không phải `is_connect()`/`is_timeout()`) — hồi quy ĐÃ ĐO trên
/// `webimport_contract.rs` chạy song song. `redirect(Policy::none())` vì chính sách chuyển
/// hướng của Story 6.7/6.8 giờ chạy THỦ CÔNG trong [`fetch`] (xem doc-comment hàm đó).
/// 🔴 **`pool_max_idle_per_host(0)` ĐÃ THỬ VÀ BỊ LOẠI** — đo trực tiếp trên
/// `reqwest 0.13.4`/`hyper-util 0.1.20` (bản đã ghim): một lời gọi ĐƠN, KHÔNG đồng thời, KHÔNG
/// lặp lại, vẫn trượt 100% với `hyper::Error(UnexpectedMessage)` — một lỗi CỦA THƯ VIỆN khi
/// `max_idle_per_host == 0`, không liên quan gì tới allowlist/vòng lặp chuyển hướng của story
/// này. Giữ NGUYÊN cấu hình pool MẶC ĐỊNH của `reqwest` (không gọi `pool_max_idle_per_host`).
fn shared_client() -> Result<&'static reqwest::blocking::Client, &'static str> {
    static POOL: OnceLock<Vec<Result<reqwest::blocking::Client, String>>> = OnceLock::new();
    static NEXT: AtomicUsize = AtomicUsize::new(0);

    let pool = POOL.get_or_init(|| {
        (0..CLIENT_POOL_SIZE)
            .map(|_| {
                reqwest::blocking::Client::builder()
                    .redirect(reqwest::redirect::Policy::none())
                    .timeout(REQUEST_TIMEOUT)
                    .build()
                    .map_err(|e| e.to_string())
            })
            .collect()
    });
    let idx = NEXT.fetch_add(1, Ordering::Relaxed) % pool.len();
    pool[idx].as_ref().map_err(String::as_str)
}

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
    /// 🔵 **Story 6.8 — tên cũ `RedirectBlockedCrossHost`, mệnh đề đổi cùng tên.** Host của
    /// URL gốc, HOẶC host của một chặng chuyển hướng, không nằm trong [`Allowlist`] cho
    /// [`ResourceKind`] đang xin (AD-41) — máy chủ đích chưa từng nhận kết nối nào. Không còn
    /// đúng nghĩa "khác host": hai host CÙNG tầng 1 chuyển hướng qua lại nhau được PHÉP (§Design
    /// Notes spec 6.8) — biến thể này chỉ bắn khi allowlist thật sự từ chối, bất kể host đó có
    /// trùng URL gốc hay không.
    NotAllowlisted,
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
/// 🔵 **Story 6.8 — `Fetcher` là chỗ DUY NHẤT cưỡng chế AD-41 (§Always spec 6.8).** `allowlist`
/// + `kind` đi vào TỪ THAM SỐ, không phải một trạng thái toàn cục — mỗi lời gọi tự mang theo
/// đủ dữ kiện để quyết định, đúng "allowlist sống đúng một lần nhập" (§Design Notes spec 6.8).
///
/// Thứ tự: phân giải URL tuyệt đối → vòng lặp thủ công tối đa [`MAX_REDIRECTS`] `+ 1` chặng,
/// MỖI chặng (kể cả chặng ĐẦU, trước khi có bất kỳ kết nối nào) hỏi `allowlist` TRƯỚC khi gửi
/// (từ chối ⇒ `NotAllowlisted`, đích đó nhận ĐÚNG 0 kết nối) → gửi qua [`shared_client`] với
/// `redirect(Policy::none())` (chính bản thân `reqwest` không tự theo chuyển hướng nữa — vòng
/// lặp ở đây tự quyết định có theo hay không) → một 3xx MANG `Location` hợp lệ thì lặp tiếp
/// với URL đó → một 3xx KHÔNG `Location` (hoặc `Location` không phân giải được) là phản hồi
/// CUỐI, coi như `HttpStatus` → mã lỗi HTTP khác trả `HttpStatus` → đọc thân qua [`Read`]
/// (KHÔNG `.bytes()`/`.text()`) theo khối, dừng NGAY khi vượt [`MAX_RESPONSE_BYTES`].
///
/// Trả về CẢ nhật ký domain đã phát sinh trong lượt gọi này (một bản ghi cho URL gốc, cộng
/// một bản ghi cho MỖI chặng chuyển hướng máy chủ THẬT SỰ thử đi tới — §Always spec 6.8: "một
/// bản ghi cho mọi lời gọi, cả cho phép lẫn từ chối") — chỗ gọi (`commands::project`) nối
/// chúng vào [`super::domain_log::DomainLogState`] của phiên chạy. `fetch` KHÔNG tự ghi vào
/// state đó: nó là một hàm THUẦN (0 `tauri::`, xem doc-comment đầu tệp `mod.rs`), và log
/// SỐNG theo phiên `AppHandle` mà chỉ tầng `commands::project`/`lib.rs` chạm tới được.
pub fn fetch(
    url: &str,
    allowlist: &Allowlist,
    kind: ResourceKind,
) -> (Result<FetchedPage, FetchError>, Vec<DomainLogEntry>) {
    let mut log: Vec<DomainLogEntry> = Vec::new();

    // 🔵 THÊM (vòng rà đối kháng 3, mục R1) — `url` không phân giải được thành một
    // `reqwest::Url` nghĩa là KHÔNG CÓ host nào để mà hỏi allowlist (0 kết nối, đúng hình
    // dạng `Denied`), nhưng lý do trượt KHÔNG PHẢI chính sách allowlist — đánh dấu `Other`
    // để phân biệt "bị chặn bởi chính sách" khỏi "không thể xác định được sẽ nối tới đâu".
    // Không có host thật, dùng nguyên văn `url` làm giá trị `domain` (chẩn đoán, không phải
    // một tên miền hợp lệ) — thà một chuỗi không hoàn hảo còn hơn 0 hàng kiểm toán, đúng
    // §Always "kể cả lượt trượt" (`domain_log.rs`).
    let mut current = match reqwest::Url::parse(url) {
        Ok(p) => p,
        Err(e) => {
            let entry = DomainLogEntry::new(now_epoch_ms(), url.to_owned(), kind, DomainLogDecision::Denied)
                .with_outcome(DomainLogOutcome::Other);
            return (Err(FetchError::InvalidUrl { detail: e.to_string() }), vec![entry]);
        }
    };

    // 🔵 THÊM (vòng rà đối kháng 3, mục R1) — ở ĐÂY `url` ĐÃ phân giải được (`current` có
    // host thật), nhưng client dùng chung dựng thất bại TRƯỚC khi kịp hỏi allowlist — cùng
    // lý lẽ nhánh trên: 0 kết nối (Denied) nhưng lý do KHÔNG PHẢI chính sách (Other).
    let client = match shared_client() {
        Ok(c) => c,
        Err(detail) => {
            let host = current.host_str().unwrap_or(url).to_owned();
            let entry = DomainLogEntry::new(now_epoch_ms(), host, kind, DomainLogDecision::Denied)
                .with_outcome(DomainLogOutcome::Other);
            return (Err(FetchError::Other { detail: detail.to_owned() }), vec![entry]);
        }
    };

    // 🔵 Vòng lặp thủ công — thay `redirect::Policy::custom` (xem doc-comment đầu tệp "MỘT
    // CLIENT DÙNG CHUNG"). Chặng đầu (`redirects_followed == 0`, chưa phải một chuyển hướng)
    // cộng tối đa `MAX_REDIRECTS` lần THEO một chuyển hướng — cùng ngân sách
    // `Policy::limited(MAX_REDIRECTS)` của `reqwest` từng cấp trước bản vá này.
    let mut redirects_followed: usize = 0;
    let resp = loop {
        let Some(host) = current.host_str().map(str::to_owned) else {
            // 🔵 THÊM (vòng rà đối kháng 3, mục R1) — cùng lý lẽ hai nhánh phía trên: một
            // chặng (chặng đầu hoặc một đích chuyển hướng) mất host giữa đường vẫn phải để
            // lại một hàng kiểm toán, không được im lặng trả `log` (có thể RỖNG nếu đây là
            // chặng đầu tiên).
            log.push(
                DomainLogEntry::new(now_epoch_ms(), current.to_string(), kind, DomainLogDecision::Denied)
                    .with_outcome(DomainLogOutcome::Other),
            );
            return (
                Err(FetchError::InvalidUrl { detail: "url khong co host".to_owned() }),
                log,
            );
        };

        // 🔴 AD-41 — host của chặng NÀY (chặng đầu HOẶC một đích chuyển hướng) bị hỏi
        // allowlist TRƯỚC khi `client.get(...).send()` được gọi. Từ chối ở đây nghĩa là ĐÚNG
        // 0 kết nối TCP mở ra tới host đó — không có `send()` nào chạy cho nó.
        match allowlist.decide(&host, kind) {
            AllowlistDecision::Denied => {
                log.push(DomainLogEntry::new(now_epoch_ms(), host, kind, DomainLogDecision::Denied));
                return (Err(FetchError::NotAllowlisted), log);
            }
            AllowlistDecision::Allowed(tier) => {
                // 🔵 **THÊM (Story 6.11, vòng rà đối kháng 3 lớp, Ice ký 2026-09-08)** — bản
                // ghi của CHÍNH chặng này chưa có KẾT QUẢ lúc `push` (chưa `send()`); nó được
                // điền vào bằng `mark_last_outcome` ở MỌI nhánh thoát bên dưới, kể cả lượt
                // trượt — xem doc-comment `DomainLogEntry::outcome`.
                log.push(DomainLogEntry::new(now_epoch_ms(), host, kind, DomainLogDecision::Allowed(tier)));
            }
        }

        let resp = match client.get(current.clone()).send() {
            Ok(r) => r,
            Err(e) => {
                let err = classify_send_error(e);
                mark_last_outcome(&mut log, outcome_for_fetch_error(&err));
                return (Err(err), log);
            }
        };

        if !resp.status().is_redirection() {
            break resp;
        }
        // Một 3xx không mang `Location` (hoặc `Location` không phân giải được thành một URL
        // hợp lệ) là phản hồi CUỐI, không phải một chuyển hướng đang chờ theo — coi như một mã
        // lỗi HTTP, cùng khuôn 4xx/5xx bên dưới (P3 vòng rà đối kháng bước 4 của Story 6.7: một
        // 3xx LÀNH, ví dụ 304, không được báo sai thành `NotAllowlisted`).
        let Some(location) = resp.headers().get(reqwest::header::LOCATION).and_then(|v| v.to_str().ok()) else {
            mark_last_outcome(&mut log, DomainLogOutcome::HttpStatus);
            return (Err(FetchError::HttpStatus { status: resp.status().as_u16() }), log);
        };
        let Ok(next) = current.join(location) else {
            mark_last_outcome(&mut log, DomainLogOutcome::HttpStatus);
            return (Err(FetchError::HttpStatus { status: resp.status().as_u16() }), log);
        };

        // Chặng NÀY thật sự chuyển hướng đi — bản ghi của nó khép lại ở đây; chặng KẾ TIẾP
        // (vòng lặp tiếp theo) mở một bản ghi RIÊNG của chính nó.
        mark_last_outcome(&mut log, DomainLogOutcome::Redirected);

        current = next;
        // `redirects_followed` đếm số chặng đã THEO (không tính chặng đầu) — so bằng
        // `MAX_REDIRECTS` ở đầu vòng lặp qua chính số lần lặp đã chạy; đếm tường minh ở đây để
        // không lệ thuộc lại một cơ chế đếm nội bộ của `reqwest`.
        redirects_followed += 1;
        if redirects_followed > MAX_REDIRECTS {
            // D4 (vòng rà đối kháng 2, 3 lớp) — bản ghi vừa đánh `Redirected` (dòng ngay
            // trên) là của chặng ĐÃ tìm ra `Location` và ĐỊNH theo, nhưng cuộc theo đó không
            // bao giờ diễn ra (vượt trần trước khi client kịp gọi host kế tiếp) — sửa lại
            // outcome thành `Other` để phản ánh đúng: fetch này KẾT THÚC bằng một lượt HUỶ vì
            // vượt trần, không phải một chuyển hướng thành công đang chờ chặng sau. Không sửa
            // lại, bản ghi cuối cùng đọc lên y hệt một chuyển hướng bình thường — không phân
            // biệt được với một `fetch` còn đang giữa chừng.
            mark_last_outcome(&mut log, DomainLogOutcome::Other);
            return (
                Err(FetchError::Other { detail: "qua tran so chang chuyen huong (P2)".to_owned() }),
                log,
            );
        }
    };

    let mut resp = match resp.error_for_status() {
        Ok(r) => r,
        Err(e) => {
            let err = match e.status() {
                Some(status) => FetchError::HttpStatus { status: status.as_u16() },
                None => classify_send_error(e),
            };
            mark_last_outcome(&mut log, outcome_for_fetch_error(&err));
            return (Err(err), log);
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
                    mark_last_outcome(&mut log, DomainLogOutcome::TooLarge);
                    return (Err(FetchError::TooLarge), log);
                }
            }
            Err(e) => {
                mark_last_outcome(&mut log, DomainLogOutcome::Other);
                return (Err(FetchError::Other { detail: e.to_string() }), log);
            }
        }
    }

    mark_last_outcome(&mut log, DomainLogOutcome::Fetched);
    (Ok(FetchedPage { bytes: out, content_type }), log)
}

/// Điền `outcome` vào bản ghi CUỐI CÙNG của `log` — đây LUÔN là bản ghi của chặng đang xử lý
/// (chặng vừa được `Allowed`, chưa từng bị ghi đè bởi một chặng khác vì mỗi chặng `push` một
/// bản ghi RIÊNG). Không làm gì nếu `log` rỗng — bất khả trên mọi nhánh gọi hàm này (luôn có
/// ít nhất một bản ghi `Allowed` vừa `push` trước đó), giữ hàm AN TOÀN thay vì `.unwrap()`.
fn mark_last_outcome(log: &mut [DomainLogEntry], outcome: DomainLogOutcome) {
    if let Some(last) = log.last_mut() {
        last.outcome = Some(outcome);
    }
}

/// Ánh xạ MỘT CHIỀU `FetchError` → `DomainLogOutcome` cho các biến thể có Ý NGHĨA MẠNG (loại
/// trừ `InvalidUrl`/`NotAllowlisted` — hai biến thể đó không bao giờ đi qua đường này: cái
/// trước không có host để `push` một bản ghi, cái sau tự `return` ngay tại nhánh `Denied`).
/// 🔵 **SỬA (vòng rà đối kháng 2, mục D3) — `InvalidUrl`/`NotAllowlisted` KHÔNG THỂ tới hàm
/// này.** Cả hai chỗ gọi (`classify_send_error(e)` sau `send()` lỗi, và sau
/// `resp.error_for_status()` lỗi) chỉ đưa vào đây những gì `classify_send_error` trả về
/// (`Timeout`/`ConnectFailed`/`Other`) hoặc `HttpStatus` dựng tại chỗ — `InvalidUrl` chỉ sinh
/// ra ở bước `Url::parse`/thiếu host (return sớm, KHÔNG qua `outcome_for_fetch_error`), và
/// `NotAllowlisted` chỉ sinh ra ở bước `allowlist.decide` (cũng return sớm, cũng KHÔNG qua
/// đây). `unreachable!()` thay vì gộp chung `Other` — nếu một lượt sửa sau này lỡ đổi luồng
/// khiến hai biến thể đó thật sự tới được đây, một PANIC lớn tiếng còn hơn một lượt gán
/// `Other` âm thầm SAI cho một lý do không phải là "lỗi mạng khác".
fn outcome_for_fetch_error(err: &FetchError) -> DomainLogOutcome {
    match err {
        FetchError::HttpStatus { .. } => DomainLogOutcome::HttpStatus,
        FetchError::TooLarge => DomainLogOutcome::TooLarge,
        FetchError::ConnectFailed { .. } => DomainLogOutcome::ConnectFailed,
        FetchError::Timeout { .. } => DomainLogOutcome::Timeout,
        FetchError::Other { .. } => DomainLogOutcome::Other,
        FetchError::InvalidUrl { .. } | FetchError::NotAllowlisted => {
            unreachable!(
                "outcome_for_fetch_error: {err:?} khong the toi day -- ca hai bien the nay \
                 chi sinh ra o mot nhanh return SOM, truoc khi outcome_for_fetch_error duoc goi"
            )
        }
    }
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
