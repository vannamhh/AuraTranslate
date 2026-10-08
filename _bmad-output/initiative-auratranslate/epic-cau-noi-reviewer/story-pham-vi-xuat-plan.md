---
title: 'Phạm vi xuất'
type: 'feature'
ticket: '2'
created: '2026-10-07'
status: done
baseline_revision: '9d5e27ce2c0c3144d072eaa52f8a3a3d319b8a17'
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

**Problem:** Người dịch chưa có màn hình xuất để chọn gửi một Chương, vài Chương hay cả Tác phẩm cho reviewer (FR89); mọi định dạng xuất sau (8.3-8.7) cần một phạm vi chung.

**Approach:** Lớp phủ "Xuất" mở từ titlebar: chọn phạm vi, thấy số Chương và số segment, cảnh báo câu chưa xác nhận, chọn thư mục đích qua hộp thoại gọi từ Rust (AD-48), và một khối xem trước kiểu Word. Rust sở hữu phạm vi và phép đếm; chưa có định dạng xuất nào.

## Boundaries & Constraints

**Always:** AD-1 (đếm ở Rust); AD-48 (hộp thoại gọi từ Rust, không thêm quyền plugin cho JS); câu bị cắt bỏ (FR133) và segment về hưu không được đếm; mọi thao tác dùng được bằng bàn phím; khối Word là ngoại lệ token có tên (`aura-allow-*` kèm lý do), màu vẫn đạt WCAG AA.

**Never:** Dựng bất kỳ định dạng xuất nào (8.3-8.7); thêm quyền vào `capabilities/main.json`; thêm phụ thuộc mới.

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected Output / Behavior | Error Handling |
|----------|--------------|---------------------------|----------------|
| Cả Tác phẩm | `Work` | số Chương, câu, câu chưa xác nhận của mọi Chương | — |
| Vài Chương | `Chapters[ids]` | chỉ các Chương đó, id trùng chỉ tính một lần | — |
| Chọn rỗng | `Chapters[]` hoặc Tác phẩm không Chương | — | `export.scope_empty` |
| Chương lạ | id không tồn tại | — | `segment.chapter_not_found` |
| Chưa mở Tác phẩm | — | — | `work.none_open` |
| Huỷ hộp thoại | người dùng đóng hộp chọn thư mục | `Ok(None)`, thư mục cũ giữ nguyên | — |

</frozen-after-approval>

## Code Map

- `src-tauri/src/core/export/scope.rs` -- `ExportScope`, `resolve_chapter_ids`, `count_scope` (`mod.rs` chỉ `pub use`); 8.3-8.7 gọi lại hai hàm này.
- `src-tauri/src/commands/export.rs` -- hàm thuần `export_scope_summary`, `export_folder_from_dialog` và hai vỏ `wire` (vỏ chọn thư mục là `(async)`).
- `src-tauri/src/core/i18n/mod.rs`, `src/i18n/vi.json` -- khoá `err.export.*`, `export.*`, `command.export.*`.
- `src/config/export.ts` -- adapter IPC không ném lỗi.
- `src/exportState.ts`, `src/ExportOverlay.vue`, `src/exportCommandDeps.ts` -- trạng thái và lớp phủ; mẫu: `tmImportState.ts` + `TmImportOverlay.vue`.
- `src/commands/index.ts`, `src/main.ts`, `src/App.vue` -- lệnh `export.*`, nút titlebar `data-export-open`.
- Tests that move: `src-tauri/tests/export_contract.rs`, `src-tauri/tests/config_invariants.rs`, `tests/frontend/exportScope.test.ts`.

## Tasks & Acceptance

**Execution:**
- [x] `core/export/mod.rs`, `commands/chapter.rs`, đăng ký trong `lib.rs` -- phạm vi, phép đếm, thư mục
- [x] test hợp đồng Rust cho ma trận I/O, vỏ chặn `(async)`
- [x] adapter, state, overlay, lệnh, nút titlebar, i18n
- [x] test vitest cho state và overlay (bàn phím, cảnh báo, khối Word)

