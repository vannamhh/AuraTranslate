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

pub mod import;
pub mod split;
