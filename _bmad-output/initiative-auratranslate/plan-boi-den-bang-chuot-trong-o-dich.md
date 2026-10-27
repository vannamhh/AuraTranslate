---
title: 'Bôi đen bằng chuột trong ô bản dịch'
type: 'bugfix'
ticket: ''
created: '2026-10-27'
status: 'built'
baseline_revision: '5fd8e9fb6057408eb576465aa4153eb0de140b8a'
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

**Problem:** Kéo chuột trong ô bản dịch không bôi đen được chữ: trong lúc kéo vùng chọn là `Range`, nhả chuột thì nó thành `Caret`. `GridPanel.onCellMouseUp` luôn đặt caret tại điểm nhả, và lượt vá `ensureCaretNextFrame` chỉ coi `Caret` là ổn định nên đặt caret về cuối ô.

**Approach:** Khi nhả chuột mà vùng chọn là một đoạn không rỗng nằm trọn trong ô, giữ đoạn đó: chụp nó trước `focus()`, đặt lại sau, và lượt vá ở frame kế tiếp khôi phục chính đoạn đó thay vì caret. Cú bấm đơn giữ nguyên hành vi đặt caret.

</frozen-after-approval>

## Implementation Notes

Oneshot: một tệp, dựng lại được trong webview thật trước khi sửa.
- Spec dò tạm (kéo bằng `pointer` action tính từ tâm ô): trước sửa `Range` lúc kéo → `Caret` sau nhả; sau sửa giữ `Range` ("ng sử dụn") cả ngay sau nhả lẫn 400 ms sau, ở cú kéo khi panel chưa hoạt động và khi ô đã có tiêu điểm.
- `editor-typing-flush` và `editor-confirm-segment` đỏ cả trên bản gốc (HEAD, và trước `5fd8e9f`): Vite của `tauri dev` mang tem HMR `?t=` sau khi sửa tệp, các hàm reset của `panelReset.mjs` import ra instance khác. Cần khởi động lại `tauri dev` rồi chạy lại.
- Canh bằng `tests/frontend/gridDragSelection.test.ts`: gỡ nhánh giữ vùng chọn ở `onCellMouseUp` ⇒ ca bôi đen đỏ (`'' ≠ 'Hắn'`).
- Sau các vá review, spec dò không kéo được nữa kể cả với bản đã xanh trước đó (không `selectstart`, không `Range` lúc kéo): trình điều khiển đổi, không phải mã. Bấm đơn giữa đoạn bôi đen và nhấp đúp chưa đo được ⇒ Ice thử tay.

## Review Triage Log

Lượt quick: 0 high · 0 medium · 4 low · 2 false · 1 maybe-false.
- low · patch — hai helper chen giữa doc comment của `placeCaretAtPoint` và `setCaret`; dời xuống sau `setCaret`.
- low · patch — `settled()` nhận mọi `Range`; nay so ranh giới với đoạn đã kéo (`holdsRange`).
- low · patch — đoạn kéo trỏ vào node đã rời ô thì `addRange` không chọn gì; nay rơi về caret cuối ô.
- low · patch — không có test canh nhánh mới; thêm `gridDragSelection.test.ts`, đối chứng gỡ nhánh ⇒ đỏ.
- maybe-false — bấm đơn vào giữa đoạn đang bôi đen: nếu WebKit chỉ thu vùng chọn sau handler `mouseup` thì nhánh mới chọn lại đoạn cũ. Spec dò không kéo được nữa nên chưa đo; Ice thử tay trong app.
- false — nhấp đúp "chưa đo" bị ghi như đã qua: câu đó đã gỡ khỏi Implementation Notes, chuyển sang thử tay.
- false — ghi nợ cho hai spec đỏ: nguyên nhân là tem HMR của Vite đang chạy, khởi động lại `tauri dev` là hết; `lenses_ran` lệch giữa diff và tệp vì diff được ghi trước lượt `sed`.

