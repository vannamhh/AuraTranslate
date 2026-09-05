//! SQL của `import_cleanup_rule` — nạp/ghi hai tầng, phân giải qua `ScopeResolver::apply_merge`
//! (Story 6.5, AD-18).
//!
//! ─────────────────────────────────────────────────────────────────────────────
//! MODULE NÀY KHÔNG GÕ TÊN `ScopeKind`/`Semantics`/`Tier` — cùng luật mọi module miền khác,
//! SIẾT CHẶT HƠN tiền lệ `core::glossary::store` (Story 6.5)
//! ─────────────────────────────────────────────────────────────────────────────
//! `tests/scope_boundary.rs::FORBIDDEN_OUTSIDE_SCOPE` cấm `Semantics`/`ScopeKind` ngoài
//! `core/scope/**`; `ScopeResolver::apply_merge` nhận `kind: &str` đúng để chỗ này gọi bằng
//! một hằng literal ([`CLEANUP_RULE_SCOPE_KIND`]) mà không phải `use` kiểu đó.
//!
//! `core::glossary::store` (tiền lệ cũ, Story 3.x) `use crate::core::scope::Tier as ScopeTier`
//! rồi `match` trên nó để dựng `GlossaryTier` — token `Tier` KHÔNG nằm trong danh sách cấm
//! của `scope_boundary.rs`, nên cách đó biên dịch sạch. Module NÀY đi một bước siết hơn
//! (`tests/cleanup_boundary.rs` khoá riêng: `core/cleanup/**` mang 0 dòng gõ bản thân token
//! `Tier`, không chỉ `ScopeKind`/`Semantics`): [`tier_from_scope_wire`] chuyển đổi qua
//! `Tiered::tier().as_str()` — một chuỗi `"global"`/`"work"` — rồi
//! [`super::CleanupRuleTier::from_wire`], KHÔNG BAO GIỜ đặt tên kiểu `core::scope::Tier`
//! trong mã của module này. Đây là siết chặt có chủ, không phải một lượt tình cờ khác đi.
//!
//! Kết quả cho dù đi đường nào cũng THUẦN NHẤT: hàng trả về luôn mang
//! [`super::CleanupRuleTier`] RIÊNG, không bao giờ rò `core::scope::Tier` ra ngoài
//! `resolve_two_tiers`.

use std::collections::BTreeMap;

use crate::core::i18n::IpcError;
use crate::core::scope::{ScopeError, ScopeResolver};
use crate::core::store::{ReadHandle, SqlResult, Store, StoreError, Transaction};

use super::{CleanupRule, CleanupRuleKind, CleanupRuleTier};

/// Khoá dây của `ScopeKind::ImportCleanupRule` (`core/scope/kinds.rs:192-198`), chép lại
/// đây làm literal — module này không được `use` `ScopeKind`.
const CLEANUP_RULE_SCOPE_KIND: &str = "import_cleanup_rule";

/// Chuyển một `&str` (`Tiered::tier().as_str()`, xem doc-comment đầu tệp cho lý do đi qua
/// chuỗi chứ không qua kiểu) thành [`CleanupRuleTier`]. Chỉ hai giá trị `core::scope::Tier`
/// từng phát ra (`"global"`/`"work"`) — nhánh còn lại KHÔNG THỂ xảy ra trên đường gọi đúng,
/// nhưng rơi về `Global` thay vì `panic!`/`unreachable!` (`panic = "abort"` giết cả tiến
/// trình) là cách an toàn hơn cho một giá trị không đến từ input người dùng.
fn tier_from_scope_wire(wire: &str) -> CleanupRuleTier {
    CleanupRuleTier::from_wire(wire).unwrap_or(CleanupRuleTier::Global)
}

/// Một hàng `import_cleanup_rule` ĐÃ NẠP — **CHƯA gắn tầng**, đúng hình dạng
/// `ScopeResolver::apply_merge` đòi ở tham số `global`/`work` (`&[V]`). Tầng được gán SAU,
/// đọc từ `Tiered::tier()` — không phải một trường tự khai trên chính hàng, vì `tier` là
/// thuộc tính của KHO đang giữ nó, không phải của bản ghi.
#[derive(Debug, Clone, PartialEq, Eq)]
struct RawRule {
    id: i64,
    pattern: String,
    kind: CleanupRuleKind,
    enabled: bool,
}

