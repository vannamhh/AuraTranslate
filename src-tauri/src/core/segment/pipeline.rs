//! Chuỗi pipeline nhập — BẢY BƯỚC, thứ tự CỐ ĐỊNH, dùng CHUNG mọi nguồn (AD-39, Story 6.2).
//!
//! ─────────────────────────────────────────────────────────────────────────────
//! 🔴 THỨ TỰ LÀ DỮ LIỆU, KHÔNG PHẢI HÌNH DẠNG MỘT HÀM — đây là toàn bộ điểm của module này
//! ─────────────────────────────────────────────────────────────────────────────
//! AD-39 khai bảy bước theo một thứ tự cố định (spine `:473-482`):
//!
//! ```text
//! 1. giải mã bảng mã
//! 2. bóc nội dung chính
//! 3. làm sạch theo luật
//! 4. chuẩn hoá đoạn & khoảng trắng
//! 5. tách Chương theo mẫu phân tách
//! 6. xem trước + sửa tay
//! 7. tách segment + cờ kết đoạn
//! ```
//!
//! (Bước 8 — ghi `.atproj` — đứng NGOÀI hằng [`PIPELINE_ORDER`] và mọi mảnh mã của module
//! này; xem §Design Notes "Vì sao chuỗi dừng ở bước 7" của spec 6.2.)
//!
//! Nếu thứ tự này nằm cứng trong thân một hàm (`decode(...); extract(...); clean(...); …`),
//! *"đặt sai thứ tự"* không biểu diễn được lúc chạy — đối chứng cho một thứ tự sai chỉ còn là
//! một phép quét CHỮ trên mã nguồn. [`PIPELINE_ORDER`] là một GIÁ TRỊ (`[Step; 7]`), và
//! [`run_import_with_order`] tiêu thụ CHÍNH giá trị đó — nên một test dựng tay một hoán vị
//! khác rồi cho chạy qua ĐÚNG bộ chạy sản phẩm là chuyện làm được ngay hôm nay, không cần chờ
//! Story 6.3 (dò bảng mã) hay Story 6.6 (mẫu phân tách người dùng cấu hình được).
//!
//! ─────────────────────────────────────────────────────────────────────────────
//! 🔵 SỬA (vòng rà đối kháng 2026-09-04) — `order` giờ được KIỂM là một hoán vị hợp lệ
//! ─────────────────────────────────────────────────────────────────────────────
//! Bản đầu nhận `&[Step]` bất kỳ mà không kiểm gì: một bước TRÙNG chạy hai lần (vô hại cho
//! `DecodeEncoding`/BOM — idempotent — nhưng KHÔNG vô hại nói chung), một bước THIẾU để lại
//! trạng thái dở (thiếu `DecodeEncoding` ⇒ văn bản hỏng cộng 0 segment mà KHÔNG lỗi nào ném
//! — đúng lớp lỗi "rỗng/hỏng im lặng" mà AGENTS.md gọi tên là trung tâm của dự án), và một
//! mảng RỖNG chạy trọn mà không làm gì. [`validate_order`] chạy TRƯỚC bước nào, một lần, ở
//! đầu [`run_import_with_order`] — xem doc-comment của nó.
//!
//! ─────────────────────────────────────────────────────────────────────────────
//! 🔴 BA BƯỚC THÂN RỖNG — CÓ CHỦ Ý, MỖI BƯỚC MỘT STORY CHỦ (§Never của spec 6.2)
//! ─────────────────────────────────────────────────────────────────────────────
//! 🔵 **SỬA 2026-09-04 (Story 6.4)** — từ "BỐN bước" xuống "BA bước": [`Step::NormalizeParagraphsAndWhitespace`]
//! không còn no-op, xem nhánh `match` của nó ngay dưới ([`normalize::normalize`]).
//! [`Step::ExtractMainContent`] (`dom_smoothie`, Story 6.9), [`Step::CleanByRules`]
//! (Story 6.5), [`Step::Preview`] (Story 6.5/6.9) — ba bước CÒN LẠI là no-op trong `match`
//! của [`run_import_with_order`]. Chúng CÓ MẶT trong [`PIPELINE_ORDER`] và trong
//! [`PipelineOutput::trace`] của MỌI lượt chạy — một bước thân rỗng vẫn phải NÓI ĐƯỢC là đã
//! đi qua, không được biến mất khỏi vết chạy chỉ vì nó không làm gì (AC6 của spec 6.2). Vết
//! chạy được ghi TỪ BÊN TRONG mỗi nhánh `match`, không phải một `trace.push` chung sau vòng
//! lặp — một `trace.push` chung phản ánh đúng `order` truyền vào, không phản ánh việc handler
//! có thật sự chạy hay không.
//!
//! [`Step::SplitChapters`] KHÔNG nằm trong bốn bước trên — xem doc-comment của nó: nó có một
//! cơ chế thật (so khớp chuỗi con literal), nhưng KHÔNG PHẢI mẫu phân tách NGƯỜI DÙNG cấu
//! hình được (đó là Story 6.6); sản phẩm hôm nay không có bề mặt nào đưa một mẫu vào, nên
//! [`PipelineInput::chapter_pattern`] luôn `None` trên đường sản phẩm ⇒ bước này vẫn là no-op
//! trong thực tế, N = 1, hành vi không đổi. Cơ chế thật chỉ được `tests/**` gọi tới — đó là
//! điều kiện để đối chứng AD-39 dựng được (xem doc-comment [`split_chapters_step`]).
//!
//! ─────────────────────────────────────────────────────────────────────────────
//! 🔴 CHỈ MỘT WRITER — KHÔNG `Store`/`Transaction` Ở ĐÂY
//! ─────────────────────────────────────────────────────────────────────────────
//! Module này thuần: không I/O, không SQL. `commands::project::create_work` chạy toàn bộ
//! [`run_import`] TRƯỚC khi mở giao dịch ghi — cùng lý do Quyết định #3 cũ của Story 1.15
//! (AD-11 giữ MỘT writer duy nhất nối tiếp; CPU trong closure ghi chặn MỌI lượt ghi khác).

