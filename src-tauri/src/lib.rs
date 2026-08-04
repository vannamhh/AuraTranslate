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

/// Tên tệp kho toàn cục dưới `$APPDATA`. Xem [`open_global_store`].
const GLOBAL_DB_FILE: &str = "global.db";

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    #[cfg(debug_assertions)]
    let selftest = std::env::var(SCOPE_SELFTEST_ENV).as_deref() == Ok("1");

    let builder = tauri::Builder::default().setup(move |app| {
        #[cfg(debug_assertions)]
        if selftest {
            wire_scope_selftest(app.handle());
        }

        open_global_store(app);
        Ok(())
    });

    // ⚠️ `build(ctx)?.run(callback)` thay cho `run(ctx)`, và phép đổi này KHÔNG có tác
    // dụng phụ nào: `Builder::run` trong `tauri-2.11.5/src/app.rs:2449-2452` **chính là**
    // `self.build(context)?.run(|_, _| {})`. Thứ duy nhất thêm vào là callback dưới đây.
    let app = tauri::Builder::build(builder, tauri::generate_context!())
        .expect("error while building tauri application");

    app.run(|handle, event| {
        if matches!(event, tauri::RunEvent::Exit) {
            close_global_store(handle);
        }
    });
}

/// Mở `$APPDATA/global.db` và đưa nó vào state của ứng dụng (AD-11, AC3).
///
/// ─────────────────────────────────────────────────────────────────────────────
/// VÌ SAO MỞ Ở `setup()` CHỨ KHÔNG PHẢI LÚC DÙNG LẦN ĐẦU
/// ─────────────────────────────────────────────────────────────────────────────
/// AC3 nói *"`global.db` **khi khởi tạo**"*. Và mở sớm làm một `$APPDATA` không ghi được
/// lộ ra **ngay lúc khởi động** thay vì lúc người dùng đang gõ dở một câu — đó là toàn
/// bộ khác biệt giữa một thông báo khó chịu và một đoạn công sức đã mất.
///
/// ─────────────────────────────────────────────────────────────────────────────
/// 🔴 MỞ KHO TRƯỢT ⇒ GHI CHẨN ĐOÁN RÕ RỒI **ĐI TIẾP**, ⛔ KHÔNG CHẶN KHỞI ĐỘNG
/// ─────────────────────────────────────────────────────────────────────────────
/// Hai lý do, và lý do thứ hai là thứ sẽ đỏ ngay hôm nay nếu làm khác:
/// 1. Chưa có bề mặt nào để **nói** với người dùng — story này ⛔ không dựng
///    `#[tauri::command]` nào (xem story §*"Vì sao vẫn KHÔNG có command"*). Một
///    `return Err(…)` ở đây làm cửa sổ không mở, và người dùng nhận đúng thứ tệ nhất:
///    im lặng. Bề mặt hiển thị thuộc **Story 1.8**, nơi `From<StoreError> for IpcError`
///    đã dựng sẵn ở `core::store` chỉ còn phải nối dây.
/// 2. `scripts/check-scope.mjs` và `check-scope-bundled.mjs` chạy nhị phân rồi đọc dòng
///    `VERDICT:`. Một `setup()` trả `Err` làm **hai cổng của Story 1.2/1.3 đỏ** vì tầng
///    ghi dữ liệu, không vì phạm vi mà chúng canh.
///
/// ⚠️ Chuỗi chẩn đoán viết KHÔNG DẤU: `scripts/check-i18n.mjs` Kiểm A quét
/// `src-tauri/**/*.rs` và `lib.rs` **không** nằm trong danh sách miễn trừ.
fn open_global_store(app: &tauri::App) {
    use tauri::Manager as _;

    // ⛔ Không viết cứng `$APPDATA` — `app.path()` là đường duy nhất, và đây là chỗ NFR14
    // (hành vi tương đương hai nền tảng) hỏng đầu tiên nếu ai đó ghép chuỗi bằng tay.
    let dir = match app.path().app_data_dir() {
        Ok(dir) => dir,
        Err(err) => {
            eprintln!("store[global] cannot resolve the app data directory: {err}");
            return;
        }
    };

    if let Err(err) = std::fs::create_dir_all(&dir) {
        eprintln!(
            "store[global] cannot create {}: {err}",
            dir.display()
        );
        return;
    }

    let spec = crate::core::store::StoreSpec::global(dir.join(GLOBAL_DB_FILE));
    match crate::core::store::Store::open(spec) {
        Ok(store) => {
            app.manage(store);
        }
        Err(err) => {
            // `Display` của `StoreError` mang sẵn tên kho và lỗi thô. Story 1.8 nối cùng
            // giá trị đó lên giao diện qua `IpcError`; ở đây nó ra stderr.
            eprintln!("store[global] open failed, running without a data layer: {err}");
        }
    }
}

/// `RunEvent::Exit` ⇒ TRUNCATE lần cuối rồi dừng các luồng của kho (AD-12, AC4).
///
/// ⚠️ Đây là đường DUY NHẤT trong bản release mà `.db-wal` được cắt về 0. Nó có trần
/// thời gian (`Tuning::close_truncate_budget`) vì `AppHandle::exit` đi **qua vòng lặp sự
/// kiện** (`tauri-2.11.5/src/app.rs:574-580`) — nghĩa là móc self-check `#[cfg(debug_assertions)]`
/// cũng chạy callback này, và một `close()` treo ở đây làm `check:scope` /
/// `check:scope:bundled` đỏ vì một lý do không liên quan tới phạm vi chúng canh.
///
/// ⛔ Vẫn còn hở, và ghi thẳng ra thay vì đánh dấu đạt: `panic = "abort"` nghĩa là một
/// lần thoát cứng **không** đi qua đây. Xem `deferred-work.md`.
fn close_global_store(handle: &tauri::AppHandle) {
    use tauri::Manager as _;

    if let Some(store) = handle.try_state::<crate::core::store::Store>() {
        store.close();
    }
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
