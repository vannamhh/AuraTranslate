//! Siêu dữ liệu tĩnh của năm nguồn — MỘT chỗ khai `code` · `display_name` ·
//! `license_*` · `attribution` · `source_url`, khớp khuôn `scope_kinds!`/`message_keys!`
//! của `src-tauri` (Dev Notes bàn giao #6 của Story 1.9): một khai báo, dùng ở nhiều
//! chỗ (chèn `dict_source`, README, và sau này màn hình Attribution — Story 10.4).
//!
//! `source_version` KHÔNG nằm ở đây — nó là dữ liệu ĐO ĐƯỢC lúc chạy (ngày dump / tag /
//! phiên bản Unicode), gắn vào lúc build (`build.rs`), không phải hằng biên dịch.

use crate::licenses;

/// Giấy phép nào nạp cho `license_text()` — khai TRỰC TIẾP trên từng hằng `SourceMeta`,
/// ⛔ không suy lại qua so khớp chuỗi `code` (Review Findings Group A: so khớp `code` cũ
/// có nhánh `unreachable!()` — một nguồn thứ sáu hoặc lỗi gõ `code` biến lỗi khai báo
/// thành panic lúc chạy thay vì lỗi biên dịch). `enum` đóng, `match` trong
/// `license_text()` vì vậy luôn TOÀN VẸN mà không cần nhánh dự phòng.
///
/// Hai biến thể cuối thuộc Story 1.10 (lớp gỡ rời) — thêm biến thể mới ở đây khi cần,
/// ⛔ KHÔNG suy ra tên biến thể bằng cách so khớp chuỗi.
#[derive(Clone, Copy)]
pub enum LicenseRef {
    CcBySa4,
    UnicodeV3,
    CcBySaAndGfdl,
    ThieuChuu,
    VietPhrase,
}

pub struct SourceMeta {
    pub code: &'static str,
    pub display_name: &'static str,
    pub license_kind: &'static str,
    /// `None` ⇒ cột `dict_source.license_id` là **`NULL`**, ⛔ không phải chuỗi rỗng.
    /// AC3 chốt cứng `NULL` cho `vietphrase`: *không có mã giấy phép mở nào áp dụng được*
    /// khác với *mã giấy phép là chuỗi rỗng* — màn hình Attribution (10.4) phải bỏ hẳn
    /// trường thay vì hiện một ô trống.
    pub license_id: Option<&'static str>,
    pub license_ref: LicenseRef,
    pub attribution: &'static str,
    pub source_url: &'static str,
}

impl SourceMeta {
    pub fn license_text(&self) -> String {
        match self.license_ref {
            LicenseRef::CcBySa4 => licenses::CC_BY_SA_4_0.to_string(),
            LicenseRef::UnicodeV3 => licenses::UNICODE_LICENSE_V3.to_string(),
            LicenseRef::CcBySaAndGfdl => licenses::cc_by_sa_and_gfdl(),
            LicenseRef::ThieuChuu => licenses::thieu_chuu_license_text(),
            LicenseRef::VietPhrase => licenses::vietphrase_license_text(),
        }
    }
}

pub const CVDICT: SourceMeta = SourceMeta {
    code: "cvdict",
    display_name: "CVDICT",
    license_kind: "open",
    license_id: Some("CC-BY-SA-4.0"),
    license_ref: LicenseRef::CcBySa4,
    attribution: "CVDICT (github.com/ph0ngp/CVDICT), phân phối theo CC BY-SA 4.0.",
    source_url: "https://github.com/ph0ngp/CVDICT",
};

pub const CC_CEDICT: SourceMeta = SourceMeta {
    code: "cc-cedict",
    display_name: "CC-CEDICT",
    license_kind: "open",
    license_id: Some("CC-BY-SA-4.0"),
    license_ref: LicenseRef::CcBySa4,
    attribution: "CC-CEDICT (mdbg.net/chinese/dictionary?page=cc-cedict), phân phối theo CC BY-SA 4.0.",
    source_url: "https://www.mdbg.net/chinese/dictionary?page=cc-cedict",
};

pub const UNIHAN: SourceMeta = SourceMeta {
    code: "unihan",
    display_name: "Unihan",
    license_kind: "open",
    license_id: Some("Unicode-3.0"),
    license_ref: LicenseRef::UnicodeV3,
    attribution: "Cơ sở dữ liệu Unihan, © Unicode, Inc. Phân phối theo Unicode License v3.",
    source_url: "https://www.unicode.org/reports/tr38/",
};

