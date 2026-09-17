//! Định dạng tệp `.prompt.md` — Story 4.5 (FR79, NFR9). Module THUẦN: vào `&str`, ra
//! `String`, không chạm hệ thống tệp (xem `exchange_io.rs` cho nửa CÓ chạm đĩa).
//!
//! ─────────────────────────────────────────────────────────────────────────────
//! 🔴 KHUÔN CHÉP TỪ `core::glossary::exchange`, KHÔNG TÁI DÙNG MODULE ĐÓ
//! ─────────────────────────────────────────────────────────────────────────────
//! `glossary_boundary.rs::GLOSSARY_ONLY_SURFACE` fences bề mặt của module kia — copy hình
//! dạng (parse thu mọi lỗi, `classify` thuần, `ConflictDecision`) là điều Design Notes của
//! spec 4.5 yêu cầu, không phải một sơ suất trùng tên.
//!
//! ─────────────────────────────────────────────────────────────────────────────
//! ĐỊNH DẠNG — Quyết định #1 (Ice ký 2026-09-17)
//! ─────────────────────────────────────────────────────────────────────────────
//! Đúng BA dòng đầu cố định, rồi thân nguyên văn:
//! ```text
//! ---
//! name: <tên bộ>
//! ---
//! <thân, byte-for-byte, tới hết tệp>
//! ```
//! `name:` là định danh MÁY ĐỌC (khuôn `COLUMNS` của Glossary — tiếng Anh, không dấu), không
//! phải nhãn hiển thị. Mockup vẽ thêm `cặp ngôn ngữ` và một danh sách `biến:` — cả hai bị
//! Quyết định #1 loại bỏ VĨNH VIỄN (một bộ prompt là TÊN + THÂN, không gì khác).
//!
//! 🔴 **HỢP ĐỒNG ĐÚNG (sửa sau kiểm chứng — bản trước ghi sai)**: ba dòng đầu là VỊ TRÍ CỐ
//! ĐỊNH, không phải "tìm `name:` ở đâu đó trước dòng `---` thứ hai". Dòng 1 phải đúng
//! `---`; dòng 2 phải bắt đầu bằng `name:` chữ thường, KHÔNG khoảng trắng đứng trước (`
//! name:`/`Name:` đều bị từ chối — [`parse`] dùng `str::strip_prefix("name:")`, không
//! `trim()` trước); dòng 3 phải đúng `---`. Một tệp hand-edited chèn thêm dòng nào giữa ba
//! dòng đó (kể cả đúng dòng mockup vẽ, `cặp ngôn ngữ`) đẩy `name:`/`---` ra khỏi vị trí cố
//! định của chúng và bị TỪ CHỐI — đo được: `"---\nname: X\ncap ngon ngu: zh-vi\n---\nthan"`
//! trả `Err([MissingClosingDelimiter])` (dòng 3 thật sự là `cap ngon ngu: …`, không phải
//! `---`). Đây đúng ý Quyết định #1 ("một bộ prompt là TÊN + THÂN, không gì khác") — không
//! phải một khoan dung ngoài ý đó.
//!
//! Không mã hoá, không nén, không khoá (§Never) — `render`/`parse` là hai hàm đối xứng, không
//! gì khác.

use std::collections::BTreeMap;

use crate::core::i18n::{IpcError, MessageKey};

use super::PromptSet;

/// Dấu phân cách frontmatter — luôn đúng ba ký tự `---` trên một dòng riêng.
const DELIMITER: &str = "---";

/// Dấu thứ tự byte UTF-8 (`EF BB BF`) — khuôn chép `core::glossary::exchange::strip_bom`.
fn strip_bom(raw: &str) -> &str {
    raw.strip_prefix('\u{feff}').unwrap_or(raw)
}

/// Render MỘT bộ thành nội dung tệp `.prompt.md` — `name`/`body` KHÔNG bị sửa (không trim,
/// không escape): [`super::store::validate_name`] đã đảm bảo `name` không mang xuống dòng
/// trên đường ghi vào kho, và `body` đi ra nguyên văn để [`parse`] đọc lại byte-for-byte.
pub fn render(name: &str, body: &str) -> String {
    format!("{DELIMITER}\nname: {name}\n{DELIMITER}\n{body}")
}

