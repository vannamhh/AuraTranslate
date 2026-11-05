//! Grouped batch translation (AD-60) driven through the real `#[tauri::command]` shells of a
//! `MockRuntime` app against a loopback SSE server that answers each connection from a script.
//! Every case reads the events a webview would get on the one `Channel`.

use std::fs;
use std::io::{Read, Write};
use std::net::TcpListener;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex, Once};

use auratranslate_lib::commands::aiconfig::{ai_config_delete_key, ai_config_save_key};
use auratranslate_lib::commands::aiprompt::{
    AiCallRecordsState, LastAssembledPromptState, read_prompt_record_for_segment,
};
use auratranslate_lib::commands::aitranslate::{
    AiTranslateBatchEventWire, AiTranslateGeneration, AiTranslateOutcomeWire, AiTranslateScope,
    wire,
};
use auratranslate_lib::commands::project::{OpenWork, OpenWorkState, create_work_from_text};
use auratranslate_lib::commands::promptset::prompt_set_create;
use auratranslate_lib::commands::segment::{
    SegmentTargetEdit, read_open_chapter_segments, save_segment_targets, set_segment_omitted,
};
use auratranslate_lib::core::ai::batch::{BlockStream, guard_marker_line, unguard_marker_line};
use auratranslate_lib::core::aiconfig::{
    AiConfigField, AiConfigTier, DEFAULT_SYSTEM_PROMPT, write_field,
};
use auratranslate_lib::core::glossary::{Category, confirm_translation, insert_manual_entry};
use auratranslate_lib::core::promptset::{PromptSetKind, PromptSetTier};
use auratranslate_lib::core::store::{Store, StoreSpec, Transaction};
use auratranslate_lib::core::tm::{PairOrigin, insert_pair, write_only_mine};
use serde_json::Value;
use tauri::Manager as _;
use tauri::test::{MockRuntime, mock_builder, mock_context, noop_assets};

static NEXT_DIR: AtomicU64 = AtomicU64::new(0);
static KEYCHAIN_LOCK: Mutex<()> = Mutex::new(());

fn temp_dir(tag: &str) -> PathBuf {
    let n = NEXT_DIR.fetch_add(1, Ordering::Relaxed);
    let dir = std::env::temp_dir().join(format!(
        "auratranslate-ai-batch-{}-{}-{}",
        std::process::id(),
        tag,
        n
    ));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap_or_else(|e| panic!("create {}: {e}", dir.display()));
    dir
}

fn save_key(value: &str) {
    static INIT: Once = Once::new();
    INIT.call_once(|| {
        assert!(keyring::Entry::store_status().is_ok());
        keyring_core::set_default_store(keyring_core::mock::Store::new().expect("mock store"));
    });
    ai_config_delete_key(AiConfigTier::Global).expect("reset key");
    ai_config_save_key(AiConfigTier::Global, value).expect("save key");
}

// ── Scripted loopback server ─────────────────────────────────────────────────────────

struct Reply {
    text: String,
    chunk_chars: usize,
    finish: Option<&'static str>,
    usage: bool,
    done: bool,
    delay_ms: u64,
    status: Option<(&'static str, &'static str)>,
}

impl Reply {
    fn stop(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            chunk_chars: 40,
            finish: Some("stop"),
            usage: false,
            done: true,
            delay_ms: 0,
            status: None,
        }
    }

    fn no_finish(text: impl Into<String>) -> Self {
        Self {
            finish: None,
            ..Self::stop(text)
        }
    }

    fn with_finish(text: impl Into<String>, finish: &'static str) -> Self {
        Self {
            finish: Some(finish),
            ..Self::stop(text)
        }
    }

    fn with_usage(mut self) -> Self {
        self.usage = true;
        self
    }

    fn status(line: &'static str, body: &'static str) -> Self {
        Self {
            status: Some((line, body)),
            ..Self::stop("")
        }
    }
}

type Responder = Arc<dyn Fn(usize, &str) -> Reply + Send + Sync>;
type Hook = Arc<Mutex<Option<Box<dyn Fn(usize) + Send>>>>;

fn json_string(text: &str) -> String {
    serde_json::to_string(text).expect("json string")
}

fn sse_body(reply: &Reply) -> String {
    let mut body = String::new();
    let chars: Vec<char> = reply.text.chars().collect();
    for chunk in chars.chunks(reply.chunk_chars.max(1)) {
        let piece: String = chunk.iter().collect();
        body.push_str(&format!(
            "data: {{\"choices\":[{{\"delta\":{{\"content\":{}}}}}]}}\n\n",
            json_string(&piece)
        ));
    }
    if let Some(finish) = reply.finish {
        body.push_str(&format!(
            "data: {{\"choices\":[{{\"delta\":{{}},\"finish_reason\":\"{finish}\"}}]}}\n\n"
        ));
    }
    if reply.usage {
        body.push_str("data: {\"choices\":[],\"usage\":{\"prompt_tokens\":11,\"completion_tokens\":7,\"total_tokens\":18}}\n\n");
    }
    if reply.done {
        body.push_str("data: [DONE]\n\n");
    }
    body
}

fn request_prompt(body: &Value, role: &str) -> Option<String> {
    body["messages"]
        .as_array()?
        .iter()
        .find(|m| m["role"] == role)
        .and_then(|m| m["content"].as_str())
        .map(str::to_owned)
}

fn spawn_server(
    connections: usize,
    responder: Responder,
    hook: Hook,
) -> (String, Arc<AtomicUsize>, Arc<Mutex<Vec<Value>>>) {
    let bodies = Arc::new(Mutex::new(Vec::<Value>::new()));
    let sink = Arc::clone(&bodies);
    let accepted = Arc::new(AtomicUsize::new(0));
    let counter = Arc::clone(&accepted);
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind");
    let port = listener.local_addr().expect("addr").port();
    std::thread::spawn(move || {
        for index in 0..connections {
            let Ok((mut stream, _)) = listener.accept() else {
                return;
            };
            counter.fetch_add(1, Ordering::SeqCst);
            let mut received = Vec::new();
            let mut chunk = [0u8; 8192];
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
            let body: Value =
                serde_json::from_slice(&received[header_end..]).unwrap_or(Value::Null);
            let prompt = request_prompt(&body, "user").unwrap_or_default();
            sink.lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .push(body);
            if let Some(hook) = hook
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .as_ref()
            {
                hook(index);
            }
            let reply = responder(index, &prompt);
            if let Some((status_line, error_body)) = reply.status {
                let response = format!(
                    "HTTP/1.1 {status_line}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{error_body}",
                    error_body.len()
                );
                let _ = stream.write_all(response.as_bytes());
                let _ = stream.flush();
                continue;
            }
            let _ = stream.write_all(
                b"HTTP/1.1 200 OK\r\nContent-Type: text/event-stream\r\nConnection: close\r\n\r\n",
            );
            let full = sse_body(&reply);
            if reply.delay_ms == 0 {
                let _ = stream.write_all(full.as_bytes());
            } else {
                for frame in full.split_inclusive("\n\n") {
                    let _ = stream.write_all(frame.as_bytes());
                    let _ = stream.flush();
                    std::thread::sleep(std::time::Duration::from_millis(reply.delay_ms));
                }
            }
            let _ = stream.flush();
        }
    });
    (
        format!("http://127.0.0.1:{port}/v1/chat/completions"),
        accepted,
        bodies,
    )
}

