//! Batched proofread scan: prepare layer, run seam with a counting fake provider, and the
//! read-only guarantee over the whole Chapter.

use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex, Once};

use auratranslate_lib::commands::aiconfig::{ai_config_delete_key, ai_config_save_key};
use auratranslate_lib::commands::aitranslate::PreparedTranslateCall;
use auratranslate_lib::commands::project::{OpenWork, create_work_from_text};
use auratranslate_lib::commands::proofread::{
    AiProofreadEventWire, AiProofreadOutcomeWire, PreparedProofreadBatch, PreparedProofreadRun,
    PreparedProofreadSegment, ProofreadPrepareOutcome, ProofreadRunError, prepare_proofread_call,
    run_proofread_call,
};
use auratranslate_lib::commands::segment::{
    ChapterSegment, confirm_segment, promote_ai_translation, read_open_chapter_segments,
    set_segment_omitted,
};
use auratranslate_lib::core::aiconfig::{AiConfigField, AiConfigTier, write_field};
use auratranslate_lib::core::i18n::IpcError;
use auratranslate_lib::core::store::{Store, StoreSpec};
use auratranslate_lib::ports::translation_provider::{
    TranslateOutcome, TranslateRequest, TranslateUsage, TranslationProvider,
};

static NEXT_DIR: AtomicU64 = AtomicU64::new(0);
static KEYCHAIN_TEST_LOCK: Mutex<()> = Mutex::new(());

fn temp_dir(tag: &str) -> PathBuf {
    let n = NEXT_DIR.fetch_add(1, Ordering::Relaxed);
    let dir = std::env::temp_dir().join(format!("auratranslate-ai-proofread-{}-{tag}-{n}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).expect("create temp dir");
    dir
}

fn install_mock_keychain_once() {
    static INIT: Once = Once::new();
    INIT.call_once(|| {
        keyring_core::set_default_store(keyring_core::mock::Store::new().expect("mock store"));
    });
}

fn set_key(value: Option<&str>) {
    install_mock_keychain_once();
    match value {
        Some(v) => ai_config_save_key(AiConfigTier::Global, v).expect("save key"),
        None => ai_config_delete_key(AiConfigTier::Global).expect("delete key"),
    }
}

fn configure(global: &Store) {
    write_field(global, AiConfigField::Endpoint, "https://api.example.invalid/v1/chat/completions")
        .expect("endpoint");
    write_field(global, AiConfigField::Model, "gpt-test").expect("model");
}

fn open_work(root: &std::path::Path, text: &str) -> OpenWork {
    create_work_from_text(root, "Proofread", "en", "", text.to_owned()).expect("create work")
}

fn segments(open: &OpenWork) -> Vec<ChapterSegment> {
    read_open_chapter_segments(Some(open)).expect("load chapter").segments
}

fn expect_err(result: Result<ProofreadPrepareOutcome, IpcError>) -> IpcError {
    match result {
        Ok(_) => panic!("expected Err"),
        Err(e) => e,
    }
}

#[derive(Debug, Clone)]
struct FakeError;
impl std::fmt::Display for FakeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "fake")
    }
}
impl std::error::Error for FakeError {}

enum Step {
    Reply(&'static str, Option<TranslateUsage>),
    Fail,
    CancelInside,
}

struct ScriptedProvider {
    steps: Vec<Step>,
    calls: AtomicUsize,
    cancel_flag: Arc<AtomicBool>,
    raise_flag_after_call: Option<usize>,
}

impl ScriptedProvider {
    fn new(steps: Vec<Step>) -> Self {
        Self { steps, calls: AtomicUsize::new(0), cancel_flag: Arc::new(AtomicBool::new(false)), raise_flag_after_call: None }
    }

    fn raising_the_cancel_flag_after_call(mut self, n: usize) -> Self {
        self.raise_flag_after_call = Some(n);
        self
    }

    fn call_count(&self) -> usize {
        self.calls.load(Ordering::SeqCst)
    }

