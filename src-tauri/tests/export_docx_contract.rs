//! Hợp đồng của `.docx` bảng hai cột (FR87): mỗi segment thuộc bản dịch một hàng, cấu trúc đoạn
//! cột phải đọc từ cờ đích đã lưu (AD-46), và tệp đọc ngược được qua `core::docx`.

use std::fs;
use std::io::Read as _;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use auratranslate_lib::commands::export::export_docx_two_column;
use auratranslate_lib::commands::project::{OpenWork, create_work_from_text};
use auratranslate_lib::core::docx::{DocxParsed, read_docx};
use auratranslate_lib::core::export::ExportScope;
use auratranslate_lib::core::store::Transaction;

static NEXT_DIR: AtomicU64 = AtomicU64::new(0);

fn temp_dir(tag: &str) -> PathBuf {
    let n = NEXT_DIR.fetch_add(1, Ordering::Relaxed);
    let dir = std::env::temp_dir().join(format!("auratranslate-docx-{}-{}-{}", std::process::id(), tag, n));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap_or_else(|e| panic!("tao {}: {e}", dir.display()));
    dir
}

struct Seg {
    source: &'static str,
    target: &'static str,
    source_end: bool,
    target_end: bool,
    omitted: bool,
    retired: bool,
}

fn seg(source: &'static str, target: &'static str) -> Seg {
    Seg { source, target, source_end: true, target_end: true, omitted: false, retired: false }
}

fn insert_chapter(open: &OpenWork, ord: i64, title: Option<&str>) -> i64 {
    let title = title.map(str::to_owned);
    open.store
        .write(move |tx: &Transaction<'_>| {
            tx.execute(
                "INSERT INTO chapter (ord, title, source_text, status, created_at, updated_at) \
                 VALUES (?1, ?2, 'x', 'not_started', 't', 't')",
                (ord, title),
            )?;
            Ok(tx.last_insert_rowid())
        })
        .expect("chen Chuong that bai")
}

fn insert_segments(open: &OpenWork, chapter_id: i64, segs: Vec<Seg>) {
    open.store
        .write(move |tx: &Transaction<'_>| {
            for (i, s) in segs.iter().enumerate() {
                tx.execute(
                    "INSERT INTO segment (chapter_id, ord, source_text, is_paragraph_end, target_text, \
                     is_target_paragraph_end, retired_at, created_at, updated_at, status, is_omitted) \
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, 't', 't', 'draft', ?8)",
                    (
                        chapter_id,
                        i64::try_from(i).unwrap_or(0),
                        s.source,
                        i64::from(s.source_end),
                        s.target,
                        i64::from(s.target_end),
                        s.retired.then_some("t"),
                        i64::from(s.omitted),
                    ),
                )?;
            }
            Ok(())
        })
        .expect("chen segment that bai");
}

struct Fixture {
    open: OpenWork,
    first: i64,
    root: PathBuf,
    out: PathBuf,
}

