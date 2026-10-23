Digest for Q2: how the grid template builds rows, trees A (cc60020) and B (b2874aa).

Method: I read the template through git at both shas and compiled both `GridPanel.vue` templates with `@vue/compiler-sfc` 3.5.40. The output is in `/private/tmp/claude-501/-Users-hoangnam-LocalSites-addon-AuraTranslate/38ff5937-f6f4-47cd-9033-1ffaf37e4590/scratchpad/RA.js` and `RB.js`. That compile has no binding metadata, so references print as `_ctx.x`; the real script-setup build reads setup state, with the same reactive semantics. Line numbers below are the .vue source.

## Tree A (cc60020)

**Nesting from root to one cell vnode** (`src/panels/GridPanel.vue`):
- `PanelFrame` component, with the template as its default slot (:1740).
- Inside the slot, `div#grid-tabpanel.grid-scroll` under `v-if="hasSegments"` (:1799).
- Then `div.grid` (:1808), an element with `:style` gridTemplateRows. Its patch flag is 4 STYLE.
- Then `div.col.col-rule` (:1815), `div.col.col-num` (:1837), `div.col.col-src` (:1852, `ref="colSrc"`) and `div.col.col-tgt` (:1977, `ref="colTgt"`). All four are plain elements, flag 544 NEED_PATCH. The rule and num columns' flags weren't read in the compiled output, which printed only the src and tgt column flags.
- Inside each column, a keyed Fragment from `renderList`, flag `128 KEYED_FRAGMENT` (`RA.js:141`, `:310`).
- Each item is a per-row Fragment `createElementBlock(Fragment, {key: s.id}, [...], 64 STABLE_FRAGMENT)` (`RA.js:144-157`). `<template v-for>` compiles to this Fragment.
- Each row Fragment holds the cell `div.cell.cell-*` (for example `div.cell-tgt`, :2030) followed by `div.strip-pad` under `v-if="s.id === stripAnchorSegmentId"`. The `v-if` gives a comment vnode when false.
- `div.strip-track` (:2058) is a sibling after `col-tgt`, outside every `v-for`.
- There are no child components inside a row. Components sit only in these places:
  - `SourceHanViet`, inside the src cell under `v-if="showHanViet"` (:1907).
  - `ChapterImage`, inside the src cell's image figures (:1896, :1959).
  - `PanelFrame`.
  - `TmFuzzyStrip`, `GlossaryConfirmStrip` and `ProofreaderStrip`, in the strip track.

**Keys:** every column row is keyed `:key="s.id"` (:1818, :1840, :1855, :2018). The src cell's sub-lists use `'lead-' + asset.asset_id` (:1895), `i` for pieces (:1929) and `asset.asset_id` (:1958).

**Memo lists, verbatim:**
- Rule, :1819: `[s, ruleClassForRow(s.id), selectedRowClassById.get(s.id), s.id === stripAnchorSegmentId]`
- Num, :1841: `[s, i, selectedRowClassById.get(s.id), s.id === stripAnchorSegmentId]`
- Src, :1856-1868:
```
[s, selectedRowClassById.get(s.id), s.id === stripAnchorSegmentId, cutOffsetsOf(s.id), glossaryTermsOf(s.id),
 concordanceSourceHighlight?.segmentId === s.id ? concordanceSourceHighlight : null, trailingImagesOf(s.id),
 s.id === firstSegmentId ? editorChapterAssets : null, editorAssetsDir, showHanViet, viewMode]
```
- Tgt, :2019-2027: `[s, i, selectedRowClassById.get(s.id), s.id === stripAnchorSegmentId, rowErrorTextById.get(s.id), editorEditedShape.get(s.id), ruleOf(s.id)]`

**Compiled shape of the column loop (load-bearing):**
- It is `renderList(editorSegments, (s, i, _, _cached) => { const _memo = [...]; if (_cached && _cached.el && _cached.key === s.id && _isMemoSame(_cached, _memo)) return _cached; const _item = (openBlock(), createElementBlock(Fragment,{key:s.id},[...],64)); _item.memo = _memo; return _item }, _cache, N)` (`RA.js:141-160`).
- The memo array is evaluated on every parent render, for every row, before the comparison. `_cache` is the component render cache, passed as the loop's cache slot.
- The cache index per loop is rule 7, num 7, src 13 and tgt 15 (`RA.js:160`, `:310-361`). The src and num values weren't read from the compile output, so treat them as unverified; only the rule/tgt `7`/`15` and the shape are confirmed.
- The loop wrapper is `openBlock(true)` with `128 KEYED_FRAGMENT`, so tracking is disabled for the loop's children.
- A memo hit returns the same vnode object, so the patch of that row is skipped.

