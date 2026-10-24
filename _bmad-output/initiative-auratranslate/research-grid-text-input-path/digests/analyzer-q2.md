# Analyzer digest — Q2 @ 99f3fc1

`git show`/`git grep` at 99f3fc1, `_bmad*` excluded; tests/e2e not counted as readers. LB = load-bearing. ~9 files read.

## Writes and whether they trigger
- LB `editedText` `shallowRef(Map)`, typing writes in place `editedText.value.set(...)`, never `triggerRef` ⇒ no reactive reader of `editorEditedText` re-runs on a keystroke — `src/panels/editorPanelState.ts:682 editedText @ 99f3fc1`, `:791 noteEditorEdit @ 99f3fc1`; comments "Written in place without triggering… The template reads `editedShape`" (`:679-680`), "not reactive per keystroke, so read it imperatively" (`:683`); export raw shallowRef, no `readonly()` (`:684 editorEditedText @ 99f3fc1`).
- LB `editedShape` mutated in place (`shapes.set`, `:792`), triggers only via `if (shape !== before) triggerRef(editedShape)` — `src/panels/editorPanelState.ts:786-793 noteEditorEdit @ 99f3fc1`; export raw ref (`:688 editorEditedShape @ 99f3fc1`).
- `tmFilledSegmentIds` shallowRef replaced with new Set; `dropTmFilled` returns early if id absent — `src/panels/editorPanelState.ts:151-161 dropTmFilled @ 99f3fc1`; exported `readonly(...)` (`:153`).
- Four notices shallowRefs set `null` by `datThongBao({})` — `src/panels/editorPanelState.ts:2678-2690 datThongBao @ 99f3fc1` (called `:813`). Inference (Vue `hasChanged`): null over null triggers nothing.
- `lastSavedAt` `ref` (`:697`, exported `readonly` `:698`); `flushError` `shallowRef` (`:1259-1262`); assigned in `flushEditorNow` (`:873`, `:877`, `:879`).
- LB flush pending set is plain `new Map()` in `createEditorFlush` closure; editorFlush.ts imports nothing from Vue — `src/panels/editorFlush.ts:129 createEditorFlush @ 99f3fc1`. Every reader imperative.
- Proofread: `clearProofreadFor` replaces `results` with new Map — `src/proofreadState.ts:262-264 clearProofreadFor @ 99f3fc1`; may write `reverts` via `dropReverts` (`:251`, `:318-323`), `state`/`runSegmentIds` (`:254` via resetProofread, `:265-269`); all refs/shallowRefs (`:54-68`); called from `src/panels/GridPanel.vue:1703 reportEdit @ 99f3fc1`.

## Readers of `editedShape` / `tmFilledSegmentIds` (reactive)
- LB `src/panels/GridPanel.vue:615-643 gridBlockViews @ 99f3fc1` computed; reads `editorEditedShape.value` (`:617`), `editorTmFilledSegmentIds.value` (`:618`) ⇒ re-runs on `triggerRef(editedShape)` and on `dropTmFilled` replace. Inside: `idsWithChangedRule` (`:599-613`) diff vs `rulesBasis` (`new Map(shapes)` snapshot `:641`); `restRulesOf` (`:278-292`) from arguments (`:634`). Feeds `v-for="block in gridBlockViews"` (`:1919`), `<GridBlock :block>` (`:1918-1926`).
- `src/panels/GridBlock.vue:311-319 v-memo @ 99f3fc1` includes `editorEditedShape.get(s.id)` (`:317`), `ruleOf(s.id)` (`:318`); `src/panels/GridBlock.vue:328 class empty @ 99f3fc1` `editorEditedShape.get(s.id) ?? textShapeOf(s.target_text)`; import `:9`. Reactive render reads (inference: script-setup unwrap); re-render on trigger, v-memo skips unchanged rows.
- No other reader of `editorTmFilledSegmentIds` in src; in-file only `dropTmFilled` (`:157-158`).

## Readers of `editedText` (imperative)
- LB `src/panels/GridPanel.vue:1168-1179 restoreEditedText @ 99f3fc1` reads `editorEditedText.value` (`:1170`); callback of `watch([() => editorSegments.value, gridRoot], restoreEditedText, { flush: 'post' })` (`:1181`); sources only `editorSegments`, `gridRoot`; keystroke does not re-run it.
- `src/panels/editorPanelState.ts:1744-1746 danhSachDieuHuong @ 99f3fc1` imperative; callers `goToNextUntranslated` `:1730`, `goToNextSegment` `:1788`, `goToPrevSegment` `:1797`; file's only `watch` at `:103`, `:107`, no `computed(`.
- Other hits are comments: `src/panels/README.md:290/:299`, `segmentHistoryState.ts:22/:291`, `config/segment.ts:1447`.

