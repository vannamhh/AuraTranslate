//! "Not an error" memory of the proofreader (FR84, AD-53 rule 7): signature, survival across
//! merging and splitting, the scan filter, the list/remove/clear commands and the scan count.

use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

use auratranslate_lib::commands::aiconfig::{ai_config_delete_key, ai_config_save_key};
use auratranslate_lib::commands::aitranslate::PreparedTranslateCall;
use auratranslate_lib::commands::project::{OpenWork, create_work_from_text};
use auratranslate_lib::commands::proofread::{
    AiProofreadEventWire, AiProofreadOutcomeWire, PreparedProofreadBatch, PreparedProofreadRun,
    PreparedProofreadSegment, ProofreadPrepareOutcome, prepare_proofread_call,
    proofread_ignore_add, proofread_ignore_clear, proofread_ignore_list, proofread_ignore_remove,
    proofread_scan_record, run_proofread_call,
};
use auratranslate_lib::commands::segment::{
    merge_segments, promote_ai_translation, read_open_chapter_segments, split_segment,
};
use auratranslate_lib::core::aiconfig::{AiConfigField, AiConfigTier, write_field};
use auratranslate_lib::core::i18n::IpcError;
use auratranslate_lib::core::segment::proofread_kind::ProofreadKind;
use auratranslate_lib::core::store::{Store, StoreSpec};
use auratranslate_lib::ports::translation_provider::{
    TranslateOutcome, TranslateRequest, TranslationProvider,
};

static NEXT_DIR: AtomicU64 = AtomicU64::new(0);
static KEYCHAIN_TEST_LOCK: Mutex<()> = Mutex::new(());

fn temp_dir(tag: &str) -> PathBuf {
    let n = NEXT_DIR.fetch_add(1, Ordering::Relaxed);
    let dir = std::env::temp_dir().join(format!(
        "auratranslate-proof-memory-{}-{tag}-{n}",
        std::process::id()
    ));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).expect("create temp dir");
    dir
}

fn work_of(tag: &str, count: usize) -> OpenWork {
    let text = (0..count)
        .map(|i| format!("Sentence {i}."))
        .collect::<Vec<_>>()
        .join("\n\n");
    create_work_from_text(&temp_dir(tag), "Memory", "en", "", text).expect("create work")
}

fn ids(open: &OpenWork) -> Vec<i64> {
    read_open_chapter_segments(Some(open))
        .expect("chapter")
        .segments
        .iter()
        .map(|s| s.id)
        .collect()
}

fn set_target(open: &OpenWork, id: i64, text: &str) {
    promote_ai_translation(Some(open), id, text, false).expect("promote");
}

fn live_text(open: &OpenWork, id: i64) -> String {
    read_open_chapter_segments(Some(open))
        .expect("chapter")
        .segments
        .into_iter()
        .find(|s| s.id == id)
        .expect("segment")
        .target_text
}

fn utf16_range(text: &str, phrase: &str) -> (u32, u32) {
    let byte = text.find(phrase).expect("phrase in text");
    let start = text[..byte].encode_utf16().count();
    (
        u32::try_from(start).unwrap(),
        u32::try_from(start + phrase.encode_utf16().count()).unwrap(),
    )
}

fn remember(
    open: &OpenWork,
    id: i64,
    text: &str,
    phrase: &str,
    kind: ProofreadKind,
) -> Result<(), IpcError> {
    let (start, end) = utf16_range(text, phrase);
    proofread_ignore_add(Some(open), id, text, start, end, phrase, kind)
}

fn table_count(open: &OpenWork, table: &'static str) -> i64 {
    open.store
        .read(move |conn| {
            conn.query_row(&format!("SELECT COUNT(*) FROM {table}"), [], |r| r.get(0))
        })
        .expect("count")
}

fn columns(open: &OpenWork, table: &'static str) -> Vec<String> {
    open.store
        .read(move |conn| {
            let mut stmt = conn.prepare(&format!(
                "SELECT name FROM pragma_table_info('{table}') ORDER BY cid"
            ))?;
            stmt.query_map([], |r| r.get(0))?
                .collect::<Result<Vec<String>, _>>()
        })
        .expect("columns")
}

#[derive(Debug, Clone)]
struct FakeError;
impl std::fmt::Display for FakeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "fake")
    }
}
impl std::error::Error for FakeError {}

struct Replying(String);

impl TranslationProvider for Replying {
    type Error = FakeError;

