//! en.wiktionary — extract **Vietnamese** (`lang_code = "vi"`), qua Wiktextract/
//! kaikki.org. Nguồn NỀN thứ bảy, Story 1.10c — LƯỚI chống tái diễn lỗi Unihan (AC5),
//! ⛔ không phải một nguồn nghĩa.
//!
//! # Quyết định #3a (Ice chốt 2026-08-06) — chỉ nạp ÂM ĐỌC có gắn nhãn
//!
//! `en.wiktionary` gắn nhãn **tách bạch** hai loại âm đọc tiếng Việt của một ký tự Hán,
//! trên `senses[].related[].tags`: `"han-viet-reading"` và `"nom-reading"`. Giá trị của
//! nguồn này ⛔ **không** nằm ở số ký tự nó thêm (294 ký tự ngoài Thiều Chửu) — nó nằm ở
//! **NHÃN phân biệt HV/Nôm**, thứ ⛔ không nguồn nào khác trong bộ có, và là lưới duy
//! nhất ngăn lỗi Unihan tái diễn ở một nguồn tương lai (`build.rs::verify_han_viet_
//! against_nom_labels`, AC5).
//!
//! ⇒ Nguồn này **KHÔNG** nạp `dict_sense`/ví dụ/trích dẫn — chỉ hai cột âm đọc. Nạp
//! trọn (nghĩa + ví dụ) đã bị LOẠI: kéo thêm ~78 MB JSONL vào đường build, chồng nghĩa
//! lên sáu nguồn nền đã có, ăn vào dư địa NFR6 cho một thứ không AC nào đòi.
//!
//! Đọc qua [`super::wiktextract_common::parse_readings`] — dùng lại đúng khuôn JSONL
//! Wiktextract mà `viwiktionary`/`en_wiktionary`/`viwiktionary_en` đã dùng, ⛔ không viết
//! parser JSON thứ hai (xem doc-comment `wiktextract_common.rs`).

use std::io::BufRead;

use crate::model::{ParseIssue, RawEntry};
use crate::sources::wiktextract_common;

pub const SOURCE_CODE: &str = "en-wiktionary-vi";

/// Cùng chữ ký với các parser còn lại: `parse(reader) -> impl Iterator<Item = Result<RawEntry>>`.
/// Lọc `lang_code = "vi"` — tệp nguồn (kaikki.org/dictionary/Vietnamese) đã 100% `vi`
/// trên dữ liệu thật, nhưng lọc tường minh thay vì giả định hình dạng tệp đầu vào.
pub fn parse<R: BufRead>(reader: R) -> impl Iterator<Item = Result<RawEntry, ParseIssue>> {
    wiktextract_common::parse_readings(reader, Some("vi"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    /// Ca thật `北` — cùng dữ liệu §Phát hiện của story trích dẫn (`EXPERIENCE.md:410`,
    /// FR113: `北涼 → Bắc Lương`).
    #[test]
    fn real_shaped_bac_character_carries_han_viet_and_nom_reading() {
        let json = "{\"pos\": \"character\", \"word\": \"北\", \"lang_code\": \"vi\", \"senses\": [{\"tags\": [\"no-gloss\"], \"related\": [{\"word\": \"bắc\", \"tags\": [\"han-viet-reading\"]}, {\"word\": \"bậc\", \"tags\": [\"nom-reading\"]}]}]}\n";
        let entries: Vec<RawEntry> = parse(Cursor::new(json.as_bytes()))
            .collect::<Result<_, _>>()
            .unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].headword, "北");
        assert_eq!(entries[0].han_viet.as_deref(), Some("bắc"));
        assert_eq!(entries[0].nom_reading.as_deref(), Some("bậc"));
        assert!(entries[0].senses.is_empty(), "Quyết định #3a: ⛔ không nạp dict_sense");
    }

    /// `SOURCE_CODE` ⛔ không được trôi khỏi `sources_meta::EN_WIKTIONARY_VI.code`.
    #[test]
    fn source_code_matches_its_source_meta() {
        assert_eq!(SOURCE_CODE, crate::sources_meta::EN_WIKTIONARY_VI.code);
    }
}
