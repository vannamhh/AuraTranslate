//! Bộ tách câu và cờ kết đoạn — Story 2.1, FR23, AD-4 · AD-37.
//!
//! **Hàm thuần, không I/O, không `Connection`** — cùng khuôn [`super::import::import_text`].
//! Đây là chỗ DUY NHẤT trong kho biết bảng chữ cái kết câu; `tests/segment_boundary.rs`
//! cưỡng chế mệnh đề đó trên cả cây nguồn.
//!
//! ─────────────────────────────────────────────────────────────────────────────
//! 🔴 VÌ SAO VIẾT MỚI CHỨ KHÔNG MƯỢN `unicode-segmentation` — ĐO 2026-08-12
//! ─────────────────────────────────────────────────────────────────────────────
//! `unicode_sentences()` của `unicode-segmentation v1.13.3` cài UAX #29. Chạy thật trên
//! năm đầu vào, **hai ca trượt đúng hai AC bắt buộc của story**:
//!
//! | Đầu vào | UAX #29 | Phán quyết |
//! |---|---|---|
//! | `他走了；她笑了。` | `n=1` | **trượt AC1** — `；` không phải ranh giới câu theo UAX #29 |
//! | `Mr. Smith went home. He slept.` | `n=3`, cắt ngay sau `Mr. ` | **trượt AC2** |
//! | `他走了。她笑了。` | `n=2` | đúng |
//! | `真的吗？太好了！` | `n=2` | đúng |
//! | `It costs 3.50 dollars. That is fine.` | `n=2` | đúng |
//!
//! Crate đó **đã nằm trong cây mặc định** qua `tauri → muda → keyboard-types`, nên dùng nó
//! là 0 byte payload — nhưng nó không làm được việc này. Luật của AC1/AC2 là một tập hữu
//! hạn đã viết sẵn trong PRD; viết tay rẻ hơn uốn một thư viện không khớp. **0 crate thêm.**
//!
//! `Intl.Segmenter('zh', { granularity: 'sentence' })` ở webview bị loại vì hai lý do độc
//! lập: AD-1 đặt tách câu ở Rust bằng chữ, và nó chạy **mỗi lần Chương nạp** — đúng thứ AC3
//! cấm.
//!
//! ─────────────────────────────────────────────────────────────────────────────
//! 🔴 RANH GIỚI ĐÓNG BĂNG VĨNH VIỄN — AD-4
//! ─────────────────────────────────────────────────────────────────────────────
//! Kết quả của bộ tách này được ghi xuống `project.db` **một lần** lúc nhập và không đường
//! mã nào tính lại lúc nạp Chương (AC3). Một ranh giới sai hôm nay là một segment sai
//! **không sửa lại được** bằng một bản vá về sau — nó chỉ sửa được bằng thao tác tái tách
//! chủ động của Story 2.8, và thao tác đó cho **về hưu** id cũ (AD-5). Vì thế mọi luật ở
//! đây thà bỏ sót một ranh giới còn hơn dựng thừa một ranh giới sai.
//!
//! 🔴 **A4 là một GIẢ ĐỊNH, không một sự thật** (`prd.md:1075`): *"tách câu tự động đúng ở
//! tỷ lệ chấp nhận được"*. FR78 tồn tại chính vì tỷ lệ đó không bao giờ là 100% — gộp/tách
//! thủ công (Story 2.8) là đường lui có chủ.

/// Giá trị `work.source_lang` chọn nhánh tiếng Trung.
///
/// ⚠️ **Mọi giá trị khác đi nhánh tiếng Anh.** FR23 chỉ khai hai ngôn ngữ, nhưng cột
/// `work.source_lang` nhận chuỗi tự do (`schema.rs`, `WORK_DDL`) — nên nhánh mặc định phải
/// được khai bằng chữ thay vì để mỗi người đọc tự suy. Giá trị phân biệt đang dùng trong
/// kho là chuỗi `'zh'` (`sourcePanelState.ts`, `dict.ts`).
pub const LANG_CHINESE: &str = "zh";

