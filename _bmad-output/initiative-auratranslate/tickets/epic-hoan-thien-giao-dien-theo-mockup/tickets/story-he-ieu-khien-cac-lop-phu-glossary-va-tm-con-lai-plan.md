---
title: 'Hệ điều khiển: các lớp phủ Glossary và TM còn lại'
type: 'refactor'
ticket: '28'
created: '2026-10-28'
status: 'done'
baseline_revision: '2e51da4aa2555ea4643c6b72dbc8a806f9152c5b'
route: 'full'
route_source: 'auto'
risk: 'medium'
review: 'quick'
review_source: 'pinned'
lenses_ran: ['quick']
review_loop_iteration: 0
followup_review_recommended: false
context:
  - '{project-root}/_bmad-output/initiative-auratranslate/tickets/epic-hoan-thien-giao-dien-theo-mockup/tickets/story-he-ieu-khien-lop-chung-menu-dung-chung-va-lop-phu-quan-ly-gl-plan.md'
  - '{project-root}/src/AGENTS.md'
warnings: ['oversized']
deferred:
  - summary: >-
      e2e glossary-quick-add.e2e.mjs chưa chạy sau khi đổi lớp của ba ô nhập GlossaryQuickAdd.
    evidence: |-
      Cổng 1420 bận (tauri dev của Ice) suốt lượt; e2e đọc ô đầu theo thứ tự DOM, thứ tự không đổi theo template. Lượt nightly e2e kế tiếp phân xử.
    location: >-
      e2e/specs/glossary-quick-add.e2e.mjs
    severity: low
---

<intent-contract>

## Intent

**Problem:** Sáu bề mặt Glossary/TM còn tự vẽ nút, ô nhập, checkbox, radio, công tắc bằng lớp scoped riêng; ba nút chính tô `primary`, hai nút chính tô nền `primary` (DESIGN.md §Điều khiển).

**Approach:** Chuyển `GlossaryQueueOverlay`, `GlossaryHarvestOverlay`, `GlossaryImportOverlay`, `GlossaryQuickAdd`, `TmManageOverlay`, `TmImportOverlay` sang lớp `ui-*` của story 27, xoá CSS dáng scoped, và nối sáu tệp vào ca vitest canh chuyển đổi.

## Decisions (build-auto, 2026-10-28)

1. Bậc: ô nhập và cặp nút gửi của một form trong lớp phủ là `lg` (ô sửa + Lưu/Huỷ của `TmManage`; chân Xác nhận/Huỷ của `TmImport`, `GlossaryImport`, `GlossaryHarvest`, kèm `ui-btn--pair`). Thân lớp phủ (thanh lọc, thanh hành động, nút đóng đầu lớp phủ) giữ `md` mặc định theo tiền lệ quyết định 3 của story 27. Hàng dày (nút trong hàng bảng, nút từng mục Harvest) và dải `GlossaryQuickAdd` (dải neo đáy Workspace, không phải lớp phủ) là `sm`.
2. Nút Chấp nhận trên từng mục Harvest là nút Phụ `sm`: DESIGN §Nút cho tối đa một nút Chính mỗi panel, nút Chính là chân "nhận các mục đã chọn".
3. `ui-switch` thêm dáng Khoá trong `controls.css` (rãnh viền đứt `outline`, chữ `on-surface-variant`, con trỏ mặc định, không hover) vì công tắc TM khoá thường xuyên và lớp chưa có `:disabled`.
4. Bỏ tô viền `tm-rule` của chip "nguồn khác" (`tm-pill-other`): `ui-chip` một dáng; giữ lớp làm móc.

## Boundaries & Constraints

**Always:**
- Ràng buộc "Always" của story 27 (token AD-34, một thuộc tính mỗi trạng thái, Kiểm H, scoped chỉ còn bố cục, `@click` một `dispatch`, `aria-*`/phím giữ nguyên) vẫn giữ.
- Lớp móc mà test/e2e đọc ở lại trong template, không mang CSS dáng; thứ tự DOM của nút/ô mà test đọc theo chỉ số giữ nguyên.
- Ngoại lệ trong ca canh là bảng có tên + lý do, khoá đúng một phần tử.
- Mục nợ mới `Chủ: Epic 12` cho lượt nhìn app thật.

