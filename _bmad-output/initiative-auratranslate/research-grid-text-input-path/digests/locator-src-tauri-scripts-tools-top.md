# Locator digest — `src-tauri/` (minus `src-tauri/tests/`), `scripts/`, `tools/`, top-level files @ 99f3fc1

No frontend reactive code in area; nothing reads/writes `editedText`, `editedShape`, `lastSavedAt`, `triggerRef`, `v-memo`, GridBlock render logic.

## Q1
- `src-tauri/src/commands/segment/wire.rs:144 (save_segment_targets) @ 99f3fc1` — `#[tauri::command]` shell for AD-35 flush; `chapterId`, `edits`; holds `OpenWorkState` MutexGuard for whole call; delegates to `flush_segment_targets`; `src/config/segment.ts` sole typer of wire names.
- `src-tauri/src/commands/segment/confirm.rs:515 (flush_segment_targets) @ 99f3fc1` — two transactions in order: `unconfirm_edited_segments` (confirmed rows with changed text → `'draft'`, `AND target_text <> ?3`), then `save_segment_targets`; doc :495-514.
- `src-tauri/src/commands/segment/targets.rs:94 (save_segment_targets) @ 99f3fc1` — empty batch returns early; one transaction, `prepare_cached` once, `UPDATE` sets only `target_text`, `updated_at`; unknown id rejects whole batch (`BatchReject`, :192); no status change, no `SegmentVersion`. Doc: batching reasons (9,850 segments, 57–64 ms parse, single serial writer). `SegmentTargetEdit` :11, `SaveOutcome` :20.
- `src-tauri/src/lib.rs:1266 (run) @ 99f3fc1` — registers command; discrete actions (`confirm_segment`) write immediately outside 2 s/5 s buffer.
- `src-tauri/AGENTS.md:26 @ 99f3fc1` — flush rule: idle 2 s, 5 s hard cap not reset by keystrokes, flush on confirm/leave segment/close Work; done only once in WAL.
- `scripts/check-layout.mjs:449 (DOM API allowlist, Kiểm C) @ 99f3fc1` — `GridPanel.vue::onBeforeInput` `preventDefault()` on paste/drop, inserts flattened text via `document.createTextNode` (AD-37).
- `scripts/check-layout.mjs:480 @ 99f3fc1` — `onBeforeInput` blocks `insertParagraph`, emits `insertLineBreak` via `document.execCommand` (keeps undo stack under `white-space: pre-line`).
- `scripts/check-layout.mjs:458 @ 99f3fc1` — `caretPositionFromPoint`/`caretRangeFromPoint` for `GridPanel.vue::placeCaretAtPoint`.

## Q2
- No reader. Doc obligations (flush first because `editorEditedText` may hold unflushed chars): `src-tauri/src/commands/segment/history.rs:258`, `wire.rs:322 (restore_segment_version)`, `wire.rs:628`, `regroup.rs:445 @ 99f3fc1`.
- `scripts/check-panel-refs.mjs:49 (§GIỚI HẠN THẬT) @ 99f3fc1` — every module-level ref in `src/**/*.ts` through reset fn or named exemption; names `resetEditorPanel()` → `clearFlushTimer()`; :363 `editorPanelState::inFlight` cleared on Work change.

## Q3
- Nothing on `v-memo`/slots/render. `scripts/check-commands.mjs:2588 (HANDLER_PROVIDER) @ 99f3fc1` — GridBlock.vue handlers defined in GridPanel.vue via `inject`; inventory :2566-2569 (`onSourceCellMouseUp`, `onCellMouseDown`, `onCellMouseUp`, `onEditKeydown` → `editor.clear_source_cuts`, `editor.merge_segments`).
- `scripts/check-commands.mjs:1939 (GRID_VUE) @ 99f3fc1` — exactly one `'source'` and one `'display'` role.

## Q4
- `scripts/check-commands.mjs:2094 (Kiểm I, EDITOR_PANEL_VUE) @ 99f3fc1` — margin-rule markers via `resolveSegmentRule` (`src/panels/editorSegments.ts`), inputs `hasCaret`, `isConfirmed`, `isTmFilled`, `targetText`; five values `confirmed`, `primary`, `tm-rule`, `draft`, `none`; checks `.rule-*` CSS in GridBlock.vue both ways.
- `scripts/check-commands.mjs:977 @ 99f3fc1` — `isTypingZone` self-tests. `:2464` `StatusBar.vue::focusOnPointerDown` unrelated.
- `src-tauri/src/core/store/mod.rs:252 (Tuning.idle_before_passive) @ 99f3fc1` — 5 s idle before PASSIVE checkpoint, longer than 2 s flush.
- `src-tauri/src/lib.rs:164 (EXIT_FLUSH_EVENT) @ 99f3fc1` — window-close handshake; `EXIT_FLUSH_BUDGET` :182 = 1,200 ms; `wire_exit_flush` docs :1974; completion :2121.
- No proofread repaint code.

## Q5
- Backend reasons: `targets.rs:94`, `confirm.rs:495-575` docs. `src-tauri/src/core/i18n/mod.rs:253 (SegmentUnknownIds)` whole-batch reject. `scripts/check-commands.mjs:2260 (Kiểm J removed) @ 99f3fc1` — typing banned until flush path passed `segment_contract.rs`.
- Nothing on Map replacement, `editedShape`, block split.

## Q6
- `vitest.config.ts:1 @ 99f3fc1` — happy-dom, only `tests/frontend/**/*.test.ts`, setup `tests/frontend/support/setup.ts`, `flushPromises()` not timer waits.
- `package.json:8 (scripts) @ 99f3fc1` — `test` `vitest run`; `test:e2e` `cargo build --features wdio` + `wdio run e2e/wdio.conf.mjs`; `test:story` `scripts/test-story.mjs`.
- `src-tauri/Cargo.toml:205 (wdio feature)`; wdio hooks `src-tauri/src/lib.rs:74`.
- Cargo tests named in docs (in `src-tauri/tests/`): `segment_contract.rs::the_flush_path_lowers_the_state_before_it_writes_the_new_text` (confirm.rs:509), `a_flush_touches_exactly_target_text_and_updated_at_and_nothing_else` (confirm.rs:551), type/flush/reload round trip + rejected batch (check-commands.mjs:2269).
- No render-cost/benchmark test in area.

## Searched, nothing found
- `git grep -E -i` over area: `editedShape`, `lastSavedAt`, `triggerRef`, `v-memo`/`vMemo`, `renderCount`, `render cost`, `benchmark`, proofread repaint/highlight, `ruleMarker`. `editedText` only as `editorEditedText` in docs; `shallowRef`/`flush` only in unrelated gate code. `tools/dict-build/` nothing; `eslint.config.js` only `vue/no-v-html`, `vue/no-ref-as-operand`; `index.html`, `vite.config.ts` nothing.
