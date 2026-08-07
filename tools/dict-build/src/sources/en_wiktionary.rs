//! en.wiktionary (cho tiếng Trung) — kaikki.org, trang **Chinese** của ấn bản
//! **tiếng Anh**: `kaikki.org/dictionary/Chinese/kaikki.org-dictionary-Chinese.jsonl`.
//! Giấy phép CC-BY-SA + GFDL.
//!
//! ⚠️ KHÔNG phải `zh.wiktionary` (ấn bản tiếng Trung) — Giai đoạn 0 đã BÁC nó vì
//! định nghĩa và nhãn từ loại đều bằng tiếng Trung (Bẫy 11 của Story 1.9). Trang
//! `Chinese` này đã được kaikki lọc sẵn theo NGÔN NGỮ MỤC TIÊU (`lang_code=zh`) từ ấn
//! bản tiếng Anh — không cần lọc thêm ở đây, nhưng vẫn kiểm phòng hờ dữ liệu lẫn dòng.
//!
//! `pos` (và `pos_title` khi có) ở ấn bản này là TIẾNG ANH ⇒ `pos_lang = 'en'` (FR35).

use std::io::BufRead;

use crate::model::{ParseIssue, RawEntry};
use crate::sources::wiktextract_common;

pub const SOURCE_CODE: &str = "en-wiktionary";

/// Cùng chữ ký với bốn parser còn lại: `parse(reader) -> impl Iterator<Item = Result<RawEntry>>`.
/// Gộp theo `headword` TRONG nguồn này qua `wiktextract_common::parse` (xem doc-comment
/// ở đó — AC1 mệnh đề 4, Review Findings Group A).
pub fn parse<R: BufRead>(reader: R) -> impl Iterator<Item = Result<RawEntry, ParseIssue>> {
    wiktextract_common::parse(reader, "en", Some("zh"), "zh")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn parses_fixture_lines() {
        let text = "{\"word\":\"詞典\",\"pos\":\"noun\",\"lang_code\":\"zh\",\"senses\":[{\"glosses\":[\"dictionary\"]}]}\n";
        let entries: Vec<RawEntry> = parse(Cursor::new(text.as_bytes()))
            .collect::<Result<_, _>>()
            .unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].senses[0].pos_lang.as_deref(), Some("en"));
    }
}
