//! Bề mặt IPC dịch MỘT segment với kết quả chảy dần — Story 4.8, Phase 2 (FR72/FR74, AD-22).
//!
//! Cùng khuôn `commands::aiprompt`: hàm thuần nhận `Option<&Store>`/`Option<&OpenWork>` trước,
//! `#[tauri::command]` chỉ là vỏ mỏng trong [`wire`]. Đây là seam THỨ BA `tests/ai_boundary.rs`
//! đã dựng khung ở Phase 1 (`AI_TRANSLATE_SEAM_COMMAND_FILE`/`_MARKER`, admits
//! `crate::core::ai::client::` theo DÒNG, đúng tệp này) — control ② (đóng băng TÊN được phép)
//! là việc của một agent SAU, không phải Phase 2.
//!
//! ─────────────────────────────────────────────────────────────────────────────
//! 🔴 KHOÁ API ĐƯỢC ĐỌC Ở ĐÂY, KHÔNG Ở `core/ai/client.rs` — AD-13
//! ─────────────────────────────────────────────────────────────────────────────
//! `tests/aiconfig_keychain_boundary.rs::AI_TRANSLATE_KEYCHAIN_CALLER_FILE` miễn trừ ĐÚNG tệp
//! này khỏi cổng raw-value-accessor. `keychain::read()` trả về, `expose_secret()` được gọi
//! đúng MỘT lần trong [`prepare_translate_call`], giá trị lộ ra đi thẳng vào
//! [`PreparedTranslateCall::api_key`] — một `String` sở hữu, không `derive(Debug)` trên bất kỳ
//! kiểu nào bọc nó (xem doc-comment [`PreparedTranslateCall`]).
//!
//! ─────────────────────────────────────────────────────────────────────────────
//! 🔴 KHÔNG GIỮ `MutexGuard` CỦA `OpenWorkState` XUYÊN QUA LƯỢT CHỜ MẠNG
//! ─────────────────────────────────────────────────────────────────────────────
//! Hai lớp, không phải một: [`prepare_translate_call`] là hàm ĐỒNG BỘ, mượn `Option<&Store>`/
//! `Option<&OpenWork>` đúng trong lúc nó chạy rồi TRẢ VỀ dữ liệu SỞ HỮU
//! ([`PreparedTranslateCall`], không còn vay mượn gì); [`send_prepared_translate_call`] là hàm
//! ASYNC, không nhận `Store`/`OpenWork` — nó chỉ thấy dữ liệu đã sở hữu. `mod wire` khoá
//! `OpenWorkState` đúng trong lúc gọi lớp đồng bộ rồi THẢ khoá đó TRƯỚC khi `.await` lớp async.
//! Giữ khoá xuyên lượt mạng (có thể dài hàng chục giây) sẽ chặn MỌI lệnh Chương/segment khác
//! của tiến trình — đúng lớp lỗi `config_invariants.rs` đã ghi nợ có chủ cho năm vỏ CHẶN khác
//! ("vẫn giữ `MutexGuard` của `OpenWorkState` xuyên suốt"); ở đây nó không phải nợ, nó được
//! THIẾT KẾ để không xảy ra.
//!
//! ─────────────────────────────────────────────────────────────────────────────
//! ⚠️ VÌ SAO `send_prepared_translate_call` TỰ SPAWN MỘT LUỒNG CHẶN, KHÔNG `.await` THẲNG
//! ─────────────────────────────────────────────────────────────────────────────
//! `ports/translation_provider.rs`'s doc-comment (Phase 1) để ngỏ đúng câu hỏi này cho Phase 2
//! đo: `on_token: &mut dyn FnMut(&str)`/`should_cancel: &dyn Fn() -> bool` là hai tham chiếu tới
//! `dyn Trait` KHÔNG mang cận `+ Send` — chữ ký cổng đã đóng băng, không phải việc của Phase 2
//! sửa. Hệ quả đo được: `Future` mà `OpenAiChatClient::translate` trả về KHÔNG `Send` (nó giữ
//! hai tham chiếu đó xuyên các điểm `.await` bên trong vòng lặp đọc khung SSE). `mod wire`'s
//! `ai_translate_segment` là một `async fn` LITERAL — cơ chế đầu tiên của loại này trong kho
//! (§Code Map spec 4.8: *"There is no literal `async fn` command in the tree today"*) — và
//! macro lệnh gửi nó qua `respond_async_serialized → async_runtime::spawn → tokio::spawn`
//! (`tauri-macros-2.6.3/src/command/wrapper.rs`), đòi `Future: Send`. `.await` thẳng một
//! `Future` không `Send` ngay trong thân `ai_translate_segment` không biên dịch được.
//!
//! Lối thoát ĐÚNG, không phải né tránh: [`send_prepared_translate_call`] gọi
//! `tauri::async_runtime::spawn_blocking` — một luồng RIÊNG của hồ bơi blocking, KHÔNG phải một
//! tác vụ đang được tokio POLL — rồi bên TRONG luồng đó (không còn ở "trong một runtime" theo
//! nghĩa tokio cấm `block_on` lồng) gọi `tauri::async_runtime::handle().block_on(...)` để chạy
//! trọn `provider.translate(...)`. Chỉ dữ liệu `Send` (chuỗi sở hữu, `Channel` — tự `Clone` để
//! di chuyển vào tác vụ spawn theo đúng thiết kế của nó, `AiTranslateGeneration` — `Arc<AtomicU64>`)
//! băng qua ranh giới `spawn_blocking`; `on_token`/`should_cancel` dựng BÊN TRONG luồng đó, không
//! bao giờ băng qua một điểm `.await` nào cần `Send`. `JoinHandle` mà `spawn_blocking` trả về
//! CÓ `Send` (nó chỉ mang `Result<TranslateOutcome, OpenAiClientError>`, cả hai đều `Send`), nên
//! `.await` nó ngay trong `ai_translate_segment` là hợp lệ.

