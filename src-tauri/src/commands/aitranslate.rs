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

use crate::commands::aiconfig::{endpoint_and_model_set, provider_needs_key};
use crate::commands::aiprompt::{
    AiCallRecord, AiCallRecordsState, AssembledPromptWire, CallSpec, LastAssembledPromptState,
    OperationPromptContext, RecordedSegment, assemble_and_record_prompt, assemble_call,
    next_call_record_stamp, prepare_operation_prompt, segment_not_in_chapter,
};
use crate::commands::project::{OpenWork, OpenWorkState};
use crate::commands::segment::source_text_hash;
use crate::core::ai::batch::{
    BlockEnd, BlockStream, CallEntry, DiscardReason, PlanScope, PlanSegment, PlannedCall,
    StreamEnd, Verdict, has_han, plan_calls, reconcile, system_with_block_format,
};
use crate::core::ai::client::{OpenAiChatClient, OpenAiClientError};
use crate::core::aiconfig::{
    AiConfigField, AiConfigKeyError, AiConfigStoreError, AiConfigTier, PROVIDER_DEFAULT, keychain,
    resolve_two_tiers,
};
use crate::core::i18n::{IpcError, MessageKey};
use crate::core::quick_translation::{
    PurgeScope, QuickTranslationStore, batch_prefix_len, group_by_key, quick_key,
};
use crate::core::scope::ScopeResolver;
use crate::core::store::{ReadHandle, Store, StoreError, StoreKind};
use crate::ports::translation_provider::{
    ConnectionCheck, ConnectionCheckOutcome, ConnectionCheckRequest, TranslateOutcome,
    TranslateRequest, TranslateUsage, TranslationProvider,
};

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

fn no_prompt_to_resend() -> IpcError {
    IpcError::new(
        "ai_translate.no_prompt_to_resend",
        MessageKey::AiTranslateNoPromptToResend,
        std::collections::BTreeMap::new(),
        false,
    )
}

/// Phân loại HOÀN CHỈNH của `OpenAiClientError` — Quyết định 2, spec 4.10: sáu HỌ NGUYÊN NHÂN,
/// mỗi họ một `code`/[`MessageKey`]/`retryable` riêng, dùng CHUNG cho cả lượt dịch MỘT segment
/// (`impl From<OpenAiClientError> for IpcError` ngay dưới) LẪN lượt dừng GIỮA một lô
/// ([`batch_stopped_error`]) — một chỗ gọi thứ hai dùng chung hàm này, không phải một bản chép
/// tay có thể trôi khỏi bản gốc.
///
/// 🔵 SỬA 2026-09-22 (Story 4.10, Phase 1) — bản trước ở đây
/// (`openai_client_error_is_retryable`) ghi: *"hai hàng ma trận đã ghi tường minh ... bốn nhánh
/// còn lại (dựng client, gửi thất bại, đọc thân lỗi giữa chừng, khung JSON hỏng) chưa có hàng ma
/// trận riêng; xếp CHẶT theo 'thất bại tạm thời, đáng thử lại' — Story 4.10 xét lại khi có chỗ
/// đo"* — và liệt `BufferOverflow` vào nhánh `false` mà không nhắc tới nó trong văn xuôi. Story
/// 4.10 CHÍNH LÀ bản xét lại đó: cả sáu họ giờ có hàng riêng trong §I/O Matrix spec 4.10, không
/// còn nhánh nào "xếp chặt" — `retryable` dưới đây khớp NGUYÊN VĂN Quyết định 2.
fn openai_client_error_family(err: &OpenAiClientError) -> (&'static str, MessageKey, bool) {
    match err {
        OpenAiClientError::RequestFailed { .. } | OpenAiClientError::ReadFailed { .. } => (
            "ai_translate.provider_unreachable",
            MessageKey::AiTranslateProviderUnreachable,
            true,
        ),
        OpenAiClientError::NonSuccessStatus { status, .. } => match status {
            401 | 403 => (
                "ai_translate.provider_key_rejected",
                MessageKey::AiTranslateProviderKeyRejected,
                false,
            ),
            402 => (
                "ai_translate.provider_payment_required",
                MessageKey::AiTranslateProviderPaymentRequired,
                false,
            ),
            404 => (
                "ai_translate.provider_endpoint_not_found",
                MessageKey::AiTranslateProviderEndpointNotFound,
                false,
            ),
            429 => (
                "ai_translate.provider_rate_limited",
                MessageKey::AiTranslateProviderRateLimited,
                false,
            ),
            500..=599 => (
                "ai_translate.provider_server_error",
                MessageKey::AiTranslateProviderServerError,
                true,
            ),
            _ => (
                "ai_translate.provider_refused",
                MessageKey::AiTranslateProviderRefused,
                false,
            ),
        },
        OpenAiClientError::StreamEndedWithoutDone => (
            "ai_translate.stream_ended_without_done",
            MessageKey::AiTranslateStreamEndedWithoutDone,
            true,
        ),
        OpenAiClientError::MalformedEvent { .. } | OpenAiClientError::BufferOverflow { .. } => (
            "ai_translate.reply_unreadable",
            MessageKey::AiTranslateReplyUnreadable,
            false,
        ),
        OpenAiClientError::EmptyReply { finish_reason } => (
            "ai_translate.reply_empty",
            if finish_reason.is_some() {
                MessageKey::AiTranslateReplyEmptyWithReason
            } else {
                MessageKey::AiTranslateReplyEmpty
            },
            false,
        ),
        OpenAiClientError::ClientBuildFailed { .. } => (
            "ai_translate.client_build_failed",
            MessageKey::AiTranslateClientBuildFailed,
            false,
        ),
        OpenAiClientError::ApiKeyHeaderInvalid { .. } => (
            "ai_translate.api_key_header_invalid",
            MessageKey::AiTranslateApiKeyHeaderInvalid,
            false,
        ),
    }
}

/// Provider trả lỗi mạng/HTTP/khung SSE hỏng — MỘT trong sáu NHÃN qua IPC theo họ nguyên nhân
/// (Quyết định 2, spec 4.10; trước bản sửa này là MỘT nhãn duy nhất cho cả bảy biến thể, xem
/// `openai_client_error_family` ngay trên). `status` (ca non-2xx) vẫn đi kèm `params` như trước
/// — khoá `AiTranslateProviderRefused` khai nó là tham số BẮT BUỘC (`core/i18n/mod.rs`).
impl From<OpenAiClientError> for IpcError {
    fn from(err: OpenAiClientError) -> Self {
        let mut params = std::collections::BTreeMap::new();
        insert_status_params(&mut params, &err);
        let (code, message_key, retryable) = openai_client_error_family(&err);
        IpcError::new(code, message_key, params, retryable)
    }
}

/// Provider stops in the middle of a grouped operation: the same six cause families as a
/// one-sentence call, with `call_index` and the first and last sentence of the call that stopped
/// as data params (AD-60 rule 8).
pub fn batch_stopped_error(
    call_index: u32,
    first_segment_id: i64,
    last_segment_id: i64,
    err: OpenAiClientError,
) -> IpcError {
    let mut params = std::collections::BTreeMap::new();
    params.insert("call_index".to_owned(), call_index.to_string());
    params.insert("first_segment_id".to_owned(), first_segment_id.to_string());
    params.insert("last_segment_id".to_owned(), last_segment_id.to_string());
    insert_status_params(&mut params, &err);
    let (code, message_key, retryable) = openai_client_error_family(&err);
    IpcError::new(code, message_key, params, retryable)
}

fn insert_status_params(
    params: &mut std::collections::BTreeMap<String, String>,
    err: &OpenAiClientError,
) {
    if let OpenAiClientError::EmptyReply {
        finish_reason: Some(reason),
    } = err
    {
        params.insert("finish_reason".to_owned(), reason.clone());
    }
    if let OpenAiClientError::NonSuccessStatus {
        status,
        provider_message,
    } = err
    {
        params.insert("status".to_owned(), status.to_string());
        if let Some(message) = provider_message {
            params.insert("provider_message".to_owned(), message.clone());
        }
    }
}

pub fn batch_panicked_error() -> IpcError {
    internal_failure_error()
}

fn internal_failure_error() -> IpcError {
    IpcError::new(
        "ai_translate.internal_failure",
        MessageKey::AiTranslateInternalFailure,
        std::collections::BTreeMap::new(),
        false,
    )
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
    pub system: Option<String>,
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
    resolved
        .get(field.as_str())
        .map(|f| f.value.clone())
        .unwrap_or_default()
}

pub(crate) struct ResolvedCallConfig<'a> {
    pub(crate) global_store: &'a Store,
    pub(crate) open_work: &'a OpenWork,
    pub(crate) provider: String,
    pub(crate) endpoint: String,
    pub(crate) model: String,
    pub(crate) temperature: Option<f64>,
    pub(crate) max_tokens: Option<u32>,
}

/// `None` only when `endpoint` or `model` is empty; an unset or unparsable
/// `temperature`/`max_tokens` is sent as absent, never treated as "not configured".
pub(crate) fn resolve_call_config<'a>(
    global: Option<&'a Store>,
    open: Option<&'a OpenWork>,
) -> Result<Option<ResolvedCallConfig<'a>>, IpcError> {
    let global_store = global.ok_or_else(store_is_missing)?;
    let open_work = open.ok_or_else(crate::commands::chapter::no_work_open)?;

    let resolver = open_work.scope.clone();
    let resolved = resolve_two_tiers(&resolver, global_store, Some(&open_work.store))?;

    let provider = resolved_field_value(&resolved, AiConfigField::Provider);
    let endpoint = resolved_field_value(&resolved, AiConfigField::Endpoint);
    let model = resolved_field_value(&resolved, AiConfigField::Model);
    if !endpoint_and_model_set(&endpoint, &model) {
        return Ok(None);
    }

    let raw_temperature = resolved_field_value(&resolved, AiConfigField::Temperature);
    let temperature: Option<f64> = if raw_temperature.trim() == PROVIDER_DEFAULT {
        None
    } else {
        raw_temperature.trim().parse().ok()
    };
    let raw_max_tokens = resolved_field_value(&resolved, AiConfigField::MaxTokens);
    let max_tokens: Option<u32> = if raw_max_tokens.trim() == PROVIDER_DEFAULT {
        None
    } else {
        match raw_max_tokens.trim().parse() {
            Ok(0) | Err(_) => None,
            Ok(v) => Some(v),
        }
    };

    Ok(Some(ResolvedCallConfig {
        global_store,
        open_work,
        provider,
        endpoint,
        model,
        temperature,
        max_tokens,
    }))
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
    let Some(ResolvedCallConfig {
        global_store,
        open_work,
        provider,
        endpoint,
        model,
        temperature,
        max_tokens,
    }) = resolve_call_config(global, open)?
    else {
        return Ok(PrepareOutcome::NotConfigured);
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

    let Some(api_key) = read_api_key(&provider)? else {
        return Ok(PrepareOutcome::NotConfigured);
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
        system: assembled.system,
    }))
}

/// Builds a call from the prompt string the record already holds, byte for byte: it never
/// assembles again and takes no prompt from the caller. Returns the record's `segment_id` with
/// the call so the result lands against the segment the prompt was built for.
///
/// # Errors
/// `ai_translate.no_prompt_to_resend` when nothing was assembled this session; store/work/keychain
/// errors as in [`prepare_translate_call`]. Unconfigured is [`PrepareOutcome::NotConfigured`].
pub fn prepare_resend_call(
    global: Option<&Store>,
    open: Option<&OpenWork>,
    record: &LastAssembledPromptState,
) -> Result<(i64, PrepareOutcome), IpcError> {
    let (segment_id, prompt, system) = {
        let guard = record
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        match guard.as_ref() {
            Some(r) => (r.segment_id, r.prompt.clone(), r.system.clone()),
            None => return Err(no_prompt_to_resend()),
        }
    };
    let Some(ResolvedCallConfig {
        provider,
        endpoint,
        model,
        temperature,
        max_tokens,
        ..
    }) = resolve_call_config(global, open)?
    else {
        return Ok((segment_id, PrepareOutcome::NotConfigured));
    };
    let Some(api_key) = read_api_key(&provider)? else {
        return Ok((segment_id, PrepareOutcome::NotConfigured));
    };
    Ok((
        segment_id,
        PrepareOutcome::Ready(PreparedTranslateCall {
            endpoint,
            model,
            temperature,
            max_tokens,
            api_key,
            prompt,
            system,
        }),
    ))
}

/// What a grouped operation translates: exactly some segments, or every untranslated one of the
/// open chapter.
#[derive(Debug, Clone, serde::Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum AiTranslateScope {
    Selection { segment_ids: Vec<i64> },
    Chapter,
}

/// A segment as the plan saw it. Sources, hashes and context text of every call come from here.
#[derive(Debug, Clone)]
pub struct SnapshotRow {
    pub id: i64,
    pub source_text: String,
    pub target_text: String,
    pub is_omitted: bool,
}

