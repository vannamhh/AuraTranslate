//! viwiktionary — **VAI B: mục từ TIẾNG TRUNG** — kaikki.org, ẤN BẢN TIẾNG VIỆT
//! (`vi.wiktionary.org`), trích thô toàn ấn bản:
//! `kaikki.org/dictionary/downloads/vi/vi-extract.jsonl`. Giấy phép CC-BY-SA + GFDL.
//!
//! ⚠️ Không giống trang `Chinese` của ấn bản `en` (đã lọc sẵn theo kaikki), tệp này
//! chứa MỌI ngôn ngữ mà ấn bản `vi` có mục từ (`Tiếng Anh`, `Tiếng Mông Cổ`, …) — lọc
//! `lang_code == "zh"` là việc của module này, ⛔ không phải việc của kaikki. Đây CHÍNH
//! LÀ ý nghĩa của "bản trích theo ngôn ngữ của ấn bản vi" trong §Thông tin kỹ thuật.
//!
//! 🔴 **Chính vì thế cùng tệp này còn mang một VAI THỨ HAI.** Lọc `lang_code == "en"`
//! trên đúng những byte đó cho ra 119.039 mục từ tiếng Anh — nguồn
//! [`super::viwiktionary_en`], dựng ở Story 1.10b để phục vụ FR34. Hai vai là **hai
//! nguồn rời nhau**, hai `source_id`, hai lượt `File::open`. ⛔ Không gộp chúng thành
//! một lượt đọc — lý do đầy đủ ở doc-comment của [`super::viwiktionary_en`].
//!
//! ⛔ Module này giữ nguyên `code = "viwiktionary"` (⛔ **không** đổi thành
//! `viwiktionary-zh` "cho đối xứng"): `dict_source.code` là khoá đối chiếu xuyên tệp mà
//! `dict-manifest.toml`, PRD §8.2/§8.3 và `epics.md` đều đã ghi.
//!
//! `pos_title` ở ấn bản này ĐÃ sẵn tiếng Việt (đã kiểm thật: `"Động từ"`) ⇒ dùng thẳng,
//! `pos_lang = 'vi'` (FR35 chỉ đòi đánh dấu NGOẠI NGỮ; tiếng Việt là ngôn ngữ hiển thị
//! mặc định nên không cần đánh dấu, nhưng cột vẫn ghi `'vi'` tường minh — ⛔ không để
//! NULL, vì NULL ở đây không phân biệt được với "không rõ ngôn ngữ nhãn").
//!
//! ⚠️ Độ phủ THẤP là ĐÚNG, không phải lỗi đọc (đã đo Giai đoạn 0: 2,76% đầu mục tiếng
//! Trung của CVDICT, chỉ 0,067% có ví dụ) — phần lớn dòng bị lọc vì không phải `zh`, và
//! phần lớn còn lại bị bỏ vì `senses` toàn `no-gloss` (đã thấy thật trên dữ liệu, ví dụ
//! `词典` với tag `no-gloss`). Cả hai đều rơi vào `skip_reasons` có tên, ⛔ không lẫn với
//! lỗi JSON thật.

use std::io::BufRead;

use crate::model::{ParseIssue, RawEntry};
use crate::sources::wiktextract_common;

pub const SOURCE_CODE: &str = "viwiktionary";

/// Cùng chữ ký với bốn parser còn lại: `parse(reader) -> impl Iterator<Item = Result<RawEntry>>`.
/// Gộp theo `headword` TRONG nguồn này qua `wiktextract_common::parse` (xem doc-comment
/// ở đó — AC1 mệnh đề 4, Review Findings Group A).
pub fn parse<R: BufRead>(reader: R) -> impl Iterator<Item = Result<RawEntry, ParseIssue>> {
    wiktextract_common::parse(reader, "vi", Some("zh"), "zh")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn filters_non_chinese_and_keeps_chinese_with_local_pos_title() {
        let text = "\
{\"word\":\"ăn\",\"pos\":\"verb\",\"pos_title\":\"Động từ\",\"lang_code\":\"cmo\",\"senses\":[{\"glosses\":[\"cho.\"]}]}
{\"word\":\"字典\",\"pos\":\"noun\",\"pos_title\":\"Danh từ\",\"lang_code\":\"zh\",\"senses\":[{\"glosses\":[\"từ điển\"]}]}
";
        let results: Vec<_> = parse(Cursor::new(text.as_bytes())).collect();
        assert_eq!(results.len(), 2);
        assert!(results[0].is_err());
        let entry = results[1].as_ref().unwrap();
        assert_eq!(entry.headword, "字典");
        assert_eq!(entry.senses[0].pos.as_deref(), Some("Danh từ"));
        assert_eq!(entry.senses[0].pos_lang.as_deref(), Some("vi"));
    }
}
