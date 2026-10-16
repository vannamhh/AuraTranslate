//! Removing reading marks (FR148) and the per-Chapter translation-origin counts (FR151).

use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};

use auratranslate_lib::commands::lifecycle::set_chapter_status;
use auratranslate_lib::commands::project::{OpenWork, create_work_from_text};
use auratranslate_lib::commands::segment::{
    clear_reading_marks, list_reading_marks, mark_reading_segment, read_open_chapter_segments,
    read_reading_run, unmark_reading_segment,
};
use auratranslate_lib::core::store::Transaction;

static NEXT_DIR: AtomicU64 = AtomicU64::new(0);

fn temp_dir() -> PathBuf {
    let n = NEXT_DIR.fetch_add(1, Ordering::Relaxed);
    let dir = std::env::temp_dir().join(format!(
        "auratranslate-reading-marks-{}-{n}",
        std::process::id()
    ));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).expect("create temp dir");
    dir
}

fn work(sentences: usize) -> (OpenWork, Vec<i64>) {
    let text = (0..sentences)
        .map(|i| format!("Sentence {i}."))
        .collect::<Vec<_>>()
        .join("\n\n");
    let mut open =
        create_work_from_text(&temp_dir(), "Marks", "en", "", text).expect("create work");
    let chapter_id = open.chapter_id;
    set_chapter_status(Some(&mut open), chapter_id, "done").expect("chapter done");
    let ids = read_open_chapter_segments(Some(&open))
        .expect("load")
        .segments
        .iter()
        .map(|s| s.id)
        .collect();
    (open, ids)
}

fn marked_ids(open: &OpenWork) -> Vec<i64> {
    list_reading_marks(Some(open))
        .expect("list")
        .iter()
        .map(|m| m.segment_id)
        .collect()
}

fn marked_flags(open: &OpenWork) -> Vec<bool> {
    read_reading_run(Some(open)).expect("run").chapters[0]
        .paragraphs
        .iter()
        .flat_map(|p| p.segments.iter().map(|s| s.is_marked))
        .collect()
}

#[test]
fn unmarking_one_removes_only_that_row_and_is_idempotent() {
    let (open, ids) = work(3);
    for id in &ids {
        mark_reading_segment(Some(&open), *id).expect("mark");
    }
    assert!(unmark_reading_segment(Some(&open), ids[1]).expect("unmark"));
    assert_eq!(marked_ids(&open), vec![ids[0], ids[2]]);
    assert_eq!(marked_flags(&open), vec![true, false, true]);

    assert!(
        !unmark_reading_segment(Some(&open), ids[1]).expect("unmark again"),
        "a sentence without a mark is false, not an error"
    );
    assert!(!unmark_reading_segment(Some(&open), 999_999).expect("unknown id"));
    assert_eq!(marked_ids(&open), vec![ids[0], ids[2]]);
}

#[test]
fn clearing_removes_every_mark_and_reports_how_many() {
    let (open, ids) = work(3);
    for id in &ids {
        mark_reading_segment(Some(&open), *id).expect("mark");
    }
    assert_eq!(clear_reading_marks(Some(&open)).expect("clear"), 3);
    assert!(marked_ids(&open).is_empty());
    assert_eq!(marked_flags(&open), vec![false, false, false]);
    assert_eq!(clear_reading_marks(Some(&open)).expect("clear empty"), 0);
}

#[test]
fn both_commands_refuse_when_no_work_is_open() {
    assert_eq!(
        unmark_reading_segment(None, 1).expect_err("no work").code(),
        "work.none_open"
    );
    assert_eq!(
        clear_reading_marks(None).expect_err("no work").code(),
        "work.none_open"
    );
}

#[test]
fn a_store_that_cannot_write_keeps_the_marks() {
    let (open, ids) = work(2);
    for id in &ids {
        mark_reading_segment(Some(&open), *id).expect("mark");
    }
    open.store
        .write(|tx: &Transaction<'_>| {
            tx.execute_batch(
                "CREATE TRIGGER block_mark_delete BEFORE DELETE ON reading_mark \
                 BEGIN SELECT RAISE(ABORT, 'blocked'); END;",
            )
        })
        .expect("install trigger");
    let err = clear_reading_marks(Some(&open)).expect_err("write must fail");
    assert_eq!(err.code(), "store.write_failed");
    assert_eq!(
        marked_ids(&open).len(),
        2,
        "marks stay when the write fails"
    );
}

fn set_origins(open: &OpenWork, ids: &[i64], origins: &[&str], omit_last: bool) {
    for (id, origin) in ids.iter().zip(origins) {
        let (id, origin) = (*id, (*origin).to_owned());
        open.store
            .write(move |tx: &Transaction<'_>| {
                tx.execute(
                    "UPDATE segment SET translation_origin = ?1 WHERE id = ?2",
                    (&origin, id),
                )
            })
            .expect("seed origin");
    }
    if omit_last {
        let last = *ids.last().expect("ids");
        open.store
            .write(move |tx: &Transaction<'_>| {
                tx.execute("UPDATE segment SET is_omitted = 1 WHERE id = ?1", [last])
            })
            .expect("omit");
    }
}

fn counts(open: &OpenWork) -> (i64, i64, i64) {
    let c = read_reading_run(Some(open)).expect("run").chapters[0].translation_origin_counts;
    (c.self_count, c.other_count, c.bilingual_import_count)
}

#[test]
fn chapter_origin_counts_cover_the_three_values_and_skip_empty_and_omitted() {
    let (open, ids) = work(6);
    set_origins(
        &open,
        &ids,
        &["self", "self", "other", "bilingual_import", "", "other"],
        true,
    );
    assert_eq!(
        counts(&open),
        (2, 1, 1),
        "'' is not counted and the omitted last sentence is not counted"
    );
}

#[test]
fn a_chapter_without_any_translation_counts_zero_everywhere() {
    let (open, _) = work(2);
    assert_eq!(counts(&open), (0, 0, 0));
}
