//! The proofreader's only road to `segment.target_text` (FR85, AD-53): behaviour of the accept
//! and revert write, and the source gate that keeps the road single.

use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};

use auratranslate_lib::commands::project::{OpenWork, create_work_from_text};
use auratranslate_lib::commands::segment::{read_open_chapter_segments, replace_proofread_range};
use auratranslate_lib::core::i18n::IpcError;
use auratranslate_lib::core::segment::proofread_kind::ProofreadKind;
use auratranslate_lib::core::store::Transaction;

#[path = "support/boundary_scan.rs"]
#[allow(dead_code)] // shared module: not every helper is used in this file
mod boundary_scan;

static NEXT_DIR: AtomicU64 = AtomicU64::new(0);

const SCANNED: &str = "Mọi nguời đã đi";
const PHRASE: &str = "nguời";
const SUGGESTION: &str = "người";
const PHRASE_START: u32 = 4;
const PHRASE_END: u32 = 9;

fn temp_dir() -> PathBuf {
    let n = NEXT_DIR.fetch_add(1, Ordering::Relaxed);
    let dir = std::env::temp_dir().join(format!(
        "auratranslate-proofread-write-{}-{n}",
        std::process::id()
    ));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).expect("create temp dir");
    dir
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Row {
    target_text: String,
    baseline_target_text: String,
    baseline_translation_origin: String,
    translation_origin: String,
    status: String,
    retired: bool,
    versions: i64,
}

struct Fixture {
    open: OpenWork,
    segment_id: i64,
}

fn fixture(
    text: &str,
    baseline_text: &str,
    baseline_origin: &str,
    stored_origin: &str,
    status: &str,
) -> Fixture {
    let open = create_work_from_text(&temp_dir(), "Proofread", "en", "", text.to_owned())
        .expect("create work");
    let segment_id = read_open_chapter_segments(Some(&open))
        .expect("load")
        .segments[0]
        .id;
    let (target, base, base_origin, stored, status) = (
        text.to_owned(),
        baseline_text.to_owned(),
        baseline_origin.to_owned(),
        stored_origin.to_owned(),
        status.to_owned(),
    );
    open.store
        .write(move |tx: &Transaction<'_>| {
            tx.execute(
                "UPDATE segment SET target_text = ?1, baseline_target_text = ?2, \
                 baseline_translation_origin = ?3, translation_origin = ?4, status = ?5 WHERE id = ?6",
                (&target, &base, &base_origin, &stored, &status, segment_id),
            )
        })
        .expect("seed row");
    Fixture { open, segment_id }
}

impl Fixture {
    fn row(&self) -> Row {
        let id = self.segment_id;
        self.open
            .store
            .write(move |tx: &Transaction<'_>| {
                tx.query_row(
                    "SELECT target_text, baseline_target_text, baseline_translation_origin, \
                     translation_origin, status, retired_at IS NOT NULL, \
                     (SELECT COUNT(*) FROM segment_version WHERE segment_id = ?1) FROM segment WHERE id = ?1",
                    [id],
                    |r| {
                        Ok(Row {
                            target_text: r.get(0)?,
                            baseline_target_text: r.get(1)?,
                            baseline_translation_origin: r.get(2)?,
                            translation_origin: r.get(3)?,
                            status: r.get(4)?,
                            retired: r.get(5)?,
                            versions: r.get(6)?,
                        })
                    },
                )
            })
            .expect("read row")
    }

    fn replace(
        &self,
        expected_text: &str,
        start: u32,
        end: u32,
        expected_phrase: &str,
        replacement: &str,
        kind: ProofreadKind,
    ) -> Result<auratranslate_lib::commands::segment::PromoteAiTranslationOutcome, IpcError> {
        replace_proofread_range(
            Some(&self.open),
            self.segment_id,
            expected_text,
            start,
            end,
            expected_phrase,
            replacement,
            kind,
        )
    }

    fn accept(
        &self,
        kind: ProofreadKind,
    ) -> Result<auratranslate_lib::commands::segment::PromoteAiTranslationOutcome, IpcError> {
        self.replace(SCANNED, PHRASE_START, PHRASE_END, PHRASE, SUGGESTION, kind)
    }
}

