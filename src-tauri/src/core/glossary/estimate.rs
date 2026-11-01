//! Read-only estimate of how many Glossary candidates an import scan would queue (FR137).
//!
//! The estimator is pure: no store, no dictionary handle. It mirrors `spawn_import_scan` chapter by
//! chapter (per-chapter threshold, dictionary probe, union by term).

use std::collections::{HashMap, HashSet};

use crate::core::store::{ReadHandle, SqlResult};

use super::scan::{DictionaryProbe, terms_passing_threshold};
use crate::core::matching::{MatchLang, fold_zh};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EstimateOutcome {
    Estimated(usize),
    Inconclusive,
    Cancelled,
}

/// What the real scan keeps out of the pending table. Its Glossary filter compares after folding
/// Traditional/Simplified forms; its `ON CONFLICT` on pending rows compares verbatim.
#[derive(Debug, Clone, Default)]
pub struct EstimateExclusions {
    pub glossary_terms: HashSet<String>,
    pub pending_terms: HashSet<String>,
}

/// `chapters` holds each Chapter's segment texts.
pub fn estimate_import_candidates(
    chapters: &[Vec<String>],
    lang: MatchLang,
    threshold: u32,
    surnames: &[char],
    excluded: &EstimateExclusions,
    probe: &mut dyn FnMut(&str) -> DictionaryProbe,
    is_cancelled: &mut dyn FnMut() -> bool,
) -> EstimateOutcome {
    let folded_glossary: HashSet<String> = excluded
        .glossary_terms
        .iter()
        .map(|term| fold_zh(term))
        .collect();
    let mut queued: HashSet<String> = HashSet::new();
    let mut probed: HashMap<String, DictionaryProbe> = HashMap::new();

    for segments in chapters {
        if is_cancelled() {
            return EstimateOutcome::Cancelled;
        }
        let refs: Vec<&str> = segments.iter().map(String::as_str).collect();
        let Some(terms) = terms_passing_threshold(&refs, lang, threshold, surnames, is_cancelled)
        else {
            return EstimateOutcome::Cancelled;
        };
        for term in terms {
            if is_cancelled() {
                return EstimateOutcome::Cancelled;
            }
            let verdict = *probed.entry(term.clone()).or_insert_with(|| probe(&term));
            match verdict {
                DictionaryProbe::Known => continue,
                DictionaryProbe::Missing => {}
                DictionaryProbe::Inconclusive => return EstimateOutcome::Inconclusive,
            }
            if !excluded.pending_terms.contains(&term) && !folded_glossary.contains(&fold_zh(&term))
            {
                queued.insert(term);
            }
        }
    }
    EstimateOutcome::Estimated(queued.len())
}

/// `source_term` of every `glossary_entry` row in one tier.
pub fn entry_source_terms(conn: ReadHandle<'_>) -> SqlResult<HashSet<String>> {
    let mut stmt = conn.prepare("SELECT source_term FROM glossary_entry")?;
    let rows = stmt.query_map([], |row| row.get::<_, String>(0))?;
    rows.collect()
}

/// `source_term` of every import-scan row in the Work's pending table, decided or not: the real
/// scan's `ON CONFLICT DO NOTHING` skips all of them.
pub fn import_scan_source_terms(conn: ReadHandle<'_>) -> SqlResult<HashSet<String>> {
    let mut stmt = conn.prepare(
        "SELECT source_term FROM glossary_candidate WHERE candidate_origin = 'import_scan'",
    )?;
    let rows = stmt.query_map([], |row| row.get::<_, String>(0))?;
    rows.collect()
}
