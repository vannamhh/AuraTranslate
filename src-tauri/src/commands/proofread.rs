//! Single-segment spelling/grammar scan over IPC (FR80, FR83, AD-22).
//!
//! Read-only: nothing here writes `project.db` or touches confirmation state. The reply is
//! parsed and every quoted phrase placed in Rust; the webview only draws what it is handed.
//! A pure layer takes `Option<&Store>`/`Option<&OpenWork>`; [`wire`] holds the thin commands.
//! The API key is read through `aitranslate::read_api_key`, never here (AD-13).

use crate::commands::aiprompt::segment_not_in_chapter;
use crate::commands::aitranslate::{
    AiTranslateUsageWire, PreparedTranslateCall, ResolvedCallConfig, read_api_key, resolve_call_config,
};
use crate::commands::project::OpenWork;
use crate::core::ai::client::OpenAiChatClient;
use crate::core::ai::proofread::{ProofreadFinding, ProofreadReplyError, locate_findings};
use crate::core::ai::rag::assemble_proofread_prompt;
use crate::core::i18n::{IpcError, MessageKey};
use crate::core::store::Store;
use crate::ports::translation_provider::{TranslateOutcome, TranslateRequest, TranslationProvider};

/// Own counter, so cancelling a translation never cancels a scan and the reverse.
#[derive(Debug, Clone, Default)]
pub struct ProofreadGeneration(std::sync::Arc<std::sync::atomic::AtomicU64>);

impl ProofreadGeneration {
    fn next(&self) -> u64 {
        self.0.fetch_add(1, std::sync::atomic::Ordering::AcqRel).wrapping_add(1)
    }

    fn is_current(&self, generation: u64) -> bool {
        self.0.load(std::sync::atomic::Ordering::Acquire) == generation
    }
}

/// Owned and ready to send. No `Debug`: it holds the exposed API key.
pub struct PreparedProofreadCall {
    pub call: PreparedTranslateCall,
    pub scanned_text: String,
}

pub enum ProofreadPrepareOutcome {
    NotConfigured,
    EmptyText,
    Ready(PreparedProofreadCall),
}

#[derive(Debug, Clone, PartialEq, serde::Serialize)]
#[serde(tag = "state", rename_all = "snake_case")]
pub enum AiProofreadOutcomeWire {
    NotConfigured,
    Done {
        usage: Option<AiTranslateUsageWire>,
        /// The exact `target_text` that was scanned; offsets in `findings` index into it.
        scanned_text: String,
        findings: Vec<ProofreadFinding>,
        unlocated: u32,
    },
    Cancelled,
}

pub enum ProofreadRunError<E> {
    Provider(E),
    ReplyMalformed,
}

pub fn reply_malformed_error() -> IpcError {
    IpcError::new(
        "ai_proofread.reply_malformed",
        MessageKey::AiProofreadReplyMalformed,
        std::collections::BTreeMap::new(),
        true,
    )
}

/// Order: Work open, configuration (empty endpoint/model is not configured), the segment in the
/// open Chapter, then the API key. Not configured is reached before any network call.
pub fn prepare_proofread_call(
    global: Option<&Store>,
    open: Option<&OpenWork>,
    segment_id: i64,
) -> Result<ProofreadPrepareOutcome, IpcError> {
    let Some(ResolvedCallConfig { open_work, endpoint, model, temperature, max_tokens, .. }) =
        resolve_call_config(global, open)?
    else {
        return Ok(ProofreadPrepareOutcome::NotConfigured);
    };

    let chapter = crate::commands::segment::read_open_chapter_segments(Some(open_work))?;
    let Some(row) = chapter.segments.iter().find(|s| s.id == segment_id) else {
        return Err(segment_not_in_chapter(segment_id, chapter.chapter_id));
    };
    if row.target_text.trim().is_empty() {
        return Ok(ProofreadPrepareOutcome::EmptyText);
    }

    let Some(api_key) = read_api_key()? else {
        return Ok(ProofreadPrepareOutcome::NotConfigured);
    };

    Ok(ProofreadPrepareOutcome::Ready(PreparedProofreadCall {
        call: PreparedTranslateCall {
            endpoint,
            model,
            temperature,
            max_tokens,
            api_key,
            prompt: assemble_proofread_prompt(&row.target_text),
        },
        scanned_text: row.target_text.clone(),
    }))
}

