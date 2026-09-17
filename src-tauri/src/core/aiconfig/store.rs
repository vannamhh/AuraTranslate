//! SQL của bảng `ai_config` — nạp/ghi hai tầng, phân giải qua `ScopeResolver::apply_override`
//! (Story 4.2, `ScopeKind::AiConfig`).
//!
//! ─────────────────────────────────────────────────────────────────────────────
//! MODULE NÀY KHÔNG GÕ TÊN `ScopeKind`/`Semantics`/`Tier` — cùng luật mọi module miền khác
//! ─────────────────────────────────────────────────────────────────────────────
//! `tests/scope_boundary.rs::FORBIDDEN_OUTSIDE_SCOPE` cấm `Semantics`/`ScopeKind` ngoài
//! `core/scope/**`; `ScopeResolver::apply_override` nhận `kind: &str` đúng để chỗ này gọi
//! bằng một hằng literal ([`AI_CONFIG_SCOPE_KIND`]) mà không phải `use` kiểu đó.

use std::collections::BTreeMap;

use crate::core::i18n::{IpcError, MessageKey};
use crate::core::scope::{ScopeError, ScopeResolver};
use crate::core::store::{ReadHandle, Store, StoreError, Transaction};

use super::{AiConfigField, AiConfigTier};

/// Khoá dây của `ScopeKind::AiConfig` (`core/scope/kinds.rs:175`), chép lại đây làm literal —
/// module này không được `use` `ScopeKind`.
const AI_CONFIG_SCOPE_KIND: &str = "ai_config";

/// Một trường ĐÃ phân giải — hình dạng RIÊNG của domain này (`AiConfigTier`, không
/// `core::scope::Tier`), cùng lý do `core::cleanup::CleanupRule`/`core::glossary::GlossaryEntry`
/// không rò `core::scope::{Tier, Resolved}` ra khỏi module của chúng.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedField {
    pub value: String,
    pub tier: AiConfigTier,
    /// Giá trị Global bị che, khi trường này đang thắng ở tầng Tác phẩm. `None` khi trường
    /// chỉ tồn tại ở một tầng, hoặc khi `tier == AiConfigTier::Global`.
    pub shadowed: Option<String>,
}

/// Nạp toàn bộ MỘT tầng (một `global.db` hoặc một `project.db`), khoá theo tên trường — hình
/// dạng `BTreeMap` mà `ScopeResolver::apply_override` đòi ở tham số `global`/`work`.
fn load_ai_config_rows(store: &Store) -> Result<BTreeMap<String, String>, StoreError> {
    store.read(|conn: ReadHandle<'_>| {
        let mut stmt = conn.prepare("SELECT key, value FROM ai_config ORDER BY key")?;
        let mut rows = stmt.query([])?;

        let mut out = BTreeMap::new();
        while let Some(row) = rows.next()? {
            out.insert(row.get::<_, String>(0)?, row.get::<_, String>(1)?);
        }
        Ok(out)
    })
}

/// Nạp CẢ HAI tầng rồi phân giải qua `ScopeResolver::apply_override` — **theo từng trường**
/// (Ice ký 2026-08-04). Một hàng Tác phẩm cho MỘT trường không được che các trường khác đang
/// kế thừa từ Global; xem [`Resolved::shadowed`] cho giá trị Global bị che khi có.
///
/// # Lỗi
/// [`AiConfigStoreError::Store`] nếu một trong hai lượt [`load_ai_config_rows`] thất bại;
/// [`AiConfigStoreError::Scope`] nếu `apply_override` từ chối (lỗi lập trình, không nên xảy
/// ra trên đường gọi đúng — `AI_CONFIG_SCOPE_KIND` đã khớp `ScopeKind::AiConfig::Override`).
pub fn resolve_two_tiers(
    resolver: &ScopeResolver,
    global: &Store,
    work: Option<&Store>,
) -> Result<BTreeMap<String, ResolvedField>, AiConfigStoreError> {
    let global_rows = load_ai_config_rows(global)?;
    let work_rows = work.map(load_ai_config_rows).transpose()?;

    let resolved =
        resolver.apply_override(AI_CONFIG_SCOPE_KIND, &global_rows, work_rows.as_ref())?;

    resolved
        .into_iter()
        .map(|(key, r)| {
            // 🔴 `?`, KHÔNG `unwrap_or(AiConfigTier::Global)` — chỉ hai giá trị
            // `core::scope::Tier::as_str()` từng phát ra ("global"/"work"), nên nhánh này
            // không nên xảy ra trên đường gọi đúng, nhưng rơi về `Global` im lặng cho một
            // giá trị lạ là đúng lớp lỗi *"trông như đang chạy"* mà dự án cấm — trả lỗi thay
            // vì đoán. `ScopeError::UnknownKind` là biến thể gần nhất (chuỗi trên dây không
            // khớp một giá trị đã khai); nó không bao giờ vượt ranh giới IPC với nội dung
            // riêng (`AiConfigStoreError::Scope` rơi về một khoá chung không tham số).
            let wire = r.tier().as_str();
            let tier = AiConfigTier::from_wire(wire)
                .ok_or_else(|| AiConfigStoreError::Scope(ScopeError::UnknownKind { wire: wire.to_owned() }))?;
            let field = ResolvedField { value: r.value().clone(), tier, shadowed: r.shadowed().cloned() };
            Ok((key, field))
        })
        .collect()
}