/// `kind` trên đĩa không khớp `CHECK` — cùng lớp lỗi `core::glossary::store::decode_category`:
/// một chuỗi lạ ở đây chỉ xảy ra nếu đĩa bị sửa ngoài đường của module này.
fn decode_kind(raw: &str) -> SqlResult<CleanupRuleKind> {
    CleanupRuleKind::from_wire(raw).ok_or_else(|| {
        crate::core::store::SqlError::FromSqlConversionFailure(
            2,
            crate::core::store::SqlType::Text,
            format!("import_cleanup_rule.kind tren dia khong khop CHECK -- gia tri: {raw:?}")
                .into(),
        )
    })
}

const SELECT_RULES: &str =
    "SELECT id, pattern, kind, enabled FROM import_cleanup_rule ORDER BY ord, id";

fn row_to_raw(row: &crate::core::store::Row<'_>) -> SqlResult<RawRule> {
    let kind_raw: String = row.get(2)?;
    let enabled_raw: i64 = row.get(3)?;
    Ok(RawRule {
        id: row.get(0)?,
        pattern: row.get(1)?,
        kind: decode_kind(&kind_raw)?,
        enabled: enabled_raw != 0,
    })
}

/// Nạp toàn bộ MỘT tầng — cùng khuôn hàm tương ứng của `core::glossary::store` (tên KHÁC
/// ở đây có chủ ý — `tests/glossary_boundary.rs::GLOSSARY_ONLY_SURFACE` cấm CHÍNH CHUỖI
/// `load_tier` xuất hiện ngoài `core/glossary/**`, kể cả ở một module không liên quan). Riêng
/// module (không `pub`) — `RawRule` là kiểu NỘI BỘ, chưa gắn tầng; chỗ gọi ngoài
/// `core::cleanup::store` luôn đi qua [`resolve_two_tiers`]/[`list_tier`].
fn load_cleanup_rule_rows(store: &Store) -> Result<Vec<RawRule>, StoreError> {
    store.read(|conn: ReadHandle<'_>| {
        let mut stmt = conn.prepare(SELECT_RULES)?;
        let rows = stmt.query_map([], |row| row_to_raw(row))?;
        rows.collect::<SqlResult<Vec<RawRule>>>()
    })
}

/// Nạp CẢ HAI tầng rồi hợp nhất qua `ScopeResolver::apply_merge` — chỗ gọi sản phẩm ĐẦU
/// TIÊN của `Semantics::Merge` (§Design Notes spec 6.5: "story này là consumer sản phẩm
/// đầu tiên của `Merge`").
///
/// `primary: None` — không khoá chính nào khác ngoài tầng; hai luật cùng tầng giữ nguyên
/// thứ tự nạp (`ord, id`) vì `sort_by` của `resolve_merge` ổn định.
///
/// # Lỗi
/// [`CleanupStoreError::Store`] nếu một trong hai lượt [`load_cleanup_rule_rows`] thất bại;
/// [`CleanupStoreError::Scope`] nếu `apply_merge` từ chối (lỗi lập trình, không nên xảy ra
/// trên đường gọi đúng — `CLEANUP_RULE_SCOPE_KIND` đã khớp `Semantics::Merge`).
pub fn resolve_two_tiers(
    resolver: &ScopeResolver,
    global: &Store,
    work: Option<&Store>,
) -> Result<Vec<CleanupRule>, CleanupStoreError> {
    let global_rows = load_cleanup_rule_rows(global)?;
    let work_rows = work.map(load_cleanup_rule_rows).transpose()?;

    let tiered = resolver.apply_merge(
        CLEANUP_RULE_SCOPE_KIND,
        &global_rows,
        work_rows.as_deref(),
        None,
    )?;

    Ok(tiered
        .into_iter()
        .map(|t| {
            let tier = tier_from_scope_wire(t.tier().as_str());
            let raw = t.value();
            CleanupRule {
                tier,
                id: raw.id,
                pattern: raw.pattern.clone(),
                kind: raw.kind,
                enabled: raw.enabled,
            }
        })
        .collect())
}

