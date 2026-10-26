## Phase 1 (Webview)

Done: all `src/` tasks. No `scripts/check-commands.mjs` edit was needed (the buttons use plain `@click="dispatch(...)"`, no HANDLER_TABLE row existed or was needed; gate green).

New exports / behaviour
- `editorActiveSegmentId` (computed, `src/panels/editorPanelState.ts`): `caret ?? lastCaret` with lastCaret only when its id is still in `segments`. Reads internal refs, so tests that `setEditorCaret(null)` after a caret still see the last one; `resetEditorPanel()` clears both.
- `src/aiTranslateHandlers.ts`, `src/main.ts` (`ai.prompt.assemble`), `AiTranslationPanel.vue`, `AiPromptInspectorOverlay.vue` read `editorActiveSegmentId` instead of `editorCaretSegmentId`.
- `openAiPromptInspector()` (`src/aiPromptInspectorState.ts`) now imports `editorActiveSegmentId` and `selectedPromptSetName`; after `refreshAiPromptRecord()` it calls `assembleCurrentAiPrompt(set, activeId)` only if record is null, no read error, and activeId non-null. Record of another segment: no assemble.
- `aiPromptInspectorState.ts` now value-imports `./panels/editorPanelState` and `./promptSetState`: any test that `vi.mock`s `editorPanelState` must export `editorActiveSegmentId`.

DOM hooks / i18n
- Panel: removed `[data-ai-prompt-assemble]`, `.ai-inspector-actions`, `.ai-inspector-assemble` (and CSS), the assemble hint. `.ai-inspector-summary` (`[data-ai-prompt-summary-kind]`) renders only when a record exists (kind never `no_record` in DOM). New `<p class="ai-inspector-hint" data-ai-prompt-auto-note>` = `panel.ai_translation.prompt_auto_note` ("Prompt tự lắp khi dịch."). The translate hint `.ai-translate-hint` shows when no active segment, text now "Đặt con trỏ vào ô dịch của một câu để dịch câu đó." (`translate_no_segment_hint`).
- Panel watcher on `editorActiveSegmentId` clears the assemble error (so caret -> null no longer clears it).
- Overlay: `[data-aip-record-state="none"]` shows `ai.prompt.summary_no_record` when busy or active segment exists, else `ai.prompt_inspector.no_segment_hint` (new). `[data-aip-assemble-error]` shows `aiPromptAssembleError` via tError (reuses `.aip-read-error`). `[data-aip-assemble-selected]` button ("Lắp prompt cho câu đang chọn", key `ai.prompt_inspector.assemble_selected`) inside the record body, `v-if="isStale"`, `@click="dispatch('ai.prompt.assemble')"`, disabled while `aiPromptAssembleBusy`. The existing `[data-aip-stale]` note serves as the "of another segment" note.
- Removed key `panel.ai_translation.assemble_no_segment_hint` and `panel.ai_translation.assemble_busy`. Kept `command.ai.prompt.assemble` and `ai.prompt.summary_no_record`.

Gates after last edit: check:commands 0, check:i18n 0, check:tokens 0, check:lint 0, check:panel-refs 0, vue-tsc 0.

Red tests (6 planned vitest files run; 3 files fail, ~60 errors)
- Dominant cause: `vi.mock('../../src/panels/editorPanelState', ...)` factories lack `editorActiveSegmentId` ("No editorActiveSegmentId export on the mock"), hit via `aiPromptInspectorState.ts` and panel/overlay. Seen in `aiTranslate.test.ts`, `aiPromptInspector.test.ts`, `aiPromptInspectorScreen.test.ts`. Also check (not run) `aiBatchPanelRows`, `aiTranslateBatch`, `aiTranslationPromoteErrorAlert`, `segmentSelection` tests, which touch caret + AI panel.
- Expected semantic reds after mocks fixed: the case around `aiPromptInspector.test.ts:687` (open without assemble), tests asserting `[data-ai-prompt-assemble]`, `summary_no_record` in the panel or `assemble_no_segment_hint`/`assemble_busy` keys (grep in aiPromptInspector.test.ts).

## Phase 2 (Tests)

