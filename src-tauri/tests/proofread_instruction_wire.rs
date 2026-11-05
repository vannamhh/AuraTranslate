//! Proofread scan with a chosen instruction (AD-68), driven through the real
//! `#[tauri::command]` shells of a `MockRuntime` app against a loopback SSE server.

use std::fs;
use std::io::{Read, Write};
use std::net::TcpListener;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex, Once};

use auratranslate_lib::commands::aiconfig::{ai_config_delete_key, ai_config_save_key};
use auratranslate_lib::commands::project::{OpenWork, OpenWorkState, create_work_from_text};
use auratranslate_lib::commands::promptset::{
    prompt_set_create, prompt_set_proofread_choice_set_global,
    prompt_set_proofread_choice_set_work, prompt_set_rename,
};
use auratranslate_lib::commands::proofread::{ProofreadGeneration, wire};
use auratranslate_lib::commands::segment::{promote_ai_translation, read_open_chapter_segments};
use auratranslate_lib::core::ai::rag::{ProofreadBlock, assemble_proofread_batch_prompt};
use auratranslate_lib::core::aiconfig::{AiConfigField, AiConfigTier, write_field};
use auratranslate_lib::core::i18n::IpcError;
use auratranslate_lib::core::promptset::{PromptSetKind, PromptSetTier};
use auratranslate_lib::core::store::{Store, StoreSpec};
use serde_json::Value;
use tauri::Manager as _;
use tauri::test::{MockRuntime, mock_builder, mock_context, noop_assets};

static NEXT_DIR: AtomicU64 = AtomicU64::new(0);

fn save_key(value: &str) {
    static INIT: Once = Once::new();
    INIT.call_once(|| {
        assert!(keyring::Entry::store_status().is_ok());
        keyring_core::set_default_store(keyring_core::mock::Store::new().expect("mock store"));
    });
    ai_config_delete_key(AiConfigTier::Global).expect("reset key");
    ai_config_save_key(AiConfigTier::Global, value).expect("save key");
}

fn temp_dir(tag: &str) -> PathBuf {
    let n = NEXT_DIR.fetch_add(1, Ordering::Relaxed);
    let dir = std::env::temp_dir().join(format!(
        "auratranslate-proofread-wire-{}-{tag}-{n}",
        std::process::id()
    ));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).expect("temp dir");
    dir
}

struct Server {
    endpoint: String,
    accepted: Arc<AtomicUsize>,
    bodies: Arc<Mutex<Vec<Value>>>,
}

/// Answers each connection with `reply(index)` as one SSE message that carries usage.
fn spawn_server(connections: usize, reply: impl Fn(usize) -> String + Send + 'static) -> Server {
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
            sink.lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .push(body);
            let text = serde_json::to_string(&reply(index)).expect("json string");
            let sse = format!(
                "data: {{\"choices\":[{{\"delta\":{{\"content\":{text}}}}}]}}\n\n\
                 data: {{\"choices\":[{{\"delta\":{{}},\"finish_reason\":\"stop\"}}]}}\n\n\
                 data: {{\"choices\":[],\"usage\":{{\"prompt_tokens\":11,\"completion_tokens\":7,\"total_tokens\":18}}}}\n\n\
                 data: [DONE]\n\n"
            );
            let _ = stream.write_all(
                b"HTTP/1.1 200 OK\r\nContent-Type: text/event-stream\r\nConnection: close\r\n\r\n",
            );
            let _ = stream.write_all(sse.as_bytes());
            let _ = stream.flush();
        }
    });
    Server {
        endpoint: format!("http://127.0.0.1:{port}/v1/chat/completions"),
        accepted,
        bodies,
    }
}

struct Harness {
    app: tauri::App<MockRuntime>,
    server: Server,
    dirs: Vec<PathBuf>,
    ids: Vec<i64>,
}

impl Drop for Harness {
    fn drop(&mut self) {
        for dir in &self.dirs {
            let _ = fs::remove_dir_all(dir);
        }
    }
}

struct Scan {
    result: Result<Value, IpcError>,
    events: Vec<Value>,
}

