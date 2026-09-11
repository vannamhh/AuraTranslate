//! Bilingual two-column CSV/TSV row parsing — Story 6.16, FR115.
//!
//! ─────────────────────────────────────────────────────────────────────────────
//! PURE `&str` IN, TYPED ROWS OUT — 0 fs, 0 network, 0 panic points
//! ─────────────────────────────────────────────────────────────────────────────
//! Same shape as [`crate::core::segment::import`]'s input step: a module that only turns
//! characters into structured data. `parse_rows` reuses the tokenizer already proven by
//! Story 3.10 ([`crate::core::glossary::exchange`], made `pub(crate)` for this story) —
//! NFR15's precedent, one RFC 4180 subset, not a second copy.
//!
//! ─────────────────────────────────────────────────────────────────────────────
//! ROW IDENTITY SURVIVES TO THE SEGMENT SPLIT
//! ─────────────────────────────────────────────────────────────────────────────
//! `core::segment::pipeline::Step::SplitSegments` needs to know, for every pair of
//! sentence-split lists, which one grouping they came from (to compute the AD-37 row flag
//! and detect a mismatched row) — so a row is never flattened into a single joined column
//! anywhere in this module.

use crate::core::glossary::exchange::{ParseIssue, logical_lines, split_fields};

pub use crate::core::glossary::exchange::Delimiter;

/// One data row of a bilingual CSV/TSV file — the raw cells, split but not trimmed
/// (trimming a cell is `commands::project`'s job, same discipline as every other cell in
/// this module: this file only tokenizes).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BilingualRow {
    /// 1-based row number as a person counts it in a text editor/spreadsheet — counts the
    /// header row too, when one is present. This is the number the I/O Matrix's mismatch
    /// list points at.
    pub row_number: usize,
    /// Every cell of the row, in file order.
    pub cells: Vec<String>,
}

/// Every way [`parse_rows`] refuses a file whole — §I/O Matrix "Unterminated quote"/"Fewer
/// than 2 columns".
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BilingualParseIssue {
    /// A quoted cell opened but never closed — same structural failure
    /// [`crate::core::glossary::exchange::ParseIssue::UnterminatedQuotedField`] names for the
    /// Glossary's CSV/TSV, same reason it stops parsing instead of reporting a
    /// `CellCountMismatch` on the last line: nothing after the open quote resolved into rows.
    UnterminatedQuotedField {
        /// Row where the open quote started (1-based).
        row: usize,
    },
    /// The widest row has fewer than two columns — nothing to pick a source/target role
    /// from. Whole-file refusal, no row number (there is no one row at fault; the shape of
    /// the file itself has no second column).
    TooFewColumns {
        /// Column count found — the widest row's cell count.
        found: usize,
    },
    /// The shared tokenizer reported an issue this module never expects from it — today
    /// `logical_lines` only returns `UnterminatedQuotedField`. Typed instead of panicking: a
    /// file the user picked must never take the process down (spec 6.16: 0 panic points).
    UnexpectedTokenizerIssue {
        /// `ParseIssue`'s own log diagnostic (no diacritics).
        detail: String,
    },
}

impl std::fmt::Display for BilingualParseIssue {
    /// ⚠️ NO diacritics — log diagnostic (NFR16).
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BilingualParseIssue::UnterminatedQuotedField { row } => {
                write!(f, "bilingual import[row {row}]: quoted field opened but never closed")
            }
            BilingualParseIssue::TooFewColumns { found } => {
                write!(f, "bilingual import: {found} column(s) found, need at least 2")
            }
            BilingualParseIssue::UnexpectedTokenizerIssue { detail } => {
                write!(f, "bilingual import: unexpected tokenizer issue: {detail}")
            }
        }
    }
}

impl std::error::Error for BilingualParseIssue {}

