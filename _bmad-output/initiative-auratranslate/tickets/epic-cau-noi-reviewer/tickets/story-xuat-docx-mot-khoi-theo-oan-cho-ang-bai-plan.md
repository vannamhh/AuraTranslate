---
title: 'Xuất .docx một khối theo đoạn cho đăng bài'
type: 'feature'
ticket: '4'
created: '2026-10-08'
status: 'built'
baseline_revision: '8d6a6f5980a8e3ce06f4262cba9dfaebe5e51301'
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

**Problem:** Người dịch chưa xuất được bản để dán thẳng sang trình soạn thảo website (FR121). Màn hình xuất của 8.2/8.3/8.5 chỉ có `.docx` hai cột, mỗi câu một hàng, mà bản đó dán ra thành mảnh vụn bảng.

**Approach:** Thêm định dạng "`.docx` một khối" vào màn hình xuất. Mỗi Chương còn câu là một bảng hai cột chỉ một hàng, không viền. Mỗi ô gom đoạn từ cờ kết đoạn đã lưu của chính cột đó (AD-46), dời cờ của câu bị lược về câu còn lại liền trước (FR133). Ảnh ghi theo cách đã chọn của 8.5. Tiêu chí nghiệm thu là 13 tiêu chí của story `story-xuat-docx-mot-khoi-theo-oan-cho-ang-bai.md`.

## Boundaries & Constraints

**Always:**
- Tuân AD-1: gom đoạn, đếm và ghi file đều ở Rust.
- Câu đi qua `segments_in_translation`; câu về hưu không vào file.
- Cột phải dùng cờ `is_target_paragraph_end` cộng `\n` trong `target_text`, nối câu bằng một dấu cách. Cột trái dùng cờ `is_paragraph_end` và nối câu bằng `source_joiner(source_lang)`.
- Decision (Ice, 2026-10-08): cột trái nối câu theo ngôn ngữ nguồn. Tiếng Trung nối liền như Decision của story, ngôn ngữ khác thêm một dấu cách. Decision của story chỉ xét tiếng Trung.
- Decision (Ice, 2026-10-08): bản trùng tên đầu tiên mang hậu tố ` (2)`, giống `write_new_file` của 8.3, theo tiêu chí 13 của story.
- Decision (Ice, 2026-10-08): Chương chưa dịch câu nào mà có ảnh thì ô phải vẫn có đoạn ảnh. "Ô phải trống" ở tiêu chí 7 nghĩa là không có chữ bản dịch; ảnh theo tiêu chí 8.
- Câu chưa dịch ở plan này nghĩa là `target_text` chỉ có khoảng trắng. Vị từ này có một bản SQL và một bản Rust, và hai bản phải cho cùng kết quả kể cả với `\n`.
- Cảnh báo một khối nêu riêng hai số: câu có bản dịch nhưng chưa xác nhận, và câu chưa dịch.
- Tên tệp `<tên Tác phẩm>-mot-khoi.docx`, không ghi đè tệp có sẵn.
- Mọi điều khiển là phần tử gốc và dùng được bằng bàn phím.
- Mã Rust mới vào tệp riêng; sàn quần thể nâng cùng lượt (epic Notes 2026-10-07).

**Never:**
- Đổi `.docx` hai cột của 8.3/8.5: hàng, ngắt dòng mềm, số đếm và cảnh báo hiện có, dòng "nhập lại được".
- Ghi vào `project.db`.
- Thêm phụ thuộc hoặc thêm quyền `capabilities/main.json`.
- Làm cổng AD-38 (8.8), khối ghi nguồn (8.7), `.md`/text (8.6).
- Thêm một bản thứ hai của phép gom đoạn bản dịch.

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected Output / Behavior | Error Handling |
|----------|--------------|---------------------------|----------------|
| Nhiều Chương | N Chương còn câu, M Chương hết câu | N bảng một hàng theo `(ord, id)`; mỗi Chương một đoạn tiêu đề ngoài bảng | — |
| `\n` trong bản dịch | `"A\n\n\nB"`, `"\nC\n"` | ô phải: `A` · `B C`, không đoạn rỗng | — |
| Câu bị lược mang cờ | lược ở giữa, lược ở đầu Chương | đoạn kết tại câu còn lại liền trước, ở cả hai cột; câu lược đứng đầu không tạo đoạn rỗng | — |
| Câu chưa dịch | một câu, cả đoạn, cả Chương | không dấu cách thừa, không đoạn rỗng; Chương chưa dịch câu nào có ô phải trống; cột trái đủ | — |
| Ảnh | `link` / `file` theo 8.5 | một đoạn ngay sau câu neo, ở cả hai ô; ảnh thiếu URL ở chế độ link được đếm vào `images_skipped_missing_link` | `export.image_file_missing` như 8.5 |
| Tên trùng | đã có `X-mot-khoi.docx` | `X-mot-khoi (2).docx`, tệp cũ còn nguyên | — |
| Phạm vi rỗng / Chương lạ / chưa mở / không ghi được | như 8.2/8.3 | — | lỗi có tên của 8.2/8.3 |