## Readers of proofread `results` (and `state`/`runSegmentIds`/`reverts`)
- `src/panels/GridPanel.vue:1214 watch @ 99f3fc1` `watch([proofreadResults, () => editorSegments.value, gridRoot], repaintProofread, { flush: 'post' })` reactive on `results` shallowRef; `repaintProofread` (`:1184-1190`) reads `proofreadResults.value` imperatively; also called from `restoreEditedText` (`:1178`), `scheduleRepaintProofread` (`:1197-1203`).
- `src/StatusBar.vue:201-212 proofreadCountsText @ 99f3fc1` computed on `proofreadResults.value` (`:205`), `proofreadStateValue` (`:202`), rendered `:524`.
- `src/proofreadState.ts` computeds: `proofreadFindingCount` `:91-95`, `proofreadAmbiguous` `:97`, `proofreadUnlocated` `:103`, `proofreadCurrent` `:350-365`, `proofreadRevertible` `:375-378` (via `topRevert`); `editorTextOf` (`:325-327`) reads `editorSegments`, not `editedText`.
- Downstream (grep only, not traced): ProofreaderStrip.vue `:74, :104, :107, :124, :127, :143-150`, template `:197-297`; `inlineStripEligibility.ts:29-48 eligibleInlineStrips` (`:33`, `:39-40`, `:44`); `titlebarProofreadStatus.ts:8`.

## Readers of `lastSavedAt` / `flushError`
- `src/StatusBar.vue:88-94 onMounted ticker @ 99f3fc1` imperative in 1 s `setInterval` (`:92`), bumps `now`.
- LB `src/StatusBar.vue:279-283 secondsSinceSave @ 99f3fc1` computed on `editorLastSavedAt`, `now`; rendered `:594-596` last `v-else-if`.
- `src/panels/GridPanel.vue:1753 flushErrorSegmentIds @ 99f3fc1` computed on `editorFlushError.value`; `:1764-1779 rowErrorLabelById` (`:1772`, `:1774`); `rowErrorTextById` (`:1781-1785`); passed via `gridBlockContext` (`:1816-1820`), read `GridBlock.vue:316` (v-memo), `:327` (`refused`). All reactive.
- `e2e/support/flushWait.mjs:68-89` test infra.

## Readers of the four notices
- `src/StatusBar.vue` computeds: `confirmNoticeKey` `:308-312`, `regroupNoticeText` `:361-371`, `splitChapterNoticeText` `:405-412`, `navNoticeKey` `:459-463`. Render chain `v-if`/`v-else-if` `:534 → :558 → :570 → :582 → :592 (glossaryHoverText) → :594 (secondsSinceSave)` ⇒ clearing notices lets "Đã lưu" render.
- `src/chapterSplitState.ts:84 confirmChapterSplit @ 99f3fc1` imperative once after `await splitChapterHere()`.
- Comments only: `glossaryTermHoverState.ts:11-12`, `commands/index.ts:3324/3367/3435`, `main.ts:821`.

## Readers of flush pending set (imperative via `flush.*`, `flush` at `editorPanelState.ts:618`)
- `src/panels/editorPanelState.ts @ 99f3fc1`: `:740 armFlushTimer` `flush.deadline()`; `:845`, `:849 flushEditorNow` `isDirty()` (`:845` `first !== 'saved' || !flush.isDirty()` early return; `:849` clean ⇒ `'clean'`); `:853` `pending()` snapshot copy (`editorFlush.ts:137-140`); `:878` `onFlushed`; `:943-948 flushEditorBeforeDiscreteWrite` `isDirty()` twice ⇒ `'failed'`/`'clean'`/`'still-dirty'`; `:999 resetEditorPanel` `flush.reset()`.
- Callers via `flushEditorNow`/`flushEditorBeforeDiscreteWrite`: `src/main.ts:1242` `void wireExitFlush()` → `editorPanelState.ts:1176-1188` Tauri `listen` → `await flushEditorNow()`; `src/modes/libraryImport.ts:407`; `src/modes/libraryChapters.ts:313, :484`; `src/proofreadState.ts:182, :442, :497, :588`; `src/reviewModeState.ts:370`. Not in assignment: `src/panels/segmentHistoryState.ts:318`; `editorPanelState.ts :286` (setEditorCaret), `:747` (timer), `:377, :441, :497, :571, :1496, :1988, :2203, :2307, :2923`.

## Not found
- `computed`/`watch`/template reading `editorEditedText` (only `GridPanel.vue:1170`, `editorPanelState.ts:1745`).
- Reactive wrapper over pending set (`flush\.[a-zA-Z]+\(`; vue import in editorFlush.ts).
- Other Vue file importing `editorEditedShape` (only GridBlock.vue, GridPanel.vue).
- Grep-only: ProofreaderStrip.vue, libraryChapters.ts, libraryImport.ts, reviewModeState.ts, segmentHistoryState.ts.