**Reactive reads inside a row but NOT in its memo list.** These are read only on a memo miss:
- Rule cell: `unlinkedIds` (`.has(s.id)`), `ruleSlotClass(ruleClassForRow(s.id))` (:1825, covered by the memo through `ruleClassForRow`), and `s.is_paragraph_end`, which is covered by `s`.
- Num cell: only `s` fields.
- Src cell:
  - `cutCountOf(s.id)` (:1880).
  - `t(ROLE_LABEL_KEYS…)`, which reads the locale.
  - `leadingImagesOf(s.id)` (:1895).
  - `sourcePieceInfoOf(s.id, s.source_text)` (:1929).
  - `unlinkedIds` is not read in the src cell.
- Tgt cell:
  - `rowErrorLabelById.has(s.id)` (:2035).
  - `textShapeOf(s.target_text)` (the `?? ` fallback after `editorEditedShape.get`).
  - `editorCaretSegmentId === s.id` (:2037, the `editing` class).
  - `unlinkedIds.has(s.id)` (:2045).
  - `cellLabelOf(s.id, i)` (:2049, defined :1707). This reads `ruleOf` and so `editorCaretSegmentId` and `restRuleById`, then `t()`, `unlinkedIds`, and `rowErrorTextById`.
  - `s.id === stripAnchorSegmentId` (also in the memo).
- The tgt `editing` class reads `editorCaretSegmentId` directly, but `ruleOf(s.id)` in the memo returns `'primary'` exactly when `editorCaretSegmentId === s.id` (:296-298). So the memo flips for the row the caret leaves and the row it enters, which keeps `editing` correct (**load-bearing**).

**Reads outside the per-row memos (A):**
- Grid level (:1808): `editorSegments.length` and `stripAnchorIndex`, which is a computed doing `editorSegments.findIndex` over `inlineStripAnchorSegmentId`. The `:style` is recomputed on every render.
- Column level: `sourceTokenClass` on col-src (a computed over `isChinese`, :416).
- Strip track (:2058-2063): `stripAnchorIndex` for the class and the `gridRow` style.
- The memo arrays themselves read, for every row, `ruleClassForRow(s.id)` and `ruleOf(s.id)`. Each calls `ruleOf`, which reads `editorCaretSegmentId.value` and then `restRuleById.value.get(id)` (:296-302). This is the key fact for the caret question (**load-bearing**):
  - The component render effect subscribes to `editorCaretSegmentId`, in addition to the per-row memo reads.
  - `restRuleById` depends on `editorEditedShape`, `editorTmFilledSegmentIds` and `editorSegments` (:281-293).
- Other memo-array reads per row: `selectedRowClassById` (a computed over `selectedSegmentIds`, :324), `rowErrorTextById` (:1692), `editorEditedShape`, `cutOffsetsOf`, `glossaryTermsOf`, `trailingImagesOf`, `concordanceSourceHighlight`, `firstSegmentId`, `editorChapterAssets`, `editorAssetsDir`, `showHanViet` and `viewMode`.
- What changes when the caret moves:
  - `editorCaretSegmentId` changes. Every row evaluates `ruleOf`, so the whole component re-renders and walks all rows' memo arrays. Only the leaving and entering rows get a different `ruleOf`, so only those miss.
  - `inlineStripAnchorSegmentId` (`src/inlineStripEligibility.ts:61-73`) returns `editorCaretSegmentId.value` for the `glossary_confirm` case when `confirmStripIsOpen` and for the `proofreader` / `proofreader_summary` case when `proofreaderStripPlace === 'track'` (the `proofreaderStripPlace` computed itself reads `editorCaretSegmentId`). For `tm_fuzzy` and the `glossary_confirm` notice it uses other ids. When one of the caret-anchored strips wins, a caret move changes the anchor, so `stripAnchorIndex`, the grid `:style`, the strip-track style and the `s.id === stripAnchorSegmentId` memo entries all change. That adds the strip-pad `v-if` for the two rows involved.

## Tree B (b2874aa)

`git diff cc60020 b2874aa -- src` changes only `GridPanel.vue`, `gridBlocks.ts` (new) and one line in `proofreadHighlight.ts`.

**Nesting:**
- `PanelFrame`, then `div#grid-tabpanel.grid-scroll` (`v-if="hasSegments"`, :1828).
- Then `div.grid ref="gridRoot"` (:1837). It is static (`class:"grid"`, hoisted), no `:style`. Its compiled flag is 512 NEED_PATCH (`RB.js:~383`).
- Then the block loop, `renderList(gridBlockViews, block => createElementBlock("div", {key: block.index, class:"grid-block", style: normalizeStyle(...)}, [...], 4 STYLE))`, wrapped in `openBlock(true)` with `128 KEYED_FRAGMENT` (`RB.js:140-144`, `:381`). This is the **plain keyed loop**. It has no `v-memo`, so there is no `_memo` and no `_cache` slot at block level (**load-bearing**).
- Each block element is a plain `div.grid-block` element, key `block.index`. Its `:style` is `gridTemplateRows: repeat(block.segments.length + (stripRowIn(block) < 0 ? 0 : 1), auto)` (:1842).
- Inside each block: `div.col-rule`, `div.col-num`, `div.col-src` and `div.col-tgt` (all elements), then the strip-track.
- Each column holds the same keyed Fragment loop over `block.segments`, with a per-row Fragment of 64 STABLE_FRAGMENT wrapping the cell and the strip-pad `v-if`, as in A.