/// Endpoint settings and the key of one operation. Not `Debug`: `api_key` is a bare secret.
pub struct CallSettings {
    pub endpoint: String,
    pub model: String,
    pub temperature: Option<f64>,
    pub max_tokens: Option<u32>,
    pub api_key: String,
}

impl CallSettings {
    fn request(&self, prompt: String, system: Option<String>) -> PreparedTranslateCall {
        PreparedTranslateCall {
            endpoint: self.endpoint.clone(),
            model: self.model.clone(),
            temperature: self.temperature,
            max_tokens: self.max_tokens,
            api_key: self.api_key.clone(),
            prompt,
            system,
        }
    }
}

/// Everything read once at the start of an operation. `context` is `None` only when the plan
/// has no call.
pub struct PreparedOperation {
    pub settings: CallSettings,
    pub context: Option<OperationPromptContext>,
    pub snapshot: Vec<SnapshotRow>,
    pub chapter_id: i64,
    pub plan: Vec<PlannedCall>,
}

pub enum PrepareOperationOutcome {
    NotConfigured,
    Ready(Box<PreparedOperation>),
}

/// **Sync layer of a grouped operation.** Reads the chapter once, plans every call, and reads
/// the key, the prompt set, the system prompt and the TM policy once for all calls.
///
/// # Errors
/// `global.db` missing; no Work open; a selected id outside the open chapter
/// (`ai_prompt.segment_not_in_chapter`); keychain refusing to answer; a Store/Glossary/TM
/// read failing.
pub fn prepare_translate_operation(
    global: Option<&Store>,
    open: Option<&OpenWork>,
    prompt_set_name: Option<&str>,
    scope: &AiTranslateScope,
) -> Result<PrepareOperationOutcome, IpcError> {
    let Some(ResolvedCallConfig {
        open_work,
        provider,
        endpoint,
        model,
        temperature,
        max_tokens,
        ..
    }) = resolve_call_config(global, open)?
    else {
        return Ok(PrepareOperationOutcome::NotConfigured);
    };

    let chapter = crate::commands::segment::read_open_chapter_segments(Some(open_work))?;
    let selected: std::collections::BTreeSet<i64> = match scope {
        AiTranslateScope::Selection { segment_ids } => segment_ids.iter().copied().collect(),
        AiTranslateScope::Chapter => std::collections::BTreeSet::new(),
    };
    if let AiTranslateScope::Selection { .. } = scope {
        let known: std::collections::BTreeSet<i64> =
            chapter.segments.iter().map(|s| s.id).collect();
        if let Some(&missing) = selected.difference(&known).next() {
            return Err(segment_not_in_chapter(missing, chapter.chapter_id));
        }
    }
    let plan_segments: Vec<PlanSegment> = chapter
        .segments
        .iter()
        .map(|s| PlanSegment {
            id: s.id,
            is_omitted: s.is_omitted,
            translated: !s.target_text.trim().is_empty(),
        })
        .collect();
    let plan = plan_calls(
        &plan_segments,
        match scope {
            AiTranslateScope::Selection { .. } => PlanScope::Selection(&selected),
            AiTranslateScope::Chapter => PlanScope::Chapter,
        },
    );
    let snapshot: Vec<SnapshotRow> = chapter
        .segments
        .iter()
        .map(|s| SnapshotRow {
            id: s.id,
            source_text: s.source_text.clone(),
            target_text: s.target_text.clone(),
            is_omitted: s.is_omitted,
        })
        .collect();

    let (api_key, context) = if plan.is_empty() {
        (String::new(), None)
    } else {
        let Some(api_key) = read_api_key(&provider)? else {
            return Ok(PrepareOperationOutcome::NotConfigured);
        };
        let context = prepare_operation_prompt(global, Some(open_work), prompt_set_name)?;
        (api_key, Some(context))
    };

    Ok(PrepareOperationOutcome::Ready(Box::new(
        PreparedOperation {
            settings: CallSettings {
                endpoint,
                model,
                temperature,
                max_tokens,
                api_key,
            },
            context,
            snapshot,
            chapter_id: chapter.chapter_id,
            plan,
        },
    )))
}

/// Bộ đếm thế hệ dùng cho huỷ giữa chừng — đúng hình dạng
/// `commands::project::ImportScanGeneration` (`next()`/`is_current()`), một `Arc<AtomicU64>`
/// RIÊNG cho đường dịch AI (Quyết định 1, spec 4.8: cancel dựng ở CHÍNH story này). `Clone` chỉ
/// nhân bản `Arc` — vòng lặp đọc khung SSE và lệnh huỷ đọc/ghi CÙNG một bộ đếm.
#[derive(Debug, Clone, Default)]
pub struct AiTranslateGeneration(
    std::sync::Arc<std::sync::atomic::AtomicU64>,
    std::sync::Arc<std::sync::Mutex<AiTranslateActivityWire>>,
);

#[derive(Debug, Clone, Default, PartialEq, Eq, serde::Serialize)]
pub struct AiTranslateActivityWire {
    pub revision: u64,
    pub active_count: u64,
}

pub struct AiTranslateActivityGuard {
    state: AiTranslateGeneration,
    notify: Box<dyn Fn(AiTranslateActivityWire) + Send + Sync>,
}

impl Drop for AiTranslateActivityGuard {
    fn drop(&mut self) {
        let snapshot = {
            let mut state = self
                .state
                .1
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            state.active_count -= 1;
            state.revision += 1;
            state.clone()
        };
        (self.notify)(snapshot);
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum QuickTranslateTrigger {
    Auto,
    Manual,
}

pub fn quick_auto_gate(
    global: Option<&Store>,
    activity: &AiTranslateActivityWire,
    trigger: QuickTranslateTrigger,
) -> Result<Option<QuickTranslateOutcomeWire>, IpcError> {
    if trigger == QuickTranslateTrigger::Manual {
        return Ok(None);
    }
    let Some(global) = global else {
        return Ok(Some(QuickTranslateOutcomeWire::AutoOff));
    };
    let config = crate::core::aiconfig::store::resolve_two_tiers(
        &crate::core::scope::ScopeResolver::global_only(),
        global,
        None,
    )?;
    let dedicated = ["quick_endpoint", "quick_model"].iter().all(|key| {
        config
            .get(*key)
            .is_some_and(|field| !field.value.trim().is_empty())
    });
    if !crate::core::scope::store::load_global_config(global)?.quick_translate_auto() || !dedicated
    {
        return Ok(Some(QuickTranslateOutcomeWire::AutoOff));
    }
    if activity.active_count > 0 {
        return Ok(Some(QuickTranslateOutcomeWire::Paused));
    }
    Ok(None)
}

impl AiTranslateGeneration {
    pub fn activity(&self) -> AiTranslateActivityWire {
        self.1
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clone()
    }

    pub fn begin_activity(
        &self,
        notify: impl Fn(AiTranslateActivityWire) + Send + Sync + 'static,
    ) -> AiTranslateActivityGuard {
        let snapshot = {
            let mut state = self
                .1
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            state.active_count += 1;
            state.revision += 1;
            state.clone()
        };
        notify(snapshot);
        AiTranslateActivityGuard {
            state: self.clone(),
            notify: Box::new(notify),
        }
    }

    pub(crate) fn next(&self) -> u64 {
        self.0
            .fetch_add(1, std::sync::atomic::Ordering::AcqRel)
            .wrapping_add(1)
    }

    pub fn is_current(&self, generation: u64) -> bool {
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
) -> Result<SingleRun, SingleRunError<P::Error>> {
    let request = TranslateRequest {
        endpoint: &prepared.endpoint,
        model: &prepared.model,
        temperature: prepared.temperature,
        max_tokens: prepared.max_tokens,
        api_key: &prepared.api_key,
        prompt: &prepared.prompt,
        system: prepared.system.as_deref(),
    };
    let mut reader = BlockStream::new();
    let result = {
        let mut on_token = |text: &str| {
            // Pieces are pushed one line at a time so the opening a piece belongs to is known.
            for part in text.split_inclusive('\n') {
                let first_opening = reader.ended_openings(1) == 0;
                for piece in reader.push(part) {
                    if piece.index == 1 && first_opening {
                        // A closed channel or transient error leaves earlier text on screen.
                        let _ = channel.send(piece.text);
                    }
                }
            }
        };
        provider
            .translate(request, &mut on_token, should_cancel)
            .await
    }
    .map_err(SingleRunError::Provider)?;
    let blocks = reader.finish();
    match result {
        TranslateOutcome::Cancelled => Ok(SingleRun::Cancelled),
        TranslateOutcome::Done {
            usage,
            finish_reason,
        } => {
            let mut first = blocks.iter().filter(|b| b.index == 1);
            match (first.next(), first.next()) {
                (Some(block), None)
                    if !block.text.is_empty() && block.end != BlockEnd::Superseded =>
                {
                    Ok(SingleRun::Done {
                        usage,
                        finish_reason,
                        block_open: block.end == BlockEnd::StreamEnd,
                    })
                }
                _ => Err(SingleRunError::ResultMissing),
            }
        }
    }
}

/// What the single-sentence reader made of a finished call.
#[derive(Debug, PartialEq)]
pub enum SingleRun {
    Cancelled,
    Done {
        usage: Option<TranslateUsage>,
        finish_reason: Option<String>,
        /// Block 1 had not closed when the stream ended.
        block_open: bool,
    },
}

#[derive(Debug, PartialEq)]
pub enum SingleRunError<E> {
    Provider(E),
    /// The model answered, but block 1 is absent, repeated or empty.
    ResultMissing,
}

impl<E: Into<IpcError>> From<SingleRunError<E>> for IpcError {
    fn from(err: SingleRunError<E>) -> Self {
        match err {
            SingleRunError::Provider(e) => e.into(),
            SingleRunError::ResultMissing => result_missing_error(),
        }
    }
}

fn result_missing_error() -> IpcError {
    IpcError::new(
        "ai_translate.result_missing",
        MessageKey::AiTranslateResultMissing,
        std::collections::BTreeMap::new(),
        false,
    )
}

/// Generation counter of the connection check, separate from [`AiTranslateGeneration`] so a check
/// never supersedes a running translate or batch (and its cancel never stops one).
#[derive(Debug, Clone, Default)]
pub struct ConnectionCheckGeneration(std::sync::Arc<std::sync::atomic::AtomicU64>);

impl ConnectionCheckGeneration {
    pub fn next(&self) -> u64 {
        self.0
            .fetch_add(1, std::sync::atomic::Ordering::AcqRel)
            .wrapping_add(1)
    }

    pub fn is_current(&self, generation: u64) -> bool {
        self.0.load(std::sync::atomic::Ordering::Acquire) == generation
    }
}

/// The surface a quick-translation run belongs to. Another surface cannot start while a run is
/// active; the same surface replaces its own run.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QuickSurface {
    GlossaryQueue,
    ImportPreview,
}

/// The one quick-translation run of the app, separate from [`AiTranslateGeneration`] so its cancel
/// never stops a translate run or batch. The webview issues `run_id`; a cancel only takes effect
/// when it names the run that is active.
#[derive(Debug, Clone, Default)]
pub struct QuickTranslateGeneration(std::sync::Arc<QuickTranslateRuns>);

#[derive(Debug, Default)]
struct QuickTranslateRuns {
    generation: std::sync::atomic::AtomicU64,
    active_run: std::sync::Mutex<Option<(QuickSurface, u64)>>,
}

/// Proof that a run holds the app's one quick-translation slot; dropping it frees the slot.
#[derive(Debug)]
pub struct QuickRunTicket {
    runs: QuickTranslateGeneration,
    surface: QuickSurface,
    run_id: u64,
    pub generation: u64,
}

impl Drop for QuickRunTicket {
    fn drop(&mut self) {
        let mut active = self
            .runs
            .0
            .active_run
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if *active == Some((self.surface, self.run_id)) {
            *active = None;
        }
    }
}

impl QuickTranslateGeneration {
    /// Starts `run_id` on `surface`, superseding an earlier run of the same surface. `None` is
    /// busy: a run of another surface is active.
    pub fn begin(&self, surface: QuickSurface, run_id: u64) -> Option<QuickRunTicket> {
        let mut active = self
            .0
            .active_run
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if active.is_some_and(|(running, _)| running != surface) {
            return None;
        }
        *active = Some((surface, run_id));
        let generation = self
            .0
            .generation
            .fetch_add(1, std::sync::atomic::Ordering::AcqRel)
            .wrapping_add(1);
        Some(QuickRunTicket {
            runs: self.clone(),
            surface,
            run_id,
            generation,
        })
    }

    /// Cancels the active run when `run_id` names it. Returns whether it did.
    pub fn cancel(&self, run_id: u64) -> bool {
        let mut active = self
            .0
            .active_run
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if active.map(|(_, id)| id) != Some(run_id) {
            return false;
        }
        *active = None;
        self.0
            .generation
            .fetch_add(1, std::sync::atomic::Ordering::AcqRel);
        true
    }

    /// Cancels whatever run is active, whoever started it: bumps `generation` so every pending
    /// `save` fails `is_current`, and frees the slot.
    pub fn cancel_all(&self) {
        let mut active = self
            .0
            .active_run
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        *active = None;
        self.0
            .generation
            .fetch_add(1, std::sync::atomic::Ordering::AcqRel);
    }

    pub fn is_current(&self, generation: u64) -> bool {
        self.0.generation.load(std::sync::atomic::Ordering::Acquire) == generation
    }
}

#[derive(Debug, Clone, Default)]
pub struct QuickConnectionCheckGeneration(pub ConnectionCheckGeneration);

pub struct ResolvedQuickCallConfig {
    pub provider: String,
    pub endpoint: String,
    pub model: String,
    pub temperature: Option<f64>,
    pub max_tokens: Option<u32>,
    pub dedicated: bool,
}

pub fn resolve_quick_call_config(
    global: Option<&Store>,
) -> Result<Option<ResolvedQuickCallConfig>, IpcError> {
    use crate::core::aiconfig::{QuickConfigSource, quick_config_source};
    let global = global.ok_or_else(store_is_missing)?;
    let resolved = resolve_two_tiers(&ScopeResolver::global_only(), global, None)?;
    let source = quick_config_source(&resolved);
    if source == QuickConfigSource::Incomplete {
        return Ok(None);
    }
    let dedicated = source == QuickConfigSource::Dedicated;
    let value = |key: &str| {
        resolved
            .get(key)
            .map(|f| f.value.clone())
            .unwrap_or_default()
    };
    let (provider, endpoint, model) = if dedicated {
        (
            value("quick_provider"),
            value("quick_endpoint"),
            value("quick_model"),
        )
    } else {
        (value("provider"), value("endpoint"), value("model"))
    };
    if !endpoint_and_model_set(&endpoint, &model) {
        return Ok(None);
    }
    let temperature = if dedicated {
        None
    } else {
        value("temperature").trim().parse().ok()
    };
    let max_tokens = if dedicated {
        None
    } else {
        value("max_tokens")
            .trim()
            .parse::<u32>()
            .ok()
            .filter(|v| *v > 0)
    };
    Ok(Some(ResolvedQuickCallConfig {
        provider,
        endpoint,
        model,
        temperature,
        max_tokens,
        dedicated,
    }))
}

fn read_quick_api_key(config: &ResolvedQuickCallConfig) -> Result<Option<String>, IpcError> {
    if !config.dedicated {
        return read_api_key(&config.provider);
    }
    if !provider_needs_key(&config.provider) {
        return Ok(Some(String::new()));
    }
    match keychain::read_account(keychain::KeyAccount::Quick) {
        Ok(Some(secret)) => Ok(Some(secret.expose_secret().to_owned())),
        Ok(None) => Ok(None),
        Err(_) => Err(keychain_unavailable()),
    }
}

pub fn prepare_quick_connection_check(
    global: Option<&Store>,
) -> Result<ConnectionCheckPrep, IpcError> {
    let Some(config) = resolve_quick_call_config(global)? else {
        return Ok(ConnectionCheckPrep::NotConfigured);
    };
    let Some(api_key) = read_quick_api_key(&config)? else {
        return Ok(ConnectionCheckPrep::NotConfigured);
    };
    Ok(ConnectionCheckPrep::Ready(PreparedConnectionCheck {
        endpoint: config.endpoint,
        model: config.model,
        api_key: provider_needs_key(&config.provider).then_some(api_key),
    }))
}

/// Resolves the quick-translation configuration and key into a call carrying `prompt`. Not
/// configured (empty endpoint/model, or no stored key) is reached before any network call.
/// Reads and writes no Work data and records nothing.
pub fn prepare_quick_call(
    global: Option<&Store>,
    prompt: String,
    system: Option<String>,
) -> Result<PrepareOutcome, IpcError> {
    let Some(config) = resolve_quick_call_config(global)? else {
        return Ok(PrepareOutcome::NotConfigured);
    };
    let Some(api_key) = read_quick_api_key(&config)? else {
        return Ok(PrepareOutcome::NotConfigured);
    };
    Ok(PrepareOutcome::Ready(PreparedTranslateCall {
        endpoint: config.endpoint,
        model: config.model,
        temperature: config.temperature,
        max_tokens: config.max_tokens,
        api_key,
        prompt,
        system,
    }))
}

/// Builds the call that translates one Glossary context sentence for reference.
pub fn prepare_context_reference_call(
    global: Option<&Store>,
    _open: Option<&OpenWork>,
    context_example: &str,
) -> Result<PrepareOutcome, IpcError> {
    prepare_quick_call(
        global,
        crate::commands::aiprompt::context_reference_prompt(context_example),
        None,
    )
}

/// Result of one `Dịch thử`, shared by the lookup-first and send paths (AD-64 rules 5, 6).
/// Not `AiTranslateOutcomeWire`: that type is shared with the real batch.
#[derive(Debug, Clone, PartialEq, serde::Serialize)]
#[serde(tag = "state", rename_all = "snake_case")]
pub enum QuickTranslateOutcomeWire {
    AutoOff,
    Paused,
    NotConfigured,
    StoreUnavailable,
    Cached {
        text: String,
        model: String,
    },
    Done {
        model: String,
        usage: Option<AiTranslateUsageWire>,
        maybe_truncated: bool,
    },
    Cancelled,
    Busy,
}

/// Result of `quick_translation_lookup`.
#[derive(Debug, Clone, PartialEq, serde::Serialize)]
#[serde(tag = "state", rename_all = "snake_case")]
pub enum QuickLookupWire {
    StoreUnavailable,
    Miss,
    Hit { text: String, model: String },
}

#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub struct QuickModelCountWire {
    pub model: String,
    pub count: u64,
}

/// Result of `quick_translation_stats`. `bytes` is the `.db` plus its `-wal`.
#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub struct QuickStatsWire {
    pub count: u64,
    pub bytes: u64,
    pub by_model: Vec<QuickModelCountWire>,
}

