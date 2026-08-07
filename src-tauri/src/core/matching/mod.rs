//! Matcher DÙNG CHUNG: jieba + stemmer (AD-17).
//!
//! Một cài đặt khớp ngôn ngữ **duy nhất** phục vụ Glossary (FR51, Story 3.4) và
//! Translation Memory (FR61, Story 7.6) — hai đường khớp không được có hai cài đặt,
//! vì lớp lỗi mà AD-17 tồn tại để chặn là *"Glossary bắt được một biến thể mà TM không
//! không bắt được, và không ai hiểu vì sao"*.
//!
//! Crate dành cho module này: `jieba-rs` (nhánh tiếng Trung) · `tantivy-stemmers`
//! (nhánh tiếng Anh). Cả hai đã ghim `=` ở `Cargo.toml` từ Story 1.2; story 1.12 là
//! lượt đầu tiên trong dự án có mã thật gọi tới chúng.
//!
//! ─────────────────────────────────────────────────────────────────────────────
//! 🔴 RANH GIỚI: `core::dict` KHÔNG GỌI MODULE NÀY
//! ─────────────────────────────────────────────────────────────────────────────
//! AD-17 (thân Rule, `ARCHITECTURE-SPINE.md:236`) nói *mọi nơi cần khớp ngôn ngữ dùng
//! chung MỘT cài đặt* — nó **không** nói mọi đường đều phải gọi Matcher. Đường tra
//! cứu **từ điển** tiếng Anh không gọi, và đó là một **số đo** chứ không phải một
//! sở thích:
//!
//! - AD-44 ③ đo trên corpus thật: mọi dạng biến thể hình thái **đã có sẵn làm đầu mục
//!   riêng** trong `viwiktionary-en` — **16/16** mẫu thử, gồm cả bất quy tắc. Một lượt
//!   stemming chèn vào đường nóng đổi lấy **~0 recall**.
//! - Story 1.11b đo p95 đường tra cứu tiếng Anh ở **0,052–0,961 ms**. NFR1 cho backend
//!   ≤ 10 ms. Thêm một lượt stemming là tiêu ngân sách cho một khoản thu bằng không.
//!
//! ⇒ `core/dict/**` có **0** lời gọi tới module này, và mệnh đề đó được cưỡng chế bằng
//! cổng tĩnh ở `tests/matching_boundary.rs`. Người tiêu thụ là `core::glossary` và
//! `core::tm`, cả hai chưa tồn tại — xem §*"Không có người tiêu thụ hôm nay"* dưới.
//!
//! ⚠️ Sơ đồ mermaid của AD-13 (`ARCHITECTURE-SPINE.md:189`) còn một cạnh
//! `dict --> matching`. Nó vẽ **trước** lượt sửa Rule của AD-17 và nay mâu thuẫn với
//! chính thân Rule ở `:236`. Chủ sở hữu là Winston (architect); đã ghi vào
//! `deferred-work.md`. Mã theo **thân Rule**, không theo mũi tên.
//!
//! ─────────────────────────────────────────────────────────────────────────────
//! HAI ĐƯỜNG, HAI CƠ CHẾ — VÀ KHÔNG CÓ ĐƯỜNG THỨ BA
//! ─────────────────────────────────────────────────────────────────────────────
//!
//! | | `MatchLang::Zh` | `MatchLang::En` |
//! |---|---|---|
//! | Tách token | `jieba-rs` ([`tokenize`]) | run ký tự `char::is_ascii_alphanumeric` của `std` |
//! | Chuẩn hoá | **đồng nhất** (chữ Hán không có hình thái từ) | hạ chữ thường **rồi** Porter2 |
//! | n-gram | **ký tự** — không ranh giới từ (`epics.md:4946`) | **token** n-gram **sau** stemming (`epics.md:4950`) |
//! | Khớp thuật ngữ | **khớp chính xác**, chặn theo ranh giới token (`epics.md:2532`) | so khớp trên **dạng đã chuẩn hoá của cả hai vế** |
//!
//! 🔴 **Phép đếm độ dài n-gram là [`str::chars`]`().count()`, KHÔNG BAO GIỜ
//! [`str::len`].** `"山".len()` là **3** và `"中國".len()` là **6**. Một n-gram ký tự
//! cắt theo byte trên chữ Hán không chỉ sai — nó **panic** ở một biên không phải
//! ranh giới UTF-8. Bẫy này đã cắn Story 1.11 một lần.
//!
//! ─────────────────────────────────────────────────────────────────────────────
//! NGÔN NGỮ LÀ THAM SỐ TỪ CHỖ GỌI
//! ─────────────────────────────────────────────────────────────────────────────
//! **Không tồn tại** một vị từ dò script nào trong module này — không `is_han`,
//! không `is_cjk`, không `detect_lang`, không một dải Unicode nào viết cứng.
//! Xem doc-comment của [`MatchLang`] về ba lý do.
//!
//! ─────────────────────────────────────────────────────────────────────────────
//! KHÔNG CÓ NGƯỜI TIÊU THỤ HÔM NAY — VÀ ĐÓ LÀ CHỦ Ý
//! ─────────────────────────────────────────────────────────────────────────────
//! `core::glossary` và `core::tm` mỗi module còn 4 dòng doc-comment và 0 dòng mã.
//! AD-17 đòi dựng **một** cài đặt **trước** khi ba nơi mọc ba bản. Hệ quả phải chấp
//! nhận có ý thức: hình dạng API dưới đây là một **phỏng đoán có căn cứ**, không
//! phải một hợp đồng đã nghiệm thu bằng người dùng thật. Mọi hàm công khai đều suy ra
//! từ một AC có thật của Story 3.4 hoặc 7.6, và không hàm nào sống mà không có
//! ít nhất một ca test khẳng định hành vi của nó.
//!
//! **Ngoài phạm vi có chủ ý:** xếp hạng ứng viên, ngưỡng % tương đồng, chỉ mục
//! ngược, cache. Cả bốn thuộc Story 7.5/7.6 và phụ thuộc dữ liệu thật.
//!
//! ─────────────────────────────────────────────────────────────────────────────
//! MODULE NÀY LÀ **LÁ** TRONG ĐỒ THỊ PHỤ THUỘC (AD-13)
//! ─────────────────────────────────────────────────────────────────────────────
//! Không `use crate::core::*`, không `use crate::ports`, không
//! `use crate::commands`. Không chạm filesystem, không chạm database, không ra
//! mạng (AD-15: đúng ba điểm ra mạng, không có điểm thứ tư). Toàn bộ bề mặt là hàm
//! thuần trên `&str`.