use super::import::{ImportError, ImportedChapter};
use super::normalize;
use super::split::{SplitSegment, split_source_text};

// ═════════════════════════════════════════════════════════════════════════════════
// Bước — enum trần, và thứ tự là DỮ LIỆU (xem doc-comment đầu tệp)
// ═════════════════════════════════════════════════════════════════════════════════

/// Một bước trong chuỗi bảy bước của AD-39 (spine `:473-482`).
///
/// ⚠️ Bước 8 (ghi `.atproj`) KHÔNG có mặt ở đây — nó sống ở `commands::project`, canh bằng
/// một cổng ("không bước nào của chuỗi chạy SAU nó"), không bằng vị trí tệp. Xem §Design
/// Notes "Vì sao chuỗi dừng ở bước 7" của spec 6.2.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Step {
    /// Bước 1 — giải mã bảng mã ĐÃ KHAI (mặc định UTF-8; dò bảng mã là Story 6.3).
    DecodeEncoding,
    /// Bước 2 — bóc nội dung chính. THÂN RỖNG (Story 6.9, `dom_smoothie`).
    ExtractMainContent,
    /// Bước 3 — làm sạch theo luật. THÂN RỖNG (Story 6.5).
    CleanByRules,
    /// Bước 4 — chuẩn hoá đoạn & khoảng trắng ([`normalize::normalize`], Story 6.4,
    /// FR124/FR125). 🔵 SỬA 2026-09-04 — KHÔNG còn thân rỗng.
    NormalizeParagraphsAndWhitespace,
    /// Bước 5 — tách Chương theo mẫu phân tách. Có cơ chế thật (so khớp literal), nhưng
    /// KHÔNG phải mẫu người dùng cấu hình được (Story 6.6) — xem doc-comment
    /// [`split_chapters_step`].
    SplitChapters,
    /// Bước 6 — xem trước + sửa tay. THÂN RỖNG (Story 6.5/6.9).
    Preview,
    /// Bước 7 — tách segment + cờ kết đoạn. GỌI [`split_source_text`] đã có (AD-4, AD-37),
    /// không viết lại.
    SplitSegments,
}

/// Thứ tự SẢN PHẨM — khớp từng bước AD-39 (spine `:473-482`), theo ĐÚNG thứ tự đó.
///
/// 🔴 Đây là hằng số MÀ MỌI THỨ trong module này tiêu thụ như một GIÁ TRỊ, không phải một
/// chuỗi lệnh gọi lần lượt trong thân [`run_import`] — xem doc-comment đầu tệp. `tests/**`
/// dựng một `[Step; 7]` KHÁC (một hoán vị của cùng bảy biến thể) và gọi thẳng
/// [`run_import_with_order`] với nó để dựng đối chứng đỏ cho AD-39 — không sửa hằng này.
pub const PIPELINE_ORDER: [Step; 7] = [
    Step::DecodeEncoding,
    Step::ExtractMainContent,
    Step::CleanByRules,
    Step::NormalizeParagraphsAndWhitespace,
    Step::SplitChapters,
    Step::Preview,
    Step::SplitSegments,
];

/// `order` phải là một HOÁN VỊ của bảy biến thể [`Step`] — không thiếu, không thừa, không
/// trùng. Chạy TRƯỚC khi bất kỳ bước nào thực thi (vòng rà đối kháng 2026-09-04): một bước
/// TRÙNG có thể chạy lại một biến đổi không idempotent, một bước THIẾU để lại trạng thái dở
/// mà không lỗi nào ném — đúng hai lớp lỗi mà chính chuỗi bảy bước gốc tồn tại để chặn,
/// không thể để hở lại nó ở cổng vào của seam đã mở cho `tests/**`.
fn validate_order(order: &[Step]) -> Result<(), ImportError> {
    if order.len() != PIPELINE_ORDER.len() {
        return Err(ImportError::InvalidPipelineOrder {
            detail: format!(
                "do dai {} != {} (thieu hoac thua buoc)",
                order.len(),
                PIPELINE_ORDER.len()
            ),
        });
    }
    for step in PIPELINE_ORDER {
        let count = order.iter().filter(|&&s| s == step).count();
        if count != 1 {
            return Err(ImportError::InvalidPipelineOrder {
                detail: format!("buoc {step:?} xuat hien {count} lan, can dung 1"),
            });
        }
    }
    Ok(())
}

// ═════════════════════════════════════════════════════════════════════════════════
// Hình dạng đầu vào — bước 0, do module NGUỒN cung cấp (AD-39 :498)
// ═════════════════════════════════════════════════════════════════════════════════

