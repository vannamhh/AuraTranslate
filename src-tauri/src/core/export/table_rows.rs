use crate::commands::segment::{ChapterSegment, select_chapter_segments};
use crate::core::segment::omit::segments_in_translation;
use crate::core::store::{Store, StoreError};

/// Một ô của bảng xuất: các đoạn của chính cột đó và cờ kết đoạn của chính cột đó.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExportCell {
    pub paragraphs: Vec<String>,
    pub ends_paragraph: bool,
}

/// Một segment thuộc bản dịch: đúng một hàng.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExportRow {
    pub source: ExportCell,
    pub target: ExportCell,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChapterTable {
    pub chapter_id: i64,
    pub title: Option<String>,
    pub rows: Vec<ExportRow>,
}

/// Cột phải: cấu trúc đoạn đọc từ cờ đích đã lưu và từ ký tự xuống dòng trong `target_text`
/// (AD-46), không suy từ nguyên văn. Câu chưa dịch là một đoạn rỗng để ô vẫn tồn tại.
fn row_of(segment: &ChapterSegment) -> ExportRow {
    ExportRow {
        source: ExportCell {
            paragraphs: vec![segment.source_text.clone()],
            ends_paragraph: segment.is_paragraph_end,
        },
        target: ExportCell {
            paragraphs: segment.target_text.split('\n').map(str::to_owned).collect(),
            ends_paragraph: segment.is_target_paragraph_end,
        },
    }
}

/// Bảng của từng Chương, theo thứ tự `chapter_ids`. Câu bị cắt bỏ không có hàng.
pub fn load_chapter_tables(store: &Store, chapter_ids: &[i64]) -> Result<Vec<ChapterTable>, StoreError> {
    store.read(|conn| {
        let mut title_stmt = conn.prepare("SELECT title FROM chapter WHERE id = ?1")?;
        let mut tables = Vec::with_capacity(chapter_ids.len());
        for &chapter_id in chapter_ids {
            let title: Option<String> = title_stmt.query_row([chapter_id], |row| row.get(0))?;
            let segments = select_chapter_segments(conn, chapter_id)?;
            let rows = segments_in_translation(&segments).into_iter().map(row_of).collect();
            tables.push(ChapterTable { chapter_id, title, rows });
        }
        Ok(tables)
    })
}
