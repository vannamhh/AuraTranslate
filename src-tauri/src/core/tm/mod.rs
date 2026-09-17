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
