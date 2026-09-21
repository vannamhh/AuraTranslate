//! Bí mật khoá API — Story 4.3, FR65/FR67/NFR11. Đây là chỗ DUY NHẤT ngoài `core/ai/` được
//! phép gọi tên crate `keyring` (AD-29, `core/ai/mod.rs`) — khoá API là một domain CẤU HÌNH
//! hai tầng cùng `core/aiconfig/mod.rs`, không phải logic dịch, và cùng lý lẽ đó nó không
//! biết `TranslationProvider`, không gọi ra mạng, không tên `crate::core::ai`/`super::ai`.
//!
//! ─────────────────────────────────────────────────────────────────────────────
//! 🔴 MỘT ENTRY KEYCHAIN CHO TOÀN ỨNG DỤNG — GLOBAL-ONLY, KHÔNG THEO TÁC PHẨM
//! ─────────────────────────────────────────────────────────────────────────────
//! Quyết định Ice 2026-09-17 (§Intent spec 4.3): FR68 chỉ khai `provider`/`model`/tham số
//! sinh là hai tầng; FR67 và các AC của story này không nhắc gì đến tầng. Một khoá theo
//! từng Tác phẩm sẽ cần một định danh Tác phẩm ổn định làm một phần tên account CỘNG một
//! câu hỏi "mồ côi" chưa app nào trả lời được (không cơ chế nào liệt được các entry đã có
//! trong keychain, nên xoá một Tác phẩm sẽ để lại một bí mật không ai còn tìm ra). Vì vậy
//! `set`/`delete`/`configured`/`read` dưới đây KHÔNG nhận tham số tầng — khác hẳn
//! `store::write_field`/`clear_field` của năm trường kia, hai bảng này KHÔNG cùng hình
//! dạng dù sống cạnh nhau. Từ chối một yêu cầu ở tầng Tác phẩm là việc của tầng LỆNH
//! (`commands::aiconfig`, Task 6/Phase 2) — module này không có khái niệm "tầng" để mà từ
//! chối.
//!
//! ─────────────────────────────────────────────────────────────────────────────
//! ⚠️ RAW-VALUE ACCESSOR — CHỈ ĐƯỢC GỌI TỪ TRONG `core/aiconfig/**`, CỘNG ĐÚNG MỘT TỆP NGOÀI
//! ─────────────────────────────────────────────────────────────────────────────
//! [`ApiKeySecret::expose_secret`] và [`read`] trả GIÁ TRỊ THẬT của khoá.
//! 🔵 **SỬA 2026-09-21 (Story 4.8, Phase 2)** — câu cũ ở đây nói "hôm nay KHÔNG chỗ nào gọi
//! chúng ngoài `core/aiconfig/**`"; nay SAI, đo trên mã thật:
//! `commands::aitranslate::prepare_translate_call` gọi cả hai để dựng request dịch thật ra
//! provider — đúng chỗ gọi mà câu cũ đã dự đoán. `tests/aiconfig_keychain_boundary.rs::
//! AI_TRANSLATE_KEYCHAIN_CALLER_FILE` khoá ranh giới lại đúng MỘT tệp đó, không mở rộng theo
//! tiền tố — Rust tự nó không chặn được `pub(crate)` bị gọi từ một module khác trong CÙNG
//! crate, nên phép quét văn bản là hàng rào thật, không phải một phép lịch sự.

use std::fmt;

use keyring::Entry;

/// Tên dịch vụ trên keychain hệ điều hành — khớp `identifier` của `tauri.conf.json`, để một
/// entry của AuraTranslate không lẫn với ứng dụng khác trên cùng máy.
const KEYCHAIN_SERVICE: &str = "com.auratranslate.desktop";

/// Tên tài khoản — CỐ ĐỊNH, không tham số hoá theo Tác phẩm hay người dùng: đúng một entry
/// cho toàn ứng dụng (xem doc-comment đầu tệp §Global-only).
const KEYCHAIN_ACCOUNT: &str = "ai_provider_api_key";

/// Chuỗi thay thế khi hiển thị/log — KHÔNG BAO GIỜ là giá trị thật (§Always spec 4.3:
/// "no error message, parameter or log line interpolates the key or any prefix of it").
const REDACTED_PLACEHOLDER: &str = "<redacted>";

/// Giá trị khoá API — bọc một `String` mà `Debug`/`Display` đều CHE (§Always spec 4.3:
/// "Deriving `Debug` on it, or any struct holding it, is forbidden"). Không `derive(Debug)`
/// ở đây, và bất kỳ struct nào bọc kiểu này trong tương lai cũng không được `derive(Debug)`
/// — chỉ có cách viết tay mới ép người viết phải tự hỏi "trường này có được phép ra log
/// không".
pub struct ApiKeySecret(String);

impl ApiKeySecret {
    fn new(value: String) -> Self {
        Self(value)
    }

    /// Giá trị thô — xem cảnh báo ranh giới ở đầu tệp. `pub(crate)` để tầng lệnh CÓ THỂ
    /// biên dịch một chỗ gọi tương lai (Story 4.8) mà không phải sửa lại visibility ở đây;
    /// điều KHÔNG được phép là một chỗ gọi thật tồn tại hôm nay ngoài `core/aiconfig/**`.
    pub(crate) fn expose_secret(&self) -> &str {
        &self.0
    }
}

impl fmt::Debug for ApiKeySecret {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ApiKeySecret({REDACTED_PLACEHOLDER})")
    }
}

