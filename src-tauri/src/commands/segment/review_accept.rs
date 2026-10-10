use super::*;
use crate::commands::export::alignment_error;
use crate::core::export::{acceptable_change, mark_accepted};

fn change_text_changed(segment_id: i64) -> IpcError {
    IpcError::new(
        "review.change_text_changed",
        MessageKey::ReviewChangeTextChanged,
        BTreeMap::from([("segment_id".to_owned(), segment_id.to_string())]),
        false,
    )
}

/// Takes over one change of the reviewer copy (FR94): re-reads the group, refuses it
/// (`review.change_not_acceptable`) unless it is one segment and one reviewer row, and refuses it
/// (`review.change_text_changed`, nothing written) when the segment no longer holds
/// `expected_target`, the text the user saw. The reviewer's text is written as an unconfirmed
/// `draft` with origin `other` through the non-user write, with no `segment_version` row, and the
/// decision is stored in the same transaction. A draft with text and no copy in `segment_version`
/// needs `force` after `needs_confirmation`.
pub fn review_accept_change(
    open: Option<&OpenWork>,
    chapter_id: i64,
    group_id: i64,
    expected_target: &str,
    force: bool,
) -> Result<PromoteAiTranslationOutcome, IpcError> {
    let open = open.ok_or_else(crate::commands::chapter::no_work_open)?;
    let change = acceptable_change(&open.store, chapter_id, group_id).map_err(alignment_error)?;
    if change.mine != expected_target {
        return Err(change_text_changed(change.segment_id));
    }

    enum Accepted {
        Missing,
        Retired,
        GroupChanged,
        TextChanged,
        Row(String, String, String, bool),
    }

    let segment_id = change.segment_id;
    let (expected, theirs) = (change.mine, change.theirs);
    let outcome = open.store.write(move |tx: &Transaction<'_>| {
        let found = tx.query_row(
            "SELECT target_text, translation_origin, retired_at IS NOT NULL, status FROM segment WHERE id = ?1",
            [segment_id],
            |row| Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?, row.get::<_, bool>(2)?, row.get::<_, String>(3)?)),
        );
        let (current_text, current_origin, retired, current_status) = match found {
            Ok(value) => value,
            Err(SqlError::QueryReturnedNoRows) => return Ok(Accepted::Missing),
            Err(err) => return Err(err),
        };
        if retired {
            return Ok(Accepted::Retired);
        }
        if current_text != expected {
            return Ok(Accepted::TextChanged);
        }
        let live: bool = tx.query_row(
            "SELECT EXISTS (SELECT 1 FROM review_chapter WHERE id = ?1 AND stale_at IS NULL)",
            [change.review_chapter_id],
            |row| row.get(0),
        )?;
        if !live {
            return Ok(Accepted::GroupChanged);
        }
        if !force && !current_text.is_empty() {
            let has_copy: i64 = tx.query_row(
                "SELECT EXISTS(SELECT 1 FROM segment_version WHERE segment_id = ?1 AND target_text = ?2)",
                (segment_id, &current_text),
                |row| row.get(0),
            )?;
            if has_copy == 0 {
                return Ok(Accepted::Row(current_text, current_origin, current_status, true));
            }
        }
        if !mark_accepted(tx, change.review_chapter_id, group_id, segment_id, change.row_id)? {
            return Ok(Accepted::GroupChanged);
        }
        write_non_user_target(tx, segment_id, &theirs, TRANSLATION_ORIGIN_OTHER, Some(TRANSLATION_ORIGIN_OTHER))?;
        Ok(Accepted::Row(theirs, TRANSLATION_ORIGIN_OTHER.to_owned(), SEGMENT_STATUS_DRAFT.to_owned(), false))
    })?;

    let (target_text, translation_origin, status, needs_confirmation) = match outcome {
        Accepted::Missing => return Err(segment_not_found(segment_id)),
        Accepted::Retired => return Err(segment_retired(segment_id)),
        Accepted::GroupChanged => return Err(alignment_error(crate::core::export::AlignmentError::NotAcceptable)),
        Accepted::TextChanged => return Err(change_text_changed(segment_id)),
        Accepted::Row(text, written_origin, status, ask) => (text, written_origin, status, ask),
    };
    let unsigned_draft = needs_confirmation.then(|| target_text.clone());
    Ok(PromoteAiTranslationOutcome { segment_id, target_text, translation_origin, status, needs_confirmation, unsigned_draft })
}