impl Harness {
    fn new(tag: &str, source: &str, target: &str, count: usize, connections: usize) -> Self {
        Self::with_reply(tag, source, target, count, connections, |_| "[]".to_owned())
    }

    fn with_reply(
        tag: &str,
        source: &str,
        target: &str,
        count: usize,
        connections: usize,
        reply: impl Fn(usize) -> String + Send + 'static,
    ) -> Self {
        save_key("sk-wire");
        let global_dir = temp_dir(&format!("{tag}-global"));
        let work_dir = temp_dir(&format!("{tag}-work"));
        let global =
            Store::open(StoreSpec::global(global_dir.join("global.db"))).expect("open global.db");
        let server = spawn_server(connections, reply);
        write_field(&global, AiConfigField::Endpoint, &server.endpoint).expect("endpoint");
        write_field(&global, AiConfigField::Model, "gpt-wire").expect("model");
        let text = (0..count)
            .map(|i| format!("{source} {i}"))
            .collect::<Vec<_>>()
            .join("\n\n");
        let open = create_work_from_text(&work_dir, tag, "en", "", text).expect("work");
        let ids: Vec<i64> = read_open_chapter_segments(Some(&open))
            .expect("chapter")
            .segments
            .iter()
            .map(|s| s.id)
            .collect();
        for (i, id) in ids.iter().enumerate() {
            promote_ai_translation(Some(&open), *id, &format!("{target} {i}"), false)
                .expect("target");
        }
        let app = mock_builder()
            .build(mock_context(noop_assets()))
            .expect("mock app");
        app.manage(global);
        app.manage(OpenWorkState::new(Some(open)));
        app.manage(ProofreadGeneration::default());
        Self {
            app,
            server,
            dirs: vec![global_dir, work_dir],
            ids,
        }
    }

    fn global<T>(&self, f: impl FnOnce(&Store) -> T) -> T {
        f(&self.app.state::<Store>())
    }

    fn with_open<T>(&self, f: impl FnOnce(&OpenWork, &Store) -> T) -> T {
        let state = self.app.state::<OpenWorkState>();
        let guard = state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let global = self.app.state::<Store>();
        f(guard.as_ref().expect("open work"), &global)
    }

    fn create(&self, tier: PromptSetTier, kind: PromptSetKind, name: &str, body: &str) -> i64 {
        self.with_open(|open, global| {
            prompt_set_create(Some(global), Some(open), tier, kind, name, body)
                .expect("create")
                .id
        })
    }

    fn proofread(&self, tier: PromptSetTier, name: &str, body: &str) -> i64 {
        self.create(tier, PromptSetKind::Proofread, name, body)
    }

    fn choose_global(&self, name: Option<&str>) {
        self.global(|g| {
            prompt_set_proofread_choice_set_global(Some(g), name).expect("global choice");
        });
    }

    fn choose_work(&self, name: Option<&str>, off: bool) {
        self.with_open(|open, _| {
            prompt_set_proofread_choice_set_work(Some(open), name, off).expect("work choice");
        });
    }

