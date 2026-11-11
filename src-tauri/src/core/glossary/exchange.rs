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

pub use crate::core::delimited_text::Delimiter;
use crate::core::delimited_text::{
    self, DelimitedError, count_line_breaks, render_field, strip_bom, strip_formula_guard,
    unquoted_char_present,
};
pub(crate) use crate::core::delimited_text::{
    split_fields, split_first_logical_line, strip_zero_width,
};

use super::entry::{Category, GlossaryEntry};

pub(crate) fn logical_lines(
    text: &str,
    delimiter: Option<char>,
    start_line: usize,
) -> Result<Vec<(usize, &str)>, ParseIssue> {
    delimited_text::logical_lines(text, delimiter, start_line).map_err(|e| match e {
        DelimitedError::UnterminatedQuotedField { line } => {
            ParseIssue::UnterminatedQuotedField { line }
        }
    })
}

/// Sáu cột của tệp xuất — thứ tự CỐ ĐỊNH, dùng cho CẢ hàng tiêu đề lẫn việc dò cột lúc
/// nhập. Định danh máy đọc, KHÔNG DẤU (§Boundaries — Kiểm A của `check:i18n`, cùng luật đã
/// đặt cho `Category::as_str()`, AD-21/NFR16).
const COLUMNS: [&str; 6] = [
    "source_term",
    "translation",
    "note",
    "category",
    "term_origin",
    "created_at",
];

// ═════════════════════════════════════════════════════════════════════════════════
// XUẤT — render_tier
// ═════════════════════════════════════════════════════════════════════════════════

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
    /// 🔵 **THÊM 2026-08-25 (vòng rà ba lớp, cụm B, mục ②).** Một ô mở dấu ngoặc kép nhưng
    /// KHÔNG BAO GIỜ đóng lại trước khi hết văn bản — trước bản vá này, `split_first_
    /// logical_line` nuốt TOÀN BỘ phần còn lại của tệp (mọi `\n` sau đó bị hiểu là NỘI DUNG
    /// của ô đang mở) vào MỘT "dòng logic" duy nhất, rồi hàng đó trượt `CellCountMismatch`
    /// tại DÒNG CUỐI tệp — che mất hàng trăm hàng ĐÚNG phía sau và trỏ người dùng đi sai
    /// chỗ. Biến thể này có tên riêng và mang đúng số dòng nơi Ô ĐÓ mở ra.
    UnterminatedQuotedField {
        /// Số dòng nơi hàng chứa ô mở ngoặc kép bắt đầu.
        line: usize,
    },
    /// 🔵 **THÊM 2026-08-25 (vòng rà ba lớp, cụm B, mục ⑤).** Hàng tiêu đề mang HAI cột
    /// cùng một tên ĐÃ BIẾT (một trong [`COLUMNS`]) — `header.iter().position(..)` chỉ tìm
    /// khớp ĐẦU TIÊN, nên cột thứ hai (và mọi giá trị của nó) mất im lặng, không vào
    /// `ignored_columns` (nó KHÔNG phải một tên lạ), không sinh lỗi nào ở bản trước.
    DuplicateColumn {
        /// Tên cột trùng — dữ liệu, không phải câu.
        column: String,
    },
}

