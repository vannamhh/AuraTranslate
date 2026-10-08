//! Hợp đồng của `.docx` một khối (FR121): mỗi Chương còn câu là một bảng một hàng không viền,
//! mỗi ô gom đoạn từ cờ kết đoạn đã lưu của chính cột đó (AD-46).

use std::fs;
use std::io::Read as _;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use auratranslate_lib::commands::export::{export_docx_one_block, export_docx_two_column, export_scope_summary};
use auratranslate_lib::commands::project::{OpenWork, create_work_from_text};
use auratranslate_lib::core::docx::{DocxParsed, read_docx};
use auratranslate_lib::core::export::{ExportScope, ImageMode, is_untranslated};
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
    status: &'static str,
}

fn seg(source: &'static str, target: &'static str) -> Seg {
    Seg { source, target, source_end: true, target_end: true, omitted: false, retired: false, status: "draft" }
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
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, 't', 't', ?9, ?8)",
                    (
                        chapter_id,
                        i64::try_from(i).unwrap_or(0),
                        s.source,
                        i64::from(s.source_end),
                        s.target,
                        i64::from(s.target_end),
                        s.retired.then_some("t"),
                        i64::from(s.omitted),
                        s.status,
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

fn fixture_in(tag: &str, lang: &str, first_title: Option<&str>, segs: Vec<Seg>) -> Fixture {
    let root = temp_dir(tag);
    let out = temp_dir(&format!("{tag}-out"));
    let open = create_work_from_text(&root, "Tac Pham", lang, "", "Chuong mot.".to_owned()).expect("tao tac pham");
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
        let file = export_docx_one_block(Some(&self.open), scope, ImageMode::File, &self.out).expect("xuat that bai");
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

fn fixture(tag: &str, first_title: Option<&str>, segs: Vec<Seg>) -> Fixture {
    fixture_in(tag, "zh", first_title, segs)
}

fn work() -> ExportScope {
    ExportScope::Work
}

fn tag_text(xml: &str, from: usize) -> Option<(usize, String)> {
    let mut at = from;
    loop {
        let found = xml[at..].find("<w:t")? + at;
        let after = xml[found + 4..].chars().next()?;
        if after == '>' || after == ' ' {
            let open_end = xml[found..].find('>')? + found + 1;
            let close = xml[open_end..].find("</w:t>")? + open_end;
            return Some((close + 6, xml[open_end..close].to_owned()));
        }
        at = found + 4;
    }
}

/// Mỗi ô theo thứ tự tài liệu, mỗi ô là chữ của từng đoạn.
fn cells(xml: &str) -> Vec<Vec<String>> {
    xml.split("<w:tc>")
        .skip(1)
        .map(|cell| {
            let cell = cell.split("</w:tc>").next().unwrap_or("");
            cell.split("<w:p ")
                .skip(1)
                .map(|paragraph| {
                    let mut text = String::new();
                    let mut at = 0;
                    while let Some((next, piece)) = tag_text(paragraph, at) {
                        text.push_str(&piece);
                        at = next;
                    }
                    text
                })
                .collect()
        })
        .collect()
}

fn export_xml(f: &Fixture) -> String {
    document_xml(&f.export(&work()))
}

fn paras(items: &[&str]) -> Vec<String> {
    items.iter().map(|s| (*s).to_owned()).collect()
}

fn insert_asset(open: &OpenWork, chapter_id: i64, file_name: &str, source_url: Option<&str>, anchor: i64) {
    let (file_name, source_url) = (file_name.to_owned(), source_url.map(str::to_owned));
    open.store
        .write(move |tx: &Transaction<'_>| {
            tx.execute(
                "INSERT INTO asset (chapter_id, file_name, source_url, anchor_after_segment_ord, byte_len, \
                 content_type, created_at) VALUES (?1, ?2, ?3, ?4, 10, 'image/jpeg', 't')",
                rusqlite::params![chapter_id, file_name, source_url, anchor],
            )?;
            Ok(tx.last_insert_rowid())
        })
        .expect("chen asset that bai");
}

fn put_asset_file(open: &OpenWork, file_name: &str, bytes: &[u8]) {
    let dir = open.dir.join("assets");
    fs::create_dir_all(&dir).expect("tao assets");
    fs::write(dir.join(file_name), bytes).expect("ghi tep anh");
}

fn flagged(source: &'static str, target: &'static str, source_end: bool, target_end: bool) -> Seg {
    Seg { source_end, target_end, ..seg(source, target) }
}

#[test]
fn each_chapter_with_segments_is_one_one_row_two_cell_table_and_an_empty_chapter_is_only_a_heading() {
    let f = fixture("n-m", Some("Ch1"), vec![seg("s1", "t1")]);
    let second = insert_chapter(&f.open, 2, Some("Ch2"));
    let third = insert_chapter(&f.open, 3, Some("Ch3"));
    insert_segments(&f.open, second, vec![seg("s2", "t2")]);
    let mut gone = seg("x", "y");
    gone.retired = true;
    insert_segments(&f.open, third, vec![gone]);
    let path = f.export(&work());
    let doc = parsed(&path);

    assert_eq!(doc.tables.len(), 2, "Chuong het cau khong co bang");
    assert!(doc.tables.iter().all(|t| t.rows == 1 && t.cells_per_row == vec![2]));
    assert_eq!(doc.text.split("\n\n").collect::<Vec<_>>(), vec!["Ch1", "s1", "t1", "Ch2", "s2", "t2", "Ch3"]);
    let xml = document_xml(&path);
    assert!(xml.find("Ch2").unwrap_or(0) < xml.rfind("<w:tbl>").unwrap_or(0), "tieu de nam ngoai va ngay tren bang");
    f.finish();
}

#[test]
fn no_table_carries_any_border_shading_or_nested_table() {
    let f = fixture("borders", Some("T"), vec![seg("a", "b"), seg("c", "d")]);
    let xml = export_xml(&f);

    assert_eq!(xml.matches("<w:tbl>").count(), 1, "khong bang long");
    assert!(!xml.contains("w:val=\"single\""), "khong duong ke nao ke ca giua hai cot");
    for side in ["top", "left", "bottom", "right", "insideH", "insideV"] {
        assert!(xml.contains(&format!("<w:{side} w:val=\"nil\"")), "bang xoa vien {side}");
    }
    assert!(xml.matches("w:val=\"nil\"").count() >= 6 + 4 * 2, "moi o cung xoa vien cua no");
    assert!(!xml.contains("<w:shd") && !xml.contains("<w:highlight"), "khong nen mau, khong to sang");
    f.finish();
}

#[test]
fn the_right_cell_splits_on_target_flags_and_on_runs_of_line_breaks_and_joins_with_one_space() {
    let f = fixture(
        "right",
        None,
        vec![
            flagged("s1", "A\n\n\nB", false, false),
            flagged("s2", "\nC\n", false, true),
            flagged("s3", "D", false, false),
            flagged("s4", "E", true, false),
        ],
    );
    let cells = cells(&export_xml(&f));

    assert_eq!(cells[1], paras(&["A", "B C", "D E"]));
    f.finish();
}

#[test]
fn the_left_cell_follows_source_flags_only_and_joins_zh_without_a_character() {
    let segs = || {
        vec![
            flagged("一", "a", false, true),
            flagged("二", "b", true, false),
            flagged("三", "c", true, false),
        ]
    };
    let f = fixture("left", None, segs());
    let before = cells(&export_xml(&f));
    assert_eq!(before[0], paras(&["一二", "三"]));
    assert_eq!(before[1], paras(&["a", "b c"]));

    let mut flipped = segs();
    flipped[0].target_end = false;
    flipped[1].target_end = true;
    let g = fixture("left2", None, flipped);
    let after = cells(&export_xml(&g));
    assert_eq!(after[0], before[0], "doi co dich thi o trai khong doi");
    assert_eq!(after[1], paras(&["a b", "c"]));
    f.finish();
    g.finish();
}

#[test]
fn changing_source_flags_alone_leaves_the_right_cell_unchanged() {
    let build = |source_ends: [bool; 3]| {
        vec![
            flagged("s1", "a", source_ends[0], false),
            flagged("s2", "b", source_ends[1], true),
            flagged("s3", "c", source_ends[2], true),
        ]
    };
    let f = fixture("srcflag-a", None, build([true, true, true]));
    let g = fixture("srcflag-b", None, build([false, false, false]));
    let (a, b) = (cells(&export_xml(&f)), cells(&export_xml(&g)));
    assert_eq!(a[1], b[1]);
    assert_ne!(a[0], b[0]);
    f.finish();
    g.finish();
}

#[test]
fn a_non_chinese_source_joins_sentences_of_one_paragraph_with_a_space() {
    let f = fixture_in("en", "en", None, vec![flagged("One.", "a", false, true), flagged("Two.", "b", true, true)]);
    assert_eq!(cells(&export_xml(&f))[0], paras(&["One. Two."]));
    f.finish();
}

#[test]
fn paragraph_count_depends_only_on_stored_flags_not_on_punctuation_length_or_blank_lines() {
    let f = fixture("shape-a", Some("A"), vec![flagged("短。", "a.", false, false), flagged("很长很长的一句话！", "b", true, true)]);
    let second = insert_chapter(&f.open, 2, Some("B"));
    insert_segments(
        &f.open,
        second,
        vec![flagged("x", "A very long sentence?!\n\n\n\n", false, false), flagged("y", "no punct", true, true)],
    );
    let cells = cells(&export_xml(&f));
    assert_eq!((cells[0].len(), cells[1].len()), (cells[2].len(), cells[3].len()));
    f.finish();
}

#[test]
fn omitted_and_retired_segments_vanish_and_their_flags_move_to_the_surviving_sentence_before() {
    let mut cut_mid = flagged("cut-mid", "cut-mid-t", true, true);
    cut_mid.omitted = true;
    let mut cut_head = flagged("cut-head", "cut-head-t", true, true);
    cut_head.omitted = true;
    let mut retired = flagged("retired", "retired-t", true, true);
    retired.retired = true;
    let f = fixture(
        "fr133",
        None,
        vec![
            cut_head,
            flagged("s1", "t1", false, false),
            cut_mid,
            flagged("s2", "t2", false, false),
            retired,
            flagged("s3", "t3", true, true),
        ],
    );
    let xml = export_xml(&f);
    let cells = cells(&xml);

    assert_eq!(cells[0], paras(&["s1", "s2s3"]));
    assert_eq!(cells[1], paras(&["t1", "t2 t3"]));
    assert!(!xml.contains("cut-") && !xml.contains("retired"));
    f.finish();
}

#[test]
fn untranslated_sentences_leave_no_stray_space_no_empty_paragraph_and_the_left_cell_stays_whole() {
    let f = fixture(
        "untranslated",
        Some("C1"),
        vec![
            flagged("a1", "T1", false, false),
            flagged("a2", " \n ", false, true),
            flagged("a3", "", false, false),
            flagged("a4", "T4", false, false),
            flagged("a5", "", true, true),
            flagged("a6", "  ", true, true),
            flagged("a7", "T7", true, true),
        ],
    );
    let second = insert_chapter(&f.open, 2, Some("C2"));
    insert_segments(&f.open, second, vec![flagged("b1", "", true, true), flagged("b2", " ", true, true)]);
    let cells = cells(&export_xml(&f));

    assert_eq!(cells[1], paras(&["T1", "T4", "T7"]));
    assert_eq!(cells[0], paras(&["a1a2a3a4a5", "a6", "a7"]));
    assert_eq!(cells[2], paras(&["b1", "b2"]), "cot trai du");
    assert_eq!(cells[3], paras(&[""]), "o phai cua Chuong chua dich la mot doan rong duy nhat");
    f.finish();
}

#[test]
fn link_mode_puts_one_hyperlink_paragraph_after_the_anchor_in_both_cells_and_counts_skipped_images() {
    let f = fixture(
        "img-link",
        None,
        vec![flagged("s0", "t0", false, false), flagged("s1", "t1", false, false), flagged("s2", "t2", true, true)],
    );
    insert_asset(&f.open, f.first, "head.jpg", Some("https://x.test/head.jpg"), 0);
    insert_asset(&f.open, f.first, "mid.jpg", Some("https://x.test/mid.jpg"), 1);
    insert_asset(&f.open, f.first, "nolink.jpg", None, 1);
    let file = export_docx_one_block(Some(&f.open), &work(), ImageMode::Link, &f.out).expect("xuat");
    let xml = document_xml(Path::new(&file.path));
    let cells = cells(&xml);

    assert_eq!((file.image_count, file.images_skipped_missing_link, file.images_dir), (2, 1, None));
    assert_eq!(cells[0], paras(&["https://x.test/head.jpg", "s0s1", "https://x.test/mid.jpg", "s2"]), "anh cat doan thanh ba phan");
    assert_eq!(cells[1], paras(&["https://x.test/head.jpg", "t0 t1", "https://x.test/mid.jpg", "t2"]));
    assert_eq!(xml.matches("<w:hyperlink").count(), 4);
    assert!(!xml.contains("nolink"));
    f.finish();
}

#[test]
fn file_mode_writes_relative_paths_into_the_stem_folder_and_copies_the_files() {
    let f = fixture("img-file", None, vec![seg("s0", "t0")]);
    insert_asset(&f.open, f.first, "p.jpg", Some("https://x.test/p.jpg"), 0);
    put_asset_file(&f.open, "p.jpg", b"jpg");
    let file = export_docx_one_block(Some(&f.open), &work(), ImageMode::File, &f.out).expect("xuat");
    let cells = cells(&document_xml(Path::new(&file.path)));

    assert!(file.path.ends_with("Tac Pham-mot-khoi.docx"));
    let dir = file.images_dir.expect("thu muc anh");
    assert!(dir.ends_with("Tac Pham-mot-khoi-anh"));
    let name = cells[0][0].clone();
    assert!(name.starts_with("Tac Pham-mot-khoi-anh/") && name.ends_with("-p.jpg"), "{name}");
    assert_eq!(cells[0][0], cells[1][0]);
    assert!(Path::new(&dir).join(name.rsplit('/').next().unwrap_or("")).is_file());
    f.finish();
}

#[test]
fn a_missing_image_file_is_the_named_error_of_the_two_column_export() {
    let f = fixture("img-missing", None, vec![seg("s0", "t0")]);
    insert_asset(&f.open, f.first, "gone.jpg", None, 0);
    let err = export_docx_one_block(Some(&f.open), &work(), ImageMode::File, &f.out).expect_err("phai loi");
    assert_eq!(err.code(), "export.image_file_missing");
    assert_eq!(fs::read_dir(&f.out).map(Iterator::count).unwrap_or(0), 0);
    f.finish();
}

#[test]
fn an_existing_file_gets_a_numbered_name_and_stays_untouched() {
    let f = fixture("twice", None, vec![seg("a", "b")]);
    fs::write(f.out.join("Tac Pham-mot-khoi.docx"), b"cu").expect("tep co san");
    let path = f.export(&work());

    assert_eq!(path.file_name().and_then(|n| n.to_str()), Some("Tac Pham-mot-khoi (2).docx"));
    assert_eq!(fs::read(f.out.join("Tac Pham-mot-khoi.docx")).expect("doc"), b"cu");
    f.finish();
}

#[test]
fn confirmed_and_unconfirmed_sentences_are_written_identically() {
    let f = fixture("status", None, vec![seg("a", "x"), seg("b", "y")]);
    let draft = export_xml(&f);
    f.open
        .store
        .write(|tx: &Transaction<'_>| tx.execute("UPDATE segment SET status = 'confirmed'", []))
        .expect("xac nhan");
    let confirmed = export_xml(&f);

    assert_eq!(draft, confirmed);
    f.finish();
}

#[test]
fn scope_errors_a_closed_work_and_a_bad_folder_are_the_named_errors_of_the_scope_screen() {
    let f = fixture("errors", None, vec![seg("a", "b")]);
    let code = |r: Result<_, auratranslate_lib::core::i18n::IpcError>| r.map(|_: auratranslate_lib::commands::export::ExportedFile| ()).expect_err("phai loi").code().to_owned();
    assert_eq!(code(export_docx_one_block(Some(&f.open), &ExportScope::Chapters { chapter_ids: vec![] }, ImageMode::File, &f.out)), "export.scope_empty");
    assert_eq!(code(export_docx_one_block(Some(&f.open), &ExportScope::Chapters { chapter_ids: vec![9999] }, ImageMode::File, &f.out)), "segment.chapter_not_found");
    assert_eq!(code(export_docx_one_block(None, &work(), ImageMode::File, &f.out)), "work.none_open");
    assert_eq!(code(export_docx_one_block(Some(&f.open), &work(), ImageMode::File, &f.out.join("khong-co"))), "export.folder_invalid");
    f.finish();
}

#[test]
fn the_untranslated_predicate_agrees_between_sql_and_rust_including_line_breaks() {
    let cases = ["", " ", "\n", " \n\t\r ", "\u{b}\u{c}", "x", "\nx\n", " x ", "\u{a0}", "\u{3000}"];
    let segs: Vec<Seg> = cases.iter().map(|c| seg("s", c)).collect();
    let f = fixture("sql-rust", None, segs);
    let summary = export_scope_summary(Some(&f.open), &work()).expect("tom tat");
    let rust = i64::try_from(cases.iter().filter(|c| is_untranslated(c)).count()).unwrap_or(-1);

    assert_eq!(summary.untranslated_count, rust);
    assert!(rust >= 5 && rust < i64::try_from(cases.len()).unwrap_or(0), "ca doi chung phai co ca hai phia");
    f.finish();
}

#[test]
fn the_summary_counts_unconfirmed_translated_and_untranslated_apart_and_skips_omitted_and_retired() {
    let mut confirmed = seg("a", "x");
    confirmed.status = "confirmed";
    let mut blank_confirmed = seg("b", " ");
    blank_confirmed.status = "confirmed";
    let mut omitted = seg("c", "");
    omitted.omitted = true;
    let mut retired = seg("d", "y");
    retired.retired = true;
    let f = fixture("summary", None, vec![confirmed, blank_confirmed, seg("e", "z"), seg("f", "\n"), omitted, retired]);
    let summary = export_scope_summary(Some(&f.open), &work()).expect("tom tat");

    assert_eq!(
        (summary.segment_count, summary.unconfirmed_count, summary.unconfirmed_translated_count, summary.untranslated_count),
        (4, 2, 1, 2)
    );
    f.finish();
}

#[test]
fn the_two_column_export_of_the_same_work_is_unchanged_by_the_block_command() {
    let f = fixture("coexist", Some("T"), vec![seg("a", "b")]);
    let two = export_docx_two_column(Some(&f.open), &work(), ImageMode::File, &f.out).expect("hai cot");
    assert!(two.path.ends_with("Tac Pham-hai-cot.docx"));
    assert_eq!(parsed(Path::new(&two.path)).tables[0].rows, 1);
    f.finish();
}

#[test]
fn translated_sentences_with_edge_spaces_join_with_a_single_space_and_keep_no_edge_spaces() {
    let f = fixture(
        "edge-spaces",
        None,
        vec![
            flagged("s1", "A ", false, false),
            flagged("s2", " B", false, false),
            flagged("s3", "  C\n D  ", true, true),
        ],
    );
    assert_eq!(cells(&export_xml(&f))[1], paras(&["A B C", "D"]));
    f.finish();
}

#[test]
fn an_untranslated_chapter_with_an_anchored_image_shows_the_image_and_no_text_on_the_right() {
    let f = fixture("untranslated-img", None, vec![flagged("s0", " ", true, true)]);
    insert_asset(&f.open, f.first, "p.jpg", Some("https://x.test/p.jpg"), 0);
    let file = export_docx_one_block(Some(&f.open), &work(), ImageMode::Link, &f.out).expect("xuat");
    let cells = cells(&document_xml(Path::new(&file.path)));

    assert_eq!(cells[1], paras(&["https://x.test/p.jpg"]));
    assert_eq!(cells[0], paras(&["https://x.test/p.jpg", "s0"]));
    f.finish();
}
