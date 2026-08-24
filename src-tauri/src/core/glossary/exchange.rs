//! Định dạng CSV/TSV của Glossary — module THUẦN, Story 3.10, FR49/NFR9.
//!
//! ─────────────────────────────────────────────────────────────────────────────
//! 🔴 VÀO `&str`, RA `String` — MODULE NÀY KHÔNG CHẠM HỆ THỐNG TỆP
//! ─────────────────────────────────────────────────────────────────────────────
//! `render_tier`/`parse`/`classify` là ba hàm THUẦN — không hệ thống tệp, không
//! khung ứng dụng desktop nào. Đọc/ghi byte và lấy đường dẫn thuộc nửa "chọn tệp", đang chờ
//! một `AD` (`ad-brief-2026-08-24-hop-thoai-chon-tep.md`) — xem `store.rs::export_tier`/
//! `import_into_tier` cho nửa CÓ chạm kho (nhưng vẫn không chạm tệp).
//!
//! ⚠️ **Phép kiểm AC5 của spec 3.10 đọc CHÍNH TỆP NÀY bằng `grep`, và nó phải rỗng** —
//! xem §Verification. Vì vậy chính doc-comment này tránh đánh vần ba token bị cấm (mô-đun
//! chuẩn thư viện đọc tệp, kiểu đường dẫn sở hữu, và tiền tố khung desktop) nguyên văn: một
//! doc-comment NHẮC TỚI chúng để giải thích lý do cấm sẽ tự làm chính phép kiểm nó mô tả
//! báo dương tính giả.
//!
//! ─────────────────────────────────────────────────────────────────────────────
//! 🔴 KHÔNG `csv`, KHÔNG `serde_csv` — TẬP CON RFC 4180 TỰ VIẾT (NFR15)
//! ─────────────────────────────────────────────────────────────────────────────
//! Bọc/gỡ nháy kép áp CHO CẢ HAI dấu phân cách (CSV **và** TSV) — TSV theo lệ không bọc,
//! nhưng `note` là ô văn bản tự do và một ký tự Tab dán vào đó sẽ phá hàng trong im lặng.
//! Một đường bọc DÙNG CHUNG cho hai dấu phân cách là chỗ duy nhất bảo đảm vòng tròn
//! xuất→nhập khép kín — xem §Design Notes của spec 3.10.
//!
//! ⚠️ Mọi chuỗi trong `src-tauri/src/**` viết KHÔNG DẤU; doc-comment có dấu là hợp lệ.

use std::collections::BTreeMap;

use crate::core::i18n::{IpcError, MessageKey};

use super::entry::{Category, GlossaryEntry};

/// Dấu phân cách. `Delimiter::as_char()` là điểm DUY NHẤT quy ra ký tự thật — mọi hàm
/// khác trong module nhận `Delimiter`, không nhận `char` trần.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Delimiter {
    /// `,` — RFC 4180.
    Csv,
    /// `\t`.
    Tsv,
}

impl Delimiter {
    /// Ký tự thật trên tệp.
    pub const fn as_char(self) -> char {
        match self {
            Delimiter::Csv => ',',
            Delimiter::Tsv => '\t',
        }
    }
}

/// Sáu cột của tệp xuất — thứ tự CỐ ĐỊNH, dùng cho CẢ hàng tiêu đề lẫn việc dò cột lúc
/// nhập. Định danh máy đọc, KHÔNG DẤU (§Boundaries — Kiểm A của `check:i18n`, cùng luật đã
/// đặt cho `Category::as_str()`, AD-21/NFR16).
const COLUMNS: [&str; 6] =
    ["source_term", "translation", "note", "category", "term_origin", "created_at"];

/// Dấu thứ tự byte UTF-8 (`EF BB BF`) — cùng khuôn `core::segment::import::strip_bom`.
///
/// ⚠️ Bản CHÉP, không `use` hàm kia: nó là `fn` riêng tư của `core::segment::import`
/// (không `pub`), và module này cố ý không phụ thuộc `core::segment` cho một việc một
/// dòng — xem Code Map của spec 3.10 ("khuôn chép", không "tái dùng").
fn strip_bom(raw: &str) -> &str {
    raw.strip_prefix('\u{feff}').unwrap_or(raw)
}