/// Dấu kết câu của nhánh tiếng Trung — AC1, nguyên văn `。！？；`.
const ZH_TERMINATORS: [char; 4] = ['。', '！', '？', '；'];

/// Dấu kết câu của nhánh tiếng Anh — AC2.
///
/// ⚠️ `…` (U+2026) **không** có ở đây, và đó là luật 4 của Quyết định #5: dấu ba chấm không
/// kết câu. Một run `..` trở lên cũng bị loại ở [`en_run_is_boundary`] vì cùng lý do.
const EN_TERMINATORS: [char; 3] = ['.', '!', '?'];

/// Dấu đóng được **hút vào** segment ngay trước, khi nó đứng liền sau dấu kết câu.
///
/// Không có luật này, `他说：“真的吗？”她笑了。` cho ra một segment thứ hai bắt đầu bằng một
/// `”` mồ côi — hình dạng hỏng nhìn thấy được ngay trong Editor (Story 2.2), và AD-4 đóng
/// băng nó vĩnh viễn. Ranh giới vẫn do dấu kết câu quyết; luật này chỉ quyết dấu đóng thuộc
/// về phía nào của ranh giới đó.
const TRAILING_CLOSERS: [char; 11] = ['”', '’', '」', '』', '》', '）', '】', '"', '\'', ')', ']'];

/// Ký tự mở được chấp nhận thay cho một chữ HOA ở phép xác nhận ranh giới tiếng Anh.
const OPENING_MARKS: [char; 8] = ['"', '\'', '“', '‘', '(', '[', '«', '『'];

/// Bảng viết tắt tiếng Anh — luật 1 của Quyết định #5, **danh sách ĐÓNG**.
///
/// Sắp tăng dần và không trùng (`segment_contract.rs` khẳng định cả hai), nên
/// `binary_search` dùng được. Bốn nhóm, gộp vào một mảng đã sắp:
/// danh xưng · tháng và thứ viết tắt · `etc.`/`vs.`/`e.g.`/`i.e.`/`cf.`/`al.` · công ty.
///
/// ⚠️ `May` **không** có mặt: nó không có dạng viết tắt kèm dấu chấm, và thêm `May.` vào
/// đây là dựng một luật cho một chuỗi không ai gõ.
///
/// ⚠️ So khớp **phân biệt hoa thường**. `mr.` viết thường không khớp bảng — nhưng phép xác
/// nhận ranh giới ([`en_run_is_boundary`], vế cuối) vẫn bắt được đa số ca đó, vì ký tự
/// không trắng kế tiếp là chữ thường.
///
/// ─────────────────────────────────────────────────────────────────────────────
/// 🔴 `etc.` CÓ MẶT, VÀ NÓ MANG MỘT CÁI GIÁ PHẢI NÓI THẲNG — Ice ký 2026-08-12
/// ─────────────────────────────────────────────────────────────────────────────
/// Lượt code review 2026-08-12 đo ra bảng này chỉ có `al.` trong nhóm thứ ba, còn `etc.`
/// `vs.` `e.g.` `i.e.` `cf.` **vắng mặt** dù doc-comment và Quyết định #5 đều khai là có —
/// nên `"Real Madrid vs. Barcelona played."` bị cắt làm hai. Ice chọn thêm **cả năm**, đúng
/// chữ của Quyết định #5.
///
/// Cái giá: `etc.` **thường** đứng cuối một câu thật, khác hẳn `Mr.`/`Dr.`. Từ lượt này,
/// `"We sell books, pens, etc. They are cheap."` cho **một** segment, không hai — và AD-4
/// đóng băng ranh giới đó vĩnh viễn. Đây là một đánh đổi đã cân và đã ký, không phải một ca
/// bỏ sót: đường lui là gộp/tách thủ công của Story 2.8 (FR78). Nếu về sau muốn lật, chỗ lật
/// là **một dòng** — gỡ `"etc."` khỏi mảng này và đổi lại kỳ vọng của
/// `segment_contract.rs::the_english_abbreviation_rules_match_decision_five_row_by_row`.
pub const EN_ABBREVIATIONS: [&str; 39] = [
    "Apr.", "Aug.", "Co.", "Dec.", "Dr.", "Feb.", "Fri.", "Inc.", "Jan.", "Jr.", "Jul.", "Jun.",
    "Ltd.", "Mar.", "Mon.", "Mr.", "Mrs.", "Ms.", "Nov.", "Oct.", "Prof.", "Sat.", "Sep.", "Sept.",
    "Sr.", "St.", "Sun.", "Thu.", "Thur.", "Thurs.", "Tue.", "Tues.", "Wed.", "al.", "cf.", "e.g.",
    "etc.", "i.e.", "vs.",
];

