use crate::core::cleanup::CleanupRule;
use crate::core::i18n::IpcError;
use crate::core::segment::chapterpattern::ChapterPattern;
use crate::core::segment::import::{
    DocxShape, DocxSource, FileImportItem, FilesImportOutcome, ImportError,
};

use super::{
    BilingualImportEncodingPreview, FileImportBatchWire, PendingImportSourceState,
    build_file_import_batch_wire, no_pending_import_source, preview_bilingual_import,
    resolve_preview_source_lang, stash_pending_docx_import_source,
};

/// Both previews of one `.docx`; `shape` names the filled half (`text` or `bilingual`).
#[derive(Debug, Clone, serde::Serialize)]
pub struct DocxImportPreviewWire {
    pub shape: String,
    pub text: Option<FileImportBatchWire>,
    pub bilingual: Option<BilingualImportEncodingPreview>,
}

pub fn docx_shape_from_wire(shape: &str) -> Option<DocxShape> {
    match shape {
        "text" => Some(DocxShape::Text),
        "bilingual" => Some(DocxShape::Bilingual),
        _ => None,
    }
}

/// The text outcome of a lone `.docx`, built from its single read.
pub fn docx_text_outcome(source: &DocxSource) -> FilesImportOutcome {
    let (shape, sidecar) = source.text_shape();
    FilesImportOutcome {
        shape,
        docx_sidecar: Some(sidecar),
        items: vec![FileImportItem {
            path: source.label().to_owned(),
            byte_len: Some(source.byte_len),
            error: None,
        }],
    }
}

/// Builds the requested preview from `source` and, only when that succeeds, replaces the
/// pending source with the matching shape (the stash is the last step, so a refusal leaves
/// the pending source as it was). A bilingual preview needs a language chosen by hand.
pub fn preview_docx_import_shape(
    state: &PendingImportSourceState,
    source: DocxSource,
    shape: DocxShape,
    source_lang: &str,
    cleanup_rules: &[CleanupRule],
    chapter_pattern: Option<&ChapterPattern>,
    destination: Option<String>,
) -> Result<DocxImportPreviewWire, IpcError> {
    match shape {
        DocxShape::Text => {
            let outcome = docx_text_outcome(&source);
            let resolved = resolve_preview_source_lang(Some(&outcome.shape), source_lang)?;
            let (batch, _) =
                build_file_import_batch_wire(&outcome, &resolved, cleanup_rules, chapter_pattern);
            stash_pending_docx_import_source(
                state,
                outcome.shape,
                outcome.docx_sidecar,
                Some(source),
                destination,
            );
            Ok(DocxImportPreviewWire {
                shape: "text".to_owned(),
                text: Some(batch),
                bilingual: None,
            })
        }
        DocxShape::Bilingual => {
            if source_lang == super::SOURCE_LANG_AUTO {
                return Err(ImportError::BilingualNeedsSourceLang.into());
            }
            let pipeline_shape = source.bilingual_shape()?;
            let preview = preview_bilingual_import(
                &pipeline_shape,
                source_lang,
                cleanup_rules,
                chapter_pattern,
                0,
                1,
                false,
                &[],
                None,
            )?;
            stash_pending_docx_import_source(
                state,
                pipeline_shape,
                None,
                Some(source),
                destination,
            );
            Ok(DocxImportPreviewWire {
                shape: "bilingual".to_owned(),
                text: None,
                bilingual: Some(preview),
            })
        }
    }
}

/// Rebuilds the other shape from the stashed read; never touches the file path.
pub fn switch_docx_import_shape(
    state: &PendingImportSourceState,
    shape: DocxShape,
    source_lang: &str,
    cleanup_rules: &dyn Fn(Option<&str>) -> Vec<CleanupRule>,
    chapter_pattern: Option<&ChapterPattern>,
) -> Result<DocxImportPreviewWire, IpcError> {
    let (source, destination) = {
        let guard = state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let pending = guard.as_ref().ok_or_else(no_pending_import_source)?;
        let source = pending
            .docx_source
            .clone()
            .ok_or_else(no_pending_import_source)?;
        (source, pending.destination_work_id.clone())
    };
    let rules = cleanup_rules(destination.as_deref());
    preview_docx_import_shape(
        state,
        source,
        shape,
        source_lang,
        &rules,
        chapter_pattern,
        destination,
    )
}
