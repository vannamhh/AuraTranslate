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
//!
//! ─────────────────────────────────────────────────────────────────────────────
//! KHUÔN ĐÃ CHỐT (Story 1.8) — **hàm thuần trước, `#[tauri::command]` là vỏ**
//! ─────────────────────────────────────────────────────────────────────────────
//! Mỗi bề mặt IPC gồm hai lớp, và thứ tự phụ thuộc chỉ đi một chiều:
//!
//! 1. một **hàm thuần** nhận `Option<&Store>` *(hoặc thứ tương đương đã phân giải)* —
//!    đây là đường sản phẩm, và đây là thứ `tests/**` gọi được **mà không cần webview**;
//! 2. một `#[tauri::command]` mỏng trong module `wire` chỉ lấy `State` qua `try_state`
//!    rồi gọi xuống lớp 1.
//!
//! 🔴 `try_state`, ⛔ **không** `state()`: `lib.rs::open_global_store` ghi chẩn đoán rồi
//! **đi tiếp** khi mở kho thất bại, nên `app.manage(store)` có thể chưa từng chạy. Một
//! `state::<Store>()` thẳng tay panic, và `panic = "abort"` giết cả tiến trình.
//!
//! ⚠️ **Tên command trên dây là tên hàm** — nên vỏ phải mang đúng cái tên mà `invoke()`
//! gọi, và đó là lý do nó sống trong một module lồng thay vì mang một hậu tố.

pub mod chapter;
pub mod config;
pub mod dict;
pub mod project;
