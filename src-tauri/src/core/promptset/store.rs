//! SQL của bảng `prompt_set` — nạp/ghi hai tầng, phân giải qua
//! `ScopeResolver::apply_override` (Story 4.4, `ScopeKind::Prompt`).
//!
//! ─────────────────────────────────────────────────────────────────────────────
//! MODULE NÀY KHÔNG GÕ TÊN `ScopeKind`/`Semantics`/`Tier` — cùng luật mọi module miền khác
//! ─────────────────────────────────────────────────────────────────────────────
//! `tests/scope_boundary.rs::FORBIDDEN_OUTSIDE_SCOPE` cấm `Semantics`/`ScopeKind` ngoài
//! `core/scope/**`; `ScopeResolver::apply_override` nhận `kind: &str` đúng để chỗ này gọi
//! bằng một hằng literal ([`PROMPT_SET_SCOPE_KIND`]) mà không phải `use` kiểu đó.
//!
//! ─────────────────────────────────────────────────────────────────────────────
//! 🔵 `impl From<PromptSetError> for IpcError` THÊM Ở PHASE 2 (Task 8 đã khai xong khoá)
//! ─────────────────────────────────────────────────────────────────────────────
//! Phase 1 cố ý để impl này vắng mặt: `IpcError::new` đòi một `MessageKey` đã khai trong
//! `macro_rules! message_keys!` (`core/i18n/mod.rs`), và năm khoá của domain này
//! (`prompt_set.invalid_name`, `prompt_set.work_tier_unavailable`, `prompt_set.name_taken`,
//! `prompt_set.not_found`, `prompt_set.scope_error`) là Task 8 — viết impl này trước sẽ làm
//! CẢ TREE đỏ ở biên dịch. Task 8 nay đã khai đủ năm khoá đó cộng `vi.json`, nên impl sống
//! ngay dưới đây, cùng khuôn `core::aiconfig::store::{AiConfigStoreError, AiConfigKeyError}`.

use std::collections::BTreeMap;
use std::sync::{Arc, Mutex};

use crate::core::i18n::{IpcError, MessageKey};
use crate::core::scope::{ScopeError, ScopeResolver};
use crate::core::store::{ReadHandle, Store, StoreError, Transaction, is_unique_constraint_violation};

use super::vars::{MarkerWarnings, scan_markers};
use super::{PromptSet, PromptSetTier};

/// Khoá dây của `ScopeKind::Prompt` (`core/scope/kinds.rs`), chép lại đây làm literal —
/// module này không được `use` `ScopeKind`.
const PROMPT_SET_SCOPE_KIND: &str = "prompt";

/// Một tên bộ không qua được kiểm tra của chính nó — trả TRƯỚC khi bất kỳ giao dịch nào
/// mở (I/O Matrix: "Blank-ish name ... Rejected before SQL").
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InvalidName;

/// `name` — cắt khoảng trắng biên bằng ĐÚNG lớp Unicode `White_Space` mà `str::trim()`
/// dùng (25 điểm mã), khớp CHECK của [`super::super::store::schema::PROMPT_SET_DDL`] —
/// cùng luật hai lớp `core::glossary::store::insert_manual_entry` đã ghi cho `source_term`.
/// Rỗng sau khi trim ⇒ [`InvalidName`], **trước** khi một câu SQL nào chạy.
pub fn validate_name(raw: &str) -> Result<String, InvalidName> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Err(InvalidName);
    }
    Ok(trimmed.to_owned())
}

/// Nạp toàn bộ MỘT tầng (một `global.db` hoặc một `project.db`), khoá theo `name` — hình
/// dạng `BTreeMap` mà `ScopeResolver::apply_override` đòi ở tham số `global`/`work`.
///
/// ⚠️ **Tên KHÔNG phải `load_tier` trần** — `tests/glossary_boundary.rs::GLOSSARY_ONLY_SURFACE`
/// quét TOÀN `src-tauri/src/**` (trừ `core/glossary/**`) tìm CHUỖI `"load_tier"`, không phân
/// biệt được domain nào sở hữu nó (AD-36, vế thứ hai): một hàm cùng tên ở module khác trông
/// giống hệt việc lách qua `entries_eligible_for_injection`. Đặt tên dài hơn ở đây rẻ hơn nới
/// cổng đó.
pub fn load_prompt_set_tier(store: &Store) -> Result<BTreeMap<String, PromptSet>, StoreError> {
    store.read(|conn: ReadHandle<'_>| {
        let mut stmt = conn.prepare("SELECT id, name, body FROM prompt_set ORDER BY name")?;
        let mut rows = stmt.query([])?;

        let mut out = BTreeMap::new();
        while let Some(row) = rows.next()? {
            let id: i64 = row.get(0)?;
            let name: String = row.get(1)?;
            let body: String = row.get(2)?;
            out.insert(name.clone(), PromptSet { id, name, body });
        }
        Ok(out)
    })
}

