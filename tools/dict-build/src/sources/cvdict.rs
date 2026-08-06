//! CVDICT — `github.com/ph0ngp/CVDICT` → `CVDICT.u8`. Cùng khuôn văn bản với CC-CEDICT
//! (`phồn giản [pinyin] /nghĩa/nghĩa/`), giấy phép **CC BY-SA 4.0**.
//!
//! `source_version` KHÔNG nằm trong tệp — lấy từ tag/commit của repo lúc tải
//! (§Thông tin kỹ thuật của Story 1.9), nên hàm ở đây nhận nó từ caller (`build.rs`)
//! thay vì tự dò trong nội dung.

use std::io::BufRead;

use crate::model::{ParseIssue, RawEntry, RawSense};
use crate::sources::cedict_common::parse_line;

pub const SOURCE_CODE: &str = "cvdict";

/// Cùng chữ ký với bốn parser còn lại: `parse(reader) -> impl Iterator<Item = Result<RawEntry>>`.
pub fn parse<R: BufRead>(reader: R) -> impl Iterator<Item = Result<RawEntry, ParseIssue>> {
    reader.lines().enumerate().filter_map(|(idx, line)| {
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
    fn parses_fixture_lines() {
        let text = "山 山 [shan1] /mountain/hill/\n";
        let entries: Vec<_> = parse(Cursor::new(text.as_bytes()))
            .collect::<Result<_, _>>()
            .unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].headword, "山");
        assert_eq!(entries[0].senses.len(), 2);
    }

    #[test]
    fn a_broken_line_is_reported_not_panicked() {
        let text = "not a valid line at all\n";
        let results: Vec<_> = parse(Cursor::new(text.as_bytes())).collect();
        assert_eq!(results.len(), 1);
        assert!(results[0].is_err());
    }
}