impl fmt::Display for ApiKeySecret {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(REDACTED_PLACEHOLDER)
    }
}

/// Keychain hệ điều hành từ chối trả lời — khoá, quyền bị chặn, hoặc không có kho nền tảng
/// nào khởi tạo được (`keyring::Error::NoDefaultStore` và mọi biến thể khác của
/// `keyring::Error` TRỪ `NoEntry`, vốn không phải một lỗi ở module này — xem [`delete`]/
/// [`configured`]/[`read`]). Domain này không phân biệt sâu hơn: I/O Matrix spec 4.3 chỉ
/// đòi "hành động thất bại và nói rõ, màn hình vẫn dùng được", không đòi một câu riêng cho
/// từng nguyên nhân nền tảng.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KeychainUnavailable;

impl fmt::Display for KeychainUnavailable {
    /// KHÔNG DẤU (NFR16) — chẩn đoán cho log, không phải văn bản hiển thị.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("aiconfig[keychain_unavailable]")
    }
}

impl std::error::Error for KeychainUnavailable {}

/// Mở entry keychain của ứng dụng. Lỗi ở đây gồm cả `NoDefaultStore` — nền tảng không khởi
/// tạo được kho nào (§Always spec 4.3: "a test that means to use the mock store must prove
/// it got it" nói về đúng biến thể này, ở phía test).
fn entry() -> Result<Entry, KeychainUnavailable> {
    Entry::new(KEYCHAIN_SERVICE, KEYCHAIN_ACCOUNT).map_err(|_| KeychainUnavailable)
}

/// Ghi (hoặc thay) giá trị khoá — chỗ gọi đã qua [`super::validate_key`] trước khi tới đây
/// (§Always spec 4.3: "rejected before any keychain call"), nên hàm này không kiểm tra lại
/// hình dạng giá trị. Ghi đè một entry đã có KHÔNG tạo entry thứ hai — đây là hành vi của
/// chính `keyring`/nền tảng bên dưới (`set_password` trên một account đã tồn tại thay giá
/// trị tại chỗ), I/O Matrix "Overwrite an existing key".
pub(crate) fn set(value: &str) -> Result<(), KeychainUnavailable> {
    entry()?
        .set_password(value)
        .map_err(|_| KeychainUnavailable)
}

/// Xoá entry — "xoá khi không có entry nào" là THÀNH CÔNG (I/O Matrix spec 4.3: "post-state
/// is the requested one"), cùng luật `store::clear_field` của năm trường kia.
pub(crate) fn delete() -> Result<(), KeychainUnavailable> {
    match entry()?.delete_credential() {
        Ok(()) => Ok(()),
        Err(keyring::Error::NoEntry) => Ok(()),
        Err(_) => Err(KeychainUnavailable),
    }
}

/// Trạng thái "đã cấu hình / chưa cấu hình" — KHÔNG BAO GIỜ trả giá trị (§Always spec 4.3:
/// "the frontend may learn only configured / not configured"). "Không có entry" không phải
/// lỗi — nó là I/O Matrix hàng đầu tiên, "not configured" là một trạng thái, không phải một
/// lỗi (đúng bất biến toàn epic, epic-4-context.md §UX).
pub(crate) fn configured() -> Result<bool, KeychainUnavailable> {
    match entry()?.get_password() {
        Ok(_) => Ok(true),
        Err(keyring::Error::NoEntry) => Ok(false),
        Err(_) => Err(KeychainUnavailable),
    }
}

/// Đọc giá trị thô, bọc trong [`ApiKeySecret`] — Story 4.8 (`TranslationProvider`, AD-2).
///
/// 🔵 **SỬA 2026-09-21 (Story 4.8, Phase 2) — `#[allow(dead_code)]` gỡ bỏ, chỗ gọi đã có
/// thật.** `commands::aitranslate::prepare_translate_call` gọi hàm này để đọc khoá TRƯỚC khi
/// dựng `TranslateRequest`, đúng lời câu cũ đã hứa ("Story 4.8 là chỗ gọi đầu tiên"). Giữ
/// `#[allow]` sau khi có chỗ gọi thật là một lời nói dối về đồ thị gọi (root `AGENTS.md`).
/// `None` khi chưa cấu hình (khác lỗi keychain từ chối trả lời), cùng phân biệt `configured`.
pub(crate) fn read() -> Result<Option<ApiKeySecret>, KeychainUnavailable> {
    match entry()?.get_password() {
        Ok(value) => Ok(Some(ApiKeySecret::new(value))),
        Err(keyring::Error::NoEntry) => Ok(None),
        Err(_) => Err(KeychainUnavailable),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Phép kiểm THUẦN, không chạm keychain thật: `Debug`/`Display` của [`ApiKeySecret`]
    /// khớp CHÍNH XÁC hình dạng đã che, không chỉ "không chứa chuỗi thô" (một phép so
    /// `contains` sẽ tự dối khi giá trị thô trùng tình cờ với chính chuỗi thay thế).
    #[test]
    fn api_key_secret_debug_and_display_are_exactly_the_redacted_shape() {
        let secret = ApiKeySecret::new("sk-thi-du-mot-khoa-that".to_owned());

        assert_eq!(format!("{secret}"), REDACTED_PLACEHOLDER);
        assert_eq!(
            format!("{secret:?}"),
            format!("ApiKeySecret({REDACTED_PLACEHOLDER})")
        );
        assert_eq!(secret.expose_secret(), "sk-thi-du-mot-khoa-that");
    }
}
