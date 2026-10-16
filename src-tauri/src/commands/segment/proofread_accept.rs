use super::*;
use crate::core::segment::proofread_kind::{AcceptanceOrigin, ProofreadKind};
use crate::core::segment::translation_origin::{Arbitrated, UnknownBaselineOrigin, arbitrate};
use crate::core::segment::utf16::utf16_to_byte;

fn proofread_error(code: &'static str, key: MessageKey, segment_id: i64) -> IpcError {
    IpcError::new(
        code,
        key,
        BTreeMap::from([("segment_id".to_owned(), segment_id.to_string())]),
        false,
    )
}

fn text_changed(segment_id: i64) -> IpcError {
    proofread_error(
        "ai_proofread.text_changed",
        MessageKey::AiProofreadTextChanged,
        segment_id,
    )
}

pub(crate) fn range_invalid(segment_id: i64) -> IpcError {
    proofread_error(
        "ai_proofread.range_invalid",
        MessageKey::AiProofreadRangeInvalid,
        segment_id,
    )
}

fn decision_missing(decision_id: i64) -> IpcError {
    IpcError::new(
        "ai_proofread.decision_missing",
        MessageKey::AiProofreadDecisionMissing,
        BTreeMap::from([("decision_id".to_owned(), decision_id.to_string())]),
        false,
    )
}

fn text_would_be_empty(segment_id: i64) -> IpcError {
    proofread_error(
        "ai_proofread.text_would_be_empty",
        MessageKey::AiProofreadTextWouldBeEmpty,
        segment_id,
    )
}

/// The stored row a proofread range was checked against, with the range as byte offsets.
pub(crate) struct CheckedProofreadRange {
    pub text: String,
    pub baseline_text: String,
    pub baseline_origin: String,
    pub byte_start: usize,
    pub byte_end: usize,
}

/// The one check every proofreader write shares (AD-53 rule 4), run inside the write
/// transaction. Order: retired, outside the open Chapter, text differs from `expected_text`,
/// range outside the string / empty / splitting a surrogate pair, phrase differs from
/// `expected_phrase`. Any error leaves the row untouched.
pub(crate) fn check_proofread_range(
    tx: &Transaction<'_>,
    open_chapter_id: i64,
    segment_id: i64,
    expected_text: &str,
    start: u32,
    end: u32,
    expected_phrase: &str,
) -> SqlResult<Result<CheckedProofreadRange, IpcError>> {
    let found = tx.query_row(
        "SELECT target_text, baseline_target_text, baseline_translation_origin, \
         retired_at IS NOT NULL, chapter_id FROM segment WHERE id = ?1",
        [segment_id],
        |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, bool>(3)?,
                row.get::<_, i64>(4)?,
            ))
        },
    );
    let (text, baseline_text, baseline_origin, retired, chapter_id) = match found {
        Ok(value) => value,
        Err(SqlError::QueryReturnedNoRows) => return Ok(Err(segment_not_found(segment_id))),
        Err(err) => return Err(err),
    };
    if retired {
        return Ok(Err(segment_retired(segment_id)));
    }
    if chapter_id != open_chapter_id {
        return Ok(Err(segment_not_in_open_chapter(
            segment_id,
            open_chapter_id,
        )));
    }
    if text != expected_text {
        return Ok(Err(text_changed(segment_id)));
    }
    let bounds = (start < end)
        .then(|| utf16_to_byte(&text, start).zip(utf16_to_byte(&text, end)))
        .flatten();
    let Some((byte_start, byte_end)) = bounds else {
        return Ok(Err(range_invalid(segment_id)));
    };
    if text[byte_start..byte_end] != *expected_phrase {
        return Ok(Err(text_changed(segment_id)));
    }
    Ok(Ok(CheckedProofreadRange {
        text,
        baseline_text,
        baseline_origin,
        byte_start,
        byte_end,
    }))
}

/// Result of accepting a suggestion: the text write plus the `proof_decision` row it left, which
/// the webview keeps in its undo entry and hands back to [`revert_proofread_range`] (AD-56).
#[derive(Debug, Clone, serde::Serialize)]
pub struct ProofreadAcceptOutcome {
    #[serde(flatten)]
    pub write: PromoteAiTranslationOutcome,
    pub decision_id: i64,
}

enum DecisionStep {
    Accept,
    Revert { decision_id: i64 },
}

