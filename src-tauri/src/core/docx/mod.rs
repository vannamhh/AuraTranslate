//! Đọc OOXML `.docx` bằng `zip` + `quick-xml` — Story 6.12 (AD-38/AD-39).
//!
//! ─────────────────────────────────────────────────────────────────────────────
//! 🔴 VÌ SAO MÔ-ĐUN NÀY TỰ ĐỌC, KHÔNG GỌI `docx_rs::read_docx` (Quyết định Ice 2026-09-09)
//! ─────────────────────────────────────────────────────────────────────────────
//! Đo 2026-09-09 trên `docx-rs 0.4.22` đã tải: **140 điểm `unwrap()`/`expect()`/`panic!`/
//! `unreachable!()`** trong mã sản phẩm của `src/reader/` (đã cắt `#[cfg(test)]`), trên
//! **63/77 tệp** — ví dụ `read_zip.rs:20` `xml.read_to_end(&mut data).unwrap()`: một luồng
//! deflate cắt cụt là **panic**, không `Err`. `Cargo.toml:167` của crate này khai
//! `panic = "abort"` cho bản phát hành ⇒ một `.docx` hỏng giết CẢ TIẾN TRÌNH, không phải một
//! lỗi hiện ra được. `docx-rs` vẫn ở lại làm bộ GHI của `core::export` (Epic 8, AD-38) và làm
//! bộ SINH FIXTURE cho bộ test của story này — hai vai đó không đòi 0 điểm panic.
//!
//! 🔴 **Mọi hàm trong mô-đun này trả `Result`, 0 điểm panic — đây là cổng có test
//! (`tests/docx_boundary.rs`), không phải một lời hứa trong doc-comment.**
//!
//! ─────────────────────────────────────────────────────────────────────────────
//! CƠ CHẾ
//! ─────────────────────────────────────────────────────────────────────────────
//! 1. Mở `bytes` bằng `zip::ZipArchive` — không phải zip hợp lệ (kể cả một tệp `.txt` đổi
//!    đuôi thành `.docx`, hay một zip cắt cụt) ⇒ [`DocxError::NotAZip`].
//! 2. Đọc `word/document.xml` (đoạn/bảng) và `word/_rels/document.xml.rels` (rId → tệp
//!    ảnh trong `word/media/`) — thiếu/hỏng ⇒ [`DocxError::MissingEntry`]/[`DocxError::MalformedXml`].
//! 3. Duyệt cây XML bằng một bộ đọc SỰ KIỆN thuần (`quick_xml::Reader::from_str`), theo LOCAL
//!    NAME (bỏ tiền tố `w:`/`a:`/`r:`/…) — KHÔNG phân giải namespace URI đầy đủ. ⚠️ Đây là một
//!    ĐƠN GIẢN HOÁ có chủ: mọi `.docx` thật (Word lẫn `docx-rs`) đều dùng đúng bốn tiền tố
//!    chuẩn OOXML, và Word không cho người dùng đổi chúng — rủi ro một namespace KHÁC dùng
//!    trùng tên cục bộ (`t`, `p`, `tbl`, …) bên trong `word/document.xml` là lý thuyết, không
//!    quan sát được. Nợ ghi ở `deferred-work.md` nếu Ice cần siết chặt bằng URI thật.
//! 4. Đoạn (`w:p`) là đơn vị VĂN BẢN; ô bảng (`w:tc`) là **danh sách đoạn RIÊNG** — mỗi đoạn
//!    (kể cả đoạn nằm trong ô) trở thành MỘT [`crate::core::webimport::BlockBody::Paragraph`]
//!    ĐỘC LẬP, ngăn cách bởi `"\n\n"` (khuôn `exact_gap_before` mà
//!    [`crate::core::segment::pipeline::join_kept_blocks`] đã đọc cho Story 6.7/6.9) — đóng nợ
//!    `:2214` ("câu bị cắt giữa ô bảng"): ranh giới ô = ranh giới đoạn, nên bộ tách câu
//!    (`core::segment::split`) không bao giờ có cơ hội cắt vắt qua hai ô.
//! 5. Văn bản trả về (`DocxParsed::text`) được dựng bằng CHÍNH [`join_kept_blocks`] trên
//!    [`DocxParsed::blocks`] (mọi khối `machine_kept: true`, không override) — một nguồn sự
//!    thật DUY NHẤT cho cả "văn bản đi vào pipeline" LẪN "khối để tính neo ảnh" (Story 6.11
//!    tái dùng), không hai bản có thể trôi khỏi nhau.
//! 6. `w:tbl` LỒNG bên trong một ô bị bỏ qua có chủ ý (đọc byte để cân bằng cây XML, không
//!    đếm vào cấu trúc bảng của ô cha) — nợ mới, ghi ở `deferred-work.md`, chủ Ice.
//! 7. Ảnh nhúng (`a:blip r:embed="rIdN"`, DrawingML — hình dạng mà Word HIỆN ĐẠI và `docx-rs`
//!    đều sinh ra; VML `v:imagedata` không được đọc, nợ mới) phân giải qua rels rồi đọc byte
//!    thật từ `word/media/…` — MIME suy từ ĐUÔI TỆP media (không phải từ
//!    `[Content_Types].xml`, một đơn giản hoá có chủ: bốn đuôi `png`/`jpg`/`jpeg`/`gif`/`webp`
//!    đã đủ cho danh mục ĐÓNG bốn MIME mà `core::webimport::assets::extension_for_mime` chấp
//!    nhận; SVG/EMF/WMF/BMP/TIFF suy ra một MIME KHÔNG thuộc danh mục đó và bị từ chối ở tầng
//!    gọi (`images_failed`), không panic ở đây).

