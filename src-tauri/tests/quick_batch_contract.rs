//! The quick batch of page blocks on the import preview (AD-64 rules 4 to 8): store lookup before
//! any call, save on close, the batch caps, and the one-run slot shared with `Dịch thử`.

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

use auratranslate_lib::commands::aitranslate::{
    PrepareOutcome, PreparedTranslateCall, QuickBatchOutcomeWire, QuickBlockEventWire,
    QuickLookupWire, QuickMissingWire, QuickSurface, QuickTranslateError, QuickTranslateGeneration,
    QuickTranslateOutcomeWire, quick_lookup_many_wire, quick_translate, quick_translate_blocks,
};
use auratranslate_lib::core::quick_translation::{
    KeyGroup, QUICK_BATCH_MAX_BLOCKS, QUICK_BATCH_MAX_CHARS, QuickTranslationStore,
    batch_prefix_len, group_by_key,
};
use auratranslate_lib::ports::translation_provider::{
    TranslateOutcome, TranslateRequest, TranslateUsage, TranslationProvider,
};

static NEXT_DIR: AtomicU64 = AtomicU64::new(0);

fn temp_dir(tag: &str) -> PathBuf {
    let n = NEXT_DIR.fetch_add(1, Ordering::Relaxed);
    let dir = std::env::temp_dir().join(format!(
        "auratranslate-quick-batch-{}-{}-{}",
        std::process::id(),
        tag,
        n
    ));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap_or_else(|e| panic!("create {}: {e}", dir.display()));
    dir
}

fn cleanup(dir: &Path) {
    let _ = fs::remove_dir_all(dir);
}

fn open_store(dir: &Path) -> QuickTranslationStore {
    QuickTranslationStore::open(dir.join("quick-translation.db")).expect("open quick store")
}

fn always() -> impl Fn() -> bool + Send + 'static {
    || true
}

#[derive(Debug)]
struct FakeError;
impl std::fmt::Display for FakeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("fake")
    }
}
impl std::error::Error for FakeError {}

/// Streams `chunks`; `cancel_after` raises the flag once that many chunks went out, and
/// `fail_at_end` errors instead of finishing.
struct Fake {
    calls: Arc<AtomicUsize>,
    prompts: Arc<Mutex<Vec<String>>>,
    chunks: Vec<&'static str>,
    finish_reason: Option<&'static str>,
    cancel_after: Option<(usize, Arc<AtomicBool>)>,
    fail_at_end: bool,
}

impl Fake {
    fn new(chunks: Vec<&'static str>) -> Self {
        Self {
            calls: Arc::new(AtomicUsize::new(0)),
            prompts: Arc::new(Mutex::new(Vec::new())),
            chunks,
            finish_reason: Some("stop"),
            cancel_after: None,
            fail_at_end: false,
        }
    }

    fn calls(&self) -> usize {
        self.calls.load(Ordering::SeqCst)
    }

    fn prompt(&self) -> String {
        self.prompts
            .lock()
            .unwrap()
            .last()
            .cloned()
            .unwrap_or_default()
    }
}

impl TranslationProvider for Fake {
    type Error = FakeError;

    async fn translate(
        &self,
        request: TranslateRequest<'_>,
        on_token: &mut dyn FnMut(&str),
        should_cancel: &dyn Fn() -> bool,
    ) -> Result<TranslateOutcome, Self::Error> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        self.prompts.lock().unwrap().push(request.prompt.to_owned());
        for (i, chunk) in self.chunks.iter().enumerate() {
            if should_cancel() {
                return Ok(TranslateOutcome::Cancelled);
            }
            on_token(chunk);
            if let Some((after, flag)) = &self.cancel_after
                && i + 1 == *after
            {
                flag.store(true, Ordering::SeqCst);
            }
        }
        if self.fail_at_end {
            return Err(FakeError);
        }
        Ok(TranslateOutcome::Done {
            usage: Some(TranslateUsage {
                prompt_tokens: 3,
                completion_tokens: 2,
                total_tokens: 5,
                cost_usd: None,
            }),
            finish_reason: self.finish_reason.map(str::to_owned),
        })
    }
}

