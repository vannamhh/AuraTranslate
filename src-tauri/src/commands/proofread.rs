//! Batched proofread scan over a list of segments (FR80, FR82, FR83, AD-22).
//!
//! The scan is read-only: it writes neither `project.db` nor confirmation state. The commands in
//! [`wire`] after it write only the "not an error" memory and the scan count (AD-53 rule 7), never
//! `target_text`. Segments are
//! grouped into batches, one provider call per batch, run one after another; each batch that
//! lands is sent over the Channel with every quoted phrase already placed in Rust.
//! A pure layer takes `Option<&Store>`/`Option<&OpenWork>`; [`wire`] holds the thin commands.
//! The API key is read through `aitranslate::read_api_key`, never here (AD-13).

use crate::commands::aiprompt::segment_not_in_chapter;
use crate::commands::aitranslate::{
    AiTranslateUsageWire, PreparedTranslateCall, ResolvedCallConfig, read_api_key,
    resolve_call_config,
};
use crate::commands::project::OpenWork;
use crate::core::ai::client::OpenAiChatClient;
use crate::core::ai::proofread::{
    ProofreadFinding, ProofreadReplyError, ProofreadSignature, filter_remembered,
    locate_batch_findings, plan_batches, signature_of,
};
use crate::core::ai::rag::{ProofreadBlock, assemble_proofread_batch_prompt};
use crate::core::i18n::{IpcError, MessageKey};
use crate::core::segment::proofread_kind::ProofreadKind;
use crate::core::store::Store;
use crate::ports::translation_provider::{TranslateOutcome, TranslateRequest, TranslationProvider};

/// Own counter, so cancelling a translation never cancels a scan and the reverse.
#[derive(Debug, Clone, Default)]
pub struct ProofreadGeneration(std::sync::Arc<std::sync::atomic::AtomicU64>);

impl ProofreadGeneration {
    fn next(&self) -> u64 {
        self.0
            .fetch_add(1, std::sync::atomic::Ordering::AcqRel)
            .wrapping_add(1)
    }

    fn is_current(&self, generation: u64) -> bool {
        self.0.load(std::sync::atomic::Ordering::Acquire) == generation
    }
}

/// One segment of a scan, with the exact text that goes to the model.
pub struct PreparedProofreadSegment {
    pub segment_id: i64,
    pub scanned_text: String,
    pub has_source: bool,
}

/// One provider call: the prompt and the segments it covers, in prompt order.
/// No `Debug`: it holds the exposed API key.
pub struct PreparedProofreadBatch {
    pub call: PreparedTranslateCall,
    pub segments: Vec<PreparedProofreadSegment>,
}

pub struct PreparedProofreadRun {
    pub batches: Vec<PreparedProofreadBatch>,
    pub remembered: Vec<ProofreadSignature>,
    /// `proof_ignore.id` of each `remembered` entry, same order.
    pub remembered_ids: Vec<i64>,
}

pub enum ProofreadPrepareOutcome {
    NotConfigured,
    Ready(PreparedProofreadRun),
}

