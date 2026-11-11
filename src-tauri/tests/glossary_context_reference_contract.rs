//! Glossary pending-queue context reference: the Work's own translation of a context sentence
//! when it exists, and a one-shot AI translation on demand otherwise.

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

use auratranslate_lib::commands::aitranslate::{
    PrepareOutcome, QuickTranslateOutcomeWire, prepare_context_reference_call, quick_translate,
};
use auratranslate_lib::commands::glossary::glossary_context_translation;
use auratranslate_lib::commands::project::{OpenWork, create_work_from_text};
use auratranslate_lib::core::ai::rag::assemble_context_reference_prompt;
use auratranslate_lib::core::aiconfig::{AiConfigField, write_field};
use auratranslate_lib::core::quick_translation::QuickTranslationStore;
use auratranslate_lib::core::store::{Store, StoreSpec};
use auratranslate_lib::ports::translation_provider::{
    TranslateOutcome, TranslateRequest, TranslationProvider,
};

static NEXT_DIR: AtomicU64 = AtomicU64::new(0);

fn temp_dir(tag: &str) -> PathBuf {
    let n = NEXT_DIR.fetch_add(1, Ordering::Relaxed);
    let dir = std::env::temp_dir().join(format!(
        "auratranslate-ctx-ref-{}-{}-{}",
        std::process::id(),
        tag,
        n
    ));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap_or_else(|e| panic!("tao {}: {e}", dir.display()));
    dir
}

fn open_global(dir: &Path) -> Store {
    Store::open(StoreSpec::global(dir.join("global.db"))).expect("mo global.db")
}

fn open_work(root: &Path, text: &str) -> OpenWork {
    create_work_from_text(root, "CtxRef", "zh", "", text.to_owned()).expect("tao tac pham")
}

fn set_segment(open: &OpenWork, source_prefix: &str, sql_set: &str) {
    let pattern = format!("{source_prefix}%");
    let sql = format!("UPDATE segment SET {sql_set} WHERE source_text LIKE ?1");
    open.store
        .write(move |tx| {
            tx.execute(&sql, [&pattern])?;
            Ok(())
        })
        .expect("sua segment");
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
        .expect("chup")
}

#[test]
fn a_translated_live_segment_is_found_by_its_sentence() {
    let root = temp_dir("found");
    let open = open_work(&root, "一二三。四五六。");
    set_segment(&open, "一二三", "target_text = 'Mot hai ba.'");
    assert_eq!(
        glossary_context_translation(Some(&open), "一二三。").expect("tra"),
        Some("Mot hai ba.".to_owned())
    );
    drop(open);
    let _ = fs::remove_dir_all(root);
}

#[test]
fn a_blank_omitted_retired_or_unmatched_segment_is_not_a_translation() {
    let root = temp_dir("none");
    let open = open_work(&root, "一二三。四五六。七八九。");
    set_segment(&open, "一二三", "target_text = '   '");
    set_segment(
        &open,
        "四五六",
        "target_text = 'Bon nam sau.', is_omitted = 1",
    );
    set_segment(
        &open,
        "七八九",
        "target_text = 'Bay tam chin.', retired_at = '2026-01-01T00:00:00.000Z'",
    );
    for sentence in ["一二三。", "四五六。", "七八九。", "không có"] {
        assert_eq!(
            glossary_context_translation(Some(&open), sentence).expect("tra"),
            None,
            "{sentence}"
        );
    }
    assert_eq!(
        glossary_context_translation(None, "一二三。").expect("tra"),
        None
    );
    assert_eq!(
        glossary_context_translation(Some(&open), "").expect("tra"),
        None
    );
    drop(open);
    let _ = fs::remove_dir_all(root);
}

#[test]
fn a_truncated_context_matches_by_its_200_character_prefix() {
    let root = temp_dir("trunc");
    let long: String = "字".repeat(250);
    let open = open_work(&root, &format!("{long}。"));
    set_segment(&open, "字", "target_text = 'Dai.'");
    let truncated: String = long.chars().take(200).collect();
    assert_eq!(
        glossary_context_translation(Some(&open), &truncated).expect("tra"),
        Some("Dai.".to_owned())
    );
    let almost: String = long.chars().take(199).collect();
    assert_eq!(
        glossary_context_translation(Some(&open), &almost).expect("tra"),
        None
    );
    drop(open);
    let _ = fs::remove_dir_all(root);
}

#[test]
fn an_unconfigured_provider_is_a_state_reached_before_any_network_call() {
    let root = temp_dir("unconf");
    let gdir = temp_dir("unconf-g");
    let global = open_global(&gdir);
    let open = open_work(&root, "一二三。");
    let outcome =
        prepare_context_reference_call(Some(&global), Some(&open), "一二三。").expect("prepare");
    assert!(matches!(outcome, PrepareOutcome::NotConfigured));
    drop((global, open));
    let _ = fs::remove_dir_all(root);
    let _ = fs::remove_dir_all(gdir);
}

