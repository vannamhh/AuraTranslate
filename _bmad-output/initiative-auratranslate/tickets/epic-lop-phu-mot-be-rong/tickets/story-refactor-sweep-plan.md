---
title: 'Refactor sweep Epic 17'
type: 'refactor'
ticket: '7'
created: '2026-11-18'
status: 'built'
baseline_revision: '1cd69acc37e42fbba4e535b4c53daf199d64d284'
route: 'full'
route_source: 'auto'
risk: 'low'
review: 'quick'
review_source: 'pinned'
lenses_ran: ['quick']
review_loop_iteration: 0
context:
  - '{project-root}/e2e/AGENTS.md'
---

<frozen-after-approval reason="human-owned intent — do not modify unless human renegotiates">

## Intent

**Problem:** Sáu story của Epic 17 đã `built`; Kiểm L in "nợ 0" nhưng `OVERLAY_DEBT` cùng nhánh xử lý nợ vẫn còn, dù luật của epic nói danh sách nợ chết khi rỗng. Ba mục nợ `Chủ: Epic 17` còn mở ngoài lượt dùng thật của Ice: ba màn nhiều cột chưa đo `nested` trên WKWebView, DESIGN.md lệch với ngoại lệ `.ex-chapters`, và 17-6 để lại năm ca `overlay-caret-hold` chưa chạy lại trên Vite mới.

**Approach:** Phạm vi chốt lúc bắt đầu gồm đúng năm mục S1–S5 dưới đây; mỗi mục đóng, hoặc ghi lý do không làm vào sổ nợ. Không nhận việc mới đẩy sang từ story khác.

- S1 — Xoá `OVERLAY_DEBT` và mọi nhánh nợ của Kiểm L (map, nhánh trong `overlayVerdict`, khoá `debt` của `REAL_LISTS`, hai ca tự kiểm về nợ, chữ "nợ N" trong dòng OK).
- S2 — Đo `nested` của Prompt inspector và Xem trước nhập song ngữ trên WKWebView; kỳ vọng 0 (tĩnh: mỗi màn chỉ khai một vùng cuộn).
- S3 — Xem trước nhập: `.ip-config` dính và tự cuộn (max-height do `syncConfigMaxHeight` tính) bên trong vùng cuộn của màn nên lồng theo cấu tạo. Quyết định (Ice 2026-11-18): giữ nguyên mã, thành ngoại lệ có tên; ca e2e khẳng định đúng một vùng lồng và vùng đó là `.ip-config`.
- S4 — Quyết định (Ice 2026-11-18): sweep ghi vào DESIGN.md §Lớp phủ — Một vùng cuộn hai ngoại lệ có tên, `.ex-chapters` của Xuất và `.ip-config` của Xem trước nhập, đánh dấu 🔵 2026-11-18 (Ice), rồi đóng mục nợ.
- S5 — Chạy lại `overlay-caret-hold.e2e.mjs` trên phiên Vite mới; đỏ thì ghi lý do.

## Boundaries & Constraints

**Always:** Đóng mục sổ nợ bằng chữ (`→ ✅ ĐÃ ĐÓNG …`, `→ KHÔNG LÀM …`), không xoá. Đối chứng gỡ thật cho mỗi guard mới. Bộ test đầy đủ một lần trước khi xong (S1 chạm `scripts/check-*`).

**Never:** Đổi nội dung, chức năng, phím của lớp phủ; sửa `controls.css`, khung chung hay helper e2e ngoài điều S2–S3 cần; rút `focusableWithin`/`trapTab`; xử lý mục nợ lượt dùng thật của Ice (`Chủ: Epic 17`, Done when 5); đo mười lớp phủ trên khung chưa có spec (quyết định Ice 2026-11-18: ngoài sweep, Ice nhìn chúng trong lượt dùng thật).

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected Output / Behavior | Error Handling |
|----------|--------------|---------------------------|----------------|
| Lớp phủ mới không khung | `src/XOverlay.vue` không `ui-overlay`, không nằm ở popup/nhiều cột | Kiểm L đỏ "no `ui-overlay`… not in an exemption list" | Ca tự kiểm sẵn có canh |
| Màn nhiều cột kề nhau | Prompt inspector, song ngữ mở với dữ liệu thật | `nested` 0 | Đỏ ⇒ dừng, báo Ice |
| Ngoại lệ có tên | Xem trước nhập với văn bản dán đủ dài để `.ip-config` có max-height | `nested` 1, vùng lồng duy nhất mang lớp `ip-config` | Vùng lồng khác ⇒ dừng, báo Ice |