/// Hình dạng của MỘT đơn vị nội dung ở bước đầu vào — do module nguồn cung cấp:
/// `core::segment::import` cho file/dán tay hôm nay; `webimport`/`export` cho URL/`.docx`
/// các story sau (AD-39 `:498` — "chỉ cung cấp bước đầu vào, không giữ bản sao của các bước
/// dùng chung").
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ChapterInput {
    /// Byte thô, chưa giải mã — nguồn KHÔNG tự khai bảng mã (`.txt`, `.md`, phản hồi HTTP).
    RawBytes {
        /// Byte đúng như đọc được từ nguồn — CHƯA cắt BOM, CHƯA giải mã. Cả hai việc đó là
        /// [`Step::DecodeEncoding`] (bước 1 của chuỗi), không phải việc của bước đầu vào.
        bytes: Vec<u8>,
        /// Tên/đường dẫn nguồn, CHỈ để chẩn đoán khi [`Step::DecodeEncoding`] trượt
        /// (`ImportError::UndecodableBytes::path`, đổi tên 2026-09-04 Story 6.3 — xem
        /// doc-comment biến thể đó). Rỗng là hợp lệ khi nguồn không có một cái tên có
        /// nghĩa (nhánh này chỉ thật sự được đọc trên đường LỖI).
        label: String,
    },
    /// Đã LÀ văn bản — hình dạng "tự khai bảng mã" (`.docx`, Story 6.12) HOẶC văn bản dán
    /// tay (không có bảng mã nào để mà giải — nó đã là `String` từ lúc rời webview). Bước
    /// giải mã BỎ QUA vế transcode cho hình dạng này (xem doc-comment `decode_unit`).
    AlreadyText(String),
}

/// Đầu vào của TOÀN chuỗi — một khối chưa tách Chương, hoặc N đơn vị đã sẵn.
///
/// Bảng hình dạng của AD-39 (spine `:486-491`): điều kiện áp bước tách Chương phát biểu theo
/// HÌNH DẠNG đầu vào, không theo danh sách đường nhập — danh sách sẽ sai ngay khi có đường
/// thứ tư (URL, song ngữ, …), hình dạng thì đúng mãi.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PipelineShape {
    /// Một khối chưa tách Chương — [`Step::SplitChapters`] CHẠY. File `.txt`/`.md`/`.docx`,
    /// văn bản dán tay, tài liệu song ngữ (mẫu áp lên cột nguồn) đều rơi vào đây.
    Blob(ChapterInput),
    /// Đã LÀ N đơn vị Chương — [`Step::SplitChapters`] BỎ QUA. Mỗi link trong danh sách URL
    /// (Story 6.7) là một ví dụ: một link đã là một Chương — KỂ CẢ khi danh sách chỉ có
    /// ĐÚNG MỘT link (xem [`Flow::already_chaptered`], vòng rà đối kháng 2026-09-04: bỏ qua
    /// hay không là quyết định của HÌNH DẠNG này, không phải của độ dài quan sát được).
    Chapters(Vec<ChapterInput>),
}

/// Đầu vào ĐẦY ĐỦ của [`run_import`]/[`run_import_with_order`] — hình dạng cộng những gì
/// bước 1 và bước 5 cần biết để chạy.
pub struct PipelineInput {
    pub shape: PipelineShape,
    /// Bảng mã ĐÃ KHAI cho [`Step::DecodeEncoding`].
    ///
    /// 🔵 **SỬA 2026-09-04 (Story 6.3) — "KHÔNG dò" đã HẾT ĐÚNG.** Bản Story 6.2 khai cứng
    /// UTF-8 trên mọi đường sản phẩm; `core::segment::encoding::detect` (mới, Story 6.3)
    /// giờ dò thật, và `commands::project::preview_import_encoding` +
    /// [`PipelineInput::with_encoding`] là đường sản phẩm khai một bảng mã KHÁC UTF-8.
    /// [`PipelineInput::default_shaped`] VẪN khai UTF-8 cứng — nó ở lại cho hai chỗ gọi
    /// KHÔNG đi qua xem trước bảng mã: `commands::project::create_work_from_text`/
    /// `create_work_from_file` (hai hàm thuần cũ, vẫn được `tests/**` gọi trực tiếp rất
    /// nhiều nơi) và mọi ca `tests/**` dựng đối chứng AD-39 cần byte THẬT SỰ đổi hình dạng
    /// qua bước giải mã (§Design Notes "Vì sao ca đối chứng cần byte chưa giải mã" của spec
    /// 6.2) mà không cần dò gì cả.
    pub encoding: &'static encoding_rs::Encoding,
    /// Mẫu phân tách của [`Step::SplitChapters`] — chuỗi con literal, KHÔNG regex, và KHÔNG
    /// cấu hình được bởi người dùng ở story này (Story 6.6 sở hữu mẫu thật, cấu hình được).
    /// `None` ⇒ bước 5 là no-op, giữ N = 1 — đúng hành vi sản phẩm hôm nay.
    pub chapter_pattern: Option<String>,
    /// `work.source_lang` — bước 7 ([`split_source_text`]) rẽ nhánh Trung/Anh theo trường
    /// này (AD-18: không đoán từ nội dung).
    pub source_lang: String,
}

impl PipelineInput {
    /// Cấu hình MẶC ĐỊNH của đường sản phẩm hôm nay — UTF-8, không mẫu phân tách. Xem
    /// doc-comment các trường `encoding`/`chapter_pattern` ở trên cho lý do.
    pub fn default_shaped(shape: PipelineShape, source_lang: impl Into<String>) -> Self {
        PipelineInput {
            shape,
            encoding: encoding_rs::UTF_8,
            chapter_pattern: None,
            source_lang: source_lang.into(),
        }
    }

