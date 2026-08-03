# Lens phiên bản & đối chiếu thực tế — vòng cập nhật FR122–FR131

**Verdict: đạt.** Không crate nào được ghim vào Stack lần này, nên bề mặt khẳng định rất hẹp. Ba thư viện được nêu tên đều đã tra crates.io ngày 2026-08-03, không lấy từ trí nhớ.

| Crate được nêu | Phiên bản tra được | Giấy phép | GPLv3? | Ghi chú |
|---|---|---|---|---|
| `chardetng` | **1.0.0** | Apache-2.0 OR MIT | ✅ | Bộ dò bảng mã của Firefox cho nội dung web cũ — đúng bài toán GBK/Big5 của FR126 |
| `encoding_rs` | **0.8.35** | (Apache-2.0 OR MIT) AND BSD-3-Clause | ✅ | ~95 M lượt tải gần đây; nền của toàn hệ sinh thái |
| `dom_smoothie` | **0.18.0** | MIT | ✅ | Cập nhật 2026-06-07, ~214 k lượt tải. Cổng Readability **còn được bảo trì** |
| `readability` | 0.3.0 | MIT | ✅ | **Đứng im từ 2023-12-20.** Nhiều lượt tải hơn nhưng loại khỏi vị trí nền vì lý do bảo trì |

## Nhận xét

**Không có khẳng định nào chưa kiểm.** Cả ba hàng Deferred mới đều nói rõ đây là **ứng viên**, không phải lựa chọn đã chốt, và điều kiện mở lại là một mũi thăm dò trên dữ liệu thật.

**`reqwest` cho `Fetcher`:** đã có trong Stack (ghim *"mới nhất lúc dựng"*, Apache-2.0 OR MIT). Hàng Deferred nói rõ **chưa chốt** vì `Fetcher` có nhu cầu khác `TranslationProvider` — theo dõi chuyển hướng để cưỡng chế allowlist, giới hạn kích thước, timeout. Đây là sự thận trọng đúng chỗ, không phải né tránh.

**Rủi ro thư viện không lan vào spine:** AD-39, AD-40, AD-41 **không ràng buộc crate nào** — bất biến là thứ tự pipeline, ranh giới `Fetcher` \| `Extractor`, và allowlist. Đổi thư viện không đụng AD nào. Cùng khuôn đã dùng cho AD-37/AD-38.

**Một điểm nên theo dõi:** `dom_smoothie` ~214 k lượt tải là **mức khiêm tốn** so với các crate khác trong Stack. Không phải cờ đỏ với một tính năng có đường sửa tay bắt buộc (FR123), nhưng nếu mũi thăm dò cho tỉ lệ bóc sai cao thì phương án **tự viết** phải được cân nhắc nghiêm túc chứ không mặc định đi tìm crate thứ ba.