</frozen-after-approval>

## Code Map

- `scripts/check-tokens.mjs:2416-2640` -- Kiểm L: `OVERLAY_DEBT` (2419, rỗng), `overlayVerdict` nhánh `lists.debt` (2497-2500), `REAL_LISTS` (2550), dòng OK (2588), `L_CASES` hai ca `debt` (2609-2610), `asMap(lists.debt)` (2625). Không ai khác đọc `OVERLAY_DEBT`.
- `e2e/support/overlayFrame.mjs` -- `measureOverlayFrame(selector)` đếm vùng cuộn và `nested` TRONG phần tử được chọn; màn nhiều cột không mang `ui-overlay`, chọn phần tử chứa cả vùng cuộn cha.
- `e2e/specs/story-17-5-man-nhieu-cot.e2e.mjs` -- khuôn đo màn nhiều cột (Nguồn từ điển, Xuất); nơi thêm ca S2/S3.
- `e2e/specs/story-17-1-khung-lop-phu.e2e.mjs` -- đường mở Prompt inspector (`[data-ai-prompt-inspector-open]`, mở ở cỡ cửa sổ mặc định rồi mới đổi cỡ).
- `e2e/specs/story-6-28-bilingual-into-existing.e2e.mjs:40-70` -- đường mở Xem trước nhập song ngữ.
- `e2e/specs/story-12-6-import-preview.e2e.mjs:71` -- đường mở Xem trước nhập bằng văn bản dán.
- `src/AiPromptInspectorOverlay.vue:745` -- `.aip-panel { overflow: auto }`, vùng cuộn duy nhất khai báo.
- `src/BilingualImportPreviewOverlay.vue` -- không khai `overflow-y`; màn cuộn trong vùng cuộn của `LibraryMode`.
- `src/ImportPreviewOverlay.vue:933-960, 1173, 2334-2340` -- `.ip-config` `position: sticky; overflow-y: auto` + `syncConfigMaxHeight` (ResizeObserver chân); có từ d6031830 (6-23). e2e phụ thuộc: `story-6-23-import-preview-footer`, `story-6-24-cleanup-strike-in-place`, `story-12-6-import-preview`.
- `_bmad-output/initiative-auratranslate/ux-auratranslate/DESIGN.md:475` -- §Lớp phủ — Một vùng cuộn: "Ô nhập nhiều dòng tự cuộn là ngoại lệ duy nhất".
- `_bmad-output/initiative-auratranslate/deferred-work.md:14274-14281` -- hai mục nợ của 17.5; `e2e/specs/overlay-caret-hold.e2e.mjs` -- S5.

## Tasks & Acceptance

**Execution:**
- [ ] `scripts/check-tokens.mjs` -- xoá S1 (map, nhánh, khoá, hai ca, chữ "nợ"); thông điệp "not in a debt or exemption list" thành "not in an exemption list" -- danh sách nợ chết khi rỗng (Notes của epic).
- [ ] `e2e/specs/story-17-5-man-nhieu-cot.e2e.mjs` -- thêm ca Prompt inspector, Xem trước nhập song ngữ (S2) và Xem trước nhập (S3) đo `nested` -- đóng mục nợ đo ba màn.
- [ ] `_bmad-output/initiative-auratranslate/ux-auratranslate/DESIGN.md:475` -- thêm câu ngoại lệ có tên `.ex-chapters` và `.ip-config` (S4).
- [ ] `deferred-work.md` -- đóng hai mục 17.5 bằng chữ.

