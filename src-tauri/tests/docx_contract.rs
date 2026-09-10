//! Hợp đồng hành vi của Story 6.12 (`core::docx` + đường nhập `.docx`) — mọi hàng của Ma trận
//! I/O spec 6.12, chạy trên fixture THẬT (không dựng tay một `ChapterInput`).
//!
//! `#[path]` nạp [`fixtures_docx`] làm MODULE (không phụ thuộc crate) — xem doc-comment đầu
//! `fixtures_docx.rs` cho lý do đây là cách CHIA SẺ mã DUY NHẤT giữa hai binary test độc lập.
#[path = "fixtures_docx.rs"]
mod fixtures_docx;

use auratranslate_lib::commands::project::{create_work, create_work_from_file};
use auratranslate_lib::core::docx::{DocxError, read_docx};
use auratranslate_lib::core::segment::chapterpattern::ChapterPattern;
use auratranslate_lib::core::segment::import::{ImportError, import_file};
use auratranslate_lib::core::segment::pipeline::{ChapterInput, PipelineShape};
use auratranslate_lib::core::webimport::{BlockBody, DomainLogState};

use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

static COUNTER: AtomicU64 = AtomicU64::new(0);

/// Thư mục tạm RIÊNG cho một ca — cùng khuôn mọi tệp `*_contract.rs` khác (không dùng chung
/// một thư mục giữa các ca chạy song song).
fn temp_dir(label: &str) -> PathBuf {
    let n = COUNTER.fetch_add(1, Ordering::SeqCst);
    let dir = std::env::temp_dir().join(format!(
        "auratranslate-docx-contract-{label}-{}-{n}-{:?}",
        std::process::id(),
        std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default()
    ));
    std::fs::create_dir_all(&dir).unwrap_or_else(|e| panic!("tao {}: {e}", dir.display()));
    dir
}

fn write_docx(dir: &Path, name: &str, bytes: &[u8]) -> PathBuf {
    let path = dir.join(name);
    let mut f = std::fs::File::create(&path).unwrap_or_else(|e| panic!("tao {}: {e}", path.display()));
    f.write_all(bytes).unwrap_or_else(|e| panic!("ghi {}: {e}", path.display()));
    path
}

// ═════════════════════════════════════════════════════════════════════════════════
// core::docx::read_docx trực tiếp — cấu trúc bảng (AD-38), ảnh, văn bản
// ═════════════════════════════════════════════════════════════════════════════════

/// Ma trận I/O — "Văn bản thường".
#[test]
fn plain_docx_yields_the_expected_paragraph_text_and_zero_tables_or_images() {
    let bytes = fixtures_docx::plain();
    let parsed = read_docx(&bytes).expect("doc fixture plain that bai");

    assert!(parsed.text.contains("Doan mot khong bang khong anh."));
    assert!(parsed.text.contains("Doan hai ket thuc cau nay."));
    assert!(parsed.tables.is_empty(), "fixture plain khong co bang nao");
    assert!(parsed.images.is_empty(), "fixture plain khong co anh nao");
    assert!(
        parsed.blocks.iter().all(|b| matches!(b.body, BlockBody::Paragraph(_))),
        "moi khoi cua fixture plain phai la Paragraph"
    );
}

/// Ma trận I/O — "Có bảng": đếm được hàng · ô · đoạn trong từng ô; MỖI ô là một ĐOẠN RIÊNG
/// trong văn bản ra.
#[test]
fn a_table_yields_exact_row_cell_and_paragraph_counts_and_each_cell_becomes_its_own_paragraph_block() {
    let bytes = fixtures_docx::table_two_columns();
    let parsed = read_docx(&bytes).expect("doc fixture table_two_columns that bai");

    assert_eq!(parsed.tables.len(), 1, "dung mot bang cap cao nhat");
    let table = &parsed.tables[0];
    assert_eq!(table.rows, 2, "bang phai co 2 hang");
    assert_eq!(table.cells_per_row, vec![2, 2], "moi hang phai co 2 o");
    assert_eq!(
        table.paragraphs_per_cell,
        vec![vec![1, 1], vec![1, 1]],
        "moi o phai dem dung 1 doan (ca hai cau cua o dau van nam trong CUNG mot doan)"
    );

    // Mỗi ô là một Paragraph Block RIÊNG trong danh sách khối phẳng.
    let paragraph_texts: Vec<&str> = parsed
        .blocks
        .iter()
        .filter_map(|b| match &b.body {
            BlockBody::Paragraph(t) => Some(t.as_str()),
            _ => None,
        })
        .collect();
    assert_eq!(paragraph_texts.len(), 4, "4 o = 4 khoi Paragraph, khong nhieu khong it hon");
    assert!(paragraph_texts[0].contains("Cau mot that day du trong o nay."));
    assert!(paragraph_texts[0].contains("Cau hai tiep tuc luon trong cung mot doan."));
}

