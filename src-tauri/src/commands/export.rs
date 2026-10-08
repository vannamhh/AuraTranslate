//! Phạm vi xuất (FR89): đếm Chương và câu của phạm vi, và hộp thoại chọn thư mục đích (AD-48).
//! Phép đếm sống ở [`crate::core::export`]; ở đây chỉ đổi lỗi sang `IpcError`.

use std::collections::BTreeMap;

use crate::commands::chapter::{chapter_not_found, no_work_open};
use crate::commands::project::OpenWork;
use crate::core::export::{
    ExportScope, ImageFilesError, ImageMode, ImageReference, MissingLinkImage, ScopeError, count_scope,
    load_chapter_tables, resolve_chapter_ids, safe_stem, scan_images, write_docx_with_images, write_new_file,
    write_two_column_docx,
};
use crate::core::i18n::{IpcError, MessageKey};

/// Tóm tắt phạm vi xuất: phép đếm của FR89 cùng kết quả quét ảnh (AD-43).
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct ExportScopeSummary {
    pub chapter_count: i64,
    pub segment_count: i64,
    pub unconfirmed_count: i64,
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
        image_count: scan.image_count,
        missing_link_images: scan.missing_link_images,
    })
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

/// Tệp `.docx` vừa ghi. `images_dir` chỉ có khi chế độ file ghi ít nhất một ảnh.
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
    folder: &std::path::Path,
) -> Result<ExportedFile, IpcError> {
    let open = open.ok_or_else(no_work_open)?;
    let counts = count_scope(&open.store, scope).map_err(scope_error)?;
    if !folder.is_dir() {
        return Err(export_folder_invalid());
    }
    let chapter_ids = resolve_chapter_ids(&open.store, scope).map_err(scope_error)?;
    let loaded = load_chapter_tables(&open.store, &chapter_ids, image_mode).map_err(IpcError::from)?;
    let images = loaded.images();
    let stem = format!("{}-hai-cot", safe_stem(&open.meta.name, "export"));
    let (path, images_dir) = if image_mode == ImageMode::File && !images.is_empty() {
        let written = write_docx_with_images(folder, &stem, &open.dir.join("assets"), &images, |final_stem| {
            let dir_name = format!("{final_stem}{}", crate::core::export::IMAGE_DIR_SUFFIX);
            write_two_column_docx(&loaded.tables, ImageReference::Dir(&dir_name)).map_err(|e| e.0)
        })
        .map_err(|err| match err {
            ImageFilesError::SourceMissing { chapter_ord, file_name } => image_file_missing(chapter_ord, &file_name),
            ImageFilesError::Write(detail) => export_write_failed(&detail),
        })?;
        (written.docx_path, Some(written.images_dir))
    } else {
        let bytes = write_two_column_docx(&loaded.tables, ImageReference::Link).map_err(|e| export_write_failed(&e.0))?;
        let path = write_new_file(folder, &stem, "docx", &bytes).map_err(|e| export_write_failed(&e.to_string()))?;
        (path, None)
    };
    let path = path.to_str().ok_or_else(export_folder_invalid)?.to_owned();
    let images_dir = images_dir.map(|dir| dir.to_str().map(str::to_owned).ok_or_else(export_folder_invalid)).transpose()?;
    Ok(ExportedFile {
        path,
        chapter_count: counts.chapter_count,
        segment_count: counts.segment_count,
        image_count: i64::try_from(images.len()).unwrap_or(i64::MAX),
        images_skipped_missing_link: loaded.images_skipped_missing_link,
        images_dir,
    })
}

/// Một vỏ `#[tauri::command]`. Không một quy tắc nào sống ở đây.
pub mod wire {
    use super::{ExportScope, ExportScopeSummary, ExportedFile, ImageMode, IpcError};
    use crate::commands::project::OpenWorkState;

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
        folder: String,
    ) -> Result<ExportedFile, IpcError> {
        use tauri::Manager as _;

        let folder = std::path::PathBuf::from(folder);
        let Some(state) = app.try_state::<OpenWorkState>() else {
            return super::export_docx_two_column(None, &scope, image_mode, &folder);
        };
        let guard = state.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
        super::export_docx_two_column(guard.as_ref(), &scope, image_mode, &folder)
    }
}
