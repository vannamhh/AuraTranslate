//! Bảy fixture `.docx` cho bộ test của Story 6.12, SINH TẠI LÚC CHẠY — không commit blob nhị
//! phân (§Always spec 6.12). Năm fixture thường (`plain`/`table_two_columns`/
//! `table_one_row_multi_paragraph`/`image_png`/`empty`) dùng `docx-rs` — bộ GHI của
//! `core::export` (AD-38), một cài đặt ĐỘC LẬP với `core::docx` (bộ ĐỌC của story này), nên
//! sinh-rồi-đọc-lại ở đây KHÔNG phải một vòng tròn (§Never spec 6.12: "không sinh fixture bằng
//! chính bộ đọc của ta rồi đọc lại"). Hai fixture hỏng (`truncated`/`not_a_zip`) dựng tay bằng
//! byte thô, không qua `docx-rs`.
//!
//! **Helper DÙNG CHUNG** — tệp này là một target test ĐỘC LẬP (0 `#[test]`, `cargo test` báo
//! "0 tests" cho binary này, đúng khuôn một tệp hạ tầng thuần) LẪN một MODULE mà
//! `docx_contract.rs` nạp qua `#[path = "fixtures_docx.rs"] mod fixtures_docx;` — cùng khuôn
//! duy nhất Rust cho phép chia sẻ mã giữa hai binary test độc lập mà không dựng một crate phụ.
//!
//! ─────────────────────────────────────────────────────────────────────────────
//! CA "Ô NHIỀU CÂU" NẰM TRONG `table_two_columns`, KHÔNG PHẢI MỘT FIXTURE THỨ TÁM
//! ─────────────────────────────────────────────────────────────────────────────
//! Ma trận I/O spec 6.12 có một hàng riêng "Ô nhiều câu" (một ô chứa HAI CÂU trong CÙNG một
//! `<w:p>`) — [`table_two_columns`] đặt đúng ca đó vào ô `(0, 0)` để bảy fixture (không tám)
//! vẫn phủ đủ Ma trận: đếm bảng (AD-38) VÀ ranh giới ô = ranh giới đoạn (đóng nợ `:2214`) kiểm
//! được trên CÙNG một fixture.

use docx_rs::{Docx, Paragraph, Pic, Run, Table, TableCell, TableRow};

/// PNG 1×1 điểm ảnh hợp lệ THẬT (không chỉ byte rác) — 69 byte, khuôn tối thiểu mà mọi trình
/// đọc PNG chấp nhận. `Pic::new_with_dimensions` không kiểm định dạng (đọc nguồn `docx-rs` đã
/// tải: "For now only PNG is supported" — nó LUÔN ghi đuôi `.png`), nhưng dùng byte PNG THẬT
/// (thay vì byte rác) làm fixture trung thực hơn nếu một lượt sau đọc thử bằng một trình xem
/// ảnh THẬT.
const TINY_PNG: [u8; 69] = [
    0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, 0x00, 0x00, 0x00, 0x0D, 0x49, 0x48, 0x44,
    0x52, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x08, 0x02, 0x00, 0x00, 0x00, 0x90,
    0x77, 0x53, 0xDE, 0x00, 0x00, 0x00, 0x0C, 0x49, 0x44, 0x41, 0x54, 0x08, 0xD7, 0x63, 0xF8,
    0xCF, 0xC0, 0x00, 0x00, 0x03, 0x01, 0x01, 0x00, 0x18, 0xDD, 0x8D, 0xB0, 0x00, 0x00, 0x00,
    0x00, 0x49, 0x45, 0x4E, 0x44, 0xAE, 0x42, 0x60, 0x82,
];

fn text_paragraph(text: &str) -> Paragraph {
    Paragraph::new().add_run(Run::new().add_text(text))
}

/// Đóng gói `Docx` thành byte trong bộ nhớ — không tệp tạm nào trên đĩa.
fn pack(docx: Docx) -> Vec<u8> {
    let mut buf = std::io::Cursor::new(Vec::new());
    docx.build().pack(&mut buf).expect("dong goi .docx fixture that bai -- loi ha tang cua ban do, khong phai mot phep do");
    buf.into_inner()
}

