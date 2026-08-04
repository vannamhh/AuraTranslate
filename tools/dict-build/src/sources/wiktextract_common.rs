//! Đọc dòng JSONL khuôn Wiktextract dùng chung cho `viwiktionary` và `en_wiktionary`
//! (Story 1.9, Task 4). Hai nguồn cùng công cụ trích xuất (tatuylonen/wiktextract, phân
//! phối qua kaikki.org) nên cùng hình dạng JSON; khác nhau ở **ấn bản** nguồn và ở
//! `pos_lang` gắn vào mỗi nghĩa (FR35). Đây là chia sẻ CODE ĐỌC JSON, ⛔ không phải hợp
//! nhất NGHĨA — mỗi caller vẫn tự gắn `source_id` của mình.
//!
//! ⚠️ Trường thật đã kiểm 2026-08-04 bằng cách tải mẫu thật từ kaikki.org (không suy
//! đoán từ tài liệu): `word` · `pos` · `lang_code` · `senses[].glosses` ·
//! `senses[].examples[].text` + `.english`/`.translation`. `pos_title` CÓ mặt ở ấn bản
//! `vi` (ví dụ `"Động từ"`) nhưng KHÔNG thấy trong mẫu đã tải của ấn bản `en` — code ở
//! đây vì vậy ưu tiên `pos_title` khi có, lùi về `pos` khi không, cho cả hai ấn bản.

use std::collections::HashMap;
use std::io::BufRead;

use serde_json::Value;

use crate::model::{ParseIssue, RawEntry, RawExample, RawSense};

/// Kết quả tách MỘT dòng JSONL: `Ok(None)` khi dòng RỖNG (không phải lỗi đọc). Dòng bị
/// LỌC CÓ CHỦ Ý (`lang_code` không khớp `filter_lang_code`) trả `Err` với lý do riêng —
/// `"filtered, expected — không phải lỗi đọc"` — nên vẫn rơi vào một nhóm `skip_reasons`
/// tách biệt với lỗi đọc thật, dù cả hai đều là biến thể `Err`.
pub fn parse_line(
    line_no: usize,
    raw: &str,
    pos_lang: &str,
    filter_lang_code: Option<&str>,
) -> Result<Option<RawEntry>, ParseIssue> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Ok(None);
    }

    let v: Value = serde_json::from_str(trimmed).map_err(|e| ParseIssue {
        line: line_no,
        reason: format!("invalid JSON: {e}"),
    })?;

    let lang_code = v.get("lang_code").and_then(Value::as_str);
    if let Some(want) = filter_lang_code {
        if lang_code != Some(want) {
            return Err(ParseIssue {
                line: line_no,
                reason: format!("lang_code != {want} (filtered, expected — không phải lỗi đọc)"),
            });
        }
    }

    let word = v
        .get("word")
        .and_then(Value::as_str)
        .ok_or_else(|| ParseIssue {
            line: line_no,
            reason: "missing 'word' field".to_string(),
        })?;

    let pos_label = v
        .get("pos_title")
        .and_then(Value::as_str)
        .or_else(|| v.get("pos").and_then(Value::as_str));

    let reading = v.get("sounds").and_then(Value::as_array).and_then(|arr| {
        arr.iter().find_map(|s| {
            let tags = s.get("tags").and_then(Value::as_array);
            let is_pinyin = tags.is_some_and(|t| {
                t.iter().any(|x| x.as_str() == Some("Pinyin"))
                    && t.iter().any(|x| x.as_str() == Some("Mandarin"))
            });
            if is_pinyin {
                s.get("zh_pron").and_then(Value::as_str).map(String::from)
            } else {
                None
            }
        })
    });

    let senses_raw = v.get("senses").and_then(Value::as_array);
    let mut senses = Vec::new();
    if let Some(arr) = senses_raw {
        for s in arr {
            let glosses = s
                .get("glosses")
                .and_then(Value::as_array)
                .map(|g| {
                    g.iter()
                        .filter_map(Value::as_str)
                        .map(String::from)
                        .collect::<Vec<_>>()
                })
                .unwrap_or_default();
            if glosses.is_empty() {
                // Ca thật đã thấy trên dữ liệu vi-edition: `senses: [{"tags":["no-gloss"]}]`
                // — mục từ liên kết nhưng chưa có nghĩa. Bỏ qua NGHĨA đó, ⛔ không bỏ
                // qua cả DÒNG: các nghĩa khác của cùng từ vẫn hợp lệ.
                continue;
            }
            let gloss = glosses.join("; ");

            let examples = s
                .get("examples")
                .and_then(Value::as_array)
                .map(|exs| {
                    exs.iter()
                        .filter_map(|e| {
                            let text = e.get("text").and_then(Value::as_str)?.to_string();
                            let translation = e
                                .get("english")
                                .or_else(|| e.get("translation"))
                                .and_then(Value::as_str)
                                .map(String::from);
                            let translation_lang =
                                translation.as_ref().map(|_| "en".to_string());
                            Some(RawExample {
                                text,
                                translation,
                                translation_lang,
                            })
                        })
                        .collect::<Vec<_>>()
                })
                .unwrap_or_default();

            senses.push(RawSense {
                pos: pos_label.map(String::from),
                pos_lang: pos_label.map(|_| pos_lang.to_string()),
                gloss,
                note: None,
                examples,
                citations: Vec::new(),
            });
        }
    }

    if senses.is_empty() {
        return Err(ParseIssue {
            line: line_no,
            reason: "no usable glosses on any sense".to_string(),
        });
    }

    Ok(Some(RawEntry {
        lang: "zh".to_string(),
        headword: word.to_string(),
        headword_simp: None,
        reading,
        han_viet: None,
        senses,
    }))
}