**Acceptance Criteria:**
- Given màn hình xuất, when mở, then chọn được một Chương · nhiều Chương · cả Tác phẩm.
- Given phạm vi, when hiển thị, then thấy số Chương và số segment sẽ xuất.
- Given phạm vi còn câu chưa xác nhận, when chuẩn bị xuất, then có cảnh báo nêu số câu.
- Given chọn đường dẫn, when bấm, then hộp thoại thư mục mở từ Rust, `capabilities/main.json` không đổi.
- Given khối xem trước Word, when hiển thị, then màu viết cứng có miễn trừ có lý do và đạt WCAG AA.
- Given mọi nút, when dùng bàn phím, then thao tác được.

## Implementation Notes

- Rust nằm trong tệp riêng theo trách nhiệm (`core/export/scope.rs`, `commands/export.rs`) theo quyết định của Ice; hai tệp mới không làm sàn `.rs` nào đỏ (cây 116 tệp, sàn 97), còn `check:doc-refs` `FILE_FLOOR` 421→451 vì quần thể 530 tệp.
- Phép đếm lọc `retired_at IS NULL` và hằng `core::segment::omit::IN_TRANSLATION_SQL` (một bản SQL của vị từ FR133, dùng cả ở `chapter_read.rs`); `export_contract.rs` đối chiếu số đếm SQL với `count_in_translation` (đặt hằng thành `1 = 1` ⇒ 3 ca đỏ).
- "Chưa xác nhận" = `status <> 'confirmed'` trong số câu được đếm. Chọn rỗng là lỗi có tên `export.scope_empty`, không phải số 0.
- Hộp thoại thư mục dùng `blocking_pick_folder()` trong vỏ `(async)`, thêm một hàng vào `blocking_wire_cases`; `capabilities/main.json` không đổi.
- Màn hình chưa có nút xuất và chưa có chọn định dạng; `currentExportScope()` trong `exportState.ts` và `exportFolder` là chỗ 8.3-8.7 cắm vào.
- Khối Word dùng họ vai `word-text`/`word-fill`/`word-stroke` trong `tokens.json` (`word-page` `#ffffff`, `word-ink` `#1a1a1a`, `word-rule` `#767676`, `word-header-fill` `#e7e6e6`), cùng giá trị hai theme, ba cặp trong `contrast.pairs` (17,40 · 13,97 · 4,54); màu qua `var(--color-word-*)` trong CSS, không `:style`, không `aura-allow-literal`. Hạ `word-rule` xuống `#aaaaaa` ⇒ `check:tokens` đỏ vì 2,323:1 dưới sàn 4,5.
- Sàn quần thể frontend phải nâng vì 4 tệp mới: `check-tokens.mjs` FILE_FLOOR 106→115, `check-commands.mjs` TS_FLOOR 75→82, `check-panel-refs.mjs` FILE_FLOOR 75→82; `trapTab` khai vào `HANDLER_TABLE`.

## Open Questions

1. Đã giải quyết: chọn (A), tệp riêng; xem Plan Change Log.

## Plan Change Log

- Ice chốt: mã vào tệp `.rs` riêng và nâng sàn cùng lượt; màu khối Word thành token (vai `word-*`) thay cho hằng có miễn trừ; vị từ SQL một bản ở `omit.rs`. Ràng buộc "Always" và AC khối Word ở phần đóng băng nói về hằng có miễn trừ: nay được thoả bằng token đạt AA, `aura-allow-*` không còn dùng. Nợ real-app của 8-2 giữ nguyên.

## Review Triage Log

## Verification

**Commands:**
- `npm run build` rồi `cargo test --test export_contract` -- đỏ-xanh theo ma trận
- `npm run test:story 8-2` -- xanh
