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
//! 🔴 HAI BƯỚC THÂN RỖNG — CÓ CHỦ Ý, MỖI BƯỚC MỘT STORY CHỦ (§Never của spec 6.2)
//! ─────────────────────────────────────────────────────────────────────────────
//! 🔵 **SỬA 2026-09-04 (Story 6.4)** — từ "BỐN bước" xuống "BA bước": [`Step::NormalizeParagraphsAndWhitespace`]
//! không còn no-op, xem nhánh `match` của nó ngay dưới ([`normalize::normalize`]).
//! 🔵 **SỬA 2026-09-05 (Story 6.5)** — từ "BA bước" xuống "HAI bước":
//! [`Step::CleanByRules`] cũng không còn no-op, xem nhánh `match` của nó ngay dưới
//! ([`crate::core::cleanup::apply`]).
//! 🔵 **SỬA 2026-09-06 (Story 6.7) — từ "HAI bước" xuống "MỘT bước".** [`Step::ExtractMainContent`]
//! không còn no-op — mệnh đề "Story 6.9" ngay dưới đây từng đúng, ĐÃ HẾT ĐÚNG từ Story 6.7:
//! AD-16 buộc kéo thuật toán bóc lên trước màn xem trước bắt buộc (§Design Notes spec 6.7),
//! sớm hơn một story so với dự kiến ban đầu của `epics.md`. Story 6.9 (tiếp) đổi chữ ký
//! `extract` từ trả `String` phẳng sang trả `Vec<webimport::Block>` (mô hình khối giữ/loại) —
//! xem [`join_kept_blocks`]/[`effective_kept_for_blocks`] ngay dưới bước 5 — nhưng đó là một
//! thay đổi HÌNH DẠNG của một bước ĐÃ có thân, không phải bước đó CHUYỂN từ rỗng sang có thân.
//! [`Step::Preview`] (Story 6.9) — bước CÒN LẠI là no-op trong `match` của
//! [`run_import_with_order`]. Nó CÓ MẶT trong [`PIPELINE_ORDER`] và trong
//! [`PipelineOutput::trace`] của MỌI lượt chạy — một bước thân rỗng vẫn phải NÓI ĐƯỢC là đã
//! đi qua, không được biến mất khỏi vết chạy chỉ vì nó không làm gì (AC6 của spec 6.2). Vết
//! chạy được ghi TỪ BÊN TRONG mỗi nhánh `match`, không phải một `trace.push` chung sau vòng
//! lặp — một `trace.push` chung phản ánh đúng `order` truyền vào, không phản ánh việc handler
//! có thật sự chạy hay không.
//!
//! [`Step::SplitChapters`] KHÔNG nằm trong bốn bước trên — xem doc-comment của nó.
//! 🔵 **SỬA 2026-09-05 (Story 6.6) — "KHÔNG PHẢI mẫu người dùng cấu hình được" đã HẾT
//! ĐÚNG.** Bản Story 6.2 chỉ so khớp chuỗi con literal và không có bề mặt nào đưa một mẫu
//! vào ([`PipelineInput::chapter_pattern`] luôn `None` trên đường sản phẩm). Từ Story 6.6,
//! màn xem trước nhập cấu hình được cả literal LẪN regex, và bước này tách theo VỊ TRÍ khớp
//! (giữ tiêu đề ở đầu Chương mới) — xem doc-comment [`split_chapters_step`] cho cơ chế thật.
//!
//! ─────────────────────────────────────────────────────────────────────────────
//! 🔴 CHỈ MỘT WRITER — KHÔNG `Store`/`Transaction` Ở ĐÂY
//! ─────────────────────────────────────────────────────────────────────────────
//! Module này thuần: không I/O, không SQL. `commands::project::create_work` chạy toàn bộ
//! [`run_import`] TRƯỚC khi mở giao dịch ghi — cùng lý do Quyết định #3 cũ của Story 1.15
//! (AD-11 giữ MỘT writer duy nhất nối tiếp; CPU trong closure ghi chặn MỌI lượt ghi khác).