use crate::commands::aiprompt::{
    AssembledPromptWire, LastAssembledPromptState, assemble_and_record_prompt, segment_not_in_chapter,
};
use crate::commands::project::OpenWork;
use crate::core::aiconfig::{AiConfigField, resolve_two_tiers};
use crate::core::ai::client::{OpenAiChatClient, OpenAiClientError};
use crate::core::i18n::{IpcError, MessageKey};
use crate::core::store::{ReadHandle, Store, StoreError, StoreKind};
use crate::ports::translation_provider::{TranslateOutcome, TranslateRequest, TranslationProvider};

/// Kho `global.db` vắng mặt ⇒ lỗi *mở kho* — cùng khuôn `commands::aiprompt::store_is_missing`.
fn store_is_missing() -> IpcError {
    StoreError::OpenFailed {
        store: StoreKind::Global,
        detail: "the global store was never managed; see lib.rs::open_global_store".to_owned(),
    }
    .into()
}

/// `segment.is_omitted == true` ⇒ từ chối TRƯỚC khi request được dựng (§Always spec 4.8: "never
/// sent to a provider, checked before the request is built").
fn segment_is_omitted(segment_id: i64) -> IpcError {
    let mut params = std::collections::BTreeMap::new();
    params.insert("segment_id".to_owned(), segment_id.to_string());
    IpcError::new(
        "ai_translate.segment_omitted",
        MessageKey::AiTranslateSegmentOmitted,
        params,
        false,
    )
}

/// Keychain từ chối trả lời lúc dịch — tái dùng ĐÚNG khoá của Story 4.3 (§Code Map spec 4.8:
/// "reuses ... `AiConfigKeychainUnavailable` where the fact already has a key"), không một khoá
/// thứ hai cho cùng sự thật.
fn keychain_unavailable() -> IpcError {
    crate::core::aiconfig::AiConfigKeyError::Unavailable.into()
}