**Per-row memo inside blocks (load-bearing):** each column loop is still `v-memo`'d per row, with the same `_cached`/`_isMemoSame` pattern. The compiled cache indexes are `_cache, 7` (rule, `RB.js:167`), `9` (num, `:189`), `13` (src, `:302`) and `15` (tgt, `:362`), now inside the block callback.

**Memo-list differences from A:**
- Rule (:1854): identical to A.
- Num (:1876): `[s, block.start + j, selectedRowClassById.get(s.id), s.id === stripAnchorSegmentId]`. The row number is `block.start + j` instead of `i`.
- Src (:1891-): identical to A. The list was only shown up to `editorAssetsDir` in what I read, with the closing `]` on line 1903.
- Tgt (:2054-2062): `[s, block.start + j, selectedRowClassById.get(s.id), s.id === stripAnchorSegmentId, rowErrorTextById.get(s.id), editorEditedShape.get(s.id), ruleOf(s.id)]`.

**Cache indexes and block parameters:** the cache index is per loop callsite. All blocks share the same `_cache` slots 7/9/13/15 and the same `_cache` array. The caches are keyed by `s.id` through each vnode's `.key` check, not by block (`_cached.key === s.id`).

**Refs:**
- Col-src and col-tgt are now repeated per block. Their refs are function refs set only on block 0: `:ref="block.index === 0 ? setColSrcRef : undefined"` (:1887, :2013), compiled with `ref_for: true`.
- The strip track uses `:ref="setStripTrackRef"` (:2096).
- `setColSrcRef`, `setColTgtRef` and `setStripTrackRef` (:601-603) assign to `shallowRef`s.
- `gridRoot` (`useTemplateRef`, :597) is the only template ref on a stable element.

**Block data:**
- `gridBlockViews` (:611) is a computed over `editorSegments.value` only. It builds `{index, start, segments: segments.slice(start, start+rows)}` per block, so every block object is new whenever `editorSegments` changes. The segment objects `s` are the same references.
- `gridBlocksOf` (`src/panels/gridBlocks.ts:5-11`) uses `GRID_BLOCK_ROWS = 200`.
- Per-row `s` identity is preserved, so the memo's `s` entry still matches across a `gridBlockViews` recompute.

**Reads outside the row memos (B):**
- Block level, per block on every render:
  - `block.index`, `block.segments.length`, and `stripRowIn(block)` (:620-623), which reads `stripAnchorIndex.value`. `stripAnchorIndex` is a computed over `inlineStripAnchorSegmentId` and `editorSegments`.
  - `sourceTokenClass` is read once per block for col-src.
  - `block.index === 0` for the refs.
  - `block.index === stripBlockIndex` (:2094). `stripBlockIndex` (:625-628) is a computed over `stripAnchorIndex` and `GRID_BLOCK_ROWS`, giving 0 when the anchor is < 0.
- Strip track (:2093-2105): `v-if` on `stripBlockIndex`, `stripRowIn(block)` twice for the class and style. It renders in only one block.
- Grid level: nothing reactive. `div.grid` has no `:style` any more.
- Memo-array reads inside blocks are the same as A. Each row's memo array still reads `ruleOf` and `ruleClassForRow`, and so `editorCaretSegmentId`. So in B the component render still subscribes to `editorCaretSegmentId` and the render function still walks all blocks and rows on a caret move. All 50 blocks' elements and column vnodes are rebuilt each render, because the block loop has no memo (**load-bearing**).
- What changes on a caret move: same as A, `editorCaretSegmentId`, and when a caret-anchored strip wins, `stripAnchorSegmentId` / `stripAnchorIndex`. The latter changes `stripRowIn(block)` for the block(s) containing the old and new anchor (its `gridTemplateRows` style plus the strip-pad rows), and `stripBlockIndex`, which moves the strip-track `v-if` between blocks. Blocks the anchor never touches get an equal `gridTemplateRows` string, so their patch is a no-op.

## What I did not find or did not read
- I did not read the bodies of `cutOffsetsOf`, `glossaryTermsOf`, `trailingImagesOf`, `leadingImagesOf`, `sourcePieceInfoOf`, `selectedSegmentIds` or `textShapeOf` for their own deps. The per-row read lists for these are call-site level only (`GridPanel.vue` :381-386, :443-451, :475, :561).
- I did not read B's :1900-1903 tail of the src memo list. Per the diff stat, B only reshapes the loops. The src list's last entries are assumed to match A's (`showHanViet`, `viewMode`).
- Search scope: `git show` / `git grep` at cc60020 and b2874aa, plus the compile of the template only. File count: 5 read (`GridPanel.vue` at both shas, `gridBlocks.ts`, `inlineStripEligibility.ts`, and the compiled output).
