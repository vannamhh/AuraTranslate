//! The Work genre list: rows of `global.db` table `genre`, edited by the user.

use std::sync::{Arc, Mutex};

use crate::core::store::{
    ReadHandle, SqlError, Store, StoreError, Transaction, is_unique_constraint_violation,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Genre {
    pub id: String,
    pub name: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GenreError {
    Store(StoreError),
    NameEmpty,
    NameTaken { name: String },
    InUse { count: usize },
    NotFound,
}

impl From<StoreError> for GenreError {
    fn from(e: StoreError) -> Self {
        GenreError::Store(e)
    }
}

fn clean_name(raw: &str) -> Result<String, GenreError> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Err(GenreError::NameEmpty);
    }
    Ok(trimmed.to_owned())
}

/// SQLite `UNIQUE` folds ASCII case only, so Unicode case is compared here.
fn folded(name: &str) -> String {
    name.to_lowercase()
}

pub fn list(store: &Store) -> Result<Vec<Genre>, StoreError> {
    store.read(|conn: ReadHandle<'_>| {
        let mut stmt = conn.prepare("SELECT id, name FROM genre ORDER BY seq")?;
        let rows = stmt.query_map([], |row| {
            Ok(Genre {
                id: row.get(0)?,
                name: row.get(1)?,
            })
        })?;
        rows.collect()
    })
}

fn name_taken(rows: &[Genre], name: &str, except_id: Option<&str>) -> bool {
    let key = folded(name);
    rows.iter()
        .any(|g| Some(g.id.as_str()) != except_id && folded(&g.name) == key)
}

/// Appends a genre at the end of the list; the id comes from the table sequence, so a deleted
/// id is never issued again.
pub fn create(store: &Store, name: &str) -> Result<Genre, GenreError> {
    let name = clean_name(name)?;
    if name_taken(&list(store)?, &name, None) {
        return Err(GenreError::NameTaken { name });
    }
    let name_for_write = name.clone();
    let collided = Arc::new(Mutex::new(false));
    let collided_in_write = Arc::clone(&collided);
    let result = store.write(move |tx: &Transaction<'_>| {
        let inserted = tx.execute(
            "INSERT INTO genre (id, name) VALUES (\
               'g' || ((SELECT COALESCE(MAX(seq), 0) FROM sqlite_sequence WHERE name = 'genre') + 1), \
               ?1)",
            [&name_for_write],
        );
        if let Err(e) = &inserted {
            flag_unique(&collided_in_write, e);
        }
        inserted?;
        tx.query_row(
            "SELECT id FROM genre WHERE seq = last_insert_rowid()",
            [],
            |row| row.get::<_, String>(0),
        )
    });
    match result {
        Ok(id) => Ok(Genre { id, name }),
        Err(_) if was_flagged(&collided) => Err(GenreError::NameTaken { name }),
        Err(e) => Err(e.into()),
    }
}

fn flag_unique(flag: &Mutex<bool>, e: &SqlError) {
    if is_unique_constraint_violation(e) {
        *flag.lock().unwrap_or_else(|p| p.into_inner()) = true;
    }
}

fn was_flagged(flag: &Mutex<bool>) -> bool {
    *flag.lock().unwrap_or_else(|p| p.into_inner())
}

pub fn rename(store: &Store, id: &str, new_name: &str) -> Result<(), GenreError> {
    let new_name = clean_name(new_name)?;
    let rows = list(store)?;
    if !rows.iter().any(|g| g.id == id) {
        return Err(GenreError::NotFound);
    }
    if name_taken(&rows, &new_name, Some(id)) {
        return Err(GenreError::NameTaken { name: new_name });
    }
    let id_for_write = id.to_owned();
    let name_for_write = new_name.clone();
    let collided = Arc::new(Mutex::new(false));
    let collided_in_write = Arc::clone(&collided);
    let result = store.write(move |tx: &Transaction<'_>| {
        let updated = tx.execute(
            "UPDATE genre SET name = ?1 WHERE id = ?2",
            (&name_for_write, &id_for_write),
        );
        if let Err(e) = &updated {
            flag_unique(&collided_in_write, e);
        }
        updated
    });
    match result {
        Ok(_) => Ok(()),
        Err(_) if was_flagged(&collided) => Err(GenreError::NameTaken { name: new_name }),
        Err(e) => Err(e.into()),
    }
}

/// Refuses while `works_using` is above zero: the caller counts the Works carrying the id.
pub fn delete(store: &Store, id: &str, works_using: usize) -> Result<(), GenreError> {
    if works_using > 0 {
        return Err(GenreError::InUse { count: works_using });
    }
    let id_for_write = id.to_owned();
    let changed = store.write(move |tx: &Transaction<'_>| {
        tx.execute("DELETE FROM genre WHERE id = ?1", [&id_for_write])
    })?;
    if changed == 0 {
        return Err(GenreError::NotFound);
    }
    Ok(())
}