/// Ghi (hoặc cập nhật) giá trị của MỘT trường ở tầng `store` — chỗ gọi đã qua
/// [`super::validate_field`] trước khi tới đây (§Always spec 4.2: "từ chối TRƯỚC bất kỳ lượt
/// ghi nào"), nên hàm này không kiểm tra lại hình dạng giá trị.
pub fn write_field(store: &Store, field: AiConfigField, value: &str) -> Result<(), StoreError> {
    let key = field.as_str().to_owned();
    let value = value.to_owned();

    store.write(move |tx: &Transaction<'_>| {
        tx.execute(
            "INSERT INTO ai_config (key, value, updated_at)
             VALUES (?1, ?2, strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
             ON CONFLICT (key) DO UPDATE SET
               value      = excluded.value,
               updated_at = excluded.updated_at",
            (&key, &value),
        )?;
        Ok(())
    })?;

    Ok(())
}

/// Xoá hàng của MỘT trường ở tầng `store` — "trả về kế thừa" (I/O Matrix: "Clear a Work
/// override"). Xoá một khoá không tồn tại là **thành công**, cùng luật
/// `core::scope::store::delete_value`: đường đọc phân biệt "chưa ghi đè" bằng SỰ CÓ MẶT của
/// hàng, không bằng giá trị của nó.
pub fn clear_field(store: &Store, field: AiConfigField) -> Result<(), StoreError> {
    let key = field.as_str().to_owned();

    store.write(move |tx: &Transaction<'_>| {
        tx.execute("DELETE FROM ai_config WHERE key = ?1", [key.as_str()])?;
        Ok(())
    })?;

    Ok(())
}

/// Hai họ lỗi gặp nhau ở [`resolve_two_tiers`], cộng lỗi tầng đích không sẵn sàng — cùng
/// khuôn `core::cleanup::store::CleanupStoreError`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AiConfigStoreError {
    Store(StoreError),
    /// `ScopeResolver::apply_override` từ chối — lỗi LẬP TRÌNH, không nên xảy ra trên đường
    /// gọi đúng. KHÔNG BAO GIỜ vượt ranh giới IPC với `Display` của nó.
    Scope(ScopeError),
    /// Chọn tầng Tác phẩm khi chưa mở Tác phẩm nào — I/O Matrix: "Save a Work override with
    /// no Work open".
    WorkTierUnavailable,
}

impl std::fmt::Display for AiConfigStoreError {
    /// KHÔNG DẤU (NFR16).
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AiConfigStoreError::Store(e) => write!(f, "aiconfig[store] {e}"),
            AiConfigStoreError::Scope(e) => write!(f, "aiconfig[scope] {e}"),
            AiConfigStoreError::WorkTierUnavailable => write!(f, "aiconfig[work_tier_unavailable]"),
        }
    }
}

impl std::error::Error for AiConfigStoreError {}

impl From<StoreError> for AiConfigStoreError {
    fn from(e: StoreError) -> Self {
        AiConfigStoreError::Store(e)
    }
}

impl From<ScopeError> for AiConfigStoreError {
    fn from(e: ScopeError) -> Self {
        AiConfigStoreError::Scope(e)
    }
}

impl From<AiConfigStoreError> for IpcError {
    fn from(err: AiConfigStoreError) -> Self {
        match err {
            AiConfigStoreError::Store(e) => e.into(),
            AiConfigStoreError::Scope(_) => IpcError::new(
                "ai_config.scope_error",
                MessageKey::AiConfigScopeError,
                BTreeMap::new(),
                false,
            ),
            AiConfigStoreError::WorkTierUnavailable => IpcError::new(
                "ai_config.work_tier_unavailable",
                MessageKey::AiConfigWorkTierUnavailable,
                BTreeMap::new(),
                false,
            ),
        }
    }
}

