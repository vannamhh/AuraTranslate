# Handoff: Rust phase -> Webview phase

## Commands (both `#[tauri::command(async)]`, registered in `lib.rs`)

`convention_scan_segments`
- invoke args: `{ generation: number, segmentIds: number[] }` (Tauri camelCases the top-level args).
- returns the SAME `ConventionScanWire` as `convention_scan_chapter` (`generation`, `chapter_id`, `segments[{segment_id, scanned_text, spans[]}]`, `ignored`, `glossary_suppressed`, `overlapped`, `not_scanned`). Only sentences with a surviving mark appear in `segments`; a scanned sentence with no mark is simply absent (so "absent" = "clear this id's entry", the webview merge must delete the entry of every requested id that is not in `segments`).
- ids unknown / retired / outside the open Chapter are skipped silently; duplicate ids are read once. Omitted sentences add to `not_scanned`; empty target text is not scanned and not counted. No Work open => error `no_work_open` (same as chapter scan).

`convention_scan_suggestions`
- invoke args: `{ generation: number, items: { segment_id: number, text: string }[] }`. ⚠️ the item fields are snake_case (plain serde struct, no rename); only the top-level arg names are camelCased.
- returns `ConventionSuggestionScanWire`: `{ generation, chapter_id: number | null, items: [{segment_id, scanned_text, spans[]}], ignored, glossary_suppressed, overlapped }`. No `not_scanned`.
- `items` holds only suggestions with at least one surviving mark; `scanned_text` is the suggestion text that was sent. Span shape identical to `ConventionSpanWire` (`start`/`end` UTF-16, `rule_key`, `avoid`, `replacement_cased: string | null`, `tier`).
- No Work open => NOT an error: `chapter_id: null`, `items: []`. Blank text is skipped. An item whose `segment_id` is not a live sentence of the open Chapter gets no mark (needs the source from disk for the Glossary check). 0 rules => `items: []`.
- Writes nothing; no write command takes a suggestion span.
- Source sentence is read from disk only for the items that matched; one `GlossarySnapshot` per call.

Wire types are `ConventionSuggestionItem`, `ConventionSuggestionScanWire` in `src-tauri/src/commands/convention.rs`; pure fns `convention_scan_segments(global, open_state, cache, generation, &[i64])` and `convention_scan_suggestions(global, open_state, cache, generation, Vec<item>)`.

## Things the plan did not predict

- `src-tauri/tests/ipc_argument_contract.rs` (`every_registered_commands_business_parameters_match_camel_case_across_the_wire`) is RED until `src/config/convention.ts` has an `invoke('convention_scan_segments', { generation, segmentIds })` and `invoke('convention_scan_suggestions', { generation, items })` literal. It expects the TS keys to equal the Rust params exactly (camelCase). Not a bug, it is the Webview phase's first check. Do NOT add them to `NO_FRONTEND_CALLER`.
- `config_invariants.rs` also needed the global census bump (120/74 -> 120/76) besides the file row (5/4/4) and two `blocking_wire_cases` rows; all green now.
- No new i18n error keys: neither command has a new error. `convention.status.sentence_pending` is still the webview's job (`i18n/vi.json`).
- `read_scan_input` now takes `only: Option<&[i64]>`; the filtered path runs one indexed `SELECT ... WHERE id=?1 AND chapter_id=?2 AND retired_at IS NULL` per id (not a whole-Chapter read), to stay near the 1 ms/sentence budget on a 9,850-sentence Chapter. The Chapter-wide path is unchanged.
- Span building + Glossary suppression was extracted into `resolve_spans` (shared by the Chapter scan and the suggestion scan); `convention_scan_chapter` output is unchanged (all 57 contract tests green).