use std::borrow::Cow;
use std::collections::HashSet;
use std::ops::Range;
use std::sync::LazyLock;

use jieba_rs::Jieba;
use tantivy_stemmers::algorithms::english_porter_2;

// ═════════════════════════════════════════════════════════════════════════════════
// Hằng cấu hình — CHỐT TRONG MODULE, không phơi ra cho mỗi chỗ gọi tự chọn
// ═════════════════════════════════════════════════════════════════════════════════

/// Cờ `hmm` truyền cho `Jieba::cut` — **`false`**, và nó là một **hằng của module**
/// chứ không phải một tham số công khai.
///
/// 🔴 **Vì sao là hằng:** hai chỗ gọi chọn hai giá trị = hai bộ ranh giới từ = đúng lớp
/// lỗi AD-17 tồn tại để chặn. Glossary và TM **phải** thấy cùng một phép tách.
///
/// 🔴 **Vì sao `false` — ĐO THẬT, không suy đoán.** Bật HMM là bật một mô hình
/// **đoán ranh giới cho chuỗi ký tự không có trong từ điển**, và HMM chỉ **gộp** các
/// ký tự mà phép cực đại xác suất đã bỏ rời ⇒ tập ranh giới của `hmm = true` là một
/// **tập con** của `hmm = false`. Vì [`find_terms`] chỉ nhận một lượt khớp khi **cả hai
/// đầu** rơi đúng ranh giới token, gộp thêm nghĩa là **từ chối thêm**.
///
/// Số đo trên `jieba-rs` 0.10.3, dict mặc định *(2026-08-05, `Jieba::cut`)*:
///
/// | Đầu vào | `hmm = false` | `hmm = true` |
/// |---|---|---|
/// | `中国人` *(giản thể)* | `中国` · `人` | `中国` · `人` |
/// | `中國人` *(phồn thể)* | `中` · `國` · `人` | **`中國人`** *(một token)* |
/// | `我喜歡中國人的文化` | `我`·`喜`·`歡`·`中`·`國`·`人`·`的`·`文化` | `我`·`喜歡`·**`中國人`**·`的`·`文化` |
/// | `萧炎和林动` | `萧`·`炎`·`和`·`林`·`动` | `萧炎`·`和`·`林动` |
///
/// 🔴 **Hàng phồn thể là hàng quyết định.** Từ điển mặc định của `jieba-rs` là **giản
/// thể**, nên với một Tác phẩm nguồn viết phồn thể — chuyện thường ở nguồn Đài/Hồng
/// Kông, và là lý do lược đồ từ điển của dự án mang cả `headword_simp` — `hmm = true`
/// gộp gần như mọi thứ thành những khối **do HMM bịa ra**. Một thuật ngữ Glossary như
/// `中國` khi đó rơi vào **giữa** khối `中國人` ⇒ **im lặng không khớp**. Với
/// `hmm = false`, cùng đoạn văn đó rơi ra từng ký tự ⇒ thuật ngữ vẫn khớp.
///
/// ⚠️ Lý do thứ hai, cùng hướng: đầu ra của HMM phụ thuộc **ngữ cảnh xung quanh**, nên
/// cùng một thuật ngữ khớp ở câu này và không khớp ở câu kia — đúng lớp lỗi *"không
/// không ai hiểu vì sao"* mà AD-17 tồn tại để chặn.
///
/// ⚠️ Cái giá phải trả có tên: `hmm = false` **không** phát hiện từ mới, nên nó nhận
/// rộng hơn ở vùng không có trong từ điển. Với khớp thuật ngữ, nhận rộng là **đúng
/// hướng an toàn** — người dịch thấy thừa một chỗ tô màu thì bỏ qua trong một giây;
/// một thuật ngữ **im lặng vắng mặt** thì không ai phát hiện.
///
/// ⚠️ Đổi giá trị này là đổi kết quả khớp của **cả** Glossary lẫn TM cùng lúc. Nếu
/// Winston muốn nó thành một mệnh đề của AD-17 để Epic 3 và Epic 7 không mở lại,
/// đây là chỗ con số sống.
const HMM: bool = false;

