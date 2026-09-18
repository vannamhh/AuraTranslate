//! Bề mặt IPC của prompt inspector — Story 4.7 (FR71, AD-14, Decision 1/2 của spec).
//!
//! Cùng khuôn `commands::promptset`/`commands::aiconfig`: hàm thuần nhận `Option<&Store>`
//! (tầng Global) **cộng** `Option<&OpenWork>` (tầng Tác phẩm) trước, `#[tauri::command]` chỉ
//! là vỏ mỏng trong [`wire`]. Hai lệnh, đúng Decision 2 (spec 4.7): [`assemble_and_record_prompt`]
//! LẮP RÁP một prompt từ Chương đang mở rồi GHI lại bản ghi DUY NHẤT của phiên;
//! [`read_last_assembled_prompt`] chỉ ĐỌC bản ghi đó — mở màn hình soi prompt KHÔNG BAO GIỜ
//! lắp lại.
//!
//! ─────────────────────────────────────────────────────────────────────────────
//! 🔴 TỆP NÀY LÀ MỘT TRONG HAI SEAM ĐƯỢC DECISION 1 (spec 4.7) MIỄN TRỪ KHỎI AD-13
//! ─────────────────────────────────────────────────────────────────────────────
//! `tests/ai_boundary.rs` là nơi DUY NHẤT ngoài `core/ai/**` được phép gõ
//! `crate::core::ai::…`/`super::ai` trần — nhưng KHÔNG còn bằng cách miễn trừ TRỌN VẸN cả
//! tệp nữa (SỬA finding V4, loop 1): miễn trừ giờ THEO ĐÚNG DÒNG mang tiền tố
//! `crate::core::ai::rag::` (`line_is_the_approved_ai_prompt_import_in_command_file`), và một
//! cổng RIÊNG (`commands_aiprompt_rs_names_nothing_beyond_the_allowed_ai_rag_surface`) kiểm
//! TÊN đứng sau tiền tố đó khớp đúng chín tên trong `ALLOWED_AI_RAG_NAMES_IN_COMMAND_SEAM` —
//! không hơn. Hai cổng đó không phải một tấm thẻ tự do:
//! Decision 1 đòi kiểm soát ①(đối chứng dương, Phase 1) rằng gỡ `core/ai/` cộng đúng seam này
//! vẫn để phần còn lại của cây biên dịch, và ②(bản thân module này) không gọi bất kỳ thứ gì
//! khác của `core::ai` ngoài đúng hai hàm AD-14 đã đóng băng ở `core/ai/rag.rs` (Story 4.6)
//! cộng bảy kiểu mirror-type phải đặt tên để viết `From<…>`.
//! KHÔNG lắp ráp thêm, KHÔNG thay thế biến, KHÔNG quét marker lần hai — mọi quy tắc đó đã
//! sống ở `core::ai::rag`/`core::promptset::vars`, tệp này chỉ GỌI XUỐNG rồi ánh xạ ra dây.
//!
//! ⚠️ Mọi chuỗi trong tệp này viết KHÔNG DẤU — `scripts/check-i18n.mjs` Kiểm A quét
//! `src-tauri/**/*.rs`.

use crate::commands::project::OpenWork;
use crate::commands::promptset::PromptSetTierWire;
use crate::core::ai::rag::{
    GlossaryInjectionStatus, InjectedGlossaryTerm, InjectionLedger, PromptPiece, PromptPieceKind,
    SuppressedGlossaryTerm, TmInjectionStatus, assemble_prompt, gather_glossary_context,
};
use crate::core::glossary::GlossaryTier;
use crate::core::i18n::{IpcError, MessageKey};
use crate::core::promptset::{PromptSetTier, resolve_two_tiers};
use crate::core::scope::ScopeResolver;
use crate::core::store::{Store, StoreError, StoreKind};
use crate::core::tm::SimilarSegment;

/// Kho `global.db` vắng mặt ⇒ lỗi *mở kho* — cùng khuôn `commands::promptset::store_is_missing`.
fn store_is_missing() -> IpcError {
    StoreError::OpenFailed {
        store: StoreKind::Global,
        detail: "the global store was never managed; see lib.rs::open_global_store".to_owned(),
    }
    .into()
}

