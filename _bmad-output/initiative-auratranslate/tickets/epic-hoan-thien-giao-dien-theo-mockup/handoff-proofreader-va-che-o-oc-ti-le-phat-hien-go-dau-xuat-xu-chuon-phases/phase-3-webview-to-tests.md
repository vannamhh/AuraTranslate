# Phase 3 (Webview) -> Phase 4 (Tests that move) handoff

Webview done and uncommitted. `vue-tsc`, eslint on touched files, `check:commands`, `check:i18n`, `check:tokens`, `check:panel-refs`, `check:lint` are green. No test file was edited. A throwaway smoke test (ratio none/counts, origin label) passed and was deleted.

## New / changed modules

- `src/config/proofread.ts`: `ProofreadWriteResult` gains `decisionId` (accept only; `null` on revert). `revertProofreadSuggestion` args take `decisionId`, sent as `decisionId`. New `skipProofreadDecision(kind)` -> `{ ok, error }`, `getProofreadDecisionCounts()` -> `{ counts | null, error }`.
- `src/proofreadDecisionState.ts` (new, own `resetProofreadDecisions`, called from `resetProofread`): `proofreadDecisionRatio` is `null` (unread) | `{kind:'none'}` | `{kind:'counts', accepted, notError, skipped, *Percent}` (Math.round). `reloadProofreadDecisions()`, `ensureProofreadDecisionsLoaded()` (no-op while loading, loaded, or after a failed read), `proofreadDecisionError`.
- `src/ProofreadDecisionRatio.vue` (new; prop `active`): bar + legend, `data-proofread-ratio="none|counts|error"`. Mounted in `ProofreaderStrip.vue` (`:active="isVisible"`) and `ProofreadMemoryOverlay.vue` (overlay open also calls `reloadProofreadDecisions()`).
- `src/proofreadState.ts`: `RevertItem.decisionId`; accept/revert/ignore/dismiss each call `reloadProofreadDecisions()` after Rust succeeded. `dismissProofreadFinding` is now ASYNC and calls Rust (`ai_proofread_decision_skip`), guarded by `acting`; handler is `void dismissProofreadFinding()`.
- `src/config/reading.ts`: `ReadingChapter.translation_origin_counts` is validated (a run without it is rejected whole, so every reading fixture must carry it). New `unmarkReadingSegment`, `clearReadingMarks`.
- `src/modes/readingOrigin.ts` (new): `readingOriginLabel(counts)` -> `self|other|bilingual_import|mixed|none`.
- `src/modes/readingState.ts`: `readingMarksTotal` (null until read), `readingMarksClearPending`, `readingChaptersDone` (null until `list_chapters` answered), `readingCurrentChapter`, `refreshReadingMarkTotal`, `setReadingMarksCursor`, `unmarkCurrentReadingMark`, `clearAllReadingMarks` (first call arms, second calls Rust), `cancelReadingMarksClear`, `walkReadingMarks`, `openReadingWorkStart`. `ensureReadingLoaded` now ALSO fires `list_reading_marks` and `list_chapters` after the run loads, so tests asserting "last invoke call" or exact call lists on load will move.
- Commands (labelKey + CommandDeps + main.ts wiring): `reading.marks_unmark`, `reading.marks_clear`, `reading.marks_clear_cancel`, `reading.marks_walk`, `reading.work_start`.
- `ReadingMode.vue`: toolbar (Cỡ chữ group, chips from `shortcutRows`, theme label ◐ Sáng / ◑ Tối, marks button `⚑ n chỗ đã đánh dấu`), progress line (`data-reading-progress`, hidden until chapters read), chapter origin line (`data-reading-chapter-origin`), `.mark-actions` wrapper with two buttons (Đánh dấu/Đã đánh dấu + `↵ sửa`), frontier gains `data-reading-frontier-marks` (hidden at 0/unknown) and `data-reading-frontier-work-start`, own marks overlay (`.marks-overlay`; rows with `Ch n`, `Sửa →`, `Bỏ dấu`; `Sửa lần lượt cả n chỗ`; `data-reading-marks-clear`; keys on the panel itself: Arrow up/down, Enter, Backspace; Esc disarms the clear first, second Esc closes). The old Mục trước / Mục kế tiếp / Mở trong Workspace buttons and their three `mode.reading.marks_*` keys are removed (commands remain).
- Gates: `VUE_FLOOR` 44 -> 48 in `scripts/check-commands.mjs` and `scripts/check-i18n.mjs`, `FILE_FLOOR` 159 -> 170 in `scripts/check-tokens.mjs` (the new files tipped them past 80 %). `HANDLER_TABLE` gained `ReadingMode.vue::onMarksEscape` / `onMarksKeydown`. `check-i18n` `VOICE_EXCEPTIONS` gained `mode.reading.origin_self` (see risks).
- vi.json: new `proofread.ratio.*`, `mode.reading.*` (origin_*, progress, marks_*, theme_light/dark, level_group, frontier_marks/work_start), `command.reading.marks_*`/`work_start`; `mode.reading.theme` removed; `marks_button` value changed to `⚑ Chỗ đã đánh dấu`; `marks_heading` is `Chỗ đã đánh dấu khi đọc`.

## Tests known to break (measured: 20 proofread/reading vitest files, 39 failing cases)

