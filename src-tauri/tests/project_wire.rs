//! `#[tauri::command]` shells of `commands::project::wire`, driven through a `MockRuntime`
//! app carrying the states `lib.rs::open_work_slot` manages and a real `library-index.db`.

#[allow(dead_code)] // shared module: not every helper is used in this file
#[path = "fixtures_docx.rs"]
mod fixtures_docx;

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

use auratranslate_lib::commands::project::{
    AppendInProgressState, OpenWork, OpenWorkState, PendingImportSourceState, URL_IMPORT_IMAGE_PROGRESS_EVENT,
    create_work_from_text, stash_pending_import_source_for, wire,
};
use auratranslate_lib::core::library::indexer::Indexer;
use auratranslate_lib::core::scope::save_value;
use auratranslate_lib::core::segment::import::import_file;
use auratranslate_lib::core::segment::pipeline::{ChapterInput, PipelineShape};
use auratranslate_lib::core::store::{Store, StoreSpec, Transaction};
use tauri::test::{MockRuntime, mock_builder, mock_context, noop_assets};
use tauri::{Listener as _, Manager as _};

static NEXT_DIR: AtomicU64 = AtomicU64::new(0);

fn temp_dir(tag: &str) -> PathBuf {
    let n = NEXT_DIR.fetch_add(1, Ordering::Relaxed);
    let dir = std::env::temp_dir().join(format!("auratranslate-project-wire-{}-{}-{}", std::process::id(), tag, n));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap_or_else(|e| panic!("tao {}: {e}", dir.display()));
    dir
}

struct Harness {
    app: tauri::App<MockRuntime>,
    root: PathBuf,
    side: PathBuf,
}

impl Harness {
    fn new(tag: &str) -> Self {
        let root = temp_dir(&format!("{tag}-root"));
        let side = temp_dir(&format!("{tag}-side"));
        let global = Store::open(StoreSpec::global(side.join("global.db"))).expect("mo global.db");
        save_value(&global, "app_config", "library_root", &root.display().to_string()).expect("ghi library_root");
        let indexer = Indexer::open(side.join("library-index.db")).expect("mo library-index.db");

        let app = mock_builder().build(mock_context(noop_assets())).expect("dung app MockRuntime");
        app.manage(global);
        app.manage(indexer);
        app.manage(OpenWorkState::new(None));
        app.manage(AppendInProgressState::default());
        app.manage(PendingImportSourceState::new(None));
        Self { app, root, side }
    }

    fn create_indexed_work(&self, name: &str) -> OpenWork {
        let open = create_work_from_text(&self.root, name, "en", "", format!("Chuong cua {name}. Cau hai.")).expect("tao Tac pham");
        self.reindex();
        open
    }

    fn reindex(&self) {
        let global = self.app.state::<Store>();
        self.app.state::<Indexer>().rebuild(&self.root, Some(&global)).expect("rebuild chi muc");
    }

    fn open(&self, work_id: &str) {
        wire::open_work(self.app.handle().clone(), work_id.to_owned()).expect("open_work phai thanh cong");
    }

    fn indexed_chapter_count(&self, work_id: &str) -> Option<u32> {
        self.app.state::<Indexer>().find_work(work_id).expect("find_work").map(|w| w.chapter_count)
    }

    fn open_work_id(&self) -> Option<String> {
        let state = self.app.state::<OpenWorkState>();
        let guard = state.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
        guard.as_ref().map(|w| w.meta.work_id.clone())
    }

    fn stash_append(&self, shape: PipelineShape, sidecar: Option<auratranslate_lib::core::segment::import::DocxSidecar>, destination: &str) {
        let pending = self.app.state::<PendingImportSourceState>();
        stash_pending_import_source_for(&pending, shape, sidecar, Some(destination.to_owned()));
    }

    fn confirm(&self) -> Result<wire::CreatedWork, auratranslate_lib::core::i18n::IpcError> {
        wire::confirm_import_with_encoding(
            self.app.handle().clone(),
            "unused".to_owned(),
            "en".to_owned(),
            String::new(),
            "UTF-8".to_owned(),
            None,
        )
    }
}

