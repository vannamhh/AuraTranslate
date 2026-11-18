---
title: 'Lớp phủ Glossary, TM và quy ước theo khung chung'
type: 'refactor'
ticket: '3'
created: '2026-11-18'
status: 'built'
baseline_revision: '31929d9bf36bd870db51e23880d171dcf210366f'
route: 'full'
route_source: 'auto'
risk: 'medium'
review: 'quick'
review_source: 'pinned'
lenses_ran: ['quick']
review_loop_iteration: 0
context:
  - '{project-root}/src/AGENTS.md'
  - '{project-root}/e2e/AGENTS.md'
---

<frozen-after-approval reason="human-owned intent — do not modify unless human renegotiates">

## Intent

**Problem:** Chín lớp phủ Glossary/TM/quy ước còn tự chép khung với năm bề rộng (721 · 882 · 917 · 1099 · 1120px), cả khung cuộn (`overflow: auto` trên `*-panel`) lại có danh sách tự cuộn bên trong (`.gh-list` và `.crv-list` 60vh), và nằm trong danh sách nợ của Kiểm L.

**Approach:** Mỗi tệp mang bốn lớp khung của 17.1 (`ui-overlay-scrim` · `ui-overlay` · `__head` · `__body` · `__foot`) cạnh lớp scoped riêng; đầu = phần đứng trước nội dung (tiêu đề, ghi chú, thanh lọc/tab/thanh chọn, phụ đề), chân = khối nút cuối màn cùng dòng gợi ý/tóm tắt sát nó; danh sách bỏ cuộn riêng; ô chữ giãn theo R3. Xoá chín dòng nợ.

**Decision (Ice 2026-11-18, phương án A):** chân Quản lý Glossary chỉ là `gm-foot` (Thêm thuật ngữ + gợi ý); `gm-actions` và `gm-exchange` cuộn trong thân. Quản lý TM không có chân. Bác B (chân ba khối ≈ 160px, thân ở 800×600 còn ≈ 250px).

**Decision (Ice 2026-11-18):** giữ trọn phạm vi chín tệp dù plan vượt 1.600 token.

## Boundaries & Constraints

**Always:** DOM order và thứ tự Tab giữ nguyên: đầu chỉ nhận khối liền đầu màn, chân chỉ nhận khối liền cuối, không dời khối nào qua khối khác. Giữ lớp scoped (`gm-panel`, `.gq-actions`…) để selector của test không đổi; giữ z-index và miễn trừ của từng scrim. Nội dung, nhánh trạng thái, Esc, `trapTab`, tiêu điểm trả về nút mở, phím không đổi. Nhánh không có nút (đang tải, lỗi, rỗng) không vẽ chân.

**Never:** Sửa `controls.css`, token, hay luật Kiểm L (chỉ xoá dòng nợ); chuyển lớp phủ của entry 4–5; dời khối để có chân "đẹp hơn"; rút `focusableWithin`/`trapTab` (AI-11); đổi chiều cao lớp phủ.

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected Output / Behavior | Error Handling |
|---|---|---|---|
| Danh sách dài | Thu hoạch, Thay cả Chương với nhiều hàng | danh sách cao theo nội dung, chỉ `__body` cuộn, chân thấy ở đỉnh và đáy | — |
| Mũi tên trong danh sách | con trỏ hàng xuống ngoài khung nhìn | `__body` cuộn theo hàng (`scrollIntoView` hiện có) | — |
| Mẫu TM rộng | `pre.tm-mono` dài hơn khung | cuộn ngang trong `pre`, không thành vùng cuộn dọc | — |
| Menu chọn đóng | `SelectMenu` trong khung, đang đóng | helper e2e không đếm nó | — |
| Lỗi tải | `status === 'error'` | đầu + thân báo lỗi, không chân | Esc/nút đóng ở đầu |

</frozen-after-approval>

## Code Map

