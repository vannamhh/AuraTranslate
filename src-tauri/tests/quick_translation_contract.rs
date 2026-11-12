//! `quick-translation.db` and the `Dịch thử` command around it (AD-64 rules 3 to 6 and 8).

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

use auratranslate_lib::commands::aitranslate::{
    AiTranslateGeneration, PreparedTranslateCall, QuickLookupWire, QuickPrepareOutcome,
    QuickPurgeScopeWire, QuickSurface, QuickTranslateError, QuickTranslateGeneration,
    QuickTranslateOutcomeWire, quick_count_older_wire, quick_lookup_wire, quick_purge_wire,
    quick_stats_wire, quick_translate,
};
use auratranslate_lib::commands::project::{OpenWork, create_work_from_text};
use auratranslate_lib::core::quick_translation::{QuickTranslationStore, quick_key};
use auratranslate_lib::ports::translation_provider::{
    TranslateOutcome, TranslateRequest, TranslateUsage, TranslationProvider,
};

static NEXT_DIR: AtomicU64 = AtomicU64::new(0);

fn temp_dir(tag: &str) -> PathBuf {
    let n = NEXT_DIR.fetch_add(1, Ordering::Relaxed);
    let dir = std::env::temp_dir().join(format!(
        "auratranslate-quick-{}-{}-{}",
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

struct Fake {
    calls: Arc<AtomicUsize>,
    tokens: Vec<&'static str>,
    finish_reason: Option<&'static str>,
    fail: bool,
    cancel_flag_after_stream: Option<Arc<AtomicBool>>,
    gate: Option<Gate>,
}

struct Gate {
    entered: std::sync::mpsc::Sender<()>,
    release: Mutex<std::sync::mpsc::Receiver<()>>,
}

impl Fake {
    fn ok(tokens: Vec<&'static str>) -> (Self, Arc<AtomicUsize>) {
        let calls = Arc::new(AtomicUsize::new(0));
        (
            Self {
                calls: Arc::clone(&calls),
                tokens,
                finish_reason: Some("stop"),
                fail: false,
                cancel_flag_after_stream: None,
                gate: None,
            },
            calls,
        )
    }
}

impl TranslationProvider for Fake {
    type Error = FakeError;

    async fn translate(
        &self,
        _request: TranslateRequest<'_>,
        on_token: &mut dyn FnMut(&str),
        should_cancel: &dyn Fn() -> bool,
    ) -> Result<TranslateOutcome, Self::Error> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        if self.fail {
            return Err(FakeError);
        }
        if let Some(gate) = &self.gate {
            let _ = gate.entered.send(());
            let _ = gate
                .release
                .lock()
                .unwrap()
                .recv_timeout(std::time::Duration::from_secs(20));
        }
        for token in &self.tokens {
            if should_cancel() {
                return Ok(TranslateOutcome::Cancelled);
            }
            on_token(token);
        }
        if let Some(flag) = &self.cancel_flag_after_stream {
            flag.store(true, Ordering::SeqCst);
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

fn ready() -> Result<QuickPrepareOutcome, auratranslate_lib::core::i18n::IpcError> {
    Ok(QuickPrepareOutcome::Ready(PreparedTranslateCall {
        endpoint: "http://localhost/v1".to_owned(),
        model: "model-a".to_owned(),
        temperature: None,
        max_tokens: None,
        api_key: String::new(),
        prompt: "p".to_owned(),
        system: None,
    }))
}

fn not_cancelled() -> Arc<dyn Fn() -> bool + Send + Sync> {
    Arc::new(|| false)
}

fn run(
    store: Option<&QuickTranslationStore>,
    provider: &Fake,
    text: &str,
    retranslate: bool,
    should_cancel: Arc<dyn Fn() -> bool + Send + Sync>,
) -> (
    Result<QuickTranslateOutcomeWire, QuickTranslateError<FakeError>>,
    String,
) {
    let mut streamed = String::new();
    let mut on_token = |t: &str| streamed.push_str(t);
    let outcome = tauri::async_runtime::block_on(quick_translate(
        store,
        provider,
        ready,
        text,
        retranslate,
        &mut on_token,
        should_cancel,
    ));
    (outcome, streamed)
}

#[test]
fn the_key_folds_nfc_and_trim_so_two_spellings_share_one_row() {
    assert_eq!(quick_key("  e\u{301}  "), "\u{e9}");
    let dir = temp_dir("key");
    let store = open_store(&dir);
    assert!(
        store
            .save("  \u{e9}\n", "first", "m", false, always())
            .unwrap()
    );
    assert!(
        !store
            .save("e\u{301}", "second", "m", false, always())
            .unwrap()
    );
    assert_eq!(store.count().unwrap(), 1);
    assert_eq!(
        store.lookup("e\u{301} ").unwrap().unwrap().translated_text,
        "first"
    );
    drop(store);
    cleanup(&dir);
}

#[test]
fn a_plain_save_keeps_the_first_row_and_a_replace_overwrites_it() {
    let dir = temp_dir("replace");
    let store = open_store(&dir);
    assert!(store.save("src", "old", "m1", false, always()).unwrap());
    assert!(!store.save("src", "new", "m2", false, always()).unwrap());
    let hit = store.lookup("src").unwrap().unwrap();
    assert_eq!(
        (hit.translated_text.as_str(), hit.model.as_str()),
        ("old", "m1")
    );
    assert!(store.save("src", "new", "m2", true, always()).unwrap());
    let hit = store.lookup("src").unwrap().unwrap();
    assert_eq!(
        (hit.translated_text.as_str(), hit.model.as_str()),
        ("new", "m2")
    );
    assert_eq!(store.count().unwrap(), 1);
    drop(store);
    cleanup(&dir);
}

#[test]
fn the_currency_check_runs_inside_the_write_job() {
    let dir = temp_dir("job");
    let store = open_store(&dir);
    assert!(!store.save("src", "t", "m", false, || false).unwrap());
    assert_eq!(store.count().unwrap(), 0);
    let seen_on = Arc::new(Mutex::new(None));
    let sink = Arc::clone(&seen_on);
    let caller = std::thread::current().id();
    store
        .save("src", "t", "m", false, move || {
            *sink.lock().unwrap() = Some(std::thread::current().id());
            true
        })
        .unwrap();
    assert_ne!(seen_on.lock().unwrap().expect("check ran"), caller);
    drop(store);
    cleanup(&dir);
}

#[test]
fn a_garbage_file_is_rebuilt_empty_and_usable() {
    let dir = temp_dir("garbage");
    fs::write(
        dir.join("quick-translation.db"),
        b"this is not a database at all",
    )
    .unwrap();
    let store = open_store(&dir);
    assert_eq!(store.count().unwrap(), 0);
    assert!(store.save("a", "b", "m", false, always()).unwrap());
    drop(store);
    cleanup(&dir);
}

#[test]
fn a_file_of_another_schema_version_is_deleted_and_rebuilt() {
    let dir = temp_dir("version");
    let path = dir.join("quick-translation.db");
    {
        let conn = rusqlite::Connection::open(&path).unwrap();
        conn.execute_batch("CREATE TABLE quick_translation (x); PRAGMA user_version = 99;")
            .unwrap();
    }
    let store = open_store(&dir);
    assert_eq!(store.count().unwrap(), 0);
    assert!(store.save("a", "b", "m", false, always()).unwrap());
    drop(store);
    cleanup(&dir);
}

#[test]
fn a_directory_in_place_of_the_file_is_an_open_failure_not_a_panic() {
    let dir = temp_dir("unopenable");
    fs::create_dir_all(dir.join("quick-translation.db").join("x")).unwrap();
    assert!(QuickTranslationStore::open(dir.join("quick-translation.db")).is_err());
    cleanup(&dir);
}

#[test]
fn a_missing_store_is_reported_before_any_network_call() {
    let (provider, calls) = Fake::ok(vec!["x"]);
    let (outcome, streamed) = run(None, &provider, "src", false, not_cancelled());
    assert_eq!(
        outcome.unwrap(),
        QuickTranslateOutcomeWire::StoreUnavailable
    );
    assert_eq!(calls.load(Ordering::SeqCst), 0);
    assert!(streamed.is_empty());
}

#[test]
fn a_cache_hit_sends_nothing_and_reads_no_key_even_without_retranslate() {
    let dir = temp_dir("hit");
    let store = open_store(&dir);
    store
        .save("src", "cached text", "model-z", false, always())
        .unwrap();
    let (provider, calls) = Fake::ok(vec!["x"]);
    let mut on_token = |_: &str| {};
    let outcome = tauri::async_runtime::block_on(quick_translate(
        Some(&store),
        &provider,
        || panic!("a cache hit must not resolve the call configuration"),
        "  src ",
        false,
        &mut on_token,
        not_cancelled(),
    ))
    .unwrap();
    assert_eq!(
        outcome,
        QuickTranslateOutcomeWire::Cached {
            text: "cached text".to_owned(),
            model: "model-z".to_owned()
        }
    );
    assert_eq!(calls.load(Ordering::SeqCst), 0);
    drop(store);
    cleanup(&dir);
}

#[test]
fn a_miss_streams_then_saves_one_row_and_the_next_call_is_a_hit() {
    let dir = temp_dir("miss");
    let store = open_store(&dir);
    let (provider, calls) = Fake::ok(vec!["Mot ", "hai."]);
    let (outcome, streamed) = run(Some(&store), &provider, "src", false, not_cancelled());
    match outcome.unwrap() {
        QuickTranslateOutcomeWire::Done {
            model,
            usage,
            maybe_truncated,
        } => {
            assert_eq!(model, "model-a");
            assert_eq!(usage.expect("usage").total_tokens, 5);
            assert!(!maybe_truncated);
        }
        other => panic!("expected done, got {other:?}"),
    }
    assert_eq!(streamed, "Mot hai.");
    assert_eq!(store.count().unwrap(), 1);
    assert_eq!(
        store.lookup("src").unwrap().unwrap().translated_text,
        "Mot hai."
    );
    let (again, _) = run(Some(&store), &provider, "src", false, not_cancelled());
    assert!(matches!(
        again.unwrap(),
        QuickTranslateOutcomeWire::Cached { .. }
    ));
    assert_eq!(calls.load(Ordering::SeqCst), 1);
    drop(store);
    cleanup(&dir);
}

#[test]
fn retranslate_sends_despite_a_hit_and_replaces_the_row() {
    let dir = temp_dir("retranslate");
    let store = open_store(&dir);
    store
        .save("src", "old", "model-old", false, always())
        .unwrap();
    let (provider, calls) = Fake::ok(vec!["new"]);
    let (outcome, _) = run(Some(&store), &provider, "src", true, not_cancelled());
    assert!(matches!(
        outcome.unwrap(),
        QuickTranslateOutcomeWire::Done { .. }
    ));
    assert_eq!(calls.load(Ordering::SeqCst), 1);
    let hit = store.lookup("src").unwrap().unwrap();
    assert_eq!(
        (hit.translated_text.as_str(), hit.model.as_str()),
        ("new", "model-a")
    );
    assert_eq!(store.count().unwrap(), 1);
    drop(store);
    cleanup(&dir);
}

#[test]
fn a_possibly_truncated_result_is_returned_but_never_stored() {
    let dir = temp_dir("truncated");
    let store = open_store(&dir);
    let (mut provider, _) = Fake::ok(vec!["partial"]);
    provider.finish_reason = Some("length");
    let (outcome, streamed) = run(Some(&store), &provider, "src", false, not_cancelled());
    assert!(matches!(
        outcome.unwrap(),
        QuickTranslateOutcomeWire::Done {
            maybe_truncated: true,
            ..
        }
    ));
    assert_eq!(streamed, "partial");
    assert_eq!(store.count().unwrap(), 0);
    provider.finish_reason = None;
    let (outcome, _) = run(Some(&store), &provider, "src", false, not_cancelled());
    assert!(matches!(
        outcome.unwrap(),
        QuickTranslateOutcomeWire::Done {
            maybe_truncated: true,
            ..
        }
    ));
    assert_eq!(store.count().unwrap(), 0);
    drop(store);
    cleanup(&dir);
}

#[test]
fn a_cancelled_run_and_a_failed_run_leave_no_row() {
    let dir = temp_dir("cancel");
    let store = open_store(&dir);
    let (provider, calls) = Fake::ok(vec!["a", "b"]);
    let (outcome, _) = run(Some(&store), &provider, "src", false, Arc::new(|| true));
    assert_eq!(outcome.unwrap(), QuickTranslateOutcomeWire::Cancelled);
    assert_eq!(calls.load(Ordering::SeqCst), 0);

    let (mut failing, _) = Fake::ok(vec!["a"]);
    failing.fail = true;
    let (outcome, _) = run(Some(&store), &failing, "src", false, not_cancelled());
    assert!(matches!(outcome, Err(QuickTranslateError::Provider(_))));
    assert_eq!(store.count().unwrap(), 0);
    drop(store);
    cleanup(&dir);
}

#[test]
fn a_run_cancelled_after_the_stream_ended_but_before_the_write_stores_nothing() {
    let dir = temp_dir("late-cancel");
    let store = open_store(&dir);
    let flag = Arc::new(AtomicBool::new(false));
    let (mut provider, _) = Fake::ok(vec!["done"]);
    provider.cancel_flag_after_stream = Some(Arc::clone(&flag));
    let check = Arc::clone(&flag);
    let (outcome, _) = run(
        Some(&store),
        &provider,
        "src",
        false,
        Arc::new(move || check.load(Ordering::SeqCst)),
    );
    assert!(matches!(
        outcome.unwrap(),
        QuickTranslateOutcomeWire::Done { .. }
    ));
    assert_eq!(store.count().unwrap(), 0, "the job saw a stale generation");
    drop(store);
    cleanup(&dir);
}

#[test]
fn an_unconfigured_provider_is_reached_before_the_network() {
    let dir = temp_dir("unconf");
    let store = open_store(&dir);
    let (provider, calls) = Fake::ok(vec!["x"]);
    let mut on_token = |_: &str| {};
    let outcome = tauri::async_runtime::block_on(quick_translate(
        Some(&store),
        &provider,
        || Ok(QuickPrepareOutcome::NotConfigured),
        "src",
        false,
        &mut on_token,
        not_cancelled(),
    ))
    .unwrap();
    assert_eq!(outcome, QuickTranslateOutcomeWire::NotConfigured);
    assert_eq!(calls.load(Ordering::SeqCst), 0);
    drop(store);
    cleanup(&dir);
}

#[test]
fn the_lookup_wire_distinguishes_hit_miss_and_missing_store() {
    let dir = temp_dir("lookup");
    let store = open_store(&dir);
    assert_eq!(
        quick_lookup_wire(Some(&store), "src"),
        QuickLookupWire::Miss
    );
    store.save("src", "t", "m", false, always()).unwrap();
    assert_eq!(
        quick_lookup_wire(Some(&store), " src"),
        QuickLookupWire::Hit {
            text: "t".to_owned(),
            model: "m".to_owned()
        }
    );
    assert_eq!(
        quick_lookup_wire(None, "src"),
        QuickLookupWire::StoreUnavailable
    );
    drop(store);
    cleanup(&dir);
}

#[test]
fn a_sentence_translated_and_saved_touches_no_work_table() {
    let root = temp_dir("work");
    let dir = temp_dir("work-store");
    let open: OpenWork =
        create_work_from_text(&root, "Quick", "zh", "", "一二三。".to_owned()).expect("work");
    let before = snapshot(&open);
    let store = open_store(&dir);
    let (provider, _) = Fake::ok(vec!["Mot hai ba."]);
    let (outcome, _) = run(Some(&store), &provider, "一二三。", false, not_cancelled());
    assert!(matches!(
        outcome.unwrap(),
        QuickTranslateOutcomeWire::Done { .. }
    ));
    assert_eq!(store.count().unwrap(), 1);
    assert_eq!(snapshot(&open), before, "the call changed a Work row");
    drop((store, open));
    cleanup(&root);
    cleanup(&dir);
}

fn snapshot(open: &OpenWork) -> Vec<String> {
    open.store
        .read(|conn| {
            let mut out = Vec::new();
            for table in ["segment", "tm_unit", "glossary_entry", "glossary_candidate"] {
                let mut stmt = conn.prepare(&format!("SELECT * FROM {table} ORDER BY 1"))?;
                let cols = stmt.column_count();
                let mut rows = stmt.query([])?;
                while let Some(row) = rows.next()? {
                    let mut line = format!("{table}:");
                    for i in 0..cols {
                        let v: rusqlite::types::Value = row.get(i)?;
                        line.push_str(&format!("{v:?}|"));
                    }
                    out.push(line);
                }
            }
            Ok(out)
        })
        .expect("snapshot")
}

#[test]
fn a_cancel_names_its_run_and_a_stale_cancel_does_not_stop_a_newer_run() {
    let quick = QuickTranslateGeneration::default();
    let batch = AiTranslateGeneration::default();
    let first_ticket = quick
        .begin(QuickSurface::GlossaryQueue, 7)
        .expect("free slot");
    let first = first_ticket.generation;
    assert!(
        !quick.cancel(8),
        "a cancel for another run_id did something"
    );
    assert!(quick.is_current(first));
    let second_ticket = quick
        .begin(QuickSurface::GlossaryQueue, 9)
        .expect("same surface replaces");
    let second = second_ticket.generation;
    assert!(!quick.is_current(first), "a new run supersedes the old one");
    assert!(
        !quick.cancel(7),
        "a late cancel of the old run killed the new one"
    );
    assert!(quick.is_current(second));
    assert!(quick.cancel(9));
    assert!(!quick.is_current(second));
    assert!(
        batch.is_current(0),
        "quick-translation cancel moved the batch generation"
    );
}

#[test]
fn a_stale_cancel_lets_the_running_sentence_finish_and_be_stored_and_its_own_cancel_does_not() {
    let dir = temp_dir("stale-cancel");
    let store = open_store(&dir);
    let runs = QuickTranslateGeneration::default();

    let ticket = runs
        .begin(QuickSurface::GlossaryQueue, 5)
        .expect("free slot");
    let generation = ticket.generation;
    assert!(!runs.cancel(4));
    let watched = runs.clone();
    let should_cancel: Arc<dyn Fn() -> bool + Send + Sync> =
        Arc::new(move || !watched.is_current(generation));
    let (provider, _) = Fake::ok(vec!["giu lai"]);
    let (outcome, _) = run(Some(&store), &provider, "stale", false, should_cancel);
    assert!(matches!(
        outcome.unwrap(),
        QuickTranslateOutcomeWire::Done { .. }
    ));
    assert_eq!(store.count().unwrap(), 1, "a stale cancel stopped the save");

    drop(ticket);
    let ticket = runs
        .begin(QuickSurface::GlossaryQueue, 6)
        .expect("free slot");
    let generation = ticket.generation;
    assert!(runs.cancel(6));
    let watched = runs.clone();
    let should_cancel: Arc<dyn Fn() -> bool + Send + Sync> =
        Arc::new(move || !watched.is_current(generation));
    let (provider, _) = Fake::ok(vec!["bo di"]);
    let (outcome, _) = run(Some(&store), &provider, "own", false, should_cancel);
    assert!(matches!(
        outcome.unwrap(),
        QuickTranslateOutcomeWire::Cancelled
    ));
    assert_eq!(store.count().unwrap(), 1, "a cancelled run left a row");
    drop(store);
    cleanup(&dir);
}

#[test]
fn the_wires_return_a_hit_without_a_network_call_and_a_miss_reaches_the_send() {
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
        .save("cached sentence", "da dich", "model-z", false, always())
        .unwrap();

    let app = mock_builder()
        .build(mock_context(noop_assets()))
        .expect("app");
    app.manage(global);
    app.manage(quick_store);
    app.manage::<OpenWorkState>(Mutex::new(None));
    app.manage(AiTranslateGeneration::default());
    app.manage(QuickTranslateGeneration::default());
    let handle = app.handle().clone();
    let batch = app.state::<AiTranslateGeneration>();
    let quick = app.state::<QuickTranslateGeneration>();
    let channel = || tauri::ipc::Channel::<String>::new(|_| Ok(()));

    let hit = tauri::async_runtime::block_on(wire::ai_context_reference_translate(
        handle.clone(),
        "cached sentence".to_owned(),
        auratranslate_lib::commands::aitranslate::QuickTranslateTrigger::Manual,
        1,
        false,
        channel(),
    ))
    .expect("hit");
    assert_eq!(
        hit,
        QuickTranslateOutcomeWire::Cached {
            text: "da dich".to_owned(),
            model: "model-z".to_owned()
        }
    );

    let miss = tauri::async_runtime::block_on(wire::ai_context_reference_translate(
        handle.clone(),
        "other sentence".to_owned(),
        auratranslate_lib::commands::aitranslate::QuickTranslateTrigger::Manual,
        2,
        false,
        channel(),
    ));
    let error = miss.expect_err("a closed port is a connection error, which proves the send ran");
    assert_ne!(error.code(), "store.open_failed");

    assert!(quick.is_current(2));
    wire::ai_context_reference_cancel(handle.clone(), 1);
    wire::ai_context_reference_cancel(handle.clone(), 2);
    assert!(
        quick.is_current(2),
        "a cancel of a run that already ended moved the generation"
    );
    assert!(
        batch.is_current(0),
        "the quick translation moved the batch generation"
    );

    assert_eq!(
        wire::quick_translation_lookup(handle, "cached sentence".to_owned()),
        QuickLookupWire::Hit {
            text: "da dich".to_owned(),
            model: "model-z".to_owned()
        }
    );
    cleanup(&gdir);
    cleanup(&qdir);
}

#[test]
fn the_wire_reports_an_unmanaged_store_without_calling_out() {
    use auratranslate_lib::commands::aitranslate::wire;
    use auratranslate_lib::commands::project::OpenWorkState;
    use tauri::Manager as _;
    use tauri::test::{mock_builder, mock_context, noop_assets};

    let app = mock_builder()
        .build(mock_context(noop_assets()))
        .expect("app");
    app.manage::<OpenWorkState>(Mutex::new(None));
    app.manage(QuickTranslateGeneration::default());
    let handle = app.handle().clone();
    let outcome = tauri::async_runtime::block_on(wire::ai_context_reference_translate(
        handle.clone(),
        "s".to_owned(),
        auratranslate_lib::commands::aitranslate::QuickTranslateTrigger::Manual,
        1,
        false,
        tauri::ipc::Channel::<String>::new(|_| Ok(())),
    ))
    .expect("outcome");
    assert_eq!(outcome, QuickTranslateOutcomeWire::StoreUnavailable);
    assert_eq!(
        wire::quick_translation_lookup(handle, "s".to_owned()),
        QuickLookupWire::StoreUnavailable
    );
}

#[test]
fn automatic_command_wires_reject_off_incomplete_and_real_activity_before_provider_calls() {
    use auratranslate_lib::commands::aitranslate::{
        QuickBatchOutcomeWire, QuickTranslateTrigger, wire,
    };
    use auratranslate_lib::commands::project::OpenWorkState;
    use auratranslate_lib::core::aiconfig::store::{write_quick_auto, write_quick_fields};
    use auratranslate_lib::core::aiconfig::{AiConfigField, QuickAiConfigField, write_field};
    use auratranslate_lib::core::store::{Store, StoreSpec};
    use tauri::Manager as _;
    use tauri::test::{mock_builder, mock_context, noop_assets};

    let fixture = CountingServer::start();
    let (endpoint, calls) = (fixture.endpoint.clone(), fixture.calls.clone());
    let dir = temp_dir("auto-admission");
    let global = Store::open(StoreSpec::global(dir.join("global.db"))).unwrap();
    write_field(&global, AiConfigField::Provider, "ollama").unwrap();
    write_field(&global, AiConfigField::Endpoint, &endpoint).unwrap();
    write_field(&global, AiConfigField::Model, "main-fixture").unwrap();
    let dedicated = |model: &str, url: &str| {
        vec![
            (QuickAiConfigField::Provider, "ollama".to_owned()),
            (QuickAiConfigField::Endpoint, url.to_owned()),
            (QuickAiConfigField::Model, model.to_owned()),
        ]
    };
    write_quick_fields(&global, &dedicated("quick-fixture", &endpoint)).unwrap();
    let app = mock_builder().build(mock_context(noop_assets())).unwrap();
    app.manage(global);
    app.manage(open_store(&dir));
    app.manage::<OpenWorkState>(Mutex::new(None));
    app.manage(AiTranslateGeneration::default());
    app.manage(QuickTranslateGeneration::default());
    let handle = app.handle().clone();
    for (model, url) in [
        ("quick-fixture", endpoint.as_str()),
        ("", ""),
        ("quick-fixture", ""),
        ("", endpoint.as_str()),
    ] {
        write_quick_fields(&app.state::<Store>(), &dedicated(model, url)).unwrap();
        if model.is_empty() || url.is_empty() {
            app.state::<Store>().write(|tx| {
                tx.execute("INSERT INTO config_value (kind,key,value,updated_at) VALUES ('app_config','quick_translate_auto','on','fixture') ON CONFLICT(kind,key) DO UPDATE SET value='on'", [])?;
                Ok(())
            }).unwrap();
        }
        let before = calls.load(Ordering::SeqCst);
        let queue = tauri::async_runtime::block_on(wire::ai_context_reference_translate(
            handle.clone(),
            "queue automatic source".to_owned(),
            QuickTranslateTrigger::Auto,
            101,
            false,
            tauri::ipc::Channel::new(|_| Ok(())),
        ));
        assert_eq!(
            calls.load(Ordering::SeqCst) - before,
            0,
            "queue auto gate called provider with switch off or incomplete config"
        );
        assert_eq!(queue.unwrap(), QuickTranslateOutcomeWire::AutoOff);
        let batch = tauri::async_runtime::block_on(wire::quick_translate_blocks(
            handle.clone(),
            vec!["preview automatic source".to_owned()],
            QuickTranslateTrigger::Auto,
            false,
            102,
            tauri::ipc::Channel::new(|_| Ok(())),
        ));
        assert_eq!(
            calls.load(Ordering::SeqCst) - before,
            0,
            "preview auto gate called provider with switch off or incomplete config"
        );
        assert_eq!(batch.unwrap(), QuickBatchOutcomeWire::AutoOff);
    }
    write_quick_fields(
        &app.state::<Store>(),
        &dedicated("quick-fixture", &endpoint),
    )
    .unwrap();
    write_quick_auto(&app.state::<Store>(), true).unwrap();
    let generation = app.state::<AiTranslateGeneration>();
    let active = generation.begin_activity(|_| {});
    let before = calls.load(Ordering::SeqCst);
    let paused = tauri::async_runtime::block_on(wire::ai_context_reference_translate(
        handle.clone(),
        "paused source".to_owned(),
        QuickTranslateTrigger::Auto,
        103,
        false,
        tauri::ipc::Channel::new(|_| Ok(())),
    ))
    .unwrap();
    assert_eq!(paused, QuickTranslateOutcomeWire::Paused);
    let paused_batch = tauri::async_runtime::block_on(wire::quick_translate_blocks(
        handle.clone(),
        vec!["paused block".to_owned()],
        QuickTranslateTrigger::Auto,
        false,
        104,
        tauri::ipc::Channel::new(|_| Ok(())),
    ))
    .unwrap();
    assert_eq!(paused_batch, QuickBatchOutcomeWire::Paused);
    assert_eq!(calls.load(Ordering::SeqCst) - before, 0);
    let manual = tauri::async_runtime::block_on(wire::ai_context_reference_translate(
        handle.clone(),
        "manual during real activity".to_owned(),
        QuickTranslateTrigger::Manual,
        105,
        false,
        tauri::ipc::Channel::new(|_| Ok(())),
    ))
    .unwrap_or_else(|error| {
        panic!(
            "manual error {error:?}; accepted {}",
            calls.load(Ordering::SeqCst)
        )
    });
    assert!(matches!(manual, QuickTranslateOutcomeWire::Done { .. }));
    assert_eq!(calls.load(Ordering::SeqCst) - before, 1);
    wire::ai_context_reference_cancel(handle, 105);
    assert!(
        generation.is_current(0),
        "quick command/cancel changed real generation"
    );
    assert_eq!(generation.activity().active_count, 1);
    drop(active);
    fixture.finish();
    drop(app);
    cleanup(&dir);
}

struct CountingServer {
    endpoint: String,
    calls: Arc<AtomicUsize>,
    stop: Arc<AtomicBool>,
    thread: std::thread::JoinHandle<()>,
}

impl CountingServer {
    fn start() -> Self {
        use std::io::{Read, Write};
        let listener = std::net::TcpListener::bind("127.0.0.1:0").expect("fixture loopback");
        listener.set_nonblocking(true).unwrap();
        let endpoint = format!("http://{}/v1", listener.local_addr().unwrap());
        let calls = Arc::new(AtomicUsize::new(0));
        let stop = Arc::new(AtomicBool::new(false));
        let (count, stopped) = (calls.clone(), stop.clone());
        let thread = std::thread::spawn(move || {
            while !stopped.load(Ordering::SeqCst) {
                match listener.accept() {
                    Ok((mut stream, _)) => {
                        stream.set_nonblocking(false).unwrap();
                        stream
                            .set_read_timeout(Some(std::time::Duration::from_secs(3)))
                            .unwrap();
                        let mut body = Vec::new();
                        let mut chunk = [0; 4096];
                        loop {
                            let n = stream.read(&mut chunk).unwrap_or(0);
                            if n == 0 {
                                break;
                            }
                            body.extend_from_slice(&chunk[..n]);
                            if let Some(header_end) = body.windows(4).position(|w| w == b"\r\n\r\n")
                            {
                                let headers =
                                    String::from_utf8_lossy(&body[..header_end]).to_lowercase();
                                let length = headers
                                    .lines()
                                    .find_map(|l| l.strip_prefix("content-length:"))
                                    .and_then(|v| v.trim().parse::<usize>().ok())
                                    .unwrap_or(0);
                                if body.len() >= header_end + 4 + length {
                                    break;
                                }
                            }
                        }
                        count.fetch_add(1, Ordering::SeqCst);
                        let frames = "data: {\"choices\":[{\"delta\":{\"content\":\"translated\"},\"finish_reason\":null}]}\n\ndata: {\"choices\":[{\"delta\":{},\"finish_reason\":\"stop\"}]}\n\ndata: [DONE]\n\n";
                        let response = format!(
                            "HTTP/1.1 200 OK\r\nContent-Type: text/event-stream\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                            frames.len(),
                            frames
                        );
                        let _ = stream.write_all(response.as_bytes());
                    }
                    Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                        std::thread::sleep(std::time::Duration::from_millis(2))
                    }
                    Err(error) => panic!("fixture accept: {error}"),
                }
            }
        });
        Self {
            endpoint,
            calls,
            stop,
            thread,
        }
    }

    fn accepted(&self) -> usize {
        self.calls.load(Ordering::SeqCst)
    }

    fn finish(self) {
        self.stop.store(true, Ordering::SeqCst);
        self.thread.join().unwrap();
    }
}

#[derive(Clone, Copy, Debug)]
enum AfterGate {
    Nothing,
    ClearQuickEndpoint,
    ClearQuickEndpointAndModel,
    SwitchAutoOff,
}

fn apply_after_gate(global: &auratranslate_lib::core::store::Store, change: AfterGate) {
    use auratranslate_lib::core::aiconfig::QuickAiConfigField;
    use auratranslate_lib::core::aiconfig::store::{write_quick_auto, write_quick_fields};
    let set = |endpoint: &str, model: &str| {
        write_quick_fields(
            global,
            &[
                (QuickAiConfigField::Provider, "ollama".to_owned()),
                (QuickAiConfigField::Endpoint, endpoint.to_owned()),
                (QuickAiConfigField::Model, model.to_owned()),
            ],
        )
        .unwrap();
    };
    match change {
        AfterGate::Nothing => {}
        AfterGate::ClearQuickEndpoint => set("", "quick-fixture"),
        AfterGate::ClearQuickEndpointAndModel => set("", ""),
        AfterGate::SwitchAutoOff => write_quick_auto(global, false).unwrap(),
    }
}

fn configure_main_and_quick(
    global: &auratranslate_lib::core::store::Store,
    main: &str,
    quick: &str,
) {
    use auratranslate_lib::core::aiconfig::store::{write_quick_auto, write_quick_fields};
    use auratranslate_lib::core::aiconfig::{AiConfigField, QuickAiConfigField, write_field};
    write_field(global, AiConfigField::Provider, "ollama").unwrap();
    write_field(global, AiConfigField::Endpoint, main).unwrap();
    write_field(global, AiConfigField::Model, "main-fixture").unwrap();
    write_quick_fields(
        global,
        &[
            (QuickAiConfigField::Provider, "ollama".to_owned()),
            (QuickAiConfigField::Endpoint, quick.to_owned()),
            (QuickAiConfigField::Model, "quick-fixture".to_owned()),
        ],
    )
    .unwrap();
    write_quick_auto(global, true).unwrap();
}

#[test]
fn an_automatic_blocks_run_whose_dedicated_config_changes_after_the_gate_never_reaches_a_provider()
{
    use auratranslate_lib::commands::aitranslate::{
        QuickBatchOutcomeWire, QuickTranslateTrigger, wire,
    };
    use auratranslate_lib::commands::project::OpenWorkState;
    use auratranslate_lib::core::store::Store;
    use tauri::Manager as _;
    use tauri::test::{mock_builder, mock_context, noop_assets};

    for (change, expect_done) in [
        (AfterGate::ClearQuickEndpointAndModel, false),
        (AfterGate::Nothing, true),
        (AfterGate::ClearQuickEndpoint, false),
        (AfterGate::SwitchAutoOff, false),
    ] {
        let main = CountingServer::start();
        let quick = CountingServer::start();
        let dir = temp_dir("auto-after-gate-blocks");
        let global = Store::open(auratranslate_lib::core::store::StoreSpec::global(
            dir.join("global.db"),
        ))
        .unwrap();
        configure_main_and_quick(&global, &main.endpoint, &quick.endpoint);
        let store = open_store(&dir);
        store
            .save("remembered", "da dich", "m", false, always())
            .unwrap();
        let app = mock_builder().build(mock_context(noop_assets())).unwrap();
        app.manage(global);
        app.manage(store);
        app.manage::<OpenWorkState>(Mutex::new(None));
        app.manage(AiTranslateGeneration::default());
        app.manage(QuickTranslateGeneration::default());
        let handle = app.handle().clone();
        let changed = Arc::new(AtomicBool::new(false));
        let on_cached = {
            let handle = handle.clone();
            let changed = changed.clone();
            move |_event| {
                if !changed.swap(true, Ordering::SeqCst) {
                    apply_after_gate(&handle.state::<Store>(), change);
                }
                Ok(())
            }
        };
        let outcome = tauri::async_runtime::block_on(wire::quick_translate_blocks(
            handle,
            vec!["remembered".to_owned(), "new text".to_owned()],
            QuickTranslateTrigger::Auto,
            false,
            301,
            tauri::ipc::Channel::new(on_cached),
        ))
        .unwrap();
        assert!(
            changed.load(Ordering::SeqCst),
            "{change:?}: the cached event never reached the channel"
        );
        assert_eq!(
            main.accepted(),
            0,
            "{change:?}: an automatic run reached the main endpoint"
        );
        if expect_done {
            assert!(matches!(outcome, QuickBatchOutcomeWire::Done { .. }));
            assert_eq!(quick.accepted(), 1);
        } else {
            assert_eq!(outcome, QuickBatchOutcomeWire::AutoOff, "{change:?}");
            assert_eq!(quick.accepted(), 0, "{change:?}");
        }
        drop(app);
        main.finish();
        quick.finish();
        cleanup(&dir);
    }
}

#[test]
fn an_automatic_glossary_run_whose_dedicated_config_changes_after_the_gate_never_reaches_a_provider()
 {
    use auratranslate_lib::commands::aitranslate::{
        AiTranslateActivityWire, QuickTranslateTrigger, prepare_context_reference_call,
        quick_auto_gate,
    };
    use auratranslate_lib::core::ai::client::OpenAiChatClient;
    use auratranslate_lib::core::store::{Store, StoreSpec};

    for (change, expect_done) in [
        (AfterGate::ClearQuickEndpointAndModel, false),
        (AfterGate::Nothing, true),
        (AfterGate::ClearQuickEndpoint, false),
        (AfterGate::SwitchAutoOff, false),
    ] {
        let main = CountingServer::start();
        let quick = CountingServer::start();
        let dir = temp_dir("auto-after-gate-glossary");
        let global = Store::open(StoreSpec::global(dir.join("global.db"))).unwrap();
        configure_main_and_quick(&global, &main.endpoint, &quick.endpoint);
        let store = open_store(&dir);
        let admitted = quick_auto_gate(
            Some(&global),
            &AiTranslateActivityWire::default(),
            QuickTranslateTrigger::Auto,
        )
        .unwrap();
        assert!(admitted.is_none(), "{change:?}: the gate must admit");
        apply_after_gate(&global, change);
        let provider = OpenAiChatClient::new();
        let mut on_token = |_: &str| {};
        let outcome = tauri::async_runtime::block_on(quick_translate(
            Some(&store),
            &provider,
            || {
                prepare_context_reference_call(
                    Some(&global),
                    None,
                    "a sentence",
                    QuickTranslateTrigger::Auto,
                )
            },
            "a sentence",
            false,
            &mut on_token,
            not_cancelled(),
        ));
        assert_eq!(main.accepted(), 0, "{change:?}: reached the main endpoint");
        if expect_done {
            assert!(matches!(
                outcome,
                Ok(QuickTranslateOutcomeWire::Done { .. })
            ));
            assert_eq!(quick.accepted(), 1);
        } else {
            assert!(
                matches!(outcome, Ok(QuickTranslateOutcomeWire::AutoOff)),
                "{change:?}"
            );
            assert_eq!(quick.accepted(), 0, "{change:?}");
        }
        main.finish();
        quick.finish();
        cleanup(&dir);
    }
}

#[test]
fn a_manual_quick_run_keeps_the_main_fallback_when_both_quick_fields_are_empty_and_rejects_half_set()
 {
    use auratranslate_lib::commands::aitranslate::{
        QuickTranslateTrigger, prepare_context_reference_call,
    };
    use auratranslate_lib::core::store::{Store, StoreSpec};

    let dir = temp_dir("manual-r1");
    let global = Store::open(StoreSpec::global(dir.join("global.db"))).unwrap();
    configure_main_and_quick(&global, "http://main.example/v1", "http://quick.example/v1");
    apply_after_gate(&global, AfterGate::ClearQuickEndpointAndModel);
    let prepare = |trigger| prepare_context_reference_call(Some(&global), None, "s", trigger);
    match prepare(QuickTranslateTrigger::Manual).unwrap() {
        QuickPrepareOutcome::Ready(call) => assert_eq!(call.endpoint, "http://main.example/v1"),
        _ => panic!("both quick fields empty falls back to the main config"),
    }
    apply_after_gate(&global, AfterGate::ClearQuickEndpoint);
    assert!(matches!(
        prepare(QuickTranslateTrigger::Manual).unwrap(),
        QuickPrepareOutcome::NotConfigured
    ));
    cleanup(&dir);
}

fn backdate(dir: &Path, source: &str, days: i64) {
    let conn = rusqlite::Connection::open(dir.join("quick-translation.db")).expect("open raw");
    conn.execute(
        "UPDATE quick_translation SET created_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now', ?1) \
         WHERE source_text = ?2",
        rusqlite::params![format!("-{days} days"), source],
    )
    .expect("backdate");
}

fn size_on_disk(dir: &Path) -> u64 {
    ["quick-translation.db", "quick-translation.db-wal"]
        .iter()
        .map(|name| fs::metadata(dir.join(name)).map(|m| m.len()).unwrap_or(0))
        .sum()
}

#[test]
fn stats_count_rows_per_model_and_report_the_file_size() {
    let dir = temp_dir("stats");
    let store = open_store(&dir);
    for (text, model) in [("a", "m1"), ("b", "m1"), ("c", "m2")] {
        store.save(text, "x", model, false, always()).unwrap();
    }
    let stats = quick_stats_wire(Some(&store)).expect("stats");
    assert_eq!(stats.count, 3);
    assert!(stats.bytes > 0 && stats.bytes == size_on_disk(&dir));
    let by_model: Vec<(String, u64)> = stats
        .by_model
        .iter()
        .map(|m| (m.model.clone(), m.count))
        .collect();
    assert_eq!(by_model, vec![("m1".to_owned(), 2), ("m2".to_owned(), 1)]);

    let empty_dir = temp_dir("stats-empty");
    let empty = open_store(&empty_dir);
    let stats = quick_stats_wire(Some(&empty)).expect("stats");
    assert_eq!((stats.count, stats.by_model.len()), (0, 0));

    assert!(
        quick_stats_wire(None).is_err(),
        "no store is an error, not a panic"
    );
    drop((store, empty));
    cleanup(&dir);
    cleanup(&empty_dir);
}

#[test]
fn counting_and_purging_older_than_n_days_touch_only_the_old_rows() {
    let dir = temp_dir("older");
    let store = open_store(&dir);
    for text in ["old1", "old2", "fresh"] {
        store.save(text, "x", "m", false, always()).unwrap();
    }
    backdate(&dir, "old1", 40);
    backdate(&dir, "old2", 31);

    assert_eq!(quick_count_older_wire(Some(&store), 30).unwrap(), 2);
    assert_eq!(quick_count_older_wire(Some(&store), 35).unwrap(), 1);
    for bad in [0, -1] {
        assert!(quick_count_older_wire(Some(&store), bad).is_err());
        assert!(
            quick_purge_wire(
                Some(&store),
                None,
                QuickPurgeScopeWire::OlderThan { days: bad }
            )
            .is_err()
        );
    }
    assert_eq!(
        store.count().unwrap(),
        3,
        "a rejected days value deleted rows"
    );

    let outcome = quick_purge_wire(
        Some(&store),
        None,
        QuickPurgeScopeWire::OlderThan { days: 30 },
    )
    .expect("purge");
    assert_eq!(outcome.deleted, 2);
    assert_eq!(outcome.compact_error, None);
    assert_eq!(store.count().unwrap(), 1);
    assert!(store.lookup("fresh").unwrap().is_some());
    assert!(store.lookup("old1").unwrap().is_none());
    drop(store);
    cleanup(&dir);
}

#[test]
fn purging_all_rows_shrinks_db_plus_wal_on_disk() {
    let dir = temp_dir("purge-size");
    let store = open_store(&dir);
    let big = "y".repeat(8 * 1024);
    for i in 0..400 {
        store
            .save(&format!("src {i}"), &big, "m", false, always())
            .unwrap();
    }
    let before = size_on_disk(&dir);
    let outcome = quick_purge_wire(Some(&store), None, QuickPurgeScopeWire::All).expect("purge");
    let after = size_on_disk(&dir);
    assert_eq!(outcome.deleted, 400);
    assert_eq!(outcome.compact_error, None);
    assert_eq!(store.count().unwrap(), 0);
    assert!(after < before, "db+wal did not shrink: {before} -> {after}");
    assert_eq!(outcome.bytes_after, after);
    assert!(outcome.bytes_before >= before);
    drop(store);
    cleanup(&dir);
}

#[test]
fn a_run_blocked_mid_stream_writes_no_row_after_a_purge_cancels_it() {
    let dir = temp_dir("purge-inflight");
    let store = Arc::new(open_store(&dir));
    let runs = QuickTranslateGeneration::default();
    store.save("existing", "x", "m", false, always()).unwrap();

    let ticket = runs
        .begin(QuickSurface::GlossaryQueue, 1)
        .expect("free slot");
    let generation = ticket.generation;
    let watched = runs.clone();
    let should_cancel: Arc<dyn Fn() -> bool + Send + Sync> =
        Arc::new(move || !watched.is_current(generation));

    let (entered_tx, entered_rx) = std::sync::mpsc::channel();
    let (release_tx, release_rx) = std::sync::mpsc::channel();
    let (mut provider, _) = Fake::ok(vec!["moi"]);
    provider.gate = Some(Gate {
        entered: entered_tx,
        release: Mutex::new(release_rx),
    });

    let worker_store = Arc::clone(&store);
    let worker = std::thread::spawn(move || {
        let (outcome, _) = run(
            Some(&worker_store),
            &provider,
            "in flight",
            false,
            should_cancel,
        );
        outcome.is_ok()
    });
    entered_rx
        .recv_timeout(std::time::Duration::from_secs(20))
        .expect("the run reached the provider");

    let outcome =
        quick_purge_wire(Some(&store), Some(&runs), QuickPurgeScopeWire::All).expect("purge");
    assert_eq!(outcome.deleted, 1);
    release_tx.send(()).unwrap();
    assert!(
        worker.join().expect("worker"),
        "the run ended with an error"
    );

    assert_eq!(
        store.count().unwrap(),
        0,
        "a cancelled run wrote after the purge"
    );
    assert!(
        runs.begin(QuickSurface::ImportPreview, 2).is_some(),
        "the slot was not freed"
    );
    drop(ticket);
    drop(store);
    cleanup(&dir);
}

#[test]
fn the_purge_wire_reports_a_missing_store_without_cancelling_or_panicking() {
    let runs = QuickTranslateGeneration::default();
    let ticket = runs.begin(QuickSurface::GlossaryQueue, 1).expect("slot");
    assert!(quick_purge_wire(None, Some(&runs), QuickPurgeScopeWire::All).is_err());
    assert!(
        runs.is_current(ticket.generation),
        "a failed purge cancelled the run"
    );
}

#[test]
fn a_shrink_blocked_by_a_reader_keeps_the_deletion_and_reports_the_error() {
    let dir = temp_dir("purge-blocked");
    let store = open_store(&dir);
    for text in ["a", "b"] {
        store.save(text, "x", "m", false, always()).unwrap();
    }
    let reader = rusqlite::Connection::open(dir.join("quick-translation.db")).expect("open raw");
    reader.execute_batch("BEGIN").expect("begin");
    let held: i64 = reader
        .query_row("SELECT COUNT(*) FROM quick_translation", [], |row| {
            row.get(0)
        })
        .expect("snapshot");
    assert_eq!(held, 2);

    let outcome = quick_purge_wire(Some(&store), None, QuickPurgeScopeWire::All).expect("purge");
    assert_eq!(outcome.deleted, 2);
    assert!(
        outcome.compact_error.is_some(),
        "an open reader snapshot must leave the file unshrunk and say so"
    );
    assert_eq!(
        store.count().unwrap(),
        0,
        "a failed shrink rolled the deletion back"
    );

    reader.execute_batch("COMMIT").expect("commit");
    drop(reader);
    drop(store);
    cleanup(&dir);
}
