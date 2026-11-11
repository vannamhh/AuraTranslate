//! `quick-translation.db` and the `Dịch thử` command around it (AD-64 rules 3 to 6 and 8).

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

use auratranslate_lib::commands::aitranslate::{
    AiTranslateGeneration, PrepareOutcome, PreparedTranslateCall, QuickLookupWire,
    QuickTranslateError, QuickTranslateGeneration, QuickTranslateOutcomeWire, quick_lookup_wire,
    quick_translate,
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

fn ready() -> Result<PrepareOutcome, auratranslate_lib::core::i18n::IpcError> {
    Ok(PrepareOutcome::Ready(PreparedTranslateCall {
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
        || Ok(PrepareOutcome::NotConfigured),
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
    let first = quick.begin(7);
    assert!(
        !quick.cancel(8),
        "a cancel for another run_id did something"
    );
    assert!(quick.is_current(first));
    let second = quick.begin(9);
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

    let generation = runs.begin(5);
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

    let generation = runs.begin(6);
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
        2,
        false,
        channel(),
    ));
    let error = miss.expect_err("a closed port is a connection error, which proves the send ran");
    assert_ne!(error.code(), "store.open_failed");

    assert!(quick.is_current(2));
    wire::ai_context_reference_cancel(handle.clone(), 1);
    assert!(
        quick.is_current(2),
        "a cancel for a stale run_id moved the generation"
    );
    assert!(
        batch.is_current(0),
        "the quick translation moved the batch generation"
    );
    wire::ai_context_reference_cancel(handle.clone(), 2);
    assert!(!quick.is_current(2));
    assert!(batch.is_current(0));

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
