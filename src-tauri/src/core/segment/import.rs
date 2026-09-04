//! Bước ĐẦU VÀO của chuỗi pipeline nhập — AD-39, Story 1.15 (AC1/AC8), Story 6.2.
//!
//! ─────────────────────────────────────────────────────────────────────────────
//! 🔵 **SỬA 2026-09-04 (Story 6.2) — mô-đun này KHÔNG CÒN cài chuỗi, chỉ CUNG CẤP bước 0.**
//! ─────────────────────────────────────────────────────────────────────────────
//! Bản Story 1.15 (giữ nguyên ở lịch sử git, không phải ở đây) cài cả "phân loại nguồn →
//! giải mã → chuẩn hoá tối thiểu → tạo 1 Chương" trong CHÍNH tệp này. AD-39 (spine
//! `:473-482`) khai bảy bước dùng CHUNG mọi nguồn, và một mô-đun nguồn "chỉ cung cấp bước
//! đầu vào rồi trao lại" (spine `:498`) — nên bước GIẢI MÃ và [`strip_bom`] (trước đây ở
//! đây) đã CHUYỂN sang [`super::pipeline`], nơi chúng là [`super::pipeline::Step::DecodeEncoding`].
//! Tệp này giờ chỉ còn: đọc byte/nhận văn bản, từ chối phần mở rộng/kích thước KHÔNG hợp
//! lệ TRƯỚC khi trao đi (AC8 — hai việc này KHÔNG phải một phần của chuỗi bảy bước, chúng
//! xảy ra TRƯỚC khi có gì để mà chảy trong chuỗi), rồi trả về một
//! [`super::pipeline::PipelineShape`] — giá trị mà [`super::pipeline::run_import`] tiêu thụ.
//!
//! ─────────────────────────────────────────────────────────────────────────────
//! 🔴 BA ĐƯỜNG VÀO, ĐÚNG MỘT HÌNH DẠNG — AD-39:498
//! ─────────────────────────────────────────────────────────────────────────────
//! Dán văn bản · kéo-thả *(nhận đường dẫn qua `tauri://drag-drop`)* · ô nhập đường dẫn.
//! [`import_text`] và [`import_file`] là **hàm thuần**: không `tauri::`, không `rusqlite` —
//! đọc tệp qua `std::fs` là chuyện bình thường ở `core/**` (chỉ `ports/**` bị cấm chạm
//! filesystem; ranh giới đó không áp ở đây).
//!
//! ─────────────────────────────────────────────────────────────────────────────
//! 🔵 **SỬA 2026-09-04 (Story 6.2) — "BA BƯỚC CHỪA CHỖ" cũ đã hết đúng MỘT NỬA.**
//! ─────────────────────────────────────────────────────────────────────────────
//! Bản Story 1.15 khai ba bước "để trống": tách Chương, làm sạch xuống dòng/khoảng trắng,
//! dò bảng mã. Sau Story 6.2:
//! - **Tách Chương** (FR14) — có mặt trong [`super::pipeline::PIPELINE_ORDER`] với một cơ
//!   chế THẬT (so khớp literal), nhưng KHÔNG cấu hình được bởi người dùng; production luôn
//!   truyền `chapter_pattern: None` ⇒ N = 1 KHÔNG ĐỔI. Mẫu cấu hình được là Story 6.6.
//! - **Chuẩn hoá xuống dòng/khoảng trắng** (FR124/125) — vẫn THÂN RỖNG, Story 6.4/6.5.
//! - **Dò bảng mã** (FR126) — 🔵 **SỬA 2026-09-04 (Story 6.3) — "vẫn CHỈ giải mã theo MỘT
//!   bảng mã ĐÃ KHAI (mặc định UTF-8)" đã HẾT ĐÚNG.** Bộ dò thật giờ sống ở
//!   [`super::encoding`] (`sniff_bom` → `detect` → `render_candidates`, ba trạng thái tin
//!   cậy là luật CỦA TA, `chardetng` không cấp điểm số nào — xem doc-comment module đó).
//!   `commands::project::preview_import_encoding` gọi nó và
//!   `commands::project::confirm_import_with_encoding` khai bảng mã NGƯỜI DÙNG đã chọn qua
//!   [`super::pipeline::PipelineInput::with_encoding`] — hai hàm cũ
//!   (`create_work_from_text`/`create_work_from_file`) VẪN khai cứng UTF-8
//!   (`PipelineInput::default_shaped`), giữ nguyên cho `tests/**` và mọi chỗ gọi không đi
//!   qua màn xem trước. AD-4 đóng băng ranh giới segment tính lúc nhập, nên văn bản giải mã
//!   sai — hoặc sai vì người dùng chọn nhầm ứng viên — vẫn là dữ liệu không sửa lại được
//!   sau khi đã ghi xuống; đó là lý do màn xem trước tồn tại TRƯỚC khi ghi, không sau.

