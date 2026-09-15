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
//! - **Tách Chương** (FR14) — có mặt trong [`super::pipeline::PIPELINE_ORDER`]. 🔵 **SỬA
//!   2026-09-05 (Story 6.6) — "KHÔNG cấu hình được bởi người dùng" đã HẾT ĐÚNG.** Màn xem
//!   trước nhập nay cấu hình được một mẫu THẬT (literal hoặc regex,
//!   [`super::chapterpattern::ChapterPattern`]), và bước 5 tách theo VỊ TRÍ khớp — xem
//!   doc-comment `super::pipeline::split_chapters_step`.
//! - **Chuẩn hoá xuống dòng/khoảng trắng** (FR124/125) — 🔵 **SỬA 2026-09-04 (Story 6.4) —
//!   "vẫn THÂN RỖNG" đã HẾT ĐÚNG.** [`super::normalize::normalize`] nay chạy thật ở
//!   [`super::pipeline::Step::NormalizeParagraphsAndWhitespace`] (bước 4), TRƯỚC bước 7
//!   (tách segment) — xem doc-comment module đó.
//! - **Luật làm sạch** (FR124) — 🔵 **SỬA 2026-09-05 (Story 6.5) — "vẫn mở" đã HẾT ĐÚNG.**
//!   [`crate::core::cleanup::apply`] nay chạy thật ở
//!   [`super::pipeline::Step::CleanByRules`] (bước 3), TRƯỚC bước 4 (chuẩn hoá) — xem
//!   doc-comment module đó.
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

