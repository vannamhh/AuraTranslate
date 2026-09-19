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

use std::path::Path;

use crate::commands::project::OpenWork;
use crate::core::i18n::IpcError;
use crate::core::promptset::exchange::{ConflictDecision, ParsedPromptSet, PlanKind};
use crate::core::promptset::{
    ImportOutcome, MarkerWarnings, PromptSetError, PromptSetTier, PromptVariable, ResolvedPromptSet,
    create, delete, exchange, exchange_io, import_into_tier, load_one, load_prompt_set_tier, rename,
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
    /// 🔵 **THÊM Story 4.5, Quyết định #3.** `id` THẬT của hàng Global bị che — cùng điều
    /// kiện `None`/`Some` với `shadowed_body`. Cho phép màn hình vẽ hàng đó như một hàng
    /// CHỌN ĐƯỢC (`tier: "global"`, `id` này) thay vì một dòng chỉ-hiển-thị, đóng
    /// `deferred-work.md §*Deferred from: 4-2-cau-hinh-nha-cung-cap-ai (2026-09-16)*`.
    pub shadowed_id: Option<i64>,
}

impl From<ResolvedPromptSet> for PromptSetWire {
    fn from(r: ResolvedPromptSet) -> Self {
        Self {
            id: r.id,
            name: r.name,
            body: r.body,
            tier: r.tier.into(),
            shadowed_body: r.shadowed_body,
            shadowed_id: r.shadowed_id,
        }
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

// ═════════════════════════════════════════════════════════════════════════════════
// Story 4.5 (FR79, NFR9, AD-48) — xuất/nhập một bộ prompt qua tệp `.prompt.md`
// ═════════════════════════════════════════════════════════════════════════════════
//
// Bốn vỏ mới: xuất (một nhịp, mở hộp thoại LƯU) · mở-và-xem-trước một lượt nhập (nhịp một,
// mở hộp thoại CHỌN — KHÔNG hỏi tầng trước khi đọc tệp, xem doc-comment
// `prompt_set_open_import_preview`) · xác nhận (nhịp hai, nhận tầng người dùng chọn Ở MÀN
// XEM TRƯỚC cộng quyết định va chạm) · huỷ lô đang treo. Không một lệnh `fs:*`/`dialog:*` nào
// phơi ra JavaScript — webview chỉ dispatch bốn lệnh này (`capabilities/main.json` không đổi,
// §Always spec 4.5).

/// Tên tệp mặc định của lượt xuất — `<tên bộ>.prompt.md` (Quyết định #2: "the filename
/// defaulted from the set name"). Không sanitize ký tự đặc biệt của `name` — nằm ngoài phạm
/// vi I/O Matrix của story này; hộp thoại hệ điều hành là nơi người dùng sửa nếu tên mang một
/// ký tự tên tệp không hợp lệ trên máy họ.
fn default_export_file_name(name: &str) -> String {
    format!("{name}.prompt.md")
}

/// Xuất bộ `(tier, id)` ra `path` — **hàm thuần, đây là thứ test gọi**. Một NHỊP. Thao tác
/// trên hàng THẬT theo `id` (không qua [`resolve_two_tiers`]) — Quyết định #3: một bộ Global
/// đang bị che vẫn xuất được, đúng như chính nó, qua `(tier: Global, id: shadowed_id)`.
///
/// # Lỗi
/// - `global.db` vắng mặt ⇒ `store.open_failed`;
/// - `tier == Work` mà chưa mở Tác phẩm nào ⇒ `prompt_set.work_tier_unavailable`;
/// - `(tier, id)` không khớp hàng nào ⇒ `prompt_set.not_found`;
/// - ghi tệp thất bại ⇒ `prompt_set.export_write_failed`, **0** tệp cụt để lại
///   (`write_export_file` dọn `.tmp` ở cả hai nhánh lỗi).
pub fn prompt_set_export(
    global: Option<&Store>,
    open: Option<&OpenWork>,
    tier: PromptSetTier,
    id: i64,
    path: &Path,
) -> Result<(), IpcError> {
    let global_store = global.ok_or_else(store_is_missing)?;
    let store = crate::core::promptset::store::store_for_tier(global_store, open.map(|w| &w.store), tier)?;
    let row = load_one(store, id)?;
    let contents = exchange::render(&row.name, &row.body);
    exchange_io::write_export_file(path, &contents)?;
    Ok(())
}

/// Lô nhập đang TREO giữa nhịp một (mở + xem trước) và nhịp hai (xác nhận) — AD-48 §Rule ①:
/// nội dung tệp KHÔNG BAO GIỜ đi ra webview, `name`/`body` đã phân tích Ở LẠI RUST. Không mang
/// `tier` — khác [`crate::commands::glossary::PendingImport`] — vì Quyết định của spec 4.5
/// (I/O Matrix "Import picks the tier") đặt lượt CHỌN TẦNG ở màn xem trước, SAU khi tệp đã
/// đọc xong, không trước như Glossary; phân loại vì thế được tính SẴN cho CẢ HAI tầng ngay ở
/// nhịp một, và nhịp hai chỉ chọn nhánh nào đã có.
#[derive(Debug)]
pub struct PendingPromptImport {
    /// Đường dẫn tệp đã đọc — chỉ để chẩn đoán, KHÔNG đọc lại ở nhịp hai.
    pub path: std::path::PathBuf,
    pub parsed: ParsedPromptSet,
    /// Phân loại so với tầng Toàn cục — LUÔN có (tầng Toàn cục luôn sẵn khi `global.db` đã
    /// mở).
    pub global_kind: PlanKind,
    /// Phân loại so với tầng Tác phẩm — `None` khi không có Tác phẩm nào đang mở lúc XEM
    /// TRƯỚC (I/O Matrix "Import with no Work open ... Work option absent").
    pub work_kind: Option<PlanKind>,
}

/// Kiểu state Tauri quản lý cho [`PendingPromptImport`] — `None` == không lô nào đang treo,
/// cùng khuôn `commands::glossary::PendingImportState`.
pub type PendingPromptImportState = std::sync::Mutex<Option<PendingPromptImport>>;

/// Dọn nửa TÁC PHẨM của lô đang treo (nếu có) khi Tác phẩm đóng/đổi — cùng lý do
/// `commands::glossary::clear_pending_import_for_tier`, nhưng CHỈ hạ `work_kind` về `None`
/// thay vì xoá TRỌN lô: nửa Toàn cục của lô (nếu người dùng định nhập vào đó) vẫn còn dùng
/// được, không phụ thuộc Tác phẩm nào.
pub fn clear_pending_prompt_import_work_tier(pending: &PendingPromptImportState) {
    let mut guard = pending.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
    if let Some(batch) = guard.as_mut() {
        batch.work_kind = None;
    }
}

/// Hình dạng "mô hình đã kiểm" của phân loại MỘT tầng cho màn hình xem trước.
#[derive(Debug, Clone, serde::Serialize)]
pub struct PromptImportTierPreviewWire {
    /// `"new"` · `"identical"` · `"conflict"`.
    pub kind: &'static str,
    /// `Some` chỉ khi `kind == "conflict"` — thân ĐANG CÓ trong kho, để người dùng so trước
    /// khi quyết định.
    pub existing_body: Option<String>,
}

impl From<&PlanKind> for PromptImportTierPreviewWire {
    fn from(kind: &PlanKind) -> Self {
        match kind {
            PlanKind::New => Self { kind: "new", existing_body: None },
            PlanKind::Identical => Self { kind: "identical", existing_body: None },
            PlanKind::Conflict { existing_body, .. } => {
                Self { kind: "conflict", existing_body: Some(existing_body.clone()) }
            }
        }
    }
}

/// Hình dạng "mô hình đã kiểm" của màn hình xem trước lượt nhập — AD-48 §Rule ①: `name`/`body`
/// đi trên dây như dữ liệu ĐÃ PHÂN TÍCH (khớp cách Glossary gửi `file_translation` của một
/// hàng bất đồng), không phải byte thô của tệp.
#[derive(Debug, Clone, serde::Serialize)]
pub struct PromptImportPreviewWire {
    pub file_name: String,
    pub name: String,
    pub body: String,
    pub warnings: PromptSetWarningsWire,
    pub global: PromptImportTierPreviewWire,
    /// `None` ⇔ không có Tác phẩm nào đang mở — Work option ABSENT, không một tuỳ chọn bị vô
    /// hiệu hoá rỗng (I/O Matrix).
    pub work: Option<PromptImportTierPreviewWire>,
}

/// Mở-và-xem-trước lượt nhập (nhịp MỘT) — **hàm thuần theo nghĩa không chạm `AppHandle` hay
/// hộp thoại**: `path` đã được vỏ `wire` chọn xong.
///
/// 🔴 **KHÔNG nhận `tier`** — khác `glossary_open_import_preview`. I/O Matrix spec 4.5
/// "Import picks the tier: User chooses Global or Work at preview" đặt lượt chọn tầng SAU khi
/// tệp đã đọc, nên hàm này phân loại `(name, body)` so với CẢ HAI tầng đang có ngay bây giờ,
/// giữ cả hai kết quả trong `pending` để nhịp hai chọn đúng nhánh mà không phải đọc/phân tích
/// lại tệp.
///
/// # Lỗi
/// - `global.db` vắng mặt ⇒ `store.open_failed`;
/// - đọc tệp (kích thước/UTF-8/hạ tầng) ⇒ ba khoá tương ứng — **0** lô nào được giữ lại;
/// - phân tích hỏng ⇒ `IpcError` gộp MỌI dòng hỏng (`issues_to_ipc_error` — AC4: "every
///   problem is reported with its line number") — **0** lô nào được giữ lại.
pub fn prompt_set_open_import_preview(
    global: Option<&Store>,
    open: Option<&OpenWork>,
    pending: &PendingPromptImportState,
    path: &Path,
) -> Result<PromptImportPreviewWire, IpcError> {
    let global_store = global.ok_or_else(store_is_missing)?;

    let text = exchange_io::read_import_file(path)?;
    let parsed = exchange::parse(&text).map_err(issues_to_ipc_error)?;
    let warnings = crate::core::promptset::scan_markers(&parsed.body);

    let global_existing = load_prompt_set_tier(global_store)?;
    let global_kind = exchange::classify(&parsed.name, &parsed.body, &global_existing);

    let work_kind = match open {
        None => None,
        Some(w) => {
            let work_existing = load_prompt_set_tier(&w.store)?;
            Some(exchange::classify(&parsed.name, &parsed.body, &work_existing))
        }
    };

    let file_name = path.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default();
    let preview = PromptImportPreviewWire {
        file_name,
        name: parsed.name.clone(),
        body: parsed.body.clone(),
        warnings: warnings.into(),
        global: PromptImportTierPreviewWire::from(&global_kind),
        work: work_kind.as_ref().map(PromptImportTierPreviewWire::from),
    };

    let mut guard = pending.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
    *guard = Some(PendingPromptImport { path: path.to_owned(), parsed, global_kind, work_kind });

    Ok(preview)
}

/// **Hàm thuần** — tách khỏi chỗ gọi, khác chủ ý với
/// `commands::glossary::first_issue_or_unknown` (cụm F ①, spec 3.10b, "hành vi hôm nay, không
/// đổi" — `spec-epic-3-review-cum-f-muc-rai-rac-bon-tang.md:52`): Glossary chỉ đưa lỗi ĐẦU
/// TIÊN ra `IpcError` cho một tệp N-hàng có thể mang hàng trăm lỗi. AC4 spec 4.5 lại đòi
/// nguyên văn *"every problem is reported with its line number"* cho MỘT tệp `.prompt.md` —
/// và vì khối metadata chỉ có ĐÚNG BA dòng cố định (tối đa BA `ParseIssue`), gộp TOÀN BỘ vào
/// MỘT câu là rẻ và không cần cắt bớt như trường hợp Glossary. Đúng MỘT issue vẫn dùng khoá
/// RIÊNG của issue đó (câu tự nhiên hơn, và giữ nguyên hành vi/test đã có cho ca đơn); NHIỀU
/// hơn một issue dùng khoá gộp `PromptSetImportMalformed`, liệt kê MỌI dòng hỏng.
///
/// Ca `issues` rỗng (bất biến `parse()` luôn kèm ít nhất một `ParseIssue` khi trả `Err` đã vỡ)
/// vẫn có một phép kiểm CHẠY ĐƯỢC thay vì một lời khai, cùng lý do hàm gốc nó tách ra từ.
pub fn issues_to_ipc_error(issues: Vec<exchange::ParseIssue>) -> IpcError {
    if issues.is_empty() {
        eprintln!(
            "prompt_set[import_preview] bat bien vo: Err(issues) voi issues RONG (0 loi) -- \
             parse() phai luon kem it nhat mot ParseIssue khi tra Err"
        );
        return IpcError::new(
            "prompt_set.import_parse_issues_empty",
            crate::core::i18n::MessageKey::Unknown,
            std::collections::BTreeMap::new(),
            false,
        );
    }

    if issues.len() == 1 {
        let only = issues.into_iter().next().expect("kiem len() == 1 ngay tren");
        eprintln!("prompt_set[import_preview] 1 loi phan tich: {only}");
        return IpcError::from(only);
    }

    let issue_count = issues.len();
    let mut lines: Vec<&'static str> = issues.iter().map(exchange::ParseIssue::line).collect();
    lines.sort_unstable();
    lines.dedup();
    eprintln!(
        "prompt_set[import_preview] {issue_count} loi phan tich, dong hong: {}",
        lines.join(", ")
    );
    let mut params = std::collections::BTreeMap::new();
    params.insert("lines".to_owned(), lines.join(", "));
    IpcError::new(
        "prompt_set.import_malformed",
        crate::core::i18n::MessageKey::PromptSetImportMalformed,
        params,
        false,
    )
}

/// Hình dạng trên dây của [`ImportOutcome`] — Story 4.5.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PromptImportOutcomeWire {
    Inserted,
    Updated,
    Skipped,
}

impl From<ImportOutcome> for PromptImportOutcomeWire {
    fn from(o: ImportOutcome) -> Self {
        match o {
            ImportOutcome::Inserted { .. } => PromptImportOutcomeWire::Inserted,
            ImportOutcome::Updated { .. } => PromptImportOutcomeWire::Updated,
            ImportOutcome::Skipped => PromptImportOutcomeWire::Skipped,
        }
    }
}

/// Xác nhận lượt nhập (nhịp HAI) — `tier` là tầng người dùng chọn Ở MÀN XEM TRƯỚC (không lấy
/// lại từ `pending`, nó không mang tầng — xem doc-comment [`PendingPromptImport`]); `decision`
/// chỉ có ý nghĩa khi tầng đó phân loại `Conflict`.
///
/// 🔴 **Kế hoạch chỉ dọn khỏi `pending` khi giao dịch THÀNH CÔNG** — lỗi giữa chừng (kể cả
/// `ImportStaleConflict`) GIỮ LẠI lô để người dùng thử lại.
///
/// # Lỗi
/// - `global.db` vắng mặt ⇒ `store.open_failed`;
/// - không có lô nào đang treo ⇒ `prompt_set.no_pending_import`;
/// - `tier == Work` mà lô không mang phân loại Tác phẩm (chưa mở Tác phẩm lúc xem trước, hoặc
///   Tác phẩm đã đóng từ đó) ⇒ `prompt_set.work_tier_unavailable` — **0** lượt ghi;
/// - thân/`name` đích đã đổi dưới chân người dùng giữa hai nhịp ⇒
///   `prompt_set.import_stale_conflict`, lô GIỮ LẠI.
pub fn prompt_set_confirm_import(
    global: Option<&Store>,
    open: Option<&OpenWork>,
    pending: &PendingPromptImportState,
    tier: PromptSetTier,
    decision: Option<ConflictDecision>,
) -> Result<PromptImportOutcomeWire, IpcError> {
    let global_store = global.ok_or_else(store_is_missing)?;

    let mut guard = pending.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
    let Some(batch) = guard.as_ref() else {
        return Err(IpcError::from(PromptSetError::NoPendingImport));
    };

    let kind = match tier {
        PromptSetTier::Global => &batch.global_kind,
        PromptSetTier::Work => match &batch.work_kind {
            Some(k) => k,
            None => return Err(IpcError::from(PromptSetError::WorkTierUnavailable)),
        },
    };

    match import_into_tier(global_store, open.map(|w| &w.store), tier, &batch.parsed, kind, decision) {
        Ok(outcome) => {
            *guard = None; // Chi don LO khi giao dich THANH CONG.
            Ok(outcome.into())
        }
        Err(e) => Err(IpcError::from(e)), // Lo GIU LAI -- `guard` khong bi cham.
    }
}

/// Huỷ lô đang treo — **0** lượt ghi, không lỗi kể cả khi không có lô nào.
pub fn prompt_set_cancel_import(pending: &PendingPromptImportState) {
    let mut guard = pending.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
    *guard = None;
}

/// Chín vỏ `#[tauri::command]`. **Không một quy tắc nào sống ở đây.**
///
/// ⚠️ Tên command trên dây LÀ tên hàm — mọi vỏ dưới đây mang ĐÚNG tên hàm thuần ở `super::`,
/// không hậu tố. Chỗ gọi xuống dùng `super::tên_hàm(...)` đủ điều kiện.
pub mod wire {
    use super::{
        ConflictDecision, PendingPromptImportState, PromptImportOutcomeWire, PromptImportPreviewWire,
        PromptSetCreateWire, PromptSetError, PromptSetListWire, PromptSetTier, PromptSetWarningsWire,
        default_export_file_name,
    };
    use crate::commands::project::OpenWorkState;
    use crate::core::i18n::IpcError;
    use crate::core::store::Store;
    use tauri_plugin_dialog::DialogExt as _;

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

