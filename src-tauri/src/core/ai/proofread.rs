//! Reply parsing and quote placement for the single-segment spelling/grammar scan (FR83).
//!
//! The model returns quoted phrases, never offsets: Rust finds each phrase in the scanned text
//! and computes the UTF-16 offsets a JS `Range` needs (AD-1).

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FindingKind {
    Spelling,
    Grammar,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ProofreadFinding {
    pub kind: FindingKind,
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
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProofreadReplyError {
    Malformed,
}

#[derive(Deserialize)]
struct RawFinding {
    kind: FindingKind,
    quote: String,
    #[serde(default)]
    explanation: String,
    #[serde(default)]
    suggestion: String,
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

/// Parses `reply` and places every quoted phrase in `scanned_text`.
///
/// A phrase occurring more than once is placed at its first occurrence not already claimed by an
/// earlier finding. A phrase that cannot be placed is counted in `unlocated`, never dropped
/// silently. Findings come back ordered by position.
pub fn locate_findings(
    reply: &str,
    scanned_text: &str,
) -> Result<LocatedFindings, ProofreadReplyError> {
    let raw: Vec<RawFinding> = serde_json::from_str(strip_code_fence(reply))
        .map_err(|_| ProofreadReplyError::Malformed)?;

    let mut claimed: Vec<(usize, usize)> = Vec::new();
    let mut findings = Vec::new();
    let mut unlocated = 0u32;

    for finding in raw {
        let found = if finding.quote.is_empty() {
            None
        } else {
            scanned_text
                .match_indices(finding.quote.as_str())
                .map(|(start, quote)| (start, start + quote.len()))
                .find(|&(start, end)| !claimed.iter().any(|&(c_start, c_end)| start < c_end && c_start < end))
        };
        let Some((start, end)) = found else {
            unlocated += 1;
            continue;
        };
        claimed.push((start, end));
        findings.push(ProofreadFinding {
            kind: finding.kind,
            start: utf16_len(&scanned_text[..start]),
            end: utf16_len(&scanned_text[..end]),
            explanation: finding.explanation,
            suggestion: finding.suggestion,
        });
    }

    findings.sort_by_key(|f| (f.start, f.end));
    Ok(LocatedFindings { findings, unlocated })
}

fn utf16_len(text: &str) -> u32 {
    u32::try_from(text.encode_utf16().count()).unwrap_or(u32::MAX)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn reply(items: &str) -> String {
        format!("[{items}]")
    }

    #[test]
    fn a_phrase_is_placed_with_utf16_offsets_after_astral_and_combining_text() {
        let text = "😀 cafe\u{301} wrogn";
        let r = reply(r#"{"kind":"spelling","quote":"wrogn","explanation":"e","suggestion":"wrong"}"#);
        let got = locate_findings(&r, text).expect("valid reply");
        assert_eq!(got.unlocated, 0);
        let f = &got.findings[0];
        let units: Vec<u16> = text.encode_utf16().collect();
        let quote_units: Vec<u16> = "wrogn".encode_utf16().collect();
        assert_eq!(&units[f.start as usize..f.end as usize], quote_units.as_slice());
        assert_eq!(f.kind, FindingKind::Spelling);
        assert_eq!(f.suggestion, "wrong");
    }

    #[test]
    fn a_repeated_phrase_takes_the_first_occurrence_not_claimed_by_an_earlier_finding() {
        let text = "aa bb aa";
        let r = reply(
            r#"{"kind":"spelling","quote":"aa","explanation":"","suggestion":""},
               {"kind":"grammar","quote":"aa","explanation":"","suggestion":""}"#,
        );
        let got = locate_findings(&r, text).expect("valid reply");
        let spans: Vec<(u32, u32)> = got.findings.iter().map(|f| (f.start, f.end)).collect();
        assert_eq!(spans, vec![(0, 2), (6, 8)]);
    }

    #[test]
    fn a_phrase_absent_from_the_text_is_counted_as_unlocated() {
        let r = reply(
            r#"{"kind":"spelling","quote":"zzz","explanation":"","suggestion":""},
               {"kind":"spelling","quote":"","explanation":"","suggestion":""},
               {"kind":"grammar","quote":"bb","explanation":"","suggestion":""}"#,
        );
        let got = locate_findings(&r, "aa bb").expect("valid reply");
        assert_eq!(got.unlocated, 2);
        assert_eq!(got.findings.len(), 1);
    }

    #[test]
    fn a_third_occurrence_beyond_the_available_ones_is_unlocated() {
        let r = reply(
            r#"{"kind":"spelling","quote":"aa","explanation":"","suggestion":""},
               {"kind":"spelling","quote":"aa","explanation":"","suggestion":""}"#,
        );
        let got = locate_findings(&r, "aa").expect("valid reply");
        assert_eq!((got.findings.len(), got.unlocated), (1, 1));
    }

    #[test]
    fn an_empty_array_is_a_valid_reply_with_no_findings() {
        let got = locate_findings("[]", "aa").expect("valid reply");
        assert_eq!((got.findings.len(), got.unlocated), (0, 0));
    }

    #[test]
    fn a_fenced_reply_is_unwrapped() {
        let r = "```json\n[{\"kind\":\"grammar\",\"quote\":\"aa\",\"explanation\":\"x\",\"suggestion\":\"y\"}]\n```";
        let got = locate_findings(r, "aa").expect("valid reply");
        assert_eq!(got.findings.len(), 1);
    }

    #[test]
    fn replies_off_the_schema_are_malformed() {
        for bad in [
            "not json",
            "{}",
            r#"[{"kind":"style","quote":"a","explanation":"","suggestion":""}]"#,
            r#"[{"kind":"spelling","explanation":"","suggestion":""}]"#,
            "",
        ] {
            assert_eq!(locate_findings(bad, "a"), Err(ProofreadReplyError::Malformed), "{bad}");
        }
    }
}