/// Một segment do bộ tách sinh ra — văn bản **đã cắt trắng hai đầu**, kèm cờ kết đoạn.
///
/// ⚠️ Tên là `SplitSegment`, **không** `Segment`, và đó là chủ ý: `src/panels/wordBoundary.ts`
/// đã có một kiểu `Segment` ở TypeScript, nhưng nó là segment **cấp TỪ** (`Intl.Segmenter`
/// với `granularity: 'word'`, Story 1.18b), sống ở webview và tính lại mỗi lần Chương nạp.
/// Kiểu này là segment **cấp CÂU**, ở Rust, ghi xuống `project.db`, tính **một lần** rồi
/// đóng băng. Khác tầng, khác đơn vị, khác vòng đời.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SplitSegment {
    /// Văn bản của câu, đã cắt khoảng trắng hai đầu. **Không bao giờ rỗng hoặc chỉ khoảng
    /// trắng**, và **không bao giờ chứa `\n` hay `\r`** — xem doc-comment của
    /// [`split_source_text`].
    pub text: String,
    /// AD-37 — *"sau câu này là xuống dòng"*. Cờ kết đoạn của **nguyên văn** (AC6).
    ///
    /// 🔵 **CẬP NHẬT 2026-08-16 (Story 2.5d) — dòng này đã HẾT ĐÚNG VỀ MÃ, sửa tại chỗ.**
    /// Bản cũ viết *"**Một** cờ dùng chung cho cả nguyên văn và bản dịch: không
    /// `source_paragraph_end`/`target_paragraph_end`"*. **AD-46** (FR134) nới nó: bản dịch
    /// nay có cờ riêng trên đĩa — `segment.is_target_paragraph_end`, bước di trú 9.
    ///
    /// 🔴 Nhưng **bộ tách vẫn chỉ sinh MỘT cờ**, và đó là chủ ý chứ không phải một chỗ còn
    /// thiếu: lúc nhập, cờ đích **bằng** cờ nguồn (AC2 — *"bản dịch soi gương bản gốc cho
    /// tới khi người dùng đổi"*), nên một cờ thứ hai ở đây sẽ là một bản sao mà hai chỗ
    /// cùng ghi. Đường nhập chép giá trị này sang cột đích **một lần**; từ đó hai cờ sống
    /// độc lập, và người dùng đổi cờ đích bằng lệnh riêng.
    /// ⚠️ AD-37 **vẫn sở hữu** cờ nguồn và không sửa một chữ — AD-46 khai đúng như vậy.
    pub is_paragraph_end: bool,
}

