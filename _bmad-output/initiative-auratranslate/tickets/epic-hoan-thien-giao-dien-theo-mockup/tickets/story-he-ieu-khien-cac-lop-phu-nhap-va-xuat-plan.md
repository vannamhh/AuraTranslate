---
title: 'Hệ điều khiển: các lớp phủ Nhập và Xuất'
type: 'refactor'
ticket: '30'
created: '2026-10-28'
status: 'done'
baseline_revision: '3471d47efba801ffb1dcfe218a61b8a3d4d618d5'
route: 'full'
route_source: 'auto'
risk: 'medium'
review: 'quick'
review_source: 'pinned'
lenses_ran: ['quick']
review_loop_iteration: 0
followup_review_recommended: false
context:
  - '{project-root}/_bmad-output/initiative-auratranslate/tickets/epic-hoan-thien-giao-dien-theo-mockup/tickets/story-he-ieu-khien-cac-lop-phu-prompt-ai-soat-va-lich-su-plan.md'
  - '{project-root}/src/AGENTS.md'
warnings: ['oversized']
deferred:
  - summary: >-
      AC2 chưa chứng minh: e2e `story-12-6-import-preview` (đọc `lang-chip--on`, bấm nhãn cha của radio ngôn ngữ, `ip-act-primary`, `bip-act-primary`) chưa chạy.
    evidence: |-
      Cổng 1420 bận (`tauri dev` của Ice, pid 27848). Vitest phủ móc `lang-chip`/`lang-chip--on` và nhãn `ui-chip` (`libraryImportOverlay.test.ts`), nút chân của hai bản xem trước; không phủ cú bấm thật trong webview. Lượt e2e nightly hoặc chạy tay khi cổng trống phân xử; đã có trong mục nợ Chủ: Epic 12 của ticket này.
    location: >-
      e2e/specs/story-12-6-import-preview.e2e.mjs
    severity: medium (unverified)
---

<intent-contract>

## Intent

**Problem:** Chín lớp phủ Nhập/Xuất và hai component con (`ChapterOrigin`, `modes/SourceLangChips`) còn tự vẽ 87 control bằng lớp scoped: nút Đóng và nút hành động gạch chân `outline`, nút chính tô `primary`, chip lọc viên thuốc, ô nhập viền `outline` hoặc dáng gốc, checkbox/radio dáng hệ điều hành (DESIGN.md §Điều khiển).

**Approach:** Chuyển mười một tệp sang lớp `ui-*` của story 27, xoá CSS dáng scoped, nối mười một tệp vào ca canh chuyển đổi; không thêm lớp mới vào `controls.css`.

## Decisions (build-auto, 2026-10-28)

1. Bậc, theo quyết định 1 của story 28 và 29: cặp chân Xác nhận/Huỷ là `lg` + `ui-btn--pair`; chân chỉ một nút xác nhận là `lg` không `--pair`; form nhập của `LibraryImportOverlay` (ô nhập, ô nhiều dòng, ba nút gửi) là `lg`; nút Đóng, thanh hành động thân lớp phủ, chân Attribution là `md`; nút trong hàng dày là `sm`. Ô nhập và nút đứng cùng hàng với một `SelectMenu variant="field"` là `md`, vì ô chọn đó là `ui-field` bậc `md` và `DropdownMenu` (story 27) không có bổ nghĩa bậc. `ChapterOrigin` là `md` (thân panel, không phải form gửi, không phải hàng dày).
2. Giữ phân cấp hiện có (quyết định 5 của story 29): `ui-btn--primary` chỉ ở nút đang mang `*-act-primary`/`attr-action-primary`; `LibraryImportOverlay` không có nút Chính; không thêm Nguy hiểm, trừ `ip-cleanup-delete` gắn `ui-btn--danger` cùng điều kiện với `ip-cleanup-delete-pending` (tiền lệ `pl-act-danger`).
3. Nút dạng liên kết trong câu (`ip-strip-open`, `ip-domain-log-view`) là `ui-btn ui-btn--quiet ui-btn--sm`: DESIGN không có loại liên kết, mọi nút thuộc năm bậc.
4. Thẻ ứng viên mã hoá (`ip-candidate`, `bip-candidate`) giữ dáng thẻ của mockup `web-import.html` (`.candrow`) như quyết định 3 của story 29: nhãn thẻ thêm `ui-check` để radio vẽ bằng token, radio vẫn hiện, vẫn là radio trong `radiogroup`; lớp thẻ vào `NOT_A_CONTROL`. Ca "khoá chết" hiện chỉ nhận khoá nằm trên chính control, nên nới thành: khoá nằm trên control hoặc trên `label` bọc control.
5. Chip lọc Chương (`ip-chapter-filter-chip`, `bip-chapter-filter-chip` dạng `<button>`) thành `ui-chip`; `aria-pressed` có sẵn đã tô trạng thái chọn nên bỏ bind `*-active`; bỏ tô `error` của `*-needs-review` (một dáng chip, tiền lệ quyết định 4 của story 28), giữ lớp làm móc. `<span>` đếm `*-chip-clean` không phải control: chữ thường `ui-sm` `on-surface-variant`, không viền.
6. Radio ẩn của hàng ghép `AlignmentOverlay` theo tiền lệ `gq-row-cursor`: `LABEL_EXEMPT` cho radio, `SHAPE_EXCEPTIONS` cho `.al-sr-only`, `.al-row-label { position: relative }`.
7. `<kbd>` của chip lọc Bilingual và dòng gợi ý Attribution thêm `ui-kbd`.

