# Analyzer digest — Q3 @ 99f3fc1

Short answer: a GridBlock re-renders when (a) one of its five props from GridPanel changes, or (b) a reactive value its own render read changes (injected context refs, module state incl. `editorEditedShape`). Every mounted GridBlock render reads `editorEditedShape` for every row in the col-tgt `v-memo` list, so `triggerRef(editedShape)` re-runs the render function of every mounted GridBlock; inside each, `v-memo` reuses cached vnodes for rows whose memo array is unchanged.

Files: 8 of 15 (read GridBlock.vue, gridBlocks.ts, gridBlockContext.ts, GridPanel.vue, editorPanelState.ts, package.json; grep editorSegments.ts, PanelFrame.vue). "(Vue runtime)" = not repo code, vue 3.5.40 (`package.json:37 @ 99f3fc1`).

## 1. Block list and props
- `src/panels/gridBlocks.ts:1 GRID_BLOCK_ROWS @ 99f3fc1` 200 rows; `:5-11 gridBlocksOf` cuts `total` into `{start, rows}`.
- `src/panels/gridBlockContext.ts:15-20 GridBlockView @ 99f3fc1` `{index, start, segments, rules}`.
- Load-bearing: `src/panels/GridPanel.vue:1918-1926 @ 99f3fc1` `<GridBlock v-for="block in gridBlockViews" :key="block.index">` with five props: `block`, `caret-id` = `caretIdIn(block)`, `anchor-id` = `anchorIdIn(block)`, `strip-row` = `stripRowIn(block)`, `has-strip-track` = `block.index === stripBlockIndex`.
- Helpers `src/panels/GridPanel.vue @ 99f3fc1`: `:645-648 stripRowIn` anchor index in block or -1; `:650-652 anchorIdIn` `stripAnchorSegmentId` or null; `:659-662 caretIdIn` `editorCaretSegmentId` only for block holding `caretIndex`; `:664-667 stripBlockIndex` `floor(anchor/200)` or 0.
- (Vue runtime) child updated by parent only if a prop differs `!==`; caret/anchor move changes props on at most two blocks.
- GridBlock inside PanelFrame default slot (`src/panels/GridPanel.vue:1849`; `src/panels/PanelFrame.vue:174 <slot /> @ 99f3fc1`). (Vue runtime) reads in that `v-for` tracked by the render effect running the slot; `src/panels/editorPanelState.ts:679-680` calls it "the whole grid slot".

## 2. `gridBlockViews` identity (load-bearing)
`src/panels/GridPanel.vue:615-643 gridBlockViews @ 99f3fc1`:
- Reads `editorSegments` (`toRaw`), `editorEditedShape.value` (`:617`), `editorTmFilledSegmentIds.value` (`:618`).
- `idsWithChangedRule` (`:599-613`) compares shapes/tm-filled with `rulesBasis` (`:594-597`), WeakMap previous views → copy of shapes (`new Map(shapes)` `:641`), copy needed because `editedShape` mutated in place.
- Reuse previous view (`:621-632`) if `start`/length match, every segment `===`, no id in `changed`; first run `changed === null` ⇒ no reuse.
- Else rebuild `rules` via `restRulesOf` (`:278-293`); still return `before` if same segments and `sameRules` (`:295-299`, `:635-637`); else new view (`:639`).
- So after a shape-changing keystroke the block keeps the same `block` object unless its rule value changed (e.g. none → draft).
- Computed always returns fresh array (`:620`, `:642`); (Vue runtime) counts as change ⇒ slot `v-for` re-runs, props compared per block.
- `restRulesOf` passes `null` caret (`:285`); `src/panels/GridBlock.vue:74-76 ruleOf @ 99f3fc1` `props.caretId === id ? 'primary' : block.rules.get(id) ?? 'none'`.

## 3. Reactive reads besides props
- Context injected `src/panels/GridBlock.vue:25-55 @ 99f3fc1`, provided `GridPanel.vue:1816-1845`; missing ⇒ throws `'GridBlock needs the grid block context'` (`:26`).
- Module state imported: `editorAssetsDir`, `editorChapterAssets`, `editorEditedShape` (`:9`), `concordanceSourceHighlight` (`:8`), `viewMode` (`:15`).
- Context refs: `selectedRowClassById`, `unlinkedIds`, `rowErrorLabelById`, `rowErrorTextById`, `firstSegmentId`, `showHanViet`, `sourceTokenClass`; helpers read own computeds: `cutOffsetsOf` → `pendingCuts`, `glossaryTermsOf` → `glossaryMarksPerSegment`, `trailingImagesOf` → `trailingImagesBySegmentId` (`GridPanel.vue:426-473`, `:355-371`).
- (Vue runtime) read in render subscribes that GridBlock's render effect; `v-memo` expression evaluated every render even on cache hit ⇒ memo reads stay subscribed for every block; (3.5) reads only inside skipped rows not re-collected.

