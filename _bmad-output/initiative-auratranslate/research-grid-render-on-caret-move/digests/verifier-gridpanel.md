Digest. All seven lines verified. Notes: files opened with `git show <sha>:<path> | awk` (in this shell `cat -n` printed a different file's contents); the tree-A template was compiled with @vue/compiler-sfc 3.5.40 to settle the stable-slot question in [1].

ref=[1] verified. Citation: src/panels/GridPanel.vue:1740-2072 (template) @ cc60020; src/panels/PanelFrame.vue:173-175 (slot) @ cc60020; src/panels/GridPanel.vue:254-264 (showFrameStatus) @ cc60020; runtime-core.cjs.js line 3267 (renderSlot), line 687-715 (withCtx), line 6253-6265 (setupRenderEffect) @ vue@3.5.40.
- PanelFrame's template is `<p v-if="props.showStatus">` plus `<div class="panel-body"><slot /></div>`. `renderSlot` calls `slot(props)` inside the render run by PanelFrame's `ReactiveEffect`. `withCtx` does not pause tracking, so caret reads in the slot are tracked by PanelFrame's effect.
- `showFrameStatus` reads loadErrorKey, editorPending, hasSegments, showNoSegments and showEmptyChapter; none is the caret.
- Nothing in GridPanel's template sits outside the PanelFrame slot except the props `owner`, `status-key` and `:show-status`; no `<template #…>`, no `v-slot`.
- The slot is compiled and stable: the compiled output ends `_: 1 /* STABLE */, … 8 /* PROPS */, ["show-status"]`.

ref=[2] verified. Citation: src/panels/editorPanelState.ts:281-296 (setEditorCaret); :835-902 (flushEditorNow); src/panels/positionFlush.ts:81-115 (createPositionFlush); src/panels/editorPanelState.ts:636-644 (armPositionFlushTimer); :1770-1775 (doiConTroToi), all @ cc60020.
- `setEditorCaret` writes `caretSegmentId.value` and, only when `id !== null`, `lastCaretSegmentId.value`; everything else it touches is non-reactive.
- `flushEditorNow` writes no reactive state before its first await (`editorFlush.ts` and `positionFlush.ts` contain no ref/reactive/shallowRef); `saveSegmentTargets` is an IPC invoke (segment.ts:757-762); `flushError`, `lastSavedAt`, `armFlushTimer` come after an await.
- `doiConTroToi` calls `setEditorCaret(id)` then writes `caretPlacement.value = id` (shallowRef, :1309).

ref=[3] verified. Citation: src/panels/GridPanel.vue:1819, :1841, :1856-1868, :2019-2027; :296-302 (ruleOf, ruleClassForRow), all @ cc60020.
- A caret move re-runs the slot render through PanelFrame's effect; every row's `v-for` callback runs in all four columns and evaluates its `v-memo` array.
- Only the rule list (via `ruleClassForRow` → `ruleOf`) and the target list (`ruleOf(s.id)`) read the caret; number and source lists rebuild only on an anchor-flag change.
- Rows rebuild only when their array changed. The `.col` wrapper divs are rebuilt every render.
- Caveat: `ruleClassForRow` returns a class string, so a row whose rest rule maps to the same class as `'primary'` would not rebuild in the rule column.

ref=[4] verified. Citation: src/panels/GridPanel.vue:611-618 (gridBlockViews); :1839-1843 (grid); :620-623 (stripRowIn), all @ b2874aa.
- `gridBlockViews` reads only `editorSegments.value`; `gridBlocksOf` is pure, `GRID_BLOCK_ROWS = 200`, so 9.850 rows give 50 blocks; not re-evaluated on a caret move.
- The block loop has no v-memo; `git grep v-memo` on B finds only four hits, inside the column row loops (1854, 1876, 1891, 2054). The ~50 `div.grid-block` vnodes and their column wrapper vnodes are created on every render.
- `stripRowIn(block)` reads `stripAnchorIndex.value`, a `findIndex` over `editorSegments`; it is also called in the strip track (:2098-2099), which renders under `v-if="block.index === stripBlockIndex"` (:2093-2105).

ref=[5] verified. Tree A lists match verbatim (rule 4 entries, number 4, source 11, target 7). Tree B differs only in loop variables: `block.segments`, `(s, j)`, `i` → `block.start + j` in number (:1876) and target (:2056); rule list and source tail (:1892-1903) unchanged.

ref=[6] verified. `ruleOf` returns `'primary'` when `editorCaretSegmentId.value === id`, else `restRuleById.value.get(id) ?? 'none'`; `restRuleById` (:281-294) is a computed over editorSegments, editorEditedShape and editorTmFilledSegmentIds with no caret input. Edge case: if the rest rule class equals the `'primary'` class, the rule column memo may not change.

ref=[14] verified. TmFuzzyStrip:30 and ProofreaderStrip:40 are `watch(editorCaretSegmentId, …, { immediate: true })`; GlossaryConfirmStrip:60-66 is `watch([editorCaretSegmentId, editorSegments, glossaryMarks], …, { immediate: true })`. No other mention of the caret in these three files (imports aside), so no direct caret read in their templates or computeds. Each strip has its own render effect.
