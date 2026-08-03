Token màu và chữ — **đã kiểm tương phản WCAG AA ở cả hai theme** (AD-34). Không giá trị màu nào nằm trong component.

**Story sở hữu nội dung: 1.4.**

Ghi chú bàn giao từ Story 1.1: `Source Sans 3` có mặc định trục `wght = 200` và `name ID 1 = Source Sans 3 ExtraLight`. Token `ui-label` (700) **chưa từng được kiểm** — Story 1.4 phải dựng font này ở 400/600/700 rồi mới coi là đã kiểm, và nên khai descriptor `{ weight: "200 900" }` khi đăng ký `FontFace`.
