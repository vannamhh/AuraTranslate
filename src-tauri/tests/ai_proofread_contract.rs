//! Single-segment spelling/grammar scan: prepare layer, run seam with a fake provider, and the
//! read-only guarantee over the whole Chapter.

use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex, Once};

use auratranslate_lib::commands::aiconfig::{ai_config_delete_key, ai_config_save_key};
use auratranslate_lib::commands::aitranslate::PreparedTranslateCall;
use auratranslate_lib::commands::project::{OpenWork, create_work_from_text};
use auratranslate_lib::commands::proofread::{
    AiProofreadOutcomeWire, PreparedProofreadCall, ProofreadPrepareOutcome, ProofreadRunError,
    prepare_proofread_call, reply_malformed_error, run_proofread_call,
};
use auratranslate_lib::commands::segment::{
    ChapterSegment, confirm_segment, promote_ai_translation, read_open_chapter_segments,
};
use auratranslate_lib::core::ai::proofread::FindingKind;
use auratranslate_lib::core::aiconfig::{AiConfigField, AiConfigTier, write_field};
use auratranslate_lib::core::i18n::{IpcError, MessageKey};
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

struct FakeProvider {
    tokens: Vec<&'static str>,
    fail: bool,
    usage: Option<TranslateUsage>,
}

impl TranslationProvider for FakeProvider {
    type Error = FakeError;

    async fn translate(
        &self,
        _request: TranslateRequest<'_>,
        on_token: &mut dyn FnMut(&str),
        should_cancel: &dyn Fn() -> bool,
    ) -> Result<TranslateOutcome, FakeError> {
        for token in &self.tokens {
            if should_cancel() {
                return Ok(TranslateOutcome::Cancelled);
            }
            on_token(token);
        }
        if self.fail {
            return Err(FakeError);
        }
        Ok(TranslateOutcome::Done(self.usage))
    }
}

fn prepared(scanned: &str) -> PreparedProofreadCall {
    PreparedProofreadCall {
        call: PreparedTranslateCall {
            endpoint: "https://api.example.invalid/v1".to_owned(),
            model: "gpt-test".to_owned(),
            temperature: None,
            max_tokens: None,
            api_key: "sk-fake".to_owned(),
            prompt: "p".to_owned(),
        },
        scanned_text: scanned.to_owned(),
    }
}

fn collecting_channel() -> (tauri::ipc::Channel<String>, Arc<Mutex<Vec<String>>>) {
    let received = Arc::new(Mutex::new(Vec::new()));
    let sink = Arc::clone(&received);
    let channel = tauri::ipc::Channel::new(move |body| {
        if let tauri::ipc::InvokeResponseBody::Json(json) = body {
            let text: String = serde_json::from_str(&json).unwrap_or_default();
            sink.lock().unwrap_or_else(std::sync::PoisonError::into_inner).push(text);
        }
        Ok(())
    });
    (channel, received)
}

type RunResult = Result<AiProofreadOutcomeWire, ProofreadRunError<FakeError>>;

fn run(
    provider: &FakeProvider,
    scanned: &str,
    should_cancel: &dyn Fn() -> bool,
) -> (RunResult, Arc<Mutex<Vec<String>>>) {
    let (channel, received) = collecting_channel();
    let out = tauri::async_runtime::block_on(run_proofread_call(
        provider,
        &prepared(scanned),
        &channel,
        should_cancel,
    ));
    (out, received)
}

const TWO_FINDINGS: [&str; 2] = [
    r#"[{"kind":"spelling","quote":"nguời","explanation":"sai chinh ta","suggestion":"người"},"#,
    r#"{"kind":"grammar","quote":"đã đi","explanation":"thi","suggestion":"đi"}]"#,
];

