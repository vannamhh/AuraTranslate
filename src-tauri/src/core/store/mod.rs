//! Tầng ghi dữ liệu: `Writer` nối tiếp + `Reader` pool + checkpoint (AD-11, AD-12).
//!
//! MỘT writer duy nhất cho mỗi kho ghi được (AD-11). Thời điểm checkpoint là quyết
//! định của ứng dụng, không phó mặc SQLite (AD-12). Lược đồ có phiên bản; mở tiến,
//! không bao giờ mở lùi (AD-30).
//!
//! Đường dẫn `$APPDATA` LUÔN lấy qua `app.path().app_data_dir()` — không viết cứng.
//! Đây là chỗ NFR14 (hành vi tương đương hai nền tảng) hỏng đầu tiên.
//!
//! Crate dành cho module này: `rusqlite` (feature `bundled`) · `libsqlite3-sys`.
//! Story 1.7 sở hữu nội dung.
