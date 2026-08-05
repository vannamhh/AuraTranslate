//! Tra cứu từ điển — ba nhánh tiếng Trung (AD-26) và hai nhánh tiếng Anh (AD-44), chọn
//! bằng một **vị từ điều phối** đứng trên cả hai.
//!
//! KHÔNG tồn tại bước hợp nhất nguồn (AD-19): mỗi kết quả luôn mang `source` của nó.
//! Mỗi lớp gỡ rời là một file `.db` độc lập, chỉ đọc (AD-10, AD-25).
//!
//! Crate dành cho module này: `rusqlite` (đọc `.db`) — dùng chung cài đặt với `core::store`.
//!
//! ⚠️ Câu trên là **tài liệu về một ranh giới**, ⛔ không phải một lời gọi vượt qua nó:
//! module này ⛔ **không** gõ tên crate SQLite ở một vị trí mã nào. Nó viết truy vấn qua
//! các kiểu **tái xuất** của [`crate::core::store`] — [`ReadHandle`], [`SqlResult`],
//! [`Row`] — và nhận kết nối từ chỗ gọi. Đường mở tệp sống ở
//! [`crate::core::store::ReadOnlyDb`]; xem doc-comment ở đó về vì sao nó ở đấy chứ
//! không ở đây.
//!
//! ─────────────────────────────────────────────────────────────────────────────
//! HAI ĐƯỜNG, NĂM Ô — VÀ NHÁNH ĐƯỢC CHỌN BẰNG SỐ **KÝ TỰ**, ⛔ KHÔNG BẰNG SỐ BYTE
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
//! **Đường `en`** — mọi thứ còn lại. Lọc `lang = 'en'`. **HAI** nhánh, ⛔ không phải ba.
//!
//! | Chế độ                  | Độ dài *(ký tự)* | Nhánh                                  | Chỉ mục dùng          |
//! |-------------------------|------------------|----------------------------------------|-----------------------|
//! | Tra chính xác đầu mục   | bất kỳ           | [`QueryBranch::ExactBtree`]             | `idx_entry_headword`  |
//! | Chuỗi con               | ≥ 3              | [`QueryBranch::FtsTrigram`]             | FTS5 `entry_fts`      |
//! | Chuỗi con               | < 3 *(gồm cả 0)* | 🔴 [`QueryBranch::NoBranchQueryTooShort`] | — *(⛔ không nhánh nào chạy)* |
//!
//! 🔴 ⛔ **Không** ô `char_idx` cho đường tiếng Anh, và đó là một **số đo**: lớp
//! `viwiktionary-en` sinh **đúng 9** cặp `char_idx` trên **119.039** đầu mục (0,0076%).
//!
//! 🔴 **Phép đo độ dài là `chars().count()`, ⛔ không bao giờ là `len()`.** `"山".len()`
//! là **3** (UTF-8) và `"中國".len()` là **6** — chọn nhánh theo `len()` đẩy **mọi** truy
//! vấn tiếng Trung 1–2 ký tự vào FTS5 trigram, nơi chúng trả **0** hàng trong 0,01 ms mà
//! ⛔ không lỗi nào được ném. Đó chính xác là phát hiện nghiêm trọng nhất của mũi thăm dò
//! Giai đoạn 0, là lý do FR39 tồn tại, và là lý do AD-26 khai **ba** nhánh chứ không hai.
//! Đo được trên tệp thật: `entry_fts MATCH '"山"'` ⇒ 0 hàng, `char_idx` ⇒ 3.177 hàng.
//!
//! ⛔ **Không fallback dây chuyền** *(thử nhánh 1, rỗng thì thử nhánh 2…)*. AD-26 nói
//! *"tra chính xác → B-tree"*, ⛔ không nói *"thử B-tree trước"*. Một fallback ngầm làm
//! mỗi lượt tra chạy hai đến ba truy vấn — tức số đo NFR1 thành vô nghĩa — và làm
//! [`LookupResult::branch`] **nói dối** về đường đã đi.
//!
//! ─────────────────────────────────────────────────────────────────────────────
//! ⛔ PHẠM VI: MỘT TỆP, MỘT LƯỢT
//! ─────────────────────────────────────────────────────────────────────────────
//! [`lookup`] chạy trên **một** kết nối tới **một** tệp `.db`. Nó ⛔ không gom kết quả
//! nhiều tệp, ⛔ không nhóm theo nguồn, ⛔ không đọc `dict_sense`/`dict_example`/
//! `dict_citation`, và ⛔ không hợp nhất đầu mục trùng — cả bốn là **Story 1.13**, và
//! AD-19 nói cái cuối cùng ⛔ không bao giờ xảy ra.

