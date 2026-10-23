Digest: `src/` top-level files, `src/App.vue` and `src/main.ts` at cc60020

**Tree B and my area.** `git diff --stat cc60020 b2874aa`, with the four excluded paths left out, changes only `src/panels/GridPanel.vue`, `src/panels/gridBlocks.ts`, `src/panels/proofreadHighlight.ts` and `tests/frontend/proofreadHighlight.test.ts`. Nothing in my area changes, so everything below holds for both A and B.

**Q1 / Q4: caret and strip-anchor state that GridPanel reads, and where it is defined and written**

- `src/panels/editorPanelState.ts:177 editorCaretSegmentId @ cc60020`: readonly wrapper of the internal `caretSegmentId`. `:179 editorLastCaretSegmentId` and `:281 setEditorCaret` are also here. This file is outside my area; listed only for orientation.
- `src/inlineStripEligibility.ts:29 eligibleInlineStrips @ cc60020`: a computed list of the strips that could render now. It reads `editorCaretSegmentId.value` at line 34, plus `quickAddIsOpen`, `confirmStripIsOpen`, `confirmStripNoticeShown`, the proofread refs and `tmFuzzyIsEligible`.
- `src/inlineStripEligibility.ts:54 proofreaderStripPlace @ cc60020`: a computed that returns `'foot'`, `'track'` or `null`. It reads `editorCaretSegmentId`.
- `src/inlineStripEligibility.ts:61 inlineStripAnchorSegmentId @ cc60020`: the source of GridPanel's strip anchor.
  - Imported at `src/panels/GridPanel.vue:131`.
  - Wrapped as `stripAnchorSegmentId` at `GridPanel.vue:600` and `stripAnchorIndex` at `:601`.
  - Its value depends on which strip wins `topmostStrip`:
    - `tm_fuzzy`: `tmFuzzySegmentId`.
    - `glossary_confirm`: `editorCaretSegmentId`, or `confirmStripNoticeSegmentId` when only the notice is showing.
    - `proofreader` / `proofreader_summary`: `editorCaretSegmentId`, but only when the strip is placed in the track.
  - So a caret move can change `stripAnchorSegmentId`, which appears in both rows' `v-memo` lists at `GridPanel.vue:1819` and `:1841`. It also feeds `stripAnchorIndex`, which `GridPanel.vue:1808` (`gridTemplateRows`) and `:2062-2063` (strip-track class and `gridRow`) read.
- `src/tmFuzzyStripState.ts:38 tmFuzzySegmentId @ cc60020`: a computed over `shown`. `tmFuzzyIsEligible` is at line 62. `src/tmFuzzyStripState.ts:133 syncTmFuzzyStrip @ cc60020` runs `clearShown()` (line 85, `shown.value = null`) whenever the segment differs, then schedules a debounced `runScan`, which sets `shown.value` again at line 118. The net effect is that a caret move clears the anchor and later sets it again.
- `src/glossaryConfirmStripState.ts:216 confirmStripNoticeShown`, `:220 confirmStripNoticeSegmentId`, `:225 confirmStripIsOpen @ cc60020`: the anchor inputs for the glossary strip.
  - `src/glossaryConfirmStripState.ts:345 syncGlossaryConfirmStripTarget @ cc60020` is driven by the caret. It clears `undoEntry` when the caret leaves the undo segment, then calls `applyTarget`.
- `src/proofreadState.ts:276 proofreadStripHidden @ cc60020`: a computed that compares `hiddenForSegmentId` (line 274) with `editorCaretSegmentId`. It is written by `:290 syncProofreadStripHidden` and `:281 hideProofreadStrip`.
- `src/proofreadState.ts:79 proofreadResults @ cc60020`: GridPanel imports it at line 73. It is used only in a post-flush `watch` and a repaint (`GridPanel.vue:1100`, `:1123`), not in the template.

**Child components GridPanel renders (from `src/`) that watch the caret**

All three are mounted in the `#strip-track` div at `GridPanel.vue:2059-2069`.

- `src/TmFuzzyStrip.vue:30 @ cc60020`: `watch(editorCaretSegmentId, syncTmFuzzyStrip, {immediate})`. `isVisible` is at line 32, from `eligibleInlineStrips`.
- `src/GlossaryConfirmStrip.vue:60 @ cc60020`: `watch([editorCaretSegmentId, editorSegments, glossaryMarks], syncGlossaryConfirmStripTarget)`. The comment at lines 52-58 says this watch sits in this leaf component to avoid an import cycle.
- `src/ProofreaderStrip.vue:40 @ cc60020`: `watch(editorCaretSegmentId, syncProofreadStripHidden)`. `isVisible` (line 34) and `isJob` (line 63) are computeds over `eligibleInlineStrips` and `proofreaderStripPlace`.
- `src/ChapterImage.vue:38 props @ cc60020`: also a GridPanel child (`GridPanel.vue:127`). Its props are `assetsDir` and `fileName` only; it reads no caret state.

**Caret readers in my area that are not GridPanel render dependencies**

- `src/StatusBar.vue:154 positionInChapter`: caret, falling back to the last caret.
- `src/proofreadScope.ts:17 proofreadScopeIds`.
- `src/AiPromptInspectorOverlay.vue:112 isStale`.
- `src/aiTranslateHandlers.ts:45` and `:77 aiTranslateHandlers`.
- `src/main.ts:1091 boot`: calls `assembleCurrentAiPrompt` with the caret.
- Other files that reference `editorCaretSegmentId` (counts from `git grep -c`): `aiPromptInspectorState.ts` (4), `aiTranslateState.ts` (2), `aiTranslateBatchState.ts` (1), `tmConcordanceCommandDeps.ts` (2), `tmManageCommandDeps.ts` (2), `tmSettingsState.ts` (2).
- `src/App.vue:293` mounts a second `<ProofreaderStrip place="foot" />`, outside GridPanel.

**Writer of the caret in my area**

- `src/chapterSplitState.ts:80 confirmChapterSplit @ cc60020`: calls `setEditorCaret(pickedId.value)`. It is the only `setEditorCaret` call among the top-level `src/` files.

**What mounts GridPanel**

- `src/main.ts:1251 boot @ cc60020`: `createApp(App).mount('#app')`.
- `src/App.vue:283-286 @ cc60020`: `<KeepAlive>` around `LibraryMode`, `WorkspaceMode` and `ReadingMode`.
- `src/layout/WorkspaceDock.vue:33` and `:104 @ cc60020`: imports GridPanel and registers it as the `grid` panel component (outside my area; cited for the chain).

**Q2, Q3, Q5, Q6:** nothing in my area. The `v-memo` rows are at `src/panels/GridPanel.vue:1819` and `:1841`, outside my area.

**Searched with no hits in my area at cc60020:**
- `stripAnchor`: the only hits are in `src/panels/GridPanel.vue`.
- `v-memo` in top-level `src/*.vue`.
- `memo|caret|re-?render` in `src/AGENTS.md`.
- `GridPanel` in `src/App.vue` (it mounts the mode components, not GridPanel directly).
- `editorCaretSegmentId` / `setEditorCaret` writes in `src/App.vue`.
