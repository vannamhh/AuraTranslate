---
title: 'Story 11.8, lot B — webview, e2e and CI faults between Epic 11 stories, and the run that closes the story'
type: evidence
created: '2026-10-01'
status: done
route: 'dispatch'
baseline_revision: '03d9692012366d92e1801a14dba6f443bfab2380'
review_loop_iteration: 0
context:
  - '{project-root}/src/AGENTS.md'
  - '{project-root}/e2e/AGENTS.md'
  - '{project-root}/_bmad-output/implementation-artifacts/epic-11-context.md'
relates_to: 8
---

<frozen-after-approval reason="human-owned intent — do not modify unless human renegotiates">

## Intent

**Problem:** Lot A (`03d9692`) closed the Rust half. 12 items ending `Chủ: Story 11.8` are left: the 11 of Task 0 §Group B (`11-8-task0-2026-09-30.md`) and the Windows CI red from lot A Disposition 13. The visible ones: Quét lại stays disabled until restart after leaving Library mid-run; Escape on the PROMOTE question wipes source cuts; a refused row with an empty `message_key` shows no text; the nightly e2e has been red since 09-24, and nothing warns on push.

**Approach:** Every item gets one disposition. Every new guard is counter-checked by really removing its seam. The story closes on a `workflow_dispatch` run on its last pushed commit, where `attribution-focus` really measures AC11 of Story 1.19. If that run is red, a reason line with its run id goes here.

## Boundaries & Constraints

**Always:** Ledger items keep their text; only `→` lines are appended. Fixes touch the sites named in Task 0, and no other comments. If F-W-2's new case is green on the pre-fix code, the item becomes `KHÔNG LÀM (Story 11.8) — không tái lập` and the guard stays. `ci-previous-verdict.mjs` never exits non-zero and never blocks a push.

**Never:** Rust changes. A new gate. `continue-on-error` or a retry in e2e. A mass comment cleanup. `?? []` on `loaded.assets` in production. Closing the Ice item `.hv-unit` doc-comment (`deferred-work.md` ~L12552) from F-D-1 (c).

## Dispositions

Agent, 2026-10-01 (Ice may override any line at approval):
1. F-W-1 fix: `loadLibraryOrphans` stops bumping the writers' `sequence`. It drops its result when a write started or finished during its read. The new counter is reset in `resetLibraryRescan`, and the comments at `libraryRescan.ts` :66 and :218-221 state the real contract.
2. F-W-2 fix: `refreshChapterAssetsAfterRegroup` reads `sequence` without bumping it, and drops a result whose `chapter_id` is not the open one. Measure the guard before the fix (Always).
3. F-W-3 fix: `clearSourceCuts` also refuses while `editorPendingPromote` is set. Positive twin: after cancel, it clears.
4. F-W-6 fix: the `GridPanel.vue` row label renders through `tError()`, covering confirm, restore and flush.
5. F-W-7 fix, both halves: ord is cleared after a successful save (a failed save keeps it), and an out-of-range typed value is visibly reset. The DOM half is mounted first; if Vue already patches the value, that half is `KHÔNG LÀM` with the measurement.
6. F-W-8 test-only: `readFixture` and the 8 inline mocks are typed as the adapter's return type, and the 4 no-wrapper mocks get `{ loaded, error }`. `vue-tsc -p tsconfig.json` (it includes `tests/`) is the guard.
7. F-W-9: (a) the two comments at `src/commands/index.ts` :2386 and :3009 name `settings.close`, which still has no chord, with Esc handled by `@keydown.esc` in `SettingsOverlay.vue`. (b) The other 24 stale lines are `KHÔNG LÀM` (no mass cleanup). (c) The comment-resolving gate is reassigned to `Chủ: Ice`. L12692 closes.
8. F-D-1 (c): `KHÔNG LÀM`, with Task 0's reopen condition.
9. F-CI-4 fix: the script also reads the latest completed `schedule` run and warns with its run id. A push verdict of `silent` does not skip that read.
10. Windows red (L12715): measured cause, test-only. `bindingsEpochWiring.test.ts:35` labels files with `relative()`, which gives `config\shortcutsState.ts` on Windows. Normalise it to `/`. Counter-check on the Mac: force backslashes, and both cases go red with the CI diff.
11. F-B-1 layer 2: `attribution-focus.e2e.mjs` :78-91 reads `res.sources` (`SourceAttributions`).