mod query;

use crate::core::store::{ReadHandle, SqlResult};

/// Đường tra cứu — **đã quyết ở tầng trên**, adapter ⛔ không tự quyết lại (AD-44 ①).
///
/// 🔴 **NHỊ PHÂN, ⛔ không có nhánh thứ ba.** Một biến thể `Unknown` đẩy câu hỏi *"làm gì
/// với nó"* xuống **mọi** chỗ gọi, và mỗi chỗ gọi sẽ trả lời khác nhau. Một truy vấn ⛔
/// không thuộc hệ chữ nào của hai từ điển vẫn chạy một nhánh **thật** ở đường `En` và trả
/// **rỗng có lý do** — thứ nghiệm thu được — thay vì rỗng vì ⛔ không ai chọn nhánh.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QueryRoute {
    /// Truy vấn chứa ít nhất một ký tự Hán ⇒ ba nhánh của AD-26, lọc `lang = 'zh'`.
    Zh,
    /// Mọi thứ còn lại ⇒ hai nhánh của AD-44 ②, lọc `lang = 'en'`.
    En,
}

/// Vị từ điều phối — **hình dạng CHUỖI TRUY VẤN**, ⛔ không phải ngôn ngữ của Tác phẩm.
///
/// Hàm thuần, ⛔ không chạm database — điều kiện để AC1 nghiệm thu được mà ⛔ **không cần
/// một tệp `.db` nào**, tức trong CI, nơi ⛔ không có tệp từ điển nào (`.gitignore: *.db`).
///
/// 🔴 **Gọi ĐÚNG MỘT LẦN cho mỗi lượt tra**, ở tầng gom (Story 1.13) — ⛔ không bên trong
/// [`lookup`], ⛔ không bên trong `query.rs`. Để vị từ chạy **trong** adapter là để mỗi
/// tệp `.db` tự trả lời một câu hỏi thuộc về **cả lượt tra**, và hai tệp sẽ trả lời khác
/// nhau ngay khi định nghĩa [`is_han`] của chúng lệch nhau.
///
/// ⚠️ Vị từ nói về **script**, ⛔ không nói về **ngôn ngữ**: `"日本語"` chứa kanji nên nó đi
/// đường [`QueryRoute::Zh`]. Đó là hành vi **đúng** theo AD-44, ⛔ không phải một lỗi —
/// `dict-core.db` ⛔ không mang một hàng tiếng Nhật nào, và một nhánh thứ ba cho tiếng
/// Nhật là thứ ⛔ không có dữ liệu để tra.
///
/// ⛔ **Không** điều phối theo ngôn ngữ của Tác phẩm: bôi đen `API` trong một truyện tiếng
/// Trung phải ra kết quả, ⛔ không ra rỗng (AD-44 Prevents #2). Một tham số duy nhất là
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
/// ⛔ không trôi khỏi nhau: nó đọc tệp của workspace kia **dưới dạng văn bản**, ⛔ không
/// import chéo crate — hai workspace tách rời **có chủ ý** (AC4 của Story 1.9), và gọi
/// chéo là hút build tool vào cây phụ thuộc của sản phẩm.
///
/// ⚠️ Vì sao chép chứ ⛔ không thu hẹp: một bộ dải hẹp hơn (vd. chỉ BMP) đọc `𠧜`
/// (U+209DC) thành *"không phải chữ Hán"* ⇒ [`pick_route`] trả [`QueryRoute::En`] ⇒ truy
/// vấn chạy nhánh tiếng Anh, lọc `lang = 'en'`, và trả **rỗng** cho một đầu mục tiếng
/// Trung có thật — rỗng, ⛔ **không lỗi**, đúng lớp lỗi AD-26 ra đời để chặn.
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

