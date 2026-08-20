//! Bề mặt IPC "Thêm nhanh thuật ngữ" — Story 3.3, FR48.
//!
//! Cùng khuôn `commands::config`/`commands::chapter`: hàm thuần trước, `#[tauri::command]`
//! chỉ là vỏ mỏng trong `wire`. Ba hàm thuần đều nhận `Option<&Store>` cho tầng Global
//! (đúng khuôn `commands::pinned`) **cộng** `Option<&OpenWork>` cho tầng Tác phẩm (đúng
//! khuôn `commands::chapter`) — Glossary là module ĐẦU TIÊN cần cả hai cùng lúc trên một
//! bề mặt IPC.
//!
//! ─────────────────────────────────────────────────────────────────────────────
//! 🔴 CHỖ ĐẦU TIÊN `OpenWork.scope` ĐƯỢC ĐỌC TRONG MÃ SẢN PHẨM — đóng `deferred-work.md:603`
//! ─────────────────────────────────────────────────────────────────────────────
//! Trước tệp này, trường `OpenWork::scope` chỉ được ĐẶT (ở `commands::project::create_work`)
//! và không command nào khác đọc lại nó — `deferred-work.md:603` gọi đó là một lỗ hở có chủ,
//! chờ epic đầu tiên có dữ liệu tầng Tác phẩm thật. Epic 3 là epic đó, và
//! [`glossary_lookup_term`] là chỗ đầu tiên `&open.scope` chạy qua `ScopeResolver::
//! apply_override` với dữ liệu THẬT ở cả hai tầng.
//!
//! ─────────────────────────────────────────────────────────────────────────────
//! 🔴 CHỈ BA HÀM MỚI ĐƯỢC GỌI XUỐNG `core/glossary/**` — KHÔNG BA TÊN BỊ CẤM
//! ─────────────────────────────────────────────────────────────────────────────
//! `resolve_term_for_quick_add` / `add_manual_term` / `update_manual_term` là bề mặt DUY
//! NHẤT mà tệp này được gọi. `insert_manual_entry` / `confirm_translation` / `load_tier`
//! vẫn bị `glossary_boundary.rs::GLOSSARY_ONLY_SURFACE` cấm ngoài `core/glossary/**` — kể
//! cả ở đây. Đây là đường Ice đã ký ở `glossary_boundary.rs:80-88` khi Story 3.1 gặp đúng
//! vòng luẩn quẩn "hàm phơi ra không đủ, hàm nội bộ thì bị cấm gọi": sửa CHỮ KÝ (thêm ba
//! hàm mới trong `core::glossary::store`) thay vì nới cổng.
//!
//! ⚠️ Mọi chuỗi trong tệp này viết KHÔNG DẤU — `scripts/check-i18n.mjs` Kiểm A quét
//! `src-tauri/**/*.rs`.

use crate::commands::project::OpenWork;
use crate::core::glossary::{
    Category, GlossaryEntry, GlossaryTier, add_manual_term, resolve_term_for_quick_add,
    update_manual_term,
};
use crate::core::i18n::IpcError;
use crate::core::scope::ScopeResolver;
use crate::core::store::{Store, StoreError, StoreKind};

/// Kho `global.db` vắng mặt ⇒ lỗi *mở kho* — cùng khuôn và cùng lý do
/// `commands::pinned::store_is_missing`: đi qua `From<StoreError> for IpcError`, không dựng
/// `IpcError` bằng struct literal.
fn store_is_missing() -> IpcError {
    StoreError::OpenFailed {
        store: StoreKind::Global,
        detail: "the global store was never managed; see lib.rs::open_global_store".to_owned(),
    }
    .into()
}

/// Đọc `(&Store, &ScopeResolver)` của Tác phẩm **đang mở** — `None` khi chưa mở Tác phẩm
/// nào. Đây là hàm mà đoạn 🔴 đầu tệp nhắc tới: nó là chỗ ĐẦU TIÊN `OpenWork::scope` được
/// đọc trong mã sản phẩm.
fn work_context(open: Option<&OpenWork>) -> Option<(&Store, &ScopeResolver)> {
    open.map(|w| (&w.store, &w.scope))
}

/// Hình dạng trên dây của một mục Glossary tìm thấy qua [`glossary_lookup_term`] — mang
/// theo TẦNG (xem doc-comment [`GlossaryTier`]: `id` chỉ duy nhất TRONG một `Store`).
///
/// ⚠️ `#[serde(rename_all = ...)]` KHÔNG đặt — cùng luật với mọi struct qua biên IPC.
#[derive(Debug, Clone, serde::Serialize)]
pub struct QuickAddTerm {
    /// `"global"` hoặc `"work"` — tầng đang GHIM cho chế độ SỬA (§Design Notes).
    pub tier: String,
    /// `glossary_entry.id` — chỉ có nghĩa cùng với `tier` ở trên.
    pub id: i64,
    pub source_term: String,
    /// `None` == *chờ chốt* — lượt tra KHÔNG lọc `is_confirmed` (§Design Notes 🔴).
    pub translation: Option<String>,
    pub note: String,
    pub category: String,
    pub term_origin: String,
    pub created_at: String,
}