/// Provider trả lỗi mạng/HTTP/khung SSE hỏng — NHÃN duy nhất qua IPC cho toàn bộ họ lỗi của
/// `OpenAiClientError` (§Never spec 4.8: "no error-copy catalogue ... this story produces the
/// error state and an `IpcError`-shaped failure", Story 4.10 sở hữu văn bản riêng cho từng
/// nguyên nhân). `retryable` phân theo đúng hai hàng ma trận đã ghi tường minh (non-2xx ⇒
/// không thử lại; rớt kết nối trước khi thấy `[DONE]` ⇒ có thể thử lại) — bốn nhánh còn lại
/// (dựng client, gửi thất bại, đọc thân lỗi giữa chừng, khung JSON hỏng) chưa có hàng ma trận
/// riêng; xếp CHẶT theo "thất bại tạm thời, đáng thử lại" — Story 4.10 xét lại khi có chỗ đo.
impl From<OpenAiClientError> for IpcError {
    fn from(err: OpenAiClientError) -> Self {
        let mut params = std::collections::BTreeMap::new();
        let retryable = match &err {
            OpenAiClientError::NonSuccessStatus { status } => {
                params.insert("status".to_owned(), status.to_string());
                false
            }
            OpenAiClientError::StreamEndedWithoutDone
            | OpenAiClientError::RequestFailed { .. }
            | OpenAiClientError::ReadFailed { .. } => true,
            OpenAiClientError::ClientBuildFailed { .. }
            | OpenAiClientError::MalformedEvent { .. }
            | OpenAiClientError::BufferOverflow { .. } => false,
        };
        IpcError::new(
            "ai_translate.provider_call_failed",
            MessageKey::AiTranslateProviderCallFailed,
            params,
            retryable,
        )
    }
}

/// Trạng thái dựng ĐỦ để gửi — mọi trường đã SỞ HỮU (không vay `Store`/`OpenWork`), đúng điều
/// kiện để [`send_prepared_translate_call`] `.await` mạng mà không giữ khoá nào của tiến trình.
///
/// ⚠️ **KHÔNG `derive(Debug)`** — `api_key` là khoá API đã lộ (§Always spec 4.8: "never enters
/// a log line ... or a `Debug` output"), cùng luật `TranslateRequest`/`ApiKeySecret`. Các
/// trường `pub` để `tests/**` (Phase 4, một crate riêng) đối chứng được AC1 ("the request
/// carries the string `assemble_and_record_prompt` recorded, byte for byte") mà không cần một
/// cổng dò riêng — visibility trường KHÔNG phải hàng rào của luật cấm Debug, `derive` mới là.
pub struct PreparedTranslateCall {
    pub endpoint: String,
    pub model: String,
    pub temperature: Option<f64>,
    pub max_tokens: Option<u32>,
    pub api_key: String,
    pub prompt: String,
}

/// Kết quả [`prepare_translate_call`] — "chưa cấu hình" là một TRẠNG THÁI (I/O Matrix spec 4.8:
/// "Not an error — no `IpcError`, no alert"), không một nhánh của `Result::Err`.
pub enum PrepareOutcome {
    NotConfigured,
    Ready(PreparedTranslateCall),
}

/// Đọc SỐ CÒN LẠI của MỘT trường `AiConfigField` đã phân giải hai tầng — chuỗi rỗng khi trường
/// chưa được cấu hình ở tầng nào (cùng khuôn `AiConfigFieldWire::from_resolved`'s nhánh `None`).
fn resolved_field_value(
    resolved: &std::collections::BTreeMap<String, crate::core::aiconfig::ResolvedField>,
    field: AiConfigField,
) -> String {
    resolved.get(field.as_str()).map(|f| f.value.clone()).unwrap_or_default()
}

