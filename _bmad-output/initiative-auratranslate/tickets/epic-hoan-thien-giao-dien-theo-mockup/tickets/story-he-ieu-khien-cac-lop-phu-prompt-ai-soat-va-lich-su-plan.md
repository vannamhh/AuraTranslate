---
title: 'Hệ điều khiển: các lớp phủ Prompt, AI, Soát và Lịch sử'
type: 'refactor'
ticket: '29'
created: '2026-10-28'
status: 'done'
baseline_revision: '448f09cbc2744e953db2530d29e389d208e6dcab'
route: 'full'
route_source: 'auto'
risk: 'medium'
review: 'quick'
review_source: 'pinned'
lenses_ran: ['quick']
review_loop_iteration: 0
followup_review_recommended: false
context:
  - '{project-root}/_bmad-output/initiative-auratranslate/tickets/epic-hoan-thien-giao-dien-theo-mockup/tickets/story-he-ieu-khien-cac-lop-phu-glossary-va-tm-con-lai-plan.md'
  - '{project-root}/src/AGENTS.md'
warnings: ['oversized']
deferred:
  - summary: >-
      Chưa ai thấy chữ của gương `.pl-ta-mirror` nằm đúng trên chữ của ô soạn khi gõ và cuộn trong webview thật (AC2).
    evidence: |-
      e2e story-12-14-prompt-library chưa chạy: cổng 1420 bận (tauri dev của Ice). Ca vitest chỉ phủ khai báo (padding, font, viền khớp textarea.ui-field), độ đặc hiệu của luật trong suốt và việc chép scrollTop, không phủ glyph thật. Lượt e2e (nightly hoặc chạy tay khi cổng trống) phân xử; đã có trong mục nợ Chủ: Epic 12 của ticket này.
    location: >-
      src/PromptLibraryOverlay.vue .pl-ta-mirror
    severity: medium (unverified)
---

<intent-contract>

## Intent

**Problem:** Bảy bề mặt Prompt/AI/Soát/Lịch sử còn tự vẽ 63 control bằng lớp scoped: nút Đóng và nút hành động kiểu chữ gạch chân `outline`, nút chính viền `primary`, ô nhập/ô nhiều dòng viền `outline` nền `background`, checkbox/radio dáng hệ điều hành, tab gạch chân `primary` (DESIGN.md §Điều khiển).

**Approach:** Chuyển `PromptLibraryOverlay`, `PromptImportOverlay`, `PromptTemplatePicker`, `AiPromptInspectorOverlay`, `ProofreadMemoryOverlay`, `ProofreadScopeOverlay`, `SegmentHistoryOverlay` sang lớp `ui-*` của story 27, thêm `ui-tab` cho tab con, giữ gương ô soạn prompt trùng khớp ô `ui-field`, xoá CSS dáng scoped, nối bảy tệp vào ca canh chuyển đổi.

## Decisions (build-auto, 2026-10-28)

1. Bậc theo quyết định 1 của story 28: ô nhập và nút trên cùng hàng của một form trong lớp phủ là `lg` (form tạo bộ, form đổi tên, cặp chân Xác nhận/Huỷ, cặp xác nhận khôi phục của Lịch sử, chân `PromptTemplatePicker`), cặp gửi thêm `ui-btn--pair`; thanh hành động thân lớp phủ và nút Đóng là `md`; nút trong hàng dày (Xem trước của mẫu, `aip-efix`) là `sm`.
2. Tab con là lớp mới `ui-tab` trong `controls.css`, theo DESIGN §Ba cấp điều khiển: chữ `ui-md` `on-surface-variant`, nét đáy 2px trong suốt; `[aria-selected='true']` chữ `on-surface` 600, nét đáy `on-surface`; rê chuột đổi chữ sang `on-surface`; focus đổi nét đáy sang `primary`, không vòng ngoài; có trong khối `forced-colors`. DESIGN "Một nguồn" đòi mọi dáng của mục là lớp `ui-*`, và chưa có lớp nào cho tab con.
3. Hàng chọn của danh sách (`pl-row`, `pfmem-row`) và thẻ phạm vi Soát (`pfscope-cell`) giữ dáng của mockup màn mình (`prompt-library.html` `.it`, `proofreader.html` `.sc`): hàng/thẻ hai dòng, đánh dấu chọn bằng nét trái. DESIGN §Điều khiển không có loại hàng hay thẻ; `ui-menu-item` cao cố định 28px `nowrap` sẽ cắt chữ xuống dòng của `pfmem-row` và bỏ nét chọn. Ca canh miễn trừ đích danh ba lớp này. Hai nút hành động trong cột danh sách (`pl-row-new`, `pl-row-import`) không phải hàng chọn: thành `ui-btn ui-btn--quiet`, bỏ lớp `pl-row`.
4. Thanh phạm vi của Thư viện prompt là `ui-seg` (mockup `prompt-library.html` vẽ `.seg`), không phải chip.
5. Hai nút xác nhận khôi phục của Lịch sử, hai nút Quên/Xoá hết của Bộ nhớ Soát giữ không phân cấp như hiện nay (cùng là nút Phụ); không thêm Chính hay Nguy hiểm.