## Verified
- `cargo test --test convention_contract` (57), `config_invariants` (50), `convention_boundary`, `story_16_1_measure_convention` green; `cargo clippy --all-targets` clean.
- Counter-checks (real removals, reverted): ignoring the id filter turns `scanning_a_set_of_ids_touches_no_other_sentence` and `unknown_retired_...` red; passing `None` instead of the Glossary in the suggestion path turns `a_suggestion_inside_a_pinned_glossary_translation_...` red while the chapter-path glossary test stays green.
- Not run: full `cargo test`, vitest (plan says full run after the webview phase).

# Handoff: Webview phase -> Tests phase

## Exported names a test can reach
- `panels/editorPanelState.ts`: `onEditorFlushed(listener: (ids: readonly number[]) => void): () => void` (called after `lastSavedAt.value`, ids = keys of `snapshot`, listener throws are caught + `console.error`); `editorSegmentIsPending(id): boolean` (`flush.pending().has(id)`).
- `config/convention.ts`: `scanConventionSegments(generation, segmentIds)` -> `{scan: ConventionScanWire|null, error}`; `scanConventionSuggestions(generation, items: {segment_id,text}[])` -> `{scan: ConventionSuggestionScanWire|null, error}`; `isConventionSuggestionScanWire`. Invoke args: `{generation, segmentIds}` / `{generation, items}`.
- `conventionState.ts`: `scanConventionSegments(ids)` (merge, see below), `conventionGeneration` (readonly Ref<number>, mirrors the private `generation`; bumps in `resetConvention` and `scanConventionNow`).
- `conventionSuggestionMarks.ts` (new): `conventionSuggestionSingle` (`{segmentId, scannedText, spans} | null`), `conventionSuggestionBatch` (`Map<segmentId, {scannedText, spans}>`), `conventionSuggestionParts(text, entry)` -> `{text, title|null}[]` (no marks unless `entry.scannedText === text`), `resetConventionSuggestions()`.
- i18n: `convention.status.sentence_pending`.

## How a test drives it
- Flush observer: the follower is registered LAZILY inside the first `scanConventionNow()` call (`followFlushOnce`), not at module load. Reason: module-level `onEditorFlushed(...)` made ~every `vi.mock('.../editorPanelState')` factory in tests/frontend throw "No onEditorFlushed export" (30 tests in 10+ files). Lazy registration limits the breakage to tests that call `scanConventionNow`. A test must call `scanConventionNow()` once, capture the callback its mocked `onEditorFlushed` received, then call it with ids.
- `tests/frontend/conventionState.test.ts` currently FAILS (4 tests): its `vi.mock('../../src/panels/editorPanelState')` lacks `onEditorFlushed` and `editorSegmentIsPending`, and its `vi.mock('../../src/config/convention')` lacks `scanConventionSegments`. Add them (the only red in `npx vitest run tests/frontend`; `settingsOverlayAiConfigTierRender` failed once in the full run but passes alone: load flake).
- `scanConventionSegments(ids)`: dedups ids, calls adapter with the CURRENT generation (no bump), drops result if generation moved, on adapter error only `say('convention.status.scan_failed')` (no `scanFailed` flag set, no other change). Merge: deletes the entry of every requested id, then sets the ones in `scan.segments`; adds the four counts to the existing counts. Empty ids -> 0 calls.
- Suggestion scan: module-level `watch(..., {flush:'sync'})` on `aiTranslateStateValue` and `aiTranslateBatchStateValue`; any change drops that side's marks (+ invalidates in-flight scan token), `'done'` starts the scan. Single: item `{segment_id: aiTranslateRunSegmentId, text: aiTranslateAccumulatedText}` (skipped when blank/null id). Batch: ONE call with every row `status==='received' && text!==''`. Third watch on `conventionGeneration` clears both. A test either drives the real `aiTranslateState`/`aiTranslateBatchState` or mocks them with refs (`aiTranslateStateValue`, `aiTranslateAccumulatedText`, `aiTranslateRunSegmentId`, `aiTranslateBatchStateValue`, `aiTranslateBatchRows`) and sets them; mock `scanConventionSuggestions`. The module needs `./conventionState` (real or mocked with a `conventionGeneration` ref).
- Panel: `AiTranslationPanel.vue` renders `aiTranslateParts` / `batchRowParts` with `v-for` + `<span :class="{'convention-mark': title!==null}" :title>`; marked spans also carry `data-convention-suggestion-mark`. Batch rows other than `'received'` always get a single unmarked part.

