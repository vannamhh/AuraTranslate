//! Nhập lại bản reviewer (FR90, AD-52): đọc, khớp, xem trước, xác nhận, và các đường gộp/tách Chương.
//! Mọi tệp được dựng bằng hàm xuất thật.

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use auratranslate_lib::commands::chapter::{merge_chapter_into_previous, split_chapter_at_segment};
use auratranslate_lib::commands::export::{
    PendingReviewerImportState, ReviewerImportPreviewWire, export_docx_one_block, export_docx_two_column, export_text,
    reviewer_import_cancel, reviewer_import_confirm, reviewer_import_preview,
};
use auratranslate_lib::commands::project::{OpenWork, create_work_from_text};
use auratranslate_lib::core::export::{
    Attribution, ExportScope, ImageMode, ReviewCopyError, ReviewFileKind, ReviewRowKind, TextFormat, read_review_copy,
};
use auratranslate_lib::core::i18n::IpcError;
use auratranslate_lib::core::store::Transaction;

#[path = "support/boundary_scan.rs"]
#[allow(dead_code)] // shared module: not every helper is used in this file
mod boundary_scan;
use boundary_scan::{code_lines, matching_close_brace, src_root, without_test_modules};

static NEXT_DIR: AtomicU64 = AtomicU64::new(0);

fn temp_dir(tag: &str) -> PathBuf {
    let n = NEXT_DIR.fetch_add(1, Ordering::Relaxed);
    let dir = std::env::temp_dir().join(format!("auratranslate-reviewer-{}-{}-{}", std::process::id(), tag, n));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap_or_else(|e| panic!("tao {}: {e}", dir.display()));
    dir
}

struct Chap {
    title: Option<&'static str>,
    segs: Vec<(&'static str, &'static str)>,
}

fn chap(title: Option<&'static str>, segs: &[(&'static str, &'static str)]) -> Chap {
    Chap { title, segs: segs.to_vec() }
}

struct Work {
    open: OpenWork,
    chapters: Vec<i64>,
    root: PathBuf,
    out: PathBuf,
}

fn insert_segments(open: &OpenWork, chapter_id: i64, segs: &[(&'static str, &'static str, Option<&'static str>)]) {
    let segs = segs.to_vec();
    open.store
        .write(move |tx: &Transaction<'_>| {
            for (i, (source, target, role)) in segs.iter().enumerate() {
                tx.execute(
                    "INSERT INTO segment (chapter_id, ord, source_text, is_paragraph_end, target_text, \
                     is_target_paragraph_end, created_at, updated_at, status, is_omitted, role) \
                     VALUES (?1, ?2, ?3, 1, ?4, 1, 't', 't', 'draft', 0, ?5)",
                    rusqlite::params![chapter_id, i64::try_from(i).unwrap_or(0) + 1, source, target, role],
                )?;
            }
            Ok(())
        })
        .expect("chen segment");
}

fn build(tag: &str, chapters: &[Chap]) -> Work {
    build_with_roles(tag, &chapters.iter().map(|c| (c.title, c.segs.iter().map(|&(s, t)| (s, t, None)).collect())).collect::<Vec<_>>())
}

fn build_with_roles(tag: &str, chapters: &[(Option<&'static str>, Vec<(&'static str, &'static str, Option<&'static str>)>)]) -> Work {
    let root = temp_dir(tag);
    let out = temp_dir(&format!("{tag}-out"));
    let open = create_work_from_text(&root, "Tac Pham", "zh", "", "Chuong mot.".to_owned()).expect("tao tac pham");
    open.store.write(|tx: &Transaction<'_>| tx.execute("DELETE FROM segment", [])).expect("xoa segment");
    let mut ids = Vec::new();
    for (i, (title, segs)) in chapters.iter().enumerate() {
        let id = if i == 0 {
            open.chapter_id
        } else {
            let ord = i64::try_from(i).unwrap_or(0) + 1;
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
        let title = title.map(str::to_owned);
        open.store
            .write(move |tx: &Transaction<'_>| tx.execute("UPDATE chapter SET title = ?1 WHERE id = ?2", (title, id)))
            .expect("dat tieu de");
        insert_segments(&open, id, segs);
        ids.push(id);
    }
    Work { open, chapters: ids, root, out }
}

impl Work {
    fn asset(&self, chapter: usize, file_name: &str, source_url: Option<&str>, anchor: i64, with_file: bool) {
        let (file_name, source_url, chapter_id) = (file_name.to_owned(), source_url.map(str::to_owned), self.chapters[chapter]);
        let name = file_name.clone();
        self.open
            .store
            .write(move |tx: &Transaction<'_>| {
                tx.execute(
                    "INSERT INTO asset (chapter_id, file_name, source_url, anchor_after_segment_ord, byte_len, \
                     content_type, created_at) VALUES (?1, ?2, ?3, ?4, 10, 'image/jpeg', 't')",
                    rusqlite::params![chapter_id, file_name, source_url, anchor],
                )?;
                Ok(())
            })
            .expect("chen asset");
        if with_file {
            let dir = self.open.dir.join("assets");
            fs::create_dir_all(&dir).expect("tao assets");
            fs::write(dir.join(name), b"jpg").expect("ghi anh");
        }
    }

    fn docx(&self, mode: ImageMode) -> PathBuf {
        let file = export_docx_two_column(Some(&self.open), &ExportScope::Work, mode, None, &self.out).expect("xuat docx");
        PathBuf::from(file.path)
    }

    fn markdown(&self, attribution: Option<&Attribution>) -> PathBuf {
        let file =
            export_text(Some(&self.open), &ExportScope::Work, ImageMode::Link, TextFormat::Markdown, attribution, &self.out)
                .expect("xuat md");
        PathBuf::from(file.path)
    }

    fn preview(&self, pending: &PendingReviewerImportState, path: &Path) -> Result<ReviewerImportPreviewWire, IpcError> {
        reviewer_import_preview(Some(&self.open), pending, path)
    }

    fn confirm(&self, pending: &PendingReviewerImportState) -> Result<auratranslate_lib::commands::export::ReviewerImportSummaryWire, IpcError> {
        reviewer_import_confirm(Some(&self.open), None, pending)
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
                            let cells = (0..columns).map(|i| format!("{:?}", row.get_ref(i))).collect::<Vec<_>>();
                            Ok(cells.join("|"))
                        })?
                        .collect::<rusqlite::Result<Vec<_>>>()?;
                    out.insert(name, rows);
                }
                let version: i64 = conn.query_row("PRAGMA data_version", [], |r| r.get(0))?;
                out.insert("pragma:data_version".to_owned(), vec![version.to_string()]);
                Ok(out)
            })
            .expect("dump")
    }

    fn count(&self, table: &str) -> i64 {
        let sql = format!("SELECT COUNT(*) FROM {table}");
        self.open.store.read(move |conn| conn.query_row(&sql, [], |r| r.get(0))).expect("dem")
    }

    fn finish(self) {
        let dir = self.open.dir.clone();
        drop(self.open);
        let _ = fs::remove_dir_all(&dir);
        let _ = fs::remove_dir_all(&self.root);
        let _ = fs::remove_dir_all(&self.out);
    }
}