## Boundaries & Constraints

**Always:**
- Ràng buộc "Always" của story 27/28: token AD-34, một thuộc tính mỗi trạng thái, Kiểm H, scoped chỉ còn bố cục, `@click` một `dispatch`, `aria-*`/phím/`data-*` giữ nguyên.
- Lớp móc mà test/e2e đọc ở lại trong template, không mang CSS dáng; thứ tự DOM của nút/ô giữ nguyên (bẫy focus đọc nút đầu/cuối).
- Gương `.pl-ta-mirror` khai cùng `padding`, `font-family`, `font-size`, `line-height`, độ dày viền với `textarea.ui-field`.
- Mỗi ngoại lệ trong ca canh có tên và lý do; ngoại lệ không còn khớp control nào thì ca đỏ.
- Mục nợ mới `Chủ: Epic 12` cho lượt nhìn app thật.

**Never:**
- Không sửa tệp của story 30–33, bốn menu và `GlossaryManageOverlay` của 27, sáu tệp của 28; `controls.css` chỉ thêm `ui-tab` và mục `forced-colors` của nó.
- Không thêm dependency, token màu, sửa `check-tokens.mjs`, sửa DESIGN.md.
- Không đổi lớp `ui-*` hiện có.

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected Output / Behavior | Error Handling |
|----------|--------------|---------------------------|----------------|
| Ô soạn prompt đang bận | `promptSetBusy`, `textarea.ui-field:disabled` | Chữ của ô vẫn trong suốt, chỉ gương hiện chữ (không hai lớp chữ chồng) | — |
| Cuộn ô soạn | `scrollTop` 120 | Gương cuộn theo, lệch ≤ dung sai của e2e `story-12-14` | — |
| Tab đang chọn được focus | Tab vào `aip-tab` có `aria-selected=true` | Nét đáy đổi `on-surface` → `primary` | — |
| Radio phạm vi ẩn | Nhãn `ui-seg__item` bọc radio `pl-sr-only` | Mục chọn tô qua `:has(input:checked)`; Tab vào thấy focus | — |
| Lớp miễn trừ chết | Xoá mọi `pl-row` khỏi template | Ca canh đỏ: ngoại lệ không khớp control | — |

</intent-contract>

## Code Map

Số dòng ở mốc `448f09cb`. Mẫu chuyển: `src/TmManageOverlay.vue` và `src/GlossaryQueueOverlay.vue` (giữ lớp cũ cạnh lớp mới làm móc, xoá rule dáng, radio ẩn bằng `*-sr-only` sao `.gq-sr-only` :794 và nhãn `position: relative` như `.gq-chip` :708). Mọi rule xoá dưới đây là rule mà ca canh sẽ báo; đọc lại khối quanh mỗi chỗ xoá (⚠️ story 26 và 27 đều để treo selector trong danh sách dấu phẩy, ví dụ `.aip-close, .aip-efix` :768 và `.hist-close, .hist-act` :410).

