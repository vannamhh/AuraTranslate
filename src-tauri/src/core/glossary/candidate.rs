//! `GlossaryCandidate` thuần + `CandidateOrigin`/`Resolution` — Story 3.2, AD-20 · AD-36.
//!
//! ─────────────────────────────────────────────────────────────────────────────
//! 🔴 `is_pending()` LÀ VỊ TỪ DUY NHẤT ĐỊNH NGHĨA "CHỜ DUYỆT" — không cột `is_pending`
//! ─────────────────────────────────────────────────────────────────────────────
//! `glossary_candidate.resolution IS NULL` **là** trạng thái *chờ duyệt*
//! (`schema.rs::GLOSSARY_CANDIDATE_DDL`) — cùng khuôn cấu trúc mà
//! [`super::entry::GlossaryEntry::is_confirmed`] dùng cho `translation IS NULL` ở Story
//! 3.1. Không có một cột `is_pending` song song ghi lại cùng sự thật — hai dữ kiện nói
//! cùng một chuyện thì chúng lệch được, và lệch trong im lặng.
//!
//! ⚠️ Mọi chuỗi trong `src-tauri/src/**` viết KHÔNG DẤU; doc-comment có dấu là hợp lệ.

use std::fmt;

use super::entry::TermOrigin;

/// Xuất xứ một ứng viên (FR47, biến thể *tự động*) — **hai** giá trị, khớp NGUYÊN VĂN
/// `CHECK (candidate_origin IN (…))` của `GLOSSARY_CANDIDATE_DDL`.
///
/// 🔴 **Không có `Manual`.** Một mục nhập tay không đi qua bảng chờ — nó ghi thẳng vào
/// `glossary_entry` qua [`super::store::insert_manual_entry`]. Ca thứ ba đó không biểu
/// diễn được ở đây, và đó là chủ ý: [`Self::to_term_origin`] là một ánh xạ TOÀN PHẦN đúng
/// vì miền của nó đã loại sẵn giá trị không thể xảy ra.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CandidateOrigin {
    /// Quét khi nhập tài liệu (Story 3.5).
    ImportScan,
    /// Thu hoạch từ bản review (Epic 8).
    ReviewHarvest,
}

impl CandidateOrigin {
    /// Định danh máy đọc — khớp NGUYÊN VĂN giá trị trong `CHECK` của
    /// `GLOSSARY_CANDIDATE_DDL`. Không phải nhãn hiển thị (AD-21, NFR16).
    pub const fn as_str(self) -> &'static str {
        match self {
            CandidateOrigin::ImportScan => "import_scan",
            CandidateOrigin::ReviewHarvest => "review_harvest",
        }
    }

    /// Phân giải một giá trị đến từ đĩa. `None` cho mọi chuỗi không khớp — không đoán,
    /// cùng lý do [`super::entry::Category::from_wire`].
    pub fn from_wire(raw: &str) -> Option<Self> {
        match raw {
            "import_scan" => Some(CandidateOrigin::ImportScan),
            "review_harvest" => Some(CandidateOrigin::ReviewHarvest),
            _ => None,
        }
    }

    /// **Ánh xạ TOÀN PHẦN** sang [`TermOrigin`] — chỗ DUY NHẤT trong toàn kho sinh ra một
    /// `TermOrigin` khác `Manual`. Toàn phần (không `Option`, không nhánh "còn lại") là
    /// thứ làm AC *"mục vừa duyệt mang đúng xuất xứ"* đúng THEO KIỂU: không có giá trị
    /// `CandidateOrigin` nào không ánh xạ được, nên [`crate::core::glossary::candidate_store::approve_candidate`]
    /// không bao giờ cần một nhánh lỗi cho bước này.
    pub const fn to_term_origin(self) -> TermOrigin {
        match self {
            CandidateOrigin::ImportScan => TermOrigin::ImportScan,
            CandidateOrigin::ReviewHarvest => TermOrigin::ReviewHarvest,
        }
    }
}

impl fmt::Display for CandidateOrigin {
    /// KHÔNG DẤU — chẩn đoán cho log, không phải văn bản hiển thị (NFR16).
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Quyết định của người dùng trên một ứng viên (AD-36 áp cho bảng chờ) — `None` ở
/// [`GlossaryCandidate::resolution`] nghĩa là *chờ duyệt*; `Some(_)` là MỘT trong hai giá
/// trị đóng dưới đây, không bao giờ giá trị thứ ba.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Resolution {
    /// Đã duyệt — `glossary_entry` tương ứng đã sinh ra trong cùng giao dịch.
    Approved,
    /// Đã bỏ — hàng ứng viên Ở LẠI trên đĩa (không `DELETE`), chỉ đổi `resolution`.
    Rejected,
}

impl Resolution {
    /// Định danh máy đọc — khớp NGUYÊN VĂN giá trị trong `CHECK` của
    /// `GLOSSARY_CANDIDATE_DDL`.
    pub const fn as_str(self) -> &'static str {
        match self {
            Resolution::Approved => "approved",
            Resolution::Rejected => "rejected",
        }
    }

    /// Phân giải một giá trị đến từ đĩa. `None` cho mọi chuỗi không khớp.
    pub fn from_wire(raw: &str) -> Option<Self> {
        match raw {
            "approved" => Some(Resolution::Approved),
            "rejected" => Some(Resolution::Rejected),
            _ => None,
        }
    }
}

impl fmt::Display for Resolution {
    /// KHÔNG DẤU — chẩn đoán cho log, không phải văn bản hiển thị (NFR16).
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Một hàng `glossary_candidate` đã nạp — kiểu THUẦN, không mang SQL nào (SQL sống ở
/// [`super::candidate_store`]).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GlossaryCandidate {
    /// Khoá hàng.
    pub id: i64,
    /// Thuật ngữ nguồn — `UNIQUE` trong bảng (`idx_glossary_candidate_source_term`).
    pub source_term: String,
    /// Xuất xứ tự động — cách ứng viên này ra đời.
    pub candidate_origin: CandidateOrigin,
    /// `None` == *chờ duyệt*. Đây là trường DUY NHẤT [`Self::is_pending`] đọc.
    pub resolution: Option<Resolution>,
    /// ISO-8601 UTC, sinh ở tầng SQL.
    pub created_at: String,
}

impl GlossaryCandidate {
    /// **Vị từ DUY NHẤT** định nghĩa "chờ duyệt". Xem doc-comment đầu module.
    pub fn is_pending(&self) -> bool {
        self.resolution.is_none()
    }
}