impl Drop for Harness {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
        let _ = fs::remove_dir_all(&self.side);
    }
}

fn add_asset_row(open: &OpenWork, file_name: &str) {
    let chapter_id = open.chapter_id;
    let file_name = file_name.to_owned();
    open.store
        .write(move |tx: &Transaction<'_>| {
            tx.execute(
                "INSERT INTO asset (chapter_id, file_name, source_url, anchor_after_segment_ord, byte_len, content_type, created_at) \
                 VALUES (?1, ?2, NULL, 0, 1, 'image/png', '2026-01-01T00:00:00Z')",
                (chapter_id, file_name),
            )
        })
        .expect("chen hang asset");
}

fn seed_assets(open: &OpenWork, referenced: &str, stray: &str) -> (PathBuf, PathBuf) {
    let dir = open.dir.join("assets");
    fs::create_dir_all(&dir).expect("tao assets/");
    let keep = dir.join(referenced);
    let orphan = dir.join(stray);
    fs::write(&keep, b"k").expect("ghi anh duoc tham chieu");
    fs::write(&orphan, b"s").expect("ghi anh mo coi");
    add_asset_row(open, referenced);
    (keep, orphan)
}

#[test]
fn opening_a_work_through_the_shell_sweeps_only_the_unreferenced_image_files() {
    let h = Harness::new("sweep-at-open");
    let open = h.create_indexed_work("Quet Mo Coi");
    let work_id = open.meta.work_id.clone();
    let (keep, orphan) = seed_assets(&open, "keep.png", "stray.png");
    drop(open);

    h.open(&work_id);

    assert!(keep.exists(), "anh co hang asset phai o lai");
    assert!(!orphan.exists(), "anh mo coi phai bi quet khi mo Tac pham qua vo");
}

#[test]
fn opening_a_work_through_the_shell_skips_the_sweep_while_an_append_into_it_is_registered() {
    let h = Harness::new("sweep-skipped-during-append");
    let open = h.create_indexed_work("Dang Noi Them");
    let work_id = open.meta.work_id.clone();
    let (_keep, orphan) = seed_assets(&open, "keep.png", "stray.png");
    drop(open);
    h.app.state::<AppendInProgressState>().lock().unwrap_or_else(std::sync::PoisonError::into_inner).insert(work_id.clone());

    h.open(&work_id);

    assert!(orphan.exists(), "luot noi them dang do dang: anh chua tham chieu khong duoc bi quet");
}

fn two_indexed_works_with_an_image_each(h: &Harness) -> (String, String, PathBuf, PathBuf) {
    let a = h.create_indexed_work("Pham Vi A");
    let b = h.create_indexed_work("Pham Vi B");
    let (id_a, id_b) = (a.meta.work_id.clone(), b.meta.work_id.clone());
    let (dir_a, dir_b) = (a.dir.join("assets"), b.dir.join("assets"));
    drop((a, b));
    for dir in [&dir_a, &dir_b] {
        fs::create_dir_all(dir).expect("tao assets/");
        fs::write(dir.join("x.png"), b"x").expect("ghi anh");
    }
    (id_a, id_b, dir_a, dir_b)
}

#[test]
fn every_work_opened_in_the_session_keeps_its_assets_in_the_asset_scope() {
    let h = Harness::new("asset-scope");
    let (id_a, id_b, dir_a, dir_b) = two_indexed_works_with_an_image_each(&h);
    let scope = h.app.asset_protocol_scope();
    let allowed = |dir: &Path| scope.is_allowed(dir.join("x.png"));

    h.open(&id_a);
    assert!(allowed(&dir_a), "Tac pham A dang mo: scope phai cap assets/ cua A");
    assert!(!allowed(&dir_b), "B chua mo: scope khong duoc cap assets/ cua B");

    h.open(&id_b);
    assert!(allowed(&dir_b), "chuyen sang B: scope phai cap assets/ cua B");
    assert!(allowed(&dir_a), "chuyen sang B: assets/ cua A khong bi chan, forbid_directory khong co nghich dao");
}

