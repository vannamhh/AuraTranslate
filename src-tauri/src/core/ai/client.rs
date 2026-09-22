//! Cài đặt DUY NHẤT của [`crate::ports::TranslationProvider`] hôm nay — một client streaming
//! tương thích OpenAI (chat-completions, `stream: true`). Spec 4.8, Phase 2, Task 1.
//!
//! ─────────────────────────────────────────────────────────────────────────────
//! 🔴 CLIENT ASYNC, KHÔNG BLOCKING — đo trên nguồn đã ghim, không suy đoán
//! ─────────────────────────────────────────────────────────────────────────────
//! `read_timeout` (khoảng CÁCH giữa hai lần đọc, không phải trần TOÀN BỘ request) chỉ tồn tại
//! trên `reqwest::ClientBuilder` ở `async_impl/client.rs:1456` — **0** lần xuất hiện trong
//! `src/blocking/client.rs` của cùng crate (`reqwest-0.13.4`, đọc trong nguồn đã tải,
//! `~/.cargo/registry/src/…`). Một client blocking chỉ chặn được TOÀN BỘ request, nên một trần
//! đủ ngắn để bắt một kết nối treo cũng giết một lượt sinh dài mà lành, còn một trần đủ dài để
//! cho lượt sinh dài đi qua thì không phân biệt được một lượt treo với một lượt đang chạy —
//! xem thêm §Design Notes spec 4.8. KHÔNG sao chép hồ bơi client/`REQUEST_TIMEOUT`/vòng lặp
//! chuyển hướng/allowlist của `core/webimport/fetcher.rs` — AD-41 là luật của web-import, và
//! endpoint AI là một host người dùng tự gõ vào, không phải một URL bóc từ trang lạ.
//!
//! ─────────────────────────────────────────────────────────────────────────────
//! 🔴 KHÔNG CRATE MỚI, KHÔNG FEATURE MỚI
//! ─────────────────────────────────────────────────────────────────────────────
//! `Cargo.toml` ghim `reqwest = { version = "=0.13.4", features = ["blocking"] }` — feature đó
//! CỘNG THÊM vào bộ mặc định (`default-tls`, `charset`, `http2`, `system-proxy`), không thay
//! thế; `reqwest::Client` (async) đã sẵn sàng dùng mà không cần khai thêm gì. Streaming ở đây
//! đọc từng khối qua [`reqwest::Response::chunk`] (`async_impl/response.rs:310`), hàm này
//! KHÔNG nằm sau feature `stream` — chỉ `bytes_stream()` (`:351`) mới cần feature đó, và spine
//! CAP-4 vẫn để mọi crate SSE (`reqwest-sse`, `sseer`) ở trạng thái chưa rà NFR15. Thân JSON
//! dựng bằng `serde_json::to_string` (crate đã có sẵn) rồi đặt thẳng làm `body()` — không cần
//! bật feature `json` của `reqwest` (feature đó chỉ thêm một hàm tiện `.json()`, không phải
//! điều kiện để gửi được JSON).
//!
//! ─────────────────────────────────────────────────────────────────────────────
//! 🔴 KHUNG SSE ĐƯỢC TÁCH BỞI MỘT HÀM THUẦN TRÊN BỘ ĐỆM BYTE
//! ─────────────────────────────────────────────────────────────────────────────
//! [`split_sse_frames`] nhận trọn bộ đệm đã tích luỹ (phần còn lại của lượt gọi trước, cộng
//! khối byte mới nhất) và trả về CẢ danh sách sự kiện đã tách được LẪN phần dư CHƯA đủ một sự
//! kiện — không `Store`, không `reqwest`, không đồng hồ. Đây là điều kiện để ma trận I/O của
//! spec 4.8 (một khung chia làm hai khối mạng, một khung tới trong hai lượt đọc, một luồng
//! ĐÓNG mà không có `[DONE]`) kiểm được bằng test thuần, không cần mở cổng nghe cục bộ nào —
//! AD-45 cấm cổng nghe trong bộ test, và một nhị phân test tự mở cổng đã gây đỏ giả trên máy
//! này hai lần (root `AGENTS.md`).
//!
//! ─────────────────────────────────────────────────────────────────────────────
//! ⚠️ MỖI LƯỢT GỌI DỰNG MỘT `reqwest::Client` RIÊNG, KHÔNG MỘT HỒ BƠI DÙNG CHUNG
//! ─────────────────────────────────────────────────────────────────────────────
//! Khác `core/webimport/fetcher.rs::shared_client` (nhiều host đổi liên tục, tần suất cao), một
//! lượt dịch là MỘT hành động người dùng chủ động bấm — chi phí bắt tay TLS thêm một lần không
//! phải nút thắt khi chính lượt đó đã trả token qua độ trễ mạng của nhà cung cấp, và giữ một
//! client dùng chung đòi thêm khoá/`OnceLock` cho một lợi ích chưa đo được (root `AGENTS.md`:
//! đo trước khi tối ưu). Batch dịch (Story 4.9) tự quyết lại nếu số đo thật đòi hồ bơi.
//!
//! ─────────────────────────────────────────────────────────────────────────────
//! Vì SAO `Error` KHÔNG MANG `TranslateRequest`/khoá API
//! ─────────────────────────────────────────────────────────────────────────────
//! Không biến thể nào dưới đây giữ `request.api_key` hay chính `TranslateRequest` — mọi chẩn
//! đoán chỉ mang mã trạng thái HTTP hoặc một chuỗi từ `reqwest`/`serde_json` (chẩn đoán KHÔNG
//! DẤU, cùng khuôn `PromptSetError`, `src-tauri/AGENTS.md`). Vì vậy kiểu này `derive(Debug)`
//! an toàn — luật cấm `derive(Debug)` chỉ nhắm `ApiKeySecret`/`TranslateRequest`/bất cứ gì BỌC
//! khoá đã lộ, không nhắm kiểu lỗi của module này.

