//! Phạm vi xuất (FR89): đếm Chương và câu của phạm vi, và hộp thoại chọn thư mục đích (AD-48).
//! Phép đếm sống ở [`crate::core::export`]; ở đây chỉ đổi lỗi sang `IpcError`.

use std::collections::BTreeMap;

use crate::commands::chapter::{chapter_not_found, no_work_open};
use crate::commands::project::OpenWork;
use crate::core::export::{
    AlignmentError, AlignmentItem, ChapterAlignment, ConfirmError, ReviewCopyError, harvest_work, ReviewFileKind, ReviewerDocx, ReviewerImportPlan, confirm_import, plan_import, read_docx_copy,
    read_markdown_copy,
    Attribution, ExportScope, DocxWriteError, ExportImage, ImageFilesError, ImageMode, ImageReference, MissingLinkImage, ScopeError, count_scope,
    load_chapter_blocks, load_chapter_tables, load_chapter_text, render_text, TextFormat, resolve_chapter_ids, safe_stem, scan_images, write_file_with_images, write_new_file,
    write_one_block_docx, write_two_column_docx,
};
use crate::core::attribution::resolve_translator_name;
use crate::core::glossary::{ReviewHarvestDetail, ReviewHarvestProposal, enqueue_review_harvest};
use crate::core::i18n::{IpcError, MessageKey};
use crate::core::scope::ScopeResolver;
use crate::core::store::{Store, StoreError, StoreKind};

/// Tóm tắt phạm vi xuất: phép đếm của FR89 cùng kết quả quét ảnh (AD-43).
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct ExportScopeSummary {
    pub chapter_count: i64,
    pub segment_count: i64,
    pub unconfirmed_count: i64,
    pub unconfirmed_translated_count: i64,
    pub untranslated_count: i64,
    pub image_count: i64,
    pub missing_link_images: Vec<MissingLinkImage>,
}

fn scope_error(err: ScopeError) -> IpcError {
    match err {
        ScopeError::Empty => {
            IpcError::new("export.scope_empty", MessageKey::ExportScopeEmpty, BTreeMap::new(), false)
        }
        ScopeError::ChapterNotFound(id) => chapter_not_found(id),
        ScopeError::Read(e) => IpcError::from(e),
    }
}

/// Đếm Chương, câu, câu chưa xác nhận và ảnh của phạm vi xuất trong Tác phẩm đang mở (FR89), kèm
/// danh sách ảnh thiếu `source_url`.
///
/// # Lỗi
/// - chưa mở Tác phẩm ⇒ `work.none_open`;
/// - phạm vi rỗng ⇒ `export.scope_empty`;
/// - Chương không tồn tại ⇒ `segment.chapter_not_found`.
pub fn export_scope_summary(
    open: Option<&OpenWork>,
    scope: &ExportScope,
) -> Result<ExportScopeSummary, IpcError> {
    let open = open.ok_or_else(no_work_open)?;
    let counts = count_scope(&open.store, scope).map_err(scope_error)?;
    let chapter_ids = resolve_chapter_ids(&open.store, scope).map_err(scope_error)?;
    let scan = scan_images(&open.store, &chapter_ids).map_err(IpcError::from)?;
    Ok(ExportScopeSummary {
        chapter_count: counts.chapter_count,
        segment_count: counts.segment_count,
        unconfirmed_count: counts.unconfirmed_count,
        unconfirmed_translated_count: counts.unconfirmed_translated_count,
        untranslated_count: counts.untranslated_count,
        image_count: scan.image_count,
        missing_link_images: scan.missing_link_images,
    })
}

pub fn attribution_of(attribution: bool, global: Option<&Store>) -> Result<Option<Attribution>, IpcError> {
    if !attribution {
        return Ok(None);
    }
    let global = global.ok_or_else(crate::commands::attribution::store_is_missing)?;
    let translator = resolve_translator_name(&ScopeResolver::global_only(), global).map_err(IpcError::from)?;
    Ok(Some(Attribution { translator }))
}

fn export_folder_invalid() -> IpcError {
    IpcError::new("export.folder_invalid", MessageKey::ExportFolderInvalid, BTreeMap::new(), false)
}

/// Đổi kết quả hộp thoại chọn thư mục thành chuỗi đường dẫn. Huỷ hộp thoại là `Ok(None)`.
pub fn export_folder_from_dialog(picked: Option<&std::path::Path>) -> Result<Option<String>, IpcError> {
    match picked {
        None => Ok(None),
        Some(path) => path.to_str().map(|s| Some(s.to_owned())).ok_or_else(export_folder_invalid),
    }
}

/// Tệp vừa ghi. `images_dir` chỉ có khi chế độ file ghi ít nhất một ảnh.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct ExportedFile {
    pub path: String,
    pub chapter_count: i64,
    pub segment_count: i64,
    pub image_count: i64,
    pub images_skipped_missing_link: i64,
    pub images_dir: Option<String>,
}