## Boundaries & Constraints

**Always:**
- Ràng buộc "Always" của story 27–29: token AD-34, mỗi trạng thái một thuộc tính, scoped chỉ còn bố cục, `aria-*`/phím/`data-*` giữ nguyên, thứ tự DOM của nút/ô giữ nguyên.
- Lớp mà test/e2e đọc ở lại trong template, không mang CSS dáng (danh sách ở Code Map).
- Mỗi ngoại lệ mới trong ca canh có tên và lý do; ngoại lệ không còn khớp thì ca đỏ.
- Mục nợ mới `Chủ: Epic 12` cho lượt nhìn app thật.

**Never:**
- Không sửa `controls.css`, tệp của story 27–29 và 31–33 (kể cả `LibraryWorkPage.vue`, `LibraryUrlImportScreen.vue` dù nhúng hai component con).
- Không thêm dependency, token, sửa `check-tokens.mjs`, sửa DESIGN.md, đổi lớp `ui-*` hiện có.
- Không dời viền thẻ sang `box-shadow` hay mẹo khác để né ca canh.

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected Output / Behavior | Error Handling |
|----------|--------------|---------------------------|----------------|
| Chọn ứng viên mã hoá | Bấm thẻ thứ hai | Radio token tô `on-surface`, thẻ đổi viền chọn; Tab vào radio thấy focus | — |
| Lớp miễn trừ thẻ chết | Bỏ `ip-candidate` khỏi nhãn | Ca khoá chết đỏ | — |
| Chip ngôn ngữ nguồn | Chọn `zh` | Nhãn `ui-chip` nền `on-surface` qua `:has(input:checked)`, `lang-chip--on` vẫn gắn | — |
| Xoá luật dọn hai bước | Bấm Xoá lần một | Nút mang `ui-btn--danger` và `ip-cleanup-delete-pending` | — |
| Radio ẩn hàng ghép | Radio `al-sr-only` trong `label.al-row-label` không mang `ui-*` | Ca canh xanh nhờ `LABEL_EXEMPT`; xoá mục đó ⇒ đỏ đúng `AlignmentOverlay.vue:210` | — |

</intent-contract>

## Code Map

Số dòng ở mốc `3471d47e`. Mẫu chuyển: `src/PromptLibraryOverlay.vue`, `src/TmManageOverlay.vue`, `src/GlossaryQueueOverlay.vue` (`gq-sr-only`, `gq-row-label`), `src/GlossaryHarvestOverlay.vue:166` (`label.ui-check.ui-check--bare` bọc checkbox đứng riêng). Bộ rule mẫu ở mọi lớp phủ: `.XX-close`, `.XX-close:focus-visible`, `.XX-act`, `.XX-act:not(.XX-act-primary):focus-visible`, `.XX-act-primary`, `.XX-act:disabled` ⇒ xoá cả rule, giữ `.XX-actions`/`.XX-head` bố cục. ⚠️ Danh sách dấu phẩy: xoá trọn danh sách, đọc lại khối quanh mỗi chỗ xoá.

