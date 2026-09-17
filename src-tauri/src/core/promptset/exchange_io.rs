//! I/O tệp cho xuất/nhập bộ prompt — Story 4.5, AD-48 (khuôn chép
//! `core::glossary::exchange_io`, TWIN không tái dùng: xem §Design Notes của spec 4.5 —
//! `exchange_io` là "domain-free", nhưng module Glossary trả lỗi RIÊNG của Glossary
//! (`GlossaryError`) và là một story `done`; nhân đôi năm hàm ngắn này rẻ hơn mở lại một
//! module đã đóng để tổng quát hoá kiểu lỗi của nó).
//!
//! ─────────────────────────────────────────────────────────────────────────────
//! 🔴 MODULE DUY NHẤT CỦA `core/promptset/**` ĐƯỢC PHÉP CHẠM HỆ THỐNG TỆP
//! ─────────────────────────────────────────────────────────────────────────────
//! `exchange.rs` vào `&str`, ra `String`, không `std::fs`/`PathBuf`/`tauri::`. Mọi lượt đọc/
//! ghi byte sống Ở ĐÂY.
//!
//! ⚠️ Mọi chuỗi trong `src-tauri/src/**` viết KHÔNG DẤU; doc-comment có dấu là hợp lệ.

use std::io::Read as _;
use std::path::Path;

use uuid::Uuid;

use super::store::PromptSetError;

/// Trần kích thước tệp NHẬP một bộ prompt — 1 MiB. Một bộ prompt là TÊN + THÂN, không hàng
/// lặp lại như Glossary/TM; một thân prompt thật (kể cả dài) đo vài KB. 1 MiB ≈ vài trăm
/// nghìn ký tự — xa trên mọi thân người dùng gõ tay, và một tệp chạm trần chắc chắn là một
/// tệp SAI (không phải `.prompt.md` thật), không phải một tệp lớn hợp lệ.
pub const MAX_PROMPT_SET_IMPORT_BYTES: u64 = 1024 * 1024;

/// Dấu thứ tự byte UTF-8 (`EF BB BF`) — khuôn chép `core::glossary::exchange_io::strip_bom`.
fn strip_bom(raw: &str) -> &str {
    raw.strip_prefix('\u{feff}').unwrap_or(raw)
}

/// Đọc một tệp nhập bộ prompt từ đĩa — chặn THẬT ở `LIMIT + 1` byte qua
/// [`std::io::Read::take`], không qua `metadata` (cùng lý do khử TOCTOU đã ghi ở
/// `core::glossary::exchange_io::read_import_file`).
pub fn read_import_file(path: &Path) -> Result<String, PromptSetError> {
    let path_str = path.display().to_string();

    let file = std::fs::File::open(path)
        .map_err(|e| PromptSetError::ImportReadFailed { path: path_str.clone(), detail: e.to_string() })?;

    let mut bytes = Vec::new();
    file.take(MAX_PROMPT_SET_IMPORT_BYTES + 1)
        .read_to_end(&mut bytes)
        .map_err(|e| PromptSetError::ImportReadFailed { path: path_str.clone(), detail: e.to_string() })?;

    if bytes.len() as u64 > MAX_PROMPT_SET_IMPORT_BYTES {
        return Err(PromptSetError::ImportFileTooLarge {
            size: bytes.len() as u64,
            limit: MAX_PROMPT_SET_IMPORT_BYTES,
        });
    }

    let text = String::from_utf8(bytes).map_err(|_| PromptSetError::ImportNotUtf8 { path: path_str })?;

    Ok(strip_bom(&text).to_owned())
}