    fn should_cancel(&self) -> impl Fn() -> bool + use<> {
        let flag = Arc::clone(&self.cancel_flag);
        move || flag.load(Ordering::SeqCst)
    }
}

impl TranslationProvider for ScriptedProvider {
    type Error = FakeError;

    async fn translate(
        &self,
        _request: TranslateRequest<'_>,
        on_token: &mut dyn FnMut(&str),
        _should_cancel: &dyn Fn() -> bool,
    ) -> Result<TranslateOutcome, FakeError> {
        let idx = self.calls.fetch_add(1, Ordering::SeqCst);
        let step = self.steps.get(idx).unwrap_or_else(|| panic!("provider called {} times, scripted {}", idx + 1, self.steps.len()));
        match step {
            Step::Fail => Err(FakeError),
            Step::CancelInside => Ok(TranslateOutcome::Cancelled),
            Step::Reply(text, usage) => {
                on_token(text);
                if self.raise_flag_after_call == Some(idx + 1) {
                    self.cancel_flag.store(true, Ordering::SeqCst);
                }
                Ok(TranslateOutcome::Done(*usage))
            }
        }
    }
}

fn usage(total: u32) -> Option<TranslateUsage> {
    Some(TranslateUsage { prompt_tokens: total, completion_tokens: 0, total_tokens: total, cost_usd: None })
}

const NO_FINDINGS: &str = "[]";
const ONE_SPELLING: &str = r#"[{"segment":1,"kind":"spelling","quote":"nguời","explanation":"e","suggestion":"người"}]"#;

fn run_of(batch_sizes: &[usize]) -> PreparedProofreadRun {
    let mut next_id = 100i64;
    PreparedProofreadRun {
        batches: batch_sizes
            .iter()
            .map(|&n| PreparedProofreadBatch {
                call: PreparedTranslateCall {
                    endpoint: "https://api.example.invalid/v1".to_owned(),
                    model: "gpt-test".to_owned(),
                    temperature: None,
                    max_tokens: None,
                    api_key: "sk-fake".to_owned(),
                    prompt: "p".to_owned(),
                },
                segments: (0..n)
                    .map(|_| {
                        next_id += 1;
                        PreparedProofreadSegment {
                            segment_id: next_id,
                            scanned_text: format!("Mọi nguời đã đi {next_id}"),
                            has_source: true,
                        }
                    })
                    .collect(),
            })
            .collect(),
    }
}

type Events = Arc<Mutex<Vec<serde_json::Value>>>;

fn collecting_channel() -> (tauri::ipc::Channel<AiProofreadEventWire>, Events) {
    let received: Events = Arc::new(Mutex::new(Vec::new()));
    let sink = Arc::clone(&received);
    let channel = tauri::ipc::Channel::new(move |body| {
        if let tauri::ipc::InvokeResponseBody::Json(json) = body {
            let value: serde_json::Value = serde_json::from_str(&json).expect("event json");
            sink.lock().unwrap_or_else(std::sync::PoisonError::into_inner).push(value);
        }
        Ok(())
    });
    (channel, received)
}

type RunResult = Result<AiProofreadOutcomeWire, ProofreadRunError<FakeError>>;

fn run(provider: &ScriptedProvider, prepared: &PreparedProofreadRun) -> (RunResult, Vec<serde_json::Value>) {
    let (channel, received) = collecting_channel();
    let should_cancel = provider.should_cancel();
    let out = tauri::async_runtime::block_on(run_proofread_call(provider, prepared, &channel, &should_cancel));
    let events = received.lock().unwrap_or_else(std::sync::PoisonError::into_inner).clone();
    (out, events)
}

fn batch_events(events: &[serde_json::Value]) -> Vec<&serde_json::Value> {
    events.iter().filter(|e| e["kind"] == "batch").collect()
}

#[test]
fn a_clean_reply_places_findings_on_the_segment_it_names_and_carries_the_batch_usage() {
    let provider = ScriptedProvider::new(vec![Step::Reply(
        r#"[{"segment":2,"kind":"spelling","quote":"nguời","explanation":"sai","suggestion":"người"},
            {"segment":1,"kind":"grammar","quote":"đã đi","explanation":"thi","suggestion":"đi"}]"#,
        usage(7),
    )]);
    let prepared = run_of(&[2]);
    let (out, events) = run(&provider, &prepared);

