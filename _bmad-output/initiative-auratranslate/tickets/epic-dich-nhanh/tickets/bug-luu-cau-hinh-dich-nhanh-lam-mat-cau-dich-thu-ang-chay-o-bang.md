---
id: 8
type: bug
title: "Lưu cấu hình dịch nhanh làm mất câu `Dịch thử` đang chạy ở Bảng chờ Glossary"
parent: epic-dich-nhanh
covers: [R5]
after: []
assignee: ""
refined: true
hitl: false
risk: medium
severity: P2
---

# Lưu cấu hình dịch nhanh làm mất câu `Dịch thử` đang chạy ở Bảng chờ Glossary

## Description

Ở Bảng chờ Glossary, lưu cấu hình dịch nhanh (trường, khoá, xoá khoá) hoặc đổi `Tự dịch` trong Cài đặt sẽ huỷ cả lượt `Dịch thử` hay `Dịch lại` mà người dùng tự bấm: hàng quay về trạng thái trước, câu đang dịch mất, tiền đã trả cho phần đã nhận. Màn xem trước nhập URL trong cùng tình huống chỉ huỷ lượt tự chạy. Lượt bấm tay phải chạy xong ở cả hai bề mặt.

## Reproduction

1. Cài đặt có cấu hình dịch nhanh dùng được; nhà cung cấp trả chậm (provider giả chờ vài giây, hoặc mô hình thật với câu dài).
2. Mở một Tác phẩm, mở Bảng chờ Glossary, đứng ở một hàng chưa có câu dịch, bấm `Dịch thử`.
3. Trong lúc hàng đang dịch, mở Cài đặt → AI và mô hình → Dịch nhanh, sửa model rồi lưu (hoặc lưu khoá, xoá khoá, bật `Tự dịch`).
4. Thực tế: lượt bị huỷ, hàng về trạng thái trước khi bấm. Mong đợi: lượt chạy xong và câu dịch hiện.

Tìm thấy bằng đọc mã ở retro Epic 15 (F1), chưa chạy trong app.

## Cause Hypothesis

Bảng chờ và màn xem trước cùng theo dõi hai tín hiệu "cấu hình nhanh vừa lưu" và "tuỳ chọn `Tự dịch` đổi". Màn xem trước chỉ huỷ lượt đang chạy khi lượt đó là tự chạy; Bảng chờ huỷ lượt đang chạy bất kể ai khởi động nó. Hai bộ hẹn giờ tự dịch được viết song song ở Story 15.5 và 15.6 nên lệch nhau.

## Acceptance Criteria

1. **Lượt bấm tay ở Bảng chờ không bị huỷ vì Cài đặt** (R5)
   **Given** một lượt `Dịch thử` hoặc `Dịch lại` do người dùng bấm đang chạy ở Bảng chờ
   **When** người dùng lưu một trường cấu hình nhanh, lưu hoặc xoá khoá nhanh, hoặc bật hay tắt `Tự dịch`
   **Then** lượt đó chạy xong, câu dịch hiện với nhãn nguồn và mô hình thật, và được lưu vào kho như mọi lượt trọn vẹn
2. **Lượt tự chạy vẫn bị huỷ như trước** (R4, AD-64 mục 8)
   **Given** một lượt tự chạy đang chạy hoặc đang chờ hẹn giờ ở Bảng chờ
   **When** người dùng tắt `Tự dịch` hoặc lưu cấu hình nhanh
   **Then** lượt đó bị huỷ, hẹn giờ bị xoá, và không có lời gọi mạng nào sau đó cho tới khi điều kiện tự chạy lại đủ
3. **Hai bề mặt cho cùng kết quả**
   **Given** ma trận (lượt bấm tay, lượt tự chạy) × (lưu trường, lưu khoá, xoá khoá, bật `Tự dịch`, tắt `Tự dịch`)
   **When** chạy trên Bảng chờ Glossary và trên tầng khối ở màn xem trước nhập URL
   **Then** mỗi ô cho cùng kết quả huỷ hay không huỷ ở hai bề mặt
4. **Có test bắt đúng lỗi này, và test đỏ được**
   **Given** bộ test
   **When** nó chạy
   **Then** một ca chạy lượt bấm tay ở Bảng chờ rồi lưu cấu hình nhanh thấy câu dịch hiện; một ca lượt tự chạy thấy lượt bị huỷ
   **And** đưa lại cách huỷ vô điều kiện làm ca bấm tay đỏ vì câu không hiện
5. **Hoặc: không cần sửa, có bằng chứng**
   **Given** Reproduction
   **When** chạy trên mã hiện tại
   **Then** lượt bấm tay đã chạy xong, bằng chứng ghi trong Notes — thay cho 1 tới 4

## References

- parent — _bmad-output/initiative-auratranslate/tickets/epic-dich-nhanh/epic-dich-nhanh.md, Requirements R5 và Notes (Decision 2026-11-01 "mở Cài đặt không huỷ lượt đang chạy")
- architecture — _bmad-output/initiative-auratranslate/architecture-auratranslate/architecture-auratranslate.md, §AD-64 mục 8
- retrospective — _bmad-output/initiative-auratranslate/tickets/epic-dich-nhanh/epic-dich-nhanh-retrospective.md, F1, A2

## Notes

- Decision: 2026-11-12, Ice — lưu cấu hình nhanh hoặc đổi `Tự dịch` chỉ huỷ lượt tự chạy, không huỷ lượt bấm tay, ở cả hai bề mặt (retro Epic 15, A2).
- Hai chỗ theo dõi khi retro đọc: `src/glossaryQueueState.ts:248` (Bảng chờ) và `src/importPreviewState.ts:1149` (màn xem trước); tín hiệu lưu cấu hình tăng ở `src/quickAiConfigState.ts:110,138,154`.
