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
//! `dom_smoothie` 0.18.0 (bóc nội dung chính, FR123, chủ Ở LẠI module này — Story 6.9) và
//! `encoding_rs` 0.8.35 (khai tường minh nhưng đã bắc cầu qua `reqwest` từ trước, không
//! phải một crate mới). Rà NFR15 + số đo trên dữ liệu thật ở `ARCHITECTURE-SPINE.md`
//! §Deferred (ba hàng đóng 2026-09-03) và `_bmad-output/implementation-artifacts/6-1-ban-do/`.
//! Module này vẫn CHƯA có mã — `Extractor`/`Fetcher` thật là Story 6.9/6.7.
//!
//! 🔵 **SỬA 2026-09-04 (Story 6.3) — chủ của bộ dò bảng mã (FR126) ĐÃ ĐỔI, khỏi module
//! này.** Câu trên từng khai `chardetng` là crate của `Extractor` ở ĐÂY — sai ngay từ đầu:
//! bộ dò bảng mã không bóc nội dung chính (`Extractor`), nó là bước 1 của chuỗi AD-39
//! (giải mã), và bước đó sống ở `core::segment::pipeline`. Bộ dò thật (`chardetng`,
//! `sniff_bom`/`detect`/`render_candidates`) sống ở `core::segment::encoding` — module NÀY
//! không nêu tên `chardetng` nữa, và vẫn CHƯA có mã (`Extractor`/`Fetcher` là Story 6.9/6.7,
//! không đụng ở đây).