/// Instance `jieba-rs` DÙNG CHUNG — **đúng một** điểm khởi tạo trong toàn cây mã.
///
/// 🔴 **Vì sao [`LazyLock`] chứ không phải một lời gọi trong thân hàm:** feature
/// `default-dict` (mặc định) nhúng `src/data/dict.txt` — **5.071.843 byte thô** — qua
/// `include_flate::flate!`. Dựng instance là **giải nén cộng nạp từng dòng vào một cây
/// `cedar`**; đó không phải một hằng số biên dịch mà là công việc chạy lúc chạy. Một
/// lời gọi nằm trong thân hàm bị gọi lặp là một hồi quy NFR2 mà **không test nào
/// thấy**: test chạy một lần, người dùng gõ một nghìn lần.
///
/// ⚠️ Chi phí đó rơi vào **lần gọi đầu tiên** — tức có thể rơi đúng vào phím đầu tiên
/// người dùng gõ. Số đo thật ghi ở §Completion Notes của Story 1.12; nếu nó vượt NFR2
/// (50 ms) thì việc **hâm nóng ngoài đường gõ** là bàn giao có tên cho Story 3.4 —
/// story đầu tiên có một đường gõ thật để hâm nóng vào.
///
/// ⚠️ [`LazyLock`] hơn `OnceLock` ở chỗ hàm khởi tạo nằm **cạnh** khai báo thay vì rải
/// ra mọi chỗ gọi. Nó nằm trong `std::sync` (ổn định từ Rust 1.80; dự án ở
/// `rust-version = "1.85"`), nên **không** cần `once_cell`.
static JIEBA: LazyLock<Jieba> = LazyLock::new(Jieba::new);

// ═════════════════════════════════════════════════════════════════════════════════
// Kiểu công khai
// ═════════════════════════════════════════════════════════════════════════════════

