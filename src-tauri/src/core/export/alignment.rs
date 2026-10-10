use std::collections::{HashMap, HashSet};

use crate::commands::segment::{ChapterSegment, select_chapter_assets, select_chapter_segments};
use crate::core::matching::{DiffSpan, MatchLang, common_subsequence, diff_spans, similarity_percent};
use crate::core::segment::omit::segments_in_translation;
use crate::core::store::{ReadHandle, SqlError, SqlResult, Store, StoreError, Transaction};

use super::images::{ImageMode, ImageSplit, chapter_images, split_by_anchor};
use super::review_decision::{
    ReviewDecision, decisions_of_copy, delete_decisions_of_chapter, delete_decisions_of_copy, delete_decisions_of_group,
    delete_decisions_of_rows,
};
use super::reviewer_copy::{ReviewCopyError, ReviewFileKind, ReviewRow, ReviewRowKind, at_most_one, read_review_copy};
use super::text_export::{Emitted, emit_items, one_line};

/// Lowest share of common words (percent) at which an unanchored pair is joined by the machine.
pub const MIN_PAIR_SIMILARITY: u8 = 65;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DecidedBy {
    Machine,
    User,
}

impl DecidedBy {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Machine => "machine",
            Self::User => "user",
        }
    }

    fn from_column(value: &str) -> Option<Self> {
        match value {
            "machine" => Some(Self::Machine),
            "user" => Some(Self::User),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AlignmentRow {
    pub id: i64,
    pub kind: ReviewRowKind,
    pub source_text: Option<String>,
    pub target_text: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AlignmentSegment {
    pub id: i64,
    pub ord: i64,
    pub role: Option<String>,
    pub source_text: String,
    pub target_text: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AlignmentGroup {
    pub id: i64,
    pub decided_by: DecidedBy,
    pub row_ids: Vec<i64>,
    pub segment_ids: Vec<i64>,
}

/// Both sides of a reviewer copy and how they are grouped. A row or segment in no group is
/// unprocessed; `is_resolved` holds exactly when none is left.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChapterAlignment {
    pub chapter_id: i64,
    pub review_chapter_id: i64,
    pub file_name: String,
    pub file_kind: ReviewFileKind,
    pub rows: Vec<AlignmentRow>,
    pub segments: Vec<AlignmentSegment>,
    pub groups: Vec<AlignmentGroup>,
    pub unmatched_row_ids: Vec<i64>,
    pub unmatched_segment_ids: Vec<i64>,
    pub is_resolved: bool,
}

#[derive(Debug)]
pub enum AlignmentError {
    Copy(ReviewCopyError),
    /// An id is unknown to the Chapter, already in a group, or the selection is empty or repeats an id.
    InvalidSelection,
    /// The group is not 1:1 with a segment whose text is the text shown, or it has nothing to accept.
    NotAcceptable,
    Store(StoreError),
    Sql(SqlError),
}

impl From<ReviewCopyError> for AlignmentError {
    fn from(error: ReviewCopyError) -> Self {
        Self::Copy(error)
    }
}

impl From<StoreError> for AlignmentError {
    fn from(error: StoreError) -> Self {
        Self::Store(error)
    }
}

impl From<SqlError> for AlignmentError {
    fn from(error: SqlError) -> Self {
        Self::Sql(error)
    }
}

/// One thing the user sets aside as having no counterpart.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AlignmentItem {
    Segment(i64),
    Row(i64),
}

struct SegmentUnit {
    segment_ids: Vec<i64>,
    kind: ReviewRowKind,
    key: String,
    target: String,
}

struct RowUnit {
    id: i64,
    kind: ReviewRowKind,
    key: String,
    target: String,
}

struct MachineGroup {
    row_ids: Vec<i64>,
    segment_ids: Vec<i64>,
}

/// `.docx` exports one row per segment in the translation, keyed by the left cell.
fn docx_units(segments: &[&ChapterSegment]) -> Vec<SegmentUnit> {
    segments
        .iter()
        .map(|s| SegmentUnit {
            segment_ids: vec![s.id],
            kind: ReviewRowKind::Text,
            key: s.source_text.clone(),
            target: s.target_text.clone(),
        })
        .collect()
}

/// `.md` units are the paragraphs and alt/caption lines the exporter writes, with the segments each
/// was built from.
fn markdown_units(conn: ReadHandle<'_>, chapter_id: i64, segments: &[ChapterSegment]) -> SqlResult<Vec<SegmentUnit>> {
    let assets = select_chapter_assets(conn, chapter_id)?;
    let ImageSplit { head, by_anchor, orphans, .. } =
        split_by_anchor(segments, chapter_images(segments, &assets), ImageMode::File, |image| image);
    let mut units = Vec::new();
    for (item, segment_ids) in emit_items(segments, head, by_anchor, orphans) {
        match item {
            Emitted::Paragraph(text) => {
                units.push(SegmentUnit { segment_ids, kind: ReviewRowKind::Text, key: text.clone(), target: text });
            }
            Emitted::Image(image) => {
                let lines = [
                    (ReviewRowKind::Alt, image.alt_text, image.alt_segment_id),
                    (ReviewRowKind::Caption, image.caption_text, image.caption_segment_id),
                ];
                for (kind, text, segment_id) in lines {
                    let text = one_line(text.as_deref().unwrap_or(""));
                    if !text.is_empty() {
                        units.push(SegmentUnit { segment_ids: segment_id.into_iter().collect(), kind, key: text.clone(), target: text });
                    }
                }
            }
        }
    }
    Ok(units)
}

/// Between two anchors, equal counts pair by position and each pair must clear the threshold;
/// unequal counts pair nothing.
fn pair_gap(
    segment_units: &[SegmentUnit],
    row_units: &[RowUnit],
    segments: (usize, usize),
    rows: (usize, usize),
    min_similarity: u8,
    pairs: &mut Vec<(usize, usize)>,
) {
    if segments.1 - segments.0 != rows.1 - rows.0 {
        return;
    }
    for (s, r) in (segments.0..segments.1).zip(rows.0..rows.1) {
        let (seg, row) = (&segment_units[s], &row_units[r]);
        if seg.kind == row.kind && similarity_percent(&seg.target, &row.target, MatchLang::En) >= min_similarity {
            pairs.push((s, r));
        }
    }
}

fn pairs_of(
    segment_units: &[SegmentUnit],
    row_units: &[RowUnit],
    min_similarity: u8,
) -> Vec<(usize, usize)> {
    let old: Vec<(ReviewRowKind, &str)> = segment_units.iter().map(|u| (u.kind, u.key.as_str())).collect();
    let new: Vec<(ReviewRowKind, &str)> = row_units.iter().map(|u| (u.kind, u.key.as_str())).collect();
    let anchors = common_subsequence(&old, &new);

    let mut pairs = Vec::new();
    let (mut seg_from, mut row_from) = (0, 0);
    for (s, r) in anchors {
        pair_gap(segment_units, row_units, (seg_from, s), (row_from, r), min_similarity, &mut pairs);
        pairs.push((s, r));
        seg_from = s + 1;
        row_from = r + 1;
    }
    pair_gap(
        segment_units,
        row_units,
        (seg_from, segment_units.len()),
        (row_from, row_units.len()),
        min_similarity,
        &mut pairs,
    );
    pairs
}

fn exported_key(row: &ReviewRow, kind: ReviewFileKind) -> String {
    match kind {
        ReviewFileKind::Docx => row.source_text.clone().unwrap_or_default(),
        ReviewFileKind::Markdown => row.target_text.clone(),
    }
}

fn machine_groups(
    kind: ReviewFileKind,
    segments: &[ChapterSegment],
    rows: &[(i64, ReviewRow)],
    units: &[SegmentUnit],
) -> Vec<MachineGroup> {
    let row_units: Vec<RowUnit> = rows
        .iter()
        .map(|(id, row)| RowUnit { id: *id, kind: row.kind, key: exported_key(row, kind), target: row.target_text.clone() })
        .collect();
    let paired: HashMap<usize, usize> = pairs_of(units, &row_units, MIN_PAIR_SIMILARITY).into_iter().collect();

    let mut component_of: Vec<usize> = (0..units.len()).collect();
    for i in 0..units.len() {
        for j in 0..i {
            if units[i].segment_ids.iter().any(|id| units[j].segment_ids.contains(id)) {
                let (keep, drop) = (component_of[j], component_of[i]);
                for c in &mut component_of {
                    if *c == drop {
                        *c = keep;
                    }
                }
            }
        }
    }
    let mut groups: Vec<MachineGroup> = Vec::new();
    let mut seen: HashSet<usize> = HashSet::new();
    for first in 0..units.len() {
        let component = component_of[first];
        if !seen.insert(component) {
            continue;
        }
        let members: Vec<usize> = (first..units.len()).filter(|&u| component_of[u] == component).collect();
        if !members.iter().all(|u| paired.contains_key(u)) {
            continue;
        }
        let mut segment_ids: Vec<i64> = Vec::new();
        for &u in &members {
            for id in &units[u].segment_ids {
                if !segment_ids.contains(id) {
                    segment_ids.push(*id);
                }
            }
        }
        let row_ids = members.iter().map(|u| row_units[paired[u]].id).collect();
        groups.push(MachineGroup { row_ids, segment_ids });
    }

    let covered: HashSet<i64> = units.iter().flat_map(|u| u.segment_ids.iter().copied()).collect();
    for segment in segments_in_translation(segments) {
        if kind == ReviewFileKind::Markdown && segment.role.is_none() && !covered.contains(&segment.id) {
            groups.push(MachineGroup { row_ids: Vec::new(), segment_ids: vec![segment.id] });
        }
    }
    groups
}

fn load_rows(conn: ReadHandle<'_>, review_chapter_id: i64) -> SqlResult<Vec<(i64, ReviewRow)>> {
    let mut stmt = conn.prepare(
        "SELECT id, kind, source_text, target_text FROM review_row WHERE review_chapter_id = ?1 ORDER BY ord",
    )?;
    let raw = stmt
        .query_map([review_chapter_id], |row| {
            Ok((row.get::<_, i64>(0)?, row.get::<_, String>(1)?, row.get::<_, Option<String>>(2)?, row.get::<_, String>(3)?))
        })?
        .collect::<SqlResult<Vec<_>>>()?;
    Ok(raw
        .into_iter()
        .filter_map(|(id, kind, source_text, target_text)| {
            ReviewRowKind::from_column(&kind)
                .map(|kind| (id, ReviewRow { kind, source_text, target_text }))
        })
        .collect())
}

fn copy_head(conn: ReadHandle<'_>, review_chapter_id: i64) -> SqlResult<Option<(i64, String)>> {
    at_most_one(conn.query_row(
        "SELECT chapter_id, file_kind FROM review_chapter WHERE id = ?1",
        [review_chapter_id],
        |row| Ok((row.get(0)?, row.get(1)?)),
    ))
}

/// Replaces every group of one reviewer copy with the machine's reading of it and stamps
/// `aligned_at`. Runs inside the transaction that wrote the copy (AD-52 rule 7); it reads
/// `segment` and never writes it.
pub fn align_chapter(tx: &Transaction<'_>, review_chapter_id: i64) -> SqlResult<()> {
    let Some((chapter_id, file_kind)) = copy_head(tx, review_chapter_id)? else {
        return Ok(());
    };
    let kind = match file_kind.as_str() {
        "docx" => ReviewFileKind::Docx,
        _ => ReviewFileKind::Markdown,
    };
    delete_decisions_of_copy(tx, review_chapter_id)?;
    tx.execute(
        "DELETE FROM alignment_member WHERE group_id IN (SELECT id FROM alignment_group WHERE review_chapter_id = ?1)",
        [review_chapter_id],
    )?;
    tx.execute("DELETE FROM alignment_group WHERE review_chapter_id = ?1", [review_chapter_id])?;

    let segments = select_chapter_segments(tx, chapter_id)?;
    let rows = load_rows(tx, review_chapter_id)?;
    let units = match kind {
        ReviewFileKind::Docx => docx_units(&segments_in_translation(&segments)),
        ReviewFileKind::Markdown => markdown_units(tx, chapter_id, &segments)?,
    };
    for group in machine_groups(kind, &segments, &rows, &units) {
        tx.execute(
            "INSERT INTO alignment_group (review_chapter_id, decided_by) VALUES (?1, ?2)",
            (review_chapter_id, DecidedBy::Machine.as_str()),
        )?;
        let group_id = tx.last_insert_rowid();
        for row_id in group.row_ids {
            tx.execute(
                "INSERT INTO alignment_member (group_id, review_row_id, segment_id) VALUES (?1, ?2, NULL)",
                (group_id, row_id),
            )?;
        }
        for segment_id in group.segment_ids {
            tx.execute(
                "INSERT INTO alignment_member (group_id, review_row_id, segment_id) VALUES (?1, NULL, ?2)",
                (group_id, segment_id),
            )?;
        }
    }
    tx.execute(
        "UPDATE review_chapter SET aligned_at = strftime('%Y-%m-%dT%H:%M:%fZ','now') WHERE id = ?1",
        [review_chapter_id],
    )?;
    Ok(())
}

fn align_if_pending(tx: &Transaction<'_>, review_chapter_id: i64) -> SqlResult<()> {
    let pending: bool = tx.query_row(
        "SELECT aligned_at IS NULL FROM review_chapter WHERE id = ?1",
        [review_chapter_id],
        |row| row.get(0),
    )?;
    if pending { align_chapter(tx, review_chapter_id) } else { Ok(()) }
}

/// Deletes every group of the reviewer copy of `chapter_id` (AD-52 rule 6). Call it before the
/// `review_chapter` row goes, because the groups are found through it.
pub fn delete_alignment_of_chapter(tx: &Transaction<'_>, chapter_id: i64) -> SqlResult<()> {
    delete_decisions_of_chapter(tx, chapter_id)?;
    tx.execute(
        "DELETE FROM alignment_member WHERE group_id IN (SELECT id FROM alignment_group WHERE review_chapter_id IN \
         (SELECT id FROM review_chapter WHERE chapter_id = ?1))",
        [chapter_id],
    )?;
    tx.execute(
        "DELETE FROM alignment_group WHERE review_chapter_id IN (SELECT id FROM review_chapter WHERE chapter_id = ?1)",
        [chapter_id],
    )?;
    Ok(())
}

/// Number of groups the user decided for the copy of `chapter_id`.
pub(super) fn user_group_count(conn: ReadHandle<'_>, chapter_id: i64) -> SqlResult<usize> {
    let count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM alignment_group WHERE decided_by = 'user' AND review_chapter_id IN \
         (SELECT id FROM review_chapter WHERE chapter_id = ?1)",
        [chapter_id],
        |row| row.get(0),
    )?;
    Ok(usize::try_from(count).unwrap_or(0))
}

/// Segments retired by a merge or split hand their group membership to the segments that replace
/// them. A group left without any segment after that stops being a pair, so it is dissolved and
/// its rows return to the unprocessed list. Call it in the transaction that retires them.
pub fn move_members_of_retired(tx: &Transaction<'_>, retired: &[i64], fresh: &[i64]) -> SqlResult<()> {
    let mut touched: Vec<i64> = Vec::new();
    for old in retired {
        let group: Option<i64> = at_most_one(tx.query_row(
            "SELECT group_id FROM alignment_member WHERE segment_id = ?1",
            [old],
            |row| row.get(0),
        ))?;
        let Some(group_id) = group else { continue };
        tx.execute("DELETE FROM alignment_member WHERE segment_id = ?1", [old])?;
        for new in fresh {
            tx.execute(
                "INSERT OR IGNORE INTO alignment_member (group_id, review_row_id, segment_id) VALUES (?1, NULL, ?2)",
                (group_id, new),
            )?;
        }
        if !touched.contains(&group_id) {
            touched.push(group_id);
        }
    }
    for group_id in touched {
        delete_decisions_of_group(tx, group_id)?;
        let segment_members: i64 = tx.query_row(
            "SELECT COUNT(*) FROM alignment_member WHERE group_id = ?1 AND segment_id IS NOT NULL",
            [group_id],
            |row| row.get(0),
        )?;
        if segment_members == 0 {
            tx.execute("DELETE FROM alignment_member WHERE group_id = ?1", [group_id])?;
            tx.execute("DELETE FROM alignment_group WHERE id = ?1", [group_id])?;
        }
    }
    Ok(())
}

/// The one reader of a Chapter's alignment. It goes through [`read_review_copy`], so a Chapter
/// without a copy or with a stale one is a typed error, never an empty list; a copy imported before
/// step 31 is aligned here once.
pub fn read_alignment(store: &Store, chapter_id: i64) -> Result<ChapterAlignment, AlignmentError> {
    let copy = read_review_copy(store, chapter_id)?;
    let copy_id = copy.id;
    store.write(move |tx| align_if_pending(tx, copy_id))?;
    let loaded = store.read(move |conn| {
        let rows = load_rows(conn, copy_id)?;
        let segments = select_chapter_segments(conn, chapter_id)?;
        let mut stmt = conn.prepare(
            "SELECT g.id, g.decided_by, m.review_row_id, m.segment_id FROM alignment_group g \
             LEFT JOIN alignment_member m ON m.group_id = g.id WHERE g.review_chapter_id = ?1 ORDER BY g.id, m.rowid",
        )?;
        let members = stmt
            .query_map([copy_id], |row| {
                Ok((row.get::<_, i64>(0)?, row.get::<_, String>(1)?, row.get::<_, Option<i64>>(2)?, row.get::<_, Option<i64>>(3)?))
            })?
            .collect::<SqlResult<Vec<_>>>()?;
        Ok((rows, segments, members))
    })?;
    let (rows, segments, members) = loaded;

    let mut groups: Vec<AlignmentGroup> = Vec::new();
    for (group_id, decided_by, row_id, segment_id) in members {
        let Some(decided_by) = DecidedBy::from_column(&decided_by) else { continue };
        if groups.last().is_none_or(|g| g.id != group_id) {
            groups.push(AlignmentGroup { id: group_id, decided_by, row_ids: Vec::new(), segment_ids: Vec::new() });
        }
        if let Some(group) = groups.last_mut() {
            group.row_ids.extend(row_id);
            group.segment_ids.extend(segment_id);
        }
    }
    let grouped_rows: HashSet<i64> = groups.iter().flat_map(|g| g.row_ids.iter().copied()).collect();
    let grouped_segments: HashSet<i64> = groups.iter().flat_map(|g| g.segment_ids.iter().copied()).collect();

    let rows: Vec<AlignmentRow> = rows
        .into_iter()
        .map(|(id, row)| AlignmentRow { id, kind: row.kind, source_text: row.source_text, target_text: row.target_text })
        .collect();
    let segments: Vec<AlignmentSegment> = segments_in_translation(&segments)
        .into_iter()
        .map(|s| AlignmentSegment {
            id: s.id,
            ord: s.ord,
            role: s.role.clone(),
            source_text: s.source_text.clone(),
            target_text: s.target_text.clone(),
        })
        .collect();
    let unmatched_row_ids: Vec<i64> = rows.iter().map(|r| r.id).filter(|id| !grouped_rows.contains(id)).collect();
    let unmatched_segment_ids: Vec<i64> =
        segments.iter().map(|s| s.id).filter(|id| !grouped_segments.contains(id)).collect();
    let is_resolved = unmatched_row_ids.is_empty() && unmatched_segment_ids.is_empty();
    Ok(ChapterAlignment {
        chapter_id,
        review_chapter_id: copy.id,
        file_name: copy.file_name,
        file_kind: copy.file_kind,
        rows,
        segments,
        groups,
        unmatched_row_ids,
        unmatched_segment_ids,
        is_resolved,
    })
}

/// One group with my text diffed to the reviewer's: `Delete` is what only I have, `Insert` what only
/// the reviewer has.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GroupDiff {
    pub group_id: i64,
    pub decided_by: DecidedBy,
    pub segment_ids: Vec<i64>,
    pub row_ids: Vec<i64>,
    pub spans: Vec<DiffSpan>,
    /// What the user decided about the group's reviewer rows; `None` while none is decided.
    pub decision: Option<ReviewDecision>,
}

