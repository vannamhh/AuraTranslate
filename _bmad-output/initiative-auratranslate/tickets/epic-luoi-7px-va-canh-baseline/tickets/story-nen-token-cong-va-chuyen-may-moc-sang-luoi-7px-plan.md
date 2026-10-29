---
title: 'Nền token, cổng và chuyển máy móc sang lưới 7px'
type: 'refactor'
ticket: '1'
created: '2026-10-29'
status: 'built'
baseline_revision: '29837138a603170f170a3684c8c685d73e7904d1'
route: 'full'
route_source: 'auto'
risk: 'high'
review: 'quick'
review_source: 'pinned'
lenses_ran: ['quick']
review_loop_iteration: 0
context:
  - '{project-root}/_bmad-output/initiative-auratranslate/ux-auratranslate/DESIGN.md'
  - '{project-root}/_bmad-output/initiative-auratranslate/plan-luoi-7px-va-canh-baseline.md'
  - '{project-root}/src/AGENTS.md'
  - '{project-root}/scripts/AGENTS.md'
  - '{project-root}/e2e/AGENTS.md'
---

<frozen-after-approval reason="human-owned intent — do not modify unless human renegotiates">

## Intent

**Problem:** Giao diện chạy trên đơn vị 4px (`tokens.json`, `check:tokens` Kiểm K) trong khi Ice đã chốt lưới 7px cho mọi khoảng cách, kích thước cố định và giãn dòng ngoài thân lưới Workspace (DESIGN.md §Lưới 7px). Chỉ đổi `unit` 4 → 7 thì mọi khoảng phình ×1,75.

**Approach:** Một commit đổi cùng lúc token, hai bậc điều khiển 28/35, Kiểm E (giãn dòng bội 7, sàn 1.6) và Kiểm K (đơn vị 7, mở tầm sang thuộc tính kích thước) theo plan §Cổng, rồi chuyển máy móc mọi hệ số `--space-unit`, px trơn và kích thước theo bảng §Lưới 7px — Thang. Thân lưới Workspace giữ số cũ qua token `--space-grid-*`.

## Boundaries & Constraints

**Always:** Giá trị lấy từ DESIGN.md §Lưới 7px, §Bảng token typography, §Bảng token khoảng cách, §Điều khiển. Giãn dòng lưu là tỉ số không đơn vị đủ chữ số (bội 7 ÷ cỡ chữ). Thân lưới (`GridBlock.vue`, `SourceHanViet.vue` phần ô, `--grid-columns` của `GridPanel.vue`) giữ đúng px hiện tại. Mọi giá trị lệch lưới còn lại mang `aura-allow-spacing: <lý do>`. Ca vitest ghim số px đổi theo token là ca di chuyển.

**Never:** Đổi chức năng, cỡ chữ, màu, bo góc, bề dày nét. Tinh chỉnh từng bề mặt (7/14 trong/giữa nhóm — entry 6–8), bề rộng `ch` (entry 2), canh baseline (entry 4), đo `trafficLightPosition` (entry 3), làm tròn giãn dòng Chế độ đọc (entry 5). Danh sách tệp loại trừ trong cổng. Hạ ngưỡng hay thêm miễn trừ để cổng xanh khi có thể sửa giá trị.

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected Output / Behavior | Error Handling |
|----------|--------------|---------------------------|----------------|
| Kiểm E bội 7 | token ngoài thân lưới, `fontSize × lineHeight` lệch bội 7 > 0,01px | đỏ | tên token + px tính được |
| Kiểm E sàn | token `wraps: true` ở 1.59 | đỏ | — |
| Kiểm E thân lưới | `source-cjk` 16.5 × 2.05 | xanh (khai tên + lý do) | — |
| Kiểm K kích thước | `width: 200px` · `height: 4px` · `grid-template-columns: 300px 1fr` | đỏ | gợi ý bội 7 gần nhất |
| Kiểm K kích thước | `width: 203px` · `height: 1px` · `minmax(0, 1fr) 301px` | xanh | — |
| Kiểm K đơn vị khác | `%` `fr` `vh` `ch` `em` `auto` `fit-content` `var()` | xanh | — |
| Kiểm K khoảng | `padding: 8px` (cũ hợp lệ) | đỏ | — |

</frozen-after-approval>

## Code Map