/// Chế độ tra — do **chỗ gọi** quyết, ⛔ không đoán từ nội dung truy vấn.
///
/// 🔴 Một hàm tự đoán *"chắc người dùng muốn tra chính xác"* là một quy tắc nghiệp vụ
/// **ẩn** mà Auto-Lookup (1.18) và Panel Lookup (1.17) sẽ phải đoán ngược lại. AD-26 khai
/// ba nhánh, ⛔ không khai một cơ chế đoán.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LookupMode {
    /// Đầu mục **bằng đúng** truy vấn.
    Exact,
    /// Đầu mục **chứa** truy vấn như một chuỗi con.
    Substring,
}

/// Đường đã đi thật sự — **giá trị trả về**, ⛔ không phải một dòng log.
///
/// 🔴 Nhánh phải **quan sát được từ ngoài**, và đó là điều kiện để Bẫy `len()` ở trên
/// nghiệm thu được: một `eprintln!` ⛔ không khẳng định được trong test, nên một cài đặt
/// chọn sai nhánh sẽ đi qua mọi phép kiểm *"kết quả khác rỗng"* mà không ai thấy.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QueryBranch {
    /// Nhánh 1 — B-tree trên `headword` / `headword_simp`. Dùng ở **cả hai** đường.
    ExactBtree,
    /// Nhánh 2 — bảng đảo ngược `char_idx`, cho chuỗi con 1–2 ký tự. **Chỉ** đường `zh`.
    ///
    /// ⛔ Đường tiếng Anh ⛔ **không bao giờ** đi nhánh này, và đó là một **số đo** chứ ⛔
    /// không phải một sở thích: lớp `viwiktionary-en` sinh **đúng 9** cặp `char_idx` trên
    /// **119.039** đầu mục (0,0076%). Bảng đảo ngược ⛔ không áp được cho tiếng Anh.
    CharIdx,
    /// Nhánh 3 — FTS5 `entry_fts` với tokenizer `trigram`, cho chuỗi con ≥ 3 ký tự.
    /// Dùng ở **cả hai** đường.
    FtsTrigram,
    /// 🔴 ⛔ **Không nhánh nào chạy** — chuỗi con tiếng Anh < 3 ký tự (AD-44 ④).
    ///
    /// ⛔ **Không phải "không có kết quả":** nó là một trạng thái **KHÔNG HỖ TRỢ**, và
    /// Panel Lookup (FR41, Story 1.17) nói *"truy vấn quá ngắn"* chứ ⛔ không nói *"không
    /// tìm thấy"*. Hai câu đó dẫn người dùng đi hai đường khác nhau: một câu bảo *"gõ
    /// thêm"*, câu kia bảo *"từ này ⛔ không có trong từ điển"*.
    ///
    /// Vì sao ⛔ không hạ ngưỡng trigram xuống 1: FTS5 `trigram` ⛔ **không** lập chỉ mục
    /// token ngắn hơn ba ký tự — đo được `entry_fts MATCH '"山"'` ⇒ **0** hàng. Để một
    /// truy vấn 1–2 ký tự chạy nhánh trigram là để nó trả **rỗng im lặng**, đúng lớp lỗi
    /// AD-26 ra đời để chặn.
    ///
    /// ⚠️ Ca **0 ký tự** đi cùng đường: vị từ độ dài là **một** mệnh đề
    /// `chars().count() < 3`, ⛔ không phải hai mệnh đề với một ca đặc biệt ở giữa, và một
    /// chuỗi rỗng **đúng là quá ngắn**.
    ///
    /// ⚠️ **Bất đối xứng có chủ ý với đường `zh`:** ở đó một truy vấn rỗng trả
    /// [`QueryBranch::CharIdx`] với `hits` rỗng (hành vi Story 1.11). ⛔ **Đừng "đồng bộ"
    /// hai bên** — hai bảng nhánh khác nhau vì hai chỉ mục khác nhau.
    NoBranchQueryTooShort,
}