/// Một bộ ĐÃ phân giải — hình dạng RIÊNG của domain này ([`PromptSetTier`], không
/// `core::scope::Tier`), cùng lý do `core::aiconfig::store::ResolvedField` không rò
/// `core::scope::{Tier, Resolved}` ra khỏi module của nó.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedPromptSet {
    /// `id` của hàng ĐANG THẮNG (`tier` bên dưới) — 🔵 THÊM Phase 2, cùng lý do
    /// [`super::PromptSet::id`]: lớp lệnh cần định danh này để gọi
    /// [`rename`]/[`update_body`]/[`delete`] cho đúng hàng vừa liệt kê. Hàng Global bị che
    /// (nếu có) không mang `id` riêng ở đây — màn quản lý story này chỉ thao tác trên hàng
    /// đang thắng, đúng AC "the Global one is shown as shadowed" (chỉ cần THẤY, không cần
    /// sửa/xoá độc lập nó qua kết quả này).
    pub id: i64,
    pub name: String,
    pub body: String,
    pub tier: PromptSetTier,
    /// Thân bị che ở tầng Global khi bộ NÀY đang thắng ở tầng Work (Quyết định #1: ghi đè
    /// CẢ BỘ theo tên — mục Global vẫn hiện, đánh dấu bị che, `prompt-library.html:134`).
    /// `None` khi bộ chỉ tồn tại ở một tầng, hoặc khi `tier == PromptSetTier::Global`.
    pub shadowed_body: Option<String>,
}

/// Nạp CẢ HAI tầng rồi phân giải qua `ScopeResolver::apply_override` — **cả bộ, theo tên**
/// (Quyết định #1). Một bộ Work trùng tên THAY THẾ TRỌN bộ Global cùng tên; bộ Global bị
/// che vẫn có mặt trong kết quả qua [`ResolvedPromptSet::shadowed_body`].
///
/// # Lỗi
/// [`PromptSetError::Store`] nếu một trong hai lượt [`load_prompt_set_tier`] thất bại;
/// [`PromptSetError::Scope`] nếu `apply_override` từ chối (lỗi lập trình, không nên xảy ra
/// trên đường gọi đúng — [`PROMPT_SET_SCOPE_KIND`] đã khớp `ScopeKind::Prompt::Override`).
pub fn resolve_two_tiers(
    resolver: &ScopeResolver,
    global: &Store,
    work: Option<&Store>,
) -> Result<BTreeMap<String, ResolvedPromptSet>, PromptSetError> {
    let global_tier = load_prompt_set_tier(global)?;
    let work_tier = work.map(load_prompt_set_tier).transpose()?;

    let resolved =
        resolver.apply_override(PROMPT_SET_SCOPE_KIND, &global_tier, work_tier.as_ref())?;

    resolved
        .into_iter()
        .map(|(name, r)| {
            // 🔴 `?`, KHÔNG rơi về `PromptSetTier::Global` im lặng — cùng lý do
            // `core::aiconfig::store::resolve_two_tiers` đã ghi cho chính nhánh này: chỉ
            // hai giá trị `core::scope::Tier::as_str()` từng phát ra, nên nhánh này không
            // nên xảy ra trên đường gọi đúng, nhưng một giá trị lạ rơi về `Global` im lặng
            // là đúng lớp lỗi "trông như đang chạy" mà dự án cấm.
            let wire = r.tier().as_str();
            let tier = PromptSetTier::from_wire(wire).ok_or_else(|| {
                PromptSetError::Scope(ScopeError::UnknownKind { wire: wire.to_owned() })
            })?;
            let shadowed_body = r.shadowed().map(|s| s.body.clone());
            let id = r.value().id;
            let body = r.value().body.clone();
            Ok((name.clone(), ResolvedPromptSet { id, name, body, tier, shadowed_body }))
        })
        .collect()
}

