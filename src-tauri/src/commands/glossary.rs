//! Bề mặt IPC "Thêm nhanh thuật ngữ" (Story 3.3, FR48) + "Đánh dấu thuật ngữ" (Story 3.4,
//! FR50/FR51).
//!
//! Cùng khuôn `commands::config`/`commands::chapter`: hàm thuần trước, `#[tauri::command]`
//! chỉ là vỏ mỏng trong `wire`. Bốn hàm thuần đều nhận `Option<&Store>` cho tầng Global
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
//! 🔴 CHỈ CÁC HÀM CỦA `QUICK_ADD_SURFACE` ĐƯỢC GỌI XUỐNG `core/glossary/**` — KHÔNG BA TÊN BỊ CẤM
//! ─────────────────────────────────────────────────────────────────────────────
//! `resolve_term_for_quick_add` / `add_manual_term` / `update_manual_term` (Story 3.3),
//! `marks_for_source_text` (Story 3.4), `pending_candidates` (Story 3.5), `confirm_pending_
//! translation` / `approve_candidate` (Story 3.6), cộng `suggest_han_viet_batch` (Story 3.7)
//! là bề mặt DUY NHẤT mà tệp này được gọi xuống `core::glossary`.
//! `insert_manual_entry` / `confirm_translation` / `load_tier` / `insert_candidate` vẫn bị
//! `glossary_boundary.rs::GLOSSARY_ONLY_SURFACE` cấm ngoài `core/glossary/**` — kể cả ở đây.
//! Đây là đường Ice đã ký ở `glossary_boundary.rs:80-88` khi Story 3.1 gặp đúng vòng luẩn
//! quẩn "hàm phơi ra không đủ, hàm nội bộ thì bị cấm gọi": sửa CHỮ KÝ (thêm hàm mới trong
//! `core::glossary::store`) thay vì nới cổng — tiền lệ Story 3.3 dùng lại nguyên vẹn.
//! `approve_candidate` (`core::glossary::candidate_store`) KHÔNG nằm trong
//! `GLOSSARY_ONLY_SURFACE`/không cần một hàm bọc thứ hai — Story 3.5 đã cố ý để nó ngoài
//! danh sách cấm (doc-comment `glossary_boundary.rs::GLOSSARY_ONLY_SURFACE`), chờ đúng
//! "story dựng chỗ gọi sản phẩm đầu tiên" — Story 3.6 là story đó.
//!
//! ⚠️ Mọi chuỗi trong tệp này viết KHÔNG DẤU — `scripts/check-i18n.mjs` Kiểm A quét
//! `src-tauri/**/*.rs`.

use std::collections::{BTreeMap, BTreeSet};

