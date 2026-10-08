---
title: 'Chọn cách xuất hình ảnh'
type: 'feature'
ticket: '5'
created: '2026-10-08'
status: 'ready-for-dev'
baseline_revision: ''
route: 'full'
route_source: 'auto'
risk: 'low'
review: ''
review_source: ''
lenses_ran: []
review_loop_iteration: 0
context: []
---

<frozen-after-approval reason="human-owned intent — do not modify unless human renegotiates">

## Intent

**Problem:** Hiện màn hình xuất không cho chọn cách xuất ảnh (FR130). Định dạng duy nhất đang có là `.docx` hai cột (8.3), và nó không ghi ảnh nào: đọc mã thấy `core/export` không truy vấn bảng `asset`.

**Approach:** Thêm vào lớp phủ Xuất một ô chọn "theo link gốc"/"theo file ảnh". Lựa chọn này nhớ theo từng lần xuất, giống phạm vi. Rust quét phạm vi để đếm ảnh và liệt kê những ảnh thiếu `source_url`, và màn hình hiện danh sách đó ngay khi người dùng chọn. `.docx` hai cột bắt đầu ghi ảnh theo chế độ đã chọn.

**Decisions (Ice, 2026-10-08):**
- (A) 8.5 dựng phần ghi ảnh ngay trong `.docx` hai cột của 8.3: mỗi ảnh là một hàng đặt ở vị trí neo (`after_segment_id`).
  - Chế độ link: hàng ảnh mang `Hyperlink` tới `source_url`.
  - Ảnh thiếu URL vắng mặt trong tệp, nhưng đã được liệt kê trước lúc xuất.
- (A2) "Theo file ảnh" nghĩa là chép ảnh từ `assets/` ra thư mục `<tên tệp xuất>-anh/` đặt cạnh tệp `.docx`; hàng ảnh ghi đường dẫn tương đối tới tệp ảnh đó. Ảnh không nhúng vào `.docx`.
- Giữ plan dài (2.518 token) thay vì tách.

## Boundaries & Constraints

**Always:**
- Tuân AD-43: quét phạm vi trước khi xuất, không có đường nào bỏ ảnh trong im lặng.
- Tuân AD-1: quét và đếm ở Rust.
- Tuân AD-9: ảnh ở chế độ file lấy từ `<.atproj>/assets/<file_name>`.
- Không biết được thì để `Option`, không dùng `0`. Chọn rỗng vẫn trả lỗi `export.scope_empty` như 8.2.
- Phạm vi không có ảnh nào mang `source_url` ⇒ ô "theo link gốc" bị vô hiệu và nói rõ lý do ngay tại chỗ.
- Mọi điều khiển dùng được bằng bàn phím và là phần tử gốc (radio).
- Mã Rust mới vào tệp riêng theo trách nhiệm (quyết định của Ice ở Notes của epic). Nâng sàn quần thể cùng lượt nếu chạm trần.

**Never:**
- Không thêm phụ thuộc. `docx-rs =0.4.22` đã có `Pic` và `Hyperlink`.
- Không thêm quyền vào `capabilities/main.json`.
- Không tải ảnh từ mạng lúc xuất (AD-15, AD-16).
- Không dựng `.md`/text (8.6) hay `.docx` một khối (8.4).
- Không nhúng ảnh vào `.docx`.
- Không ghi vào `project.db`.

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected Output / Behavior | Error Handling |
|----------|--------------|---------------------------|----------------|
| Có ảnh, đủ link | phạm vi có N ảnh, mọi ảnh có `source_url` | tóm tắt: `image_count = N`, danh sách thiếu link rỗng, cả hai chế độ dùng được | — |
| Thiếu một phần | N ảnh, K ảnh `source_url IS NULL` | danh sách K ảnh (Chương, thứ tự ảnh trong Chương, alt nếu có), hiện dưới ô "theo link gốc" | — |
| Không ảnh nào có link | N > 0, K = N | ô "theo link gốc" bị vô hiệu, kèm lý do; chế độ đang là link thì quay về file | — |
| Không có ảnh | N = 0 | nêu "0 ảnh trong phạm vi"; ô link bị vô hiệu như hàng trên | — |
| Đổi phạm vi | chế độ link, phạm vi mới có K > 0 | danh sách cập nhật cùng lượt với phép đếm, không lẫn với kết quả cũ | thứ tự `sequence` như 8.2 |
| Xuất theo link | N ảnh, K thiếu URL | `.docx` có N−K hàng link ở đúng vị trí neo; kết quả báo số ảnh đã ghi và K ảnh bỏ vì thiếu link | — |
| Xuất theo file | N ảnh | `.docx` cùng thư mục `<stem>-anh/` chứa N tệp ảnh; mỗi hàng ảnh trỏ tới tệp tương ứng; tên thư mục theo đúng stem cuối cùng của `.docx` (kể cả hậu tố ` (n)`) | — |
| Tệp ảnh mất trên đĩa | chế độ file, hàng `asset` có nhưng tệp trong `assets/` không còn | — | lỗi có tên, nêu Chương; không bỏ qua trong im lặng |
| Đích đã tồn tại | `<stem>-anh/` hoặc tệp ảnh trùng tên | không ghi đè thứ gì có sẵn | như `write_new_file` của 8.3 |

