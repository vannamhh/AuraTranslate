---
title: 'Rà bề mặt Cài đặt và 22 lớp phủ theo lưới 7px'
type: 'refactor'
ticket: '8'
created: '2026-10-30'
status: 'built'
baseline_revision: '2218d8054c5e4f9a84d678108097801d9293987a'
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

**Problem:** Phép chuyển máy móc của entry 1 để Cài đặt và 22 tệp `*Overlay.vue` lệch lưới: khung lớp phủ và khối có viền cộng nét vào đệm 14/7 (mép chữ cách mép ngoài 15, 8), khe giữa các nút trong một hàng còn 14, cặp nút chân lớp phủ `ui-btn--pair` rộng `8.5em` (110,5px), và các bề rộng `em`/`rem` (8rem, 10em, 12em, 16rem …) ra số lẻ mà Kiểm K không thấy (DESIGN.md §Lưới 7px — Trong nhóm 7, giữa nhóm 14 · Nét kẻ nằm trong ô lưới · Bề rộng).

**Approach:** Một spec e2e đo Cài đặt và ba lớp phủ đại diện trên WKWebView trước, rồi sửa CSS của Cài đặt, 22 lớp phủ và `.ui-btn--pair` cho tới khi spec xanh.

**Decision (Ice 2026-10-30):** chân lớp phủ giữ dáng hôm nay (canh trái, nút chính trước, không nét trên, nằm trong dòng nội dung); chỉ đổi `min-width` 112 và khe 7. Mockup mục 5 chỉ minh hoạ nhịp 7/14 và cỡ điều khiển. Phương án bị bác: dải chân theo mockup (nét trên, đệm `13 14 14`, canh phải, Huỷ trước).

**Decision (Ice 2026-10-30):** hàng thiết lập giữ cấu trúc từng mục hôm nay; chỉ áp bù nét, 7/14 và bề rộng bội 7. Phương án bị bác: một hàng chung `.srow` (`1fr auto`) cho mọi mục.

## Boundaries & Constraints

**Always:** Khối có viền 1px trừ 1px ở đệm cùng phía; vạch trái 2/3px trừ bề dày vạch ở đệm trái. Khe giữa các nút cùng hàng và giữa nhãn với điều khiển của nó là 7; giữa hai nhóm (khối, hàng nút, gợi ý đẩy sang phải) là 14. Mỗi giá trị lệch lưới vì bù nét mang `/* aura-allow-spacing: <reason> */` cùng dòng. Bề rộng `em`/`rem` cố định thành px bội 7, làm tròn lên từ số px tính được trên WKWebView. `.ui-btn--pair` `min-width: 112px` ở `controls.css`.

**Never:** Thêm token `--space-*` chỉ để lách Kiểm K. Đổi cỡ chữ, màu, chiều cao điều khiển, thứ tự tab hay hành vi lưu. Chạm Workspace, Tra cứu, Library, Chế độ đọc (entry 6, 7). Đổi khe cột 21 của bố cục hai cột (`.ip-two`, `.bip-two`, `.aip-split`), vì nó đã đúng lưới.

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected Output / Behavior | Error Handling |
|----------|--------------|---------------------------|----------------|
| Hàng thiết lập | Cài đặt, mục Đọc và Bố cục | cao ngoài mỗi `.sr-row`/`.sl-item` bội 7; bề rộng `.sr-label`/`.sl-name` bội 7; mép chữ đầu của `.set-main` cách mép ngoài `.set-panel` một bội 7 | — |
| Chân lớp phủ | Lớp phủ Thu hoạch Glossary (`.gh-actions`) | hai nút `ui-btn--pair` cao 35, mỗi nút rộng ≥ 112 (±0,5), khe giữa chúng 7 | — |
| Khung lớp phủ | Lớp phủ Quản lý TM | mép chữ đầu cách mép ngoài `.tm-panel` 14 ở cả trên lẫn trái; khe giữa các nút `.tm-actions` 7 | — |
| Hàng có viền | Lớp phủ Căn hàng (`.al-row`) | cao ngoài mỗi hàng bội 7 | — |
| Bẫy | trả `.ui-btn--pair` về `min-width: 8.5em` | ca chân lớp phủ đỏ | — |