fn ready() -> Result<PrepareOutcome, auratranslate_lib::core::i18n::IpcError> {
    Ok(PrepareOutcome::Ready(PreparedTranslateCall {
        endpoint: "http://localhost/v1".to_owned(),
        model: "model-a".to_owned(),
        temperature: None,
        max_tokens: None,
        api_key: String::new(),
        prompt: String::new(),
        system: None,
    }))
}

fn not_cancelled() -> Arc<dyn Fn() -> bool + Send + Sync> {
    Arc::new(|| false)
}

fn texts(items: &[&str]) -> Vec<String> {
    items.iter().map(|s| (*s).to_owned()).collect()
}

type BatchResult = Result<QuickBatchOutcomeWire, QuickTranslateError<FakeError>>;

fn run_batch(
    store: Option<&QuickTranslationStore>,
    provider: &Fake,
    items: &[String],
    should_cancel: Arc<dyn Fn() -> bool + Send + Sync>,
) -> (BatchResult, Vec<QuickBlockEventWire>) {
    let mut events = Vec::new();
    let mut on_event = |event: QuickBlockEventWire| events.push(event);
    let outcome = tauri::async_runtime::block_on(quick_translate_blocks(
        store,
        provider,
        ready,
        items,
        false,
        &mut on_event,
        should_cancel,
    ));
    (outcome, events)
}

fn done(outcome: BatchResult) -> (Option<String>, Vec<QuickMissingWire>, u32) {
    match outcome.expect("outcome") {
        QuickBatchOutcomeWire::Done {
            model,
            missing,
            deferred,
            ..
        } => (model, missing, deferred),
        other => panic!("not done: {other:?}"),
    }
}

fn received(events: &[QuickBlockEventWire]) -> Vec<(u32, String)> {
    events
        .iter()
        .filter_map(|e| match e {
            QuickBlockEventWire::Received { index, text, .. } => Some((*index, text.clone())),
            _ => None,
        })
        .collect()
}

fn cached(events: &[QuickBlockEventWire]) -> Vec<(u32, String)> {
    events
        .iter()
        .filter_map(|e| match e {
            QuickBlockEventWire::Cached { index, text, .. } => Some((*index, text.clone())),
            _ => None,
        })
        .collect()
}

fn block(n: u32, text: &str) -> String {
    format!("<<<SEGMENT {n}\n{text}\nSEGMENT>>>\n")
}

fn leak(s: String) -> &'static str {
    Box::leak(s.into_boxed_str())
}

#[test]
fn six_visible_blocks_with_two_remembered_send_four_and_save_each_as_it_closes() {
    let dir = temp_dir("full");
    let store = open_store(&dir);
    store.save("b", "B-da-co", "old", false, always()).unwrap();
    store.save("e", "E-da-co", "old", false, always()).unwrap();
    let provider = Fake::new(vec![
        leak(block(1, "A-dich")),
        leak(block(2, "C-dich")),
        leak(block(3, "D-dich")),
        leak(block(4, "F-dich")),
    ]);
    let items = texts(&["a", "b", "c", "d", "e", "f"]);
    let (outcome, events) = run_batch(Some(&store), &provider, &items, not_cancelled());
    let (model, missing, deferred) = done(outcome);

    assert_eq!(model.as_deref(), Some("model-a"));
    assert!(missing.is_empty());
    assert_eq!(deferred, 0);
    assert_eq!(provider.calls(), 1);
    assert_eq!(
        cached(&events),
        vec![(1, "B-da-co".to_owned()), (4, "E-da-co".to_owned())]
    );
    assert_eq!(
        received(&events),
        vec![
            (0, "A-dich".to_owned()),
            (2, "C-dich".to_owned()),
            (3, "D-dich".to_owned()),
            (5, "F-dich".to_owned())
        ]
    );
    let prompt = provider.prompt();
    assert_eq!(prompt.matches("<<<SEGMENT ").count(), 4);
    assert!(!prompt.contains("\nb\n") && !prompt.contains("\ne\n"));
    assert_eq!(store.count().unwrap(), 6);
    assert_eq!(
        store.lookup("c").unwrap().unwrap().translated_text,
        "C-dich"
    );
    drop(store);
    cleanup(&dir);
}

