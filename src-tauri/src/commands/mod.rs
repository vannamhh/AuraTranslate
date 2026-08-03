//! Bề mặt IPC — các hàm `#[tauri::command]` mà frontend gọi qua.
//! Adapter thuần, KHÔNG chứa quy tắc nghiệp vụ (AD-1).
//!
//! ⚠️ Đừng nhầm với `src/commands/` phía frontend: đó là `CommandRegistry` của
//! giao diện (AD-34, FR22, Story 1.6). Hai thứ không ánh xạ một-một và không được gộp.
//!
//! Mọi lỗi vượt ranh giới IPC mang hình dạng `{ code, message_key, params, retryable }`
//! (AD-21) — `message_key` lấy từ danh mục ở `core::i18n`.