- `src/tokens/tokens.json` -- `spacing` (unit 7px, panel-inline/block 14, head 35, titlebar 42, status 35, gutter 21, traffic-light-inset 91, thêm `grid-*`), `typography.*.lineHeight` theo DESIGN; mục deviation `lookup-gloss`/`lookup-example` lineHeight chết, mục `traffic-light-inset` sửa lý do sang 91.
- `src/tokens/index.ts:106` -- phát `--space-<key>` từ `spacing`; key `grid-*` thành `--space-grid-*`, không cần sửa.
- `src/tokens/controls.css` -- bậc 24/28/32 → 28/35: xoá `ui-btn--sm`, `ui-icon-btn--sm`, `ui-field--sm`, `ui-seg--sm`; chip 24 → 28; `--lg` 35; đệm ngang nút 14, ô nhập 7; menu item đệm 14; checkbox vùng bấm 28, công tắc 28×14. Hình vẽ bên trong (tick 8×4, chấm 6, núm 10, thanh cuộn 10) nhận miễn trừ có lý do.
- 117 lần dùng `--sm` trong 31 tệp `src/**` và test -- gỡ lớp bổ nghĩa.
- `scripts/check-tokens.mjs:226` `EXPECTED_TYPOGRAPHY`, `:270` `EXPECTED_SPACING` -- đổi theo bảng; `:1400` Kiểm E; `:2037–2184` Kiểm K (`SPACE_UNIT_PX`, `GRID_PROPS`, `isOnGridToken`, `K_CASES`). Cuối tệp: chuỗi in "lưới 4px", ghi chú người rà soát.
- 50 tệp dùng `var(--space-unit)`: hệ số ×1:122 ×2:231 ×3:87 ×4:35 ×5:8 ×7:2 ×8:4 ×21:1 ×40:1 ×56:2 ×100:1 ×-1:1; 8 chỗ `var(--space-unit)` trần; `src/layout/dockview-theme.css:120` `calc(var(--space-unit) / 2)`.
- Thân lưới: `src/panels/GridBlock.vue` (`padding: 4px 8px`, `padding-bottom: 16px`, `padding: 4px 0`, `margin: 4px 0`), `src/panels/SourceHanViet.vue:971,975,1105` (`var(--space-unit)` = 4px) → `var(--space-grid-*)`. `GridPanel.vue:2080` `--grid-columns` giữ nguyên.
- `src-tauri/tauri.conf.json:24` `trafficLightPosition` -- KHÔNG chạm (entry 3).
- Vitest ghim px/tỉ số (17 tệp): `controls-css`, `checkTokensScan`, `panelHeaders`, `titlebarDragRegion`, `titlebarToolsMenu`, `glossaryManage`, `tmManage`, `libraryWorks`, `settingsFrame`, `settingsReadingDataBodies`, `readingState`, `readingTypography`, `layoutUserPresets`, `workspaceDockTier`, `dropdownMenu`, `reviewMode`, `attributionOverlayActions`.
- `e2e/specs/` -- spec mới đo trên WKWebView.

## Tasks & Acceptance

**Execution:**
- [x] `src/tokens/tokens.json` -- token mới theo Code Map; `lineHeight` = `String(px / fontSize)` -- R2, R3.
- [x] `scripts/check-tokens.mjs` -- bảng kỳ vọng; Kiểm E: phép bội 7 (miễn bốn token thân lưới có tên + lý do), sàn 1.6, ca tự kiểm ở Matrix; Kiểm K: unit 7, tầm 15 thuộc tính kích thước, chỉ xét số `px` kể cả trong hàm, |n| ≤ 3 hợp lệ ở thuộc tính kích thước, ca tự kiểm plan §Cổng + `padding: 8px` đỏ -- R1, R3.
- [x] `src/tokens/controls.css` + 31 tệp dùng `--sm` -- hai bậc -- R5.
- [x] 50 tệp `src/**/*.{vue,css}` -- chuyển bằng codemod một lần (script ở scratchpad, không commit): hệ số → px cũ → bảng Thang → hệ số mới; px trơn và kích thước theo cùng bảng; > 42px làm tròn tới bội 7 gần nhất (hoà lên); rà lại 17 miễn trừ cũ -- R1.
- [x] `GridBlock.vue`, `SourceHanViet.vue` -- dùng `--space-grid-*` -- R2.
- [x] 17 tệp vitest -- đổi số ghim theo token.
- [x] `e2e/specs/story-13-1-luoi-7px.e2e.mjs` -- trên WKWebView: dòng `ui-md` cao 21px, thanh panel 35px, padding một ô lưới bằng số cũ.

**Acceptance Criteria:**
- Given cây sau commit, when `npm run check:tokens`, then xanh và in số ca tự kiểm E, K mới.
- Given đối chứng gỡ thật phép bội 7 khỏi Kiểm E (hoặc trả `SPACE_UNIT_PX` về 4), when chạy lại `check:tokens`, then đỏ đúng ở ca tự kiểm của luật đó.
- Given app chạy, when spec e2e mới đo, then dòng `ui` 21px, thanh panel 35px, đệm ô lưới giữ số cũ.
- Given `src/**` trừ `selftest/`, when grep `--sm` của năm lớp điều khiển, then 0.

