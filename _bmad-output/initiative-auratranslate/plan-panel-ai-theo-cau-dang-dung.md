---
title: 'Panel Đề xuất AI chỉ nói về câu con trỏ đang đứng; system prompt mặc định cấm Markdown'
type: 'bugfix'
ticket: ''
created: '2026-10-26'
status: 'done'
baseline_revision: 'af68f23a1fdb1cd6e2751d06baa46266e9e4ac0d'
route: 'full'
route_source: 'auto'
risk: 'low'
review: 'quick'
review_source: 'pinned'
lenses_ran: ['quick']
review_loop_iteration: 0
context:
  - '{project-root}/src/AGENTS.md'
---

<frozen-after-approval reason="human-owned intent — do not modify unless human renegotiates">

## Intent

**Problem:** Dịch câu 1 rồi đặt con trỏ sang câu 2, panel Đề xuất AI vẫn hiện tóm tắt prompt và bản dịch của câu 1 kèm hai dòng đỏ ("Bản ghi này được lắp cho câu số 1… Lắp lại để xem đúng câu đang xem.", "Kết quả này được dịch cho câu số 1…"). Đây là trạng thái bình thường sau mỗi câu nhưng trông như lỗi, và câu "Lắp lại" chỉ tới nút Lắp đã bỏ ở `af68f23`. Ngoài ra mô hình nhỏ trả bản dịch kèm Markdown (`**…**`) vì system prompt mặc định chưa cấm.

**Approach:** Panel chỉ hiện thông tin của câu hiện tại (`editorActiveSegmentId`): tóm tắt prompt và kết quả dịch thuộc câu khác thì ẩn; quay lại câu đó thì hiện lại. System prompt mặc định thêm một dòng cấm Markdown.

**Đã chốt (Ice, 2026-10-26):**
- Panel theo câu đang đứng; bỏ hai dòng đỏ ở panel.
- Thêm vào `default_system_prompt.txt` dòng: "Không dùng Markdown: không thêm tiêu đề, dấu ** hay ký hiệu định dạng." Chỉ đổi văn bản mặc định; system prompt Ice đã sửa trong Cài đặt giữ nguyên.
- Hai phần làm chung một plan (Ice chọn làm cả hai trong cùng lượt).
- Thao tác trên kết quả dịch một câu (Đưa sang bản dịch, Thử lại, kể cả bằng phím tắt) chỉ áp khi kết quả thuộc câu con trỏ đang đứng. Điều này đảo luật C2 của Story 9-9 (kết quả đơn được đưa sang câu của nó dù con trỏ đã dời): lô 12-13 rồi dịch đơn 11, con trỏ 13 ⇒ đưa bản dịch lô của 13; con trỏ ở câu không có kết quả nào ⇒ không ghi gì (Ice, 2026-10-26).

## Boundaries & Constraints

**Always:**
- Ẩn chỉ là hiển thị: bản ghi prompt và kết quả dịch không bị xoá; "Đưa sang bản dịch" của dịch một câu vẫn ghi vào `aiTranslateRunSegmentId`.
- Khi kết quả thuộc câu khác, "Đưa sang bản dịch" của dịch một câu bị khoá (không đưa một bản dịch đang ẩn); nhánh dịch lô vẫn theo câu hiện tại như cũ.
- Đang dịch câu khác thì "Huỷ dịch" vẫn bấm được và có một dòng trung tính "Đang dịch câu số {n}…".

**Never:**
- Không đổi màn Xem prompt (ghi chú "của câu khác" và nút lắp ở đó giữ nguyên).
- Không đổi lỗi/xác nhận của thao tác Đưa sang bản dịch (`editorPromoteAiTranslationError`, hộp xác nhận).

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected Output / Behavior | Error Handling |
|----------|--------------|---------------------------|----------------|
| Sang câu khác | Đã dịch câu 1, con trỏ sang câu 2 | Không tóm tắt prompt, không bản dịch, không dòng đỏ; "Dịch câu này" dịch câu 2 | — |
| Quay lại | Từ câu 2 về câu 1 (chưa dịch câu nào khác) | Tóm tắt và bản dịch câu 1 hiện lại; Đưa sang bản dịch ghi vào câu 1 | — |
| Đang dịch câu khác | Đang stream câu 1, con trỏ sang câu 2 | Dòng "Đang dịch câu số 1…", Huỷ dịch bấm được, không hiện chữ đang stream | — |
| Lỗi của câu khác | Dịch câu 1 lỗi, con trỏ sang câu 2 | Không hiện lỗi đó | — |
| Markdown | Chưa sửa system prompt | System prompt gửi đi có dòng cấm Markdown | — |

</frozen-after-approval>

## Code Map

- `src/panels/AiTranslationPanel.vue:190,236` -- `isRecordStale`, `isAiTranslateStale` (đã đọc `editorActiveSegmentId`). `:509-535` -- `.ai-inspector-bar`: tóm tắt (`:512`), dòng đỏ `data-ai-prompt-summary-stale-notice` (`:522`). Khối kết quả (~`:600-690`): trạng thái lỗi `data-ai-translate-state="error"`, dòng đỏ `data-ai-translate-stale-notice` (~`:661`), `data-ai-translate-text` (~`:670`), dòng usage. `canPromoteAiTranslate` (~`:216-232`).
- `src/aiTranslateState.ts` -- `isAiTranslateResultStale`, `aiTranslateRunSegmentId`, `aiTranslateAccumulatedText`, trạng thái chạy.
- `src/i18n/vi.json` -- `ai.translate.stale_notice` (chỉ panel dùng ⇒ xoá nếu hết người dùng); `ai.prompt_inspector.stale_notice` (màn Xem prompt còn dùng ⇒ giữ).
- `src-tauri/src/core/aiconfig/default_system_prompt.txt` -- thêm một dòng. Test Rust đọc hằng `DEFAULT_SYSTEM_PROMPT`, không chép văn bản.

