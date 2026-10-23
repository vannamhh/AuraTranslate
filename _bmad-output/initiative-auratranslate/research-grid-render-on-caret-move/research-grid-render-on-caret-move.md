---
title: 'Codebase research: grid render on caret move'
type: codebase
ask: |-
  Why the 200-row block prototype of the comparison grid renders 2–3x heavier per caret move than the current grid layout (measured: JS from mousedown to bubble listener 62–122 ms vs 174–356 ms at 9,850 segments), as the factual base for choosing the next step of story 2-16.
ticket: '_bmad-output/initiative-auratranslate/tickets/epic-bien-tap-theo-segment/tickets/story-luoi-doi-chieu-chia-khoi-200-hang-plan.md'
repository: '/Users/hoangnam/LocalSites/addon/AuraTranslate'
commit: 'cc6002023bc7de1df1c4a3ef474fd628c9658802'
branch: 'master'
status: complete
validation: 'normal'
claims: {disputed: 2, verified: 24}
created: '2026-10-23'
updated: '2026-10-23'
---

# Codebase research: grid render on caret move

## Questions

Two trees are read: **A** = `cc60020` (master), **B** = `b2874aa`, a detached commit whose parent is `cc60020` and which holds `blocks-prototype-2.patch` plus `src/panels/gridBlocks.ts` from `_bmad-output/implementation-artifacts/2-16-ban-do/`. It is on no branch, so `git gc` can remove it; applying that patch to `cc60020` and copying that folder's `gridBlocks.ts.txt` to `src/panels/gridBlocks.ts` rebuilds the same tree. The Vue runtime is read from `node_modules` at 3.5.40. Subagents loaded the repository's `AGENTS.md`, so the firewall held only in part.

1. (how) When the editor caret moves from one segment to another in the comparison grid (`src/panels/GridPanel.vue`), which reactive state changes, and which render functions run again as a result? Answer for tree A and for tree B.
2. (how) How does the grid template build its rows — the `v-for` levels, `key`s, `v-memo` and its dependency list, Fragments, `subgrid` — and which reactive values does each row read while rendering? Answer for tree A and for tree B.
3. (how) In the installed Vue 3.5.40 runtime, when a parent component re-renders, what work does Vue do for children that are (a) inline `v-for` vnodes carrying `v-memo`, (b) the same vnodes wrapped in a Fragment or an outer nested `v-for`, (c) child components whose props did not change? And what does `patchKeyedChildren` do when given a new array whose elements are the same as before?
4. (what touches) Which places in `src/` read the caret and strip-anchor state (`setEditorCaret`, `stripAnchorIndex` and the state they write) and are render dependencies of `GridPanel`?
5. (why) Why do the grid's rows carry `v-memo` with the dependency list they have at `cc60020`?
6. (how) How are `GridPanel`'s rendering and caret movement tested?

## Findings

Each line restates a section's verified answer; the evidence and citations are in the section named.

- At both trees the whole grid is `PanelFrame`'s default slot, so a caret move re-runs one render effect over every row of every column (section 1).
- At tree A each column's row loop has its own `_cache` slot, so a caret move rebuilds only the rows the caret leaves and enters; the other rows return their cached vnode and `patch` skips them (sections 1, 2, 3).
- At tree B the four row loops run once per 200-row block and every block passes the same four `_cache` slots, so each block receives the previous block's vnodes, the key guard rejects them, and every row rebuilds its vnode on every caret move once a chapter has two or more blocks. This comes from reading the runtime and the compiled template; it was not executed (sections 2, 3).
- At tree B the ~50 block elements carry no `v-memo` and are created anew on every render (sections 1, 2).
- The `v-memo` lists came in `43729b3`, against `PanelFrame` re-rendering the whole slot; no measurement of `v-memo` alone was recorded (section 5).
- No test observes a memo hit or miss or a render run on a caret move (section 6).

## 1. What changes on a caret move, and which render functions run again

**Writes in the same tick** (identical at A and B; `editorPanelState.ts`, `inlineStripEligibility.ts` and the strip state files are not in the A→B diff):

- Mouse: `onCellMouseDown` calls `setEditorCaret(id)` and does not write `caretPlacement` (`src/panels/GridPanel.vue:851-855 (onCellMouseDown) @ cc60020`, `:884` at B). `onSelectionChange`, registered on `document` `selectionchange` (`:1235`), fires later and writes the same id again (`src/panels/GridPanel.vue:1143-1150 (onSelectionChange) @ cc60020`, `:1175` at B).
- `setEditorCaret` writes `caretSegmentId` and, for a non-null id, `lastCaretSegmentId` (`src/panels/editorPanelState.ts:281-296 (setEditorCaret) @ cc60020`). `flushEditorNow` writes no reactive state before its first `await` (`:835-902`). `positionFlush.markMoved` and `armPositionFlushTimer` (`:292-295`) are closure variables and a `setTimeout` (`src/panels/positionFlush.ts:81-111 (createPositionFlush) @ cc60020`), not reactive state.
- Keyboard: `goToNextSegment` → `doiConTroToi`, which calls `setEditorCaret(id)` then writes `caretPlacement.value = id` (`src/panels/editorPanelState.ts:1770-1775 (doiConTroToi) @ cc60020`, `:1787-1789`, `caretPlacement` shallowRef `:1309`) — the only extra write against the mouse path.

**Writes later in the same flush cycle:** `TmFuzzyStrip`'s caret watch → `syncTmFuzzyStrip` → `clearShown`, which writes `shown`, `aimed`, `scanError`, `acceptError`, `pendingAccept`, `expandedRows`; `shown` is set again only after a 150 ms debounce plus IPC (`src/tmFuzzyStripState.ts:85-92 (clearShown) @ cc60020`, `:118`, `:133-149`). `ProofreaderStrip`'s watch → `syncProofreadStripHidden` writes `hiddenForSegmentId` when it differs (`src/proofreadState.ts:290-292 (syncProofreadStripHidden) @ cc60020`). `GlossaryConfirmStrip`'s watch → `syncGlossaryConfirmStripTarget` writes `undoEntry` and calls `applyTarget`, not traced (`src/glossaryConfirmStripState.ts:345-356 (syncGlossaryConfirmStripTarget) @ cc60020`). `src/panels/AiTranslationPanel.vue:130 (watch) @ cc60020` watches the caret. On the keyboard path, `watch(editorCaretPlacement, …, {flush: 'post'})` runs after render, writes `caretPlacement` back to null and focuses the target cell (`src/panels/GridPanel.vue:1255-1275 (watch) @ cc60020`, `:1287` at B, where the lookup is `targetCellById`).

