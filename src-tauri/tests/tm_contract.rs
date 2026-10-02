//! The Work-tier Translation Memory is written exactly at the draft -> confirmed
//! transition, as a pair independent of `segment.id` (FR56, AD-6, AD-31).

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use auratranslate_lib::commands::project::{
    OpenWork, OpenWorkState, confirm_bilingual_import, create_work_from_text, stash_pending_import_source,
};
use auratranslate_lib::commands::segment::{
    SegmentTargetEdit, TRANSLATION_ORIGIN_BILINGUAL_IMPORT, TRANSLATION_ORIGIN_OTHER, TRANSLATION_ORIGIN_SELF, confirm_segment, merge_segments, promote_ai_translation,
    flush_segment_targets, read_open_chapter_segments, split_segment, wire,
};
use auratranslate_lib::core::i18n::MessageKey;
use auratranslate_lib::core::scope::ScopeResolver;
use auratranslate_lib::core::store::{GLOBAL_MIGRATIONS, Store, StoreSpec, Transaction};
use auratranslate_lib::core::tm::{PairOrigin, PairSide, TmPair, TmStoreError, TmTier, pairs_for_source};
use auratranslate_lib::core::segment::import::import_bilingual_file;
use tauri::Manager as _;
use tauri::test::{MockRuntime, mock_builder, mock_context, noop_assets};

static NEXT_DIR: AtomicU64 = AtomicU64::new(0);

fn temp_dir(tag: &str) -> PathBuf {
    let n = NEXT_DIR.fetch_add(1, Ordering::Relaxed);
    let dir = std::env::temp_dir().join(format!("auratranslate-tm-{}-{}-{}", std::process::id(), tag, n));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap_or_else(|e| panic!("tao {}: {e}", dir.display()));
    dir
}

struct DirGuard(PathBuf);

impl Drop for DirGuard {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(Path::new(&self.0));
    }
}

/// `(id, source_text, target_text, translation_origin, created_at)` of every `tm_unit` row.
type TmRow = (i64, String, String, String, String);

fn tm_rows(open: &OpenWork) -> Vec<TmRow> {
    open.store
        .read(|conn| {
            let mut stmt = conn.prepare(
                "SELECT id, source_text, target_text, translation_origin, created_at FROM tm_unit ORDER BY id",
            )?;
            let rows = stmt
                .query_map([], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?, r.get(4)?)))?
                .collect::<Result<Vec<_>, _>>()?;
            Ok(rows)
        })
        .expect("doc tm_unit")
}

fn pairs(open: &OpenWork) -> Vec<(String, String)> {
    tm_rows(open).into_iter().map(|r| (r.1, r.2)).collect()
}

fn segment_ids(open: &OpenWork) -> (i64, Vec<i64>) {
    let chapter = read_open_chapter_segments(Some(open)).expect("nap chuong");
    (chapter.chapter_id, chapter.segments.iter().map(|s| s.id).collect())
}

fn source_of(open: &OpenWork, id: i64) -> String {
    open.store
        .read(move |conn| conn.query_row("SELECT source_text FROM segment WHERE id = ?1", [id], |r| r.get(0)))
        .expect("doc source_text")
}

fn state_of(open: &OpenWork, id: i64) -> (String, String, i64) {
    open.store
        .read(move |conn| {
            let (status, origin): (String, String) = conn.query_row(
                "SELECT status, translation_origin FROM segment WHERE id = ?1",
                [id],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )?;
            let versions: i64 = conn.query_row(
                "SELECT COUNT(*) FROM segment_version WHERE segment_id = ?1",
                [id],
                |r| r.get(0),
            )?;
            Ok((status, origin, versions))
        })
        .expect("doc trang thai")
}

fn type_text(open: &OpenWork, chapter_id: i64, id: i64, text: &str) {
    flush_segment_targets(Some(open), chapter_id, &[SegmentTargetEdit { id, target_text: text.to_owned() }])
        .expect("ghi lo that bai");
}

fn set_role(open: &OpenWork, id: i64, role: &'static str) {
    open.store
        .write(move |tx: &Transaction<'_>| tx.execute("UPDATE segment SET role = ?1 WHERE id = ?2", (role, id)))
        .expect("dat role that bai");
}

fn work(tag: &str, text: &str) -> (PathBuf, OpenWork) {
    let root = temp_dir(tag);
    let open = create_work_from_text(&root, tag, "zh", "", text.to_owned()).expect("tao tac pham");
    (root, open)
}

#[test]
fn the_first_confirm_writes_one_pair_with_the_origin_written_to_the_segment() {
    let (root, open) = work("first", "一。二。");
    let (chapter_id, ids) = segment_ids(&open);
    let id = ids[0];
    type_text(&open, chapter_id, id, "Ban dich B.");
    assert!(tm_rows(&open).is_empty(), "go chu khong ghi cap nao");

    confirm_segment(Some(&open), id).expect("xac nhan");

    let rows = tm_rows(&open);
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].1, source_of(&open, id));
    assert_eq!(rows[0].2, "Ban dich B.");
    assert_eq!(rows[0].3, state_of(&open, id).1, "origin cua cap la origin vua ghi vao segment");
    assert!(!rows[0].3.is_empty());
    assert!(!rows[0].4.is_empty());
    drop(open);
    let _ = fs::remove_dir_all(root);
}

#[test]
fn confirming_again_without_an_edit_writes_no_pair_and_no_version() {
    let (root, open) = work("again", "一。二。");
    let (chapter_id, ids) = segment_ids(&open);
    type_text(&open, chapter_id, ids[0], "B");
    confirm_segment(Some(&open), ids[0]).expect("xac nhan");
    let before = tm_rows(&open);

    let outcome = confirm_segment(Some(&open), ids[0]).expect("xac nhan lai");

    assert!(!outcome.version_created);
    assert_eq!(tm_rows(&open), before);
    assert_eq!(state_of(&open, ids[0]).2, 1);
    drop(open);
    let _ = fs::remove_dir_all(root);
}