    // ── Story 4.5 — hộp thoại chọn tệp nối vào xuất/nhập bộ prompt (AD-48) ──────────

    /// Cùng vai `commands::glossary::wire::work_tier_is_open` — khoá mở rồi đóng NGAY trong
    /// một biểu thức, không một biến `guard` nào sống ra khỏi nó.
    fn work_tier_is_open(app: &tauri::AppHandle) -> bool {
        use tauri::Manager as _;
        app.try_state::<OpenWorkState>()
            .map(|s| s.lock().unwrap_or_else(std::sync::PoisonError::into_inner).is_some())
            .unwrap_or(false)
    }

    /// Tên hiện tại của bộ `(Global, id)` — chỉ để gợi ý tên tệp mặc định của hộp thoại LƯU.
    /// Không đòi `OpenWorkState`, không `.lock()` nào ở đây.
    fn global_row_name(global: Option<&Store>, id: i64) -> Option<String> {
        crate::core::promptset::load_one(global?, id).ok().map(|p| p.name)
    }

    /// Cùng vai `global_row_name`, tầng Tác phẩm — hàm TỰ CHỨA (`.lock()` của nó không sống
    /// ra khỏi hàm này), cùng lý do `work_tier_is_open` ngay trên: gọi nó từ thân
    /// `prompt_set_export` không để lại một `.lock()` nào TRƯỚC hộp thoại trong CHÍNH thân
    /// hàm đó — điều kiện để `the_open_work_mutex_guard_in_the_promptset_dialog_wires_is_
    /// acquired_after_the_blocking_call_not_before` (`config_invariants.rs`) đọc đúng.
    fn work_row_name(app: &tauri::AppHandle, id: i64) -> Option<String> {
        use tauri::Manager as _;
        let work_state = app.try_state::<OpenWorkState>()?;
        let guard = work_state.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
        let open = guard.as_ref()?;
        crate::core::promptset::load_one(&open.store, id).ok().map(|p| p.name)
    }