- `src/tokens/controls.css` -- thêm `ui-tab` (quyết định 2) sau `.ui-seg--icons` (:567), hover trước focus; thêm `.ui-tab:focus-visible` vào khối `forced-colors` (:610); thêm "tab" vào comment đầu tệp. Mẫu dáng: `.lookup-tab` ở `src/panels/LookupPanel.vue:1053-1070` (không sửa tệp đó, thuộc 33).
- `src/PromptLibraryOverlay.vue` (template 625-928, style 930-1451) -- đóng :635 → `ui-btn ui-btn--quiet`; `fieldset.pl-seg` :642 giữ bố cục, bọc các nhãn trong `div.ui-seg`, nhãn :644 → `pl-pill ui-seg__item` (bỏ bind `pl-pill-on`), radio :645 thêm `pl-sr-only` (lớp mới, sao `gq-sr-only`), `.pl-pill { position: relative }`; `pl-row` :689/:705 giữ nguyên (quyết định 3; `pl-row-on` là móc `promptLibraryOverlayRender.test.ts:297`); `pl-radio-label` :725/:776/:780 → thêm `ui-check`; :736 xuất nhiều → `ui-btn`; :753/:756/:761 → `ui-btn ui-btn--quiet pl-row-new|pl-row-import` (bỏ `pl-row`, giữ `.pl-row-form` vì test đọc 13 lần); `pl-input` :794/:838 → `pl-input ui-field ui-field--lg ui-field--medium`; `pl-textarea` :802 → `pl-textarea ui-field ui-field--full`; :814 → `ui-btn ui-btn--lg ui-btn--primary ui-btn--pair` (giữ `pl-act-primary`), :822 → `ui-btn ui-btn--lg ui-btn--pair`; :843 đổi tên → `ui-btn ui-btn--lg`; :863 → `pl-textarea pl-ta-input ui-field ui-field--full`; :890-905 → `ui-btn`; :910 → thêm `'ui-btn--danger': deletePending` cùng `:class` với `pl-act-danger`. CSS: xoá `.pl-close` 984-1000, phần dáng của `.pl-row-new, .pl-row-import` 1080-1084 (xoá cả rule), `.pl-input` 1186-1199, `.pl-textarea` 1201-1215, `.pl-pill` dáng 1294-1317 (giữ bố cục), `.pl-act*` 1413-1442; bỏ `font-*` khỏi `.pl-radio-label` 1377. Gương: `.pl-ta-mirror` 1222-1238 đổi `padding` sang `6px calc(var(--space-unit) * 2)` (kèm `aura-allow-spacing` có lý do: khớp `textarea.ui-field`), font sang `--face-ui-md-wrap`/`--font-ui-md-wrap`/`--leading-ui-md-wrap`; `.pl-ta-wrap` 1217 nền `surface-sunken` + `border-radius: var(--radius-default)`; nền/chữ trong suốt của ô đặt ở `.pl-ta-wrap .pl-ta-input` (độ đặc hiệu (0,3,0) thắng `.ui-field:disabled`), không dùng `:disabled`.
- `src/PromptImportOverlay.vue` -- đóng :122 quiet; `pi-radio-label` :168/:177/:211/:220 → thêm `ui-check`, bỏ `font-*` ở 430; :241 → `ui-btn ui-btn--lg ui-btn--primary ui-btn--pair` (giữ `pi-act pi-act-primary`, móc `promptImportOverlayRender.test.ts:194-197`), :249 → `ui-btn ui-btn--lg ui-btn--pair` (giữ `pi-act`). Xoá 306-321, 459-483.
- `src/PromptTemplatePicker.vue` -- `pt-radio` :71/:75/:83 → thêm `ui-check`, bỏ `font-*` ở 200; :96 → `ui-btn ui-btn--sm`; :112 → `ui-btn ui-btn--lg`, :117 → `ui-btn ui-btn--lg ui-btn--primary ui-btn--pair`, :122 → `ui-btn ui-btn--lg ui-btn--pair`. Xoá 229-252.
- `src/AiPromptInspectorOverlay.vue` (template 270-725, style 727-1217) -- ba `aip-close` :285/:304/:397 → `ui-btn ui-btn--quiet`; tab :339/:350 → `aip-tab ui-tab` (bỏ bind `aip-tab-on`); `aip-efix` :599/:608/:633/:648 → `ui-btn ui-btn--sm ui-btn--quiet`; `aip-btn` :439/:706/:715 → `ui-btn`, :697 → `ui-btn ui-btn--primary`. Xoá 768-791 (cả hai selector), 832-848, 1185-1210.
- `src/ProofreadMemoryOverlay.vue` -- đóng :142 quiet; `pfmem-row` :157 giữ (quyết định 3); :188/:197 → `ui-btn`. Xoá 257-273, 346-366.
- `src/ProofreadScopeOverlay.vue` -- đóng :68 quiet (móc `proofreadScopeOverlay.test.ts:142`); `pfscope-cell` :73/:85/:97 giữ (quyết định 3). Xoá 157-173.
- `src/SegmentHistoryOverlay.vue` -- đóng :199 quiet; :238/:241 → `ui-btn ui-btn--lg ui-btn--pair`; :336/:344 → `ui-btn`; giữ `hist-act` (móc `segmentHistoryOverlay.test.ts:135`, e2e `story-12-19` :76/:84). Xoá 410-427.
- `tests/frontend/controls-adoption.test.ts` -- `FILES` thêm bảy tệp; `SHAPE_EXCEPTIONS` thêm `src/PromptLibraryOverlay.vue .pl-sr-only`; bảng mới `NOT_A_CONTROL` khoá `src/PromptLibraryOverlay.vue .pl-row`, `src/ProofreadMemoryOverlay.vue .pfmem-row`, `src/ProofreadScopeOverlay.vue .pfscope-cell` với lý do (quyết định 3): control mang lớp đó bỏ qua ở cả hai ca, và lớp của nó không vào tập lớp mà ca hai soát; một ca mới đỏ khi một khoá `NOT_A_CONTROL` không khớp control nào trong tệp của nó.
- `tests/frontend/controls-css.test.ts` -- ca `ui-tab` (chọn ⇒ nét đáy `on-surface` + 600; focus đổi nét đáy sang `primary`, đứng sau hover); thêm `.ui-tab:focus-visible` vào danh sách ca `forced-colors` (:58); ca gương: so khai báo `padding`, `font-family`, `font-size`, `line-height` của `.pl-ta-mirror` (đọc từ `PromptLibraryOverlay.vue`) với `textarea.ui-field`, và cả hai viền 1px.

