//! viwiktionary — **VAI A: mục từ TIẾNG ANH** (Story 1.10b, FR34).
//!
//! # 🔴 Một tệp thô, HAI nguồn — đọc chỗ này trước khi sửa gì
//!
//! Module này đọc **CHÍNH XÁC CÙNG MỘT TỆP** với [`super::viwiktionary`]:
//! `raw/viwiktionary/vi-extract.jsonl`. Đó ⛔ **không phải nhầm lẫn** và ⛔ **không phải
//! thứ cần gộp lại**.
//!
//! `vi-extract.jsonl` là bản trích **TOÀN ẤN BẢN** `vi.wiktionary.org` — nó chứa mục từ
//! của **mọi** ngôn ngữ mà ấn bản đó có (415.254 dòng, 273 MB). Bộ lọc `lang_code` ở
//! top-level quyết định lượt đọc lấy được **vai** nào:
//!
//! | Vai | Module | `filter_lang_code` | `entry_lang` | `pos_lang` | Đầu mục | Phục vụ |
//! |---|---|---|---|---|---:|---|
//! | **B** | [`super::viwiktionary`] | `"zh"` | `"zh"` | `"vi"` | 1.598 | lớp từ loại ZH (PRD §8.3) |
//! | **A** | **module này** | `"en"` | **`"en"`** | `"vi"` | 119.039 | **FR34**, cặp Anh → Việt |
//!
//! Trước Story 1.10b chỉ vai B được dựng, nên `dict-core.db` có 473.499 đầu mục **100%
//! `lang='zh'`** và FR34 — *"mục từ tiếng Anh phải có nhãn từ loại và nghĩa tiếng
//! Việt"* — ⛔ không có một byte dữ liệu nào để đứng lên. Nguyên nhân gốc là mơ hồ của
//! PRD (§8.2 giao vai A, §8.3 bàn cùng tệp ở vai B, không chỗ nào nói đó là hai vai song
//! song), ⛔ không phải lỗi cài đặt của Story 1.9.
//!
//! # ⛔ KHÔNG gộp hai vai thành một lượt đọc
//!
//! Tệp 273 MB đọc hai lần trông lãng phí. Đừng. [`super::wiktextract_common::parse`] gộp
//! theo headword **trong một lượt gọi**; một lượt gọi phát cả hai vai ⇒ hai nguồn dùng
//! chung một `HashMap` ⇒ một headword có mặt ở cả hai vai bị gộp thành MỘT `RawEntry`
//! mang MỘT `source_id`. Đó **là** hợp nhất xuyên nguồn — đúng thứ AD-19 cấm và đúng
//! thứ miễn trừ `dict-build:allow .entry(` tuyên bố không bao giờ xảy ra. Hai `File::open`,
//! hai lượt `parse` độc lập, hai `source_id`.
//!
//! # `pos_lang = "vi"`, ⛔ không phải `"en"`
//!
//! Ấn bản `vi` mang `pos_title` **đã sẵn tiếng Việt** kể cả trên mục từ tiếng Anh — đã
//! kiểm thật trên fixture: `API` ⇒ `"Danh từ"`, `Wikipedia` ⇒ `"Danh từ riêng"`. FR35 chỉ
//! đòi đánh dấu nhãn **NGOẠI NGỮ**; nhãn ở đây là tiếng Việt.
//!
//! Giấy phép CC-BY-SA 4.0 + GFDL 1.3 — **y hệt vai B**, cùng tệp, cùng kho.

use std::io::BufRead;

use crate::model::{ParseIssue, RawEntry};
use crate::sources::wiktextract_common;

pub const SOURCE_CODE: &str = "viwiktionary-en";

