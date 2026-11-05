use std::collections::BTreeMap;

use crate::core::i18n::{IpcError, MessageKey};
use crate::core::store::{ReadHandle, SqlResult, Store, StoreError, Transaction};

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
    pub fn into_rule(self) -> Rule {
        Rule {
            key: self.rule_key,
            avoid_phrase: self.avoid_phrase,
            replacement_phrase: self.replacement_phrase,
            tier: RuleTier::Global,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConventionStoreError {
    Store(StoreError),
    EmptyAvoidPhrase,
    DuplicateAvoidPhrase,
}

impl std::fmt::Display for ConventionStoreError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Store(e) => write!(f, "convention[store] {e}"),
            Self::EmptyAvoidPhrase => write!(f, "convention[empty_avoid_phrase]"),
            Self::DuplicateAvoidPhrase => write!(f, "convention[duplicate_avoid_phrase]"),
        }
    }
}

impl std::error::Error for ConventionStoreError {}

impl From<StoreError> for ConventionStoreError {
    fn from(e: StoreError) -> Self {
        Self::Store(e)
    }
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
             VALUES (?1, ?2, ?3, 1, strftime('%Y-%m-%dT%H:%M:%fZ','now'))",
            (&avoid, &replacement, &key),
        )?;
        Ok(Some(tx.last_insert_rowid()))
    })?;
    inserted.ok_or(ConventionStoreError::DuplicateAvoidPhrase)
}
