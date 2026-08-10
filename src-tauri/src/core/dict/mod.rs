//! Tra cứu từ điển — ba nhánh tiếng Trung (AD-26) và hai nhánh tiếng Anh (AD-44), chọn
//! bằng một **vị từ điều phối** đứng trên cả hai.
//!
//! KHÔNG tồn tại bước hợp nhất nguồn (AD-19): mỗi kết quả luôn mang `source` của nó.
//! Mỗi lớp gỡ rời là một file `.db` độc lập, chỉ đọc (AD-10, AD-25).
//!
//! Crate dành cho module này: `rusqlite` (đọc `.db`) — dùng chung cài đặt với `core::store`.
//!
//! ⚠️ Câu trên là **tài liệu về một ranh giới**, không phải một lời gọi vượt qua nó:
//! module này **không** gõ tên crate SQLite ở một vị trí mã nào. Nó viết truy vấn qua
//! các kiểu **tái xuất** của [`crate::core::store`] — [`ReadHandle`], [`SqlResult`],
//! [`Row`] — và nhận kết nối từ chỗ gọi. Đường mở tệp sống ở
//! [`crate::core::store::ReadOnlyDb`]; xem doc-comment ở đó về vì sao nó ở đấy chứ
//! không ở đây.
//!
//! ─────────────────────────────────────────────────────────────────────────────
//! HAI ĐƯỜNG, NĂM Ô — VÀ NHÁNH ĐƯỢC CHỌN BẰNG SỐ **KÝ TỰ**, KHÔNG BẰNG SỐ BYTE
//! ─────────────────────────────────────────────────────────────────────────────
//! [`pick_route`] quyết đường **một lần cho mỗi lượt tra**; [`pick_branch`] quyết ô.
//!
//! **Đường `zh`** — truy vấn chứa ít nhất một ký tự Hán. Lọc `lang = 'zh'`.
//!
//! | Chế độ                  | Độ dài *(ký tự)* | Nhánh                       | Chỉ mục dùng                                     |
//! |-------------------------|------------------|-----------------------------|--------------------------------------------------|
//! | Tra chính xác đầu mục   | bất kỳ           | [`QueryBranch::ExactBtree`]  | `idx_entry_headword` + `idx_entry_headword_simp` |
//! | Chuỗi con               | 1–2              | [`QueryBranch::CharIdx`]     | bảng đảo ngược `char_idx`                        |
//! | Chuỗi con               | ≥ 3              | [`QueryBranch::FtsTrigram`]  | FTS5 `entry_fts` (`trigram`)                     |
//!
//! **Đường `en`** — mọi thứ còn lại. Lọc `lang = 'en'`. **HAI** nhánh, không phải ba.
//!
//! | Chế độ                  | Độ dài *(ký tự)* | Nhánh                                  | Chỉ mục dùng          |
//! |-------------------------|------------------|----------------------------------------|-----------------------|
//! | Tra chính xác đầu mục   | bất kỳ           | [`QueryBranch::ExactBtree`]             | `idx_entry_headword`  |
//! | Chuỗi con               | ≥ 3              | [`QueryBranch::FtsTrigram`]             | FTS5 `entry_fts`      |
//! | Chuỗi con               | < 3 *(gồm cả 0)* | 🔴 [`QueryBranch::NoBranchQueryTooShort`] | — *(không nhánh nào chạy)* |
//!
//! 🔴 **Không** ô `char_idx` cho đường tiếng Anh, và đó là một **số đo**: lớp
//! `viwiktionary-en` sinh **đúng 9** cặp `char_idx` trên **119.039** đầu mục (0,0076%).
//!
//! 🔴 **Phép đo độ dài là `chars().count()`, không bao giờ là `len()`.** `"山".len()`
//! là **3** (UTF-8) và `"中國".len()` là **6** — chọn nhánh theo `len()` đẩy **mọi** truy
//! vấn tiếng Trung 1–2 ký tự vào FTS5 trigram, nơi chúng trả **0** hàng trong 0,01 ms mà
//! không lỗi nào được ném. Đó chính xác là phát hiện nghiêm trọng nhất của mũi thăm dò
//! Giai đoạn 0, là lý do FR39 tồn tại, và là lý do AD-26 khai **ba** nhánh chứ không hai.
//! Đo được trên tệp thật: `entry_fts MATCH '"山"'` ⇒ 0 hàng, `char_idx` ⇒ 3.177 hàng.
//!
//! **Không fallback dây chuyền** *(thử nhánh 1, rỗng thì thử nhánh 2…)*. AD-26 nói
//! *"tra chính xác → B-tree"*, không nói *"thử B-tree trước"*. Một fallback ngầm làm
//! mỗi lượt tra chạy hai đến ba truy vấn — tức số đo NFR1 thành vô nghĩa — và làm
//! [`LookupResult::branch`] **nói dối** về đường đã đi.
//!
//! ─────────────────────────────────────────────────────────────────────────────
//! PHẠM VI: MỘT TỆP, MỘT LƯỢT
//! ─────────────────────────────────────────────────────────────────────────────
//! [`lookup`] chạy trên **một** kết nối tới **một** tệp `.db`. Nó không gom kết quả
//! nhiều tệp, không nhóm theo nguồn, không đọc `dict_sense`/`dict_example`/
//! `dict_citation`, và không hợp nhất đầu mục trùng — cả bốn là **Story 1.13**, và
//! AD-19 nói cái cuối cùng không bao giờ xảy ra.
//!
//! 🔄 **Story 1.13 đã giao ba trong bốn thứ đó, và mệnh đề trên không đổi một chữ:**
//! [`lookup`] **vẫn** chạy trên một tệp. Việc gom sống ở [`lookup_grouped`] — một tầng
//! **TRÊN** nó — việc nhóm theo nguồn ở [`SourceGroup`], việc đọc nghĩa ở
//! [`DictionarySource::senses`]. Thứ thứ tư *(hợp nhất đầu mục trùng)* không **vẫn không tồn
//! tại**, và `tests/dict_boundary.rs` nay canh nó bằng máy.
//!
//! ─────────────────────────────────────────────────────────────────────────────
//! 🔴 HAI PHA, VÀ VÌ SAO KHÔNG PHẢI MỘT (Story 1.13 §Quyết định #1B)
//! ─────────────────────────────────────────────────────────────────────────────
//! [`lookup_grouped`] trả **nhóm theo nguồn + đầu mục** *(rẻ)*; [`DictionarySource::senses`]
//! đọc nghĩa cho một tập đầu mục **do chỗ gọi chọn**. Lý do là một **số đo**: nhánh
//! `char_idx` một ký tự (`山`) trả **3.177** đầu mục ở p95 **7,324 ms** trên bản release —
//! chi phí của **một** tệp, **chưa đọc một hàng `dict_sense` nào**. Đọc nghĩa cho từng
//! đó đầu mục × ba tệp ngay trong pha một vượt trần 10 ms một cách chắc chắn, và đường ra
//! duy nhất là một `LIMIT` — tức module này sẽ tự quyết một **chính sách sản phẩm** mà
//! Story 1.11 đã giao tường minh cho **Panel Lookup (1.17)**.
//!
//! ⇒ **Không cache, không chỉ mục ngược trong bộ nhớ, không xếp hạng** ở đây. Ba
//! thứ đó thuộc 1.18 và phụ thuộc hành vi người dùng thật.
//!
//! 🔴 **CẬP NHẬT — Story 1.17 (2026-08-06):** `LIMIT` **có** ở đây, nhưng **không phải
//! một hằng cục bộ** — [`lookup`], [`lookup_with_branch`], [`lookup_grouped`] nhận `limit`
//! làm **tham số từ chỗ gọi** (cùng doctrine `route`/`branch`), và Panel Lookup
//! (`commands/dict.rs`) là nơi quyết giá trị đó. Trần áp **sau** phép xác minh chuỗi con
//! (`query.rs::verify_substring`) cho hai nhánh cần nó (Bẫy 11); [`LookupResult::truncated`]
//! báo khi trần đã cắt — xem §Quyết định #4 của story để có đo đạc đầy đủ.

mod han_viet;
mod layer;
mod query;
mod senses;

use std::collections::BTreeSet;

use crate::core::store::{ReadHandle, SqlResult};
use crate::ports::DictionarySource;

pub use han_viet::HAN_VIET_BATCH;
pub use layer::{
    DictLayer, DictLayers, MINIMUM_SCHEMA_VERSION, SUPPORTED_SCHEMA_VERSION, SkipReason,
    SkippedLayer,
};
pub use senses::SENSE_BATCH;

/// Đường tra cứu — **đã quyết ở tầng trên**, adapter không tự quyết lại (AD-44 ①).
///
/// 🔴 **NHỊ PHÂN, không có nhánh thứ ba.** Một biến thể `Unknown` đẩy câu hỏi *"làm gì
/// với nó"* xuống **mọi** chỗ gọi, và mỗi chỗ gọi sẽ trả lời khác nhau. Một truy vấn không
/// không thuộc hệ chữ nào của hai từ điển vẫn chạy một nhánh **thật** ở đường `En` và trả
/// **rỗng có lý do** — thứ nghiệm thu được — thay vì rỗng vì không ai chọn nhánh.
///
/// ⚠️ **`Serialize`** (Story 1.17, Quyết định #2) — ra dây bằng **chuỗi định danh máy**
/// qua `#[serde(rename = …)]` **từng biến thể**, không `#[serde(rename_all)]`: một
/// `usize` trên dây là thứ đảo nghĩa im lặng khi ai đó chèn một biến thể mới, và một câu
/// snake_case tự động của `En` sẽ ra `"en"` đúng nhưng không đảm bảo cho mọi biến thể tương
/// lai — rename từng cái là tường minh, không đoán.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
pub enum QueryRoute {
    /// Truy vấn chứa ít nhất một ký tự Hán ⇒ ba nhánh của AD-26, lọc `lang = 'zh'`.
    #[serde(rename = "zh")]
    Zh,
    /// Mọi thứ còn lại ⇒ hai nhánh của AD-44 ②, lọc `lang = 'en'`.
    #[serde(rename = "en")]
    En,
}