- Every reading fixture lacks `translation_origin_counts` -> run rejected -> page empty (readingState, readingFrontier*, readingMarks, readingModeImages, readingUnconfirmed, readingTypography...). Add `translation_origin_counts: { self_count, other_count, bilingual_import_count }` to the shared fixture.
- `readingMarks.test.ts`: extra `list_reading_marks` / `list_chapters` invokes at load; the Tab-trap overlay case expects the old buttons.
- `proofreadActions` ("dismiss writes nothing...") and `proofreadMemory` ("Bỏ qua of 9.4 stays memory only: no ai_proofread_ignore_add"): dismiss now awaits and calls `ai_proofread_decision_skip`; make them async and assert the new call.
- `proofreaderStripSlots.test.ts:131` calls `dismissProofreadFinding()` synchronously.
- Tests that mock `config/proofread` must return `decisionId` for accept and expect `decisionId` in the revert args.

## New cases to write (plan Phase 4)

- I/O matrix ratio rows (1·0·1, reopen, revert 0·0·1, none, read error -> error line and no number, unread -> nothing shown).
- `readingOriginLabel`: 7 non-empty combinations + `none`.
- Clear confirm: first click arms (text `Bỏ hết n dấu? Xác nhận`), Esc via `reading.marks_clear_cancel` disarms with no `clear_reading_marks` call, second click calls Rust once; unmark removes row and decrements `readingMarksTotal`.
- Counter-checks by real removal: drop `skipProofreadDecision` call in dismiss; drop the `reloadProofreadDecisions()` after a decision; drop the Rust call in `clearAllReadingMarks`.
- e2e `story-12-15-...` and the `Chủ: Epic 12` debt item still to do.

## Risks / decisions to surface

- "bản dịch của bạn" (Ice's exact label) violates `check:i18n` Kiểm D (UX-DR47 bans "bạn"). I kept Ice's wording via a named `VOICE_EXCEPTIONS` entry; the alternative is an impersonal wording. Ice should pick. The marks heading I reworded to be compliant.
- "Sửa lần lượt" only opens the first mark in the Workspace (no persisted walk position; opening a mark resets the reading state). True step-through needs a Workspace-side "next mark" affordance, not in plan scope; consider a debt item.
- "Về đầu Tác phẩm" opens the first Chapter from `list_chapters` and reloads the run; the "↵" hint on "Dịch tiếp" was not added (no bound chord would back it).
- Percentages are independent `Math.round`, so they can sum to 99 or 101.

## Phase 3b (after Ice's two Decisions; uncommitted)

Gates green after the change: `vue-tsc`, eslint on touched files, `check:commands`, `check:i18n`, `check:tokens`, `check:lint`. No test file edited and no behavioural test run (type/gate checks only).

### What changed
- Self label: `mode.reading.origin_self` is now "bản dịch của tôi"; `VOICE_EXCEPTIONS` in `scripts/check-i18n.mjs` is back to `[]` (Kiểm D clean, no exception). The Phase 3 "Risks" bullet about "bạn" is closed.
- Real step-through, state in `src/modes/readingState.ts`:
  - `walkReadingMarks()` snapshots `readingMarks`, opens mark 0 in the Workspace (shared helper `openWalkMark`), and only after the open succeeded stores the walk `{ marks, index }` (`marksWalk`, module-level, deliberately NOT cleared by `resetReadingMarks`, which `openCurrentReadingMark`-style opens trigger). A failed open arms nothing.
  - `walkNextReadingMark()` opens `marks[index+1]`; a failed open keeps the position; after opening the last mark the walk is cleared.
  - `readingMarksWalkProgress` (computed): `{ next, total }` with `next = index+1` (count already opened), `null` when no walk or nothing left to step to (so a 1-mark walk never shows the control, and it disappears on reaching the last mark).
  - `resetReading()` without `preserveAnchor` (Work change / chapter delete / import finish) clears the walk; `preserveAnchor: true` (the walk's own opens) keeps it.
- Control: `StatusBar.vue` button `data-reading-walk-next`, shown when `currentMode === 'workspace' && readingMarksWalkProgress !== null`, text `status.reading_walk_next` = "Chỗ kế tiếp ({next}/{total})", `dispatch('reading.marks_walk_next')`. New command `reading.marks_walk_next` (`command.reading.marks_walk_next`, dep `walkNextReadingMark`, wired in `main.ts`).
- Gate: `DISPATCH_FLOOR` 299 -> 322 in `scripts/check-commands.mjs` (gate message demanded it).

### Tests that may break
- Anything asserting the `VOICE_EXCEPTIONS` list or the old "bản dịch của bạn" text; `readingOrigin` label tests should expect "bản dịch của tôi" via `t()`.
- `commandRegistry`/command-count style tests (one more command) and any "marks_walk only opens first" assertion in `readingMarks.test.ts`.
- `StatusBar` tests: one more optional button; none render it unless a walk is armed.

### New cases and counter-checks needed
- walk: start with 3 marks -> opens mark 0, `readingMarksWalkProgress` = {1,3}; `walkNextReadingMark` -> opens mark 1 ({2,3}); next -> opens mark 2 and progress `null`; extra call is a no-op. 1 mark -> progress `null`.
- failed `openChapterById` (false): walk not armed on start; position unchanged on next.
- `resetReading()` clears the walk; `resetReading({ preserveAnchor: true })` keeps it.
- StatusBar: button visible only in workspace mode with progress; click dispatches `reading.marks_walk_next`; text "Chỗ kế tiếp (1/3)".
- Counter-checks by real removal: drop the `marksWalk.value = {...}` set in `walkReadingMarks`; drop the `marksWalk.value = null` in `resetReading`; remove the `currentMode === 'workspace'` guard (hides in reading mode test must go red).
- e2e should step Sửa lần lượt -> Workspace button -> last mark -> button gone.

### Risks
- The walk uses a snapshot of the list taken at start; a mark removed elsewhere meanwhile is still visited.
- The control lives in the shell-wide status bar (gated by mode), not inside `WorkspaceMode.vue`.
