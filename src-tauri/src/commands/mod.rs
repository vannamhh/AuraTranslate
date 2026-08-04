//! Bề mặt IPC — các hàm `#[tauri::command]` mà frontend gọi qua.
//! Adapter thuần, KHÔNG chứa quy tắc nghiệp vụ (AD-1).
//!
//! ⚠️ Đừng nhầm với `src/commands/` phía frontend: đó là `CommandRegistry` của
//! giao diện (AD-34, FR22, Story 1.6). Hai thứ không ánh xạ một-một và không được gộp.
//!
//! Mọi lỗi vượt ranh giới IPC mang hình dạng `{ code, message_key, params, retryable }`
//! (AD-21). Kiểu là `crate::core::i18n::IpcError`; trường `message_key` có kiểu
//! `crate::core::i18n::MessageKey` — một danh mục đóng, nên một khoá không có trong
//! đó **không biên dịch được** thay vì lộ ra lúc chạy.
//!
//! ⛔ Đừng đặt `#[serde(rename_all = "camelCase")]` lên `IpcError` (thói quen viết
//! Tauri). Bốn tên trường là dây, không phải sở thích — `tests/ipc_contract.rs` khoá
//! chúng lại. ⛔ `params` mang DỮ LIỆU, không mang câu: Rust không bao giờ trả về văn
//! bản hiển thị.
//!
//! ⛔ **Dựng lỗi CHỈ qua `IpcError::new(code, message_key, params, retryable)`** — bốn
//! trường là riêng tư, và đó là chỗ duy nhất `message_key` gặp `params`. Một chỗ gọi
//! quên tham số mà chuỗi đòi (`params: BTreeMap::new()` cho một khoá có `{path}`) biên
//! dịch sạch và qua mọi phép kiểm còn lại — rồi đặt nguyên văn `{path}` lên màn hình
//! người dùng. Danh sách tham số bắt buộc khai cạnh khoá trong `message_keys!`.
