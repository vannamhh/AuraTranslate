//! I/O and edge-case matrix of AD-54 (unconfirm a Glossary translation), run on both tiers.
//!
//! Every case builds its own temp dir and drops the stores before removing it.

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use auratranslate_lib::core::glossary::{
    Category, GlossaryError, GlossaryInjectionTerm, GlossaryTier, ReviewHarvestDetail,
    ReviewHarvestProposal, add_manual_term, approve_candidate, confirm_pending_translation,
    confirm_translation, confirmed_terms_for_injection, enqueue_review_harvest, load_tier,
    pending_candidates, promote_to_global, unconfirm_pending_translation, update_manual_term,
};
use auratranslate_lib::core::i18n::IpcError;
use auratranslate_lib::core::matching::MatchLang;
use auratranslate_lib::core::scope::{ScopeResolver, WorkScope};
use auratranslate_lib::core::store::{
    GLOBAL_MIGRATIONS, Migration, PROJECT_MIGRATIONS, Store, StoreError, StoreKind, StoreSpec,
    Transaction,
};

static NEXT_DIR: AtomicU64 = AtomicU64::new(0);

fn temp_dir(tag: &str) -> PathBuf {
    let n = NEXT_DIR.fetch_add(1, Ordering::Relaxed);
    let dir = std::env::temp_dir().join(format!(
        "auratranslate-unconfirm-{}-{}-{}",
        std::process::id(),
        tag,
        n
    ));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap_or_else(|e| panic!("tao {}: {e}", dir.display()));
    dir
}

fn cleanup(dir: &Path) {
    let _ = fs::remove_dir_all(dir);
}

const TIERS: [GlossaryTier; 2] = [GlossaryTier::Global, GlossaryTier::Work];

struct Fixture {
    dir: PathBuf,
    global: Store,
    work: Store,
    tier: GlossaryTier,
}

impl Fixture {
    fn new(tag: &str, tier: GlossaryTier) -> Self {
        let dir = temp_dir(tag);
        let global = Store::open(StoreSpec::global(dir.join("global.db"))).expect("global.db");
        let work = Store::open(StoreSpec::project(dir.join("project.db"))).expect("project.db");
        Self {
            dir,
            global,
            work,
            tier,
        }
    }

    fn store(&self) -> &Store {
        match self.tier {
            GlossaryTier::Global => &self.global,
            GlossaryTier::Work => &self.work,
        }
    }

    fn add(&self, term: &str, translation: Option<&str>, note: &str) -> i64 {
        add_manual_term(
            &self.global,
            Some(&self.work),
            self.tier,
            term,
            translation,
            note,
            Category::Person,
        )
        .expect("them muc")
    }

    fn confirm(&self, id: i64, text: &str) -> Option<String> {
        confirm_pending_translation(&self.global, Some(&self.work), self.tier, id, text)
            .expect("chot")
    }

    fn unconfirm(&self, id: i64, text: &str, mark: &str) -> Result<(), GlossaryError> {
        unconfirm_pending_translation(&self.global, Some(&self.work), self.tier, id, text, mark)
    }

    fn row(&self, id: i64) -> (Option<String>, Option<String>, String, String) {
        self.store()
            .read(|conn| {
                conn.query_row(
                    "SELECT translation, pending_confirmed_at, note, category \
                     FROM glossary_entry WHERE id = ?1",
                    [id],
                    |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?)),
                )
            })
            .expect("doc hang")
    }

    fn injected(&self, text: &str) -> Vec<GlossaryInjectionTerm> {
        let resolver = ScopeResolver::with_work(WorkScope {
            work_id: "0192f3c4-5678-4abc-8def-0123456789ab".to_owned(),
        });
        let work = match self.tier {
            GlossaryTier::Global => None,
            GlossaryTier::Work => Some(&self.work),
        };
        let fallback = ScopeResolver::global_only();
        let resolver = if work.is_some() { &resolver } else { &fallback };
        confirmed_terms_for_injection(resolver, &self.global, work, text, MatchLang::Zh)
            .expect("lap prompt")
            .injected
    }

    fn finish(self) {
        let Self {
            dir, global, work, ..
        } = self;
        drop(global);
        drop(work);
        cleanup(&dir);
    }
}