/// Gộp mọi dòng JSONL CÙNG `headword` — TRONG CÙNG một nguồn/lượt gọi — thành MỘT
/// `RawEntry` với nhiều `RawSense`. kaikki/Wiktextract phát MỘT dòng JSONL cho MỖI
/// (headword, từ loại), nên một đầu mục nhiều từ loại nằm ở NHIỀU dòng — AC1 mệnh đề 4
/// đòi "một từ nhiều từ loại ⇒ nhiều hàng `dict_sense`", hàm ý một `entry_id`, không
/// phải nhiều. Đã thấy thật: `馬` xuất hiện ở BA dòng JSONL riêng trong dữ liệu thật
/// (`tests/fixtures/raw/en_wiktionary/Chinese.jsonl`, trích nguyên văn từ
/// `tools/dict-build/raw/en_wiktionary/Chinese.jsonl` thật) — hai dòng có nghĩa dùng
/// được (`pos: "character"`) cộng một dòng thật `tags: ["no-gloss"]` (không có nghĩa,
/// bị bỏ qua đúng như `parse_line` xử lý, ⛔ không chặn việc gộp hai dòng còn lại).
///
/// dict-build:allow .entry( — gộp senses THEO HEADWORD, TRONG một nguồn/lượt gọi (AC2:
/// khử trùng lặp trong MỘT `source_id` là hợp lệ); ⛔ KHÔNG xuyên nguồn (AD-19) — mỗi
/// caller (`viwiktionary`/`en_wiktionary`) gọi hàm này RIÊNG trên tệp của chính nó,
/// không headword nào đi qua hai lượt gọi khác nguồn.
///
/// ⚠️ KHÔNG lazy theo dòng (cùng lý do với `sources::unihan`) — phải đọc hết mới biết
/// một headword còn xuất hiện ở dòng sau hay không. Chỉ tích luỹ dữ liệu ĐÃ TRÍCH XUẤT
/// (`RawEntry`), không giữ JSON thô, nên không tỉ lệ thuận với kích thước tệp gốc.
pub fn parse<R: BufRead>(
    reader: R,
    pos_lang: &str,
    filter_lang_code: Option<&str>,
) -> impl Iterator<Item = Result<RawEntry, ParseIssue>> {
    let mut issues: Vec<ParseIssue> = Vec::new();
    let mut order: Vec<String> = Vec::new();
    let mut by_headword: HashMap<String, RawEntry> = HashMap::new();

    for (idx, line) in reader.lines().enumerate() {
        let line_no = idx + 1;
        let raw = match line {
            Ok(l) => l,
            Err(e) => {
                issues.push(ParseIssue {
                    line: line_no,
                    reason: format!("I/O error: {e}"),
                });
                continue;
            }
        };
        match parse_line(line_no, &raw, pos_lang, filter_lang_code) {
            Ok(None) => {}
            Ok(Some(entry)) => {
                if let Some(existing) = by_headword.get_mut(&entry.headword) {
                    existing.senses.extend(entry.senses);
                    if existing.reading.is_none() {
                        existing.reading = entry.reading;
                    }
                } else {
                    order.push(entry.headword.clone());
                    by_headword.insert(entry.headword.clone(), entry);
                }
            }
            Err(issue) => issues.push(issue),
        }
    }

    let entries = order.into_iter().filter_map(move |hw| by_headword.remove(&hw)).map(Ok);

    issues.into_iter().map(Err).chain(entries)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn parses_a_real_shaped_entry() {
        let json = r#"{"word":"詞典","pos":"noun","lang":"Chinese","lang_code":"zh",
          "senses":[{"glosses":["dictionary"],"examples":[{"text":"詞典很好用","english":"The dictionary is very useful"}]}],
          "sounds":[{"zh_pron":"cí diǎn","tags":["Mandarin","Pinyin"]}]}"#;
        let entry = parse_line(1, json, "en", Some("zh")).unwrap().unwrap();
        assert_eq!(entry.headword, "詞典");
        assert_eq!(entry.reading.as_deref(), Some("cí diǎn"));
        assert_eq!(entry.senses.len(), 1);
        assert_eq!(entry.senses[0].gloss, "dictionary");
        assert_eq!(entry.senses[0].pos_lang.as_deref(), Some("en"));
        assert_eq!(entry.senses[0].examples.len(), 1);
        assert_eq!(
            entry.senses[0].examples[0].translation.as_deref(),
            Some("The dictionary is very useful")
        );
    }

    #[test]
    fn prefers_pos_title_when_present() {
        let json = r#"{"word":"ăn","pos":"verb","pos_title":"Động từ","lang_code":"zh",
          "senses":[{"glosses":["to eat"]}]}"#;
        let entry = parse_line(1, json, "vi", Some("zh")).unwrap().unwrap();
        assert_eq!(entry.senses[0].pos.as_deref(), Some("Động từ"));
    }

    #[test]
    fn non_matching_lang_code_is_filtered_not_an_entry() {
        let json = r#"{"word":"ăn","pos":"verb","lang_code":"cmo","senses":[{"glosses":["cho."]}]}"#;
        let result = parse_line(1, json, "vi", Some("zh"));
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .reason
                .contains("filtered, expected")
        );
    }

    #[test]
    fn a_sense_with_no_gloss_tag_is_skipped_not_the_whole_entry() {
        let json = r#"{"word":"词典","pos":"unknown","lang_code":"zh",
          "senses":[{"tags":["no-gloss"]},{"glosses":["dictionary"]}]}"#;
        let entry = parse_line(1, json, "en", Some("zh")).unwrap().unwrap();
        assert_eq!(entry.senses.len(), 1);
        assert_eq!(entry.senses[0].gloss, "dictionary");
    }

    #[test]
    fn entry_with_zero_usable_senses_is_an_error() {
        let json = r#"{"word":"词典","pos":"unknown","lang_code":"zh","senses":[{"tags":["no-gloss"]}]}"#;
        assert!(parse_line(1, json, "en", Some("zh")).is_err());
    }

    #[test]
    fn invalid_json_is_reported_not_panicked() {
        assert!(parse_line(1, "{not json", "en", None).is_err());
    }

    #[test]
    fn blank_line_is_none_not_err() {
        assert!(parse_line(1, "   ", "en", None).unwrap().is_none());
    }

    #[test]
    fn missing_word_field_is_an_error() {
        let json = r#"{"pos":"noun","senses":[{"glosses":["x"]}]}"#;
        assert!(parse_line(1, json, "en", None).is_err());
    }

    /// AC1 mệnh đề 4, hình dạng thật của `馬` trong fixture en.wiktionary: hai dòng
    /// JSONL riêng, cùng headword, khác nghĩa — phải gộp thành MỘT `RawEntry` với HAI
    /// `RawSense`, ⛔ không phải hai `RawEntry` (Review Findings Group A).
    #[test]
    fn same_headword_on_two_lines_becomes_one_entry_with_two_senses() {
        let text = "\
{\"word\":\"馬\",\"pos\":\"character\",\"lang_code\":\"zh\",\"senses\":[{\"glosses\":[\"horse (radical 187)\"]}]}
{\"word\":\"馬\",\"pos\":\"character\",\"lang_code\":\"zh\",\"senses\":[{\"glosses\":[\"surname\"]}]}
";
        let entries: Vec<RawEntry> = parse(Cursor::new(text.as_bytes()), "en", Some("zh"))
            .collect::<Result<_, _>>()
            .unwrap();
        assert_eq!(entries.len(), 1, "one headword across two JSONL lines must be one dict_entry");
        assert_eq!(entries[0].senses.len(), 2);
        assert_eq!(entries[0].senses[0].gloss, "horse (radical 187)");
        assert_eq!(entries[0].senses[1].gloss, "surname");
    }

    #[test]
    fn different_headwords_stay_as_separate_entries() {
        let text = "\
{\"word\":\"馬\",\"pos\":\"character\",\"lang_code\":\"zh\",\"senses\":[{\"glosses\":[\"horse\"]}]}
{\"word\":\"山\",\"pos\":\"character\",\"lang_code\":\"zh\",\"senses\":[{\"glosses\":[\"mountain\"]}]}
";
        let entries: Vec<RawEntry> = parse(Cursor::new(text.as_bytes()), "en", Some("zh"))
            .collect::<Result<_, _>>()
            .unwrap();
        assert_eq!(entries.len(), 2);
    }
}
