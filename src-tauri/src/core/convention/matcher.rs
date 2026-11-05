use aho_corasick::{AhoCorasick, MatchKind};

use super::normalize::{Normalized, normalize};
use super::rule::Rule;

/// A match on the stored string, in UTF-16 units, and the rule it belongs to.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Hit {
    pub start: u32,
    pub end: u32,
    pub rule: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Scan {
    pub hits: Vec<Hit>,
    /// Matches that lost to a longer (or earlier-keyed) rule over the same text.
    pub overlapped: u32,
}

/// Built once from the resolved rules and reused for every sentence. Rules are ordered by key,
/// so equal inputs give equal matchers.
pub struct Matcher {
    automaton: Option<AhoCorasick>,
    rules: Vec<Rule>,
    key_chars: Vec<usize>,
}

impl Matcher {
    pub fn build(mut rules: Vec<Rule>) -> Self {
        rules.retain(|rule| !rule.key.is_empty());
        rules.sort_by(|a, b| a.key.cmp(&b.key));
        rules.dedup_by(|a, b| a.key == b.key);
        let key_chars = rules.iter().map(|rule| rule.key.chars().count()).collect();
        let automaton = (!rules.is_empty())
            .then(|| {
                AhoCorasick::builder()
                    .match_kind(MatchKind::Standard)
                    .build(rules.iter().map(|rule| rule.key.as_bytes()))
                    .ok()
            })
            .flatten();
        Self {
            automaton,
            rules,
            key_chars,
        }
    }

    pub fn rule(&self, index: usize) -> &Rule {
        &self.rules[index]
    }

    pub fn len(&self) -> usize {
        self.rules.len()
    }

    pub fn is_empty(&self) -> bool {
        self.rules.is_empty()
    }

    /// Matches over `text` as stored. Spans are UTF-16 offsets into `text` itself, whatever its
    /// normalisation form (AD-67 rule 3). A match starts and ends at the text edge or next to a
    /// char that is neither a letter nor a digit.
    pub fn scan(&self, text: &str) -> Scan {
        let Some(automaton) = &self.automaton else {
            return Scan::default();
        };
        let Normalized {
            text: haystack,
            chars,
        } = normalize(text);
        let char_at = |byte: usize| -> usize {
            let byte = u32::try_from(byte).unwrap_or(u32::MAX);
            chars.partition_point(|entry| entry.byte < byte)
        };
        let mut candidates: Vec<(usize, usize, usize)> = Vec::new();
        for found in automaton.find_overlapping_iter(haystack.as_str()) {
            let first = char_at(found.start());
            let after = char_at(found.end());
            let before_ok = first == 0
                || haystack[..found.start()]
                    .chars()
                    .next_back()
                    .is_some_and(|c| !c.is_alphanumeric());
            let after_ok = haystack[found.end()..]
                .chars()
                .next()
                .is_none_or(|c| !c.is_alphanumeric());
            if before_ok && after_ok && after > first {
                candidates.push((found.start(), found.end(), found.pattern().as_usize()));
            }
        }
        candidates.sort_by(|a, b| {
            self.key_chars[b.2]
                .cmp(&self.key_chars[a.2])
                .then_with(|| self.rules[a.2].key.cmp(&self.rules[b.2].key))
                .then_with(|| a.0.cmp(&b.0))
        });
        let mut accepted: Vec<(usize, usize, usize)> = Vec::new();
        let mut overlapped = 0u32;
        for candidate in candidates {
            if accepted
                .iter()
                .any(|kept| candidate.0 < kept.1 && kept.0 < candidate.1)
            {
                overlapped += 1;
            } else {
                accepted.push(candidate);
            }
        }
        accepted.sort_by_key(|hit| hit.0);
        let hits = accepted
            .into_iter()
            .map(|(from, to, rule)| Hit {
                start: chars[char_at(from)].utf16_start,
                end: chars[char_at(to) - 1].utf16_end,
                rule,
            })
            .collect();
        Scan { hits, overlapped }
    }
}

/// UTF-16 ranges of `phrase` inside `text`, compared on NFC and Unicode lowercase. Used to find a
/// pinned translation inside the sentence a rule also matched.
pub fn phrase_spans(text: &str, phrase: &str) -> Vec<(u32, u32)> {
    let needle = normalize(phrase).text;
    if needle.is_empty() {
        return Vec::new();
    }
    let Normalized {
        text: haystack,
        chars,
    } = normalize(text);
    haystack
        .match_indices(needle.as_str())
        .filter_map(|(at, found)| {
            let byte = |b: usize| u32::try_from(b).unwrap_or(u32::MAX);
            let first = chars.partition_point(|entry| entry.byte < byte(at));
            let last = chars.partition_point(|entry| entry.byte < byte(at + found.len()));
            (last > first).then(|| (chars[first].utf16_start, chars[last - 1].utf16_end))
        })
        .collect()
}