fn image_file_missing(chapter_ord: i64, file_name: &str) -> IpcError {
    IpcError::new(
        "export.image_file_missing",
        MessageKey::ExportImageFileMissing,
        BTreeMap::from([("chapter_ord".to_owned(), chapter_ord.to_string()), ("file_name".to_owned(), file_name.to_owned())]),
        false,
    )
}

fn export_write_failed(detail: &str) -> IpcError {
    eprintln!("xuat tep that bai: {detail}");
    IpcError::new("export.write_failed", MessageKey::ExportWriteFailed, BTreeMap::new(), true)
}

/// Ghi `.docx` bảng hai cột (FR87) của phạm vi vào `folder`, không ghi đè tệp có sẵn. Ảnh có
/// hàng ở vị trí neo (FR130): `Link` ghi `source_url`, `File` chép ảnh ra `<tên tệp>-anh/`.
///
/// # Lỗi
/// Như [`export_scope_summary`], thêm `export.folder_invalid` khi `folder` không phải thư mục,
/// `export.image_file_missing` khi tệp ảnh không còn trong `assets/` (chế độ file) và
/// `export.write_failed` khi ghi thất bại.
pub fn export_docx_two_column(
    open: Option<&OpenWork>,
    scope: &ExportScope,
    image_mode: ImageMode,
    attribution: Option<&Attribution>,
    folder: &std::path::Path,
) -> Result<ExportedFile, IpcError> {
    let open = open.ok_or_else(no_work_open)?;
    let counts = count_scope(&open.store, scope).map_err(scope_error)?;
    if !folder.is_dir() {
        return Err(export_folder_invalid());
    }
    let chapter_ids = resolve_chapter_ids(&open.store, scope).map_err(scope_error)?;
    let loaded = load_chapter_tables(&open.store, &chapter_ids, image_mode, attribution).map_err(IpcError::from)?;
    let images = loaded.images();
    let stem = format!("{}-hai-cot", safe_stem(&open.meta.name, "export"));
    let (path, images_dir) = write_export_file(open, folder, &stem, "docx", image_mode, &images, |reference| {
        write_two_column_docx(&loaded.tables, reference)
    })?;
    exported_file(&counts, &loaded_summary(&images, loaded.images_skipped_missing_link), path, images_dir)
}

struct ImageSummary {
    image_count: i64,
    images_skipped_missing_link: i64,
}

fn loaded_summary(images: &[&ExportImage], images_skipped_missing_link: i64) -> ImageSummary {
    ImageSummary { image_count: i64::try_from(images.len()).unwrap_or(i64::MAX), images_skipped_missing_link }
}