#[test]
fn editing_then_reconfirming_adds_a_pair_and_leaves_the_first_one_untouched() {
    let (root, open) = work("edit", "一。二。");
    let (chapter_id, ids) = segment_ids(&open);
    let id = ids[0];
    let source = source_of(&open, id);
    type_text(&open, chapter_id, id, "B");
    confirm_segment(Some(&open), id).expect("xac nhan B");
    let first = tm_rows(&open);

    type_text(&open, chapter_id, id, "C");
    assert_eq!(state_of(&open, id).0, "draft", "sua mot segment da ky dua no ve nhap");
    confirm_segment(Some(&open), id).expect("xac nhan C");

    let rows = tm_rows(&open);
    assert_eq!(rows.len(), 2);
    assert_eq!(rows[0], first[0], "hang dau khong bi sua");
    assert_eq!((rows[1].1.as_str(), rows[1].2.as_str()), (source.as_str(), "C"));
    drop(open);
    let _ = fs::remove_dir_all(root);
}

#[test]
fn the_same_pair_confirmed_twice_over_a_detour_is_stored_three_times() {
    let (root, open) = work("dup", "一。二。");
    let (chapter_id, ids) = segment_ids(&open);
    let id = ids[0];
    let source = source_of(&open, id);
    for text in ["B", "C", "B"] {
        type_text(&open, chapter_id, id, text);
        confirm_segment(Some(&open), id).expect("xac nhan");
    }

    let got = pairs(&open);
    assert_eq!(got.len(), 3);
    assert_eq!(got.iter().filter(|p| **p == (source.clone(), "B".to_owned())).count(), 2);
    drop(open);
    let _ = fs::remove_dir_all(root);
}

#[test]
fn an_empty_or_whitespace_target_is_refused_and_writes_no_pair() {
    let (root, open) = work("empty", "一。二。");
    let (chapter_id, ids) = segment_ids(&open);
    let id = ids[0];
    for text in ["", "  "] {
        type_text(&open, chapter_id, id, text);
        let err = confirm_segment(Some(&open), id).expect_err("phai bi tu choi");
        assert_eq!(err.message_key(), MessageKey::SegmentNothingToConfirm);
    }
    assert!(tm_rows(&open).is_empty());
    drop(open);
    let _ = fs::remove_dir_all(root);
}

#[test]
fn a_missing_or_retired_segment_writes_no_pair() {
    let (root, open) = work("gone", "一。二。");
    let (chapter_id, ids) = segment_ids(&open);
    type_text(&open, chapter_id, ids[1], "B");
    merge_segments(Some(&open), ids[1]).expect("gop");

    let retired = confirm_segment(Some(&open), ids[1]).expect_err("segment da ve huu");
    assert_eq!(retired.message_key(), MessageKey::SegmentRetired);
    let missing = confirm_segment(Some(&open), 9_999_999).expect_err("segment khong co");
    assert_eq!(missing.message_key(), MessageKey::SegmentNotFound);
    assert!(tm_rows(&open).is_empty());
    drop(open);
    let _ = fs::remove_dir_all(root);
}

#[test]
fn merging_or_splitting_prose_after_confirm_leaves_every_pair_byte_identical() {
    let (root, open) = work("regroup", "一。二。三。四。");
    let (chapter_id, ids) = segment_ids(&open);
    for (i, id) in ids.iter().take(3).enumerate() {
        type_text(&open, chapter_id, *id, &format!("Ban dich {i}"));
        confirm_segment(Some(&open), *id).expect("xac nhan");
    }
    let before = tm_rows(&open);
    assert_eq!(before.len(), 3);

    let merged = merge_segments(Some(&open), ids[1]).expect("gop");
    assert_eq!(tm_rows(&open), before, "gop ve huu hang segment nhung khong cham cap");

    let new_id = merged.new_segments[0].id;
    split_segment(Some(&open), new_id, vec![1]).expect("tach");
    assert_eq!(tm_rows(&open), before, "tach ve huu hang segment nhung khong cham cap");
    drop(open);
    let _ = fs::remove_dir_all(root);
}

#[test]
fn alt_and_caption_segments_write_pairs_like_prose() {
    let (root, open) = work("roles", "一。二。三。");
    let (chapter_id, ids) = segment_ids(&open);
    set_role(&open, ids[0], "alt");
    set_role(&open, ids[1], "caption");
    for (i, id) in ids.iter().take(2).enumerate() {
        type_text(&open, chapter_id, *id, &format!("Vai {i}"));
        confirm_segment(Some(&open), *id).expect("xac nhan");
    }

    let got = pairs(&open);
    assert_eq!(got.len(), 2);
    assert_eq!(got[0].0, source_of(&open, ids[0]));
    assert_eq!(got[1].1, "Vai 1");
    drop(open);
    let _ = fs::remove_dir_all(root);
}

#[test]
fn merging_or_splitting_with_a_role_segment_is_refused_and_writes_nothing() {
    let (root, open) = work("role-refuse", "一。二。三。");
    let (_, ids) = segment_ids(&open);
    set_role(&open, ids[1], "caption");
    let rows_before = || -> Vec<(i64, Option<String>, bool)> {
        open.store
            .read(|conn| {
                let mut stmt = conn.prepare("SELECT id, role, retired_at IS NOT NULL FROM segment ORDER BY id")?;
                let rows = stmt
                    .query_map([], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)))?
                    .collect::<Result<Vec<_>, _>>()?;
                Ok(rows)
            })
            .expect("doc segment")
    };
    let before = rows_before();

    for (label, result) in [
        ("gop voi hang tren", merge_segments(Some(&open), ids[1])),
        ("gop hang duoi vao hang vai", merge_segments(Some(&open), ids[2])),
        ("tach", split_segment(Some(&open), ids[1], vec![1])),
    ] {
        let err = result.expect_err(label);
        assert_eq!(err.code(), "segment.has_role", "{label}");
        assert_eq!(err.message_key(), MessageKey::SegmentHasRole, "{label}");
        assert!(!err.retryable(), "{label}");
    }
    assert_eq!(rows_before(), before, "khong hang nao bi ve huu hay them");
    drop(open);
    let _ = fs::remove_dir_all(root);
}

