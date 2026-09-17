//! Bề mặt IPC cho cấu hình nhà cung cấp AI — Story 4.2 (FR68, AD-18) + Story 4.3 (FR65/FR67,
//! NFR11, khoá API trong keychain).
//!
//! Cùng khuôn `commands::cleanup`: hàm thuần nhận `Option<&Store>` (tầng Global) **cộng**
//! `Option<&OpenWork>` (tầng Tác phẩm) trước, `#[tauri::command]` chỉ là vỏ mỏng trong
//! [`wire`]. Năm lệnh: đọc năm trường đã phân giải hai tầng cộng trạng thái khoá API · ghi
//! một trường ở một tầng · trả một trường tầng Tác phẩm về kế thừa · ghi khoá API (Global-only)
//! · xoá khoá API (Global-only).
//!
//! ─────────────────────────────────────────────────────────────────────────────
//! 🔴 KHOÁ API — LUÔN GLOBAL, TẦNG BỊ TỪ CHỐI Ở ĐÂY, KHÔNG Ở `core/aiconfig/keychain.rs`
//! ─────────────────────────────────────────────────────────────────────────────
//! `core/aiconfig/keychain.rs` không nhận tham số tầng (một entry cho toàn ứng dụng — xem
//! doc-comment đầu tệp đó). [`ai_config_save_key`]/[`ai_config_delete_key`] vẫn nhận
//! `tier: AiConfigTier` — cùng chữ ký [`ai_config_save_field`] — để một yêu cầu tầng Tác
//! phẩm bị TỪ CHỐI TẠI ĐÂY (§Always spec 4.3: "refused at the command layer, not merely
//! hidden in the UI") thay vì chỉ không có nút bấm cho nó trên webview. `tier != Global`
//! trả `AiConfigKeyError::TierNotGlobal` TRƯỚC khi [`validate_key`] hay `keychain::set`/
//! `keychain::delete` chạy — "nothing written and nothing read" (I/O Matrix spec 4.3).
//!
//! ⚠️ Mọi chuỗi trong tệp này viết KHÔNG DẤU — `scripts/check-i18n.mjs` Kiểm A quét
//! `src-tauri/**/*.rs`.

use crate::commands::project::OpenWork;
use crate::core::aiconfig::{
    AiConfigField, AiConfigKeyError, AiConfigStoreError, AiConfigTier, ResolvedField, clear_field,
    keychain, resolve_two_tiers, validate_field, validate_key, write_field,
};
use crate::core::i18n::IpcError;
use crate::core::scope::ScopeResolver;
use crate::core::store::{Store, StoreError, StoreKind};

/// Kho `global.db` vắng mặt ⇒ lỗi *mở kho* — cùng khuôn `commands::cleanup::store_is_missing`.
fn store_is_missing() -> IpcError {
    StoreError::OpenFailed {
        store: StoreKind::Global,
        detail: "the global store was never managed; see lib.rs::open_global_store".to_owned(),
    }
    .into()
}

/// Định tuyến `&Store` theo tầng người dùng chọn — `Global` cần `global.db` đã quản lý;
/// `Work` cần một Tác phẩm đang mở.
fn store_for_tier<'a>(
    tier: AiConfigTier,
    global: Option<&'a Store>,
    open: Option<&'a OpenWork>,
) -> Result<&'a Store, IpcError> {
    match tier {
        AiConfigTier::Global => global.ok_or_else(store_is_missing),
        AiConfigTier::Work => {
            open.map(|w| &w.store).ok_or_else(|| AiConfigStoreError::WorkTierUnavailable.into())
        }
    }
}

/// Nhãn tầng trên dây — hình dạng OUTPUT, cùng khuôn `commands::project::CleanupRuleTierWire`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AiConfigTierWire {
    Global,
    Work,
}

impl From<AiConfigTier> for AiConfigTierWire {
    fn from(t: AiConfigTier) -> Self {
        match t {
            AiConfigTier::Global => AiConfigTierWire::Global,
            AiConfigTier::Work => AiConfigTierWire::Work,
        }
    }
}

/// Một trường đã phân giải — hình dạng trả về của [`ai_config_get`]. `shadowed` có mặt
/// (`Some`) đúng lúc `tier == Work` và Global cũng có giá trị cho trường đó
/// (`mockups/settings.html:172`/`:188`/`:200`).
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct AiConfigFieldWire {
    pub field: String,
    pub value: String,
    pub tier: AiConfigTierWire,
    pub shadowed: Option<String>,
}

