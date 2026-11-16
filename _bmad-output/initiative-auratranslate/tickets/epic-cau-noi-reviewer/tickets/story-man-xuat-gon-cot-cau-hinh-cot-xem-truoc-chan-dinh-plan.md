---
title: 'Màn Xuất gọn: cột cấu hình, cột xem trước, chân dính'
type: 'feature'
ticket: '18'
created: '2026-11-16'
status: 'built'
baseline_revision: 'd455075b9b0718e39b746a5cffbc55598044d6ee'
route: 'full'
route_source: 'auto'
risk: 'medium'
review: 'quick'
review_source: 'pinned'
lenses_ran: ['quick']
review_loop_iteration: 0
context:
  - '{project-root}/_bmad-output/initiative-auratranslate/ux-auratranslate/EXPERIENCE.md'
  - '{project-root}/_bmad-output/initiative-auratranslate/ux-auratranslate/mockups/export-gon.html'
---

<frozen-after-approval reason="human-owned intent — do not modify unless human renegotiates">

## Intent

**Problem:** Màn Xuất xếp dọc bảy nhóm lựa chọn với 13 câu mô tả; trên mockup 1280×860 phải cuộn 469 px mới thấy bản xem trước và 899 px mới tới nút Xuất, nên đổi một lựa chọn không thấy ngay tệp sẽ ra sao.

**Approach:** Dựng lại `ExportOverlay.vue` theo EXPERIENCE.md §Màn Xuất (Phương án A, làm mờ): lớp phủ chiếm khung, cột cấu hình 343 px tự cuộn, cột xem trước tự cuộn, chân dính chứa dòng tóm tắt gom mọi cảnh báo cùng thư mục, Huỷ, Xuất. Chỉ đổi bề mặt.

## Boundaries & Constraints

**Always:**
- Giữ FR121 (nhãn *nhập lại được* / *chỉ để đăng* / *không nhập lại được* ngay trên thẻ định dạng; câu chưa xác nhận cảnh báo trước lúc xuất), FR130 (danh sách ảnh thiếu link mở sẵn, mỗi ảnh một dòng, không gấp), FR131 (ghi nguồn mặc định tắt).
- Lựa chọn không áp dụng thì làm mờ tại chỗ (viền đứt, chữ `on-surface-variant`) kèm lý do vài chữ bên phải; không ẩn.
- Mọi thao tác làm được bằng bàn phím; bẫy Tab, Esc, ⌘↵ và trả tiêu điểm giữ nguyên.
- Chỉ token sẵn có, `ui-seg` cho điều khiển đoạn, `ui-check` cho hộp kiểm/nút chọn; thẻ đang chọn viền `on-surface`, không dùng `primary` cho trạng thái chọn. Kích thước theo lưới 7px (Kiểm K).

- Decision (Ice 2026-11-16): bỏ lưới bốn trường ghi nguồn (`data-export-attribution-fields`); khi bật ghi nguồn và Chương đầu phạm vi có trường rỗng, cạnh công tắc ghi một dòng *thiếu: {nhãn các trường rỗng}*, vì bản xem trước bỏ im lặng trường rỗng (`attribution.rs:31`). Không có trường rỗng thì không có dòng đó.

**Never:** Không đổi lệnh Rust, `exportState.ts` ngoài phần hiển thị cần thêm, hình dạng tệp xuất hay tập lựa chọn; không thêm token, không dùng `opacity` trung gian, không `v-html`.

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected Output / Behavior | Error Handling |
|---|---|---|---|
| Hai cột, câu chưa xác nhận | `docx_two_column`, `unconfirmed_count` 5 | Tóm tắt: *N Chương · M câu · 5 câu chưa xác nhận* | — |
| Định dạng để đăng | một khối/.md/.txt, `unconfirmed_translated_count` 3, `untranslated_count` 2 | *… · 3 câu chưa xác nhận, tệp không phân biệt · 2 câu chưa dịch* | — |
| Không nhập lại được | hai cột + Đánh số câu hoặc Đánh dấu | Thẻ hai cột ghi *không nhập lại được* (`error`) và câu nói vì sao, cách gỡ; tóm tắt thêm *không nhập lại được* | — |
| Theo link gốc, ảnh thiếu link | `missing_link_images` 2 | Tóm tắt *2 ảnh thiếu link*; danh sách hai dòng dưới điều khiển đoạn | — |
| Không ảnh nào có link | `exportLinkUsable()` false | *Theo link gốc* mờ, lý do *không ảnh nào có link gốc* | — |
| Ghi nguồn bật, trường rỗng | Chương đầu thiếu `origin_site_name`, `origin_published_at` | Cạnh công tắc: *thiếu: Báo / website, Ngày đăng gốc*; tắt ghi nguồn thì ghi *mặc định tắt* | — |
| Nhiều Chương | phạm vi `chapters`/`work` | Tiêu đề xem trước *Xem trước · {Chương} (Chương đầu của phạm vi)* | — |
| Lỗi IPC / chưa chọn thư mục | lỗi đếm, xem trước, chạy, thư mục | Lỗi hiện qua `tError()` ở vùng tương ứng (`role="alert"`), nút Xuất tắt như cũ | giữ nhánh lỗi hiện có |

