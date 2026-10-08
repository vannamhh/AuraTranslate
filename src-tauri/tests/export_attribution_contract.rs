//! Hợp đồng của khối ghi nguồn (8.7, AD-43): dựng lúc xuất từ bốn cột `chapter.origin_*` và tên
//! người dịch toàn cục, đứng trước tiêu đề Chương, ngoài mọi bảng, ở cả bốn định dạng.

use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

use auratranslate_lib::commands::attribution::{translator_name_get, translator_name_save};
use auratranslate_lib::commands::chapter::update_chapter_origin;
use auratranslate_lib::commands::export::{attribution_of, export_docx_one_block, export_docx_two_column, export_text};
use auratranslate_lib::commands::project::{OpenWork, create_work_from_text};
use auratranslate_lib::core::docx::{DocxParsed, read_docx};
use auratranslate_lib::core::export::{Attribution, ExportScope, ImageMode, TextFormat};
use auratranslate_lib::core::store::{Store, StoreSpec, Transaction, Tuning};

static NEXT_DIR: AtomicU64 = AtomicU64::new(0);

fn temp_dir(tag: &str) -> PathBuf {
    let n = NEXT_DIR.fetch_add(1, Ordering::Relaxed);
    let dir = std::env::temp_dir().join(format!("auratranslate-attr-{}-{}-{}", std::process::id(), tag, n));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap_or_else(|e| panic!("tao {}: {e}", dir.display()));
    dir
}

fn open_global(dir: &std::path::Path) -> Store {
    Store::open(StoreSpec {
        tuning: Tuning {
            checkpoint_tick: Duration::from_millis(50),
            idle_before_passive: Duration::from_secs(3600),
            wal_threshold_bytes: u64::MAX,
            close_truncate_budget: Duration::from_secs(5),
            ..Tuning::default()
        },
        ..StoreSpec::global(dir.join("global.db"))
    })
    .expect("mo kho global")
}

struct Fixture {
    open: OpenWork,
    second: i64,
    out: PathBuf,
}

fn add_segment(open: &OpenWork, chapter_id: i64, source: &'static str, target: &'static str) {
    open.store
        .write(move |tx: &Transaction<'_>| {
            tx.execute(
                "INSERT INTO segment (chapter_id, ord, source_text, is_paragraph_end, target_text, \
                 is_target_paragraph_end, created_at, updated_at, status, is_omitted) \
                 VALUES (?1, 1, ?2, 1, ?3, 1, 't', 't', 'draft', 0)",
                rusqlite::params![chapter_id, source, target],
            )?;
            Ok(())
        })
        .expect("chen segment");
}