/// Vị từ điều phối — **hình dạng CHUỖI TRUY VẤN**, không phải ngôn ngữ của Tác phẩm.
///
/// Hàm thuần, không chạm database — điều kiện để AC1 nghiệm thu được mà **không cần
/// một tệp `.db` nào**, tức trong CI, nơi không có tệp từ điển nào (`.gitignore: *.db`).
///
/// 🔴 **Gọi ĐÚNG MỘT LẦN cho mỗi lượt tra**, ở tầng gom (Story 1.13) — không bên trong
/// [`lookup`], không bên trong `query.rs`. Để vị từ chạy **trong** adapter là để mỗi
/// tệp `.db` tự trả lời một câu hỏi thuộc về **cả lượt tra**, và hai tệp sẽ trả lời khác
/// nhau ngay khi định nghĩa [`is_han`] của chúng lệch nhau.
///
/// ⚠️ Vị từ nói về **script**, không nói về **ngôn ngữ**: `"日本語"` chứa kanji nên nó đi
/// đường [`QueryRoute::Zh`]. Đó là hành vi **đúng** theo AD-44, không phải một lỗi —
/// `dict-core.db` không mang một hàng tiếng Nhật nào, và một nhánh thứ ba cho tiếng
/// Nhật là thứ không có dữ liệu để tra.
///
/// **Không** điều phối theo ngôn ngữ của Tác phẩm: bôi đen `API` trong một truyện tiếng
/// Trung phải ra kết quả, không ra rỗng (AD-44 Prevents #2). Một tham số duy nhất là
/// cách mệnh đề đó cưỡng chế được.
pub fn pick_route(query: &str) -> QueryRoute {
    if query.chars().any(is_han) {
        QueryRoute::Zh
    } else {
        QueryRoute::En
    }
}

/// Bảy dải CJK — **chép nguyên văn** `tools/dict-build/src/char_idx.rs::is_han`.
///
/// 🔴 **MỘT định nghĩa trong toàn `src-tauri/**`.** Cổng parity văn bản
/// `han_ranges_are_verbatim_from_dict_build_char_idx` (`tests/dict_lookup.rs`) giữ hai bản
/// không trôi khỏi nhau: nó đọc tệp của workspace kia **dưới dạng văn bản**, không
/// import chéo crate — hai workspace tách rời **có chủ ý** (AC4 của Story 1.9), và gọi
/// chéo là hút build tool vào cây phụ thuộc của sản phẩm.
///
/// ⚠️ Vì sao chép chứ không thu hẹp: một bộ dải hẹp hơn (vd. chỉ BMP) đọc `𠧜`
/// (U+209DC) thành *"không phải chữ Hán"* ⇒ [`pick_route`] trả [`QueryRoute::En`] ⇒ truy
/// vấn chạy nhánh tiếng Anh, lọc `lang = 'en'`, và trả **rỗng** cho một đầu mục tiếng
/// Trung có thật — rỗng, **không lỗi**, đúng lớp lỗi AD-26 ra đời để chặn.
pub fn is_han(c: char) -> bool {
    let cp = c as u32;
    matches!(cp,
        0x3400..=0x4DBF     // CJK Extension A
        | 0x4E00..=0x9FFF   // CJK Unified Ideographs
        | 0xF900..=0xFAFF   // CJK Compatibility Ideographs
        | 0x20000..=0x2A6DF // Extension B
        | 0x2A700..=0x2EBEF // Extension C..F
        | 0x2F800..=0x2FA1F // Compatibility Supplement
        | 0x30000..=0x3134F // Extension G
    )
}

/// Chế độ tra — do **chỗ gọi** quyết, không đoán từ nội dung truy vấn.
///
/// 🔴 Một hàm tự đoán *"chắc người dùng muốn tra chính xác"* là một quy tắc nghiệp vụ
/// **ẩn** mà Auto-Lookup (1.18) và Panel Lookup (1.17) sẽ phải đoán ngược lại. AD-26 khai
/// ba nhánh, không khai một cơ chế đoán.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LookupMode {
    /// Đầu mục **bằng đúng** truy vấn.
    Exact,
    /// Đầu mục **chứa** truy vấn như một chuỗi con.
    Substring,
}

/// Đường đã đi thật sự — **giá trị trả về**, không phải một dòng log.
///
/// 🔴 Nhánh phải **quan sát được từ ngoài**, và đó là điều kiện để Bẫy `len()` ở trên
/// nghiệm thu được: một `eprintln!` không khẳng định được trong test, nên một cài đặt
/// chọn sai nhánh sẽ đi qua mọi phép kiểm *"kết quả khác rỗng"* mà không ai thấy.
///
/// ⚠️ **`Serialize`** (Story 1.17, Quyết định #2) — cùng doctrine [`QueryRoute`]: chuỗi
/// định danh máy qua `#[serde(rename = …)]` từng biến thể. `NoBranchQueryTooShort` ra dây
/// là `"query_too_short"` — không phải một chuyển đổi snake_case cơ học của tên biến
/// thể (sẽ ra `"no_branch_query_too_short"`), vì Panel Lookup đọc mã này để nói *"truy
/// vấn quá ngắn"*, và cái tên ngắn gọn hơn khớp thẳng câu đó.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
pub enum QueryBranch {
    /// Nhánh 1 — B-tree trên `headword` / `headword_simp`. Dùng ở **cả hai** đường.
    #[serde(rename = "exact_btree")]
    ExactBtree,
    /// Nhánh 2 — bảng đảo ngược `char_idx`, cho chuỗi con 1–2 ký tự. **Chỉ** đường `zh`.
    ///
    /// Đường tiếng Anh **không bao giờ** đi nhánh này, và đó là một **số đo** chứ không
    /// không phải một sở thích: lớp `viwiktionary-en` sinh **đúng 9** cặp `char_idx` trên
    /// **119.039** đầu mục (0,0076%). Bảng đảo ngược không áp được cho tiếng Anh.
    #[serde(rename = "char_idx")]
    CharIdx,
    /// Nhánh 3 — FTS5 `entry_fts` với tokenizer `trigram`, cho chuỗi con ≥ 3 ký tự.
    /// Dùng ở **cả hai** đường.
    #[serde(rename = "fts_trigram")]
    FtsTrigram,
    /// 🔴 **Không nhánh nào chạy** — chuỗi con tiếng Anh < 3 ký tự (AD-44 ④).
    ///
    /// **Không phải "không có kết quả":** nó là một trạng thái **KHÔNG HỖ TRỢ**, và
    /// Panel Lookup (FR41, Story 1.17) nói *"truy vấn quá ngắn"* chứ không nói *"không
    /// tìm thấy"*. Hai câu đó dẫn người dùng đi hai đường khác nhau: một câu bảo *"gõ
    /// thêm"*, câu kia bảo *"từ này không có trong từ điển"*.
    ///
    /// Vì sao không hạ ngưỡng trigram xuống 1: FTS5 `trigram` **không** lập chỉ mục
    /// token ngắn hơn ba ký tự — đo được `entry_fts MATCH '"山"'` ⇒ **0** hàng. Để một
    /// truy vấn 1–2 ký tự chạy nhánh trigram là để nó trả **rỗng im lặng**, đúng lớp lỗi
    /// AD-26 ra đời để chặn.
    ///
    /// ⚠️ Ca **0 ký tự** đi cùng đường: vị từ độ dài là **một** mệnh đề
    /// `chars().count() < 3`, không phải hai mệnh đề với một ca đặc biệt ở giữa, và một
    /// chuỗi rỗng **đúng là quá ngắn**.
    ///
    /// ⚠️ **Bất đối xứng có chủ ý với đường `zh`:** ở đó một truy vấn rỗng trả
    /// [`QueryBranch::CharIdx`] với `hits` rỗng (hành vi Story 1.11). **Đừng "đồng bộ"
    /// hai bên** — hai bảng nhánh khác nhau vì hai chỉ mục khác nhau.
    #[serde(rename = "query_too_short")]
    NoBranchQueryTooShort,
}

/// Một đầu mục khớp. **Một hàng của `dict_entry`, không phải một nghĩa.**
///
/// Không `dict_sense`, không `dict_example`, không `dict_citation` ở đây — đọc
/// nghĩa là **Story 1.13** (FR29–FR32), và hình dạng của nó phụ thuộc vào quyết định
/// nhóm-theo-nguồn mà story này không được phép đoán trước.
/// ⚠️ **`Serialize`** (Story 1.17, Quyết định #2a) — mọi trường đã `snake_case`, đúng như
/// trên dây; không `#[serde(rename_all)]`.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct EntryHit {
    /// `dict_entry.id` — chỉ duy nhất **trong một tệp `.db`**.
    pub entry_id: i64,

    /// 🔴 `dict_source.code` — chuỗi, **không** `source_id: i64`.
    ///
    /// Mỗi tệp `.db` mang bảng `dict_source` **RIÊNG**, nên `id = 1` tồn tại ở **cả ba**
    /// tệp và trỏ ba nguồn khác nhau (`viwiktionary` · `thieu-chuu` · `vietphrase`).
    /// Khoá theo `id` sẽ dán nhãn *"Thiều Chửu"* cho một nghĩa thật ra từ CVDICT ngay khi
    /// Story 1.13 gom nhiều tệp — FR31 vỡ theo cách thầm lặng nhất có thể, và nó vỡ ở
    /// **story sau** chứ không ở story này, tức đắt gấp đôi để lần ra.
    pub source_code: String,

    /// 🔴 `dict_entry.lang` — **một TRƯỜNG, không phải một KIỂU** (AD-44 ⑤).
    ///
    /// Story 1.11 viết ở đây rằng *"một hằng ngầm ở chỗ gọi là thứ 1.11b sẽ phải gỡ"*.
    /// Story 1.11b đã gỡ nó: giá trị nay là `"zh"` **hoặc** `"en"` tuỳ
    /// [`QueryRoute`] của lượt tra, và **không tồn tại** một bản ghi kết quả thứ hai
    /// dành riêng cho tiếng Anh. Một `EnEntryHit` song song sẽ buộc **mọi** chỗ gọi phải
    /// phân nhánh theo kiểu, và bước hợp nhất hai nhánh đó lại chính là thứ AD-19 cấm.
    pub lang: String,

    /// Đầu mục như nguồn ghi (phồn thể với đa số nguồn).
    pub headword: String,

    /// Dạng giản thể, `None` khi nguồn không phân biệt phồn/giản.
    pub headword_simp: Option<String>,
}