/// `scope` argument of `quick_translation_purge`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum QuickPurgeScopeWire {
    All,
    OlderThan { days: i64 },
}

/// Result of `quick_translation_purge`. `compact_error` set means the rows are deleted but the
/// file was not shrunk.
#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub struct QuickPurgeWire {
    pub deleted: u64,
    pub bytes_before: u64,
    pub bytes_after: u64,
    pub compact_error: Option<String>,
}

fn quick_store_missing() -> IpcError {
    crate::core::quick_translation::store_not_managed().into()
}

fn invalid_days_error() -> IpcError {
    IpcError::new(
        "quick_translation.invalid_days",
        MessageKey::QuickTranslationInvalidDays,
        std::collections::BTreeMap::new(),
        false,
    )
}

fn checked_days(days: i64) -> Result<u32, IpcError> {
    u32::try_from(days)
        .ok()
        .filter(|d| *d >= 1)
        .ok_or_else(invalid_days_error)
}

pub fn quick_stats_wire(store: Option<&QuickTranslationStore>) -> Result<QuickStatsWire, IpcError> {
    let stats = store.ok_or_else(quick_store_missing)?.stats()?;
    Ok(QuickStatsWire {
        count: stats.count,
        bytes: stats.bytes,
        by_model: stats
            .by_model
            .into_iter()
            .map(|m| QuickModelCountWire {
                model: m.model,
                count: m.count,
            })
            .collect(),
    })
}

pub fn quick_count_older_wire(
    store: Option<&QuickTranslationStore>,
    days: i64,
) -> Result<u64, IpcError> {
    let days = checked_days(days)?;
    Ok(store
        .ok_or_else(quick_store_missing)?
        .count_older_than(days)?)
}

/// Cancels every quick-translation run BEFORE the delete job is queued, so a `save` queued
/// after it fails `is_current` and one queued before it is deleted by the purge.
pub fn quick_purge_wire(
    store: Option<&QuickTranslationStore>,
    runs: Option<&QuickTranslateGeneration>,
    scope: QuickPurgeScopeWire,
) -> Result<QuickPurgeWire, IpcError> {
    let scope = match scope {
        QuickPurgeScopeWire::All => PurgeScope::All,
        QuickPurgeScopeWire::OlderThan { days } => PurgeScope::OlderThanDays(checked_days(days)?),
    };
    let store = store.ok_or_else(quick_store_missing)?;
    if let Some(runs) = runs {
        runs.cancel_all();
    }
    let outcome = store.purge(scope)?;
    Ok(QuickPurgeWire {
        deleted: outcome.deleted,
        bytes_before: outcome.bytes_before,
        bytes_after: outcome.bytes_after,
        compact_error: outcome.compact_error,
    })
}

pub fn quick_lookup_wire(store: Option<&QuickTranslationStore>, text: &str) -> QuickLookupWire {
    let Some(store) = store else {
        return QuickLookupWire::StoreUnavailable;
    };
    match store.lookup(text) {
        Ok(Some(hit)) => QuickLookupWire::Hit {
            text: hit.translated_text,
            model: hit.model,
        },
        Ok(None) => QuickLookupWire::Miss,
        Err(err) => {
            eprintln!("quick_translation[lookup] read failed: {err}");
            QuickLookupWire::StoreUnavailable
        }
    }
}

pub fn quick_lookup_many_wire(
    store: Option<&QuickTranslationStore>,
    texts: &[String],
) -> Vec<QuickLookupWire> {
    let Some(store) = store else {
        return vec![QuickLookupWire::StoreUnavailable; texts.len()];
    };
    let keys: Vec<String> = texts.iter().map(|t| quick_key(t)).collect();
    match store.lookup_keys(&keys) {
        Ok(hits) => keys
            .iter()
            .zip(hits)
            .map(|(key, hit)| match hit {
                Some(hit) if !key.is_empty() => QuickLookupWire::Hit {
                    text: hit.translated_text,
                    model: hit.model,
                },
                _ => QuickLookupWire::Miss,
            })
            .collect(),
        Err(err) => {
            eprintln!("quick_translation[lookup_many] read failed: {err}");
            vec![QuickLookupWire::StoreUnavailable; texts.len()]
        }
    }
}

#[derive(Debug)]
pub enum QuickTranslateError<E> {
    Provider(E),
    Ipc(IpcError),
}

impl<E: Into<IpcError>> From<QuickTranslateError<E>> for IpcError {
    fn from(err: QuickTranslateError<E>) -> Self {
        match err {
            QuickTranslateError::Provider(e) => e.into(),
            QuickTranslateError::Ipc(e) => e,
        }
    }
}

/// One `Dịch thử` after the surface has resolved its inputs: a missing store, then the cache
/// (skipped by `retranslate`), then the provider, then a save of a complete result. `prepare`
/// runs only after a cache miss, so a cache hit reads no key.
pub async fn quick_translate<P: TranslationProvider>(
    store: Option<&QuickTranslationStore>,
    provider: &P,
    prepare: impl FnOnce() -> Result<PrepareOutcome, IpcError>,
    text: &str,
    retranslate: bool,
    on_token: &mut dyn FnMut(&str),
    should_cancel: std::sync::Arc<dyn Fn() -> bool + Send + Sync>,
) -> Result<QuickTranslateOutcomeWire, QuickTranslateError<P::Error>> {
    let Some(store) = store else {
        return Ok(QuickTranslateOutcomeWire::StoreUnavailable);
    };
    if !retranslate {
        match store.lookup(text) {
            Ok(Some(hit)) => {
                return Ok(QuickTranslateOutcomeWire::Cached {
                    text: hit.translated_text,
                    model: hit.model,
                });
            }
            Ok(None) => {}
            Err(err) => {
                eprintln!("quick_translation[lookup] read failed: {err}");
                return Ok(QuickTranslateOutcomeWire::StoreUnavailable);
            }
        }
    }
    let prepared = match prepare().map_err(QuickTranslateError::Ipc)? {
        PrepareOutcome::NotConfigured => return Ok(QuickTranslateOutcomeWire::NotConfigured),
        PrepareOutcome::Ready(prepared) => prepared,
    };
    if should_cancel() {
        return Ok(QuickTranslateOutcomeWire::Cancelled);
    }

    let request = TranslateRequest {
        endpoint: &prepared.endpoint,
        model: &prepared.model,
        temperature: prepared.temperature,
        max_tokens: prepared.max_tokens,
        api_key: &prepared.api_key,
        prompt: &prepared.prompt,
        system: prepared.system.as_deref(),
    };
    let mut received = String::new();
    let mut forward = |token: &str| {
        received.push_str(token);
        on_token(token);
    };
    let outcome = provider
        .translate(request, &mut forward, &*should_cancel)
        .await
        .map_err(QuickTranslateError::Provider)?;
    let TranslateOutcome::Done {
        usage,
        finish_reason,
    } = outcome
    else {
        return Ok(QuickTranslateOutcomeWire::Cancelled);
    };

    let maybe_truncated = finish_reason.as_deref() != Some("stop");
    if !maybe_truncated {
        let still_current = std::sync::Arc::clone(&should_cancel);
        if let Err(err) = store.save(text, &received, &prepared.model, retranslate, move || {
            !still_current()
        }) {
            eprintln!("quick_translation[save] write failed: {err}");
        }
    }
    Ok(QuickTranslateOutcomeWire::Done {
        model: prepared.model,
        usage: usage.map(AiTranslateUsageWire::from),
        maybe_truncated,
    })
}

