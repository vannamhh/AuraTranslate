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
//! translation` / `approve_candidate` (Story 3.6), `suggest_han_viet_batch` (Story 3.7),
//! `reject_candidate` (Story 3.8), cộng `list_all_entries` / `delete_manual_term` /
//! `promote_to_global` (Story 3.9) là bề mặt DUY NHẤT mà tệp này được gọi xuống
//! `core::glossary`.
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
use std::path::Path;

use crate::commands::project::OpenWork;
use crate::core::dict::DictLayers;
use crate::core::glossary::{
    Category, ConflictDecision, Delimiter, GlossaryEntry, GlossaryError, GlossaryMark,
    GlossaryTier, HanVietSuggestion, ImportSummary, RowPlan, RowPlanKind, add_manual_term,
    approve_candidate, classify_import_rows, confirm_pending_translation, delete_manual_term,
    export_tier, import_into_tier, list_all_entries, match_lang_for_source_lang,
    marks_for_source_text, parse as parse_glossary_import, pending_candidates, promote_to_global,
    read_import_file, reject_candidate, resolve_term_for_quick_add, suggest_han_viet_batch,
    update_manual_term, write_export_file,
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

// ═════════════════════════════════════════════════════════════════════════════════
// Story 3.8 — BỎ một ứng viên (vỏ IPC ghi) — vỏ IPC đầu tiên của `reject_candidate`
// ═════════════════════════════════════════════════════════════════════════════════

/// Bỏ một ứng viên (`id`) — **hàm thuần, đây là thứ test gọi**. Vỏ IPC GHI đầu tiên của
/// [`reject_candidate`] (Story 3.2 dựng, 0 chỗ gọi sản phẩm cho tới lượt này) — chép nguyên
/// khuôn [`glossary_approve_candidate`] ngay trên: cùng `no_work_open` khi chưa mở Tác
/// phẩm, cùng lý do (bảng chờ chỉ tồn tại ở `project.db`).
///
/// # Lỗi
/// - chưa mở Tác phẩm nào ⇒ `project.no_work_open`;
/// - `id` không khớp hàng nào, hoặc ứng viên `id` ĐÃ quyết (đã duyệt hoặc đã bỏ) ⇒
///   `store.write_failed`, mang `message_key`, KHÔNG `Ok` rỗng.
pub fn glossary_reject_candidate(open: Option<&OpenWork>, id: i64) -> Result<(), IpcError> {
    let open = open.ok_or_else(crate::commands::chapter::no_work_open)?;
    reject_candidate(&open.store, id)?;
    Ok(())
}

// ═════════════════════════════════════════════════════════════════════════════════
// Story 3.9 — Quản lý Glossary: liệt kê cả hai tầng · xoá · đẩy tầng (Work → Global)
// ═════════════════════════════════════════════════════════════════════════════════

/// Hình dạng trên dây của một mục Glossary trong màn hình quản lý — Story 3.9.
///
/// ⚠️ `#[serde(rename_all = ...)]` KHÔNG đặt — cùng luật với mọi struct qua biên IPC.
///
/// Khác [`QuickAddTerm`]: mang thêm `is_shadowed` (Rust tính, KHÔNG chép quy tắc "Tác phẩm
/// thắng" sang TypeScript — §Always của spec), và KHÔNG lọc `is_confirmed` (một mục chờ
/// chốt vẫn phải hiện ra để SỬA/XOÁ).
#[derive(Debug, Clone, serde::Serialize)]
pub struct GlossaryEntryWire {
    /// `"global"` hoặc `"work"`.
    pub tier: String,
    /// `glossary_entry.id` — chỉ có nghĩa CÙNG VỚI `tier` ở trên.
    pub id: i64,
    pub source_term: String,
    /// `None` == *chờ chốt*.
    pub translation: Option<String>,
    pub note: String,
    pub category: String,
    pub term_origin: String,
    pub created_at: String,
    /// `true` ⇔ một mục Work cùng `source_term` đang thắng — hàng này KHÔNG được ép vào
    /// prompt (AD-36) dù vẫn hiện trên màn hình quản lý.
    pub is_shadowed: bool,
}

impl GlossaryEntryWire {
    fn from_resolved(tier: GlossaryTier, entry: GlossaryEntry, is_shadowed: bool) -> Self {
        Self {
            tier: tier.as_str().to_owned(),
            id: entry.id,
            source_term: entry.source_term,
            translation: entry.translation,
            note: entry.note,
            category: entry.category.as_str().to_owned(),
            term_origin: entry.term_origin.as_str().to_owned(),
            created_at: entry.created_at,
            is_shadowed,
        }
    }
}

/// Mọi mục Glossary của **cả hai tầng** — **hàm thuần, đây là thứ test gọi**.
///
/// # Lỗi
/// - `global.db` vắng mặt ⇒ `store.open_failed`;
/// - đường đọc trượt (một trong hai tầng) ⇒ `store.read_failed`/`store.write_failed`;
/// - `ScopeResolver::apply_override` từ chối ⇒ `glossary.scope_error` (lỗi lập trình,
///   không nên xảy ra trên đường gọi đúng).
pub fn glossary_list_entries(
    global: Option<&Store>,
    open: Option<&OpenWork>,
) -> Result<Vec<GlossaryEntryWire>, IpcError> {
    let global = global.ok_or_else(store_is_missing)?;

    let default_resolver = ScopeResolver::global_only();
    let context = work_context(open);
    let (resolver, work_store) = match context {
        Some((store, resolver)) => (resolver, Some(store)),
        None => (&default_resolver, None),
    };

    let rows = list_all_entries(resolver, global, work_store)?;
    Ok(rows
        .into_iter()
        .map(|(tier, entry, shadowed)| GlossaryEntryWire::from_resolved(tier, entry, shadowed))
        .collect())
}

/// Xoá mục `(tier, id)` — **hàm thuần, đây là thứ test gọi**.
///
/// # Lỗi
/// - `global.db` vắng mặt ⇒ `store.open_failed`;
/// - `tier == GlossaryTier::Work` mà chưa mở Tác phẩm nào ⇒ `glossary.work_tier_unavailable`;
/// - `(tier, id)` không khớp hàng nào ⇒ `glossary.entry_missing`.
pub fn glossary_delete_term(
    global: Option<&Store>,
    open: Option<&OpenWork>,
    tier: GlossaryTier,
    id: i64,
) -> Result<(), IpcError> {
    let global = global.ok_or_else(store_is_missing)?;
    let work_store = work_context(open).map(|(store, _)| store);

    delete_manual_term(global, work_store, tier, id)?;
    Ok(())
}

/// Đẩy mục `id` ở tầng **Tác phẩm** lên tầng **Toàn cục** — **hàm thuần, đây là thứ test
/// gọi**.
///
/// ⚠️ **Không tham số `tier`** — khác [`glossary_delete_term`]. Lệnh này chỉ có nghĩa cho
/// một hàng tầng Work (§I/O Matrix của spec: *"Đẩy một mục Global ⇒ Lệnh không áp dụng"*) —
/// nửa giao diện không được phép gọi lệnh này cho một hàng `tier === 'global'`, và cấm đó
/// đứng ở TẦNG UI (không hiện nút/không dispatch), không ở đây: `promote_to_global` (Rust)
/// luôn đọc `id` từ `open.store` (`project.db`), nên gọi nó với một `id` chỉ tồn tại ở
/// `global.db` tự nhiên rơi vào `glossary.entry_missing`.
///
/// # Lỗi
/// - chưa mở Tác phẩm nào ⇒ `project.no_work_open` (bảng `glossary_entry` tầng Work chỉ
///   tồn tại trong MỘT `project.db` — không có gì để đẩy nếu không có nó, cùng khuôn
///   `glossary_approve_candidate`/`glossary_reject_candidate`);
/// - `id` không khớp hàng nào ở tầng Work ⇒ `glossary.entry_missing`;
/// - `source_term` đã có ở tầng Toàn cục ⇒ `glossary.global_term_exists`, **0 lượt ghi**.
pub fn glossary_promote_term_to_global(
    global: Option<&Store>,
    open: Option<&OpenWork>,
    id: i64,
) -> Result<(), IpcError> {
    let global = global.ok_or_else(store_is_missing)?;
    let open = open.ok_or_else(crate::commands::chapter::no_work_open)?;

    promote_to_global(global, &open.store, id)?;
    Ok(())
}

// ═════════════════════════════════════════════════════════════════════════════════
// Story 3.10b — hộp thoại chọn tệp nối vào xuất/nhập CSV/TSV (AD-48)
// ═════════════════════════════════════════════════════════════════════════════════
//
// Bốn vỏ IPC mới trong `wire` (xuất · mở-và-xem-trước nhập · xác nhận nhập · huỷ lô
// treo), gọi thẳng `export_tier`/`import_into_tier` — hai tên đó KHÔNG nằm trong
// `GLOSSARY_ONLY_SURFACE` (§Code Map của spec), nên không cần một hàm bọc thứ hai như
// `classify_import_rows` phải có cho `load_tier`.

/// Lô nhập đang TREO giữa nhịp một (mở + xem trước) và nhịp hai (xác nhận) — AD-48
/// §Rule ①: nội dung tệp KHÔNG BAO GIỜ đi ra webview, kế hoạch đã phân tích Ở LẠI RUST.
/// Dọn khi: huỷ ([`glossary_cancel_import`]), một lô MỚI thay nó ([`glossary_open_import_preview`]
/// ghi đè `*guard`), xác nhận THÀNH CÔNG ([`glossary_confirm_import`]), hoặc Tác phẩm đóng
/// khi lô đang treo thuộc tầng Work (`lib.rs`).
#[derive(Debug)]
pub struct PendingImport {
    /// Đường dẫn tệp đã đọc — chỉ để chẩn đoán, KHÔNG đọc lại ở nhịp hai.
    pub path: std::path::PathBuf,
    /// Tầng đã chọn lúc mở — nhịp hai ghi vào ĐÚNG tầng này, không nhận lại qua tham số.
    pub tier: GlossaryTier,
    /// Kế hoạch đã phân loại — mô hình mà [`glossary_confirm_import`] ghi theo.
    pub plans: Vec<RowPlan>,
}

/// Kiểu state Tauri quản lý cho [`PendingImport`] — `None` == không lô nào đang treo, cùng
/// khuôn `commands::project::OpenWorkState`.
pub type PendingImportState = std::sync::Mutex<Option<PendingImport>>;

/// Chọn `&Store` theo `tier` cho đường XUẤT — `export_tier` (khác `import_into_tier`)
/// KHÔNG nhận `tier` (§Code Map của spec: hai chữ ký LỆCH nhau, đừng giả định giống), nên
/// bước phân giải này sống Ở ĐÂY.
fn resolve_tier_store<'a>(
    global: &'a Store,
    work: Option<&'a Store>,
    tier: GlossaryTier,
) -> Result<&'a Store, GlossaryError> {
    match tier {
        GlossaryTier::Global => Ok(global),
        GlossaryTier::Work => work.ok_or(GlossaryError::WorkTierUnavailable),
    }
}

