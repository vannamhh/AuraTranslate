//! Bề mặt IPC cho luật làm sạch lúc nhập — Story 6.5, FR124, AD-18.
//!
//! Cùng khuôn `commands::pinned`/`commands::glossary`: hàm thuần nhận `Option<&Store>` (tầng
//! Global) **cộng** `Option<&OpenWork>` (tầng Tác phẩm) trước, `#[tauri::command]` chỉ là vỏ
//! mỏng trong [`wire`]. Năm lệnh: liệt (hai tầng đã hợp nhất) · thêm · sửa · xoá · bật/tắt —
//! bốn lệnh sau định tuyến `&Store` theo [`CleanupRuleTier`] mà chỗ gọi chọn, vì danh tính
//! một luật trên dây là CẶP `(tier, id)`, không phải `id` trần.
//!
//! Bật/tắt và mọi lượt soạn là một lượt ghi THẬT vào bảng luật (§Always spec 6.5) — không
//! trạng thái bật/tắt nào chỉ sống trong bộ nhớ frontend; sau mỗi lệnh, `importPreviewState.ts`
//! dựng lại xem trước bằng cách gọi lại `preview_import_encoding_from_*`.
//!
//! ⚠️ Mọi chuỗi trong tệp này viết KHÔNG DẤU — `scripts/check-i18n.mjs` Kiểm A quét
//! `src-tauri/**/*.rs`.

use crate::commands::project::{CleanupRuleTierWire, OpenWork};
use crate::core::cleanup::{
    CleanupRule, CleanupRuleKind, CleanupStoreError, CleanupRuleTier, add_rule, delete_rule,
    edit_rule, resolve_two_tiers, set_enabled,
};
use crate::core::i18n::IpcError;
use crate::core::scope::ScopeResolver;
use crate::core::store::{Store, StoreError, StoreKind};

/// Kho `global.db` vắng mặt ⇒ lỗi *mở kho* — cùng khuôn `commands::pinned::store_is_missing`.
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
    tier: CleanupRuleTier,
    global: Option<&'a Store>,
    open: Option<&'a OpenWork>,
) -> Result<&'a Store, IpcError> {
    match tier {
        CleanupRuleTier::Global => global.ok_or_else(store_is_missing),
        CleanupRuleTier::Work => {
            open.map(|w| &w.store).ok_or_else(|| CleanupStoreError::WorkTierUnavailable.into())
        }
    }
}

/// Một luật, hai tầng đã hợp nhất — hình dạng trả về của [`cleanup_list_rules`]. Không mang
/// số đếm theo Chương — đó là dữ liệu của lượt xem trước
/// (`commands::project::preview_import_encoding`), không của lệnh liệt kê chung này.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct CleanupRuleWire {
    pub tier: CleanupRuleTierWire,
    pub id: i64,
    pub pattern: String,
    pub kind: String,
    pub enabled: bool,
}

impl From<CleanupRule> for CleanupRuleWire {
    fn from(r: CleanupRule) -> Self {
        Self {
            tier: r.tier.into(),
            id: r.id,
            pattern: r.pattern,
            kind: r.kind.as_str().to_owned(),
            enabled: r.enabled,
        }
    }
}

/// Liệt kê hai tầng đã hợp nhất — **hàm thuần**.
///
/// # Lỗi
/// - `global.db` vắng mặt ⇒ `store.open_failed`;
/// - `ScopeResolver::apply_merge` từ chối (lỗi lập trình) ⇒ `cleanup.scope_error`.
pub fn cleanup_list_rules(
    global: Option<&Store>,
    open: Option<&OpenWork>,
) -> Result<Vec<CleanupRuleWire>, IpcError> {
    let global = global.ok_or_else(store_is_missing)?;
    let resolver = open.map(|w| w.scope.clone()).unwrap_or_else(ScopeResolver::global_only);
    let work_store = open.map(|w| &w.store);
    let rules = resolve_two_tiers(&resolver, global, work_store)?;
    Ok(rules.into_iter().map(CleanupRuleWire::from).collect())
}

