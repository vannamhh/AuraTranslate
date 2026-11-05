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
use crate::core::scope::{
    KEY_PROOFREAD_PROMPT_SET, ScopeError, ScopeResolver, delete_value, load_global_config,
    save_value,
};
use crate::core::store::{
    ReadHandle, SqlError, SqlType, Store, StoreError, Transaction, is_unique_constraint_violation,
};

use super::exchange::PlanKind;
use super::vars::{MarkerWarnings, scan_markers};
use super::{PromptSet, PromptSetKind, PromptSetTier};

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
/// giống hệt việc lách qua `confirmed_terms_for_injection`. Đặt tên dài hơn ở đây rẻ hơn nới
/// cổng đó.
pub fn load_prompt_set_tier(
    store: &Store,
    kind: PromptSetKind,
) -> Result<BTreeMap<String, PromptSet>, StoreError> {
    store.read(|conn: ReadHandle<'_>| {
        let mut stmt =
            conn.prepare("SELECT id, name, body FROM prompt_set WHERE kind = ?1 ORDER BY name")?;
        let mut rows = stmt.query([kind.as_str()])?;

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
    /// 🔵 **THÊM Story 4.5, Quyết định #3.** `id` THẬT của hàng Global đang bị che —
    /// `deferred-work.md §*Deferred from: 4-2-cau-hinh-nha-cung-cap-ai (2026-09-16)*` ghi rằng thiếu trường này là lý do hàng đó CHỈ hiện
    /// được, không thao tác được (không sửa/xoá/đổi tên/xuất thẳng nó qua kết quả này). Luôn
    /// mang tầng [`PromptSetTier::Global`] khi `Some` — một Work-tier row chỉ có thể che một
    /// Global-tier row cùng tên (Quyết định #1), không bao giờ ngược lại. `None` cùng điều
    /// kiện với `shadowed_body`.
    pub shadowed_id: Option<i64>,
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
    kind: PromptSetKind,
) -> Result<BTreeMap<String, ResolvedPromptSet>, PromptSetError> {
    let global_tier = load_prompt_set_tier(global, kind)?;
    let work_tier = work.map(|w| load_prompt_set_tier(w, kind)).transpose()?;

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
                PromptSetError::Scope(ScopeError::UnknownKind {
                    wire: wire.to_owned(),
                })
            })?;
            let shadowed_body = r.shadowed().map(|s| s.body.clone());
            let shadowed_id = r.shadowed().map(|s| s.id);
            let id = r.value().id;
            let body = r.value().body.clone();
            Ok((
                name.clone(),
                ResolvedPromptSet {
                    id,
                    name,
                    body,
                    tier,
                    shadowed_body,
                    shadowed_id,
                },
            ))
        })
        .collect()
}