use crate::ports::translation_provider::{TranslateOutcome, TranslateRequest, TranslationProvider};
use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Khoảng cách CHO PHÉP giữa hai lần đọc liên tiếp từ thân phản hồi — không phải trần toàn bộ
/// request (xem doc-comment đầu tệp). Số ban đầu, chưa có một lượt đo thật trên một nhà cung
/// cấp thật nào (Phase 2 không có chỗ gọi sản phẩm) — Phase 2/3 (chỗ gọi thật, `commands::aitranslate`)
/// hoặc một lượt đo sau khi có người dùng thật là nơi đúng để xét lại con số này.
const READ_TIMEOUT: Duration = Duration::from_secs(60);

/// Trần cho bộ đệm tích luỹ CHƯA-ĐỦ-một-sự-kiện của [`split_sse_frames`] — cùng khuôn
/// `core::webimport::fetcher::MAX_RESPONSE_BYTES`, ở quy mô NHỎ hơn nhiều vì đây chỉ là MỘT
/// khung SSE còn dang dở (payload JSON một delta, thường dưới 1 KB), không phải thân trang.
///
/// 🔵 **THÊM (review) —** không có trần, một endpoint sai hình hoặc ác ý không bao giờ gửi dấu
/// phân cách `\n\n`/`\r\n\r\n` khiến `buf` phình VÔ HẠN (`translate()` vẫn `.chunk().await` đều
/// đặn, mỗi khối lại `extend_from_slice` thêm vào cùng một `Vec` không giới hạn). Số ban đầu,
/// chưa có một lượt đo thật — xét lại cùng lúc với [`READ_TIMEOUT`] khi có người dùng thật.
const MAX_SSE_BUFFER_BYTES: usize = 1024 * 1024;

/// Cài đặt [`TranslationProvider`] cho một provider tương thích chat-completions của OpenAI.
/// Không giữ trạng thái nào giữa các lượt gọi (xem doc-comment đầu tệp — không hồ bơi client).
#[derive(Debug, Clone, Copy, Default)]
pub struct OpenAiChatClient;

