//! Bộ prompt theo thể loại — Story 4.4, FR69, `core/scope/kinds.rs`
//! (`ScopeKind::Prompt => "prompt" : Semantics::Override`).
//!
//! ─────────────────────────────────────────────────────────────────────────────
//! 🔴 GHI ĐÈ CẢ BỘ, THEO TÊN — KHÔNG PHẢI THEO TỪNG TRƯỜNG NHƯ `core::aiconfig`
//! ─────────────────────────────────────────────────────────────────────────────
//! Quyết định #1 của spec 4.4 (ghi vào doc-comment của `ScopeKind::Prompt`): một bộ Tác
//! phẩm trùng TÊN với một bộ Toàn cục THAY THẾ TRỌN bộ đó, mục Toàn cục vẫn hiện nhưng
//! đánh dấu bị che. Vì thế bảng lưu khoá theo TÊN (`(name, body)`), và
//! [`store::resolve_two_tiers`] phân giải qua `ScopeResolver::apply_override` đúng khuôn
//! `core::glossary::store::load_tier` + `apply_override` — KHÔNG khuôn `core::aiconfig`
//! (bảng đó khoá theo TÊN TRƯỜNG, một khái niệm không tồn tại ở đây: một bộ không có
//! trường con nào để mà ghi đè riêng lẻ).
//!
//! ─────────────────────────────────────────────────────────────────────────────
//! MỘT BỘ LÀ TÊN + THÂN, KHÔNG GÌ KHÁC — Quyết định #4
//! ─────────────────────────────────────────────────────────────────────────────
//! Không trường ngôn ngữ, không trường "áp cho mọi segment / lời thoại / tả cảnh" mà
//! mockup vẽ — không FR/AC nào đòi chúng, và không ai chuẩn hoá được chúng có nghĩa gì.
//! Ghi nợ ở `deferred-work.md`, chủ Story 4.5 (cùng lúc với định dạng xuất/nhập).
//!
//! ─────────────────────────────────────────────────────────────────────────────
//! MODULE NÀY KHÔNG LẮP RÁP PROMPT, KHÔNG THAY THẾ BIẾN
//! ─────────────────────────────────────────────────────────────────────────────
//! Lưu và hiển thị văn bản MANG dấu ngoặc `{{...}}` — nó không bao giờ mở rộng một dấu
//! ngoặc thành nội dung thật. Đó là `RagInjector`, một hàm THUẦN, Story 4.6 (AD-14).
//!
//! ─────────────────────────────────────────────────────────────────────────────
//! KHÔNG DÙNG `ScopeError` LÀM LỖI DOMAIN CỦA MODULE NÀY
//! ─────────────────────────────────────────────────────────────────────────────
//! `ScopeError` là lỗi riêng của Glossary theo quyết định đã ghi (`deferred-work.md §*Deferred from: 3-2-bang-cho-ung-vien-tach-han-khoi-glossary (rà soát ba lớp, 2026-08-20)*`).
//! [`store::PromptSetError`] là kiểu lỗi RIÊNG của domain này, cùng khuôn
//! `core::aiconfig::store::AiConfigKeyError`/`AiConfigStoreError`.
//!
//! ─────────────────────────────────────────────────────────────────────────────
//! MODULE NÀY KHÔNG GÕ TÊN `ScopeKind`/`Semantics`/`Tier` — cùng luật mọi module miền khác
//! ─────────────────────────────────────────────────────────────────────────────
//! `tests/scope_boundary.rs` canh mệnh đề đó cho `Semantics`/`ScopeKind`; `PromptSetTier`
//! ở đây là kiểu RIÊNG (không tái dùng `core::scope::Tier`), cùng lý do
//! `core::aiconfig::AiConfigTier`/`core::glossary::GlossaryTier` đã có kiểu riêng.

pub mod exchange;
pub mod exchange_io;
pub mod store;
pub mod vars;

pub use store::{
    ImportOutcome, PromptSetError, ResolvedPromptSet, create, delete, import_into_tier,
    load_one, load_prompt_set_tier, rename, resolve_two_tiers, update_body,
};
pub use vars::{MarkerWarnings, PromptVariable, scan_markers};

use std::fmt;

/// Một hàng `prompt_set` đã nạp — kiểu THUẦN, không mang SQL (SQL sống ở [`store`]).
///
/// Quyết định #4: đúng hai trường mang nghĩa. `name` lặp lại khoá của `BTreeMap` mà
/// [`store::load_prompt_set_tier`] trả về — cùng thừa số [`crate::core::glossary::GlossaryEntry`]
/// giữ với `source_term`, giữ cho một giá trị đã tách khỏi map (ví dụ đi qua
/// [`ResolvedPromptSet`]) vẫn tự mang tên của chính nó.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PromptSet {
    /// Khoá hàng SQL (`prompt_set.id`) — chỉ duy nhất TRONG một `Store`, cùng lý do
    /// `core::glossary::GlossaryEntry::id`. 🔵 THÊM Phase 2: [`store::create`]/
    /// [`store::rename`]/[`store::update_body`]/[`store::delete`] đều nhận `id`, nhưng
    /// `store::load_prompt_set_tier` trước bản vá này không đọc cột `id` — lớp lệnh
    /// (`commands/promptset.rs`) không thể liệt kê rồi cho người dùng sửa/xoá một bộ, vì
    /// không có gì để gửi lại làm định danh. Không có hàng SQL nào cần sửa: `id` đã tồn
    /// tại trong bảng từ Task 3 của Phase 1, chỉ là không được `SELECT`.
    pub id: i64,
    /// Tên bộ — khoá ghi đè hai tầng (Quyết định #1). `UNIQUE` trong bảng
    /// (`idx_prompt_set_name`), rào rỗng 25 điểm mã `White_Space`.
    pub name: String,
    /// Thân prompt — văn bản tự do, có thể mang `{{...}}`. Không bao giờ bị từ chối vì nội
    /// dung của nó (§Always spec 4.4) — chỉ [`vars::scan_markers`] soi nó, không từ chối.
    pub body: String,
}

/// Nhãn tầng của MỘT bộ đã phân giải — kiểu RIÊNG, không tái dùng `core::scope::Tier`,
/// cùng lý do `core::aiconfig::AiConfigTier`/`core::glossary::GlossaryTier`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Deserialize)]
pub enum PromptSetTier {
    #[serde(rename = "global")]
    Global,
    #[serde(rename = "work")]
    Work,
}

impl PromptSetTier {
    /// Định danh máy đọc — thứ đi trên dây. Không phải nhãn hiển thị (AD-21, NFR16).
    pub const fn as_str(self) -> &'static str {
        match self {
            PromptSetTier::Global => "global",
            PromptSetTier::Work => "work",
        }
    }

    /// Phân giải một giá trị đến từ `core::scope::Tier::as_str()` — chỗ DUY NHẤT module
    /// này đọc chuỗi đó, cùng lý do `core::aiconfig::AiConfigTier::from_wire`.
    pub fn from_wire(raw: &str) -> Option<Self> {
        match raw {
            "global" => Some(PromptSetTier::Global),
            "work" => Some(PromptSetTier::Work),
            _ => None,
        }
    }
}

impl fmt::Display for PromptSetTier {
    /// KHÔNG DẤU — chẩn đoán cho log, không phải văn bản hiển thị (NFR16).
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}