/// **Lớp ĐỒNG BỘ** — phân giải cấu hình, đọc khoá, từ chối một segment `is_omitted`, rồi gọi
/// 4.7's producer ([`assemble_and_record_prompt`]) để lấy ĐÚNG chuỗi nó đã ghi (AD-14, §Always
/// spec 4.8: "never a second assembly"). **Hàm thuần, đây là thứ test gọi** — không mạng, không
/// `Channel`, không `.await`.
///
/// # Thứ tự kiểm
/// Tác phẩm đang mở, TRƯỚC cấu hình (một Tác phẩm khác có thể ghi đè tầng Work): cấu hình
/// (`endpoint`/`model` rỗng ⇒ [`PrepareOutcome::NotConfigured`]), rồi `is_omitted` (đọc CHÍNH
/// hàng mà `assemble_and_record_prompt` sắp đọc lại — chi phí một lượt đọc Chương thêm, không
/// một truy vấn mới, xem §Code Map spec 4.8) — **TRƯỚC khoá API có chủ ý** (🔵 rà soát: một
/// câu đã cắt khỏi bản dịch không đáng một lượt chạm keychain hệ điều hành thật), rồi mới khoá
/// API (`keychain::read() == Ok(None)` ⇒ cùng trạng thái [`PrepareOutcome::NotConfigured`] —
/// I/O Matrix "or `key_configured == Some(false)`"; `Err(KeychainUnavailable)` ⇒
/// `ai_config.keychain_unavailable`), rồi mới gọi 4.7's producer — nó tự kiểm lại "bộ prompt
/// hiệu lực"/"segment thuộc Chương" và trả đúng lỗi của chính nó nếu trượt
/// (`ai_prompt.no_set_selected`/`ai_prompt.segment_not_in_chapter`).
///
/// # Lỗi
/// - `global.db` vắng mặt ⇒ `store.open_failed`;
/// - chưa có Tác phẩm nào đang mở ⇒ `work.none_open`;
/// - `segment_id` không có trong Chương đang mở ⇒ `ai_prompt.segment_not_in_chapter`;
/// - segment mang `is_omitted == true` ⇒ `ai_translate.segment_omitted`;
/// - keychain từ chối trả lời ⇒ `ai_config.keychain_unavailable` (retryable);
/// - đường đọc Store/Glossary trượt lúc lắp prompt ⇒ lỗi truyền từ [`assemble_and_record_prompt`].
pub fn prepare_translate_call(
    global: Option<&Store>,
    open: Option<&OpenWork>,
    record: &LastAssembledPromptState,
    prompt_set_name: Option<&str>,
    segment_id: i64,
) -> Result<PrepareOutcome, IpcError> {
    let global_store = global.ok_or_else(store_is_missing)?;
    let open_work = open.ok_or_else(crate::commands::chapter::no_work_open)?;

    let resolver = open_work.scope.clone();
    let resolved = resolve_two_tiers(&resolver, global_store, Some(&open_work.store))?;

    let endpoint = resolved_field_value(&resolved, AiConfigField::Endpoint);
    let model = resolved_field_value(&resolved, AiConfigField::Model);
    if endpoint.trim().is_empty() || model.trim().is_empty() {
        return Ok(PrepareOutcome::NotConfigured);
    }

    // 🔵 QUYET DINH 6 (Ice, 2026-09-21) -- mot `temperature`/`max_tokens` CHUA DAT KHONG phai
    // ly do goi nha cung cap la "chua cau hinh". Ma tran I/O spec 4.8 neu ten dung
    // `endpoint`/`model`/khoa cho hang do va khong gi khac; `aiConfigState.ts:69` khoi tao ca
    // hai truong la chuoi rong va bieu mau khong danh dau chung bat buoc, nen ban truoc cua
    // doan nay khoa nguoi dung ngoai cua vinh vien ma khong noi thieu gi. Chua dat ⇒ `None` ⇒
    // `core::ai::client` BO HAN truong do khoi JSON (`skip_serializing_if`), va endpoint tuong
    // thich OpenAI tu ap mac dinh cua no. Mot gia tri CO MAT nhung HONG cung cho `None`: no da
    // qua `validate_field` luc ghi, nen khong phan tich duoc nghia la khong con gi de gui.
    let temperature: Option<f64> =
        resolved_field_value(&resolved, AiConfigField::Temperature).trim().parse().ok();
    let max_tokens: Option<u32> =
        match resolved_field_value(&resolved, AiConfigField::MaxTokens).trim().parse() {
            Ok(0) | Err(_) => None,
            Ok(v) => Some(v),
        };

    // `is_omitted` -- hàng đã có sẵn trong tay qua chính lượt đọc Chương này (§Code Map spec
    // 4.8: "the row is already in hand, so the guard costs one field access, not a query").
    // 🔵 SUA (rà soát) -- doc VA tu choi segment nay TRUOC lượt gọi keychain ngay dưới: một câu
    // đã bị cắt khỏi bản dịch không đáng một lượt chạm keychain hệ điều hành thật, dù lượt đó
    // chỉ đọc chứ không ghi gì.
    let chapter = crate::commands::segment::read_open_chapter_segments(Some(open_work))?;
    let Some(row) = chapter.segments.iter().find(|s| s.id == segment_id) else {
        return Err(segment_not_in_chapter(segment_id, chapter.chapter_id));
    };
    if row.is_omitted {
        return Err(segment_is_omitted(segment_id));
    }

    let api_key = match crate::core::aiconfig::keychain::read() {
        Ok(Some(secret)) => secret.expose_secret().to_owned(),
        Ok(None) => return Ok(PrepareOutcome::NotConfigured),
        Err(_unavailable) => return Err(keychain_unavailable()),
    };

    let assembled: AssembledPromptWire = assemble_and_record_prompt(
        Some(global_store),
        Some(open_work),
        record,
        prompt_set_name,
        segment_id,
    )?;

    Ok(PrepareOutcome::Ready(PreparedTranslateCall {
        endpoint,
        model,
        temperature,
        max_tokens,
        api_key,
        prompt: assembled.prompt,
    }))
}

