---
title: 'Màn nhiều cột không cuộn lồng'
type: 'feature'
ticket: '5'
created: '2026-11-18'
status: 'built'
baseline_revision: '2e719bbb500795a2ca612dd6f07dbad4df422f69'
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

**Problem:** Hai màn nhiều cột còn vùng cuộn dọc nằm trong vùng cuộn dọc khác, trái R4 (DESIGN.md §Lớp phủ — Một vùng cuộn): ở Xuất, danh sách Chương `.ex-chapters` (cao tối đa 140px, 5 hàng × 28) cuộn bên trong cột cấu hình `.ex-config` cũng cuộn; ở Nguồn từ điển, `.attr-license-text` (`pre`, `max-height: 50vh`) cuộn bên trong `.attr-screen` cuộn cả màn.

**Approach:** Bỏ cuộn riêng của `.attr-license-text` (văn bản giấy phép nằm trọn trong vùng cuộn của màn). Năm màn vẫn tràn cửa sổ, lề `panel-inline`, và vẫn ở `OVERLAY_MULTI_COLUMN_EXEMPT` của Kiểm L (miễn R4 ở cổng nguồn theo Design Notes của 17.1); e2e là phép canh lồng. Đóng mục nợ `.attr-license-text` (`Chủ: Epic 17`) trong `deferred-work.md`.

**Decision (Ice 2026-11-18):** màn Xuất giữ như hiện tại — không sắp ba cột, không bỏ cuộn của `.ex-chapters`. Danh sách Chương 5 hàng cuộn trong `.ex-config` là ngoại lệ có tên duy nhất của R4 ở màn này; e2e khoá đúng một vùng lồng là `.ex-chapters` và chân dính thấy được khi cuộn đáy. Bác phương án A (xem trước còn 84px ở cửa sổ 800) và B (cuộn qua N × 28px mới tới Định dạng).

**Decision (Ice 2026-11-18):** Nguồn từ điển không dựng đầu/chân dính; chỉ hết cuộn lồng.

**Decision (Ice 2026-11-18):** giữ trọn plan dù vượt 1.600 token.

## Boundaries & Constraints

**Always:** Giữ nội dung, nhánh trạng thái, Esc, `trapTab`, tiêu điểm trả về nút mở, thứ tự Tab của từng màn. `pre.attr-license-text` giữ `tabindex="0"` (đổi là đổi thứ tự Tab) và thêm `overflow-wrap: anywhere` để dòng dài không tràn ngang.

**Never:** Sửa `ExportOverlay.vue`; đổi Kiểm L hay ba danh sách của nó; sửa `e2e/support/overlayFrame.mjs`; chạm ba màn kia (Xem trước nhập, Xem trước nhập song ngữ, Prompt inspector) — chỉ đo và ghi nợ nếu lồng; đổi chiều cao lớp phủ.

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected Output / Behavior | Error Handling |
|---|---|---|---|
| Xuất, phạm vi cả Tác phẩm | không có danh sách Chương | 2 vùng cuộn kề nhau, `nested` 0 | — |
| Xuất, phạm vi Nhiều Chương | 40 Chương | `nested` 1 và vùng lồng duy nhất là `.ex-chapters`; nút Xuất thấy được khi `.ex-config` và `.ex-preview` cuộn đáy | — |
| Nguồn từ điển, giấy phép dài | văn bản 200 dòng đang mở | một vùng cuộn (`.attr-screen`), `nested` 0 | — |
| Giấy phép lỗi / rỗng | `error` hoặc `text === ''` | như hiện tại | `attr-error` |

</frozen-after-approval>

## Code Map

- `src/ExportOverlay.vue:242-523` (chỉ đọc) -- `.ex-body` lưới `343px minmax(0,1fr)`: `.ex-config` (`overflow-y: auto`, :687) chứa fieldset Phạm vi với `.ex-chapters` (`max-height: calc(unit*20)`, `overflow-y: auto`, :763) hiện khi `listVisible`; `.ex-preview` (`overflow-y: auto`, :695); `.ex-page` rộng tối đa `calc(unit*117)` = 819px (:969). `.ex-footer` (:526) ngoài `.ex-body` nên đã dính.
- `src/AttributionOverlay.vue:186-381,383-600` -- `.attr-screen` `position:absolute; overflow:auto` là vùng cuộn duy nhất; `.attr-license-text` :562 mang `max-height: 50vh; overflow: auto`.
- `scripts/check-tokens.mjs:2427` `OVERLAY_MULTI_COLUMN_EXEMPT` -- giữ nguyên năm tên.
- `e2e/support/overlayFrame.mjs` -- `measureOverlayFrame(selector)` → `{width, scrollers, nested}`, `isInViewport`, `resizeLogical`; dùng lại.
- `e2e/specs/story-12-17-export-content-options.e2e.mjs` -- đường mở Xuất (`[data-export-open]`, `.ex-panel`); `e2e/specs/attribution-focus.e2e.mjs` -- đường mở Nguồn từ điển (`openWorkspaceWithWork`, `[data-attribution-open]` trong panel Lookup, cần tầng full). Văn bản giấy phép dài đặt qua `import()` module trạng thái (`attributionLicenseView`), vì CI không có từ điển thật.
- `tests/frontend/attributionOverlayActions.test.ts`, `tests/frontend/exportDocx.test.ts:122` -- giữ xanh.
- `src/ImportPreviewOverlay.vue:2333` `.ip-config` `position: sticky; overflow-y: auto` -- nghi lồng trong vùng cuộn màn; ngoài phạm vi sửa.
- `_bmad-output/initiative-auratranslate/deferred-work.md:14265` -- mục nợ `.attr-license-text`, đóng bằng `→ ✅ ĐÃ ĐÓNG`.