fn write_export_file(
    open: &OpenWork,
    folder: &std::path::Path,
    stem: &str,
    extension: &str,
    image_mode: ImageMode,
    images: &[&ExportImage],
    build: impl Fn(ImageReference<'_>) -> Result<Vec<u8>, DocxWriteError>,
) -> Result<(std::path::PathBuf, Option<std::path::PathBuf>), IpcError> {
    if image_mode == ImageMode::File && !images.is_empty() {
        let written = write_file_with_images(folder, stem, extension, &open.dir.join("assets"), images, |final_stem| {
            let dir_name = format!("{final_stem}{}", crate::core::export::IMAGE_DIR_SUFFIX);
            build(ImageReference::Dir(&dir_name)).map_err(|e| e.0)
        })
        .map_err(|err| match err {
            ImageFilesError::SourceMissing { chapter_ord, file_name } => image_file_missing(chapter_ord, &file_name),
            ImageFilesError::Write(detail) => export_write_failed(&detail),
        })?;
        Ok((written.file_path, Some(written.images_dir)))
    } else {
        let bytes = build(ImageReference::Link).map_err(|e| export_write_failed(&e.0))?;
        let path = write_new_file(folder, stem, extension, &bytes).map_err(|e| export_write_failed(&e.to_string()))?;
        Ok((path, None))
    }
}

fn exported_file(
    counts: &crate::core::export::ScopeCounts,
    images: &ImageSummary,
    path: std::path::PathBuf,
    images_dir: Option<std::path::PathBuf>,
) -> Result<ExportedFile, IpcError> {
    let path = path.to_str().ok_or_else(export_folder_invalid)?.to_owned();
    let images_dir = images_dir.map(|dir| dir.to_str().map(str::to_owned).ok_or_else(export_folder_invalid)).transpose()?;
    Ok(ExportedFile {
        path,
        chapter_count: counts.chapter_count,
        segment_count: counts.segment_count,
        image_count: images.image_count,
        images_skipped_missing_link: images.images_skipped_missing_link,
        images_dir,
    })
}

/// Never overwrites an existing file.
pub fn export_docx_one_block(
    open: Option<&OpenWork>,
    scope: &ExportScope,
    image_mode: ImageMode,
    attribution: Option<&Attribution>,
    folder: &std::path::Path,
) -> Result<ExportedFile, IpcError> {
    let open = open.ok_or_else(no_work_open)?;
    let counts = count_scope(&open.store, scope).map_err(scope_error)?;
    if !folder.is_dir() {
        return Err(export_folder_invalid());
    }
    let chapter_ids = resolve_chapter_ids(&open.store, scope).map_err(scope_error)?;
    let loaded =
        load_chapter_blocks(&open.store, &chapter_ids, image_mode, &open.meta.source_lang, attribution).map_err(IpcError::from)?;
    let images = loaded.images();
    let stem = format!("{}-mot-khoi", safe_stem(&open.meta.name, "export"));
    let (path, images_dir) = write_export_file(open, folder, &stem, "docx", image_mode, &images, |reference| {
        write_one_block_docx(&loaded.blocks, reference)
    })?;
    exported_file(&counts, &loaded_summary(&images, loaded.images_skipped_missing_link), path, images_dir)
}

/// Writes `<Work name>.md` or `<Work name>.txt`; never overwrites an existing file.
pub fn export_text(
    open: Option<&OpenWork>,
    scope: &ExportScope,
    image_mode: ImageMode,
    format: TextFormat,
    attribution: Option<&Attribution>,
    folder: &std::path::Path,
) -> Result<ExportedFile, IpcError> {
    let open = open.ok_or_else(no_work_open)?;
    let counts = count_scope(&open.store, scope).map_err(scope_error)?;
    if !folder.is_dir() {
        return Err(export_folder_invalid());
    }
    let chapter_ids = resolve_chapter_ids(&open.store, scope).map_err(scope_error)?;
    let loaded = load_chapter_text(&open.store, &chapter_ids, image_mode, attribution).map_err(IpcError::from)?;
    let images = loaded.images();
    let stem = safe_stem(&open.meta.name, "export");
    let extension = match format {
        TextFormat::Markdown => "md",
        TextFormat::Plain => "txt",
    };
    let (path, images_dir) = write_export_file(open, folder, &stem, extension, image_mode, &images, |reference| {
        Ok(render_text(&loaded, format, reference).into_bytes())
    })?;
    exported_file(&counts, &loaded_summary(&images, loaded.images_skipped_missing_link), path, images_dir)
}

const MAX_REVIEWER_DOCX_BYTES: u64 = 64 * 1024 * 1024;
const MAX_REVIEWER_MD_BYTES: u64 = 16 * 1024 * 1024;

/// Plan of a reviewer file that was previewed and not yet confirmed (AD-48). The parsed file stays
/// in Rust; the webview only sees [`ReviewerImportPreviewWire`].
#[derive(Debug)]
pub struct PendingReviewerImport {
    plan: ReviewerImportPlan,
}

pub type PendingReviewerImportState = std::sync::Mutex<Option<PendingReviewerImport>>;

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct ReviewerImportReplacedWire {
    pub file_name: String,
    pub stale: bool,
    pub user_group_count: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct ReviewerImportChapterWire {
    pub chapter_id: i64,
    pub chapter_ord: i64,
    pub title: Option<String>,
    pub row_count: i64,
    pub replaces: Option<ReviewerImportReplacedWire>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct ReviewerImportSkippedWire {
    pub heading: String,
    pub row_count: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct ReviewerImportPreviewWire {
    pub file_name: String,
    pub file_kind: String,
    pub chapters: Vec<ReviewerImportChapterWire>,
    pub skipped: Vec<ReviewerImportSkippedWire>,
    pub image_rows_ignored: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct ReviewerImportSummaryWire {
    pub chapter_count: i64,
    pub row_count: i64,
    pub replaced_count: i64,
    /// Glossary candidates queued or refreshed by the harvest; `None` when the harvest failed.
    pub harvest_candidate_count: Option<i64>,
    /// Set exactly when `harvest_candidate_count` is `None`. The import itself was written.
    pub harvest_error: Option<IpcError>,
}

fn count_wire(n: usize) -> i64 {
    i64::try_from(n).unwrap_or(i64::MAX)
}

fn reviewer_unreadable(detail: &str) -> IpcError {
    eprintln!("nhap lai tep reviewer that bai: {detail}");
    IpcError::new("export.reviewer_import_unreadable", MessageKey::ExportReviewerUnreadable, BTreeMap::new(), false)
}

fn reviewer_preview_stale() -> IpcError {
    IpcError::new("export.reviewer_import_preview_stale", MessageKey::ExportReviewerPreviewStale, BTreeMap::new(), false)
}

fn harvest_failed(detail: &str) -> IpcError {
    eprintln!("thu hoach thuat ngu tu ban reviewer that bai: {detail}");
    IpcError::new("export.harvest_failed", MessageKey::ExportHarvestFailed, BTreeMap::new(), false)
}

fn reviewer_no_pending() -> IpcError {
    IpcError::new("export.reviewer_import_no_pending", MessageKey::ExportReviewerNoPending, BTreeMap::new(), false)
}

fn reviewer_wrong_work(work_name: &str) -> IpcError {
    IpcError::new(
        "export.reviewer_import_wrong_work",
        MessageKey::ExportReviewerWrongWork,
        BTreeMap::from([("work_name".to_owned(), work_name.to_owned())]),
        false,
    )
}

fn read_reviewer_file(path: &std::path::Path) -> Result<crate::core::export::ReviewerCopy, IpcError> {
    let file_name = path.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default();
    let extension = path.extension().map(|e| e.to_string_lossy().to_lowercase()).unwrap_or_default();
    let (kind, limit) = match extension.as_str() {
        "docx" => (ReviewFileKind::Docx, MAX_REVIEWER_DOCX_BYTES),
        "md" => (ReviewFileKind::Markdown, MAX_REVIEWER_MD_BYTES),
        _ => return Err(reviewer_unreadable("extension")),
    };
    let size = std::fs::metadata(path).map_err(|e| reviewer_unreadable(&e.to_string()))?.len();
    if size > limit {
        return Err(reviewer_unreadable("too large"));
    }
    let bytes = std::fs::read(path).map_err(|e| reviewer_unreadable(&e.to_string()))?;
    let copy = match kind {
        ReviewFileKind::Docx => {
            let parsed = crate::core::docx::read_docx(&bytes).map_err(|e| reviewer_unreadable(&format!("{e:?}")))?;
            let docx = ReviewerDocx::admit(parsed).map_err(IpcError::from)?;
            read_docx_copy(&docx, &file_name)
        }
        ReviewFileKind::Markdown => {
            let text = String::from_utf8(bytes).map_err(|_| reviewer_unreadable("not utf-8"))?;
            read_markdown_copy(&text, &file_name)
        }
    };
    copy.map_err(|_| reviewer_unreadable("shape"))
}

fn reviewer_preview_wire(plan: &ReviewerImportPlan) -> ReviewerImportPreviewWire {
    ReviewerImportPreviewWire {
        file_name: plan.copy.file_name.clone(),
        file_kind: plan.copy.kind.as_str().to_owned(),
        chapters: plan
            .chapters
            .iter()
            .map(|c| ReviewerImportChapterWire {
                chapter_id: c.chapter_id,
                chapter_ord: c.chapter_ord,
                title: c.title.clone(),
                row_count: count_wire(c.rows.len()),
                replaces: c.replaces.as_ref().map(|r| ReviewerImportReplacedWire {
                    file_name: r.file_name.clone(),
                    stale: r.stale,
                    user_group_count: count_wire(r.user_group_count),
                }),
            })
            .collect(),
        skipped: plan
            .skipped
            .iter()
            .map(|s| ReviewerImportSkippedWire { heading: s.heading.clone(), row_count: count_wire(s.row_count) })
            .collect(),
        image_rows_ignored: count_wire(plan.image_rows_ignored),
    }
}

/// Drops any previewed reviewer file; used when a new preview opens, on cancel, and when the Work closes.
pub fn reviewer_import_cancel(pending: &PendingReviewerImportState) {
    *pending.lock().unwrap_or_else(std::sync::PoisonError::into_inner) = None;
}

/// First beat of the reviewer re-import (FR90): parse `path`, match it to the open Work and keep the
/// plan in `pending`. Writes nothing to `project.db`.
///
/// # Errors
/// - no Work open => `work.none_open`;
/// - not a two-column `.docx` / `.md` => `export.reviewer_import_unreadable`;
/// - a one-block publishing copy => `export.publish_copy_not_reimportable`;
/// - no section matches a Chapter => `export.reviewer_import_wrong_work`.
pub fn reviewer_import_preview(
    open: Option<&OpenWork>,
    pending: &PendingReviewerImportState,
    path: &std::path::Path,
) -> Result<ReviewerImportPreviewWire, IpcError> {
    reviewer_import_cancel(pending);
    let open = open.ok_or_else(no_work_open)?;
    let copy = read_reviewer_file(path)?;
    let plan = open
        .store
        .read(|conn| match plan_import(conn, &copy) {
            Err(ReviewCopyError::Sql(e)) => Err(e),
            other => Ok(other),
        })
        .map_err(IpcError::from)?
        .map_err(|e| match e {
            ReviewCopyError::NoMatch => reviewer_wrong_work(&open.meta.name),
            ReviewCopyError::Store(e) => IpcError::from(e),
            other => reviewer_unreadable(&format!("{other:?}")),
        })?;
    let wire = reviewer_preview_wire(&plan);
    *pending.lock().unwrap_or_else(std::sync::PoisonError::into_inner) = Some(PendingReviewerImport { plan });
    Ok(wire)
}

/// Queues the Glossary candidates the live reviewer copies of the open Work give, after the import
/// is written. Returns how many candidates were inserted or refreshed.
fn harvest_reviewer_copies(open: &OpenWork, global: Option<&Store>) -> Result<i64, IpcError> {
    let global = global.ok_or_else(|| harvest_failed("global store missing"))?;
    let findings = harvest_work(&open.scope, global, &open.store, &open.meta.source_lang)
        .map_err(|e| harvest_failed(&format!("{e:?}")))?;
    let proposals: Vec<ReviewHarvestProposal> = findings
        .into_iter()
        .map(|f| ReviewHarvestProposal {
            source_term: f.source_term,
            detail: ReviewHarvestDetail {
                replaced_translation: f.replaced_translation,
                proposed_translation: f.proposed_translation,
                changed_count: f.changed_count,
                seen_count: f.seen_count,
            },
        })
        .collect();
    let queued = enqueue_review_harvest(&open.store, &proposals).map_err(|e| harvest_failed(&format!("{e:?}")))?;
    Ok(queued.inserted + queued.updated)
}

/// Second beat: write the previewed reviewer copy in one transaction (AD-52 rule 3). The match is
/// redone inside the transaction; a different set of Chapters writes nothing.
///
/// # Errors
/// - no Work open => `work.none_open`;
/// - nothing previewed => `export.reviewer_import_no_pending`;
/// - Chapters changed since the preview => `export.reviewer_import_preview_stale` (pending dropped);
/// - a write failure rolls back and keeps the preview so it can be retried.
///
/// A failed harvest after the write is not an error of this call: it comes back as
/// `harvest_error` (`export.harvest_failed`) and the import stays.
pub fn reviewer_import_confirm(
    open: Option<&OpenWork>,
    global: Option<&Store>,
    pending: &PendingReviewerImportState,
) -> Result<ReviewerImportSummaryWire, IpcError> {
    let open = open.ok_or_else(no_work_open)?;
    let mut guard = pending.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
    let Some(previewed) = guard.as_ref().map(|p| p.plan.clone()) else {
        return Err(reviewer_no_pending());
    };
    let outcome = open
        .store
        .write(move |tx| match confirm_import(tx, &previewed) {
            Ok(summary) => Ok(Some(summary)),
            Err(ConfirmError::Stale) => Ok(None),
            Err(ConfirmError::Sql(e)) => Err(e),
        })
        .map_err(IpcError::from)?;
    match outcome {
        Some(summary) => {
            *guard = None;
            let (harvest_candidate_count, harvest_error) = match harvest_reviewer_copies(open, global) {
                Ok(count) => (Some(count), None),
                Err(error) => (None, Some(error)),
            };
            Ok(ReviewerImportSummaryWire {
                chapter_count: count_wire(summary.chapter_count),
                row_count: count_wire(summary.row_count),
                replaced_count: count_wire(summary.replaced_count),
                harvest_candidate_count,
                harvest_error,
            })
        }
        None => {
            *guard = None;
            Err(reviewer_preview_stale())
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct AlignmentRowWire {
    pub id: i64,
    pub kind: String,
    pub source_text: Option<String>,
    pub target_text: String,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct AlignmentSegmentWire {
    pub id: i64,
    pub ord: i64,
    pub role: Option<String>,
    pub source_text: String,
    pub target_text: String,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct AlignmentGroupWire {
    pub id: i64,
    pub decided_by: String,
    pub row_ids: Vec<i64>,
    pub segment_ids: Vec<i64>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct ChapterAlignmentWire {
    pub chapter_id: i64,
    pub file_name: String,
    pub file_kind: String,
    pub rows: Vec<AlignmentRowWire>,
    pub segments: Vec<AlignmentSegmentWire>,
    pub groups: Vec<AlignmentGroupWire>,
    pub unmatched_row_ids: Vec<i64>,
    pub unmatched_segment_ids: Vec<i64>,
    pub is_resolved: bool,
}

fn alignment_wire(alignment: ChapterAlignment) -> ChapterAlignmentWire {
    ChapterAlignmentWire {
        chapter_id: alignment.chapter_id,
        file_name: alignment.file_name,
        file_kind: alignment.file_kind.as_str().to_owned(),
        rows: alignment
            .rows
            .into_iter()
            .map(|r| AlignmentRowWire {
                id: r.id,
                kind: r.kind.as_str().to_owned(),
                source_text: r.source_text,
                target_text: r.target_text,
            })
            .collect(),
        segments: alignment
            .segments
            .into_iter()
            .map(|s| AlignmentSegmentWire {
                id: s.id,
                ord: s.ord,
                role: s.role,
                source_text: s.source_text,
                target_text: s.target_text,
            })
            .collect(),
        groups: alignment
            .groups
            .into_iter()
            .map(|g| AlignmentGroupWire {
                id: g.id,
                decided_by: g.decided_by.as_str().to_owned(),
                row_ids: g.row_ids,
                segment_ids: g.segment_ids,
            })
            .collect(),
        unmatched_row_ids: alignment.unmatched_row_ids,
        unmatched_segment_ids: alignment.unmatched_segment_ids,
        is_resolved: alignment.is_resolved,
    }
}

fn alignment_error(error: AlignmentError) -> IpcError {
    match error {
        AlignmentError::Copy(ReviewCopyError::NotImported) => {
            IpcError::new("export.alignment_not_imported", MessageKey::ExportAlignmentNotImported, BTreeMap::new(), false)
        }
        AlignmentError::Copy(ReviewCopyError::Stale) => {
            IpcError::new("export.alignment_stale", MessageKey::ExportAlignmentStale, BTreeMap::new(), false)
        }
        AlignmentError::InvalidSelection => IpcError::new(
            "export.alignment_invalid_selection",
            MessageKey::ExportAlignmentInvalidSelection,
            BTreeMap::new(),
            false,
        ),
        AlignmentError::Copy(ReviewCopyError::Store(e)) | AlignmentError::Store(e) => IpcError::from(e),
        AlignmentError::Copy(ReviewCopyError::Sql(e)) | AlignmentError::Sql(e) => IpcError::from(StoreError::ReadFailed { store: StoreKind::Project, detail: format!("{e:?}") }),
        AlignmentError::Copy(other) => reviewer_unreadable(&format!("{other:?}")),
    }
}

fn read_alignment_wire(open: &OpenWork, chapter_id: i64) -> Result<ChapterAlignmentWire, IpcError> {
    crate::core::export::read_alignment(&open.store, chapter_id).map(alignment_wire).map_err(alignment_error)
}

/// Both sides of the reviewer copy of `chapter_id` and how they are grouped. A copy imported before
/// step 31 is aligned by the machine on this first call.
///
/// # Errors
/// - no Work open => `work.none_open`;
/// - no copy for the Chapter => `export.alignment_not_imported`;
/// - the Chapter was merged or split after the import => `export.alignment_stale`.
pub fn alignment_open(open: Option<&OpenWork>, chapter_id: i64) -> Result<ChapterAlignmentWire, IpcError> {
    let open = open.ok_or_else(no_work_open)?;
    read_alignment_wire(open, chapter_id)
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct ReviewDiffPairWire {
    pub group_id: i64,
    pub decided_by: String,
    pub segment_ids: Vec<i64>,
    pub row_ids: Vec<i64>,
    pub spans: Vec<crate::core::matching::DiffSpan>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct ReviewDiffWire {
    pub chapter_id: i64,
    pub pairs: Vec<ReviewDiffPairWire>,
}

/// Every alignment group of the reviewer copy with my text diffed to the reviewer's, in translation order.
///
/// # Errors
/// As [`alignment_open`].
pub fn review_diff(open: Option<&OpenWork>, chapter_id: i64) -> Result<ReviewDiffWire, IpcError> {
    let open = open.ok_or_else(no_work_open)?;
    let pairs = crate::core::export::review_diff(&open.store, chapter_id)
        .map_err(alignment_error)?
        .into_iter()
        .map(|d| ReviewDiffPairWire {
            group_id: d.group_id,
            decided_by: d.decided_by.as_str().to_owned(),
            segment_ids: d.segment_ids,
            row_ids: d.row_ids,
            spans: d.spans,
        })
        .collect();
    Ok(ReviewDiffWire { chapter_id, pairs })
}

/// Joins at least one segment and one reviewer row into a user group and returns the new state.
///
/// # Errors
/// As [`alignment_open`], and `export.alignment_invalid_selection` (nothing written) for an
/// empty side, a repeated or unknown id, or an id already in a group.
pub fn alignment_join(
    open: Option<&OpenWork>,
    chapter_id: i64,
    segment_ids: &[i64],
    row_ids: &[i64],
) -> Result<ChapterAlignmentWire, IpcError> {
    let open = open.ok_or_else(no_work_open)?;
    crate::core::export::join(&open.store, chapter_id, segment_ids, row_ids).map_err(alignment_error)?;
    read_alignment_wire(open, chapter_id)
}

/// Sets exactly one segment or one reviewer row aside as a one-sided user group.
///
/// # Errors
/// As [`alignment_join`]; giving both ids or neither is an invalid selection.
pub fn alignment_skip(
    open: Option<&OpenWork>,
    chapter_id: i64,
    segment_id: Option<i64>,
    row_id: Option<i64>,
) -> Result<ChapterAlignmentWire, IpcError> {
    let open = open.ok_or_else(no_work_open)?;
    let item = match (segment_id, row_id) {
        (Some(id), None) => AlignmentItem::Segment(id),
        (None, Some(id)) => AlignmentItem::Row(id),
        _ => return Err(alignment_error(AlignmentError::InvalidSelection)),
    };
    crate::core::export::skip(&open.store, chapter_id, item).map_err(alignment_error)?;
    read_alignment_wire(open, chapter_id)
}

/// Dissolves a group; its members return to the unprocessed list.
///
/// # Errors
/// As [`alignment_join`]; a group of another Chapter is an invalid selection.
pub fn alignment_unjoin(open: Option<&OpenWork>, chapter_id: i64, group_id: i64) -> Result<ChapterAlignmentWire, IpcError> {
    let open = open.ok_or_else(no_work_open)?;
    crate::core::export::unjoin(&open.store, chapter_id, group_id).map_err(alignment_error)?;
    read_alignment_wire(open, chapter_id)
}

/// Một vỏ `#[tauri::command]`. Không một quy tắc nào sống ở đây.
pub mod wire {
    use super::{
        ChapterAlignmentWire, ExportScope, ExportScopeSummary, ExportedFile, ImageMode, IpcError, PendingReviewerImportState,
        ReviewerImportPreviewWire, ReviewerImportSummaryWire, TextFormat,
    };
    use crate::commands::project::OpenWorkState;
    use crate::core::store::Store;

    /// Vỏ IPC của [`super::export_scope_summary`].
    #[tauri::command]
    pub fn export_scope_summary(
        app: tauri::AppHandle,
        scope: ExportScope,
    ) -> Result<ExportScopeSummary, IpcError> {
        use tauri::Manager as _;

        let Some(state) = app.try_state::<OpenWorkState>() else {
            return super::export_scope_summary(None, &scope);
        };
        let guard = state.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
        super::export_scope_summary(guard.as_ref(), &scope)
    }

    /// `(async)`: `blocking_pick_folder()` on the main thread deadlocks the event loop the
    /// dialog waits on.
    #[tauri::command(async)]
    pub fn export_choose_folder(app: tauri::AppHandle) -> Result<Option<String>, IpcError> {
        use tauri_plugin_dialog::DialogExt as _;

        let path = match app.dialog().file().blocking_pick_folder() {
            Some(picked) => Some(picked.into_path().map_err(|_| super::export_folder_invalid())?),
            None => None,
        };
        super::export_folder_from_dialog(path.as_deref())
    }

    /// `(async)`: serialises every Chapter's text and writes the file, work the main thread
    /// must not carry.
    #[tauri::command(async)]
    pub fn export_docx_two_column(
        app: tauri::AppHandle,
        scope: ExportScope,
        image_mode: ImageMode,
        attribution: bool,
        folder: String,
    ) -> Result<ExportedFile, IpcError> {
        use tauri::Manager as _;

        let folder = std::path::PathBuf::from(folder);
        let global = app.try_state::<Store>();
        let attribution = super::attribution_of(attribution, global.as_deref())?;
        let Some(state) = app.try_state::<OpenWorkState>() else {
            return super::export_docx_two_column(None, &scope, image_mode, attribution.as_ref(), &folder);
        };
        let guard = state.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
        super::export_docx_two_column(guard.as_ref(), &scope, image_mode, attribution.as_ref(), &folder)
    }

    /// `(async)`: serialises every Chapter's text and writes the file, work the main thread
    /// must not carry.
    #[tauri::command(async)]
    pub fn export_docx_one_block(
        app: tauri::AppHandle,
        scope: ExportScope,
        image_mode: ImageMode,
        attribution: bool,
        folder: String,
    ) -> Result<ExportedFile, IpcError> {
        use tauri::Manager as _;

        let folder = std::path::PathBuf::from(folder);
        let global = app.try_state::<Store>();
        let attribution = super::attribution_of(attribution, global.as_deref())?;
        let Some(state) = app.try_state::<OpenWorkState>() else {
            return super::export_docx_one_block(None, &scope, image_mode, attribution.as_ref(), &folder);
        };
        let guard = state.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
        super::export_docx_one_block(guard.as_ref(), &scope, image_mode, attribution.as_ref(), &folder)
    }

    /// `(async)`: serialises every Chapter's text and writes the file, work the main thread
    /// must not carry.
    #[tauri::command(async)]
    pub fn export_text(
        app: tauri::AppHandle,
        scope: ExportScope,
        image_mode: ImageMode,
        format: TextFormat,
        attribution: bool,
        folder: String,
    ) -> Result<ExportedFile, IpcError> {
        use tauri::Manager as _;

        let folder = std::path::PathBuf::from(folder);
        let global = app.try_state::<Store>();
        let attribution = super::attribution_of(attribution, global.as_deref())?;
        let Some(state) = app.try_state::<OpenWorkState>() else {
            return super::export_text(None, &scope, image_mode, format, attribution.as_ref(), &folder);
        };
        let guard = state.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
        super::export_text(guard.as_ref(), &scope, image_mode, format, attribution.as_ref(), &folder)
    }
    /// `(async)`: `blocking_pick_file()` on the main thread deadlocks the event loop the dialog
    /// waits on. `OpenWorkState` is locked only after the dialog closes.
    #[tauri::command(async)]
    pub fn reviewer_import_open_preview(app: tauri::AppHandle) -> Result<Option<ReviewerImportPreviewWire>, IpcError> {
        use tauri::Manager as _;
        use tauri_plugin_dialog::DialogExt as _;

        let Some(pending) = app.try_state::<PendingReviewerImportState>() else {
            return Err(super::reviewer_no_pending());
        };
        super::reviewer_import_cancel(pending.inner());
        let Some(picked) =
            app.dialog().file().add_filter("Reviewer", &["docx", "md"]).blocking_pick_file()
        else {
            return Ok(None);
        };
        let path = picked.into_path().map_err(|_| super::reviewer_unreadable("dialog path"))?;
        let work_state = app.try_state::<OpenWorkState>();
        let guard = work_state.as_ref().map(|s| s.lock().unwrap_or_else(std::sync::PoisonError::into_inner));
        let open = guard.as_ref().and_then(|g| g.as_ref());
        super::reviewer_import_preview(open, pending.inner(), &path).map(Some)
    }

    /// `(async)`: holds `PendingReviewerImportState` for the whole write.
    #[tauri::command(async)]
    pub fn reviewer_import_confirm(app: tauri::AppHandle) -> Result<ReviewerImportSummaryWire, IpcError> {
        use tauri::Manager as _;

        let Some(pending) = app.try_state::<PendingReviewerImportState>() else {
            return Err(super::reviewer_no_pending());
        };
        let work_state = app.try_state::<OpenWorkState>();
        let guard = work_state.as_ref().map(|s| s.lock().unwrap_or_else(std::sync::PoisonError::into_inner));
        let open = guard.as_ref().and_then(|g| g.as_ref());
        let global = app.try_state::<Store>();
        super::reviewer_import_confirm(open, global.as_deref(), pending.inner())
    }

    /// `(async)`: locks `PendingReviewerImportState`, which `reviewer_import_confirm` holds for its write.
    #[tauri::command(async)]
    pub fn reviewer_import_cancel(app: tauri::AppHandle) -> Result<(), IpcError> {
        use tauri::Manager as _;

        if let Some(pending) = app.try_state::<PendingReviewerImportState>() {
            super::reviewer_import_cancel(pending.inner());
        }
        Ok(())
    }

    fn with_open<T>(
        app: &tauri::AppHandle,
        job: impl FnOnce(Option<&crate::commands::project::OpenWork>) -> Result<T, IpcError>,
    ) -> Result<T, IpcError> {
        use tauri::Manager as _;

        let Some(state) = app.try_state::<OpenWorkState>() else {
            return job(None);
        };
        let guard = state.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
        job(guard.as_ref())
    }

    /// Vỏ IPC của [`super::alignment_open`].
    #[tauri::command]
    pub fn alignment_open(app: tauri::AppHandle, chapter_id: i64) -> Result<ChapterAlignmentWire, IpcError> {
        with_open(&app, |open| super::alignment_open(open, chapter_id))
    }

    /// Vỏ IPC của [`super::review_diff`].
    #[tauri::command]
    pub fn review_diff(app: tauri::AppHandle, chapter_id: i64) -> Result<super::ReviewDiffWire, IpcError> {
        with_open(&app, |open| super::review_diff(open, chapter_id))
    }

    /// Vỏ IPC của [`super::alignment_join`].
    #[tauri::command]
    pub fn alignment_join(
        app: tauri::AppHandle,
        chapter_id: i64,
        segment_ids: Vec<i64>,
        row_ids: Vec<i64>,
    ) -> Result<ChapterAlignmentWire, IpcError> {
        with_open(&app, |open| super::alignment_join(open, chapter_id, &segment_ids, &row_ids))
    }

    /// Vỏ IPC của [`super::alignment_skip`].
    #[tauri::command]
    pub fn alignment_skip(
        app: tauri::AppHandle,
        chapter_id: i64,
        segment_id: Option<i64>,
        row_id: Option<i64>,
    ) -> Result<ChapterAlignmentWire, IpcError> {
        with_open(&app, |open| super::alignment_skip(open, chapter_id, segment_id, row_id))
    }

    /// Vỏ IPC của [`super::alignment_unjoin`].
    #[tauri::command]
    pub fn alignment_unjoin(app: tauri::AppHandle, chapter_id: i64, group_id: i64) -> Result<ChapterAlignmentWire, IpcError> {
        with_open(&app, |open| super::alignment_unjoin(open, chapter_id, group_id))
    }
}