/// Ba phần mở rộng được nhận (FR13). 🔵 **SỬA 2026-09-09 (Story 6.12) — "`.docx` đóng ở Epic
/// 6" đã HẾT ĐÚNG.** `.docx` nay được nhận, qua [`crate::core::docx::read_docx`] (module
/// riêng, không qua `core::webimport/`) — xem nhánh `.docx` của [`import_file`]. Mọi thứ khác
/// vẫn đóng — xem AC8.
const SUPPORTED_EXTENSIONS: [&str; 3] = ["txt", "md", "docx"];

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
    /// Phần mở rộng chưa được nhận — bất kỳ thứ gì khác `.txt`/`.md`/`.docx`. 🔵 **SỬA
    /// 2026-09-09 (Story 6.12)** — `.docx` không còn ví dụ ở đây, nó rời sang nhánh chấp
    /// nhận của [`import_file`].
    UnsupportedFormat {
        /// Phần mở rộng đọc được (không có dấu chấm), thường thấy: `"epub"`, `"pdf"`. Rỗng
        /// nếu tệp không có phần mở rộng nào.
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
    /// **THÊM 2026-09-15 (Story 6.6b)** — [`import_files`] nhận `paths` RỖNG. Xem doc-comment
    /// [`crate::core::i18n::MessageKey::ImportEmptyFileList`].
    EmptyFileList,
    /// **THÊM 2026-09-15 (Story 6.6b)** — một tệp bên trong một batch N > 1 không phải
    /// `.txt`/`.md` (§Decisions "A batch is .txt/.md only"). Xem doc-comment
    /// [`crate::core::i18n::MessageKey::ImportBatchUnsupportedFormat`].
    BatchUnsupportedFormat {
        /// Phần mở rộng đọc được (không có dấu chấm).
        format: String,
    },
    /// Tệp vượt [`MAX_IMPORT_BYTES`].
    TooLarge {
        /// Kích thước thật, tính bằng byte.
        size: u64,
        /// Trần, tính bằng byte.
        limit: u64,
    },
    /// **THÊM 2026-09-09 (Story 6.12)** — `.docx` không đọc được: không phải zip hợp lệ (kể
    /// cả một tệp KHÁC đổi đuôi thành `.docx` — nhận theo NỘI DUNG, không theo tên), zip cắt
    /// cụt, hoặc `word/document.xml` hỏng XML. Đúng MỘT lỗi cho cả ba ca của Ma trận I/O
    /// spec 6.12 ("Tệp hỏng"/"Đuôi `.docx`, không phải zip") — người dùng không cần phân biệt
    /// LÝ DO kỹ thuật, chỉ cần biết tệp chưa nhập được.
    DocxUnreadable {
        /// Đường dẫn tệp — tham số `path` của [`MessageKey::DocxUnreadable`].
        path: String,
        /// Chẩn đoán [`crate::core::docx::DocxError`] — CHỈ cho log (KHÔNG DẤU, NFR16),
        /// không đi vào `IpcError`.
        detail: String,
    },
    /// **THÊM 2026-09-09 (Story 6.12)** — `.docx` đọc được nhưng 0 đoạn có chữ (Ma trận I/O
    /// "Rỗng"). ⚠️ Đo được: `.txt`/`.md` KHÔNG có một guard tương đương hôm nay (một tệp
    /// `.txt` rỗng đi trọn đường sản phẩm, tạo một Chương với `source_text` rỗng, 0 segment,
    /// không lỗi nào ném) — spec 6.12 gọi đây là "cùng khuôn tệp rỗng của .txt" nhưng khuôn
    /// đó chưa từng được cài; biến thể NÀY là guard THẬT ĐẦU TIÊN của dự án cho nội dung
    /// rỗng ở đường nhập tệp, chỉ áp cho `.docx`. Ghi ra thay vì để người sau đọc doc-comment
    /// và tưởng `.txt`/`.md` đã có cùng cơ chế.
    DocxEmptyText {
        /// Đường dẫn tệp.
        path: String,
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
    /// 🔵 **THÊM 2026-09-05 (Story 6.5)** — [`super::pipeline::Step::CleanByRules`] gọi
    /// [`crate::core::cleanup::apply`] và nhận [`crate::core::cleanup::CleanupError`]: một
    /// luật `regex` đã LƯU không biên dịch được. Không nên xảy ra trên đường sản phẩm —
    /// `core::cleanup::store::add_rule`/`edit_rule` biên dịch thử TRƯỚC khi ghi (§Always
    /// spec 6.5) — cùng lớp "lỗi lập trình không nên tồn tại" với
    /// [`ImportError::InvalidPipelineOrder`].
    InvalidCleanupPattern {
        /// Chẩn đoán CHỈ cho log (không đi vào `IpcError`, `Unknown` không nhận tham số).
        detail: String,
    },
    /// **THÊM 2026-09-05 (Story 6.6)** — mẫu phân tách Chương `kind = "regex"` không biên
    /// dịch được (`core::segment::chapterpattern::compile`). KHÁC
    /// [`ImportError::InvalidCleanupPattern`]/[`ImportError::InvalidPipelineOrder`]: đây LÀ
    /// một đường lỗi NGƯỜI DÙNG THẬT sự — mẫu phân tách không đi qua một bước lưu-trước-khi-
    /// dùng như luật làm sạch (nó là tham số MỖI LƯỢT NHẬP), nên một mẫu hỏng ĐẾN ĐƯỢC tới
    /// đây trên đường sản phẩm mỗi khi người dùng gõ một regex chưa đóng ngoặc. `commands::
    /// project` biên dịch thử NGAY khi nhận mẫu từ dây (trước khi gọi `run_pipeline`) nên
    /// biến thể này thường được ném TRƯỚC khi chạm `run_pipeline` — vẫn khai trong
    /// [`ImportError`] (không phải một kiểu lỗi riêng ở `commands::project`) vì nó là một
    /// cách "chuỗi nhập trượt", cùng họ với `InvalidCleanupPattern`.
    InvalidChapterPattern {
        /// Chẩn đoán cho log VÀ tham số `detail` của
        /// [`crate::core::i18n::MessageKey::ImportInvalidChapterPattern`] — KHÔNG DẤU
        /// (NFR16), vì `regex::Error::to_string()` là chẩn đoán máy, không phải câu người.
        detail: String,
    },
    /// **THÊM 2026-09-06 (Story 6.7)** — [`super::pipeline::Step::ExtractMainContent`]
    /// (bước 2, chỉ chạy thật khi `extract_main_content == true` — đường nhập URL) gọi
    /// [`crate::core::webimport::extract`] và nhận `Err`. Đây là ca DUY NHẤT của
    /// `ImportError` xảy ra TRÊN đường nhập URL sau khi TOÀN BỘ N link đã tải THÀNH CÔNG —
    /// một mục tải được nhưng bóc RA RỖNG (I/O Matrix spec 6.7: *"Trang bóc ra rỗng"*).
    ///
    /// ⚠️ Bảy trong tám lý do của [`crate::core::webimport::WebImportItemFailureReason`]
    /// (URL không hợp lệ, HTTP lỗi, timeout, không kết nối, chuyển hướng bị chặn, vượt trần,
    /// không phải HTML) xảy ra Ở TẦNG `commands::project` TRƯỚC KHI có gì để mà chạy
    /// `run_import` — biến thể NÀY chỉ phủ nhánh còn lại (`ExtractionEmpty`), nhánh duy nhất
    /// nằm THỰC SỰ bên trong chuỗi bảy bước.
    WebImportItemFailed {
        /// URL của mục thất bại — dữ liệu, không phải câu.
        url: String,
        /// Luôn là [`crate::core::webimport::WebImportItemFailureReason::ExtractionEmpty`]
        /// trên đường ĐI QUA BIẾN THỂ NÀY (xem cảnh báo ở trên) — giữ kiểu ĐẦY ĐỦ (không thu
        /// hẹp còn một `bool`) để `commands::project` dùng LẠI đúng một hàm ánh xạ
        /// `WebImportItemFailureReason -> IpcError` cho cả bảy lý do tầng ngoài LẪN lý do
        /// duy nhất tầng trong này — một nguồn ánh xạ, không hai.
        reason: crate::core::webimport::WebImportItemFailureReason,
        /// Chẩn đoán CHỈ cho log — không dấu (NFR16), không đi vào `IpcError`.
        detail: String,
    },
    /// **THÊM 2026-09-11 (Story 6.16, FR115)** — [`super::pipeline::Step::DecodeEncoding`]
    /// table-bang cho hình dạng [`super::pipeline::PipelineShape::Bilingual`] gặp một ô mở
    /// dấu ngoặc kép nhưng không bao giờ đóng ([`super::bilingual::BilingualParseIssue::UnterminatedQuotedField`]).
    /// Khác [`ImportError::InvalidPipelineOrder`]/[`ImportError::InvalidCleanupPattern`]:
    /// đây LÀ một lỗi NGƯỜI DÙNG thật (một tệp `.csv`/`.tsv` họ tự chọn), không phải một lỗi
    /// lập trình.
    BilingualUnterminatedQuotedField {
        /// Số dòng nơi ô mở ngoặc kép bắt đầu (1-based).
        row: usize,
    },
    /// **THÊM 2026-09-11 (Story 6.16, FR115)** — tệp song ngữ có ít hơn hai cột. Từ chối
    /// TRƯỚC khi có gì để xem trước (§I/O Matrix "Fewer than 2 columns").
    BilingualTooFewColumns {
        /// Số cột đếm được — hàng rộng nhất.
        found: usize,
    },
    /// **THÊM 2026-09-11 (Story 6.16, FR115)** — lượt xác nhận bị từ chối vì còn ít nhất một
    /// hàng lệch cặp (§Boundaries: "Mismatched row => confirm is locked", Rust-side, không
    /// chỉ ở nút webview).
    BilingualMismatchedRows {
        /// Số hàng lệch cặp còn lại.
        count: usize,
    },
    /// **THÊM 2026-09-11 (Story 6.16, FR115)** — đuôi tệp không phải `.csv`/`.tsv` trên
    /// đường nhập song ngữ. Biến thể RIÊNG với [`ImportError::UnsupportedFormat`] (không tái
    /// dùng nó): câu hiển thị đúng phải nêu đúng HAI đuôi được nhận ở đây, không phải ba đuôi
    /// của đường văn xuôi.
    BilingualUnsupportedFormat {
        /// Phần mở rộng đọc được (không có dấu chấm).
        format: String,
    },
    /// **THÊM 2026-09-12 (Story 6.17, FR116)** — webview gửi `Skip` cho một hàng mà CẢ HAI
    /// phía đều có ít nhất một câu. §Never: "No skip on a row whose two sides both have at
    /// least one sentence" — [`super::bilingual::resolve`] trả `Err` thay vì âm thầm để hàng
    /// nằm lại danh sách lệch cặp, vì đây là một vi phạm hợp đồng của webview, không một
    /// trạng thái người dùng có thể tạo ra qua đường sản phẩm (nút chỉ hiện khi một phía = 0).
    BilingualSkipNotAllowed {
        /// Số hàng file (1-based) đã bị từ chối.
        row_number: usize,
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
            ImportError::EmptyFileList => {
                write!(f, "import: empty file list")
            }
            ImportError::BatchUnsupportedFormat { format } => {
                write!(f, "import[batch]: unsupported format {format:?}")
            }
            ImportError::TooLarge { size, limit } => {
                write!(f, "import: file is {size} bytes, limit is {limit}")
            }
            ImportError::DocxUnreadable { path, detail } => {
                write!(f, "import[{path}]: docx unreadable: {detail}")
            }
            ImportError::DocxEmptyText { path } => {
                write!(f, "import[{path}]: docx has 0 paragraphs with text")
            }
            ImportError::InvalidPipelineOrder { detail } => {
                write!(f, "import: invalid pipeline order: {detail}")
            }
            ImportError::UnrecognizedEncoding { wire_id } => {
                write!(f, "import: unrecognized encoding wire id {wire_id:?}")
            }
            ImportError::InvalidCleanupPattern { detail } => {
                write!(f, "import: invalid cleanup pattern: {detail}")
            }
            ImportError::InvalidChapterPattern { detail } => {
                write!(f, "import: invalid chapter pattern: {detail}")
            }
            ImportError::WebImportItemFailed { url, reason, detail } => {
                write!(f, "import[webimport {url}]: {reason:?}: {detail}")
            }
            ImportError::BilingualUnterminatedQuotedField { row } => {
                write!(f, "import[bilingual row {row}]: quoted field opened but never closed")
            }
            ImportError::BilingualTooFewColumns { found } => {
                write!(f, "import[bilingual]: {found} column(s) found, need at least 2")
            }
            ImportError::BilingualMismatchedRows { count } => {
                write!(f, "import[bilingual]: {count} mismatched row(s), confirm refused")
            }
            ImportError::BilingualUnsupportedFormat { format } => {
                write!(f, "import[bilingual]: unsupported format {format:?}")
            }
            ImportError::BilingualSkipNotAllowed { row_number } => {
                write!(f, "import[bilingual row {row_number}]: skip not allowed, both sides have sentences")
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
            ImportError::EmptyFileList => IpcError::new(
                "import.empty_file_list",
                MessageKey::ImportEmptyFileList,
                BTreeMap::new(),
                false,
            ),
            ImportError::BatchUnsupportedFormat { format } => {
                let mut params = BTreeMap::new();
                params.insert("format".to_owned(), format);
                IpcError::new(
                    "import.batch_unsupported_format",
                    MessageKey::ImportBatchUnsupportedFormat,
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
            ImportError::DocxUnreadable { path, detail } => {
                eprintln!("import[{path}] docx khong doc duoc: {detail}");
                let mut params = BTreeMap::new();
                params.insert("path".to_owned(), path);
                IpcError::new("docx.unreadable", MessageKey::DocxUnreadable, params, false)
            }
            ImportError::DocxEmptyText { path } => {
                let mut params = BTreeMap::new();
                params.insert("path".to_owned(), path);
                IpcError::new("docx.empty_text", MessageKey::DocxEmptyText, params, false)
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
            ImportError::InvalidCleanupPattern { .. } => {
                // 🔴 KHÔNG BAO GIỜ chạm người dùng thật (xem doc-comment biến thể) — cùng
                // khuôn `InvalidPipelineOrder`.
                IpcError::new(
                    "import.invalid_cleanup_pattern",
                    MessageKey::Unknown,
                    BTreeMap::new(),
                    false,
                )
            }
            ImportError::InvalidChapterPattern { detail } => {
                // 🔴 KHÁC hai nhánh ngay trên — ĐÂY LÀ một lỗi NGƯỜI DÙNG THẬT (xem
                // doc-comment biến thể), nên khoá là `ImportInvalidChapterPattern` THẬT,
                // KHÔNG `Unknown`. `detail` (chẩn đoán máy của `regex::Error`) chỉ đi vào
                // log, KHÔNG vào `params` — cùng luật `CleanupInvalidRegex` (0 tham số):
                // câu hiển thị không cần lặp lại lỗi cú pháp regex thô cho người dùng.
                eprintln!("import[chapter_pattern] mau khong bien dich duoc: {detail}");
                IpcError::new(
                    "import.invalid_chapter_pattern",
                    MessageKey::ImportInvalidChapterPattern,
                    BTreeMap::new(),
                    false,
                )
            }
            ImportError::WebImportItemFailed { url, reason, detail } => {
                eprintln!("import[webimport {url}]: {reason:?}: {detail}");
                // `ExtractionEmpty` — nhánh DUY NHẤT `ImportError` mang, không có mã HTTP.
                web_import_item_failure_ipc_error(&url, reason, None)
            }
            ImportError::BilingualUnterminatedQuotedField { row } => {
                let mut params = BTreeMap::new();
                params.insert("row".to_owned(), row.to_string());
                IpcError::new(
                    "import.bilingual_unterminated_quoted_field",
                    MessageKey::ImportBilingualUnterminatedQuotedField,
                    params,
                    false,
                )
            }
            ImportError::BilingualTooFewColumns { found } => {
                let mut params = BTreeMap::new();
                params.insert("found".to_owned(), found.to_string());
                IpcError::new(
                    "import.bilingual_too_few_columns",
                    MessageKey::ImportBilingualTooFewColumns,
                    params,
                    false,
                )
            }
            ImportError::BilingualMismatchedRows { count } => {
                let mut params = BTreeMap::new();
                params.insert("count".to_owned(), count.to_string());
                IpcError::new(
                    "import.bilingual_mismatched_rows",
                    MessageKey::ImportBilingualMismatchedRows,
                    params,
                    false,
                )
            }
            ImportError::BilingualUnsupportedFormat { format } => {
                let mut params = BTreeMap::new();
                params.insert("format".to_owned(), format);
                IpcError::new(
                    "import.bilingual_unsupported_format",
                    MessageKey::ImportBilingualUnsupportedFormat,
                    params,
                    false,
                )
            }
            ImportError::BilingualSkipNotAllowed { row_number } => {
                let mut params = BTreeMap::new();
                params.insert("row".to_owned(), row_number.to_string());
                IpcError::new(
                    "import.bilingual_skip_not_allowed",
                    MessageKey::ImportBilingualSkipNotAllowed,
                    params,
                    false,
                )
            }
        }
    }
}

/// Ánh xạ DUY NHẤT `WebImportItemFailureReason -> IpcError` — dùng CHUNG bởi
/// `From<ImportError>` ở trên (nhánh `ExtractionEmpty`, ném từ TRONG chuỗi pipeline) VÀ
/// `commands::project` (bảy nhánh còn lại, xảy ra TRƯỚC khi có gì để mà chạy pipeline — một
/// URL rác, một mã lỗi HTTP, một chuyển hướng bị chặn, … không đi qua `ImportError` vì
/// `run_import` chưa từng được gọi cho mục đó). Một nguồn ánh xạ, không hai — tám khoá
/// `err.import.web_*` (`core::i18n`) và tên biến thể ở đây phải khớp nhau 1:1
/// (`ipc_contract.rs` khoá cả tám). `http_status` chỉ có nghĩa cho
/// [`crate::core::webimport::WebImportItemFailureReason::HttpStatus`] — bỏ qua ở bảy nhánh
/// còn lại.
pub fn web_import_item_failure_ipc_error(
    url: &str,
    reason: crate::core::webimport::WebImportItemFailureReason,
    http_status: Option<u16>,
) -> IpcError {
    use crate::core::webimport::WebImportItemFailureReason as Reason;

    let mut params = BTreeMap::new();
    params.insert("url".to_owned(), url.to_owned());

    let key = match reason {
        Reason::InvalidUrl => MessageKey::ImportWebInvalidUrl,
        Reason::HttpStatus => {
            params.insert("status".to_owned(), http_status.map_or_else(|| "?".to_owned(), |s| s.to_string()));
            MessageKey::ImportWebHttpStatus
        }
        Reason::Timeout => MessageKey::ImportWebTimeout,
        Reason::ConnectFailed => MessageKey::ImportWebConnectFailed,
        Reason::RedirectBlocked => MessageKey::ImportWebRedirectBlocked,
        Reason::TooLarge => MessageKey::ImportWebTooLarge,
        Reason::NotHtml => MessageKey::ImportWebNotHtml,
        Reason::ExtractionEmpty => MessageKey::ImportWebExtractionEmpty,
    };
    IpcError::new("import.web_item_failed", key, params, false)
}

/// Kết quả của một lượt chạy chuỗi cho MỘT Chương — sẵn sàng ghi.
///
/// 🔵 **SỬA 2026-09-04 (Story 6.2) — "đúng một Chương" và "không mang segment" đã hết
/// đúng.** Kiểu này KHÔNG đổi hình dạng (vẫn đúng một `source_text` + segment của CHÍNH
/// Chương đó), nhưng chỗ dựng nó đổi: trước đây `import_text`/`import_file` tự dựng, giờ
/// [`super::pipeline::run_import`] dựng — MỘT lượt gọi trả về `Vec<ImportedChapter>`. 🔵
/// **SỬA 2026-09-05 (Story 6.6)** — "N = 1 trên đường sản phẩm hôm nay" đã HẾT ĐÚNG: mẫu
/// phân tách Chương (`PipelineInput::chapter_pattern`) là một bề mặt SẢN PHẨM thật từ story
/// này, và N > 1 là một kết quả THẬT khi mẫu khớp được nhiều lần.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImportedChapter {
    /// Văn bản nguồn của Chương, sau khi chuỗi bảy bước AD-39 đã chạy hết.
    pub source_text: String,
    /// Segment cấp câu + cờ kết đoạn của CHÍNH Chương này — bước 7 của chuỗi
    /// ([`super::pipeline::Step::SplitSegments`], gọi [`super::split::split_source_text`]
    /// đã có), tính SẴN ở đây để `commands::project::create_work` chỉ còn việc GHI, không
    /// còn tự tính (AC13 không đổi: vẫn ghi cùng giao dịch với hàng `chapter`).
    pub segments: Vec<SplitSegment>,
    /// **THÊM 2026-09-05 (Story 6.5)** — báo cáo của [`super::pipeline::Step::CleanByRules`]
    /// (bước 3) cho CHÍNH Chương này. `None` khi bước 3 không tạo được báo cáo cho đơn vị
    /// này — nhánh `Unit::Undecoded` bất khả trên mọi thứ tự HỢP LỆ (cùng lý do
    /// `Unit::Undecoded` bất khả ở bước 4, xem `pipeline.rs`), hoặc khi
    /// [`super::pipeline::Step::SplitChapters`] tách MỘT đơn vị thành N sau khi bước 3 đã
    /// chạy. 🔵 **SỬA 2026-09-05 (Story 6.6)** — "chỉ `tests/**` khai `chapter_pattern:
    /// Some(..)`" đã HẾT ĐÚNG: màn xem trước nhập là một bề mặt SẢN PHẨM có thể khai mẫu
    /// thật ⇒ N > 1 và `cleanup_report` reset về `None` cho mọi Chương kết quả là một
    /// đường THẬT trên sản phẩm, không chỉ trong test — xem
    /// `commands::project::cleanup_preview_for` cho cách tầng gọi bù lại số liệu tổng.
    pub cleanup_report: Option<crate::core::cleanup::CleanupReport>,
    /// **THÊM 2026-09-05 (Story 6.6)** — dòng tiêu đề của Chương này, đọc từ CHÍNH dòng khớp
    /// mẫu phân tách (`pipeline::title_line_of`), TRIM hai đầu. `None` khi: mẫu không được
    /// cấu hình, mẫu không khớp gì (một Chương duy nhất), hoặc đây là Chương lời tựa (phần
    /// văn bản TRƯỚC khớp đầu tiên — §Always spec 6.6: không bao giờ vứt, luôn hiện ra).
    pub title: Option<String>,
    /// **THÊM 2026-09-07 (Story 6.9)** — mô hình khối CẢ TRANG mà
    /// [`super::pipeline::Step::ExtractMainContent`] vừa dựng cho Chương này, DỮ LIỆU THÔ
    /// (`Block::machine_kept`, CHƯA áp `PipelineInput::block_overrides` — bước đó chỉ dùng
    /// override để GHÉP `source_text`, không sửa lại chính mô hình). `None` khi bước 2 không
    /// chạy cho Chương này (`extract_main_content == false` — đường tệp/dán tay, mọi Chương
    /// SAU Chương đầu tiên của đường URL cũng mang `Some` — §Never spec 6.9 chỉ giới hạn
    /// TẦNG HIỂN THỊ ở Chương đầu, không giới hạn dữ liệu domain này). `commands::project`
    /// dựng dây tầng 2 từ trường này, đọc lại CÙNG override đã dùng để ghép `source_text`.
    pub blocks: Option<Vec<crate::core::webimport::Block>>,
    /// **THÊM 2026-09-08 (Story 6.10)** — số LẦN [`super::pipeline::Step::NormalizeParagraphsAndWhitespace`]
    /// (bước 4) đã NỐI hai dòng làm một, cho CHÍNH Chương này — con số FR125 đã tính rồi bị
    /// vứt trước bản sửa này (`pipeline.rs`, xem doc-comment `super::pipeline::Flow::joined_line_counts`
    /// cho cơ chế đầy đủ). `None` = *không đo được cho Chương này*, KHÔNG BAO GIỜ mặc định hoá
    /// thành `0` — trên [`super::pipeline::PipelineShape::Blob`] con số đo được TRƯỚC khi tách
    /// Chương thuộc về TOÀN TÀI LIỆU, không quy về Chương nào được (kể cả `ord = 1`); trên
    /// [`super::pipeline::PipelineShape::Chapters`] (đã chia Chương từ đầu) con số của mỗi
    /// Chương là THẬT.
    pub joined_line_count: Option<usize>,
    /// **THÊM 2026-09-10 (Story 6.15)** — xuất xứ tài liệu (FR128/AD-43) mà
    /// [`super::pipeline::Step::ExtractMainContent`] vừa bóc cho Chương này (ba trường từ
    /// HTML cộng URL yêu cầu, echo từ `label` — xem doc-comment
    /// [`crate::core::webimport::origin`]). `None` khi bước 2 không chạy cho Chương này
    /// (`extract_main_content == false` — đường tệp/dán tay). Một trang tách thành nhiều
    /// Chương (mẫu phân tách khớp N lần) ⇒ MỌI Chương con mang CÙNG giá trị — xem doc-comment
    /// `super::pipeline::Flow::origins`.
    pub origin: Option<crate::core::webimport::ChapterOrigin>,
    /// **THÊM 2026-09-11 (Story 6.16, FR115)** — segment ĐÃ CẶP nguồn/đích của Chương này,
    /// trên đường nhập song ngữ. `Some` chỉ khi hình dạng đầu vào là
    /// [`super::pipeline::PipelineShape::Bilingual`]; `None` cho MỌI đường khác (không đổi
    /// hành vi — §Always spec 6.16: đường nhập văn xuôi `.txt`/`.md`/`.docx` byte-identical).
    /// Chỉ mang hàng CẶP ĐƯỢC (số câu nguồn = số câu đích) — một hàng lệch cặp không đóng
    /// góp segment nào vào đây, nó chỉ có mặt trong [`super::pipeline::PipelineOutput::bilingual_mismatches`].
    pub bilingual_segments: Option<Vec<super::bilingual::BilingualSegment>>,
    /// **THÊM 2026-09-15 (Story 6.6b)** — tên/đường dẫn tệp NGUỒN của Chương này, đọc từ
    /// [`super::pipeline::Flow::labels`] ở cuối [`super::pipeline::run_import_with_order`].
    /// `None` khi nhãn rỗng — đúng MỌI hình dạng KHÁC [`super::pipeline::PipelineShape::Files`]
    /// hôm nay (`Blob`/`Bilingual` chưa từng mang một nhãn có nghĩa; `Chapters`/URL mang nhãn
    /// nhưng chỉ dùng NỘI BỘ cho `Step::ExtractMainContent`, không lộ ra tầng hiển thị qua
    /// trường này — xem `commands::project::chapter_input_page_url` cho đường ĐÓ). Trên
    /// [`super::pipeline::PipelineShape::Files`], `Some(path)` cho MỌI Chương — kể cả khi một
    /// tệp bị mẫu phân tách cắt thành k > 1 mảnh (mọi mảnh của CÙNG một tệp mang CÙNG
    /// `source_file`, §Always spec 6.6b: "pieces keep unit i's label").
    pub source_file: Option<String>,
}

/// Bước ĐẦU VÀO — nhánh dán văn bản của AC1. Trả về [`PipelineShape`], KHÔNG tự giải mã/
/// strip BOM (chuyển vào [`super::pipeline`], xem doc-comment đầu tệp) — văn bản dán tay
/// vốn đã LÀ `String` (không có bảng mã nào để mà giải), nên hình dạng đúng là
/// [`ChapterInput::AlreadyText`].
pub fn import_text(raw: String) -> PipelineShape {
    PipelineShape::Blob(ChapterInput::AlreadyText(raw))
}

/// **THÊM 2026-09-09 (Story 6.12)** — phần đi kèm mà [`import_file`] trả cho `.docx`, mang
/// dữ liệu mà chuỗi bảy bước AD-39 KHÔNG có chỗ chở (nó chỉ mang `String`/byte thô, không
/// mang khối/ảnh) nhưng pha ảnh Story 6.11 (`commands::project::prepare_chapter_images`) cần
/// để nối đúng ảnh nhúng vào đúng vị trí. `None` cho mọi phần mở rộng khác — không byte nào
/// bị đọc thừa, không cấu trúc nào bị dựng thừa cho `.txt`/`.md`.
///
/// 🔴 **Chỉ Chương ĐẦU TIÊN đọc trường này** — cùng giới hạn mà
/// [`super::pipeline::PipelineInput::block_overrides`] đã theo (Story 6.9): một mẫu phân
/// tách Chương (Story 6.6) áp lên văn bản `.docx` (cạnh hiếm, ngoài Ma trận I/O spec 6.12)
/// làm N > 1 Chương, và chỉ Chương `ord = 1` có `blocks`/ảnh — nợ có chủ nếu Ice cần mở rộng.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DocxSidecar {
    /// Đoạn + ảnh của TOÀN tài liệu, cùng hình dạng mà `core::webimport::Extractor` dùng cho
    /// HTML — gắn vào `ImportedChapter::blocks` của Chương đầu tiên sau khi `run_pipeline`
    /// chạy xong (`commands::project::create_work`), để `core::segment::anchor::compute_anchor`
    /// tái dùng nguyên vẹn.
    pub blocks: Vec<crate::core::webimport::Block>,
    /// Byte thật của mọi ảnh nhúng phân giải được, theo chỉ số khối trong `blocks`.
    pub images: Vec<crate::core::docx::DocxImage>,
}

/// Bước ĐẦU VÀO — nhánh tệp của AC1 (kéo-thả **hoặc** ô nhập đường dẫn — cả hai nhận một
/// đường dẫn thật, không phải nội dung tệp đã đọc sẵn từ webview, xem AD-1/AD-16).
///
/// Thứ tự: từ chối theo phần mở rộng **trước khi mở tệp** → hỏi kích thước trước khi đọc →
/// `std::fs::read` → trả [`PipelineShape`] mang byte THÔ CHƯA giải mã (`.txt`/`.md`) HOẶC
/// văn bản ĐÃ giải mã cộng [`DocxSidecar`] (`.docx` — Story 6.12).
///
/// 🔵 **SỬA 2026-09-04 (Story 6.2) — hàm này KHÔNG còn tự giải mã (`.txt`/`.md`).** Giải mã
/// của hai định dạng đó vẫn là [`super::pipeline::Step::DecodeEncoding`] — xem doc-comment cũ
/// giữ nguyên bên dưới cho lý do. `.docx` KHÔNG đi qua bước đó: nó **tự khai bảng mã** (OOXML
/// luôn UTF-8 trong `word/document.xml`), nên hình dạng đúng là
/// [`ChapterInput::AlreadyText`] — cùng hộc với văn bản dán tay (§Design Notes spec 6.12).
///
/// 🔵 **SỬA 2026-09-09 (Story 6.12) — chữ ký đổi từ `Result<PipelineShape, _>` sang
/// `Result<(PipelineShape, Option<DocxSidecar>), _>`.** Đây là thay đổi DUY NHẤT của story
/// này lên chữ ký hàm thuần đã có — mọi chỗ gọi (`commands::project::create_work_from_file`,
/// `wire::preview_import_encoding_from_file`, `tests/project_contract.rs`) đọc `.0` khi không
/// cần sidecar.
pub fn import_file(path: &Path) -> Result<(PipelineShape, Option<DocxSidecar>), ImportError> {
    reject_unsupported_extension(path)?;

    // 🔴 Hỏi KÍCH THƯỚC trước khi đọc — không đọc rồi mới đo. `metadata` là một lượt
    // `stat`, không nạp một byte nội dung nào; đo sau khi `fs::read` thì bộ nhớ đã cạn
    // xong rồi mới biết. Xem [`MAX_IMPORT_BYTES`]. Áp CHO CẢ `.docx` (Ma trận I/O spec 6.12
    // "Quá cỡ": từ chối TRƯỚC khi đọc zip).
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

    // `reject_unsupported_extension` ở trên đã xác nhận phần mở rộng nằm trong
    // `SUPPORTED_EXTENSIONS` — đọc lại một lần nữa ở đây (không tái dùng một biến đã tính)
    // vì nhánh `.docx` cần chính giá trị hạ chữ thường này.
    let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("").to_ascii_lowercase();

    if ext == "docx" {
        let path_display = path.display().to_string();
        let parsed = crate::core::docx::read_docx(&bytes).map_err(|e| match e {
            crate::core::docx::DocxError::EmptyText => {
                ImportError::DocxEmptyText { path: path_display.clone() }
            }
            other => ImportError::DocxUnreadable { path: path_display.clone(), detail: other.to_string() },
        })?;
        let sidecar = DocxSidecar { blocks: parsed.blocks, images: parsed.images };
        return Ok((PipelineShape::Blob(ChapterInput::AlreadyText(parsed.text)), Some(sidecar)));
    }

    Ok((
        PipelineShape::Blob(ChapterInput::RawBytes {
            bytes,
            label: path.display().to_string(),
        }),
        None,
    ))
}

/// **THÊM 2026-09-15 (Story 6.6b)** — kết quả BỌC của [`import_files`]: hình dạng để nạp
/// thẳng vào chuỗi ([`PipelineShape::Blob`] khi N = 1, [`PipelineShape::Files`] khi N > 1),
/// `DocxSidecar` (chỉ CÓ THỂ `Some` khi N = 1 — §Decisions "A batch is .txt/.md only", nên
/// N > 1 luôn `None`), cộng một mục PER-ITEM cho từng đường dẫn trong `paths` — theo ĐÚNG thứ
/// tự đã gửi, kể cả khi N = 1 (§Always: "the file wire's envelope returns the per-item batch
/// shape for every N, N = 1 included").
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FilesImportOutcome {
    pub shape: PipelineShape,
    pub docx_sidecar: Option<DocxSidecar>,
    pub items: Vec<FileImportItem>,
}

/// Một mục của [`FilesImportOutcome::items`] — copy hình dạng của mục danh sách URL
/// (`commands::project::UrlImportItem`, `mod.rs:3963-3981` của Code Map spec 6.6b): `path`
/// GIỮ ĐÚNG vị trí trong `paths` dù thành công hay không, `error` mang [`ImportError`] ĐẦY ĐỦ
/// khi mục này thất bại — asymmetry với URL: một tệp CÓ THỂ đọc lại (không cấm re-fetch như
/// URL), nên không có trường `raw`/byte đã đọc để mà giữ ở đây; `import_files` gọi lại từ đầu
/// mỗi lượt.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileImportItem {
    pub path: String,
    pub error: Option<ImportError>,
}

/// **THÊM 2026-09-15 (Story 6.6b)** — nhánh N tệp của AC1: đường DUY NHẤT
/// `wire::preview_import_encoding_from_file` gọi, dù `paths` mang một hay nhiều tệp
/// (§Always: "one shape to reason about"). N = 1 uỷ quyền THẲNG cho [`import_file`] (giữ
/// nguyên `Blob` cộng `DocxSidecar` — hành vi đơn-tệp KHÔNG đổi một byte, §Always spec 6.6b),
/// và một lỗi ở N = 1 vẫn là `Err` TOÀN CỤC như hôm nay (không có "phần còn lại" nào để mà giữ
/// khi chỉ có một tệp). N > 1: phần mở rộng bị từ chối TRƯỚC khi mở tệp (tái dùng
/// [`reject_unsupported_extension`] qua [`import_file`], cùng thứ tự AC8 đã theo), MỘT
/// `.docx`/`.csv`/`.tsv` bên trong batch trở thành một mục lỗi RIÊNG
/// ([`ImportError::BatchUnsupportedFormat`], §Decisions), và một lần đọc/kích thước trượt
/// ([`ImportError::ReadFailed`]/[`ImportError::TooLarge`]) cũng trở thành một mục lỗi —
/// KHÔNG BAO GIỜ một `Err` làm hỏng cả lượt (§Decisions "An unreadable file keeps its place,
/// locks confirm, and can be removed").
///
/// Danh sách RỖNG bị từ chối TRƯỚC KHI mở bất kỳ tệp nào (I/O Matrix "Empty list") —
/// [`ImportError::EmptyFileList`], không phải một `PipelineShape::Files(vec![])` im lặng.
pub fn import_files(paths: &[String]) -> Result<FilesImportOutcome, ImportError> {
    if paths.is_empty() {
        return Err(ImportError::EmptyFileList);
    }

    if paths.len() == 1 {
        let (shape, docx_sidecar) = import_file(Path::new(&paths[0]))?;
        return Ok(FilesImportOutcome {
            shape,
            docx_sidecar,
            items: vec![FileImportItem { path: paths[0].clone(), error: None }],
        });
    }

    let mut inputs: Vec<ChapterInput> = Vec::with_capacity(paths.len());
    let mut items: Vec<FileImportItem> = Vec::with_capacity(paths.len());
    for p in paths {
        let path = Path::new(p);
        if let Err(e) = reject_batch_unsupported_extension(path) {
            items.push(FileImportItem { path: p.clone(), error: Some(e) });
            continue;
        }
        match import_file(path) {
            // `reject_batch_unsupported_extension` above already narrowed the extension to
            // `.txt`/`.md` — `import_file` on either of those two always returns `Blob`
            // carrying `RawBytes` (it only ever produces `AlreadyText` for `.docx`, refused
            // above), never a `DocxSidecar`.
            Ok((PipelineShape::Blob(input), _)) => {
                inputs.push(input);
                items.push(FileImportItem { path: p.clone(), error: None });
            }
            Ok((PipelineShape::Chapters(_) | PipelineShape::Bilingual { .. } | PipelineShape::Files(_), _)) => {
                unreachable!("import_file only ever returns PipelineShape::Blob")
            }
            Err(e) => {
                items.push(FileImportItem { path: p.clone(), error: Some(e) });
            }
        }
    }

    Ok(FilesImportOutcome { shape: PipelineShape::Files(inputs), docx_sidecar: None, items })
}

/// Từ chối một phần mở rộng KHÔNG phải `.txt`/`.md` trên đường N > 1 tệp — RIÊNG với
/// [`reject_unsupported_extension`] (đó vẫn từ chối/chấp nhận đúng ba đuôi của đường MỘT tệp,
/// KHÔNG đụng — §Always spec 6.6b: "Single-file .docx ... untouched"). `.docx` tự nó là một
/// đuôi HỢP LỆ của [`import_file`] — không chặn nó Ở ĐÂY thì `import_file` sẽ ĐỌC nó thành
/// công và nuốt mất chính chỗ hở mà §Decisions "A batch is .txt/.md only" đóng (một `.docx`
/// trong batch cần một `DocxSidecar` PER-UNIT, ngoài phạm vi story — xem `deferred-work.md`).
/// Không phần mở rộng nào ⇒ [`ImportError::MissingExtension`] (tái dùng, không một hạng lỗi
/// thứ hai cho cùng một sự thật).
fn reject_batch_unsupported_extension(path: &Path) -> Result<(), ImportError> {
    let Some(ext) = path.extension().and_then(|e| e.to_str()) else {
        return Err(ImportError::MissingExtension { path: path.display().to_string() });
    };
    let ext = ext.to_ascii_lowercase();
    if ext == "txt" || ext == "md" {
        return Ok(());
    }
    Err(ImportError::BatchUnsupportedFormat { format: ext })
}

/// **THÊM 2026-09-11 (Story 6.16, FR115)** — hai đuôi được nhận trên đường nhập song ngữ,
/// RIÊNG với [`SUPPORTED_EXTENSIONS`] (§Boundaries: "Scope: .csv and .tsv only"; `.md`/
/// `.docx` bảng biểu bị hoãn, chủ Ice, `deferred-work.md`).
const BILINGUAL_SUPPORTED_EXTENSIONS: [&str; 2] = ["csv", "tsv"];

/// Bước ĐẦU VÀO — nhánh song ngữ của FR115 (Story 6.16). Cùng khuôn [`import_file`]: từ chối
/// theo phần mở rộng TRƯỚC khi mở tệp, hỏi kích thước TRƯỚC khi đọc, `std::fs::read` một
/// lần — 0 chuỗi nào ghi xuống đĩa (§Boundaries: "0 bytes on disk before confirm").
///
/// 🔴 Trả về byte THÔ CHƯA giải mã, KHÔNG tự table-parse ở đây — cả giải mã LẪN table-parse
/// là việc của [`super::pipeline::Step::DecodeEncoding`] (AD-39: *"table parsing happens
/// right after decode, inside the chain, and re-runs on every encoding candidate"*). Hàm
/// này chỉ quyết định đúng MỘT điều mà table-parse không tự suy được: dấu phân cách, từ
/// CHÍNH đuôi tệp (`.csv` ⇒ dấu phẩy, `.tsv` ⇒ Tab — §Always: "Delimiter from the
/// extension").
pub fn import_bilingual_file(path: &Path) -> Result<super::pipeline::PipelineShape, ImportError> {
    let Some(ext) = path.extension().and_then(|e| e.to_str()) else {
        return Err(ImportError::MissingExtension { path: path.display().to_string() });
    };
    let ext = ext.to_ascii_lowercase();

    if !BILINGUAL_SUPPORTED_EXTENSIONS.contains(&ext.as_str()) {
        return Err(ImportError::BilingualUnsupportedFormat { format: ext });
    }
    let delimiter = match ext.as_str() {
        "csv" => crate::core::glossary::exchange::Delimiter::Csv,
        // `BILINGUAL_SUPPORTED_EXTENSIONS` has exactly two entries; the guard above already
        // rejected everything else.
        _ => crate::core::glossary::exchange::Delimiter::Tsv,
    };

    let size = std::fs::metadata(path)
        .map_err(|e| ImportError::ReadFailed {
            path: path.display().to_string(),
            detail: e.to_string(),
        })?
        .len();
    if size > MAX_IMPORT_BYTES {
        return Err(ImportError::TooLarge { size, limit: MAX_IMPORT_BYTES });
    }

    let bytes = std::fs::read(path).map_err(|e| ImportError::ReadFailed {
        path: path.display().to_string(),
        detail: e.to_string(),
    })?;

    Ok(super::pipeline::PipelineShape::Bilingual {
        input: ChapterInput::RawBytes { bytes, label: path.display().to_string() },
        delimiter,
    })
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
