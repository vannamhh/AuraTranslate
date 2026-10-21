//! Update check (FR111, AD-15 exit 2): every row of the I/O matrix against a local fake HTTP
//! server, plus a source scan that nothing under the update code writes a file or starts a process.

use std::io::{Read, Write};
use std::net::TcpListener;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::time::Duration;

use auratranslate_lib::commands::config::put_config;
use auratranslate_lib::commands::update::{UpdateCheckWire, open_release_page, run_update_check};
use auratranslate_lib::core::i18n::MessageKey;
use auratranslate_lib::core::store::{Store, StoreSpec, Tuning};
use auratranslate_lib::core::update::{self, UpdateError, Version};

#[path = "support/boundary_scan.rs"]
#[allow(dead_code)] // shared module: not every helper is used in this file
mod boundary_scan;
use boundary_scan::{code_lines, rust_sources, src_root, without_test_modules};

static NEXT_DIR: AtomicU64 = AtomicU64::new(0);

fn temp_dir() -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "auratranslate-update-{}-{}",
        std::process::id(),
        NEXT_DIR.fetch_add(1, Ordering::Relaxed)
    ));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("temp dir");
    dir
}

fn open_store(dir: &std::path::Path) -> Store {
    Store::open(StoreSpec {
        tuning: Tuning {
            checkpoint_tick: Duration::from_millis(50),
            idle_before_passive: Duration::from_secs(3600),
            wal_threshold_bytes: u64::MAX,
            ..Tuning::default()
        },
        ..StoreSpec::global(dir.join("global.db"))
    })
    .expect("open global store")
}

struct Fake {
    base: String,
    hits: Arc<AtomicUsize>,
}

fn serve(status: &'static str, body: String, delay: Duration) -> Fake {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind");
    let base = format!("http://127.0.0.1:{}", listener.local_addr().unwrap().port());
    let hits = Arc::new(AtomicUsize::new(0));
    let counter = hits.clone();
    std::thread::spawn(move || {
        for stream in listener.incoming() {
            let Ok(mut stream) = stream else { return };
            counter.fetch_add(1, Ordering::SeqCst);
            let mut buf = [0u8; 4096];
            let _ = stream.read(&mut buf);
            std::thread::sleep(delay);
            let reply = format!(
                "HTTP/1.1 {status}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
                body.len()
            );
            let _ = stream.write_all(reply.as_bytes());
        }
    });
    Fake { base, hits }
}

