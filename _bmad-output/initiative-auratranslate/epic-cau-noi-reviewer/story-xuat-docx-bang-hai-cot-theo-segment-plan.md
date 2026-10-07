---
title: 'Xuất .docx bảng hai cột theo segment'
type: 'feature'
ticket: '3'
created: '2026-10-07'
status: 'in-review'
baseline_revision: 'a524023bae24e506c2f5cffba84282dd179c66da'
route: 'full'
route_source: 'auto'
risk: 'medium'
review: ''
review_source: ''
lenses_ran: []
review_loop_iteration: 0
context: []
---

<frozen-after-approval reason="human-owned intent — do not modify unless human renegotiates">

## Intent

**Problem:** Người dịch chưa gửi được cho reviewer một file mà mỗi câu nằm một hàng (FR87); màn hình xuất của 8.2 mới chọn được phạm vi và thư mục, chưa ghi file nào.

**Approach:** Thêm định dạng `.docx` bảng hai cột (trái nguyên văn, phải bản dịch, mỗi segment một hàng) vào màn hình xuất của 8.2, ghi bằng `docx-rs`. Rust chọn câu qua `core/segment/omit.rs`, đọc cấu trúc đoạn từ cờ kết đoạn đã lưu, và màn hình nói rõ đây là định dạng nhập lại được.

## Boundaries & Constraints

**Always:** AD-1 (ghi file ở Rust); câu bị cắt bỏ và segment về hưu không vào file (`segments_in_translation`); cấu trúc đoạn cột phải đọc từ `is_target_paragraph_end` và ký tự xuống dòng trong `target_text` (AD-46), cột trái từ `is_paragraph_end`; câu chưa dịch vẫn một hàng, ô phải trống; câu chưa xác nhận không bị đánh dấu trong file; không ghi đè file có sẵn; mọi lệnh dùng được bằng bàn phím.

**Never:** `.docx` một khối theo đoạn, `.md`, khối ghi nguồn (8.4-8.7); phép nhập ngược thật và cổng hình dạng AD-38 (8.8-8.9); thêm phụ thuộc hoặc quyền `capabilities/main.json`; viết lại vị từ câu bị cắt bỏ.

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected Output / Behavior | Error Handling |
|----------|--------------|---------------------------|----------------|
| Một Chương | `Chapters[id]`, thư mục hợp lệ | một tệp `.docx`, một bảng hai cột, một hàng mỗi câu thuộc bản dịch, theo `(ord, id)` | — |
| Nhiều Chương / cả Tác phẩm | nhiều Chương | mỗi Chương một tiêu đề đoạn rồi một bảng, theo `(ord, id)` | — |
| Câu chưa dịch | `target_text = ''` | hàng vẫn có, ô phải một đoạn rỗng | — |
| Câu bị cắt bỏ / về hưu | `is_omitted = 1` hoặc `retired_at` | không có hàng | — |
| Câu có nhiều đoạn dịch | `target_text` chứa `\n` | ô phải một đoạn, mỗi `\n` một dấu xuống dòng (Decision 2026-10-07) | — |
| Cờ kết đoạn | `is_paragraph_end` / `is_target_paragraph_end` | đoạn cuối của ô có khoảng cách sau theo cờ của chính cột đó | — |
| Chương không còn câu thuộc bản dịch | mọi câu bị cắt | tiêu đề Chương, không bảng | — |
| Tên tệp trùng | tệp cùng tên có sẵn | ghi tệp mới với hậu tố số, không đè | — |
| Thư mục không ghi được | quyền/đường dẫn hỏng | — | `export.write_failed` |
| Phạm vi rỗng / Chương lạ / chưa mở Tác phẩm | như 8.2 | — | lỗi có tên của 8.2 |

</frozen-after-approval>

## Code Map

- `src-tauri/src/core/export/table_rows.rs` -- mới: `TableRow`, `ChapterTable`, `load_chapter_tables` (đọc `select_chapter_segments`, lọc `segments_in_translation`, tách đoạn).
- `src-tauri/src/core/export/docx_table.rs` -- mới: `write_two_column_docx` bằng `docx-rs`, không panic.
- `src-tauri/src/core/export/new_file.rs` -- mới: tên tệp an toàn và hậu tố không đè.
- `src-tauri/src/commands/export.rs` -- thêm `export_docx_two_column` + vỏ `wire` `(async)`; `lib.rs` đăng ký.
- `src-tauri/src/commands/segment/chapter_read.rs` -- `select_chapter_segments` thành `pub(crate)`.
- `src-tauri/src/core/i18n/mod.rs`, `src/i18n/vi.json` -- `ExportWriteFailed`, khoá `export.format.*`, `export.run.*`, `command.export.run`.
- `src/config/export.ts`, `src/exportState.ts`, `src/ExportOverlay.vue`, `src/exportCommandDeps.ts`, `src/commands/index.ts`, `src/main.ts` -- chọn định dạng, nút xuất, kết quả.
- Mốc cắm: `currentExportScope()`, `exportFolder` (8.2). Tests that move: `src-tauri/tests/export_contract.rs`, `config_invariants.rs`, `tests/frontend/exportScope.test.ts`; sàn quần thể theo số đo thật.
- Không đổi: `core::docx` (bộ đọc), `omit.rs`, `scope.rs`.

## Tasks & Acceptance