fn code_of(err: GlossaryError) -> String {
    IpcError::from(err).code().to_owned()
}

#[test]
fn confirm_writes_translation_and_mark_together_and_unconfirm_returns_the_row_to_pending() {
    for tier in TIERS {
        let fx = Fixture::new("roundtrip", tier);
        let id = fx.add("慕容", None, "ghi chu");
        assert_eq!(fx.row(id).0, None);

        let mark = fx.confirm(id, "Mộ Dung").expect("lan chot dau tra moc");
        let (translation, stored_mark, _, _) = fx.row(id);
        assert_eq!(translation.as_deref(), Some("Mộ Dung"), "{tier:?}");
        assert_eq!(stored_mark.as_deref(), Some(mark.as_str()), "{tier:?}");
        assert_eq!(
            fx.injected("慕容").len(),
            1,
            "{tier:?}: da chot thi duoc chen"
        );

        fx.unconfirm(id, "Mộ Dung", &mark).expect("go chot");
        let (translation, stored_mark, note, category) = fx.row(id);
        assert_eq!((translation, stored_mark), (None, None), "{tier:?}");
        assert_eq!(
            (note.as_str(), category.as_str()),
            ("ghi chu", "person"),
            "{tier:?}: cot khac khong doi"
        );
        assert!(
            fx.injected("慕容").is_empty(),
            "{tier:?}: muc ve cho chot phai roi prompt lan lap ke"
        );

        let again = fx.confirm(id, "Mộ Dung").expect("chot lai sau go chot");
        assert_eq!(fx.row(id).1.as_deref(), Some(again.as_str()), "{tier:?}");
        fx.finish();
    }
}

#[test]
fn confirming_an_already_confirmed_entry_edits_without_a_mark() {
    for tier in TIERS {
        let fx = Fixture::new("confirm-race", tier);
        let id = fx.add("慕容", None, "");
        let mark = fx.confirm(id, "Mộ Dung").expect("chot");

        let second = confirm_translation(fx.store(), id, "Mộ Dong").expect("chot lai");
        assert_eq!(second, None, "{tier:?}: chot lai khong dat cot");
        let (translation, stored_mark, _, _) = fx.row(id);
        assert_eq!(translation.as_deref(), Some("Mộ Dong"), "{tier:?}");
        assert_eq!(stored_mark, None, "{tier:?}: AFTER trigger xoa moc cu");
        assert_eq!(
            code_of(fx.unconfirm(id, "Mộ Dong", &mark).unwrap_err()),
            "glossary.not_unconfirmable",
            "{tier:?}"
        );
        fx.finish();
    }
}

#[test]
fn a_changed_translation_refuses_with_translation_changed_and_writes_nothing() {
    for tier in TIERS {
        let fx = Fixture::new("changed", tier);
        let id = fx.add("慕容", None, "");
        let mark = fx.confirm(id, "Mộ Dung").expect("chot");
        update_manual_term(
            &fx.global,
            Some(&fx.work),
            tier,
            id,
            Some("Mộ Dong"),
            "",
            Category::Person,
        )
        .expect("sua tay");
        let before = fx.row(id);

        let err = fx.unconfirm(id, "Mộ Dung", &mark).unwrap_err();
        assert_eq!(code_of(err), "glossary.translation_changed", "{tier:?}");
        assert_eq!(fx.row(id), before, "{tier:?}: loi khong ghi gi");
        fx.finish();
    }
}

#[test]
fn editing_back_to_the_old_text_is_not_unconfirmable() {
    for tier in TIERS {
        let fx = Fixture::new("edit-back", tier);
        let id = fx.add("慕容", None, "");
        let mark = fx.confirm(id, "Mộ Dung").expect("chot");
        for text in ["Mộ Dong", "Mộ Dung"] {
            update_manual_term(
                &fx.global,
                Some(&fx.work),
                tier,
                id,
                Some(text),
                "",
                Category::Person,
            )
            .expect("sua tay");
        }

        let err = fx.unconfirm(id, "Mộ Dung", &mark).unwrap_err();
        assert_eq!(code_of(err), "glossary.not_unconfirmable", "{tier:?}");
        assert_eq!(fx.row(id).0.as_deref(), Some("Mộ Dung"), "{tier:?}");
        fx.finish();
    }
}