impl QuickAddTerm {
    fn from_resolved(tier: GlossaryTier, entry: GlossaryEntry) -> Self {
        Self {
            tier: tier.as_str().to_owned(),
            id: entry.id,
            source_term: entry.source_term,
            translation: entry.translation,
            note: entry.note,
            category: entry.category.as_str().to_owned(),
            term_origin: entry.term_origin.as_str().to_owned(),
            created_at: entry.created_at,
        }
    }
}

/// Phong bì trả lời của [`glossary_lookup_term`] — **không chỉ một `Option<QuickAddTerm>`
/// trần**, và đó là chủ ý.
///
/// ─────────────────────────────────────────────────────────────────────────────
/// 🔴 VÌ SAO `work_tier_available` PHẢI ĐI CÙNG, KHÔNG PHẢI MỘT TRUY VẤN RIÊNG
/// ─────────────────────────────────────────────────────────────────────────────
/// I/O Matrix của spec: *"Chọn tầng Tác phẩm khi chưa mở Tác phẩm ⇒ Lựa chọn hiện KÈM LÝ
/// DO, không biến mất — rỗng có lý do, không rỗng im lặng."* Dải "Thêm thuật ngữ" phải hiện
/// hai lựa chọn tầng LUÔN LUÔN và giải thích thẳng khi tầng Tác phẩm chưa dùng được — TRƯỚC
/// khi người dùng bấm Lưu và ăn một lỗi, không chỉ SAU đó. Muốn hiện lý do đó tường minh,
/// webview phải biết "có Tác phẩm nào đang mở không" **ngay trong đúng lượt gọi đã đọc
/// `OpenWorkState`** — dựng một command thứ tư chỉ để hỏi "có Tác phẩm đang mở không" là
/// một vòng IPC thừa cho dữ liệu mà [`glossary_lookup_term`] đã có sẵn trong tay.
#[derive(Debug, Clone, serde::Serialize)]
pub struct QuickAddLookup {
    /// `true` ⇔ có một Tác phẩm đang mở, tức tầng [`GlossaryTier::Work`] dùng được cho lượt
    /// `glossary_add_term`/`glossary_update_term` tiếp theo.
    pub work_tier_available: bool,
    /// Mục tìm thấy qua hai tầng, hoặc `None` — chế độ THÊM/SỬA của dải suy từ trường này
    /// (§Design Notes: `mode(source_term, lookup)`).
    pub entry: Option<QuickAddTerm>,
}

/// Tra `source_term` qua hai tầng — **hàm thuần, đây là thứ test gọi**. Quyết định chế độ
/// THÊM/SỬA của dải "Thêm thuật ngữ" (§Design Notes: `mode(source_term, lookup)`).
///
/// # Lỗi
/// - `global.db` vắng mặt ⇒ `store.open_failed`;
/// - đường đọc trượt (một trong hai tầng) ⇒ `store.read_failed`/`store.write_failed`;
/// - `ScopeResolver::apply_override` từ chối ⇒ `glossary.scope_error` (lỗi lập trình,
///   không nên xảy ra trên đường gọi đúng).
pub fn glossary_lookup_term(
    global: Option<&Store>,
    open: Option<&OpenWork>,
    source_term: &str,
) -> Result<QuickAddLookup, IpcError> {
    let global = global.ok_or_else(store_is_missing)?;

    // ⚠️ `default_resolver` sống ĐỦ LÂU cho lượt gọi này: `ScopeResolver::global_only()`
    // không mang gì cần `Drop`, nên giữ một bản `const` cục bộ rẻ hơn `Clone` từ
    // `open.scope` mỗi lượt gọi khi `open` là `Some`.
    let default_resolver = ScopeResolver::global_only();
    let context = work_context(open);
    let (resolver, work_store) = match context {
        Some((store, resolver)) => (resolver, Some(store)),
        None => (&default_resolver, None),
    };

    let found = resolve_term_for_quick_add(resolver, global, work_store, source_term)?;
    Ok(QuickAddLookup {
        work_tier_available: context.is_some(),
        entry: found.map(|(tier, entry)| QuickAddTerm::from_resolved(tier, entry)),
    })
}