/// Ca AD-38 đặt tên trong spec: "bảng một hàng ô nhiều đoạn". Story này chỉ CẤP năng lực đếm
/// — không một phán quyết nào được đưa ra ở đây (đó là Story 8.8, §Never spec 6.12).
#[test]
fn a_one_row_table_with_a_multi_paragraph_cell_is_counted_exactly_rows_cells_and_paragraphs() {
    let bytes = fixtures_docx::table_one_row_multi_paragraph();
    let parsed = read_docx(&bytes).expect("doc fixture table_one_row_multi_paragraph that bai");

    assert_eq!(parsed.tables.len(), 1);
    let table = &parsed.tables[0];
    assert_eq!(table.rows, 1, "dung mot hang");
    assert_eq!(table.cells_per_row, vec![2], "hang do co hai o");
    assert_eq!(
        table.paragraphs_per_cell,
        vec![vec![2, 1]],
        "o dau co HAI doan RIENG (hai the w:p khac nhau), o sau co mot doan"
    );
}

/// Ma trận I/O — "Ảnh nhúng": một tệp thật trong `assets/`, một hàng `asset`, `source_url IS
/// NULL`, neo đúng vị trí. Đây là ca `read_docx` trực tiếp — phần "đi trọn đường sản phẩm"
/// (ghi tệp + hàng `asset`) được kiểm ở phần dưới, qua `import_file`/`create_work` thật.
#[test]
fn an_embedded_image_is_read_as_a_real_block_with_real_bytes_at_the_right_position() {
    let bytes = fixtures_docx::image_png();
    let parsed = read_docx(&bytes).expect("doc fixture image_png that bai");

    assert_eq!(parsed.images.len(), 1, "dung mot anh nhung duoc doc ra");
    let image_block_index = parsed
        .blocks
        .iter()
        .position(|b| matches!(b.body, BlockBody::Image { .. }))
        .expect("phai co dung mot khoi Image trong danh sach khoi");
    assert_eq!(parsed.images[0].block_index, image_block_index, "block_index phai tro dung vi tri anh trong `blocks`");
    assert!(!parsed.images[0].bytes.is_empty(), "byte anh phai la byte THAT, khong rong");
    assert_eq!(parsed.images[0].content_type, "image/png");

    // Đoạn TRƯỚC và SAU ảnh phải có mặt, đúng thứ tự — đủ điều kiện cho `compute_anchor`.
    let paragraph_texts: Vec<&str> = parsed
        .blocks
        .iter()
        .filter_map(|b| match &b.body {
            BlockBody::Paragraph(t) => Some(t.as_str()),
            _ => None,
        })
        .collect();
    assert_eq!(paragraph_texts, vec!["Doan truoc anh.", "Doan sau anh."]);
    // Ảnh phải nằm GIỮA hai đoạn theo chỉ số khối.
    let before_idx = parsed.blocks.iter().position(|b| matches!(&b.body, BlockBody::Paragraph(t) if t == "Doan truoc anh.")).unwrap();
    let after_idx = parsed.blocks.iter().position(|b| matches!(&b.body, BlockBody::Paragraph(t) if t == "Doan sau anh.")).unwrap();
    assert!(before_idx < image_block_index && image_block_index < after_idx, "anh phai dung GIUA hai doan");
}

/// Ma trận I/O — "Rỗng": lỗi "không có văn bản".
#[test]
fn an_empty_docx_is_rejected_as_having_no_text() {
    let bytes = fixtures_docx::empty();
    let err = read_docx(&bytes).expect_err("docx 0 doan co chu phai bi tu choi");
    assert!(matches!(err, DocxError::EmptyText), "loi phai la EmptyText, khong phai mot loai khac: {err:?}");
}