// ═════════════════════════════════════════════════════════════════════════════════
// XUẤT — render_tier
// ═════════════════════════════════════════════════════════════════════════════════

/// Một ô cần bọc nháy kép khi nó chứa dấu phân cách, nháy kép, hoặc xuống dòng (`\n`/`\r`)
/// — RFC 4180, áp CHO CẢ HAI dấu phân cách (xem doc-comment đầu tệp).
fn field_needs_quoting(field: &str, delimiter: char) -> bool {
    field.contains(delimiter) || field.contains('"') || field.contains('\n') || field.contains('\r')
}

/// Bọc một ô: nháy kép nhân đôi bên trong, cả cụm bọc trong một cặp nháy kép — RFC 4180.
fn quote_field(field: &str) -> String {
    let mut out = String::with_capacity(field.len() + 2);
    out.push('"');
    for c in field.chars() {
        if c == '"' {
            out.push('"');
        }
        out.push(c);
    }
    out.push('"');
    out
}

/// Render một ô: bọc nếu cần, nguyên văn nếu không.
fn render_field(field: &str, delimiter: char) -> String {
    if field_needs_quoting(field, delimiter) {
        quote_field(field)
    } else {
        field.to_owned()
    }
}

/// Xuất một tầng thành một `String` — hàng tiêu đề + N hàng, **6 cột, không cột `id`**
/// (§Never: `id` chỉ duy nhất TRONG một kho, một `id` trong tệp của người khác là một con
/// số vô nghĩa và một cái bẫy va khoá).
///
/// Tầng rỗng ⇒ **chỉ** hàng tiêu đề — một tệp có tiêu đề nói *"rỗng"*, một tệp trống không
/// nói gì (I/O Matrix).
///
/// Dòng kết thúc bằng `\n` — [`parse`] nhận cả `\n` lẫn `\r\n` (I/O Matrix "`\r\n` và `\n`
/// lẫn lộn"), nên chọn `\n` ở chiều xuất không phá vòng tròn.
pub fn render_tier(tier: &BTreeMap<String, GlossaryEntry>, delimiter: Delimiter) -> String {
    let d = delimiter.as_char();
    let sep = d.to_string();

    let mut out = String::new();
    out.push_str(&COLUMNS.join(&sep));
    out.push('\n');

    for entry in tier.values() {
        let fields = [
            render_field(&entry.source_term, d),
            render_field(entry.translation.as_deref().unwrap_or(""), d),
            render_field(&entry.note, d),
            render_field(entry.category.as_str(), d),
            render_field(entry.term_origin.as_str(), d),
            render_field(&entry.created_at, d),
        ];
        out.push_str(&fields.join(&sep));
        out.push('\n');
    }

    out
}

// ═════════════════════════════════════════════════════════════════════════════════
// NHẬP — parse
// ═════════════════════════════════════════════════════════════════════════════════

