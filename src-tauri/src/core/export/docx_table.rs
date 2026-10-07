use std::io::Cursor;

use docx_rs::{BreakType, Docx, LineSpacing, Paragraph, Run, Table, TableCell, TableLayoutType, TableRow, WidthType};

use super::table_rows::{ChapterTable, ExportCell};

const COLUMN_WIDTH_DXA: usize = 4513;
const PARAGRAPH_GAP_DXA: u32 = 240;

/// Lý do ghi `.docx` thất bại; chỉ để chẩn đoán, không đi lên giao diện.
#[derive(Debug)]
pub struct DocxWriteError(pub String);

fn paragraph_of(text: &str, gap_after: bool) -> Paragraph {
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

/// Mỗi Chương: một đoạn tiêu đề rồi một bảng hai cột, mỗi segment một hàng, không hàng đầu bảng.
/// Chương không có hàng nào chỉ có đoạn tiêu đề (bảng không hàng làm Word từ chối tệp).
pub fn write_two_column_docx(tables: &[ChapterTable]) -> Result<Vec<u8>, DocxWriteError> {
    let mut docx = Docx::new();
    for chapter in tables {
        docx = docx.add_paragraph(paragraph_of(chapter.title.as_deref().unwrap_or(""), true));
        if chapter.rows.is_empty() {
            continue;
        }
        let rows = chapter
            .rows
            .iter()
            .map(|row| TableRow::new(vec![cell_of(&row.source), cell_of(&row.target)]).cant_split())
            .collect();
        docx = docx.add_table(
            Table::new(rows).set_grid(vec![COLUMN_WIDTH_DXA, COLUMN_WIDTH_DXA]).layout(TableLayoutType::Fixed),
        );
    }
    let mut buffer = Cursor::new(Vec::new());
    docx.build().pack(&mut buffer).map_err(|e| DocxWriteError(e.to_string()))?;
    Ok(buffer.into_inner())
}
