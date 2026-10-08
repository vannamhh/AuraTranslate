//! Hợp đồng của `.md` và text thuần (FR88): đoạn theo cờ kết đoạn đích (AD-46), ảnh theo chế độ
//! của 8.5, alt-text và chú thích là bản dịch của hai segment vai (AD-42).

use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};

use auratranslate_lib::commands::export::{export_text, ExportedFile};
use auratranslate_lib::commands::project::{OpenWork, create_work_from_text};
use auratranslate_lib::core::export::{ExportScope, ImageMode, TextFormat};
use auratranslate_lib::core::store::Transaction;

static NEXT_DIR: AtomicU64 = AtomicU64::new(0);

fn temp_dir(tag: &str) -> PathBuf {
    let n = NEXT_DIR.fetch_add(1, Ordering::Relaxed);
    let dir = std::env::temp_dir().join(format!("auratranslate-text-{}-{}-{}", std::process::id(), tag, n));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap_or_else(|e| panic!("tao {}: {e}", dir.display()));
    dir
}

struct Seg {
    source: &'static str,
    target: &'static str,
    target_end: bool,
    omitted: bool,
    role: Option<&'static str>,
}

fn seg(source: &'static str, target: &'static str) -> Seg {
    Seg { source, target, target_end: true, omitted: false, role: None }
}

fn flagged(source: &'static str, target: &'static str, target_end: bool) -> Seg {
    Seg { target_end, ..seg(source, target) }
}

fn role(kind: &'static str, target: &'static str) -> Seg {
    Seg { role: Some(kind), ..seg("goc", target) }
}

struct Fixture {
    open: OpenWork,
    first: i64,
    root: PathBuf,
    out: PathBuf,
}