/// One event of a quick batch on its `Channel`. `index` is the position in the `texts` the
/// webview sent; a text shared by several positions is answered at each of them.
#[derive(Debug, Clone, PartialEq, serde::Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum QuickBlockEventWire {
    /// Already remembered: no call was made for it.
    Cached {
        index: u32,
        text: String,
        model: String,
    },
    /// The positions the one call carries.
    Started {
        indices: Vec<u32>,
    },
    Token {
        index: u32,
        text: String,
    },
    /// A block closed correctly. `saved` is false when the store refused the write.
    Received {
        index: u32,
        text: String,
        model: String,
        saved: bool,
    },
}

#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub struct QuickMissingWire {
    pub index: u32,
    pub reason: &'static str,
}

/// Result of one quick batch. A provider error leaves through `Err`; the blocks received before
/// it stay saved and were already reported as `Received`.
#[derive(Debug, Clone, PartialEq, serde::Serialize)]
#[serde(tag = "state", rename_all = "snake_case")]
pub enum QuickBatchOutcomeWire {
    AutoOff,
    Paused,
    NotConfigured,
    StoreUnavailable,
    Busy,
    Cancelled,
    Done {
        /// `None` when nothing needed a call.
        model: Option<String>,
        usage: Option<AiTranslateUsageWire>,
        missing: Vec<QuickMissingWire>,
        /// Positions that did not fit this call; the next call takes them.
        deferred: u32,
    },
}

fn wire_index(index: usize) -> u32 {
    u32::try_from(index).unwrap_or(u32::MAX)
}

/// One batch of page blocks (AD-64 rules 4 to 8): remembered texts answer from the store, the
/// rest go in one call, and each block that closes correctly is saved at once, so a cancel keeps
/// what was already paid for. `prepare` supplies the quick configuration; its prompt and system
/// are replaced here. `retranslate` skips the store read and replaces the saved rows (`Dịch lại`).
/// Never writes `target_text`, TM, Glossary, and creates no Work.
pub async fn quick_translate_blocks<P: TranslationProvider>(
    store: Option<&QuickTranslationStore>,
    provider: &P,
    prepare: impl FnOnce() -> Result<PrepareOutcome, IpcError>,
    texts: &[String],
    retranslate: bool,
    on_event: &mut dyn FnMut(QuickBlockEventWire),
    should_cancel: std::sync::Arc<dyn Fn() -> bool + Send + Sync>,
) -> Result<QuickBatchOutcomeWire, QuickTranslateError<P::Error>> {
    let Some(store) = store else {
        return Ok(QuickBatchOutcomeWire::StoreUnavailable);
    };
    let groups = group_by_key(texts);
    let keys: Vec<String> = groups.iter().map(|g| g.key.clone()).collect();
    let hits = if retranslate {
        vec![None; keys.len()]
    } else {
        match store.lookup_keys(&keys) {
            Ok(hits) => hits,
            Err(err) => {
                eprintln!("quick_translation[batch lookup] read failed: {err}");
                return Ok(QuickBatchOutcomeWire::StoreUnavailable);
            }
        }
    };
    let mut misses = Vec::new();
    for (group, hit) in groups.into_iter().zip(hits) {
        match hit {
            Some(hit) => {
                for &index in &group.indices {
                    on_event(QuickBlockEventWire::Cached {
                        index: wire_index(index),
                        text: hit.translated_text.clone(),
                        model: hit.model.clone(),
                    });
                }
            }
            None => misses.push(group),
        }
    }
    let taken = batch_prefix_len(&misses);
    let deferred: usize = misses[taken..].iter().map(|g| g.indices.len()).sum();
    misses.truncate(taken);
    let deferred = wire_index(deferred);
    if misses.is_empty() {
        return Ok(QuickBatchOutcomeWire::Done {
            model: None,
            usage: None,
            missing: Vec::new(),
            deferred,
        });
    }

    let mut prepared = match prepare().map_err(QuickTranslateError::Ipc)? {
        PrepareOutcome::NotConfigured => return Ok(QuickBatchOutcomeWire::NotConfigured),
        PrepareOutcome::Ready(prepared) => prepared,
    };
    if should_cancel() {
        return Ok(QuickBatchOutcomeWire::Cancelled);
    }
    let sources: Vec<&str> = misses.iter().map(|g| g.key.as_str()).collect();
    prepared.prompt = crate::commands::aiprompt::quick_batch_prompt(&sources);
    prepared.system = Some(system_with_block_format(None));
    on_event(QuickBlockEventWire::Started {
        indices: misses
            .iter()
            .flat_map(|g| g.indices.iter().map(|&i| wire_index(i)))
            .collect(),
    });

    let request = TranslateRequest {
        endpoint: &prepared.endpoint,
        model: &prepared.model,
        temperature: prepared.temperature,
        max_tokens: prepared.max_tokens,
        api_key: &prepared.api_key,
        prompt: &prepared.prompt,
        system: prepared.system.as_deref(),
    };
    let model = prepared.model.clone();
    let saved_blocks: std::cell::RefCell<std::collections::BTreeSet<u32>> =
        std::cell::RefCell::default();
    let save_block = |block: u32, text: &str, on_event: &mut dyn FnMut(QuickBlockEventWire)| {
        let Some(group) = usize::try_from(block)
            .ok()
            .and_then(|n| n.checked_sub(1))
            .and_then(|i| misses.get(i))
        else {
            return;
        };
        if text.trim().is_empty() || should_cancel() || !saved_blocks.borrow_mut().insert(block) {
            return;
        }
        let still_current = std::sync::Arc::clone(&should_cancel);
        let saved = match store.save(&group.key, text, &model, retranslate, move || {
            !still_current()
        }) {
            Ok(written) => written,
            Err(err) => {
                eprintln!("quick_translation[batch save] write failed: {err}");
                false
            }
        };
        for &index in &group.indices {
            on_event(QuickBlockEventWire::Received {
                index: wire_index(index),
                text: text.to_owned(),
                model: model.clone(),
                saved,
            });
        }
    };

    let mut reader = BlockStream::new();
    let result = {
        let mut on_token = |chunk: &str| {
            for piece in reader.push(chunk) {
                let Some(group) = usize::try_from(piece.index)
                    .ok()
                    .and_then(|n| n.checked_sub(1))
                    .and_then(|i| misses.get(i))
                else {
                    continue;
                };
                for &index in &group.indices {
                    on_event(QuickBlockEventWire::Token {
                        index: wire_index(index),
                        text: piece.text.clone(),
                    });
                }
            }
            for block in reader.take_closed() {
                save_block(block.index, &block.text, on_event);
            }
        };
        provider
            .translate(request, &mut on_token, &*should_cancel)
            .await
    };
    let blocks = reader.finish();

    let (end, usage) = match result {
        Ok(TranslateOutcome::Done {
            usage,
            finish_reason,
        }) => (Some(finish_reason), usage.map(AiTranslateUsageWire::from)),
        Ok(TranslateOutcome::Cancelled) => return Ok(QuickBatchOutcomeWire::Cancelled),
        Err(err) if P::is_empty_reply(&err) => (Some(None), None),
        Err(err) => return Err(QuickTranslateError::Provider(err)),
    };
    let stream_end = match &end {
        Some(reason) => StreamEnd::Finished {
            finish_reason: reason.as_deref(),
        },
        None => StreamEnd::Interrupted,
    };
    let verdicts = reconcile(misses.len(), &blocks, &stream_end);
    let mut missing = Vec::new();
    for (i, (group, verdict)) in misses.iter().zip(&verdicts).enumerate() {
        let block = wire_index(i + 1);
        match verdict {
            Verdict::Received { text } => save_block(block, text, on_event),
            Verdict::Discarded { reason } if !saved_blocks.borrow().contains(&block) => {
                missing.extend(group.indices.iter().map(|&index| QuickMissingWire {
                    index: wire_index(index),
                    reason: reason.as_str(),
                }));
            }
            Verdict::Discarded { .. } => {}
        }
    }
    if should_cancel() {
        return Ok(QuickBatchOutcomeWire::Cancelled);
    }
    Ok(QuickBatchOutcomeWire::Done {
        model: Some(model),
        usage,
        missing,
        deferred,
    })
}

/// Keychain từ chối trả lời lúc dịch — tái dùng ĐÚNG khoá của Story 4.3 (§Code Map spec 4.8:
/// "reuses ... `AiConfigKeychainUnavailable` where the fact already has a key"), không một khoá
/// thứ hai cho cùng sự thật.
fn keychain_unavailable() -> IpcError {
    AiConfigKeyError::Unavailable.into()
}

/// The key to send for `provider`: empty for a local runtime (nothing is stored or read), the
/// stored key otherwise. `None` is "no key stored".
pub(crate) fn read_api_key(provider: &str) -> Result<Option<String>, IpcError> {
    if !provider_needs_key(provider) {
        return Ok(Some(String::new()));
    }
    match keychain::read() {
        Ok(Some(secret)) => Ok(Some(secret.expose_secret().to_owned())),
        Ok(None) => Ok(None),
        Err(_unavailable) => Err(keychain_unavailable()),
    }
}

/// A connection check ready to send. Not `Debug`: `api_key` is a bare secret.
pub struct PreparedConnectionCheck {
    pub endpoint: String,
    pub model: String,
    pub api_key: Option<String>,
}

/// Result of [`prepare_connection_check`]: "not configured" is a state, not an error.
pub enum ConnectionCheckPrep {
    NotConfigured,
    Ready(PreparedConnectionCheck),
}