/// Ma trận I/O — "Tệp hỏng" (zip cắt cụt) và "Đuôi `.docx`, không phải zip": cùng một lỗi cho
/// cả hai, nhận theo NỘI DUNG không theo tên (§Never spec 6.12).
#[test]
fn a_truncated_zip_and_a_non_zip_file_are_both_rejected_as_not_a_valid_zip() {
    let truncated_err = read_docx(&fixtures_docx::truncated()).expect_err("zip cat cut phai bi tu choi");
    assert!(matches!(truncated_err, DocxError::NotAZip { .. }), "phai la NotAZip: {truncated_err:?}");

    let not_zip_err = read_docx(&fixtures_docx::not_a_zip()).expect_err("mot tep khong phai zip phai bi tu choi");
    assert!(matches!(not_zip_err, DocxError::NotAZip { .. }), "phai la NotAZip: {not_zip_err:?}");
}

/// Đối chứng ÂM — một `.docx` HỢP LỆ (nguyên vẹn, không cắt) không bao giờ bị `read_docx` từ
/// chối oan bằng `NotAZip`/`MalformedXml`.
#[test]
fn every_well_formed_fixture_is_accepted_without_error() {
    for (label, bytes) in [
        ("plain", fixtures_docx::plain()),
        ("table_two_columns", fixtures_docx::table_two_columns()),
        ("table_one_row_multi_paragraph", fixtures_docx::table_one_row_multi_paragraph()),
        ("image_png", fixtures_docx::image_png()),
    ] {
        let result = read_docx(&bytes);
        assert!(result.is_ok(), "fixture {label} HOP LE phai duoc chap nhan, khong duoc loi: {result:?}");
    }
}

// ═════════════════════════════════════════════════════════════════════════════════
// import_file — nhánh .docx, AC8 (đuôi mở rộng), 0 byte vào transcode
// ═════════════════════════════════════════════════════════════════════════════════

/// §Always spec 6.12 — `.docx` đến pipeline dưới `ChapterInput::AlreadyText`, không
/// `RawBytes` (0 byte đi vào vế transcode của `Step::DecodeEncoding`).
#[test]
fn docx_files_enter_the_pipeline_as_already_text_not_raw_bytes() {
    let dir = temp_dir("shape");
    let path = write_docx(&dir, "mot.docx", &fixtures_docx::plain());

    let (shape, sidecar) = import_file(&path).expect("nhap .docx hop le phai thanh cong");
    match shape {
        PipelineShape::Blob(ChapterInput::AlreadyText(text)) => {
            assert!(text.contains("Doan mot khong bang khong anh."));
        }
        other => panic!("shape phai la Blob(AlreadyText), khong phai: {other:?}"),
    }
    assert!(sidecar.is_some(), "mot .docx phai luon mang mot DocxSidecar (co the rong anh, khong bao gio None)");
}

/// Đối chứng đỏ ① của §Verification spec 6.12 — GỠ `"docx"` khỏi danh sách chấp nhận phải
/// làm import_file từ chối. Vì `SUPPORTED_EXTENSIONS` là `const` riêng tư của
/// `core::segment::import`, đối chứng THẬT (sửa hằng rồi build lại) chạy TAY, không tự động
/// hoá được trong CHÍNH bộ test — ca này khoá hành vi NGƯỢC LẠI: hôm nay `.docx` PHẢI được
/// chấp nhận, để một lượt sửa lỡ tay xoá "docx" khỏi mảng đó làm ĐÚNG ca này đỏ.
#[test]
fn docx_extension_is_currently_accepted_the_red_side_of_verification_counterfactual_1() {
    let dir = temp_dir("ac1");
    let path = write_docx(&dir, "mot.docx", &fixtures_docx::plain());
    let result = import_file(&path);
    assert!(result.is_ok(), "docx phai duoc chap nhan hom nay -- neu do la mot loi UnsupportedFormat, doi chung 1 da dao nguoc");
}

/// Ma trận I/O — "Quá cỡ": từ chối TRƯỚC khi đọc zip. Dùng `truncated`/`not_a_zip` cũng đủ
/// byte để trần 100 MB không chạm tới trong test thường — ca NÀY canh đúng THỨ TỰ (kích thước
/// hỏi trước khi mở zip), không canh con số 100 MB (đã có test khác canh hằng đó).
#[test]
fn an_oversized_docx_is_rejected_before_the_zip_is_ever_opened() {
    let dir = temp_dir("oversize");
    // Một chuỗi > 100 MB nhưng KHÔNG phải zip hợp lệ -- nếu `import_file` mở zip TRƯỚC khi đo
    // kich thuoc, no se tra DocxUnreadable (NotAZip) thay vi TooLarge; nếu đo kích thước
    // TRƯỚC (đúng thứ tự spec), nó phải trả TooLarge dù nội dung không phải zip.
    let oversized = vec![b'x'; 100 * 1024 * 1024 + 1];
    let path = write_docx(&dir, "qua_co.docx", &oversized);
    let err = import_file(&path).expect_err("tep qua tran phai bi tu choi");
    assert!(
        matches!(err, ImportError::TooLarge { .. }),
        "phai la TooLarge (do kich thuoc TRUOC khi mo zip), khong phai mot loai loi khac: {err:?}"
    );
}