/// Tách `source_text` thành các segment cấp câu, kèm cờ kết đoạn — **một lượt quét duy nhất**.
///
/// # Nhánh ngôn ngữ
///
/// `source_lang == "zh"` ⇒ nhánh tiếng Trung ([`LANG_CHINESE`]); **mọi giá trị khác** ⇒
/// nhánh tiếng Anh. Nhánh chọn theo trường `work.source_lang` — **không** đoán từ nội dung
/// (AD-18: `source_lang` là trường bất biến đặt lúc tạo).
///
/// # Tập ranh giới
///
/// Ranh giới = *(run dấu kết câu, theo luật của nhánh)* **∪** *(mọi `\n` và `\r`)*.
///
/// 🔴 **Vế thứ hai là một bổ sung của lượt thực thi, và nó có lý do đo được.** Chỉ cắt ở dấu
/// kết câu để hở một ca thật và thường gặp — một dòng **không** kết thúc bằng dấu kết câu:
///
/// ```text
/// 第一章 开端        ← tiêu đề chương
/// 他走了。
/// ```
///
/// cho ra **một** segment `"第一章 开端\n他走了。"` mang một `\n` **bên trong**. Đó là một
/// ranh giới đoạn không có chỗ nào mô hình hoá được: AD-37 định nghĩa cờ là *"sau câu này là
/// xuống dòng"*, và Story 8.4/8.6 dựng lại đoạn lúc xuất **chỉ từ cờ đã lưu**. AD-4 đóng
/// băng sai lầm đó vĩnh viễn.
///
/// # Luật *"một câu phải có ít nhất một chữ"* — áp cho **cả hai** nhánh
///
/// Một dấu kết câu **không** chốt ranh giới nếu phần văn bản đang dựng trước nó không chứa
/// một ký tự `char::is_alphabetic` nào. Một dấu chấm sau một chuỗi chỉ gồm chữ số và dấu là
/// một **mốc đánh số**, không phải một câu.
///
/// 🔴 **Luật này là luật THỨ NĂM, ngoài bốn luật của Quyết định #5, và nó do một phép đo
/// trên dữ liệu thật dựng ra — không do một ca giả định.** Đo 2026-08-12 trên 21 Chương
/// Epic 1 có thật: Chương lớn nhất (`48.640` ký tự) là một tài liệu **Markdown**, và mục lục
/// đánh số của nó (`* 0\. Triết Lý Nền Tảng…` trên **một** dòng) bị cắt ngay tại dấu chấm
/// của mốc danh sách — **26 ranh giới sai trên 99** trong 100 segment đầu, tất cả cùng một
/// nguyên nhân. Sau luật này: **0/99**.
///
/// ⚠️ Luật áp cho ranh giới **dấu kết câu**, KHÔNG áp cho ranh giới **xuống dòng**. Một dòng
/// là một dòng: gộp một dòng không có chữ (`---` của Markdown, một hàng số) vào dòng sau là
/// phá đúng bổ sung của Quyết định #3 ngay trên đây.
///
/// ⚠️ Đây **không** phải một luật Markdown, và nó có chủ ý không phải: làm sạch Markdown là
/// FR124/FR125 và Story 6.5, một tầng khác. Mệnh đề ở đây thuần về kiểu chữ và không biết
/// định dạng nào — *một dấu chấm cần một câu đứng trước nó mới kết được câu*.
///
/// # Cờ kết đoạn (AD-37, AC6, AC7)
///
/// Cờ của segment thứ *i* bật khi **khe** giữa cuối segment *i* và đầu segment *i+1* chứa ít
/// nhất một ký tự xuống dòng. Segment **cuối cùng**: cờ **tắt, luôn luôn** (AC7) — kể cả khi
/// văn bản gốc kết thúc bằng một dòng trống.
///
/// Ba ca biên của AD-37 (`ARCHITECTURE-SPINE.md:449-453`), ghi ra đây để Story 2.8 không
/// phải đi tìm lại:
///
/// | Ca | Cờ đi đâu | Chủ |
/// |---|---|---|
/// | Gộp segment | theo **câu cuối** của nhóm gộp | Story 2.8 |
/// | Tách segment | theo **mảnh cuối**; mọi mảnh trước nhận cờ **tắt** | Story 2.8 |
/// | Segment cuối Chương | **tắt, luôn luôn** | **Story 2.1 — AC7** |
///
/// 🔵 **CẬP NHẬT 2026-08-16 (Story 2.5d, AD-46) — bảng này nay chạy cho HAI cờ.**
/// Bản dịch có cờ kết đoạn **riêng** (`segment.is_target_paragraph_end`, bước di trú 9), và
/// AC3 của 2.5d đòi ba ca trên áp cho nó **y nguyên**. Ba dòng bảng **không đổi một chữ** —
/// thứ đổi là **số lần** chúng chạy.
///
/// 🔴 **Và đây là chỗ Story 2.8 sẽ vấp nếu chỉ đọc bảng:** cách viết tự nhiên là lấy
/// `is_paragraph_end` của câu cuối rồi coi cờ đích *"chắc cũng vậy"*. Lượt suy đó **xoá
/// quyết định ngắt đoạn của người dùng** ở mọi câu mà hai cờ đã khác nhau — và không cổng
/// nào đỏ. Hai cờ đi theo câu cuối **độc lập**.
/// ⇒ Bảng đã thành mã: [`crate::core::segment::paragraph`] *(`at_end_of_chapter` · `merged` ·
/// `split_into`, cộng test hợp đồng)*. **Gọi nó, đừng cài lại.**
///
/// ⚠️ Bộ tách ở tệp này vẫn chỉ sinh **một** cờ, và đó là chủ ý: lúc nhập, cờ đích **bằng**
/// cờ nguồn (AC2 của 2.5d), nên một cờ thứ hai ở đây sẽ là một bản sao mà hai chỗ cùng ghi.
/// Phép soi gương có đúng một tên: `paragraph::ParagraphFlags::mirrored`.
///
/// # Bất biến của giá trị trả về
///
/// 1. Không segment nào rỗng hoặc chỉ khoảng trắng.
/// 2. Không segment nào chứa `\n` hay `\r` — vế `\r` là AC11, mà `deferred-work.md:561`
///    giao đích danh cho story này: dữ liệu THẬT trên đĩa mang `\r\n` chưa chuẩn hoá, và
///    chuẩn hoá thật (FR124/FR125) là Epic 6. Bộ tách **không** chuẩn hoá
///    `chapter.source_text`; nó tự phòng thủ bằng cách coi `\r` là khoảng trắng.
/// 3. Văn bản rỗng hoặc chỉ khoảng trắng ⇒ `Vec` rỗng. Chỗ gọi phải chịu được một Chương
///    **0 segment**.
pub fn split_source_text(source_text: &str, source_lang: &str) -> Vec<SplitSegment> {
    let chinese = source_lang == LANG_CHINESE;
    let mut out: Vec<SplitSegment> = Vec::new();

    // Vị trí byte nơi segment đang dựng bắt đầu, và vị trí quét hiện tại. Hai con số tách
    // rời vì một run dấu kết câu KHÔNG cắt (viết tắt, số thập phân) đẩy `cursor` đi mà
    // `start` ở lại.
    let mut start = 0usize;
    let mut cursor = 0usize;

    // Segment đang dựng đã gặp một chữ cái nào chưa — xem [`en_run_is_boundary`] và luật
    // *"một câu phải có ít nhất một chữ"* ở doc-comment của hàm này. Giữ thành một cờ chạy
    // dọc vòng lặp thay vì quét lại `source_text[start..cursor]` ở mỗi dấu kết câu: bản quét
    // lại là O(n²) trên một đoạn dài có nhiều dấu chấm bị từ chối, và một Chương được phép
    // nặng tới 100 MB (`MAX_IMPORT_BYTES`).
    let mut pending_has_letter = false;

    while cursor < source_text.len() {
        let ch = char_at(source_text, cursor);
        let ch_end = cursor + ch.len_utf8();

        // ── Ranh giới CỨNG: xuống dòng ────────────────────────────────────────────────
        // `\r` cũng cắt, không chỉ `\n`. Không có vế `\r`, một tệp xuống dòng kiểu CR đơn
        // (`"abc\rdef"`) cho ra một segment mang `\r` ở giữa thân — đúng thứ AC11 cấm.
        if ch == '\n' || ch == '\r' {
            push_segment(&mut out, &source_text[start..cursor]);
            let (next_start, gap_has_break) = skip_gap(source_text, cursor);
            mark_paragraph_end(&mut out, gap_has_break);
            start = next_start;
            cursor = next_start;
            pending_has_letter = false;
            continue;
        }

        // ── Ranh giới theo dấu kết câu ────────────────────────────────────────────────
        if is_terminator(ch, chinese) {
            let run_end = terminator_run_end(source_text, cursor, chinese);
            let closed_end = absorb_tail(source_text, run_end, chinese);

            // Nhánh tiếng Trung: AC1 nói tách theo `。！？；`, không điều kiện nào thêm —
            // ngoài luật *"một câu phải có ít nhất một chữ"*, áp cho **cả hai** nhánh.
            let boundary = pending_has_letter
                && (chinese || en_run_is_boundary(source_text, cursor, run_end, closed_end));

            if boundary {
                push_segment(&mut out, &source_text[start..closed_end]);
                let (next_start, gap_has_break) = skip_gap(source_text, closed_end);
                mark_paragraph_end(&mut out, gap_has_break);
                start = next_start;
                cursor = next_start;
                pending_has_letter = false;
            } else {
                // Không phải ranh giới ⇒ run đi vào thân segment đang dựng.
                cursor = run_end;
            }
            continue;
        }

        if ch.is_alphabetic() {
            pending_has_letter = true;
        }
        cursor = ch_end;
    }

    push_segment(&mut out, &source_text[start..]);

    // 🔴 AC7 — cờ của segment cuối cùng **tắt, luôn luôn**. Một văn bản kết thúc bằng dòng
    // trống đi qua `mark_paragraph_end` với `true` ngay trước đây; dòng này là chỗ mệnh đề
    // tuyệt đối của AC7 được cưỡng chế, chứ không phải một nhánh `if` rải trong vòng lặp.
    if let Some(last) = out.last_mut() {
        last.is_paragraph_end = false;
    }

    out
}

