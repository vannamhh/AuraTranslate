//! Siêu dữ liệu tĩnh của mọi nguồn — MỘT chỗ khai `code` · `display_name` ·
//! `license_*` · `attribution` · `source_url`, khớp khuôn `scope_kinds!`/`message_keys!`
//! của `src-tauri` (Dev Notes bàn giao #6 của Story 1.9): một khai báo, dùng ở nhiều
//! chỗ (chèn `dict_source`, README, và sau này màn hình Attribution — Story 10.4).
//!
//! `source_version` KHÔNG nằm ở đây — nó là dữ liệu ĐO ĐƯỢC lúc chạy (ngày dump / tag /
//! phiên bản Unicode), gắn vào lúc build (`build.rs`), không phải hằng biên dịch.

use crate::licenses;

/// Giấy phép nào nạp cho `license_text()` — khai TRỰC TIẾP trên từng hằng `SourceMeta`,
/// không suy lại qua so khớp chuỗi `code` (Review Findings Group A: so khớp `code` cũ
/// có nhánh `unreachable!()` — một nguồn thứ sáu hoặc lỗi gõ `code` biến lỗi khai báo
/// thành panic lúc chạy thay vì lỗi biên dịch). `enum` đóng, `match` trong
/// `license_text()` vì vậy luôn TOÀN VẸN mà không cần nhánh dự phòng.
///
/// Hai biến thể `ThieuChuu`/`VietPhrase` thuộc Story 1.10 (lớp gỡ rời); `TranVanChanh`
/// thuộc Story 1.10c (lớp gỡ rời thứ ba) — thêm biến thể mới ở đây khi cần, KHÔNG suy
/// ra tên biến thể bằng cách so khớp chuỗi.
#[derive(Clone, Copy)]
pub enum LicenseRef {
    CcBySa4,
    UnicodeV3,
    CcBySaAndGfdl,
    ThieuChuu,
    VietPhrase,
    TranVanChanh,
}