impl OpenAiChatClient {
    pub fn new() -> Self {
        Self
    }
}

/// Mọi cách [`OpenAiChatClient::translate`] có thể thất bại — KHÔNG mang `TranslateRequest`
/// lẫn khoá API (xem doc-comment đầu tệp). Cấu trúc đủ để tầng lệnh (Phase 2 của story này,
/// `commands::aitranslate`, NGOÀI phạm vi Phase 2 hiện tại) đóng gói thành `IpcError` (AD-21)
/// mà không cần biết chi tiết bên trong.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OpenAiClientError {
    /// Dựng `reqwest::Client` thất bại (backend TLS thiếu, …) — chưa từng đo được trên máy
    /// nào, nhưng `Result` không được bỏ qua.
    ClientBuildFailed { detail: String },
    /// Gửi request thất bại TRƯỚC khi có phản hồi — DNS, từ chối kết nối, hoặc `read_timeout`
    /// hết hạn giữa hai khung (`reqwest::Error::is_timeout()`/`is_connect()` đều rơi vào đây,
    /// chẩn đoán giữ nguyên từ `reqwest`).
    RequestFailed { detail: String },
    /// Khoá API đã lưu không vào được giá trị header HTTP (`HeaderValue::from_str` thất bại) —
    /// TÁCH RIÊNG khỏi [`RequestFailed`] (Story 4.10, Quyết định 2): một khoá không hợp lệ theo
    /// hình dạng header thất bại GIỐNG HỆT mỗi lần thử lại, không phải một sự cố mạng tạm thời.
    ApiKeyHeaderInvalid { detail: String },
    /// Máy chủ trả một mã KHÔNG phải 2xx — I/O Matrix spec 4.8 "Provider returns a non-2xx".
    NonSuccessStatus { status: u16 },
    /// Đọc thân phản hồi lỗi GIỮA CHỪNG, sau khi đã nhận 2xx.
    ReadFailed { detail: String },
    /// Kết nối ĐÓNG lại trước khi thấy khung kết thúc `[DONE]` — I/O Matrix spec 4.8 "Stream
    /// ends without `[DONE]`". KHÔNG tự động thử lại (AD-22) — trả `Err`, chỗ gọi tự quyết.
    StreamEndedWithoutDone,
    /// Một khung `data:` không phải JSON hợp lệ, hoặc thiếu đúng hình dạng chunk
    /// chat-completions — không `unwrap`/`expect` trên dữ liệu từ mạng (`panic = "abort"`,
    /// `src-tauri/AGENTS.md`).
    MalformedEvent { detail: String },
    /// Bộ đệm tích luỹ SSE vượt [`MAX_SSE_BUFFER_BYTES`] mà vẫn chưa thấy dấu phân cách sự
    /// kiện nào — một endpoint sai hình/ác ý, không phải một lượt sinh dài lành mạnh (nội dung
    /// SINH ra nằm trong `data:` của TỪNG sự kiện đã tách xong, không nằm trong phần dư CHƯA
    /// tách được).
    BufferOverflow { size: usize },
}

impl std::fmt::Display for OpenAiClientError {
    /// KHÔNG DẤU (NFR16) — chẩn đoán cho log, không phải văn bản hiển thị.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OpenAiClientError::ClientBuildFailed { detail } => {
                write!(f, "ai_client[client_build_failed] detail={detail}")
            }
            OpenAiClientError::RequestFailed { detail } => {
                write!(f, "ai_client[request_failed] detail={detail}")
            }
            OpenAiClientError::ApiKeyHeaderInvalid { detail } => {
                write!(f, "ai_client[api_key_header_invalid] detail={detail}")
            }
            OpenAiClientError::NonSuccessStatus { status } => {
                write!(f, "ai_client[non_success_status] status={status}")
            }
            OpenAiClientError::ReadFailed { detail } => {
                write!(f, "ai_client[read_failed] detail={detail}")
            }
            OpenAiClientError::StreamEndedWithoutDone => {
                write!(f, "ai_client[stream_ended_without_done]")
            }
            OpenAiClientError::MalformedEvent { detail } => {
                write!(f, "ai_client[malformed_event] detail={detail}")
            }
            OpenAiClientError::BufferOverflow { size } => {
                write!(f, "ai_client[buffer_overflow] size={size}")
            }
        }
    }
}