## Tasks & Acceptance

**Execution:**
- [ ] `src/AttributionOverlay.vue` -- `.attr-license-text` bỏ `max-height`/`overflow`, thêm `overflow-wrap: anywhere`.
- [ ] `e2e/specs/story-17-5-man-nhieu-cot.e2e.mjs` -- ở 1280×800: Xuất phạm vi cả Tác phẩm `nested === 0`; Xuất phạm vi Nhiều Chương (đủ Chương để danh sách tràn) `nested === 1` và phần tử lồng duy nhất mang `ex-chapters`, cuộn `.ex-config` và `.ex-preview` tới đáy thì nút Xuất `isInViewport`; Nguồn từ điển với giấy phép 200 dòng `nested === 0`. Đo thêm ba màn còn lại nếu mở được bằng đường của spec có sẵn; lồng thì ghi nợ.
- [ ] `tests/frontend/attributionOverlayActions.test.ts` -- giữ xanh (không đổi tên lớp).
- [ ] `_bmad-output/initiative-auratranslate/deferred-work.md` -- đóng mục `.attr-license-text`; mục mới (≤5 dòng, `Chủ: Epic 17`) cho màn nào trong ba màn còn lại đo ra `nested > 0`.

**Acceptance Criteria:**
- Given cửa sổ 1280×800, Xuất mở với phạm vi Nhiều Chương và danh sách dài hơn khung, when đo `.ex-panel`, then vùng lồng duy nhất là `.ex-chapters` và nút Xuất nằm trong khung nhìn khi mọi cột cuộn đáy.
- Given Nguồn từ điển mở với văn bản giấy phép 200 dòng, when đo `.attr-screen`, then `nested` 0.
- Given đối chứng gỡ thật (`.attr-license-text` lấy lại `max-height: 50vh; overflow: auto`), when chạy spec, then ca Nguồn từ điển đỏ `nested` 1 và ca Xuất vẫn xanh.
- Given `npm run check:tokens` và `npm run check:debt-owner`, then xanh.

## Verification

**Commands:**
- `npm run check:tokens` và `npm run check:debt-owner` -- exit 0.
- `npx vitest run tests/frontend/attributionOverlayActions.test.ts tests/frontend/exportDocx.test.ts` -- xanh.
- `npm run build` rồi `npm run test:e2e -- --spec e2e/specs/story-17-5-man-nhieu-cot.e2e.mjs` -- xanh, ngoài sandbox, bản `--features wdio`.
- `npm run check:lint` và `npx vue-tsc --noEmit` -- xanh.

## Implementation Notes

- Chỉ `src/AttributionOverlay.vue` đổi mã sản phẩm; Xuất không đổi theo quyết định của Ice.
- Spec đặt văn bản giấy phép qua `attributionLicenseView.__v_raw.value` vì ref xuất ra là `readonly` và CI không có từ điển thật; Vue đổi nội bộ thì ca đỏ ồn (TypeError), không xanh oan.
- 40 Chương dựng bằng `preview_import_encoding_from_text` + `confirm_import_with_encoding` với mẫu literal `第`; `create_work_from_text` chỉ ra một Chương.
- Ca Nguồn từ điển chạy đầu tệp: chạy sau ca Xuất thì `openWorkspaceWithWork` không nạp được lưới (chưa rõ vì sao); ca đóng lớp phủ trong `finally`.
- Đối chứng gỡ thật: trả `max-height: 50vh; overflow: auto` cho `.attr-license-text` ⇒ ca Nguồn từ điển đỏ `nested` 1, hai ca Xuất xanh; khôi phục.
- Ca vitest mới cho nhánh giấy phép rỗng (`.attr-partial`) vì hàng ma trận "Giấy phép lỗi / rỗng" chưa có ca phủ nhánh rỗng.
- `check:debt-owner` đỏ từ baseline (sàn `PLANNED_TICKET_FLOOR` 225 dưới 80% của 283); sàn nâng lên 242 trong commit riêng.

## Plan Change Log

## Review Triage Log

Lượt 1 (quick) — high 0 · medium 0 · low 2 · false 4 · maybe-false 0.

- low · reject — `__v_raw` dựa nội bộ Vue: hỏng thì ca đỏ ồn, không xanh oan; sửa phải thêm setter test vào mã sản phẩm. Phần "không ghi đối chứng" là sửa plan, loại.
- false — "thiếu đối chứng cho ca Xuất": AC đối chứng chỉ đòi ca Nguồn từ điển đỏ, ca Xuất xanh.
- false — `check-debt-owner.mjs` ngoài diff: cố ý, commit riêng; cây sạch lúc bắt đầu story.
- false — sửa mock trong `attributionOverlayActions.test.ts` đổi ca khác: chỉ ca mới dùng `ok: ''` (dòng 371); ca phủ hàng ma trận theo bước kiểm ma trận.
- low · reject — ba màn còn lại không đo, lý do mỏng: plan để tuỳ chọn, mục nợ `Chủ: Epic 17` đã ghi.
- false — mục nợ thứ hai không trỏ DESIGN.md: dòng "Cần" nêu DESIGN.md §Lớp phủ — Một vùng cuộn.
