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

/// Replaces `[start, end)` (UTF-16) of the segment's translation with `replacement`, verbatim,
/// when the segment still holds `expected_text` and that range still holds `expected_phrase`
/// (AD-53 rules 2-4). Accepting a suggestion and reverting one are this call with the two
/// phrases swapped. The write is the non-user write: text, both baseline columns and
/// `translation_origin` set together by the kind's origin rule, `status` back to `draft`, no
/// `segment_version` row.
#[expect(
    clippy::too_many_arguments,
    reason = "each parameter is an independent import option; a parameter struct would only rename the list"
)]
pub fn replace_proofread_range(
    open: Option<&OpenWork>,
    segment_id: i64,
    expected_text: &str,
    start: u32,
    end: u32,
    expected_phrase: &str,
    replacement: &str,
    kind: ProofreadKind,
) -> Result<PromoteAiTranslationOutcome, IpcError> {
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
        write_non_user_target(
            tx,
            segment_id,
            &new_text,
            declared_origin,
            Some(declared_origin),
        )?;
        Ok(Ok((new_text, declared_origin)))
    })?;

    let (target_text, declared_origin) = outcome?;
    Ok(PromoteAiTranslationOutcome {
        segment_id,
        target_text,
        translation_origin: declared_origin.to_owned(),
        status: SEGMENT_STATUS_DRAFT.to_owned(),
        needs_confirmation: false,
        unsigned_draft: None,
    })
}