fn configured_local() -> (PathBuf, PathBuf, Store, OpenWork) {
    let root = temp_dir("conf");
    let gdir = temp_dir("conf-g");
    let global = open_global(&gdir);
    write_field(&global, AiConfigField::Provider, "ollama").expect("provider");
    write_field(
        &global,
        AiConfigField::Endpoint,
        "http://localhost:11434/v1",
    )
    .expect("endpoint");
    write_field(&global, AiConfigField::Model, "m").expect("model");
    let open = open_work(&root, "一二三。");
    (root, gdir, global, open)
}

#[test]
fn the_prepared_call_carries_the_fixed_prompt_and_touches_no_stored_row() {
    let (root, gdir, global, open) = configured_local();
    let before = snapshot(&open);
    let prepared = match prepare_context_reference_call(Some(&global), Some(&open), "一二三。")
        .expect("prepare")
    {
        PrepareOutcome::Ready(p) => p,
        PrepareOutcome::NotConfigured => panic!("phai san sang"),
    };
    assert_eq!(
        prepared.prompt,
        assemble_context_reference_prompt("一二三。")
    );
    assert!(prepared.system.is_none());

    let qdir = temp_dir("prompt-q");
    let store = QuickTranslationStore::open(qdir.join("quick-translation.db")).expect("quick");
    let provider = Fake {
        tokens: vec!["Mot ", "hai ba."],
    };
    let received = Arc::new(Mutex::new(Vec::<String>::new()));
    let sink = Arc::clone(&received);
    let mut on_token = move |text: &str| sink.lock().unwrap().push(text.to_owned());
    let outcome = tauri::async_runtime::block_on(quick_translate(
        Some(&store),
        &provider,
        move || Ok(PrepareOutcome::Ready(prepared)),
        "一二三。",
        false,
        &mut on_token,
        Arc::new(|| false),
    ))
    .expect("chay");
    assert!(matches!(outcome, QuickTranslateOutcomeWire::Done { .. }));
    assert_eq!(*received.lock().unwrap(), ["Mot ", "hai ba."]);
    assert_eq!(snapshot(&open), before, "lan goi khong duoc ghi hang nao");
    drop((global, open, store));
    let _ = fs::remove_dir_all(root);
    let _ = fs::remove_dir_all(gdir);
    let _ = fs::remove_dir_all(qdir);
}

struct Fake {
    tokens: Vec<&'static str>,
}

#[derive(Debug, Clone)]
struct FakeError;
impl std::fmt::Display for FakeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("fake")
    }
}
impl std::error::Error for FakeError {}

impl TranslationProvider for Fake {
    type Error = FakeError;

    async fn translate(
        &self,
        _request: TranslateRequest<'_>,
        on_token: &mut dyn FnMut(&str),
        should_cancel: &dyn Fn() -> bool,
    ) -> Result<TranslateOutcome, Self::Error> {
        for token in &self.tokens {
            if should_cancel() {
                return Ok(TranslateOutcome::Cancelled);
            }
            on_token(token);
        }
        Ok(TranslateOutcome::Done {
            usage: None,
            finish_reason: Some("stop".to_owned()),
        })
    }
}

#[test]
fn the_wire_reaches_the_send_on_its_own_generation_and_leaves_the_batch_and_rows_alone() {
    use auratranslate_lib::commands::aitranslate::{
        AiTranslateGeneration, QuickTranslateGeneration, wire,
    };
    use auratranslate_lib::commands::project::OpenWorkState;
    use tauri::Manager as _;
    use tauri::test::{mock_builder, mock_context, noop_assets};

    let (root, gdir, global, open) = configured_local();
    write_field(&global, AiConfigField::Endpoint, "http://127.0.0.1:1/v1").expect("endpoint");
    let before = snapshot(&open);
    let qdir = temp_dir("wire-q");
    let quick_store =
        QuickTranslationStore::open(qdir.join("quick-translation.db")).expect("quick");

    let app = mock_builder()
        .build(mock_context(noop_assets()))
        .expect("app");
    app.manage(global);
    app.manage(quick_store);
    app.manage::<OpenWorkState>(Mutex::new(Some(open)));
    app.manage(AiTranslateGeneration::default());
    app.manage(QuickTranslateGeneration::default());
    let handle = app.handle().clone();
    let batch = app.state::<AiTranslateGeneration>();
    let reference = app.state::<QuickTranslateGeneration>();

    let channel = tauri::ipc::Channel::<String>::new(|_| Ok(()));
    let result = tauri::async_runtime::block_on(wire::ai_context_reference_translate(
        handle.clone(),
        "一二三。".to_owned(),
        auratranslate_lib::commands::aitranslate::QuickTranslateTrigger::Manual,
        5,
        false,
        channel,
    ));
    let error = result.expect_err("a closed port is a connection error, which proves the send ran");
    assert_ne!(error.code(), "store.open_failed");
    assert_ne!(error.code(), "work.none_open");
    assert!(batch.is_current(0), "the call bumped the batch generation");
    assert!(reference.is_current(1));

    wire::ai_context_reference_cancel(handle, 5);
    assert!(batch.is_current(0), "cancel bumped the batch generation");
    assert!(
        reference.is_current(1),
        "a cancel of a run that already ended moved the generation"
    );

    let guard = app.state::<OpenWorkState>();
    let open_ref = guard.lock().unwrap();
    assert_eq!(
        snapshot(open_ref.as_ref().expect("work")),
        before,
        "the call wrote a row"
    );
    drop(open_ref);
    let _ = fs::remove_dir_all(root);
    let _ = fs::remove_dir_all(gdir);
    let _ = fs::remove_dir_all(qdir);
}