/// `(n, source lines)` for every SEGMENT block of a prompt, TM and CONTEXT blocks skipped.
fn prompt_blocks(prompt: &str) -> Vec<(u32, String)> {
    let mut out = Vec::new();
    let mut current: Option<(u32, Vec<&str>)> = None;
    let mut in_tm = false;
    for line in prompt.lines() {
        let t = line.trim();
        if let Some(n) = t.strip_prefix("<<<SEGMENT ") {
            current = Some((n.trim().parse().expect("block number"), Vec::new()));
        } else if t == "SEGMENT>>>" {
            if let Some((n, lines)) = current.take() {
                out.push((n, lines.join("\n")));
            }
        } else if t == "<<<TM" {
            in_tm = true;
        } else if t == "TM>>>" {
            in_tm = false;
        } else if let Some((_, lines)) = current.as_mut()
            && !in_tm
        {
            lines.push(line);
        }
    }
    out
}

fn echo(prompt: &str, skip: &[u32]) -> String {
    prompt_blocks(prompt)
        .into_iter()
        .filter(|(n, _)| !skip.contains(n))
        .map(|(n, source)| format!("<<<SEGMENT {n}\nVI {source}\nSEGMENT>>>"))
        .collect::<Vec<_>>()
        .join("\n")
}

// ── Harness ──────────────────────────────────────────────────────────────────────────

fn sentences(count: usize) -> String {
    (1..=count)
        .map(|i| format!("Sentence number {i} arrives."))
        .collect::<Vec<_>>()
        .join(" ")
}

struct Harness {
    app: tauri::App<MockRuntime>,
    dirs: Vec<PathBuf>,
    accepted: Arc<AtomicUsize>,
    bodies: Arc<Mutex<Vec<Value>>>,
    hook: Hook,
    ids: Vec<i64>,
}

impl Harness {
    fn new(tag: &str, text: &str, connections: usize, responder: Responder) -> Self {
        Self::with_set(tag, text, connections, responder, "{{source_segment}}")
    }

    fn with_set(
        tag: &str,
        text: &str,
        connections: usize,
        responder: Responder,
        set_body: &str,
    ) -> Self {
        let global_dir = temp_dir(&format!("{tag}-global"));
        let work_dir = temp_dir(&format!("{tag}-work"));
        let global =
            Store::open(StoreSpec::global(global_dir.join("global.db"))).expect("open global.db");
        let hook: Hook = Arc::new(Mutex::new(None));
        let (endpoint, accepted, bodies) = spawn_server(connections, responder, Arc::clone(&hook));
        write_field(&global, AiConfigField::Endpoint, &endpoint).expect("endpoint");
        write_field(&global, AiConfigField::Model, "gpt-wire").expect("model");
        let open = create_work_from_text(&work_dir, tag, "en", "", text.to_owned()).expect("work");
        prompt_set_create(
            Some(&global),
            Some(&open),
            PromptSetTier::Global,
            PromptSetKind::Translate,
            "Plain",
            set_body,
        )
        .expect("prompt set");
        let ids: Vec<i64> = read_open_chapter_segments(Some(&open))
            .expect("chapter")
            .segments
            .iter()
            .map(|s| s.id)
            .collect();
        let app = mock_builder()
            .build(mock_context(noop_assets()))
            .expect("mock app");
        app.manage(global);
        app.manage(OpenWorkState::new(Some(open)));
        app.manage(LastAssembledPromptState::new(None));
        app.manage(AiCallRecordsState::default());
        app.manage(AiTranslateGeneration::default());
        Self {
            app,
            dirs: vec![global_dir, work_dir],
            accepted,
            bodies,
            hook,
            ids,
        }
    }

    fn with_open<T>(&self, f: impl FnOnce(&OpenWork, &Store) -> T) -> T {
        let state = self.app.state::<OpenWorkState>();
        let guard = state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let global = self.app.state::<Store>();
        f(guard.as_ref().expect("open work"), &global)
    }

    fn run(
        &self,
        scope: AiTranslateScope,
    ) -> (
        Result<AiTranslateOutcomeWire, auratranslate_lib::core::i18n::IpcError>,
        Vec<Value>,
    ) {
        self.run_with(scope, |_| {})
    }

    fn run_with(
        &self,
        scope: AiTranslateScope,
        on_event: impl Fn(&Value) + Send + Sync + 'static,
    ) -> (
        Result<AiTranslateOutcomeWire, auratranslate_lib::core::i18n::IpcError>,
        Vec<Value>,
    ) {
        let events = Arc::new(Mutex::new(Vec::<Value>::new()));
        let sink = Arc::clone(&events);
        let channel = tauri::ipc::Channel::<AiTranslateBatchEventWire>::new(move |body| {
            if let tauri::ipc::InvokeResponseBody::Json(json) = body {
                let value: Value = serde_json::from_str(&json).expect("event json");
                on_event(&value);
                sink.lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner)
                    .push(value);
            }
            Ok(())
        });
        let result = tauri::async_runtime::block_on(wire::ai_translate_batch(
            self.app.handle().clone(),
            scope,
            Some("Plain".to_owned()),
            channel,
        ));
        let seen = events
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clone();
        (result, seen)
    }

    fn selection(&self, ids: &[i64]) -> AiTranslateScope {
        AiTranslateScope::Selection {
            segment_ids: ids.to_vec(),
        }
    }

    fn targets(&self) -> Vec<String> {
        self.with_open(|open, _| {
            read_open_chapter_segments(Some(open))
                .expect("chapter")
                .segments
                .iter()
                .map(|s| s.target_text.clone())
                .collect()
        })
    }

    fn prompts(&self) -> Vec<String> {
        self.bodies
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .iter()
            .filter_map(|b| request_prompt(b, "user"))
            .collect()
    }

    fn translate_ids(&self, ids: &[i64], text: &str) {
        self.with_open(|open, _| {
            let edits: Vec<SegmentTargetEdit> = ids
                .iter()
                .map(|&id| SegmentTargetEdit {
                    id,
                    target_text: text.to_owned(),
                })
                .collect();
            let chapter = read_open_chapter_segments(Some(open)).expect("chapter");
            save_segment_targets(Some(open), chapter.chapter_id, &edits).expect("save targets");
        });
    }
}

impl Drop for Harness {
    fn drop(&mut self) {
        for dir in &self.dirs {
            let _ = fs::remove_dir_all(Path::new(dir));
        }
    }
}

fn echo_responder() -> Responder {
    Arc::new(|_, prompt| Reply::stop(echo(prompt, &[])).with_usage())
}

fn lock_keychain(key: &str) -> std::sync::MutexGuard<'static, ()> {
    let guard = KEYCHAIN_LOCK
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    save_key(key);
    guard
}

fn of_kind<'a>(events: &'a [Value], kind: &str) -> Vec<&'a Value> {
    events.iter().filter(|e| e["kind"] == kind).collect()
}

fn reasons(events: &[Value]) -> Vec<(i64, String)> {
    of_kind(events, "discarded")
        .iter()
        .map(|e| {
            (
                e["segment_id"].as_i64().expect("segment_id"),
                e["reason"].as_str().expect("reason").to_owned(),
            )
        })
        .collect()
}

fn received_ids(events: &[Value]) -> Vec<i64> {
    of_kind(events, "received")
        .iter()
        .map(|e| e["segment_id"].as_i64().expect("segment_id"))
        .collect()
}

fn planned_sizes(events: &[Value]) -> Vec<usize> {
    events[0]["calls"]
        .as_array()
        .expect("calls")
        .iter()
        .map(|c| c["segment_ids"].as_array().expect("ids").len())
        .collect()
}

/// Text of each CONTEXT block that stands before the first SEGMENT block, and how many CONTEXT
/// blocks stand after it.
fn context_layout(prompt: &str) -> (Vec<String>, usize) {
    let mut before = Vec::new();
    let mut after = 0;
    let mut seen_segment = false;
    let mut current: Option<Vec<&str>> = None;
    for line in prompt.lines() {
        match line.trim() {
            "<<<CONTEXT" => current = Some(Vec::new()),
            "CONTEXT>>>" => {
                if let Some(lines) = current.take() {
                    if seen_segment {
                        after += 1;
                    } else {
                        before.push(lines.join("\n"));
                    }
                }
            }
            t if t.starts_with("<<<SEGMENT ") => seen_segment = true,
            _ => {
                if let Some(lines) = current.as_mut() {
                    lines.push(line);
                }
            }
        }
    }
    (before, after)
}