impl From<super::InvalidFieldValue> for IpcError {
    /// Trước khi tới đây, `commands::aiconfig` đã gọi [`super::validate_field`] và thất bại
    /// — chưa giao dịch nào mở (§Always spec 4.2).
    fn from(err: super::InvalidFieldValue) -> Self {
        let mut params = BTreeMap::new();
        params.insert("field".to_owned(), err.field.as_str().to_owned());
        IpcError::new("ai_config.invalid_value", MessageKey::AiConfigInvalidValue, params, false)
    }
}

/// Ba sự thật riêng của bí mật khoá API (Story 4.3) — KHÔNG chung enum với
/// [`AiConfigStoreError`]: `resolve_two_tiers`/`write_field`/`clear_field` không có gì để
/// nói về khoá (nó không đi qua bảng `ai_config`), và `TierNotGlobal` không có tương đương
/// ở năm trường kia (chúng CÓ hai tầng thật; khoá thì không, §Intent spec 4.3).
///
/// Ba biến thể khớp đúng ba hàng lỗi của I/O Matrix spec 4.3: từ chối giá trị TRƯỚC khi
/// chạm keychain, keychain từ chối trả lời, và một yêu cầu ở tầng Tác phẩm bị chặn tại
/// tầng lệnh — không biến thể nào mang giá trị khoá hay một phần của nó (§Always spec 4.3).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AiConfigKeyError {
    /// Rỗng hoặc chỉ khoảng trắng — [`super::validate_key`] đã từ chối trước khi gọi vào
    /// `keychain::set`.
    InvalidValue,
    /// Keychain từ chối trả lời (khoá, quyền bị chặn, không có kho nền tảng) — hành động
    /// thất bại, màn hình cấu hình vẫn dùng được (I/O Matrix "Keychain refuses").
    Unavailable,
    /// Yêu cầu khoá ở tầng Tác phẩm — bị chặn TẠI TẦNG LỆNH (§Always spec 4.3: "refused at
    /// the command layer, not merely hidden in the UI"), không phải một khái niệm mà module
    /// này hay `keychain.rs` tự biết.
    TierNotGlobal,
}

impl std::fmt::Display for AiConfigKeyError {
    /// KHÔNG DẤU (NFR16).
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AiConfigKeyError::InvalidValue => write!(f, "aiconfig[key_invalid_value]"),
            AiConfigKeyError::Unavailable => write!(f, "aiconfig[key_unavailable]"),
            AiConfigKeyError::TierNotGlobal => write!(f, "aiconfig[key_tier_not_global]"),
        }
    }
}

impl std::error::Error for AiConfigKeyError {}

impl From<super::InvalidKeyValue> for AiConfigKeyError {
    fn from(_: super::InvalidKeyValue) -> Self {
        AiConfigKeyError::InvalidValue
    }
}

impl From<super::keychain::KeychainUnavailable> for AiConfigKeyError {
    fn from(_: super::keychain::KeychainUnavailable) -> Self {
        AiConfigKeyError::Unavailable
    }
}

impl From<AiConfigKeyError> for IpcError {
    /// Không nhánh nào chèn `params` — cả ba câu của `vi.json` đều không cần nội suy gì, và
    /// đây chính là điều kiện cấu trúc giữ giá trị khoá không bao giờ lọt vào một tham số
    /// lỗi (§Always spec 4.3).
    fn from(err: AiConfigKeyError) -> Self {
        match err {
            AiConfigKeyError::InvalidValue => IpcError::new(
                "ai_config.key_invalid_value",
                MessageKey::AiConfigKeyInvalidValue,
                BTreeMap::new(),
                false,
            ),
            // `retryable = true`: khác `InvalidValue`/`TierNotGlobal` (cùng đầu vào sẽ luôn
            // thất bại y hệt), một keychain bị khoá/từ chối quyền CÓ THỂ thành công ở lượt
            // bấm lại sau khi người dùng mở khoá — cùng lý lẽ `StoreError::retryable` đã ghi
            // cho `WriteFailed`/`ReadFailed`.
            AiConfigKeyError::Unavailable => IpcError::new(
                "ai_config.keychain_unavailable",
                MessageKey::AiConfigKeychainUnavailable,
                BTreeMap::new(),
                true,
            ),
            AiConfigKeyError::TierNotGlobal => IpcError::new(
                "ai_config.key_is_global",
                MessageKey::AiConfigKeyIsGlobal,
                BTreeMap::new(),
                false,
            ),
        }
    }
}