/// Mọi cách một tệp nhập bị từ chối TRỌN — mỗi biến thể mang số dòng (1-based). Ba dòng đầu
/// cố định nên CẢ BA đều được kiểm ĐỘC LẬP (không dừng ở lỗi đầu), khớp §Always "parsing
/// collects every issue, not first-fail".
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParseIssue {
    /// Dòng 1 không đúng `---`.
    MissingOpeningDelimiter,
    /// Dòng 2 không bắt đầu bằng `name:`.
    MissingNameField,
    /// Dòng 2 mang `name:` nhưng giá trị rỗng/toàn khoảng trắng sau khi cắt.
    BlankName,
    /// Dòng 3 không đúng `---`.
    MissingClosingDelimiter,
}

impl ParseIssue {
    /// Số dòng (1-based, luôn `"1"`/`"2"`/`"3"`) mà issue này chỉ ra — dùng để gộp NHIỀU
    /// issue thành MỘT câu khi một tệp hỏng ở hơn một trong ba dòng đầu (AC4 spec 4.5: "every
    /// problem is reported with its line number"; xem
    /// `commands::promptset::issues_to_ipc_error`).
    pub fn line(&self) -> &'static str {
        match self {
            ParseIssue::MissingOpeningDelimiter => "1",
            ParseIssue::MissingNameField | ParseIssue::BlankName => "2",
            ParseIssue::MissingClosingDelimiter => "3",
        }
    }
}

impl std::fmt::Display for ParseIssue {
    /// ⚠️ KHÔNG DẤU — chẩn đoán cho log, không phải văn bản hiển thị (NFR16).
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParseIssue::MissingOpeningDelimiter => {
                write!(f, "prompt_set import[line 1]: expected '---'")
            }
            ParseIssue::MissingNameField => {
                write!(f, "prompt_set import[line 2]: expected 'name: <value>'")
            }
            ParseIssue::BlankName => {
                write!(f, "prompt_set import[line 2]: name is blank")
            }
            ParseIssue::MissingClosingDelimiter => {
                write!(f, "prompt_set import[line 3]: expected '---'")
            }
        }
    }
}

impl std::error::Error for ParseIssue {}

/// 🔴 Đi qua [`IpcError::new`], không dựng struct literal — cùng luật mọi chuyển đổi lỗi
/// khác của dự án.
impl From<ParseIssue> for IpcError {
    fn from(issue: ParseIssue) -> Self {
        match issue {
            ParseIssue::MissingOpeningDelimiter => {
                let mut params = BTreeMap::new();
                params.insert("line".to_owned(), "1".to_owned());
                IpcError::new(
                    "prompt_set.import_missing_opening_delimiter",
                    MessageKey::PromptSetImportMissingOpeningDelimiter,
                    params,
                    false,
                )
            }
            ParseIssue::MissingNameField => {
                let mut params = BTreeMap::new();
                params.insert("line".to_owned(), "2".to_owned());
                IpcError::new(
                    "prompt_set.import_missing_name_field",
                    MessageKey::PromptSetImportMissingNameField,
                    params,
                    false,
                )
            }
            ParseIssue::BlankName => {
                let mut params = BTreeMap::new();
                params.insert("line".to_owned(), "2".to_owned());
                IpcError::new(
                    "prompt_set.import_blank_name",
                    MessageKey::PromptSetImportBlankName,
                    params,
                    false,
                )
            }
            ParseIssue::MissingClosingDelimiter => {
                let mut params = BTreeMap::new();
                params.insert("line".to_owned(), "3".to_owned());
                IpcError::new(
                    "prompt_set.import_missing_closing_delimiter",
                    MessageKey::PromptSetImportMissingClosingDelimiter,
                    params,
                    false,
                )
            }
        }
    }
}

/// Kết quả THÀNH CÔNG của [`parse`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedPromptSet {
    pub name: String,
    /// Byte-for-byte, kể cả mọi `{{...}}` — không bị sửa (AC "round-trip").
    pub body: String,
}