    /// **THÊM 2026-09-04 (Story 6.3)** — cấu hình mang một bảng mã ĐÃ CHỌN, đứng CẠNH
    /// [`PipelineInput::default_shaped`] (§Always spec 6.3: không sửa/xoá constructor cũ).
    /// Đường sản phẩm dùng hàm này khi người dùng đã xác nhận một ứng viên ở màn xem trước
    /// bảng mã (`commands::project::confirm_import_with_encoding`) — `chapter_pattern` vẫn
    /// `None` (Never clause của spec 6.2/6.3: mẫu phân tách người dùng cấu hình được là
    /// Story 6.6, không thuộc phạm vi này).
    pub fn with_encoding(
        shape: PipelineShape,
        encoding: &'static encoding_rs::Encoding,
        source_lang: impl Into<String>,
    ) -> Self {
        PipelineInput { shape, encoding, chapter_pattern: None, source_lang: source_lang.into() }
    }
}

/// Thủ công vì `encoding_rs::Encoding` không tự `Debug` — in TÊN NHÃN WHATWG
/// (`Encoding::name()`) thay vì cố in kiểu Rust của nó. Ba kiểu công khai khác của module
/// này ([`ChapterInput`]/[`PipelineShape`]/[`PipelineOutput`]) đều `#[derive(Debug)]`; kiểu
/// này lẽ ra cũng vậy nếu không vướng đúng một trường (vòng rà đối kháng 2026-09-04).
impl std::fmt::Debug for PipelineInput {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PipelineInput")
            .field("shape", &self.shape)
            .field("encoding", &self.encoding.name())
            .field("chapter_pattern", &self.chapter_pattern)
            .field("source_lang", &self.source_lang)
            .finish()
    }
}

/// Kết quả một lượt chạy chuỗi — N Chương (N = 1 ở story này, xem `chapter_pattern: None`),
/// cộng vết chạy.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PipelineOutput {
    pub chapters: Vec<ImportedChapter>,
    /// Đúng các bước ĐÃ thực thi, theo thứ tự — kể cả bước thân rỗng (AC6 spec 6.2: "bước
    /// rỗng vẫn có mặt trong vết chạy", không nuốt im lặng). Ghi TỪ BÊN TRONG mỗi nhánh xử
    /// lý, không phải một `trace.push` chung sau vòng lặp — xem doc-comment đầu tệp.
    pub trace: Vec<Step>,
}

// ═════════════════════════════════════════════════════════════════════════════════
// Trạng thái chảy trong chuỗi
// ═════════════════════════════════════════════════════════════════════════════════

/// Một đơn vị đang chảy trong chuỗi — byte thô (trước bước giải mã) hoặc văn bản.
#[derive(Debug, Clone)]
enum Unit {
    Undecoded { bytes: Vec<u8>, label: String },
    Decoded(String),
}

impl From<ChapterInput> for Unit {
    fn from(v: ChapterInput) -> Unit {
        match v {
            ChapterInput::RawBytes { bytes, label } => Unit::Undecoded { bytes, label },
            ChapterInput::AlreadyText(t) => Unit::Decoded(t),
        }
    }
}

/// Trạng thái đầy đủ chảy qua từng bước.
struct Flow {
    units: Vec<Unit>,
    /// Segment đã tách cho từng phần tử `units`, SONG SONG theo INDEX. `None` = chưa tách.
    ///
    /// ⚠️ Trên MỌI thứ tự HỢP LỆ (đã qua [`validate_order`]) mà spec 6.2 đòi hỏi (thứ tự
    /// sản phẩm, và hoán vị decode/split-chapters của đối chứng AD-39), [`Step::SplitSegments`]
    /// luôn chạy SAU CÙNG khi `units` đã ổn định — trường này chỉ còn `None` khi một thứ tự
    /// (vẫn HỢP LỆ về mặt tập hợp bảy bước) đặt `SplitSegments` trước khi Chương được giải
    /// mã hoặc tách xong; xem `split_segments_step` cho cách nó không panic trên ca đó.
    segments: Vec<Option<Vec<SplitSegment>>>,
    /// Hình dạng đầu vào GỐC — `true` khi [`PipelineShape::Chapters`], `false` khi
    /// [`PipelineShape::Blob`]. [`split_chapters_step`] rẽ theo TRƯỜNG NÀY, không suy từ
    /// `units.len()` (vòng rà đối kháng 2026-09-04: một danh sách URL có ĐÚNG MỘT link vẫn
    /// là hình dạng "đã chia Chương" và không được đem đi tách lại, dù độ dài quan sát được
    /// trùng với độ dài của một `Blob` chưa tách).
    already_chaptered: bool,
}

// ═════════════════════════════════════════════════════════════════════════════════
// Bộ chạy
// ═════════════════════════════════════════════════════════════════════════════════