</frozen-after-approval>

## Code Map

- `e2e/specs/story-13-7-ra-be-mat-library-doc.e2e.mjs`, `story-13-6-…` -- khuôn `onSeven`, đo trong `browser.execute`. Mở Cài đặt: `openSettingsSection(name)` ở `story-13-2-be-rong-ch.e2e.mjs:17` (`[data-settings-open]` → `[data-tools-item="settings.open"]` → `[data-settings-section]`). Mở lớp phủ qua menu Công cụ: `data-tools-item="glossary.harvest.open"`, `tm.manage.open`, `export.alignment.open` (`TitlebarToolsMenu.vue:110–190`); Workspace qua `openWorkspaceWithWork`.
- `src/tokens/controls.css:62` `.ui-btn--pair` `min-width: 8.5em` -- dùng ở mọi chân hai nút. `scripts/check-tokens.mjs:2331` chỉ dùng `8.5em` làm chuỗi thử, không đổi.
- Settings (`SettingsOverlay.vue`): `.set-head:959` đệm 14 + nét đáy ⇒ đáy 13; `.set-nav-item:989` vạch trái 2 + đệm 14 ⇒ 12; `.set-scope:1082`, `.set-table th/td:1039/1049` đệm 7 + nét đáy ⇒ 6; `.set-tag:1066`, `.ai-preset-confirm:1153` viền 1 quanh đệm; `.ai-field:1188` đệm 7 + nét đáy; `.ai-actions:1184`, `.ai-key-actions:1231` khe 14 giữa nút ⇒ 7. `.set-grid` `203px 1fr` đã đúng.
- Section: `SettingsReadingSection.vue:118` `.sr-row`, `SettingsLayoutSection.vue:142` `.sl-item` đệm đáy 14 + nét ⇒ 13; `.sd-actions` (Data :89) khe 14 ⇒ 7; `.ts-field` (Export, Tm), `.gs-field` (Glossary) nhãn → ô 14 ⇒ 7; `.sc-table th/td` (Shortcuts :380/392) đệm đáy 14 + nét ⇒ 13; vạch trái 2 + 14 của `.sr-note`, `.sl-note`, `.sc-note` và các alert `ts`/`gs`/`su` ⇒ 12; `min-width` `10em` (Data :70, Reading :128) và `12em` (Layout :152).
- Khung lớp phủ: mọi `.xx-panel` đệm 14 + viền 1 ⇒ 13 (tm 1005, pl 959, gm 900, gq 482, al 317, hist 383, ex 636 và mọi panel nhóm B: cs 171, cm 176, ti 176, pfmem 231, ri 250, gh 262, gi 277, pi 279, pfscope 129, lib 344). `aip-panel`, `ip`, `bip`, `attr-panel` không viền, giữ.
- Khối có viền trong lớp phủ (bù nét cùng phía): ip 1761, 1833, 1855, 1885, 1994, 2124, 2177, 2295, 2377, 2415, 2445; bip 837, 893, 916, 1013, 1036, 1084, 1096; tm 1226, 1231, 1363, 1380, 1401, 1443, 1458; pl 1011, 1076, 1116, 1128, 1245; gm 1002/1013, 1187, 1238; aip 799, 824, 843, 945, 1004, 1028, 1053, 1120, 1141; gq 533, 571 (`padding-block`), 630; ex 689, 787, 801, 815; attr 439/457, 560, 582; hist 421, 443, 463, 498; al 378; cs 216; cm 205, 227; pfmem 284; gh 319; gi 386; pi 367; pfscope 168; lib 441. `pl` 1196 đã có miễn trừ.
- Khe 14 giữa nút cùng hàng ⇒ 7: `.tm-actions:1390`, `.gm-actions:1248`, `.gq-actions:841`, `.hist-confirm-acts:451`, `.gh-actions:393`. Giữ 14 giữa hàng nút và gợi ý đẩy phải (`.gm-foot`, `.attr-foot`, `.tm-hint`).
- `em`/`rem`: ip 2346 `12rem`; tm 1051/1052, 1107, 1177, 1185, 1224 (`9rem 6rem`), 1354; gm 948, 1090, 1211; gq 701, 764; al 394 (`3rem`), 440; `dt` `8rem` ti 237, ri 311, gi 338, pi 341; pi 375 `max-height: 12rem`; `min-height: 1em` của dòng lỗi cs 244, cm 272, lib 459/468 ⇒ một dòng `ui` 21px.
- `controls-adoption.test.ts:17` cấm `height`/`min-height`/`border` ở luật scoped trên lớp điều khiển; `min-width` được.

