//! Kind migration (global step 16, project step 43): build at latest, downgrade by SQL, reopen.

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use auratranslate_lib::commands::project::{OpenWork, create_work, open_work};
use auratranslate_lib::commands::promptset::{PromptSetWire, prompt_set_create, prompt_set_list};
use auratranslate_lib::core::library::indexer::IndexedWork;
use auratranslate_lib::core::promptset::{PromptSetKind, PromptSetTier};
use auratranslate_lib::core::segment::pipeline::{ChapterInput, PipelineShape};
use auratranslate_lib::core::store::{Store, StoreSpec, Transaction};

static NEXT_DIR: AtomicU64 = AtomicU64::new(0);

const DOWNGRADE_PROMPT_SET_KIND: &str = "\
DROP INDEX idx_prompt_set_kind_name; \
CREATE UNIQUE INDEX idx_prompt_set_name ON prompt_set (name); \
ALTER TABLE prompt_set DROP COLUMN kind;";

fn temp_dir(tag: &str) -> PathBuf {
    let n = NEXT_DIR.fetch_add(1, Ordering::Relaxed);
    let dir = std::env::temp_dir().join(format!(
        "auratranslate-promptset-kind-migration-{}-{}-{}",
        std::process::id(),
        tag,
        n
    ));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap_or_else(|e| panic!("tao {}: {e}", dir.display()));
    dir
}

fn cleanup_dir(dir: &Path) {
    let _ = fs::remove_dir_all(dir);
}

fn open_work_real(documents_root: &Path) -> OpenWork {
    create_work(
        documents_root,
        "Work Stub",
        "en",
        "",
        PipelineShape::Blob(ChapterInput::AlreadyText("noi dung".to_owned())),
        encoding_rs::UTF_8,
        Vec::new(),
        None,
        Vec::new(),
        0,
        1,
        false,
        &[],
        &std::sync::Mutex::new(Vec::new()),
        None,
        &[],
    )
    .expect("tao OpenWork that bai")
}

fn reopen_work(open: OpenWork) -> OpenWork {
    let indexed = IndexedWork {
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
    };
    drop(open);
    open_work(&indexed.work_id, Some(&indexed)).expect("mo lai tac pham")
}

fn names(sets: &[PromptSetWire]) -> Vec<(&str, &str)> {
    sets.iter()
        .map(|s| (s.name.as_str(), s.body.as_str()))
        .collect()
}

#[test]
fn a_global_store_from_step_15_keeps_its_sets_as_translate_and_accepts_a_proofread_twin() {
    let root = temp_dir("global");
    let path = root.join("global.db");
    {
        let store = Store::open(StoreSpec::global(path.clone())).expect("mo global.db");
        prompt_set_create(
            Some(&store),
            None,
            PromptSetTier::Global,
            PromptSetKind::Translate,
            "A",
            "than cu",
        )
        .expect("tao bo");
        assert_eq!(store.schema_version(), 18, "kho moi phai co buoc 18");
        store
            .write(|tx: &Transaction<'_>| {
                tx.execute_batch(&format!(
                    "{DOWNGRADE_PROMPT_SET_KIND} \
                     DROP TABLE convention_rule; \
                     DROP TABLE genre; \
                     DELETE FROM schema_migration_log WHERE version IN (16, 17, 18); \
                     PRAGMA user_version = 15;"
                ))
            })
            .expect("ha ve buoc 15");
    }

    let store = Store::open(StoreSpec::global(path)).expect("mo lai global.db");
    assert_eq!(store.schema_version(), 18);
    let translate = prompt_set_list(Some(&store), None, PromptSetKind::Translate).unwrap();
    assert_eq!(names(&translate.sets), [("A", "than cu")]);
    assert!(
        prompt_set_list(Some(&store), None, PromptSetKind::Proofread)
            .unwrap()
            .sets
            .is_empty()
    );
    prompt_set_create(
        Some(&store),
        None,
        PromptSetTier::Global,
        PromptSetKind::Proofread,
        "A",
        "chi dan",
    )
    .expect("bo proofread cung ten phai chen duoc");
    let proofread = prompt_set_list(Some(&store), None, PromptSetKind::Proofread).unwrap();
    assert_eq!(names(&proofread.sets), [("A", "chi dan")]);

    drop(store);
    cleanup_dir(&root);
}

#[test]
fn a_project_store_from_step_42_gains_kind_and_the_proofread_columns_with_neutral_values() {
    let root = temp_dir("project");
    let global = Store::open(StoreSpec::global(root.join("global.db"))).expect("mo global.db");
    let open = open_work_real(&root);
    prompt_set_create(
        Some(&global),
        Some(&open),
        PromptSetTier::Work,
        PromptSetKind::Translate,
        "A",
        "than cu",
    )
    .expect("tao bo Work");
    assert_eq!(open.store.schema_version(), 45, "kho moi phai co buoc 45");
    open.store
        .write(|tx: &Transaction<'_>| {
            tx.execute_batch(&format!(
                "{DOWNGRADE_PROMPT_SET_KIND} \
                 ALTER TABLE work DROP COLUMN chosen_proofread_prompt_set_name; \
                 ALTER TABLE work DROP COLUMN proofread_instruction_off; \
                 ALTER TABLE proof_scan DROP COLUMN instruction_set_name; \
                 ALTER TABLE proof_scan DROP COLUMN instruction_set_tier; \
                 INSERT INTO proof_scan (id, scope, scanned_at, reported, filtered) VALUES (1, 'chapter', '2026-01-01T00:00:00Z', 0, 0); \
                 DROP TABLE convention_ignore; DROP TABLE convention_rule; DELETE FROM schema_migration_log WHERE version IN (43, 44, 45); \
                 PRAGMA user_version = 42;"
            ))
        })
        .expect("ha ve buoc 42");
    let open = reopen_work(open);

    assert_eq!(open.store.schema_version(), 45);
    let translate = prompt_set_list(Some(&global), Some(&open), PromptSetKind::Translate).unwrap();
    assert_eq!(names(&translate.sets), [("A", "than cu")]);
    prompt_set_create(
        Some(&global),
        Some(&open),
        PromptSetTier::Work,
        PromptSetKind::Proofread,
        "A",
        "chi dan",
    )
    .expect("bo proofread cung ten phai chen duoc");

    let (off, name): (i64, Option<String>) = open
        .store
        .read(|conn| {
            conn.query_row(
                "SELECT proofread_instruction_off, chosen_proofread_prompt_set_name FROM work WHERE id = 1",
                [],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
        })
        .expect("doc hang work");
    assert_eq!((off, name), (0, None));
    let scan: (Option<String>, Option<String>) = open
        .store
        .read(|conn| {
            conn.query_row(
                "SELECT instruction_set_name, instruction_set_tier FROM proof_scan LIMIT 1",
                [],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
        })
        .expect("doc hang proof_scan");
    assert_eq!(scan, (None, None));

    drop(open);
    drop(global);
    cleanup_dir(&root);
}
