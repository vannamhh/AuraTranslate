//! Trần Văn Chánh — dữ liệu Hán Việt bổ sung, `catusf/tudien` →
//! `dict/Tu-dien-ThienChuu-TranVanChanh.tab`. TSV **2 cột**: chữ Hán · `[âm(, âm...)]
//! định nghĩa`. Lớp GỠ RỜI **thứ ba**, Story 1.10c (Quyết định #1, Ice chốt 2026-08-06).
//!
//! # 🔴 Rủi ro pháp lý — xem `sources_meta::TRAN_VAN_CHANH` và `assets/licenses/
//! tran-van-chanh.txt`
//!
//! *Từ điển Hán Việt* (Trần Văn Chánh, 1999) **còn trong bản quyền** — tác giả còn sống.
//! `catusf/tudien` khai CC0-1.0 cho **repo**, nhưng CC0 do **người số hoá** tuyên bố không
//! **không** xoá được bản quyền của tác phẩm **gốc**. Giảm thiểu: đóng gói làm **lớp gỡ
//! rời** ⇒ FR112 thực thi bằng xoá một tệp (AC8).
//!
//! # Định dạng
//!
//! ```text
//! 躄\t[tích] ① Khoèo cả hai chân gọi là tích 躄, khoèo một chân gọi là bả 跛 (…).
//! 檔\t[đáng, đương] ① Tủ đựng hồ sơ: …
//! ```
//!
//! Nhiều âm tách bằng `,` — **GIỮ NGUYÊN chuỗi trong ngoặc**, đúng tiền lệ
//! `thieu_chuu.rs:70` (âm đọc không phải điều kiện tách `dict_entry`). ⚠️ Đây là quy ước
//! tách thứ HAI dùng dấu phẩy trong cùng lược đồ nhưng ở một NGUỒN khác `en-wiktionary-vi`
//! — cả hai đều "giữ nguyên", không được chuẩn hoá về nhau ở đây (Bẫy 4 của story).
//!
//! Phần sau `]` tách thành nhiều nghĩa tại số khoanh tròn ①②③ kế tiếp; dòng
//! không có số khoanh là MỘT nghĩa. Xem [`split_at_circled_ordinals`].
//!
//! # Review Findings — headword nhiều ký tự KHÔNG bị lọc, có chủ ý
//!
//! Tệp thô trộn cả headword MỘT KÝ TỰ (12.081 đầu mục, con số AC4 đo) lẫn headword
//! NHIỀU KÝ TỰ (từ/cụm từ — phần còn lại của 22.030 dòng). Parser này **không** lọc
//! theo độ dài headword — nhất quán với MỌI parser khác trong crate (`thieu_chuu.rs`,
//! `unihan.rs`, … không nguồn nào tự lọc theo độ dài headword ở tầng build). Việc
//! đọc CHỈ ký tự đơn (nếu một tính năng cần) thuộc về tầng TIÊU THỤ — Story 1.16 —
//! không phải tầng dựng dữ liệu này. Test khoá hành vi này:
//! `tran_van_chanh_does_not_filter_multi_character_headwords_by_design`
//! (`tools/dict-build/tests/parse.rs`).

use std::io::BufRead;

use crate::model::{ParseIssue, RawEntry, RawSense};

pub const SOURCE_CODE: &str = "tran-van-chanh";

/// `catusf/tudien` không gắn thẻ phiên bản cho tệp này — ghim **commit SHA** của
/// `master` tại thời điểm tải (Task 1 của story: *"ghim commit SHA, không `master`"*).
pub const SOURCE_VERSION: &str =
    "catusf/tudien@a7dd918ecc67de8c2d15034f885d919b9295eba4 (2025-12-19)";