    /// Vỏ IPC của [`super::prompt_set_export`] — mở hộp thoại LƯU rồi gọi hàm thuần.
    ///
    /// 🔴 **P1 — cùng khuôn `commands::glossary::wire::glossary_export_tier`.** Kiểm `Store`
    /// có mặt và (`tier == Work` ⇒ Tác phẩm đang mở) TRƯỚC khi mở hộp thoại; `OpenWorkState`
    /// khoá LẦN THỨ HAI, MỚI, SAU khi hộp thoại đóng — không tái dùng giá trị đã đọc trước
    /// dialog. Tên gợi ý cho hộp thoại đọc qua `global_row_name`/`work_row_name` — hai hàm TỰ
    /// CHỨA ở trên — nên `.lock()` DUY NHẤT xuất hiện trong chính thân hàm này là lượt khoá
    /// SAU `blocking_save_file()`.
    ///
    /// `#[tauri::command(async)]` — thiếu nó là TREO ứng dụng, cùng lý do đã đo ở
    /// `glossary_export_tier` (`blocking_save_file()` chặn vòng lặp sự kiện mà chính hộp
    /// thoại đang chờ).
    #[tauri::command(async)]
    pub fn prompt_set_export(
        app: tauri::AppHandle,
        tier: PromptSetTier,
        id: i64,
    ) -> Result<Option<String>, IpcError> {
        use tauri::Manager as _;

        let global = app.try_state::<Store>();
        if global.is_none() {
            return Err(IpcError::from(super::store_is_missing()));
        }
        if tier == PromptSetTier::Work && !work_tier_is_open(&app) {
            return Err(IpcError::from(PromptSetError::WorkTierUnavailable));
        }

        let name_hint = match tier {
            PromptSetTier::Global => global_row_name(global.as_deref(), id),
            PromptSetTier::Work => work_row_name(&app, id),
        }
        .unwrap_or_default();

        let Some(picked) = app
            .dialog()
            .file()
            .add_filter("Prompt", &["md"])
            .set_file_name(default_export_file_name(&name_hint))
            .blocking_save_file()
        else {
            return Ok(None);
        };
        let path = picked.into_path().map_err(|_| IpcError::from(PromptSetError::DialogPathInvalid))?;

        let work_state = app.try_state::<OpenWorkState>();
        let guard =
            work_state.as_ref().map(|s| s.lock().unwrap_or_else(std::sync::PoisonError::into_inner));
        let open = guard.as_ref().and_then(|g| g.as_ref());

        super::prompt_set_export(global.as_deref(), open, tier, id, &path)?;
        Ok(Some(path.display().to_string()))
    }

