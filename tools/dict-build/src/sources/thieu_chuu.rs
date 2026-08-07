//! Thiều Chửu — Hán Việt Tự Điển. TSV 3 cột: chữ Hán · âm Hán Việt (nhiều âm tách `|`) ·
//! nghĩa (nhiều nghĩa tách `<br>` + số thứ tự `1.` `2.` …). Lớp GỠ RỜI, Story 1.10.
//! `source_version` là hằng cố định — kho gốc `catusf/tudien@2.2` không có header ngày
//! trong nội dung để đo lúc chạy (khác CC-CEDICT).

use std::io::BufRead;

use crate::char_idx::is_han;
use crate::model::{ParseIssue, RawCitation, RawEntry, RawSense};

pub const SOURCE_CODE: &str = "thieu-chuu";
pub const SOURCE_VERSION: &str = "catusf/tudien@2.2 (2022-10-10)";

const BOM: char = '\u{feff}';

/// Chữ ký giống sáu module còn lại: `parse(reader) -> impl Iterator<Item = Result<RawEntry, ParseIssue>>`.
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
        // Đối xứng với `vietphrase.rs`: `trim()` KHÔNG bỏ U+FEFF (ký tự `Cf`, không phải
        // whitespace), nên một lượt `iconv` để lại BOM sẽ tạo headword "\u{feff}一" —
        // không rỗng, không lỗi, một đầu mục vĩnh viễn không tra ra được.
        if first_line {
            first_line = false;
            if let Some(stripped) = raw.strip_prefix(BOM) {
                raw = stripped.to_string();
            }
        }
        if raw.trim().is_empty() {
            return None;
        }

        // 🔴 Dòng 108 hỏng thật ('亯') chỉ có 2 cột (thẻ HTML rơi rớt '</h4>') — ParseIssue,
        // không panic!. Đây là ca thật trong dữ liệu, không phải giả thuyết.
        let cols: Vec<&str> = raw.split('\t').collect();
        if cols.len() != 3 {
            return Some(Err(ParseIssue {
                line: line_no,
                reason: format!("expected 3 tab-separated columns, got {}", cols.len()),
            }));
        }

        let headword = cols[0].trim();
        let han_viet_raw = cols[1].trim();
        let senses_raw = cols[2].trim();

        if headword.is_empty() {
            return Some(Err(ParseIssue {
                line: line_no,
                reason: "empty headword (column 1)".to_string(),
            }));
        }
        if senses_raw.is_empty() {
            return Some(Err(ParseIssue {
                line: line_no,
                reason: "empty senses field (column 3)".to_string(),
            }));
        }

        // Nhiều âm Hán Việt tách bằng '|' — GIỮ NGUYÊN chuỗi, không nhân bản entry
        // (âm đọc không phải nghĩa, không phải điều kiện tách dict_entry).
        let han_viet = if han_viet_raw.is_empty() {
            None
        } else {
            Some(han_viet_raw.to_string())
        };

        let senses = split_senses(senses_raw);
        if senses.is_empty() {
            return Some(Err(ParseIssue {
                line: line_no,
                reason: "no senses parsed from column 3".to_string(),
            }));
        }

        Some(Ok(RawEntry {
            lang: "zh".to_string(),
            headword: headword.to_string(),
            headword_simp: None,
            reading: None,
            han_viet,
            nom_reading: None,
            senses,
        }))
    })
}