impl AiConfigFieldWire {
    /// `resolved = None` ⇒ trường chưa được cấu hình ở tầng nào — I/O Matrix "No
    /// configuration anywhere": chuỗi rỗng, `tier: Global` (trạng thái nghỉ, không phải một
    /// ghi đè), `shadowed: None`.
    fn from_resolved(field: AiConfigField, resolved: Option<&ResolvedField>) -> Self {
        match resolved {
            Some(r) => Self {
                field: field.as_str().to_owned(),
                value: r.value.clone(),
                tier: r.tier.into(),
                shadowed: r.shadowed.clone(),
            },
            None => Self {
                field: field.as_str().to_owned(),
                value: String::new(),
                tier: AiConfigTierWire::Global,
                shadowed: None,
            },
        }
    }
}

/// Phong bì trả lời của [`ai_config_get`] — **không chỉ một `Vec<AiConfigFieldWire>` trần**,
/// và đó là chủ ý.
///
/// ─────────────────────────────────────────────────────────────────────────────
/// 🔴 VÌ SAO `work_tier_available` ĐI CÙNG, KHÔNG PHẢI MỘT TRUY VẤN RIÊNG
/// ─────────────────────────────────────────────────────────────────────────────
/// Cùng lý do `commands::glossary::QuickAddLookup` (xem doc-comment của struct đó): webview
/// phải học "có Tác phẩm nào đang mở không" **ngay trong đúng lượt gọi đã đọc
/// `OpenWorkState`**, không phải suy từ một tín hiệu UI khác (`currentMode`). Trước bản vá
/// này, `settingsState.ts::loadAiConfig` tính tầng ghi bằng `currentMode.value !== 'library'`
/// — một proxy chế độ giao diện KHÔNG tương đương `OpenWorkState`: `close_open_work` (duy
/// nhất xoá `OpenWorkState`) chỉ chạy ở nhánh `RunEvent::Exit`, không IPC nào đóng một Tác
/// phẩm, nên `OpenWorkState` có thể còn `Some` trong khi `currentMode` đã quay lại
/// `'library'`. Trường `work_tier_available` ở đây đóng đúng lỗ đó: nó tính TRỰC TIẾP từ
/// `Option<&OpenWork>` của lượt gọi này, không đi qua chế độ UI.
#[derive(Debug, Clone, serde::Serialize)]
pub struct AiConfigGetWire {
    /// `true` ⇔ có một Tác phẩm đang mở, tức tầng [`AiConfigTier::Work`] dùng được cho lượt
    /// `ai_config_save_field` tiếp theo.
    pub work_tier_available: bool,
    /// Năm trường, hai tầng đã phân giải — luôn đủ [`AiConfigField::ALL`], kể cả trường chưa
    /// ai cấu hình.
    pub fields: Vec<AiConfigFieldWire>,
    /// `Some(true)`/`Some(false)` ⇔ keychain hệ điều hành trả lời được thăm dò — KHÔNG BAO
    /// GIỜ giá trị thật (§Always spec 4.3: "the frontend may learn only configured / not
    /// configured"). `None` ⇔ keychain TỪ CHỐI trả lời (khoá, quyền bị chặn, không có kho
    /// nền tảng) — 🔴 **KHÔNG được đọc là `false`**: "chưa cấu hình" là một trạng thái bình
    /// thường của epic này, còn "không hỏi được" là một trạng thái KHÁC, và gộp hai trạng
    /// thái đó vào cùng một `false` là đúng lớp lỗi *SILENT EMPTINESS* mà `AGENTS.md` gọi
    /// tên là lớp lỗi trung tâm của kho này ("A value that can be UNKNOWN gets an
    /// `Option`/`NULL`, never a `0` or a `.unwrap_or(0)`") — webview phải vẽ `None` thành
    /// một trạng thái riêng (vd. "không kiểm tra được"), không phải render như "chưa cấu
    /// hình". Khoá là Global-only nên trường này không đi qua [`AiConfigFieldWire`]/
    /// `shadowed` — nó không có khái niệm tầng để mà che.
    pub key_configured: Option<bool>,
}

