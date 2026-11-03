//! `#[tauri::command]` shells of the AI translation runs, driven through a `MockRuntime`
//! app with the states `lib.rs::open_work_slot` manages and a loopback SSE server.
//!
//! Guards the two `mark_prompt_as_sent` call sites that no pure-function case reaches:
//! the single-run `Done` branch and the batch `last_sent` block.

use std::fs;
use std::io::{Read, Write};
use std::net::TcpListener;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex, Once};

use auratranslate_lib::commands::aiconfig::{ai_config_delete_key, ai_config_save_key};
use auratranslate_lib::commands::aiprompt::{
    AiCallRecordsState, LastAssembledPromptState, assemble_and_record_prompt,
    read_last_assembled_prompt,
};
use auratranslate_lib::commands::aitranslate::{
    AiTranslateBatchEventWire, AiTranslateGeneration, AiTranslateOutcomeWire, AiTranslateScope,
    wire,
};
use auratranslate_lib::commands::project::{OpenWork, OpenWorkState, create_work_from_text};
use auratranslate_lib::commands::promptset::prompt_set_create;
use auratranslate_lib::commands::segment::read_open_chapter_segments;
use auratranslate_lib::core::aiconfig::{AiConfigField, AiConfigTier, write_field};
use auratranslate_lib::core::promptset::PromptSetTier;
use auratranslate_lib::core::store::{Store, StoreSpec};
use tauri::Manager as _;
use tauri::test::{MockRuntime, mock_builder, mock_context, noop_assets};

static NEXT_DIR: AtomicU64 = AtomicU64::new(0);

fn temp_dir(tag: &str) -> PathBuf {
    let n = NEXT_DIR.fetch_add(1, Ordering::Relaxed);
    let dir = std::env::temp_dir().join(format!(
        "auratranslate-ai-wire-{}-{}-{}",
        std::process::id(),
        tag,
        n
    ));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap_or_else(|e| panic!("tao {}: {e}", dir.display()));
    dir
}

static KEYCHAIN_KEY_TEST_LOCK: Mutex<()> = Mutex::new(());

fn install_mock_keychain_store_once() {
    static INIT: Once = Once::new();
    INIT.call_once(|| {
        let real_store_status = keyring::Entry::store_status();
        assert!(
            real_store_status.is_ok(),
            "kho keychain THAT khoi tao that bai ({real_store_status:?})"
        );
        keyring_core::set_default_store(
            keyring_core::mock::Store::new().expect("keyring_core::mock::Store::new that bai"),
        );
    });
}

fn save_key(value: &str) {
    install_mock_keychain_store_once();
    ai_config_delete_key(AiConfigTier::Global).expect("dat lai trang thai khoa that bai");
    ai_config_save_key(AiConfigTier::Global, value).expect("luu khoa API that bai");
}

const USAGE_CHUNK: &str = "data: {\"choices\":[],\"usage\":{\"prompt_tokens\":11,\"completion_tokens\":7,\"total_tokens\":18}}\n\n";

fn spawn_counted_sse_server(
    connections: usize,
    with_usage: bool,
    fixed_reply: Option<(String, String)>,
) -> (String, Arc<AtomicUsize>, Arc<Mutex<Vec<String>>>) {
    let bodies = Arc::new(Mutex::new(Vec::<String>::new()));
    let body_sink = Arc::clone(&bodies);
    let accepted = Arc::new(AtomicUsize::new(0));
    let counter = Arc::clone(&accepted);
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind cong tam");
    let port = listener.local_addr().expect("local_addr").port();
    std::thread::spawn(move || {
        for _ in 0..connections {
            let Ok((mut stream, _)) = listener.accept() else {
                return;
            };
            counter.fetch_add(1, Ordering::SeqCst);
            let mut received = Vec::new();
            let mut chunk = [0u8; 4096];
            let (header_end, content_length) = loop {
                let n = stream.read(&mut chunk).unwrap_or(0);
                if n == 0 {
                    break (received.len(), 0);
                }
                received.extend_from_slice(&chunk[..n]);
                if let Some(pos) = received.windows(4).position(|w| w == b"\r\n\r\n") {
                    let head = String::from_utf8_lossy(&received[..pos]).to_lowercase();
                    let len = head
                        .lines()
                        .find_map(|l| l.strip_prefix("content-length:"))
                        .and_then(|v| v.trim().parse::<usize>().ok())
                        .unwrap_or(0);
                    break (pos + 4, len);
                }
            };
            while received.len() < header_end + content_length {
                let n = stream.read(&mut chunk).unwrap_or(0);
                if n == 0 {
                    break;
                }
                received.extend_from_slice(&chunk[..n]);
            }
            body_sink
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .push(String::from_utf8_lossy(&received[header_end..]).into_owned());
            if let Some((status_line, error_body)) = &fixed_reply {
                let response = format!(
                    "HTTP/1.1 {status_line}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{error_body}",
                    error_body.len()
                );
                let _ = stream.write_all(response.as_bytes());
                let _ = stream.flush();
                continue;
            }
            let body = format!(
                "data: {{\"choices\":[{{\"delta\":{{\"content\":\"<<<SEGMENT 1\\nXin chao\\nSEGMENT>>>\"}}}}]}}\n\n{}data: [DONE]\n\n",
                if with_usage { USAGE_CHUNK } else { "" }
            );
            let response = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: text/event-stream\r\nConnection: close\r\n\r\n{body}"
            );
            let _ = stream.write_all(response.as_bytes());
            let _ = stream.flush();
        }
    });
    (
        format!("http://127.0.0.1:{port}/v1/chat/completions"),
        accepted,
        bodies,
    )
}

