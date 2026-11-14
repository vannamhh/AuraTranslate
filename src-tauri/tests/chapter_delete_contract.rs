//! Contract for deleting a Chapter: outright when nothing in it is worth keeping, soft (row moved
//! to `chapter_deleted`, segments retired) otherwise, and the Work folder removed only when
//! neither table holds a Chapter any more.

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use auratranslate_lib::commands::aiprompt::AiCallRecordsState;
use auratranslate_lib::commands::chapter::{
    RemovedWork, complete_chapter_delete, delete_chapter, delete_chapter_in_app,
    delete_chapter_indexed, delete_chapter_preview, list_chapters, read_open_chapter,
};
use auratranslate_lib::commands::lifecycle::reindex_after_lifecycle_write;
use auratranslate_lib::commands::project::{
    OpenWork, OpenWorkState, create_work_from_text, open_work,
};
use auratranslate_lib::commands::segment::{list_reading_marks, read_open_chapter_segments};
use auratranslate_lib::commands::tm::{PendingTmxImport, PendingTmxImportState};
use auratranslate_lib::core::chapter_delete::Disposition;
use auratranslate_lib::core::export::{
    ExportScope, count_scope, plan_import, read_markdown_copy, resolve_chapter_ids,
};
use auratranslate_lib::core::i18n::MessageKey;
use auratranslate_lib::core::library::delete_work_folder;
use auratranslate_lib::core::library::indexer::{IndexedWork, Indexer, WorkQuery};
use auratranslate_lib::core::library::meta::WorkMeta;
use auratranslate_lib::core::scope::save_value;
use auratranslate_lib::core::store::{PROJECT_MIGRATIONS, Store, StoreSpec, Transaction};
use auratranslate_lib::core::tm::TmTier;
use tauri::Manager as _;
use tauri::test::{mock_builder, mock_context, noop_assets};

static NEXT_DIR: AtomicU64 = AtomicU64::new(0);

