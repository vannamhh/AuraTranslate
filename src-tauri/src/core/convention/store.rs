use std::collections::BTreeMap;

use crate::core::delimited_text::{ReadError, WriteError};
use crate::core::i18n::{IpcError, MessageKey};
use crate::core::store::{ReadHandle, SqlError, SqlResult, Store, StoreError, Transaction};

use super::ignore::ConventionSignature;
use super::normalize::rule_key;
use super::rule::{Rule, RuleTier};

/// A stored rule as the management list shows it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuleRow {
    pub id: i64,
    pub rule_key: String,
    pub avoid_phrase: String,
    pub replacement_phrase: Option<String>,
    pub enabled: bool,
}

impl RuleRow {
    pub fn into_rule(self, tier: RuleTier) -> Rule {
        Rule {
            key: self.rule_key,
            avoid_phrase: self.avoid_phrase,
            replacement_phrase: self.replacement_phrase,
            tier,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConventionStoreError {
    Store(StoreError),
    EmptyAvoidPhrase,
    DuplicateAvoidPhrase,
    RuleMissing,
    WorkTierUnavailable,
    ImportDelimiterUnresolved,
    ImportMissingColumn {
        column: &'static str,
    },
    ImportDuplicateColumn {
        column: &'static str,
    },
    ImportCellCountMismatch {
        line: usize,
        expected: usize,
        found: usize,
    },
    ImportBlankAvoidPhrase {
        line: usize,
    },
    ImportDuplicateAvoidPhrase {
        first_line: usize,
        second_line: usize,
    },
    ImportUnterminatedQuotedField {
        line: usize,
    },
    ImportFileTooLarge {
        limit: u64,
    },
    ImportNotUtf8 {
        path: String,
    },
    ImportReadFailed {
        path: String,
    },
    ImportDecisionUnknownRule {
        rule_key: String,
    },
    ImportStale,
    NoPendingImport,
    ExportWriteFailed {
        path: String,
    },
    DialogPathInvalid,
}

impl std::fmt::Display for ConventionStoreError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Store(e) => write!(f, "convention[store] {e}"),
            Self::EmptyAvoidPhrase => write!(f, "convention[empty_avoid_phrase]"),
            Self::DuplicateAvoidPhrase => write!(f, "convention[duplicate_avoid_phrase]"),
            Self::RuleMissing => write!(f, "convention[rule_missing]"),
            Self::WorkTierUnavailable => write!(f, "convention[work_tier_unavailable]"),
            Self::ImportDelimiterUnresolved => write!(f, "convention[import_delimiter_unresolved]"),
            Self::ImportMissingColumn { column } => {
                write!(f, "convention[import_missing_column] column={column}")
            }
            Self::ImportDuplicateColumn { column } => {
                write!(f, "convention[import_duplicate_column] column={column}")
            }
            Self::ImportCellCountMismatch {
                line,
                expected,
                found,
            } => write!(
                f,
                "convention[import_cell_count_mismatch] line={line} expected={expected} found={found}"
            ),
            Self::ImportBlankAvoidPhrase { line } => {
                write!(f, "convention[import_blank_avoid_phrase] line={line}")
            }
            Self::ImportDuplicateAvoidPhrase {
                first_line,
                second_line,
            } => write!(
                f,
                "convention[import_duplicate_avoid_phrase] first_line={first_line} second_line={second_line}"
            ),
            Self::ImportUnterminatedQuotedField { line } => {
                write!(
                    f,
                    "convention[import_unterminated_quoted_field] line={line}"
                )
            }
            Self::ImportFileTooLarge { limit } => {
                write!(f, "convention[import_file_too_large] limit={limit}")
            }
            Self::ImportNotUtf8 { path } => write!(f, "convention[import_not_utf8] path={path}"),
            Self::ImportReadFailed { path } => {
                write!(f, "convention[import_read_failed] path={path}")
            }
            Self::ImportDecisionUnknownRule { rule_key } => {
                write!(
                    f,
                    "convention[import_decision_unknown_rule] rule_key={rule_key}"
                )
            }
            Self::ImportStale => write!(f, "convention[import_stale]"),
            Self::NoPendingImport => write!(f, "convention[no_pending_import]"),
            Self::ExportWriteFailed { path } => {
                write!(f, "convention[export_write_failed] path={path}")
            }
            Self::DialogPathInvalid => write!(f, "convention[dialog_path_invalid]"),
        }
    }
}

