# Analyzer digest — Q4 @ 99f3fc1

Read through `git show`/`git grep` at 99f3fc1. Budget reached at 15 files.

## Event order for one keystroke in a translation cell
- keydown, capture on `window`:
  - `src/commands/keys.ts:601-607 attachKeymap @ 99f3fc1` — calls gate `isBlocked`; true ⇒ exit without `preventDefault`; else `keymap.handle`.
  - Gate `src/main.ts:1118-1152 boot @ 99f3fc1` — OR of ~20 overlay/mode refs, read every keydown.
  - `keys.ts:510-545 handle @ 99f3fc1`: `isComposing` ⇒ false (`:514`); `Mod+Z` in typing zone ⇒ false (`:522`); loops every compiled chord (`:525`), `continue` unless `code` and mods match (`:526`); match without primary modifier in typing zone ⇒ false (`:527`, load-bearing); else `preventDefault`, non-repeatable repeat stops (`:539`), else dispatch (`:540`).
  - `keys.ts:439-447 isTypingZone @ 99f3fc1` true when `isContentEditable === true`; reached only after a chord matches. Comment `src/panels/GridPanel.vue:1578-1596 onEditKeydown`: bare `Enter` exits at `sameMods`.
- keydown, bubble on `.col-tgt` (`src/panels/GridBlock.vue:271-281 @ 99f3fc1`, one listener per column) → `GridPanel.vue:1608-1696 onEditKeydown @ 99f3fc1`:
  - `event.isComposing` ⇒ return (`:1614`).
  - `!composing` + target cell ⇒ `unwrapProofread(typed, true)` then `scheduleRepaintProofread()` (`:1615-1620`). Load-bearing: runs for every non-composing keydown in a cell, incl. arrows, Shift, Cmd.
  - `Escape` ⇒ `dispatch('editor.clear_source_cuts')`, no `preventDefault` (`:1644-1647`); return unless `Backspace` (`:1652`); return if `event.repeat` (`:1663`) or `caretAtCellStart` false (`:1669`); else `preventDefault` + `dispatch('editor.merge_segments')` (`:1680-1695`).
- beforeinput: `GridPanel.vue:1435-1440 onBeforeInput @ 99f3fc1` — not composing ⇒ `unwrapProofread(cell, …)` again, `mergeText` only for `historyUndo`/`historyRedo`; reason `src/panels/proofreadHighlight.ts:67-70 @ 99f3fc1`: a merge from an input event detaches the text node WebKit is inserting into.
- input: `GridPanel.vue:1712-1715 onEditInput` → `:1699-1704 reportEdit` → `noteEditorEdit`, `clearProofreadFor`.
- selectionchange: two `document` listeners `GridPanel.vue:1325-1332 @ 99f3fc1`; code comments say it fires per typed character in `contenteditable` (`:1279`, `:1299-1300`).
  - `GridPanel.vue:1234-1240 onSelectionChange` — return unless anchor in `.col-tgt` (`:669-672 isInTargetColumn`); else `setEditorCaret(segmentIdOf(targetCellOf(anchor)))`.
  - `GridPanel.vue:1289-1323 onSourceSelectionChange` — always writes `sourceSelecting.value = isSourceColumnSelecting(...)` (`:1291`; false for collapsed selection, `src/panels/gridSourceSelection.ts:7-11`); `closest('[data-col="src"]')`, return at `:1311` outside source (keeps typing from clearing mouse glossary hover, comment `:1296-1310`).
- keyup: `src/panels/selectionContract.ts:390-396 attachSelectionWatcher @ 99f3fc1` (attached `src/main.ts:1271`), returns unless `event.key === 'Shift'`.

## Caret path (load-bearing)
- `src/panels/editorPanelState.ts:281-296 setEditorCaret @ 99f3fc1`, called on every selectionchange in target column ⇒ every keystroke.
  - Assigns `caretSegmentId.value = id`, `lastCaretSegmentId.value = id` (plain `ref`s `:148-149`).
  - `flushEditorNow` only when `left !== null && id !== null && left !== id` (`:286`) — never within one segment.
  - `id !== null` + chapter open ⇒ `positionFlush.markMoved(...)` + `armPositionFlushTimer()` (`:292-295`), unconditional ⇒ every keystroke clears and re-arms that timer. `src/panels/positionFlush.ts:89-92 markMoved` → `schedule.onChange(now)`; `src/layout/writeSchedule.ts:91-102 onChange`; `POSITION_IDLE_MS 500`, `POSITION_HARD_CAP_MS 5000` (`positionFlush.ts:35`, `:42`). `editorPanelState.ts:636-644 armPositionFlushTimer`, `:652-670 flushChapterPositionNow` → `saveChapterPosition` IPC. During continuous typing in one segment a position write fires at least every 5 s, plus ~500 ms after stop; same segment id each time.
- Inference (Vue semantics): same-value ref assignment does not trigger ⇒ these do not re-run per keystroke while caret stays: `src/TmFuzzyStrip.vue:30`, `src/ProofreaderStrip.vue:40`, `src/GlossaryConfirmStrip.vue:60-66`, `src/inlineStripEligibility.ts:34,57,66,69`, `src/proofreadScope.ts:19`, `src/proofreadState.ts:276-278,350-383`, `src/StatusBar.vue:154-160 positionInChapter`, `GridPanel.vue:654-662 caretIndex/caretIdIn`.
- `useSelectionSurface` (`selectionContract.ts:132-152`) registers via `watch(elRef, …, flush:'sync')`; grid calls it twice `GridPanel.vue:739-740`; nothing per keystroke.
- `onColumnFocusOut`/`onStripFocusOut` (`GridPanel.vue:1132-1142`) null caret only on focusout outside.