/// Cùng chữ ký với các parser còn lại: `parse(reader) -> impl Iterator<Item = Result<RawEntry>>`.
pub fn parse<R: BufRead>(reader: R) -> impl Iterator<Item = Result<RawEntry, ParseIssue>> {
    reader.lines().enumerate().filter_map(move |(idx, line)| {
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
        if raw.trim().is_empty() {
            return None;
        }

        let mut cols = raw.splitn(2, '\t');
        let (headword, rest) = match (cols.next(), cols.next()) {
            (Some(a), Some(b)) => (a.trim(), b),
            _ => {
                return Some(Err(ParseIssue {
                    line: line_no,
                    reason: "expected 2 tab-separated columns (headword, '[reading] definition')"
                        .to_string(),
                }));
            }
        };
        if headword.is_empty() {
            return Some(Err(ParseIssue {
                line: line_no,
                reason: "empty headword (column 1)".to_string(),
            }));
        }

        let rest_trimmed = rest.trim_start();
        let Some(after_open) = rest_trimmed.strip_prefix('[') else {
            return Some(Err(ParseIssue {
                line: line_no,
                reason: "column 2 does not start with '[reading]'".to_string(),
            }));
        };
        let Some(close_idx) = after_open.find(']') else {
            return Some(Err(ParseIssue {
                line: line_no,
                reason: "unterminated '[reading]' bracket".to_string(),
            }));
        };
        // Review Findings — một dấu '[' LỌT vào bên trong ngoặc âm đọc (dữ liệu hỏng/
        // lạc dấu) trước đây lọt thẳng vào `han_viet_raw` vì `close_idx` chỉ bắt `]`
        // ĐẦU TIÊN. Từ chối tường minh thay vì âm thầm nạp một chuỗi han_viet sai hình
        // dạng.
        if after_open[..close_idx].contains('[') {
            return Some(Err(ParseIssue {
                line: line_no,
                reason: "nested '[' inside '[reading]' bracket".to_string(),
            }));
        }

        let han_viet_raw = after_open[..close_idx].trim();
        let definition = after_open[close_idx + 1..].trim();

        if han_viet_raw.is_empty() {
            return Some(Err(ParseIssue {
                line: line_no,
                reason: "empty reading bracket '[]'".to_string(),
            }));
        }
        if definition.is_empty() {
            return Some(Err(ParseIssue {
                line: line_no,
                reason: "empty definition after ']'".to_string(),
            }));
        }

        Some(Ok(RawEntry {
            lang: "zh".to_string(),
            headword: headword.to_string(),
            headword_simp: None,
            reading: None,
            han_viet: Some(han_viet_raw.to_string()),
            nom_reading: None,
            senses: split_at_circled_ordinals(definition)
                .into_iter()
                .map(|gloss| RawSense {
                    pos: None,
                    pos_lang: None,
                    gloss,
                    note: None,
                    examples: Vec::new(),
                    citations: Vec::new(),
                })
                .collect(),
        }))
    })
}

/// Value of a circled ordinal: `①`–`⑳`, `㉑`–`㉟`, `㊱`–`㊿`.
fn circled_ordinal(c: char) -> Option<u32> {
    match c as u32 {
        n @ 0x2460..=0x2473 => Some(n - 0x2460 + 1),
        n @ 0x3251..=0x325F => Some(n - 0x3251 + 21),
        n @ 0x32B1..=0x32BF => Some(n - 0x32B1 + 36),
        _ => None,
    }
}

