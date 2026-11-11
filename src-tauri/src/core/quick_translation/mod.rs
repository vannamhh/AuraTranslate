//! Cache of one-sentence quick translations, `quick-translation.db` (AD-64).
//!
//! `QuickTranslationStore::open` is the only place that opens this store
//! (`tests/quick_translation_boundary.rs`). Rows are a derived cache: a schema mismatch or an
//! unreadable file deletes the file and rebuilds it empty.

use std::path::PathBuf;

use sha2::{Digest, Sha256};
use unicode_normalization::UnicodeNormalization;

use crate::core::store::{
    QUICK_TRANSLATION_MIGRATIONS, Store, StoreError, StoreKind, StoreSpec, Transaction,
};

/// The one place a cache key is made: NFC, then trimmed.
pub fn quick_key(text: &str) -> String {
    text.nfc().collect::<String>().trim().to_owned()
}

pub const QUICK_BATCH_MAX_BLOCKS: usize = 40;
pub const QUICK_BATCH_MAX_CHARS: usize = 4000;

/// All input positions whose text has the same [`quick_key`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KeyGroup {
    pub key: String,
    pub indices: Vec<usize>,
}

/// Groups `texts` by key in order of first appearance. A text whose key is empty has no group.
pub fn group_by_key(texts: &[String]) -> Vec<KeyGroup> {
    let mut groups: Vec<KeyGroup> = Vec::new();
    let mut seen: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
    for (index, text) in texts.iter().enumerate() {
        let key = quick_key(text);
        if key.is_empty() {
            continue;
        }
        match seen.get(&key) {
            Some(&at) => groups[at].indices.push(index),
            None => {
                seen.insert(key.clone(), groups.len());
                groups.push(KeyGroup {
                    key,
                    indices: vec![index],
                });
            }
        }
    }
    groups
}

/// Splits `groups` (page order) at the batch caps: how many lead the call. At least one is
/// taken, so a block alone over [`QUICK_BATCH_MAX_CHARS`] still goes, in a call of its own.
pub fn batch_prefix_len(groups: &[KeyGroup]) -> usize {
    let mut chars = 0usize;
    for (taken, group) in groups.iter().enumerate() {
        chars += group.key.chars().count();
        if taken >= QUICK_BATCH_MAX_BLOCKS || (taken > 0 && chars > QUICK_BATCH_MAX_CHARS) {
            return taken;
        }
    }
    groups.len()
}

fn source_hash(key: &str) -> Vec<u8> {
    Sha256::digest(key.as_bytes()).to_vec()
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QuickHit {
    pub translated_text: String,
    pub model: String,
}

pub struct QuickTranslationStore {
    store: Store,
}

impl QuickTranslationStore {
    pub fn open(path: PathBuf) -> Result<Self, StoreError> {
        crate::core::store::delete_if_schema_version_differs(
            &path,
            StoreKind::QuickTranslation,
            QUICK_TRANSLATION_MIGRATIONS,
        )?;
        let store = Store::open(StoreSpec::quick_translation(path))?;
        Ok(Self { store })
    }

    pub fn close(&self) {
        self.store.close();
    }

    pub fn lookup(&self, text: &str) -> Result<Option<QuickHit>, StoreError> {
        let key = quick_key(text);
        if key.is_empty() {
            return Ok(None);
        }
        let hash = source_hash(&key);
        self.store.read(|conn| {
            let mut stmt = conn.prepare(
                "SELECT translated_text, model FROM quick_translation WHERE source_hash = ?1",
            )?;
            let mut rows = stmt.query([&hash])?;
            match rows.next()? {
                Some(row) => Ok(Some(QuickHit {
                    translated_text: row.get(0)?,
                    model: row.get(1)?,
                })),
                None => Ok(None),
            }
        })
    }

    /// One lookup per key, in one read. `keys` are already [`quick_key`] output.
    pub fn lookup_keys(&self, keys: &[String]) -> Result<Vec<Option<QuickHit>>, StoreError> {
        self.store.read(|conn| {
            let mut stmt = conn.prepare(
                "SELECT translated_text, model FROM quick_translation WHERE source_hash = ?1",
            )?;
            keys.iter()
                .map(|key| {
                    let hash = source_hash(key);
                    let mut rows = stmt.query([&hash])?;
                    match rows.next()? {
                        Some(row) => Ok(Some(QuickHit {
                            translated_text: row.get(0)?,
                            model: row.get(1)?,
                        })),
                        None => Ok(None),
                    }
                })
                .collect()
        })
    }

    /// Stores one complete translation. `still_current` runs inside the write job, so a run
    /// cancelled or superseded before the job starts writes nothing. `replace` is `Dịch lại`;
    /// otherwise an existing row stays. Returns whether a row was written.
    pub fn save(
        &self,
        text: &str,
        translation: &str,
        model: &str,
        replace: bool,
        still_current: impl Fn() -> bool + Send + 'static,
    ) -> Result<bool, StoreError> {
        let key = quick_key(text);
        if key.is_empty() || translation.trim().is_empty() {
            return Ok(false);
        }
        let hash = source_hash(&key);
        let translation = translation.to_owned();
        let model = model.to_owned();
        self.store.write(move |tx: &Transaction<'_>| {
            if !still_current() {
                return Ok(false);
            }
            let verb = if replace {
                "INSERT OR REPLACE"
            } else {
                "INSERT OR IGNORE"
            };
            let sql = format!(
                "{verb} INTO quick_translation \
                 (source_hash, source_text, translated_text, model, created_at) \
                 VALUES (?1, ?2, ?3, ?4, strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))"
            );
            let changed = tx.execute(&sql, (&hash, &key, &translation, &model))?;
            Ok(changed > 0)
        })
    }

    pub fn count(&self) -> Result<u64, StoreError> {
        self.store.read(|conn| {
            conn.query_row("SELECT COUNT(*) FROM quick_translation", [], |row| {
                row.get::<_, i64>(0)
            })
            .map(|n| n as u64)
        })
    }
}
