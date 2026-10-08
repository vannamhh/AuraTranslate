//! Cổng hình dạng `.docx` nhập lại (AD-38): bản một khối dành cho đăng bài bị từ chối, bản hai cột đi qua.

use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};

use auratranslate_lib::commands::export::{export_docx_one_block, export_docx_two_column};
use auratranslate_lib::commands::project::{OpenWork, create_work_from_text};
use auratranslate_lib::core::docx::{DocxParsed, TableShape, read_docx};
use auratranslate_lib::core::export::{ExportScope, ImageMode, ReimportShapeError, ReviewerDocx};
use auratranslate_lib::core::i18n::{IpcError, MessageKey};
use auratranslate_lib::core::store::Transaction;

static NEXT_DIR: AtomicU64 = AtomicU64::new(0);

fn temp_dir(tag: &str) -> PathBuf {
    let n = NEXT_DIR.fetch_add(1, Ordering::Relaxed);
    let dir = std::env::temp_dir().join(format!("auratranslate-gate-{}-{}-{}", std::process::id(), tag, n));
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
                (ord,),
            )?;
            Ok(tx.last_insert_rowid())
        })
        .expect("chen Chuong that bai")
}

fn insert_segments(open: &OpenWork, chapter_id: i64, count: usize, one_paragraph: bool) {
    open.store
        .write(move |tx: &Transaction<'_>| {
            for i in 0..count {
                tx.execute(
                    "INSERT INTO segment (chapter_id, ord, source_text, is_paragraph_end, target_text, \
                     is_target_paragraph_end, created_at, updated_at, status, is_omitted) \
                     VALUES (?1, ?2, ?3, ?5, ?4, ?5, 't', 't', 'draft', 0)",
                    (chapter_id, i64::try_from(i).unwrap_or(0), format!("s{i}"), format!("t{i}"), i64::from(!one_paragraph || i + 1 == count)),
                )?;
            }
            Ok(())
        })
        .expect("chen segment that bai");
}

struct Fixture {
    open: OpenWork,
    root: PathBuf,
    out: PathBuf,
}

fn fixture(tag: &str, segments_per_chapter: &[usize]) -> Fixture {
    fixture_with(tag, segments_per_chapter, false)
}

fn fixture_with(tag: &str, segments_per_chapter: &[usize], one_paragraph: bool) -> Fixture {
    let root = temp_dir(tag);
    let out = temp_dir(&format!("{tag}-out"));
    let open = create_work_from_text(&root, "Tac Pham", "zh", "", "Chuong mot.".to_owned()).expect("tao tac pham");
    open.store.write(|tx: &Transaction<'_>| tx.execute("DELETE FROM segment", [])).expect("xoa segment");
    for (i, &count) in segments_per_chapter.iter().enumerate() {
        let chapter = if i == 0 { open.chapter_id } else { insert_chapter(&open, i64::try_from(i).unwrap_or(0) + 1) };
        insert_segments(&open, chapter, count, one_paragraph);
    }
    Fixture { open, root, out }
}

impl Fixture {
    fn one_block(&self) -> DocxParsed {
        let file = export_docx_one_block(Some(&self.open), &ExportScope::Work, ImageMode::File, None, &self.out).expect("xuat");
        read_docx(&fs::read(file.path).expect("doc tep")).expect("doc docx")
    }

    fn two_column(&self) -> DocxParsed {
        let file = export_docx_two_column(Some(&self.open), &ExportScope::Work, ImageMode::File, None, &self.out).expect("xuat");
        read_docx(&fs::read(file.path).expect("doc tep")).expect("doc docx")
    }

    fn finish(self) {
        let dir = self.open.dir.clone();
        drop(self.open);
        let _ = fs::remove_dir_all(&dir);
        let _ = fs::remove_dir_all(&self.root);
        let _ = fs::remove_dir_all(&self.out);
    }
}

fn parsed_with(tables: Vec<TableShape>) -> DocxParsed {
    let mut parsed = read_docx(&empty_docx()).expect("doc docx rong");
    parsed.tables = tables;
    parsed
}

fn empty_docx() -> Vec<u8> {
    let f = fixture("empty", &[1]);
    let file = export_docx_two_column(Some(&f.open), &ExportScope::Work, ImageMode::File, None, &f.out).expect("xuat");
    let bytes = fs::read(file.path).expect("doc tep");
    f.finish();
    bytes
}

fn shape(cells: Vec<Vec<usize>>) -> TableShape {
    TableShape { rows: cells.len(), cells_per_row: cells.iter().map(Vec::len).collect(), paragraphs_per_cell: cells }
}

fn assert_publish_copy(result: Result<ReviewerDocx, ReimportShapeError>) {
    let error = result.expect_err("phai tu choi");
    assert_eq!(error, ReimportShapeError::PublishCopy);
    let ipc = IpcError::from(error);
    assert_eq!(ipc.message_key(), MessageKey::ExportPublishCopyNotReimportable);
    assert_eq!(ipc.message_key().as_str(), "err.export.publish_copy_not_reimportable");
}

#[test]
fn a_real_one_block_export_is_refused() {
    let f = fixture("block", &[3]);
    assert_publish_copy(ReviewerDocx::admit(f.one_block()));
    f.finish();
}

#[test]
fn one_multi_paragraph_chapter_among_several_refuses_the_whole_file() {
    let f = fixture("block-many", &[1, 1, 3, 1]);
    let parsed = f.one_block();
    assert!(parsed.tables.len() >= 4);
    assert_publish_copy(ReviewerDocx::admit(parsed));
    f.finish();
}

#[test]
fn a_real_two_column_export_passes() {
    let f = fixture("columns", &[3]);
    let parsed = f.two_column();
    let admitted = ReviewerDocx::admit(parsed.clone()).expect("phai di qua");
    assert_eq!(admitted.parsed(), &parsed);
    f.finish();
}

fn insert_linked_image(open: &OpenWork, anchor: i64) {
    let chapter_id = open.chapter_id;
    open.store
        .write(move |tx: &Transaction<'_>| {
            tx.execute(
                "INSERT INTO asset (chapter_id, file_name, source_url, anchor_after_segment_ord, byte_len, \
                 content_type, created_at) VALUES (?1, ?2, ?3, ?4, 10, 'image/jpeg', 't')",
                (chapter_id, format!("a{anchor}.jpg"), format!("https://x.test/a{anchor}.jpg"), anchor),
            )?;
            Ok(())
        })
        .expect("chen asset that bai");
}

