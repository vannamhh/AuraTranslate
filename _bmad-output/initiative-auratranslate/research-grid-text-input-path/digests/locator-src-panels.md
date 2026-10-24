# Locator digest — area `src/panels/` @ 99f3fc1

Read via `git show 99f3fc1:` / `git grep 99f3fc1`. Symbol = enclosing definition.

**Premise correction (Q5):** at 99f3fc1 the whole `Map` is not replaced per edit; `editedText` is written in place without trigger; only `editedShape` triggers, only when a row's shape changes. Replace-per-edit removed in `043f3bc`.

## Q1
- `src/panels/GridBlock.vue:271 (template, col-tgt) @ 99f3fc1` — listeners on column `div.col.col-tgt`: `@beforeinput="onBeforeInput"` (:276), `@compositionstart`/`@compositionend` (:277-278), `@input="onEditInput"` (:279), `@keydown="onEditKeydown"` (:280). Cells `contenteditable="true"`, `data-col="tgt"`, `data-segment-id` (:322-344), render `s.target_text` (load-time value).
- `src/panels/gridBlockContext.ts:44 (GridBlockContext) @ 99f3fc1` — handler types; `src/panels/GridPanel.vue:1816 (gridBlockContext)` builds, `provide` (:1845); `GridBlock.vue:25 (inject)`.
- `src/panels/GridPanel.vue:1435 (onBeforeInput) @ 99f3fc1` — `targetCellOf` (:746); if not composing `unwrapProofread(cell, history?)` (:1439); ① `insertParagraph` ⇒ `preventDefault` + `execCommand('insertLineBreak')` guarded by `insertingLineBreak` (:1088, :1461-1493), no `reportEdit` (engine `input` does it); `insertLineBreak` passes (:1494); ② paste/drop/replacement ⇒ `preventDefault`, flatten, Range API insert, `reportEdit(cell)` (:1497-1560); ③ other (`insertText`, deletions) left to engine.
- `src/panels/GridPanel.vue:1712 (onEditInput) @ 99f3fc1` → `:1699 (reportEdit)`: `segmentIdOf` (:751) → `noteEditorEdit(id, cell.textContent)` → `clearProofreadFor(id)` (imported from `../proofreadState`, :74).
- `src/panels/editorPanelState.ts:758 (noteEditorEdit) @ 99f3fc1` — in order: ① `dangChuyenChuong` early return (:779); ② `textShapeOf(targetText)`, before-shape `editedShape.get(id)` or on first edit `segments.value.find(...)` (:786-790); ③ `editedText.value.set(...)` in place, no trigger (:791); ④ `shapes.set(...)` (:792); ⑤ `triggerRef(editedShape)` only if shape changed (:793); ⑥ `dropTmFilled(id)` (:794 → :156) replaces `tmFilledSegmentIds` Set only if id in it; ⑦ `flush.markChanged(id, text, Date.now())` (:796); ⑧ `armFlushTimer()` (:797 → :738) clear + re-set `setTimeout` to `flush.deadline()`; ⑨ `datThongBao({})` (:813 → :2678) sets `confirmNotice`, `regroupNotice`, `navNotice`, `splitChapterNotice` (all `shallowRef`) to `null`.
- `src/panels/editorFlush.ts:132 (createEditorFlush.markChanged) @ 99f3fc1` — pending set is plain non-reactive `Map dirty` (:129); `schedule.onChange(now)` from `../layout/writeSchedule`. Constants :48, :61, :83.
- `src/panels/editorPanelState.ts:835 (flushEditorNow) @ 99f3fc1` — reads `flush.pending()`, `saveSegmentTargets`, `flush.onFlushed`, `lastSavedAt.value = Date.now()` (:879) or `flushError.value` (:873); `inFlight` declared :714.
- `src/panels/editorSegments.ts:178 (TextShape)` / `:180 (textShapeOf) @ 99f3fc1` — 0 empty, 1 whitespace, 2 text.

