//! IPC surface of the Work genre list: list, create, rename, delete.

use std::collections::BTreeMap;

use crate::commands::library::indexer_is_missing;
use crate::core::genre::{self, GenreError};
use crate::core::i18n::{IpcError, MessageKey};
use crate::core::library::indexer::Indexer;
use crate::core::store::{Store, StoreError, StoreKind};

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct GenreWire {
    pub id: String,
    pub name: String,
}

impl From<genre::Genre> for GenreWire {
    fn from(g: genre::Genre) -> Self {
        GenreWire {
            id: g.id,
            name: g.name,
        }
    }
}

impl From<GenreError> for IpcError {
    fn from(e: GenreError) -> Self {
        match e {
            GenreError::Store(e) => e.into(),
            GenreError::NameEmpty => IpcError::new(
                "genre.name_empty",
                MessageKey::GenreNameEmpty,
                BTreeMap::new(),
                false,
            ),
            GenreError::NameTaken { name } => IpcError::new(
                "genre.name_taken",
                MessageKey::GenreNameTaken,
                BTreeMap::from([("name".to_owned(), name)]),
                false,
            ),
            GenreError::InUse { count } => IpcError::new(
                "genre.in_use",
                MessageKey::GenreInUse,
                BTreeMap::from([("count".to_owned(), count.to_string())]),
                false,
            ),
            GenreError::NotFound => IpcError::new(
                "genre.not_found",
                MessageKey::GenreNotFound,
                BTreeMap::new(),
                false,
            ),
        }
    }
}

fn store_is_missing() -> IpcError {
    StoreError::OpenFailed {
        store: StoreKind::Global,
        detail: "the global store was never managed; see lib.rs::open_global_store".to_owned(),
    }
    .into()
}

pub fn genre_list(global: Option<&Store>) -> Result<Vec<GenreWire>, IpcError> {
    let store = global.ok_or_else(store_is_missing)?;
    let rows = genre::list(store)?;
    Ok(rows.into_iter().map(GenreWire::from).collect())
}

pub fn genre_create(global: Option<&Store>, name: &str) -> Result<GenreWire, IpcError> {
    let store = global.ok_or_else(store_is_missing)?;
    Ok(genre::create(store, name)?.into())
}

pub fn genre_rename(global: Option<&Store>, id: &str, new_name: &str) -> Result<(), IpcError> {
    let store = global.ok_or_else(store_is_missing)?;
    Ok(genre::rename(store, id, new_name)?)
}

/// Refuses while any indexed Work carries the genre; the count travels in `params.count`.
pub fn genre_delete(
    global: Option<&Store>,
    indexer: Option<&Indexer>,
    id: &str,
) -> Result<(), IpcError> {
    let store = global.ok_or_else(store_is_missing)?;
    let indexer = indexer.ok_or_else(indexer_is_missing)?;
    let in_use = indexer.count_works_with_genre(id)?;
    Ok(genre::delete(store, id, in_use)?)
}

pub mod wire {
    use crate::core::i18n::IpcError;
    use crate::core::library::indexer::Indexer;
    use crate::core::store::Store;

    use super::GenreWire;

    #[tauri::command]
    pub fn genre_list(app: tauri::AppHandle) -> Result<Vec<GenreWire>, IpcError> {
        use tauri::Manager as _;
        let global = app.try_state::<Store>();
        super::genre_list(global.as_deref())
    }

    #[tauri::command]
    pub fn genre_create(app: tauri::AppHandle, name: String) -> Result<GenreWire, IpcError> {
        use tauri::Manager as _;
        let global = app.try_state::<Store>();
        super::genre_create(global.as_deref(), &name)
    }

    #[tauri::command]
    pub fn genre_rename(
        app: tauri::AppHandle,
        id: String,
        new_name: String,
    ) -> Result<(), IpcError> {
        use tauri::Manager as _;
        let global = app.try_state::<Store>();
        super::genre_rename(global.as_deref(), &id, &new_name)
    }

    #[tauri::command]
    pub fn genre_delete(app: tauri::AppHandle, id: String) -> Result<(), IpcError> {
        use tauri::Manager as _;
        let global = app.try_state::<Store>();
        let indexer = app.try_state::<Indexer>();
        super::genre_delete(global.as_deref(), indexer.as_deref(), &id)
    }
}
