//! `#[tauri::command]` shell of `glossary_pending_candidates`, driven through a `MockRuntime`
//! app so the lock it holds on `OpenWorkState` is measured, not text-scanned.

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};

use auratranslate_lib::commands::glossary::wire;
use auratranslate_lib::commands::project::{OpenWorkState, create_work_from_text};
use auratranslate_lib::core::glossary::insert_import_scan_candidates;
use auratranslate_lib::core::glossary::scan::ScanCandidate;
use tauri::Manager as _;
use tauri::test::{mock_builder, mock_context, noop_assets};

static NEXT_DIR: AtomicU64 = AtomicU64::new(0);

const CHAPTERS: usize = 100;
const TERMS: usize = 200;
const CHARS_PER_CHAPTER: usize = 4_000;

fn temp_dir(tag: &str) -> PathBuf {
    let n = NEXT_DIR.fetch_add(1, Ordering::Relaxed);
    let dir = std::env::temp_dir().join(format!("auratranslate-glossary-wire-{}-{}-{}", std::process::id(), tag, n));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap_or_else(|e| panic!("tao {}: {e}", dir.display()));
    dir
}

struct DirGuard(PathBuf);

impl Drop for DirGuard {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(Path::new(&self.0));
    }
}

fn alphabet() -> Vec<char> {
    (0..60u32).map(|i| char::from_u32(0x4E00 + i * 7).expect("ky tu CJK")).collect()
}

fn seed(open: &auratranslate_lib::commands::project::OpenWork) {
    let alphabet = alphabet();
    let mut state = 0x2545_F491_4F6C_DD1Du64;
    let mut next = move || {
        state ^= state << 13;
        state ^= state >> 7;
        state ^= state << 17;
        state
    };
    let chapters: Vec<String> = (0..CHAPTERS)
        .map(|_| (0..CHARS_PER_CHAPTER).map(|_| alphabet[(next() % alphabet.len() as u64) as usize]).collect())
        .collect();
    open.store
        .write(move |tx| {
            for (i, text) in chapters.iter().enumerate() {
                tx.execute(
                    "INSERT INTO chapter (ord, title, source_text, status, created_at, updated_at) \
                     VALUES (?1, NULL, '', 'not_started', strftime('%Y-%m-%dT%H:%M:%fZ','now'), \
                     strftime('%Y-%m-%dT%H:%M:%fZ','now'))",
                    [i as i64 + 10],
                )?;
                let chapter_id = tx.last_insert_rowid();
                tx.execute(
                    "INSERT INTO segment (chapter_id, ord, source_text, is_paragraph_end, created_at, updated_at) \
                     VALUES (?1, 1, ?2, 1, strftime('%Y-%m-%dT%H:%M:%fZ','now'), strftime('%Y-%m-%dT%H:%M:%fZ','now'))",
                    (chapter_id, text),
                )?;
            }
            Ok(())
        })
        .expect("gieo Chuong");
    let terms: Vec<ScanCandidate> = (0..TERMS)
        .map(|i| ScanCandidate {
            source_term: format!("{}{}", alphabet[i % alphabet.len()], alphabet[(i / alphabet.len() + i * 7 + 1) % alphabet.len()]),
            occurrence_count: 1 + i as i64,
            context_example: String::new(),
        })
        .collect();
    insert_import_scan_candidates(&open.store, &terms).expect("gieo ung vien");
}

#[test]
fn the_pending_candidates_shell_releases_the_open_work_lock_while_it_scans() {
    let dir = temp_dir("pending-lock");
    let _guard = DirGuard(dir.clone());
    let open = create_work_from_text(&dir, "pending-lock", "zh", "", "萧炎在城中。".to_owned()).expect("tao Tac pham");
    seed(&open);
    let app = mock_builder().build(mock_context(noop_assets())).expect("dung app MockRuntime");
    app.manage(OpenWorkState::new(Some(open)));

    let handle = app.handle().clone();
    let scan = std::thread::spawn(move || {
        let started = Instant::now();
        let rows = wire::glossary_pending_candidates(handle).expect("doc hang cho");
        (rows.len(), started.elapsed())
    });

    std::thread::sleep(Duration::from_millis(20));
    let waited = Instant::now();
    drop(app.state::<OpenWorkState>().lock().unwrap_or_else(std::sync::PoisonError::into_inner));
    let wait = waited.elapsed();

    let (rows, scan_time) = scan.join().expect("luong quet");
    assert!(rows > 0, "tien de: bang cho co ung vien");
    assert!(
        scan_time > Duration::from_millis(100),
        "tien de: luot quet phai du lau de do ({scan_time:?})"
    );
    assert!(
        wait < scan_time / 4,
        "OpenWorkState bi giu {wait:?} trong khi luot quet chay {scan_time:?}: vo phai nha khoa truoc khi tinh Han-Viet va so Chuong"
    );
}
