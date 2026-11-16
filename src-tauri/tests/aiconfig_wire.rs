//! Every IPC shell that reads the API key, against a keychain that does not answer: the
//! open-Work lock must stay free while it waits, or every main-thread command needing the
//! Work freezes the app.
//!
//! Own test binary because the keychain store is process-global.

use std::any::Any;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::mpsc::{self, Sender};
use std::sync::{Arc, Condvar, Mutex, Once};
use std::time::Duration;

use auratranslate_lib::commands::aiconfig::wire as aiconfig_wire;
use auratranslate_lib::commands::aiprompt::{
    AiCallRecord, AiCallRecordsState, LastAssembledPromptState, assemble_and_record_prompt,
    next_call_record_stamp,
};
use auratranslate_lib::commands::aitranslate::{
    AiTranslateBatchEventWire, AiTranslateGeneration, AiTranslateOutcomeWire, AiTranslateScope,
    ConnectionCheckGeneration, ConnectionCheckWire, wire as aitranslate_wire,
};
use auratranslate_lib::commands::project::{OpenWork, OpenWorkState, create_work_from_text};
use auratranslate_lib::commands::promptset::prompt_set_create;
use auratranslate_lib::commands::proofread::{
    AiProofreadEventWire, AiProofreadOutcomeWire, ProofreadGeneration, wire as proofread_wire,
};
use auratranslate_lib::commands::segment::{promote_ai_translation, read_open_chapter_segments};
use auratranslate_lib::core::ai::rag::{
    GlossaryInjectionStatus, InjectionLedger, TmInjectionStatus,
};
use auratranslate_lib::core::aiconfig::{AiConfigField, AiConfigTier, write_field};
use auratranslate_lib::core::promptset::{PromptSetKind, PromptSetTier};
use auratranslate_lib::core::store::{Store, StoreSpec};
use keyring_core::api::{CredentialApi, CredentialStoreApi};
use keyring_core::{Credential, Entry};
use tauri::Manager as _;
use tauri::test::{MockRuntime, mock_builder, mock_context, noop_assets};

static SERIAL: Mutex<()> = Mutex::new(());
static ENTERED: Mutex<Option<Sender<()>>> = Mutex::new(None);
static RELEASED: (Mutex<bool>, Condvar) = (Mutex::new(false), Condvar::new());

fn set_released(value: bool) {
    let (lock, cv) = &RELEASED;
    *lock
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner) = value;
    cv.notify_all();
}

/// A keychain entry whose read waits, like one behind an access prompt nobody answers, then
/// reports no stored key.
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

fn install_prompted_store() {
    static INSTALL: Once = Once::new();
    INSTALL.call_once(|| {
        let real_store_status = keyring::Entry::store_status();
        assert!(real_store_status.is_ok(), "{real_store_status:?}");
        keyring_core::set_default_store(Arc::new(PromptedStore));
    });
}

fn scratch_dir(tag: &str) -> PathBuf {
    static SEQ: AtomicU64 = AtomicU64::new(0);
    let dir = std::env::temp_dir().join(format!(
        "aiconfig-wire-{tag}-{}-{}",
        std::process::id(),
        SEQ.fetch_add(1, Ordering::Relaxed)
    ));
    std::fs::create_dir_all(&dir).expect("tao thu muc tam");
    dir
}

/// An open Work with a configured cloud provider, one translated and one untranslated
/// segment, a recorded single prompt and a recorded batch call: every shell gets as far as
/// the key read.
struct Fixture {
    app: tauri::App<MockRuntime>,
    ids: Vec<i64>,
    dirs: Vec<PathBuf>,
}

