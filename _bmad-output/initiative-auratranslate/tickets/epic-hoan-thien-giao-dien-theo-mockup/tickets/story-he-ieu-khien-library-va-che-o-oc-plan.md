---
title: 'Hệ điều khiển: Library và Chế độ đọc'
type: 'refactor'
ticket: '32'
created: '2026-10-28'
status: 'built'
baseline_revision: 'a9b3d7d0b32f9683462eb468b58df356f5860aad'
route: 'full'
route_source: 'auto'
risk: 'medium'
review: 'quick'
review_source: 'pinned'
lenses_ran: ['quick']
review_loop_iteration: 0
followup_review_recommended: false
context:
  - '{project-root}/_bmad-output/initiative-auratranslate/tickets/epic-hoan-thien-giao-dien-theo-mockup/tickets/story-he-ieu-khien-cai-at-plan.md'
  - '{project-root}/src/AGENTS.md'
warnings: ['oversized']
deferred:
  - summary: >-
      Nút `ui-btn` có `aria-pressed` ở thanh Chế độ đọc (Song ngữ, chỉnh chữ, theme) không có dáng đang bấm.
    evidence: |-
      `controls.css` chỉ tô `aria-pressed` cho `ui-seg__item`, `ui-icon-btn`, `ui-chip`; trước đổi các nút này là nút trơn không CSS nên cũng không có dáng bấm. Sửa cần lớp mới trong `controls.css`, ngoài phạm vi; đã ghi trong mục nợ `Chủ: Epic 12`.
    location: >-
      src/modes/ReadingMode.vue:487
    severity: low
---

<intent-contract>

## Intent

**Problem:** `LibraryMode`, `LibraryWorkPage`, `LibrarySearchResults`, `LibraryUrlImportScreen` và `ReadingMode` còn 104 control không mang lớp `ui-*` (đo bằng bản sao ca canh chuyển đổi chạy trên năm tệp, mốc `a9b3d7d0`): nút viền `outline` chữ `primary`, nút chính tô `primary`, ô tìm/ô nhập viền `outline`, chip viên thuốc tô `primary`, thanh phân đoạn tự vẽ; nút của `ReadingMode` chạy dáng gốc.

**Approach:** Chuyển năm tệp sang lớp `ui-*` của story 27 theo DESIGN.md §Điều khiển, xoá CSS dáng scoped (chỉ giữ bố cục), nối năm tệp vào `tests/frontend/controls-adoption.test.ts`; không thêm lớp vào `controls.css`; giữ lưới, thẻ, tông bìa của story 24.

## Decisions (build-auto, 2026-10-28)

