//! Tuỳ chọn nội dung của màn Xuất (FR152): số câu, phụ lục thuật ngữ đã chốt, dấu câu chưa xác
//! nhận. Tệp mang số câu hoặc dấu không nhập lại được; xem trước dựng từ cùng bộ nạp với tệp xuất.

use std::collections::BTreeMap;
use std::fs;
use std::io::Read as _;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::sync::atomic::{AtomicU64, Ordering};

use auratranslate_lib::commands::export::{
    export_content_of, export_docx_one_block, export_docx_two_column, export_preview, export_text,
    reviewer_import_preview,
};
use auratranslate_lib::commands::project::{OpenWork, create_work_from_text};
use auratranslate_lib::core::docx::{DocxBodyItem, DocxParsed, read_docx};
use auratranslate_lib::core::export::{
    ExportContent, ExportFormat, ExportOptions, ExportScope, ImageMode, TextFormat,
};
use auratranslate_lib::core::glossary::{Category, GlossaryTier, add_manual_term};
use auratranslate_lib::core::store::{Store, StoreSpec, Transaction};

static NEXT_DIR: AtomicU64 = AtomicU64::new(0);

fn temp_dir(tag: &str) -> PathBuf {
    let n = NEXT_DIR.fetch_add(1, Ordering::Relaxed);
    let dir = std::env::temp_dir().join(format!(
        "auratranslate-content-{}-{}-{}",
        std::process::id(),
        tag,
        n
    ));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap_or_else(|e| panic!("tao {}: {e}", dir.display()));
    dir
}

#[derive(Clone, Copy)]
struct Seg {
    source: &'static str,
    target: &'static str,
    status: &'static str,
    omitted: bool,
    retired: bool,
}

const fn seg(source: &'static str, target: &'static str, status: &'static str) -> Seg {
    Seg {
        source,
        target,
        status,
        omitted: false,
        retired: false,
    }
}

struct Fixture {
    open: OpenWork,
    global: Store,
    chapters: Vec<i64>,
    out: PathBuf,
}

/// Hai Chương: 5 câu chưa xác nhận trong bản dịch, 1 câu đã xác nhận, 1 câu bị cắt bỏ và 1 câu
/// về hưu (hai câu này cũng chưa xác nhận nhưng không thuộc bản dịch).
fn two_chapters() -> Vec<Vec<Seg>> {
    let mut omitted = seg("cat bo", "khong xuat", "draft");
    omitted.omitted = true;
    let mut retired = seg("ve huu", "khong xuat", "draft");
    retired.retired = true;
    vec![
        vec![
            seg("a1", "dich a1", "draft"),
            seg("a2", "dich a2", "confirmed"),
            omitted,
            seg("a3", "", "draft"),
        ],
        vec![
            seg("b1", "dich b1", "draft"),
            retired,
            seg("b2", "dich b2", "in_progress"),
        ],
    ]
}

fn fixture(tag: &str, chapters: &[Vec<Seg>]) -> Fixture {
    let root = temp_dir(tag);
    let out = temp_dir(&format!("{tag}-out"));
    let open = create_work_from_text(&root, "Tac Pham", "zh", "", "Chuong mot.".to_owned())
        .expect("tao tac pham");
    open.store
        .write(|tx: &Transaction<'_>| tx.execute("DELETE FROM segment", []))
        .expect("xoa segment");
    let mut ids = Vec::new();
    for (i, segs) in chapters.iter().enumerate() {
        let ord = i64::try_from(i).unwrap_or(0) + 1;
        let id = if i == 0 {
            open.chapter_id
        } else {
            open.store
                .write(move |tx: &Transaction<'_>| {
                    tx.execute(
                        "INSERT INTO chapter (ord, title, source_text, status, created_at, updated_at) \
                         VALUES (?1, NULL, 'x', 'not_started', 't', 't')",
                        (ord,),
                    )?;
                    Ok(tx.last_insert_rowid())
                })
                .expect("chen Chuong")
        };
        let segs = segs.clone();
        open.store
            .write(move |tx: &Transaction<'_>| {
                for (n, s) in segs.iter().enumerate() {
                    tx.execute(
                        "INSERT INTO segment (chapter_id, ord, source_text, is_paragraph_end, target_text, \
                         is_target_paragraph_end, retired_at, created_at, updated_at, status, is_omitted) \
                         VALUES (?1, ?2, ?3, 1, ?4, 1, ?5, 't', 't', ?6, ?7)",
                        rusqlite::params![
                            id,
                            i64::try_from(n).unwrap_or(0),
                            s.source,
                            s.target,
                            s.retired.then_some("t"),
                            s.status,
                            i64::from(s.omitted)
                        ],
                    )?;
                }
                Ok(())
            })
            .expect("chen segment");
        ids.push(id);
    }
    let global = Store::open(StoreSpec::global(
        temp_dir(&format!("{tag}-g")).join("global.db"),
    ))
    .expect("mo global.db");
    Fixture {
        open,
        global,
        chapters: ids,
        out,
    }
}

