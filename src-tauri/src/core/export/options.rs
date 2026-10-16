use std::collections::BTreeMap;

use super::block_paragraphs::is_untranslated;
use crate::commands::segment::ChapterSegment;
use crate::core::i18n::{IpcError, MessageKey};
use crate::core::segment::omit::segments_in_translation;

/// First line of an annotated `.md` export; the `.docx` twin is `core::docx::EXPORT_ANNOTATED_PROPERTY`.
pub const MARKDOWN_ANNOTATED_MARKER: &str = "<!-- aura:export-annotated -->";

/// Content switches of the export screen (FR152). Every one is off by default; with all three off
/// the file is byte-identical to the one written before they existed.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, serde::Deserialize)]
#[serde(default)]
pub struct ExportOptions {
    pub number_segments: bool,
    pub glossary_appendix: bool,
    pub mark_unconfirmed: bool,
}

impl ExportOptions {
    /// Sentence numbers and marks are glued into the two existing cells, so a file carrying
    /// either cannot be re-imported. The appendix is its own paragraphs and does not count.
    pub fn annotated(&self) -> bool {
        self.number_segments || self.mark_unconfirmed
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExportFormat {
    DocxTwoColumn,
    DocxOneBlock,
    Markdown,
    Plain,
}

impl ExportFormat {
    pub fn extension(self) -> &'static str {
        match self {
            Self::DocxTwoColumn | Self::DocxOneBlock => "docx",
            Self::Markdown => "md",
            Self::Plain => "txt",
        }
    }

    pub fn stem_suffix(self) -> &'static str {
        match self {
            Self::DocxTwoColumn => "-hai-cot",
            Self::DocxOneBlock => "-mot-khoi",
            Self::Markdown | Self::Plain => "",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExportOptionsError {
    MarkNeedsTwoColumn,
}

impl From<ExportOptionsError> for IpcError {
    fn from(error: ExportOptionsError) -> Self {
        match error {
            ExportOptionsError::MarkNeedsTwoColumn => IpcError::new(
                "export.mark_two_column_only",
                MessageKey::ExportMarkTwoColumnOnly,
                BTreeMap::new(),
                false,
            ),
        }
    }
}

/// Options plus the resolved glossary appendix `(source_term, translation)`; empty unless the
/// appendix switch is on and the Glossary has confirmed terms.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ExportContent {
    pub options: ExportOptions,
    pub appendix: Vec<(String, String)>,
}

impl ExportContent {
    /// FR121 keeps the one-block copy a publishing shape, so the mark exists only in the
    /// two-column `.docx`.
    pub fn check_for(&self, format: ExportFormat) -> Result<(), ExportOptionsError> {
        if self.options.mark_unconfirmed && format != ExportFormat::DocxTwoColumn {
            Err(ExportOptionsError::MarkNeedsTwoColumn)
        } else {
            Ok(())
        }
    }

    /// Title line then one `term = translation` line per term; empty when there is no appendix.
    pub fn appendix_lines(&self) -> Vec<String> {
        if self.appendix.is_empty() {
            return Vec::new();
        }
        let mut lines = Vec::with_capacity(self.appendix.len() + 1);
        // aura-allow-text: heading written into the exported file, not UI text
        lines.push("Thuật ngữ đã chốt".to_owned());
        lines.extend(
            self.appendix
                .iter()
                .map(|(term, translation)| format!("{term} = {translation}")),
        );
        lines
    }
}

/// 1-based position of each segment among the segments that belong to the translation.
pub(super) fn segment_numbers(segments: &[ChapterSegment]) -> BTreeMap<i64, i64> {
    segments_in_translation(segments)
        .into_iter()
        .zip(1_i64..)
        .map(|(segment, n)| (segment.id, n))
        .collect()
}

pub(super) fn numbered(n: i64, text: &str) -> String {
    format!("[{n}] {text}")
}

/// Appends the unconfirmed-sentence line to the target cell text.
pub(super) fn marked_unconfirmed(target: &str) -> String {
    // aura-allow-text: label written into the exported file, not UI text
    const LABEL: &str = "— câu chưa xác nhận —";
    if is_untranslated(target) {
        LABEL.to_owned()
    } else {
        format!("{target}\n{LABEL}")
    }
}
