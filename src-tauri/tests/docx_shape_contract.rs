//! A lone `.docx` is read once, its shape detected from content, and either shape rebuilt
//! from the stash without reading the path again.

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::sync::atomic::{AtomicU64, Ordering};

use auratranslate_lib::commands::project::{
    PendingImportSourceState, confirm_import_with_encoding, preview_docx_import_shape,
    switch_docx_import_shape,
};
use auratranslate_lib::core::segment::import::{
    DocxShape, docx_source_from_bytes, read_docx_source,
};
use auratranslate_lib::core::segment::pipeline::PipelineShape;
use docx_rs::{Docx, Paragraph, Run, Table, TableCell, TableRow};

fn para(text: &str) -> Paragraph {
    Paragraph::new().add_run(Run::new().add_text(text))
}

fn table(rows: &[&[&str]]) -> Table {
    Table::new(
        rows.iter()
            .map(|r| {
                TableRow::new(
                    r.iter()
                        .map(|c| TableCell::new().add_paragraph(para(c)))
                        .collect(),
                )
            })
            .collect(),
    )
}

fn pack(docx: Docx) -> Vec<u8> {
    let mut buf = std::io::Cursor::new(Vec::new());
    docx.build().pack(&mut buf).expect("pack docx fixture");
    buf.into_inner()
}

fn plain() -> Vec<u8> {
    pack(Docx::new().add_paragraph(para("Doan mot. Doan hai.")))
}

fn two_column_table() -> Vec<u8> {
    pack(Docx::new().add_table(table(&[
        &["Cau mot nguon.", "Cau mot dich."],
        &["Cau hai nguon.", "Cau hai dich."],
    ])))
}

fn detect(bytes: &[u8]) -> DocxShape {
    docx_source_from_bytes(bytes, "t.docx".to_owned())
        .expect("read docx")
        .detected
}

fn temp_dir(tag: &str) -> PathBuf {
    static N: AtomicU64 = AtomicU64::new(0);
    let dir = std::env::temp_dir().join(format!(
        "aura-docx-shape-{tag}-{}-{}",
        std::process::id(),
        N.fetch_add(1, Ordering::Relaxed)
    ));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).expect("create temp dir");
    dir
}

fn pending_state() -> PendingImportSourceState {
    Mutex::new(None)
}

fn pending_kind(state: &PendingImportSourceState) -> (&'static str, bool) {
    let guard = state.lock().expect("lock");
    let pending = guard.as_ref().expect("a pending source");
    let kind = match &pending.shape {
        PipelineShape::Blob(_) => "blob",
        PipelineShape::BilingualTables { .. } => "tables",
        _ => "other",
    };
    (kind, pending.docx_sidecar.is_some())
}

fn open(
    state: &PendingImportSourceState,
    path: &Path,
    source_lang: &str,
) -> Result<
    auratranslate_lib::commands::project::DocxImportPreviewWire,
    auratranslate_lib::core::i18n::IpcError,
> {
    let source = read_docx_source(path).expect("read docx");
    let shape = source.detected;
    preview_docx_import_shape(state, source, shape, source_lang, &[], None, None)
}

fn switch(
    state: &PendingImportSourceState,
    shape: DocxShape,
    source_lang: &str,
) -> Result<
    auratranslate_lib::commands::project::DocxImportPreviewWire,
    auratranslate_lib::core::i18n::IpcError,
> {
    switch_docx_import_shape(state, shape, source_lang, &|_| Vec::new(), None)
}

#[test]
fn a_document_of_paragraphs_only_is_text() {
    assert_eq!(detect(&plain()), DocxShape::Text);
}

#[test]
fn a_two_column_table_holding_the_text_is_bilingual() {
    assert_eq!(detect(&two_column_table()), DocxShape::Bilingual);
}

#[test]
fn long_prose_with_a_small_two_column_table_is_text() {
    let prose = "chu ".repeat(200);
    let bytes = pack(
        Docx::new()
            .add_paragraph(para(&prose))
            .add_table(table(&[&["a nguon", "a dich"]])),
    );
    assert_eq!(detect(&bytes), DocxShape::Text);
}

#[test]
fn a_one_column_table_never_makes_a_document_bilingual() {
    let bytes = pack(Docx::new().add_table(table(&[&["mot cot"], &["van mot cot"]])));
    assert_eq!(detect(&bytes), DocxShape::Text);
}

#[test]
fn a_table_whose_only_two_cell_row_is_blank_opens_as_text() {
    let bytes = pack(
        Docx::new()
            .add_paragraph(para("Doan van ngan."))
            .add_table(table(&[&["", ""], &["mot cot"]])),
    );
    let source = docx_source_from_bytes(&bytes, "t.docx".to_owned()).expect("read docx");
    assert_eq!(source.detected, DocxShape::Text);
}