**Execution:**
- [x] `core/export/table_rows.rs`, `docx_table.rs`, `new_file.rs` -- mô hình hàng, bộ ghi, tên tệp
- [x] `commands/export.rs`, `lib.rs`, i18n -- lệnh `export_docx_two_column`, lỗi `export.write_failed`
- [x] `tests/export_docx_contract.rs` -- ma trận I/O, đọc ngược qua `core::docx`, đối chứng gỡ
- [x] webview: định dạng, nút xuất, kết quả, i18n, vitest
- [x] `deferred-work.md` -- đóng mục `core/export/mod.rs` rỗng, ghi AD-46 của 8.3, thêm một mục real-app `Chủ: Epic 8`

**Acceptance Criteria:**
- Given phạm vi và thư mục, when xuất, then `.docx` có bảng hai cột, mỗi câu một hàng, trái nguyên văn, phải bản dịch.
- Given câu chưa dịch, when xuất, then hàng còn, ô phải trống.
- Given câu bị cắt bỏ, when xuất, then tệp không chứa văn bản của nó.
- Given tệp đã ghi, when đọc bằng `core::docx`, then ra đúng văn bản và số đoạn mỗi ô như đã ghi.
- Given màn hình xuất, when chọn định dạng này, then thấy câu nói đây là định dạng nhập lại được.
- Given mở bằng Word hoặc LibreOffice, then hiển thị đúng — chỉ người kiểm được, ghi nợ `Chủ: Epic 8`.

## Implementation Notes

- Cột phải đọc `is_target_paragraph_end` và `\n` của `target_text` (AD-46); cột trái đọc `is_paragraph_end`. Ranh giới đoạn thành khoảng cách sau (`w:after` 240 hay 0) của đoạn cuối mỗi ô, vì bảng một-segment-một-hàng không có chỗ nào khác chở cờ. Đối chứng: thay cờ đích bằng cờ nguồn ⇒ `each_column_takes_its_gap_from_its_own_paragraph_end_flag` đỏ; bỏ `segments_in_translation` ⇒ 3 ca đỏ; bỏ `split('\n')` ⇒ ca xuống dòng đỏ. 🔵 2026-10-07: không còn `split('\n')`, xem Decision 1.
- Nhiều Chương: mỗi Chương một đoạn tiêu đề rồi một bảng riêng (hai bảng liền nhau Word gộp làm một); Chương hết câu thuộc bản dịch chỉ còn tiêu đề. Tiêu đề rỗng khi `chapter.title` là NULL vì Rust không mang chữ hiển thị.
- Không có hàng đầu bảng: một hàng đầu không phải segment, và chữ của nó là chữ hiển thị mà Rust không được viết.
- Đọc ngược qua `core::docx`: số hàng, số ô mỗi hàng, số đoạn mỗi ô và thứ tự văn bản khớp với đã ghi. Nhập ngược thật và cổng hình dạng AD-38 là 8.8-8.9.
- Câu `alt`/`caption` của ảnh cũng là segment nên cũng một hàng; chưa có cột/nhãn vai.
- Tệp tên `<tên Tác phẩm>-hai-cot.docx`, `create_new` và hậu tố ` (n)`, không bao giờ ghi đè. Lệnh `(async)` giữ khoá `OpenWorkState` suốt lượt ghi, như `export_scope_summary` giữ khoá lúc đếm.
- `docx-rs` đã ở `Cargo.toml` (AD-38): không phụ thuộc mới. Ba tệp `.rs` mới (cây 119 tệp) không làm sàn `.rs` nào trôi dưới 80% (cần 122), frontend 0 tệp mới nên không sàn nào đổi; `config_invariants.rs` `COMMAND_FILE_CENSUS` của `commands/export.rs` 1/1/1 → 1/2/2.
- Decision (2026-10-07, Ice): `\n` trong một segment ghi thành dấu xuống dòng (`<w:br/>`) trong MỘT đoạn, không tách đoạn. Lý do: Chương một segment có `\n` sẽ ra bảng một hàng có ô nhiều đoạn, đúng hình dạng AD-38 từ chối là bản xuất bản không nhập lại được; sau đổi mọi ô của 8.3 là một đoạn. `core::docx` đọc `w:br`/`w:cr` về `\n` trong ô bảng (trước đó về dấu cách nên không khứ hồi). Đối chứng: ghi lại thành nhiều đoạn ⇒ hai ca xuống dòng đỏ.
- Decision (2026-10-07, Ice): `w:br`/`w:cr` về `\n` chỉ khi đọc ô bảng (`parse_cell`); đoạn thân tài liệu và chú thích vẫn về dấu cách để đường nhập tài liệu của Epic 6 (`core::segment::import` → chuẩn hoá FR125) không đổi. Khoá bằng `a_soft_break_in_a_body_paragraph_reads_as_a_space_not_a_newline`; đối chứng: đoạn thân phát `\n` ⇒ ca đỏ.
- Decision (2026-10-07, Ice): bỏ hàng đầu bảng "Văn bản gốc / Bản dịch" khỏi xem trước Word của 8.2 vì tệp thật không có; xoá token `word-header-fill`, cặp tương phản `word-ink|word-header-fill` và hai khoá i18n `export.preview.source_header`/`target_header`.
- Câu hỏi mở cho Ice: (1) đã chốt, xem Decision thứ hai ở trên. (2) Một bảng mỗi Chương kèm tiêu đề có hợp với cổng hình dạng AD-38 của 8.8 (đang hiểu là một bảng hai cột) hay 8.8 phải đọc nhiều bảng.

## Plan Change Log

## Review Triage Log

## Verification

**Commands:**
- `npm run build` rồi `cargo test --test export_docx_contract --test export_contract` -- xanh
- `npm run test:story 8-3` -- xanh
