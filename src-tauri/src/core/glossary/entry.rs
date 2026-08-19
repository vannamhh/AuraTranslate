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

/// Phân loại thuật ngữ (FR46). Ba giá trị, cưỡng chế lại ở `CHECK (category IN (…))` của
/// `GLOSSARY_ENTRY_DDL` — cột SQL và kiểu Rust không được phép trôi khỏi nhau.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Category {
    /// Tên người.
    Person,
    /// Địa danh.
    Place,
    /// Thuật ngữ chuyên ngành.
    DomainTerm,
    /// Còn lại.
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