fn releases(tags: &[(&str, bool, bool)]) -> String {
    let items: Vec<String> = tags
        .iter()
        .map(|(tag, draft, pre)| {
            format!(r#"{{"tag_name":"{tag}","draft":{draft},"prerelease":{pre},"html_url":"https://evil.example/x"}}"#)
        })
        .collect();
    format!("[{}]", items.join(","))
}

fn check(fake: &Fake) -> Result<UpdateCheckWire, auratranslate_lib::core::i18n::IpcError> {
    let dir = temp_dir();
    let store = open_store(&dir);
    let out = run_update_check(Some(&store), &fake.base, "0.1.0");
    drop(store);
    let _ = std::fs::remove_dir_all(&dir);
    out
}

#[test]
fn a_higher_app_tag_is_reported_with_the_numeric_not_lexical_maximum() {
    let fake = serve(
        "200 OK",
        releases(&[
            ("v0.2.0", false, false),
            ("v0.10.0", false, false),
            ("v0.9.0", false, false),
        ]),
        Duration::ZERO,
    );
    assert_eq!(
        check(&fake).unwrap(),
        UpdateCheckWire::UpdateAvailable {
            latest: "0.10.0".to_owned()
        }
    );
}

#[test]
fn an_equal_or_lower_tag_is_up_to_date() {
    for tag in ["v0.1.0", "v0.0.9"] {
        let fake = serve("200 OK", releases(&[(tag, false, false)]), Duration::ZERO);
        assert_eq!(
            check(&fake).unwrap(),
            UpdateCheckWire::UpToDate {
                current: "0.1.0".to_owned()
            },
            "{tag}"
        );
    }
}

#[test]
fn data_tags_only_means_no_app_release_never_up_to_date() {
    let fake = serve(
        "200 OK",
        releases(&[
            ("dict-v2", false, false),
            ("dict-raw-v1", false, false),
            ("dict-v1", false, false),
        ]),
        Duration::ZERO,
    );
    assert_eq!(check(&fake).unwrap(), UpdateCheckWire::NoAppRelease);
}

#[test]
fn drafts_prereleases_and_malformed_tags_are_skipped() {
    let fake = serve(
        "200 OK",
        releases(&[
            ("v9.0.0", true, false),
            ("v8.0.0", false, true),
            ("v7.1", false, false),
            ("v6.0.0-rc1", false, false),
            ("6.0.0", false, false),
            ("v0.3.0", false, false),
        ]),
        Duration::ZERO,
    );
    assert_eq!(
        check(&fake).unwrap(),
        UpdateCheckWire::UpdateAvailable {
            latest: "0.3.0".to_owned()
        }
    );
    assert_eq!(
        update::highest_app_version(releases(&[("v9.0.0", true, false)]).as_bytes()),
        Ok(None)
    );
}

#[test]
fn the_switch_off_refuses_the_command_and_opens_zero_connections() {
    let fake = serve(
        "200 OK",
        releases(&[("v0.2.0", false, false)]),
        Duration::ZERO,
    );
    let dir = temp_dir();
    let store = open_store(&dir);
    put_config(Some(&store), "app_config", "update_check_enabled", "off").expect("write switch");
    let err = run_update_check(Some(&store), &fake.base, "0.1.0").unwrap_err();
    assert_eq!(err.code(), "update.disabled");
    assert_eq!(err.message_key(), MessageKey::UpdateDisabled);
    std::thread::sleep(Duration::from_millis(200));
    assert_eq!(fake.hits.load(Ordering::SeqCst), 0);

    put_config(Some(&store), "app_config", "update_check_enabled", "").expect("clear switch");
    assert!(run_update_check(Some(&store), &fake.base, "0.1.0").is_ok());
    assert_eq!(
        fake.hits.load(Ordering::SeqCst),
        1,
        "empty value reads as on"
    );
    drop(store);
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn rate_limit_statuses_map_to_their_own_key() {
    for status in ["403 Forbidden", "429 Too Many Requests"] {
        let fake = serve(status, "{}".to_owned(), Duration::ZERO);
        let err = check(&fake).unwrap_err();
        assert_eq!(err.message_key(), MessageKey::UpdateRateLimited, "{status}");
    }
}

#[test]
fn a_broken_body_or_other_status_is_a_bad_response() {
    for (status, body) in [
        ("200 OK", "not json"),
        ("200 OK", "{}"),
        ("500 Internal Server Error", "[]"),
    ] {
        let fake = serve(status, body.to_owned(), Duration::ZERO);
        let err = check(&fake).unwrap_err();
        assert_eq!(
            err.message_key(),
            MessageKey::UpdateBadResponse,
            "{status} {body}"
        );
    }
}

#[test]
fn a_refused_connection_maps_to_connect_failed() {
    let port = {
        let l = TcpListener::bind("127.0.0.1:0").unwrap();
        l.local_addr().unwrap().port()
    };
    assert_eq!(
        update::check(&format!("http://127.0.0.1:{port}"), Version(0, 1, 0)),
        Err(UpdateError::Connect)
    );
}

#[test]
fn a_host_that_does_not_answer_in_time_maps_to_timeout() {
    let fake = serve(
        "200 OK",
        releases(&[("v0.2.0", false, false)]),
        Duration::from_secs(3),
    );
    assert_eq!(
        update::check_within(&fake.base, Version(0, 1, 0), Duration::from_millis(300)),
        Err(UpdateError::Timeout)
    );
    assert_eq!(fake.hits.load(Ordering::SeqCst), 1);
}

#[test]
fn a_body_over_the_cap_is_refused_even_when_it_is_valid_json() {
    let body = format!("[{}]", " ".repeat(4 * 1024 * 1024 + 1));
    let fake = serve("200 OK", body, Duration::ZERO);
    assert_eq!(
        update::check(&fake.base, Version(0, 1, 0)),
        Err(UpdateError::BadResponse)
    );
}

#[test]
fn a_missing_store_is_a_store_error_not_a_connection() {
    let fake = serve("200 OK", "[]".to_owned(), Duration::ZERO);
    let err = run_update_check(None, &fake.base, "0.1.0").unwrap_err();
    assert_eq!(err.code(), "store.open_failed");
    assert_eq!(fake.hits.load(Ordering::SeqCst), 0);
}

#[test]
fn the_release_page_is_built_from_the_parsed_version_and_bad_patterns_open_nothing() {
    let opened = std::cell::RefCell::new(Vec::<String>::new());
    let open = |url: &str| {
        opened.borrow_mut().push(url.to_owned());
        Ok(())
    };
    open_release_page("0.2.0", &open).unwrap();
    assert_eq!(
        opened.borrow().as_slice(),
        ["https://github.com/vannamhh/AuraTranslate/releases/tag/v0.2.0"]
    );
    for bad in [
        "../x",
        "1.2",
        "v1.2.3",
        "1.2.3.4",
        "1.2.x",
        "",
        "1.2.3/../../x",
        " 1.2.3",
    ] {
        let err = open_release_page(bad, &open).unwrap_err();
        assert_eq!(
            err.message_key(),
            MessageKey::UpdateInvalidVersion,
            "{bad:?}"
        );
    }
    assert_eq!(opened.borrow().len(), 1, "bad patterns opened nothing");
}

#[test]
fn version_ordering_is_numeric() {
    assert!(Version::parse_tag("v0.10.0") > Version::parse_tag("v0.9.9"));
    assert_eq!(Version::parse_tag("v01.2.3"), Some(Version(1, 2, 3)));
    assert_eq!(Version::parse_tag("v1.2.3-rc1"), None);
}

const FORBIDDEN: [&str; 9] = [
    "std::fs",
    "tokio::fs",
    "std::process",
    "open_path",
    "File::create",
    "Command::new",
    "include_bytes",
    "tokio::process",
    "OpenOptions",
];

fn update_sources() -> Vec<(String, String)> {
    rust_sources(&src_root())
        .into_iter()
        .filter(|(rel, _)| rel.starts_with("core/update/") || rel == "commands/update.rs")
        .collect()
}

fn forbidden_hits(sources: &[(String, String)]) -> Vec<String> {
    let mut hits = Vec::new();
    for (rel, text) in sources {
        for (line, code) in code_lines(&without_test_modules(text)) {
            if FORBIDDEN.iter().any(|t| code.contains(t)) {
                hits.push(format!("{rel}:{line}  {code}"));
            }
        }
    }
    hits
}

#[test]
fn update_code_never_writes_a_file_or_starts_a_process() {
    let sources = update_sources();
    assert_eq!(
        sources.len(),
        2,
        "scan population: core/update/mod.rs + commands/update.rs"
    );
    let lines: usize = sources.iter().map(|(_, t)| code_lines(t).count()).sum();
    assert!(lines > 50, "scan must read real code lines, read {lines}");
    assert_eq!(forbidden_hits(&sources), Vec::<String>::new());

    let planted = vec![(
        "core/update/mod.rs".to_owned(),
        "fn x() {\n    std::fs::write(\"a\", b\"b\").ok();\n}\n".to_owned(),
    )];
    assert_eq!(
        forbidden_hits(&planted).len(),
        1,
        "positive case: a real write is caught"
    );
    let commented = vec![(
        "core/update/mod.rs".to_owned(),
        "// std::fs::write(\"a\", b\"b\");\nfn x() {}\n".to_owned(),
    )];
    assert!(
        forbidden_hits(&commented).is_empty(),
        "negative case: a comment is not code"
    );
}
