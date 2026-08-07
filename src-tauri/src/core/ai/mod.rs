//! Nhóm năng lực C6, C7 — dịch bằng AI và Smart RAG Injector.
//!
//! **KHÔNG module nào ngoài `ai/` được import `ai/`** (AD-13). Đây là ranh giới
//! cứng nhất của cây nguồn: gỡ trọn `ai/` ra thì phần còn lại của ứng dụng vẫn phải
//! biên dịch và chạy được. Test cưỡng chế ranh giới này thuộc **Story 4.1**.
//!
//! Lắp prompt là một hàm thuần (AD-14). Streaming qua Channel, không tự kết nối lại (AD-22).
//! Khoá API chỉ tồn tại trong Rust, lấy qua crate `keyring` trực tiếp (AD-29).
//!
//! Crate dành cho module này: `reqwest` · `keyring`.