Ice, 2026-10-01:
12. F-W-4: keep blocking. Delete the dead `settingsOverlayIsOpen` term from `clearSourceCuts`, delete case ⑦b, and rewrite `main.ts` :951-966 to the true contract: no chord runs while Settings is open, including the Shortcuts section. Counter-check: put the term back, and nothing goes red.
13. F-B-1 layer 1, option (a): the `ci.yml` job `e2e` logs `system_profiler SPDisplaysDataType`, then sets the display to ≥ 1440×900 with `displayplacer`. The spec keeps measuring the docked Lookup. Before adding it, read the licence in the downloaded source and add an NFR15 row to the spine's Stack table. If the run shows the display cannot be changed, stop and bring the logged resolution to Ice; the drawer route is not taken without Ice.
14. The closing phase pushes to `master` itself. `pre-push` runs the gates, never `--no-verify`. Then it runs `gh workflow run ci.yml --ref master` and reads the run.
15. The spec stays whole at ~2.6k tokens; per-item detail lives in Task 0.

</frozen-after-approval>

## Code Map

- Per item (file:line at `8c801ef`, guard, counter-check, risks): Task 0 §Group B. Since then, only `src/i18n/vi.json` (+1 line) and `scripts/check-debt-owner.mjs` (1 line) changed under `src`, `tests`, `e2e`, `scripts` and `.github`, so the lines still hold.
- Webview: `src/modes/libraryRescan.ts`, `src/panels/editorPanelState.ts` :2632-2652, `src/editorClearSourceCuts.ts`, `src/panels/GridPanel.vue` :1602-1628 and :1921-1928, `src/modes/libraryChapters.ts` :518-554, `src/modes/LibraryMode.vue` :1166-1170, `src/commands/index.ts` :2386 and :3009.
- Tests: `libraryRescan.test.ts`, `editorRegroupAssetRefresh.test.ts`, `editorClearSourceCuts.test.ts` (pattern `aiTranslatePromoteConfirmFlow.test.ts:60-110`), `gridPanelRowErrorPriority.test.ts`, `libraryChapters.test.ts` :736-770, `support/segmentFixture.ts` :143-154, `ciPreviousVerdict.test.ts` (`makeRun` must route on `--event`), and `bindingsEpochWiring.test.ts:35`.
- e2e and CI: `e2e/specs/attribution-focus.e2e.mjs`, `e2e/support/layoutTier.mjs` (resizes with `setWindowSize(10000,10000)`; the tier comes from `[data-layout-tier]`, fed with CSS px from `WorkspaceDock.vue:577`), `src/layout/lookupDrawerState.ts`, and `.github/workflows/ci.yml` job `e2e` (~:760-830). Also `scripts/ci-previous-verdict.mjs`.

## Tasks & Acceptance

**Execution:**
- [x] `src/**` plus each item's test file: Dispositions 1-5, 7a, 12. For each fix, write the guard first, record its pre-fix result (red, or green for F-W-2), then fix and really remove the seam once. Webview phase.
- [x] `tests/frontend/**`, `scripts/ci-previous-verdict.mjs`, `e2e/**`, `.github/workflows/ci.yml`: Dispositions 6, 9, 10, 11, 13. Run one vitest file per change, plus `vue-tsc` for 6. Run the full suite once, because `ci.yml` changes. Tests phase.
- [x] `deferred-work.md`, push, dispatch: one `→` line per item. Push and dispatch per Disposition 14, then read the CI run and the nightly. Closing phase.