- `src/ImportPreviewOverlay.vue` (template 832-1715, style 1716-2708, không nút Đóng đầu) -- `sm`: 900 `ip-url-reload`, 910/995 `ip-url-remove`, 1378 `ip-cleanup-edit`, 1392 `ip-cleanup-delete` (+ `'ui-btn--danger'` trong cùng `:class`), 1652 `ip-act` huỷ tải ảnh. Quiet `sm`: 1128 `ip-strip-open`, 1677 `ip-domain-log-view`. `md` cạnh `SelectMenu`: 1072 `ip-chapters-pattern-input` → `ui-field ui-field--full`; 1332 `ip-cleanup-edit-pattern`, 1413 `ip-cleanup-add-pattern` → `ui-field`; 1341 save → `ui-btn`, 1344 cancel → `ui-btn ui-btn--quiet`, 1427 add-submit → `ui-btn`. 1356 `ip-cleanup-tick` bọc `label.ui-check.ui-check--bare` (giữ `aria-label` trên input). 1136 nhãn `ip-candidate` thêm `ui-check` (quyết định 4; `.ip-candidate` giữ `display:flex; flex-direction:column`). 1460 chip → `ip-chapter-filter-chip ip-chapter-filter-chip-needs-review ui-chip`, bỏ bind `-active`; 1475 span đếm (quyết định 5). 1514 nhãn `ip-chapters-sort-toggle` thêm `ui-check`. 1688 → `ip-act ip-act-primary ui-btn ui-btn--lg ui-btn--primary ui-btn--pair`, 1702 → `ip-act ui-btn ui-btn--lg ui-btn--pair`. CSS xoá/rút: 1867 (giữ `margin-left`), 1879, 2195-2225 (bốn selector, rút về `flex: none`), 2246/2257, 2275/2286, 2291/2302/2307, 2341/2350, 2494-2522 (giữ `margin: 0` cho span qua lớp `-clean`), 2620-2639 (rút về `flex: none`), 2664 (giữ `margin-left`), 2683-2708. Giữ `.ip-candidate*` (lớp thẻ miễn trừ).
- `src/BilingualImportPreviewOverlay.vue` (template 359-797, style 799-1272) -- 441 `bip-swap ui-btn`; 463 nhãn `bip-checkbox-field ui-check`; 472 ô mẫu Chương → `ui-field ui-field--full` (`md`); 508 nhãn `bip-candidate` thêm `ui-check`; 556 chip như ImportPreview, `kbd.bip-chapter-filter-key` thêm `ui-kbd`; 692/700 `bip-mismatch-action ui-btn ui-btn--sm`; 729/738 `bip-act ui-btn ui-btn--lg`; 748 `bip-act bip-act-primary ui-btn ui-btn--lg ui-btn--primary ui-btn--pair`; 756 `bip-act ui-btn ui-btn--lg ui-btn--pair`. Xoá 978-1000 (ba danh sách hai selector), 1019-1051, 1239-1262. Giữ 912-930 (`.bip-candidate*`), `.bip-chapters-entry*`, `.bip-mismatch-focus*`.
- `src/LibraryImportOverlay.vue` (template 121-318, style 320-526) -- 130 `li-close ui-btn ui-btn--quiet`; 143 `btn ui-btn` (`md`, không cùng hàng form); 157-168 nhãn `radio-field ui-check` (giữ `align-items: flex-start`); ô nhập ~188/189/212/230/258/283 → `ui-field ui-field--lg ui-field--full`; 247 textarea → `ui-field ui-field--full`; 252/276/302 → `btn ui-btn ui-btn--lg`. Xoá 368-384, 401-418 (`.field input, .field select, .field textarea` và `:focus-visible`: không bị ca báo nhưng scoped thắng `.ui-field`), 469-490 (giữ `.btn { align-self: flex-start }`). Giữ `.dropzone`, `.field`.
- `src/ReviewerImportOverlay.vue` -- 83 `ri-close ui-btn ui-btn--quiet`; 195 `ri-act ri-act-primary ui-btn ui-btn--lg ui-btn--primary ui-btn--pair`; 203 `ri-act ui-btn ui-btn--lg ui-btn--pair`. Xoá 277-293, 376-400.
- `src/ExportOverlay.vue` (template 204-613, style 615-861) -- 209 `ex-close` quiet; mọi `label.ex-choice` (223, 239, 253, 318, 330, 342, 351, 367, 377, 397, 422, 432, 460) thêm `ui-check`; 502 `ex-act ui-btn`; 600 `ex-act ui-btn ui-btn--lg ui-btn--pair`; 603 `ex-act ui-btn ui-btn--lg ui-btn--primary ui-btn--pair`. Xoá 663-680, 772-790; `.ex-choice` 698 bỏ phần trùng `ui-check` (font, màu).
- `src/AlignmentOverlay.vue` -- 164 `al-close` quiet; 268 `al-act al-act-primary ui-btn ui-btn--primary`; 272/275/279/282 `al-act ui-btn`; `.al-row-label` thêm `position: relative` (giữ danh sách `.al-cols, .al-row-label` 404). Xoá 344-360, 490-510.
- `src/ChapterMergeOverlay.vue` -- 84 `cm-close` quiet; 141 `cm-act cm-act-primary ui-btn ui-btn--lg ui-btn--primary`. Xoá 203-219, 302-327.
- `src/ChapterSplitOverlay.vue` -- 91 `cs-close` quiet; 104 nhãn `cs-label ui-check`; 136 `cs-act cs-act-primary ui-btn ui-btn--lg ui-btn--primary`. Xoá 198-214, 274-299.
- `src/AttributionOverlay.vue` -- 203 `attr-close` quiet; 329/359 `attr-action ui-btn`; 350 `attr-action attr-action-primary ui-btn ui-btn--primary` (vẫn là nút chân đầu tiên); 371 `kbd` thêm `ui-kbd`, xoá `.attr-hint kbd` 547. Xoá 413-429, 558-590.
- `src/ChapterOrigin.vue` -- 66/77/88/99 `chapter-origin-input ui-field ui-field--full`. Xoá 144-158, 166-170; giữ `::placeholder` 160 (token nghiêng riêng).
- `src/modes/SourceLangChips.vue` -- nhãn `lang-chip ui-chip`, giữ bind `lang-chip--on`; `.lang-chip` rút về `position: relative`; xoá 66-89 trừ `.lang-chip-input`.
- `tests/frontend/controls-adoption.test.ts` -- `FILES` thêm mười một tệp; `NOT_A_CONTROL` thêm `src/ImportPreviewOverlay.vue .ip-candidate`, `src/BilingualImportPreviewOverlay.vue .bip-candidate` (lý do quyết định 4); `LABEL_EXEMPT` thêm `src/AlignmentOverlay.vue input[name="al-row-cursor"]`; `SHAPE_EXCEPTIONS` thêm `src/AlignmentOverlay.vue .al-sr-only`; ca khoá chết nhận khoá trên control hoặc trên `label` có control con.

