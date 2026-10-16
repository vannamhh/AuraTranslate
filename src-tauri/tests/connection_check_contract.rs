//! Connection check (FR143) and shortcut-file exchange (FR144). The failure families are driven
//! twice: through a scripted provider, and through the real `OpenAiChatClient` against a local
//! fake HTTP server.

use auratranslate_lib::commands::aitranslate::{
    ConnectionCheckWire, PreparedConnectionCheck, run_connection_check,
};
use auratranslate_lib::commands::shortcuts::{shortcuts_read_file, shortcuts_write_file};
use auratranslate_lib::core::ai::client::{
    OpenAiClientError, chat_completions_url, check_model_list, models_url,
};
use auratranslate_lib::core::i18n::MessageKey;
use auratranslate_lib::ports::translation_provider::{
    ConnectionCheck, ConnectionCheckOutcome, ConnectionCheckRequest,
};

struct Scripted(Result<ConnectionCheckOutcome, OpenAiClientError>);

impl ConnectionCheck for Scripted {
    type Error = OpenAiClientError;

    async fn check_connection(
        &self,
        _request: ConnectionCheckRequest<'_>,
        _should_cancel: &dyn Fn() -> bool,
    ) -> Result<ConnectionCheckOutcome, OpenAiClientError> {
        self.0.clone()
    }
}

fn prepared() -> PreparedConnectionCheck {
    PreparedConnectionCheck {
        endpoint: "http://localhost:11434/v1".to_owned(),
        model: "qwen2.5:14b".to_owned(),
        api_key: None,
    }
}

fn run(
    result: Result<ConnectionCheckOutcome, OpenAiClientError>,
) -> Result<ConnectionCheckWire, auratranslate_lib::core::i18n::IpcError> {
    tauri::async_runtime::block_on(run_connection_check(
        &Scripted(result),
        &prepared(),
        &|| false,
    ))
}

#[test]
fn chat_url_keeps_a_saved_full_url_and_completes_a_base_url() {
    for saved in [
        "http://h/v1",
        "http://h/v1/",
        "http://h/v1/chat/completions",
        "http://h/v1/chat/completions/",
    ] {
        assert_eq!(
            chat_completions_url(saved),
            "http://h/v1/chat/completions",
            "{saved}"
        );
        assert_eq!(models_url(saved), "http://h/v1/models", "{saved}");
    }
}

#[test]
fn starting_or_cancelling_a_check_never_touches_the_translate_generation() {
    use auratranslate_lib::commands::aiconfig::wire;
    use auratranslate_lib::commands::aitranslate::{
        AiTranslateGeneration, ConnectionCheckGeneration,
    };
    use auratranslate_lib::core::aiconfig::AiConfigTier;
    use tauri::Manager as _;
    use tauri::test::{mock_builder, mock_context, noop_assets};

    let app = mock_builder()
        .build(mock_context(noop_assets()))
        .expect("app");
    app.manage(AiTranslateGeneration::default());
    app.manage(ConnectionCheckGeneration::default());
    let handle = app.handle().clone();

    let _ = tauri::async_runtime::block_on(wire::ai_config_test_connection(
        handle.clone(),
        AiConfigTier::Global,
    ));
    let translate = app.state::<AiTranslateGeneration>();
    let check = app.state::<ConnectionCheckGeneration>();
    assert!(
        translate.is_current(0),
        "translate generation bumped by a check"
    );
    assert!(check.is_current(1), "check generation not bumped");

    wire::ai_config_test_connection_cancel(handle);
    assert!(
        translate.is_current(0),
        "cancel of a check bumped translate"
    );
    assert!(check.is_current(2));
}

#[test]
fn models_url_appends_models_to_a_base_or_a_full_chat_url() {
    assert_eq!(models_url("http://h/v1"), "http://h/v1/models");
    assert_eq!(models_url("http://h/v1/"), "http://h/v1/models");
    assert_eq!(
        models_url("https://api.openai.com/v1/chat/completions"),
        "https://api.openai.com/v1/models"
    );
}

