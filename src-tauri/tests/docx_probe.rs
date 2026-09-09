//! Bàn đo AD-38 của Story 6.12 — spec `spec-6-12-doc-docx.md`.
//!
//! ─────────────────────────────────────────────────────────────────────────────
//! 🔴 ĐÂY LÀ MỘT BÀN ĐO, KHÔNG PHẢI MỘT CỔNG NGHIỆM THU
//! ─────────────────────────────────────────────────────────────────────────────
//! `docx_contract.rs` đã kiểm `core::docx::read_docx` trên fixture TỰ SINH (qua `docx-rs`) —
//! điều đó chứng minh **luật của TA** đọc đúng OOXML mà TA tự viết ra, không chứng minh
//! **hình dạng XML mà Word THẬT sự sinh ra** cũng đọc được y hệt (Quyết định Ice 2026-09-09,
//! đường ⚠️ đã ghi rõ trong spec: mẫu tự quy định "không đo được 'Word thật sinh ra hình dạng
//! XML gì'"). File này là bàn đo cho đúng vế đó — quét `src-tauri/tests/fixtures/docx/*.docx`
//! (tệp Word THẬT, gitignore), gọi `read_docx`, ghi TSV. `#[ignore]` vì đây là số đo trên dữ
//! liệu Ice tự cấp, không phải một cổng đúng/sai — đúng khuôn tiền lệ `webimport_probe.rs`.
//!
//! Chạy tay:
//!
//!     cargo test --locked --manifest-path src-tauri/Cargo.toml --test docx_probe \
//!       -- --ignored --nocapture
//!
//! ─────────────────────────────────────────────────────────────────────────────
//! 0 MẪU ⇒ THOÁT KHÁC 0 — khuôn `webimport_probe.rs:260`
//! ─────────────────────────────────────────────────────────────────────────────
//! Thư mục rỗng/vắng mặt là LỖI HẠ TẦNG của bàn đo (Ice chưa thả tệp Word thật vào), không
//! phải một phép đo với tỉ lệ 0%. `assert!` dưới đây panic (thoát khác 0) đúng nghĩa đó —
//! không được lặng lẽ ghi "0 mẫu" rồi coi như đã đo xong.

use std::fs;
use std::path::{Path, PathBuf};

use auratranslate_lib::core::docx::read_docx;

/// `CARGO_MANIFEST_DIR` của crate này là `src-tauri/` — thư mục fixture nằm ngay dưới `tests/`.
fn fixtures_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/docx")
}

/// Thư mục bàn giao của story — khuôn `6-1-ban-do/`.
fn ban_do_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../_bmad-output/implementation-artifacts/6-12-ban-do")
}

fn tsv_escape(s: &str) -> String {
    s.replace(['\t', '\n', '\r'], " ")
}

fn write_tsv(name: &str, header: &str, rows: &[String]) {
    let dir = ban_do_dir();
    fs::create_dir_all(&dir).unwrap_or_else(|e| panic!("tao {}: {e}", dir.display()));
    let path = dir.join(name);
    let mut out = String::new();
    out.push_str(header);
    out.push('\n');
    for row in rows {
        out.push_str(row);
        out.push('\n');
    }
    fs::write(&path, out).unwrap_or_else(|e| panic!("ghi {}: {e}", path.display()));
    println!("BAN_DO_TSV\t{}\t{} hàng", path.display(), rows.len());
}

/// Bàn đo AD-38 — đọc MỌI `.docx` thật trong `fixtures_dir()`, ghi một hàng TSV cho mỗi tệp
/// (tên tệp · số hàng · số ô · số đoạn từng ô · số ảnh · lỗi), **0 mẫu ⇒ thoát khác 0**.
///
/// 🔵 SỬA (khuôn kế thừa từ `webimport_probe.rs`) — GHI HÀNG RỒI MỚI ASSERT: một mẫu lỗi
/// (một `.docx` không đọc được) không được xoá sạch hàng của các mẫu KHÁC đã đọc thành công.
#[test]
#[ignore = "ban do can tep .docx THAT cua Ice trong tests/fixtures/docx/, khong phai mot cong"]
fn read_docx_records_the_real_table_and_image_shape_of_every_real_word_file_or_fails_loudly_on_zero_samples()
{
    let dir = fixtures_dir();
    let mut files: Vec<PathBuf> = fs::read_dir(&dir)
        .into_iter()
        .flatten()
        .filter_map(Result::ok)
        .map(|e| e.path())
        .filter(|p| p.extension().and_then(|e| e.to_str()).map(|e| e.eq_ignore_ascii_case("docx")).unwrap_or(false))
        .collect();
    files.sort();

    println!("DOCX_SAMPLES\t{}", files.len());
    assert!(
        !files.is_empty(),
        "0 mẫu trong {} — Ice CHƯA thả tệp .docx THẬT vào. Đây là LỖI HẠ TẦNG của bàn đo, \
         không phải một phép đo với tỉ lệ đúng 0% — ghi nợ vào deferred-work.md, chủ Ice, \
         không suy phán quyết AD-38 từ đây.",
        dir.display()
    );

    let mut rows = Vec::new();
    let mut ok_count = 0usize;
    let mut err_count = 0usize;

    for path in &files {
        let file_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("").to_owned();
        let bytes = match fs::read(path) {
            Ok(b) => b,
            Err(e) => {
                err_count += 1;
                rows.push(format!("{file_name}\t\t\t\t\t\tio_err: {e}"));
                continue;
            }
        };

        match read_docx(&bytes) {
            Ok(parsed) => {
                ok_count += 1;
                let rows_total: usize = parsed.tables.iter().map(|t| t.rows).sum();
                let cells_total: usize = parsed.tables.iter().flat_map(|t| t.cells_per_row.iter()).sum();
                let paragraphs_per_cell_flat: String = parsed
                    .tables
                    .iter()
                    .flat_map(|t| t.paragraphs_per_cell.iter())
                    .map(|row| {
                        row.iter().map(usize::to_string).collect::<Vec<_>>().join(",")
                    })
                    .collect::<Vec<_>>()
                    .join(";");
                rows.push(format!(
                    "{file_name}\t{}\t{rows_total}\t{cells_total}\t{}\t{}\t",
                    parsed.tables.len(),
                    tsv_escape(&paragraphs_per_cell_flat),
                    parsed.images.len(),
                ));
            }
            Err(e) => {
                err_count += 1;
                rows.push(format!("{file_name}\t\t\t\t\t\t{}", tsv_escape(&e.to_string())));
            }
        }
    }

    write_tsv(
        "docx-raw.tsv",
        "file_name\ttable_count\trows_total\tcells_total\tparagraphs_per_cell\timage_count\terror",
        &rows,
    );
    println!("DOCX_SUMMARY\tsamples={}\tok={ok_count}\terr={err_count}", files.len());

    assert_eq!(rows.len(), files.len(), "phải ghi đúng một hàng cho mỗi tệp, không bỏ sót");
}