pub struct SourceMeta {
    pub code: &'static str,
    pub display_name: &'static str,
    pub license_kind: &'static str,
    /// `None` ⇒ cột `dict_source.license_id` là **`NULL`**, không phải chuỗi rỗng.
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
            LicenseRef::TranVanChanh => licenses::tran_van_chanh_license_text(),
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

/// viwiktionary **VAI A** — mục từ TIẾNG ANH của cùng tệp thô mà [`VIWIKTIONARY`] (vai
/// B) đọc. Nguồn NỀN thứ sáu, thêm ở Story 1.10b để FR34 có dữ liệu.
///
/// 🔴 `license_ref` dùng LẠI [`LicenseRef::CcBySaAndGfdl`] của vai B — **không** thêm
/// biến thể `enum` mới. Cùng tệp `vi-extract.jsonl`, cùng kho kaikki.org, cùng
/// CC-BY-SA 4.0 + GFDL 1.3: một biến thể thứ hai chỉ nhân đôi cùng một văn bản và tạo
/// chỗ cho hai bản trôi khỏi nhau.
///
/// ⚠️ `display_name` **phải khác** vai B. Màn hình Attribution (Story 10.4) liệt kê cả
/// sáu nguồn nền; hai dòng "Wiktionary tiếng Việt" giống hệt nhau là lỗi hiển thị.
pub const VIWIKTIONARY_EN: SourceMeta = SourceMeta {
    code: "viwiktionary-en",
    display_name: "Wiktionary tiếng Việt (mục tiếng Anh)",
    license_kind: "open",
    license_id: Some("CC-BY-SA-4.0"),
    license_ref: LicenseRef::CcBySaAndGfdl,
    attribution: "Wiktionary tiếng Việt (vi.wiktionary.org), mục tiếng Anh, qua Wiktextract/kaikki.org. Phân phối theo CC BY-SA 4.0 và GFDL.",
    source_url: "https://kaikki.org/dictionary/downloads/vi/vi-extract.jsonl",
};

/// en.wiktionary — extract **Vietnamese** (`lang_code = "vi"`), qua Wiktextract/
/// kaikki.org. Nguồn NỀN thứ bảy, Story 1.10c — LƯỚI chống tái diễn lỗi Unihan (AC5),
/// không một nguồn nghĩa (Quyết định #3a: chỉ nạp âm đọc gắn nhãn, không
/// `dict_sense`).
///
/// 🔴 `license_ref` dùng LẠI [`LicenseRef::CcBySaAndGfdl`] — cùng kho kaikki.org/
/// Wiktextract, cùng CC-BY-SA 4.0 + GFDL 1.3 như ba nguồn Wiktextract khác đã có, không
/// không thêm biến thể `enum` mới cho cùng một cặp giấy phép.
pub const EN_WIKTIONARY_VI: SourceMeta = SourceMeta {
    code: "en-wiktionary-vi",
    display_name: "Wiktionary tiếng Anh (mục tiếng Việt — âm Hán Việt/Nôm gắn nhãn)",
    license_kind: "open",
    license_id: Some("CC-BY-SA-4.0"),
    license_ref: LicenseRef::CcBySaAndGfdl,
    attribution: "English Wiktionary (en.wiktionary.org), mục tiếng Việt, qua Wiktextract/kaikki.org. Phân phối theo CC BY-SA 4.0 và GFDL.",
    source_url: "https://kaikki.org/dictionary/Vietnamese/kaikki.org-dictionary-Vietnamese.jsonl",
};

/// Thiều Chửu (1942, Nguyễn Hữu Kha †1954) — lớp GỠ RỜI, Story 1.10.
/// `license_kind = "public-domain"` + `license_id = "CC0-1.0"`: tác phẩm gốc hết hạn
/// bảo hộ, VÀ bản số hoá phát hành CC0 1.0 (đã đối chiếu SHA-256 byte-for-byte với kho
/// gốc `catusf/tudien@2.2` — §Thông tin kỹ thuật). `attribution` nêu tên tác giả — nghĩa
/// vụ pháp lý theo quyền nhân thân vô thời hạn, không phải phép lịch sự.
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
/// thuật). `license_id = None` ⇒ cột là **`NULL`** (AC3), không phải chuỗi rỗng —
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

/// Trần Văn Chánh — lớp GỠ RỜI thứ ba, Story 1.10c (Quyết định #1, Ice chốt 2026-08-06).
///
/// 🔴 `license_kind = "copyrighted"` — KHÔNG `"public-domain"`, KHÔNG `"unknown"`: khác
/// VietPhrase (tác giả KHÔNG xác định được), ở đây tác giả **xác định được** (Trần Văn
/// Chánh) và tác phẩm **còn trong bản quyền** — CC0 của người số hoá `catusf/tudien`
/// KHÔNG xoá được bản quyền tác phẩm gốc (AC8, xem `assets/licenses/tran-van-chanh.txt`).
/// `license_id = None` ⇒ cột là `NULL` — không có mã giấy phép mở nào áp dụng được
/// cho TOÀN BỘ nội dung, cùng lý do `VIETPHRASE.license_id`.
pub const TRAN_VAN_CHANH: SourceMeta = SourceMeta {
    code: "tran-van-chanh",
    display_name: "Trần Văn Chánh — Từ điển Hán Việt",
    license_kind: "copyrighted",
    license_id: None,
    license_ref: LicenseRef::TranVanChanh,
    attribution: "Trần Văn Chánh, Từ điển Hán Việt (1999) — CÒN TRONG BẢN QUYỀN, tác giả còn sống. Bản số hoá catusf/tudien khai CC0 1.0 cho công sức số hoá; điều đó KHÔNG xoá bản quyền tác phẩm gốc. Đóng gói làm lớp gỡ rời (FR112) — xem assets/licenses/tran-van-chanh.txt.",
    source_url: "https://github.com/catusf/tudien",
};

/// Đúng BẢY nguồn NỀN, đúng thứ tự chèn — không hơn không kém (Bẫy 10).
/// Đổi tên từ `ALL` (Story 1.9) → `BASE_ALL` (Story 1.10) khi tách hai danh sách.
///
/// 🔴 **Thứ tự chèn = thứ tự `dict_source.id`.** `viwiktionary-en` (Story 1.10b) và
/// `en-wiktionary-vi` (Story 1.10c) thêm vào **CUỐI**, theo đúng thứ tự thêm — để các
/// nguồn cũ giữ nguyên `id` của chúng. Chèn một nguồn mới vào giữa "cho gọn" làm mọi
/// `id` sau nó dịch đi: vô hại hôm nay (mọi FK nằm trong cùng tệp, cùng lượt dựng),
/// nhưng nó khiến hai lượt dựng khác nhau ra hai bảng `id` khác nhau mà không đổi lại
/// được gì.
pub const BASE_ALL: [&SourceMeta; 7] = [
    &CVDICT,
    &CC_CEDICT,
    &UNIHAN,
    &VIWIKTIONARY,
    &EN_WIKTIONARY,
    &VIWIKTIONARY_EN,
    &EN_WIKTIONARY_VI,
];

/// Đúng BA lớp gỡ rời trong phạm vi Story 1.10c (Ice chốt 2026-08-06: `tran-van-chanh`
/// thêm vào cuối). HVTĐTD và Cổ hán văn vẫn chưa có nguồn thô — xem `deferred-work.md`.
/// KHÔNG dựng tệp `.db` rỗng cho hai lớp đó; chúng CHƯA TỒN TẠI trong bảng phân phối,
/// không phải "tồn tại nhưng thiếu dữ liệu" (§Bẫy 7).
pub const DETACHABLE_ALL: [&SourceMeta; 3] = [&THIEU_CHUU, &VIETPHRASE, &TRAN_VAN_CHANH];

#[cfg(test)]
mod tests {
    use super::*;

