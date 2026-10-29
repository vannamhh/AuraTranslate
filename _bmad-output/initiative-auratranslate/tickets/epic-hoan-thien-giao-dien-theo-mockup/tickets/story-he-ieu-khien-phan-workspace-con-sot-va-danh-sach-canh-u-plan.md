---
title: 'Hệ điều khiển: phần Workspace còn sót và danh sách canh đủ'
type: 'refactor'
ticket: '33'
created: '2026-10-29'
status: 'built'
baseline_revision: '8473b0bb2fc22825666f9d4b632a00e79efa03cd'
route: 'full'
route_source: 'auto'
risk: 'medium'
review: 'quick'
review_source: 'pinned'
lenses_ran: ['quick']
review_loop_iteration: 0
context:
  - '{project-root}/_bmad-output/initiative-auratranslate/tickets/epic-hoan-thien-giao-dien-theo-mockup/tickets/story-he-ieu-khien-library-va-che-o-oc-plan.md'
  - '{project-root}/src/AGENTS.md'
---

<frozen-after-approval reason="human-owned intent — do not modify unless human renegotiates">

## Intent

**Problem:** Đo ở mốc `8473b0bb` bằng bản sao ca canh chạy trên mọi `.vue` dưới `src/` (67 tệp): 10 tệp còn 41 control không mang `ui-*` (App 4, GlossaryConfirmStrip 5, ProofreaderStrip 11, StatusBar 1, TmFuzzyStrip 5, LookupDrawer 1, ReviewDock 1, WorkspaceMode 4, LookupPanel 5, ReviewMinePanel 4) kèm 62 dòng CSS scoped khai lại viền, trạng thái hay focus; `ReadingLookupPopup` (ngoài danh sách 12 của ticket) có thêm 2 control (hai liên kết trong câu) và 10 dòng CSS. `AiTranslationPanel` và `PanelTab` đã sạch, không cần sửa. Ca canh chỉ đọc 43 tệp đã liệt kê, nên một `<button>` thêm vào tệp ngoài danh sách không làm gì đỏ.

**Approach:** Chuyển các tệp còn sót sang lớp `ui-*` của story 27 (không sửa `controls.css`), xoá CSS dáng scoped, rồi đổi `controls-adoption.test.ts` từ danh sách tay sang đọc mọi `.vue` dưới `src/`, mỗi ngoại lệ có tên và lý do.

## Boundaries & Constraints

**Always:**
- Ràng buộc "Always" của story 27–32: token AD-34, mỗi trạng thái một thuộc tính, scoped chỉ còn bố cục; `aria-*`, phím, `data-*`, thứ tự DOM, `@click` một `dispatch` giữ nguyên. Lớp cũ mà test/e2e đọc ở lại trong template làm móc, không mang CSS dáng.
- Bậc theo DESIGN §Điều khiển: dải gợi ý, thanh trạng thái, thanh tiêu đề `sm`; thân panel `md`; nút Đóng đầu ngăn kéo `md` Chìm.
- Mỗi ngoại lệ mới của ca canh có tên và lý do; ngoại lệ hết khớp thì ca đỏ.
- Một mục nợ mới `Chủ: Epic 12` cho lượt nhìn app thật.

**Never:**
- Không sửa `controls.css`, `DropdownMenu.vue`, `SelectMenu.vue`, DESIGN.md, `check-tokens.mjs`; không thêm dependency, token hay lớp `ui-*`.
- Không biến ca canh thành cổng ở `scripts/` hay `pre-push` (câu hỏi mở của epic dành cho Ice và Winston sau ticket này).
- Không né ca canh bằng `box-shadow`, selector thuộc tính hay mẹo khác.

**Decisions (Ice, 2026-10-29):**
- Bốn liên kết trong câu `lookup-char`, `lookup-concordance-pointer` (`LookupPanel`), `rl-char`, `rl-concordance` (`ReadingLookupPopup`) là khoá `NOT_A_CONTROL` có tên, lý do "liên kết trong câu, DESIGN không có loại liên kết"; CSS giữ nguyên; `ReadingLookupPopup.vue` không chuyển (nút thứ ba của nó đã `ui-*`).
- Ba tab chế độ `mode-tab` ở `App.vue` là khoá `NOT_A_CONTROL`, CSS giữ nguyên; `titlebarWorkName.test.ts` không sửa.
- `.layout-switch { border-width: 0 }` ở `App.vue` là khoá `SHAPE_EXCEPTIONS` "thanh bố cục trên thanh tiêu đề không viền".

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected Output / Behavior | Error Handling |
|----------|--------------|---------------------------|----------------|
| Tệp mới | `<button>` không lớp thêm vào tệp chưa từng nằm trong danh sách (vd `src/panels/PanelFrame.vue`) | Ca đỏ ghi đúng `tệp:dòng` | Gỡ lại, `cmp` trùng byte |
| Tệp không control | `.vue` không có control (`AppIcon`, `PanelTab`...) | Không ca nào đỏ chỉ vì thiếu control | — |
| Ngoại lệ chết | Khoá `NOT_A_CONTROL` hay `SHAPE_EXCEPTIONS` không còn khớp phần tử nào | Ca "khoá chết" đỏ | Xoá khoá hoặc trả phần tử |
| Thư mục con | `.vue` ở `src/layout`, `src/panels`, `src/modes` | Được đọc như tệp ở gốc | — |

