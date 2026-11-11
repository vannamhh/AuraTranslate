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

use crate::core::delimited_text::{self, ReadError, WriteError};

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

/// Reads a Glossary import file; the cap and BOM handling live in `delimited_text::read_text_file`.
pub fn read_import_file(path: &Path) -> Result<String, GlossaryError> {
    delimited_text::read_text_file(path, MAX_GLOSSARY_IMPORT_BYTES).map_err(|e| match e {
        ReadError::ReadFailed { path, detail } => GlossaryError::ImportReadFailed { path, detail },
        ReadError::TooLarge { size, limit } => GlossaryError::ImportFileTooLarge { size, limit },
        ReadError::NotUtf8 { path } => GlossaryError::ImportNotUtf8 { path },
    })
}

/// Writes an export file atomically; see `delimited_text::write_text_file_atomic`.
pub fn write_export_file(path: &Path, contents: &str) -> Result<(), GlossaryError> {
    delimited_text::write_text_file_atomic(path, contents)
        .map_err(|WriteError { path, detail }| GlossaryError::ExportWriteFailed { path, detail })
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
            file.seek(SeekFrom::Start(MAX_GLOSSARY_IMPORT_BYTES + 1))
                .unwrap();
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

    /// 🔴 **Mục ⑧, đối chứng gỡ chỗ nối.** Một tệp trên đĩa LỚN HƠN RẤT NHIỀU lần trần vẫn
    /// phải bị từ chối bằng `ImportFileTooLarge`, và `size` báo về phải bằng ĐÚNG
    /// `LIMIT + 1` — con số đó CHỈ đúng nếu lượt đọc bị CHẶN THẬT ở `LIMIT + 1` byte, không
    /// phải đọc trọn tệp (~134 MB) rồi mới so sánh. Nếu bản vá ⑧ bị gỡ (khôi phục
    /// `metadata` ⇒ so ⇒ `std::fs::read` không chặn), `metadata.len()` sẽ trả kích thước
    /// THẬT của tệp (~134.217.729 byte, không phải `LIMIT + 1` = 16.777.217) — ca này ĐỎ.
    #[test]
    fn read_import_file_never_reads_more_than_the_cap_plus_one_byte_even_for_a_file_far_larger_than_that()
     {
        let dir = std::env::temp_dir().join(format!(
            "aura-glossary-io-test-{}-{}",
            std::process::id(),
            "far_larger_than_cap"
        ));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("huge.csv");
        // Tệp THƯA (sparse) TÁM LẦN trần -- seek roi ghi 1 byte, cung khuon voi ca "over_cap"
        // o tren, khong that su cap phat dung luong do tren hau het he tep.
        {
            use std::io::{Seek, SeekFrom, Write as _};
            let mut file = std::fs::File::create(&path).unwrap();
            file.seek(SeekFrom::Start(MAX_GLOSSARY_IMPORT_BYTES * 8))
                .unwrap();
            file.write_all(b"x").unwrap();
        }

        let err = read_import_file(&path).unwrap_err();
        match err {
            GlossaryError::ImportFileTooLarge { size, limit } => {
                assert_eq!(limit, MAX_GLOSSARY_IMPORT_BYTES);
                assert_eq!(
                    size,
                    MAX_GLOSSARY_IMPORT_BYTES + 1,
                    "size phai DUNG BANG tran+1 -- bang chung DUY NHAT mot test co the quan \
                     sat duoc rang lan doc bi CHAN THAT o do, khong doc tron tep (~134 MB) roi \
                     moi vut di"
                );
            }
            other => panic!("mong ImportFileTooLarge, duoc {other:?}"),
        }

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

        assert_eq!(
            std::fs::read_to_string(&path).unwrap(),
            "source_term,translation\na,b\n"
        );
        // 🔵 SỬA 2026-08-25 (mục ⑨) — tên tạm nay mang hậu tố pid+uuid KHÔNG đoán trước
        // được, nên phép kiểm "không tệp tạm nào sót lại" phải QUÉT thư mục thay vì đoán
        // đúng MỘT cái tên `out.csv.tmp` -- kiểm mạnh hơn bản trước: bắt được BẤT KỲ tệp
        // `.tmp` nào sót lại, không chỉ một cái tên cụ thể.
        let leftover_tmp: Vec<_> = std::fs::read_dir(&dir)
            .unwrap()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_name().to_string_lossy().contains(".tmp"))
            .collect();
        assert!(
            leftover_tmp.is_empty(),
            "khong tep .tmp nao duoc de lai sau mot luot ghi thanh cong, nhung thay: {leftover_tmp:?}"
        );

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
        assert!(
            !dir.exists(),
            "thu muc dich khong ton tai thi khong co gi duoc tao ra ca"
        );
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
        // 🔵 SỬA 2026-08-25 (mục ⑨) — QUÉT thư mục thay vì đoán đúng MỘT tên `.tmp` (xem ca
        // "write_ok" ở trên cho lý do). `target` (thư mục "existing_dir") vẫn phải còn đó --
        // chỉ loại nó ra, mọi entry KHÁC trong `dir` là rác.
        let leftover: Vec<_> = std::fs::read_dir(&dir)
            .unwrap()
            .filter_map(|e| e.ok())
            .filter(|e| e.path() != target)
            .collect();
        assert!(
            leftover.is_empty(),
            "tep tam .tmp khong duoc de lai sau mot luot `rename` truot, nhung thay: {leftover:?}"
        );

        let _ = std::fs::remove_dir_all(&dir);
    }

    /// 🔴 **Mục ⑨, ca TRUNG TÂM.** I/O Matrix "Hai lượt xuất song song cùng một đích": mỗi
    /// lượt phải dùng một tệp tạm RIÊNG (không đoán trước tên bằng cách nào để đảm bảo điều
    /// đó ngoài đọc mã); đích cuối phải là MỘT trong hai bản TRỌN VẸN, không phải một bản
    /// TRỘN của cả hai. `Barrier` xếp hai luồng khởi động lượt ghi GẦN NHƯ ĐỒNG THỜI để tối
    /// đa hoá cửa sổ va chạm; nội dung đủ lớn (vài MB) để lượt ghi tốn đủ thời gian cho cửa
    /// sổ đó có ý nghĩa.
    ///
    /// **Đối chứng gỡ chỗ nối (thủ công):** trả tên tạm về `<tên>.tmp` trần (bỏ hậu tố
    /// pid+uuid) rồi chạy lại NHIỀU LẦN — ca này phải ĐỎ (nội dung cuối cùng KHÔNG khớp
    /// nguyên vẹn một trong hai bản, do cả hai luồng cùng ghi/sync/rename một tệp tạm CHUNG).
    #[test]
    fn two_concurrent_exports_to_the_same_destination_each_use_their_own_temp_file_and_the_final_file_is_one_whole_copy_not_a_merge()
     {
        let dir = std::env::temp_dir().join(format!(
            "aura-glossary-io-test-{}-{}",
            std::process::id(),
            "concurrent_exports"
        ));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("out.csv");

        let content_a: String = "a".repeat(3_000_000);
        let content_b: String = "b".repeat(3_000_000);

        let barrier = std::sync::Arc::new(std::sync::Barrier::new(2));

        let (path_a, content_a_move, barrier_a) =
            (path.clone(), content_a.clone(), barrier.clone());
        let thread_a = std::thread::spawn(move || {
            barrier_a.wait();
            write_export_file(&path_a, &content_a_move)
        });

        let (path_b, content_b_move, barrier_b) =
            (path.clone(), content_b.clone(), barrier.clone());
        let thread_b = std::thread::spawn(move || {
            barrier_b.wait();
            write_export_file(&path_b, &content_b_move)
        });

        thread_a
            .join()
            .expect("luong A khong panic")
            .expect("luot ghi A phai thanh cong");
        thread_b
            .join()
            .expect("luong B khong panic")
            .expect("luot ghi B phai thanh cong");

        let final_content = std::fs::read_to_string(&path).unwrap();
        assert!(
            final_content == content_a || final_content == content_b,
            "dich cuoi phai la MOT trong hai ban TRON VEN -- do dai cuoi cung quan sat duoc: {}",
            final_content.len()
        );

        let leftover_tmp: Vec<_> = std::fs::read_dir(&dir)
            .unwrap()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_name().to_string_lossy().contains(".tmp"))
            .collect();
        assert!(
            leftover_tmp.is_empty(),
            "khong duoc de lai tep .tmp mo coi nao sau khi CA HAI luot da xong: {leftover_tmp:?}"
        );

        let _ = std::fs::remove_dir_all(&dir);
    }
}
