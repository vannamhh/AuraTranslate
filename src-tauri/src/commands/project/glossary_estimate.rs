//! Read-only estimate of the Glossary candidates an import would queue (FR137). Writes nothing
//! to `glossary_candidate` or `glossary_entry` and never touches [`ImportScanGeneration`].

use super::*;

pub const GLOSSARY_ESTIMATE_OUTCOME_ESTIMATED: &str = "estimated";
pub const GLOSSARY_ESTIMATE_OUTCOME_INCONCLUSIVE: &str = "inconclusive";
pub const GLOSSARY_ESTIMATE_OUTCOME_CANCELLED: &str = "cancelled";

/// `count` is `None` unless `outcome` is `estimated`; an unknown number is never `0`.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct GlossaryEstimateWire {
    pub outcome: &'static str,
    pub count: Option<u64>,
}

/// Pipeline inputs of the pending source, as the confirm step would pass them.
pub struct GlossaryEstimateSource {
    pub shape: PipelineShape,
    pub encoding: &'static encoding_rs::Encoding,
    pub source_lang: String,
    pub cleanup_rules: Vec<CleanupRule>,
    pub chapter_pattern: Option<ChapterPattern>,
    pub block_overrides: Vec<Option<bool>>,
}

/// `excluded` is the Glossary scope of the destination (plus its pending import-scan terms on an
/// append).
pub fn estimate_import_glossary(
    source: GlossaryEstimateSource,
    excluded: &std::collections::HashSet<String>,
    config: &crate::core::scope::GlobalConfig,
    layers: Option<&crate::core::dict::DictLayers>,
    is_cancelled: &mut dyn FnMut() -> bool,
) -> Result<GlossaryEstimateWire, IpcError> {
    let extract_main_content = matches!(
        &source.shape,
        PipelineShape::Chapters(cs) if matches!(cs.first(), Some(ChapterInput::RawBytes { .. }))
    );
    let lang = crate::core::glossary::match_lang_for_source_lang(&source.source_lang);
    let outcome = run_pipeline(
        PipelineInput::with_encoding(source.shape, source.encoding, source.source_lang)
            .with_cleanup_rules(source.cleanup_rules)
            .with_chapter_pattern(source.chapter_pattern)
            .with_extract_main_content(extract_main_content)
            .with_block_overrides(source.block_overrides),
    )?;
    let chapters: Vec<Vec<String>> = outcome
        .chapters
        .iter()
        .map(|chapter| chapter.segments.iter().map(|s| s.text.clone()).collect())
        .collect();

    let disabled = config.disabled_source_codes();
    let mut probe_dictionary = |term: &str| {
        let Some(layers) = layers else {
            return crate::core::glossary::DictionaryProbe::Inconclusive;
        };
        let result = crate::core::dict::lookup_grouped(
            layers,
            term,
            crate::core::dict::LookupMode::Exact,
            1,
            &disabled,
        );
        dictionary_probe_from_grouped(&result)
    };
    let estimate = crate::core::glossary::estimate_import_candidates(
        &chapters,
        lang,
        config.glossary_scan_threshold(),
        crate::core::glossary::COMMON_SURNAMES,
        excluded,
        &mut probe_dictionary,
        is_cancelled,
    );
    let (outcome, count) = match estimate {
        crate::core::glossary::EstimateOutcome::Estimated(n) => (
            GLOSSARY_ESTIMATE_OUTCOME_ESTIMATED,
            Some(u64::try_from(n).unwrap_or(u64::MAX)),
        ),
        crate::core::glossary::EstimateOutcome::Inconclusive => {
            (GLOSSARY_ESTIMATE_OUTCOME_INCONCLUSIVE, None)
        }
        crate::core::glossary::EstimateOutcome::Cancelled => {
            (GLOSSARY_ESTIMATE_OUTCOME_CANCELLED, None)
        }
    };
    Ok(GlossaryEstimateWire { outcome, count })
}

/// Retires an in-flight estimate when `next()` is called again; same shape as
/// [`ImportScanGeneration`] but a separate counter, so an estimate can never cancel a real scan.
#[derive(Debug, Clone, Default)]
pub struct ImportEstimateGeneration(Arc<AtomicU64>);

impl ImportEstimateGeneration {
    pub fn next(&self) -> u64 {
        self.0.fetch_add(1, Ordering::AcqRel).wrapping_add(1)
    }

    pub fn is_current(&self, generation: u64) -> bool {
        self.0.load(Ordering::Acquire) == generation
    }
}
