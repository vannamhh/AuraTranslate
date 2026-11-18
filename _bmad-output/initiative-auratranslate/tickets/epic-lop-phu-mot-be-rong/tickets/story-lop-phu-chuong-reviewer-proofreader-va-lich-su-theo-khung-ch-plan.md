---
title: 'Lớp phủ Chương, Reviewer, Proofreader và lịch sử theo khung chung'
type: 'refactor'
ticket: '4'
created: '2026-11-18'
status: 'built'
baseline_revision: 'd6e145568e81dca0da3b7dc2ce87cb3e404648d1'
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

**Problem:** Bảy lớp phủ (Gộp Chương, Tách Chương, Nhập, Nhập bản reviewer, Nối tay, Lịch sử segment, Chỗ đã bỏ qua) còn tự chép khung với bốn bề rộng (560 · 721 · 959 · 1099px); khung cuộn cả khối trong khi `.cs-list` (50vh), `.al-list` (55vh) và `.pfmem-list` tự cuộn bên trong, và cả bảy nằm trong danh sách nợ của Kiểm L.

**Approach:** Theo khuôn 17.1/17.3: mỗi tệp mang lớp khung (`ui-overlay-scrim` · `ui-overlay` · `__head` · `__body` · `__foot`) cạnh lớp scoped riêng; đầu là `div.<tiền tố>-top.ui-overlay__head` bọc `header`, chân là khối liền cuối màn (dòng lỗi/trạng thái/gợi ý sát nút cùng khối nút); danh sách bỏ cuộn riêng; ô URL nhiều dòng giữ cuộn với miễn trừ `aura-allow-scroll`. Xoá bảy dòng nợ.

**Decision (Ice 2026-11-18, phương án A):** Lịch sử segment dùng một thân cuộn: thông báo, khối xác nhận và cả hai cột trong `__body`; mũi tên lên/xuống đưa hàng nhắm vào khung nhìn bằng `scrollIntoView({ block: 'nearest' })`. Bác B (hai cột tự cuộn dưới một dải không cuộn: dải xác nhận khôi phục ≈ 210px + 26px mỗi dòng nháp ước lượng từ token làm hai cột còn ≈ 220px ở 800×600 và tràn khung khi nháp dài).

**Decision (Ice 2026-11-18):** giữ trọn phạm vi bảy tệp dù plan vượt 1.600 token.

## Boundaries & Constraints

**Always:** DOM order và thứ tự Tab giữ nguyên: chân chỉ nhận khối liền cuối, không dời khối qua khối khác. Giữ lớp scoped và `data-*` để selector test không đổi; giữ z-index và miễn trừ của từng scrim. Nội dung, nhánh trạng thái, Esc, `trapTab`, tiêu điểm trả về nút mở, phím không đổi. Nhánh không có nút (đang tải, lỗi, rỗng, Nhập reviewer đã xong) không vẽ chân. `.li-panel` giữ `position: relative` cho `.li-dropveil`.

**Never:** Sửa `controls.css`, token, hay luật Kiểm L (chỉ xoá dòng nợ); chuyển lớp phủ của entry 5; rút `focusableWithin`/`trapTab` (AI-11); đổi chiều cao lớp phủ; sửa spec e2e của 17.1–17.3.

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected Output / Behavior | Error Handling |
|---|---|---|---|
| Danh sách dài | Tách Chương 200 câu, Nối tay nhiều hàng | danh sách cao theo nội dung, chỉ `__body` cuộn, chân thấy ở đỉnh và đáy | — |
| Con trỏ Nối tay | mũi tên đổi radio tới hàng ngoài khung nhìn | `__body` cuộn theo (trình duyệt cuộn phần tử nhận tiêu điểm); hàng không nằm dưới `.al-cols` | — |
| Kéo thả tệp vào Nhập | `isDragOver` | `.li-dropveil` phủ cả khung | — |
| Chỗ đã bỏ qua | mở từ `Công cụ ▾` | khung 1120 canh giữa trên nền che đặc, không còn popup 560 lệch phải | — |
| Lỗi tải | Tách Chương `failed`, Nối tay/Nhập reviewer `error` | đầu + thân báo lỗi, không chân | nút đóng ở đầu, Esc |

