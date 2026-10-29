---
title: Lưới 7px và canh baseline cho toàn app
author: Sally (UX)
created: 2026-10-29
status: thiết kế và cổng đã chốt — chờ Sofia (epic)
sources:
  - ux-auratranslate/DESIGN.md §Bảng token khoảng cách và hình dạng · §Layout & Spacing · §Điều khiển
  - src/tokens/tokens.json `spacing` · src/tokens/controls.css · scripts/check-tokens.mjs Kiểm K
---

# Lưới 7px và canh baseline cho toàn app

## Mục tiêu

Mọi khoảng cách và mọi kích thước cố định của giao diện là bội số của **7px** (7 · 14 · 21 · 28 · 35 · 42 …), khoảng nhỏ nhất khác 0 là 7px. Trong mỗi hàng ngang, biểu tượng, chữ và số đứng chung một baseline. Không chức năng nào đổi: chỉ đổi token, CSS và cách dựng giãn dòng.

**Ngoại lệ (Ice, 2026-10-29):** thân lưới Workspace giữ nguyên số đo hiện tại: chữ nguyên văn (kể cả dòng Hán Việt), chữ bản dịch, ô đang sửa, và đệm của các ô đó. Khung quanh lưới (thanh panel, thanh công cụ, dải TM/Glossary/Soát, thanh trạng thái) theo lưới 7. Chế độ đọc và panel Tra cứu **không** được miễn.

## Hiện trạng (đo 2026-10-29, `src/**/*.{vue,css}` trừ `selftest/`, 70 tệp)

- Đơn vị 4px, khoá ở `tokens.json` và `check:tokens` Kiểm K (`SPACE_UNIT_PX = 4`, chỉ quét `padding`/`margin`/`gap`).
- 505 lần dùng `var(--space-unit)`, 494 lần ở dạng `calc(var(--space-unit) * N)`. Hệ số: ×1: 122 · ×2: 231 · ×3: 87 · ×4: 35 · ×5: 8 · khác: 11.
- `--space-panel-block` (12px) dùng 220 lần, `--space-panel-inline` (16px) dùng 162 lần.
- 131 giá trị px trơn trong `padding`/`margin`/`gap`: 4 (42) · 8 (42) · 12 (31) · 16 (7) · khác (9). 2 giá trị đã là bội 7.
- 17 miễn trừ `aura-allow-spacing`.
- 91 hàng `align-items: center`, 68 hàng `baseline`.
- Thân lưới Workspace (`GridBlock.vue`, `GridPanel.vue`, `SourceHanViet.vue`): 16 khai báo khoảng cách.

⚠️ Đổi `unit` từ 4 sang 7 mà không viết lại hệ số thì mọi khoảng phình ×1,75 (×2 = 8 → 14). Hệ số phải đổi theo giá trị px, không giữ nguyên.

## Quyết định Ice đã chốt (2026-10-29)

| # | Câu hỏi | Chốt |
|---|---|---|
| Q1 | Phạm vi ngoại lệ | Chỉ thân lưới Workspace. Chế độ đọc và Tra cứu ép giãn dòng về bội 7 |
| Q2 | Thang cao điều khiển | **28 / 28 / 35**: bỏ bậc `sm` 24px, còn `md` 28 và `lg` 35 |
| Q3 | Giãn dòng chữ giao diện xuống dòng | **21px** ở 13px = 1.615, dưới sàn 1.66 cũ 0,58px |
| Q4 | Biểu tượng trong hàng baseline | **Đáy biểu tượng đặt trên baseline**, như một chữ hoa |

## Quy ước design system

Đã ghi vào `ux-auratranslate/DESIGN.md` §Lưới 7px (nguồn duy nhất), cùng các bảng token, §Điều khiển và §Giãn dòng sửa tại chỗ bằng 🔵 2026-10-29.

Phần còn thuộc mã, không thuộc DESIGN.md:

- `tokens.json`: `spacing.unit` = `7px` cùng các token mới; thân lưới Workspace giữ số cũ qua token `--space-grid-*`.
- `check:tokens` Kiểm K: `SPACE_UNIT_PX` 4 → 7, ca tự kiểm đổi theo; sửa một cổng có sẵn, không thêm cổng.

## Quyết định trên mockup (Ice, 2026-10-29)

- **D1 — chọn (a):** giãn dòng chữ nội dung là bội 7 nhỏ nhất không chật hơn hôm nay.
- **D2 — chọn 7/14:** trong nhóm 7, giữa nhóm 14.
- **Lưới áp cả chiều ngang** (Ice nhắc thêm): đệm trái/phải, khe, lề, thụt, bề rộng cố định; bề rộng `ch` làm tròn lên bội 7. Bề rộng ôm chữ không ép được bằng CSS, ghi ở DESIGN.md.

## Cổng (Winston, Ice chốt 2026-10-29)

Đo 2026-10-29 trên `src/**/*.{vue,css}` trừ `selftest/`. Không cần AD mới: spine không khai đơn vị khoảng cách, giãn dòng hay sàn 1.66; AD-58 mục 6 nhận 🔵 cho thanh 42px.

**Kiểm E — giãn dòng.** 393 khai báo `line-height`, 0 cái dùng px: mọi giãn dòng đi qua `--leading-*`, nên cổng đặt ở tầng token.

- Giãn dòng vẫn là tỉ số không đơn vị (Ice chọn thay cho `px`; 137 rule đặt `font-size` mà không tự đặt giãn dòng sẽ thừa hưởng nguyên `px`).
- Thêm phép kiểm: `fontSize × lineHeight` cách một bội 7 không quá 0,01px, cho mọi token trừ bốn token thân lưới `source-cjk` · `source-latin` · `source-hanviet` · `editor` (khai bằng tên trong `check-tokens.mjs`, có lý do).
- `LINE_HEIGHT_FLOOR` 1.66 → **1.6** cho mọi token `wraps: true` (Ice chọn: `lookup-example` 21px). Hai mục deviation `lookup-gloss`/`lookup-example` trong `tokens.json` chết theo.
- `EXPECTED_TYPOGRAPHY` đổi theo bảng của DESIGN.md.
- Đối chứng đỏ: đổi cỡ chữ của một token mà không đổi tỉ số ⇒ đỏ; một token `wraps` ở 1.59 ⇒ đỏ.

**Kiểm K — khoảng cách và kích thước.** `SPACE_UNIT_PX` 4 → 7.

- Tầm mở thêm (D3, Ice chọn): `height` · `min-height` · `max-height` · `width` · `min-width` · `max-width` · `flex-basis` · `grid-template-columns` · `grid-template-rows` · `grid-auto-rows` · `top` · `right` · `bottom` · `left` · `inset`. Đo: 440 khai báo, 103 giá trị px lệch bội 7 trong 33 tệp.
- Với các thuộc tính này chỉ xét số `px`, kể cả bên trong `min()`/`minmax()`/`repeat()`; `%`, `fr`, `vh`, `ch`, `auto`, `fit-content` cho qua. Padding/margin/gap giữ luật cũ: cú pháp lạ ⇒ đỏ.
- 0 · 1 · 2 · 3px hợp lệ không cần miễn trừ, vì là bề dày nét (DESIGN.md §Lưới 7px, Không theo lưới), và chỉ ở thuộc tính kích thước. Ngoài tập đó, mọi giá trị lệch lưới cần `aura-allow-spacing` (thanh cuộn 10px là ví dụ).
- Bề rộng `ch` chưa canh: Story E kiểm `round()` trên WKWebView trước, rồi mới gắn cổng. Ghi nợ `Chủ: Epic N` khi epic có số.
- Tự kiểm K thêm ca: `width: 200px` đỏ · `width: 203px` xanh · `height: 1px` xanh · `height: 4px` đỏ · `grid-template-columns: 300px 1fr` đỏ · `minmax(0, 1fr) 301px` xanh.

## Lộ trình