fn fixture(tag: &str, title: Option<&str>, segs: Vec<Seg>) -> Fixture {
    let root = temp_dir(tag);
    let out = temp_dir(&format!("{tag}-out"));
    let open = create_work_from_text(&root, "Tac Pham", "zh", "", "Chuong mot.".to_owned()).expect("tao tac pham");
    open.store.write(|tx: &Transaction<'_>| tx.execute("DELETE FROM segment", [])).expect("xoa segment");
    let first = open.chapter_id;
    let title = title.map(str::to_owned);
    open.store
        .write(move |tx: &Transaction<'_>| tx.execute("UPDATE chapter SET title = ?1 WHERE id = ?2", (title, first)))
        .expect("dat tieu de");
    open.store
        .write(move |tx: &Transaction<'_>| {
            for (i, s) in segs.iter().enumerate() {
                tx.execute(
                    "INSERT INTO segment (chapter_id, ord, source_text, is_paragraph_end, target_text, \
                     is_target_paragraph_end, created_at, updated_at, status, is_omitted, role) \
                     VALUES (?1, ?2, ?3, 1, ?4, ?5, 't', 't', 'draft', ?6, ?7)",
                    rusqlite::params![
                        first,
                        i64::try_from(i).unwrap_or(0) + 1,
                        s.source,
                        s.target,
                        i64::from(s.target_end),
                        i64::from(s.omitted),
                        s.role
                    ],
                )?;
            }
            Ok(())
        })
        .expect("chen segment that bai");
    Fixture { open, first, root, out }
}

impl Fixture {
    fn asset(&self, file_name: &str, source_url: Option<&str>, anchor: i64) {
        let (file_name, source_url, chapter_id) = (file_name.to_owned(), source_url.map(str::to_owned), self.first);
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
    }

    fn put_file(&self, file_name: &str) {
        let dir = self.open.dir.join("assets");
        fs::create_dir_all(&dir).expect("tao assets");
        fs::write(dir.join(file_name), b"jpg").expect("ghi anh");
    }

    fn run(&self, mode: ImageMode, format: TextFormat) -> (ExportedFile, String) {
        let file = export_text(Some(&self.open), &ExportScope::Work, mode, format, &self.out).expect("xuat");
        let text = fs::read_to_string(&file.path).expect("doc tep");
        (file, text)
    }

    fn finish(self) {
        let dir = self.open.dir.clone();
        drop(self.open);
        let _ = fs::remove_dir_all(&dir);
        let _ = fs::remove_dir_all(&self.root);
        let _ = fs::remove_dir_all(&self.out);
    }
}

const MD: TextFormat = TextFormat::Markdown;
const TXT: TextFormat = TextFormat::Plain;

#[test]
fn paragraphs_follow_the_target_flag_and_newlines_split_them() {
    let f = fixture("para", Some("Mo dau"), vec![flagged("a", "t1", false), seg("b", "t2\nt3"), seg("c", "t4")]);
    let (file, md) = f.run(ImageMode::File, MD);
    assert!(file.path.ends_with("Tac Pham.md"), "{}", file.path);
    assert_eq!(md, "## Mo dau\n\nt1 t2\n\nt3\n\nt4\n");
    let (file, txt) = f.run(ImageMode::File, TXT);
    assert!(file.path.ends_with("Tac Pham.txt"), "{}", file.path);
    assert_eq!(txt, "Mo dau\n\nt1 t2\n\nt3\n\nt4\n");
    f.finish();
}

#[test]
fn an_omitted_segment_hands_its_end_flag_to_the_survivor_before_it() {
    let omitted = Seg { omitted: true, ..seg("b", "tB") };
    let f = fixture("omit", None, vec![flagged("a", "tA", false), omitted, seg("c", "tC")]);
    let (_, md) = f.run(ImageMode::File, MD);
    assert_eq!(md, "## Chương 1\n\ntA\n\ntC\n");
    f.finish();
}

#[test]
fn alt_and_caption_come_from_the_role_segments_and_never_leak_as_prose() {
    let f = fixture(
        "roles",
        Some("T"),
        vec![seg("p", "Hello"), role("alt", "Anh [dep]"), role("caption", "Chu *thich*"), seg("q", "Sau")],
    );
    f.asset("a.jpg", Some("https://x.test/a.jpg"), 1);
    let (file, md) = f.run(ImageMode::Link, MD);
    assert_eq!(md, "## T\n\nHello\n\n![Anh \\[dep\\]](https://x.test/a.jpg)\n*Chu \\*thich\\**\n\nSau\n");
    assert_eq!((file.image_count, file.images_skipped_missing_link, file.images_dir), (1, 0, None));
    let (_, txt) = f.run(ImageMode::Link, TXT);
    assert_eq!(txt, "T\n\nHello\n\n[Ảnh: Anh [dep]] https://x.test/a.jpg\nChu *thich*\n\nSau\n");
    assert!(!md.contains("goc") && !txt.contains("goc"));
    f.finish();
}

#[test]
fn an_untranslated_alt_is_empty_and_an_untranslated_caption_has_no_line() {
    let f = fixture("blank-roles", Some("T"), vec![seg("p", "Hello"), role("alt", " "), role("caption", "")]);
    f.asset("a.jpg", Some("https://x.test/a.jpg"), 1);
    let (_, md) = f.run(ImageMode::Link, MD);
    assert_eq!(md, "## T\n\nHello\n\n![](https://x.test/a.jpg)\n");
    let (_, txt) = f.run(ImageMode::Link, TXT);
    assert_eq!(txt, "T\n\nHello\n\n[Ảnh: ] https://x.test/a.jpg\n");
    f.finish();
}

#[test]
fn markdown_escapes_what_commonmark_would_read_and_plain_text_escapes_nothing() {
    let lines = ["# a", "> b", "- c", "+ d", "1. e", "2) f", "g*h_i`j<k&l[m]n\\o", "~~~"];
    let f = fixture("escape", Some("Tieu #"), lines.iter().map(|l| seg("s", l)).collect());
    let (_, md) = f.run(ImageMode::File, MD);
    assert_eq!(
        md,
        "## Tieu \\#\n\n\\# a\n\n\\> b\n\n\\- c\n\n\\+ d\n\n1\\. e\n\n2\\) f\n\ng\\*h\\_i\\`j\\<k\\&l\\[m\\]n\\\\o\n\n\\~~~\n"
    );
    let (_, txt) = f.run(ImageMode::File, TXT);
    assert_eq!(txt, format!("Tieu #\n\n{}\n", lines.join("\n\n")));
    f.finish();
}

#[test]
fn link_mode_skips_and_counts_images_without_a_link() {
    let f = fixture("skip", Some("T"), vec![seg("p", "Hello")]);
    f.asset("a.jpg", Some("https://x.test/a b.jpg"), 1);
    f.asset("b.jpg", None, 1);
    let (file, md) = f.run(ImageMode::Link, MD);
    assert_eq!((file.image_count, file.images_skipped_missing_link), (1, 1));
    assert_eq!(md, "## T\n\nHello\n\n![](<https://x.test/a b.jpg>)\n");
    f.finish();
}

#[test]
fn file_mode_copies_images_next_to_the_file_and_references_them_relatively() {
    let f = fixture("file", Some("T"), vec![seg("p", "Hello")]);
    f.asset("a.jpg", None, 0);
    f.put_file("a.jpg");
    let (file, md) = f.run(ImageMode::File, MD);
    let dir = file.images_dir.clone().expect("thu muc anh");
    assert!(dir.ends_with("Tac Pham-anh"), "{dir}");
    let copied: Vec<String> =
        fs::read_dir(&dir).expect("doc thu muc").map(|e| e.expect("muc").file_name().to_string_lossy().into_owned()).collect();
    assert_eq!(copied.len(), 1);
    assert_eq!(md, format!("## T\n\n![](<Tac Pham-anh/{}>)\n\nHello\n", copied[0]));
    f.finish();
}

#[test]
fn a_missing_image_file_is_the_named_error_and_nothing_is_overwritten() {
    let f = fixture("gone", Some("T"), vec![seg("p", "Hello")]);
    f.asset("gone.jpg", None, 0);
    let err = export_text(Some(&f.open), &ExportScope::Work, ImageMode::File, MD, &f.out).expect_err("phai loi");
    assert_eq!(err.code(), "export.image_file_missing");
    let g = fixture("twice", Some("T"), vec![seg("p", "Hello")]);
    let (first, _) = g.run(ImageMode::File, MD);
    let (second, _) = g.run(ImageMode::File, MD);
    assert!(first.path.ends_with("Tac Pham.md") && second.path.ends_with("Tac Pham (2).md"), "{}", second.path);
    f.finish();
    g.finish();
}

#[test]
fn a_chapter_with_nothing_translated_and_no_image_is_absent() {
    let f = fixture("empty", Some("T"), vec![seg("p", " ")]);
    let (_, md) = f.run(ImageMode::File, MD);
    assert_eq!(md, "");
    f.finish();
}

#[test]
fn an_empty_scope_an_unknown_chapter_and_no_open_work_are_the_named_errors() {
    let f = fixture("scope", Some("T"), vec![seg("p", "Hello")]);
    let code = |open: Option<&OpenWork>, scope: ExportScope| {
        export_text(open, &scope, ImageMode::File, MD, &f.out).expect_err("phai loi").code().to_owned()
    };
    assert_eq!(code(Some(&f.open), ExportScope::Chapters { chapter_ids: vec![] }), "export.scope_empty");
    assert_eq!(code(Some(&f.open), ExportScope::Chapters { chapter_ids: vec![9_999] }), "segment.chapter_not_found");
    assert_eq!(code(None, ExportScope::Work), "work.none_open");
    assert_eq!(fs::read_dir(&f.out).expect("doc thu muc xuat").count(), 0);
    f.finish();
}