impl std::error::Error for ConventionStoreError {}

impl From<StoreError> for ConventionStoreError {
    fn from(e: StoreError) -> Self {
        Self::Store(e)
    }
}

impl From<ReadError> for ConventionStoreError {
    fn from(e: ReadError) -> Self {
        match e {
            ReadError::ReadFailed { path, .. } => Self::ImportReadFailed { path },
            ReadError::TooLarge { limit, .. } => Self::ImportFileTooLarge { limit },
            ReadError::NotUtf8 { path } => Self::ImportNotUtf8 { path },
        }
    }
}

impl From<WriteError> for ConventionStoreError {
    fn from(e: WriteError) -> Self {
        Self::ExportWriteFailed { path: e.path }
    }
}

fn params(pairs: &[(&str, String)]) -> BTreeMap<String, String> {
    pairs
        .iter()
        .map(|(key, value)| ((*key).to_owned(), value.clone()))
        .collect()
}

impl From<ConventionStoreError> for IpcError {
    fn from(err: ConventionStoreError) -> Self {
        match err {
            ConventionStoreError::Store(e) => e.into(),
            ConventionStoreError::EmptyAvoidPhrase => IpcError::new(
                "convention.empty_avoid_phrase",
                MessageKey::ConventionEmptyAvoidPhrase,
                BTreeMap::new(),
                false,
            ),
            ConventionStoreError::DuplicateAvoidPhrase => IpcError::new(
                "convention.duplicate_avoid_phrase",
                MessageKey::ConventionDuplicateAvoidPhrase,
                BTreeMap::new(),
                false,
            ),
            ConventionStoreError::RuleMissing => IpcError::new(
                "convention.rule_missing",
                MessageKey::ConventionRuleMissing,
                BTreeMap::new(),
                false,
            ),
            ConventionStoreError::WorkTierUnavailable => IpcError::new(
                "convention.work_tier_unavailable",
                MessageKey::ConventionWorkTierUnavailable,
                BTreeMap::new(),
                false,
            ),
            ConventionStoreError::ImportDelimiterUnresolved => IpcError::new(
                "convention.import_delimiter_unresolved",
                MessageKey::ConventionImportDelimiterUnresolved,
                BTreeMap::new(),
                false,
            ),
            ConventionStoreError::ImportMissingColumn { column } => IpcError::new(
                "convention.import_missing_column",
                MessageKey::ConventionImportMissingColumn,
                params(&[("column", column.to_owned())]),
                false,
            ),
            ConventionStoreError::ImportDuplicateColumn { column } => IpcError::new(
                "convention.import_duplicate_column",
                MessageKey::ConventionImportDuplicateColumn,
                params(&[("column", column.to_owned())]),
                false,
            ),
            ConventionStoreError::ImportCellCountMismatch {
                line,
                expected,
                found,
            } => IpcError::new(
                "convention.import_cell_count_mismatch",
                MessageKey::ConventionImportCellCountMismatch,
                params(&[
                    ("line", line.to_string()),
                    ("expected", expected.to_string()),
                    ("found", found.to_string()),
                ]),
                false,
            ),
            ConventionStoreError::ImportBlankAvoidPhrase { line } => IpcError::new(
                "convention.import_blank_avoid_phrase",
                MessageKey::ConventionImportBlankAvoidPhrase,
                params(&[("line", line.to_string())]),
                false,
            ),
            ConventionStoreError::ImportDuplicateAvoidPhrase {
                first_line,
                second_line,
            } => IpcError::new(
                "convention.import_duplicate_avoid",
                MessageKey::ConventionImportDuplicateAvoid,
                params(&[
                    ("first_line", first_line.to_string()),
                    ("second_line", second_line.to_string()),
                ]),
                false,
            ),
            ConventionStoreError::ImportUnterminatedQuotedField { line } => IpcError::new(
                "convention.import_unterminated_quoted_field",
                MessageKey::ConventionImportUnterminatedQuotedField,
                params(&[("line", line.to_string())]),
                false,
            ),
            ConventionStoreError::ImportFileTooLarge { limit } => IpcError::new(
                "convention.import_file_too_large",
                MessageKey::ImportTooLarge,
                params(&[("limit", limit.to_string())]),
                false,
            ),
            ConventionStoreError::ImportNotUtf8 { path } => IpcError::new(
                "convention.import_not_utf8",
                MessageKey::ImportNotUtf8,
                params(&[("path", path)]),
                false,
            ),
            ConventionStoreError::ImportReadFailed { path } => IpcError::new(
                "convention.import_read_failed",
                MessageKey::IoReadFailed,
                params(&[("path", path)]),
                false,
            ),
            ConventionStoreError::ImportDecisionUnknownRule { rule_key } => IpcError::new(
                "convention.import_decision_unknown_rule",
                MessageKey::ConventionImportDecisionUnknownRule,
                params(&[("value", rule_key)]),
                false,
            ),
            ConventionStoreError::ImportStale => IpcError::new(
                "convention.import_stale",
                MessageKey::ConventionImportStale,
                BTreeMap::new(),
                false,
            ),
            ConventionStoreError::NoPendingImport => IpcError::new(
                "convention.no_pending_import",
                MessageKey::ConventionNoPendingImport,
                BTreeMap::new(),
                false,
            ),
            ConventionStoreError::ExportWriteFailed { path } => IpcError::new(
                "convention.export_write_failed",
                MessageKey::ConventionExportWriteFailed,
                params(&[("path", path)]),
                false,
            ),
            ConventionStoreError::DialogPathInvalid => IpcError::new(
                "convention.dialog_path_invalid",
                MessageKey::ConventionDialogPathInvalid,
                BTreeMap::new(),
                false,
            ),
        }
    }
}

