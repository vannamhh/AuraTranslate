//! Reply parsing and quote placement for the batched proofread scan (FR81, FR83).
//!
//! The model returns quoted phrases, never offsets: Rust finds each phrase in the segment the
//! phrase is assigned to and computes the UTF-16 offsets a JS `Range` needs (AD-1).

use serde::{Deserialize, Serialize};
use unicode_normalization::UnicodeNormalization as _;

use crate::core::segment::proofread_kind::{ProofreadKind, SignatureScope};
use crate::core::segment::utf16::utf16_to_byte;

/// A batch closes before the source plus target characters of its segments would pass this.
pub const PROOFREAD_BATCH_CHAR_BUDGET: usize = 4_000;
/// A batch never holds more segments than this.
pub const PROOFREAD_BATCH_MAX_SEGMENTS: usize = 20;

/// Splits segments, given each one's source plus target character count, into consecutive
/// batches. A segment larger than the budget gets a batch of its own.
pub fn plan_batches(char_counts: &[usize]) -> Vec<std::ops::Range<usize>> {
    let mut batches = Vec::new();
    let mut start = 0usize;
    let mut used = 0usize;
    for (i, &size) in char_counts.iter().enumerate() {
        let count = i - start;
        if count > 0
            && (count >= PROOFREAD_BATCH_MAX_SEGMENTS || used + size > PROOFREAD_BATCH_CHAR_BUDGET)
        {
            batches.push(start..i);
            start = i;
            used = 0;
        }
        used += size;
    }
    if start < char_counts.len() {
        batches.push(start..char_counts.len());
    }
    batches
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ProofreadFinding {
    pub kind: ProofreadKind,
    /// UTF-16 code unit offsets into the scanned text, `start` inclusive, `end` exclusive.
    pub start: u32,
    pub end: u32,
    pub explanation: String,
    pub suggestion: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LocatedFindings {
    pub findings: Vec<ProofreadFinding>,
    pub unlocated: u32,
    pub ambiguous: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProofreadReplyError {
    Malformed,
}

#[derive(Deserialize)]
struct RawFinding {
    #[serde(default)]
    segment: Option<serde_json::Value>,
    kind: String,
    quote: String,
    #[serde(default)]
    explanation: String,
    #[serde(default)]
    suggestion: String,
}

impl RawFinding {
    /// 1-based position in the batch. A missing index is unambiguous only in a one-segment batch.
    fn batch_position(&self, batch_len: usize) -> Option<usize> {
        let position = match &self.segment {
            None | Some(serde_json::Value::Null) => return (batch_len == 1).then_some(0),
            Some(serde_json::Value::Number(n)) => usize::try_from(n.as_u64()?).ok()?,
            Some(serde_json::Value::String(s)) => s.trim().parse::<usize>().ok()?,
            Some(_) => return None,
        };
        (1..=batch_len).contains(&position).then(|| position - 1)
    }
}

/// Models often wrap JSON in a Markdown fence despite the instruction; unwrap one if present.
fn strip_code_fence(reply: &str) -> &str {
    let trimmed = reply.trim();
    let Some(rest) = trimmed.strip_prefix("```") else {
        return trimmed;
    };
    let rest = rest.split_once('\n').map_or("", |(_, body)| body);
    rest.trim_end().strip_suffix("```").unwrap_or(rest).trim()
}

/// Parses `reply` and places every quoted phrase in the text of the segment it names.
///
/// `segments` holds each batch segment's `(scanned_text, has_source)` in batch order; a finding
/// names its segment by 1-based position. An index outside the batch, a label outside
/// [`ProofreadKind`], or a kind judged against the source when that segment has none is counted
/// in the `unlocated` of the segment it points at, or of the first segment when the index
/// itself is unusable.
///
/// A spelling phrase occurring more than once is placed at its first occurrence not already
/// claimed by an earlier finding in the same segment. A context-scoped kind whose phrase occurs
/// more than once is not placed and is counted in `ambiguous` (AD-53 rule 5). A phrase that cannot
/// be placed is counted in `unlocated`, never dropped silently. Findings come back ordered by position.
pub fn locate_batch_findings(
    reply: &str,
    segments: &[(&str, bool)],
) -> Result<Vec<LocatedFindings>, ProofreadReplyError> {
    let raw: Vec<RawFinding> = serde_json::from_str(strip_code_fence(reply))
        .map_err(|_| ProofreadReplyError::Malformed)?;

    let mut claimed: Vec<Vec<(usize, usize)>> = vec![Vec::new(); segments.len()];
    let mut out: Vec<LocatedFindings> = segments
        .iter()
        .map(|_| LocatedFindings {
            findings: Vec::new(),
            unlocated: 0,
            ambiguous: 0,
        })
        .collect();

    for finding in raw {
        let Some(position) = finding.batch_position(segments.len()) else {
            if let Some(first) = out.first_mut() {
                first.unlocated += 1;
            }
            continue;
        };
        let (scanned_text, has_source) = segments[position];
        let kind =
            ProofreadKind::from_wire(&finding.kind).filter(|k| has_source || !k.needs_source());
        let occurrences: Vec<(usize, usize)> = if kind.is_none() || finding.quote.is_empty() {
            Vec::new()
        } else {
            scanned_text
                .match_indices(finding.quote.as_str())
                .map(|(start, quote)| (start, start + quote.len()))
                .collect()
        };
        if let Some(kind) = kind
            && kind.signature_scope() == SignatureScope::PhraseAndContext
            && occurrences.len() > 1
        {
            out[position].ambiguous += 1;
            continue;
        }
        let found = occurrences.into_iter().find(|&(start, end)| {
            !claimed[position]
                .iter()
                .any(|&(c_start, c_end)| start < c_end && c_start < end)
        });
        let (Some(kind), Some((start, end))) = (kind, found) else {
            out[position].unlocated += 1;
            continue;
        };
        claimed[position].push((start, end));
        out[position].findings.push(ProofreadFinding {
            kind,
            start: utf16_len(&scanned_text[..start]),
            end: utf16_len(&scanned_text[..end]),
            explanation: finding.explanation,
            suggestion: finding.suggestion,
        });
    }

    for located in &mut out {
        located.findings.sort_by_key(|f| (f.start, f.end));
    }
    Ok(out)
}

fn utf16_len(text: &str) -> u32 {
    u32::try_from(text.encode_utf16().count()).unwrap_or(u32::MAX)
}

/// Unicode characters of context kept on each side of a context-scoped phrase (AD-53 rule 7).
pub const PROOFREAD_CONTEXT_CHARS: usize = 32;

/// What "not an error" memory stores and matches on (AD-53 rule 7). `before` and `after` are
/// empty for a phrase-scoped kind and are not consulted when matching it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProofreadSignature {
    pub kind: ProofreadKind,
    pub phrase: String,
    pub before: String,
    pub after: String,
}

/// The signature of `text[byte_start..byte_end]`; `None` when the phrase is empty after `trim`.
pub fn signature_of(
    kind: ProofreadKind,
    text: &str,
    byte_start: usize,
    byte_end: usize,
) -> Option<ProofreadSignature> {
    let phrase: String = text.get(byte_start..byte_end)?.trim().nfc().collect();
    if phrase.is_empty() {
        return None;
    }
    if kind.signature_scope() == SignatureScope::Phrase {
        return Some(ProofreadSignature {
            kind,
            phrase,
            before: String::new(),
            after: String::new(),
        });
    }
    let before: Vec<char> = text[..byte_start].nfc().collect();
    let after: String = text[byte_end..]
        .nfc()
        .take(PROOFREAD_CONTEXT_CHARS)
        .collect();
    let before: String = before[before.len().saturating_sub(PROOFREAD_CONTEXT_CHARS)..]
        .iter()
        .collect();
    Some(ProofreadSignature {
        kind,
        phrase,
        before,
        after,
    })
}

/// Whether a stored signature silences `current`. A stored side that is empty only matches an
/// empty current side; a non-empty one matches when it is the tail (before) or head (after) of
/// the current side, so merging segments, which only lengthens context, keeps the memory.
pub fn signature_matches(stored: &ProofreadSignature, current: &ProofreadSignature) -> bool {
    if stored.kind != current.kind || stored.phrase != current.phrase {
        return false;
    }
    if stored.kind.signature_scope() == SignatureScope::Phrase {
        return true;
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

/// Drops the findings `remembered` silences and returns how many it dropped.
pub fn filter_remembered(
    scanned_text: &str,
    findings: Vec<ProofreadFinding>,
    remembered: &[ProofreadSignature],
) -> (Vec<ProofreadFinding>, u32) {
    if remembered.is_empty() {
        return (findings, 0);
    }
    let mut filtered = 0u32;
    let kept = findings
        .into_iter()
        .filter(|finding| {
            let current = utf16_to_byte(scanned_text, finding.start)
                .zip(utf16_to_byte(scanned_text, finding.end))
                .and_then(|(start, end)| signature_of(finding.kind, scanned_text, start, end));
            let silenced = current
                .is_some_and(|current| remembered.iter().any(|s| signature_matches(s, &current)));
            filtered += u32::from(silenced);
            !silenced
        })
        .collect();
    (kept, filtered)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn reply(items: &str) -> String {
        format!("[{items}]")
    }

    fn locate_findings(
        reply: &str,
        scanned_text: &str,
        has_source: bool,
    ) -> Result<LocatedFindings, ProofreadReplyError> {
        locate_batch_findings(reply, &[(scanned_text, has_source)]).map(|mut v| v.remove(0))
    }

    #[test]
    fn a_phrase_is_placed_with_utf16_offsets_after_astral_and_combining_text() {
        let text = "😀 cafe\u{301} wrogn";
        let r =
            reply(r#"{"kind":"spelling","quote":"wrogn","explanation":"e","suggestion":"wrong"}"#);
        let got = locate_findings(&r, text, true).expect("valid reply");
        assert_eq!(got.unlocated, 0);
        let f = &got.findings[0];
        let units: Vec<u16> = text.encode_utf16().collect();
        let quote_units: Vec<u16> = "wrogn".encode_utf16().collect();
        assert_eq!(
            &units[f.start as usize..f.end as usize],
            quote_units.as_slice()
        );
        assert_eq!(f.kind, ProofreadKind::Spelling);
        assert_eq!(f.suggestion, "wrong");
    }

    #[test]
    fn a_repeated_spelling_phrase_takes_the_first_occurrence_not_claimed_by_an_earlier_finding() {
        let text = "aa bb aa";
        let r = reply(
            r#"{"kind":"spelling","quote":"aa","explanation":"","suggestion":""},
               {"kind":"spelling","quote":"aa","explanation":"","suggestion":""}"#,
        );
        let got = locate_findings(&r, text, true).expect("valid reply");
        let spans: Vec<(u32, u32)> = got.findings.iter().map(|f| (f.start, f.end)).collect();
        assert_eq!(spans, vec![(0, 2), (6, 8)]);
    }

    #[test]
    fn a_repeated_context_kind_phrase_is_not_placed_and_counts_as_ambiguous() {
        let r = reply(
            r#"{"kind":"grammar","quote":"aa","explanation":"","suggestion":""},
               {"kind":"mistranslation","quote":"bb","explanation":"","suggestion":""},
               {"kind":"obscure","quote":"cc","explanation":"","suggestion":""}"#,
        );
        let got = locate_findings(&r, "aa bb aa bb cc", true).expect("valid reply");
        assert_eq!(
            (got.findings.len(), got.unlocated, got.ambiguous),
            (1, 0, 2)
        );
        assert_eq!(got.findings[0].kind, ProofreadKind::Obscure);
    }

    #[test]
    fn a_phrase_absent_from_the_text_is_counted_as_unlocated() {
        let r = reply(
            r#"{"kind":"spelling","quote":"zzz","explanation":"","suggestion":""},
               {"kind":"spelling","quote":"","explanation":"","suggestion":""},
               {"kind":"grammar","quote":"bb","explanation":"","suggestion":""}"#,
        );
        let got = locate_findings(&r, "aa bb", true).expect("valid reply");
        assert_eq!(got.unlocated, 2);
        assert_eq!(got.findings.len(), 1);
    }

    #[test]
    fn a_third_occurrence_beyond_the_available_ones_is_unlocated() {
        let r = reply(
            r#"{"kind":"spelling","quote":"aa","explanation":"","suggestion":""},
               {"kind":"spelling","quote":"aa","explanation":"","suggestion":""}"#,
        );
        let got = locate_findings(&r, "aa", true).expect("valid reply");
        assert_eq!((got.findings.len(), got.unlocated), (1, 1));
    }

    #[test]
    fn an_empty_array_is_a_valid_reply_with_no_findings() {
        let got = locate_findings("[]", "aa", true).expect("valid reply");
        assert_eq!((got.findings.len(), got.unlocated), (0, 0));
    }

    #[test]
    fn a_fenced_reply_is_unwrapped() {
        let r = "```json\n[{\"kind\":\"grammar\",\"quote\":\"aa\",\"explanation\":\"x\",\"suggestion\":\"y\"}]\n```";
        let got = locate_findings(r, "aa", true).expect("valid reply");
        assert_eq!(got.findings.len(), 1);
    }

    #[test]
    fn replies_off_the_schema_are_malformed() {
        for bad in [
            "not json",
            "{}",
            r#"[{"kind":"spelling","explanation":"","suggestion":""}]"#,
            "",
        ] {
            assert_eq!(
                locate_findings(bad, "a", true),
                Err(ProofreadReplyError::Malformed),
                "{bad}"
            );
        }
    }

    #[test]
    fn an_unknown_label_is_unlocated_and_the_rest_still_shows() {
        let r = reply(
            r#"{"kind":"style","quote":"aa","explanation":"","suggestion":""},
               {"kind":"mistranslation","quote":"bb","explanation":"","suggestion":""}"#,
        );
        let got = locate_findings(&r, "aa bb", true).expect("valid reply");
        assert_eq!(got.unlocated, 1);
        assert_eq!(got.findings.len(), 1);
        assert_eq!(got.findings[0].kind, ProofreadKind::Mistranslation);
    }

    #[test]
    fn a_source_judged_kind_without_a_source_is_unlocated() {
        let r = reply(
            r#"{"kind":"obscure","quote":"aa","explanation":"","suggestion":""},
               {"kind":"grammar","quote":"bb","explanation":"","suggestion":""}"#,
        );
        let got = locate_findings(&r, "aa bb", false).expect("valid reply");
        assert_eq!((got.findings.len(), got.unlocated), (1, 1));
        assert_eq!(got.findings[0].kind, ProofreadKind::Grammar);
    }

    #[test]
    fn a_missing_quote_is_malformed_even_with_an_unknown_label() {
        let bad = r#"[{"kind":"style","explanation":"","suggestion":""}]"#;
        assert_eq!(
            locate_findings(bad, "a", true),
            Err(ProofreadReplyError::Malformed)
        );
    }

    #[test]
    fn the_budget_closes_a_batch_and_an_oversized_segment_goes_alone() {
        assert_eq!(plan_batches(&[]), Vec::<std::ops::Range<usize>>::new());
        assert_eq!(plan_batches(&[1_500, 1_500, 1_000, 1]), vec![0..3, 3..4]);
        assert_eq!(plan_batches(&[10, 5_000, 10, 10]), vec![0..1, 1..2, 2..4]);
        assert_eq!(
            plan_batches(&[PROOFREAD_BATCH_CHAR_BUDGET, 1]),
            vec![0..1, 1..2]
        );
    }

    #[test]
    fn a_batch_holds_at_most_twenty_segments() {
        let batches = plan_batches(&[1; 45]);
        assert_eq!(batches, vec![0..20, 20..40, 40..45]);
    }

    #[test]
    fn findings_land_in_the_segment_they_name_and_a_foreign_index_is_unlocated() {
        let r = reply(
            r#"{"segment":2,"kind":"spelling","quote":"bb","explanation":"","suggestion":"B"},
               {"segment":1,"kind":"grammar","quote":"aa","explanation":"","suggestion":"A"},
               {"segment":3,"kind":"spelling","quote":"aa","explanation":"","suggestion":""},
               {"segment":0,"kind":"spelling","quote":"aa","explanation":"","suggestion":""},
               {"segment":"2","kind":"spelling","quote":"aa","explanation":"","suggestion":""},
               {"kind":"spelling","quote":"aa","explanation":"","suggestion":""}"#,
        );
        let got = locate_batch_findings(&r, &[("aa", true), ("bb", true)]).expect("valid reply");
        assert_eq!((got[0].findings.len(), got[0].unlocated), (1, 3));
        assert_eq!((got[1].findings.len(), got[1].unlocated), (1, 1));
        assert_eq!(got[1].findings[0].suggestion, "B");
    }

    #[test]
    fn a_claimed_span_is_per_segment() {
        let r = reply(
            r#"{"segment":1,"kind":"spelling","quote":"aa","explanation":"","suggestion":""},
               {"segment":2,"kind":"spelling","quote":"aa","explanation":"","suggestion":""}"#,
        );
        let got = locate_batch_findings(&r, &[("aa", true), ("aa", true)]).expect("valid reply");
        assert_eq!((got[0].findings.len(), got[1].findings.len()), (1, 1));
    }

    #[test]
    fn a_meaning_kind_is_unlocated_only_in_the_segment_without_a_source() {
        let r = reply(
            r#"{"segment":1,"kind":"mistranslation","quote":"aa","explanation":"","suggestion":""},
               {"segment":2,"kind":"mistranslation","quote":"bb","explanation":"","suggestion":""}"#,
        );
        let got = locate_batch_findings(&r, &[("aa", false), ("bb", true)]).expect("valid reply");
        assert_eq!((got[0].findings.len(), got[0].unlocated), (0, 1));
        assert_eq!((got[1].findings.len(), got[1].unlocated), (1, 0));
    }

    fn signature(kind: ProofreadKind, text: &str, phrase: &str) -> ProofreadSignature {
        let start = text.find(phrase).expect("phrase in text");
        signature_of(kind, text, start, start + phrase.len()).expect("signature")
    }

    #[test]
    fn a_signature_trims_and_normalises_the_phrase() {
        let text = "ab  cafe\u{301}  cd";
        let start = text.find("  cafe").unwrap();
        let end = text.find("  cd").unwrap() + 2;
        let sig = signature_of(ProofreadKind::Spelling, text, start, end).expect("signature");
        assert_eq!(sig.phrase, "caf\u{e9}");
        assert_eq!((sig.before.as_str(), sig.after.as_str()), ("", ""));
        assert_eq!(signature_of(ProofreadKind::Spelling, "a   b", 1, 4), None);
    }

    #[test]
    fn a_context_signature_keeps_at_most_n_characters_each_side() {
        let long = "x".repeat(PROOFREAD_CONTEXT_CHARS + 10);
        let text = format!("{long}[mid]{long}");
        let sig = signature(ProofreadKind::Grammar, &text, "[mid]");
        assert_eq!(sig.before, "x".repeat(PROOFREAD_CONTEXT_CHARS));
        assert_eq!(sig.after, "x".repeat(PROOFREAD_CONTEXT_CHARS));
    }

    #[test]
    fn a_stored_side_matches_when_it_is_the_tail_or_head_of_the_current_side() {
        let stored = signature(ProofreadKind::Grammar, "aa bb cc", "bb");
        assert!(signature_matches(
            &stored,
            &signature(ProofreadKind::Grammar, "zz aa bb cc dd", "bb")
        ));
        assert!(!signature_matches(
            &stored,
            &signature(ProofreadKind::Grammar, "zz aa bb ce dd", "bb")
        ));
        assert!(!signature_matches(
            &stored,
            &signature(ProofreadKind::Grammar, "bb cc", "bb")
        ));
        assert!(!signature_matches(
            &stored,
            &signature(ProofreadKind::Mistranslation, "aa bb cc", "bb")
        ));
    }

    #[test]
    fn an_empty_stored_side_only_matches_an_empty_current_side() {
        let at_edge = signature(ProofreadKind::Grammar, "bb cc", "bb");
        assert!(signature_matches(
            &at_edge,
            &signature(ProofreadKind::Grammar, "bb cc", "bb")
        ));
        assert!(!signature_matches(
            &at_edge,
            &signature(ProofreadKind::Grammar, "aa bb cc", "bb")
        ));
    }

    #[test]
    fn a_spelling_signature_matches_whatever_surrounds_the_phrase() {
        let stored = signature(ProofreadKind::Spelling, "aa wrogn bb", "wrogn");
        assert!(signature_matches(
            &stored,
            &signature(ProofreadKind::Spelling, "zzz wrogn", "wrogn")
        ));
        assert!(!signature_matches(
            &stored,
            &signature(ProofreadKind::Spelling, "zzz wrong", "wrong")
        ));
    }

    #[test]
    fn filtering_drops_only_remembered_findings_and_counts_them() {
        let text = "aa wrogn bb wrogn";
        let at = |start: u32| ProofreadFinding {
            kind: ProofreadKind::Spelling,
            start,
            end: start + 5,
            explanation: String::new(),
            suggestion: String::new(),
        };
        let grammar = ProofreadFinding {
            kind: ProofreadKind::Grammar,
            ..at(3)
        };
        let remembered = [signature(ProofreadKind::Spelling, text, "wrogn")];
        let (kept, filtered) =
            filter_remembered(text, vec![at(3), grammar.clone(), at(12)], &remembered);
        assert_eq!((kept, filtered), (vec![grammar], 2));
        let (all, none) = filter_remembered(text, vec![at(3)], &[]);
        assert_eq!((all.len(), none), (1, 0));
    }
}