**Never:**
- Không sửa tệp của story 29–33 hay bốn menu/`GlossaryManageOverlay` của 27 (trừ `controls.css` cho quyết định 3).
- Không thêm dependency, token màu, `check-tokens.mjs`.
- Không đổi lớp `ui-*` hiện có ngoài `ui-switch`.

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected Output / Behavior | Error Handling |
|----------|--------------|---------------------------|----------------|
| Công tắc TM khoá | `TmManage` công tắc `disabled` | Rãnh viền đứt, rê chuột không đổi viền | — |
| Chip lọc chọn | Radio trong `ui-chip`/`ui-seg__item` | Radio ẩn thị giác, chip đổi dáng chọn qua `:has(input:checked)`; Tab vào thấy focus | — |
| Radio con trỏ hàng Queue | `input[name=gq-row-cursor]` ẩn, cả hàng là vùng bấm | Không nhận `ui-*`; ca canh miễn trừ đích danh | — |
| TmImport đang xác nhận | `tmImportConfirming` | Hai nút chân `disabled`, dáng Khoá của `ui-btn` (không nền `primary`) | — |

</intent-contract>

## Code Map

Số dòng ở mốc `2e51da4a`. Mẫu chuyển: `GlossaryManageOverlay.vue` (giữ lớp cũ cạnh lớp mới, xoá rule dáng cũ, radio ẩn bằng lớp `*-sr-only` + `SHAPE_EXCEPTIONS`).