    assert_eq!(out.ok(), Some(AiProofreadOutcomeWire::Done { not_scanned: 0 }));
    assert_eq!(events[0], serde_json::json!({"kind":"planned","segments":2,"batches":1}));
    let batch = &events[1];
    assert_eq!(batch["kind"], "batch");
    assert_eq!(batch["usage"]["total_tokens"], 7);
    let segs = batch["segments"].as_array().expect("segments");
    assert_eq!(segs[0]["segment_id"], 101);
    assert_eq!(segs[0]["scanned_text"], "Mọi nguời đã đi 101");
    assert_eq!(segs[0]["findings"][0]["kind"], "grammar");
    assert_eq!(segs[1]["segment_id"], 102);
    assert_eq!(segs[1]["findings"][0]["kind"], "spelling");
    assert_eq!((segs[1]["findings"][0]["start"].as_u64(), segs[1]["findings"][0]["end"].as_u64()), (Some(4), Some(9)));
    assert_eq!(segs[0]["unlocated"], 0);
}

#[test]
fn a_segment_with_no_findings_is_still_reported_as_scanned() {
    let provider = ScriptedProvider::new(vec![Step::Reply(NO_FINDINGS, None)]);
    let (_, events) = run(&provider, &run_of(&[3]));
    let batches = batch_events(&events);
    assert_eq!(batches[0]["segments"].as_array().map(Vec::len), Some(3));
    assert!(batches[0]["usage"].is_null());
}

#[test]
fn a_foreign_segment_index_is_counted_unlocated_and_never_lands_on_a_wrong_segment() {
    let provider = ScriptedProvider::new(vec![Step::Reply(
        r#"[{"segment":9,"kind":"spelling","quote":"nguời","explanation":"","suggestion":""}]"#,
        None,
    )]);
    let (_, events) = run(&provider, &run_of(&[2]));
    let segs = batch_events(&events)[0]["segments"].as_array().expect("segments").clone();
    let findings: usize = segs.iter().map(|s| s["findings"].as_array().map_or(0, Vec::len)).sum();
    let unlocated: u64 = segs.iter().filter_map(|s| s["unlocated"].as_u64()).sum();
    assert_eq!((findings, unlocated), (0, 1));
}

#[test]
fn cancel_after_batch_two_of_four_calls_the_provider_exactly_twice() {
    let provider = ScriptedProvider::new(vec![
        Step::Reply(ONE_SPELLING, usage(5)),
        Step::Reply(ONE_SPELLING, usage(6)),
    ])
    .raising_the_cancel_flag_after_call(2);
    let (out, events) = run(&provider, &run_of(&[2, 3, 4, 5]));

    assert_eq!(provider.call_count(), 2);
    assert_eq!(out.ok(), Some(AiProofreadOutcomeWire::Cancelled { not_scanned: 9 }));
    assert_eq!(batch_events(&events).len(), 2, "findings that already arrived are kept");
}

#[test]
fn a_provider_cancelled_inside_batch_three_counts_that_batch_as_not_scanned() {
    let provider = ScriptedProvider::new(vec![
        Step::Reply(NO_FINDINGS, None),
        Step::Reply(NO_FINDINGS, None),
        Step::CancelInside,
    ]);
    let (out, events) = run(&provider, &run_of(&[2, 2, 3, 4]));
    assert_eq!(provider.call_count(), 3);
    assert_eq!(out.ok(), Some(AiProofreadOutcomeWire::Cancelled { not_scanned: 7 }));
    assert_eq!(batch_events(&events).len(), 2);
}