/// Đuôi tệp THẬT của đường dẫn NGƯỜI DÙNG VỪA CHỌN quyết định dấu phân cách (§I/O Matrix:
/// "Đuôi lạ ⇒ CSV") — không theo trạng thái UI trước lượt chọn.
fn delimiter_from_path(path: &Path) -> Delimiter {
    match path.extension().and_then(|e| e.to_str()) {
        Some(ext) if ext.eq_ignore_ascii_case("tsv") => Delimiter::Tsv,
        _ => Delimiter::Csv,
    }
}

/// Xuất tầng `tier` ra `path` — **hàm thuần, đây là thứ test gọi**. Một NHỊP.
///
/// # Lỗi
/// - `global.db` vắng mặt ⇒ `store.open_failed`;
/// - `tier == Work` mà chưa mở Tác phẩm nào ⇒ `glossary.work_tier_unavailable`;
/// - ghi tệp thất bại ⇒ `glossary.export_write_failed`, **0** tệp cụt để lại
///   (`write_export_file` dọn `.tmp` ở cả hai nhánh lỗi).
pub fn glossary_export_tier(
    global: Option<&Store>,
    open: Option<&OpenWork>,
    tier: GlossaryTier,
    path: &Path,
) -> Result<(), IpcError> {
    let global = global.ok_or_else(store_is_missing)?;
    let work_store = work_context(open).map(|(store, _)| store);
    let store = resolve_tier_store(global, work_store, tier)?;

    let delimiter = delimiter_from_path(path);
    let contents = export_tier(store, delimiter)?;
    write_export_file(path, &contents)?;
    Ok(())
}

