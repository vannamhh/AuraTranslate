use std::io::Cursor;

use docx_rs::{
    BreakType, Docx, Hyperlink, HyperlinkType, LineSpacing, Paragraph, Run, Table, TableCell, TableLayoutType, TableRow,
    WidthType,
};

use super::image_files::copied_name;
use super::table_rows::{ChapterTable, ExportCell, ExportImage, ExportRow};

/// Điều một hàng ảnh ghi ra: link tới `source_url`, hoặc đường dẫn tương đối tới tệp trong thư mục ảnh.
#[derive(Debug, Clone, Copy)]
pub enum ImageReference<'a> {
    Link,
    Dir(&'a str),
}

pub(super) const COLUMN_WIDTH_DXA: usize = 4513;
const PARAGRAPH_GAP_DXA: u32 = 240;

/// Lý do ghi `.docx` thất bại; chỉ để chẩn đoán, không đi lên giao diện.
#[derive(Debug)]
pub struct DocxWriteError(pub String);

pub(super) fn paragraph_of(text: &str, gap_after: bool) -> Paragraph {
    let mut run = Run::new();
    for (i, line) in text.split('\n').enumerate() {
        if i > 0 {
            run = run.add_break(BreakType::TextWrapping);
        }
        run = run.add_text(line);
    }
    let gap = if gap_after { PARAGRAPH_GAP_DXA } else { 0 };
    Paragraph::new().add_run(run).line_spacing(LineSpacing::new().after(gap))
}

fn cell_of(cell: &ExportCell) -> TableCell {
    TableCell::new().width(COLUMN_WIDTH_DXA, WidthType::Dxa).add_paragraph(paragraph_of(&cell.text, cell.ends_paragraph))
}

pub(super) fn image_paragraph(image: &ExportImage, reference: ImageReference<'_>) -> Paragraph {
    let paragraph = Paragraph::new().line_spacing(LineSpacing::new().after(PARAGRAPH_GAP_DXA));
    match (reference, image.source_url.as_deref()) {
        (ImageReference::Link, Some(url)) => paragraph
            .add_hyperlink(Hyperlink::new(url, HyperlinkType::External).add_run(Run::new().add_text(url))),
        (ImageReference::Dir(dir), _) => {
            paragraph.add_run(Run::new().add_text(format!("{dir}/{}", copied_name(image))))
        }
        (ImageReference::Link, None) => paragraph.add_run(Run::new().add_text(&image.file_name)),
    }
}

fn image_cell(image: &ExportImage, reference: ImageReference<'_>) -> TableCell {
    TableCell::new().width(COLUMN_WIDTH_DXA, WidthType::Dxa).add_paragraph(image_paragraph(image, reference))
}

fn row_of(row: &ExportRow, reference: ImageReference<'_>) -> TableRow {
    match row {
        ExportRow::Text { source, target } => TableRow::new(vec![cell_of(source), cell_of(target)]),
        ExportRow::Image(image) => TableRow::new(vec![image_cell(image, reference), image_cell(image, reference)]),
    }
    .cant_split()
}

/// Mỗi Chương: một đoạn tiêu đề rồi một bảng hai cột, mỗi segment một hàng, không hàng đầu bảng.
/// Chương không có hàng nào chỉ có đoạn tiêu đề (bảng không hàng làm Word từ chối tệp).
pub fn write_two_column_docx(tables: &[ChapterTable], reference: ImageReference<'_>) -> Result<Vec<u8>, DocxWriteError> {
    let mut docx = Docx::new();
    for chapter in tables {
        if !chapter.attribution.is_empty() {
            docx = docx.add_paragraph(paragraph_of(&chapter.attribution.join("\n"), true));
        }
        docx = docx.add_paragraph(paragraph_of(chapter.title.as_deref().unwrap_or(""), true));
        if chapter.rows.is_empty() {
            continue;
        }
        let rows = chapter.rows.iter().map(|row| row_of(row, reference)).collect();
        docx = docx.add_table(
            Table::new(rows).set_grid(vec![COLUMN_WIDTH_DXA, COLUMN_WIDTH_DXA]).layout(TableLayoutType::Fixed),
        );
    }
    let mut buffer = Cursor::new(Vec::new());
    docx.build().pack(&mut buffer).map_err(|e| DocxWriteError(e.to_string()))?;
    Ok(buffer.into_inner())
}
