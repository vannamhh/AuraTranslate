//! Hợp đồng của phạm vi xuất (FR89): đếm Chương/câu/câu chưa xác nhận và hộp thoại thư mục.

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use auratranslate_lib::commands::chapter::{export_folder_from_dialog, export_scope_summary};
use auratranslate_lib::commands::project::{OpenWork, create_work_from_text};
use auratranslate_lib::core::export::ExportScope;
use auratranslate_lib::core::i18n::MessageKey;
use auratranslate_lib::core::store::Transaction;

static NEXT_DIR: AtomicU64 = AtomicU64::new(0);

fn temp_dir(tag: &str) -> PathBuf {
    let n = NEXT_DIR.fetch_add(1, Ordering::Relaxed);
    let dir = std::env::temp_dir().join(format!("auratranslate-export-{}-{}-{}", std::process::id(), tag, n));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap_or_else(|e| panic!("tao {}: {e}", dir.display()));
    dir
}

fn insert_chapter(open: &OpenWork, ord: i64) -> i64 {
    open.store
        .write(move |tx: &Transaction<'_>| {
            tx.execute(
                "INSERT INTO chapter (ord, title, source_text, status, created_at, updated_at) \
                 VALUES (?1, NULL, 'x', 'not_started', 't', 't')",
                [ord],
            )?;
            Ok(tx.last_insert_rowid())
        })
        .expect("chen Chuong that bai")
}

/// `(status, is_omitted, retired)` cho mỗi câu.
fn insert_segments(open: &OpenWork, chapter_id: i64, rows: &[(&str, i64, bool)]) {
    let rows: Vec<(String, i64, bool)> = rows.iter().map(|(s, o, r)| ((*s).to_owned(), *o, *r)).collect();
    open.store
        .write(move |tx: &Transaction<'_>| {
            for (i, (status, omitted, retired)) in rows.iter().enumerate() {
                let retired_at = retired.then_some("t");
                tx.execute(
                    "INSERT INTO segment (chapter_id, ord, source_text, is_paragraph_end, retired_at, \
                     created_at, updated_at, status, is_omitted) VALUES (?1, ?2, 's', 1, ?3, 't', 't', ?4, ?5)",
                    (chapter_id, i64::try_from(i).unwrap_or(0), retired_at, status, omitted),
                )?;
            }
            Ok(())
        })
        .expect("chen segment that bai");
}

fn clear_segments(open: &OpenWork) {
    open.store
        .write(|tx: &Transaction<'_>| tx.execute("DELETE FROM segment", []))
        .expect("xoa segment that bai");
}

struct Fixture {
    open: OpenWork,
    first: i64,
    second: i64,
    root: PathBuf,
}

fn fixture(tag: &str) -> Fixture {
    let root = temp_dir(tag);
    let open = create_work_from_text(&root, "Pham Vi Xuat", "zh", "", "Chuong mot.".to_owned())
        .expect("tao tac pham that bai");
    clear_segments(&open);
    let first = open.chapter_id;
    let second = insert_chapter(&open, 2);
    insert_segments(
        &open,
        first,
        &[("confirmed", 0, false), ("draft", 0, false), ("draft", 1, false), ("confirmed", 0, true)],
    );
    insert_segments(&open, second, &[("draft", 0, false), ("confirmed", 0, false), ("confirmed", 0, false)]);
    Fixture { open, first, second, root }
}

impl Fixture {
    fn finish(self) {
        let dir = self.open.dir.clone();
        drop(self.open);
        let _ = fs::remove_dir_all(&dir);
        let _ = fs::remove_dir_all(&self.root);
    }
}

#[test]
fn the_whole_work_counts_every_chapter_and_only_live_non_omitted_segments() {
    let f = fixture("work");
    let counts = export_scope_summary(Some(&f.open), &ExportScope::Work).expect("dem that bai");
    assert_eq!(counts.chapter_count, 2);
    assert_eq!(counts.segment_count, 5, "bo cau bi cat bo va cau ve huu");
    assert_eq!(counts.unconfirmed_count, 2);
    f.finish();
}

#[test]
fn one_chapter_counts_only_that_chapter() {
    let f = fixture("one");
    let counts = export_scope_summary(Some(&f.open), &ExportScope::Chapters { chapter_ids: vec![f.second] })
        .expect("dem that bai");
    assert_eq!((counts.chapter_count, counts.segment_count, counts.unconfirmed_count), (1, 3, 1));
    f.finish();
}

#[test]
fn duplicate_chapter_ids_count_the_chapter_once() {
    let f = fixture("dup");
    let ids = vec![f.first, f.first, f.second];
    let counts =
        export_scope_summary(Some(&f.open), &ExportScope::Chapters { chapter_ids: ids }).expect("dem that bai");
    assert_eq!((counts.chapter_count, counts.segment_count), (2, 5));
    f.finish();
}

#[test]
fn a_chapter_without_segments_still_counts_as_a_chapter() {
    let f = fixture("empty-chapter");
    let third = insert_chapter(&f.open, 3);
    let counts = export_scope_summary(Some(&f.open), &ExportScope::Chapters { chapter_ids: vec![third] })
        .expect("dem that bai");
    assert_eq!((counts.chapter_count, counts.segment_count, counts.unconfirmed_count), (1, 0, 0));
    f.finish();
}

#[test]
fn an_empty_selection_is_a_named_error_not_zero_counts() {
    let f = fixture("empty-selection");
    let err = export_scope_summary(Some(&f.open), &ExportScope::Chapters { chapter_ids: vec![] })
        .expect_err("chon rong phai la loi");
    assert_eq!(err.code(), "export.scope_empty");
    assert_eq!(err.message_key(), MessageKey::ExportScopeEmpty);
    f.finish();
}

#[test]
fn an_unknown_chapter_id_is_a_named_error() {
    let f = fixture("unknown");
    let err = export_scope_summary(Some(&f.open), &ExportScope::Chapters { chapter_ids: vec![f.first, 9_999] })
        .expect_err("Chuong la phai la loi");
    assert_eq!(err.code(), "segment.chapter_not_found");
    f.finish();
}

#[test]
fn no_open_work_is_a_named_error() {
    let err = export_scope_summary(None, &ExportScope::Work).expect_err("chua mo Tac pham phai la loi");
    assert_eq!(err.code(), "work.none_open");
}

#[test]
fn the_scope_deserialises_from_the_wire_shape_the_webview_sends() {
    let work: ExportScope = serde_json::from_str(r#"{"kind":"work"}"#).expect("work");
    assert_eq!(work, ExportScope::Work);
    let some: ExportScope = serde_json::from_str(r#"{"kind":"chapters","chapter_ids":[3,4]}"#).expect("chapters");
    assert_eq!(some, ExportScope::Chapters { chapter_ids: vec![3, 4] });
    assert!(serde_json::from_str::<ExportScope>(r#"{"kind":"book"}"#).is_err());
}

#[test]
fn cancelling_the_folder_dialog_is_ok_none_and_a_picked_folder_is_returned_as_text() {
    assert_eq!(export_folder_from_dialog(None).expect("huy"), None);
    let dir = temp_dir("folder");
    let got = export_folder_from_dialog(Some(Path::new(&dir))).expect("chon");
    assert_eq!(got.as_deref(), dir.to_str());
    let _ = fs::remove_dir_all(&dir);
}
