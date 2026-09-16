//! Bề mặt IPC cho cấu hình nhà cung cấp AI — Story 4.2, FR68, AD-18.
//!
//! Cùng khuôn `commands::cleanup`: hàm thuần nhận `Option<&Store>` (tầng Global) **cộng**
//! `Option<&OpenWork>` (tầng Tác phẩm) trước, `#[tauri::command]` chỉ là vỏ mỏng trong
//! [`wire`]. Ba lệnh: đọc năm trường đã phân giải hai tầng · ghi một trường ở một tầng ·
//! trả một trường tầng Tác phẩm về kế thừa.
//!
//! ⚠️ Mọi chuỗi trong tệp này viết KHÔNG DẤU — `scripts/check-i18n.mjs` Kiểm A quét
//! `src-tauri/**/*.rs`.

use crate::commands::project::OpenWork;
use crate::core::aiconfig::{
    AiConfigField, AiConfigStoreError, AiConfigTier, ResolvedField, clear_field, resolve_two_tiers,
    validate_field, write_field,
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
}

/// Đọc năm trường, hai tầng đã phân giải, cộng `work_tier_available` — **hàm thuần**. Luôn
/// trả đủ năm trường (`AiConfigField::ALL`), kể cả trường chưa ai cấu hình.
///
/// # Lỗi
/// - `global.db` vắng mặt ⇒ `store.open_failed`;
/// - `ScopeResolver::apply_override` từ chối (lỗi lập trình) ⇒ `ai_config.scope_error`.
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

    Ok(AiConfigGetWire { work_tier_available: open.is_some(), fields })
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

/// Ba vỏ `#[tauri::command]`. **Không một quy tắc nào sống ở đây.**
///
/// ⚠️ Tên command trên dây LÀ tên hàm — ba vỏ dưới đây mang ĐÚNG tên ba hàm thuần ở
/// `super::`, không hậu tố. Chỗ gọi xuống dùng `super::tên_hàm(...)` đủ điều kiện.
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
}