use std::collections::BTreeMap;
use std::io::{Cursor, Read};

use quick_xml::Reader;
use quick_xml::events::{BytesStart, Event};

use crate::core::webimport::{Block, BlockBody};

/// Mọi cách [`read_docx`] thất bại. `detail`/`name` CHỈ để chẩn đoán/log (KHÔNG DẤU, NFR16) —
/// `commands::segment::import` ánh xạ sang [`crate::core::segment::import::ImportError`], nơi
/// người dùng đọc một câu tiếng Việt qua `vi.json`, không đọc `detail` thô.
#[derive(Debug)]
pub enum DocxError {
    /// Không mở được như một kho zip — tệp cắt cụt, hoặc một tệp KHÁC đổi đuôi thành
    /// `.docx` (Ma trận I/O: "nhận theo NỘI DUNG, không theo tên").
    NotAZip { detail: String },
    /// Zip mở được nhưng thiếu một mục bắt buộc (`word/document.xml`).
    MissingEntry { name: String },
    /// Một mục đọc được nhưng không phải UTF-8, hoặc không phải XML hợp lệ theo cấu trúc mà
    /// bộ đọc này mong đợi (thẻ mở không có thẻ đóng khớp, EOF giữa một phần tử, …).
    MalformedXml { detail: String },
    /// Đọc xong nhưng **0 đoạn có chữ** — Ma trận I/O hàng "Rỗng".
    EmptyText,
}

impl std::fmt::Display for DocxError {
    /// KHÔNG DẤU — chẩn đoán cho log (NFR16).
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DocxError::NotAZip { detail } => write!(f, "docx: khong mo duoc nhu mot kho zip: {detail}"),
            DocxError::MissingEntry { name } => write!(f, "docx: thieu muc bat buoc {name:?}"),
            DocxError::MalformedXml { detail } => write!(f, "docx: xml khong hop le: {detail}"),
            DocxError::EmptyText => write!(f, "docx: 0 doan co chu sau khi doc"),
        }
    }
}

impl std::error::Error for DocxError {}

/// Một ảnh nhúng đã đọc byte thật — `block_index` là chỉ số của khối [`BlockBody::Image`]
/// tương ứng trong [`DocxParsed::blocks`] (dùng bởi `commands::project::prepare_chapter_images`
/// để nối đúng ảnh với đúng vị trí, tái dùng máy Story 6.11 — xem doc-comment đầu module).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DocxImage {
    pub block_index: usize,
    pub bytes: Vec<u8>,
    /// MIME suy từ đuôi tệp media — KHÔNG BẢO ĐẢM thuộc danh mục ĐÓNG bốn kiểu mà
    /// `core::webimport::assets::extension_for_mime` chấp nhận; tầng gọi tự quyết định
    /// giữ/bỏ, module này không lọc.
    pub content_type: String,
}