/// Không có bộ prompt hiệu lực nào cho tên được gửi lên — hoặc vì webview không gửi tên nào
/// (chưa chọn bộ nào), hoặc vì tên gửi lên không khớp bộ nào đã phân giải (I/O Matrix "No set
/// selected — No effective prompt set"). Cùng một sự thật cho cả hai nguyên nhân, nên cùng
/// MỘT khoá — hai khoá cho cùng câu là hai chuỗi phải giữ khớp nhau bằng kỷ luật.
fn no_set_selected() -> IpcError {
    IpcError::new(
        "ai_prompt.no_set_selected",
        MessageKey::AiPromptNoSetSelected,
        std::collections::BTreeMap::new(),
        false,
    )
}

/// `segment_id` không có trong Chương đang mở — I/O Matrix "No Work, or an id the chapter
/// does not hold": ca RIÊNG của id lạ, KHÔNG mượn `WorkNoneOpen` (ca "chưa mở Tác phẩm nào"
/// đã có, tái dùng qua `crate::commands::chapter::no_work_open` — xem chỗ gọi ở
/// [`assemble_and_record_prompt`]) vì đây là một sự thật khác hẳn: Tác phẩm ĐANG MỞ, Chương
/// ĐỌC ĐƯỢC, chỉ riêng `segment_id` này không thuộc nó.
fn segment_not_in_chapter(segment_id: i64, chapter_id: i64) -> IpcError {
    let mut params = std::collections::BTreeMap::new();
    params.insert("segment_id".to_owned(), segment_id.to_string());
    params.insert("chapter_id".to_owned(), chapter_id.to_string());
    IpcError::new(
        "ai_prompt.segment_not_in_chapter",
        MessageKey::AiPromptSegmentNotInChapter,
        params,
        false,
    )
}

// ═════════════════════════════════════════════════════════════════════════════════
// Nhãn tầng Glossary trên dây — `core::glossary::GlossaryTier` chỉ mang `Deserialize`
// (tham số lệnh `glossary.add_term`/`update_term` giải mã trực tiếp từ đó), KHÔNG
// `Serialize`. Cùng khuôn mirror-type `PromptSetTierWire`/`AiConfigTierWire`: một twin
// `serde::Serialize`, chuyển bằng `match` tường minh, KHÔNG `#[serde(rename_all)]` trên
// STRUCT nào cả (đây là enum thuần giá trị, không có trường).
// ═════════════════════════════════════════════════════════════════════════════════

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum GlossaryTierWire {
    Global,
    Work,
}

impl From<GlossaryTier> for GlossaryTierWire {
    fn from(t: GlossaryTier) -> Self {
        match t {
            GlossaryTier::Global => GlossaryTierWire::Global,
            GlossaryTier::Work => GlossaryTierWire::Work,
        }
    }
}

/// Hình dạng OUTPUT của [`InjectedGlossaryTerm`] — mang lại ĐỦ NĂM trường, kể cả `start`/`end`/
/// `tier` (§Always spec 4.7: "the ledger already carries `tier`, `start` and `end` … the
/// screen must not drop them again" — đây chính là chỗ một bản trước có thể đánh rơi chúng).
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct InjectedGlossaryTermWire {
    pub source_term: String,
    pub translation: String,
    pub start: usize,
    pub end: usize,
    pub tier: GlossaryTierWire,
}

impl From<InjectedGlossaryTerm> for InjectedGlossaryTermWire {
    fn from(t: InjectedGlossaryTerm) -> Self {
        Self {
            source_term: t.source_term,
            translation: t.translation,
            start: t.start,
            end: t.end,
            tier: t.tier.into(),
        }
    }
}

/// Hình dạng OUTPUT của [`SuppressedGlossaryTerm`] — cùng năm trường, cùng lý do.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct SuppressedGlossaryTermWire {
    pub source_term: String,
    pub translation: String,
    pub start: usize,
    pub end: usize,
    pub tier: GlossaryTierWire,
}

impl From<SuppressedGlossaryTerm> for SuppressedGlossaryTermWire {
    fn from(t: SuppressedGlossaryTerm) -> Self {
        Self {
            source_term: t.source_term,
            translation: t.translation,
            start: t.start,
            end: t.end,
            tier: t.tier.into(),
        }
    }
}