/// **`plain`** — hai đoạn văn bản thường, KHÔNG bảng, KHÔNG ảnh. Ma trận I/O hàng "Văn bản
/// thường".
pub fn plain() -> Vec<u8> {
    pack(
        Docx::new()
            .add_paragraph(text_paragraph("Doan mot khong bang khong anh."))
            .add_paragraph(text_paragraph("Doan hai ket thuc cau nay.")),
    )
}

/// **`table_two_columns`** — bảng 2 hàng × 2 cột. Ô `(0, 0)` mang HAI CÂU trong CÙNG một
/// đoạn (ca "Ô nhiều câu" của Ma trận I/O — xem doc-comment đầu tệp); ba ô còn lại mang một
/// câu đơn để phép đếm (rows=2, cells_per_row=[2,2], paragraphs_per_cell=[[1,1],[1,1]]) không
/// lẫn với số câu.
pub fn table_two_columns() -> Vec<u8> {
    let cell_two_sentences = TableCell::new()
        .add_paragraph(text_paragraph("Cau mot that day du trong o nay. Cau hai tiep tuc luon trong cung mot doan."));
    // ⚠️ BA ô còn lại KHÔNG kết bằng dấu kết câu, cố ý — một ô bảng thật (tên, số liệu, nhãn
    // ngắn) thường KHÔNG mang dấu chấm cuối. Nếu ranh giới ô # ranh giới đoạn (ví dụ nối các ô
    // bằng một khoảng trắng thay vì `"\n\n"`), bộ tách câu sẽ KHÔNG có dấu hiệu nào để dừng và
    // nối chữ cuối ô này với chữ đầu ô kế tiếp thành MỘT segment — đây là điều kiện khiến
    // `no_segment_crosses_a_table_cell_boundary_after_writing_the_whole_document` THẬT SỰ đo
    // được ranh giới đoạn, không chỉ đo lại ranh giới CÂU (mà một cách nối phẳng bằng khoảng
    // trắng vẫn giữ đúng nhờ dấu chấm — xem Đối chứng đỏ ② §Verification spec 6.12).
    let cell_a2 = TableCell::new().add_paragraph(text_paragraph("O hang mot cot hai"));
    let cell_b1 = TableCell::new().add_paragraph(text_paragraph("O hang hai cot mot"));
    let cell_b2 = TableCell::new().add_paragraph(text_paragraph("O hang hai cot hai"));

    let table = Table::new(vec![
        TableRow::new(vec![cell_two_sentences, cell_a2]),
        TableRow::new(vec![cell_b1, cell_b2]),
    ]);

    pack(Docx::new().add_table(table))
}

/// **`table_one_row_multi_paragraph`** — MỘT hàng, hai ô; ô `(0, 0)` mang HAI đoạn RIÊNG
/// (hai `<w:p>` khác nhau, không phải một đoạn hai câu), ô `(0, 1)` mang một đoạn. Ca AD-38
/// mà spec đặt tên ("bảng một hàng ô nhiều đoạn") — story này chỉ CẤP năng lực đếm
/// (`paragraphs_per_cell == [[2, 1]]`), KHÔNG cài cổng chặn (Story 8.8).
pub fn table_one_row_multi_paragraph() -> Vec<u8> {
    let cell_two_paragraphs = TableCell::new()
        .add_paragraph(text_paragraph("Doan mot cua o nhieu doan."))
        .add_paragraph(text_paragraph("Doan hai cua cung o do."));
    let cell_one_paragraph = TableCell::new().add_paragraph(text_paragraph("O con lai chi mot doan."));

    let table = Table::new(vec![TableRow::new(vec![cell_two_paragraphs, cell_one_paragraph])]);

    pack(Docx::new().add_table(table))
}