/// Cấu trúc ĐẾM của một bảng (`w:tbl` cấp CAO NHẤT — không tính `w:tbl` lồng trong ô, xem
/// doc-comment đầu module mục 6) — năng lực AD-38 mà story này CẤP, không CÀI cổng chặn (đó
/// là Story 8.8).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TableShape {
    pub rows: usize,
    /// Số ô của MỖI hàng, theo thứ tự hàng.
    pub cells_per_row: Vec<usize>,
    /// Số đoạn (`w:p`, ĐẾM THÔ — kể cả đoạn rỗng) của MỖI ô, `[hàng][cột]`.
    pub paragraphs_per_cell: Vec<Vec<usize>>,
}

/// Kết quả đọc trọn một `.docx`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DocxParsed {
    /// Văn bản đã ghép theo đoạn — dựng bằng CHÍNH [`join_kept_blocks`] trên `blocks` (xem
    /// doc-comment đầu module mục 5). Đây là giá trị đi vào
    /// `ChapterInput::AlreadyText`.
    pub text: String,
    /// Đoạn + ảnh, cùng hình dạng [`Block`] mà `core::webimport::Extractor` dùng cho HTML —
    /// tái dùng máy tính neo ảnh của Story 6.11 (`core::segment::anchor::compute_anchor`).
    pub blocks: Vec<Block>,
    /// Cấu trúc đếm bảng cho AD-38 — một phần tử cho mỗi `w:tbl` cấp cao nhất, theo thứ tự
    /// tài liệu.
    pub tables: Vec<TableShape>,
    /// Byte thật của mọi ảnh nhúng phân giải được — xem [`DocxImage`].
    pub images: Vec<DocxImage>,
}

/// Đọc trọn một `.docx` từ byte trong bộ nhớ — hàm THUẦN, không chạm đĩa ngoài `bytes` được
/// truyền vào, **`Result` ở mọi nhánh, 0 điểm panic** (đây là điều kiện mà Quyết định 3 của
/// spec 6.12 dựa lên — xem `tests/docx_boundary.rs`).
pub fn read_docx(bytes: &[u8]) -> Result<DocxParsed, DocxError> {
    let cursor = Cursor::new(bytes);
    let mut archive = zip::ZipArchive::new(cursor).map_err(|e| DocxError::NotAZip { detail: e.to_string() })?;

    let document_xml_bytes = read_zip_entry(&mut archive, "word/document.xml")
        .ok_or_else(|| DocxError::MissingEntry { name: "word/document.xml".to_owned() })?;
    let document_xml = String::from_utf8(document_xml_bytes)
        .map_err(|e| DocxError::MalformedXml { detail: format!("word/document.xml khong phai UTF-8: {e}") })?;

    // Rels là TUỲ CHỌN theo cơ chế đọc này — thiếu nó chỉ làm MỌI ảnh không phân giải được
    // (đếm vào `images_failed` ở tầng gọi), không làm cả lượt đọc trượt: một `.docx` không
    // ảnh vẫn hợp lệ mà không cần tệp rels này tồn tại đúng khuôn OOXML.
    let rels_map: BTreeMap<String, String> = match read_zip_entry(&mut archive, "word/_rels/document.xml.rels") {
        Some(raw) => match String::from_utf8(raw) {
            Ok(text) => parse_rels(&text)?,
            // 🔴 SỬA (bắt được sau code review) — bản trước `unwrap_or_default()` NUỐT IM
            // LẶNG: một tệp rels không phải UTF-8 cho map RỖNG mà không một dòng nào nói vì
            // sao, nên MỌI ảnh của tài liệu mất ánh xạ rId mà không ai biết lý do thật. Vẫn
            // KHÔNG làm trượt cả lượt đọc (rels là tuỳ chọn, xem doc-comment ngay trên) —
            // chỉ thêm một dòng chẩn đoán, đúng khuôn `eprintln!` đã dùng trong tệp này.
            Err(e) => {
                // ⚠️ KHÔNG viết tiền tố `"docx[rels] ..."` (khuôn `"asset[mime] ..."` của
                // `commands::project`) — chuỗi đó khớp NHẦM vị từ chỉ số mảng trần của
                // `docx_boundary.rs::line_indexes_a_slice_raw` (nó chỉ nhìn văn bản, không
                // phân biệt được một chuỗi hiển thị khỏi mã thật) và báo đỏ oan cổng 0-điểm-
                // panic. Viết bằng dấu hai chấm để không tạo hình dạng `<định danh>[...]`.
                eprintln!(
                    "docx rels khong phai UTF-8 (word/_rels/document.xml.rels), bo qua anh xa \
                     anh: moi anh cua tai lieu se mat ri -- {e}"
                );
                BTreeMap::new()
            }
        },
        None => BTreeMap::new(),
    };

    let items = parse_body(&document_xml)?;

    let mut blocks: Vec<Block> = Vec::new();
    let mut images: Vec<DocxImage> = Vec::new();
    let mut tables: Vec<TableShape> = Vec::new();
    let mut first_text_seen = false;

    for item in &items {
        match item {
            BodyItem::Para(p) => {
                push_paragraph(p, &mut blocks, &mut images, &mut first_text_seen, &rels_map, &mut archive);
            }
            BodyItem::Table(t) => {
                let mut cells_per_row = Vec::with_capacity(t.rows.len());
                let mut paragraphs_per_cell = Vec::with_capacity(t.rows.len());
                for row in &t.rows {
                    cells_per_row.push(row.cells.len());
                    let mut per_cell = Vec::with_capacity(row.cells.len());
                    for cell in &row.cells {
                        per_cell.push(cell.paragraphs.len());
                        for p in &cell.paragraphs {
                            push_paragraph(p, &mut blocks, &mut images, &mut first_text_seen, &rels_map, &mut archive);
                        }
                    }
                    paragraphs_per_cell.push(per_cell);
                }
                tables.push(TableShape { rows: t.rows.len(), cells_per_row, paragraphs_per_cell });
            }
        }
    }

    // Dựng văn bản bằng CHÍNH hàm mà pipeline dùng để ghép khối GIỮ — một nguồn sự thật, xem
    // doc-comment đầu module mục 5. Mọi khối ở đây `machine_kept: true`, không override nào
    // (`.docx` không có tầng 2 sửa tay) ⇒ `effective_kept` = toàn `true`.
    let all_kept = vec![true; blocks.len()];
    let text = crate::core::segment::pipeline::join_kept_blocks(&blocks, &all_kept);

    if text.trim().is_empty() {
        return Err(DocxError::EmptyText);
    }

    Ok(DocxParsed { text, blocks, tables, images })
}

