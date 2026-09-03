//! Nhập từ web. Hai nửa, hai ranh giới cứng (AD-40, AD-41):
//!
//! - `Fetcher` = ĐIỂM RA MẠNG THỨ BA của toàn ứng dụng (AD-15). Nó chỉ tải, KHÔNG
//!   phân tích nội dung. Canh bởi allowlist một-lần-nhập (AD-41).
//! - `Extractor` KHÔNG BAO GIỜ chạm mạng.
//!
//! Nội dung nhập từ ngoài không bao giờ render thành HTML (AD-16).
//!
//! Crate dành cho module này: `reqwest` (dùng chung với `core::ai`).
//!
//! 🔵 **2026-09-03 (Story 6.1, mũi thăm dò) — ghim thêm hai crate mới cho `Extractor`.**
//! `dom_smoothie` 0.18.0 (bóc nội dung chính, FR123) và `chardetng` 1.0.0 (dò bảng mã,
//! FR126); `encoding_rs` 0.8.35 cũng được khai tường minh nhưng đã bắc cầu qua `reqwest`
//! từ trước, không phải một crate mới. Rà NFR15 + số đo trên dữ liệu thật ở
//! `ARCHITECTURE-SPINE.md` §Deferred (ba hàng đóng 2026-09-03) và
//! `_bmad-output/implementation-artifacts/6-1-ban-do/`. Module này vẫn CHƯA có mã —
//! `Extractor`/`Fetcher` thật là Story 6.2/6.3/6.9.