/// Kết quả một lượt tra: **đường đã đi** cộng các hàng khớp.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LookupResult {
    /// Nhánh đã chạy — xem [`QueryBranch`].
    pub branch: QueryBranch,
    /// Các đầu mục khớp, thứ tự của `dict_entry.id` tăng dần.
    pub hits: Vec<EntryHit>,
    /// 🔴 **Quyết định #4 (Story 1.17), hệ quả ②.** `true` ⇔ trần `limit` đã cắt bớt kết
    /// quả **của tệp này** — có thể còn đầu mục khớp khác, kể cả của một nguồn khác trong
    /// cùng tệp, mà lượt tra này không thấy được. **Không** phải một lỗi: đây là
    /// đường (b) đã chốt ở §hệ quả ② — panel nói ra "danh sách nguồn chưa đầy đủ" thay vì
    /// đảm bảo mọi nguồn có mặt bằng một truy vấn per-source đắt hơn (đo: `ROW_NUMBER()
    /// OVER (PARTITION BY source_id)` không dừng sớm được và CHẬM HƠN cả không có `limit`
    /// nào — xem §Debug Log References của story).
    pub truncated: bool,
}

/// Chọn nhánh cho một truy vấn. **Hàm thuần, `pub`, không chạm database.**
///
/// 🔴 Tách thành một hàm riêng là điều kiện để AC1 nghiệm thu được mà **không cần một
/// tệp `.db` nào** — tức phép kiểm đắt nhất của story này chạy được trong CI, nơi không
/// không có tệp từ điển nào (`.gitignore: *.db`).
///
/// 🔴 `route` là **tham số**, không phải một lời gọi [`pick_route`] bên trong: vị từ
/// điều phối chạy **ĐÚNG MỘT LẦN cho mỗi lượt tra**, ở tầng gom (AD-44 ①, vá A1).
///
/// ⚠️ `chars().count()` là phép đo, không phải `len()`. Xem doc-comment của module.
pub fn pick_branch(query: &str, mode: LookupMode, route: QueryRoute) -> QueryBranch {
    match mode {
        // Tra chính xác không phụ thuộc độ dài **ở cả hai đường**: một đầu mục một ký
        // tự và một đầu mục mười ký tự đều nằm trên cùng chỉ mục B-tree.
        LookupMode::Exact => QueryBranch::ExactBtree,

        // 🔴 `chars().count()` — KHÔNG `len()`. Đây là dòng đắt nhất của cả story;
        // xem bảng ở doc-comment của module. Hai đường dùng **cùng** phép đo và **khác**
        // ngưỡng, vì chúng dựng trên hai chỉ mục khác nhau.
        LookupMode::Substring => match route {
            QueryRoute::Zh => {
                if query.chars().count() <= 2 {
                    QueryBranch::CharIdx
                } else {
                    QueryBranch::FtsTrigram
                }
            }
            QueryRoute::En => {
                if query.chars().count() < 3 {
                    QueryBranch::NoBranchQueryTooShort
                } else {
                    QueryBranch::FtsTrigram
                }
            }
        },
    }
}

/// Tra một truy vấn trên **một** kết nối tới **một** tệp `.db`.
///
/// 🔴 Nhận [`ReadHandle`], **không** nhận [`crate::core::store::ReadOnlyDb`]: đây là
/// một **hàm thuần theo kết nối**, và chỗ gọi là bên mở kho. Ba hệ quả, cả ba đều là điều
/// kiện của một story sau:
///
/// 1. Story 1.13 gọi hàm này **một lần cho mỗi tệp** rồi gom — với một chữ ký nhận
///    `ReadOnlyDb`, nó phải mở/đóng hoặc mượn lồng nhau.
/// 2. Test dựng fixture rồi gọi thẳng, không phải dựng cả một `ReadOnlyDb` cho một ca
///    thuần logic.
/// 3. Cùng khuôn `bootstrap_config(store: Option<&Store>)` của Story 1.8: **hàm thuần là
///    đường sản phẩm**, vỏ là thứ bỏ đi được trong test.
///
/// 🔴 `route` **nhận từ chỗ gọi**, **không** tính lại ở đây (AD-44 ①, vá A1). Hàm này
/// và [`query`] **không bao giờ** gọi [`pick_route`] — một adapter không tự phân xử
/// lại một câu hỏi thuộc về **cả lượt tra**. Ba lý do, cả ba cưỡng chế được:
///
/// 1. Story 1.13 gọi hàm này **một lần cho mỗi tệp `.db`** và phải truyền **cùng một**
///    `route` xuống mọi tệp — để mỗi tệp tự tính là để hai tệp trả lời khác nhau ngay khi
///    định nghĩa [`is_han`] của chúng lệch nhau.
/// 2. AD-44 ① nói thẳng vị từ chạy **TRÊN** adapter.
/// 3. Test **ép được** tổ hợp `(truy vấn Hán, route = En)` mà [`pick_route`] không bao
///    giờ sinh ra — và đó là cách bộ lọc `lang` của đường tiếng Anh trở thành thứ
///    **nghiệm thu được** thay vì thứ *"chắc là đúng vì đầu vào không bao giờ tới đó"*.
///
/// Mọi nhánh lọc `lang` **tường minh trong SQL** — `'zh'` trên đường `Zh`, `'en'` trên
/// đường `En`. Xem [`query`] về vì sao vế đó không bỏ được. **Không** tồn tại một sổ
/// đăng ký *"tệp `.db` nào chứa ngôn ngữ nào"* (AD-44 ①, vá A2): **mọi** tệp đang gắn đều
/// được tra, và `lang` lọc trong SQL.
pub fn lookup(
    db: ReadHandle<'_>,
    query: &str,
    mode: LookupMode,
    route: QueryRoute,
    limit: usize,
) -> SqlResult<LookupResult> {
    let branch = pick_branch(query, mode, route);
    lookup_with_branch(db, query, route, branch, limit)
}

/// Cùng đường tra của [`lookup`], nhưng nhận **`branch` đã tính sẵn** thay vì tự gọi
/// [`pick_branch`] lại từ đầu.
///
/// 🔴 **Chỗ gọi duy nhất: tầng gom (Story 1.13, [`DictLayer::lookup`]).** `branch` là một
/// **GIÁ TRỊ của cả lượt tra** ([`GroupedLookup::branch`]) — tính **ĐÚNG MỘT LẦN** ở
/// [`lookup_grouped`] và phải đi xuống **mọi** tệp là **cùng một** giá trị, không phải
/// N lần tính lại độc lập rồi tin một `debug_assert_eq!` — thứ **vô tác dụng ở bản
/// release** — giữ chúng khớp nhau. [`lookup`] (hàm ở trên) vẫn giữ nguyên bốn tham số cho
/// mọi chỗ gọi khác *(`tests/dict_lookup.rs`, thuộc Story 1.11/1.11b, không đổi)*.
pub(crate) fn lookup_with_branch(
    db: ReadHandle<'_>,
    query: &str,
    route: QueryRoute,
    branch: QueryBranch,
    limit: usize,
) -> SqlResult<LookupResult> {
    let (hits, truncated) = match branch {
        QueryBranch::ExactBtree => match route {
            QueryRoute::Zh => query::exact(db, query, limit)?,
            QueryRoute::En => query::exact_en(db, query, limit)?,
        },

        // Nhánh 2 là **của riêng đường `zh`** — [`pick_branch`] không bao giờ chọn nó
        // cho đường `En`. Câu SQL bên trong lọc `lang = 'zh'`, nên tổ hợp đó (nếu ai đó
        // dựng ra bằng tay) trả rỗng chứ không trả nhầm hàng tiếng Anh.
        QueryBranch::CharIdx => query::char_idx(db, query, limit)?,

        QueryBranch::FtsTrigram => match route {
            QueryRoute::Zh => query::fts_trigram(db, query, limit)?,
            QueryRoute::En => query::fts_trigram_en(db, query, limit)?,
        },

        // 🔴 **Không một câu SQL nào được chuẩn bị** — đó là mệnh đề của AD-44 ④, không
        // không phải một phép tối ưu. Trạng thái *"không hỗ trợ"* phải **phân biệt được**
        // với một lượt tra đã chạy mà không tìm thấy gì, và `branch` là chỗ nó khai ra.
        // `limit` không áp cho nhánh này — không câu SQL nào chạy nên không gì bị cắt.
        QueryBranch::NoBranchQueryTooShort => (Vec::new(), false),
    };

    Ok(LookupResult { branch, hits, truncated })
}

// ═════════════════════════════════════════════════════════════════════════════════
// Story 1.13 — TẦNG GOM: nhóm theo nguồn, KHÔNG hợp nhất (AD-19)
// ═════════════════════════════════════════════════════════════════════════════════

/// Một nguồn từ điển, khoá theo **`code` (chuỗi)**.
///
/// 🔴 **Không** `source_id: i64`, và đây là bẫy **im lặng nhất** của cả đường gom: mỗi
/// tệp `.db` mang bảng `dict_source` **RIÊNG**, nên `id = 1` tồn tại ở **cả ba** tệp và trỏ
/// ba nguồn khác nhau. Gom theo `id` dán nhãn *"Thiều Chửu"* cho một nghĩa thật ra của
/// CVDICT — **FR31 vỡ, không lỗi, không test hành vi nào đỏ** trừ khi ca test dùng
/// **ít nhất hai tệp**.
///
/// 🔴 **SÁU trường giấy phép của `dict_source` KHÔNG đọc ở đây** — `license_kind` ·
/// `license_id` · `license_text` · `attribution` · `source_version` · `source_url`.
/// *(Bản trước ghi "bốn"; `license_text` và `source_version` bị đếm sót. Số thật là **sáu**,
/// đo trên lược đồ `tools/dict-build/src/schema.rs`.)*
///
/// 🔴 **Story 1.19 §Quyết định #5a — chúng đi bằng một kiểu RIÊNG ([`SourceAttribution`]),
/// không nới kiểu này.** Lý do là một số đo, không một sở thích: `SourceInfo` nằm trong
/// **mọi** [`SourceGroup`] của **mọi** lượt tra — tức trên đúng đường nóng NFR1 mà
/// Auto-Lookup (Story 1.18) chạm hàng trăm lần mỗi Chương. Đo trên bốn tệp `.db` thật
/// 2026-08-08: `license_text` một mình là **43.304 ký tự**, và bảy nguồn của `dict-core.db`
/// cộng lại **~215 KB**. Nhồi ngần đó vào đây là đổ 215 KB qua IPC mỗi lần bôi đen.
/// ⚠️ **`Serialize`** (Story 1.17, Quyết định #2a).
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct SourceInfo {
    /// `dict_source.code` — định danh máy đọc, **duy nhất trong toàn tập lớp**.
    pub code: String,
    /// `dict_source.display_name` — tên hiển thị **của chính tệp chứa nó**.
    pub display_name: String,
}