// ═════════════════════════════════════════════════════════════════════════════════
// Đọc zip — chịu được đường mang `\` (tệp nén trên Windows), xem doc-comment code map spec
// ═════════════════════════════════════════════════════════════════════════════════

fn read_zip_entry<R: Read + std::io::Seek>(archive: &mut zip::ZipArchive<R>, target: &str) -> Option<Vec<u8>> {
    let normalized_target = target.replace('\\', "/");
    for i in 0..archive.len() {
        // 🔴 SỬA (bắt được sau code review) — bản trước dùng `.ok()?`, nên MỘT mục bất kỳ
        // (bất kỳ chỉ số nào, không riêng `target`) mở lỗi làm THOÁT TRỌN hàm này qua `?` —
        // một `word/document.xml` lành lặn bị báo `MissingEntry` chỉ vì một mục KHÁC (ví dụ
        // một `word/media/*` hỏng) không mở được. Bỏ qua ĐÚNG mục lỗi đó, quét tiếp — chỉ
        // mục `target` mới quyết định kết quả của hàm này.
        let Ok(mut file) = archive.by_index(i) else { continue };
        let name = file.name().replace('\\', "/");
        if name == normalized_target {
            let mut buf = Vec::new();
            return file.read_to_end(&mut buf).ok().map(|_| buf);
        }
    }
    None
}

// ═════════════════════════════════════════════════════════════════════════════════
// Rels — rId -> Target ("media/imageN.ext", tương đối với thư mục "word/")
// ═════════════════════════════════════════════════════════════════════════════════

fn xml_err(e: quick_xml::Error) -> DocxError {
    DocxError::MalformedXml { detail: e.to_string() }
}

/// Trả về SỞ HỮU (không mượn) — `BytesStart::local_name()` trả một `LocalName<'_>` buộc vào
/// đời sống của THAM CHIẾU `&self`, không vào đời sống dữ liệu `'a` của chính `BytesStart` (đo
/// bằng trình biên dịch: một chữ ký mượn ở đây không qua được `E0515`) — một chuỗi nhỏ SỞ HỮU
/// là lối thoát đơn giản nhất, không đáng để đổi cả cây gọi sang kiểu mượn phức tạp hơn cho
/// một tên thẻ (vài ký tự).
fn local_name_of(e: &BytesStart) -> String {
    std::str::from_utf8(e.local_name().as_ref()).unwrap_or("").to_owned()
}