impl std::error::Error for OpenAiClientError {}

/// Thân JSON gửi đi — MỘT tin nhắn `user` mang `request.prompt` NGUYÊN VĂN (AD-14, §Always
/// spec 4.8: *"never a second assembly, never a concatenation at the call site"*). `content`
/// đi qua `serde_json` để mã hoá JSON (thoát dấu ngoặc kép/`\n`/…) — đây KHÔNG phải một lượt
/// lắp lại prompt, chỉ là bước mã hoá bắt buộc để đặt một chuỗi bất kỳ vào một thân JSON hợp
/// lệ; giá trị `content` vẫn đúng CHÍNH XÁC `request.prompt`, từng byte.
/// `pub` (không `pub(crate)`) và không mang trường `pub` nào -- chỉ để KIỂU này (không phải
/// nội dung của nó) đi qua được chữ ký của [`build_request_body`], hàm `pub` mà
/// `tests/ai_translate_contract.rs` gọi để đối chứng Quyết định 6 (`skip_serializing_if` bỏ
/// hẳn `temperature`/`max_tokens` khỏi JSON khi `None`) mà KHÔNG cần một lượt gọi mạng thật
/// (AD-45 cấm một cổng nghe trong bộ test) -- serde tự sinh `impl Serialize` với đầy đủ quyền
/// đọc trường riêng tư ngay trong module này, nên các trường không cần `pub` để
/// `serde_json::to_string(&body)` chạy được từ một crate khác.
#[derive(Serialize)]
pub struct ChatCompletionsRequestBody<'a> {
    model: &'a str,
    /// Quyet dinh 6 spec 4.8 -- bo HAN truong nay khoi JSON khi nguoi dung chua dat, thay vi
    /// gui mot so thay ho. Endpoint tuong thich OpenAI coi ca hai la tuy chon.
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    max_tokens: Option<u32>,
    stream: bool,
    messages: [ChatMessage<'a>; 1],
}

#[derive(Serialize)]
struct ChatMessage<'a> {
    role: &'a str,
    content: &'a str,
}

/// Dựng thân JSON -- tách THUẦN khỏi [`OpenAiChatClient::translate`] (bản thân hàm đó không
/// đổi hành vi, chỉ gọi qua hàm này) đúng lý do doc-comment [`ChatCompletionsRequestBody`] nêu:
/// đây là seam DUY NHẤT để một test đối chứng `skip_serializing_if` mà không cần mở cổng nghe
/// nào. `request.prompt` đi NGUYÊN VĂN vào `content` -- không lắp lại, không nối thêm (AD-14).
pub fn build_request_body<'a>(request: &TranslateRequest<'a>) -> ChatCompletionsRequestBody<'a> {
    ChatCompletionsRequestBody {
        model: request.model,
        temperature: request.temperature,
        max_tokens: request.max_tokens,
        stream: true,
        messages: [ChatMessage { role: "user", content: request.prompt }],
    }
}

/// Hình dạng TỐI THIỂU của một chunk chat-completions cần đọc — `choices[0].delta.content`.
/// Mọi trường khác nhà cung cấp gửi thêm (`id`, `usage`, …) bị `serde` bỏ qua mặc định.
#[derive(Deserialize)]
struct ChatCompletionsChunk {
    #[serde(default)]
    choices: Vec<ChunkChoice>,
}