#[test]
fn a_wrong_mark_is_not_unconfirmable() {
    for tier in TIERS {
        let fx = Fixture::new("wrong-mark", tier);
        let id = fx.add("慕容", None, "");
        fx.confirm(id, "Mộ Dung").expect("chot");
        let err = fx
            .unconfirm(id, "Mộ Dung", "2000-01-01T00:00:00.000Z")
            .unwrap_err();
        assert_eq!(code_of(err), "glossary.not_unconfirmable", "{tier:?}");
        assert_eq!(fx.row(id).0.as_deref(), Some("Mộ Dung"), "{tier:?}");
        fx.finish();
    }
}

#[test]
fn an_entry_inserted_confirmed_has_no_mark_and_cannot_be_unconfirmed() {
    for tier in TIERS {
        let fx = Fixture::new("inserted-confirmed", tier);
        let id = fx.add("慕容", Some("Mộ Dung"), "");
        assert_eq!(fx.row(id).1, None, "{tier:?}: INSERT de cot NULL");
        let err = fx.unconfirm(id, "Mộ Dung", "x").unwrap_err();
        assert_eq!(code_of(err), "glossary.not_unconfirmable", "{tier:?}");
        fx.finish();
    }
}

#[test]
fn a_row_from_before_the_new_step_cannot_be_unconfirmed_after_migrating() {
    for tier in TIERS {
        let dir = temp_dir("old-row");
        let (old_global, old_work) = (&GLOBAL_MIGRATIONS[..13], &PROJECT_MIGRATIONS[..38]);
        {
            let global = Store::open(StoreSpec {
                migrations: old_global,
                ..StoreSpec::global(dir.join("global.db"))
            })
            .expect("global o bac 13");
            let work = Store::open(StoreSpec {
                migrations: old_work,
                ..StoreSpec::project(dir.join("project.db"))
            })
            .expect("project o bac 39");
            assert_eq!((global.schema_version(), work.schema_version()), (13, 39));
            for store in [&global, &work] {
                store
                    .write(|tx: &Transaction<'_>| {
                        tx.execute(
                            "INSERT INTO glossary_entry \
                             (source_term, translation, note, category, term_origin, created_at) \
                             VALUES ('慕容', 'Mộ Dung', '', 'person', 'manual', 'x')",
                            [],
                        )
                    })
                    .expect("chen hang cu");
            }
        }
        let fx = Fixture {
            global: Store::open(StoreSpec::global(dir.join("global.db"))).expect("global moi"),
            work: Store::open(StoreSpec::project(dir.join("project.db"))).expect("project moi"),
            dir,
            tier,
        };
        assert_eq!(
            (fx.global.schema_version(), fx.work.schema_version()),
            (15, 40)
        );
        let id = fx.row_id("慕容");
        assert_eq!(fx.row(id).1, None);
        let err = fx
            .unconfirm(id, "Mộ Dung", "2026-10-18T00:00:00.000Z")
            .unwrap_err();
        assert_eq!(code_of(err), "glossary.not_unconfirmable", "{tier:?}");
        fx.finish();
    }
}

impl Fixture {
    fn row_id(&self, term: &str) -> i64 {
        load_tier(self.store()).expect("nap tang")[term].id
    }
}