- `tests/frontend/controls-adoption.test.ts` -- thêm sáu tệp vào `FILES`; `SHAPE_EXCEPTIONS` thêm `gq-sr-only`, `tm-sr-only` (radio ẩn); thêm bảng miễn trừ "control không cần nhãn `ui-*`" khoá đúng `input[name="gq-row-cursor"]` (nhãn tổ tiên `gq-row-label` là cả hàng, `display: contents`). Ca "reaches every file" vẫn đòi ≥1 control mỗi tệp.
- `src/tokens/controls.css` -- `ui-switch:disabled` (quyết định 3); hover rãnh chỉ khi không khoá; đặt trước khối `forced-colors`. Ca mới ở `tests/frontend/controls-css.test.ts` (khuôn dòng 73-75).
- `src/GlossaryQueueOverlay.vue` -- đóng :254 → `ui-btn--quiet`; chip :395 label → `ui-chip`, radio :401 thêm `gq-sr-only`; giữ `:class` `on` (móc `glossaryQueue.test.ts:648`); :436 Chấp nhận → `ui-btn ui-btn--primary`; :439-448 bốn nút → `ui-btn`; `<kbd>` → `ui-kbd`. Xoá 509-524, 730-736, 850-870, 890-909. Giữ `.gq-sr-only` 815-825.
- `src/GlossaryHarvestOverlay.vue` -- đóng :133 → quiet; checkbox :166 bọc `<label class="ui-check ui-check--bare">`; :187, :195 → `ui-btn ui-btn--sm` (giữ `gh-mini`, `gh-mini-primary` làm móc; `glossaryHarvest.test.ts:202` focus nút đầu = Chấp nhận); kbd → `ui-kbd`; chân :216 → `ui-btn ui-btn--lg ui-btn--primary ui-btn--pair`, :225 → `ui-btn ui-btn--lg ui-btn--pair`. Xoá 283-298, 389-413, 437-455.
- `src/GlossaryImportOverlay.vue` -- đóng :122 → quiet; `gi-radio-label` :202-219 → thêm `ui-check` (giữ radio thật, `glossaryImportPreview.test.ts:331`); chân :233/:247 như Harvest. Xoá 304-319, 427-435, 450-474.
- `src/GlossaryQuickAdd.vue` -- ba `gqa-input` :129/142/153 → `ui-field ui-field--sm` (e2e lấy ô đầu theo thứ tự DOM); `gqa-radio` :175/183/194 → `ui-check`; :235 → `ui-btn ui-btn--sm ui-btn--primary`, :244 → `ui-btn ui-btn--sm ui-btn--quiet`. Xoá 299-312, 356-380. Giữ `fieldset.gqa-group` (viền fieldset, `@keydown` số).
- `src/TmManageOverlay.vue` (template 495-930, style 978-1563) -- đóng :495 quiet; ô tìm :507 → `ui-field` (md, phải là `input` đầu của `.tm-toolbar`); nhóm tầng `fieldset.tm-seg` :517 → `ui-seg`, nhãn → `ui-seg__item` (xoá `border: none` :1161); hai nhóm nguồn :535/:552 nhãn → `ui-chip`; radio của ba nhóm thêm `tm-sr-only` (lớp mới, sao `gq-sr-only`); `tm-link` :626 → `ui-btn ui-btn--sm ui-btn--quiet`; Sửa/Xoá hàng :719/:724 → `ui-btn ui-btn--sm` (+`'ui-btn--danger'` cùng `:class` với `tm-act-danger`); ô sửa :750 → `ui-field ui-field--lg`; Lưu :766/Huỷ :769 → `ui-btn--lg` (+`--primary`) `--pair`; :817, :830 (+danger), :839, :842, :912, :921, :930 → `ui-btn`; công tắc :851 → `tm-switch ui-switch` + con `<span class="ui-switch__track" aria-hidden="true">` (bỏ `tm-act`); :874/:885/:896 `tm-radio-label` → `ui-check`. Xoá 1026-1041, 1164-1191, 1213-1226, 1383-1385, 1435-1463, 1499-1514. Móc: `.tm-actions button:nth-of-type(1|2)`, `.tm-exchange-actions button`[0/1], `.tm-exchange input[type="checkbox"]` (chỉ một), `[role="switch"]`, `[data-tm-import-open]`, `.tm-edit-form input`.
- `src/TmImportOverlay.vue` -- đóng :82 quiet; `label.ti-own` :114 → thêm `ui-check`; :137/:146 → `ui-btn--lg` (+`--primary`) `--pair`, giữ `ti-act-primary`. Xoá 203-218, 304-328. Bẫy focus `tmImport.test.ts:468-473` cần nút đầu = đóng, cuối = huỷ.
- Comma list: không rule xoá nào nằm trong danh sách selector chung (đã kiểm `gq` 624/680/685/738/765, `gh` 300/311, `gi` 379, `tm` 1043/1256, `gqa` 291). ⚠️ Story 26 và 27 đều làm treo selector khi xoá; đọc lại khối quanh mỗi chỗ xoá.

## Tasks & Acceptance

**Execution:**
- [ ] `src/tokens/controls.css`, `tests/frontend/controls-css.test.ts` -- dáng Khoá `ui-switch` + ca -- quyết định 3.
- [ ] Sáu tệp `.vue` ở Code Map -- đổi lớp, xoá CSS dáng -- như Code Map.
- [ ] `tests/frontend/controls-adoption.test.ts` -- sáu tệp + miễn trừ đích danh -- ca canh nối dài. Đối chứng: gỡ `ui-btn` khỏi một nút `TmManageOverlay` ⇒ đỏ đúng tệp:dòng, rồi trả lại.
- [ ] `tests/frontend/` -- sửa test hiện có chỉ khi móc đổi, ghi lý do.
- [ ] `_bmad-output/initiative-auratranslate/deferred-work.md` -- một mục `Chủ: Epic 12` dưới tiêu đề mới: ảnh app thật sáu bề mặt, sáng/tối, cạnh `he-dieu-khien.html`; `ui-kbd` trong nút Chính Queue; chip "nguồn khác" mất viền `tm-rule`.