#[test]
fn a_cancel_already_raised_calls_no_provider_at_all() {
    let provider = ScriptedProvider::new(vec![]);
    provider.cancel_flag.store(true, Ordering::SeqCst);
    let (out, events) = run(&provider, &run_of(&[2, 3]));
    assert_eq!(provider.call_count(), 0);
    assert_eq!(out.ok(), Some(AiProofreadOutcomeWire::Cancelled { not_scanned: 5 }));
    assert!(batch_events(&events).is_empty());
}

#[test]
fn a_malformed_reply_loses_only_its_own_batch() {
    let provider = ScriptedProvider::new(vec![
        Step::Reply(ONE_SPELLING, None),
        Step::Reply("not json", usage(9)),
        Step::Reply(ONE_SPELLING, None),
    ]);
    let (out, events) = run(&provider, &run_of(&[2, 3, 4]));
    assert_eq!(provider.call_count(), 3);
    assert_eq!(out.ok(), Some(AiProofreadOutcomeWire::Done { not_scanned: 3 }));
    assert_eq!(batch_events(&events).len(), 3, "the malformed batch still reports its usage");
    assert_eq!(batch_events(&events)[1]["segments"].as_array().map(Vec::len), Some(0));
    assert_eq!(batch_events(&events)[1]["usage"]["total_tokens"], 9);
    let ids: Vec<u64> = batch_events(&events)
        .iter()
        .flat_map(|b| b["segments"].as_array().expect("segments").iter().filter_map(|s| s["segment_id"].as_u64()))
        .collect();
    assert_eq!(ids.len(), 6);
    assert!(!ids.contains(&103), "the segments of the malformed batch draw nothing");
}

#[test]
fn a_single_malformed_reply_is_the_reply_malformed_error_not_a_silent_done() {
    let provider = ScriptedProvider::new(vec![Step::Reply("not json", usage(2))]);
    let (out, events) = run(&provider, &run_of(&[1]));
    let Err(ProofreadRunError::ReplyMalformed { not_scanned }) = out else { panic!("reply malformed") };
    assert_eq!(not_scanned, 1);
    let batches = batch_events(&events);
    assert_eq!((batches.len(), batches[0]["segments"].as_array().map(Vec::len)), (1, Some(0)));
    assert_eq!(batches[0]["usage"]["total_tokens"], 2);
    let err = auratranslate_lib::commands::proofread::reply_malformed_error();
    assert_eq!(err.code(), "ai_proofread.reply_malformed");
    assert!(err.retryable());
}

#[test]
fn a_provider_failure_stops_the_run_keeps_batch_one_and_counts_the_rest_not_scanned() {
    let provider = ScriptedProvider::new(vec![Step::Reply(ONE_SPELLING, None), Step::Fail]);
    let (out, events) = run(&provider, &run_of(&[2, 3, 4]));
    assert_eq!(provider.call_count(), 2);
    let Err(ProofreadRunError::Provider { not_scanned, .. }) = out else { panic!("provider error") };
    assert_eq!(not_scanned, 7);
    assert_eq!(batch_events(&events).len(), 1);
}

#[test]
fn nothing_to_scan_calls_no_provider_and_is_done() {
    let provider = ScriptedProvider::new(vec![]);
    let (out, events) = run(&provider, &run_of(&[]));
    assert_eq!(provider.call_count(), 0);
    assert_eq!(out.ok(), Some(AiProofreadOutcomeWire::Done { not_scanned: 0 }));
    assert_eq!(events, vec![serde_json::json!({"kind":"planned","segments":0,"batches":0})]);
}