fn attr_value(e: &BytesStart, local_name: &str) -> Option<String> {
    for attr in e.attributes().flatten() {
        if attr.key.local_name().as_ref() == local_name.as_bytes() {
            // `unescape_value()` bị `#[cfg(not(feature = "encoding"))]` khoá — `docx-rs` bật
            // feature `encoding` của chính `quick-xml` (hợp nhất feature toàn cây, xem
            // Cargo.toml), nên `normalized_value` (thay thế được khuyến nghị) là hàm CÒN LẠI
            // không phụ thuộc feature đó.
            if let Ok(v) = attr.normalized_value(quick_xml::XmlVersion::Implicit1_0) {
                return Some(v.into_owned());
            }
        }
    }
    None
}

fn parse_rels(xml: &str) -> Result<BTreeMap<String, String>, DocxError> {
    let mut reader = Reader::from_str(xml);
    reader.config_mut().trim_text(true);
    let mut map = BTreeMap::new();
    loop {
        match reader.read_event().map_err(xml_err)? {
            Event::Eof => break,
            Event::Start(e) | Event::Empty(e) => {
                if local_name_of(&e) == "Relationship" {
                    if let (Some(id), Some(target)) = (attr_value(&e, "Id"), attr_value(&e, "Target")) {
                        map.insert(id, target);
                    }
                }
            }
            _ => {}
        }
    }
    Ok(map)
}

/// `target` của một rels đích ảnh — tương đối với thư mục `word/` trừ khi bắt đầu bằng `/`
/// (tuyệt đối trong gói OOXML, hiếm nhưng hợp lệ theo đặc tả).
fn normalize_media_path(target: &str) -> String {
    if let Some(stripped) = target.strip_prefix('/') {
        stripped.to_owned()
    } else {
        format!("word/{target}")
    }
}

/// MIME suy từ đuôi tệp media — xem doc-comment đầu module mục 7 cho lý do và giới hạn.
fn mime_for_media_extension(ext: &str) -> String {
    match ext.to_ascii_lowercase().as_str() {
        "png" => "image/png".to_owned(),
        "jpg" | "jpeg" => "image/jpeg".to_owned(),
        "gif" => "image/gif".to_owned(),
        "webp" => "image/webp".to_owned(),
        // Đuôi ngoài bốn kiểu đóng (bmp/emf/wmf/tiff/svg, …) -- KHÔNG một MIME ảnh THẬT, để
        // `core::webimport::assets::extension_for_mime` (điểm quyết định DUY NHẤT của tầng
        // gọi) tự nhiên từ chối, đúng khuôn "MIME ngoài danh mục 4 -> bo anh" của I/O Matrix.
        other => format!("application/octet-stream; ext={other}"),
    }
}

// ═════════════════════════════════════════════════════════════════════════════════
// Cây tài liệu — mô hình tối thiểu đủ cho AD-38 + AD-39 (đoạn, bảng/hàng/ô, ảnh nhúng)
// ═════════════════════════════════════════════════════════════════════════════════

enum ParaPiece {
    Text(String),
    /// rId của ảnh (đã đọc từ `a:blip`/`v:imagedata`).
    Image(String),
}

struct ParaContent {
    pieces: Vec<ParaPiece>,
}

struct CellContent {
    paragraphs: Vec<ParaContent>,
}

struct RowContent {
    cells: Vec<CellContent>,
}

struct TableContent {
    rows: Vec<RowContent>,
}

enum BodyItem {
    Para(ParaContent),
    Table(TableContent),
}