</frozen-after-approval>

## Code Map

Số dòng ở mốc `8473b0bb`. Mẫu chuyển và quy tắc xoá: Code Map của plan 32 (`context`). ⚠️ Selector dấu phẩy: xoá trọn rồi đọc lại khối quanh chỗ xoá.

- `tests/frontend/controls-adoption.test.ts` -- `FILES` tay (43 tệp) thay bằng duyệt đệ quy `src/**/*.vue`; giữ `CONTROL_TAGS`, `SHAPE_EXCEPTIONS`, `NOT_A_CONTROL`, `LABEL_EXEMPT`, `inspect()`. Bỏ `expect(controls.length).toBeGreaterThan(0)` (tệp không control hợp lệ) và ca "reaches every listed file" (đổi thành: số tệp đã đọc bằng số `.vue` tìm được, tổng control > 0, `GlossaryManageOverlay` ≥ 20). `descriptor.template` có thể `null`: bỏ qua.
- `src/ProofreaderStrip.vue` (460) -- nút 196/204/216/228/240/268/271/274/278/281/289 `pfs-btn`/`pfs-link`/`pfs-scope`: `ui-btn ui-btn--sm`; `pfs-link` (204) `ui-btn--quiet`. Xoá `.pfs-btn*` 440-460 và `.pfs-link` 367 (giữ `margin-left: auto`).
- `src/TmFuzzyStrip.vue` (440) -- 149/152/182/226/236 `tmf-btn`: `ui-btn ui-btn--sm`, 149 thêm `ui-btn--primary` (mockup `he-dieu-khien.html:195`). Xoá `.tmf-btn*` 416-440.
- `src/GlossaryConfirmStrip.vue` (506) -- 185/287/288 `gcs-act`: `ui-btn ui-btn--sm`, 287 `--primary`, 185 (`gcs-undo`) `--quiet`; 252 `gcs-input`: `ui-field ui-field--sm`; 227 `gcs-option` giữ nguyên (xem Design Notes), thêm khoá `NOT_A_CONTROL`. Xoá `.gcs-act*` 481-506 và `.gcs-input*` 450-465.
- `src/StatusBar.vue` (836) -- 552 `status-link`: `ui-btn ui-btn--quiet ui-btn--sm`; xoá `.status-link` 802-816 trừ `margin-left`. Bốn nút khác đã `ui-btn--sm`.
- `src/panels/LookupPanel.vue` (1277) -- 355/368/381 `lookup-tab`: `ui-tab` (đã có `role="tab"`, `aria-selected`); xoá `.lookup-tab*` 1020-1040, bố cục (`gap`, đệm hàng) dồn vào `.lookup-tabs`, không khai `height` trên tab. 554 `lookup-char`, 566 `lookup-concordance-pointer`: không chuyển, khoá `NOT_A_CONTROL`, CSS giữ.
- `src/panels/ReviewMinePanel.vue` (173) -- 50/73 `review-act review-act-primary`: `ui-btn ui-btn--primary`; 60/76 `ui-btn`. Xoá `.review-act*` 133-160.
- `src/modes/WorkspaceMode.vue` (220) -- 123/132/138/141 `review-notice-act`: `ui-btn ui-btn--sm`; xoá 200-220.
- `src/layout/LookupDrawer.vue` (211) -- 129 `ld-close`: `ui-btn ui-btn--quiet`; xoá 188-205 (`workspaceDockTier.test.ts:495` bấm `.ld-close`, lớp ở lại).
- `src/layout/ReviewDock.vue` (170) -- 86 `review-align-link`: `ui-btn ui-btn--quiet ui-btn--sm`; xoá `.review-align-link` 144.
- `src/App.vue` (613) -- 277 `titlebar-import`: `ui-btn ui-btn--quiet ui-btn--sm`, xoá 477-490; 211/219/227 `mode-tab` không sửa (khoá `NOT_A_CONTROL`); 466 `.layout-switch` giữ (khoá `SHAPE_EXCEPTIONS`).
- `src/modes/ReadingLookupPopup.vue` (305) -- không sửa; 160 `rl-char`, 183 `rl-concordance` vào ca canh qua khoá `NOT_A_CONTROL`.
- `src/panels/AiTranslationPanel.vue`, `src/panels/PanelTab.vue` -- đã sạch; chỉ cần dưới ca canh, không sửa.
- `tests/frontend/titlebarWorkName.test.ts:111` đọc `border-bottom` trong rule `.mode-tab` của `App.vue` -- không sửa, vì `.mode-tab` giữ CSS.
- Móc phải giữ: `.mode-tab`, `.layout-switch-btn`, `.gcs-act:not(.gcs-act-primary)` (`glossaryConfirmStripTemplate`), `.gcs-option`/`.gcs-input`/`.gcs-undo` (vitest + e2e 12-10, 12-20), `.pfs-actions .pfs-btn`, `.tmf-row .tmf-btn`, `.ld-close`, `#lookup-tab-*`, `.lookup-char`, mọi `data-*`.
- `_bmad-output/initiative-auratranslate/deferred-work.md` -- cuối tệp (grep, không đọc cả tệp), theo mẫu `## Deferred from:` dòng 13534.