fn fixture(tag: &str) -> Fixture {
    let root = temp_dir(tag);
    let out = temp_dir(&format!("{tag}-out"));
    let open = create_work_from_text(&root, "Tac Pham", "zh", "", "Chuong mot.".to_owned()).expect("tao tac pham");
    open.store.write(|tx: &Transaction<'_>| tx.execute("DELETE FROM segment", [])).expect("xoa segment");
    let first = open.chapter_id;
    let second = open
        .store
        .write(|tx: &Transaction<'_>| {
            tx.execute(
                "INSERT INTO chapter (ord, title, source_text, status, created_at, updated_at) \
                 VALUES (2, 'Chuong B', 'x', 'not_started', 't', 't')",
                [],
            )?;
            Ok(tx.last_insert_rowid())
        })
        .expect("chen Chuong");
    open.store
        .write(move |tx: &Transaction<'_>| tx.execute("UPDATE chapter SET title = 'Chuong A' WHERE id = ?1", [first]))
        .expect("dat tieu de");
    add_segment(&open, first, "a goc", "a dich");
    add_segment(&open, second, "b goc", "b dich");
    Fixture { open, second, out }
}

fn set_origin(f: &mut Fixture, second: bool, author: &str, site: &str, url: &str, published: &str) {
    let chapter_id = if second { f.second } else { f.open.chapter_id };
    update_chapter_origin(Some(&mut f.open), chapter_id, author, site, url, published, None).expect("dat xuat xu");
}

fn translator(name: &str) -> Attribution {
    Attribution { translator: Some(name.to_owned()) }
}

fn md(f: &Fixture, attribution: Option<&Attribution>) -> String {
    let file = export_text(Some(&f.open), &ExportScope::Work, ImageMode::Link, TextFormat::Markdown, attribution, &f.out)
        .expect("xuat md");
    fs::read_to_string(file.path).expect("doc md")
}

fn txt(f: &Fixture, attribution: Option<&Attribution>) -> String {
    let file = export_text(Some(&f.open), &ExportScope::Work, ImageMode::Link, TextFormat::Plain, attribution, &f.out)
        .expect("xuat txt");
    fs::read_to_string(file.path).expect("doc txt")
}

fn two_column(f: &Fixture, attribution: Option<&Attribution>) -> DocxParsed {
    let file =
        export_docx_two_column(Some(&f.open), &ExportScope::Work, ImageMode::Link, attribution, &f.out).expect("xuat");
    read_docx(&fs::read(file.path).expect("doc tep")).expect("doc docx")
}

fn one_block(f: &Fixture, attribution: Option<&Attribution>) -> DocxParsed {
    let file =
        export_docx_one_block(Some(&f.open), &ExportScope::Work, ImageMode::Link, attribution, &f.out).expect("xuat");
    read_docx(&fs::read(file.path).expect("doc tep")).expect("doc docx")
}

const FULL: [&str; 4] = ["Tác giả: Tg", "Nguồn: Site · https://x.test/c1", "Ngày đăng gốc: 2026-01-02", "Người dịch: Ice"];

#[test]
fn off_writes_no_block_in_any_format() {
    let mut f = fixture("off");
    set_origin(&mut f, false, "Tg", "Site", "https://x.test/c1", "2026-01-02");
    assert_eq!(md(&f, None), "## Chuong A\n\na dich\n\n## Chuong B\n\nb dich\n");
    assert_eq!(txt(&f, None), "Chuong A\n\na dich\n\nChuong B\n\nb dich\n");
    for doc in [two_column(&f, None), one_block(&f, None)] {
        assert!(!doc.text.contains("Tác giả"), "{}", doc.text);
    }
}

#[test]
fn full_block_precedes_the_heading_with_hard_breaks_in_markdown() {
    let mut f = fixture("full-md");
    set_origin(&mut f, false, "Tg", "Site", "https://x.test/c1", "2026-01-02");
    let text = md(&f, Some(&translator("Ice")));
    let expected = "Tác giả: Tg\\\nNguồn: Site · https://x.test/c1\\\nNgày đăng gốc: 2026-01-02\\\nNgười dịch: Ice\n\n## Chuong A\n";
    assert!(text.starts_with(expected), "{text}");
}

#[test]
fn full_block_is_plain_lines_in_text_and_one_paragraph_before_the_title_in_both_docx() {
    let mut f = fixture("full-other");
    set_origin(&mut f, false, "Tg", "Site", "https://x.test/c1", "2026-01-02");
    let a = translator("Ice");
    let plain = txt(&f, Some(&a));
    assert!(plain.starts_with(&format!("{}\n{}\n{}\n{}\n\nChuong A\n", FULL[0], FULL[1], FULL[2], FULL[3])), "{plain}");
    for doc in [two_column(&f, Some(&a)), one_block(&f, Some(&a))] {
        let block = doc.text.find("Tác giả: Tg").expect("co khoi");
        let title = doc.text.find("Chuong A").expect("co tieu de");
        assert!(block < title, "{}", doc.text);
        assert!(doc.text.contains("Người dịch: Ice"));
    }
}

#[test]
fn blocks_leave_docx_table_shape_unchanged() {
    let mut f = fixture("shape");
    set_origin(&mut f, false, "Tg", "Site", "https://x.test/c1", "2026-01-02");
    let a = translator("Ice");
    let off = two_column(&f, None);
    let on = two_column(&f, Some(&a));
    assert_eq!(off.tables.len(), on.tables.len());
    for (x, y) in off.tables.iter().zip(&on.tables) {
        assert_eq!(x.rows, y.rows);
        assert_eq!(x.cells_per_row, y.cells_per_row);
        assert_eq!(x.paragraphs_per_cell, y.paragraphs_per_cell);
    }
    let off = one_block(&f, None);
    let on = one_block(&f, Some(&a));
    assert_eq!(off.tables.len(), on.tables.len());
    for (x, y) in off.tables.iter().zip(&on.tables) {
        assert_eq!((x.rows, &x.cells_per_row, &x.paragraphs_per_cell), (y.rows, &y.cells_per_row, &y.paragraphs_per_cell));
    }
}

#[test]
fn missing_date_and_url_keep_only_the_site() {
    let mut f = fixture("partial");
    set_origin(&mut f, false, "", "Site", "", "");
    let plain = txt(&f, Some(&translator("Ice")));
    assert!(plain.starts_with("Nguồn: Site\nNgười dịch: Ice\n\nChuong A\n"), "{plain}");
    assert!(!plain.contains("Ngày đăng gốc"));
    assert!(!plain.contains('·'));
    for doc in [two_column(&f, Some(&translator("Ice"))), one_block(&f, Some(&translator("Ice")))] {
        assert!(doc.text.contains("Nguồn: Site"), "{}", doc.text);
        assert!(!doc.text.contains("Ngày đăng gốc") && !doc.text.contains('·'), "{}", doc.text);
    }
}

#[test]
fn an_unset_translator_name_drops_only_that_line() {
    let mut f = fixture("no-name");
    set_origin(&mut f, false, "Tg", "", "", "");
    let plain = txt(&f, Some(&Attribution { translator: None }));
    assert!(plain.starts_with("Tác giả: Tg\n\nChuong A\n"), "{plain}");
    assert!(!plain.contains("Người dịch"));
    let none = Attribution { translator: None };
    for doc in [two_column(&f, Some(&none)), one_block(&f, Some(&none))] {
        assert!(doc.text.contains("Tác giả: Tg") && !doc.text.contains("Người dịch"), "{}", doc.text);
    }
}

#[test]
fn a_chapter_with_no_origin_and_no_name_has_no_block() {
    let f = fixture("empty");
    let none = Attribution { translator: None };
    assert_eq!(txt(&f, Some(&none)), txt(&f, None));
    assert_eq!(md(&f, Some(&none)), "## Chuong A\n\na dich\n\n## Chuong B\n\nb dich\n");
    for doc in [two_column(&f, Some(&none)), one_block(&f, Some(&none))] {
        assert!(!doc.text.contains("Tác giả") && !doc.text.contains("Nguồn") && !doc.text.contains("Người dịch"), "{}", doc.text);
    }
}

#[test]
fn each_chapter_carries_its_own_origin_and_edits_show_on_the_next_export() {
    let mut f = fixture("multi");
    set_origin(&mut f, false, "Tg A", "", "", "");
    set_origin(&mut f, true, "Tg B", "", "", "");
    let a = translator("Ice");
    let first_run = txt(&f, Some(&a));
    assert!(first_run.contains("Tác giả: Tg A\nNgười dịch: Ice\n\nChuong A"), "{first_run}");
    assert!(first_run.contains("Tác giả: Tg B\nNgười dịch: Ice\n\nChuong B"), "{first_run}");
    set_origin(&mut f, true, "Tg C", "", "", "");
    let again = txt(&f, Some(&a));
    assert!(again.contains("Tác giả: Tg C"), "{again}");
    assert!(!again.contains("Tg B"));
    for doc in [two_column(&f, Some(&a)), one_block(&f, Some(&a))] {
        assert!(doc.text.contains("Tác giả: Tg A") && doc.text.contains("Tác giả: Tg C"), "{}", doc.text);
        assert!(!doc.text.contains("Tg B"), "{}", doc.text);
        let (a_at, b_at) = (doc.text.find("Tg A").expect("A"), doc.text.find("Tg C").expect("C"));
        assert!(a_at < doc.text.find("Chuong A").expect("tieu de A") && doc.text.find("Chuong A") < Some(b_at), "{}", doc.text);
    }
}

#[test]
fn attribution_on_with_no_global_store_is_an_error_not_a_silent_drop() {
    assert!(attribution_of(true, None).is_err());
    assert_eq!(attribution_of(false, None).expect("tat"), None);
}

#[test]
fn the_translator_name_round_trips_through_the_global_store_and_blank_clears_it() {
    let dir = temp_dir("global");
    let global = open_global(&dir);
    assert_eq!(translator_name_get(Some(&global)).expect("doc"), None);
    translator_name_save(Some(&global), "  Ice  ").expect("ghi");
    assert_eq!(translator_name_get(Some(&global)).expect("doc"), Some("Ice".to_owned()));
    translator_name_save(Some(&global), "   ").expect("xoa");
    assert_eq!(translator_name_get(Some(&global)).expect("doc"), None);
    assert!(translator_name_get(None).is_err(), "kho global vang mat la loi, khong phai None");
}

#[test]
fn attribution_on_carries_the_stored_translator_name() {
    let dir = temp_dir("of");
    let global = open_global(&dir);
    assert_eq!(attribution_of(true, Some(&global)).expect("bat"), Some(Attribution { translator: None }));
    translator_name_save(Some(&global), "Ice").expect("ghi");
    assert_eq!(attribution_of(true, Some(&global)).expect("bat"), Some(translator("Ice")));
    assert_eq!(attribution_of(false, Some(&global)).expect("tat"), None);
}