## Tasks & Acceptance

**Execution:**
- [ ] `src/tokens/controls.css`, `tests/frontend/controls-css.test.ts` -- `ui-tab` + `forced-colors` + ca `ui-tab` + ca gương -- quyết định 2, ràng buộc gương.
- [ ] Bảy tệp `.vue` ở Code Map -- đổi lớp, xoá CSS dáng, gương khớp `ui-field` -- như Code Map.
- [ ] `tests/frontend/controls-adoption.test.ts` -- bảy tệp, `pl-sr-only`, `NOT_A_CONTROL` + ca khoá chết -- ca canh nối dài. Đối chứng thật: gỡ `ui-btn` khỏi một nút `pl-act` của `PromptLibraryOverlay` ⇒ đỏ đúng tệp:dòng, rồi trả lại; đổi `padding` của `.pl-ta-mirror` ⇒ ca gương đỏ, rồi trả lại.
- [ ] `tests/frontend/` -- sửa test hiện có chỉ khi móc đổi, ghi lý do.
- [ ] `_bmad-output/initiative-auratranslate/deferred-work.md` -- cuối tệp, tiêu đề `## Deferred from: hệ điều khiển các lớp phủ Prompt, AI, Soát và Lịch sử (2026-10-28, ticket 29 của Epic 12)`, một mục ≤ 5 dòng `Chủ: Epic 12`: ảnh app thật bảy bề mặt, sáng/tối, cạnh `he-dieu-khien.html`; gương ô soạn khớp chữ khi gõ và cuộn; ba lớp hàng/thẻ giữ dáng mockup màn (quyết định 3) cần Ice xác nhận; e2e `story-12-14-prompt-library` nếu chưa chạy.