#[test]
fn promoting_onto_a_confirmed_segment_returns_it_to_draft_and_the_next_confirm_writes_a_pair() {
    let (root, open) = work("promote", "一。二。");
    let (chapter_id, ids) = segment_ids(&open);
    let id = ids[0];
    type_text(&open, chapter_id, id, "B");
    confirm_segment(Some(&open), id).expect("xac nhan");
    assert_eq!(tm_rows(&open).len(), 1);

    let out = promote_ai_translation(Some(&open), id, "AI van ban", false).expect("nang");

    assert!(!out.needs_confirmation);
    assert_eq!(out.status, "draft");
    assert_eq!(out.translation_origin, TRANSLATION_ORIGIN_OTHER);
    let (status, origin, _) = state_of(&open, id);
    assert_eq!((status.as_str(), origin.as_str()), ("draft", TRANSLATION_ORIGIN_OTHER));
    assert_eq!(tm_rows(&open).len(), 1, "nang cap khong ghi cap");

    confirm_segment(Some(&open), id).expect("xac nhan lai");
    let got = pairs(&open);
    assert_eq!(got.len(), 2);
    assert_eq!(got[1].1, "AI van ban");
    assert_eq!(tm_rows(&open)[1].3, TRANSLATION_ORIGIN_OTHER);
    drop(open);
    let _ = fs::remove_dir_all(root);
}

#[test]
fn a_held_back_promote_writes_nothing_and_keeps_the_status() {
    let (root, open) = work("held", "一。二。");
    let (chapter_id, ids) = segment_ids(&open);
    let id = ids[0];
    type_text(&open, chapter_id, id, "Chua ky");
    let before = state_of(&open, id);

    let out = promote_ai_translation(Some(&open), id, "AI van ban", false).expect("nang");

    assert!(out.needs_confirmation);
    assert_eq!(out.status, "draft");
    assert_eq!(state_of(&open, id), before);
    assert!(tm_rows(&open).is_empty());
    drop(open);
    let _ = fs::remove_dir_all(root);
}

#[test]
fn a_failing_pair_insert_rolls_the_whole_confirm_back() {
    let (root, open) = work("rollback", "一。二。");
    let (chapter_id, ids) = segment_ids(&open);
    let id = ids[0];
    type_text(&open, chapter_id, id, "B");
    open.store
        .write(|tx: &Transaction<'_>| {
            tx.execute_batch(
                "CREATE TRIGGER tm_unit_refuse BEFORE INSERT ON tm_unit \
                 BEGIN SELECT RAISE(ABORT, 'tm refused'); END;",
            )
        })
        .expect("dat trigger");

    confirm_segment(Some(&open), id).expect_err("insert tm hong thi confirm hong");

    assert_eq!(state_of(&open, id), ("draft".to_owned(), String::new(), 0));
    assert!(tm_rows(&open).is_empty());
    drop(open);
    let _ = fs::remove_dir_all(root);
}

#[test]
fn tm_unit_carries_no_reference_to_segment_chapter_or_position() {
    let (root, open) = work("schema", "一。");
    let cols: Vec<String> = open
        .store
        .read(|conn| {
            let mut stmt = conn.prepare("SELECT name FROM pragma_table_info('tm_unit') ORDER BY cid")?;
            let rows = stmt.query_map([], |r| r.get(0))?.collect::<Result<Vec<String>, _>>()?;
            Ok(rows)
        })
        .expect("doc cot");
    assert_eq!(cols, ["id", "source_text", "target_text", "translation_origin", "created_at"]);
    let fks: i64 = open
        .store
        .read(|conn| conn.query_row("SELECT COUNT(*) FROM pragma_foreign_key_list('tm_unit')", [], |r| r.get(0)))
        .expect("doc fk");
    assert_eq!(fks, 0);
    drop(open);
    let _ = fs::remove_dir_all(root);
}

#[test]
fn the_wire_confirm_writes_the_pair_through_the_command_shell() {
    let dir = temp_dir("wire");
    let open = create_work_from_text(&dir, "wire", "en", "", "A dragon roared.".to_owned()).expect("tao tac pham");
    let app: tauri::App<MockRuntime> = mock_builder().build(mock_context(noop_assets())).expect("dung app");
    app.manage(OpenWorkState::new(Some(open)));
    let _guard = DirGuard(dir);

    let id = {
        let state = app.state::<OpenWorkState>();
        let guard = state.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
        let open = guard.as_ref().expect("dang mo");
        let (chapter_id, ids) = segment_ids(open);
        type_text(open, chapter_id, ids[0], "Rong gam.");
        ids[0]
    };

    let outcome = wire::confirm_segment(app.handle().clone(), id).expect("xac nhan");
    assert!(outcome.version_created);

    let state = app.state::<OpenWorkState>();
    let guard = state.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
    let rows = pairs(guard.as_ref().expect("dang mo"));
    assert_eq!(rows, [("A dragon roared.".to_owned(), "Rong gam.".to_owned())]);
}

fn bilingual_work(tag: &str, rows: usize) -> (PathBuf, OpenWork) {
    let root = temp_dir(tag);
    let csv: String = (0..rows)
        .map(|i| format!("Nguon {i}.,Dich {i}.\n"))
        .collect();
    let path = root.join("song-ngu.csv");
    fs::write(&path, csv).expect("ghi csv");
    let shape = import_bilingual_file(&path).expect("doc csv");
    let state = std::sync::Mutex::new(None);
    stash_pending_import_source(&state, shape, None);
    let open = confirm_bilingual_import(
        &root,
        &state,
        tag,
        "en",
        "",
        "UTF-8",
        Vec::new(),
        None,
        0,
        1,
        false,
        Vec::new(),
    )
    .expect("tao tac pham song ngu");
    (root, open)
}

#[test]
fn a_typed_then_confirmed_pair_is_mine() {
    let (root, open) = work("typed-self", "一。");
    let (chapter_id, ids) = segment_ids(&open);
    type_text(&open, chapter_id, ids[0], "B");
    confirm_segment(Some(&open), ids[0]).expect("xac nhan");
    assert_eq!(tm_rows(&open)[0].3, TRANSLATION_ORIGIN_SELF);
    drop(open);
    let _ = fs::remove_dir_all(root);
}

#[test]
fn a_verbatim_bilingual_confirm_keeps_the_bilingual_import_origin() {
    let (root, open) = bilingual_work("bi-verbatim", 1);
    let (_, ids) = segment_ids(&open);
    confirm_segment(
        Some(&open),
        ids[0])
    .expect("xac nhan");
    let rows = tm_rows(&open);
    assert_eq!(
        (rows[0].2.as_str(), rows[0].3.as_str()),
        ("Dich 0.", TRANSLATION_ORIGIN_BILINGUAL_IMPORT)
    );
    drop(open);
    let _ = fs::remove_dir_all(root);
}

