//! Văn bản giấy phép NGUYÊN VĂN, tải thật từ nguồn chính thức 2026-08-04 (Story 1.9,
//! Task 3/11) — ⛔ không phải bản tóm tắt tự viết. `include_str!` nhúng lúc biên dịch,
//! khớp khuôn `generate_context!` nhúng `dist/` của `src-tauri` (Dev Notes bàn giao #3).
//!
//! - CC BY-SA 4.0: `creativecommons.org/licenses/by-sa/4.0/legalcode.txt`
//! - Unicode License v3: `unicode.org/license.txt`
//! - GFDL 1.3: `gnu.org/licenses/fdl-1.3.txt`

pub const CC_BY_SA_4_0: &str = include_str!("../assets/licenses/CC-BY-SA-4.0.txt");
pub const UNICODE_LICENSE_V3: &str = include_str!("../assets/licenses/Unicode-License-v3.txt");
pub const GFDL_1_3: &str = include_str!("../assets/licenses/GFDL-1.3.txt");

/// Wiktionary (cả hai ấn bản) song hành hai giấy phép — nội dung mới hơn CC BY-SA 4.0,
/// nội dung cũ hơn có thể chỉ có GFDL. Ghi cả hai nguyên văn, nối bằng dòng phân cách rõ
/// ràng, thay vì chọn một và bỏ một.
pub fn cc_by_sa_and_gfdl() -> String {
    format!(
        "{CC_BY_SA_4_0}\n\n\
         =======================================================================\n\
         Nội dung Wiktionary song hành giấy phép — văn bản GFDL 1.3 dưới đây\n\
         =======================================================================\n\n\
         {GFDL_1_3}"
    )
}
