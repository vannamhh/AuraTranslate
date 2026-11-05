use super::*;
use crate::core::convention::{RuleRow, rule_key, transfer_case};

/// What the webview needs after a convention write: the sentence as stored, and where the
/// inserted phrase sits in it (UTF-16) for the undo tray and for shifting the other marks.
#[derive(Debug, Clone, serde::Serialize)]
pub struct ConventionWriteOutcome {
    pub segment_id: i64,
    pub target_text: String,
    pub translation_origin: String,
    pub status: String,
    pub chapter_started: bool,
    pub start: u32,
    pub end: u32,
    pub phrase: String,
    pub version_created: bool,
}

fn convention_error(code: &'static str, key: MessageKey, segment_id: i64) -> IpcError {
    IpcError::new(
        code,
        key,
        BTreeMap::from([("segment_id".to_owned(), segment_id.to_string())]),
        false,
    )
}

pub(crate) fn rule_unavailable() -> IpcError {
    IpcError::new(
        "convention.rule_unavailable",
        MessageKey::ConventionRuleUnavailable,
        BTreeMap::new(),
        false,
    )
}

/// The shared range check speaks the Proofreader's codes; the user of this write must not read a
/// Proofreader message.
fn as_convention_error(err: IpcError, segment_id: i64) -> IpcError {
    match err.code() {
        "ai_proofread.text_changed" => convention_error(
            "convention.text_changed",
            MessageKey::ConventionTextChanged,
            segment_id,
        ),
        "ai_proofread.range_invalid" => convention_error(
            "convention.range_invalid",
            MessageKey::ConventionRangeInvalid,
            segment_id,
        ),
        _ => err,
    }
}

fn splice_convention_range(
    open: Option<&OpenWork>,
    segment_id: i64,
    expected_text: &str,
    start: u32,
    end: u32,
    expected_phrase: &str,
    replacement: &str,
) -> Result<ConventionWriteOutcome, IpcError> {
    let open = open.ok_or_else(crate::commands::chapter::no_work_open)?;
    let open_chapter_id = open.chapter_id;
    let (expected_text, expected_phrase, replacement) = (
        expected_text.to_owned(),
        expected_phrase.to_owned(),
        replacement.to_owned(),
    );
    let inserted_len = u32::try_from(replacement.encode_utf16().count()).unwrap_or(u32::MAX);

    open.store.write(move |tx: &Transaction<'_>| {
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
            Err(err) => return Ok(Err(as_convention_error(err, segment_id))),
        };
        let mut new_text = String::with_capacity(checked.text.len() + replacement.len());
        new_text.push_str(&checked.text[..checked.byte_start]);
        new_text.push_str(&replacement);
        new_text.push_str(&checked.text[checked.byte_end..]);
        if new_text.trim().is_empty() {
            return Ok(Err(convention_error(
                "convention.text_would_be_empty",
                MessageKey::ConventionTextWouldBeEmpty,
                segment_id,
            )));
        }
        let pair_origin = match arbitrate_before_write(&checked, segment_id) {
            Ok(pair_origin) => pair_origin,
            Err(err) => return Ok(Err(as_convention_error(err, segment_id))),
        };
        let (status, source_text): (String, String) = tx.query_row(
            "SELECT status, source_text FROM segment WHERE id = ?1",
            [segment_id],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )?;
        let confirmed = status == SEGMENT_STATUS_CONFIRMED;
        write_convention_target(
            tx,
            segment_id,
            &new_text,
            pair_origin,
            &source_text,
            confirmed,
        )?;
        let chapter_started =
            crate::commands::lifecycle::mark_segment_chapter_started(tx, segment_id)?;
        Ok(Ok(ConventionWriteOutcome {
            segment_id,
            target_text: new_text,
            translation_origin: pair_origin.as_str().to_owned(),
            status,
            chapter_started,
            start,
            end: start + inserted_len,
            phrase: replacement,
            version_created: confirmed,
        }))
    })?
}

/// Replaces one match `[start, end)` (UTF-16) of `rule` with its replacement, spelled in the case
/// of `expected_phrase` (AD-67 rules 3, 7). `rule` is read by the command shell just before the
/// transaction (it lives in another database file, so no transaction spans both); a rule without
/// a replacement, or whose key is not that of `expected_phrase`, refuses.
pub fn convention_accept_range(
    open: Option<&OpenWork>,
    segment_id: i64,
    expected_text: &str,
    start: u32,
    end: u32,
    expected_phrase: &str,
    rule: &RuleRow,
) -> Result<ConventionWriteOutcome, IpcError> {
    if open.is_none() {
        return Err(crate::commands::chapter::no_work_open());
    }
    let Some(replacement) = rule.replacement_phrase.as_deref() else {
        return Err(rule_unavailable());
    };
    if rule_key(expected_phrase) != rule.rule_key {
        return Err(rule_unavailable());
    }
    let cased = transfer_case(expected_phrase, replacement);
    splice_convention_range(
        open,
        segment_id,
        expected_text,
        start,
        end,
        expected_phrase,
        &cased,
    )
}

/// Undoes [`convention_accept_range`]: `expected_phrase` is what the text holds now, `original_phrase`
/// what it held before. Same check, same origin rule, same sentence-state rules as accepting.
pub fn convention_revert_range(
    open: Option<&OpenWork>,
    segment_id: i64,
    expected_text: &str,
    start: u32,
    end: u32,
    expected_phrase: &str,
    original_phrase: &str,
) -> Result<ConventionWriteOutcome, IpcError> {
    splice_convention_range(
        open,
        segment_id,
        expected_text,
        start,
        end,
        expected_phrase,
        original_phrase,
    )
}