use crate::commands::project::OpenWork;
use crate::core::dict::DictLayers;
use crate::core::glossary::{
    Category, GlossaryEntry, GlossaryMark, GlossaryTier, HanVietSuggestion, add_manual_term,
    approve_candidate, confirm_pending_translation, match_lang_for_source_lang,
    marks_for_source_text, pending_candidates, resolve_term_for_quick_add, suggest_han_viet_batch,
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

// ═════════════════════════════════════════════════════════════════════════════════
// Story 3.4 — dau khop thuat ngu cho mot doan van ban (FR50/FR51)
// ═════════════════════════════════════════════════════════════════════════════════

/// Hình dạng trên dây của một [`GlossaryMark`] — Story 3.4.
///
/// ⚠️ `#[serde(rename_all = ...)]` KHÔNG đặt — cùng luật với mọi struct qua biên IPC.
#[derive(Debug, Clone, serde::Serialize)]
pub struct GlossaryMarkWire {
    /// Điểm mã bắt đầu (bao gồm) — KHÔNG phải byte, KHÔNG phải UTF-16 (§Design Notes của
    /// story: quy đổi làm TRONG Rust, một lần, một chỗ).
    pub start: usize,
    /// Điểm mã kết thúc (không bao gồm).
    pub end: usize,
    /// `"global"` hoặc `"work"`.
    pub tier: String,
    /// `false` == mục *chờ chốt* — dấu vẫn ra, nửa giao diện phải vẽ khác.
    pub is_confirmed: bool,
    /// `None` khi mục đang *chờ chốt*.
    pub translation: Option<String>,
    /// 🔵 THÊM 2026-08-22 (Story 3.6) — `glossary_entry.id`, cùng `tier` đủ để chốt mục này
    /// qua `glossary_confirm_pending_translation` mà không cần tra lại.
    pub id: i64,
    /// 🔵 THÊM 2026-08-22 (Story 3.6) — khoá ghi thật, có thể KHÁC bề mặt đã khớp trên màn
    /// hình (nhánh tiếng Anh khớp theo hình thái).
    pub source_term: String,
    /// 🔵 THÊM 2026-08-24 (Story 3.7, FR113) — đề xuất âm Hán Việt, hoặc `None` (bốn trong
    /// năm nhánh của `HanVietSuggestion` — xem `han_viet_status`).
    pub han_viet_suggestion: Option<String>,
    /// 🔵 THÊM 2026-08-24 (Story 3.7) — một trong năm chuỗi đóng (`"ok"` · `"not_chinese"` ·
    /// `"no_reading"` · `"dict_unavailable"` · `"not_requested"`). `"not_requested"` cho MỌI
    /// mục ĐÃ CHỐT (`is_confirmed == true`) — dấu đó chưa từng đi qua một lượt tra Hán Việt.
    pub han_viet_status: String,
}

impl From<GlossaryMark> for GlossaryMarkWire {
    fn from(mark: GlossaryMark) -> Self {
        Self {
            start: mark.start,
            end: mark.end,
            tier: mark.tier.as_str().to_owned(),
            is_confirmed: mark.is_confirmed,
            translation: mark.translation,
            id: mark.id,
            source_term: mark.source_term,
            han_viet_suggestion: mark.han_viet_suggestion,
            han_viet_status: mark.han_viet_status.to_owned(),
        }
    }
}

/// Tìm mọi dấu khớp thuật ngữ trong `text` — **hàm thuần, đây là thứ test gọi**.
///
/// ─────────────────────────────────────────────────────────────────────────────
/// 🔴 VÌ SAO NHẬN `text`/`source_lang` LÀM THAM SỐ, KHÔNG TỰ ĐỌC `chapter` TỪ ĐĨA
/// ─────────────────────────────────────────────────────────────────────────────
/// Cùng khuôn [`glossary_lookup_term`] (nhận `source_term` làm tham số, không tự tra gì từ
/// vùng chọn): frontend đã có `source_text`/`source_lang` từ `read_open_chapter` trước khi
/// gọi lệnh này (nửa giao diện — tách khỏi story này). Đọc lại `chapter` từ đĩa ở đây là một
/// lượt SQL thứ hai cho dữ liệu chỗ gọi đã có sẵn trong tay, và nó buộc hàm này phải đòi một
/// Tác phẩm đang mở — trái với I/O Matrix *"Chưa mở Tác phẩm ⇒ Chỉ khớp tầng Global, không
/// lỗi"*: không có Chương nào để đọc khi chưa có Tác phẩm, nhưng `text`/`source_lang` do
/// CHỖ GỌI đưa vào không phụ thuộc điều đó.
///
/// `source_lang` suy ra `MatchLang` qua [`match_lang_for_source_lang`] — không đoán từ nội
/// dung, và đúng MỘT chỗ viết phép chọn đó (xem doc-comment của chính hàm kia).
///
/// # Lỗi
/// - `global.db` vắng mặt ⇒ `store.open_failed`;
/// - đường đọc trượt (một trong hai tầng) ⇒ `store.read_failed`/`store.write_failed`;
/// - `ScopeResolver::apply_override` từ chối ⇒ `glossary.scope_error` (lỗi lập trình,
///   không nên xảy ra trên đường gọi đúng).
///
/// ⚠️ **Không thêm khoá lỗi mới cho hàm này.** Bốn khoá đã có (`store.open_failed` ·
/// `store.read_failed` · `store.write_failed` · `glossary.scope_error`, qua
/// `impl From<GlossaryError> for IpcError`) diễn đạt trọn mọi nhánh lỗi mà
/// `marks_for_source_text` có thể trả — hàm đó chỉ gọi `load_tier` và
/// `ScopeResolver::apply_override`, không nhánh nào khác. Thêm một khoá không có nhánh nào
/// đi qua là đúng thứ Story 1.7 §Completion Notes #3 cấm ("không khoá nào cho một tính năng
/// chưa tồn tại").
///
/// 🔵 THÊM 2026-08-24 (Story 3.7) — `layers`/`disabled` đi thẳng xuống `marks_for_source_text`
/// (đề xuất âm Hán Việt, FR113); `disabled` là bộ lọc nguồn đã tắt của tab Hán Việt, tái dùng
/// qua `commands::dict::disabled_sources`.
pub fn glossary_marks_for_chapter(
    global: Option<&Store>,
    open: Option<&OpenWork>,
    text: &str,
    source_lang: &str,
    layers: &DictLayers,
    disabled: &BTreeSet<String>,
) -> Result<Vec<GlossaryMarkWire>, IpcError> {
    let global = global.ok_or_else(store_is_missing)?;

    let default_resolver = ScopeResolver::global_only();
    let context = work_context(open);
    let (resolver, work_store) = match context {
        Some((store, resolver)) => (resolver, Some(store)),
        None => (&default_resolver, None),
    };

    let lang = match_lang_for_source_lang(source_lang);

    let marks =
        marks_for_source_text(resolver, global, work_store, text, lang, layers, disabled)?;
    Ok(marks.into_iter().map(GlossaryMarkWire::from).collect())
}

// ═════════════════════════════════════════════════════════════════════════════════
// Story 3.5 — vỏ IPC CHỈ-ĐỌC cho bảng chờ, chỗ gọi sản phẩm ĐẦU TIÊN của `pending_candidates`
// ═════════════════════════════════════════════════════════════════════════════════

/// Hình dạng trên dây của một [`crate::core::glossary::GlossaryCandidate`] — Story 3.5.
///
/// ⚠️ `#[serde(rename_all = ...)]` KHÔNG đặt — cùng luật với mọi struct qua biên IPC
/// (`:64`/`:201` của tệp này).
#[derive(Debug, Clone, serde::Serialize)]
pub struct GlossaryCandidateWire {
    pub id: i64,
    pub source_term: String,
    pub candidate_origin: String,
    /// `None` == chờ duyệt — đây là VỊ TỪ DUY NHẤT của
    /// [`crate::core::glossary::GlossaryCandidate::is_pending`],
    /// phơi ra dưới dạng dữ liệu chứ không một cờ `is_pending` song song (cùng khuôn
    /// `resolution` của `glossary_entry.translation`).
    pub resolution: Option<String>,
    pub created_at: String,
    pub occurrence_count: i64,
    pub context_example: Option<String>,
    /// 🔵 THÊM 2026-08-24 (Story 3.7, FR113) — đề xuất âm Hán Việt cho `source_term` của
    /// ứng viên; hình dạng KHỚP `GlossaryMarkWire` (cùng cặp trường, cùng năm chuỗi trạng
    /// thái). Một ứng viên là **chờ duyệt**, không bao giờ đã chốt, nên `"not_requested"`
    /// không bao giờ xuất hiện ở đây (khác `GlossaryMarkWire`, nơi mục đã chốt gán nhãn đó).
    pub han_viet_suggestion: Option<String>,
    pub han_viet_status: String,
}

/// Mọi ứng viên **chờ duyệt** của Tác phẩm đang mở — **hàm thuần, đây là thứ test gọi**.
///
/// ─────────────────────────────────────────────────────────────────────────────
/// 🔴 CHỖ GỌI SẢN PHẨM ĐẦU TIÊN CỦA `core::glossary::pending_candidates` — Story 3.2 dựng,
/// 0 chỗ gọi cho tới lượt này
/// ─────────────────────────────────────────────────────────────────────────────
/// §Intent của story: *"Bảng chờ được phơi qua một vỏ IPC CHỈ-ĐỌC để lượt quét nghiệm thu
/// được bằng mắt."* Không tham số nào khác ngoài `Store` của Tác phẩm — bảng chờ chỉ tồn
/// tại ở `project.db` (§Never/Code Map của story), nên hàm này trả `Ok(vec![])` khi chưa
/// mở Tác phẩm nào, KHÔNG một lỗi: *"chưa có Tác phẩm nào để có bảng chờ"* là một trạng
/// thái bình thường của ứng dụng lúc khởi động, không phải một sự cố.
///
/// # Lỗi
/// đường đọc trượt (kho đóng giữa chừng, …) ⇒ lỗi kho (`store.*`), qua `From<StoreError>`.
///
/// 🔵 THÊM 2026-08-24 (Story 3.7, FR113) — `layers`/`disabled` cho đề xuất âm Hán Việt. Bảng
/// chờ ứng viên KHÔNG đi qua `marks_for_source_text` (nó không phải một dấu khớp trên văn
/// bản, nó là một hàng chờ duyệt), nên đây là chỗ gọi `suggest_han_viet_batch` TRỰC TIẾP —
/// **một lượt cho cả tập** `source_term` đang chờ duyệt, không một lượt cho mỗi hàng.
pub fn glossary_pending_candidates(
    open: Option<&OpenWork>,
    layers: &DictLayers,
    disabled: &BTreeSet<String>,
) -> Result<Vec<GlossaryCandidateWire>, IpcError> {
    let Some(open) = open else {
        return Ok(Vec::new());
    };

    let rows = pending_candidates(&open.store)?;

    let terms: Vec<&str> = rows.iter().map(|c| c.source_term.as_str()).collect();
    let suggestions = suggest_han_viet_batch(layers, disabled, &terms);
    debug_assert_eq!(
        rows.len(),
        suggestions.len(),
        "suggest_han_viet_batch phai tra dung mot phan tu cho moi thuat ngu dau vao"
    );

    // 🔵 SỬA 2026-08-24 (vòng rà Bước 4) — ghép theo KHOÁ, không theo VỊ TRÍ.
    //
    // Bản đầu dùng `rows.into_iter().zip(suggestions)`, tức đúng cặp CHỈ KHI hai vế cùng độ
    // dài và cùng thứ tự. `debug_assert_eq!` ngay trên KHÔNG đỡ được điều đó ở bản phát hành
    // -- `debug_assert` biên dịch thành hư vô ở release, nên một lượt lệch độ dài sẽ để `zip`
    // CẮT CỤT phần đuôi trong im lặng, và một lượt sắp lại `rows` giữa hai dòng sẽ dán đề
    // xuất của thuật ngữ này lên thuật ngữ khác. Cả hai là đúng lớp *rỗng/sai IM LẶNG* mà
    // `AGENTS.md:46` gọi là lỗi trung tâm của kho: không lỗi nào được ném, và màn hình đề
    // xuất "Bắc Lương" cho một thuật ngữ khác hẳn.
    //
    // Khoá bằng `source_term` an toàn vì `glossary_candidate` mang `UNIQUE (source_term)`
    // (`schema.rs:432`) -- không hai hàng chờ duyệt nào chung khoá. Đây cũng đúng khuôn mà
    // `core::glossary::store::marks_for_source_text` đã dùng, vì cùng một lý do.
    let suggestion_by_term: BTreeMap<&str, HanVietSuggestion> =
        terms.iter().copied().zip(suggestions).collect();

    Ok(rows
        .iter()
        .map(|c| {
            let suggestion = suggestion_by_term
                .get(c.source_term.as_str())
                .unwrap_or(&HanVietSuggestion::NotRequested);
            (c, suggestion)
        })
        .map(|(c, suggestion)| GlossaryCandidateWire {
            id: c.id,
            source_term: c.source_term.clone(),
            candidate_origin: c.candidate_origin.as_str().to_owned(),
            resolution: c.resolution.map(|r| r.as_str().to_owned()),
            created_at: c.created_at.clone(),
            occurrence_count: c.occurrence_count,
            context_example: c.context_example.clone(),
            han_viet_suggestion: suggestion.suggestion_text().map(str::to_owned),
            han_viet_status: suggestion.as_status_str().to_owned(),
        })
        .collect())
}

// ═════════════════════════════════════════════════════════════════════════════════
// Story 3.6 — CHỐT trạng thái chờ chốt (FR114) + NHẬN một ứng viên (vỏ IPC ghi)
// ═════════════════════════════════════════════════════════════════════════════════

/// Chốt bản dịch cho mục `(tier, id)` — **hàm thuần, đây là thứ test gọi**. Chỗ gọi sản
/// phẩm ĐẦU TIÊN của [`confirm_pending_translation`] (và, gián tiếp, của
/// `core::glossary::store::confirm_translation` — bị `GLOSSARY_ONLY_SURFACE` cấm gọi thẳng
/// từ đây).
///
/// Dùng được cho CẢ HAI chiều hợp lệ của `confirm_translation` (chốt lần đầu, hoặc sửa một
/// mục ĐÃ chốt sang bản dịch khác) — dải "Chờ chốt" của Story 3.6 chỉ dựng đường gọi cho
/// chiều đầu, nhưng hàm thuần này không hẹp hơn hàm nó bọc.
///
/// # Lỗi
/// - `global.db` vắng mặt ⇒ `store.open_failed`;
/// - `tier == GlossaryTier::Work` mà chưa mở Tác phẩm nào ⇒ `glossary.work_tier_unavailable`;
/// - `translation` rỗng/khoảng trắng, hoặc `id` không khớp hàng nào ⇒ `store.write_failed`.
pub fn glossary_confirm_pending_translation(
    global: Option<&Store>,
    open: Option<&OpenWork>,
    tier: GlossaryTier,
    id: i64,
    translation: &str,
) -> Result<(), IpcError> {
    let global = global.ok_or_else(store_is_missing)?;
    let work_store = work_context(open).map(|(store, _)| store);

    confirm_pending_translation(global, work_store, tier, id, translation)?;
    Ok(())
}

/// Nhận một ứng viên (id) thành một mục Glossary — **hàm thuần, đây là thứ test gọi**. Vỏ
/// IPC GHI đầu tiên của [`approve_candidate`] (Story 3.2 dựng, 0 chỗ gọi sản phẩm cho tới
/// lượt này).
///
/// `translation = None` ⇒ mục sinh ra ở trạng thái *chờ chốt* (FR114) — nhận một ứng viên
/// không bắt buộc phải chốt bản dịch ngay (§I/O Matrix: *"Nhận một ứng viên không có đề
/// xuất"*).
///
/// ⚠️ **Bảng chờ chỉ tồn tại ở `project.db`** (§Never/Code Map của story) — không như
/// [`glossary_pending_candidates`] (một lượt ĐỌC, `None` ⇒ `Ok(vec![])` hợp lý), một lượt
/// GHI không có Tác phẩm nào đang mở không có hàng nào để nhận — đây LÀ một sự cố (một `id`
/// người dùng vừa thấy trên màn hình mà không còn kho nào chứa nó, ca đua "đóng Tác phẩm
/// giữa lúc bấm Nhận"). Tái dùng `commands::chapter::no_work_open` — cùng câu mà
/// `commands::segment` đã dùng cho đúng tình huống *"chưa mở Tác phẩm nào"*.
///
/// # Lỗi
/// - chưa mở Tác phẩm nào ⇒ `project.no_work_open`;
/// - `id` không khớp hàng nào, hoặc ứng viên `id` ĐÃ quyết (đã duyệt hoặc đã bỏ) ⇒
///   `store.write_failed`, mang `message_key`, KHÔNG `Ok` rỗng;
/// - `translation` là chuỗi rỗng/khoảng trắng ⇒ `store.write_failed` (`CHECK`).
pub fn glossary_approve_candidate(
    open: Option<&OpenWork>,
    id: i64,
    translation: Option<&str>,
    category: Category,
) -> Result<i64, IpcError> {
    let open = open.ok_or_else(crate::commands::chapter::no_work_open)?;
    let new_id = approve_candidate(&open.store, id, translation, category)?;
    Ok(new_id)
}

/// Bảy vỏ `#[tauri::command]`. **Không một quy tắc nào sống ở đây.**
pub mod wire {
    use super::{
        Category, GlossaryCandidateWire, GlossaryMarkWire, GlossaryTier, IpcError, QuickAddLookup,
    };
    use crate::commands::project::OpenWorkState;
    use crate::core::dict::DictLayers;
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

    /// Vỏ IPC của [`super::glossary_marks_for_chapter`]. Story 3.4.
    ///
    /// 🔵 THÊM 2026-08-24 (Story 3.7) — `DictLayers` qua `try_state` (khuôn
    /// `commands::dict::wire::read_han_viet`: state có thể chưa từng được `app.manage`, và
    /// `panic = "abort"` giết cả tiến trình nếu ta thẳng tay `state::<T>()`); `disabled` đọc
    /// qua `commands::dict::disabled_sources`, tái dùng đúng phép đọc mà tab Hán Việt dùng.
    #[tauri::command]
    pub fn glossary_marks_for_chapter(
        app: tauri::AppHandle,
        text: String,
        source_lang: String,
    ) -> Result<Vec<GlossaryMarkWire>, IpcError> {
        use tauri::Manager as _;

        let global = app.try_state::<Store>();
        let layers = app.try_state::<DictLayers>();
        let empty_layers = DictLayers::empty();
        let layers = layers.as_deref().unwrap_or(&empty_layers);
        let disabled = crate::commands::dict::disabled_sources(global.as_deref());

        let Some(work_state) = app.try_state::<OpenWorkState>() else {
            return super::glossary_marks_for_chapter(
                global.as_deref(),
                None,
                &text,
                &source_lang,
                layers,
                &disabled,
            );
        };
        let guard = work_state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        super::glossary_marks_for_chapter(
            global.as_deref(),
            guard.as_ref(),
            &text,
            &source_lang,
            layers,
            &disabled,
        )
    }

    /// Vỏ IPC của [`super::glossary_pending_candidates`]. Story 3.5.
    ///
    /// 🔵 THÊM 2026-08-24 (Story 3.7) — cùng khuôn `glossary_marks_for_chapter` ngay trên:
    /// `DictLayers` qua `try_state`, `disabled` qua `commands::dict::disabled_sources`.
    #[tauri::command]
    pub fn glossary_pending_candidates(
        app: tauri::AppHandle,
    ) -> Result<Vec<GlossaryCandidateWire>, IpcError> {
        use tauri::Manager as _;

        let global = app.try_state::<Store>();
        let layers = app.try_state::<DictLayers>();
        let empty_layers = DictLayers::empty();
        let layers = layers.as_deref().unwrap_or(&empty_layers);
        let disabled = crate::commands::dict::disabled_sources(global.as_deref());

        let Some(work_state) = app.try_state::<OpenWorkState>() else {
            return super::glossary_pending_candidates(None, layers, &disabled);
        };
        let guard = work_state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        super::glossary_pending_candidates(guard.as_ref(), layers, &disabled)
    }

    /// Vỏ IPC của [`super::glossary_confirm_pending_translation`]. Story 3.6.
    #[tauri::command]
    pub fn glossary_confirm_pending_translation(
        app: tauri::AppHandle,
        tier: GlossaryTier,
        id: i64,
        translation: String,
    ) -> Result<(), IpcError> {
        use tauri::Manager as _;

        let global = app.try_state::<Store>();
        let Some(work_state) = app.try_state::<OpenWorkState>() else {
            return super::glossary_confirm_pending_translation(
                global.as_deref(),
                None,
                tier,
                id,
                &translation,
            );
        };
        let guard = work_state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        super::glossary_confirm_pending_translation(
            global.as_deref(),
            guard.as_ref(),
            tier,
            id,
            &translation,
        )
    }

    /// Vỏ IPC của [`super::glossary_approve_candidate`]. Story 3.6.
    #[tauri::command]
    pub fn glossary_approve_candidate(
        app: tauri::AppHandle,
        id: i64,
        translation: Option<String>,
        category: Category,
    ) -> Result<i64, IpcError> {
        use tauri::Manager as _;

        let Some(work_state) = app.try_state::<OpenWorkState>() else {
            return super::glossary_approve_candidate(None, id, translation.as_deref(), category);
        };
        let guard = work_state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        super::glossary_approve_candidate(guard.as_ref(), id, translation.as_deref(), category)
    }
}
