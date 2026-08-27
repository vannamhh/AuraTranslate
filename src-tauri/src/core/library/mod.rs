//! Chỉ mục Library là DẪN XUẤT, một đường ghi duy nhất (AD-8).
//!
//! `.atproj` ghi trước, `library-index.db` ghi sau. Quét lại luôn dựng lại được
//! chỉ mục từ đĩa — mất `library-index.db` không mất dữ liệu.
//!
//! ─────────────────────────────────────────────────────────────────────────────
//! HÌNH DẠNG THÊM Ở STORY 1.15 — `.atproj/` LÀ MỘT THƯ MỤC (AD-9)
//! ─────────────────────────────────────────────────────────────────────────────
//! - [`meta`] — `meta.json`, đọc được không cần mở SQLite (AC3), ghi nguyên tử
//!   (Quyết định #3), dựng lại được từ `project.db` (AD-33).
//! - [`atproj`] — dựng `<Tên>.atproj/` + `assets/` trên đĩa (AC2, AC5, AC6).
//! - [`indexer`] — `library-index.db`, chỉ mục dẫn xuất (AD-8) — **Story 5.2**.
//! - [`orphan_store`] — bảng `library_orphan` của `global.db` — **phán quyết Ice #1, Story
//!   5.3, 2026-08-27**: cờ mồ côi là dữ liệu người dùng, không phải trạng thái của chỉ mục
//!   dẫn xuất, nên nó sống ở đây chứ không phải một cột trong `indexer::IndexedWork`.
//!
//! 🔵 **CẬP NHẬT 2026-08-27 (Story 5.2) — câu dưới đây đã HẾT ĐÚNG, sửa tại chỗ thay vì để nó
//! lặng lẽ sai.** ~~*"Story này không dựng `library-index.db` — màn hình Library đọc thẳng
//! `meta.json` chỉ vì chưa có chỉ mục (Story 5.2 sở hữu `library-index.db`)."*~~ `library-index.db`
//! nay tồn tại (xem [`indexer`]). Điều còn đúng: chưa màn hình nào đọc nó — [`indexer::Indexer::list_works`]
//! là đường ĐỌC duy nhất có test hôm nay; bề mặt hiển thị (lưới Tác phẩm) là Story 5.6.

pub mod atproj;
pub mod indexer;
pub mod meta;
pub mod orphan_store;

pub use atproj::{create_work_folder, remove_folder, sanitize_name};
pub use meta::{META_SCHEMA_VERSION, MetaError, WorkMeta};

use crate::core::i18n::{IpcError, MessageKey};

/// Mọi cách một thao tác ở tầng Tác phẩm hỏng mà **không phải** lỗi kho SQLite.
///
/// ⚠️ Vì sao đây KHÔNG chỉ là `StoreError` được đổi tên: `.docx` bị từ chối hay tệp
/// nửa vời bị dọn dẹp đều xảy ra **trước** hoặc **ngoài** một giao dịch SQL —
/// `tests/scope_contract.rs::every_command_error_comes_from_the_store_vocabulary` đúng khi
/// đỏ trên chúng nếu chúng bị nhét vào `StoreError`, vì AC8 của story này PHÁ mệnh đề "mọi
/// lỗi command đều từ từ vựng kho" một cách có ý thức (xem `tests/scope_contract.rs` và
/// Completion Notes của story).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WorkError {
    /// Không dựng được `<Tên>.atproj/` trên đĩa — quyền, tên trùng, hoặc I/O khác.
    CreateFailed {
        /// Lỗi thô, chỉ để chẩn đoán.
        detail: String,
    },
}

// **KHÔNG có biến thể `MetaTooNew` ở đây, và đó là một quyết định** — Ice chốt ở lượt
// code review 2026-08-06. Cơ chế từ chối một `meta.json` mới hơn **vẫn còn nguyên và vẫn
// có test** ([`MetaError::SchemaTooNew`] + `WorkMeta::read` +
// `tests/project_contract.rs::a_newer_meta_schema_is_refused_without_touching_a_single_byte`).
// Thứ bị gỡ là **bề mặt HIỂN THỊ** của nó: story này không dựng màn hình "mở lại một
// `.atproj` đã có", nên `WorkMeta::read` không có một chỗ gọi sản phẩm nào, nên một
// `MessageKey` + một khoá `vi.json` cho nó là **một khoá cho tính năng chưa tồn tại** —
// đúng thứ Story 1.7 §Completion Notes #3 cấm, và `tests/scope_contract.rs` trích lại
// nguyên văn. 🔴 **Story nào dựng đường mở lại `.atproj` (ứng viên: Epic 5, lưới Tác phẩm)
// sở hữu việc thêm lại biến thể này + `MessageKey` + khoá `vi.json` CÙNG MỘT LƯỢT.**

impl std::fmt::Display for WorkError {
    /// ⚠️ KHÔNG DẤU — chẩn đoán cho log (NFR16).
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WorkError::CreateFailed { detail } => {
                write!(f, "work create failed: {detail}")
            }
        }
    }
}

impl std::error::Error for WorkError {}

impl From<MetaError> for WorkError {
    fn from(err: MetaError) -> Self {
        match err {
            MetaError::Io { detail, .. } => WorkError::CreateFailed { detail },
            // ⚠️ Gộp vào `CreateFailed` **có chủ ý** — hai con số giữ lại trong chuỗi chẩn
            // đoán, không mất. Xem khối comment ở trên về vì sao không có một hạng
            // lỗi hiển thị riêng cho ca này hôm nay.
            MetaError::SchemaTooNew { found, supported } => WorkError::CreateFailed {
                detail: format!("meta schema {found} is newer than supported {supported}"),
            },
        }
    }
}

/// 🔴 Đi qua [`IpcError::new`], không dựng struct literal — cùng luật với
/// `From<StoreError> for IpcError`.
impl From<WorkError> for IpcError {
    fn from(err: WorkError) -> Self {
        use std::collections::BTreeMap;

        match err {
            WorkError::CreateFailed { .. } => {
                IpcError::new("work.create_failed", MessageKey::WorkCreateFailed, BTreeMap::new(), false)
            }
        }
    }
}