/// One group with the text of each side joined the way a reader sees it: my translation units and
/// the reviewer's rows, each in document order. `order` sorts groups in the order of the translation
/// (groups with no segment last, in row order).
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct GroupTexts {
    pub group_id: i64,
    pub decided_by: DecidedBy,
    pub segment_ids: Vec<i64>,
    pub row_ids: Vec<i64>,
    pub mine: String,
    pub theirs: String,
    order: (usize, usize),
}

/// The alignment of `chapter_id` with both sides of every group joined into one string. Writes
/// nothing but the machine grouping a copy still waits for.
pub(super) fn group_texts(store: &Store, chapter_id: i64) -> Result<(ChapterAlignment, Vec<GroupTexts>), AlignmentError> {
    let alignment = read_alignment(store, chapter_id)?;
    let kind = alignment.file_kind;
    let units = store.read(move |conn| {
        let segments = select_chapter_segments(conn, chapter_id)?;
        match kind {
            ReviewFileKind::Docx => Ok(docx_units(&segments_in_translation(&segments))),
            ReviewFileKind::Markdown => markdown_units(conn, chapter_id, &segments),
        }
    })?;
    let row_index: HashMap<i64, usize> = alignment.rows.iter().enumerate().map(|(i, r)| (r.id, i)).collect();

    let mut texts: Vec<GroupTexts> = alignment
        .groups
        .iter()
        .map(|group| {
            let mine: Vec<usize> = units
                .iter()
                .enumerate()
                .filter(|(_, unit)| unit.segment_ids.iter().any(|id| group.segment_ids.contains(id)))
                .map(|(i, _)| i)
                .collect();
            let mine_text = mine.iter().map(|&i| units[i].target.as_str()).collect::<Vec<_>>().join(" ");
            let mut row_ids = group.row_ids.clone();
            row_ids.sort_by_key(|id| row_index.get(id).copied());
            let theirs_text = row_ids
                .iter()
                .filter_map(|id| row_index.get(id).map(|&i| alignment.rows[i].target_text.as_str()))
                .collect::<Vec<_>>()
                .join(" ");
            let order = match (mine.first(), row_ids.first().and_then(|id| row_index.get(id))) {
                (Some(&unit), _) => (0, unit),
                (None, row) => (1, row.copied().unwrap_or(usize::MAX)),
            };
            GroupTexts {
                group_id: group.id,
                decided_by: group.decided_by,
                segment_ids: group.segment_ids.clone(),
                row_ids,
                mine: mine_text,
                theirs: theirs_text,
                order,
            }
        })
        .collect();
    texts.sort_by_key(|t| t.order);
    Ok((alignment, texts))
}