/// Ngôn ngữ khớp — **THAM SỐ từ chỗ gọi**, không đoán từ nội dung.
///
/// 🔴 **Ba lý do, cả ba cưỡng chế được:**
///
/// 1. Đã có một cổng đang canh: `exactly_one_definition_of_is_han_exists_under_src_tauri`
///    (`tests/dict_boundary.rs`, Story 1.11b) quét **toàn** `src-tauri/**` và **sẽ đỏ**
///    nếu module này thêm một định nghĩa thứ hai. Hai bản sẽ trôi khỏi nhau, và bản hẹp
///    hơn đọc `𠧜` (U+209DC) thành *"không phải chữ Hán"*.
/// 2. **Ngữ nghĩa khác hẳn `core::dict::QueryRoute`.** `QueryRoute` trả lời *"tra vào
///    bảng nào của tệp `.db` nào"* — một thuộc tính của **hình dạng chuỗi truy vấn**
///    (AD-44 ①). Kiểu này trả lời *"khớp thuật ngữ trong văn bản của MỘT Tác phẩm"*, và
///    ngôn ngữ nguồn của Tác phẩm là một trường **bất biến trong `meta.json`, đặt lúc
///    tạo** (`prd.md:765-774`). Đoán lại từ nội dung là **bỏ đi một dữ kiện đã có** và
///    thay bằng một phỏng đoán.
/// 3. Cùng luật đã đặt ở `core::dict::LookupMode`: *"chế độ do chỗ gọi quyết, không
///    đoán từ nội dung"*.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MatchLang {
    /// Tiếng Trung — tách bằng `jieba-rs`, n-gram **ký tự**, khớp chính xác.
    Zh,
    /// Tiếng Anh — tách theo `char::is_ascii_alphanumeric`, n-gram **token sau stemming**.
    En,
}

/// Một token cùng **span byte vào chuỗi GỐC**.
///
/// 🔴 Span là **byte**, không phải chỉ số ký tự: Story 3.4 tô màu thuật ngữ trong
/// Panel Source (`epics.md:2528`) và nó cắt chuỗi bằng byte. Span luôn là một cặp ranh
/// giới UTF-8 hợp lệ ⇒ `text.get(token.span.clone())` luôn trả `Some`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MatchToken<'a> {
    /// Lát cắt của chuỗi gốc — **chưa chuẩn hoá**. Dùng [`normalize`] nếu cần dạng
    /// so khớp.
    pub text: &'a str,
    /// Vị trí byte trong chuỗi gốc: `text_goc[span] == token.text`.
    pub span: Range<usize>,
}

/// Một lượt khớp thuật ngữ.
///
/// ⚠️ `term_index` trỏ vào lát `terms` mà **chỗ gọi** truyền vào [`find_terms`] —
/// **không** phải một id Glossary. Module này không biết Glossary tồn tại, và đó
/// là điều kiện để nó là **lá** trong đồ thị phụ thuộc (AD-13).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TermMatch {
    /// Chỉ số trong lát `terms` của chỗ gọi.
    pub term_index: usize,
    /// Vị trí byte **trong chuỗi gốc** — không phải trong chuỗi đã chuẩn hoá.
    pub span: Range<usize>,
}

// ═════════════════════════════════════════════════════════════════════════════════
// Tokenize
// ═════════════════════════════════════════════════════════════════════════════════

/// Tách `text` thành token, mỗi token mang **span byte vào chuỗi gốc**.
///
/// **`Zh`** đi qua `Jieba::cut` với [`HMM`]. `jieba-rs` 0.10.3 trả `Token` **đã mang
/// sẵn** `byte_start`/`byte_end` — đừng tự tính lại offset.
///
/// ⚠️ **Không** dùng `cut_for_search`: nó thêm các gram con có trong từ điển và vì
/// vậy sinh **span chồng nhau**, tức hai lượt tô màu chồng nhau ở Story 3.4.
/// ⚠️ không Cũng đừng đi vòng qua `Jieba::tokenize(s, TokenizeMode::Default, hmm)` — nó
/// **chính là** `cut(s, hmm)` (`jieba-rs` lib.rs:1019).
///
/// **`En`** tách theo run ký tự `char::is_ascii_alphanumeric` của `std` — **không**
/// crate mới. Mọi thứ không phải chữ/số ASCII (khoảng trắng, dấu câu, gạch nối, và mọi
/// ký tự ngoài ASCII) là **dấu tách** và không vào token nào.
///
/// 🔴 **Có chủ ý giới hạn về ASCII — vá lúc code review (2026-08-05).** Bản trước dùng
/// `char::is_alphanumeric` (Unicode-rộng), và hàm đó nhận CẢ chữ Hán/script khác là "chữ
/// cái", nên một đoạn lẫn script (vd. `"hello世界world"`) dính thành MỘT token vô nghĩa
/// thay vì tách theo script. Giới hạn ASCII chặn lỗi dính đó, đổi lấy một đánh đổi đã
/// biết và chấp nhận: chữ Latin có dấu (vd. `café`) bị cắt SAI thành `caf` — AC11 cấm
/// thêm crate như `unicode-segmentation` vốn mới tách đúng theo script mà vẫn giữ được
/// dấu. Xem ca test `english_tokenization_is_ascii_only_and_never_fuses_other_scripts`.
///
/// 🔴 **Đường `Zh` giữ mọi token, kể cả dấu câu và khoảng trắng**, vì [`find_terms`]
/// dùng chính tập ranh giới này để phân xử, và một tập ranh giới **thủng lỗ** ở chỗ dấu
/// câu sẽ từ chối một thuật ngữ đứng ngay sát dấu câu. Đường `En` thì không cần —
/// nó so khớp trên dãy token chứ không trên ranh giới byte.
pub fn tokenize(text: &str, lang: MatchLang) -> Vec<MatchToken<'_>> {
    match lang {
        MatchLang::Zh => JIEBA
            .cut(text, HMM)
            .into_iter()
            .map(|token| MatchToken {
                text: token.word,
                span: token.byte_start..token.byte_end,
            })
            .collect(),
        MatchLang::En => {
            let mut out = Vec::new();
            let mut start: Option<usize> = None;

            for (offset, ch) in text.char_indices() {
                if ch.is_ascii_alphanumeric() {
                    start.get_or_insert(offset);
                } else if let Some(begin) = start.take() {
                    out.push(MatchToken {
                        text: &text[begin..offset],
                        span: begin..offset,
                    });
                }
            }
            if let Some(begin) = start {
                out.push(MatchToken {
                    text: &text[begin..],
                    span: begin..text.len(),
                });
            }
            out
        }
    }
}

