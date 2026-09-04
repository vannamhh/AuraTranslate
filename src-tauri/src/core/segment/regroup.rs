//! **Gộp và tách segment — phép tính THUẦN cho hàng mới** (Story 2.8, FR78, AD-5, AD-47 ④).
//!
//! ─────────────────────────────────────────────────────────────────────────────
//! 🔴 VÌ SAO MODULE NÀY TÁCH KHỎI `commands::segment`
//! ─────────────────────────────────────────────────────────────────────────────
//! Một lượt gộp phải quyết **năm** giá trị cho hàng mới — văn bản nguồn, bản dịch, cặp cờ
//! kết đoạn, cờ cắt bỏ, xuất xứ — và **bốn trong năm** đến từ một luật viết ra ở đâu đó
//! ngoài mã: AD-37 (cờ), AD-47 ④ (xuất xứ), chữ ký #5(a) và #3(b) của Ice. Để chúng lẫn
//! trong một closure `Store::write` là đặt bốn luật vào chỗ mà `tests/**` **không gọi tới
//! được** — đúng phép đo đã buộc `flush_segment_targets` ra đời (`commands/segment.rs`:
//! *"đảo hai dòng đó rồi chạy `cargo test` cho 54/54 XANH"*).
//!
//! ⇒ Ở đây: phép tính thuần, không SQL, không `OpenWork`. Ở `commands::segment`: một câu
//! `UPDATE` / `INSERT` và không một quyết định nào.
//!
//! ─────────────────────────────────────────────────────────────────────────────
//! 🔴 AD-32 LÀ CÁI BẪY SONG SINH — đọc nhầm nó thì mọi AC hỏng mà mã vẫn biên dịch
//! ─────────────────────────────────────────────────────────────────────────────
//! AD-32 là luật cho gộp/tách **CHƯƠNG** (FR15) và nó nói **ngược lại** module này: giữ
//! nguyên `segment.id`, chỉ đổi `chapter_id` và `ord`. Luật của **SEGMENT** là AD-5:
//! **về hưu + tạo mới**, `id` cũ không bao giờ quay lại (AD-3).
//!
//! ─────────────────────────────────────────────────────────────────────────────
//! ⚠️ CÁI MODULE NÀY **KHÔNG** QUYẾT — ghi ra để không ai đi tìm ở đây
//! ─────────────────────────────────────────────────────────────────────────────
//! - **`ord`** và **về hưu** là việc của tầng SQL: chúng nói về *vị trí trong Chương* và
//!   *một cột thời điểm*, không về nội dung hàng mới.
//! - **Ca ① của AD-37** *(segment CUỐI Chương ⇒ cả hai cờ TẮT, luôn luôn)* cũng vậy: nó
//!   hỏi *"hàng này có phải hàng cuối Chương không"*, một câu hỏi về **vị trí**. Chỗ gọi áp
//!   [`super::paragraph::at_end_of_chapter`] sau khi biết vị trí — đúng như doc-comment của
//!   hàm đó đã dặn trước cho Story 2.8.

use super::paragraph::{merged, split_into, ParagraphFlags};
use super::split::LANG_CHINESE;

/// Một **mảnh đi vào** một lượt gộp, hoặc segment nguồn của một lượt tách.
///
/// ⚠️ Mượn chứ không sở hữu: chỗ gọi vừa đọc chúng ra từ một `query_map` và không có lý do
/// nào để nhân đôi bộ nhớ cho một phép tính không giữ gì lại.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SegmentPart<'a> {
    /// `segment.source_text`.
    pub source_text: &'a str,
    /// `segment.target_text` — **chuỗi rỗng = chưa dịch**, không một giá trị vắng mặt.
    pub target_text: &'a str,
    /// Cặp cờ kết đoạn (AD-37 + AD-46).
    pub flags: ParagraphFlags,
    /// `segment.is_omitted` (FR133, bước di trú 8).
    pub is_omitted: bool,
    /// `segment.translation_origin` (FR117, bước di trú 11). `""` = chưa có bản dịch.
    pub translation_origin: &'a str,
}