fn pending() -> PendingReviewerImportState {
    PendingReviewerImportState::new(None)
}

fn is_pending(state: &PendingReviewerImportState) -> bool {
    state.lock().unwrap_or_else(std::sync::PoisonError::into_inner).is_some()
}

fn without_review_tables(mut dump: BTreeMap<String, Vec<String>>) -> BTreeMap<String, Vec<String>> {
    dump.remove("review_chapter");
    dump.remove("review_row");
    dump.remove("alignment_group");
    dump.remove("alignment_member");
    dump.remove("review_decision");
    dump.remove("pragma:data_version");
    dump
}

fn two_chapters(targets: [&'static str; 2]) -> Vec<Chap> {
    vec![
        chap(Some("Mo dau"), &[("a1", targets[0]), ("a2", "dich a2"), ("a3", "dich a3")]),
        chap(None, &[("b1", "dich b1"), ("b2", targets[1])]),
    ]
}

fn reviewer_targets(tag: &str) -> (Work, Work) {
    let mine = build(&format!("{tag}-mine"), &two_chapters(["dich a1", "dich b2"]));
    let reviewer = build(&format!("{tag}-rev"), &two_chapters(["a1 da sua", "b2 da sua"]));
    (mine, reviewer)
}

// ───────────────────────── .docx: the I/O matrix ─────────────────────────

#[test]
fn an_edited_two_column_docx_previews_each_chapter_with_its_row_count_and_writes_nothing() {
    let (mine, reviewer) = reviewer_targets("preview");
    let file = reviewer.docx(ImageMode::File);
    let before = mine.dump();
    let state = pending();

    let preview = mine.preview(&state, &file).expect("xem truoc");

    assert_eq!(preview.file_kind, "docx");
    let shape: Vec<(i64, Option<&str>, i64)> =
        preview.chapters.iter().map(|c| (c.chapter_ord, c.title.as_deref(), c.row_count)).collect();
    assert_eq!(shape, vec![(1, Some("Mo dau"), 3), (2, None, 2)]);
    assert!(preview.skipped.is_empty() && preview.chapters.iter().all(|c| c.replaces.is_none()));
    assert!(is_pending(&state));
    assert_eq!(mine.dump(), before, "xem truoc khong duoc ghi byte nao, ke ca data_version");
    mine.finish();
    reviewer.finish();
}

#[test]
fn confirming_writes_one_copy_per_chapter_and_leaves_every_segment_column_byte_for_byte() {
    let (mine, reviewer) = reviewer_targets("confirm");
    let file = reviewer.docx(ImageMode::File);
    let state = pending();
    mine.preview(&state, &file).expect("xem truoc");
    let before = without_review_tables(mine.dump());

    let summary = mine.confirm(&state).expect("xac nhan");

    assert_eq!((summary.chapter_count, summary.row_count, summary.replaced_count), (2, 5, 0));
    assert!(!is_pending(&state));
    assert_eq!(without_review_tables(mine.dump()), before, "segment, baseline va cot xuat xu phai nguyen ven");
    assert_eq!((mine.count("review_chapter"), mine.count("review_row")), (2, 5));
    let copy = read_review_copy(&mine.open.store, mine.chapters[0]).expect("doc ban");
    assert_eq!(copy.file_kind, ReviewFileKind::Docx);
    let rows: Vec<(ReviewRowKind, Option<&str>, &str)> =
        copy.rows.iter().map(|r| (r.kind, r.source_text.as_deref(), r.target_text.as_str())).collect();
    assert_eq!(
        rows,
        vec![
            (ReviewRowKind::Text, Some("a1"), "a1 da sua"),
            (ReviewRowKind::Text, Some("a2"), "dich a2"),
            (ReviewRowKind::Text, Some("a3"), "dich a3"),
        ]
    );
    mine.finish();
    reviewer.finish();
}

fn assert_image_rows_are_skipped(tag: &str, mode: ImageMode) {
    let chapters = vec![chap(Some("Anh"), &[("giong", "giong"), ("p", "p da sua"), ("q", "q da sua")])];
    let mine = build(&format!("{tag}-mine"), &chapters);
    let reviewer = build(&format!("{tag}-rev"), &chapters);
    for work in [&mine, &reviewer] {
        work.asset(0, "a.jpg", Some("https://x.test/a.jpg"), 1, true);
        work.asset(0, "b.jpg", None, 2, true);
    }
    let file = reviewer.docx(mode);
    let state = pending();
    let segments_before = mine.dump()["segment"].clone();

    let preview = mine.preview(&state, &file).expect("xem truoc");

    let expected_images = if mode == ImageMode::Link { 1 } else { 2 };
    assert_eq!(preview.image_rows_ignored, expected_images, "mode {mode:?}: link mode drops the image with no URL");
    assert_eq!(preview.chapters[0].row_count, 3, "hang giong het o hai cot nhung khong phai anh van la mot hang");
    mine.confirm(&state).expect("xac nhan");
    let texts: Vec<String> = mine
        .open
        .store
        .read(|conn| {
            conn.prepare("SELECT target_text FROM review_row ORDER BY ord")?
                .query_map([], |r| r.get::<_, String>(0))?
                .collect::<rusqlite::Result<Vec<_>>>()
        })
        .expect("doc hang");
    assert_eq!(texts, vec!["giong", "p da sua", "q da sua"]);
    assert_eq!(mine.dump()["segment"], segments_before, "hang anh khong tao hay doi segment nao");
    mine.finish();
    reviewer.finish();
}

#[test]
fn image_rows_written_as_links_are_recognised_and_skipped() {
    assert_image_rows_are_skipped("img-link", ImageMode::Link);
}

#[test]
fn image_rows_written_as_copied_files_are_recognised_and_skipped() {
    assert_image_rows_are_skipped("img-file", ImageMode::File);
}

fn error_code(result: Result<ReviewerImportPreviewWire, IpcError>) -> String {
    result.expect_err("phai tu choi").code().to_owned()
}

#[test]
fn a_one_block_publishing_copy_is_refused_with_its_own_key_and_nothing_is_written() {
    let mine = build("block-mine", &two_chapters(["dich a1", "dich b2"]));
    let reviewer = build("block-rev", &two_chapters(["a1 da sua", "b2 da sua"]));
    let file = export_docx_one_block(Some(&reviewer.open), &ExportScope::Work, ImageMode::File, None, &reviewer.out)
        .expect("xuat mot khoi");
    let before = mine.dump();
    let state = pending();

    let error = mine.preview(&state, Path::new(&file.path)).expect_err("phai tu choi");

    assert_eq!(error.code(), "export.publish_copy_not_reimportable");
    assert!(!is_pending(&state));
    assert_eq!(mine.dump(), before);
    mine.finish();
    reviewer.finish();
}

#[test]
fn a_file_from_another_work_names_the_open_work_and_writes_nothing() {
    let mine = build("other-mine", &two_chapters(["dich a1", "dich b2"]));
    let reviewer = build("other-rev", &[chap(Some("Mo dau"), &[("khac 1", "x"), ("khac 2", "y")])]);
    let file = reviewer.docx(ImageMode::File);
    let before = mine.dump();
    let state = pending();

    let error = mine.preview(&state, &file).expect_err("phai tu choi");

    assert_eq!(error.code(), "export.reviewer_import_wrong_work");
    assert_eq!(error.params().get("work_name").map(String::as_str), Some("Tac Pham"));
    assert_eq!(mine.dump(), before);
    mine.finish();
    reviewer.finish();
}

#[test]
fn a_title_alone_never_matches_a_docx_chapter_whose_source_cells_are_absent() {
    let mine = build("title-mine", &two_chapters(["dich a1", "dich b2"]));
    let reviewer = build("title-rev", &[chap(Some("Mo dau"), &[("khong co trong tac pham", "y")])]);
    let state = pending();
    assert_eq!(error_code(mine.preview(&state, &reviewer.docx(ImageMode::File))), "export.reviewer_import_wrong_work");
    mine.finish();
    reviewer.finish();
}

#[test]
fn a_section_that_matches_nothing_is_listed_as_skipped_while_the_others_still_import() {
    let mine = build("skip-mine", &two_chapters(["dich a1", "dich b2"]));
    let reviewer = build(
        "skip-rev",
        &[
            chap(Some("Mo dau"), &[("a1", "sua"), ("a2", "sua"), ("a3", "sua")]),
            chap(Some("La"), &[("khong ai biet", "x")]),
        ],
    );
    let state = pending();

    let preview = mine.preview(&state, &reviewer.docx(ImageMode::File)).expect("xem truoc");

    assert_eq!(preview.chapters.len(), 1);
    assert_eq!(preview.chapters[0].chapter_ord, 1);
    assert_eq!(
        preview.skipped.iter().map(|s| (s.heading.as_str(), s.row_count)).collect::<Vec<_>>(),
        vec![("La", 1)]
    );
    mine.finish();
    reviewer.finish();
}

#[test]
fn two_sections_aimed_at_the_same_chapter_are_both_treated_as_unmatched() {
    let mine = build("dup-mine", &two_chapters(["dich a1", "dich b2"]));
    let reviewer = build(
        "dup-rev",
        &[chap(Some("X"), &[("a1", "1"), ("a2", "2")]), chap(Some("Y"), &[("a3", "3"), ("a1", "4")])],
    );
    let state = pending();
    assert_eq!(error_code(mine.preview(&state, &reviewer.docx(ImageMode::File))), "export.reviewer_import_wrong_work");
    mine.finish();
    reviewer.finish();
}

// ───────────────────────── .md ─────────────────────────

#[test]
fn an_edited_markdown_export_matches_by_heading_including_the_untitled_chapter_form() {
    let (mine, reviewer) = reviewer_targets("md");
    let file = reviewer.markdown(None);
    let state = pending();

    let preview = mine.preview(&state, &file).expect("xem truoc");

    assert_eq!(preview.file_kind, "md");
    let shape: Vec<(i64, i64)> = preview.chapters.iter().map(|c| (c.chapter_ord, c.row_count)).collect();
    assert_eq!(shape, vec![(1, 3), (2, 2)]);
    mine.confirm(&state).expect("xac nhan");
    let copy = read_review_copy(&mine.open.store, mine.chapters[1]).expect("doc ban");
    assert_eq!(copy.file_kind, ReviewFileKind::Markdown);
    assert!(copy.rows.iter().all(|r| r.source_text.is_none() && r.kind == ReviewRowKind::Text));
    assert_eq!(copy.rows.iter().map(|r| r.target_text.as_str()).collect::<Vec<_>>(), vec!["dich b1", "b2 da sua"]);
    mine.finish();
    reviewer.finish();
}

#[test]
fn markdown_escapes_round_trip_and_the_attribution_block_is_not_glued_onto_the_previous_chapter() {
    let lines = ["# a", "> b", "- c", "1. e", "g*h_i`j<k&l[m]n\\o", "~~~"];
    let chapters = vec![
        chap(Some("Mot"), &[("s1", "mot 1"), ("s2", "mot 2")]),
        chap(Some("Hai #"), &lines.iter().map(|l| ("s", *l)).collect::<Vec<_>>()),
    ];
    let reviewer = build("md-esc-rev", &chapters);
    let mine = build("md-esc-mine", &chapters);
    let chapter_two = reviewer.chapters[1];
    reviewer
        .open
        .store
        .write(move |tx: &Transaction<'_>| {
            tx.execute("UPDATE chapter SET origin_author = 'Tac Gia' WHERE id = ?1", [chapter_two])
        })
        .expect("dat tac gia");
    let file = reviewer.markdown(Some(&Attribution { translator: Some("Nguoi Dich".to_owned()) }));
    let text = fs::read_to_string(&file).expect("doc tep");
    assert!(text.contains("Nguoi Dich"), "ca hai Chuong deu co khoi ghi nguon: {text}");
    let state = pending();

    let preview = mine.preview(&state, &file).expect("xem truoc");

    assert_eq!(preview.chapters.iter().map(|c| c.row_count).collect::<Vec<_>>(), vec![2, 6]);
    mine.confirm(&state).expect("xac nhan");
    let first = read_review_copy(&mine.open.store, mine.chapters[0]).expect("doc ban 1");
    assert_eq!(first.rows.iter().map(|r| r.target_text.as_str()).collect::<Vec<_>>(), vec!["mot 1", "mot 2"]);
    let second = read_review_copy(&mine.open.store, mine.chapters[1]).expect("doc ban 2");
    assert_eq!(second.rows.iter().map(|r| r.target_text.as_str()).collect::<Vec<_>>(), lines.to_vec());
    mine.finish();
    reviewer.finish();
}