    /// Vỏ IPC của [`super::prompt_set_open_import_preview`] — mở hộp thoại CHỌN rồi gọi hàm
    /// thuần. Nhịp MỘT của lượt nhập. **KHÔNG nhận `tier`** — xem doc-comment
    /// `super::PendingPromptImport`.
    ///
    /// `#[tauri::command(async)]` — cùng lý do `prompt_set_export` (`blocking_pick_file()`
    /// chặn vòng lặp sự kiện).
    #[tauri::command(async)]
    pub fn prompt_set_open_import_preview(
        app: tauri::AppHandle,
    ) -> Result<Option<PromptImportPreviewWire>, IpcError> {
        use tauri::Manager as _;

        let global = app.try_state::<Store>();
        if global.is_none() {
            return Err(IpcError::from(super::store_is_missing()));
        }
        if app.try_state::<PendingPromptImportState>().is_none() {
            eprintln!(
                "prompt_set[import_preview] PendingPromptImportState chua duoc quan ly -- loi cau hinh setup()"
            );
            return Err(IpcError::from(PromptSetError::NoPendingImport));
        }

        let Some(picked) = app.dialog().file().add_filter("Prompt", &["md"]).blocking_pick_file() else {
            return Ok(None);
        };
        let path = picked.into_path().map_err(|_| IpcError::from(PromptSetError::DialogPathInvalid))?;

        let work_state = app.try_state::<OpenWorkState>();
        let guard =
            work_state.as_ref().map(|s| s.lock().unwrap_or_else(std::sync::PoisonError::into_inner));
        let open = guard.as_ref().and_then(|g| g.as_ref());

        let Some(pending) = app.try_state::<PendingPromptImportState>() else {
            return Err(IpcError::from(PromptSetError::NoPendingImport));
        };

        let preview = super::prompt_set_open_import_preview(global.as_deref(), open, pending.inner(), &path)?;
        Ok(Some(preview))
    }