/// Bộ đếm thế hệ dùng cho huỷ giữa chừng — đúng hình dạng
/// `commands::project::ImportScanGeneration` (`next()`/`is_current()`), một `Arc<AtomicU64>`
/// RIÊNG cho đường dịch AI (Quyết định 1, spec 4.8: cancel dựng ở CHÍNH story này). `Clone` chỉ
/// nhân bản `Arc` — vòng lặp đọc khung SSE và lệnh huỷ đọc/ghi CÙNG một bộ đếm.
#[derive(Debug, Clone, Default)]
pub struct AiTranslateGeneration(std::sync::Arc<std::sync::atomic::AtomicU64>);

impl AiTranslateGeneration {
    fn next(&self) -> u64 {
        self.0.fetch_add(1, std::sync::atomic::Ordering::AcqRel).wrapping_add(1)
    }

    fn is_current(&self, generation: u64) -> bool {
        self.0.load(std::sync::atomic::Ordering::Acquire) == generation
    }
}

/// **Hàm THUẦN-ASYNC, seam mà `tests/**` (Phase 4) thay THẬT một `TranslationProvider` giả vào**
/// (§Never spec 4.8: "The port trait is the seam tests substitute at") — generic trên `P`, gọi
/// `provider.translate(...)` NGUYÊN VĂN, không `Store`, không `spawn_blocking`, không
/// `AppHandle`. `channel` chỉ cần `Channel::new(closure)` (§Code Map spec 4.8: "no `Runtime`,
/// no `AppHandle`"), nên một test gọi thẳng hàm này qua `tauri::async_runtime::block_on(...)`
/// ở tầng NGOÀI CÙNG (không lồng trong một runtime khác) mà không cần dựng cổng nghe nào (AD-45).
pub async fn run_translate_call<P: TranslationProvider>(
    provider: &P,
    prepared: &PreparedTranslateCall,
    channel: &tauri::ipc::Channel<String>,
    should_cancel: &dyn Fn() -> bool,
) -> Result<TranslateOutcome, P::Error> {
    let request = TranslateRequest {
        endpoint: &prepared.endpoint,
        model: &prepared.model,
        temperature: prepared.temperature,
        max_tokens: prepared.max_tokens,
        api_key: &prepared.api_key,
        prompt: &prepared.prompt,
    };
    let mut on_token = |text: &str| {
        // Kenh mat nguoi nhan (webview dong)/loi tam thoi -- token da nhan TRUOC do van o lai
        // tren man hinh (§Always spec 4.8), khong co gi de lam voi mot loi gui THEM.
        let _ = channel.send(text.to_owned());
    };
    provider.translate(request, &mut on_token, should_cancel).await
}

