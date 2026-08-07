//! Chuỗi pipeline nhập tối thiểu — AD-39, Story 1.15, AC1/AC8.
//!
//! ─────────────────────────────────────────────────────────────────────────────
//! 🔴 BA ĐƯỜNG VÀO, ĐÚNG MỘT HÀM THUẦN — AD-39:498
//! ─────────────────────────────────────────────────────────────────────────────
//! Dán văn bản · kéo-thả *(nhận đường dẫn qua `tauri://drag-drop`)* · ô nhập đường dẫn.
//! Hai đường sau gặp nhau ở [`import_file`] (`std::fs::read` → giải mã), rồi cả ba đổ vào
//! **cùng** [`import_text`] — không module nào giữ một bản sao của bước "chuẩn hoá tối
//! thiểu". [`import_file`] và [`import_text`] là **hàm thuần**: không `tauri::`, không
//! `rusqlite` — đọc tệp qua `std::fs` là chuyện bình thường ở `core/**` (chỉ `ports/**` bị
//! cấm chạm filesystem; ranh giới đó không áp ở đây).
//!
//! ─────────────────────────────────────────────────────────────────────────────
//! 🔴 THỨ TỰ CỐ ĐỊNH — LỆNH GHI Ở CUỐI (AD-39:485)
//! ─────────────────────────────────────────────────────────────────────────────
//! `phân loại nguồn → giải mã (UTF-8) → chuẩn hoá tối thiểu → tạo 1 Chương → ghi`.
//! Module này dừng lại ở **"tạo 1 Chương"** — nó trả về [`ImportedChapter`], một giá trị
//! thuần; bước **"ghi"** cần `Store::write` (SQL), nên nó sống ở `commands::project`
//! (Task 6 của story), không ở đây. AD-39 cấm chèn một bước biến đổi văn bản **sau**
//! lệnh ghi — module này không cho phép điều đó xảy ra vì nó không hề biết tới lệnh ghi.
//!
//! ─────────────────────────────────────────────────────────────────────────────
//! BA BƯỚC CHỪA CHỖ, KHÔNG CÀI — quyết định có chủ ý, không phải thiếu sót
//! ─────────────────────────────────────────────────────────────────────────────
//! - **Tách Chương** (FR14) — Epic 6. Story này tạo đúng MỘT Chương từ toàn bộ văn bản.
//! - **Làm sạch xuống dòng/khoảng trắng** (FR124/125) — Story 6.4/6.5.
//! - **Dò bảng mã** (FR126: UTF-8/GB18030/GBK/Big5/UTF-16) — Story 6.1-6.3. Story này
//!   **chỉ** nhận UTF-8 và từ chối tường minh mọi thứ khác (Quyết định #6) — AD-4 đóng
//!   băng ranh giới segment tính lúc nhập, nên văn bản giải mã sai ghi xuống hôm nay là
//!   dữ liệu Epic 6 không sửa lại được.
//!
//! `chuẩn hoá tối thiểu` trong [`import_text`] vì thế là một bước **có mặt nhưng rỗng** —
//! đúng vị trí trong chuỗi, sẵn sàng cho ba story trên, không phải một bước bị bỏ quên.

use std::collections::BTreeMap;
use std::path::Path;

use crate::core::i18n::{IpcError, MessageKey};

/// Hai phần mở rộng được nhận ở đường tối thiểu này (FR13 nhánh tối thiểu). `.docx` và mọi
/// thứ khác đóng ở Epic 6 — xem AC8.
const SUPPORTED_EXTENSIONS: [&str; 2] = ["txt", "md"];

/// Trần kích thước một tệp nhập — **100 MB**, Ice chốt ở lượt code review 2026-08-06.
///
/// 🔴 Vì sao phải có trần: [`import_file`] gọi `std::fs::read` (đọc TRỌN tệp vào bộ nhớ),
/// rồi `String::from_utf8` (một bản nữa), rồi chỗ gọi bind cả chuỗi vào một cột SQLite —
/// tất cả trên **luồng invoke đồng bộ**. Không có trần, một tệp vài GB làm cạn bộ nhớ,
/// và `panic = "abort"` biến chuyện đó thành **giết cả tiến trình**, không phải một lỗi
/// hiện ra được.
///
/// ⚠️ Con số này là **TẠM và chưa được đo** — chưa ai đo đỉnh RSS thật cho một tệp
/// 100 MB đi hết chuỗi (bytes + String + bind SQLite ≈ 3 bản). Nó cũng **không** phải
/// một phép đo về *"bao nhiêu thì Editor còn dùng được"* — đó là **Story 2.4** (sáu số
/// `Tuning`), story sở hữu việc đo lại. Trần này chỉ để một tệp bệnh hoạn không giết
/// tiến trình.
const MAX_IMPORT_BYTES: u64 = 100 * 1024 * 1024;