pub const VIWIKTIONARY: SourceMeta = SourceMeta {
    code: "viwiktionary",
    display_name: "Wiktionary tiếng Việt",
    license_kind: "open",
    license_id: Some("CC-BY-SA-4.0"),
    license_ref: LicenseRef::CcBySaAndGfdl,
    attribution: "Wiktionary tiếng Việt (vi.wiktionary.org), qua Wiktextract/kaikki.org. Phân phối theo CC BY-SA 4.0 và GFDL.",
    source_url: "https://kaikki.org/dictionary/downloads/vi/vi-extract.jsonl",
};

pub const EN_WIKTIONARY: SourceMeta = SourceMeta {
    code: "en-wiktionary",
    display_name: "Wiktionary tiếng Anh (mục tiếng Trung)",
    license_kind: "open",
    license_id: Some("CC-BY-SA-4.0"),
    license_ref: LicenseRef::CcBySaAndGfdl,
    attribution: "English Wiktionary (en.wiktionary.org), mục tiếng Trung, qua Wiktextract/kaikki.org. Phân phối theo CC BY-SA 4.0 và GFDL.",
    source_url: "https://kaikki.org/dictionary/Chinese/kaikki.org-dictionary-Chinese.jsonl",
};

/// Thiều Chửu (1942, Nguyễn Hữu Kha †1954) — lớp GỠ RỜI, Story 1.10.
/// `license_kind = "public-domain"` + `license_id = "CC0-1.0"`: tác phẩm gốc hết hạn
/// bảo hộ, VÀ bản số hoá phát hành CC0 1.0 (đã đối chiếu SHA-256 byte-for-byte với kho
/// gốc `catusf/tudien@2.2` — §Thông tin kỹ thuật). `attribution` nêu tên tác giả — nghĩa
/// vụ pháp lý theo quyền nhân thân vô thời hạn, ⛔ không phải phép lịch sự.
pub const THIEU_CHUU: SourceMeta = SourceMeta {
    code: "thieu-chuu",
    display_name: "Thiều Chửu — Hán Việt Tự Điển",
    license_kind: "public-domain",
    license_id: Some("CC0-1.0"),
    license_ref: LicenseRef::ThieuChuu,
    attribution: "Thiều Chửu (Nguyễn Hữu Kha, 1902–1954), Hán Việt Tự Điển (1942). Bản số hoá catusf/tudien@2.2, phát hành theo CC0 1.0 Universal.",
    source_url: "https://github.com/catusf/tudien",
};

/// VietPhrase — lớp GỠ RỜI, Story 1.10. `license_kind = "unknown"` — KHÔNG phải
/// `public-domain`: *không biết tác giả* ≠ *không có bản quyền* (AD-10, §Thông tin kỹ
/// thuật). `license_id = None` ⇒ cột là **`NULL`** (AC3), ⛔ không phải chuỗi rỗng —
/// không có mã giấy phép mở nào áp dụng được.
pub const VIETPHRASE: SourceMeta = SourceMeta {
    code: "vietphrase",
    display_name: "VietPhrase",
    license_kind: "unknown",
    license_id: None,
    license_ref: LicenseRef::VietPhrase,
    attribution: "Dữ liệu cộng đồng VietPhrase (github.com/truyencuatui/VietPhrase), không xác định được tác giả gốc.",
    source_url: "https://github.com/truyencuatui/VietPhrase",
};

/// Đúng NĂM nguồn NỀN của Story 1.9, đúng thứ tự chèn — ⛔ không hơn không kém (Bẫy 10).
/// Đổi tên từ `ALL` (Story 1.9) → `BASE_ALL` (Story 1.10) khi tách hai danh sách; nội
/// dung ⛔ giữ nguyên.
pub const BASE_ALL: [&SourceMeta; 5] = [&CVDICT, &CC_CEDICT, &UNIHAN, &VIWIKTIONARY, &EN_WIKTIONARY];

/// Đúng HAI lớp gỡ rời trong phạm vi Story 1.10 (Ice chốt 2026-08-05). HVTĐTD và Cổ hán
/// văn chuyển sang story nối tiếp — xem `deferred-work.md`. ⛔ KHÔNG dựng tệp `.db` rỗng
/// cho hai lớp đó; chúng CHƯA TỒN TẠI trong bảng phân phối, không phải "tồn tại nhưng
/// thiếu dữ liệu" (§Bẫy 7).
pub const DETACHABLE_ALL: [&SourceMeta; 2] = [&THIEU_CHUU, &VIETPHRASE];