**Acceptance Criteria:**
- Given Quét lại, Chọn thư mục or Gỡ mồ côi is running, when the user leaves and returns to Library before it ends, then its report is applied and the button is usable again immediately.
- Given the PROMOTE question is open with source cuts set, when `editor.clear_source_cuts` runs, then the cuts stay.
- Given confirm, restore or flush fails with an empty `message_key`, when the row renders, then its label is the non-empty `tError` fallback.
- Given each new guard, when its seam is really removed, then only that guard's target goes red, for that reason.
- Given the story's last commit is pushed, when a `workflow_dispatch` run finishes, then `check` is green on both OSes and `attribution-focus` ran (not skipped) and passed, or a reason line with the run id is here.
- Given lot B is done, when the 12 items are read, then each ends in one `→` disposition and `npm run check:debt-owner` is green.

## Implementation Notes

Phase working notes: [11-8-lo-b-phases-2026-10-01.md](11-8-lo-b-phases-2026-10-01.md).
- Every webview guard was red on the pre-fix code, F-W-2 included, so F-W-2 is a reproduced bug, not `KHÔNG LÀM`. The F-W-7 DOM half is real: Vue keeps the typed `99` because `null ?? ''` does not change.
- F-W-1: the orphan read compares a separate `writeEpoch`, moved only where a write applies its result and on reset. A write that fails while a read is in flight no longer drops that read (review patch).
- F-W-4 (Ice 12): restoring the deleted term turns nothing red. Nothing drives `boot()`'s `isBlocked` for any overlay; that is now a ledger item, `Chủ: Amelia`.
- Windows red: `relative()` gives `\` file labels; normalised to `/` in the test. Production is unaffected.
- F-W-8: `readFixture` now returns `caret_segment_id: null`, not the first id; the first id moved the caret in `editorConfirmSegment` ②.
- F-CI-4: the nightly read is its own function, so a `silent` push verdict cannot skip it. `failure`, `timed_out` and `startup_failure` warn.
- F-B-1 layer 2 was measured locally on a `--features wdio` build: passing, and the old `Array.isArray` line throws `[BÀN ĐO HỎNG]`. Layer 1 (`displayplacer` 1.4.0, MIT, full sha256 in the Stack row): dispatch `36813979650` measured the `macos-26` runner display at 1024x768 with no 1440x900 mode (1600x900, 1920x1080 and 1600x1200 are listed), so the step sets 1920x1080; `full` needs a work area ≥ 820 tall.
- Closing run: dispatch `36820520664` on `bd73e98` is green (`check` on both OSes, e2e 29/29, `attribution-focus` passing, not skipped). The first dispatch, `36813979650` on `a50c1cb`, was red only at the 1440x900 step; its `check (windows-2025)` was already green. The latest nightly, `36781779309`, is red on `8c801ef`, which predates both lots.

## Spec Change Log

## Review Triage Log

Loop 0 (Blind B1-B10, Edge E1-E11, Verification-gap G1 + V1-V2):
- G1/B3/E8 nothing drives `main.ts` `isBlocked` with Settings open now that ⑦b is gone — medium, defer: no test drives `boot()`'s `isBlocked` for any overlay (pre-existing); Disposition 12 deleted ⑦b by decision.
- B2/E1 a read dropped because a write started and then failed leaves the orphan block unloaded — low, patch: `writeEpoch` moves only when a write applies its result (Task 0's minimal variant), not at start or on failure.
- B1/E7 two overlapping refreshes in one Chapter can resolve out of order — low, rejected: needs two regroups within one IPC round trip plus out-of-order replies; the fix adds a counter and a reset entry.
- B6a/E3 `timed_out`/`startup_failure` nightlies print a dash — low, patch: they warn like `failure`.
- E4 a cancelled latest nightly masks an earlier red — low, rejected: rare, and the fix adds a multi-run scan.
- B6b old red warns until the next green — false: intended (Task 0 §F-CI-4 risk note).
- B6c/V2 duplicate `gh auth status`; `success`/`silent` print and `WORKFLOW`/`BRANCH` args untested — low, rejected: no named harm; shared constants.
- B4a/E5 `brew install displayplacer` is unpinned against the audited 1.4.0 — low, rejected: the audit is of the MIT licence; a version gate would go red on Homebrew bumps, not product faults.
- E6 `displayplacer` may exit 0 without applying — low, rejected: the after-log shows the resolution and `ensureFullLayoutTier` still fails loudly.
- B4b `setWindowSize` is a second option not presented — false: the harness already uses it; Ice chose (a) in Disposition 13.
- B4c the new step separates the 🔴 no-retry comment from `test:e2e` — low, patch: the step moves above that comment block.
- B5 truncated sha256 in the spine row — low, patch: the full hash.
- B7 ord not cleared on Chapter change — false: the `chapter_id` watch resets it (`libraryChapters.ts:173`).
- B8 imperative `input.value` write in `LibraryMode.vue` — low, rejected: no named harm.
- B9a/E9 `confirmErrorParams`/`restoreErrorParams` unused — false: they feed `errorSegmentId`/`restoreErrorSegmentId`.
- B9b changed comment lines carry history and are not in English (also `libraryRescan.ts`, `editorPanelState.ts`) — low, patch: changed comment lines rewritten in English, history dropped.
- B9c `console.warn` spied but not asserted — low, rejected: no named harm.
- B10 four duplicated inline mocks; `setTimeout(0)` waits — low, rejected: Disposition 6 types each mock; no failing order shown.
- E2 reset during a write leaves `rescanBusy` true — false: `resetLibraryRescan` sets it false (`libraryRescan.ts:277`).
- E10 all three sources assert the same fallback — false: each case fails exactly one source.
- E11 dispatch AC unmet — false at review time: the closing phase is unchecked by design.
- B3b ⑩ has no counter-check — false: recorded in the phase notes (term removed ⇒ red).
- V1 `displayplacer` step untestable locally — observation; measured by the closing dispatch.

## Verification

**Commands:**
- `npx vitest run tests/frontend/<file>` for each changed file; `npx vue-tsc --noEmit -p tsconfig.json`; `npm run check:commands`, `npm run check:i18n`, `npm run check:debt-owner` -- green.
- The full suite once (the change to `ci.yml`). `gh run view <dispatch id>` -- green per AC.

## Acceptance criteria from epics.md

Source: `epics.md` §Story 11.8 (v6, nay ở archive-v6).

As a chủ dự án,
I want các lỗi mà hai story Epic 11 cùng tạo ra ở chỗ giao nhau — nơi không phiên nào thấy cả hai phía — được sửa hoặc quyết dứt điểm,
So that Epic 7 dựng trên một nền có nightly e2e xanh và guard đỏ được khi gỡ seam.

Thừa kế AC chung của epic (xem Notes của `epic-tra-no-nen.md`).

**Acceptance Criteria:**

**Given** mọi mục mang `Chủ: Story 11.8` trong `deferred-work.md`
**When** story hoàn tất
**Then** thoả AC chung của Epic 11; AC riêng rút từ các mục lúc `create-story`

**Given** một lượt Quét lại, Chọn thư mục hay Gỡ mồ côi đang chạy
**When** người dùng rời rồi quay lại Library trước khi lượt đó xong
**Then** nút tương ứng dùng lại được ngay, không phải khởi động lại app

**Given** `confirm_segment` nhận một xuất xứ lúc nạp nằm ngoài danh mục FR117
**When** lệnh chạy
**Then** lệnh từ chối và không ghi gì vào `segment`, nên Tác phẩm vẫn mở được ở lần sau

**Given** commit cuối của story
**When** story lên `review`
**Then** một lượt e2e `workflow_dispatch` trên commit đó xanh, trong đó `attribution-focus` thật sự đo AC11 của Story 1.19; nếu đỏ, một dòng lý do nêu run id nằm trong spec