/// Parse `text` into rows of cells, split on `delimiter`. Row numbers are 1-based and count
/// every logical row seen on disk — a blank logical row is skipped (same "trailing/interior
/// blank line is not a row" rule the Glossary's `parse` already applies, same reason: an
/// editor's trailing newline must not read as a data row), but its own line number is still
/// consumed so row numbers keep matching what a person counting lines in the file would see.
///
/// Does **not** know about headers or which column is source/target — that is a decision
/// [`commands::project`] makes with the *columns already parsed*, so toggling the header
/// checkbox or swapping columns never re-reads the file (§I/O Matrix "Header checkbox
/// on"/"Swap columns": the preview rebuilds in memory).
pub fn parse_rows(text: &str, delimiter: Delimiter) -> Result<Vec<BilingualRow>, BilingualParseIssue> {
    let d = delimiter.as_char();
    let lines = logical_lines(text, Some(d), 1).map_err(|issue| match issue {
        ParseIssue::UnterminatedQuotedField { line } => {
            BilingualParseIssue::UnterminatedQuotedField { row: line }
        }
        // `logical_lines` only ever returns `UnterminatedQuotedField` today — every other
        // `ParseIssue` variant belongs to the Glossary's column-aware `parse`. Still typed, not
        // `unreachable!`: a future tokenizer change must surface as an error, not a panic.
        other => BilingualParseIssue::UnexpectedTokenizerIssue { detail: other.to_string() },
    })?;

    let mut rows = Vec::with_capacity(lines.len());
    for (row_number, raw) in lines {
        if raw.trim().is_empty() {
            continue;
        }
        rows.push(BilingualRow { row_number, cells: split_fields(raw, d) });
    }
    Ok(rows)
}

/// Widest row's cell count — 0 when `rows` is empty. Used by the command layer to refuse a
/// file with fewer than two columns *before* a preview is built (§I/O Matrix "Fewer than 2
/// columns").
pub fn widest_row_column_count(rows: &[BilingualRow]) -> usize {
    rows.iter().map(|r| r.cells.len()).max().unwrap_or(0)
}

/// A segment paired with its bilingual translation, ready to write — Story 6.16, AD-47 ③.
/// Distinct from [`crate::core::segment::split::SplitSegment`]: that type carries ONE flag
/// (the source flag, AD-37) because on every other import path the target starts out as a
/// mirror of the source (AD-46, AC2 of Story 2.5d) written later, one column at a time. A
/// bilingual row already HAS its own target text at import time, so this type carries both
/// texts and the ALREADY-MIRRORED flag (AD-46: at import, target flag = source flag) in one
/// place — the row-flag rule below computes both together, not by mirroring after the fact.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BilingualSegment {
    pub source_text: String,
    pub target_text: String,
    /// AD-37 flag, mirrored for both `is_paragraph_end`/`is_target_paragraph_end` at
    /// `commands::segment::insert_bilingual_segments` (AD-46: they start equal).
    pub is_paragraph_end: bool,
}

/// One mismatched row — §I/O Matrix "Mismatched row"/"Blank target cell". `chapter_index` is
/// 0-based (position in the import's chapter list); `row_number` is the 1-based file row
/// number [`BilingualRow::row_number`] already carries, echoed here so the preview UI can
/// list "Chapter N, row M" without re-walking rows.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BilingualMismatch {
    pub chapter_index: usize,
    pub row_number: usize,
    pub source_sentence_count: usize,
    pub target_sentence_count: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn splits_csv_rows_by_comma() {
        let rows = parse_rows("a,b\nc,d\n", Delimiter::Csv).unwrap();
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0].row_number, 1);
        assert_eq!(rows[0].cells, vec!["a".to_owned(), "b".to_owned()]);
        assert_eq!(rows[1].row_number, 2);
        assert_eq!(rows[1].cells, vec!["c".to_owned(), "d".to_owned()]);
    }

    #[test]
    fn splits_tsv_rows_by_tab() {
        let rows = parse_rows("a\tb\nc\td\n", Delimiter::Tsv).unwrap();
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0].cells, vec!["a".to_owned(), "b".to_owned()]);
    }

    #[test]
    fn blank_rows_are_skipped_but_still_count_line_numbers() {
        let rows = parse_rows("a,b\n\nc,d\n", Delimiter::Csv).unwrap();
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0].row_number, 1);
        assert_eq!(rows[1].row_number, 3);
    }

    #[test]
    fn unterminated_quote_is_refused_with_its_row_number() {
        let err = parse_rows("a,b\n\"c,d\n", Delimiter::Csv).unwrap_err();
        assert_eq!(err, BilingualParseIssue::UnterminatedQuotedField { row: 2 });
    }

    #[test]
    fn widest_row_wins_the_column_count() {
        let rows = parse_rows("a,b\nc,d,e\n", Delimiter::Csv).unwrap();
        assert_eq!(widest_row_column_count(&rows), 3);
    }

    #[test]
    fn empty_text_has_0_columns() {
        assert_eq!(widest_row_column_count(&[]), 0);
    }
}