/// Bộ chạy nhận THỨ TỰ TUỲ Ý — công khai để `tests/**` gọi được, dựng đối chứng đỏ cho AD-39
/// bằng cách đảo thứ tự rồi cho chạy qua ĐÚNG bộ chạy sản phẩm.
///
/// 🔴 Đúng MỘT chỗ gọi sản phẩm của bộ chạy NÀY: [`run_import`] ngay dưới — nó KHÔNG được
/// gọi trực tiếp từ `commands/**` (`segment_pipeline_boundary.rs` canh mệnh đề đó, cho CẢ
/// hai tên `run_import`/`run_import_with_order`). Nếu không, cái seam mở cho test sẽ thành
/// một đường tắt cho một story sau: một chỗ gọi sản phẩm thứ hai có thể âm thầm truyền một
/// thứ tự KHÁC `PIPELINE_ORDER` mà không ai ký.
pub fn run_import_with_order(
    order: &[Step],
    input: PipelineInput,
) -> Result<PipelineOutput, ImportError> {
    validate_order(order)?;

    let PipelineInput { shape, encoding, chapter_pattern, source_lang } = input;

    let (initial_units, already_chaptered): (Vec<Unit>, bool) = match shape {
        PipelineShape::Blob(c) => (vec![Unit::from(c)], false),
        PipelineShape::Chapters(cs) => (cs.into_iter().map(Unit::from).collect(), true),
    };
    let n = initial_units.len();
    let mut flow = Flow { units: initial_units, segments: vec![None; n], already_chaptered };

    let mut trace: Vec<Step> = Vec::with_capacity(order.len());
    for &step in order {
        flow = match step {
            Step::DecodeEncoding => {
                let Flow { units: old_units, segments, already_chaptered } = flow;
                let mut units = Vec::with_capacity(old_units.len());
                for u in old_units {
                    units.push(decode_unit(u, encoding)?);
                }
                trace.push(step);
                Flow { units, segments, already_chaptered }
            }
            Step::ExtractMainContent => {
                trace.push(step);
                flow
            }
            Step::CleanByRules => {
                trace.push(step);
                flow
            }
            // 🔴 THÂN THẬT — Story 6.4, FR124/FR125, AD-39 bước 4. GỌI `normalize::normalize`,
            // không viết lại nội tuyến (Task list spec 6.4) — mọi luật (bảng kết câu, bảng
            // nối theo ngôn ngữ) sống ở `split.rs`/`regroup.rs`, module này chỉ GỌI chúng.
            // `trace.push` Ở LẠI BÊN TRONG nhánh (AC6 spec 6.2, doc-comment đầu tệp) —
            // KHÔNG gộp vào một `trace.push` chung sau vòng lặp.
            Step::NormalizeParagraphsAndWhitespace => {
                let Flow { units: old_units, segments, already_chaptered } = flow;
                let units = old_units
                    .into_iter()
                    .map(|u| match u {
                        Unit::Decoded(text) => {
                            Unit::Decoded(normalize::normalize(&text, &source_lang).text)
                        }
                        // `Unit::Undecoded` ở bước này là BẤT KHẢ trên mọi thứ tự HỢP LỆ
                        // (`validate_order` đã kiểm — bước 1 luôn đứng trước bước 4). Giữ
                        // nguyên là phòng thủ cho một thứ tự SAI (đối chứng AD-39 đặt bước
                        // này TRƯỚC giải mã): `normalize` cần `&str`, không có nghĩa gì để
                        // chạy nó trên byte thô — cùng khuôn `split_segments_step` ngay
                        // dưới, cũng bỏ qua `Unit::Undecoded` vì cùng lý do.
                        other @ Unit::Undecoded { .. } => other,
                    })
                    .collect();
                trace.push(step);
                Flow { units, segments, already_chaptered }
            }
            Step::SplitChapters => {
                let next = split_chapters_step(flow, chapter_pattern.as_deref());
                trace.push(step);
                next
            }
            Step::Preview => {
                trace.push(step);
                flow
            }
            Step::SplitSegments => {
                let next = split_segments_step(flow, &source_lang);
                trace.push(step);
                next
            }
        };
    }

    let chapters: Vec<ImportedChapter> = flow
        .units
        .into_iter()
        .zip(flow.segments)
        .map(|(u, s)| -> Result<ImportedChapter, ImportError> {
            let source_text = match u {
                Unit::Decoded(t) => t,
                // 🔴 KHÔNG THỂ xảy ra sau `validate_order`: `DecodeEncoding` xuất hiện ĐÚNG
                // một lần trong mọi thứ tự hợp lệ, và nó giải mã MỌI đơn vị đang có tại thời
                // điểm nó chạy; không bước nào sau đó tạo lại một `Unit::Undecoded` từ một
                // `Unit::Decoded`. Dù vậy, đây là chỗ DỰNG KẾT QUẢ cuối cùng — một byte thô
                // lọt tới đây phải là một TỪ CHỐI tường minh (vòng rà đối kháng 2026-09-04:
                // bản trước dùng `String::from_utf8_lossy`, ghi `U+FFFD` vào văn bản TRONG
                // IM LẶNG — đúng lớp lỗi "rỗng/hỏng ngầm" mà AGENTS.md gọi tên là trung tâm
                // của dự án), không phải một lượt thay ký tự.
                Unit::Undecoded { label, .. } => {
                    return Err(ImportError::UndecodableBytes {
                        path: label,
                        encoding: encoding.name().to_owned(),
                    });
                }
            };
            Ok(ImportedChapter { source_text, segments: s.unwrap_or_default() })
        })
        .collect::<Result<Vec<_>, ImportError>>()?;

    Ok(PipelineOutput { chapters, trace })
}

/// Đường SẢN PHẨM — uỷ quyền cho [`run_import_with_order`] với [`PIPELINE_ORDER`]. Chỗ gọi
/// sản phẩm DUY NHẤT: `commands::project::create_work`
/// (`segment_pipeline_boundary.rs::run_import_is_the_one_product_call_site`).
pub fn run_import(input: PipelineInput) -> Result<PipelineOutput, ImportError> {
    run_import_with_order(&PIPELINE_ORDER, input)
}

// ═════════════════════════════════════════════════════════════════════════════════
// Bước 1 — giải mã bảng mã ĐÃ KHAI, cộng strip BOM
// ═════════════════════════════════════════════════════════════════════════════════