#[test]
fn the_model_list_decides_connected_or_model_missing() {
    let body = br#"{"data":[{"id":"qwen2.5:14b"},{"id":"llama3:latest"}]}"#;
    assert_eq!(
        check_model_list(body, "qwen2.5:14b").unwrap(),
        ConnectionCheckOutcome::Connected { model_count: 2 }
    );
    assert_eq!(
        check_model_list(body, "llama3").unwrap(),
        ConnectionCheckOutcome::Connected { model_count: 2 }
    );
    assert_eq!(
        check_model_list(body, "gpt-4o").unwrap(),
        ConnectionCheckOutcome::ModelMissing
    );
    assert!(check_model_list(b"<html>", "x").is_err());
    assert!(check_model_list(br#"{"models":[]}"#, "x").is_err());
}

#[test]
fn every_failure_family_has_its_own_message_key() {
    let key_of = |r| run(r).expect_err("loi").message_key();
    assert_eq!(
        key_of(Err(OpenAiClientError::NonSuccessStatus { status: 401 })),
        MessageKey::AiConfigTestKeyRejected
    );
    assert_eq!(
        key_of(Err(OpenAiClientError::NonSuccessStatus { status: 403 })),
        MessageKey::AiConfigTestKeyRejected
    );
    assert_eq!(
        key_of(Err(OpenAiClientError::NonSuccessStatus { status: 404 })),
        MessageKey::AiConfigTestEndpointNotFound
    );
    assert_eq!(
        key_of(Err(OpenAiClientError::NonSuccessStatus { status: 500 })),
        MessageKey::AiConfigTestRefused
    );
    assert_eq!(
        key_of(Err(OpenAiClientError::RequestFailed {
            detail: "x".to_owned()
        })),
        MessageKey::AiConfigTestUnreachable
    );
    assert_eq!(
        key_of(Err(OpenAiClientError::MalformedEvent {
            detail: "x".to_owned()
        })),
        MessageKey::AiConfigTestReplyUnreadable
    );
    assert_eq!(
        key_of(Ok(ConnectionCheckOutcome::ModelMissing)),
        MessageKey::AiConfigTestModelMissing
    );
}

#[test]
fn connected_and_cancelled_are_outcomes_not_errors() {
    assert_eq!(
        run(Ok(ConnectionCheckOutcome::Connected { model_count: 3 })).unwrap(),
        ConnectionCheckWire::Connected { model_count: 3 }
    );
    assert_eq!(
        run(Ok(ConnectionCheckOutcome::Cancelled)).unwrap(),
        ConnectionCheckWire::Cancelled
    );
}

#[test]
fn a_shortcut_file_round_trips_and_a_bad_file_is_refused() {
    let dir = std::env::temp_dir().join(format!("aura-shortcuts-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("set.json");

    shortcuts_write_file(&path, "{\"version\":1}").unwrap();
    assert_eq!(shortcuts_read_file(&path).unwrap(), "{\"version\":1}");

    std::fs::write(&path, vec![b'a'; 1024 * 1024 + 1]).unwrap();
    assert_eq!(
        shortcuts_read_file(&path).unwrap_err().message_key(),
        MessageKey::ShortcutsFileTooLarge
    );
    std::fs::write(&path, [0xff, 0xfe, 0x00]).unwrap();
    assert_eq!(
        shortcuts_read_file(&path).unwrap_err().message_key(),
        MessageKey::ShortcutsFileUnreadable
    );
    assert_eq!(
        shortcuts_read_file(&dir.join("missing.json"))
            .unwrap_err()
            .message_key(),
        MessageKey::ShortcutsFileUnreadable
    );
    std::fs::remove_dir_all(&dir).unwrap();
}

mod fake_server {
    use std::io::{Read, Write};
    use std::net::TcpListener;
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::{Arc, Mutex};
    use std::thread::JoinHandle;
    use std::time::Duration;

    pub type Responder = Box<dyn Fn(&mut std::net::TcpStream) + Send>;

    pub struct FakeServer {
        pub base_url: String,
        requests: Arc<Mutex<Vec<String>>>,
        stop: Arc<AtomicBool>,
        thread: Option<JoinHandle<()>>,
    }

    pub fn reply(status_line: &str, body: &str) -> Responder {
        let raw = format!(
            "HTTP/1.1 {status_line}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
            body.len()
        );
        Box::new(move |s| {
            let _ = s.write_all(raw.as_bytes());
        })
    }

    impl FakeServer {
        pub fn start(responder: Responder) -> Self {
            let listener = TcpListener::bind("127.0.0.1:0").expect("bind");
            listener.set_nonblocking(true).expect("nonblocking");
            let port = listener.local_addr().expect("addr").port();
            let requests = Arc::new(Mutex::new(Vec::new()));
            let stop = Arc::new(AtomicBool::new(false));
            let (sink, flag) = (Arc::clone(&requests), Arc::clone(&stop));
            let thread = std::thread::spawn(move || {
                while !flag.load(Ordering::SeqCst) {
                    let Ok((mut stream, _)) = listener.accept() else {
                        std::thread::sleep(Duration::from_millis(5));
                        continue;
                    };
                    stream.set_nonblocking(false).expect("blocking");
                    let mut head = Vec::new();
                    let mut chunk = [0u8; 2048];
                    while !head.windows(4).any(|w| w == b"\r\n\r\n") {
                        match stream.read(&mut chunk) {
                            Ok(0) | Err(_) => break,
                            Ok(n) => head.extend_from_slice(&chunk[..n]),
                        }
                    }
                    sink.lock()
                        .unwrap()
                        .push(String::from_utf8_lossy(&head).into_owned());
                    responder(&mut stream);
                    let _ = stream.flush();
                }
            });
            Self {
                base_url: format!("http://127.0.0.1:{port}/v1"),
                requests,
                stop,
                thread: Some(thread),
            }
        }

        pub fn finish(mut self) -> Vec<String> {
            self.stop.store(true, Ordering::SeqCst);
            if let Some(t) = self.thread.take() {
                t.join().expect("server thread");
            }
            self.requests.lock().unwrap().clone()
        }
    }
}

use fake_server::{FakeServer, reply};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

fn check_against(
    endpoint: &str,
    api_key: Option<&str>,
    should_cancel: &(dyn Fn() -> bool + Sync),
) -> Result<ConnectionCheckWire, auratranslate_lib::core::i18n::IpcError> {
    let prepared = PreparedConnectionCheck {
        endpoint: endpoint.to_owned(),
        model: "qwen2.5:14b".to_owned(),
        api_key: api_key.map(str::to_owned),
    };
    tauri::async_runtime::block_on(run_connection_check(
        &auratranslate_lib::core::ai::client::OpenAiChatClient::new(),
        &prepared,
        should_cancel,
    ))
}

const LISTED: &str = r#"{"data":[{"id":"qwen2.5:14b"},{"id":"llama3"}]}"#;

#[test]
fn http_wrong_key_is_401_key_rejected_sent_once_with_the_bearer_and_never_retried() {
    let server = FakeServer::start(reply("401 Unauthorized", "{}"));
    let err = check_against(&server.base_url, Some("sk-wrong"), &|| false).expect_err("401");
    assert_eq!(err.message_key(), MessageKey::AiConfigTestKeyRejected);
    let requests = server.finish();
    assert_eq!(requests.len(), 1, "an automatic retry reached the server");
    assert!(
        requests[0].starts_with("GET /v1/models "),
        "{}",
        requests[0]
    );
    assert!(
        requests[0]
            .to_lowercase()
            .contains("authorization: bearer sk-wrong"),
        "{}",
        requests[0]
    );
}

#[test]
fn http_403_is_also_key_rejected() {
    let server = FakeServer::start(reply("403 Forbidden", "{}"));
    let err = check_against(&server.base_url, Some("k"), &|| false).expect_err("403");
    assert_eq!(err.message_key(), MessageKey::AiConfigTestKeyRejected);
    server.finish();
}

#[test]
fn http_404_500_and_html_each_get_their_own_family() {
    let server = FakeServer::start(reply("404 Not Found", "{}"));
    let err = check_against(&server.base_url, None, &|| false).expect_err("404");
    assert_eq!(err.message_key(), MessageKey::AiConfigTestEndpointNotFound);
    server.finish();

    let server = FakeServer::start(reply("500 Internal Server Error", "{}"));
    let err = check_against(&server.base_url, None, &|| false).expect_err("500");
    assert_eq!(err.message_key(), MessageKey::AiConfigTestRefused);
    server.finish();

    let server = FakeServer::start(reply("200 OK", "<html>login</html>"));
    let err = check_against(&server.base_url, None, &|| false).expect_err("html");
    assert_eq!(err.message_key(), MessageKey::AiConfigTestReplyUnreadable);
    server.finish();
}

#[test]
fn http_listed_model_connects_and_an_unlisted_one_is_model_missing() {
    let server = FakeServer::start(reply("200 OK", LISTED));
    assert_eq!(
        check_against(&server.base_url, None, &|| false).unwrap(),
        ConnectionCheckWire::Connected { model_count: 2 }
    );
    let requests = server.finish();
    assert!(
        !requests[0].to_lowercase().contains("authorization:"),
        "no key saved, yet a header was sent: {}",
        requests[0]
    );

    let server = FakeServer::start(reply("200 OK", r#"{"data":[{"id":"other"}]}"#));
    let err = check_against(&server.base_url, None, &|| false).expect_err("missing");
    assert_eq!(err.message_key(), MessageKey::AiConfigTestModelMissing);
    server.finish();
}

#[test]
fn http_a_saved_full_chat_url_still_checks_the_models_path_of_the_same_server() {
    let server = FakeServer::start(reply("200 OK", LISTED));
    let saved = format!("{}/chat/completions", server.base_url);
    assert!(check_against(&saved, None, &|| false).is_ok());
    let requests = server.finish();
    assert!(
        requests[0].starts_with("GET /v1/models "),
        "{}",
        requests[0]
    );
}

#[test]
fn http_dead_network_is_unreachable_and_retryable() {
    let port = {
        let l = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        l.local_addr().unwrap().port()
    };
    let err = check_against(&format!("http://127.0.0.1:{port}/v1"), None, &|| false)
        .expect_err("refused");
    assert_eq!(err.message_key(), MessageKey::AiConfigTestUnreachable);
    assert!(err.retryable());
}

#[test]
fn http_cancel_before_sending_reaches_no_server() {
    let server = FakeServer::start(reply("200 OK", LISTED));
    assert_eq!(
        check_against(&server.base_url, None, &|| true).unwrap(),
        ConnectionCheckWire::Cancelled
    );
    assert!(server.finish().is_empty(), "a cancelled check hit the net");
}

#[test]
fn http_cancel_between_reply_chunks_gives_cancelled_not_connected() {
    let cancel = Arc::new(AtomicBool::new(false));
    let server_side = Arc::clone(&cancel);
    let server = FakeServer::start(Box::new(move |stream| {
        use std::io::Write as _;
        let body = LISTED;
        let (first, rest) = body.split_at(10);
        let head = format!(
            "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
            body.len()
        );
        let _ = stream.write_all(head.as_bytes());
        let _ = stream.write_all(first.as_bytes());
        let _ = stream.flush();
        std::thread::sleep(std::time::Duration::from_millis(150));
        server_side.store(true, Ordering::SeqCst);
        let _ = stream.write_all(rest.as_bytes());
    }));
    let flag = Arc::clone(&cancel);
    let outcome = check_against(&server.base_url, None, &move || flag.load(Ordering::SeqCst));
    server.finish();
    assert_eq!(outcome.unwrap(), ConnectionCheckWire::Cancelled);
}