/// Ký tự tại vị trí byte `at`. Chỗ gọi bảo đảm `at` là một biên ký tự hợp lệ.
fn char_at(text: &str, at: usize) -> char {
    text[at..]
        .chars()
        .next()
        .expect("vi tri byte phai la mot bien ky tu hop le")
}

fn is_terminator(ch: char, chinese: bool) -> bool {
    if chinese {
        ZH_TERMINATORS.contains(&ch)
    } else {
        EN_TERMINATORS.contains(&ch)
    }
}

/// Vị trí byte ngay sau **run** dấu kết câu bắt đầu tại `from`.
///
/// Ca biên ③ của Task 1: `真的吗？？！` cho **một** ranh giới, không ba, và không segment
/// rỗng nào ở giữa.
fn terminator_run_end(text: &str, from: usize, chinese: bool) -> usize {
    let mut end = from;
    while end < text.len() {
        let ch = char_at(text, end);
        if !is_terminator(ch, chinese) {
            break;
        }
        end += ch.len_utf8();
    }
    end
}

/// Hút các dấu đóng đứng liền sau run dấu kết câu — xem [`TRAILING_CLOSERS`].
fn absorb_trailing_closers(text: &str, from: usize) -> usize {
    let mut end = from;
    while end < text.len() {
        let ch = char_at(text, end);
        if !TRAILING_CLOSERS.contains(&ch) {
            break;
        }
        end += ch.len_utf8();
    }
    end
}