fn fixture(tag: &str, first_title: Option<&str>, segs: Vec<Seg>) -> Fixture {
    let root = temp_dir(tag);
    let out = temp_dir(&format!("{tag}-out"));
    let open = create_work_from_text(&root, "Tac Pham", "zh", "", "Chuong mot.".to_owned()).expect("tao tac pham");
    open.store.write(|tx: &Transaction<'_>| tx.execute("DELETE FROM segment", [])).expect("xoa segment");
    let first = open.chapter_id;
    let title = first_title.map(str::to_owned);
    open.store
        .write(move |tx: &Transaction<'_>| tx.execute("UPDATE chapter SET title = ?1 WHERE id = ?2", (title, first)))
        .expect("dat tieu de");
    insert_segments(&open, first, segs);
    Fixture { open, first, root, out }
}

impl Fixture {
    fn export(&self, scope: &ExportScope) -> PathBuf {
        let file = export_docx_two_column(Some(&self.open), scope, &self.out).expect("xuat that bai");
        PathBuf::from(file.path)
    }

    fn finish(self) {
        let dir = self.open.dir.clone();
        drop(self.open);
        let _ = fs::remove_dir_all(&dir);
        let _ = fs::remove_dir_all(&self.root);
        let _ = fs::remove_dir_all(&self.out);
    }
}

fn parsed(path: &Path) -> DocxParsed {
    read_docx(&fs::read(path).expect("doc tep")).expect("core::docx doc duoc tep vua ghi")
}

fn document_xml(path: &Path) -> String {
    let file = fs::File::open(path).expect("mo tep");
    let mut archive = zip::ZipArchive::new(file).expect("zip");
    let mut xml = String::new();
    archive.by_name("word/document.xml").expect("document.xml").read_to_string(&mut xml).expect("doc xml");
    xml
}

/// Khoảng cách sau (`w:after`) của đoạn cuối trong từng ô, theo thứ tự tài liệu.
fn last_paragraph_gaps(xml: &str) -> Vec<String> {
    xml.split("<w:tc>")
        .skip(1)
        .map(|cell| {
            let cell = cell.split("</w:tc>").next().unwrap_or("");
            let last = cell.rsplit("<w:p>").next().unwrap_or("");
            let after = last.split("w:after=\"").nth(1).unwrap_or("");
            after.split('"').next().unwrap_or("").to_owned()
        })
        .collect()
}

#[test]
fn one_chapter_becomes_one_two_column_table_with_one_row_per_in_translation_segment() {
    let mut omitted = seg("bi cat bo", "khong duoc xuat");
    omitted.omitted = true;
    let mut retired = seg("ve huu", "cung khong");
    retired.retired = true;
    let f = fixture(
        "rows",
        Some("Tieu de"),
        vec![seg("cau mot", "dich mot"), omitted, retired, seg("cau hai", "dich hai")],
    );
    let path = f.export(&ExportScope::Chapters { chapter_ids: vec![f.first] });
    let doc = parsed(&path);

    assert_eq!(doc.tables.len(), 1);
    assert_eq!(doc.tables[0].rows, 2, "chi cau thuoc ban dich co hang");
    assert_eq!(doc.tables[0].cells_per_row, vec![2, 2], "hai cot moi hang");
    let texts: Vec<String> = doc.text.split("\n\n").map(str::to_owned).collect();
    assert_eq!(texts, vec!["Tieu de", "cau mot", "dich mot", "cau hai", "dich hai"]);
    let xml = document_xml(&path);
    assert!(!xml.contains("khong duoc xuat") && !xml.contains("cung khong"), "cau bi cat bo/ve huu khong vao tep");
    f.finish();
}

#[test]
fn an_untranslated_segment_keeps_its_row_with_an_empty_right_cell() {
    let f = fixture("empty-cell", None, vec![seg("cau mot", ""), seg("cau hai", "dich hai")]);
    let path = f.export(&ExportScope::Work);
    let doc = parsed(&path);

    assert_eq!(doc.tables[0].rows, 2);
    assert_eq!(doc.tables[0].paragraphs_per_cell, vec![vec![1, 1], vec![1, 1]], "o phai rong van la mot doan");
    assert!(doc.text.contains("cau mot") && doc.text.contains("dich hai"));
    f.finish();
}

#[test]
fn a_newline_inside_a_segment_is_a_line_break_in_one_paragraph_and_survives_the_round_trip() {
    let f = fixture(
        "newline",
        Some("Tieu de"),
        vec![seg("nguon mot\nnguon hai", "doan mot\ndoan hai\ndoan ba"), seg("cau sau", "dich sau")],
    );
    let path = f.export(&ExportScope::Work);
    let doc = parsed(&path);

    assert_eq!(doc.tables[0].paragraphs_per_cell, vec![vec![1, 1], vec![1, 1]]);
    let texts: Vec<&str> = doc.text.split("\n\n").collect();
    assert_eq!(texts, vec!["Tieu de", "nguon mot\nnguon hai", "doan mot\ndoan hai\ndoan ba", "cau sau", "dich sau"]);
    assert!(document_xml(&path).contains("<w:br"), "dau xuong dong la w:br");
    f.finish();
}

#[test]
fn a_one_segment_chapter_with_a_newline_is_a_one_row_table_of_single_paragraph_cells() {
    let f = fixture("one-row", None, vec![seg("mot cau nguon", "doan mot\ndoan hai")]);
    let doc = parsed(&f.export(&ExportScope::Work));

    assert_eq!(doc.tables[0].rows, 1);
    assert_eq!(doc.tables[0].paragraphs_per_cell, vec![vec![1, 1]], "hinh dang ma AD-38 tu choi la mot hang, mot o nhieu doan");
    f.finish();
}

#[test]
fn each_column_takes_its_gap_from_its_own_paragraph_end_flag() {
    let mut differs = seg("nguon ket doan", "dich khong ket doan");
    differs.source_end = true;
    differs.target_end = false;
    let mut reverse = seg("nguon khong ket", "dich ket doan");
    reverse.source_end = false;
    reverse.target_end = true;
    let f = fixture("flags", None, vec![differs, reverse]);
    let path = f.export(&ExportScope::Work);

    let gaps = last_paragraph_gaps(&document_xml(&path));
    assert_eq!(gaps, vec!["240", "0", "0", "240"], "trai theo co nguon, phai theo co dich");
    f.finish();
}

#[test]
fn several_chapters_each_get_a_heading_and_a_table_in_chapter_order() {
    let f = fixture("chapters", Some("Chuong A"), vec![seg("a1", "A1")]);
    let second = insert_chapter(&f.open, 2, Some("Chuong B"));
    insert_segments(&f.open, second, vec![seg("b1", "B1"), seg("b2", "B2")]);
    let path = f.export(&ExportScope::Work);
    let doc = parsed(&path);

    assert_eq!(doc.tables.len(), 2);
    assert_eq!(doc.tables.iter().map(|t| t.rows).collect::<Vec<_>>(), vec![1, 2]);
    let texts: Vec<&str> = doc.text.split("\n\n").collect();
    assert_eq!(texts, vec!["Chuong A", "a1", "A1", "Chuong B", "b1", "B1", "b2", "B2"]);
    f.finish();
}

#[test]
fn a_chapter_whose_segments_are_all_omitted_writes_its_heading_and_no_table() {
    let mut cut = seg("bi cat", "bi cat");
    cut.omitted = true;
    let f = fixture("all-omitted", Some("Chuong A"), vec![cut]);
    let second = insert_chapter(&f.open, 2, Some("Chuong B"));
    insert_segments(&f.open, second, vec![seg("b1", "B1")]);
    let path = f.export(&ExportScope::Work);
    let doc = parsed(&path);

    assert_eq!(doc.tables.len(), 1, "Chuong het cau thuoc ban dich khong co bang");
    assert!(doc.text.contains("Chuong A") && doc.text.contains("Chuong B"));
    f.finish();
}

#[test]
fn exporting_twice_never_overwrites_the_first_file() {
    let f = fixture("twice", None, vec![seg("a", "b")]);
    let first = f.export(&ExportScope::Work);
    let before = fs::read(&first).expect("doc lan mot");
    let second = f.export(&ExportScope::Work);

    assert_ne!(first, second);
    assert_eq!(fs::read(&first).expect("doc lai lan mot"), before);
    assert!(second.exists());
    f.finish();
}

#[test]
fn the_written_name_is_safe_and_ends_in_docx() {
    let f = fixture("name", None, vec![seg("a", "b")]);
    let path = f.export(&ExportScope::Work);
    let name = path.file_name().and_then(|n| n.to_str()).expect("ten tep");
    assert!(name.ends_with("-hai-cot.docx"), "{name}");
    assert!(!name.contains(['/', '\\', ':']));
    f.finish();
}

#[test]
fn a_folder_that_does_not_exist_is_a_named_error_and_writes_nothing() {
    let f = fixture("nofolder", None, vec![seg("a", "b")]);
    let missing = f.out.join("khong-co");
    let err = export_docx_two_column(Some(&f.open), &ExportScope::Work, &missing).expect_err("thu muc la phai la loi");
    assert_eq!(err.code(), "export.folder_invalid");
    assert!(!missing.exists());
    f.finish();
}

#[test]
fn scope_errors_and_a_closed_work_stay_the_named_errors_of_the_scope_screen() {
    let f = fixture("errors", None, vec![seg("a", "b")]);
    let empty = export_docx_two_column(Some(&f.open), &ExportScope::Chapters { chapter_ids: vec![] }, &f.out)
        .expect_err("pham vi rong");
    assert_eq!(empty.code(), "export.scope_empty");
    let unknown =
        export_docx_two_column(Some(&f.open), &ExportScope::Chapters { chapter_ids: vec![9_999] }, &f.out)
            .expect_err("Chuong la");
    assert_eq!(unknown.code(), "segment.chapter_not_found");
    let none = export_docx_two_column(None, &ExportScope::Work, &f.out).expect_err("chua mo Tac pham");
    assert_eq!(none.code(), "work.none_open");
    assert_eq!(fs::read_dir(&f.out).map(Iterator::count).unwrap_or(0), 0, "loi khong de lai tep");
    f.finish();
}

#[test]
fn the_returned_counts_match_the_rows_written() {
    let mut cut = seg("bi cat", "bi cat");
    cut.omitted = true;
    let f = fixture("counts", None, vec![seg("a", "b"), cut, seg("c", "d")]);
    let file = export_docx_two_column(Some(&f.open), &ExportScope::Work, &f.out).expect("xuat");
    assert_eq!((file.chapter_count, file.segment_count), (1, 2));
    assert_eq!(parsed(Path::new(&file.path)).tables[0].rows, 2);
    f.finish();
}