/// Chọn kho theo `tier` — khuôn chép `core::glossary::store::add_manual_term`.
///
/// 🔵 **THÊM Story 4.5: `pub(crate)`, không đổi thân.** `commands::promptset::prompt_set_export`
/// cần routing NÀY để chọn kho cho [`load_one`] (hàm đó nhận một `&Store` đã chốt, không
/// `(global, work, tier)` như `create`/`rename`/…) — gọi thẳng hàm này qua đường dẫn đủ
/// (`crate::core::promptset::store::store_for_tier`) rẻ hơn chép lại hai nhánh `match` ở
/// lớp lệnh, đúng lý lẽ "tier routing lives in core, not here" mà doc-comment đầu
/// `commands/promptset.rs` đã ghi.
pub(crate) fn store_for_tier<'a>(
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
    kind: PromptSetKind,
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
            "INSERT INTO prompt_set (kind, name, body, created_at) \
             VALUES (?1, ?2, ?3, strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))",
            (kind.as_str(), &name_for_write, &body),
        ) {
            Ok(_) => Ok(tx.last_insert_rowid()),
            Err(e) if is_unique_constraint_violation(&e) => {
                *collided_for_closure
                    .lock()
                    .unwrap_or_else(|p| p.into_inner()) = true;
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
    kind: PromptSetKind,
    id: i64,
    new_name: &str,
) -> Result<(), PromptSetError> {
    let store = store_for_tier(global, work, tier)?;
    let new_name = validate_name(new_name)?;

    let collided = Arc::new(Mutex::new(false));
    let collided_for_closure = Arc::clone(&collided);
    let name_for_write = new_name.clone();

    let result = store.write(move |tx: &Transaction<'_>| {
        match tx.execute(
            "UPDATE prompt_set SET name = ?1 WHERE id = ?2 AND kind = ?3",
            (&name_for_write, id, kind.as_str()),
        ) {
            Ok(changed) => Ok(changed),
            Err(e) if is_unique_constraint_violation(&e) => {
                *collided_for_closure
                    .lock()
                    .unwrap_or_else(|p| p.into_inner()) = true;
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
    kind: PromptSetKind,
    id: i64,
    body: &str,
) -> Result<MarkerWarnings, PromptSetError> {
    let store = store_for_tier(global, work, tier)?;
    let body = body.to_owned();
    let warnings = scan_markers(&body);

    let changed = store.write(move |tx: &Transaction<'_>| {
        tx.execute(
            "UPDATE prompt_set SET body = ?1 WHERE id = ?2 AND kind = ?3",
            (&body, id, kind.as_str()),
        )
    })?;

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
    kind: PromptSetKind,
    id: i64,
) -> Result<(), PromptSetError> {
    let store = store_for_tier(global, work, tier)?;

    let changed = store.write(move |tx: &Transaction<'_>| {
        tx.execute(
            "DELETE FROM prompt_set WHERE id = ?1 AND kind = ?2",
            (id, kind.as_str()),
        )
    })?;

    if changed == 0 {
        return Err(PromptSetError::NotFound);
    }
    Ok(())
}

/// `work.chosen_prompt_set_name` of a Work store; `None` is no hand pick.
pub fn read_chosen_name(work: &Store) -> Result<Option<String>, StoreError> {
    work.read(|conn| {
        conn.query_row(
            "SELECT chosen_prompt_set_name FROM work WHERE id = 1",
            [],
            |r| r.get(0),
        )
    })
}

/// Sets `work.chosen_prompt_set_name` and nothing else: no `updated_at`, no `meta.json`.
/// A blank name is stored as `NULL`.
pub fn write_chosen_name(work: &Store, name: Option<&str>) -> Result<(), StoreError> {
    let value = name
        .map(str::trim)
        .filter(|n| !n.is_empty())
        .map(str::to_owned);
    work.write(move |tx: &Transaction<'_>| {
        tx.execute(
            "UPDATE work SET chosen_prompt_set_name = ?1 WHERE id = 1",
            [value],
        )
    })?;
    Ok(())
}

/// Work-tier choice of the proofread instruction set (AD-68 rule 3), three states.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProofreadWorkChoice {
    FollowGlobal,
    Off,
    Named(String),
}

pub fn read_proofread_work_choice(work: &Store) -> Result<ProofreadWorkChoice, StoreError> {
    let (name, off): (Option<String>, i64) = work.read(|conn| {
        conn.query_row(
            "SELECT chosen_proofread_prompt_set_name, proofread_instruction_off \
             FROM work WHERE id = 1",
            [],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
    })?;
    Ok(match (off, name) {
        (0, None) => ProofreadWorkChoice::FollowGlobal,
        (0, Some(name)) => ProofreadWorkChoice::Named(name),
        _ => ProofreadWorkChoice::Off,
    })
}

/// Writes both choice columns in one `UPDATE`. `Off` and `FollowGlobal` store a `NULL` name;
/// a blank `Named` is rejected, it never means anything.
pub fn write_proofread_work_choice(
    work: &Store,
    choice: &ProofreadWorkChoice,
) -> Result<(), PromptSetError> {
    let (name, off): (Option<String>, i64) = match choice {
        ProofreadWorkChoice::FollowGlobal => (None, 0),
        ProofreadWorkChoice::Off => (None, 1),
        ProofreadWorkChoice::Named(name) => (Some(validate_name(name)?), 0),
    };
    work.write(move |tx: &Transaction<'_>| {
        tx.execute(
            "UPDATE work SET chosen_proofread_prompt_set_name = ?1, \
             proofread_instruction_off = ?2 WHERE id = 1",
            (name, off),
        )
    })?;
    Ok(())
}