    fn scan(&self) -> Scan {
        let events = Arc::new(Mutex::new(Vec::<Value>::new()));
        let sink = Arc::clone(&events);
        let channel = tauri::ipc::Channel::new(move |body| {
            if let tauri::ipc::InvokeResponseBody::Json(json) = body {
                sink.lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner)
                    .push(serde_json::from_str(&json).expect("event json"));
            }
            Ok(())
        });
        let result = tauri::async_runtime::block_on(wire::ai_proofread_segments(
            self.app.handle().clone(),
            self.ids.clone(),
            channel,
        ))
        .map(|outcome| serde_json::to_value(outcome).expect("outcome json"));
        let events = events
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clone();
        Scan { result, events }
    }

    fn record(&self, scan_id: u64) -> Result<(), IpcError> {
        let work_id = self.with_open(|open, _| open.meta.work_id.clone());
        wire::ai_proofread_scan_record(
            self.app.handle().clone(),
            scan_id,
            work_id,
            "chapter".to_owned(),
            0,
            0,
            Vec::new(),
        )
    }

    fn trace_rows(&self) -> Vec<(Option<String>, Option<String>)> {
        self.with_open(|open, _| {
            open.store
                .read(|c| {
                    let mut stmt = c.prepare(
                        "SELECT instruction_set_name, instruction_set_tier FROM proof_scan ORDER BY id",
                    )?;
                    stmt.query_map([], |r| Ok((r.get(0)?, r.get(1)?)))?
                        .collect::<Result<Vec<_>, _>>()
                })
                .expect("proof_scan rows")
        })
    }

    fn prompts(&self) -> Vec<String> {
        self.server
            .bodies
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .iter()
            .map(|b| {
                b["messages"]
                    .as_array()
                    .and_then(|m| m.iter().find(|m| m["role"] == "user"))
                    .and_then(|m| m["content"].as_str())
                    .unwrap_or_default()
                    .to_owned()
            })
            .collect()
    }

    fn scan_id(scan: &Scan) -> u64 {
        scan.result.as_ref().expect("scan ok")["scan_id"]
            .as_u64()
            .expect("scan_id")
    }
}

const FORMAT_LINE_START: &str = "Reply with only a JSON array of";

#[test]
fn no_choice_sends_the_prompt_without_an_instruction_and_records_a_null_trace() {
    let h = Harness::new("none", "Source", "Target", 2, 1);
    h.proofread(PromptSetTier::Global, "B", "Never sent");
    let scan = h.scan();
    let prompts = h.prompts();
    assert_eq!(prompts.len(), 1);

    let blocks = [
        ProofreadBlock {
            source_text: "Source 0",
            target_text: "Target 0",
        },
        ProofreadBlock {
            source_text: "Source 1",
            target_text: "Target 1",
        },
    ];
    assert_eq!(prompts[0], assemble_proofread_batch_prompt(&blocks, None));
    h.record(Harness::scan_id(&scan)).expect("record");
    assert_eq!(h.trace_rows(), vec![(None, None)]);
}

#[test]
fn a_global_choice_reaches_the_request_before_the_reply_contract_and_is_traced() {
    let h = Harness::new("global", "Source", "Target", 1, 1);
    h.proofread(
        PromptSetTier::Global,
        "B",
        "Only compare with the source block.",
    );
    h.choose_global(Some("B"));
    let scan = h.scan();

    let prompt = &h.prompts()[0];
    let at = prompt
        .find("Only compare with the source block.")
        .expect("instruction in prompt");
    assert!(
        at < prompt.find(FORMAT_LINE_START).expect("format"),
        "{prompt}"
    );
    let request = h.server.bodies.lock().unwrap()[0].clone();
    let messages = request["messages"].as_array().expect("messages");
    assert!(
        messages.iter().all(|m| m["role"] == "user"),
        "no system message"
    );

    h.record(Harness::scan_id(&scan)).expect("record");
    assert_eq!(
        h.trace_rows(),
        vec![(Some("B".to_owned()), Some("global".to_owned()))]
    );
}

#[test]
fn the_work_choice_beats_the_global_one_and_the_trace_names_the_tier_the_set_resolved_to() {
    let h = Harness::new("work", "Source", "Target", 1, 3);
    h.proofread(PromptSetTier::Global, "B", "global instruction");
    h.proofread(PromptSetTier::Work, "A", "work instruction");
    h.proofread(PromptSetTier::Global, "OnlyGlobal", "from the global tier");
    h.choose_global(Some("B"));

    h.choose_work(Some("A"), false);
    let first = h.scan();
    h.record(Harness::scan_id(&first)).expect("record");

    h.choose_work(Some("OnlyGlobal"), false);
    let second = h.scan();
    h.record(Harness::scan_id(&second)).expect("record");

    h.choose_work(None, true);
    let third = h.scan();
    h.record(Harness::scan_id(&third)).expect("record");

    let prompts = h.prompts();
    assert!(prompts[0].contains("work instruction") && !prompts[0].contains("global instruction"));
    assert!(prompts[1].contains("from the global tier"));
    assert!(!prompts[2].contains("global instruction"));
    assert!(!prompts[2].contains("work instruction"));
    assert_eq!(
        h.trace_rows(),
        vec![
            (Some("A".to_owned()), Some("work".to_owned())),
            (Some("OnlyGlobal".to_owned()), Some("global".to_owned())),
            (None, None),
        ]
    );
}

