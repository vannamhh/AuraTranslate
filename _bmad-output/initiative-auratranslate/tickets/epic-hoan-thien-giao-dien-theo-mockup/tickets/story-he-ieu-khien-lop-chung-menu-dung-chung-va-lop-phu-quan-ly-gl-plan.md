---
title: 'Hệ điều khiển: lớp chung, menu dùng chung và lớp phủ Quản lý Glossary'
type: 'feature'
ticket: '27'
created: '2026-10-28'
status: 'built'
baseline_revision: '999419a3c676bdb0f2877244375d9e7a74a64f08'
route: 'full'
route_source: 'auto'
risk: 'medium'
review: 'quick'
review_source: 'pinned'
lenses_ran: ['quick']
review_loop_iteration: 0
context:
  - '{project-root}/_bmad-output/initiative-auratranslate/ux-auratranslate/mockups/he-dieu-khien.html'
  - '{project-root}/src/AGENTS.md'
---

<frozen-after-approval reason="human-owned intent — do not modify unless human renegotiates">

## Intent

**Problem:** 309/356 nút tự vẽ bằng 148 lớp riêng; ô nhập, checkbox, menu không có dáng chung; hover gần như không có (DESIGN.md §Điều khiển, Ice duyệt 2026-10-28).

**Approach:** Dựng đủ lớp `ui-*` của §Điều khiển trong `src/tokens/controls.css` theo CSS của `mockups/he-dieu-khien.html`, chuyển bốn menu dùng chung và trọn `GlossaryManageOverlay` sang chúng, và dựng ca vitest canh chuyển đổi mà story 28–33 nối dài.

## Decisions (Ice, 2026-10-28)

1. Giữ một plan, không tách `GlossaryManageOverlay` ra (plan quá trần 1.600 token, Ice biết).
2. Rê chuột nút Nguy hiểm theo DESIGN: viền `error` → `on-surface-variant`, chữ giữ `error`; không theo nền `surface-sunken` của CSS mockup.
3. Quản lý Glossary toàn lớp phủ bậc `md` 28px (thanh lọc, nút hàng, Xuất/Nhập, Thêm mục); chip `sm`; ô sửa trong ô bảng `sm`.

## Boundaries & Constraints

**Always:**
- Màu chỉ qua token (AD-34); ngoại lệ duy nhất là từ khoá hệ thống `Highlight` trong `@media (forced-colors: active)`. Không token màu mới.
- Mỗi trạng thái đổi một thuộc tính, không đổi độ dày viền, đệm, cỡ chữ; `:focus-visible` đứng sau `:hover`.
- Kiểm H giữ nguyên: control có viền thì focus đổi màu viền sang `primary`; nút Chính, checkbox đang chọn, mục menu giữ vòng 2px.
- Component chỉ còn bố cục (khoảng cách, chiều rộng) trong `<style scoped>`; lớp móc mà test/e2e đọc giữ trong template, không mang CSS.
- `@click` vẫn đúng một `dispatch('<id>')`; mọi lối vào, phím, `aria-*` giữ nguyên.

**Never:**
- Không sửa tệp của story 28–33 (kể cả `LibraryWorkPage`, `ImportPreviewOverlay` có ô trên nền trũng); chỉ đổi dáng chúng gián tiếp qua `DropdownMenu`/`SelectMenu`/`ui-icon-btn`.
- Không thêm dependency; không đổi `check-tokens.mjs`; không đụng thanh trượt, `ui-seg` của Workspace, vạch lề segment.
- Không chụp lại ảnh của story 26.

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected Output / Behavior | Error Handling |
|----------|--------------|---------------------------|----------------|
| Focus bàn phím | Tab tới nút Phụ / ô nhập | Viền `primary`, không vòng ngoài; Chính giữ vòng 2px | — |
| Tương phản cao | `forced-colors: active` | `:focus-visible` có `outline: 2px solid Highlight` | Chỉ kiểm được trên Windows (B7) |
| Checkbox đứng riêng | Ô không nhãn bọc | Ô 14px trong vùng bấm 24×24px | — |
| Xoá chờ xác nhận | Bấm Xoá một lần | Nút thành bậc Nguy hiểm, vẫn là nút thứ hai của `.gm-actions` | — |
| Đang sửa hàng | `manageEditing` | Bộ lọc khoá dáng viền đứt | — |