/// Chọn kho theo `tier` — khuôn chép `core::glossary::store::add_manual_term`.
fn store_for_tier<'a>(
    global: &'a Store,
    work: Option<&'a Store>,
    tier: PromptSetTier,
) -> Result<&'a Store, PromptSetError> {
    match tier {
        PromptSetTier::Global => Ok(global),
        PromptSetTier::Work => work.ok_or(PromptSetError::WorkTierUnavailable),
    }
}

/// Tạo một bộ mới ở tầng `tier`. Trả về `id` hàng mới CỘNG cảnh báo dấu ngoặc của chính
/// thân vừa lưu (Quyết định #3) — chỗ gọi (lớp lệnh, Phase 2) chuyển thẳng cảnh báo này ra
/// dây, không tự quét lại.
///
/// 🔴 **Trùng tên TRONG CÙNG tầng bị từ chối, hàng đang có KHÔNG bị đụng tới** (I/O Matrix:
/// "Duplicate name in same tier ... existing row untouched"). Phát hiện NGAY tại chỗ
/// `INSERT` trượt vì `UNIQUE`, trong CÙNG giao dịch — không một cửa sổ đua đọc-rồi-ghi nào
/// — qua một cờ kênh riêng, cùng khuôn `core::glossary::store::import_into_tier` dùng cho
/// va chạm `UNIQUE` của chính nó. Tên trùng KHÔNG được đọc lại từ chuỗi lỗi SQL; nó vẫn
/// nằm sẵn trong tay hàm này (`name`), nên không cần phân tích chuỗi thô.
///
/// # Lỗi
/// [`PromptSetError::WorkTierUnavailable`] nếu `tier` là [`PromptSetTier::Work`] mà `work`
/// là `None`. [`PromptSetError::InvalidName`] nếu `name` rỗng/toàn khoảng trắng — từ chối
/// TRƯỚC khi mở giao dịch, 0 lượt ghi. [`PromptSetError::NameTaken`] nếu `name` đã tồn tại
/// ở `tier` đó.
pub fn create(
    global: &Store,
    work: Option<&Store>,
    tier: PromptSetTier,
    name: &str,
    body: &str,
) -> Result<(i64, MarkerWarnings), PromptSetError> {
    let store = store_for_tier(global, work, tier)?;
    let name = validate_name(name)?;
    let body = body.to_owned();
    let warnings = scan_markers(&body);

    let collided = Arc::new(Mutex::new(false));
    let collided_for_closure = Arc::clone(&collided);
    let name_for_write = name.clone();

    let result = store.write(move |tx: &Transaction<'_>| {
        match tx.execute(
            "INSERT INTO prompt_set (name, body, created_at) \
             VALUES (?1, ?2, strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))",
            (&name_for_write, &body),
        ) {
            Ok(_) => Ok(tx.last_insert_rowid()),
            Err(e) if is_unique_constraint_violation(&e) => {
                *collided_for_closure.lock().unwrap_or_else(|p| p.into_inner()) = true;
                Err(e)
            }
            Err(e) => Err(e),
        }
    });

    match result {
        Ok(id) => Ok((id, warnings)),
        Err(e) => {
            if *collided.lock().unwrap_or_else(|p| p.into_inner()) {
                Err(PromptSetError::NameTaken { name })
            } else {
                Err(PromptSetError::from(e))
            }
        }
    }
}

