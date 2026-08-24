//! `GlossaryEntry` thuần + `Category`/`TermOrigin` — Story 3.1, AD-36, FR46/FR47.
//!
//! ─────────────────────────────────────────────────────────────────────────────
//! 🔴 `is_confirmed()` LÀ VỊ TỪ DUY NHẤT ĐỊNH NGHĨA "ĐÃ CHỐT" — không cột `status`
//! ─────────────────────────────────────────────────────────────────────────────
//! `glossary_entry.translation IS NULL` **là** trạng thái *chờ chốt* (`schema.rs::
//! GLOSSARY_ENTRY_DDL`). Không có một cột `status` song song ghi lại cùng sự thật — hai
//! dữ kiện nói cùng một chuyện thì chúng lệch được, và lệch trong im lặng, đúng ca AD-36
//! sinh ra để chặn: *"đã chốt mà bản dịch rỗng"*. [`GlossaryEntry::is_confirmed`] đọc
//! đúng và chỉ đúng `translation.is_some()` — không nơi nào khác trong `core/glossary/**`
//! được phép tự hỏi câu này bằng một biểu thức khác.
//!
//! ⚠️ Mọi chuỗi trong `src-tauri/src/**` viết KHÔNG DẤU; doc-comment có dấu là hợp lệ.

use std::fmt;

/// Phân loại thuật ngữ (FR46). Bốn giá trị, cưỡng chế lại ở `CHECK (category IN (…))` của
/// `GLOSSARY_ENTRY_DDL` — cột SQL và kiểu Rust không được phép trôi khỏi nhau.
///
/// 🔵 **THÊM 2026-08-20 (Story 3.3) — `serde::Deserialize`, `rename` TỪNG biến thể.** Bề
/// mặt IPC đầu tiên của `core/glossary/**` (`commands::glossary::wire`) nhận `category` như
/// một tham số lệnh; Tauri tự giải mã JSON bằng `serde` TRƯỚC khi hàm thuần chạy — cùng
/// khuôn `commands::project::ChapterDirection`. `#[serde(rename = …)]` từng biến thể (không
/// `#[serde(rename_all = …)]`) vì đó là tiền lệ đã chọn ở `ChapterDirection`: đúng cho hôm
/// nay nhưng tường minh hơn một quy tắc chuyển đổi ngầm cho biến thể tương lai.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Deserialize)]
pub enum Category {
    /// Tên người.
    #[serde(rename = "person")]
    Person,
    /// Địa danh.
    #[serde(rename = "place")]
    Place,
    /// Thuật ngữ chuyên ngành.
    #[serde(rename = "domain_term")]
    DomainTerm,
    /// Còn lại.
    #[serde(rename = "other")]
    Other,
}

impl Category {
    /// Định danh máy đọc — khớp NGUYÊN VĂN giá trị trong `CHECK` của `GLOSSARY_ENTRY_DDL`.
    /// Không phải nhãn hiển thị (AD-21, NFR16).
    pub const fn as_str(self) -> &'static str {
        match self {
            Category::Person => "person",
            Category::Place => "place",
            Category::DomainTerm => "domain_term",
            Category::Other => "other",
        }
    }

    /// Phân giải một giá trị đến từ đĩa. `None` cho mọi chuỗi không khớp — không đoán.
    ///
    /// ⚠️ Trên đường đọc `load_tier`, một chuỗi lạ ở đây là bằng chứng đĩa đã trôi khỏi
    /// `CHECK` (dữ liệu do một bản ứng dụng cũ/hỏng ghi ra) — CHECK đã đóng cửa này ở mọi
    /// lượt `INSERT`/`UPDATE` của chính module này, nên `None` ở đây là một ca không nên
    /// xảy ra, không phải một nhánh nghiệp vụ bình thường.
    pub fn from_wire(raw: &str) -> Option<Self> {
        match raw {
            "person" => Some(Category::Person),
            "place" => Some(Category::Place),
            "domain_term" => Some(Category::DomainTerm),
            "other" => Some(Category::Other),
            _ => None,
        }
    }
}

impl fmt::Display for Category {
    /// KHÔNG DẤU — chẩn đoán cho log, không phải văn bản hiển thị (NFR16).
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Xuất xứ mục Glossary (FR47) — **không** lẫn với `segment.translation_origin` hay
/// xuất xứ tài liệu nguồn; xem doc-comment của `GLOSSARY_ENTRY_DDL` cho lý do đặt tên
/// `term_origin` thay vì `origin` trần.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TermOrigin {
    /// Người dùng gõ tay.
    Manual,
    /// Quét khi nhập tài liệu (Story 3.5) rồi được duyệt qua bảng chờ (AD-20).
    ImportScan,
    /// Thu hoạch từ bản review (Epic 8) rồi được duyệt qua bảng chờ (AD-20).
    ReviewHarvest,
}