Móc phải giữ: `ip-act`, `ip-act-primary`, `ip-chapter-filter-chip-needs-review`, `ip-chapters-sort-toggle`, `ip-chapters-pattern-input`, `ip-cleanup-tick|add-pattern|edit-pattern|delete-pending|add|action-form|edit-form`, `ip-candidate-radio`, `ip-domain-log-view`, `ip-url-action-form`; `bip-swap`, `bip-act`, `bip-act-primary`, `bip-chapter-filter-chip-needs-review`; `.import-form`, `.field`, `.li-panel`, `.error`; `al-act-primary`; `attr-action`, `attr-action-primary`, `.attr-foot button`, `.attr-foot kbd`; `chapter-origin-input`; `lang-chip`, `lang-chip--on` (e2e `story-12-6` bấm nhãn cha của radio).

## Tasks & Acceptance

**Execution:**
- [x] Mười một tệp `.vue` ở Code Map -- đổi lớp, xoá CSS dáng -- quyết định 1–7.
- [x] `tests/frontend/controls-adoption.test.ts` -- mười một tệp, ba bảng miễn trừ, ca khoá chết nới cho `label` -- ca canh nối dài. Đối chứng thật: gỡ `ui-btn` khỏi nút `ip-act-primary` ⇒ đỏ đúng tệp:dòng; bỏ `ip-candidate` khỏi nhãn ⇒ đỏ ca khoá chết; xoá mục `LABEL_EXEMPT` của Alignment ⇒ đỏ; cả ba trả lại.
- [x] `tests/frontend/` -- sửa test hiện có chỉ khi móc đổi, ghi lý do.
- [x] `_bmad-output/initiative-auratranslate/deferred-work.md` -- cuối tệp, tiêu đề `## Deferred from: hệ điều khiển các lớp phủ Nhập và Xuất (2026-10-28, ticket 30 của Epic 12)`, một mục ≤ 5 dòng `Chủ: Epic 12`: ảnh app thật chín lớp phủ và hai component con (cả trong Library), sáng/tối, cạnh `he-dieu-khien.html`; thẻ ứng viên giữ dáng mockup màn (quyết định 4); ô `md` cạnh ô chọn (quyết định 1); e2e `story-12-6-import-preview` nếu chưa chạy.