/// Đổi TÊN của bộ `id` ở tầng `tier` — thao tác TÁCH BIỆT với sửa thân (§Design Notes spec
/// 4.4: đổi tên di chuyển khoá ghi đè và có thể va chạm; sửa thân thì không).
///
/// 🔴 **Nhận `(tier, id)`, không chỉ `id`** — `id` chỉ duy nhất TRONG một `Store`, cùng lý
/// do `core::glossary::GlossaryTier` tồn tại.
///
/// # Lỗi
/// [`PromptSetError::WorkTierUnavailable`]/[`PromptSetError::InvalidName`] — cùng
/// [`create`]. [`PromptSetError::NameTaken`] nếu `new_name` đã tồn tại ở `tier` đó (I/O
/// Matrix: "Rename to an existing name ... both rows untouched" — giao dịch rollback trọn,
/// hàng đang đổi tên giữ nguyên tên cũ). [`PromptSetError::NotFound`] nếu `(tier, id)`
/// không khớp hàng nào.
pub fn rename(
    global: &Store,
    work: Option<&Store>,
    tier: PromptSetTier,
    id: i64,
    new_name: &str,
) -> Result<(), PromptSetError> {
    let store = store_for_tier(global, work, tier)?;
    let new_name = validate_name(new_name)?;

    let collided = Arc::new(Mutex::new(false));
    let collided_for_closure = Arc::clone(&collided);
    let name_for_write = new_name.clone();

    let result = store.write(move |tx: &Transaction<'_>| {
        match tx.execute("UPDATE prompt_set SET name = ?1 WHERE id = ?2", (&name_for_write, id)) {
            Ok(changed) => Ok(changed),
            Err(e) if is_unique_constraint_violation(&e) => {
                *collided_for_closure.lock().unwrap_or_else(|p| p.into_inner()) = true;
                Err(e)
            }
            Err(e) => Err(e),
        }
    });

    let changed = match result {
        Ok(changed) => changed,
        Err(e) => {
            return if *collided.lock().unwrap_or_else(|p| p.into_inner()) {
                Err(PromptSetError::NameTaken { name: new_name })
            } else {
                Err(PromptSetError::from(e))
            };
        }
    };

    if changed == 0 {
        return Err(PromptSetError::NotFound);
    }
    Ok(())
}

/// Sửa THÂN của bộ `id` ở tầng `tier` — không đổi tên, không đổi khoá ghi đè. Trả về cảnh
/// báo dấu ngoặc của thân MỚI (Quyết định #3), cùng lý do [`create`] trả nó.
///
/// 🔴 **Thân KHÔNG BAO GIỜ bị từ chối vì nội dung của nó** (§Always spec 4.4) — hàm này
/// không có nhánh lỗi nào đọc `body`; mọi kiểm tra dừng ở [`scan_markers`], và
/// [`scan_markers`] chỉ CẢNH BÁO, không bao giờ `Err`.
///
/// # Lỗi
/// [`PromptSetError::WorkTierUnavailable`] — cùng [`create`]. [`PromptSetError::NotFound`]
/// nếu `(tier, id)` không khớp hàng nào.
pub fn update_body(
    global: &Store,
    work: Option<&Store>,
    tier: PromptSetTier,
    id: i64,
    body: &str,
) -> Result<MarkerWarnings, PromptSetError> {
    let store = store_for_tier(global, work, tier)?;
    let body = body.to_owned();
    let warnings = scan_markers(&body);

    let changed = store
        .write(move |tx: &Transaction<'_>| tx.execute("UPDATE prompt_set SET body = ?1 WHERE id = ?2", (&body, id)))?;

    if changed == 0 {
        return Err(PromptSetError::NotFound);
    }
    Ok(warnings)
}

/// Xoá bộ `id` ở tầng `tier`. Nếu bộ này đang CHE một bộ Global cùng tên (Quyết định #1),
/// xoá bộ Work không chạm gì tới hàng Global — bộ Global trở lại hiệu lực ngay ở lượt
/// [`resolve_two_tiers`] tiếp theo, không cần một thao tác riêng nào (I/O Matrix: "Delete a
/// shadowing Work set ... the Global set becomes effective").
///
/// # Lỗi
/// [`PromptSetError::WorkTierUnavailable`] — cùng [`create`]. [`PromptSetError::NotFound`]
/// nếu `(tier, id)` không khớp hàng nào.
pub fn delete(
    global: &Store,
    work: Option<&Store>,
    tier: PromptSetTier,
    id: i64,
) -> Result<(), PromptSetError> {
    let store = store_for_tier(global, work, tier)?;

    let changed =
        store.write(move |tx: &Transaction<'_>| tx.execute("DELETE FROM prompt_set WHERE id = ?1", [id]))?;

    if changed == 0 {
        return Err(PromptSetError::NotFound);
    }
    Ok(())
}

/// Lỗi domain của bộ prompt — RIÊNG, không tái dùng `ScopeError` (đó là lỗi Glossary-only
/// theo quyết định đã ghi, `deferred-work.md:6073`), cùng khuôn
/// `core::aiconfig::store::AiConfigStoreError`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PromptSetError {
    Store(StoreError),
    /// `ScopeResolver::apply_override` từ chối — lỗi LẬP TRÌNH, không nên xảy ra trên
    /// đường gọi đúng.
    Scope(ScopeError),
    /// Chọn tầng Tác phẩm khi chưa mở Tác phẩm nào.
    WorkTierUnavailable,
    /// Tên rỗng/toàn khoảng trắng — [`validate_name`] đã từ chối trước khi chạm SQL.
    InvalidName,
    /// `name` đã tồn tại ở tầng đang ghi — hàng đang có KHÔNG bị đụng tới.
    NameTaken { name: String },
    /// `(tier, id)` không khớp hàng nào — mục đã bị xoá/đổi tầng ở nơi khác giữa chừng.
    NotFound,
}

impl std::fmt::Display for PromptSetError {
    /// KHÔNG DẤU (NFR16) — chẩn đoán cho log, không phải văn bản hiển thị.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PromptSetError::Store(e) => write!(f, "promptset[store] {e}"),
            PromptSetError::Scope(e) => write!(f, "promptset[scope] {e}"),
            PromptSetError::WorkTierUnavailable => write!(f, "promptset[work_tier_unavailable]"),
            PromptSetError::InvalidName => write!(f, "promptset[invalid_name]"),
            PromptSetError::NameTaken { name } => write!(f, "promptset[name_taken] name={name}"),
            PromptSetError::NotFound => write!(f, "promptset[not_found]"),
        }
    }
}

