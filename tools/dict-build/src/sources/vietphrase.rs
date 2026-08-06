//! VietPhrase — `<đầu mục>=<nghĩa1>/<nghĩa2>/…`. Lớp GỠ RỜI, Story 1.10. Đầu mục không
//! chỉ là từ — có cả cụm và cả câu (`去那里要干什么?`); đó là ĐÚNG dữ liệu, ⛔ không lọc
//! theo độ dài (§Thông tin kỹ thuật, Bẫy nguồn #2).
//!
//! Tệp `docs/dics/VietPhrase.txt` hôm nay ĐÃ là UTF-8, không BOM (Ice chuyển 2026-08-05).
//! Dòng phòng vệ BOM dưới đây rẻ và vô hại — bảo vệ một lượt `iconv` khác cấu hình để lại
//! BOM ở lần tải sau (§Bẫy 5 của story).

use std::io::BufRead;

use crate::model::{ParseIssue, RawEntry, RawSense};

pub const SOURCE_CODE: &str = "vietphrase";
pub const SOURCE_VERSION: &str = "truyencuatui/VietPhrase@master (tải 2026-08-05, chuyển UTF-8)";

const BOM: char = '\u{feff}';

/// Chữ ký giống bảy module còn lại: `parse(reader) -> impl Iterator<Item = Result<RawEntry, ParseIssue>>`.
pub fn parse<R: BufRead>(reader: R) -> impl Iterator<Item = Result<RawEntry, ParseIssue>> {
    let mut first_line = true;
    reader.lines().enumerate().filter_map(move |(idx, line)| {
        let line_no = idx + 1;
        let mut raw = match line {
            Ok(l) => l,
            Err(e) => {
                return Some(Err(ParseIssue {
                    line: line_no,
                    reason: format!("I/O error: {e}"),
                }));
            }
        };
        if first_line {
            first_line = false;
            if let Some(stripped) = raw.strip_prefix(BOM) {
                raw = stripped.to_string();
            }
        }
        if raw.trim().is_empty() {
            return None;
        }

        // 🔴 `splitn(2, '=')` — đúng nghĩa hơn `split`: chỉ tách ở dấu '=' ĐẦU TIÊN,
        // miễn nhiễm với dòng lạ có nhiều dấu '=' (679.311/679.311 dòng thật có đúng
        // một dấu '=', nhưng splitn không phụ thuộc giả định đó để đúng).
        let mut parts = raw.splitn(2, '=');
        let headword = parts.next().unwrap_or("").trim();
        let rest = match parts.next() {
            Some(r) => r.trim(),
            None => {
                return Some(Err(ParseIssue {
                    line: line_no,
                    reason: "missing '=' separator".to_string(),
                }));
            }
        };

        if headword.is_empty() {
            return Some(Err(ParseIssue {
                line: line_no,
                reason: "empty headword before '='".to_string(),
            }));
        }

        // 🔴 Luật lọc rác: bỏ dòng có nghĩa rỗng hoặc bằng "()" — spam quảng cáo thật
        // trong dữ liệu (9 dòng), mỗi dòng bỏ có lý do vào ParseIssue/SourceStats.
        if rest.is_empty() || rest == "()" {
            return Some(Err(ParseIssue {
                line: line_no,
                reason: "empty or placeholder '()' gloss field".to_string(),
            }));
        }

        // Tách '/' ⇒ nhiều dict_sense, ord giữ THỨ TỰ ƯU TIÊN của tệp gốc (mục đầu là
        // bản dịch được ưu tiên). ⛔ Không tách theo ',' — dòng đa đầu mục dùng ',' được
        // nạp NGUYÊN VĂN, không tự bóc (tách sai ⇒ một đầu mục ghép mang nghĩa của đầu
        // mục khác, sai nguồn ở mức nghĩa).
        let senses: Vec<RawSense> = rest
            .split('/')
            .map(|g| g.trim())
            .filter(|g| !g.is_empty())
            .map(|gloss| RawSense {
                pos: None,
                pos_lang: None,
                gloss: gloss.to_string(),
                note: None,
                examples: Vec::new(),
                citations: Vec::new(),
            })
            .collect();

        if senses.is_empty() {
            return Some(Err(ParseIssue {
                line: line_no,
                reason: "no non-empty senses after splitting on '/'".to_string(),
            }));
        }

        Some(Ok(RawEntry {
            lang: "zh".to_string(),
            headword: headword.to_string(),
            headword_simp: None,
            reading: None,
            han_viet: None,
            nom_reading: None,
            senses,
        }))
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn parses_a_multi_sense_entry_in_priority_order() {
        let text = "出没=qua lại/thường lui tới/ẩn hiện/xuất ẩn\n";
        let entries: Vec<_> = parse(Cursor::new(text.as_bytes()))
            .collect::<Result<_, _>>()
            .unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].headword, "出没");
        assert_eq!(entries[0].senses.len(), 4);
        assert_eq!(entries[0].senses[0].gloss, "qua lại");
        assert_eq!(entries[0].senses[3].gloss, "xuất ẩn");
    }

    /// 🔴 9 dòng rác thật trong dữ liệu — nghĩa rỗng hoặc "()".
    #[test]
    fn placeholder_parens_gloss_is_a_parse_issue_not_a_panic() {
        let text = "(txt8 小说下载网 -Www.txt8.cn)=()\ntxt8 小说下载网 -Www.txt8.cn=()\n";
        let results: Vec<_> = parse(Cursor::new(text.as_bytes())).collect();
        assert_eq!(results.len(), 2);
        assert!(results[0].is_err());
        assert!(results[1].is_err());
    }

    /// Đa đầu mục dùng ',' KHÔNG được tự bóc tách — nạp nguyên chuỗi.
    #[test]
    fn comma_separated_multi_headword_line_is_ingested_verbatim_not_split() {
        let text = "太一,正一,纯一=thái nhất,chính nhất,thuần nhất\n";
        let entries: Vec<_> = parse(Cursor::new(text.as_bytes()))
            .collect::<Result<_, _>>()
            .unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].headword, "太一,正一,纯一");
        assert_eq!(entries[0].senses.len(), 1);
        assert_eq!(entries[0].senses[0].gloss, "thái nhất,chính nhất,thuần nhất");
    }

    /// Đầu mục là cả câu — dữ liệu ĐÚNG, ⛔ không lọc theo độ dài.
    #[test]
    fn sentence_length_headword_is_a_valid_entry() {
        let text = "去那里要干什么?=đi vào đó để làm gì?\n";
        let entries: Vec<_> = parse(Cursor::new(text.as_bytes()))
            .collect::<Result<_, _>>()
            .unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].headword, "去那里要干什么?");
    }

    #[test]
    fn leading_bom_on_first_line_is_stripped() {
        let text = "\u{feff}一个又一个=lần lượt\n黄沙=cát vàng\n";
        let entries: Vec<_> = parse(Cursor::new(text.as_bytes()))
            .collect::<Result<_, _>>()
            .unwrap();
        assert_eq!(entries[0].headword, "一个又一个");
        assert_eq!(entries[1].headword, "黄沙");
    }

    /// `SOURCE_CODE` ⛔ không được trôi khỏi `sources_meta::VIETPHRASE.code`.
    #[test]
    fn source_code_matches_its_source_meta() {
        assert_eq!(SOURCE_CODE, crate::sources_meta::VIETPHRASE.code);
    }

    #[test]
    fn missing_equals_separator_is_a_parse_issue() {
        let text = "no equals sign here\n";
        let results: Vec<_> = parse(Cursor::new(text.as_bytes())).collect();
        assert_eq!(results.len(), 1);
        assert!(results[0].is_err());
    }
}