#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub struct AiProofreadSegmentWire {
    pub segment_id: i64,
    /// The exact `target_text` that was scanned; offsets in `findings` index into it.
    pub scanned_text: String,
    pub findings: Vec<ProofreadFinding>,
    pub unlocated: u32,
    /// Context-scoped phrases that occur more than once in `scanned_text`, so have no position.
    pub ambiguous: u32,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum AiProofreadEventWire {
    /// Sent once, before the first provider call: how much the scan will cover.
    Planned { segments: u32, batches: u32 },
    /// One batch came back and parsed; carries every segment of that batch, findings or not.
    Batch {
        usage: Option<AiTranslateUsageWire>,
        segments: Vec<AiProofreadSegmentWire>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ProofreadBlockedWire {
    pub ignore_id: i64,
    pub count: u32,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize)]
#[serde(tag = "state", rename_all = "snake_case")]
pub enum AiProofreadOutcomeWire {
    NotConfigured,
    /// `not_scanned`: segments of batches whose reply was unusable. `filtered`: findings the
    /// "not an error" memory silenced before they left Rust.
    Done {
        not_scanned: u32,
        filtered: u32,
        /// `filtered` split by the `proof_ignore` entry that silenced each finding.
        blocked: Vec<ProofreadBlockedWire>,
    },
    /// `not_scanned`: segments of the batch cut short and of every batch after it.
    Cancelled {
        not_scanned: u32,
    },
}

pub enum ProofreadRunError<E> {
    /// `not_scanned`: segments of the failing batch and of every batch after it.
    Provider { err: E, not_scanned: u32 },
    /// Every batch came back unusable; none parsed.
    ReplyMalformed { not_scanned: u32 },
}

pub fn reply_malformed_error() -> IpcError {
    IpcError::new(
        "ai_proofread.reply_malformed",
        MessageKey::AiProofreadReplyMalformed,
        std::collections::BTreeMap::new(),
        true,
    )
}

/// Order: Work open, configuration (empty endpoint/model is not configured), every id in the
/// open Chapter, then the API key (only when something is left to scan). Not configured is
/// reached before any network call. Segments come back in document order; omitted segments and
/// segments with a blank translation are left out.
pub fn prepare_proofread_call(
    global: Option<&Store>,
    open: Option<&OpenWork>,
    segment_ids: &[i64],
) -> Result<ProofreadPrepareOutcome, IpcError> {
    let Some(ResolvedCallConfig {
        open_work,
        endpoint,
        model,
        temperature,
        max_tokens,
        ..
    }) = resolve_call_config(global, open)?
    else {
        return Ok(ProofreadPrepareOutcome::NotConfigured);
    };

    let chapter = crate::commands::segment::read_open_chapter_segments(Some(open_work))?;
    let requested: std::collections::BTreeSet<i64> = segment_ids.iter().copied().collect();
    let known: std::collections::BTreeSet<i64> = chapter.segments.iter().map(|s| s.id).collect();
    if let Some(&missing) = requested.difference(&known).next() {
        return Err(segment_not_in_chapter(missing, chapter.chapter_id));
    }
    let rows: Vec<_> = chapter
        .segments
        .iter()
        .filter(|s| requested.contains(&s.id) && !s.is_omitted && !s.target_text.trim().is_empty())
        .collect();
    if rows.is_empty() {
        return Ok(ProofreadPrepareOutcome::Ready(PreparedProofreadRun {
            batches: Vec::new(),
            remembered: Vec::new(),
            remembered_ids: Vec::new(),
        }));
    }

    let Some(api_key) = read_api_key()? else {
        return Ok(ProofreadPrepareOutcome::NotConfigured);
    };

    let (remembered, remembered_ids) = read_remembered(open_work)?;
    let sizes: Vec<usize> = rows
        .iter()
        .map(|r| r.source_text.chars().count() + r.target_text.chars().count())
        .collect();
    let batches = plan_batches(&sizes)
        .into_iter()
        .map(|range| {
            let group = &rows[range];
            let blocks: Vec<ProofreadBlock<'_>> = group
                .iter()
                .map(|r| ProofreadBlock {
                    source_text: &r.source_text,
                    target_text: &r.target_text,
                })
                .collect();
            PreparedProofreadBatch {
                call: PreparedTranslateCall {
                    endpoint: endpoint.clone(),
                    model: model.clone(),
                    temperature,
                    max_tokens,
                    api_key: api_key.clone(),
                    prompt: assemble_proofread_batch_prompt(&blocks),
                },
                segments: group
                    .iter()
                    .map(|r| PreparedProofreadSegment {
                        segment_id: r.id,
                        scanned_text: r.target_text.clone(),
                        has_source: !r.source_text.trim().is_empty(),
                    })
                    .collect(),
            }
        })
        .collect();

    Ok(ProofreadPrepareOutcome::Ready(PreparedProofreadRun {
        batches,
        remembered,
        remembered_ids,
    }))
}

fn read_remembered(open: &OpenWork) -> Result<(Vec<ProofreadSignature>, Vec<i64>), IpcError> {
    Ok(open.store.read(|conn| {
        let mut stmt = conn.prepare(
            "SELECT kind, phrase, context_before, context_after, id FROM proof_ignore ORDER BY id",
        )?;
        let rows = stmt.query_map([], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, String>(3)?,
                row.get::<_, i64>(4)?,
            ))
        })?;
        let mut out = Vec::new();
        let mut ids = Vec::new();
        for row in rows {
            let (kind, phrase, before, after, id) = row?;
            if let Some(kind) = ProofreadKind::from_wire(&kind) {
                out.push(ProofreadSignature {
                    kind,
                    phrase,
                    before,
                    after,
                });
                ids.push(id);
            }
        }
        Ok((out, ids))
    })?)
}