#[test]
fn a_missing_row_and_a_promoted_row_report_entry_missing() {
    for tier in TIERS {
        let fx = Fixture::new("missing", tier);
        let err = fx.unconfirm(9999, "x", "y").unwrap_err();
        assert_eq!(code_of(err), "glossary.entry_missing", "{tier:?}");
        fx.finish();
    }

    let fx = Fixture::new("promoted", GlossaryTier::Work);
    let id = fx.add("慕容", None, "");
    let mark = fx.confirm(id, "Mộ Dung").expect("chot");
    promote_to_global(&fx.global, &fx.work, id).expect("day len Global");
    let err = fx.unconfirm(id, "Mộ Dung", &mark).unwrap_err();
    assert_eq!(code_of(err), "glossary.entry_missing");

    let global_id = fx.global_row_id("慕容");
    let err = unconfirm_pending_translation(
        &fx.global,
        Some(&fx.work),
        GlossaryTier::Global,
        global_id,
        "Mộ Dung",
        &mark,
    )
    .unwrap_err();
    assert_eq!(
        code_of(err),
        "glossary.not_unconfirmable",
        "ban Global khong co moc"
    );
    fx.finish();
}

impl Fixture {
    fn global_row_id(&self, term: &str) -> i64 {
        load_tier(&self.global).expect("nap global")[term].id
    }
}

#[test]
fn editing_only_note_or_category_keeps_the_mark_and_unconfirm_still_works() {
    for tier in TIERS {
        let fx = Fixture::new("note-only", tier);
        let id = fx.add("慕容", None, "");
        let mark = fx.confirm(id, "Mộ Dung").expect("chot");
        update_manual_term(
            &fx.global,
            Some(&fx.work),
            tier,
            id,
            Some("Mộ Dung"),
            "ghi chu moi",
            Category::Place,
        )
        .expect("sua note");
        let (_, stored, note, category) = fx.row(id);
        assert_eq!(stored.as_deref(), Some(mark.as_str()), "{tier:?}");
        assert_eq!((note.as_str(), category.as_str()), ("ghi chu moi", "place"));

        fx.unconfirm(id, "Mộ Dung", &mark).expect("go chot");
        let (translation, stored, note, category) = fx.row(id);
        assert_eq!((translation, stored), (None, None), "{tier:?}");
        assert_eq!((note.as_str(), category.as_str()), ("ghi chu moi", "place"));
        fx.finish();
    }
}

#[test]
fn a_bare_update_to_null_is_still_aborted_on_a_marked_row() {
    for tier in TIERS {
        let fx = Fixture::new("bare-null", tier);
        let id = fx.add("慕容", None, "");
        fx.confirm(id, "Mộ Dung").expect("chot");
        let before = fx.row(id);
        let err = fx
            .store()
            .write(move |tx: &Transaction<'_>| {
                tx.execute(
                    "UPDATE glossary_entry SET translation = NULL WHERE id = ?1",
                    [id],
                )
            })
            .expect_err("UPDATE translation = NULL tran phai bi chan");
        assert!(
            err.to_string().contains("one-way"),
            "{tier:?}: loi kho tho, khong anh xa kieu: {err}"
        );
        assert_eq!(fx.row(id), before, "{tier:?}");
        fx.finish();
    }
}

#[test]
fn a_mark_cannot_be_set_on_a_row_that_is_already_confirmed() {
    for tier in TIERS {
        let fx = Fixture::new("mark-on-confirmed", tier);
        let id = fx.add("慕容", Some("Mộ Dung"), "");
        let result = fx.store().write(move |tx: &Transaction<'_>| {
            tx.execute(
                "UPDATE glossary_entry SET pending_confirmed_at = 'x' WHERE id = ?1",
                [id],
            )
        });
        assert!(
            result.is_err(),
            "{tier:?}: NULL -> khac NULL tren muc da chot phai ABORT"
        );
        assert_eq!(fx.row(id).1, None, "{tier:?}");
        fx.finish();
    }
}