/// Trim + rào rỗng, TẦNG LỆNH — cùng lý do `GLOSSARY_ENTRY_DDL`: hai lớp phải cùng nói một
/// ngôn ngữ, `str::trim()` cắt 25 điểm mã `White_Space`, `CHECK` của DDL liệt cùng tập.
/// Regex biên dịch thử NGAY Ở ĐÂY (trước khi ghi) — điều kiện để [`super::CleanupError`]
/// không nên xảy ra trên đường sản phẩm.
fn validate_pattern(kind: CleanupRuleKind, pattern: &str) -> Result<String, CleanupStoreError> {
    let trimmed = pattern.trim().to_owned();
    if trimmed.is_empty() {
        return Err(CleanupStoreError::EmptyPattern);
    }
    if kind == CleanupRuleKind::Regex
        && crate::core::cleanup::compile_cleanup_regex(&trimmed).is_err()
    {
        return Err(CleanupStoreError::InvalidRegex);
    }
    Ok(trimmed)
}

/// Liệt kê một tầng — dùng bởi lệnh `cleanup.list_rules` (không mang số đếm theo Chương;
/// đó là dữ liệu của lượt xem trước, xem `commands::project::preview_import_encoding`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CleanupRuleRow {
    pub id: i64,
    pub pattern: String,
    pub kind: CleanupRuleKind,
    pub enabled: bool,
}

impl From<RawRule> for CleanupRuleRow {
    fn from(r: RawRule) -> Self {
        Self { id: r.id, pattern: r.pattern, kind: r.kind, enabled: r.enabled }
    }
}

/// Đọc một tầng (`&Store` đã chọn) — hàm THUẦN, `commands::cleanup` định tuyến `&Store`
/// theo `CleanupRuleTier` mà `wire::` nhận được.
pub fn list_tier(store: &Store) -> Result<Vec<CleanupRuleRow>, StoreError> {
    Ok(load_cleanup_rule_rows(store)?.into_iter().map(CleanupRuleRow::from).collect())
}

/// Thêm một luật mới — **hàm thuần**. Mẫu rỗng/chỉ khoảng trắng hoặc regex hỏng bị từ chối
/// TRƯỚC khi mở giao dịch — bảng không đổi một hàng (§Always spec 6.5).
pub fn add_rule(store: &Store, pattern: &str, kind: CleanupRuleKind) -> Result<i64, CleanupStoreError> {
    let pattern = validate_pattern(kind, pattern)?;
    let kind_str = kind.as_str();

    let id = store.write(move |tx: &Transaction<'_>| {
        tx.execute(
            "INSERT INTO import_cleanup_rule (pattern, kind, enabled, ord, created_at) \
             VALUES (?1, ?2, 1, \
                      (SELECT COALESCE(MAX(ord), 0) + 1 FROM import_cleanup_rule), \
                      strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))",
            (&pattern, kind_str),
        )?;
        Ok(tx.last_insert_rowid())
    })?;

    Ok(id)
}

/// Sửa mẫu/hình dạng của luật `id` — **hàm thuần**. `id` không khớp hàng nào ⇒
/// [`CleanupStoreError::RuleMissing`] (đúng lớp lỗi *rỗng im lặng* mà
/// `core::glossary::store::confirm_translation` đã đóng cho Glossary).
pub fn edit_rule(
    store: &Store,
    id: i64,
    pattern: &str,
    kind: CleanupRuleKind,
) -> Result<(), CleanupStoreError> {
    let pattern = validate_pattern(kind, pattern)?;
    let kind_str = kind.as_str();

    let changed = store.write(move |tx: &Transaction<'_>| {
        tx.execute(
            "UPDATE import_cleanup_rule SET pattern = ?1, kind = ?2 WHERE id = ?3",
            (&pattern, kind_str, id),
        )
    })?;

    if changed == 0 {
        return Err(CleanupStoreError::RuleMissing);
    }
    Ok(())
}

/// Bật/tắt luật `id` — **hàm thuần**. `id` không khớp hàng nào ⇒
/// [`CleanupStoreError::RuleMissing`].
pub fn set_enabled(store: &Store, id: i64, enabled: bool) -> Result<(), CleanupStoreError> {
    let changed = store.write(move |tx: &Transaction<'_>| {
        tx.execute(
            "UPDATE import_cleanup_rule SET enabled = ?1 WHERE id = ?2",
            (i64::from(enabled), id),
        )
    })?;

    if changed == 0 {
        return Err(CleanupStoreError::RuleMissing);
    }
    Ok(())
}