/// `None` removes the Global choice; a blank name is rejected.
pub fn write_proofread_global_choice(
    global: &Store,
    name: Option<&str>,
) -> Result<(), PromptSetError> {
    match name {
        None => delete_value(global, "app_config", KEY_PROOFREAD_PROMPT_SET)?,
        Some(name) => save_value(
            global,
            "app_config",
            KEY_PROOFREAD_PROMPT_SET,
            &validate_name(name)?,
        )?,
    }
    Ok(())
}

/// The proofread instruction a scan runs with: the set that was resolved, with the tier it
/// resolved to.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProofreadInstruction {
    pub name: String,
    pub tier: PromptSetTier,
    pub body: String,
}

/// Chosen name and the tier that holds the choice (AD-68 rule 3 order). `None` is no choice or
/// the Work switched the instruction off.
fn proofread_choice_in_effect(
    global: &Store,
    work: Option<&Store>,
) -> Result<Option<(String, PromptSetTier)>, PromptSetError> {
    let work_choice = work.map(read_proofread_work_choice).transpose()?;
    Ok(match work_choice {
        Some(ProofreadWorkChoice::Off) => None,
        Some(ProofreadWorkChoice::Named(name)) => Some((name, PromptSetTier::Work)),
        Some(ProofreadWorkChoice::FollowGlobal) | None => load_global_config(global)?
            .proofread_prompt_set()
            .map(|name| (name, PromptSetTier::Global)),
    })
}

/// Resolves the instruction once per scan. No choice, a switched-off Work, or a body that is
/// blank after trimming is `Ok(None)`.
///
/// # Errors
/// [`PromptSetError::ProofreadSetMissing`] when the chosen name resolves in neither tier.
pub fn resolve_proofread_instruction(
    resolver: &ScopeResolver,
    global: &Store,
    work: Option<&Store>,
) -> Result<Option<ProofreadInstruction>, PromptSetError> {
    let Some((name, selected_at)) = proofread_choice_in_effect(global, work)? else {
        return Ok(None);
    };
    let mut sets = resolve_two_tiers(resolver, global, work, PromptSetKind::Proofread)?;
    let Some(set) = sets.remove(&name) else {
        return Err(PromptSetError::ProofreadSetMissing { name, selected_at });
    };
    if set.body.trim().is_empty() {
        return Ok(None);
    }
    Ok(Some(ProofreadInstruction {
        name: set.name,
        tier: set.tier,
        body: set.body,
    }))
}

/// Renames a `proofread` set and moves the choices that resolved to it (AD-68 rule 4). The
/// store is written first, the choices after. The open Work's choice follows only when the
/// renamed set is the one that choice resolves to; the Global choice follows only for a
/// Global set.
pub fn rename_proofread_set(
    global: &Store,
    work: Option<&Store>,
    tier: PromptSetTier,
    id: i64,
    new_name: &str,
) -> Result<(), PromptSetError> {
    let kind = PromptSetKind::Proofread;
    let store = store_for_tier(global, work, tier)?;
    let old_name = load_one(store, kind, id)?.name;
    rename(global, work, tier, kind, id, new_name)?;
    let new_name = load_one(store, kind, id)?.name;
    if old_name == new_name {
        return Ok(());
    }

    if tier == PromptSetTier::Global
        && load_global_config(global)?
            .proofread_prompt_set()
            .as_deref()
            == Some(old_name.as_str())
    {
        write_proofread_global_choice(global, Some(&new_name))?;
    }
    if let Some(work) = work {
        let work_names = if tier == PromptSetTier::Global {
            load_prompt_set_tier(work, kind)?
        } else {
            BTreeMap::new()
        };
        if !work_names.contains_key(&old_name) && !work_names.contains_key(&new_name) {
            work.write(move |tx: &Transaction<'_>| {
                tx.execute(
                    "UPDATE work SET chosen_proofread_prompt_set_name = ?1 \
                     WHERE id = 1 AND chosen_proofread_prompt_set_name = ?2",
                    (&new_name, &old_name),
                )
            })?;
        }
    }
    Ok(())
}

/// What the next scan meets after a `proofread` set is deleted (AD-68 rule 4).
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ProofreadDeleteConsequence {
    None,
    GlobalTakesOver,
    SetMissing,
}