/// **Ghi công đầy đủ của một nguồn** — Story 1.19, AC7/AC9. Đọc từ **chính tệp** mang nó.
///
/// 🔴 **Một kiểu RIÊNG với [`SourceInfo`], và một đường đọc RIÊNG** (§Quyết định #5a): kiểu
/// này chỉ đi qua dây khi màn hình Attribution mở — **một lần**, không mỗi lượt tra. Xem
/// doc-comment của [`SourceInfo`] cho số đo đứng sau quyết định đó.
///
/// 🔴 [`Self::license_kind`] là một **chuỗi mở, KHÔNG một enum** (AD-10, và
/// `tools/dict-build/src/schema.rs:26-31` viết sẵn lý do bằng chữ). Đo trên dữ liệu thật
/// 2026-08-08: **bốn** giá trị khác nhau đang tồn tại — `open` · `public-domain` ·
/// `copyrighted` · `unknown` — và **hai** trong bốn không có `license_id`. Tầng hiển thị
/// ánh xạ chuỗi này ra một câu ở `vi.json` và **phải có nhánh mặc định**: một giá trị chưa
/// gặp bao giờ *(một nguồn thêm ở bản sau)* phải ra một câu **có nghĩa**, không một ô trống
/// và không một chuỗi máy thô.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct SourceAttribution {
    /// `dict_source.code` — cùng khoá chuỗi với [`SourceInfo::code`] (Bẫy 3: `id` trùng
    /// giữa các tệp, `code` thì duy nhất trong **toàn tập lớp**).
    pub code: String,
    /// `dict_source.display_name`.
    pub display_name: String,
    /// `dict_source.license_kind` — `NOT NULL` trong lược đồ. Chuỗi mở; xem doc-comment kiểu.
    pub license_kind: String,
    /// `dict_source.license_id` — **cột NULL được duy nhất** trong sáu trường giấy phép.
    ///
    /// ⚠️ `None` là hình dạng THẬT, không một ca biên: `tran-van-chanh` và `vietphrase`
    /// đều mang `NULL` hôm nay. Màn hình **không** được suy nó từ [`Self::license_kind`],
    /// và **không** được hiện một ô trống — nó đọc câu của `license_kind` (AC9).
    pub license_id: Option<String>,
    /// **ĐỘ DÀI** của `dict_source.license_text`, không phải nội dung — §Quyết định #5a.
    ///
    /// 🔴 Văn bản giấy phép đầy đủ **không đi trên dây ở story này**: nó tới 43.304 ký tự
    /// cho một nguồn, và đường *"Mở văn bản giấy phép"* thuộc **Story 10.4** theo bảng chia
    /// đôi đã chốt. Con số này tồn tại để 10.4 biết nó phải mở cái gì, và để một tệp có
    /// `license_text` rỗng phân biệt được với một tệp có.
    pub license_text_len: i64,
    /// `dict_source.attribution` — `NOT NULL`.
    ///
    /// 🔴 **Hiện NGUYÊN VĂN, ĐẦY ĐỦ, không `text-overflow: ellipsis`** (AC7). Trường này
    /// không phải lúc nào cũng là một lời cảm ơn: `tran-van-chanh` mang một **cảnh báo pháp
    /// lý** trong đó *("CÒN TRONG BẢN QUYỀN, tác giả còn sống…")*, và cắt nó là cắt đúng
    /// nửa sau của một câu pháp lý.
    ///
    /// 🔴 Đây cũng là chỗ **DANH TÍNH TÁC GIẢ** sống — AC9: không một tên tác giả nào được
    /// viết cứng trong `src/**` hay `vi.json`. Đó là điều kiện để chỗ giữ `author-grant`
    /// dùng lại được cho một nguồn **khác** với một tác giả **khác**.
    pub attribution: String,
    /// `dict_source.source_version` — `NOT NULL`. Một trong hai trường mà lượt đếm "bốn
    /// trường giấy phép" của các story trước bỏ quên.
    pub source_version: String,
    /// `dict_source.source_url` — `NOT NULL`.
    pub source_url: String,
    /// `dict_meta('layer')` của **tệp chứa nguồn này** — `"base"` hoặc mã lớp gỡ rời.
    pub layer: String,
    /// Nguồn này thuộc lớp **NỀN** hay một lớp **GỠ RỜI** (AC7, cột *"Lớp"*).
    ///
    /// 🔴 Đọc từ `dict_meta('layer') == "base"` của chính tệp, **không** từ tên tệp và
    /// **không** từ một sổ đăng ký *(AD-44 ① vá A2)*. Xem [`BASE_LAYER_NAME`].
    pub is_base: bool,
    /// 🔴 **Tập ĐƯỜNG NGÔN NGỮ nguồn này phục vụ** — `dict_source.lang`, Story 1.19 AC6, Ice
    /// chốt ở code review 2026-08-10.
    ///
    /// Mã hoá *"cắt theo `,`, trim, bỏ rỗng"* — **cùng** quy ước [`crate::core::scope::
    /// parse_disabled_sources`], để webview không phải học một quy ước thứ hai. Hôm nay mọi
    /// nguồn thật cho đúng một giá trị (`"zh"` hoặc `"en"`), nhưng đây là một **TẬP** vì bất
    /// biến đó là một số đo chứ không một mệnh đề.
    ///
    /// 🔴 **Vì sao trường này phải tồn tại:** AC6 đòi trạng thái *"mọi nguồn đều tắt"* hỏi
    /// theo **đường đang tra**, không theo toàn tập. Đúng **MỘT** nguồn thật phục vụ đường
    /// tiếng Anh (`viwiktionary-en`); tắt riêng nó ⇒ mọi truy vấn tiếng Anh trả rỗng trong
    /// khi bảy nguồn tiếng Trung vẫn bật, và một vị từ hỏi *"toàn tập còn nguồn nào không"*
    /// trả `false`, nên panel nói *"không tìm thấy trong từ điển"* — một câu **SAI**, hệ
    /// thống không hề tra. Không có trường này thì webview không cách nào hỏi đúng câu.
    ///
    /// ⚠️ Đây **không** phải một sổ đăng ký `code → lang` (thứ AD-44 ① vá A2 cấm): giá trị
    /// được **ĐO từ `dict_entry` của chính tệp** lúc dựng *(`dict-build/src/insert.rs::
    /// backfill_source_langs`)*, đúng như `is_base` đọc từ `dict_meta` của chính tệp.
    pub lang: String,
}

/// Một **mục nghĩa** = một hàng `dict_sense`.
///
/// 🔴 **FR29: một từ nhiều từ loại ⇒ nhiều mục riêng biệt**, không nối `gloss` thành một
/// chuỗi. Một chuỗi nối là một quyết định trình bày chôn vào tầng dữ liệu, và 1.17 không
/// gỡ ngược ra được.
/// ⚠️ **`Serialize`** (Story 1.17, Quyết định #2a).
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct SenseRecord {
    /// `dict_sense.entry_id` — đầu mục mang nghĩa này, trong **chính tệp** của nó.
    pub entry_id: i64,
    /// `dict_sense.id` — khoá để ví dụ và trích dẫn treo vào (FR30).
    pub sense_id: i64,
    /// Nhãn từ loại. `None` khi nguồn không ghi.
    pub pos: Option<String>,
    /// 🔴 **FR35** — nhãn ngoại ngữ là một **TRƯỜNG**, không đoán từ nội dung [`Self::pos`].
    ///
    /// `tools/dict-build/src/schema.rs:57-58` viết sẵn lý do trường này tồn tại: *"`pos_lang`
    /// tồn tại vì **FR35** — nhãn từ loại ngoại ngữ phải được **ĐÁNH DẤU RÕ**, không đoán
    /// được từ nội dung `pos`"*. **Không** một bảng tra `"noun" ⇒ tiếng Anh` nào, ở bất kỳ
    /// tầng nào: một bảng như thế sai im lặng với mọi nhãn nó chưa gặp.
    ///
    /// Tầng này **không** dịch, không viết lại, không ẩn nhãn ngoại ngữ — 1.17
    /// **hiển thị** dấu hiệu đó; việc của story này là làm cho nó **không mất trên đường đi**.
    pub pos_lang: Option<String>,
    /// 🔴 **FR35 — nhãn này CÓ PHẢI nhãn NGOẠI NGỮ không.** Quyết định của **Rust**, không của
    /// webview (AD-1: quy tắc nghiệp vụ ở Rust).
    ///
    /// ⚠️ **không Đồng nghĩa với `pos_lang.is_some()`** — và đó là điểm. Một nhãn ghi
    /// `pos_lang = "vi"` là nhãn **tiếng Việt**: nó có ngôn ngữ, nhưng **không** ngoại
    /// ngữ, nên FR35 không đòi đánh dấu nó. Bản đầu của 1.17 bật dấu hiệu theo `pos_lang
    /// !== null` ở webview và dán chip `VI` lên đúng những nhãn bản ngữ (bắt ở code review
    /// 2026-08-07). Xem [`is_foreign_lang`] cho định nghĩa *"bản ngữ"*.
    pub pos_is_foreign: bool,
    /// Nghĩa. `NOT NULL` trong lược đồ.
    pub gloss: String,
    /// Ghi chú — phần **thứ sáu** trong sáu phần FR28 liệt kê.
    pub note: Option<String>,
    /// Thứ tự trong nguồn.
    ///
    /// ⚠️ **Không duy nhất**: `tools/dict-build/src/sources/vietphrase.rs` tách `/` vô
    /// điều kiện và sinh nhiều hàng **cùng `ord`**. Thứ tự tất định đến từ khoá phụ
    /// [`Self::sense_id`], không từ trường này một mình.
    pub ord: i64,
    /// **FR30** — ví dụ treo theo **TỪ LOẠI** (`sense_id`), không theo đầu mục.
    pub examples: Vec<ExampleRecord>,
    /// **FR30** — bảng **RIÊNG** với ví dụ: trích dẫn mang **xuất xứ**.
    pub citations: Vec<CitationRecord>,
}

/// 🔴 **Ngôn ngữ BẢN NGỮ của ứng dụng** — tiếng Việt. AuraTranslate là công cụ dịch **sang
/// tiếng Việt**, nên *"ngoại ngữ"* của FR35 nghĩa là *"không phải tiếng Việt"*.
///
/// ⚠️ Một hằng có tên chứ không một chuỗi `"vi"` rải trong mã: ngày ứng dụng có ngôn ngữ đích
/// thứ hai, đây là chỗ **duy nhất** phải đọc lại.
pub const NATIVE_LANG: &str = "vi";