/// Diffs every group of the reviewer copy of `chapter_id`, in the order of the translation (groups
/// with no segment last, in row order). Writes nothing but the machine grouping a copy still waits for.
pub fn review_diff(store: &Store, chapter_id: i64) -> Result<Vec<GroupDiff>, AlignmentError> {
    let (alignment, texts) = group_texts(store, chapter_id)?;
    let review_chapter_id = alignment.review_chapter_id;
    let decisions = store.read(move |conn| decisions_of_copy(conn, review_chapter_id))?;
    Ok(texts
        .into_iter()
        .map(|t| GroupDiff {
            group_id: t.group_id,
            decided_by: t.decided_by,
            decision: t.row_ids.iter().find_map(|id| decisions.get(id).copied()),
            segment_ids: t.segment_ids,
            row_ids: t.row_ids,
            spans: diff_spans(&t.mine, &t.theirs, MatchLang::En),
        })
        .collect())
}

/// The one change of a 1:1 group that can be taken over: my segment, the reviewer's row and the text
/// of each side as shown.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AcceptableChange {
    pub review_chapter_id: i64,
    pub segment_id: i64,
    pub row_id: i64,
    pub mine: String,
    pub theirs: String,
}

/// Reads group `group_id` through [`group_texts`] and returns what accepting it would write.
/// A group that is not exactly one segment and one row, whose segment text is not the text shown,
/// or whose two sides are already equal is [`AlignmentError::NotAcceptable`]. Writes nothing but
/// the machine grouping a copy still waits for.
pub fn acceptable_change(store: &Store, chapter_id: i64, group_id: i64) -> Result<AcceptableChange, AlignmentError> {
    let (alignment, texts) = group_texts(store, chapter_id)?;
    let Some(group) = texts.into_iter().find(|t| t.group_id == group_id) else {
        return Err(AlignmentError::InvalidSelection);
    };
    let ([segment_id], [row_id]) = (group.segment_ids.as_slice(), group.row_ids.as_slice()) else {
        return Err(AlignmentError::NotAcceptable);
    };
    let shown = alignment.segments.iter().find(|s| s.id == *segment_id).map(|s| s.target_text.as_str());
    if shown != Some(group.mine.as_str()) || group.mine == group.theirs {
        return Err(AlignmentError::NotAcceptable);
    }
    Ok(AcceptableChange {
        review_chapter_id: alignment.review_chapter_id,
        segment_id: *segment_id,
        row_id: *row_id,
        mine: group.mine,
        theirs: group.theirs,
    })
}

