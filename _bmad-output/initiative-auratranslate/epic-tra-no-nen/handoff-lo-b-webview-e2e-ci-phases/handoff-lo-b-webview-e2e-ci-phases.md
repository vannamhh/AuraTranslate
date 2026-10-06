---
type: handoff
title: "Story 11.8 lot B — phase handoff (working notes, not spec)"
status: done
created: 2026-10-01
skill: bmad-build
---

# Story 11.8 lot B — phase handoff (working notes, not spec)

## Webview phase

Scope done: Dispositions 1-5, 7a, 12. Nothing committed. Touched: `src/modes/libraryRescan.ts`, `src/panels/editorPanelState.ts`, `src/editorClearSourceCuts.ts`, `src/panels/GridPanel.vue`, `src/modes/libraryChapters.ts`, `src/modes/LibraryMode.vue`, `src/main.ts` (comment), `src/commands/index.ts` (2 comments), and the five test files below. Each guard was written first, run on the pre-fix code, then the fix, then the seam really removed and restored.

| Item | Guard (file) | Pre-fix | Counter-check (seam removed) |
|---|---|---|---|
| F-W-1 | `libraryRescan.test.ts`, new describe "leaving and returning to Library mid-write" (3 writers x it.each, plus in-flight-read-dropped, plus reset-drops-read) | 3 writer cases red (`libraryRescanBusy` stuck true); other 2 green by design | `loadLibraryOrphans` back to `++sequence`: same 3 red, existing :113 case stays green |
| F-W-2 | `editorRegroupAssetRefresh.test.ts`, new case (merge pending, `resetEditorPanel` + chapter-B load, then merge resolves) | RED (`editorChapterId` null: chapter-B load dropped). Measured, so not `KHÔNG LÀM` | `++sequence` back: red. `chapter_id` check removed: red (A's assets applied) |
| F-W-3 | `editorClearSourceCuts.test.ts` case ⑩ (cuts stay while pending promote, clear after `cancelPendingPromote`) | RED (cuts cleared) | `editorPendingPromote` term removed: ⑩ red |
| F-W-6 | `gridPanelRowErrorPriority.test.ts`, it.each confirm/restore/flush with `message_key: ''` | 3 RED (empty label) | render back to `t(err.message_key, ...)`: 3 red |
| F-W-7 state | `libraryChapters.test.ts` success clears ord, failed save keeps it | success case RED (ord 5) | null-assignment removed: red |
| F-W-7 DOM | `libraryChapters.test.ts` mount, type `99`, `change`, input `.value === ''` | RED ('99' kept): Vue does NOT patch, so the DOM half is real | `input.value = ...` write removed: red |
| F-W-4 | case ⑦b deleted | n/a | `settingsOverlayIsOpen` term put back in `clearSourceCuts`: all 14 cases green (term was dead) |

Notes for next phase:
- F-W-1 design: `sequence` now bumped only by the three writers and `resetLibraryRescan`; new module cell `writeEpoch` bumped only where a write applies its result (rescan, choose-root, forget) and in reset (review patch: a failed write no longer drops an overlapping read). `loadLibraryOrphans` compares `writeEpoch`. `check:panel-refs` green.
- F-W-6: `rowErrorLabelById` map value is now the source `IpcError`; render is `tError(...)`. Test fakes in `gridPanelRowErrorPriority.test.ts` gained an `emptyMessageKey` switch; `saveRecordingEmptyKey` wraps `recordSave`.
- F-W-4: `main.ts` comment rewritten (no chord runs while Settings is open, incl. Shortcuts). `settings` import/return removed from `editorClearSourceCuts.test.ts`.
- F-W-9 (a): both comments in `commands/index.ts` now name `settings.close` (verified: `keys: undefined`, Esc via `@keydown.esc` in `SettingsOverlay.vue:234`). `check:commands` green.
- F-W-8 is yours: `readFixture` in `support/segmentFixture.ts` still lacks `assets`/`assets_dir`; my new F-W-2 case builds its own `loaded` objects, so it does not depend on it. The new `editorClearSourceCuts` mock of `promoteAiTranslation` is local.
- Verified: the 8 touched/neighbour vitest files (134 tests) green, `vue-tsc -p tsconfig.json` clean, eslint on touched files clean, `check:commands`/`i18n`/`panel-refs`/`lint`/`tokens` green. Full suite NOT run (left to Tests phase, since `ci.yml` changes there).
- Hazard noted, not widened: `ensureSegmentsLoaded` never resets `requested` after a dropped load.
- Ledger lines for the closing phase: F-W-1, F-W-2 (fixed, measured red pre-fix), F-W-3, F-W-6, F-W-7 (both halves real), F-W-4 (Ice option 1, term dead per counter-check), F-W-9 (a) fixed.

## Tests phase

Scope done: Dispositions 6, 9, 10, 11, 13. Nothing committed, pushed or dispatched.

| Item | Change | Counter-check (seam really removed) |
|---|---|---|
| F-W-8 (D6) | `readFixture` and 4 mocks typed `ReadChapterSegmentsResult`; the 4 no-wrapper mocks now `{ loaded, error }` (`chapter_id: 1`, empty segments); 8 inline mocks typed or switched to `readFixture` | `assets_dir` deleted from `readFixture`: `vue-tsc` red on `segmentFixture.ts`. `assets` deleted from `chapterPosition` `docSegmentGia`: red on that file |
| F-CI-4 (D9) | `ciNightlyVerdict` + `printNightlyVerdict` in `ci-previous-verdict.mjs` (separate from `ciPreviousVerdict`, so a push `silent` cannot skip it); `makeRun` routes on `--event`; 7 new cases | `'schedule'` -> `'push'` in the call: 4 red. `case 'failure'` of the nightly print renamed: 1 red (print case). `main()` wiring is not unit-reachable; checked by running the script: it printed the nightly warning with real run 36781779309, exit 0 |
| Windows red (D10) | `bindingsEpochWiring.test.ts` label `.split(sep).join('/')` | forced `'\\'` joins: both cases red with `config\\shortcutsState.ts` |
| F-B-1 layer 2 (D11) | `attribution-focus.e2e.mjs` reads `res.sources`; message reworded | local `--features wdio` run of that spec: green with the fix (`1 passing`, ran, not skipped). Old `Array.isArray(res)` restored: red with `[BÀN ĐO HỎNG]`. Fix restored |
| F-B-1 layer 1 (D13) | `ci.yml` job `e2e`: new step before `npm run test:e2e` logs `system_profiler SPDisplaysDataType`, `brew install displayplacer`, `displayplacer list`, sets `res:1440x900` on the first Persistent screen id, logs again. Spine Stack row added (`displayplacer` 1.4.0, MIT; read `LICENSE` of the downloaded `v1.4.0.tar.gz`, sha256 matches the Homebrew formula) | none possible locally; the dispatch run is the evidence. If the step fails or the tier stays `narrow`, stop and bring the logged resolution to Ice (no drawer route) |

Notes for the closing phase:
- `readFixture` first set `caret_segment_id` to the first segment id and that turned `editorConfirmSegment.test.ts` case ② red (caret placement); it is now `null`, matching the old absent field.
- `check:gates` stays green (no new gate). The nightly script prints a second line `ci (đêm)`; `.githooks/pre-push:65` is unchanged.
- Side effect to know: I ran `brew cat`, which switched Homebrew developer mode on; I ran `brew developer off` right after.
- The e2e counter-check left a `--features wdio` binary in `src-tauri/target/debug`; the next `cargo test` rebuilt it, so `pre-push` is unaffected.
- Full run (once, `ci.yml` changed): 12 gates green; `npm run test` 121 files / 1660 tests green; `npm run build` green; `cargo test --locked` 73 suites / 1952 passed / 0 failed (the first attempt hit my 1500 s perl cap on the slow boundary suites, exit 142, then a rerun with a 3300 s cap went green; no Rust code was touched).
- Unverified until dispatch: the `displayplacer` step on a real runner (brew install time, whether a headless macos-26 display accepts 1440x900, and whether the CSS px reach full tier 1100x700).