#[test]
fn a_real_two_column_export_with_image_rows_passes() {
    let f = fixture("columns-images", &[3]);
    insert_linked_image(&f.open, 0);
    insert_linked_image(&f.open, 1);
    let file = export_docx_two_column(Some(&f.open), &ExportScope::Work, ImageMode::Link, None, &f.out).expect("xuat");
    assert_eq!(file.image_count, 2);
    let parsed = read_docx(&fs::read(file.path).expect("doc tep")).expect("doc docx");
    assert_eq!(parsed.tables[0].rows, 5);
    ReviewerDocx::admit(parsed).expect("hang anh khong lam ban hai cot thanh dang mot khoi");
    f.finish();
}

#[test]
fn a_real_one_block_export_of_one_multi_sentence_paragraph_passes() {
    let f = fixture_with("block-one-paragraph", &[3], true);
    let parsed = f.one_block();
    assert_eq!(parsed.tables.len(), 1);
    assert_eq!(parsed.tables[0].paragraphs_per_cell, vec![vec![1, 1]]);
    ReviewerDocx::admit(parsed).expect("ca sot da chap nhan o AD-38");
    f.finish();
}

#[test]
fn a_real_one_block_export_of_single_sentence_chapters_passes() {
    let f = fixture("block-single", &[1, 1]);
    ReviewerDocx::admit(f.one_block()).expect("ca sot da chap nhan o AD-38");
    f.finish();
}

#[test]
fn shapes_built_directly() {
    let pass = |tables: Vec<TableShape>| ReviewerDocx::admit(parsed_with(tables)).is_ok();
    assert!(pass(vec![shape(vec![vec![1, 1]])]), "hai cot mot cau");
    assert!(pass(vec![shape(vec![vec![1, 1], vec![1, 1], vec![1, 1]])]), "hai cot nhieu hang");
    assert!(pass(vec![]), "khong co bang");
    assert!(pass(vec![TableShape { rows: 0, cells_per_row: vec![], paragraphs_per_cell: vec![] }]), "bang 0 hang");
    assert!(!pass(vec![shape(vec![vec![2, 1]])]), "mot hang, o trai 2 doan");
    assert!(!pass(vec![shape(vec![vec![1, 5]])]), "mot hang, o phai 5 doan");
    assert!(!pass(vec![shape(vec![vec![1, 1], vec![1, 1]]), shape(vec![vec![1, 3]])]), "mot bang khop la du tu choi");
    assert!(pass(vec![shape(vec![vec![3, 1], vec![1, 1]])]), "nhieu hang khong thuoc dang mot khoi");
}

#[test]
fn unreadable_bytes_never_reach_the_gate() {
    assert!(read_docx(b"not a zip").is_err());
}
