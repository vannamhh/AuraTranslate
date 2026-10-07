---
title: 'Phạm vi xuất'
type: 'feature'
ticket: '2'
created: '2026-10-07'
status: 'in-progress'
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

- `src-tauri/src/core/export/scope.rs` -- `ExportScope`, `resolve_chapter_ids`, `count_scope` (mới); 8.3-8.7 gọi lại hai hàm này.
- `src-tauri/src/commands/export.rs` -- hàm thuần `export_scope_summary`, `export_folder_from_dialog` và vỏ `wire` (mới; vỏ chọn thư mục là `(async)`).
- `src-tauri/src/core/i18n/mod.rs`, `src/i18n/vi.json` -- khoá `err.export.*`.
- `src-tauri/tests/config_invariants.rs` -- danh sách vỏ CHẶN (`blocking_wire_cases`) và số đếm tệp.
- `src/config/export.ts` -- adapter IPC không ném lỗi.
- `src/exportState.ts`, `src/ExportOverlay.vue`, `src/exportCommandDeps.ts` -- trạng thái và lớp phủ; mẫu: `tmImportState.ts` + `TmImportOverlay.vue`.
- `src/commands/index.ts`, `src/main.ts`, `src/App.vue` -- lệnh `export.*`, nút titlebar `data-export-open`.

## Tasks & Acceptance

**Execution:**
- [x] `core/export/scope.rs`, `commands/export.rs`, đăng ký trong `lib.rs` -- phạm vi, phép đếm, thư mục
- [ ] test hợp đồng Rust cho ma trận I/O, vỏ chặn `(async)`
- [ ] adapter, state, overlay, lệnh, nút titlebar, i18n
- [ ] test vitest cho state và overlay (bàn phím, cảnh báo, khối Word)

**Acceptance Criteria:**
- Given màn hình xuất, when mở, then chọn được một Chương · nhiều Chương · cả Tác phẩm.
- Given phạm vi, when hiển thị, then thấy số Chương và số segment sẽ xuất.
- Given phạm vi còn câu chưa xác nhận, when chuẩn bị xuất, then có cảnh báo nêu số câu.
- Given chọn đường dẫn, when bấm, then hộp thoại thư mục mở từ Rust, `capabilities/main.json` không đổi.
- Given khối xem trước Word, when hiển thị, then màu viết cứng có miễn trừ có lý do và đạt WCAG AA.
- Given mọi nút, when dùng bàn phím, then thao tác được.

## Implementation Notes

## Plan Change Log

## Review Triage Log

## Verification

**Commands:**
- `npm run build` rồi `cargo test --test export_contract` -- đỏ-xanh theo ma trận
- `npm run test:story 8-2` -- xanh
