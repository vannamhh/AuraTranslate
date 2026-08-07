//! Bộ đọc dòng dùng chung cho CVDICT và CC-CEDICT — cả hai cùng khuôn văn bản
//! (`phồn giản [pinyin] /nghĩa/nghĩa/`, §Thông tin kỹ thuật của Story 1.9). Đây là
//! chia sẻ CODE PARSE-DÒNG, không phải hợp nhất NGHĨA giữa hai nguồn — mỗi module
//! gọi hàm này vẫn tự gắn `source_id` của chính nó, và không hàng `dict_sense` nào đi
//! qua đây biết tới nguồn kia. AD-19 cấm hợp nhất *nghĩa*, không cấm dùng lại một hàm
//! tách chuỗi.

use crate::model::ParseIssue;

/// Một dòng đã tách, chưa gắn nguồn.
pub struct CedictLine {
    pub traditional: String,
    pub simplified: String,
    pub pinyin: String,
    /// Mỗi phần tử là một nghĩa riêng (tách bằng `/`), đã trim, đã loại rỗng.
    pub glosses: Vec<String>,
}

/// Tách một dòng dữ liệu CEDICT-khuôn. Dòng trống hoặc bắt đầu bằng `#` trả `None`
/// (không phải lỗi — đó là header/comment hợp lệ của định dạng).
///
/// Trả `Err` kèm lý do khi dòng KHÔNG rỗng, KHÔNG phải comment, nhưng không khớp
/// khuôn `TRAD SIMP [PINYIN] /defs/` — dòng hỏng phải được ĐẾM (§Quyết định #8), không
/// không được `panic!`, không được nuốt im lặng.
pub fn parse_line(line_no: usize, raw: &str) -> Result<Option<CedictLine>, ParseIssue> {
    let trimmed = raw.trim_end_matches(['\r', '\n']);
    if trimmed.trim().is_empty() || trimmed.trim_start().starts_with('#') {
        return Ok(None);
    }

    let bracket_open = trimmed.find('[').ok_or_else(|| ParseIssue {
        line: line_no,
        reason: "missing '[' before pinyin field".to_string(),
    })?;
    let bracket_close = trimmed[bracket_open..].find(']').map(|i| i + bracket_open);
    let bracket_close = bracket_close.ok_or_else(|| ParseIssue {
        line: line_no,
        reason: "missing ']' after pinyin field".to_string(),
    })?;

    let head = trimmed[..bracket_open].trim();
    let pinyin = trimmed[bracket_open + 1..bracket_close].trim().to_string();
    let rest = trimmed[bracket_close + 1..].trim();

    let mut head_parts = head.split_whitespace();
    let traditional = head_parts.next().ok_or_else(|| ParseIssue {
        line: line_no,
        reason: "missing traditional headword".to_string(),
    })?;
    let simplified = head_parts.next().ok_or_else(|| ParseIssue {
        line: line_no,
        reason: "missing simplified headword".to_string(),
    })?;
    if head_parts.next().is_some() {
        return Err(ParseIssue {
            line: line_no,
            reason: "more than two headword fields before '['".to_string(),
        });
    }

    if !rest.starts_with('/') || !rest.ends_with('/') || rest.len() < 2 {
        return Err(ParseIssue {
            line: line_no,
            reason: "gloss field is not wrapped in '/../'".to_string(),
        });
    }
    let glosses: Vec<String> = rest[1..rest.len() - 1]
        .split('/')
        .map(|g| g.trim().to_string())
        .filter(|g| !g.is_empty())
        .collect();
    if glosses.is_empty() {
        return Err(ParseIssue {
            line: line_no,
            reason: "gloss field has no non-empty definitions".to_string(),
        });
    }

    Ok(Some(CedictLine {
        traditional: traditional.to_string(),
        simplified: simplified.to_string(),
        pinyin,
        glosses,
    }))
}

/// `#! date=YYYY-MM-DD` — dòng header khai `source_version` của chính tệp (CC-CEDICT).
/// CVDICT không có dòng này; module đó truyền `source_version` từ tag/commit repo.
pub fn find_date_header(lines: &[String]) -> Option<String> {
    for l in lines.iter().take(64) {
        if let Some(rest) = l.strip_prefix("#! date=") {
            return Some(rest.trim().to_string());
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_a_normal_line() {
        let l = parse_line(1, "中國 中国 [Zhong1 guo2] /China/Middle Kingdom/")
            .unwrap()
            .unwrap();
        assert_eq!(l.traditional, "中國");
        assert_eq!(l.simplified, "中国");
        assert_eq!(l.pinyin, "Zhong1 guo2");
        assert_eq!(l.glosses, vec!["China", "Middle Kingdom"]);
    }

    #[test]
    fn comment_and_blank_lines_are_none_not_err() {
        assert!(parse_line(1, "# a comment").unwrap().is_none());
        assert!(parse_line(2, "").unwrap().is_none());
        assert!(parse_line(3, "   ").unwrap().is_none());
    }

    #[test]
    fn missing_brackets_is_an_error_not_a_panic() {
        assert!(parse_line(1, "中國 中国 Zhong1 guo2 /China/").is_err());
    }

    #[test]
    fn unwrapped_gloss_field_is_an_error() {
        assert!(parse_line(1, "中國 中国 [Zhong1 guo2] China").is_err());
    }

    #[test]
    fn date_header_is_found_within_first_lines() {
        let lines: Vec<String> = vec![
            "# CC-CEDICT".to_string(),
            "#! date=2026-01-15".to_string(),
        ];
        assert_eq!(find_date_header(&lines).as_deref(), Some("2026-01-15"));
    }

    #[test]
    fn missing_date_header_is_none() {
        let lines: Vec<String> = vec!["# CC-CEDICT".to_string()];
        assert_eq!(find_date_header(&lines), None);
    }
}