1. Bậc: thân panel `md` (nút ô trống, gốc, mồ côi, dựng lại chỉ mục, mốc biên, đầu trang Tác phẩm, đổi tên, chân Nhập URL, chân hai bảng Mục lục/Đánh dấu). Hàng điều khiển và hàng dày `sm`: thanh công cụ Chế độ đọc, `wp-bar` (ô tìm, chip lọc Chương, chip sắp xếp), `wp-selbar`, hàng kết quả tìm kiếm, nút trong câu, hàng đánh dấu, nút ẩn hiện trên bìa. Thanh trên Library là `md` vì ô chọn dạng field cố định `md`; `ui-seg` cố định 24px (mục nợ, như story 31).
2. "Nút chính thôi tô `primary`": `btn-primary`/`us-btn-primary`/`work-go` thành `ui-btn--primary` (nền mực). Không thêm Chính hay Nguy hiểm nào khác.
3. "Ô chọn bộ lọc dạng field": `DropdownMenu` Trạng thái và ba `SelectMenu` (lĩnh vực, ngôn ngữ nguồn, sắp xếp) ở thanh trên Library đổi `variant="chip"` → `variant="field"` (mockup `he-dieu-khien.html`, hàng "Tìm · Mọi loại ▾"). `SelectMenu` sắp xếp trong `wp-bar` giữ `variant="chip"` vì hàng đó `sm` và field không có bậc `sm`.
4. Ô tìm thanh trên: `search-input` mang `ui-field ui-field--full`; khung `.search-field` chỉ còn bố cục (`position: relative`, flex, `max-width`, `min-width`), bỏ viền/nền/đệm/`:focus-within`; biểu tượng `⌕` đặt tuyệt đối bên trái trong ô, `pointer-events: none`, ô thêm `padding-left` chừa chỗ.
5. Mục menu Trạng thái và menu `⋯` thành `menu-item ui-menu-item`. Vạch trên "Xoá bộ lọc" thành `<div class="ui-menu-sep" aria-hidden="true">` ngay trước nút đó; bỏ lớp `menu-item--rule`. Mục đang chọn (`aria-pressed="true"`) giữ một rule scoped `background: var(--color-surface-accent)`, bỏ chữ `primary`, vì `ui-menu-item` không có dáng chọn và nhãn trường chỉ còn "N trạng thái" khi chọn từ hai (mục nợ).
6. Thanh phân đoạn: `.segmented` → `segmented ui-seg`, `.segment` → `segment ui-seg__item`. Ba nhãn phạm vi tìm giữ radio, radio mang `lib-sr-only` (ẩn kiểu sr-only, tiền lệ `gm-sr-only`), nhãn thêm `position: relative`; bỏ bind `segment--on` (`:has(input:checked)` tô thay). Mức đọc Thoáng/Cân/Đặc: bọc ba nút trong `<span class="ui-seg">`, mỗi nút `btn ui-seg__item`; `.level-group` đổi `align-items` sang `center`.
7. Chip lọc Chương `wp-pill` → `wp-pill ui-chip` (`aria-pressed` tô sẵn). Ô tìm Chương `wp-search ui-field ui-field--sm ui-field--on-sunken` (nền `wp-bar` là `surface-sunken`), giữ `min-width: 200px`.
8. Nút chỉ có ký hiệu `‹`/`›` (mồ côi, điều hướng kết quả tìm) → `btn ui-icon-btn` (`--sm` ở kết quả tìm). Nút dạng liên kết (`wp-group-toggle`, `mark-affordance`) → `ui-btn ui-btn--quiet ui-btn--sm` (quyết định 3 của story 30); xoá rule `.mark-affordance`, `.wp-group-toggle` rút về `margin-left: auto`.
9. Nút "Tiếp tục dịch" trên bìa: bọc trong `<div class="work-go-slot">`; luật ẩn/hiện theo rê chuột/tiêu điểm/con trỏ của thẻ chuyển từ `.work-go` sang `.work-go-slot` (ẩn hiện là trạng thái của thẻ, không phải dáng control); bỏ rule `.work-cell:not(.work-cell--current) .work-go`.
10. Ô dán URL: `us-textarea ui-field ui-field--full`; khung `.us-paste` chỉ còn bố cục (flex, `height: 260px`, khoảng cách), bỏ viền/nền và `:has(:focus-visible)`; cột số dòng `.us-gutter` đứng ngoài ô, theo font/giãn dòng `ui-md-wrap` của `textarea.ui-field` và đệm trên `7px` (viền 1px + đệm 6px của ô) để số khớp dòng; `.us-textarea` chỉ còn `flex: 1; min-width: 0; resize: none; overflow: auto`. Hai radio đích `us-radio ui-check` (giữ `align-items: flex-start`). Ô nhập `ui-field ui-field--full` (`md`, cùng cột với `SelectMenu` field).
11. Thanh trượt Chế độ đọc: hai `input[type=range]` thêm lớp `tuner-range`, khoá `NOT_A_CONTROL` `src/modes/ReadingMode.vue .tuner-range` (tiền lệ `sr-range`).

## Boundaries & Constraints

**Always:**
- Ràng buộc "Always" của story 27–31: token AD-34, mỗi trạng thái một thuộc tính, scoped chỉ còn bố cục, `aria-*`/phím/`data-*`/`@click` một `dispatch`/thứ tự DOM của nút và ô giữ nguyên (ngoại lệ: `span.ui-seg` mức đọc, `div.work-go-slot`, `div.ui-menu-sep`).
- Lớp cũ mà test/e2e đọc ở lại trong template, không mang CSS dáng (danh sách ở Code Map).
- Mỗi miễn trừ mới trong ca canh có tên và lý do; miễn trừ không còn khớp thì ca đỏ.
- Mục nợ mới `Chủ: Epic 12` cho lượt nhìn app thật.

