//! `#[tauri::command]` shell of `promote_ai_translation`, driven through a `MockRuntime` app
//! so argument forwarding and the missing-state path are executed, not text-scanned.

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use auratranslate_lib::commands::project::{OpenWork, OpenWorkState, create_work_from_text};
use auratranslate_lib::commands::segment::{TRANSLATION_ORIGIN_OTHER, read_open_chapter_segments, wire};
use tauri::Manager as _;
use tauri::test::{MockRuntime, mock_builder, mock_context, noop_assets};

static NEXT_DIR: AtomicU64 = AtomicU64::new(0);

fn temp_dir(tag: &str) -> PathBuf {
    let n = NEXT_DIR.fetch_add(1, Ordering::Relaxed);
    let dir = std::env::temp_dir().join(format!("auratranslate-segment-wire-{}-{}-{}", std::process::id(), tag, n));
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

// `_dir` is declared after `app` so the Work's Store closes before the folder is removed.
struct Harness {
    app: tauri::App<MockRuntime>,
    _dir: DirGuard,
}

impl Harness {
    fn with_open_work(tag: &str) -> Self {
        let dir = temp_dir(tag);
        let open = create_work_from_text(&dir, tag, "en", "", "A dragon roared.".to_owned()).expect("tao Tac pham");
        let app = mock_builder().build(mock_context(noop_assets())).expect("dung app MockRuntime");
        app.manage(OpenWorkState::new(Some(open)));
        Self { app, _dir: DirGuard(dir) }
    }

    fn without_state(tag: &str) -> Self {
        let dir = temp_dir(tag);
        let app = mock_builder().build(mock_context(noop_assets())).expect("dung app MockRuntime");
        Self { app, _dir: DirGuard(dir) }
    }

    fn with_open<T>(&self, f: impl FnOnce(&OpenWork) -> T) -> T {
        let state = self.app.state::<OpenWorkState>();
        let guard = state.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
        f(guard.as_ref().expect("Tac pham dang mo"))
    }

    fn segment_id(&self) -> i64 {
        self.with_open(|open| read_open_chapter_segments(Some(open)).expect("nap chuong").segments[0].id)
    }

    fn text_and_origin(&self, id: i64) -> (String, String) {
        self.with_open(|open| {
            open.store
                .read(move |conn| {
                    conn.query_row(
                        "SELECT target_text, translation_origin FROM segment WHERE id = ?1",
                        [id],
                        |r| Ok((r.get(0)?, r.get(1)?)),
                    )
                })
                .expect("doc hang segment")
        })
    }

    fn promote(&self, id: i64, text: &str, force: bool) -> Result<auratranslate_lib::commands::segment::PromoteAiTranslationOutcome, auratranslate_lib::core::i18n::IpcError> {
        wire::promote_ai_translation(self.app.handle().clone(), id, text.to_owned(), force)
    }
}

#[test]
fn the_promote_shell_writes_the_text_and_origin_other_to_disk() {
    let h = Harness::with_open_work("promote-write");
    let id = h.segment_id();

    let outcome = h.promote(id, "Con rong gam.", false).expect("promote qua vo");

    assert!(!outcome.needs_confirmation);
    assert_eq!(h.text_and_origin(id), ("Con rong gam.".to_owned(), TRANSLATION_ORIGIN_OTHER.to_owned()));
}

#[test]
fn the_promote_shell_refuses_a_retired_segment_and_writes_nothing() {
    let h = Harness::with_open_work("promote-retired");
    let id = h.segment_id();
    h.with_open(|open| {
        open.store
            .write(move |tx| tx.execute("UPDATE segment SET retired_at = '2026-09-30T00:00:00.000Z' WHERE id = ?1", [id]))
            .expect("ve huu segment");
    });
    let before = h.text_and_origin(id);

    for force in [false, true] {
        let err = h.promote(id, "Con rong gam.", force).err().expect("phai la Err");
        assert_eq!(err.code(), "segment.retired");
    }

    assert_eq!(h.text_and_origin(id), before);
}

#[test]
fn the_promote_shell_forwards_force_to_the_pure_function() {
    let h = Harness::with_open_work("promote-force");
    let id = h.segment_id();
    h.promote(id, "Ban nhap mot.", false).expect("promote dau");

    let held = h.promote(id, "Ban nhap hai.", false).expect("giu lai la ket qua");
    assert!(held.needs_confirmation, "ban nhap chua ky + force=false phai giu lai");
    assert_eq!(h.text_and_origin(id).0, "Ban nhap mot.");

    let forced = h.promote(id, "Ban nhap hai.", true).expect("promote force");
    assert!(!forced.needs_confirmation);
    assert_eq!(h.text_and_origin(id).0, "Ban nhap hai.", "force=true qua vo phai ghi de");
}

#[test]
fn the_promote_shell_without_an_open_work_state_reports_no_work_open_instead_of_panicking() {
    let h = Harness::without_state("promote-no-state");

    let err = h.promote(1, "Con rong gam.", false).err().expect("phai la Err");

    assert_eq!(err.code(), "work.none_open");
}
