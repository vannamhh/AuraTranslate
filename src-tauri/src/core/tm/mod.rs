//! Translation Memory — khoá theo CẶP VĂN BẢN, không theo `segment.id` (AD-6).
//!
//! Nhờ vậy TM sống sót qua gộp/tách segment và dùng lại được xuyên Tác phẩm.
//!
//! ─────────────────────────────────────────────────────────────────────────────
//! 🔵 THÊM 2026-09-17 (Story 4.6) — `SimilarSegment`, hình dạng TỐI THIỂU tham số TM của
//! `core::ai::rag` cần
//! ─────────────────────────────────────────────────────────────────────────────
//! `RagInjector` (AD-14) nhận TM làm tham số thứ tư, nhưng Epic 7 (module thật của TM) chạy
//! SAU Epic 4 — chữ ký cần một kiểu PHẦN TỬ thật ngay hôm nay, không phải `()` hay một kiểu
//! đoán mò. Kiểu này KHÔNG thể sống dưới `core/ai/**`: `tests/ai_boundary.rs:99` cấm module
//! khác gõ tên `crate::core::ai`, nên nếu Epic 7 cần đặt TÊN kiểu phần tử của chính module
//! mình (để trả về từ một hàm tìm kiếm thật) mà kiểu đó sống ở `core::ai`, `core::tm` sẽ phải
//! `use crate::core::ai::SimilarSegment` — đúng phụ thuộc mà AD-13 cấm. `core::ai` được PHÉP
//! đọc `core::tm` (chiều ngược của AD-13), không phải chiều kia.
//!
//! Epic 7 sẽ LỚN kiểu này lên (điểm tương đồng, id đoạn nguồn, …) — `core::ai::rag` chỉ nhận
//! `&[SimilarSegment]`, không tháo rời từng trường, nên việc lớn lên không đổi chữ ký của
//! `gather_glossary_context`/`assemble_prompt`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SimilarSegment {
    /// Câu nguồn đã lưu trong TM.
    pub source_text: String,
    /// Bản dịch đã lưu song song với `source_text`.
    pub target_text: String,
}

/// FR118 binary axis over the FR117 origins that may enter TM.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PairSide {
    Mine,
    Others,
}

/// The origins a TM pair may carry (AD-47 ⑥); `''` ("no answer") is not representable.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PairOrigin {
    SelfTranslated,
    Other,
    BilingualImport,
}

impl PairOrigin {
    pub fn from_stored(value: &str) -> Option<Self> {
        match value {
            "self" => Some(Self::SelfTranslated),
            "other" => Some(Self::Other),
            "bilingual_import" => Some(Self::BilingualImport),
            _ => None,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::SelfTranslated => "self",
            Self::Other => "other",
            Self::BilingualImport => "bilingual_import",
        }
    }

    pub fn side(self) -> PairSide {
        match self {
            Self::SelfTranslated => PairSide::Mine,
            Self::Other | Self::BilingualImport => PairSide::Others,
        }
    }
}

/// Appends a pair in the caller's transaction; existing rows are never updated (AD-6).
pub fn insert_pair(
    tx: &crate::core::store::Transaction<'_>,
    source_text: &str,
    target_text: &str,
    translation_origin: PairOrigin,
) -> crate::core::store::SqlResult<()> {
    tx.execute(
        "INSERT INTO tm_unit (source_text, target_text, translation_origin, created_at) \
         VALUES (?1, ?2, ?3, strftime('%Y-%m-%dT%H:%M:%fZ','now'))",
        (source_text, target_text, translation_origin.as_str()),
    )?;
    Ok(())
}