/// Hình dạng "mô hình đã kiểm" của MỘT hàng bất đồng cho màn hình xem trước — AD-48 §Rule
/// ①: cả hai bản dịch, không văn bản thô nào khác của tệp.
#[derive(Debug, Clone, serde::Serialize)]
pub struct ImportPreviewConflictWire {
    pub source_term: String,
    /// Bản dịch ĐANG CÓ trong kho, trước khi có quyết định nào.
    pub existing_translation: Option<String>,
    /// Bản dịch tệp mang.
    pub file_translation: Option<String>,
}

/// Hình dạng "mô hình đã kiểm" của màn hình xem trước lượt nhập — AD-48 §Rule ①: nội dung
/// tệp KHÔNG đi ra đây, chỉ số liệu ĐÃ PHÂN TÍCH VÀ ĐÃ PHÂN LOẠI.
///
/// ⚠️ `#[serde(rename_all = ...)]` KHÔNG đặt — cùng luật mọi struct qua biên IPC.
#[derive(Debug, Clone, serde::Serialize)]
pub struct ImportPreviewWire {
    pub file_name: String,
    /// `"global"` hoặc `"work"` — tầng đã chọn lúc mở, khớp [`PendingImport::tier`].
    pub tier: String,
    pub row_count: usize,
    /// `header_columns.len() - ignored_columns.len()` — "N cột nhận ra được" của mockup.
    pub recognized_column_count: usize,
    /// Tên cột lạ ở hàng tiêu đề — NÓI RA, không im lặng vứt (I/O Matrix).
    pub ignored_columns: Vec<String>,
    /// `true` ⇔ hàng tiêu đề có cột `term_origin` — cột này LUÔN bị đọc rồi bỏ giá trị
    /// (mọi mục vào đều mang `file_import`, §Design Notes). Đây là chỗ DUY NHẤT hiển thị
    /// được sự thật đó cho người dùng.
    pub term_origin_column_present: bool,
    pub new_count: usize,
    pub identical_count: usize,
    /// Mọi hàng *bất đồng* — mặc định giữ của tôi (frontend không gửi quyết định cho hàng
    /// người dùng không đổi ý).
    pub conflicts: Vec<ImportPreviewConflictWire>,
}