/// Resolves the SAVED configuration of `tier` (Global reads `global.db` only; Work resolves both
/// tiers) and, for a provider that needs one, the stored key. No network.
///
/// # Errors
/// `global.db` missing; Work tier without an open Work (`ai_config.work_tier_unavailable`);
/// keychain refusing to answer (`ai_config.keychain_unavailable`).
pub fn prepare_connection_check(
    global: Option<&Store>,
    open: Option<&OpenWork>,
    tier: AiConfigTier,
) -> Result<ConnectionCheckPrep, IpcError> {
    let global_store = global.ok_or_else(store_is_missing)?;
    let resolved = match tier {
        AiConfigTier::Global => {
            resolve_two_tiers(&ScopeResolver::global_only(), global_store, None)?
        }
        AiConfigTier::Work => {
            let work = open.ok_or(AiConfigStoreError::WorkTierUnavailable)?;
            resolve_two_tiers(&work.scope.clone(), global_store, Some(&work.store))?
        }
    };
    let value = |field: AiConfigField| {
        resolved
            .get(field.as_str())
            .map(|f| f.value.clone())
            .unwrap_or_default()
    };
    let (provider, endpoint, model) = (
        value(AiConfigField::Provider),
        value(AiConfigField::Endpoint),
        value(AiConfigField::Model),
    );
    if !endpoint_and_model_set(&endpoint, &model) {
        return Ok(ConnectionCheckPrep::NotConfigured);
    }
    let api_key = if provider_needs_key(&provider) {
        match read_api_key(&provider)? {
            Some(key) => Some(key),
            None => return Ok(ConnectionCheckPrep::NotConfigured),
        }
    } else {
        None
    };
    Ok(ConnectionCheckPrep::Ready(PreparedConnectionCheck {
        endpoint,
        model,
        api_key,
    }))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ConnectionCheckWire {
    Connected { model_count: usize },
    NotConfigured,
    Cancelled,
}

fn connection_check_error(err: OpenAiClientError) -> IpcError {
    let no_params = std::collections::BTreeMap::new;
    let mut status_params = std::collections::BTreeMap::new();
    insert_status_params(&mut status_params, &err);
    let message_only = || {
        let mut params = std::collections::BTreeMap::new();
        if let Some(message) = status_params.get("provider_message") {
            params.insert("provider_message".to_owned(), message.clone());
        }
        params
    };
    let with_status = |code: &str, key: MessageKey, retryable: bool| {
        IpcError::new(code, key, status_params.clone(), retryable)
    };
    match err {
        OpenAiClientError::NonSuccessStatus {
            status: 401 | 403, ..
        } => IpcError::new(
            "ai_config.test_key_rejected",
            MessageKey::AiConfigTestKeyRejected,
            message_only(),
            false,
        ),
        OpenAiClientError::NonSuccessStatus { status: 404, .. } => IpcError::new(
            "ai_config.test_endpoint_not_found",
            MessageKey::AiConfigTestEndpointNotFound,
            message_only(),
            false,
        ),
        OpenAiClientError::NonSuccessStatus { status: 402, .. } => with_status(
            "ai_config.test_payment_required",
            MessageKey::AiConfigTestPaymentRequired,
            false,
        ),
        OpenAiClientError::NonSuccessStatus { status: 429, .. } => with_status(
            "ai_config.test_rate_limited",
            MessageKey::AiConfigTestRateLimited,
            false,
        ),
        OpenAiClientError::NonSuccessStatus {
            status: 500..=599, ..
        } => with_status(
            "ai_config.test_server_error",
            MessageKey::AiConfigTestServerError,
            true,
        ),
        OpenAiClientError::NonSuccessStatus { .. } => with_status(
            "ai_config.test_refused",
            MessageKey::AiConfigTestRefused,
            false,
        ),
        OpenAiClientError::RequestFailed { .. } | OpenAiClientError::ReadFailed { .. } => {
            IpcError::new(
                "ai_config.test_unreachable",
                MessageKey::AiConfigTestUnreachable,
                no_params(),
                true,
            )
        }
        OpenAiClientError::MalformedEvent { .. } | OpenAiClientError::BufferOverflow { .. } => {
            IpcError::new(
                "ai_config.test_reply_unreadable",
                MessageKey::AiConfigTestReplyUnreadable,
                no_params(),
                false,
            )
        }
        other @ (OpenAiClientError::ClientBuildFailed { .. }
        | OpenAiClientError::ApiKeyHeaderInvalid { .. }
        | OpenAiClientError::StreamEndedWithoutDone
        | OpenAiClientError::EmptyReply { .. }) => IpcError::from(other),
    }
}

/// Seam for tests: runs one check through any [`ConnectionCheck`] implementation backed by the
/// real client error type. A listed-but-different model is an error, not an outcome.
pub async fn run_connection_check<P: ConnectionCheck<Error = OpenAiClientError>>(
    provider: &P,
    prepared: &PreparedConnectionCheck,
    should_cancel: &dyn Fn() -> bool,
) -> Result<ConnectionCheckWire, IpcError> {
    let request = ConnectionCheckRequest {
        endpoint: &prepared.endpoint,
        model: &prepared.model,
        api_key: prepared.api_key.as_deref(),
    };
    match provider
        .check_connection(request, should_cancel)
        .await
        .map_err(connection_check_error)?
    {
        ConnectionCheckOutcome::Connected { model_count } => {
            Ok(ConnectionCheckWire::Connected { model_count })
        }
        ConnectionCheckOutcome::Cancelled => Ok(ConnectionCheckWire::Cancelled),
        ConnectionCheckOutcome::ModelMissing => Err(IpcError::new(
            "ai_config.test_model_missing",
            MessageKey::AiConfigTestModelMissing,
            std::collections::BTreeMap::from([("model".to_owned(), prepared.model.clone())]),
            false,
        )),
    }
}

pub(crate) async fn send_prepared_connection_check(
    prepared: PreparedConnectionCheck,
    generation_state: ConnectionCheckGeneration,
    generation: u64,
) -> Result<ConnectionCheckWire, IpcError> {
    let join = tauri::async_runtime::spawn_blocking(move || {
        let provider = OpenAiChatClient::new();
        let should_cancel = || !generation_state.is_current(generation);
        tauri::async_runtime::handle().block_on(run_connection_check(
            &provider,
            &prepared,
            &should_cancel,
        ))
    });
    match join.await {
        Ok(outcome) => outcome,
        Err(_join_err) => Err(internal_failure_error()),
    }
}

/// Số liệu sử dụng + ước tính chi phí, hình dạng WIRE — Story 4.11. Mirror trực tiếp
/// [`crate::ports::translation_provider::TranslateUsage`] — `cost_usd` KHÔNG bị bỏ hẳn khỏi
/// JSON khi là `None` (§Always spec 4.11: "the token count appears and no money is shown at
/// all" là một trạng thái webview PHẢI đọc được, `cost_usd: null` trên dây — không phải một
/// trường vắng mặt mà webview không phân biệt được với "chưa từng gửi usage" ở tầng NGOÀI,
/// `usage: None`).
///
/// Tên trường giữ nguyên `snake_case` — `src/AGENTS.md`: "the RETURNED struct's fields stay
/// `snake_case`", đúng khuôn mọi wire khác của kho.
#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize)]
pub struct AiTranslateUsageWire {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
    pub cost_usd: Option<f64>,
}

impl From<TranslateUsage> for AiTranslateUsageWire {
    fn from(usage: TranslateUsage) -> Self {
        Self {
            prompt_tokens: usage.prompt_tokens,
            completion_tokens: usage.completion_tokens,
            total_tokens: usage.total_tokens,
            cost_usd: usage.cost_usd,
        }
    }
}

/// One planned call as the webview first learns of it.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct PlannedCallWire {
    pub call_index: u32,
    pub segment_ids: Vec<i64>,
}

/// One event of a grouped operation on its single `Channel` (AD-22, AD-60 rule 5). Every event
/// except `Planned` carries its `call_index`.
///
/// `Token` only shows progress. A sentence has text solely from `Received`, sent after its
/// call's stream ended; `Discarded` says why it has none. Errors are not events: they leave
/// through the command's `Err`.
#[derive(Debug, Clone, serde::Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum AiTranslateBatchEventWire {
    Planned {
        calls: Vec<PlannedCallWire>,
    },
    CallStarted {
        call_index: u32,
    },
    Token {
        call_index: u32,
        segment_id: i64,
        text: String,
    },
    Received {
        call_index: u32,
        segment_id: i64,
        text: String,
        has_han: bool,
        source_hash: String,
    },
    Discarded {
        call_index: u32,
        segment_id: i64,
        reason: &'static str,
    },
    /// `usage` is the call's own, `None` when the provider sent none. The operation total is the
    /// webview's sum over `call_index`.
    CallDone {
        call_index: u32,
        finish_reason: Option<String>,
        usage: Option<AiTranslateUsageWire>,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AiTranslateBatchOutcome {
    Done,
    Cancelled,
}

/// A sentence of a call about to be sent, with the snapshot facts its events carry.
#[derive(Debug, Clone)]
pub struct SentSegment {
    pub segment_id: i64,
    pub source_text: String,
    pub source_hash: String,
}

/// One call ready to send. `segments` are the live sentences in block order (block `n` is
/// element `n - 1`); `retired` are planned sentences that no longer exist, never sent. With no
/// live sentence nothing is sent.
pub struct CallToSend {
    pub request: PreparedTranslateCall,
    pub segments: Vec<SentSegment>,
    pub retired: Vec<i64>,
}

/// Where `run_translate_operation` gets each call, assembled just before it is sent.
pub trait CallSource {
    fn prepare_call(&mut self, call: &PlannedCall) -> Result<CallToSend, IpcError>;
    fn call_sent(&mut self, call_index: u32, usage: Option<&AiTranslateUsageWire>);
}

pub enum OperationError<E> {
    Provider {
        call_index: u32,
        first_segment_id: i64,
        last_segment_id: i64,
        err: E,
    },
    Prepare(IpcError),
}

/// **Async seam tests substitute a real `TranslationProvider` at.** Runs the planned calls one
/// after another on one `Channel`: `Planned` once, then per call `CallStarted`, `Token`s, one
/// `Received` or `Discarded` per planned sentence once the call's stream is over, and
/// `CallDone`. Cancel is asked before every call and between frames. A provider error ends the
/// operation after the failed call's valid blocks were kept; nothing is retried.
pub async fn run_translate_operation<P: TranslationProvider, S: CallSource>(
    provider: &P,
    plan: &[PlannedCall],
    source: &mut S,
    channel: &tauri::ipc::Channel<AiTranslateBatchEventWire>,
    should_cancel: &dyn Fn() -> bool,
) -> Result<AiTranslateBatchOutcome, OperationError<P::Error>> {
    let _ = channel.send(AiTranslateBatchEventWire::Planned {
        calls: plan
            .iter()
            .map(|call| PlannedCallWire {
                call_index: call.call_index,
                segment_ids: call.segment_ids(),
            })
            .collect(),
    });

    for call in plan {
        if should_cancel() {
            return Ok(AiTranslateBatchOutcome::Cancelled);
        }
        let planned_ids = call.segment_ids();
        let (Some(&first), Some(&last)) = (planned_ids.first(), planned_ids.last()) else {
            continue;
        };
        let call_index = call.call_index;
        let to_send = source.prepare_call(call).map_err(OperationError::Prepare)?;
        let live_ids: Vec<i64> = to_send.segments.iter().map(|s| s.segment_id).collect();

        if to_send.segments.is_empty() {
            let _ = channel.send(AiTranslateBatchEventWire::CallStarted { call_index });
            emit_verdicts(channel, call_index, &planned_ids, &to_send, &[]);
            let _ = channel.send(AiTranslateBatchEventWire::CallDone {
                call_index,
                finish_reason: None,
                usage: None,
            });
            continue;
        }

        let _ = channel.send(AiTranslateBatchEventWire::CallStarted { call_index });
        let request = TranslateRequest {
            endpoint: &to_send.request.endpoint,
            model: &to_send.request.model,
            temperature: to_send.request.temperature,
            max_tokens: to_send.request.max_tokens,
            api_key: &to_send.request.api_key,
            prompt: &to_send.request.prompt,
            system: to_send.request.system.as_deref(),
        };
        let mut reader = BlockStream::new();
        let result = {
            let mut on_token = |text: &str| {
                for piece in reader.push(text) {
                    let Some(&segment_id) = usize::try_from(piece.index)
                        .ok()
                        .and_then(|n| n.checked_sub(1))
                        .and_then(|i| live_ids.get(i))
                    else {
                        continue;
                    };
                    let _ = channel.send(AiTranslateBatchEventWire::Token {
                        call_index,
                        segment_id,
                        text: piece.text,
                    });
                }
            };
            provider
                .translate(request, &mut on_token, should_cancel)
                .await
        };
        let blocks = reader.finish();

        let (end, usage, finish_reason, stop) = match result {
            Ok(TranslateOutcome::Done {
                usage,
                finish_reason,
            }) => (
                Some(finish_reason.clone()),
                usage.map(AiTranslateUsageWire::from),
                finish_reason,
                None,
            ),
            Ok(TranslateOutcome::Cancelled) => (
                None,
                None,
                None,
                Some(Ok(AiTranslateBatchOutcome::Cancelled)),
            ),
            Err(err) if P::is_empty_reply(&err) => (Some(None), None, None, None),
            Err(err) => (
                None,
                None,
                None,
                Some(Err(OperationError::Provider {
                    call_index,
                    first_segment_id: first,
                    last_segment_id: last,
                    err,
                })),
            ),
        };
        let stream_end = match &end {
            Some(reason) => StreamEnd::Finished {
                finish_reason: reason.as_deref(),
            },
            None => StreamEnd::Interrupted,
        };
        let verdicts = reconcile(live_ids.len(), &blocks, &stream_end);
        emit_verdicts(channel, call_index, &planned_ids, &to_send, &verdicts);
        let _ = channel.send(AiTranslateBatchEventWire::CallDone {
            call_index,
            finish_reason,
            usage,
        });
        source.call_sent(call_index, usage.as_ref());
        if let Some(stopped) = stop {
            return stopped;
        }
    }

    Ok(AiTranslateBatchOutcome::Done)
}

fn emit_verdicts(
    channel: &tauri::ipc::Channel<AiTranslateBatchEventWire>,
    call_index: u32,
    planned_ids: &[i64],
    sent: &CallToSend,
    verdicts: &[Verdict],
) {
    for &segment_id in planned_ids {
        let live = sent
            .segments
            .iter()
            .position(|s| s.segment_id == segment_id);
        let event = match live.and_then(|i| verdicts.get(i).map(|v| (&sent.segments[i], v))) {
            Some((segment, Verdict::Received { text })) => AiTranslateBatchEventWire::Received {
                call_index,
                segment_id,
                text: text.clone(),
                has_han: has_han(&segment.source_text, text),
                source_hash: segment.source_hash.clone(),
            },
            Some((_, Verdict::Discarded { reason })) => AiTranslateBatchEventWire::Discarded {
                call_index,
                segment_id,
                reason: reason.as_str(),
            },
            None => AiTranslateBatchEventWire::Discarded {
                call_index,
                segment_id,
                reason: DiscardReason::Retired.as_str(),
            },
        };
        let _ = channel.send(event);
    }
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
    activity: AiTranslateActivityGuard,
) -> Result<SingleRun, IpcError> {
    send_with_cancel(prepared, channel, activity, move || {
        !generation_state.is_current(generation)
    })
    .await
}

async fn send_with_cancel(
    prepared: PreparedTranslateCall,
    channel: tauri::ipc::Channel<String>,
    activity: AiTranslateActivityGuard,
    should_cancel: impl Fn() -> bool + Send + 'static,
) -> Result<SingleRun, IpcError> {
    let join = tauri::async_runtime::spawn_blocking(move || {
        let _activity = activity;
        let provider = OpenAiChatClient::new();
        if should_cancel() {
            return Ok(SingleRun::Cancelled);
        }

        tauri::async_runtime::handle().block_on(run_translate_call(
            &provider,
            &prepared,
            &channel,
            &should_cancel,
        ))
    });

    match join.await {
        Ok(outcome) => outcome.map_err(IpcError::from),
        Err(_join_err) => Err(internal_failure_error()),
    }
}

/// Endpoint settings and key for resending a recorded call; `None` when not configured.
pub fn resolve_call_settings(
    global: Option<&Store>,
    open: Option<&OpenWork>,
) -> Result<Option<CallSettings>, IpcError> {
    let Some(ResolvedCallConfig {
        provider,
        endpoint,
        model,
        temperature,
        max_tokens,
        ..
    }) = resolve_call_config(global, open)?
    else {
        return Ok(None);
    };
    let Some(api_key) = read_api_key(&provider)? else {
        return Ok(None);
    };
    Ok(Some(CallSettings {
        endpoint,
        model,
        temperature,
        max_tokens,
        api_key,
    }))
}

/// How the operation reaches the managed stores. The app implements it with a fresh lock per
/// call, so no `OpenWorkState` guard lives across a network wait; tests may pass fixed stores.
pub trait StoreAccess {
    fn with_stores<T>(&self, f: impl FnOnce(Option<&Store>, Option<&OpenWork>) -> T) -> T;
    fn with_records<T>(&self, f: impl FnOnce(&AiCallRecordsState) -> T) -> Option<T>;
}

pub struct AppStores<R: tauri::Runtime>(pub tauri::AppHandle<R>);

impl<R: tauri::Runtime> StoreAccess for AppStores<R> {
    fn with_stores<T>(&self, f: impl FnOnce(Option<&Store>, Option<&OpenWork>) -> T) -> T {
        use tauri::Manager as _;

        let global = self.0.try_state::<Store>();
        let work_state = self.0.try_state::<OpenWorkState>();
        let guard = work_state
            .as_ref()
            .map(|s| s.lock().unwrap_or_else(std::sync::PoisonError::into_inner));
        let open = guard.as_ref().and_then(|g| g.as_ref());
        f(global.as_deref(), open)
    }

    fn with_records<T>(&self, f: impl FnOnce(&AiCallRecordsState) -> T) -> Option<T> {
        use tauri::Manager as _;

        self.0
            .try_state::<AiCallRecordsState>()
            .map(|state| f(state.inner()))
    }
}

fn mark_call_record_sent<A: StoreAccess>(
    access: &A,
    operation_id: u64,
    call_index: u32,
    usage: Option<&AiTranslateUsageWire>,
) {
    let Some(sent_at) = access.with_stores(|global, _| global.and_then(now_utc_iso8601)) else {
        return;
    };
    let usage = usage.copied();
    access.with_records(|records| {
        records
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .mark_sent(operation_id, call_index, &sent_at, usage);
    });
}

/// Assembles each call of a fresh operation from its snapshot just before it is sent, so a
/// sentence retired in the meantime is left out and the blocks stay numbered without a gap.
pub struct OperationCallSource<A: StoreAccess> {
    pub access: A,
    pub operation_id: u64,
    pub settings: CallSettings,
    pub context: OperationPromptContext,
    pub snapshot: std::collections::BTreeMap<i64, SnapshotRow>,
    pub chapter_id: i64,
}

impl<A: StoreAccess> CallSource for OperationCallSource<A> {
    fn prepare_call(&mut self, call: &PlannedCall) -> Result<CallToSend, IpcError> {
        let chapter = self
            .access
            .with_stores(|_, open| crate::commands::segment::read_open_chapter_segments(open))?;
        let alive: std::collections::BTreeSet<i64> = if chapter.chapter_id == self.chapter_id {
            chapter
                .segments
                .iter()
                .filter(|s| !s.is_omitted)
                .map(|s| s.id)
                .collect()
        } else {
            std::collections::BTreeSet::new()
        };

        let mut specs: Vec<CallSpec<'_>> = Vec::new();
        let mut segments: Vec<SentSegment> = Vec::new();
        let mut context_ids: Vec<i64> = Vec::new();
        let mut retired: Vec<i64> = Vec::new();
        for entry in &call.entries {
            match *entry {
                CallEntry::Context(id) => {
                    if let Some(row) = self.snapshot.get(&id) {
                        context_ids.push(id);
                        specs.push(CallSpec::Context {
                            source_text: &row.source_text,
                            target_text: &row.target_text,
                        });
                    }
                }
                CallEntry::Translate(id) => match self.snapshot.get(&id) {
                    Some(row) if alive.contains(&id) => {
                        specs.push(CallSpec::Segment {
                            source_text: &row.source_text,
                        });
                        segments.push(SentSegment {
                            segment_id: id,
                            source_text: row.source_text.clone(),
                            source_hash: source_text_hash(&row.source_text),
                        });
                    }
                    _ => retired.push(id),
                },
            }
        }

        let system = system_with_block_format(self.context.resolved_system());
        let (prompt, ledger) = if segments.is_empty() {
            (String::new(), None)
        } else {
            let (prompt, ledger) = assemble_call(&self.context, &specs)?;
            (prompt, Some(ledger))
        };
        if let Some(ledger) = ledger {
            let record = AiCallRecord {
                operation_id: self.operation_id,
                call_index: call.call_index,
                stamp: next_call_record_stamp(),
                prompt: prompt.clone(),
                system: system.clone(),
                segments: segments
                    .iter()
                    .map(|s| RecordedSegment {
                        segment_id: s.segment_id,
                        source_text: s.source_text.clone(),
                        source_hash: s.source_hash.clone(),
                    })
                    .collect(),
                context_segment_ids: context_ids,
                ledger,
                chapter_id: self.chapter_id,
                prompt_set_name: self.context.prompt_set_name().to_owned(),
                prompt_set_tier: self.context.prompt_set_tier(),
                model: self.settings.model.clone(),
                sent_at: None,
                sent_usage: None,
            };
            self.access.with_records(|records| {
                records
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner)
                    .put(record);
            });
        }
        Ok(CallToSend {
            request: self.settings.request(prompt, Some(system)),
            segments,
            retired,
        })
    }

    fn call_sent(&mut self, call_index: u32, usage: Option<&AiTranslateUsageWire>) {
        mark_call_record_sent(&self.access, self.operation_id, call_index, usage);
    }
}

