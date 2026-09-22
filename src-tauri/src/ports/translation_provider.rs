//! Cổng **thứ ba và cuối cùng** trong đúng ba của AD-2 — một nhà cung cấp AI dịch, nhìn qua
//! một trait streaming. Spec 4.8, Phase 1, Task 1.
//!
//! ─────────────────────────────────────────────────────────────────────────────
//! 🔴 CHỈ KHAI HÌNH DẠNG, KHÔNG MANG CÀI ĐẶT — cùng luật với [`super::DictionarySource`] và
//! [`super::ProjectStore`]
//! ─────────────────────────────────────────────────────────────────────────────
//! `tests/dict_boundary.rs::ports_declare_shape_and_never_open_anything` quét thư mục này:
//! không `rusqlite`, không `Connection::open`, không `fs::`, không `PathBuf` — ở **vị trí
//! mã**. Tệp này thêm luật riêng của nó, canh bởi `tests/webimport_boundary.rs` /
//! `tests/ai_boundary.rs`: không `reqwest` (điểm ra mạng là việc của **cài đặt**, không của
//! **hình dạng**), không `tauri::ipc::Channel` (AD-22 nói streaming đi qua MỘT Channel, nhưng
//! đó là chi tiết dây IPC của tầng lệnh — `commands::aitranslate`, Phase 2/3 — không phải một
//! điều cổng này cần biết để mà khai đúng hình dạng của nó). Cài đặt thật (dựng request
//! tương thích OpenAI, lái `reqwest` async, tách khung SSE) sống ở `core::ai::client`
//! (Phase 2), không ở đây.
//!
//! ─────────────────────────────────────────────────────────────────────────────
//! ⚠️ VÌ SAO CỔNG NÀY CHƯA CÓ CÀI ĐẶT NÀO Ở PHASE 1 — cùng hoàn cảnh `ProjectStore` tại
//! Story 1.15
//! ─────────────────────────────────────────────────────────────────────────────
//! `ports/project_store.rs` tự ghi: *"Đây là cùng hoàn cảnh `TranslationProvider` (Epic 4)
//! đang ở: khai trước, cắm cài đặt sau, khi có consumer thật."* Spec 4.8 giữ đúng lời đó:
//! Phase 1 chỉ khai trait; `core/ai/client.rs` (cài đặt) và `commands/aitranslate.rs` (chỗ
//! gọi thật) là việc của Phase 2. Viết cài đặt trước cổng sẽ đảo ngược đúng thứ tự phụ thuộc
//! mà cổng này tồn tại để diễn đạt (§Task 1 rationale, spec 4.8).
//!
//! ─────────────────────────────────────────────────────────────────────────────
//! VÌ SAO "HUỶ" LÀ MỘT THAM SỐ CỦA [`TranslationProvider::translate`], KHÔNG PHẢI MỘT
//! METHOD RIÊNG
//! ─────────────────────────────────────────────────────────────────────────────
//! AD-22 đòi *"mọi lời gọi AI huỷ được giữa chừng"*, và Quyết định 1 của spec 4.8 xác nhận
//! huỷ dựng ở Phase 2/3 của CHÍNH story này, không dời sang 4.9. Cơ chế đã có tiền lệ trong
//! kho — `commands::project::ImportScanGeneration` (`next()` + `is_current(generation)`) —
//! là một CỜ CHIA SẺ mà vòng lặp streaming tự hỏi lại GIỮA MỖI khung, không phải một lệnh gọi
//! đồng thời vào MỘT `&self` đang bận chạy `translate()` (Rust không cho hai lời gọi mượn
//! `&self` chồng chéo thực thi song song trên một luồng dữ liệu đơn); `&self` ở đây thậm chí
//! không cần `&mut`. `should_cancel` mô hình hoá đúng cơ chế đó ở tầng CỔNG: cài đặt hỏi nó
//! sau mỗi khung nhận được, và khi nó trả `true`, dừng gửi NGAY — không khung nào rời sau
//! thời điểm đó, đúng "Đối chứng cancel bằng phép ĐO, không bằng cờ" ở §Verification.
//!
//! ─────────────────────────────────────────────────────────────────────────────
//! VÌ SAO TranslationProvider::translate LÀ async fn
//! ─────────────────────────────────────────────────────────────────────────────
//! Cài đặt (Phase 2) lái reqwest ASYNC — §Code Map spec 4.8: "the async client, not
//! the blocking one: read_timeout exists only on the async builder". Một method ĐỒNG BỘ ở
//! đây buộc cài đặt phải block_on một runtime đang chạy TỪ BÊN TRONG một tác vụ tokio đã
//! spawn sẵn (#[tauri::command(async)] đã đẩy thân lệnh vào async_runtime::spawn,
//! src-tauri/AGENTS.md) — chặn một luồng-thợ của chính pool đang phục vụ nó, đúng lớp rủi
//! ro Tokio khuyến cáo tránh. async fn trong trait là mã nguồn ỔN ĐỊNH từ Rust 1.75 (crate
//! này ở rust-version 1.85), không cần thêm crate async-trait (§Never spec 4.8: "no new
//! crate"). Không cần dyn TranslationProvider — cài đặt hôm nay đúng MỘT, nên phân phối
//! TĨNH (generic/impl Trait) đủ dùng và tránh luôn câu hỏi "object safety" của async fn
//! trong trait.
//!
//! ⚠️ `cargo check` phát `warning: async_fn_in_trait` (thiếu bound Send tự động) — biết và
//! chấp nhận ở Phase 1: kho này không chạy clippy như một cổng (`check:lint` là eslint cho
//! frontend, không phải Rust), nên cảnh báo không chặn build. Nếu Phase 2's wire shell cần
//! `Send` tường minh trên Future trả về (vd. để spawn RIÊNG thay vì await NGAY trong khối
//! async đã có sẵn của `#[tauri::command(async)]`), desugar sang `-> impl Future<Output = ...>
//! + Send` là việc của Phase 2, khi có chỗ gọi thật để đo có cần hay không.

