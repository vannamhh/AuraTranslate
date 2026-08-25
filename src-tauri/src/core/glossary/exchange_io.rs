//! I/O tệp cho xuất/nhập Glossary — Story 3.10b, AD-48.
//!
//! ─────────────────────────────────────────────────────────────────────────────
//! 🔴 MODULE DUY NHẤT CỦA `core/glossary/**` ĐƯỢC PHÉP CHẠM HỆ THỐNG TỆP
//! ─────────────────────────────────────────────────────────────────────────────
//! `exchange.rs` (Story 3.10) là một AC ĐÃ `done`: nó vào `&str`, ra `String`, không
//! `std::fs`/`PathBuf`/`tauri::`. Đặt mọi lượt đọc/ghi byte VÀO module RIÊNG này là thứ
//! giữ AC đó còn đúng — `grep -rn "std::fs\|PathBuf\|tauri::" exchange.rs` phải VẪN rỗng
//! sau story này. Module này thì ngược lại: nó CHỈ làm I/O, không phân tích định dạng,
//! không phân loại — hai việc đó vẫn ở `exchange.rs`/`store.rs`.
//!
//! ⚠️ Mọi chuỗi trong `src-tauri/src/**` viết KHÔNG DẤU; doc-comment có dấu là hợp lệ.

use std::path::Path;

use super::store::GlossaryError;

/// Trần kích thước tệp NHẬP Glossary — 16 MiB, RIÊNG với 100 MiB của
/// `core::segment::import::MAX_IMPORT_BYTES`.
///
/// Một tệp Glossary 100 MiB là một tệp SAI, không phải một tệp lớn: một hàng thật cỡ
/// ~200 byte (sáu cột, `note` là ô dài nhất), nên 16 MiB ≈ 80.000 hàng — xa trên mọi bộ
/// thuật ngữ người thật dựng được (mockup lấy ví dụ 604 dòng). Đỉnh bộ nhớ ở trần này:
/// văn bản đọc vào `String`, rồi `Vec<ImportRow>`, rồi `Vec<RowPlan>` — hệ số ~3-5 lần,
/// tức ~80 MiB cho một tệp chạm trần. Chấp nhận được; trần cao hơn thì không.
pub const MAX_GLOSSARY_IMPORT_BYTES: u64 = 16 * 1024 * 1024;

/// Dấu thứ tự byte UTF-8 (`EF BB BF`) — khuôn chép `core::segment::import::strip_bom`.
/// Chỉ cắt ở ĐẦU: một `U+FEFF` giữa văn bản là nội dung thật.
fn strip_bom(raw: &str) -> &str {
    raw.strip_prefix('\u{feff}').unwrap_or(raw)
}

/// Đọc một tệp nhập Glossary từ đĩa.
///
/// 🔴 **Thứ tự CỐ ĐỊNH, khuôn chép `core/segment/import.rs:250-269`:** `metadata` ⇒ so
/// trần TRƯỚC khi đọc byte nào ⇒ `std::fs::read` ⇒ `String::from_utf8` (KHÔNG `_lossy` —
/// phi-UTF-8 bị từ chối tường minh, không đoán bảng mã; dò bảng mã là Epic 6) ⇒ cắt BOM.
/// Đọc trọn rồi mới đo là đúng thứ trần này tồn tại để chặn — kiểm bằng `metadata` giữ
/// **0** byte nạp vào bộ nhớ cho một tệp đã vượt trần.
pub fn read_import_file(path: &Path) -> Result<String, GlossaryError> {
    let path_str = path.display().to_string();

    let metadata = std::fs::metadata(path).map_err(|e| GlossaryError::ImportReadFailed {
        path: path_str.clone(),
        detail: e.to_string(),
    })?;
    let size = metadata.len();
    if size > MAX_GLOSSARY_IMPORT_BYTES {
        return Err(GlossaryError::ImportFileTooLarge { size, limit: MAX_GLOSSARY_IMPORT_BYTES });
    }

    let bytes = std::fs::read(path).map_err(|e| GlossaryError::ImportReadFailed {
        path: path_str.clone(),
        detail: e.to_string(),
    })?;

    let text =
        String::from_utf8(bytes).map_err(|_| GlossaryError::ImportNotUtf8 { path: path_str })?;

    Ok(strip_bom(&text).to_owned())
}

