//! Glossary pending-queue context reference: the Work's own translation of a context sentence
//! when it exists, and a one-shot AI translation on demand otherwise.

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

use auratranslate_lib::commands::aitranslate::{
    AiTranslateGeneration, ContextReferenceGeneration, PrepareOutcome, PreparedTranslateCall,
    SingleRun, prepare_context_reference_call, run_plain_call,
};
use auratranslate_lib::commands::glossary::glossary_context_translation;
use auratranslate_lib::commands::project::{OpenWork, create_work_from_text};
use auratranslate_lib::core::ai::rag::assemble_context_reference_prompt;
use auratranslate_lib::core::aiconfig::{AiConfigField, write_field};
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

    let provider = Fake {
        tokens: vec!["Mot ", "hai ba."],
    };
    let received = Arc::new(Mutex::new(Vec::<String>::new()));
    let sink = Arc::clone(&received);
    let channel = tauri::ipc::Channel::new(move |body| {
        if let tauri::ipc::InvokeResponseBody::Json(json) = body {
            sink.lock()
                .unwrap()
                .push(serde_json::from_str(&json).unwrap_or_default());
        }
        Ok(())
    });
    let outcome =
        tauri::async_runtime::block_on(run_plain_call(&provider, &prepared, &channel, &|| false))
            .expect("chay");
    assert!(matches!(outcome, SingleRun::Done { .. }));
    assert_eq!(*received.lock().unwrap(), ["Mot ", "hai ba."]);
    assert_eq!(snapshot(&open), before, "lan goi khong duoc ghi hang nao");
    drop((global, open));
    let _ = fs::remove_dir_all(root);
    let _ = fs::remove_dir_all(gdir);
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
    use auratranslate_lib::commands::aitranslate::wire;
    use auratranslate_lib::commands::project::OpenWorkState;
    use tauri::Manager as _;
    use tauri::test::{mock_builder, mock_context, noop_assets};

    let (root, gdir, global, open) = configured_local();
    write_field(&global, AiConfigField::Endpoint, "http://127.0.0.1:1/v1").expect("endpoint");
    let before = snapshot(&open);

    let app = mock_builder()
        .build(mock_context(noop_assets()))
        .expect("app");
    app.manage(global);
    app.manage::<OpenWorkState>(Mutex::new(Some(open)));
    app.manage(AiTranslateGeneration::default());
    app.manage(ContextReferenceGeneration::default());
    let handle = app.handle().clone();
    let batch = app.state::<AiTranslateGeneration>();
    let reference = app.state::<ContextReferenceGeneration>();

    let channel = tauri::ipc::Channel::<String>::new(|_| Ok(()));
    let result = tauri::async_runtime::block_on(wire::ai_context_reference_translate(
        handle.clone(),
        "一二三。".to_owned(),
        channel,
    ));
    let error = result.expect_err("a closed port is a connection error, which proves the send ran");
    assert_ne!(error.code(), "store.open_failed");
    assert_ne!(error.code(), "work.none_open");
    assert!(batch.is_current(0), "the call bumped the batch generation");
    assert!(reference.is_current(1));

    wire::ai_context_reference_cancel(handle);
    assert!(batch.is_current(0), "cancel bumped the batch generation");
    assert!(reference.is_current(2));

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
}

#[test]
fn a_cancelled_reference_call_sends_no_token() {
    let reference = ContextReferenceGeneration::default();
    let g = reference.next();
    let provider = Fake {
        tokens: vec!["a", "b"],
    };
    let seen = Arc::new(Mutex::new(0usize));
    let sink = Arc::clone(&seen);
    let channel = tauri::ipc::Channel::new(move |_| {
        *sink.lock().unwrap() += 1;
        Ok(())
    });
    let prepared = PreparedTranslateCall {
        endpoint: "http://localhost/v1".to_owned(),
        model: "m".to_owned(),
        temperature: None,
        max_tokens: None,
        api_key: String::new(),
        prompt: "p".to_owned(),
        system: None,
    };
    reference.next();
    let outcome =
        tauri::async_runtime::block_on(run_plain_call(&provider, &prepared, &channel, &|| {
            !reference.is_current(g)
        }))
        .expect("chay");
    assert!(matches!(outcome, SingleRun::Cancelled));
    assert_eq!(*seen.lock().unwrap(), 0);
}