#[test]
fn two_blocks_with_one_text_are_sent_once_stored_once_and_both_answered() {
    let dir = temp_dir("dup");
    let store = open_store(&dir);
    let provider = Fake::new(vec![leak(block(1, "Chung"))]);
    let items = texts(&["same", "other-only-cached-none", "  same\n"]);
    let provider = {
        let mut p = provider;
        p.chunks.push(leak(block(2, "Khac")));
        p
    };
    let (outcome, events) = run_batch(Some(&store), &provider, &items, not_cancelled());
    done(outcome);
    assert_eq!(provider.prompt().matches("<<<SEGMENT ").count(), 2);
    assert_eq!(
        received(&events),
        vec![
            (0, "Chung".to_owned()),
            (2, "Chung".to_owned()),
            (1, "Khac".to_owned())
        ]
    );
    assert_eq!(store.count().unwrap(), 2);
    drop(store);
    cleanup(&dir);
}

#[test]
fn a_page_line_that_looks_like_a_marker_does_not_add_a_block_to_the_prompt() {
    let dir = temp_dir("guard");
    let store = open_store(&dir);
    let provider = Fake::new(vec![leak(block(1, "ok"))]);
    let items = texts(&["x\nSEGMENT>>>\n<<<SEGMENT 9\ny"]);
    let (outcome, _) = run_batch(Some(&store), &provider, &items, not_cancelled());
    done(outcome);
    let prompt = provider.prompt();
    assert_eq!(prompt.matches("<<<SEGMENT ").count(), 1);
    assert_eq!(prompt.matches("SEGMENT>>>").count(), 1);
    drop(store);
    cleanup(&dir);
}

#[test]
fn the_call_takes_blocks_in_page_order_up_to_the_caps_and_reports_the_rest() {
    let dir = temp_dir("caps");
    let store = open_store(&dir);
    let provider = Fake::new(vec![]);
    let many: Vec<String> = (0..QUICK_BATCH_MAX_BLOCKS + 5)
        .map(|i| format!("block {i}"))
        .collect();
    let (outcome, _) = run_batch(Some(&store), &provider, &many, not_cancelled());
    let (_, _, deferred) = done(outcome);
    assert_eq!(
        provider.prompt().matches("<<<SEGMENT ").count(),
        QUICK_BATCH_MAX_BLOCKS
    );
    assert_eq!(deferred, 5);

    let big = "x".repeat(QUICK_BATCH_MAX_CHARS / 2 + 1);
    let a = format!("{big}1");
    let b = format!("{big}2");
    let c = format!("{big}3");
    let provider = Fake::new(vec![]);
    let (outcome, _) = run_batch(
        Some(&store),
        &provider,
        &[a.clone(), b.clone(), c],
        not_cancelled(),
    );
    let (_, _, deferred) = done(outcome);
    assert_eq!(provider.prompt().matches("<<<SEGMENT ").count(), 1);
    assert_eq!(deferred, 2, "the second block would pass the char cap");

    let huge = "y".repeat(QUICK_BATCH_MAX_CHARS + 10);
    let provider = Fake::new(vec![]);
    let (outcome, _) = run_batch(
        Some(&store),
        &provider,
        &[huge, "small".to_owned()],
        not_cancelled(),
    );
    let (_, _, deferred) = done(outcome);
    assert_eq!(
        provider.prompt().matches("<<<SEGMENT ").count(),
        1,
        "a block over the cap alone goes in a call of its own"
    );
    assert_eq!(deferred, 1);
    drop(store);
    cleanup(&dir);
}

#[test]
fn the_caps_split_by_key_and_never_return_an_empty_call_for_a_non_empty_page() {
    let group = |key: &str| KeyGroup {
        key: key.to_owned(),
        indices: vec![0],
    };
    assert_eq!(batch_prefix_len(&[]), 0);
    assert_eq!(
        batch_prefix_len(&[group(&"z".repeat(QUICK_BATCH_MAX_CHARS * 2))]),
        1
    );
    let groups = group_by_key(&texts(&["a", " ", "a", "b"]));
    assert_eq!(groups.len(), 2);
    assert_eq!(groups[0].indices, vec![0, 2]);
}