/// Mở-và-xem-trước lượt nhập (nhịp MỘT) — **hàm thuần theo nghĩa không chạm `AppHandle`
/// hay hộp thoại**: `path` đã được vỏ `wire` chọn xong; hàm này đọc, phân tích, phân loại
/// so với `tier`, rồi GIỮ kế hoạch trong `pending` — AD-48 §Rule ①.
///
/// 🔴 **`pending` là chính `Mutex` được `.manage(...)`, nhận qua tham số** — cùng khuôn
/// `commands::project::filter_and_enqueue_current_import_scan`: test khoá/mở được không
/// cần webview, và một lô CŨ còn treo bị THAY vô điều kiện (§I/O Matrix: "mở lô thứ hai
/// khi lô cũ còn treo ⇒ lô mới thay lô cũ; lô cũ không bao giờ ghi được nữa").
///
/// # Lỗi
/// - `global.db` vắng mặt ⇒ `store.open_failed`;
/// - `tier == Work` mà chưa mở Tác phẩm nào ⇒ `glossary.work_tier_unavailable`;
/// - đọc tệp (kích thước/UTF-8/hạ tầng) ⇒ ba khoá `exchange_io` tương ứng — **0** lô nào
///   được giữ lại khi bước đọc trượt;
/// - phân tích hỏng ⇒ `IpcError` của [`crate::core::i18n::MessageKey`] ứng với lỗi ĐẦU
///   TIÊN tìm được (đường dây chở đúng MỘT lỗi; `parse` đã tự gộp toàn bộ vào chẩn đoán
///   log) — **0** lô nào được giữ lại.
pub fn glossary_open_import_preview(
    global: Option<&Store>,
    open: Option<&OpenWork>,
    pending: &PendingImportState,
    tier: GlossaryTier,
    path: &Path,
) -> Result<ImportPreviewWire, IpcError> {
    let global_store = global.ok_or_else(store_is_missing)?;
    let work_store = work_context(open).map(|(store, _)| store);
    // Xac nhan tang chon duoc TRUOC khi cham dia -- cung mot mon voi duong xuat.
    resolve_tier_store(global_store, work_store, tier)?;

    let text = read_import_file(path)?;
    let parsed = parse_glossary_import(&text).map_err(|issues| {
        eprintln!(
            "glossary[import_preview] {} loi phan tich, dau tien: {}",
            issues.len(),
            issues[0]
        );
        IpcError::from(issues.into_iter().next().expect("Err chi dung khi issues khong rong"))
    })?;

    let plans = classify_import_rows(global_store, work_store, tier, &parsed.rows)?;

    let new_count = plans.iter().filter(|p| matches!(p.kind, RowPlanKind::New)).count();
    let identical_count = plans.iter().filter(|p| matches!(p.kind, RowPlanKind::Identical)).count();
    let conflicts: Vec<ImportPreviewConflictWire> = plans
        .iter()
        .filter_map(|p| match &p.kind {
            RowPlanKind::Conflict { existing_translation, .. } => Some(ImportPreviewConflictWire {
                source_term: p.source_term.clone(),
                existing_translation: existing_translation.clone(),
                file_translation: p.translation.clone(),
            }),
            _ => None,
        })
        .collect();

    let file_name = path.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default();
    let recognized_column_count = parsed.header_columns.len().saturating_sub(parsed.ignored_columns.len());
    let term_origin_column_present =
        parsed.header_columns.iter().any(|c| c == "term_origin");

    let preview = ImportPreviewWire {
        file_name,
        tier: tier.as_str().to_owned(),
        row_count: parsed.rows.len(),
        recognized_column_count,
        ignored_columns: parsed.ignored_columns,
        term_origin_column_present,
        new_count,
        identical_count,
        conflicts,
    };

    let mut guard = pending.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
    *guard = Some(PendingImport { path: path.to_owned(), tier, plans });

    Ok(preview)
}

/// Hình dạng trên dây của [`ImportSummary`] — Story 3.10b.
#[derive(Debug, Clone, Copy, serde::Serialize)]
pub struct ImportSummaryWire {
    pub inserted: i64,
    pub updated: i64,
    pub identical: i64,
}

impl From<ImportSummary> for ImportSummaryWire {
    fn from(s: ImportSummary) -> Self {
        Self { inserted: s.inserted, updated: s.updated, identical: s.identical }
    }
}