/// 🔴 **FR35 — vị từ *"đây có phải nhãn NGOẠI NGỮ không"*, MỘT bản, ở Rust** (AD-1).
///
/// `None` ⇒ `false`: nguồn không ghi ngôn ngữ thì không có gì để đánh dấu, và AC4 cấm đích danh
/// việc mặc định thành *"tiếng Việt"* hay đoán từ nội dung nhãn.
/// `Some("vi")` ⇒ `false` — nhãn **bản ngữ**, không ngoại ngữ.
/// Mọi giá trị khác ⇒ `true`.
///
/// ⚠️ So khớp **không phân biệt hoa/thường** (`"EN"` = `"en"`): mã ngôn ngữ trong `dict_sense`
/// đến từ mười nguồn dựng khác nhau, và một `"VI"` viết hoa ở một nguồn không được phép biến
/// một nhãn bản ngữ thành ngoại ngữ. Cùng lý do và cùng cách với `verify_substring`:
/// [`str::eq_ignore_ascii_case`] không phụ thuộc locale.
pub fn is_foreign_lang(lang: Option<&str>) -> bool {
    match lang {
        None => false,
        Some(lang) => !lang.eq_ignore_ascii_case(NATIVE_LANG),
    }
}

/// Một ví dụ minh hoạ của **một nghĩa**.
/// ⚠️ **`Serialize`** (Story 1.17, Quyết định #2a).
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct ExampleRecord {
    /// Câu ví dụ như nguồn ghi.
    pub text: String,
    /// Bản dịch của ví dụ, nếu nguồn có.
    pub translation: Option<String>,
    /// 🔴 Ngôn ngữ của [`Self::translation`] — thứ để nói *"bản dịch ví dụ này là tiếng
    /// Anh"* mà không phải đoán từ nội dung. Bỏ trường này là làm FR35 không nghiệm
    /// thu được ở 1.17, và lỗi lộ ra ở **story sau**.
    pub translation_lang: Option<String>,
    /// 🔴 **FR35, cùng luật [`SenseRecord::pos_is_foreign`]** — AC4 nói *"cùng luật áp cho
    /// `ExampleRecord::translation_lang`"*, nên phép quyết định cũng ở **cùng một chỗ**
    /// ([`is_foreign_lang`]), không hai bản chép.
    pub translation_is_foreign: bool,
    /// Thứ tự trong nguồn. Cùng cảnh báo với [`SenseRecord::ord`].
    pub ord: i64,
}

/// Một **trích dẫn văn bản** của một nghĩa — bảng RIÊNG với ví dụ vì nó mang **xuất xứ**.
/// ⚠️ **`Serialize`** (Story 1.17, Quyết định #2a).
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct CitationRecord {
    /// Đoạn được trích.
    pub text: String,
    /// Tác phẩm. `None` khi nguồn không ghi.
    pub work: Option<String>,
    /// Tác giả. `None` khi nguồn không ghi.
    pub author: Option<String>,
    /// Thứ tự trong nguồn. Cùng cảnh báo với [`SenseRecord::ord`].
    pub ord: i64,
}

/// Một hàng thô của cột `dict_entry.han_viet` — Story 1.16, Quyết định #2.
///
/// 🔴 **`reading` chưa tách nhiều âm.** Một chuỗi như `"đinh|chênh"` (Thiều Chửu, phân
/// tách bằng `|`) hay `"tợ tử"` (nguồn khác, phân tách bằng khoảng trắng) đi qua **nguyên
/// văn** — tách chuỗi là việc của **tầng gom** ([`lookup_han_viet`], Quyết định #3), không
/// không phải của method này. Cổng `DictionarySource::han_viet` chỉ đọc **một tệp**, và
/// một tệp không biết quy ước phân tách nào đang áp — biết điều đó là biết *"tệp nào
/// chứa gì"*, đúng thứ AD-44 ① vá A2 cấm.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HanVietHit {
    /// Ký tự đã khớp — **giá trị nằm trong tập truy vấn** (`headword` hoặc
    /// `headword_simp` của hàng, tuỳ bên nào khớp), không phải luôn là `headword`.
    pub character: String,
    /// `dict_entry.han_viet` nguyên văn — có thể mang nhiều âm chưa tách.
    pub reading: String,
    /// `dict_source.code` — cùng luật khoá-theo-chuỗi với [`EntryHit::source_code`].
    pub source_code: String,
}

/// Một nhóm kết quả = **một nguồn**.
///
/// 🔴 **AD-19: không có bước hợp nhất nào, ở bất kỳ đâu.** Hai nguồn bất đồng về cùng
/// một đầu mục ⇒ **cả hai nhóm có mặt**, nghĩa giữ nguyên, không nhóm nào bị chọn làm
/// *"câu trả lời"* (FR32). Người dịch tự phán xét — đó là toàn bộ điểm của Epic 1.
/// ⚠️ **`Serialize`** (Story 1.17, Quyết định #2a).
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct SourceGroup {
    /// Danh tính lớp chứa nguồn này — `"base"` hoặc mã lớp gỡ rời.
    ///
    /// 🔴 Đây là đường vào **pha hai**: [`DictLayers::layer`] nhận đúng chuỗi này, và
    /// [`SenseRecord::entry_id`] chỉ có nghĩa **trong tệp của lớp đó**.
    pub layer: String,
    /// Nguồn — khoá là [`SourceInfo::code`].
    pub source: SourceInfo,
    /// Các đầu mục khớp, thứ tự `dict_entry.id` tăng dần.
    ///
    /// ⚠️ **Không bao giờ rỗng**: một nguồn đã tra mà không khớp gì ⇒ **không sinh
    /// nhóm**. *"Đã tra mà không khớp"* và *"lớp không nạp được"* không được phép trông
    /// giống nhau ở 1.17 — cái sau nằm ở [`GroupedLookup::skipped`].
    pub entries: Vec<EntryHit>,

    /// 🔴 **Tổng số đầu mục khớp CỦA NGUỒN NÀY, không bị trần cắt** — Quyết định #4 §hệ quả
    /// ③, đường (a). `None` ⇔ trần không chạm lớp này ⇒ [`Self::entries`] đã là **toàn bộ** và
    /// `entries.len()` chính là số thật.
    ///
    /// ⚠️ Trường này tồn tại để thanh nhịp không **khẳng định một con số nó không biết** (AC12):
    /// khi trần đã cắt, `entries.len()` là một **cận dưới**, không phải số thật — và một nguồn
    /// có thể bị cắt **sạch** khỏi [`GroupedLookup::groups`] mà vẫn có `total_entries > 0`
    /// ở đây (xem [`GroupedLookup::hidden_sources`]).
    pub total_entries: Option<i64>,
}

/// Kết quả **pha một** của một lượt tra trên **cả tập lớp**.
///
/// ⚠️ **`Serialize`** (Story 1.17, Quyết định #2a/#2a.2) — derive thẳng, **không một ngoại
/// lệ**: [`Self::skipped`] đi qua [`serialize_skipped_as_wire_codes`] thay vì serialize
/// [`SkippedLayer`] nguyên vẹn — [`SkipReason`] mang lỗi thô SQLite (`detail: String`) mà
/// AD-21 cấm đi lên giao diện. Kiểu Rust của trường **không đổi** (`Vec<SkippedLayer>`);
/// chỉ hình dạng TRÊN DÂY đổi.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct GroupedLookup {
    /// Đường đã đi — một **GIÁ TRỊ của cả lượt tra**, không phải của từng tệp.
    ///
    /// 🔴 [`pick_route`] chạy **ĐÚNG MỘT LẦN**, ở đây, và **cùng một** giá trị đi xuống
    /// **mọi** tệp (AD-44 ①). Để mỗi tệp tự tính là để hai tệp trả lời khác nhau ngay khi
    /// định nghĩa [`is_han`] của chúng lệch nhau.
    pub route: QueryRoute,

    /// Nhánh đã chạy — cũng là thuộc tính của **cả lượt tra**.
    ///
    /// 🔴 Gồm cả [`QueryBranch::NoBranchQueryTooShort`]: *"rỗng **có lý do**"*, không
    /// phải *"không có kết quả"* (AD-44 ④). Panel Lookup (1.17) đọc **đúng trường này**
    /// để nói *"truy vấn quá ngắn"* thay vì *"không tìm thấy"* — hai câu đó dẫn người
    /// dùng đi hai đường khác nhau.
    pub branch: QueryBranch,

    /// Một nhóm cho một nguồn, không nhóm rỗng nào. Xem [`SourceGroup`].
    pub groups: Vec<SourceGroup>,

    /// Các lớp không nạp được, hoặc nạp được mà lượt tra trên chúng hỏng.
    ///
    /// 🔴 **Giá trị, không phải một dòng log** — nó là thứ duy nhất phân biệt *"không
    /// có kết quả"* với *"một phần từ điển không trả lời"*.
    ///
    /// 🔴 **Trên dây (Quyết định #2a.2), trường này ra một MẢNG CHUỖI mã máy** — `["open_
    /// failed", "schema_too_new", …]`, **không** `path`, **không** `detail`. Panel chỉ
    /// cần *"một phần từ điển không trả lời"* (độ dài mảng > 0) cộng mã để chẩn đoán —
    /// không cần biết tệp nào hỏng thế nào. Xem [`serialize_skipped_as_wire_codes`].
    #[serde(serialize_with = "serialize_skipped_as_wire_codes")]
    pub skipped: Vec<SkippedLayer>,

    /// 🔴 **Quyết định #4 (Story 1.17), hệ quả ②** — danh tính các LỚP mà trần `limit` đã
    /// cắt bớt kết quả. **Không** phải `skipped` (lớp đó vẫn nạp và tra được bình
    /// thường) — nó là dấu hiệu *"tệp này có thể còn đầu mục khớp khác, kể cả của một
    /// nguồn khác, mà lượt tra vừa rồi không thấy"*. Panel Lookup đọc trường này để nói
    /// "danh sách nguồn chưa đầy đủ", không im (AC12).
    pub truncated_layers: Vec<String>,

    /// 🔴 **Các nguồn có đầu mục khớp mà trần đã cắt SẠCH khỏi [`Self::groups`]** —
    /// Quyết định #4 §hệ quả ③. Mỗi mục là `(display_name, số đầu mục)`.
    ///
    /// ⚠️ Đây là câu trả lời cho đúng ca mà AC12 dựng ra: `dict-core.db` mang nhiều nguồn
    /// trong **một** tệp, và một trần cấp-tệp có thể lấy hết chỗ cho nguồn có `entry_id`
    /// nhỏ hơn, làm nguồn kia **biến mất hoàn toàn**. Panel đọc trường này để nói ra
    /// **nguồn nào** đang vắng thay vì một câu chung chung — FR31 đòi *"mọi định nghĩa
    /// hiển thị nguồn"*, và một nguồn bị giấu tên là không hiển thị.
    ///
    /// Rỗng ⇔ không nguồn nào bị cắt sạch (kể cả khi [`Self::truncated_layers`] khác rỗng —
    /// trần có thể chỉ cắt bớt đầu mục **trong** các nguồn vẫn còn mặt).
    pub hidden_sources: Vec<(String, i64)>,

    /// 🔴 **AC6 (Story 1.17), ca thứ năm** — `false` ⇔ **không một lớp từ điển nào đang
    /// gắn** — trạng thái BÌNH THƯỜNG có tên (AD-25, `src-tauri/resources/dict/` rỗng
    /// trong git), và nó phải hiện ra bằng một chuỗi KHÁC với ca *"đã tra mà không tìm
    /// thấy"*.
    ///
    /// ⚠️ **Vì sao trường này phải tồn tại tường minh**: `groups` rỗng VÀ `skipped` rỗng
    /// xảy ra ở CẢ HAI ca — *"0 lớp"* (thư mục từ điển rỗng, `layers.layers()` rỗng) VÀ
    /// *"đã tra mà không khớp"* (có lớp, tra xong, không hàng nào khớp). Hai ca đó
    /// **không phân biệt được** chỉ từ `groups`/`skipped` — đúng doctrine
    /// `HanVietLookup::layers_loaded` của Story 1.16 đã áp lại ở đây.
    pub layers_loaded: bool,
}