/// Hút **toàn bộ** đuôi kết câu: dấu đóng và dấu kết câu **xen kẽ nhau**, tới khi không bên
/// nào tiến thêm được.
///
/// 🔴 **Vì sao phải lặp chứ không hút một lượt** — code review 2026-08-12, đo thật.
/// [`absorb_trailing_closers`] một lượt chỉ chặn hình dạng hỏng ở **phía sau** ranh giới.
/// Nó để hở phía trước: sau một ranh giới, `pending_has_letter` về `false`, nên một dấu kết
/// câu đứng ngay sau dấu đóng **không tự cắt được** (luật *"một câu phải có ít nhất một
/// chữ"* chặn nó) và rơi vào **đầu** segment kế tiếp.
///
/// Đo trước khi vá: `split_source_text("你好？”！再见。", "zh")` cho
/// `["你好？”", "！再见。"]` — một `！` mồ côi mở đầu segment hai, đúng loại hình dạng hỏng
/// mà [`TRAILING_CLOSERS`] tồn tại để ngăn, và AD-4 đóng băng nó vĩnh viễn.
///
/// ⚠️ `run_end` của chỗ gọi **không** đi qua đây, và đó là chủ ý: [`en_run_is_boundary`] đọc
/// `text[run_start..run_end]` để nhận ra dấu ba chấm (luật 4), nên run đó phải ở nguyên dạng
/// *chỉ dấu kết câu liền nhau*. Hàm này chỉ đẩy `closed_end` — tức chỗ **cắt**, không phải
/// chỗ **xét luật**.
fn absorb_tail(text: &str, from: usize, chinese: bool) -> usize {
    let mut end = from;
    loop {
        let after_closers = absorb_trailing_closers(text, end);
        let after_run = terminator_run_end(text, after_closers, chinese);
        if after_run == end {
            return end;
        }
        end = after_run;
    }
}