#[test]
fn the_outcome_wire_keeps_its_state_tags_and_counts() {
    assert_eq!(serde_json::to_value(AiProofreadOutcomeWire::NotConfigured).unwrap(), serde_json::json!({"state":"not_configured"}));
    assert_eq!(
        serde_json::to_value(AiProofreadOutcomeWire::Done { not_scanned: 2 }).unwrap(),
        serde_json::json!({"state":"done","not_scanned":2})
    );
    assert_eq!(
        serde_json::to_value(AiProofreadOutcomeWire::Cancelled { not_scanned: 4 }).unwrap(),
        serde_json::json!({"state":"cancelled","not_scanned":4})
    );
}

fn chapter_of(count: usize, work: &PathBuf) -> OpenWork {
    let text = (0..count).map(|i| format!("Sentence {i}.")).collect::<Vec<_>>().join("\n\n");
    let open = open_work(work, &text);
    assert_eq!(segments(&open).len(), count, "fixture: one segment per paragraph");
    open
}

fn ready(out: Result<ProofreadPrepareOutcome, IpcError>) -> PreparedProofreadRun {
    match out.expect("prepare") {
        ProofreadPrepareOutcome::Ready(run) => run,
        ProofreadPrepareOutcome::NotConfigured => panic!("expected Ready"),
    }
}

fn scanned_ids(run: &PreparedProofreadRun) -> Vec<i64> {
    run.batches.iter().flat_map(|b| b.segments.iter().map(|s| s.segment_id)).collect()
}

#[test]
fn a_whole_chapter_scans_every_filled_segment_in_document_order_and_skips_empty_and_omitted() {
    let _g = KEYCHAIN_TEST_LOCK.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
    set_key(Some("sk-proofread"));
    let global = Store::open(StoreSpec::global(temp_dir("g5").join("global.db"))).expect("global");
    configure(&global);
    let open = chapter_of(30, &temp_dir("w5"));
    let rows = segments(&open);
    for (i, row) in rows.iter().enumerate().skip(3) {
        promote_ai_translation(Some(&open), row.id, &format!("Mọi nguời đã đi {i}"), false).expect("promote");
    }
    set_segment_omitted(Some(&open), rows[3].id, true).expect("omit");

    let mut ids: Vec<i64> = rows.iter().map(|r| r.id).collect();
    ids.reverse();
    let prepared = ready(prepare_proofread_call(Some(&global), Some(&open), &ids));
    let expected: Vec<i64> = rows.iter().skip(4).map(|r| r.id).collect();
    assert_eq!(scanned_ids(&prepared), expected, "26 segments, document order");
    assert_eq!(prepared.batches.iter().map(|b| b.segments.len()).collect::<Vec<_>>(), vec![20, 6]);
    for batch in &prepared.batches {
        for (n, segment) in batch.segments.iter().enumerate() {
            assert!(batch.call.prompt.contains(&format!("<<<SEGMENT {}\n", n + 1)));
            assert!(batch.call.prompt.contains(&segment.scanned_text));
        }
        assert_eq!(batch.call.api_key, "sk-proofread");
    }

    let provider = ScriptedProvider::new(vec![Step::Reply(NO_FINDINGS, usage(1)), Step::Reply(NO_FINDINGS, usage(1))]);
    let (out, events) = run(&provider, &prepared);
    assert_eq!(out.ok(), Some(AiProofreadOutcomeWire::Done { not_scanned: 0 }));
    assert_eq!(events[0], serde_json::json!({"kind":"planned","segments":26,"batches":2}));
    assert_eq!(provider.call_count(), 2);
}

#[test]
fn a_selection_scans_exactly_its_segments_and_a_foreign_id_is_rejected() {
    let _g = KEYCHAIN_TEST_LOCK.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
    set_key(Some("sk-proofread"));
    let global = Store::open(StoreSpec::global(temp_dir("g6").join("global.db"))).expect("global");
    configure(&global);
    let open = chapter_of(8, &temp_dir("w6"));
    let rows = segments(&open);
    for row in &rows {
        promote_ai_translation(Some(&open), row.id, "Mọi nguời đã đi", false).expect("promote");
    }
    let picked = [rows[6].id, rows[2].id, rows[4].id, rows[3].id, rows[5].id];
    let run = ready(prepare_proofread_call(Some(&global), Some(&open), &picked));
    assert_eq!(scanned_ids(&run), vec![rows[2].id, rows[3].id, rows[4].id, rows[5].id, rows[6].id]);

    let err = expect_err(prepare_proofread_call(Some(&global), Some(&open), &[rows[0].id, i64::MAX]));
    assert_eq!(err.code(), "ai_prompt.segment_not_in_chapter");
}

