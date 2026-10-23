Digest for Q4. A = cc60020, B = b2874aa. I read code only and ran nothing. `src/` was searched excluding tests.

**State written**
- `src/panels/editorPanelState.ts:281-296 (setEditorCaret) @ A` (same at B) writes two refs synchronously: `caretSegmentId` (`:283`) and `lastCaretSegmentId` (`:284`, only when id is non-null). Both are declared at `:148-149`. They are exported read-only as `editorCaretSegmentId` (`:177`) and `editorLastCaretSegmentId` (`:179`).
- The rest of `setEditorCaret` writes no other Vue ref.
  - `:286` calls `void flushEditorNow()` when the caret moves from one non-null id to a different one.
  - `:292-295` calls `positionFlush.markMoved(...)` and `armPositionFlushTimer()`. I did not trace these further.
- `flushEditorNow` (`:835`) writes no ref before its first `await`. It reads `inFlight` and `flush`, calls `clearFlushTimer()`, and sets `inFlight`. `flushError` (`:873`, `:877`) and `lastSavedAt` (`:879`) are written only after `await saveSegmentTargets` (`:859`), so asynchronously.
- Nothing in `GridPanel` watches the caret: no `watch` on `editorCaretSegmentId` or `stripAnchorSegmentId`.

**Strip-anchor chain** (computeds)
- `src/inlineStripEligibility.ts:29-48 (eligibleInlineStrips) @ A,B` reads `editorCaretSegmentId.value` at `:34`. It also reads `quickAddIsOpen`, `confirmStripIsOpen`, `confirmStripNoticeShown`, the `proofread*` refs, `tmFuzzyIsEligible` and `proofreadRunSegmentIds`.
- `src/inlineStripEligibility.ts:53-58 (proofreaderStripPlace) @ A,B` reads the caret at `:57`.
- `src/inlineStripEligibility.ts:61-73 (inlineStripAnchorSegmentId) @ A,B` reads the caret at `:66` and `:69`, and `tmFuzzySegmentId` at `:64`.
- `src/panels/GridPanel.vue:600 (stripAnchorSegmentId) @ A` and `:605 @ B` is a computed that returns `inlineStripAnchorSegmentId.value`.
- `src/panels/GridPanel.vue:601-604 (stripAnchorIndex) @ A` and `:606-609 @ B` reads `stripAnchorSegmentId` and `editorSegments`, and does a `findIndex` over all segments.
- `src/panels/GridPanel.vue:296-298 (ruleOf) @ A` and `:297-298 @ B` reads `editorCaretSegmentId.value` at `:297`.
- `src/panels/GridPanel.vue:300-301 (ruleClassForRow) @ A` and `:301-302 @ B` calls `ruleOf`.
- `src/panels/GridPanel.vue:1707-1708 (cellLabelOf) @ A` and `:1736-1737 @ B` calls `ruleOf` at `:1708` / `:1737`.
- B only: `src/panels/GridPanel.vue:620-623 (stripRowIn)` reads `stripAnchorIndex.value`.
- B only: `src/panels/GridPanel.vue:625-628 (stripBlockIndex)` reads `stripAnchorIndex.value`.

**Reads in the GridPanel template at A**

*Once per render (not inside any v-memo list):*
- `:1808 (stripAnchorIndex)`: the `.grid` `:style` reads it at `repeat(${editorSegments.length + (stripAnchorIndex < 0 ? 0 : 1)}, auto)`.
- `:2062-2063 (stripAnchorIndex)`: `#strip-track` `:class` and `:style` read it.
- `:2066-2068`: `TmFuzzyStrip`, `GlossaryConfirmStrip` and `ProofreaderStrip` are always rendered. Their own reads are listed below.

*Inside a per-row `v-memo` template (the v-memo list is evaluated once per row per render; the body runs only when a listed value changed):*
- Col-rule, `v-memo` at `:1819`: the list reads `ruleClassForRow(s.id)` (so `ruleOf`, so the caret) and `s.id === stripAnchorSegmentId`.
  - In the body, `:1825` calls `ruleClassForRow` again and `:1827` reads `stripAnchorSegmentId`.
- Col-num, `v-memo` at `:1841`: the list reads `s.id === stripAnchorSegmentId`. The body reads it at `:1847`.
- Col-src, `v-memo` at `:1856-`: the list reads `s.id === stripAnchorSegmentId` at `:1859`. The body reads it at `:1967`.
- Col-tgt, `v-memo` at `:2019-2027`:
  - The list reads `s.id === stripAnchorSegmentId` at `:2023` and `ruleOf(s.id)` at `:2026` (the caret).
  - The list does not contain `editorCaretSegmentId` itself.
  - The body reads the caret at `:2037` (`editing: editorCaretSegmentId === s.id`), which is not in the memo list.
  - The body also reads `cellLabelOf(s.id, i)` at `:2049` (so `ruleOf`), `stripAnchorSegmentId` at `:2051`, and `stripAnchorSegmentId` at `:2053`.
  - The `:2037` and `:2049` reads count as render reads, but they run only when the memo list changed. They are kept in step by `ruleOf(s.id)` in the list at `:2026`.
- The col-src list and body also hold `ChapterImage` at `:1896` and `:1959`, and `SourceHanViet` at `:1907`. `PanelFrame` is at `:1740`.
  - No caret or strip-anchor read exists in `SourceHanViet`, `ChapterImage` or `PanelFrame`. `git grep` of the caret and anchor symbols over `src` @ A,B finds none in them.

