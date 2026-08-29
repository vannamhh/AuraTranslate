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
    /// `meta.json` của một `.atproj` **đã có trên đĩa** mang `meta_schema_version` mới hơn
    /// bản ứng dụng hiểu — **THÊM Story 5.7**, xem khối comment ngay dưới cho vì sao đây là
    /// một biến thể MỚI thay vì gộp vào [`Self::CreateFailed`] như `From<MetaError>` từng
    /// làm trước lượt này.
    MetaTooNew {
        /// Phiên bản đọc được.
        found: u32,
        /// Phiên bản cao nhất bản ứng dụng này hiểu.
        supported: u32,
    },
    /// Mở lại một `.atproj` **đã có trên đĩa** thất bại vì một lý do KHÁC `meta.json` quá
    /// mới — thư mục biến mất, quyền đọc, `Store::open` từ chối vì một lý do khác
    /// `SchemaTooNew` (nhánh đó có `MessageKey` riêng, `store.schema_too_new`, không đi qua
    /// đây). **THÊM Story 5.7**.
    OpenFailed {
        /// Tên hiển thị của Tác phẩm, cho câu báo (`err.work.open_failed`).
        name: String,
        /// Lỗi thô, chỉ để chẩn đoán.
        detail: String,
    },
}

// 🔵 **SỬA 2026-08-29 (Story 5.7) — khối comment dưới đây đã HẾT ĐÚNG, giữ lại kèm gạch
// ngang thay vì xoá, đúng luật "mệnh đề hết đúng thì sửa tại chỗ, đừng xoá".** ~~*"KHÔNG có
// biến thể `MetaTooNew` ở đây... Story nào dựng đường mở lại `.atproj` sở hữu việc thêm lại
// biến thể này + `MessageKey` + khoá `vi.json` CÙNG MỘT LƯỢT."*~~ Story 5.7 LÀ story đó —
// `commands::project::open_work` (chỗ gọi sản phẩm ĐẦU TIÊN của `WorkMeta::read`, xem
// doc-comment của hàm đó ở `core/library/meta.rs`) mở lại một `.atproj` đã có trên đĩa, nên
// biến thể [`WorkError::MetaTooNew`] + `MessageKey::WorkMetaTooNew` + khoá
// `err.work.meta_too_new` trong `vi.json` được thêm lại đúng như khối comment cũ đã tiên
// liệu. Cơ chế từ chối bên dưới ([`MetaError::SchemaTooNew`] + `WorkMeta::read`) không đổi
// một dòng — chỉ bề mặt HIỂN THỊ của nó, từ "không tồn tại" thành "tồn tại".

impl std::fmt::Display for WorkError {
    /// ⚠️ KHÔNG DẤU — chẩn đoán cho log (NFR16).
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WorkError::CreateFailed { detail } => {
                write!(f, "work create failed: {detail}")
            }
            WorkError::MetaTooNew { found, supported } => {
                write!(f, "work meta schema {found} is newer than supported {supported}")
            }
            WorkError::OpenFailed { name, detail } => {
                write!(f, "work open failed for {name}: {detail}")
            }
        }
    }
}

impl std::error::Error for WorkError {}

impl From<MetaError> for WorkError {
    fn from(err: MetaError) -> Self {
        match err {
            MetaError::Io { detail, .. } => WorkError::CreateFailed { detail },
            // 🔵 **SỬA 2026-08-29 (Story 5.7)** — trước lượt này gộp vào `CreateFailed` bằng
            // một chuỗi chẩn đoán, vì không chỗ gọi sản phẩm nào của `WorkMeta::read` cần
            // phân biệt nó với các lỗi tạo mới khác. Nay `open_work` CẦN phân biệt (AC8: câu
            // báo phải "nói đúng loại lỗi", có `found`/`supported`) ⇒ ánh xạ sang biến thể
            // riêng thay vì gộp.
            MetaError::SchemaTooNew { found, supported } => WorkError::MetaTooNew { found, supported },
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
            WorkError::MetaTooNew { found, supported } => {
                let mut params = BTreeMap::new();
                params.insert("found".to_owned(), found.to_string());
                params.insert("supported".to_owned(), supported.to_string());
                IpcError::new("work.meta_too_new", MessageKey::WorkMetaTooNew, params, false)
            }
            WorkError::OpenFailed { name, .. } => {
                let mut params = BTreeMap::new();
                params.insert("name".to_owned(), name);
                IpcError::new("work.open_failed", MessageKey::WorkOpenFailed, params, false)
            }
        }
    }
}
