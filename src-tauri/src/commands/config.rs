//! **Hai `#[tauri::command]` đầu tiên của dự án** — Story 1.8, AC5.
//!
//! Adapter thuần (AD-1): quy tắc phân giải hai tầng sống ở [`crate::core::scope`], đường
//! đọc/ghi sống ở [`crate::core::store`]. Ở đây chỉ có việc lấy `State`, đổi hình dạng, và
//! bọc lỗi thành [`IpcError`].
//!
//! ─────────────────────────────────────────────────────────────────────────────
//! 🔴 HÀM THUẦN THEO `Option<&Store>`, `#[tauri::command]` CHỈ LÀ VỎ — §Quyết định #6
//! ─────────────────────────────────────────────────────────────────────────────
//! [`bootstrap_config`] và [`put_config`] nhận `Option<&Store>` và **là đường sản phẩm
//! thật** — không phải một fixture cho test. Hai `#[tauri::command]` ngay dưới chúng chỉ
//! làm đúng một việc: lấy `State<Store>` qua `try_state` rồi gọi xuống.
//!
//! Đó là điều kiện để chữa `ipc_error_wire_shape` cho tử tế: `deferred-work.md:49` ghi
//! rằng phép kiểm đó là một **mệnh đề vòng** *(nó quét chính fixture nó tự dựng)*, và
//! đồng thời cấm dựng một command giả để đóng nợ. Một hàm thuần mà command thật bọc lại
//! thì **không phải** command giả — nó là chính thứ chạy trên máy người dùng.
//!
//! ─────────────────────────────────────────────────────────────────────────────
//! 🔴 ⛔ `State<Store>` KHÔNG PHẢI LÚC NÀO CŨNG CÓ — và nhánh vắng mặt là cả lý do tồn tại
//! ─────────────────────────────────────────────────────────────────────────────
//! `lib.rs:84-116` ghi chẩn đoán rồi **đi tiếp** khi mở kho thất bại, nên `app.manage(store)`
//! **có thể chưa từng chạy**. Một `state::<Store>()` thẳng tay sẽ panic — và `panic = "abort"`
//! giết luôn tiến trình. Dùng `try_state`, và nhánh `None` chính là bề mặt lỗi mà
//! `deferred-work.md:177` chờ: từ hôm nay một `$APPDATA` không ghi được **nói ra** thay vì
//! chỉ ra `stderr`.
//!
//! ─────────────────────────────────────────────────────────────────────────────
//! ⛔ KHÔNG KHOÁ `MessageKey` MỚI, KHÔNG CHUỖI `vi.json` MỚI — §Quyết định #7
//! ─────────────────────────────────────────────────────────────────────────────
//! Mọi lỗi hai hàm này phát ra đều là lỗi **kho**, và cả năm khoá đã có từ Story 1.7 kèm
//! `From<StoreError> for IpcError` và test `every_store_error_converts_to_a_complete_ipc_error`.
//! `core::scope::ScopeError` là **lỗi lập trình** — nó ⛔ không `impl From<..> for IpcError`
//! và ⛔ không bao giờ vượt ranh giới này.
//!
//! ⚠️ Mọi chuỗi trong tệp này viết KHÔNG DẤU — `scripts/check-i18n.mjs` Kiểm A quét
//! `src-tauri/**/*.rs` và `src/commands/**` không nằm trong danh sách miễn trừ.

use std::collections::BTreeMap;

use serde::Serialize;

use crate::core::i18n::IpcError;
use crate::core::scope;
use crate::core::store::{Store, StoreError, StoreKind};

/// Cấu hình mà frontend cần **trước** lượt render đầu tiên.
///
/// ⛔⛔ **KHÔNG `#[serde(rename_all = "camelCase")]`.** Thói quen viết Tauri là đặt nó lên
/// mọi struct qua IPC cho hợp phong cách JS; ở đây nó biến `layout_presets` thành
/// `layoutPresets` và chỗ đọc nhận `undefined`. Cùng luật, cùng lý do với [`IpcError`] —
/// bốn tên trường của AD-21 là **dây**, không phải sở thích. Khoá trên dây là `snake_case`.
///
/// ⚠️ `BTreeMap`, ⛔ không `HashMap`: thứ tự khoá ổn định thì test so JSON mới ổn định qua
/// từng lượt chạy. Cùng lý do mà `IpcError::params` là `BTreeMap`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct BootstrapConfig {
    /// Theme đang chọn. Rơi về `core::scope::DEFAULT_THEME` khi chưa ai chọn gì.
    pub theme: String,
    /// Chế độ cuối cùng người dùng ở. Rơi về `core::scope::DEFAULT_MODE`.
    pub mode: String,
    /// Hợp âm phím tắt theo id thao tác. Rỗng = dùng hợp âm mặc định của `installCommands`.
    pub shortcuts: BTreeMap<String, String>,
    /// Preset bố cục đã đặt tên. Nội dung của chúng là **Story 1.14**.
    pub layout_presets: BTreeMap<String, String>,
}

