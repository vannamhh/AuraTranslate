use std::collections::HashSet;

use crate::core::store::{ReadHandle, SqlError, SqlResult, Store, StoreError, Transaction};

use super::alignment::{align_chapter, delete_alignment_of_chapter, user_group_count};
use super::review_decision::accepted_group_count;
use super::attribution::is_attribution_line;
use super::image_files::{IMAGE_DIR_SUFFIX, copied_name};
use super::reimport_gate::ReviewerDocx;
use super::table_rows::ExportImage;
use super::text_export::{heading_text, one_line};
use crate::core::docx::DocxBodyItem;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReviewFileKind {
    Docx,
    Markdown,
}

impl ReviewFileKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Docx => "docx",
            Self::Markdown => "md",
        }
    }

    fn from_column(value: &str) -> Option<Self> {
        match value {
            "docx" => Some(Self::Docx),
            "md" => Some(Self::Markdown),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ReviewRowKind {
    Text,
    Alt,
    Caption,
}

impl ReviewRowKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Text => "text",
            Self::Alt => "alt",
            Self::Caption => "caption",
        }
    }

    pub(super) fn from_column(value: &str) -> Option<Self> {
        match value {
            "text" => Some(Self::Text),
            "alt" => Some(Self::Alt),
            "caption" => Some(Self::Caption),
            _ => None,
        }
    }
}

/// One unit of the file: a `.docx` table row or a `.md` paragraph (AD-52 rule 2). `source_text` is
/// `None` exactly when the file is `.md`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReviewRow {
    pub kind: ReviewRowKind,
    pub source_text: Option<String>,
    pub target_text: String,
}

/// A table (`.docx`) or a `##` section (`.md`), before it is matched to a Chapter.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReviewSection {
    pub heading: String,
    pub rows: Vec<ReviewRow>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReviewerCopy {
    pub file_name: String,
    pub kind: ReviewFileKind,
    pub sections: Vec<ReviewSection>,
}

#[derive(Debug)]
pub enum ReviewCopyError {
    /// The file is not the two-column or Markdown shape the app writes.
    Unreadable,
    /// No section of the file matches any Chapter of the open Work.
    NoMatch,
    /// Nothing was imported for this Chapter.
    NotImported,
    /// The Chapter was merged or split after the copy was imported.
    Stale,
    Store(StoreError),
    Sql(SqlError),
}

impl From<StoreError> for ReviewCopyError {
    fn from(error: StoreError) -> Self {
        Self::Store(error)
    }
}

impl From<SqlError> for ReviewCopyError {
    fn from(error: SqlError) -> Self {
        Self::Sql(error)
    }
}

pub fn read_docx_copy(docx: &ReviewerDocx, file_name: &str) -> Result<ReviewerCopy, ReviewCopyError> {
    let mut sections = Vec::new();
    let mut last_paragraph: Option<&str> = None;
    for item in &docx.parsed().body {
        match item {
            DocxBodyItem::Paragraph(text) => last_paragraph = Some(text),
            DocxBodyItem::Table(table) => {
                let mut rows = Vec::with_capacity(table.len());
                for row in table {
                    let [source, target] = row.as_slice() else {
                        return Err(ReviewCopyError::Unreadable);
                    };
                    rows.push(ReviewRow {
                        kind: ReviewRowKind::Text,
                        source_text: Some(source.clone()),
                        target_text: target.clone(),
                    });
                }
                sections.push(ReviewSection { heading: last_paragraph.take().unwrap_or("").to_owned(), rows });
            }
        }
    }
    if sections.is_empty() {
        return Err(ReviewCopyError::Unreadable);
    }
    Ok(ReviewerCopy { file_name: file_name.to_owned(), kind: ReviewFileKind::Docx, sections })
}

fn unescape_markdown(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut chars = text.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '\\' && chars.peek().is_some_and(char::is_ascii_punctuation) {
            if let Some(next) = chars.next() {
                out.push(next);
            }
        } else {
            out.push(c);
        }
    }
    out
}

fn take_until_unescaped(chars: &mut std::iter::Peekable<std::str::Chars<'_>>, close: char) -> Option<String> {
    let mut raw = String::new();
    while let Some(c) = chars.next() {
        if c == '\\' {
            raw.push(c);
            raw.push(chars.next()?);
        } else if c == close {
            return Some(raw);
        } else {
            raw.push(c);
        }
    }
    None
}