/// Sends a recorded call again, byte for byte (FR145): the prompt and system come from the
/// record, nothing is assembled.
pub struct ResendCallSource<A: StoreAccess> {
    pub access: A,
    pub operation_id: u64,
    pub settings: CallSettings,
    pub record: AiCallRecord,
}

impl<A: StoreAccess> CallSource for ResendCallSource<A> {
    fn prepare_call(&mut self, _call: &PlannedCall) -> Result<CallToSend, IpcError> {
        Ok(CallToSend {
            request: self
                .settings
                .request(self.record.prompt.clone(), Some(self.record.system.clone())),
            segments: self
                .record
                .segments
                .iter()
                .map(|s| SentSegment {
                    segment_id: s.segment_id,
                    source_text: s.source_text.clone(),
                    source_hash: s.source_hash.clone(),
                })
                .collect(),
            retired: Vec::new(),
        })
    }

    fn call_sent(&mut self, call_index: u32, usage: Option<&AiTranslateUsageWire>) {
        mark_call_record_sent(&self.access, self.operation_id, call_index, usage);
    }
}

/// The plan of a resent call: the recorded call alone, under its original `call_index`.
pub fn resend_plan(record: &AiCallRecord) -> Vec<PlannedCall> {
    vec![PlannedCall {
        call_index: record.call_index,
        entries: record
            .segments
            .iter()
            .map(|s| CallEntry::Translate(s.segment_id))
            .collect(),
    }]
}

/// Maps the end of an operation to what the command returns.
pub fn operation_result_wire(
    result: Result<AiTranslateBatchOutcome, OperationError<OpenAiClientError>>,
) -> Result<AiTranslateOutcomeWire, IpcError> {
    match result {
        Ok(AiTranslateBatchOutcome::Cancelled) => Ok(AiTranslateOutcomeWire::Cancelled),
        Ok(AiTranslateBatchOutcome::Done) => Ok(AiTranslateOutcomeWire::Done {
            usage: None,
            maybe_truncated: false,
        }),
        Err(OperationError::Provider {
            call_index,
            first_segment_id,
            last_segment_id,
            err,
        }) => Err(batch_stopped_error(
            call_index,
            first_segment_id,
            last_segment_id,
            err,
        )),
        Err(OperationError::Prepare(err)) => Err(err),
    }
}