#[test]
fn only_empty_or_omitted_segments_prepare_an_empty_run_without_reading_the_key() {
    let _g = KEYCHAIN_TEST_LOCK.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
    set_key(None);
    let global = Store::open(StoreSpec::global(temp_dir("g7").join("global.db"))).expect("global");
    configure(&global);
    let open = chapter_of(3, &temp_dir("w7"));
    let ids: Vec<i64> = segments(&open).iter().map(|s| s.id).collect();
    let prepared = ready(prepare_proofread_call(Some(&global), Some(&open), &ids));
    assert!(prepared.batches.is_empty());
}

#[test]
fn no_open_work_is_work_none_open() {
    let global = Store::open(StoreSpec::global(temp_dir("g1").join("global.db"))).expect("global");
    configure(&global);
    let err = expect_err(prepare_proofread_call(Some(&global), None, &[1]));
    assert_eq!(err.code(), "work.none_open");
}

#[test]
fn unconfigured_ai_is_not_configured_before_any_call() {
    let _g = KEYCHAIN_TEST_LOCK.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
    set_key(None);
    let global = Store::open(StoreSpec::global(temp_dir("g2").join("global.db"))).expect("global");
    let open = open_work(&temp_dir("w2"), "One.");
    let id = segments(&open)[0].id;
    let out = prepare_proofread_call(Some(&global), Some(&open), &[id]).expect("not an error");
    assert!(matches!(out, ProofreadPrepareOutcome::NotConfigured));

    configure(&global);
    promote_ai_translation(Some(&open), id, "Một câu.", false).expect("promote");
    let out = prepare_proofread_call(Some(&global), Some(&open), &[id]).expect("not an error");
    assert!(matches!(out, ProofreadPrepareOutcome::NotConfigured), "no key saved");
}

type Snapshot = Vec<(i64, String, String, String, String, String, i64)>;

fn full_snapshot(open: &OpenWork) -> Snapshot {
    open.store
        .read(|conn| {
            let mut stmt = conn.prepare(
                "SELECT s.id, s.target_text, s.translation_origin, s.status, s.baseline_target_text, \
                 s.baseline_translation_origin, (SELECT COUNT(*) FROM segment_version v WHERE v.segment_id = s.id) \
                 FROM segment s ORDER BY s.id",
            )?;
            let rows = stmt.query_map([], |r| {
                Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?, r.get(4)?, r.get(5)?, r.get(6)?))
            })?;
            rows.collect::<Result<Vec<_>, _>>()
        })
        .expect("snapshot")
}