fn members_of_group(
    tx: &Transaction<'_>,
    review_chapter_id: i64,
    group_id: i64,
) -> SqlResult<(Vec<i64>, Vec<i64>)> {
    let mut stmt = tx.prepare(
        "SELECT m.review_row_id, m.segment_id FROM alignment_group g JOIN alignment_member m ON m.group_id = g.id \
         WHERE g.id = ?1 AND g.review_chapter_id = ?2",
    )?;
    let members = stmt
        .query_map((group_id, review_chapter_id), |row| Ok((row.get::<_, Option<i64>>(0)?, row.get::<_, Option<i64>>(1)?)))?
        .collect::<SqlResult<Vec<_>>>()?;
    Ok((members.iter().filter_map(|m| m.0).collect(), members.iter().filter_map(|m| m.1).collect()))
}

/// In the caller's transaction: when `group_id` of the copy `review_chapter_id` is still exactly
/// `segment_id` and `row_id`, records the row as accepted and returns `true`; otherwise writes nothing.
pub fn mark_accepted(
    tx: &Transaction<'_>,
    review_chapter_id: i64,
    group_id: i64,
    segment_id: i64,
    row_id: i64,
) -> SqlResult<bool> {
    let (rows, segments) = members_of_group(tx, review_chapter_id, group_id)?;
    if rows != [row_id] || segments != [segment_id] {
        return Ok(false);
    }
    super::review_decision::record_decision(tx, review_chapter_id, &rows, ReviewDecision::Accepted)?;
    Ok(true)
}

