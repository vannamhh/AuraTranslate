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
use auratranslate_lib::core::store::Transaction;
use auratranslate_lib::core::tm::{PairOrigin, PairSide};
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

    confirm_segment(Some(&open), id, "", "").expect("xac nhan");

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
    confirm_segment(Some(&open), ids[0], "", "").expect("xac nhan");
    let before = tm_rows(&open);

    let outcome = confirm_segment(Some(&open), ids[0], "", "").expect("xac nhan lai");

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
    confirm_segment(Some(&open), id, "", "").expect("xac nhan B");
    let first = tm_rows(&open);

    type_text(&open, chapter_id, id, "C");
    assert_eq!(state_of(&open, id).0, "draft", "sua mot segment da ky dua no ve nhap");
    confirm_segment(Some(&open), id, "B", "self").expect("xac nhan C");

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
        confirm_segment(Some(&open), id, "", "").expect("xac nhan");
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
        let err = confirm_segment(Some(&open), id, "", "").expect_err("phai bi tu choi");
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

    let retired = confirm_segment(Some(&open), ids[1], "", "").expect_err("segment da ve huu");
    assert_eq!(retired.message_key(), MessageKey::SegmentRetired);
    let missing = confirm_segment(Some(&open), 9_999_999, "", "").expect_err("segment khong co");
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
        confirm_segment(Some(&open), *id, "", "").expect("xac nhan");
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
        confirm_segment(Some(&open), *id, "", "").expect("xac nhan");
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
    confirm_segment(Some(&open), id, "", "").expect("xac nhan");
    assert_eq!(tm_rows(&open).len(), 1);

    let out = promote_ai_translation(Some(&open), id, "AI van ban", false).expect("nang");

    assert!(!out.needs_confirmation);
    assert_eq!(out.status, "draft");
    assert_eq!(out.translation_origin, TRANSLATION_ORIGIN_OTHER);
    let (status, origin, _) = state_of(&open, id);
    assert_eq!((status.as_str(), origin.as_str()), ("draft", TRANSLATION_ORIGIN_OTHER));
    assert_eq!(tm_rows(&open).len(), 1, "nang cap khong ghi cap");

    confirm_segment(Some(&open), id, "AI van ban", TRANSLATION_ORIGIN_OTHER).expect("xac nhan lai");
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

    confirm_segment(Some(&open), id, "", "").expect_err("insert tm hong thi confirm hong");

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

    let outcome = wire::confirm_segment(app.handle().clone(), id, String::new(), String::new()).expect("xac nhan");
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
    confirm_segment(Some(&open), ids[0], "", "").expect("xac nhan");
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
        ids[0],
        "Dich 0.",
        TRANSLATION_ORIGIN_BILINGUAL_IMPORT,
    )
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
        ids[0],
        "Dich 0.",
        TRANSLATION_ORIGIN_BILINGUAL_IMPORT,
    )
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
    confirm_segment(Some(&open), ids[0], "B", TRANSLATION_ORIGIN_OTHER).expect("nguyen van");
    type_text(&open, chapter_id, ids[1], "C");
    confirm_segment(Some(&open), ids[1], "B", TRANSLATION_ORIGIN_OTHER).expect("viet lai");
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
        let original = format!("Dich {i}.");
        if i % 3 == 0 {
            type_text(&open, chapter_id, *id, &format!("Viet lai {i}."));
        }
        confirm_segment(
            Some(&open),
            *id,
            &original,
            TRANSLATION_ORIGIN_BILINGUAL_IMPORT,
        )
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

#[test]
fn a_stale_empty_origin_echo_on_unchanged_text_is_refused_and_writes_nothing() {
    let (root, open) = bilingual_work("stale-empty", 1);
    let (_, ids) = segment_ids(&open);
    let before = state_of(&open, ids[0]);
    let err = confirm_segment(Some(&open), ids[0], "Dich 0.", "").expect_err("phai tu choi");
    assert_eq!(err.code(), "segment.unknown_translation_origin");
    assert!(
        tm_rows(&open).is_empty(),
        "khong cap nao, nhat la khong cap origin rong"
    );
    assert_eq!(state_of(&open, ids[0]), before);
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