const SELECT_RULES: &str = "SELECT id, rule_key, avoid_phrase, replacement_phrase, enabled \
                            FROM convention_rule ORDER BY rule_key";

fn row(row: &crate::core::store::Row<'_>) -> SqlResult<RuleRow> {
    let enabled: i64 = row.get(4)?;
    Ok(RuleRow {
        id: row.get(0)?,
        rule_key: row.get(1)?,
        avoid_phrase: row.get(2)?,
        replacement_phrase: row.get(3)?,
        enabled: enabled != 0,
    })
}

pub fn list_rules(store: &Store) -> Result<Vec<RuleRow>, StoreError> {
    store.read(|conn: ReadHandle<'_>| {
        let mut stmt = conn.prepare(SELECT_RULES)?;
        let rows = stmt.query_map([], row)?;
        rows.collect::<SqlResult<Vec<RuleRow>>>()
    })
}

/// Every rule of the store the transaction belongs to.
pub fn list_rules_in(tx: &Transaction<'_>) -> SqlResult<Vec<RuleRow>> {
    let mut stmt = tx.prepare(SELECT_RULES)?;
    let rows = stmt.query_map([], row)?;
    rows.collect::<SqlResult<Vec<RuleRow>>>()
}

/// The enabled rule with `key`, or `None` when it is gone or disabled.
pub fn find_active_rule(store: &Store, key: &str) -> Result<Option<RuleRow>, StoreError> {
    let key = key.to_owned();
    store.read(move |conn: ReadHandle<'_>| {
        let mut stmt = conn.prepare(
            "SELECT id, rule_key, avoid_phrase, replacement_phrase, enabled \
             FROM convention_rule WHERE rule_key = ?1 AND enabled = 1",
        )?;
        let mut rows = stmt.query_map([key], row)?;
        rows.next().transpose()
    })
}

/// Adds a rule. An empty replacement is stored as `NULL` ("do not use"). A blank avoid phrase or a
/// key already present is refused before anything is written.
pub fn add_rule(
    store: &Store,
    avoid_phrase: &str,
    replacement_phrase: Option<&str>,
) -> Result<i64, ConventionStoreError> {
    let avoid = avoid_phrase.trim().to_owned();
    if avoid.is_empty() {
        return Err(ConventionStoreError::EmptyAvoidPhrase);
    }
    let replacement = replacement_phrase
        .map(str::trim)
        .filter(|text| !text.is_empty())
        .map(str::to_owned);
    let key = rule_key(&avoid);
    let inserted = store.write(move |tx: &Transaction<'_>| {
        let existing: Option<(i64, i64)> = match tx.query_row(
            "SELECT id, enabled FROM convention_rule WHERE rule_key = ?1",
            [&key],
            |r| Ok((r.get(0)?, r.get(1)?)),
        ) {
            Ok(found) => Some(found),
            Err(SqlError::QueryReturnedNoRows) => None,
            Err(err) => return Err(err),
        };
        match existing {
            Some((_, enabled)) if enabled != 0 => return Ok(None),
            Some((id, _)) => {
                tx.execute(
                    "UPDATE convention_rule SET avoid_phrase = ?1, replacement_phrase = ?2, \
                     enabled = 1 WHERE id = ?3",
                    (&avoid, &replacement, id),
                )?;
                return Ok(Some(id));
            }
            None => {}
        }
        tx.execute(
            "INSERT INTO convention_rule \
             (avoid_phrase, replacement_phrase, rule_key, enabled, created_at) \
             VALUES (?1, ?2, ?3, 1, strftime('%Y-%m-%dT%H:%M:%fZ','now'))",
            (&avoid, &replacement, &key),
        )?;
        Ok(Some(tx.last_insert_rowid()))
    })?;
    inserted.ok_or(ConventionStoreError::DuplicateAvoidPhrase)
}