/// Mọi cách một tệp nhập bị từ chối TRỌN (0 hàng nào được phân tích ra) — §I/O Matrix,
/// mỗi biến thể là đúng MỘT hàng của ma trận mang "0 lượt ghi" ở phần *phân tích*.
///
/// ⚠️ Khác `GlossaryError::ImportUniqueConflict` (`store.rs`) — biến thể đó là lỗi lúc GHI,
/// biến thể ở đây là lỗi lúc PHÂN TÍCH, trước khi có Store nào được chạm tới.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParseIssue {
    /// Hàng tiêu đề chứa CẢ hai dấu phân cách hoặc KHÔNG cái nào — không đoán được nên chọn
    /// cái nào.
    DelimiterUnresolved,
    /// Thiếu cột bắt buộc `source_term` ở hàng tiêu đề.
    MissingColumn {
        /// Tên cột thiếu — dữ liệu, không phải câu.
        column: &'static str,
    },
    /// Số ô của một hàng dữ liệu không khớp số cột của hàng tiêu đề.
    CellCountMismatch {
        /// Số dòng (1-based, tính cả hàng tiêu đề).
        line: usize,
        /// Số cột mà hàng tiêu đề khai.
        expected: usize,
        /// Số ô đếm được ở hàng này.
        found: usize,
    },
    /// `category` không khớp một trong bốn giá trị đã biết.
    UnknownCategory {
        /// Số dòng.
        line: usize,
        /// Giá trị đọc được — dữ liệu, không phải câu.
        value: String,
    },
    /// `source_term` rỗng hoặc chỉ toàn khoảng trắng sau khi cắt — cùng mệnh đề `CHECK` của
    /// `GLOSSARY_ENTRY_DDL`, bắt ở Rust TRƯỚC khi SQL bắt.
    BlankSourceTerm {
        /// Số dòng.
        line: usize,
    },
    /// Cùng `source_term` xuất hiện ở hai hàng dữ liệu trong CHÍNH tệp — không "dòng sau
    /// thắng" im lặng.
    DuplicateSourceTerm {
        /// Số dòng của lượt xuất hiện ĐẦU TIÊN.
        first_line: usize,
        /// Số dòng của lượt xuất hiện THỨ HAI.
        second_line: usize,
    },
    /// 🔵 **THÊM 2026-08-25 (vòng rà ba lớp, P3).** Cột `created_at` có mặt, không rỗng,
    /// nhưng KHÔNG khớp hình dạng ISO-8601 UTC mà `strftime('%Y-%m-%dT%H:%M:%fZ', 'now')`
    /// sinh ra. Cột này chảy thẳng vào một cột `created_at TEXT NOT NULL` mà mọi đường ghi
    /// khác đều đặt bằng `strftime` — một giá trị tự do (`hom qua`) đi lọt vào đó là một cột
    /// "ISO-8601 UTC" nói dối kể từ mục đó trở đi.
    InvalidCreatedAt {
        /// Số dòng.
        line: usize,
        /// Giá trị đọc được — dữ liệu, không phải câu.
        value: String,
    },
}

impl std::fmt::Display for ParseIssue {
    /// ⚠️ KHÔNG DẤU — chẩn đoán cho log (NFR16).
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParseIssue::DelimiterUnresolved => {
                write!(f, "glossary import: header has both or neither of ',' and TAB")
            }
            ParseIssue::MissingColumn { column } => {
                write!(f, "glossary import: header missing required column {column:?}")
            }
            ParseIssue::CellCountMismatch { line, expected, found } => {
                write!(f, "glossary import[line {line}]: {found} cells, header has {expected}")
            }
            ParseIssue::UnknownCategory { line, value } => {
                write!(f, "glossary import[line {line}]: unknown category {value:?}")
            }
            ParseIssue::BlankSourceTerm { line } => {
                write!(f, "glossary import[line {line}]: source_term is blank")
            }
            ParseIssue::DuplicateSourceTerm { first_line, second_line } => {
                write!(
                    f,
                    "glossary import: source_term duplicated at lines {first_line} and {second_line}"
                )
            }
            ParseIssue::InvalidCreatedAt { line, value } => {
                write!(f, "glossary import[line {line}]: created_at not iso-8601 utc {value:?}")
            }
        }
    }
}

impl std::error::Error for ParseIssue {}