/// The seam tests substitute a fake `TranslationProvider` at. Tokens are forwarded as they land
/// and accumulated; the reply is parsed only when the stream ends cleanly.
pub async fn run_proofread_call<P: TranslationProvider>(
    provider: &P,
    prepared: &PreparedProofreadCall,
    channel: &tauri::ipc::Channel<String>,
    should_cancel: &dyn Fn() -> bool,
) -> Result<AiProofreadOutcomeWire, ProofreadRunError<P::Error>> {
    let request = TranslateRequest {
        endpoint: &prepared.call.endpoint,
        model: &prepared.call.model,
        temperature: prepared.call.temperature,
        max_tokens: prepared.call.max_tokens,
        api_key: &prepared.call.api_key,
        prompt: &prepared.call.prompt,
    };
    let mut reply = String::new();
    let outcome = {
        let mut on_token = |text: &str| {
            reply.push_str(text);
            let _ = channel.send(text.to_owned());
        };
        provider.translate(request, &mut on_token, should_cancel).await
    };

    match outcome {
        Err(err) => Err(ProofreadRunError::Provider(err)),
        Ok(TranslateOutcome::Cancelled) => Ok(AiProofreadOutcomeWire::Cancelled),
        Ok(TranslateOutcome::Done(usage)) => {
            let located = locate_findings(&reply, &prepared.scanned_text)
                .map_err(|ProofreadReplyError::Malformed| ProofreadRunError::ReplyMalformed)?;
            Ok(AiProofreadOutcomeWire::Done {
                usage: usage.map(AiTranslateUsageWire::from),
                scanned_text: prepared.scanned_text.clone(),
                findings: located.findings,
                unlocated: located.unlocated,
            })
        }
    }
}

/// Runs [`run_proofread_call`] on a blocking-pool thread: the provider future borrows non-`Send`
/// callbacks, while the command future must be `Send`.
async fn send_prepared_proofread_call(
    prepared: PreparedProofreadCall,
    channel: tauri::ipc::Channel<String>,
    generation_state: ProofreadGeneration,
    generation: u64,
) -> Result<AiProofreadOutcomeWire, IpcError> {
    let join = tauri::async_runtime::spawn_blocking(move || {
        let provider = OpenAiChatClient::new();
        let should_cancel = || !generation_state.is_current(generation);
        tauri::async_runtime::handle().block_on(run_proofread_call(
            &provider,
            &prepared,
            &channel,
            &should_cancel,
        ))
    });

    match join.await {
        Ok(Ok(outcome)) => Ok(outcome),
        Ok(Err(ProofreadRunError::Provider(err))) => Err(IpcError::from(err)),
        Ok(Err(ProofreadRunError::ReplyMalformed)) => Err(reply_malformed_error()),
        Err(_join_err) => Err(IpcError::new(
            "ai_translate.internal_failure",
            MessageKey::AiTranslateInternalFailure,
            std::collections::BTreeMap::new(),
            false,
        )),
    }
}

pub mod wire {
    use super::{
        AiProofreadOutcomeWire, ProofreadGeneration, ProofreadPrepareOutcome, prepare_proofread_call,
        send_prepared_proofread_call,
    };
    use crate::commands::aitranslate::state_missing;
    use crate::commands::project::OpenWorkState;
    use crate::core::i18n::IpcError;
    use crate::core::store::Store;

    /// The `OpenWorkState` lock is released before any `.await`.
    #[tauri::command]
    pub async fn ai_proofread_segment<R: tauri::Runtime>(
        app: tauri::AppHandle<R>,
        segment_id: i64,
        channel: tauri::ipc::Channel<String>,
    ) -> Result<AiProofreadOutcomeWire, IpcError> {
        use tauri::Manager as _;

        // Taken before the slow prepare so a cancel arriving during it is not overwritten.
        let Some(generation_state) = app.try_state::<ProofreadGeneration>() else {
            return Err(state_missing(
                "proofread",
                "ProofreadGeneration",
                "ai_proofread.generation_state_missing",
            ));
        };
        let generation_state = generation_state.inner().clone();
        let generation = generation_state.next();

        let prepared = {
            let global = app.try_state::<Store>();
            let work_state = app.try_state::<OpenWorkState>();
            let guard = work_state
                .as_ref()
                .map(|s| s.lock().unwrap_or_else(std::sync::PoisonError::into_inner));
            let open = guard.as_ref().and_then(|g| g.as_ref());
            prepare_proofread_call(global.as_deref(), open, segment_id)?
        };

        let prepared = match prepared {
            ProofreadPrepareOutcome::NotConfigured => return Ok(AiProofreadOutcomeWire::NotConfigured),
            ProofreadPrepareOutcome::EmptyText => {
                return Ok(AiProofreadOutcomeWire::Done {
                    usage: None,
                    scanned_text: String::new(),
                    findings: Vec::new(),
                    unlocated: 0,
                });
            }
            ProofreadPrepareOutcome::Ready(p) => p,
        };

        send_prepared_proofread_call(prepared, channel, generation_state, generation).await
    }

    #[tauri::command]
    pub fn ai_proofread_cancel(app: tauri::AppHandle) {
        use tauri::Manager as _;

        let Some(generation_state) = app.try_state::<ProofreadGeneration>() else {
            eprintln!("ai_proofread[cancel] ProofreadGeneration chua duoc quan ly -- loi cau hinh setup()");
            return;
        };
        generation_state.next();
    }
}