/// Bước 1. `Unit::Decoded` đi qua CHỈ với [`strip_bom`] — vế TRANSCODE bỏ qua cho hình dạng
/// đã-là-văn-bản (`.docx`, dán tay), đúng bảng hình dạng AD-39; strip BOM vẫn chạy đều cho
/// CẢ hai hình dạng vì nó là một chuẩn hoá vô hại và Story 1.15 đã luôn chạy nó cho CẢ đường
/// dán tay lẫn đường đọc tệp — giữ nguyên hành vi đó là điều kiện của §Always ("hành vi sản
/// phẩm không đổi"). ⚠️ Nhánh này không có ca hồi quy riêng cho tới vòng rà đối kháng
/// 2026-09-04 — `project_contract.rs::a_pasted_leading_bom_is_stripped_too` đóng chỗ hở đó.
///
/// `Unit::Undecoded`: **hai đường**, tuỳ bảng mã.
///
/// - `encoding == UTF_8` (ĐƯỜNG SẢN PHẨM, Never clause của spec 6.2) → `String::from_utf8`,
///   TÁI DÙNG chính bộ đệm của `bytes` — **0 memcpy**. Nghiêm y hệt
///   `decode_without_bom_handling_and_without_replacement` (đo dưới đây), nhưng không trả
///   một `Cow::Borrowed` rồi `.into_owned()` COPY TRỌN bộ đệm trong khi bản gốc còn sống —
///   vòng rà đối kháng 2026-09-04 đo ra bản trước làm đúng việc đó, đỉnh bộ nhớ ~2× ở trần
///   `MAX_IMPORT_BYTES` cộng một memcpy đầy, trên CHÍNH đường chạy cho MỌI tệp sản phẩm.
/// - Bảng mã KHÁC UTF-8 (chỉ `tests/**` khai — dựng đối chứng AD-39) →
///   [`encoding_rs::Encoding::decode_without_bom_handling_and_without_replacement`], KHÔNG
///   phải `decode_without_bom_handling` (bản `_lossy`): Quyết định #6 cũ của Story 1.15 đòi
///   TỪ CHỐI tường minh byte không hợp lệ với bảng mã đã khai, không thay thế bằng `U+FFFD`
///   (Bẫy 8 kế thừa từ `import.rs` cũ). Transcode giữa hai bảng mã khác nhau không có đường
///   0-chép nào để mà tối ưu, nên đây đúng là công cụ, không phải một đánh đổi.
fn decode_unit(unit: Unit, encoding: &'static encoding_rs::Encoding) -> Result<Unit, ImportError> {
    match unit {
        Unit::Decoded(t) => Ok(Unit::Decoded(strip_bom(t))),
        Unit::Undecoded { bytes, label } => {
            let text = if encoding == encoding_rs::UTF_8 {
                String::from_utf8(bytes).map_err(|_| ImportError::UndecodableBytes {
                    path: label,
                    encoding: encoding.name().to_owned(),
                })?
            } else {
                encoding
                    .decode_without_bom_handling_and_without_replacement(&bytes)
                    .ok_or_else(|| ImportError::UndecodableBytes {
                        path: label,
                        encoding: encoding.name().to_owned(),
                    })?
                    .into_owned()
            };
            Ok(Unit::Decoded(strip_bom(text)))
        }
    }
}

/// Cắt dấu thứ tự byte (`U+FEFF`) ở **đầu** chuỗi, nếu có.
///
/// 🔵 **CHUYỂN 2026-09-04 (Story 6.2) — hàm này SỐNG Ở ĐÂY, không còn ở `core::segment::import`.**
/// Doc-comment gốc (`import.rs` cũ, Story 1.15) giải thích đầy đủ LÝ DO nó là một bước GIẢI
/// MÃ chứ không phải một bước CHUẨN HOÁ của Epic 6 — lý lẽ đó không đổi, chỉ chỗ Ở của hàm
/// đổi, đúng khuôn "bước giải mã và strip BOM chuyển vào chuỗi" của spec 6.2 (Task 3).
///
/// BOM là một tạo tác của phép mã hoá, không phải một đặc điểm của văn bản — mọi bộ giải mã
/// UTF-8 nghiêm túc đều nuốt nó, và cắt nó hoàn tất đúng bước mà Quyết định #6 (Story 1.15)
/// đã giao: giải mã, không đoán bảng mã. `EF BB BF` là UTF-8 HỢP LỆ nên nó đi lọt một phép
/// giải mã nghiêm mà không cổng nào kêu; AD-4 đóng băng ranh giới segment tính MỘT LẦN lúc
/// nhập, nên một `U+FEFF` nằm lại sẽ trở thành ký tự đầu của segment #1 VĨNH VIỄN.
///
/// CRLF thì NGƯỢC LẠI và cố ý KHÔNG đụng ở đây — xuống dòng LÀ chuẩn hoá văn bản thật
/// (FR124/125), khác TẦNG với BOM. 🔵 **SỬA 2026-09-04 (Story 6.4)** — "Epic 6" ở câu trên
/// từng là một lời hẹn tương lai; nay là một chỗ CỤ THỂ: bước 4 của chuỗi AD-39
/// (`Step::NormalizeParagraphsAndWhitespace`, [`normalize::normalize`], ĐỨNG SAU bước NÀY).
/// Chỉ cắt ở ĐẦU: một `U+FEFF` ở giữa văn bản là zero-width no-break space, một ký tự thật
/// của nội dung.
fn strip_bom(raw: String) -> String {
    match raw.strip_prefix('\u{feff}') {
        Some(rest) => rest.to_owned(),
        None => raw,
    }
}