#[test]
fn markdown_image_lines_become_alt_and_caption_rows_and_never_the_image_reference() {
    let segs = vec![("p", "Hello", None), ("goc", "Anh [dep]", Some("alt")), ("goc", "Chu *thich*", Some("caption")), ("q", "Sau", None)];
    let reviewer = build_with_roles("md-img-rev", &[(Some("T"), segs.clone())]);
    let mine = build_with_roles("md-img-mine", &[(Some("T"), segs)]);
    reviewer.asset(0, "a.jpg", Some("https://x.test/a.jpg"), 1, false);
    let file = reviewer.markdown(None);
    let state = pending();

    mine.preview(&state, &file).expect("xem truoc");
    mine.confirm(&state).expect("xac nhan");

    let copy = read_review_copy(&mine.open.store, mine.chapters[0]).expect("doc ban");
    let rows: Vec<(ReviewRowKind, &str)> = copy.rows.iter().map(|r| (r.kind, r.target_text.as_str())).collect();
    assert_eq!(
        rows,
        vec![
            (ReviewRowKind::Text, "Hello"),
            (ReviewRowKind::Alt, "Anh [dep]"),
            (ReviewRowKind::Caption, "Chu *thich*"),
            (ReviewRowKind::Text, "Sau"),
        ]
    );
    assert!(copy.rows.iter().all(|r| !r.target_text.contains("x.test")));
    mine.finish();
    reviewer.finish();
}

