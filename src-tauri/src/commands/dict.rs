//! Bề mặt IPC đọc âm Hán Việt cho tab Hán Việt — Story 1.16, AC4.
//!
//! ─────────────────────────────────────────────────────────────────────────────
//! 🔴 VÌ SAO MỘT COMMAND RIÊNG, ⛔ KHÔNG GỘP VÀO `read_open_chapter`
//! ─────────────────────────────────────────────────────────────────────────────
//! `commands::chapter::read_open_chapter` trả `source_text` — dữ liệu bắt buộc cho MỌI
//! Chương. Gom âm Hán Việt vào cùng lượt trả về đó buộc MỌI lần mở Chương (kể cả nguồn
//! **tiếng Anh**, AC3 — ⛔ không tab Hán Việt) tính toán qua tập ba tệp `.db` từ điển. Tách
//! thành một command riêng để webview chỉ gọi nó **khi `source_lang == "zh"`** — cùng
//! nguyên tắc "adapter mỏng, chỗ gọi quyết khi nào cần" đã áp cho `bootstrap_config`/
//! `put_config` (Story 1.8).
//!
//! ⛔ **Không cổng thứ tư** (AD-2): command này chỉ là vỏ IPC gọi xuống
//! [`crate::core::dict::lookup_han_viet`] — cổng vẫn đúng ba: `DictionarySource` (+ `Store`
//! + `ReadOnlyDb`, hai cổng còn lại của bộ ba AD-2).
//!
//! ⚠️ Mọi chuỗi trong tệp này viết KHÔNG DẤU — `scripts/check-i18n.mjs` Kiểm A quét
//! `src-tauri/**/*.rs`.

use crate::core::dict::{DictLayers, HanVietLookup, lookup_han_viet};

/// Đọc âm Hán Việt cho `chars` — **hàm thuần, đây là thứ test gọi**.
///
/// `layers = None` đối xử **giống hệt** một tập lớp rỗng — `DictLayers` luôn được
/// `app.manage(...)` ở `setup()` (kể cả khi rỗng, xem `lib.rs::open_dict_layers`), nên
/// nhánh `None` chỉ xảy ra nếu cấu hình `setup()` sai, và hành vi đúng của nó là **giống
/// ca "0 lớp"** — một trạng thái BÌNH THƯỜNG có tên (AD-25), ⛔ không phải một lỗi.
///
/// ⛔ **Không có nhánh lỗi**: một lớp hỏng lúc tra được [`lookup_han_viet`] xử lý bằng cách
/// coi lớp đó không đóng góp gì (cùng luật `lookup_grouped`), ⛔ không làm hỏng cả lượt.
pub fn read_han_viet(layers: Option<&DictLayers>, chars: &[String]) -> HanVietLookup {
    let refs: Vec<&str> = chars.iter().map(String::as_str).collect();
    let empty = DictLayers::empty();
    lookup_han_viet(layers.unwrap_or(&empty), &refs)
}

/// Một vỏ `#[tauri::command]`. ⛔ **Không một quy tắc nào sống ở đây.**
pub mod wire {
    use super::{DictLayers, HanVietLookup};

    /// Vỏ IPC của [`super::read_han_viet`].
    #[tauri::command]
    pub fn read_han_viet(app: tauri::AppHandle, chars: Vec<String>) -> HanVietLookup {
        use tauri::Manager as _;

        let managed = app.try_state::<DictLayers>();
        super::read_han_viet(managed.as_deref(), &chars)
    }
}