struct Harness {
    app: tauri::App<MockRuntime>,
    dirs: Vec<PathBuf>,
    accepted: Arc<AtomicUsize>,
    bodies: Arc<Mutex<Vec<String>>>,
}

impl Harness {
    fn new(tag: &str, text: &str, connections: usize) -> Self {
        Self::build(tag, text, connections, true, false, None)
    }

    fn with_usage(tag: &str, text: &str, connections: usize) -> Self {
        Self::build(tag, text, connections, true, true, None)
    }

    fn with_error_reply(
        tag: &str,
        text: &str,
        connections: usize,
        status_line: &str,
        body: &str,
    ) -> Self {
        Self::build(
            tag,
            text,
            connections,
            true,
            false,
            Some((status_line.to_owned(), body.to_owned())),
        )
    }

    fn build(
        tag: &str,
        text: &str,
        connections: usize,
        manage_generation: bool,
        with_usage: bool,
        fixed_reply: Option<(String, String)>,
    ) -> Self {
        let global_dir = temp_dir(&format!("{tag}-global"));
        let work_dir = temp_dir(&format!("{tag}-work"));
        let global =
            Store::open(StoreSpec::global(global_dir.join("global.db"))).expect("mo global.db");
        let (endpoint, accepted, bodies) =
            spawn_counted_sse_server(connections, with_usage, fixed_reply);
        write_field(&global, AiConfigField::Endpoint, &endpoint).expect("ghi endpoint");
        write_field(&global, AiConfigField::Model, "gpt-wire").expect("ghi model");

        let open =
            create_work_from_text(&work_dir, tag, "en", "", text.to_owned()).expect("tao Tac pham");
        prompt_set_create(
            Some(&global),
            Some(&open),
            PromptSetTier::Global,
            "Plain",
            "{{source_segment}}",
        )
        .expect("tao bo prompt");

        let app = mock_builder()
            .build(mock_context(noop_assets()))
            .expect("dung app MockRuntime");
        app.manage(global);
        app.manage(OpenWorkState::new(Some(open)));
        app.manage(LastAssembledPromptState::new(None));
        app.manage(AiCallRecordsState::default());
        if manage_generation {
            app.manage(AiTranslateGeneration::default());
        }
        Self {
            app,
            dirs: vec![global_dir, work_dir],
            accepted,
            bodies,
        }
    }

    fn segment_ids(&self) -> Vec<i64> {
        let state = self.app.state::<OpenWorkState>();
        let guard = state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        read_open_chapter_segments(guard.as_ref())
            .expect("nap chuong")
            .segments
            .iter()
            .map(|s| s.id)
            .collect()
    }

    fn with_open<T>(&self, f: impl FnOnce(&OpenWork, &Store) -> T) -> T {
        let state = self.app.state::<OpenWorkState>();
        let guard = state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let global = self.app.state::<Store>();
        f(guard.as_ref().expect("Tac pham dang mo"), &global)
    }

    fn assemble(&self, segment_id: i64) {
        self.with_open(|open, global| {
            let record = self.app.state::<LastAssembledPromptState>();
            assemble_and_record_prompt(
                Some(global),
                Some(open),
                record.inner(),
                Some("Plain"),
                segment_id,
            )
            .expect("lap rap prompt");
        });
    }

    fn record(&self) -> auratranslate_lib::commands::aiprompt::AssembledPromptWire {
        let record = self.app.state::<LastAssembledPromptState>();
        read_last_assembled_prompt(record.inner()).expect("phien phai co ban ghi")
    }
}

impl Drop for Harness {
    fn drop(&mut self) {
        for dir in &self.dirs {
            let _ = fs::remove_dir_all(Path::new(dir));
        }
    }
}

#[test]
fn a_single_run_that_reaches_done_stamps_the_record_as_sent() {
    let _guard = KEYCHAIN_KEY_TEST_LOCK
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    save_key("sk-wire-single");
    let h = Harness::new("single", "A dragon roared.", 1);
    let segment_id = h.segment_ids()[0];
    h.assemble(segment_id);
    assert!(h.record().sent_at.is_none(), "lap rap thuan chua gui gi");

    let tokens = Arc::new(Mutex::new(Vec::<String>::new()));
    let sink = Arc::clone(&tokens);
    let channel = tauri::ipc::Channel::<String>::new(move |body| {
        if let tauri::ipc::InvokeResponseBody::Json(json) = body {
            sink.lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .push(json);
        }
        Ok(())
    });

    let outcome = tauri::async_runtime::block_on(wire::ai_translate_segment(
        h.app.handle().clone(),
        segment_id,
        Some("Plain".to_owned()),
        channel,
    ))
    .expect("ai_translate_segment phai tra Ok");
    assert!(
        matches!(outcome, AiTranslateOutcomeWire::Done { .. }),
        "ket qua: {outcome:?}"
    );
    assert!(
        tokens
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .iter()
            .any(|t| t.contains("Xin chao")),
        "server SSE gia phai da chay that: token phai toi kenh"
    );

    let record = h.record();
    assert_eq!(record.segment_id, segment_id);
    assert!(
        record.sent_at.is_some(),
        "nhanh Done cua vo phai danh dau ban ghi da gui"
    );
    assert_eq!(record.sent_model.as_deref(), Some("gpt-wire"));
}

