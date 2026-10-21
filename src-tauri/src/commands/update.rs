//! IPC surface of the update check (FR111). Pure functions first; `wire` holds the thin
//! `#[tauri::command]` shells. The only network exit is `core::update::check`.

use std::collections::BTreeMap;

use serde::Serialize;

use crate::core::i18n::{IpcError, MessageKey};
use crate::core::scope;
use crate::core::store::{Store, StoreError, StoreKind};
use crate::core::update::{self, UpdateError, UpdateOutcome, Version};

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum UpdateCheckWire {
    UpToDate { current: String },
    UpdateAvailable { latest: String },
    NoAppRelease,
}

fn error(code: &'static str, key: MessageKey, retryable: bool) -> IpcError {
    IpcError::new(code, key, BTreeMap::new(), retryable)
}

fn update_error(e: UpdateError) -> IpcError {
    match e {
        UpdateError::Connect => error(
            "update.connect_failed",
            MessageKey::UpdateConnectFailed,
            true,
        ),
        UpdateError::Timeout => error("update.timeout", MessageKey::UpdateTimeout, true),
        UpdateError::RateLimited => {
            error("update.rate_limited", MessageKey::UpdateRateLimited, true)
        }
        UpdateError::BadResponse => {
            error("update.bad_response", MessageKey::UpdateBadResponse, true)
        }
    }
}

/// Blocking; reads the switch before any connection is opened.
pub fn run_update_check(
    store: Option<&Store>,
    base_url: &str,
    current: &str,
) -> Result<UpdateCheckWire, IpcError> {
    let store = store.ok_or_else(|| {
        IpcError::from(StoreError::OpenFailed {
            store: StoreKind::Global,
            detail: "the global store was never managed".to_owned(),
        })
    })?;
    if !scope::load_global_config(store)?.update_check_enabled() {
        return Err(error("update.disabled", MessageKey::UpdateDisabled, false));
    }
    let current_version = Version::parse(current)
        .ok_or_else(|| error("update.bad_response", MessageKey::UpdateBadResponse, false))?;
    Ok(
        match update::check(base_url, current_version).map_err(update_error)? {
            UpdateOutcome::UpToDate => UpdateCheckWire::UpToDate {
                current: current_version.display(),
            },
            UpdateOutcome::Available(latest) => UpdateCheckWire::UpdateAvailable {
                latest: latest.display(),
            },
            UpdateOutcome::NoAppRelease => UpdateCheckWire::NoAppRelease,
        },
    )
}

/// Validates `version` (`X.Y.Z`) and hands the URL built from the parsed triple to `open`.
pub fn open_release_page(
    version: &str,
    open: &dyn Fn(&str) -> Result<(), String>,
) -> Result<(), IpcError> {
    let parsed = Version::parse(version).ok_or_else(|| {
        error(
            "update.invalid_version",
            MessageKey::UpdateInvalidVersion,
            false,
        )
    })?;
    open(&parsed.release_page_url())
        .map_err(|_| error("update.open_failed", MessageKey::UpdateOpenFailed, false))
}

pub mod wire {
    use tauri::Manager as _;
    use tauri_plugin_opener::OpenerExt as _;

    use super::{UpdateCheckWire, open_release_page, run_update_check};
    use crate::core::i18n::{IpcError, MessageKey};
    use crate::core::store::Store;
    use crate::core::update::RELEASES_API_BASE;

    /// Runs only when the user clicks. Refused with `update.disabled` while the switch is off.
    #[tauri::command]
    pub async fn update_check(app: tauri::AppHandle) -> Result<UpdateCheckWire, IpcError> {
        let join = tauri::async_runtime::spawn_blocking(move || {
            let store = app.try_state::<Store>();
            run_update_check(
                store.as_deref(),
                RELEASES_API_BASE,
                env!("CARGO_PKG_VERSION"),
            )
        });
        match join.await {
            Ok(outcome) => outcome,
            Err(_) => Err(IpcError::new(
                "update.bad_response",
                MessageKey::UpdateBadResponse,
                std::collections::BTreeMap::new(),
                true,
            )),
        }
    }

    #[tauri::command]
    pub fn update_open_release_page(
        app: tauri::AppHandle,
        version: String,
    ) -> Result<(), IpcError> {
        let open = |url: &str| {
            app.opener()
                .open_url(url, None::<&str>)
                .map_err(|e| e.to_string())
        };
        open_release_page(&version, &open)
    }
}