/// Hình dạng OUTPUT của [`GlossaryInjectionStatus`] — BA giá trị không được collapse
/// (§Always spec 4.7). `kind` là TAG, cùng khuôn hand-rolled
/// `commands::promptset::PromptImportTierPreviewWire::kind` (một chuỗi `&'static str` phân
/// biệt, KHÔNG một `#[serde(tag = …)]` của serde — tệp này không đúc thêm quy ước tag mới).
/// `injected`/`suppressed_by_pending_overlap` LUÔN đi cùng nhau: cả hai `Some` khi
/// `kind == "asked"`, cả hai `None` khi `kind == "not_asked"` — không có tổ hợp thứ ba.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct GlossaryInjectionStatusWire {
    pub kind: &'static str,
    pub injected: Option<Vec<InjectedGlossaryTermWire>>,
    pub suppressed_by_pending_overlap: Option<Vec<SuppressedGlossaryTermWire>>,
}

impl From<GlossaryInjectionStatus> for GlossaryInjectionStatusWire {
    fn from(status: GlossaryInjectionStatus) -> Self {
        match status {
            GlossaryInjectionStatus::NotAsked => {
                Self { kind: "not_asked", injected: None, suppressed_by_pending_overlap: None }
            }
            GlossaryInjectionStatus::Asked { injected, suppressed_by_pending_overlap } => Self {
                kind: "asked",
                injected: Some(injected.into_iter().map(Into::into).collect()),
                suppressed_by_pending_overlap: Some(
                    suppressed_by_pending_overlap.into_iter().map(Into::into).collect(),
                ),
            },
        }
    }
}

/// Hình dạng OUTPUT của [`SimilarSegment`] — Epic 7 sẽ là caller thật đầu tiên; story này
/// không bao giờ tạo được biến thể `Searched` (tham số `tm` của [`assemble_prompt`] luôn
/// `None` ở đây), nhưng kiểu wire phải tồn tại để `From<TmInjectionStatus>` không sót nhánh.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct SimilarSegmentWire {
    pub source_text: String,
    pub target_text: String,
}

impl From<SimilarSegment> for SimilarSegmentWire {
    fn from(s: SimilarSegment) -> Self {
        Self { source_text: s.source_text, target_text: s.target_text }
    }
}

/// Hình dạng OUTPUT của [`TmInjectionStatus`] — cùng khuôn tag `kind` với
/// [`GlossaryInjectionStatusWire`]: `"not_built_yet"` (TM chưa tồn tại tới Epic 7) tách hẳn
/// khỏi `"searched"` (kể cả lát cắt rỗng).
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct TmInjectionStatusWire {
    pub kind: &'static str,
    pub similar_segments: Option<Vec<SimilarSegmentWire>>,
}

impl From<TmInjectionStatus> for TmInjectionStatusWire {
    fn from(status: TmInjectionStatus) -> Self {
        match status {
            TmInjectionStatus::NotBuiltYet => Self { kind: "not_built_yet", similar_segments: None },
            TmInjectionStatus::Searched(segments) => Self {
                kind: "searched",
                similar_segments: Some(segments.into_iter().map(Into::into).collect()),
            },
        }
    }
}

/// Hình dạng OUTPUT của [`PromptPieceKind`] — cùng khuôn tag `kind` hand-rolled với
/// [`GlossaryInjectionStatusWire`]/[`TmInjectionStatusWire`]: `"authored"` | `"glossary"` |
/// `"source_segment"` | `"tm"`. **SỬA loop 2, finding P7** — `"source_segment"` là biến thể
/// MỚI (trước đây câu nguồn ánh xạ vào `"authored"`); xem doc-comment
/// [`core::ai::rag::PromptPieceKind::SourceSegment`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PromptPieceKindWire {
    Authored,
    Glossary,
    SourceSegment,
    Tm,
}

impl From<PromptPieceKind> for PromptPieceKindWire {
    fn from(k: PromptPieceKind) -> Self {
        match k {
            PromptPieceKind::Authored => PromptPieceKindWire::Authored,
            PromptPieceKind::Glossary => PromptPieceKindWire::Glossary,
            PromptPieceKind::SourceSegment => PromptPieceKindWire::SourceSegment,
            PromptPieceKind::Tm => PromptPieceKindWire::Tm,
        }
    }
}

/// Hình dạng OUTPUT của [`PromptPiece`] — Story 4.7 loop 1, finding B1. Màn hình vẽ TỪNG mảnh
/// bằng `kind` của nó thay vì tô cả `prompt` cùng một màu; nối `.text` của toàn bộ `pieces`
/// theo đúng thứ tự phải cho lại `prompt` TỪNG BYTE (đối chứng ở `ai_prompt_contract.rs` và ở
/// `tests/frontend/aiPromptInspector.test.ts`).
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct PromptPieceWire {
    pub kind: PromptPieceKindWire,
    pub text: String,
}