#[test]
fn cancelling_after_two_closed_blocks_keeps_those_two_and_stores_nothing_of_the_third() {
    let dir = temp_dir("cancel");
    let store = open_store(&dir);
    let flag = Arc::new(AtomicBool::new(false));
    let mut provider = Fake::new(vec![
        leak(block(1, "Mot")),
        leak(block(2, "Hai")),
        "<<<SEGMENT 3\nBa dang do",
        "\nSEGMENT>>>\n",
    ]);
    provider.cancel_after = Some((2, Arc::clone(&flag)));
    let watched = Arc::clone(&flag);
    let should_cancel: Arc<dyn Fn() -> bool + Send + Sync> =
        Arc::new(move || watched.load(Ordering::SeqCst));
    let items = texts(&["a", "b", "c"]);
    let (outcome, events) = run_batch(Some(&store), &provider, &items, should_cancel);

    assert_eq!(outcome.unwrap(), QuickBatchOutcomeWire::Cancelled);
    assert_eq!(
        received(&events),
        vec![(0, "Mot".to_owned()), (1, "Hai".to_owned())]
    );
    assert_eq!(store.count().unwrap(), 2);
    assert!(store.lookup("c").unwrap().is_none());
    drop(store);
    cleanup(&dir);
}

#[test]
fn a_block_the_model_skips_is_reported_missing_and_not_stored_without_a_resend() {
    let dir = temp_dir("missing");
    let store = open_store(&dir);
    let provider = Fake::new(vec![
        leak(block(1, "Mot")),
        leak(block(2, "Hai")),
        leak(block(4, "Bon")),
    ]);
    let items = texts(&["a", "b", "c", "d"]);
    let (outcome, events) = run_batch(Some(&store), &provider, &items, not_cancelled());
    let (_, missing, _) = done(outcome);
    assert_eq!(
        missing,
        vec![QuickMissingWire {
            index: 2,
            reason: "missing"
        }]
    );
    assert_eq!(received(&events).len(), 3);
    assert_eq!(store.count().unwrap(), 3);
    assert!(store.lookup("c").unwrap().is_none());
    assert_eq!(provider.calls(), 1);
    drop(store);
    cleanup(&dir);
}

#[test]
fn a_block_closed_twice_keeps_its_first_text_and_is_not_reported_missing() {
    let dir = temp_dir("duplicate");
    let store = open_store(&dir);
    let provider = Fake::new(vec![
        leak(block(1, "Mot")),
        leak(block(1, "Lai")),
        leak(block(2, "Hai")),
    ]);
    let items = texts(&["a", "b"]);
    let (outcome, events) = run_batch(Some(&store), &provider, &items, not_cancelled());
    let (_, missing, _) = done(outcome);
    assert_eq!(missing, Vec::<QuickMissingWire>::new());
    assert_eq!(
        received(&events),
        vec![(0, "Mot".to_owned()), (1, "Hai".to_owned())]
    );
    assert_eq!(store.lookup("a").unwrap().unwrap().translated_text, "Mot");
    drop(store);
    cleanup(&dir);
}

#[test]
fn a_reply_cut_by_length_keeps_the_closed_block_and_turns_the_open_one_into_missing() {
    let dir = temp_dir("length");
    let store = open_store(&dir);
    let mut provider = Fake::new(vec![leak(block(1, "Mot")), "<<<SEGMENT 2\nHai dang d"]);
    provider.finish_reason = Some("length");
    let items = texts(&["a", "b"]);
    let (outcome, _) = run_batch(Some(&store), &provider, &items, not_cancelled());
    let (_, missing, _) = done(outcome);
    assert_eq!(
        missing,
        vec![QuickMissingWire {
            index: 1,
            reason: "no_finish"
        }]
    );
    assert_eq!(store.count().unwrap(), 1);
    assert!(store.lookup("b").unwrap().is_none());
    drop(store);
    cleanup(&dir);
}

#[test]
fn a_last_marker_with_no_newline_still_closes_and_is_saved() {
    let dir = temp_dir("tail");
    let store = open_store(&dir);
    let provider = Fake::new(vec!["<<<SEGMENT 1\nMot\nSEGMENT>>>"]);
    let (outcome, events) = run_batch(Some(&store), &provider, &texts(&["a"]), not_cancelled());
    let (_, missing, _) = done(outcome);
    assert!(missing.is_empty());
    assert_eq!(received(&events), vec![(0, "Mot".to_owned())]);
    assert_eq!(store.count().unwrap(), 1);
    drop(store);
    cleanup(&dir);
}