#[derive(Deserialize, Default)]
struct ChunkChoice {
    #[serde(default)]
    delta: ChunkDelta,
}

#[derive(Deserialize, Default)]
struct ChunkDelta {
    #[serde(default)]
    content: Option<String>,
}

impl TranslationProvider for OpenAiChatClient {
    type Error = OpenAiClientError;

    async fn translate(
        &self,
        request: TranslateRequest<'_>,
        on_token: &mut dyn FnMut(&str),
        should_cancel: &dyn Fn() -> bool,
    ) -> Result<TranslateOutcome, Self::Error> {
        let client = reqwest::Client::builder()
            .read_timeout(READ_TIMEOUT)
            .build()
            .map_err(|e| OpenAiClientError::ClientBuildFailed { detail: e.to_string() })?;

        let body = build_request_body(&request);
        // `serde_json::to_string` trên một kiểu dựng tay, đủ trường hợp lệ -- khong that bai
        // tren du lieu that; giu Result thay vi unwrap dung dung `AGENTS.md` (khong panic tren
        // duong nong), khong phai vi mong doi mot loi that.
        let body_json = serde_json::to_string(&body)
            .map_err(|e| OpenAiClientError::RequestFailed { detail: e.to_string() })?;

        // 🔴 **SUA 2026-09-21 (Story 4.8, Phase 2) -- `HeaderValue::set_sensitive(true)`, khong
        // con mot chuoi `format!` tran.** Mot `HeaderValue` KHONG danh dau nhay cam in nguyen
        // van gia tri cua no khi bi debug-print (vd. mot `{:?}` tren `RequestBuilder`/`Request`
        // trong mot lan go loi trong tuong lai) -- danh dau `sensitive` la dieu kien CAU TRUC
        // khien dieu do KHONG the xay ra, thay vi mot loi hua rang khong ai tung debug-print
        // request nay (§Always spec 4.8: "The API key never crosses IPC, never enters a log
        // line, an error message, an `IpcError` param, or a `Debug` output").
        // 🔵 SỬA 2026-09-22 (Story 4.10, Phase 1) -- biến thể riêng `ApiKeyHeaderInvalid`, không
        // còn dùng chung `RequestFailed`: thất bại ở đây là hình dạng khoá đã lưu, không phải
        // một sự cố mạng tạm thời -- xem doc-comment `OpenAiClientError::ApiKeyHeaderInvalid`.
        let mut auth_value = reqwest::header::HeaderValue::from_str(&format!("Bearer {}", request.api_key))
            .map_err(|e| OpenAiClientError::ApiKeyHeaderInvalid { detail: e.to_string() })?;
        auth_value.set_sensitive(true);

        let mut response = client
            .post(request.endpoint)
            .header(reqwest::header::AUTHORIZATION, auth_value)
            .header(reqwest::header::CONTENT_TYPE, "application/json")
            .header(reqwest::header::ACCEPT, "text/event-stream")
            .body(body_json)
            .send()
            .await
            .map_err(|e| OpenAiClientError::RequestFailed { detail: e.to_string() })?;

        if !response.status().is_success() {
            return Err(OpenAiClientError::NonSuccessStatus { status: response.status().as_u16() });
        }

        let mut buf: Vec<u8> = Vec::new();
        loop {
            // `should_cancel` duoc hoi GIUA moi khung nhan duoc -- ca truoc khi doc khung ke
            // tiep tu mang (o day) LAN truoc khi phat tung su kien da tach duoc trong khung do
            // (vong lap ben duoi) -- xem doc-comment `TranslationProvider::translate`.
            if should_cancel() {
                return Ok(TranslateOutcome::Cancelled);
            }

            let chunk = response
                .chunk()
                .await
                .map_err(|e| OpenAiClientError::ReadFailed { detail: e.to_string() })?;

            let Some(bytes) = chunk else {
                // Ket noi dong lai -- khong `[DONE]` nao tung thay (neu da thay, vong lap da
                // return o nhanh ben duoi TRUOC khi toi day). I/O Matrix spec 4.8 "Stream ends
                // without [DONE]".
                return Err(OpenAiClientError::StreamEndedWithoutDone);
            };

            buf.extend_from_slice(&bytes);
            let (events, remaining) = split_sse_frames(&buf);
            buf = remaining;
            enforce_sse_buffer_cap(&buf)?;

            for event in events {
                if should_cancel() {
                    return Ok(TranslateOutcome::Cancelled);
                }
                match interpret_sse_event(&event)? {
                    SseEventOutcome::Done => return Ok(TranslateOutcome::Done),
                    SseEventOutcome::Token(text) => on_token(&text),
                    SseEventOutcome::Ignore => {}
                }
            }
        }
    }
}

