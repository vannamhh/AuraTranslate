//! Cấu hình nhà cung cấp AI — Story 4.2, FR68, `core/scope/kinds.rs:175`
//! (`ScopeKind::AiConfig => "ai_config" : Semantics::Override`).
//!
//! ─────────────────────────────────────────────────────────────────────────────
//! 🔴 NGOÀI `core/ai/`, VÀ KHÔNG GÕ TÊN NÓ — AD-13
//! ─────────────────────────────────────────────────────────────────────────────
//! Đây là một domain CẤU HÌNH hai tầng, cùng hình dạng `core/cleanup/`/`core/glossary/`,
//! không phải logic AI: nó không gọi ra ngoài, không biết `TranslationProvider` (chưa được
//! khai — AD-2), và không tên `crate::core::ai`/`super::ai` ở bất kỳ đâu trong cây con này
//! (`tests/ai_boundary.rs` canh mệnh đề đó trên toàn `src-tauri/src/**` trừ `core/ai/**`).
//!
//! ─────────────────────────────────────────────────────────────────────────────
//! 🔴 GHI ĐÈ THEO TỪNG TRƯỜNG — KHÔNG PHẢI CẢ STRUCT
//! ─────────────────────────────────────────────────────────────────────────────
//! Ice ký 2026-08-04: một Tác phẩm ghi đè `endpoint` không được che mất `provider`/`model`/
//! `temperature`/`max_tokens` đang kế thừa từ Global. Vì thế bảng lưu là `(key, value)` —
//! cùng hình dạng `config_value`, nhưng RIÊNG (Override cần một bảng tồn tại ở CẢ HAI kho,
//! còn `config_value` chỉ phục vụ ba loại `GlobalOnly` — xem doc-comment
//! `core::store::schema::CONFIG_VALUE_DDL`) — và [`store::resolve_two_tiers`] phân giải qua
//! `ScopeResolver::apply_override`, đúng khuôn `core::glossary::store::load_tier` +
//! `apply_override`, không khuôn `core::cleanup::store` (đó là `Merge`).
//!
//! ─────────────────────────────────────────────────────────────────────────────
//! GIÁ TRỊ LUÔN LÀ CHUỖI — cùng mọi bảng cấu hình khác của dự án
//! ─────────────────────────────────────────────────────────────────────────────
//! `temperature`/`max_tokens` đi qua kiểm tra bằng một hàm THUẦN riêng ([`validate_field`])
//! TRƯỚC khi chạm SQL — không `CHECK` ở tầng DDL, vì `ai_config.value` phục vụ NĂM khoá khác
//! hình dạng nhau (chuỗi tự do, số thực có khoảng, số nguyên dương) và một `CHECK` chung cho
//! cả năm không biểu diễn được gì hữu ích.

pub mod store;

/// Bí mật khoá API — keychain hệ điều hành, Global-only. `pub(crate)`: tầng lệnh
/// (`commands::aiconfig`) gọi qua đây; xem cảnh báo ranh giới ở đầu `keychain.rs` cho lý do
/// bên trong module đó vẫn còn hẹp hơn.
pub(crate) mod keychain;

use std::fmt;

pub use store::{
    AiConfigKeyError, AiConfigStoreError, ResolvedField, clear_field, resolve_two_tiers,
    write_field,
};

/// Năm trường của cấu hình AI — FR68 ("nhà cung cấp, mô hình, tham số sinh"). Không trường
/// khoá API: FR65/NFR11 đặt khoá trong keychain hệ điều hành, đó là Story 4.3 (§Intent).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Deserialize)]
pub enum AiConfigField {
    #[serde(rename = "provider")]
    Provider,
    #[serde(rename = "endpoint")]
    Endpoint,
    #[serde(rename = "model")]
    Model,
    #[serde(rename = "temperature")]
    Temperature,
    #[serde(rename = "max_tokens")]
    MaxTokens,
}

impl AiConfigField {
    /// Mọi trường — thứ tự này là thứ tự hiển thị của form (`mockups/settings.html`).
    pub const ALL: &'static [AiConfigField] = &[
        AiConfigField::Provider,
        AiConfigField::Endpoint,
        AiConfigField::Model,
        AiConfigField::Temperature,
        AiConfigField::MaxTokens,
    ];

    /// Định danh máy đọc — khoá `ai_config.key` trên đĩa và trên dây. Không phải nhãn hiển
    /// thị (AD-21, NFR16).
    pub const fn as_str(self) -> &'static str {
        match self {
            AiConfigField::Provider => "provider",
            AiConfigField::Endpoint => "endpoint",
            AiConfigField::Model => "model",
            AiConfigField::Temperature => "temperature",
            AiConfigField::MaxTokens => "max_tokens",
        }
    }
}