/// The seam tests substitute a fake `TranslationProvider` at. `should_cancel` is asked before
/// each batch and inside the provider call. A reply that does not parse costs only its own
/// batch; a provider error stops the run.
pub async fn run_proofread_call<P: TranslationProvider>(
    provider: &P,
    run: &PreparedProofreadRun,
    channel: &tauri::ipc::Channel<AiProofreadEventWire>,
    should_cancel: &dyn Fn() -> bool,
) -> Result<AiProofreadOutcomeWire, ProofreadRunError<P::Error>> {
    let total: usize = run.batches.iter().map(|b| b.segments.len()).sum();
    let _ = channel.send(AiProofreadEventWire::Planned {
        segments: u32::try_from(total).unwrap_or(u32::MAX),
        batches: u32::try_from(run.batches.len()).unwrap_or(u32::MAX),
    });

    let mut not_scanned = 0usize;
    let mut filtered = 0u32;
    let mut blocked_by_entry = vec![0u32; run.remembered.len()];
    let mut parsed_batches = 0usize;
    for (index, batch) in run.batches.iter().enumerate() {
        let remaining = || {
            run.batches[index..]
                .iter()
                .map(|b| b.segments.len())
                .sum::<usize>()
        };
        let cancelled = |lost: usize| AiProofreadOutcomeWire::Cancelled {
            not_scanned: u32::try_from(not_scanned + lost).unwrap_or(u32::MAX),
        };
        if should_cancel() {
            return Ok(cancelled(remaining()));
        }

        let request = TranslateRequest {
            endpoint: &batch.call.endpoint,
            model: &batch.call.model,
            temperature: batch.call.temperature,
            max_tokens: batch.call.max_tokens,
            api_key: &batch.call.api_key,
            prompt: &batch.call.prompt,
        };
        let mut reply = String::new();
        let outcome = {
            let mut on_token = |text: &str| reply.push_str(text);
            provider
                .translate(request, &mut on_token, should_cancel)
                .await
        };

        let usage = match outcome {
            Err(err) => {
                return Err(ProofreadRunError::Provider {
                    err,
                    not_scanned: u32::try_from(not_scanned + remaining()).unwrap_or(u32::MAX),
                });
            }
            Ok(TranslateOutcome::Cancelled) => return Ok(cancelled(remaining())),
            Ok(TranslateOutcome::Done(usage)) => usage,
        };

        let texts: Vec<(&str, bool)> = batch
            .segments
            .iter()
            .map(|s| (s.scanned_text.as_str(), s.has_source))
            .collect();
        match locate_batch_findings(&reply, &texts) {
            Err(ProofreadReplyError::Malformed) => {
                not_scanned += batch.segments.len();
                let _ = channel.send(AiProofreadEventWire::Batch {
                    usage: usage.map(AiTranslateUsageWire::from),
                    segments: Vec::new(),
                });
            }
            Ok(located) => {
                parsed_batches += 1;
                let segments = batch
                    .segments
                    .iter()
                    .zip(located)
                    .map(|(segment, located)| {
                        let (findings, blocked) = filter_remembered(
                            &segment.scanned_text,
                            located.findings,
                            &run.remembered,
                        );
                        for (total, n) in blocked_by_entry.iter_mut().zip(blocked) {
                            *total = total.saturating_add(n);
                            filtered = filtered.saturating_add(n);
                        }
                        AiProofreadSegmentWire {
                            segment_id: segment.segment_id,
                            scanned_text: segment.scanned_text.clone(),
                            findings,
                            unlocated: located.unlocated,
                            ambiguous: located.ambiguous,
                        }
                    })
                    .collect();
                let _ = channel.send(AiProofreadEventWire::Batch {
                    usage: usage.map(AiTranslateUsageWire::from),
                    segments,
                });
            }
        }
    }

    let not_scanned = u32::try_from(not_scanned).unwrap_or(u32::MAX);
    if parsed_batches == 0 && not_scanned > 0 {
        return Err(ProofreadRunError::ReplyMalformed { not_scanned });
    }
    let blocked = run
        .remembered_ids
        .iter()
        .zip(blocked_by_entry)
        .filter(|(_, count)| *count > 0)
        .map(|(ignore_id, count)| ProofreadBlockedWire {
            ignore_id: *ignore_id,
            count,
        })
        .collect();
    Ok(AiProofreadOutcomeWire::Done {
        not_scanned,
        filtered,
        blocked,
    })
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct ProofreadIgnoreEntryWire {
    pub id: i64,
    pub kind: ProofreadKind,
    pub phrase: String,
    pub context_before: String,
    pub context_after: String,
    pub created_at: String,
    /// `None` for a row that predates the counter.
    pub blocked_count: Option<i64>,
}

fn scan_record_rejected() -> IpcError {
    IpcError::new(
        "ai_proofread.scan_record_rejected",
        MessageKey::AiProofreadScanRecordRejected,
        std::collections::BTreeMap::new(),
        false,
    )
}

/// Remembers the finding at `[start, end)` (UTF-16) of the segment as "not an error" for the
/// whole Work. The range is checked by [`crate::commands::segment::check_proofread_range`] and
/// the signature comes from [`signature_of`]; remembering twice is one row.
pub fn proofread_ignore_add(
    open: Option<&OpenWork>,
    segment_id: i64,
    expected_text: &str,
    start: u32,
    end: u32,
    expected_phrase: &str,
    kind: ProofreadKind,
) -> Result<(), IpcError> {
    let open = open.ok_or_else(crate::commands::chapter::no_work_open)?;
    let open_chapter_id = open.chapter_id;
    let (expected_text, expected_phrase) = (expected_text.to_owned(), expected_phrase.to_owned());

    open.store.write(move |tx| {
        let checked = match crate::commands::segment::check_proofread_range(
            tx,
            open_chapter_id,
            segment_id,
            &expected_text,
            start,
            end,
            &expected_phrase,
        )? {
            Ok(checked) => checked,
            Err(err) => return Ok(Err(err)),
        };
        let Some(signature) = signature_of(kind, &checked.text, checked.byte_start, checked.byte_end) else {
            return Ok(Err(crate::commands::segment::range_invalid(segment_id)));
        };
        tx.execute(
            "INSERT OR IGNORE INTO proof_ignore (kind, phrase, context_before, context_after, created_at, blocked_count) \
             VALUES (?1, ?2, ?3, ?4, strftime('%Y-%m-%dT%H:%M:%fZ','now'), 0)",
            (signature.kind.as_str(), &signature.phrase, &signature.before, &signature.after),
        )?;
        Ok(Ok(()))
    })?
}

/// Every remembered finding of the open Work, newest first.
pub fn proofread_ignore_list(
    open: Option<&OpenWork>,
) -> Result<Vec<ProofreadIgnoreEntryWire>, IpcError> {
    let open = open.ok_or_else(crate::commands::chapter::no_work_open)?;
    Ok(open.store.read(|conn| {
        let mut stmt = conn.prepare(
            "SELECT id, kind, phrase, context_before, context_after, created_at, blocked_count FROM proof_ignore \
             ORDER BY created_at DESC, id DESC",
        )?;
        let rows = stmt.query_map([], |row| {
            Ok((
                row.get::<_, i64>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, String>(3)?,
                row.get::<_, String>(4)?,
                row.get::<_, String>(5)?,
                row.get::<_, Option<i64>>(6)?,
            ))
        })?;
        let mut out = Vec::new();
        for row in rows {
            let (id, kind, phrase, context_before, context_after, created_at, blocked_count) =
                row?;
            if let Some(kind) = ProofreadKind::from_wire(&kind) {
                out.push(ProofreadIgnoreEntryWire {
                    id,
                    kind,
                    phrase,
                    context_before,
                    context_after,
                    created_at,
                    blocked_count,
                });
            }
        }
        Ok(out)
    })?)
}

/// Forgets one remembered finding; `false` when no such row.
pub fn proofread_ignore_remove(open: Option<&OpenWork>, id: i64) -> Result<bool, IpcError> {
    let open = open.ok_or_else(crate::commands::chapter::no_work_open)?;
    Ok(open
        .store
        .write(move |tx| Ok(tx.execute("DELETE FROM proof_ignore WHERE id = ?1", [id])? > 0))?)
}

/// Forgets every remembered finding of the open Work and returns how many rows went.
pub fn proofread_ignore_clear(open: Option<&OpenWork>) -> Result<u32, IpcError> {
    let open = open.ok_or_else(crate::commands::chapter::no_work_open)?;
    let removed = open
        .store
        .write(|tx| tx.execute("DELETE FROM proof_ignore", []))?;
    Ok(u32::try_from(removed).unwrap_or(u32::MAX))
}

/// Records one finished scan and raises the blocked counter of each entry that silenced findings,
/// in the same transaction. The counts come from the webview, so Rust checks only what it can:
/// `work_id` is the open Work, `scope` is one of the closed set, the counts are not negative.
/// A row with a `NULL` counter stays `NULL`.
pub fn proofread_scan_record(
    open: Option<&OpenWork>,
    work_id: &str,
    scope: &str,
    reported: i64,
    filtered: i64,
    blocked: &[ProofreadBlockedWire],
) -> Result<(), IpcError> {
    let open = open.ok_or_else(crate::commands::chapter::no_work_open)?;
    if open.meta.work_id != work_id
        || !matches!(scope, "segment" | "selection" | "chapter")
        || reported < 0
        || filtered < 0
    {
        return Err(scan_record_rejected());
    }
    let scope = scope.to_owned();
    let blocked = blocked.to_vec();
    open.store.write(move |tx| {
        tx.execute(
            "INSERT INTO proof_scan (scope, scanned_at, reported, filtered) \
             VALUES (?1, strftime('%Y-%m-%dT%H:%M:%fZ','now'), ?2, ?3)",
            (&scope, reported, filtered),
        )?;
        for entry in &blocked {
            tx.execute(
                "UPDATE proof_ignore SET blocked_count = blocked_count + ?1 \
                 WHERE id = ?2 AND blocked_count IS NOT NULL",
                (entry.count, entry.ignore_id),
            )?;
        }
        Ok(())
    })?;
    Ok(())
}

fn with_not_scanned(base: IpcError, not_scanned: u32) -> IpcError {
    let mut params = base.params().clone();
    params.insert("not_scanned".to_owned(), not_scanned.to_string());
    IpcError::new(base.code(), base.message_key(), params, base.retryable())
}

/// Runs [`run_proofread_call`] on a blocking-pool thread: the provider future borrows non-`Send`
/// callbacks, while the command future must be `Send`.
async fn send_prepared_proofread_call(
    run: PreparedProofreadRun,
    channel: tauri::ipc::Channel<AiProofreadEventWire>,
    generation_state: ProofreadGeneration,
    generation: u64,
) -> Result<AiProofreadOutcomeWire, IpcError> {
    let join = tauri::async_runtime::spawn_blocking(move || {
        let provider = OpenAiChatClient::new();
        let should_cancel = || !generation_state.is_current(generation);
        tauri::async_runtime::handle().block_on(run_proofread_call(
            &provider,
            &run,
            &channel,
            &should_cancel,
        ))
    });

    match join.await {
        Ok(Ok(outcome)) => Ok(outcome),
        Ok(Err(ProofreadRunError::Provider { err, not_scanned })) => {
            Err(with_not_scanned(IpcError::from(err), not_scanned))
        }
        Ok(Err(ProofreadRunError::ReplyMalformed { not_scanned })) => {
            Err(with_not_scanned(reply_malformed_error(), not_scanned))
        }
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
        AiProofreadEventWire, AiProofreadOutcomeWire, ProofreadGeneration,
        ProofreadIgnoreEntryWire, ProofreadPrepareOutcome, prepare_proofread_call,
        send_prepared_proofread_call,
    };
    use crate::commands::aitranslate::state_missing;
    use crate::commands::project::OpenWorkState;
    use crate::core::i18n::IpcError;
    use crate::core::segment::proofread_kind::ProofreadKind;
    use crate::core::store::Store;

    /// The `OpenWorkState` lock is released before any `.await`.
    #[tauri::command]
    pub async fn ai_proofread_segments<R: tauri::Runtime>(
        app: tauri::AppHandle<R>,
        segment_ids: Vec<i64>,
        channel: tauri::ipc::Channel<AiProofreadEventWire>,
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
            prepare_proofread_call(global.as_deref(), open, &segment_ids)?
        };

        let prepared = match prepared {
            ProofreadPrepareOutcome::NotConfigured => {
                return Ok(AiProofreadOutcomeWire::NotConfigured);
            }
            ProofreadPrepareOutcome::Ready(run) => run,
        };

        send_prepared_proofread_call(prepared, channel, generation_state, generation).await
    }

    fn with_open<T>(
        app: &tauri::AppHandle<impl tauri::Runtime>,
        job: impl FnOnce(Option<&crate::commands::project::OpenWork>) -> Result<T, IpcError>,
    ) -> Result<T, IpcError> {
        use tauri::Manager as _;

        let Some(state) = app.try_state::<OpenWorkState>() else {
            return job(None);
        };
        let guard = state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        job(guard.as_ref())
    }

    #[tauri::command]
    pub fn ai_proofread_ignore_add<R: tauri::Runtime>(
        app: tauri::AppHandle<R>,
        segment_id: i64,
        expected_text: String,
        start: u32,
        end: u32,
        expected_phrase: String,
        kind: ProofreadKind,
    ) -> Result<(), IpcError> {
        with_open(&app, |open| {
            super::proofread_ignore_add(
                open,
                segment_id,
                &expected_text,
                start,
                end,
                &expected_phrase,
                kind,
            )
        })
    }

    #[tauri::command]
    pub fn ai_proofread_ignore_list<R: tauri::Runtime>(
        app: tauri::AppHandle<R>,
    ) -> Result<Vec<ProofreadIgnoreEntryWire>, IpcError> {
        with_open(&app, super::proofread_ignore_list)
    }

    #[tauri::command]
    pub fn ai_proofread_ignore_remove<R: tauri::Runtime>(
        app: tauri::AppHandle<R>,
        id: i64,
    ) -> Result<bool, IpcError> {
        with_open(&app, |open| super::proofread_ignore_remove(open, id))
    }

    #[tauri::command]
    pub fn ai_proofread_ignore_clear<R: tauri::Runtime>(
        app: tauri::AppHandle<R>,
    ) -> Result<u32, IpcError> {
        with_open(&app, super::proofread_ignore_clear)
    }

    #[tauri::command]
    pub fn ai_proofread_scan_record<R: tauri::Runtime>(
        app: tauri::AppHandle<R>,
        work_id: String,
        scope: String,
        reported: i64,
        filtered: i64,
        blocked: Vec<super::ProofreadBlockedWire>,
    ) -> Result<(), IpcError> {
        with_open(&app, |open| {
            super::proofread_scan_record(open, &work_id, &scope, reported, filtered, &blocked)
        })
    }

    #[tauri::command]
    pub fn ai_proofread_cancel(app: tauri::AppHandle) {
        use tauri::Manager as _;

        let Some(generation_state) = app.try_state::<ProofreadGeneration>() else {
            eprintln!(
                "ai_proofread[cancel] ProofreadGeneration chua duoc quan ly -- loi cau hinh setup()"
            );
            return;
        };
        generation_state.next();
    }
}
