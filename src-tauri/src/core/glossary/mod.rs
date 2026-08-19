//! Glossary hai tầng + bảng chờ ứng viên TÁCH RIÊNG (AD-20, AD-36).
//!
//! Đề xuất tự động luôn vào bảng chờ, KHÔNG BAO GIỜ vào Glossary (AD-20).
//! Vòng đời ba trạng thái; chỉ trạng thái cuối được chèn vào prompt (AD-36).
//!
//! ─────────────────────────────────────────────────────────────────────────────
//! HÌNH DẠNG ĐÃ DỰNG (Story 3.1)
//! ─────────────────────────────────────────────────────────────────────────────
//! - [`entry`] — kiểu THUẦN: [`entry::GlossaryEntry`], [`entry::Category`],
//!   [`entry::TermOrigin`]. [`entry::GlossaryEntry::is_confirmed`] là vị từ DUY NHẤT định
//!   nghĩa "đã chốt" — không cột `status` song song (AD-36).
//! - [`store`] — SQL: `insert_entry` · `confirm_translation` · `load_tier`, và **đúng MỘT**
//!   hàm phơi ra module khác, [`store::entries_eligible_for_injection`], lọc SAU khi phân
//!   giải qua `ScopeResolver::apply_override("glossary", ..)` (AD-18). Điều kiện chèn sống
//!   NGAY TRONG module này (AD-36) — cố ý lệch tiền lệ `core/segment/**`; tiền lệ đúng là
//!   `core/scope/store.rs`.
//! - Bảng riêng, `glossary_entry` — không một hàng `kind = 'glossary'` trong
//!   `config_value` (§Quyết định #1 của `core::store::schema`, `store.rs:283-291` từ chối
//!   đúng ca đó có chủ ý).
//!
//! ⚠️ **GIỚI HẠN THẬT — tầng Tác phẩm chưa đọc lại được sau khi khởi động lại ứng dụng.**
//! `ScopeResolver::with_work` chỉ được dựng ở `commands::project::create_work` — tức lúc
//! **TẠO MỚI** một `.atproj` trong phiên hiện tại. Hôm nay **không tồn tại đường mở lại**
//! một `.atproj` đã có trên đĩa (`OpenWorkState` khởi động luôn `None`, và không command
//! IPC nào ngoài `create_work_*` đặt được giá trị vào đó — `deferred-work.md:2465`). Hệ
//! quả cho Glossary: mục tầng Tác phẩm của một Tác phẩm đã đóng rồi mở lại **vẫn nằm
//! nguyên vẹn** trong `project.db` của nó — không mất dữ liệu — nhưng đường Rust để nạp
//! lại `ScopeResolver::with_work` cho phiên mới **chưa tồn tại**, nên
//! [`store::entries_eligible_for_injection`] không phân giải được tầng đó cho tới khi ai
//! đó mở Tác phẩm này lại. **Chủ: Epic 5** (đường mở lại `.atproj` — xem `deferred-work.md`
//! cho mục đóng đầy đủ).

pub mod entry;
pub mod store;

pub use entry::{Category, GlossaryEntry, TermOrigin};
pub use store::{
    GlossaryError, confirm_translation, entries_eligible_for_injection, insert_entry, load_tier,
};