</frozen-after-approval>

## Code Map

- `src-tauri/src/core/export/scope.rs` -- `ScopeCounts`, `count_scope`, `resolve_chapter_ids`. Phép quét ảnh đi kèm tóm tắt phạm vi để giữ một lời gọi IPC cho mỗi lần đổi phạm vi.
- `src-tauri/src/commands/segment/chapter_read.rs:260` -- `select_chapter_assets` (`pub(super)`, là câu `SELECT … FROM asset` duy nhất). Nâng lên `pub(crate)` để tái dùng, đừng chép truy vấn.
- `src-tauri/src/core/segment/image.rs:148` -- `resolve_chapter_images` ánh xạ ảnh → `after_segment_id`, `alt_text`, `caption_text`. Mỗi chỗ gọi dùng một bộ cờ riêng: lưới `(true,false)`, đọc `(false,true)`.
- `src-tauri/src/core/store/schema.rs:1247` -- `ASSET_DDL`. `source_url` NULL với ảnh nhúng `.docx` và `data:`; không có `retired_at`.
- `src-tauri/src/core/export/{table_rows,docx_table,new_file}.rs`, `commands/export.rs:64` -- đường xuất hai cột: `load_chapter_tables`, `write_two_column_docx`, `export_docx_two_column(open, scope, folder)`. Hiện không có ảnh.
- `src-tauri/tests/ipc_argument_contract.rs` -- tên tham số lệnh Rust phải khớp khoá `invoke()`.
- `src/exportState.ts` -- `currentExportScope`, `refreshCounts`, `setExportFormat`, `runExport` (gọi cứng lệnh hai cột), `resetExport`. Chế độ ảnh theo khuôn của `format`.
- `src/ExportOverlay.vue:189-218` -- fieldset định dạng rồi đến thư mục. Ô chọn ảnh đặt sau định dạng, theo mockup `ux-auratranslate/mockups/export-images-attribution.html`.
- `src/config/export.ts` -- adapter không ném lỗi.
- `src/i18n/vi.json:332-360` -- `export.*`.
- `src-tauri/src/core/i18n/mod.rs:899` -- khoá lỗi `export.*`.
- Tests that move:
  - `src-tauri/tests/export_contract.rs`, `src-tauri/tests/export_docx_contract.rs`. Fixture hiện chưa có `asset`; mẫu `insert_asset`/`set_role` ở `tests/segment_image_contract.rs:60,83`.
  - `tests/frontend/exportScope.test.ts`, `tests/frontend/exportDocx.test.ts`.
- Sàn quần thể:
  - `src-tauri/src` hiện 119 tệp `.rs`. Khoảng 20 tệp `*_boundary.rs` dùng sàn 97, nên trần là 121.
  - Frontend hiện 132 tệp, trần 136.
  - Thêm tệp TS thì xét lại `check-tokens.mjs`, `check-commands.mjs`, `check-panel-refs.mjs`.

## Tasks & Acceptance

