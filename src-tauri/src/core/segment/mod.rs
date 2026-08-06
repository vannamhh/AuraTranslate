//! Segment: tách · gộp · tách đôi · về hưu (AD-3, AD-4, AD-5) + pipeline nhập (AD-39).
//!
//! ID segment bền và không bao giờ tái dùng (AD-3). Ranh giới tính một lần lúc nhập,
//! không bao giờ tính lại (AD-4). Gộp/tách là "về hưu + tạo mới", không sửa tại chỗ (AD-5).
//!
//! [`import`] — chuỗi pipeline nhập tối thiểu (Story 1.15): dán văn bản, kéo-thả, ô nhập
//! đường dẫn đổ vào **cùng một** hàm thuần. ⛔ **Không** tạo `segment` nào ở đây — tách
//! segment thật là Story 2.1 (FR23), câu chuyện khác hẳn "một Chương nguyên khối".

pub mod import;