</frozen-after-approval>

## Code Map

- `src-tauri/src/core/segment/reading.rs:53` -- `paragraphs_in_translation`: phép gom duy nhất của phía đích, có dời cờ câu bị lược. Tham số hoá phép gom theo cờ cần đọc để phía nguồn đi qua cùng mã; hành vi của hàm hiện có không đổi.
- `src-tauri/src/core/segment/regroup.rs:108` -- `source_joiner(source_lang)` (`pub(super)`): `""` với zh, `" "` với ngôn ngữ khác. Nâng lên `pub(crate)` để dùng lại, không chép.
- `src-tauri/src/core/export/table_rows.rs` -- dùng lại `ExportImage` và `load_chapter_tables`, cùng cách đặt ảnh theo neo (`resolve_chapter_images(false,false)`, ảnh không neo đứng đầu); dùng lại cách nạp segment. Không đổi hành vi.
- `src-tauri/src/core/export/docx_table.rs` -- dùng lại `ImageReference` và `image_paragraph` (Hyperlink hoặc đường dẫn `dir/asset_id-file_name`), cùng đoạn tiêu đề Chương. Không đổi bản hai cột.
- `src-tauri/src/core/export/{new_file,image_files,images}.rs` -- dùng nguyên `safe_stem`, `write_new_file`, `write_docx_with_images`, `ImageMode`.
- `src-tauri/src/core/export/scope.rs:68` -- `count_scope`, nơi định nghĩa `unconfirmed_count` là `status <> 'confirmed'`. Thêm hai số đếm mới, giữ nguyên số cũ. Bản SQL "chưa dịch" hiện có là `trim(target_text) = ''` ở `chapter_read.rs:552`, nhưng hàm này không cắt `\n`.
- `src-tauri/src/commands/export.rs:103,181` -- `export_docx_two_column`, `ExportedFile`, `ExportScopeSummary`. Lệnh mới đặt cạnh, đăng ký ở `lib.rs:1107`.
- `src-tauri/tests/config_invariants.rs` -- `blocking_wire_cases` ở `:1166` thêm một hàng; `COMMAND_FILE_CENSUS` của `export.rs` đổi `1,2,2` → `1,3,3` (`:1764`).
- Sàn quần thể: `src-tauri/src` có 121 tệp `.rs`, đúng trần của sàn 97. Mỗi tệp mới làm đỏ 19 hằng `SRC_RS_FLOOR`/`RS_FLOOR`/`RUST_FLOOR`/`SRC_ONLY_RS_FLOOR` trong `tests/*_boundary.rs` và `scripts/check-i18n.mjs:235` `RS_FLOOR=98`. Nâng theo `ceil(0.85 × số đếm thật)`. Xét lại `dict_boundary.rs:269` (160, src+tests 197), `ipc_argument_contract.rs:530`, `check-doc-refs` `FILE_FLOOR`.
- `docx-rs =0.4.22` -- bỏ viền bằng `Table::clear_all_border()` và `TableCell::clear_all_border()`.
- `src/config/export.ts` -- `ExportScopeCounts`, `exportDocxTwoColumn`. Thêm adapter và `CMD_*` cho lệnh mới; tên khoá `invoke` phải khớp tham số Rust (`ipc_argument_contract.rs`).
- `src/exportState.ts` -- `ExportFormat` (`:15`), `runExport` (`:196`, hiện gọi cứng lệnh hai cột), `resetExport`.
- `src/ExportOverlay.vue` -- cảnh báo `:194`, fieldset định dạng `:201-215`, dòng `data-export-reimportable` `:212`. Thêm radio thứ hai, dòng "không nhập lại được", hai cảnh báo một khối.
- `src/i18n/vi.json` `export.*`, `src-tauri/src/core/i18n/mod.rs`.
- Tests that move: `src-tauri/tests/export_contract.rs` (số đếm), `src-tauri/tests/export_docx_contract.rs` (mẫu fixture `seg`/`fixture`/`document_xml`/`parsed`, đọc ngược qua `core::docx::read_docx`), `tests/frontend/exportDocx.test.ts`, `exportScope.test.ts`, `exportImages.test.ts`.