fn sentence(n: usize) -> String {
    format!("Sentence number {n} arrives.")
}

struct SingleRun {
    result: Result<AiTranslateOutcomeWire, auratranslate_lib::core::i18n::IpcError>,
    pieces: Vec<String>,
}

impl SingleRun {
    fn shown(&self) -> String {
        self.pieces.concat()
    }

    fn error_code(&self) -> String {
        self.result
            .as_ref()
            .expect_err("run must fail")
            .code()
            .to_owned()
    }
}

fn run_single(h: &Harness, id: i64) -> SingleRun {
    let pieces = Arc::new(Mutex::new(Vec::<String>::new()));
    let sink = Arc::clone(&pieces);
    let channel = tauri::ipc::Channel::<String>::new(move |body| {
        if let tauri::ipc::InvokeResponseBody::Json(json) = body {
            sink.lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .push(serde_json::from_str(&json).expect("piece json"));
        }
        Ok(())
    });
    let result = tauri::async_runtime::block_on(wire::ai_translate_segment(
        h.app.handle().clone(),
        id,
        Some("Plain".to_owned()),
        channel,
    ));
    let pieces = pieces
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .clone();
    SingleRun { result, pieces }
}

fn one_block(text: &str) -> String {
    format!("<<<SEGMENT 1\n{text}\nSEGMENT>>>")
}

// ── Cases (a)-(f): reading and reconciling one call ─────────────────────────────────

#[test]
fn a_stream_cut_without_a_finish_reason_keeps_closed_blocks_and_marks_the_rest_no_finish() {
    let _k = lock_keychain("sk-batch-a");
    let h = Harness::new(
        "a",
        &sentences(4),
        1,
        Arc::new(|_, _| {
            Reply::no_finish(
                "<<<SEGMENT 1\nVI one\nSEGMENT>>>\n<<<SEGMENT 2\nVI two\nSEGMENT>>>\n<<<SEGMENT 3\nVI half",
            )
        }),
    );
    let (result, events) = h.run(h.selection(&h.ids));
    assert!(matches!(result, Ok(AiTranslateOutcomeWire::Done { .. })));
    assert_eq!(received_ids(&events), vec![h.ids[0], h.ids[1]]);
    assert_eq!(
        reasons(&events),
        vec![
            (h.ids[2], "no_finish".to_owned()),
            (h.ids[3], "no_finish".to_owned())
        ]
    );
    assert!(
        h.targets().iter().all(String::is_empty),
        "a batch never writes target_text"
    );
}

#[test]
fn finish_reason_length_is_a_cut_like_an_absent_one() {
    let _k = lock_keychain("sk-batch-b");
    let h = Harness::new(
        "b",
        &sentences(3),
        1,
        Arc::new(|_, _| {
            Reply::with_finish(
                "<<<SEGMENT 1\nVI one\nSEGMENT>>>\n<<<SEGMENT 2\nVI tw",
                "length",
            )
        }),
    );
    let (_, events) = h.run(h.selection(&h.ids));
    assert_eq!(received_ids(&events), vec![h.ids[0]]);
    assert_eq!(
        reasons(&events),
        vec![
            (h.ids[1], "no_finish".to_owned()),
            (h.ids[2], "no_finish".to_owned())
        ]
    );
}

#[test]
fn a_stop_with_one_block_missing_loses_only_that_sentence_and_the_next_call_still_goes() {
    let _k = lock_keychain("sk-batch-c");
    let h = Harness::new(
        "c",
        &sentences(150),
        2,
        Arc::new(|index, prompt| {
            let skip: &[u32] = if index == 0 { &[7] } else { &[] };
            Reply::stop(echo(prompt, skip))
        }),
    );
    let (result, events) = h.run(h.selection(&h.ids));
    assert!(result.is_ok());
    assert_eq!(
        h.accepted.load(Ordering::SeqCst),
        2,
        "the second call must be sent"
    );
    assert_eq!(reasons(&events), vec![(h.ids[6], "missing".to_owned())]);
    assert_eq!(received_ids(&events).len(), 149);
}

#[test]
fn two_blocks_with_one_index_are_both_dropped_and_the_index_is_never_received() {
    let _k = lock_keychain("sk-batch-d");
    let h = Harness::new(
        "d",
        &sentences(3),
        1,
        Arc::new(|_, _| {
            Reply::stop(
                "<<<SEGMENT 1\nVI one\nSEGMENT>>>\n<<<SEGMENT 2\nVI a\nSEGMENT>>>\n<<<SEGMENT 2\nVI b\nSEGMENT>>>\n<<<SEGMENT 3\nVI three\nSEGMENT>>>",
            )
        }),
    );
    let (_, events) = h.run(h.selection(&h.ids));
    assert_eq!(received_ids(&events), vec![h.ids[0], h.ids[2]]);
    assert_eq!(
        reasons(&events),
        vec![(h.ids[1], "duplicate_index".to_owned())]
    );
}

#[test]
fn an_empty_block_is_discarded_as_empty() {
    let _k = lock_keychain("sk-batch-e");
    let h = Harness::new(
        "e",
        &sentences(2),
        1,
        Arc::new(|_, _| {
            Reply::stop("<<<SEGMENT 1\n  \nSEGMENT>>>\n<<<SEGMENT 2\nVI two\nSEGMENT>>>")
        }),
    );
    let (_, events) = h.run(h.selection(&h.ids));
    assert_eq!(reasons(&events), vec![(h.ids[0], "empty".to_owned())]);
    assert_eq!(received_ids(&events), vec![h.ids[1]]);
}

#[test]
fn an_open_line_missing_its_angle_brackets_still_opens_the_block() {
    let _k = lock_keychain("sk-batch-f");
    let h = Harness::new(
        "f",
        &sentences(2),
        1,
        Arc::new(|_, _| {
            Reply::stop("<<<SEGMENT 1\nVI one\nSEGMENT>>>\nSEGMENT 2\nVI two\nSEGMENT>>>")
        }),
    );
    let (_, events) = h.run(h.selection(&h.ids));
    assert_eq!(received_ids(&events), vec![h.ids[0], h.ids[1]]);
    let second = of_kind(&events, "received")[1];
    assert_eq!(second["text"], "VI two");
}

#[test]
fn a_source_line_equal_to_a_marker_is_guarded_and_comes_back_whole_without_closing_early() {
    let _k = lock_keychain("sk-batch-g");
    let h = Harness::new("g", "Placeholder.", 1, echo_responder());
    assert_eq!(h.ids.len(), 1);
    let id = h.ids[0];
    h.with_open(|open, _| {
        open.store
            .write(move |tx: &Transaction<'_>| {
                tx.execute(
                    "UPDATE segment SET source_text = 'First line.\nSEGMENT>>>\nThird line.' WHERE id = ?1",
                    [id],
                )
            })
            .expect("set multi-line source");
    });
    let (_, events) = h.run(h.selection(&h.ids));
    let prompt = &h.prompts()[0];
    assert!(prompt.contains("S\u{200b}EGMENT>>>"), "{prompt:?}");
    assert_eq!(
        prompt.matches("SEGMENT>>>").count(),
        1,
        "only the real closer: {prompt:?}"
    );
    let received = of_kind(&events, "received");
    assert_eq!(received.len(), 1, "{events:?}");
    assert_eq!(
        received[0]["text"].as_str().expect("text"),
        "VI First line.\nSEGMENT>>>\nThird line."
    );
}

// ── Cases (h)-(h2): cancel and the moment of Received ───────────────────────────────