fn bare_app() -> tauri::App<MockRuntime> {
    mock_builder()
        .build(mock_context(noop_assets()))
        .expect("dung app MockRuntime")
}

fn assert_state_missing(err: &auratranslate_lib::core::i18n::IpcError, code: &str) {
    assert_eq!(err.code(), code);
    assert_eq!(
        err.message_key(),
        auratranslate_lib::core::i18n::MessageKey::Unknown
    );
    assert!(
        err.params().is_empty(),
        "params phai rong: {:?}",
        err.params()
    );
    assert!(!err.retryable());
}

#[test]
fn single_run_without_prompt_record_state_fails_with_record_state_missing() {
    let app = bare_app();
    let channel = tauri::ipc::Channel::<String>::new(|_| Ok(()));
    let err = tauri::async_runtime::block_on(wire::ai_translate_segment(
        app.handle().clone(),
        1,
        None,
        channel,
    ))
    .expect_err("thieu state ghi prompt phai la Err");
    assert_state_missing(&err, "ai_translate.record_state_missing");
}

#[test]
fn batch_without_prompt_record_state_fails_with_record_state_missing() {
    let app = bare_app();
    let channel = tauri::ipc::Channel::<AiTranslateBatchEventWire>::new(|_| Ok(()));
    let err = tauri::async_runtime::block_on(wire::ai_translate_batch(
        app.handle().clone(),
        AiTranslateScope::Selection {
            segment_ids: vec![1],
        },
        None,
        channel,
    ))
    .expect_err("thieu state ghi prompt phai la Err");
    assert_state_missing(&err, "ai_translate.record_state_missing");
}

#[test]
fn single_run_without_generation_state_fails_with_generation_state_missing() {
    let _guard = KEYCHAIN_KEY_TEST_LOCK
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    save_key("sk-wire-nogen-single");
    let h = Harness::build("nogen-single", "A dragon roared.", 0, false, false, None);
    let segment_id = h.segment_ids()[0];
    let channel = tauri::ipc::Channel::<String>::new(|_| Ok(()));
    let err = tauri::async_runtime::block_on(wire::ai_translate_segment(
        h.app.handle().clone(),
        segment_id,
        Some("Plain".to_owned()),
        channel,
    ))
    .expect_err("thieu state the he phai la Err");
    assert_state_missing(&err, "ai_translate.generation_state_missing");
}

#[test]
fn batch_without_generation_state_fails_with_generation_state_missing() {
    let _guard = KEYCHAIN_KEY_TEST_LOCK
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    save_key("sk-wire-nogen-batch");
    let h = Harness::build("nogen-batch", "A dragon roared.", 0, false, false, None);
    let ids = h.segment_ids();
    let channel = tauri::ipc::Channel::<AiTranslateBatchEventWire>::new(|_| Ok(()));
    let err = tauri::async_runtime::block_on(wire::ai_translate_batch(
        h.app.handle().clone(),
        AiTranslateScope::Selection { segment_ids: ids },
        Some("Plain".to_owned()),
        channel,
    ))
    .expect_err("thieu state the he phai la Err");
    assert_state_missing(&err, "ai_translate.generation_state_missing");
}

fn cancel_while_prepare_is_pinned<T: Send>(h: &Harness, run: impl FnOnce() -> T + Send) -> T {
    let state = h.app.state::<OpenWorkState>();
    let pin = state
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    std::thread::scope(|scope| {
        let call = scope.spawn(run);
        std::thread::sleep(std::time::Duration::from_millis(500));
        wire::ai_translate_cancel(h.app.handle().clone());
        drop(pin);
        call.join().expect("luong goi lenh khong duoc panic")
    })
}

#[test]
fn a_cancel_arriving_during_single_prepare_is_not_overwritten() {
    let _guard = KEYCHAIN_KEY_TEST_LOCK
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    save_key("sk-wire-cancel-single");
    let h = Harness::new("cancel-single", "A dragon roared.", 1);
    let segment_id = h.segment_ids()[0];
    let handle = h.app.handle().clone();
    let outcome = cancel_while_prepare_is_pinned(&h, move || {
        let channel = tauri::ipc::Channel::<String>::new(|_| Ok(()));
        tauri::async_runtime::block_on(wire::ai_translate_segment(
            handle,
            segment_id,
            Some("Plain".to_owned()),
            channel,
        ))
    })
    .expect("ai_translate_segment phai tra Ok");
    assert!(
        matches!(outcome, AiTranslateOutcomeWire::Cancelled),
        "ket qua: {outcome:?}"
    );
    assert_eq!(
        h.accepted.load(Ordering::SeqCst),
        0,
        "huy giua prepare khong duoc gui yeu cau nao"
    );
}