#[test]
fn a_rewritten_bilingual_confirm_is_mine() {
    let (root, open) = bilingual_work("bi-rewrite", 1);
    let (chapter_id, ids) = segment_ids(&open);
    type_text(&open, chapter_id, ids[0], "C");
    confirm_segment(
        Some(&open),
        ids[0])
    .expect("xac nhan");
    let rows = tm_rows(&open);
    assert_eq!(
        (rows[0].2.as_str(), rows[0].3.as_str()),
        ("C", TRANSLATION_ORIGIN_SELF)
    );
    assert_eq!(state_of(&open, ids[0]).1, TRANSLATION_ORIGIN_SELF);
    drop(open);
    let _ = fs::remove_dir_all(root);
}

#[test]
fn an_ai_promoted_sentence_is_other_verbatim_and_mine_when_rewritten() {
    let (root, open) = work("other-two", "一。二。");
    let (chapter_id, ids) = segment_ids(&open);
    for id in [ids[0], ids[1]] {
        promote_ai_translation(Some(&open), id, "B", false).expect("nang");
    }
    confirm_segment(Some(&open), ids[0]).expect("nguyen van");
    type_text(&open, chapter_id, ids[1], "C");
    confirm_segment(Some(&open), ids[1]).expect("viet lai");
    let rows = tm_rows(&open);
    assert_eq!(
        (rows[0].2.as_str(), rows[0].3.as_str()),
        ("B", TRANSLATION_ORIGIN_OTHER)
    );
    assert_eq!(
        (rows[1].2.as_str(), rows[1].3.as_str()),
        ("C", TRANSLATION_ORIGIN_SELF)
    );
    drop(open);
    let _ = fs::remove_dir_all(root);
}

#[test]
fn an_edited_bilingual_work_writes_one_pair_per_confirm_each_with_its_own_origin() {
    let (root, open) = bilingual_work("bi-many", 60);
    let (chapter_id, ids) = segment_ids(&open);
    assert_eq!(ids.len(), 60);
    for (i, id) in ids.iter().enumerate() {
        if i % 3 == 0 {
            type_text(&open, chapter_id, *id, &format!("Viet lai {i}."));
        }
        confirm_segment(
            Some(&open),
            *id)
        .expect("xac nhan");
    }
    let rows = tm_rows(&open);
    assert_eq!(rows.len(), 60, "khong cap nao bi bo");
    for (i, row) in rows.iter().enumerate() {
        if i % 3 == 0 {
            assert_eq!(
                (row.2.clone(), row.3.as_str()),
                (format!("Viet lai {i}."), TRANSLATION_ORIGIN_SELF),
                "cau {i}"
            );
        } else {
            assert_eq!(
                (row.2.clone(), row.3.as_str()),
                (format!("Dich {i}."), TRANSLATION_ORIGIN_BILINGUAL_IMPORT),
                "cau {i}"
            );
        }
    }
    drop(open);
    let _ = fs::remove_dir_all(root);
}

fn set_baseline(open: &OpenWork, id: i64, text: &'static str, origin: &'static str) {
    open.store
        .write(move |tx: &Transaction<'_>| {
            tx.execute(
                "UPDATE segment SET baseline_target_text = ?1, baseline_translation_origin = ?2 WHERE id = ?3",
                (text, origin, id),
            )
        })
        .expect("dat moc that bai");
}

fn indexed_of(open: &OpenWork) -> auratranslate_lib::core::library::indexer::IndexedWork {
    auratranslate_lib::core::library::indexer::IndexedWork {
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
    auratranslate_lib::commands::project::open_work(&indexed.work_id, Some(&indexed)).expect("mo lai tac pham")
}

#[test]
fn an_out_of_set_baseline_origin_on_unchanged_text_is_refused_and_writes_nothing() {
    let (root, open) = bilingual_work("bad-baseline", 1);
    let (_, ids) = segment_ids(&open);
    let first = ids[0];
    open.store
        .write(move |tx: &Transaction<'_>| {
            tx.execute(
                "UPDATE segment SET baseline_translation_origin = 'unknown' WHERE id = ?1",
                [first],
            )
        })
        .expect("dung fixture");
    let before = state_of(&open, ids[0]);
    let err = confirm_segment(Some(&open), ids[0]).expect_err("phai tu choi");
    assert_eq!(err.code(), "segment.unknown_translation_origin");
    assert!(tm_rows(&open).is_empty(), "khong cap nao voi origin ngoai tap");
    assert_eq!(state_of(&open, ids[0]), before);
    drop(open);
    let _ = fs::remove_dir_all(root);
}

#[test]
fn a_rewrite_flushed_before_a_chapter_reload_and_an_app_restart_is_still_mine() {
    let (root, open) = bilingual_work("reload", 1);
    let (chapter_id, ids) = segment_ids(&open);
    type_text(&open, chapter_id, ids[0], "C");

    let chapter = read_open_chapter_segments(Some(&open)).expect("nap lai chuong");
    assert_eq!(chapter.segments[0].target_text, "C");
    let open = reopen(open);

    confirm_segment(Some(&open), ids[0]).expect("xac nhan");
    let rows = tm_rows(&open);
    assert_eq!(rows.len(), 1);
    assert_eq!((rows[0].2.as_str(), rows[0].3.as_str()), ("C", TRANSLATION_ORIGIN_SELF));
    drop(open);
    let _ = fs::remove_dir_all(root);
}

#[test]
fn merging_two_rewritten_unconfirmed_bilingual_drafts_then_confirming_is_mine() {
    let (root, open) = bilingual_work("merge-rewritten", 2);
    let (chapter_id, ids) = segment_ids(&open);
    type_text(&open, chapter_id, ids[0], "Viet lai mot.");
    type_text(&open, chapter_id, ids[1], "Viet lai hai.");

    let merged = merge_segments(Some(&open), ids[1]).expect("gop");
    let merged_id = merged.new_segments[0].id;
    confirm_segment(Some(&open), merged_id).expect("xac nhan");

    let rows = tm_rows(&open);
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].3, TRANSLATION_ORIGIN_SELF);
    drop(open);
    let _ = fs::remove_dir_all(root);
}