/// 🔴 Đi qua [`IpcError::new`], không dựng struct literal — cùng luật với mọi chuyển đổi
/// lỗi khác của dự án.
impl From<ParseIssue> for IpcError {
    fn from(issue: ParseIssue) -> Self {
        match issue {
            ParseIssue::DelimiterUnresolved => IpcError::new(
                "glossary.import_delimiter_unresolved",
                MessageKey::GlossaryImportDelimiterUnresolved,
                BTreeMap::new(),
                false,
            ),
            ParseIssue::MissingColumn { column } => {
                let mut params = BTreeMap::new();
                params.insert("column".to_owned(), column.to_owned());
                IpcError::new(
                    "glossary.import_missing_column",
                    MessageKey::GlossaryImportMissingColumn,
                    params,
                    false,
                )
            }
            ParseIssue::CellCountMismatch { line, expected, found } => {
                let mut params = BTreeMap::new();
                params.insert("line".to_owned(), line.to_string());
                params.insert("expected".to_owned(), expected.to_string());
                params.insert("found".to_owned(), found.to_string());
                IpcError::new(
                    "glossary.import_cell_count_mismatch",
                    MessageKey::GlossaryImportCellCountMismatch,
                    params,
                    false,
                )
            }
            ParseIssue::UnknownCategory { line, value } => {
                let mut params = BTreeMap::new();
                params.insert("line".to_owned(), line.to_string());
                params.insert("value".to_owned(), value);
                IpcError::new(
                    "glossary.import_unknown_category",
                    MessageKey::GlossaryImportUnknownCategory,
                    params,
                    false,
                )
            }
            ParseIssue::BlankSourceTerm { line } => {
                let mut params = BTreeMap::new();
                params.insert("line".to_owned(), line.to_string());
                IpcError::new(
                    "glossary.import_blank_source_term",
                    MessageKey::GlossaryImportBlankSourceTerm,
                    params,
                    false,
                )
            }
            ParseIssue::DuplicateSourceTerm { first_line, second_line } => {
                let mut params = BTreeMap::new();
                params.insert("first_line".to_owned(), first_line.to_string());
                params.insert("second_line".to_owned(), second_line.to_string());
                IpcError::new(
                    "glossary.import_duplicate_source_term",
                    MessageKey::GlossaryImportDuplicateSourceTerm,
                    params,
                    false,
                )
            }
            ParseIssue::InvalidCreatedAt { line, value } => {
                let mut params = BTreeMap::new();
                params.insert("line".to_owned(), line.to_string());
                params.insert("value".to_owned(), value);
                IpcError::new(
                    "glossary.import_invalid_created_at",
                    MessageKey::GlossaryImportInvalidCreatedAt,
                    params,
                    false,
                )
            }
        }
    }
}

/// Một hàng dữ liệu đã phân tích thành công.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImportRow {
    /// Số dòng nguồn (1-based, tính cả hàng tiêu đề) — cho chẩn đoán và cho [`ParseIssue`].
    pub line: usize,
    /// Đã cắt khoảng trắng biên, đã xác nhận không rỗng.
    pub source_term: String,
    /// `None` == chờ chốt (ô rỗng/chỉ khoảng trắng, hoặc cột vắng mặt).
    pub translation: Option<String>,
    /// Đã cắt khoảng trắng biên. Vắng cột ⇒ `""`.
    pub note: String,
    /// Vắng cột, hoặc ô rỗng ⇒ [`Category::Other`] (không phải một lỗi — I/O Matrix chỉ coi
    /// một GIÁ TRỊ LẠ, ví dụ `"weapon"`, là lỗi).
    pub category: Category,
    /// `Some(giá trị trong tệp)` khi cột có mặt và không rỗng; `None` khi vắng cột hoặc ô
    /// rỗng — [`super::store::import_into_tier`] điền hôm nay cho nhánh `None`.
    pub created_at: Option<String>,
}

/// Kết quả THÀNH CÔNG của [`parse`].
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ParsedImport {
    /// Mọi hàng dữ liệu hợp lệ.
    pub rows: Vec<ImportRow>,
    /// Tên cột lạ ở hàng tiêu đề (không thuộc [`COLUMNS`]) — **bị bỏ qua, không phải lỗi**
    /// (I/O Matrix: "Bỏ qua cột đó và NÓI RA — không im lặng vứt"). Đây là chỗ "nói ra": chỗ
    /// gọi đọc danh sách này để hiển thị một ghi chú, không phải một placeholder rỗng.
    pub ignored_columns: Vec<String>,
}

