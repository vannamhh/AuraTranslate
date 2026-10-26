---
title: 'Luồng AI lấy câu hiện tại theo con trỏ, kể cả khi tiêu điểm đã sang panel AI'
type: 'bugfix'
ticket: ''
created: '2026-10-26'
status: 'built'
baseline_revision: 'cc8d88c6e115dd7251138a9c1472fb6ea5825a35'
route: 'full'
route_source: 'auto'
risk: 'medium'
review: 'quick'
review_source: 'pinned'
lenses_ran: ['quick']
review_loop_iteration: 0
context:
  - '{project-root}/src/AGENTS.md'
---

<frozen-after-approval reason="human-owned intent — do not modify unless human renegotiates">

## Intent

**Problem:** `GridPanel.vue:1131` (`onColumnFocusOut`) đặt `editorCaretSegmentId` về `null` ngay khi tiêu điểm rời cột bản dịch, mà luồng AI chỉ đọc giá trị này. Hậu quả với Ice:
- Bấm vào panel Đề xuất AI thì panel báo "Chọn một câu trong Chương" dù con trỏ vẫn ở câu 1.
- Các nút Dịch câu này / Lắp / Đưa sang bản dịch bị khoá ngay lúc nhấn chuột xuống (Vue vá DOM trước mouseup), nên bấm chuột không chạy; chỉ phím tắt chạy.
- Phần xem trước prompt ("Chưa lắp ráp prompt nào trong phiên này" + "Lắp prompt cho câu này") đứng giữa panel như việc bắt buộc, khiến Ice tưởng mỗi câu cần một prompt.
- Màn Xem prompt chỉ báo "Chưa lắp ráp prompt nào trong phiên này" khi chưa dịch câu nào.

**Approach:** Luồng AI lấy câu theo khuôn `StatusBar.vue:159` / `proofreadScope.ts:19`: câu con trỏ đang đứng, không có thì câu cuối cùng con trỏ đứng nếu câu đó còn trong Chương. Panel thu gọn phần xem trước prompt và nói rõ prompt tự lắp khi dịch. Màn Xem prompt hiện được prompt của câu đang chọn.

**Đã chốt (Ice, 2026-10-26):**
- Ice duyệt hướng sửa này; chỉ cần con trỏ nằm trong ô dịch của câu, không phải bôi đen cả câu.
- Màn Xem prompt: mở mà **chưa có bản ghi nào** thì tự lắp cho câu hiện tại. Đã có bản ghi của câu khác thì giữ nguyên bản ghi đó (tab Kết quả, Gửi lại không đổi), hiện ghi chú "của câu khác" và nút "Lắp prompt cho câu đang chọn". Không đổi Rust.
- Panel Đề xuất AI: bỏ dòng "Chưa lắp ráp prompt nào…", gợi ý "Chọn một câu…" của phần lắp, và nút "Lắp prompt cho câu này"; thêm một dòng ngắn "Prompt tự lắp khi dịch". Tóm tắt số thuật ngữ Glossary / câu TM đã chèn chỉ hiện khi đã có bản ghi. Lệnh `ai.prompt.assemble` vẫn còn (phím tắt, nút trong màn Xem prompt).
- Giữ một plan dù vượt 1600 token (1498 từ, chưa đếm được bằng tiktoken).

## Boundaries & Constraints

**Always:**
- Một hàm/computed duy nhất cho "câu hiện tại của luồng AI" trong `editorPanelState.ts`; mọi chỗ của luồng AI đọc nó, không tự ghép `??` tại chỗ.
- Câu cuối cùng chỉ được dùng khi `id` còn trong `editorSegments` (gộp/tách có thể để lại id đã retire).
- "Đưa sang bản dịch" của dịch lô ghi vào đúng câu hiện tại đó (Decision 3, `aiTranslateBatchState.ts:38`); dịch một câu vẫn ghi vào `aiTranslateRunSegmentId` như cũ.
- Vạch `primary` và `[data-caret]` của lưới vẫn theo `editorCaretSegmentId` trần (con trỏ thật), không đổi.

