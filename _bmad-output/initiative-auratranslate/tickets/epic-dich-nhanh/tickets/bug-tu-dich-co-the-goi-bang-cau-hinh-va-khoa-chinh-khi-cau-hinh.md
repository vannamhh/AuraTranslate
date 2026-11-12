---
id: 9
type: bug
title: "Tự dịch có thể gọi bằng cấu hình và khoá chính khi cấu hình nhanh vừa bị xoá"
parent: epic-dich-nhanh
covers: [R4, R11]
after: []
assignee: ""
refined: true
hitl: false
risk: medium
severity: P2
---

# Tự dịch có thể gọi bằng cấu hình và khoá chính khi cấu hình nhanh vừa bị xoá

## Description

Tự dịch chỉ được chạy bằng cấu hình nhanh khai riêng; AD-64 mục 2 nói tự chạy không bao giờ dùng cấu hình chính. Nếu người dùng xoá endpoint hoặc model nhanh đúng lúc một lượt tự chạy đã qua bước kiểm điều kiện mà chưa chuẩn bị lời gọi, lượt đó đi bằng nhà cung cấp, endpoint, model và khoá chính, tức một lời gọi tốn tiền mô hình chính mà người dùng không bấm. Lượt đó phải dừng với `auto_off` và không gọi mạng.

## Reproduction

Khe thời gian quá hẹp để bấm tay; tái hiện bằng test Rust qua lệnh thật, ở mỗi bề mặt:

1. Global có cấu hình chính trỏ provider giả A, cấu hình nhanh khai riêng trỏ provider giả B, `Tự dịch` bật.
2. Gọi lệnh dịch của Bảng chờ (hoặc lệnh lô của tầng khối) với `trigger: auto`.
3. Sau khi bước kiểm điều kiện tự chạy cho qua và trước bước chuẩn bị lời gọi, xoá endpoint nhanh.
4. Thực tế (theo đọc mã): A nhận một lời gọi với khoá chính. Mong đợi: lệnh trả `auto_off`, A và B đều nhận 0 lời gọi.

Tìm thấy bằng đọc mã ở retro Epic 15 (F2), chưa chạy.

## Cause Hypothesis

Bước kiểm điều kiện tự chạy đọc cấu hình Global một lần và xác nhận cấu hình nhanh khai riêng. Bước chuẩn bị lời gọi đọc lại cấu hình qua hàm phân giải dùng chung với cú bấm tay; hàm này không biết lượt là tự chạy, nên khi endpoint hoặc model nhanh trống nó rơi về trọn cấu hình chính đúng như R1 cho cú bấm tay.

## Acceptance Criteria

1. **Tự chạy không bao giờ tới cấu hình chính** (R4, R11; AD-64 mục 2)
   **Given** một lượt `trigger: auto` ở Bảng chờ hoặc ở tầng khối
   **When** cấu hình nhanh không còn khai đủ endpoint và model ở bất kỳ thời điểm nào trước khi lời gọi được dựng, kể cả sau bước kiểm điều kiện
   **Then** lệnh trả `auto_off`, không đọc khoá chính, và không provider nào nhận lời gọi
2. **Cú bấm tay giữ nguyên R1**
   **Given** cấu hình nhanh trống cả endpoint lẫn model
   **When** người dùng bấm `Dịch thử` hoặc dịch các khối đang thấy
   **Then** lời gọi đi bằng trọn cấu hình chính ở tầng Global như trước; khai một nửa vẫn là `not_configured`
3. **Có test bắt đúng lỗi này, và test đỏ được**
   **Given** bộ test
   **When** nó chạy
   **Then** một ca qua lệnh thật của mỗi bề mặt xoá cấu hình nhanh giữa bước kiểm điều kiện và bước chuẩn bị rồi thấy `auto_off` với 0 lời gọi
   **And** gỡ chốt mới làm ca đó đỏ vì provider chính nhận một lời gọi
4. **Hoặc: không cần sửa, có bằng chứng**
   **Given** Reproduction
   **When** chạy trên mã hiện tại
   **Then** lượt đã trả `auto_off` với 0 lời gọi, bằng chứng ghi trong Notes — thay cho 1 tới 3

## References

- parent — _bmad-output/initiative-auratranslate/tickets/epic-dich-nhanh/epic-dich-nhanh.md, Requirements R1, R4, R11
- architecture — _bmad-output/initiative-auratranslate/architecture-auratranslate/architecture-auratranslate.md, §AD-64 mục 1, 2
- retrospective — _bmad-output/initiative-auratranslate/tickets/epic-dich-nhanh/epic-dich-nhanh-retrospective.md, F2, A3

## Notes

- Decision: 2026-11-12, Ice — mở bug riêng cho lỗi này, tách khỏi Bug 15.8 (retro Epic 15, A3).
- Chỗ retro đọc: `quick_auto_gate` và `resolve_quick_call_config` trong `src-tauri/src/commands/aitranslate.rs`; bước chuẩn bị chạy trong `spawn_blocking` của hai lệnh wire.