/// **Cầu nối riêng của `mod wire`** — cài đặt DUY NHẤT hôm nay (`OpenAiChatClient`, đóng cứng,
/// đúng lời cổng doc-comment "không `dyn TranslationProvider`, phân phối TĨNH"), chạy
/// [`run_translate_call`] BÊN TRONG một luồng của hồ bơi blocking (xem doc-comment đầu tệp
/// §"vì sao tự spawn một luồng chặn" cho lý do `Send` đầy đủ). KHÔNG phải seam test thay được —
/// đó là [`run_translate_call`] ở trên; hàm này chỉ tồn tại để giải bài toán `Future: Send` mà
/// riêng CƠ CHẾ điều phối lệnh của Tauri đòi.
async fn send_prepared_translate_call(
    prepared: PreparedTranslateCall,
    channel: tauri::ipc::Channel<String>,
    generation_state: AiTranslateGeneration,
    generation: u64,
) -> Result<TranslateOutcome, OpenAiClientError> {
    let join = tauri::async_runtime::spawn_blocking(move || {
        let provider = OpenAiChatClient::new();
        let should_cancel = || !generation_state.is_current(generation);

        tauri::async_runtime::handle().block_on(run_translate_call(
            &provider,
            &prepared,
            &channel,
            &should_cancel,
        ))
    });

    match join.await {
        Ok(outcome) => outcome,
        Err(_join_err) => Err(OpenAiClientError::RequestFailed {
            detail: "ai_translate blocking task panicked or was aborted".to_owned(),
        }),
    }
}

/// Ngày giờ UTC, ISO-8601, lấy qua `strftime` của CHÍNH SQLite -- cùng kỷ luật mọi cột
/// `created_at`/`updated_at` của kho (root `AGENTS.md`'s Consistency Conventions: "lưu ISO-8601
/// UTC ... không lấy từ đồng hồ Rust"). `None` khi `global.db` không đọc được -- `sent_at` là
/// một sự thật CHẨN ĐOÁN cho prompt inspector (Story 4.7), không một điều kiện của bất kỳ AC
/// nào; bỏ qua an toàn hơn là làm cả lượt gửi đã THÀNH CÔNG báo lỗi vì một lượt đọc phụ trượt.
fn now_utc_iso8601(global: &Store) -> Option<String> {
    global
        .read(|conn: ReadHandle<'_>| {
            conn.query_row("SELECT strftime('%Y-%m-%dT%H:%M:%fZ', 'now')", [], |row| {
                row.get::<_, String>(0)
            })
        })
        .ok()
}

/// Kết quả cuối của MỘT lượt gọi [`wire::ai_translate_segment`] — BA trong năm giá trị trạng
/// thái toàn epic (§Always spec 4.8): `generating` không có mặt ở đây (nó là trạng thái webview
/// tự giữ TRONG LÚC lời gọi này chưa trả về — Phase 3), và `error` đi qua `Result::Err(IpcError)`
/// của chính lệnh này, không qua biến thể nào ở đây.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(tag = "state", rename_all = "snake_case")]
pub enum AiTranslateOutcomeWire {
    /// `endpoint`/`model` rỗng, hoặc chưa có khoá API nào lưu — KHÔNG một lượt gọi mạng nào đã
    /// chạy (I/O Matrix spec 4.8).
    NotConfigured,
    /// Provider gửi khung kết thúc hợp lệ.
    Done,
    /// Huỷ giữa chừng — token đã nhận ở lại trên màn hình webview, không khung nào gửi thêm.
    Cancelled,
}

/// Hai vỏ `#[tauri::command]`. **Không một quy tắc nào sống ở đây** ngoài việc mở/thả khoá
/// đúng lúc (xem doc-comment đầu tệp).
pub mod wire {
    use super::{
        AiTranslateGeneration, AiTranslateOutcomeWire, PrepareOutcome, prepare_translate_call,
        send_prepared_translate_call,
    };
    use crate::commands::aiprompt::{LastAssembledPromptState, mark_prompt_as_sent};
    use crate::commands::project::OpenWorkState;
    use crate::core::i18n::IpcError;
    use crate::core::store::Store;
    use crate::ports::translation_provider::TranslateOutcome;