**Never:**
- Không sửa `controls.css`, `DropdownMenu.vue`, `SelectMenu.vue`, tệp của story 27–31, DESIGN.md, `check-tokens.mjs`; không thêm dependency, token hay lớp `ui-*` mới.
- Không đổi lưới `auto-fill`, kích thước thẻ, tông bìa, chữ tên trên bìa của story 24.
- Không né ca canh bằng `box-shadow`, selector thuộc tính hay mẹo khác.

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected Output / Behavior | Error Handling |
|----------|--------------|---------------------------|----------------|
| Thẻ Tác phẩm | Rê chuột hoặc con trỏ ở thẻ | "Tiếp tục dịch" hiện ở bìa, nền mực, dưới tên ba dòng, trong bìa 136px | — |
| Lọc trạng thái | Chọn Chưa bắt đầu + Xong | Trường hiện "2 trạng thái"; hai mục nền `surface-accent` | — |
| Phạm vi tìm | Gõ chữ, chọn "Bản dịch" | Ô "Bản dịch" nền mực; radio ẩn vẫn `checked`, phím mũi tên đổi phạm vi | — |
| Miễn trừ chết | Bỏ lớp `tuner-range` khỏi cả hai thanh trượt | Ca "every NOT_A_CONTROL key still names a control" đỏ | — |

</intent-contract>

## Code Map

Số dòng ở mốc `a9b3d7d0`. Mẫu chuyển và quy tắc xoá: Code Map của plan ticket 31 (context) và 30. ⚠️ Danh sách selector dấu phẩy: xoá trọn, đọc lại khối quanh mỗi chỗ xoá. Mọi lớp cũ (`btn`, `btn-primary`, `segment`, `menu-item`, `wp-pill`, `us-btn`…) ở lại trong template làm móc.

- `src/modes/LibraryMode.vue` (template 381-949, style 951-1519) -- 393 `search-input ui-field ui-field--full` (quyết định 4); 405/425 `segmented ui-seg`; 406/415 `segment ui-seg__item`; 431/441/451 `segment ui-seg__item`, bỏ bind, radio 432/442/452 thêm `class="lib-sr-only"`; 464 `variant="field"`; 470-500 `menu-item ui-menu-item`; 510 `menu-item ui-menu-item` + `div.ui-menu-sep` trước nó; 522/539/556 `variant="field"`; 600 `btn btn-primary ui-btn ui-btn--primary`; 603 `btn ui-btn`; 632 bọc `div.work-go-slot`, nút `work-go ui-btn ui-btn--sm ui-btn--primary`; 645/654 `menu-item ui-menu-item`; 744/752/897/905/919 `btn ui-btn`; 877/888 `btn ui-icon-btn`. CSS xoá: 1015-1042 (`.field*` chết, không phần tử nào mang `field`), 1044-1065 (`.btn` rút về `align-self: flex-start`; `:focus-visible`, `:disabled`), 1253-1287 (`.work-go, .work-more { display: none }` và danh sách hiện chuyển `.work-go` → `.work-go-slot`; xoá `.work-go`, `:not(--current) .work-go`, `.work-go:disabled`), 1378-1404 (`.menu-item*`, giữ một rule `[aria-pressed='true']` quyết định 5), 1431-1468 (`.search-field` còn bố cục, `.search-input` còn `flex: 1; min-width: 0; padding-left`, xoá `:focus-within`, `:focus-visible`), 1470-1505 (`.segmented`, `.segment*`; thêm `.segment { position: relative }`), 1507-1511 (`.btn-primary`); thêm `.lib-sr-only` (sao `gm-sr-only`, kèm `aura-allow-spacing` như bản gốc). Giữ `.work-cell*`, `.work-cover*`, `.work-name`, `.chips`, `.empty-actions`.
- `src/modes/LibraryWorkPage.vue` (template 223-731, style 733-1292) -- 226/317/326 `btn ui-btn`; 342 `wp-search ui-field ui-field--sm ui-field--on-sunken`; 351-391 `wp-pill ui-chip`; 421/440-497 `btn ui-btn ui-btn--sm`; 430 `btn btn-primary ui-btn ui-btn--sm ui-btn--primary`; 511-538 `btn ui-btn`; 552 input `ui-field ui-field--medium`; 562 `btn btn-primary ui-btn ui-btn--primary`; 571 `btn ui-btn`; 631 `wp-group-toggle ui-btn ui-btn--quiet ui-btn--sm`; 712 input `ui-field ui-field--short`. CSS xoá: 746-780 (`.btn` rút về `align-self: flex-start`; `:focus-visible`, `:disabled`, `.btn-primary`, `.btn-primary:disabled`), 809-822 (`.field input`, `:focus-visible`; giữ `.field`), 989-1027 (`.wp-search` rút về `min-width: 200px`; xoá `:focus-visible`, mọi `.wp-pill*`), 1081-1094 (`.wp-rename-field input*`), 1153-1168 (`.wp-group-toggle` rút về `margin-left: auto`; xoá `:focus-visible`).
- `src/modes/LibrarySearchResults.vue` (template 166-414, style 416-661) -- 198/312/322/336/348/359/395/402 `btn ui-btn ui-btn--sm`; 227/242 `btn ui-icon-btn ui-icon-btn--sm`. Xoá 500-515 (`.btn`, `.btn:focus-visible`). Giữ `.search-chip*` (span, không phải control).
- `src/modes/LibraryUrlImportScreen.vue` (template 85-249, style 251-473) -- 107 `us-textarea ui-field ui-field--full`; 135/147 `us-radio ui-check`; 182/183/194/214 `ui-field ui-field--full`; 231 `us-btn ui-btn`; 234 `us-btn us-btn-primary ui-btn ui-btn--primary`. CSS theo quyết định 10: 309-322 (`.us-paste` bỏ `border`, `border-radius`, `background`; xoá `:has()` và `.us-textarea:focus-visible`), 324-338 (`.us-gutter` font/giãn dòng `ui-md-wrap`, `padding: 7px 8px` kèm `aura-allow-spacing`), 340-354 (`.us-textarea` rút gọn), 420-433 (`.us-field input, .us-field select*`), 442-473 (`.us-btn*`). Giữ `.us-actions`, `.us-radio` bố cục.
- `src/modes/ReadingMode.vue` (template 484-925, style 927-1292) -- 487/530/539/551/556 `btn ui-btn ui-btn--sm`; 500/509/518 bọc `<span class="ui-seg">`, mỗi nút `btn ui-seg__item`; 578/590 thêm `class="tuner-range"`; 722/729 `mark-affordance ui-btn ui-btn--quiet ui-btn--sm`; 771/781/791/845/849/859/909/912/915/918 `btn ui-btn`; 826/834 `btn ui-btn ui-btn--sm`. CSS: xoá `.mark-affordance` 1083-1088; `.level-group` 1252 `align-items: center`. Không có CSS `.btn`.
- `tests/frontend/controls-adoption.test.ts` -- `FILES` thêm năm tệp; `SHAPE_EXCEPTIONS` thêm `'src/modes/LibraryMode.vue .lib-sr-only'` (lý do như `gm-sr-only`); `NOT_A_CONTROL` thêm `'src/modes/ReadingMode.vue .tuner-range'` (lý do như `sr-range`).