#[test]
fn a_markdown_file_with_no_matching_heading_is_the_wrong_work_error() {
    let mine = build("md-wrong-mine", &two_chapters(["dich a1", "dich b2"]));
    let reviewer = build("md-wrong-rev", &[chap(Some("Tieu de la"), &[("x", "y")])]);
    let state = pending();
    assert_eq!(error_code(mine.preview(&state, &reviewer.markdown(None))), "export.reviewer_import_wrong_work");
    mine.finish();
    reviewer.finish();
}

// ───────────────────────── preview lifecycle ─────────────────────────

#[test]
fn cancelling_drops_the_pending_plan_and_leaves_the_database_untouched() {
    let (mine, reviewer) = reviewer_targets("cancel");
    let file = reviewer.docx(ImageMode::File);
    let state = pending();
    let before = mine.dump();
    mine.preview(&state, &file).expect("xem truoc");

    reviewer_import_cancel(&state);

    assert!(!is_pending(&state));
    assert_eq!(mine.dump(), before);
    let error = mine.confirm(&state).expect_err("khong con gi de xac nhan");
    assert_eq!(error.code(), "export.reviewer_import_no_pending");
    mine.finish();
    reviewer.finish();
}

#[test]
fn a_failed_preview_also_drops_the_plan_left_by_an_earlier_one() {
    let (mine, reviewer) = reviewer_targets("replace-pending");
    let good = reviewer.docx(ImageMode::File);
    let bad = reviewer.out.join("khong-phai.txt");
    fs::write(&bad, "x").expect("ghi tep");
    let state = pending();
    mine.preview(&state, &good).expect("xem truoc");
    assert!(is_pending(&state));

    assert_eq!(error_code(mine.preview(&state, &bad)), "export.reviewer_import_unreadable");

    assert!(!is_pending(&state));
    mine.finish();
    reviewer.finish();
}