/// Ma trận I/O — "Rỗng" (đường `import_file`, không chỉ `read_docx` trực tiếp).
#[test]
fn importing_an_empty_docx_file_fails_with_a_distinguishable_error() {
    let dir = temp_dir("empty");
    let path = write_docx(&dir, "rong.docx", &fixtures_docx::empty());
    let err = import_file(&path).expect_err("docx rong phai bi tu choi qua import_file");
    assert!(matches!(err, ImportError::DocxEmptyText { .. }), "phai la DocxEmptyText: {err:?}");
}

/// Ma trận I/O — "Tệp hỏng"/"Đuôi `.docx`, không phải zip" qua `import_file` — cùng MỘT lỗi
/// cho cả hai (§Never spec 6.12).
#[test]
fn importing_a_truncated_or_non_zip_docx_file_fails_with_the_same_distinguishable_error() {
    let dir = temp_dir("hong");

    let truncated_path = write_docx(&dir, "cat_cut.docx", &fixtures_docx::truncated());
    let truncated_err = import_file(&truncated_path).expect_err("docx cat cut phai bi tu choi");
    assert!(matches!(truncated_err, ImportError::DocxUnreadable { .. }), "{truncated_err:?}");

    let disguised_path = write_docx(&dir, "gia_docx.docx", &fixtures_docx::not_a_zip());
    let disguised_err = import_file(&disguised_path).expect_err("tep .txt doi duoi .docx phai bi tu choi");
    assert!(matches!(disguised_err, ImportError::DocxUnreadable { .. }), "{disguised_err:?}");
}

// ═════════════════════════════════════════════════════════════════════════════════
// Đi trọn đường sản phẩm — create_work_from_file (import_file → run_pipeline → ghi)
// ═════════════════════════════════════════════════════════════════════════════════

type AssetRow = (i64, String, Option<String>, i64, i64, String);

fn read_asset_rows(store: &auratranslate_lib::core::store::Store) -> Vec<AssetRow> {
    store
        .read(|conn| {
            let mut stmt = conn.prepare(
                "SELECT chapter_id, file_name, source_url, anchor_after_segment_ord, byte_len, \
                 content_type FROM asset ORDER BY id",
            )?;
            let mut rows = stmt.query([])?;
            let mut out = Vec::new();
            while let Some(row) = rows.next()? {
                out.push((
                    row.get::<_, i64>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, Option<String>>(2)?,
                    row.get::<_, i64>(3)?,
                    row.get::<_, i64>(4)?,
                    row.get::<_, String>(5)?,
                ));
            }
            Ok(out)
        })
        .expect("doc bang asset that bai")
}

fn read_segment_texts(store: &auratranslate_lib::core::store::Store, chapter_id: i64) -> Vec<String> {
    store
        .read(|conn| {
            let mut stmt =
                conn.prepare("SELECT source_text FROM segment WHERE chapter_id = ?1 ORDER BY ord")?;
            let mut rows = stmt.query([chapter_id])?;
            let mut out = Vec::new();
            while let Some(row) = rows.next()? {
                out.push(row.get::<_, String>(0)?);
            }
            Ok(out)
        })
        .expect("doc bang segment that bai")
}