Móc phải giữ: `.work-cover .work-go` (e2e 12-24, đo hình học), `.us-actions .us-btn:not(.us-btn-primary)` (e2e 12-6), `.root-actions .btn:nth-of-type(2)`, `.orphan-actions .btn:nth-of-type(3)` (e2e 5-3, 12-3), `.marks-list li .btn` (e2e 5-13), `.mark-affordance` (vitest `readingMarks`, `readingModeImages`), `.frontier`, mọi `data-*` (`data-library-chip`, `data-lifecycle-filter`, `data-library-chip-option`…).

## Tasks & Acceptance

**Execution:**
- [ ] Năm tệp `.vue` ở Code Map -- đổi lớp, thêm ba phần tử bọc, xoá CSS dáng -- quyết định 1–11.
- [ ] `tests/frontend/controls-adoption.test.ts` -- năm tệp, một `SHAPE_EXCEPTIONS`, một `NOT_A_CONTROL` -- ca canh nối dài. Đối chứng thật: gỡ mọi `ui-*` khỏi nút "Gộp Chương" (`btn btn-primary`, `LibraryWorkPage.vue:430`) ⇒ đỏ đúng `LibraryWorkPage.vue:<dòng>`; bỏ `tuner-range` khỏi cả hai thanh trượt ⇒ đỏ ca khoá chết; cả hai trả lại trùng byte (`cmp`).
- [ ] `tests/frontend/` -- sửa test hiện có chỉ khi móc đổi, ghi lý do; thêm assert cho hai hàng ma trận: mục Trạng thái đang chọn mang `aria-pressed="true"` và `ui-menu-item`; radio phạm vi đang chọn `checked` trong nhãn `ui-seg__item`.
- [ ] `_bmad-output/initiative-auratranslate/deferred-work.md` -- cuối tệp, `## Deferred from: hệ điều khiển Library và Chế độ đọc (2026-10-28, ticket 32 của Epic 12)`, một mục ≤ 5 dòng `Chủ: Epic 12`: ảnh app thật Library (lưới, trang Tác phẩm, kết quả tìm, Nhập URL) và Chế độ đọc sáng/tối cạnh `he-dieu-khien.html`, `library-luoi-rong.html`; `ui-seg` 24px giữa thanh trên `md`; mục Trạng thái chọn tô `surface-accent`; cột số dòng ngoài ô dán URL; nút `aria-pressed` của thanh Chế độ đọc không có dáng bấm.