#[test]
fn a_clean_reply_with_two_findings_ends_done_with_located_findings_and_no_unlocated() {
    let usage = TranslateUsage { prompt_tokens: 3, completion_tokens: 4, total_tokens: 7, cost_usd: None };
    let provider = FakeProvider { tokens: TWO_FINDINGS.to_vec(), fail: false, usage: Some(usage) };
    let text = "Mọi nguời đã đi";
    let (out, received) = run(&provider, text, &|| false);

    let Ok(AiProofreadOutcomeWire::Done { usage, scanned_text, findings, unlocated }) = out else {
        panic!("expected Done");
    };
    assert_eq!(scanned_text, text);
    assert_eq!(unlocated, 0);
    assert_eq!(usage.map(|u| u.total_tokens), Some(7));
    assert_eq!(findings.len(), 2);
    assert_eq!(findings[0].kind, FindingKind::Spelling);
    assert_eq!((findings[0].start, findings[0].end), (4, 9));
    assert_eq!(findings[1].kind, FindingKind::Grammar);
    assert_eq!(findings[0].suggestion, "người");
    assert_eq!(received.lock().unwrap().len(), 2, "every token is forwarded");
}

#[test]
fn a_quote_absent_from_the_text_is_counted_unlocated_not_dropped_silently() {
    let provider = FakeProvider {
        tokens: vec![r#"[{"kind":"spelling","quote":"khong co","explanation":"","suggestion":""}]"#],
        fail: false,
        usage: None,
    };
    let (out, _) = run(&provider, "van ban khac", &|| false);
    let Ok(AiProofreadOutcomeWire::Done { findings, unlocated, .. }) = out else { panic!("Done") };
    assert_eq!((findings.len(), unlocated), (0, 1));
}

#[test]
fn a_reply_off_the_schema_is_malformed_and_the_error_is_retryable() {
    let provider = FakeProvider { tokens: vec!["not json"], fail: false, usage: None };
    let (out, _) = run(&provider, "x", &|| false);
    assert!(matches!(out, Err(ProofreadRunError::ReplyMalformed)));
    let err = reply_malformed_error();
    assert_eq!(err.code(), "ai_proofread.reply_malformed");
    assert_eq!(err.message_key(), MessageKey::AiProofreadReplyMalformed);
    assert!(err.retryable());
}

#[test]
fn cancel_mid_stream_reports_cancelled_draws_nothing_and_sends_no_further_token() {
    let provider = FakeProvider { tokens: vec!["[", "]", "x"], fail: false, usage: None };
    let calls = AtomicUsize::new(0);
    let (out, received) = run(&provider, "x", &|| calls.fetch_add(1, Ordering::SeqCst) >= 1);
    assert!(matches!(out, Ok(AiProofreadOutcomeWire::Cancelled)));
    assert_eq!(received.lock().unwrap().clone(), vec!["[".to_owned()]);
}

#[test]
fn a_provider_failure_is_surfaced_as_a_provider_error() {
    let provider = FakeProvider { tokens: vec![], fail: true, usage: None };
    let (out, _) = run(&provider, "x", &|| false);
    assert!(matches!(out, Err(ProofreadRunError::Provider(_))));
}

#[test]
fn no_open_work_is_work_none_open_and_a_foreign_segment_is_rejected() {
    let global_dir = temp_dir("g1");
    let global = Store::open(StoreSpec::global(global_dir.join("global.db"))).expect("global");
    configure(&global);
    let err = expect_err(prepare_proofread_call(Some(&global), None, 1));
    assert_eq!(err.code(), "work.none_open");

    let work_dir = temp_dir("w1");
    let open = open_work(&work_dir, "One.");
    let err = expect_err(prepare_proofread_call(Some(&global), Some(&open), i64::MAX));
    assert_eq!(err.code(), "ai_prompt.segment_not_in_chapter");
}

#[test]
fn unconfigured_ai_is_not_configured_before_any_call() {
    let _g = KEYCHAIN_TEST_LOCK.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
    set_key(None);
    let global_dir = temp_dir("g2");
    let work_dir = temp_dir("w2");
    let global = Store::open(StoreSpec::global(global_dir.join("global.db"))).expect("global");
    let open = open_work(&work_dir, "One.");
    let id = segments(&open)[0].id;
    let out = prepare_proofread_call(Some(&global), Some(&open), id).expect("not an error");
    assert!(matches!(out, ProofreadPrepareOutcome::NotConfigured));

    configure(&global);
    promote_ai_translation(Some(&open), id, "Một câu.", false).expect("promote");
    let out = prepare_proofread_call(Some(&global), Some(&open), id).expect("not an error");
    assert!(matches!(out, ProofreadPrepareOutcome::NotConfigured), "no key saved");
}

#[test]
fn a_configured_scan_sends_a_prompt_carrying_the_target_text_and_scans_that_exact_text() {
    let _g = KEYCHAIN_TEST_LOCK.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
    set_key(Some("sk-proofread"));
    let global_dir = temp_dir("g3");
    let work_dir = temp_dir("w3");
    let global = Store::open(StoreSpec::global(global_dir.join("global.db"))).expect("global");
    configure(&global);
    let open = open_work(&work_dir, "One.");
    let id = segments(&open)[0].id;

    let out = prepare_proofread_call(Some(&global), Some(&open), id).expect("ok");
    assert!(matches!(out, ProofreadPrepareOutcome::EmptyText), "nothing typed yet");

    promote_ai_translation(Some(&open), id, "Mọi nguời đã đi", false).expect("promote");
    let ProofreadPrepareOutcome::Ready(p) = prepare_proofread_call(Some(&global), Some(&open), id).expect("ok")
    else {
        panic!("Ready");
    };
    assert_eq!(p.scanned_text, "Mọi nguời đã đi");
    assert!(p.call.prompt.contains("Mọi nguời đã đi"));
    assert_eq!(p.call.api_key, "sk-proofread");
}

#[test]
fn a_scan_with_findings_changes_no_target_text_origin_or_confirmation_in_the_chapter() {
    let _g = KEYCHAIN_TEST_LOCK.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
    set_key(Some("sk-proofread"));
    let global_dir = temp_dir("g4");
    let work_dir = temp_dir("w4");
    let global = Store::open(StoreSpec::global(global_dir.join("global.db"))).expect("global");
    configure(&global);
    let open = open_work(&work_dir, "One.\n\nTwo.\n\nThree.");
    let rows = segments(&open);
    assert!(rows.len() >= 2, "fixture needs several segments");
    for (i, row) in rows.iter().enumerate() {
        promote_ai_translation(Some(&open), row.id, &format!("Mọi nguời đã đi {i}"), false).expect("promote");
    }
    confirm_segment(Some(&open), rows[0].id).expect("confirm one");

    let snapshot = |open: &OpenWork| -> Vec<(i64, String, String, String)> {
        segments(open)
            .into_iter()
            .map(|s| (s.id, s.target_text, s.translation_origin, s.status))
            .collect()
    };
    let before = snapshot(&open);
    assert!(before.iter().any(|r| r.3 != before[rows.len() - 1].3), "mixed confirmation states");

    let ProofreadPrepareOutcome::Ready(p) =
        prepare_proofread_call(Some(&global), Some(&open), rows[1].id).expect("ok")
    else {
        panic!("Ready");
    };
    let provider = FakeProvider { tokens: TWO_FINDINGS.to_vec(), fail: false, usage: None };
    let (channel, _) = collecting_channel();
    let out = tauri::async_runtime::block_on(run_proofread_call(&provider, &p, &channel, &|| false));
    let Ok(AiProofreadOutcomeWire::Done { findings, .. }) = out else { panic!("Done") };
    assert!(!findings.is_empty(), "the scan must have findings for this guard to mean anything");

    assert_eq!(snapshot(&open), before);
}
