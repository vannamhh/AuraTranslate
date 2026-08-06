//! Cổng **thứ ba** trong đúng ba của AD-2 — một Tác phẩm **đã mở**, nhìn qua một trait.
//!
//! ─────────────────────────────────────────────────────────────────────────────
//! 🔴 CHỈ KHAI HÌNH DẠNG, ⛔ KHÔNG MANG CÀI ĐẶT — cùng luật với [`super::DictionarySource`]
//! ─────────────────────────────────────────────────────────────────────────────
//! `tests/dict_boundary.rs::ports_declare_shape_and_never_open_anything` quét thư mục này:
//! ⛔ không `rusqlite`, ⛔ không `Connection::open`, ⛔ không `fs::`, ⛔ không `PathBuf` — ở
//! **vị trí mã**. Cài đặt thật (mở `.atproj/`, đọc/ghi `project.db` và `meta.json`) sống ở
//! `core::library` (`atproj.rs`, `meta.rs`) và `core::store` (`StoreSpec::project`), ⛔
//! không ở đây.
//!
//! ─────────────────────────────────────────────────────────────────────────────
//! ⚠️ VÌ SAO CỔNG NÀY CHƯA CÓ CÀI ĐẶT NÀO Ở STORY 1.15, KHÁC VỚI `DictionarySource`
//! ─────────────────────────────────────────────────────────────────────────────
//! `commands::project` (Task 6 của Story 1.15) gọi thẳng `Option<&Store>` — cùng khuôn
//! `commands::config` — vì câu chuyện của story này (tạo một Tác phẩm, đọc lại metadata)
//! chưa cần một lớp trừu tượng phía trên `Store`. Cổng này khai **hình dạng** mà các epic
//! sau (Editor — Epic 2, Glossary — Epic 3) sẽ cần khi họ thao tác trên "một Tác phẩm đã
//! mở" mà không muốn biết nó là SQLite hay bất cứ thứ gì khác — đúng lý do AD-2 tồn tại.
//! Đây là cùng hoàn cảnh `TranslationProvider` (Epic 4) đang ở: khai trước, cắm cài đặt
//! sau, khi có consumer thật.
//!
//! ⛔ **Và tệp này ⛔ không gõ tên crate SQLite, ⛔ không `Connection::open`, ⛔ không chạm
//! filesystem.** Kiểu bản ghi ([`WorkMeta`]) sống ở [`crate::core::library`]; lỗi
//! ([`StoreError`]) sống ở [`crate::core::store`]. Trait này **tham chiếu** chúng.

use crate::core::library::WorkMeta;
use crate::core::store::StoreError;

/// Một Tác phẩm **đã mở**, nhìn qua cổng.
///
/// 🔴 Đơn vị là **một Tác phẩm đang mở**, ⛔ không phải "một cách mở tệp" — cùng tinh thần
/// với AD-44 ⑤ cấm một adapter theo ngôn ngữ ở [`super::DictionarySource`]: người gọi cổng
/// này không cần biết `.atproj/` có mấy tệp bên trong, chỉ cần biết Tác phẩm nào và Chương
/// nào.
pub trait ProjectStore {
    /// Metadata hiện tại — cùng hình dạng `meta.json` (AD-33).
    fn meta(&self) -> &WorkMeta;

    /// Văn bản nguồn nguyên khối của một Chương. `chapter_id` là số nguyên cục bộ của
    /// `project.db` (AD-28) — vô nghĩa ở bất kỳ Tác phẩm nào khác.
    fn chapter_source_text(&self, chapter_id: i64) -> Result<String, StoreError>;
}