</frozen-after-approval>

## Code Map

- `src/tokens/controls.css` -- đã có `ui-btn` (+`--primary/--quiet/--danger/--sm`), `ui-icon-btn` 26px, `ui-seg`, `ui-kbd`, thanh cuộn. Thêm theo mockup: `--lg`, hover/nhấn năm bậc, focus đổi viền (Chính giữ vòng), `ui-btn--pair` (min-width 8.5em); `ui-icon-btn` 28px mặc định + `--sm`/`--lg`; `ui-field` (+`--lg/--sm/--short/--medium/--full`, `textarea`, `button.ui-field` + `.ui-field__caret`, `.is-invalid`, `ui-field-error`, `--on-sunken` nền `surface`); `ui-check` (+`--bare` 24×24, radio); `ui-switch` + `.ui-switch__track` (bật theo `[aria-checked='true']` của chính nút, dùng được trong `menuitemcheckbox`); `ui-menu`, `ui-menu-item`, `ui-menu-sep`; `ui-chip`. `ui-seg__item`/`ui-chip` nhận trạng thái chọn cả qua `:has(input:checked)` và focus qua `:has(input:focus-visible)` (nhãn bọc radio ẩn ở Glossary). Khối `forced-colors` cuối tệp.
- `src/tokens/reset.css` -- thêm `button, input, textarea, select { font: inherit; }`.
- `src/DropdownMenu.vue` -- `summary.menu-trigger`: `chip`→`ui-chip` (`active` = `aria-pressed` dáng), `field`→`ui-field`, `more`→`ui-btn ui-btn--quiet`; khoá → dáng khoá của lớp; `.menu-panel`→`ui-menu`. Xoá CSS dáng 117-180; giữ `position`, `max-height`, cột.
- `src/SelectMenu.vue` -- `label.select-item`→`ui-menu-item ui-check`; xoá padding/chữ/màu 120-136.
- `src/TitlebarToolsMenu.vue` -- `.tools-list`→`ui-menu`, `.tools-item`→`ui-menu-item` (giữ `justify-content`), `.tools-switch`→`ui-switch__track` (giữ lớp móc `tools-switch`/`on`: `titlebarToolsMenu.test.ts:226,235`). Xoá 210-264.
- `src/panels/LookupSourcesMenu.vue` -- tương tự; `.sources-switch` đang tô `primary` → `ui-switch__track`; giữ `.off`, `.sources-count` (`lookupSourcesPopup.test.ts:171-182`). Trigger giữ `ui-btn ui-btn--sm` (DESIGN: nút mở menu là nút Phụ).
- `src/GlossaryManageOverlay.vue` (template 411-876, style 878-1379) -- `gm-close` :422 → Chìm; `gm-seg` :439 → `ui-seg`; `gm-input` :464, :648, :684 → `ui-field`; `gm-pill` :476-517 → `ui-chip` (bỏ viền `draft` của `gm-pill-pending.gm-pill-on`); SelectMenu :523, :664 ăn theo component; checkbox :535 và radio :794/:803 → `ui-check`; `gm-act` :722-869 → `ui-btn` (+`--primary` :722, :869; `--danger` thay `gm-act-danger` khi chờ xoá, giữ cả lớp móc). Xoá dáng ở 926-941, 980-993, 1093-1145, 1312-1343, 1365-1373; giữ bố cục 951-971, 1076, 1125-1159, 1179, 1194-1204, 1301-1310, 1355-1378. Móc test: `glossaryManage.test.ts` (`.gm-actions button:nth-of-type(2)`, `gm-act-danger`, `.gm-seg-opt`, `.gm-seg-count`, `.gm-toolbar input[type="text"]`), `glossaryExchangeGate.test.ts`, e2e `input[name="gm-tier"]`.
- `src/panels/LookupPanel.vue:436,485,720`, `LookupRecord.vue:113,157` -- không sửa: không ghi đè cỡ, lên 28px qua `controls.css`; hàng cha cao 40px/32px/auto chứa được.
- `tests/frontend/iconButtonLabels.test.ts` -- khuôn đọc `.vue` từ đĩa; ca mới dùng `vue/compiler-sfc` (`parse` + AST template), không regex.
- Nền trũng (đếm 2026-10-28): 0 ô ở thanh trạng thái (nền `surface`), tab dock, dải đầu nguồn. Ô trên nền trũng có ở `.wp-bar` (`LibraryWorkPage.vue:341`, 1 input + 1 SelectMenu chip, story 32) và hàng luật khoá `.ip-cleanup-rule-off` (`ImportPreviewOverlay.vue:1303`, story 30).