impl fmt::Display for AiConfigField {
    /// KHÔNG DẤU — chẩn đoán cho log, không phải văn bản hiển thị (NFR16).
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Nhãn tầng của MỘT trường — kiểu RIÊNG, không tái dùng `core::scope::Tier` (cùng lý lẽ
/// `core::cleanup::CleanupRuleTier`: tệp này còn đi tiếp một chặng — tham số lệnh
/// `commands::aiconfig` — mà `Tier` không đi).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Deserialize)]
pub enum AiConfigTier {
    #[serde(rename = "global")]
    Global,
    #[serde(rename = "work")]
    Work,
}

impl AiConfigTier {
    pub const fn as_str(self) -> &'static str {
        match self {
            AiConfigTier::Global => "global",
            AiConfigTier::Work => "work",
        }
    }

    /// Phân giải một giá trị đến từ `core::scope::Tier::as_str()` — chỗ DUY NHẤT module này
    /// đọc chuỗi đó, để `commands::aiconfig` không bao giờ phải gõ tên `core::scope::Tier`
    /// (cùng kỷ luật `core::cleanup::store::tier_from_scope_wire`, "siết hơn" tiền lệ
    /// `core::glossary::store`). `None` không thể xảy ra trên đường gọi đúng — chỉ hai giá
    /// trị `"global"`/`"work"` từng phát ra từ `core::scope`.
    pub fn from_wire(raw: &str) -> Option<Self> {
        match raw {
            "global" => Some(AiConfigTier::Global),
            "work" => Some(AiConfigTier::Work),
            _ => None,
        }
    }
}

impl fmt::Display for AiConfigTier {
    /// KHÔNG DẤU — chẩn đoán cho log, không phải văn bản hiển thị (NFR16).
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Một trường không qua được kiểm tra của chính nó — trả TRƯỚC khi bất cứ giao dịch nào mở
/// (§Always spec 4.2: "một giá trị không hợp lệ bị từ chối trước bất kỳ lượt ghi nào").
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InvalidFieldValue {
    pub field: AiConfigField,
}

/// Khoá API không qua được kiểm tra của chính nó — rỗng hoặc chỉ toàn khoảng trắng
/// (§Always spec 4.3: từ chối TRƯỚC khi chạm keychain). KHÔNG mang `field`: khoá không
/// phải một biến thể của [`AiConfigField`] (§Design Notes spec 4.3 — "vì sao khoá không
/// gia nhập `AiConfigField`"), và không mang GIÁ TRỊ: một lỗi đi ngang IPC không bao giờ
/// được phép cõng theo chuỗi khoá hay một phần của nó.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InvalidKeyValue;

/// `temperature` — số thực hữu hạn trong `[0.0, 2.0]`. `"-1"`/`"3"`/`"abc"` đều bị từ chối
/// (I/O Matrix spec 4.2). Trần trên 2.0 khớp giới hạn tham số sinh phổ biến nhất của các nhà
/// cung cấp tương thích kiểu OpenAI — không có FR/mockup nào khai một trần khác, và một
/// trần RỘNG hơn thực tế mọi nhà cung cấp chấp nhận không bảo vệ được gì.
fn validate_temperature(raw: &str) -> Result<String, ()> {
    let trimmed = raw.trim();
    let parsed: f64 = trimmed.parse().map_err(|_| ())?;
    if !parsed.is_finite() || !(0.0..=2.0).contains(&parsed) {
        return Err(());
    }
    Ok(trimmed.to_owned())
}

/// `max_tokens` — số nguyên dương (`u32`, `> 0`). `"0"`/`"-5"`/`"1.5"`/`"abc"` đều bị từ chối
/// (I/O Matrix spec 4.2) — `parse::<u32>()` tự chặn dấu trừ và phần thập phân.
fn validate_max_tokens(raw: &str) -> Result<String, ()> {
    let trimmed = raw.trim();
    match trimmed.parse::<u32>() {
        Ok(0) | Err(_) => Err(()),
        Ok(_) => Ok(trimmed.to_owned()),
    }
}

/// `endpoint` — một URL TUYỆT ĐỐI, lược đồ `http`/`https`, mang một host. `"localhost:11434"`
/// (không lược đồ) và chuỗi rỗng đều bị từ chối (I/O Matrix spec 4.2).
///
/// ─────────────────────────────────────────────────────────────────────────────
/// 🔴 TỰ TAY PHÂN TÍCH — KHÔNG `reqwest::Url::parse`
/// ─────────────────────────────────────────────────────────────────────────────
/// `tests/webimport_boundary.rs::reqwest_is_named_only_inside_core_webimport_or_core_ai`
/// khoá token `reqwest` chỉ được phép xuất hiện trong `core/webimport/`/`core/ai/` — đúng
/// điểm ra mạng thứ ba của NFR12, và cổng đó KHÔNG phân biệt "chỉ phân tích cú pháp" với
/// "gọi mạng thật", nó cấm chính CÁI TÊN. Module này không phải `core/webimport/`/`core/ai/`
/// (§Never spec 4.2: "no file in this story names crate::core::ai"), nên nó không được
/// dùng `reqwest::Url` dù chỉ để phân tích chuỗi. Rào tự viết dưới đây đủ cho hình dạng cần
/// kiểm: một lược đồ `http`/`https` cộng một host không rỗng, không khoảng trắng.
fn validate_endpoint(raw: &str) -> Result<String, ()> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Err(());
    }

