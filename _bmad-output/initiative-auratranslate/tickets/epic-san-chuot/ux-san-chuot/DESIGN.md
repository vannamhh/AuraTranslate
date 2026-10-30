---
name: Sàn chuột
status: final
created: 2026-10-30
updated: 2026-10-30
extends: ../../../ux-auratranslate/DESIGN.md
---

# Sàn chuột — Design

Không token mới, không dáng nút mới. Mọi chỗ dùng lớp `ui-*` ở `src/tokens/controls.css`, theo `ux-auratranslate/DESIGN.md` §Điều khiển. Hình mẫu: `mockups/noi-dat-thao-tac-cau.html`, phương án D. Spine thắng khi bản vẽ khác spine.

## Layout & Spacing

Menu câu rộng 294px, mở tại con trỏ và lật vào trong khi sát mép cửa sổ. Lưới không thêm hàng nào.

Hai nút ‹ › ở thanh trạng thái dùng bậc 28px, cách dòng vị trí 7px. Điểm cắt song ngữ rộng 14px, cao 28px. Story 14.7 đo tâm cách tâm trên tệp thật; dưới 24px thì nâng rộng lên 21px.

## Components

| Chỗ | Dáng |
|---|---|
| Menu câu | Lớp `ui-menu`, nổi bằng `--shadow-popover`; tiêu đề `Câu N` là chữ `ui-label` màu `on-surface-variant`, không bấm được |
| Mục menu câu | `ui-menu-item`, phím canh phải màu `on-surface-variant`, nhóm ngăn bằng nét `outline`; mục tắt theo dáng Khoá |
| ‹ › Chương · ‹ › Chương nhập · lên/xuống diff | `ui-icon-btn` |
| Ô Review Mode | `ui-seg__item`, biểu tượng mới vẽ tay trong `AppIcon.vue` theo §Biểu tượng: hai cột bản dịch cạnh nhau |
| Công tắc panel | `ui-menu-item` cộng `ui-switch__track`, như `layout.toggle_panel_headers` |
| Cụm nút trên khối, Cặp lệch trước/sau, Mục trước, Huỷ | Nút Phụ |
| Tới luật làm sạch, Bỏ chọn, Ẩn | Nút Chìm |
| Số câu khi rê chuột | Nền `surface-sunken`, con trỏ bấm được; câu đã chọn giữ dáng vùng chọn hiện có |
| Điểm cắt song ngữ | `¦` màu `on-surface-variant`; đang nhắm hoặc rê chuột thì `on-surface`; đã cắt là `‖` màu `on-surface`. Phân biệt bằng hình, không bằng màu |

## Do's and Don'ts

- Do: giữ `primary` cho ba việc ở `ux-auratranslate/DESIGN.md` §Colors; câu đang nhắm dùng dáng caret hiện có, không thêm màu.
- Don't: thêm một nút hay hàng nào lên Lưới để gợi ý menu; gợi ý nằm ở thanh trạng thái.