/// Ghi `contents` NGUYÊN TỬ xuống `path` — tạm cạnh đích (hậu tố `pid`+`uuid` DUY NHẤT, cùng
/// lý do `core::glossary::exchange_io::write_export_file`) ⇒ `write_all` ⇒ `sync_all` ⇒
/// `rename` ⇒ dọn `.tmp` ở CẢ HAI nhánh lỗi ⇒ fsync thư mục cha.
pub fn write_export_file(path: &Path, contents: &str) -> Result<(), PromptSetError> {
    let path_str = path.display().to_string();

    let Some(file_name) = path.file_name() else {
        return Err(PromptSetError::ExportWriteFailed {
            path: path_str,
            detail: "duong dan khong co thanh phan ten tep -- khong doan duoc ten tep tam".to_owned(),
        });
    };

    let mut tmp_name = file_name.to_os_string();
    tmp_name.push(format!(".{}-{}.tmp", std::process::id(), Uuid::new_v4()));
    let tmp = path.with_file_name(tmp_name);

    let write_result = (|| -> std::io::Result<()> {
        use std::io::Write as _;
        let mut file = std::fs::File::create(&tmp)?;
        file.write_all(contents.as_bytes())?;
        file.sync_all()?;
        Ok(())
    })();

    if let Err(e) = write_result {
        let _ = std::fs::remove_file(&tmp);
        return Err(PromptSetError::ExportWriteFailed { path: path_str, detail: e.to_string() });
    }

    if let Err(e) = std::fs::rename(&tmp, path) {
        let _ = std::fs::remove_file(&tmp);
        return Err(PromptSetError::ExportWriteFailed { path: path_str, detail: e.to_string() });
    }

    if let Some(dir) = path.parent() {
        if let Ok(dir_handle) = std::fs::File::open(dir) {
            let _ = dir_handle.sync_all();
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn read_import_file_rejects_a_file_over_the_cap_without_reading_its_bytes() {
        let dir = std::env::temp_dir()
            .join(format!("aura-promptset-io-test-{}-{}", std::process::id(), "over_cap"));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("big.prompt.md");
        {
            use std::io::{Seek, SeekFrom, Write as _};
            let mut file = std::fs::File::create(&path).unwrap();
            file.seek(SeekFrom::Start(MAX_PROMPT_SET_IMPORT_BYTES + 1)).unwrap();
            file.write_all(b"x").unwrap();
        }

        let err = read_import_file(&path).unwrap_err();
        match err {
            PromptSetError::ImportFileTooLarge { size, limit } => {
                assert_eq!(limit, MAX_PROMPT_SET_IMPORT_BYTES);
                assert_eq!(size, MAX_PROMPT_SET_IMPORT_BYTES + 1);
            }
            other => panic!("mong ImportFileTooLarge, duoc {other:?}"),
        }

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn read_import_file_rejects_non_utf8_bytes_explicitly() {
        let dir =
            std::env::temp_dir().join(format!("aura-promptset-io-test-{}-{}", std::process::id(), "not_utf8"));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("bad.prompt.md");
        std::fs::write(&path, [0xff, 0xfe, 0x00]).unwrap();

        let err = read_import_file(&path).unwrap_err();
        assert!(matches!(err, PromptSetError::ImportNotUtf8 { .. }));

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn read_import_file_strips_a_leading_bom() {
        let dir = std::env::temp_dir().join(format!("aura-promptset-io-test-{}-{}", std::process::id(), "bom"));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("bom.prompt.md");
        let mut bytes = vec![0xEF, 0xBB, 0xBF];
        bytes.extend_from_slice(b"---\nname: X\n---\nthan");
        std::fs::write(&path, bytes).unwrap();

        let text = read_import_file(&path).unwrap();
        assert!(!text.starts_with('\u{feff}'));
        assert!(text.starts_with("---\n"));

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn write_export_file_writes_atomically_and_leaves_no_tmp_file_behind() {
        let dir = std::env::temp_dir().join(format!("aura-promptset-io-test-{}-{}", std::process::id(), "write_ok"));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("out.prompt.md");

        write_export_file(&path, "---\nname: X\n---\nthan").unwrap();

        assert_eq!(std::fs::read_to_string(&path).unwrap(), "---\nname: X\n---\nthan");
        let leftover_tmp: Vec<_> = std::fs::read_dir(&dir)
            .unwrap()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_name().to_string_lossy().contains(".tmp"))
            .collect();
        assert!(leftover_tmp.is_empty(), "khong tep .tmp nao duoc de lai sau mot luot ghi thanh cong");

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn write_export_file_cleans_up_the_tmp_file_when_the_directory_does_not_exist() {
        let dir =
            std::env::temp_dir().join(format!("aura-promptset-io-test-{}-{}", std::process::id(), "write_fail"));
        let path = dir.join("out.prompt.md");

        let err = write_export_file(&path, "x").unwrap_err();
        assert!(matches!(err, PromptSetError::ExportWriteFailed { .. }));
        assert!(!dir.exists());
    }

    #[test]
    fn write_export_file_refuses_a_path_with_no_file_name_component_explicitly() {
        let err = write_export_file(Path::new("/"), "x").unwrap_err();
        match err {
            PromptSetError::ExportWriteFailed { path, .. } => assert_eq!(path, "/"),
            other => panic!("mong ExportWriteFailed, duoc {other:?}"),
        }
        assert!(!Path::new("/.tmp").exists());
    }
}