    let after_scheme =
        trimmed.strip_prefix("https://").or_else(|| trimmed.strip_prefix("http://"));
    let Some(rest) = after_scheme else {
        return Err(());
    };

    let host_end = rest.find(['/', '?', '#']).unwrap_or(rest.len());
    let host = &rest[..host_end];
    if host.is_empty() || host.chars().any(char::is_whitespace) {
        return Err(());
    }

    Ok(trimmed.to_owned())
}

/// `provider`/`model` — văn bản tự do, không rỗng sau khi trim. Không danh sách nhà cung cấp
/// đóng ở đây: `TranslationProvider` chưa được khai (AD-2, §Never spec 4.2), nên không có
/// tập hợp lệ nào để mà `CHECK`.
fn validate_non_empty(raw: &str) -> Result<String, ()> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Err(());
    }
    Ok(trimmed.to_owned())
}

/// Một hàm parse/validate THUẦN cho mỗi trường (§Code Map spec 4.2) — gọi được thẳng từ test
/// mà không cần `Store`. Trả giá trị đã trim (dạng ghi xuống đĩa) hoặc [`InvalidFieldValue`].
pub fn validate_field(field: AiConfigField, raw: &str) -> Result<String, InvalidFieldValue> {
    let result = match field {
        AiConfigField::Provider | AiConfigField::Model => validate_non_empty(raw),
        AiConfigField::Endpoint => validate_endpoint(raw),
        AiConfigField::Temperature => validate_temperature(raw),
        AiConfigField::MaxTokens => validate_max_tokens(raw),
    };
    result.map_err(|()| InvalidFieldValue { field })
}

/// Khoá API — văn bản tự do, không rỗng sau khi trim (I/O Matrix spec 4.3: "Save an empty
/// or whitespace-only key"). Không hình dạng nào khác bị áp: mỗi nhà cung cấp đặt ra một
/// định dạng khoá riêng của họ, và `TranslationProvider` (AD-2) CHƯA được khai (§Never spec
/// 4.3) nên không có tập giá trị hợp lệ nào để mà `CHECK` sâu hơn — cùng lý lẽ
/// [`validate_non_empty`] đã ghi cho `provider`/`model`.
///
/// Trả giá trị đã trim, cùng khuôn [`validate_field`] — đó là hình dạng thứ đi xuống
/// keychain, không phải chuỗi thô người dùng gõ.
pub fn validate_key(raw: &str) -> Result<String, InvalidKeyValue> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Err(InvalidKeyValue);
    }
    Ok(trimmed.to_owned())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn temperature_accepts_the_closed_boundaries_and_rejects_outside_them() {
        assert!(validate_field(AiConfigField::Temperature, "0").is_ok());
        assert!(validate_field(AiConfigField::Temperature, "2").is_ok());
        assert!(validate_field(AiConfigField::Temperature, "0.7").is_ok());
        assert!(validate_field(AiConfigField::Temperature, "-1").is_err());
        assert!(validate_field(AiConfigField::Temperature, "3").is_err());
        assert!(validate_field(AiConfigField::Temperature, "abc").is_err());
    }

    #[test]
    fn max_tokens_rejects_zero_negative_fractional_and_non_numeric() {
        assert!(validate_field(AiConfigField::MaxTokens, "1").is_ok());
        assert!(validate_field(AiConfigField::MaxTokens, "0").is_err());
        assert!(validate_field(AiConfigField::MaxTokens, "-5").is_err());
        assert!(validate_field(AiConfigField::MaxTokens, "1.5").is_err());
        assert!(validate_field(AiConfigField::MaxTokens, "abc").is_err());
    }

    #[test]
    fn endpoint_rejects_a_schemeless_host_and_an_empty_string() {
        assert!(validate_field(AiConfigField::Endpoint, "https://api.example.com/v1").is_ok());
        assert!(validate_field(AiConfigField::Endpoint, "http://localhost:11434").is_ok());
        assert!(validate_field(AiConfigField::Endpoint, "localhost:11434").is_err());
        assert!(validate_field(AiConfigField::Endpoint, "").is_err());
    }

    #[test]
    fn provider_and_model_reject_blank_input() {
        assert!(validate_field(AiConfigField::Provider, "anthropic").is_ok());
        assert!(validate_field(AiConfigField::Provider, "   ").is_err());
        assert!(validate_field(AiConfigField::Model, "").is_err());
    }

    #[test]
    fn key_rejects_empty_and_whitespace_only_but_trims_a_real_value() {
        assert_eq!(validate_key("sk-abc123"), Ok("sk-abc123".to_owned()));
        assert_eq!(validate_key("  sk-abc123  "), Ok("sk-abc123".to_owned()));
        assert_eq!(validate_key(""), Err(InvalidKeyValue));
        assert_eq!(validate_key("   "), Err(InvalidKeyValue));
    }
}