fn image_alt_of(line: &str) -> Option<String> {
    let mut chars = line.strip_prefix("![")?.chars().peekable();
    let alt = take_until_unescaped(&mut chars, ']')?;
    if chars.next()? != '(' {
        return None;
    }
    if chars.peek() == Some(&'<') {
        chars.next();
        take_until_unescaped(&mut chars, '>')?;
        if chars.next()? != ')' {
            return None;
        }
    } else {
        take_until_unescaped(&mut chars, ')')?;
    }
    if chars.next().is_some() {
        return None;
    }
    Some(unescape_markdown(&alt))
}

fn caption_of(line: &str) -> Option<String> {
    let inner = line.strip_prefix('*')?.strip_suffix('*')?;
    Some(unescape_markdown(inner))
}

enum MarkdownBlock {
    Heading(String),
    Paragraph(Vec<String>),
}

fn is_attribution_block(lines: &[String]) -> bool {
    let last = lines.len().saturating_sub(1);
    !lines.is_empty()
        && lines.iter().enumerate().all(|(i, line)| is_attribution_line(line) && (i == last || line.ends_with('\\')))
}

fn markdown_blocks(text: &str) -> Vec<MarkdownBlock> {
    let normalized = text.replace("\r\n", "\n");
    let mut blocks: Vec<MarkdownBlock> = Vec::new();
    let mut current: Vec<String> = Vec::new();
    let mut lines = normalized.split('\n').map(str::to_owned).collect::<Vec<_>>();
    lines.push(String::new());
    for line in lines {
        if line.trim().is_empty() {
            if !current.is_empty() {
                blocks.push(MarkdownBlock::Paragraph(std::mem::take(&mut current)));
            }
        } else {
            current.push(line);
        }
    }
    let mut out: Vec<MarkdownBlock> = Vec::new();
    for block in blocks {
        match block {
            MarkdownBlock::Paragraph(lines) => {
                let heading = lines.first().and_then(|first| first.strip_prefix("## "));
                match heading {
                    Some(heading) => {
                        if matches!(out.last(), Some(MarkdownBlock::Paragraph(prev)) if is_attribution_block(prev)) {
                            out.pop();
                        }
                        out.push(MarkdownBlock::Heading(unescape_markdown(heading.trim_end())));
                        let rest: Vec<String> = lines.iter().skip(1).cloned().collect();
                        if !rest.is_empty() {
                            out.push(MarkdownBlock::Paragraph(rest));
                        }
                    }
                    None => out.push(MarkdownBlock::Paragraph(lines)),
                }
            }
            MarkdownBlock::Heading(_) => {}
        }
    }
    out
}

pub fn read_markdown_copy(text: &str, file_name: &str) -> Result<ReviewerCopy, ReviewCopyError> {
    let mut sections: Vec<ReviewSection> = Vec::new();
    for block in markdown_blocks(text) {
        match block {
            MarkdownBlock::Heading(heading) => sections.push(ReviewSection { heading, rows: Vec::new() }),
            MarkdownBlock::Paragraph(lines) => {
                let Some(section) = sections.last_mut() else { continue };
                let target_row = |kind: ReviewRowKind, target_text: String| ReviewRow { kind, source_text: None, target_text };
                match lines.first().and_then(|first| image_alt_of(first)) {
                    Some(alt) => {
                        if !alt.is_empty() {
                            section.rows.push(target_row(ReviewRowKind::Alt, alt));
                        }
                        if let Some(caption) = lines.get(1).and_then(|line| caption_of(line)) {
                            section.rows.push(target_row(ReviewRowKind::Caption, caption));
                        }
                    }
                    None => {
                        let paragraph = lines.iter().map(|line| unescape_markdown(line)).collect::<Vec<_>>().join("\n");
                        section.rows.push(target_row(ReviewRowKind::Text, paragraph));
                    }
                }
            }
        }
    }
    if sections.is_empty() {
        return Err(ReviewCopyError::Unreadable);
    }
    Ok(ReviewerCopy { file_name: file_name.to_owned(), kind: ReviewFileKind::Markdown, sections })
}

struct ChapterContext {
    id: i64,
    ord: i64,
    title: Option<String>,
    sources: HashSet<String>,
    images: Vec<ExportImage>,
}