/// Consequence of deleting the `proofread` set `(tier, id)`; depends on the deleted set's tier.
/// A Work set: the Global set of the same name takes over if there is one, else the scan
/// reports a missing set. A Global set: the scan reports a missing set, for a Work with the
/// same name is not affected.
pub fn proofread_delete_consequence(
    global: &Store,
    work: Option<&Store>,
    tier: PromptSetTier,
    id: i64,
) -> Result<ProofreadDeleteConsequence, PromptSetError> {
    let kind = PromptSetKind::Proofread;
    let name = load_one(store_for_tier(global, work, tier)?, kind, id)?.name;
    let global_choice = load_global_config(global)?.proofread_prompt_set();
    let work_choice = work.map(read_proofread_work_choice).transpose()?;

    Ok(match tier {
        PromptSetTier::Work => {
            let chosen = match work_choice {
                Some(ProofreadWorkChoice::Named(n)) => n == name,
                Some(ProofreadWorkChoice::FollowGlobal) => {
                    global_choice.as_deref() == Some(name.as_str())
                }
                Some(ProofreadWorkChoice::Off) | None => false,
            };
            if !chosen {
                ProofreadDeleteConsequence::None
            } else if load_prompt_set_tier(global, kind)?.contains_key(&name) {
                ProofreadDeleteConsequence::GlobalTakesOver
            } else {
                ProofreadDeleteConsequence::SetMissing
            }
        }
        PromptSetTier::Global => {
            let work_names_it = match (&work_choice, work) {
                (Some(ProofreadWorkChoice::Named(n)), Some(work)) => {
                    *n == name && !load_prompt_set_tier(work, kind)?.contains_key(&name)
                }
                _ => false,
            };
            if global_choice.as_deref() == Some(name.as_str()) || work_names_it {
                ProofreadDeleteConsequence::SetMissing
            } else {
                ProofreadDeleteConsequence::None
            }
        }
    })
}

/// Nạp MỘT hàng bằng `id` trực tiếp — Story 4.5: xuất một bộ thao tác trên hàng THẬT người
/// dùng chọn (kể cả một hàng Global đang bị che, Quyết định #3), không trên hàng "đang
/// thắng" mà [`resolve_two_tiers`] tính ra.
///
/// # Lỗi
/// [`PromptSetError::NotFound`] nếu `id` không khớp hàng nào trong `store`.
pub fn load_one(store: &Store, kind: PromptSetKind, id: i64) -> Result<PromptSet, PromptSetError> {
    // 🔴 `QueryReturnedNoRows` bat NGAY TRONG closure, truoc khi no tro thanh mot
    // `StoreError` da bi chuoi hoa -- cung khuon `core::dict::layer`/`core::library::indexer`
    // (`Err(SqlError::QueryReturnedNoRows) => Ok(None)`), vi `Store::read` khong tai xuat
    // `SqlError` o tang loi cua no.
    let row: Option<PromptSet> = store.read(|conn: ReadHandle<'_>| {
        match conn.query_row(
            "SELECT id, name, body FROM prompt_set WHERE id = ?1 AND kind = ?2",
            (id, kind.as_str()),
            |row| {
                Ok(PromptSet {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    body: row.get(2)?,
                })
            },
        ) {
            Ok(v) => Ok(Some(v)),
            Err(SqlError::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e),
        }
    })?;

    row.ok_or(PromptSetError::NotFound)
}

/// Kết quả THẬT của một lượt [`import_into_tier`] — Story 4.5.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ImportOutcome {
    /// `name` chưa có ở tầng đích — hàng mới được chèn.
    Inserted { id: i64 },
    /// Va chạm tên, quyết định [`super::exchange::ConflictDecision::TakeTheirs`] — thân của
    /// hàng đang có được thay.
    Updated { id: i64 },
    /// Không ghi gì — hoặc [`PlanKind::Identical`], hoặc va chạm tên với quyết định
    /// [`super::exchange::ConflictDecision::KeepMine`] (mặc định).
    Skipped,
}

/// Lỗi ĐÁNH DẤU để buộc `store.write` rollback khi nhánh `Conflict`/`TakeTheirs` phát hiện
/// hàng đích đã đổi dưới chân người dùng, hoặc đã biến mất, giữa nhịp xem trước và nhịp xác
/// nhận — khuôn chép `core::glossary::store::stale_conflict_marker_error`. Nội dung KHÔNG
/// bao giờ được đọc lại: kết quả thật đi qua kênh riêng (`Arc<Mutex<…>>`).
fn import_marker_error(reason: &'static str) -> SqlError {
    SqlError::FromSqlConversionFailure(0, SqlType::Text, reason.into())
}

