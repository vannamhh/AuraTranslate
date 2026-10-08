use super::images::{ImageMode, ScopeImage, chapter_images};
use crate::commands::segment::{ChapterSegment, select_chapter_assets, select_chapter_segments};
use crate::core::segment::omit::segments_in_translation;
use crate::core::store::{Store, StoreError};

/// Một ô của bảng xuất: đúng một đoạn (`\n` trong chữ thành dấu xuống dòng của đoạn đó) và cờ
/// kết đoạn của chính cột đó.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExportCell {
    pub text: String,
    pub ends_paragraph: bool,
}

/// Một ảnh ở vị trí neo của nó; cả hai cột mang cùng một tham chiếu.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExportImage {
    pub chapter_id: i64,
    pub chapter_ord: i64,
    pub asset_id: i64,
    pub file_name: String,
    pub source_url: Option<String>,
}

/// Một segment thuộc bản dịch, hoặc một ảnh: đúng một hàng.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExportRow {
    Text { source: ExportCell, target: ExportCell },
    Image(ExportImage),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChapterTable {
    pub chapter_id: i64,
    pub title: Option<String>,
    pub rows: Vec<ExportRow>,
}

/// Cột phải: cờ kết đoạn đọc từ cờ đích đã lưu (AD-46), không suy từ nguyên văn. Mỗi ô là một
/// đoạn duy nhất để bảng một hàng không bao giờ mang hình dạng AD-38 từ chối. Câu chưa dịch là
/// một đoạn rỗng để ô vẫn tồn tại.
fn row_of(segment: &ChapterSegment) -> ExportRow {
    ExportRow::Text {
        source: ExportCell {
            text: segment.source_text.clone(),
            ends_paragraph: segment.is_paragraph_end,
        },
        target: ExportCell {
            text: segment.target_text.clone(),
            ends_paragraph: segment.is_target_paragraph_end,
        },
    }
}

/// Bảng đã nạp cùng số ảnh bị bỏ vì chế độ link không có URL của chúng.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LoadedTables {
    pub tables: Vec<ChapterTable>,
    pub images_skipped_missing_link: i64,
}

impl LoadedTables {
    pub fn images(&self) -> Vec<&ExportImage> {
        self.tables
            .iter()
            .flat_map(|t| &t.rows)
            .filter_map(|row| match row {
                ExportRow::Image(image) => Some(image),
                ExportRow::Text { .. } => None,
            })
            .collect()
    }
}

fn image_row(chapter_id: i64, chapter_ord: i64, image: ScopeImage) -> ExportRow {
    ExportRow::Image(ExportImage {
        chapter_id,
        chapter_ord,
        asset_id: image.asset_id,
        file_name: image.file_name,
        source_url: image.source_url,
    })
}

/// Bảng của từng Chương, theo thứ tự `chapter_ids`. Câu bị cắt bỏ không có hàng. Ảnh xen vào
/// ngay sau hàng neo của nó; ở chế độ link, ảnh không có URL không có hàng và được đếm.
pub fn load_chapter_tables(
    store: &Store,
    chapter_ids: &[i64],
    image_mode: ImageMode,
) -> Result<LoadedTables, StoreError> {
    store.read(|conn| {
        let mut title_stmt = conn.prepare("SELECT ord, title FROM chapter WHERE id = ?1")?;
        let mut tables = Vec::with_capacity(chapter_ids.len());
        let mut skipped = 0_i64;
        for &chapter_id in chapter_ids {
            let (chapter_ord, title): (i64, Option<String>) =
                title_stmt.query_row([chapter_id], |row| Ok((row.get(0)?, row.get(1)?)))?;
            let segments = select_chapter_segments(conn, chapter_id)?;
            let assets = select_chapter_assets(conn, chapter_id)?;
            let mut pending = Vec::new();
            for image in chapter_images(&segments, &assets) {
                if image_mode == ImageMode::Link && image.source_url.is_none() {
                    skipped += 1;
                } else {
                    pending.push(image);
                }
            }
            let mut rows = Vec::new();
            let (head, mut rest): (Vec<_>, Vec<_>) = pending.into_iter().partition(|i| i.after_segment_id.is_none());
            rows.extend(head.into_iter().map(|image| image_row(chapter_id, chapter_ord, image)));
            for segment in segments_in_translation(&segments) {
                rows.push(row_of(segment));
                let (here, later): (Vec<_>, Vec<_>) =
                    rest.into_iter().partition(|i| i.after_segment_id == Some(segment.id));
                rest = later;
                rows.extend(here.into_iter().map(|image| image_row(chapter_id, chapter_ord, image)));
            }
            rows.extend(rest.into_iter().map(|image| image_row(chapter_id, chapter_ord, image)));
            tables.push(ChapterTable { chapter_id, title, rows });
        }
        Ok(LoadedTables { tables, images_skipped_missing_link: skipped })
    })
}