#[test]
fn a_cancel_between_two_calls_sends_exactly_the_calls_already_started_and_closes_the_first() {
    let _k = lock_keychain("sk-batch-h");
    let h = Harness::new("h", &sentences(150), 2, echo_responder());
    let handle = h.app.handle().clone();
    let (result, events) = h.run_with(h.selection(&h.ids), move |event| {
        if event["kind"] == "call_done" && event["call_index"] == 1 {
            wire::ai_translate_cancel(handle.clone());
        }
    });
    assert!(matches!(result, Ok(AiTranslateOutcomeWire::Cancelled)));
    assert_eq!(h.accepted.load(Ordering::SeqCst), 1);
    assert_eq!(of_kind(&events, "call_done").len(), 1);
    assert!(of_kind(&events, "call_started").len() == 1);
    assert_eq!(received_ids(&events).len(), 100);
}

#[test]
fn a_cancel_mid_stream_still_closes_the_call_and_marks_the_open_sentences_interrupted() {
    let _k = lock_keychain("sk-batch-h-mid");
    let h = Harness::new(
        "h-mid",
        &sentences(3),
        1,
        Arc::new(|_, _| {
            let mut reply = Reply::stop(
                "<<<SEGMENT 1\nVI one\nSEGMENT>>>\n<<<SEGMENT 2\nVI two\nSEGMENT>>>\n<<<SEGMENT 3\nVI three\nSEGMENT>>>",
            );
            reply.chunk_chars = 12;
            reply.delay_ms = 40;
            reply
        }),
    );
    let handle = h.app.handle().clone();
    let (result, events) = h.run_with(h.selection(&h.ids), move |event| {
        if event["kind"] == "token" {
            wire::ai_translate_cancel(handle.clone());
        }
    });
    assert!(matches!(result, Ok(AiTranslateOutcomeWire::Cancelled)));
    assert_eq!(
        of_kind(&events, "call_done").len(),
        1,
        "CallDone is sent on cancel"
    );
    let interrupted: Vec<_> = reasons(&events)
        .into_iter()
        .filter(|(_, r)| r == "interrupted")
        .collect();
    assert!(!interrupted.is_empty(), "{events:?}");
}

#[test]
fn received_is_never_sent_before_the_stream_ends_so_a_block_that_repeats_later_gets_none() {
    let _k = lock_keychain("sk-batch-h2");
    let h = Harness::new(
        "h2",
        &sentences(2),
        1,
        Arc::new(|_, _| {
            Reply::stop(
                "<<<SEGMENT 1\nVI one\nSEGMENT>>>\n<<<SEGMENT 2\nVI a\nSEGMENT>>>\n<<<SEGMENT 2\nVI b\nSEGMENT>>>",
            )
        }),
    );
    let (_, events) = h.run(h.selection(&h.ids));
    assert!(
        !received_ids(&events).contains(&h.ids[1]),
        "no event may let sentence 2 reach the editor: {events:?}"
    );
    let last_token = events
        .iter()
        .rposition(|e| e["kind"] == "token")
        .expect("tokens were streamed");
    let first_verdict = events
        .iter()
        .position(|e| e["kind"] == "received" || e["kind"] == "discarded")
        .expect("verdicts");
    let done = events
        .iter()
        .position(|e| e["kind"] == "call_done")
        .expect("done");
    assert!(last_token < first_verdict && first_verdict < done);
}

// ── Cases (i)-(m): planning ──────────────────────────────────────────────────────────

#[test]
fn a_selection_of_two_separate_ranges_makes_two_calls_and_no_call_holds_both_ranges_segments() {
    let _k = lock_keychain("sk-batch-i");
    let h = Harness::new("i", &sentences(5), 2, echo_responder());
    let ids = [h.ids[0], h.ids[1], h.ids[3], h.ids[4]];
    let (_, events) = h.run(h.selection(&ids));
    assert_eq!(planned_sizes(&events), vec![2, 2]);
    let prompts = h.prompts();
    assert_eq!(prompts.len(), 2);
    let sources = |prompt: &str| -> Vec<String> {
        prompt_blocks(prompt).into_iter().map(|(_, s)| s).collect()
    };
    assert_eq!(sources(&prompts[0]), vec![sentence(1), sentence(2)]);
    assert_eq!(sources(&prompts[1]), vec![sentence(4), sentence(5)]);
    assert_eq!(context_layout(&prompts[0]), (vec![], 0));
    assert_eq!(
        context_layout(&prompts[1]),
        (vec![sentence(1), sentence(2), sentence(3)], 0),
        "the second call carries its own three preceding sentences, taken in text order"
    );
}

#[test]
fn two_hundred_fifty_sentences_become_calls_of_100_100_and_50_and_planned_lists_them_all() {
    let _k = lock_keychain("sk-batch-j");
    let h = Harness::new("j", &sentences(250), 3, echo_responder());
    let (result, events) = h.run(AiTranslateScope::Chapter);
    assert!(result.is_ok());
    assert_eq!(planned_sizes(&events), vec![100, 100, 50]);
    assert_eq!(of_kind(&events, "planned").len(), 1);
    assert_eq!(h.accepted.load(Ordering::SeqCst), 3);
    assert_eq!(received_ids(&events).len(), 250);
}

#[test]
fn a_run_of_four_translated_sentences_cuts_the_call_and_the_new_one_carries_three_as_context() {
    let _k = lock_keychain("sk-batch-k");
    let h = Harness::new("k", &sentences(10), 2, echo_responder());
    h.translate_ids(&h.ids[3..7], "Da dich");
    let (_, events) = h.run(AiTranslateScope::Chapter);
    assert_eq!(planned_sizes(&events), vec![3, 3]);
    let prompts = h.prompts();
    assert_eq!(
        prompts[0].matches("<<<CONTEXT").count(),
        0,
        "{}",
        prompts[0]
    );
    assert_eq!(
        prompts[1].matches("<<<CONTEXT").count(),
        3,
        "{}",
        prompts[1]
    );
    assert!(
        !prompts[1].contains("number 4 arrives"),
        "only the last three: {}",
        prompts[1]
    );
    assert!(prompts[1].contains("number 5 arrives"));
}

#[test]
fn a_short_run_of_translated_sentences_stays_inside_one_call_as_context() {
    let _k = lock_keychain("sk-batch-k2");
    let h = Harness::new("k2", &sentences(6), 1, echo_responder());
    h.translate_ids(&h.ids[2..4], "Da dich");
    let (_, events) = h.run(AiTranslateScope::Chapter);
    assert_eq!(planned_sizes(&events), vec![4]);
    assert_eq!(h.prompts()[0].matches("<<<CONTEXT").count(), 2);
}

#[test]
fn an_omitted_sentence_is_in_no_prompt_and_does_not_split_a_selection() {
    let _k = lock_keychain("sk-batch-l");
    let h = Harness::new("l", &sentences(4), 1, echo_responder());
    h.with_open(|open, _| set_segment_omitted(Some(open), h.ids[2], true).expect("omit"));
    let (_, events) = h.run(h.selection(&h.ids));
    assert_eq!(
        planned_sizes(&events),
        vec![3],
        "adjacency is kept across an omitted sentence"
    );
    for prompt in h.prompts() {
        assert!(!prompt.contains("number 3 arrives"), "{prompt}");
    }
}

#[test]
fn context_sentences_are_in_the_prompt_but_no_event_carries_their_id() {
    let _k = lock_keychain("sk-batch-m");
    let h = Harness::new("m", &sentences(5), 1, echo_responder());
    h.translate_ids(&h.ids[2..3], "Da dich");
    let (_, events) = h.run(AiTranslateScope::Chapter);
    assert!(h.prompts()[0].contains("<<<CONTEXT"));
    let context_id = h.ids[2];
    assert!(
        !events.iter().any(|e| e["segment_id"] == context_id),
        "no event names the context sentence"
    );
    let planned: Vec<i64> = events[0]["calls"]
        .as_array()
        .expect("calls")
        .iter()
        .flat_map(|c| c["segment_ids"].as_array().expect("ids").iter())
        .map(|v| v.as_i64().expect("id"))
        .collect();
    assert!(!planned.contains(&context_id), "{planned:?}");
}