/// `s` khớp hình dạng `YYYY-MM-DDTHH:MM:SS.mmmZ` — đúng thứ SQLite
/// `strftime('%Y-%m-%dT%H:%M:%fZ', 'now')` sinh ra ở MỌI đường ghi khác vào cột
/// `created_at`. Đây là kiểm HÌNH DẠNG (đúng chữ số ở đúng vị trí, đúng dấu phân cách), KHÔNG
/// phải kiểm LỊCH (không từ chối `2026-02-30`) — cột này đã là ISO-8601 UTC theo Consistency
/// Conventions của kho; kiểm hình dạng là đủ để bắt một giá trị tự do như `hom qua` mà không
/// phải kéo một crate ngày-giờ mới (NFR15).
fn looks_like_iso8601_utc(s: &str) -> bool {
    let b = s.as_bytes();
    if b.len() != 24 {
        return false;
    }
    let digits = |range: std::ops::Range<usize>| b[range].iter().all(u8::is_ascii_digit);
    digits(0..4)
        && b[4] == b'-'
        && digits(5..7)
        && b[7] == b'-'
        && digits(8..10)
        && b[10] == b'T'
        && digits(11..13)
        && b[13] == b':'
        && digits(14..16)
        && b[16] == b':'
        && digits(17..19)
        && b[19] == b'.'
        && digits(20..23)
        && b[23] == b'Z'
}

/// Tìm ranh giới DÒNG LOGIC đầu tiên trong `text` — dừng ở `\n`/`\r\n` KHÔNG nằm trong một
/// ô đang bọc nháy kép. Trả `(dòng, phần còn lại SAU dấu xuống dòng)`.
///
/// ⚠️ Chưa biết dấu phân cách ở bước này — và không cần biết: việc dò RANH GIỚI dòng chỉ
/// phụ thuộc trạng thái "đang trong nháy kép hay không", không phụ thuộc dấu phân cách.
/// Đúng vì sao [`parse`] gọi hàm này TRƯỚC khi chọn `Delimiter`.
///
/// Một cặp nháy kép nhân đôi (`""`, cách thoát một nháy kép RFC 4180 bên trong ô) đảo
/// trạng thái HAI LẦN — vào rồi ra ngay — nên phép "đảo cờ mỗi lần gặp `"`" vẫn đúng cho
/// mục đích dò ranh giới, dù nó không tách được TỪNG Ô (đó là việc của [`split_fields`]).
fn split_first_logical_line(text: &str) -> (&str, &str) {
    let mut in_quotes = false;
    let mut chars = text.char_indices().peekable();

    while let Some((i, c)) = chars.next() {
        match c {
            '"' => in_quotes = !in_quotes,
            '\n' if !in_quotes => return (&text[..i], &text[i + 1..]),
            '\r' if !in_quotes => {
                if let Some(&(j, '\n')) = chars.peek() {
                    return (&text[..i], &text[j + 1..]);
                }
                return (&text[..i], &text[i + 1..]);
            }
            _ => {}
        }
    }

    (text, "")
}

/// Mọi dòng logic của `text`, kèm số dòng NGUỒN (1-based) mà nó BẮT ĐẦU — một ô bọc nháy
/// kép mang xuống dòng thật bên trong làm một "dòng logic" trải qua NHIỀU dòng nguồn; số
/// dòng báo lỗi là dòng nó bắt đầu, đúng thứ một người đọc file bằng trình soạn thảo sẽ đếm.
fn logical_lines(text: &str) -> Vec<(usize, &str)> {
    let mut out = Vec::new();
    let mut remaining = text;
    let mut line_no = 1usize;

    while !remaining.is_empty() {
        let (line, rest) = split_first_logical_line(remaining);
        out.push((line_no, line));
        line_no += line.matches('\n').count() + 1;
        remaining = rest;
    }

    out
}

/// Tách MỘT dòng logic thành các ô theo `delimiter` — nháy kép mở/đóng một ô (chỉ khi nó
/// đứng NGAY ĐẦU ô), nháy kép nhân đôi bên trong là một nháy kép thoát, dấu phân cách/xuống
/// dòng bên trong một ô đang bọc là NỘI DUNG chứ không phải ranh giới.
fn split_fields(line: &str, delimiter: char) -> Vec<String> {
    let mut fields = Vec::new();
    let mut field = String::new();
    let mut in_quotes = false;
    let mut at_field_start = true;
    let mut chars = line.chars().peekable();

    while let Some(c) = chars.next() {
        if in_quotes {
            if c == '"' {
                if chars.peek() == Some(&'"') {
                    field.push('"');
                    chars.next();
                } else {
                    in_quotes = false;
                }
            } else {
                field.push(c);
            }
            continue;
        }

        if c == '"' && at_field_start {
            in_quotes = true;
            at_field_start = false;
            continue;
        }

        if c == delimiter {
            fields.push(std::mem::take(&mut field));
            at_field_start = true;
            continue;
        }

        field.push(c);
        at_field_start = false;
    }

    fields.push(field);
    fields
}