**Whose render effect runs the grid.** `GridPanel`'s template root is `<PanelFrame … :show-status="showFrameStatus">`, and all grid markup is its default slot (`src/panels/GridPanel.vue:1740 (template) @ cc60020`, `:1769` at B), rendered by `<slot />` (`src/panels/PanelFrame.vue:174 (slot) @ cc60020`). The slot function is compiled in `GridPanel` but executed by `renderSlot` during `PanelFrame`'s render (`runtime-core.cjs.js line 3267 (renderSlot) @ vue@3.5.40`); `withCtx` switches `currentRenderingInstance` and block tracking, not the active reactive effect (`runtime-core.cjs.js line 687-715 (withCtx) @ vue@3.5.40`); each component renders inside its own `ReactiveEffect` whose scheduler queues its job (`runtime-core.cjs.js line 6253-6263 (setupRenderEffect) @ vue@3.5.40`). So every reactive read in the slot — the four column loops, the memo arrays, `ruleOf`, `stripAnchorIndex` — is tracked by `PanelFrame`'s effect, and a caret change queues `PanelFrame`'s job. The slot is compiled as stable (`_: 1`), and nothing in `GridPanel`'s template sits outside it except `PanelFrame`'s props `owner`, `status-key` and `:show-status`. `GridPanel`'s own render reads only `showFrameStatus`, which has no caret dependency (`src/panels/GridPanel.vue:254-262 (showFrameStatus) @ cc60020`), so `GridPanel`'s own render is not queued by a caret move. Same at A and B.

**How the caret reaches the slot render:** `ruleOf` reads `editorCaretSegmentId.value` (`src/panels/GridPanel.vue:296-298 (ruleOf) @ cc60020`) and is called per row in the rule memo (via `ruleClassForRow`, `:1819`) and the target memo (`:2026`), and through `cellLabelOf` (`:1707-1708`); the target body reads `editorCaretSegmentId === s.id` (`:2037`); `stripAnchorSegmentId`/`stripAnchorIndex` read `inlineStripAnchorSegmentId`, which reads the caret through `eligibleInlineStrips` (`:34`), `proofreaderStripPlace` (`:57`) and the `glossary_confirm`/`proofreader` branches (`:66`, `:69`) (`src/inlineStripEligibility.ts:61-73 (inlineStripAnchorSegmentId) @ cc60020`). With only the TM strip showing, the anchor is `tmFuzzySegmentId` and changes when `shown` is cleared or set, not with the caret directly.

**Other renders queued on a caret move:** `StatusBar` through `positionInChapter`/`positionText` (`src/StatusBar.vue:154-160 (positionInChapter) @ cc60020`, `:168-175`), re-rendering only if the text changed; `AiTranslationPanel` only if one of its caret computeds changed value; the three strips only if a value they read changes. `eligibleInlineStrips` builds a new array each time it re-evaluates. Which of the three strips actually re-render was not determined (templates not read).

**Whole slot or part.** Verified for tree A only; at tree B the per-row reuse below does not happen, because of the shared cache slots (section 3, *Where the answers meet*). At A the whole slot render function re-runs on every caret move; every row's `v-for` callback in all four columns runs and evaluates its memo array; where `isMemoSame` holds the cached vnode is reused and the row body is skipped (`runtime-core.cjs.js line 8620-8635 (isMemoSame) @ vue@3.5.40`); reused vnodes return at `n1 === n2` in `patch` (`:5440`), while each column's keyed diff still walks its whole child array. There is no per-row component or effect. The cost per move, by this reading, is O(rows) callbacks and memo compares across four columns plus an O(rows) keyed diff, with O(1) row bodies rebuilt. It is a reading, not a measurement.

**A against B on this path.** At B the four columns and the strip track sit inside `v-for="block in gridBlockViews"` (`src/panels/GridPanel.vue:1839 (grid) @ b2874aa`), still inside the `PanelFrame` slot, so the same single effect re-runs all blocks and rows. `gridBlockViews` depends only on `editorSegments` and is not re-evaluated on a caret move (`:611-618`). Extra per-move work at B: each `.grid-block` `:style` calls `stripRowIn(block)` (`:620-623`) and creates a new style object, about 50 times at 9,850 rows; `v-if="block.index === stripBlockIndex"` (`:2094`, `:625-628`) mounts or unmounts a block's strip track and its three strip components when the anchor crosses a block boundary; the refs become function refs writing shallowRefs (`:598-603`), and whether Vue calls them on every patch was not traced.

Not done: strip templates, `saveSegmentTargets`, `applyTarget`; no timing.

## 2. How the grid template builds its rows

Both templates were also compiled with `@vue/compiler-sfc` 3.5.40 (output kept beside this document as `compiled/RA.js` and `compiled/RB.js`, cited below by line; that compile has no binding metadata, so references print as `_ctx.x`).

**Tree A.** The grid is column-major and a row is not a DOM element (`src/panels/GridPanel.vue:4-24 (GridPanel) @ cc60020`). Nesting from the root to one cell:

- `PanelFrame` component, the whole grid as its default slot (`src/panels/GridPanel.vue:1740 (template) @ cc60020`; `src/panels/PanelFrame.vue:174 (slot) @ cc60020`)
  - `div#grid-tabpanel.grid-scroll`, `v-if="hasSegments"` (`src/panels/GridPanel.vue:1799 (grid-scroll) @ cc60020`)
    - `div.grid`, `:style` `gridTemplateRows: repeat(editorSegments.length + (stripAnchorIndex<0?0:1), auto)`, patch flag 4 STYLE (`src/panels/GridPanel.vue:1808 (grid) @ cc60020`)
      - four `div.col` elements: rule `:1815`, number `:1837`, source `ref="colSrc"` `:1852`, target `ref="colTgt"` `:1977`
        - per column one keyed loop: `openBlock(true)` + `128 KEYED_FRAGMENT` (`RA.js line 141`, `:310`)
          - per row a Fragment `{key: s.id}`, `64 STABLE_FRAGMENT` (`RA.js line 144-157`), holding the cell and a `div.strip-pad` under `v-if="s.id === stripAnchorSegmentId"` (`:1827`, `:1847`, `:2053`)
      - `div.strip-track` after the target column, outside every `v-for` (`src/panels/GridPanel.vue:2058-2068 (strip-track) @ cc60020`), holding `TmFuzzyStrip`, `GlossaryConfirmStrip`, `ProofreaderStrip`

Rows are keyed `:key="s.id"` in all four columns (`:1818`, `:1840`, `:1855`, `:2018`). No child component sits inside a row except `SourceHanViet` (`:1907`) and `ChapterImage` (`:1896`, `:1959`) inside the source cell. Columns are subgrids: `.col { grid-template-rows: subgrid; grid-row: 1 / -1 }` (`src/panels/GridPanel.vue:2209 (.col) @ cc60020`).

The `v-memo` lists, verbatim:

- Rule (`src/panels/GridPanel.vue:1819 (col-rule) @ cc60020`): `[s, ruleClassForRow(s.id), selectedRowClassById.get(s.id), s.id === stripAnchorSegmentId]`
- Number (`:1841`): `[s, i, selectedRowClassById.get(s.id), s.id === stripAnchorSegmentId]`
- Source (`:1856-1868`): `[s, selectedRowClassById.get(s.id), s.id === stripAnchorSegmentId, cutOffsetsOf(s.id), glossaryTermsOf(s.id), concordanceSourceHighlight?.segmentId === s.id ? concordanceSourceHighlight : null, trailingImagesOf(s.id), s.id === firstSegmentId ? editorChapterAssets : null, editorAssetsDir, showHanViet, viewMode]`
- Target (`:2019-2027`): `[s, i, selectedRowClassById.get(s.id), s.id === stripAnchorSegmentId, rowErrorTextById.get(s.id), editorEditedShape.get(s.id), ruleOf(s.id)]`

The compiled column loop is `renderList(editorSegments, (s, i, _, _cached) => { const _memo = [...]; if (_cached && _cached.el && _cached.key === s.id && _isMemoSame(_cached, _memo)) return _cached; … _item.memo = _memo; return _item }, _cache, N)` (`RA.js line 141-160`). The memo array is evaluated for every row on every render of the component, before the comparison; a hit returns the same vnode object. Each column loop passes its own cache slot: 7 (rule, `RA.js` line 160), 9 (number, line 182), 13 (source, line 294), 15 (target, line 353); no two loops share one, and all four are memoized (`const _memo` at lines 142, 165, 190, 311). The `.col` wrapper divs are rebuilt on every render. A row whose rest rule produces the same class string as `'primary'` would not change the rule column's memo when the caret enters or leaves it.

Reads inside a row that are not in its memo list and so run only on a miss: target cell `editorCaretSegmentId === s.id` (the `editing` class, `:2037`), `rowErrorLabelById.has` (`:2035`), `unlinkedIds.has` (`:2045`), `cellLabelOf(s.id, i)` (`:2049`, defined `:1707`, which reads `ruleOf`); source cell `cutCountOf` (`:1880`), `leadingImagesOf` (`:1895`), `sourcePieceInfoOf` (`:1929`), the locale through `t()`; rule cell `unlinkedIds`. The `editing` class stays correct because `ruleOf(s.id)` in the target memo returns `'primary'` exactly when `editorCaretSegmentId === s.id` (`src/panels/GridPanel.vue:296-298 (ruleOf) @ cc60020`), so the memo flips for the row the caret leaves and the row it enters.

Reads outside the per-row memos: `editorSegments.length` and `stripAnchorIndex` on `div.grid` (`:1808`); `sourceTokenClass` on the source column (`:416`); `stripAnchorIndex` on the strip track (`:2062-2063`). Every row's memo array calls `ruleOf` (directly or via `ruleClassForRow`), and `ruleOf` reads `editorCaretSegmentId.value` then `restRuleById.value.get(id)` (`src/panels/GridPanel.vue:296-302 (ruleOf) @ cc60020`). So the render effect that runs the grid slot (section 1: `PanelFrame`'s) is subscribed to `editorCaretSegmentId`; a caret move re-runs the slot render, walks every row's memo array, and only the two rows whose `ruleOf` changed miss. When a caret-anchored strip wins (`glossary_confirm` open, or `proofreader` placed in the track), the anchor follows the caret (`src/inlineStripEligibility.ts:61-73 (inlineStripAnchorSegmentId) @ cc60020`), so `stripAnchorIndex`, the grid `:style`, the strip-track style and the `s.id === stripAnchorSegmentId` entries of the two rows also change.

**Tree B.** `git diff cc60020 b2874aa -- src` touches `GridPanel.vue`, the new `gridBlocks.ts` and one selector in `proofreadHighlight.ts`. Nesting:

- `PanelFrame` → `div.grid-scroll` (`src/panels/GridPanel.vue:1828 (grid-scroll) @ b2874aa`)
  - `div.grid ref="gridRoot"`, static, no `:style`, flag 512 (`src/panels/GridPanel.vue:1837 (grid) @ b2874aa`)
    - block loop `v-for="block in gridBlockViews" :key="block.index"`, a plain keyed loop with no `v-memo`: `openBlock(true)` + `128 KEYED_FRAGMENT`, no `_memo`, no `_cache` slot at block level (`src/panels/GridPanel.vue:1839-1844 (grid) @ b2874aa`; `RB.js line 140-144`, `:381`)
      - `div.grid-block` per block, `:style` `gridTemplateRows: repeat(block.segments.length + (stripRowIn(block) < 0 ? 0 : 1), auto)` (`:1842`)
        - the same four `div.col` elements, each with the same per-row keyed, memoized loop over `block.segments`; then `#strip-track` with `v-if="block.index === stripBlockIndex"` (`src/panels/GridPanel.vue:2093-2105 (strip-track) @ b2874aa`)

