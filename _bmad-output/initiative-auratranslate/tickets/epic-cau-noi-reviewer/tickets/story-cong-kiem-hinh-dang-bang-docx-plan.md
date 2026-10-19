---
title: 'Cổng kiểm hình dạng bảng .docx'
type: 'feature'
ticket: '8'
created: '2026-10-08'
status: 'built'
baseline_revision: '51c122d30cf870a9c1772a7dafdbc625e942a3d7'
route: 'full'
route_source: 'auto'
risk: 'low'
review: 'quick'
review_source: 'pinned'
lenses_ran: ['quick']
review_loop_iteration: 0
context: []
---

<frozen-after-approval reason="human-owned intent — do not modify unless human renegotiates">

## Intent

**Problem:** Bản `.docx` một khối của 8.4 và bản hai cột của 8.3 cùng đuôi, cùng là bảng hai cột. Nếu đường nhập lại của reviewer (8.9) nhận bản một khối, alignment sẽ ghi đè cả Chương đã xác nhận bằng một khối văn bản mà không báo lỗi (AD-38).

**Approach:** Thêm cổng hình dạng thuần ở Rust trong `core/export/`. Cổng đọc `DocxParsed.tables` của `core::docx::read_docx` và từ chối khi có bảng đúng một hàng mà có ô nhiều hơn một đoạn. Lời từ chối là một khoá i18n có tên, nói đây là bản dành cho đăng bài và không nhập lại được. Hiện chưa có đường nhập lại nào (8.9 chưa xây), nên cổng là bước đầu tiên mà 8.9 bắt buộc đi qua.

## Boundaries & Constraints

**Always:**
- Nhận dạng chỉ dựa trên `tables` (hình dạng). Không đọc tên tệp, metadata, `docProps` hay chuỗi nào trong văn bản.
- Đếm đoạn theo `paragraphs_per_cell`, là số đếm thô (đoạn rỗng và đoạn chỉ có ảnh đều tính). Chỉ một bảng khớp cũng đủ để từ chối cả tệp.
- Cổng là hàm thuần trên byte hoặc `DocxParsed`: không chạm đĩa, không panic, mọi nhánh trả `Result`.
- Mã mới vào tệp `.rs` riêng (epic Notes 2026-10-07). Nếu số tệp vượt ngưỡng 80% thì nâng sàn quần thể cùng lượt.
- Decision (Ice, 2026-10-08): cổng là cách duy nhất có được đầu vào của alignment. `ReviewerDocx::admit(DocxParsed) -> Result<ReviewerDocx, ReimportShapeError>`, trường private, không có đường dựng nào khác (`From`, `Default`, `pub` field, `new` không kiểm). Alignment của 8.9 chỉ nhận `ReviewerDocx`. Phương án hàm kiểm thuần `check_reimport_shape` bị bỏ vì 8.9 chỉ còn trí nhớ để gọi nó.

**Never:**
- Chạm đường tạo Tác phẩm mới từ `.docx` (6.12, `core/segment/import.rs:781`). Đường đó không có alignment hay ghi đè.
- Xây alignment, lệnh IPC nhập lại hay giao diện nhập lại (8.9/8.10).
- Thêm phụ thuộc. Đổi `core::docx` ngoài doc comment sai của `TableShape`.
- Thêm `aura-allow-*` hay miễn trừ để làm xanh cổng.

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected Output / Behavior | Error Handling |
|----------|--------------|---------------------------|----------------|
| Bản một khối 8.4 | tệp xuất thật, Chương nhiều đoạn | từ chối | khoá `err.export.publish_copy_not_reimportable` |
| Bản một khối, nhiều Chương | chỉ một Chương nhiều đoạn | từ chối | như trên |
| Bản hai cột 8.3/8.5 | nhiều hàng; có hàng ảnh | đi qua | — |
| Hai cột, Chương một câu | 1 hàng, `[[1,1]]` | đi qua | — |
| Một khối, Chương một đoạn | 1 hàng, mỗi ô 1 đoạn | đi qua (ca sót đã chấp nhận ở AD-38) | — |
| Không có bảng / bảng 0 hàng | `tables` rỗng hoặc `rows == 0` | đi qua | — |
| Tệp hỏng | không phải zip | không vào cổng | lỗi `err.docx.unreadable` sẵn có |

