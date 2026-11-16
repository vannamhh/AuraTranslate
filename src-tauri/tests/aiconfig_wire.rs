//! The `ai_config_get` IPC shell against a keychain that does not answer: the open-Work lock
//! must stay free, or every main-thread command needing the Work freezes the app.
//!
//! Own test binary because the keychain store is process-global.

use std::any::Any;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::mpsc::{self, Sender};
use std::sync::{Arc, Condvar, Mutex};
use std::time::Duration;

use auratranslate_lib::commands::aiconfig::wire;
use auratranslate_lib::commands::project::{OpenWork, OpenWorkState};
use auratranslate_lib::core::store::{Store, StoreSpec};
use keyring_core::api::{CredentialApi, CredentialStoreApi};
use keyring_core::{Credential, Entry};
use tauri::Manager as _;
use tauri::test::{mock_builder, mock_context, noop_assets};

static ENTERED: Mutex<Option<Sender<()>>> = Mutex::new(None);
static RELEASED: (Mutex<bool>, Condvar) = (Mutex::new(false), Condvar::new());

fn release_the_keychain() {
    let (lock, cv) = &RELEASED;
    *lock
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner) = true;
    cv.notify_all();
}

/// A keychain entry whose read waits, like one behind an access prompt nobody answers.
#[derive(Debug)]
struct PromptedCredential;

impl CredentialApi for PromptedCredential {
    fn set_secret(&self, _secret: &[u8]) -> keyring_core::Result<()> {
        Ok(())
    }

    fn get_secret(&self) -> keyring_core::Result<Vec<u8>> {
        if let Some(tx) = ENTERED
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .take()
        {
            let _ = tx.send(());
        }
        let (lock, cv) = &RELEASED;
        let mut released = lock
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        while !*released {
            released = cv
                .wait(released)
                .unwrap_or_else(std::sync::PoisonError::into_inner);
        }
        Err(keyring_core::Error::NoEntry)
    }

    fn delete_credential(&self) -> keyring_core::Result<()> {
        Ok(())
    }

    fn get_credential(&self) -> keyring_core::Result<Option<Arc<Credential>>> {
        Ok(None)
    }

    fn get_specifiers(&self) -> Option<(String, String)> {
        None
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

#[derive(Debug)]
struct PromptedStore;

impl CredentialStoreApi for PromptedStore {
    fn vendor(&self) -> String {
        "aiconfig_wire test".to_owned()
    }

    fn id(&self) -> String {
        "prompted".to_owned()
    }

    fn build(
        &self,
        _service: &str,
        _user: &str,
        _modifiers: Option<&HashMap<&str, &str>>,
    ) -> keyring_core::Result<Entry> {
        Ok(Entry::new_with_credential(Arc::new(PromptedCredential)))
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

fn scratch_dir() -> PathBuf {
    static SEQ: AtomicU64 = AtomicU64::new(0);
    let dir = std::env::temp_dir().join(format!(
        "aiconfig-wire-{}-{}",
        std::process::id(),
        SEQ.fetch_add(1, Ordering::Relaxed)
    ));
    std::fs::create_dir_all(&dir).expect("tao thu muc tam");
    dir
}

#[test]
fn the_open_work_lock_stays_free_while_the_keychain_probe_waits() {
    let real_store_status = keyring::Entry::store_status();
    assert!(real_store_status.is_ok(), "{real_store_status:?}");
    keyring_core::set_default_store(Arc::new(PromptedStore));

    let dir = scratch_dir();
    let app = mock_builder()
        .build(mock_context(noop_assets()))
        .expect("app");
    app.manage(Store::open(StoreSpec::global(dir.join("global.db"))).expect("mo global.db"));
    app.manage::<OpenWorkState>(Mutex::new(None::<OpenWork>));

    let (tx, rx) = mpsc::channel();
    *ENTERED
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner) = Some(tx);
    let handle = app.handle().clone();
    let worker = std::thread::spawn(move || wire::ai_config_get(handle).is_ok());

    let probing = rx.recv_timeout(Duration::from_secs(10));
    let lock_free = app.state::<OpenWorkState>().try_lock().is_ok();
    release_the_keychain();
    let answered = worker.join().expect("luong ai_config_get panic");

    assert!(probing.is_ok(), "ai_config_get khong tham do keychain");
    assert!(
        lock_free,
        "ai_config_get giu khoa Tac pham trong luc cho keychain"
    );
    assert!(answered, "ai_config_get loi sau khi keychain tra loi");

    drop(app);
    let _ = std::fs::remove_dir_all(&dir);
}
