use unicode_normalization::UnicodeNormalization;
use unicode_normalization::char::canonical_combining_class;

/// NFC characters of context kept on each side of a remembered match (AD-67 rule 9). It may grow,
/// never shrink: a shorter context would stop matching rows already stored.
pub const CONVENTION_CONTEXT_CHARS: usize = 32;

/// What the "not an error" memory stores and matches on. `phrase` is for display only and takes
/// no part in matching; the rule key already carries the case-insensitive match.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConventionSignature {
    pub rule_key: String,
    pub phrase: String,
    pub before: String,
    pub after: String,
}

/// The NFC characters of one sentence with the UTF-16 offset of the cluster each came from, so
/// the context of any number of matches costs one normalisation pass.
struct ContextIndex {
    chars: Vec<char>,
    utf16_start: Vec<u32>,
}

impl ContextIndex {
    fn new(text: &str) -> Self {
        let mut chars = Vec::with_capacity(text.len());
        let mut utf16_start = Vec::with_capacity(text.len());
        let mut utf16_pos = 0u32;
        let mut iter = text.char_indices().peekable();
        while let Some((start, first)) = iter.next() {
            let mut end = start + first.len_utf8();
            let mut width = u32::try_from(first.len_utf16()).unwrap_or(2);
            while let Some(&(at, next)) = iter.peek() {
                if canonical_combining_class(next) == 0 {
                    break;
                }
                end = at + next.len_utf8();
                width += u32::try_from(next.len_utf16()).unwrap_or(2);
                iter.next();
            }
            for c in text[start..end].nfc() {
                chars.push(c);
                utf16_start.push(utf16_pos);
            }
            utf16_pos += width;
        }
        Self { chars, utf16_start }
    }

    fn boundary(&self, utf16: u32) -> usize {
        self.utf16_start.partition_point(|&at| at < utf16)
    }

    fn signature(&self, start: u32, end: u32, rule_key: &str) -> Option<ConventionSignature> {
        let (from, to) = (self.boundary(start), self.boundary(end));
        if from >= to {
            return None;
        }
        let phrase: String = self.chars[from..to].iter().collect();
        let phrase = phrase.trim().to_owned();
        if phrase.is_empty() {
            return None;
        }
        let before = self.chars[from.saturating_sub(CONVENTION_CONTEXT_CHARS)..from]
            .iter()
            .collect();
        let after = self.chars[to..(to + CONVENTION_CONTEXT_CHARS).min(self.chars.len())]
            .iter()
            .collect();
        Some(ConventionSignature {
            rule_key: rule_key.to_owned(),
            phrase,
            before,
            after,
        })
    }
}

/// The signature of `text[start..end]` (UTF-16) for `rule_key`; `None` when the range holds no
/// text after `trim`.
pub fn signature_of(
    text: &str,
    start_utf16: u32,
    end_utf16: u32,
    rule_key: &str,
) -> Option<ConventionSignature> {
    ContextIndex::new(text).signature(start_utf16, end_utf16, rule_key)
}

/// Whether a stored signature silences `current`. A stored side that is empty only matches an
/// empty current side; a non-empty one matches when it is the tail (before) or head (after) of
/// the current side, so merging segments, which only lengthens context, keeps the memory.
pub fn signature_matches(stored: &ConventionSignature, current: &ConventionSignature) -> bool {
    if stored.rule_key != current.rule_key {
        return false;
    }
    let side = |stored: &str, current: &str, ends: bool| {
        if stored.is_empty() {
            current.is_empty()
        } else if ends {
            current.ends_with(stored)
        } else {
            current.starts_with(stored)
        }
    };
    side(&stored.before, &current.before, true) && side(&stored.after, &current.after, false)
}

/// Drops the matches of `text` that `ignored` silences and counts them. `locate` gives a match as
/// `(start_utf16, end_utf16, rule_key)`. With nothing remembered no normalisation happens.
pub fn filter_ignored<'k, T>(
    text: &str,
    matches: Vec<T>,
    ignored: &[ConventionSignature],
    locate: impl Fn(&T) -> (u32, u32, &'k str),
) -> (Vec<T>, u32) {
    if ignored.is_empty() || matches.is_empty() {
        return (matches, 0);
    }
    let index = ContextIndex::new(text);
    let mut dropped = 0u32;
    let kept = matches
        .into_iter()
        .filter(|found| {
            let (start, end, key) = locate(found);
            let silenced = index
                .signature(start, end, key)
                .is_some_and(|current| ignored.iter().any(|s| signature_matches(s, &current)));
            if silenced {
                dropped += 1;
            }
            !silenced
        })
        .collect();
    (kept, dropped)
}