/// Dấu thứ tự byte UTF-8 (`EF BB BF`) — xem [`strip_bom`].
const BOM: char = '\u{feff}';

/// Mọi cách đường nhập từ chối một tệp/nội dung — AC8.
///
/// ⚠️ Vì sao đây KHÔNG PHẢI `StoreError`: cả hai biến thể xảy ra **trước** khi có gì chạm
/// tới `project.db` — không đĩa, không giao dịch SQL. Nhét chúng vào `StoreError` là gọi
/// một lỗi định dạng đầu vào là "lỗi kho", và `tests/scope_contract.rs` đúng khi đỏ trên
/// điều đó — xem ghi chú tại chỗ nó bị khoanh lại có ý thức trong `tests/project_contract.rs`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ImportError {
    /// Phần mở rộng chưa được nhận — `.docx`, hoặc bất kỳ thứ gì khác `.txt`/`.md`.
    UnsupportedFormat {
        /// Phần mở rộng đọc được (không có dấu chấm), thường thấy: `"docx"`. Rỗng nếu tệp
        /// không có phần mở rộng nào.
        format: String,
    },
    /// Nội dung tệp không giải mã được bằng UTF-8 (Quyết định #6).
    NotUtf8 {
        /// Đường dẫn tệp, cho chẩn đoán và cho tham số `path` của [`MessageKey::ImportNotUtf8`].
        path: String,
    },
    /// Đọc tệp trượt ở tầng I/O — quyền, tệp không tồn tại, ổ đĩa rút giữa chừng.
    ReadFailed {
        /// Đường dẫn tệp.
        path: String,
        /// Lỗi thô, chỉ để chẩn đoán.
        detail: String,
    },
    /// Tệp không có phần mở rộng nào — tách riêng khỏi [`ImportError::UnsupportedFormat`]
    /// vì một `format` rỗng làm câu thông báo vỡ (*"Định dạng . chưa được nhận"*).
    MissingExtension {
        /// Đường dẫn tệp.
        path: String,
    },
    /// Tệp vượt [`MAX_IMPORT_BYTES`].
    TooLarge {
        /// Kích thước thật, tính bằng byte.
        size: u64,
        /// Trần, tính bằng byte.
        limit: u64,
    },
}

impl std::fmt::Display for ImportError {
    /// ⚠️ KHÔNG DẤU — chẩn đoán cho log (NFR16).
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ImportError::UnsupportedFormat { format } => {
                write!(f, "import: unsupported format {format:?}")
            }
            ImportError::NotUtf8 { path } => write!(f, "import[{path}]: not valid utf-8"),
            ImportError::ReadFailed { path, detail } => {
                write!(f, "import[{path}]: read failed: {detail}")
            }
            ImportError::MissingExtension { path } => {
                write!(f, "import[{path}]: no file extension")
            }
            ImportError::TooLarge { size, limit } => {
                write!(f, "import: file is {size} bytes, limit is {limit}")
            }
        }
    }
}

impl std::error::Error for ImportError {}