use std::collections::BTreeMap;
use std::path::Path;

use crate::core::i18n::{IpcError, MessageKey};

use super::pipeline::{ChapterInput, PipelineShape};
use super::split::SplitSegment;

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
    /// Nội dung không giải mã được bằng bảng mã ĐÃ KHAI/ĐÃ CHỌN (Quyết định #6, Story 1.15).
    ///
    /// 🔵 **ĐỔI TÊN 2026-09-04 (Story 6.3) — `NotUtf8` → `UndecodableBytes`, cộng trường
    /// `encoding`.** Bản Story 6.2 chỉ đặt tên đúng cho MỘT trường hợp (bảng mã đã khai
    /// LUÔN là UTF-8 trên đường sản phẩm); từ story này bảng mã có thể là bất kỳ nhãn nào
    /// trong FR126 mà người dùng đã CHỌN ở màn xem trước (`core::segment::encoding`), nên
    /// `NotUtf8` trở thành một NHÃN SAI đúng vào ngày GBK/Big5/GB18030/UTF-16 khai được —
    /// nợ đã ghi chủ Story 6.3 ở `deferred-work.md` (khối 6.2). Đây là đường lỗi DUY NHẤT
    /// còn lại cho bảng mã: byte không giải mã được với chính bảng mã ĐÃ CHỌN (§Always spec
    /// 6.3 — "không có trạng thái lỗi cho bảng mã đoán sai", đoán sai chỉ ra chữ không đọc
    /// được, mắt phân xử; đây là ca byte THẬT SỰ không hợp lệ với bảng đã chọn).
    UndecodableBytes {
        /// Đường dẫn/tên nguồn, cho chẩn đoán và cho tham số `path` của
        /// [`MessageKey::ImportUndecodableBytes`].
        path: String,
        /// Tên WHATWG của bảng mã ĐÃ CHỌN (`Encoding::name()`, ví dụ `"GBK"`) — dữ liệu,
        /// không phải câu (AD-21). Tham số `encoding` của
        /// [`MessageKey::ImportUndecodableBytes`] — I/O Matrix spec 6.3: "Xác nhận với
        /// bảng mã đã chọn... Từ chối tường minh, nêu ĐÍCH DANH bảng mã đã chọn".
        encoding: String,
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
    /// 🔵 **THÊM (vòng rà đối kháng 2026-09-04) — `order` truyền cho
    /// `pipeline::run_import_with_order` không phải một hoán vị hợp lệ của bảy biến thể
    /// `pipeline::Step`** (thiếu bước, thừa bước, hoặc một bước lặp lại).
    ///
    /// ⚠️ CHỈ xảy ra khi CHÍNH MÃ RUST vi phạm hợp đồng của hàm đó — đường sản phẩm luôn
    /// truyền `&PIPELINE_ORDER` không đổi (`pipeline::run_import`); không đầu vào NGƯỜI
    /// DÙNG nào lái được tới nhánh này. Dùng [`crate::core::i18n::MessageKey::Unknown`]
    /// (khoá dự phòng AD-21, KHÔNG tham số) khi chuyển sang `IpcError` — một câu tường
    /// minh cho người dùng không cần tồn tại, vì ca này không bao giờ chạm một bề mặt IPC
    /// thật.
    InvalidPipelineOrder {
        /// Chẩn đoán CHỈ cho log (không đi vào `IpcError`, `Unknown` không nhận tham số).
        detail: String,
    },
    /// 🔵 **THÊM 2026-09-04 (Story 6.3)** — `commands::project::confirm_import_with_encoding`
    /// nhận một `wire_id` không giải ngược được thành `&'static encoding_rs::Encoding` qua
    /// [`crate::core::segment::encoding::encoding_for_wire_id`]. §Design Notes spec 6.3,
    /// "Nhãn đi qua dây phải KHÔNG MẤT MÁT": *"một nhãn KHÔNG NHẬN RA là một vi phạm hợp
    /// đồng ⇒ `IpcError` tường minh, KHÔNG âm thầm rơi về UTF-8"*. Đường sản phẩm luôn
    /// truyền một `wire_id` mà chính Rust vừa cấp ở lượt xem trước
    /// (`EncodingCandidate::wire_id`), nên nhánh này chỉ chạm khi webview gửi một chuỗi lạ
    /// (lỗi lập trình phía frontend, hoặc một phiên rất cũ mang khoá đã đổi hình dạng).
    UnrecognizedEncoding {
        /// Chuỗi wire nhận được — dữ liệu, không phải câu.
        wire_id: String,
    },
}