impl From<PromptPiece> for PromptPieceWire {
    fn from(p: PromptPiece) -> Self {
        Self { kind: p.kind.into(), text: p.text }
    }
}

/// Hình dạng OUTPUT của [`InjectionLedger`] — không trường nào bị bỏ (§Always spec 4.7: "the
/// ledger already carries … the screen must not drop them again").
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct InjectionLedgerWire {
    pub glossary: GlossaryInjectionStatusWire,
    pub tm: TmInjectionStatusWire,
    pub unknown_markers: Vec<String>,
    pub source_segment_missing: bool,
    pub pieces: Vec<PromptPieceWire>,
}

impl From<InjectionLedger> for InjectionLedgerWire {
    fn from(l: InjectionLedger) -> Self {
        Self {
            glossary: l.glossary.into(),
            tm: l.tm.into(),
            unknown_markers: l.unknown_markers,
            source_segment_missing: l.source_segment_missing,
            pieces: l.pieces.into_iter().map(Into::into).collect(),
        }
    }
}

// ═════════════════════════════════════════════════════════════════════════════════
// Bản ghi DUY NHẤT của phiên (Decision 2 + §Never spec 4.7: "no new table … the shape
// `PendingPromptImportState` already established")
// ═════════════════════════════════════════════════════════════════════════════════

/// Kết quả của MỘT lượt [`assemble_and_record_prompt`] — mang chuỗi prompt, TOÀN VẸN ledger
/// (`core::ai::rag::InjectionLedger` thật, không phải mirror — chỉ chuyển sang wire lúc ra
/// dây, xem [`AssembledPromptWire`]) và ĐỊNH DANH của thứ đã tạo ra nó (§Always spec 4.7:
/// "The record carries the identity of what produced it — which segment, which prompt set,
/// which tier — so a record from an earlier segment cannot be read as describing the segment
/// now focused").
#[derive(Debug, Clone)]
pub struct AssembledPromptRecord {
    pub prompt: String,
    pub ledger: InjectionLedger,
    pub segment_id: i64,
    pub chapter_id: i64,
    pub prompt_set_name: String,
    pub prompt_set_tier: PromptSetTier,
}

/// Kiểu state Tauri quản lý cho [`AssembledPromptRecord`] — `None` == chưa lắp prompt nào
/// trong phiên này (I/O Matrix "Nothing recorded yet … this is a state, not an error"). Cùng
/// khuôn `commands::promptset::PendingPromptImportState` (`Mutex<Option<T>>` trần, không một
/// bọc riêng — `commands/project/mod.rs::OpenWorkState` là tiền lệ của chính hình dạng này).
pub type LastAssembledPromptState = std::sync::Mutex<Option<AssembledPromptRecord>>;

/// Hình dạng OUTPUT của [`AssembledPromptRecord`] — thứ cả hai lệnh trả về. `prompt_set_tier`
/// tái dùng [`PromptSetTierWire`] của `commands::promptset` (đúng tầng, đúng ý nghĩa — không
/// đúc một bản chép Global/Work thứ ba khi bộ prompt đã có sẵn một cái đúng của chính nó).
#[derive(Debug, Clone, serde::Serialize)]
pub struct AssembledPromptWire {
    pub prompt: String,
    pub segment_id: i64,
    pub chapter_id: i64,
    pub prompt_set_name: String,
    pub prompt_set_tier: PromptSetTierWire,
    pub ledger: InjectionLedgerWire,
}

impl From<AssembledPromptRecord> for AssembledPromptWire {
    fn from(r: AssembledPromptRecord) -> Self {
        Self {
            prompt: r.prompt,
            segment_id: r.segment_id,
            chapter_id: r.chapter_id,
            prompt_set_name: r.prompt_set_name,
            prompt_set_tier: r.prompt_set_tier.into(),
            ledger: r.ledger.into(),
        }
    }
}

