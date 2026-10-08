use std::collections::{BTreeMap, BTreeSet};

use crate::commands::segment::{ChapterSegment, select_chapter_assets, select_chapter_segments};
use crate::core::segment::image::resolve_chapter_images;
use crate::core::store::{Store, StoreError};

/// Cách ghi ảnh vào tệp xuất (FR130): `Link` ghi `source_url` của bài gốc, `File` chép tệp ảnh
/// từ `assets/` ra thư mục cạnh tệp.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ImageMode {
    Link,
    File,
}

/// Một ảnh của Chương, đã đặt vào không gian hàng của bảng xuất.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScopeImage {
    pub asset_id: i64,
    pub file_name: String,
    pub source_url: Option<String>,
    /// Hàng (segment thuộc bản dịch) đứng ngay trước ảnh; `None` ⇒ đầu Chương.
    pub after_segment_id: Option<i64>,
    pub alt_text: Option<String>,
    pub caption_text: Option<String>,
    /// Thứ tự ảnh trong Chương, từ 1.
    pub index_in_chapter: i64,
}

/// Ảnh của một Chương theo thứ tự `asset.id`. Neo xét trên đúng tập segment có hàng trong bảng
/// xuất: không về hưu, không bị cắt bỏ, vai `alt`/`caption` vẫn là hàng.
pub(crate) fn chapter_images(
    segments: &[ChapterSegment],
    assets: &[crate::core::segment::image::RawAsset],
) -> Vec<ScopeImage> {
    resolve_chapter_images(segments, assets, false, false)
        .into_iter()
        .enumerate()
        .map(|(i, image)| ScopeImage {
            asset_id: image.asset_id,
            file_name: image.file_name,
            source_url: image.source_url,
            after_segment_id: image.after_segment_id,
            alt_text: image.alt_text,
            caption_text: image.caption_text,
            index_in_chapter: i64::try_from(i).map_or(i64::MAX, |n| n + 1),
        })
        .collect()
}

pub(super) struct ImageSplit<T> {
    pub head: Vec<T>,
    pub by_anchor: BTreeMap<i64, Vec<T>>,
    pub orphans: Vec<T>,
    pub skipped: i64,
}

/// Link mode drops images without `source_url` and counts them; an anchor on an omitted segment
/// makes the image an orphan placed at the end of the Chapter.
pub(super) fn split_by_anchor<T>(
    segments: &[ChapterSegment],
    images: Vec<ScopeImage>,
    image_mode: ImageMode,
    mut make: impl FnMut(ScopeImage) -> T,
) -> ImageSplit<T> {
    let survivors: BTreeSet<i64> = segments.iter().filter(|s| !s.is_omitted).map(|s| s.id).collect();
    let mut split = ImageSplit { head: Vec::new(), by_anchor: BTreeMap::new(), orphans: Vec::new(), skipped: 0 };
    for image in images {
        if image_mode == ImageMode::Link && image.source_url.is_none() {
            split.skipped += 1;
            continue;
        }
        match image.after_segment_id {
            None => split.head.push(make(image)),
            Some(id) if survivors.contains(&id) => split.by_anchor.entry(id).or_default().push(make(image)),
            Some(_) => split.orphans.push(make(image)),
        }
    }
    split
}

/// Ảnh thiếu `source_url` của phạm vi, để người dùng thấy trước khi chọn chế độ link.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct MissingLinkImage {
    pub chapter_id: i64,
    pub chapter_ord: i64,
    pub chapter_title: Option<String>,
    pub image_index: i64,
    pub alt_text: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct ImageScan {
    pub image_count: i64,
    pub missing_link_images: Vec<MissingLinkImage>,
}

/// Quét ảnh của các Chương trong một lượt đọc (AD-43): đếm mọi ảnh và liệt kê ảnh thiếu link.
pub fn scan_images(store: &Store, chapter_ids: &[i64]) -> Result<ImageScan, StoreError> {
    store.read(|conn| {
        let mut title_stmt = conn.prepare("SELECT ord, title FROM chapter WHERE id = ?1")?;
        let mut image_count = 0_i64;
        let mut missing_link_images = Vec::new();
        for &chapter_id in chapter_ids {
            let assets = select_chapter_assets(conn, chapter_id)?;
            if assets.is_empty() {
                continue;
            }
            let segments = select_chapter_segments(conn, chapter_id)?;
            let images = chapter_images(&segments, &assets);
            image_count += i64::try_from(images.len()).unwrap_or(i64::MAX);
            let mut missing = images.into_iter().filter(|image| image.source_url.is_none()).peekable();
            if missing.peek().is_none() {
                continue;
            }
            let (chapter_ord, chapter_title): (i64, Option<String>) =
                title_stmt.query_row([chapter_id], |row| Ok((row.get(0)?, row.get(1)?)))?;
            for image in missing {
                missing_link_images.push(MissingLinkImage {
                    chapter_id,
                    chapter_ord,
                    chapter_title: chapter_title.clone(),
                    image_index: image.index_in_chapter,
                    alt_text: image.alt_text,
                });
            }
        }
        Ok(ImageScan { image_count, missing_link_images })
    })
}