**Acceptance Criteria:**
- Given một trong chín lớp phủ mở, when nhìn mọi control ở hai theme, then dáng theo bậc ở quyết định 1, không nút gạch chân `outline` hay tô `primary`, không chip viên thuốc, không checkbox/radio dáng hệ điều hành.
- Given Nhập văn bản đi qua bản xem trước tới xác nhận, when chạy e2e `story-12-6-import-preview`, then xanh.
- Given ca canh chuyển đổi, when gỡ lớp `ui-*` khỏi một control của mười một tệp, then đỏ đúng tệp:dòng.

## Implementation Notes

- Mười một tệp chuyển sang `ui-*`; CSS dáng scoped xoá, chỉ giữ bố cục (margin, flex, position).
- `span` đếm `*-chip-clean` bỏ lớp `*-chip`, có rule chữ `ui-sm` riêng; `<label>` bọc `ip-cleanup-tick` là `ui-check--bare`.
- Ca khoá chết `NOT_A_CONTROL` nhận khoá trên control hoặc trên `label` bọc control (`wrapsControl`).
- Đối chứng đỏ đã làm và trả lại: gỡ `ui-*` khỏi nút `ip-act-primary`, bỏ `ip-candidate` khỏi nhãn, xoá `LABEL_EXEMPT` Alignment (đỏ `AlignmentOverlay.vue:210`).
- e2e `story-12-6-import-preview` chưa chạy: cổng 1420 bận.
- `kbd` ⌥W của `ImportPreviewOverlay` cũng thành `ui-kbd` cho khớp bản Bilingual (quyết định 7 bỏ sót); rule `margin-top` của `.ip-cleanup-tick` xoá vì ô nay nằm trong nhãn `ui-check--bare` 24px.
- Ma trận có assert riêng: nhãn thẻ ứng viên mang `ui-check` và đổi `ip-candidate-selected` (`importPreviewOverlayRender`), chip ngôn ngữ mang `ui-chip` (`libraryImportOverlay`), nút xoá chờ mang `ui-btn--danger` (`importPreviewCleanup`); gỡ thật từng chỗ ⇒ đỏ đúng ca, trả lại trùng byte.

## Plan Change Log

## Review Triage Log

### 2026-10-28 — Review pass
- verdicts: 7 findings — high 0, medium 0, low 1, false 5, maybe-false 1
- findings:
  - `[maybe-false]` `[defer]` AC2 chưa chứng minh: e2e `story-12-6-import-preview` chưa chạy — cần lượt e2e khi cổng 1420 trống; nếu thật thì medium; ghi vào `deferred` và mục nợ `Chủ: Epic 12`.
  - `[false]` `[reject]` Ô nhiều dòng của `LibraryImportOverlay` thiếu `ui-field--lg` nên nhỏ hơn một bậc — `textarea.ui-field` (độ đặc hiệu 0,1,1) đặt `height: auto` thắng `.ui-field--lg` (0,1,0) chỉ khai `height`, nên `--lg` không đổi gì trên textarea.
  - `[false]` `[reject]` Ô mẫu luật/mẫu Chương mất font mono — DESIGN §Điều khiển: "Chữ luôn `ui-md`" cho mọi control; `ui-field` theo đúng luật đó, giữ mono là khai lại chữ riêng trong scoped.
  - `[low]` `[reject]` Nhãn `ui-check--bare` của `ip-cleanup-tick` mất `flex: none` có thể co — `.ip-cleanup-rule-body` là `flex: 1` nền 0 nên chỉ co khi riêng ô tick và hai nút vượt bề ngang hàng, không gặp ở cỡ lớp phủ; sửa cần thêm lớp + rule mới.
  - `[false]` `[reject]` Ca canh không kiểm nhãn bọc ô tick — ca một đọc tổ tiên `label` của chính input (`ImportPreviewOverlay.vue:1356-1365`), nên nhãn có `ui-check` và bọc input là điều kiện để ca xanh.
  - `[false]` `[reject]` Đường kiểm chính có thể không nhận khoá `NOT_A_CONTROL` trên nhãn — `notAControl` đọc lớp của mọi nút, không riêng control; đối chứng gỡ `ip-candidate` khỏi nhãn đỏ cả ca khoá chết lẫn ca dáng (`.ip-candidate-selected` khai `border-color`).
  - `[false]` `[reject]` Không test nào canh dáng chọn của chip lọc Chương — `ui-chip[aria-pressed='true']` trong `controls.css` tô dáng chọn, và `aria-pressed` vẫn bind; test hiện có đọc `aria-pressed`, nên dáng chọn không thể lệch trạng thái.