fn normalized(
    avoid_phrase: &str,
    replacement_phrase: Option<&str>,
) -> Result<(String, Option<String>, String), ConventionStoreError> {
    let avoid = avoid_phrase.trim().to_owned();
    if avoid.is_empty() {
        return Err(ConventionStoreError::EmptyAvoidPhrase);
    }
    let replacement = replacement_phrase
        .map(str::trim)
        .filter(|text| !text.is_empty())
        .map(str::to_owned);
    let key = rule_key(&avoid);
    Ok((avoid, replacement, key))
}

/// The row with `key` whether or not it is enabled.
pub fn find_row_by_key(store: &Store, key: &str) -> Result<Option<RuleRow>, StoreError> {
    let key = key.to_owned();
    store.read(move |conn: ReadHandle<'_>| {
        let mut stmt = conn.prepare(
            "SELECT id, rule_key, avoid_phrase, replacement_phrase, enabled \
             FROM convention_rule WHERE rule_key = ?1",
        )?;
        let mut rows = stmt.query_map([key], row)?;
        rows.next().transpose()
    })
}

/// Rewrites rule `id`. Another row of the same store already holding the new key is refused; a
/// row that is gone is `RuleMissing`. Nothing is written on a refusal.
pub fn edit_rule(
    store: &Store,
    id: i64,
    avoid_phrase: &str,
    replacement_phrase: Option<&str>,
) -> Result<(), ConventionStoreError> {
    let (avoid, replacement, key) = normalized(avoid_phrase, replacement_phrase)?;
    store.write(move |tx: &Transaction<'_>| {
        let exists: bool = tx.query_row(
            "SELECT EXISTS(SELECT 1 FROM convention_rule WHERE id = ?1)",
            [id],
            |r| r.get(0),
        )?;
        if !exists {
            return Ok(Err(ConventionStoreError::RuleMissing));
        }
        let clash: bool = tx.query_row(
            "SELECT EXISTS(SELECT 1 FROM convention_rule WHERE rule_key = ?1 AND id <> ?2)",
            (&key, id),
            |r| r.get(0),
        )?;
        if clash {
            return Ok(Err(ConventionStoreError::DuplicateAvoidPhrase));
        }
        tx.execute(
            "UPDATE convention_rule SET avoid_phrase = ?1, replacement_phrase = ?2, rule_key = ?3 \
             WHERE id = ?4",
            (&avoid, &replacement, &key, id),
        )?;
        Ok(Ok(()))
    })?
}

/// Deletes rule `id`; a row that is already gone is not an error.
pub fn delete_rule(store: &Store, id: i64) -> Result<(), ConventionStoreError> {
    store.write(move |tx: &Transaction<'_>| {
        tx.execute("DELETE FROM convention_rule WHERE id = ?1", [id])
    })?;
    Ok(())
}

