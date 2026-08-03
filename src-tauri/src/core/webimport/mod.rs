//! Nhập từ web. Hai nửa, hai ranh giới cứng (AD-40, AD-41):
//!
//! - `Fetcher` = ĐIỂM RA MẠNG THỨ BA của toàn ứng dụng (AD-15). Nó chỉ tải, KHÔNG
//!   phân tích nội dung. Canh bởi allowlist một-lần-nhập (AD-41).
//! - `Extractor` KHÔNG BAO GIỜ chạm mạng.
//!
//! Nội dung nhập từ ngoài không bao giờ render thành HTML (AD-16).
//!
//! Crate dành cho module này: `reqwest` (dùng chung với `core::ai`).