#[cfg(test)]
mod tests {
    use super::*;

    /// Bẫy 10 / Bẫy 4: Thiều Chửu · Cổ hán văn · VietPhrase · HVTĐTD KHÔNG thuộc
    /// `BASE_ALL` — chúng là lớp gỡ rời (Story 1.10; Thiều Chửu + VietPhrase giao ở
    /// story này, hai lớp còn lại ở story nối tiếp). Test này khoá đúng năm mã nguồn NỀN
    /// của `epics.md`.
    #[test]
    fn exactly_five_sources_with_the_epics_md_codes() {
        assert_eq!(BASE_ALL.len(), 5);
        let codes: Vec<&str> = BASE_ALL.iter().map(|s| s.code).collect();
        assert_eq!(
            codes,
            vec!["cvdict", "cc-cedict", "unihan", "viwiktionary", "en-wiktionary"]
        );
    }

    /// Story 1.10: đúng hai lớp gỡ rời trong phạm vi hôm nay. HVTĐTD + Cổ hán văn chuyển
    /// sang story nối tiếp — chưa có nguồn thô (HVTĐTD phải xin trực tiếp tác giả; Cổ
    /// hán văn cần quyết lại "nó là lớp gì" trước khi đi tìm tệp). Xem `deferred-work.md`.
    #[test]
    fn exactly_two_detachable_sources_in_scope_today() {
        assert_eq!(DETACHABLE_ALL.len(), 2);
        let codes: Vec<&str> = DETACHABLE_ALL.iter().map(|s| s.code).collect();
        assert_eq!(codes, vec!["thieu-chuu", "vietphrase"]);
    }

    /// AC1: không mã nguồn nào xuất hiện ở CẢ HAI danh sách — điều kiện cấu trúc để
    /// `dict-core.db` và các tệp lớp gỡ rời không lẫn nguồn của nhau.
    #[test]
    fn base_and_detachable_code_sets_are_disjoint() {
        let base_codes: std::collections::HashSet<&str> =
            BASE_ALL.iter().map(|s| s.code).collect();
        let detachable_codes: std::collections::HashSet<&str> =
            DETACHABLE_ALL.iter().map(|s| s.code).collect();
        assert!(
            base_codes.is_disjoint(&detachable_codes),
            "BASE_ALL and DETACHABLE_ALL must not share any code, got base={base_codes:?} detachable={detachable_codes:?}"
        );
    }

    #[test]
    fn every_source_declares_a_non_empty_license_text() {
        for s in BASE_ALL.iter().chain(DETACHABLE_ALL.iter()) {
            assert!(!s.license_text().is_empty(), "{} has empty license_text", s.code);
        }
    }

    /// AC3 đối chứng âm: `vietphrase` KHÔNG được gán `public-domain` — *không biết tác
    /// giả* ≠ *không có bản quyền*. Đây là lỗi gán nhãn dễ mắc nhất của story này.
    #[test]
    fn vietphrase_is_unknown_not_public_domain() {
        assert_eq!(VIETPHRASE.license_kind, "unknown");
        assert_ne!(VIETPHRASE.license_kind, "public-domain");
    }

    /// AC3: bảng chốt cứng `license_id = NULL` cho `vietphrase` — ⛔ không phải chuỗi
    /// rỗng. `None` và `Some("")` là hai giá trị KHÁC nhau ở đường đọc 1.11/1.13.
    #[test]
    fn vietphrase_declares_no_license_id_at_all() {
        assert_eq!(VIETPHRASE.license_id, None);
    }

    /// AC3: Thiều Chửu mang đúng cặp `public-domain` + `CC0-1.0`.
    #[test]
    fn thieu_chuu_is_public_domain_with_the_cc0_identifier() {
        assert_eq!(THIEU_CHUU.license_kind, "public-domain");
        assert_eq!(THIEU_CHUU.license_id, Some("CC0-1.0"));
    }

    /// AC2: attribution của Thiều Chửu nêu đích danh tên tác giả — nghĩa vụ quyền nhân
    /// thân, ⛔ không phải phép lịch sự.
    #[test]
    fn thieu_chuu_attribution_names_the_author() {
        assert!(THIEU_CHUU.attribution.contains("Thiều Chửu"));
        assert!(THIEU_CHUU.attribution.contains("Nguyễn Hữu Kha"));
    }
}