/// Thêm một luật mới vào tầng `tier` — **hàm thuần**. Mẫu rỗng/chỉ khoảng trắng hoặc regex
/// hỏng bị từ chối TRƯỚC khi mở giao dịch (§Always spec 6.5).
///
/// 🔴 **`tier` vẫn TỔNG QUÁT ở tầng lệnh này — bề mặt SOẠN của màn xem trước nhập (vòng rà
/// 2026-09-06, phán quyết Ice) chỉ gửi [`CleanupRuleTier::Global`], KHÔNG phải một luật của
/// hàm này.** Lý do đo được: `commands::project::store_for_tier` (bên dưới) phân giải tầng
/// Tác phẩm từ `OpenWorkState` — Tác phẩm ĐANG MỞ. Màn xem trước nhập (`ImportPreviewOverlay.vue`)
/// lại đang TẠO một Tác phẩm CHƯA TỒN TẠI, nên gọi hàm này với `tier = Work` từ MÀN ĐÓ sẽ hoặc
/// trượt `WorkTierUnavailable` (không Tác phẩm nào đang mở), hoặc — tệ hơn, nếu một Tác phẩm
/// KHÁC tình cờ đang mở — đính luật vào `project.db` của Tác phẩm đó, im lặng (đúng lớp "luật
/// ẩn xoá nhầm" mà FR124 tồn tại để chặn). `ImportPreviewOverlay.vue::onAddCleanupRule` vì thế
/// hard-code `'global'`, không có ô chọn tầng. Hàm này KHÔNG tự chặn `tier = Work` — nó vẫn là
/// một CRUD chung, hợp lệ cho bất kỳ bề mặt nào có một Tác phẩm ĐANG MỞ thật (một màn Cài đặt
/// của chính Tác phẩm đó, chẳng hạn — chưa dựng, ghi nợ có chủ ở `deferred-work.md`), và các
/// ca `cleanup_contract.rs` dựng `tier: Work` trực tiếp để kiểm hai tầng độc lập vẫn hợp lệ.
///
/// # Lỗi
/// - tầng đích không sẵn sàng ⇒ `store.open_failed`/`cleanup.work_tier_unavailable`;
/// - mẫu rỗng ⇒ `cleanup.empty_pattern`; regex hỏng ⇒ `cleanup.invalid_regex`.
pub fn cleanup_add_rule(
    global: Option<&Store>,
    open: Option<&OpenWork>,
    tier: CleanupRuleTier,
    pattern: &str,
    kind: CleanupRuleKind,
) -> Result<i64, IpcError> {
    let store = store_for_tier(tier, global, open)?;
    let id = add_rule(store, pattern, kind)?;
    Ok(id)
}

/// Sửa mẫu/hình dạng của luật `(tier, id)` — **hàm thuần**.
///
/// # Lỗi
/// - cùng ba ca của [`cleanup_add_rule`];
/// - `id` không khớp hàng nào trong tầng đó ⇒ `cleanup.rule_missing`.
pub fn cleanup_edit_rule(
    global: Option<&Store>,
    open: Option<&OpenWork>,
    tier: CleanupRuleTier,
    id: i64,
    pattern: &str,
    kind: CleanupRuleKind,
) -> Result<(), IpcError> {
    let store = store_for_tier(tier, global, open)?;
    edit_rule(store, id, pattern, kind)?;
    Ok(())
}

/// Xoá luật `(tier, id)` — **hàm thuần**. Vô hại cho một `id` đã biến mất (cùng khuôn
/// `commands::pinned::unpin_entry`).
pub fn cleanup_delete_rule(
    global: Option<&Store>,
    open: Option<&OpenWork>,
    tier: CleanupRuleTier,
    id: i64,
) -> Result<(), IpcError> {
    let store = store_for_tier(tier, global, open)?;
    delete_rule(store, id)?;
    Ok(())
}

/// Bật/tắt luật `(tier, id)` — **hàm thuần**. Đây LÀ một lượt ghi thật (§Always spec 6.5) —
/// không trạng thái bật/tắt nào chỉ sống trong bộ nhớ frontend.
///
/// # Lỗi
/// - tầng đích không sẵn sàng ⇒ `store.open_failed`/`cleanup.work_tier_unavailable`;
/// - `id` không khớp hàng nào ⇒ `cleanup.rule_missing`.
pub fn cleanup_set_enabled(
    global: Option<&Store>,
    open: Option<&OpenWork>,
    tier: CleanupRuleTier,
    id: i64,
    enabled: bool,
) -> Result<(), IpcError> {
    let store = store_for_tier(tier, global, open)?;
    set_enabled(store, id, enabled)?;
    Ok(())
}