/// Switches a Global rule off for the open Work: inserts the Work-tier row with the same key,
/// no replacement and `enabled = 0`. A Work row already holding the key refuses.
pub fn disable_in_work(
    work: &Store,
    avoid_phrase: &str,
    key: &str,
) -> Result<i64, ConventionStoreError> {
    let (avoid, key) = (avoid_phrase.to_owned(), key.to_owned());
    let inserted = work.write(move |tx: &Transaction<'_>| {
        let exists: bool = tx.query_row(
            "SELECT EXISTS(SELECT 1 FROM convention_rule WHERE rule_key = ?1)",
            [&key],
            |r| r.get(0),
        )?;
        if exists {
            return Ok(None);
        }
        tx.execute(
            "INSERT INTO convention_rule \
             (avoid_phrase, replacement_phrase, rule_key, enabled, created_at) \
             VALUES (?1, NULL, ?2, 0, strftime('%Y-%m-%dT%H:%M:%fZ','now'))",
            (&avoid, &key),
        )?;
        Ok(Some(tx.last_insert_rowid()))
    })?;
    inserted.ok_or(ConventionStoreError::DuplicateAvoidPhrase)
}

/// Switches a Global rule back on for the open Work: deletes the disabled Work row with `key`.
/// No such row is `RuleMissing`.
pub fn enable_in_work(work: &Store, key: &str) -> Result<(), ConventionStoreError> {
    let key = key.to_owned();
    let changed = work.write(move |tx: &Transaction<'_>| {
        tx.execute(
            "DELETE FROM convention_rule WHERE rule_key = ?1 AND enabled = 0",
            [&key],
        )
    })?;
    if changed == 0 {
        return Err(ConventionStoreError::RuleMissing);
    }
    Ok(())
}

/// A remembered "not an error" place as the management card shows it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IgnoreRow {
    pub id: i64,
    pub rule_key: String,
    pub avoid_phrase: String,
    pub context_before: String,
    pub context_after: String,
    pub created_at: String,
}

/// Remembers `signature` inside the caller's write transaction; the same signature again is the
/// same row. Returns whether a row was added.
pub fn add_ignore(tx: &Transaction<'_>, signature: &ConventionSignature) -> SqlResult<bool> {
    let added = tx.execute(
        "INSERT OR IGNORE INTO convention_ignore \
         (rule_key, avoid_phrase, context_before, context_after, created_at) \
         VALUES (?1, ?2, ?3, ?4, strftime('%Y-%m-%dT%H:%M:%fZ','now'))",
        (
            &signature.rule_key,
            &signature.phrase,
            &signature.before,
            &signature.after,
        ),
    )?;
    Ok(added > 0)
}

/// Every remembered signature of the Work, for the scan to filter with.
pub fn list_ignores(store: &Store) -> Result<Vec<ConventionSignature>, StoreError> {
    store.read(|conn: ReadHandle<'_>| {
        let mut stmt = conn.prepare(
            "SELECT rule_key, avoid_phrase, context_before, context_after \
             FROM convention_ignore ORDER BY id",
        )?;
        let rows = stmt.query_map([], |r| {
            Ok(ConventionSignature {
                rule_key: r.get(0)?,
                phrase: r.get(1)?,
                before: r.get(2)?,
                after: r.get(3)?,
            })
        })?;
        rows.collect::<SqlResult<Vec<ConventionSignature>>>()
    })
}

/// Every remembered place of the Work, newest first.
pub fn list_ignore_rows(store: &Store) -> Result<Vec<IgnoreRow>, StoreError> {
    store.read(|conn: ReadHandle<'_>| {
        let mut stmt = conn.prepare(
            "SELECT id, rule_key, avoid_phrase, context_before, context_after, created_at \
             FROM convention_ignore ORDER BY created_at DESC, id DESC",
        )?;
        let rows = stmt.query_map([], |r| {
            Ok(IgnoreRow {
                id: r.get(0)?,
                rule_key: r.get(1)?,
                avoid_phrase: r.get(2)?,
                context_before: r.get(3)?,
                context_after: r.get(4)?,
                created_at: r.get(5)?,
            })
        })?;
        rows.collect::<SqlResult<Vec<IgnoreRow>>>()
    })
}

/// Forgets one remembered place; `false` when the row is already gone.
pub fn remove_ignore(store: &Store, id: i64) -> Result<bool, StoreError> {
    store.write(move |tx: &Transaction<'_>| {
        Ok(tx.execute("DELETE FROM convention_ignore WHERE id = ?1", [id])? > 0)
    })
}

/// Forgets every remembered place and returns how many rows went.
pub fn clear_ignores(store: &Store) -> Result<u32, StoreError> {
    let removed =
        store.write(|tx: &Transaction<'_>| tx.execute("DELETE FROM convention_ignore", []))?;
    Ok(u32::try_from(removed).unwrap_or(u32::MAX))
}