impl Fixture {
    fn term(&self, tier: GlossaryTier, source: &str, translation: &str) {
        add_manual_term(
            &self.global,
            Some(&self.open.store),
            tier,
            source,
            Some(translation),
            "",
            Category::Person,
        )
        .expect("them thuat ngu");
    }

    fn content(&self, options: ExportOptions) -> ExportContent {
        export_content_of(Some(&self.open), options, Some(&self.global)).expect("noi dung xuat")
    }

    fn dump(&self) -> BTreeMap<String, Vec<String>> {
        self.open
            .store
            .read(|conn| {
                let names = conn
                    .prepare("SELECT name FROM sqlite_master WHERE type = 'table' AND name NOT LIKE 'sqlite_%' ORDER BY name")?
                    .query_map([], |r| r.get::<_, String>(0))?
                    .collect::<rusqlite::Result<Vec<_>>>()?;
                let mut out = BTreeMap::new();
                for name in names {
                    let mut stmt = conn.prepare(&format!("SELECT * FROM \"{name}\""))?;
                    let columns = stmt.column_count();
                    let rows = stmt
                        .query_map([], |row| {
                            let cells = (0..columns)
                                .map(|i| format!("{:?}", row.get_ref(i)))
                                .collect::<Vec<_>>();
                            Ok(cells.join("|"))
                        })?
                        .collect::<rusqlite::Result<Vec<_>>>()?;
                    out.insert(name, rows);
                }
                Ok(out)
            })
            .expect("dump")
    }

    fn two_column(&self, content: &ExportContent) -> PathBuf {
        let file = export_docx_two_column(
            Some(&self.open),
            &ExportScope::Work,
            ImageMode::Link,
            content,
            None,
            &self.out,
        )
        .expect("xuat hai cot");
        PathBuf::from(file.path)
    }

    fn text(&self, format: TextFormat, content: &ExportContent) -> PathBuf {
        let file = export_text(
            Some(&self.open),
            &ExportScope::Work,
            ImageMode::Link,
            format,
            content,
            None,
            &self.out,
        )
        .expect("xuat van ban");
        PathBuf::from(file.path)
    }

    fn out_is_empty(&self) -> bool {
        fs::read_dir(&self.out).map(Iterator::count).unwrap_or(0) == 0
    }
}

fn parsed(path: &Path) -> DocxParsed {
    read_docx(&fs::read(path).expect("doc tep")).expect("doc docx")
}

fn zip_entry(path: &Path, name: &str) -> String {
    let mut archive = zip::ZipArchive::new(fs::File::open(path).expect("mo")).expect("zip");
    let mut text = String::new();
    archive
        .by_name(name)
        .expect("muc zip")
        .read_to_string(&mut text)
        .expect("doc muc");
    text
}

fn rows_of(doc: &DocxParsed) -> Vec<Vec<String>> {
    doc.body
        .iter()
        .filter_map(|item| match item {
            DocxBodyItem::Table(rows) => Some(rows.clone()),
            DocxBodyItem::Paragraph(_) => None,
        })
        .flatten()
        .collect()
}

fn paragraphs_of(doc: &DocxParsed) -> Vec<String> {
    doc.body
        .iter()
        .filter_map(|item| match item {
            DocxBodyItem::Paragraph(text) => Some(text.clone()),
            DocxBodyItem::Table(_) => None,
        })
        .collect()
}