    /// Bẫy 10 / Bẫy 4: Thiều Chửu · Cổ hán văn · VietPhrase · HVTĐTD KHÔNG thuộc
    /// `BASE_ALL` — chúng là lớp gỡ rời (Story 1.10; Thiều Chửu + VietPhrase giao ở
    /// story này, hai lớp còn lại ở story nối tiếp). Test này khoá đúng các mã nguồn NỀN
    /// của `epics.md`.
    ///
    /// ⚠️ Story 1.10b nâng 5 → 6 vì nó thêm một nguồn **NỀN** thật (`viwiktionary-en`,
    /// vai A của cùng tệp thô), **không** vì nó kéo một lớp gỡ rời vào `BASE_ALL`.
    /// Mệnh đề mà test này khoá — bốn lớp gỡ rời KHÔNG thuộc `BASE_ALL` — không đổi.
    #[test]
    fn exactly_seven_sources_with_the_epics_md_codes() {
        assert_eq!(BASE_ALL.len(), 7);
        let codes: Vec<&str> = BASE_ALL.iter().map(|s| s.code).collect();
        assert_eq!(
            codes,
            vec![
                "cvdict",
                "cc-cedict",
                "unihan",
                "viwiktionary",
                "en-wiktionary",
                "viwiktionary-en",
                "en-wiktionary-vi",
            ]
        );
    }

    /// 🔴 AC4: hai vai của cùng một tệp thô là **hai nguồn phân biệt được** — khác `code`
    /// (khoá máy) VÀ khác `display_name` (nhãn người đọc). Màn hình Attribution của Story
    /// 10.4 liệt kê cả hai; hai dòng "Wiktionary tiếng Việt" giống hệt nhau là một lỗi
    /// hiển thị, không phải chi tiết thẩm mỹ.
    #[test]
    fn viwiktionary_and_viwiktionary_en_are_two_distinct_sources() {
        assert_ne!(VIWIKTIONARY.code, VIWIKTIONARY_EN.code);
        assert_ne!(VIWIKTIONARY.display_name, VIWIKTIONARY_EN.display_name);
        assert_eq!(VIWIKTIONARY.code, "viwiktionary");
        assert_eq!(VIWIKTIONARY_EN.code, "viwiktionary-en");
    }

    /// AC4: `attribution` của vai A phải nêu rõ **mục tiếng Anh**, nếu không màn
    /// Attribution không phân biệt được nó với vai B.
    #[test]
    fn viwiktionary_en_attribution_says_it_is_the_english_entries() {
        assert!(VIWIKTIONARY_EN.attribution.contains("mục tiếng Anh"));
        assert!(VIWIKTIONARY_EN.attribution.contains("vi.wiktionary.org"));
        assert!(VIWIKTIONARY_EN.attribution.contains("kaikki.org"));
        assert!(VIWIKTIONARY_EN.attribution.contains("CC BY-SA 4.0"));
        assert!(VIWIKTIONARY_EN.attribution.contains("GFDL"));
        assert!(VIWIKTIONARY_EN.display_name.contains("mục tiếng Anh"));
    }

    /// AC4: cả bốn trường giấy phép khác rỗng, và `license_text` là **văn bản thật**
    /// (dùng lại `LicenseRef::CcBySaAndGfdl` của vai B — cùng tệp, cùng giấy phép),
    /// không phải chuỗi giữ chỗ.
    #[test]
    fn viwiktionary_en_carries_all_four_license_fields_filled() {
        assert_eq!(VIWIKTIONARY_EN.license_kind, "open");
        assert_eq!(VIWIKTIONARY_EN.license_id, Some("CC-BY-SA-4.0"));
        assert!(!VIWIKTIONARY_EN.attribution.is_empty());
        assert!(!VIWIKTIONARY_EN.source_url.is_empty());

        let text = VIWIKTIONARY_EN.license_text();
        assert!(!text.is_empty());
        assert_eq!(
            text,
            VIWIKTIONARY.license_text(),
            "cùng tệp thô ⇒ cùng giấy phép ⇒ cùng văn bản, không thêm biến thể enum"
        );
    }