## Tasks & Acceptance

**Execution:**
- [x] `core/segment/reading.rs` -- tham số hoá phép gom theo cờ; ca sẵn có của `paragraphs_in_translation` giữ xanh.
- [x] `core/export/block_paragraphs.rs` (mới) -- mô hình `ChapterBlock` gồm tiêu đề và hai danh sách đoạn (văn bản hoặc ảnh), dựng từ segment; tách `\n` và ghép câu như phần Always; vị từ "chưa dịch" bản Rust.
- [x] `core/export/docx_block.rs` (mới) -- ghi một bảng một hàng không viền cho mỗi Chương còn câu, còn Chương hết câu chỉ ghi tiêu đề; không panic.
- [x] `core/export/scope.rs`, `commands/export.rs`, `lib.rs`, i18n Rust -- thêm `unconfirmed_translated_count` và `untranslated_count`; lệnh `export_docx_one_block(scope, image_mode, folder)` `(async)`, trả `ExportedFile`.
- [x] `tests/export_block_contract.rs` (mới) -- phủ cả ma trận I/O và tiêu chí 1–10 và 13 của story. Đọc viền và Hyperlink từ `word/document.xml`, đếm đoạn mỗi ô qua `read_docx`. Có ca đối chiếu SQL/Rust cho vị từ "chưa dịch".
- [x] `config_invariants.rs`, các sàn quần thể -- census, `blocking_wire_cases`, nâng sàn theo số đo thật.
- [x] `src/config/export.ts`, `exportState.ts`, `ExportOverlay.vue`, `src/i18n/vi.json` -- định dạng thứ hai, dòng không nhập lại được, hai cảnh báo (số bằng 0 thì ẩn), `runExport` theo định dạng.
- [x] `tests/frontend/exportBlock.test.ts` (mới) -- tiêu chí 11–12: chọn bằng bàn phím, đổi dòng ghi chú, hai cảnh báo, giá trị gửi đi; bản hai cột không đổi.
- [x] `deferred-work.md` -- ghi phần của 8.4 vào hai mục ở References của story, và thêm mục nợ dán thật `Chủ: Epic 8`.

**Acceptance Criteria:**
- Given 13 tiêu chí của story, when chạy test hợp đồng Rust và vitest, then mỗi tiêu chí có ít nhất một ca chạm đúng đường mã của nó.
- Given bản hai cột, when xuất lại các fixture của 8.3/8.5, then tệp và số đếm không đổi.

## Design Notes

- Một ô là chuỗi đoạn. Ảnh neo giữa đoạn thì cắt đoạn đó thành ba phần: văn bản trước ảnh, đoạn ảnh, văn bản sau ảnh.
- Cột phải: gom theo cờ đích. Mỗi câu bỏ `\n` ở đầu và cuối; mỗi cụm `\n` ở giữa câu là một ranh giới đoạn. Câu chưa dịch không đóng góp chữ nhưng cờ của nó vẫn có tác dụng. Đoạn rỗng thì bỏ.
- Câu hỏi mở của story ("nguyên văn có chứa `\n` không") đã có trả lời trong repo: không. Bất biến ở `split.rs:213` và ca `no_segment_ever_carries_a_line_break`, nên cột trái không tách đoạn theo `\n`.
- Bảng một hàng có ô nhiều đoạn chính là hình dạng mà AD-38 từ chối. Ở đây đó là chủ ý, đúng với "không nhập lại được".

## Verification

**Commands:**
- `npm run build` rồi `cargo test --test export_block_contract --test export_docx_contract --test export_contract` -- xanh; đối chứng gỡ thật: bỏ dời cờ câu lược ở phía nguồn ⇒ ca FR133 cột trái đỏ; gỡ `clear_all_border` ⇒ ca viền đỏ.
- `cargo test --test config_invariants --test ipc_argument_contract` và các `*_boundary` có sàn vừa nâng -- xanh.
- `npx vitest run tests/frontend/exportBlock.test.ts tests/frontend/exportDocx.test.ts` -- xanh.
- Chạm `lib.rs` (đăng ký lệnh) ⇒ chạy cả bộ một lần theo AGENTS.md.