// ── Cases (n)-(q): flags, prompt assembly, errors ───────────────────────────────────

#[test]
fn han_left_in_a_translation_of_a_han_source_sets_the_flag_and_the_sentence_is_still_received() {
    let _k = lock_keychain("sk-batch-n");
    let h = Harness::new(
        "n",
        "He said 龙 loudly. Plain words here.",
        1,
        Arc::new(|_, _| {
            Reply::stop(
                "<<<SEGMENT 1\nAnh ta noi 龙\nSEGMENT>>>\n<<<SEGMENT 2\nHan 龙 o day\nSEGMENT>>>",
            )
        }),
    );
    let (_, events) = h.run(h.selection(&h.ids));
    let received = of_kind(&events, "received");
    assert_eq!(received.len(), 2);
    assert_eq!(received[0]["has_han"], true);
    assert_eq!(
        received[1]["has_han"], false,
        "the source of sentence 2 has no Han"
    );
}

#[test]
fn the_format_instruction_follows_the_system_prompt_and_alone_forms_the_message_when_it_is_empty() {
    let _k = lock_keychain("sk-batch-o");
    let h = Harness::new("o", &sentences(2), 2, echo_responder());
    let _ = h.run(h.selection(&h.ids));
    let system = |h: &Harness, i: usize| {
        request_prompt(&h.bodies.lock().unwrap()[i], "system").expect("system message")
    };
    let first = system(&h, 0);
    assert!(first.starts_with(DEFAULT_SYSTEM_PROMPT), "{first}");
    assert!(first.ends_with("do not translate or copy them."), "{first}");

    h.with_open(|_, global| write_field(global, AiConfigField::SystemPrompt, "").expect("blank"));
    let _ = h.run(h.selection(&h.ids));
    let second = system(&h, 1);
    assert!(second.starts_with("Reply format:"), "{second}");
    assert!(!second.contains(DEFAULT_SYSTEM_PROMPT.trim()));
}

#[test]
fn a_body_without_the_glossary_marker_asks_for_no_glossary_and_the_ledger_says_so() {
    let _k = lock_keychain("sk-batch-p");
    let h = Harness::new("p", &sentences(2), 1, echo_responder());
    let _ = h.run(h.selection(&h.ids));
    let single = h.app.state::<LastAssembledPromptState>();
    let calls = h.app.state::<AiCallRecordsState>();
    let wire_record = read_prompt_record_for_segment(single.inner(), calls.inner(), h.ids[0])
        .expect("record of the call");
    assert_eq!(wire_record.ledger.glossary.kind, "not_asked");
    assert_eq!(wire_record.ledger.tm.kind, "not_asked");
    let call = wire_record.call.expect("grouped call info");
    assert_eq!(call.call_index, 1);
    assert_eq!(call.requested_role, "translate");
    assert_eq!(call.block_number, Some(1));
    assert!(
        !h.prompts()[0].contains('→'),
        "no glossary pair in the prompt"
    );
}

#[test]
fn a_500_on_the_second_call_stops_the_operation_keeps_call_one_and_names_call_two() {
    let _k = lock_keychain("sk-batch-q");
    let h = Harness::new(
        "q",
        &sentences(150),
        2,
        Arc::new(|index, prompt| {
            if index == 0 {
                Reply::stop(echo(prompt, &[]))
            } else {
                Reply::status("500 Internal Server Error", "{}")
            }
        }),
    );
    let (result, events) = h.run(h.selection(&h.ids));
    let err = result.expect_err("operation must end with the provider error");
    assert_eq!(err.code(), "ai_translate.provider_server_error");
    assert_eq!(err.params().get("call_index"), Some(&"2".to_owned()));
    assert_eq!(
        err.params().get("first_segment_id"),
        Some(&h.ids[100].to_string())
    );
    assert_eq!(
        err.params().get("last_segment_id"),
        Some(&h.ids[149].to_string())
    );
    assert_eq!(received_ids(&events).len(), 100, "call one is kept");
    assert_eq!(of_kind(&events, "call_done").len(), 2);
    let interrupted = reasons(&events)
        .into_iter()
        .filter(|(_, r)| r == "interrupted")
        .count();
    assert_eq!(interrupted, 50);
    assert!(h.targets().iter().all(String::is_empty));
}

#[test]
fn a_call_that_fails_after_it_was_sent_is_still_recorded_as_sent() {
    let _k = lock_keychain("sk-batch-q2");
    let h = Harness::new(
        "q2",
        &sentences(150),
        2,
        Arc::new(|index, prompt| {
            if index == 0 {
                Reply::stop(echo(prompt, &[]))
            } else {
                Reply::status("500 Internal Server Error", "{}")
            }
        }),
    );
    let (result, _) = h.run(h.selection(&h.ids));
    assert!(result.is_err());
    let single = h.app.state::<LastAssembledPromptState>();
    let calls = h.app.state::<AiCallRecordsState>();
    let record = read_prompt_record_for_segment(single.inner(), calls.inner(), h.ids[100])
        .expect("record of the failed call");
    assert_eq!(record.call.expect("grouped call info").call_index, 2);
    assert!(record.sent_at.is_some(), "the failed call went out");
}

// ── Cases (r)-(u) ────────────────────────────────────────────────────────────────────

#[test]
fn a_retry_selection_resends_an_already_translated_sentence_as_a_block_not_as_context() {
    let _k = lock_keychain("sk-batch-r");
    let h = Harness::new("r", &sentences(4), 1, echo_responder());
    h.translate_ids(&h.ids[1..2], "Da dich");
    let ids = [h.ids[0], h.ids[1], h.ids[3]];
    let (_, events) = h.run(h.selection(&ids));
    let calls = events[0]["calls"].as_array().expect("calls");
    let planned: Vec<i64> = calls
        .iter()
        .flat_map(|c| c["segment_ids"].as_array().expect("ids").iter())
        .map(|v| v.as_i64().expect("id"))
        .collect();
    assert!(
        planned.contains(&h.ids[1]),
        "the translated sentence is re-sent: {planned:?}"
    );
    let prompt = &h.prompts()[0];
    assert!(!prompt.contains("<<<CONTEXT"));
    assert!(prompt.contains("number 2 arrives"));
}

#[test]
fn promoting_a_received_sentence_is_refused_after_its_source_changed_and_writes_nothing() {
    let _k = lock_keychain("sk-batch-s");
    let h = Harness::new("s", &sentences(2), 1, echo_responder());
    let (_, events) = h.run(h.selection(&h.ids));
    let received = of_kind(&events, "received");
    let hash = received[0]["source_hash"]
        .as_str()
        .expect("hash")
        .to_owned();
    let text = received[0]["text"].as_str().expect("text").to_owned();
    let id = h.ids[0];

    h.with_open(|open, _| {
        open.store
            .write(move |tx: &Transaction<'_>| {
                tx.execute(
                    "UPDATE segment SET source_text = 'Edited meanwhile.' WHERE id = ?1",
                    [id],
                )
            })
            .expect("edit source");
    });
    let err = auratranslate_lib::commands::segment::wire::promote_ai_translation(
        h.app.handle().clone(),
        id,
        text.clone(),
        false,
        Some(hash.clone()),
    )
    .expect_err("a changed source must be refused");
    assert_eq!(err.code(), "segment.source_changed");
    assert!(h.targets().iter().all(String::is_empty), "nothing written");

    let other = of_kind(&events, "received")[1];
    let outcome = auratranslate_lib::commands::segment::wire::promote_ai_translation(
        h.app.handle().clone(),
        h.ids[1],
        other["text"].as_str().expect("text").to_owned(),
        false,
        Some(other["source_hash"].as_str().expect("hash").to_owned()),
    )
    .expect("an unchanged source goes through");
    assert!(!outcome.needs_confirmation);
}

