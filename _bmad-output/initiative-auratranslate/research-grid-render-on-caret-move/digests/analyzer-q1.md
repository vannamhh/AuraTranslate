DIGEST: caret move in GridPanel, tree A (cc60020) and tree B (b2874aa). Read through git; Vue runtime read on disk. Strip templates and `saveSegmentTargets` not read (see "Not done").

NOTE: in zsh `$A:src/...` is parsed as a modifier; I used literal shas.

## 1. Reactive sources written in the same tick (same in A and B)

Mouse path:
- `onCellMouseDown` (A GridPanel.vue:851-855 @ cc60020; B :884 @ b2874aa) calls `setEditorCaret(id)` for the cell under the pointer. The mouse path does not write `caretPlacement`.
- `onSelectionChange` (A :1143-1150; B :1175) also calls `setEditorCaret(id)`. It is registered on `document` `selectionchange` (A :1235). It fires later and writes the same id again.

`setEditorCaret` (editorPanelState.ts:281-296 @ cc60020), synchronous writes:
- `caretSegmentId.value = id` (ref, :148) and `lastCaretSegmentId.value = id` (ref, :149, only when `id !== null`).
- On A→B (both non-null, different) it calls `void flushEditorNow()` (:286). `flushEditorNow` (:835-902) writes no reactive state before its first `await`: it returns `'clean'` if `!flush.isDirty()`, otherwise it runs `clearFlushTimer` and starts the `saveSegmentTargets` IPC. Its reactive writes (`flushError`, `lastSavedAt`) happen after the await, i.e. a later tick.
- `positionFlush.markMoved` plus `armPositionFlushTimer` (:292-295) are plain closure state plus a `setTimeout` (`createPositionFlush` is plain variables, positionFlush.ts:81-111). They write no reactive state.

`goToNextSegment` (:1787-1789) calls `doiConTroToi` (:1770-1775), which calls `setEditorCaret(id)` and then `caretPlacement.value = id` (shallowRef, :1309). This is the only extra write versus the mouse path.

Writes in later steps of the same flush cycle:
- `watch(editorCaretSegmentId)` in `TmFuzzyStrip.vue:30` calls `syncTmFuzzyStrip` (tmFuzzyStripState.ts:133-149). On a segment change it calls `clearShown` (:85-92), which writes `shown`, `aimed`, `scanError`, `acceptError`, `pendingAccept` and `expandedRows`. `shown` is set again only after a 150 ms debounce plus IPC (:118).
- `ProofreaderStrip.vue:40` calls `syncProofreadStripHidden` (proofreadState.ts:290-292), which writes `hiddenForSegmentId` only when it differs from the new id.
- `GlossaryConfirmStrip.vue:60-66` watches `[editorCaretSegmentId, editorSegments, glossaryMarks]` and calls `syncGlossaryConfirmStripTarget` (glossaryConfirmStripState.ts:345-356). It writes `undoEntry` and calls `applyTarget`; I did not trace what `applyTarget` writes.
- `AiTranslationPanel.vue:130` has `watch(editorCaretSegmentId, clearAiPromptAssembleError)`.
- On the `goToNext` path, `watch(editorCaretPlacement, …, {flush:'post'})` (A :1255-1275; B :1287) runs after render. It calls `clearEditorCaretPlacement()` (writes `caretPlacement` back to null) and then `target.focus()` and `setCaret`. B swaps the lookup for `targetCellById` (gridRoot `querySelector`).

## 2. Which render effects depend on those sources