## Deviations / things the plan did not predict
- Plan says clear suggestion marks "khi đổi câu". NOT done on purpose: the panel hides a stale suggestion anyway, and clearing would make the marks silently absent when the caret returns to the run's sentence. Instead marks are kept keyed by `segmentId` and rendering requires `entry.scannedText === displayed text` (and `entry.segmentId === aiTranslateRunSegmentId` for the single one). Chapter/Work/rule-list changes clear via `conventionGeneration`. A test for "đổi câu -> xoá" would fail; flip to "đổi câu -> không dấu trên suggestion cũ vì stale" if wanted.
- `conventionGeneration` also bumps on every `scanConventionNow` (replace-chapter rescan, accept in Overlay), so those clear the suggestion marks too until the next run.
- Counts are additive per the plan, so a rescanned sentence that was glossary-suppressed in the Chapter scan counts again (`glossarySuppressed` can exceed the true distinct total). Not hit by any display today (replace overlay uses its own scan).
- `scripts/check-panel-refs.mjs`: added a named EXEMPT for `editorPanelState.ts::flushedListeners` (same reason shape as `chapterOpenedListeners`). `check:panel-refs` still has 15 pre-existing FAILs in `conventionManageState.ts`/`conventionReplaceState.ts` (not mine).
- `ipc_argument_contract` is green now (the two `invoke` literals exist).

## Verified
`vue-tsc --noEmit -p tsconfig.json`, eslint on touched files, `check:i18n`, `check:commands`, `check:tokens` green; `cargo test --test ipc_argument_contract` green; a throwaway vitest (deleted) confirmed flush merge + counts + `sentence_pending` at both `say` sites + parts splitting. Not run: e2e, real-app render.
- `check:panel-refs`: second named EXEMPT `conventionState.ts::followsFlush` (lazy follower flag); the remaining 15 FAILs are pre-existing.

# Handoff: Tests phase -> review

## Files
- `tests/frontend/conventionState.test.ts`: mock factories repaired (`onEditorFlushed` captured in a list, `editorSegmentIsPending` over a set, `scanConventionSegments`); new blocks `rescan after flush` (8 cases of merge/ids/generation/counts/failure) and `F8 while the caret sentence is still pending`.
- `tests/frontend/conventionRescanAfterFlush.test.ts` (new): REAL `editorPanelState` + REAL `conventionState`, mocks only at `config/segment` and `config/convention`. Typing => 0 scan calls; flush => exactly one with exactly the flushed ids; failed save => 0; failed rescan => save still `saved` and stamp advances; a never-resolving scan does not block the flush; generation moved => dropped; throwing follower contained; `sentence_pending`.
- `tests/frontend/conventionSuggestionMarks.test.ts` (new): single + batch (50 rows => ONE call, only `received` rows, queued/missing get no mark), generating => 0, stale generation dropped, new run clears, parts split only when `scannedText === text`.
- `tests/frontend/aiTranslationPanelConventionMarks.test.ts` (new): mounted panel, `done` => one titled `[data-convention-suggestion-mark]`, no `data-convention-accept`/`-strip`/`-replace-chapter`; `generating` => 0 marks, 0 scans.
- `tests/frontend/aiTranslate.test.ts`, `aiTranslateBatch.test.ts`, `aiBatchPanelRows.test.ts`: +4 lines each, a `vi.mock('config/convention')` stub. Needed because `conventionSuggestionMarks` now fires the real adapter when those tests drive a run to `done`, and their `config/aitranslate` mock has no `isIpcError` (adapter `failure()` throws outside its `try`) => 27 unhandled rejections.
- `e2e/specs/story-16-4-convention-rescan-after-flush.e2e.mjs` (new).
- Plan Code Map `Tests:` line extended with the new/changed test files so `npm run test:story 16-4` selects 7 frontend files + 3 Rust targets with no THIẾU KHAI. No Rust, `src/` or `check-panel-refs.mjs` change.