/// Năm vỏ `#[tauri::command]`. **Không một quy tắc nào sống ở đây.**
///
/// ⚠️ Tên command trên dây LÀ tên hàm — năm vỏ dưới đây mang ĐÚNG tên năm hàm thuần ở
/// `super::`, không hậu tố (cùng khuôn `commands::pinned::wire`). Chỗ gọi xuống dùng
/// `super::tên_hàm(...)` đủ điều kiện, không `use` trực tiếp, để tránh trùng tên với vỏ.
pub mod wire {
    use super::CleanupRuleWire;
    use crate::commands::project::OpenWorkState;
    use crate::core::cleanup::{CleanupRuleKind, CleanupRuleTier};
    use crate::core::i18n::IpcError;
    use crate::core::store::Store;

    /// `try_state`, không `state()` — cùng lý do mọi vỏ khác của kho: `app.manage(store)`
    /// (`global.db`) và `app.manage(OpenWorkState)` có thể chưa từng chạy, và `panic =
    /// "abort"` giết cả tiến trình nếu ta thẳng tay `state::<T>()`.
    #[tauri::command]
    pub fn cleanup_list_rules(app: tauri::AppHandle) -> Result<Vec<CleanupRuleWire>, IpcError> {
        use tauri::Manager as _;

        let global = app.try_state::<Store>();
        let Some(work_state) = app.try_state::<OpenWorkState>() else {
            return super::cleanup_list_rules(global.as_deref(), None);
        };
        let guard = work_state.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
        super::cleanup_list_rules(global.as_deref(), guard.as_ref())
    }

    #[tauri::command]
    pub fn cleanup_add_rule(
        app: tauri::AppHandle,
        tier: CleanupRuleTier,
        pattern: String,
        kind: CleanupRuleKind,
    ) -> Result<i64, IpcError> {
        use tauri::Manager as _;

        let global = app.try_state::<Store>();
        let Some(work_state) = app.try_state::<OpenWorkState>() else {
            return super::cleanup_add_rule(global.as_deref(), None, tier, &pattern, kind);
        };
        let guard = work_state.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
        super::cleanup_add_rule(global.as_deref(), guard.as_ref(), tier, &pattern, kind)
    }

    #[tauri::command]
    pub fn cleanup_edit_rule(
        app: tauri::AppHandle,
        tier: CleanupRuleTier,
        id: i64,
        pattern: String,
        kind: CleanupRuleKind,
    ) -> Result<(), IpcError> {
        use tauri::Manager as _;

        let global = app.try_state::<Store>();
        let Some(work_state) = app.try_state::<OpenWorkState>() else {
            return super::cleanup_edit_rule(global.as_deref(), None, tier, id, &pattern, kind);
        };
        let guard = work_state.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
        super::cleanup_edit_rule(global.as_deref(), guard.as_ref(), tier, id, &pattern, kind)
    }

    #[tauri::command]
    pub fn cleanup_delete_rule(
        app: tauri::AppHandle,
        tier: CleanupRuleTier,
        id: i64,
    ) -> Result<(), IpcError> {
        use tauri::Manager as _;

        let global = app.try_state::<Store>();
        let Some(work_state) = app.try_state::<OpenWorkState>() else {
            return super::cleanup_delete_rule(global.as_deref(), None, tier, id);
        };
        let guard = work_state.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
        super::cleanup_delete_rule(global.as_deref(), guard.as_ref(), tier, id)
    }

    #[tauri::command]
    pub fn cleanup_set_enabled(
        app: tauri::AppHandle,
        tier: CleanupRuleTier,
        id: i64,
        enabled: bool,
    ) -> Result<(), IpcError> {
        use tauri::Manager as _;

        let global = app.try_state::<Store>();
        let Some(work_state) = app.try_state::<OpenWorkState>() else {
            return super::cleanup_set_enabled(global.as_deref(), None, tier, id, enabled);
        };
        let guard = work_state.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
        super::cleanup_set_enabled(global.as_deref(), guard.as_ref(), tier, id, enabled)
    }
}
