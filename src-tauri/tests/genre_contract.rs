//! Genre list contract: the seeded table, name rules, rename, and the in-use delete guard.

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use auratranslate_lib::commands::genre::{
    GenreWire, genre_create, genre_delete, genre_list, genre_rename,
};
use auratranslate_lib::commands::promptset::prompt_set_list;
use auratranslate_lib::core::i18n::MessageKey;
use auratranslate_lib::core::library::indexer::Indexer;
use auratranslate_lib::core::library::meta::{META_SCHEMA_VERSION, WorkMeta};
use auratranslate_lib::core::promptset::{PromptSetKind, PromptSetTier, create};
use auratranslate_lib::core::store::{Store, StoreSpec, Transaction};

static NEXT_DIR: AtomicU64 = AtomicU64::new(0);

fn temp_dir(tag: &str) -> PathBuf {
    let n = NEXT_DIR.fetch_add(1, Ordering::Relaxed);
    let dir = std::env::temp_dir().join(format!(
        "auratranslate-genre-{}-{}-{}",
        std::process::id(),
        tag,
        n
    ));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap_or_else(|e| panic!("tao {}: {e}", dir.display()));
    dir
}

fn open_global(dir: &Path) -> Store {
    Store::open(StoreSpec::global(dir.join("global.db"))).expect("mo global.db")
}

fn names(rows: &[GenreWire]) -> Vec<&str> {
    rows.iter().map(|g| g.name.as_str()).collect()
}

fn list(store: &Store) -> Vec<GenreWire> {
    genre_list(Some(store)).expect("liet ke the loai")
}

fn id_of(rows: &[GenreWire], name: &str) -> String {
    rows.iter()
        .find(|g| g.name == name)
        .unwrap_or_else(|| panic!("khong co the loai {name}"))
        .id
        .clone()
}

fn write_work_with_genre(root: &Path, folder: &str, work_id: &str, genre: &str) {
    let dir = root.join(format!("{folder}.atproj"));
    fs::create_dir_all(&dir).expect("tao .atproj");
    let meta = WorkMeta {
        meta_schema_version: META_SCHEMA_VERSION,
        work_id: work_id.to_owned(),
        name: folder.to_owned(),
        source_lang: "en".to_owned(),
        genre: genre.to_owned(),
        created_at: "2026-08-01T00:00:00.000Z".to_owned(),
        updated_at: "2026-08-01T00:00:00.000Z".to_owned(),
        chapter_count: 1,
        status: Some("not_started".to_owned()),
        status_is_override: false,
        chapter_done_count: Some(0),
    };
    meta.write_atomic(&dir).expect("ghi meta.json");
    fs::write(dir.join("project.db"), b"not a real sqlite file").expect("ghi project.db gia");
}