#[test]
fn merging_two_verbatim_bilingual_drafts_keeps_the_bilingual_import_origin() {
    let (root, open) = bilingual_work("merge-verbatim", 2);
    let (_, ids) = segment_ids(&open);

    let merged = merge_segments(Some(&open), ids[1]).expect("gop");
    confirm_segment(Some(&open), merged.new_segments[0].id).expect("xac nhan");

    assert_eq!(tm_rows(&open)[0].3, TRANSLATION_ORIGIN_BILINGUAL_IMPORT);
    drop(open);
    let _ = fs::remove_dir_all(root);
}

#[test]
fn restoring_an_older_version_then_confirming_unchanged_is_others_even_when_the_baseline_was_mine() {
    let (root, open) = work("restore-baseline", "一。二。");
    let (chapter_id, ids) = segment_ids(&open);
    let id = ids[0];
    type_text(&open, chapter_id, id, "A");
    confirm_segment(Some(&open), id).expect("xac nhan A");
    type_text(&open, chapter_id, id, "C");
    confirm_segment(Some(&open), id).expect("xac nhan C");
    set_baseline(&open, id, "C", TRANSLATION_ORIGIN_SELF);
    let version_a: i64 = open
        .store
        .read(move |conn| {
            conn.query_row(
                "SELECT id FROM segment_version WHERE segment_id = ?1 AND target_text = 'A'",
                [id],
                |r| r.get(0),
            )
        })
        .expect("tim phien ban A");

    auratranslate_lib::commands::segment::restore_segment_version(Some(&open), id, version_a, false)
        .expect("khoi phuc");
    confirm_segment(Some(&open), id).expect("xac nhan sau khoi phuc");

    let rows = tm_rows(&open);
    assert_eq!(rows.len(), 3);
    assert_eq!((rows[2].2.as_str(), rows[2].3.as_str()), ("A", TRANSLATION_ORIGIN_OTHER));
    drop(open);
    let _ = fs::remove_dir_all(root);
}

#[test]
fn a_row_typed_before_step_28_is_mine_after_migration_when_confirmed_unchanged() {
    let (root, open) = work("migrated", "一。");
    let (chapter_id, ids) = segment_ids(&open);
    type_text(&open, chapter_id, ids[0], "Viet truoc buoc 28.");
    open.store
        .write(|tx: &Transaction<'_>| {
            tx.execute_batch(
                "ALTER TABLE segment DROP COLUMN baseline_target_text; \
                 ALTER TABLE segment DROP COLUMN baseline_translation_origin; \
                 DROP INDEX tm_unit_source_text; \
                 DELETE FROM schema_migration_log WHERE version IN (28, 29); \
                 PRAGMA user_version = 27;",
            )
        })
        .expect("ha ve buoc 27");
    let open = reopen(open);
    assert_eq!(open.store.schema_version(), 29, "mo lai phai chay buoc 28 va 29 that");

    confirm_segment(Some(&open), ids[0]).expect("xac nhan");
    let rows = tm_rows(&open);
    assert_eq!((rows[0].2.as_str(), rows[0].3.as_str()), ("Viet truoc buoc 28.", TRANSLATION_ORIGIN_SELF));
    drop(open);
    let _ = fs::remove_dir_all(root);
}

#[test]
fn the_step_28_backfill_copies_target_and_origin_into_the_baseline_of_every_row_retired_included() {
    let (root, open) = bilingual_work("migrated-backfill", 2);
    let (_, ids) = segment_ids(&open);
    let (draft, retired) = (ids[0], ids[1]);
    open.store
        .write(move |tx: &Transaction<'_>| {
            tx.execute(
                "UPDATE segment SET target_text = 'Cu cua nguoi khac.', translation_origin = ?1, \
                 retired_at = '2026-08-12T00:00:00.000Z' WHERE id = ?2",
                (TRANSLATION_ORIGIN_OTHER, retired),
            )?;
            tx.execute_batch(
                "ALTER TABLE segment DROP COLUMN baseline_target_text; \
                 ALTER TABLE segment DROP COLUMN baseline_translation_origin; \
                 DROP INDEX tm_unit_source_text; \
                 DELETE FROM schema_migration_log WHERE version IN (28, 29); \
                 PRAGMA user_version = 27;",
            )
        })
        .expect("ha ve buoc 27");
    let open = reopen(open);
    assert_eq!(open.store.schema_version(), 29);

    let rows: Vec<(i64, String, String, String, String)> = open
        .store
        .read(|conn| {
            let mut stmt = conn.prepare(
                "SELECT id, target_text, translation_origin, baseline_target_text, \
                 baseline_translation_origin FROM segment ORDER BY id",
            )?;
            let rows = stmt
                .query_map([], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?, r.get(4)?)))?
                .collect::<Result<Vec<_>, _>>()?;
            Ok(rows)
        })
        .expect("doc hang");
    assert_eq!(rows.len(), 2);
    assert!(rows.iter().any(|r| r.0 == retired), "hang da nghi huu phai nam trong phep doc");
    for row in &rows {
        assert_eq!((&row.3, &row.4), (&row.1, &row.2), "moc phai bang (target_text, translation_origin) o hang {}", row.0);
    }

    confirm_segment(Some(&open), draft).expect("xac nhan");
    let pairs = tm_rows(&open);
    assert_eq!(pairs.len(), 1);
    assert_eq!(pairs[0].3, TRANSLATION_ORIGIN_BILINGUAL_IMPORT);
    drop(open);
    let _ = fs::remove_dir_all(root);
}

#[test]
fn the_pair_origin_projection_pins_all_three_values() {
    for (stored, pair_origin, side) in [
        (
            TRANSLATION_ORIGIN_SELF,
            PairOrigin::SelfTranslated,
            PairSide::Mine,
        ),
        (
            TRANSLATION_ORIGIN_OTHER,
            PairOrigin::Other,
            PairSide::Others,
        ),
        (
            TRANSLATION_ORIGIN_BILINGUAL_IMPORT,
            PairOrigin::BilingualImport,
            PairSide::Others,
        ),
    ] {
        assert_eq!(PairOrigin::from_stored(stored), Some(pair_origin));
        assert_eq!(pair_origin.as_str(), stored);
        assert_eq!(pair_origin.side(), side);
    }
    assert_eq!(PairOrigin::from_stored(""), None);
    assert_eq!(PairOrigin::from_stored("unknown"), None);
}

