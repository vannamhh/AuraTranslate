Digest, area `src/panels/` at `cc60020` (A) and `b2874aa` (B). Inside `src/panels/`, B changes `GridPanel.vue` (+74/−35), adds `gridBlocks.ts`, and changes one selector in `proofreadHighlight.ts`. Outside the area it changes one test, `tests/frontend/proofreadHighlight.test.ts` (+1 line).

## Q1: what changes when the caret moves, and what renders again (A and B)

**Caret state and how it is written**
- `src/panels/editorPanelState.ts:148 @ cc60020`: the module refs `caretSegmentId` and `lastCaretSegmentId` (`ref<number|null>`).
- `src/panels/editorPanelState.ts:177 (editorCaretSegmentId) @ cc60020`: the read-only export of `caretSegmentId`.
- `src/panels/editorPanelState.ts:281 (setEditorCaret) @ cc60020`: writes `caretSegmentId` and `lastCaretSegmentId`. When the caret goes from segment A to segment B it calls `flushEditorNow()`, then `positionFlush.markMoved` and `armPositionFlushTimer`.
- `src/panels/editorPanelState.ts:1770 (doiConTroToi) @ cc60020`: the keyboard and command path. It calls `setEditorCaret(id)` and also sets `caretPlacement.value = id`; `caretPlacement` is a `shallowRef` declared at `:1309`.
  - Callers: `goToNextSegment` at `:1787`, `goToPrevSegment` at `:1796`, `moveEditorCaretTo` at `:1778`.
  - Other writers of `caretPlacement`: `:1537` and `:2969`. Other calls to `setEditorCaret`: `:1536` (confirm) and `:2968` (regroup).

**Where `GridPanel` sets the caret**
- Mouse: `src/panels/GridPanel.vue:851 (onCellMouseDown) @ cc60020`, which is `:884 @ b2874aa`.
- Selection: `GridPanel.vue:1143 (onSelectionChange) @ cc60020`. It is registered on document `selectionchange` at `:1235`. At `b2874aa` it is at `:1175` and checks `isInTargetColumn(anchor)` instead of `colTgt.contains`.
- Clearing the caret (`null`): `onColumnFocusOut` at `:1044` and `onStripFocusOut` at `:1050 @ cc60020` (`:1077` @ b2874aa).
- `GridPanel.vue:1255 (watch editorCaretPlacement) @ cc60020`, which is `:1287 @ b2874aa`: uses `flush:'post'`, calls `clearEditorCaretPlacement`, finds the target cell and calls `focus()`.
  - A finds the cell with `colTgt.querySelector`; B uses `targetCellById` (`b2874aa:635`).

**Derived state in `GridPanel` that depends on the caret**
- `GridPanel.vue:296 (ruleOf) @ cc60020` (`:297` @ b2874aa) returns `'primary'` when `editorCaretSegmentId.value === id`. `ruleClassForRow` at `:300` wraps it.
- `GridPanel.vue:1707 (cellLabelOf) @ cc60020` (`:1736` @ b2874aa) calls `ruleOf`.
- `GridPanel.vue:600-604 (stripAnchorSegmentId, stripAnchorIndex) @ cc60020` (`:605-609` @ b2874aa) wraps `inlineStripAnchorSegmentId`.
  - That computed is outside the area: `src/inlineStripEligibility.ts:61 @ cc60020`. In its `glossary_confirm` (open) and `proofreader` (track) branches it returns `editorCaretSegmentId`, so the strip anchor follows the caret in those cases.
- Only in B:
  - `GridPanel.vue:611 (gridBlockViews) @ b2874aa` reads only `editorSegments`.
  - `:620 (stripRowIn)` and `:625 (stripBlockIndex) @ b2874aa` read `stripAnchorIndex`.

**What re-renders**
- All of the grid's markup is the default slot of `PanelFrame`: `<PanelFrame …>` at `GridPanel.vue:1740 @ cc60020` (`:1769` @ b2874aa), and `<slot />` at `src/panels/PanelFrame.vue:174 @ cc60020`. `PanelFrame` holds its own `focused` ref at `:78`.
- Commit `43729b3` is an ancestor of A and is the commit that added `v-memo`. Its message attributes about 225 ms per caret move to "`PanelFrame` dựng lại cả slot lưới" (PanelFrame rebuilding the whole grid slot), and about 390 ms to WebKit recomputing layout for the whole subgrid.