/// 🔴 Đi qua [`IpcError::new`], không dựng struct literal — cùng luật với mọi chuyển đổi
/// lỗi khác của dự án.
impl From<ImportError> for IpcError {
    fn from(err: ImportError) -> Self {
        match err {
            ImportError::UnsupportedFormat { format } => {
                let mut params = BTreeMap::new();
                params.insert("format".to_owned(), format);
                IpcError::new(
                    "import.unsupported_format",
                    MessageKey::ImportUnsupportedFormat,
                    params,
                    false,
                )
            }
            ImportError::NotUtf8 { path } => {
                let mut params = BTreeMap::new();
                params.insert("path".to_owned(), path);
                IpcError::new("import.not_utf8", MessageKey::ImportNotUtf8, params, false)
            }
            ImportError::ReadFailed { path, .. } => {
                let mut params = BTreeMap::new();
                params.insert("path".to_owned(), path);
                // ⚠️ Tái dùng khoá CÓ SẴN `IoReadFailed` (Story 1.5) — đây là lỗi I/O
                // chung chung, không phải một hạng lỗi mới của story này.
                //
                // 🔴 `retryable = false`, KHÔNG `true`. Ca thật phổ biến nhất trên
                // đường này là một đường dẫn **gõ sai** trong ô nhập (`ENOENT`) — bấm lại
                // đúng nút ấy với đúng chuỗi ấy cho đúng kết quả ấy. AC8 gọi tên chính xác
                // chuyện này: *"một nút thử lại ở đó là **nói dối**"*. Một lỗi I/A thoáng
                // qua thật (ổ mạng chớp) tồn tại, nhưng người dùng sửa nó bằng cách **sửa
                // đường dẫn hoặc cắm lại ổ**, không bằng cách bấm lại — nên câu trung
                // thực là `false`.
                IpcError::new("io.read_failed", MessageKey::IoReadFailed, params, false)
            }
            ImportError::MissingExtension { path } => {
                let mut params = BTreeMap::new();
                params.insert("path".to_owned(), path);
                IpcError::new(
                    "import.missing_extension",
                    MessageKey::ImportMissingExtension,
                    params,
                    false,
                )
            }
            ImportError::TooLarge { size, limit } => {
                let mut params = BTreeMap::new();
                // `params` mang DỮ LIỆU, không mang CÂU (AD-21) — hai con số thô,
                // tầng hiển thị tự lo cách đọc chúng ra tiếng người.
                params.insert("size".to_owned(), size.to_string());
                params.insert("limit".to_owned(), limit.to_string());
                IpcError::new("import.too_large", MessageKey::ImportTooLarge, params, false)
            }
        }
    }
}

/// Kết quả của một lượt nhập tối thiểu: **đúng một** Chương, nguyên khối, sẵn sàng ghi.
///
/// Không mang `segment` nào (Quyết định #4) — chỉ văn bản nguyên khối của Chương.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImportedChapter {
    /// Văn bản nguồn của Chương, sau bước "chuẩn hoá tối thiểu".
    pub source_text: String,
}

/// **Hàm thuần duy nhất** mà cả ba đường vào của AC1 đổ vào (AD-39:498).
///
/// Bước "chuẩn hoá tối thiểu" của chuỗi AD-39 — vẫn **rỗng có chủ ý** (FR124/125 là Epic
/// 6). Thứ duy nhất chạy ở đây là [`strip_bom`], và nó **không phải** bước chuẩn hoá —
/// xem doc-comment của hàm đó.
pub fn import_text(raw: String) -> ImportedChapter {
    ImportedChapter {
        source_text: strip_bom(raw),
    }
}

/// Cắt dấu thứ tự byte (`U+FEFF`) ở **đầu** chuỗi, nếu có.
///
/// ─────────────────────────────────────────────────────────────────────────────
/// 🔴 ĐÂY LÀ BƯỚC **GIẢI MÃ**, KHÔNG PHẢI BƯỚC **CHUẨN HOÁ** CỦA EPIC 6
/// ─────────────────────────────────────────────────────────────────────────────
/// Ranh giới này là thứ giữ cho story không lấn sang FR124/125, nên nó phải được nói rõ:
/// BOM là một **tạo tác của phép mã hoá**, không phải một đặc điểm của văn bản. Mọi bộ
/// giải mã UTF-8 nghiêm túc đều nuốt nó. Cắt nó hoàn tất đúng bước mà **Quyết định #6** đã
/// giao cho story này *(giải mã, không đoán bảng mã)* — nó **không** chạm tới xuống
/// dòng, khoảng trắng hay bất cứ thứ gì Epic 6 sở hữu.
///
/// 🔴 Vì sao KHÔNG hoãn được sang Epic 6 — lập luận y hệt Quyết định #6: `EF BB BF` là
/// UTF-8 **hợp lệ**, nên nó đi lọt `String::from_utf8` mà không một cổng nào kêu. AD-4
/// đóng băng ranh giới segment tính **một lần lúc nhập**, nên một `U+FEFF` nằm lại sẽ trở
/// thành ký tự đầu của **segment #1**, với một `segment.id` mà AD-3 nói **không bao giờ**
/// được tái dùng. ⇒ Epic 6 **không sửa lại được**. Mọi tệp `.txt` do Notepad của Windows
/// lưu ở dạng "UTF-8" đều mang nó.
///
/// ⚠️ **CRLF thì ngược lại, và story này không CỐ Ý KHÔNG ĐỤNG** — xuống dòng **là** chuẩn hoá
/// văn bản thật (FR124/125), nó đổi chỗ ngắt đoạn, tức là đụng thẳng vào thứ Story 2.1 và
/// Epic 6 sở hữu. Sửa nó ở đây đúng là cái bẫy *"bộ tách tạm"* mà §ĐỌC TRƯỚC TIÊN ② cấm.
/// Ghi thành nợ trong `deferred-work.md`, không cài. *(Ice chốt, code review 2026-08-06.)*
///
/// Chỉ cắt ở **đầu**: một `U+FEFF` ở giữa văn bản là zero-width no-break space, một ký
/// tự thật của nội dung — cắt nó là sửa văn bản của người dùng.
fn strip_bom(raw: String) -> String {
    match raw.strip_prefix(BOM) {
        Some(rest) => rest.to_owned(),
        None => raw,
    }
}

