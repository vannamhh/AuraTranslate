//! Crate root của AuraTranslate.
//!
//! Bố cục `lib.rs` + `main.rs` là quy ước của chính framework Tauri v2, và nó là
//! điều kiện để `tests/` `use auratranslate_lib::…` được. Đừng viết test vào `main.rs`.
//!
//! ⚠️ `mod core` là module miền của dự án, KHÔNG phải crate `core` của Rust.
//! Trong crate này luôn viết `crate::core::…`; đừng viết `use core::…` — nó nhập
//! nhằng với crate chuẩn.

pub mod commands;
pub mod core;
pub mod ports;

/// Tên biến môi trường bật Kiểm 3 của Story 1.2 (phạm vi asset protocol).
///
/// Bật thì ứng dụng chạy self-check trong webview, in kết quả ra stdout rồi **thoát
/// với mã 0/1**. Đó là thứ Story 1.3 gắn vào pipeline được; một phép kiểm chỉ hiện
/// kết quả trên màn hình thì không cưỡng chế được gì.
///
/// ⚠️ Móc này **chỉ tồn tại trong bản debug** (`#[cfg(debug_assertions)]`). Nó đóng
/// toàn bộ cửa sổ rồi thoát tiến trình — một móc như vậy không có việc gì trong bản
/// phát hành, và mã self-check phía frontend cũng không vào bundle release
/// (`App.vue` chỉ `import()` động khi `VITE_SCOPE_SELFTEST=1`). Hai đầu phải đối xứng.
pub const SCOPE_SELFTEST_ENV: &str = "AURA_SCOPE_SELFTEST";

/// Tên event mà self-check của frontend phát về. Khớp với `src/selftest/scopeCheck.ts`.
pub const SCOPE_SELFTEST_EVENT: &str = "selftest:scope-check";

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    #[cfg(debug_assertions)]
    let selftest = std::env::var(SCOPE_SELFTEST_ENV).as_deref() == Ok("1");

    #[allow(unused_mut)]
    let mut builder = tauri::Builder::default();

    #[cfg(debug_assertions)]
    {
        builder = builder.setup(move |app| {
            if selftest {
                wire_scope_selftest(app.handle());
            }
            Ok(())
        });
    }

    builder
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

/// Nghe kết quả Kiểm 3 từ webview, in ra stdout, rồi thoát với mã tương ứng.
///
/// Chỉ nối khi `AURA_SCOPE_SELFTEST=1`, và chỉ tồn tại trong bản debug.
#[cfg(debug_assertions)]
fn wire_scope_selftest(handle: &tauri::AppHandle) {
    use tauri::{Listener as _, Manager as _};

    let app = handle.clone();
    handle.listen(SCOPE_SELFTEST_EVENT, move |event| {
        let payload = event.payload();

        // Đọc verdict từ JSON đã parse, KHÔNG so chuỗi con trên payload thô: một
        // `detail` chứa đúng chuỗi đó, hay một serializer chèn khoảng trắng
        // (`{"verdict": "PASS"}`), là đủ để mã thoát nói ngược lại dòng in ra.
        let parsed = serde_json::from_str::<serde_json::Value>(payload).ok();

        let text = parsed
            .as_ref()
            .and_then(|v| v.get("text").and_then(|t| t.as_str()).map(str::to_owned))
            .unwrap_or_else(|| payload.to_owned());

        // Không parse được ⇒ FAIL. Một payload không đọc được không bao giờ là "đạt".
        let verdict_is_pass = parsed
            .as_ref()
            .and_then(|v| v.get("verdict").and_then(|t| t.as_str()))
            .map(|v| v == "PASS")
            .unwrap_or(false);

        println!("{text}");

        let code = if verdict_is_pass { 0 } else { 1 };
        for (_, window) in app.webview_windows() {
            let _ = window.close();
        }
        app.exit(code);
    });
}