- `src/{GlossaryManage,GlossaryImport,GlossaryHarvest,GlossaryQueue,TmManage,TmImport,ConventionManage,ConventionImport,ConventionReplace}Overlay.vue` -- scrim `div.*-scrim` > `section.*-panel`; CSS khung riêng (`max-width` + `max-height:100%` + `overflow:auto` trên `*-panel`, khối `.*-scrim` + quy tắc mac) bị thay bởi lớp khung; z-index ở lại trong `.*-scrim`.
- Khuôn đã chuyển: `src/PromptImportOverlay.vue:105-257` (chân `v-if` theo nhánh có nút, đầu cao 35) và `src/SettingsOverlay.vue:337-426`.
- Đầu (khối liền đầu): gm `gm-head`+`gm-note`+`gm-toolbar` · tm `tm-head`+`tm-note`+`tm-toolbar` (`tm-health` thuộc thân) · cvm `cvm-head`+`cvm-tabs` · crv `crv-head`+`crv-bar` · gq `gq-head`+`gq-subtitle` · gh `gh-head`+`gh-subtitle` · gi/ti/cvi chỉ `*-head`.
- Chân (khối liền cuối): gq `.gq-actions`+`.gq-footer` (bỏ mẹo `sticky; bottom:-13px`) · gh `.gh-actions`+`.gh-hint` · gi `.gi-actions` · ti `.ti-actions` · cvi `.cvi-actions` · cvm `footer.cvm-foot` · crv `.crv-foot`+`.crv-confirm`+`footer.crv-actions` · gm `.gm-foot` · tm không chân. Năm màn gq · gh · gi · ti · cvi có khối cuối nằm trong `<template v-else-if>` ⇒ chân lặp điều kiện đó như PromptImport.
- Cuộn riêng: `.gh-list` `GlossaryHarvestOverlay.vue:316`, `.crv-list` `ConventionReplaceOverlay.vue:584` (bỏ `max-height`/`overflow`); `.tm-mono` `TmManageOverlay.vue:1474` `overflow-x:auto` tính ra y `auto` ⇒ thêm `overflow-y: hidden` (giữ cuộn ngang, hết vùng cuộn dọc).
- R3: `.gm-field-search` `:1104` và `.cvm-field-search` `:695` bỏ `max-width: 294px`, giữ `flex: 1`; ô chữ khác trong form/ô bảng đã `width:100%`. Không ô số nào trong chín màn.
- `scripts/check-tokens.mjs:2419-2428` -- xoá chín dòng `entry 3`; Kiểm L in "nợ 7".
- `e2e/support/overlayFrame.mjs` `measureOverlayFrame` -- bỏ phần tử không có hộp (`getClientRects().length === 0`) trước khi đếm; đóng mục nợ `Chủ: Story 17.3` ở `deferred-work.md` (dòng ~14267). Lọc `menu-panel` trong `e2e/specs/story-17-2-cai-dat-va-thu-vien-prompt.e2e.mjs:59` thành thừa, xoá.
- Đường mở e2e: `e2e/specs/story-12-14-tm-only-mine.e2e.mjs:105` (TM), `story-16-2-convention-replace-chapter.e2e.mjs:107` (Thay cả Chương), `glossary-mouse.e2e.mjs` (Glossary). Khuôn spec: `e2e/specs/story-17-1-khung-lop-phu.e2e.mjs`.
- `tests/frontend/glossaryQueue.test.ts:701-713` đọc nguồn và đòi `.gq-actions` sticky ⇒ đổi thành `.gq-actions` nằm trong `.ui-overlay__foot`; các test khác chọn theo lớp scoped, giữ nguyên.

## Tasks & Acceptance

**Execution:**
- [x] chín tệp `src/*Overlay.vue` ở Code Map -- thêm lớp khung theo bảng đầu/chân, bọc phần giữa trong `div.<tiền tố>-scroll.ui-overlay__body`, xoá CSS khung riêng và cuộn riêng, R3 ô tìm -- R1/R3/R4.
- [x] `scripts/check-tokens.mjs` -- xoá chín dòng nợ.
- [x] `e2e/support/overlayFrame.mjs`, `e2e/specs/story-17-2-…e2e.mjs`, `deferred-work.md` -- helper bỏ phần tử không hộp, xoá lọc thừa, đóng mục nợ bằng `→ ✅ ĐÃ ĐÓNG 2026-11-18 (Story 17.3)`.
- [x] `e2e/specs/story-17-3-glossary-tm-quy-uoc.e2e.mjs` -- spec theo `verify` của ticket.
- [x] `tests/frontend/glossaryQueue.test.ts` -- assert `.gq-actions` nằm trong chân khung thay assert sticky; tám tệp còn lại không thêm ca vitest vì Kiểm L canh nguồn và e2e đo DOM.

**Acceptance Criteria:**
- Given cửa sổ 1280px trên WKWebView, when mở Quản lý Glossary, Quản lý TM và Thay cả Chương, then mỗi khung rộng 1120 ±1 và `measureOverlayFrame` trả đúng một phần tử cuộn dọc, `nested` 0.
- Given cổng chạy trên cây, when chín tệp đã dùng khung, then `npm run check:tokens` xanh và in "nợ 7".
- Given đối chứng gỡ thật (`.crv-list` lấy lại `max-height: 60vh; overflow: auto`), when chạy Kiểm L và ca Thay cả Chương, then cả hai đỏ vì vùng cuộn lồng.

## Implementation Notes