    /// Vỏ IPC của lượt dịch một segment — `async fn` LITERAL (xem doc-comment đầu tệp cho lý
    /// do và cơ chế). `try_state`, không `state()` — cùng lý do mọi vỏ khác của kho.
    ///
    /// 🔴 Khoá `OpenWorkState` được mở và THẢ trong khối `{ ... }` dưới đây, TRƯỚC bất kỳ
    /// `.await` nào — xem doc-comment đầu tệp.
    #[tauri::command]
    pub async fn ai_translate_segment(
        app: tauri::AppHandle,
        segment_id: i64,
        prompt_set_name: Option<String>,
        channel: tauri::ipc::Channel<String>,
    ) -> Result<AiTranslateOutcomeWire, IpcError> {
        use tauri::Manager as _;

        let Some(record_state) = app.try_state::<LastAssembledPromptState>() else {
            // Khong bao gio xay ra tren duong san pham -- cung khuon `ai_prompt_assemble`.
            eprintln!(
                "ai_translate[segment] LastAssembledPromptState chua duoc quan ly -- loi cau hinh setup()"
            );
            return Err(IpcError::new(
                "ai_translate.record_state_missing",
                crate::core::i18n::MessageKey::Unknown,
                std::collections::BTreeMap::new(),
                false,
            ));
        };

        let prepared = {
            let global = app.try_state::<Store>();
            let work_state = app.try_state::<OpenWorkState>();
            let guard = work_state
                .as_ref()
                .map(|s| s.lock().unwrap_or_else(std::sync::PoisonError::into_inner));
            let open = guard.as_ref().and_then(|g| g.as_ref());

            prepare_translate_call(
                global.as_deref(),
                open,
                record_state.inner(),
                prompt_set_name.as_deref(),
                segment_id,
            )?
            // `guard`/`global` roi khoi pham vi o day -- khong khoa nao cua tien trinh con
            // song sang phia duoi.
        };

        let prepared = match prepared {
            PrepareOutcome::NotConfigured => return Ok(AiTranslateOutcomeWire::NotConfigured),
            PrepareOutcome::Ready(p) => p,
        };

        let Some(generation_state) = app.try_state::<AiTranslateGeneration>() else {
            eprintln!(
                "ai_translate[segment] AiTranslateGeneration chua duoc quan ly -- loi cau hinh setup()"
            );
            return Err(IpcError::new(
                "ai_translate.generation_state_missing",
                crate::core::i18n::MessageKey::Unknown,
                std::collections::BTreeMap::new(),
                false,
            ));
        };
        let generation_state = generation_state.inner().clone();
        let generation = generation_state.next();

        // `prepared.model`/`prepared.prompt` doc TRUOC khi `prepared` bi MOVE lam tham so cua
        // `send_prepared_translate_call` ngay duoi day -- can hai gia tri nay SAU luot `.await`
        // do (de ghi vao ban ghi cua 4.7's producer), luc `prepared` da khong con nua. `prompt_
        // sent` la CHUOI DUNG DA GUI (khong phai doc lai `record_state` sau await -- ban ghi do
        // co the da bi mot lot lap rap MOI ghi de trong luc dang cho mang, xem doc-comment
        // `mark_prompt_as_sent`), nen day la moc so DUY NHAT dung de quyet co stamp hay khong.
        let model_sent = prepared.model.clone();
        let prompt_sent = prepared.prompt.clone();
        let outcome = send_prepared_translate_call(prepared, channel, generation_state, generation).await;

        match outcome {
            Ok(TranslateOutcome::Cancelled) => Ok(AiTranslateOutcomeWire::Cancelled),
            Ok(TranslateOutcome::Done) => {
                if let Some(global) = app.try_state::<Store>() {
                    if let Some(sent_at) = super::now_utc_iso8601(&global) {
                        mark_prompt_as_sent(
                            record_state.inner(),
                            segment_id,
                            &prompt_sent,
                            &model_sent,
                            &sent_at,
                        );
                    }
                }
                Ok(AiTranslateOutcomeWire::Done)
            }
            Err(err) => Err(err.into()),
        }
    }

    /// Vỏ IPC huỷ lượt dịch đang chạy — bơm thế hệ lên MỘT, làm thế hệ đang chạy (nếu có)
    /// không còn là thế hệ hiện hành. Không đọc/ghi `Store` nào — chỉ một `AtomicU64`, nên
    /// không cần `(async)`.
    #[tauri::command]
    pub fn ai_translate_cancel(app: tauri::AppHandle) {
        use tauri::Manager as _;

        let Some(generation_state) = app.try_state::<AiTranslateGeneration>() else {
            eprintln!(
                "ai_translate[cancel] AiTranslateGeneration chua duoc quan ly -- loi cau hinh setup()"
            );
            return;
        };
        generation_state.next();
    }
}