## Implementation Notes

- Phase 1 xong (5 Execution đầu), `npm run check:tokens` xanh: Kiểm E 8 ca tự kiểm, Kiểm K 26 ca, 1644 khai báo quét, 21 miễn trừ. Chưa chạy vitest/build; 17 tệp vitest + spec e2e còn lại cho pha sau. Chưa commit.
- Codemod (script scratchpad, không commit) chạy đúng một lần trên `<style>` và `.css` của `src/**` trừ `src/tokens/controls.css` (sửa tay). Bảng: 2·4·6·8→7, 12·16→14, 20·24→21, 28→28, 32·36→35, 40→42, bội 7 giữ, còn lại làm tròn tới bội 7 gần nhất (hoà lên). Hệ số `calc(var(--space-unit) * N)`: 1→1, 2→1, 3→2, 4→2, 5→3, 7→4, 8→5, 40→23, 56→32, 100→57, -1→-1. Phạm vi thuộc tính: padding/margin/gap (kể cả logical, theo cạnh), 15 thuộc tính kích thước của Kiểm K cộng `inline-size`/`block-size`, `flex`, `columns`, và biến tuỳ chỉnh `--*-height|width` (`--lookup-head-height` 40→42). Thuộc tính kích thước ≤3px không đổi.
- Làm tròn ngoài bảng (đã nằm trong cây, pha sau đừng ngạc nhiên): `max-width` 720→721 (10 chỗ) · 960→959 · 1100→1099 · 920→917 · 880→882 · 310→308 · 290→287 · 300→301 · 260→259 · 200→203 · 180→182 · 240→238 · 320→322 · 360→357 · 176→175 · 520→518 · 440→441 · 480→483 · 96→98 · 48→49 · 44→42 · 34→35 (3 chỗ) · 18→21 (`AppIcon.vue:57`) · 136→133 · 160→161 · 72→70 · 74→77 · 54→56 · 128→126 · 170→168.
- Sửa tay ngoài codemod: `controls.css` (xoá `ui-btn--sm`/`ui-icon-btn--sm`/`ui-field--sm`/`ui-seg--sm`; `--lg` 35; chip 28; `ui-check` min-height 28, `--bare` 28; công tắc 28×14, núm `top: 1px`; menu `min-width` 203; textarea `padding: 6px calc(unit*1)` kèm miễn trừ vì 6 + viền 1 = một bậc 7; tick 8×4, chấm 6, núm 10, thanh cuộn 10 mang `aura-allow-spacing`). `.ui-btn--pair { min-width: 8.5em }` và `.ui-chip--md` để nguyên (chưa phải 112px; DESIGN §Bề rộng đòi 112px — việc Story E hoặc pha sau, tuỳ Ice).
- `--dv-tab-group-line-height: calc(var(--space-unit) / 2)` (là bề dày vạch 2px) đổi thành `2px`. Hai chỗ mirror/gutter textarea (`PromptLibraryOverlay.vue` `.pl-ta-mirror` giữ `6px`, `LibraryUrlImportScreen.vue` `.us-gutter` giữ `7px`) bị codemod đổi 6→7 rồi trả lại tay để chữ trùng với textarea; đã bỏ miễn trừ cũ của gutter vì 7 đã là bội 7.
- Thân lưới: tokens `grid-cell-block 4px` · `grid-cell-inline 8px` · `grid-para-end 16px` · `grid-hv-gap 4px` trong `tokens.json` và khối riêng ở `EXPECTED_SPACING`. `GridBlock.vue` (4 chỗ) và `SourceHanViet.vue` (3 chỗ `var(--space-unit)`) dùng chúng; `--grid-columns` ở `GridPanel.vue` không chạm (biến tuỳ chỉnh, cổng không quét).
- `tokens.json`: `lineHeight` = `String(px / fontSize)` (read-lg 2.210526315789474 · read-md 2 · read-sm 1.75 · read-title 1.5217391304347827 · lookup-headword 1.4583333333333333 · lookup-gloss 1.9310344827586208 · mọi `ui-*` và `lookup-example` 1.6153846153846154). Hai deviation `lookup-gloss`/`lookup-example` đã xoá; `traffic-light-inset` 91px (sửa lý do, không chạm `tauri.conf.json`). Bốn token miễn Kiểm E bội 7: `source-cjk` · `source-latin` · `source-hanviet` · `editor`.
- Đối chứng gỡ thật đã làm và trả lại: `SPACE_UNIT_PX` 7→4 làm đỏ 4 ca K; bỏ nhánh bội 7 của `leadingProblems` làm đỏ 3 ca E.
- Pha sau phải biết: `--sm` còn trong `tests/frontend/{controls-css,libraryWorks,tmManage,titlebarToolsMenu,dropdownMenu,glossaryManage}.test.ts`; mọi ca ghim px/tỉ số cũ (24/28/32 điều khiển, 36/34/40 thanh, 12/16 đệm panel, giãn dòng 1.5/1.4/1.66...) là ca di chuyển. Kiểm E đã đổi tên tiêu đề ("giãn dòng: bội 7px và sàn 1.6"), `checkTokensScan.test.ts` có thể đọc chuỗi/hằng của cổng. Chưa quét `style=""` nội tuyến trong template và `.ts` (cổng cũng không quét `.ts`).
- Pha 2: chỉ 4 tệp vitest thật sự đỏ (controls-css, libraryWorks, titlebarToolsMenu; dropdownMenu dọn `--sm` còn sót); 13 tệp còn lại trong danh sách 17 vẫn xanh sau codemod nên không sửa. Hai ca `--sm` của controls-css thành một ca "chỉ còn hai bậc".
- Spec e2e `story-13-1-luoi-7px` (3 ca) xanh trên WKWebView ngoài sandbox; thanh panel đo `.dv-tabs-and-actions-container` lọc chiều cao > 0 (có nhóm ẩn cao 0). Đệm ô lưới đo `.cell-tgt` 4/8/8.
- Verification: `check:tokens` exit 0, `npm run build` + `npx vitest run` đủ bộ 242 tệp/3660 ca xanh, eslint e2e xanh. Chưa chạy `cargo test`, chưa chạy các e2e spec khác, chưa commit.