/// **`image_png`** — một đoạn TRƯỚC ảnh, một ảnh nhúng, một đoạn SAU ảnh — đủ để
/// `core::segment::anchor::compute_anchor` có cả tiền tố lẫn hậu tố THẬT để kiểm.
pub fn image_png() -> Vec<u8> {
    let pic = Pic::new_with_dimensions(TINY_PNG.to_vec(), 1, 1);
    pack(
        Docx::new()
            .add_paragraph(text_paragraph("Doan truoc anh."))
            .add_paragraph(Paragraph::new().add_run(Run::new().add_image(pic)))
            .add_paragraph(text_paragraph("Doan sau anh.")),
    )
}

/// **`empty`** — 0 đoạn có chữ (một đoạn RỖNG duy nhất, không `Run`/`w:t` nào). Ma trận I/O
/// hàng "Rỗng".
pub fn empty() -> Vec<u8> {
    pack(Docx::new().add_paragraph(Paragraph::new()))
}

/// **`truncated`** — nửa ĐẦU của [`plain`] (một tệp `.docx` HỢP LỆ bị cắt cụt giữa chừng, mô
/// phỏng một lượt tải/sao chép dở dang). Ma trận I/O hàng "Tệp hỏng".
pub fn truncated() -> Vec<u8> {
    let full = plain();
    full[..full.len() / 2].to_vec()
}

/// **`not_a_zip`** — byte văn bản thường, hoàn toàn KHÔNG phải một kho zip. Ma trận I/O hàng
/// "Đuôi `.docx`, không phải zip" — [`crate::core::docx::read_docx`] phải từ chối theo NỘI
/// DUNG (không mở được như zip), không theo tên tệp (tệp này không hề có một cái tên ở đây).
pub fn not_a_zip() -> Vec<u8> {
    b"Day chi la mot tep .txt doi duoi thanh .docx, khong phai zip.".to_vec()
}

/// **`image_unsupported_mime`** — ảnh nhúng có đuôi tệp media NGOÀI danh mục ĐÓNG bốn MIME
/// (`.bmp`), kẹp giữa hai đoạn chữ. Phủ vế XỬ LÝ LỖI của hàng "Ảnh nhúng" trong Ma trận I/O
/// spec 6.12 — *"MIME ngoài danh mục 4 ⇒ bỏ ảnh, ghi `images_failed`, Chương **vẫn** nhập"* —
/// vế mà bảy fixture kia KHÔNG chạm tới được.
///
/// 🔴 **Dựng TAY, không qua `docx-rs`, và đó là điều kiện bắt buộc chứ không phải một sở
/// thích:** `Pic::new_with_dimensions` của `docx-rs` LUÔN ghi đuôi `.png` (đọc nguồn đã tải —
/// *"For now only PNG is supported"*), nên không có cách nào bảo nó sinh ra một media `.bmp`.
/// Một fixture cho ca NGOÀI danh mục vì thế phải tự dựng zip.
///
/// Gói này chỉ mang ba mục mà [`auratranslate_lib::core::docx::read_docx`] thật sự đọc
/// (`word/document.xml`, `word/_rels/document.xml.rels`, `word/media/image1.bmp`) — cố ý
/// KHÔNG kèm `[Content_Types].xml`/`_rels/.rels`: bộ đọc của ta không mở hai tệp đó, và một
/// fixture chở thêm thứ không ai đọc chỉ làm mờ điều đang được kiểm.
pub fn image_unsupported_mime() -> Vec<u8> {
    let document_xml = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"
            xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main"
            xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships">
  <w:body>
    <w:p><w:r><w:t>Doan truoc anh bmp.</w:t></w:r></w:p>
    <w:p><w:r><w:drawing><a:graphic><a:graphicData><a:blip r:embed="rId9"/></a:graphicData></a:graphic></w:drawing></w:r></w:p>
    <w:p><w:r><w:t>Doan sau anh bmp.</w:t></w:r></w:p>
  </w:body>
</w:document>"#;

    let rels_xml = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
  <Relationship Id="rId9" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/image" Target="media/image1.bmp"/>
</Relationships>"#;

    // Đầu tệp BMP hợp lệ tối thiểu (`BM` + kích thước) — nội dung ảnh không quan trọng ở đây:
    // ca này đo CỔNG MIME, và cổng đó quyết định bằng ĐUÔI tệp media, không mở byte ra xem.
    let bmp_bytes: Vec<u8> = vec![0x42, 0x4D, 0x1E, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00];

    pack_zip_entries(&[
        ("word/document.xml", document_xml.as_bytes()),
        ("word/_rels/document.xml.rels", rels_xml.as_bytes()),
        ("word/media/image1.bmp", bmp_bytes.as_slice()),
    ])
}

