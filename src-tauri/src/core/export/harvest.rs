use std::collections::{BTreeMap, HashSet};

use unicode_normalization::UnicodeNormalization;

use crate::core::glossary::{GlossaryError, WorkContext, confirmed_term_translations, match_lang_for_source_lang};
use crate::core::matching::find_terms;
use crate::core::scope::ScopeResolver;
use crate::core::store::{Store, StoreError};

use super::alignment::{AlignmentError, group_texts};
use super::reviewer_copy::ReviewCopyError;

/// A confirmed Glossary term whose confirmed translation the reviewer replaced: `changed_count` of
/// the `seen_count` occurrences of `replaced_translation` became `proposed_translation`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HarvestFinding {
    pub source_term: String,
    pub replaced_translation: String,
    pub proposed_translation: String,
    pub changed_count: i64,
    pub seen_count: i64,
}

#[derive(Debug)]
pub enum HarvestError {
    Alignment(AlignmentError),
    Glossary(GlossaryError),
    Store(StoreError),
}

/// Fewest changed occurrences that make a replacement worth proposing.
pub const MIN_CHANGED_COUNT: i64 = 2;

type Phrase = Vec<String>;

/// Words of `text` in NFC, split on whitespace with the punctuation at both ends of a word cut off;
/// a token of punctuation only becomes an empty word, which no phrase can contain.
fn words(text: &str) -> Vec<String> {
    let text: String = text.nfc().collect();
    text.split_whitespace().map(|w| w.trim_matches(|c: char| !c.is_alphanumeric()).to_owned()).collect()
}

fn count_phrase(tokens: &[String], phrase: &[String]) -> usize {
    if phrase.is_empty() || tokens.len() < phrase.len() {
        return 0;
    }
    tokens.windows(phrase.len()).filter(|w| *w == phrase).count()
}

fn phrases(tokens: &[String], max_len: usize) -> HashSet<Phrase> {
    let mut out = HashSet::new();
    for len in 1..=max_len {
        for window in tokens.windows(len) {
            if window.iter().all(|w| !w.is_empty()) {
                out.insert(window.to_vec());
            }
        }
    }
    out
}

#[derive(Default)]
struct Tally {
    seen: i64,
    changed: i64,
    changed_groups: Vec<HashSet<Phrase>>,
}

/// The phrase the reviewer wrote where `replaced` stood: among phrases of the reviewer's text that
/// are not in mine, the one found in most changed groups, then sharing most words with `replaced`,
/// then the shortest.
fn best_replacement(replaced: &[String], changed_groups: &[HashSet<Phrase>]) -> Option<Phrase> {
    let mut groups_with: BTreeMap<&Phrase, usize> = BTreeMap::new();
    for group in changed_groups {
        for phrase in group {
            *groups_with.entry(phrase).or_default() += 1;
        }
    }
    groups_with
        .into_iter()
        .filter(|(phrase, _)| phrase.len() <= replaced.len() + 2)
        .max_by(|(a, a_groups), (b, b_groups)| {
            let shared = |p: &Phrase| p.iter().filter(|w| replaced.contains(w)).count();
            a_groups
                .cmp(b_groups)
                .then_with(|| shared(a).cmp(&shared(b)))
                .then_with(|| b.len().cmp(&a.len()))
                .then_with(|| b.cmp(a))
        })
        .map(|(phrase, _)| phrase.clone())
}

/// Reads every live reviewer copy of the Work and returns, per confirmed Glossary term, the
/// replacement the reviewer chose and how consistently. Writes nothing to the reviewer copy, the
/// segments or the Glossary; the machine grouping a copy still waits for is the only side effect
/// of reading the alignment.
///
/// # Errors
/// A failed read of the Glossary tiers, the alignment or the Chapter list.
pub fn harvest_work(
    resolver: &ScopeResolver,
    global: &Store,
    work: &Store,
    source_lang: &str,
) -> Result<Vec<HarvestFinding>, HarvestError> {
    let scope = WorkContext::new(Some((resolver, work))).map_err(HarvestError::Glossary)?;
    let confirmed = confirmed_term_translations(&scope, global).map_err(HarvestError::Glossary)?;
    if confirmed.is_empty() {
        return Ok(Vec::new());
    }
    let lang = match_lang_for_source_lang(source_lang);
    let terms: Vec<&str> = confirmed.iter().map(|(source, _)| source.as_str()).collect();
    let replaced: Vec<Phrase> = confirmed.iter().map(|(_, translation)| words(translation)).collect();
    let max_len = replaced.iter().map(Vec::len).max().unwrap_or(1) + 2;

    let chapter_ids = work
        .read(|conn| {
            let mut stmt = conn.prepare("SELECT chapter_id FROM review_chapter WHERE stale_at IS NULL ORDER BY chapter_id")?;
            stmt.query_map([], |row| row.get::<_, i64>(0))?.collect::<Result<Vec<_>, _>>()
        })
        .map_err(HarvestError::Store)?;

    let mut tallies: BTreeMap<usize, Tally> = BTreeMap::new();
    for chapter_id in chapter_ids {
        let (alignment, groups) = match group_texts(work, chapter_id) {
            Ok(found) => found,
            Err(AlignmentError::Copy(ReviewCopyError::Stale | ReviewCopyError::NotImported)) => continue,
            Err(error) => return Err(HarvestError::Alignment(error)),
        };
        for group in groups.iter().filter(|g| !g.segment_ids.is_empty() && !g.row_ids.is_empty()) {
            let source = alignment
                .segments
                .iter()
                .filter(|s| group.segment_ids.contains(&s.id))
                .map(|s| s.source_text.as_str())
                .collect::<Vec<_>>()
                .join(" ");
            let in_source: HashSet<usize> = find_terms(&source, &terms, lang).into_iter().map(|m| m.term_index).collect();
            if in_source.is_empty() {
                continue;
            }
            let mine = words(&group.mine);
            let theirs = words(&group.theirs);
            let mut new_phrases: Option<HashSet<Phrase>> = None;
            for index in in_source {
                let mine_count = count_phrase(&mine, &replaced[index]);
                if mine_count == 0 {
                    continue;
                }
                let theirs_count = count_phrase(&theirs, &replaced[index]);
                let tally = tallies.entry(index).or_default();
                tally.seen += i64::try_from(mine_count).unwrap_or(i64::MAX);
                if theirs_count < mine_count {
                    tally.changed += i64::try_from(mine_count - theirs_count).unwrap_or(i64::MAX);
                    let new_phrases = new_phrases.get_or_insert_with(|| {
                        let mine_phrases = phrases(&mine, max_len);
                        phrases(&theirs, max_len).into_iter().filter(|p| !mine_phrases.contains(p)).collect()
                    });
                    tally.changed_groups.push(new_phrases.clone());
                }
            }
        }
    }

    let mut findings = Vec::new();
    for (index, tally) in tallies {
        if tally.changed < MIN_CHANGED_COUNT {
            continue;
        }
        let Some(proposed) = best_replacement(&replaced[index], &tally.changed_groups) else {
            continue;
        };
        findings.push(HarvestFinding {
            source_term: confirmed[index].0.clone(),
            replaced_translation: confirmed[index].1.clone(),
            proposed_translation: proposed.join(" "),
            changed_count: tally.changed,
            seen_count: tally.seen,
        });
    }
    findings.sort_by(|a, b| (&a.source_term, &a.proposed_translation).cmp(&(&b.source_term, &b.proposed_translation)));
    Ok(findings)
}
