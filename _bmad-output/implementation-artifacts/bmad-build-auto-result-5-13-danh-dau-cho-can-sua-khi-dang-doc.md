---
status: done
story: 5.13
slug: 5-13-danh-dau-cho-can-sua-khi-dang-doc
---

# BMad Build Auto Result

## Status

done

## Blocking condition — resolved

Story 5.13 từng bị chặn vì có hai thiết kế hợp lệ cho marker của segment về hưu. Ice đã chọn phương án B; vòng tự động tiếp tục và hoàn tất triển khai, review, xác minh ngày 2026-08-31.

### Phương án A — duy trì vị trí của mọi tombstone bằng `ord`

- Không thêm khái niệm anchor vào marker; marker chỉ giữ `segment_id` gốc.
- Mỗi lượt gộp/tách phải dịch hoặc quy lại `ord` cho các tombstone bị ảnh hưởng trong cả Chương, không chỉ nhóm vừa về hưu.
- Chi phí ghi tăng theo số tombstone liên quan trong Chương: `O(T)` với `T` là số segment đã về hưu cần dịch vị trí.
- Điểm yếu: vị trí điều hướng tiếp tục được biểu diễn bằng số thứ tự khả biến, trong khi AD-3 yêu cầu dữ liệu gắn với segment tham chiếu ID chứ không tham chiếu vị trí.

### Phương án B — marker giữ một ID neo sống riêng

- Bảng marker giữ `segment_id` gốc để bảo toàn danh tính/nội dung và thêm `navigation_segment_id` để điều hướng.
- Khi gộp/tách làm neo về hưu, cùng transaction cập nhật neo sang ID sống đầu tiên của nhóm thay thế.
- Một lượt gộp chỉ chạm tối đa hai tập marker đang neo vào hai segment cũ; một lượt tách chạm một tập. Chi phí phụ thuộc số marker thực sự có neo bị thay, không phụ thuộc toàn bộ tombstone của Chương.
- Điểm yếu: `write_regroup` phải biết và bảo trì quan hệ neo của tính năng marker; schema có thêm một quan hệ bền vững.

## Evidence

- `ARCHITECTURE-SPINE.md` AD-3: dữ liệu tham chiếu segment bằng ID, không bằng vị trí.
- `ARCHITECTURE-SPINE.md` AD-5: marker của segment đã về hưu phải còn và vẫn mở đúng vị trí trong Chương.
- `src-tauri/src/commands/segment.rs` (`write_regroup`): hiện chỉ gán `ord` cho các hàng vừa về hưu bằng đầu nhóm thay thế; chưa dịch tombstone cũ khi một lần regroup sau xảy ra trước vị trí đó.
- `src-tauri/src/core/store/schema.rs`: migration project hiện dừng ở 17; cả hai phương án đều cần migration 18 cho bảng marker, còn phương án B thêm cột neo ID.

## Decision taken

Ice chọn **B — `navigation_segment_id`**.

## Resolution

Ice chọn **B — `navigation_segment_id`** ngày 2026-08-31. Vòng tự động đã tiếp tục tại
`spec-5-13-danh-dau-cho-can-sua-khi-dang-doc.md`.

## Outcome

Marker bền vững, exact navigation, marker list, neo đọc theo ID và rebase cùng transaction đã được
triển khai. Review bốn lớp đã áp 20 bản vá; toàn bộ static gates, 766 Vitest, Rust suite và bàn đo
WebKit Story 5.13 đều xanh. Repository được hoàn tất bằng một commit cục bộ, không push.