/// Phân tích TRỌN văn bản một tệp CSV/TSV thành các hàng — hàm THUẦN, không chạm Store
/// (§Design Notes: phân tích trọn TRƯỚC khi mở giao dịch là chỗ trả về được MỘT danh sách
/// lỗi, không phải dừng ở lỗi đầu tiên rồi để người dùng sửa từng dòng một).
///
/// Trả `Err` với **mọi** [`ParseIssue`] tìm được trên toàn văn bản (trừ hai lỗi cấu trúc ở
/// hàng tiêu đề — [`ParseIssue::DelimiterUnresolved`]/[`ParseIssue::MissingColumn`] — dừng
/// ngay vì không còn gì để dò tiếp mà không biết dấu phân cách hay cột `source_term` nằm
/// đâu).
pub fn parse(text: &str) -> Result<ParsedImport, Vec<ParseIssue>> {
    let text = strip_bom(text);

    // I/O Matrix "Văn bản rỗng / chỉ có tiêu đề" — vế RỖNG HOÀN TOÀN: không hàng tiêu đề
    // nào để mà dò dấu phân cách. 0 mục, không lỗi.
    if text.trim().is_empty() {
        return Ok(ParsedImport::default());
    }

    let lines = logical_lines(text);
    let (_, header_text) = lines[0];

    let has_comma = header_text.contains(',');
    let has_tab = header_text.contains('\t');
    let delimiter = match (has_comma, has_tab) {
        (true, false) => Delimiter::Csv,
        (false, true) => Delimiter::Tsv,
        _ => return Err(vec![ParseIssue::DelimiterUnresolved]),
    };
    let d = delimiter.as_char();

    let header: Vec<String> = split_fields(header_text, d).iter().map(|s| s.trim().to_owned()).collect();
    let expected = header.len();

    let source_term_idx = header.iter().position(|c| c == "source_term");
    let Some(source_term_idx) = source_term_idx else {
        return Err(vec![ParseIssue::MissingColumn { column: "source_term" }]);
    };
    let translation_idx = header.iter().position(|c| c == "translation");
    let note_idx = header.iter().position(|c| c == "note");
    let category_idx = header.iter().position(|c| c == "category");
    let created_at_idx = header.iter().position(|c| c == "created_at");
    // `term_origin` -- vị trí của nó ở header không được đọc: đường nhập luôn tự đặt
    // `file_import` (§Boundaries: xuất xứ tự đặt, KHÔNG nhận qua tệp) — cột này chỉ mang
    // thông tin cho người ĐỌC tệp, không có tác dụng trên đường ghi.
    let known: std::collections::BTreeSet<&str> = COLUMNS.iter().copied().collect();
    let ignored_columns: Vec<String> =
        header.iter().filter(|c| !known.contains(c.as_str())).cloned().collect();

    let mut issues: Vec<ParseIssue> = Vec::new();
    let mut rows: Vec<ImportRow> = Vec::new();
    let mut seen: BTreeMap<String, usize> = BTreeMap::new();

    for (line, raw) in &lines[1..] {
        let line = *line;

        // 🔵 THÊM 2026-08-25 (vòng rà ba lớp, P2) — một dòng logic RỖNG (trim ra chuỗi rỗng)
        // KHÔNG phải một hàng dữ liệu: bỏ qua, không sinh `ParseIssue`. Áp CÙNG MỘT luật cho
        // cả hai vị trí nó có thể xuất hiện, và đó là một quyết định, không phải một chỗ hụt:
        // ① dòng trống Ở CUỐI tệp — vô số trình soạn thảo tự thêm một `\n` cuối cùng, và một
        //    tệp `…\n\n` (đo được: `慕容,...\n\n`) trước lượt vá này bị từ chối TRỌN VẸN với
        //    `CellCountMismatch` trỏ vào một dòng người dùng nhìn thấy là rỗng — một tệp hợp
        //    lệ bị từ chối vì một thói quen của trình soạn thảo, không phải vì dữ liệu sai.
        // ② dòng trống Ở GIỮA tệp (một dòng Enter thừa khi dán/soạn tay) — xử CÙNG MỘT LUẬT
        //    với ①: bỏ qua lặng lẽ, không phải lỗi. Cân nhắc đã cân: coi nó là lỗi
        //    (`CellCountMismatch`) sẽ phạt một lỗi đánh máy vô hại giống hệt cách phạt một
        //    dòng dữ liệu thật bị cắt cụt — hai tình huống khác hẳn nhau về mức nghiêm trọng
        //    nhưng bề ngoài (một dòng trống giữa hai dòng có nội dung) giống nhau, và người
        //    dùng không có cách nào tự phân biệt "trình soạn thảo thêm dòng trống" với "tôi
        //    gõ nhầm Enter" để mà sửa cho đúng. Không đề xuất số dòng bị lệch (số dòng thật
        //    KHÔNG đổi — `logical_lines` đã đánh số theo dòng NGUỒN, một dòng bị bỏ qua không
        //    làm dòng sau đó đổi số).
        if raw.trim().is_empty() {
            continue;
        }

        let cells = split_fields(raw, d);
        if cells.len() != expected {
            issues.push(ParseIssue::CellCountMismatch { line, expected, found: cells.len() });
            continue;
        }

        let source_term = cells[source_term_idx].trim();
        if source_term.is_empty() {
            issues.push(ParseIssue::BlankSourceTerm { line });
            continue;
        }

        let category = match category_idx {
            None => Category::Other,
            Some(idx) => {
                let raw_value = cells[idx].trim();
                if raw_value.is_empty() {
                    Category::Other
                } else {
                    match Category::from_wire(raw_value) {
                        Some(c) => c,
                        None => {
                            issues.push(ParseIssue::UnknownCategory {
                                line,
                                value: raw_value.to_owned(),
                            });
                            continue;
                        }
                    }
                }
            }
        };

        // 🔵 THÊM 2026-08-25 (vòng rà ba lớp, P3) — kiểm hình dạng TRƯỚC khi chấp nhận, cùng
        // vị trí trong chuỗi mà `category`/`source_term` đã được kiểm ngay trên. Cột vắng
        // hoặc ô rỗng vẫn là `None` (I/O Matrix "Vắng cột tuỳ chọn ⇒ created_at = hôm nay") —
        // chỉ một Ô CÓ MẶT VÀ SAI HÌNH DẠNG mới sinh lỗi.
        let created_at = match created_at_idx.map(|idx| cells[idx].trim()) {
            None => None,
            Some(s) if s.is_empty() => None,
            Some(s) if looks_like_iso8601_utc(s) => Some(s.to_owned()),
            Some(s) => {
                issues.push(ParseIssue::InvalidCreatedAt { line, value: s.to_owned() });
                continue;
            }
        };

        if let Some(&first_line) = seen.get(source_term) {
            issues.push(ParseIssue::DuplicateSourceTerm { first_line, second_line: line });
            continue;
        }

        let translation = translation_idx
            .map(|idx| cells[idx].trim())
            .filter(|s| !s.is_empty())
            .map(str::to_owned);
        let note = note_idx.map(|idx| cells[idx].trim().to_owned()).unwrap_or_default();

        seen.insert(source_term.to_owned(), line);
        rows.push(ImportRow {
            line,
            source_term: source_term.to_owned(),
            translation,
            note,
            category,
            created_at,
        });
    }

    if !issues.is_empty() {
        return Err(issues);
    }

    Ok(ParsedImport { rows, ignored_columns })
}