/// Đối chứng đỏ ③ của §Verification spec 6.12 — GIỮ hàng: một `.docx` có ảnh nhúng đi TRỌN
/// đường sản phẩm phải cho **một tệp thật** trong `assets/` VÀ **một hàng `asset`** với
/// `source_url IS NULL`. Ca này phải ĐỎ nếu ai chỉ ghi hàng SQL mà không ghi tệp (kiểm bằng
/// `fs::metadata` trên chính đường dẫn `assets/<file_name>`, không suy từ `images_saved`).
#[test]
fn a_docx_with_an_embedded_image_writes_a_real_file_and_a_null_source_url_asset_row() {
    let source_dir = temp_dir("e2e-image-src");
    let root = temp_dir("e2e-image-root");
    let path = write_docx(&source_dir, "co_anh.docx", &fixtures_docx::image_png());

    let opened =
        create_work_from_file(&root, "Docx Co Anh", "en", "", &path).expect("tao Tac pham tu .docx co anh that bai");

    assert_eq!(opened.images_saved, 1, "dung mot anh duoc luu");
    assert_eq!(opened.images_failed, 0, "khong anh nao duoc mong doi that bai");

    let rows = read_asset_rows(&opened.store);
    assert_eq!(rows.len(), 1, "dung mot hang asset");
    let (chapter_id, file_name, source_url, anchor, byte_len, content_type) = &rows[0];
    assert_eq!(*chapter_id, opened.chapter_id);
    assert_eq!(source_url, &None, "anh .docx khong den tu mang -- source_url PHAI la NULL");
    assert_eq!(content_type, "image/png");
    assert!(*byte_len > 0, "byte_len phai duong");
    assert_eq!(*anchor, 1, "dung mot doan (\"Doan truoc anh.\") dung TRUOC anh");

    let asset_path = opened.dir.join("assets").join(file_name);
    let metadata = std::fs::metadata(&asset_path);
    assert!(
        metadata.is_ok(),
        "tep {} PHAI ton tai that su tren dia -- mot hang SQL khong kem tep la mot ca DO",
        asset_path.display()
    );
    assert!(metadata.expect("da kiem o tren").len() > 0, "tep anh phai co byte, khong duoc rong");
}

/// Ma trận I/O — "Ô nhiều câu": ranh giới ô = ranh giới đoạn ⇒ **0** segment vắt qua hai ô
/// (đóng nợ `:2214`). Đi TRỌN đường sản phẩm — kiểm trên `segment` THẬT đã ghi xuống, không
/// suy từ `read_docx` một mình.
#[test]
fn no_segment_crosses_a_table_cell_boundary_after_writing_the_whole_document() {
    let source_dir = temp_dir("e2e-table-src");
    let root = temp_dir("e2e-table-root");
    let path = write_docx(&source_dir, "bang.docx", &fixtures_docx::table_two_columns());

    let opened = create_work_from_file(&root, "Docx Bang", "en", "", &path).expect("tao Tac pham tu .docx bang that bai");

    let segments = read_segment_texts(&opened.store, opened.chapter_id);
    assert!(segments.len() >= 4, "it nhat bon cau rieng biet phai duoc tach (hai cau cua o dau + ba o don cau)");

    // Hai câu của Ô (0,0) phải là HAI segment TÁCH BIỆT (bộ tách câu vẫn hoạt động ĐÚNG bên
    // trong một ô), nhưng KHÔNG segment nào được nối chữ CUỐI của ô này với chữ ĐẦU của ô kế
    // tiếp -- đúng bằng chứng đóng nợ `:2214`.
    assert!(
        segments.iter().any(|s| s.contains("Cau mot that day du trong o nay.")),
        "cau dau cua o (0,0) phai la mot segment day du: {segments:?}"
    );
    assert!(
        segments.iter().any(|s| s.contains("Cau hai tiep tuc luon trong cung mot doan.")),
        "cau hai cua o (0,0) phai la mot segment day du: {segments:?}"
    );
    for text in &segments {
        assert!(
            !(text.contains("cung mot doan.") && text.contains("O hang mot cot hai")),
            "MOT segment KHONG duoc noi chu cuoi cua o (0,0) voi chu dau cua o (0,1) -- day \
             chinh la nguy co ma no `:2214` mo ta: {text:?}"
        );
    }
}

/// Ma trận I/O — "Tệp hỏng"/"Đuôi `.docx`, không phải zip": **0 byte** ghi xuống, **0** thư
/// mục `.atproj` để lại. `import_file` thất bại TRƯỚC khi `create_work_folder` từng được gọi
/// (`create_work_from_file` gọi `import_file(path)?` TRƯỚC `create_work`) — kiểm bằng cách
/// đếm SỐ MỤC trong `documents_root` sau lượt trượt, không suy từ "hàm trả Err".
#[test]
fn a_broken_docx_import_leaves_zero_atproj_folders_behind() {
    let source_dir = temp_dir("no-leftover-src");
    let root = temp_dir("no-leftover-root");

    for (label, bytes) in [("cat_cut", fixtures_docx::truncated()), ("gia_docx", fixtures_docx::not_a_zip())] {
        let path = write_docx(&source_dir, &format!("{label}.docx"), &bytes);
        let result = create_work_from_file(&root, &format!("Docx Hong {label}"), "en", "", &path);
        assert!(result.is_err(), "mot .docx hong phai bi tu choi qua create_work_from_file");
    }

    let entries: Vec<_> = std::fs::read_dir(&root)
        .unwrap_or_else(|e| panic!("doc {}: {e}", root.display()))
        .filter_map(Result::ok)
        .collect();
    assert!(
        entries.is_empty(),
        "0 thu muc .atproj duoc phep con lai sau MOI lan .docx hong bi tu choi -- tim thay: {:?}",
        entries.iter().map(|e| e.path()).collect::<Vec<_>>()
    );
}