/// Replaces `[start, end)` (UTF-16) of the segment's translation with `replacement`, verbatim,
/// when the segment still holds `expected_text` and that range still holds `expected_phrase`
/// (AD-53 rules 2-4). Accepting and reverting are this splice with the two phrases swapped. The
/// write is the non-user write: text, both baseline columns and `translation_origin` set together
/// by the kind's origin rule, `status` back to `draft`, no `segment_version` row. The
/// `proof_decision` row is inserted (accept) or deleted (revert) in the same transaction (AD-56).
#[expect(
    clippy::too_many_arguments,
    reason = "each parameter is an independent import option; a parameter struct would only rename the list"
)]
fn splice_proofread_range(
    open: Option<&OpenWork>,
    segment_id: i64,
    expected_text: &str,
    start: u32,
    end: u32,
    expected_phrase: &str,
    replacement: &str,
    kind: ProofreadKind,
    step: DecisionStep,
) -> Result<(PromoteAiTranslationOutcome, Option<i64>), IpcError> {
    let open = open.ok_or_else(crate::commands::chapter::no_work_open)?;
    let open_chapter_id = open.chapter_id;
    let (expected_text, expected_phrase, replacement) = (
        expected_text.to_owned(),
        expected_phrase.to_owned(),
        replacement.to_owned(),
    );

    let outcome = open.store.write(move |tx: &Transaction<'_>| {
        let checked = match check_proofread_range(
            tx,
            open_chapter_id,
            segment_id,
            &expected_text,
            start,
            end,
            &expected_phrase,
        )? {
            Ok(checked) => checked,
            Err(err) => return Ok(Err(err)),
        };
        let mut new_text = String::with_capacity(checked.text.len() + replacement.len());
        new_text.push_str(&checked.text[..checked.byte_start]);
        new_text.push_str(&replacement);
        new_text.push_str(&checked.text[checked.byte_end..]);
        if new_text.trim().is_empty() {
            return Ok(Err(text_would_be_empty(segment_id)));
        }
        let declared_origin = match kind.acceptance_origin() {
            AcceptanceOrigin::OtherTranslated => TRANSLATION_ORIGIN_OTHER,
            AcceptanceOrigin::ArbitratedBeforeWrite => {
                match arbitrate(
                    &checked.text,
                    &checked.baseline_text,
                    &checked.baseline_origin,
                ) {
                    Ok(Arbitrated::PairOrigin(pair_origin)) => pair_origin.as_str(),
                    Ok(Arbitrated::Unsigned) => return Ok(Err(text_changed(segment_id))),
                    Err(UnknownBaselineOrigin(value)) => {
                        return Ok(Err(segment_unknown_translation_origin(segment_id, &value)));
                    }
                }
            }
        };
        if let DecisionStep::Revert { decision_id } = step {
            let removed = tx.execute(
                "DELETE FROM proof_decision WHERE id = ?1 AND outcome = 'accepted' AND kind = ?2",
                (decision_id, kind.as_str()),
            )?;
            if removed == 0 {
                return Ok(Err(decision_missing(decision_id)));
            }
        }
        write_non_user_target(
            tx,
            segment_id,
            &new_text,
            declared_origin,
            Some(declared_origin),
        )?;
        let decision_id = if matches!(step, DecisionStep::Accept) {
            tx.execute(
                "INSERT INTO proof_decision (kind, outcome, decided_at) \
                 VALUES (?1, 'accepted', strftime('%Y-%m-%dT%H:%M:%fZ','now'))",
                [kind.as_str()],
            )?;
            Some(tx.last_insert_rowid())
        } else {
            None
        };
        Ok(Ok((new_text, declared_origin, decision_id)))
    })?;

    let (target_text, declared_origin, decision_id) = outcome?;
    Ok((
        PromoteAiTranslationOutcome {
            segment_id,
            target_text,
            translation_origin: declared_origin.to_owned(),
            status: SEGMENT_STATUS_DRAFT.to_owned(),
            needs_confirmation: false,
            unsigned_draft: None,
        },
        decision_id,
    ))
}

/// Accepts a suggestion: `expected_phrase` is the flagged phrase, `replacement` the suggestion.
/// Leaves one `accepted` `proof_decision` row and returns its id.
#[expect(
    clippy::too_many_arguments,
    reason = "each parameter is an independent import option; a parameter struct would only rename the list"
)]
pub fn accept_proofread_range(
    open: Option<&OpenWork>,
    segment_id: i64,
    expected_text: &str,
    start: u32,
    end: u32,
    expected_phrase: &str,
    replacement: &str,
    kind: ProofreadKind,
) -> Result<ProofreadAcceptOutcome, IpcError> {
    let (write, decision_id) = splice_proofread_range(
        open,
        segment_id,
        expected_text,
        start,
        end,
        expected_phrase,
        replacement,
        kind,
        DecisionStep::Accept,
    )?;
    Ok(ProofreadAcceptOutcome {
        write,
        decision_id: decision_id.ok_or_else(|| decision_missing(0))?,
    })
}

/// Reverts an accepted suggestion: `expected_phrase` is the suggestion now in the text,
/// `replacement` the original phrase. Deletes the `accepted` row `decision_id`; no such row
/// (of this `kind`) ⇒ `ai_proofread.decision_missing` and nothing is written.
#[expect(
    clippy::too_many_arguments,
    reason = "each parameter is an independent import option; a parameter struct would only rename the list"
)]
pub fn revert_proofread_range(
    open: Option<&OpenWork>,
    segment_id: i64,
    expected_text: &str,
    start: u32,
    end: u32,
    expected_phrase: &str,
    replacement: &str,
    kind: ProofreadKind,
    decision_id: i64,
) -> Result<PromoteAiTranslationOutcome, IpcError> {
    splice_proofread_range(
        open,
        segment_id,
        expected_text,
        start,
        end,
        expected_phrase,
        replacement,
        kind,
        DecisionStep::Revert { decision_id },
    )
    .map(|(write, _)| write)
}