/// Phần tử XML mà không mang chữ VÀ không mang ảnh — an toàn bỏ TRỌN subtree mà không đếm
/// hụt bất kỳ đoạn/ô/ảnh nào. Mọi phần tử KHÁC (kể cả những tên lạ như `w:hyperlink`,
/// `w:ins`/`w:del`, `w:sdt`/`w:sdtContent`, `w:smartTag`) được coi là TRONG SUỐT — Start/End
/// của nó bị bỏ qua nhưng KHÔNG bị nhảy subtree, nên chữ/ảnh bên trong vẫn được thấy ở lượt
/// đọc tiếp theo. Đây là lựa chọn AN TOÀN HƠN một danh sách "trong suốt đã biết": một tên
/// lồng lạ chưa từng nghĩ tới vẫn được duyệt qua đúng, chỉ có phần tử CHẮC CHẮN không mang gì
/// (thuộc tính định dạng) mới bị liệt vào đây.
///
/// 🔴 **`Fallback` (`mc:Fallback`) THÊM có chủ ý — bắt được sau code review.** Word thật gói
/// MỘT ảnh thành `mc:AlternateContent > mc:Choice > w:drawing` (DrawingML, ảnh THẬT) VÀ
/// `mc:Fallback > w:pict` (VML, cùng ảnh, cùng rId — dự phòng cho phiên bản Word không hiểu
/// DrawingML). `mc:Choice` vẫn TRONG SUỐT (để `w:drawing` bên trong được thấy như thường);
/// `mc:Fallback` phải OPAQUE — nếu không, phép duyệt phẳng thấy CẢ `w:drawing` (trong Choice)
/// LẪN `w:pict` (trong Fallback) cho ĐÚNG một ảnh, sinh hai khối `Image`, hai tệp, hai hàng
/// `asset` cho một ảnh duy nhất. Xem `docx_contract.rs::a_word_alternate_content_wrapper_around_one_image_yields_exactly_one_image_block`.
const OPAQUE_LOCAL_NAMES: [&str; 9] =
    ["rPr", "pPr", "tblPr", "tblGrid", "trPr", "tcPr", "sectPr", "tblPrEx", "Fallback"];

fn is_opaque(local: &str) -> bool {
    OPAQUE_LOCAL_NAMES.contains(&local)
}

fn parse_body(xml: &str) -> Result<Vec<BodyItem>, DocxError> {
    let mut reader = Reader::from_str(xml);
    reader.config_mut().trim_text(false);
    let mut items = Vec::new();
    loop {
        match reader.read_event().map_err(xml_err)? {
            Event::Eof => break,
            Event::Start(e) => {
                let local = local_name_of(&e);
                match local.as_str() {
                    "p" => items.push(BodyItem::Para(parse_paragraph(&mut reader)?)),
                    "tbl" => items.push(BodyItem::Table(parse_table(&mut reader)?)),
                    other if is_opaque(other) => {
                        reader.read_to_end(e.name()).map_err(xml_err)?;
                    }
                    // "document"/"body" và mọi vỏ khác -- trong suốt, xem doc-comment
                    // `OPAQUE_LOCAL_NAMES`.
                    _ => {}
                }
            }
            Event::Empty(e) => {
                let local = local_name_of(&e);
                if local == "p" {
                    items.push(BodyItem::Para(ParaContent { pieces: Vec::new() }));
                } else if local == "tbl" {
                    items.push(BodyItem::Table(TableContent { rows: Vec::new() }));
                }
            }
            _ => {}
        }
    }
    Ok(items)
}

fn parse_table(reader: &mut Reader<&[u8]>) -> Result<TableContent, DocxError> {
    let mut rows = Vec::new();
    loop {
        match reader.read_event().map_err(xml_err)? {
            Event::Eof => return Err(DocxError::MalformedXml { detail: "eof giua w:tbl".to_owned() }),
            Event::End(e) if local_name_of_end(&e) == "tbl" => break,
            Event::Start(e) => {
                let local = local_name_of(&e);
                if local == "tr" {
                    rows.push(parse_row(reader)?);
                } else if is_opaque(&local) {
                    reader.read_to_end(e.name()).map_err(xml_err)?;
                }
            }
            Event::Empty(e) if local_name_of(&e) == "tr" => rows.push(RowContent { cells: Vec::new() }),
            _ => {}
        }
    }
    Ok(TableContent { rows })
}

fn parse_row(reader: &mut Reader<&[u8]>) -> Result<RowContent, DocxError> {
    let mut cells = Vec::new();
    loop {
        match reader.read_event().map_err(xml_err)? {
            Event::Eof => return Err(DocxError::MalformedXml { detail: "eof giua w:tr".to_owned() }),
            Event::End(e) if local_name_of_end(&e) == "tr" => break,
            Event::Start(e) => {
                let local = local_name_of(&e);
                if local == "tc" {
                    cells.push(parse_cell(reader)?);
                } else if is_opaque(&local) {
                    reader.read_to_end(e.name()).map_err(xml_err)?;
                }
            }
            Event::Empty(e) if local_name_of(&e) == "tc" => cells.push(CellContent { paragraphs: Vec::new() }),
            _ => {}
        }
    }
    Ok(RowContent { cells })
}