**Acceptance Criteria:**
- Given Library, trang Tác phẩm, kết quả tìm, màn Nhập URL và Chế độ đọc mở, when nhìn mọi control ở hai theme, then không còn nút tô `primary`, viền `outline`, chip bo tròn viên thuốc hay nút dáng gốc; control cùng hàng cùng chiều cao, trừ `ui-seg` 24px ghi nợ.
- Given ca canh chuyển đổi, when gỡ lớp `ui-*` khỏi một control của năm tệp, then đỏ đúng tệp:dòng.
- Given test vitest hiện có của Library và Chế độ đọc, when chạy, then xanh; `check:tokens`, `check:i18n` xanh.
- Given e2e `story-12-24-library-wide-grid` (ngoài sandbox, cổng 1420 trống), when chạy, then xanh (tên ba dòng không đè nút bìa).

## Implementation Notes

- Năm tệp chuyển theo quyết định 1–11; ca canh nối dài cả năm tệp, một `SHAPE_EXCEPTIONS` (`.lib-sr-only`), một `NOT_A_CONTROL` (`.tuner-range`). Đối chứng thật: gỡ `ui-*` khỏi nút Gộp Chương ⇒ đỏ `LibraryWorkPage.vue:430`; gỡ `tuner-range` ⇒ đỏ ca khoá chết; trả lại nguyên byte.
- Hai ca ma trận ở `libraryTopBar.test.ts`: hai mục Trạng thái chọn ⇒ trường `ui-field` ghi "Trạng thái (2)", rule chọn chỉ khai `surface-accent`; radio phạm vi ẩn bằng `clip`, không `display: none`, cùng `name`. Đối chứng gỡ thật (rule chọn sang `primary`; thêm `display: none` vào `.lib-sr-only`) đỏ đúng ca rồi trả lại trùng byte.
- e2e `story-12-24` lượt đầu đỏ ở `before all` ("sample Library never filled the grid"), chạy lại cùng bản build xanh 6/6 hai lần; chưa tìm nguyên nhân lượt đỏ.

## Plan Change Log

## Review Triage Log

### 2026-10-28 — Review pass
- verdicts: 8 findings — high 0, medium 0, low 5, false 3, maybe-false 0
- findings:
  - `[low]` `[patch]` Lý do `aura-allow-spacing` của `.lib-sr-only` viết tiếng Việt, AGENTS.md đòi comment tiếng Anh — vá: đổi thành `sr-only geometry, not grid spacing`.
  - `[false]` `[reject]` `.btn { align-self: flex-start }` lệch hàng `wp-bar` — rule có sẵn từ trước, không đổi; `wp-bar` không chứa `.btn` nào.
  - `[low]` `[reject]` Ma trận ghi "2 trạng thái", mã hiện "Trạng thái (2)" — sửa duy nhất là chỉnh khối intent của plan, workflow cấm.
  - `[low]` `[reject]` Mục Trạng thái đang chọn không đổi khi rê chuột — hiếm gặp, sửa phải thêm rule; dáng chọn đã trong mục nợ cho Ice.
  - `[false]` `[reject]` Bọc "Tiếp tục dịch" trong `work-go-slot` có thể vỡ bố cục bìa — e2e `story-12-24` đo hình học đó, xanh 6/6 hai lượt; lượt đỏ đầu dừng ở `before all`, trước mọi phép đo.
  - `[false]` `[reject]` Cột số dòng có thể lệch dòng ô dán URL — cột và ô cùng cỡ chữ, giãn dòng `ui-md-wrap`, cùng độ lệch trên 7px; `.ui-field:focus` là `outline: none`.
  - `[low]` `[defer]` Nút `aria-pressed` của thanh Chế độ đọc không có dáng bấm — có từ trước (nút trơn không CSS), sửa cần `controls.css`; ghi vào `deferred`.
  - `[low]` `[reject]` Test mới chỉ đọc lớp và chữ CSS, không bấm thật — bind `aria-pressed` không đổi; e2e `story-5-4` bấm thật bộ lọc Trạng thái dạng field xanh 2/2, `story-5-6` (lĩnh vực, sắp xếp) xanh 1/1, `story-12-5` bấm thật radio phạm vi ẩn xanh 5/6 (ca đỏ là `openChip` thiếu import từ `03e3d8c0`, ghi nợ).