// ═════════════════════════════════════════════════════════════════════════════════
// Bước 5 — tách Chương theo mẫu phân tách
// ═════════════════════════════════════════════════════════════════════════════════

/// Bước 5.
///
/// ─────────────────────────────────────────────────────────────────────────────
/// 🔴 ĐÂY LÀ CƠ CHẾ TỐI THIỂU, KHÔNG PHẢI MẪU NGƯỜI DÙNG CẤU HÌNH ĐƯỢC (Story 6.6)
/// ─────────────────────────────────────────────────────────────────────────────
/// So khớp CHUỖI CON LITERAL (không regex, không tiêu đề thông minh, không cấu hình theo
/// Tác phẩm) — đủ để chứng minh THỨ TỰ có ý nghĩa thật (điều kiện của AC5 spec 6.2), KHÔNG
/// đủ và KHÔNG định thay thế mẫu thật của Story 6.6. Sản phẩm hôm nay không có bề mặt nào
/// đưa một `chapter_pattern` vào [`PipelineInput`] — `commands::project::create_work` luôn
/// khai `None` ([`PipelineInput::default_shaped`]) ⇒ bước này LUÔN LÀ NO-OP trên đường sản
/// phẩm, N = 1, hành vi không đổi. Chỉ `tests/**` khai `Some(..)`.
///
/// ─────────────────────────────────────────────────────────────────────────────
/// 🔴 CHỖ TRIỆU CHỨNG AD-39 SỐNG — nhánh `Unit::Undecoded`
/// ─────────────────────────────────────────────────────────────────────────────
/// Khi bước này chạy TRƯỚC [`Step::DecodeEncoding`] (một thứ tự SAI), đơn vị đang chảy vẫn
/// là byte thô theo bảng mã đã khai (ví dụ GBK). Mẫu phân tách là một `&str` Rust — bao giờ
/// cũng là byte UTF-8. Tìm CHUỖI BYTE UTF-8 của mẫu bên trong byte GBK: byte GBK của một chữ
/// Hán không trùng byte UTF-8 của CHÍNH chữ đó (hai bảng mã khác nhau ở tầng byte), nên phép
/// tìm dưới đây trả 0 khớp một cách TỰ NHIÊN — không ép, không bắt lỗi giả. Kết quả: một
/// mảnh DUY NHẤT, y hệt input, KHÔNG NÉM LỖI NÀO. Đây CHÍNH LÀ câu spine `:470` mô tả — cả
/// file ra đúng một Chương, không lỗi nào được ném. Nếu bước này chạy SAU khi đã giải mã
/// đúng, mẫu (cũng UTF-8) khớp trên văn bản đã giải mã đúng và tách ra nhiều Chương thật —
/// khác biệt QUAN SÁT ĐƯỢC đó là đối chứng cho AC5 (spine `:498`, §Design Notes spec 6.2).
///
/// Chỉ chạm hình dạng [`PipelineShape::Blob`] — rẽ theo [`Flow::already_chaptered`] (HÌNH
/// DẠNG khai báo), KHÔNG suy từ `units.len()` (vòng rà đối kháng 2026-09-04: một
/// [`PipelineShape::Chapters`] với ĐÚNG MỘT phần tử có cùng độ dài quan sát được với một
/// `Blob` chưa tách, nhưng phải BỎ QUA — đúng bảng hình dạng AD-39, spine `:486-491`).
fn split_chapters_step(mut flow: Flow, pattern: Option<&str>) -> Flow {
    if flow.already_chaptered {
        return flow;
    }
    let Some(pattern) = pattern else {
        return flow;
    };

    // Bất biến: hình dạng `Blob` khởi tạo ĐÚNG MỘT đơn vị, và không bước nào TRƯỚC bước này
    // (trong bất kỳ hoán vị HỢP LỆ nào — `validate_order` đã kiểm ở đầu
    // `run_import_with_order`) làm tăng số đơn vị. `Vec::pop` không panic trên Vec rỗng (trả
    // `None`) — an toàn hơn `.into_iter().next().expect(..)` của bản trước (vòng rà đối
    // kháng 2026-09-04, item 4: panic trên đường ghi/tính pipeline không có kiểu).
    let Some(unit) = flow.units.pop() else {
        return flow; // rỗng bất thường — không có gì để tách, giữ nguyên trạng thái rỗng
    };

    let pieces: Vec<Unit> = match unit {
        Unit::Decoded(text) => split_on_literal(&text, pattern)
            .into_iter()
            .map(|s| Unit::Decoded(s.to_owned()))
            .collect(),
        Unit::Undecoded { bytes, label } => split_on_literal_bytes(&bytes, pattern.as_bytes())
            .into_iter()
            .map(|b| Unit::Undecoded { bytes: b.to_vec(), label: label.clone() })
            .collect(),
    };

    let n = pieces.len();
    flow.units = pieces;
    // Reset — số phần tử vừa đổi, một `Some(..)` cũ (nếu có, từ một thứ tự bị đảo NGOÀI
    // phạm vi đối chứng chính thức) không còn khớp INDEX nào có nghĩa.
    flow.segments = vec![None; n];
    flow
}