/// Runs the operation on a blocking-pool thread (the translate future is not `Send`, see the
/// file header).
async fn send_prepared_operation<S: CallSource + Send + 'static>(
    mut source: S,
    plan: Vec<PlannedCall>,
    channel: tauri::ipc::Channel<AiTranslateBatchEventWire>,
    generation_state: AiTranslateGeneration,
    generation: u64,
    activity: AiTranslateActivityGuard,
) -> Result<AiTranslateOutcomeWire, IpcError> {
    let join = tauri::async_runtime::spawn_blocking(move || {
        let _activity = activity;
        let provider = OpenAiChatClient::new();
        let should_cancel = || !generation_state.is_current(generation);
        tauri::async_runtime::handle().block_on(run_translate_operation(
            &provider,
            &plan,
            &mut source,
            &channel,
            &should_cancel,
        ))
    });
    match join.await {
        Ok(result) => operation_result_wire(result),
        Err(_join_err) => Err(batch_panicked_error()),
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

/// Kết quả cuối của MỘT lượt gọi [`wire::ai_translate_segment`] **hoặc** [`wire::ai_translate_batch`]
/// (Story 4.9, Phase 2: tái dùng NGUYÊN VẸN — Code Map spec 4.9: "the three values are exactly
/// right and a second enum would drift") — BA trong năm giá trị trạng thái toàn epic (§Always
/// spec 4.8): `generating` không có mặt ở đây (nó là trạng thái webview tự giữ TRONG LÚC lời gọi
/// này chưa trả về — Phase 3), và `error` đi qua `Result::Err(IpcError)` của chính lệnh này,
/// không qua biến thể nào ở đây.
#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize)]
#[serde(tag = "state", rename_all = "snake_case")]
pub enum AiTranslateOutcomeWire {
    /// `endpoint`/`model` rỗng, hoặc chưa có khoá API nào lưu — KHÔNG một lượt gọi mạng nào đã
    /// chạy (I/O Matrix spec 4.8).
    NotConfigured,
    /// Provider gửi khung kết thúc hợp lệ.
    ///
    /// `usage` — Story 4.11: mang số liệu của LƯỢT DỊCH MỘT SEGMENT (`wire::ai_translate_segment`,
    /// `Channel<String>` không có chỗ để đính một số, xem §Design Notes spec 4.11 "Why the two
    /// paths carry it differently"). Lượt LÔ (`wire::ai_translate_batch`) LUÔN đặt `None` ở
    /// đây — tổng của lô đi qua từng [`AiTranslateBatchEventWire::Done`], không qua giá trị cuối
    /// dùng chung này (hai wire khác hình dạng, không nên đọc `usage` ở đây cho một lô).
    ///
    /// `maybe_truncated`: the provider sent no `finish_reason` or one other than `"stop"`; only
    /// the one-sentence path can set it.
    Done {
        usage: Option<AiTranslateUsageWire>,
        maybe_truncated: bool,
    },
    /// Huỷ giữa chừng — token đã nhận ở lại trên màn hình webview, không khung nào gửi thêm.
    Cancelled,
}

/// Ánh xạ THUẦN `TranslateOutcome -> AiTranslateOutcomeWire` cho lượt dịch MỘT SEGMENT — tách
/// khỏi `wire::ai_translate_segment` (một `#[tauri::command]` đòi `AppHandle` thật, không gọi
/// được từ một test không dựng webview) đúng lý do mọi hàm thuần khác của tệp này tách
/// (`prepare_translate_call`, `run_translate_call`, …): để một test gọi được TRỰC TIẾP.
///
/// 🔴 **THÊM (rà soát coordinator, Story 4.11)** — trước hàm này, không một ca nào canh rằng
/// `usage` THẬT SỰ đi ra dây cho lượt dịch MỘT segment: thay `usage: usage.map(AiTranslateUsageWire::from)`
/// bằng `usage: None` (đúng hình dạng nhánh `ai_translate_batch` hợp lệ đứng NGAY BÊN CẠNH —
/// một lỗi copy-paste dễ mắc) vẫn qua sạch `cargo test`/`npx vitest run`/mọi gate. Hàm này là
/// SEAM để `tests/ai_translate_contract.rs` đối chứng đúng phép ánh xạ đó mà không cần một
/// `AppHandle`.
pub fn single_run_outcome_wire(outcome: SingleRun) -> AiTranslateOutcomeWire {
    match outcome {
        SingleRun::Cancelled => AiTranslateOutcomeWire::Cancelled,
        SingleRun::Done {
            usage,
            finish_reason,
            block_open,
        } => AiTranslateOutcomeWire::Done {
            usage: usage.map(AiTranslateUsageWire::from),
            maybe_truncated: block_open || finish_reason.as_deref() != Some("stop"),
        },
    }
}

/// Ba vỏ `#[tauri::command]` (Story 4.9, Phase 2 thêm `ai_translate_batch` bên cạnh hai vỏ
/// 4.8). **Không một quy tắc nào sống ở đây** ngoài việc mở/thả khoá đúng lúc (xem doc-comment
/// đầu tệp).
pub mod wire {
    use super::AiTranslateUsageWire;
    use super::{
        AiTranslateBatchEventWire, AiTranslateGeneration, AiTranslateOutcomeWire, AiTranslateScope,
        AppStores, OperationCallSource, PrepareOperationOutcome, PrepareOutcome, ResendCallSource,
        SingleRun, StoreAccess, prepare_resend_call, prepare_translate_call,
        prepare_translate_operation, resend_plan, send_prepared_operation,
        send_prepared_translate_call, single_run_outcome_wire,
    };
    use super::{
        ConnectionCheckGeneration, ConnectionCheckPrep, ConnectionCheckWire, QuickBatchOutcomeWire,
        QuickBlockEventWire, QuickLookupWire, QuickPurgeScopeWire, QuickPurgeWire, QuickStatsWire,
        QuickSurface, QuickTranslateGeneration, QuickTranslateOutcomeWire, internal_failure_error,
        prepare_connection_check, prepare_context_reference_call, prepare_quick_call,
        quick_count_older_wire, quick_lookup_many_wire, quick_lookup_wire, quick_purge_wire,
        quick_stats_wire, quick_translate, send_prepared_connection_check,
    };
    use crate::commands::aiprompt::{
        AiCallRecordsState, LastAssembledPromptState, mark_prompt_as_sent,
    };
    use crate::commands::project::OpenWorkState;
    use crate::core::ai::client::OpenAiChatClient;
    use crate::core::aiconfig::AiConfigTier;
    use crate::core::i18n::{IpcError, state_missing};
    use crate::core::quick_translation::QuickTranslationStore;
    use crate::core::store::Store;

    /// Vỏ IPC của lượt dịch một segment — `async fn` LITERAL (xem doc-comment đầu tệp cho lý
    /// do và cơ chế). `try_state`, không `state()` — cùng lý do mọi vỏ khác của kho.
    ///
    /// 🔴 Khoá `OpenWorkState` được mở và THẢ trong khối `{ ... }` dưới đây, TRƯỚC bất kỳ
    /// `.await` nào — xem doc-comment đầu tệp.
    #[tauri::command]
    pub async fn ai_translate_segment<R: tauri::Runtime>(
        app: tauri::AppHandle<R>,
        segment_id: i64,
        prompt_set_name: Option<String>,
        channel: tauri::ipc::Channel<String>,
    ) -> Result<AiTranslateOutcomeWire, IpcError> {
        use tauri::Manager as _;

        let Some(record_state) = app.try_state::<LastAssembledPromptState>() else {
            return Err(state_missing(
                "segment",
                "LastAssembledPromptState",
                "ai_translate.record_state_missing",
            ));
        };

        // Taken before the slow prepare so a cancel arriving during it is not overwritten.
        let Some(generation_state) = app.try_state::<AiTranslateGeneration>() else {
            return Err(state_missing(
                "segment",
                "AiTranslateGeneration",
                "ai_translate.generation_state_missing",
            ));
        };
        let generation_state = generation_state.inner().clone();
        let generation = generation_state.next();
        let activity_app = app.clone();
        let activity = generation_state.begin_activity(move |snapshot| {
            use tauri::Emitter as _;
            let _ = activity_app.emit("ai-translate-activity-v1", snapshot);
        });

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

        // `prepared.model`/`prepared.prompt` doc TRUOC khi `prepared` bi MOVE lam tham so cua
        // `send_prepared_translate_call` ngay duoi day -- can hai gia tri nay SAU luot `.await`
        // do (de ghi vao ban ghi cua 4.7's producer), luc `prepared` da khong con nua. `prompt_
        // sent` la CHUOI DUNG DA GUI (khong phai doc lai `record_state` sau await -- ban ghi do
        // co the da bi mot lot lap rap MOI ghi de trong luc dang cho mang, xem doc-comment
        // `mark_prompt_as_sent`), nen day la moc so DUY NHAT dung de quyet co stamp hay khong.
        let model_sent = prepared.model.clone();
        let prompt_sent = prepared.prompt.clone();
        let system_sent = prepared.system.clone();
        let outcome =
            send_prepared_translate_call(prepared, channel, generation_state, generation, activity)
                .await;

        match outcome {
            Ok(SingleRun::Cancelled) => Ok(single_run_outcome_wire(SingleRun::Cancelled)),
            Ok(SingleRun::Done {
                usage,
                finish_reason,
                block_open,
            }) => {
                if let Some(global) = app.try_state::<Store>()
                    && let Some(sent_at) = super::now_utc_iso8601(&global)
                {
                    mark_prompt_as_sent(
                        record_state.inner(),
                        segment_id,
                        &prompt_sent,
                        system_sent.as_deref(),
                        &model_sent,
                        &sent_at,
                        usage.map(AiTranslateUsageWire::from),
                    );
                }
                Ok(single_run_outcome_wire(SingleRun::Done {
                    usage,
                    finish_reason,
                    block_open,
                }))
            }
            Err(err) => Err(err),
        }
    }

    /// Resends the recorded prompt unchanged (FR145): no webview text goes in, nothing is
    /// assembled again, and the result only flows to the channel. Same lock discipline and
    /// generation counter as [`ai_translate_segment`].
    #[tauri::command]
    pub async fn ai_translate_resend<R: tauri::Runtime>(
        app: tauri::AppHandle<R>,
        channel: tauri::ipc::Channel<String>,
    ) -> Result<AiTranslateOutcomeWire, IpcError> {
        use tauri::Manager as _;

        let Some(record_state) = app.try_state::<LastAssembledPromptState>() else {
            return Err(state_missing(
                "resend",
                "LastAssembledPromptState",
                "ai_translate.record_state_missing",
            ));
        };

        let Some(generation_state) = app.try_state::<AiTranslateGeneration>() else {
            return Err(state_missing(
                "resend",
                "AiTranslateGeneration",
                "ai_translate.generation_state_missing",
            ));
        };
        let generation_state = generation_state.inner().clone();
        let generation = generation_state.next();
        let activity_app = app.clone();
        let activity = generation_state.begin_activity(move |snapshot| {
            use tauri::Emitter as _;
            let _ = activity_app.emit("ai-translate-activity-v1", snapshot);
        });

        let (segment_id, prepared) = {
            let global = app.try_state::<Store>();
            let work_state = app.try_state::<OpenWorkState>();
            let guard = work_state
                .as_ref()
                .map(|s| s.lock().unwrap_or_else(std::sync::PoisonError::into_inner));
            let open = guard.as_ref().and_then(|g| g.as_ref());

            prepare_resend_call(global.as_deref(), open, record_state.inner())?
        };

        let prepared = match prepared {
            PrepareOutcome::NotConfigured => return Ok(AiTranslateOutcomeWire::NotConfigured),
            PrepareOutcome::Ready(p) => p,
        };

        let model_sent = prepared.model.clone();
        let prompt_sent = prepared.prompt.clone();
        let system_sent = prepared.system.clone();
        let outcome =
            send_prepared_translate_call(prepared, channel, generation_state, generation, activity)
                .await;

        match outcome {
            Ok(SingleRun::Cancelled) => Ok(single_run_outcome_wire(SingleRun::Cancelled)),
            Ok(SingleRun::Done {
                usage,
                finish_reason,
                block_open,
            }) => {
                if let Some(global) = app.try_state::<Store>()
                    && let Some(sent_at) = super::now_utc_iso8601(&global)
                {
                    mark_prompt_as_sent(
                        record_state.inner(),
                        segment_id,
                        &prompt_sent,
                        system_sent.as_deref(),
                        &model_sent,
                        &sent_at,
                        usage.map(AiTranslateUsageWire::from),
                    );
                }
                Ok(single_run_outcome_wire(SingleRun::Done {
                    usage,
                    finish_reason,
                    block_open,
                }))
            }
            Err(err) => Err(err),
        }
    }

    /// Grouped translation of a selection or of the open chapter (AD-60): Rust plans the calls,
    /// sends them one after another and reports on one `Channel`. Same `Future: Send`
    /// mechanism and lock discipline as [`ai_translate_segment`]; one `AiTranslateGeneration`
    /// for every translate run, so starting an operation supersedes a running one.
    ///
    /// `scope` travels as `{ kind: "selection", segment_ids: [..] }` or `{ kind: "chapter" }`;
    /// the keys inside it are `snake_case`.
    #[tauri::command]
    pub async fn ai_translate_batch<R: tauri::Runtime>(
        app: tauri::AppHandle<R>,
        scope: AiTranslateScope,
        prompt_set_name: Option<String>,
        channel: tauri::ipc::Channel<AiTranslateBatchEventWire>,
    ) -> Result<AiTranslateOutcomeWire, IpcError> {
        use tauri::Manager as _;

        if app.try_state::<AiCallRecordsState>().is_none() {
            return Err(state_missing(
                "batch",
                "AiCallRecordsState",
                "ai_translate.record_state_missing",
            ));
        }

        // Taken before the slow prepare so a cancel arriving during it is not overwritten.
        let Some(generation_state) = app.try_state::<AiTranslateGeneration>() else {
            return Err(state_missing(
                "batch",
                "AiTranslateGeneration",
                "ai_translate.generation_state_missing",
            ));
        };
        let generation_state = generation_state.inner().clone();
        let generation = generation_state.next();
        let activity_app = app.clone();
        let activity = generation_state.begin_activity(move |snapshot| {
            use tauri::Emitter as _;
            let _ = activity_app.emit("ai-translate-activity-v1", snapshot);
        });

        let prepared = AppStores(app.clone()).with_stores(|global, open| {
            prepare_translate_operation(global, open, prompt_set_name.as_deref(), &scope)
        })?;

        let prepared = match prepared {
            PrepareOperationOutcome::NotConfigured => {
                return Ok(AiTranslateOutcomeWire::NotConfigured);
            }
            PrepareOperationOutcome::Ready(p) => *p,
        };
        let Some(context) = prepared.context else {
            let _ = channel.send(AiTranslateBatchEventWire::Planned { calls: Vec::new() });
            return Ok(AiTranslateOutcomeWire::Done {
                usage: None,
                maybe_truncated: false,
            });
        };

        let source = OperationCallSource {
            access: AppStores(app.clone()),
            operation_id: generation,
            settings: prepared.settings,
            context,
            snapshot: prepared.snapshot.into_iter().map(|r| (r.id, r)).collect(),
            chapter_id: prepared.chapter_id,
        };
        send_prepared_operation(
            source,
            prepared.plan,
            channel,
            generation_state,
            generation,
            activity,
        )
        .await
    }

    /// Sends one recorded call of an earlier operation again, unchanged (FR145), and reads the
    /// reply by the same rules. The events are those of a one-call operation that keeps the
    /// original `call_index`.
    #[tauri::command]
    pub async fn ai_translate_resend_call<R: tauri::Runtime>(
        app: tauri::AppHandle<R>,
        operation_id: u64,
        call_index: u32,
        channel: tauri::ipc::Channel<AiTranslateBatchEventWire>,
    ) -> Result<AiTranslateOutcomeWire, IpcError> {
        use tauri::Manager as _;

        let Some(records) = app.try_state::<AiCallRecordsState>() else {
            return Err(state_missing(
                "resend_call",
                "AiCallRecordsState",
                "ai_translate.record_state_missing",
            ));
        };
        let Some(generation_state) = app.try_state::<AiTranslateGeneration>() else {
            return Err(state_missing(
                "resend_call",
                "AiTranslateGeneration",
                "ai_translate.generation_state_missing",
            ));
        };
        let generation_state = generation_state.inner().clone();
        let generation = generation_state.next();
        let activity_app = app.clone();
        let activity = generation_state.begin_activity(move |snapshot| {
            use tauri::Emitter as _;
            let _ = activity_app.emit("ai-translate-activity-v1", snapshot);
        });

        let record = records
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .get(operation_id, call_index)
            .cloned();
        let Some(record) = record else {
            return Err(super::no_prompt_to_resend());
        };

        let settings = AppStores(app.clone()).with_stores(super::resolve_call_settings)?;
        let Some(settings) = settings else {
            return Ok(AiTranslateOutcomeWire::NotConfigured);
        };

        let plan = resend_plan(&record);
        let source = ResendCallSource {
            access: AppStores(app.clone()),
            operation_id,
            settings,
            record,
        };
        send_prepared_operation(
            source,
            plan,
            channel,
            generation_state,
            generation,
            activity,
        )
        .await
    }

    /// Vỏ IPC huỷ lượt dịch đang chạy — bơm thế hệ lên MỘT, làm thế hệ đang chạy (nếu có)
    /// không còn là thế hệ hiện hành. Không đọc/ghi `Store` nào — chỉ một `AtomicU64`, nên
    /// không cần `(async)`. Dùng CHUNG cho cả lượt dịch một segment lẫn một lô (Decision "one
    /// `AiTranslateGeneration` and it stays one", spec 4.9).
    #[tauri::command]
    pub fn ai_translate_cancel<R: tauri::Runtime>(app: tauri::AppHandle<R>) {
        use tauri::Manager as _;

        let Some(generation_state) = app.try_state::<AiTranslateGeneration>() else {
            eprintln!(
                "ai_translate[cancel] AiTranslateGeneration chua duoc quan ly -- loi cau hinh setup()"
            );
            return;
        };
        generation_state.next();
    }

    #[tauri::command]
    pub fn ai_translate_activity<R: tauri::Runtime>(
        app: tauri::AppHandle<R>,
    ) -> Result<super::AiTranslateActivityWire, IpcError> {
        use tauri::Manager as _;
        let state = app.try_state::<AiTranslateGeneration>().ok_or_else(|| {
            state_missing(
                "activity",
                "AiTranslateGeneration",
                "ai_translate.generation_state_missing",
            )
        })?;
        Ok(state.activity())
    }

    #[tauri::command]
    pub async fn ai_context_reference_translate<R: tauri::Runtime>(
        app: tauri::AppHandle<R>,
        context_example: String,
        trigger: super::QuickTranslateTrigger,
        run_id: u64,
        retranslate: bool,
        channel: tauri::ipc::Channel<String>,
    ) -> Result<QuickTranslateOutcomeWire, IpcError> {
        use tauri::Manager as _;

        let real_activity = if trigger == super::QuickTranslateTrigger::Auto {
            ai_translate_activity(app.clone())?
        } else {
            super::AiTranslateActivityWire::default()
        };
        let global = app.try_state::<Store>();
        if let Some(outcome) = super::quick_auto_gate(global.as_deref(), &real_activity, trigger)? {
            return Ok(outcome);
        }

        let Some(generation_state) = app.try_state::<QuickTranslateGeneration>() else {
            return Err(state_missing(
                "context_reference",
                "QuickTranslateGeneration",
                "ai_translate.generation_state_missing",
            ));
        };
        let generation_state = generation_state.inner().clone();
        let Some(ticket) = generation_state.begin(QuickSurface::GlossaryQueue, run_id) else {
            return Ok(QuickTranslateOutcomeWire::Busy);
        };
        let generation = ticket.generation;

        let join = tauri::async_runtime::spawn_blocking(move || {
            let _ticket = ticket;
            let provider = OpenAiChatClient::new();
            let store = app.try_state::<QuickTranslationStore>();
            let prepare = || {
                let global = app.try_state::<Store>();
                prepare_context_reference_call(global.as_deref(), None, &context_example)
            };
            let should_cancel: std::sync::Arc<dyn Fn() -> bool + Send + Sync> =
                std::sync::Arc::new(move || !generation_state.is_current(generation));
            let mut on_token = |text: &str| {
                let _ = channel.send(text.to_owned());
            };
            tauri::async_runtime::handle().block_on(quick_translate(
                store.as_deref(),
                &provider,
                prepare,
                &context_example,
                retranslate,
                &mut on_token,
                should_cancel,
            ))
        });
        match join.await {
            Ok(outcome) => outcome.map_err(IpcError::from),
            Err(_join_err) => Err(internal_failure_error()),
        }
    }

    /// Translates the blocks the user sees on the import preview. Every text answered from the
    /// store costs no call; the rest go in one call, up to the batch caps (AD-64 rule 7).
    #[tauri::command]
    pub async fn quick_translate_blocks<R: tauri::Runtime>(
        app: tauri::AppHandle<R>,
        texts: Vec<String>,
        trigger: super::QuickTranslateTrigger,
        retranslate: bool,
        run_id: u64,
        channel: tauri::ipc::Channel<QuickBlockEventWire>,
    ) -> Result<QuickBatchOutcomeWire, IpcError> {
        use tauri::Manager as _;

        let real_activity = if trigger == super::QuickTranslateTrigger::Auto {
            ai_translate_activity(app.clone())?
        } else {
            super::AiTranslateActivityWire::default()
        };
        let global = app.try_state::<Store>();
        if let Some(outcome) = super::quick_auto_gate(global.as_deref(), &real_activity, trigger)? {
            return Ok(match outcome {
                QuickTranslateOutcomeWire::AutoOff => QuickBatchOutcomeWire::AutoOff,
                _ => QuickBatchOutcomeWire::Paused,
            });
        }

        let Some(generation_state) = app.try_state::<QuickTranslateGeneration>() else {
            return Err(state_missing(
                "quick_translate_blocks",
                "QuickTranslateGeneration",
                "ai_translate.generation_state_missing",
            ));
        };
        let generation_state = generation_state.inner().clone();
        let Some(ticket) = generation_state.begin(QuickSurface::ImportPreview, run_id) else {
            return Ok(QuickBatchOutcomeWire::Busy);
        };
        let generation = ticket.generation;

        let join = tauri::async_runtime::spawn_blocking(move || {
            let _ticket = ticket;
            let provider = OpenAiChatClient::new();
            let store = app.try_state::<QuickTranslationStore>();
            let prepare = || {
                let global = app.try_state::<Store>();
                prepare_quick_call(global.as_deref(), String::new(), None)
            };
            let should_cancel: std::sync::Arc<dyn Fn() -> bool + Send + Sync> =
                std::sync::Arc::new(move || !generation_state.is_current(generation));
            let mut on_event = |event: QuickBlockEventWire| {
                let _ = channel.send(event);
            };
            tauri::async_runtime::handle().block_on(super::quick_translate_blocks(
                store.as_deref(),
                &provider,
                prepare,
                &texts,
                retranslate,
                &mut on_event,
                should_cancel,
            ))
        });
        match join.await {
            Ok(outcome) => outcome.map_err(IpcError::from),
            Err(_join_err) => Err(internal_failure_error()),
        }
    }

    /// Cancels the quick-translation run named `run_id`; any other `run_id` does nothing.
    #[tauri::command]
    pub fn ai_context_reference_cancel<R: tauri::Runtime>(app: tauri::AppHandle<R>, run_id: u64) {
        use tauri::Manager as _;

        if let Some(generation_state) = app.try_state::<QuickTranslateGeneration>() {
            generation_state.cancel(run_id);
        }
    }

    /// The remembered quick translation of `text`, if any. Reads one row; writes nothing.
    #[tauri::command]
    pub fn quick_translation_lookup<R: tauri::Runtime>(
        app: tauri::AppHandle<R>,
        text: String,
    ) -> QuickLookupWire {
        use tauri::Manager as _;

        let store = app.try_state::<QuickTranslationStore>();
        quick_lookup_wire(store.as_deref(), &text)
    }

    /// The remembered quick translation of each text, in the order sent. Reads, writes nothing.
    #[tauri::command]
    pub fn quick_translation_lookup_many<R: tauri::Runtime>(
        app: tauri::AppHandle<R>,
        texts: Vec<String>,
    ) -> Vec<QuickLookupWire> {
        use tauri::Manager as _;

        let store = app.try_state::<QuickTranslationStore>();
        quick_lookup_many_wire(store.as_deref(), &texts)
    }

    /// Row count, size on disk and rows per model. Reads only.
    #[tauri::command]
    pub fn quick_translation_stats<R: tauri::Runtime>(
        app: tauri::AppHandle<R>,
    ) -> Result<QuickStatsWire, IpcError> {
        use tauri::Manager as _;

        let store = app.try_state::<QuickTranslationStore>();
        quick_stats_wire(store.as_deref())
    }

    /// Rows older than `days` days by `created_at`. `days < 1` is an error.
    #[tauri::command]
    pub fn quick_translation_count_older<R: tauri::Runtime>(
        app: tauri::AppHandle<R>,
        days: i64,
    ) -> Result<u64, IpcError> {
        use tauri::Manager as _;

        let store = app.try_state::<QuickTranslationStore>();
        quick_count_older_wire(store.as_deref(), days)
    }

    /// Cancels every quick-translation run, deletes the rows in `scope`, then shrinks the file.
    #[tauri::command(async)]
    pub fn quick_translation_purge<R: tauri::Runtime>(
        app: tauri::AppHandle<R>,
        scope: QuickPurgeScopeWire,
    ) -> Result<QuickPurgeWire, IpcError> {
        use tauri::Manager as _;

        let store = app.try_state::<QuickTranslationStore>();
        let runs = app.try_state::<QuickTranslateGeneration>();
        quick_purge_wire(store.as_deref(), runs.as_deref(), scope)
    }

    #[tauri::command]
    pub async fn quick_ai_config_test_connection<R: tauri::Runtime>(
        app: tauri::AppHandle<R>,
    ) -> Result<ConnectionCheckWire, IpcError> {
        use tauri::Manager as _;
        let state = app
            .try_state::<super::QuickConnectionCheckGeneration>()
            .ok_or_else(|| {
                state_missing(
                    "test_connection",
                    "QuickConnectionCheckGeneration",
                    "ai_config.generation_state_missing",
                )
            })?;
        let generation_state = state.0.clone();
        let generation = generation_state.next();
        let prep = {
            let global = app.try_state::<Store>();
            super::prepare_quick_connection_check(global.as_deref())?
        };
        let ConnectionCheckPrep::Ready(prepared) = prep else {
            return Ok(ConnectionCheckWire::NotConfigured);
        };
        send_prepared_connection_check(prepared, generation_state, generation).await
    }

    #[tauri::command]
    pub fn quick_ai_config_test_connection_cancel<R: tauri::Runtime>(app: tauri::AppHandle<R>) {
        use tauri::Manager as _;
        if let Some(state) = app.try_state::<super::QuickConnectionCheckGeneration>() {
            state.0.next();
        }
    }

    /// Checks the SAVED configuration of `tier` against the provider: `GET {endpoint}/models`
    /// (no tokens), then the saved model must be listed. Has its own `ConnectionCheckGeneration`:
    /// `ai_config_test_connection_cancel` cancels it, and it never touches a translate run. The Work lock is released before any `.await`.
    #[tauri::command]
    pub async fn ai_config_test_connection<R: tauri::Runtime>(
        app: tauri::AppHandle<R>,
        tier: AiConfigTier,
    ) -> Result<ConnectionCheckWire, IpcError> {
        use tauri::Manager as _;

        let Some(generation_state) = app.try_state::<ConnectionCheckGeneration>() else {
            return Err(state_missing(
                "test_connection",
                "ConnectionCheckGeneration",
                "ai_config.generation_state_missing",
            ));
        };
        let generation_state = generation_state.inner().clone();
        let generation = generation_state.next();

        let prep = {
            let global = app.try_state::<Store>();
            let work_state = app.try_state::<OpenWorkState>();
            let guard = work_state
                .as_ref()
                .map(|s| s.lock().unwrap_or_else(std::sync::PoisonError::into_inner));
            let open = guard.as_ref().and_then(|g| g.as_ref());
            prepare_connection_check(global.as_deref(), open, tier)?
        };
        let prepared = match prep {
            ConnectionCheckPrep::NotConfigured => return Ok(ConnectionCheckWire::NotConfigured),
            ConnectionCheckPrep::Ready(p) => p,
        };
        send_prepared_connection_check(prepared, generation_state, generation).await
    }

    /// Cancels the running connection check only. Reads and writes no store.
    #[tauri::command]
    pub fn ai_config_test_connection_cancel<R: tauri::Runtime>(app: tauri::AppHandle<R>) {
        use tauri::Manager as _;

        if let Some(generation_state) = app.try_state::<ConnectionCheckGeneration>() {
            generation_state.next();
        }
    }
}