const MARK: &str = "— câu chưa xác nhận —";

fn all_on() -> ExportOptions {
    ExportOptions {
        number_segments: true,
        glossary_appendix: true,
        mark_unconfirmed: true,
    }
}

fn numbering_only() -> ExportOptions {
    ExportOptions {
        number_segments: true,
        ..ExportOptions::default()
    }
}

fn appendix_only() -> ExportOptions {
    ExportOptions {
        glossary_appendix: true,
        ..ExportOptions::default()
    }
}

#[test]
fn all_three_options_on_two_column_number_every_sentence_append_terms_and_mark_exactly_the_unconfirmed_ones()
 {
    let f = fixture("all-on", &two_chapters());
    f.term(GlossaryTier::Work, "北凉", "Bac Luong");
    f.term(GlossaryTier::Global, "徐凤年", "Tu Phung Nien");
    let path = f.two_column(&f.content(all_on()));
    let doc = parsed(&path);
    let rows = rows_of(&doc);

    assert_eq!(rows.len(), 5, "cau cat bo va ve huu khong co hang");
    let sources: Vec<&str> = rows.iter().map(|r| r[0].as_str()).collect();
    assert_eq!(
        sources,
        ["[1] a1", "[2] a2", "[3] a3", "[1] b1", "[2] b2"],
        "so cau tinh theo thu tu trong Chuong tren cac cau duoc xuat"
    );
    let marked: Vec<bool> = rows.iter().map(|r| r[1].contains(MARK)).collect();
    assert_eq!(
        marked,
        [true, false, true, true, true],
        "dau o dung 4 cau chua xac nhan co hang, cau da xac nhan khong dau"
    );
    assert_eq!(
        rows[2][1], MARK,
        "cau chua dich chi mang dong dau, khong so"
    );
    assert_eq!(rows[0][1], format!("[1] dich a1\n{MARK}"));
    assert_eq!(rows[1][1], "[2] dich a2");

    let paragraphs = paragraphs_of(&doc);
    assert_eq!(
        &paragraphs[..3],
        [
            "Thuật ngữ đã chốt",
            "北凉 = Bac Luong",
            "徐凤年 = Tu Phung Nien"
        ],
        "phu luc la doan van dau tai lieu, du hai tang"
    );
    assert!(doc.export_annotated);
    assert!(!f.out_is_empty());
}

#[test]
fn unconfirmed_count_on_the_export_screen_equals_the_marks_in_the_file() {
    let f = fixture("count", &two_chapters());
    let summary = auratranslate_lib::commands::export::export_scope_summary(
        Some(&f.open),
        &ExportScope::Work,
    )
    .expect("tom tat");
    let options = ExportOptions {
        mark_unconfirmed: true,
        ..ExportOptions::default()
    };
    let doc = parsed(&f.two_column(&f.content(options)));
    let marks = rows_of(&doc).iter().filter(|r| r[1].contains(MARK)).count();
    assert_eq!(
        i64::try_from(marks).unwrap_or(-1),
        summary.unconfirmed_count
    );
}

#[test]
fn all_confirmed_sentences_get_no_mark_and_the_count_is_zero() {
    let f = fixture(
        "all-confirmed",
        &[vec![
            seg("a1", "dich a1", "confirmed"),
            seg("a2", "dich a2", "confirmed"),
        ]],
    );
    let summary = auratranslate_lib::commands::export::export_scope_summary(
        Some(&f.open),
        &ExportScope::Work,
    )
    .expect("tom tat");
    let options = ExportOptions {
        mark_unconfirmed: true,
        ..ExportOptions::default()
    };
    let path = f.two_column(&f.content(options));
    let doc = parsed(&path);
    let rows = rows_of(&doc);

    assert_eq!(rows.len(), 2);
    assert_eq!(rows.iter().filter(|r| r[1].contains(MARK)).count(), 0);
    assert!(!zip_entry(&path, "word/document.xml").contains(MARK));
    assert_eq!(summary.unconfirmed_count, 0);
}

