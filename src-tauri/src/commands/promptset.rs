//! Bề mặt IPC cho bộ prompt theo thể loại — Story 4.4 (FR69, AD-18,
//! `ScopeKind::Prompt => "prompt" : Override`).
//!
//! Cùng khuôn `commands::aiconfig`: hàm thuần nhận `Option<&Store>` (tầng Global) **cộng**
//! `Option<&OpenWork>` (tầng Tác phẩm) trước, `#[tauri::command]` chỉ là vỏ mỏng trong
//! [`wire`]. Năm lệnh: liệt kê hai tầng đã phân giải (Quyết định #1: cả bộ, theo tên, hàng
//! Global bị che vẫn hiện) · tạo một bộ ở một tầng · đổi tên · sửa thân · xoá.
//!
//! ─────────────────────────────────────────────────────────────────────────────
//! 🔴 KHÔNG LẮP RÁP PROMPT, KHÔNG THAY THẾ BIẾN — MODULE NÀY CHỈ CRUD TÊN + THÂN
//! ─────────────────────────────────────────────────────────────────────────────
//! Cảnh báo dấu ngoặc (`MarkerWarnings`) đi trên dây như DỮ LIỆU trả về của
//! [`prompt_set_create`]/[`prompt_set_update_body`], không bao giờ như một lỗi — Quyết định
//! #3: `core::promptset::scan_markers` không bao giờ trả `Err` (§Always spec 4.4: "A prompt
//! body is never rejected for its markers").
//!
//! ⚠️ Mọi chuỗi trong tệp này viết KHÔNG DẤU — `scripts/check-i18n.mjs` Kiểm A quét
//! `src-tauri/**/*.rs`.

use crate::commands::project::OpenWork;
use crate::core::i18n::IpcError;
use crate::core::promptset::{
    MarkerWarnings, PromptSetTier, PromptVariable, ResolvedPromptSet, create, delete, rename,
    resolve_two_tiers, update_body,
};
use crate::core::scope::ScopeResolver;
use crate::core::store::{Store, StoreError, StoreKind};

/// Kho `global.db` vắng mặt ⇒ lỗi *mở kho* — cùng khuôn `commands::aiconfig::store_is_missing`.
fn store_is_missing() -> IpcError {
    StoreError::OpenFailed {
        store: StoreKind::Global,
        detail: "the global store was never managed; see lib.rs::open_global_store".to_owned(),
    }
    .into()
}

// ⚠️ Không một `store_for_tier` riêng ở lớp lệnh này — khác `commands::aiconfig`.
// `core::promptset::store::{create, rename, update_body, delete}` đều nhận CHỮ KÝ
// `(global: &Store, work: Option<&Store>, tier: PromptSetTier, …)` và tự định tuyến tầng
// BÊN TRONG (Phase 1's shape, mirroring `core::glossary::store::add_manual_term`) — khác
// `core::aiconfig::store::write_field(store: &Store, …)`, thứ nhận một `&Store` ĐÃ được lớp
// lệnh chọn sẵn. Hệ quả đo được: MỌI lượt ghi bộ prompt — kể cả một yêu cầu tầng Work —
// đòi `global.db` phải đang quản lý, vì chữ ký core không nhận `Option<&Store>` cho
// `global`. `store_is_missing()` bên dưới vì thế luôn là kiểm TRƯỚC TIÊN, không phân biệt
// `tier`.

/// Nhãn tầng trên dây — hình dạng OUTPUT, cùng khuôn `commands::aiconfig::AiConfigTierWire`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PromptSetTierWire {
    Global,
    Work,
}

impl From<PromptSetTier> for PromptSetTierWire {
    fn from(t: PromptSetTier) -> Self {
        match t {
            PromptSetTier::Global => PromptSetTierWire::Global,
            PromptSetTier::Work => PromptSetTierWire::Work,
        }
    }
}

/// Cảnh báo dấu ngoặc trên dây — hình dạng OUTPUT của [`MarkerWarnings`] (Quyết định #3).
/// KHÔNG BAO GIỜ là một `IpcError`; đi kèm phản hồi THÀNH CÔNG của
/// [`prompt_set_create`]/[`prompt_set_update_body`].
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct PromptSetWarningsWire {
    /// Toàn văn (`"{{foo}}"`) mỗi dấu ngoặc lạ, thứ tự gặp lần đầu — I/O Matrix: "a warning
    /// names the unknown token".
    pub unknown_markers: Vec<String>,
    /// `true` khi thân KHÔNG mang `{{glossary_terms}}` — I/O Matrix: "a warning says
    /// Glossary Enforcement is off for this set".
    pub glossary_terms_missing: bool,
}