fn open_global_db(dir: &Path) -> Store {
    Store::open(StoreSpec::global(dir.join("global.db"))).expect("mo global.db")
}

fn seed(store: &Store, rows: &[(&'static str, &'static str, &'static str)]) {
    let rows: Vec<_> = rows.to_vec();
    store
        .write(move |tx: &Transaction<'_>| {
            for (source, target, origin) in &rows {
                tx.execute(
                    "INSERT INTO tm_unit (source_text, target_text, translation_origin, created_at) \
                     VALUES (?1, ?2, ?3, '2026-01-01T00:00:00.000Z')",
                    (source, target, origin),
                )?;
            }
            Ok(())
        })
        .expect("gieo tm_unit");
}

fn lookup(global: &Store, open: Option<&OpenWork>, source: &str) -> Result<Vec<TmPair>, TmStoreError> {
    let resolver = open.map_or_else(ScopeResolver::global_only, |o| o.scope.clone());
    pairs_for_source(&resolver, global, open.map(|o| &o.store), source)
}

fn shape(pairs: &[TmPair]) -> Vec<(&str, TmTier)> {
    pairs.iter().map(|p| (p.target_text.as_str(), p.tier)).collect()
}

#[test]
fn both_tiers_hit_lists_work_then_global() {
    let (root, open) = work("dual-both", "一。");
    let global = open_global_db(&root);
    seed(&open.store, &[("S", "w", "self")]);
    seed(&global, &[("S", "g", "self")]);
    let got = lookup(&global, Some(&open), "S").expect("tra");
    assert_eq!(shape(&got), [("w", TmTier::Work), ("g", TmTier::Global)]);
    drop((open, global));
    let _ = fs::remove_dir_all(root);
}

#[test]
fn origin_beats_tier_so_a_global_mine_pair_precedes_a_work_others_pair() {
    let (root, open) = work("dual-origin", "一。");
    let global = open_global_db(&root);
    seed(&open.store, &[("S", "w", "other")]);
    seed(&global, &[("S", "g", "self")]);
    let got = lookup(&global, Some(&open), "S").expect("tra");
    assert_eq!(shape(&got), [("g", TmTier::Global), ("w", TmTier::Work)]);
    drop((open, global));
    let _ = fs::remove_dir_all(root);
}

#[test]
fn within_the_others_side_the_work_tier_comes_first() {
    let (root, open) = work("dual-side", "一。");
    let global = open_global_db(&root);
    seed(&open.store, &[("S", "w", "bilingual_import")]);
    seed(&global, &[("S", "g", "other")]);
    let got = lookup(&global, Some(&open), "S").expect("tra");
    assert_eq!(shape(&got), [("w", TmTier::Work), ("g", TmTier::Global)]);
    drop((open, global));
    let _ = fs::remove_dir_all(root);
}

#[test]
fn same_side_and_tier_keeps_id_order() {
    let (root, open) = work("dual-id", "一。");
    let global = open_global_db(&root);
    seed(&open.store, &[("S", "first", "self"), ("T", "x", "self"), ("S", "second", "self")]);
    let got = lookup(&global, Some(&open), "S").expect("tra");
    assert_eq!(shape(&got), [("first", TmTier::Work), ("second", TmTier::Work)]);
    drop((open, global));
    let _ = fs::remove_dir_all(root);
}

#[test]
fn with_no_work_open_only_global_pairs_come_back() {
    let root = temp_dir("dual-nowork");
    let global = open_global_db(&root);
    seed(&global, &[("S", "g", "self")]);
    let got = lookup(&global, None, "S").expect("tra");
    assert_eq!(shape(&got), [("g", TmTier::Global)]);
    drop(global);
    let _ = fs::remove_dir_all(root);
}

#[test]
fn a_source_with_no_row_in_either_tier_returns_empty() {
    let (root, open) = work("dual-empty", "一。");
    let global = open_global_db(&root);
    seed(&open.store, &[("T", "w", "self")]);
    assert!(lookup(&global, Some(&open), "S").expect("tra").is_empty());
    drop((open, global));
    let _ = fs::remove_dir_all(root);
}

#[test]
fn an_unknown_stored_origin_in_either_tier_is_an_error_naming_the_value() {
    let (root, open) = work("dual-unknown", "一。");
    let global = open_global_db(&root);
    seed(&global, &[("S", "g", "x")]);
    let err = lookup(&global, Some(&open), "S").expect_err("nguon goc la");
    assert_eq!(err, TmStoreError::UnknownOrigin { value: "x".to_owned() });
    assert!(err.to_string().contains("\"x\""));

    let (root2, open2) = work("dual-unknown-work", "一。");
    let global2 = open_global_db(&root2);
    seed(&open2.store, &[("S", "w", "")]);
    let err = lookup(&global2, Some(&open2), "S").expect_err("nguon goc rong");
    assert_eq!(err, TmStoreError::UnknownOrigin { value: String::new() });
    drop((open, global, open2, global2));
    let _ = fs::remove_dir_all(root);
    let _ = fs::remove_dir_all(root2);
}

#[test]
fn the_wire_confirm_writes_only_the_work_tier_with_both_stores_managed() {
    let dir = temp_dir("dual-wire");
    let open = create_work_from_text(&dir, "dual-wire", "en", "", "A dragon roared.".to_owned()).expect("tao tac pham");
    let global = open_global_db(&dir);
    let app: tauri::App<MockRuntime> = mock_builder().build(mock_context(noop_assets())).expect("dung app");
    app.manage(OpenWorkState::new(Some(open)));
    app.manage(global);
    let _guard = DirGuard(dir);

    let id = {
        let state = app.state::<OpenWorkState>();
        let guard = state.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
        let open = guard.as_ref().expect("dang mo");
        let (chapter_id, ids) = segment_ids(open);
        type_text(open, chapter_id, ids[0], "Rong gam.");
        ids[0]
    };
    wire::confirm_segment(app.handle().clone(), id).expect("xac nhan");

    let count = |store: &Store| -> i64 {
        store
            .read(|conn| conn.query_row("SELECT COUNT(*) FROM tm_unit", [], |r| r.get(0)))
            .expect("dem tm_unit")
    };
    let state = app.state::<OpenWorkState>();
    let guard = state.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
    assert_eq!(count(&guard.as_ref().expect("dang mo").store), 1);
    assert_eq!(count(&app.state::<Store>()), 0);
}

#[test]
fn a_fresh_and_an_upgraded_global_db_both_end_at_version_12_with_tm_unit_and_its_source_index() {
    let has_tm = |s: &Store| -> i64 {
        s.read(|conn| conn.query_row("SELECT COUNT(*) FROM sqlite_master WHERE name = 'tm_unit'", [], |r| r.get(0)))
            .expect("doc master")
    };
    let has_index = |s: &Store| -> i64 {
        s.read(|conn| conn.query_row("SELECT COUNT(*) FROM sqlite_master WHERE name = 'tm_unit_source_text'", [], |r| r.get(0)))
            .expect("doc master")
    };
    let fresh_dir = temp_dir("dual-fresh");
    let fresh = open_global_db(&fresh_dir);
    assert_eq!((fresh.schema_version(), has_tm(&fresh), has_index(&fresh)), (12, 1, 1));

    let old_dir = temp_dir("dual-upgrade");
    let old = Store::open(StoreSpec { migrations: &GLOBAL_MIGRATIONS[..11], ..StoreSpec::global(old_dir.join("global.db")) })
        .expect("mo o buoc 11");
    assert_eq!((old.schema_version(), has_tm(&old), has_index(&old)), (11, 1, 0));
    drop(old);
    let upgraded = open_global_db(&old_dir);
    assert_eq!((upgraded.schema_version(), has_tm(&upgraded), has_index(&upgraded)), (12, 1, 1));
    drop((fresh, upgraded));
    let _ = fs::remove_dir_all(fresh_dir);
    let _ = fs::remove_dir_all(old_dir);
}

#[test]
fn a_project_db_at_step_28_gains_the_source_index_at_step_29() {
    let (root, open) = work("index-upgrade", "一。");
    open.store
        .write(|tx: &Transaction<'_>| {
            tx.execute_batch("DROP INDEX tm_unit_source_text; DELETE FROM schema_migration_log WHERE version = 29; PRAGMA user_version = 28;")
        })
        .expect("ha ve buoc 28");
    let open = reopen(open);
    let has_index: i64 = open
        .store
        .read(|conn| conn.query_row("SELECT COUNT(*) FROM sqlite_master WHERE name = 'tm_unit_source_text'", [], |r| r.get(0)))
        .expect("doc master");
    assert_eq!((open.store.schema_version(), has_index), (29, 1));
    drop(open);
    let _ = fs::remove_dir_all(root);
}