#[test]
fn every_refused_file_leaves_every_table_and_the_data_version_unchanged() {
    let mine = build("refuse-mine", &two_chapters(["dich a1", "dich b2"]));
    let dir = temp_dir("refuse-files");
    let bad_extension = dir.join("a.txt");
    let garbage_docx = dir.join("a.docx");
    let no_headings = dir.join("a.md");
    let not_utf8 = dir.join("b.md");
    fs::write(&bad_extension, "x").expect("ghi");
    fs::write(&garbage_docx, b"khong phai zip").expect("ghi");
    fs::write(&no_headings, "chi co mot doan\n\nva mot doan nua\n").expect("ghi");
    fs::write(&not_utf8, [0xff, 0xfe, 0xfd]).expect("ghi");
    let missing = dir.join("khong-ton-tai.docx");
    let before = mine.dump();
    let state = pending();

    for path in [&bad_extension, &garbage_docx, &no_headings, &not_utf8, &missing] {
        let error = mine.preview(&state, path).expect_err("phai tu choi");
        assert_eq!(error.code(), "export.reviewer_import_unreadable", "{}", path.display());
        assert_eq!(mine.dump(), before, "{}", path.display());
        assert!(!is_pending(&state));
    }
    assert_eq!(reviewer_import_preview(None, &state, &bad_extension).expect_err("khong co tac pham").code(), "work.none_open");
    let _ = fs::remove_dir_all(&dir);
    mine.finish();
}

// ───────────────────────── confirm replaces, never duplicates ─────────────────────────