</frozen-after-approval>

## Code Map

- `src/{ChapterMerge,ChapterSplit,LibraryImport,ReviewerImport,Alignment,SegmentHistory,ProofreadMemory}Overlay.vue` -- scrim `div.*-scrim` > `section.*-panel`; CSS khung riêng (`width`/`max-width`/`max-height:100%`/`overflow:auto`/`padding`/viền trên `*-panel`, khối `*-scrim` + quy tắc mac) bị thay bởi lớp khung; z-index ở lại.
- Khuôn đã chuyển: `src/ConventionReplaceOverlay.vue` (đầu `crv-top`, chân xếp chồng), `src/PromptImportOverlay.vue` (chân `v-if` theo nhánh có nút, dòng gợi ý `flex: 1` trái + nút phải), `src/PromptLibraryOverlay.vue:1175-1190` (`.pl-cols` hai cột tự cuộn, miễn trừ `aura-allow-scroll`).
- Đầu: chỉ `header` cũ trong cả bảy tệp; ghi chú mở đầu ở lại thân.
- Chân: cm `p.cm-error`+`.cm-actions` · cs `p.cs-error`+`.cs-actions` · li `p.error`+`footer.li-foot` (chân xếp cột) · ri nhánh `loaded` (ba `p` trạng thái xác nhận + `.ri-hint` + `.ri-actions`) · al nhánh `loaded` (ba `p` trạng thái + `.al-actions` + `p.al-hint`) · pfmem ba `p.pfmem-note` cuối + `.pfmem-actions` · hist không chân (khối nút cuối nằm trong cột chi tiết).
- Cuộn riêng phải bỏ: `.cs-list` `max-height: 50vh; overflow-y: auto`, `.al-list` `max-height: 55vh; overflow: auto`, `.pfmem-list` `overflow-y: auto` (+ `.pfmem-panel` `max-height: 80vh`, `min-width`, scrim `justify-content: flex-end` không nền). Giữ: `.li-url-textarea` `overflow: auto` + `aura-allow-scroll`; `overflow: hidden` của `.li-url-gutter`, `.cm-name`, `.cs-text`, `.al-sr-only` không phải vùng cuộn.
- `.al-cols` (nhãn cột `aria-hidden`, cùng lưới `minmax(0,1fr) 49px minmax(0,1fr)` với hàng) ở lại thân, `position: sticky; top: 0` có nền; `.al-row` mang `scroll-margin-top` bằng cao `.al-cols`. Không đưa lên đầu: thanh cuộn cổ điển trên Windows làm lệch cột giữa.
- `scripts/check-tokens.mjs:2420-2426` -- xoá bảy dòng `entry 4`; Kiểm L in "nợ 0".
- `src/SegmentHistoryOverlay.vue` `@keydown.up/down` → `moveHistoryAim` chỉ đổi trạng thái, không dời tiêu điểm, không `scrollIntoView` (hàng nhắm có thể ra ngoài khung nhìn ngay hôm nay).
- Đường mở e2e: Tách Chương `e2e/specs/story-12-4-work-page-chapter-list.e2e.mjs:24-26,255`; Nối tay `story-12-16-reviewer-import-hand-join.e2e.mjs:24` (`[data-tools-item="export.alignment.open"]`); Chỗ đã bỏ qua `[data-tools-item="ai.proofread.memory.open"]` (`titlebar-tools.e2e.mjs:11`). Helper `e2e/support/overlayFrame.mjs` (`resizeLogical`, `measureOverlayFrame`); khuôn spec `e2e/specs/story-17-3-glossary-tm-quy-uoc.e2e.mjs`.
- Vitest liên quan: `tests/frontend/{alignment,proofreadMemory,proofreadMemoryContent,libraryImportOverlay,libraryImportEntryMatrix,reviewerImport,dialogAccessibleName,controls-adoption}.test.ts` -- không ca nào đọc `overflow`/bề rộng; `controls-adoption` cấm `height`/`border` trên phần tử mang lớp `ui-*` (lý do bọc `header` trong `*-top`).