#[test]
fn call_done_carries_each_calls_own_usage_and_nothing_else_carries_usage() {
    let _k = lock_keychain("sk-batch-t");
    let h = Harness::new("t", &sentences(150), 2, echo_responder());
    let (_, events) = h.run(h.selection(&h.ids));
    let done = of_kind(&events, "call_done");
    assert_eq!(done.len(), 2);
    let total: i64 = done
        .iter()
        .map(|e| e["usage"]["total_tokens"].as_i64().expect("usage"))
        .sum();
    assert_eq!(total, 36);
    for event in &events {
        if event["kind"] != "call_done" {
            assert!(event.get("usage").is_none(), "{event}");
        }
    }
    assert!(done.iter().all(|e| e["finish_reason"] == "stop"));
}

#[test]
fn the_tm_policy_is_read_once_per_operation_so_a_switch_flipped_between_calls_changes_nothing() {
    let _k = lock_keychain("sk-batch-u");
    let text = (1..=150)
        .map(|i| format!("The old dragon breathed fire at the quiet village {i}."))
        .collect::<Vec<_>>()
        .join(" ");
    let h = Harness::with_set(
        "u",
        &text,
        2,
        echo_responder(),
        "{{tm_similar_segments}}\n{{source_segment}}",
    );
    h.with_open(|open, _| {
        open.store
            .write(|tx: &Transaction<'_>| {
                insert_pair(
                    tx,
                    "The old dragon breathed fire at the quiet village.",
                    "Con rong tham khao",
                    PairOrigin::Other,
                )
            })
            .expect("seed tm");
    });
    let handle = h.app.handle().clone();
    *h.hook.lock().unwrap() = Some(Box::new(move |index| {
        if index == 0 {
            let state = handle.state::<OpenWorkState>();
            let guard = state
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            write_only_mine(&guard.as_ref().expect("work").store, true).expect("flip policy");
        }
    }));
    let _ = h.run(h.selection(&h.ids));
    let prompts = h.prompts();
    assert_eq!(prompts.len(), 2);
    assert!(
        prompts[0].contains("Con rong tham khao"),
        "call 1 sees the pair"
    );
    assert!(
        prompts[1].contains("Con rong tham khao"),
        "call 2 uses the policy read at the start, not the flipped one"
    );
}

#[test]
fn the_glossary_is_read_once_per_operation_so_an_edit_between_calls_changes_nothing() {
    let _k = lock_keychain("sk-batch-u-glossary");
    let text = (1..=150)
        .map(|i| format!("The old dragon breathed fire at the quiet village {i}."))
        .collect::<Vec<_>>()
        .join(" ");
    let h = Harness::with_set(
        "u-glossary",
        &text,
        2,
        echo_responder(),
        "{{glossary_terms}}\n{{source_segment}}",
    );
    let dragon_id = h.with_open(|open, _| {
        insert_manual_entry(&open.store, "dragon", Some("Rong A"), "", Category::Other)
            .expect("seed term")
    });
    let handle = h.app.handle().clone();
    *h.hook.lock().unwrap() = Some(Box::new(move |index| {
        if index == 0 {
            let state = handle.state::<OpenWorkState>();
            let guard = state
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            let store = &guard.as_ref().expect("work").store;
            confirm_translation(store, dragon_id, "Rong B").expect("edit term");
            insert_manual_entry(store, "village", Some("Lang moi"), "", Category::Other)
                .expect("add term");
        }
    }));
    let _ = h.run(h.selection(&h.ids));
    let prompts = h.prompts();
    assert_eq!(prompts.len(), 2);
    assert!(prompts[0].contains("Rong A"), "call 1 sees the term");
    assert!(
        prompts[1].contains("Rong A") && !prompts[1].contains("Rong B"),
        "call 2 keeps the translation read at the start"
    );
    assert!(
        !prompts[1].contains("Lang moi"),
        "a term added between calls is not injected"
    );
}

// ── Single sentence path flag and resend ────────────────────────────────────────────

#[test]
fn a_single_sentence_run_without_a_finish_reason_is_marked_maybe_truncated() {
    let _k = lock_keychain("sk-batch-single");
    let h = Harness::new(
        "single",
        "One sentence only.",
        2,
        Arc::new(|index, _| {
            if index == 0 {
                Reply::no_finish(one_block("Mot cau."))
            } else {
                Reply::stop(one_block("Mot cau."))
            }
        }),
    );
    let first = run_single(&h, h.ids[0]);
    assert!(matches!(
        first.result,
        Ok(AiTranslateOutcomeWire::Done {
            maybe_truncated: true,
            ..
        })
    ));
    assert_eq!(first.shown(), "Mot cau.");
    let second = run_single(&h, h.ids[0]);
    assert!(matches!(
        second.result,
        Ok(AiTranslateOutcomeWire::Done {
            maybe_truncated: false,
            ..
        })
    ));
    assert_eq!(second.shown(), "Mot cau.");
}

#[test]
fn resending_a_recorded_call_sends_its_prompt_byte_for_byte_and_reads_the_reply_the_same_way() {
    let _k = lock_keychain("sk-batch-resend");
    let h = Harness::new("resend", &sentences(3), 2, echo_responder());
    let _ = h.run(h.selection(&h.ids));
    let operation_id = {
        let records = h.app.state::<AiCallRecordsState>();
        let single = h.app.state::<LastAssembledPromptState>();
        read_prompt_record_for_segment(single.inner(), records.inner(), h.ids[0])
            .and_then(|w| w.call)
            .expect("recorded call")
            .operation_id
    };
    let events = Arc::new(Mutex::new(Vec::<Value>::new()));
    let sink = Arc::clone(&events);
    let channel = tauri::ipc::Channel::<AiTranslateBatchEventWire>::new(move |body| {
        if let tauri::ipc::InvokeResponseBody::Json(json) = body {
            sink.lock()
                .unwrap()
                .push(serde_json::from_str(&json).expect("json"));
        }
        Ok(())
    });
    let outcome = tauri::async_runtime::block_on(wire::ai_translate_resend_call(
        h.app.handle().clone(),
        operation_id,
        1,
        channel,
    ))
    .expect("resend");
    assert!(matches!(outcome, AiTranslateOutcomeWire::Done { .. }));
    let prompts = h.prompts();
    assert_eq!(prompts.len(), 2);
    assert_eq!(prompts[0], prompts[1], "same prompt, byte for byte");
    let events = events.lock().unwrap().clone();
    assert_eq!(received_ids(&events).len(), 3);
    assert_eq!(events[0]["calls"][0]["call_index"], 1);
}

// ── AD-65: neighbour sentences and the block reader on the single-sentence path ─────

fn ok_reply() -> Responder {
    Arc::new(|_, _| Reply::stop(one_block("VI mot")))
}

#[test]
fn a_single_sentence_mid_chapter_carries_exactly_three_preceding_context_blocks_skipping_an_omitted_one()
 {
    let _k = lock_keychain("sk-batch-65a");
    let h = Harness::new("65a", &sentences(7), 1, ok_reply());
    h.with_open(|open, _| set_segment_omitted(Some(open), h.ids[4], true).expect("omit"));
    let run = run_single(&h, h.ids[5]);
    assert!(run.result.is_ok(), "{:?}", run.result);
    let prompt = &h.prompts()[0];
    assert_eq!(
        context_layout(prompt),
        (vec![sentence(2), sentence(3), sentence(4)], 0),
        "three in `ord` order; the omitted sentence 5 is skipped and sentence 2 fills in: {prompt}"
    );
    assert_eq!(prompt_blocks(prompt), vec![(1, sentence(6))]);
    let recorded = h.app.state::<LastAssembledPromptState>().inner();
    let preview = read_prompt_record_for_segment(
        recorded,
        h.app.state::<AiCallRecordsState>().inner(),
        h.ids[5],
    )
    .expect("record");
    assert_eq!(
        &preview.prompt, prompt,
        "the inspector shows the sent bytes"
    );
}