const ACCEPTED: &str = "Mọi người đã đi";

/// (name, baseline text, baseline origin, spelling/grammar origin, meaning-kind origin)
fn origin_rows() -> [(
    &'static str,
    &'static str,
    &'static str,
    &'static str,
    &'static str,
); 4] {
    [
        (
            "user typed over an AI baseline",
            "Khác hẳn",
            "other",
            "self",
            "other",
        ),
        ("AI sentence untouched", SCANNED, "other", "other", "other"),
        (
            "bilingual import untouched",
            SCANNED,
            "bilingual_import",
            "bilingual_import",
            "other",
        ),
        ("TM 100% untouched", SCANNED, "self", "self", "other"),
    ]
}

#[test]
fn accepting_declares_the_origin_of_ad_53_rule_2_for_each_row_and_column() {
    for (name, baseline, baseline_origin, spelling_origin, meaning_origin) in origin_rows() {
        for (kind, expected_origin) in [
            (ProofreadKind::Spelling, spelling_origin),
            (ProofreadKind::Grammar, spelling_origin),
            (ProofreadKind::Mistranslation, meaning_origin),
            (ProofreadKind::TooLoose, meaning_origin),
            (ProofreadKind::Obscure, meaning_origin),
        ] {
            let f = fixture(
                SCANNED,
                baseline,
                baseline_origin,
                baseline_origin,
                "confirmed",
            );
            let before = f.row();
            let out = f
                .accept(kind)
                .unwrap_or_else(|e| panic!("{name} {kind:?}: {}", e.code()));
            let after = f.row();
            assert_eq!(after.target_text, ACCEPTED, "{name} {kind:?}");
            assert_eq!(out.target_text, ACCEPTED);
            assert_eq!(after.translation_origin, expected_origin, "{name} {kind:?}");
            assert_eq!(
                after.baseline_translation_origin, expected_origin,
                "{name} {kind:?}"
            );
            assert_eq!(after.baseline_target_text, ACCEPTED, "{name} {kind:?}");
            assert_eq!(after.status, "draft", "{name} {kind:?}");
            assert_eq!(out.status, "draft");
            assert_eq!(out.translation_origin, expected_origin);
            assert_eq!(
                after.versions, before.versions,
                "{name} {kind:?}: no segment_version row"
            );
        }
    }
}

#[test]
fn signing_right_after_accepting_keeps_the_declared_origin() {
    use auratranslate_lib::commands::segment::confirm_segment;
    let f = fixture(
        SCANNED,
        SCANNED,
        "bilingual_import",
        "bilingual_import",
        "draft",
    );
    f.accept(ProofreadKind::Spelling).expect("accept");
    confirm_segment(Some(&f.open), f.segment_id).expect("confirm");
    assert_eq!(f.row().translation_origin, "bilingual_import");
}

#[test]
fn reverting_restores_the_exact_bytes_and_the_origin_of_the_spelling_row() {
    for (name, baseline, baseline_origin, spelling_origin, _) in origin_rows() {
        let f = fixture(
            SCANNED,
            baseline,
            baseline_origin,
            baseline_origin,
            "confirmed",
        );
        f.accept(ProofreadKind::Spelling).expect("accept");
        let end = PHRASE_START + u32::try_from(SUGGESTION.encode_utf16().count()).expect("len");
        f.replace(
            ACCEPTED,
            PHRASE_START,
            end,
            SUGGESTION,
            PHRASE,
            ProofreadKind::Spelling,
        )
        .unwrap_or_else(|e| panic!("{name}: {}", e.code()));
        let after = f.row();
        assert_eq!(after.target_text, SCANNED, "{name}: byte-exact");
        assert_eq!(after.translation_origin, spelling_origin, "{name}");
        assert_eq!(after.status, "draft");
        assert_eq!(after.versions, 0);
    }
}

