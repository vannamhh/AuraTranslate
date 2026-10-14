//! Read-only estimate of how many Glossary candidates an import scan would queue (FR137).
//!
//! The estimator is pure: no store, no dictionary handle. It mirrors `spawn_import_scan` chapter by
//! chapter (per-chapter threshold, dictionary probe, union by term).

use std::collections::{HashMap, HashSet};

use crate::core::store::{ReadHandle, SqlResult};

use super::scan::{DictionaryProbe, terms_passing_threshold};
use crate::core::matching::MatchLang;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EstimateOutcome {
    Estimated(usize),
    Inconclusive,
    Cancelled,
}

/// `chapters` holds each Chapter's segment texts. `excluded` is the Glossary scope (plus the
/// pending import-scan terms on an append).
pub fn estimate_import_candidates(
    chapters: &[Vec<String>],
    lang: MatchLang,
    threshold: u32,
    surnames: &[char],
    excluded: &HashSet<String>,
    probe: &mut dyn FnMut(&str) -> DictionaryProbe,
    is_cancelled: &mut dyn FnMut() -> bool,
) -> EstimateOutcome {
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
            if !excluded.contains(&term) {
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
