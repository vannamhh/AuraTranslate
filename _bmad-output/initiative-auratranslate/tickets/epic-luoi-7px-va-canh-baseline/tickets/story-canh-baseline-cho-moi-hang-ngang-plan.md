---
title: 'Canh baseline cho mọi hàng ngang'
type: 'refactor'
ticket: '4'
created: '2026-10-29'
status: 'built'
baseline_revision: '94209ea5b874a4158c9ba42ebcc5644d536acc70'
route: 'full'
route_source: 'auto'
risk: 'high'
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

**Problem:** Hàng ngang canh giữa (91 chỗ `align-items: center`), nên chữ `ui`, số `mono`, chip phím và biểu tượng 14px đứng lệch nhau vài phần px; nút chỉ có biểu tượng không có baseline nên không đứng được trong hàng baseline (DESIGN.md §Lưới 7px — Canh baseline, mockup `luoi-7px.html` mục 2).

**Approach:** `ui-icon-btn` và mục `ui-seg--icons` mang hộp dòng rỗng; điều khiển, ba thanh cố định (tiêu đề, panel, trạng thái) và các hàng đứng riêng chuyển sang canh baseline với nhóm nằm giữa thanh/nút; phần tử không chữ, không biểu tượng canh giữa hàng.

## Boundaries & Constraints

**Always:** Biểu tượng 14px đặt đáy trên baseline của chữ cạnh nó. Hàng baseline đứng tự do, một dòng, khai chiều cao bội 7 (21 hàng chữ, 28 hàng điều khiển); hàng có thể xuống dòng không khai. Hộp dòng rỗng lấy cỡ và giãn dòng qua `--font-ui-md`/`--leading-ui-md` (không có `--lh-ui`; Kiểm I đỏ với biến chưa định nghĩa). Sửa chiều cao điều khiển chỉ ở `controls.css` (`controls-adoption.test.ts`). Nét ngăn, thanh tiến độ, ô vuông checkbox, ô màu chú giải: `align-self: center`.

**Decision (Ice 2026-10-29, cách A — phần tử bọc):** khung cao cố định (nút, mục menu, chip, ba thanh, hàng cao cố định) giữ `align-items: center`; khi con lẫn font hoặc có biểu tượng, chúng nằm trong một span lớp `ui-row` (`controls.css`: `display: inline-flex; align-items: baseline`). Không `padding-block` lệch lưới để canh giữa. Hàng đứng tự do (không cao cố định) chuyển thẳng sang `align-items: baseline`, không bọc.

**Never:** Chạm thân lưới Workspace (`GridBlock.vue`, phần ô của `SourceHanViet.vue`, hàng lưới) — kể cả số thứ tự câu. Đo hay sửa `trafficLightPosition` (entry 3). Đổi khoảng 7/14 trong/giữa nhóm (entry 6–8). Đổi cỡ chữ, màu, chiều cao điều khiển hay thanh. Thêm ngoại lệ cổng để khỏi sửa giá trị.

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected Output / Behavior | Error Handling |
|----------|--------------|---------------------------|----------------|
| Thanh tiêu đề | tab chế độ, tên Tác phẩm, trạng thái, nút Nhập + chip phím, nút Công cụ + chevron | baseline mọi mục chữ/số lệch ≤ 0,5px; đáy chevron trên baseline ±0,5px | — |
| Nút biểu tượng cạnh nút chữ | mục `ui-seg--icons` 28px cạnh `ui-btn` 28px | cùng đỉnh ±0,5px | — |
| Bẫy | gỡ hộp dòng rỗng | nút biểu tượng lệch đỉnh ~7px ⇒ spec đỏ | — |
| Thanh panel | tab có chip phím | baseline chữ và chip lệch ≤ 0,5px | — |
| Thanh trạng thái 35px | mục chữ + nút 28px | baseline lệch ≤ 0,5px, nhóm nằm giữa thanh ±0,5px | — |

</frozen-after-approval>

## Code Map