**Never:**
- Không đổi `onColumnFocusOut` hay nghĩa của `editorCaretSegmentId`.
- Không đổi cách chọn nhiều câu cho dịch lô (`segmentSelectionState.ts`).
- Không thêm `@mousedown.prevent` để giữ tiêu điểm lưới.

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected Output / Behavior | Error Handling |
|----------|--------------|---------------------------|----------------|
| Con trỏ trong ô dịch | Con trỏ ở câu 3, Ice bấm "Dịch câu này" bằng chuột | Dịch câu 3 | — |
| Tiêu điểm sang panel | Con trỏ ở câu 3, Ice bấm ô Bộ prompt rồi bấm "Dịch câu này" | Panel không báo "Chọn một câu"; dịch câu 3 | — |
| Chưa chạm câu nào | Vừa mở Chương, chưa đặt con trỏ | Nút khoá, có gợi ý đặt con trỏ vào ô dịch | — |
| Câu đã retire | Câu cuối cùng là câu vừa bị gộp mất, tiêu điểm ngoài lưới | Coi như chưa chạm câu nào | — |
| Đổi Chương | Sang Chương khác, chưa đặt con trỏ | Không mang câu của Chương cũ | — |
| Xem prompt, chưa có bản ghi | Con trỏ ở câu 3, chưa dịch câu nào, mở Xem prompt | Tự lắp và hiện prompt câu 3 | Chưa chọn bộ prompt: hiện lỗi `no_set_selected` qua `tError`; chưa chạm câu nào: gợi ý đặt con trỏ vào ô dịch, không gọi lắp |
| Xem prompt, bản ghi của câu khác | Đã dịch câu 1, con trỏ ở câu 3, mở Xem prompt | Vẫn hiện prompt đã gửi của câu 1 + ghi chú + nút lắp cho câu 3; không tự lắp | — |
| Lỗi lắp còn hiện | Lắp lỗi ở câu 3, Ice bấm vào panel | Lỗi vẫn hiện (chỉ xoá khi câu hiện tại đổi thật) | — |

</frozen-after-approval>

## Code Map

- `src/panels/editorPanelState.ts:148-179,281-285,973-990` -- `caretSegmentId`, `lastCaretSegmentId`, `setEditorCaret`, `resetEditorPanel` (xoá cả hai khi đổi Chương). Thêm computed "câu hiện tại của luồng AI" ở đây.
- `src/proofreadScope.ts:19-20` -- khuôn có sẵn: `caret ?? lastCaret`, kiểm id còn trong `editorSegments`.
- `src/panels/AiTranslationPanel.vue:135` -- `watch` caret chỉ gọi `clearAiPromptAssembleError()`; caret về `null` khi bấm vào panel làm lỗi lắp biến mất. `:196,201,216,237,243,532,683` -- `isRecordStale`, `canAssemble`, `canRunAiTranslate`, `canPromoteAiTranslate`, `isAiTranslateStale`, hai ghi chú stale. `:515-594` -- khối `.ai-inspector-bar` (tóm tắt `summary_no_record`, gợi ý, nút Lắp) và `.ai-surface` (Dịch/Huỷ/Đưa sang, `translate_no_segment_hint`). `:446-454` -- nút "Xem prompt".
- `src/aiTranslateHandlers.ts:45,77` -- `runAiTranslate(set, caret)`; `promoteAiTranslate` nhánh lô ghi vào `caretId`.
- `src/main.ts:1094-1101` -- lệnh `ai.prompt.assemble` gọi `assembleCurrentAiPrompt(set, caret)`.
- `src/aiPromptInspectorState.ts:157,294` -- `openAiPromptInspector` chỉ đọc bản ghi; `aiPromptRecordIsStale`. `src/AiPromptInspectorOverlay.vue:106,113,227,278` -- `canConfirmNow`, `isStale`, khối rỗng `summary_no_record`, ghi chú stale.
- `src/i18n/vi.json:191,800,1410-1411` -- `summary_no_record`, `command.ai.prompt.assemble`, hai gợi ý "Chọn một câu trong Chương…".
- Không đổi: `GridPanel.vue` (`onColumnFocusOut`, `onSelectionChange`), `segmentSelectionState.ts`, `StatusBar.vue`.