#[test]
fn a_cancel_arriving_during_batch_prepare_is_not_overwritten() {
    let _guard = KEYCHAIN_KEY_TEST_LOCK
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    save_key("sk-wire-cancel-batch");
    let h = Harness::new("cancel-batch", "A dragon roared. The knight fled.", 2);
    let ids = h.segment_ids();
    let handle = h.app.handle().clone();
    let outcome = cancel_while_prepare_is_pinned(&h, move || {
        let channel = tauri::ipc::Channel::<AiTranslateBatchEventWire>::new(|_| Ok(()));
        tauri::async_runtime::block_on(wire::ai_translate_batch(
            handle,
            AiTranslateScope::Selection { segment_ids: ids },
            Some("Plain".to_owned()),
            channel,
        ))
    })
    .expect("ai_translate_batch phai tra Ok");
    assert!(
        matches!(outcome, AiTranslateOutcomeWire::Cancelled),
        "ket qua: {outcome:?}"
    );
    assert_eq!(
        h.accepted.load(Ordering::SeqCst),
        0,
        "huy giua prepare khong duoc gui yeu cau nao"
    );
}

fn clear_key() {
    install_mock_keychain_store_once();
    ai_config_delete_key(AiConfigTier::Global).expect("dat lai trang thai khoa that bai");
}

#[test]
fn single_run_without_a_saved_key_returns_not_configured() {
    let _guard = KEYCHAIN_KEY_TEST_LOCK
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    clear_key();
    let h = Harness::new("nokey-single", "A dragon roared.", 0);
    let segment_id = h.segment_ids()[0];
    let channel = tauri::ipc::Channel::<String>::new(|_| Ok(()));
    let outcome = tauri::async_runtime::block_on(wire::ai_translate_segment(
        h.app.handle().clone(),
        segment_id,
        Some("Plain".to_owned()),
        channel,
    ))
    .expect("ai_translate_segment phai tra Ok");
    assert!(
        matches!(outcome, AiTranslateOutcomeWire::NotConfigured),
        "ket qua: {outcome:?}"
    );
}

#[test]
fn batch_without_a_saved_key_returns_not_configured() {
    let _guard = KEYCHAIN_KEY_TEST_LOCK
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    clear_key();
    let h = Harness::new("nokey-batch", "A dragon roared. The knight fled.", 0);
    let ids = h.segment_ids();
    let channel = tauri::ipc::Channel::<AiTranslateBatchEventWire>::new(|_| Ok(()));
    let outcome = tauri::async_runtime::block_on(wire::ai_translate_batch(
        h.app.handle().clone(),
        AiTranslateScope::Selection { segment_ids: ids },
        Some("Plain".to_owned()),
        channel,
    ))
    .expect("ai_translate_batch phai tra Ok");
    assert!(
        matches!(outcome, AiTranslateOutcomeWire::NotConfigured),
        "ket qua: {outcome:?}"
    );
}

fn sink_channel() -> tauri::ipc::Channel<String> {
    tauri::ipc::Channel::<String>::new(|_| Ok(()))
}

fn target_texts(h: &Harness) -> Vec<String> {
    h.with_open(|open, _| {
        read_open_chapter_segments(Some(open))
            .expect("nap chuong")
            .segments
            .iter()
            .map(|s| s.target_text.clone())
            .collect()
    })
}

fn assert_usage_18(usage: Option<auratranslate_lib::commands::aitranslate::AiTranslateUsageWire>) {
    let usage = usage.expect("sent_usage phai mang so provider tra");
    assert_eq!(
        (
            usage.prompt_tokens,
            usage.completion_tokens,
            usage.total_tokens
        ),
        (11, 7, 18)
    );
}

#[test]
fn a_single_run_that_ends_with_provider_usage_stamps_it_on_the_record() {
    let _guard = KEYCHAIN_KEY_TEST_LOCK
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    save_key("sk-wire-usage");
    let h = Harness::with_usage("single-usage", "A dragon roared.", 1);
    let segment_id = h.segment_ids()[0];
    let outcome = tauri::async_runtime::block_on(wire::ai_translate_segment(
        h.app.handle().clone(),
        segment_id,
        Some("Plain".to_owned()),
        sink_channel(),
    ))
    .expect("ai_translate_segment phai tra Ok");
    assert!(
        matches!(outcome, AiTranslateOutcomeWire::Done { usage: Some(_), .. }),
        "ket qua: {outcome:?}"
    );
    assert_usage_18(h.record().sent_usage);
}

