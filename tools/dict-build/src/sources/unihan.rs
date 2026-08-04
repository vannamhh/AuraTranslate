//! Unihan — `unicode.org` → `Unihan.zip`, tám tệp `.txt` dạng
//! `U+XXXX<TAB>kProperty<TAB>value`, UTF-8/NFC. Giấy phép **Unicode License**.
//!
//! Chỉ dùng bốn thuộc tính (§Thông tin kỹ thuật của Story 1.9): `kDefinition` (nghĩa
//! tiếng Anh), `kMandarin` (pinyin), `kVietnamese` (âm Hán Việt — nạp vào
//! `dict_entry.han_viet`, ⛔ KHÔNG thành `dict_sense`), `kSimplifiedVariant` (nối sang
//! `headword_simp`). Vì bốn thuộc tính này rải trên nhiều tệp khác nhau nhưng property
//! name tự mô tả (không cần biết tệp gốc), `build.rs` NỐI nội dung các tệp cần dùng
//! thành MỘT reader rồi đưa vào đây — dòng nào mang property lạ bị bỏ qua êm (không
//! phải lỗi, chỉ là thuộc tính ngoài phạm vi story này).
//!
//! Một ký tự trải nhiều dòng (mỗi dòng một thuộc tính) ⇒ không thể lazy theo từng dòng
//! như CEDICT/JSONL. Hàm này gom cả tệp vào bộ nhớ, tích luỹ theo mã điểm, rồi sinh
//! `RawEntry` khi đã đọc hết — vẫn đúng chữ ký `parse(reader) -> impl Iterator<Item =
//! Result<RawEntry>>`, chỉ khác ở chỗ KHÔNG lazy.

use std::collections::BTreeMap;
use std::io::BufRead;

use crate::model::{ParseIssue, RawEntry, RawSense};

pub const SOURCE_CODE: &str = "unihan";

#[derive(Default)]
struct Accum {
    definition: Option<String>,
    mandarin: Option<String>,
    vietnamese: Option<String>,
    simplified_variant: Option<char>,
}

fn parse_codepoint(field: &str) -> Option<char> {
    let hex = field.strip_prefix("U+")?;
    let n = u32::from_str_radix(hex, 16).ok()?;
    char::from_u32(n)
}