## Tasks & Acceptance

**Execution:**
- [ ] sáu tệp `src/{ChapterMerge,ChapterSplit,LibraryImport,ReviewerImport,Alignment,ProofreadMemory}Overlay.vue` -- thêm lớp khung theo Code Map, bọc phần giữa trong `div.<tiền tố>-scroll.ui-overlay__body`, xoá CSS khung và cuộn riêng; `.al-cols` sticky -- R1/R4.
- [ ] `src/SegmentHistoryOverlay.vue` -- khung chung, không chân; `.hist-two` trong `__body`; sau `moveHistoryAim` đưa `li.hist-item` đang nhắm vào khung nhìn (`block: 'nearest'`) -- R1/R4.
- [ ] `scripts/check-tokens.mjs` -- xoá bảy dòng nợ.
- [ ] `e2e/specs/story-17-4-chuong-reviewer-proofreader.e2e.mjs` -- spec theo `verify` của ticket; Nối tay đo thêm ở khung ép thấp: chân thấy ở đỉnh và đáy thân, hàng cuối nhận tiêu điểm thì nằm trong khung nhìn và dưới `.al-cols`.
- [ ] `tests/frontend/alignment.test.ts`, `tests/frontend/proofreadMemory.test.ts`, `tests/frontend/segmentHistoryOverlay.test.ts` -- nút hành động nằm trong `.ui-overlay__foot`; nhánh lỗi không có chân. Ca vitest Lịch sử segment: mũi tên xuống gọi `scrollIntoView` trên hàng vừa nhắm.

**Acceptance Criteria:**
- Given cửa sổ 1280px trên WKWebView, when mở Tách Chương, Nối tay và Chỗ đã bỏ qua, then mỗi khung rộng 1120 ±1 và `measureOverlayFrame` trả `nested` 0.
- Given cổng chạy trên cây, when bảy tệp đã dùng khung, then `npm run check:tokens` xanh và Kiểm L in "nợ 0".
- Given đối chứng gỡ thật (`.cs-list` lấy lại `max-height: 50vh; overflow-y: auto`), when chạy Kiểm L và ca Tách Chương, then cả hai đỏ vì vùng cuộn lồng.

## Implementation Notes

- Mũi tên ở Nối tay không dời tiêu điểm (`onKeydown` chặn mặc định rồi dispatch `export.alignment.next/prev`), nên trình duyệt không tự cuộn như Code Map giả định; watcher `alignmentCursor` sẵn có nay gọi `scrollIntoView({ block: 'nearest' })` trên `.al-row-current`.
- `.al-row` mang `scroll-margin-top: calc(21px + panel-block + unit)`: 21px là cao dòng `ui-sm` (token leading không có đơn vị), `unit` là đệm trên của `__body` — trên WKWebView nhãn cột dính cách mép vùng cuộn 7px, thiếu nó hàng đầu lọt 7px dưới nhãn.
- `.al-cols` dính cần `z-index: 1` có miễn trừ: `.al-row-label` được định vị nên vẽ đè lên nhãn.
- Chân xếp cột (`flex-direction: column`) ở sáu tệp có chân vì mỗi chân là dòng lỗi/trạng thái trên khối nút; Gộp, Nhập và Chỗ đã bỏ qua luôn có chân, Tách, Nhập reviewer và Nối tay chỉ ở nhánh `loaded`.
- e2e Nối tay chặn `fetch` tới `alignment_open` (khuôn `review-mouse`) để có 80 mục chưa xử lý.
- Đối chứng gỡ thật: trả `max-height: 50vh; overflow-y: auto` cho `.cs-list` ⇒ Kiểm L đỏ "nested scroll region", ca Tách Chương của spec 17-4 đỏ `nested` 1; gỡ `v-if` của chân Tách Chương ⇒ ca vitest nhánh `failed` đỏ; khôi phục rồi `touch`.
- Bảng I/O: hàng lỗi tải có ca vitest cho Tách Chương (`libraryChapterBulkWrites`), Nhập reviewer (`reviewerImport`), Nối tay (`alignment`); hàng kéo thả kiểm `.li-dropveil` là con trực tiếp của `.li-panel.ui-overlay` (jsdom không có bố cục).