impl TermOrigin {
    /// Định danh máy đọc — khớp NGUYÊN VĂN giá trị trong `CHECK` của `GLOSSARY_ENTRY_DDL`.
    pub const fn as_str(self) -> &'static str {
        match self {
            TermOrigin::Manual => "manual",
            TermOrigin::ImportScan => "import_scan",
            TermOrigin::ReviewHarvest => "review_harvest",
        }
    }

    /// Phân giải một giá trị đến từ đĩa. `None` cho mọi chuỗi không khớp — cùng lý do
    /// [`Category::from_wire`].
    pub fn from_wire(raw: &str) -> Option<Self> {
        match raw {
            "manual" => Some(TermOrigin::Manual),
            "import_scan" => Some(TermOrigin::ImportScan),
            "review_harvest" => Some(TermOrigin::ReviewHarvest),
            _ => None,
        }
    }
}

impl fmt::Display for TermOrigin {
    /// KHÔNG DẤU — chẩn đoán cho log, không phải văn bản hiển thị (NFR16).
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Tầng mà một mục Glossary **thắng** sau khi phân giải hai tầng (AD-18) — Story 3.3.
///
/// ─────────────────────────────────────────────────────────────────────────────
/// 🔴 VÌ SAO MỘT KIỂU MỚI, KHÔNG TÁI DÙNG `core::scope::Tier`
/// ─────────────────────────────────────────────────────────────────────────────
/// `core::scope::Tier` đã có hai biến thể y hệt (`Global`/`Work`) và không bị
/// `scope_boundary.rs::FORBIDDEN_OUTSIDE_SCOPE` cấm gọi từ ngoài `core/scope/**`. Nhưng
/// giá trị này còn đi tiếp một chặng nữa mà `Tier` không đi: nó là DỮ LIỆU TRÊN DÂY (dải
/// "Thêm thuật ngữ" ghim chế độ SỬA theo đúng giá trị này, `src/config/glossary.ts` đọc nó
/// như một chuỗi `"global"`/`"work"`). Một kiểu riêng của `core::glossary` giữ cho hình
/// dạng dây của module này KHÔNG phụ thuộc vào một quyết định biểu diễn của
/// `core::scope` (đổi cách `Tier::as_str()` viết ra sẽ đổi luôn dây IPC của Glossary nếu
/// dùng chung) — cùng lý lẽ mà `Category`/`TermOrigin` đã có kiểu riêng thay vì mượn một
/// enum chuỗi tự do.
///
/// `id` của [`GlossaryEntry`] chỉ DUY NHẤT **trong một `Store`** (`deferred-work.md:5352`)
/// — hai `Store` khác nhau (`global.db`/`project.db` của Tác phẩm đang mở) có thể cùng
/// đánh số `id = 7` cho hai hàng khác hẳn nhau. Một `id` trần đi qua dây IPC không đủ để
/// sửa lại đúng hàng; cặp `(GlossaryTier, id)` mới đủ — xem
/// [`super::store::resolve_term_for_quick_add`] và [`super::store::update_manual_term`].
/// 🔵 **THÊM 2026-08-20 — `serde::Deserialize`, cùng lý do và cùng khuôn `Category` ngay
/// trên: tham số lệnh `tier` của `glossary.add_term`/`glossary.update_term` được Tauri
/// giải mã trực tiếp thành kiểu này.**
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Deserialize)]
pub enum GlossaryTier {
    /// `global.db` — dùng cho mọi Tác phẩm.
    #[serde(rename = "global")]
    Global,
    /// `project.db` của Tác phẩm đang mở.
    #[serde(rename = "work")]
    Work,
}