/// Ghi kết quả MỘT lượt nhập vào `tier`, MỘT giao dịch (§Always: "Import is one transaction,
/// all-or-nothing") — Story 4.5, khuôn chép rút gọn của
/// `core::glossary::store::import_into_tier` cho đúng MỘT hàng (một tệp = một bộ prompt).
///
/// `plan.kind` được chụp lúc XEM TRƯỚC ([`super::exchange::classify`]); `decision` chỉ có ý
/// nghĩa khi `plan.kind` là [`PlanKind::Conflict`] — vắng mặt (`None`) ⇒
/// [`super::exchange::ConflictDecision::KeepMine`] (§Always: mặc định giữ của tôi, cùng luật
/// Glossary).
///
/// # Lỗi
/// [`PromptSetError::WorkTierUnavailable`] nếu `tier` là [`PromptSetTier::Work`] mà `work` là
/// `None` — kiểm TRƯỚC khi mở giao dịch, **0** lượt ghi (I/O Matrix "Import to Work, no Work
/// open ... Refused before any write"). [`PromptSetError::ImportStaleConflict`] nếu
/// `TakeTheirs` so lạc quan với `existing_body` đã chụp mà hàng đích đã đổi (hoặc bị xoá)
/// giữa hai nhịp — batch KHÔNG bị đụng bởi lỗi này, chỗ gọi (lớp lệnh) giữ nguyên lô để thử
/// lại. [`PromptSetError::NotFound`] nếu hàng đích đã bị xoá HẲN (không còn để mà so).
pub fn import_into_tier(
    global: &Store,
    work: Option<&Store>,
    tier: PromptSetTier,
    plan: &super::exchange::ParsedPromptSet,
    kind: &PlanKind,
    decision: Option<super::exchange::ConflictDecision>,
) -> Result<ImportOutcome, PromptSetError> {
    use super::exchange::ConflictDecision;

    let store = store_for_tier(global, work, tier)?;
    let name = plan.name.clone();
    let body = plan.body.clone();
    let set_kind = plan.kind;
    let kind = kind.clone();
    let decision = decision.unwrap_or(ConflictDecision::KeepMine);

    let stale = Arc::new(Mutex::new(false));
    let stale_for_closure = Arc::clone(&stale);
    let missing = Arc::new(Mutex::new(false));
    let missing_for_closure = Arc::clone(&missing);

    let result = store.write(move |tx: &Transaction<'_>| -> Result<ImportOutcome, SqlError> {
        match &kind {
            PlanKind::New => match tx.execute(
                "INSERT INTO prompt_set (kind, name, body, created_at) \
                 VALUES (?1, ?2, ?3, strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))",
                (set_kind.as_str(), &name, &body),
            ) {
                Ok(_) => Ok(ImportOutcome::Inserted { id: tx.last_insert_rowid() }),
                // Tên vua duoc mot luot ghi KHAC chen vao giua nhip xem truoc va nhip xac
                // nhan -- cung lop "da doi duoi chan nguoi dung" ma nhanh Conflict canh,
                // nen dung CHUNG nhan Stale (khong mot nhan rieng thu ba).
                Err(e) if is_unique_constraint_violation(&e) => {
                    *stale_for_closure.lock().unwrap_or_else(|p| p.into_inner()) = true;
                    Err(import_marker_error("prompt_set import -- rollback: name da bi chiem giua hai nhip"))
                }
                Err(e) => Err(e),
            },
            PlanKind::Identical => Ok(ImportOutcome::Skipped),
            PlanKind::Conflict { existing_id, existing_body } => match decision {
                ConflictDecision::KeepMine => Ok(ImportOutcome::Skipped),
                ConflictDecision::TakeTheirs => {
                    // So LAC QUAN -- `body IS ?3` (khong `= ?3`) khop ca ca NULL, cung ly do
                    // `core::glossary::store::import_into_tier` da ghi cho `translation IS ?3`.
                    // `AND name = ?4` -- hang dich bi DOI TEN (than khong doi) giua hai nhip
                    // cung phai roi vao nhanh stale, khong chi hang bi doi THAN; I/O Matrix
                    // "Collision, body moved under the user" coi ca so nay la that.
                    let changed = tx.execute(
                        "UPDATE prompt_set SET body = ?1 WHERE id = ?2 AND body IS ?3 AND name = ?4 AND kind = ?5",
                        (&body, existing_id, existing_body, &name, set_kind.as_str()),
                    )?;
                    if changed == 0 {
                        let still: Option<String> = match tx.query_row(
                            "SELECT body FROM prompt_set WHERE id = ?1 AND kind = ?2",
                            (*existing_id, set_kind.as_str()),
                            |r| r.get(0),
                        ) {
                            Ok(v) => Some(v),
                            Err(SqlError::QueryReturnedNoRows) => None,
                            Err(e) => return Err(e),
                        };
                        match still {
                            None => {
                                *missing_for_closure.lock().unwrap_or_else(|p| p.into_inner()) = true;
                                Err(import_marker_error("prompt_set import -- rollback: hang dich da bi xoa"))
                            }
                            Some(_current_body_now_different) => {
                                *stale_for_closure.lock().unwrap_or_else(|p| p.into_inner()) = true;
                                Err(import_marker_error(
                                    "prompt_set import -- rollback: than da doi duoi chan nguoi dung",
                                ))
                            }
                        }
                    } else {
                        Ok(ImportOutcome::Updated { id: *existing_id })
                    }
                }
            },
        }
    });

    match result {
        Ok(outcome) => Ok(outcome),
        Err(e) => {
            if *stale.lock().unwrap_or_else(|p| p.into_inner()) {
                return Err(PromptSetError::ImportStaleConflict {
                    name: plan.name.clone(),
                });
            }
            if *missing.lock().unwrap_or_else(|p| p.into_inner()) {
                return Err(PromptSetError::NotFound);
            }
            Err(PromptSetError::from(e))
        }
    }
}

