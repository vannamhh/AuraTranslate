use super::attribution::{Attribution, lines_for};
use super::block_paragraphs::target_pieces;
use super::docx_table::ImageReference;
use super::image_files::copied_name;
use super::images::{ImageMode, ImageSplit, chapter_images, split_by_anchor};
use super::table_rows::ExportImage;
use crate::commands::segment::{select_chapter_assets, select_chapter_segments};
use crate::core::segment::reading::paragraphs_by_flag;
use crate::core::store::{Store, StoreError};

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TextFormat {
    Markdown,
    Plain,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct TextImage {
    image: ExportImage,
    alt: String,
    caption: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum TextItem {
    Paragraph(String),
    Image(TextImage),
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct TextChapter {
    chapter_ord: i64,
    title: Option<String>,
    attribution: Vec<String>,
    items: Vec<TextItem>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LoadedText {
    chapters: Vec<TextChapter>,
    pub images_skipped_missing_link: i64,
}

pub(super) fn one_line(text: &str) -> String {
    text.split(char::is_whitespace).filter(|word| !word.is_empty()).collect::<Vec<_>>().join(" ")
}

pub fn load_chapter_text(
    store: &Store,
    chapter_ids: &[i64],
    image_mode: ImageMode,
    attribution: Option<&Attribution>,
) -> Result<LoadedText, StoreError> {
    store.read(|conn| {
        let mut title_stmt = conn.prepare("SELECT ord, title FROM chapter WHERE id = ?1")?;
        let mut chapters = Vec::with_capacity(chapter_ids.len());
        let mut skipped = 0_i64;
        for &chapter_id in chapter_ids {
            let (chapter_ord, title): (i64, Option<String>) =
                title_stmt.query_row([chapter_id], |row| Ok((row.get(0)?, row.get(1)?)))?;
            let segments = select_chapter_segments(conn, chapter_id)?;
            let assets = select_chapter_assets(conn, chapter_id)?;
            let ImageSplit { head, mut by_anchor, orphans, skipped: chapter_skipped } =
                split_by_anchor(&segments, chapter_images(&segments, &assets), image_mode, |image| TextImage {
                    alt: one_line(image.alt_text.as_deref().unwrap_or("")),
                    caption: one_line(image.caption_text.as_deref().unwrap_or("")),
                    image: ExportImage {
                        chapter_id,
                        chapter_ord,
                        asset_id: image.asset_id,
                        file_name: image.file_name,
                        source_url: image.source_url,
                    },
                });
            skipped += chapter_skipped;
            let mut items: Vec<TextItem> = head.into_iter().map(TextItem::Image).collect();
            for group in paragraphs_by_flag(&segments, |s| s.is_target_paragraph_end) {
                let mut pieces: Vec<String> = Vec::new();
                let flush = |pieces: &mut Vec<String>, items: &mut Vec<TextItem>| {
                    if !pieces.is_empty() {
                        items.push(TextItem::Paragraph(pieces.join(" ")));
                        pieces.clear();
                    }
                };
                for segment in group {
                    if segment.role.is_none() {
                        for (i, piece) in target_pieces(&segment.target_text).into_iter().enumerate() {
                            if i > 0 {
                                flush(&mut pieces, &mut items);
                            }
                            pieces.push(piece.to_owned());
                        }
                    }
                    if let Some(images) = by_anchor.remove(&segment.id) {
                        flush(&mut pieces, &mut items);
                        items.extend(images.into_iter().map(TextItem::Image));
                    }
                }
                flush(&mut pieces, &mut items);
            }
            items.extend(orphans.into_iter().map(TextItem::Image));
            let attribution = lines_for(conn, chapter_id, attribution)?;
            chapters.push(TextChapter { chapter_ord, title, attribution, items });
        }
        Ok(LoadedText { chapters, images_skipped_missing_link: skipped })
    })
}

impl LoadedText {
    pub fn images(&self) -> Vec<&ExportImage> {
        self.chapters
            .iter()
            .flat_map(|c| &c.items)
            .filter_map(|item| match item {
                TextItem::Image(i) => Some(&i.image),
                TextItem::Paragraph(_) => None,
            })
            .collect()
    }
}

fn escape_inline(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for c in text.chars() {
        if matches!(c, '\\' | '*' | '_' | '`' | '[' | ']' | '<' | '&') {
            out.push('\\');
        }
        out.push(c);
    }
    out
}

fn escape_line_start(escaped: String) -> String {
    let digits = escaped.chars().take_while(char::is_ascii_digit).count();
    if digits > 0 && matches!(escaped[digits..].chars().next(), Some('.' | ')')) {
        return format!("{}\\{}", &escaped[..digits], &escaped[digits..]);
    }
    if escaped.starts_with(['#', '>', '-', '+', '~', '=']) {
        return format!("\\{escaped}");
    }
    escaped
}

fn markdown_text(text: &str) -> String {
    escape_line_start(escape_inline(text))
}

fn markdown_heading(text: &str) -> String {
    let mut escaped = escape_inline(text);
    if escaped.ends_with('#') {
        escaped.insert(escaped.len() - 1, '\\');
    }
    escaped
}

fn markdown_destination(dest: &str) -> String {
    let flat: String = dest.chars().map(|c| if c.is_control() { ' ' } else { c }).collect();
    if flat.is_empty() || flat.contains(|c: char| c.is_whitespace() || matches!(c, '(' | ')' | '<' | '>' | '\\')) {
        let inner: String =
            flat.chars().flat_map(|c| if matches!(c, '<' | '>' | '\\') { vec!['\\', c] } else { vec![c] }).collect();
        format!("<{inner}>")
    } else {
        flat
    }
}

fn destination_of(image: &ExportImage, reference: ImageReference<'_>) -> String {
    match (reference, image.source_url.as_deref()) {
        (ImageReference::Link, Some(url)) => url.to_owned(),
        (ImageReference::Link, None) => image.file_name.clone(),
        (ImageReference::Dir(dir), _) => format!("{dir}/{}", copied_name(image)),
    }
}

fn markdown_block(item: &TextItem, reference: ImageReference<'_>) -> String {
    match item {
        TextItem::Paragraph(text) => markdown_text(text),
        TextItem::Image(i) => {
            let dest = markdown_destination(&destination_of(&i.image, reference));
            let line = format!("![{}]({dest})", escape_inline(&i.alt));
            if i.caption.is_empty() { line } else { format!("{line}\n*{}*", escape_inline(&i.caption)) }
        }
    }
}

fn plain_block(item: &TextItem, reference: ImageReference<'_>) -> String {
    match item {
        TextItem::Paragraph(text) => text.clone(),
        TextItem::Image(i) => {
            // aura-allow-text: file content the publisher reads, fixed by the export format, not UI text
            let line = format!("[Ảnh: {}] {}", i.alt, destination_of(&i.image, reference));
            if i.caption.is_empty() { line } else { format!("{line}\n{}", i.caption) }
        }
    }
}

pub(super) fn heading_text(title: Option<&str>, chapter_ord: i64) -> String {
    match title.map(one_line) {
        Some(title) if !title.is_empty() => title,
        // aura-allow-text: heading written into the exported file, not UI text
        _ => format!("Chương {chapter_ord}"),
    }
}

fn heading_of(chapter: &TextChapter) -> String {
    heading_text(chapter.title.as_deref(), chapter.chapter_ord)
}

fn attribution_block(lines: &[String], format: TextFormat) -> String {
    match format {
        TextFormat::Markdown => {
            lines.iter().map(|line| markdown_text(line)).collect::<Vec<_>>().join("\\\n")
        }
        TextFormat::Plain => lines.join("\n"),
    }
}

pub fn render_text(loaded: &LoadedText, format: TextFormat, reference: ImageReference<'_>) -> String {
    let mut sections = Vec::new();
    for chapter in loaded.chapters.iter().filter(|c| !c.items.is_empty()) {
        let heading = match format {
            TextFormat::Markdown => format!("## {}", markdown_heading(&heading_of(chapter))),
            TextFormat::Plain => heading_of(chapter),
        };
        let mut blocks = Vec::new();
        if !chapter.attribution.is_empty() {
            blocks.push(attribution_block(&chapter.attribution, format));
        }
        blocks.push(heading);
        blocks.extend(chapter.items.iter().map(|item| match format {
            TextFormat::Markdown => markdown_block(item, reference),
            TextFormat::Plain => plain_block(item, reference),
        }));
        sections.push(blocks.join("\n\n"));
    }
    if sections.is_empty() { String::new() } else { format!("{}\n", sections.join("\n\n")) }
}