## Proofread underline repaint (load-bearing)
- `GridPanel.vue:1197-1203 scheduleRepaintProofread @ 99f3fc1` — `setTimeout(…, 0)` debounce, one pending timer reset per call.
- `GridPanel.vue:1184-1190 repaintProofread` — exits if no `gridRoot`, `composing`, or `results.size === 0 && !proofreadPainted`; else `repaintProofreadHost` on whole grid root.
- `src/panels/proofreadHighlight.ts:117-128 repaintProofreadHost @ 99f3fc1` — `unwrapAllProofread(host)` (`querySelectorAll` over whole grid, `unwrapProofread(cell, true)` + `normalize()` per cell with spans, `:102-109`), then per result entry one `host.querySelector` + `wrapProofread`. While any proofread results exist, each keydown repaints every scanned cell in the grid.
- `proofreadHighlight.ts:84-100 wrapProofread` — no-op if `cell.textContent !== scannedText` or no marks (`:85`); edited cell never rewrapped; arrow in scanned unedited cell unwraps on keydown, rewraps on timer.
- `proofreadHighlight.ts:51-65 keepingSelection` — offsets via `offsetOf` (Range `toString`), apply, map back with `pointAt` (TreeWalker), `selection.setBaseAndExtent`; else apply directly. Inference: restoring selection fires another `selectionchange`.
- Watch `GridPanel.vue:1214 @ 99f3fc1` `[proofreadResults, editorSegments, gridRoot]` `flush:'post'` repaints. `src/proofreadState.ts:249-271 clearProofreadFor` returns unless segment in `runSegmentIds` (`:250`); else replaces `results.value` with new Map (`:262-264`) ⇒ triggers watch (in practice first keystroke into a scanned segment); single-segment scanning ⇒ `resetProofread()` (`:252-256`).
- IME `GridPanel.vue:1205-1212` — start `composing = true` suppresses keydown/beforeinput unwrap and repaint; end clears and schedules repaint.
- `GridPanel.vue:1168-1181 restoreEditedText` not per keystroke; skips focused cell, then `repaintProofread`.

## Rule markers and row classes
- `editorPanelState.ts:786-793 noteEditorEdit` `textShapeOf` (0/1/2, `src/panels/editorSegments.ts:177-183`), `triggerRef(editedShape)` only when shape class changes (load-bearing). `dropTmFilled` (`:156-161`) replaces set only if TM-filled.
- On trigger `GridPanel.vue:615-643 gridBlockViews` recomputes: `idsWithChangedRule` (`:599-613`); unchanged blocks reused; others rerun `restRulesOf` (`:278-293`, `resolveSegmentRule` with `targetText` `''`/`'x'`); `sameRules` keeps old object (`:635-637`).
- `src/panels/GridBlock.vue:74-76 ruleOf` `'primary'` when `props.caretId === id` ⇒ typed row stays `primary`.
- Rule-column memo on `ruleClassForRow` (`GridBlock.vue:113`); target memo on `editorEditedShape.get(s.id)`, `ruleOf(s.id)` (`:311-319`); `empty` `editorEditedShape.get(s.id) ?? textShapeOf(s.target_text)` (`:328`); `aria-label` `cellLabelOf` (`:86-93`, `:341`).
- Inference: every mounted GridBlock reads `editorEditedShape` ⇒ `triggerRef` re-renders every instance, `v-memo` skips unchanged rows. Cell renders load-time `{{ s.target_text }}` (`:344`); see `GridPanel.vue:1144-1158`.

## Status bar
- Only per-keystroke write: `editorPanelState.ts:813` `datThongBao({})`, `:2678-2690` nulls four notice `shallowRef`s (`:1285`, `:2488`, `:2617`, `:2645`) ⇒ first keystroke after a notice clears it; afterwards null→null, no trigger (Vue semantics). Template `v-if` chain `src/StatusBar.vue:534-596`; computeds `:308`, `:361`, `:459`.
- Glossary hover line (`:592`) shielded by `GridPanel.vue:1311`.
- "Saved N seconds ago": `StatusBar.vue:85-94` 1 s `setInterval`, writes `now` once `editorLastSavedAt !== null`, independent of keys; `secondsSinceSave` `:279-283`. Saved timestamp moves only when editor flush runs; flush timer re-armed per key `editorPanelState.ts:796-797` (`armFlushTimer` `:738-749`); cadence `src/AGENTS.md:29`.
- `positionText`, `confirmedCounterText` (`StatusBar.vue:168-190`) read caret and `editorSegments`, unchanged per key.

## Two places unwrapping proofread spans per keystroke
- keydown `GridPanel.vue:1618` (`mergeText = true`) + scheduled repaint.
- beforeinput `GridPanel.vue:1439` (`mergeText` undo/redo only), no repaint.

## Searched, not found
- No other document/window `keydown`, `selectionchange`, `input`, `beforeinput`, `keyup` listeners in `src` (`addEventListener('(keydown|selectionchange|input|beforeinput|keyup)'` @ 99f3fc1) besides selectionContract `keyup`/`mouseup` `:395-396`.
- No component outside GridPanel/GridBlock reads `editorEditedShape`, `editorEditedText`, `editorTmFilledSegmentIds`.
- Not read in full: `src/inlineStripEligibility.ts`, `src/TmFuzzyStrip.vue`, `src/ProofreaderStrip.vue`, `src/proofreadScope.ts`; bodies of `syncTmFuzzyStrip`, `syncGlossaryConfirmStripTarget`, `syncProofreadStripHidden` beyond `proofreadState.ts:290-292`; `onBeforeInput` beyond `:1435-1470`; `caretAtCellStart`.