/// 🔴 Chuyển `Vec<SkippedLayer>` thành một mảng **mã máy** — điều kiện `serialize_with`
/// của [`GroupedLookup::skipped`]. `SkippedLayer`/[`SkipReason`] **không bao giờ**
/// `derive(Serialize)`: bốn biến thể của `SkipReason` mang `detail: String` là lỗi thô
/// SQLite, và đi qua dây nguyên vẹn là vi phạm AD-21 ở đúng chỗ không cổng nào nhìn thấy
/// (`check-i18n.mjs` Kiểm A quét **chuỗi trong mã**, không quét **dữ liệu chạy qua dây**).
fn serialize_skipped_as_wire_codes<S>(skipped: &[SkippedLayer], serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    use serde::ser::SerializeSeq;
    let mut seq = serializer.serialize_seq(Some(skipped.len()))?;
    for item in skipped {
        seq.serialize_element(item.reason.wire_code())?;
    }
    seq.end()
}

/// **Pha một** — tra `query` trên **toàn bộ** tập lớp và nhóm kết quả theo nguồn.
///
/// 🔴 [`pick_route`] gọi **ĐÚNG MỘT LẦN**, ở đây; [`pick_branch`] cũng vậy. Cùng một
/// [`QueryRoute`] đi xuống **mọi** tệp (AD-44 ①, vá A1).
///
/// ⚠️ `mode` là **tham số từ chỗ gọi**, không đoán từ nội dung — cùng luật [`LookupMode`]
/// đã chốt ở Story 1.11.
///
/// **Hàm này không đọc nghĩa.** Xem §HAI PHA ở doc-comment module về vì sao — và về vì
/// sao một `LIMIT` ở đây là một quyết định sản phẩm thuộc về Story 1.17.
///
/// 🔴 `limit` **nhận từ chỗ gọi** (cùng doctrine `route`/`branch`): Panel Lookup
/// (`commands/dict.rs`) là nơi quyết chính sách trang, không phải tầng gom — cùng lý do
/// [`pick_route`] không tự chạy trong adapter.
///
/// ─────────────────────────────────────────────────────────────────────────────
/// 🔴 STORY 1.19 · §QUYẾT ĐỊNH #2a — `disabled` LÀ THAM SỐ, CÙNG DOCTRINE `route`/`limit`
/// ─────────────────────────────────────────────────────────────────────────────
/// Tập `dict_source.code` người dùng đã **TẮT**. Nó **nhận từ chỗ gọi** và **cùng một giá
/// trị** đi xuống **mọi** tệp — hàm này **không** đọc `Store`, và `core/dict/**` không gõ
/// tên một `code` nguồn cụ thể nào (AC4, canh bằng máy ở `tests/dict_boundary.rs`).
///
/// 🔴 **Lọc theo `code`, không theo tệp**, và **SAU** [`DictLayer::source`]: một tệp mang
/// nhiều nguồn *(`dict-core.db` mang bảy)*, nên *"tắt một nguồn"* không bao giờ đồng nghĩa
/// *"bỏ một lớp"*. Lớp vẫn được tra, vẫn báo `truncated`, vẫn góp `skipped` — chỉ những hàng
/// thuộc nguồn đã tắt bị bỏ.
///
/// ⚠️ **Trần `limit` chạy TRƯỚC phép lọc này** *(nó nằm trong câu SQL của từng tệp)*. Hệ quả
/// đo được: tắt một nguồn **không làm trang đầy hơn** — các nguồn còn lại giữ **đúng** tập
/// đầu mục cũ, không nhiều hơn và **không bao giờ ít hơn** (AC3). Đường lọc thẳng trong SQL
/// *(§Quyết định #2b)* sẽ cho trang đầy hơn, và nó là một **món nợ có số** chứ không một
/// khuyết tật — xem §Debug Log References của story cho tỉ lệ chạm trần đo được.
pub fn lookup_grouped(
    layers: &DictLayers,
    query: &str,
    mode: LookupMode,
    limit: usize,
    disabled: &BTreeSet<String>,
) -> GroupedLookup {
    let route = pick_route(query);
    let branch = pick_branch(query, mode, route);

    let mut groups: Vec<SourceGroup> = Vec::new();
    // Danh sách lớp hỏng lúc **mở** đi cùng **mọi** lượt tra: 1.17 không có đường nào
    // khác để biết một phần từ điển đang vắng mặt.
    let mut skipped: Vec<SkippedLayer> = layers.skipped().to_vec();
    let mut truncated_layers: Vec<String> = Vec::new();
    let mut hidden_sources: Vec<(String, i64)> = Vec::new();

    for layer in layers.layers() {
        let result = match layer.lookup(query, route, branch, limit) {
            Ok(result) => result,
            Err(err) => {
                // Một lớp hỏng lúc **tra** không được làm hỏng cả lượt tra — các lớp còn
                // lại vẫn trả lời, và lý do đi ra theo **giá trị** (AC4, cùng luật).
                skipped.push(SkippedLayer {
                    path: layer.path().to_path_buf(),
                    reason: SkipReason::LookupFailed {
                        detail: err.to_string(),
                    },
                });
                continue;
            }
        };

        if result.truncated {
            truncated_layers.push(layer.layer().to_owned());
        }

        // Nhóm **trong phạm vi một lớp** trước, rồi nối vào kết quả: thứ tự nhóm vì thế là
        // (thứ tự lớp, mã nguồn) — tất định, không phụ thuộc thứ tự hàng SQLite trả về.
        let mut in_layer: Vec<SourceGroup> = Vec::new();
        for hit in result.hits {
            // 🔴 Story 1.19 · AC3 — nguồn đã TẮT không sinh nhóm và không góp một đầu mục
            // nào. Lọc ở ĐÂY (sau `layer.lookup`, trước khi dựng nhóm) chứ không ở webview:
            // `hidden_sources` và `count_by_source` bên dưới đọc **cùng** tập này, nên thanh
            // nhịp không bao giờ đếm một nguồn mà màn hình không hiện (§Quyết định #2a lý do 2).
            if disabled.contains(&hit.source_code) {
                continue;
            }
            if let Some(group) = in_layer
                .iter_mut()
                .find(|group| group.source.code == hit.source_code)
            {
                group.entries.push(hit);
                continue;
            }

            // 🔴 `display_name` lấy từ **chính tệp** chứa đầu mục, không từ một bảng tra
            // `id → nguồn` dựng lại ở tầng gom (`deferred-work.md`, mục *"Khoá theo `code`
            // chứ không theo `id`"*).
            let Some(source) = layer.source(&hit.source_code) else {
                // `dict_source` của tệp không có `code` mà chính `JOIN` của nó vừa trả
                // về là bất khả trong một tệp toàn vẹn — bỏ hàng còn hơn dựng một nhãn
                // nguồn không ai xác nhận được. `debug_assert!` bắt ca này sớm lúc phát
                // triển; `eprintln!` giữ nó **nhìn thấy được** ở bản release, nơi assert
                // vô tác dụng — cùng bẫy AC5 nêu tên cho `char_idx()`, không lặp lại ở
                // đây một lần nữa dưới dạng im lặng tuyệt đối.
                debug_assert!(
                    false,
                    "a hit carries a source code that its own file does not declare"
                );
                eprintln!(
                    "dict[layers] {} has a hit with source code {} that its own dict_source \
                     does not declare; dropping the entry",
                    layer.path().display(),
                    hit.source_code
                );
                continue;
            };

            in_layer.push(SourceGroup {
                layer: layer.layer().to_owned(),
                source: source.clone(),
                entries: vec![hit],
                total_entries: None,
            });
        }

        in_layer.sort_by(|a, b| a.source.code.cmp(&b.source.code));

        // 🔴 §Hệ quả ③ đường (a) — **CHỈ KHI** trần đã cắt lớp này. Phần lớn lượt tra không
        // chạm trần, và bắt chúng trả giá một `COUNT` để phục vụ thiểu số là đúng thứ
        // §hệ quả ③ đã cân nhắc rồi loại. Một `COUNT` hỏng không được làm hỏng cả lượt tra:
        // không có số đếm thì `total_entries` ở lại `None` và thanh nhịp đọc như cận dưới,
        // không phải panic hay một lượt tra rỗng.
        if result.truncated {
            if let Ok(counts) = layer.count_by_source(query, route, branch) {
                for (code, total) in &counts {
                    // 🔴 Story 1.19 · Bẫy 2 — `count_by_source` là đường THỨ HAI, và nó chỉ
                    // chạy khi trần đã cắt, tức **phần lớn lượt tra không đi qua đây**. Một
                    // bản quên lọc ở chỗ này chạy đúng trong mọi test nhỏ và sai đúng trên
                    // truy vấn đông kết quả: thanh nhịp đọc *"7 nguồn"* trong khi màn hình
                    // hiện 4. AC12 của Story 1.17 cấm đích danh con số đó.
                    if disabled.contains(code) {
                        continue;
                    }
                    if let Some(group) = in_layer.iter_mut().find(|g| g.source.code == *code) {
                        group.total_entries = Some(*total);
                        continue;
                    }
                    // Nguồn có đầu mục khớp mà trần đã cắt SẠCH khỏi trang này — đúng ca
                    // FR31 mà AC12 dựng ra. Tên hiển thị lấy từ CHÍNH tệp (AC2), không một
                    // bảng tra dựng lại ở tầng gom.
                    if let Some(source) = layer.source(code) {
                        hidden_sources.push((source.display_name.clone(), *total));
                    }
                }
            }
        }

        groups.append(&mut in_layer);
    }

    GroupedLookup {
        route,
        branch,
        groups,
        skipped,
        truncated_layers,
        hidden_sources,
        layers_loaded: !layers.layers().is_empty(),
    }
}