## Traps found
- ⚠️ `window.__TAURI_INTERNALS__.invoke` is non-writable AND non-configurable (probed on WKWebView); the plan's "wrap `invoke` from the spec" cannot work, and a wrapper that silently does nothing makes "0 calls while typing" vacuous (it did). The spec counts at the transport instead: wraps `window.fetch`, commands leave as `ipc://localhost/<cmd>` (`http://ipc.localhost/<cmd>` on WebView2, in the regex, never run on Windows). It carries a self-check (calls the product adapter, must see `convention_scan_segments`) so a dead wrapper throws a named bench failure. The wdio driver itself polls `plugin%3Awdio|get_window_states`; filtered out.
- After the flush the log also shows `save_chapter_position` (not on the typing path; none seen within 300 ms of typing).
- Lazy follower (`followFlushOnce` inside `scanConventionNow`): a test must call `scanConventionNow()` first or no follower exists.
- Webview-phase deviation stands: no "clear on sentence change"; no test asserts it.

## Verification (real exit codes)
- `npm run build` 0; `cargo test --test convention_contract --test convention_boundary` 0 (57 + 6 passed).
- `npm run test:story 16-4` 0 (config_invariants 50, convention_contract 57, ipc_argument_contract 1; vitest 7 files / 121 tests).
- full `cargo test` 0 (110 `test result` lines, 0 failed). `check:commands` 0 (`327 điểm chuột · 36 khai pointer · 0 nợ`), `check:i18n` 0.
- full `npx vitest run tests/frontend`: clean 9d61b71a worktree (node_modules symlinked) exit 0, 261 files / 4145 tests, 0 errors, load avg 57 -> 107. Working tree BEFORE the 3-file mock fix: exit 1, 264 / 4176 passed, `Errors 28` = 27 `No "isIpcError" export is defined on the "../../src/config/aitranslate" mock` (aiTranslate 10, aiTranslateBatch 10, aiBatchPanelRows 7; stack `scanSingle/scanBatch -> scanConventionSuggestions -> failure`) + 1 `EnvironmentTeardownError ... onUserConsoleLog` (the known flake in the debt ledger; did not recur). So the 27 were NOT pre-existing: the Webview phase introduced them. AFTER the fix: exit 0, 264 files / 4176 tests, 0 errors (load avg 18 -> 157).
- e2e: ran on WKWebView outside the sandbox, `npm run test:e2e -- --spec e2e/specs/story-16-4-...` exit 0, 3/3 passed. Counter-check: `notifyFlushed(...)` replaced by `void 0` (reverted, `cmp` identical) => case 2 red with "the mark never came back after the flush".
- Vitest counter-checks (real edits, each reverted): no `notifyFlushed` => 4 red in conventionRescanAfterFlush; drop `status==='received'` filter => 2 red; `sentence_pending` forced off => 3 red; no `next.delete(wanted)` => 1 red; replace instead of merge => 5 red.

## Not done / risk
- Not run: CI (Windows half, UTC run) and nightly e2e read; no `gh` check made.
- No mounted-panel test for batch rows (only single suggestion): `batchRowParts` `status === 'received'` guard is covered at state level (which ids are sent) but not at render level.
- No test for "đổi câu" and none for AC "scanned_text của câu kia không bị ghi lại" beyond object identity of the untouched entry.
- Counts are additive, so `glossarySuppressed` can double count a re-flushed sentence (Webview-phase note).