/// Xác nhận lượt nhập (nhịp HAI) — ghi `decisions` cho lô ĐANG TREO trong `pending`.
///
/// 🔴 **Kế hoạch chỉ dọn khỏi `pending` khi giao dịch THÀNH CÔNG.** Lỗi giữa chừng (kể cả
/// `ImportDecisionUnknownTerm`) GIỮ LẠI lô để người dùng thử lại — đúng chữ của §I/O
/// Matrix "Lỗi giữa chừng ⇒ rollback trọn, kế hoạch GIỮ LẠI để thử lại".
///
/// # Lỗi
/// - `global.db` vắng mặt ⇒ `store.open_failed`;
/// - không có lô nào đang treo ⇒ `glossary.no_pending_import`;
/// - một khoá của `decisions` không khớp `source_term` nào trong lô ⇒
///   `glossary.import_decision_unknown_term`, **0** lượt ghi, lô GIỮ LẠI;
/// - `tier == Work` mà Tác phẩm đã đóng từ lúc mở lô ⇒ `glossary.work_tier_unavailable`;
/// - va `UNIQUE` giữa chừng ⇒ `glossary.import_unique_conflict`, lô GIỮ LẠI.
pub fn glossary_confirm_import(
    global: Option<&Store>,
    open: Option<&OpenWork>,
    pending: &PendingImportState,
    decisions: &BTreeMap<String, ConflictDecision>,
) -> Result<ImportSummaryWire, IpcError> {
    let global = global.ok_or_else(store_is_missing)?;
    let work_store = work_context(open).map(|(store, _)| store);

    let mut guard = pending.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
    let Some(batch) = guard.as_ref() else {
        return Err(IpcError::from(GlossaryError::NoPendingImport));
    };

    // Moi khoa cua `decisions` PHAI khop mot source_term MANG RowPlanKind::Conflict trong
    // lo -- khong roi vao hu khong (§Always: "mot quyet dinh tro toi source_term khong co
    // trong lo la mot loi tuong minh"). Kiem TRUOC khi ghi.
    //
    // 🔴 P5 (vòng rà ba lớp 2026-08-25) — SIẾT xuống đúng hàng `Conflict`, không phải MỌI
    // hàng của lô. Bản trước gom `source_term` của CẢ `New`/`Identical`/`Conflict`, nên một
    // quyết định trỏ vào một hàng `New`/`Identical` (những hàng KHÔNG có khái niệm "giữ của
    // tôi"/"lấy của file" — `import_into_tier` chỉ tra `decisions` cho nhánh `Conflict`) qua
    // được phép kiểm này rồi KHÔNG có tác dụng gì, im lặng — đúng lớp lỗi mà chính phép
    // kiểm này tồn tại để chặn, chỉ lùi một hàng.
    let known_conflict_terms: BTreeSet<&str> = batch
        .plans
        .iter()
        .filter(|p| matches!(p.kind, RowPlanKind::Conflict { .. }))
        .map(|p| p.source_term.as_str())
        .collect();
    if let Some(unknown) = decisions.keys().find(|k| !known_conflict_terms.contains(k.as_str())) {
        return Err(IpcError::from(GlossaryError::ImportDecisionUnknownTerm {
            term: unknown.clone(),
        }));
    }

    match import_into_tier(global, work_store, batch.tier, &batch.plans, decisions) {
        Ok(summary) => {
            *guard = None; // Chi don LO khi giao dich THANH CONG.
            Ok(ImportSummaryWire::from(summary))
        }
        Err(e) => Err(IpcError::from(e)), // Lo GIU LAI -- `guard` khong bi cham.
    }
}

/// Huỷ lô đang treo — **0** lượt ghi, không lỗi kể cả khi không có lô nào (huỷ hai lần là
/// vô hại, cùng khuôn `closeGlossaryQueue`/`closeGlossaryManage` phía frontend).
pub fn glossary_cancel_import(pending: &PendingImportState) {
    let mut guard = pending.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
    *guard = None;
}

/// Dọn lô đang treo NẾU nó thuộc tầng `tier` — **hàm thuần**, gọi từ hai chỗ trong
/// `lib.rs`/`commands::project`: đóng Tác phẩm (`RunEvent::Exit`) VÀ mở một Tác phẩm KHÁC
/// (`replace_open_work`), cả hai đều làm store `project.db` của lô đang treo (nếu có)
/// biến mất hoặc đổi ý nghĩa — một `RowPlan::Conflict::existing_id` chốt từ kho CŨ không
/// còn trỏ đúng hàng nào ở kho MỚI. §I/O Matrix: "Đóng Tác phẩm khi còn lô nhập treo ⇒ Lô
/// bị dọn; nhịp hai sau đó trả `NoPendingImport`".
pub fn clear_pending_import_for_tier(pending: &PendingImportState, tier: GlossaryTier) {
    let mut guard = pending.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
    if guard.as_ref().is_some_and(|batch| batch.tier == tier) {
        *guard = None;
    }
}

/// Mười lăm vỏ `#[tauri::command]`. **Không một quy tắc nào sống ở đây.**
pub mod wire {
    use std::collections::BTreeMap;

    use super::{
        Category, ConflictDecision, GlossaryCandidateWire, GlossaryEntryWire, GlossaryError,
        GlossaryMarkWire, GlossaryTier, ImportPreviewWire, ImportSummaryWire, IpcError,
        PendingImportState, QuickAddLookup,
    };
    use crate::commands::project::OpenWorkState;
    use crate::core::dict::DictLayers;
    use crate::core::store::Store;
    use tauri_plugin_dialog::DialogExt as _;

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