fn parse_cell(reader: &mut Reader<&[u8]>) -> Result<CellContent, DocxError> {
    let mut paragraphs = Vec::new();
    loop {
        match reader.read_event().map_err(xml_err)? {
            Event::Eof => return Err(DocxError::MalformedXml { detail: "eof giua w:tc".to_owned() }),
            Event::End(e) if local_name_of_end(&e) == "tc" => break,
            Event::Start(e) => {
                let local = local_name_of(&e);
                if local == "p" {
                    paragraphs.push(parse_paragraph(reader)?);
                } else if local == "tbl" {
                    // 🔴 Bảng LỒNG trong ô -- đọc byte để cân bằng cây XML, KHÔNG đếm vào cấu
                    // trúc (nợ mới, ghi ở deferred-work.md — xem doc-comment đầu module mục 6).
                    reader.read_to_end(e.name()).map_err(xml_err)?;
                } else if is_opaque(&local) {
                    reader.read_to_end(e.name()).map_err(xml_err)?;
                }
            }
            Event::Empty(e) if local_name_of(&e) == "p" => paragraphs.push(ParaContent { pieces: Vec::new() }),
            _ => {}
        }
    }
    Ok(CellContent { paragraphs })
}

fn parse_paragraph(reader: &mut Reader<&[u8]>) -> Result<ParaContent, DocxError> {
    let mut pieces = Vec::new();
    let mut current = String::new();
    loop {
        match reader.read_event().map_err(xml_err)? {
            Event::Eof => return Err(DocxError::MalformedXml { detail: "eof giua w:p".to_owned() }),
            Event::End(e) if local_name_of_end(&e) == "p" => break,
            Event::Start(e) => {
                let local = local_name_of(&e);
                match local.as_str() {
                    "t" => current.push_str(&read_element_text(reader)?),
                    "tab" => current.push('\t'),
                    "br" | "cr" => current.push(' '),
                    "drawing" | "pict" => {
                        if let Some(rid) = find_image_rel_id(reader)? {
                            if !current.is_empty() {
                                pieces.push(ParaPiece::Text(std::mem::take(&mut current)));
                            }
                            pieces.push(ParaPiece::Image(rid));
                        }
                    }
                    other if is_opaque(other) => {
                        reader.read_to_end(e.name()).map_err(xml_err)?;
                    }
                    // "r"/"hyperlink"/"ins"/"del"/"sdt"/"sdtContent"/… -- trong suốt.
                    _ => {}
                }
            }
            Event::Empty(e) => {
                let local = local_name_of(&e);
                match local.as_str() {
                    "tab" => current.push('\t'),
                    "br" | "cr" => current.push(' '),
                    "t" => {}
                    _ => {}
                }
            }
            _ => {}
        }
    }
    if !current.is_empty() {
        pieces.push(ParaPiece::Text(current));
    }
    Ok(ParaContent { pieces })
}

/// Đọc TRỌN nội dung chữ của một `w:t` -- caller đã tiêu thụ đúng `Event::Start` của nó.
fn read_element_text(reader: &mut Reader<&[u8]>) -> Result<String, DocxError> {
    let mut out = String::new();
    let mut depth = 0i32;
    loop {
        match reader.read_event().map_err(xml_err)? {
            Event::Eof => return Err(DocxError::MalformedXml { detail: "eof giua w:t".to_owned() }),
            Event::Text(t) => {
                let decoded = t.decode().map_err(|e| DocxError::MalformedXml { detail: e.to_string() })?;
                out.push_str(&decoded);
            }
            Event::CData(t) => {
                if let Ok(s) = std::str::from_utf8(t.as_ref()) {
                    out.push_str(s);
                }
            }
            Event::Start(_) => depth += 1,
            Event::End(_) => {
                if depth == 0 {
                    break;
                }
                depth -= 1;
            }
            _ => {}
        }
    }
    Ok(out)
}

