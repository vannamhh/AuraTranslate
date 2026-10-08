use std::collections::BTreeMap;

use super::attribution::{Attribution, lines_for};
use super::images::{ImageMode, ImageSplit, chapter_images, split_by_anchor};
use super::table_rows::ExportImage;
use crate::commands::segment::{ChapterSegment, select_chapter_assets, select_chapter_segments};
use crate::core::segment::reading::paragraphs_by_flag;
use crate::core::segment::regroup::source_joiner;
use crate::core::store::{Store, StoreError};

const BLANK_CHARS: [char; 6] = [' ', '\t', '\n', '\r', '\u{b}', '\u{c}'];

/// Must share one whitespace set with `is_untranslated`, including `\n`.
pub const UNTRANSLATED_SQL: &str = "trim(target_text, ' ' || char(9) || char(10) || char(11) || char(12) || char(13)) = ''";

/// Must share one whitespace set with `UNTRANSLATED_SQL`, including `\n`.
pub fn is_untranslated(target_text: &str) -> bool {
    is_blank(target_text)
}

fn is_blank(text: &str) -> bool {
    text.trim_matches(BLANK_CHARS).is_empty()
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BlockParagraph {
    Text(String),
    Image(ExportImage),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChapterBlock {
    pub chapter_id: i64,
    pub title: Option<String>,
    pub attribution: Vec<String>,
    pub source: Vec<BlockParagraph>,
    pub target: Vec<BlockParagraph>,
}

impl ChapterBlock {
    pub fn has_table(&self) -> bool {
        !self.source.is_empty()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LoadedBlocks {
    pub blocks: Vec<ChapterBlock>,
    pub images_skipped_missing_link: i64,
}

impl LoadedBlocks {
    pub fn images(&self) -> Vec<&ExportImage> {
        self.blocks
            .iter()
            .flat_map(|b| &b.source)
            .filter_map(|p| match p {
                BlockParagraph::Image(image) => Some(image),
                BlockParagraph::Text(_) => None,
            })
            .collect()
    }
}

struct ColumnBuilder {
    joiner: &'static str,
    paragraphs: Vec<BlockParagraph>,
    pieces: Vec<String>,
}

impl ColumnBuilder {
    fn new(joiner: &'static str) -> Self {
        Self { joiner, paragraphs: Vec::new(), pieces: Vec::new() }
    }

    fn flush(&mut self) {
        if !self.pieces.is_empty() {
            let text = self.pieces.join(self.joiner);
            self.paragraphs.push(BlockParagraph::Text(text));
            self.pieces.clear();
        }
    }

    fn push_images(&mut self, images: &[ExportImage]) {
        self.flush();
        self.paragraphs.extend(images.iter().cloned().map(BlockParagraph::Image));
    }
}

pub(super) fn target_pieces(text: &str) -> Vec<&str> {
    text.split('\n').map(|piece| piece.trim_matches(BLANK_CHARS)).filter(|piece| !is_blank(piece)).collect()
}

fn source_pieces(text: &str) -> Vec<&str> {
    if is_blank(text) { Vec::new() } else { vec![text] }
}

fn build_column<'a>(
    segments: &'a [ChapterSegment],
    ends_paragraph: impl Fn(&ChapterSegment) -> bool,
    joiner: &'static str,
    pieces_of: impl Fn(&'a ChapterSegment) -> Vec<&'a str>,
    head: &[ExportImage],
    by_anchor: &BTreeMap<i64, Vec<ExportImage>>,
    orphans: &[ExportImage],
) -> Vec<BlockParagraph> {
    let mut column = ColumnBuilder::new(joiner);
    column.push_images(head);
    for group in paragraphs_by_flag(segments, ends_paragraph) {
        for segment in group {
            for (i, piece) in pieces_of(segment).into_iter().enumerate() {
                if i > 0 {
                    column.flush();
                }
                column.pieces.push(piece.to_owned());
            }
            if let Some(images) = by_anchor.get(&segment.id) {
                column.push_images(images);
            }
        }
        column.flush();
    }
    column.push_images(orphans);
    column.paragraphs
}

fn build_chapter_block(
    chapter_id: i64,
    title: Option<String>,
    attribution: Vec<String>,
    segments: &[ChapterSegment],
    images: ImageSplit<ExportImage>,
    source_lang: &str,
) -> ChapterBlock {
    let ImageSplit { head, by_anchor, orphans, .. } = images;
    let source = build_column(
        segments,
        |s| s.is_paragraph_end,
        source_joiner(source_lang),
        |s| source_pieces(&s.source_text),
        &head,
        &by_anchor,
        &orphans,
    );
    let target = build_column(
        segments,
        |s| s.is_target_paragraph_end,
        " ",
        |s| target_pieces(&s.target_text),
        &head,
        &by_anchor,
        &orphans,
    );
    ChapterBlock { chapter_id, title, attribution, source, target }
}

pub fn load_chapter_blocks(
    store: &Store,
    chapter_ids: &[i64],
    image_mode: ImageMode,
    source_lang: &str,
    attribution: Option<&Attribution>,
) -> Result<LoadedBlocks, StoreError> {
    store.read(|conn| {
        let mut title_stmt = conn.prepare("SELECT ord, title FROM chapter WHERE id = ?1")?;
        let mut blocks = Vec::with_capacity(chapter_ids.len());
        let mut skipped = 0_i64;
        for &chapter_id in chapter_ids {
            let (chapter_ord, title): (i64, Option<String>) =
                title_stmt.query_row([chapter_id], |row| Ok((row.get(0)?, row.get(1)?)))?;
            let segments = select_chapter_segments(conn, chapter_id)?;
            let assets = select_chapter_assets(conn, chapter_id)?;
            let split = split_by_anchor(&segments, chapter_images(&segments, &assets), image_mode, |image| ExportImage {
                chapter_id,
                chapter_ord,
                asset_id: image.asset_id,
                file_name: image.file_name,
                source_url: image.source_url,
            });
            skipped += split.skipped;
            let lines = lines_for(conn, chapter_id, attribution)?;
            blocks.push(build_chapter_block(chapter_id, title, lines, &segments, split, source_lang));
        }
        Ok(LoadedBlocks { blocks, images_skipped_missing_link: skipped })
    })
}