</frozen-after-approval>

## Code Map

- `src/ExportOverlay.vue` -- tệp chính (837 dòng, template 208-629, CSS 631-836). Giữ toàn bộ `<script>` logic (bẫy Tab, `runShortcut`, focus trả về, `missingLabel`, `chapterMeta`), viết lại template và CSS. `.ex-panel` hiện `max-width: 721px`.
- `src/exportState.ts` -- vị từ dùng lại nguyên: `exportMarkAvailable`, `exportCurlyQuotesAvailable`, `exportNotReimportable`, `exportAppendixAvailable`, `exportLinkUsable`, `exportFormatIsForPublishing`, `exportFirstScopeChapter`. Không đổi.
- `src/ImportPreviewOverlay.vue:2322-2394` -- tiền lệ cùng khuôn: lưới `343px minmax(0, 1fr)`, chân dính `.ip-footer`. Chép cách làm, không import.
- `src/tokens/controls.css` -- `ui-seg`/`ui-seg__item` (ví dụ `src/SettingsReadingSection.vue:38`), `ui-check`, mẫu viền đứt khi tắt `:116-121`.
- `src/i18n/vi.json:715-761` -- khoá `export.*`. Thêm khoá cho nhãn thẻ, lý do làm mờ, mảnh tóm tắt, *Ghi vào*, tiêu đề xem trước; khoá mô tả cũ không còn dùng thì xoá (`check:i18n` không bắt khoá thừa).
- Giữ tên thuộc tính `data-export-*` khi nghĩa còn đúng để test ít đổi: `-unconfirmed`, `-block-unconfirmed`, `-block-untranslated`, `-not-reimportable`, `-missing-links`, `-link-disabled`, `-mark-disabled`, `-curly-quotes-disabled`, `-appendix-empty`, `-word-preview`, `-text-preview`, `-file-name`, `-run`, `-cancel`, `-translator*`.
- Tests that move: `tests/frontend/{exportScope,exportDocx,exportBlock,exportText,exportImages,exportAttribution,exportContent,readingExportChapter}.test.ts`; e2e `e2e/specs/story-12-17-export-content-options.e2e.mjs` (đọc `[data-export-not-reimportable][role="status"]`, `[data-export-mark-disabled]`, `.ex-panel`).

## Tasks & Acceptance

**Execution:**
- [x] `src/ExportOverlay.vue` -- template ba vùng (đầu · thân lưới 343px|1fr · chân dính) và CSS; trạng thái nạp/lỗi giữ trên thân -- theo §Màn Xuất.
- [x] `src/i18n/vi.json` -- thêm/xoá khoá `export.*` theo template mới.
- [x] `tests/frontend/export*.test.ts`, `readingExportChapter.test.ts` -- sửa ca vỡ do đổi chỗ; mỗi cảnh báo của I/O Matrix có đúng một ca khẳng định nó nằm trong dòng tóm tắt hoặc dưới lựa chọn của nó; mỗi lựa chọn mờ có ca khẳng định nó còn trong DOM, `disabled`, kèm lý do.
- [x] `e2e/specs/story-12-17-export-content-options.e2e.mjs` -- cập nhật selector; thêm phép đo ở cửa sổ 1280×860: đỉnh bản xem trước và đáy nút Xuất nằm trong khung không cuộn.

**Acceptance Criteria:**
- Given cửa sổ 1280×860, when mở màn Xuất, then bản xem trước và nút Xuất hiện mà không cuộn.
- Given bất kỳ lựa chọn nào đổi, when bản xem trước dựng lại, then cột cấu hình giữ vị trí cuộn và chân dính vẫn hiện.
- Given Tuỳ chọn Nội dung, Ảnh, Ghi nguồn, when hiển thị, then không còn câu mô tả bên dưới, chỉ số đếm cạnh tuỳ chọn; chỉ thẻ định dạng đang chọn mở một câu mô tả.

## Implementation Notes