    /// Vỏ IPC của [`super::prompt_set_confirm_import`] — nhịp HAI của lượt nhập. KHÔNG mở hộp
    /// thoại, KHÔNG `(async)` — một giao dịch MỘT hàng là tức thời, khác lượt ghi hàng loạt
    /// của `glossary_confirm_import`.
    #[tauri::command]
    pub fn prompt_set_confirm_import(
        app: tauri::AppHandle,
        tier: PromptSetTier,
        decision: Option<ConflictDecision>,
    ) -> Result<PromptImportOutcomeWire, IpcError> {
        use tauri::Manager as _;

        let global = app.try_state::<Store>();
        let work_state = app.try_state::<OpenWorkState>();
        let guard =
            work_state.as_ref().map(|s| s.lock().unwrap_or_else(std::sync::PoisonError::into_inner));
        let open = guard.as_ref().and_then(|g| g.as_ref());

        let Some(pending) = app.try_state::<PendingPromptImportState>() else {
            return Err(IpcError::from(PromptSetError::NoPendingImport));
        };

        super::prompt_set_confirm_import(global.as_deref(), open, pending.inner(), tier, decision)
    }

    /// Vỏ IPC của [`super::prompt_set_cancel_import`] — huỷ lô đang treo.
    #[tauri::command]
    pub fn prompt_set_cancel_import(app: tauri::AppHandle) -> Result<(), IpcError> {
        use tauri::Manager as _;

        if let Some(pending) = app.try_state::<PendingPromptImportState>() {
            super::prompt_set_cancel_import(pending.inner());
        }
        Ok(())
    }
}
