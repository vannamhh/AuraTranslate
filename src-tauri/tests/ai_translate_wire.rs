//! `#[tauri::command]` shells of the AI translation runs, driven through a `MockRuntime`
//! app with the states `lib.rs::open_work_slot` manages and a loopback SSE server.
//!
//! Guards the two `mark_prompt_as_sent` call sites that no pure-function case reaches:
//! the single-run `Done` branch and the batch `last_sent` block.

use std::fs;
use std::io::{Read, Write};
use std::net::TcpListener;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex, Once};

use auratranslate_lib::commands::aiconfig::{ai_config_delete_key, ai_config_save_key};
use auratranslate_lib::commands::aiprompt::{
    LastAssembledPromptState, assemble_and_record_prompt, read_last_assembled_prompt,
};
use auratranslate_lib::commands::aitranslate::{
    AiTranslateBatchEventWire, AiTranslateGeneration, AiTranslateOutcomeWire, wire,
};
use auratranslate_lib::commands::project::{OpenWork, OpenWorkState, create_work_from_text};
use auratranslate_lib::commands::promptset::prompt_set_create;
use auratranslate_lib::commands::segment::{read_open_chapter_segments, set_segment_omitted};
use auratranslate_lib::core::aiconfig::{AiConfigField, AiConfigTier, write_field};
use auratranslate_lib::core::promptset::PromptSetTier;
use auratranslate_lib::core::store::{Store, StoreSpec};
use tauri::Manager as _;
use tauri::test::{MockRuntime, mock_builder, mock_context, noop_assets};

static NEXT_DIR: AtomicU64 = AtomicU64::new(0);

fn temp_dir(tag: &str) -> PathBuf {
    let n = NEXT_DIR.fetch_add(1, Ordering::Relaxed);
    let dir = std::env::temp_dir().join(format!("auratranslate-ai-wire-{}-{}-{}", std::process::id(), tag, n));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap_or_else(|e| panic!("tao {}: {e}", dir.display()));
    dir
}

static KEYCHAIN_KEY_TEST_LOCK: Mutex<()> = Mutex::new(());