/// Đối chứng ÂM — `.txt`/`.md`/dán tay không đổi MỘT BYTE nào so với trước story này (§Always
/// spec 6.12 — "phép đối chứng ④").
#[test]
fn a_plain_text_file_import_is_completely_unaffected_by_the_docx_branch() {
    let dir = temp_dir("txt-unaffected");
    let content = b"Noi dung .txt binh thuong, khong lien quan gi den .docx.";
    let path = write_docx(&dir, "thuong.txt", content);

    let (shape, sidecar) = import_file(&path).expect("nhap .txt hop le phai thanh cong");
    assert!(sidecar.is_none(), ".txt khong bao gio mang mot DocxSidecar");
    match shape {
        PipelineShape::Blob(ChapterInput::RawBytes { bytes, .. }) => {
            assert_eq!(bytes, content, ".txt phai giu NGUYEN byte, chua giai ma (Step::DecodeEncoding moi giai ma)");
        }
        other => panic!(".txt phai la Blob(RawBytes), khong phai: {other:?}"),
    }
}

/// Ma trận I/O — vế XỬ LÝ LỖI của hàng "Ảnh nhúng": *"MIME ngoài danh mục 4 ⇒ bỏ ảnh, ghi
/// `images_failed`, Chương **vẫn** nhập"*.
///
/// 🔴 **Hàng này KHÔNG có ca test nào cho tới lượt rà 2026-09-09** — bảy fixture đầu đều mang
/// ảnh PNG hợp lệ, nên vế "ngoài danh mục" chưa từng được một dòng test nào chạm. Đúng lớp
/// lỗi mà `AGENTS.md` gọi tên (một bộ test xanh không chứng minh chỗ nối được canh) và Story
/// 6.10 đã dính một lần với ba hàng ma trận không ca nào.
///
/// Kiểm bằng TRẠNG THÁI THẬT trên đĩa và trong `project.db`, không suy từ `images_failed`:
/// thư mục `assets/` phải RỖNG và bảng `asset` phải 0 hàng, trong khi văn bản hai đoạn kẹp
/// quanh ảnh vẫn vào đủ.
#[test]
fn an_embedded_image_with_a_mime_outside_the_closed_four_is_dropped_and_the_chapter_still_imports() {
    let source_dir = temp_dir("e2e-bmp-src");
    let root = temp_dir("e2e-bmp-root");
    let path = write_docx(&source_dir, "anh_bmp.docx", &fixtures_docx::image_unsupported_mime());

    let opened = create_work_from_file(&root, "Docx Anh Bmp", "en", "", &path)
        .expect("mot anh MIME ngoai danh muc KHONG duoc lam truot ca luot nhap");

    assert_eq!(opened.images_saved, 0, "khong anh nao duoc luu");
    assert_eq!(opened.images_failed, 1, "dung mot vi tri anh bi bo vi MIME ngoai danh muc");

    assert_eq!(read_asset_rows(&opened.store).len(), 0, "bang asset phai 0 hang");

    let assets_dir = opened.dir.join("assets");
    let entries = std::fs::read_dir(&assets_dir)
        .map(|it| it.count())
        .unwrap_or_else(|e| panic!("doc {}: {e}", assets_dir.display()));
    assert_eq!(entries, 0, "thu muc assets/ phai RONG -- 0 tep duoc ghi cho mot MIME bi tu choi");

    let texts = read_segment_texts(&opened.store, opened.chapter_id).join(" ");
    assert!(texts.contains("Doan truoc anh bmp"), "doan TRUOC anh phai vao du: {texts:?}");
    assert!(texts.contains("Doan sau anh bmp"), "doan SAU anh phai vao du: {texts:?}");
}

// ═════════════════════════════════════════════════════════════════════════════════
// Bắt được sau code review — bốn ca mới
// ═════════════════════════════════════════════════════════════════════════════════