**Acceptance Criteria:**
- Given một trong bảy bề mặt mở, when nhìn mọi control ở hai theme, then dáng theo bậc ở quyết định 1, không nút nào gạch chân `outline` hay viền `primary`, không checkbox/radio dáng hệ điều hành.
- Given Thư viện prompt mở một bộ, when gõ và cuộn ô soạn, then chữ của gương nằm đúng trên chữ của ô (e2e `story-12-14-prompt-library` xanh).
- Given bàn phím, when Tab qua `AiPromptInspectorOverlay` và `PromptLibraryOverlay`, then mỗi control có dấu focus theo bảng §Trạng thái, tab con đổi nét đáy sang `primary`.
- Given ca canh chuyển đổi, when gỡ lớp `ui-*` khỏi một control của bảy tệp, then nó đỏ đúng tệp:dòng.

## Implementation Notes

- `ui-tab` thêm vào `controls.css` và khối `forced-colors`; `aip-tab-on` bỏ khỏi template (test không đọc).
- Ca canh có bảng `NOT_A_CONTROL` (`pl-row`, `pfmem-row`, `pfscope-cell`) và ca khoá chết; `SHAPE_EXCEPTIONS` thêm `pl-sr-only`.
- Gương `.pl-ta-mirror` khớp `textarea.ui-field` (padding, font `-wrap`); ca trong `controls-css.test.ts` đọc cả hai.
- Đối chứng: gỡ `ui-btn` khỏi nút Dùng bộ ⇒ đỏ `PromptLibraryOverlay.vue:893`; đổi padding gương ⇒ ca gương đỏ; cả hai đã trả lại.
- e2e `story-12-14-prompt-library` chưa chạy: cổng 1420 bận (`tauri dev` của Ice). Hàng 1 và 2 của ma trận có ca vitest riêng: `controls-css.test.ts` (luật trong suốt của ô soạn có độ đặc hiệu cao hơn `.ui-field:disabled`, không dùng `:disabled`), `promptLibraryOverlayRender.test.ts` (cuộn ô gõ ⇒ gương cuộn theo). Đối chứng thật: hạ selector về `.pl-ta-input` ⇒ đỏ ca trong suốt; xoá dòng chép `scrollTop` ⇒ đỏ ca cuộn; bỏ lớp `pfscope-cell` ⇒ đỏ ca khoá chết `NOT_A_CONTROL`; cả ba đã trả lại, `cmp` trùng bản trước.

## Plan Change Log

## Review Triage Log

### 2026-10-28 — Review pass
- verdicts: 5 findings — high 0, medium 0, low 3, false 1, maybe-false 1
- findings:
  - `[maybe-false]` `[defer]` AC2 chưa chứng minh: e2e gương ô soạn chưa chạy — cần lượt e2e `story-12-14-prompt-library` khi cổng 1420 trống; nếu thật thì medium; ghi vào `deferred` và mục nợ `Chủ: Epic 12`.
  - `[low]` `[reject]` Ô soạn khoá giữ nền `surface-sunken` của `.pl-ta-wrap` trong khi `ui-field:disabled` bỏ nền — chỉ hiện thoáng qua khi `promptSetBusy` lúc lưu; sửa cần `:has(.pl-ta-input:disabled)` mà ca canh báo là "styles a state", tức thêm miễn trừ.
  - `[false]` `[reject]` `ui-tab` đệm đáy 0 làm nét gạch dính nhãn — đúng như `.lookup-tab` (`LookupPanel.vue:1053`, `padding: 0`), mẫu tab con DESIGN §Ba cấp điều khiển mô tả; quyết định 2 dựng theo mẫu đó.
  - `[low]` `[reject]` Implementation Notes có dòng nhật ký đối chứng — sửa là sửa plan của chính lượt dựng này.
  - `[low]` `[reject]` Ô Tasks chưa tích khi đã `in-review` — sửa là sửa plan của chính lượt dựng này.

## Verification

