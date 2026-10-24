---
title: 'Codebase research: grid text input path'
type: codebase
ask: |-
  Dig into the solution before planning the fix: at 40 keys/s on a 122-sentence Chapter, keystrokes queue in the webview (NFR18 window 7–19 s; Rust save ≤ 1.1 ms; suppressing the flush storm did not shrink the window). Map what runs on each keystroke in the grid at HEAD, as the factual base for the plan that reduces per-input cost; a web pass on solutions follows.
repository: '/Users/hoangnam/LocalSites/addon/AuraTranslate'
commit: '99f3fc1c0c46326e6743b6a60e63f96889c480c4'
branch: 'master'
status: complete
validation: 'normal'
claims: {disputed: 1, verified: 26}
created: '2026-10-24'
updated: '2026-10-24'
---

# Codebase research: grid text input path

Readers: the plan step that will change the code, and a person checking one claim. Every claim cites a line at `99f3fc1` or a commit, except those marked as an unverified reading. This document makes no recommendation.

## Questions

Each question carries its shape: how, what touches, or why.

1. (how) What steps does a `beforeinput` or `input` event in a target-column cell of the grid go through, from the DOM listener to the state writes it causes?
2. (what touches) Which components, computeds, watchers or render functions read the reactive state that the path in question 1 writes (`editedText`, `editedShape`, `lastSavedAt`, the flush pending set, and any other state written there)?
3. (how) How does `GridBlock` decide whether to re-render, and what does a re-render of a block or a row produce (`v-memo` lists, slots, props passed down from `GridPanel`)?
4. (how) Besides state writes, what other work runs around one keystroke in the grid: caret and selection handling, rule markers, proofread repaints, the typing zone, the status bar?
5. (why) Why is the edited-text state shaped the way it is (the whole `Map` replaced per edit, `editedShape` with `triggerRef`), and why is the grid split into blocks the way it is? — The premise "the whole `Map` replaced per edit" no longer holds at this commit; section 5 says when it changed.
6. (how) Which tests (vitest, cargo, e2e) cover typing in the grid and its render cost, and what does each assert?

**Limits.**
- Subagents received only the brief and their own question. Every search and read excluded `_bmad-output/` and `_bmad/`, which hold plans, tickets and bench results.
- "Unverified reading" marks a statement about how Vue 3.5.40 behaves at runtime. It is a researcher's reading of the framework, not code in this repository.
- The parenthesis in a citation holds the nearest named symbol or token, not always the enclosing definition.
- The commit messages cited in section 5 were written by an AI agent; this document did not re-run their figures.
- Not read: `tests/frontend/support/setup.ts` in full, `e2e/specs/proofread-underline.e2e.mjs`, `src-tauri/tests/segment_contract.rs` past line 4640.
- No ADR files exist in the repository; architectural decisions live in the architecture spine.

## 1. What steps does a `beforeinput` or `input` event in a target-column cell go through?

**Wiring.**
- The listeners sit on the column, not on each cell: `div.col.col-tgt` binds `@beforeinput="onBeforeInput"`, `@compositionstart/end`, `@input="onEditInput"` and `@keydown="onEditKeydown"`, and events bubble up from the cells — src/panels/GridBlock.vue:271-281 (onBeforeInput) @ 99f3fc1.
- Each target cell has a static `contenteditable="true"`, `data-col="tgt"`, `:data-segment-id="s.id"`, and renders `{{ s.target_text }}`, the text as loaded — src/panels/GridBlock.vue:322-344 (target_text) @ 99f3fc1.
- The handlers reach `GridBlock` through `inject(GRID_BLOCK_CONTEXT)`, which throws `'GridBlock needs the grid block context'` when missing — src/panels/GridBlock.vue:25-55 (GRID_BLOCK_CONTEXT) @ 99f3fc1. The context is built and provided in src/panels/GridPanel.vue:1816-1845 (gridBlockContext) @ 99f3fc1.

**Before the input events.** A window keydown listener in the capture phase runs first and does not call `preventDefault` for a plain typed character, so `beforeinput` and `input` still fire — src/commands/keys.ts:510-545 (handle) @ 99f3fc1. Section 4, step 1, lists its steps.