impl GlossaryTier {
    /// Định danh máy đọc — thứ đi trên dây IPC. Không phải nhãn hiển thị (AD-21, NFR16).
    pub const fn as_str(self) -> &'static str {
        match self {
            GlossaryTier::Global => "global",
            GlossaryTier::Work => "work",
        }
    }

    /// 🔵 **SỬA 2026-08-20 (Ice bắt) — mệnh đề trước SAI về chỗ gọi.** Bản trước viết hàm
    /// này "phân giải một giá trị đến từ dây (đối số IPC)" — không đúng: tham số `tier` của
    /// `glossary.add_term`/`glossary.update_term` được **`serde` giải mã trực tiếp qua
    /// `#[serde(rename = …)]`** ở khai báo enum trên, KHÔNG đi qua hàm này. `from_wire` vì
    /// vậy có **0 chỗ gọi sản phẩm** — cùng đúng thứ `TermOrigin`/`Category` giữ (đọc lại từ
    /// đĩa), nhưng `GlossaryTier` **không có cột đĩa nào** để đọc lại từ đó (tầng là DẪN
    /// XUẤT từ `Resolved::tier()` lúc phân giải, xem [`super::store::resolve_term_for_quick_add`]),
    /// nên hàm này thật ra không có chỗ gọi sản phẩm nào cả.
    ///
    /// Chỗ gọi DUY NHẤT hôm nay là
    /// `glossary_contract.rs::category_and_glossary_tier_wire_strings_agree_between_as_str_and_serde_rename`
    /// — một phép kiểm chéo TƯỜNG MINH rằng `as_str()`/`from_wire()` (bản chép này) và
    /// `#[serde(rename)]` (bản chép serde) không lệch nhau. Test LÀ chỗ gọi hợp lệ ở đây,
    /// không phải một dấu hiệu hàm chết — giữ hàm này công khai để cổng đó dựng được.
    pub fn from_wire(raw: &str) -> Option<Self> {
        match raw {
            "global" => Some(GlossaryTier::Global),
            "work" => Some(GlossaryTier::Work),
            _ => None,
        }
    }
}