#[test]
fn a_context_block_the_model_copies_never_reaches_the_channel_and_the_pieces_join_to_block_one() {
    let _k = lock_keychain("sk-batch-65b");
    let h = Harness::new(
        "65b",
        &sentences(4),
        1,
        Arc::new(|_, _| Reply {
            chunk_chars: 3,
            ..Reply::stop(
                "<<<CONTEXT\nSentence number 2 arrives. → Hai\nCONTEXT>>>\n<<<SEGMENT 1\nVI mot\nVI hai\nSEGMENT>>>\n",
            )
        }),
    );
    let run = run_single(&h, h.ids[3]);
    assert!(
        matches!(
            run.result,
            Ok(AiTranslateOutcomeWire::Done {
                maybe_truncated: false,
                ..
            })
        ),
        "{:?}",
        run.result
    );
    assert_eq!(run.shown(), "VI mot\nVI hai");
    assert!(h.targets().iter().all(String::is_empty));
}

#[test]
fn a_single_sentence_stream_cut_inside_block_one_shows_the_text_so_far_and_sets_maybe_truncated() {
    let _k = lock_keychain("sk-batch-65c1");
    let h = Harness::new(
        "65c1",
        &sentences(2),
        1,
        Arc::new(|_, _| Reply::no_finish("<<<SEGMENT 1\nVI nua cau")),
    );
    let run = run_single(&h, h.ids[1]);
    assert!(matches!(
        run.result,
        Ok(AiTranslateOutcomeWire::Done {
            maybe_truncated: true,
            ..
        })
    ));
    assert_eq!(run.shown(), "VI nua cau");
}

#[test]
fn a_single_sentence_reply_without_a_usable_block_one_is_result_missing_and_writes_nothing() {
    let _k = lock_keychain("sk-batch-65c2");
    let replies = [
        "Mot cau khong co khoi.",
        "<<<SEGMENT 2\nVI sai so\nSEGMENT>>>",
        "<<<SEGMENT 1\nVI a\nSEGMENT>>>\n<<<SEGMENT 1\nVI b\nSEGMENT>>>",
        "<<<SEGMENT 1\n\nSEGMENT>>>",
        "<<<SEGMENT 1\nVI a\n<<<SEGMENT 2\nVI b\nSEGMENT>>>",
        "<<<SEGMENT 1\n",
    ];
    let h = Harness::new(
        "65c2",
        &sentences(2),
        replies.len(),
        Arc::new(move |index, _| Reply::stop(replies[index])),
    );
    for (index, reply) in replies.iter().enumerate() {
        let run = run_single(&h, h.ids[1]);
        assert_eq!(
            run.error_code(),
            "ai_translate.result_missing",
            "reply {index}: {reply:?}"
        );
    }
    assert!(h.targets().iter().all(String::is_empty));
}

#[test]
fn the_first_sentence_has_no_context_block_and_the_second_has_exactly_one() {
    let _k = lock_keychain("sk-batch-65d");
    let h = Harness::new("65d", &sentences(3), 2, ok_reply());
    assert!(run_single(&h, h.ids[0]).result.is_ok());
    assert!(run_single(&h, h.ids[1]).result.is_ok());
    let prompts = h.prompts();
    assert_eq!(context_layout(&prompts[0]), (vec![], 0));
    assert_eq!(context_layout(&prompts[1]), (vec![sentence(1)], 0));
}

#[test]
fn an_untranslated_neighbour_is_source_only_and_a_translated_one_keeps_its_arrow() {
    let _k = lock_keychain("sk-batch-65e");
    let h = Harness::new("65e", &sentences(4), 1, ok_reply());
    h.translate_ids(&h.ids[1..2], "Da dich");
    assert!(run_single(&h, h.ids[3]).result.is_ok());
    assert_eq!(
        context_layout(&h.prompts()[0]),
        (
            vec![
                sentence(1),
                format!("{} → Da dich", sentence(2)),
                sentence(3)
            ],
            0
        )
    );
}

#[test]
fn retrying_a_missing_sentence_carries_the_same_neighbours_the_first_run_would_have() {
    let _k = lock_keychain("sk-batch-65g");
    let h = Harness::new(
        "65g",
        &sentences(5),
        2,
        Arc::new(|index, prompt| {
            if index == 0 {
                Reply::stop(echo(prompt, &[4]))
            } else {
                Reply::stop(echo(prompt, &[]))
            }
        }),
    );
    let (_, events) = h.run(AiTranslateScope::Chapter);
    assert_eq!(reasons(&events).len(), 1);
    let missing = reasons(&events)[0].0;
    assert_eq!(missing, h.ids[3]);
    let (_, retry_events) = h.run(h.selection(&[missing]));
    assert_eq!(received_ids(&retry_events), vec![missing]);
    let prompts = h.prompts();
    assert_eq!(
        context_layout(&prompts[1]),
        (vec![sentence(1), sentence(2), sentence(3)], 0)
    );
    assert_eq!(prompt_blocks(&prompts[1]), vec![(1, sentence(4))]);
}

#[test]
fn a_chapter_run_gives_the_first_call_three_neighbours_and_a_cap_cut_call_three_source_only_ones() {
    let _k = lock_keychain("sk-batch-65h1");
    let h = Harness::new("65h1", &sentences(10), 1, echo_responder());
    h.translate_ids(&h.ids[0..5], "Da dich");
    let (_, events) = h.run(AiTranslateScope::Chapter);
    assert_eq!(planned_sizes(&events), vec![5]);
    assert_eq!(
        context_layout(&h.prompts()[0]),
        (
            vec![
                format!("{} → Da dich", sentence(3)),
                format!("{} → Da dich", sentence(4)),
                format!("{} → Da dich", sentence(5)),
            ],
            0
        ),
        "a leading run of more than three translated sentences still gives the call three"
    );

    let cap = Harness::new("65h2", &sentences(250), 3, echo_responder());
    let (_, events) = cap.run(AiTranslateScope::Chapter);
    assert_eq!(planned_sizes(&events), vec![100, 100, 50]);
    let prompts = cap.prompts();
    assert_eq!(context_layout(&prompts[0]), (vec![], 0));
    assert_eq!(
        context_layout(&prompts[1]),
        (vec![sentence(98), sentence(99), sentence(100)], 0),
        "the call cut by the cap carries the last three sentences of the previous call, source only"
    );
    assert_eq!(
        context_layout(&prompts[2]),
        (vec![sentence(198), sentence(199), sentence(200)], 0)
    );
}

#[test]
fn a_context_neighbour_is_never_scanned_for_glossary_terms_and_no_event_names_it() {
    let _k = lock_keychain("sk-batch-65i");
    let h = Harness::with_set(
        "65i",
        "The old dragon slept. The quiet village woke.",
        2,
        echo_responder(),
        "{{glossary_terms}}\n{{source_segment}}",
    );
    h.with_open(|open, _| {
        insert_manual_entry(&open.store, "dragon", Some("Rong A"), "", Category::Other)
            .expect("seed term");
    });
    let (_, events) = h.run(h.selection(&h.ids[1..2]));
    assert!(run_single(&h, h.ids[1]).result.is_ok());
    for prompt in h.prompts() {
        assert_eq!(
            context_layout(&prompt),
            (vec!["The old dragon slept.".to_owned()], 0)
        );
        assert!(!prompt.contains("Rong A"), "{prompt}");
    }
    assert!(!events.iter().any(|e| e["segment_id"] == h.ids[0]));
}