/// Lỗi domain của bộ prompt — RIÊNG, không tái dùng `ScopeError` (đó là lỗi Glossary-only
/// theo quyết định đã ghi, `deferred-work.md §*Deferred from: 3-2-bang-cho-ung-vien-tach-han-khoi-glossary (rà soát ba lớp, 2026-08-20)*`), cùng khuôn
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
    NameTaken {
        name: String,
    },
    /// `(tier, id)` không khớp hàng nào — mục đã bị xoá/đổi tầng ở nơi khác giữa chừng.
    NotFound,
    /// The chosen proofread instruction set resolves in neither tier; `selected_at` holds the
    /// choice.
    ProofreadSetMissing {
        name: String,
        selected_at: PromptSetTier,
    },

    // ── Story 4.5 (FR79, NFR9, AD-48) — bảy biến thể MỚI, hộp thoại chọn tệp xuất/nhập ──
    /// Tệp nhập vượt trần [`super::exchange_io::MAX_PROMPT_SET_IMPORT_BYTES`] (1 MiB).
    ImportFileTooLarge {
        size: u64,
        limit: u64,
    },
    /// Nội dung tệp không giải mã được bằng UTF-8.
    ImportNotUtf8 {
        path: String,
    },
    /// Mở/đọc tệp nhập thất bại vì lý do KHÁC kích thước và bảng mã.
    ImportReadFailed {
        path: String,
        detail: String,
    },
    /// Ghi tệp xuất thất bại — `exchange_io::write_export_file` dọn `.tmp` ở CẢ HAI nhánh
    /// lỗi trước khi biến thể này được dựng, không tệp cụt nào bị để lại.
    ExportWriteFailed {
        path: String,
        detail: String,
    },
    /// `FilePath::into_path()` của `tauri-plugin-dialog` trả lỗi.
    DialogPathInvalid,
    /// Xác nhận lượt nhập (nhịp hai) khi chưa qua nhịp một, hoặc lô đã bị dọn (huỷ, một lô
    /// mới thay nó, đóng Tác phẩm khi lô đang treo mang nửa tầng Work).
    NoPendingImport,
    /// Thân của hàng đích đã đổi (hoặc chính hàng đã biến mất, hoặc — nhánh `New` — `name`
    /// vừa bị một lượt ghi KHÁC chiếm) giữa nhịp xem trước và nhịp xác nhận — `TakeTheirs` so
    /// lạc quan trượt. `name` là tên bộ đang nhập, dữ liệu không phải một câu.
    ImportStaleConflict {
        name: String,
    },
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
            PromptSetError::ProofreadSetMissing { name, selected_at } => {
                write!(
                    f,
                    "promptset[proofread_set_missing] name={name} tier={selected_at}"
                )
            }
            PromptSetError::ImportFileTooLarge { size, limit } => {
                write!(
                    f,
                    "promptset[import_file_too_large] size={size} limit={limit}"
                )
            }
            PromptSetError::ImportNotUtf8 { path } => {
                write!(f, "promptset[import_not_utf8] path={path}")
            }
            PromptSetError::ImportReadFailed { path, detail } => {
                write!(
                    f,
                    "promptset[import_read_failed] path={path} detail={detail}"
                )
            }
            PromptSetError::ExportWriteFailed { path, detail } => {
                write!(
                    f,
                    "promptset[export_write_failed] path={path} detail={detail}"
                )
            }
            PromptSetError::DialogPathInvalid => write!(f, "promptset[dialog_path_invalid]"),
            PromptSetError::NoPendingImport => write!(f, "promptset[no_pending_import]"),
            PromptSetError::ImportStaleConflict { name } => {
                write!(f, "promptset[import_stale_conflict] name={name}")
            }
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
                IpcError::new(
                    "prompt_set.name_taken",
                    MessageKey::PromptSetNameTaken,
                    params,
                    false,
                )
            }
            PromptSetError::NotFound => IpcError::new(
                "prompt_set.not_found",
                MessageKey::PromptSetNotFound,
                BTreeMap::new(),
                false,
            ),
            PromptSetError::ProofreadSetMissing { name, selected_at } => {
                let mut params = BTreeMap::new();
                params.insert("name".to_owned(), name);
                params.insert("tier".to_owned(), selected_at.as_str().to_owned());
                IpcError::new(
                    "ai_proofread.instruction_missing",
                    MessageKey::AiProofreadInstructionMissing,
                    params,
                    false,
                )
            }
            // 🔵 Story 4.5 — ba biến thể đầu MƯỢN khoá CHUNG với `core::segment::import`/
            // `core::glossary` (`MessageKey::ImportTooLarge`/`ImportNotUtf8`/`IoReadFailed`):
            // câu đúng là câu chung, không câu riêng của domain này — cùng lý lẽ Glossary đã
            // ghi tại chỗ nó mượn ba khoá này (`core/glossary/store.rs`).
            PromptSetError::ImportFileTooLarge { size: _, limit } => {
                let mut params = BTreeMap::new();
                params.insert("limit".to_owned(), limit.to_string());
                IpcError::new(
                    "prompt_set.import_file_too_large",
                    MessageKey::ImportTooLarge,
                    params,
                    false,
                )
            }
            PromptSetError::ImportNotUtf8 { path } => {
                let mut params = BTreeMap::new();
                params.insert("path".to_owned(), path);
                IpcError::new(
                    "prompt_set.import_not_utf8",
                    MessageKey::ImportNotUtf8,
                    params,
                    false,
                )
            }
            PromptSetError::ImportReadFailed { path, .. } => {
                let mut params = BTreeMap::new();
                params.insert("path".to_owned(), path);
                IpcError::new(
                    "prompt_set.import_read_failed",
                    MessageKey::IoReadFailed,
                    params,
                    false,
                )
            }
            PromptSetError::ExportWriteFailed { path, .. } => {
                let mut params = BTreeMap::new();
                params.insert("path".to_owned(), path);
                IpcError::new(
                    "prompt_set.export_write_failed",
                    MessageKey::PromptSetExportWriteFailed,
                    params,
                    false,
                )
            }
            PromptSetError::DialogPathInvalid => IpcError::new(
                "prompt_set.dialog_path_invalid",
                MessageKey::PromptSetDialogPathInvalid,
                BTreeMap::new(),
                false,
            ),
            PromptSetError::NoPendingImport => IpcError::new(
                "prompt_set.no_pending_import",
                MessageKey::PromptSetNoPendingImport,
                BTreeMap::new(),
                false,
            ),
            PromptSetError::ImportStaleConflict { name } => {
                let mut params = BTreeMap::new();
                params.insert("name".to_owned(), name);
                IpcError::new(
                    "prompt_set.import_stale_conflict",
                    MessageKey::PromptSetImportStaleConflict,
                    params,
                    false,
                )
            }
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