use super::chapterpattern::ChapterPattern;
use super::import::{ImportError, ImportedChapter};
use super::normalize;
use super::split::{SplitSegment, split_source_text};
use crate::core::webimport::{Block, BlockBody};

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
    /// Bước 2 — bóc nội dung chính (`dom_smoothie`, Story 6.7). 🔵 **SỬA 2026-09-07 (Story
    /// 6.9)** — "THÂN RỖNG" hết đúng từ Story 6.7; từ story này thân trả về mô hình khối
    /// giữ/loại (`webimport::Block`), không còn một `String` phẳng.
    ExtractMainContent,
    /// Bước 3 — làm sạch theo luật ([`crate::core::cleanup::apply`], Story 6.5, FR124).
    /// 🔵 SỬA 2026-09-05 — KHÔNG còn thân rỗng.
    CleanByRules,
    /// Bước 4 — chuẩn hoá đoạn & khoảng trắng ([`normalize::normalize`], Story 6.4,
    /// FR124/FR125). 🔵 SỬA 2026-09-04 — KHÔNG còn thân rỗng.
    NormalizeParagraphsAndWhitespace,
    /// Bước 5 — tách Chương theo mẫu phân tách. 🔵 SỬA 2026-09-05 (Story 6.6) — mẫu NGƯỜI
    /// DÙNG cấu hình được (literal hoặc regex), tách theo VỊ TRÍ khớp — xem doc-comment
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
/// `core::segment::import` cho file (`.txt`/`.md`/`.docx`, qua `core::docx` cho định dạng
/// cuối — Story 6.12)/dán tay; `webimport` cho URL. 🔵 **SỬA 2026-09-09 (Story 6.12)** —
/// câu cũ dự đoán bước 0 của `.docx` sẽ sống ở `webimport`/`export`; nó sống ở
/// `core::segment::import` (gọi `core::docx::read_docx`), module RIÊNG không phải hai module
/// đó (AD-39 `:498` — "chỉ cung cấp bước đầu vào, không giữ bản sao của các bước dùng
/// chung").
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
    ///
    /// 🔵 **SỬA 2026-09-07 (Story 6.7) — trường này là MỘT bảng mã cho **CẢ** danh sách, và
    /// điều đó là một QUYẾT ĐỊNH, không phải một chỗ chưa làm.** Với
    /// [`PipelineShape::Chapters`] mang N đơn vị (đường nhập URL, N link ⇒ N Chương),
    /// `Step::DecodeEncoding` gọi `decode_unit(u, encoding)` với **cùng một** giá trị này cho
    /// **mọi** đơn vị; bảng mã ấy được chốt từ đơn vị **ĐẦU TIÊN** ở tầng lệnh
    /// (`commands::project::preview_import_encoding`, nhánh `PipelineShape::Chapters` dò trên
    /// `chapters.first()`). Ice chốt 2026-09-06, đường ② của mục nợ *"`PipelineShape::Chapters`
    /// chọn MỘT bảng mã"* trong `deferred-work.md`.
    ///
    /// ⚠️ **Ca còn hở, ghi ra thay vì để người sau tưởng đã xét:** link A trả GBK còn link B
    /// trả UTF-8 thì B bị giải mã bằng bảng mã của A — hoặc ra chữ rác một cách lặng lẽ (nếu
    /// byte của B vẫn giải mã được qua GBK, không `Malformed`), hoặc trượt
    /// [`ImportError::UndecodableBytes`] với một lý do **không nói đúng nguyên nhân** (bảng mã
    /// SAI cho đúng đơn vị đó, không phải "byte hỏng"). Đổi lại: 0 dòng mã pipeline đổi, đường
    /// [`PipelineShape::Blob`] không bị chạm, và một dải năm ứng viên duy nhất đúng như mockup
    /// vẽ. Đường ① (dò ĐỘC LẬP từng đơn vị, mỗi đơn vị một dải ứng viên và một bảng mã riêng)
    /// vẫn MỞ cho một story sau — nó là một thay đổi hình dạng của trường này, không phải một
    /// bản vá tại chỗ.
    pub encoding: &'static encoding_rs::Encoding,
    /// Mẫu phân tách của [`Step::SplitChapters`] — literal HOẶC regex, tham số MỖI LƯỢT
    /// NHẬP (Story 6.6, FR14). `None` ⇒ bước 5 là no-op, giữ N = 1.
    ///
    /// 🔵 **SỬA 2026-09-05 (Story 6.6) — kiểu đổi từ `Option<String>` sang
    /// `Option<ChapterPattern>`.** Bản Story 6.2 chỉ khai được chuỗi con literal; từ story
    /// này người dùng cấu hình được cả hình dạng regex ngay trên màn xem trước nhập — xem
    /// [`super::chapterpattern::ChapterPattern`].
    pub chapter_pattern: Option<ChapterPattern>,
    /// `work.source_lang` — bước 7 ([`split_source_text`]) rẽ nhánh Trung/Anh theo trường
    /// này (AD-18: không đoán từ nội dung).
    pub source_lang: String,
    /// **THÊM 2026-09-05 (Story 6.5)** — luật làm sạch ĐÃ PHÂN GIẢI (hai tầng đã hợp nhất
    /// qua `ScopeResolver::apply_merge` ở chỗ gọi, xem
    /// `core::cleanup::store::resolve_two_tiers`) cho [`Step::CleanByRules`] (bước 3).
    /// Rỗng ⇒ bước 3 là no-op thật (không luật nào để mà áp — khác "thân rỗng theo thiết
    /// kế" của bản Story 6.2, đây là "0 luật vì người dùng chưa soạn", Ice chốt xuất xưởng
    /// 0 luật mặc định, spec 6.5 §Intent).
    pub cleanup_rules: Vec<crate::core::cleanup::CleanupRule>,
    /// **THÊM 2026-09-06 (Story 6.7)** — cờ MỘT chiều cho [`Step::ExtractMainContent`] (bước
    /// 2). `false` (mặc định — [`Self::default_shaped`]/[`Self::with_encoding`] đều đặt
    /// `false`) ⇒ bước 2 GIỮ NGUYÊN thân rỗng như mọi story trước (đường tệp/dán tay KHÔNG
    /// bị `dom_smoothie` chạm tới — §Always spec 6.7: "đường tệp/dán tay không bị bóc").
    /// `true` (chỉ đường nhập URL, qua [`Self::with_extract_main_content`]) ⇒ bước 2 gọi
    /// xuống [`crate::core::webimport::extract`] cho từng đơn vị `Unit::Decoded`, dùng
    /// `label` của CHÍNH đơn vị đó (xem [`Flow::labels`]) làm URL để `dom_smoothie` phân
    /// giải đường dẫn tương đối bên trong tài liệu.
    pub extract_main_content: bool,
    /// **THÊM 2026-09-07 (Story 6.9)** — trạng thái giữ/loại người dùng đã ĐẶT BẰNG TAY cho
    /// khối của Chương ĐẦU TIÊN (`units[0]`, giới hạn `tier2_url_first_note` — §Never spec
    /// 6.9), theo INDEX trong `Vec<webimport::Block>` mà [`Step::ExtractMainContent`] vừa
    /// dựng cho đơn vị đó. `overrides[i] == Some(v)` ⇒ khối `i` GIỮ (`v == true`) hay LOẠI
    /// (`v == false`) bất kể `Block::machine_kept` nói gì; `None`/ngoài phạm vi ⇒ dùng
    /// `machine_kept`. RỖNG (mặc định, [`Self::default_shaped`]/[`Self::with_encoding`]) ⇒
    /// mọi khối dùng nguyên `machine_kept` — đúng hành vi "chưa ai sửa gì".
    ///
    /// 🔴 Chỉ `units[0]` đọc trường này (`Step::ExtractMainContent` áp nó cho phần tử ĐẦU
    /// TIÊN của `old_units`, các phần tử sau dùng nguyên `machine_kept`) — hai lý do: ①
    /// `Tier2BlockOverridesState` phía `commands::project` dùng CHUNG một vector cho cả năm
    /// ứng viên bảng mã (Quyết định #2, §Spec Change Log spec 6.9), không tách theo Chương;
    /// ② tầng 2 chưa mở rộng ra ngoài Chương đầu tiên (§Never spec 6.9).
    pub block_overrides: Vec<Option<bool>>,
}

impl PipelineInput {
    /// Cấu hình MẶC ĐỊNH của đường sản phẩm hôm nay — UTF-8, không mẫu phân tách, không
    /// luật làm sạch. Xem doc-comment các trường `encoding`/`chapter_pattern` ở trên cho
    /// lý do.
    pub fn default_shaped(shape: PipelineShape, source_lang: impl Into<String>) -> Self {
        PipelineInput {
            shape,
            encoding: encoding_rs::UTF_8,
            chapter_pattern: None,
            source_lang: source_lang.into(),
            cleanup_rules: Vec::new(),
            extract_main_content: false,
            block_overrides: Vec::new(),
        }
    }

    /// **THÊM 2026-09-04 (Story 6.3)** — cấu hình mang một bảng mã ĐÃ CHỌN, đứng CẠNH
    /// [`PipelineInput::default_shaped`] (§Always spec 6.3: không sửa/xoá constructor cũ).
    /// Đường sản phẩm dùng hàm này khi người dùng đã xác nhận một ứng viên ở màn xem trước
    /// bảng mã (`commands::project::confirm_import_with_encoding`) — `chapter_pattern` vẫn
    /// `None` (Never clause của spec 6.2/6.3: mẫu phân tách người dùng cấu hình được là
    /// Story 6.6, không thuộc phạm vi này). `cleanup_rules` rỗng — dùng
    /// [`PipelineInput::with_cleanup_rules`] để đính luật vào SAU khi dựng.
    pub fn with_encoding(
        shape: PipelineShape,
        encoding: &'static encoding_rs::Encoding,
        source_lang: impl Into<String>,
    ) -> Self {
        PipelineInput {
            shape,
            encoding,
            chapter_pattern: None,
            source_lang: source_lang.into(),
            cleanup_rules: Vec::new(),
            extract_main_content: false,
            block_overrides: Vec::new(),
        }
    }

    /// **THÊM 2026-09-05 (Story 6.5)** — builder đính `cleanup_rules` vào một cấu hình đã
    /// dựng, đứng CẠNH hai constructor trên (không sửa/xoá chúng — §Always spec 6.5 lặp lại
    /// đúng luật mà Story 6.3 đã theo cho `with_encoding`).
    #[must_use]
    pub fn with_cleanup_rules(mut self, rules: Vec<crate::core::cleanup::CleanupRule>) -> Self {
        self.cleanup_rules = rules;
        self
    }