impl From<MarkerWarnings> for PromptSetWarningsWire {
    fn from(w: MarkerWarnings) -> Self {
        Self { unknown_markers: w.unknown_markers, glossary_terms_missing: w.glossary_terms_missing }
    }
}

/// Một bộ đã phân giải trên dây — hình dạng OUTPUT của [`prompt_set_list`].
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct PromptSetWire {
    /// `prompt_set.id` của hàng ĐANG THẮNG (`tier` bên dưới) — định danh cho
    /// [`prompt_set_rename`]/[`prompt_set_update_body`]/[`prompt_set_delete`] tiếp theo.
    pub id: i64,
    pub name: String,
    pub body: String,
    pub tier: PromptSetTierWire,
    /// Thân của bộ Global cùng tên đang bị bộ này che (Quyết định #1) — `None` khi bộ chỉ
    /// tồn tại ở một tầng, hoặc khi `tier == PromptSetTierWire::Global`.
    /// `prompt-library.html:134` ("bị prompt cùng tên ở trên che").
    pub shadowed_body: Option<String>,
}

impl From<ResolvedPromptSet> for PromptSetWire {
    fn from(r: ResolvedPromptSet) -> Self {
        Self { id: r.id, name: r.name, body: r.body, tier: r.tier.into(), shadowed_body: r.shadowed_body }
    }
}

/// Phong bì trả lời của [`prompt_set_list`] — cùng lý do `AiConfigGetWire`/`QuickAddLookup`
/// mang `work_tier_available`: webview phải học "có Tác phẩm nào đang mở không" NGAY trong
/// đúng lượt gọi đã đọc `OpenWorkState`, không suy từ một tín hiệu UI khác.
///
/// ─────────────────────────────────────────────────────────────────────────────
/// 🔴 `variables` LÀ NGUỒN SỰ THẬT DUY NHẤT CỦA MÀN SOẠN THẢO — Phase 4c, đóng khoảng hở
/// Phase 4b để lại (`PromptLibraryOverlay.vue`'s cũ `PROMPT_VARIABLES`)
/// ─────────────────────────────────────────────────────────────────────────────
/// §Always spec 4.4: "The three ratified variable names live in ONE closed Rust type that
/// both the validator and the compose screen's variable list read from... A second
/// hand-written list anywhere is a defect." Trường này gói [`PromptVariable::ALL`] mà KHÔNG
/// thêm một command thứ sáu (Phase 2 đã khoá đúng năm wire — `tests/ipc_contract.rs::
/// the_prompt_set_wires_are_registered` đếm chúng). `prompt_set_list` là màn hình DUY NHẤT
/// đọc danh sách bộ, nên gói từ vựng biến số VÀO CHÍNH phong bì này là chỗ rẻ nhất: một lượt
/// gọi, không một dây mới. `PromptLibraryOverlay.vue` render từ trường này; nó không còn
/// một mảng gõ tay nào của riêng nó.
#[derive(Debug, Clone, serde::Serialize)]
pub struct PromptSetListWire {
    /// `true` ⇔ có một Tác phẩm đang mở, tức tầng [`PromptSetTier::Work`] dùng được cho lượt
    /// tạo/đổi tên/sửa/xoá tiếp theo.
    pub work_tier_available: bool,
    /// Mọi bộ đã phân giải, theo TÊN (Quyết định #1) — I/O Matrix: "List with zero sets
    /// anywhere ... Empty state saying nothing is configured": `Vec` rỗng, KHÔNG một lỗi.
    pub sets: Vec<PromptSetWire>,
    /// Tên (không mang `{{`/`}}`) của mỗi biến số ratify, ĐÚNG thứ tự
    /// [`PromptVariable::ALL`] — dựng từ `ALL`, không bao giờ một literal tay. Màn soạn thảo
    /// tự ghép `{{name}}` và tự suy khoá i18n `prompt.library.var_<name>_desc` theo QUY ƯỚC,
    /// không một bảng tra tên thứ hai.
    pub variables: Vec<String>,
}