impl fmt::Display for GlossaryTier {
    /// KHÔNG DẤU — chẩn đoán cho log, không phải văn bản hiển thị (NFR16).
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Một hàng `glossary_entry` đã nạp — kiểu THUẦN, không mang SQL nào (SQL sống ở
/// [`super::store`]).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GlossaryEntry {
    /// Khoá hàng.
    pub id: i64,
    /// Thuật ngữ nguồn — `UNIQUE` trong bảng (`idx_glossary_entry_source_term`).
    pub source_term: String,
    /// `None` == *chờ chốt* (FR114). Đây là trường DUY NHẤT [`Self::is_confirmed`] đọc.
    pub translation: Option<String>,
    /// Ghi chú. Vắng mặt và rỗng là CÙNG một điều — khác `translation`, xem doc-comment
    /// của `GLOSSARY_ENTRY_DDL`.
    pub note: String,
    /// Phân loại (FR46).
    pub category: Category,
    /// Xuất xứ (FR47).
    pub term_origin: TermOrigin,
    /// ISO-8601 UTC, sinh ở tầng SQL.
    pub created_at: String,
}

impl GlossaryEntry {
    /// **Vị từ DUY NHẤT** định nghĩa "đã chốt" — AD-36. Xem doc-comment đầu module.
    pub fn is_confirmed(&self) -> bool {
        self.translation.is_some()
    }
}

/// Một dấu khớp thuật ngữ trong một đoạn văn bản — Story 3.4, FR51.
///
/// ─────────────────────────────────────────────────────────────────────────────
/// 🔴 `start`/`end` LÀ ĐIỂM MÃ, KHÔNG PHẢI BYTE
/// ─────────────────────────────────────────────────────────────────────────────
/// `core::matching::find_terms` trả span **byte** (`Range<usize>` vào chuỗi gốc — xem
/// doc-comment của nó). [`super::store::marks_for_source_text`] quy đổi byte → điểm mã
/// **một lần, ở đúng một chỗ** trước khi trả kiểu này ra ngoài (§Design Notes của Story
/// 3.4: ba đơn vị đo — Rust/Matcher là BYTE, dây/lưới là ĐIỂM MÃ, DOM/`Range` là UTF-16 —
/// và Rust là nơi quy đổi byte→điểm mã, không để việc đó rơi xuống frontend nơi chuỗi JS
/// lại là một đơn vị THỨ BA). `start`/`end` ở đây luôn là số ĐIỂM MÃ tính từ đầu chuỗi.
///
/// 🔵 **SỬA 2026-08-22 (Story 3.6) — MANG THÊM `id` + `source_term`, mệnh đề "không mang
/// `source_term`/`id` trần" hết đúng.** Bản trước (Story 3.4) viết đúng cho PHẠM VI của nó:
/// `tier` + `is_confirmed` + `translation` là đủ để VẼ DẤU, và lúc đó không đường sản phẩm
/// nào cần ghi ngược lại một mục Glossary từ một dấu. Story 3.6 mở đường đó (dải mọc chốt
/// lần đầu gặp): chốt một mục *chờ chốt* cần biết CHÍNH XÁC hàng nào (`id`, cùng lý do
/// `GlossaryTier` tồn tại — `id` chỉ duy nhất TRONG một `Store`) và cần một KHOÁ GHI đúng
/// (`source_term` — xem §Design Notes của Story 3.6: bề mặt tiếng Anh khớp theo hình thái,
/// `dragons` trên màn hình có thể là hàng `dragon` trong Glossary; ghi bằng bề mặt là ghi
/// vào một mục không tồn tại). Không có hai trường này, dải phải tự đoán khoá ghi từ chuỗi
/// đã cắt trên màn hình (sai với biến thể hình thái) hoặc dựng một vòng IPC thứ hai để tra
/// lại dữ liệu mà Rust đã cầm sẵn trong tay lúc dựng dấu.
///
/// 🔵 **THÊM 2026-08-24 (Story 3.7) — `han_viet_suggestion` + `han_viet_status`.** Dải chốt
/// (Story 3.6) mở một ô nhập TRẦN cho mục chờ chốt; với danh từ riêng chữ Hán, âm Hán Việt
/// **chính là** bản dịch quy ước và nó đã nằm sẵn trong dữ liệu nhúng (FR113, AD-36). Hai
/// trường mới chở đề xuất đó cùng lượt với dấu — [`super::store::marks_for_source_text`] đã
/// cầm sẵn `source_term` của mục trong tay, gọi `suggest_han_viet_batch` một lần cho cả
/// Chương thay vì để nửa giao diện tự tra lại bằng một vòng IPC thứ hai.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GlossaryMark {
    /// Vị trí ĐIỂM MÃ bắt đầu (bao gồm), tính từ đầu chuỗi được khớp.
    pub start: usize,
    /// Vị trí ĐIỂM MÃ kết thúc (không bao gồm).
    pub end: usize,
    /// Tầng của mục thắng (AD-18) — cùng lý do [`GlossaryTier`] tồn tại: `id` chỉ duy nhất
    /// TRONG một `Store`, nên nửa giao diện cần biết tầng để hỏi lại đúng kho nếu cần.
    pub tier: GlossaryTier,
    /// `false` == mục *chờ chốt* — dấu vẫn ra (lượt tra KHÔNG lọc `is_confirmed`), nhưng
    /// nửa giao diện phải VẼ KHÁC (§I/O Matrix: "Mục chờ chốt ⇒ Có dấu, `is_confirmed=false`").
    pub is_confirmed: bool,
    /// `None` khi mục đang *chờ chốt*. `Some` mang đúng bản dịch đã chốt.
    pub translation: Option<String>,
    /// 🔵 **THÊM 2026-08-22 (Story 3.6).** `glossary_entry.id`, cùng với `tier` ở trên đủ để
    /// gọi `confirm_pending_translation(global, work, tier, id, ..)` mà không cần tra lại.
    pub id: i64,
    /// 🔵 **THÊM 2026-08-22 (Story 3.6).** KHOÁ GHI thật của mục — có thể KHÁC bề mặt đã
    /// khớp trên màn hình (`text[start..end]`) khi nhánh tiếng Anh khớp theo hình thái. Dải
    /// chốt phải hỏi VÀ ghi vào đúng `source_term` này, không phải chuỗi cắt từ văn bản.
    pub source_term: String,
    /// 🔵 **THÊM 2026-08-24 (Story 3.7).** Chuỗi đề xuất (`suggest_han_viet_batch(..).
    /// suggestion_text()`), hoặc `None` cho bốn trong năm nhánh của
    /// [`super::HanVietSuggestion`]. Mục ĐÃ CHỐT (`is_confirmed == true`) luôn mang `None` ở
    /// đây (nhánh `NotRequested`) — **0** lượt tra Hán Việt chạy cho mục đó.
    pub han_viet_suggestion: Option<String>,
    /// 🔵 **THÊM 2026-08-24 (Story 3.7).** `suggest_han_viet_batch(..).as_status_str()` —
    /// một trong năm chuỗi đóng (`"ok"` · `"not_chinese"` · `"no_reading"` ·
    /// `"dict_unavailable"` · `"not_requested"`). Ba lý do RỖNG của `han_viet_suggestion`
    /// phải phân biệt được trên dây; trường này là chỗ chúng phân biệt được (§I/O Matrix).
    pub han_viet_status: &'static str,
}