/// Đọc năm trường, hai tầng đã phân giải, cộng `work_tier_available` và `key_configured` —
/// **hàm thuần**. Luôn trả đủ năm trường (`AiConfigField::ALL`), kể cả trường chưa ai cấu
/// hình — **kể cả khi keychain từ chối trả lời thăm dò `configured`.**
///
/// ─────────────────────────────────────────────────────────────────────────────
/// 🔵 SỬA 2026-09-17 (rà lại sau Phase 2) — bản trước làm CẢ LƯỢT GỌI này thất bại khi
/// keychain từ chối trả lời (`keychain::configured().map_err(..)?`). Đó là đọc SAI I/O
/// Matrix spec 4.3, dòng "Keychain refuses": "the section stays usable **and other settings
/// still save**" — `ai_config_get` sập cả lượt thì năm trường plaintext (không đụng gì tới
/// keychain) cũng biến mất theo trên webview (`SettingsOverlay.vue` chỉ vẽ MỘT nhánh:
/// lỗi HOẶC nội dung, không cả hai). Sửa: gói lỗi vào `None` bằng `.ok()` — năm trường vẫn
/// trả đủ, `key_configured: None` là tín hiệu "không hỏi được", KHÁC `Some(false)` ("đã hỏi,
/// chưa cấu hình"). `IpcError` `ai_config.keychain_unavailable` vẫn đúng chỗ Matrix đặt nó —
/// trên hành động GHI/XOÁ ([`ai_config_save_key`]/[`ai_config_delete_key`]), không trên GET.
/// ─────────────────────────────────────────────────────────────────────────────
///
/// # Lỗi
/// - `global.db` vắng mặt ⇒ `store.open_failed`;
/// - `ScopeResolver::apply_override` từ chối (lỗi lập trình) ⇒ `ai_config.scope_error`.
///
/// Keychain từ chối trả lời KHÔNG nằm trong danh sách lỗi ở trên nữa — xem khối 🔵 ngay
/// trên: nó trở thành `key_configured: None`, không một `IpcError`.
pub fn ai_config_get(
    global: Option<&Store>,
    open: Option<&OpenWork>,
) -> Result<AiConfigGetWire, IpcError> {
    let global_store = global.ok_or_else(store_is_missing)?;
    let resolver = open.map(|w| w.scope.clone()).unwrap_or_else(ScopeResolver::global_only);
    let work_store = open.map(|w| &w.store);
    let resolved = resolve_two_tiers(&resolver, global_store, work_store)?;

    let fields = AiConfigField::ALL
        .iter()
        .map(|&field| AiConfigFieldWire::from_resolved(field, resolved.get(field.as_str())))
        .collect();

    // `.ok()`, KHONG `?` -- mot keychain tu choi tra loi khong duoc phep lam nam truong
    // plaintext ben tren bien mat theo (xem khoi doc-comment SUA o tren).
    let key_configured = keychain::configured().ok();

    Ok(AiConfigGetWire { work_tier_available: open.is_some(), fields, key_configured })
}

/// Ghi (hoặc thay) khoá API — **hàm thuần**. Chỉ tầng [`AiConfigTier::Global`] được chấp
/// nhận (§Always spec 4.3: "whatever tier the screen is operating in"); một yêu cầu tầng
/// Tác phẩm bị từ chối TRƯỚC khi [`validate_key`] hay `keychain::set` chạy. Giá trị được
/// kiểm bằng [`validate_key`] TRƯỚC khi chạm keychain (§Always spec 4.3: "rejected before
/// any keychain call").
///
/// # Lỗi
/// - `tier != Global` ⇒ `ai_config.key_is_global` — **không đọc, không ghi gì**;
/// - giá trị rỗng/toàn khoảng trắng ⇒ `ai_config.key_invalid_value` — **0 lượt ghi keychain**;
/// - keychain từ chối trả lời ⇒ `ai_config.keychain_unavailable` (retryable).
pub fn ai_config_save_key(tier: AiConfigTier, value: &str) -> Result<(), IpcError> {
    if tier != AiConfigTier::Global {
        return Err(AiConfigKeyError::TierNotGlobal.into());
    }
    let validated = validate_key(value).map_err(AiConfigKeyError::from)?;
    keychain::set(&validated).map_err(AiConfigKeyError::from)?;
    Ok(())
}

/// Xoá khoá API — **hàm thuần**. Chỉ tầng [`AiConfigTier::Global`] được chấp nhận, cùng lý
/// lẽ [`ai_config_save_key`]. Xoá khi không có entry nào là THÀNH CÔNG (I/O Matrix spec 4.3:
/// "Delete when none exists" — hậu trạng thái là trạng thái được yêu cầu).
///
/// # Lỗi
/// - `tier != Global` ⇒ `ai_config.key_is_global` — **không đọc, không ghi gì**;
/// - keychain từ chối trả lời ⇒ `ai_config.keychain_unavailable` (retryable).
pub fn ai_config_delete_key(tier: AiConfigTier) -> Result<(), IpcError> {
    if tier != AiConfigTier::Global {
        return Err(AiConfigKeyError::TierNotGlobal.into());
    }
    keychain::delete().map_err(AiConfigKeyError::from)?;
    Ok(())
}

