---
title: 'Xuất .md và text thuần'
type: 'feature'
ticket: '6'
created: '2026-10-08'
status: 'built'
baseline_revision: 'd81c1f588605aef3bbf8c0a0f3bd6de0e48a29d3'
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

**Problem:** Màn hình xuất mới có hai định dạng `.docx` (FR87, FR121). Người đăng chưa có bản Markdown hay text thuần giữ ảnh, alt-text và chú thích đã dịch (FR88).

**Approach:** Thêm hai định dạng "Markdown (`.md`)" và "Text thuần (`.txt`)". Đoạn lấy từ cờ kết đoạn của bản dịch (AD-46) qua phép gom `paragraphs_by_flag` đã có. Ảnh theo chế độ đã chọn ở 8.5. Alt-text và chú thích là bản dịch, đọc từ hai segment vai của ảnh (AD-42), không in thành đoạn văn xuôi.

## Boundaries & Constraints

**Always:**
- Tuân AD-1: gom đoạn, escape và ghi tệp ở Rust.
- Câu qua `segments_in_translation`; chỉ cột bản dịch; không nhân bản phép gom đoạn.
- Màn hình nói text thuần không nhập lại được, cùng nhóm với `.docx` một khối. `.md` không mang dòng đó.
- Hai cảnh báo của 8.4 (câu chưa xác nhận, câu chưa dịch) áp cho cả hai định dạng mới.
- Không ghi đè tệp có sẵn (`write_new_file`). Mã Rust mới vào tệp riêng.

**Decisions (Ice, 2026-10-08):**
- Chú thích trong `.md`: dòng nghiêng `*<chú thích đã dịch>*` ngay dưới dòng ảnh `![<alt đã dịch>](<đích>)`.
- Ảnh trong text thuần: mỗi ảnh một dòng `[Ảnh: <alt đã dịch>] <link-hoặc-đường-dẫn>`, tiếp theo là dòng chú thích đã dịch (nếu có), để người đăng dựng lại được bài.
- Tên tệp: `<tên Tác phẩm>.md` và `<tên Tác phẩm>.txt`, không hậu tố; `.md` mỗi Chương mở bằng `## <tên Chương>`, text thuần bằng dòng tên Chương rồi dòng trống.
- Giữ cả hai định dạng trong một vé, chấp nhận plan dài ~2800 token, vì hai định dạng dùng chung bộ nạp.

**Never:**
- Thêm phụ thuộc hoặc quyền `capabilities/main.json`; ghi vào `project.db`; tải ảnh từ mạng.
- Đổi hai bản `.docx`, khối ghi nguồn (8.7), cổng AD-38 (8.8), đường nhập lại `.md` (8.9).
- Suy cấu trúc đoạn từ nội dung.

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected Output / Behavior | Error Handling |
|----------|--------------|---------------------------|----------------|
| Đoạn | cờ đích, `\n` trong bản dịch, câu bị lược mang cờ | ranh giới đoạn như 8.4 cột phải | — |
| Ảnh có alt, chú thích | hai segment vai đã dịch | một khối ảnh Markdown mang alt đã dịch, chú thích đã dịch tách bạch alt | — |
| Chưa dịch alt/chú thích | `target_text` trắng | alt rỗng; không dòng chú thích | — |
| Ký tự đặc biệt | `*`, `_`, `[`, `#`, `>` ở đầu dòng; `]` trong alt | `.md` vẫn hiển thị đúng chữ gốc; text thuần không escape | — |
| Ảnh link thiếu URL | chế độ link | bỏ, đếm vào `images_skipped_missing_link` | như 8.5 |
| Ảnh trong text thuần | link hoặc file | dòng `[Ảnh: <alt đã dịch>] <link-hoặc-đường-dẫn>`, rồi dòng chú thích đã dịch nếu có | — |
| Tên trùng, phạm vi rỗng, Chương lạ, tệp ảnh mất | như 8.2/8.5 | — | lỗi có tên cũ |

</frozen-after-approval>

## Code Map

- `src-tauri/src/core/export/block_paragraphs.rs` -- `build_column`, `target_pieces`, `load_chapter_blocks`, `UNTRANSLATED_SQL`: mẫu nạp. Vai alt/caption hiện thành đoạn chữ ở đây; không đổi.
- `src-tauri/src/core/segment/reading.rs:58` -- `paragraphs_by_flag`; `core/segment/image.rs` -- `strip_role_segments`, `resolve_chapter_images` (đã trả `caption_text`).
- `src-tauri/src/core/export/images.rs` -- `ScopeImage` có `alt_text`, thiếu `caption_text`; `chapter_images` sửa một chỗ.
- `src-tauri/src/core/export/{docx_table,image_files,new_file}.rs` -- `ImageReference`, `write_docx_with_images`, `write_new_file`. `write_docx_with_images` gắn cứng `.docx`; cần dạng nhận đuôi tệp.
- `src-tauri/src/commands/export.rs` (273 dòng) -- `write_export_docx`, `exported_file`, mẫu `export_docx_one_block` và vỏ `wire`; đăng ký ở `lib.rs:1110`.
- `src/config/export.ts`, `src/exportState.ts` (`ExportFormat`, `runExport`), `src/ExportOverlay.vue:224-240`, `src/i18n/vi.json:348-354` -- định dạng, dòng không nhập lại được, cảnh báo.
- Tests: `tests/export_block_contract.rs` (mẫu fixture), `tests/frontend/exportBlock.test.ts`, `ipc_argument_contract.rs`.
- Không có crate hay hàm escape Markdown trong repo; tự viết, không phụ thuộc mới (NFR15 không kích hoạt).
- Cổng quần thể `.rs`: `src-tauri/src` có 123 tệp, sàn 105 (tối đa 131 cho 80%). Thêm 1 tệp thì chưa phải nâng sàn.