/// So khớp trên `&str` — nhánh `Unit::Decoded` của [`split_chapters_step`].
///
/// 🔵 **SỬA (vòng rà đối kháng 2026-09-04) — KHÔNG còn `.map(str::trim)` trên mảnh giữ
/// lại.** Bản trước trim mỗi mảnh trước khi lưu — đó là CHUẨN HOÁ KHOẢNG TRẮNG, việc của
/// [`Step::NormalizeParagraphsAndWhitespace`], không phải việc của bước NÀY (bước 5 đứng
/// SAU bước 4 trong [`PIPELINE_ORDER`]); nó từng xoá mất khoảng trắng đầu dòng/tiêu đề có
/// chủ ý của người viết ngay trong `source_text` được lưu. 🔵 **SỬA THÊM 2026-09-04 (Story
/// 6.4)** — bước 4 KHÔNG còn "thân rỗng": [`normalize::normalize`] nay trim thật hai đầu
/// MỖI DÒNG trước khi bước này chạy; mệnh đề "không trim ở ĐÂY" vẫn đúng và giờ có một lý do
/// MẠNH hơn (không phải "chưa ai làm", mà "đã có nơi làm ĐÚNG, làm lại ở đây là một nguồn sự
/// thật thứ hai"). `s.trim().is_empty()` chỉ dùng để QUYẾT ĐỊNH có giữ một
/// mảnh hay không (một khoảng trống thuần giữa hai lần khớp liền nhau không phải một
/// Chương) — KHÔNG áp lên giá trị trả về, mảnh giữ lại đi ra NGUYÊN VĂN. Nếu không mảnh nào
/// còn lại (mẫu không khớp, hoặc chỉ khớp ở đầu/cuối), trả nguyên văn bản làm MỘT Chương —
/// không có mẫu khớp nghĩa là không có gì để tách, không phải một danh sách rỗng.
fn split_on_literal<'a>(text: &'a str, pattern: &str) -> Vec<&'a str> {
    if pattern.is_empty() {
        return vec![text];
    }
    let parts: Vec<&str> = text.split(pattern).filter(|s| !s.trim().is_empty()).collect();
    if parts.is_empty() { vec![text] } else { parts }
}

/// So khớp trên byte thô — nhánh `Unit::Undecoded` của [`split_chapters_step`]. Cùng LUẬT
/// "không khớp ⇒ trả nguyên bản" và "mảnh giữ lại đi ra NGUYÊN VĂN, không cắt gì" với
/// [`split_on_literal`] — nhưng phép NHẬN BIẾT "rỗng" KHÔNG THỂ giống nhau, và đó không phải
/// một chỗ lệch cần vá: [`split_on_literal`] nhận biết khoảng trắng qua `char::is_whitespace`
/// trên một `&str` ĐÃ BIẾT bảng mã (luôn UTF-8); ở ĐÂY dữ liệu là byte THÔ theo một bảng mã
/// CHƯA giải mã (đó chính là điều kiện để triệu chứng AD-39 dựng được, xem doc-comment
/// [`split_chapters_step`]) — không có cách nào an toàn để nhận diện "khoảng trắng" trên byte
/// của một bảng mã tuỳ ý mà không giải mã nó trước, và giải mã Ở ĐÂY sẽ là một bước giải mã
/// THỨ HAI ngoài [`Step::DecodeEncoding`]. Vì vậy chỉ lát byte ĐỘ DÀI 0 bị loại. Viết tay vì
/// `[u8]` không có `split` theo một CHUỖI CON tuỳ ý trong thư viện chuẩn (chỉ có tách theo
/// một phần tử/vị từ trên từng phần tử).
fn split_on_literal_bytes<'a>(data: &'a [u8], pattern: &[u8]) -> Vec<&'a [u8]> {
    if pattern.is_empty() {
        return vec![data];
    }
    let mut out = Vec::new();
    let mut start = 0usize;
    let mut i = 0usize;
    while i + pattern.len() <= data.len() {
        if &data[i..i + pattern.len()] == pattern {
            out.push(&data[start..i]);
            i += pattern.len();
            start = i;
        } else {
            i += 1;
        }
    }
    out.push(&data[start..]);

    let filtered: Vec<&[u8]> = out.into_iter().filter(|s| !s.is_empty()).collect();
    if filtered.is_empty() { vec![data] } else { filtered }
}

// ═════════════════════════════════════════════════════════════════════════════════
// Bước 7 — tách segment + cờ kết đoạn (GỌI bộ tách đã có, không viết lại)
// ═════════════════════════════════════════════════════════════════════════════════

/// Bước 7. Gọi [`split_source_text`] cho từng đơn vị CHƯA có segment (`segments[i] ==
/// None`) — trên MỌI thứ tự mà spec 6.2 đòi hỏi, đây luôn là toàn bộ `units`, vì bước này
/// đứng CUỐI [`PIPELINE_ORDER`] và chưa bước nào khác từng đặt `segments[i]`.
///
/// ⚠️ `Unit::Undecoded` bị BỎ QUA (để `None`) — phòng thủ cho một thứ tự (vẫn HỢP LỆ về mặt
/// tập hợp bảy bước, xem [`validate_order`]) đặt bước NÀY trước cả giải mã; `split_source_text`
/// cần `&str`, và không có nghĩa để chạy nó trên byte thô. Vòng dựng [`PipelineOutput`] cuối
/// cùng đọc `None` thành 0 segment — không panic.
fn split_segments_step(mut flow: Flow, source_lang: &str) -> Flow {
    for (unit, seg) in flow.units.iter().zip(flow.segments.iter_mut()) {
        if seg.is_some() {
            continue;
        }
        if let Unit::Decoded(text) = unit {
            *seg = Some(split_source_text(text, source_lang));
        }
    }
    flow
}