fn temp_dir(tag: &str) -> PathBuf {
    let n = NEXT_DIR.fetch_add(1, Ordering::Relaxed);
    let dir = std::env::temp_dir().join(format!(
        "auratranslate-chapter-delete-{}-{}-{}",
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

fn count(store: &Store, sql: &'static str) -> i64 {
    store
        .read(move |conn| conn.query_row(sql, [], |row| row.get(0)))
        .expect("count")
}

fn insert_chapter(opened: &OpenWork, ord: i64, title: &str) -> i64 {
    let title = title.to_owned();
    opened
        .store
        .write(move |tx: &Transaction<'_>| {
            tx.execute(
                "INSERT INTO chapter (ord, title, source_text, status, created_at, updated_at) \
                 VALUES (?1, ?2, 'nguon', 'not_started', strftime('%Y-%m-%dT%H:%M:%fZ','now'), \
                 strftime('%Y-%m-%dT%H:%M:%fZ','now'))",
                (ord, &title),
            )?;
            Ok(tx.last_insert_rowid())
        })
        .expect("insert chapter")
}

fn insert_segment(opened: &OpenWork, chapter_id: i64, ord: i64, target: &str) -> i64 {
    let target = target.to_owned();
    opened
        .store
        .write(move |tx: &Transaction<'_>| {
            tx.execute(
                "INSERT INTO segment (chapter_id, ord, source_text, target_text, \
                 is_paragraph_end, is_target_paragraph_end, retired_at, status, is_omitted, \
                 translation_origin, created_at, updated_at) \
                 VALUES (?1, ?2, 'nguon', ?3, 0, 0, NULL, 'draft', 0, '', \
                 '2026-11-14T00:00:00.000Z', '2026-11-14T00:00:00.000Z')",
                (chapter_id, ord, &target),
            )?;
            Ok(tx.last_insert_rowid())
        })
        .expect("insert segment")
}

fn exec(opened: &OpenWork, sql: &'static str, id: i64) {
    opened
        .store
        .write(move |tx: &Transaction<'_>| tx.execute(sql, [id]))
        .expect("exec");
}

/// Rows keyed by a Chapter or its segments: asset, position, review copy, alignment, marks.
fn attach_dependents(opened: &OpenWork, chapter_id: i64, segment_id: i64) {
    opened
        .store
        .write(move |tx: &Transaction<'_>| {
            tx.execute(
                "INSERT INTO asset (chapter_id, file_name, source_url, anchor_after_segment_ord, \
                 byte_len, content_type, created_at) VALUES (?1, 'a.jpg', NULL, 0, 10, \
                 'image/jpeg', '2026-11-14T00:00:00.000Z')",
                [chapter_id],
            )?;
            tx.execute(
                "INSERT INTO chapter_position (chapter_id, segment_id, updated_at) \
                 VALUES (?1, ?2, '2026-11-14T00:00:00.000Z')",
                (chapter_id, segment_id),
            )?;
            tx.execute(
                "INSERT INTO review_chapter (chapter_id, file_name, file_kind, imported_at) \
                 VALUES (?1, 'r.md', 'md', '2026-11-14T00:00:00.000Z')",
                [chapter_id],
            )?;
            let review_chapter_id = tx.last_insert_rowid();
            tx.execute(
                "INSERT INTO review_row (review_chapter_id, ord, kind, source_text, target_text) \
                 VALUES (?1, 1, 'text', NULL, 'x')",
                [review_chapter_id],
            )?;
            tx.execute(
                "INSERT INTO alignment_group (review_chapter_id, decided_by) VALUES (?1, 'machine')",
                [review_chapter_id],
            )?;
            let group_id = tx.last_insert_rowid();
            tx.execute(
                "INSERT INTO alignment_member (group_id, review_row_id, segment_id) \
                 VALUES (?1, NULL, ?2)",
                (group_id, segment_id),
            )?;
            tx.execute(
                "INSERT INTO reading_mark (segment_id, navigation_segment_id, marked_at) \
                 VALUES (?1, ?1, '2026-11-14T00:00:00.000Z')",
                [segment_id],
            )?;
            tx.execute(
                "INSERT INTO segment_unlinked (segment_id, created_at) \
                 VALUES (?1, '2026-11-14T00:00:00.000Z')",
                [segment_id],
            )?;
            Ok(())
        })
        .expect("attach dependents");
}

fn attach_version(opened: &OpenWork, segment_id: i64) {
    exec(
        opened,
        "INSERT INTO segment_version (segment_id, target_text, translation_origin, created_at) \
         VALUES (?1, 'v', 'self', '2026-11-14T00:00:00.000Z')",
        segment_id,
    );
}

fn work_with(root: &Path, name: &str) -> Option<OpenWork> {
    Some(
        create_work_from_text(root, name, "zh", "", "Chuong mot.".to_owned())
            .expect("tao tac pham"),
    )
}

fn first(open: &OpenWork) -> i64 {
    open.chapter_id.expect("Tac pham vua tao co Chuong dang mo")
}

fn ords(store: &Store) -> Vec<(i64, i64)> {
    store
        .read(|conn| {
            let mut stmt = conn.prepare("SELECT id, ord FROM chapter ORDER BY ord, id")?;
            let rows = stmt.query_map([], |row| Ok((row.get(0)?, row.get(1)?)))?;
            rows.collect()
        })
        .expect("ords")
}

fn last_chapter_id(store: &Store) -> Option<i64> {
    store
        .read(|conn| {
            conn.query_row("SELECT last_chapter_id FROM work WHERE id = 1", [], |r| {
                r.get(0)
            })
        })
        .expect("last_chapter_id")
}

fn indexed_from(open: &OpenWork) -> IndexedWork {
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

fn open_index_pair(side: &Path) -> (Indexer, Store) {
    (
        Indexer::open(side.join("library-index.db")).expect("indexer"),
        Store::open(StoreSpec::global(side.join("global.db"))).expect("global"),
    )
}

fn close_removed(removed: Option<RemovedWork>) {
    if let Some(RemovedWork { open }) = removed {
        open.store.close();
    }
}

const DEPENDENT_TABLES: [&str; 8] = [
    "SELECT COUNT(*) FROM asset",
    "SELECT COUNT(*) FROM chapter_position",
    "SELECT COUNT(*) FROM review_chapter",
    "SELECT COUNT(*) FROM review_row",
    "SELECT COUNT(*) FROM alignment_group",
    "SELECT COUNT(*) FROM alignment_member",
    "SELECT COUNT(*) FROM reading_mark",
    "SELECT COUNT(*) FROM segment_unlinked",
];

#[test]
fn deleting_an_untranslated_chapter_removes_every_row_of_it_and_renumbers_the_rest() {
    let root = temp_dir("hard");
    let mut state = work_with(&root, "Xoa Han");
    let (a, b, c) = {
        let open = state.as_ref().expect("open");
        let a = first(open);
        let b = insert_chapter(open, 2, "B");
        let c = insert_chapter(open, 3, "C");
        insert_segment(open, a, 1, "");
        let sb = insert_segment(open, b, 1, "");
        insert_segment(open, b, 2, "   ");
        insert_segment(open, c, 1, "");
        attach_dependents(open, b, sb);
        (a, b, c)
    };

    let (deleted, removed) = delete_chapter(&mut state, b).expect("xoa");

    assert!(removed.is_none());
    assert_eq!(deleted.disposition, Disposition::Hard);
    assert!(!deleted.work_removed);
    let open = state.as_ref().expect("work con mo");
    assert_eq!(ords(&open.store), vec![(a, 1), (c, 2)]);
    assert_eq!(
        deleted
            .chapters
            .iter()
            .map(|r| (r.chapter_id, r.ord))
            .collect::<Vec<_>>(),
        vec![(a, 1), (c, 2)]
    );
    assert_eq!(
        count(
            &open.store,
            "SELECT COUNT(*) FROM segment WHERE chapter_id NOT IN (SELECT id FROM chapter)"
        ),
        0,
        "khong cau nao con tro vao Chuong da xoa"
    );
    for sql in DEPENDENT_TABLES {
        assert_eq!(count(&open.store, sql), 0, "{sql}");
    }
    assert_eq!(
        count(&open.store, "SELECT COUNT(*) FROM segment_version"),
        0
    );
    assert_eq!(
        count(&open.store, "SELECT COUNT(*) FROM chapter_deleted"),
        0
    );

    let dir = open.dir.clone();
    drop(state);
    cleanup(&dir);
    cleanup(&root);
}

#[test]
fn a_chapter_marked_done_but_holding_no_translation_is_still_removed_outright() {
    let root = temp_dir("done-untranslated");
    let mut state = work_with(&root, "Done Rong");
    let b = {
        let open = state.as_ref().expect("open");
        let b = insert_chapter(open, 2, "B");
        insert_segment(open, b, 1, "");
        exec(open, "UPDATE chapter SET status = 'done' WHERE id = ?1", b);
        b
    };

    let (deleted, _) = delete_chapter(&mut state, b).expect("xoa");

    assert_eq!(deleted.disposition, Disposition::Hard);
    let open = state.as_ref().expect("open");
    assert_eq!(
        count(&open.store, "SELECT COUNT(*) FROM chapter_deleted"),
        0
    );
    let dir = open.dir.clone();
    drop(state);
    cleanup(&dir);
    cleanup(&root);
}

#[test]
fn deleting_a_translated_chapter_moves_it_and_loses_no_translation_byte() {
    let root = temp_dir("soft");
    let mut state = work_with(&root, "Xoa Mem");
    let (a, b, c) = {
        let open = state.as_ref().expect("open");
        let a = first(open);
        let b = insert_chapter(open, 2, "Ten B");
        let c = insert_chapter(open, 3, "C");
        insert_segment(open, a, 1, "");
        let sb = insert_segment(open, b, 1, "ban dich mot");
        insert_segment(open, b, 2, "");
        insert_segment(open, c, 1, "");
        attach_dependents(open, b, sb);
        attach_version(open, sb);
        (a, b, c)
    };
    let (nonblank_before, versions_before) = {
        let store = &state.as_ref().expect("open").store;
        (
            count(
                store,
                "SELECT COUNT(*) FROM segment WHERE trim(target_text) <> ''",
            ),
            count(store, "SELECT COUNT(*) FROM segment_version"),
        )
    };

    let (deleted, removed) = delete_chapter(&mut state, b).expect("xoa");

    assert!(removed.is_none());
    assert_eq!(deleted.disposition, Disposition::Soft);
    let open = state.as_ref().expect("open");
    assert_eq!(ords(&open.store), vec![(a, 1), (c, 2)]);
    assert_eq!(
        count(
            &open.store,
            "SELECT COUNT(*) FROM segment WHERE trim(target_text) <> ''"
        ),
        nonblank_before,
        "0 byte target_text bi xoa"
    );
    assert_eq!(
        count(&open.store, "SELECT COUNT(*) FROM segment_version"),
        versions_before,
        "0 hang segment_version bi xoa"
    );
    assert_eq!(
        count(
            &open.store,
            "SELECT COUNT(*) FROM chapter_deleted WHERE title = 'Ten B' AND deleted_at <> ''"
        ),
        1
    );
    assert_eq!(
        count(
            &open.store,
            "SELECT COUNT(*) FROM segment WHERE target_text = 'ban dich mot' AND retired_at IS NOT NULL"
        ),
        1,
        "cau cua Chuong da xoa mem mang retired_at, chu nguyen ven"
    );
    assert_eq!(
        count(
            &open.store,
            "SELECT COUNT(*) FROM segment WHERE chapter_id NOT IN (SELECT id FROM chapter) AND retired_at IS NULL"
        ),
        0,
        "khong cau song nao con tro vao Chuong da xoa"
    );
    for sql in &DEPENDENT_TABLES[1..] {
        assert_eq!(count(&open.store, sql), 0, "{sql}");
    }
    assert_eq!(
        count(&open.store, DEPENDENT_TABLES[0]),
        1,
        "hang asset o lai: luot quet anh mo coi se xoa tep neu mat hang"
    );
    let listed = list_chapters(state.as_ref()).expect("list");
    assert_eq!(
        listed.iter().map(|r| r.chapter_id).collect::<Vec<_>>(),
        vec![a, c]
    );

    let work = resolve_chapter_ids(&open.store, &ExportScope::Work).expect("scope");
    assert_eq!(work, vec![a, c], "ban xuat khong thay Chuong da xoa");
    let counts = count_scope(&open.store, &ExportScope::Work).expect("count");
    assert_eq!(counts.chapter_count, 2);

    let dir = open.dir.clone();
    drop(state);
    cleanup(&dir);
    cleanup(&root);
}

#[test]
fn the_reading_mark_list_still_reads_after_a_soft_delete() {
    let root = temp_dir("soft-marks");
    let mut state = work_with(&root, "Dau Doc");
    let b = {
        let open = state.as_ref().expect("open");
        let b = insert_chapter(open, 2, "B");
        let sb = insert_segment(open, b, 1, "ban dich");
        let sa = insert_segment(open, first(open), 1, "");
        attach_dependents(open, b, sb);
        exec(
            open,
            "INSERT INTO reading_mark (segment_id, navigation_segment_id, marked_at) \
             VALUES (?1, ?1, '2026-11-14T00:00:00.000Z')",
            sa,
        );
        b
    };

    let (deleted, _) = delete_chapter(&mut state, b).expect("xoa mem");

    assert_eq!(deleted.disposition, Disposition::Soft);
    let marks = list_reading_marks(state.as_ref()).expect("danh sach dau doc van doc duoc");
    assert_eq!(marks.len(), 1, "chi con dau cua Chuong con lai");

    let dir = state.as_ref().expect("open").dir.clone();
    drop(state);
    cleanup(&dir);
    cleanup(&root);
}

/// One Chapter `b` next to the first, holding one segment built by `make`; returns the state, the
/// root and `b`.
fn chapter_with_hidden_content(
    tag: &str,
    make: impl FnOnce(&OpenWork, i64),
) -> (Option<OpenWork>, PathBuf, i64) {
    let root = temp_dir(tag);
    let state = work_with(&root, tag);
    let b = {
        let open = state.as_ref().expect("open");
        let b = insert_chapter(open, 2, "B");
        make(open, b);
        b
    };
    (state, root, b)
}

fn disposition_via_preview_and_delete(state: &mut Option<OpenWork>, b: i64) -> Disposition {
    let previewed = delete_chapter_preview(state.as_ref(), b)
        .expect("preview")
        .disposition;
    let (deleted, removed) = delete_chapter(state, b).expect("xoa");
    close_removed(removed);
    assert_eq!(
        previewed, deleted.disposition,
        "preview va xoa cung phep thu"
    );
    deleted.disposition
}

#[test]
fn a_translation_only_in_an_omitted_sentence_still_makes_the_delete_soft() {
    let (mut state, root, b) = chapter_with_hidden_content("omitted", |open, b| {
        let s = insert_segment(open, b, 1, "chu bi cat");
        exec(open, "UPDATE segment SET is_omitted = 1 WHERE id = ?1", s);
    });
    assert_eq!(
        disposition_via_preview_and_delete(&mut state, b),
        Disposition::Soft
    );
    let open = state.as_ref().expect("open");
    assert_eq!(
        count(
            &open.store,
            "SELECT COUNT(*) FROM segment WHERE target_text = 'chu bi cat'"
        ),
        1
    );
    let dir = open.dir.clone();
    drop(state);
    cleanup(&dir);
    cleanup(&root);
}

#[test]
fn a_translation_only_in_a_retired_sentence_still_makes_the_delete_soft() {
    let (mut state, root, b) = chapter_with_hidden_content("retired", |open, b| {
        let s = insert_segment(open, b, 1, "chu da ve huu");
        exec(
            open,
            "UPDATE segment SET retired_at = '2026-11-14T00:00:00.000Z' WHERE id = ?1",
            s,
        );
    });
    assert_eq!(
        disposition_via_preview_and_delete(&mut state, b),
        Disposition::Soft
    );
    let open = state.as_ref().expect("open");
    assert_eq!(
        count(
            &open.store,
            "SELECT COUNT(*) FROM segment WHERE target_text = 'chu da ve huu'"
        ),
        1
    );
    let dir = open.dir.clone();
    drop(state);
    cleanup(&dir);
    cleanup(&root);
}

#[test]
fn a_segment_version_alone_makes_the_delete_soft_and_survives_it() {
    let (mut state, root, b) = chapter_with_hidden_content("version-only", |open, b| {
        let s = insert_segment(open, b, 1, "");
        attach_version(open, s);
    });
    assert_eq!(
        disposition_via_preview_and_delete(&mut state, b),
        Disposition::Soft
    );
    let open = state.as_ref().expect("open");
    assert_eq!(
        count(&open.store, "SELECT COUNT(*) FROM segment_version"),
        1
    );
    let dir = open.dir.clone();
    drop(state);
    cleanup(&dir);
    cleanup(&root);
}

#[test]
fn a_target_of_only_unicode_white_space_is_not_worth_keeping() {
    let (mut state, root, b) = chapter_with_hidden_content("ideographic-blank", |open, b| {
        insert_segment(open, b, 1, "\u{3000}\u{00A0}\u{2003}");
    });
    assert_eq!(
        disposition_via_preview_and_delete(&mut state, b),
        Disposition::Hard
    );
    let open = state.as_ref().expect("open");
    assert_eq!(
        count(&open.store, "SELECT COUNT(*) FROM chapter_deleted"),
        0
    );
    let dir = open.dir.clone();
    drop(state);
    cleanup(&dir);
    cleanup(&root);
}

#[test]
fn the_preview_names_the_path_and_whether_the_work_goes_without_writing_a_row() {
    let root = temp_dir("preview");
    let state = work_with(&root, "Xem Truoc");
    let (only, b) = {
        let open = state.as_ref().expect("open");
        let only = first(open);
        insert_segment(open, only, 1, "");
        (only, insert_chapter(open, 2, "B"))
    };
    let open = state.as_ref().expect("open");
    let rows_before = count(&open.store, "SELECT COUNT(*) FROM segment");

    let middle = delete_chapter_preview(state.as_ref(), b).expect("preview b");
    assert_eq!(middle.disposition, Disposition::Hard);
    assert!(!middle.removes_work, "con Chuong khac");

    exec(open, "DELETE FROM chapter WHERE id = ?1", b);
    let last = delete_chapter_preview(state.as_ref(), only).expect("preview cuoi");
    assert_eq!(last.disposition, Disposition::Hard);
    assert!(last.removes_work, "Chuong cuoi, khong con gi de giu");

    insert_segment(open, only, 2, "da dich");
    let kept = delete_chapter_preview(state.as_ref(), only).expect("preview con gi de giu");
    assert_eq!(kept.disposition, Disposition::Soft);
    assert!(!kept.removes_work, "xoa mem khong bao gio xoa Tac pham");

    assert_eq!(
        count(&open.store, "SELECT COUNT(*) FROM chapter_deleted"),
        0,
        "xem truoc khong ghi hang nao"
    );
    assert_eq!(
        count(&open.store, "SELECT COUNT(*) FROM segment"),
        rows_before + 1
    );

    let err = delete_chapter_preview(state.as_ref(), 9_999).expect_err("id la");
    assert_eq!(err.message_key(), MessageKey::ChapterNotFound);
    assert_eq!(
        delete_chapter_preview(None, only)
            .expect_err("khong Tac pham")
            .code(),
        "work.none_open"
    );

    let dir = open.dir.clone();
    drop(state);
    cleanup(&dir);
    cleanup(&root);
}

#[test]
fn the_preview_says_the_work_stays_while_a_soft_deleted_chapter_remains() {
    let root = temp_dir("preview-with-soft");
    let mut state = work_with(&root, "Xem Truoc Mem");
    let (a, b) = {
        let open = state.as_ref().expect("open");
        let b = insert_chapter(open, 2, "B");
        insert_segment(open, b, 1, "da dich");
        insert_segment(open, first(open), 1, "");
        (first(open), b)
    };
    delete_chapter(&mut state, b).expect("xoa mem");

    let preview = delete_chapter_preview(state.as_ref(), a).expect("preview");

    assert_eq!(preview.disposition, Disposition::Hard);
    assert!(
        !preview.removes_work,
        "Chuong da xoa mem van la thu de giu: Tac pham o lai"
    );

    let dir = state.as_ref().expect("open").dir.clone();
    drop(state);
    cleanup(&dir);
    cleanup(&root);
}

#[test]
fn a_soft_deleted_chapter_no_longer_appears_in_library_search_or_chapter_count() {
    let root = temp_dir("soft-search");
    let side = temp_dir("soft-search-side");
    let (indexer, global) = open_index_pair(&side);
    let mut state = work_with(&root, "Tim Kiem");
    let (b, work_id) = {
        let open = state.as_ref().expect("open");
        let b = insert_chapter(open, 2, "B");
        insert_segment(open, b, 1, "tuvungduynhat");
        (b, open.meta.work_id.clone())
    };
    reindex_after_lifecycle_write(Some(&indexer), Some(&global), &root);
    let before = indexer.search("tuvungduynhat", 10).expect("search");
    assert_eq!(before.total, 1, "tien de: tu khoa tim thay truoc khi xoa");

    delete_chapter_indexed(&mut state, Some(&indexer), Some(&global), Some(&root), b).expect("xoa");

    let after = indexer.search("tuvungduynhat", 10).expect("search");
    assert_eq!(after.total, 0);
    let works = indexer
        .list_works(WorkQuery::default())
        .expect("works")
        .works;
    let card = works.iter().find(|w| w.work_id == work_id).expect("the");
    assert_eq!(card.chapter_count, 1);

    let dir = state.as_ref().expect("open").dir.clone();
    drop(state);
    indexer.close();
    global.close();
    cleanup(&dir);
    cleanup(&root);
    cleanup(&side);
}

#[test]
fn a_soft_deleted_chapter_is_not_a_target_of_a_reviewer_copy() {
    let root = temp_dir("soft-reviewer");
    let mut state = work_with(&root, "Ban Reviewer");
    let (b, c) = {
        let open = state.as_ref().expect("open");
        let b = insert_chapter(open, 2, "Ten B");
        let c = insert_chapter(open, 3, "Ten C");
        insert_segment(open, b, 1, "ban dich mot");
        insert_segment(open, c, 1, "");
        (b, c)
    };
    let copy = read_markdown_copy(
        "## Ten B\n\nban sua cua B\n\n## Ten C\n\nban sua cua C\n",
        "reviewer.md",
    )
    .expect("doc tep");
    let planned_ids = |state: &Option<OpenWork>| -> Vec<i64> {
        let open = state.as_ref().expect("open");
        let plan = open
            .store
            .read(|conn| Ok(plan_import(conn, &copy)))
            .expect("read")
            .expect("plan");
        plan.chapters.iter().map(|p| p.chapter_id).collect()
    };
    assert_eq!(
        planned_ids(&state),
        vec![b, c],
        "tien de: Ten B khop truoc khi xoa"
    );

    let (deleted, _) = delete_chapter(&mut state, b).expect("xoa");

    assert_eq!(deleted.disposition, Disposition::Soft);
    assert_eq!(
        planned_ids(&state),
        vec![c],
        "ban reviewer khong thay Chuong da xoa mem"
    );

    let dir = state.as_ref().expect("open").dir.clone();
    drop(state);
    cleanup(&dir);
    cleanup(&root);
}

#[test]
fn deleting_the_open_chapter_moves_the_pointer_to_the_previous_one_else_the_next() {
    let root = temp_dir("pointer");
    let mut state = work_with(&root, "Con Tro");
    let (a, b, c) = {
        let open = state.as_ref().expect("open");
        (
            first(open),
            insert_chapter(open, 2, "B"),
            insert_chapter(open, 3, "C"),
        )
    };
    {
        let open = state.as_mut().expect("open");
        open.chapter_id = Some(b);
        exec(open, "UPDATE work SET last_chapter_id = ?1", b);
    }

    let (deleted, _) = delete_chapter(&mut state, b).expect("xoa giua");
    assert_eq!(deleted.open_chapter_id, Some(a));
    let open = state.as_ref().expect("open");
    assert_eq!(open.chapter_id, Some(a));
    assert_eq!(last_chapter_id(&open.store), Some(a));

    let (deleted, _) = delete_chapter(&mut state, a).expect("xoa dau");
    assert_eq!(deleted.open_chapter_id, Some(c));
    let open = state.as_ref().expect("open");
    assert_eq!(open.chapter_id, Some(c));
    assert_eq!(last_chapter_id(&open.store), Some(c));

    let dir = open.dir.clone();
    drop(state);
    cleanup(&dir);
    cleanup(&root);
}

#[test]
fn deleting_a_chapter_that_is_not_open_leaves_the_pointer_alone() {
    let root = temp_dir("pointer-elsewhere");
    let mut state = work_with(&root, "Con Tro Khac");
    let (a, b) = {
        let open = state.as_ref().expect("open");
        (first(open), insert_chapter(open, 2, "B"))
    };

    delete_chapter(&mut state, b).expect("xoa");

    let open = state.as_ref().expect("open");
    assert_eq!(open.chapter_id, Some(a));
    assert_eq!(last_chapter_id(&open.store), None, "khong ghi con tro");
    let dir = open.dir.clone();
    drop(state);
    cleanup(&dir);
    cleanup(&root);
}

#[test]
fn an_unknown_chapter_id_is_refused_and_touches_no_row() {
    let root = temp_dir("not-found");
    let mut state = work_with(&root, "Khong Co");
    let b = {
        let open = state.as_ref().expect("open");
        let b = insert_chapter(open, 2, "B");
        insert_segment(open, b, 1, "x");
        b
    };
    delete_chapter(&mut state, b).expect("xoa lan dau");
    let before = ords(&state.as_ref().expect("open").store);
    let segments_before = count(
        &state.as_ref().expect("open").store,
        "SELECT COUNT(*) FROM segment",
    );

    for id in [b, 9_999] {
        let err = delete_chapter(&mut state, id).expect_err("phai tu choi");
        assert_eq!(err.code(), "chapter.not_found");
        assert_eq!(err.message_key(), MessageKey::ChapterNotFound);
    }

    let open = state.as_ref().expect("open");
    assert_eq!(ords(&open.store), before);
    assert_eq!(
        count(&open.store, "SELECT COUNT(*) FROM segment"),
        segments_before
    );
    assert_eq!(
        count(&open.store, "SELECT COUNT(*) FROM chapter_deleted"),
        1
    );
    let dir = open.dir.clone();
    drop(state);
    cleanup(&dir);
    cleanup(&root);
}

#[test]
fn deleting_with_no_work_open_is_refused() {
    let mut state: Option<OpenWork> = None;
    let err = delete_chapter(&mut state, 1).expect_err("phai tu choi");
    assert_eq!(err.code(), "work.none_open");
}

#[test]
fn after_a_delete_the_reopened_work_has_contiguous_chapters_and_the_new_progress() {
    let root = temp_dir("reopen-middle");
    let mut state = work_with(&root, "Mo Lai Giua");
    let (a, b, c) = {
        let open = state.as_ref().expect("open");
        let a = first(open);
        let b = insert_chapter(open, 2, "B");
        let c = insert_chapter(open, 3, "C");
        insert_segment(open, b, 1, "");
        exec(open, "UPDATE chapter SET status = 'done' WHERE id = ?1", c);
        exec(open, "UPDATE chapter SET status = 'done' WHERE id = ?1", a);
        (a, b, c)
    };

    delete_chapter(&mut state, b).expect("xoa giua");

    let open = state.as_ref().expect("open");
    let rebuilt = WorkMeta::rebuild_from_store(&open.store).expect("rebuild");
    assert_eq!(rebuilt.chapter_count, 2);
    assert_eq!(rebuilt.chapter_done_count, Some(2));
    assert_eq!(open.meta.chapter_count, 2, "meta.json da ghi lai");
    let indexed = indexed_from(open);
    let (work_id, dir) = (open.meta.work_id.clone(), open.dir.clone());
    drop(state);

    let reopened = open_work(&work_id, Some(&indexed)).expect("mo lai");

    assert_eq!(ords(&reopened.store), vec![(a, 1), (c, 2)]);
    assert_eq!(reopened.meta.chapter_count, 2);
    assert_eq!(reopened.chapter_id, Some(a));
    drop(reopened);
    cleanup(&dir);
    cleanup(&root);
}

#[test]
fn deleting_the_last_untranslated_chapter_removes_the_folder_the_library_entry_and_any_orphan() {
    let root = temp_dir("last-hard");
    let side = temp_dir("last-hard-side");
    let (indexer, global) = open_index_pair(&side);
    let mut state = work_with(&root, "Cuoi Han");
    let (dir, work_id, only) = {
        let open = state.as_ref().expect("open");
        insert_segment(open, first(open), 1, "");
        (open.dir.clone(), open.meta.work_id.clone(), first(open))
    };
    reindex_after_lifecycle_write(Some(&indexer), Some(&global), &root);
    assert!(
        indexer
            .list_works(WorkQuery::default())
            .expect("works")
            .works
            .iter()
            .any(|w| w.work_id == work_id),
        "tien de: Tac pham co trong Library"
    );

    let deleted =
        delete_chapter_indexed(&mut state, Some(&indexer), Some(&global), Some(&root), only)
            .expect("xoa chuong cuoi");

    assert!(deleted.work_removed);
    assert_eq!(deleted.disposition, Disposition::Hard);
    assert!(state.is_none(), "Tac pham khong con mo");
    assert!(!dir.exists(), "thu muc .atproj bien mat");
    assert!(
        !indexer
            .list_works(WorkQuery::default())
            .expect("works")
            .works
            .iter()
            .any(|w| w.work_id == work_id)
    );
    let orphans = indexer.list_orphans(Some(&global)).expect("orphans");
    assert!(
        orphans.iter().all(|o| o.work_id != work_id),
        "0 muc mo coi cho Tac pham vua xoa: {orphans:?}"
    );

    indexer.close();
    global.close();
    cleanup(&root);
    cleanup(&side);
}

#[test]
fn deleting_the_last_translated_chapter_keeps_the_work_and_it_reopens_with_zero_chapters() {
    let root = temp_dir("last-soft");
    let side = temp_dir("last-soft-side");
    let (indexer, global) = open_index_pair(&side);
    let mut state = work_with(&root, "Cuoi Mem");
    let (dir, work_id, only) = {
        let open = state.as_ref().expect("open");
        insert_segment(open, first(open), 1, "da dich");
        (open.dir.clone(), open.meta.work_id.clone(), first(open))
    };

    let deleted =
        delete_chapter_indexed(&mut state, Some(&indexer), Some(&global), Some(&root), only)
            .expect("xoa mem chuong cuoi");

    assert!(!deleted.work_removed);
    assert_eq!(deleted.disposition, Disposition::Soft);
    assert_eq!(deleted.open_chapter_id, None);
    assert!(deleted.chapters.is_empty());
    assert!(dir.join("project.db").exists(), "thu muc con tren dia");
    let open = state.as_ref().expect("Tac pham van mo");
    assert_eq!(open.chapter_id, None);
    assert_eq!(count(&open.store, "SELECT COUNT(*) FROM chapter"), 0);
    assert_eq!(
        count(&open.store, "SELECT COUNT(*) FROM chapter_deleted"),
        1
    );
    assert_eq!(last_chapter_id(&open.store), None);
    let works = indexer
        .list_works(WorkQuery::default())
        .expect("works")
        .works;
    let card = works
        .iter()
        .find(|w| w.work_id == work_id)
        .expect("the o lai Library");
    assert_eq!(card.chapter_count, 0);
    let indexed = indexed_from(open);
    drop(state);

    let reopened = open_work(&work_id, Some(&indexed)).expect("Tac pham 0 Chuong mo lai duoc");

    assert_eq!(reopened.chapter_id, None, "khong Chuong nao dang mo");
    assert!(
        list_chapters(Some(&reopened)).expect("list").is_empty(),
        "danh sach Chuong rong"
    );
    assert!(
        read_open_chapter(Some(&reopened)).is_err(),
        "khong co Chuong de doc"
    );
    assert!(read_open_chapter_segments(Some(&reopened)).is_err());
    drop(reopened);

    indexer.close();
    global.close();
    cleanup(&dir);
    cleanup(&root);
    cleanup(&side);
}

#[test]
fn the_folder_stays_when_the_last_visible_chapter_goes_but_a_soft_deleted_one_remains() {
    let root = temp_dir("last-with-soft");
    let mut state = work_with(&root, "Con Mem");
    let (dir, a, b) = {
        let open = state.as_ref().expect("open");
        let b = insert_chapter(open, 2, "B");
        insert_segment(open, b, 1, "da dich");
        insert_segment(open, first(open), 1, "");
        (open.dir.clone(), first(open), b)
    };
    let (soft, _) = delete_chapter(&mut state, b).expect("xoa mem");
    assert_eq!(soft.disposition, Disposition::Soft);

    let (hard, removed) = delete_chapter(&mut state, a).expect("xoa han chuong cuoi hien");

    assert_eq!(hard.disposition, Disposition::Hard);
    assert!(removed.is_none());
    assert!(!hard.work_removed);
    let open = state.as_ref().expect("Tac pham van mo");
    assert_eq!(open.chapter_id, None);
    assert!(dir.join("project.db").exists(), "thu muc KHONG bi xoa");
    assert_eq!(
        count(&open.store, "SELECT COUNT(*) FROM chapter_deleted"),
        1
    );

    drop(state);
    cleanup(&dir);
    cleanup(&root);
}

#[test]
fn a_work_folder_that_cannot_be_deleted_is_an_error_and_never_work_removed() {
    let root = temp_dir("folder-refused");
    let mut state = work_with(&root, "Khong Xoa Duoc");
    let only = {
        let open = state.as_ref().expect("open");
        insert_segment(open, first(open), 1, "");
        first(open)
    };
    let step = delete_chapter(&mut state, only).expect("xoa");
    let dir = step
        .1
        .as_ref()
        .expect("Tac pham het Chuong")
        .open
        .dir
        .clone();
    step.1.as_ref().expect("het Chuong").open.store.close();
    cleanup(&dir);
    fs::write(&dir, b"not a folder").expect("tao tep chan");

    let err = complete_chapter_delete(Ok(step), None, None, None).expect_err("phai bao loi");

    assert_eq!(err.message_key(), MessageKey::WorkDeleteFailed);
    assert!(state.is_none(), "Tac pham van da dong");
    assert!(dir.exists(), "khong xoa gi khi that bai");
    let _ = fs::remove_file(&dir);
    cleanup(&root);
}

#[test]
fn deleting_a_path_that_is_not_an_atproj_folder_is_refused_untouched() {
    let root = temp_dir("not-atproj");
    let victim = root.join("Tai Lieu");
    fs::create_dir_all(&victim).expect("tao");
    fs::write(victim.join("keep.txt"), b"keep").expect("ghi");

    let err = delete_work_folder(&victim, "Tai Lieu").expect_err("phai tu choi");

    let ipc: auratranslate_lib::core::i18n::IpcError = err.into();
    assert_eq!(ipc.message_key(), MessageKey::WorkDeleteFailed);
    assert!(victim.join("keep.txt").exists());
    cleanup(&root);
}

#[test]
fn a_missing_folder_counts_as_deleted() {
    let root = temp_dir("already-gone");
    delete_work_folder(&root.join("Da Mat.atproj"), "Da Mat").expect("idempotent");
    cleanup(&root);
}

#[test]
fn the_wire_closes_the_work_clears_its_per_work_state_and_removes_the_folder() {
    let root = temp_dir("wire-last");
    let side = temp_dir("wire-last-side");
    let global = Store::open(StoreSpec::global(side.join("global.db"))).expect("global");
    save_value(
        &global,
        "app_config",
        "library_root",
        &root.display().to_string(),
    )
    .expect("library_root");
    let indexer = Indexer::open(side.join("library-index.db")).expect("indexer");
    let work = work_with(&root, "Wire Cuoi").expect("tao");
    let (dir, work_id, only) = (work.dir.clone(), work.meta.work_id.clone(), first(&work));
    insert_segment(&work, only, 1, "");
    indexer
        .rebuild(&root, Some(&global))
        .expect("rebuild truoc khi xoa");

    let app = mock_builder()
        .build(mock_context(noop_assets()))
        .expect("app");
    app.manage(global);
    app.manage(indexer);
    app.manage(OpenWorkState::new(Some(work)));
    app.manage(PendingTmxImportState::new(Some(PendingTmxImport {
        tier: TmTier::Work,
        pairs: Vec::new(),
        work_id: Some(work_id.clone()),
        already_count: 0,
    })));
    app.manage(AiCallRecordsState::default());

    let deleted = delete_chapter_in_app(app.handle(), only).expect("xoa qua vo IPC");

    assert!(deleted.work_removed);
    assert!(
        app.state::<OpenWorkState>()
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .is_none(),
        "khong Tac pham nao con mo"
    );
    assert!(
        app.state::<PendingTmxImportState>()
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .is_none(),
        "lo nhap TMX tang Work bi don nhu hai duong dong con lai"
    );
    assert!(!dir.exists());
    let global = app.state::<Store>();
    let orphans = app
        .state::<Indexer>()
        .list_orphans(Some(&global))
        .expect("orphans");
    assert!(orphans.iter().all(|o| o.work_id != work_id), "{orphans:?}");
    cleanup(&root);
    cleanup(&side);
}

#[test]
fn chapter_deleted_mirrors_every_chapter_column_and_is_the_last_migration_step() {
    let root = temp_dir("mirror");
    let store = Store::open(StoreSpec::project(root.join("project.db"))).expect("project.db");
    let names = |table: &'static str| -> Vec<String> {
        store
            .read(move |conn| {
                let mut stmt =
                    conn.prepare(&format!("SELECT name FROM pragma_table_info('{table}')"))?;
                let rows = stmt.query_map([], |row| row.get::<_, String>(0))?;
                rows.collect()
            })
            .expect("table_info")
    };
    let mut chapter = names("chapter");
    let mut deleted = names("chapter_deleted");
    assert!(deleted.contains(&"deleted_at".to_owned()));
    deleted.retain(|n| n != "deleted_at");
    chapter.sort();
    deleted.sort();
    assert_eq!(
        chapter, deleted,
        "chapter_deleted phai soi moi cot cua chapter"
    );

    let last = PROJECT_MIGRATIONS.last().expect("buoc cuoi");
    assert_eq!(last.to_version, 46);
    assert!(last.sql.contains("CREATE TABLE chapter_deleted"));

    store.close();
    cleanup(&root);
}