#[test]
fn reverting_a_meaning_kind_declares_other_even_over_a_user_sentence() {
    let f = fixture(SCANNED, "Khác hẳn", "other", "self", "draft");
    f.accept(ProofreadKind::Mistranslation).expect("accept");
    let end = PHRASE_START + u32::try_from(SUGGESTION.encode_utf16().count()).expect("len");
    f.replace(
        ACCEPTED,
        PHRASE_START,
        end,
        SUGGESTION,
        PHRASE,
        ProofreadKind::Mistranslation,
    )
    .expect("revert");
    assert_eq!(f.row().translation_origin, "other");
    assert_eq!(f.row().target_text, SCANNED);
}

#[test]
fn offsets_are_utf16_units_and_the_replacement_is_not_normalised() {
    let text = "😀 cafe\u{301} wrogn";
    let f = fixture(text, text, "other", "other", "draft");
    let nfd_suggestion = "wro\u{6e}\u{303}g";
    let start = u32::try_from("😀 cafe\u{301} ".encode_utf16().count()).expect("start");
    let end = start + 5;
    f.replace(
        text,
        start,
        end,
        "wrogn",
        nfd_suggestion,
        ProofreadKind::Spelling,
    )
    .expect("accept");
    assert_eq!(
        f.row().target_text,
        format!("😀 cafe\u{301} {nfd_suggestion}")
    );
}

struct Case {
    name: &'static str,
    code: &'static str,
}

fn assert_refused_without_writing(
    f: &Fixture,
    result: Result<impl std::fmt::Debug, IpcError>,
    case: &Case,
) {
    let before_snapshot = f.row();
    let err = result.expect_err(case.name);
    assert_eq!(err.code(), case.code, "{}", case.name);
    assert_eq!(f.row(), before_snapshot, "{}: nothing written", case.name);
}

#[test]
fn every_check_error_writes_nothing_and_carries_its_code() {
    let kind = ProofreadKind::Spelling;
    let seeded = || fixture(SCANNED, SCANNED, "other", "other", "confirmed");

    let f = seeded();
    let r = f.replace(
        "Mọi nguời đã đi.",
        PHRASE_START,
        PHRASE_END,
        PHRASE,
        SUGGESTION,
        kind,
    );
    assert_refused_without_writing(
        &f,
        r,
        &Case {
            name: "text differs",
            code: "ai_proofread.text_changed",
        },
    );

    let f = seeded();
    let r = f.replace(SCANNED, PHRASE_START, PHRASE_END, "nguoi", SUGGESTION, kind);
    assert_refused_without_writing(
        &f,
        r,
        &Case {
            name: "phrase differs",
            code: "ai_proofread.text_changed",
        },
    );

    for (name, start, end) in [
        ("end past the string", 10, 99),
        ("start past the string", 99, 120),
        ("empty range", 4, 4),
        ("reversed", 9, 4),
    ] {
        let f = seeded();
        let r = f.replace(SCANNED, start, end, PHRASE, SUGGESTION, kind);
        assert_refused_without_writing(
            &f,
            r,
            &Case {
                name,
                code: "ai_proofread.range_invalid",
            },
        );
    }

    let astral = "😀 wrogn";
    let f = fixture(astral, astral, "other", "other", "confirmed");
    let r = f.replace(astral, 1, 3, "x", SUGGESTION, kind);
    assert_refused_without_writing(
        &f,
        r,
        &Case {
            name: "start inside a surrogate pair",
            code: "ai_proofread.range_invalid",
        },
    );
    let r = f.replace(astral, 0, 1, "x", SUGGESTION, kind);
    assert_refused_without_writing(
        &f,
        r,
        &Case {
            name: "end inside a surrogate pair",
            code: "ai_proofread.range_invalid",
        },
    );

    let f = seeded();
    let r = f.replace(SCANNED, 0, 15, SCANNED, " \n", kind);
    assert_refused_without_writing(
        &f,
        r,
        &Case {
            name: "empty after trim",
            code: "ai_proofread.text_would_be_empty",
        },
    );

    let f = seeded();
    let id = f.segment_id;
    f.open
        .store
        .write(move |tx: &Transaction<'_>| {
            tx.execute("UPDATE segment SET retired_at = strftime('%Y-%m-%dT%H:%M:%fZ','now') WHERE id = ?1", [id])
        })
        .expect("retire");
    let r = f.accept(kind);
    assert_refused_without_writing(
        &f,
        r,
        &Case {
            name: "retired",
            code: "segment.retired",
        },
    );

    let f = seeded();
    let r = replace_proofread_range(
        Some(&f.open),
        f.segment_id + 1_000,
        SCANNED,
        4,
        9,
        PHRASE,
        SUGGESTION,
        kind,
    );
    assert_eq!(r.expect_err("unknown segment").code(), "segment.not_found");
    assert_eq!(f.row().target_text, SCANNED);

    let f = fixture(SCANNED, SCANNED, "mystery", "other", "confirmed");
    let r = f.accept(kind);
    assert_refused_without_writing(
        &f,
        r,
        &Case {
            name: "unknown baseline origin",
            code: "segment.unknown_translation_origin",
        },
    );
}

