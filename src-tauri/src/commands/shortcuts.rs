//! Shortcut-set file exchange (FR144, AD-48): the native dialog and the file bytes stay in Rust;
//! the webview owns the JSON shape and validates it before any write.
//!
//! String literals here carry no diacritics: `check:i18n` scans `src-tauri/**/*.rs`.

use crate::core::i18n::{IpcError, MessageKey};
use std::path::Path;

/// A shortcut set is a few KiB; the cap only stops a wrong file from being read whole.
const MAX_SHORTCUTS_FILE_BYTES: u64 = 1024 * 1024;

fn file_error(code: &'static str, key: MessageKey, detail: &str, retryable: bool) -> IpcError {
    eprintln!("shortcuts[{code}] {detail}");
    IpcError::new(code, key, std::collections::BTreeMap::new(), retryable)
}

/// Writes `content` to `path` through a temp file and a rename, so a failed write never leaves a
/// half file.
pub fn shortcuts_write_file(path: &Path, content: &str) -> Result<(), IpcError> {
    let failed = |detail: String| {
        file_error(
            "shortcuts.file_write_failed",
            MessageKey::ShortcutsFileWriteFailed,
            &detail,
            true,
        )
    };
    let Some(file_name) = path.file_name() else {
        return Err(failed("no file name".to_owned()));
    };
    let mut tmp_name = file_name.to_os_string();
    tmp_name.push(format!(
        ".{}-{}.tmp",
        std::process::id(),
        uuid::Uuid::new_v4()
    ));
    let tmp = path.with_file_name(tmp_name);
    let written = (|| -> std::io::Result<()> {
        use std::io::Write as _;
        let mut file = std::fs::File::create(&tmp)?;
        file.write_all(content.as_bytes())?;
        file.sync_all()
    })();
    if let Err(e) = written {
        let _ = std::fs::remove_file(&tmp);
        return Err(failed(e.to_string()));
    }
    if let Err(e) = std::fs::rename(&tmp, path) {
        let _ = std::fs::remove_file(&tmp);
        return Err(failed(e.to_string()));
    }
    Ok(())
}

/// Reads a shortcut file as UTF-8 text. Over the cap or not UTF-8 is refused, nothing is kept.
pub fn shortcuts_read_file(path: &Path) -> Result<String, IpcError> {
    use std::io::Read as _;

    let unreadable = |detail: String| {
        file_error(
            "shortcuts.file_unreadable",
            MessageKey::ShortcutsFileUnreadable,
            &detail,
            false,
        )
    };
    let file = std::fs::File::open(path).map_err(|e| unreadable(e.to_string()))?;
    let mut bytes = Vec::new();
    file.take(MAX_SHORTCUTS_FILE_BYTES + 1)
        .read_to_end(&mut bytes)
        .map_err(|e| unreadable(e.to_string()))?;
    if bytes.len() as u64 > MAX_SHORTCUTS_FILE_BYTES {
        return Err(file_error(
            "shortcuts.file_too_large",
            MessageKey::ShortcutsFileTooLarge,
            "over the import cap",
            false,
        ));
    }
    String::from_utf8(bytes).map_err(|e| unreadable(e.to_string()))
}

pub mod wire {
    use crate::core::i18n::IpcError;

    fn picked_to_path(
        picked: tauri_plugin_dialog::FilePath,
    ) -> Result<std::path::PathBuf, IpcError> {
        picked.into_path().map_err(|_| {
            IpcError::new(
                "shortcuts.dialog_path_invalid",
                crate::core::i18n::MessageKey::ShortcutsFileUnreadable,
                Default::default(),
                false,
            )
        })
    }

    /// Writes `content` to a file the user picks in the native save dialog. Returns the written
    /// path, or `None` when the dialog was cancelled (nothing written). Async: the dialog blocks.
    #[tauri::command(async)]
    pub fn shortcuts_export<R: tauri::Runtime>(
        app: tauri::AppHandle<R>,
        content: String,
    ) -> Result<Option<String>, IpcError> {
        use tauri_plugin_dialog::DialogExt as _;

        let Some(picked) = app
            .dialog()
            .file()
            .add_filter("JSON", &["json"])
            .set_file_name("auratranslate-shortcuts.json")
            .blocking_save_file()
        else {
            return Ok(None);
        };
        let path = picked_to_path(picked)?;
        super::shortcuts_write_file(&path, &content)?;
        Ok(Some(path.display().to_string()))
    }

    /// Returns the text of a file the user picks in the native open dialog, or `None` when the
    /// dialog was cancelled. The webview validates and applies it. Async: the dialog blocks.
    #[tauri::command(async)]
    pub fn shortcuts_import<R: tauri::Runtime>(
        app: tauri::AppHandle<R>,
    ) -> Result<Option<String>, IpcError> {
        use tauri_plugin_dialog::DialogExt as _;

        let Some(picked) = app
            .dialog()
            .file()
            .add_filter("JSON", &["json"])
            .blocking_pick_file()
        else {
            return Ok(None);
        };
        super::shortcuts_read_file(&picked_to_path(picked)?).map(Some)
    }
}
