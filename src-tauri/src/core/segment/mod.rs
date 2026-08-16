//! Segment: tách · gộp · tách đôi · về hưu (AD-3, AD-4, AD-5) + pipeline nhập (AD-39).
//!
//! ID segment bền và không bao giờ tái dùng (AD-3). Ranh giới tính một lần lúc nhập,
//! không bao giờ tính lại (AD-4). Gộp/tách là "về hưu + tạo mới", không sửa tại chỗ (AD-5).
//!
//! [`import`] — chuỗi pipeline nhập tối thiểu (Story 1.15): dán văn bản, kéo-thả, ô nhập
//! đường dẫn đổ vào **cùng một** hàm thuần. Nó **không** tạo `segment` nào; nó chỉ dựng
//! `chapter.source_text` nguyên khối.
//!
//! [`split`] — bộ tách câu cấp CÂU và cờ kết đoạn (Story 2.1, FR23, AD-37). Hàm thuần,
//! chạy **một lần** lúc nhập; kết quả ghi xuống `project.db` và không đường mã nào tính lại
//! lúc nạp Chương (AC3). Đây là chỗ DUY NHẤT trong kho biết bảng chữ cái kết câu —
//! `tests/segment_boundary.rs` cưỡng chế mệnh đề đó trên cả cây nguồn.

//! [`omit`] — chốt LỌC cho mọi đầu ra (Story 2.5c, FR133, AC5). Một câu người dùng đã cắt
//! bỏ phải biến mất khỏi Chế độ đọc và mọi bản xuất; cả hai bề mặt đó chưa tồn tại, nên
//! module này dựng sẵn cái chốt thay vì giao nghĩa vụ cho trí nhớ. Xem doc-comment của nó —
//! nghĩa vụ FR133 hôm nay chỉ phát biểu MỘT CHIỀU và không AC nào ở phía tiêu thụ canh nó.

//! [`paragraph`] — bảng ba ca biên của AD-37 áp cho một **CẶP** cờ kết đoạn (Story 2.5d,
//! FR134, AD-46, AC3). Cùng khuôn và cùng lý do với [`omit`]: nghĩa vụ có thật, bề mặt tiêu
//! thụ *(gộp/tách tường minh — Story 2.8)* thì chưa. Dựng cái chốt thay vì giao cho trí nhớ.

pub mod import;
pub mod omit;
pub mod paragraph;
pub mod split;