/// Đóng gói một danh sách mục (đường dẫn, byte) thành một zip tối giản — helper DÙNG CHUNG
/// cho mọi fixture dựng tay cần nhiều hơn hai mục. Chỉ liệt đúng những mục mà
/// [`auratranslate_lib::core::docx::read_docx`] thật sự đọc — cố ý KHÔNG kèm
/// `[Content_Types].xml`/`_rels/.rels` (bộ đọc không mở hai tệp đó, xem doc-comment
/// `image_unsupported_mime`).
fn pack_zip_entries(entries: &[(&str, &[u8])]) -> Vec<u8> {
    let mut buf = std::io::Cursor::new(Vec::new());
    {
        let mut zip = zip::ZipWriter::new(&mut buf);
        let options: zip::write::SimpleFileOptions = Default::default();
        for (name, bytes) in entries {
            zip.start_file(*name, options)
                .unwrap_or_else(|e| panic!("mo muc zip {name} that bai -- loi ha tang cua fixture: {e}"));
            std::io::Write::write_all(&mut zip, bytes)
                .unwrap_or_else(|e| panic!("ghi muc zip {name} that bai -- loi ha tang cua fixture: {e}"));
        }
        zip.finish().expect("dong goi zip fixture that bai -- loi ha tang cua fixture");
    }
    buf.into_inner()
}

/// **`image_wrapped_in_alternate_content`** — MỘT ảnh gói thành `mc:AlternateContent` với HAI
/// nhánh cùng trỏ một rId: `mc:Choice > w:drawing` (DrawingML, hình dạng Word HIỆN ĐẠI sinh
/// ra) VÀ `mc:Fallback > w:pict` (VML, dự phòng cho Word cũ) — khuôn THẬT mà Word áp cho MỌI
/// ảnh nó nhúng, không phải một ca hiếm. Dựng TAY (`docx-rs` không tự sinh
/// `mc:AlternateContent`) — đúng lý do `image_unsupported_mime` đã dựng tay.
///
/// Đối chứng cho mệnh đề *"đúng MỘT khối ảnh"*: nếu `mc:Fallback` không được nhảy trọn
/// subtree, phép duyệt phẳng thấy CẢ `w:drawing` LẪN `w:pict` cho đúng một ảnh, sinh HAI khối.
pub fn image_wrapped_in_alternate_content() -> Vec<u8> {
    let document_xml = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"
            xmlns:mc="http://schemas.openxmlformats.org/markup-compatibility/2006"
            xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships"
            xmlns:a="http://schemas.openxmlformats.org/drawingml/2006/main"
            xmlns:v="urn:schemas-microsoft-com:vml">
  <w:body>
    <w:p><w:r><w:t>Truoc anh goi AlternateContent.</w:t></w:r></w:p>
    <w:p><w:r>
      <mc:AlternateContent>
        <mc:Choice Requires="wpg">
          <w:drawing><a:graphic><a:graphicData><a:blip r:embed="rId1"/></a:graphicData></a:graphic></w:drawing>
        </mc:Choice>
        <mc:Fallback>
          <w:pict><v:shape><v:imagedata r:id="rId1"/></v:shape></w:pict>
        </mc:Fallback>
      </mc:AlternateContent>
    </w:r></w:p>
    <w:p><w:r><w:t>Sau anh goi AlternateContent.</w:t></w:r></w:p>
  </w:body>
</w:document>"#;

    let rels_xml = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
  <Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/image" Target="media/image1.png"/>
</Relationships>"#;

    pack_zip_entries(&[
        ("word/document.xml", document_xml.as_bytes()),
        ("word/_rels/document.xml.rels", rels_xml.as_bytes()),
        ("word/media/image1.png", &TINY_PNG),
    ])
}