// ═════════════════════════════════════════════════════════════════════════════════
// Normalize
// ═════════════════════════════════════════════════════════════════════════════════

/// Dạng chuẩn hoá của **một** token.
///
/// **`Zh` ⇒ đồng nhất.** Chữ Hán không có hình thái từ để chuẩn hoá: không chia
/// động từ, không số nhiều, không hoa/thường. Trả nguyên token là hành vi **đúng**
/// chứ không phải một chỗ chưa làm.
///
/// **`En` ⇒ hạ chữ thường TRƯỚC, rồi mới stem.** 🔴 Thứ tự này là bắt buộc và crate nói
/// thẳng: *"Tokens are expected to be lowercased beforehand"* (`tantivy-stemmers`
/// lib.rs:12). Sai thứ tự ⇒ `Running` và `running` cho hai stem khác nhau ⇒ đúng lỗ chữ
/// HOA mà AD-44 ③ vừa bịt ở đường tra cứu, tái sinh ở đường khớp.
///
/// 🔴 Phép hạ chữ thường là [`str::to_lowercase`] của Rust — **không phụ thuộc
/// locale**. AD-44 ③ đã trả giá cho bài học này một lần: một phép fold theo locale cho
/// **cùng một đầu vào hai kết quả trên hai máy** cài ngôn ngữ hệ điều hành khác nhau —
/// một hồi quy không tái lập được trên máy người sửa.
///
/// Thuật toán là `english_porter_2` (Porter2/English). Feature `english_porter_2` **mặc
/// định đã bật** ⇒ không cần đổi `Cargo.toml`. Gọi **thẳng hàm**; **không** đi qua
/// `StemmerTokenizer`/`StemmerFilter` vì hai thứ đó đòi hạ tầng `Tokenizer` của
/// `tantivy`, mà `tantivy` chỉ là dev-dependency của crate kia và không có trong cây
/// phụ thuộc của ta.
///
/// ⚠️ **Đây là *stemming*, KHÔNG phải *lemmatization*** (FR40, giới hạn đã tuyên bố).
/// Đo thật trên `english_porter_2`, 2026-08-05:
///
/// | Vào | Ra | Dạng gốc | Ra của dạng gốc | Gặp nhau? |
/// |---|---|---|---|---|
/// | `went` | `went` | `go` | `go` | không |
/// | `gone` | `gone` | `go` | `go` | không |
/// | `children` | `children` | `child` | `child` | không |
/// | `mice` | `mice` | `mouse` | `mous` | không |
/// | `better` | `better` | `good` | `good` | không |
/// | 🔴 `happiest` | `happiest` | `happy` | `happi` | không |
///
/// 🔴 Hàng `happiest` là một giới hạn **đo được mà story không đoán trước**: Porter2
/// **không** có luật cho hậu tố so sánh/cực cấp (`-er` · `-est`), nên biến thể **có
/// quy tắc** đó cũng không về được dạng gốc. Nó đứng chung hàng với các dạng bất quy
/// tắc, không phải một lỗi cài đặt.
///
/// Giới hạn đó là một **ca test có tên** ở `tests/matching_contract.rs`, không phải
/// một câu trong doc-comment: nó **đỏ** vào ngày ai đó đổi sang lemmatizer, và lúc đó
/// người sửa **phải** đọc lý do trước khi đổi con số.
pub fn normalize(token: &str, lang: MatchLang) -> Cow<'_, str> {
    match lang {
        MatchLang::Zh => Cow::Borrowed(token),
        MatchLang::En => {
            // ⚠️ Hạ chữ thường TRƯỚC — điều kiện tiên quyết của crate, xem doc-comment.
            let lowered = token.to_lowercase();
            let stemmed = english_porter_2(&lowered).into_owned();

            // ⚠️ `stemmed` mượn từ `lowered`, một chuỗi CỤC BỘ — nên nó phải được sở hữu
            // trước khi ra khỏi hàm. Trả `Borrowed` khi kết quả trùng đúng token gốc chỉ
            // là một phép tiết kiệm cấp phát; hành vi hai nhánh giống hệt nhau.
            if stemmed == token {
                Cow::Borrowed(token)
            } else {
                Cow::Owned(stemmed)
            }
        }
    }
}