/// Word gói MỘT ảnh thành `mc:AlternateContent` với hai nhánh CÙNG một rId
/// (`mc:Choice > w:drawing` DrawingML, `mc:Fallback > w:pict` VML dự phòng). ⚠️ **Tần suất
/// THẬT của khuôn này chưa đo được** — kho có 0 tệp Word thật (xem `6-12-ban-do/REPORT.md`),
/// nên đừng đọc ca này thành "Word áp khuôn đó cho MỌI ảnh"; điều ca này chứng minh là: NẾU
/// khuôn đó tới thì bộ đọc cho đúng một khối. Trước lượt sửa này, `mc:Fallback` KHÔNG được
/// nhảy trọn subtree, nên phép duyệt phẳng thấy cả hai nhánh cho đúng một ảnh, sinh HAI khối.
#[test]
fn a_word_alternate_content_wrapper_around_one_image_yields_exactly_one_image_block() {
    let bytes = fixtures_docx::image_wrapped_in_alternate_content();
    let parsed = read_docx(&bytes).expect("doc fixture AlternateContent that bai");

    let image_blocks: Vec<_> =
        parsed.blocks.iter().filter(|b| matches!(b.body, BlockBody::Image { .. })).collect();
    assert_eq!(
        image_blocks.len(),
        1,
        "mc:Choice/w:drawing va mc:Fallback/w:pict cung mot rId phai cho DUNG MOT khoi Image: {image_blocks:?}"
    );
    assert_eq!(parsed.images.len(), 1, "dung mot anh duoc doc ra byte, khong hai");
    assert!(!parsed.images[0].bytes.is_empty(), "byte anh phai la byte THAT, khong rong");
}

/// Nhánh VML (`w:pict`/`v:imagedata`) của `find_image_rel_id` trước lượt sửa này KHÔNG có ca
/// test nào chạm tới — fixture chỉ mang `w:pict`, KHÔNG một `w:drawing` nào trong toàn tài
/// liệu, để đúng nhánh đó (không nhánh DrawingML) là nhánh duy nhất được thực thi.
#[test]
fn a_vml_only_image_with_no_drawing_anywhere_still_resolves_its_rel_id_and_reads_real_bytes() {
    let bytes = fixtures_docx::image_vml_only();
    let parsed = read_docx(&bytes).expect("doc fixture VML-only that bai");

    assert_eq!(parsed.images.len(), 1, "dung mot anh VML duoc doc ra");
    assert!(!parsed.images[0].bytes.is_empty(), "byte anh phai la byte THAT, khong rong");
    let image_block_index = parsed
        .blocks
        .iter()
        .position(|b| matches!(b.body, BlockBody::Image { .. }))
        .expect("phai co dung mot khoi Image trong danh sach khoi");
    assert_eq!(parsed.images[0].block_index, image_block_index, "block_index phai tro dung vi tri anh VML");
}

/// `parse_cell` nhảy trọn một `w:tbl` LỒNG trong ô rồi đi tiếp (nợ có chủ: nội dung bảng lồng
/// không được đọc) — nhưng phần CÒN LẠI của ô, đứng SAU bảng lồng, phải vẫn được đọc đúng.
/// Trước lượt sửa này, mệnh đề đó chỉ tự khai trong doc-comment, không phép đo nào canh.
#[test]
fn a_paragraph_after_a_nested_table_in_the_same_cell_is_still_read() {
    let bytes = fixtures_docx::table_with_nested_table_and_trailing_paragraph();
    let parsed = read_docx(&bytes).expect("doc fixture bang long that bai");

    assert_eq!(parsed.tables.len(), 1, "bang LONG khong duoc dem la mot TableShape rieng -- no bi nhay tron, khong doc");
    let outer = &parsed.tables[0];
    assert_eq!(outer.rows, 1);
    assert_eq!(outer.cells_per_row, vec![1]);
    assert_eq!(
        outer.paragraphs_per_cell,
        vec![vec![2]],
        "doan TRUOC va SAU bang long phai duoc dem la hai doan cua o NGOAI, bang long khong duoc lan vao"
    );

    let paragraph_texts: Vec<&str> = parsed
        .blocks
        .iter()
        .filter_map(|b| match &b.body {
            BlockBody::Paragraph(t) => Some(t.as_str()),
            _ => None,
        })
        .collect();
    assert_eq!(
        paragraph_texts,
        vec!["Doan truoc bang long.", "Doan sau bang long."],
        "doan SAU bang long phai co mat -- phep nhay subtree khong duoc nuot mat phan con lai cua o"
    );
    assert!(
        !paragraph_texts.iter().any(|t| t.contains("Doan trong bang long")),
        "noi dung CUA bang long phai KHONG xuat hien -- no bi nhay tron subtree, khong doc (nợ co chu)"
    );
}