**Other places in `src/panels` that read the caret (not `GridPanel` render)**
- `AiTranslationPanel.vue:130` (a watch) and its computeds at `:188`, `:193`, `:208`, `:229`, `:235 @ cc60020`.
- `segmentHistoryState.ts:124 (openSegmentHistory)`.
- `segmentSelectionState.ts:133 (moveSegmentSelectionFocus)`.
- `segmentSelectionIds` at `segmentSelectionState.ts:81 @ cc60020` does not read the caret.

## Q2: how the template builds rows (A and B)

**Tree A (`cc60020`)**
- Header comment, `GridPanel.vue:5-24 @ cc60020`: the grid is column-major and a row is not a DOM element.
- One `div.grid` at `:1808`. Its `:style` is `gridTemplateRows: repeat(editorSegments.length + (stripAnchorIndex<0?0:1), auto)`.
- Four columns. Each has one `<template v-for … :key="s.id" v-memo>` over `editorSegments`; a `<template>` with `v-for` compiles to a Fragment.

| Column | Location @ cc60020 | `v-for` | `v-memo` dependency list |
|---|---|---|---|
| Rule | `:1815-1828` | `:1817` | `[s, ruleClassForRow(s.id), selectedRowClassById.get(s.id), s.id === stripAnchorSegmentId]` (`:1819`) |
| Number | `:1837-1849` | `:1839`, `(s,i)` | `[s, i, selectedRowClassById.get(s.id), s.id === stripAnchorSegmentId]` (`:1841`); renders `{{ i + 1 }}` |
| Source (`ref="colSrc"`) | `:1852` | `:1854` | `:1856-1868`: `s`, selectedRowClass, strip flag, `cutOffsetsOf`, `glossaryTermsOf`, `concordanceSourceHighlight`, `trailingImagesOf`, `editorChapterAssets` (first segment only), `editorAssetsDir`, `showHanViet`, `viewMode` |
| Target (`ref="colTgt"`) | `:1977` | `:2017`, `(s,i)` | `:2019-2027`: `s`, `i`, selectedRowClass, strip flag, `rowErrorTextById.get`, `editorEditedShape.get`, `ruleOf(s.id)` |

- Source column details: nested `v-for` loops at `:1895` (leading images, key `'lead-'+asset_id`), `:1929` (pieces, `:key="i"`) and `:1958` (trailing images). The strip pad is at `:1967`.
- Target column: inside the memo the cell's `:class` reads `editing: editorCaretSegmentId === s.id` (`:2037`). The caret itself is not in the memo list; it reaches the memo only through `ruleOf(s.id)`. The cell also reads `rowErrorLabelById` and `unlinkedIds` (`:2035`, `:2045`), `cellLabelOf` (`:2049`) and `aria-details` (`:2051`).
- Each row emits `div.strip-pad v-if="s.id === stripAnchorSegmentId"` (`:1827`, `:1847`, `:2053`).
- `#strip-track` at `:2058-2068` sits outside any memo. It reads `stripAnchorIndex` for `:class` and `:style gridRow` and contains `TmFuzzyStrip`, `GlossaryConfirmStrip` and `ProofreaderStrip`.

**Subgrid CSS**
- `.grid` at `GridPanel.vue:2193 @ cc60020` sets the four grid columns.
- `.col { grid-template-rows: subgrid; grid-row: 1 / -1 }` at `:2209 @ cc60020` (`:2250` @ b2874aa).
- Comment at `:2323-2324 @ cc60020`: every rule keeps a 3 px border, because changing the border width re-lays out the whole subgrid on WebKit once per caret move.

**Tree B (`b2874aa`)**
- `div.grid ref="gridRoot"` at `:1837`.
- Outer `v-for="block in gridBlockViews" :key="block.index"` at `:1839-1844` creates `div.grid-block`. Each block has its own `:style` row count: `block.segments.length + (stripRowIn(block)<0?0:1)`.
- Blocks come from `src/panels/gridBlocks.ts:1-11 (GRID_BLOCK_ROWS = 200, gridBlocksOf) @ b2874aa`.
- The four inner `v-for` loops now iterate `block.segments`:

| Column | `v-for` @ b2874aa | `v-memo` @ b2874aa |
|---|---|---|
| Rule | `:1852` | `:1854`, same list as A |
| Number | `:1874`, `(s,j)` | `:1876`, uses `block.start + j` |
| Source | `:1889` | `:1891-1903`, same list as A |
| Target | `:2052`, `(s,j)` | `:2054-2062`, uses `block.start + j` |