## Tasks & Acceptance

**Execution:**
- [x] `src/tokens/controls.css`, `src/tokens/reset.css` -- dựng mọi lớp ở Code Map -- một nguồn cho §Điều khiển.
- [x] `src/DropdownMenu.vue`, `src/SelectMenu.vue`, `src/TitlebarToolsMenu.vue`, `src/panels/LookupSourcesMenu.vue` -- đổi sang lớp `ui-*`, xoá CSS dáng -- menu dùng chung.
- [x] `src/GlossaryManageOverlay.vue` -- như Code Map -- tracer bullet đủ mọi loại control.
- [x] `tests/frontend/controls-adoption.test.ts` -- danh sách `FILES` (năm tệp trên); với mỗi tệp: mỗi `<button>`/`<input>` (trừ `hidden`)/`<textarea>`/`<summary>` có `ui-` trong `class`/`:class`, hoặc có `<label>` tổ tiên gần nhất mang `ui-`; mọi rule scoped nhắm một lớp nằm trên control đó không khai `height`, `min-height`, `border*`, `outline*` và không có `:hover`/`:focus*`/`:active`/`:disabled`. Ngoại lệ là bảng có tên + lý do. Đối chứng: gỡ `ui-btn` khỏi một nút `GlossaryManageOverlay` → đỏ đúng nút đó, rồi trả lại.
- [x] `tests/frontend/` -- sửa test hiện có chỉ khi lớp móc đổi; mỗi lần sửa ghi lý do.
- [x] `deferred-work.md` -- mục B7: vòng focus `forced-colors` trên Windows.

**Acceptance Criteria:**
- Given Quản lý Glossary mở, when nhìn nút, ô tìm, ô chọn loại, chip, checkbox, radio ở hai theme, then mỗi cái đúng dáng hàng tương ứng của mockup.
- Given bàn phím, when Tab qua mọi control của lớp phủ và menu Công cụ, then mỗi control hiện dấu focus theo bảng §Trạng thái.
- Given ca vitest canh chuyển đổi, when gỡ lớp `ui-*` khỏi một control của năm tệp, then nó đỏ và chỉ đúng tệp:dòng.

## Implementation Notes

- `ui-seg__item`, `ui-chip`, `ui-menu-item` đọc trạng thái chọn/focus/khoá qua `:has(input:…)` để dùng được với nhãn bọc radio ẩn; nút mở menu `<summary>` khoá bằng `[aria-disabled='true']`.
- Công tắc trong mục menu (`menuitemcheckbox`) là `ui-switch__track`, bật theo `[aria-checked='true']` của chính nút; vòng focus của `ui-switch` nằm trên rãnh (miễn trừ `aura-allow-outline-none` có lý do).
- `ui-seg` chỉ có cao 24px, nên thanh lọc Glossary là ô tìm 28px cạnh nhóm tầng và chip 24px; không dựng bậc `ui-seg` mới.
- ⚠️ Lượt xoá CSS của agent dựng lặp lại lỗi của story 26: selector `.gm-seg-opt,` treo nhập vào `.gm-pills` (`border: none` xoá viền ô phân đoạn), `.gm-pill { border: outline }` đè viền chip, `.sources-name` bị nhân đôi thành gạch ngang mọi tên nguồn, `.sources-switch` cũ đè `ui-switch__track`. Test xanh vì ca canh chỉ đọc lớp trên chính control; nay đọc cả phần tử mang `ui-*` (nhãn, rãnh), đối chứng: trả `border` vào `.gm-pill` ⇒ đỏ `:1073 .gm-pill declares border`.
- `.gm-seg-opt` giữ `position: relative` và `inline-flex` (bố cục): `ui-seg__item` là `inline-grid`, xếp chữ và số đếm `<b>` thành hai hàng.
- Hàng ma trận có ca riêng: `controls-css.test.ts` (focus, thứ tự hover/focus, `forced-colors`, checkbox 24×24, dáng khoá) và `glossaryManage.test.ts` (Xoá chờ ⇒ `ui-btn--danger`; đang sửa ⇒ bộ lọc khoá trong lớp `ui-*`). Đối chứng: gỡ khối `forced-colors`, gỡ `:disabled` của ô tìm ⇒ đỏ đúng ca.
- e2e `titlebar-tools.e2e.mjs` xanh 7/7 trước các bản vá sau review; sau đó không chạy lại vì cổng 1420 đang bận (`tauri dev` của Ice).
- `check:debt-owner` cảnh báo `PLANNED_TICKET_FLOOR` 171 trôi dưới 80% của 214 ticket; không sửa trong story này.