/// Khe giữa hai segment: bỏ qua khoảng trắng từ `from`, trả về *(vị trí bắt đầu segment kế
/// tiếp, khe có chứa xuống dòng hay không)*.
///
/// `char::is_whitespace` gồm cả `\r`, `\t`, và các khoảng trắng Unicode như `\u{3000}`
/// (khoảng trắng toàn giác, thường gặp trong văn bản tiếng Trung).
fn skip_gap(text: &str, from: usize) -> (usize, bool) {
    let mut at = from;
    let mut has_break = false;
    while at < text.len() {
        let ch = char_at(text, at);
        if !ch.is_whitespace() {
            break;
        }
        if ch == '\n' || ch == '\r' {
            has_break = true;
        }
        at += ch.len_utf8();
    }
    (at, has_break)
}

/// Đẩy một segment vào `out` — **chỉ khi** phần văn bản còn lại gì sau khi cắt trắng.
///
/// Cờ khởi tạo `false`; [`mark_paragraph_end`] bật nó lên sau, khi khe đã đọc xong.
fn push_segment(out: &mut Vec<SplitSegment>, raw: &str) {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return;
    }
    out.push(SplitSegment {
        text: trimmed.to_owned(),
        is_paragraph_end: false,
    });
}

/// Bật cờ kết đoạn của segment **đã đẩy gần nhất**, nếu khe vừa đọc chứa xuống dòng.
///
/// 🔴 **Hợp nhất (OR), không gán.** Một vùng chỉ khoảng trắng — dòng trống giữa hai đoạn —
/// không đẩy segment nào, nên khe của nó phải thuộc về segment **trước đó**. Gán thẳng sẽ
/// TẮT lại một cờ vừa bật đúng: `"一。\n\n二。"` đọc khe `"\n"` (bật), rồi khe `"\n"` thứ hai
/// cũng bật — nhưng `"一。\n  "` rồi `"  二。"` sẽ tắt cờ vừa bật nếu dùng phép gán.
fn mark_paragraph_end(out: &mut [SplitSegment], has_break: bool) {
    if !has_break {
        return;
    }
    if let Some(last) = out.last_mut() {
        last.is_paragraph_end = true;
    }
}