impl std::fmt::Display for ImportError {
    /// ⚠️ KHÔNG DẤU — chẩn đoán cho log (NFR16).
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ImportError::UnsupportedFormat { format } => {
                write!(f, "import: unsupported format {format:?}")
            }
            ImportError::UndecodableBytes { path, encoding } => {
                write!(f, "import[{path}]: undecodable bytes for encoding {encoding}")
            }
            ImportError::ReadFailed { path, detail } => {
                write!(f, "import[{path}]: read failed: {detail}")
            }
            ImportError::MissingExtension { path } => {
                write!(f, "import[{path}]: no file extension")
            }
            ImportError::TooLarge { size, limit } => {
                write!(f, "import: file is {size} bytes, limit is {limit}")
            }
            ImportError::InvalidPipelineOrder { detail } => {
                write!(f, "import: invalid pipeline order: {detail}")
            }
            ImportError::UnrecognizedEncoding { wire_id } => {
                write!(f, "import: unrecognized encoding wire id {wire_id:?}")
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
            ImportError::UndecodableBytes { path, encoding } => {
                let mut params = BTreeMap::new();
                params.insert("path".to_owned(), path);
                params.insert("encoding".to_owned(), encoding);
                IpcError::new(
                    "import.undecodable_bytes",
                    MessageKey::ImportUndecodableBytes,
                    params,
                    false,
                )
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
            ImportError::InvalidPipelineOrder { .. } => {
                // 🔴 KHÔNG BAO GIỜ chạm người dùng thật (xem doc-comment biến thể) —
                // `MessageKey::Unknown` là khoá dự phòng AD-21, đúng chỗ cho một lỗi
                // "không nên tồn tại" mà vẫn phải là kiểu, không phải một `panic!`.
                IpcError::new(
                    "import.invalid_pipeline_order",
                    MessageKey::Unknown,
                    BTreeMap::new(),
                    false,
                )
            }
            ImportError::UnrecognizedEncoding { wire_id } => {
                let mut params = BTreeMap::new();
                params.insert("encoding".to_owned(), wire_id);
                IpcError::new(
                    "import.unrecognized_encoding",
                    MessageKey::ImportUnrecognizedEncoding,
                    params,
                    false,
                )
            }
        }
    }
}