/// Tách dòng ĐẦU TIÊN của `text` (không mang dấu xuống dòng) và phần CÒN LẠI SAU dấu đó —
/// nhận cả `\n` và `\r\n`. Không có dấu xuống dòng nào ⇒ `(text, "")`.
fn split_first_line(text: &str) -> (&str, &str) {
    match text.find('\n') {
        Some(i) => {
            let line = &text[..i];
            let line = line.strip_suffix('\r').unwrap_or(line);
            (line, &text[i + 1..])
        }
        None => (text, ""),
    }
}

/// Phân tích TRỌN văn bản một tệp `.prompt.md` — hàm THUẦN, không chạm Store. Ba dòng đầu
/// đều được kiểm ĐỘC LẬP; `body` là mọi thứ SAU dòng thứ ba, nguyên văn (không trim, giữ mọi
/// dấu xuống dòng của chính nó).
pub fn parse(text: &str) -> Result<ParsedPromptSet, Vec<ParseIssue>> {
    let text = strip_bom(text);

    let (line1, rest) = split_first_line(text);
    let (line2, rest) = split_first_line(rest);
    let (line3, body) = split_first_line(rest);

    let mut issues: Vec<ParseIssue> = Vec::new();

    if line1 != DELIMITER {
        issues.push(ParseIssue::MissingOpeningDelimiter);
    }

    let name = match line2.strip_prefix("name:") {
        Some(raw) => {
            let trimmed = raw.trim();
            if trimmed.is_empty() {
                issues.push(ParseIssue::BlankName);
                None
            } else {
                Some(trimmed.to_owned())
            }
        }
        None => {
            issues.push(ParseIssue::MissingNameField);
            None
        }
    };

    if line3 != DELIMITER {
        issues.push(ParseIssue::MissingClosingDelimiter);
    }

    if !issues.is_empty() {
        return Err(issues);
    }

    Ok(ParsedPromptSet { name: name.expect("khong con issue nao thi name da Some"), body: body.to_owned() })
}

/// Quyết định của người dùng cho một va chạm tên — mặc định [`ConflictDecision::KeepMine`]
/// (§Always: "A name collision is surfaced and decided, never resolved silently").
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Deserialize)]
pub enum ConflictDecision {
    /// Giữ bộ đang có trong kho — KHÔNG ghi gì. Mặc định.
    #[serde(rename = "keep_mine")]
    KeepMine,
    /// Lấy thân từ tệp — `UPDATE` thân của hàng đang có.
    #[serde(rename = "take_theirs")]
    TakeTheirs,
}

/// Phân loại một bộ (`name`, `body`) đã phân tích so với tầng ĐÍCH — hàm THUẦN, khuôn chép
/// `core::glossary::exchange::classify` rút gọn cho MỘT hàng (một tệp = một bộ).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PlanKind {
    /// `name` chưa có ở tầng đích.
    New,
    /// Cùng `name`, CÙNG `body` — không đề nghị gì, không ghi.
    Identical,
    /// Cùng `name`, KHÁC `body` — mang cả hai thân để người dùng quyết.
    Conflict {
        /// `id` của hàng đang có ở tầng đích.
        existing_id: i64,
        /// Thân ĐANG CÓ trong kho, trước khi có quyết định nào — chụp lại NGAY LÚC XEM
        /// TRƯỚC để [`super::store::import_into_tier`] so lạc quan ở nhịp xác nhận (cùng
        /// khuôn `RowPlanKind::Conflict::existing_translation` của Glossary).
        existing_body: String,
    },
}