/// Tách cột 3 thành nhiều nghĩa bằng `<br>` **VÀ** số thứ tự `1.` `2.` `3.` (Task 6) —
/// không nối thành một `gloss` (FR29 đòi mỗi nghĩa một hàng).
///
/// 🔴 Vì sao phải tách theo CẢ HAI: nguồn thật có dòng thiếu `<br>` giữa hai nghĩa —
/// `丐` = `"1. Xin. … ăn xin. 2. Cho. Như thiêm cái hậu nhân …"` (một mảnh, hai nghĩa) và
/// `下` nghĩa 3+4 dính nhau. Chỉ tách `<br>` thì hai nghĩa dồn vào một hàng, không lỗi
/// nào được ném và mọi thống kê vẫn xanh — đúng lớp hỏng im lặng mà FR29 tồn tại để chặn.
/// 22/22.658 nghĩa trên dữ liệu thật rơi vào ca này.
///
/// 🔴 Luật cắt CHẶT: chỉ cắt ở `"N."` khi `N` đúng bằng **số thứ tự kế tiếp đang chờ**.
/// Đây là điều kiện làm phép tách an toàn — một con số bất kỳ trong văn bản (`"1942."`,
/// `"250 giới luật"`) không thể trở thành ranh giới nghĩa, vì nó gần như không bao giờ
/// đúng bằng số kế tiếp. Một mảnh không mang số thứ tự vẫn là một nghĩa hợp lệ.
///
/// `ord` do `insert::insert_entry` gán theo VỊ TRÍ (0-based), giống cả sáu nguồn còn lại.
/// Sau lượt tách này, vị trí bám sát số thứ tự của nguồn (`ord == N - 1`) — cùng một
/// thứ tự, không phải hai quy ước `ord` khác nhau trong cùng một lược đồ.
fn split_senses(raw: &str) -> Vec<RawSense> {
    let mut pieces: Vec<String> = Vec::new();
    let mut expected: u32 = 1;
    for fragment in raw.split("<br>") {
        for piece in split_at_sequential_ordinals(fragment.trim(), &mut expected) {
            if !piece.is_empty() {
                pieces.push(piece);
            }
        }
    }

    pieces
        .into_iter()
        .map(|text| {
            let citations: Vec<RawCitation> = extract_citation(&text).into_iter().collect();
            RawSense {
                pos: None,
                pos_lang: None,
                gloss: text,
                note: None,
                examples: Vec::new(),
                citations,
            }
        })
        .collect()
}

/// Cắt MỘT mảnh tại mọi vị trí `"N."` mà `N == *expected`, bỏ luôn tiền tố số đó khỏi
/// `gloss`. Trả về các đoạn đã trim. `expected` tiến lên sau mỗi lần cắt, nên thứ tự
/// `1. 2. 3.` được tôn trọng còn một con số lạc lõng thì không.
fn split_at_sequential_ordinals(fragment: &str, expected: &mut u32) -> Vec<String> {
    let mut out = Vec::new();
    let mut cut_start = 0usize;
    let bytes = fragment.as_bytes();
    let mut i = 0usize;

    while i < fragment.len() {
        if !fragment.is_char_boundary(i) || !bytes[i].is_ascii_digit() {
            i += 1;
            continue;
        }
        // Số thứ tự phải đứng đầu mảnh hoặc ngay sau khoảng trắng — không nhận
        // chữ số nằm giữa một từ/số dài hơn.
        let at_boundary = i == 0 || bytes[i - 1].is_ascii_whitespace();
        if !at_boundary {
            i += 1;
            continue;
        }
        let mut j = i;
        while j < fragment.len() && bytes[j].is_ascii_digit() {
            j += 1;
        }
        // Phải là `"N."` và theo sau là khoảng trắng hoặc hết mảnh.
        let is_ordinal_marker = j < fragment.len()
            && bytes[j] == b'.'
            && (j + 1 == fragment.len() || bytes[j + 1].is_ascii_whitespace());
        if !is_ordinal_marker {
            i = j.max(i + 1);
            continue;
        }
        let n: u32 = match fragment[i..j].parse() {
            Ok(v) => v,
            Err(_) => {
                i = j;
                continue;
            }
        };
        if n != *expected {
            i = j;
            continue;
        }
        out.push(fragment[cut_start..i].trim().to_string());
        cut_start = j + 1;
        *expected += 1;
        i = j + 1;
    }

    out.push(fragment[cut_start..].trim().to_string());
    out
}

