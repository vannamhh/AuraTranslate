use std::collections::BTreeMap;

use crate::core::i18n::{IpcError, MessageKey};
use crate::core::scope::{ScopeError, ScopeResolver};
use crate::core::store::{ReadHandle, Store, StoreError, Transaction};

// Literal: `tests/scope_boundary.rs` bans `ScopeKind` outside `core/scope`.
const TRANSLATOR_NAME_SCOPE_KIND: &str = "translator_name";

pub const TRANSLATOR_NAME_KEY: &str = "name";

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AttributionError {
    Store(StoreError),
    Scope(ScopeError),
}

impl From<StoreError> for AttributionError {
    fn from(e: StoreError) -> Self {
        AttributionError::Store(e)
    }
}

impl From<ScopeError> for AttributionError {
    fn from(e: ScopeError) -> Self {
        AttributionError::Scope(e)
    }
}

impl From<AttributionError> for IpcError {
    fn from(err: AttributionError) -> Self {
        match err {
            AttributionError::Store(e) => e.into(),
            AttributionError::Scope(_) => {
                IpcError::new("attribution.scope_error", MessageKey::AttributionScopeError, BTreeMap::new(), false)
            }
        }
    }
}

fn load_rows(store: &Store) -> Result<BTreeMap<String, String>, StoreError> {
    store.read(|conn: ReadHandle<'_>| {
        let mut stmt = conn.prepare("SELECT key, value FROM translator_name ORDER BY key")?;
        let mut rows = stmt.query([])?;
        let mut out = BTreeMap::new();
        while let Some(row) = rows.next()? {
            out.insert(row.get::<_, String>(0)?, row.get::<_, String>(1)?);
        }
        Ok(out)
    })
}

pub fn resolve_translator_name(resolver: &ScopeResolver, global: &Store) -> Result<Option<String>, AttributionError> {
    let rows = load_rows(global)?;
    let resolved = resolver.apply_override(TRANSLATOR_NAME_SCOPE_KIND, &rows, None)?;
    Ok(resolved
        .get(TRANSLATOR_NAME_KEY)
        .map(|r| r.value().trim().to_owned())
        .filter(|name| !name.is_empty()))
}

pub fn write_translator_name(global: &Store, name: &str) -> Result<(), StoreError> {
    let name = name.trim().to_owned();
    global.write(move |tx: &Transaction<'_>| {
        if name.is_empty() {
            tx.execute("DELETE FROM translator_name WHERE key = ?1", [TRANSLATOR_NAME_KEY])?;
        } else {
            tx.execute(
                "INSERT INTO translator_name (key, value, updated_at)
                 VALUES (?1, ?2, strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
                 ON CONFLICT (key) DO UPDATE SET
                   value      = excluded.value,
                   updated_at = excluded.updated_at",
                (TRANSLATOR_NAME_KEY, &name),
            )?;
        }
        Ok(())
    })?;
    Ok(())
}