**LOAD-BEARING 1: the grid markup is rendered by PanelFrame's render effect, not GridPanel's.**
- GridPanel's template root is `<PanelFrame … :show-status="showFrameStatus">` (A GridPanel.vue:1740; B :1769). All grid markup is its default slot, and PanelFrame renders it with `<slot />` (PanelFrame.vue:174).
- The slot function is compiled inside GridPanel but is executed by `renderSlot` (`path:3267 (renderSlot) @ vue@3.5.40`) during PanelFrame's render.
- `withCtx` (`runtime-core.cjs.js:687-715 (withCtx) @ vue@3.5.40`) only switches `currentRenderingInstance` and block tracking. It does not change the active reactive effect.
- Each component's render runs inside its own `ReactiveEffect` (`runtime-core.cjs.js:6253-6263 (setupRenderEffect) @ vue@3.5.40`; `effect.scheduler = () => queueJob(job)`).
- So every reactive read inside the slot (all four column loops, `v-memo` arrays, `stripAnchorIndex`, `ruleOf`) is tracked by PanelFrame's effect, and a caret change queues PanelFrame's job.
- GridPanel's own render function reads only `showFrameStatus` (A :254-262). That computed has no caret dependency (it reads `loadErrorKey`, `editorPending`, `hasSegments`, `showNoSegments`, …). GridPanel's own render is therefore not queued by a caret move.
- Same for A and B: the slot structure is the same and B only adds a block `v-for` inside it.

**LOAD-BEARING 2: the caret enters the slot render through several reads.**
- `ruleOf(id)` reads `editorCaretSegmentId.value` (A :296-298). It is called per row in the `v-memo` arrays: `ruleClassForRow(s.id)` in the rule column (A :1819) and `ruleOf(s.id)` in the translation column (A :2026). It is also reached through `cellLabelOf` (A :1707-1708) in the `aria-label`.
- Inside the memoized body of the translation cell, `editing: editorCaretSegmentId === s.id` (A :2037) is another read.
- `stripAnchorSegmentId` and `stripAnchorIndex` (A :600-604; B :605-609) read `inlineStripAnchorSegmentId` (inlineStripEligibility.ts:61-73).
  - That computed reads `editorCaretSegmentId` through `eligibleInlineStrips` (:34), `proofreaderStripPlace` (:57) and the `glossary_confirm` / `proofreader` branches (:66, :69).
  - The `tm_fuzzy` branch returns `tmFuzzySegmentId` instead, so with only the TM strip showing the anchor does not follow the caret. It changes when `shown` is cleared at `syncTmFuzzyStrip` (tmFuzzyStripState.ts:139) or set at `:118`.
  - Computed values are compared by primitive value, so the anchor triggers dependents only when the id actually changes.
- Slot-wide uses of the anchor and its index: `s.id === stripAnchorSegmentId` in all four `v-memo` arrays and in the `strip-pad` and `aria-details` bindings; `stripAnchorIndex` in the `.grid` `:style` (A :1808) and in `#strip-track` (A :2062-2063).

**Queued renders (both trees):**
- PanelFrame effect, always on a caret change.
- StatusBar, via the `positionInChapter` / `positionText` computeds (StatusBar.vue:154-160, :168-175; `findIndex` over segments). It re-renders if the position text changed.
- `AiTranslationPanel`: its computeds read the caret at :188, :193, :208, :229, :235 and the watcher at :130. Its render re-runs only if a computed's value changed (e.g. `isRecordStale`, `canAssemble`).
- `proofreadScope.ts:19` and `proofreadStripHidden` (proofreadState.ts:276) also read the caret.
- Strip components (`TmFuzzyStrip`, `GlossaryConfirmStrip`, `ProofreaderStrip`): their watchers run as pre-flush jobs. Their renders are queued only if values they read change (`shown` cleared, `hiddenForSegmentId`, `isVisible`, `eligibleInlineStrips`). `eligibleInlineStrips` builds a new array each time it re-evaluates, so the computed itself changes identity on every caret move. I did not read the strip templates, so I cannot say which of the three actually re-render.
- GridPanel: not queued by the caret (see above), unless some other source it reads changes.

## 3. Whole grid render function or part?