fn install_mock_keychain_store_once() {
    static INIT: Once = Once::new();
    INIT.call_once(|| {
        let real_store_status = keyring::Entry::store_status();
        assert!(real_store_status.is_ok(), "kho keychain THAT khoi tao that bai ({real_store_status:?})");
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

/// Loopback server answering every POST with one streamed token then `[DONE]`; returns the
/// endpoint URL. The thread ends when the listener is dropped with the process.
fn spawn_sse_server(connections: usize) -> String {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind cong tam");
    let port = listener.local_addr().expect("local_addr").port();
    std::thread::spawn(move || {
        for _ in 0..connections {
            let Ok((mut stream, _)) = listener.accept() else { return };
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
            let body = "data: {\"choices\":[{\"delta\":{\"content\":\"Xin chao\"}}]}\n\ndata: [DONE]\n\n";
            let response = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: text/event-stream\r\nConnection: close\r\n\r\n{body}"
            );
            let _ = stream.write_all(response.as_bytes());
            let _ = stream.flush();
        }
    });
    format!("http://127.0.0.1:{port}/v1/chat/completions")
}

struct Harness {
    app: tauri::App<MockRuntime>,
    dirs: Vec<PathBuf>,
}

impl Harness {
    fn new(tag: &str, text: &str, connections: usize) -> Self {
        let global_dir = temp_dir(&format!("{tag}-global"));
        let work_dir = temp_dir(&format!("{tag}-work"));
        let global = Store::open(StoreSpec::global(global_dir.join("global.db"))).expect("mo global.db");
        let endpoint = spawn_sse_server(connections);
        write_field(&global, AiConfigField::Endpoint, &endpoint).expect("ghi endpoint");
        write_field(&global, AiConfigField::Model, "gpt-wire").expect("ghi model");

        let open = create_work_from_text(&work_dir, tag, "en", "", text.to_owned()).expect("tao Tac pham");
        prompt_set_create(Some(&global), Some(&open), PromptSetTier::Global, "Plain", "{{source_segment}}")
            .expect("tao bo prompt");

        let app = mock_builder().build(mock_context(noop_assets())).expect("dung app MockRuntime");
        app.manage(global);
        app.manage(OpenWorkState::new(Some(open)));
        app.manage(LastAssembledPromptState::new(None));
        app.manage(AiTranslateGeneration::default());
        Self { app, dirs: vec![global_dir, work_dir] }
    }

    fn segment_ids(&self) -> Vec<i64> {
        let state = self.app.state::<OpenWorkState>();
        let guard = state.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
        read_open_chapter_segments(guard.as_ref()).expect("nap chuong").segments.iter().map(|s| s.id).collect()
    }

    fn with_open<T>(&self, f: impl FnOnce(&OpenWork, &Store) -> T) -> T {
        let state = self.app.state::<OpenWorkState>();
        let guard = state.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
        let global = self.app.state::<Store>();
        f(guard.as_ref().expect("Tac pham dang mo"), &global)
    }

    fn assemble(&self, segment_id: i64) {
        self.with_open(|open, global| {
            let record = self.app.state::<LastAssembledPromptState>();
            assemble_and_record_prompt(Some(global), Some(open), record.inner(), Some("Plain"), segment_id)
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
    let _guard = KEYCHAIN_KEY_TEST_LOCK.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
    save_key("sk-wire-single");
    let h = Harness::new("single", "A dragon roared.", 1);
    let segment_id = h.segment_ids()[0];
    h.assemble(segment_id);
    assert!(h.record().sent_at.is_none(), "lap rap thuan chua gui gi");

    let tokens = Arc::new(Mutex::new(Vec::<String>::new()));
    let sink = Arc::clone(&tokens);
    let channel = tauri::ipc::Channel::<String>::new(move |body| {
        if let tauri::ipc::InvokeResponseBody::Json(json) = body {
            sink.lock().unwrap_or_else(std::sync::PoisonError::into_inner).push(json);
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
    assert!(matches!(outcome, AiTranslateOutcomeWire::Done { .. }), "ket qua: {outcome:?}");
    assert!(
        tokens.lock().unwrap_or_else(std::sync::PoisonError::into_inner).iter().any(|t| t.contains("Xin chao")),
        "server SSE gia phai da chay that: token phai toi kenh"
    );

    let record = h.record();
    assert_eq!(record.segment_id, segment_id);
    assert!(record.sent_at.is_some(), "nhanh Done cua vo phai danh dau ban ghi da gui");
    assert_eq!(record.sent_model.as_deref(), Some("gpt-wire"));
}

#[test]
fn a_batch_stamps_only_the_last_sentence_that_was_sent_and_never_an_omitted_tail() {
    let _guard = KEYCHAIN_KEY_TEST_LOCK.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
    save_key("sk-wire-batch");
    let h = Harness::new("batch", "A dragon roared. The knight fled. A bell rang.", 2);
    let ids = h.segment_ids();
    assert_eq!(ids.len(), 3, "fixture phai co dung ba cau: {ids:?}");
    h.with_open(|open, _| set_segment_omitted(Some(open), ids[2], true).expect("cat cau cuoi"));

    let events = Arc::new(Mutex::new(Vec::<String>::new()));
    let sink = Arc::clone(&events);
    let channel = tauri::ipc::Channel::<AiTranslateBatchEventWire>::new(move |body| {
        if let tauri::ipc::InvokeResponseBody::Json(json) = body {
            sink.lock().unwrap_or_else(std::sync::PoisonError::into_inner).push(json);
        }
        Ok(())
    });

    let outcome = tauri::async_runtime::block_on(wire::ai_translate_batch(
        h.app.handle().clone(),
        ids.clone(),
        Some("Plain".to_owned()),
        channel,
    ))
    .expect("ai_translate_batch phai tra Ok");
    assert!(matches!(outcome, AiTranslateOutcomeWire::Done { .. }), "ket qua: {outcome:?}");

    let seen = events.lock().unwrap_or_else(std::sync::PoisonError::into_inner).clone();
    assert_eq!(seen.iter().filter(|e| e.contains("\"done\"")).count(), 2, "hai cau dich: {seen:?}");
    assert!(seen.iter().any(|e| e.contains("\"skipped\"")), "cau cuoi bi cat phai phat skipped: {seen:?}");

    let record = h.record();
    assert_eq!(record.segment_id, ids[1], "ban ghi phien giu cau CUOI CUNG duoc dich, khong phai cau bi cat");
    assert!(record.sent_at.is_some(), "khoi last_sent cua lo phai danh dau ban ghi da gui");
    assert_eq!(record.sent_model.as_deref(), Some("gpt-wire"));
}