impl Fixture {
    fn new(tag: &str) -> Self {
        let global_dir = scratch_dir(&format!("{tag}-global"));
        let work_dir = scratch_dir(&format!("{tag}-work"));
        let global =
            Store::open(StoreSpec::global(global_dir.join("global.db"))).expect("mo global.db");
        write_field(&global, AiConfigField::Endpoint, "http://127.0.0.1:9/v1").expect("endpoint");
        write_field(&global, AiConfigField::Model, "gpt-wire").expect("model");
        let open: OpenWork = create_work_from_text(
            &work_dir,
            tag,
            "en",
            "",
            "A dragon roared. The night fell.".to_owned(),
        )
        .expect("tao Tac pham");
        prompt_set_create(
            Some(&global),
            Some(&open),
            PromptSetTier::Global,
            PromptSetKind::Translate,
            "Plain",
            "{{source_segment}}",
        )
        .expect("tao bo prompt");
        let ids: Vec<i64> = read_open_chapter_segments(Some(&open))
            .expect("nap chuong")
            .segments
            .iter()
            .map(|s| s.id)
            .collect();
        assert!(ids.len() >= 2, "can hai segment, co {}", ids.len());
        promote_ai_translation(Some(&open), ids[0], "Mot con rong gam.", false).expect("ban dich");
        let record = LastAssembledPromptState::new(None);
        assemble_and_record_prompt(Some(&global), Some(&open), &record, Some("Plain"), ids[0])
            .expect("lap rap prompt");
        let calls = AiCallRecordsState::default();
        calls
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .put(recorded_call());

        let app = mock_builder()
            .build(mock_context(noop_assets()))
            .expect("app");
        app.manage(global);
        app.manage(OpenWorkState::new(Some(open)));
        app.manage(record);
        app.manage(calls);
        app.manage(AiTranslateGeneration::default());
        app.manage(ConnectionCheckGeneration::default());
        app.manage(ProofreadGeneration::default());
        Self {
            app,
            ids,
            dirs: vec![global_dir, work_dir],
        }
    }

    fn handle(&self) -> tauri::AppHandle<MockRuntime> {
        self.app.handle().clone()
    }

    /// Runs `call` on its own thread, waits until it is inside the keychain read, and fails
    /// unless the open-Work lock was free at that moment.
    fn lock_free_while_keychain_waits<T: Send>(
        &self,
        shell: &str,
        call: impl FnOnce() -> T + Send,
    ) -> T {
        let _serial = SERIAL
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        install_prompted_store();
        set_released(false);
        let (tx, rx) = mpsc::channel();
        *ENTERED
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = Some(tx);
        std::thread::scope(|scope| {
            let worker = scope.spawn(call);
            let probing = rx.recv_timeout(Duration::from_secs(10));
            let lock_free = self.app.state::<OpenWorkState>().try_lock().is_ok();
            set_released(true);
            let out = worker.join().expect("luong goi lenh panic");
            assert!(probing.is_ok(), "{shell} khong doc keychain");
            assert!(
                lock_free,
                "{shell} giu khoa Tac pham trong luc cho keychain"
            );
            out
        })
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        for dir in &self.dirs {
            let _ = std::fs::remove_dir_all(dir);
        }
    }
}

fn recorded_call() -> AiCallRecord {
    AiCallRecord {
        operation_id: 1,
        call_index: 0,
        stamp: next_call_record_stamp(),
        prompt: "A dragon roared.".to_owned(),
        system: String::new(),
        segments: Vec::new(),
        context_segment_ids: Vec::new(),
        ledger: InjectionLedger {
            glossary: GlossaryInjectionStatus::NotAsked,
            tm: TmInjectionStatus::NotAsked,
            unknown_markers: Vec::new(),
            source_segment_missing: false,
            pieces: Vec::new(),
        },
        chapter_id: 0,
        prompt_set_name: "Plain".to_owned(),
        prompt_set_tier: PromptSetTier::Global,
        model: "gpt-wire".to_owned(),
        sent_at: None,
        sent_usage: None,
    }
}

fn text_channel() -> tauri::ipc::Channel<String> {
    tauri::ipc::Channel::new(|_| Ok(()))
}

fn batch_channel() -> tauri::ipc::Channel<AiTranslateBatchEventWire> {
    tauri::ipc::Channel::new(|_| Ok(()))
}