// ═════════════════════════════════════════════════════════════════════════════════
// n-gram
// ═════════════════════════════════════════════════════════════════════════════════

/// Sinh n-gram của `text`.
///
/// **`Zh` ⇒ n-gram KÝ TỰ** — cửa sổ trượt theo ký tự, **không** theo token và không
/// không theo byte (`epics.md:4946`: *"n-gram ký tự — không có ranh giới từ"*).
/// `"中國人"` với `n = 2` ⇒ `["中國", "國人"]`.
///
/// **`En` ⇒ token n-gram SAU stemming** (`epics.md:4950`) — cửa sổ trượt trên **danh
/// sách token đã chuẩn hoá**, không trên chuỗi gốc. Các token trong một n-gram nối
/// bằng **một dấu cách**, nên chuỗi trả về không mang lại khoảng trắng gốc.
///
/// **Ca biên, cả ba trả rỗng và không panic:** `n == 0` · chuỗi rỗng · `n` lớn hơn
/// quần thể. **Không** trả một n-gram cụt — một n-gram ngắn hơn `n` là một phần tử
/// mà chỗ gọi không phân biệt được với một n-gram thật, tức một lỗi đếm im lặng ở
/// Story 7.6.
///
/// 🔴 Phép đếm quần thể ở nhánh `Zh` là [`str::chars`]`().count()`, **không bao giờ**
/// [`str::len`]. Xem doc-comment của module.
pub fn ngrams(text: &str, lang: MatchLang, n: usize) -> Vec<String> {
    if n == 0 {
        return Vec::new();
    }

    match lang {
        MatchLang::Zh => {
            // 🔴 `chars()`, KHÔNG `len()`: `"山".len()` là 3 và một cửa sổ trượt theo
            // byte trên chữ Hán panic ở biên không phải ranh giới UTF-8.
            let chars: Vec<char> = text.chars().collect();
            if n > chars.len() {
                return Vec::new();
            }
            chars
                .windows(n)
                .map(|window| window.iter().collect::<String>())
                .collect()
        }
        MatchLang::En => {
            let stems: Vec<String> = tokenize(text, lang)
                .into_iter()
                .map(|token| normalize(token.text, lang).into_owned())
                .collect();
            if n > stems.len() {
                return Vec::new();
            }
            stems.windows(n).map(|window| window.join(" ")).collect()
        }
    }
}

// ═════════════════════════════════════════════════════════════════════════════════
// find_terms — điểm vào của Glossary (FR51, Story 3.4)
// ═════════════════════════════════════════════════════════════════════════════════

