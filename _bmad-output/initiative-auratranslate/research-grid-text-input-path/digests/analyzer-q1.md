# Analyzer digest — Q1 @ 99f3fc1

Read through git at 99f3fc1, `_bmad-output/`, `_bmad/` excluded; 10 of 15 files plus one `git grep` into `src/main.ts`.

## 0. Wiring
- Listeners on the column, not per cell: `div.col.col-tgt` binds `@beforeinput="onBeforeInput"`, `@compositionstart/end`, `@input="onEditInput"`, `@keydown="onEditKeydown"`; events bubble from cells. — `src/panels/GridBlock.vue:271-281 @ 99f3fc1`
- Each target cell `div.cell.cell-tgt`, static `contenteditable="true"`, `:data-segment-id="s.id"`, `data-col="tgt"`, renders `{{ s.target_text }}` (loaded text, not live). — `src/panels/GridBlock.vue:322-344 @ 99f3fc1`
- Handlers from injected context: `GridBlock.vue:25-55` `inject(GRID_BLOCK_CONTEXT)`, throws `'GridBlock needs the grid block context'` if missing (`:26`). Type `src/panels/gridBlockContext.ts:22-50 (GridBlockContext)`; `onBeforeInput(event: InputEvent)` `:44`, `onEditInput(event: Event)` `:47`. Built `src/panels/GridPanel.vue:1816-1844 (gridBlockContext)`, provided `:1845` @ 99f3fc1.
- First to run: window keydown capture. `src/main.ts:1118` `attachKeyboard(window, …)` → `src/commands/index.ts:5442-5464 (attachKeyboard)` → `src/commands/keys.ts:601-607 (attachKeymap)` `addEventListener('keydown', …, { capture: true })`. In `keys.ts:510-545 (handle)`: `isComposing` ⇒ false (`:514`); `Mod+Z`/`Mod+Shift+Z` in typing zone gives way (`:522`); no-modifier chord in contenteditable ⇒ false via `isTypingZone` (`:527`; `:439-447`, `isContentEditable === true`); only matching chord ⇒ `preventDefault()` + dispatch (`:530-540`). Plain characters pass, `beforeinput`/`input` still fire.

## 1. `onBeforeInput` — `src/panels/GridPanel.vue:1435-1561 @ 99f3fc1` (load-bearing)
1. `targetCellOf(event.target)` = `closest('[data-col="tgt"]')` (`:746-749`). Check: no cell ⇒ return (`:1437`).
2. If `!composing && !event.isComposing` ⇒ `unwrapProofread(cell, mergeText = historyUndo/historyRedo)` (`:1438-1440`): proofread spans replaced by children, optional `normalize()`, selection restored — `src/panels/proofreadHighlight.ts:71-78 (unwrapProofread)`, `:51-65 (keepingSelection)`. `composing` module-level `let` `GridPanel.vue:1194`; `onCompositionStart` sets `:1205-1207`; `onCompositionEnd` clears + `scheduleRepaintProofread` `:1209-1212`.
3. ① `insertParagraph` (`:1461-1493`): `preventDefault()`; check: latch `insertingLineBreak` (`:1088`) returns if set (`:1468`); `document.execCommand('insertLineBreak')` with latch, reset in `finally` (`:1477-1482`); check: `false` ⇒ `console.warn('[grid] insertLineBreak refused…')` + `scheduleRepaintProofread()` (`:1483-1488`); no `reportEdit` — nested `insertLineBreak` fires its own `input` (`:1489-1492`).
4. `insertLineBreak` returns, engine handles (`:1494`).
5. ② if inputType not in `['insertFromPaste','insertFromDrop','insertReplacementText']` return (`:1497-1498`): insertText, deletions left to engine, write on `input`.
6. Outside insertions: `preventDefault()` before every exit (`:1504`); raw `dataTransfer.getData('text/plain') ?? event.data ?? ''` (`:1505`); flattened `\r\n?`→`\n`, `[ \t]+`→`' '` (`:1524`). Checks: `flat === ''` repaint + return (`:1528-1531`); no selection / `rangeCount === 0` warn `'[grid] insert skipped: no readable selection'` (`:1536-1540`); range starts outside cell warn (`:1542-1549`). Else `range.deleteContents()`, insert text node, collapse after (`:1551-1557`), `reportEdit(cell)` by hand (`:1560`).

## 2. `onEditInput` — `GridPanel.vue:1712-1715 @ 99f3fc1`
- `targetCellOf(event.target)`; non-null ⇒ `reportEdit(cell)`. Never reads `event.data`.