/// Một hàng `segment` **sắp được tạo** — kết quả của một lượt gộp hoặc một mảnh của tách.
///
/// ⚠️ Không `id`, không `ord`, không `status`, không `retired_at`: bốn thứ đó do tầng SQL
/// cấp. `status` đặc biệt **không** có mặt ở đây vì AD-5 đã khoá nó thành hằng — *"segment
/// mới bắt đầu ở trạng thái **chưa xác nhận** với lịch sử rỗng"* — và một trường ở đây sẽ
/// mời chỗ gọi truyền một giá trị khác.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NewSegment {
    /// Văn bản nguồn của hàng mới.
    pub source_text: String,
    /// Bản dịch của hàng mới. Chuỗi rỗng = chưa dịch.
    pub target_text: String,
    /// Cặp cờ kết đoạn, tính bằng [`super::paragraph`].
    pub flags: ParagraphFlags,
    /// Cờ cắt bỏ, theo chữ ký #5(a) của Ice.
    pub is_omitted: bool,
    /// Xuất xứ, theo AD-47 ④.
    pub translation_origin: String,
}

/// Giá trị xuất xứ *"người khác dịch"* — AD-47 ④, ca bất đồng.
///
/// 🔴 **Chép hằng, không mượn `commands::segment`**, và đó là một mệnh đề về hướng phụ
/// thuộc chứ không một lượt lười: `core::**` **không** được phụ thuộc `commands::**` —
/// chiều đúng là commands → core. `tests/segment_boundary.rs` cưỡng chế lằn ranh đó.
///
/// ⚠️ Cái giá, ghi ra thay vì giấu: hai chuỗi `"other"` ở hai chỗ, và **không cổng nào**
/// canh chúng khớp nhau. Đường đóng là một ca hợp đồng so hai hằng — đã dựng ở
/// `segment_contract.rs`.
pub const ORIGIN_OTHER: &str = "other";

/// Xuất xứ *"chưa có bản dịch nào để khai"* — cùng giá trị `TRANSLATION_ORIGIN_NONE`.
///
/// ⚠️ Cùng lý do và cùng cái giá với [`ORIGIN_OTHER`].
pub const ORIGIN_NONE: &str = "";

/// Dấu nối hai `source_text` — **theo ngôn ngữ NGUỒN** (chữ ký #3(b) của Ice, 2026-08-17).
///
/// 🔴 Nhánh chọn bằng [`LANG_CHINESE`] của [`super::split`], **không** một hằng thứ hai:
/// bảng *"ngôn ngữ nào không dùng khoảng trắng"* đã có đúng một chủ trong kho, và cái giá
/// mà chữ ký #3(b) phải trả — *"dựng một nguồn sự thật thứ hai"* — chỉ tránh được bằng cách
/// tái dùng chính hằng đó.
///
/// ⚠️ **Mọi giá trị khác `"zh"` đi nhánh khoảng trắng**, đúng luật mặc định mà
/// [`super::split`] đã khai bằng chữ.
///
/// 🔵 **MỞ `pub(super)` 2026-09-04 (Story 6.4, FR124/FR125)** — chủ THỨ HAI xuất hiện:
/// `core::segment::normalize::normalize` GỌI hàm này để nối dòng giữa câu, đúng luật §Always
/// spec 6.4 *"bảng ngôn ngữ nối không dấu cách: `regroup::source_joiner` là chủ duy nhất"*.
/// `pub(super)` (không `pub`) giữ nguyên phạm vi trong `core::segment` — không lộ ra ngoài
/// module cha, chỉ thêm MỘT chỗ gọi được phép, không dựng một bảng thứ hai.
#[must_use]
pub(super) fn source_joiner(source_lang: &str) -> &'static str {
    if source_lang == LANG_CHINESE {
        ""
    } else {
        " "
    }
}