/// Cuts only at the ordinal equal to the next one expected, the rule `thieu_chuu.rs` uses:
/// some source lines repeat a number (`① ② ②`), and the repeat stays inside its sense.
fn split_at_circled_ordinals(definition: &str) -> Vec<String> {
    let mut pieces = Vec::new();
    let mut expected = 1;
    let mut start = 0;
    for (at, c) in definition.char_indices() {
        if circled_ordinal(c) == Some(expected) {
            pieces.push(definition[start..at].trim().to_string());
            start = at + c.len_utf8();
            expected += 1;
        }
    }
    pieces.push(definition[start..].trim().to_string());
    pieces.retain(|p| !p.is_empty());
    pieces
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    /// Task 5: hình dạng thật `[tích]` — MỘT âm, không dấu phẩy.
    #[test]
    fn single_reading_bracket_shape() {
        let text = "躄\t[tích] ① Khoèo cả hai chân gọi là tích 躄, khoèo một chân gọi là bả 跛 (có chỗ đọc là bí).\n";
        let entries: Vec<_> = parse(Cursor::new(text.as_bytes()))
            .collect::<Result<_, _>>()
            .unwrap();
        assert_eq!(entries.len(), 1);
        let e = &entries[0];
        assert_eq!(e.headword, "躄");
        assert_eq!(e.han_viet.as_deref(), Some("tích"));
        assert_eq!(e.senses.len(), 1);
        assert!(e.senses[0].gloss.starts_with("Khoèo cả hai chân"));
    }

    fn glosses(text: &str) -> Vec<String> {
        let entries: Vec<_> = parse(Cursor::new(text.as_bytes()))
            .collect::<Result<_, _>>()
            .unwrap();
        entries[0].senses.iter().map(|s| s.gloss.clone()).collect()
    }

    #[test]
    fn circled_ordinals_split_the_definition_into_senses_without_the_marker() {
        assert_eq!(
            glosses("及\t[cập] ① Kịp, đến. ② Bằng, như bất cập nhân 不及人. ③ Cùng.\n"),
            vec!["Kịp, đến.", "Bằng, như bất cập nhân 不及人.", "Cùng."]
        );
    }

    #[test]
    fn only_the_next_expected_ordinal_cuts_so_a_repeated_one_stays_in_its_sense() {
        assert_eq!(
            glosses("氐\t[để] ① Nền, gốc. ② Một âm là đê. ② Sao đê 氐. ③ Tên.\n"),
            vec!["Nền, gốc.", "Một âm là đê. ② Sao đê 氐.", "Tên."]
        );
    }

    #[test]
    fn text_before_the_first_ordinal_is_its_own_sense() {
        assert_eq!(
            glosses("乃\t[nãi] Như 迺. ① Bèn. ② Là.\n"),
            vec!["Như 迺.", "Bèn.", "Là."]
        );
    }

    #[test]
    fn ordinals_past_twenty_use_the_enclosed_cjk_block() {
        let mut line = String::from("字\t[tự]");
        for n in 1..=21 {
            let marker = if n <= 20 {
                char::from_u32(0x2460 + n - 1).unwrap()
            } else {
                char::from_u32(0x3251 + n - 21).unwrap()
            };
            line.push_str(&format!(" {marker} n{n}."));
        }
        line.push('\n');
        let senses = glosses(&line);
        assert_eq!(senses.len(), 21);
        assert_eq!(senses[20], "n21.");
    }

    #[test]
    fn a_definition_without_ordinals_stays_one_sense() {
        assert_eq!(glosses("行\t[hạnh] Phẩm hạnh.\n"), vec!["Phẩm hạnh."]);
    }

    /// Task 5: hình dạng thật `[đáng, đương]` — NHIỀU âm, giữ nguyên dấu phẩy + khoảng
    /// trắng đúng như nguồn (không chuẩn hoá).
    #[test]
    fn multi_reading_bracket_shape_keeps_the_verbatim_separator() {
        let text = "檔\t[đáng, đương] ① Tủ đựng hồ sơ: 歸檔 Cất vào tủ hồ sơ.\n";
        let entries: Vec<_> = parse(Cursor::new(text.as_bytes()))
            .collect::<Result<_, _>>()
            .unwrap();
        assert_eq!(entries[0].han_viet.as_deref(), Some("đáng, đương"));
    }

    #[test]
    fn missing_bracket_is_a_parse_issue_not_a_panic() {
        let text = "字\thán no bracket here\n";
        let results: Vec<_> = parse(Cursor::new(text.as_bytes())).collect();
        assert_eq!(results.len(), 1);
        assert!(results[0].is_err());
        assert!(
            results[0]
                .as_ref()
                .unwrap_err()
                .reason
                .contains("[reading]")
        );
    }

    #[test]
    fn unterminated_bracket_is_a_parse_issue() {
        let text = "字\t[tích no closing bracket\n";
        let results: Vec<_> = parse(Cursor::new(text.as_bytes())).collect();
        assert_eq!(results.len(), 1);
        assert!(results[0].is_err());
        assert!(
            results[0]
                .as_ref()
                .unwrap_err()
                .reason
                .contains("unterminated")
        );
    }

    #[test]
    fn empty_definition_after_bracket_is_a_parse_issue() {
        let text = "字\t[tích]\n";
        let results: Vec<_> = parse(Cursor::new(text.as_bytes())).collect();
        assert_eq!(results.len(), 1);
        assert!(results[0].is_err());
        assert!(
            results[0]
                .as_ref()
                .unwrap_err()
                .reason
                .contains("empty definition")
        );
    }

    #[test]
    fn blank_lines_are_skipped_not_errors() {
        let text = "\n\n躄\t[tích] Khoèo.\n";
        let results: Vec<_> = parse(Cursor::new(text.as_bytes())).collect();
        assert_eq!(results.len(), 1);
        assert!(results[0].is_ok());
    }

    /// Ca thật: hai dòng CÙNG headword, một mảnh Thiều-Chửu-style (âm cũ) một mảnh
    /// TVC-style (âm hiện đại) — file KHÔNG gộp trong nguồn này (đối xứng
    /// `thieu_chuu.rs`), mỗi dòng một `dict_entry` riêng.
    #[test]
    fn duplicate_headword_lines_stay_as_separate_entries() {
        let text = "行\t[hành, hạnh, hàng, hạng] ① Bước đi.\n行\t[hạnh] Phẩm hạnh.\n";
        let entries: Vec<_> = parse(Cursor::new(text.as_bytes()))
            .collect::<Result<_, _>>()
            .unwrap();
        assert_eq!(
            entries.len(),
            2,
            "TVC không gộp theo headword, mỗi dòng nguồn = một dict_entry"
        );
        assert_eq!(
            entries[0].han_viet.as_deref(),
            Some("hành, hạnh, hàng, hạng")
        );
        assert_eq!(entries[1].han_viet.as_deref(), Some("hạnh"));
    }

    /// `SOURCE_CODE` không được trôi khỏi `sources_meta::TRAN_VAN_CHANH.code`.
    #[test]
    fn source_code_matches_its_source_meta() {
        assert_eq!(SOURCE_CODE, crate::sources_meta::TRAN_VAN_CHANH.code);
    }
}