/// §Always spec 6.12 khai đường `.docx` là "0 mạng, 0 `Allowlist`, 0 `DomainLogState`" — trước
/// lượt sửa này, `grep domain_log` trong tệp này trúng 0 lần, tức mệnh đề đó tự khai mà không
/// phép đo nào canh. Nhập một `.docx` CÓ ảnh (đường duy nhất còn có thể chạm `DomainLogState`)
/// qua đúng `create_work` sản phẩm rồi đọc lại nhật ký domain — phải RỖNG.
#[test]
fn importing_a_docx_with_an_image_leaves_the_domain_log_empty() {
    let source_dir = temp_dir("domainlog-src");
    let root = temp_dir("domainlog-root");
    let path = write_docx(&source_dir, "co_anh.docx", &fixtures_docx::image_png());

    let (shape, docx_sidecar) = import_file(&path).expect("nhap .docx co anh phai thanh cong");
    let domain_log_state: DomainLogState = std::sync::Mutex::new(Vec::new());
    let _opened = create_work(
        &root,
        "Docx Domain Log",
        "en",
        "",
        shape,
        encoding_rs::UTF_8,
        Vec::new(),
        None,
        Vec::new(), &[],
        &domain_log_state,
        docx_sidecar)
    .expect("tao Tac pham tu .docx co anh that bai");

    let log = domain_log_state.lock().unwrap_or_else(|e| e.into_inner());
    assert!(
        log.is_empty(),
        "duong .docx khong duoc ghi nhat ky domain (0 mang, §Always spec 6.12) -- tim thay: {log:?}"
    );
}

/// `DocxSidecar::blocks` gắn TRỌN VẸN vào Chương ĐẦU TIÊN (`chapters.first_mut()`), nhưng
/// `.docx` vẫn đi qua `Step::SplitChapters` — một mẫu phân tách THẬT sự cho N > 1 Chương.
/// ĐO ĐƯỢC (không suy luận, xem mục nợ mới trong `deferred-work.md`): ảnh đứng SAU ranh giới
/// phân tách (thuộc Chương thứ hai) vẫn nằm trong `blocks` của Chương ĐẦU (danh sách không bị
/// cắt), nên `compute_anchor` tính trên `source_text` CỦA CHƯƠNG ĐẦU (đã bị cắt cụt tại ranh
/// giới) và TỰ KIỂM bắt được độ lệch — ảnh đó TRƯỢT (không lưu vào Chương nào), KHÔNG bị lưu
/// SAI vào Chương đầu.
#[test]
fn an_image_after_a_chapter_split_boundary_fails_distinguishably_instead_of_being_saved_to_the_wrong_chapter() {
    let source_dir = temp_dir("split-boundary-src");
    let root = temp_dir("split-boundary-root");
    let path = write_docx(
        &source_dir,
        "hai_chuong.docx",
        &fixtures_docx::images_on_both_sides_of_a_chapter_split_boundary(),
    );

    let (shape, docx_sidecar) = import_file(&path).expect("nhap .docx hai chuong phai thanh cong");
    let pattern = ChapterPattern::regex(r"^Chuong \d+:.*$");
    let domain_log_state: DomainLogState = std::sync::Mutex::new(Vec::new());

    let opened = create_work(
        &root,
        "Docx Hai Chuong",
        "en",
        "",
        shape,
        encoding_rs::UTF_8,
        Vec::new(),
        Some(pattern),
        Vec::new(), &[],
        &domain_log_state,
        docx_sidecar)
    .expect("tao Tac pham tu .docx hai chuong that bai");

    assert_eq!(opened.images_saved, 1, "chi anh DUNG TRUOC ranh gioi (Chuong dau) duoc luu");
    assert_eq!(opened.images_failed, 1, "anh SAU ranh gioi phai troi VA duoc dem, khong bi bo qua im lang");

    let rows = read_asset_rows(&opened.store);
    assert_eq!(rows.len(), 1, "dung mot hang asset duoc ghi: {rows:?}");
    assert_eq!(
        rows[0].0, opened.chapter_id,
        "hang asset con lai phai thuoc Chuong DAU TIEN (opened.chapter_id) -- anh sau ranh gioi \
         KHONG duoc xuat hien o day (no da truot, khong duoc luu nham vao Chuong nay)"
    );
}
