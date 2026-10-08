use std::io::Cursor;

use docx_rs::{Docx, Table, TableCell, TableLayoutType, TableRow, WidthType};

use super::block_paragraphs::{BlockParagraph, ChapterBlock};
use super::docx_table::{COLUMN_WIDTH_DXA, DocxWriteError, ImageReference, image_paragraph, paragraph_of};

fn cell_of(paragraphs: &[BlockParagraph], reference: ImageReference<'_>) -> TableCell {
    let mut cell = TableCell::new().width(COLUMN_WIDTH_DXA, WidthType::Dxa).clear_all_border();
    for paragraph in paragraphs {
        cell = cell.add_paragraph(match paragraph {
            BlockParagraph::Text(text) => paragraph_of(text, true),
            BlockParagraph::Image(image) => image_paragraph(image, reference),
        });
    }
    // OOXML requires every cell to end in a paragraph, so an empty cell gets one empty paragraph.
    if paragraphs.is_empty() {
        cell = cell.add_paragraph(paragraph_of("", false));
    }
    cell
}

pub fn write_one_block_docx(blocks: &[ChapterBlock], reference: ImageReference<'_>) -> Result<Vec<u8>, DocxWriteError> {
    let mut docx = Docx::new();
    for block in blocks {
        if !block.attribution.is_empty() {
            docx = docx.add_paragraph(paragraph_of(&block.attribution.join("\n"), true));
        }
        docx = docx.add_paragraph(paragraph_of(block.title.as_deref().unwrap_or(""), true));
        if !block.has_table() {
            continue;
        }
        let row = TableRow::new(vec![cell_of(&block.source, reference), cell_of(&block.target, reference)]);
        docx = docx.add_table(
            Table::new(vec![row])
                .set_grid(vec![COLUMN_WIDTH_DXA, COLUMN_WIDTH_DXA])
                .layout(TableLayoutType::Fixed)
                .clear_all_border(),
        );
    }
    let mut buffer = Cursor::new(Vec::new());
    docx.build().pack(&mut buffer).map_err(|e| DocxWriteError(e.to_string()))?;
    Ok(buffer.into_inner())
}