/// Một đầu mục khớp. **Một hàng của `dict_entry`, ⛔ không phải một nghĩa.**
///
/// ⛔ Không `dict_sense`, ⛔ không `dict_example`, ⛔ không `dict_citation` ở đây — đọc
/// nghĩa là **Story 1.13** (FR29–FR32), và hình dạng của nó phụ thuộc vào quyết định
/// nhóm-theo-nguồn mà story này ⛔ không được phép đoán trước.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EntryHit {
    /// `dict_entry.id` — chỉ duy nhất **trong một tệp `.db`**.
    pub entry_id: i64,

    /// 🔴 `dict_source.code` — chuỗi, ⛔ **không** `source_id: i64`.
    ///
    /// Mỗi tệp `.db` mang bảng `dict_source` **RIÊNG**, nên `id = 1` tồn tại ở **cả ba**
    /// tệp và trỏ ba nguồn khác nhau (`viwiktionary` · `thieu-chuu` · `vietphrase`).
    /// Khoá theo `id` sẽ dán nhãn *"Thiều Chửu"* cho một nghĩa thật ra từ CVDICT ngay khi
    /// Story 1.13 gom nhiều tệp — FR31 vỡ theo cách thầm lặng nhất có thể, và nó vỡ ở
    /// **story sau** chứ không ở story này, tức đắt gấp đôi để lần ra.
    pub source_code: String,

    /// 🔴 `dict_entry.lang` — **một TRƯỜNG, ⛔ không phải một KIỂU** (AD-44 ⑤).
    ///
    /// Story 1.11 viết ở đây rằng *"một hằng ngầm ở chỗ gọi là thứ 1.11b sẽ phải gỡ"*.
    /// Story 1.11b đã gỡ nó: giá trị nay là `"zh"` **hoặc** `"en"` tuỳ
    /// [`QueryRoute`] của lượt tra, và ⛔ **không tồn tại** một bản ghi kết quả thứ hai
    /// dành riêng cho tiếng Anh. Một `EnEntryHit` song song sẽ buộc **mọi** chỗ gọi phải
    /// phân nhánh theo kiểu, và bước hợp nhất hai nhánh đó lại chính là thứ AD-19 cấm.
    pub lang: String,

    /// Đầu mục như nguồn ghi (phồn thể với đa số nguồn).
    pub headword: String,

    /// Dạng giản thể, `None` khi nguồn ⛔ không phân biệt phồn/giản.
    pub headword_simp: Option<String>,
}

/// Kết quả một lượt tra: **đường đã đi** cộng các hàng khớp.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LookupResult {
    /// Nhánh đã chạy — xem [`QueryBranch`].
    pub branch: QueryBranch,
    /// Các đầu mục khớp, thứ tự của `dict_entry.id` tăng dần.
    pub hits: Vec<EntryHit>,
}

