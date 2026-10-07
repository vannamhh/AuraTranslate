//! Phạm vi xuất (FR89): đếm Chương và câu của phạm vi, và hộp thoại chọn thư mục đích (AD-48).
//! Phép đếm sống ở [`crate::core::export`]; ở đây chỉ đổi lỗi sang `IpcError`.

use std::collections::BTreeMap;

use crate::commands::chapter::{chapter_not_found, no_work_open};
use crate::commands::project::OpenWork;
use crate::core::export::{ExportScope, ScopeCounts, ScopeError, count_scope};
use crate::core::i18n::{IpcError, MessageKey};

/// Đếm Chương, câu và câu chưa xác nhận của phạm vi xuất trong Tác phẩm đang mở (FR89).
///
/// # Lỗi
/// - chưa mở Tác phẩm ⇒ `work.none_open`;
/// - phạm vi rỗng ⇒ `export.scope_empty`;
/// - Chương không tồn tại ⇒ `segment.chapter_not_found`.
pub fn export_scope_summary(
    open: Option<&OpenWork>,
    scope: &ExportScope,
) -> Result<ScopeCounts, IpcError> {
    let open = open.ok_or_else(no_work_open)?;
    count_scope(&open.store, scope).map_err(|err| match err {
        ScopeError::Empty => {
            IpcError::new("export.scope_empty", MessageKey::ExportScopeEmpty, BTreeMap::new(), false)
        }
        ScopeError::ChapterNotFound(id) => chapter_not_found(id),
        ScopeError::Read(e) => IpcError::from(e),
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

/// Một vỏ `#[tauri::command]`. Không một quy tắc nào sống ở đây.
pub mod wire {
    use super::{ExportScope, IpcError, ScopeCounts};
    use crate::commands::project::OpenWorkState;

    /// Vỏ IPC của [`super::export_scope_summary`].
    #[tauri::command]
    pub fn export_scope_summary(
        app: tauri::AppHandle,
        scope: ExportScope,
    ) -> Result<ScopeCounts, IpcError> {
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
}