/// Một lượt gọi dịch đã sẵn sàng gửi — đúng những trường cổng cần, không hơn.
///
/// 🔴 `prompt` là chuỗi 4.7's producer **ĐÃ GHI**, nguyên văn (AD-14, §Always spec 4.8: *"never
/// a second assembly, never a concatenation at the call site"*) — cài đặt của cổng này KHÔNG
/// được lắp ráp lại hay nối thêm bất cứ gì vào `prompt` trước khi đặt nó vào thân request HTTP;
/// nó chỉ được truyền tiếp NGUYÊN VĂN.
///
/// ⚠️ `api_key` là chuỗi **đã lộ** (`&str`), không phải `ApiKeySecret` — quyết định có chủ ý:
/// giữ cổng này không cần biết `core::aiconfig` tồn tại (cùng luật "chỉ tham chiếu core khi
/// cổng THẬT SỰ cần kiểu bản ghi của core đó", [`super::DictionarySource`] tham chiếu
/// `core::dict`, [`super::ProjectStore`] tham chiếu `core::library` — nhưng cổng này không
/// cần biết HÌNH DẠNG của bí mật khoá, chỉ cần MỘT chuỗi để đặt vào header `Authorization`).
/// Chỗ gọi (Phase 2: `commands::aitranslate`) chịu trách nhiệm gọi
/// `ApiKeySecret::expose_secret()` **ngay trước** khi dựng giá trị này, và KHÔNG log, KHÔNG
/// giữ nó lâu hơn cần thiết, KHÔNG đặt nó vào bất kỳ kiểu nào có `derive(Debug)` (§Always spec
/// 4.8: *"The API key never crosses IPC, never enters a log line, an error message, an
/// `IpcError` param, or a `Debug` output"*).
pub struct TranslateRequest<'a> {
    /// URL đầy đủ của endpoint tương thích OpenAI — đã qua `aiconfig::validate_field`.
    pub endpoint: &'a str,
    /// Tên mô hình — đã qua `aiconfig::validate_field`.
    pub model: &'a str,
    /// `temperature` đã xác thực, `[0.0, 2.0]` (`core::aiconfig::validate_field`). `None` =
    /// người dùng CHƯA đặt — Quyết định 6 spec 4.8: cài đặt BỎ HẲN trường này khỏi request,
    /// không đúc một giá trị thay người dùng.
    pub temperature: Option<f64>,
    /// `max_tokens` đã xác thực, số nguyên dương (`core::aiconfig::validate_field`). `None` =
    /// chưa đặt — xem [`Self::temperature`].
    pub max_tokens: Option<u32>,
    /// Khoá API đã lộ — xem cảnh báo ở doc-comment của kiểu này.
    pub api_key: &'a str,
    /// Prompt 4.7's producer đã ghi — nguyên văn, không lắp lại (AD-14).
    pub prompt: &'a str,
}

/// Số liệu sử dụng nhà cung cấp trả về cho MỘT lượt gọi, cộng ước tính chi phí đã tính sẵn —
/// Story 4.11 (spec 4.11). Khai ở TẦNG CỔNG, không ở `core::ai::client`: cài đặt
/// (`OpenAiChatClient`) đọc trường `usage` của chunk cuối SSE rồi gọi `core::ai::pricing`
/// (module KHÁC, nhưng cùng nằm trong ranh giới `core/ai/`, nên không phạm AD-13) để đúc
/// `cost_usd` trước khi trả giá trị này ra — khai kiểu này ở `core::ai::client` sẽ buộc
/// `commands/aitranslate.rs` gọi tên THỨ BA của module đó, vượt quá
/// `ai_boundary.rs::ALLOWED_AI_CLIENT_NAMES_IN_COMMAND_SEAM: [&str; 2]` (spec 4.11 §Code Map:
/// *"its length is in the type"*) — cổng chỉ khai HÌNH DẠNG (đúng luật đầu tệp), không mang
/// hành vi tính giá.
///
/// `cost_usd` là `None` khi `model_id` (đã dùng để gọi) không có hàng trong bảng giá — mô
/// hình cục bộ (Ollama/LM Studio), hoặc bất kỳ id nào bảng giá chưa được dạy. Đây LÀ quy tắc
/// "mô hình cục bộ" của Quyết định Ice 2026-09-22: ứng dụng không có bộ phân biệt cục bộ/đám
/// mây (`provider` là chuỗi tự do), nên vắng mặt khỏi bảng là tín hiệu DUY NHẤT.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TranslateUsage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
    pub cost_usd: Option<f64>,
}