#[test]
fn reopening_a_work_after_leaving_it_grants_its_assets_again() {
    let h = Harness::new("asset-scope-reopen");
    let (id_a, id_b, dir_a, dir_b) = two_indexed_works_with_an_image_each(&h);
    let scope = h.app.asset_protocol_scope();
    let allowed = |dir: &Path| scope.is_allowed(dir.join("x.png"));

    h.open(&id_a);
    h.open(&id_b);
    h.open(&id_a);

    assert!(allowed(&dir_a), "mo lai A: assets/ cua A phai doc duoc");
    assert!(allowed(&dir_b), "mo lai A: assets/ cua B van doc duoc, khong bi chan");
}

#[test]
fn an_append_into_the_open_work_moves_the_library_index_row_of_that_work() {
    let h = Harness::new("append-reindex");
    let open = h.create_indexed_work("Noi Them Chi Muc");
    let work_id = open.meta.work_id.clone();
    drop(open);
    h.open(&work_id);
    assert_eq!(h.indexed_chapter_count(&work_id), Some(1), "tien de: chi muc dang ghi 1 Chuong");

    h.stash_append(
        PipelineShape::Chapters(vec![ChapterInput::AlreadyText("Chuong moi them vao Tac pham dang mo.".to_owned())]),
        None,
        &work_id,
    );
    h.confirm().expect("noi them vao Tac pham dang mo phai thanh cong");

    assert_eq!(h.open_work_id().as_deref(), Some(work_id.as_str()), "Tac pham dang mo khong doi");
    assert_eq!(
        h.indexed_chapter_count(&work_id),
        Some(2),
        "noi them vao Tac pham DANG MO phai dua Chuong moi vao library-index.db qua vo"
    );
}

fn docx_append_shape(dir: &Path) -> (PipelineShape, Option<auratranslate_lib::core::segment::import::DocxSidecar>) {
    let path = dir.join("co_anh.docx");
    fs::write(&path, fixtures_docx::image_png()).expect("ghi .docx fixture");
    import_file(&path).expect("doc .docx co anh")
}

#[test]
fn an_append_into_the_open_work_registers_the_append_guard_while_its_images_are_written() {
    let h = Harness::new("append-guard-open");
    let open = h.create_indexed_work("Giu Khoa Noi Them");
    let work_id = open.meta.work_id.clone();
    drop(open);
    h.open(&work_id);

    let seen = Arc::new(Mutex::new(Vec::<bool>::new()));
    let sink = Arc::clone(&seen);
    let handle = h.app.handle().clone();
    let watched = work_id.clone();
    h.app.listen(URL_IMPORT_IMAGE_PROGRESS_EVENT, move |_| {
        let state = handle.state::<AppendInProgressState>();
        let registered = state.lock().unwrap_or_else(std::sync::PoisonError::into_inner).contains(&watched);
        sink.lock().unwrap_or_else(std::sync::PoisonError::into_inner).push(registered);
    });

    let (shape, sidecar) = docx_append_shape(&h.side);
    h.stash_append(shape, sidecar, &work_id);
    h.confirm().expect("noi them .docx co anh vao Tac pham dang mo");

    let seen = seen.lock().unwrap_or_else(std::sync::PoisonError::into_inner).clone();
    assert!(!seen.is_empty(), "nguon .docx co anh phai phat it nhat mot su kien tien do anh");
    assert!(
        seen.iter().all(|registered| *registered),
        "trong luc anh dang ghi, work_id phai nam trong AppendInProgressState: {seen:?}"
    );
    assert!(
        h.app.state::<AppendInProgressState>().lock().unwrap_or_else(std::sync::PoisonError::into_inner).is_empty(),
        "sau luot goi, guard phai da nha"
    );
}