/// Cùng chữ ký với bốn parser còn lại: `parse(reader) -> impl Iterator<Item = Result<RawEntry>>`.
///
/// ⚠️ KHÔNG lazy (xem doc-comment module) — toàn bộ `reader` được đọc trước khi bất kỳ
/// mục nào được sinh ra.
pub fn parse<R: BufRead>(reader: R) -> impl Iterator<Item = Result<RawEntry, ParseIssue>> {
    let mut issues: Vec<ParseIssue> = Vec::new();
    let mut accum: BTreeMap<char, Accum> = BTreeMap::new();

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
        let trimmed = raw.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        let mut parts = trimmed.splitn(3, '\t');
        let (cp_field, prop, value) = match (parts.next(), parts.next(), parts.next()) {
            (Some(a), Some(b), Some(c)) => (a, b, c),
            _ => {
                issues.push(ParseIssue {
                    line: line_no,
                    reason: "expected 3 tab-separated fields (codepoint, property, value)"
                        .to_string(),
                });
                continue;
            }
        };
        let ch = match parse_codepoint(cp_field) {
            Some(c) => c,
            None => {
                issues.push(ParseIssue {
                    line: line_no,
                    reason: format!("unparseable codepoint field '{cp_field}'"),
                });
                continue;
            }
        };

        // dict-build:allow .entry( — tích luỹ THUỘC TÍNH của MỘT ký tự Unihan qua nhiều
        // dòng cùng nguồn; không hợp nhất NGHĨA giữa các nguồn khác nhau (AD-19).
        let entry = accum.entry(ch).or_default();
        match prop {
            "kDefinition" => {
                // Giá trị rỗng-sau-trim không tới được đây: dòng bị `raw.trim()` ở
                // đầu vòng lặp (dòng 58) xén luôn TAB phân cách cuối cùng, nên rơi
                // vào nhánh "expected 3 tab-separated fields" ở trên trước khi tới
                // match này — đã xác nhận bằng test, không cần canh gác thêm ở đây.
                if entry.definition.is_some() {
                    issues.push(ParseIssue {
                        line: line_no,
                        reason: format!("duplicate kDefinition for {cp_field} (kept first)"),
                    });
                } else {
                    entry.definition = Some(value.trim().to_string());
                }
            }
            "kMandarin" => {
                // Có thể liệt kê nhiều âm đọc cách nhau bằng khoảng trắng — lấy âm đầu
                // tiên làm `reading` chính, đủ cho tra cứu (đường tra cứu đầy đủ là
                // Story 1.11, không phải story này).
                if entry.mandarin.is_some() {
                    issues.push(ParseIssue {
                        line: line_no,
                        reason: format!("duplicate kMandarin for {cp_field} (kept first)"),
                    });
                } else {
                    entry.mandarin = value.split_whitespace().next().map(|s| s.to_string());
                }
            }
            "kVietnamese" => {
                if entry.vietnamese.is_some() {
                    issues.push(ParseIssue {
                        line: line_no,
                        reason: format!("duplicate kVietnamese for {cp_field} (kept first)"),
                    });
                } else {
                    entry.vietnamese = Some(value.trim().to_string());
                }
            }
            "kSimplifiedVariant" => {
                // Giá trị có thể liệt kê nhiều biến thể; lấy biến thể đầu tiên.
                if let Some(first) = value.split_whitespace().next() {
                    if entry.simplified_variant.is_some() {
                        issues.push(ParseIssue {
                            line: line_no,
                            reason: format!("duplicate kSimplifiedVariant for {cp_field} (kept first)"),
                        });
                    } else {
                        match parse_codepoint(first) {
                            Some(c) => entry.simplified_variant = Some(c),
                            None => issues.push(ParseIssue {
                                line: line_no,
                                reason: format!("unparseable kSimplifiedVariant codepoint '{first}'"),
                            }),
                        }
                    }
                }
            }
            _ => { /* thuộc tính ngoài phạm vi story này — bỏ qua êm, không phải lỗi */ }
        }
    }

    let entries = accum
        .into_iter()
        .filter(|(_, a)| {
            a.definition.is_some()
                || a.mandarin.is_some()
                || a.vietnamese.is_some()
                || a.simplified_variant.is_some()
        })
        .map(|(ch, a)| {
            let senses = match a.definition {
                Some(gloss) => vec![RawSense {
                    pos: None,
                    pos_lang: Some("en".to_string()),
                    gloss,
                    note: Some("Unihan kDefinition".to_string()),
                    examples: Vec::new(),
                    citations: Vec::new(),
                }],
                None => Vec::new(),
            };
            RawEntry {
                lang: "zh".to_string(),
                headword: ch.to_string(),
                headword_simp: a.simplified_variant.map(|c| c.to_string()),
                reading: a.mandarin,
                han_viet: a.vietnamese,
                senses,
            }
        });

    issues.into_iter().map(Err).chain(entries.map(Ok))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn accumulates_multiple_properties_of_the_same_character() {
        let text = "\
U+4E2D\tkDefinition\tmiddle; center
U+4E2D\tkMandarin\tzhong1
U+4E2D\tkVietnamese\ttrung
";
        let entries: Vec<_> = parse(Cursor::new(text.as_bytes()))
            .collect::<Result<_, _>>()
            .unwrap();
        assert_eq!(entries.len(), 1);
        let e = &entries[0];
        assert_eq!(e.headword, "中");
        assert_eq!(e.reading.as_deref(), Some("zhong1"));
        assert_eq!(e.han_viet.as_deref(), Some("trung"));
        assert_eq!(e.senses.len(), 1);
        assert_eq!(e.senses[0].pos_lang.as_deref(), Some("en"));
    }

    #[test]
    fn simplified_variant_links_to_headword_simp() {
        let text = "U+8C9E\tkSimplifiedVariant\tU+8D5D\n";
        let entries: Vec<_> = parse(Cursor::new(text.as_bytes()))
            .collect::<Result<_, _>>()
            .unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].headword, "\u{8C9E}");
        assert_eq!(entries[0].headword_simp.as_deref(), Some("\u{8D5D}"));
    }

    #[test]
    fn unrecognized_property_is_ignored_not_an_error() {
        let text = "U+4E2D\tkRSUnicode\t2.1\n";
        let results: Vec<_> = parse(Cursor::new(text.as_bytes())).collect();
        assert_eq!(results.len(), 0);
    }

    #[test]
    fn malformed_line_is_reported_not_panicked() {
        let text = "this is not tab separated at all\n";
        let results: Vec<_> = parse(Cursor::new(text.as_bytes())).collect();
        assert_eq!(results.len(), 1);
        assert!(results[0].is_err());
    }

    #[test]
    fn unparseable_codepoint_is_reported() {
        let text = "NOTHEX\tkDefinition\tfoo\n";
        let results: Vec<_> = parse(Cursor::new(text.as_bytes())).collect();
        assert_eq!(results.len(), 1);
        assert!(results[0].is_err());
    }

    #[test]
    fn unparseable_ksimplifiedvariant_codepoint_is_reported_not_silently_dropped() {
        let text = "U+4E2D\tkSimplifiedVariant\tNOTHEX\n";
        let results: Vec<_> = parse(Cursor::new(text.as_bytes())).collect();
        assert_eq!(results.len(), 1, "must surface a ParseIssue, not vanish with zero trace");
        assert!(results[0].is_err());
        assert!(results[0].as_ref().unwrap_err().reason.contains("kSimplifiedVariant"));
    }

    /// Một dòng `kDefinition` với giá trị rỗng (tab cuối không có gì theo sau) không
    /// bao giờ tới được nhánh gán `entry.definition` — `raw.trim()` xén luôn TAB phân
    /// cách cuối, nên dòng rơi vào lỗi "expected 3 tab-separated fields" TRƯỚC. Test
    /// này khoá đúng hành vi thật (Review Findings Group A: giả thuyết "gloss rỗng
    /// lọt qua" ban đầu KHÔNG tái lập được sau khi truy vết `trim()`).
    #[test]
    fn kdefinition_with_nothing_after_the_trailing_tab_is_a_field_count_error() {
        let text = "U+4E2D\tkDefinition\t\n";
        let results: Vec<_> = parse(Cursor::new(text.as_bytes())).collect();
        assert_eq!(results.len(), 1);
        assert!(results[0].is_err());
        assert!(
            results[0]
                .as_ref()
                .unwrap_err()
                .reason
                .contains("expected 3 tab-separated fields")
        );
    }

    #[test]
    fn duplicate_property_line_keeps_first_value_and_reports_an_issue() {
        let text = "\
U+4E2D\tkDefinition\tmiddle; center
U+4E2D\tkDefinition\tsomething else entirely
";
        let results: Vec<_> = parse(Cursor::new(text.as_bytes())).collect();
        let issues: Vec<_> = results.iter().filter(|r| r.is_err()).collect();
        let entries: Vec<_> = results.iter().filter_map(|r| r.as_ref().ok()).collect();
        assert_eq!(issues.len(), 1, "duplicate must be reported, not silently overwritten");
        assert!(issues[0].as_ref().unwrap_err().reason.contains("duplicate kDefinition"));
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].senses[0].gloss, "middle; center", "first value kept");
    }
}