#[test]
fn every_other_writer_of_translation_runs_on_a_marked_row_and_clears_the_mark() {
    for tier in TIERS {
        let fx = Fixture::new("other-writers", tier);

        let manual = fx.add("慕容", None, "");
        fx.confirm(manual, "Mộ Dung").expect("chot");
        update_manual_term(
            &fx.global,
            Some(&fx.work),
            tier,
            manual,
            Some("Mộ Dong"),
            "",
            Category::Person,
        )
        .expect("update_manual_term tren hang co moc");
        assert_eq!(fx.row(manual).1, None, "{tier:?}: update_manual_term");

        if tier == GlossaryTier::Work {
            let harvested = fx.add("青丘", None, "");
            fx.confirm(harvested, "Thanh Khâu").expect("chot");
            assert!(fx.row(harvested).1.is_some());
            enqueue_review_harvest(
                &fx.work,
                &[ReviewHarvestProposal {
                    source_term: "青丘".to_owned(),
                    detail: ReviewHarvestDetail {
                        replaced_translation: "Thanh Khâu".to_owned(),
                        proposed_translation: "Thanh Khâu moi".to_owned(),
                        changed_count: 2,
                        seen_count: 2,
                    },
                }],
            )
            .expect("xep ung vien thu hoach");
            let candidate = pending_candidates(&fx.work)
                .expect("bang cho")
                .into_iter()
                .find(|c| c.source_term == "青丘")
                .expect("ung vien")
                .id;
            approve_candidate(&fx.work, candidate, None, Category::Person)
                .expect("approve_candidate tren hang co moc");
            let (translation, mark, _, _) = fx.row(harvested);
            assert_eq!(translation.as_deref(), Some("Thanh Khâu moi"), "{tier:?}");
            assert_eq!(mark, None, "{tier:?}: approve_candidate");
        }
        fx.finish();
    }
}

#[test]
fn an_import_take_theirs_over_a_marked_row_clears_the_mark() {
    use auratranslate_lib::core::glossary::{
        ConflictDecision, ImportRow, classify, import_into_tier,
    };
    use std::collections::BTreeMap;

    for tier in TIERS {
        let fx = Fixture::new("take-theirs", tier);
        let id = fx.add("慕容", None, "");
        fx.confirm(id, "Mộ Dung").expect("chot");
        assert!(fx.row(id).1.is_some());

        let existing = load_tier(fx.store()).expect("nap tang");
        let row = ImportRow {
            line: 2,
            created_at: None,
            source_term: "慕容".to_owned(),
            translation: Some("Mộ Dong".to_owned()),
            note: String::new(),
            category: Category::Person,
        };
        let plans = classify(&[row], &existing);
        let mut decisions = BTreeMap::new();
        decisions.insert("慕容".to_owned(), ConflictDecision::TakeTheirs);
        import_into_tier(&fx.global, Some(&fx.work), tier, &plans, &decisions)
            .expect("TakeTheirs tren hang co moc");

        let (translation, mark, _, _) = fx.row(id);
        assert_eq!(translation.as_deref(), Some("Mộ Dong"), "{tier:?}");
        assert_eq!(mark, None, "{tier:?}: TakeTheirs");
        fx.finish();
    }
}

#[test]
fn confirming_and_unconfirming_never_touch_a_segment_target() {
    let fx = Fixture::new("segments", GlossaryTier::Work);
    fx.work
        .write(|tx: &Transaction<'_>| {
            tx.execute(
                "INSERT INTO work (id, work_id, name, source_lang, genre, created_at, updated_at) \
                 VALUES (1, 'w', 'W', 'zh', '', 'x', 'x')",
                (),
            )?;
            tx.execute(
                "INSERT INTO chapter (ord, title, source_text, status, created_at, updated_at) \
                 VALUES (1, NULL, '慕容', 'not_started', 'x', 'x')",
                (),
            )?;
            let chapter_id = tx.last_insert_rowid();
            tx.execute(
                "INSERT INTO segment (chapter_id, ord, source_text, is_paragraph_end, \
                 is_target_paragraph_end, target_text, translation_origin, created_at, updated_at) \
                 VALUES (?1, 1, '慕容', 1, 0, 'Mộ Dung noi.', 'self', 'x', 'x')",
                [chapter_id],
            )
        })
        .expect("chen segment");
    let snapshot = |fx: &Fixture| -> Vec<(i64, String, String)> {
        fx.work
            .read(|conn| {
                let mut stmt = conn.prepare(
                    "SELECT id, target_text, translation_origin FROM segment ORDER BY id",
                )?;
                stmt.query_map([], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)))?
                    .collect()
            })
            .expect("doc segment")
    };
    let before = snapshot(&fx);
    assert_eq!(before.len(), 1);

    let id = fx.add("慕容", None, "");
    let mark = fx.confirm(id, "Mộ Dung").expect("chot");
    fx.unconfirm(id, "Mộ Dung", &mark).expect("go chot");
    assert_eq!(snapshot(&fx), before);
    fx.finish();
}