    /// Vỏ IPC của [`super::glossary_reject_candidate`]. Story 3.8.
    #[tauri::command]
    pub fn glossary_reject_candidate(app: tauri::AppHandle, id: i64) -> Result<(), IpcError> {
        use tauri::Manager as _;

        let Some(work_state) = app.try_state::<OpenWorkState>() else {
            return super::glossary_reject_candidate(None, id);
        };
        let guard = work_state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        super::glossary_reject_candidate(guard.as_ref(), id)
    }

    /// Vỏ IPC của [`super::glossary_list_entries`]. Story 3.9.
    #[tauri::command]
    pub fn glossary_list_entries(app: tauri::AppHandle) -> Result<Vec<GlossaryEntryWire>, IpcError> {
        use tauri::Manager as _;

        let global = app.try_state::<Store>();
        let Some(work_state) = app.try_state::<OpenWorkState>() else {
            return super::glossary_list_entries(global.as_deref(), None);
        };
        let guard = work_state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        super::glossary_list_entries(global.as_deref(), guard.as_ref())
    }

    /// Vỏ IPC của [`super::glossary_delete_term`]. Story 3.9.
    #[tauri::command]
    pub fn glossary_delete_term(app: tauri::AppHandle, tier: GlossaryTier, id: i64) -> Result<(), IpcError> {
        use tauri::Manager as _;

        let global = app.try_state::<Store>();
        let Some(work_state) = app.try_state::<OpenWorkState>() else {
            return super::glossary_delete_term(global.as_deref(), None, tier, id);
        };
        let guard = work_state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        super::glossary_delete_term(global.as_deref(), guard.as_ref(), tier, id)
    }

    /// Vỏ IPC của [`super::glossary_promote_term_to_global`]. Story 3.9.
    #[tauri::command]
    pub fn glossary_promote_term_to_global(app: tauri::AppHandle, id: i64) -> Result<(), IpcError> {
        use tauri::Manager as _;

        let global = app.try_state::<Store>();
        let Some(work_state) = app.try_state::<OpenWorkState>() else {
            return super::glossary_promote_term_to_global(global.as_deref(), None, id);
        };
        let guard = work_state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        super::glossary_promote_term_to_global(global.as_deref(), guard.as_ref(), id)
    }

    // ── Story 3.10b (AD-48) — hộp thoại chọn tệp nối vào xuất/nhập ──────────────────

    /// `OpenWorkState` đang mang một `OpenWork` hay không — khoá GIỮ ĐÚNG một biểu thức,
    /// `MutexGuard` KHÔNG thoát ra ngoài hàm này.
    ///
    /// 🔴 **P1 (vòng rà ba lớp 2026-08-25).** Bản trước khoá `OpenWorkState` một lần rồi
    /// GIỮ `MutexGuard` xuyên suốt `blocking_save_file()`/`blocking_pick_file()` — hộp
    /// thoại hệ điều hành có thể mở NHIỀU PHÚT (người dùng duyệt thư mục), và giữ khoá đó
    /// chặn MỌI lệnh khác cần `OpenWorkState` trong lúc đó, gồm cả đường flush AD-35 (trần
    /// cứng 5s, KHÔNG reset bởi phím gõ). Hàm này là chỗ DUY NHẤT khoá `OpenWorkState` để
    /// hỏi "có đang mở không" — khoá mở rồi đóng ngay trong một biểu thức, không có biến
    /// `guard` nào sống ra khỏi nó.
    fn work_tier_is_open(app: &tauri::AppHandle) -> bool {
        use tauri::Manager as _;
        app.try_state::<OpenWorkState>()
            .map(|s| s.lock().unwrap_or_else(std::sync::PoisonError::into_inner).is_some())
            .unwrap_or(false)
    }