## Verification

**Commands:**
- `npx vitest run tests/frontend/controls-adoption.test.ts tests/frontend/controls-css.test.ts tests/frontend/dialogAccessibleName.test.ts $(grep -lE "ImportPreviewOverlay|BilingualImportPreviewOverlay|LibraryImportOverlay|ReviewerImportOverlay|ExportOverlay|AlignmentOverlay|ChapterMergeOverlay|ChapterSplitOverlay|AttributionOverlay|ChapterOrigin|SourceLangChips" tests/frontend/*.ts)` -- expected: mã thoát 0, không dòng Errors.
- `npm run check:tokens && npm run check:i18n` -- expected: xanh.
- `npx vue-tsc --noEmit` -- expected: không lỗi.
- `npm run test:e2e -- --spec e2e/specs/story-12-6-import-preview.e2e.mjs` (ngoài sandbox, cổng 1420 trống) -- expected: xanh; cổng bận thì ghi lại, không suy ra.

**Manual checks:**
- Ice: ảnh app thật chín lớp phủ và hai component con, sáng và tối, cạnh `he-dieu-khien.html` (mục nợ `Chủ: Epic 12`).

## Auto Run Result

- Thay đổi: chín lớp phủ Nhập/Xuất và hai component con chuyển sang lớp `ui-*` của story 27, xoá CSS dáng scoped; thẻ ứng viên mã hoá giữ dáng mockup màn, radio vẽ bằng `ui-check`; ca canh chuyển đổi đọc thêm mười một tệp.
- Tệp: `src/ImportPreviewOverlay.vue`, `src/BilingualImportPreviewOverlay.vue` (nút, ô, chip `ui-chip`, `ui-kbd`, thẻ ứng viên `ui-check`); `src/LibraryImportOverlay.vue` (form `lg`, radio `ui-check`, bỏ rule `.field input`); `src/ReviewerImportOverlay.vue`, `src/ExportOverlay.vue`, `src/ChapterMergeOverlay.vue`, `src/ChapterSplitOverlay.vue`, `src/AttributionOverlay.vue`, `src/AlignmentOverlay.vue` (Đóng Chìm, chân `lg`, nút Chính đen); `src/ChapterOrigin.vue` (`ui-field`); `src/modes/SourceLangChips.vue` (`ui-chip`); `tests/frontend/controls-adoption.test.ts` (mười một tệp, `NOT_A_CONTROL` hai thẻ, `LABEL_EXEMPT`/`SHAPE_EXCEPTIONS` Alignment, ca khoá chết nhận nhãn bọc control); `importPreviewOverlayRender`, `libraryImportOverlay`, `importPreviewCleanup` (assert ma trận); `deferred-work.md` (mục `Chủ: Epic 12`).
- Review: 7 phát hiện; 0 patch, 1 defer (maybe-false, medium nếu thật: e2e `story-12-6` chưa chạy), 6 bác (1 low: ô tick mất `flex: none`, chỉ co khi hàng hẹp phi thực tế; 5 false: `--lg` không tác dụng trên textarea, mono trái "Chữ luôn `ui-md`", nhãn ô tick đã được ca đọc, đường kiểm chính nhận khoá trên nhãn, dáng chọn chip bám `aria-pressed`).
- Follow-up review: false (không patch nào).
- Kiểm: 40 tệp vitest của Verification xanh 750/750, mã thoát 0, không dòng Errors; sau khi thêm assert ma trận, bốn tệp liên quan xanh 113/113; `check:tokens`, `check:i18n`, `vue-tsc --noEmit` mã thoát 0; đối chứng gỡ thật đỏ đúng ca rồi trả lại trùng byte (`ui-btn--danger`; `ui-check` của thẻ; `ui-chip` của chip ngôn ngữ; lớp `ip-candidate` ⇒ ca khoá chết; mọi `ui-*` của nút huỷ chân ⇒ `ImportPreviewOverlay.vue:1706`). e2e `story-12-6-import-preview` chưa chạy: cổng 1420 bận.
- Rủi ro còn: chưa ai nhìn app thật mười một bề mặt (cả `ChapterOrigin`, `SourceLangChips` trong Library); ô mẫu luật/mẫu Chương nay dùng chữ `ui-md` thay mono theo DESIGN, Ice có thể muốn giữ mono cho regex; chip "cần xem lại" mất tô `error` — cả ba trong mục nợ `Chủ: Epic 12` hoặc lượt nhìn của nó.