#[test]
fn quick_routing_uses_dedicated_or_whole_main_global_despite_work_overrides() {
    use auratranslate_lib::commands::aiconfig::{
        QuickAiConfigFieldValue, quick_ai_config_save_fields,
    };
    use auratranslate_lib::core::aiconfig::QuickAiConfigField;
    let (root, gdir, global, open) = configured_local();
    write_field(
        &open.store,
        AiConfigField::Endpoint,
        "http://work.invalid/v1",
    )
    .unwrap();
    write_field(&open.store, AiConfigField::Model, "work-model").unwrap();
    write_field(&global, AiConfigField::Temperature, "0.25").unwrap();
    write_field(&global, AiConfigField::MaxTokens, "321").unwrap();
    for dedicated in [true, false] {
        let fields = [
            (QuickAiConfigField::Provider, "lmstudio"),
            (
                QuickAiConfigField::Endpoint,
                if dedicated {
                    "http://quick.invalid/v1"
                } else {
                    ""
                },
            ),
            (
                QuickAiConfigField::Model,
                if dedicated { "quick-model" } else { "" },
            ),
        ]
        .map(|(field, value)| QuickAiConfigFieldValue {
            field,
            value: value.into(),
        });
        quick_ai_config_save_fields(
            Some(&global),
            auratranslate_lib::core::aiconfig::AiConfigTier::Global,
            &fields,
        )
        .unwrap();
        let resolved =
            auratranslate_lib::commands::aitranslate::resolve_quick_call_config(Some(&global))
                .unwrap()
                .unwrap();
        assert_eq!(
            resolved.provider,
            if dedicated { "lmstudio" } else { "ollama" }
        );
        for work in [None, Some(&open)] {
            let PrepareOutcome::Ready(call) =
                prepare_context_reference_call(Some(&global), work, "句").unwrap()
            else {
                panic!("ready")
            };
            assert_eq!(
                call.endpoint,
                if dedicated {
                    "http://quick.invalid/v1"
                } else {
                    "http://localhost:11434/v1"
                }
            );
            assert_eq!(call.model, if dedicated { "quick-model" } else { "m" });
            assert_eq!(call.temperature, if dedicated { None } else { Some(0.25) });
            assert_eq!(call.max_tokens, if dedicated { None } else { Some(321) });
        }
    }
    drop((global, open));
    let _ = fs::remove_dir_all(root);
    let _ = fs::remove_dir_all(gdir);
}

#[test]
fn quick_wire_and_connection_test_work_without_any_work_state_and_cancel_independently() {
    use auratranslate_lib::commands::aitranslate::{
        AiTranslateGeneration, ConnectionCheckGeneration, QuickConnectionCheckGeneration,
        QuickTranslateGeneration, wire,
    };
    use tauri::Manager as _;
    use tauri::test::{mock_builder, mock_context, noop_assets};
    let (root, gdir, global, open) = configured_local();
    write_field(&global, AiConfigField::Endpoint, "http://127.0.0.1:1/v1").unwrap();
    let qdir = temp_dir("no-work-wire");
    let app = mock_builder().build(mock_context(noop_assets())).unwrap();
    app.manage(global);
    app.manage(QuickTranslationStore::open(qdir.join("quick-translation.db")).unwrap());
    app.manage(AiTranslateGeneration::default());
    app.manage(QuickTranslateGeneration::default());
    app.manage(ConnectionCheckGeneration::default());
    app.manage(QuickConnectionCheckGeneration::default());
    let handle = app.handle().clone();
    let result = tauri::async_runtime::block_on(wire::ai_context_reference_translate(
        handle.clone(),
        "句".into(),
        auratranslate_lib::commands::aitranslate::QuickTranslateTrigger::Manual,
        77,
        false,
        tauri::ipc::Channel::new(|_| Ok(())),
    ));
    assert!(
        result.is_err(),
        "closed port must be reached without a Work"
    );
    assert_ne!(result.err().unwrap().code(), "work.none_open");
    let result =
        tauri::async_runtime::block_on(wire::quick_ai_config_test_connection(handle.clone()));
    assert!(result.is_err(), "connection test must reach closed port");
    wire::quick_ai_config_test_connection_cancel(handle);
    assert!(app.state::<ConnectionCheckGeneration>().is_current(0));
    assert!(app.state::<AiTranslateGeneration>().is_current(0));
    assert!(app.state::<QuickTranslateGeneration>().is_current(1));
    assert!(
        !app.state::<QuickConnectionCheckGeneration>()
            .0
            .is_current(1)
    );
    drop(open);
    let _ = fs::remove_dir_all(root);
    let _ = fs::remove_dir_all(gdir);
    let _ = fs::remove_dir_all(qdir);
}