    /// Vỏ IPC của [`super::glossary_export_tier`] — mở hộp thoại LƯU rồi gọi hàm thuần.
    ///
    /// 🔴 **P1 — kiểm CẢ BA (`Store` có mặt · `tier == Work` thì Tác phẩm phải đang mở)
    /// TRƯỚC khi mở hộp thoại**, và tra `OpenWorkState` qua [`work_tier_is_open`] — một
    /// khoá NGẮN, không giữ `MutexGuard` qua lượt `blocking_save_file()` (xem doc-comment
    /// của hàm đó). Không lãng phí một lượt tương tác người dùng cho một thao tác chắc
    /// chắn trượt.
    ///
    /// 🔴 **Tác phẩm có thể ĐÓNG trong lúc hộp thoại còn mở** — `OpenWorkState` được khoá
    /// LẦN THỨ HAI, MỚI, ngay TRƯỚC khi gọi hàm thuần ghi tệp, không tái dùng giá trị đã
    /// đọc trước dialog. Một Tác phẩm đóng giữa chừng ⇒ `open` ở lần khoá thứ hai là
    /// `None` ⇒ hàm thuần tự trả `WorkTierUnavailable` qua `resolve_tier_store`, **0**
    /// lượt ghi — không ghi vào một kho đã cũ.
    ///
    /// Trả `Ok(None)` khi người dùng HUỶ hộp thoại (§Always: "Huỷ hộp thoại là `Ok(None)`,
    /// không một biến thể lỗi") — `Ok(Some(path))` mang đường dẫn đã ghi khi thành công.
    /// 🔴 **`(async)` KHÔNG PHẢI TRANG TRÍ — thiếu nó là TREO ỨNG DỤNG.** Tauri chạy một
    /// `#[tauri::command]` ĐỒNG BỘ trên **luồng chính**; `blocking_save_file()`/
    /// `blocking_pick_file()` chặn ở đó, tức chặn đúng vòng lặp sự kiện mà hộp thoại đang
    /// chờ ⇒ bế tắc, macOS báo *"Open and Save Panel Service (auratranslate) (Not
    /// Responding)"*. Đo 2026-08-25 trên cửa sổ thật của Ice.
    ///
    /// `#[tauri::command(async)]` trên một hàm ĐỒNG BỘ cho `sync_threadpool`
    /// (`tauri-macros-2.6.3/src/command/wrapper.rs:264`) — chạy ngoài luồng chính, **không
    /// đổi một dòng thân hàm**. Cùng vai với việc lệnh `open` của chính plugin là
    /// `async fn` (`tauri-plugin-dialog-2.7.2/src/commands.rs:121`), thứ mà bản đầu của
    /// story này nhìn thấy `blocking_pick_file` bên trong rồi kết luận nhầm là an toàn ở
    /// một lệnh đồng bộ. Cổng canh:
    /// `config_invariants.rs::the_dialog_wires_run_off_the_main_thread`.
    #[tauri::command(async)]
    pub fn glossary_export_tier(app: tauri::AppHandle, tier: GlossaryTier) -> Result<Option<String>, IpcError> {
        use tauri::Manager as _;

        let global = app.try_state::<Store>();
        if global.is_none() {
            return Err(IpcError::from(super::store_is_missing()));
        }
        if tier == GlossaryTier::Work && !work_tier_is_open(&app) {
            return Err(IpcError::from(GlossaryError::WorkTierUnavailable));
        }

        let extension = if tier == GlossaryTier::Global { "glossary_global" } else { "glossary_work" };
        // 🔴 P4 (vòng rà ba lớp 2026-08-25) — CHỈ MỘT bộ lọc, khớp tên mặc định.
        // `set_file_name` không có móc phản ứng khi người dùng đổi bộ lọc trong hộp thoại
        // (API `blocking_save_file` của `rfd` không phát sự kiện đó ra ngoài), nên trước
        // đây bộ lọc TSV có mặt trong khi tên mặc định luôn `.csv` — chọn bộ lọc TSV vẫn
        // nhận tên `….csv` và phải tự gõ lại đuôi mới thật sự ra TSV, một cái bẫy im lặng.
        // Dấu phân cách vẫn suy TRỌN VẸN từ đuôi THẬT của đường dẫn đã chọn
        // (`delimiter_from_path`, §I/O Matrix "Đuôi tệp quyết dấu phân cách") — người dùng
        // muốn TSV vẫn gõ được `….tsv` trong ô tên, chỉ là hộp thoại không còn GỢI Ý một
        // lựa chọn mà tên mặc định không theo kịp.
        let Some(picked) = app
            .dialog()
            .file()
            .add_filter("CSV", &["csv"])
            .set_file_name(format!("{extension}.csv"))
            .blocking_save_file()
        else {
            return Ok(None);
        };
        let path = picked.into_path().map_err(|_| IpcError::from(GlossaryError::DialogPathInvalid))?;

        // Khoá MỚI, sau khi hộp thoại đã đóng — không tái dùng bất kỳ giá trị nào đọc
        // trước dialog (P1).
        let work_state = app.try_state::<OpenWorkState>();
        let guard = work_state
            .as_ref()
            .map(|s| s.lock().unwrap_or_else(std::sync::PoisonError::into_inner));
        let open = guard.as_ref().and_then(|g| g.as_ref());

        super::glossary_export_tier(global.as_deref(), open, tier, &path)?;
        Ok(Some(path.display().to_string()))
    }