/// Ghi `contents` NGUYÊN TỬ xuống `path` — khuôn chép `core/library/meta.rs::write_atomic`
/// (`:131-166`): tạm cạnh đích ⇒ `write_all` ⇒ `sync_all` ⇒ `rename` ⇒ dọn `.tmp` ở CẢ
/// HAI nhánh lỗi ⇒ fsync thư mục cha.
///
/// ⚠️ **Kho chưa có tiền lệ ghi ra một đường dẫn TUỲ Ý người dùng chọn** — khuôn gốc ghi
/// vào một đường dẫn NỘI BỘ CỐ ĐỊNH (`meta.json` cạnh `.atproj/`). Rủi ro mới: tệp `.tmp`
/// cạnh đích có thể bị hệ điều hành từ chối tạo ở một thư mục người dùng chọn (§Ask
/// First của spec). Lý do vẫn chọn nguyên tử: một tệp cụt sau lượt ghi dở là một BẢN SAO
/// LƯU người dùng tưởng mình đang có — cùng lớp "hỏng trong im lặng" mà kho đã hụt bốn
/// lần.
pub fn write_export_file(path: &Path, contents: &str) -> Result<(), GlossaryError> {
    let path_str = path.display().to_string();

    // 🔴 P8 (vòng rà ba lớp 2026-08-25) — TỪ CHỐI TƯỜNG MINH khi `path` không có thành
    // phần TÊN TỆP (vd. kết thúc bằng `..`, hoặc là gốc `/`) — không ĐOÁN. Bản trước
    // `unwrap_or_default()` trên `file_name()` biến một đường dẫn KHÔNG có tên tệp thành
    // một tệp tạm TRẦN `.tmp` ngay trong thư mục cha (`OsString::default()` rỗng + đuôi
    // `.tmp` = tên tệp `".tmp"`) — một tệp KHÁC HẲN thứ người dùng yêu cầu, ghi ra mà không
    // nói một câu nào.
    let Some(file_name) = path.file_name() else {
        return Err(GlossaryError::ExportWriteFailed {
            path: path_str,
            detail: "duong dan khong co thanh phan ten tep -- khong doan duoc ten tep tam"
                .to_owned(),
        });
    };

    let mut tmp_name = file_name.to_os_string();
    tmp_name.push(".tmp");
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
        return Err(GlossaryError::ExportWriteFailed { path: path_str, detail: e.to_string() });
    }

    if let Err(e) = std::fs::rename(&tmp, path) {
        // ⚠️ Dọn tệp tạm ở CẢ nhánh này — không có nó, một `rename` trượt để lại
        // `<tên>.tmp` nằm cạnh một đích vắng mặt, và lần sau lại thêm một cái nữa.
        let _ = std::fs::remove_file(&tmp);
        return Err(GlossaryError::ExportWriteFailed { path: path_str, detail: e.to_string() });
    }

    // fsync THƯ MỤC CHA — không thừa: `file.sync_all()` làm bền NỘI DUNG tệp tạm,
    // `rename` sửa THƯ MỤC, và mục thư mục đó nằm trong cache hệ tệp cho tới khi chính
    // thư mục được fsync. Im lặng khi trượt có chủ ý (khuôn `meta.rs`) — một khác biệt
    // nền tảng (vd. `File::open` một thư mục thất bại trên Windows) không biến thành
    // một lỗi cho người dùng khi bản ghi chính đã thành công.
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
        let dir = std::env::temp_dir().join(format!(
            "aura-glossary-io-test-{}-{}",
            std::process::id(),
            "over_cap"
        ));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("big.csv");
        // Ghi một tệp vượt trần bằng cách seek rồi write 1 byte — không thật sự cấp phát
        // 16 MiB trong bộ nhớ của TEST, chỉ trên đĩa (sparse trên hầu hết hệ tệp).
        {
            use std::io::{Seek, SeekFrom, Write as _};
            let mut file = std::fs::File::create(&path).unwrap();
            file.seek(SeekFrom::Start(MAX_GLOSSARY_IMPORT_BYTES + 1)).unwrap();
            file.write_all(b"x").unwrap();
        }

        let err = read_import_file(&path).unwrap_err();
        match err {
            GlossaryError::ImportFileTooLarge { size, limit } => {
                assert_eq!(limit, MAX_GLOSSARY_IMPORT_BYTES);
                assert!(size > limit);
            }
            other => panic!("mong ImportFileTooLarge, duoc {other:?}"),
        }

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn read_import_file_rejects_non_utf8_bytes_explicitly() {
        let dir = std::env::temp_dir().join(format!(
            "aura-glossary-io-test-{}-{}",
            std::process::id(),
            "not_utf8"
        ));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("bad.csv");
        std::fs::write(&path, [0xff, 0xfe, 0x00]).unwrap();

        let err = read_import_file(&path).unwrap_err();
        assert!(matches!(err, GlossaryError::ImportNotUtf8 { .. }));

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn read_import_file_strips_a_leading_bom() {
        let dir = std::env::temp_dir().join(format!(
            "aura-glossary-io-test-{}-{}",
            std::process::id(),
            "bom"
        ));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("bom.csv");
        let mut bytes = vec![0xEF, 0xBB, 0xBF];
        bytes.extend_from_slice(b"source_term,translation,note,category,term_origin,created_at\n");
        std::fs::write(&path, bytes).unwrap();

        let text = read_import_file(&path).unwrap();
        assert!(!text.starts_with('\u{feff}'));
        assert!(text.starts_with("source_term,"));

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn write_export_file_writes_atomically_and_leaves_no_tmp_file_behind() {
        let dir = std::env::temp_dir().join(format!(
            "aura-glossary-io-test-{}-{}",
            std::process::id(),
            "write_ok"
        ));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("out.csv");

        write_export_file(&path, "source_term,translation\na,b\n").unwrap();

        assert_eq!(std::fs::read_to_string(&path).unwrap(), "source_term,translation\na,b\n");
        let tmp = dir.join("out.csv.tmp");
        assert!(!tmp.exists(), "tep tam .tmp khong duoc de lai sau mot luot ghi thanh cong");

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn write_export_file_cleans_up_the_tmp_file_when_the_directory_does_not_exist() {
        let dir = std::env::temp_dir().join(format!(
            "aura-glossary-io-test-{}-{}",
            std::process::id(),
            "write_fail"
        ));
        // Thư mục KHÔNG được tạo -- `File::create` phải trượt ở lượt ghi tệp tạm.
        let path = dir.join("out.csv");

        let err = write_export_file(&path, "x").unwrap_err();
        assert!(matches!(err, GlossaryError::ExportWriteFailed { .. }));
        assert!(!dir.join("out.csv.tmp").exists());
    }

    /// P8 (vòng rà ba lớp 2026-08-25) — một đường dẫn KHÔNG có thành phần tên tệp (gốc
    /// `/`) phải bị TỪ CHỐI TƯỜNG MINH, không được đoán ra một tệp tạm trần `.tmp`.
    #[test]
    fn write_export_file_refuses_a_path_with_no_file_name_component_explicitly() {
        let err = write_export_file(Path::new("/"), "x").unwrap_err();
        match err {
            GlossaryError::ExportWriteFailed { path, .. } => assert_eq!(path, "/"),
            other => panic!("mong ExportWriteFailed, duoc {other:?}"),
        }
        assert!(
            !Path::new("/.tmp").exists(),
            "khong duoc doan ra va ghi mot tep tam TRAN o thu muc goc"
        );
    }

    /// Nhánh lỗi THỨ HAI (`rename` trượt, khác nhánh `File::create`/`write_all` ở ca ngay
    /// trên) — dựng bằng cách nhắm đích vào một THƯ MỤC đang có (rename một tệp đè lên một
    /// thư mục luôn trượt). Tệp tạm PHẢI được tạo thành công ở bước này (khác ca trên, nơi
    /// `File::create` chính là bước trượt), nên đây là ca DUY NHẤT thật sự kiểm được dòng
    /// dọn `.tmp` của nhánh `rename`.
    #[test]
    fn write_export_file_cleans_up_the_tmp_file_when_rename_onto_an_existing_directory_fails() {
        let dir = std::env::temp_dir().join(format!(
            "aura-glossary-io-test-{}-{}",
            std::process::id(),
            "rename_fail"
        ));
        std::fs::create_dir_all(&dir).unwrap();
        let target = dir.join("existing_dir");
        std::fs::create_dir_all(&target).unwrap(); // Dich la mot THU MUC -- rename phai truot.

        let err = write_export_file(&target, "x").unwrap_err();
        assert!(matches!(err, GlossaryError::ExportWriteFailed { .. }));
        let mut tmp_name = target.file_name().unwrap().to_os_string();
        tmp_name.push(".tmp");
        assert!(
            !dir.join(tmp_name).exists(),
            "tep tam .tmp khong duoc de lai sau mot luot `rename` truot"
        );

        let _ = std::fs::remove_dir_all(&dir);
    }
}