/// Xoá luật `id` — **hàm thuần**, thao tác VÔ HẠI cho một `id` đã biến mất (cùng khuôn
/// `commands::pinned::unpin_entry`: xoá một mục chưa/không còn tồn tại không phải một lỗi).
pub fn delete_rule(store: &Store, id: i64) -> Result<(), CleanupStoreError> {
    store.write(move |tx: &Transaction<'_>| {
        tx.execute("DELETE FROM import_cleanup_rule WHERE id = ?1", [id])
    })?;
    Ok(())
}

/// Hai họ lỗi gặp nhau ở [`resolve_two_tiers`], cộng bốn lỗi tầng lệnh CRUD — cùng khuôn
/// `core::glossary::store::GlossaryError`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CleanupStoreError {
    Store(StoreError),
    /// `ScopeResolver::apply_merge` từ chối — lỗi LẬP TRÌNH, không nên xảy ra trên đường
    /// gọi đúng (xem doc-comment [`ScopeError`]). KHÔNG BAO GIỜ vượt ranh giới IPC với
    /// `Display` của nó — cùng luật `GlossaryError::Scope`.
    Scope(ScopeError),
    /// Mẫu rỗng/chỉ khoảng trắng sau khi trim.
    EmptyPattern,
    /// Mẫu `regex` không biên dịch được.
    InvalidRegex,
    /// `(tier, id)` không khớp hàng nào — sửa/bật-tắt một luật đã biến mất.
    RuleMissing,
    /// Chọn tầng Tác phẩm khi chưa mở Tác phẩm nào — cùng khuôn
    /// `GlossaryError::WorkTierUnavailable`.
    WorkTierUnavailable,
}

impl std::fmt::Display for CleanupStoreError {
    /// KHÔNG DẤU (NFR16).
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CleanupStoreError::Store(e) => write!(f, "cleanup[store] {e}"),
            CleanupStoreError::Scope(e) => write!(f, "cleanup[scope] {e}"),
            CleanupStoreError::EmptyPattern => write!(f, "cleanup[empty_pattern]"),
            CleanupStoreError::InvalidRegex => write!(f, "cleanup[invalid_regex]"),
            CleanupStoreError::RuleMissing => write!(f, "cleanup[rule_missing]"),
            CleanupStoreError::WorkTierUnavailable => write!(f, "cleanup[work_tier_unavailable]"),
        }
    }
}

impl std::error::Error for CleanupStoreError {}

impl From<StoreError> for CleanupStoreError {
    fn from(e: StoreError) -> Self {
        CleanupStoreError::Store(e)
    }
}

impl From<ScopeError> for CleanupStoreError {
    fn from(e: ScopeError) -> Self {
        CleanupStoreError::Scope(e)
    }
}

impl From<CleanupStoreError> for IpcError {
    fn from(err: CleanupStoreError) -> Self {
        use crate::core::i18n::MessageKey;

        match err {
            CleanupStoreError::Store(e) => e.into(),
            CleanupStoreError::Scope(_) => {
                IpcError::new("cleanup.scope_error", MessageKey::CleanupScopeError, BTreeMap::new(), false)
            }
            CleanupStoreError::EmptyPattern => IpcError::new(
                "cleanup.empty_pattern",
                MessageKey::CleanupEmptyPattern,
                BTreeMap::new(),
                false,
            ),
            CleanupStoreError::InvalidRegex => IpcError::new(
                "cleanup.invalid_regex",
                MessageKey::CleanupInvalidRegex,
                BTreeMap::new(),
                false,
            ),
            CleanupStoreError::RuleMissing => IpcError::new(
                "cleanup.rule_missing",
                MessageKey::CleanupRuleMissing,
                BTreeMap::new(),
                false,
            ),
            CleanupStoreError::WorkTierUnavailable => IpcError::new(
                "cleanup.work_tier_unavailable",
                MessageKey::CleanupWorkTierUnavailable,
                BTreeMap::new(),
                false,
            ),
        }
    }
}