## Implementation Notes

- Lệnh `export_docx_one_block(scope, imageMode, folder)` (async) trả đúng `ExportedFile` của bản hai cột; tên tệp `<tên Tác phẩm>-mot-khoi.docx`, thư mục ảnh `<stem>-anh/`. Không thêm khoá i18n Rust, không đổi `core/i18n`.
- Phép gom đoạn duy nhất là `paragraphs_by_flag` trong `reading.rs` (tham số hoá theo cờ), cả hai cột đi qua nó; `source_joiner` lên `pub(crate)`; `docx_table.rs` mở `paragraph_of`/`image_paragraph`/`COLUMN_WIDTH_DXA` cho `docx_block.rs` mà bản hai cột không đổi.
- `ExportScopeSummary` thêm `unconfirmed_translated_count` và `untranslated_count`; `unconfirmed_count` giữ nguyên (vẫn đếm cả câu chưa dịch). Vị từ "chưa dịch" có bản SQL và bản Rust, ca đối chiếu nằm trong `export_block_contract.rs`.
- Sàn quần thể: `src-tauri/src` nay 123 tệp `.rs` (kế hoạch ghi 121; thêm `block_paragraphs.rs`, `docx_block.rs`); 21 hằng 97 sang 105, `SRC_TAURI_RS_FLOOR` 160 sang 171, `RS_FLOOR` của `check-i18n.mjs` 98 sang 106.
- `ipc_argument_contract` đỏ cố ý ở pha Rust cho tới khi `src/config/export.ts` có `invoke('export_docx_one_block', {scope, imageMode, folder})`; nay xanh, lệnh không nằm trong `NO_FRONTEND_CALLER`.
- Đối chứng gỡ thật: bỏ phép dời cờ câu lược trong `paragraphs_by_flag` làm ca FR133 đỏ; bỏ `clear_all_border` làm ca viền đỏ.
- Màn hình: chọn một khối ẩn cảnh báo `unconfirmed_count` cũ và hiện hai cảnh báo riêng (mỗi số bằng 0 thì ẩn); bản hai cột giữ nguyên cảnh báo và dòng "nhập lại được". `runExport` chọn lệnh theo `format`.
- Sổ nợ: phần của 8.4 ghi vào hai mục FR133 một chiều và AC4 (cả hai còn 🟡, chủ cũ giữ), thêm mục dán thật sang trình soạn thảo `Chủ: Epic 8`.
- 🔵 2026-10-08: bộ đầy đủ đã chạy sau lượt vá review: 12 cổng, build và `cargo test` xanh; vitest 1940/1940 ở một lượt, lượt khác đỏ 3–4 ca hết giờ 5 s ngoài 8.4 khi tải máy 278–319. Chưa chạy: CI hai nền tảng, e2e.

## Review Triage Log

Lượt 1 (quick): high 0 · medium 1 · low 4 · false 1 · maybe-false 0.
- low · patch — doc comment mới bằng tiếng Việt, mang FR121/AD-46, nhiều dòng chỉ nhắc lại chữ ký (AGENTS.md §Code comments): xoá, chỉ giữ hai bất biến bằng tiếng Anh.
- medium · patch — chưa chạy bộ đầy đủ dù chạm `lib.rs`, `config_invariants.rs` và các sàn: chạy ở phía điều phối sau khi vá.
- false · reject — "vị từ thứ ba ở `chapter_read.rs:552` làm hai màn hình hiểu 'chưa dịch' khác nhau": đó là điều kiện cho phép tự điền TM (chỉ điền câu đang trống), không phải phép đếm trên màn hình nào.
- low · patch — `source_pieces` gọi vị từ của `target_text` lên nguyên văn: tách `is_blank` riêng, hành vi không đổi.
- low · patch — `target_pieces` giữ khoảng trắng ở mép nên `"A "` + `" B"` ra hai dấu cách: cắt khoảng trắng mép từng mẩu, thêm ca test.
- low · patch — Chương chưa dịch có ảnh: tiêu chí 7 và 8 đánh nhau; Ice chốt ảnh vẫn ở ô phải (Decision trong phần đóng băng); thêm ca test khoá hành vi.