| Bước | Ai | Việc | Đầu ra |
|---|---|---|---|
| 0 | Sally | Mockup `ux-auratranslate/mockups/luoi-7px.html` (đã dựng 2026-10-29): trước/sau cho Workspace (khung quanh lưới), baseline cận cảnh, Tra cứu, Chế độ đọc (D1), Cài đặt và chân lớp phủ, thẻ Library (D2); font thật, hai theme | Ice duyệt |
| 1 | Sally | Đã sửa `DESIGN.md` (front-matter, bảng chữ, §Bảng token khoảng cách, §Giãn dòng, §Lưới 7px, §Điều khiển, §Components) và hai dòng `EXPERIENCE.md`, 2026-10-29 | Tài liệu nguồn |
| 2 | Winston | Đã xong 2026-10-29: AD-58 mục 6 🔵; D3 và Kiểm E ở §Cổng; không AD mới | Spine |
| 3 | Sofia | Epic mới (Epic 12 đã đóng cả 34 story) và các story dưới đây; ghi nợ kiểm tay `Chủ: Epic N` và `B7` | Cây vé |
| 4 | Amelia | Story A — **Nền token và cổng**: `tokens.json`, `controls.css` (bỏ `--sm`), Kiểm E và Kiểm K theo §Cổng, token `--space-grid-*` cho thân lưới, phép chuyển máy móc mọi hệ số, px trơn và kích thước theo bảng, giãn dòng theo DESIGN.md. Một commit, vì cổng đổi cùng lúc với mã | Cả app ở lưới 7 |
| 5 | Amelia | Story B — **Canh baseline**: baseline cho `ui-icon-btn`, biểu tượng 14px, rà 91 hàng `align-items: center`, các thanh cố định | |
| 6 | Amelia | Story C — **Thanh tiêu đề 42px**: `tauri.conf.json` `trafficLightPosition`, `traffic-light-inset`, đo trên máy Ice (AD-58 mục 6) | |
| 7 | Amelia | Story D — **Giãn dòng nội dung**: Chế độ đọc (preset và thanh trượt), Tra cứu, tiêu đề Chương theo D1 (a) | |
| 8 | Amelia | Story E — **Rà từng bề mặt** theo D2 (7/14) và lưới ngang (bề rộng `ch` làm tròn bằng `round()`, kiểm trên WKWebView): Library, trang Tác phẩm, Tra cứu, Cài đặt, 22 tệp `*Overlay.vue` | |
| 9 | Ice | Một lượt dùng thật trước khi epic `done` | |

Story A đổi `scripts/check-tokens.mjs` nên chạy trọn bộ test một lần. Story C đổi `tauri.conf.json`, cũng chạy trọn bộ. Các story còn lại chạy test của phần mình chạm. Ít nhất 10 tệp vitest đang ghim số px (`controls-css`, `controls-adoption`, `statusBar`, `panelHeaders`, `glossaryManage`, …) sẽ đổi theo; một ca đổi số vì token đổi là ca di chuyển, không phải đỏ cần sửa mã.

## Rủi ro

- **Mật độ đổi không đều.** 4 → 7 làm 164 chỗ thưa ra; 8 → 7 làm 273 chỗ chặt lại; `panel-inline` 16 → 14 hẹp lề panel 2px mỗi bên; `panel-block` 12 → 14 cao thêm. Story E rà bằng mắt, không bằng số.
- **Thanh trạng thái 35px chứa nút 28px.** Còn 3,5px mỗi bên, không chia đều trên lưới 7; canh baseline giữa các mục, cả nhóm nằm giữa thanh.
- **Hiệu năng lưới.** Thân lưới không đổi nên không có thêm lượt tính bố cục. Story B không chạm hàng lưới; nếu sau này canh số thứ tự câu theo baseline dòng đầu thì đo lại thời gian dời con trỏ ở 9.850 câu, cùng điều kiện máy.
- **Chữ 21px ở 13px** dưới sàn 1.66 cũ; Ice đã chọn có biết. Mockup kiểm bằng chuỗi dày dấu (`ườ` trên `ộ`).