#[test]
fn resend_sends_the_recorded_prompt_stamps_usage_and_leaves_target_text_alone() {
    let _guard = KEYCHAIN_KEY_TEST_LOCK
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    save_key("sk-wire-resend");
    let h = Harness::with_usage("resend", "A dragon roared.", 1);
    let segment_id = h.segment_ids()[0];
    h.assemble(segment_id);
    let before = target_texts(&h);
    let recorded = h.record();
    assert!(recorded.sent_at.is_none() && recorded.sent_usage.is_none());

    let outcome = tauri::async_runtime::block_on(wire::ai_translate_resend(
        h.app.handle().clone(),
        sink_channel(),
    ))
    .expect("ai_translate_resend phai tra Ok");
    assert!(
        matches!(outcome, AiTranslateOutcomeWire::Done { usage: Some(_), .. }),
        "ket qua: {outcome:?}"
    );
    assert_eq!(h.accepted.load(Ordering::SeqCst), 1);
    let bodies = h
        .bodies
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .clone();
    assert_eq!(bodies.len(), 1);
    assert!(
        bodies[0].contains(&serde_json::to_string(&recorded.prompt).expect("json")),
        "than gui di phai mang dung prompt cua ban ghi: {}",
        bodies[0]
    );

    let after = h.record();
    assert!(after.sent_at.is_some());
    assert_eq!(after.sent_model.as_deref(), Some("gpt-wire"));
    assert_usage_18(after.sent_usage);
    assert_eq!(
        target_texts(&h),
        before,
        "gui lai khong duoc ghi target_text"
    );
}

#[test]
fn resend_without_a_record_returns_no_prompt_to_resend() {
    let _guard = KEYCHAIN_KEY_TEST_LOCK
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    save_key("sk-wire-resend-none");
    let h = Harness::new("resend-none", "A dragon roared.", 0);
    let err = tauri::async_runtime::block_on(wire::ai_translate_resend(
        h.app.handle().clone(),
        sink_channel(),
    ))
    .expect_err("khong co ban ghi thi phai loi");
    assert_eq!(err.code(), "ai_translate.no_prompt_to_resend");
}

const WIRE_LEAKY_KEY: &str = "sk-wire-leaky-0123456789abcdef";

fn single_run_error(h: &Harness) -> auratranslate_lib::core::i18n::IpcError {
    let segment_id = h.segment_ids()[0];
    tauri::async_runtime::block_on(wire::ai_translate_segment(
        h.app.handle().clone(),
        segment_id,
        Some("Plain".to_owned()),
        sink_channel(),
    ))
    .expect_err("loi nha cung cap phai tra Err")
}

#[test]
fn a_402_whose_message_echoes_the_key_reaches_the_ipc_error_with_the_key_redacted() {
    let _guard = KEYCHAIN_KEY_TEST_LOCK
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    save_key(WIRE_LEAKY_KEY);
    let body = format!(
        r#"{{"error":{{"code":"payment_required","message":"Key {WIRE_LEAKY_KEY} has no balance."}}}}"#
    );
    let h = Harness::with_error_reply(
        "err-402",
        "A dragon roared.",
        1,
        "402 Payment Required",
        &body,
    );
    let err = single_run_error(&h);
    assert_eq!(err.code(), "ai_translate.provider_payment_required");
    assert_eq!(err.params().get("status"), Some(&"402".to_owned()));
    assert_eq!(
        err.params().get("provider_message"),
        Some(&"Key *** has no balance.".to_owned())
    );
    assert!(!err.retryable());
    assert!(!format!("{err:?}").contains(WIRE_LEAKY_KEY));
    assert!(!format!("{err:?}").contains("sk-wire-leak"));
    assert_eq!(h.accepted.load(Ordering::SeqCst), 1, "khong tu thu lai");
}

#[test]
fn a_message_that_leaks_a_long_key_fragment_the_redaction_cannot_cover_is_dropped() {
    let _guard = KEYCHAIN_KEY_TEST_LOCK
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    save_key(WIRE_LEAKY_KEY);
    let body = r#"{"error":{"message":"Bad key ending ...ky-0123456789ab"}}"#;
    let h = Harness::with_error_reply("err-frag", "A dragon roared.", 1, "401 Unauthorized", body);
    let err = single_run_error(&h);
    assert_eq!(err.code(), "ai_translate.provider_key_rejected");
    assert!(err.params().get("provider_message").is_none());
    assert!(!format!("{err:?}").contains("0123456789ab"));
}

#[test]
fn a_502_with_an_html_body_gives_the_server_error_key_and_no_provider_message() {
    let _guard = KEYCHAIN_KEY_TEST_LOCK
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    save_key("sk-wire-502");
    let h = Harness::with_error_reply(
        "err-502",
        "A dragon roared.",
        1,
        "502 Bad Gateway",
        "<html><body>Bad gateway</body></html>",
    );
    let err = single_run_error(&h);
    assert_eq!(err.code(), "ai_translate.provider_server_error");
    assert_eq!(err.params().get("status"), Some(&"502".to_owned()));
    assert!(err.params().get("provider_message").is_none());
    assert!(err.retryable());
}

#[test]
fn a_gemini_style_array_body_yields_its_message_through_the_real_path() {
    let _guard = KEYCHAIN_KEY_TEST_LOCK
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    save_key("sk-wire-gemini");
    let body = r#"[{"error":{"code":402,"message":"Prepayment depleted.","status":"RESOURCE_EXHAUSTED"}}]"#;
    let h = Harness::with_error_reply(
        "err-arr",
        "A dragon roared.",
        1,
        "402 Payment Required",
        body,
    );
    let err = single_run_error(&h);
    assert_eq!(err.code(), "ai_translate.provider_payment_required");
    assert_eq!(
        err.params().get("provider_message"),
        Some(&"Prepayment depleted.".to_owned())
    );
}