**`onBeforeInput`** — src/panels/GridPanel.vue:1435-1561 (onBeforeInput) @ 99f3fc1:
- `targetCellOf` is `closest('[data-col="tgt"]')` (`:746-749`); no cell ⇒ return (`:1437`).
- When not composing, it calls `unwrapProofread(cell, mergeText)` with `mergeText` true only for `historyUndo`/`historyRedo` (`:1438-1440`). That replaces proofread spans by their children and restores the selection — src/panels/proofreadHighlight.ts:71-78 (unwrapProofread) @ 99f3fc1, src/panels/proofreadHighlight.ts:51-65 (keepingSelection) @ 99f3fc1.
- `insertParagraph`: `preventDefault()`, a reentrancy latch `insertingLineBreak` (`:1468`), `document.execCommand('insertLineBreak')` (`:1477-1482`), a warning and repaint when it returns false (`:1483-1488`); no `reportEdit` — the nested `insertLineBreak` fires its own `input` (`:1489-1492`).
- `insertLineBreak` returns to the engine (`:1494`). For any inputType other than paste, drop or replacement text, the handler returns early (`:1497-1498`): `insertText` and deletions are left to the engine and written on `input`.
- Paste, drop, replacement: `preventDefault()` (`:1504`), text flattened (`:1505`, `:1524`), three rejecting checks — empty text (`:1528-1531`), no readable selection (`:1536-1540`), range outside the cell (`:1542-1549`) — then a Range insert (`:1551-1557`) and `reportEdit(cell)` by hand (`:1560`).

**`onEditInput`** calls `reportEdit(cell)` for any target cell and never reads `event.data` — src/panels/GridPanel.vue:1712-1715 (onEditInput) @ 99f3fc1.

**`reportEdit`** is the only point where DOM text enters state: `segmentIdOf` reads `data-segment-id` (null ⇒ return at `:1701`), then `noteEditorEdit(id, cell.textContent)`, then `clearProofreadFor(id)` — src/panels/GridPanel.vue:1699-1704 (reportEdit) @ 99f3fc1.

**`noteEditorEdit`** — src/panels/editorPanelState.ts:758-814 (noteEditorEdit) @ 99f3fc1, in order:
1. While a chapter switch is in progress (`dangChuyenChuong`), it logs and drops the edit (`:779-784`).
2. `editedText.value.set(id, text)` in place, with no trigger (`:791`); `editedText` is a `shallowRef` — src/panels/editorPanelState.ts:682 (editedText) @ 99f3fc1.
3. `editedShape` updated in place; `triggerRef(editedShape)` only when `textShapeOf` changed — src/panels/editorPanelState.ts:786-793 (triggerRef) @ 99f3fc1.
4. `dropTmFilled(id)` replaces `tmFilledSegmentIds` only when the id was in it — src/panels/editorPanelState.ts:151-161 (dropTmFilled) @ 99f3fc1.
5. `flush.markChanged(id, text, Date.now())` (`:796`) does `dirty.set` and `schedule.onChange(now)` — src/panels/editorFlush.ts:132-135 (markChanged) @ 99f3fc1 — which sets `due = min(now + idleMs, firstChangeAt + hardCapMs)` — src/layout/writeSchedule.ts:91-103 (onChange) @ 99f3fc1.
6. `armFlushTimer()` clears and re-arms one `setTimeout` — src/panels/editorPanelState.ts:738-749 (armFlushTimer) @ 99f3fc1.
7. `datThongBao({})` sets `confirmNotice`, `regroupNotice`, `navNotice`, `splitChapterNotice` to `null` — src/panels/editorPanelState.ts:2678-2690 (datThongBao) @ 99f3fc1.

**`clearProofreadFor`** returns unless the id is in `runSegmentIds`; otherwise it drops reverts, resets a single-segment scan in progress, or replaces `results` with a copy minus the id — src/proofreadState.ts:249-271 (clearProofreadFor) @ 99f3fc1.

Two paths feed `reportEdit`: engine-handled edits arrive through `onEditInput`; paste, drop and replacement arrive from `onBeforeInput` after `preventDefault()`. Both read `cell.textContent`.

Not found: any reactive write of the typed text into `segments`; cells keep `s.target_text`.

## 2. What reads the state the input path writes?

**`editedText`** never triggers: it is written in place, and a comment there says "not reactive per keystroke, so read it imperatively" — src/panels/editorPanelState.ts:679-684 (editorEditedText) @ 99f3fc1. Its readers are imperative:
- `restoreEditedText`, run by a watch whose sources are only `editorSegments` and `gridRoot` — src/panels/GridPanel.vue:1168-1181 (restoreEditedText) @ 99f3fc1.
- `danhSachDieuHuong`, called from `goToNextUntranslated`, `goToNextSegment`, `goToPrevSegment` — src/panels/editorPanelState.ts:1744-1746 (danhSachDieuHuong) @ 99f3fc1.

**`editedShape`** triggers only when a row's shape (empty / whitespace / text) changes (section 1, step 3). Reactive readers:
- the computed `gridBlockViews` (`:617`) — src/panels/GridPanel.vue:615-643 (gridBlockViews) @ 99f3fc1;
- the target-column `v-memo` (`:317`) and the `empty` class (`:328`) — src/panels/GridBlock.vue:311-328 (editorEditedShape) @ 99f3fc1.