- `src/tokens/controls.css` -- `.ui-icon-btn` `:159` là `inline-grid; place-items:center` (một `::before` sẽ thành ô lưới thứ hai ⇒ đổi sang `inline-flex` canh giữa rồi thêm `::before{content:"\200b";width:0;overflow:hidden}` với font/giãn dòng `ui-md`); `.ui-seg--icons .ui-seg__item` `:628` cùng bẫy. `.ui-btn` `:3`, `.ui-field` `:205`, `.ui-menu-item` `:479`, `.ui-chip` `:520` đang `center`; `.ui-check` `:302` (ô vuông 14px canh giữa), `.ui-switch` track `:404` đã `align-self:center`. `controls-css.test.ts:72` cấm `.ui-seg__item` khai `height`.
- `src/AppIcon.vue:51` -- `<svg class="app-icon">` 14×14, `--wide` 21×14, không prop cỡ; 10 chỗ gọi. Svg không có baseline nên flex tổng hợp từ đáy hộp: trong hàng baseline đáy tự rơi đúng baseline.
- `src/App.vue:430` `.titlebar` + `:472` `.titlebar-right` -- con: `.mode-tab` (gạch chân 2px), `.titlebar-work` (cùng gạch trong suốt), `.titlebar-status`, `.layout-switch.ui-seg--icons` (`:249`), `.titlebar-import` + `.titlebar-import-chord` (`:277`), `TitlebarToolsMenu` (`:85` trigger + chevron, `:110–183` mục menu có chip phím và công tắc).
- `src/panels/PanelTab.vue:102` `span.tab` -- chữ + `.tab-chord.ui-kbd`; thanh `.dv-tabs-and-actions-container` của dockview (`dockview-theme.css:111`) không tự canh.
- `src/StatusBar.vue:733` `.status`, `:748` `.status-info,.status-end` -- chữ `ui-sm`, nút `ui-btn` quanh `:670–710`, ngăn bằng `border-left`; không biểu tượng, không mono.
- 91 `align-items: center` trong `src/**/*.{vue,css}` (68 `baseline`): giữ `center` 3 chỗ (`LibraryWorkPage.vue:1068` `.wp-cover`, `ReadingMode.vue:1253` `.toc-overlay`, `:1297` `.marks-overlay`); 88 chỗ còn lại chuyển, trong đó `TmManageOverlay.vue:1094` (ô màu), `LibraryWorkPage.vue:1401` (`.chapter-mini`) cho phần tử không chữ `align-self:center`; `LibraryMode.vue:1330` `⌕` là chữ nên theo baseline. Hàng cao cố định khác: `LookupRecord.vue:237` `.lookup-source-head` 35px, `LibraryWorkPage.vue:1324` `.wp-item` 42px.
- `e2e/specs/story-13-1-luoi-7px.e2e.mjs` -- khuôn: `openWorkspaceWithWork`, `waitForGridRows`, đo trong `browser.execute`. Đo baseline bằng đầu dò `inline-block` 0×0 `vertical-align:baseline` chèn trước nút chữ (như `textBaseline()` ở mockup `luoi-7px.html:480`).
- `e2e/specs/titlebar-tools.e2e.mjs:42–90`, `tests/frontend/titlebarDragRegion.test.ts:60` -- đọc khối `.titlebar`; phải còn xanh.

## Tasks & Acceptance

**Execution:**
- [x] `e2e/specs/story-13-4-canh-baseline.e2e.mjs` -- các ca của Matrix trên WKWebView (thanh tiêu đề, thanh panel, thanh trạng thái) -- R4.
- [x] `src/tokens/controls.css` -- hộp dòng rỗng cho `ui-icon-btn` và `ui-seg--icons`; lớp `ui-row`; ô checkbox `align-self:center` -- R4.
- [x] `src/App.vue`, `src/TitlebarToolsMenu.vue`, `src/panels/PanelTab.vue`, `src/StatusBar.vue` -- bọc `ui-row` cho con của ba thanh cố định và cho 19 điều khiển lẫn nội dung (`App.vue:277`, `GlossaryHarvestOverlay.vue:189,197`, `ReadingMode.vue:591,645,657,662`, `TitlebarToolsMenu.vue:85,110–183`, `LookupSourcesMenu.vue:94`, `LookupRecord.vue:111,155`) -- R4.
- [x] 88 hàng ở Code Map -- hàng đứng tự do: `baseline`, chiều cao bội 7 khi một dòng; hàng cao cố định (`.lookup-source-head`, `.wp-item`): giữ `center` + `ui-row`; phần tử không chữ `align-self:center` -- R4.

**Acceptance Criteria:**
- Given app chạy ngoài sandbox, when spec `story-13-4` chạy, then xanh; given hộp dòng rỗng của `ui-seg--icons` bị gỡ thật, then đỏ đúng ở ca cùng đỉnh.
- Given cây sau story, when `npm run check:tokens`, then xanh, số miễn trừ `aura-allow-spacing` không tăng.
- Given `src/**/*.{vue,css}`, when grep `align-items: center`, then mỗi chỗ còn lại là một trong 3 chỗ giữ ở Code Map hoặc một khung cao cố định có con `ui-row` hay chỉ một con.
- Given spec `titlebar-tools` và `story-13-1`, when chạy lại, then xanh.