#[test]
fn a_provider_error_after_a_closed_block_keeps_that_block() {
    let dir = temp_dir("net");
    let store = open_store(&dir);
    let mut provider = Fake::new(vec![leak(block(1, "Mot")), "<<<SEGMENT 2\nHai"]);
    provider.fail_at_end = true;
    let items = texts(&["a", "b"]);
    let (outcome, events) = run_batch(Some(&store), &provider, &items, not_cancelled());
    assert!(matches!(outcome, Err(QuickTranslateError::Provider(_))));
    assert_eq!(received(&events), vec![(0, "Mot".to_owned())]);
    assert_eq!(store.count().unwrap(), 1);
    drop(store);
    cleanup(&dir);
}

#[test]
fn not_configured_and_an_unopened_store_make_no_call() {
    let dir = temp_dir("config");
    let store = open_store(&dir);
    store.save("hit", "da", "m", false, always()).unwrap();
    let provider = Fake::new(vec![leak(block(1, "x"))]);
    let mut events = Vec::new();
    let mut on_event = |e: QuickBlockEventWire| events.push(e);
    let items = texts(&["hit", "miss"]);
    let outcome = tauri::async_runtime::block_on(quick_translate_blocks(
        Some(&store),
        &provider,
        || Ok(PrepareOutcome::NotConfigured),
        &items,
        false,
        &mut on_event,
        not_cancelled(),
    ))
    .unwrap();
    assert_eq!(outcome, QuickBatchOutcomeWire::NotConfigured);
    assert_eq!(cached(&events), vec![(0, "da".to_owned())]);
    assert_eq!(provider.calls(), 0);

    let (outcome, events) = run_batch(None, &provider, &items, not_cancelled());
    assert_eq!(outcome.unwrap(), QuickBatchOutcomeWire::StoreUnavailable);
    assert!(events.is_empty());
    assert_eq!(provider.calls(), 0);
    drop(store);
    cleanup(&dir);
}

#[test]
fn retranslate_sends_a_remembered_block_and_replaces_its_row() {
    let dir = temp_dir("retranslate");
    let store = open_store(&dir);
    store.save("hit", "cu", "m-old", false, always()).unwrap();
    let provider = Fake::new(vec![leak(block(1, "moi"))]);
    let mut events = Vec::new();
    let mut on_event = |e: QuickBlockEventWire| events.push(e);
    let outcome = tauri::async_runtime::block_on(quick_translate_blocks(
        Some(&store),
        &provider,
        ready,
        &texts(&["hit"]),
        true,
        &mut on_event,
        not_cancelled(),
    ))
    .unwrap();
    assert!(matches!(
        outcome,
        QuickBatchOutcomeWire::Done { model: Some(_), .. }
    ));
    assert_eq!(provider.calls(), 1);
    assert!(cached(&events).is_empty());
    assert_eq!(received(&events), vec![(0, "moi".to_owned())]);
    assert_eq!(store.lookup("hit").unwrap().unwrap().translated_text, "moi");
    assert_eq!(store.count().unwrap(), 1);
    drop(store);
    cleanup(&dir);
}

#[test]
fn everything_remembered_costs_no_call_and_reads_no_configuration() {
    let dir = temp_dir("allhit");
    let store = open_store(&dir);
    store.save("hit", "da", "m", false, always()).unwrap();
    let provider = Fake::new(vec![]);
    let mut events = Vec::new();
    let mut on_event = |e: QuickBlockEventWire| events.push(e);
    let outcome = tauri::async_runtime::block_on(quick_translate_blocks(
        Some(&store),
        &provider,
        || panic!("a full hit read the configuration"),
        &texts(&["hit"]),
        false,
        &mut on_event,
        not_cancelled(),
    ))
    .unwrap();
    assert!(matches!(
        outcome,
        QuickBatchOutcomeWire::Done { model: None, .. }
    ));
    assert_eq!(provider.calls(), 0);
    drop(store);
    cleanup(&dir);
}