## Tasks & Acceptance

**Execution:**
- [ ] `core/export/images.rs` -- thêm `caption_text` vào `ScopeImage` -- bản dịch chú thích đi cùng ảnh
- [ ] `core/export/image_files.rs` -- `write_docx_with_images` nhận đuôi tệp (`docx`/`md`/`txt`), hành vi `.docx` giữ nguyên -- dùng lại thư mục `<stem>-anh/`
- [ ] `core/export/text_export.rs` (mới) -- nạp Chương, gom đoạn bản dịch, bỏ vai khỏi văn xuôi, dựng Markdown và text thuần, escape Markdown -- một chỗ duy nhất
- [ ] `core/export/mod.rs`, `commands/export.rs`, `lib.rs` -- lệnh `export_text(scope, imageMode, format, folder)`, trả `ExportedFile`
- [ ] `src/config/export.ts`, `exportState.ts`, `ExportOverlay.vue`, `vi.json` -- hai định dạng, dòng không nhập lại được cho text thuần, hai cảnh báo, `runExport` theo định dạng
- [ ] `tests/export_text_contract.rs` (mới), `tests/frontend/exportText.test.ts` (mới) -- ma trận I/O; đối chứng gỡ thật cờ đích ⇒ ca AD-46 đỏ; `ipc_argument_contract.rs`
- [ ] `deferred-work.md` -- đóng bằng chữ mục AD-46 có `Chủ: Story 8.6` (grep, không đọc cả tệp) -- nợ hết chủ khi vé done

**Acceptance Criteria:**
- Given phạm vi đã chọn, when xuất `.md`, then tệp qua một bộ phân tích CommonMark ra đúng đoạn, ảnh và chữ.
- Given phạm vi đã chọn, when xuất text thuần, then tệp không còn ký hiệu Markdown nào do bộ xuất thêm.
- Given Chương có ảnh, when xuất `.md`, then ảnh tham chiếu theo link hoặc file như đã chọn ở 8.5, và ảnh thiếu link được đếm.
- Given ảnh có alt-text và chú thích đã dịch, when xuất, then alt đã dịch ở phần alt, chú thích đã dịch tách riêng, không chữ gốc nào lọt vào.
- Given cờ kết đoạn đích khác cờ nguồn, when xuất, then đoạn theo cờ đích.
- Given màn hình xuất, when chọn text thuần, then nêu không nhập lại được; chọn `.md` thì không nêu.

## Design Notes

- Alt đặt vào `![alt](dest)`: bỏ `\n`, escape `[` `]` `\`. Đích dùng dạng `<dest>` nếu có khoảng trắng; tên tệp ảnh có thể chứa khoảng trắng (`<asset_id>-<file_name>`).
- Escape chỉ tại vị trí có nghĩa với CommonMark (đầu dòng: `#` `>` `-` `+` `1.`; mọi nơi: `\` `*` `_` `` ` `` `[` `]` `<`).
- `TextFormat { Markdown, Plain }` dùng chung bộ nạp để hai tệp không lệch cấu trúc.

## Verification

**Commands:**
- `npm run build` rồi `cargo test --test export_text_contract --test export_block_contract --test export_contract` -- xanh
- `cargo test --test ipc_argument_contract --test config_invariants` -- xanh
- `npx vitest run tests/frontend/exportText.test.ts tests/frontend/exportBlock.test.ts` -- xanh
- Chạm `lib.rs` ⇒ chạy cả bộ một lần theo AGENTS.md.

**Manual checks (if no CLI):**
- Dán `.md` vào trình xem Markdown thật; kiểm ảnh link và ảnh file hiện đúng (nợ real-app của Epic 8).

## Review Triage Log

- AC1 không qua bộ phân tích CommonMark — medium; repo không có parser, thêm một cái trái "Never: thêm phụ thuộc" trong intent — Ice chọn giữ so chuỗi viết tay, phần còn lại là mục nợ kiểm tay (Chủ: Epic 8).
- `ipc_argument_contract.rs` không được sửa — false; test quét `generate_handler!` và `invoke()` chung cho mọi lệnh, không cần mục riêng; chạy lại ở bước kiểm.
- Phép tách ảnh theo neo và đếm ảnh thiếu link bị chép từ `block_paragraphs.rs` — medium; sửa ảnh ở `.docx` một khối sẽ không tới `.md`/`.txt` — patch (helper chung).
- `LoadedText.image_count` không ai đọc — low; hai số đếm có thể lệch — patch (xoá).
- Phạm vi toàn Chương trống ghi tệp rỗng, báo thành công — low; cùng hành vi với `.docx` một khối của 8.4, chặn cần khoá lỗi mới — defer (Chủ: Ice).
- `export.images.link` vẫn nói "mỗi hàng ảnh" — low; radio nay áp cho `.md`/`.txt` — patch.
- Doc comment tiếng Việt trên `render_text`, doc comment lặp tên trên `exportFormatIsForPublishing` — low; trái quy tắc comment của AGENTS.md — patch (xoá).
- `exportFormatIsForPublishing` dùng `!== 'docx_two_column'` — low; định dạng thêm sau tự nhận cảnh báo đăng bài — patch (liệt kê rõ).
- Thực thể HTML trong đích ảnh `.md` không escape — low; chỉ khi URL/tên tệp chứa dạng `&tên;`, hiếm, sửa cần nhánh mới — rejected.