impl std::error::Error for PromptSetError {}

impl From<StoreError> for PromptSetError {
    fn from(e: StoreError) -> Self {
        PromptSetError::Store(e)
    }
}

impl From<ScopeError> for PromptSetError {
    fn from(e: ScopeError) -> Self {
        PromptSetError::Scope(e)
    }
}

impl From<InvalidName> for PromptSetError {
    fn from(_: InvalidName) -> Self {
        PromptSetError::InvalidName
    }
}

/// Hình dạng IPC của [`PromptSetError`] — cùng khuôn
/// `core::aiconfig::store::AiConfigStoreError` (`Store` đi qua `From<StoreError>` sẵn có,
/// bốn biến thể còn lại dựng `IpcError` qua `IpcError::new` DUY NHẤT, không struct literal).
impl From<PromptSetError> for IpcError {
    fn from(err: PromptSetError) -> Self {
        match err {
            PromptSetError::Store(e) => e.into(),
            PromptSetError::Scope(_) => IpcError::new(
                "prompt_set.scope_error",
                MessageKey::PromptSetScopeError,
                BTreeMap::new(),
                false,
            ),
            PromptSetError::WorkTierUnavailable => IpcError::new(
                "prompt_set.work_tier_unavailable",
                MessageKey::PromptSetWorkTierUnavailable,
                BTreeMap::new(),
                false,
            ),
            PromptSetError::InvalidName => IpcError::new(
                "prompt_set.invalid_name",
                MessageKey::PromptSetInvalidName,
                BTreeMap::new(),
                false,
            ),
            PromptSetError::NameTaken { name } => {
                let mut params = BTreeMap::new();
                params.insert("name".to_owned(), name);
                IpcError::new("prompt_set.name_taken", MessageKey::PromptSetNameTaken, params, false)
            }
            PromptSetError::NotFound => IpcError::new(
                "prompt_set.not_found",
                MessageKey::PromptSetNotFound,
                BTreeMap::new(),
                false,
            ),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validate_name_rejects_blank_and_the_three_thousand_space_but_trims_a_real_value() {
        assert_eq!(validate_name("Xianxia"), Ok("Xianxia".to_owned()));
        assert_eq!(validate_name("  Xianxia  "), Ok("Xianxia".to_owned()));
        assert_eq!(validate_name(""), Err(InvalidName));
        assert_eq!(validate_name("   "), Err(InvalidName));
        assert_eq!(validate_name("\u{3000}"), Err(InvalidName));
        assert_eq!(validate_name("\t"), Err(InvalidName));
    }
}