#[test]
fn a_chosen_name_that_is_gone_stops_the_scan_before_any_network_call() {
    let h = Harness::new("missing", "Source", "Target", 1, 1);
    h.choose_work(Some("X"), false);
    let scan = h.scan();
    let err = scan.result.expect_err("missing set");
    assert_eq!(err.code(), "ai_proofread.instruction_missing");
    assert_eq!(err.params().get("name").map(String::as_str), Some("X"));
    assert_eq!(err.params().get("tier").map(String::as_str), Some("work"));
    assert!(!err.retryable());
    assert_eq!(h.server.accepted.load(Ordering::SeqCst), 0);
    assert!(scan.events.is_empty());
    assert!(h.trace_rows().is_empty());

    h.choose_work(None, false);
    h.choose_global(Some("G"));
    let err = h.scan().result.expect_err("missing global set");
    assert_eq!(err.params().get("tier").map(String::as_str), Some("global"));
    assert_eq!(h.server.accepted.load(Ordering::SeqCst), 0);
}

#[test]
fn an_empty_scan_reports_a_missing_set_before_it_ends_and_traces_the_chosen_one() {
    let mut h = Harness::new("empty-scan", "Source", "Target", 1, 1);
    h.ids = Vec::new();
    h.choose_work(Some("X"), false);
    let err = h.scan().result.expect_err("missing set");
    assert_eq!(err.code(), "ai_proofread.instruction_missing");

    h.proofread(PromptSetTier::Global, "G", "global instruction");
    h.choose_work(Some("G"), false);
    let scan = h.scan();
    let id = Harness::scan_id(&scan);
    h.record(id).expect("record");
    assert_eq!(
        h.trace_rows(),
        vec![(Some("G".to_owned()), Some("global".to_owned()))]
    );
}

#[test]
fn a_translate_set_with_the_same_name_is_not_what_the_scan_sends() {
    let h = Harness::new("kinds", "Source", "Target", 1, 1);
    h.create(
        PromptSetTier::Global,
        PromptSetKind::Translate,
        "A",
        "translate body {{source_segment}}",
    );
    h.proofread(PromptSetTier::Global, "A", "proofread body");
    h.choose_global(Some("A"));
    h.scan();
    let prompt = &h.prompts()[0];
    assert!(prompt.contains("proofread body"), "{prompt}");
    assert!(!prompt.contains("translate body"), "{prompt}");
}

#[test]
fn a_marker_in_a_proofread_body_is_sent_as_typed() {
    let h = Harness::new("marker", "Source", "Target", 1, 1);
    h.proofread(
        PromptSetTier::Global,
        "M",
        "Use {{source_segment}} and {{glossary}} as written.",
    );
    h.choose_global(Some("M"));
    h.scan();
    let prompt = &h.prompts()[0];
    assert!(
        prompt.contains("Use {{source_segment}} and {{glossary}} as written."),
        "{prompt}"
    );
}

#[test]
fn a_reply_in_prose_leaves_the_batch_unscanned_and_never_clean() {
    let source = "s".repeat(1_400);
    let target = "t".repeat(1_400);
    let h = Harness::with_reply("prose", &source, &target, 3, 3, |index| {
        if index == 0 {
            "Everything looks fine to me, no problems at all.".to_owned()
        } else {
            "[]".to_owned()
        }
    });
    h.proofread(PromptSetTier::Global, "P", "Be gentle.");
    h.choose_global(Some("P"));
    let scan = h.scan();
    assert_eq!(h.prompts().len(), 3);
    let outcome = scan.result.expect("scan finishes");
    assert_eq!(outcome["state"], "done");
    assert_eq!(outcome["not_scanned"], 1);
    let batches: Vec<usize> = scan
        .events
        .iter()
        .filter(|e| e["kind"] == "batch")
        .map(|e| e["segments"].as_array().map_or(0, Vec::len))
        .collect();
    assert_eq!(
        batches,
        vec![0, 1, 1],
        "the prose batch returns no scanned segment"
    );
}