</frozen-after-approval>

## Code Map

- `src-tauri/src/core/docx/mod.rs:141` -- `read_docx(&[u8]) -> Result<DocxParsed, DocxError>`. Hàm thuần, bị `tests/docx_boundary.rs` canh. `TableShape` ở `:113` có `rows`, `cells_per_row`, `paragraphs_per_cell[hàng][cột]`. Bảng lồng được tách thành `TableShape` anh em ngay sau bảng cha (`absorb_table` `:672`). Doc comment ở `:107-109` và `:131` vẫn ghi "cấp cao nhất, không tính lồng", nhưng đó là sai: sửa hai chỗ đó cho khớp mục 6 của doc đầu module.
- `src-tauri/src/core/export/mod.rs:19-39` -- module con đều private, re-export ở `:28-39`. Tệp mới đặt ở đây và re-export hàm cổng cùng kiểu lỗi.
- `src-tauri/src/core/export/docx_block.rs:23` -- `write_one_block_docx`: mỗi Chương còn câu là một bảng 1×2. Ô rỗng có 1 đoạn rỗng (`:17`). Ghi nguồn và tiêu đề nằm ngoài bảng. Không đổi.
- `src-tauri/src/core/export/docx_table.rs:67` -- bản hai cột: mỗi segment hoặc ảnh một hàng, mỗi ô đúng 1 đoạn, không hàng tiêu đề. Không đổi.
- `src-tauri/src/core/i18n/mod.rs:62` -- macro `message_keys!`. Mẫu `ExportImageFileMissing` ở `:906`. `IpcError::new(code, MessageKey, params, retryable)` ở `:958`. Câu tiếng Việt đặt ở `src/i18n/vi.json`, cạnh `err.export.image_file_missing` (`:194`).
- `src-tauri/tests/export_block_contract.rs` -- `fixture`/`fixture_in` `:86,127`, `Fixture::export` `:101` (gọi `export_docx_one_block`), `parsed` `:115`. `src-tauri/tests/export_docx_contract.rs` có `fixture` `:84`, `export_with` `:327` và `three()` `:331`. Ca `[[1,1]]` ở `:192-199` là bản hai cột một câu. Chép mẫu helper, không import chéo giữa các tệp test.
- Sàn quần thể: `src-tauri/src` hiện có 128 tệp `.rs`, sàn 105 nên chịu được tới 131. Thêm một tệp không cần nâng sàn. Chạy lại các `*_boundary` để xác nhận.
- `deferred-work.md:12953` -- mục hàng ảnh của bản hai cột khi nhập lại, `Chủ: Story 8.9`, không đổi chủ.

## Tasks & Acceptance

**Execution:**
- [x] `src-tauri/src/core/export/reimport_gate.rs` (mới) -- `ReviewerDocx` (trường private, có accessor đọc `&DocxParsed`), `ReviewerDocx::admit`, lỗi `ReimportShapeError::PublishCopy`, và ánh xạ sang `IpcError` có khoá có tên.
- [x] `src-tauri/src/core/export/mod.rs` -- khai module, re-export.
- [x] `src-tauri/src/core/i18n/mod.rs`, `src/i18n/vi.json` -- khoá `err.export.publish_copy_not_reimportable`. Câu phải nói đây là bản dành cho đăng bài, không nhập lại được, và nên dùng bản `.docx` hai cột theo segment.
- [x] `src-tauri/src/core/docx/mod.rs` -- sửa hai doc comment sai về bảng lồng, không đổi mã.
- [x] `src-tauri/tests/reimport_gate_contract.rs` (mới) -- phủ cả ma trận I/O. Hai ca dùng tệp xuất thật qua `export_docx_one_block` và `export_docx_two_column`, rồi đọc lại bằng `read_docx`. Các ca hình dạng còn lại dựng `TableShape` trực tiếp.
- [x] `deferred-work.md` -- thêm một mục `Chủ: Story 8.9`: alignment và lệnh nhập lại nhận `ReviewerDocx`, không nhận `DocxParsed`; kèm ca chứng minh không ghi gì khi bị từ chối.

