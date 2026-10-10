use std::collections::HashMap;

use crate::core::store::{ReadHandle, SqlResult, Transaction};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReviewDecision {
    Accepted,
    Skipped,
}

impl ReviewDecision {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Accepted => "accepted",
            Self::Skipped => "skipped",
        }
    }

    fn from_column(value: &str) -> Option<Self> {
        match value {
            "accepted" => Some(Self::Accepted),
            "skipped" => Some(Self::Skipped),
            _ => None,
        }
    }
}

/// Decision of every decided reviewer row of one copy, by `review_row.id`.
pub(super) fn decisions_of_copy(conn: ReadHandle<'_>, review_chapter_id: i64) -> SqlResult<HashMap<i64, ReviewDecision>> {
    let mut stmt = conn.prepare("SELECT review_row_id, decision FROM review_decision WHERE review_chapter_id = ?1")?;
    let raw = stmt
        .query_map([review_chapter_id], |row| Ok((row.get::<_, i64>(0)?, row.get::<_, String>(1)?)))?
        .collect::<SqlResult<Vec<_>>>()?;
    Ok(raw.into_iter().filter_map(|(id, value)| ReviewDecision::from_column(&value).map(|d| (id, d))).collect())
}

/// Records `decision` on every reviewer row of `row_ids`; a row decided before is overwritten.
pub(super) fn record_decision(
    tx: &Transaction<'_>,
    review_chapter_id: i64,
    row_ids: &[i64],
    decision: ReviewDecision,
) -> SqlResult<()> {
    for row_id in row_ids {
        tx.execute(
            "INSERT OR REPLACE INTO review_decision (review_row_id, review_chapter_id, decision, decided_at) \
             VALUES (?1, ?2, ?3, strftime('%Y-%m-%dT%H:%M:%fZ','now'))",
            (row_id, review_chapter_id, decision.as_str()),
        )?;
    }
    Ok(())
}

/// Forgets the decisions of every reviewer row of the copy of `chapter_id`.
pub(super) fn delete_decisions_of_chapter(tx: &Transaction<'_>, chapter_id: i64) -> SqlResult<()> {
    tx.execute(
        "DELETE FROM review_decision WHERE review_chapter_id IN (SELECT id FROM review_chapter WHERE chapter_id = ?1)",
        [chapter_id],
    )?;
    Ok(())
}

/// Forgets the decisions of every reviewer row of one copy.
pub(super) fn delete_decisions_of_copy(tx: &Transaction<'_>, review_chapter_id: i64) -> SqlResult<()> {
    tx.execute("DELETE FROM review_decision WHERE review_chapter_id = ?1", [review_chapter_id])?;
    Ok(())
}

/// Forgets the decisions of the reviewer rows now in `group_id`; call it before the group changes.
pub(super) fn delete_decisions_of_group(tx: &Transaction<'_>, group_id: i64) -> SqlResult<()> {
    tx.execute(
        "DELETE FROM review_decision WHERE review_row_id IN \
         (SELECT review_row_id FROM alignment_member WHERE group_id = ?1 AND review_row_id IS NOT NULL)",
        [group_id],
    )?;
    Ok(())
}

/// Forgets the decision of reviewer rows about to enter a new group.
pub(super) fn delete_decisions_of_rows(tx: &Transaction<'_>, row_ids: &[i64]) -> SqlResult<()> {
    for row_id in row_ids {
        tx.execute("DELETE FROM review_decision WHERE review_row_id = ?1", [row_id])?;
    }
    Ok(())
}

/// Groups of the copy of `chapter_id` the user accepted; importing again discards them.
pub(super) fn accepted_group_count(conn: ReadHandle<'_>, chapter_id: i64) -> SqlResult<usize> {
    let count: i64 = conn.query_row(
        "SELECT COUNT(DISTINCT m.group_id) FROM review_decision d \
         JOIN alignment_member m ON m.review_row_id = d.review_row_id \
         WHERE d.decision = 'accepted' AND d.review_chapter_id IN (SELECT id FROM review_chapter WHERE chapter_id = ?1)",
        [chapter_id],
        |row| row.get(0),
    )?;
    Ok(usize::try_from(count).unwrap_or(0))
}
