//! Deleting one Chapter inside a caller-owned transaction.
//!
//! A Chapter with nothing worth keeping is removed outright (the `merge_into_previous_tx`
//! cleanup plus its segments). One that holds a non-blank `target_text` or any `segment_version`
//! moves to `chapter_deleted` and its segments are retired, so no translation byte is lost.

use crate::commands::chapter::normalize_chapter_ord;
use crate::core::store::schema::segment_has_keepable_sql;
use crate::core::store::{ReadHandle, SqlResult, Transaction};

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Disposition {
    Hard,
    Soft,
}

/// What [`delete_chapter_tx`] would do, computed by the same test it runs.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
pub struct DeletePreview {
    pub disposition: Disposition,
    /// Deleting leaves neither a `chapter` nor a `chapter_deleted` row, so the Work goes too.
    pub removes_work: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DeleteOutcome {
    pub disposition: Disposition,
    /// Chapter the open pointer should rest on afterwards; `None` when no live Chapter is left.
    pub next_open_chapter_id: Option<i64>,
    pub work_empty: bool,
}

fn chapter_exists(conn: ReadHandle<'_>, chapter_id: i64) -> SqlResult<bool> {
    conn.query_row(
        "SELECT EXISTS (SELECT 1 FROM chapter WHERE id = ?1)",
        [chapter_id],
        |row| row.get(0),
    )
}

fn disposition_of(conn: ReadHandle<'_>, chapter_id: i64) -> SqlResult<Disposition> {
    let keepable: bool = conn.query_row(
        concat!(
            "SELECT EXISTS (SELECT 1 FROM segment s WHERE s.chapter_id = ?1 AND ",
            segment_has_keepable_sql!(),
            ")"
        ),
        [chapter_id],
        |row| row.get(0),
    )?;
    Ok(if keepable {
        Disposition::Soft
    } else {
        Disposition::Hard
    })
}

/// `Ok(None)` when `chapter_id` is not a `chapter` row.
pub fn preview(conn: ReadHandle<'_>, chapter_id: i64) -> SqlResult<Option<DeletePreview>> {
    if !chapter_exists(conn, chapter_id)? {
        return Ok(None);
    }
    let disposition = disposition_of(conn, chapter_id)?;
    let others: i64 = conn.query_row(
        "SELECT (SELECT COUNT(*) FROM chapter WHERE id <> ?1) + (SELECT COUNT(*) FROM chapter_deleted)",
        [chapter_id],
        |row| row.get(0),
    )?;
    Ok(Some(DeletePreview {
        disposition,
        removes_work: disposition == Disposition::Hard && others == 0,
    }))
}

/// `Ok(None)` when `chapter_id` is not a `chapter` row; nothing is written then. The open
/// pointer is not touched here: the caller moves it through `set_open_chapter`.
pub fn delete_chapter_tx(
    tx: &Transaction<'_>,
    chapter_id: i64,
    open_chapter_id: Option<i64>,
) -> SqlResult<Option<DeleteOutcome>> {
    if !chapter_exists(tx, chapter_id)? {
        return Ok(None);
    }
    let disposition = disposition_of(tx, chapter_id)?;

    let next_open = if open_chapter_id == Some(chapter_id) {
        neighbour_of(tx, chapter_id)?
    } else {
        open_chapter_id
    };

    let of_chapter = "(SELECT id FROM segment WHERE chapter_id = ?1)";
    tx.execute(
        "DELETE FROM chapter_position WHERE chapter_id = ?1",
        [chapter_id],
    )?;
    crate::core::export::delete_alignment_of_chapter(tx, chapter_id)?;
    tx.execute(
        "DELETE FROM review_row WHERE review_chapter_id IN \
         (SELECT id FROM review_chapter WHERE chapter_id = ?1)",
        [chapter_id],
    )?;
    tx.execute(
        "DELETE FROM review_chapter WHERE chapter_id = ?1",
        [chapter_id],
    )?;
    tx.execute(
        &format!("DELETE FROM alignment_member WHERE segment_id IN {of_chapter}"),
        [chapter_id],
    )?;
    tx.execute(
        &format!(
            "DELETE FROM reading_mark WHERE segment_id IN {of_chapter} \
             OR navigation_segment_id IN {of_chapter}"
        ),
        [chapter_id],
    )?;
    tx.execute(
        &format!("DELETE FROM segment_unlinked WHERE segment_id IN {of_chapter}"),
        [chapter_id],
    )?;

    match disposition {
        Disposition::Soft => {
            tx.execute(
                "INSERT INTO chapter_deleted (id, ord, title, source_text, status, created_at, \
                 updated_at, origin_author, origin_site_name, origin_url, origin_published_at, \
                 deleted_at) \
                 SELECT id, ord, title, source_text, status, created_at, updated_at, \
                 origin_author, origin_site_name, origin_url, origin_published_at, \
                 strftime('%Y-%m-%dT%H:%M:%fZ','now') FROM chapter WHERE id = ?1",
                [chapter_id],
            )?;
            tx.execute(
                "UPDATE segment SET retired_at = strftime('%Y-%m-%dT%H:%M:%fZ','now') \
                 WHERE chapter_id = ?1 AND retired_at IS NULL",
                [chapter_id],
            )?;
        }
        Disposition::Hard => {
            tx.execute("DELETE FROM asset WHERE chapter_id = ?1", [chapter_id])?;
            tx.execute(
                &format!("DELETE FROM segment_version WHERE segment_id IN {of_chapter}"),
                [chapter_id],
            )?;
            tx.execute("DELETE FROM segment WHERE chapter_id = ?1", [chapter_id])?;
        }
    }

    tx.execute("DELETE FROM chapter WHERE id = ?1", [chapter_id])?;
    normalize_chapter_ord(tx)?;

    let left: i64 = tx.query_row(
        "SELECT (SELECT COUNT(*) FROM chapter) + (SELECT COUNT(*) FROM chapter_deleted)",
        [],
        |row| row.get(0),
    )?;

    Ok(Some(DeleteOutcome {
        disposition,
        next_open_chapter_id: next_open,
        work_empty: left == 0,
    }))
}

/// The Chapter just before `chapter_id` by `(ord, id)`, else the one just after it, else none.
/// Runs before the row is deleted, so the Chapter itself is excluded.
fn neighbour_of(tx: &Transaction<'_>, chapter_id: i64) -> SqlResult<Option<i64>> {
    let (ord, id): (i64, i64) = tx.query_row(
        "SELECT ord, id FROM chapter WHERE id = ?1",
        [chapter_id],
        |row| Ok((row.get(0)?, row.get(1)?)),
    )?;
    let mut previous = tx.prepare(
        "SELECT id FROM chapter WHERE ord < ?1 OR (ord = ?1 AND id < ?2) \
         ORDER BY ord DESC, id DESC LIMIT 1",
    )?;
    if let Some(found) = previous.query_map((ord, id), |row| row.get(0))?.next() {
        return found.map(Some);
    }
    let mut following = tx.prepare(
        "SELECT id FROM chapter WHERE ord > ?1 OR (ord = ?1 AND id > ?2) \
         ORDER BY ord, id LIMIT 1",
    )?;
    following
        .query_map((ord, id), |row| row.get(0))?
        .next()
        .transpose()
}