/// Thêm một mục **nhập tay** ở tầng `tier` — chế độ THÊM. **Hàm thuần.**
///
/// # Lỗi
/// - `global.db` vắng mặt ⇒ `store.open_failed`;
/// - `tier == GlossaryTier::Work` mà chưa mở Tác phẩm nào ⇒ `glossary.work_tier_unavailable`;
/// - `source_term`/`translation` rỗng, hoặc `source_term` đã có ⇒ `store.write_failed`.
pub fn glossary_add_term(
    global: Option<&Store>,
    open: Option<&OpenWork>,
    tier: GlossaryTier,
    source_term: &str,
    translation: Option<&str>,
    note: &str,
    category: Category,
) -> Result<i64, IpcError> {
    let global = global.ok_or_else(store_is_missing)?;
    let work_store = work_context(open).map(|(store, _)| store);

    let id = add_manual_term(global, work_store, tier, source_term, translation, note, category)?;
    Ok(id)
}

/// Sửa `translation`/`note`/`category` của mục `(tier, id)` — chế độ SỬA. **Hàm thuần.**
///
/// # Lỗi
/// - `global.db` vắng mặt ⇒ `store.open_failed`;
/// - `tier == GlossaryTier::Work` mà chưa mở Tác phẩm nào ⇒ `glossary.work_tier_unavailable`;
/// - `(tier, id)` không khớp hàng nào ⇒ `glossary.entry_missing`;
/// - `translation` rỗng, hoặc trigger một chiều từ chối ⇒ `store.write_failed`.
pub fn glossary_update_term(
    global: Option<&Store>,
    open: Option<&OpenWork>,
    tier: GlossaryTier,
    id: i64,
    translation: Option<&str>,
    note: &str,
    category: Category,
) -> Result<(), IpcError> {
    let global = global.ok_or_else(store_is_missing)?;
    let work_store = work_context(open).map(|(store, _)| store);

    update_manual_term(global, work_store, tier, id, translation, note, category)?;
    Ok(())
}

/// Ba vỏ `#[tauri::command]`. **Không một quy tắc nào sống ở đây.**
pub mod wire {
    use super::{Category, GlossaryTier, IpcError, QuickAddLookup};
    use crate::commands::project::OpenWorkState;
    use crate::core::store::Store;

    /// Vỏ IPC của [`super::glossary_lookup_term`].
    ///
    /// ⚠️ `try_state`, không `state()` — cùng lý do mọi vỏ khác của kho: `app.manage(store)`
    /// (`global.db`) và `app.manage(OpenWorkState)` có thể chưa từng chạy, và `panic =
    /// "abort"` giết cả tiến trình nếu ta thẳng tay `state::<T>()`.
    #[tauri::command]
    pub fn glossary_lookup_term(
        app: tauri::AppHandle,
        source_term: String,
    ) -> Result<QuickAddLookup, IpcError> {
        use tauri::Manager as _;

        let global = app.try_state::<Store>();
        let Some(work_state) = app.try_state::<OpenWorkState>() else {
            return super::glossary_lookup_term(global.as_deref(), None, &source_term);
        };
        let guard = work_state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        super::glossary_lookup_term(global.as_deref(), guard.as_ref(), &source_term)
    }

    /// Vỏ IPC của [`super::glossary_add_term`].
    #[tauri::command]
    pub fn glossary_add_term(
        app: tauri::AppHandle,
        tier: GlossaryTier,
        source_term: String,
        translation: Option<String>,
        note: String,
        category: Category,
    ) -> Result<i64, IpcError> {
        use tauri::Manager as _;

        let global = app.try_state::<Store>();
        let Some(work_state) = app.try_state::<OpenWorkState>() else {
            return super::glossary_add_term(
                global.as_deref(),
                None,
                tier,
                &source_term,
                translation.as_deref(),
                &note,
                category,
            );
        };
        let guard = work_state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        super::glossary_add_term(
            global.as_deref(),
            guard.as_ref(),
            tier,
            &source_term,
            translation.as_deref(),
            &note,
            category,
        )
    }

    /// Vỏ IPC của [`super::glossary_update_term`].
    #[tauri::command]
    pub fn glossary_update_term(
        app: tauri::AppHandle,
        tier: GlossaryTier,
        id: i64,
        translation: Option<String>,
        note: String,
        category: Category,
    ) -> Result<(), IpcError> {
        use tauri::Manager as _;

        let global = app.try_state::<Store>();
        let Some(work_state) = app.try_state::<OpenWorkState>() else {
            return super::glossary_update_term(
                global.as_deref(),
                None,
                tier,
                id,
                translation.as_deref(),
                &note,
                category,
            );
        };
        let guard = work_state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        super::glossary_update_term(
            global.as_deref(),
            guard.as_ref(),
            tier,
            id,
            translation.as_deref(),
            &note,
            category,
        )
    }
}