## Tasks & Acceptance

**Execution:**
- [x] `src/panels/editorPanelState.ts` -- computed `editorActiveSegmentId`: `caret ?? lastCaret`, lastCaret chỉ khi id còn trong `editorSegments` -- một nguồn cho luồng AI.
- [x] `src/panels/AiTranslationPanel.vue` -- mọi chỗ đọc caret của luồng AI (`:135` watch, `:196,201,216,237,243,532,683`) đổi sang `editorActiveSegmentId`; bỏ dòng `summary_no_record`, gợi ý lắp và nút Lắp khỏi `.ai-inspector-bar`, tóm tắt chỉ hiện khi có bản ghi, thêm dòng "Prompt tự lắp khi dịch"; gợi ý dịch đổi chữ thành "Đặt con trỏ vào ô dịch của một câu để dịch câu đó".
- [x] `src/aiTranslateHandlers.ts`, `src/main.ts` -- `runAiTranslate`, nhánh lô của `promoteAiTranslate`, lệnh `ai.prompt.assemble` dùng `editorActiveSegmentId`.
- [x] `src/aiPromptInspectorState.ts`, `src/AiPromptInspectorOverlay.vue` -- mở màn: không có bản ghi và có câu hiện tại ⇒ `assembleCurrentAiPrompt(selectedPromptSetName, câu hiện tại)`; không có câu ⇒ gợi ý đặt con trỏ; bản ghi của câu khác ⇒ ghi chú + nút "Lắp prompt cho câu đang chọn" (dispatch `ai.prompt.assemble`). `isStale`/`canConfirmNow` đọc `editorActiveSegmentId`.
- [x] `src/i18n/vi.json`, `scripts/check-commands.mjs` -- khoá mới/đổi chữ; gỡ hàng `HANDLER_TABLE` của nút đã bỏ, thêm hàng cho nút mới (không hạ sàn).
- [x] `tests/frontend/editorLeaveSegment.test.ts` -- `editorActiveSegmentId`: caret thắng; caret null ⇒ lastCaret; lastCaret đã retire ⇒ null; đổi Chương ⇒ null.
- [x] `tests/frontend/aiTranslateHandlersWiring.test.ts`, `aiTranslatePromoteConfirmFlow.test.ts`, `aiTranslate.test.ts` -- caret null + lastCaret = câu 3 ⇒ dịch / đưa sang ghi đúng câu 3; nút không khoá khi tiêu điểm ở panel; lỗi lắp không mất khi caret về null.
- [x] `tests/frontend/aiPromptInspector.test.ts`, `aiPromptInspectorScreen.test.ts` -- sửa ca `:687` (mở không lắp) theo quyết định: không bản ghi + có câu ⇒ lắp đúng một lần; có bản ghi câu khác ⇒ không lắp, có ghi chú + nút; không câu ⇒ gợi ý, không lắp.
- [x] Đối chứng đỏ: gỡ thật vế `?? lastCaret` ⇒ ca "tiêu điểm sang panel" đỏ; gỡ thật phép kiểm id còn trong Chương ⇒ ca "câu đã retire" đỏ.

**Acceptance Criteria:**
- Given con trỏ trong ô dịch câu 3, when Ice bấm chuột vào "Dịch câu này" ở panel AI, then câu 3 được dịch.
- Given chưa dịch câu nào và con trỏ ở câu 3, when mở Xem prompt, then thấy prompt đã lắp của câu 3 thay vì "Chưa lắp ráp prompt nào".
- Given panel Đề xuất AI mở và chưa dịch câu nào, when nhìn panel, then không còn dòng "Chưa lắp ráp prompt nào" hay nút "Lắp prompt cho câu này"; có dòng "Prompt tự lắp khi dịch".

## Implementation Notes