#[test]
fn one_block_markdown_and_text_take_numbers_and_appendix_but_refuse_the_mark_without_writing() {
    let f = fixture("others", &two_chapters());
    f.term(GlossaryTier::Work, "北凉", "Bac Luong");
    let with_numbers = f.content(ExportOptions {
        number_segments: true,
        glossary_appendix: true,
        mark_unconfirmed: false,
    });

    let block = export_docx_one_block(
        Some(&f.open),
        &ExportScope::Work,
        ImageMode::Link,
        &with_numbers,
        None,
        &f.out,
    )
    .expect("xuat mot khoi");
    let doc = parsed(Path::new(&block.path));
    assert_eq!(
        doc.tables.iter().map(|t| t.rows).collect::<Vec<_>>(),
        [1, 1]
    );
    let cells = rows_of(&doc);
    assert!(cells[0][0].contains("[1] a1") && cells[0][0].contains("[3] a3"));
    assert!(cells[0][1].contains("[1] dich a1") && cells[0][1].contains("[2] dich a2"));
    assert!(!cells[0][1].contains(MARK));
    assert_eq!(paragraphs_of(&doc)[1], "北凉 = Bac Luong");
    assert!(doc.export_annotated);

    let md = fs::read_to_string(f.text(TextFormat::Markdown, &with_numbers)).expect("doc md");
    assert!(
        md.starts_with("<!-- aura:export-annotated -->\n\n"),
        "dong dau la dau may doc duoc: {md}"
    );
    assert!(md.contains("北凉 = Bac Luong"));
    assert!(md.contains("\\[1\\] dich a1"), "so cau trong .md: {md}");
    assert!(
        !md.contains("khong xuat"),
        "cau cat bo/ve huu khong vao .md: {md}"
    );
    assert!(
        md.contains("\\[2\\] dich b2"),
        "so tiep ngay sau cau ve huu: {md}"
    );
    let txt = fs::read_to_string(f.text(TextFormat::Plain, &with_numbers)).expect("doc txt");
    assert!(txt.starts_with("Thuật ngữ đã chốt\n\n北凉 = Bac Luong\n\n"));
    assert!(txt.contains("[1] dich a1") && txt.contains("[2] dich b2"));
    assert!(
        !txt.contains("khong xuat"),
        "cau cat bo/ve huu khong vao .txt: {txt}"
    );

    let before = fs::read_dir(&f.out).map(Iterator::count).unwrap_or(0);
    let marked = f.content(ExportOptions {
        mark_unconfirmed: true,
        ..ExportOptions::default()
    });
    let code = |result: Result<_, auratranslate_lib::core::i18n::IpcError>| -> String {
        result
            .map(|_: auratranslate_lib::commands::export::ExportedFile| ())
            .expect_err("phai tu choi")
            .code()
            .to_owned()
    };
    assert_eq!(
        code(export_docx_one_block(
            Some(&f.open),
            &ExportScope::Work,
            ImageMode::Link,
            &marked,
            None,
            &f.out
        )),
        "export.mark_two_column_only"
    );
    for format in [TextFormat::Markdown, TextFormat::Plain] {
        assert_eq!(
            code(export_text(
                Some(&f.open),
                &ExportScope::Work,
                ImageMode::Link,
                format,
                &marked,
                None,
                &f.out
            )),
            "export.mark_two_column_only"
        );
    }
    assert_eq!(
        fs::read_dir(&f.out).map(Iterator::count).unwrap_or(0),
        before,
        "bi tu choi thi khong ghi tep"
    );
}