## Tasks & Acceptance

**Execution:**
- [x] `src/panels/AiTranslationPanel.vue` -- tóm tắt prompt chỉ hiện khi `!isRecordStale`; bỏ dòng đỏ của tóm tắt; khối kết quả (chữ, usage, lỗi của lượt chạy) chỉ hiện khi `!isAiTranslateStale`; bỏ dòng đỏ của kết quả; thêm dòng trung tính "Đang dịch câu số {n}…" khi đang chạy cho câu khác; `canPromoteAiTranslate` nhánh dịch một câu false khi stale.
- [x] `src/i18n/vi.json` -- khoá "Đang dịch câu số {n}…"; xoá `ai.translate.stale_notice` nếu không còn chỗ dùng.
- [x] `src-tauri/src/core/aiconfig/default_system_prompt.txt` -- thêm dòng cấm Markdown.
- [x] `tests/frontend/aiTranslate.test.ts`, `tests/frontend/aiPromptInspector.test.ts` -- sửa ca đang khẳng định hai dòng đỏ; thêm ca cho từng hàng ma trận; đối chứng đỏ bằng gỡ thật điều kiện `!isAiTranslateStale` ở khối kết quả.

**Acceptance Criteria:**
- Given đã dịch câu 1, when đặt con trỏ vào ô dịch câu 2, then panel không còn dòng chữ đỏ nào và không hiện bản dịch của câu 1.
- Given chưa sửa system prompt, when dịch một câu rồi mở Xem prompt, then khối System prompt có dòng cấm Markdown.

## Implementation Notes

- Khoá thao tác trên kết quả của câu khác nằm trong handler (`promoteAiTranslate` nhánh dịch một câu, và handler Thử lại), không chỉ ở nút; phím tắt `Mod+Shift+Enter` từng đưa được bản dịch đang ẩn.
- Ngoài plan, cũng ẩn nút Thử lại và dòng "đã huỷ" khi kết quả thuộc câu khác, để chúng không nói về một kết quả đang ẩn.
- Ca phím tắt đổi gán (Shift+M) trong `aiTranslate.test.ts` từng khẳng định đưa sang câu 21 khi con trỏ ở câu 99, tức đúng hành vi cũ; nay đưa con trỏ về câu 21 trước khi bấm.
- Lần sửa đầu đã xoá nhầm khoảng 300 dòng test không liên quan trong `aiPromptInspector.test.ts`. Đã khôi phục từ `af68f23` và chỉ thay khối B6; tệp trở lại 66 ca.
- Unhandled rejection của `aiPromptInspectorScreen.test.ts` có từ `af68f23` (mở màn tự lắp mà mock trả `undefined`). Đã commit riêng ở `80266f14f788026dcc70d5be7abfd9e2b6ba5e2d`.

## Plan Change Log

## Review Triage Log

Lượt 1 (quick): high 1 · medium 1 · low 3 · false 3 · rejected-low 1.
- high, patch — `aiPromptInspector.test.ts` mất 309 dòng: năm khối test không liên quan (B15, P6, B2, V5, Glossary "so với câu trước"), số ca từ 66 xuống 51 ⇒ người điều phối khôi phục từ `af68f23`, chỉ thay khối B6.
- low, patch — ca "CÙNG một câu" khẳng định một thuộc tính đã bỏ khỏi template, nên không bao giờ đỏ ⇒ khẳng định dòng tóm tắt có mặt.
- medium, patch — hàng ma trận Markdown / AC2 không có test nào ⇒ thêm ca Rust khẳng định `DEFAULT_SYSTEM_PROMPT` có dòng cấm Markdown.
- false — handler nằm ngoài Code Map: đúng luật Always (khoá chính hành động), đã ghi ở Implementation Notes.
- low, patch — phím tắt Thử lại vẫn chạy lại lỗi của câu khác đang ẩn ⇒ thêm cùng phép kiểm vào handler.
- false — thiếu bằng chứng đối chứng đỏ: agent báo gỡ `!isAiTranslateStale` làm 2 ca đỏ; gỡ guard trong handler làm 1 ca đỏ.
- false — thay đổi ở `aiPromptInspectorScreen.test.ts` nằm ngoài plan: đây là bản sửa cho `af68f23`, đã commit riêng.
- low, rejected — comment mang số story phía trên `canPromoteAiTranslate`: có từ trước, không thuộc dòng đang sửa; AGENTS.md cấm dọn hàng loạt.

## Verification

**Commands:**
- `npx vitest run tests/frontend/aiTranslate.test.ts tests/frontend/aiPromptInspector.test.ts tests/frontend/aiPromptInspectorScreen.test.ts` -- expected: xanh.
- `cargo test --test aiconfig_contract --test ai_prompt_contract --test ai_translate_contract` (sau `npm run build`) -- expected: xanh.
- Đủ 14 cổng `pre-push` và toàn bộ vitest -- expected: xanh.