#[test]
fn a_whole_chapter_scan_with_findings_changes_no_text_status_origin_baseline_or_version() {
    let _g = KEYCHAIN_TEST_LOCK.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
    set_key(Some("sk-proofread"));
    let global = Store::open(StoreSpec::global(temp_dir("g4").join("global.db"))).expect("global");
    configure(&global);
    let open = chapter_of(5, &temp_dir("w4"));
    let rows = segments(&open);
    for (i, row) in rows.iter().enumerate() {
        promote_ai_translation(Some(&open), row.id, &format!("Mọi nguời đã đi {i}"), false).expect("promote");
    }
    confirm_segment(Some(&open), rows[0].id).expect("confirm one");

    let before = full_snapshot(&open);
    assert!(before.iter().any(|r| r.3 != before[4].3), "mixed confirmation states");
    assert!(before.iter().all(|r| !r.4.is_empty() || !r.5.is_empty()), "baselines are populated");

    let ids: Vec<i64> = rows.iter().map(|r| r.id).collect();
    let prepared = ready(prepare_proofread_call(Some(&global), Some(&open), &ids));
    let provider = ScriptedProvider::new(vec![Step::Reply(
        r#"[{"segment":1,"kind":"spelling","quote":"nguời","explanation":"","suggestion":"người"},
            {"segment":3,"kind":"grammar","quote":"đã đi","explanation":"","suggestion":"đi"}]"#,
        usage(3),
    )]);
    let (out, events) = run(&provider, &prepared);
    assert_eq!(out.ok(), Some(AiProofreadOutcomeWire::Done { not_scanned: 0 }));
    let findings: usize = batch_events(&events)
        .iter()
        .flat_map(|b| b["segments"].as_array().expect("segments").iter())
        .map(|s| s["findings"].as_array().map_or(0, Vec::len))
        .sum();
    assert_eq!(findings, 2, "the scan must have findings for this guard to mean anything");

    assert_eq!(full_snapshot(&open), before);
}

#[test]
fn a_meaning_finding_reaches_the_wire_with_its_exact_kind_string() {
    let provider = ScriptedProvider::new(vec![Step::Reply(
        r#"[{"segment":1,"kind":"mistranslation","quote":"đã đi","explanation":"lech nghia","suggestion":"sẽ đi"},
            {"segment":1,"kind":"too_loose","quote":"Mọi","explanation":"","suggestion":""},
            {"segment":1,"kind":"obscure","quote":"nguời","explanation":"","suggestion":""}]"#,
        None,
    )]);
    let (_, events) = run(&provider, &run_of(&[1]));
    let kinds: Vec<String> = batch_events(&events)[0]["segments"][0]["findings"]
        .as_array()
        .expect("findings")
        .iter()
        .map(|f| f["kind"].as_str().expect("kind").to_owned())
        .collect();
    assert_eq!(kinds, ["too_loose", "obscure", "mistranslation"]);
}

#[test]
fn a_blank_source_counts_a_meaning_finding_unlocated_and_keeps_the_spelling_one() {
    let mut prepared = run_of(&[1]);
    prepared.batches[0].segments[0].has_source = false;
    let provider = ScriptedProvider::new(vec![Step::Reply(
        r#"[{"segment":1,"kind":"mistranslation","quote":"đã đi","explanation":"","suggestion":""},
            {"segment":1,"kind":"spelling","quote":"nguời","explanation":"","suggestion":"người"}]"#,
        None,
    )]);
    let (_, events) = run(&provider, &prepared);
    let seg = &batch_events(&events)[0]["segments"][0];
    assert_eq!((seg["findings"].as_array().map(Vec::len), seg["unlocated"].as_u64()), (Some(1), Some(1)));
    assert_eq!(seg["findings"][0]["kind"], "spelling");
}

#[test]
fn a_repeated_context_phrase_reaches_the_wire_as_ambiguous_and_never_as_a_finding() {
    let mut prepared = run_of(&[1]);
    prepared.batches[0].segments[0].scanned_text = "đi và đi nữa nguời nguời".to_owned();
    let provider = ScriptedProvider::new(vec![Step::Reply(
        r#"[{"segment":1,"kind":"grammar","quote":"đi","explanation":"","suggestion":"đến"},
            {"segment":1,"kind":"spelling","quote":"nguời","explanation":"","suggestion":"người"}]"#,
        None,
    )]);
    let (out, events) = run(&provider, &prepared);
    let seg = &batch_events(&events)[0]["segments"][0];
    assert_eq!(seg["ambiguous"], 1);
    assert_eq!(seg["unlocated"], 0);
    let kinds: Vec<&str> = seg["findings"].as_array().expect("findings").iter().filter_map(|f| f["kind"].as_str()).collect();
    assert_eq!(kinds, ["spelling"]);
    assert_eq!(out.ok(), Some(AiProofreadOutcomeWire::Done { not_scanned: 0 }));
}