## Plan Change Log

## Review Triage Log

Lượt 1 (quick): 7 phát hiện + 1 tự thấy — medium 2 · low 3 · false 3 · patch 4 nhóm.

- medium · patch — `e2e/specs/titlebar-tools.e2e.mjs:91-92` ghim 40px/88px, token nay 42/91: spec đỏ ở nightly.
- medium · patch — `e2e/specs/story-12-24-library-wide-grid.e2e.mjs:322` ghim `coverHeight` 136, `.work-cover` nay 133.
- low · patch — `check-tokens.mjs:2328-2334` ghi chú người rà soát còn số thứ tự deviation cũ sau khi xoá hai mục.
- low · patch — `K_SIZE_CASES` gọi thẳng `offGridSizePx`, gỡ một thuộc tính khỏi `SIZE_PROPS` không làm ca nào đỏ (AGENTS.md: ca gọi thẳng hàm chỉ canh chính nó).
- low · patch (tự thấy) — doc comment `LEADING_GRID_BODY_EXEMPT` mang ngày, AGENTS.md cấm ngày trong mã.
- false — nợ `8.5em` → 112px: entry 8 (`tickets.toml`, "chân lớp phủ `min-width` 112px") đã sở hữu; `.ui-chip--md` không mang `8.5em` (chỉ `controls.css:63`).
- false — miễn trừ `6px` của textarea lệch trục ngang: đệm ngang ô nhập 7px là số của DESIGN.md §Điều khiển; miễn trừ chỉ phủ trục dọc và có từ trước.
- false — `style=""` nội tuyến và `.ts` chưa quét: chính người rà grep không thấy giá trị px nào; tầm quét của cổng có từ trước.

## Design Notes

Chuyển hệ số: `calc(var(--space-unit) * 2)` (8px) → 7px → `calc(var(--space-unit) * 1)`; `* 3` (12) → 14 → `* 2`; `* 56` (224) → `* 32`; `* 40` (160) → 161 → `* 23`. Vì chuyển theo px, không theo hệ số, phép chuyển không chạy hai lần được: chạy đúng một lần trên cây sạch.

Bội 7 của Kiểm E: `|fs × lh − 7·round(fs × lh / 7)| ≤ 0.01`. Token `wraps: false` vẫn chịu phép bội 7.

`--space-grid-*` vào `EXPECTED_SPACING` dưới một khối riêng dẫn DESIGN.md §Lưới 7px — Ngoại lệ, vì DESIGN khai chính cơ chế này; không qua `deviations`.

## Verification

**Commands:**
- `npm run check:tokens` -- expected: exit 0.
- `npm run build && npx vitest run` (đủ bộ: plan §Lộ trình, đổi `scripts/check-*`) -- expected: 0 failed, 0 Errors.
- `npm run test:e2e -- --spec e2e/specs/story-13-1-luoi-7px.e2e.mjs` ngoài sandbox -- expected: pass.
