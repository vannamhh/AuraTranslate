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
//! Toàn bộ phần sau `]` là MỘT `gloss` — tệp không mang cấu trúc `<br>`/số thứ tự thống
//! nhất như Thiều Chửu (một số dòng dùng số khoanh tròn ①②③, một số khác không), nên
//! Task 5 của story không đòi tách nhiều nghĩa; xem Testing standards.
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
pub const SOURCE_VERSION: &str = "catusf/tudien@a7dd918ecc67de8c2d15034f885d919b9295eba4 (2025-12-19)";

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
                    reason: "expected 2 tab-separated columns (headword, '[reading] definition')".to_string(),
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
            senses: vec![RawSense {
                pos: None,
                pos_lang: None,
                gloss: definition.to_string(),
                note: None,
                examples: Vec::new(),
                citations: Vec::new(),
            }],
        }))
    })
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
        assert!(e.senses[0].gloss.starts_with("① Khoèo cả hai chân"));
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
        assert!(results[0].as_ref().unwrap_err().reason.contains("[reading]"));
    }

    #[test]
    fn unterminated_bracket_is_a_parse_issue() {
        let text = "字\t[tích no closing bracket\n";
        let results: Vec<_> = parse(Cursor::new(text.as_bytes())).collect();
        assert_eq!(results.len(), 1);
        assert!(results[0].is_err());
        assert!(results[0].as_ref().unwrap_err().reason.contains("unterminated"));
    }

    #[test]
    fn empty_definition_after_bracket_is_a_parse_issue() {
        let text = "字\t[tích]\n";
        let results: Vec<_> = parse(Cursor::new(text.as_bytes())).collect();
        assert_eq!(results.len(), 1);
        assert!(results[0].is_err());
        assert!(results[0].as_ref().unwrap_err().reason.contains("empty definition"));
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
        assert_eq!(entries.len(), 2, "TVC không gộp theo headword, mỗi dòng nguồn = một dict_entry");
        assert_eq!(entries[0].han_viet.as_deref(), Some("hành, hạnh, hàng, hạng"));
        assert_eq!(entries[1].han_viet.as_deref(), Some("hạnh"));
    }

    /// `SOURCE_CODE` không được trôi khỏi `sources_meta::TRAN_VAN_CHANH.code`.
    #[test]
    fn source_code_matches_its_source_meta() {
        assert_eq!(SOURCE_CODE, crate::sources_meta::TRAN_VAN_CHANH.code);
    }
}
