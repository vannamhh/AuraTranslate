---
title: 'Thanh lọc Quản lý Glossary: mọi control cùng một bậc chiều cao'
type: 'bugfix'
ticket: ''
created: '2026-10-28'
status: 'built'
baseline_revision: 'be61e16a44f66836da26a3c63bbd85a87a40e70e'
route: 'oneshot'
route_source: 'auto'
risk: 'low'
review: 'quick'
review_source: 'pinned'
lenses_ran: ['quick']
review_loop_iteration: 0
context: []
---

<frozen-after-approval reason="human-owned intent — do not modify unless human renegotiates">

## Intent

**Problem:** Trên thanh lọc của `GlossaryManageOverlay.vue` (đo trong trình duyệt ở cổng 1420, dpr 2, lớp phủ mở với IPC không có), các control trộn ba chiều cao: khung `ui-seg` Toàn cục/Tác phẩm/Cả hai 28px (fieldset bị flex kéo dãn) nhưng ô bên trong chỉ 24px, ô Tìm `ui-field` 28px, ô Sắp theo 28px, còn chip Tất cả…Khác và chip "Mọi xuất xứ" 24px. Hàng con trỏ nằm lệch nhau 2px, và ô đang chọn trong `ui-seg` không lấp kín khung của nó. Hàng Sửa…Chuyển xuống và hàng Xuất/Nhập CSV đã đều 28px.

**Approach:** Theo DESIGN.md §Điều khiển, "mọi control trên cùng một hàng dùng cùng một bậc". Ice chọn bậc `md` 28px (2026-10-28, "chuyển tất cả lên 28px"), thay vì `sm` 24px mà DESIGN.md đặt cho chip lọc. Có hai thay đổi:
- `ui-seg` chuyển sang tổng chiều cao 28px tính cả viền, áp cho cả 10 nơi đang dùng (hiện 26px, không trùng bậc nào).
- Trên thanh lọc Glossary, mọi chip (kể cả trigger "Mọi xuất xứ") mang bổ nghĩa `ui-chip--md` 28px, và hàng căn giữa theo chiều dọc.

Mặc định 24px của `ui-chip` ở các màn khác giữ nguyên. Câu "chip lọc dùng bậc `sm`" trong DESIGN.md nhận một ghi chú 🔵.

</frozen-after-approval>

## Implementation Notes

Tuyến oneshot: khoảng 20 dòng, chạm `src/GlossaryManageOverlay.vue`, có thể chạm `src/tokens/controls.css`, cộng một ca test.

- Đo trước khi sửa (cổng 1420, dpr 2): khung `ui-seg` bị flex kéo lên 28px còn ô bên trong 24px, chip 24px, ô Tìm 28px. Hàng Sửa…Chuyển xuống và hàng Xuất/Nhập CSV đều 28px sẵn.
- `ui-seg` mang `height: 28px` trên khung; ô con bỏ chiều cao riêng và giãn theo khung, nên không còn hở dù flex kéo hay không.
- Đẩy mặc định lên 28px làm hỏng thanh tiêu đề (trước 24/24, sau 28/24) và Chế độ đọc (26/24 thành 28/24). Ice chọn thêm `ui-seg--sm` 24px cho hai chỗ này; đo lại thấy cả hai cùng `top`/`height` với hàng xóm.
- `DropdownMenu`/`SelectMenu` nhận prop `size?: 'md'` để trigger chip "Mọi xuất xứ" mang `ui-chip--md` mà component không phải tự khai chiều cao.
- Sáu nơi dùng `ui-seg` cần IPC nên chưa đo được; đã ghi nợ, Chủ: Story 12.33.
- `check:lint` đỏ sẵn trên HEAD ở `controls-css.test.ts` (`?? ''` thừa); đổi sang `?.[1] ?? ''`.

## Review Triage Log

Lượt `quick` 1: medium 1 · low 3 · false 2.
- medium, defer: mặc định `ui-seg` 28px đổi sáu nơi chưa đo. TM có ô Tìm 28px, `ui-seg` và chip 24px trên cùng thanh, tức hàng vốn đã trộn bậc trước bản sửa. Ice chọn phương án này khi biết sáu nơi chưa đo; mục nợ Story 12.33 đã liệt kê TM.
- low, patch: mục kiểm tay của plan ghi 24 thay vì 28. Đã sửa.
- low, patch: lệnh Verification thiếu `titlebarToolsMenu.test.ts`. Đã thêm.
- low, patch: ca test chưa canh `ui-seg` của Glossary không mang `--sm`. Đã thêm assert.
- false: prop `size` chỉ có tác dụng với chip. Kiểu `'md'` không đổi gì ở `field`, vì field đã 28px mặc định.
- false: trong Chế độ đọc, lớp `.btn` đè chiều cao ô con. Đo trong trình duyệt: ô con 22px, khung 24px.

## Verification

**Commands:**
- `npx vitest run tests/frontend/glossaryManage.test.ts tests/frontend/controls-css.test.ts tests/frontend/titlebarToolsMenu.test.ts` -- expected: xanh, có ca canh `ui-chip--md` và `ui-seg` không mang `--sm` trên thanh lọc, cùng ca canh chiều cao `ui-seg`/`ui-seg--sm`

**Manual checks (if no CLI):**
- Mở Quản lý Glossary ở `http://localhost:1420` và đo `getBoundingClientRect()` mọi `.ui-*` trong `.gm-toolbar`. Kết quả đúng: cùng `top`, cùng `height` 28.
- Thanh tiêu đề (Workspace) và thanh Chế độ đọc: `ui-seg--sm` cùng `top` với hàng xóm, cùng `height` 24.
