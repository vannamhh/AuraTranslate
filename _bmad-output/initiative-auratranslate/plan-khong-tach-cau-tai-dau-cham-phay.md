---
title: 'Không tách câu tại dấu chấm phẩy khi nhập'
type: 'feature'
ticket: ''
created: '2026-11-04'
status: 'built'
baseline_revision: 'f410c0437496e51ad270328709457f75db5c3aa1'
route: 'oneshot'
route_source: 'auto'
risk: 'medium'
review: 'quick'
review_source: 'pinned'
lenses_ran: ['quick']
review_loop_iteration: 0
context: []
---

<frozen-after-approval reason="human-owned intent — do not modify unless human renegotiates">

## Intent

**Problem:** Khi nhập văn bản tiếng Trung, bộ tách câu cắt tại `；` (`ZH_TERMINATORS`, `src-tauri/src/core/segment/split.rs:51`), nên một câu ghép như `他走了；她笑了。` thành hai segment. Ice muốn `；` không còn là ranh giới câu.

**Approach:** Gỡ `；` khỏi `ZH_TERMINATORS` (bảng còn `。！？`). Nhánh tiếng Anh đã không tách tại `;` ASCII, giữ nguyên. Bảng kết câu vẫn có đúng một chủ (`split.rs`), nên bước nối dòng của `normalize` (gọi `line_ends_a_sentence`) cũng thôi coi một dòng kết bằng `；` là trọn câu: dòng đó được nối với dòng sau trong cùng đoạn. Đo trên 42 Tác phẩm `zh` thật ở `~/Documents/AuraTranslate` (chỉ đọc, 2026-11-04): 0 dòng kết bằng `；`, nên hệ quả này gần như không xảy ra. Segment đã nhập không đổi (AD-4); chỉ lượt nhập mới theo luật mới.

**Quyết định (Ice, 2026-11-04):** FR23 ở `prd-auratranslate.md` và `spec-auratranslate/requirements.md` sửa tại chỗ thành `。！？`, kèm một dòng 🔵 ghi ngày và lý do, cùng commit với mã.

</frozen-after-approval>

## Tasks

- `src-tauri/src/core/segment/split.rs` -- `ZH_TERMINATORS: [char; 3] = ['。', '！', '？']`; sửa doc-comment dòng 50 và comment dòng 264 cho khớp; thêm ca `line_ends_a_sentence("他走了；", true) == false`.
- `src-tauri/tests/segment_contract.rs` -- ca AC1 thành ba dấu; ca `；` đảo thành khẳng định `他走了；她笑了。` ra ĐÚNG MỘT segment.
- `src-tauri/tests/segment_boundary.rs`, `src-tauri/tests/segment_normalize_boundary.rs` -- bảng so `ZH_TERMINATORS`/`ZH_TERMINATOR_CHARS` còn ba ký tự (đối chứng dương ở `segment_boundary.rs` đọc `split.rs`, phải khớp bảng mới).
- Đối chứng đỏ: chạy `segment_contract` trước khi sửa `split.rs` ⇒ ca `；` mới phải đỏ.

## Implementation Notes

Oneshot: thay một hằng và bốn chỗ test/cổng, dưới 40 dòng.
- Đối chứng đỏ: `a_chinese_semicolon_does_not_end_a_sentence` đỏ trước khi sửa `split.rs` (`["他走了；", "她笑了。"]`), xanh sau.
- Năm tệp `e2e/**` mang câu chú thích/thông báo `bộ tách nhìn 。！？；` — đổi thành `。！？` vì câu đó hết đúng; fixture e2e không chứa `；` nên không ca nào đổi số câu.

## Review Triage Log

Quick: 3 patch · 1 defer · 3 bác.
- low · patch — `split.rs:11,15` bảng UAX #29 còn gọi `；` là trượt AC1; đổi thành "đúng".
- low · patch — `segment_boundary.rs:55` câu có ngày/số đo trong dòng đang sửa; đã xoá phần đo.
- medium · patch — normalize chưa ca nào đi qua dòng kết bằng `；`; thêm `a_line_ending_in_a_chinese_semicolon_joins_the_next_line`.
- medium · defer — Story 2.1 `verify` trong `tickets.toml` và plan story còn `。！？；`; workflow cấm ghi tệp vé, vào `deferred-work.md` (Chủ: Ice).
- false — trùng tên test giữa unit và integration: hai crate khác nhau, không xung đột.
- false — e2e ngoài Verification: chỉ đổi chú thích/chuỗi thông báo, fixture không chứa `；`.
- false — status/ticket rỗng của plan: đặt khi chốt; việc này không có vé.

## Verification

- `npm run build` (nếu thiếu `dist/`) rồi `cargo test --test segment_contract --test segment_boundary --test segment_normalize_boundary` và `cargo test --lib core::segment` -- xanh.