- The `editing` class is at `:2072`.
- Template refs: `colSrc` and `colTgt` are bound through function refs only on block 0 (`:1887`, `:2013`). The callbacks are `setColSrcRef`, `setColTgtRef` and `setStripTrackRef` at `:598-603`.
- `#strip-track` at `:2093-2105` is `v-if="block.index === stripBlockIndex"` and uses `stripRowIn(block)` for its class and row.
- CSS: `.grid` holds `--grid-columns` and `.grid-block` is the grid (`:2230-2238`); `.col` is unchanged (`:2250`).
- `proofreadHighlight.ts` changes its selector to `[data-col="tgt"][data-segment-id=…]` (`src/panels/proofreadHighlight.ts:124 (repaintProofreadHost) @ b2874aa`).
- The restore and repaint watchers now watch `gridRoot`, where A watched `colTgt`: A `:1094`, `:1123` → B `restoreEditedText` and `repaintProofread` (watchers just after `:1111` and `:1152`).

## Q4: what reads the caret and strip-anchor state and is a `GridPanel` render dependency
- `GridPanel.vue:106, 121 (imports editorCaretSegmentId, setEditorCaret)` and `:131 (inlineStripAnchorSegmentId) @ cc60020`.
- Reads that happen during render:
  - `ruleOf` (`:296`, through `ruleClassForRow` in the rule-column memo at `:1819` and the target-column memo at `:2026`).
  - `cellLabelOf` (`:1708`).
  - `editing: editorCaretSegmentId === s.id` (`:2037`).
  - `stripAnchorSegmentId` in all four memo lists and in the strip-pad `v-if`s (`:1819`, `:1827`, `:1841`, `:1847`, `:1859`, `:1967`, `:2023`, `:2051`, `:2053`).
  - `stripAnchorIndex` in `div.grid :style` (`:1808`) and in strip-track (`:2062-2063`).
- At B the same reads apply, plus `stripRowIn` and `stripBlockIndex` (`b2874aa:620`, `:625`, `:1843`, `:2094-2099`).
- `GridPanel.vue:2602-2611 @ cc60020` (CSS comment): the `editing` background is painted on the target cell only, to avoid five parallel `editorCaretSegmentId === s.id` comparisons.

## Q5: why `v-memo` exists (A)
- Nothing in the code or its comments explains `v-memo`. A search for `memo` in the script/CSS part of `GridPanel.vue @ cc60020` found nothing related.
- The reason is in commit `43729b3` (2026-10-23), which touches `src/panels/GridPanel.vue`. Its message says:
  - Moving the caret at 9,850 segments took about 600–740 ms in the debug build. About 390 ms was WebKit relaying out the subgrid when a rule `v-if` inserted a `div.rule`, and about 225 ms was `PanelFrame` rebuilding the grid slot.
  - The fix made the rule always present, added "`v-memo` ở bốn cột" (`v-memo` on the four columns), and kept every rule at 3 px.
  - It added `tests/frontend/gridCaretMoveCost.test.ts` and `tests/frontend/gridRuleStable.test.ts`.
- The later commit `043f3bc` (between `43729b3` and A) explains `editorEditedShape`: the grid is notified only when a segment's emptiness changes.
- The 3 px border rationale is also in the code at `GridPanel.vue:2323-2324 @ cc60020`.

## Q6: tests
- There are no test files under `src/panels/`.
- Under `tests/frontend/` at `cc60020`, 30 test files plus `support/setup.ts` mention `GridPanel` (outside my area, so I did not read them). The grid-specific ones:
  - `gridCaretMoveCost.test.ts`
  - `gridRuleStable.test.ts`
  - `gridTypingCost.test.ts`
  - `gridRestoreEditedText.test.ts`
  - `gridPanelStripTrack.test.ts`
  - `gridPanelCellLabel.test.ts`
  - `gridPanelImages.test.ts`
  - `gridPanelRowErrorPriority.test.ts`
  - `gridPanelTmFilled.test.ts`
  - `gridPanelHanVietTabFocus.test.ts`
  - `editorCaretAtCellStart.test.ts`
- B changes only `tests/frontend/proofreadHighlight.test.ts` (+1 line). It adds no tests for `gridBlocks.ts` or for the block template.

## Searched, nothing found
- `v-memo` outside the template, and `onBeforeUpdate`, `onUpdated`, `renderTracked`, `renderTriggered` in `src/panels/GridPanel.vue @ cc60020`: none.
- A literal `Fragment` in `src/panels @ cc60020` or `@ b2874aa`: none. Fragments come only from the `<template v-for>` loops above.
- A test for `gridBlocks` at `b2874aa`: none in the diff.