/// Skips the group: remembers that its reviewer rows were looked at and left alone. Any group with
/// at least one reviewer row can be skipped.
pub fn skip_change(store: &Store, chapter_id: i64, group_id: i64) -> Result<(), AlignmentError> {
    let outcome = store.write(move |tx| {
        let review_chapter_id = match live_copy(tx, chapter_id)? {
            Ok(id) => id,
            Err(error) => return Ok(Err(AlignmentError::Copy(error))),
        };
        align_if_pending(tx, review_chapter_id)?;
        let (row_ids, _) = members_of_group(tx, review_chapter_id, group_id)?;
        if row_ids.is_empty() {
            return Ok(Err(AlignmentError::InvalidSelection));
        }
        super::review_decision::record_decision(tx, review_chapter_id, &row_ids, ReviewDecision::Skipped)?;
        Ok(Ok(()))
    })?;
    outcome
}

/// Id of the live copy of `chapter_id`, or the typed reason there is none.
fn live_copy(conn: ReadHandle<'_>, chapter_id: i64) -> SqlResult<Result<i64, ReviewCopyError>> {
    let found: Option<(i64, Option<String>)> = at_most_one(conn.query_row(
        "SELECT id, stale_at FROM review_chapter WHERE chapter_id = ?1",
        [chapter_id],
        |row| Ok((row.get(0)?, row.get(1)?)),
    ))?;
    Ok(match found {
        None => Err(ReviewCopyError::NotImported),
        Some((_, Some(_))) => Err(ReviewCopyError::Stale),
        Some((id, None)) => Ok(id),
    })
}