/// Từ chối `buf` một khi nó đã vượt [`MAX_SSE_BUFFER_BYTES`] mà vẫn chưa tách được sự kiện nào
/// — HÀM THUẦN, tách khỏi `translate()` đúng lý do [`build_request_body`] đã tách: một test đối
/// chứng được trần này mà không cần mở một kết nối mạng nào (AD-45).
pub fn enforce_sse_buffer_cap(buf: &[u8]) -> Result<(), OpenAiClientError> {
    if buf.len() > MAX_SSE_BUFFER_BYTES {
        Err(OpenAiClientError::BufferOverflow { size: buf.len() })
    } else {
        Ok(())
    }
}

/// Kết quả đọc MỘT payload `data:` đã tách bởi [`split_sse_frames`] — HÀM THUẦN, tách khỏi
/// `translate()` đúng lý do [`build_request_body`] đã tách: một test đối chứng được trực tiếp
/// mà không cần mở một kết nối mạng nào (AD-45).
///
/// 🔵 **SỬA (review) — một payload RỖNG (`"data:"`/`"data: "`, không mang gì thật) bị BỎ QUA,
/// không đi qua `parse_chunk_content`.** Một số nhà cung cấp gửi dòng này làm nhịp giữ kết nối
/// GIỮA các token thật; trước bản sửa này nó đi thẳng vào `serde_json::from_str("")`, luôn thất
/// bại (chuỗi rỗng không phải JSON hợp lệ), biến một nhịp giữ kết nối vô hại thành
/// `MalformedEvent` và huỷ NGANG một lượt gọi đã trả tiền. Một khung CHỈ-CHÚ-THÍCH (`": ping"`)
/// không đi tới đây bằng đường khác — nó không mang dòng `data:` nào nên
/// [`extract_data_payload`] đã lọc nó ra từ trước, `split_sse_frames` không bao giờ tạo một sự
/// kiện cho nó.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SseEventOutcome {
    /// Không có gì để làm — payload rỗng (hoặc toàn khoảng trắng), hoặc một chunk hợp lệ nhưng
    /// không mang `delta.content` (vd. chunk mở đầu chỉ mang `role`).
    Ignore,
    /// `"[DONE]"` — luồng kết thúc SẠCH.
    Done,
    /// Văn bản để gọi `on_token`.
    Token(String),
}

pub fn interpret_sse_event(event: &str) -> Result<SseEventOutcome, OpenAiClientError> {
    if event.trim().is_empty() {
        return Ok(SseEventOutcome::Ignore);
    }
    if event.trim() == "[DONE]" {
        return Ok(SseEventOutcome::Done);
    }
    match parse_chunk_content(event)? {
        Some(text) if !text.is_empty() => Ok(SseEventOutcome::Token(text)),
        _ => Ok(SseEventOutcome::Ignore),
    }
}

/// Phân tích MỘT khung `data:` (đã tách bởi [`split_sse_frames`], không mang `"[DONE]"`) thành
/// đoạn văn bản nó mang, nếu có — `None` khi chunk hợp lệ nhưng không mang `delta.content`
/// (vd. chunk mở đầu chỉ mang `role`).
fn parse_chunk_content(event: &str) -> Result<Option<String>, OpenAiClientError> {
    let chunk: ChatCompletionsChunk = serde_json::from_str(event)
        .map_err(|e| OpenAiClientError::MalformedEvent { detail: e.to_string() })?;
    Ok(chunk.choices.into_iter().next().and_then(|c| c.delta.content))
}