struct Wired {
    app: tauri::App<MockRuntime>,
    _guard: DirGuard,
}

fn wired(tag: &str, text: &str, manage_global: bool) -> Wired {
    let dir = temp_dir(tag);
    let open = create_work_from_text(&dir, tag, "en", "", text.to_owned()).expect("tao tac pham");
    let global = open_global_db(&dir);
    let app: tauri::App<MockRuntime> = mock_builder().build(mock_context(noop_assets())).expect("dung app");
    app.manage(OpenWorkState::new(Some(open)));
    if manage_global {
        app.manage(global);
    }
    Wired { app, _guard: DirGuard(dir) }
}

impl Wired {
    fn with_open<R>(&self, f: impl FnOnce(&OpenWork) -> R) -> R {
        let state = self.app.state::<OpenWorkState>();
        let guard = state.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
        f(guard.as_ref().expect("dang mo"))
    }

    fn load(&self) -> auratranslate_lib::commands::segment::ChapterSegments {
        wire::read_open_chapter_segments(self.app.handle().clone()).expect("nap chuong")
    }

    fn first_id(&self) -> i64 {
        self.with_open(|open| segment_ids(open).1[0])
    }

    fn source(&self, id: i64) -> String {
        self.with_open(|open| source_of(open, id))
    }

    fn seed_work(&self, source: &str, target: &'static str, origin: &'static str) {
        let source: &'static str = Box::leak(source.to_owned().into_boxed_str());
        self.with_open(|open| seed(&open.store, &[(source, target, origin)]));
    }

    fn seed_global(&self, source: &str, target: &'static str, origin: &'static str) {
        let source: &'static str = Box::leak(source.to_owned().into_boxed_str());
        seed(&self.app.state::<Store>(), &[(source, target, origin)]);
    }

    fn target_and_baseline(&self, id: i64) -> (String, String, String) {
        self.with_open(|open| {
            open.store
                .read(move |conn| {
                    conn.query_row(
                        "SELECT target_text, baseline_target_text, baseline_translation_origin FROM segment WHERE id = ?1",
                        [id],
                        |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
                    )
                })
                .expect("doc muc")
        })
    }
}

#[test]
fn an_exact_work_pair_prefills_an_empty_draft_with_its_origin_baseline_and_no_version() {
    let w = wired("fill-hit", "A dragon roared.", true);
    let id = w.first_id();
    w.seed_work(&w.source(id), "Rong gam.", "self");

    let chapter = w.load();

    assert_eq!(chapter.tm_filled_segment_ids, [id]);
    assert_eq!(chapter.segments[0].target_text, "Rong gam.");
    assert_eq!(w.with_open(|o| state_of(o, id)), ("draft".to_owned(), "self".to_owned(), 0));
    assert_eq!(
        w.target_and_baseline(id),
        ("Rong gam.".to_owned(), "Rong gam.".to_owned(), "self".to_owned())
    );
}

#[test]
fn a_global_pair_of_mine_is_filled_ahead_of_a_work_pair_of_others() {
    let w = wired("fill-origin", "A dragon roared.", true);
    let id = w.first_id();
    w.seed_work(&w.source(id), "Cua Work.", "other");
    w.seed_global(&w.source(id), "Cua Global.", "self");

    let chapter = w.load();

    assert_eq!(chapter.segments[0].target_text, "Cua Global.");
    assert_eq!(w.with_open(|o| state_of(o, id)).1, "self");
}

#[test]
fn a_segment_with_no_pair_is_left_untouched() {
    let w = wired("fill-none", "A dragon roared.", true);
    w.seed_work("Mot cau khac.", "Khac.", "self");

    let chapter = w.load();

    assert!(chapter.tm_filled_segment_ids.is_empty());
    assert_eq!(chapter.segments[0].target_text, "");
    assert_eq!(w.with_open(|o| state_of(o, chapter.segments[0].id)).0, "draft");
}

