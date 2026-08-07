//! Văn bản giấy phép NGUYÊN VĂN, tải thật từ nguồn chính thức 2026-08-04 (Story 1.9,
//! Task 3/11) — không phải bản tóm tắt tự viết. `include_str!` nhúng lúc biên dịch,
//! khớp khuôn `generate_context!` nhúng `dist/` của `src-tauri` (Dev Notes bàn giao #3).
//!
//! - CC BY-SA 4.0: `creativecommons.org/licenses/by-sa/4.0/legalcode.txt`
//! - Unicode License v3: `unicode.org/license.txt`
//! - GFDL 1.3: `gnu.org/licenses/fdl-1.3.txt`
//!
//! Hai lớp GỠ RỜI của Story 1.10 mang thêm hai văn bản:
//! - CC0 1.0 Universal (Thiều Chửu): `creativecommons.org/publicdomain/zero/1.0/legalcode.txt`
//! - Tuyên bố xuất xứ soạn tay cho Thiều Chửu và VietPhrase (`thieu-chuu.txt`,
//!   `vietphrase.txt`) — KHÔNG phải giấy phép mở có sẵn để tải, nên là văn bản do dự
//!   án soạn, ghi rõ nguồn và trạng thái pháp lý đã kiểm chứng (§Thông tin kỹ thuật).

pub const CC_BY_SA_4_0: &str = include_str!("../assets/licenses/CC-BY-SA-4.0.txt");
pub const UNICODE_LICENSE_V3: &str = include_str!("../assets/licenses/Unicode-License-v3.txt");
pub const GFDL_1_3: &str = include_str!("../assets/licenses/GFDL-1.3.txt");
pub const CC0_1_0: &str = include_str!("../assets/licenses/CC0-1.0.txt");
pub const THIEU_CHUU_DECLARATION: &str = include_str!("../assets/licenses/thieu-chuu.txt");
pub const VIETPHRASE_DECLARATION: &str = include_str!("../assets/licenses/vietphrase.txt");
/// Story 1.10c — tuyên bố xuất xứ soạn tay cho Trần Văn Chánh, nêu RÕ rủi ro pháp lý
/// (AC8): CC0 của người số hoá không xoá bản quyền tác phẩm gốc của tác giả còn sống.
pub const TRAN_VAN_CHANH_DECLARATION: &str =
    include_str!("../assets/licenses/tran-van-chanh.txt");

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

/// Thiều Chửu: tuyên bố xuất xứ (nguồn, nghĩa vụ ghi công quyền nhân thân) ĐI TRƯỚC,
/// rồi toàn văn CC0 1.0 — người đọc `license_text` thấy ngay bối cảnh trước khi đọc
/// văn bản giấy phép hình thức (Story 1.10, AC2).
pub fn thieu_chuu_license_text() -> String {
    format!(
        "{THIEU_CHUU_DECLARATION}\n\n\
         =======================================================================\n\
         Toàn văn giấy phép CC0 1.0 Universal — áp dụng cho bản số hoá\n\
         =======================================================================\n\n\
         {CC0_1_0}"
    )
}

/// VietPhrase: KHÔNG có giấy phép mở nào để đính kèm (`license_kind = 'unknown'`) —
/// `license_text` là tuyên bố xuất xứ, không hơn không kém (Story 1.10, AC3).
pub fn vietphrase_license_text() -> String {
    VIETPHRASE_DECLARATION.to_string()
}

/// Trần Văn Chánh: tuyên bố xuất xứ (rủi ro pháp lý ĐI TRƯỚC, AC8) rồi toàn văn CC0 1.0
/// của người số hoá — cùng khuôn `thieu_chuu_license_text`, nhưng **không** giống về
/// Ý NGHĨA: ở đây CC0 chỉ phủ công sức số hoá, KHÔNG phủ bản quyền tác phẩm gốc (còn
/// hiệu lực) — tuyên bố xuất xứ nói rõ điều này trước khi văn bản CC0 xuất hiện, để
/// người đọc không hiểu lầm CC0 = "nguồn sạch".
pub fn tran_van_chanh_license_text() -> String {
    format!(
        "{TRAN_VAN_CHANH_DECLARATION}\n\n\
         =======================================================================\n\
         Toàn văn CC0 1.0 Universal của NGƯỜI SỐ HOÁ — KHÔNG áp dụng cho bản quyền\n\
         tác phẩm gốc (xem cảnh báo pháp lý ở trên)\n\
         =======================================================================\n\n\
         {CC0_1_0}"
    )
}