/// Chọn nhánh cho một truy vấn. **Hàm thuần, `pub`, ⛔ không chạm database.**
///
/// 🔴 Tách thành một hàm riêng là điều kiện để AC1 nghiệm thu được mà ⛔ **không cần một
/// tệp `.db` nào** — tức phép kiểm đắt nhất của story này chạy được trong CI, nơi ⛔
/// không có tệp từ điển nào (`.gitignore: *.db`).
///
/// 🔴 `route` là **tham số**, ⛔ không phải một lời gọi [`pick_route`] bên trong: vị từ
/// điều phối chạy **ĐÚNG MỘT LẦN cho mỗi lượt tra**, ở tầng gom (AD-44 ①, vá A1).
///
/// ⚠️ `chars().count()` là phép đo, ⛔ không phải `len()`. Xem doc-comment của module.
pub fn pick_branch(query: &str, mode: LookupMode, route: QueryRoute) -> QueryBranch {
    match mode {
        // Tra chính xác ⛔ không phụ thuộc độ dài **ở cả hai đường**: một đầu mục một ký
        // tự và một đầu mục mười ký tự đều nằm trên cùng chỉ mục B-tree.
        LookupMode::Exact => QueryBranch::ExactBtree,

        // 🔴 `chars().count()` — ⛔ KHÔNG `len()`. Đây là dòng đắt nhất của cả story;
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
/// 🔴 Nhận [`ReadHandle`], ⛔ **không** nhận [`crate::core::store::ReadOnlyDb`]: đây là
/// một **hàm thuần theo kết nối**, và chỗ gọi là bên mở kho. Ba hệ quả, cả ba đều là điều
/// kiện của một story sau:
///
/// 1. Story 1.13 gọi hàm này **một lần cho mỗi tệp** rồi gom — với một chữ ký nhận
///    `ReadOnlyDb`, nó phải mở/đóng hoặc mượn lồng nhau.
/// 2. Test dựng fixture rồi gọi thẳng, ⛔ không phải dựng cả một `ReadOnlyDb` cho một ca
///    thuần logic.
/// 3. Cùng khuôn `bootstrap_config(store: Option<&Store>)` của Story 1.8: **hàm thuần là
///    đường sản phẩm**, vỏ là thứ bỏ đi được trong test.
///
/// 🔴 `route` **nhận từ chỗ gọi**, ⛔ **không** tính lại ở đây (AD-44 ①, vá A1). Hàm này
/// và [`query`] ⛔ **không bao giờ** gọi [`pick_route`] — một adapter ⛔ không tự phân xử
/// lại một câu hỏi thuộc về **cả lượt tra**. Ba lý do, cả ba cưỡng chế được:
///
/// 1. Story 1.13 gọi hàm này **một lần cho mỗi tệp `.db`** và phải truyền **cùng một**
///    `route` xuống mọi tệp — để mỗi tệp tự tính là để hai tệp trả lời khác nhau ngay khi
///    định nghĩa [`is_han`] của chúng lệch nhau.
/// 2. AD-44 ① nói thẳng vị từ chạy **TRÊN** adapter.
/// 3. Test **ép được** tổ hợp `(truy vấn Hán, route = En)` mà [`pick_route`] ⛔ không bao
///    giờ sinh ra — và đó là cách bộ lọc `lang` của đường tiếng Anh trở thành thứ
///    **nghiệm thu được** thay vì thứ *"chắc là đúng vì đầu vào không bao giờ tới đó"*.
///
/// Mọi nhánh lọc `lang` **tường minh trong SQL** — `'zh'` trên đường `Zh`, `'en'` trên
/// đường `En`. Xem [`query`] về vì sao vế đó ⛔ không bỏ được. ⛔ **Không** tồn tại một sổ
/// đăng ký *"tệp `.db` nào chứa ngôn ngữ nào"* (AD-44 ①, vá A2): **mọi** tệp đang gắn đều
/// được tra, và `lang` lọc trong SQL.
pub fn lookup(
    db: ReadHandle<'_>,
    query: &str,
    mode: LookupMode,
    route: QueryRoute,
) -> SqlResult<LookupResult> {
    let branch = pick_branch(query, mode, route);

    let hits = match branch {
        QueryBranch::ExactBtree => match route {
            QueryRoute::Zh => query::exact(db, query)?,
            QueryRoute::En => query::exact_en(db, query)?,
        },

        // Nhánh 2 là **của riêng đường `zh`** — [`pick_branch`] ⛔ không bao giờ chọn nó
        // cho đường `En`. Câu SQL bên trong lọc `lang = 'zh'`, nên tổ hợp đó (nếu ai đó
        // dựng ra bằng tay) trả rỗng chứ ⛔ không trả nhầm hàng tiếng Anh.
        QueryBranch::CharIdx => query::char_idx(db, query)?,

        QueryBranch::FtsTrigram => match route {
            QueryRoute::Zh => query::fts_trigram(db, query)?,
            QueryRoute::En => query::fts_trigram_en(db, query)?,
        },

        // 🔴 ⛔ **Không một câu SQL nào được chuẩn bị** — đó là mệnh đề của AD-44 ④, ⛔
        // không phải một phép tối ưu. Trạng thái *"không hỗ trợ"* phải **phân biệt được**
        // với một lượt tra đã chạy mà ⛔ không tìm thấy gì, và `branch` là chỗ nó khai ra.
        QueryBranch::NoBranchQueryTooShort => Vec::new(),
    };

    Ok(LookupResult { branch, hits })
}