Blocks come from `gridBlocksOf` with `GRID_BLOCK_ROWS = 200` (`src/panels/gridBlocks.ts:1-11 (gridBlocksOf) @ b2874aa`). `gridBlockViews` is a computed over `editorSegments.value` only; it builds `{index, start, segments: segments.slice(...)}` per block, so block objects are new whenever `editorSegments` changes while each `s` keeps its identity (`src/panels/GridPanel.vue:611 (gridBlockViews) @ b2874aa`). The memo lists are A's, except that the number and target lists use `block.start + j` where A uses `i` (`:1876`, `:2056`); the rule list and the source list's tail (`:1892-1903`) are unchanged. `v-memo` appears only on the four column loops at B (`:1854`, `:1876`, `:1891`, `:2054`).

The column loops inside the block callback compile with cache slots 7 (rule, `RB.js line 167`), 9 (number, `:189`), 13 (source, `:302`) and 15 (target, `:362`). The slot numbers are literals at the call site, so every block passes the same four slots of the same component-level `_cache` array; the outer block loop passes no `_cache` and carries no memo (`RB.js` line 140). The per-row guard compares `_cached.key === s.id` (`RB.js` lines 150, 173, 211, 329). What this does at run time is in section 3 and *Where the answers meet*.

Template refs: `colSrc`/`colTgt` are function refs set only on block 0 (`:ref="block.index === 0 ? setColSrcRef : undefined"`, `:1887`, `:2013`, setters `:598-603`); `gridRoot` is the only template ref on a stable element (`:597`).

Reads outside the row memos at B, per block on every render: `block.index`, `block.segments.length`, `stripRowIn(block)` (`:620-623`, reads `stripAnchorIndex`), `sourceTokenClass`, `block.index === 0`, `block.index === stripBlockIndex` (`:625-628`, `:2094`). Every row's memo still calls `ruleOf`, so at B too the slot's render effect is subscribed to `editorCaretSegmentId` and a caret move walks all blocks and rows; the 50 block elements and their column vnodes are created anew each render because the block loop has no memo.

Not read: the bodies of `cutOffsetsOf`, `glossaryTermsOf`, `trailingImagesOf`, `leadingImagesOf`, `sourcePieceInfoOf`, `selectedSegmentIds`, `textShapeOf`.

## 3. What Vue 3.5.40 does for children on a parent re-render

Read on disk; cited `@ vue@3.5.40`. One compile of three sample templates through `@vue/compiler-dom` was run to see the codegen.

**Codegen.** `v-for` with `v-memo` on the item does not use `withMemo`; the memo is compiled into the `renderList` callback as `const _memo = (exp); if (_cached && _cached.el && _cached.key === <key> && isMemoSame(_cached,_memo)) return _cached; const _item = <block>; _item.memo = _memo; return _item`, and `renderList` gets the extra arguments `_cache, <slot index>` (`node_modules/@vue/compiler-core/dist/compiler-core.cjs.js line 5029-5052 (transformFor) @ vue@3.5.40`). `withMemo` is emitted only for `v-memo` outside a `v-for` (`compiler-core.cjs.js line 6642 (transformMemo) @ vue@3.5.40`). A `v-for` fragment is `STABLE_FRAGMENT` (64) only for a constant source, otherwise `KEYED_FRAGMENT` (128) with a `:key`, else `UNKEYED_FRAGMENT` (256) (`compiler-core.cjs.js line 4950-4951 @ vue@3.5.40`); a dynamic source gets `openBlock(true)`, which disables block tracking.

**`renderList` and the cache.** `renderList` reads `cached = cache && cache[index]` (`node_modules/@vue/runtime-core/dist/runtime-core.cjs.js line 3194 (renderList) @ vue@3.5.40`), passes `cached && cached[i]` as each callback's fourth argument (`:3211`, `:3223`, `:3230`, `:3237`), and after the loop stores `cache[index] = ret` (`:3244`). The cache is one array per slot index, replaced on every call. `isMemoSame` compares memo arrays element-wise with `hasChanged`; on equality it pushes `cached` into `currentBlock` only when block tracking is on, which inside a `v-for` fragment it is not (`runtime-core.cjs.js line 8620-8636 (isMemoSame) @ vue@3.5.40`).

**(a) Flat `v-for` + `v-memo`, memo unchanged.** The parent's render body runs (`runtime-core.cjs.js line 6208 (componentUpdateFn) @ vue@3.5.40`), the callback runs per item and evaluates `_memo` plus the key check, and returns the previous vnode object; the item's child expressions do not run and no vnode is allocated. The keyed fragment goes through `patchChildren` (`:6279-6294`) to `patchKeyedChildren` (`:6399`), whose head loop calls `cloneIfMounted` when `optimized`, which returns a vnode carrying `memo` as is (`:7880-7882 (cloneIfMounted)`), then `patch`, which returns at `if (n1 === n2) return;` (`:5440 (patch)`). With every pair the same type the head loop runs to the end, nothing is mounted or unmounted, and the key map / longest-increasing-subsequence section is never reached (`:6399-6420 (patchKeyedChildren)`; the head loop was read, the later branches inferred from its exit condition): O(N) `patch` calls, no moves, no DOM writes. Per item the cost is the callback, the memo compare, `cloneIfMounted`, `isSameVNodeType` and one `patch` call; each `renderList` call also allocates one `new Array(n)`.

**(b) The same vnodes inside a Fragment or an outer `v-for`.** Wrapper vnodes have no memo and are created anew per outer item on every render. A `<template v-for>` wrapper with a keyed child compiles to a `Fragment` (flag 64) holding an inner `openBlock(true)` + `KEYED_FRAGMENT` `renderList`; `processFragment` (`:5914-5975`) uses `patchBlockChildren` (`:5854-5890`) when the flag is 64 and the dynamic-children lengths match, else `patchChildren` on the full list. Inside, the inner keyed list is patched as in (a).