#[test]
fn a_segment_outside_the_open_chapter_is_refused_before_the_text_is_compared() {
    let f = fixture(SCANNED, SCANNED, "other", "other", "confirmed");
    let id = f.segment_id;
    let other_chapter = f.open.chapter_id + 1_000;
    f.open
        .store
        .write(move |tx: &Transaction<'_>| {
            tx.execute("PRAGMA foreign_keys = OFF", [])?;
            tx.execute(
                "UPDATE segment SET chapter_id = ?1 WHERE id = ?2",
                (other_chapter, id),
            )
        })
        .expect("move segment");
    let r = f.replace(
        "not the text",
        PHRASE_START,
        PHRASE_END,
        PHRASE,
        SUGGESTION,
        ProofreadKind::Spelling,
    );
    assert_eq!(
        r.expect_err("other chapter").code(),
        "segment.not_in_open_chapter"
    );
    assert_eq!(f.row().target_text, SCANNED);
}

#[test]
fn no_work_open_is_refused() {
    let r = replace_proofread_range(
        None,
        1,
        SCANNED,
        4,
        9,
        PHRASE,
        SUGGESTION,
        ProofreadKind::Spelling,
    );
    assert_eq!(r.expect_err("no work").code(), "work.none_open");
}

const SRC_RS_FLOOR: usize = 118;
const WEBVIEW_PROOFREAD_FILE_FLOOR: usize = 8;
const WRITE_CALL: &str = "write_non_user_target(";
const ACCEPT_FILE: &str = "commands/segment/proofread_accept.rs";
const WEBVIEW_FORBIDDEN: [&str; 2] = ["save_segment_targets", "saveSegmentTargets"];

fn product_sources() -> Vec<(String, String)> {
    boundary_scan::rust_sources(&boundary_scan::src_root())
}

fn write_calls(text: &str) -> Vec<usize> {
    boundary_scan::code_lines(&boundary_scan::without_test_modules(text))
        .filter(|(_, line)| {
            line.contains(WRITE_CALL) && !line.contains("fn write_non_user_target(")
        })
        .map(|(n, _)| n)
        .collect()
}

fn proofread_rust_files(rel: &str) -> bool {
    rel == "core/ai/proofread.rs"
        || rel == "commands/proofread.rs"
        || rel.starts_with("core/ai/proofread/")
}

fn rust_violations(sources: &[(String, String)]) -> Vec<String> {
    let mut found = Vec::new();
    for (rel, text) in sources {
        let calls = write_calls(text);
        if proofread_rust_files(rel) && !calls.is_empty() {
            found.push(format!(
                "{rel}: the scan side calls the non-user write at lines {calls:?}"
            ));
        }
        if rel == ACCEPT_FILE && calls.len() != 1 {
            found.push(format!(
                "{rel}: {} calls of the non-user write, expected exactly 1",
                calls.len()
            ));
        }
    }
    found
}