**Acceptance Criteria:**
- Given cây sau S1, when chạy `npm run check:tokens`, then Kiểm L xanh, không còn chữ "nợ", tự kiểm còn 18 ca.
- Given ba màn nhiều cột mở trên WKWebView, when đo, then Prompt inspector và song ngữ `nested` 0, Xem trước nhập `nested` 1 đúng ở `.ip-config`, và DESIGN.md nêu tên cả hai ngoại lệ.
- Given `overlay-caret-hold` chạy trên Vite mới khởi động, then mọi ca xanh hoặc lý do đỏ ghi vào Implementation Notes.

## Implementation Notes

- S1: `OVERLAY_DEBT` và mọi nhánh nợ của Kiểm L đã xoá; tự kiểm còn 18 ca.
- S2/S3: `story-17-5-man-nhieu-cot.e2e.mjs` thêm ba ca. Prompt inspector đo ở `.aip-scrim` (panel tự là vùng cuộn); song ngữ và Xem trước nhập cuộn trong vùng cuộn của host nên spec gắn nhãn tổ tiên cuộn gần nhất rồi đo bằng `measureOverlayFrame`. Song ngữ cần tên Work và nguồn `zh` mới mở được. Kết quả: 0, 0, và 1 đúng ở `ip-config`.
- Đối chứng gỡ `overflow-y` của `.ip-config` làm ca Xem trước nhập đỏ (Expected 1, Received 0); đã khôi phục.
- Đối chứng thêm `max-height` + `overflow-y: auto` vào `.aip-head` và `.bip-head` làm ca Prompt inspector và song ngữ đỏ `nested` Expected 0, Received 1; đã khôi phục và `touch`. Ca song ngữ nay chờ `.bip-act-primary` bật trước khi đo, và nhãn `data-e2e-measure` được gỡ sau mỗi ca.
- S4/S5: DESIGN.md nêu hai ngoại lệ có tên (🔵); `overlay-caret-hold` 6/6 xanh trên Vite mới; hai mục nợ 17.5 đóng bằng chữ. pre-push đầy đủ xanh (1181 s).

## Plan Change Log

## Review Triage Log

Lượt 1 (quick) — high 0 · medium 1 · low 3 · false 1 · maybe-false 0.

- false — "ca Xem trước nhập không buộc vùng lồng là `.ip-config`": `.ip-config` nằm trong phần tử chủ được đo, nên hễ là vùng cuộn thì nó là vùng lồng; `nested` 1 cộng `ip-config` có trong danh sách vùng cuộn nghĩa là vùng lồng duy nhất chính là nó.
- medium · patch — ca song ngữ đo ngay khi `.bip-screen` hiện, chưa chờ dữ liệu nạp xong (6-28 chờ `.bip-act-primary` bật). Thân rỗng cho `nested` 0 dù màn có lồng hay không; cần chờ nạp xong trước khi đo.
- low · patch — `tagHostScroller` không gỡ `data-e2e-measure`; hai màn ở hai phần tử chủ khác nhau thì ca sau đo nhầm phần tử chủ cũ. Gỡ thuộc tính trong `finally`.
- low · patch — ca Prompt inspector và ca song ngữ chưa có đối chứng đỏ; thêm tạm một vùng cuộn trong cho từng màn để thấy ca đỏ `nested`, rồi khôi phục.
- low · patch — câu "không thêm vùng lồng thứ ba" trong DESIGN.md là một luật mới Ice chưa quyết, gán cho Ice; xoá vế đó.
- reject — ô `[ ]` của Execution, Triage Log trống, CI/nightly chưa đọc: sửa plan hoặc thuộc bước trình bày, không phải lỗi của diff.

## Verification

**Commands:**
- `npm run check:tokens` -- expected: Kiểm L OK, 18 ca tự kiểm.
- `npm run test:e2e -- --spec e2e/specs/story-17-5-man-nhieu-cot.e2e.mjs` -- expected: mọi ca xanh (ngoài sandbox).
- `npm run test:e2e -- --spec e2e/specs/overlay-caret-hold.e2e.mjs` -- expected: xanh trên Vite mới.
- Bộ test đầy đủ (`.githooks/pre-push`) -- expected: xanh.