## Tasks & Acceptance

**Execution:**
- [ ] `e2e/specs/story-13-8-ra-be-mat-cai-dat-lop-phu.e2e.mjs` -- các ca của Matrix trước mọi sửa; ghi ca nào đỏ trên cây gốc -- R1, R6.
- [ ] `src/tokens/controls.css` -- `.ui-btn--pair` 112px -- R6.
- [ ] `src/SettingsOverlay.vue` và 8 tệp `src/Settings*Section.vue` -- bù nét, 7/14, bề rộng bội 7 theo Code Map -- R1, R6.
- [ ] 21 tệp `src/*Overlay.vue` còn lại -- khung, khối có viền, khe nút, `em`/`rem` theo Code Map -- R1, R6.

**Acceptance Criteria:**
- Given app chạy ngoài sandbox, when spec `story-13-8` chạy, then xanh; given phép gỡ ở hàng Bẫy, then đỏ đúng ca chân lớp phủ.
- Given cây sau story, when `npm run check:tokens`, then xanh, và mỗi miễn trừ `aura-allow-spacing` mới nằm trên một khai báo bù đúng bề dày nét cùng phía (số miễn trừ ghi ở Implementation Notes).
- Given `src/*Overlay.vue` và `src/Settings*.vue`, when grep `[0-9.]r?em` ở thuộc tính kích thước, then không còn chỗ nào ngoài chữ.
- Given spec `story-12-18-settings`, `story-12-14-tm-only-mine`, `titlebar-tools`, `story-13-2`, when chạy lại, then xanh, hoặc ca đỏ được khoanh là đỏ cả trên cây gốc.

## Implementation Notes

- Cây gốc đỏ cả năm ca của spec `story-13-8`; trả `.ui-btn--pair` về `8.5em` thì chỉ ca chân lớp phủ đỏ.
- Ca chân Thu hoạch Glossary và ca `.al-row` dựng phần tử thử mang thuộc tính scoped của panel: nút thật của `.gh-actions` và hàng `.al-row` chỉ có khi đã nạp bản người soát, mà bước đó mở hộp chọn tệp gốc WebDriver không lái được. Hai ca đo CSS đã giao, không đo dữ liệu lớp phủ.
- 116 dòng mới mang `aura-allow-spacing`, mỗi dòng bù đúng bề dày nét cùng phía (1px viền, 2/3px vạch trái); `.set-head`, `.set-main` và `.set-nav-item` bù thêm 1px viền của `.set-panel`.
- Bảng `border-collapse` (`.ex-word-table`, `.set-table`, `.gm-list`, `.attr-table`, `.sc-table`) chỉ trừ ở đệm đáy: nét giữa hai hàng tính một lần, nên mỗi hàng cao đệm trên + chữ + đệm đáy + 1.
- `em`/`rem` đổi sang px làm tròn lên bội 7 từ rem 16px và chữ `ui` 13px (8rem → 133, 10em → 133, 12em → 161, 26rem → 420); `min-height: 1em` của dòng lỗi thành 21px.
- Hàng thanh trượt `.sr-row` cao 54/50 vì ô `range` mang chiều cao và lề riêng của WebKit; `.sr-range` nay `height: 21px; margin: 0; align-self: center`, `.sr-value` canh giữa theo, nên số giá trị có thể lệch baseline ~1px.
- Khe nhãn → điều khiển của `.sr-row`, `.sl-item`, `.sd-row`, `.ts-row` và khe các thanh công cụ (`gm-toolbar`, `tm-toolbar`, `pl-bar`, `aip-setbar`) giữ 14: Code Map không liệt kê.
- `story-12-18-settings` đỏ một ca (chờ 5 `[data-ai-provider]`, có 9) và `story-12-14-prompt-library` đỏ ở phép so bề rộng mirror/textarea, cả hai đỏ y như vậy trên cây gốc.