/// Liệt kê mọi bộ prompt, hai tầng đã phân giải — **hàm thuần, đây là thứ test gọi**.
///
/// # Lỗi
/// - `global.db` vắng mặt ⇒ `store.open_failed` (I/O Matrix: "Resolve when store missing ...
///   Reported as unavailable, not as 'no sets'");
/// - đường đọc trượt (một trong hai tầng) ⇒ `store.read_failed`;
/// - `ScopeResolver::apply_override` từ chối ⇒ `prompt_set.scope_error` (lỗi lập trình,
///   không nên xảy ra trên đường gọi đúng).
pub fn prompt_set_list(
    global: Option<&Store>,
    open: Option<&OpenWork>,
) -> Result<PromptSetListWire, IpcError> {
    let global_store = global.ok_or_else(store_is_missing)?;
    let resolver = open.map(|w| w.scope.clone()).unwrap_or_else(ScopeResolver::global_only);
    let work_store = open.map(|w| &w.store);
    let resolved = resolve_two_tiers(&resolver, global_store, work_store)?;

    let sets = resolved.into_values().map(PromptSetWire::from).collect();
    let variables = PromptVariable::ALL.iter().map(|v| v.as_str().to_owned()).collect();
    Ok(PromptSetListWire { work_tier_available: open.is_some(), sets, variables })
}

/// Phong bì trả lời của [`prompt_set_create`] — `id` của hàng mới CỘNG cảnh báo dấu ngoặc
/// của chính thân vừa lưu (Quyết định #3).
#[derive(Debug, Clone, serde::Serialize)]
pub struct PromptSetCreateWire {
    pub id: i64,
    pub warnings: PromptSetWarningsWire,
}

/// Tạo một bộ mới ở tầng `tier` — **hàm thuần, đây là thứ test gọi**.
///
/// # Lỗi
/// - `tier == Global` mà `global.db` vắng mặt ⇒ `store.open_failed`;
/// - `tier == Work` mà chưa mở Tác phẩm nào ⇒ `prompt_set.work_tier_unavailable`;
/// - `name` rỗng/toàn khoảng trắng ⇒ `prompt_set.invalid_name` — **0 lượt ghi**;
/// - `name` đã tồn tại ở tầng đó ⇒ `prompt_set.name_taken` — hàng đang có KHÔNG bị đụng tới.
pub fn prompt_set_create(
    global: Option<&Store>,
    open: Option<&OpenWork>,
    tier: PromptSetTier,
    name: &str,
    body: &str,
) -> Result<PromptSetCreateWire, IpcError> {
    let global_store = global.ok_or_else(store_is_missing)?;
    let (id, warnings) = create(global_store, open.map(|w| &w.store), tier, name, body)?;
    Ok(PromptSetCreateWire { id, warnings: warnings.into() })
}

/// Đổi TÊN của bộ `(tier, id)` — **hàm thuần, đây là thứ test gọi**.
///
/// # Lỗi
/// - `tier == Work` mà chưa mở Tác phẩm nào ⇒ `prompt_set.work_tier_unavailable`;
/// - `new_name` rỗng/toàn khoảng trắng ⇒ `prompt_set.invalid_name` — **0 lượt ghi**;
/// - `new_name` đã tồn tại ở `tier` đó ⇒ `prompt_set.name_taken` — cả hai hàng KHÔNG bị đụng;
/// - `(tier, id)` không khớp hàng nào ⇒ `prompt_set.not_found`.
pub fn prompt_set_rename(
    global: Option<&Store>,
    open: Option<&OpenWork>,
    tier: PromptSetTier,
    id: i64,
    new_name: &str,
) -> Result<(), IpcError> {
    let global_store = global.ok_or_else(store_is_missing)?;
    rename(global_store, open.map(|w| &w.store), tier, id, new_name)?;
    Ok(())
}

/// Sửa THÂN của bộ `(tier, id)` — **hàm thuần, đây là thứ test gọi**. Trả về cảnh báo dấu
/// ngoặc của thân MỚI (Quyết định #3); thân không bao giờ bị từ chối vì nội dung của nó.
///
/// # Lỗi
/// - `tier == Work` mà chưa mở Tác phẩm nào ⇒ `prompt_set.work_tier_unavailable`;
/// - `(tier, id)` không khớp hàng nào ⇒ `prompt_set.not_found`.
pub fn prompt_set_update_body(
    global: Option<&Store>,
    open: Option<&OpenWork>,
    tier: PromptSetTier,
    id: i64,
    body: &str,
) -> Result<PromptSetWarningsWire, IpcError> {
    let global_store = global.ok_or_else(store_is_missing)?;
    let warnings = update_body(global_store, open.map(|w| &w.store), tier, id, body)?;
    Ok(warnings.into())
}