/// Vì sao một lượt dịch streaming DỪNG — không phải MỌI lượt dừng đều là một lỗi, và cổng
/// này phân biệt hai lượt dừng SẠCH khỏi một `Err` (một lượt dừng KHÔNG sạch, vd. kết nối rớt
/// giữa chừng hay một mã trạng thái non-2xx — I/O Matrix spec 4.8's "Stream ends without
/// `[DONE]`"/"Provider returns a non-2xx").
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TranslateOutcome {
    /// Provider gửi khung kết thúc hợp lệ (`[DONE]`, hoặc kết nối đóng sau khi đã nhận đủ).
    /// Mang [`TranslateUsage`] khi VÀ CHỈ KHI provider thật sự gửi một khung `usage` trước khi
    /// kết thúc (Story 4.11) — `None` là trạng thái "nhà cung cấp không trả về số liệu", không
    /// một `0` giả.
    Done(Option<TranslateUsage>),
    /// `should_cancel` trả `true` giữa chừng — không khung nào gửi thêm sau thời điểm đó
    /// (I/O Matrix spec 4.8's "Cancel mid-stream": *"received tokens stay visible, no further
    /// Channel message arrives"*).
    Cancelled,
}

/// Một nhà cung cấp AI dịch, nhìn qua cổng — Epic 4, cổng thứ BA và CUỐI của AD-2
/// ([`super`]'s bảng).
///
/// 🔴 Đơn vị là **một lượt gọi dịch streaming**, không phải "một cách nói HTTP" — cùng tinh
/// thần AD-44 ⑤ ([`super::DictionarySource`]) và cách [`super::ProjectStore`] không lộ số
/// tệp của `.atproj/`: người gọi cổng này không cần biết provider nói SSE tương thích OpenAI
/// hay một hình dạng khác, chỉ cần biết prompt nào, cho segment nào, và nhận token về khi nó
/// tới.
pub trait TranslationProvider {
    /// Kiểu lỗi CỤ THỂ do cài đặt chọn — cùng lý lẽ [`super::DictionarySource::lookup`] trả
    /// `StoreError` **của nó**, không phải một kiểu lỗi tổng quát mượn từ nơi khác: hình dạng
    /// lỗi (mạng rớt, HTTP non-2xx, keychain từ chối trả lời, …) là chi tiết **cài đặt**, và
    /// cổng chỉ đòi nó là [`std::error::Error`] để tầng lệnh đóng gói được thành `IpcError`
    /// (AD-21) mà không cần biết cấu trúc bên trong.
    type Error: std::error::Error;

    /// Gửi `request.prompt` NGUYÊN VĂN tới provider và gọi `on_token` một lần cho **mỗi** đoạn
    /// văn bản khi nó tới — không gộp nhiều khung thành một lần gọi, không trễ tới khi cả
    /// response về xong (I/O Matrix spec 4.8: *"tokens arrive on the Channel and render as they
    /// land"*).
    ///
    /// `should_cancel` được hỏi lại **giữa mỗi khung nhận được** — không phải một lần trước khi
    /// bắt đầu. Khi nó trả `true`, việc gửi dừng **NGAY**, KHÔNG khung nào gửi thêm, và hàm trả
    /// `Ok(TranslateOutcome::Cancelled)` — không phải một `Err`, vì huỷ là một quyết định của
    /// người dùng, không phải một thất bại (§Always spec 4.8: *"Every call is cancellable
    /// mid-flight, and a cancelled call sends nothing further"*).
    ///
    /// KHÔNG tự động thử lại ở bất kỳ tầng nào (AD-22, §Never spec 4.8: *"No self-reconnecting
    /// SSE client and no automatic retry of any kind, at any layer"*) — một kết nối rớt hay một
    /// mã trạng thái non-2xx là một `Err`, không bao giờ một lượt gọi lại tự động; token đã
    /// nhận được ở lại trên màn hình, đó là việc của chỗ gọi (Phase 2/3), không phải của cổng.
    async fn translate(
        &self,
        request: TranslateRequest<'_>,
        on_token: &mut dyn FnMut(&str),
        should_cancel: &dyn Fn() -> bool,
    ) -> Result<TranslateOutcome, Self::Error>;
}