/// Tìm rId ảnh ĐẦU TIÊN (`a:blip[@r:embed]` hoặc `v:imagedata[@r:id]`) bên trong một
/// `w:drawing`/`w:pict` -- caller đã tiêu thụ đúng `Event::Start` của nó. Đọc TRỌN subtree
/// bằng đếm ĐỘ SÂU chung (không so tên) — XML hợp lệ đảm bảo Start/End cân bằng, nên lượt
/// `End` đầu tiên gặp ở độ sâu 0 CHÍNH LÀ End của phần tử đã mở, bất kể tên của nó.
fn find_image_rel_id(reader: &mut Reader<&[u8]>) -> Result<Option<String>, DocxError> {
    let mut depth = 0i32;
    let mut found: Option<String> = None;
    loop {
        match reader.read_event().map_err(xml_err)? {
            Event::Eof => return Err(DocxError::MalformedXml { detail: "eof giua w:drawing/w:pict".to_owned() }),
            Event::Start(e) => {
                if found.is_none() {
                    let local = local_name_of(&e);
                    if local == "blip" {
                        found = attr_value(&e, "embed");
                    } else if local == "imagedata" {
                        found = attr_value(&e, "id");
                    }
                }
                depth += 1;
            }
            Event::Empty(e) => {
                if found.is_none() {
                    let local = local_name_of(&e);
                    if local == "blip" {
                        found = attr_value(&e, "embed");
                    } else if local == "imagedata" {
                        found = attr_value(&e, "id");
                    }
                }
            }
            Event::End(_) => {
                if depth == 0 {
                    break;
                }
                depth -= 1;
            }
            _ => {}
        }
    }
    Ok(found)
}

fn local_name_of_end(e: &quick_xml::events::BytesEnd) -> String {
    std::str::from_utf8(e.local_name().as_ref()).unwrap_or("").to_owned()
}

/// Đẩy một đoạn (đến từ thân tài liệu HOẶC một ô bảng — cùng khuôn) thành các [`Block`], gán
/// `exact_gap_before = "\n\n"` cho mọi khối chữ KHÔNG PHẢI khối chữ đầu tiên toàn tài liệu
/// (xem doc-comment đầu module mục 4/5) và phân giải byte ảnh nhúng qua `rels_map`.
fn push_paragraph<R: Read + std::io::Seek>(
    para: &ParaContent,
    blocks: &mut Vec<Block>,
    images: &mut Vec<DocxImage>,
    first_text_seen: &mut bool,
    rels_map: &BTreeMap<String, String>,
    archive: &mut zip::ZipArchive<R>,
) {
    for piece in &para.pieces {
        match piece {
            ParaPiece::Text(t) => {
                if t.is_empty() {
                    continue;
                }
                let gap = if *first_text_seen { "\n\n" } else { "" };
                *first_text_seen = true;
                blocks.push(Block {
                    body: BlockBody::Paragraph(t.clone()),
                    machine_kept: true,
                    exact_gap_before: gap.to_owned(),
                    is_list_item: false,
                });
            }
            ParaPiece::Image(rid) => {
                let block_index = blocks.len();
                blocks.push(Block {
                    body: BlockBody::Image { src: None, alt: None },
                    machine_kept: true,
                    exact_gap_before: String::new(),
                    is_list_item: false,
                });
                if let Some(target) = rels_map.get(rid) {
                    let path = normalize_media_path(target);
                    if let Some(data) = read_zip_entry(archive, &path) {
                        // 🔴 SỬA (bắt được sau code review) — `rsplit('.').next()` trên một
                        // chuỗi KHÔNG mang dấu chấm trả về NGUYÊN chuỗi đó (không phải rỗng,
                        // `rsplit` trên một mẫu không khớp cho đúng một phần tử là toàn bộ
                        // input) — một tên tệp media không đuôi (hiếm nhưng hợp lệ trong
                        // OOXML) làm `ext` thành CẢ ĐƯỜNG DẪN (`"word/media/anh"`), và chuỗi
                        // chẩn đoán `application/octet-stream; ext=...` của
                        // `mime_for_media_extension` nói dối bằng cách dán cả đường dẫn vào
                        // chỗ một đuôi tệp. `rsplit_once('.')` cho `None` đúng nghĩa khi
                        // không có dấu chấm — kết quả TỪ CHỐI (MIME ngoài danh mục 4) giữ
                        // nguyên, chỉ chuỗi chẩn đoán nói đúng sự thật.
                        let basename = path.rsplit('/').next().unwrap_or(path.as_str());
                        let ext = basename.rsplit_once('.').map(|(_, e)| e).unwrap_or("");
                        images.push(DocxImage {
                            block_index,
                            bytes: data,
                            content_type: mime_for_media_extension(ext),
                        });
                    }
                }
            }
        }
    }
}