#[test]
fn the_table_share_threshold_is_inclusive_at_half_of_the_characters() {
    let build = |paragraph: &str| {
        pack(
            Docx::new()
                .add_paragraph(para(paragraph))
                .add_table(table(&[&["aaaaa", "bbbbb"]])),
        )
    };
    assert_eq!(detect(&build("cccccccccc")), DocxShape::Bilingual);
    assert_eq!(detect(&build("ccccccccccc")), DocxShape::Text);
}

#[test]
fn switching_and_back_rebuilds_each_shape_from_the_stash_after_the_file_is_deleted() {
    let dir = temp_dir("switch-back");
    let path = dir.join("song-ngu.docx");
    fs::write(&path, two_column_table()).expect("write docx");
    let state = pending_state();

    let opened = open(&state, &path, "en").expect("open");
    assert_eq!(opened.shape, "bilingual");
    assert!(opened.bilingual.is_some() && opened.text.is_none());
    assert_eq!(pending_kind(&state), ("tables", false));

    fs::remove_file(&path).expect("delete the file");

    let as_text = switch(&state, DocxShape::Text, "en").expect("switch to text");
    assert_eq!(as_text.shape, "text");
    assert!(
        as_text
            .text
            .as_ref()
            .and_then(|t| t.encoding_preview.as_ref())
            .is_some()
    );
    assert_eq!(pending_kind(&state), ("blob", true));

    let back = switch(&state, DocxShape::Bilingual, "en").expect("switch back");
    assert_eq!(back.shape, "bilingual");
    assert_eq!(pending_kind(&state), ("tables", false));
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn a_refused_switch_leaves_the_pending_source_and_preview_unchanged() {
    let dir = temp_dir("refused");
    let path = dir.join("van-ban.docx");
    fs::write(&path, plain()).expect("write docx");
    let state = pending_state();
    assert_eq!(open(&state, &path, "en").expect("open").shape, "text");
    assert_eq!(pending_kind(&state), ("blob", true));

    let refused = switch(&state, DocxShape::Bilingual, "en").expect_err("no two-column table");
    assert_eq!(refused.code(), "import.bilingual_no_table");
    assert_eq!(pending_kind(&state), ("blob", true));
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn a_bilingual_docx_with_an_auto_language_opens_no_preview_and_keeps_nothing() {
    let dir = temp_dir("auto-lang");
    let path = dir.join("song-ngu.docx");
    fs::write(&path, two_column_table()).expect("write docx");
    let state = pending_state();
    let refused = open(&state, &path, "auto").expect_err("auto needs a chosen language");
    assert_eq!(refused.code(), "import.bilingual_needs_source_lang");
    assert!(state.lock().expect("lock").is_none());
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn a_bilingual_docx_with_two_qualifying_tables_opens_asking_for_the_table() {
    let dir = temp_dir("two-tables");
    let path = dir.join("hai-bang.docx");
    let bytes = pack(
        Docx::new()
            .add_table(table(&[&["Mot nguon.", "Mot dich."]]))
            .add_paragraph(para("x"))
            .add_table(table(&[&["Hai nguon.", "Hai dich."]])),
    );
    fs::write(&path, bytes).expect("write docx");
    let state = pending_state();
    let opened = open(&state, &path, "en").expect("open");
    assert_eq!(opened.shape, "bilingual");
    let preview = opened.bilingual.expect("bilingual half");
    assert!(preview.table_choice_required);
    assert_eq!(preview.source_tables.len(), 2);
    assert_eq!(pending_kind(&state), ("tables", false));
    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn a_switch_without_a_pending_docx_is_refused() {
    let state = pending_state();
    assert_eq!(
        switch(&state, DocxShape::Text, "en")
            .expect_err("nothing pending")
            .code(),
        "import.no_pending_source"
    );
}

#[test]
fn confirming_after_a_switch_to_text_creates_a_text_docx_work() {
    let dir = temp_dir("confirm");
    let path = dir.join("song-ngu.docx");
    fs::write(&path, two_column_table()).expect("write docx");
    let state = pending_state();
    open(&state, &path, "en").expect("open");
    switch(&state, DocxShape::Text, "en").expect("switch to text");

    let root = dir.join("library");
    fs::create_dir_all(&root).expect("library root");
    let work = confirm_import_with_encoding(
        &root,
        &state,
        "Tu Docx",
        "en",
        "",
        "UTF-8",
        Vec::new(),
        None,
        Vec::new(),
        Vec::new(),
        &Mutex::new(Vec::new()),
    )
    .expect("confirm the text import");
    let count: i64 = work
        .store
        .read(|conn| conn.query_row("SELECT COUNT(*) FROM segment", [], |r| r.get(0)))
        .expect("count segments");
    assert!(count > 0);
    let _ = fs::remove_dir_all(&dir);
}