#[test]
fn importing_again_replaces_the_chapter_copy_with_a_fresh_id_and_the_preview_says_so() {
    let (mine, reviewer) = reviewer_targets("again");
    let state = pending();
    mine.preview(&state, &reviewer.docx(ImageMode::File)).expect("xem truoc");
    mine.confirm(&state).expect("lan mot");
    let first_ids = ids(&mine);

    let second_reviewer = build("again-rev2", &two_chapters(["lan hai", "lan hai b"]));
    let second_file = second_reviewer.docx(ImageMode::File);
    let preview = mine.preview(&state, &second_file).expect("xem truoc lan hai");
    assert!(preview.chapters.iter().all(|c| c.replaces.as_ref().is_some_and(|r| !r.stale)));
    let summary = mine.confirm(&state).expect("lan hai");

    assert_eq!(summary.replaced_count, 2);
    let second_ids = ids(&mine);
    assert_eq!(second_ids.len(), 2, "moi Chuong dung mot review_chapter");
    assert!(second_ids.iter().zip(&first_ids).all(|(new, old)| new > old), "{first_ids:?} -> {second_ids:?}");
    assert_eq!(mine.count("review_row"), 5, "hang cu da bi xoa theo ban cu");
    let copy = read_review_copy(&mine.open.store, mine.chapters[0]).expect("doc ban");
    assert_eq!(copy.rows[0].target_text, "lan hai");
    mine.finish();
    reviewer.finish();
    second_reviewer.finish();
}

#[test]
fn the_preview_counts_accepted_groups_and_confirming_deletes_their_decisions_in_the_same_write() {
    let (mine, reviewer) = reviewer_targets("accepted-lost");
    let state = pending();
    mine.preview(&state, &reviewer.docx(ImageMode::File)).expect("xem truoc");
    mine.confirm(&state).expect("lan mot");
    let first = auratranslate_lib::core::export::review_diff(&mine.open.store, mine.chapters[0]).expect("diff");
    let changed = first.iter().find(|g| g.spans.iter().any(|s| s.kind != auratranslate_lib::core::matching::DiffKind::Equal)).expect("nhom doi");
    let group = changed.group_id;
    auratranslate_lib::core::export::skip_change(&mine.open.store, mine.chapters[0], group).expect("bo qua");
    let accepted = auratranslate_lib::commands::segment::review_accept_change(
        Some(&mine.open),
        mine.chapters[0],
        group,
        "dich a1",
        true,
    )
    .expect("chap nhan");
    assert_eq!(accepted.target_text, "a1 da sua");
    assert_eq!(mine.count("review_decision"), 1);

    let again = build("accepted-lost-rev2", &two_chapters(["lan hai", "lan hai b"]));
    let preview = mine.preview(&state, &again.docx(ImageMode::File)).expect("xem truoc lan hai");
    let counts: Vec<(i64, i64)> =
        preview.chapters.iter().filter_map(|c| c.replaces.as_ref()).map(|r| (r.user_group_count, r.accepted_group_count)).collect();
    assert_eq!(counts, vec![(0, 1), (0, 0)]);
    assert_eq!(mine.count("review_decision"), 1, "xem truoc khong ghi gi");

    mine.confirm(&state).expect("lan hai");
    assert_eq!(mine.count("review_decision"), 0);
    mine.finish();
    reviewer.finish();
    again.finish();
}

fn ids(work: &Work) -> Vec<i64> {
    work.open
        .store
        .read(|conn| {
            conn.prepare("SELECT id FROM review_chapter ORDER BY chapter_id")?
                .query_map([], |r| r.get::<_, i64>(0))?
                .collect::<rusqlite::Result<Vec<_>>>()
        })
        .expect("doc id")
}

#[test]
fn confirming_after_the_matched_chapters_changed_writes_nothing_and_reports_it() {
    let (mine, reviewer) = reviewer_targets("stale-confirm");
    let state = pending();
    mine.preview(&state, &reviewer.docx(ImageMode::File)).expect("xem truoc");
    let mut open = mine.open;
    let second = mine.chapters[1];
    merge_chapter_into_previous(Some(&mut open), second).expect("gop");
    let mine = Work { open, ..mine };
    let before = mine.dump();

    let error = mine.confirm(&state).expect_err("phai loi");

    assert_eq!(error.code(), "export.reviewer_import_preview_stale");
    assert_eq!(mine.dump(), before, "khong ghi gi, ke ca data_version");
    assert_eq!(mine.count("review_chapter"), 0);
    assert!(!is_pending(&state));
    mine.finish();
    reviewer.finish();
}

#[test]
fn confirming_when_only_some_of_the_previewed_chapters_still_match_writes_nothing() {
    let three = |a: &'static str| {
        vec![
            chap(Some("Mot"), &[("a1", a), ("a2", "x")]),
            chap(Some("Hai"), &[("b1", "y"), ("b2", "y")]),
            chap(Some("Ba"), &[("c1", "z"), ("c2", "z")]),
        ]
    };
    let mine = build("partial-mine", &three("dich"));
    let reviewer = build("partial-rev", &three("sua"));
    let state = pending();
    assert_eq!(mine.preview(&state, &reviewer.docx(ImageMode::File)).expect("xem truoc").chapters.len(), 3);
    let mut open = mine.open;
    let third = mine.chapters[2];
    merge_chapter_into_previous(Some(&mut open), third).expect("gop");
    let mine = Work { open, ..mine };
    let before = mine.dump();

    let error = mine.confirm(&state).expect_err("phai loi");

    assert_eq!(error.code(), "export.reviewer_import_preview_stale");
    assert_eq!(mine.dump(), before);
    mine.finish();
    reviewer.finish();
}