    /// **THÊM 2026-09-05 (Story 6.6)** — builder đính `chapter_pattern` vào một cấu hình đã
    /// dựng, cùng khuôn [`Self::with_cleanup_rules`] (không sửa/xoá hai constructor cũ —
    /// §Always spec 6.6 lặp lại đúng luật mà Story 6.3/6.5 đã theo).
    #[must_use]
    pub fn with_chapter_pattern(mut self, pattern: Option<ChapterPattern>) -> Self {
        self.chapter_pattern = pattern;
        self
    }

    /// **THÊM 2026-09-06 (Story 6.7)** — builder đính `extract_main_content` vào một cấu
    /// hình đã dựng, cùng khuôn [`Self::with_cleanup_rules`]/[`Self::with_chapter_pattern`]
    /// (không sửa/xoá hai constructor cũ). Chỉ đường nhập URL (`commands::project`) gọi hàm
    /// này với `true`; mọi chỗ gọi khác (kể cả `tests/**` không cố ý dựng đối chứng cho bước
    /// 2) giữ mặc định `false`.
    #[must_use]
    pub fn with_extract_main_content(mut self, extract: bool) -> Self {
        self.extract_main_content = extract;
        self
    }

    /// **THÊM 2026-09-07 (Story 6.9)** — builder đính `block_overrides` vào một cấu hình đã
    /// dựng, cùng khuôn ba builder trên (không sửa/xoá constructor cũ). Chỉ `units[0]` đọc
    /// trường này — xem doc-comment [`Self::block_overrides`].
    #[must_use]
    pub fn with_block_overrides(mut self, overrides: Vec<Option<bool>>) -> Self {
        self.block_overrides = overrides;
        self
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
            .field("cleanup_rules", &self.cleanup_rules)
            .field("extract_main_content", &self.extract_main_content)
            .field("block_overrides", &self.block_overrides)
            .finish()
    }
}

/// Kết quả một lượt chạy chuỗi — N Chương (N = 1 khi không có mẫu phân tách; 🔵 SỬA
/// 2026-09-05, Story 6.6 — N > 1 là một kết quả THẬT trên đường sản phẩm khi
/// `chapter_pattern` khớp được nhiều lần), cộng vết chạy.
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
    /// **THÊM 2026-09-05 (Story 6.5)** — báo cáo [`Step::CleanByRules`] cho từng phần tử
    /// `units`, SONG SONG theo INDEX — cùng khuôn `segments`. `None` = chưa có báo cáo (bước
    /// 3 chưa chạy, hoặc đơn vị này là `Unit::Undecoded` khi bước 3 chạy). Reset về
    /// `vec![None; n]` khi [`split_chapters_step`] THẬT SỰ đổi số phần tử — cùng lý do
    /// `segments` reset ở đó: một báo cáo tính cho ĐƠN VỊ TRƯỚC khi tách không còn khớp
    /// INDEX nào có nghĩa sau khi tách.
    cleanup_reports: Vec<Option<crate::core::cleanup::CleanupReport>>,
    /// **THÊM 2026-09-05 (Story 6.6)** — tiêu đề của từng phần tử `units`, SONG SONG theo
    /// INDEX — cùng khuôn `segments`/`cleanup_reports`. `None` = không có tiêu đề (Chương
    /// lời tựa trước khớp đầu tiên, hoặc mẫu không khớp gì/không được cấu hình). Reset về
    /// `vec![None; n]` khi [`split_chapters_step`] THẬT SỰ đổi số phần tử — cùng lý do
    /// `segments`/`cleanup_reports` reset ở đó.
    chapter_titles: Vec<Option<String>>,
    /// **THÊM 2026-09-06 (Story 6.7)** — nhãn (URL, cho đơn vị đến từ danh sách nhập URL;
    /// rỗng cho mọi nguồn khác) của từng phần tử `units`, SONG SONG theo INDEX — cùng khuôn
    /// `segments`/`cleanup_reports`/`chapter_titles`. [`Step::ExtractMainContent`] đọc
    /// trường này để biết URL nào truyền cho `dom_smoothie` phân giải đường dẫn tương đối.
    /// Reset về `vec![String::new(); n]` khi [`split_chapters_step`] THẬT SỰ đổi số phần tử
    /// — cùng lý do ba trường kia reset ở đó (một nhãn tính cho ĐƠN VỊ TRƯỚC khi tách không
    /// còn khớp INDEX nào có nghĩa sau khi tách). Trong thực tế điều này không xảy ra trên
    /// đường sản phẩm: `extract_main_content = true` chỉ đi cùng `PipelineShape::Chapters`
    /// (đã chia Chương, bước 5 bỏ qua — xem `Flow::already_chaptered`), nên `labels` không
    /// bao giờ bị reset trên đường đó; giữ đúng khuôn cho MỌI thứ tự hợp lệ mà `tests/**`
    /// dựng được (kể cả một hoán vị đặt `SplitChapters` chạy trên `Blob`).
    labels: Vec<String>,
    /// **THÊM 2026-09-07 (Story 6.9)** — mô hình khối CẢ TRANG mà [`Step::ExtractMainContent`]
    /// vừa dựng cho từng phần tử `units`, SONG SONG theo INDEX — cùng khuôn `labels`. `None` =
    /// bước 2 không chạy cho đơn vị này (`extract_main_content == false`, đường tệp/dán tay).
    /// Đây là dữ liệu THÔ (`Block::machine_kept`, chưa áp `PipelineInput::block_overrides`) —
    /// `commands::project` tự áp lại overrides khi dựng dây, dùng ĐÚNG hàm
    /// [`effective_kept_for_blocks`] mà chính bước này gọi để GHÉP văn bản, để hai nơi không
    /// lệch nhau. Reset về `vec![None; n]` khi [`split_chapters_step`] THẬT SỰ đổi số phần tử
    /// — cùng lý do `labels` reset ở đó (không xảy ra trên đường sản phẩm, cùng lý lẽ).
    blocks: Vec<Option<Vec<crate::core::webimport::Block>>>,
    /// **THÊM 2026-09-08 (Story 6.10)** — số LẦN [`normalize::normalize`] đã NỐI hai dòng làm
    /// một cho từng phần tử `units`, SONG SONG theo INDEX — cùng khuôn `cleanup_reports`.
    /// Được GÁN tại [`Step::NormalizeParagraphsAndWhitespace`] (bước 4), đọc từ
    /// [`normalize::Normalized::joined_lines`] mà bước đó ĐÃ TÍNH RỒI VỨT trước bản sửa này
    /// (`pipeline.rs:644` trước đây chỉ giữ `.text`) — xem doc-comment `Step::NormalizeParagraphsAndWhitespace`
    /// nhánh `match` ngay dưới cho cách con số này được gán.
    ///
    /// 🔴 **RESET VỀ `None` TOÀN BỘ khi [`split_chapters_step`] THẬT SỰ đổi số phần tử — KHÁC
    /// `cleanup_reports`, KHÔNG gắn báo cáo của toàn blob vào Chương ĐẦU.** Trên đường `Blob`,
    /// bước 4 (chuẩn hoá) chạy TRƯỚC bước 5 (tách Chương) trong [`PIPELINE_ORDER`] — lúc bước 4
    /// chạy chỉ có MỘT đơn vị là TOÀN TÀI LIỆU, nên con số nối dòng đo được thuộc về TÀI LIỆU,
    /// không thuộc về Chương nào. Gán nó cho Chương 1 (như `cleanup_reports` làm) sẽ cho Chương
    /// 1 một số cao vô lý — nó gánh cả N-1 Chương kia — và đẩy chính nó qua hàng rào Tukey một
    /// cách oan uổng (§Design Notes spec 6.10 "Vì sao `Blob` không cho `ord = 1` con số FR125,
    /// trong khi `cleanup` thì có"). Trên [`PipelineShape::Chapters`] (đã chia Chương từ đầu,
    /// `already_chaptered = true`, [`split_chapters_step`] return sớm, KHÔNG đụng trường này),
    /// con số của mỗi Chương là THẬT (bước 4 chạy trên TỪNG đơn vị riêng).
    joined_line_counts: Vec<Option<usize>>,
}