## Q2
- `editorEditedShape`: `src/panels/GridPanel.vue:615 (gridBlockViews) @ 99f3fc1` reads at :617; `src/panels/GridBlock.vue:317 (template v-memo, col-tgt)` lists `editorEditedShape.get(s.id)`, `:328 (class empty)` reads it. Inference (locator): module ref read in each block render ⇒ every mounted `GridBlock` render effect subscribes; `v-memo` decides which rows patch. `GridPanel.vue:280 (restRulesOf)`, `:599 (idsWithChangedRule)` take `shapes` as parameter.
- `editorEditedText` (no reactive readers): `GridPanel.vue:1168 (restoreEditedText)` via watcher on `editorSegments`+`gridRoot` (:1181); `editorPanelState.ts:1744 (danhSachDieuHuong)`.
- `editorTmFilledSegmentIds`: `GridPanel.vue:618 (gridBlockViews)`, identity compared in `idsWithChangedRule` (:608).
- Notices + `lastSavedAt`: no reader in `src/panels/`; out of area `src/StatusBar.vue:279 (secondsSinceSave)`, `:309/362/406/460`.
- Pending set: not reactive; `flush.isDirty()`/`flush.pending()` in `editorPanelState.ts:845,849,853 (flushEditorNow)`, `:943 (flushEditorBeforeDiscreteWrite)`.
- `proofreadResults` (via `clearProofreadFor`): `GridPanel.vue:1214 (watch → repaintProofread)` `flush: 'post'`, `:1184 (repaintProofread)`.
- `editorCaretSegmentId` (written on `selectionchange`): `GridPanel.vue:654 (caretIndex)`, `:659 (caretIdIn)` ⇒ `caret-id` prop; out of area `TmFuzzyStrip.vue:30`, `ProofreaderStrip.vue:40`, `GlossaryConfirmStrip.vue:61`, `StatusBar.vue:156`.

## Q3
- `src/panels/gridBlocks.ts:1 (GRID_BLOCK_ROWS = 200)`, `:5 (gridBlocksOf) @ 99f3fc1`.
- `src/panels/gridBlockContext.ts:15 (GridBlockView) @ 99f3fc1` — `{index, start, segments, rules}`.
- `src/panels/GridPanel.vue:615 (gridBlockViews) @ 99f3fc1` — reuses previous block object when segments keep identity and `idsWithChangedRule` (:599, vs `rulesBasis` WeakMap :594) has no id in block; else recomputes `restRulesOf` for slice (:278), reuses if `sameRules` (:295); `rulesBasis.set(views, {shapes: new Map(shapes), ...})` (:641) copies whole shapes Map on each recompute.
- `src/panels/GridPanel.vue:1918 (template GridBlock v-for) @ 99f3fc1` — props `:key="block.index"`, `:block`, `:caret-id="caretIdIn(block)"`, `:anchor-id="anchorIdIn(block)"` (:650), `:strip-row="stripRowIn(block)"` (:645), `:has-strip-track` (:664); inside `PanelFrame` slot (:1849).
- `src/panels/GridBlock.vue:17 (defineProps) @ 99f3fc1`; `ruleOf` (:74) `'primary'` if `caretId === id` else `block.rules.get(id)`; `cellLabelOf` (:86); `concordanceHighlightOf`/`chapterAssetsOf` (:65-71, :95).
- `v-memo` lists: rule `GridBlock.vue:113` `[s, ruleClassForRow, selectedRowClass, s.id===anchorId]`; number `:135` `[s, block.start+j, selectedRowClass, anchor]`; source `:150-162` `[s, selectedRowClass, anchor, cutOffsetsOf, glossaryTermsOf, concordanceHighlightOf, trailingImagesOf, chapterAssetsOf, editorAssetsDir, showHanViet, viewMode]`; target `:311-319` `[s, block.start+j, selectedRowClass, anchor, rowErrorTextById.get, editorEditedShape.get(s.id), ruleOf(s.id)]`.
- Source column: `SourceHanViet` (:201) or pieces (:223), `ChapterImage`; strip track (`TmFuzzyStrip`, `GlossaryConfirmStrip`, `ProofreaderStrip`) only where `hasStripTrack` (:350-362).