    /// Story 1.10c: đúng BA lớp gỡ rời trong phạm vi hôm nay (`tran-van-chanh` thêm ở
    /// story này). HVTĐTD + Cổ hán văn vẫn chưa có nguồn thô — xem `deferred-work.md`.
    #[test]
    fn exactly_three_detachable_sources_in_scope_today() {
        assert_eq!(DETACHABLE_ALL.len(), 3);
        let codes: Vec<&str> = DETACHABLE_ALL.iter().map(|s| s.code).collect();
        assert_eq!(codes, vec!["thieu-chuu", "vietphrase", "tran-van-chanh"]);
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

    /// AC3: bảng chốt cứng `license_id = NULL` cho `vietphrase` — không phải chuỗi
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
    /// thân, không phải phép lịch sự.
    #[test]
    fn thieu_chuu_attribution_names_the_author() {
        assert!(THIEU_CHUU.attribution.contains("Thiều Chửu"));
        assert!(THIEU_CHUU.attribution.contains("Nguyễn Hữu Kha"));
    }

    // ═══════════════════════════════════════════════════════════════════════════════
    // Story 1.10c — `EN_WIKTIONARY_VI` (nền thứ 7) · `TRAN_VAN_CHANH` (gỡ rời thứ 3)
    // ═══════════════════════════════════════════════════════════════════════════════

    /// AC8 đối chứng âm — lỗi gán nhãn dễ mắc nhất của story này: `tran-van-chanh`
    /// KHÔNG được gán `public-domain` lẫn `unknown`. Khác VietPhrase (tác giả không xác
    /// định được), ở đây tác giả xác định được VÀ tác phẩm còn trong bản quyền.
    #[test]
    fn tran_van_chanh_is_copyrighted_not_public_domain_or_unknown() {
        assert_eq!(TRAN_VAN_CHANH.license_kind, "copyrighted");
        assert_ne!(TRAN_VAN_CHANH.license_kind, "public-domain");
        assert_ne!(TRAN_VAN_CHANH.license_kind, "unknown");
    }

    /// AC8: `license_id = NULL` — CC0 của người số hoá không phải một mã giấy phép mở áp
    /// dụng được cho toàn bộ nội dung.
    #[test]
    fn tran_van_chanh_declares_no_license_id_at_all() {
        assert_eq!(TRAN_VAN_CHANH.license_id, None);
    }

    /// AC8: rủi ro pháp lý phải hiện ra ngay trong `attribution` — nó đi theo dữ liệu ra
    /// tới màn hình Attribution (Story 10.4), không ở lại trong một story file.
    #[test]
    fn tran_van_chanh_attribution_states_the_copyright_risk() {
        assert!(TRAN_VAN_CHANH.attribution.contains("Trần Văn Chánh"));
        assert!(TRAN_VAN_CHANH.attribution.contains("CÒN TRONG BẢN QUYỀN"));
        assert!(TRAN_VAN_CHANH.attribution.contains("CC0"));
    }

    /// AC8: `license_text` phải là văn bản thật (tuyên bố xuất xứ + toàn văn CC0 1.0),
    /// và phải nêu rõ CC0 KHÔNG xoá bản quyền tác phẩm gốc — không phải một chuỗi giữ
    /// chỗ hay một bản CC0 trần trụi gây hiểu lầm "nguồn sạch".
    #[test]
    fn tran_van_chanh_license_text_states_the_cc0_does_not_clear_original_copyright() {
        let text = TRAN_VAN_CHANH.license_text();
        assert!(text.len() > 200);
        assert!(text.contains("KHÔNG") && text.contains("bản quyền"));
        assert!(text.contains("CC0"));
    }

    /// `en-wiktionary-vi` dùng LẠI `LicenseRef::CcBySaAndGfdl` — cùng kho kaikki.org,
    /// cùng cặp giấy phép như ba nguồn Wiktextract khác, không thêm biến thể mới.
    #[test]
    fn en_wiktionary_vi_reuses_the_cc_by_sa_and_gfdl_license_text() {
        assert_eq!(EN_WIKTIONARY_VI.license_kind, "open");
        assert_eq!(EN_WIKTIONARY_VI.license_id, Some("CC-BY-SA-4.0"));
        assert_eq!(EN_WIKTIONARY_VI.license_text(), EN_WIKTIONARY.license_text());
    }

    /// AC3: `display_name` phải phân biệt được với `en-wiktionary` (mục tiếng Trung) —
    /// hai dòng giống hệt nhau trên màn hình Attribution là lỗi hiển thị.
    #[test]
    fn en_wiktionary_vi_display_name_is_distinct_from_the_chinese_role() {
        assert_ne!(EN_WIKTIONARY_VI.display_name, EN_WIKTIONARY.display_name);
        assert_ne!(EN_WIKTIONARY_VI.code, EN_WIKTIONARY.code);
    }
}