/// Nối các bản dịch của một lượt gộp.
///
/// 🔴 **Luôn dấu cách, không hỏi `source_lang`** — và đây là một mệnh đề, không một lượt bỏ
/// sót. `target_text` là **tiếng Việt** ở mọi Tác phẩm (`GridPanel.vue:295-300` khai bằng
/// chữ: *"cột bản dịch chứa tiếng Việt đã dịch; từ điển nhúng là zh→vi / en→vi"*). Hỏi
/// `source_lang` ở đây là để ngôn ngữ **nguồn** quyết định hình dạng chữ **đích** — sai một
/// cách im lặng, và sai đúng ở Tác phẩm tiếng Trung, tức ca thường nhất.
///
/// 🔴 **Mảnh RỖNG bị bỏ qua, không nối thành khoảng trắng.** Nối `"A"` với `""` bằng `" "`
/// cho `"A "` — một ký tự người dùng **chưa từng gõ**, nằm trên đĩa vĩnh viễn. Nó không lộ
/// ra ở đâu cả: `confirm_segment` nay `trim()` **hai vế** (chữ ký thứ mười của Ice,
/// 2026-08-16) nên phép so mốc vẫn cho đúng kết quả, và không cổng nào đọc `target_text`.
/// ⇒ Một lỗi im lặng hoàn hảo, và chỗ chặn nó là **đây**, lúc chuỗi được dựng.
fn join_targets(parts: &[SegmentPart<'_>]) -> String {
    parts
        .iter()
        .map(|p| p.target_text)
        .filter(|t| !t.is_empty())
        .collect::<Vec<_>>()
        .join(" ")
}

/// **AD-47 ④** — xuất xứ của segment sinh từ gộp/tách.
///
/// > *"Mọi mảnh mang **cùng một** giá trị ⇒ segment mới giữ giá trị đó. **Bất kỳ bất đồng
/// > nào** ⇒ **người khác dịch**. Tách là ca tầm thường của luật này."*
/// > — `ARCHITECTURE-SPINE.md:715-717`
///
/// ⚠️ AD-47 ④ ghi sẵn **cái mất**, chép vào đây để người sau không đọc nó thành một lỗi:
/// gộp một câu `""` *(chưa dịch)* với một câu *tôi dịch* cũng rơi vào nhánh bất đồng. Luật
/// chọn **chiều nói dối rẻ**, không chọn chiều đúng — vì ở ca bất đồng **không có** giá trị
/// đúng.
///
/// ⚠️ Nhóm rỗng ⇒ `None`, cùng luật với [`merged`]: không bịa một câu trả lời cho một câu
/// hỏi vô nghĩa.
#[must_use]
fn merged_origin(parts: &[SegmentPart<'_>]) -> Option<String> {
    let first = parts.first()?.translation_origin;
    if parts.iter().all(|p| p.translation_origin == first) {
        Some(first.to_owned())
    } else {
        Some(ORIGIN_OTHER.to_owned())
    }
}

/// **GỘP** một nhóm segment liền nhau thành một hàng mới.
///
/// `source_lang` là `work.source_lang` — trường **bất biến** đặt lúc tạo (AD-18).
///
/// ⚠️ Nhóm rỗng ⇒ `None`. Nhóm **một** phần tử thì hợp lệ và trả về chính nó — chỗ gọi
/// quyết định đó có phải một thao tác vô nghĩa không, vì *"vô nghĩa"* ở đây là một mệnh đề
/// về **bề mặt người dùng**, không về phép tính.
///
/// 🔴 **Ca ① của AD-37 KHÔNG áp ở đây** — xem doc-comment đầu module.
#[must_use]
pub fn merge(parts: &[SegmentPart<'_>], source_lang: &str) -> Option<NewSegment> {
    let flags = merged(&parts.iter().map(|p| p.flags).collect::<Vec<_>>())?;
    let translation_origin = merged_origin(parts)?;

    Some(NewSegment {
        // 🔵 **2026-08-17, code review — `.filter()` này THÊM VÀO, và nó là bản sao của một
        // bản vá đã có ở [`join_targets`].** Bản đầu nối **vô điều kiện**, nên một mảnh
        // `source_text` rỗng cho `"A" + " " + ""` = `"A "` — một ký tự **không có trong
        // nguyên văn của Tác phẩm**, nằm trên đĩa vĩnh viễn, và AD-4 nói ranh giới chỉ tính
        // MỘT LẦN lúc nhập nên không đường nào tính lại để phát hiện.
        //
        // ⚠️ Ghi ra vì sao nó **chưa** với tới hôm nay, thay vì để người sau tưởng đây là
        // một lỗi đang chảy máu: [`split_at`] từ chối mọi chỗ cắt để lại mảnh rỗng, đường
        // nhập `trim()`, và với Tác phẩm tiếng Trung [`source_joiner`] trả `""` nên phép nối
        // vô hại. Với nguồn `en` thì nó là **đúng** cái lỗi đã được vá một lần ở trường anh
        // em — và một bản vá chỉ áp cho một trong hai trường là một bản vá sẽ lệch.
        source_text: parts
            .iter()
            .map(|p| p.source_text)
            .filter(|s| !s.is_empty())
            .collect::<Vec<_>>()
            .join(source_joiner(source_lang)),
        target_text: join_targets(parts),
        flags,
        // Chữ ký #5(a) của Ice, 2026-08-17: **bất kỳ** mảnh nào đã cắt ⇒ hàng mới đã cắt.
        //
        // ⚠️ Chiều này NGƯỢC với AD-47 ④ ở ca bất đồng — 47 ④ chọn chiều bi quan cho một
        // NHÃN, đây chọn chiều an toàn cho một QUYẾT ĐỊNH của người dùng. Ice phán định
        // 2026-08-17 rằng luật này nằm trong biên độ AD-5 và không cần một `AD` mới; món nợ
        // *"luật này chưa có chỗ đứng trong spine"* ghi ở `deferred-work.md`, chủ Ice.
        is_omitted: parts.iter().any(|p| p.is_omitted),
        translation_origin,
    })
}

/// **TÁCH** một segment tại `cuts` — mỗi phần tử là một chỉ số tính bằng **ký tự Unicode**
/// của `source_text`. `n` chỗ cắt cho `n + 1` mảnh.
///
/// ─────────────────────────────────────────────────────────────────────────────
/// 🔵 2026-08-17 — NHẬN **MỘT TẬP** CHỖ CẮT. Chữ ký cho AC7 vế *"nhiều mảnh"*
/// ─────────────────────────────────────────────────────────────────────────────
/// Bản đầu nhận `cut: usize` và viết cứng `split_into(part.flags, 2)` — tức **đúng hai**
/// mảnh mỗi lượt. Code review 2026-08-17 bắt được rằng `epics.md:2522` đòi *"tách một
/// segment thành **nhiều** mảnh ⇒ cờ theo mảnh cuối, mọi mảnh trước nhận cờ tắt"*, và khoảng
/// hở đó — khác hẳn khoảng hở song sinh của AC6 — **không** có quyết định, **không** có chữ
/// ký, **không** có một dòng nợ nào.
///
/// **Ice ký 2026-08-17: dựng đa-mảnh trong chính story này.** Cơ chế bề mặt là **tích luỹ** —
/// mỗi cú bấm vào cột nguyên văn thêm một điểm, `⌘/` cắt tại tất cả cùng một lượt. Đó là một
/// tương tác **chưa tài liệu nào của dự án mô tả**; nó được nêu ra và ký, không suy ra.
///
/// 🔴 **Một lượt, không `n` lượt** — và đây là lý do cấu trúc, không một lượt tối ưu. Gọi
/// `⌘/` hai lần để có ba mảnh là đường **(c)** của Quyết định #1, thứ đã bị bác **bằng số
/// đo**: nó cho **5** hàng về hưu thay vì 3, cộng một segment trung gian mang một `id` vĩnh
/// viễn mà không người dùng nào từng thấy — và AD-3 nói `id` không bao giờ quay lại.
///
/// # Trả `None`
/// 🔴 Mọi ca dưới đây là *"rỗng im lặng"* ở dạng tệ nhất nếu cho đi qua — một hàng `segment`
/// không có văn bản nguồn, thứ không đường mã nào phía sau biết xử lý:
/// - `cuts` **rỗng** ⇒ không có lượt tách nào được yêu cầu;
/// - một chỗ cắt bằng `0`, ở cuối chuỗi, hay ngoài chuỗi;
/// - **hai chỗ cắt TRÙNG NHAU** ⇒ mảnh giữa chúng rỗng. Ca này chỉ tồn tại từ lượt đa-mảnh
///   và không có ở bản hai-mảnh — ghi ra vì nó là một biên **mới**.
///
/// ⚠️ `cuts` **được sắp lại tại chỗ gọi này**, không đòi chỗ gọi sắp sẵn: thứ tự bấm chuột
/// của người dùng không có lý do nào phải trùng thứ tự trong câu.
///
/// ⚠️ **`chars()`, KHÔNG byte.** `source_text` là tiếng Trung hoặc tiếng Anh, và một chỉ số
/// byte rơi giữa một ký tự nhiều byte làm `str::split_at` **panic** — mà `panic = "abort"`
/// giết cả tiến trình, không unwind, không flush WAL. Đây là chỗ duy nhất của story này nhận
/// một chỉ số từ webview, nên nó là chỗ duy nhất phải chịu được một số bất kỳ.
///
/// ⚠️ **Bản dịch đi theo mảnh ĐẦU** (chữ ký #3(b), vế tách): không có phép chiếu nào từ chỗ
/// cắt bên nguồn sang bản dịch (`epics.md:2552`), và một lượt tách **xoá** bản dịch là cái
/// giá mà đường (c) phải trả còn đường đã ký thì không. Với `n` mảnh mệnh đề ấy **không đổi
/// một chữ** — mảnh đầu giữ trọn, mọi mảnh sau rỗng.
#[must_use]
pub fn split_at(part: &SegmentPart<'_>, cuts: &[usize]) -> Option<Vec<NewSegment>> {
    let chars: Vec<char> = part.source_text.chars().collect();
    if cuts.is_empty() {
        return None;
    }

    let mut moc: Vec<usize> = cuts.to_vec();
    moc.sort_unstable();
    // Hai cho cat trung nhau cho mot manh RONG o giua. `windows(2)` tren mot lat da sap xep
    // bat ca do bang mot phep so duy nhat — va no cung bat luon ca `cuts` mot phan tu, vi
    // luc do khong co cap nao de so va vong lap khong chay.
    if moc.windows(2).any(|w| w[0] == w[1]) {
        return None;
    }
    // Da sap xep ⇒ chi can kiem hai dau. `first`/`last` tra `Option` nen khong mot chi so
    // tran nao o day; `moc` khong rong da bao dam o tren nhung mot `panic` cho ca tien trinh.
    if moc.first().is_some_and(|&c| c == 0) || moc.last().is_some_and(|&c| c >= chars.len()) {
        return None;
    }

    let so_manh = moc.len() + 1;
    let flags = split_into(part.flags, so_manh);

    // Bien cua manh thu `i` la `[moc[i-1], moc[i])`, voi hai dau mo rong ra `0` va `len`.
    let mut bien: Vec<usize> = Vec::with_capacity(so_manh + 1);
    bien.push(0);
    bien.extend_from_slice(&moc);
    bien.push(chars.len());

    let mut ra: Vec<NewSegment> = Vec::with_capacity(so_manh);
    for i in 0..so_manh {
        // `bien` co dung `so_manh + 1` phan tu nen ca hai chi so hop le; `get` thay cho chi
        // so tran vi `panic = "abort"` giet ca tien trinh, khong mot dong log nao.
        let (Some(&dau), Some(&cuoi)) = (bien.get(i), bien.get(i + 1)) else {
            return None;
        };
        let dau_manh = i == 0;
        ra.push(NewSegment {
            source_text: chars[dau..cuoi].iter().collect(),
            // Manh DAU giu tron ban dich; moi manh sau rong (chu ky #3(b)).
            target_text: if dau_manh {
                part.target_text.to_owned()
            } else {
                String::new()
            },
            // `split_into` tra dung `so_manh` phan tu — bao dam cua chinh ham do. `get` van
            // dung o day vi mot chi so tran la mot `panic` khong ai doc duoc.
            flags: *flags.get(i)?,
            is_omitted: part.is_omitted,
            // 🔴 **`""` cho mọi mảnh KHÔNG PHẢI mảnh đầu, KHÔNG xuất xứ của segment gốc** —
            // và đây là một SUY DẪN, không một luật mới. AD-47 ④ nói *"mọi mảnh mang cùng
            // một giá trị ⇒ giữ giá trị đó"*, nhưng nó nói về xuất xứ của **một bản dịch**;
            // các mảnh này **không có** bản dịch.
            //
            // `insert_segments` (`commands/segment.rs:125-128`) đã khai đúng mệnh đề ấy
            // bằng chữ cho cùng ca: *"một segment vừa tách ra từ văn bản nguồn **chưa có bản
            // dịch**, nên nó chưa có xuất xứ nào để khai. Cho nó `TRANSLATION_ORIGIN_SELF`
            // là ký thay người dùng"*.
            //
            // ⚠️ Và hậu quả nếu làm ngược: một mảnh mang `"self"` với `target_text` rỗng là
            // một hàng **TỰ MÂU THUẪN** trên đĩa — *"tôi đã dịch câu này"* + *"chưa có bản
            // dịch"*. Story 2.7 đã gặp đúng hình dạng đó một lần (§ca thường nhật ②).
            translation_origin: if dau_manh {
                part.translation_origin.to_owned()
            } else {
                ORIGIN_NONE.to_owned()
            },
        });
    }
    Some(ra)
}
