# Locator digest — area `tests/`, `e2e/`, `src-tauri/tests/` @ 99f3fc1

All searches via `git grep` / `git show` at 99f3fc1, `_bmad-output/` and `_bmad/` excluded. Area is tests only: Q6 directly, Q1–Q5 indirectly (what tests drive/assert, commit messages).

## Q1
- `tests/frontend/gridTypingCost.test.ts:108 (type) @ 99f3fc1` — keystroke simulated: set `cell.textContent`, dispatch bubbling `input` on `[data-col="tgt"][data-segment-id]`, `$nextTick`; reads `state.editorEditedText.value.get(id)`.
- `tests/frontend/editorTypingZone.test.ts:285 @ 99f3fc1` — `input` ⇒ `editorEditedText` holds DOM text read from the cell.
- `tests/frontend/editorTypingZone.test.ts:200 @ 99f3fc1` — `beforeinput insertParagraph` is `defaultPrevented` (replaced by line break); `insertLineBreak`, `keydown Enter` (composing or not) not prevented; `data-segment-id` count unchanged. `:263` newline reaches pending set with `\n`.
- `tests/frontend/editorTypingZone.test.ts:383 @ 99f3fc1` — IME composition: nothing prevented; `editorEditedText` gets committed text.
- `tests/frontend/proofreadWiring.test.ts:174, :181, :190 @ 99f3fc1` — source-regex checks on `src/panels/GridPanel.vue` pinning handler order: `noteEditorEdit(id, cell.textContent)` then `clearProofreadFor(id)`; keydown not composing: `unwrapProofread(typed, true)` then `scheduleRepaintProofread()`; `input` not composing: `unwrapProofread(cell, inputType === 'historyUndo'|'historyRedo')`; cancelled `beforeinput` without write calls `scheduleRepaintProofread()`; `GridBlock.vue` binds `@compositionstart`/`@compositionend`.
- `tests/frontend/statusBar.test.ts:278 @ 99f3fc1` — `noteEditorEdit` also clears the status notice; "Đã lưu" returns.

## Q2
- `tests/frontend/gridTypingCost.test.ts:130, :144 @ 99f3fc1` — typing toggles `.empty` on target cell at once; `.col-rule .margin-slot` rule class checked after caret leaves.
- `tests/frontend/gridTypingCost.test.ts:158 @ 99f3fc1` — `goToNextUntranslated()` treats typed-not-flushed row as translated.
- `tests/frontend/gridTypingCost.test.ts:179 @ 99f3fc1` — `resetEditorPanel()` empties `editorEditedText`.
- `tests/frontend/gridCaretMoveCost.test.ts:140, :152 @ 99f3fc1` — typing in a row without caret changes `empty` and rule; draft row edited to blanks changes `empty`, rule stays `rule-draft`.
- `tests/frontend/gridRuleStable.test.ts:92 @ 99f3fc1` — typing into empty row ⇒ `rule-draft`.
- `tests/frontend/gridRestoreEditedText.test.ts:48, :61 @ 99f3fc1` — after `replaceEditorSegment` pending text restored; restore looks up only pending ids, no `querySelectorAll('[data-segment-id]')` scan.
- `tests/frontend/editorTypingZone.test.ts:535 @ 99f3fc1` — typed text survives GridPanel unmount/remount; `editorEditedText` lives in state module.
- `tests/frontend/statusBar.test.ts:89, :105 @ 99f3fc1` — StatusBar shows "Đã lưu N giây trước" after first flush, reads `editorLastSavedAt`.
- `tests/frontend/editorLeaveSegment.test.ts:111, :128 @ 99f3fc1`, `tests/frontend/editorFlushRetry.test.ts:116 @ 99f3fc1` — `editorLastSavedAt` null after failed flush, non-null after success.
- `e2e/support/flushWait.mjs:72 (readLastSavedAt) @ 99f3fc1` — e2e waits for flush via `editorLastSavedAt`; comment `:27` cites `src/panels/editorPanelState.ts:246`.
- Not found: `editedShape|triggerRef|flushPending|pendingIds` in this area; pending set only via `createEditorFlush().pending()` in `editorFlush.test.ts`.

## Q3
- `tests/frontend/gridBlockRenderCost.test.ts:99 @ 99f3fc1` — 1,050 segments; mixin `updated()` records `$props.block.index`. Caret in block `[1]`; across 399→400 `[1,2]`; strip anchor + caret to other block `[1,3]` with 4 `.strip-pad`; anchor only `[1,4]`; confirm at boundary `[1,2]`; confirm inside `[1]`. No typing case.
- `tests/frontend/gridBlocks.test.ts:97 (gridBlocksOf) @ 99f3fc1` — `GRID_BLOCK_ROWS = 200`; 450 rows ⇒ 200/200/50; 201st row opens second block. `:123` mount as blocks, numbering, strip pad, split/merge at 199, caret in block 30; `:235, :272` per-cell source surface, selection/copy across block boundary.
- `tests/frontend/gridTypingCost.test.ts:115 @ 99f3fc1` — stubbed `PanelFrame` counts slot renders (`probe.frameRenders`): 0 renders and 0 `segmentRuleInputOf` calls over 3 keystrokes at 20 and 200 rows.
- `tests/frontend/gridCaretMoveCost.test.ts:111 @ 99f3fc1` — caret move: 0 `segmentRuleInputOf` calls; `:125–:270` each memo input reaches DOM (caret, written text, glossary, HanViet, row error, unlinked, selection, source cut, concordance, strip anchor).
- `tests/frontend/gridConfirmWorkCost.test.ts:180 @ 99f3fc1` — confirm work equal at 1,050 and 2,100 segments.
- `tests/frontend/gridRuleStable.test.ts:43, :79 @ 99f3fc1` — caret move adds/removes no rule element; rule border width equals `.margin-slot`.
- `e2e/specs/grid-blocks.e2e.mjs:80, :89, :114, :150 @ 99f3fc1` — 1,200 sentences ⇒ blocks `[200×6]`; cross-block jump, Lookup block 5, `Mod+Shift+↓` across boundary.