## Plan Change Log

## Review Triage Log

Lượt 1 (quick): 7 dòng (6 phát hiện, phát hiện 1 tách theo hàng) — low 5 (patch 2 · patch hoàn lại 1 · reject 2) · false 2.

- low · patch — `SettingsReadingSection.vue` `.sr-row` khe 14 nằm giữa nhãn và điều khiển của chính nó (nút phân đoạn; thanh trượt + số), trái luật Always 7.
- false — `.sl-item`, `.sd-row`, `.ts-row` khe 14: hàng nối nhóm thông tin (tên · thẻ · phím; nhãn · đường dẫn; ô có nhãn) với nhóm nút, nên 14 là khe giữa nhóm; khe nhãn → ô bên trong `.ts-field` đã 7.
- false — `.gm-toolbar`, `.tm-toolbar`, `.pl-bar`, `.aip-setbar` khe 14: mỗi thanh gồm các nhóm điều khiển riêng (ô tìm, bộ lọc phân đoạn, ghi chú, nút đóng), không phải một hàng nút.
- low · patch, hoàn lại — `.sr-value` `align-self: center` đưa số khỏi baseline của nhãn. Bỏ khai báo đó thì hai hàng cỡ chữ và giãn dòng cao 50 (số `mono` theo baseline cạnh ô `range` 21px), lệch lưới 1px, nên đã trả lại; giữ ~1px lệch baseline như Implementation Notes ghi.
- low · reject — ca chân Thu hoạch Glossary và `.al-row` đo phần tử thử, hàng thử của Căn hàng chỉ có `.al-row-label` một dòng: dựng hàng thật cần nạp bản người soát qua hộp chọn tệp gốc (Implementation Notes).
- low · patch — ca TM: `gaps` ghi `null` cho cặp xuống dòng mà `gaps.length > 0` đếm cả `null`, nên ca xanh mà không đo khe nào khi mọi nút xuống dòng; đòi số khe khác `null` > 0.
- low · reject — `expect(onSeven(x) ? 0 : x).toBe(0)` khó đọc: viết vậy để lúc đỏ thông báo in giá trị lệch; đổi chỉ là đổi kiểu viết.

## Design Notes

Đo từ mép ngoài khối có viền: viền 1 + đệm 13 = 14. Bù nét như Story 13.7:

```css
.tm-panel {
  padding: 13px; /* aura-allow-spacing: 1px border sits inside the 14px inset */
}
```

## Verification

**Commands:**
- `npm run check:tokens` -- expected: exit 0.
- `npm run test:e2e -- --spec e2e/specs/story-13-8-ra-be-mat-cai-dat-lop-phu.e2e.mjs`, rồi các spec ở AC thứ tư, ngoài sandbox -- expected: pass.
- `npx vitest run tests/frontend/controls-css.test.ts tests/frontend/controls-adoption.test.ts` và mọi tệp test import một tệp đã sửa -- expected: 0 failed, 0 Errors.

**Manual checks (if no CLI):**
- Không có; lượt nhìn app thật của Ice thuộc mục nợ `Chủ: Epic 13`.