/// Xoá bộ `(tier, id)` — **hàm thuần, đây là thứ test gọi**. Xoá một bộ Work đang che một bộ
/// Global cùng tên làm bộ Global đó hiệu lực trở lại ngay ở lượt [`prompt_set_list`] tiếp
/// theo (I/O Matrix: "Delete a shadowing Work set ... the Global set becomes effective").
///
/// # Lỗi
/// - `tier == Work` mà chưa mở Tác phẩm nào ⇒ `prompt_set.work_tier_unavailable`;
/// - `(tier, id)` không khớp hàng nào ⇒ `prompt_set.not_found`.
pub fn prompt_set_delete(
    global: Option<&Store>,
    open: Option<&OpenWork>,
    tier: PromptSetTier,
    id: i64,
) -> Result<(), IpcError> {
    let global_store = global.ok_or_else(store_is_missing)?;
    delete(global_store, open.map(|w| &w.store), tier, id)?;
    Ok(())
}

/// Năm vỏ `#[tauri::command]`. **Không một quy tắc nào sống ở đây.**
///
/// ⚠️ Tên command trên dây LÀ tên hàm — năm vỏ dưới đây mang ĐÚNG tên năm hàm thuần ở
/// `super::`, không hậu tố. Chỗ gọi xuống dùng `super::tên_hàm(...)` đủ điều kiện.
pub mod wire {
    use super::{PromptSetCreateWire, PromptSetListWire, PromptSetTier, PromptSetWarningsWire};
    use crate::commands::project::OpenWorkState;
    use crate::core::i18n::IpcError;
    use crate::core::store::Store;

    /// `try_state`, không `state()` — cùng lý do mọi vỏ khác của kho: `app.manage(store)`
    /// (`global.db`) và `app.manage(OpenWorkState)` có thể chưa từng chạy.
    #[tauri::command]
    pub fn prompt_set_list(app: tauri::AppHandle) -> Result<PromptSetListWire, IpcError> {
        use tauri::Manager as _;

        let global = app.try_state::<Store>();
        let Some(work_state) = app.try_state::<OpenWorkState>() else {
            return super::prompt_set_list(global.as_deref(), None);
        };
        let guard = work_state.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
        super::prompt_set_list(global.as_deref(), guard.as_ref())
    }

    #[tauri::command]
    pub fn prompt_set_create(
        app: tauri::AppHandle,
        tier: PromptSetTier,
        name: String,
        body: String,
    ) -> Result<PromptSetCreateWire, IpcError> {
        use tauri::Manager as _;

        let global = app.try_state::<Store>();
        let Some(work_state) = app.try_state::<OpenWorkState>() else {
            return super::prompt_set_create(global.as_deref(), None, tier, &name, &body);
        };
        let guard = work_state.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
        super::prompt_set_create(global.as_deref(), guard.as_ref(), tier, &name, &body)
    }

    #[tauri::command]
    pub fn prompt_set_rename(
        app: tauri::AppHandle,
        tier: PromptSetTier,
        id: i64,
        new_name: String,
    ) -> Result<(), IpcError> {
        use tauri::Manager as _;

        let global = app.try_state::<Store>();
        let Some(work_state) = app.try_state::<OpenWorkState>() else {
            return super::prompt_set_rename(global.as_deref(), None, tier, id, &new_name);
        };
        let guard = work_state.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
        super::prompt_set_rename(global.as_deref(), guard.as_ref(), tier, id, &new_name)
    }

    #[tauri::command]
    pub fn prompt_set_update_body(
        app: tauri::AppHandle,
        tier: PromptSetTier,
        id: i64,
        body: String,
    ) -> Result<PromptSetWarningsWire, IpcError> {
        use tauri::Manager as _;

        let global = app.try_state::<Store>();
        let Some(work_state) = app.try_state::<OpenWorkState>() else {
            return super::prompt_set_update_body(global.as_deref(), None, tier, id, &body);
        };
        let guard = work_state.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
        super::prompt_set_update_body(global.as_deref(), guard.as_ref(), tier, id, &body)
    }

    #[tauri::command]
    pub fn prompt_set_delete(app: tauri::AppHandle, tier: PromptSetTier, id: i64) -> Result<(), IpcError> {
        use tauri::Manager as _;

        let global = app.try_state::<Store>();
        let Some(work_state) = app.try_state::<OpenWorkState>() else {
            return super::prompt_set_delete(global.as_deref(), None, tier, id);
        };
        let guard = work_state.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
        super::prompt_set_delete(global.as_deref(), guard.as_ref(), tier, id)
    }
}