**Acceptance Criteria:**
- Given một trong sáu bề mặt mở, when nhìn mọi control ở hai theme, then dáng theo bậc ở quyết định 1, không control nào tô `primary` hay giữ dáng gốc hệ điều hành.
- Given bàn phím, when Tab qua mọi control của `TmManageOverlay` và `GlossaryQuickAdd`, then dấu focus theo bảng §Trạng thái.
- Given ca canh chuyển đổi, when gỡ lớp `ui-*` khỏi một control của sáu tệp, then nó đỏ đúng tệp:dòng.

## Implementation Notes

- Nhóm tầng của `TmManageOverlay` không đặt `ui-seg` lên `fieldset`: `ui-seg` có `overflow: hidden` và sẽ cắt `<legend>`; `fieldset.tm-seg` giữ bố cục, một `div.ui-seg` bọc các nhãn `ui-seg__item`.
- Ca canh có bảng `LABEL_EXEMPT` khoá đúng `input[name="gq-row-cursor"]`; `SHAPE_EXCEPTIONS` thêm `gq-sr-only`, `tm-sr-only`.
- Đối chứng: gỡ `ui-btn` khỏi nút Trước (rồi nút Sau) của `TmManageOverlay` ⇒ đỏ đúng `:844`/`:847 <button>`, trả lại ⇒ xanh.
- Ma trận hàng 2 và 4 thêm ca riêng: `tmManage.test.ts` (radio lọc ẩn trong nhãn `ui-seg__item`/`ui-chip`), `tmImport.test.ts` (đang xác nhận ⇒ hai nút chân khoá, mang `ui-btn`), `controls-css.test.ts` (nhãn bọc radio chọn ⇒ nền `on-surface`; `ui-btn:disabled` viền đứt).
- ⚠️ Lượt chạy 12 tệp đỏ 1–5 ca, mỗi lượt một bộ khác, tất cả "Test timed out in 5000ms" (kể cả hàm thuần `quickAddModeFor`) khi tải máy 43–81 trên 16 lõi (Blender); các tệp đó xanh khi chạy riêng và với `--testTimeout=60000`. e2e `glossary-quick-add` chưa chạy: cổng 1420 bận (`tauri dev` của Ice).

## Plan Change Log

## Review Triage Log

### 2026-10-28 — Review pass
- verdicts: 8 findings — high 0, medium 1, low 4, false 3, maybe-false 0
- findings:
  - `[medium]` `[patch]` Radio chip Queue `gq-sr-only` tuyệt đối mà `label.gq-chip` không định vị — `.ui-chip` không có `position`, rule `.gq-chip` cũ đã xoá; thêm `.gq-chip { position: relative; }`.
  - `[low]` `[reject]` Công tắc khoá thắng hover nhờ thứ tự nguồn; khoá-bật chỉ khác khoá-tắt ở vị trí núm; nháy viền đứt khi `tmPolicyBusy` — thắng bằng độ đặc hiệu (0,4,0)/(0,3,0) > (0,3,0)/(0,2,0), không bằng thứ tự; vị trí núm là hình, không màu; nháy khi bận là trạng thái khoá có sẵn từ trước, sửa cần thêm nhánh `aria-busy`.
  - `[false]` `[reject]` `forced-colors` thiếu `.ui-seg__item:has(input:focus-visible)` — focus của `ui-seg__item` là `outline` 2px (không `outline: none`), `forced-colors` giữ outline và chỉ ép màu, nên dấu focus còn.
  - `[false]` `[reject]` Ca công tắc khoá vẫn xanh nếu dời khối lên trước — khối thắng bằng độ đặc hiệu ở mọi vị trí, nên dời không tắt khoá.
  - `[low]` `[patch]` Tên ca tmImport nói "thay vì nền primary" mà không kiểm — đổi tên thành "both footer buttons are disabled and carry the ui-btn classes".
  - `[low]` `[patch]` `tm-pill-on` còn bind mà không CSS, không test đọc — xoá khỏi ba `:class`, giữ `tm-pill-other`.
  - `[false]` `[reject]` Nhãn `ui-check--bare` của Harvest làm vùng đệm bật checkbox — đúng DESIGN (vùng bấm 24×24, WCAG 2.5.8) và plan; không có kết quả xấu.
  - `[low]` `[defer]` e2e `glossary-quick-add` chưa chạy — cổng 1420 bận; ghi vào mục nợ `Chủ: Epic 12` của ticket này, đọc lượt nightly.