/// **`image_vml_only`** — MỘT ảnh chỉ mang `w:pict`/`v:imagedata` (VML), KHÔNG `w:drawing` nào
/// trong toàn tài liệu — đúng nhánh mà [`find_image_rel_id`]'s VML case (`core/docx/mod.rs`)
/// trước lượt sửa này CHƯA từng chạy qua một ca test nào. Dựng TAY, cùng lý do
/// `image_wrapped_in_alternate_content`.
pub fn image_vml_only() -> Vec<u8> {
    let document_xml = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"
            xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships"
            xmlns:v="urn:schemas-microsoft-com:vml">
  <w:body>
    <w:p><w:r><w:t>Truoc anh VML.</w:t></w:r></w:p>
    <w:p><w:r><w:pict><v:shape><v:imagedata r:id="rId1"/></v:shape></w:pict></w:r></w:p>
    <w:p><w:r><w:t>Sau anh VML.</w:t></w:r></w:p>
  </w:body>
</w:document>"#;

    let rels_xml = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
  <Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/image" Target="media/image1.png"/>
</Relationships>"#;

    pack_zip_entries(&[
        ("word/document.xml", document_xml.as_bytes()),
        ("word/_rels/document.xml.rels", rels_xml.as_bytes()),
        ("word/media/image1.png", &TINY_PNG),
    ])
}

/// **`images_on_both_sides_of_a_chapter_split_boundary`** — hai "Chương" (đánh dấu bằng dòng
/// khớp mẫu phân tách `"Chuong N: ..."`), MỖI Chương mang một ảnh: một ảnh TRƯỚC ranh giới
/// (thuộc Chương 1), một ảnh SAU ranh giới (thuộc Chương 2). Đối chứng cho giới hạn
/// `DocxSidecar` — "chỉ Chương đầu tiên đọc `blocks`" — khi mẫu phân tách THẬT SỰ cho N > 1
/// Chương trên đường sản phẩm.
pub fn images_on_both_sides_of_a_chapter_split_boundary() -> Vec<u8> {
    let pic1 = Pic::new_with_dimensions(TINY_PNG.to_vec(), 1, 1);
    let pic2 = Pic::new_with_dimensions(TINY_PNG.to_vec(), 1, 1);
    pack(
        Docx::new()
            .add_paragraph(text_paragraph("Chuong 1: Mo Dau"))
            .add_paragraph(text_paragraph("Anh dau tien o day."))
            .add_paragraph(Paragraph::new().add_run(Run::new().add_image(pic1)))
            .add_paragraph(text_paragraph("Chuong 2: Tiep Theo"))
            .add_paragraph(text_paragraph("Anh thu hai o day."))
            .add_paragraph(Paragraph::new().add_run(Run::new().add_image(pic2))),
    )
}

/// **`table_with_nested_table_and_trailing_paragraph`** — bảng NGOÀI một hàng một ô; ô đó
/// mang một đoạn TRƯỚC, một bảng LỒNG (một hàng một ô, một đoạn riêng của chính nó), rồi một
/// đoạn SAU — cùng trong ô ngoài. Đối chứng cho mệnh đề *"nhảy trọn `w:tbl` lồng rồi đi tiếp,
/// không bỏ luôn phần còn lại của ô/hàng SAU lượt nhảy"* (`core/docx/mod.rs::parse_cell`).
pub fn table_with_nested_table_and_trailing_paragraph() -> Vec<u8> {
    let nested_table = Table::new(vec![TableRow::new(vec![
        TableCell::new().add_paragraph(text_paragraph("Doan trong bang long.")),
    ])]);
    let outer_cell = TableCell::new()
        .add_paragraph(text_paragraph("Doan truoc bang long."))
        .add_table(nested_table)
        .add_paragraph(text_paragraph("Doan sau bang long."));
    let outer_table = Table::new(vec![TableRow::new(vec![outer_cell])]);

    pack(Docx::new().add_table(outer_table))
}