    async fn translate(
        &self,
        _request: TranslateRequest<'_>,
        on_token: &mut dyn FnMut(&str),
        _should_cancel: &dyn Fn() -> bool,
    ) -> Result<TranslateOutcome, FakeError> {
        on_token(&self.0);
        Ok(TranslateOutcome::Done(None))
    }
}

struct Cancelling;

impl TranslationProvider for Cancelling {
    type Error = FakeError;

    async fn translate(
        &self,
        _request: TranslateRequest<'_>,
        _on_token: &mut dyn FnMut(&str),
        _should_cancel: &dyn Fn() -> bool,
    ) -> Result<TranslateOutcome, FakeError> {
        Ok(TranslateOutcome::Cancelled)
    }
}

fn reply_of(kind: &str, quote: &str) -> String {
    format!(r#"[{{"kind":"{kind}","quote":"{quote}","explanation":"e","suggestion":"s"}}]"#)
}

/// Runs the real prepare layer's memory read and the real filter, over one segment holding `text`.
fn scan(open: &OpenWork, id: i64, text: &str, kind: &str, quote: &str) -> (usize, u32) {
    let remembered = {
        let mut run = prepared_run(id, text);
        // The memory is read by `prepare_proofread_call`; this fixture reads it the same way.
        run.remembered = remembered_of(open);
        run
    };
    scan_run(&remembered, &reply_of(kind, quote))
}

fn prepared_run(id: i64, text: &str) -> PreparedProofreadRun {
    PreparedProofreadRun {
        batches: vec![PreparedProofreadBatch {
            call: PreparedTranslateCall {
                endpoint: "https://api.example.invalid/v1".to_owned(),
                model: "gpt-test".to_owned(),
                temperature: None,
                max_tokens: None,
                api_key: "sk-fake".to_owned(),
                prompt: "p".to_owned(),
            },
            segments: vec![PreparedProofreadSegment {
                segment_id: id,
                scanned_text: text.to_owned(),
                has_source: true,
            }],
        }],
        remembered: Vec::new(),
    }
}

fn scan_run(run: &PreparedProofreadRun, reply: &str) -> (usize, u32) {
    let received: Arc<Mutex<Vec<serde_json::Value>>> = Arc::new(Mutex::new(Vec::new()));
    let sink = Arc::clone(&received);
    let channel = tauri::ipc::Channel::new(move |body| {
        if let tauri::ipc::InvokeResponseBody::Json(json) = body {
            sink.lock()
                .unwrap()
                .push(serde_json::from_str(&json).expect("event json"));
        }
        Ok(())
    });
    let provider = Replying(reply.to_owned());
    let out =
        tauri::async_runtime::block_on(run_proofread_call(&provider, run, &channel, &|| false));
    let Ok(AiProofreadOutcomeWire::Done { filtered, .. }) = out else {
        panic!("scan did not finish")
    };
    let events = received.lock().unwrap().clone();
    let findings: usize = events
        .iter()
        .filter(|e| e["kind"] == "batch")
        .flat_map(|b| b["segments"].as_array().expect("segments").iter())
        .map(|s| s["findings"].as_array().map_or(0, Vec::len))
        .sum();
    (findings, filtered)
}

fn remembered_of(
    open: &OpenWork,
) -> Vec<auratranslate_lib::core::ai::proofread::ProofreadSignature> {
    let _g = KEYCHAIN_TEST_LOCK
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    keyring_core::set_default_store(keyring_core::mock::Store::new().expect("mock store"));
    ai_config_save_key(AiConfigTier::Global, "sk-memory").expect("save key");
    let global =
        Store::open(StoreSpec::global(temp_dir("global").join("global.db"))).expect("global");
    write_field(
        &global,
        AiConfigField::Endpoint,
        "https://api.example.invalid/v1/chat/completions",
    )
    .expect("endpoint");
    write_field(&global, AiConfigField::Model, "gpt-test").expect("model");
    let segs = read_open_chapter_segments(Some(open))
        .expect("chapter")
        .segments;
    let filled: Vec<i64> = segs
        .iter()
        .filter(|s| !s.target_text.trim().is_empty())
        .map(|s| s.id)
        .collect();
    let out = prepare_proofread_call(Some(&global), Some(open), &filled).expect("prepare");
    let _ = ai_config_delete_key(AiConfigTier::Global);
    match out {
        ProofreadPrepareOutcome::Ready(run) => run.remembered,
        ProofreadPrepareOutcome::NotConfigured => panic!("expected Ready"),
    }
}

const GRAMMAR_TEXT: &str = "Hôm qua tôi đã đi chợ mua rau và cá về nhà nấu cơm cho cả nhà.";
const GRAMMAR_PHRASE: &str = "đã đi chợ";

#[test]
fn a_remembered_spelling_phrase_is_not_sent_again_and_is_counted() {
    let open = work_of("spelling", 2);
    let id = ids(&open)[0];
    let text = "Mọi nguời đã đi.";
    set_target(&open, id, text);

    assert_eq!(scan(&open, id, text, "spelling", "nguời"), (1, 0));
    remember(&open, id, text, "nguời", ProofreadKind::Spelling).expect("remember");
    assert_eq!(scan(&open, id, text, "spelling", "nguời"), (0, 1));

    let other = "Nguời ấy đến muộn.";
    assert_eq!(
        scan(&open, id, other, "spelling", "Nguời"),
        (1, 0),
        "a different phrase is still reported"
    );
    assert_eq!(table_count(&open, "proof_ignore"), 1);
}

#[test]
fn a_spelling_signature_ignores_context_and_a_grammar_one_does_not() {
    let open = work_of("context", 2);
    let (id, second) = (ids(&open)[0], ids(&open)[1]);
    set_target(&open, id, GRAMMAR_TEXT);
    set_target(&open, second, "Mọi nguời đã đi.");
    remember(
        &open,
        id,
        GRAMMAR_TEXT,
        GRAMMAR_PHRASE,
        ProofreadKind::Grammar,
    )
    .expect("remember grammar");
    remember(
        &open,
        second,
        "Mọi nguời đã đi.",
        "nguời",
        ProofreadKind::Spelling,
    )
    .expect("remember spelling");

    let elsewhere =
        "Chiều nay mẹ nói rằng em đã đi chợ từ sớm, còn anh thì chưa kịp ăn gì cả ngày.";
    assert_eq!(
        scan(&open, id, elsewhere, "grammar", GRAMMAR_PHRASE),
        (1, 0),
        "same phrase, other context"
    );
    assert_eq!(
        scan(&open, id, GRAMMAR_TEXT, "grammar", GRAMMAR_PHRASE),
        (0, 1)
    );
    assert_eq!(
        scan(
            &open,
            id,
            "Họ thấy nguời lạ ở cửa, không ai biết là ai.",
            "spelling",
            "nguời"
        ),
        (0, 1)
    );
}

#[test]
fn a_context_scoped_memory_survives_merging_and_splitting_the_segment_it_came_from() {
    let open = work_of("regroup", 3);
    let before = ids(&open);
    set_target(&open, before[0], "Câu đầu tiên của đoạn.");
    set_target(&open, before[1], GRAMMAR_TEXT);
    remember(
        &open,
        before[1],
        GRAMMAR_TEXT,
        GRAMMAR_PHRASE,
        ProofreadKind::Grammar,
    )
    .expect("remember");

    let merged = merge_segments(Some(&open), before[1]).expect("merge");
    let merged_id = merged.new_segments[0].id;
    let merged_text = live_text(&open, merged_id);
    assert!(
        merged_text.contains(GRAMMAR_TEXT) && merged_text != GRAMMAR_TEXT,
        "{merged_text}"
    );
    assert_eq!(
        scan(&open, merged_id, &merged_text, "grammar", GRAMMAR_PHRASE),
        (0, 1)
    );

    let split = split_segment(Some(&open), merged_id, vec![4]).expect("split");
    let head = &split.new_segments[0];
    assert!(
        head.target_text.contains(GRAMMAR_PHRASE),
        "the head keeps the sentence: {}",
        head.target_text
    );
    assert_eq!(
        scan(&open, head.id, &head.target_text, "grammar", GRAMMAR_PHRASE),
        (0, 1)
    );
}

#[test]
fn a_phrase_at_the_segment_edge_is_reported_again_once_the_segment_is_merged() {
    let open = work_of("edge", 3);
    let before = ids(&open);
    let edge = "đã đi chợ mua rau";
    set_target(&open, before[0], "Câu đầu tiên của đoạn.");
    set_target(&open, before[1], edge);
    remember(
        &open,
        before[1],
        edge,
        "đã đi chợ mua rau",
        ProofreadKind::Grammar,
    )
    .expect("remember");
    assert_eq!(
        scan(&open, before[1], edge, "grammar", "đã đi chợ mua rau"),
        (0, 1)
    );

    let merged = merge_segments(Some(&open), before[1]).expect("merge");
    let text = live_text(&open, merged.new_segments[0].id);
    assert_eq!(
        scan(
            &open,
            merged.new_segments[0].id,
            &text,
            "grammar",
            "đã đi chợ mua rau"
        ),
        (1, 0)
    );
}

#[test]
fn another_work_still_reports_the_same_phrase() {
    let first = work_of("work-a", 2);
    let second = work_of("work-b", 2);
    let (a, b) = (ids(&first)[0], ids(&second)[0]);
    set_target(&first, a, GRAMMAR_TEXT);
    set_target(&second, b, GRAMMAR_TEXT);
    remember(
        &first,
        a,
        GRAMMAR_TEXT,
        GRAMMAR_PHRASE,
        ProofreadKind::Grammar,
    )
    .expect("remember");

    assert_eq!(
        scan(&first, a, GRAMMAR_TEXT, "grammar", GRAMMAR_PHRASE),
        (0, 1)
    );
    assert_eq!(
        scan(&second, b, GRAMMAR_TEXT, "grammar", GRAMMAR_PHRASE),
        (1, 0)
    );
    assert_eq!(table_count(&second, "proof_ignore"), 0);
}

#[test]
fn forgetting_one_or_all_makes_the_next_scan_report_again() {
    let open = work_of("forget", 2);
    let (id, second) = (ids(&open)[0], ids(&open)[1]);
    set_target(&open, id, GRAMMAR_TEXT);
    set_target(&open, second, "Mọi nguời đã đi.");
    remember(
        &open,
        id,
        GRAMMAR_TEXT,
        GRAMMAR_PHRASE,
        ProofreadKind::Grammar,
    )
    .expect("grammar");
    remember(
        &open,
        second,
        "Mọi nguời đã đi.",
        "nguời",
        ProofreadKind::Spelling,
    )
    .expect("spelling");
    remember(
        &open,
        second,
        "Mọi nguời đã đi.",
        "nguời",
        ProofreadKind::Spelling,
    )
    .expect("twice is one row");

    let listed = proofread_ignore_list(Some(&open)).expect("list");
    assert_eq!(listed.len(), 2, "UNIQUE keeps one row per signature");
    let spelling = listed
        .iter()
        .find(|e| e.kind == ProofreadKind::Spelling)
        .expect("spelling entry");
    assert_eq!(
        (
            spelling.context_before.as_str(),
            spelling.context_after.as_str()
        ),
        ("", "")
    );

    assert!(proofread_ignore_remove(Some(&open), spelling.id).expect("remove"));
    assert!(!proofread_ignore_remove(Some(&open), spelling.id).expect("remove again"));
    assert_eq!(
        scan(&open, id, "Mọi nguời đã đi.", "spelling", "nguời"),
        (1, 0)
    );
    assert_eq!(
        scan(&open, id, GRAMMAR_TEXT, "grammar", GRAMMAR_PHRASE),
        (0, 1)
    );

    assert_eq!(proofread_ignore_clear(Some(&open)).expect("clear"), 1);
    assert_eq!(
        scan(&open, id, GRAMMAR_TEXT, "grammar", GRAMMAR_PHRASE),
        (1, 0)
    );
    assert!(proofread_ignore_list(Some(&open)).expect("list").is_empty());
}

#[test]
fn the_tables_carry_no_segment_position_explanation_or_suggestion_column() {
    let open = work_of("columns", 1);
    assert_eq!(
        columns(&open, "proof_ignore"),
        [
            "id",
            "kind",
            "phrase",
            "context_before",
            "context_after",
            "created_at"
        ]
    );
    assert_eq!(
        columns(&open, "proof_scan"),
        ["id", "scope", "scanned_at", "reported", "filtered"]
    );
}

#[test]
fn every_check_error_writes_nothing() {
    let open = work_of("errors", 3);
    let before = ids(&open);
    let id = before[1];
    let text = "Mọi nguời đã đi.";
    set_target(&open, id, text);
    set_target(&open, before[2], "a  b");
    let (start, end) = utf16_range(text, "nguời");

    let cases: Vec<(&str, Result<(), IpcError>)> = vec![
        (
            "ai_proofread.text_changed",
            proofread_ignore_add(
                Some(&open),
                id,
                "Mọi người đã đi.",
                start,
                end,
                "nguời",
                ProofreadKind::Spelling,
            ),
        ),
        (
            "ai_proofread.text_changed",
            proofread_ignore_add(
                Some(&open),
                id,
                text,
                start,
                end,
                "người",
                ProofreadKind::Spelling,
            ),
        ),
        (
            "ai_proofread.range_invalid",
            proofread_ignore_add(
                Some(&open),
                id,
                text,
                start,
                start,
                "",
                ProofreadKind::Spelling,
            ),
        ),
        (
            "ai_proofread.range_invalid",
            proofread_ignore_add(Some(&open), id, text, 3, 99, "x", ProofreadKind::Spelling),
        ),
        (
            "ai_proofread.range_invalid",
            proofread_ignore_add(
                Some(&open),
                before[2],
                "a  b",
                1,
                3,
                "  ",
                ProofreadKind::Spelling,
            ),
        ),
        (
            "work.none_open",
            proofread_ignore_add(None, id, text, start, end, "nguời", ProofreadKind::Spelling),
        ),
    ];
    for (code, result) in cases {
        let err = result.expect_err(code);
        assert_eq!(err.code(), code);
        assert_eq!(
            table_count(&open, "proof_ignore"),
            0,
            "{code} must write nothing"
        );
    }

    let merged = merge_segments(Some(&open), id).expect("merge");
    assert!(!merged.retired.is_empty());
    let retired = proofread_ignore_add(
        Some(&open),
        id,
        text,
        start,
        end,
        "nguời",
        ProofreadKind::Spelling,
    );
    assert!(retired.is_err(), "a retired segment is refused");
    assert_eq!(table_count(&open, "proof_ignore"), 0);
}

#[test]
fn the_scan_count_records_one_row_and_refuses_what_rust_can_check() {
    let open = work_of("scan-record", 1);
    let work_id = open.meta.work_id.clone();

    proofread_scan_record(Some(&open), &work_id, "chapter", 4, 1).expect("record");
    assert_eq!(table_count(&open, "proof_scan"), 1);
    let row: (String, i64, i64, String) = open
        .store
        .read(|conn| {
            conn.query_row(
                "SELECT scope, reported, filtered, scanned_at FROM proof_scan",
                [],
                |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?)),
            )
        })
        .expect("row");
    assert_eq!((row.0.as_str(), row.1, row.2), ("chapter", 4, 1));
    assert!(row.3.ends_with('Z'), "{}", row.3);

    for (work, scope, reported, filtered) in [
        ("another-work", "chapter", 1, 0),
        (work_id.as_str(), "whole_library", 1, 0),
        (work_id.as_str(), "segment", -1, 0),
        (work_id.as_str(), "segment", 0, -1),
    ] {
        let err = proofread_scan_record(Some(&open), work, scope, reported, filtered)
            .expect_err("refused");
        assert_eq!(err.code(), "ai_proofread.scan_record_rejected");
    }
    assert_eq!(table_count(&open, "proof_scan"), 1);
    assert_eq!(
        proofread_scan_record(None, &work_id, "segment", 0, 0)
            .expect_err("none")
            .code(),
        "work.none_open"
    );
}

#[test]
fn a_cancelled_scan_and_a_scan_never_write_the_memory_tables() {
    let open = work_of("cancel", 1);
    let id = ids(&open)[0];
    set_target(&open, id, "Mọi nguời đã đi.");
    remember(
        &open,
        id,
        "Mọi nguời đã đi.",
        "nguời",
        ProofreadKind::Spelling,
    )
    .expect("remember");
    let run = prepared_run(id, "Mọi nguời đã đi.");
    let (channel, _events) = {
        let received: Arc<Mutex<Vec<serde_json::Value>>> = Arc::new(Mutex::new(Vec::new()));
        (
            tauri::ipc::Channel::<AiProofreadEventWire>::new(|_| Ok(())),
            received,
        )
    };
    let out =
        tauri::async_runtime::block_on(run_proofread_call(&Cancelling, &run, &channel, &|| false));
    assert_eq!(
        out.ok(),
        Some(AiProofreadOutcomeWire::Cancelled { not_scanned: 1 })
    );
    scan_run(&run, &reply_of("spelling", "nguời"));
    assert_eq!(table_count(&open, "proof_scan"), 0);
    assert_eq!(table_count(&open, "proof_ignore"), 1);
}