/// Phân loại `(name, body)` so với `existing` (tầng ĐÍCH, đã nạp bằng
/// [`super::store::load_prompt_set_tier`]).
pub fn classify(name: &str, body: &str, existing: &BTreeMap<String, PromptSet>) -> PlanKind {
    match existing.get(name) {
        None => PlanKind::New,
        Some(row) if row.body == body => PlanKind::Identical,
        Some(row) => PlanKind::Conflict { existing_id: row.id, existing_body: row.body.clone() },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn render_then_parse_round_trips_name_and_body_byte_for_byte_including_markers() {
        let body = "Dich: {{source_segment}}\nThuat ngu: {{glossary_terms}}\nLa: {{glosary_terms}}\n";
        let file = render("Tien hiep", body);
        let parsed = parse(&file).expect("tep dung dinh dang phai phan tich duoc");
        assert_eq!(parsed.name, "Tien hiep");
        assert_eq!(parsed.body, body);
    }

    #[test]
    fn an_empty_body_round_trips_to_an_empty_string() {
        let file = render("X", "");
        let parsed = parse(&file).unwrap();
        assert_eq!(parsed.body, "");
    }

    #[test]
    fn a_hand_edited_body_imports_with_the_edit_intact() {
        let file = "---\nname: X\n---\ndong 1\ndong 2\r\ndong 3 khong xuong dong cuoi";
        let parsed = parse(file).unwrap();
        assert_eq!(parsed.body, "dong 1\ndong 2\r\ndong 3 khong xuong dong cuoi");
    }

    #[test]
    fn missing_opening_delimiter_is_reported_with_line_one() {
        let file = "khong phai ---\nname: X\n---\nthan";
        let err = parse(file).unwrap_err();
        assert_eq!(err, vec![ParseIssue::MissingOpeningDelimiter]);
    }

    #[test]
    fn missing_name_field_is_reported_with_line_two() {
        let file = "---\nkhong phai name\n---\nthan";
        let err = parse(file).unwrap_err();
        assert_eq!(err, vec![ParseIssue::MissingNameField]);
    }

    #[test]
    fn a_blank_name_after_trim_is_reported() {
        let file = "---\nname:    \n---\nthan";
        let err = parse(file).unwrap_err();
        assert_eq!(err, vec![ParseIssue::BlankName]);
    }

    #[test]
    fn missing_closing_delimiter_is_reported_with_line_three() {
        let file = "---\nname: X\nkhong phai ---\nthan";
        let err = parse(file).unwrap_err();
        assert_eq!(err, vec![ParseIssue::MissingClosingDelimiter]);
    }

    #[test]
    fn line_reports_the_fixed_line_number_of_each_issue_variant() {
        assert_eq!(ParseIssue::MissingOpeningDelimiter.line(), "1");
        assert_eq!(ParseIssue::MissingNameField.line(), "2");
        assert_eq!(ParseIssue::BlankName.line(), "2");
        assert_eq!(ParseIssue::MissingClosingDelimiter.line(), "3");
    }

    #[test]
    fn every_broken_line_is_collected_at_once_not_first_fail() {
        let file = "sai\nsai\nsai\nthan";
        let err = parse(file).unwrap_err();
        assert_eq!(
            err,
            vec![
                ParseIssue::MissingOpeningDelimiter,
                ParseIssue::MissingNameField,
                ParseIssue::MissingClosingDelimiter,
            ]
        );
    }

    #[test]
    fn a_file_with_fewer_than_three_lines_reports_every_missing_line_instead_of_panicking() {
        let err = parse("---").unwrap_err();
        assert_eq!(err, vec![ParseIssue::MissingNameField, ParseIssue::MissingClosingDelimiter]);
    }

    #[test]
    fn classify_reports_new_when_the_name_is_absent_from_the_target_tier() {
        let existing = BTreeMap::new();
        assert_eq!(classify("X", "body", &existing), PlanKind::New);
    }

    #[test]
    fn classify_reports_identical_when_name_and_body_both_match() {
        let mut existing = BTreeMap::new();
        existing.insert("X".to_owned(), PromptSet { id: 1, name: "X".to_owned(), body: "body".to_owned() });
        assert_eq!(classify("X", "body", &existing), PlanKind::Identical);
    }

    #[test]
    fn classify_reports_conflict_with_the_existing_id_and_body_when_only_the_body_differs() {
        let mut existing = BTreeMap::new();
        existing.insert("X".to_owned(), PromptSet { id: 7, name: "X".to_owned(), body: "old".to_owned() });
        assert_eq!(
            classify("X", "new", &existing),
            PlanKind::Conflict { existing_id: 7, existing_body: "old".to_owned() }
        );
    }
}
