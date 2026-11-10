use super::*;
use crate::core::convention::{RuleRow, apply_spans, effective_row, rule_key, transfer_case};

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

/// What one convention write left on a sentence.
struct SentenceWritten {
    pair_origin: crate::core::tm::PairOrigin,
    status: String,
    confirmed: bool,
    chapter_started: bool,
}

/// The write every convention splice shares, inside the caller's transaction: refuses an empty
/// result (with `empty`'s error) and an unarbitrable origin, then writes `new_text` by the
/// sentence's status. Returns `Ok(Err(..))` before anything is written, so a batch can discard
/// the transaction.
fn write_convention_sentence(
    tx: &Transaction<'_>,
    segment_id: i64,
    checked: &CheckedProofreadRange,
    new_text: &str,
    empty: fn(i64) -> IpcError,
) -> SqlResult<Result<SentenceWritten, IpcError>> {
    if new_text.trim().is_empty() {
        return Ok(Err(empty(segment_id)));
    }
    let pair_origin = match arbitrate_before_write(checked, segment_id) {
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
        new_text,
        pair_origin,
        &source_text,
        confirmed,
    )?;
    let chapter_started = crate::commands::lifecycle::mark_segment_chapter_started(tx, segment_id)?;
    Ok(Ok(SentenceWritten {
        pair_origin,
        status,
        confirmed,
        chapter_started,
    }))
}

fn empty_single(segment_id: i64) -> IpcError {
    convention_error(
        "convention.text_would_be_empty",
        MessageKey::ConventionTextWouldBeEmpty,
        segment_id,
    )
}