- `editorActiveSegmentId` (computed ở `editorPanelState.ts`) là nguồn duy nhất của luồng AI. `aiTranslateHandlersWiring.test.ts` quét mã để chắc các tệp luồng AI không còn đọc caret trần trên dòng mã.
- Không cần đổi `HANDLER_TABLE`, vì nút bị bỏ và nút mới đều là `@click="dispatch(...)"` trực tiếp.
- `aiPromptInspectorState.ts` giờ import giá trị từ `editorPanelState` và `promptSetState`. Đã kiểm: không có vòng import nào đi ngược về tệp này.
- `aiTranslatePromoteConfirmFlow.test.ts` không thêm ca mới: "Đưa sang bản dịch" của dịch một câu ghi vào `aiTranslateRunSegmentId`, không đọc câu hiện tại. Nhánh lô được canh ở `aiTranslateBatch.test.ts` trên state thật.
- Các ca dùng mock (`aiTranslate.test.ts`) tự dựng `editorActiveSegmentId`, nên chúng không canh phép gỡ vế `?? lastCaret`. Guard thật nằm ở `editorLeaveSegment.test.ts` và `aiTranslateBatch.test.ts`, đều đỏ khi gỡ.

## Plan Change Log

## Review Triage Log

Lượt 1 (quick): high 0 · medium 1 · low 6 · false 4.
- low, patch — comment ở `AiTranslationPanel.vue:156-160,192` còn nói đọc `editorCaretSegmentId`, nói "mở không bao giờ lắp", và nhắc nút Lắp đã bỏ.
- low, patch — doc ở `aiPromptInspectorState.ts:191,296-298` (cùng `aiTranslateState.ts:80,105`, `aiTranslateBatchState.ts:38` và các banner trong test) nay sai.
- medium, patch — `emptyKey` hiện "Chưa lắp ráp prompt nào…" trong lúc màn đang đọc bản ghi rồi tự lắp, tức đúng dòng chữ Ice phàn nàn ⇒ thêm trạng thái "Đang lắp prompt…".
- low, patch — mở màn không xoá `aiPromptAssembleError` cũ, nên lỗi `no_set_selected` cũ vẫn hiện cạnh bản ghi mới.
- low, rejected — tự lắp sau khi đã đóng màn: chỉ chạy khi chưa có bản ghi nào, nên không ghi đè gì; người dùng không gặp.
- false — `aiTranslatePromoteConfirmFlow` không thêm ca: dịch một câu không đọc câu hiện tại; đã ghi ở Implementation Notes.
- low, patch — phép tự lắp khi mở chưa được đối chứng đỏ bằng gỡ thật ⇒ làm thêm. Vế `?? lastCaret` đã có guard trên state thật (`editorLeaveSegment`, `aiTranslateBatch`) và đều đỏ khi gỡ.
- low, rejected — phép quét chỉ đọc bốn tệp: các tệp AI khác chỉ nhắc caret trần trong comment (đã grep); một comment HTML trùng tên thì làm phép quét đỏ ồn ào, không im lặng.
- low, rejected — nút mới dùng chung class `aip-close`: chỉ là thẩm mỹ.
- false — sổ plan chưa cập nhật: phần này do người điều phối ghi, đã ghi.
- false — `HANDLER_TABLE` không đổi: các nút là `dispatch` trực tiếp, `check:commands` xanh.

## Verification

**Commands:**
- `npx vitest run tests/frontend/aiTranslateHandlersWiring.test.ts tests/frontend/aiTranslate.test.ts tests/frontend/aiTranslatePromoteConfirmFlow.test.ts tests/frontend/aiPromptInspector.test.ts tests/frontend/aiPromptInspectorScreen.test.ts tests/frontend/editorLeaveSegment.test.ts` -- expected: xanh.
- Đủ 14 cổng `pre-push` -- expected: xanh.

**Manual checks (if no CLI):**
- Trong app thật: đặt con trỏ vào ô dịch câu 3, bấm chuột "Dịch câu này" ở panel AI ⇒ câu 3 được dịch; mở Xem prompt ⇒ thấy prompt của câu 3.