/// Kết quả của một lượt chạy chuỗi cho MỘT Chương — sẵn sàng ghi.
///
/// 🔵 **SỬA 2026-09-04 (Story 6.2) — "đúng một Chương" và "không mang segment" đã hết
/// đúng.** Kiểu này KHÔNG đổi hình dạng (vẫn đúng một `source_text` + segment của CHÍNH
/// Chương đó), nhưng chỗ dựng nó đổi: trước đây `import_text`/`import_file` tự dựng, giờ
/// [`super::pipeline::run_import`] dựng — MỘT lượt gọi giờ trả về `Vec<ImportedChapter>`
/// (N = 1 trên đường sản phẩm hôm nay, tổng quát hơn cho Story 6.6/6.7 sau này).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImportedChapter {
    /// Văn bản nguồn của Chương, sau khi chuỗi bảy bước AD-39 đã chạy hết.
    pub source_text: String,
    /// Segment cấp câu + cờ kết đoạn của CHÍNH Chương này — bước 7 của chuỗi
    /// ([`super::pipeline::Step::SplitSegments`], gọi [`super::split::split_source_text`]
    /// đã có), tính SẴN ở đây để `commands::project::create_work` chỉ còn việc GHI, không
    /// còn tự tính (AC13 không đổi: vẫn ghi cùng giao dịch với hàng `chapter`).
    pub segments: Vec<SplitSegment>,
}

/// Bước ĐẦU VÀO — nhánh dán văn bản của AC1. Trả về [`PipelineShape`], KHÔNG tự giải mã/
/// strip BOM (chuyển vào [`super::pipeline`], xem doc-comment đầu tệp) — văn bản dán tay
/// vốn đã LÀ `String` (không có bảng mã nào để mà giải), nên hình dạng đúng là
/// [`ChapterInput::AlreadyText`].
pub fn import_text(raw: String) -> PipelineShape {
    PipelineShape::Blob(ChapterInput::AlreadyText(raw))
}

/// Bước ĐẦU VÀO — nhánh tệp của AC1 (kéo-thả **hoặc** ô nhập đường dẫn — cả hai nhận một
/// đường dẫn thật, không phải nội dung tệp đã đọc sẵn từ webview, xem AD-1/AD-16).
///
/// Thứ tự: từ chối theo phần mở rộng **trước khi mở tệp** (không đọc một byte cho
/// `.docx`) → hỏi kích thước trước khi đọc → `std::fs::read` → trả [`PipelineShape`] mang
/// byte THÔ, CHƯA giải mã.
///
/// 🔵 **SỬA 2026-09-04 (Story 6.2) — hàm này KHÔNG còn tự giải mã.** Trước story này, bước
/// cuối là `String::from_utf8` nghiêm (Bẫy 8) rồi đổ vào `import_text`. Giải mã giờ là
/// [`super::pipeline::Step::DecodeEncoding`] — cùng phép giải mã NGHIÊM đó (không `_lossy`),
/// chỉ dời sang chuỗi để mọi nguồn (kể cả URL/song ngữ các story sau) đi qua ĐÚNG một chỗ.
/// Hệ quả quan sát được duy nhất: một tệp không hợp lệ với bảng mã đã khai giờ thất bại ở
/// [`super::pipeline::run_import`] (sau khi thư mục `.atproj` đã tạo) thay vì ngay ở hàm
/// này — `commands::project::create_work` cuộn lại TRỌN VẸN trên lỗi đó, cùng khuôn đã có
/// cho lỗi ghi `meta.json`, nên KHÔNG `.atproj` nào bị bỏ lại nửa vời (AC8 không đổi).
pub fn import_file(path: &Path) -> Result<PipelineShape, ImportError> {
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

    Ok(PipelineShape::Blob(ChapterInput::RawBytes {
        bytes,
        label: path.display().to_string(),
    }))
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
