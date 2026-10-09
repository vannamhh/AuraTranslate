//! Xuất: docx · md · TMX + segment alignment + khối ghi nguồn (AD-38, AD-43).
//!
//! Cấu trúc đoạn là dữ liệu ĐƯỢC LƯU, không phải thứ suy ra lúc xuất (AD-37).
//! Khối ghi nguồn dựng lúc xuất từ các cột `chapter.origin_*`, không lưu sẵn (AD-43).
//!
//! Crate dành cho module này: `docx-rs` (bộ GHI `.docx`).
//!
//! 🔵 **SỬA 2026-09-09 (Story 6.12) — `docx-rs` nay cũng có một đường ĐỌC trong kho, nhưng
//! KHÔNG ở đây.** `core::docx` (mới) tự đọc OOXML bằng `zip` + `quick-xml`, không gọi
//! `docx_rs::read_docx` — 140 điểm panic trong `docx-rs/src/reader/` dưới `panic = "abort"`
//! làm một cổng "tệp hỏng" không đóng được bằng bất kỳ lớp chắn nào (xem doc-comment đầu
//! `core::docx`). `docx-rs` ở lại ĐÚNG một vai tại đây: bộ GHI cho `core::export` (AD-38,
//! Epic 8) — và thêm vai THỨ HAI, bộ SINH FIXTURE cho bộ test của Story 6.12
//! (`tests/fixtures_docx.rs`), một cài đặt ĐỘC LẬP với `core::docx` nên không phải một vòng
//! tròn "tự sinh rồi tự đọc lại".

mod alignment;
mod attribution;
mod block_paragraphs;
mod docx_block;
mod docx_table;
mod image_files;
mod images;
mod new_file;
mod reimport_gate;
mod reviewer_copy;
mod scope;
mod table_rows;
mod text_export;

pub use alignment::{
    AlignmentError, AlignmentGroup, AlignmentItem, AlignmentRow, AlignmentSegment, ChapterAlignment, DecidedBy, GroupDiff,
    MIN_PAIR_SIMILARITY, align_chapter, delete_alignment_of_chapter, join, move_members_of_retired, read_alignment,
    review_diff, skip, unjoin,
};
pub use attribution::{Attribution, ChapterOrigin, attribution_lines};
pub use block_paragraphs::{
    BlockParagraph, ChapterBlock, LoadedBlocks, UNTRANSLATED_SQL, is_untranslated, load_chapter_blocks,
};
pub use docx_block::write_one_block_docx;
pub use docx_table::{DocxWriteError, ImageReference, write_two_column_docx};
pub use image_files::{IMAGE_DIR_SUFFIX, ImageFilesError, write_file_with_images};
pub use images::{ImageMode, ImageScan, MissingLinkImage, scan_images};
pub use new_file::{safe_stem, write_new_file};
pub use reimport_gate::{ReimportShapeError, ReviewerDocx};
pub use reviewer_copy::{
    ConfirmError, ImportSummary, PlannedChapter, ReplacedCopy, ReviewCopy, ReviewCopyError, ReviewFileKind, ReviewRow,
    ReviewRowKind, ReviewSection, ReviewerCopy, ReviewerImportPlan, SkippedSection, confirm_import, plan_import,
    read_docx_copy, read_markdown_copy, read_review_copy,
};
pub use scope::{ExportScope, ScopeCounts, ScopeError, count_scope, resolve_chapter_ids};
pub use text_export::{TextFormat, load_chapter_text, render_text};
pub use table_rows::{ChapterTable, ExportCell, ExportImage, ExportRow, LoadedTables, load_chapter_tables};