## 4. `v-memo` per column (row `v-for` over `block.segments`, key `s.id`) — `src/panels/GridBlock.vue @ 99f3fc1`
| Column | Lines | Memo | Read in row, not in memo |
|---|---|---|---|
| col-rule | `:110-122`, memo `:113` | `[s, ruleClassForRow(s.id), selectedRowClassById.get(s.id), s.id === anchorId]` | `unlinkedIds.has(s.id)` (`:117`) |
| col-num | `:132-142`, memo `:135` | `[s, block.start + j, selectedRowClassById.get(s.id), s.id === anchorId]` | none |
| col-src | `:147-262`, memo `:150-162` | `[s, selectedRowClassById.get(s.id), s.id === anchorId, cutOffsetsOf(s.id), glossaryTermsOf(s.id), concordanceHighlightOf(s.id), trailingImagesOf(s.id), chapterAssetsOf(s.id), editorAssetsDir, showHanViet, viewMode]` | `cutCountOf` (`:174`), `leadingImagesOf` (`:189`), `sourcePieceInfoOf` (`:223`), `ROLE_LABEL_KEYS`/`t()` (`:184`) |
| col-tgt (load-bearing) | `:308-346`, memo `:311-319` | `[s, block.start + j, selectedRowClassById.get(s.id), s.id === anchorId, rowErrorTextById.get(s.id), editorEditedShape.get(s.id), ruleOf(s.id)]` | `rowErrorLabelById.has` (`:327`), `unlinkedIds.has` (`:337`, `cellLabelOf` `:88`), `caretId === s.id` (`editing`, `:329`) |
- `defineExpose({ concordanceHighlightOf, chapterAssetsOf })` only so vue-tsc counts them (`:94-95`).
- Stable identities: `cutOffsetsOf` → `EMPTY_CUTS` (`GridPanel.vue:433-436`); `trailingImagesOf` → `NO_IMAGES` (`:367-371`); `glossaryMarksPerSegment` returns `previous` when JSON-equal (`:460-468`); `selectedRowClassById` new Map each time (`:321-328`) but memo compares `.get` string.
- Memo-hit consequences: `unlinkedIds` alone, `rowErrorLabelById` alone (unless `rowErrorTextById` `:1781-1783` changes), locale change in `t()` do not patch hit rows.
- Row miss produces the column's cell div(s) plus `strip-pad` when `s.id === anchorId` (`:121`, `:141`, `:261`, `:345`). col-tgt cells static `contenteditable="true"` (`:338`), text `{{ s.target_text }}` (`:344`); typed text never reaches template.

## 5. Block-level output outside memo
- `.grid-block` root `gridTemplateRows: repeat(segments.length + (stripRow < 0 ? 0 : 1), auto)` (`src/panels/GridBlock.vue:99-102`).
- col-src `:class="sourceTokenClass"` (`:146`); col-tgt column-level handlers (`:271-281`).
- Strip track (`:350-362`) `v-if="hasStripTrack"`: `:ref="setStripTrackRef"` (`GridPanel.vue:586`), `strip-track-off` when `stripRow < 0` else `gridRow: stripRow + 2`; children `<TmFuzzyStrip/>`, `<GlossaryConfirmStrip/>`, `<ProofreaderStrip place="track"/>` without changing props; (Vue runtime) GridBlock re-render does not re-render them.

## 6. Keystroke path and `editedShape` (load-bearing)
- `src/panels/editorPanelState.ts:682-688 @ 99f3fc1` — `editedText`, `editedShape` both `shallowRef(Map)`; "Written in place without triggering"; "The template reads `editedShape`"; "Triggers only when a row's effective shape changes".
- `src/panels/editorPanelState.ts:786-793 @ 99f3fc1` — `shape = textShapeOf(targetText)`, `before` stored or loaded shape; `editedText.value.set` no trigger; `shapes.set` in place; `if (shape !== before) triggerRef(editedShape)`.
- `TextShape = 0 | 1 | 2` empty/whitespace/text (`src/panels/editorSegments.ts:178-183 textShapeOf @ 99f3fc1`) ⇒ trigger only on those transitions, not every keystroke.
- `src/panels/editorPanelState.ts:1001` `editedShape.value = new Map()` (reassignment triggers).
- When `triggerRef(editedShape)` fires: (a) every mounted GridBlock re-runs render (each read `editorEditedShape.get(s.id)` per row at `GridBlock.vue:317`); each evaluates ~200 rows × 4 memo arrays incl. `ruleClassForRow`, `cutOffsetsOf`; col-tgt misses only the changed row (plus `ruleOf` changes if `block.rules` new); col-rule misses rows whose `ruleClassForRow` changed; col-num/col-src all hit. Missed col-tgt row recomputes `empty` (`:328`) and `aria-label` via `cellLabelOf`. (b) `gridBlockViews` recomputes (`GridPanel.vue:617`), slot `v-for` re-runs; only block with new view gets new `block` prop; (Vue runtime) merges with update queued in (a).

## Looked for, not found (`git grep` @ 99f3fc1)
- Reads of `editorEditedShape` other than `GridBlock.vue:317`, `:328`, `GridPanel.vue:617`.
- Template read of `editorEditedText`/`editedText` in GridBlock.
- `shallowReactive` or per-row ref splitting of shape state.
- `v-once`; `v-memo` on `<GridBlock>` `v-for` (`GridPanel.vue:1918-1926`); `memo`/`markRaw` around block views.
- Not read: PanelFrame.vue beyond `<slot />`; TmFuzzyStrip / GlossaryConfirmStrip / ProofreaderStrip.