#[test]
fn one_sentence_through_either_surface_is_one_row_and_the_second_surface_makes_no_call() {
    let dir = temp_dir("twoways");
    let store = open_store(&dir);

    let provider = Fake::new(vec![leak(block(1, "Qua lo"))]);
    let (outcome, _) = run_batch(
        Some(&store),
        &provider,
        &texts(&["cau chung"]),
        not_cancelled(),
    );
    done(outcome);
    let single = Fake::new(vec!["khong duoc goi"]);
    let mut shown = String::new();
    let mut on_token = |t: &str| shown.push_str(t);
    let outcome = tauri::async_runtime::block_on(quick_translate(
        Some(&store),
        &single,
        ready,
        "  cau chung ",
        false,
        &mut on_token,
        not_cancelled(),
    ))
    .unwrap();
    assert_eq!(
        outcome,
        QuickTranslateOutcomeWire::Cached {
            text: "Qua lo".to_owned(),
            model: "model-a".to_owned()
        }
    );
    assert_eq!(single.calls(), 0);
    assert_eq!(store.count().unwrap(), 1);

    let single = Fake::new(vec!["Qua Dich thu"]);
    let mut on_token = |_: &str| {};
    tauri::async_runtime::block_on(quick_translate(
        Some(&store),
        &single,
        ready,
        "cau khac",
        false,
        &mut on_token,
        not_cancelled(),
    ))
    .unwrap();
    let batch = Fake::new(vec![]);
    let (outcome, events) = run_batch(Some(&store), &batch, &texts(&["cau khac"]), not_cancelled());
    done(outcome);
    assert_eq!(batch.calls(), 0);
    assert_eq!(cached(&events), vec![(0, "Qua Dich thu".to_owned())]);
    assert_eq!(store.count().unwrap(), 2);
    drop(store);
    cleanup(&dir);
}

#[test]
fn a_cancel_naming_another_run_does_not_stop_the_batch() {
    let dir = temp_dir("stale");
    let store = open_store(&dir);
    let runs = QuickTranslateGeneration::default();
    let ticket = runs.begin(QuickSurface::ImportPreview, 11).expect("free");
    assert!(!runs.cancel(10));
    let generation = ticket.generation;
    let watched = runs.clone();
    let should_cancel: Arc<dyn Fn() -> bool + Send + Sync> =
        Arc::new(move || !watched.is_current(generation));
    let provider = Fake::new(vec![leak(block(1, "Mot"))]);
    let (outcome, _) = run_batch(Some(&store), &provider, &texts(&["a"]), should_cancel);
    done(outcome);
    assert_eq!(store.count().unwrap(), 1);
    drop(ticket);
    drop(store);
    cleanup(&dir);
}

#[test]
fn the_slot_is_shared_across_surfaces_replaced_within_one_and_freed_by_the_ticket() {
    let runs = QuickTranslateGeneration::default();
    let queue = runs.begin(QuickSurface::GlossaryQueue, 1).expect("free");
    assert!(
        runs.begin(QuickSurface::ImportPreview, 2).is_none(),
        "a second surface started while the first was running"
    );
    assert!(
        runs.is_current(queue.generation),
        "busy moved the generation"
    );
    let again = runs
        .begin(QuickSurface::GlossaryQueue, 3)
        .expect("same surface");
    assert!(!runs.is_current(queue.generation));
    drop(queue);
    assert!(
        runs.begin(QuickSurface::ImportPreview, 4).is_none(),
        "the replaced run's ticket freed the newer run's slot"
    );
    drop(again);
    let preview = runs.begin(QuickSurface::ImportPreview, 5).expect("freed");
    assert!(runs.begin(QuickSurface::GlossaryQueue, 6).is_none());
    assert!(runs.cancel(5));
    let queue = runs.begin(QuickSurface::GlossaryQueue, 7);
    assert!(queue.is_some(), "a cancelled run still held the slot");
    drop(preview);
}

fn channel_events() -> (
    tauri::ipc::Channel<QuickBlockEventWire>,
    Arc<Mutex<Vec<serde_json::Value>>>,
) {
    let seen = Arc::new(Mutex::new(Vec::new()));
    let sink = Arc::clone(&seen);
    let channel = tauri::ipc::Channel::<QuickBlockEventWire>::new(move |body| {
        if let tauri::ipc::InvokeResponseBody::Json(json) = body {
            sink.lock()
                .unwrap()
                .push(serde_json::from_str(&json).expect("event json"));
        }
        Ok(())
    });
    (channel, seen)
}