impl std::fmt::Display for ParseIssue {
    /// ⚠️ KHÔNG DẤU — chẩn đoán cho log (NFR16).
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParseIssue::DelimiterUnresolved => {
                write!(
                    f,
                    "glossary import: header has both or neither of ',' and TAB"
                )
            }
            ParseIssue::MissingColumn { column } => {
                write!(
                    f,
                    "glossary import: header missing required column {column:?}"
                )
            }
            ParseIssue::CellCountMismatch {
                line,
                expected,
                found,
            } => {
                write!(
                    f,
                    "glossary import[line {line}]: {found} cells, header has {expected}"
                )
            }
            ParseIssue::UnknownCategory { line, value } => {
                write!(
                    f,
                    "glossary import[line {line}]: unknown category {value:?}"
                )
            }
            ParseIssue::BlankSourceTerm { line } => {
                write!(f, "glossary import[line {line}]: source_term is blank")
            }
            ParseIssue::DuplicateSourceTerm {
                first_line,
                second_line,
            } => {
                write!(
                    f,
                    "glossary import: source_term duplicated at lines {first_line} and {second_line}"
                )
            }
            ParseIssue::InvalidCreatedAt { line, value } => {
                write!(
                    f,
                    "glossary import[line {line}]: created_at not iso-8601 utc {value:?}"
                )
            }
            ParseIssue::UnterminatedQuotedField { line } => {
                write!(
                    f,
                    "glossary import[line {line}]: quoted field opened but never closed"
                )
            }
            ParseIssue::DuplicateColumn { column } => {
                write!(
                    f,
                    "glossary import: header column {column:?} appears more than once"
                )
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
            ParseIssue::CellCountMismatch {
                line,
                expected,
                found,
            } => {
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
            ParseIssue::DuplicateSourceTerm {
                first_line,
                second_line,
            } => {
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
            ParseIssue::UnterminatedQuotedField { line } => {
                let mut params = BTreeMap::new();
                params.insert("line".to_owned(), line.to_string());
                IpcError::new(
                    "glossary.import_unterminated_quoted_field",
                    MessageKey::GlossaryImportUnterminatedQuotedField,
                    params,
                    false,
                )
            }
            ParseIssue::DuplicateColumn { column } => {
                let mut params = BTreeMap::new();
                params.insert("column".to_owned(), column);
                IpcError::new(
                    "glossary.import_duplicate_column",
                    MessageKey::GlossaryImportDuplicateColumn,
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
    /// 🔵 **THÊM 2026-08-25 (Story 3.10b).** TOÀN BỘ tên cột của hàng tiêu đề, đã cắt
    /// khoảng trắng, theo ĐÚNG thứ tự trong tệp — nhận diện lẫn bị bỏ qua. Màn hình xem
    /// trước (AD-48 §Rule ①: chỉ mô hình đã kiểm, không văn bản thô) suy ra từ trường này:
    /// *"N cột nhận ra được"* = `header_columns.len() - ignored_columns.len()`, và *"tệp có
    /// cột `term_origin`"* = `header_columns.contains(&"term_origin".to_owned())` — cột đó
    /// LUÔN nằm trong [`COLUMNS`] (không bao giờ rơi vào `ignored_columns`) nhưng giá trị
    /// của nó bị đọc rồi bỏ (§Design Notes của spec 3.10b), nên overlay cần một tín hiệu
    /// RIÊNG để nói ra điều đó — không suy được từ `ignored_columns`.
    pub header_columns: Vec<String>,
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

/// Phân tích TRỌN văn bản một tệp CSV/TSV thành các hàng — hàm THUẦN, không chạm Store
/// (§Design Notes: phân tích trọn TRƯỚC khi mở giao dịch là chỗ trả về được MỘT danh sách
/// lỗi, không phải dừng ở lỗi đầu tiên rồi để người dùng sửa từng dòng một).
///
/// Trả `Err` với **mọi** [`ParseIssue`] tìm được trên toàn văn bản (trừ bốn lỗi CẤU TRÚC —
/// [`ParseIssue::UnterminatedQuotedField`] ở hàng tiêu đề, [`ParseIssue::DelimiterUnresolved`],
/// [`ParseIssue::DuplicateColumn`], [`ParseIssue::MissingColumn`] — dừng ngay vì không còn
/// gì để dò tiếp mà không biết dấu phân cách/chỉ số cột nằm đâu. 🔵 **CẬP NHẬT 2026-08-25
/// (vòng rà ba lớp, cụm B)** — trước bản vá chỉ có HAI lỗi cấu trúc dừng-ngay; mục ② và ⑤
/// thêm hai lỗi nữa vào nhóm đó. [`ParseIssue::UnterminatedQuotedField`] ở phần THÂN tệp
/// cũng dừng ngay, nhưng vì một lý do KHÁC — xem ngay dưới).
///
/// 🔵 **SỬA 2026-08-25 (vòng rà ba lớp, P5) — lý do "không biết dấu phân cách/chỉ số cột nằm
/// đâu" KHÔNG áp được cho ca THÂN TỆP**, và câu bản trước gộp cả hai ca vào một lý do là sai
/// phạm vi: khi thân tệp bắt đầu được đọc thì dấu phân cách VÀ chỉ số cột đều đã chốt xong.
/// Lý do THẬT của ca thân tệp: một ô mở nháy kép mà không đóng nuốt toàn bộ phần văn bản còn
/// lại vào MỘT dòng logic, nên không còn hàng nào phân giải được để mà kiểm — dừng ở đó không
/// đánh rơi lỗi nào, vì [`logical_lines`] chạy TRỌN VẸN trước khi vòng kiểm hàng bắt đầu, tức
/// lúc nó trả `Err` thì chưa một [`ParseIssue`] cấp-hàng nào được sinh ra.
pub fn parse(text: &str) -> Result<ParsedImport, Vec<ParseIssue>> {
    let text = strip_bom(text);

    // I/O Matrix "Văn bản rỗng / chỉ có tiêu đề" — vế RỖNG HOÀN TOÀN: không hàng tiêu đề
    // nào để mà dò dấu phân cách. 0 mục, không lỗi.
    if text.trim().is_empty() {
        return Ok(ParsedImport::default());
    }

    // 🔵 Story 3.10b — hàng TIÊU ĐỀ tách ra TRƯỚC, bằng luật quét "cả hai ứng viên"
    // (`delimiter = None`); dấu phân cách chỉ chốt được SAU khi đọc xong hàng đó, nên
    // các hàng DỮ LIỆU phải tách bằng một lượt `logical_lines` THỨ HAI, dùng ĐÚNG MỘT
    // ký tự đã chốt — xem doc-comment của `split_first_logical_line`.
    //
    // 🔵 THÊM 2026-08-25 (mục ②) — hàng tiêu đề CŨNG có thể mở một ô nháy kép không bao giờ
    // đóng (`,"tieu de mo mai`); trước bản vá không nhánh nào bắt ca này ở ĐÂY, nó trôi tiếp
    // xuống dò delimiter/dò cột với một `header_text` là TOÀN BỘ phần còn lại của tệp.
    let (header_text, body_text, header_unterminated) = split_first_logical_line(text, None);
    if header_unterminated {
        return Err(vec![ParseIssue::UnterminatedQuotedField { line: 1 }]);
    }
    let header_line_count = count_line_breaks(header_text) + 1;

    // 🔵 SỬA 2026-08-25 (mục ④) — dò trên ô đã bọc được NHẬN DIỆN, không trên văn bản thô.
    // Xem doc-comment của `unquoted_char_present` cho lý do không tách `split_fields` hai
    // lượt (phương án bị loại ở §Ask First của spec).
    let has_comma = unquoted_char_present(header_text, ',');
    let has_tab = unquoted_char_present(header_text, '\t');
    let delimiter = match (has_comma, has_tab) {
        (true, false) => Delimiter::Csv,
        (false, true) => Delimiter::Tsv,
        _ => return Err(vec![ParseIssue::DelimiterUnresolved]),
    };
    let d = delimiter.as_char();

    let header: Vec<String> = split_fields(header_text, d)
        .iter()
        .map(|s| s.trim().to_owned())
        .collect();
    let expected = header.len();

    let known: std::collections::BTreeSet<&str> = COLUMNS.iter().copied().collect();

    // 🔵 THÊM 2026-08-25 (mục ⑤) — hai cột trùng TÊN ĐÃ BIẾT ở hàng tiêu đề. `position(..)`
    // ở dưới chỉ tìm khớp ĐẦU TIÊN; không có kiểm này, cột THỨ HAI (và mọi giá trị của nó)
    // mất im lặng — nó KHÔNG lọt vào `ignored_columns` (không phải một tên LẠ) và không sinh
    // lỗi nào. Dừng NGAY ở đây (giống `DelimiterUnresolved`/`MissingColumn`): không biết cột
    // nào trong hai cột trùng tên là cột "thật" thì không dò chỉ số nào phía dưới còn đáng
    // tin cả.
    let mut known_column_counts: BTreeMap<&str, usize> = BTreeMap::new();
    for c in &header {
        if let Some(&canonical) = known.get(c.as_str()) {
            *known_column_counts.entry(canonical).or_insert(0) += 1;
        }
    }
    let duplicate_column_issues: Vec<ParseIssue> = known_column_counts
        .into_iter()
        .filter(|&(_, count)| count > 1)
        .map(|(column, _)| ParseIssue::DuplicateColumn {
            column: column.to_owned(),
        })
        .collect();
    if !duplicate_column_issues.is_empty() {
        return Err(duplicate_column_issues);
    }

    let source_term_idx = header.iter().position(|c| c == "source_term");
    let Some(source_term_idx) = source_term_idx else {
        return Err(vec![ParseIssue::MissingColumn {
            column: "source_term",
        }]);
    };
    let translation_idx = header.iter().position(|c| c == "translation");
    let note_idx = header.iter().position(|c| c == "note");
    let category_idx = header.iter().position(|c| c == "category");
    let created_at_idx = header.iter().position(|c| c == "created_at");
    // `term_origin` -- vị trí của nó ở header không được đọc: đường nhập luôn tự đặt
    // `file_import` (§Boundaries: xuất xứ tự đặt, KHÔNG nhận qua tệp) — cột này chỉ mang
    // thông tin cho người ĐỌC tệp, không có tác dụng trên đường ghi.
    let ignored_columns: Vec<String> = header
        .iter()
        .filter(|c| !known.contains(c.as_str()))
        .cloned()
        .collect();

    // 🔵 SỬA 2026-08-25 (mục ②) — `logical_lines` nay trả `Result`; một ô mở ngoặc kép
    // không bao giờ đóng ở phần THÂN tệp dừng phân tích NGAY (không phải một `CellCountMismatch`
    // ở dòng CUỐI tệp nuốt mọi hàng đúng phía sau — xem doc-comment của `logical_lines`).
    let lines = match logical_lines(body_text, Some(d), 1 + header_line_count) {
        Ok(lines) => lines,
        Err(issue) => return Err(vec![issue]),
    };

    let mut issues: Vec<ParseIssue> = Vec::new();
    let mut rows: Vec<ImportRow> = Vec::new();
    let mut seen: BTreeMap<String, usize> = BTreeMap::new();

    for (line, raw) in &lines {
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
            issues.push(ParseIssue::CellCountMismatch {
                line,
                expected,
                found: cells.len(),
            });
            continue;
        }

        // 🔵 SỬA 2026-08-25 (mục ⑥/⑥b) — cắt ZERO-WIDTH trước khi trim khoảng trắng thường,
        // rồi trước khi kiểm rỗng. `source_term_no_zero_width` phải sống hết vòng lặp này:
        // `source_term` (biến ngay dưới) chỉ MƯỢN từ nó.
        let source_term_no_zero_width = strip_zero_width(&cells[source_term_idx]);
        let source_term = strip_formula_guard(source_term_no_zero_width.trim());
        if source_term.is_empty() {
            issues.push(ParseIssue::BlankSourceTerm { line });
            continue;
        }

        // 🔵 SỬA 2026-08-25 (Story 3.10b) — `category`/`created_at`/trùng `source_term` nay
        // KHÔNG `continue` riêng lẻ nữa: mỗi ca lỗi CHỈ đánh dấu `row_ok = false` rồi ĐI
        // TIẾP qua các kiểm còn lại của CÙNG hàng. Bản trước dừng ở lỗi ĐẦU TIÊN tìm được
        // trên một hàng (category sai ⇒ `continue` trước khi kịp kiểm trùng), nên một hàng
        // vừa trùng `source_term` VỪA sai `category` chỉ báo được MỘT trong hai lỗi — sai
        // đúng mệnh đề I/O Matrix "Hàng trùng source_term VÀ category lạ ⇒ báo CẢ HAI lỗi
        // cho hàng đó" (đóng `deferred-work.md §*Deferred from: 3-6-trang-thai-cho-chot-va-dai-moc-chot-lan-dau-gap (2026-08-22)*`).
        //
        // 🔵 SỬA 2026-08-25 (mục ⑦) — CÂU TRÊN (bản trước) HẾT ĐÚNG một phần: "`seen` vẫn
        // chỉ nhận hàng row_ok" ĐÚNG bản trước NHƯNG đó chính là lỗ hổng ⑦. `seen` nay ghi
        // nhận NGAY khi gặp `source_term` LẦN ĐẦU, bất kể hàng đó row_ok hay không (xem
        // `seen.entry(..)` phía dưới) — một hàng bị bác vì category/created_at KHÔNG được
        // phép làm lượt trùng ở một hàng SAU "không thấy" trùng nữa.
        let mut row_ok = true;

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
                            row_ok = false;
                            Category::Other
                        }
                    }
                }
            }
        };

        // Kiểm hình dạng TRƯỚC khi chấp nhận, cùng vị trí trong chuỗi mà `category`/
        // `source_term` đã được kiểm ngay trên. Cột vắng hoặc ô rỗng vẫn là `None` (I/O
        // Matrix "Vắng cột tuỳ chọn ⇒ created_at = hôm nay") — chỉ một Ô CÓ MẶT VÀ SAI
        // HÌNH DẠNG mới sinh lỗi.
        let created_at = match created_at_idx.map(|idx| cells[idx].trim()) {
            None => None,
            Some("") => None,
            Some(s) if looks_like_iso8601_utc(s) => Some(s.to_owned()),
            Some(s) => {
                issues.push(ParseIssue::InvalidCreatedAt {
                    line,
                    value: s.to_owned(),
                });
                row_ok = false;
                None
            }
        };

        // 🔵 SỬA 2026-08-25 (mục ⑦) — `entry(..).or_insert(line)` ghi nhận LẦN ĐẦU gặp
        // `source_term` này ngay tại đây, KHÔNG chờ hàng có row_ok hay không: nếu đã có một
        // mục cho `source_term` (từ một hàng TRƯỚC, dù hàng đó valid hay không), `or_insert`
        // KHÔNG ghi đè, trả về dòng đã ghi nhận trước đó — đúng "lần xuất hiện ĐẦU TIÊN".
        // Nếu chưa có, nó chèn `line` hiện tại và trả về CHÍNH `line`, nên `first_seen ==
        // line` ⇒ đây là lần đầu, không phải trùng.
        let first_seen = *seen.entry(source_term.to_owned()).or_insert(line);
        if first_seen != line {
            issues.push(ParseIssue::DuplicateSourceTerm {
                first_line: first_seen,
                second_line: line,
            });
            row_ok = false;
        }

        if !row_ok {
            continue;
        }

        // ⚠️ **BẤT ĐỐI XỨNG CÓ CHỦ, ghi ra vì không cổng nào canh nó** (vòng rà ba lớp, P8):
        // `render_field` rào công thức cho CẢ SÁU cột lúc xuất, còn `strip_formula_guard` chỉ
        // gỡ ở BA cột văn bản tự do (`source_term` · `translation` · `note`). Ba cột còn lại
        // KHÔNG cần gỡ vì không giá trị hợp lệ nào của chúng bắt đầu bằng `'`: `category` và
        // `term_origin` là enum đóng (một giá trị lạ đã bị `Category::from_str`/`TermOrigin`
        // bác thành `UnknownCategory` trước khi tới đây), `created_at` bị kiểm HÌNH DẠNG
        // ISO-8601 nên chỉ nhận chữ số và dấu phân cách. ⇒ Nếu sau này một trong ba cột đó
        // thành văn bản tự do, phải nối nó vào `strip_formula_guard` CÙNG LƯỢT.
        //
        // 🔵 SỬA 2026-08-25 (vòng rà ba lớp, mục ⑥c — Ice ký nới phạm vi) — `strip_zero_width`
        // nay áp cho CẢ BA cột văn bản tự do, không riêng `source_term`. Bản trước chỉ đóng
        // `source_term` và để `translation`/`note` — lấy ra cách đó hai dòng trong CÙNG vòng
        // lặp — chỉ có `.trim()`, tức chính lớp lỗi vừa đóng ở cột bên cạnh vẫn mở ở đây.
        //
        // 🔴 Vì sao vế `translation` nặng hơn hẳn hai vế kia: `.filter(|s| !s.is_empty())` là
        // thứ quyết định một mục vào CHỜ CHỐT (`None`) hay ĐÃ CHỐT (`Some`). Một ô toàn
        // U+200B không rỗng theo `str::trim()` (đo được: `is_whitespace` của Unicode KHÔNG
        // gồm zero-width), nên nó lọt thành `Some("\u{200B}")` ⇒ một mục ĐÃ CHỐT mang bản
        // dịch VÔ HÌNH. Trigger `glossary_entry_lifecycle_is_one_way` của AD-36 khiến trạng
        // thái đó KHÔNG lùi lại được, và `CHECK trim(translation, <25 điểm mã>) <> ''` phía
        // SQL cũng không cắt zero-width nên nó không đỡ hộ — đúng lớp "rỗng im lặng" mà kho
        // tự ghi là bug trung tâm.
        let translation = translation_idx
            .map(|idx| strip_zero_width(&cells[idx]))
            .map(|cell| strip_formula_guard(cell.trim()).to_owned())
            .filter(|s| !s.is_empty());
        let note = note_idx
            .map(|idx| strip_zero_width(&cells[idx]))
            .map(|cell| strip_formula_guard(cell.trim()).to_owned())
            .unwrap_or_default();

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

    Ok(ParsedImport {
        rows,
        ignored_columns,
        header_columns: header,
    })
}

// ═════════════════════════════════════════════════════════════════════════════════
// PHÂN LOẠI — classify
// ═════════════════════════════════════════════════════════════════════════════════

/// Quyết định của người dùng cho một hàng *bất đồng* — mặc định [`ConflictDecision::KeepMine`]
/// (§Always: "Không im lặng ghi đè... Mặc định là giữ của tôi").
///
/// 🔵 **THÊM 2026-08-25 (Story 3.10b) — `serde::Deserialize`.** Bản đồ quyết định của
/// nhịp hai (`commands::glossary::wire::confirm_import`) đi qua dây IPC dưới dạng
/// `BTreeMap<String, ConflictDecision>` — Tauri tự giải mã JSON TRƯỚC khi hàm thuần
/// chạy, cùng khuôn `Category`/`GlossaryTier`. `#[serde(rename = …)]` từng biến thể
/// (không `rename_all`), cùng tiền lệ đã chọn cho hai kiểu kia.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Deserialize)]
pub enum ConflictDecision {
    /// Giữ bản dịch đang có trong kho — KHÔNG ghi gì cho hàng này. Mặc định.
    #[serde(rename = "keep_mine")]
    KeepMine,
    /// Lấy bản dịch từ tệp — `UPDATE` **CHỈ** cột `translation` của hàng đang có.
    ///
    /// 🔵 **SỬA 2026-08-25 (Story 3.10b) — doc cũ ("Lấy bản dịch/ghi chú/phân loại từ
    /// tệp") HẾT ĐÚNG từ bản vá P1 của Story 3.10.** `store.rs::import_into_tier` đã đổi
    /// câu `UPDATE` sang chỉ chạm `translation` từ 2026-08-25 (§Spec Change Log của story
    /// đó) — `note`/`category` không bao giờ bị ghi qua nhánh này, kể cả khi tệp mang
    /// giá trị THẬT cho chúng. Sửa tại chỗ thay vì để câu cũ nói sai mãi.
    #[serde(rename = "take_theirs")]
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