/// Ghi một trường ở tầng `tier` — **hàm thuần**. Giá trị được kiểm bằng
/// [`crate::core::aiconfig::validate_field`] TRƯỚC khi tầng đích được định tuyến hay bất kỳ
/// giao dịch nào mở (§Always spec 4.2).
///
/// # Lỗi
/// - giá trị không hợp lệ ⇒ `ai_config.invalid_value` — **0 lượt ghi**;
/// - tầng đích không sẵn sàng ⇒ `store.open_failed`/`ai_config.work_tier_unavailable`.
pub fn ai_config_save_field(
    global: Option<&Store>,
    open: Option<&OpenWork>,
    tier: AiConfigTier,
    field: AiConfigField,
    value: &str,
) -> Result<(), IpcError> {
    let validated = validate_field(field, value)?;
    let store = store_for_tier(tier, global, open)?;
    write_field(store, field, &validated)?;
    Ok(())
}

/// Trả một trường TẦNG TÁC PHẨM về kế thừa — **hàm thuần**. Xoá hàng Tác phẩm của `field`;
/// trường sau đó phân giải lại từ Global (I/O Matrix "Clear a Work override"). Luôn nhắm
/// tầng Tác phẩm — trả tầng Global về kế thừa không có nghĩa (không có tầng nào dưới nó).
///
/// # Lỗi
/// chưa có Tác phẩm nào đang mở ⇒ `ai_config.work_tier_unavailable`.
pub fn ai_config_clear_override(open: Option<&OpenWork>, field: AiConfigField) -> Result<(), IpcError> {
    // `global: None` — tầng đích ở đây LUÔN là Work (xem doc-comment), nên nhánh Global của
    // `store_for_tier` không bao giờ chạy; gọi qua nó thay vì chép lại nhánh Work tại chỗ để
    // một lượt sửa luật định tuyến sau này chỉ phải sửa MỘT nơi.
    let store = store_for_tier(AiConfigTier::Work, None, open)?;
    clear_field(store, field)?;
    Ok(())
}

/// Năm vỏ `#[tauri::command]`. **Không một quy tắc nào sống ở đây.**
///
/// ⚠️ Tên command trên dây LÀ tên hàm — năm vỏ dưới đây mang ĐÚNG tên năm hàm thuần ở
/// `super::`, không hậu tố. Chỗ gọi xuống dùng `super::tên_hàm(...)` đủ điều kiện.
///
/// `ai_config_save_key`/`ai_config_delete_key` không cần `app: tauri::AppHandle` — khác ba
/// vỏ kia, chúng không đọc `Store`/`OpenWorkState`: khoá API sống trong keychain hệ điều
/// hành, không trong `global.db`/`project.db`, và tầng bị từ chối bằng tham số `tier` tới
/// tay chứ không bằng việc dò `OpenWorkState` (xem doc-comment đầu tệp).
pub mod wire {
    use super::{AiConfigField, AiConfigGetWire, AiConfigTier};
    use crate::commands::project::OpenWorkState;
    use crate::core::i18n::IpcError;
    use crate::core::store::Store;

    /// `try_state`, không `state()` — cùng lý do mọi vỏ khác của kho: `app.manage(store)`
    /// (`global.db`) và `app.manage(OpenWorkState)` có thể chưa từng chạy.
    #[tauri::command]
    pub fn ai_config_get(app: tauri::AppHandle) -> Result<AiConfigGetWire, IpcError> {
        use tauri::Manager as _;

        let global = app.try_state::<Store>();
        let Some(work_state) = app.try_state::<OpenWorkState>() else {
            return super::ai_config_get(global.as_deref(), None);
        };
        let guard = work_state.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
        super::ai_config_get(global.as_deref(), guard.as_ref())
    }

    #[tauri::command]
    pub fn ai_config_save_field(
        app: tauri::AppHandle,
        tier: AiConfigTier,
        field: AiConfigField,
        value: String,
    ) -> Result<(), IpcError> {
        use tauri::Manager as _;

        let global = app.try_state::<Store>();
        let Some(work_state) = app.try_state::<OpenWorkState>() else {
            return super::ai_config_save_field(global.as_deref(), None, tier, field, &value);
        };
        let guard = work_state.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
        super::ai_config_save_field(global.as_deref(), guard.as_ref(), tier, field, &value)
    }

    #[tauri::command]
    pub fn ai_config_clear_override(app: tauri::AppHandle, field: AiConfigField) -> Result<(), IpcError> {
        use tauri::Manager as _;

        let Some(work_state) = app.try_state::<OpenWorkState>() else {
            return super::ai_config_clear_override(None, field);
        };
        let guard = work_state.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
        super::ai_config_clear_override(guard.as_ref(), field)
    }

    #[tauri::command]
    pub fn ai_config_save_key(tier: AiConfigTier, value: String) -> Result<(), IpcError> {
        super::ai_config_save_key(tier, &value)
    }

    #[tauri::command]
    pub fn ai_config_delete_key(tier: AiConfigTier) -> Result<(), IpcError> {
        super::ai_config_delete_key(tier)
    }
}