## Tasks & Acceptance

**Execution:**
- [x] `tests/frontend/controls-adoption.test.ts` -- duyệt mọi `.vue` dưới `src/`, bỏ ràng buộc "≥ 1 control", đổi ca phủ như Code Map -- chạy trước các tệp khác, kỳ vọng đỏ đúng 12 tệp ở Problem.
- [x] Mười tệp `.vue` ở Code Map -- đổi lớp, xoá CSS dáng; 36 control chuyển, 5 (3 `mode-tab`, 2 liên kết của `LookupPanel`) khai ngoại lệ.
- [x] `tests/frontend/controls-adoption.test.ts` -- thêm `NOT_A_CONTROL`: `src/GlossaryConfirmStrip.vue .gcs-option` (hàng gợi ý nhiều phần, vạch ngắm bằng viền, tiền lệ `pfmem-row`/`pl-row`), `src/panels/LookupPanel.vue .lookup-char` và `.lookup-concordance-pointer`, `src/modes/ReadingLookupPopup.vue .rl-char` và `.rl-concordance` (lý do: liên kết trong câu, DESIGN không có loại liên kết), `src/App.vue .mode-tab` (lý do: tab chế độ, DESIGN không có loại tab chế độ và mockup `workspace-tinh-chinh.html` vẽ khác `ui-tab`); `SHAPE_EXCEPTIONS`: `src/App.vue .layout-switch` (thanh bố cục trên thanh tiêu đề không viền).
- [x] `_bmad-output/initiative-auratranslate/deferred-work.md` -- một mục ≤ 5 dòng `Chủ: Epic 12`: ảnh app thật Workspace (ba dải gợi ý, tab Tra cứu, ngăn kéo, thanh trạng thái, thanh tiêu đề, sáng/tối) cạnh `he-dieu-khien.html` và `workspace-tinh-chinh.html`; dải gợi ý đổi từ nền `background` sang `surface-sunken`; phải nêu tên cả hai chỗ chưa chuyển: bốn liên kết trong câu còn gạch chân `outline` 1,31:1 (`lookup-char`, `lookup-concordance-pointer`, `rl-char`, `rl-concordance`) và ba `mode-tab` chưa dùng `ui-*`; kèm thanh bố cục không viền.

**Acceptance Criteria:**
- Given `src/` ở mốc plan, when chạy ca canh mới, then nó đọc đủ 67 `.vue` (kể cả `src/layout`, `src/panels`, `src/modes`) và xanh sau khi chuyển; mọi ngoại lệ còn lại nằm trong ba bảng của test kèm lý do.
- Given `src/panels/PanelFrame.vue` (chưa từng trong danh sách) và một `<button type="button">x</button>` không lớp thêm vào template, when chạy `controls-adoption`, then đỏ đúng `src/panels/PanelFrame.vue:<dòng>`; gỡ lại thì xanh và `cmp` trùng byte (đối chứng thật: thêm vào mã sản xuất, chỉ chạy tệp test này).
- Given mười tệp đã chuyển (trừ ba `mode-tab` và bốn liên kết trong câu, đã khai ngoại lệ), when xem Workspace sáng và tối, then dải gợi ý, thanh trạng thái, ngăn kéo Tra cứu và panel Review không còn nút viền `outline`, chữ `primary` hay nút trơn; control cùng hàng cùng chiều cao (phần nhìn là mục nợ cho Ice).
- Given test hiện có của panel, dải gợi ý, thanh trạng thái và thanh tiêu đề, when chạy, then xanh; `check:tokens`, `check:i18n`, `check:commands` xanh.