/// Bốn luật của Quyết định #5, theo thứ tự, chỉ áp cho nhánh tiếng Anh.
///
/// `run` là `text[run_start..run_end]` — chuỗi dấu kết câu liền nhau. `closed_end` là vị trí
/// sau khi đã hút dấu đóng, tức chỗ phép xác nhận nhìn tới.
fn en_run_is_boundary(text: &str, run_start: usize, run_end: usize, closed_end: usize) -> bool {
    let run = &text[run_start..run_end];

    // Luật 4 — dấu ba chấm không kết câu. `…` không nằm trong `EN_TERMINATORS` nên nó không
    // bao giờ mở một run; ca cần chặn ở đây là `...` viết bằng ba dấu chấm ASCII.
    if run.len() >= 2 && run.chars().all(|c| c == '.') {
        return false;
    }

    // Ba luật còn lại chỉ nói về MỘT dấu chấm đơn. Một run như `?!` hay `!` đi thẳng xuống
    // phép xác nhận — không viết tắt nào kết thúc bằng `!`, và không số thập phân nào dùng `?`.
    if run == "." {
        let before = text[..run_start].chars().next_back();
        let after = text[run_end..].chars().next();

        // Luật 3 — số thập phân: chữ số ở CẢ HAI bên (`3.50`).
        if matches!(before, Some(b) if b.is_ascii_digit())
            && matches!(after, Some(a) if a.is_ascii_digit())
        {
            return false;
        }

        // Luật 2 — chữ cái đầu đơn: đúng MỘT chữ HOA đứng trước dấu chấm, và trước chữ đó
        // không phải một chữ cái/chữ số (`J. R. R. Tolkien`).
        //
        // 🔴 CA LUẬT NÀY NUỐT, ĐO 2026-08-12 — Ice chấp nhận, ghi ra để không ai chẩn lại.
        // Luật không phân biệt được một chữ cái đầu tên với một TỪ tiếng Anh hợp lệ chỉ gồm
        // một chữ HOA đứng cuối câu. Đo thật: `"You got an A. Great job!"` cho **1** segment,
        // không 2; `"Turn the knob to A. It should click."` cũng vậy.
        //
        // Đây là hình dạng của một ưu tiên đã khai ở doc-comment đầu module — *"thà bỏ sót
        // một ranh giới còn hơn dựng thừa một ranh giới sai"* — vì AD-4 đóng băng ranh giới
        // vĩnh viễn, và một segment dính hai câu còn tách lại được bằng Story 2.8, còn một
        // ranh giới dựng thừa thì đã cắt mất một câu làm đôi. Đường siết đã cân và **không**
        // chọn: đòi thêm một chữ HOA đơn nữa theo sau — nó vá `"an A. Great"` nhưng vẫn hỏng
        // ở `"Tolkien, J. R. R. Wrote this."`, tức đổi một ca sai lấy một ca sai khác.
        if let Some(b) = before {
            if b.is_uppercase() {
                let head = &text[..run_start - b.len_utf8()];
                let before_initial = head.chars().next_back();
                if before_initial.is_none_or(|c| !c.is_alphanumeric()) {
                    return false;
                }
            }
        }

        // Luật 1 — bảng viết tắt. "Từ" là đoạn từ sau khoảng trắng gần nhất tới hết dấu chấm.
        let word_start = text[..run_end]
            .rfind(char::is_whitespace)
            .map_or(0, |i| i + text[i..].chars().next().map_or(1, char::len_utf8));
        let word = &text[word_start..run_end];
        if EN_ABBREVIATIONS.binary_search(&word).is_ok() {
            return false;
        }
    }

    // Phép xác nhận — Quyết định #5, câu cuối: *"Sau dấu kết câu thật, ranh giới chỉ chốt khi
    // ký tự không trắng kế tiếp là chữ HOA hoặc mở ngoặc kép — hoặc khi đã hết văn bản."*
    //
    // Đây là lưới an toàn cho mọi ca mà ba luật trên bỏ sót: dấu chấm giữa `e.g.` (từ `e.`
    // không có trong bảng) không cắt, vì ký tự kế tiếp là `g` viết thường.
    let (next_start, _) = skip_gap(text, closed_end);
    match text[next_start..].chars().next() {
        None => true,
        Some(c) => c.is_uppercase() || OPENING_MARKS.contains(&c),
    }
}