// ═════════════════════════════════════════════════════════════════════════════════
// PHÂN LOẠI — classify
// ═════════════════════════════════════════════════════════════════════════════════

/// Quyết định của người dùng cho một hàng *bất đồng* — mặc định [`ConflictDecision::KeepMine`]
/// (§Always: "Không im lặng ghi đè... Mặc định là giữ của tôi").
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConflictDecision {
    /// Giữ bản dịch đang có trong kho — KHÔNG ghi gì cho hàng này. Mặc định.
    KeepMine,
    /// Lấy bản dịch/ghi chú/phân loại từ tệp — `UPDATE` hàng đang có.
    TakeTheirs,
}

/// Phân loại một hàng đã phân tích so với tầng đích.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RowPlanKind {
    /// `source_term` chưa có ở tầng đích.
    New,
    /// Cùng `source_term`, CÙNG `translation` — không đề nghị gì, không ghi.
    Identical,
    /// Cùng `source_term`, KHÁC `translation` — mang cả hai bản dịch để người dùng quyết.
    Conflict {
        /// `id` của hàng đang có ở tầng đích — cùng kho với `tier` của lượt nhập.
        existing_id: i64,
        /// Bản dịch ĐANG CÓ trong kho, trước khi có quyết định nào.
        existing_translation: Option<String>,
    },
}

