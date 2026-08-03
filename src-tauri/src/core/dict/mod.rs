//! Tra cứu từ điển — ba nhánh truy vấn tiếng Trung (AD-26).
//!
//! KHÔNG tồn tại bước hợp nhất nguồn (AD-19): mỗi kết quả luôn mang `source` của nó.
//! Mỗi lớp gỡ rời là một file `.db` độc lập, chỉ đọc (AD-10, AD-25).
//!
//! Crate dành cho module này: `rusqlite` (đọc `.db`) — dùng chung cài đặt với `core::store`.