fn webview_files() -> Vec<(String, String)> {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("src");
    boundary_scan::paths_with_extensions(&root, &["ts", "vue"])
        .into_iter()
        .filter(|p| {
            p.file_name()
                .and_then(|n| n.to_str())
                .is_some_and(|n| n.to_ascii_lowercase().contains("proofread"))
        })
        .map(|p| {
            let text =
                fs::read_to_string(&p).unwrap_or_else(|e| panic!("doc {}: {e}", p.display()));
            (
                p.file_name().expect("name").to_string_lossy().into_owned(),
                text,
            )
        })
        .collect()
}

fn webview_violations(files: &[(String, String)]) -> Vec<String> {
    let mut found = Vec::new();
    for (name, text) in files {
        for (line, content) in boundary_scan::code_lines(text) {
            if WEBVIEW_FORBIDDEN.iter().any(|f| content.contains(f)) {
                found.push(format!("{name}:{line}: {content}"));
            }
        }
    }
    found
}

#[test]
fn the_scanned_trees_are_large_enough_to_be_real() {
    boundary_scan::assert_population_floor(
        SRC_RS_FLOOR,
        product_sources().len(),
        "SRC_RS_FLOOR",
        "`.rs` files under `src-tauri/src/**`",
    );
    boundary_scan::assert_population_floor(
        WEBVIEW_PROOFREAD_FILE_FLOOR,
        webview_files().len(),
        "WEBVIEW_PROOFREAD_FILE_FLOOR",
        "proofreader files under `src/**`",
    );
}

#[test]
fn the_gate_sees_the_accept_call_and_the_other_non_user_writers() {
    let sources = product_sources();
    let in_accept = sources
        .iter()
        .find(|(rel, _)| rel == ACCEPT_FILE)
        .map(|(_, t)| write_calls(t).len());
    assert_eq!(
        in_accept,
        Some(1),
        "`{ACCEPT_FILE}` must hold the one call and the gate must see it"
    );
    let elsewhere: usize = sources
        .iter()
        .filter(|(rel, _)| rel != ACCEPT_FILE)
        .map(|(_, t)| write_calls(t).len())
        .sum();
    assert!(
        elsewhere >= 4,
        "the other non-user writers must be seen, found {elsewhere}"
    );
}

#[test]
fn only_the_accept_file_writes_for_the_proofreader_and_the_webview_has_no_second_road() {
    let rust = rust_violations(&product_sources());
    assert!(
        rust.is_empty(),
        "a second proofreader write road (FR85, AD-53 rule 3): {rust:#?}"
    );
    let webview = webview_violations(&webview_files());
    assert!(
        webview.is_empty(),
        "the proofreader webview saves targets itself (FR85): {webview:#?}"
    );
}

#[test]
fn the_predicates_catch_each_forbidden_shape_and_ignore_comments() {
    let scan_side = vec![(
        "core/ai/proofread.rs".to_owned(),
        "fn a() {\n    write_non_user_target(tx, 1, \"x\", \"o\", None);\n}\n".to_owned(),
    )];
    assert_eq!(rust_violations(&scan_side).len(), 1);

    let two_calls = vec![(
        ACCEPT_FILE.to_owned(),
        "fn a() {\n    write_non_user_target(tx);\n    write_non_user_target(tx);\n}\n".to_owned(),
    )];
    assert_eq!(rust_violations(&two_calls).len(), 1);

    let commented = vec![(
        "core/ai/proofread.rs".to_owned(),
        "// write_non_user_target(tx);\nfn a() {}\n".to_owned(),
    )];
    assert!(rust_violations(&commented).is_empty());

    let ts = vec![(
        "proofreadState.ts".to_owned(),
        "await invoke('save_segment_targets', {})\n".to_owned(),
    )];
    assert_eq!(webview_violations(&ts).len(), 1);
    let ts_comment = vec![(
        "proofreadState.ts".to_owned(),
        "// invoke('save_segment_targets')\n".to_owned(),
    )];
    assert!(webview_violations(&ts_comment).is_empty());
}
