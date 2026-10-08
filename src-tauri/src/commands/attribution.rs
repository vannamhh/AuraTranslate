use crate::core::attribution::{resolve_translator_name, write_translator_name};
use crate::core::i18n::IpcError;
use crate::core::scope::ScopeResolver;
use crate::core::store::{Store, StoreError, StoreKind};

pub(crate) fn store_is_missing() -> IpcError {
    StoreError::OpenFailed {
        store: StoreKind::Global,
        detail: "the global store was never managed; see lib.rs::open_global_store".to_owned(),
    }
    .into()
}

pub fn translator_name_get(global: Option<&Store>) -> Result<Option<String>, IpcError> {
    let global = global.ok_or_else(store_is_missing)?;
    resolve_translator_name(&ScopeResolver::global_only(), global).map_err(IpcError::from)
}

pub fn translator_name_save(global: Option<&Store>, name: &str) -> Result<(), IpcError> {
    let global = global.ok_or_else(store_is_missing)?;
    write_translator_name(global, name).map_err(IpcError::from)
}

pub mod wire {
    use crate::core::i18n::IpcError;
    use crate::core::store::Store;

    #[tauri::command]
    pub fn translator_name_get(app: tauri::AppHandle) -> Result<Option<String>, IpcError> {
        use tauri::Manager as _;

        let global = app.try_state::<Store>();
        super::translator_name_get(global.as_deref())
    }

    #[tauri::command]
    pub fn translator_name_save(app: tauri::AppHandle, name: String) -> Result<(), IpcError> {
        use tauri::Manager as _;

        let global = app.try_state::<Store>();
        super::translator_name_save(global.as_deref(), &name)
    }
}
