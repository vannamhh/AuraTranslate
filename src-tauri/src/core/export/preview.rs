use super::attribution::Attribution;
use super::block_paragraphs::{BlockParagraph, load_chapter_blocks};
use super::images::ImageMode;
use super::options::{ExportContent, ExportFormat};
use super::scope::{ExportScope, ScopeError, resolve_chapter_ids};
use super::table_rows::{ExportRow, load_chapter_tables};
use super::text_export::load_chapter_text;
use crate::core::store::Store;

/// Rows or paragraphs the preview carries from the first Chapter.
pub const PREVIEW_LIMIT: usize = 6;

const APPENDIX_PREVIEW_TERMS: usize = 5;

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct ExportPreviewRow {
    pub source: Option<String>,
    pub target: String,
}

/// The first rows of the file an export would write for the first Chapter of the scope, built
/// by the same loaders as the export itself. Images are left out.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct ExportPreview {
    pub chapter_id: i64,
    pub chapter_ord: i64,
    pub chapter_title: Option<String>,
    pub file_name: String,
    pub attribution: Vec<String>,
    pub appendix: Vec<String>,
    pub appendix_term_count: i64,
    pub not_reimportable: bool,
    pub rows: Vec<ExportPreviewRow>,
}

fn join_paragraphs(paragraphs: &[BlockParagraph]) -> String {
    paragraphs
        .iter()
        .filter_map(|p| match p {
            BlockParagraph::Text(text) => Some(text.as_str()),
            BlockParagraph::Image(_) => None,
        })
        .take(PREVIEW_LIMIT)
        .collect::<Vec<_>>()
        .join("\n\n")
}

/// # Errors
/// Same families as [`resolve_chapter_ids`].
pub fn build_preview(
    store: &Store,
    scope: &ExportScope,
    format: ExportFormat,
    content: &ExportContent,
    attribution: Option<&Attribution>,
    source_lang: &str,
    work_name: &str,
) -> Result<ExportPreview, ScopeError> {
    let chapter_ids = resolve_chapter_ids(store, scope)?;
    let first = chapter_ids.iter().copied().take(1).collect::<Vec<_>>();
    let &[chapter_id] = first.as_slice() else {
        return Err(ScopeError::Empty);
    };
    let (chapter_ord, chapter_title): (i64, Option<String>) = store.read(|conn| {
        conn.query_row(
            "SELECT ord, title FROM chapter WHERE id = ?1",
            [chapter_id],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
    })?;
    let options = content.options;
    let (chapter_attribution, rows) = match format {
        ExportFormat::DocxTwoColumn => {
            let loaded = load_chapter_tables(store, &first, ImageMode::Link, attribution, options)?;
            let table = loaded.tables.into_iter().next();
            let attribution = table
                .as_ref()
                .map(|t| t.attribution.clone())
                .unwrap_or_default();
            let rows = table
                .into_iter()
                .flat_map(|t| t.rows)
                .filter_map(|row| match row {
                    ExportRow::Text { source, target } => Some(ExportPreviewRow {
                        source: Some(source.text),
                        target: target.text,
                    }),
                    ExportRow::Image(_) => None,
                })
                .take(PREVIEW_LIMIT)
                .collect();
            (attribution, rows)
        }
        ExportFormat::DocxOneBlock => {
            let loaded = load_chapter_blocks(
                store,
                &first,
                ImageMode::Link,
                source_lang,
                attribution,
                options,
            )?;
            match loaded.blocks.into_iter().next() {
                Some(block) if block.has_table() => {
                    let row = ExportPreviewRow {
                        source: Some(join_paragraphs(&block.source)),
                        target: join_paragraphs(&block.target),
                    };
                    (block.attribution, vec![row])
                }
                Some(block) => (block.attribution, Vec::new()),
                None => (Vec::new(), Vec::new()),
            }
        }
        ExportFormat::Markdown | ExportFormat::Plain => {
            let loaded = load_chapter_text(store, &first, ImageMode::Link, attribution, options)?;
            let (attribution, paragraphs) =
                loaded.preview(PREVIEW_LIMIT, format == ExportFormat::Markdown);
            let rows = paragraphs
                .into_iter()
                .map(|target| ExportPreviewRow {
                    source: None,
                    target,
                })
                .collect();
            (attribution, rows)
        }
    };
    let appendix: Vec<String> = content
        .appendix_lines()
        .into_iter()
        .take(APPENDIX_PREVIEW_TERMS + 1)
        .collect();
    Ok(ExportPreview {
        chapter_id,
        chapter_ord,
        chapter_title,
        file_name: format!(
            "{}{}.{}",
            super::new_file::safe_stem(work_name, "export"),
            format.stem_suffix(),
            format.extension()
        ),
        attribution: chapter_attribution,
        appendix,
        appendix_term_count: i64::try_from(content.appendix.len()).unwrap_or(i64::MAX),
        not_reimportable: content.options.annotated(),
        rows,
    })
}
