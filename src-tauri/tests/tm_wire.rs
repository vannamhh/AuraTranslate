//! `#[tauri::command]` shells of the TMX import/export, driven through a `MockRuntime` app so the
//! `OpenWorkState` lock they hold is measured, not text-scanned.

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::mpsc;
use std::time::{Duration, Instant};

use auratranslate_lib::commands::project::{OpenWorkState, create_work_from_text};
use auratranslate_lib::commands::tm::{PendingTmxImportState, wire};
use auratranslate_lib::core::store::{Store, StoreSpec};
use tauri::Manager as _;
use tauri::test::{MockRuntime, mock_builder, mock_context, noop_assets};

static NEXT_DIR: AtomicU64 = AtomicU64::new(0);

const SEEDED_PAIRS: usize = 2_000;
const SEEDED_TEXT_CHARS: usize = 3_000;
const PREVIEW_SEEDED_PAIRS: usize = 10_000;
const FILE_PAIRS: usize = 40_000;

fn temp_dir(tag: &str) -> PathBuf {
    let n = NEXT_DIR.fetch_add(1, Ordering::Relaxed);
    let dir = std::env::temp_dir().join(format!("auratranslate-tm-wire-{}-{}-{}", std::process::id(), tag, n));
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

fn seed_tm(store: &Store, count: usize, filler_chars: usize) {
    store
        .write(move |tx| {
            let mut stmt = tx.prepare(
                "INSERT INTO tm_unit (source_text, target_text, translation_origin, created_at) \
                 VALUES (?1, ?2, 'self', '2026-03-01T10:20:30.123Z')",
            )?;
            let filler = "源&<>文".repeat(filler_chars / 5);
            for i in 0..count {
                stmt.execute((format!("{i}{filler}"), format!("{filler}{i}")))?;
            }
            Ok(())
        })
        .expect("gieo tm_unit");
}

fn tmx_file(path: &Path, pairs: usize) {
    let mut body = String::new();
    for i in 0..pairs {
        body.push_str(&format!(
            "<tu><tuv xml:lang=\"zh\"><seg>新{i}</seg></tuv><tuv xml:lang=\"vi\"><seg>moi {i}</seg></tuv></tu>"
        ));
    }
    fs::write(
        path,
        format!("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<tmx version=\"1.4\"><header srclang=\"zh\"/><body>{body}</body></tmx>"),
    )
    .expect("ghi tep tmx");
}

fn app_with_work(dir: &Path, tag: &str) -> tauri::App<MockRuntime> {
    let global = Store::open(StoreSpec::global(dir.join("global.db"))).expect("mo global.db");
    let open = create_work_from_text(dir, tag, "zh", "", "一。".to_owned()).expect("tao Tac pham");
    let app = mock_builder().build(mock_context(noop_assets())).expect("dung app MockRuntime");
    app.manage(global);
    app.manage(OpenWorkState::new(Some(open)));
    app.manage(PendingTmxImportState::new(None));
    app
}

/// Runs `shell` on a thread while this thread holds `OpenWorkState`; a regression that re-locks it
/// fails on the timeout instead of hanging.
fn finishes_while_open_work_is_held<T: Send + 'static>(
    app: &tauri::App<MockRuntime>,
    shell: impl FnOnce(tauri::AppHandle<MockRuntime>) -> T + Send + 'static,
) -> T {
    let handle = app.handle().clone();
    let state = app.state::<OpenWorkState>();
    let held = state.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
    let (tx, rx) = mpsc::channel();
    std::thread::spawn(move || {
        let _ = tx.send(shell(handle));
    });
    let out = rx.recv_timeout(Duration::from_secs(5));
    drop(held);
    out.expect("lenh Global phai xong khi OpenWorkState dang bi giu o noi khac")
}

/// Polls `OpenWorkState` with `try_lock` from the start of `shell` and returns the summed time it
/// was seen taken (`None` if never seen taken) against the shell's whole run.
fn lock_window_against_run<T: Send + 'static>(
    app: &tauri::App<MockRuntime>,
    shell: impl FnOnce(tauri::AppHandle<MockRuntime>) -> T + Send + 'static,
) -> (T, Option<Duration>, Duration) {
    let handle = app.handle().clone();
    let started = Instant::now();
    let run = std::thread::spawn(move || {
        let out = shell(handle);
        (out, started.elapsed())
    });
    let state = app.state::<OpenWorkState>();
    let mut taken_since: Option<Instant> = None;
    let mut held = Duration::ZERO;
    let mut seen_taken = false;
    while !run.is_finished() {
        match state.try_lock() {
            Ok(_) | Err(std::sync::TryLockError::Poisoned(_)) => {
                if let Some(since) = taken_since.take() {
                    held += since.elapsed();
                }
            }
            Err(std::sync::TryLockError::WouldBlock) => {
                seen_taken = true;
                taken_since.get_or_insert_with(Instant::now);
            }
        }
        std::thread::sleep(Duration::from_micros(500));
    }
    let (out, run_time) = run.join().expect("luong lenh");
    if let Some(since) = taken_since {
        held += since.elapsed();
    }
    (out, seen_taken.then_some(held), run_time)
}