**Commands:**
- `npx vitest run tests/frontend/controls-adoption.test.ts tests/frontend/controls-css.test.ts tests/frontend/promptLibraryOverlayRender.test.ts tests/frontend/promptImportOverlayRender.test.ts tests/frontend/promptTemplatePicker.test.ts tests/frontend/promptSetExportMany.test.ts tests/frontend/aiPromptInspector.test.ts tests/frontend/aiPromptInspectorScreen.test.ts tests/frontend/proofreadMemory.test.ts tests/frontend/proofreadMemoryContent.test.ts tests/frontend/proofreadScopeOverlay.test.ts tests/frontend/segmentHistoryOverlay.test.ts tests/frontend/dialogAccessibleName.test.ts` -- expected: mã thoát 0, không dòng Errors.
- `npm run check:tokens && npm run check:i18n` -- expected: xanh.
- `npx vue-tsc --noEmit` -- expected: không lỗi.
- `npm run test:e2e -- --spec e2e/specs/story-12-14-prompt-library.e2e.mjs` (ngoài sandbox, cổng 1420 trống) -- expected: xanh; cổng bận thì ghi lại, không suy ra.

**Manual checks:**
- Ice: ảnh app thật bảy bề mặt, sáng và tối, cạnh `he-dieu-khien.html` (mục nợ `Chủ: Epic 12`).

## Auto Run Result

- Thay đổi: bảy bề mặt Prompt/AI/Soát/Lịch sử chuyển sang lớp `ui-*` của story 27, xoá CSS dáng scoped; `controls.css` có `ui-tab` cho tab con; gương ô soạn prompt khớp `textarea.ui-field`; ca canh chuyển đổi đọc thêm bảy tệp, miễn trừ đích danh ba lớp hàng/thẻ.
- Tệp: `src/PromptLibraryOverlay.vue` (`ui-seg` phạm vi, `ui-field`, `ui-btn`, gương khớp, ô soạn trong suốt cả khi khoá); `src/PromptImportOverlay.vue`, `src/PromptTemplatePicker.vue`, `src/AiPromptInspectorOverlay.vue` (`ui-tab`), `src/ProofreadMemoryOverlay.vue`, `src/ProofreadScopeOverlay.vue`, `src/SegmentHistoryOverlay.vue` (lớp `ui-*`, xoá rule dáng); `src/tokens/controls.css` (`ui-tab`, `forced-colors`); `tests/frontend/controls-adoption.test.ts` (bảy tệp, `pl-sr-only`, `NOT_A_CONTROL`, ca khoá chết); `tests/frontend/controls-css.test.ts` (ca `ui-tab`, ca gương, ca ô soạn trong suốt); `tests/frontend/promptLibraryOverlayRender.test.ts` (ca cuộn gương); `deferred-work.md` (mục `Chủ: Epic 12`).
- Review: 5 phát hiện; 0 patch, 1 defer (maybe-false, medium nếu thật: e2e gương chưa chạy), 4 bác (1 low: nền trũng khi ô khoá thoáng qua lúc lưu, sửa cần miễn trừ trạng thái; 1 false: `ui-tab` đệm 0 đúng mẫu `.lookup-tab`; 2 low: sửa là sửa plan này).
- Follow-up review: false (không patch nào).
- Kiểm: 13 tệp vitest của Verification xanh 266/266, mã thoát 0, không dòng Errors; sau khi thêm hai ca, ba tệp `controls-css`, `controls-adoption`, `promptLibraryOverlayRender` xanh 81/81; `check:tokens`, `check:i18n`, `vue-tsc --noEmit` mã thoát 0; đối chứng gỡ thật đỏ đúng ca (gỡ `ui-btn` ⇒ `PromptLibraryOverlay.vue:893`; padding gương; selector trong suốt; chép `scrollTop`; lớp `pfscope-cell`). e2e `story-12-14-prompt-library` chưa chạy: cổng 1420 bận.
- Rủi ro còn: chưa ai nhìn app thật bảy bề mặt; gương chưa đo glyph trong webview; ba lớp hàng/thẻ giữ dáng mockup màn (quyết định 3) chờ Ice xác nhận — cả ba trong mục nợ `Chủ: Epic 12`.