Done. 11 vitest files (plan's 6 + `aiBatchPanelRows`, `aiTranslateBatch`, `aiTranslationPromoteErrorAlert`, `segmentSelection*`): 230 passed. `vue-tsc` and eslint on the touched tests clean. Not run: full suite, the 14 gates.

Changes
- Mocks of `editorPanelState` in `aiTranslate`, `aiPromptInspector` (two factories), `aiPromptInspectorScreen`: added `editorActiveSegmentId` = `computed(caret ?? last)` with a sync watch recording the last non-null caret (faithful to the real computed, minus the in-Chapter check). `aiTranslationPromoteErrorAlert`: static `ref(null)`. `freshPanel` in `aiPromptInspector.test.ts` wires `assembleAiPrompt` through the mocked `editorActiveSegmentId`.
- `aiPromptInspector.test.ts`: the old "nút Lắp prompt cho câu này" describe replaced by panel cases (no button, no `no_record` summary, auto-note, dispatch assemble, error survives caret -> null and clears when active segment changes, busy flag); new describe "mở màn Xem prompt" (5 cases). The `:687` open case now has a record present.
- `aiTranslate.test.ts`: new describe (3 cases). `aiTranslateBatch.test.ts`: 1 case (real editor state). `editorLeaveSegment.test.ts`: 5 cases. `aiTranslateHandlersWiring.test.ts`: source scan (AI flow files do not read bare `editorCaretSegmentId` on code lines; main.ts assemble uses `editorActiveSegmentId`).
- Production change (one string): `aiPromptInspectorState.ts` console.warn text said `editorCaretSegmentId === null`; now `editorActiveSegmentId`. Found by the new scan. No real defect found.
- `aiTranslatePromoteConfirmFlow.test.ts` and `aiPromptInspectorScreen.test.ts` got no new case: single-run promote does not read the caret (uses `aiTranslateRunSegmentId`), batch promote is covered in `aiTranslateBatch`; Screen only needed the mock fix.

Counter-checks (real removal in `editorPanelState.ts`, restored byte-identical)
- Replace body of `editorActiveSegmentId` by `return caretSegmentId.value` (drops `?? lastCaret`): red = `editorLeaveSegment` "null caret falls back", "switching Chapter", and `aiTranslateBatch` "tiêu điểm đã rời lưới". The mock-based `aiTranslate` case stays green by design (the mock owns the computed); the real-state tests are the guard.
- Replace the in-Chapter check by `return last`: red = `editorLeaveSegment` "last caret whose id is no longer in the Chapter".

Row map (matrix / AC -> test -> passed)
- Con trỏ trong ô dịch -> aiTranslate "con trỏ ở câu 3 rồi về null... bấm chuột dịch câu 3" and existing click cases ("generating: token chảy..." runMock 42) -> pass
- Tiêu điểm sang panel -> aiTranslate "🔴 con trỏ ở câu 3 rồi về null" + "dispatch ai.translate.run với con trỏ null" + aiTranslateBatch "tiêu điểm đã rời lưới" + editorLeaveSegment "null caret falls back" -> pass
- Chưa chạm câu nào -> aiTranslate "chưa chạm câu nào ⇒ nút Dịch khoá, gợi ý" + editorLeaveSegment "no segment has been touched yet" -> pass
- Câu đã retire -> editorLeaveSegment "last caret whose id is no longer in the Chapter" (id 999 not in Chapter; red counter-check 2) -> pass
- Đổi Chương -> editorLeaveSegment "switching Chapter (reset) drops the last caret" -> pass
- Xem prompt chưa bản ghi -> aiPromptInspector "không bản ghi + con trỏ ở câu 3 ⇒ lắp ĐÚNG một lần", "...tiêu điểm đã rời lưới", "...không câu nào đã chạm ⇒ gợi ý", "...chưa chọn bộ prompt ⇒ lỗi no_set_selected" -> pass
- Xem prompt bản ghi câu khác -> aiPromptInspector "bản ghi của câu 1, con trỏ ở câu 3 ⇒ không lắp; ghi chú + nút" -> pass
- Lỗi lắp còn hiện -> aiPromptInspector "lỗi Lắp còn hiện khi tiêu điểm rời lưới" -> pass
- AC1 -> aiTranslate "con trỏ ở câu 3 rồi về null... bấm chuột" -> pass
- AC2 -> aiPromptInspector "không bản ghi + con trỏ ở câu 3" -> pass
- AC3 -> aiPromptInspector "không bản ghi: không nút Lắp, không dòng tóm tắt 'chưa lắp', có dòng 'Prompt tự lắp khi dịch'" + V5 "no_record ⇒ không dòng tóm tắt" -> pass

Gaps
- The overlay "chưa có bản ghi" auto-assemble guard was not counter-checked by removal (only the two plan-required removals were).
- Matrix "Chưa chạm câu nào" for Xem prompt covers the overlay hint; Rust untouched.