#[test]
fn typed_text_is_never_overwritten_by_a_pair() {
    let w = wired("fill-has-text", "A dragon roared.", true);
    let id = w.first_id();
    w.with_open(|open| {
        let (chapter_id, _) = segment_ids(open);
        type_text(open, chapter_id, id, "Toi tu go.");
    });
    w.seed_work(&w.source(id), "Cua TM.", "self");

    let chapter = w.load();

    assert!(chapter.tm_filled_segment_ids.is_empty());
    assert_eq!(chapter.segments[0].target_text, "Toi tu go.");
}

#[test]
fn confirmed_omitted_and_retired_segments_are_left_untouched() {
    let w = wired("fill-ineligible", "Mot. Hai. Ba.", true);
    let ids = w.with_open(|open| segment_ids(open).1);
    assert!(ids.len() >= 3, "can ba segment");
    for id in &ids {
        w.seed_work(&w.source(*id), "Cua TM.", "self");
    }
    w.with_open(|open| {
        let (chapter_id, _) = segment_ids(open);
        type_text(open, chapter_id, ids[0], "Da ky.");
        confirm_segment(Some(open), ids[0]).expect("xac nhan");
        auratranslate_lib::commands::segment::set_segment_omitted(Some(open), ids[1], true).expect("cat bo");
        let retire = ids[2];
        open.store
            .write(move |tx: &Transaction<'_>| {
                tx.execute("UPDATE segment SET retired_at = '2026-01-01T00:00:00.000Z' WHERE id = ?1", [retire])
            })
            .expect("ve huu");
    });

    let chapter = w.load();

    assert!(chapter.tm_filled_segment_ids.is_empty());
    let by_id = |id: i64| w.with_open(|o| w_target(o, id));
    assert_eq!(by_id(ids[0]), "Da ky.");
    assert_eq!(by_id(ids[1]), "");
    assert_eq!(by_id(ids[2]), "");
}

fn w_target(open: &OpenWork, id: i64) -> String {
    open.store
        .read(move |conn| conn.query_row("SELECT target_text FROM segment WHERE id = ?1", [id], |r| r.get(0)))
        .expect("doc target")
}

#[test]
fn confirming_a_filled_segment_unedited_keeps_the_pair_origin_in_the_tm_and_on_the_segment() {
    let w = wired("fill-confirm", "A dragon roared.", true);
    let id = w.first_id();
    w.seed_work(&w.source(id), "Rong gam.", "bilingual_import");
    w.load();

    wire::confirm_segment(w.app.handle().clone(), id).expect("xac nhan");

    w.with_open(|open| {
        assert_eq!(state_of(open, id).1, TRANSLATION_ORIGIN_BILINGUAL_IMPORT);
        let rows = tm_rows(open);
        let last = rows.last().expect("co cap moi");
        assert_eq!((last.2.as_str(), last.3.as_str()), ("Rong gam.", TRANSLATION_ORIGIN_BILINGUAL_IMPORT));
    });
}

#[test]
fn confirming_a_filled_segment_after_one_character_changed_is_mine() {
    let w = wired("fill-confirm-edited", "A dragon roared.", true);
    let id = w.first_id();
    w.seed_work(&w.source(id), "Rong gam.", "other");
    w.load();
    w.with_open(|open| {
        let (chapter_id, _) = segment_ids(open);
        type_text(open, chapter_id, id, "Rong gam!");
    });

    wire::confirm_segment(w.app.handle().clone(), id).expect("xac nhan");

    w.with_open(|open| assert_eq!(state_of(open, id).1, TRANSLATION_ORIGIN_SELF));
}

#[test]
fn a_second_load_reports_no_filled_ids_and_keeps_the_filled_draft() {
    let w = wired("fill-reload", "A dragon roared.", true);
    let id = w.first_id();
    w.seed_work(&w.source(id), "Rong gam.", "self");
    assert_eq!(w.load().tm_filled_segment_ids, [id]);

    let again = w.load();

    assert!(again.tm_filled_segment_ids.is_empty());
    assert_eq!(again.segments[0].target_text, "Rong gam.");
    assert_eq!(w.with_open(|o| state_of(o, id)).0, "draft");
}

#[test]
fn a_load_without_a_managed_global_store_fails_and_writes_nothing() {
    let w = wired("fill-no-global", "A dragon roared.", false);
    let id = w.first_id();
    w.seed_work(&w.source(id), "Rong gam.", "self");

    let err = wire::read_open_chapter_segments(w.app.handle().clone()).expect_err("thieu global");

    assert_eq!(err.code(), "store.open_failed");
    assert_eq!(w.with_open(|o| w_target(o, id)), "");
}

#[test]
fn two_empty_drafts_with_the_same_source_are_both_filled_from_one_pair() {
    let w = wired("fill-dup", "Lap lai. Lap lai.", true);
    let ids = w.with_open(|open| segment_ids(open).1);
    assert_eq!(ids.len(), 2, "can hai segment");
    assert_eq!(w.source(ids[0]), w.source(ids[1]), "hai segment cung nguon");
    w.seed_work(&w.source(ids[0]), "Dich chung.", "self");

    let chapter = w.load();

    assert_eq!(chapter.tm_filled_segment_ids, ids);
    assert!(chapter.segments.iter().all(|s| s.target_text == "Dich chung."));
}

#[test]
fn the_exact_lookup_uses_the_source_index_in_both_stores() {
    let plan = |store: &Store| -> String {
        store
            .read(|conn| {
                let mut stmt = conn.prepare(
                    "EXPLAIN QUERY PLAN SELECT source_text, target_text, translation_origin FROM tm_unit WHERE source_text = ?1",
                )?;
                let rows = stmt
                    .query_map(["S"], |r| r.get::<_, String>(3))?
                    .collect::<Result<Vec<_>, _>>()?;
                Ok(rows.join(" | "))
            })
            .expect("doc ke hoach")
    };
    let (root, open) = work("plan", "一。");
    let global = open_global_db(&root);
    for (name, store) in [("project", &open.store), ("global", &global)] {
        let p = plan(store);
        assert!(p.contains("tm_unit_source_text"), "{name}: {p}");
    }
    drop((open, global));
    let _ = fs::remove_dir_all(root);
}