fn empty_batch(segment_id: i64) -> IpcError {
    convention_error(
        "convention.empty_result",
        MessageKey::ConventionEmptyResult,
        segment_id,
    )
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
        let new_text = apply_spans(
            &checked.text,
            &[(checked.byte_start, checked.byte_end, &replacement)],
        )
        .unwrap_or_default();
        let written =
            match write_convention_sentence(tx, segment_id, &checked, &new_text, empty_single)? {
                Ok(written) => written,
                Err(err) => return Ok(Err(err)),
            };
        Ok(Ok(ConventionWriteOutcome {
            segment_id,
            target_text: new_text,
            translation_origin: written.pair_origin.as_str().to_owned(),
            status: written.status,
            chapter_started: written.chapter_started,
            start,
            end: start + inserted_len,
            phrase: replacement,
            version_created: written.confirmed,
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

/// One match the user kept selected, as the scan reported it.
#[derive(Debug, Clone, serde::Deserialize)]
pub struct ConventionReplaceSpot {
    pub start: u32,
    pub end: u32,
    pub expected_phrase: String,
    pub rule_key: String,
}

/// The selected spots of one sentence, ascending by `start`.
#[derive(Debug, Clone, serde::Deserialize)]
pub struct ConventionReplaceGroup {
    pub segment_id: i64,
    pub expected_text: String,
    pub spots: Vec<ConventionReplaceSpot>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct ConventionReplacedSpan {
    pub start: u32,
    pub end: u32,
    pub phrase: String,
}

/// One sentence as stored after the batch; `spans` are the inserted phrases in the new text.
#[derive(Debug, Clone, serde::Serialize)]
pub struct ConventionReplacedSentence {
    pub segment_id: i64,
    pub target_text: String,
    pub translation_origin: String,
    pub status: String,
    pub version_created: bool,
    pub spans: Vec<ConventionReplacedSpan>,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct ConventionReplaceOutcome {
    pub outcomes: Vec<ConventionReplacedSentence>,
    pub replaced: u32,
    pub sentences: u32,
    pub confirmed: u32,
    pub drafts: u32,
    pub chapter_started: bool,
}

struct PlannedSentence {
    segment_id: i64,
    checked: CheckedProofreadRange,
    new_text: String,
    spans: Vec<ConventionReplacedSpan>,
    spots: u32,
}

fn chapter_changed(segment_ids: &[i64]) -> IpcError {
    IpcError::new(
        "convention.chapter_changed",
        MessageKey::ConventionChapterChanged,
        BTreeMap::from([(
            "segment_ids".to_owned(),
            segment_ids
                .iter()
                .map(i64::to_string)
                .collect::<Vec<_>>()
                .join(", "),
        )]),
        false,
    )
}

fn rule_changed() -> IpcError {
    IpcError::new(
        "convention.rule_changed",
        MessageKey::ConventionRuleChanged,
        BTreeMap::new(),
        false,
    )
}

/// The rule in force for each distinct `rule_key` the groups name, resolved across both tiers
/// from the Global rows read just before the write and the Work rows read in `tx`. One that is
/// gone, disabled or unknown refuses the whole batch as `convention.rule_changed`.
fn resolve_convention_rules(
    tx: &Transaction<'_>,
    global_rows: &[RuleRow],
    groups: &[ConventionReplaceGroup],
) -> SqlResult<Result<BTreeMap<String, RuleRow>, IpcError>> {
    let work_rows = crate::core::convention::list_rules_in(tx)?;
    let mut rules = BTreeMap::new();
    for key in groups
        .iter()
        .flat_map(|group| group.spots.iter().map(|spot| spot.rule_key.as_str()))
    {
        if rules.contains_key(key) {
            continue;
        }
        let Some(rule) = effective_row(key, global_rows, &work_rows) else {
            return Ok(Err(rule_changed()));
        };
        rules.insert(key.to_owned(), rule);
    }
    Ok(Ok(rules))
}

/// Checks one sentence of the batch against the stored row and builds its new text; nothing is
/// written. `Err((segment_id, err))` names the sentence that fails.
fn plan_sentence(
    tx: &Transaction<'_>,
    open_chapter_id: i64,
    group: &ConventionReplaceGroup,
    rules: &BTreeMap<String, RuleRow>,
) -> SqlResult<Result<PlannedSentence, IpcError>> {
    let segment_id = group.segment_id;
    if group.spots.is_empty() {
        return Ok(Err(as_convention_error(
            range_invalid(segment_id),
            segment_id,
        )));
    }
    let omitted: i64 = match tx.query_row(
        "SELECT is_omitted FROM segment WHERE id = ?1",
        [segment_id],
        |row| row.get(0),
    ) {
        Ok(value) => value,
        Err(SqlError::QueryReturnedNoRows) => return Ok(Err(segment_not_found(segment_id))),
        Err(err) => return Err(err),
    };
    let mut checked_first: Option<CheckedProofreadRange> = None;
    let mut cased = Vec::with_capacity(group.spots.len());
    for spot in &group.spots {
        let Some(rule) = rules.get(&spot.rule_key) else {
            return Ok(Err(rule_changed()));
        };
        let Some(replacement) = rule.replacement_phrase.as_deref() else {
            return Ok(Err(rule_changed()));
        };
        if rule_key(&spot.expected_phrase) != rule.rule_key {
            return Ok(Err(rule_changed()));
        }
        let checked = match check_proofread_range(
            tx,
            open_chapter_id,
            segment_id,
            &group.expected_text,
            spot.start,
            spot.end,
            &spot.expected_phrase,
        )? {
            Ok(checked) => checked,
            Err(err) => return Ok(Err(as_convention_error(err, segment_id))),
        };
        cased.push((
            checked.byte_start,
            checked.byte_end,
            transfer_case(&spot.expected_phrase, replacement),
        ));
        checked_first.get_or_insert(checked);
    }
    let Some(checked) = checked_first else {
        return Ok(Err(as_convention_error(
            range_invalid(segment_id),
            segment_id,
        )));
    };
    if omitted != 0 {
        return Ok(Err(convention_error(
            "convention.text_changed",
            MessageKey::ConventionTextChanged,
            segment_id,
        )));
    }
    let borrowed: Vec<(usize, usize, &str)> = cased
        .iter()
        .map(|(start, end, text)| (*start, *end, text.as_str()))
        .collect();
    let Some(new_text) = apply_spans(&checked.text, &borrowed) else {
        return Ok(Err(as_convention_error(
            range_invalid(segment_id),
            segment_id,
        )));
    };
    if let Err(err) = arbitrate_before_write(&checked, segment_id) {
        return Ok(Err(as_convention_error(err, segment_id)));
    }
    let mut shift = 0i64;
    let mut spans = Vec::with_capacity(group.spots.len());
    for (spot, (_, _, phrase)) in group.spots.iter().zip(&cased) {
        let len = i64::try_from(phrase.encode_utf16().count()).unwrap_or(i64::MAX);
        let start = i64::from(spot.start) + shift;
        spans.push(ConventionReplacedSpan {
            start: u32::try_from(start).unwrap_or(u32::MAX),
            end: u32::try_from(start + len).unwrap_or(u32::MAX),
            phrase: phrase.clone(),
        });
        shift += len - i64::from(spot.end - spot.start);
    }
    Ok(Ok(PlannedSentence {
        segment_id,
        checked,
        new_text,
        spans,
        spots: u32::try_from(group.spots.len()).unwrap_or(u32::MAX),
    }))
}

/// Replaces every selected spot of the open Chapter in ONE transaction (AD-67 rules 7, 12b): every
/// sentence is checked before any is written, and one that fails any check refuses the whole batch
/// with 0 rows changed. Text changes (including a sentence omitted since the scan) are reported
/// together as `convention.chapter_changed`; any other refusal returns the first one found.
/// `global_rows` is the Global rule list read by the shell just before the write; the Work half of
/// the rule check runs inside the transaction.
pub fn convention_replace_chapter(
    open: Option<&OpenWork>,
    groups: &[ConventionReplaceGroup],
    global_rows: &[RuleRow],
) -> Result<ConventionReplaceOutcome, IpcError> {
    let open = open.ok_or_else(crate::commands::chapter::no_work_open)?;
    if groups.is_empty() {
        return Ok(ConventionReplaceOutcome {
            outcomes: Vec::new(),
            replaced: 0,
            sentences: 0,
            confirmed: 0,
            drafts: 0,
            chapter_started: false,
        });
    }
    let open_chapter_id = open.chapter_id;
    let groups = groups.to_vec();
    let global_rows = global_rows.to_vec();
    let reject: Arc<Mutex<Option<IpcError>>> = Arc::new(Mutex::new(None));
    let reject_in = Arc::clone(&reject);

    let outcome = open.store.write(move |tx: &Transaction<'_>| {
        let set_reject = |err: IpcError| {
            *reject_in
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner) = Some(err);
        };
        let rules = match resolve_convention_rules(tx, &global_rows, &groups)? {
            Ok(rules) => rules,
            Err(err) => {
                set_reject(err);
                return Err(SqlError::QueryReturnedNoRows);
            }
        };
        let mut planned = Vec::with_capacity(groups.len());
        let mut changed: Vec<i64> = Vec::new();
        let mut other: Option<IpcError> = None;
        for group in &groups {
            match plan_sentence(tx, open_chapter_id, group, &rules)? {
                Ok(sentence) => planned.push(sentence),
                Err(err) if err.code() == "convention.text_changed" => {
                    changed.push(group.segment_id);
                }
                Err(err) => {
                    other.get_or_insert(err);
                }
            }
        }
        if !changed.is_empty() {
            set_reject(chapter_changed(&changed));
            return Err(SqlError::QueryReturnedNoRows);
        }
        if let Some(err) = other {
            set_reject(err);
            return Err(SqlError::QueryReturnedNoRows);
        }

        let mut outcomes = Vec::with_capacity(planned.len());
        let (mut replaced, mut confirmed, mut drafts) = (0u32, 0u32, 0u32);
        let mut chapter_started = false;
        for sentence in planned {
            let written = match write_convention_sentence(
                tx,
                sentence.segment_id,
                &sentence.checked,
                &sentence.new_text,
                empty_batch,
            )? {
                Ok(written) => written,
                Err(err) => {
                    set_reject(err);
                    return Err(SqlError::QueryReturnedNoRows);
                }
            };
            replaced += sentence.spots;
            if written.confirmed {
                confirmed += 1;
            } else {
                drafts += 1;
            }
            chapter_started |= written.chapter_started;
            outcomes.push(ConventionReplacedSentence {
                segment_id: sentence.segment_id,
                target_text: sentence.new_text,
                translation_origin: written.pair_origin.as_str().to_owned(),
                status: written.status,
                version_created: written.confirmed,
                spans: sentence.spans,
            });
        }
        let sentences = u32::try_from(outcomes.len()).unwrap_or(u32::MAX);
        Ok(ConventionReplaceOutcome {
            outcomes,
            replaced,
            sentences,
            confirmed,
            drafts,
            chapter_started,
        })
    });

    match outcome {
        Ok(done) => Ok(done),
        Err(err) => {
            let taken = reject
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .take();
            Err(taken.unwrap_or_else(|| err.into()))
        }
    }
}