// ═════════════════════════════════════════════════════════════════════════════════
// Story 1.19 — GHI CÔNG: dựng từ TỆP CÓ MẶT, không từ một danh sách viết cứng
// ═════════════════════════════════════════════════════════════════════════════════

/// **Ghi công của mọi nguồn trong mọi tệp đang gắn** — Story 1.19, AC1 · AC7 · AC8.
///
/// 🔴 **Dẫn xuất từ [`DictLayers`], KHÔNG từ một hằng trong mã và KHÔNG từ một bảng trong
/// `global.db`** (AD-44 ① vá A2). Hệ quả trực tiếp, và cả hai đều nghiệm thu được bằng phép
/// thử của AD-10 *(thả một tệp vào thư mục / xoá một tệp đi)*:
/// - thêm một tệp `.db` ⇒ ghi công của nó xuất hiện, **không sửa một dòng mã**;
/// - xoá một tệp `.db` ⇒ ghi công của **mọi** nguồn trong tệp đó biến mất, **0** mục mồ côi.
///
/// 🔴 **Nguồn bị TẮT vẫn có mặt đầy đủ ở đây** (AC10) — hàm này **không** nhận tập bị tắt,
/// và đó là một mệnh đề chứ không một thiếu sót: *"tắt"* chỉ giấu một nguồn khỏi **kết quả
/// tra cứu**; *"gỡ"* là xoá tệp dữ liệu và là việc của **người đóng gói** (FR112). Một bảng
/// ghi công rụng mất một hàng vì người dùng tắt một chip là bảng ghi công **sai** — nghĩa vụ
/// CC-BY-SA gắn với việc **phân phối** dữ liệu, không với việc hiển thị nó.
///
/// ⚠️ Một lớp mà `dict_source` **không đọc được lúc này** bị bỏ khỏi bảng, kèm một dòng
/// chẩn đoán ra `stderr`: nửa bảng còn hơn không bảng nào, và cùng luật rỗng-có-lý-do mà
/// [`lookup_grouped`] áp cho một lớp hỏng lúc tra.
///
/// Thứ tự tất định: thứ tự lớp của [`DictLayers::layers`], rồi `ORDER BY code` trong tệp.
pub fn list_source_attributions(layers: &DictLayers) -> Vec<SourceAttribution> {
    let mut out: Vec<SourceAttribution> = Vec::new();
    for layer in layers.layers() {
        match layer.attributions() {
            Ok(mut rows) => out.append(&mut rows),
            Err(err) => eprintln!(
                "dict[layers] cannot read dict_source for attribution from {}: {err}",
                layer.path().display()
            ),
        }
    }
    out
}

// ═════════════════════════════════════════════════════════════════════════════════
// Story 1.16 — TẦNG GOM ÂM HÁN VIỆT: thứ tự ưu tiên theo LỚP, tách nhiều âm, một lượt
// cho cả CHƯƠNG (AC5, Quyết định #1 & #3)
// ═════════════════════════════════════════════════════════════════════════════════

/// Danh tính của lớp **nền** — cùng giá trị với `layer::BASE_LAYER`, không phải một
/// hằng thứ hai đứng ĐỘC LẬP với nó: cả hai đều đọc từ `dict_meta('layer')` của chính
/// tệp, **không** phải một sổ đăng ký tên tệp (AD-44 ① vá A2). So sánh `layer() ==
/// "base"` khác về bản chất với so `code == "thieu-chuu"`: `"base"`/`"gỡ rời"` là một
/// PHÂN LOẠI CẤU TRÚC mà chính tầng dữ liệu đã gắn nhãn cho mọi tệp (đúng một trong hai),
/// không phải danh tính của MỘT nguồn cụ thể nào.
///
/// 🔴 **MỘT hằng, không hai bản chép.** Bản đầu của Story 1.16 khai một hằng thứ hai ở
/// đây cạnh [`layer::BASE_LAYER`] đã có. Hai hằng độc lập cho cùng một giá trị, không
/// một lưới canh nào: đổi một bên mà quên bên kia làm [`priority_order`] **đảo ngược im
/// lặng** — lớp nền thắng mọi lớp gỡ rời, tức lật đúng Quyết định #1 của story mà không
/// một cổng nào đỏ. Lượt code review 2026-08-06 gộp lại thành một `pub(super)`.
use layer::BASE_LAYER as BASE_LAYER_NAME;

/// Âm Hán Việt của **một** ký tự, đã qua tầng gom — hoặc **không có âm nào**.
///
/// 🔴 `Option`, **không** một chuỗi rỗng: *"ký tự này không có âm ở bất kỳ lớp nào"*
/// và *"ký tự này có một âm rỗng"* là hai câu khác nhau, và câu thứ hai không tồn tại
/// trong dữ liệu thật — `HAN_VIET_SQL` đã lọc `IS NOT NULL`.
///
/// ⚠️ `Serialize` — kiểu này đi qua IPC nguyên vẹn (`commands::dict::wire::read_han_viet`),
/// cùng tiền lệ `core::library::WorkMeta`. Không `#[serde(rename_all = "camelCase")]` —
/// mọi trường đã `snake_case`, đúng như trên dây (AD-21).
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct CharacterReading {
    /// Ký tự trong nguyên văn — có thể LẶP LẠI giữa các phần tử của
    /// [`HanVietLookup::characters`], vì đầu ra giữ **đúng vị trí** của `chars` chỗ gọi
    /// truyền vào (Panel Source render theo vị trí, không theo tập ký tự duy nhất).
    pub character: String,
    /// `None` ⇔ không lớp nào (đang gắn) mang âm cho ký tự này — trạng thái **đã tra mà
    /// không có**, khác với ca "0 lớp gắn" (đọc ở [`HanVietLookup::layers_loaded`]).
    pub reading: Option<HanVietReading>,
}

/// Âm Hán Việt đã CHỌN cho một ký tự — kết quả của thứ tự ưu tiên theo lớp.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct HanVietReading {
    /// Âm **ĐẦU TIÊN** sau khi tách nhiều âm (Quyết định #3) — cái tab hiện lên màn hình.
    pub primary: String,
    /// Toàn bộ danh sách âm đã tách, giữ nguyên thứ tự của nguồn — Story 1.17 (Panel
    /// Lookup) và 3.7 (FR113) cần danh sách đầy đủ, không chỉ âm đầu tiên.
    pub all: Vec<String>,
    /// `dict_source.code` của lớp đã THẮNG ưu tiên cho ký tự này (FR31 — nguồn bắt buộc
    /// trên mọi bản ghi).
    pub source_code: String,
}

/// Kết quả của một lượt gom âm Hán Việt cho **toàn bộ** ký tự do chỗ gọi truyền vào.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct HanVietLookup {
    /// Một phần tử cho **mỗi** ký tự chỗ gọi truyền vào, ĐÚNG vị trí và ĐÚNG số lượng —
    /// kể cả ký tự lặp lại nhiều lần trong văn bản.
    pub characters: Vec<CharacterReading>,
    /// `dict_source.code` của MỌI nguồn đã đóng góp ít nhất một âm cho lượt này — deduped,
    /// sắp theo `code`. Quyết định #1, mệnh đề 3: hiển thị **một** dòng `ui-label` liệt kê
    /// nguồn cho cả lượt, không một nhãn cho mỗi ký tự.
    pub sources_used: Vec<String>,
    /// `false` ⇔ **không một lớp từ điển nào đang gắn** — trạng thái BÌNH THƯỜNG có tên
    /// (AD-25, `src-tauri/resources/dict/` rỗng trong git), và nó phải hiện ra bằng một
    /// chuỗi KHÁC với ca "đã tra mà ký tự này không có âm" (AC4 — ba trạng thái, không
    /// một; doctrine `QueryBranch::NoBranchQueryTooShort` của Story 1.13 áp lại ở đây).
    pub layers_loaded: bool,
}

/// Tách một chuỗi `dict_entry.han_viet` **thô** thành các âm riêng biệt — Quyết định #3(a).
///
/// 🔴 **MỘT luật áp cho MỌI tệp**: cắt trên `|` **và** `,` **và** khoảng trắng. Không mã
/// riêng cho từng tệp/nguồn (AD-10) — an toàn vì một âm Hán Việt là MỘT âm tiết tiếng Việt,
/// và nó không bao giờ tự chứa `|`, `,` hay khoảng trắng.
///
/// 🔴 **BA quy ước tồn tại song song, không phải hai** — đo trên bốn tệp `.db` thật ở
/// `tools/dict-build/out/`, lượt code review 2026-08-06:
///
/// | Tệp | hàng có `han_viet` | chứa `,` | chứa `\|` | chứa khoảng trắng |
/// |---|---|---|---|---|
/// | `dict-core.db` *(lớp NỀN)* | 1.145 | **284 = 24,8 %** | 0 | 1 |
/// | `dict-tran-van-chanh.db` *(gỡ rời, ưu tiên CAO NHẤT)* | 22.030 | **2.326** | 0 | 2.397 |
/// | `dict-thieu-chuu.db` | 9.897 | 0 | 1.639 | 15 |
///
/// ⚠️ **Bản đầu của Story 1.16 bỏ sót `,`** và mục bàn giao của `1-10c` đã cảnh báo đích
/// danh ba quy ước. Hai kiểu hỏng thật nó gây ra:
/// - `西 → "tây,tê"` *(không khoảng trắng)* tách ra **một** phần tử ⇒ tab hiện nguyên
///   chuỗi `tây,tê` như thể đó là **một** âm.
/// - `譫 → "chiêm, thiềm"` tách trên khoảng trắng ⇒ `["chiêm,", "thiềm"]` ⇒ `primary` mang
///   **dấu phẩy đuôi** lên màn hình — `.map(str::trim)` chỉ cắt khoảng trắng, không cắt `,`.
///
/// 🔴 Và nó rơi vào đúng chỗ tệ nhất: **24,8 % của lớp NỀN** — chính lớp mà FR36 rơi về khi
/// mọi lớp gỡ rời bị xoá.
///
/// Hàm **thuần**, không chạm database — điều kiện để test chạy trên cả hai hình dạng
/// thật mà không cần một tệp `.db` nào.
fn split_readings(raw: &str) -> Vec<String> {
    raw.split(|c: char| c == '|' || c == ',' || c.is_whitespace())
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_owned)
        .collect()
}