/// LẮP RÁP một prompt cho `segment_id` bằng bộ prompt hiệu lực tên `prompt_set_name`, rồi GHI
/// bản ghi DUY NHẤT của phiên (đè bản ghi cũ, nếu có) — **hàm thuần, đây là thứ test gọi**.
/// Đúng Decision 2 (spec 4.7): đây là nhịp DUY NHẤT lắp ráp; [`read_last_assembled_prompt`]
/// không bao giờ gọi lại các bước dưới đây.
///
/// Thứ tự kiểm: bộ prompt hiệu lực TRƯỚC (không cần Tác phẩm đang mở để phân giải hai tầng,
/// cùng `prompt_set_list`), rồi Tác phẩm/Chương/segment SAU (cần `OpenWork` thật để đọc câu
/// nguồn). Đây là một lựa chọn thứ tự khi CẢ HAI đều thiếu, không phải một hợp đồng —
/// I/O Matrix của spec không xếp hạng ưu tiên hai ca.
///
/// # Lỗi
/// - `global.db` vắng mặt ⇒ `store.open_failed`;
/// - đường đọc Store trượt ⇒ lỗi kho, truyền từ [`resolve_two_tiers`]/[`gather_glossary_context`];
/// - `prompt_set_name` là `None`/rỗng, hoặc không khớp bộ nào đã phân giải ⇒
///   `ai_prompt.no_set_selected` — I/O Matrix "No set selected";
/// - chưa có Tác phẩm nào đang mở ⇒ `work.none_open` (tái dùng
///   [`crate::commands::chapter::no_work_open`], KHÔNG một khoá thứ hai cho cùng câu);
/// - `segment_id` không có trong Chương đang mở ⇒ `ai_prompt.segment_not_in_chapter` —
///   I/O Matrix "an id the chapter does not hold", khoá RIÊNG của ca này (khác `WorkNoneOpen`:
///   Tác phẩm ĐANG MỞ, chỉ riêng id này lạ).
pub fn assemble_and_record_prompt(
    global: Option<&Store>,
    open: Option<&OpenWork>,
    record: &LastAssembledPromptState,
    prompt_set_name: Option<&str>,
    segment_id: i64,
) -> Result<AssembledPromptWire, IpcError> {
    let global_store = global.ok_or_else(store_is_missing)?;
    let resolver = open.map(|w| w.scope.clone()).unwrap_or_else(ScopeResolver::global_only);
    let work_store = open.map(|w| &w.store);
    let mut resolved = resolve_two_tiers(&resolver, global_store, work_store)?;

    let Some(name) = prompt_set_name.map(str::trim).filter(|n| !n.is_empty()) else {
        return Err(no_set_selected());
    };
    let Some(set) = resolved.remove(name) else {
        return Err(no_set_selected());
    };

    let open_work = open.ok_or_else(crate::commands::chapter::no_work_open)?;
    let chapter = crate::commands::segment::read_open_chapter_segments(Some(open_work))?;
    let Some(row) = chapter.segments.iter().find(|s| s.id == segment_id) else {
        return Err(segment_not_in_chapter(segment_id, chapter.chapter_id));
    };
    let sentence = row.source_text.as_str();
    let source_lang = open_work.meta.source_lang.as_str();

    let glossary = gather_glossary_context(
        &set.body,
        &resolver,
        global_store,
        work_store,
        source_lang,
        sentence,
    )?;
    let (prompt, ledger) = assemble_prompt(&set.body, sentence, glossary, None);

    let record_value = AssembledPromptRecord {
        prompt,
        ledger,
        segment_id,
        chapter_id: chapter.chapter_id,
        prompt_set_name: set.name,
        prompt_set_tier: set.tier,
    };

    let mut guard = record.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
    *guard = Some(record_value.clone());
    drop(guard);

    Ok(record_value.into())
}

/// ĐỌC lại bản ghi hiện tại của phiên — KHÔNG lắp ráp gì, KHÔNG chạm `Store`/`OpenWork`
/// (Decision 2: nhịp mở màn hình soi prompt). **Hàm thuần, đây là thứ test gọi.**
///
/// `None` ⇔ chưa có lượt [`assemble_and_record_prompt`] nào chạy trong phiên này — I/O Matrix
/// "Nothing recorded yet … this is a state, not an error" (§Always spec 4.7: "no record yet
/// this session" tách hẳn khỏi "a record whose prompt is empty" — một bản ghi RỖNG vẫn là
/// `Some`, mang `prompt: String::new()`, không phải `None`).
pub fn read_last_assembled_prompt(record: &LastAssembledPromptState) -> Option<AssembledPromptWire> {
    let guard = record.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
    guard.clone().map(AssembledPromptWire::from)
}