#[test]
fn all_options_off_writes_the_files_the_exporter_wrote_before_the_options_existed() {
    let f = fixture("off", &two_chapters());
    f.term(GlossaryTier::Work, "北凉", "Bac Luong");
    let off = f.content(ExportOptions::default());
    assert_eq!(off, ExportContent::default());

    let docx = f.two_column(&off);
    assert!(!zip_entry(&docx, "docProps/custom.xml").contains("aura.export_annotated"));
    let doc = parsed(&docx);
    assert!(!doc.export_annotated);
    assert!(paragraphs_of(&doc).iter().all(|p| !p.contains("Thuật ngữ")));
    assert!(rows_of(&doc).iter().all(|r| !r[0].starts_with('[')));

    let md = fs::read_to_string(f.text(TextFormat::Markdown, &off)).expect("doc md");
    assert_eq!(
        md,
        "## Chương 1\n\ndich a1\n\ndich a2\n\n## Chương 2\n\ndich b1\n\ndich b2\n"
    );
    let txt = fs::read_to_string(f.text(TextFormat::Plain, &off)).expect("doc txt");
    assert_eq!(
        txt,
        "Chương 1\n\ndich a1\n\ndich a2\n\nChương 2\n\ndich b1\n\ndich b2\n"
    );

    let block = export_docx_one_block(
        Some(&f.open),
        &ExportScope::Work,
        ImageMode::Link,
        &off,
        None,
        &f.out,
    )
    .expect("xuat mot khoi");
    let block_path = Path::new(&block.path);
    assert!(!zip_entry(block_path, "docProps/custom.xml").contains("aura.export_annotated"));
    let block_doc = parsed(block_path);
    assert!(!block_doc.export_annotated);
    assert!(
        paragraphs_of(&block_doc)
            .iter()
            .all(|p| !p.contains("Thuật ngữ"))
    );
    assert!(
        rows_of(&block_doc)
            .iter()
            .flatten()
            .all(|c| !c.contains('['))
    );
}

#[test]
fn a_two_column_docx_with_numbers_or_marks_is_refused_on_re_import_and_changes_nothing() {
    let f = fixture("reimport-docx", &two_chapters());
    let pending = Mutex::new(None);
    let before = f.dump();
    for options in [
        numbering_only(),
        ExportOptions {
            mark_unconfirmed: true,
            ..ExportOptions::default()
        },
    ] {
        let docx = f.two_column(&f.content(options));
        let err = reviewer_import_preview(Some(&f.open), &pending, &docx).expect_err("tu choi");
        assert_eq!(err.code(), "export.annotated_copy_not_reimportable");
    }
    assert!(
        pending.lock().expect("khoa").is_none(),
        "khong giu ke hoach nhap"
    );
    assert_eq!(f.dump(), before, "project.db khong doi");
}

#[test]
fn a_markdown_with_numbers_is_refused_on_re_import_and_changes_nothing() {
    let f = fixture("reimport-md", &two_chapters());
    let pending = Mutex::new(None);
    let before = f.dump();
    let md = f.text(TextFormat::Markdown, &f.content(numbering_only()));
    let err = reviewer_import_preview(Some(&f.open), &pending, &md).expect_err("tu choi");
    assert_eq!(err.code(), "export.annotated_copy_not_reimportable");
    assert!(
        pending.lock().expect("khoa").is_none(),
        "khong giu ke hoach nhap"
    );
    assert_eq!(f.dump(), before, "project.db khong doi");
}

#[test]
fn an_appendix_only_export_still_re_imports_and_the_appendix_never_becomes_a_chapter() {
    let f = fixture("appendix-only", &two_chapters());
    f.term(GlossaryTier::Work, "北凉", "Bac Luong");
    f.term(GlossaryTier::Global, "徐凤年", "Tu Phung Nien");
    let content = f.content(appendix_only());
    assert_eq!(content.appendix.len(), 2);
    let pending = Mutex::new(None);

    let docx = f.two_column(&content);
    assert!(
        !parsed(&docx).export_annotated,
        "chi phu luc thi khong co dau"
    );
    let preview = reviewer_import_preview(Some(&f.open), &pending, &docx).expect("nhap lai docx");
    assert_eq!(preview.chapters.len(), 2);
    assert!(
        preview.skipped.is_empty(),
        "phu luc khong thanh Chuong: {preview:?}"
    );

    let md = f.text(TextFormat::Markdown, &content);
    assert!(
        !fs::read_to_string(&md)
            .expect("doc md")
            .contains("aura:export-annotated")
    );
    let preview = reviewer_import_preview(Some(&f.open), &pending, &md).expect("nhap lai md");
    assert_eq!(preview.chapters.len(), 2);
    assert!(
        preview.skipped.is_empty(),
        "phu luc khong thanh Chuong: {preview:?}"
    );
}

#[test]
fn an_empty_glossary_adds_no_appendix() {
    let f = fixture("no-terms", &two_chapters());
    let content = f.content(appendix_only());
    assert!(content.appendix.is_empty());
    let doc = parsed(&f.two_column(&content));
    assert!(paragraphs_of(&doc).iter().all(|p| !p.contains("Thuật ngữ")));
}