#[test]
fn an_error_body_over_the_cap_is_ignored_and_the_status_key_still_applies() {
    let _guard = KEYCHAIN_KEY_TEST_LOCK
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    save_key("sk-wire-huge");
    let body = format!(r#"{{"error":{{"message":"{}"}}}}"#, "x".repeat(40 * 1024));
    let h = Harness::with_error_reply(
        "err-huge",
        "A dragon roared.",
        1,
        "429 Too Many Requests",
        &body,
    );
    let err = single_run_error(&h);
    assert_eq!(err.code(), "ai_translate.provider_rate_limited");
    assert!(err.params().get("provider_message").is_none());
}

#[test]
fn a_batch_stopped_by_a_429_names_the_sentence_the_status_and_the_provider_message() {
    let _guard = KEYCHAIN_KEY_TEST_LOCK
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    save_key("sk-wire-batch-429");
    let h = Harness::with_error_reply(
        "err-batch",
        "A dragon roared. The knight fled.",
        1,
        "429 Too Many Requests",
        r#"{"error":{"message":"Slow down"}}"#,
    );
    let ids = h.segment_ids();
    let (first, last) = (ids[0], ids[1]);
    let err = tauri::async_runtime::block_on(wire::ai_translate_batch(
        h.app.handle().clone(),
        AiTranslateScope::Selection { segment_ids: ids },
        Some("Plain".to_owned()),
        tauri::ipc::Channel::<AiTranslateBatchEventWire>::new(|_| Ok(())),
    ))
    .expect_err("lo phai dung voi Err");
    assert_eq!(err.code(), "ai_translate.provider_rate_limited");
    assert_eq!(err.params().get("call_index"), Some(&"1".to_owned()));
    assert_eq!(
        err.params().get("first_segment_id"),
        Some(&first.to_string())
    );
    assert_eq!(err.params().get("last_segment_id"), Some(&last.to_string()));
    assert_eq!(err.params().get("status"), Some(&"429".to_owned()));
    assert_eq!(
        err.params().get("provider_message"),
        Some(&"Slow down".to_owned())
    );
}

fn sse_body(frames: &[&str]) -> String {
    let mut out = String::new();
    for f in frames {
        out.push_str("data: ");
        out.push_str(f);
        out.push_str("\n\n");
    }
    out.push_str("data: [DONE]\n\n");
    out
}

const GEMINI_FRAMES: [&str; 4] = [
    r#"{"choices":[{"delta":{"content":"<<<SEGMENT 1\nCon rong ","role":"assistant"},"index":0}],"model":"gemini-3.5-flash-lite","object":"chat.completion.chunk","usage":{"completion_tokens":2,"prompt_tokens":11,"total_tokens":13}}"#,
    r#"{"choices":[{"delta":{"content":"gam ","role":"assistant"},"index":0}],"model":"gemini-3.5-flash-lite","object":"chat.completion.chunk","usage":{"completion_tokens":3,"prompt_tokens":11,"total_tokens":14}}"#,
    r#"{"choices":[{"delta":{"content":"len.\nSEGMENT>>>\n","role":"assistant"},"index":0}],"model":"gemini-3.5-flash-lite","object":"chat.completion.chunk","usage":{"completion_tokens":7,"prompt_tokens":11,"total_tokens":18}}"#,
    r#"{"choices":[{"delta":{"extra_content":{"google":{"thought_signature":"abc"}},"role":"assistant"},"finish_reason":"stop","index":0}],"model":"gemini-3.5-flash-lite","object":"chat.completion.chunk","usage":{"completion_tokens":7,"prompt_tokens":11,"total_tokens":18}}"#,
];

const GEMINI_EMPTY_FRAMES: [&str; 2] = [
    r#"{"choices":[{"delta":{"role":"assistant","extra_content":{"google":{"thought_signature":"abc"}}}}],"usage":{"prompt_tokens":11,"completion_tokens":0,"total_tokens":11}}"#,
    r#"{"choices":[{"delta":{},"finish_reason":"content_filter"}],"usage":{"prompt_tokens":11,"completion_tokens":0,"total_tokens":11}}"#,
];

#[test]
fn a_gemini_stream_with_usage_on_every_frame_delivers_all_text_and_the_final_usage() {
    let _guard = KEYCHAIN_KEY_TEST_LOCK
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    save_key("sk-wire-gemini");
    let h = Harness::with_error_reply(
        "gemini-ok",
        "A dragon roared.",
        1,
        "200 OK",
        &sse_body(&GEMINI_FRAMES),
    );
    let segment_id = h.segment_ids()[0];
    let tokens = Arc::new(Mutex::new(Vec::<String>::new()));
    let sink = Arc::clone(&tokens);
    let channel = tauri::ipc::Channel::<String>::new(move |body| {
        if let tauri::ipc::InvokeResponseBody::Json(json) = body {
            sink.lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .push(json);
        }
        Ok(())
    });
    let outcome = tauri::async_runtime::block_on(wire::ai_translate_segment(
        h.app.handle().clone(),
        segment_id,
        Some("Plain".to_owned()),
        channel,
    ))
    .expect("luong Gemini co chu phai tra Ok");
    assert!(
        matches!(outcome, AiTranslateOutcomeWire::Done { usage: Some(_), .. }),
        "ket qua: {outcome:?}"
    );
    let seen = tokens
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .clone();
    assert_eq!(
        seen,
        vec![
            "\"Con rong \"".to_owned(),
            "\"gam \"".to_owned(),
            "\"len.\"".to_owned()
        ],
        "tung doan chu qua kenh dung thu tu"
    );
    assert_usage_18(h.record().sent_usage);
}

#[test]
fn a_stream_that_ends_with_no_text_fails_with_empty_reply_and_the_finish_reason() {
    let _guard = KEYCHAIN_KEY_TEST_LOCK
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    save_key("sk-wire-empty");
    let h = Harness::with_error_reply(
        "gemini-empty",
        "A dragon roared.",
        1,
        "200 OK",
        &sse_body(&GEMINI_EMPTY_FRAMES),
    );
    let err = single_run_error(&h);
    assert_eq!(err.code(), "ai_translate.reply_empty");
    assert_eq!(
        err.message_key(),
        auratranslate_lib::core::i18n::MessageKey::AiTranslateReplyEmptyWithReason
    );
    assert_eq!(
        err.params().get("finish_reason"),
        Some(&"content_filter".to_owned())
    );
    assert!(!err.retryable());
    assert!(target_texts(&h).iter().all(String::is_empty));
}

#[test]
fn a_stream_with_only_done_fails_with_empty_reply_without_a_reason() {
    let _guard = KEYCHAIN_KEY_TEST_LOCK
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    save_key("sk-wire-empty-bare");
    let h = Harness::with_error_reply(
        "empty-bare",
        "A dragon roared.",
        1,
        "200 OK",
        &sse_body(&[]),
    );
    let err = single_run_error(&h);
    assert_eq!(
        err.message_key(),
        auratranslate_lib::core::i18n::MessageKey::AiTranslateReplyEmpty
    );
    assert!(err.params().get("finish_reason").is_none());
}

#[test]
fn a_stream_of_only_whitespace_text_fails_with_empty_reply() {
    let _guard = KEYCHAIN_KEY_TEST_LOCK
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    save_key("sk-wire-empty-blank");
    let h = Harness::with_error_reply(
        "empty-blank",
        "A dragon roared.",
        1,
        "200 OK",
        &sse_body(&[
            r#"{"choices":[{"delta":{"content":"\n","role":"assistant"}}],"usage":{"prompt_tokens":11,"completion_tokens":1,"total_tokens":12}}"#,
            r#"{"choices":[{"delta":{"content":"  \n"},"finish_reason":"stop"}],"usage":{"prompt_tokens":11,"completion_tokens":2,"total_tokens":13}}"#,
        ]),
    );
    let err = single_run_error(&h);
    assert_eq!(err.code(), "ai_translate.reply_empty");
    assert_eq!(err.params().get("finish_reason"), Some(&"stop".to_owned()));
    assert!(target_texts(&h).iter().all(String::is_empty));
}

#[test]
fn a_batch_call_whose_stream_has_no_text_discards_every_sentence_and_the_operation_goes_on() {
    let _guard = KEYCHAIN_KEY_TEST_LOCK
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    save_key("sk-wire-empty-batch");
    let h = Harness::with_error_reply(
        "empty-batch",
        "A dragon roared. The knight fled.",
        1,
        "200 OK",
        &sse_body(&GEMINI_EMPTY_FRAMES),
    );
    let ids = h.segment_ids();
    let events = Arc::new(Mutex::new(Vec::<String>::new()));
    let sink = Arc::clone(&events);
    let channel = tauri::ipc::Channel::<AiTranslateBatchEventWire>::new(move |body| {
        if let tauri::ipc::InvokeResponseBody::Json(json) = body {
            sink.lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .push(json);
        }
        Ok(())
    });
    let outcome = tauri::async_runtime::block_on(wire::ai_translate_batch(
        h.app.handle().clone(),
        AiTranslateScope::Selection { segment_ids: ids },
        Some("Plain".to_owned()),
        channel,
    ))
    .expect("a reply without text is not an operation error");
    assert!(
        matches!(outcome, AiTranslateOutcomeWire::Done { .. }),
        "{outcome:?}"
    );
    let seen = events
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .clone();
    assert_eq!(
        seen.iter()
            .filter(|e| e.contains("\"discarded\"") && e.contains("\"no_finish\""))
            .count(),
        2,
        "{seen:?}"
    );
    assert!(seen.iter().all(|e| !e.contains("\"received\"")), "{seen:?}");
    assert_eq!(h.accepted.load(Ordering::SeqCst), 1);
    assert!(target_texts(&h).iter().all(String::is_empty));
}

const RESEND_COPIED_CONTEXT_FRAME: &str = r#"{"choices":[{"delta":{"content":"<<<CONTEXT\nA dragon roared.\nCONTEXT>>>\n<<<SEGMENT 1\nCon rong\nSEGMENT>>>"},"finish_reason":"stop"}]}"#;

#[test]
fn resending_a_single_sentence_sends_the_recorded_neighbours_and_reads_the_reply_by_blocks() {
    let _guard = KEYCHAIN_KEY_TEST_LOCK
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    save_key("sk-wire-resend-blocks");
    let h = Harness::with_error_reply(
        "resend-blocks",
        "A dragon roared. The knight fled. The bell rang. The gate fell.",
        1,
        "200 OK",
        &sse_body(&[RESEND_COPIED_CONTEXT_FRAME]),
    );
    let segment_id = h.segment_ids()[3];
    h.assemble(segment_id);
    let recorded = h.record();
    assert_eq!(recorded.prompt.matches("<<<CONTEXT").count(), 3);

    let pieces = Arc::new(Mutex::new(Vec::<String>::new()));
    let sink = Arc::clone(&pieces);
    let channel = tauri::ipc::Channel::<String>::new(move |body| {
        if let tauri::ipc::InvokeResponseBody::Json(json) = body {
            sink.lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .push(serde_json::from_str(&json).expect("manh chu"));
        }
        Ok(())
    });
    let outcome =
        tauri::async_runtime::block_on(wire::ai_translate_resend(h.app.handle().clone(), channel))
            .expect("ai_translate_resend phai tra Ok");
    assert!(
        matches!(
            outcome,
            AiTranslateOutcomeWire::Done {
                maybe_truncated: false,
                ..
            }
        ),
        "{outcome:?}"
    );
    let bodies = h
        .bodies
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .clone();
    assert!(
        bodies[0].contains(&serde_json::to_string(&recorded.prompt).expect("json")),
        "than gui di phai mang dung prompt co cau ke: {}",
        bodies[0]
    );
    assert_eq!(
        pieces
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .concat(),
        "Con rong",
        "khoi ngu canh mo hinh chep lai khong duoc toi kenh"
    );
}

#[test]
fn resending_a_single_sentence_whose_reply_has_no_block_one_fails_with_result_missing() {
    let _guard = KEYCHAIN_KEY_TEST_LOCK
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    save_key("sk-wire-resend-missing");
    let h = Harness::with_error_reply(
        "resend-missing",
        "A dragon roared.",
        1,
        "200 OK",
        &sse_body(&[
            r#"{"choices":[{"delta":{"content":"Con rong gam."},"finish_reason":"stop"}]}"#,
        ]),
    );
    h.assemble(h.segment_ids()[0]);
    let err = tauri::async_runtime::block_on(wire::ai_translate_resend(
        h.app.handle().clone(),
        sink_channel(),
    ))
    .expect_err("chu thuong ngoai khoi phai la Err");
    assert_eq!(err.code(), "ai_translate.result_missing");
    assert!(target_texts(&h).iter().all(String::is_empty));
}

mod provider_message_pure {
    use auratranslate_lib::core::ai::client::{extract_provider_message, redact_api_key};

    const KEY: &str = "sk-abcdefghijklmnop";

    #[test]
    fn redaction_replaces_every_full_key_occurrence() {
        assert_eq!(
            redact_api_key(&format!("a {KEY} b {KEY}"), KEY).as_deref(),
            Some("a *** b ***")
        );
    }

    #[test]
    fn redaction_drops_the_message_when_a_twelve_char_key_run_survives() {
        assert_eq!(redact_api_key("tail abcdefghijkl", KEY), None);
        assert_eq!(
            redact_api_key("tail abcdefghijk", KEY).as_deref(),
            Some("tail abcdefghijk")
        );
    }

    #[test]
    fn redaction_ignores_an_empty_key() {
        assert_eq!(redact_api_key("hello", "").as_deref(), Some("hello"));
        assert_eq!(redact_api_key("hello", "   ").as_deref(), Some("hello"));
    }

    #[test]
    fn redaction_treats_a_short_key_as_one_whole_fragment() {
        assert_eq!(redact_api_key("pw=abc!", "abc").as_deref(), Some("pw=***!"));
    }

    #[test]
    fn extraction_reads_the_three_accepted_shapes() {
        assert_eq!(
            extract_provider_message(br#"{"error":{"message":"one"}}"#).as_deref(),
            Some("one")
        );
        assert_eq!(
            extract_provider_message(br#"[{"error":{"message":"two"}}]"#).as_deref(),
            Some("two")
        );
        assert_eq!(
            extract_provider_message(br#"{"error":"three"}"#).as_deref(),
            Some("three")
        );
    }

    #[test]
    fn extraction_rejects_other_shapes_and_empty_messages() {
        for body in [
            &b"<html>x</html>"[..],
            b"{}",
            br#"{"error":{"message":42}}"#,
            br#"{"error":{"message":"   "}}"#,
            br#"[{"error":{"message":"a"}},{"error":{"message":"b"}}]"#,
            br#"{"error":null}"#,
        ] {
            assert_eq!(extract_provider_message(body), None, "{body:?}");
        }
    }

    #[test]
    fn extraction_strips_control_characters_collapses_whitespace_and_cuts_at_300() {
        let got = extract_provider_message(br#"{"error":{"message":"a\u0007b\n\n  c\td"}}"#);
        assert_eq!(got.as_deref(), Some("ab c d"));
        let long = format!(r#"{{"error":{{"message":"{}"}}}}"#, "\u{e9}".repeat(500));
        assert_eq!(
            extract_provider_message(long.as_bytes()).map(|m| m.chars().count()),
            Some(300)
        );
    }
}