**Execution:**
- [ ] `src-tauri/src/core/export/images.rs` (mới) -- quét ảnh của phạm vi, đánh dấu ảnh thiếu `source_url`; `scope.rs`/`commands/export.rs` trả kết quả quét cùng tóm tắt -- AD-43 quét trước.
- [ ] `core/export/table_rows.rs`, `docx_table.rs`, `new_file.rs` -- hàng ảnh ở vị trí neo (link: `Hyperlink`; file: đường dẫn tương đối), thư mục `<stem>-anh/` không ghi đè -- quyết định (A)/(A2).
- [ ] `commands/export.rs` -- `export_docx_two_column` nhận `image_mode`; `ExportedFile` báo số ảnh đã ghi và số ảnh bỏ vì thiếu link -- lựa chọn đi theo từng lần xuất, không im lặng.
- [ ] `src-tauri/tests/export_docx_contract.rs` -- ca link, file, thiếu URL, tệp ảnh mất, thư mục trùng tên.
- [ ] `src-tauri/tests/export_contract.rs` -- fixture có ảnh, có/không `source_url`, ảnh gắn câu về hưu hoặc bị cắt bỏ; phủ hết ma trận I/O.
- [ ] `src/config/export.ts`, `src/exportState.ts`, `src/ExportOverlay.vue`, `src/i18n/vi.json`, `core/i18n/mod.rs` -- ô chọn, danh sách thiếu link, ô bị vô hiệu kèm lý do, quay về file khi link không dùng được.
- [ ] `tests/frontend/exportImages.test.ts` (mới) -- chọn, bàn phím, danh sách, ô bị vô hiệu, thứ tự khi đổi phạm vi, giá trị gửi đi.

**Acceptance Criteria:**
- Given màn hình xuất, when mở, then có hai lựa chọn "theo link gốc" và "theo file ảnh", mặc định là theo file ảnh.
- Given đã chọn một chế độ, when đóng rồi mở lại lớp phủ, then chế độ được giữ như phạm vi; lần xuất gửi đúng chế độ đang chọn.
- Given phạm vi có ảnh thiếu link, when chọn "theo link gốc", then danh sách ảnh thiếu link hiện ngay ở ô đó, không nằm trong tài liệu hướng dẫn.
- Given chế độ link, when xuất, then mỗi ảnh có URL thành một hàng link tới URL ảnh của bài gốc, đúng vị trí giữa các câu như trong Chương.
- Given chế độ file, when xuất, then ảnh lấy từ `.atproj/assets/` nằm trong thư mục cạnh tệp `.docx`, mở được bằng trình xem ảnh, và mỗi hàng ảnh nêu đúng tệp của nó.
- Given xuất xong, when xem kết quả, then thấy số ảnh đã xuất và số ảnh bỏ vì thiếu link.

## Design Notes

- Hàng ảnh mang cùng một tham chiếu ở cả hai cột, để mỗi cột tự đủ khi được bôi đen dán đi. Alt và caption vẫn là các hàng câu riêng của chúng (AD-42); chúng không gộp vào hàng ảnh.
- Thứ tự ghi: chọn stem cuối cùng trước, rồi tạo thư mục ảnh, rồi đến `.docx`. Có lỗi giữa chừng thì báo lỗi có tên, không trả `Ok`.
- `ExportRow` có thể đổi thành enum (câu | ảnh). Bốn đường đọc của 8.3 phải giữ nguyên hành vi với phạm vi không ảnh.

## Implementation Notes

## Plan Change Log

## Review Triage Log

## Verification

**Commands:**
- `npm run build` rồi `cargo test --test export_contract` và `cargo test --test export_docx_contract` -- kỳ vọng: ma trận I/O xanh; gỡ phép lọc `source_url IS NULL` khỏi phép quét ⇒ ca "thiếu một phần" đỏ.
- `npx vitest run tests/frontend/exportImages.test.ts` -- kỳ vọng: xanh.
- `npm run test:story 8-5` -- kỳ vọng: xanh.
- Vì có thêm lệnh IPC hoặc thêm tham số: chạy `cargo test --test ipc_argument_contract` và `npm run check:commands`.
