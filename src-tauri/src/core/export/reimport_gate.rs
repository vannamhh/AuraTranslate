use std::collections::BTreeMap;

use super::options::MARKDOWN_ANNOTATED_MARKER;
use crate::core::docx::DocxParsed;
use crate::core::i18n::{IpcError, MessageKey};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReimportShapeError {
    PublishCopy,
    AnnotatedCopy,
}

impl From<ReimportShapeError> for IpcError {
    fn from(error: ReimportShapeError) -> Self {
        match error {
            ReimportShapeError::PublishCopy => IpcError::new(
                "export.publish_copy_not_reimportable",
                MessageKey::ExportPublishCopyNotReimportable,
                BTreeMap::new(),
                false,
            ),
            ReimportShapeError::AnnotatedCopy => IpcError::new(
                "export.annotated_copy_not_reimportable",
                MessageKey::ExportAnnotatedCopyNotReimportable,
                BTreeMap::new(),
                false,
            ),
        }
    }
}

/// The only way to obtain an alignment input: the shape of a one-block publishing copy (AD-38)
/// is rejected here, so a downstream importer cannot overwrite confirmed text with it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReviewerDocx {
    parsed: DocxParsed,
}

impl ReviewerDocx {
    pub fn admit(parsed: DocxParsed) -> Result<Self, ReimportShapeError> {
        if parsed.export_annotated {
            return Err(ReimportShapeError::AnnotatedCopy);
        }
        let is_publish_copy = parsed.tables.iter().any(|table| {
            table.rows == 1
                && table
                    .paragraphs_per_cell
                    .iter()
                    .flatten()
                    .any(|&paragraphs| paragraphs > 1)
        });
        if is_publish_copy {
            Err(ReimportShapeError::PublishCopy)
        } else {
            Ok(Self { parsed })
        }
    }

    pub fn parsed(&self) -> &DocxParsed {
        &self.parsed
    }
}

/// The `.md` counterpart of [`ReviewerDocx::admit`]: a Markdown file the exporter marked as
/// carrying sentence numbers is refused before it is parsed.
pub fn admit_markdown(text: &str) -> Result<&str, ReimportShapeError> {
    let first_line = text
        .trim_start_matches('\u{feff}')
        .lines()
        .find(|line| !line.trim().is_empty())
        .unwrap_or("");
    if first_line.trim() == MARKDOWN_ANNOTATED_MARKER {
        Err(ReimportShapeError::AnnotatedCopy)
    } else {
        Ok(text)
    }
}