fn open_with_extra_step(kind: StoreKind, sql: &'static str) -> Result<Store, StoreError> {
    let dir = temp_dir("lifecycle-check");
    let (base, db, spec): (&'static [Migration], _, fn(PathBuf) -> StoreSpec) = match kind {
        StoreKind::Global => (GLOBAL_MIGRATIONS, "global.db", StoreSpec::global),
        _ => (PROJECT_MIGRATIONS, "project.db", StoreSpec::project),
    };
    let last = base.last().expect("ladder").to_version;
    let mut ladder = base.to_vec();
    ladder.push(Migration {
        to_version: last + 1,
        sql,
    });
    let result = Store::open(StoreSpec {
        migrations: Box::leak(ladder.into_boxed_slice()),
        ..spec(dir.join(db))
    });
    let result = result.map(|store| {
        drop(store);
    });
    cleanup(&dir);
    result.map(|()| unreachable!("the check must refuse"))
}

#[test]
fn opening_a_store_whose_later_step_lost_part_of_the_lifecycle_is_refused_with_the_missing_names() {
    const DROP_AFTER: &str = "DROP TRIGGER glossary_entry_edit_clears_unconfirm;";
    const OLD_BODY: &str = "DROP TRIGGER glossary_entry_edit_clears_unconfirm; \
DROP TRIGGER glossary_entry_lifecycle_is_one_way; \
CREATE TRIGGER glossary_entry_lifecycle_is_one_way \
BEFORE UPDATE OF translation ON glossary_entry \
WHEN OLD.translation IS NOT NULL AND NEW.translation IS NULL \
BEGIN SELECT RAISE(ABORT, 'glossary lifecycle is one-way'); END; \
CREATE TRIGGER glossary_entry_edit_clears_unconfirm AFTER UPDATE OF translation ON glossary_entry \
WHEN OLD.translation IS NOT NULL AND NEW.translation IS NOT OLD.translation \
  AND NEW.pending_confirmed_at IS NOT NULL \
BEGIN UPDATE glossary_entry SET pending_confirmed_at = NULL WHERE id = NEW.id; END;";
    const NO_COLUMN: &str = "DROP TRIGGER glossary_entry_edit_clears_unconfirm; \
DROP TRIGGER glossary_entry_lifecycle_is_one_way; \
ALTER TABLE glossary_entry DROP COLUMN pending_confirmed_at; \
CREATE TRIGGER glossary_entry_lifecycle_is_one_way \
BEFORE UPDATE OF translation ON glossary_entry \
WHEN OLD.translation IS NOT NULL AND NEW.translation IS NULL \
BEGIN SELECT RAISE(ABORT, 'glossary lifecycle is one-way'); END;";

    let cases: [(&str, &str, &[&str]); 3] = [
        (
            "dropped AFTER trigger",
            DROP_AFTER,
            &["glossary_entry_edit_clears_unconfirm"],
        ),
        (
            "old one-way body",
            OLD_BODY,
            &["glossary_entry_lifecycle_is_one_way"],
        ),
        (
            "no column",
            NO_COLUMN,
            &[
                "pending_confirmed_at",
                "glossary_entry_lifecycle_is_one_way",
                "glossary_entry_edit_clears_unconfirm",
            ],
        ),
    ];
    for kind in [StoreKind::Global, StoreKind::Project] {
        for (label, sql, expected) in cases {
            let err = open_with_extra_step(kind, sql)
                .err()
                .unwrap_or_else(|| panic!("{kind:?} / {label}: the store must refuse to open"));
            match err {
                StoreError::GlossaryLifecycleMissing { store, missing } => {
                    assert_eq!(store, kind, "{label}");
                    assert_eq!(missing, expected.join(", "), "{kind:?} / {label}");
                }
                other => panic!("{kind:?} / {label}: wrong error {other:?}"),
            }
        }
    }
}
