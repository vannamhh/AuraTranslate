//! CC-CEDICT — `mdbg.net/chinese/export/cedict/cedict_1_0_ts_utf-8_mdbg.txt.gz`.
//! Khuôn `phồn giản [pinyin] /nghĩa/nghĩa/`. Giấy phép CC BY-SA 4.0.
//! `source_version` lấy từ dòng header `#! date=` NGAY TRONG tệp, không viết cứng
//! (§Thông tin kỹ thuật của Story 1.9).

use std::io::BufRead;

use crate::model::{ParseIssue, RawEntry, RawSense};
use crate::sources::cedict_common::{find_date_header, parse_line};

pub const SOURCE_CODE: &str = "cc-cedict";

/// Đọc `source_version` từ dòng `#! date=` — chỉ đáng tin sau khi đã đọc phần header,
/// nên hàm này nhận toàn bộ nội dung tệp (đã tách dòng) thay vì tự mở tệp.
pub fn source_version(lines: &[String]) -> Option<String> {
    find_date_header(lines)
}

/// Cùng chữ ký với bốn parser còn lại: `parse(reader) -> impl Iterator<Item = Result<RawEntry>>`.
pub fn parse<R: BufRead>(reader: R) -> impl Iterator<Item = Result<RawEntry, ParseIssue>> {
    reader
        .lines()
        .enumerate()
        .filter_map(|(idx, line)| {
            let line_no = idx + 1;
            let raw = match line {
                Ok(l) => l,
                Err(e) => {
                    return Some(Err(ParseIssue {
                        line: line_no,
                        reason: format!("I/O error: {e}"),
                    }));
                }
            };
            match parse_line(line_no, &raw) {
                Ok(None) => None,
                Ok(Some(l)) => {
                    let senses = l
                        .glosses
                        .into_iter()
                        .map(|gloss| RawSense {
                            pos: None,
                            pos_lang: None,
                            gloss,
                            note: None,
                            examples: Vec::new(),
                            citations: Vec::new(),
                        })
                        .collect::<Vec<_>>();
                    Some(Ok(RawEntry {
                        lang: "zh".to_string(),
                        headword: l.traditional,
                        headword_simp: Some(l.simplified),
                        reading: if l.pinyin.is_empty() {
                            None
                        } else {
                            Some(l.pinyin)
                        },
                        han_viet: None,
                        nom_reading: None,
                        senses,
                    }))
                }
                Err(issue) => Some(Err(issue)),
            }
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn parses_fixture_lines_into_entries_with_multiple_senses() {
        let text = "\
# CC-CEDICT
#! date=2026-01-15
中國 中国 [Zhong1 guo2] /China/Middle Kingdom/
";
        let entries: Vec<_> = parse(Cursor::new(text.as_bytes()))
            .collect::<Result<_, _>>()
            .unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].headword, "中國");
        assert_eq!(entries[0].headword_simp.as_deref(), Some("中国"));
        assert_eq!(entries[0].senses.len(), 2);

        let lines: Vec<String> = text.lines().map(|s| s.to_string()).collect();
        assert_eq!(source_version(&lines).as_deref(), Some("2026-01-15"));
    }

    #[test]
    fn a_broken_line_is_reported_not_panicked() {
        let text = "中國 中国 Zhong1 guo2 /China/\n中國 中国 [Zhong1 guo2] /China/\n";
        let results: Vec<_> = parse(Cursor::new(text.as_bytes())).collect();
        assert_eq!(results.len(), 2);
        assert!(results[0].is_err());
        assert!(results[1].is_ok());
    }
}