/// Đường tệp (kéo-thả **hoặc** ô nhập đường dẫn — cả hai nhận một đường dẫn thật, không
/// phải nội dung tệp đã đọc sẵn từ webview, xem AD-1/AD-16).
///
/// Thứ tự: từ chối theo phần mở rộng **trước khi mở tệp** (không đọc một byte cho
/// `.docx`) → `std::fs::read` → giải mã UTF-8 (`String::from_utf8`, **không** `_lossy` —
/// Bẫy 8) → đổ vào [`import_text`].
pub fn import_file(path: &Path) -> Result<ImportedChapter, ImportError> {
    reject_unsupported_extension(path)?;

    // 🔴 Hỏi KÍCH THƯỚC trước khi đọc — không đọc rồi mới đo. `metadata` là một lượt
    // `stat`, không nạp một byte nội dung nào; đo sau khi `fs::read` thì bộ nhớ đã cạn
    // xong rồi mới biết. Xem [`MAX_IMPORT_BYTES`].
    //
    // ⚠️ Vẫn còn một cửa sổ đua (tệp phình ra giữa `stat` và `read`) — không đóng ở
    // story này: nó đòi đọc theo khối có trần, và đường nhập theo khối là Epic 6.
    let size = std::fs::metadata(path)
        .map_err(|e| ImportError::ReadFailed {
            path: path.display().to_string(),
            detail: e.to_string(),
        })?
        .len();

    if size > MAX_IMPORT_BYTES {
        return Err(ImportError::TooLarge {
            size,
            limit: MAX_IMPORT_BYTES,
        });
    }

    let bytes = std::fs::read(path).map_err(|e| ImportError::ReadFailed {
        path: path.display().to_string(),
        detail: e.to_string(),
    })?;

    let text = String::from_utf8(bytes).map_err(|_| ImportError::NotUtf8 {
        path: path.display().to_string(),
    })?;

    Ok(import_text(text))
}

/// Từ chối một phần mở rộng chưa được nhận — **trước** khi mở tệp, không đọc một byte.
///
/// ⚠️ Không phân biệt hoa/thường: `.TXT`, `.Md` đều được nhận.
///
/// 🔴 **Không có phần mở rộng ⇒ một hạng lỗi RIÊNG**, không phải `UnsupportedFormat`
/// với `format` rỗng: khoá `err.import.unsupported_format` nội suy `{format}` vào giữa
/// câu, nên một chuỗi rỗng cho ra *"Định dạng . chưa được nhận…"* — một câu vỡ, đọc như
/// một lỗi của ứng dụng chứ không phải một lời giải thích. Cùng lớp với thứ §Voice and
/// Tone cấm.
fn reject_unsupported_extension(path: &Path) -> Result<(), ImportError> {
    let Some(ext) = path.extension().and_then(|e| e.to_str()) else {
        return Err(ImportError::MissingExtension {
            path: path.display().to_string(),
        });
    };

    let ext = ext.to_ascii_lowercase();

    if SUPPORTED_EXTENSIONS.contains(&ext.as_str()) {
        return Ok(());
    }

    Err(ImportError::UnsupportedFormat { format: ext })
}