/// Một hàng đã phân tích, gắn kèm kết quả phân loại — [`super::store::import_into_tier`]
/// đọc `kind` để quyết định ghi gì.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RowPlan {
    /// Số dòng nguồn — cho chẩn đoán.
    pub line: usize,
    /// `source_term` đã cắt khoảng trắng.
    pub source_term: String,
    /// Bản dịch ĐỌC ĐƯỢC TỪ TỆP — với [`RowPlanKind::Conflict`], đây là vế "bản dịch của
    /// tệp"; vế "bản dịch đang có" nằm trong chính biến thể đó.
    pub translation: Option<String>,
    pub note: String,
    pub category: Category,
    /// `Some` khi tệp mang cột `created_at` không rỗng cho hàng này — [`RowPlanKind::New`]
    /// dùng giá trị này khi ghi (vòng tròn xuất→nhập giữ nguyên mốc); các biến thể khác
    /// không đụng tới trường này.
    pub created_at: Option<String>,
    pub kind: RowPlanKind,
}

/// Kết quả một lượt [`super::store::import_into_tier`] — số hàng THẬT đã đổi trên đĩa.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ImportSummary {
    /// Số hàng MỚI đã chèn ([`RowPlanKind::New`]).
    pub inserted: i64,
    /// Số hàng ĐÃ CÓ được cập nhật ([`RowPlanKind::Conflict`] + quyết định `TakeTheirs`).
    pub updated: i64,
    /// Số hàng phân loại *giống hệt* — không đổi gì, chỉ để hiển thị.
    pub identical: i64,
}

/// Phân loại `rows` so với `existing` (tầng ĐÍCH, đã nạp bằng `load_tier`) — hàm THUẦN
/// (§Boundaries: "phân loại là hàm THUẦN để nghiệm thu được không cần kho").
///
/// So sánh CHỈ trên `translation` (I/O Matrix: "Hàng giống hệt | Cùng source_term, cùng
/// translation" — không so `note`/`category`) — một khác biệt ở hai trường đó không được
/// ma trận liệt là một lớp phân loại riêng, nên nó không tạo ra một nhánh thứ tư ở đây.
pub fn classify(rows: &[ImportRow], existing: &BTreeMap<String, GlossaryEntry>) -> Vec<RowPlan> {
    rows.iter()
        .map(|row| {
            let kind = match existing.get(&row.source_term) {
                None => RowPlanKind::New,
                Some(entry) if entry.translation == row.translation => RowPlanKind::Identical,
                Some(entry) => RowPlanKind::Conflict {
                    existing_id: entry.id,
                    existing_translation: entry.translation.clone(),
                },
            };
            RowPlan {
                line: row.line,
                source_term: row.source_term.clone(),
                translation: row.translation.clone(),
                note: row.note.clone(),
                category: row.category,
                created_at: row.created_at.clone(),
                kind,
            }
        })
        .collect()
}