/// Xoá bản ghi phiên khi Tác phẩm đang mở ĐÓNG — `segment_id`/`chapter_id` là khoá hàng của
/// CHÍNH `project.db` sắp đóng; mở một Tác phẩm KHÁC có thể trùng số đó NGẪU NHIÊN (hai
/// `.atproj` khác nhau đều đánh số lại từ 1), nên để bản ghi sống qua lượt mở sẽ đọc sai
/// "đúng Chương/segment này" cho một Tác phẩm nó chưa từng thấy.
///
/// ⚠️ **SỬA 2026-09-18, lượt rà soát build** — trước bản sửa, ba dòng này nằm THẲNG trong
/// `lib.rs::close_open_work` (một `fn` KHÔNG `pub`, không cách nào gọi được từ
/// `src-tauri/tests/*.rs`) và không một ca nào canh chúng. Cùng khuôn hai người láng giềng
/// đã có tiền lệ — `commands::promptset::clear_pending_prompt_import_work_tier` và
/// `commands::glossary::clear_pending_import_for_tier`, cả hai đều là `pub fn` RIÊNG, được
/// `lib.rs::close_open_work` GỌI XUỐNG chứ không viết tay tại chỗ, và cả hai đều có ca đơn vị
/// trực tiếp (`glossary_import_dialog_contract.rs`) — hàm này giờ theo đúng khuôn đó, để
/// [`the_record_is_cleared_when_the_open_work_closes`] (`ai_prompt_contract.rs`) canh được
/// nó mà không cần một hạ tầng test Tauri (`tauri::test`/`MockRuntime`) mà kho này chưa có
/// (xem `ipc_contract.rs:907`/`project_contract.rs:1075`).
pub fn clear_last_assembled_prompt_on_work_close(record: &LastAssembledPromptState) {
    let mut guard = record.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
    *guard = None;
}

/// Hai vỏ `#[tauri::command]`. **Không một quy tắc nào sống ở đây.**
pub mod wire {
    use super::{AssembledPromptWire, LastAssembledPromptState};
    use crate::commands::project::OpenWorkState;
    use crate::core::i18n::IpcError;
    use crate::core::store::Store;

    /// `try_state`, không `state()` — cùng lý do mọi vỏ khác của kho.
    #[tauri::command]
    pub fn ai_prompt_assemble(
        app: tauri::AppHandle,
        prompt_set_name: Option<String>,
        segment_id: i64,
    ) -> Result<AssembledPromptWire, IpcError> {
        use tauri::Manager as _;

        let global = app.try_state::<Store>();
        let work_state = app.try_state::<OpenWorkState>();
        let guard =
            work_state.as_ref().map(|s| s.lock().unwrap_or_else(std::sync::PoisonError::into_inner));
        let open = guard.as_ref().and_then(|g| g.as_ref());

        let Some(record_state) = app.try_state::<LastAssembledPromptState>() else {
            // Khong bao gio xay ra tren duong san pham: `LastAssembledPromptState` duoc
            // `app.manage` VO DIEU KIEN o `open_work_slot` (lib.rs), cung khuon
            // `OpenWorkState`. Nhanh nay chi con lai vi mot loi cau hinh setup() -- cung
            // khuon `prompt_set_open_import_preview` da ghi cho `PendingPromptImportState`.
            eprintln!(
                "ai_prompt[assemble] LastAssembledPromptState chua duoc quan ly -- loi cau hinh setup()"
            );
            return Err(IpcError::new(
                "ai_prompt.record_state_missing",
                crate::core::i18n::MessageKey::Unknown,
                std::collections::BTreeMap::new(),
                false,
            ));
        };

        super::assemble_and_record_prompt(
            global.as_deref(),
            open,
            record_state.inner(),
            prompt_set_name.as_deref(),
            segment_id,
        )
    }

    /// Không `Result`: "chưa có bản ghi nào" là một TRẠNG THÁI, không một lỗi (I/O Matrix).
    /// `LastAssembledPromptState` chưa được quản lý đọc y hệt "chưa có bản ghi nào" — cùng
    /// khuôn `prompt_set_list` đọc `OpenWorkState` vắng mặt y hệt "chưa mở Tác phẩm nào".
    #[tauri::command]
    pub fn ai_prompt_read_record(app: tauri::AppHandle) -> Option<AssembledPromptWire> {
        use tauri::Manager as _;

        let record_state = app.try_state::<LastAssembledPromptState>()?;
        super::read_last_assembled_prompt(record_state.inner())
    }
}