- Đầu khung là `div.<tiền tố>-top.ui-overlay__head` bọc `header` cũ cùng thanh lọc/tab, không gắn lớp khung lên `header`: `controls-adoption.test.ts` cấm `height`/`border` trên phần tử mang lớp `ui-*`, mà mọi đầu read-title khai cao 35.
- Cùng lỗi đó đã đỏ ở baseline `31929d9b` trên `.pi-head`, `.pl-head`, `.set-head` (17.1, 17.2, chưa push). Ice chọn bọc giống 17.3 thay vì miễn trừ có tên: ba tệp đổi DOM, hai assert vitest theo `.pl-top`/`.set-top`, ca `.set-head` của e2e 13-9 đo bậc 7 trên `.set-top` vì viền dưới nay thuộc khung.
- `npm run build` cũng đỏ ở baseline: bốn lỗi kiểu JSDoc trong Kiểm L (`last`, `asMap`), sửa kiểu chứ không đổi luật.
- Thay cả Chương giữ đệm 13px và nền riêng của từng khối, nên khe khung để đệm 0; các vạch giữa đầu/thanh chọn và giữa ba khối chân không còn, chỉ còn vạch của khung.
- `measureOverlayFrame` phải bỏ cả phần tử nằm trong `details` đang đóng: WKWebView vẫn cấp hộp cho `.menu-panel` ẩn, riêng `getClientRects()` không đủ.
- Đối chứng gỡ thật: trả `max-height: 60vh; overflow: auto` cho `.crv-list` ⇒ Kiểm L đỏ "nested scroll region", ca Thay cả Chương của spec 17-3 đỏ `nested` 1 thay vì 0; khôi phục rồi `touch`.
- Bảng I/O: hàng lỗi tải và chân của Thu hoạch có ca vitest; hàng danh sách dài và mũi tên đo trên Thay cả Chương với khung ép thấp 280px trong spec 17-3.

## Plan Change Log

## Review Triage Log

Lượt 1 (quick) — high 0 · medium 0 · low 1 · false 6 · maybe-false 0.

- low · patch — `ConventionReplaceOverlay.vue` mất ba vạch trong: giữa `.crv-head` và `.crv-bar`, và giữa các khối chân xếp chồng (`.crv-foot`, `.crv-confirm`, `.crv-actions`); vạch đặt ở `.crv-head + .crv-bar` (không trên `.crv-head`, để không thành vạch đôi với vạch khung khi thiếu thanh chọn) và `border-top` cho khối chân đứng sau một khối chân khác.
- false — "chân Thu hoạch hẹp hơn nhánh cũ khi `error` mà `harvestLoadError` null": `glossaryHarvestState.ts:60-61,81-82` luôn đặt `loadError` khác null cùng lúc với `error`, nên còn lại chỉ `loaded` có hàng.
- false — "chân Bảng chờ dựa vào ghép ngầm với `queueEmptyReason`": `queueEmptyReasonFor` (`glossaryQueueState.ts:422-431`) trả null đúng khi nhánh `v-else-if="queueStatus === 'loaded'"` cũ được vẽ; phần "thiếu ca vitest chân vắng" không có lỗi đi kèm.
- false — "tên ca `.set-head` của 13-9 sai và sửa 13-9 ngoài phạm vi": `.set-head` vẫn đo hộp nội dung 28, `.set-top` cao 55 cộng viền khung 1 là 56 như tên ca; phạm vi mở rộng là lựa chọn A của Ice.
- false — "Cài đặt có thể mất lề ngang": `.ui-overlay__head` cấp `padding: 0 13px` (`controls.css:785`) cho `.set-top`/`.pi-top`/`.pl-top`; e2e 13-9 và 17-2 xanh.
- false — "mở TM bằng tổ hợp phím cứng": cùng tổ hợp `Meta+Alt+b` với `story-12-14-tm-only-mine.e2e.mjs:104`, e2e chỉ chạy trên macOS.
- false — "lọc `details:not([open])` giấu vùng cuộn thật": nội dung `details` đang đóng không hiện cho người dùng, nên không phải vùng cuộn người dùng gặp; mở ra thì được đo lại.

## Verification

**Commands:**
- `npm run check:tokens` -- exit 0, Kiểm L "nợ 7".
- `npx vitest run tests/frontend/glossaryQueue.test.ts tests/frontend/glossaryManage.test.ts tests/frontend/glossaryHarvest.test.ts tests/frontend/glossaryImportPreview.test.ts tests/frontend/tmManage.test.ts tests/frontend/tmImport.test.ts tests/frontend/conventionReplaceOverlay.test.ts tests/frontend/conventionImport.test.ts tests/frontend/dialogAccessibleName.test.ts tests/frontend/controls-adoption.test.ts` -- xanh, mã thoát 0.
- `npm run test:e2e -- --spec e2e/specs/story-17-3-glossary-tm-quy-uoc.e2e.mjs` rồi `--spec e2e/specs/story-17-2-cai-dat-va-thu-vien-prompt.e2e.mjs` -- xanh, ngoài sandbox, bản `--features wdio`.
- `npm run check:lint` và `npx vue-tsc --noEmit` -- xanh.
