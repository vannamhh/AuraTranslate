//! The proofreader's decision rows (FR150, AD-56): the four writers, the counts command, the
//! absence of any backfill, and the rule that remembering or forgetting "not an error" never
//! touches a decision.

use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};

use auratranslate_lib::commands::project::{OpenWork, create_work_from_text, open_work};
use auratranslate_lib::commands::proofread::{
    proofread_decision_counts, proofread_decision_skip, proofread_ignore_add,
    proofread_ignore_clear, proofread_ignore_remove,
};
use auratranslate_lib::commands::segment::{
    SegmentTargetEdit, accept_proofread_range, promote_ai_translation, read_open_chapter_segments,
    revert_proofread_range, save_segment_targets,
};
use auratranslate_lib::core::ai::proofread::ProofDecisionCounts;
use auratranslate_lib::core::library::indexer::IndexedWork;
use auratranslate_lib::core::segment::proofread_kind::ProofreadKind;
use auratranslate_lib::core::store::Transaction;

static NEXT_DIR: AtomicU64 = AtomicU64::new(0);

const SCANNED: &str = "Mọi nguời đã đi";
const PHRASE: &str = "nguời";
const SUGGESTION: &str = "người";
const START: u32 = 4;
const END: u32 = 9;

fn temp_dir() -> PathBuf {
    let n = NEXT_DIR.fetch_add(1, Ordering::Relaxed);
    let dir = std::env::temp_dir().join(format!(
        "auratranslate-proof-decision-{}-{n}",
        std::process::id()
    ));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).expect("create temp dir");
    dir
}

struct Fixture {
    open: OpenWork,
    segment_id: i64,
}

fn fixture() -> Fixture {
    let open = create_work_from_text(&temp_dir(), "Decisions", "en", "", SCANNED.to_owned())
        .expect("create work");
    let segment_id = read_open_chapter_segments(Some(&open))
        .expect("load")
        .segments[0]
        .id;
    promote_ai_translation(Some(&open), segment_id, SCANNED, false).expect("seed target");
    Fixture { open, segment_id }
}

impl Fixture {
    fn accept(&self, kind: ProofreadKind) -> i64 {
        accept_proofread_range(
            Some(&self.open),
            self.segment_id,
            SCANNED,
            START,
            END,
            PHRASE,
            SUGGESTION,
            kind,
        )
        .expect("accept")
        .decision_id
    }

    fn ignore(&self, kind: ProofreadKind) {
        proofread_ignore_add(
            Some(&self.open),
            self.segment_id,
            SCANNED,
            START,
            END,
            PHRASE,
            kind,
        )
        .expect("ignore");
    }

    fn counts(&self) -> (i64, i64, i64) {
        let c = proofread_decision_counts(Some(&self.open)).expect("counts");
        (c.accepted, c.not_error, c.skipped)
    }

    fn rows(&self, table: &'static str) -> i64 {
        self.open
            .store
            .read(move |conn| {
                conn.query_row(&format!("SELECT COUNT(*) FROM {table}"), [], |r| r.get(0))
            })
            .expect("count rows")
    }
}

fn indexed_of(open: &OpenWork) -> IndexedWork {
    IndexedWork {
        work_id: open.meta.work_id.clone(),
        atproj_path: open.dir.clone(),
        name: open.meta.name.clone(),
        source_lang: open.meta.source_lang.clone(),
        genre: open.meta.genre.clone(),
        created_at: open.meta.created_at.clone(),
        updated_at: open.meta.updated_at.clone(),
        chapter_count: open.meta.chapter_count,
        status: open.meta.status.clone(),
        status_is_override: open.meta.status_is_override,
        chapter_done_count: open.meta.chapter_done_count,
    }
}

fn reopen(open: OpenWork) -> OpenWork {
    let indexed = indexed_of(&open);
    drop(open);
    open_work(&indexed.work_id, Some(&indexed)).expect("reopen work")
}

#[test]
fn a_work_with_no_decision_counts_zero_in_every_cell() {
    let f = fixture();
    let counts = proofread_decision_counts(Some(&f.open)).expect("counts");
    assert_eq!(
        (counts.accepted, counts.not_error, counts.skipped),
        (0, 0, 0)
    );
    assert_eq!(counts.by_kind.len(), ProofreadKind::ALL.len());
    assert!(
        counts
            .by_kind
            .iter()
            .all(|k| k.accepted + k.not_error + k.skipped == 0)
    );
}

#[test]
fn each_of_the_four_writers_leaves_its_own_outcome_and_kind() {
    let f = fixture();
    f.ignore(ProofreadKind::Grammar);
    proofread_decision_skip(Some(&f.open), ProofreadKind::Obscure).expect("skip");
    let spelling = f.accept(ProofreadKind::Spelling);
    assert_eq!(f.counts(), (1, 1, 1));

    let end = START + u32::try_from(SUGGESTION.encode_utf16().count()).expect("len");
    revert_proofread_range(
        Some(&f.open),
        f.segment_id,
        &SCANNED.replace(PHRASE, SUGGESTION),
        START,
        end,
        SUGGESTION,
        PHRASE,
        ProofreadKind::Spelling,
        spelling,
    )
    .expect("revert");
    assert_eq!(f.counts(), (0, 1, 1));
    f.accept(ProofreadKind::Mistranslation);

    let counts: ProofDecisionCounts = proofread_decision_counts(Some(&f.open)).expect("counts");
    let of = |kind: ProofreadKind| {
        let k = counts
            .by_kind
            .iter()
            .find(|k| k.kind == kind)
            .expect("kind");
        (k.accepted, k.not_error, k.skipped)
    };
    assert_eq!(of(ProofreadKind::Spelling), (0, 0, 0));
    assert_eq!(of(ProofreadKind::Grammar), (0, 1, 0));
    assert_eq!(of(ProofreadKind::Obscure), (0, 0, 1));
    assert_eq!(of(ProofreadKind::Mistranslation), (1, 0, 0));
}