## Q4
- keydown: `src/panels/GridPanel.vue:1608 (onEditKeydown) @ 99f3fc1` returns if `isComposing`; else every key `unwrapProofread(typed, true)` + `scheduleRepaintProofread()` (:1615-1620); `Escape` ⇒ `editor.clear_source_cuts` (:1644); `Backspace` at cell start ⇒ `preventDefault` + `editor.merge_segments` (:1652-1695).
- Proofread: `src/panels/proofreadHighlight.ts:71 (unwrapProofread)`, `:51 (keepingSelection)` save/restore selection; `:117 (repaintProofreadHost)` unwraps all via `querySelectorAll` then re-wraps each scanned cell (`:84 wrapProofread`). `GridPanel.vue:1197 (scheduleRepaintProofread)` `setTimeout(0)`; `:1205/:1209 (onCompositionStart/End)` set `composing`.
- Selection: `GridPanel.vue:1325` two document-level `selectionchange` listeners; `:1234 (onSelectionChange)` ⇒ `setEditorCaret(id)` when anchor in target column; `:1289 (onSourceSelectionChange)` always sets `sourceSelecting.value`, early return outside source (:1311). `src/panels/editorPanelState.ts:281 (setEditorCaret)` writes `caretSegmentId`, `lastCaretSegmentId` (same value ⇒ no trigger, inference); `flushEditorNow` only on row change (:286); when `id` not null runs `positionFlush.markMoved` + `armPositionFlushTimer` (:292-295) on every `selectionchange` in a target cell. `src/panels/positionFlush.ts:89 (markMoved)`, `editorPanelState.ts:636 (armPositionFlushTimer)`. `src/panels/selectionContract.ts:132 (useSelectionSurface)` on grid root `GridPanel.vue:739-740`, no per-keystroke listener.
- Caret helpers (click/command, not typing): `GridPanel.vue:1090 (ensureCaretNextFrame)`, `:768 (setCaret)`, `:1346 (watch editorCaretPlacement)`.
- Rule markers: change via `editedShape` trigger or `tmFilled`, plus `caretId` → `ruleOf` → `'primary'`.
- Typing zone: comments only here (`GridPanel.vue:1578-1597`, cites `keys.ts:508-510`, `editorTypingZone.test.ts`).
- Status bar: `datThongBao({})` and `lastSavedAt` only.

## Q5
- `src/panels/editorPanelState.ts:672-688 (editedText / editedShape doc-comments) @ 99f3fc1` — `editedText` front-end-local, AD-1 exception, apart from `segments`; "Written in place without triggering: a reactive read here in the grid template would re-render the whole grid slot on every keystroke"; `editedShape` "triggers only when a row's effective shape changes".
- `src/panels/GridPanel.vue:1144-1158 (DOM SỞ HỮU VĂN BẢN) @ 99f3fc1` — DOM owns typed text; rendering live text made caret jump to 0 and text vanish.
- `src/panels/GridPanel.vue:266-277 (restRulesOf doc-comment)`; `src/panels/editorPanelState.ts:606-618` (flush module-level, survives `api.clear()`, NFR18); `:703-714 (inFlight)` promise not flag.
- `src/panels/README.md:253-257 (Lưới chia khối, Story 2-16) @ 99f3fc1` — 200-row blocks, `provide`/`inject`, single selection surface on root; `GridPanel.vue:4-24` header — column organisation with `subgrid`.
- Commits (6): `043f3bc` replace-Map-per-input ran 4 cols × n rows per key ⇒ in-place + `editedShape`/`triggerRef`; 9,850 rows release still ~0.5 s/key, cause unidentified. `43729b3` `v-memo` on four columns, always-present rule slot (inserting `div.rule` relayouts whole `subgrid`). `742096c` 200-row blocks + `GridBlock`. `f62012f`, `8260a92` block identity reuse, per-block `rules` ⇒ `Mod+Enter` re-renders 1 block not 6. `2aa120f` review fixes.
- `gridBlocks.ts` no doc-comment on why 200.

## Q6
- No test files under `src/panels/`. In-area code mentions `e2e/specs/grid-empty-cell.e2e.mjs` (`GridPanel.vue:1115`), `editorTypingZone.test.ts` (`:1597`), `e2e/specs/segment-navigation.e2e.mjs` (`:1403`), `tests/frontend/editorSourceCutGesture.test.ts` (`:735`), `tests/frontend/editorSegmentRule.test.ts` (`editorSegments.ts:210`).

## Searched, nothing found (src/panels @ 99f3fc1)
- `selectionchange` outside `GridPanel.vue`; per-keystroke `watch` on `editorEditedText`; reader of `editorLastSavedAt` or the four notices; any test file; rationale for `GRID_BLOCK_ROWS = 200`.