**Acceptance Criteria:**
- Given một bản `.docx` một khối vừa xuất bằng `export_docx_one_block`, when đưa byte của nó qua `read_docx` rồi qua cổng, then nhận lỗi có khoá `err.export.publish_copy_not_reimportable`.
- Given bản hai cột vừa xuất từ cùng fixture, when qua cổng, then đi qua.
- Given đối chứng gỡ thật (cổng luôn trả `Ok`), when chạy `reimport_gate_contract`, then các ca từ chối đỏ.

## Design Notes

- Ca sót thứ hai, suy ra từ quy tắc AD-38 và không đổi ở đây: bản hai cột của một Chương chỉ một câu, nếu reviewer nhấn Enter trong ô, sẽ thành 1 hàng có ô 2 đoạn và bị từ chối. Bảng lồng 1 hàng nhiều đoạn trong ô của bản reviewer cũng vậy. Hậu quả là từ chối nhầm, không mất dữ liệu. Muốn đổi thì phải có AD mới.

## Verification

**Commands:**
- `npm run build` rồi `cargo test --test reimport_gate_contract --test docx_boundary` -- xanh. Đối chứng gỡ thật như AC thứ ba.
- `cargo test --test naming_boundary --test export_block_contract` cùng một `*_boundary` có `SRC_RS_FLOOR` -- xanh với tệp mới.
- `npm run check:i18n` -- xanh với khoá mới.

## Implementation Notes

- `ReviewerDocx::admit(DocxParsed)` ở `core/export/reimport_gate.rs` là đường dựng duy nhất (trường private, chỉ có accessor `parsed()`). `ReimportShapeError::PublishCopy` sang `IpcError` có code `export.publish_copy_not_reimportable`. Chưa có nơi nào gọi `admit`; việc nối vào 8.9 là mục nợ `Chủ: Story 8.9`.
- Không phải nâng sàn quần thể: `src-tauri/src` nay có 129 tệp `.rs`, sàn 105 chịu được tới 131.
- Bổ sung so với plan: một ca bản hai cột xuất thật có hàng ảnh (chế độ link) để phủ hàng "có hàng ảnh" của ma trận, và một ca bản một khối xuất thật của Chương ba câu trong một đoạn để phủ ca sót đúng như AD-38 mô tả.
- Đối chứng gỡ thật do bên điều phối chạy: xoá điều kiện từ chối, `admit` luôn trả `Ok`, chữ ký giữ nguyên. Kết quả đỏ đúng 3 ca từ chối (`a_real_one_block_export_is_refused`, `one_multi_paragraph_chapter_among_several_refuses_the_whole_file`, `shapes_built_directly`). Khôi phục thì 8/8 xanh.
- Chưa chạy bộ đầy đủ, vì thay đổi không chạm phần nối chung. Đã chạy `reimport_gate_contract`, `docx_boundary`, `naming_boundary`, `export_block_contract`, `segment_baseline_guard`, `ipc_contract`, `dict_boundary` và `check:i18n`.

## Plan Change Log

## Review Triage Log

Lượt 1 (quick): high 0 · medium 0 · low 3 · false 0 · maybe-false 0.
- low · patch — chưa có bằng chứng đối chứng gỡ cho AC thứ ba: bên điều phối tự gỡ thật và đo lại (Implementation Notes).
- low · patch — ca sót AD-38 (một đoạn nhiều câu) chưa có ca xuất thật: thêm `a_real_one_block_export_of_one_multi_sentence_paragraph_passes`.
- low · reject — thư mục tạm không dọn khi test đỏ: khó gặp, sửa phải thêm `Drop` guard, và các tệp test anh em dùng cùng khuôn.