/// How long `OpenWorkState` stays taken after `shell` started, against how long the shell ran in
/// all. Polls with `try_lock` from 5 ms on; the first poll must find it taken, or the shell did
/// not lock it early enough to measure.
fn lock_held_against_run<T: Send + 'static>(
    app: &tauri::App<MockRuntime>,
    shell: impl FnOnce(tauri::AppHandle<MockRuntime>) -> T + Send + 'static,
) -> (T, Duration, Duration) {
    let handle = app.handle().clone();
    let started = Instant::now();
    let run = std::thread::spawn(move || {
        let out = shell(handle);
        (out, started.elapsed())
    });
    std::thread::sleep(Duration::from_millis(5));
    let state = app.state::<OpenWorkState>();
    let mut first = true;
    let held = loop {
        match state.try_lock() {
            Ok(_) | Err(std::sync::TryLockError::Poisoned(_)) => break started.elapsed(),
            Err(std::sync::TryLockError::WouldBlock) => {
                first = false;
                std::thread::sleep(Duration::from_millis(1));
            }
        }
        if run.is_finished() {
            break started.elapsed();
        }
    };
    let (out, run_time) = run.join().expect("luong lenh");
    assert!(!first, "tien de: sau 5 ms OpenWorkState phai dang bi giu de do");
    (out, held, run_time)
}

#[test]
fn a_global_confirm_preview_and_export_finish_while_open_work_state_is_held_elsewhere() {
    let dir = temp_dir("global-free");
    let _guard = DirGuard(dir.clone());
    let app = app_with_work(&dir, "global-free");
    let file = dir.join("in.tmx");
    tmx_file(&file, 3);

    let preview_path = file.clone();
    let preview = finishes_while_open_work_is_held(&app, move |h| wire::open_import_preview_from(&h, "global", &preview_path));
    assert_eq!(preview.expect("xem truoc Global").new_count, 3);

    let summary = finishes_while_open_work_is_held(&app, |h| wire::tm_confirm_import(h, false));
    assert_eq!(summary.expect("ghi Global").inserted, 3);

    let out = dir.join("out.tmx");
    let export_path = out.clone();
    let exported = finishes_while_open_work_is_held(&app, move |h| wire::export_tier_to(&h, "global", &export_path));
    assert!(exported.expect("xuat Global").is_some());
    assert!(fs::read_to_string(&out).expect("doc tep xuat").contains("moi 0"));
}

#[test]
fn a_work_plan_confirmed_through_the_shell_is_written_into_the_open_work() {
    let dir = temp_dir("work-confirm");
    let _guard = DirGuard(dir.clone());
    let app = app_with_work(&dir, "work-confirm");
    let file = dir.join("in.tmx");
    tmx_file(&file, 3);

    let preview = wire::open_import_preview_from(app.handle(), "work", &file).expect("xem truoc Work");
    assert_eq!(preview.new_count, 3);
    let summary = wire::tm_confirm_import(app.handle().clone(), false).expect("ghi Work");

    assert_eq!(summary.inserted, 3);
    let state = app.state::<OpenWorkState>();
    let guard = state.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
    let count: i64 = guard
        .as_ref()
        .expect("Tac pham")
        .store
        .read(|conn| conn.query_row("SELECT COUNT(*) FROM tm_unit", [], |r| r.get(0)))
        .expect("dem tm_unit");
    assert_eq!(count, 3, "lo nhap Work phai nam trong kho cua Tac pham dang mo");
}

#[test]
fn the_work_export_shell_holds_open_work_state_only_while_it_reads_the_store() {
    let dir = temp_dir("work-export");
    let _guard = DirGuard(dir.clone());
    let app = app_with_work(&dir, "work-export");
    seed_tm(&app.state::<OpenWorkState>().lock().unwrap().as_ref().expect("Tac pham").store, SEEDED_PAIRS, SEEDED_TEXT_CHARS);
    let out = dir.join("out.tmx");

    let (result, held, run) = lock_held_against_run(&app, move |h| wire::export_tier_to(&h, "work", &out));

    result.expect("xuat Work");
    assert!(run > Duration::from_millis(150), "tien de: luot xuat phai du lau de do ({run:?})");
    assert!(held < run / 2, "OpenWorkState bi giu {held:?} trong khi luot xuat chay {run:?}: chi duoc giu luc doc kho");
}