/// Tìm mọi lượt xuất hiện của `terms` trong `text`, trả **span byte vào chuỗi gốc**.
///
/// 🔴 **Vì sao module này giao cả điểm vào chứ không chỉ ba nguyên hàm:** AD-17 nói
/// *"một **component**"*, không nói *"một túi hàm tiện ích"*, và `epics.md:1509` đòi
/// *"tồn tại **đúng một** cài đặt khớp ngôn ngữ"*. Nếu Story 3.4 và Story 7.6 mỗi bên
/// tự lắp một vòng khớp trên các nguyên hàm thì **vòng khớp thứ hai chính là cài đặt
/// thứ hai** — đúng thứ AD-17 tồn tại để chặn.
///
/// 🔴 **Span trỏ vào chuỗi GỐC, không vào chuỗi đã chuẩn hoá.** Story 3.4 tô màu
/// trên văn bản gốc; một span đo trên chuỗi đã hạ chữ thường **vẫn đúng độ dài với
/// ASCII thuần và sai với mọi thứ khác**, tức một lỗi đi trọn bộ test tiếng Anh mà không
/// không đỏ một ca nào.
///
/// ─────────────────────────────────────────────────────────────────────────────
/// `Zh` — KHỚP CHÍNH XÁC, CHẶN THEO RANH GIỚI TOKEN
/// ─────────────────────────────────────────────────────────────────────────────
/// `epics.md:2532` (Story 3.4): *"văn bản tiếng Trung → dùng khớp chính xác"*. Cài đặt
/// là phép tìm chuỗi con **thô** trên chuỗi gốc, **lọc** bằng ranh giới token của jieba:
/// một lượt khớp chỉ được nhận khi **cả hai đầu** của nó rơi đúng một ranh giới token.
///
/// 🔴 **Luật thật, phát biểu cho đúng thứ mã làm:** module này **không** tự phân xử
/// *"cụm này có phải một từ không"* — nó **nhường** câu hỏi đó cho jieba. Một lượt khớp
/// được nhận ⟺ jieba **đã cắt** đúng ở cả hai đầu của nó.
///
/// **Ca `中國` nằm trong `中國人` — số đo, không phải trực giác:**
///
/// | Văn bản | jieba cắt *(`hmm = false`)* | Thuật ngữ `中国`/`中國` |
/// |---|---|---|
/// | `中国人` *(giản thể)* | `中国` · `人` | ✅ **nhận** — jieba tự nói `中国` là một từ ở đây |
/// | `中國人` *(phồn thể)* | `中` · `國` · `人` | ✅ **nhận** — cả hai đầu đều là ranh giới |
/// | `文化`, thuật ngữ `文` | `文化` *(một token)* | không **từ chối** — đầu cuối rơi giữa token |
///
/// ⚠️ Nghĩa là luật này **không** phải *"cấm khớp chuỗi con"*. Nó chặn đúng một thứ:
/// một thuật ngữ **cắt ngang** một từ mà jieba đã nhận diện. Tô màu nửa token là nói
/// với người dịch rằng thuật ngữ của họ có mặt ở một chỗ nó không có mặt.
///
/// ⚠️ Một tên riêng không có trong từ điển jieba rơi ra **từng ký tự** (vì [`HMM`] là
/// `false`) nên nó luôn nằm gọn trong một dãy token liền nhau và **luôn** khớp — đo
/// được ở `萧炎和林动` ⇒ `萧`·`炎`·`和`·`林`·`动`. Xem doc-comment của [`HMM`].
///
/// ─────────────────────────────────────────────────────────────────────────────
/// `En` — SO KHỚP TRÊN DẠNG ĐÃ CHUẨN HOÁ CỦA **CẢ HAI** VẾ
/// ─────────────────────────────────────────────────────────────────────────────
/// Thuật ngữ và văn bản cùng đi qua [`normalize`], nên `running` trong văn bản khớp
/// thuật ngữ `run` — và đó **chính là** cơ chế của FR40, không phải một bảng biến
/// thể viết tay. Một thuật ngữ nhiều từ khớp một **dãy token liền nhau**; span trả về
/// chạy từ đầu token đầu tới cuối token cuối, nên nó **gồm cả** khoảng trắng và dấu câu
/// nằm giữa chúng trong văn bản gốc.
///
/// 🔴 **Vá lúc code review (2026-08-05):** một dãy token liền nhau **không** được coi
/// là khớp nếu dấu tách nằm giữa hai token bất kỳ trong dãy chứa dấu kết câu
/// (`.`/`!`/`?`) hoặc xuống dòng — chặn một thuật ngữ nhiều từ nối XUYÊN hai câu không
/// liên quan. Trước lượt vá này, `find_terms("…fast. Dog…", &["fast dog"], En)` khớp
/// `"fast. Dog"` dù hai từ thuộc hai câu khác nhau, vì nhánh này vốn coi dấu chấm câu và
/// khoảng trắng là dấu tách giống hệt nhau. Story 3.4 dùng thẳng span này để tô màu.
///
/// **Ngoài phạm vi:** xếp hạng, ngưỡng % tương đồng, chỉ mục ngược, cache — Story
/// 7.5/7.6. Thuật ngữ **rỗng** (hoặc chỉ gồm dấu tách) không bao giờ khớp.
///
/// Kết quả sắp theo `(span.start, span.end, term_index)` — thứ tự **tất định**, để hai
/// lượt chạy trên cùng đầu vào không cho hai thứ tự tô màu khác nhau.
pub fn find_terms(text: &str, terms: &[&str], lang: MatchLang) -> Vec<TermMatch> {
    let mut out: Vec<TermMatch> = Vec::new();

    match lang {
        MatchLang::Zh => {
            // Tập ranh giới token: mọi đầu và mọi cuối token. Tính MỘT lần cho cả lát
            // `terms` — phép tách của jieba là phần đắt nhất của lượt gọi này.
            let boundaries: HashSet<usize> = tokenize(text, lang)
                .into_iter()
                .flat_map(|token| [token.span.start, token.span.end])
                .collect();

            for (term_index, term) in terms.iter().enumerate() {
                // 🔴 Vá lúc code review (2026-08-05): bản trước chỉ chặn `is_empty()`,
                // không chặn thuật ngữ CHỈ GỒM dấu tách (vd. một chuỗi khoảng trắng) —
                // lệch với nhánh `En` (vốn đã chặn qua `needle.is_empty()`) và với chính
                // lời hứa "thuật ngữ rỗng hoặc chỉ gồm dấu tách không bao giờ khớp" ở
                // doc-comment của hàm này. Nếu jieba tách một chuỗi khoảng trắng thành
                // một token riêng, `text.find(term)` có thể khớp đúng token đó.
                if term.chars().all(|c| !c.is_alphanumeric()) {
                    continue;
                }
                let mut from = 0;
                while from <= text.len() {
                    let Some(rel) = text[from..].find(term) else {
                        break;
                    };
                    let start = from + rel;
                    let end = start + term.len();
                    if boundaries.contains(&start) && boundaries.contains(&end) {
                        out.push(TermMatch {
                            term_index,
                            span: start..end,
                        });
                    }
                    // ⚠️ Nhích tới ranh giới UTF-8 kế tiếp chứ không nhảy qua cả lượt
                    // khớp: hai lượt xuất hiện chồng nhau của cùng một thuật ngữ (`AA`
                    // trong `AAA`) đều là lượt xuất hiện thật.
                    from = start + text[start..].chars().next().map_or(1, char::len_utf8);
                }
            }
        }
        MatchLang::En => {
            let tokens = tokenize(text, lang);
            let stems: Vec<Cow<'_, str>> = tokens
                .iter()
                .map(|token| normalize(token.text, lang))
                .collect();

            for (term_index, term) in terms.iter().enumerate() {
                let needle: Vec<String> = tokenize(term, lang)
                    .into_iter()
                    .map(|token| normalize(token.text, lang).into_owned())
                    .collect();
                if needle.is_empty() || needle.len() > stems.len() {
                    continue;
                }

                for start in 0..=(stems.len() - needle.len()) {
                    let hit = needle
                        .iter()
                        .zip(&stems[start..start + needle.len()])
                        .all(|(want, got)| want.as_str() == got.as_ref());
                    // 🔴 Vá lúc code review (2026-08-05): từ chối một dãy token liền nhau
                    // nếu dấu tách giữa hai token bất kỳ trong dãy chứa dấu kết câu hoặc
                    // xuống dòng — không nối một thuật ngữ nhiều từ XUYÊN hai câu.
                    let end = start + needle.len();
                    let crosses_sentence_boundary = tokens[start..end - 1]
                        .iter()
                        .zip(&tokens[start + 1..end])
                        .any(|(a, b)| {
                            text[a.span.end..b.span.start].contains(['.', '!', '?', '\n'])
                        });
                    if hit && !crosses_sentence_boundary {
                        out.push(TermMatch {
                            term_index,
                            // 🔴 Span vào chuỗi GỐC: đầu token đầu → cuối token cuối.
                            span: tokens[start].span.start..tokens[end - 1].span.end,
                        });
                    }
                }
            }
        }
    }

    out.sort_by(|a, b| {
        (a.span.start, a.span.end, a.term_index).cmp(&(b.span.start, b.span.end, b.term_index))
    });
    out
}