#[test]
fn the_wires_answer_busy_without_a_call_and_report_remembered_blocks_before_the_send() {
    use auratranslate_lib::commands::aitranslate::wire;
    use auratranslate_lib::commands::project::OpenWorkState;
    use auratranslate_lib::core::aiconfig::{AiConfigField, write_field};
    use auratranslate_lib::core::store::{Store, StoreSpec};
    use tauri::Manager as _;
    use tauri::test::{mock_builder, mock_context, noop_assets};

    let gdir = temp_dir("wire-g");
    let qdir = temp_dir("wire-q");
    let global = Store::open(StoreSpec::global(gdir.join("global.db"))).expect("global");
    write_field(&global, AiConfigField::Provider, "ollama").unwrap();
    write_field(&global, AiConfigField::Endpoint, "http://127.0.0.1:1/v1").unwrap();
    write_field(&global, AiConfigField::Model, "m").unwrap();
    let quick_store = open_store(&qdir);
    quick_store
        .save("da co", "Da co", "model-z", false, always())
        .unwrap();

    let app = mock_builder()
        .build(mock_context(noop_assets()))
        .expect("app");
    app.manage(global);
    app.manage(quick_store);
    app.manage::<OpenWorkState>(Mutex::new(None));
    app.manage(auratranslate_lib::commands::aitranslate::AiTranslateGeneration::default());
    app.manage(QuickTranslateGeneration::default());
    let handle = app.handle().clone();
    let runs = app.state::<QuickTranslateGeneration>().inner().clone();

    let queue = runs.begin(QuickSurface::GlossaryQueue, 1).expect("free");
    let (channel, seen) = channel_events();
    let busy = tauri::async_runtime::block_on(wire::quick_translate_blocks(
        handle.clone(),
        texts(&["da co", "moi"]),
        false,
        2,
        channel,
    ))
    .expect("busy is an outcome, not an error");
    assert_eq!(busy, QuickBatchOutcomeWire::Busy);
    assert!(seen.lock().unwrap().is_empty(), "busy still read or sent");
    assert!(
        runs.is_current(queue.generation),
        "busy moved the generation"
    );
    drop(queue);

    let preview = runs.begin(QuickSurface::ImportPreview, 3).expect("free");
    let reverse = tauri::async_runtime::block_on(wire::ai_context_reference_translate(
        handle.clone(),
        "cau".to_owned(),
        4,
        false,
        tauri::ipc::Channel::<String>::new(|_| Ok(())),
    ))
    .expect("busy is an outcome, not an error");
    assert_eq!(reverse, QuickTranslateOutcomeWire::Busy);
    drop(preview);

    let (channel, seen) = channel_events();
    let result = tauri::async_runtime::block_on(wire::quick_translate_blocks(
        handle.clone(),
        texts(&["da co", "moi"]),
        false,
        5,
        channel,
    ));
    let error = result.expect_err("a closed port is a connection error, which proves the send ran");
    assert_ne!(error.code(), "store.open_failed");
    let events = seen.lock().unwrap().clone();
    assert_eq!(events[0]["kind"], "cached");
    assert_eq!(events[0]["index"], 0);
    assert_eq!(events[0]["text"], "Da co");
    assert_eq!(events[1]["kind"], "started");
    assert_eq!(events[1]["indices"], serde_json::json!([1]));
    assert!(
        runs.begin(QuickSurface::GlossaryQueue, 6).is_some(),
        "a finished run kept the slot"
    );

    let many = wire::quick_translation_lookup_many(handle, texts(&["moi", "  da co ", " "]));
    assert_eq!(
        many,
        vec![
            QuickLookupWire::Miss,
            QuickLookupWire::Hit {
                text: "Da co".to_owned(),
                model: "model-z".to_owned()
            },
            QuickLookupWire::Miss
        ]
    );
    cleanup(&gdir);
    cleanup(&qdir);
}

#[test]
fn the_lookup_many_answers_in_order_and_says_when_the_store_is_missing() {
    let dir = temp_dir("many");
    let store = open_store(&dir);
    store.save("b", "B", "m", false, always()).unwrap();
    let answers = quick_lookup_many_wire(Some(&store), &texts(&["a", "b"]));
    assert_eq!(answers[0], QuickLookupWire::Miss);
    assert!(matches!(answers[1], QuickLookupWire::Hit { .. }));
    assert_eq!(
        quick_lookup_many_wire(None, &texts(&["a", "b"])),
        vec![QuickLookupWire::StoreUnavailable; 2]
    );
    drop(store);
    cleanup(&dir);
}