/// Lớp nào đọc TRƯỚC lớp nào, cho MỘT lượt gom — tính **đúng một lần**, không lặp lại
/// cho mỗi ký tự (cùng doctrine `route`/`branch` của [`lookup_grouped`]).
///
/// Quy tắc **phát biểu được**, không viết cứng tên lớp: *"lớp gỡ rời đứng trước dữ liệu tổng
/// hợp"* — trong nhóm lớp gỡ rời, giữ nguyên thứ tự ổn định của [`DictLayers::layers`]
/// (`base` trước rồi mã lớp tăng dần); lớp NỀN bị đẩy xuống cuối, sau MỌI lớp gỡ rời.
fn priority_order(layers: &DictLayers) -> Vec<&DictLayer> {
    let mut detachable: Vec<&DictLayer> = Vec::new();
    let mut base: Vec<&DictLayer> = Vec::new();
    for layer in layers.layers() {
        if layer.layer() == BASE_LAYER_NAME {
            base.push(layer);
        } else {
            detachable.push(layer);
        }
    }
    detachable.extend(base);
    detachable
}

/// **Tầng gom âm Hán Việt** — Story 1.16, AC5.
///
/// 🔴 Đọc theo LÔ, một lượt gọi [`DictionarySource::han_viet`] cho **mỗi lớp** (không
/// cho mỗi ký tự) — dedupe `chars` **trước khi tra**, cùng lý do `senses.rs`/`han_viet.rs`.
/// Đầu ra giữ nguyên vị trí VÀ số lượng của `chars` (kể cả ký tự lặp) — chỗ gọi (Panel
/// Source) zip trực tiếp với văn bản gốc, không tự tra lại theo tập duy nhất.
///
/// ─────────────────────────────────────────────────────────────────────────────
/// 🔴 STORY 1.19 · §QUYẾT ĐỊNH #3a — BỘ LỌC NGUỒN **CÓ** ÁP CHO ĐƯỜNG NÀY
/// ─────────────────────────────────────────────────────────────────────────────
/// FR37 nói *"kết quả từ nguồn đó **không xuất hiện**"*. Âm Hán Việt **là** một kết quả tra
/// cứu mang `source_code` (FR31), và [`HanVietLookup::sources_used`] **viết tên nguồn lên
/// màn hình**. Để đường này ngoài bộ lọc là để một nguồn *"đã tắt"* vẫn viết chữ lên tab
/// Hán Việt — một câu tự mâu thuẫn ngay trên màn hình.
///
/// ⚠️ **HỆ QUẢ BẮT BUỘC, và nó không hiển nhiên:** [`priority_order`] đẩy lớp NỀN xuống
/// cuối, nên tắt một lớp gỡ rời **ĐỔI ÂM hiển thị** chứ không chỉ giấu bớt — một ký tự có
/// thể đi từ âm của lớp gỡ rời về âm của lớp nền. Đó là hành vi **ĐÚNG** *(cùng cơ chế mà
/// FR36 dựa vào khi một lớp bị gỡ khỏi bản cài)*, nhưng nó phải **đo và ghi ra**, không để
/// người đọc phát hiện sau — xem `tests/dict_sources.rs` §Bẫy 6, ca khẳng định **âm cụ thể**
/// chứ không chỉ khẳng định `sources_used` sạch.
///
/// 🔴 Lọc **TRƯỚC** khi chọn ưu tiên, không sau: lọc sau nghĩa là một ký tự mà lớp thắng đã
/// bị tắt sẽ trả `None` thay vì rơi về lớp kế tiếp — tức *"tắt một nguồn"* biến thành *"xoá
/// âm của ký tự đó"*, đúng thứ FR36 tồn tại để không xảy ra.
pub fn lookup_han_viet(
    layers: &DictLayers,
    chars: &[&str],
    disabled: &BTreeSet<String>,
) -> HanVietLookup {
    use std::collections::HashMap;

    let order = priority_order(layers);

    let mut seen = std::collections::HashSet::new();
    let unique: Vec<&str> = chars.iter().copied().filter(|c| seen.insert(*c)).collect();

    // Mỗi lớp trả về các hàng KHỚP (chưa chọn ưu tiên) cho TOÀN BỘ tập ký tự duy nhất —
    // một lượt gọi cho mỗi lớp, không N lượt cho N ký tự.
    let mut by_layer: Vec<(&str, HashMap<&str, &HanVietHit>)> = Vec::new();
    let mut hits_storage: Vec<Vec<HanVietHit>> = Vec::new();

    for layer in &order {
        match layer.han_viet(&unique) {
            Ok(hits) => hits_storage.push(hits),
            // Một lớp hỏng lúc tra không được làm hỏng cả lượt gom — cùng luật
            // `lookup_grouped`. FR36 nghiệm thu ở mức "vẫn chạy", không "không lỗi nào
            // từng xảy ra".
            Err(_) => hits_storage.push(Vec::new()),
        }
    }
    for (layer, hits) in order.iter().zip(hits_storage.iter()) {
        // 🔴 Hàng ĐẦU TIÊN thắng cho mỗi ký tự TRONG CÙNG một lớp — `HAN_VIET_SQL` sắp
        // `ORDER BY e.id`, và `read_han_viet` lọc theo tập của **chính lô** nên mọi hit của
        // một ký tự đều phát ra từ đúng một lô ⇒ đây là đầu mục có `id` nhỏ nhất mang ký tự
        // đó, xác định.
        //
        // ⚠️ Mệnh đề này ĐỨNG được **nhờ** phép lọc theo lô ở `han_viet.rs`. Với phép lọc
        // theo tập đầy đủ (bản đầu của Story 1.16), `out` nối theo thứ tự LÔ chứ không
        // theo `e.id`, và `or_insert` dưới đây chọn theo thứ tự đến — tức âm phụ thuộc vị
        // trí ký tự trong Chương. Đừng nới phép lọc đó ra mà không đọc lại dòng này.
        let mut per_char: HashMap<&str, &HanVietHit> = HashMap::new();
        for hit in hits {
            // Story 1.19 · §Quyết định #3a — hàng của một nguồn đã TẮT không được vào cuộc
            // đua ưu tiên. Bỏ ở đây (không ở vòng chọn dưới) để một ký tự vẫn rơi về lớp kế
            // tiếp thay vì mất âm.
            if disabled.contains(&hit.source_code) {
                continue;
            }
            per_char.entry(hit.character.as_str()).or_insert(hit);
        }
        by_layer.push((layer.layer(), per_char));
    }

    let mut sources_used: std::collections::BTreeSet<String> = std::collections::BTreeSet::new();
    let mut resolved: HashMap<&str, HanVietReading> = HashMap::new();

    for char_key in &unique {
        for (_layer_name, per_char) in &by_layer {
            if let Some(hit) = per_char.get(char_key) {
                let all = split_readings(&hit.reading);
                let Some(primary) = all.first().cloned() else {
                    // Chuỗi thô toàn `|`/khoảng trắng — dữ liệu hỏng ở tầng nguồn; bỏ qua
                    // hàng này, không panic, và tiếp tục xét lớp ưu tiên kế tiếp.
                    continue;
                };
                sources_used.insert(hit.source_code.clone());
                resolved.insert(
                    char_key,
                    HanVietReading {
                        primary,
                        all,
                        source_code: hit.source_code.clone(),
                    },
                );
                break;
            }
        }
    }

    let characters = chars
        .iter()
        .map(|c| CharacterReading {
            character: (*c).to_owned(),
            reading: resolved.get(c).cloned(),
        })
        .collect();

    HanVietLookup {
        characters,
        sources_used: sources_used.into_iter().collect(),
        layers_loaded: !layers.layers().is_empty(),
    }
}

// ⚠️ Ca hành vi trên HAI HÌNH DẠNG THẬT ("đinh|chênh" · "tợ tử") sống ở
// `tests/dict_sources.rs::multiple_readings_split_on_both_the_pipe_and_whitespace_conventions`
// — KHÔNG lặp lại ở đây bằng chuỗi tiếng Việt: `src-tauri/src/**/*.rs` nằm trong phạm vi
// Kiểm A của `check-i18n.mjs` (không chuỗi tiếng Việt CÓ DẤU ở vị trí mã), trong khi
// `src-tauri/tests/**` được miễn trừ (thông báo test, không vượt IPC). Unit test dưới đây
// dùng chuỗi ASCII trung tính — cùng cơ chế tách, chỉ khác bộ ký tự — cho các ca biên mà
// test hành vi ở `tests/` không phủ (đệm nhiều dấu phân tách liên tiếp, một âm duy nhất).
#[cfg(test)]
mod split_readings_tests {
    use super::split_readings;

    #[test]
    fn splits_on_the_pipe_convention() {
        assert_eq!(split_readings("ab|cd"), vec!["ab", "cd"]);
    }

    #[test]
    fn splits_on_the_whitespace_convention() {
        assert_eq!(split_readings("ab cd"), vec!["ab", "cd"]);
    }

    #[test]
    fn a_single_reading_is_a_list_of_one() {
        assert_eq!(split_readings("ab"), vec!["ab"]);
    }

    #[test]
    fn consecutive_delimiters_never_produce_an_empty_reading() {
        assert_eq!(split_readings("a||b  c"), vec!["a", "b", "c"]);
        assert_eq!(split_readings(" a "), vec!["a"]);
    }
}