**LOAD-BEARING 3: the whole slot render function re-runs on every caret move, and only the vnode creation for unchanged rows is skipped.**
- There is no per-row component or per-row effect. All rows are `v-for` callbacks in one slot function, executed by one PanelFrame effect.
- Every row's `v-for` callback runs in all four columns (rule, number, source, translation). Each callback evaluates its `v-memo` array expression, e.g. `ruleClassForRow(s.id)`, `selectedRowClassById.get(s.id)`, `s.id === stripAnchorSegmentId`, and for the translation column `rowErrorTextById.get`, `editorEditedShape.get`, `ruleOf`.
- `isMemoSame` (`runtime-core.cjs.js:8620-8635 (isMemoSame) @ vue@3.5.40`) compares the arrays element by element. Where equal, the cached vnode is reused and the row body (the vnode tree, `class`, `aria-label`, `t(...)` etc.) is skipped. Only rows whose memo array changed rebuild their vnodes: the old caret row and the new caret row (via `ruleClassForRow` / `ruleOf`), plus rows whose anchor flag changed.
- On patch, `patch` returns early when `n1 === n2` (`runtime-core.cjs.js:5440 (patch) @ vue@3.5.40`), so reused vnodes are not diffed. The keyed children diff of each column's fragment still walks the whole child array.
- The cost per move is thus O(rows) callbacks and memo comparisons across 4 columns, plus O(rows) keyed diff, with O(1) body rebuilds. This is a reading of the code and Vue source, not a measurement.
- What makes it so: no component boundary or `computed` per row. `ruleOf`, `stripAnchorIndex` and `editorCaretSegmentId` are read directly in the shared slot render, so their change dirties the one effect for the whole slot.

## 4. Differences A vs B on this path

**LOAD-BEARING 4: B does not change which effect re-runs or how much of it. It adds block wrapping inside the same slot.**
- B (GridPanel.vue:1839) wraps the four columns and the strip track in `v-for="block in gridBlockViews"` (200 rows per block). The row loops iterate `block.segments` (B :1852, :1874, :1889, :2052). Everything is still inside the PanelFrame default slot, so the same single PanelFrame effect re-runs all blocks and all rows.
- `gridBlockViews` (B :611-618) depends only on `editorSegments`. On a caret move it is not re-evaluated, and the `block` objects and `block.segments` arrays keep their identity.
- Extra per-move work in B:
  - The `:style` binding of each `.grid-block` calls `stripRowIn(block)` (B :620-623), which reads `stripAnchorIndex`. It is evaluated per block (about 50 at 9,850 rows) and creates a new style object each time. Whether the DOM style actually changes depends on the anchor.
  - `v-if="block.index === stripBlockIndex"` on `#strip-track` (B :2094) depends on `stripBlockIndex` (B :625-628), derived from `stripAnchorIndex`. It changes when the anchor crosses a block boundary, mounting or unmounting that block's strip track together with the three strip components inside it.
  - The old `.grid` `:style` computed from `editorSegments.length + (stripAnchorIndex…)` (A :1808) is removed; the per-block style above replaces it.
  - The `ref` bindings become functions (`block.index === 0 ? setColSrcRef : undefined`, `:ref="setStripTrackRef"`) that write shallowRefs `colSrc`, `colTgt`, `stripTrack` (B :598-603). Same-value writes to a shallowRef do not trigger. Whether Vue invokes the function ref on every patch I did not trace.
- B also replaces `colTgt`-based DOM lookups with `gridRoot` and `targetCellById` (B :614-617 region, :1287 onward): `onSelectionChange` uses `isInTargetColumn` (closest `.col-tgt`); the placement watcher uses `targetCellById`. These are not reactive differences.
- The caret and strip state writes are identical to A, since `editorPanelState.ts`, `inlineStripEligibility.ts` and the strip state files are not in the A→B diff (`git diff --stat -- src`: only GridPanel.vue, gridBlocks.ts, proofreadHighlight.ts).

## Not done / not found
- I did not read the strip components' templates, so I do not know which of them re-render on a caret move.
- I did not read `saveSegmentTargets`, so I assumed it writes no reactive state before its first await (the code up to the await in `flushEditorNow` writes none).
- `applyTarget` (glossary strip) writes were not traced.
- No timing was run: no profile or measurement of the render cost.
- Searches: `git grep editorCaretSegmentId cc60020 -- src` (full reader list above), `git grep syncTmFuzzyStrip|syncProofreadStripHidden`, diff of A→B on `src`.