#[test]
fn a_neighbour_line_that_equals_a_block_marker_is_guarded_with_a_zero_width_space() {
    let _k = lock_keychain("sk-batch-65j");
    let h = Harness::new("65j", &sentences(2), 1, ok_reply());
    h.translate_ids(&h.ids[0..1], "Da dich\nSEGMENT>>>");
    assert!(run_single(&h, h.ids[1]).result.is_ok());
    let prompt = &h.prompts()[0];
    let guarded = guard_marker_line("SEGMENT>>>");
    assert!(prompt.lines().any(|l| l == guarded), "{prompt}");
    assert_eq!(
        prompt.lines().filter(|l| l.trim() == "SEGMENT>>>").count(),
        1,
        "only the closing line of block 1 is a real marker: {prompt}"
    );
}

#[test]
fn chapter_context_stays_an_unknown_marker_while_the_neighbours_are_still_sent() {
    let _k = lock_keychain("sk-batch-65l");
    let h = Harness::with_set(
        "65l",
        &sentences(5),
        1,
        ok_reply(),
        "{{chapter_context}}\n{{source_segment}}",
    );
    assert!(run_single(&h, h.ids[4]).result.is_ok());
    let prompt = &h.prompts()[0];
    assert!(prompt.contains("{{chapter_context}}"), "{prompt}");
    assert_eq!(
        context_layout(prompt),
        (vec![sentence(2), sentence(3), sentence(4)], 0)
    );
}

#[test]
fn a_neighbour_never_comes_from_another_chapter() {
    let _k = lock_keychain("sk-batch-65-chapter");
    let h = Harness::new("65-chapter", &sentences(6), 2, ok_reply());
    let second_chapter = h.with_open(|open, _| {
        let moved = [h.ids[3], h.ids[4], h.ids[5]];
        open.store
            .write(move |tx: &Transaction<'_>| {
                tx.execute(
                    "INSERT INTO chapter (ord, title, source_text, status, created_at, updated_at) \
                     SELECT 2, 'Hai', '', status, created_at, updated_at FROM chapter LIMIT 1",
                    [],
                )?;
                let id = tx.last_insert_rowid();
                for segment_id in moved {
                    tx.execute(
                        "UPDATE segment SET chapter_id = ?1 WHERE id = ?2",
                        [id, segment_id],
                    )?;
                }
                Ok(id)
            })
            .expect("second chapter")
    });
    {
        let state = h.app.state::<OpenWorkState>();
        let mut guard = state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        guard.as_mut().expect("open work").chapter_id = second_chapter;
    }
    assert!(run_single(&h, h.ids[3]).result.is_ok());
    assert!(run_single(&h, h.ids[4]).result.is_ok());
    let prompts = h.prompts();
    assert_eq!(
        context_layout(&prompts[0]),
        (vec![], 0),
        "the first sentence of a chapter has no neighbour, whatever stands before it in the text"
    );
    assert_eq!(context_layout(&prompts[1]), (vec![sentence(4)], 0));
}

// ── Pure pieces the cases above rely on ──────────────────────────────────────────────

#[test]
fn guarding_changes_only_whole_marker_lines_and_unguarding_undoes_only_that() {
    for marker in [
        "SEGMENT>>>",
        "  <<<SEGMENT 3 ",
        "SEGMENT 12",
        "<<<CONTEXT",
        "TM>>>",
    ] {
        let guarded = guard_marker_line(marker);
        assert_ne!(guarded, marker, "{marker:?}");
        assert_eq!(unguard_marker_line(&guarded), marker);
    }
    for plain in [
        "the SEGMENT>>> word",
        "SEGMENT",
        "<<<SEGMENT x",
        "a\u{200b}b",
    ] {
        assert_eq!(guard_marker_line(plain), plain);
        assert_eq!(
            unguard_marker_line(plain),
            plain,
            "a zero-width space elsewhere is kept"
        );
    }
}

#[test]
fn text_shown_while_streaming_never_contains_a_marker_prefix_and_tm_copies_are_dropped() {
    let mut reader = BlockStream::new();
    let mut shown = String::new();
    for chunk in [
        "<<<SEG",
        "MENT 1\nHel",
        "lo wor",
        "ld\n<<<T",
        "M\nSrc → Tgt\nTM>>>\nSEGM",
        "ENT>>>\n",
    ] {
        for piece in reader.push(chunk) {
            assert_eq!(piece.index, 1);
            shown.push_str(&piece.text);
        }
    }
    assert_eq!(shown, "Hello world");
    let blocks = reader.finish();
    assert_eq!(blocks.len(), 1);
    assert_eq!(blocks[0].text, "Hello world");
}

#[test]
fn a_context_block_copied_inside_a_segment_never_reaches_the_shown_text_or_the_result() {
    let mut reader = BlockStream::new();
    let mut shown = String::new();
    for chunk in [
        "<<<SEGMENT 1\nHello\n<<<CONT",
        "EXT\nctx source\nctx target\nCONTEXT>>>\nworld\nSEGMENT>>>\n",
    ] {
        for piece in reader.push(chunk) {
            shown.push_str(&piece.text);
        }
    }
    assert_eq!(shown, "Hello\nworld");
    let blocks = reader.finish();
    assert_eq!(blocks.len(), 1);
    assert_eq!(blocks[0].text, "Hello\nworld");
}

// ── AD-66: the entry note travels with the pair ─────────────────────────────────────

const NOTE_LINE: &str = "dragon → Rong A — Lão long, xưng ta";

fn note_harness(tag: &str, text: &str, term_translation: Option<&str>) -> Harness {
    let h = Harness::with_set(
        tag,
        text,
        2,
        echo_responder(),
        "{{glossary_terms}}\n{{source_segment}}",
    );
    h.with_open(|open, _| {
        insert_manual_entry(
            &open.store,
            "dragon",
            term_translation,
            "Lão long, xưng ta",
            Category::Other,
        )
        .expect("seed term");
    });
    h
}

#[test]
fn a_confirmed_note_reaches_the_prompt_on_the_single_and_the_batch_path() {
    let _k = lock_keychain("sk-batch-66a");
    let h = note_harness("66a", "The old dragon slept.", Some("Rong A"));
    assert!(run_single(&h, h.ids[0]).result.is_ok());
    let (_, _) = h.run(h.selection(&h.ids));
    let prompts = h.prompts();
    assert_eq!(prompts.len(), 2, "one single call and one batch call");
    let carries_note: Vec<bool> = prompts
        .iter()
        .map(|p| p.lines().any(|l| l == NOTE_LINE))
        .collect();
    assert_eq!(carries_note, [true, true], "[single, batch]: {prompts:?}");
}

#[test]
fn a_pending_entry_note_never_reaches_the_prompt_on_either_path() {
    let _k = lock_keychain("sk-batch-66b");
    let h = note_harness("66b", "The old dragon slept.", None);
    assert!(run_single(&h, h.ids[0]).result.is_ok());
    let (_, _) = h.run(h.selection(&h.ids));
    let prompts = h.prompts();
    assert_eq!(prompts.len(), 2);
    for prompt in &prompts {
        assert!(!prompt.contains("Lão long"), "{prompt}");
    }
}

#[test]
fn a_note_appears_once_when_two_sentences_of_one_batch_call_share_the_term() {
    let _k = lock_keychain("sk-batch-66e");
    let h = note_harness(
        "66e",
        "The old dragon slept. The young dragon woke.",
        Some("Rong A"),
    );
    let (_, _) = h.run(h.selection(&h.ids));
    let prompts = h.prompts();
    assert_eq!(prompts.len(), 1, "both sentences travel in one call");
    assert_eq!(
        prompts[0].matches("Lão long, xưng ta").count(),
        1,
        "{}",
        prompts[0]
    );
}