#[test]
fn ai_config_get_leaves_the_open_work_lock_free_while_the_keychain_waits() {
    let f = Fixture::new("config-get");
    let handle = f.handle();
    let wire = f.lock_free_while_keychain_waits("ai_config_get", move || {
        aiconfig_wire::ai_config_get(handle)
    });
    assert_eq!(wire.expect("ai_config_get").key_configured, Some(false));
}

#[test]
fn ai_translate_segment_leaves_the_open_work_lock_free_while_the_keychain_waits() {
    let f = Fixture::new("segment");
    let (handle, id) = (f.handle(), f.ids[1]);
    let outcome = f.lock_free_while_keychain_waits("ai_translate_segment", move || {
        tauri::async_runtime::block_on(aitranslate_wire::ai_translate_segment(
            handle,
            id,
            Some("Plain".to_owned()),
            text_channel(),
        ))
    });
    assert!(matches!(
        outcome.expect("ai_translate_segment"),
        AiTranslateOutcomeWire::NotConfigured
    ));
}

#[test]
fn ai_translate_resend_leaves_the_open_work_lock_free_while_the_keychain_waits() {
    let f = Fixture::new("resend");
    let handle = f.handle();
    let outcome = f.lock_free_while_keychain_waits("ai_translate_resend", move || {
        tauri::async_runtime::block_on(aitranslate_wire::ai_translate_resend(
            handle,
            text_channel(),
        ))
    });
    assert!(matches!(
        outcome.expect("ai_translate_resend"),
        AiTranslateOutcomeWire::NotConfigured
    ));
}

#[test]
fn ai_translate_batch_leaves_the_open_work_lock_free_while_the_keychain_waits() {
    let f = Fixture::new("batch");
    let handle = f.handle();
    let outcome = f.lock_free_while_keychain_waits("ai_translate_batch", move || {
        tauri::async_runtime::block_on(aitranslate_wire::ai_translate_batch(
            handle,
            AiTranslateScope::Chapter,
            Some("Plain".to_owned()),
            batch_channel(),
        ))
    });
    assert!(matches!(
        outcome.expect("ai_translate_batch"),
        AiTranslateOutcomeWire::NotConfigured
    ));
}

#[test]
fn ai_translate_resend_call_leaves_the_open_work_lock_free_while_the_keychain_waits() {
    let f = Fixture::new("resend-call");
    let handle = f.handle();
    let outcome = f.lock_free_while_keychain_waits("ai_translate_resend_call", move || {
        tauri::async_runtime::block_on(aitranslate_wire::ai_translate_resend_call(
            handle,
            1,
            0,
            batch_channel(),
        ))
    });
    assert!(matches!(
        outcome.expect("ai_translate_resend_call"),
        AiTranslateOutcomeWire::NotConfigured
    ));
}

#[test]
fn ai_config_test_connection_leaves_the_open_work_lock_free_while_the_keychain_waits() {
    let f = Fixture::new("connection");
    let handle = f.handle();
    let outcome = f.lock_free_while_keychain_waits("ai_config_test_connection", move || {
        tauri::async_runtime::block_on(aitranslate_wire::ai_config_test_connection(
            handle,
            AiConfigTier::Work,
        ))
    });
    assert_eq!(
        outcome.expect("ai_config_test_connection"),
        ConnectionCheckWire::NotConfigured
    );
}

#[test]
fn ai_proofread_segments_leaves_the_open_work_lock_free_while_the_keychain_waits() {
    let f = Fixture::new("proofread");
    let (handle, id) = (f.handle(), f.ids[0]);
    let outcome = f.lock_free_while_keychain_waits("ai_proofread_segments", move || {
        tauri::async_runtime::block_on(proofread_wire::ai_proofread_segments(
            handle,
            vec![id],
            tauri::ipc::Channel::<AiProofreadEventWire>::new(|_| Ok(())),
        ))
    });
    assert!(matches!(
        outcome.expect("ai_proofread_segments"),
        AiProofreadOutcomeWire::NotConfigured
    ));
}