## Implementation Notes

- 36 control chuyển `ui-*`, 5 khai ngoại lệ có tên (3 `mode-tab`, 2 liên kết của `LookupPanel`); `ReadingLookupPopup` vào ca canh chỉ qua hai khoá liên kết.
- `WorkspaceMode` và `ReviewDock` giữ rule chỉ còn `margin-left` thay vì xoá cả khối như Code Map, vì xoá thì mất khoảng cách.
- Ca canh thêm ca khoá chết cho `SHAPE_EXCEPTIONS` (trước chỉ `NOT_A_CONTROL` có); đối chứng: gỡ rule `.layout-switch` ở `App.vue` ⇒ đỏ đúng khoá đó.
- Ca "đọc mọi .vue" so `FILES` với `readdirSync(recursive)` độc lập, chuẩn hoá `\` thành `/` cho Windows; đối chứng: cắt nhánh đệ quy của `vueFiles` ⇒ đỏ.
- e2e `story-12-10`, `story-12-20`, `titlebar-tools` chưa chạy: cổng 1420 bị một tiến trình `node` khác giữ lúc build (2026-10-29).

## Plan Change Log

## Review Triage Log

- false · reject — mục `HANDLER_TABLE` của `onSegmentTextMouseDown` trong diff: là commit riêng `69ce0401` vá `02305186`, nằm trong diff chỉ vì diff tính từ mốc `8473b0bb`; không thuộc story này.
- low · patch — ca "đọc mọi .vue" so `FILES` với chính nó, lỗi đệ quy bỏ sót thư mục vẫn xanh; đổi sang so với một liệt kê `src/**/*.vue` độc lập.
- false · reject — sáu khoá `*-sr-only` có thể chết làm ca khoá chết `SHAPE_EXCEPTIONS` đỏ: ca chạy xanh 137/137, cả bảy khoá còn che rule.
- reject — checkbox và Implementation Notes trống: sửa là sửa plan, điền ở bước trình bày.

## Design Notes

- Bậc: bảng ba bậc của DESIGN ghi dải gợi ý là `sm`; mockup `he-dieu-khien.html:195` vẽ nút Nhận của dải `ui-btn--sm ui-btn--primary`, nên `tmf-btn-primary`, `gcs-act-primary`, `review-act-primary` thành `ui-btn--primary` (quyết định 2 của plan 32). Ô `gcs-input` là `ui-field--sm`, không `--on-sunken` vì dải đứng trên nền `surface`/`surface-tm`.
- Nút dạng liên kết (`pfs-link`, `status-link`, `review-align-link`, `titlebar-import`, `gcs-undo`) → `ui-btn--quiet`, tiền lệ quyết định 3 của story 30 và 8 của story 31.
- `gcs-option` là hàng chọn có phím, nhãn nguồn và vạch ngắm; `ui-btn` nowrap sẽ cắt chữ. Giữ hàng, CSS dáng của nó là ngoại lệ có tên (không phải quyết định mới).
- `ui-tab` không khai chiều cao và đệm 0: hàng tab tự quyết bố cục qua `.lookup-tabs`; thêm `height` vào tab sẽ làm ca canh đỏ.
- Chia pha gợi ý cho build: (1) ca canh + đỏ có chủ đích, (2) ba dải gợi ý + `ReviewMinePanel` + `WorkspaceMode`, (3) `LookupPanel`, `LookupDrawer`, `ReviewDock`, `StatusBar`, `App`, (4) test sửa + mục nợ.

## Verification

**Commands:**
- `npx vitest run tests/frontend/controls-adoption.test.ts tests/frontend/controls-css.test.ts $(grep -lE "StatusBar|ProofreaderStrip|TmFuzzyStrip|GlossaryConfirmStrip|LookupPanel|ReviewMine|WorkspaceMode|ReviewDock|LookupDrawer|titlebar|workspaceDockTier" tests/frontend/*.ts)` -- expected: mã thoát 0, không dòng Errors.
- `npm run check:tokens && npm run check:i18n && npm run check:commands` -- expected: xanh.
- `npx vue-tsc --noEmit` -- expected: không lỗi.
- e2e `story-12-10-glossary-tm-suggestions`, `story-12-20-glossary-unconfirm`, `titlebar-tools` (ngoài sandbox, cổng 1420 trống) -- expected: xanh; bận thì ghi lại, không suy ra.

**Manual checks:**
- Ice: ảnh app thật Workspace sáng và tối (mục nợ `Chủ: Epic 12`).