## 3. `reportEdit` — `GridPanel.vue:1699-1704 @ 99f3fc1` (load-bearing; only point DOM text enters state)
- `segmentIdOf(cell)` reads `data-segment-id` via `Number()` (`:751-756`); check: null if missing/not finite ⇒ return (`:1701`).
- `noteEditorEdit(id, cell.textContent)`, then `clearProofreadFor(id)`.

## 4. State writes
- `noteEditorEdit` — `src/panels/editorPanelState.ts:758-814 @ 99f3fc1` (load-bearing)
  - Check: `dangChuyenChuong` (declared `:1952`, set `:1973/:2199`) ⇒ `console.info('[editor] bo qua mot luot go…')`, edit dropped (`:779-784`).
  - `editedText.value.set(id, text)` (`:791`) in place, no trigger; shallowRef (`:682`), commented deliberately non-reactive per keystroke (`:679-680`).
  - `editedShape.value.set(id, shape)`, `triggerRef(editedShape)` only when `textShapeOf` changed vs previous (`:786-793`); `GridBlock.vue:317,328` reads it for `v-memo` and `empty` class.
  - `dropTmFilled(id)` replaces `tmFilledSegmentIds` only if id present (`:794`, `:156-161`).
  - `flush.markChanged(id, text, Date.now())` (`:796`) → `src/panels/editorFlush.ts:132-135` `dirty.set(id, text)` + `schedule.onChange(now)` → `src/layout/writeSchedule.ts:91-103 (createWriteSchedule onChange)` `firstChangeAt ??= now`, `due = min(now + idleMs, firstChangeAt + hardCapMs)`; `EDITOR_IDLE_MS = 2000`, `EDITOR_HARD_CAP_MS = 5000` (`editorFlush.ts:48,61`); `flush` singleton at module scope (`editorPanelState.ts:618`).
  - `armFlushTimer()` (`:797`, `:738-749`) clears and re-arms one `setTimeout` to `flush.deadline()` → `flushEditorNow()`.
  - `datThongBao({})` (`:813`, `:2678-2690`) nulls `confirmNotice`, `regroupNotice`, `navNotice`, `splitChapterNotice`.
- `clearProofreadFor` — `src/proofreadState.ts:249-271 @ 99f3fc1`: return if id not in `runSegmentIds`; `dropReverts(id)` (`:318-323`); while scanning: single-segment run ⇒ `resetProofread()` (`:302-316`), else add to `editedWhileScanning`; results exist ⇒ replace `results` with copy minus id; `done` + one segment ⇒ bump `sequence`, `idle`, empty `runSegmentIds`, `clearResult()` (`:117-125`). Downstream `watch([proofreadResults, editorSegments, gridRoot], repaintProofread)` (`GridPanel.vue:1214`); `repaintProofread` no-op while `composing` (`:1184-1190`).

## Call stack
- window keydown capture `keys.ts:607` → `handle` `:510`
- `col-tgt @beforeinput` `GridBlock.vue:276` → `onBeforeInput` `GridPanel.vue:1435` → `targetCellOf` `:746`, `unwrapProofread` `proofreadHighlight.ts:71`, [insertParagraph] `execCommand('insertLineBreak')` `:1479`, [paste/drop/replacement] `reportEdit` `:1560`
- `col-tgt @input` `GridBlock.vue:279` → `onEditInput` `GridPanel.vue:1712` → `reportEdit` `:1699` → `segmentIdOf` `:751`; `noteEditorEdit` `editorPanelState.ts:758` → `dropTmFilled` `:156`, `markChanged` `editorFlush.ts:132` → `onChange` `writeSchedule.ts:91`, `armFlushTimer` `editorPanelState.ts:738`, `datThongBao` `:2678`; `clearProofreadFor` `proofreadState.ts:249`

## Two patterns writing the same text (unranked)
- Engine-handled edits (typing, deletes, undo, IME commit, nested `insertLineBreak`) reach `reportEdit` via `onEditInput`.
- Paste/drop/replacement call `reportEdit` directly from `onBeforeInput` after `preventDefault()`.
- Both read `cell.textContent`.

## Related, not traced
- `onEditKeydown` (`GridPanel.vue:1608-1621`) unwraps (`mergeText = true`) and schedules repaint; Backspace merge `preventDefault()` per `editorPanelState.ts:2717-2718`, dispatched `GridPanel.vue:1695`, not traced.

## Looked for, not found / not followed
- No reactive write of typed text into `segments`; cells keep `s.target_text`; `restoreEditedText` (`GridPanel.vue:1168-1181`) copies `editedText` back after re-render, skipping focused cell.
- `noteEditorEdit` does not trigger `editedText`.
- Not followed: `flushEditorNow`, `KeymapGate` from `main.ts:1118`, `textShapeOf`, `repaintProofreadHost`.