#[test]
fn a_fresh_global_store_seeds_29_genres_in_the_old_menu_order() {
    let dir = temp_dir("seed");
    let store = open_global(&dir);
    let rows = list(&store);
    assert_eq!(rows.len(), 29);
    let ids: Vec<&str> = rows.iter().map(|g| g.id.as_str()).collect();
    assert_eq!(&ids[..3], ["news", "commentary", "tcm"]);
    assert_eq!(ids[28], "other");
    assert_eq!(rows[0].name, "Thời sự");
    assert_eq!(rows[15].id, "xianxia");
    assert_eq!(rows[15].name, "Tiên hiệp");
    assert_eq!(rows[28].name, "Khác");
    drop(store);
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn a_created_genre_lands_last_with_a_generated_id() {
    let dir = temp_dir("create");
    let store = open_global(&dir);
    let made = genre_create(Some(&store), "  Truyện ngắn  ").expect("tao");
    assert_eq!(made.name, "Truyện ngắn");
    let rows = list(&store);
    assert_eq!(rows.len(), 30);
    assert_eq!(rows[29], made);
    assert!(rows[..29].iter().all(|g| g.id != made.id));
    drop(store);
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn a_deleted_genre_id_is_never_issued_again() {
    let dir = temp_dir("reuse");
    let store = open_global(&dir);
    let indexer = Indexer::open(dir.join("library-index.db")).expect("mo indexer");
    let first = genre_create(Some(&store), "Một").expect("tao");
    genre_delete(Some(&store), Some(&indexer), &first.id).expect("xoa");
    let second = genre_create(Some(&store), "Hai").expect("tao");
    assert_ne!(first.id, second.id);
    drop(indexer);
    drop(store);
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn an_empty_name_is_refused() {
    let dir = temp_dir("empty");
    let store = open_global(&dir);
    let err = genre_create(Some(&store), "   ").expect_err("ten rong");
    assert_eq!(err.message_key(), MessageKey::GenreNameEmpty);
    assert_eq!(list(&store).len(), 29);
    let id = list(&store)[0].id.clone();
    let err = genre_rename(Some(&store), &id, "\u{3000} ").expect_err("ten rong");
    assert_eq!(err.message_key(), MessageKey::GenreNameEmpty);
    drop(store);
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn a_name_equal_ignoring_unicode_case_is_refused_on_create_and_rename() {
    let dir = temp_dir("taken");
    let store = open_global(&dir);
    let err = genre_create(Some(&store), "tiên hiệp").expect_err("trung");
    assert_eq!(err.message_key(), MessageKey::GenreNameTaken);
    assert_eq!(list(&store).len(), 29);

    let rows = list(&store);
    let news = id_of(&rows, "Thời sự");
    let err = genre_rename(Some(&store), &news, "đô thị").expect_err("trung Dh/dh");
    assert_eq!(err.message_key(), MessageKey::GenreNameTaken);
    assert_eq!(names(&list(&store))[0], "Thời sự");
    drop(store);
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn renaming_a_genre_to_its_own_name_in_another_case_is_allowed() {
    let dir = temp_dir("selfcase");
    let store = open_global(&dir);
    let id = id_of(&list(&store), "Thời sự");
    genre_rename(Some(&store), &id, "THỜI SỰ").expect("doi hoa thuong");
    assert_eq!(list(&store)[0].name, "THỜI SỰ");
    drop(store);
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn a_seeded_genre_renames_in_place_and_keeps_its_id_and_position() {
    let dir = temp_dir("rename");
    let store = open_global(&dir);
    genre_rename(Some(&store), "news", "Tin tức").expect("doi ten");
    let rows = list(&store);
    assert_eq!(rows[0].id, "news");
    assert_eq!(rows[0].name, "Tin tức");
    let err = genre_rename(Some(&store), "missing", "X").expect_err("khong co");
    assert_eq!(err.message_key(), MessageKey::GenreNotFound);
    drop(store);
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn a_genre_in_use_is_not_deleted_and_the_error_carries_the_work_count() {
    let dir = temp_dir("inuse");
    let store = open_global(&dir);
    let indexer = Indexer::open(dir.join("library-index.db")).expect("mo indexer");
    let root = dir.join("library");
    write_work_with_genre(&root, "a", "work-a", "news");
    write_work_with_genre(&root, "b", "work-b", "news");
    write_work_with_genre(&root, "c", "work-c", "");
    indexer.rebuild(&root, Some(&store)).expect("lap chi muc");

    let err = genre_delete(Some(&store), Some(&indexer), "news").expect_err("dang dung");
    assert_eq!(err.message_key(), MessageKey::GenreInUse);
    assert_eq!(err.params().get("count").map(String::as_str), Some("2"));
    assert_eq!(list(&store).len(), 29);

    genre_delete(Some(&store), Some(&indexer), "wuxia").expect("xoa the loai khong dung");
    let rows = list(&store);
    assert_eq!(rows.len(), 28);
    assert!(rows.iter().all(|g| g.id != "wuxia"));
    drop(indexer);
    drop(store);
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn deleting_without_a_library_index_is_an_error_not_a_silent_zero_count() {
    let dir = temp_dir("noindex");
    let store = open_global(&dir);
    let err = genre_delete(Some(&store), None, "wuxia").expect_err("khong co chi muc");
    assert_eq!(err.message_key(), MessageKey::StoreOpenFailed);
    assert_eq!(list(&store).len(), 29);
    drop(store);
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn a_missing_global_store_is_an_error() {
    let err = genre_list(None).expect_err("khong co kho");
    assert_eq!(err.message_key(), MessageKey::StoreOpenFailed);
}

#[test]
fn a_global_store_stopped_at_step_17_migrates_to_18_with_the_seed_and_keeps_its_rows() {
    let dir = temp_dir("from17");
    let path = dir.join("global.db");
    {
        let store = Store::open(StoreSpec::global(path.clone())).expect("mo global.db");
        create(
            &store,
            None,
            PromptSetTier::Global,
            PromptSetKind::Translate,
            "Bo cu",
            "than cu",
        )
        .expect("tao bo");
        store
            .write(|tx: &Transaction<'_>| {
                tx.execute_batch(
                    "DROP TABLE genre; \
                     DELETE FROM schema_migration_log WHERE version = 18; \
                     PRAGMA user_version = 17;",
                )
            })
            .expect("ha ve buoc 17");
    }

    let store = Store::open(StoreSpec::global(path)).expect("mo lai global.db");
    assert_eq!(store.schema_version(), 18);
    let rows = list(&store);
    assert_eq!(rows.len(), 29);
    assert_eq!(
        (rows[0].id.as_str(), rows[0].name.as_str()),
        ("news", "Thời sự")
    );
    assert_eq!(
        (rows[28].id.as_str(), rows[28].name.as_str()),
        ("other", "Khác")
    );
    let sets = prompt_set_list(Some(&store), None, PromptSetKind::Translate).expect("liet ke");
    assert_eq!(sets.sets.len(), 1);
    assert_eq!(sets.sets[0].name, "Bo cu");
    drop(store);
    let _ = fs::remove_dir_all(&dir);
}