#[test]
fn when_every_reply_is_prose_the_scan_reports_the_malformed_reply_with_all_segments_unscanned() {
    let h = Harness::with_reply("prose-all", "Source", "Target", 3, 1, |_| {
        "Everything looks fine to me, no problems at all.".to_owned()
    });
    h.proofread(PromptSetTier::Global, "P", "Be gentle.");
    h.choose_global(Some("P"));
    let err = h.scan().result.expect_err("nothing parsed");
    assert_eq!(err.code(), "ai_proofread.reply_malformed");
    assert_eq!(
        err.params().get("not_scanned").map(String::as_str),
        Some("3")
    );
}

#[test]
fn a_long_instruction_rides_every_batch_and_every_batch_usage_reaches_the_run() {
    let long = "Compare with the source only. ".repeat(200);
    let source = "s".repeat(1_400);
    let target = "t".repeat(1_400);
    let h = Harness::new("batches", &source, &target, 3, 3);
    h.proofread(PromptSetTier::Global, "Long", &long);
    h.choose_global(Some("Long"));
    let scan = h.scan();

    let prompts = h.prompts();
    assert!(prompts.len() >= 2, "{} batches", prompts.len());
    for prompt in &prompts {
        assert!(
            prompt.contains(long.trim()),
            "every request carries the instruction"
        );
    }
    let planned = scan
        .events
        .iter()
        .find(|e| e["kind"] == "planned")
        .expect("planned");
    assert_eq!(planned["batches"], prompts.len());
    let usage_total: u64 = scan
        .events
        .iter()
        .filter(|e| e["kind"] == "batch")
        .map(|e| {
            e["usage"]["total_tokens"]
                .as_u64()
                .expect("usage per batch")
        })
        .sum();
    assert_eq!(usage_total, 18 * prompts.len() as u64);
}

#[test]
fn renaming_the_set_the_open_work_chose_makes_the_next_scan_use_it_under_the_new_name() {
    let h = Harness::new("rename", "Source", "Target", 1, 1);
    let id = h.proofread(PromptSetTier::Work, "A", "renamed body");
    h.choose_work(Some("A"), false);
    h.with_open(|open, global| {
        prompt_set_rename(
            Some(global),
            Some(open),
            PromptSetTier::Work,
            PromptSetKind::Proofread,
            id,
            "C",
        )
        .expect("rename");
    });

    let scan = h.scan();
    assert!(h.prompts()[0].contains("renamed body"));
    h.record(Harness::scan_id(&scan)).expect("record");
    assert_eq!(
        h.trace_rows(),
        vec![(Some("C".to_owned()), Some("work".to_owned()))]
    );
}

#[test]
fn a_scan_trace_is_taken_once_and_only_for_the_latest_scan() {
    let h = Harness::new("trace-codes", "Source", "Target", 1, 2);
    h.proofread(PromptSetTier::Global, "B", "instruction");
    h.choose_global(Some("B"));

    let first = Harness::scan_id(&h.scan());
    let second = Harness::scan_id(&h.scan());
    assert_ne!(first, second);

    for rejected in [first, second + 100, 0] {
        let err = h.record(rejected).expect_err("rejected");
        assert_eq!(
            err.code(),
            "ai_proofread.scan_record_rejected",
            "code {rejected}"
        );
    }
    assert!(h.trace_rows().is_empty());

    h.record(second).expect("latest scan records");
    assert_eq!(h.trace_rows().len(), 1);
    let again = h.record(second).expect_err("a code is used once");
    assert_eq!(again.code(), "ai_proofread.scan_record_rejected");
    assert_eq!(h.trace_rows().len(), 1);
}