#[test]
fn a_write_failure_halfway_through_confirm_rolls_back_every_row_and_keeps_the_preview_for_a_retry() {
    let (mine, reviewer) = reviewer_targets("write-failure");
    let state = pending();
    mine.preview(&state, &reviewer.docx(ImageMode::File)).expect("xem truoc");
    mine.open
        .store
        .write(|tx| {
            tx.execute_batch(
                "CREATE TRIGGER fail_second_row BEFORE INSERT ON review_row WHEN NEW.ord = 1 \
                 BEGIN SELECT RAISE(ABORT, 'loi ghi gia'); END;",
            )
        })
        .expect("cai trigger");
    let before = mine.dump();

    mine.confirm(&state).expect_err("loi ghi phai thanh loi");

    assert_eq!(mine.dump(), before, "review_chapter da chen truoc loi phai bi rollback");
    assert_eq!((mine.count("review_chapter"), mine.count("review_row")), (0, 0));
    assert!(is_pending(&state), "loi ghi giu ban xem truoc de thu lai");
    mine.open.store.write(|tx| tx.execute_batch("DROP TRIGGER fail_second_row;")).expect("go trigger");
    assert_eq!(mine.confirm(&state).expect("thu lai").chapter_count, 2);
    mine.finish();
    reviewer.finish();
}