/// Tách MỌI sự kiện SSE TRỌN VẸN hiện có trong `buf`, theo đúng thứ tự tới — HÀM THUẦN, không
/// `reqwest`, không đồng hồ, không `Store` (xem doc-comment đầu tệp). Trả về payload `data:`
/// của mỗi sự kiện (nối nhiều dòng `data:` liên tiếp bằng `\n`, đúng đặc tả SSE) cộng phần dư
/// CHƯA đủ một sự kiện — chỗ gọi tích luỹ phần dư đó vào khối byte KẾ TIẾP rồi gọi lại hàm này
/// trên TOÀN BỘ bộ đệm đã gộp, đây là điều kiện để một ranh giới sự kiện bị CHIA làm hai khối
/// mạng vẫn được nhận ra đúng ở lượt gọi kế tiếp.
///
/// Một sự kiện kết thúc bằng một dòng TRẮNG — `"\n\n"` hoặc `"\r\n\r\n"`: SSE thường chỉ dùng
/// LF, nhưng một tầng trung gian viết lại xuống dòng thành CRLF không phải điều hàm này cần từ
/// chối đọc. Một khung không mang dòng `data:` nào (chỉ `event:`/`id:`/chú thích) bị bỏ qua —
/// không nhà cung cấp tương thích OpenAI nào cần tới chúng ở đây.
pub fn split_sse_frames(buf: &[u8]) -> (Vec<String>, Vec<u8>) {
    let mut events = Vec::new();
    let mut rest = buf;

    while let Some((frame_end, delim_len)) = find_event_boundary(rest) {
        let frame = &rest[..frame_end];
        if let Some(data) = extract_data_payload(frame) {
            events.push(data);
        }
        rest = &rest[frame_end + delim_len..];
    }

    (events, rest.to_vec())
}

/// Vị trí TƯƠNG ĐỐI (trong `buf`) của ranh giới sự kiện SỚM NHẤT, cộng độ dài dấu phân cách đã
/// khớp (2 cho `"\n\n"`, 4 cho `"\r\n\r\n"`) — `None` khi `buf` chưa mang đủ một sự kiện TRỌN
/// VẸN nào.
fn find_event_boundary(buf: &[u8]) -> Option<(usize, usize)> {
    let lf_lf = buf.windows(2).position(|w| w == b"\n\n");
    let crlf_crlf = buf.windows(4).position(|w| w == b"\r\n\r\n");
    match (lf_lf, crlf_crlf) {
        (Some(a), Some(b)) => {
            if b <= a {
                Some((b, 4))
            } else {
                Some((a, 2))
            }
        }
        (Some(a), None) => Some((a, 2)),
        (None, Some(b)) => Some((b, 4)),
        (None, None) => None,
    }
}

/// Nối mọi dòng `data:`/`data: ` bên trong MỘT khung sự kiện (không mang dấu phân cách kết
/// thúc) thành một chuỗi, đúng đặc tả SSE ("nhiều dòng `data:` nối bằng `\n`) — `None` khi
/// khung không mang dòng `data:` nào (heartbeat/comment/`event:` đơn độc).
fn extract_data_payload(frame: &[u8]) -> Option<String> {
    let text = String::from_utf8_lossy(frame);
    let mut data_lines: Vec<&str> = Vec::new();
    for raw_line in text.split('\n') {
        let line = raw_line.strip_suffix('\r').unwrap_or(raw_line);
        if let Some(value) = line.strip_prefix("data:") {
            data_lines.push(value.strip_prefix(' ').unwrap_or(value));
        }
    }
    if data_lines.is_empty() { None } else { Some(data_lines.join("\n")) }
}