/// Nhãn chẩn đoán của một [`ChapterInput`] — `RawBytes::label` nếu có (byte thô CHƯA giải
/// mã, dấu hiệu THẬT của "đến từ mạng"), chuỗi rỗng cho `AlreadyText` (không có nhãn để mà
/// giữ, `Unit::Decoded` không giữ lại nó).
///
/// 🔵 **NÂNG LÊN module-scope, `pub(crate)` (vòng rà đối kháng 2, mục F1).** Trước đây hàm
/// này SỐNG BÊN TRONG `run_import_with_order` (một closure lồng, riêng tư), và
/// `commands::project::chapter_input_page_url` giữ một BẢN CHÉP TAY 4 dòng y hệt (không mở
/// được một chỗ gọi vào một hàm lồng riêng tư của module khác) — một cổng test riêng
/// (`chapter_page_url_drift_boundary.rs`) phải canh hai bản đó không trôi khỏi nhau. Nâng
/// hàm này lên `pub(crate)` xoá bản chép, xoá cổng canh trôi, và xoá luôn CHÍNH nguy cơ trôi
/// (một nguồn sự thật duy nhất) — rẻ hơn hẳn việc canh hai bản đồng bộ mãi mãi.
pub(crate) fn label_of(c: &ChapterInput) -> String {
    match c {
        ChapterInput::RawBytes { label, .. } => label.clone(),
        ChapterInput::AlreadyText(_) => String::new(),
    }
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

    let PipelineInput {
        shape,
        encoding,
        chapter_pattern,
        source_lang,
        cleanup_rules,
        extract_main_content,
        block_overrides,
    } = input;

    // `labels` phải được đọc TRƯỚC khi `ChapterInput` bị `Unit::from` tiêu thụ —
    // `Unit::Decoded` (nhánh `AlreadyText`) không giữ lại nhãn, nên đây là nơi DUY NHẤT còn
    // thấy nó cho cả hai hình dạng đơn vị.
    let (initial_units, initial_labels, already_chaptered): (Vec<Unit>, Vec<String>, bool) = match shape {
        PipelineShape::Blob(c) => {
            let label = label_of(&c);
            (vec![Unit::from(c)], vec![label], false)
        }
        PipelineShape::Chapters(cs) => {
            let labels: Vec<String> = cs.iter().map(label_of).collect();
            (cs.into_iter().map(Unit::from).collect(), labels, true)
        }
    };
    let n = initial_units.len();
    let mut flow = Flow {
        units: initial_units,
        segments: vec![None; n],
        already_chaptered,
        cleanup_reports: vec![None; n],
        chapter_titles: vec![None; n],
        labels: initial_labels,
        blocks: vec![None; n],
        joined_line_counts: vec![None; n],
    };

    let mut trace: Vec<Step> = Vec::with_capacity(order.len());
    for &step in order {
        flow = match step {
            Step::DecodeEncoding => {
                let Flow { units: old_units, segments, already_chaptered, cleanup_reports, chapter_titles, labels, blocks, joined_line_counts } =
                    flow;
                let mut units = Vec::with_capacity(old_units.len());
                for u in old_units {
                    units.push(decode_unit(u, encoding)?);
                }
                trace.push(step);
                Flow { units, segments, already_chaptered, cleanup_reports, chapter_titles, labels, blocks, joined_line_counts }
            }
            // 🔴 THÂN THẬT — Story 6.7 (bóc), Story 6.9 (mô hình khối + trạng thái sửa tay),
            // AD-39 bước 2. `extract_main_content == false` (đường tệp/dán tay — §Always spec
            // 6.7) GIỮ NGUYÊN thân rỗng như mọi story trước; chỉ `true` (đường URL) mới GỌI
            // XUỐNG `webimport::extract`, không viết lại nội tuyến (cùng luật "gọi xuống, đừng
            // chép lại" của bước 3/bước 4). `trace.push` Ở LẠI BÊN TRONG nhánh (AC6 spec 6.2,
            // doc-comment đầu tệp).
            //
            // 🔴 **SỬA 2026-09-07 (Story 6.9) — `extract` nay trả `Vec<Block>`, không `String`
            // phẳng.** Văn bản ghi xuống là kết quả GHÉP các khối ĐANG GIỮ (xem
            // [`join_kept_blocks`]) — "giữ" là `Block::machine_kept`, TRỪ khi
            // [`PipelineInput::block_overrides`] ghi đè cho ĐÚNG đơn vị ĐẦU TIÊN (`index ==
            // 0` — giới hạn "chỉ Chương đầu tiên", §Never spec 6.9; xem doc-comment trường
            // đó). Mô hình khối THÔ (chưa áp override) được giữ lại ở `Flow::blocks` để
            // `commands::project` dựng lại dây tầng 2 — CÙNG hàm [`effective_kept_for_blocks`]
            // áp override ở CẢ HAI nơi, để văn bản ghi xuống và dây hiển thị không lệch nhau.
            Step::ExtractMainContent => {
                if !extract_main_content {
                    trace.push(step);
                    flow
                } else {
                    let Flow { units: old_units, segments, already_chaptered, cleanup_reports, chapter_titles, labels, blocks: _, joined_line_counts } =
                        flow;
                    let mut units = Vec::with_capacity(old_units.len());
                    let mut blocks: Vec<Option<Vec<crate::core::webimport::Block>>> =
                        Vec::with_capacity(old_units.len());
                    for (index, (u, label)) in old_units.into_iter().zip(labels.iter()).enumerate() {
                        match u {
                            Unit::Decoded(html) => {
                                let page_blocks = crate::core::webimport::extract(&html, label)
                                    .map_err(|e| ImportError::WebImportItemFailed {
                                        url: label.clone(),
                                        reason:
                                            crate::core::webimport::WebImportItemFailureReason::ExtractionEmpty,
                                        detail: e.detail,
                                    })?;
                                // Chỉ đơn vị ĐẦU TIÊN đọc `block_overrides` — xem doc-comment
                                // `PipelineInput::block_overrides`.
                                let overrides_for_unit: &[Option<bool>] =
                                    if index == 0 { &block_overrides } else { &[] };
                                let effective_kept =
                                    effective_kept_for_blocks(&page_blocks, overrides_for_unit);
                                let joined = join_kept_blocks(&page_blocks, &effective_kept);
                                units.push(Unit::Decoded(joined));
                                blocks.push(Some(page_blocks));
                            }
                            // Bất khả trên mọi thứ tự HỢP LỆ (bước 1 luôn đứng trước bước 2)
                            // — giữ nguyên là phòng thủ cho một thứ tự SAI, cùng khuôn các
                            // nhánh `Unit::Undecoded` khác trong hàm này.
                            other @ Unit::Undecoded { .. } => {
                                units.push(other);
                                blocks.push(None);
                            }
                        }
                    }
                    trace.push(step);
                    Flow { units, segments, already_chaptered, cleanup_reports, chapter_titles, labels, blocks, joined_line_counts }
                }
            }
            // 🔴 THÂN THẬT — Story 6.5, FR124, AD-39 bước 3. GỌI `core::cleanup::apply`,
            // không viết lại nội tuyến (cùng luật "gọi xuống, đừng chép lại" mà bước 4 —
            // Story 6.4 — đã theo cho `normalize::normalize`). `trace.push` Ở LẠI BÊN TRONG
            // nhánh (AC6 spec 6.2, doc-comment đầu tệp).
            Step::CleanByRules => {
                let Flow {
                    units: old_units,
                    segments,
                    already_chaptered,
                    cleanup_reports: _,
                    chapter_titles,
                    labels,
                    blocks,
                    joined_line_counts,
                } = flow;
                let mut units = Vec::with_capacity(old_units.len());
                let mut cleanup_reports = Vec::with_capacity(old_units.len());
                for u in old_units {
                    match u {
                        Unit::Decoded(text) => {
                            let cleaned = crate::core::cleanup::apply(&text, &cleanup_rules)
                                .map_err(|e| ImportError::InvalidCleanupPattern {
                                    detail: e.to_string(),
                                })?;
                            cleanup_reports.push(Some(crate::core::cleanup::CleanupReport {
                                matches: cleaned.matches,
                                per_rule_counts: cleaned.per_rule_counts,
                            }));
                            units.push(Unit::Decoded(cleaned.text));
                        }
                        // `Unit::Undecoded` ở bước này là BẤT KHẢ trên mọi thứ tự HỢP LỆ
                        // (bước 1 luôn đứng trước bước 3) — giữ nguyên là phòng thủ cho một
                        // thứ tự SAI, cùng khuôn `NormalizeParagraphsAndWhitespace` ngay dưới.
                        other @ Unit::Undecoded { .. } => {
                            cleanup_reports.push(None);
                            units.push(other);
                        }
                    }
                }
                trace.push(step);
                Flow { units, segments, already_chaptered, cleanup_reports, chapter_titles, labels, blocks, joined_line_counts }
            }
            // 🔴 THÂN THẬT — Story 6.4, FR124/FR125, AD-39 bước 4. GỌI `normalize::normalize`,
            // không viết lại nội tuyến (Task list spec 6.4) — mọi luật (bảng kết câu, bảng
            // nối theo ngôn ngữ) sống ở `split.rs`/`regroup.rs`, module này chỉ GỌI chúng.
            // `trace.push` Ở LẠI BÊN TRONG nhánh (AC6 spec 6.2, doc-comment đầu tệp) —
            // KHÔNG gộp vào một `trace.push` chung sau vòng lặp.
            //
            // 🔴 **SỬA 2026-09-08 (Story 6.10) — `joined_lines` KHÔNG còn bị VỨT.** Bản trước
            // chỉ giữ `.text` của [`normalize::Normalized`], ném thẳng `.joined_lines` — số
            // FR125 đã tính RỒI trên chính TOÀN VĂN từng đơn vị, tại đúng chỗ này. Nay số đó
            // được GÁN vào `Flow::joined_line_counts` cùng INDEX — xem doc-comment trường đó
            // cho lý do đây là con số THẬT trên [`PipelineShape::Chapters`] nhưng KHÔNG quy về
            // được Chương nào trên `Blob` (bị [`split_chapters_step`] reset về `None` ngay sau).
            Step::NormalizeParagraphsAndWhitespace => {
                let Flow { units: old_units, segments, already_chaptered, cleanup_reports, chapter_titles, labels, blocks, joined_line_counts: _ } =
                    flow;
                let mut units = Vec::with_capacity(old_units.len());
                let mut joined_line_counts = Vec::with_capacity(old_units.len());
                for u in old_units {
                    match u {
                        Unit::Decoded(text) => {
                            let normalized = normalize::normalize(&text, &source_lang);
                            joined_line_counts.push(Some(normalized.joined_lines));
                            units.push(Unit::Decoded(normalized.text));
                        }
                        // `Unit::Undecoded` ở bước này là BẤT KHẢ trên mọi thứ tự HỢP LỆ
                        // (`validate_order` đã kiểm — bước 1 luôn đứng trước bước 4). Giữ
                        // nguyên là phòng thủ cho một thứ tự SAI (đối chứng AD-39 đặt bước
                        // này TRƯỚC giải mã): `normalize` cần `&str`, không có nghĩa gì để
                        // chạy nó trên byte thô — cùng khuôn `split_segments_step` ngay
                        // dưới, cũng bỏ qua `Unit::Undecoded` vì cùng lý do. `None` ở đây là
                        // "không đo được", đúng nghĩa `Flow::joined_line_counts`.
                        other @ Unit::Undecoded { .. } => {
                            joined_line_counts.push(None);
                            units.push(other);
                        }
                    }
                }
                trace.push(step);
                Flow { units, segments, already_chaptered, cleanup_reports, chapter_titles, labels, blocks, joined_line_counts }
            }
            Step::SplitChapters => {
                let next = split_chapters_step(flow, chapter_pattern.as_ref())?;
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
        .zip(flow.cleanup_reports)
        .zip(flow.chapter_titles)
        .zip(flow.blocks)
        .zip(flow.joined_line_counts)
        .map(|(((((u, s), cleanup_report), title), blocks), joined_line_count)| -> Result<ImportedChapter, ImportError> {
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
            Ok(ImportedChapter {
                source_text,
                segments: s.unwrap_or_default(),
                cleanup_report,
                title,
                blocks,
                joined_line_count,
            })
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
// Bước 2 (tiếp) — Story 6.9: áp `block_overrides` rồi GHÉP văn bản từ khối ĐANG GIỮ
// ═════════════════════════════════════════════════════════════════════════════════

/// Trạng thái giữ HIỆU LỰC của từng khối trong `blocks` — `overrides[i]` thắng
/// `blocks[i].machine_kept` khi có mặt (`Some`), đúng thứ tự index; `overrides` ngắn hơn
/// `blocks` (kể cả rỗng — trường hợp phổ biến, "chưa ai sửa gì") là hợp lệ, phần còn lại dùng
/// nguyên `machine_kept`. **Hàm thuần, dùng CHUNG** bởi [`join_kept_blocks`] (bước ghép văn
/// bản, ở trên) VÀ `commands::project` (dựng dây tầng 2 hiển thị cho người dùng) — hai nơi đọc
/// override phải thấy CÙNG MỘT kết quả, nếu không màn hình sẽ hiện một trạng thái mà văn bản
/// ghi xuống không khớp.
pub fn effective_kept_for_blocks(blocks: &[Block], overrides: &[Option<bool>]) -> Vec<bool> {
    blocks
        .iter()
        .enumerate()
        .map(|(i, b)| overrides.get(i).copied().flatten().unwrap_or(b.machine_kept))
        .collect()
}

/// Ghép văn bản từ các khối ĐANG GIỮ (`effective_kept[i] == true`), theo ĐÚNG thứ tự tài
/// liệu — bỏ qua `BlockBody::Image` (không có "chữ" để ghép) và mọi thân rỗng.
///
/// ─────────────────────────────────────────────────────────────────────────────
/// KHUÔN NỐI — chính xác khi KHÔNG override (AC "trùng đúng Story 6.7"), best-effort khi CÓ
/// ─────────────────────────────────────────────────────────────────────────────
/// Hai khối liền nhau trong đầu ra chỉ được nối bằng `Block::exact_gap_before` (khoảng trắng
/// GỐC, byte-for-byte từ `text_content`) khi chúng CŨNG là một cặp `machine_kept` LIỀN KỀ
/// TRONG DÃY GỐC (không khối `machine_kept` nào khác đứng giữa chúng theo INDEX TUYỆT ĐỐI) —
/// đây là điều kiện DUY NHẤT khiến `exact_gap_before` có nghĩa (xem doc-comment trường đó ở
/// `extractor.rs`). Khi không ai sửa gì, MỌI cặp liền kề trong đầu ra đều thoả điều kiện này
/// theo cấu trúc (`effective_kept == machine_kept`), nên phép ghép tái tạo lại ĐÚNG TỪNG BYTE
/// `text_content` mà [`crate::core::webimport::extract`] đã đọc. Khi người dùng đã bấm
/// `Space`/`[`/`]` làm lệch cặp liền kề đó (loại một khối gốc đứng giữa, hoặc thêm một khối
/// gốc bị loại vào giữa), không còn khoảng trắng GỐC nào để mà đọc — dùng khuôn nối DỰ PHÒNG
/// (`Block::is_list_item`: một dòng đơn cho `<li>`, hai dòng cho mọi khối khác, đúng luật
/// `elem_require_linebreak`/`li` của `dom_query::node::text_formatting::format_text` mà
/// `TextMode::Formatted` dùng — xem doc-comment đầu `extractor.rs`). AC vòng rà chỉ đòi băng
/// qua đúng CA KHÔNG OVERRIDE; ca CÓ OVERRIDE chỉ cần Rust là nơi DUY NHẤT ghép (AD-1) và tự
/// nhất quán giữa xem trước và đĩa (AC "trùng từng byte" — cả hai gọi ĐÚNG hàm này).
pub fn join_kept_blocks(blocks: &[Block], effective_kept: &[bool]) -> String {
    let mut out = String::new();
    let mut last_machine_kept_idx: Option<usize> = None;
    let mut last_emitted_idx: Option<usize> = None;

    for (i, b) in blocks.iter().enumerate() {
        let text: Option<&str> = match &b.body {
            BlockBody::Paragraph(t) | BlockBody::Caption(t) => Some(t.as_str()),
            BlockBody::Image { .. } => None,
        };
        let is_kept = effective_kept.get(i).copied().unwrap_or(b.machine_kept);

        if is_kept {
            if let Some(t) = text {
                if !t.is_empty() {
                    if let Some(prev) = last_emitted_idx {
                        let exact_pair_is_original_neighbors =
                            b.machine_kept && last_machine_kept_idx == Some(prev);
                        if exact_pair_is_original_neighbors {
                            out.push_str(&b.exact_gap_before);
                        } else if blocks[prev].is_list_item {
                            out.push('\n');
                        } else {
                            out.push_str("\n\n");
                        }
                    }
                    out.push_str(t);
                    last_emitted_idx = Some(i);
                }
            }
        }

        if b.machine_kept && text.is_some() {
            last_machine_kept_idx = Some(i);
        }
    }

    out
}

// ═════════════════════════════════════════════════════════════════════════════════
// Bước 5 — tách Chương theo mẫu phân tách
// ═════════════════════════════════════════════════════════════════════════════════

/// Bước 5.
///
/// ─────────────────────────────────────────────────────────────────────────────
/// 🔵 **SỬA 2026-09-05 (Story 6.6) — mẫu người dùng cấu hình được, tách theo VỊ TRÍ.**
/// ─────────────────────────────────────────────────────────────────────────────
/// Trước story này bước 5 chỉ là một cơ chế tối thiểu (so khớp chuỗi con literal qua
/// `str::split`, KHÔNG cấu hình được bởi người dùng) — đủ để chứng minh THỨ TỰ có ý nghĩa
/// thật (AC5 spec 6.2), không hơn. Từ story này [`PipelineInput::chapter_pattern`] mang một
/// [`ChapterPattern`] THẬT (literal HOẶC regex), và nhánh `Unit::Decoded` tách theo **VỊ TRÍ
/// KHỚP** — Chương thứ *i* là dải nửa-mở `[start_i, start_{i+1})` — thay vì `str::split`:
/// `split` XOÁ dấu phân tách khỏi mọi mảnh, nên một mẫu TIÊU ĐỀ cấu hình được sẽ cho ra N
/// Chương mà KHÔNG Chương nào còn tiêu đề — khuyết tật ngữ nghĩa mà §Design Notes spec 6.6
/// "Vì sao tách theo VỊ TRÍ" gọi tên. Dòng khớp ở lại NGUYÊN VẸN ở ĐẦU Chương mới, và
/// [`title_line_of`] đọc `title` từ chính dòng đó.
///
/// Văn bản TRƯỚC khớp đầu tiên (lời tựa, mục lục) LUÔN LUÔN giữ làm Chương `ord = 1` với
/// `title = None` — KHÔNG BAO GIỜ bị vứt (§Always spec 6.6: mất byte im lặng bị cấm).
///
/// ─────────────────────────────────────────────────────────────────────────────
/// 🔴 CHỖ TRIỆU CHỨNG AD-39 SỐNG — nhánh `Unit::Undecoded`, GIỮ NGUYÊN literal-byte thuần
/// ─────────────────────────────────────────────────────────────────────────────
/// Khi bước này chạy TRƯỚC [`Step::DecodeEncoding`] (một thứ tự SAI), đơn vị đang chảy vẫn
/// là byte thô theo bảng mã đã khai (ví dụ GBK). Với mẫu `Literal`, mẫu phân tách là một
/// `&str` Rust — bao giờ cũng là byte UTF-8; tìm CHUỖI BYTE UTF-8 của mẫu bên trong byte GBK:
/// byte GBK của một chữ Hán không trùng byte UTF-8 của CHÍNH chữ đó (hai bảng mã khác nhau ở
/// tầng byte), nên phép tìm trả 0 khớp một cách TỰ NHIÊN — không ép, không bắt lỗi giả. Kết
/// quả: một mảnh DUY NHẤT, y hệt input, KHÔNG NÉM LỖI NÀO — đúng câu spine `:470`. Với mẫu
/// `Regex`, một regex engine trên `&[u8]` CÓ THỂ khớp được ở đây (nó không biết gì về bảng
/// mã) và sẽ PHÁ đúng cặp đối chứng AD-39 — vì vậy nhánh này KHÔNG bao giờ biên dịch/chạy một
/// regex trên byte thô: mẫu `Regex` gặp `Unit::Undecoded` LUÔN cho 0 khớp, cùng ngữ nghĩa với
/// `Literal` không khớp (§Always spec 6.6). Nếu bước này chạy SAU khi đã giải mã đúng, mẫu
/// khớp trên văn bản đã giải mã đúng và tách ra nhiều Chương thật — khác biệt QUAN SÁT ĐƯỢC
/// đó là đối chứng cho AC5 (spine `:498`, §Design Notes spec 6.2).
///
/// Chỉ chạm hình dạng [`PipelineShape::Blob`] — rẽ theo [`Flow::already_chaptered`] (HÌNH
/// DẠNG khai báo), KHÔNG suy từ `units.len()` (vòng rà đối kháng 2026-09-04: một
/// [`PipelineShape::Chapters`] với ĐÚNG MỘT phần tử có cùng độ dài quan sát được với một
/// `Blob` chưa tách, nhưng phải BỎ QUA — đúng bảng hình dạng AD-39, spine `:486-491`).
fn split_chapters_step(mut flow: Flow, pattern: Option<&ChapterPattern>) -> Result<Flow, ImportError> {
    if flow.already_chaptered {
        return Ok(flow);
    }
    let Some(pattern) = pattern else {
        return Ok(flow);
    };
    // `extract_main_content = true` chỉ đi cùng `PipelineShape::Chapters` (đã chia Chương,
    // return sớm ở nhánh `already_chaptered` ngay trên) — nhánh này không bao giờ chạm
    // `labels` mang URL thật trên đường sản phẩm; xử SỚM để phần bên dưới không cần bận
    // tâm giữ nhãn cho một phần tử duy nhất trước khi tách.

    // Bất biến: hình dạng `Blob` khởi tạo ĐÚNG MỘT đơn vị, và không bước nào TRƯỚC bước này
    // (trong bất kỳ hoán vị HỢP LỆ nào — `validate_order` đã kiểm ở đầu
    // `run_import_with_order`) làm tăng số đơn vị. `Vec::pop` không panic trên Vec rỗng (trả
    // `None`) — an toàn hơn `.into_iter().next().expect(..)` của bản trước (vòng rà đối
    // kháng 2026-09-04, item 4: panic trên đường ghi/tính pipeline không có kiểu).
    let Some(unit) = flow.units.pop() else {
        return Ok(flow); // rỗng bất thường — không có gì để tách, giữ nguyên trạng thái rỗng
    };

    let pieces: Vec<(Unit, Option<String>)> = match unit {
        Unit::Decoded(text) => split_on_positions(&text, pattern)?,
        Unit::Undecoded { bytes, label } => match pattern.kind {
            // Nhánh AD-39 — GIỮ NGUYÊN literal-byte thuần, không đụng tới (§Always spec 6.6).
            super::chapterpattern::ChapterPatternKind::Literal => {
                split_on_literal_bytes(&bytes, pattern.pattern.as_bytes())
                    .into_iter()
                    .map(|b| (Unit::Undecoded { bytes: b.to_vec(), label: label.clone() }, None))
                    .collect()
            }
            // Regex trên byte CHƯA giải mã luôn 0 khớp — không biên dịch/chạy gì cả, cùng
            // ngữ nghĩa AD-39 với nhánh Literal không khớp ở trên.
            super::chapterpattern::ChapterPatternKind::Regex => {
                vec![(Unit::Undecoded { bytes, label }, None)]
            }
        },
    };

    let n = pieces.len();
    let (units, titles): (Vec<Unit>, Vec<Option<String>>) = pieces.into_iter().unzip();
    flow.units = units;
    // Reset — số phần tử vừa đổi, một `Some(..)` cũ (nếu có, từ một thứ tự bị đảo NGOÀI
    // phạm vi đối chứng chính thức) không còn khớp INDEX nào có nghĩa.
    flow.segments = vec![None; n];
    // 🔴 **SỬA 2026-09-05 (Story 6.6) — KHÔNG còn vứt báo cáo của luật làm sạch.**
    //
    // ─────────────────────────────────────────────────────────────────────────────
    // 🔴 VÌ SAO GẮN LẠI VÀO CHƯƠNG ĐẦU, KHÔNG PHÂN BỔ THEO TỪNG CHƯƠNG
    // ─────────────────────────────────────────────────────────────────────────────
    // Bản Story 6.5 reset thẳng về `vec![None; n]` — đúng khi N luôn là 1 (mẫu phân tách
    // chưa cấu hình được), nhưng SAI khi N > 1 THẬT SỰ xảy ra (Story 6.6): một luật CÓ khớp
    // sẽ hiện `count_in_chapter = 0` — một lời NÓI DỐI tệ hơn cả "yếu", đúng lớp lỗi rỗng-im-
    // lặng mà AGENTS.md gọi tên là trung tâm của dự án. Không có cách nào phân bổ ĐÚNG báo
    // cáo (tính trên văn bản TRƯỚC bước 4 chuẩn hoá) theo ranh giới N Chương (tính SAU bước 4)
    // mà không có một cơ chế theo dõi vị trí xuyên bước chuẩn hoá — cơ chế đó CHƯA TỒN TẠI, và
    // dựng nó không phải phạm vi story này (`commands::project::cleanup_and_chapters_preview_for`
    // ghi rõ giới hạn này). Gắn NGUYÊN VẸN báo cáo (đo trên TOÀN blob) vào Chương ĐẦU —
    // `count_in_import` (tổng qua N Chương, tính ở `commands::project`) và `count_in_chapter`
    // (đọc từ Chương đầu) vì thế bằng nhau khi N Chương đến từ MẪU PHÂN TÁCH — con số ĐÚNG
    // (không bịa), chỉ không CHI TIẾT theo Chương. Khác `PipelineShape::Chapters` (N đơn vị
    // NGAY TỪ ĐẦU, KHÔNG đi qua hàm này — `already_chaptered` return sớm ở đầu hàm): ở đó bước
    // 3 lặp `apply` một lần MỖI ĐƠN VỊ nên mỗi Chương đã có báo cáo THẬT của riêng nó, hai số
    // trên THỰC SỰ khác nhau khi có ý nghĩa để khác nhau.
    let whole_blob_report = flow.cleanup_reports.into_iter().next().flatten();
    flow.cleanup_reports = vec![None; n];
    if let (Some(report), Some(first)) = (whole_blob_report, flow.cleanup_reports.first_mut()) {
        *first = Some(report);
    }
    flow.chapter_titles = titles;
    // Cùng khuôn reset của `segments`/`chapter_titles` — xem doc-comment `Flow::labels`.
    // Không có nhãn nào có ý nghĩa để mà phân bổ (bước này chỉ chạy trên `Blob`, chưa từng
    // mang một URL thật), nên chuỗi rỗng cho MỌI phần tử là đúng, không phải một xấp xỉ.
    flow.labels = vec![String::new(); n];
    // Cùng khuôn reset của `labels` — xem doc-comment `Flow::blocks`. Bước này chỉ chạy trên
    // `Blob` (return sớm khi `already_chaptered`, đúng nhánh mà `extract_main_content = true`
    // luôn đi cùng `Chapters`), nên `blocks` ở đây luôn TOÀN `None` trước lượt reset này —
    // gán lại `vec![None; n]` không mất dữ liệu có nghĩa nào.
    flow.blocks = vec![None; n];
    // 🔴 **THÊM 2026-09-08 (Story 6.10) — RESET VỀ `None` CHO CẢ N, KHÔNG gắn vào Chương đầu
    // như `cleanup_reports` làm ngay trên.** Con số nối dòng đo được TRƯỚC bước này (bước 4
    // luôn đứng trước bước 5 trong `PIPELINE_ORDER`) là của TOÀN TÀI LIỆU — thuộc về tài liệu,
    // không thuộc về Chương nào, kể cả `ord = 1` (xem doc-comment `Flow::joined_line_counts`
    // cho lý do đầy đủ, và Design Notes spec 6.10 "Vì sao `Blob` không cho `ord = 1` con số
    // FR125, trong khi `cleanup` thì có").
    flow.joined_line_counts = vec![None; n];
    Ok(flow)
}

/// Tách `text` tại VỊ TRÍ khớp của `pattern` — Chương thứ *i* là dải nửa-mở
/// `[start_i, start_{i+1})`, cùng quy ước mà `SegmentTermSpan`/`CleanupSpanWire` đã dùng khắp
/// kho. Dòng khớp ở lại NGUYÊN VẸN Ở ĐẦU mảnh — không `str::split` (xem doc-comment
/// [`split_chapters_step`] cho lý do). Không khớp chỗ nào ⇒ trả nguyên văn bản làm MỘT
/// Chương, `title = None` (không có mẫu khớp nghĩa là không có gì để tách).
///
/// Phần TRƯỚC khớp đầu tiên (nếu có) là một Chương RIÊNG, `title = None` — lời tựa/mục lục
/// không bao giờ bị vứt (§Always spec 6.6).
/// 🔴 **SỬA (vòng nghiệm thu 2026-09-06) — mẫu hỏng KHÔNG còn bị nuốt thành "không khớp
/// gì".** Bản đầu gọi `.unwrap_or_default()` trên `match_starts`, biến một `Err` (regex
/// không biên dịch được) thành `Vec` rỗng ⇒ một Chương DUY NHẤT, KHÔNG lỗi nào ném — đúng
/// lớp lỗi "rỗng im lặng" mà `AGENTS.md` gọi tên là trung tâm của dự án, và lệch với chính
/// module anh em: [`Step::CleanByRules`] (nhánh `match` ngay trên trong hàm này) TRUYỀN lỗi
/// của `core::cleanup::apply` lên thành [`ImportError::InvalidCleanupPattern`], không nuốt.
/// `commands::project::resolve_chapter_pattern` biên dịch thử mẫu TRƯỚC khi gọi
/// `run_pipeline` nên nhánh lỗi ở đây không nên chạm tới trên đường sản phẩm — nhưng đó là
/// rào của MỘT CHỖ GỌI, còn [`run_import`]/[`run_import_with_order`] là seam CÔNG KHAI
/// (`tests/**` gọi thẳng, và `segment_pipeline_boundary.rs` tồn tại chính vì thế) — một lời
/// gọi khác không đi qua `resolve_chapter_pattern` xứng đáng nhận lại đúng lỗi, không một
/// Chương giả trông như bình thường.
fn split_on_positions(text: &str, pattern: &ChapterPattern) -> Result<Vec<(Unit, Option<String>)>, ImportError> {
    let starts = pattern
        .match_starts(text)
        .map_err(|e| ImportError::InvalidChapterPattern { detail: e.to_string() })?;
    if starts.is_empty() {
        return Ok(vec![(Unit::Decoded(text.to_owned()), None)]);
    }

    let mut pieces = Vec::with_capacity(starts.len() + 1);
    if starts[0] > 0 {
        pieces.push((Unit::Decoded(text[..starts[0]].to_owned()), None));
    }
    for (i, &start) in starts.iter().enumerate() {
        let end = starts.get(i + 1).copied().unwrap_or(text.len());
        let piece = &text[start..end];
        pieces.push((Unit::Decoded(piece.to_owned()), title_line_of(piece)));
    }
    Ok(pieces)
}

/// Tiêu đề HIỂN THỊ của một Chương vừa tách — dòng ĐẦU của `piece` (dòng chứa vị trí khớp),
/// TRIM hai đầu. CHỈ dùng cho cột `title`; KHÔNG áp lên `source_text` được lưu (`piece` giữ
/// NGUYÊN VĂN trong [`split_on_positions`]). Dòng trim rỗng ⇒ `None` (không có gì gọi là tiêu
/// đề).
///
/// 🔴 **SỬA 2026-09-06 (vòng rà đối kháng 3, mục 1) — một mảnh CHỈ CÓ ĐÚNG MỘT DÒNG không có
/// dòng tiêu đề RIÊNG BIỆT với thân.** Bản trước gọi `.lines().next().unwrap_or(piece)` VÔ
/// ĐIỀU KIỆN: khi bước 4 ([`normalize::normalize`], luôn chạy TRƯỚC bước này trong
/// [`PIPELINE_ORDER`]) gộp một nguồn KHÔNG có dòng trống ngăn cách tiêu đề khỏi thân thành
/// ĐÚNG MỘT dòng (luật gộp của bước 4: nối khi dòng không kết bằng dấu kết câu VÀ hai dòng
/// cùng một đoạn), "dòng đầu" của `piece` LÀ CHÍNH `piece` — cả thân Chương (có thể dài hàng
/// nghìn ký tự) đi thẳng vào cột `chapter.title` của `project.db`. Quy tắc sửa là CẤU TRÚC
/// (đếm SỐ DÒNG của `piece`), KHÔNG một ngưỡng ĐỘ DÀI nào (hằng số phù thuỷ bị cấm) — một
/// mảnh đúng một dòng không có cách nào tách "tiêu đề" khỏi "thân" một cách có nghĩa, nên
/// `title = None`. Piece CÓ từ hai dòng trở lên (kể cả khi dòng thứ hai là dòng trống, tức
/// nguồn gốc có dòng trống ngăn cách) giữ nguyên hành vi cũ: dòng đầu, trim, rỗng ⇒ `None`.
fn title_line_of(piece: &str) -> Option<String> {
    let mut lines = piece.lines();
    let first_line = lines.next()?;
    if lines.next().is_none() {
        // Đúng một dòng — không có gì để tách "tiêu đề" ra khỏi "thân" (xem doc-comment).
        return None;
    }
    let trimmed = first_line.trim();
    if trimmed.is_empty() { None } else { Some(trimmed.to_owned()) }
}

/// So khớp trên byte thô — nhánh `Unit::Undecoded` với mẫu `Literal` của
/// [`split_chapters_step`] — GIỮ NGUYÊN chưa từng đổi kể từ Story 6.2 (chính là dụng cụ đo
/// AD-39, §Design Notes spec 6.6 "Vì sao nhánh byte ở lại literal"). "không khớp ⇒ trả
/// nguyên bản", chỉ lát byte ĐỘ DÀI 0 bị loại (không có cách nào an toàn để nhận diện
/// "khoảng trắng" trên byte của một bảng mã CHƯA giải mã mà không giải mã nó trước — giải mã
/// Ở ĐÂY sẽ là một bước giải mã THỨ HAI ngoài [`Step::DecodeEncoding`]). Viết tay vì `[u8]`
/// không có `split` theo một CHUỖI CON tuỳ ý trong thư viện chuẩn (chỉ có tách theo một phần
/// tử/vị từ trên từng phần tử).
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