## Plan Change Log

## Review Triage Log

Lượt 1 (quick): high 1 · medium 3 · low 4 · false 1 · maybe-false 0.

| Verdict | Route | Finding | Evidence |
|---|---|---|---|
| high | patch | Chip đang chọn mất dấu focus | `.ui-chip:has(input:checked)` (0,2,1) đứng sau `.ui-chip:has(input:focus-visible)` (0,2,1), trả viền về `on-surface` khi `outline: none`; Tab vào nhóm radio rơi đúng chip đang chọn |
| medium | patch | `forced-colors` thiếu chip bọc radio | Khối chỉ có `.ui-chip:focus-visible`; chip Glossary focus qua `:has(input:focus-visible)` với `outline: none` |
| medium | patch | `aria-pressed` trên `<summary>` | ARIA mới trên phần tử không vai nút, trái "aria-* giữ nguyên"; chỉ là móc dáng |
| medium | patch | Ca canh chỉ đọc compound cuối của selector | `selector.split(...).pop()`: `.gm-pill:hover input` lọt |
| low | patch | `.is-invalid` che focus | Đứng sau `:focus`/`:focus-visible` cùng độ đặc hiệu; chưa ai dùng |
| low | patch | Chip không `gap`, `▾` dính nhãn | `.ui-chip` thiếu `gap`; `.menu-trigger` gap 4px đã xoá; ảnh hưởng mọi chip `DropdownMenu` |
| low | patch | Hover Nguy hiểm (quyết định 2) không ca nào chạm | `controls-css.test.ts` không có ca hover |
| low | patch | Hai comment giải thích trong `controls-adoption.test.ts` | AGENTS.md §Code comments |
| false | — | `check:tokens` chưa chạy; px cố định | Đã chạy, xanh; Kiểm K chỉ canh padding/margin/gap; `descriptor.template.ast` là API công khai |

## Design Notes

- Mockup là nguồn số: chiều cao `--h-*` 32/28/24, đệm 12/8/6px, `ui-check input` 14px, rãnh switch 28×16. Đổi `px` mockup sang `--space-unit` khi là bội 4 (Kiểm K); 6px của `ui-field--sm` cần `aura-allow-spacing` có lý do.
- Mockup vẽ "Nguồn 8/10 ▾" bằng `ui-field--sm` ở hàng minh hoạ bậc; câu chữ DESIGN ("nút mở menu là nút Phụ hoặc Chìm, không phải dáng thứ ba") thắng, nên trigger giữ `ui-btn--sm`.
- Viền `ui-menu` theo mockup: `control-rule` thay `outline`.

## Verification

**Commands:**
- `npx vitest run tests/frontend/controls-adoption.test.ts tests/frontend/dropdownMenu.test.ts tests/frontend/selectMenu.test.ts tests/frontend/titlebarToolsMenu.test.ts tests/frontend/lookupSourcesPopup.test.ts tests/frontend/glossaryManage.test.ts tests/frontend/glossaryExchangeGate.test.ts tests/frontend/iconButtonLabels.test.ts` -- expected: mã thoát 0, không dòng Errors.
- `npm run check:tokens && npm run check:i18n && npm run check:commands` -- expected: xanh.
- `npm run test:e2e -- --spec e2e/specs/titlebar-tools.e2e.mjs` (ngoài sandbox, cổng 1420 trống) -- expected: xanh.

**Manual checks:**
- Ice: ảnh app thật lớp phủ Quản lý Glossary và menu Công cụ, sáng và tối, cạnh `he-dieu-khien.html`; màu checkbox thật (token hay màu nhấn macOS); font `<button>` trước và sau `font: inherit`.