For a nested `v-for`, the compiled inner `renderList` call passes the same `_cache` slot index on every outer iteration (sample output: `outer renderList(groups, g => Fragment{key:g.id}[ inner KEYED_FRAGMENT renderList(g.rows, ..., _cache, 0) ], 64)`). Because `renderList` overwrites `cache[index]` after each call (`runtime-core.cjs.js line 3244 (renderList) @ vue@3.5.40`), the inner `cached[i]` seen by outer item j is the vnode array of outer item j−1 (for j = 0, of the last outer item of the previous render). The compiled guard `_cached.key === <key>` then fails whenever keys differ across groups, so the memo misses and the item is rebuilt on every render. A flat `v-for` has one cache array per slot and is not affected. Checked against tree B's compiled output: for block k, `cache[slot]` holds the array block k−1's inner call stored in the same render (for block 0, the last block's from the previous render), segment ids differ across blocks, so `_cached.key === s.id` fails and every row rebuilds its vnode on every render; nothing in the compiled output keeps the cache per block. With a single block (≤ 200 segments) the previous array is that block's own and the memo can still hit. This rests on reading the code and the compiled output; it was not executed.

**(c) Child components with unchanged props.** The parent render still creates a new component vnode per child; `patch` goes to `processComponent` (`:5977`) and `updateComponent` (`:6052`), which calls `shouldUpdateComponent` (`runtime-core.cjs.js line 4771-4825 (shouldUpdateComponent) @ vue@3.5.40`). When it returns false only `n2.el = n1.el` and `instance.vnode = n2` run; `instance.update()` is not called, so the child's render function and subtree patch do not run. In optimized mode it returns true for `DYNAMIC_SLOTS` (1024), compares all props for `FULL_PROPS` (16), only the listed `dynamicProps` for `PROPS` (8), and false for static props with flag 0. In non-optimized mode any slot children without `$stable` return true. Directives or a transition on the next vnode return true before any of these checks (`runtime-core.cjs.js` line 6052-6070 (updateComponent)).

**Where the parent patch starts.** On update, `componentUpdateFn` renders `nextTree` and calls `patch(prevTree, nextTree, …)` with `optimized` defaulting to `!!n2.dynamicChildren` (`:6205-6223`, `:5439`). `patchElement` uses `patchBlockChildren` only when `dynamicChildren` exists and the lengths match, else falls back to a full diff (`:5758-5800`); this was not read end to end.

Not read: `normalizeVNode` (`:7864`), `traverseStaticChildren`, `mountComponent`; the cost of `renderList` over a reactive array (`shallowReadArray`, `toReactive`, `:3198-3206`) was seen and not analyzed.

## 4. What reads the caret and strip-anchor state as a `GridPanel` render dependency

**State written.** `setEditorCaret` writes two refs synchronously, `caretSegmentId` and `lastCaretSegmentId` (the latter only for a non-null id) (`src/panels/editorPanelState.ts:281-296 (setEditorCaret) @ cc60020`, refs declared `:148-149`, exported read-only as `editorCaretSegmentId` `:177` and `editorLastCaretSegmentId` `:179`); B is the same. It calls `void flushEditorNow()` when the caret moves between two different non-null ids (`:286`) and `positionFlush.markMoved(...)` / `armPositionFlushTimer()` (`:292-295`, not traced further). `flushEditorNow` writes no ref before its first `await`; `flushError` and `lastSavedAt` are written after `await saveSegmentTargets`, asynchronously (`src/panels/editorPanelState.ts:835 (flushEditorNow) @ cc60020`, `:859`, `:873-879`). Nothing in `GridPanel` watches `editorCaretSegmentId` or `stripAnchorSegmentId`.

**Computeds and helpers between the state and the template** (same at A and B unless noted):

- `src/inlineStripEligibility.ts:29-48 (eligibleInlineStrips) @ cc60020` — reads the caret at `:34`
- `src/inlineStripEligibility.ts:53-58 (proofreaderStripPlace) @ cc60020` — reads the caret at `:57`
- `src/inlineStripEligibility.ts:61-73 (inlineStripAnchorSegmentId) @ cc60020` — reads the caret at `:66`, `:69`, and `tmFuzzySegmentId` at `:64`
- `src/panels/GridPanel.vue:600 (stripAnchorSegmentId) @ cc60020` (`:605` at B) — returns `inlineStripAnchorSegmentId.value`
- `src/panels/GridPanel.vue:601-604 (stripAnchorIndex) @ cc60020` (`:606-609` at B) — `findIndex` over all of `editorSegments`
- `src/panels/GridPanel.vue:296-298 (ruleOf) @ cc60020` (`:297-298` at B) — reads `editorCaretSegmentId.value`
- `src/panels/GridPanel.vue:300-301 (ruleClassForRow) @ cc60020` — calls `ruleOf`
- `src/panels/GridPanel.vue:1707-1708 (cellLabelOf) @ cc60020` (`:1736-1737` at B) — calls `ruleOf`
- B only: `src/panels/GridPanel.vue:620-623 (stripRowIn) @ b2874aa` and `src/panels/GridPanel.vue:625-628 (stripBlockIndex) @ b2874aa` — read `stripAnchorIndex`

**Template reads at A, once per render:** `src/panels/GridPanel.vue:1808 (grid) @ cc60020` (`stripAnchorIndex` in the `.grid` `:style`); `:2062-2063` (`stripAnchorIndex` in the strip track's `:class` and `:style`); `:2066-2068` (the three strips, always rendered).

**Template reads at A, inside a per-row memo** (the list evaluated once per row per render; the body only on a miss):

- rule: list `:1819` reads `ruleClassForRow(s.id)` and `s.id === stripAnchorSegmentId`; body `:1825`, `:1827`
- number: list `:1841` reads `s.id === stripAnchorSegmentId`; body `:1847`
- source: list `:1859` reads `s.id === stripAnchorSegmentId`; body `:1967`
- target: list `:2023` (anchor) and `:2026` (`ruleOf(s.id)`); body `:2037` (`editing: editorCaretSegmentId === s.id`, not in the list), `:2049` (`cellLabelOf`), `:2051`, `:2053` (anchor). The body's caret reads are kept in step by `ruleOf(s.id)` in the list.

`SourceHanViet`, `ChapterImage` and `PanelFrame` hold no caret or anchor read (`git grep` over `src` at both shas).

**Template reads at B.** The same four lists, nested in blocks: rule `:1854` (body `:1860`, `:1862`), number `:1876` (body `:1882`), source `:1894` (body `:2002`), target `:2058` (anchor) and `:2061` (`ruleOf`) (body `:2072` caret, `:2084` `cellLabelOf`, `:2086`, `:2088` anchor), all at B. Per block, outside any memo: `stripRowIn(block)` at `:1842` (`gridTemplateRows`) and twice for the strip track's class and style (`:2098-2099`), and `block.index === stripBlockIndex` at `:2094`. The three strips render only in the block where that `v-if` holds (`:2102-2104`).

**Children of `GridPanel`:**

- `src/TmFuzzyStrip.vue:30 (watch) @ cc60020` — `watch(editorCaretSegmentId, syncTmFuzzyStrip, {immediate: true})`: a watch, not a render read; `:32 (isVisible)` reads `eligibleInlineStrips`, so the caret (template line not read); `:47-50 (currentTarget)` reads `tmFuzzySegmentId`
- `src/GlossaryConfirmStrip.vue:60-66 (watch) @ cc60020` — `watch([editorCaretSegmentId, editorSegments, glossaryMarks], syncGlossaryConfirmStripTarget)`: a watch; `isVisible` (`:74`) reads `confirmStripIsOpen`, `quickAddIsOpen`, not the caret
- `src/ProofreaderStrip.vue:40 (watch) @ cc60020` — `watch(editorCaretSegmentId, syncProofreadStripHidden)`: a watch; `:34-38 (isVisible)` reads `eligibleInlineStrips` and `proofreaderStripPlace`, so the caret; `:63 (isJob)`; `:220`, `:225` read `proofreadScopeCounts.segment`, which resolves to `src/proofreadScope.ts:19 (proofreadScopeIds) @ cc60020` reading `editorCaretSegmentId.value ?? editorLastCaretSegmentId.value` (whether a `v-if` gates it was not checked)

**Caret readers that are not `GridPanel` render dependencies:**

- `src/StatusBar.vue:154-156 (positionInChapter) @ cc60020`
- `AiTranslationPanel.vue`: `:130` watch; computeds `:188`, `:193`, `:208`, `:229`, `:235`
- `src/AiPromptInspectorOverlay.vue:112 (isStale) @ cc60020`
- `proofreadState.ts`
- `src/panels/segmentSelectionState.ts:133 (moveSegmentSelectionFocus) @ cc60020`
- `src/panels/segmentHistoryState.ts:124 (openSegmentHistory) @ cc60020`
- `src/tmSettingsState.ts:64 (saveTmSettings) @ cc60020`
- `src/tmManageCommandDeps.ts:73 (tmManageCommandDeps) @ cc60020`
- `src/tmConcordanceCommandDeps.ts:35 (searchFromVietnameseSelection) @ cc60020`
- `src/aiTranslateHandlers.ts:45 (aiTranslateHandlers) @ cc60020` and `:77`
- `src/main.ts:1090-1092 (boot) @ cc60020`

Whether `src/panels/segmentSelectionState.ts:133 (moveSegmentSelectionFocus) @ cc60020` reaches the render through `selectedRowClassById` was not checked. No file in `src/commands`, `src/config`, `src/i18n`, `src/layout`, `src/modes`, `src/selftest`, `src/tokens` reads the caret or anchor; `WorkspaceMode.vue` and `WorkspaceDock.vue`, which host `GridPanel`, read no caret state (`src/layout/WorkspaceDock.vue:1144-1157 (template) @ cc60020`; `src/modes/WorkspaceMode.vue:137 (stage-slot) @ cc60020`).

**Writers of the caret in `GridPanel`:** `src/panels/GridPanel.vue:854 (onCellMouseDown) @ cc60020`, `:1047`, `:1053`, `:1149` at A (`:887`, `:1080`, `:1086`, `:1180` at B).

Not read: the three strip templates in full; `tmFuzzyStripState.ts` and `glossaryConfirmStripState.ts` in full.

## 5. Why the rows carry `v-memo` with these lists

All four `v-memo` attributes came in one commit, `43729b3 "perf(grid): 2-14 dừng ở blocked vì dời con trỏ ở 9.850 câu vẫn ~600–740 ms trên bản debug — sáu phép đo tách cho thấy ~390 ms là WebKit tính lại bố cục cả lưới `subgrid` khi `v-if` của vạch lề chèn một `div.rule` vào hàng nhận con trỏ, còn ~225 ms là `PanelFrame` dựng lại cả slot lưới; số ô sửa được, vùng chọn, bộ gõ và CSS của lưới không phải nguyên nhân, và bàn phát lại đột biến đầu tiên báo sai "0 mutation" vì `MutationObserver` mang callback rỗng; nay vạch lề luôn có mặt ở mọi hàng, `v-memo` ở bốn cột, và mọi vạch dày 3 px chỉ khác kiểu nét và màu (`confirmed` thành nét đôi) vì đổi bề dày viền cũng ép tính lại bố cục cả lưới (~700 → ~160 ms trên bàn biến thể CSS); Ice ngừng sửa lưới ở đây và đo lại trên bản dựng thật sau, một phím gõ ở 9.850 câu vẫn ~700 ms"` (2026-10-23); `git log -S'v-memo' cc60020 -- src` returns only it, and `git blame` attributes to it `src/panels/GridPanel.vue:1819 (col-rule) @ cc60020`, `:1841`, `:1856-1868` and `:2019-2027` at A except `:2025`, which `git blame` attributes to `043f3bc "perf(editor): 2-15 dừng ở blocked vì bỏ lượt dựng lại cả lưới mỗi phím chỉ giảm ~20% ở 9.850 câu — mỗi `input` thay cả `Map` chữ đang sửa, mà thân lưới là slot của `PanelFrame` nên mỗi phím chạy lại 4 cột × n hàng cộng `restRuleById` trên n câu; nay `Map` ghi tại chỗ, lưới chỉ được báo khi tính rỗng của câu đổi (thứ duy nhất lưới hiển thị theo chữ đang gõ), và ca đếm ở 20 và 200 câu đỏ đúng hai ca khi gỡ bản sửa; nhưng bản release gõ thật 300 phím ở ~10 phím/giây vẫn tụt 157–163 s ở 9.850 câu (trước 188–193 s), ~0,5 s mỗi phím chưa quy nguồn, còn 122 câu đã đạt ≤ 55 ms từ trước khi sửa; `event.timeStamp` của WebKit là giờ chạy handler chứ không phải giờ phím tới, nên bàn đo nội suy giờ gửi giữa hai mốc `osascript`"`, which replaced the `editorEditedText.get(s.id)` entry that 43729b3 had put there with `editorEditedShape.get(s.id)`.

Per entry:

- `s` (all four), `i` (number, target): 43729b3; no reason stated.
- `ruleClassForRow(s.id)` (rule) and `ruleOf(s.id)` (target): 43729b3 defines `ruleOf`, `ruleClassForRow` and `restRuleById` in the same diff; `ruleOf` returns `'primary'` for the caret row and otherwise the `restRuleById` entry, replacing the earlier `ruleById`/`ruleClassById` computeds that took the caret as input and so changed for every row on every caret move. With the split only the old and new caret rows change their memo dependency; the diff shows this, the message does not say it. `cellLabelOf` moved to `ruleOf(id)` in the same diff.
- `selectedRowClassById.get(s.id)` (all four): the Map dates from `915f1af`; 43729b3 adds it because the cells already read it in `:class`.
- `s.id === stripAnchorSegmentId` (all four): the variable dates from `a92008f`; rows already emit a `strip-pad` after the anchor row.
- `rowErrorTextById.get(s.id)` (target): the Map dates from `de7c67e`; the target cell already used it.
- `editorEditedShape.get(s.id)` (target): 043f3bc's message says each `input` replaced the whole edited-text Map, so "mỗi phím chạy lại 4 cột × n hàng cộng `restRuleById` trên n câu"; the fix mutates the Map in place and notifies the grid only when a sentence's emptiness changes ("thứ duy nhất lưới hiển thị theo chữ đang gõ").
- Source entries (`cutOffsetsOf`, `glossaryTermsOf`, the `concordanceSourceHighlight` ternary, `trailingImagesOf`, `editorChapterAssets` for the first row, `editorAssetsDir`, `showHanViet`, `viewMode`): 43729b3, the inputs the source cell already read; the accessors date from `a664dac`, `53035e7`, `3a2e03c` and `46f068f`. The one related change is `trailingImagesOf` returning a shared `NO_IMAGES` constant instead of a fresh `[]`, keeping the memo comparison stable.

What the memo was meant to make cheaper: 43729b3's message puts about 390 ms of a 600–740 ms caret move at 9,850 segments (debug build) on WebKit re-laying out the whole subgrid when a rule `v-if` inserted a `div.rule`, and about 225 ms on "`PanelFrame` dựng lại cả slot lưới". It lists three fixes: the rule always present (`div.margin-slot`, `ruleSlotClass`), `v-memo` on the four columns, and all rule borders at 3 px. Its only number tied to a fix, "~700 → ~160 ms trên bàn biến thể CSS", was measured on a CSS-variant bench; it gives no figure for `v-memo` alone.

Neither 43729b3 nor 043f3bc changes `src/panels/PanelFrame.vue`; the only earlier commits on it are `90d4428`, `505c8bc` and `5a23410`. 043f3bc's message says "thân lưới là slot của `PanelFrame` nên mỗi phím chạy lại 4 cột × n hàng". 043f3bc added `tests/frontend/gridTypingCost.test.ts`, which counts stub `PanelFrame` renders on a keystroke; 43729b3 added `gridCaretMoveCost.test.ts` and `gridRuleStable.test.ts`, which stub `PanelFrame` as a plain slot and do not mention memo.

Budget: 2 of 10 commits followed (43729b3, 043f3bc), plus `git log -S` lookups for the older entries.

## 6. How `GridPanel` rendering and caret movement are tested

Vitest runs under happy-dom (`vitest.config.ts:57 (vitest) @ cc60020`), which has no layout; `tests/AGENTS.md:10 (Conventions) @ cc60020` assigns geometry, layout and real-engine claims to e2e. Every vitest "cost" test counts calls or DOM mutations.

**The three render-cost tests (A):**

- `tests/frontend/gridCaretMoveCost.test.ts:111-123 (GridPanel caret move cost) @ cc60020` — wraps `segmentRuleInputOf` with a counter (`:13-22`), mounts the real `GridPanel` with `PanelFrame` stubbed as a plain slot (`:69`, `mountGrid` `:91-100`), moves the caret 100 → 101 → 102 with a tick after each (`:114-120`), and asserts `ruleInputCalls === 0` (`:121`), at 20 and 200 rows (`many(n)`, `:79-89`). It fails if any caret move re-resolves a row's rule through that function. It does not see whether a row template re-ran, whether a render function ran, how many vnodes were patched, or any time. `:125-305` hold one test per value a row reads (caret `:125`, text `:140`, `:152`, glossary `:164`, Han Viet `:179`, row error `:188`, `:201`, unlinked `:215`, selection `:230`, source cut `:245`, concordance `:257`, strip anchor `:270`), each checking DOM output after one change — a guard that no memo list misses a value.
- `tests/frontend/gridTypingCost.test.ts:114-128 (GridPanel typing cost) @ cc60020` — counts renders of a `PanelFrame` stub whose render reads `slots.default()` (`probe.frameRenders`, `:58-67`) plus the rule counter; types three strings by `textContent` + `input` (`:108-112`) and asserts both counters stay 0 (`:124-125`), at 20 and 200 rows. It is the only proxy for "the grid slot re-rendered", and it is asserted for typing only, never for a caret move.
- `tests/frontend/gridRuleStable.test.ts:42-101 (GridPanel margin rule column) @ cc60020` — a `MutationObserver` on `.col-rule` with `childList`/`subtree` (`:50-52`) across a caret walk 13, 12, 11, null (`:54-60`), asserting no node added or removed and identical slot nodes (`:61-67`); attributes are not observed. `:79-90` reads `GridPanel.vue` CSS text and compares `border-left` widths.

No test observes a `v-memo` hit or miss, a render-function run on a caret move, or a patch count; the probes that exist on a caret move are the rule-resolution counter and `gridRuleStable`'s `childList` observer: `git grep` for `v-memo|vMemo|onRenderTriggered|renderTracked|onUpdated|patchFlag|renderCount` over `tests/` and `e2e/` at `cc60020` returns nothing. A memo miss surfaces only as an extra `segmentRuleInputOf` call, when the missed row's work includes the rule.

**Tests whose DOM reads change because tree B adds a block wrapper:** `tests/frontend/gridPanelStripTrack.test.ts:69-70 (grid) @ cc60020`, `:79-80`, `:98`, `:107` at A read `.grid` `style.gridTemplateRows` and expect `repeat(3|4, auto)`; at B `.grid` has no inline style, so these four assertions fail. `tests/frontend/gridPanelImages.test.ts:155-157 (gridPanelImages) @ cc60020` fails the same way. `e2e/specs/story-12-7-workspace-grid-status-bar.e2e.mjs:70-85 (workspace-grid-status-bar) @ cc60020` expects four computed grid columns on `.grid`; predicted to change at B, not run. Single-block fixtures keep the `.col-*`, `.strip-pad` and `.strip-track` checks passing (`tests/frontend/gridPanelStripTrack.test.ts:71 (grid) @ cc60020`, `:81-89`, `:109`; `tests/frontend/gridPanelImages.test.ts:159-163 (gridPanelImages) @ cc60020`); no test read mounts more than 200 rows. Tree B's only test change is one line in `tests/frontend/proofreadHighlight.test.ts:117 (hostWithTwoCells) @ b2874aa`, matching the new `[data-col="tgt"]` selector (`src/panels/proofreadHighlight.ts:124 (repaintProofreadHost) @ b2874aa`).

**Caret through DOM selection:** `tests/frontend/editorTypingZone.test.ts:105-113 (putCaretIn) @ cc60020` places a caret with `Range` + `getSelection()` + `selectionchange`; `:140`, `:160` check `contenteditable` and offset across a move.

**e2e:** no spec measures a caret move driven by a real mouse or key press, and none asserts a time threshold. `e2e/specs/story-12-9-strip-row-track.e2e.mjs:44-83 (focusAndSample) @ cc60020` focuses a cell and samples `requestAnimationFrame` gaps; `:86-175` prints medians and maxima (`:169-174`) and asserts only that the strip appears before a 15 s timeout, a ≤ 2 px geometry check and unchanged scrollTop (`:111`, `:126-128`, `:147`), on the debug build (`:170`). `e2e/specs/segment-navigation.e2e.mjs:189-304 (segment-navigation) @ cc60020` checks scrollTop after `⌘⌥↓`. `e2e/support/gridWait.mjs:72-82 (countGridCells) @ cc60020` are count/wait helpers.

Not read: `tests/frontend/proofreadWiring.test.ts:163-217 (proofreadWiring) @ cc60020`, source-regex tests on `GridPanel.vue`; the `segmentRuleInputOf` mock block, `tests/frontend/gridTypingCost.test.ts:14-23 (gridTypingCost) @ cc60020`.

## Where the answers meet

**`renderList` and its `_cache` slots (sections 1, 2, 3).** Every row loop of the grid reaches `renderList` with a literal cache slot (`node_modules/@vue/runtime-core/dist/runtime-core.cjs.js` line 3194, 3244 (renderList) @ vue@3.5.40), and the check that rejects a cached row is the key comparison the compiler puts before `isMemoSame` (`node_modules/@vue/compiler-core/dist/compiler-core.cjs.js` line 5029-5052 (transformFor) @ vue@3.5.40). At A each column loop owns its slot, so the comparison sees the same row's previous vnode (section 2). At B the four call sites run once per block with shared slots (`src/panels/GridPanel.vue:1839-1844 (grid) @ b2874aa`), so the comparison sees the previous block's row and rejects it (section 3). Section 1's two-rows reading therefore holds at A and not at B.

**`PanelFrame`'s render effect (sections 1, 5, 6).** The whole grid is `PanelFrame`'s stable default slot, so a caret move queues `PanelFrame`'s effect and re-runs the slot at both trees (`src/panels/GridPanel.vue:1740 (template) @ cc60020`; `src/panels/PanelFrame.vue:174 (slot) @ cc60020`). `43729b3` names the same mechanism ("`PanelFrame` dựng lại cả slot lưới") and left `PanelFrame.vue` unchanged (section 5). The only test probe on that effect, `frameRenders`, is asserted for typing, not for a caret move (`tests/frontend/gridTypingCost.test.ts:114-128 (GridPanel typing cost) @ cc60020`, section 6).

**`ruleOf` (sections 1, 2, 4, 5).** `ruleOf` is the read that subscribes the slot render to `editorCaretSegmentId` and the memo entry that limits rebuilt rows to the two caret rows at A (`src/panels/GridPanel.vue:296-302 (ruleOf) @ cc60020`); it came with the `v-memo` lists in `43729b3` (section 5) and is unchanged at B.

## Open questions

- Section 3's tree-B finding comes from reading the runtime and the compiled output, not from running them. No question in this set measures how many row callbacks return a cached vnode on one caret move at B. Next ask: at tree B with 9,850 segments, how many of the four columns' row callbacks return `_cached` on one caret move, and how many build a new row vnode?
- Which of `TmFuzzyStrip`, `GlossaryConfirmStrip` and `ProofreaderStrip` re-render on a caret move was not settled; their templates were not read (section 1, section 4). Next ask: what do the three strip templates read during render?
- What `applyTarget` in `src/glossaryConfirmStripState.ts` writes was not traced (section 1).

## Prior documents

- `research-go-cham-luoi-subgrid-webkit` — WebKit layout cost of the chapter-wide subgrid; in reach, not opened (this ask is about render cost, not layout).
- `research-phase-0-spike-results`, `research-technical-auratranslate-tauri-rust-local-first` — unrelated.
- No ADR files in the repository.