fn load_context(conn: ReadHandle<'_>) -> SqlResult<Vec<ChapterContext>> {
    let mut chapters = Vec::new();
    let mut stmt = conn.prepare("SELECT id, ord, title FROM chapter ORDER BY ord, id")?;
    let heads = stmt
        .query_map([], |row| Ok((row.get::<_, i64>(0)?, row.get::<_, i64>(1)?, row.get::<_, Option<String>>(2)?)))?
        .collect::<SqlResult<Vec<_>>>()?;
    let mut sources_stmt = conn.prepare(
        "SELECT source_text FROM segment WHERE chapter_id = ?1 AND retired_at IS NULL AND is_omitted = 0",
    )?;
    let mut assets_stmt = conn.prepare("SELECT id, file_name, source_url FROM asset WHERE chapter_id = ?1 ORDER BY id")?;
    for (id, ord, title) in heads {
        let sources = sources_stmt
            .query_map([id], |row| row.get::<_, String>(0))?
            .collect::<SqlResult<HashSet<_>>>()?;
        let images = assets_stmt
            .query_map([id], |row| {
                Ok(ExportImage {
                    chapter_id: id,
                    chapter_ord: ord,
                    asset_id: row.get(0)?,
                    file_name: row.get(1)?,
                    source_url: row.get(2)?,
                })
            })?
            .collect::<SqlResult<Vec<_>>>()?;
        chapters.push(ChapterContext { id, ord, title, sources, images });
    }
    Ok(chapters)
}

fn is_image_cell(images: &[ExportImage], cell: &str) -> bool {
    images.iter().any(|image| {
        image.source_url.as_deref() == Some(cell)
            || image.file_name == cell
            || cell.ends_with(&format!("{IMAGE_DIR_SUFFIX}/{}", copied_name(image)))
    })
}

fn docx_score(section: &ReviewSection, chapter: &ChapterContext) -> usize {
    section
        .rows
        .iter()
        .filter(|row| row.source_text.as_deref().is_some_and(|s| !s.is_empty() && chapter.sources.contains(s)))
        .count()
}

fn same_title(heading: &str, chapter: &ChapterContext) -> bool {
    one_line(heading) == one_line(chapter.title.as_deref().unwrap_or(""))
}

fn match_docx_section(section: &ReviewSection, chapters: &[ChapterContext]) -> Option<usize> {
    let scores: Vec<usize> = chapters.iter().map(|chapter| docx_score(section, chapter)).collect();
    let best = scores.iter().copied().max().filter(|&score| score > 0)?;
    let tied: Vec<usize> = scores.iter().enumerate().filter(|&(_, &s)| s == best).map(|(i, _)| i).collect();
    match tied.as_slice() {
        [only] => Some(*only),
        _ => {
            let titled: Vec<usize> = tied.into_iter().filter(|&i| chapters.get(i).is_some_and(|c| same_title(&section.heading, c))).collect();
            match titled.as_slice() {
                [only] => Some(*only),
                _ => None,
            }
        }
    }
}

