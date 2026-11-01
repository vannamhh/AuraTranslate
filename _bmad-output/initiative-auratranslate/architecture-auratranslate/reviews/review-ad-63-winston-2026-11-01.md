# Winston rà AD-63, 2026-11-01

Rà chỉ đọc trên `63e5fbda4b16e35918e11e818ecf5f801c487929`. Phán quyết: chưa ký; ký được sau khi Ice trả lời D1, D2 và các mục dưới xong.

## A. Sửa chữ spine

- A1. Tiêu đề và `Binds` hẹp hơn phạm vi Rule 5; C8 không có lý do ghi ra.
- A2. `Prevents` chỉ nêu lỗi đầu tiên, thiếu TM và quick-add.
- A3. Rule 2 thiếu chỗ dựa thứ hai: `load_same_length_pair_rows` lọc độ dài byte.
- A4. Rule 3 đọc như mọi đường đều phân xử; `chapter_span_counts_in`, `harvest_work` đọc thẳng `find_terms`.
- A5. Rule 4 nói không đổi `En` rộng hơn thực tế; thiếu lý do bảng không phạm luật không gộp nguồn, AD-25, AD-10, FR112.
- A6. "Không ghi đầu ra gấp xuống đĩa" là bất biến nền, đang là mệnh đề phụ.
- A7. Thiếu giới hạn: 3.701/7.362 cặp một chữ có một vế ngoài BMP không vào bảng.
- A8. Thiếu giới hạn: lọc ứng viên không so với hàng chờ `glossary_candidate`.
- A9. Phương án bị loại sai ở `託` và thiếu bốn phương án Rule 5 đã loại.
- A10. Bằng chứng đo trên bản băm của `afdc4205`, thiếu số TM.
- A11. 🔵 ở AD-17 sai vì điền sẵn FR58 vẫn nguyên văn.
- A12. AD-18 không có dòng trỏ về thứ tự tầng ở `resolve_overlaps`.
- A13. Thiếu mục Test canh theo khuôn AD-61, AD-62.

## B. Sửa chữ PRD

- B1. FR51 và hàng addendum đổi chữ không kèm 🔵.
- B2. FR61 chưa nói n-gram tính trên ký tự đã gấp.
- B3. Hàng addendum đọc như mọi khớp chính xác đều gấp.

## C. Mã và test

- C1. Ước lượng FR137 (`estimate.rs:53`) so nguyên văn, lượt quét thật so sau gấp.
- C2. Thứ tự nguyên văn trước tầng chưa có ca canh.
- C3. Ranh giới trên văn bản phồn chưa có ca.
- C4. Bất biến không ghi đầu ra gấp xuống đĩa chưa có cổng.

## D. Ice quyết

- D1. Ranh giới jieba gần như không có tác dụng trên văn bản phồn. Ice chốt giữ và khai giới hạn.
- D2. Nguyên văn trước tầng làm ghi đè Tác phẩm chỉ hiệu lực trên một dạng chữ. Ice chốt giữ nguyên văn trước tầng.

## Lượt rà lại

Winston rà lại sau khi A, B, C áp dụng: `ký sau khi sửa các mục đã nêu`, nhận cách tách `EstimateExclusions` ở C1. Năm mục đã nêu đã áp dụng: hai câu ở Rule 4, `Binds`, quần thể số đo D1, bằng chứng chạy ở `plan-ad-63-theo-ra-soat-winston.md`. Review độc lập sau đó sửa thêm tiêu đề, Rule 5, Rule 6 và quần thể D1; Winston chưa đọc lại phần này.