## Q4
- Caret: `tests/frontend/editorTypingZone.test.ts:160, :469, :507 @ 99f3fc1` — caret offset survives; `appendData` keeps same `firstChild` text node ("gõ ngược" regression when template rendered `editorEditedText.get(id) ?? target_text`); typing one row never changes another.
- `tests/frontend/editorCaretAtCellStart.test.ts:79–154 @ 99f3fc1` — caret-at-cell-start detection.
- Typing zone: `tests/frontend/editorTypingZone.test.ts:121, :140 @ 99f3fc1` — every target cell `contenteditable` from mount; 2×N `data-segment-id` anchors.
- Proofread: `e2e/specs/proofread-underline.e2e.mjs:119, :406, :415, :424 @ 99f3fc1` — spans drop to 0 on keydown, return 150 ms later, return after blocked `beforeinput`, stay during composition.
- Flush cadence: `tests/frontend/editorFlush.test.ts:53 @ 99f3fc1` — 2000/5000; ≥6 flushes in 30 s stream, gaps ≤5 s; one key ⇒ one flush at 2000; in-flight typing kept; successful write does not reset cap; batch only changed ids. `editorFlushRetry.test.ts:81–175`, `editorLeaveSegment.test.ts:42–131`.
- Status bar: `tests/frontend/statusBar.test.ts:89–347 @ 99f3fc1`.

## Q5 (commit messages, ≤10)
- `043f3bc` (2026-10-23, "perf(editor): 2-15 …") — every `input` replaced the whole edited-text `Map`; grid body is `PanelFrame` slot so each key re-ran 4 cols × n rows + `restRuleById`. Fix writes Map in place, notifies grid only when a row's emptiness changes. Remaining ~0.5 s/key at 9,850 sentences release. Conflicts with brief's premise of whole-Map replacement — check source.
- `43729b3` (2026-10-23, "perf(grid): 2-14") — caret move ~390 ms WebKit `subgrid` relayout when `v-if` inserted `div.rule` + ~225 ms slot re-render; fix: rule always present, `v-memo` on four columns, rule 3 px.
- `742096c` (2026-10-24) — grid split into 200-row blocks, one `GridBlock` each, selection surface moved to grid root.
- `f62012f` — confirm re-rendered all 50 blocks via identity changes of shared values (`gridBlockViews`, `restRuleById`, `unlinkedIds`, `glossaryMarksPerSegment`); blocks kept by identity.
- `8260a92` — confirm work O(1) per Chapter; `gridConfirmWorkCost` added.
- `7c9a3a4` — `gridBlocks.test.ts`, `grid-blocks.e2e.mjs` added. `2aa120f` — review fixes, cross-block selection/copy.
- `src-tauri/tests/flush_cadence_contract.rs:1–52 @ 99f3fc1` — flush/checkpoint conflict would show as typing stalls.

## Q6
- vitest render-cost counters (count calls/renders, no wall-clock): `gridTypingCost.test.ts:115–179`, `gridCaretMoveCost.test.ts:111–270`, `gridBlockRenderCost.test.ts:99`, `gridConfirmWorkCost.test.ts:180`, `gridRuleStable.test.ts:43–92`, `gridBlocks.test.ts:97–293`.
- vitest typing behaviour: `editorTypingZone.test.ts:121–535`, `gridRestoreEditedText.test.ts:48–61`, `editorCaretAtCellStart.test.ts`, `editorFlush.test.ts:53–131`, `editorFlushRetry.test.ts`, `editorLeaveSegment.test.ts`, `statusBar.test.ts`, `proofreadWiring.test.ts:132–190` (`:132` typing+flush make 0 `ai_proofread_*` invokes).
- cargo: `src-tauri/tests/flush_cadence_contract.rs:127 (the_checkpoint_idle_gate_stays_strictly_longer_than_the_editor_flush_cadence) @ 99f3fc1`; self-checks `:157, :188, :195, :210`. `segment_contract.rs::typed_text_round_trips_through_the_flush_and_the_load_command` referenced from `e2e/specs/editor-typing-flush.e2e.mjs:1`, not opened. `nfr_bench_wiring_contract.rs:58` unrelated.
- e2e: `e2e/specs/editor-typing-flush.e2e.mjs:92 @ 99f3fc1` (real WKWebView, contenteditable, `execCommand('insertText')` reaches `project.db`, status `^Đã lưu \d+ giây trước$`; comment ~`:165` says deliberately red on empty untranslated span, 2026-08-13 — possibly stale, unverified); `:234` Enter inserts `\n`, no row split. `proofread-underline.e2e.mjs:119–424`, `grid-blocks.e2e.mjs:80–150`.

## Searched, found nothing
- `editedShape|EditedShape|triggerRef` — no hits.
- `flushPending|pendingIds|editorPending` — only unrelated `editorRegroupAssetRefresh.test.ts:315`.
- `keystroke|typing latency|input latency|performance.now` — no per-keystroke timing test.
- No test counts `GridBlock` updates on a keystroke.