fn is_ungrouped_row(conn: ReadHandle<'_>, review_chapter_id: i64, row_id: i64) -> SqlResult<bool> {
    conn.query_row(
        "SELECT EXISTS (SELECT 1 FROM review_row WHERE id = ?1 AND review_chapter_id = ?2) \
         AND NOT EXISTS (SELECT 1 FROM alignment_member WHERE review_row_id = ?1)",
        (row_id, review_chapter_id),
        |row| row.get(0),
    )
}

fn is_ungrouped_segment(conn: ReadHandle<'_>, chapter_id: i64, segment_id: i64) -> SqlResult<bool> {
    conn.query_row(
        "SELECT EXISTS (SELECT 1 FROM segment WHERE id = ?1 AND chapter_id = ?2 AND retired_at IS NULL AND is_omitted = 0) \
         AND NOT EXISTS (SELECT 1 FROM alignment_member WHERE segment_id = ?1)",
        (segment_id, chapter_id),
        |row| row.get(0),
    )
}

fn has_repeat(ids: &[i64]) -> bool {
    ids.iter().collect::<HashSet<_>>().len() != ids.len()
}

/// Writes one user group after checking every id; any failed check writes nothing.
fn write_user_group(
    store: &Store,
    chapter_id: i64,
    segment_ids: &[i64],
    row_ids: &[i64],
) -> Result<i64, AlignmentError> {
    if has_repeat(segment_ids) || has_repeat(row_ids) {
        return Err(AlignmentError::InvalidSelection);
    }
    let segment_ids = segment_ids.to_vec();
    let row_ids = row_ids.to_vec();
    let outcome = store.write(move |tx| {
        let review_chapter_id = match live_copy(tx, chapter_id)? {
            Ok(id) => id,
            Err(error) => return Ok(Err(AlignmentError::Copy(error))),
        };
        align_if_pending(tx, review_chapter_id)?;
        for &row_id in &row_ids {
            if !is_ungrouped_row(tx, review_chapter_id, row_id)? {
                return Ok(Err(AlignmentError::InvalidSelection));
            }
        }
        for &segment_id in &segment_ids {
            if !is_ungrouped_segment(tx, chapter_id, segment_id)? {
                return Ok(Err(AlignmentError::InvalidSelection));
            }
        }
        delete_decisions_of_rows(tx, &row_ids)?;
        tx.execute(
            "INSERT INTO alignment_group (review_chapter_id, decided_by) VALUES (?1, ?2)",
            (review_chapter_id, DecidedBy::User.as_str()),
        )?;
        let group_id = tx.last_insert_rowid();
        for row_id in &row_ids {
            tx.execute(
                "INSERT INTO alignment_member (group_id, review_row_id, segment_id) VALUES (?1, ?2, NULL)",
                (group_id, row_id),
            )?;
        }
        for segment_id in &segment_ids {
            tx.execute(
                "INSERT INTO alignment_member (group_id, review_row_id, segment_id) VALUES (?1, NULL, ?2)",
                (group_id, segment_id),
            )?;
        }
        Ok(Ok(group_id))
    })?;
    outcome
}