/// Tên trích dẫn — mẫu `"(Luận ngữ 論語)"` / `"(Nguyễn Du 阮攸)"`: ngoặc đơn chứa chữ
/// Latinh viết hoa đứng đầu CỘNG ít nhất một chữ Hán.
///
/// 🔴 Ghi vào cột **`work`**, **không** `author` (Ice chốt 2026-08-05, Review Findings).
/// Đo trên dữ liệu thật: 263 trích dẫn, 105 giá trị phân biệt, phổ biến nhất là
/// `Luận ngữ` (27) · `Thi Kinh` (17) · `Thư Kinh` (10) · `Tiêu dao du` (8) — **tên sách,
/// tên thiên, tên bài**, không phải người; chỉ ~10% là tên tác giả. Ghi tất cả vào
/// `author` là bịa đúng thứ Task 6 cấm và là cột hiện thẳng lên UI ở 1.11/1.13.
/// ⚠️ Đánh đổi đã biết và đã chấp nhận: một thiểu số (`Nguyễn Du`, `Mạnh Tử`) là tác giả
/// thật, giờ nằm ở cột `work`. Sai 10% ở cột ít nguy hiểm hơn, thay vì sai 90% ở cột
/// nguy hiểm hơn. `author` để `NULL` — không đoán.
///
/// `text` là NGUYÊN VĂN cả câu nghĩa (không cắt), không tự đoán ranh giới câu trích.
fn extract_citation(text: &str) -> Option<RawCitation> {
    let start = text.find('(')?;
    let rel_end = text[start..].find(')')?;
    let end = start + rel_end;
    let inner = text[start + 1..end].trim();
    if inner.is_empty() {
        return None;
    }
    let starts_upper = inner.chars().next()?.is_uppercase();
    let han_start = inner.find(is_han)?;
    if !starts_upper {
        return None;
    }
    let name = inner[..han_start].trim();
    if name.is_empty() {
        return None;
    }
    Some(RawCitation {
        text: text.to_string(),
        work: Some(name.to_string()),
        author: None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn parses_multi_reading_and_multi_sense_entry() {
        let text = "丁\tđinh|chênh\t1. Can Ðinh.<br>2. Ðang.<br>8. Một âm là chênh.<br>\n";
        let entries: Vec<_> = parse(Cursor::new(text.as_bytes()))
            .collect::<Result<_, _>>()
            .unwrap();
        assert_eq!(entries.len(), 1);
        let e = &entries[0];
        assert_eq!(e.headword, "丁");
        assert_eq!(e.han_viet.as_deref(), Some("đinh|chênh"));
        assert_eq!(e.senses.len(), 3);
        assert_eq!(e.senses[0].gloss, "Can Ðinh.");
        assert_eq!(e.senses[1].gloss, "Ðang.");
    }

    /// 🔴 Dòng 108 hỏng thật (chỉ 2 cột, thẻ HTML rơi rớt) — ParseIssue, không panic.
    #[test]
    fn line_with_only_two_columns_is_a_parse_issue_not_a_panic() {
        let text = "亯\thanh</h4> anh [heng1] Cũng như chữ hanh 亨.\n";
        let results: Vec<_> = parse(Cursor::new(text.as_bytes())).collect();
        assert_eq!(results.len(), 1);
        assert!(results[0].is_err());
    }

    /// 🔴 Tên trong ngoặc vào cột `work`, `author` để `NULL` — không bịa tác giả.
    /// Hai ca THẬT trong cùng một mục `关` của nguồn: `(Thăng Long 升龍)` là tên BÀI THƠ,
    /// còn tác giả thật (`Nguyễn Du`) nằm NGOÀI ngoặc. Ghi nó vào `author` là bịa.
    #[test]
    fn parenthesized_name_goes_to_work_not_author() {
        let text = "关\tquan\t1. Song nhãn trừng trừng 雙眼瞪瞪 (Nguyễn Du 阮攸) hai mắt trừng trừng.<br>\n";
        let entries: Vec<_> = parse(Cursor::new(text.as_bytes()))
            .collect::<Result<_, _>>()
            .unwrap();
        let sense = &entries[0].senses[0];
        assert_eq!(sense.citations.len(), 1);
        assert_eq!(sense.citations[0].work.as_deref(), Some("Nguyễn Du"));
        assert_eq!(sense.citations[0].author, None);
    }

    #[test]
    fn a_work_title_in_parentheses_is_never_recorded_as_an_author() {
        let text = "反\tphản\t1. Cử nhất ngung 舉一隅 (Luận ngữ 論語) cất một góc.<br>\n";
        let entries: Vec<_> = parse(Cursor::new(text.as_bytes()))
            .collect::<Result<_, _>>()
            .unwrap();
        let cit = &entries[0].senses[0].citations[0];
        assert_eq!(cit.work.as_deref(), Some("Luận ngữ"));
        assert_eq!(cit.author, None, "'Luận ngữ' là TÊN SÁCH, không phải tác giả");
    }

    /// 🔴 Ca THẬT `丐` (fixture dòng 11): hai nghĩa trong CÙNG một mảnh, không có
    /// `<br>` ngăn giữa. Chỉ tách `<br>` sẽ ra 1 hàng thay vì 2 — FR29 vỡ im lặng.
    #[test]
    fn two_senses_inside_one_br_fragment_are_split_by_ordinal_number() {
        let text = "丐\tcái\t1. Xin. Như khất cái 乞丐 người ăn mày, ăn xin. 2. Cho. Như thiêm cái hậu nhân 沾丐後人 để ơn lại cho người sau.<br>\n";
        let entries: Vec<_> = parse(Cursor::new(text.as_bytes()))
            .collect::<Result<_, _>>()
            .unwrap();
        let senses = &entries[0].senses;
        assert_eq!(senses.len(), 2, "'丐' phải ra ĐÚNG hai nghĩa");
        assert!(senses[0].gloss.starts_with("Xin."));
        assert!(senses[1].gloss.starts_with("Cho."));
    }

    /// 🔴 Ca THẬT `下` (fixture dòng 8): nghĩa 3 và 4 dính nhau trong mảnh cuối.
    #[test]
    fn ordinal_split_works_across_br_boundaries() {
        let text = "下\thạ|há\t1. Dưới.<br>2. Bề dưới.<br>3. Một âm là há. Như há sơn 下山 xuống núi. 4. Cuốn. Như há kì 下旗 cuốn cờ.<br>\n";
        let entries: Vec<_> = parse(Cursor::new(text.as_bytes()))
            .collect::<Result<_, _>>()
            .unwrap();
        let senses = &entries[0].senses;
        assert_eq!(senses.len(), 4, "'下' phải ra ĐÚNG bốn nghĩa");
        assert!(senses[3].gloss.starts_with("Cuốn."));
    }

    /// Luật cắt CHẶT: chỉ cắt ở số thứ tự KẾ TIẾP đang chờ. Một con số lạc trong văn bản
    /// (năm, số lượng) không được biến thành ranh giới nghĩa.
    #[test]
    fn a_stray_number_in_the_text_is_not_a_sense_boundary() {
        let text = "丁\tđinh\t1. Ta 18 tuổi phải đóng sưu. Xuất bản 1942. Vẫn là một nghĩa.<br>\n";
        let entries: Vec<_> = parse(Cursor::new(text.as_bytes()))
            .collect::<Result<_, _>>()
            .unwrap();
        assert_eq!(entries[0].senses.len(), 1);
        assert!(entries[0].senses[0].gloss.contains("1942."));
    }

    /// Một mảnh không mang số thứ tự vẫn là một nghĩa hợp lệ, không phải lỗi.
    #[test]
    fn a_fragment_without_an_ordinal_is_still_one_valid_sense() {
        let text = "与\tdữ\tTục dùng như chữ 與.<br>\n";
        let entries: Vec<_> = parse(Cursor::new(text.as_bytes()))
            .collect::<Result<_, _>>()
            .unwrap();
        assert_eq!(entries[0].senses.len(), 1);
        assert_eq!(entries[0].senses[0].gloss, "Tục dùng như chữ 與.");
    }

    /// Đối xứng với `vietphrase.rs`: `trim()` KHÔNG bỏ U+FEFF — BOM lọt vào sẽ thành
    /// một đầu mục vĩnh viễn không tra ra được, không phải một lỗi.
    #[test]
    fn leading_bom_on_first_line_is_stripped() {
        let text = "\u{feff}一\tnhất\t1. Một.<br>\n丁\tđinh\t1. Can Ðinh.<br>\n";
        let entries: Vec<_> = parse(Cursor::new(text.as_bytes()))
            .collect::<Result<_, _>>()
            .unwrap();
        assert_eq!(entries[0].headword, "一");
        assert_eq!(entries[1].headword, "丁");
    }

    /// `SOURCE_CODE` không được trôi khỏi `sources_meta::THIEU_CHUU.code` — tên tệp
    /// đầu ra, `dict_source.code` và `[[detachable]].name` đều là CÙNG chuỗi đó.
    #[test]
    fn source_code_matches_its_source_meta() {
        assert_eq!(SOURCE_CODE, crate::sources_meta::THIEU_CHUU.code);
    }

    #[test]
    fn blank_lines_are_skipped_not_errors() {
        let text = "\n\n一\tnhất\t1. Một.<br>\n";
        let results: Vec<_> = parse(Cursor::new(text.as_bytes())).collect();
        assert_eq!(results.len(), 1);
        assert!(results[0].is_ok());
    }
}
