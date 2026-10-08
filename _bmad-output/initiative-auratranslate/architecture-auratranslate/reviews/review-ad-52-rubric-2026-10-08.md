# Review rubric AD-52 (2026-10-08)

Phạm vi đọc: AD-52 (spine dòng 848-880), AD-38, AD-47, hàng Deferred, hàng C8, ER, brief, `commands/chapter.rs` (gộp: dòng 872-930), `commands/segment/regroup.rs` (rebase `reading_mark`).

Verdict: ĐẠT CÓ SỬA — hướng đúng (hai bảng, UNIQUE, một người ghi, bất biến, không chạm AD-16/38/47, trả lời cả 5 câu brief, ER và C8 đã cập nhật), nhưng còn 2 lỗ divergence và 1 mâu thuẫn với mã.

## Findings

1. HIGH — Mục 5 mâu thuẫn với đường gộp thật. `merge_chapter_into_previous` XOÁ hàng `chapter` nguồn (chapter.rs:930, sau khi xoá `chapter_position`:929). Project.db không bật FK, không CASCADE. Mục 5 chỉ nói "đặt `stale_at`, không xoá": bản của Chương bị gộp mồ côi vĩnh viễn, ER `CHAPTER ||--o| REVIEW_CHAPTER` sai, và `UNIQUE(chapter_id)` không bảo vệ gì. Sửa: nêu rõ trường hợp Chương bị xoá: xoá `review_chapter` + `review_row` của nó trong giao dịch gộp (cùng khuôn `chapter_position`), chỉ Chương sống sót (gộp) và hai nửa (tách) được đặt `stale_at`; kèm cặp test đối chứng ở 8.9/8.12.

2. HIGH — Q5 chỉ trả lời nửa. Mục 7 nói thu hoạch "đọc từ các bảng này", nhưng FR54 cần cặp (nguồn ↔ chữ reviewer đổi) và FR95 đòi chạy khi chưa mở Review Mode. Cặp chỉ có từ alignment, mà AD không nói alignment chạy lúc nhập hay lúc mở Review Mode, và kết quả có bền hay không. 8.10, 8.11, 8.14 sẽ tự chọn. Sửa: chốt (a) alignment tự chạy trong luồng nhập/thu hoạch, không phụ thuộc Review Mode; (b) kết quả được lưu (bền) hay tính lại; (c) tối thiểu hợp đồng cardinality nhiều-nhiều và cách biểu diễn "không khớp" mà 8.11 và 8.14 cùng đọc.

3. MEDIUM — Mục 6 để Deferred những thứ không chỉ là thuật toán. Trạng thái "đã chấp nhận từng thay đổi" (FR94, 8.12-8.13) và nối tay có nằm trong kết quả alignment hay bảng khác? Nếu nằm trong đó thì mâu thuẫn mục 4 về "bất biến" ở tinh thần, và 8.10/8.13 sẽ chia nhau. Ngoài ra "sống sót khi segment về hưu theo khuôn READING_MARK" thực chất là rebase tường minh trong regroup.rs:341 (UPDATE `navigation_segment_id`), không tự sống sót. Sửa: nói rõ cơ chế (rebase trong giao dịch retire hoặc id không tái dùng + tra chuỗi), nêu ai sở hữu trạng thái chấp nhận, và sửa hàng Deferred "Thuật toán alignment" thành chỉ thuật toán.

4. MEDIUM — Hạt của `review_row` chưa cố định. Một hàng là một hàng bảng, một đoạn hay một ô nhiều đoạn? `.docx` ô nhiều đoạn ở bảng nhiều hàng (AD-38 chỉ chặn bảng một hàng) nối thế nào? `.md` một đoạn một hàng? Người ghi (8.9) và người đọc (8.10, 8.14) có thể lệch. Thêm một dòng quy tắc hạt + dấu nối đoạn, và thứ tự `alt`/`caption` so với ảnh.

5. LOW — Thiếu quy ước: `imported_at`/`stale_at` ISO-8601 UTC (Consistency Conventions "Ngày giờ"); xoá bản cũ phải xoá `review_row` tường minh vì không CASCADE; mục "Phương án bị loại: Xoá cùng giao dịch" diễn đạt mơ hồ (xoá cái gì, khi nào), nên viết lại.

Kiểm khác: không làm yếu AD-16 (chỉ văn bản thuần), AD-38 (cổng vẫn chạy trước mọi ghi, mục 2 chỉ ghi sau xác nhận), AD-46 (lý do loại `segment_id`), AD-47 ③ (lượt nhập không ghi `segment`).