**B differences in GridPanel**
- Line numbers shift to: `ruleOf` `:297`, `ruleClassForRow` `:301`, `cellLabelOf` `:1736`, `stripAnchorSegmentId` `:605`, `stripAnchorIndex` `:606`.
- The single `.grid` is now `v-for="block in gridBlockViews"` (`:1838-1843`, with `gridBlockViews` at `:611-618`). `gridBlockViews` itself does not read the caret or anchor.
- Per block, once per render of that block (not inside a `v-memo`):
  - `:1842 (stripRowIn)` reads it in `gridTemplateRows`.
  - `:2094 (stripBlockIndex)` reads it in `v-if="block.index === stripBlockIndex"`.
  - `:2098` and `:2099 (stripRowIn)` read it for the class and style, so `stripRowIn` is called three times per block, at `:1842`, `:2098` and `:2099` (twice at `:2099`).
  - `:2104` renders `ProofreaderStrip` and `:2102`-`:2103` the other two strips, only inside the block where `v-if` holds.
- Per-row `v-memo` reads at B:
  - Col-rule: list `:1854`, body `:1860` and `:1862`.
  - Col-num: list `:1876`, body `:1882`.
  - Col-src: list `:1891-` with `:1894`, body `:2002`.
  - Col-tgt: list `:2054-` with `:2058` (anchor) and `:2061` (`ruleOf`). In the body, `:2072` is the caret, `:2084` is `cellLabelOf`, `:2086` is the anchor, and `:2088` is the anchor.
- `stripAnchorSegmentId` is still tested per row in every v-memo list and body (`:1854`, `:1862`, `:1876`, `:1882`, `:1894`, `:2002`, `:2058`, `:2086`, `:2088`). The `v-memo` is the same four lists as at A, now nested in blocks.
- The CSS comment mentioning the `editorCaretSegmentId === s.id` comparisons is at `GridPanel.vue:2606 @ A` and `:2647 @ B`. It is a comment, not a read.

**Children rendered by GridPanel** (all three strips are rendered once per GridPanel render, in the strip-track div)
- `src/TmFuzzyStrip.vue:30 (watch editorCaretSegmentId) @ A,B` is a **watch, not render**. Its callback calls `syncTmFuzzyStrip(segmentId)`, with `immediate: true`.
- `src/TmFuzzyStrip.vue:32 (isVisible) @ A,B` is a computed that reads `eligibleInlineStrips`, and so the caret through `:34`. It is read in render, but I did not read the template to confirm the exact `v-if` line.
- `src/TmFuzzyStrip.vue:47-50 (currentTarget)` reads `tmFuzzySegmentId` and `editorSegments`. It does not read the caret directly.
- `src/GlossaryConfirmStrip.vue:60-66` (`watch([editorCaretSegmentId, editorSegments, glossaryMarks])`) is a **watch, not render**. It calls `syncGlossaryConfirmStripTarget`. The component's `isVisible` (`:74`) reads `confirmStripIsOpen` and `quickAddIsOpen`, not the caret.
- `src/ProofreaderStrip.vue:40 (watch editorCaretSegmentId) @ A,B` is a **watch, not render**. Its callback calls `syncProofreadStripHidden`.
- `src/ProofreaderStrip.vue:34-38 (isVisible)` reads `eligibleInlineStrips` and `proofreaderStripPlace`, so the caret through `inlineStripEligibility.ts:34` and `:57`.
- `src/ProofreaderStrip.vue:63 (isJob)` reads `eligibleInlineStrips`.
- `src/ProofreaderStrip.vue:220`, `:225` (`proofreadScopeCounts.segment`) is in the template. It resolves to `src/proofreadScope.ts:19 (proofreadScopeIds.segment)`, which reads `editorCaretSegmentId.value ?? editorLastCaretSegmentId.value`. The count is read in render, but I did not check whether a `v-if` on `isVisible` gates it.

**Other readers of the caret**
These are not GridPanel render dependencies: `StatusBar.vue:156`, `AiTranslationPanel.vue`, `AiPromptInspectorOverlay.vue`, `proofreadState.ts`, `segmentSelectionState.ts:133`, `segmentHistoryState.ts:124`, `tmSettingsState.ts:64`, `tmManageCommandDeps.ts:73`, `tmConcordanceCommandDeps.ts:35`, `aiTranslateHandlers.ts`, `main.ts:1091`.
- `src/panels/segmentSelectionState.ts:133` is the only one that might reach `GridPanel` render through `selectedRowClassById`. I did not check whether it does.

**Callers of setEditorCaret in GridPanel**
These are writes, not reads: `GridPanel.vue:854`, `:1047`, `:1053`, `:1149` @ A. At B they are `:887`, `:1080`, `:1086`, `:1180`.

**Not found or not done**
- I did not check the `GridPanel.vue` selection/click handlers (the writes above) for event-time reads of `stripAnchorIndex`, since the brief asked for render dependencies only.
- I did not read the full `TmFuzzyStrip`, `GlossaryConfirmStrip` and `ProofreaderStrip` templates, only their scripts. I also did not read the full `tmFuzzyStripState.ts` or `glossaryConfirmStripState.ts` to see whether `tmFuzzySegmentId` and `confirmStripIsOpen` are written from the caret watches. The code comments say they are.
