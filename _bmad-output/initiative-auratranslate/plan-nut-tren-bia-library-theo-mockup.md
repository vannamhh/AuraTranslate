---
title: 'Nút "Tiếp tục dịch" và "⋯" trên bìa thẻ Library theo mockup'
type: 'bugfix'
ticket: ''
created: '2026-10-28'
status: 'built'
baseline_revision: '47b16b4a43b88e4947d61c10ccd12ddfb22843df'
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

**Problem:** Commit `8d4c4081` (12-32) chuyển nút "Tiếp tục dịch" trên bìa thẻ Library sang `ui-btn--sm ui-btn--primary`. Nút đó nền mực, đặt trên bìa tối nên chìm vào bìa. Nút "⋯" dùng `ui-btn--quiet`: chữ xám, không nền, gần như không thấy, và cao 28px so với 24px của nút bên cạnh.

**Approach:** Ice chọn giữ dáng của mockup `ux-auratranslate/mockups/library-luoi-rong.html` (2026-10-28, "giữ như mockup đẹp hơn"):
- **"Tiếp tục dịch" trên thẻ đang chọn:** nền `primary`, chữ `on-primary`, đậm.
- **"Tiếp tục dịch" trên thẻ đang rê chuột:** nền `surface`, chữ `primary`.
- **"⋯":** nền `surface`, chữ `on-surface-variant`.

Hai nút cùng bậc `sm` 24px. Theo §Điều khiển "Một nguồn", dáng nằm ở `controls.css` dưới dạng bổ nghĩa `ui-btn--cover` và `ui-btn--cover-on`, không đặt trong `<style scoped>`.

</frozen-after-approval>

## Implementation Notes

Tuyến oneshot: khoảng 40 dòng ở `controls.css`, `DropdownMenu.vue` (thêm prop lớp cho trigger), `LibraryMode.vue`, cộng test.

- Mockup tô `⋯` bằng `ink-2`, không đậm, nên cần bổ nghĩa thứ ba `ui-btn--cover-plain`; dùng chung `ui-btn--cover` thì ra chữ `primary`.
- `DropdownMenu` nhận `triggerClass` cho dạng `more`, mặc định vẫn `ui-btn--quiet`.
- Focus của `ui-btn--cover-on` là vòng 2px `on-cover`, vì đổi viền sang `primary` thì không thấy trên nền `primary`.
- Đo trên hai thẻ dựng tay có thuộc tính scoped của `LibraryMode` (cổng 1420, dpr 2, theme sáng). Bốn nút cùng `top`, cao 24. Nền và chữ: thẻ chọn `#2f5d63`/`#fbfaf6`, thẻ rê chuột `#fbfaf6`/`#2f5d63`, `⋯` `#fbfaf6`/`#6b6459`. Phần kiểm trong app thật ghi nợ, Chủ: Epic 12.

## Review Triage Log

Lượt `quick` 1: medium 2 · low 3 · false 1.
- medium, patch: lúc bấm, `.ui-btn:active` (0,2,0) đè nền `primary` của `cover-on` bằng nền trũng, chữ sáng chìm. Thêm `:active` cho cả ba bổ nghĩa, giữ nền.
- medium, patch: lúc khoá (`libraryOpenWorkBusy`), `.ui-btn:disabled` vẽ viền đứt không nền lên bìa tối. Thêm `:disabled` giữ nền, đặt sau luật chung. Đo trên trang: nền và viền giữ nguyên.
- low, patch: test chưa canh `:active`/`:disabled`/vòng focus `on-cover`. Đã thêm ca.
- low, patch: mặc định `ui-btn--quiet` của `triggerClass` chưa có test. Đã thêm ca ở `dropdownMenu.test.ts`.
- low, reject: rê chuột đổi viền sang `on-surface-variant` là dấu hover chung của §Điều khiển, mockup không vẽ trạng thái rê.
- false: theme tối chưa đo. Đã nằm trong mục nợ (Chủ: Epic 12), không phải lỗi của diff.

## Verification

**Commands:**
- `npx vitest run tests/frontend/controls-css.test.ts tests/frontend/libraryWorks.test.ts tests/frontend/dropdownMenu.test.ts` -- expected: xanh

**Manual checks (if no CLI):**
- Trong app thật: thẻ đang chọn có nút nền `primary`; thẻ đang rê chuột có nút nền `surface` chữ `primary`; "⋯" nền `surface`, cùng cao 24px với nút bên cạnh.