## Implementation Notes

- Converted 77 free-standing rows to `baseline`; the plan's 88 does not match a recount of `align-items: center` outside `controls.css` (85 sites, 8 kept as fixed frames or plan-kept rows).
- `.ui-check::before` is a flex item, so `margin-right: calc(-1 * 1 unit)` cancels the container gap (0 for `--bare`); a variant that overrides the gap would shift the label.
- Icon-bottom-on-baseline is not asserted for icon-only buttons: they centre the icon in the 28px box (3.5px above baseline); only same top and the empty line box are asserted. The chevron inside a `ui-row` is asserted.
- `.chapter-row` is a `display: grid`, so it takes `align-items: baseline; align-content: center` and no `ui-row` wrapper (a wrapper becomes one grid item); group rows keep `.wp-line.ui-row`.
- Heights (21/28) added only to `.us-actions`, `.gm-foot`, `.orphan-actions`, `.sl-save` (28) and `.wp-crumb` (21). Radio/checkbox label rows are excluded because `controls-adoption.test.ts` rejects `height` on control classes; wrapping rows and unverified single-line rows were left without height.
- `story-12-34` fails in its IPC seeding (segment counts) before any DOM step, (not re-run on a clean tree, so unattributed).

## Plan Change Log

## Review Triage Log

Lượt 1 (quick): 11 phát hiện — medium 2 · low 3 · false 5 · reject 2 · patch 3 · defer 1.

- medium · patch — `SettingsLayoutSection.vue` `.sl-form, .sl-save` khai `height: 28px` mà chứa `ui-btn--lg`/`ui-field--lg` 35px, tràn 7px; `LibraryWorkPage.vue` `.wp-crumb` 21px chứa `ui-btn--quiet` 28px.
- low · patch — `ChapterSplitOverlay.vue:219` `.cs-label` là `ui-check` có `gap` 14px; `margin-right` −7px của `::before` chỉ bù một nửa nên ô tick lùi 7px. Các nhãn `ui-check` khác đều gap 7.
- low · patch — `StatusBar.vue`: khối template trong `.status-line` chưa thụt lề, hai khai báo `.status-info` đổi chỗ vô cớ.
- medium · defer — `story-12-34` đỏ ở bước gieo IPC, chưa gán nguyên nhân và chưa có chủ; diff không chạm Rust hay đường IPC, `src-tauri` không đổi từ `5af8d0e4`.
- false — kéo cửa sổ hỏng vì `.titlebar-line` phủ thanh: `App.vue:108` đặt `data-tauri-drag-region="deep"` trên macOS, và `drag.js` của tauri 2.11.5 cho cả cây con kéo được (trừ phần tử tương tác).
- false — `gap: inherit` của `.wp-line` mất khoảng: `.wp-item` khai `gap: calc(var(--space-unit) * 2)` và giữ `align-items: center`.
- false — AC grep `center`: 15 chỗ còn lại là 7 điều khiển cao cố định ở `controls.css`, 5 khung cao cố định có con `ui-row` hoặc một con (`.titlebar`, `.status`, `.tab`, `.lookup-source-head`, `.wp-item`) và 3 chỗ giữ.
- false — đầu dò thành flex item nên đo sai: hộp `inline-block` 0×0 có baseline tổng hợp ở đáy, nên trong hàng baseline nó rơi đúng baseline của hàng; `.mode-tab` là `<button>` chữ nội dòng. Phép so giữa thanh trạng thái đỏ nếu `.status` bỏ `center`.
- false — track công tắc mất `align-self: center`: `controls.css:432` vẫn còn.
- low · reject — spec không đo `::before` của `ui-icon-btn` hay mục menu có chip: Matrix chỉ đòi mục `ui-seg--icons`; thêm ca là mã mới cho chỗ ít gặp.
- low · reject — ô Tasks chưa tick khi đã `in-review`: sửa chính plan của lượt build, không phải lỗi của diff.

## Verification

**Commands:**
- `npm run check:tokens` -- expected: exit 0.
- `npx vitest run tests/frontend/controls-css.test.ts tests/frontend/controls-adoption.test.ts tests/frontend/titlebarDragRegion.test.ts` -- expected: 0 failed, 0 Errors.
- `npm run test:e2e -- --spec e2e/specs/story-13-4-canh-baseline.e2e.mjs` rồi `titlebar-tools` và `story-13-1-luoi-7px`, ngoài sandbox -- expected: pass.

**Manual checks (if no CLI):**
- Không có; lượt nhìn app thật của Ice thuộc mục nợ `Chủ: Epic 13`.