- Nhãn phạm vi "Nhiều Chương đã chọn" thành "Nhiều Chương" (theo mockup) và đệm mục `ui-seg` 7 px: ở cột 343 px ba nhãn cũ tràn và bị `ui-seg` cắt.
- Dòng tóm tắt mang dấu `·` trong chuỗi i18n chứ không ở `::before`, để văn bản `role="status"` đọc được.
- Phép đo e2e liệt kê mọi `.ex-seg` có `scrollWidth > clientWidth`: font thật chỉ có trong webview, Playwright WebKit chạy font dự phòng.
- 🔵 2026-11-16 (Ice xem app thật): bỏ nhãn nhóm Ghi nguồn; hàng tuỳ chọn canh baseline (trước lệch 3,5 px) và lùi 8 px cho thẳng mép với nội dung thẻ; bỏ nút `Đổi thư mục…` cùng lệnh `export.choose_folder`, bấm Xuất mới mở hộp chọn thư mục (Ice chọn thư mục thay vì hộp Lưu thành, Rust giữ nguyên). e2e 12-17 canh baseline, mép trái/phải và tâm ô kiểm (lệch 0,34 px so với tâm chữ thường).
- Lượt dùng thật của Ice (vị trí cuộn khi đổi lựa chọn, tổ hợp dài nhất) ghi nợ `Chủ: Epic 8` trong `deferred-work.md`.

## Plan Change Log

## Review Triage Log

Lượt 1 (quick + đo WebKit của người điều phối): medium 2 · low 3 · false 5 · maybe-false 0.
- medium · patch — lỗi chọn thư mục bị dòng kết quả xuất che: `chooseExportFolder` chỉ `clearRun()` khi `picked`, nên sau lần xuất `done` lỗi hộp thoại không hiện.
- medium · patch — ba nhãn phạm vi tràn: WebKit Playwright (font dự phòng) đo cần 377 px trên 313 px, `ui-seg` cắt "Cả Tác phẩm"; đổi nhãn theo mockup, đệm 7 px, e2e canh `scrollWidth <= clientWidth` bằng font thật.
- low · patch — dấu `·` chỉ ở `::before`, văn bản `role="status"` đọc liền "12 câu 7 câu chưa xác nhận".
- low · patch — mảnh "không nhập lại được" hiện cho cả định dạng để đăng khi bật đánh số; §Màn Xuất và mockup chỉ gắn nó với bảng hai cột.
- low · reject — AC giữ vị trí cuộn cột cấu hình chưa có ca: happy-dom không có hình học, thuộc lượt dùng thật của Ice (nợ `Chủ: Epic 8`).
- low · reject — chân dính cao thêm khi có đường dẫn dài: vài dòng, hai cột vẫn hiện.
- false — `<legend>` flex trong WebKit: đo được cùng hàng, rộng 315 px.
- false — e2e không đặt 1280×860: cửa sổ `main` trong `tauri.conf.json` mặc định 1280×860, spec không đổi cỡ.
- false — hạ `role="alert"` của cảnh báo: §Màn Xuất gom cảnh báo vào dòng tóm tắt; FR121/FR130 chỉ đòi có mặt trước lúc xuất.
- false — thiếu ca lựa chọn mờ: `exportContent.test.ts:141,165,209`, `exportImages.test.ts:197`.
- false — plan trong diff còn `in-progress`: diff dựng trước khi đổi frontmatter.
- medium · patch (phát hiện lúc kiểm bản vá) — phép canh e2e mới đỏ ở điều khiển ảnh: lý do *không ảnh nào có link gốc* chung hàng ép `ui-seg` còn 154 px trên 173 px cần; `ui-seg` giữ bề rộng tự nhiên, lý do co phần còn lại, e2e xanh.

## Design Notes

- Trạng thái chạy (đang xuất, xong + đường dẫn, ảnh đã ghi/bỏ qua) và lỗi thư mục/chạy thay dòng *Ghi vào …* trong chân dính, để kết quả nằm cạnh nút vừa bấm; đặc tả không nói chỗ, đây là chọn mặc định.
- Dòng tóm tắt là một `role="status"`; mảnh cảnh báo giữ `data-export-*` riêng để test đọc từng mảnh.

## Verification

**Commands:**
- `npx vitest run tests/frontend/exportScope.test.ts tests/frontend/exportDocx.test.ts tests/frontend/exportBlock.test.ts tests/frontend/exportText.test.ts tests/frontend/exportImages.test.ts tests/frontend/exportAttribution.test.ts tests/frontend/exportContent.test.ts tests/frontend/readingExportChapter.test.ts` -- xanh, exit 0, không dòng Errors.
- `npm run check:tokens && npm run check:i18n && npm run check:commands && npm run check:lint` -- xanh.
- `npm run test:e2e -- --spec e2e/specs/story-12-17-export-content-options.e2e.mjs` (ngoài sandbox) -- xanh.

**Manual checks:**
- Ice mở màn Xuất trong app thật ở 1280×860; nếu chưa làm trước khi `built`, ghi nợ `Chủ: Epic 8`.
