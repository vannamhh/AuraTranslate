---
type: epic
title: "Lưới 7px và canh baseline cho toàn app"
parent: initiative-auratranslate
covers: []
sources: [_bmad-output/initiative-auratranslate/ux-auratranslate/DESIGN.md, _bmad-output/initiative-auratranslate/ux-auratranslate/EXPERIENCE.md, _bmad-output/initiative-auratranslate/plan-luoi-7px-va-canh-baseline.md]
companions: [_bmad-output/initiative-auratranslate/ux-auratranslate/mockups/luoi-7px.html, _bmad-output/initiative-auratranslate/architecture-auratranslate/architecture-auratranslate.md]
after: []
refined: true
risk: high
---

# Lưới 7px và canh baseline cho toàn app

## Description

Ice chốt 2026-10-29 một nhịp bố cục mới: mọi khoảng cách và kích thước cố định là bội 7px theo cả chiều dọc lẫn chiều ngang, và mọi hàng ngang canh theo baseline. Thân lưới Workspace (nguyên văn, bản dịch, ô đang sửa) giữ nguyên. Không chức năng nào đổi; epic này đổi token, CSS, cổng `check:tokens` và chiều cao thanh tiêu đề macOS. Quy ước nằm ở `DESIGN.md` §Lưới 7px, cổng và lộ trình ở `plan-luoi-7px-va-canh-baseline.md`.

## Outcome

Ice dùng app hằng ngày trên một nhịp 7px đều ở mọi màn, và lượt dùng thật của Ice xác nhận các màn khớp mockup `luoi-7px.html`.

## Requirements

- R1 (DESIGN.md §Lưới 7px — Luật, Thang, Kích thước cố định): mọi khoảng cách và kích thước cố định ngoài thân lưới Workspace là bội 7px theo cả hai chiều; `check:tokens` Kiểm K canh padding/margin/gap và thuộc tính kích thước với đơn vị 7.
- R2 (DESIGN.md §Lưới 7px — Ngoại lệ): thân lưới Workspace giữ đúng số đo hiện tại qua token có tên `--space-grid-*` và bốn token chữ `source-cjk`, `source-latin`, `source-hanviet`, `editor`.
- R3 (DESIGN.md §Lưới 7px — Giãn dòng; plan §Cổng, Kiểm E): mọi token chữ ngoài thân lưới cho hộp dòng bằng bội 7 (tỉ số = bội 7 ÷ cỡ chữ), sàn chữ xuống dòng 1.6; Kiểm E đỏ khi cỡ × tỉ số lệch bội 7.
- R4 (DESIGN.md §Lưới 7px — Canh baseline): trong mọi hàng ngang, chữ, số, biểu tượng và điều khiển chung một baseline; đáy biểu tượng 14px trên baseline; nút chỉ có biểu tượng đứng cùng hàng với nút chữ cùng bậc.
- R5 (DESIGN.md §Điều khiển — Thang kích thước): điều khiển còn hai bậc 28 và 35px; không còn bậc 24px.
- R6 (DESIGN.md §Lưới 7px — Bề rộng; Trong nhóm 7, giữa nhóm 14): bề rộng đo bằng `ch` làm tròn lên bội 7; khoảng trong nhóm 7, giữa nhóm 14.
- R7 (DESIGN.md §Lưới 7px — Giãn dòng; EXPERIENCE.md Chế độ đọc): thanh trượt giãn dòng ở Chế độ đọc hiện bội 7 nhỏ nhất ≥ cỡ × tỉ số đã lưu; giá trị lưu không đổi.
- R8 (AD-58 mục 6; DESIGN.md §Lưới 7px — Kích thước cố định): thanh tiêu đề macOS cao 42px, ba nút cửa sổ nằm giữa thanh theo số đo trên máy Ice.

## Constraints

- Mọi chức năng khác không đổi (Ice 2026-10-29): chỉ token, CSS, cổng, cách dựng giãn dòng và `trafficLightPosition` của `tauri.conf.json`.

## Done when

1. `check:tokens` chạy Kiểm E và Kiểm K trên lưới 7 và xanh; mỗi luật mới có ca tự kiểm đỏ, và CI xanh cả nửa Windows.
2. Các test hành vi có từ trước vẫn xanh, chỉ những ca ghim số px được đổi theo token.
3. Mục nợ `Chủ: Epic 13` đã đóng: lượt dùng thật của Ice trên sáu bề mặt của mockup và ba nút cửa sổ ở thanh 42px.
4. Phần chỉ kiểm được trên Windows nằm trong một mục `B7`.

## Boundaries

- Ngoài epic: chữ và đệm ô của thân lưới Workspace; cỡ chữ, màu, bo góc, bề dày nét; hệ số scale giao diện do người dùng chỉnh (câu hỏi để ngỏ ở `DESIGN.md` §Bảng token khoảng cách).
- Touch point: `tauri.conf.json` `trafficLightPosition` — đo lại cho thanh 42px; owner: epic-luoi-7px-va-canh-baseline.
- Must not change: luật kéo cửa sổ và vùng kéo của AD-58; hiệu năng dời con trỏ trong lưới (thân lưới không đổi).

## References

- parent — _bmad-output/initiative-auratranslate/initiative-auratranslate.md, section Requirements
- design — _bmad-output/initiative-auratranslate/ux-auratranslate/DESIGN.md, §Lưới 7px · §Điều khiển · §Giãn dòng
- design — _bmad-output/initiative-auratranslate/ux-auratranslate/mockups/luoi-7px.html
- plan — _bmad-output/initiative-auratranslate/plan-luoi-7px-va-canh-baseline.md, §Cổng · §Lộ trình · §Rủi ro
- architecture — _bmad-output/initiative-auratranslate/architecture-auratranslate/architecture-auratranslate.md, AD-58 mục 6

## Notes

- Decision: 2026-10-29, Ice — epic đứng cuối thứ tự xây, sau Epic 10; bản phát hành đầu không chờ lưới 7px.
- Decision: 2026-10-29, Ice — việc rà bề mặt chia ba story theo bề mặt; có story Refactor sweep đóng epic.
- Decision: 2026-10-29 — `covers` để trống như Epic 11: lưới 7px là quyết định thiết kế, không phải một FR của PRD.
- Decision: 2026-10-29 — đường xuyên suốt là entry 1 (nền token, cổng và chuyển máy móc trong một commit, vì cổng đổi đơn vị cùng lúc với mã); entry 2 đi ngay sau vì `round()` trên WKWebView là phần chưa chắc nhất.
- Decision: 2026-10-29 — entry 3 chờ entry 4: cả hai sửa thanh tiêu đề, và ba nút cửa sổ đo trên bố cục thanh đã canh baseline xong.
- Decision: 2026-10-29 — một làn cho phần dùng chung: entry 4 chờ 2 (cùng sửa `controls.css`), entry 5 chờ 2 (cùng chạm thước đọc của Chế độ đọc); ba story rà bề mặt chờ 2 và 4, riêng Chế độ đọc chờ thêm 5. Thứ tự bảng: 1, 2, 4, 3, 5, 6, 7, 8, 9.
- Decision: 2026-10-29 — mục nợ lượt dùng thật (Done when 3) do Sofia ghi lúc mở epic, `Chủ: Epic 13`; Ice chạy trước khi epic lên done.