#[test]
fn the_work_preview_shell_holds_open_work_state_only_while_it_reads_existing_pairs() {
    let dir = temp_dir("work-preview");
    let _guard = DirGuard(dir.clone());
    let app = app_with_work(&dir, "work-preview");
    seed_tm(&app.state::<OpenWorkState>().lock().unwrap().as_ref().expect("Tac pham").store, PREVIEW_SEEDED_PAIRS, 0);
    let file = dir.join("in.tmx");
    tmx_file(&file, FILE_PAIRS);

    let (result, held, run) = lock_window_against_run(&app, move |h| wire::open_import_preview_from(&h, "work", &file));

    assert_eq!(result.expect("xem truoc Work").new_count, FILE_PAIRS);
    let held = held.expect("tien de: phai thay OpenWorkState bi giu luc doc cap co san");
    assert!(run > Duration::from_millis(150), "tien de: luot xem truoc phai du lau de do ({run:?})");
    assert!(held < run / 8, "OpenWorkState bi giu {held:?} trong khi luot xem truoc chay {run:?}: chi duoc giu luc doc cap co san");
}

#[test]
fn closing_or_switching_work_does_not_wait_for_a_global_confirm_that_is_writing() {
    let dir = temp_dir("clear-during-global");
    let _guard = DirGuard(dir.clone());
    let app = app_with_work(&dir, "clear-during-global");
    let file = dir.join("in.tmx");
    tmx_file(&file, FILE_PAIRS);
    wire::open_import_preview_from(app.handle(), "global", &file).expect("xem truoc Global");

    let handle = app.handle().clone();
    let started = Instant::now();
    let confirm = std::thread::spawn(move || {
        let out = wire::tm_confirm_import(handle, false);
        (out, started.elapsed())
    });
    let pending = app.state::<PendingTmxImportState>();
    let mut busy_since: Option<Instant> = None;
    let busy = loop {
        if confirm.is_finished() {
            break false;
        }
        match pending.try_lock() {
            Err(std::sync::TryLockError::WouldBlock) => {
                if busy_since.get_or_insert_with(Instant::now).elapsed() >= Duration::from_millis(20) {
                    break true;
                }
            }
            _ => busy_since = None,
        }
        std::thread::sleep(Duration::from_micros(200));
    };

    let (tx, rx) = mpsc::channel();
    let handle = app.handle().clone();
    std::thread::spawn(move || {
        let pending = handle.state::<PendingTmxImportState>();
        auratranslate_lib::commands::tm::clear_pending_tmx_import_for_work(&pending);
        let _ = tx.send(());
    });
    let cleared = rx.recv_timeout(Duration::from_secs(2));
    let finished_meanwhile = confirm.is_finished();

    let (summary, run) = confirm.join().expect("luong ghi");
    assert_eq!(summary.expect("ghi Global").inserted, FILE_PAIRS);
    assert!(busy && run > Duration::from_millis(150), "tien de: luot ghi Global phai du lau de do ({run:?})");
    cleared.expect("dong hoac doi Tac pham khong duoc cho luot ghi Global");
    assert!(!finished_meanwhile, "viec xoa phai tra ve khi luot ghi con chay");
}

fn self_labelled_tmx(path: &Path) {
    fs::write(
        path,
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<tmx version=\"1.4\"><header srclang=\"zh\"/><body>\
         <tu><prop type=\"x-aura-origin\">self</prop><tuv xml:lang=\"zh\"><seg>一</seg></tuv><tuv xml:lang=\"vi\"><seg>A</seg></tuv></tu>\
         </body></tmx>",
    )
    .expect("ghi tep tmx");
}

fn origins_in(store: &Store) -> Vec<String> {
    store
        .read(|conn| {
            let mut stmt = conn.prepare("SELECT translation_origin FROM tm_unit")?;
            let rows = stmt.query_map([], |r| r.get::<_, String>(0))?.collect::<Result<Vec<_>, _>>()?;
            Ok(rows)
        })
        .expect("doc tm_unit")
}

#[test]
fn the_shell_forwards_the_ownership_flag_on_a_global_plan() {
    let dir = temp_dir("own-global");
    let _guard = DirGuard(dir.clone());
    let app = app_with_work(&dir, "own-global");
    let file = dir.join("in.tmx");
    self_labelled_tmx(&file);
    wire::open_import_preview_from(app.handle(), "global", &file).expect("xem truoc Global");

    wire::tm_confirm_import(app.handle().clone(), true).expect("ghi Global");

    assert_eq!(origins_in(&app.state::<Store>()), vec!["self"]);
}

#[test]
fn the_shell_forwards_the_ownership_flag_on_a_work_plan() {
    let dir = temp_dir("own-work");
    let _guard = DirGuard(dir.clone());
    let app = app_with_work(&dir, "own-work");
    let file = dir.join("in.tmx");
    self_labelled_tmx(&file);
    wire::open_import_preview_from(app.handle(), "work", &file).expect("xem truoc Work");

    wire::tm_confirm_import(app.handle().clone(), true).expect("ghi Work");

    let state = app.state::<OpenWorkState>();
    let guard = state.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
    assert_eq!(origins_in(&guard.as_ref().expect("Tac pham").store), vec!["self"]);
}