/// Cùng chữ ký với sáu parser còn lại: `parse(reader) -> impl Iterator<Item = Result<RawEntry>>`.
/// Gộp theo `headword` TRONG nguồn này qua [`wiktextract_common::parse`].
pub fn parse<R: BufRead>(reader: R) -> impl Iterator<Item = Result<RawEntry, ParseIssue>> {
    wiktextract_common::parse(reader, "vi", Some("en"), "en")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    /// 🔴 AC3. Dòng `lang_code:"en"` phải ra `lang == "en"`; dòng `lang_code:"zh"` phải
    /// bị **LỌC** thành `ParseIssue`, ⛔ không lọt vào nguồn này.
    #[test]
    fn keeps_english_lines_tagged_en_and_filters_chinese_ones() {
        let text = "\
{\"word\":\"字典\",\"pos\":\"noun\",\"pos_title\":\"Danh từ\",\"lang_code\":\"zh\",\"senses\":[{\"glosses\":[\"từ điển\"]}]}
{\"word\":\"dictionary\",\"pos\":\"noun\",\"pos_title\":\"Danh từ\",\"lang_code\":\"en\",\"senses\":[{\"glosses\":[\"từ điển\"]}]}
";
        let results: Vec<_> = parse(Cursor::new(text.as_bytes())).collect();
        assert_eq!(results.len(), 2);

        let issue = results[0].as_ref().unwrap_err();
        assert!(
            issue.reason.contains("filtered, expected"),
            "dòng zh phải bị LỌC có chủ ý, ⛔ không phải lỗi đọc: {}",
            issue.reason
        );

        let entry = results[1].as_ref().unwrap();
        assert_eq!(entry.headword, "dictionary");
        assert_eq!(entry.lang, "en", "🔴 AC3: ⛔ KHÔNG được mang nhãn 'zh'");
        assert_eq!(entry.senses[0].pos.as_deref(), Some("Danh từ"));
        assert_eq!(
            entry.senses[0].pos_lang.as_deref(),
            Some("vi"),
            "ấn bản vi có pos_title sẵn tiếng Việt (FR35)"
        );
        assert_eq!(entry.senses[0].gloss, "từ điển");
    }

    /// Đối chứng âm: nguồn này ⛔ **không** sinh nổi một hàng `lang = "zh"` nào, kể cả
    /// khi tệp thô đầy dòng tiếng Trung — vì bộ lọc chặn chúng TRƯỚC khi dán nhãn.
    #[test]
    fn produces_zero_entries_tagged_zh_no_matter_the_input() {
        let text = "\
{\"word\":\"馬\",\"pos\":\"character\",\"lang_code\":\"zh\",\"senses\":[{\"glosses\":[\"ngựa\"]}]}
{\"word\":\"字典\",\"pos\":\"noun\",\"lang_code\":\"zh\",\"senses\":[{\"glosses\":[\"từ điển\"]}]}
{\"word\":\"book\",\"pos\":\"noun\",\"lang_code\":\"en\",\"senses\":[{\"glosses\":[\"quyển sách\"]}]}
";
        let entries: Vec<RawEntry> = parse(Cursor::new(text.as_bytes()))
            .filter_map(Result::ok)
            .collect();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries.iter().filter(|e| e.lang == "zh").count(), 0);
    }

    /// AD-19 ở mức module: hai vai đọc cùng một tệp phải cho hai tập đầu mục **rời
    /// nhau**, ⛔ không headword nào đi qua cả hai.
    #[test]
    fn the_two_roles_read_the_same_bytes_into_disjoint_entry_sets() {
        let text = "\
{\"word\":\"字典\",\"pos\":\"noun\",\"lang_code\":\"zh\",\"senses\":[{\"glosses\":[\"từ điển\"]}]}
{\"word\":\"dictionary\",\"pos\":\"noun\",\"lang_code\":\"en\",\"senses\":[{\"glosses\":[\"từ điển\"]}]}
";
        let role_a: Vec<String> = parse(Cursor::new(text.as_bytes()))
            .filter_map(Result::ok)
            .map(|e| e.headword)
            .collect();
        let role_b: Vec<String> = super::super::viwiktionary::parse(Cursor::new(text.as_bytes()))
            .filter_map(Result::ok)
            .map(|e| e.headword)
            .collect();

        assert_eq!(role_a, vec!["dictionary".to_string()]);
        assert_eq!(role_b, vec!["字典".to_string()]);
        assert!(
            role_a.iter().all(|h| !role_b.contains(h)),
            "AD-19: ⛔ không headword nào được thuộc cả hai vai"
        );
    }
}
