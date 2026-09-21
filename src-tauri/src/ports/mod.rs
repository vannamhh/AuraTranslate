//! ĐÚNG BA cổng, không hơn: `DictionarySource` · `TranslationProvider` · `ProjectStore` (AD-2).
//!
//! Cổng thứ tư phải là một AD mới — không thêm trait vào đây bằng suy luận tại chỗ.
//!
//! Story 1.2 chưa khai trait nào; mỗi cổng do story sở hữu năng lực tương ứng dựng.
//!
//! | Cổng | Trạng thái | Chủ |
//! |---|---|---|
//! | [`DictionarySource`] | ✅ **đã khai** | Story 1.13 — tầng gom nhiều tệp `.db` |
//! | [`TranslationProvider`] | ✅ **đã khai** *(chưa có cài đặt — xem doc-comment của module)* | Story 4.8 *(Phase 1 — cài đặt đến ở Phase 2, `core/ai/client.rs`)* |
//! | [`ProjectStore`] | ✅ **đã khai** *(chưa có cài đặt — xem doc-comment của module)* | Story 1.15 *(`.atproj` trên đĩa)* |
//!
//! **Không** trait thứ tư ở đây. AD-2 khai **đúng ba**, và một cổng thứ tư phải là một
//! AD mới chứ không phải một suy luận tại chỗ.

pub mod dict_source;
pub mod project_store;
pub mod translation_provider;

pub use dict_source::DictionarySource;
pub use project_store::ProjectStore;
pub use translation_provider::TranslationProvider;
