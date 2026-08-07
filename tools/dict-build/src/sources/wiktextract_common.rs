//! Đọc dòng JSONL khuôn Wiktextract dùng chung cho `viwiktionary`, `en_wiktionary` và
//! `viwiktionary_en` (Story 1.9 Task 4; tham số hoá ở Story 1.10b Task 2). Ba nguồn cùng
//! công cụ trích xuất (tatuylonen/wiktextract, phân phối qua kaikki.org) nên cùng hình
//! dạng JSON; khác nhau ở **ấn bản** nguồn, ở `pos_lang` gắn vào mỗi nghĩa (FR35), và ở
//! **ngôn ngữ của chính đầu mục**. Đây là chia sẻ CODE ĐỌC JSON, không phải hợp nhất
//! NGHĨA — mỗi caller vẫn tự gắn `source_id` của mình.
//!
//! # 🔴 `filter_lang_code` và `entry_lang` là HAI thứ khác nhau
//!
//! | Tham số | Câu hỏi nó trả lời | Ảnh hưởng tới |
//! |---|---|---|
//! | `filter_lang_code` | *"ĐỌC dòng nào?"* | dòng nào thành `RawEntry`, dòng nào thành `ParseIssue` |
//! | `entry_lang` | *"GHI nhãn gì?"* | `RawEntry.lang` ⇒ cột `dict_entry.lang` |
//! | `pos_lang` | *"nhãn từ loại viết bằng tiếng gì?"* | `RawSense.pos_lang` (FR35) |
//!
//! **Ba thứ này KHÔNG suy ra được từ nhau.** `en_wiktionary` là bằng chứng sống:
//! `filter_lang_code = "zh"` *(đọc mục từ tiếng Trung)* nhưng `pos_lang = "en"` *(nhãn
//! từ loại tiếng Anh, vì đó là ấn bản `en`)*. Và `viwiktionary_en` là bằng chứng chiều
//! còn lại: `filter = "en"` với `pos_lang = "vi"` *(ấn bản `vi` có `pos_title` sẵn tiếng
//! Việt)*.
//!
//! 🔴 `entry_lang` là tham số **BẮT BUỘC**, **không có giá trị mặc định** — và đó là
//! điều có chủ ý. Trước Story 1.10b hàm này viết cứng `lang: "zh"`, nên nguồn thứ sáu
//! *(mục từ tiếng Anh)* sẽ lặng lẽ đổ 119.039 đầu mục MANG NHÃN TIẾNG TRUNG vào
//! `dict_entry` với build XANH và mọi test khác XANH. Một giá trị mặc định `"zh"` biến
//! đúng lỗi đó thành thứ **lặp lại được ở nguồn thứ bảy**; bắt buộc khai báo thì mỗi
//! caller mới buộc phải trả lời câu hỏi *"nguồn của tôi mang nhãn gì?"* ngay lúc biên
//! dịch.
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
///
/// `entry_lang` đi THẲNG vào `RawEntry.lang`. Tham số BẮT BUỘC, không mặc định —
/// xem doc-comment module.
pub fn parse_line(
    line_no: usize,
    raw: &str,
    pos_lang: &str,
    filter_lang_code: Option<&str>,
    entry_lang: &str,
) -> Result<Option<RawEntry>, ParseIssue> {
    debug_assert!(
        matches!(entry_lang, "zh" | "en"),
        "entry_lang không thuộc tập đã biết {{\"zh\",\"en\"}}: {entry_lang:?}"
    );

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
                // — mục từ liên kết nhưng chưa có nghĩa. Bỏ qua NGHĨA đó, không bỏ
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
        lang: entry_lang.to_string(),
        headword: word.to_string(),
        headword_simp: None,
        reading,
        han_viet: None,
        nom_reading: None,
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
/// bị bỏ qua đúng như `parse_line` xử lý, không chặn việc gộp hai dòng còn lại).
///
/// dict-build:allow .entry( — gộp senses THEO HEADWORD, TRONG một nguồn/lượt gọi (AC2:
/// khử trùng lặp trong MỘT `source_id` là hợp lệ); KHÔNG xuyên nguồn (AD-19) — mỗi
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
    entry_lang: &str,
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
        match parse_line(line_no, &raw, pos_lang, filter_lang_code, entry_lang) {
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

// ═════════════════════════════════════════════════════════════════════════════════
// Story 1.10c — `en-wiktionary-vi`, đường đọc THỨ HAI cho khuôn JSONL Wiktextract
// ═════════════════════════════════════════════════════════════════════════════════
//
// ⚠️ Vì sao hai hàm riêng thay vì tham số hoá `parse_line`/`parse`: nguồn này KHÔNG
// trích `glosses`/`examples` (Quyết định #3a — giá trị của nó nằm ở NHÃN gắn trên
// `senses[].related[].tags`, không ở nghĩa). Với dữ liệu THẬT, gọi thẳng `parse_line`
// trên các dòng ký tự Hán-Nôm sẽ LUÔN lỗi "no usable glosses" — mọi dòng `pos:
// "character"` chỉ mang `senses: [{"tags": ["no-gloss"], "related": [...]}]`, không một
// `glosses` nào. Đây vẫn là "dùng lại `wiktextract_common.rs`" theo đúng nghĩa story đòi
// (`⚠️ wiktextract_common.rs đã có sẵn — dùng lại, đừng viết parser thứ hai`): MỘT tệp,
// MỘT chỗ hiểu khuôn JSONL Wiktextract — chỉ khác trường nào được trích ra khỏi nó.

/// Đọc MỘT dòng JSONL, trích âm HÁN VIỆT / NÔM có gắn nhãn TƯỜNG MINH từ
/// `senses[].related[].tags` — dùng cho `en-wiktionary-vi` (Quyết định #3a của Story
/// 1.10c, AC3).
///
/// 🔴 Lọc `pos == "character"` — đo thật trên dữ liệu: 1861/1864 dòng mang nhãn
/// `han-viet-reading` có `pos: "character"`; ba dòng còn lại là TỪ (`夢想`, `公僕`,
/// `金甌`), ngoài phạm vi "âm một ký tự" mà AC3/story đo (§Phát hiện, song song với
/// `Unihan kVietnamese` — cũng chỉ phủ TỪNG KÝ TỰ).
///
/// **Không suy đoán nhãn.** Một `related` thiếu `tags` hoặc mang `tags` khác
/// `han-viet-reading`/`nom-reading` bị BỎ QUA — không được hiểu ngầm là một trong hai
/// loại. Một MỤC không mang nhãn nào trong CẢ HAI tập bị bỏ hẳn (`Err`), không nạp một
/// hàng rỗng (Bẫy 3 của story).
///
/// Nhiều âm cùng loại trên một mục ⇒ nối bằng `,`, GIỮ NGUYÊN chuỗi, không nhân bản entry
/// — cùng tinh thần `thieu_chuu.rs:70` (âm đọc không phải điều kiện tách `dict_entry`).
/// ⚠️ Đây là quy ước tách thứ TƯ trong cùng lược đồ (Thiều Chửu dùng `|`, TVC dùng `,`
/// [trùng ký hiệu, khác nguồn], Unihan dùng khoảng trắng) — Bẫy 4 của story: KHÔNG chuẩn
/// hoá bốn quy ước này về một ở đây; Story 1.16 tách bằng một luật duy nhất (`|`, `,`,
/// khoảng trắng đều là ranh giới).
pub fn parse_reading_line(
    line_no: usize,
    raw: &str,
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

    let pos = v.get("pos").and_then(Value::as_str);
    if pos != Some("character") {
        return Err(ParseIssue {
            line: line_no,
            reason: "pos != character (filtered, expected — không phải lỗi đọc)".to_string(),
        });
    }

    let word = v
        .get("word")
        .and_then(Value::as_str)
        .ok_or_else(|| ParseIssue {
            line: line_no,
            reason: "missing 'word' field".to_string(),
        })?;

    let mut han_viet: Vec<String> = Vec::new();
    let mut nom_reading: Vec<String> = Vec::new();

    if let Some(senses) = v.get("senses").and_then(Value::as_array) {
        for sense in senses {
            let Some(related) = sense.get("related").and_then(Value::as_array) else {
                continue;
            };
            for rel in related {
                let Some(tags) = rel.get("tags").and_then(Value::as_array) else {
                    continue;
                };
                let Some(reading_word) = rel.get("word").and_then(Value::as_str) else {
                    continue;
                };
                let is_hv = tags.iter().any(|t| t.as_str() == Some("han-viet-reading"));
                let is_nom = tags.iter().any(|t| t.as_str() == Some("nom-reading"));
                // dict-build:allow .entry( — dedup TRONG một dòng/một nguồn, không hợp
                // nhất xuyên nguồn (AD-19).
                if is_hv && !han_viet.iter().any(|w| w == reading_word) {
                    han_viet.push(reading_word.to_string());
                }
                if is_nom && !nom_reading.iter().any(|w| w == reading_word) {
                    nom_reading.push(reading_word.to_string());
                }
            }
        }
    }

    if han_viet.is_empty() && nom_reading.is_empty() {
        return Err(ParseIssue {
            line: line_no,
            reason: "no han-viet-reading/nom-reading tags on this entry".to_string(),
        });
    }

    Ok(Some(RawEntry {
        lang: "zh".to_string(),
        headword: word.to_string(),
        headword_simp: None,
        reading: None,
        han_viet: (!han_viet.is_empty()).then(|| han_viet.join(",")),
        nom_reading: (!nom_reading.is_empty()).then(|| nom_reading.join(",")),
        senses: Vec::new(),
    }))
}

/// Gộp mọi dòng JSONL CÙNG headword TRONG một lượt gọi — hiếm trên dữ liệu thật (2 ca:
/// `k`, `𦄂`), nhưng cùng quy ước với [`parse`] (AD-19: gộp TRONG một nguồn hợp lệ, xuyên
/// nguồn thì không).
pub fn parse_readings<R: BufRead>(
    reader: R,
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
        match parse_reading_line(line_no, &raw, filter_lang_code) {
            Ok(None) => {}
            Ok(Some(entry)) => {
                if let Some(existing) = by_headword.get_mut(&entry.headword) {
                    existing.han_viet =
                        union_reading_lists(existing.han_viet.take(), entry.han_viet);
                    existing.nom_reading =
                        union_reading_lists(existing.nom_reading.take(), entry.nom_reading);
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

/// Hợp hai chuỗi âm đọc phân tách bằng `,`, khử trùng lặp, giữ thứ tự xuất hiện.
fn union_reading_lists(a: Option<String>, b: Option<String>) -> Option<String> {
    match (a, b) {
        (None, x) => x,
        (x, None) => x,
        (Some(a), Some(b)) => {
            let mut parts: Vec<String> = a.split(',').map(str::to_string).collect();
            for p in b.split(',') {
                if !parts.iter().any(|existing| existing == p) {
                    parts.push(p.to_string());
                }
            }
            Some(parts.join(","))
        }
    }
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
        let entry = parse_line(1, json, "en", Some("zh"), "zh").unwrap().unwrap();
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
        let entry = parse_line(1, json, "vi", Some("zh"), "zh").unwrap().unwrap();
        assert_eq!(entry.senses[0].pos.as_deref(), Some("Động từ"));
    }

    #[test]
    fn non_matching_lang_code_is_filtered_not_an_entry() {
        let json = r#"{"word":"ăn","pos":"verb","lang_code":"cmo","senses":[{"glosses":["cho."]}]}"#;
        let result = parse_line(1, json, "vi", Some("zh"), "zh");
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
        let entry = parse_line(1, json, "en", Some("zh"), "zh").unwrap().unwrap();
        assert_eq!(entry.senses.len(), 1);
        assert_eq!(entry.senses[0].gloss, "dictionary");
    }

    #[test]
    fn entry_with_zero_usable_senses_is_an_error() {
        let json = r#"{"word":"词典","pos":"unknown","lang_code":"zh","senses":[{"tags":["no-gloss"]}]}"#;
        assert!(parse_line(1, json, "en", Some("zh"), "zh").is_err());
    }

    #[test]
    fn invalid_json_is_reported_not_panicked() {
        assert!(parse_line(1, "{not json", "en", None, "zh").is_err());
    }

    #[test]
    fn blank_line_is_none_not_err() {
        assert!(parse_line(1, "   ", "en", None, "zh").unwrap().is_none());
    }

    #[test]
    fn missing_word_field_is_an_error() {
        let json = r#"{"pos":"noun","senses":[{"glosses":["x"]}]}"#;
        assert!(parse_line(1, json, "en", None, "zh").is_err());
    }

    /// AC1 mệnh đề 4, hình dạng thật của `馬` trong fixture en.wiktionary: hai dòng
    /// JSONL riêng, cùng headword, khác nghĩa — phải gộp thành MỘT `RawEntry` với HAI
    /// `RawSense`, không phải hai `RawEntry` (Review Findings Group A).
    #[test]
    fn same_headword_on_two_lines_becomes_one_entry_with_two_senses() {
        let text = "\
{\"word\":\"馬\",\"pos\":\"character\",\"lang_code\":\"zh\",\"senses\":[{\"glosses\":[\"horse (radical 187)\"]}]}
{\"word\":\"馬\",\"pos\":\"character\",\"lang_code\":\"zh\",\"senses\":[{\"glosses\":[\"surname\"]}]}
";
        let entries: Vec<RawEntry> = parse(Cursor::new(text.as_bytes()), "en", Some("zh"), "zh")
            .collect::<Result<_, _>>()
            .unwrap();
        assert_eq!(entries.len(), 1, "one headword across two JSONL lines must be one dict_entry");
        assert_eq!(entries[0].senses.len(), 2);
        assert_eq!(entries[0].senses[0].gloss, "horse (radical 187)");
        assert_eq!(entries[0].senses[1].gloss, "surname");
    }

    /// 🔴 §Bẫy 1 của Story 1.10b. `parse_line` từng viết cứng `lang: "zh"`, nên một
    /// nguồn tiếng Anh dựng qua hàm này sẽ đổ 119.039 đầu mục MANG NHÃN TIẾNG TRUNG vào
    /// `dict_entry` mà build vẫn XANH. Test này khoá `entry_lang` đi THẲNG vào
    /// `RawEntry.lang` — cho CẢ HAI giá trị, không chỉ giá trị mới.
    #[test]
    fn entry_lang_lands_verbatim_in_raw_entry_lang_for_both_values() {
        let zh = r#"{"word":"字典","pos":"noun","lang_code":"zh","senses":[{"glosses":["từ điển"]}]}"#;
        let en = r#"{"word":"dictionary","pos":"noun","lang_code":"en","senses":[{"glosses":["từ điển"]}]}"#;

        let zh_entry = parse_line(1, zh, "vi", Some("zh"), "zh").unwrap().unwrap();
        assert_eq!(zh_entry.lang, "zh", "vai B phải giữ nguyên nhãn 'zh'");

        let en_entry = parse_line(1, en, "vi", Some("en"), "en").unwrap().unwrap();
        assert_eq!(en_entry.lang, "en", "vai A phải mang nhãn 'en', không phải 'zh'");
    }

    /// `filter_lang_code` và `entry_lang` là HAI thứ khác nhau và KHÔNG suy ra được
    /// từ nhau. `en_wiktionary` là bằng chứng sống của chiều ngược lại (`filter="zh"`,
    /// `pos_lang="en"`); test này khoá chiều còn lại — lọc một `lang_code` rồi dán một
    /// nhãn khác vẫn phải chạy, vì đó đúng là thứ hàm này được yêu cầu làm.
    #[test]
    fn filter_lang_code_and_entry_lang_are_independent_knobs() {
        let json = r#"{"word":"馬","pos":"character","lang_code":"zh","senses":[{"glosses":["ngựa"]}]}"#;
        let entry = parse_line(1, json, "vi", Some("zh"), "en").unwrap().unwrap();
        assert_eq!(entry.headword, "馬");
        assert_eq!(
            entry.lang, "en",
            "entry_lang không được suy ra từ filter_lang_code"
        );
    }

    /// Phép gộp theo headword không được làm rơi `entry_lang` của lượt gọi.
    ///
    /// ⚠️ Tên test cố ý không chứa từ vựng hợp nhất tiếng Anh — Kiểm A của
    /// `check-dict-build.mjs` quét token đó trên toàn cây `src/**` và không phân biệt
    /// mã thật với tên test. Nới miễn trừ cho một tên test là làm hỏng chính phép kiểm.
    #[test]
    fn two_lines_of_one_headword_keep_the_entry_lang_of_the_call() {
        let text = "\
{\"word\":\"set\",\"pos\":\"noun\",\"lang_code\":\"en\",\"senses\":[{\"glosses\":[\"bộ\"]}]}
{\"word\":\"set\",\"pos\":\"verb\",\"lang_code\":\"en\",\"senses\":[{\"glosses\":[\"đặt\"]}]}
";
        let entries: Vec<RawEntry> = parse(Cursor::new(text.as_bytes()), "vi", Some("en"), "en")
            .collect::<Result<_, _>>()
            .unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].senses.len(), 2);
        assert_eq!(entries[0].lang, "en");
    }

    #[test]
    fn different_headwords_stay_as_separate_entries() {
        let text = "\
{\"word\":\"馬\",\"pos\":\"character\",\"lang_code\":\"zh\",\"senses\":[{\"glosses\":[\"horse\"]}]}
{\"word\":\"山\",\"pos\":\"character\",\"lang_code\":\"zh\",\"senses\":[{\"glosses\":[\"mountain\"]}]}
";
        let entries: Vec<RawEntry> = parse(Cursor::new(text.as_bytes()), "en", Some("zh"), "zh")
            .collect::<Result<_, _>>()
            .unwrap();
        assert_eq!(entries.len(), 2);
    }

    // ═════════════════════════════════════════════════════════════════════════════
    // Story 1.10c — `parse_reading_line`/`parse_readings` (en-wiktionary-vi)
    // ═════════════════════════════════════════════════════════════════════════════

    /// 🔴 Ca thật `北` trích nguyên văn từ kaikki.org/dictionary/Vietnamese (2026-08-06):
    /// một `han-viet-reading` ("bắc") cộng NHIỀU `nom-reading`. Đây đúng là ví dụ
    /// `EXPERIENCE.md:410` dùng cho FR113 (`北涼 → Bắc Lương`).
    #[test]
    fn real_shaped_bac_character_yields_han_viet_and_nom_reading() {
        let json = r#"{"pos": "character", "word": "北", "lang": "Vietnamese", "lang_code": "vi", "senses": [{"tags": ["no-gloss"], "related": [{"word": "bắc", "tags": ["han-viet-reading"]}, {"word": "bậc", "tags": ["nom-reading"]}, {"word": "bấc", "tags": ["nom-reading"]}]}]}"#;
        let entry = parse_reading_line(1, json, Some("vi")).unwrap().unwrap();
        assert_eq!(entry.headword, "北");
        assert_eq!(entry.han_viet.as_deref(), Some("bắc"));
        assert_eq!(entry.nom_reading.as_deref(), Some("bậc,bấc"));
        assert!(entry.senses.is_empty(), "Quyết định #3a: không nạp dict_sense");
    }

    /// AC3: nhiều âm CÙNG LOẠI trên một mục ⇒ nối bằng `,`, GIỮ NGUYÊN chuỗi, khử trùng
    /// lặp — ca thật `西` (tây, tê).
    #[test]
    fn multiple_han_viet_readings_on_one_character_are_comma_joined() {
        let json = r#"{"pos": "character", "word": "西", "lang_code": "vi", "senses": [{"related": [{"word": "tây", "tags": ["han-viet-reading"]}, {"word": "tê", "tags": ["han-viet-reading"]}, {"word": "tây", "tags": ["han-viet-reading"]}]}]}"#;
        let entry = parse_reading_line(1, json, Some("vi")).unwrap().unwrap();
        assert_eq!(entry.han_viet.as_deref(), Some("tây,tê"), "trùng lặp 'tây' phải bị khử");
    }

    /// Bẫy 3 của story: một mục KHÔNG mang tag nào trong hai tập ⇒ BỎ (`Err`), không
    /// suy đoán và không nạp một hàng rỗng.
    #[test]
    fn a_character_with_no_han_viet_or_nom_tags_is_not_ingested() {
        let json = r#"{"pos": "character", "word": "字", "lang_code": "vi", "senses": [{"related": [{"word": "chữ", "tags": ["letter"]}]}]}"#;
        let result = parse_reading_line(1, json, Some("vi"));
        assert!(result.is_err());
        assert!(result.unwrap_err().reason.contains("no han-viet-reading/nom-reading"));
    }

    /// AC3: chỉ đọc mục `pos == "character"` — ba từ đa âm tiết thật (`夢想`, `公僕`,
    /// `金甌`) mang `han-viet-reading` nhưng `pos != "character"` bị LỌC, không phải lỗi
    /// đọc.
    #[test]
    fn non_character_pos_is_filtered_not_an_entry() {
        let json = r#"{"pos": "noun", "word": "公僕", "lang_code": "vi", "senses": [{"related": [{"word": "công bộc", "tags": ["han-viet-reading"]}]}]}"#;
        let result = parse_reading_line(1, json, Some("vi"));
        assert!(result.is_err());
        assert!(result.unwrap_err().reason.contains("pos != character"));
    }

    /// Đối chứng âm cho Bẫy 2 của story: một `related` mang nhãn KHÁC (`letter`,
    /// `Latin`+`character`) không được hiểu ngầm là han-viet-reading hay nom-reading.
    #[test]
    fn unrelated_tags_are_never_misread_as_han_viet_or_nom() {
        let json = r#"{"pos": "character", "word": "A", "lang_code": "vi", "senses": [{"related": [{"word": "a", "tags": ["Latin", "character"]}]}]}"#;
        let result = parse_reading_line(1, json, Some("vi"));
        assert!(result.is_err(), "tag lạ không được biến thành han-viet-reading/nom-reading ngầm định");
    }

    #[test]
    fn filtered_lang_code_is_an_error_not_a_silent_skip() {
        let json = r#"{"pos": "character", "word": "北", "lang_code": "en", "senses": [{"related": [{"word": "bắc", "tags": ["han-viet-reading"]}]}]}"#;
        let result = parse_reading_line(1, json, Some("vi"));
        assert!(result.is_err());
        assert!(result.unwrap_err().reason.contains("filtered, expected"));
    }

    /// `parse_readings` gộp CÙNG headword TRONG một lượt gọi (ca hiếm nhưng thật: `k`,
    /// `𦄂`) — hợp cả hai tập, khử trùng lặp, không nhân đôi entry.
    #[test]
    fn parse_readings_unions_duplicate_headword_lines() {
        let text = "\
{\"pos\": \"character\", \"word\": \"k\", \"lang_code\": \"vi\", \"senses\": [{\"related\": [{\"word\": \"ca\", \"tags\": [\"han-viet-reading\"]}]}]}
{\"pos\": \"character\", \"word\": \"k\", \"lang_code\": \"vi\", \"senses\": [{\"related\": [{\"word\": \"cơ\", \"tags\": [\"nom-reading\"]}]}]}
";
        let entries: Vec<RawEntry> = parse_readings(Cursor::new(text.as_bytes()), Some("vi"))
            .collect::<Result<_, _>>()
            .unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].han_viet.as_deref(), Some("ca"));
        assert_eq!(entries[0].nom_reading.as_deref(), Some("cơ"));
    }

    #[test]
    fn parse_readings_skips_blank_lines_and_reports_invalid_json() {
        let text = "\n{not json\n";
        let results: Vec<_> = parse_readings(Cursor::new(text.as_bytes()), Some("vi")).collect();
        assert_eq!(results.len(), 1);
        assert!(results[0].is_err());
    }
}