## Verification

**Commands:**
- `npx vitest run tests/frontend/controls-adoption.test.ts tests/frontend/controls-css.test.ts tests/frontend/glossaryQueue.test.ts tests/frontend/glossaryQueueOpener.test.ts tests/frontend/glossaryHarvest.test.ts tests/frontend/glossaryImportPreview.test.ts tests/frontend/glossaryQuickAdd.test.ts tests/frontend/glossaryQuickAddStrip.test.ts tests/frontend/tmManage.test.ts tests/frontend/tmImport.test.ts tests/frontend/dialogAccessibleName.test.ts tests/frontend/reviewerImportFocus.test.ts` -- expected: mã thoát 0, không dòng Errors.
- `npm run check:tokens && npm run check:i18n` -- expected: xanh.
- `npm run test:e2e -- --spec e2e/specs/glossary-quick-add.e2e.mjs` (ngoài sandbox, cổng 1420 trống) -- expected: xanh; cổng bận thì ghi lại, không suy ra.

**Manual checks:**
- Ice: ảnh app thật sáu bề mặt, sáng và tối, cạnh `he-dieu-khien.html` (mục nợ `Chủ: Epic 12`).

## Auto Run Result

- Thay đổi: sáu bề mặt Glossary/TM chuyển sang lớp `ui-*` của story 27, xoá CSS dáng scoped; `ui-switch` có dáng Khoá; ca canh chuyển đổi đọc thêm sáu tệp.
- Tệp: `src/GlossaryQueueOverlay.vue`, `src/GlossaryHarvestOverlay.vue`, `src/GlossaryImportOverlay.vue`, `src/GlossaryQuickAdd.vue`, `src/TmManageOverlay.vue`, `src/TmImportOverlay.vue` (lớp `ui-*`, xoá rule dáng); `src/tokens/controls.css` (`ui-switch:disabled`); `tests/frontend/controls-adoption.test.ts` (sáu tệp, miễn trừ đích danh); `tests/frontend/controls-css.test.ts`, `tmManage.test.ts`, `tmImport.test.ts` (ca ma trận); `deferred-work.md` (mục `Chủ: Epic 12`).
- Review: 8 phát hiện; 3 patch (1 medium, 2 low), 1 defer (e2e chưa chạy), 4 bác (1 low: sửa cần thêm nhánh `aria-busy`; 3 false: xem Triage Log).
- Follow-up review: false (một medium được patch, không high).
- Kiểm: 12 tệp vitest của Verification — mọi đỏ là "timed out 5000ms" dưới tải 43–81, năm tệp đó xanh 126/126 với `--testTimeout=60000`; `controls-adoption` 23/23; `check:tokens`, `check:i18n` xanh; `vue-tsc --noEmit` không lỗi; đối chứng gỡ `ui-btn` ⇒ đỏ đúng `src/TmManageOverlay.vue:847`. e2e chưa chạy.
- `check:debt-owner` đỏ vì chính plan này: 215 ticket có plan, sàn `PLANNED_TICKET_FLOOR` 171 dưới 80%; nâng lên 183 = ceil(0,85 × 215) như cổng ghi, nay xanh. Chạm `scripts/check-*` nên lượt đủ bộ cần chạy (pre-push); chưa chạy tay vì tải máy 81.
- Rủi ro còn: chưa ai nhìn app thật (mục nợ `Chủ: Epic 12`); e2e QuickAdd chờ nightly.