fn match_markdown_section(section: &ReviewSection, chapters: &[ChapterContext]) -> Option<usize> {
    let wanted = one_line(&section.heading);
    let hits: Vec<usize> = chapters
        .iter()
        .enumerate()
        .filter(|(_, chapter)| heading_text(chapter.title.as_deref(), chapter.ord) == wanted)
        .map(|(i, _)| i)
        .collect();
    match hits.as_slice() {
        [only] => Some(*only),
        _ => None,
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReplacedCopy {
    pub file_name: String,
    pub stale: bool,
    /// Groups the user decided by hand; importing again discards them.
    pub user_group_count: usize,
    /// Groups the user accepted; importing again discards them, and the text they wrote stays.
    pub accepted_group_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlannedChapter {
    pub section_index: usize,
    pub chapter_id: i64,
    pub chapter_ord: i64,
    pub title: Option<String>,
    pub rows: Vec<ReviewRow>,
    pub replaces: Option<ReplacedCopy>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SkippedSection {
    pub heading: String,
    pub row_count: usize,
}

/// Parsed file plus the Chapters it matched. Nothing is written until [`confirm_import`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReviewerImportPlan {
    pub copy: ReviewerCopy,
    pub chapters: Vec<PlannedChapter>,
    pub skipped: Vec<SkippedSection>,
    pub image_rows_ignored: usize,
}

pub(super) fn at_most_one<T>(found: SqlResult<T>) -> SqlResult<Option<T>> {
    match found {
        Ok(value) => Ok(Some(value)),
        Err(SqlError::QueryReturnedNoRows) => Ok(None),
        Err(error) => Err(error),
    }
}

fn existing_copy(conn: ReadHandle<'_>, chapter_id: i64) -> SqlResult<Option<ReplacedCopy>> {
    let found = at_most_one(conn.query_row(
        "SELECT file_name, stale_at IS NOT NULL FROM review_chapter WHERE chapter_id = ?1",
        [chapter_id],
        |row| Ok((row.get::<_, String>(0)?, row.get::<_, bool>(1)?)),
    ))?;
    let Some((file_name, stale)) = found else { return Ok(None) };
    Ok(Some(ReplacedCopy {
        file_name,
        stale,
        user_group_count: user_group_count(conn, chapter_id)?,
        accepted_group_count: accepted_group_count(conn, chapter_id)?,
    }))
}

pub fn plan_import(conn: ReadHandle<'_>, copy: &ReviewerCopy) -> Result<ReviewerImportPlan, ReviewCopyError> {
    let chapters = load_context(conn)?;
    let mut targets: Vec<Option<usize>> = copy
        .sections
        .iter()
        .map(|section| match copy.kind {
            ReviewFileKind::Docx => match_docx_section(section, &chapters),
            ReviewFileKind::Markdown => match_markdown_section(section, &chapters),
        })
        .collect();
    let snapshot = targets.clone();
    for (i, target) in targets.iter_mut().enumerate() {
        if target.is_some() && snapshot.iter().enumerate().any(|(j, other)| j != i && *other == *target) {
            *target = None;
        }
    }

    let mut planned = Vec::new();
    let mut skipped = Vec::new();
    let mut image_rows_ignored = 0;
    for (section_index, (section, target)) in copy.sections.iter().zip(&targets).enumerate() {
        let Some(chapter) = target.and_then(|i| chapters.get(i)) else {
            skipped.push(SkippedSection { heading: section.heading.clone(), row_count: section.rows.len() });
            continue;
        };
        let rows: Vec<ReviewRow> = section
            .rows
            .iter()
            .filter(|row| {
                let is_image = copy.kind == ReviewFileKind::Docx
                    && row.source_text.as_deref() == Some(row.target_text.as_str())
                    && is_image_cell(&chapter.images, &row.target_text);
                if is_image {
                    image_rows_ignored += 1;
                }
                !is_image
            })
            .cloned()
            .collect();
        if rows.is_empty() {
            skipped.push(SkippedSection { heading: section.heading.clone(), row_count: 0 });
            continue;
        }
        planned.push(PlannedChapter {
            section_index,
            chapter_id: chapter.id,
            chapter_ord: chapter.ord,
            title: chapter.title.clone(),
            rows,
            replaces: existing_copy(conn, chapter.id)?,
        });
    }
    if planned.is_empty() {
        return Err(ReviewCopyError::NoMatch);
    }
    Ok(ReviewerImportPlan { copy: copy.clone(), chapters: planned, skipped, image_rows_ignored })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ImportSummary {
    pub chapter_count: usize,
    pub row_count: usize,
    pub replaced_count: usize,
}

#[derive(Debug)]
pub enum ConfirmError {
    /// The Chapters the file matches now differ from the ones previewed; nothing was written.
    Stale,
    Sql(SqlError),
}

impl From<SqlError> for ConfirmError {
    fn from(error: SqlError) -> Self {
        Self::Sql(error)
    }
}

/// The only writer of `review_chapter` / `review_row` (AD-52 rule 3). Call it inside the write
/// transaction: the match is redone there, and a replaced copy is deleted then inserted, never
/// `REPLACE`d, so the new copy gets a fresh id.
pub fn confirm_import(tx: &Transaction<'_>, previewed: &ReviewerImportPlan) -> Result<ImportSummary, ConfirmError> {
    let current = match plan_import(tx, &previewed.copy) {
        Ok(plan) => plan,
        Err(ReviewCopyError::NoMatch) => return Err(ConfirmError::Stale),
        Err(ReviewCopyError::Sql(e)) => return Err(ConfirmError::Sql(e)),
        Err(_) => return Err(ConfirmError::Stale),
    };
    let written = |plan: &ReviewerImportPlan| -> Vec<(usize, i64, Vec<ReviewRow>)> {
        plan.chapters.iter().map(|c| (c.section_index, c.chapter_id, c.rows.clone())).collect()
    };
    if written(&current) != written(previewed) {
        return Err(ConfirmError::Stale);
    }

    let mut row_count = 0;
    let mut replaced_count = 0;
    for chapter in &current.chapters {
        delete_alignment_of_chapter(tx, chapter.chapter_id)?;
        tx.execute(
            "DELETE FROM review_row WHERE review_chapter_id IN (SELECT id FROM review_chapter WHERE chapter_id = ?1)",
            [chapter.chapter_id],
        )?;
        if chapter.replaces.is_some() {
            replaced_count += 1;
        }
        tx.execute("DELETE FROM review_chapter WHERE chapter_id = ?1", [chapter.chapter_id])?;
        tx.execute(
            "INSERT INTO review_chapter (chapter_id, file_name, file_kind, imported_at, stale_at) \
             VALUES (?1, ?2, ?3, strftime('%Y-%m-%dT%H:%M:%fZ','now'), NULL)",
            (chapter.chapter_id, &current.copy.file_name, current.copy.kind.as_str()),
        )?;
        let review_chapter_id = tx.last_insert_rowid();
        let mut insert = tx.prepare(
            "INSERT INTO review_row (review_chapter_id, ord, kind, source_text, target_text) VALUES (?1, ?2, ?3, ?4, ?5)",
        )?;
        for (ord, row) in chapter.rows.iter().enumerate() {
            insert.execute((
                review_chapter_id,
                i64::try_from(ord).unwrap_or(i64::MAX),
                row.kind.as_str(),
                &row.source_text,
                &row.target_text,
            ))?;
            row_count += 1;
        }
        drop(insert);
        align_chapter(tx, review_chapter_id)?;
    }
    Ok(ImportSummary { chapter_count: current.chapters.len(), row_count, replaced_count })
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReviewCopy {
    pub id: i64,
    pub chapter_id: i64,
    pub file_name: String,
    pub file_kind: ReviewFileKind,
    pub imported_at: String,
    pub rows: Vec<ReviewRow>,
    /// `review_row.id` of each entry of `rows`, same order.
    pub row_ids: Vec<i64>,
}

/// The one reader of an imported reviewer copy (AD-52 rule 6). A Chapter without a copy and a
/// Chapter whose boundary changed after import are typed errors, never an empty row list.
pub fn read_review_copy(store: &Store, chapter_id: i64) -> Result<ReviewCopy, ReviewCopyError> {
    let found = store.read(|conn| {
        let head: Option<(i64, String, String, String, Option<String>)> = at_most_one(conn.query_row(
            "SELECT id, file_name, file_kind, imported_at, stale_at FROM review_chapter WHERE chapter_id = ?1",
            [chapter_id],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?, row.get(4)?)),
        ))?;
        let Some((id, file_name, file_kind, imported_at, stale_at)) = head else {
            return Ok(None);
        };
        let mut stmt = conn.prepare(
            "SELECT kind, source_text, target_text, id FROM review_row WHERE review_chapter_id = ?1 ORDER BY ord",
        )?;
        let rows = stmt
            .query_map([id], |row| Ok((row.get::<_, String>(0)?, row.get(1)?, row.get(2)?, row.get::<_, i64>(3)?)))?
            .collect::<SqlResult<Vec<(String, Option<String>, String, i64)>>>()?;
        Ok(Some((id, file_name, file_kind, imported_at, stale_at, rows)))
    })?;
    let Some((id, file_name, file_kind, imported_at, stale_at, rows)) = found else {
        return Err(ReviewCopyError::NotImported);
    };
    if stale_at.is_some() {
        return Err(ReviewCopyError::Stale);
    }
    let file_kind = ReviewFileKind::from_column(&file_kind).ok_or(ReviewCopyError::Unreadable)?;
    let row_ids = rows.iter().map(|(_, _, _, id)| *id).collect();
    let rows = rows
        .into_iter()
        .map(|(kind, source_text, target_text, _)| {
            ReviewRowKind::from_column(&kind).map(|kind| ReviewRow { kind, source_text, target_text })
        })
        .collect::<Option<Vec<_>>>()
        .ok_or(ReviewCopyError::Unreadable)?;
    Ok(ReviewCopy { id, chapter_id, file_name, file_kind, imported_at, rows, row_ids })
}