/// Kho vắng mặt ⇒ lỗi *mở kho*, và đó là câu đúng theo nghĩa đen.
///
/// 🔴 Đi qua `From<StoreError> for IpcError`, ⛔ không dựng `IpcError` bằng struct literal
/// và ⛔ không gọi `IpcError::new` với khoá gõ tay. Lý do: `IpcError::new` là chỗ **duy
/// nhất** `message_key` gặp `params`, và `From` là chỗ duy nhất một `StoreError` chọn
/// khoá của nó. Hai lần "duy nhất" đó chỉ có giá trị nếu không ai đi vòng.
///
/// ⚠️ `detail` mang lý do cho người chẩn đoán và ⛔ **không** đi vào `params` — AD-21 nói
/// `params` mang **dữ liệu**, không mang câu (Story 1.7 §Completion Notes #5).
fn store_is_missing() -> IpcError {
    StoreError::OpenFailed {
        store: StoreKind::Global,
        detail: "the global store was never managed; see lib.rs::open_global_store".to_owned(),
    }
    .into()
}

/// Nạp cấu hình khởi động từ `global.db` — **hàm thuần, đây là thứ test gọi**.
///
/// `store = None` ⇒ [`IpcError`] mang `code = "store.open_failed"`,
/// [`MessageKey::StoreOpenFailed`], `params = {"store": "global"}`, `retryable = false`.
///
/// # Lỗi
/// - kho vắng mặt ⇒ `store.open_failed`;
/// - đường đọc trượt ⇒ `store.read_failed` *(qua `From<StoreError>`)*.
///
/// ⛔ Không nhánh nào sinh ra một [`IpcError`] **không** dẫn xuất từ `StoreError` —
/// `tests/scope_contract.rs::every_command_error_comes_from_the_store_vocabulary` canh
/// mệnh đề đó.
pub fn bootstrap_config(store: Option<&Store>) -> Result<BootstrapConfig, IpcError> {
    let store = store.ok_or_else(store_is_missing)?;
    let config = scope::load_global_config(store)?;

    Ok(BootstrapConfig {
        theme: config.theme().to_owned(),
        mode: config.mode().to_owned(),
        shortcuts: config.shortcuts(),
        layout_presets: config.layout_presets(),
    })
}

/// Ghi một giá trị cấu hình xuống tầng Global — **hàm thuần**.
///
/// `kind` đến từ bên kia ranh giới nên nó là dữ liệu **không tin được**; phép phân giải và
/// phép từ chối nằm ở [`scope::save_value`], ⛔ không ở đây. Adapter không phán xét.
///
/// # Lỗi
/// kho vắng mặt ⇒ `store.open_failed`; `kind` lạ hoặc đường ghi trượt ⇒ `store.write_failed`.
pub fn put_config(
    store: Option<&Store>,
    kind: &str,
    key: &str,
    value: &str,
) -> Result<(), IpcError> {
    let store = store.ok_or_else(store_is_missing)?;
    scope::save_value(store, kind, key, value)?;
    Ok(())
}

/// Hai vỏ `#[tauri::command]`. ⛔ **Không một quy tắc nào sống ở đây.**
///
/// ⚠️ Module lồng chứ không phải hai hàm cạnh nhau, và đó là một ràng buộc chứ không phải
/// một cách sắp xếp: **tên command trên dây là tên hàm**. Frontend gọi
/// `invoke('bootstrap_config')`, nên hàm `#[tauri::command]` phải mang đúng tên đó — mà
/// tên đó đã thuộc về hàm thuần ở trên. Một hậu tố `_command` sẽ đổi tên trên dây và
/// `invoke` sẽ không tìm thấy gì.
///
/// ⛔ Đừng đảo hướng: hàm thuần là **đường sản phẩm**, vỏ là thứ có thể bỏ đi trong test.
pub mod wire {
    use super::{BootstrapConfig, IpcError, Store};

    /// Vỏ IPC của [`super::bootstrap_config`].
    ///
    /// ⚠️ `try_state`, ⛔ không `state()` — xem doc-comment của module về `panic = "abort"`.
    #[tauri::command]
    pub fn bootstrap_config(app: tauri::AppHandle) -> Result<BootstrapConfig, IpcError> {
        use tauri::Manager as _;

        let managed = app.try_state::<Store>();
        super::bootstrap_config(managed.as_deref())
    }

    /// Vỏ IPC của [`super::put_config`].
    #[tauri::command]
    pub fn put_config(
        app: tauri::AppHandle,
        kind: String,
        key: String,
        value: String,
    ) -> Result<(), IpcError> {
        use tauri::Manager as _;

        let managed = app.try_state::<Store>();
        super::put_config(managed.as_deref(), &kind, &key, &value)
    }
}