## Plan Change Log

## Review Triage Log

Lượt 1 (quick) — high 0 · medium 1 · low 4 · false 2 · maybe-false 0.

- medium · patch — chân của Gộp, Tách, Nhập, Nhập reviewer, Nối tay đè `padding: 0; gap: 0` lên đệm `6px 13px` của khung mà không con nào có đệm riêng (`.cm-actions`, `.cs-actions`, `.ri-actions`, `.al-actions`, `.li-foot` chỉ là hàng flex), nên nút và dòng lỗi dính viền khung, dòng lỗi dính hàng nút; giữ đệm khung, khe `unit` như `.pfmem-bottom`.
- low · patch — lý do `aura-allow-scroll` mới của `.li-url-textarea` viết tiếng Việt, trái luật chú thích tiếng Anh của AGENTS.md; dịch sang tiếng Anh.
- low · patch — `scroll-margin-top` của `.al-row` chép tay 21px của cao dòng `ui-sm`; tính từ `--font-ui-sm` × `--leading-ui-sm`.
- low · reject — e2e không đo Gộp, Nhập, Nhập reviewer, Lịch sử; AC chỉ đòi ba màn, thêm spec không phải sửa trực tiếp.
- low · reject — `measureOverlayFrame` không đo lề trong của chân; cùng lý do, thêm phép đo là thêm nhánh cho helper chung.
- false — "đầu Tách Chương khác cao 35 của 13.9": ở baseline `d6e14556` `.cs-head` đã không khai `height: 35px` (tiêu đề `read-title` ghép tên Chương); thay đổi chỉ đổi `margin-bottom` thành đệm dọc trong đầu khung.
- false — "`scrollIntoView` chạy cả khi con trỏ đổi bằng chuột": R4 đòi vùng cuộn theo con trỏ bất kể nguồn; `block: 'nearest'` không cuộn khi hàng đã trong khung nhìn, nên cú bấm vào hàng đang thấy không làm gì.

## Verification

**Commands:**
- `npm run check:tokens` -- exit 0, Kiểm L "nợ 0".
- `npx vitest run tests/frontend/alignment.test.ts tests/frontend/proofreadMemory.test.ts tests/frontend/proofreadMemoryContent.test.ts tests/frontend/libraryImportOverlay.test.ts tests/frontend/libraryImportEntryMatrix.test.ts tests/frontend/reviewerImport.test.ts tests/frontend/segmentHistoryOverlay.test.ts tests/frontend/dialogAccessibleName.test.ts tests/frontend/controls-adoption.test.ts` -- mã thoát 0, không dòng Errors.
- `npm run test:e2e -- --spec e2e/specs/story-17-4-chuong-reviewer-proofreader.e2e.mjs` rồi `--spec e2e/specs/story-12-4-work-page-chapter-list.e2e.mjs` và `--spec e2e/specs/story-12-16-reviewer-import-hand-join.e2e.mjs` -- xanh, ngoài sandbox, bản `--features wdio`.
- `npm run check:lint` và `npx vue-tsc --noEmit` -- xanh.