#[test]
fn the_custom_property_survives_the_exporter_and_is_read_back_as_a_flag() {
    let f = fixture("custom", &two_chapters());
    let docx = f.two_column(&f.content(numbering_only()));
    let xml = zip_entry(&docx, "docProps/custom.xml");
    assert!(xml.contains("aura.export_annotated"), "{xml}");
    assert!(parsed(&docx).export_annotated);
}

#[test]
fn the_preview_repeats_the_first_rows_of_the_exported_file_for_every_format() {
    let f = fixture("preview", &two_chapters());
    f.term(GlossaryTier::Work, "北凉", "Bac Luong");
    let content = f.content(all_on());
    let scope = ExportScope::Work;

    let preview = export_preview(
        Some(&f.open),
        &scope,
        ExportFormat::DocxTwoColumn,
        &content,
        None,
        Some(&f.global),
    )
    .expect("xem truoc hai cot");
    let exported = rows_of(&parsed(&f.two_column(&content)));
    let shown: Vec<(Option<String>, String)> = preview
        .rows
        .iter()
        .map(|r| (r.source.clone(), r.target.clone()))
        .collect();
    let expected: Vec<(Option<String>, String)> = exported
        .iter()
        .take(3)
        .map(|r| (Some(r[0].clone()), r[1].clone()))
        .collect();
    assert_eq!(shown, expected, "chi Chuong dau, cung tien to va dau");
    assert_eq!(preview.chapter_ord, 1);
    assert_eq!(preview.file_name, "Tac Pham-hai-cot.docx");
    assert!(preview.not_reimportable);
    assert_eq!(preview.appendix, ["Thuật ngữ đã chốt", "北凉 = Bac Luong"]);
    assert_eq!(preview.appendix_term_count, 1);

    let numbers = f.content(numbering_only());
    let md = export_preview(
        Some(&f.open),
        &scope,
        ExportFormat::Markdown,
        &numbers,
        None,
        Some(&f.global),
    )
    .expect("xem truoc md");
    assert_eq!(md.file_name, "Tac Pham.md");
    assert_eq!(
        md.rows
            .iter()
            .map(|r| r.target.as_str())
            .collect::<Vec<_>>(),
        ["\\[1\\] dich a1", "\\[2\\] dich a2"]
    );
    assert!(md.rows.iter().all(|r| r.source.is_none()));

    let block = export_preview(
        Some(&f.open),
        &scope,
        ExportFormat::DocxOneBlock,
        &numbers,
        None,
        Some(&f.global),
    )
    .expect("xem truoc mot khoi");
    assert_eq!(block.rows.len(), 1);
    assert!(block.rows[0].target.contains("[1] dich a1"));

    let marked = f.content(ExportOptions {
        mark_unconfirmed: true,
        ..ExportOptions::default()
    });
    let err = export_preview(
        Some(&f.open),
        &scope,
        ExportFormat::Plain,
        &marked,
        None,
        None,
    )
    .expect_err("dau chi o hai cot");
    assert_eq!(err.code(), "export.mark_two_column_only");

    let off = export_preview(
        Some(&f.open),
        &scope,
        ExportFormat::DocxTwoColumn,
        &ExportContent::default(),
        None,
        Some(&f.global),
    )
    .expect("xem truoc tat het");
    assert!(!off.not_reimportable && off.appendix.is_empty());
    assert_eq!(off.rows[0].source.as_deref(), Some("a1"));
    assert_eq!(f.chapters.len(), 2);
}

#[test]
fn the_term_count_is_reported_while_the_appendix_switch_is_off() {
    let f = fixture("count-off", &two_chapters());
    f.term(GlossaryTier::Work, "北凉", "Bac Luong");
    f.term(GlossaryTier::Global, "徐凤年", "Tu Phung Nien");
    let preview = export_preview(
        Some(&f.open),
        &ExportScope::Work,
        ExportFormat::DocxTwoColumn,
        &ExportContent::default(),
        None,
        Some(&f.global),
    )
    .expect("xem truoc");
    assert_eq!(preview.appendix_term_count, 2);
    assert!(preview.appendix.is_empty());
}