#[test]
fn skipping_writes_one_row_per_call_and_touches_no_segment_or_memory() {
    let f = fixture();
    let segments_before: (String, String) = f
        .open
        .store
        .read(|conn| {
            conn.query_row("SELECT target_text, status FROM segment LIMIT 1", [], |r| {
                Ok((r.get(0)?, r.get(1)?))
            })
        })
        .expect("segment");
    proofread_decision_skip(Some(&f.open), ProofreadKind::Spelling).expect("skip once");
    proofread_decision_skip(Some(&f.open), ProofreadKind::Spelling).expect("skip twice");
    assert_eq!(f.counts(), (0, 0, 2));
    assert_eq!(f.rows("proof_ignore"), 0);
    let segments_after: (String, String) = f
        .open
        .store
        .read(|conn| {
            conn.query_row("SELECT target_text, status FROM segment LIMIT 1", [], |r| {
                Ok((r.get(0)?, r.get(1)?))
            })
        })
        .expect("segment");
    assert_eq!(segments_before, segments_after);
    assert_eq!(
        proofread_decision_skip(None, ProofreadKind::Spelling)
            .expect_err("no work")
            .code(),
        "work.none_open"
    );
}

#[test]
fn remembering_the_same_finding_twice_is_one_signature_and_two_decisions() {
    let f = fixture();
    f.ignore(ProofreadKind::Spelling);
    f.ignore(ProofreadKind::Spelling);
    assert_eq!(f.rows("proof_ignore"), 1);
    assert_eq!(f.counts(), (0, 2, 0));
}

#[test]
fn forgetting_a_remembered_finding_never_removes_a_decision() {
    let f = fixture();
    f.ignore(ProofreadKind::Spelling);
    let id: i64 = f
        .open
        .store
        .read(|conn| conn.query_row("SELECT id FROM proof_ignore", [], |r| r.get(0)))
        .expect("ignore id");
    assert!(proofread_ignore_remove(Some(&f.open), id).expect("remove"));
    assert_eq!(f.counts(), (0, 1, 0));
    f.ignore(ProofreadKind::Spelling);
    proofread_ignore_clear(Some(&f.open)).expect("clear");
    assert_eq!((f.rows("proof_ignore"), f.counts()), (0, (0, 2, 0)));
}

#[test]
fn a_refused_ignore_leaves_no_decision() {
    let f = fixture();
    let refused = proofread_ignore_add(
        Some(&f.open),
        f.segment_id,
        "stale text",
        START,
        END,
        PHRASE,
        ProofreadKind::Spelling,
    );
    assert!(refused.is_err());
    assert_eq!(f.counts(), (0, 0, 0));
}

#[test]
fn decisions_survive_closing_and_reopening_the_work() {
    let f = fixture();
    f.accept(ProofreadKind::Spelling);
    proofread_decision_skip(Some(&f.open), ProofreadKind::Grammar).expect("skip");
    let reopened = Fixture {
        open: reopen(f.open),
        segment_id: f.segment_id,
    };
    assert_eq!(reopened.counts(), (1, 0, 1));
}

#[test]
fn a_work_from_before_step_38_has_no_decisions_even_with_remembered_findings() {
    let f = fixture();
    f.ignore(ProofreadKind::Spelling);
    f.open
        .store
        .write(|tx: &Transaction<'_>| {
            tx.execute_batch(
                "DROP TABLE proof_decision; \
                 DROP TRIGGER segment_version_requires_translation_origin; \
                 ALTER TABLE segment_version DROP COLUMN translation_origin; \
                 DROP TRIGGER glossary_entry_edit_clears_unconfirm; \
                 DROP TRIGGER glossary_entry_lifecycle_is_one_way; \
                 ALTER TABLE glossary_entry DROP COLUMN pending_confirmed_at; \
                 CREATE TRIGGER glossary_entry_lifecycle_is_one_way BEFORE UPDATE OF translation ON glossary_entry \
                 WHEN OLD.translation IS NOT NULL AND NEW.translation IS NULL \
                 BEGIN SELECT RAISE(ABORT, 'glossary lifecycle is one-way'); END; \
                 DELETE FROM schema_migration_log WHERE version IN (38, 39, 40); \
                 PRAGMA user_version = 37;",
            )
        })
        .expect("step back to 37");
    let reopened = Fixture {
        open: reopen(f.open),
        segment_id: f.segment_id,
    };
    assert_eq!(reopened.rows("proof_ignore"), 1, "the old memory is kept");
    assert_eq!(
        reopened.counts(),
        (0, 0, 0),
        "AD-56: no decision is back-filled from proof_ignore"
    );
}

#[test]
fn the_users_own_typing_restoring_the_phrase_leaves_the_accepted_decision() {
    let f = fixture();
    f.accept(ProofreadKind::Spelling);
    let chapter_id = read_open_chapter_segments(Some(&f.open))
        .expect("load")
        .chapter_id;
    save_segment_targets(
        Some(&f.open),
        chapter_id,
        &[SegmentTargetEdit {
            id: f.segment_id,
            target_text: SCANNED.to_owned(),
        }],
    )
    .expect("user write");
    let target: String = f
        .open
        .store
        .read(|conn| conn.query_row("SELECT target_text FROM segment LIMIT 1", [], |r| r.get(0)))
        .expect("target");
    assert_eq!(target, SCANNED, "the typing write put the phrase back");
    assert_eq!(f.counts(), (1, 0, 0));
}
