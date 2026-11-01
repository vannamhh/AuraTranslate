---
title: 'Bỏ cử chỉ Backspace gộp câu ở đầu ô bản dịch'
type: 'bugfix'
ticket: ''
created: '2026-11-01'
status: 'done'
baseline_revision: '48ea2d4bdd7038a17a3b9fa186c0505ea17d1e30'
route: 'oneshot'
route_source: 'auto'
risk: 'medium'
review: 'quick'
review_source: 'pinned'
lenses_ran: ['quick']
review_loop_iteration: 0
context: []
---

<frozen-after-approval reason="human-owned intent — do not modify unless human renegotiates">

## Intent

**Problem:** Xoá chữ trong ô bản dịch bằng `delete` rồi bấm thừa một cú, caret đang ở đầu ô nên `onEditKeydown` gộp câu với câu trên: không hỏi, không hoàn tác được, và caret về đầu ô mới nên mỗi cú sau gộp thêm một câu. Câu mới mất trạng thái xác nhận và lịch sử hiển thị (AD-5), người dùng đọc thành "câu bị xoá".

**Approach:** Ice chọn 2026-11-01 bỏ hẳn cử chỉ: `Backspace` ở đầu ô không làm gì, gộp chỉ còn `⌘M` và mục menu chuột phải của câu. Lật AC1 và AC6 của Story 2.9 và phần cử chỉ của UX-DR32; nghiệp vụ gộp, AD-5, AD-49 không đổi.

</frozen-after-approval>

## Implementation Notes

Route oneshot: chủ yếu là xoá mã; phần thêm trong `src/` và `scripts/` dưới 100 dòng, hơn năm tệp nhưng cơ học (bản vá, chú thích đã sai, test, tài liệu).

- Bằng chứng: `_bmad-output/implementation-artifacts/bo-cu-chi-backspace-ban-do/ket-qua.md`. Phím thật trên mã cũ tái hiện gộp dây chuyền (6 hàng còn 2); mã mới giữ 9 hàng ở lưới và đĩa, `⌘M` và menu câu vẫn gộp.
- Không `preventDefault()`: WebKit không xoá gì ở offset 0 của editing host, đo lại bằng phím thật.
- Giữ tên tệp `e2e/specs/segment-backspace-merge.e2e.mjs` vì sổ nợ và plan cũ trỏ tới nó; ca AC7 trong đó không liên quan và giữ nguyên.
- Ca e2e `⌘M` thay ca cử chỉ để không mất nghiệm thu từ chối ở câu đầu Chương và dòng báo hệ quả (AC4).
- Đối chứng gỡ: thêm lại nhánh gộp vào `onEditKeydown` thì hai ca vitest âm đỏ và `check:commands` Kiểm K đỏ, ca dương `⌘M` xanh.
- Hai mục sổ nợ mới đẩy `check:debt-owner` qua ngưỡng trôi, nên `ITEM_FLOOR` nâng 775 lên 825 đúng công thức cổng in ra.
- Mục sổ nợ ba mệnh đề của cử chỉ đóng bằng `KHÔNG LÀM`. `regroupInFlight` giữ lại vì `⌘M` bấm nhanh vẫn dispatch hai lần.

## Review Triage Log

Một lượt `quick`: 0 high, 0 medium, 6 low, 1 từ chối.

- low, patch: hai chú thích `src/StatusBar.vue` còn tả cử chỉ như đang sống; đã sửa.
- low, patch: dòng chú thích vừa đổi còn ngày, id story, tiếng Việt ở header e2e; đã sửa, xoá hẳn khối `beforeinput` ở `GridPanel.vue`.
- low, defer: thiếu mục WebView2 cho quyết định bỏ `preventDefault()`; đã ghi `Chủ: B7`.
- low, patch: `ket-qua-*.json` chưa track; đưa vào commit.
- low, patch: `verify` của Story 2.9 và dòng sổ nợ cũ còn nói cử chỉ sống; sửa tại chỗ bằng 🔵.
- low, patch: câu "dưới 100 dòng" thiếu phạm vi đo; đã nêu `src/` và `scripts/`.
- low, từ chối: dạng `KHÔNG LÀM (plan-…)` khác `(Story x.y)`; không có story nào cho sửa lỗi này, và sổ nợ đã có tiền lệ `(phiếu quyết #66)`.

## Verification

**Commands:**
- `npx vitest run tests/frontend/editorBackspaceNoMerge.test.ts` -- expected: xanh; khôi phục nhánh gộp thì đỏ.
- `npm run check:commands` -- expected: xanh; thêm lại `dispatch('editor.merge_segments')` vào `onEditKeydown` thì Kiểm K đỏ.
- Đủ bộ gates, vitest, `cargo test` vì đổi `scripts/check-commands.mjs` -- expected: xanh, trừ `check:panel-refs` đỏ từ `48ea2d4b`, đã ghi `Chủ: Story 14.11`.

**Manual checks (if no CLI):**
- Bàn đo phím thật trên WKWebView: ô rỗng và ô có chữ với caret đầu ô, `delete` nhiều cú, lưới và đĩa không đổi; `⌘M` và menu chuột phải vẫn gộp.