/// Joins at least one segment and at least one reviewer row into a user group.
pub fn join(store: &Store, chapter_id: i64, segment_ids: &[i64], row_ids: &[i64]) -> Result<i64, AlignmentError> {
    if segment_ids.is_empty() || row_ids.is_empty() {
        return Err(AlignmentError::InvalidSelection);
    }
    write_user_group(store, chapter_id, segment_ids, row_ids)
}

/// Sets one segment or one reviewer row aside as a one-sided user group.
pub fn skip(store: &Store, chapter_id: i64, item: AlignmentItem) -> Result<i64, AlignmentError> {
    match item {
        AlignmentItem::Segment(id) => write_user_group(store, chapter_id, &[id], &[]),
        AlignmentItem::Row(id) => write_user_group(store, chapter_id, &[], &[id]),
    }
}

/// Dissolves a group; its members return to the unprocessed list.
pub fn unjoin(store: &Store, chapter_id: i64, group_id: i64) -> Result<(), AlignmentError> {
    let outcome = store.write(move |tx| {
        let review_chapter_id = match live_copy(tx, chapter_id)? {
            Ok(id) => id,
            Err(error) => return Ok(Err(AlignmentError::Copy(error))),
        };
        let owned: bool = tx.query_row(
            "SELECT EXISTS (SELECT 1 FROM alignment_group WHERE id = ?1 AND review_chapter_id = ?2)",
            (group_id, review_chapter_id),
            |row| row.get(0),
        )?;
        if !owned {
            return Ok(Err(AlignmentError::InvalidSelection));
        }
        delete_decisions_of_group(tx, group_id)?;
        tx.execute("DELETE FROM alignment_member WHERE group_id = ?1", [group_id])?;
        tx.execute("DELETE FROM alignment_group WHERE id = ?1", [group_id])?;
        Ok(Ok(()))
    })?;
    outcome
}
