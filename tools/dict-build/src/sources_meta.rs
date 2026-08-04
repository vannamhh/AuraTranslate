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
#[derive(Clone, Copy)]
pub enum LicenseRef {
    CcBySa4,
    UnicodeV3,
    CcBySaAndGfdl,
}

pub struct SourceMeta {
    pub code: &'static str,
    pub display_name: &'static str,
    pub license_kind: &'static str,
    pub license_id: &'static str,
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
        }
    }
}

pub const CVDICT: SourceMeta = SourceMeta {
    code: "cvdict",
    display_name: "CVDICT",
    license_kind: "open",
    license_id: "CC-BY-SA-4.0",
    license_ref: LicenseRef::CcBySa4,
    attribution: "CVDICT (github.com/ph0ngp/CVDICT), phân phối theo CC BY-SA 4.0.",
    source_url: "https://github.com/ph0ngp/CVDICT",
};

pub const CC_CEDICT: SourceMeta = SourceMeta {
    code: "cc-cedict",
    display_name: "CC-CEDICT",
    license_kind: "open",
    license_id: "CC-BY-SA-4.0",
    license_ref: LicenseRef::CcBySa4,
    attribution: "CC-CEDICT (mdbg.net/chinese/dictionary?page=cc-cedict), phân phối theo CC BY-SA 4.0.",
    source_url: "https://www.mdbg.net/chinese/dictionary?page=cc-cedict",
};

pub const UNIHAN: SourceMeta = SourceMeta {
    code: "unihan",
    display_name: "Unihan",
    license_kind: "open",
    license_id: "Unicode-3.0",
    license_ref: LicenseRef::UnicodeV3,
    attribution: "Cơ sở dữ liệu Unihan, © Unicode, Inc. Phân phối theo Unicode License v3.",
    source_url: "https://www.unicode.org/reports/tr38/",
};

pub const VIWIKTIONARY: SourceMeta = SourceMeta {
    code: "viwiktionary",
    display_name: "Wiktionary tiếng Việt",
    license_kind: "open",
    license_id: "CC-BY-SA-4.0",
    license_ref: LicenseRef::CcBySaAndGfdl,
    attribution: "Wiktionary tiếng Việt (vi.wiktionary.org), qua Wiktextract/kaikki.org. Phân phối theo CC BY-SA 4.0 và GFDL.",
    source_url: "https://kaikki.org/dictionary/downloads/vi/vi-extract.jsonl",
};

pub const EN_WIKTIONARY: SourceMeta = SourceMeta {
    code: "en-wiktionary",
    display_name: "Wiktionary tiếng Anh (mục tiếng Trung)",
    license_kind: "open",
    license_id: "CC-BY-SA-4.0",
    license_ref: LicenseRef::CcBySaAndGfdl,
    attribution: "English Wiktionary (en.wiktionary.org), mục tiếng Trung, qua Wiktextract/kaikki.org. Phân phối theo CC BY-SA 4.0 và GFDL.",
    source_url: "https://kaikki.org/dictionary/Chinese/kaikki.org-dictionary-Chinese.jsonl",
};

/// Đúng NĂM nguồn của story này, đúng thứ tự chèn — ⛔ không hơn không kém (Bẫy 10).
pub const ALL: [&SourceMeta; 5] = [&CVDICT, &CC_CEDICT, &UNIHAN, &VIWIKTIONARY, &EN_WIKTIONARY];

#[cfg(test)]
mod tests {
    use super::*;

    /// Bẫy 10: Thiều Chửu · Cổ hán văn · VietPhrase · HVTĐTD KHÔNG thuộc story này —
    /// chúng là lớp gỡ rời (Story 1.10). Test này khoá đúng năm mã nguồn của `epics.md`.
    #[test]
    fn exactly_five_sources_with_the_epics_md_codes() {
        assert_eq!(ALL.len(), 5);
        let codes: Vec<&str> = ALL.iter().map(|s| s.code).collect();
        assert_eq!(
            codes,
            vec!["cvdict", "cc-cedict", "unihan", "viwiktionary", "en-wiktionary"]
        );
    }

    #[test]
    fn every_source_declares_a_non_empty_license_text() {
        for s in ALL {
            assert!(!s.license_text().is_empty(), "{} has empty license_text", s.code);
        }
    }
}