    /// Vỏ IPC của [`super::glossary_open_import_preview`] — mở hộp thoại CHỌN rồi gọi hàm
    /// thuần. Nhịp MỘT của lượt nhập.
    ///
    /// 🔴 **P1 — kiểm CẢ BA (`Store` · `tier == Work` · `PendingImportState`) TRƯỚC khi mở
    /// hộp thoại**, cùng lý do và cùng khuôn [`glossary_export_tier`] ngay trên — bản
    /// trước chỉ kiểm `tier`/`Store` trước dialog, còn `PendingImportState` nổ SAU khi
    /// người dùng đã chọn xong tệp, đúng thứ chính doc-comment của hàm này từng khai là
    /// không được làm.
    ///
    /// 🔴 **Tác phẩm có thể ĐÓNG trong lúc hộp thoại còn mở** — `OpenWorkState` khoá LẦN
    /// THỨ HAI, MỚI, sau khi hộp thoại trả về, cùng lý do [`glossary_export_tier`].
    ///
    /// Trả `Ok(None)` khi người dùng HUỶ hộp thoại — không đọc gì, không lỗi, **không** kế
    /// hoạch nào để lại trong `State` (§I/O Matrix).
    /// 🔴 **`(async)` KHÔNG PHẢI TRANG TRÍ — thiếu nó là TREO ỨNG DỤNG.** Tauri chạy một
    /// `#[tauri::command]` ĐỒNG BỘ trên **luồng chính**; `blocking_save_file()`/
    /// `blocking_pick_file()` chặn ở đó, tức chặn đúng vòng lặp sự kiện mà hộp thoại đang
    /// chờ ⇒ bế tắc, macOS báo *"Open and Save Panel Service (auratranslate) (Not
    /// Responding)"*. Đo 2026-08-25 trên cửa sổ thật của Ice.
    ///
    /// `#[tauri::command(async)]` trên một hàm ĐỒNG BỘ cho `sync_threadpool`
    /// (`tauri-macros-2.6.3/src/command/wrapper.rs:264`) — chạy ngoài luồng chính, **không
    /// đổi một dòng thân hàm**. Cùng vai với việc lệnh `open` của chính plugin là
    /// `async fn` (`tauri-plugin-dialog-2.7.2/src/commands.rs:121`), thứ mà bản đầu của
    /// story này nhìn thấy `blocking_pick_file` bên trong rồi kết luận nhầm là an toàn ở
    /// một lệnh đồng bộ. Cổng canh:
    /// `config_invariants.rs::the_dialog_wires_run_off_the_main_thread`.
    #[tauri::command(async)]
    pub fn glossary_open_import_preview(
        app: tauri::AppHandle,
        tier: GlossaryTier,
    ) -> Result<Option<ImportPreviewWire>, IpcError> {
        use tauri::Manager as _;

        let global = app.try_state::<Store>();
        if global.is_none() {
            return Err(IpcError::from(super::store_is_missing()));
        }
        if tier == GlossaryTier::Work && !work_tier_is_open(&app) {
            return Err(IpcError::from(GlossaryError::WorkTierUnavailable));
        }
        if app.try_state::<PendingImportState>().is_none() {
            eprintln!(
                "glossary[import_preview] PendingImportState chua duoc quan ly -- loi cau hinh setup()"
            );
            return Err(IpcError::from(GlossaryError::NoPendingImport));
        }

        let Some(picked) = app.dialog().file().add_filter("CSV/TSV", &["csv", "tsv"]).blocking_pick_file()
        else {
            return Ok(None);
        };
        let path = picked.into_path().map_err(|_| IpcError::from(GlossaryError::DialogPathInvalid))?;

        // Khoá MỚI, sau khi hộp thoại đã đóng — không tái dùng bất kỳ giá trị nào đọc
        // trước dialog (P1).
        let work_state = app.try_state::<OpenWorkState>();
        let guard = work_state
            .as_ref()
            .map(|s| s.lock().unwrap_or_else(std::sync::PoisonError::into_inner));
        let open = guard.as_ref().and_then(|g| g.as_ref());

        let Some(pending) = app.try_state::<PendingImportState>() else {
            // Đã kiểm TRƯỚC dialog — đây là một cửa sổ đua HIẾM (state bị gỡ giữa chừng,
            // không xảy ra trên đường sản phẩm bình thường), không phải đường thường nhật.
            return Err(IpcError::from(GlossaryError::NoPendingImport));
        };

        let preview =
            super::glossary_open_import_preview(global.as_deref(), open, pending.inner(), tier, &path)?;
        Ok(Some(preview))
    }

    /// Vỏ IPC của [`super::glossary_confirm_import`] — nhịp HAI của lượt nhập.
    #[tauri::command]
    pub fn glossary_confirm_import(
        app: tauri::AppHandle,
        decisions: BTreeMap<String, ConflictDecision>,
    ) -> Result<ImportSummaryWire, IpcError> {
        use tauri::Manager as _;

        let global = app.try_state::<Store>();
        let work_state = app.try_state::<OpenWorkState>();
        let guard = work_state
            .as_ref()
            .map(|s| s.lock().unwrap_or_else(std::sync::PoisonError::into_inner));
        let open = guard.as_ref().and_then(|g| g.as_ref());

        let Some(pending) = app.try_state::<PendingImportState>() else {
            return Err(IpcError::from(GlossaryError::NoPendingImport));
        };

        super::glossary_confirm_import(global.as_deref(), open, pending.inner(), &decisions)
    }

    /// Vỏ IPC của [`super::glossary_cancel_import`] — huỷ lô đang treo, dùng cả cho nút Huỷ
    /// của màn hình xem trước.
    #[tauri::command]
    pub fn glossary_cancel_import(app: tauri::AppHandle) -> Result<(), IpcError> {
        use tauri::Manager as _;

        if let Some(pending) = app.try_state::<PendingImportState>() {
            super::glossary_cancel_import(pending.inner());
        }
        Ok(())
    }
}