**`tmFilledSegmentIds`** triggers only when the id was TM-filled; its reader is `gridBlockViews` (`:618`) — src/panels/GridPanel.vue:615-643 (gridBlockViews) @ 99f3fc1.

**The four notices** trigger only when a notice held a value. Assigning null over null does not trigger (unverified reading: Vue's ref setter checks `hasChanged`). Readers are StatusBar computeds — src/StatusBar.vue:308-312 (confirmNoticeKey) @ 99f3fc1, src/StatusBar.vue:361-371 (regroupNoticeText) @ 99f3fc1, src/StatusBar.vue:405-412 (splitChapterNoticeText) @ 99f3fc1, src/StatusBar.vue:459-463 (navNoticeKey) @ 99f3fc1 — rendered as one `v-if` chain ending in the saved stamp — src/StatusBar.vue:534-596 (secondsSinceSave) @ 99f3fc1.

**`lastSavedAt` and `flushError`** are written by `flushEditorNow` (`:873`, `:877`, `:879`), not by the keystroke — src/panels/editorPanelState.ts:835-880 (flushEditorNow) @ 99f3fc1. Readers: src/StatusBar.vue:279-283 (secondsSinceSave) @ 99f3fc1 and its 1 s ticker src/StatusBar.vue:85-94 (onMounted) @ 99f3fc1; src/panels/GridPanel.vue:1753 (flushErrorSegmentIds) @ 99f3fc1, src/panels/GridPanel.vue:1764-1785 (rowErrorLabelById) @ 99f3fc1, read in the block at `:316` and `:327` of src/panels/GridBlock.vue:311-328 (rowErrorTextById) @ 99f3fc1.

**The flush pending set** is not reactive: a plain `new Map()` inside `createEditorFlush`, and the file imports nothing from Vue — src/panels/editorFlush.ts:129 (createEditorFlush) @ 99f3fc1. It is read imperatively through `flush.*` at `:740`, `:845`, `:849`, `:853`, `:878`, `:943-948` and `:999` of src/panels/editorPanelState.ts:618 (flush) @ 99f3fc1; the callers that reach it are listed at the end of this section.

**Proofread `results`**, replaced by `clearProofreadFor` (`:262-264` of src/proofreadState.ts:249-271 (clearProofreadFor) @ 99f3fc1). Reactive readers:
- the watch that repaints — src/panels/GridPanel.vue:1214 (repaintProofread) @ 99f3fc1;
- src/StatusBar.vue:201-212 (proofreadCountsText) @ 99f3fc1;
- computeds in the proofread module — src/proofreadState.ts:91-105 (proofreadFindingCount) @ 99f3fc1, src/proofreadState.ts:350-378 (proofreadCurrent) @ 99f3fc1.

**Callers that reach the flush pending set**, through `flushEditorNow` or `flushEditorBeforeDiscreteWrite`, each on a discrete action rather than per keystroke:
- src/main.ts:1242 (wireExitFlush) @ 99f3fc1
- src/modes/libraryImport.ts:407 (beginSubmit) @ 99f3fc1
- src/modes/libraryChapters.ts:313 (openWorkById) @ 99f3fc1, src/modes/libraryChapters.ts:484 (beginChapterReorg) @ 99f3fc1
- src/proofreadState.ts:182 (runProofread) @ 99f3fc1, src/proofreadState.ts:442 (acceptProofreadFinding) @ 99f3fc1, src/proofreadState.ts:497 (revertProofreadFinding) @ 99f3fc1, src/proofreadState.ts:588 (ignoreProofreadFinding) @ 99f3fc1
- src/reviewModeState.ts:370 (runAccept) @ 99f3fc1
- src/panels/segmentHistoryState.ts:318 (flushEditorBeforeDiscreteWrite) @ 99f3fc1
- within the state module at `:286`, `:377`, `:441`, `:497`, `:571`, `:747`, `:1496`, `:1988`, `:2203`, `:2307`, `:2923` of src/panels/editorPanelState.ts:835 (flushEditorNow) @ 99f3fc1; `:747` is the flush timer itself.

Not found: any `computed`, `watch` or template that reads `editorEditedText`; any Vue file other than `GridBlock.vue` and `GridPanel.vue` importing `editorEditedShape`.

## 3. How does `GridBlock` decide to re-render, and what does a re-render produce?

A `GridBlock` re-renders when one of its five props changes or when a reactive value its own render read changes.

**Props.** `<GridBlock v-for="block in gridBlockViews" :key="block.index">` sits inside the `PanelFrame` slot and passes `block`, `caret-id`, `anchor-id`, `strip-row` and `has-strip-track` — src/panels/GridPanel.vue:1918-1926 (gridBlockViews) @ 99f3fc1. The helpers: src/panels/GridPanel.vue:645-648 (stripRowIn) @ 99f3fc1, src/panels/GridPanel.vue:650-652 (anchorIdIn) @ 99f3fc1, src/panels/GridPanel.vue:659-662 (caretIdIn) @ 99f3fc1, src/panels/GridPanel.vue:664-667 (stripBlockIndex) @ 99f3fc1. Blocks are 200 rows — src/panels/gridBlocks.ts:1-11 (GRID_BLOCK_ROWS) @ 99f3fc1; a view is `{index, start, segments, rules}` — src/panels/gridBlockContext.ts:15-20 (GridBlockView) @ 99f3fc1.

**Block identity** — src/panels/GridPanel.vue:594-643 (idsWithChangedRule) @ 99f3fc1. `gridBlockViews` reads `editorEditedShape.value` (`:617`) and `editorTmFilledSegmentIds.value` (`:618`). `idsWithChangedRule` (`:599-613`) diffs them against `rulesBasis` (`:594-597`), a copy taken with `new Map(shapes)` (`:641`) because `editedShape` is mutated in place. A block keeps its previous view when start, length and every segment's identity match and no id changed (`:621-632`); otherwise `restRulesOf` (src/panels/GridPanel.vue:278-299 (restRulesOf) @ 99f3fc1) rebuilds its rules, and `sameRules` keeps the old object when the rules are equal (`:635-637`). The computed always returns a fresh array (`:620`, `:642`). `ruleOf` puts the caret on top: `props.caretId === id ? 'primary' : block.rules.get(id) ?? 'none'` — src/panels/GridBlock.vue:74-76 (ruleOf) @ 99f3fc1.

**Reactive reads inside a block.** Module state imported directly (`editorEditedShape`, `concordanceSourceHighlight`, `viewMode`, `editorAssetsDir`, `editorChapterAssets`) — src/panels/GridBlock.vue:8-15 (editorEditedShape) @ 99f3fc1 — and context refs such as `selectedRowClassById`, `unlinkedIds`, `rowErrorLabelById`, `rowErrorTextById`, `showHanViet`, `sourceTokenClass` — src/panels/GridBlock.vue:25-55 (GRID_BLOCK_CONTEXT) @ 99f3fc1.

**`v-memo` per column**, per row of `block.segments`:

| Column | Citation | Memo | Read in the row but not in the memo |
|---|---|---|---|
| rule | src/panels/GridBlock.vue:110-122 (ruleClassForRow) @ 99f3fc1 | `[s, ruleClassForRow(s.id), selectedRowClassById.get(s.id), s.id === anchorId]` | `unlinkedIds.has(s.id)` (`:117`) |
| number | src/panels/GridBlock.vue:132-142 (selectedRowClassById) @ 99f3fc1 | `[s, block.start + j, selectedRowClassById.get(s.id), s.id === anchorId]` | — |
| source | src/panels/GridBlock.vue:147-262 (cutOffsetsOf) @ 99f3fc1 | `[s, selectedRowClassById.get(s.id), s.id === anchorId, cutOffsetsOf(s.id), glossaryTermsOf(s.id), concordanceHighlightOf(s.id), trailingImagesOf(s.id), chapterAssetsOf(s.id), editorAssetsDir, showHanViet, viewMode]` | `cutCountOf` (`:174`), `leadingImagesOf` (`:189`), `sourcePieceInfoOf` (`:223`) |
| target | src/panels/GridBlock.vue:308-346 (editorEditedShape) @ 99f3fc1 | `[s, block.start + j, selectedRowClassById.get(s.id), s.id === anchorId, rowErrorTextById.get(s.id), editorEditedShape.get(s.id), ruleOf(s.id)]` | `rowErrorLabelById.has` (`:327`), `unlinkedIds.has` (`:337`), `caretId === s.id` (`:329`) |

A row that misses its memo produces that column's cell, plus a `strip-pad` div on the anchor row. Target cells render `{{ s.target_text }}`; typed text never reaches the template.

**When `triggerRef(editedShape)` fires.** It fires only on an empty / whitespace / text transition — src/panels/editorSegments.ts:178-183 (textShapeOf) @ 99f3fc1. Each block's render reads `editorEditedShape.get(s.id)` for every row in the target-column memo, with no `v-if` or `v-memo` guard on the column, the block root or the `<GridBlock>` `v-for` (`:317` of src/panels/GridBlock.vue:308-346 (editorEditedShape) @ 99f3fc1). So every mounted `GridBlock` re-runs its render function and evaluates the memo arrays of its ~200 rows × 4 columns. In the target column only the changed row misses, plus rows whose `ruleOf` changed; in the rule column only rows whose `ruleClassForRow` changed miss; the number and source columns all hit. `gridBlockViews` also recomputes, and only a block with a new view object receives a new `block` prop. Unverified reading: that a read during render subscribes each block's render effect, and that memo expressions are evaluated on every render even on a cache hit.

Outside the memo, at block level: the root's `gridTemplateRows` — src/panels/GridBlock.vue:99-102 (gridTemplateRows) @ 99f3fc1 — `col-src`'s `:class="sourceTokenClass"`, and the strip track (`TmFuzzyStrip`, `GlossaryConfirmStrip`, `ProofreaderStrip`) only where `hasStripTrack` — src/panels/GridBlock.vue:350-362 (hasStripTrack) @ 99f3fc1.

Not found: `v-once`; `v-memo` on the `<GridBlock>` `v-for`; `shallowReactive` or per-row splitting of the shape state; `markRaw` around block views.

## 4. What other work runs around one keystroke?

Event order for one ordinary character in a target cell:

1. **keydown, window capture** — a listener on `window` runs on every keydown, before any grid handler — src/commands/keys.ts:601-607 (attachKeymap) @ 99f3fc1. It first evaluates the gate passed from boot, an OR of about 20 overlay and mode refs — src/main.ts:1118-1152 (attachKeyboard) @ 99f3fc1. Then src/commands/keys.ts:510-545 (handle) @ 99f3fc1:
   - `isComposing` returns false (`:514`).
   - `Mod+Z` in a typing zone returns false, leaving undo to the browser (`:522`).
   - It loops over every compiled chord (`:525`) and skips a chord unless `code` and modifiers match (`:526`).
   - A matching chord with no primary modifier in a typing zone returns false (`:527`).
   - Otherwise it calls `preventDefault` and dispatches (`:530-540`).
   - A typing zone is `isContentEditable === true` — src/commands/keys.ts:439-447 (isTypingZone) @ 99f3fc1.
2. **keydown, `.col-tgt` bubble** — src/panels/GridPanel.vue:1608-1696 (onEditKeydown) @ 99f3fc1: returns when composing (`:1614`); otherwise, for every non-composing keydown in a target cell — arrows, Shift and Cmd included — `unwrapProofread(typed, true)` and `scheduleRepaintProofread()` (`:1615-1620`). `Escape` dispatches `editor.clear_source_cuts` (`:1644-1647`); `Backspace` at cell start, not repeating, dispatches `editor.merge_segments` (`:1652-1695`).
3. **beforeinput** — `unwrapProofread` a second time in the same keystroke, `mergeText` only for undo/redo and with no repaint (`:1439` of src/panels/GridPanel.vue:1435-1561 (onBeforeInput) @ 99f3fc1). A merge from an input event detaches the text node WebKit is inserting into — src/panels/proofreadHighlight.ts:67-71 (unwrapProofread) @ 99f3fc1.
4. **input** — `reportEdit` (section 1).
5. **selectionchange** — two `document` listeners — src/panels/GridPanel.vue:1325-1332 (onSelectionChange) @ 99f3fc1. A comment in `GridPanel.vue` says `selectionchange` fires per typed character in `contenteditable` — src/panels/GridPanel.vue:1279 (onSelectionChange) @ 99f3fc1.
   - `onSelectionChange` calls `setEditorCaret(...)` when the anchor is in the target column — src/panels/GridPanel.vue:1234-1240 (onSelectionChange) @ 99f3fc1.
   - `onSourceSelectionChange` always writes `sourceSelecting.value` (`:1291`), then returns outside the source column (`:1311`) — src/panels/GridPanel.vue:1289-1323 (onSourceSelectionChange) @ 99f3fc1.
6. **keyup** — returns unless the key is `Shift` — src/panels/selectionContract.ts:390-396 (attachSelectionWatcher) @ 99f3fc1.

**Caret.** `setEditorCaret` assigns `caretSegmentId` and `lastCaretSegmentId`, flushes only when the caret leaves one row for another (`:286`), and — whenever `id` is not null and a chapter is open — calls `positionFlush.markMoved(...)` and `armPositionFlushTimer()` (`:292-295`) — src/panels/editorPanelState.ts:281-296 (setEditorCaret) @ 99f3fc1. So every keystroke clears and re-arms the position timer: idle 500 ms, hard cap 5,000 ms — src/panels/positionFlush.ts:35-42 (POSITION_IDLE_MS) @ 99f3fc1. When it fires, the position flush writes over IPC — src/panels/editorPanelState.ts:652-670 (flushChapterPositionNow) @ 99f3fc1. The caret readers outside the grid (strips, proofread scope, status-bar position) re-evaluate only when `editorCaretSegmentId` changes. Unverified reading: assigning the same value to a ref does not trigger them.

**Proofread repaint.** `scheduleRepaintProofread` is a `setTimeout(…, 0)` debounce — src/panels/GridPanel.vue:1197-1203 (scheduleRepaintProofread) @ 99f3fc1. `repaintProofread` exits when there is no grid root, while composing, or when there are no results and nothing painted; otherwise it repaints the whole grid root — src/panels/GridPanel.vue:1184-1190 (repaintProofread) @ 99f3fc1. `repaintProofreadHost` first unwraps: `unwrapAllProofread` runs a `querySelectorAll` over the proofread spans (`.proofread-error, .proofread-meaning`), takes each span's `closest('[data-segment-id]')`, and unwraps only the cells that hold spans — src/panels/proofreadHighlight.ts:102-109 (unwrapAllProofread) @ 99f3fc1. It then runs one `querySelector` and `wrapProofread` per result entry, skipping an entry with no target cell — src/panels/proofreadHighlight.ts:117-128 (repaintProofreadHost) @ 99f3fc1. `wrapProofread` skips a cell whose `textContent` differs from the scanned text — src/panels/proofreadHighlight.ts:84-100 (wrapProofread) @ 99f3fc1. The watch at src/panels/GridPanel.vue:1214 (repaintProofread) @ 99f3fc1 also repaints when `clearProofreadFor` replaces `results`.

**Rule markers** change only through the `editedShape` trigger or a `tmFilled` change (section 3); the typed row stays `primary` while the caret is in it — src/panels/GridBlock.vue:74-76 (ruleOf) @ 99f3fc1.

**Status bar.** The only per-keystroke write that reaches it is `datThongBao({})` (section 1, step 7); the saved-stamp ticker is a 1 s `setInterval` independent of keys — src/StatusBar.vue:85-94 (onMounted) @ 99f3fc1.

Not found: any other `document` or `window` listener for `keydown`, `selectionchange`, `input`, `beforeinput` or `keyup` in `src` besides those above.

## 5. Why is the edited-text state shaped this way, and why the block split?

Commits cited in this section:
- 6a4e6b8 "feat: add frontend tests for editor typing zone and status bar"
- ca33072 "feat(story-2.5b): lưới hai cột — và chỗ hỏng thật không nằm ở lưới, nó nằm ở ai giành tiêu điểm"
- 043f3bc "perf(editor): 2-15 dừng ở blocked vì bỏ lượt dựng lại cả lưới mỗi phím chỉ giảm ~20% ở 9.850 câu"
- 43729b3 "perf(grid): 2-14 dừng ở blocked vì dời con trỏ ở 9.850 câu vẫn ~600–740 ms trên bản debug"
- e6c190b "docs(research): lưới chia khối ~200 hàng là bố cục duy nhất giữ ≤ 100 ms mỗi phím trên cả ba engine"
- f19743a "docs(spine): lưới đối chiếu chia khối ~200 hàng thay quyết định #1 của 2-5b, ghi ở hàng Deferred chứ không thành AD"
- 30049b3 "chore(bench): 2-16 bản thử chia khối 200 hàng hết xếp hàng phím nhưng dời con trỏ và bố cục đầu chậm hơn"
- 742096c "feat(grid): 2-16 lưới chia khối 200 hàng, mỗi khối một component `GridBlock`"
- f62012f "perf(grid): 2-16 một lượt `Mod+Enter` làm cả 50 khối render lại vì bốn giá trị dùng chung đổi danh tính dù nội dung không đổi"
- 8260a92 "perf(grid): 2-16 sau một lượt `Mod+Enter` vẫn còn bốn việc duyệt cả Chương"

**The edited text.**
- The whole-`Map`-per-edit form was introduced in 6a4e6b8 (2026-08-13): `editedText = ref(new Map())`, and `noteEditorEdit` copied the Map and reassigned it; the grid template then read `editedText.get(id) ?? s.target_text`. The commit gives no reason for copying.
- 043f3bc (2026-10-23) removed it. Its message says each `input` replaced the whole Map, and because the grid body is the `PanelFrame` slot, each key re-ran 4 columns × n rows plus `restRuleById` over n sentences. It made `editedText` a `shallowRef` written in place and added `editedShape` with `triggerRef` only on a shape change. The message records the reduction as about 20 % at 9,850 sentences (157–163 s against 188–193 s for a 300-key release run), about 0.5 s per key left unattributed, and the 122-sentence case already at or under 55 ms before the change.
- The code carries the reason today: "Written in place without triggering: a reactive read here in the grid template would re-render the whole grid slot on every keystroke" — src/panels/editorPanelState.ts:672-688 (editedShape) @ 99f3fc1.
- Why the template never renders typed text: the comment block "DOM SỞ HỮU VĂN BẢN" — src/panels/GridPanel.vue:1144-1158 (target_text) @ 99f3fc1 — first written in `EditorPanel.vue` at 6a4e6b8, then moved and condensed into `GridPanel.vue` by ca33072 (2026-08-15) when the two panels merged. It records two defects — the caret jumping to offset 0 when Vue rewrote `textContent`, and translated text vanishing through one shared `frozenText` — with the common cause "hai chủ sở hữu cho một text node", and names the DOM as owner.

**The block split.**
- 43729b3 (2026-10-23): a caret move at 9,850 segments cost about 600–740 ms in a debug build, about 390 ms of it WebKit relayout of the whole `subgrid` when a `v-if` inserted a `div.rule`, about 225 ms the `PanelFrame` slot. It made the rule always present, put `v-memo` on the four columns, and made every rule 3 px.
- e6c190b: on a synthetic 9,850-row page with no Vue, the whole-grid `subgrid` cost 468 ms per key on the OS WKWebView, 998 ms on WebKit 627 and 172 ms on Chromium; 200-row blocks gave 10 / 15 / 3 ms, in an order the message does not tie to engines explicitly. The app loads WebKit.framework 20621.3.11.11.3 on macOS 15.8.2.
- f19743a: the block split replaces the one chapter-wide `subgrid` and is recorded in the spine's Deferred row as a presentation decision, not an AD.
- 30049b3 (bench prototype, not in `src/`): with the 200-row split, keystrokes no longer queued at 9,850 sentences (last key after 55–84 ms), while caret moves and first layout got slower.
- 742096c introduced `GRID_BLOCK_ROWS = 200` (src/panels/gridBlocks.ts:1-11 (GRID_BLOCK_ROWS) @ 99f3fc1), `GridBlock.vue`, `gridBlockContext.ts`, and one selection surface per role on the grid root, after a trial with surfaces on block 0 only left later blocks unrecognised. src/panels/README.md:253-257 (GridBlock) @ 99f3fc1 restates the split.
- f62012f: one `Mod+Enter` re-rendered all 50 blocks because four shared values changed identity; rules moved into `block.rules` per block and block objects are kept when contents are equal. 8260a92 removed four chapter-wide passes after a confirm and added `gridConfirmWorkCost`. Together they wrote src/panels/GridPanel.vue:594-643 (idsWithChangedRule) @ 99f3fc1.
- Why 200: it is the figure e6c190b measured and 30049b3 used; the gate was "≤ 100 ms mỗi phím".
- Not found: a commit outside the excluded paths that compares another block size; a change to `GRID_BLOCK_ROWS` after 742096c; a comment on the size in `gridBlocks.ts`. Bench results under the excluded paths were not read, so a comparison there is not ruled out.

## 6. Which tests cover typing in the grid and its render cost?

Across `tests/`, `e2e/` and `src-tauri/tests/`, no test counts how many `GridBlock`s re-render on a keystroke, and no test measures wall-clock time for typing (searches for `updated(`, `mixins`, `renderTracked`, `onRenderTriggered`, `performance.now` and `Date.now` at `99f3fc1`).

**Config.** vitest runs on happy-dom and picks up only `tests/frontend/**/*.test.ts`; its header says happy-dom is not WebKit and real-engine claims belong to the bench page or e2e — vitest.config.ts:43-60 (defineConfig) @ 99f3fc1.

**Render-cost counters (vitest).**
- The typing-cost helper sets `cell.textContent` and dispatches a plain `input` — tests/frontend/gridTypingCost.test.ts:108-112 (type) @ 99f3fc1. The file counts renders of a stubbed `PanelFrame` — tests/frontend/gridTypingCost.test.ts:58-67 (frameRenders) @ 99f3fc1 — and calls to `segmentRuleInputOf` — tests/frontend/gridTypingCost.test.ts:14-23 (segmentRuleInputOf) @ 99f3fc1 — not `GridBlock` renders. At 20 and 200 rows, three keystrokes give 0 frame renders and 0 rule-input calls — tests/frontend/gridTypingCost.test.ts:115-128 (frameRenders) @ 99f3fc1.
- The only `GridBlock` render counter records `$props.block.index` from a global mixin's `updated()` — tests/frontend/gridBlockRenderCost.test.ts:74-82 (updated) @ 99f3fc1 — at 1,050 segments; its cases drive caret moves, anchor moves and confirm, none types — tests/frontend/gridBlockRenderCost.test.ts:99-165 (settle) @ 99f3fc1.
- Others: tests/frontend/gridCaretMoveCost.test.ts:111-123 (ruleInputCalls) @ 99f3fc1 (caret moves), tests/frontend/gridConfirmWorkCost.test.ts:149-187 (gridQueries) @ 99f3fc1 (confirm at 1,050 and 2,100), tests/frontend/gridRuleStable.test.ts:43-100 (MutationObserver) @ 99f3fc1, tests/frontend/gridBlocks.test.ts:97-121 (gridBlocksOf) @ 99f3fc1, tests/frontend/gridRestoreEditedText.test.ts:48-74 (querySelectorAll) @ 99f3fc1.

**Typing behaviour (vitest).**
- An ordinary sentence is `textContent` plus `new Event('input')` only — tests/frontend/editorTypingZone.test.ts:285-297 (editorEditedText) @ 99f3fc1.
- Enter, paste and IME cases dispatch `keydown`/`beforeinput` — tests/frontend/editorTypingZone.test.ts:200-250 (insertParagraph) @ 99f3fc1, tests/frontend/editorTypingZone.test.ts:299-343 (insertFromPaste) @ 99f3fc1, tests/frontend/editorTypingZone.test.ts:383-467 (compositionstart) @ 99f3fc1.
- `appendData` three times keeps the same text node — tests/frontend/editorTypingZone.test.ts:469-505 (appendData) @ 99f3fc1.
- The pure flush module is tested with no DOM — tests/frontend/editorFlush.test.ts:54-131 (createEditorFlush) @ 99f3fc1.
- Handler order is checked by regex over the source of `GridPanel.vue` and `GridBlock.vue` — tests/frontend/proofreadWiring.test.ts:174-196 (clearProofreadFor) @ 99f3fc1.

In vitest no ordinary typed character is preceded by a `keydown` or a `beforeinput insertText`.

**e2e (real WKWebView).**
- `browser.keys()` fires only `keydown` and no text lands, so the spec types with `execCommand('insertText')`, which fires `beforeinput` then `input` — e2e/specs/editor-typing-flush.e2e.mjs:38-55 (execCommand) @ 99f3fc1.
- The case that types has no skip marker; a comment says it is meant to be red — e2e/specs/editor-typing-flush.e2e.mjs:161-182 (insertText) @ 99f3fc1. It asserts text reaches disk after a flush and the saved status — e2e/specs/editor-typing-flush.e2e.mjs:198-218 (target_text) @ 99f3fc1.
- Six blocks at 1,200 sentences with synthetic chords and no typing — e2e/specs/grid-blocks.e2e.mjs:80-176 (KeyboardEvent) @ 99f3fc1.
- No e2e case counts renders or measures timing.

**cargo.**
- The checkpoint idle gate stays longer than `EDITOR_IDLE_MS` — src-tauri/tests/flush_cadence_contract.rs:126-147 (the_checkpoint_idle_gate_stays_strictly_longer_than_the_editor_flush_cadence) @ 99f3fc1.
- Typed text round-trips through `save_segment_targets` and the load command — src-tauri/tests/segment_contract.rs:4566 (typed_text_round_trips_through_the_flush_and_the_load_command) @ 99f3fc1.

## Where the answers meet

- `reportEdit` — questions 1, 2, 4: the only point where DOM text enters state; it calls `noteEditorEdit` then `clearProofreadFor`, and its one rejecting check is `segmentIdOf` returning null — src/panels/GridPanel.vue:1699-1704 (reportEdit) @ 99f3fc1.
- `noteEditorEdit` — questions 1, 2, 3, 5: its `triggerRef(editedShape)` is the only write on the typing path that every mounted `GridBlock` reads; it drops the edit while `dangChuyenChuong` is set — src/panels/editorPanelState.ts:758-814 (noteEditorEdit) @ 99f3fc1.
- `gridBlockViews` — questions 2, 3, 5: reads `editorEditedShape` and `editorTmFilledSegmentIds` and decides block identity — src/panels/GridPanel.vue:615-643 (gridBlockViews) @ 99f3fc1.
- `unwrapProofread` — questions 1, 4: called from keydown and from beforeinput on the same keystroke — src/panels/proofreadHighlight.ts:71-78 (unwrapProofread) @ 99f3fc1.

## Prior documents

- `research-grid-render-on-caret-move` (codebase, before the block split): render on caret move; not continued here.
- `research-go-cham-luoi-subgrid-webkit` (technical, web): the web pass that follows this document deepens it.