#[test]
fn confirming_after_an_asset_changed_so_the_rows_would_differ_writes_nothing() {
    let chapters = vec![chap(Some("Anh"), &[("p", "p da sua"), ("https://x.test/a.jpg", "https://x.test/a.jpg")])];
    let mine = build("asset-mine", &chapters);
    let reviewer = build("asset-rev", &chapters);
    mine.asset(0, "a.jpg", Some("https://x.test/a.jpg"), 1, false);
    let state = pending();
    let preview = mine.preview(&state, &reviewer.docx(ImageMode::File)).expect("xem truoc");
    assert_eq!((preview.chapters[0].row_count, preview.image_rows_ignored), (1, 1));
    mine.open
        .store
        .write(|tx: &Transaction<'_>| tx.execute("UPDATE asset SET file_name = 'b.jpg', source_url = NULL", []))
        .expect("doi ten anh");
    let before = mine.dump();

    let error = mine.confirm(&state).expect_err("phai loi");

    assert_eq!(error.code(), "export.reviewer_import_preview_stale");
    assert_eq!(mine.dump(), before);
    mine.finish();
    reviewer.finish();
}

// ───────────────────────── merge and split (AD-52 rule 5) ─────────────────────────

fn import_both(mine: &Work, reviewer: &Work) {
    let state = pending();
    mine.preview(&state, &reviewer.docx(ImageMode::File)).expect("xem truoc");
    mine.confirm(&state).expect("xac nhan");
    assert_eq!(mine.count("review_chapter"), 2);
}

#[test]
fn merging_b_into_a_deletes_the_copy_of_b_and_marks_the_copy_of_a_stale() {
    let (mine, reviewer) = reviewer_targets("merge");
    import_both(&mine, &reviewer);
    let (a, b) = (mine.chapters[0], mine.chapters[1]);
    let b_copy = read_review_copy(&mine.open.store, b).expect("ban cua B").id;
    let mut open = mine.open;
    merge_chapter_into_previous(Some(&mut open), b).expect("gop");
    let mine = Work { open, ..mine };

    assert!(matches!(read_review_copy(&mine.open.store, b), Err(ReviewCopyError::NotImported)));
    assert!(matches!(read_review_copy(&mine.open.store, a), Err(ReviewCopyError::Stale)));
    let orphan_rows: i64 = mine
        .open
        .store
        .read(move |conn| conn.query_row("SELECT COUNT(*) FROM review_row WHERE review_chapter_id = ?1", [b_copy], |r| r.get(0)))
        .expect("dem");
    assert_eq!((orphan_rows, mine.count("review_chapter")), (0, 1));
    assert_eq!(mine.count("review_row"), 3, "hang cua A van con, bat bien sau khi nhap");
    mine.finish();
    reviewer.finish();
}

#[test]
fn splitting_a_marks_its_copy_stale_and_the_new_chapter_has_none() {
    let (mine, reviewer) = reviewer_targets("split");
    import_both(&mine, &reviewer);
    let a = mine.chapters[0];
    let second_segment: i64 = mine
        .open
        .store
        .read(move |conn| conn.query_row("SELECT id FROM segment WHERE chapter_id = ?1 AND ord = 2", [a], |r| r.get(0)))
        .expect("tim segment");
    let mut open = mine.open;
    split_chapter_at_segment(Some(&mut open), second_segment).expect("tach");
    let mine = Work { open, ..mine };
    let new_chapter: i64 = mine
        .open
        .store
        .read(|conn| conn.query_row("SELECT MAX(id) FROM chapter", [], |r| r.get(0)))
        .expect("tim Chuong moi");

    assert!(matches!(read_review_copy(&mine.open.store, a), Err(ReviewCopyError::Stale)));
    assert!(matches!(read_review_copy(&mine.open.store, new_chapter), Err(ReviewCopyError::NotImported)));
    assert!(read_review_copy(&mine.open.store, mine.chapters[1]).is_ok(), "Chuong khong lien quan van doc duoc");
    mine.finish();
    reviewer.finish();
}

#[test]
fn a_chapter_that_was_never_imported_reads_as_not_imported_never_as_an_empty_list() {
    let mine = build("never", &two_chapters(["dich a1", "dich b2"]));
    assert!(matches!(read_review_copy(&mine.open.store, mine.chapters[0]), Err(ReviewCopyError::NotImported)));
    mine.finish();
}

// ───────────────────────── scan gate (AD-52 rule 5) ─────────────────────────

fn squash(text: &str) -> String {
    text.replace("\\\n", "").split_whitespace().collect::<Vec<_>>().join(" ")
}

fn touches_chapter_membership(body: &str) -> bool {
    let squashed = squash(body);
    let deletes_chapter = squashed.match_indices("DELETE FROM chapter").any(|(at, hit)| {
        !squashed[at + hit.len()..].chars().next().is_some_and(|c| c.is_ascii_alphanumeric() || c == '_')
    });
    deletes_chapter || squashed.contains("UPDATE segment SET chapter_id")
}

/// `(function name, body)` for every `fn` with a body in `text`, comments stripped.
fn function_bodies(text: &str) -> Vec<(String, String)> {
    let code: String = code_lines(text).map(|(_, line)| line).collect::<Vec<_>>().join("\n");
    let chars: Vec<char> = code.chars().collect();
    let mut out = Vec::new();
    let mut i = 0;
    while i + 3 < chars.len() {
        let at_fn = chars[i] == 'f'
            && chars[i + 1] == 'n'
            && chars[i + 2] == ' '
            && (i == 0 || !(chars[i - 1].is_ascii_alphanumeric() || chars[i - 1] == '_'));
        if !at_fn {
            i += 1;
            continue;
        }
        let name: String = chars[i + 3..].iter().take_while(|c| c.is_ascii_alphanumeric() || **c == '_').collect();
        let open = chars[i..].iter().position(|&c| c == '{' || c == ';').map(|p| p + i);
        if let Some(open) = open.filter(|&p| chars[p] == '{' && !name.is_empty()) {
            if let Some(close) = matching_close_brace(&chars, open + 1) {
                out.push((name, chars[open + 1..close].iter().collect()));
            }
        }
        i += 3;
    }
    out
}

fn membership_functions(files: &[(String, String)]) -> (Vec<String>, Vec<String>) {
    let (mut touching, mut unguarded) = (Vec::new(), Vec::new());
    for (rel, text) in files {
        for (name, body) in function_bodies(&without_test_modules(text)) {
            if touches_chapter_membership(&body) {
                touching.push(format!("{rel}::{name}"));
                if !body.contains("review_chapter") {
                    unguarded.push(format!("{rel}::{name}"));
                }
            }
        }
    }
    (touching, unguarded)
}

#[test]
fn every_function_that_deletes_a_chapter_or_moves_its_segments_names_review_chapter() {
    let files = boundary_scan::rust_sources(&src_root());
    let (touching, unguarded) = membership_functions(&files);
    assert!(
        touching.iter().any(|f| f.ends_with("::merge_chapter_into_previous"))
            && touching.iter().any(|f| f.ends_with("::split_chapter_at_segment")),
        "cong quet khong thay hai duong gop/tach that: {touching:?}"
    );
    assert!(unguarded.is_empty(), "ham doi ranh gioi Chuong ma khong chap `review_chapter` (AD-52 muc 5): {unguarded:?}");
}

#[test]
fn the_membership_scan_goes_red_on_a_seeded_function_and_stays_green_on_a_guarded_one() {
    let rogue = "fn rogue_merge(tx: &Tx) {\n    tx.execute(\"DELETE FROM chapter WHERE id = ?1\", [1]);\n}\n";
    let moves = "pub fn rogue_move(tx: &Tx) {\n    tx.execute(\"UPDATE segment SET chapter_id = 2 \\\n WHERE chapter_id = 1\", []);\n}\n";
    let guarded = "fn fine(tx: &Tx) {\n    tx.execute(\"DELETE FROM chapter WHERE id = 1\", []);\n    tx.execute(\"UPDATE review_chapter SET stale_at = 1\", []);\n}\n";
    let commented = "fn quiet(tx: &Tx) {\n    // DELETE FROM chapter WHERE id = 1\n}\n";
    let position_only = "fn pos(tx: &Tx) {\n    tx.execute(\"DELETE FROM chapter_position WHERE chapter_id = 1\", []);\n}\n";
    let files = vec![
        ("seed/rogue.rs".to_owned(), rogue.to_owned()),
        ("seed/moves.rs".to_owned(), moves.to_owned()),
        ("seed/guarded.rs".to_owned(), guarded.to_owned()),
        ("seed/commented.rs".to_owned(), commented.to_owned()),
        ("seed/position.rs".to_owned(), position_only.to_owned()),
    ];

    let (touching, unguarded) = membership_functions(&files);

    assert_eq!(unguarded, vec!["seed/rogue.rs::rogue_merge".to_owned(), "seed/moves.rs::rogue_move".to_owned()]);
    assert_eq!(touching.len(), 3);
}