## Verification

**Commands:**
- `npx vitest run tests/frontend/controls-adoption.test.ts tests/frontend/controls-css.test.ts $(grep -lE "LibraryMode|LibraryWorkPage|LibrarySearchResults|LibraryUrlImportScreen|ReadingMode" tests/frontend/*.ts)` -- expected: mã thoát 0, không dòng Errors.
- `npm run check:tokens && npm run check:i18n && npm run check:commands` -- expected: xanh.
- `npx vue-tsc --noEmit` -- expected: không lỗi.
- `npm run test:e2e -- --spec e2e/specs/story-12-24-library-wide-grid.e2e.mjs` (ngoài sandbox, cổng 1420 trống) -- expected: xanh; cổng bận thì ghi lại, không suy ra.

**Manual checks:**
- Ice: ảnh app thật năm bề mặt, sáng và tối (mục nợ `Chủ: Epic 12`).

## Auto Run Result

- Thay đổi: `LibraryMode`, `LibraryWorkPage`, `LibrarySearchResults`, `LibraryUrlImportScreen`, `ReadingMode` chuyển sang lớp `ui-*` của story 27; CSS dáng scoped xoá, chỉ giữ bố cục; ca canh chuyển đổi đọc thêm năm tệp.
- Tệp: `src/modes/LibraryMode.vue` (ô tìm `ui-field` có biểu tượng tuyệt đối, `ui-seg` chế độ dấu và phạm vi với radio ẩn `lib-sr-only`, bốn ô chọn dạng field, mục menu `ui-menu-item` + `ui-menu-sep`, "Tiếp tục dịch" trong `work-go-slot` nền mực, `‹ ›` `ui-icon-btn`); `LibraryWorkPage.vue` (chip lọc Chương `ui-chip`, ô tìm `sm` trên nền trũng, thanh chọn `sm`, ô đổi tên/áp dụng tới `ui-field`); `LibrarySearchResults.vue` (nút `sm`, `‹ ›` `ui-icon-btn--sm`); `LibraryUrlImportScreen.vue` (ô dán `ui-field`, cột số dòng ngoài ô theo `ui-md-wrap`, radio `ui-check`, nút `ui-btn`); `ReadingMode.vue` (thanh công cụ `sm`, mức đọc `ui-seg`, thanh trượt `tuner-range`); `tests/frontend/controls-adoption.test.ts` (năm tệp, `.lib-sr-only`, `.tuner-range`); `tests/frontend/libraryTopBar.test.ts` (hai ca ma trận); `deferred-work.md` (hai mục `Chủ: Epic 12`).
- Review: 8 phát hiện; 1 patch (low: comment tiếng Anh), 1 defer (low: nút `aria-pressed` Chế độ đọc), 6 bác (3 false, 3 low — lý do ở Triage Log).
- Follow-up review: false (patch 1 low, 0 medium, 0 high).
- Kiểm: vitest 19 tệp xanh 503/503, mã thoát 0, không dòng Errors (trước bản vá comment); sau bản vá, cùng lệnh đỏ do timeout ở các ca khác nhau mỗi lượt khi tải máy 82–103 (Blender ~960% CPU), bốn tệp bị timeout chạy riêng xanh 141/141; `check:tokens`, `check:i18n`, `check:commands`, `check:debt-owner`, `vue-tsc --noEmit` xanh; e2e ngoài sandbox: `story-12-24` 6/6, `story-5-4` 2/2, `story-5-6` 1/1, `story-12-5` 5/6 (ca đỏ có từ trước, ghi nợ); đối chứng gỡ thật cho ca canh (Gộp Chương, `tuner-range`) và hai ca ma trận đỏ đúng ca, trả lại trùng byte.
- Rủi ro còn: chưa ai nhìn app thật năm bề mặt ở hai theme; `ui-seg` 24px giữa thanh trên 28px; mục Trạng thái chọn chỉ tô `surface-accent`; nút `aria-pressed` Chế độ đọc không dáng bấm; lượt e2e `story-12-24` đầu đỏ ở `before all` chưa rõ nguyên nhân; toàn bộ vitest chưa chạy lại xanh sau bản vá comment vì tải máy.
