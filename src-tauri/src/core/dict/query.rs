//! Ba nhánh SQL tiếng Trung (AD-26) và hai nhánh SQL tiếng Anh (AD-44), cộng phép
//! **xác minh chuỗi con ở Rust** — **một** bản, dùng chung cho cả hai đường.
//!
//! ⛔ **Tệp này ⛔ không bao giờ gọi vị từ điều phối.** Đường đã được quyết ở tầng trên và
//! đi xuống đây như một tham số (AD-44 ①, vá A1); `tests/dict_boundary.rs` cưỡng chế điều
//! đó **bằng máy**, và cổng đó đếm **tệp** — nên tên vị từ ⛔ không được nhắc ở đây, kể cả
//! trong một dòng chú thích. Một cổng có ngoại lệ *"trừ khi là comment"* là một cổng chờ
//! ngoại lệ thứ hai.
//!
//! ⛔ **Và ⛔ không tồn tại một sổ đăng ký *"tệp `.db` nào chứa ngôn ngữ nào"*** (vá A2).
//! Một sổ như thế là **nguồn sự thật thứ hai cho một dữ kiện đã nằm trong dữ liệu** —
//! cùng lớp lỗi AD-8 và AD-33 tồn tại để chặn — và nó sai **im lặng** vào đúng ngày một
//! lớp gỡ rời được thêm hay gỡ đi (FR112). **Mọi** tệp đang gắn đều được tra; `lang` lọc
//! **trong SQL**.
//!
//! ─────────────────────────────────────────────────────────────────────────────
//! 🔴 BA LUẬT CỦA TỆP NÀY — CẢ BA ĐỀU HỎNG THÀNH MỘT LƯỢT CI XANH
//! ─────────────────────────────────────────────────────────────────────────────
//! 1. **⛔ Không `LIKE`, ⛔ không `GLOB`, ⛔ không `instr(`.** Giai đoạn 0 đo `LIKE` 1 ký
//!    tự **20,09 ms** và 2 ký tự **50,14 ms**, so với `char_idx` **0,15 / 4,49 ms** —
//!    nhanh hơn 134× và 11×. `LIKE` nằm trong danh sách *"Không dùng, đã loại có lý do"*
//!    của bảng Stack, và `tests/dict_boundary.rs` cưỡng chế điều đó **bằng máy**.
//!    ⚠️ `instr(` **được phép** trong SQL nghiệm thu chạy tay ở `sqlite3` (nó là cách
//!    tái lập con số 350 của AC4); nó ⛔ không được phép ở đây, vì cùng lý do quét bảng.
//! 2. **Mọi nhánh lọc `lang` TƯỜNG MINH — `'zh'` ở đường zh, `'en'` ở đường en.**
//!    Mệnh đề đảo chiều theo đường, nhưng luật thì **một**: ⛔ không nhánh nào được giả
//!    định *"tệp này chỉ có một ngôn ngữ"*. `dict-core.db` nay mang **119.039** hàng
//!    `lang = 'en'` (20,1% của 592.538 đầu mục — Story 1.10b). Với truy vấn **thuần Hán**
//!    rò rỉ đo được là 0, nhưng với một truy vấn **Latin** — người dùng bôi đen một chữ
//!    Latin trong văn bản tiếng Trung, chuyện thường — rò rỉ là thật và lớn: đã đo
//!    `entry_fts MATCH '"dic"'` ⇒ **572** hàng, **100%** `lang='en'`. Không lọc thì chúng
//!    đi lên giao diện **dán nhãn kết quả tiếng Trung**.
//! 3. **Chuỗi con phải được XÁC MINH LẠI.** `char_idx` là bảng `(ký tự, entry_id)`; nó
//!    trả lời *"đầu mục có chứa cả hai ký tự"*, ⛔ **không** trả lời *"có chứa hai ký tự
//!    ĐÓ LIỀN NHAU"*. Đã đo: tra `中國` cho **390** ứng viên, trong đó **40** là dương
//!    tính giả (`國中`, …). Người dùng tra *"Trung Quốc"* nhận về *"trong trường"* — kết
//!    quả **khác rỗng**, **sai**, và ⛔ không phép kiểm nào phát biểu bằng `> 0` bắt được.
//!
//! ⚠️ Mọi truy vấn dùng **tham số ràng buộc**, ⛔ không bao giờ `format!` chuỗi của người
//! dùng vào câu SQL.

use crate::core::store::{ReadHandle, Row, SqlResult, ToSql};

use super::EntryHit;

/// Bốn cột dựng nên một [`EntryHit`], cộng `s.code` của AC6.
///
/// 🔴 `s.code` chứ ⛔ **không** `e.source_id`: mỗi tệp `.db` có bảng `dict_source` riêng,
/// nên `id = 1` trỏ ba nguồn khác nhau ở ba tệp. Xem [`EntryHit::source_code`].
const COLUMNS: &str = "e.id, s.code, e.lang, e.headword, e.headword_simp";

/// `JOIN` lấy `code` — bắt buộc ở **cả ba** nhánh, cùng lý do với [`COLUMNS`].
const JOIN_SOURCE: &str = "JOIN dict_source s ON s.id = e.source_id";

fn row_to_hit(row: &Row<'_>) -> SqlResult<EntryHit> {
    Ok(EntryHit {
        entry_id: row.get(0)?,
        source_code: row.get(1)?,
        lang: row.get(2)?,
        headword: row.get(3)?,
        headword_simp: row.get(4)?,
    })
}

/// Chạy một truy vấn có tham số ràng buộc và dựng danh sách [`EntryHit`].
///
/// ⚠️ `prepare_cached` chứ ⛔ không `prepare`: sáu hình dạng SQL hằng (ba nhánh đường zh,
/// hai nhánh đường en, cộng biến thể 1/2-ký-tự của `char_idx`), và một lượt tra cứu là
/// đường nóng của NFR1 — biên dịch lại câu ở mỗi lượt gõ là chi phí trả đi trả lại cho
/// cùng một thứ.
fn run(db: ReadHandle<'_>, sql: &str, params: &[&dyn ToSql]) -> SqlResult<Vec<EntryHit>> {
    let mut stmt = db.prepare_cached(sql)?;
    let rows = stmt.query_map(params, row_to_hit)?;
    rows.collect()
}

/// 🔴 **Quyết định #4 (Story 1.17)** — cắt `hits` còn tối đa `limit` phần tử, trả kèm cờ
/// **`truncated`**: `true` ⇔ còn hàng bị cắt bỏ.
///
/// ⚠️ Chỗ gọi phải truyền vào một `hits` đã **XÁC MINH ĐẦY ĐỦ** ([`verify_substring`] đã
/// chạy nếu nhánh cần nó) — cắt TRƯỚC khi xác minh là Bẫy 11: trang hiện ra ít hơn `limit`
/// mục thật và một dòng "còn M nữa" nói dối, vì `verify_substring` loại thêm hàng sau khi
/// đã cắt.
fn cap(mut hits: Vec<EntryHit>, limit: usize) -> (Vec<EntryHit>, bool) {
    let limit = effective_limit(limit);
    let truncated = hits.len() > limit;
    hits.truncate(limit);
    (hits, truncated)
}

/// 🔴 **Sàn DƯỚI của cỡ trang — `limit == 0` ⛔ không được đọc thành "⛔ trả gì cả".**
///
/// [`lookup`](super::lookup) và bạn bè là `pub`, nên `0` đi vào được. Một `LIMIT 0` cho
/// `groups` **rỗng** kèm `truncated = true`, và panel khi đó hiện ĐỒNG THỜI *"⛔ tìm thấy"*
/// và *"danh sách ⛔ đầy đủ"* — hai câu loại trừ nhau, ⛔ câu nào đúng. Một cỡ trang `0`
/// ⛔ phải một yêu cầu hợp lệ mà là một lỗi chỗ gọi; hành vi ít gây hại nhất là coi nó như
/// `1` và để cờ `truncated` nói phần còn lại.
fn effective_limit(limit: usize) -> usize {
    limit.max(1)
}

/// 🔴 **Trần hàng để ĐẶT VÀO SQL — ⛔ `limit as i64`.**
///
/// `usize::MAX` là thành ngữ tự nhiên nhất cho *"⛔ giới hạn"*, và `usize::MAX as i64` là
/// **-1**: `saturating_add(1)` biến nó thành `LIMIT 0` ⇒ **0 hàng, `truncated = false`** —
/// mất sạch dữ liệu, im lặng, ở một hàm `pub`. `saturating_add` ⛔ cứu được gì vì phép tràn
/// đã xảy ra ở bước ép kiểu TRƯỚC nó. `try_from` + `unwrap_or(i64::MAX)` bão hoà đúng
/// chiều: một trần lớn hơn số hàng khả dĩ đọc ra là *"lấy hết"*, ⛔ *"⛔ lấy gì"*.
///
/// Lấy `limit + 1` hàng để [`cap`] phân biệt được *"vừa đủ `limit`"* với *"còn nữa"*.
fn fetch_rows(limit: usize) -> i64 {
    i64::try_from(effective_limit(limit)).unwrap_or(i64::MAX).saturating_add(1)
}

/// 🔴 Phép **xác minh chuỗi con**, chạy ở Rust — dùng chung cho nhánh 2 và nhánh 3.
///
/// Giữ một hàng khi `query` là chuỗi con của `headword` **hoặc** của `headword_simp`,
/// so khớp **không phân biệt hoa/thường** (hạ chữ thường ở RUST, cùng lý do và cùng cách
/// với [`exact_en`]: `str::to_lowercase()`, ⛔ không phụ thuộc locale).
///
/// 🔴 **Bắt buộc không phân biệt hoa/thường** — tokenizer `trigram` của FTS5 **không**
/// phân biệt hoa/thường khi tìm ứng viên (đo thật: `entry_fts MATCH '"api"'` khớp hàng
/// `headword = 'API'`). Xác minh phân biệt hoa/thường ở đây sẽ **âm thầm loại** đúng ứng
/// viên mà FTS5 vừa tìm ra — rỗng, ⛔ không lỗi, đúng lớp lỗi AD-26 ra đời để chặn. Vô hại
/// với đường `zh`: chữ Hán ⛔ không có khái niệm hoa/thường.
///
/// ⚠️ Vế `headword_simp` ⛔ **không bỏ được**: bỏ nó làm `国` (giản thể) trả rỗng, đúng
/// Bẫy 8 của Story 1.9 — `char_idx` phủ **cả** hai trường, nên một ứng viên có thể khớp
/// **chỉ** ở `headword_simp`, và loại nó ở bước xác minh là vứt đi đúng những hàng mà
/// bước dựng chỉ mục đã cố công giữ.
///
/// ⛔ Một hàm, ⛔ không hai bản: nhánh 3 chạy **cùng** phép xác minh này. Đo được là
/// `中國人` ⇒ 33 ứng viên → 33 sau xác minh, **0** dương tính giả — và con số 0 đó là một
/// **phép đo**, ⛔ không phải cái cớ để bỏ bước. Bỏ nó là để một hành vi không được kiểm
/// chứng của FTS5 quyết định đúng/sai của FR39.
fn verify_substring(hits: Vec<EntryHit>, query: &str) -> Vec<EntryHit> {
    let needle = query.to_lowercase();
    hits.into_iter()
        .filter(|hit| {
            hit.headword.to_lowercase().contains(&needle)
                || hit
                    .headword_simp
                    .as_deref()
                    .is_some_and(|simp| simp.to_lowercase().contains(&needle))
        })
        .collect()
}

/// **Nhánh 1** — tra chính xác đầu mục qua B-tree.
///
/// `idx_entry_headword` và `idx_entry_headword_simp` đều tồn tại (`schema.rs:110-111`),
/// nên vế `OR` đi qua kế hoạch `MULTI-INDEX OR` của SQLite. `EXPLAIN QUERY PLAN` nguyên
/// văn nằm ở §Debug Log References của story — thấy `SCAN dict_entry` thì câu này phải
/// tách thành hai truy vấn `UNION`.
///
/// 🔴 **`LIMIT ?2` — Quyết định #4 (Story 1.17), tham số RÀNG BUỘC.** Nhánh này ⛔ không
/// có bước xác minh (⛔ không [`verify_substring`]) nên `LIMIT` ở SQL an toàn — đo được
/// kế hoạch vẫn `MULTI-INDEX OR` + `USE TEMP B-TREE FOR ORDER BY` cho ca này, nhưng nhánh
/// 1 luôn rất nhanh (< 1 ms mọi ca đo) nên `LIMIT` ở đây chủ yếu cắt băng thông IPC.
pub(super) fn exact(db: ReadHandle<'_>, query: &str, limit: usize) -> SqlResult<(Vec<EntryHit>, bool)> {
    let sql = format!(
        "SELECT {COLUMNS} FROM dict_entry e {JOIN_SOURCE} \
         WHERE (e.headword = ?1 OR e.headword_simp = ?1) AND e.lang = 'zh' \
         ORDER BY e.id LIMIT ?2"
    );
    let fetch = fetch_rows(limit);
    let hits = run(db, &sql, &[&query, &fetch])?;
    Ok(cap(hits, limit))
}

/// **Nhánh 2** — bảng đảo ngược `char_idx`, cho chuỗi con **1–2 ký tự**.
///
/// Hai đường tách nhau theo số **ký tự**, ⛔ không theo byte:
///
/// - **1 ký tự** ⇒ một tập `char_idx`, và ⛔ **không xác minh**: một ký tự có mặt trong
///   `char_idx` của một đầu mục **⇔** nó là chuỗi con của đầu mục đó. Mệnh đề này viết ra
///   thay vì để ngầm, vì bước xác minh ở đây sẽ là một vòng lặp trên 3.177 hàng ⛔ không
///   loại được hàng nào.
/// - **2 ký tự** ⇒ 🔴 `INTERSECT` **hai** tập, rồi **xác minh**.
///
/// 🔴 ⛔ **Không viết `ch IN (?1, ?2)`** — đó là phép **hợp**, ⛔ không phải phép **giao**:
/// nó trả mọi đầu mục chứa `中` *hoặc* `國` (hàng chục nghìn), và cả hai cách viết đều cho
/// kết quả *"khác rỗng"*, nên mọi AC phát biểu bằng `> 0` đều xanh trên bản sai. Con số
/// duy nhất bắt được sai lệch là **390** ứng viên của AC4.
///
/// ⚠️ **Giới hạn đã biết, ⛔ không phải một lỗi:** một truy vấn 2 ký tự mà **một ký tự
/// không phải chữ Hán** (vd. `A山`) cho tập ứng viên **rỗng** — `char_idx` chỉ chứa ký tự
/// khớp `char_idx::is_han` của `tools/dict-build`. Đó là hành vi **đúng** cho một đường
/// tra cứu **tiếng Trung**; đường tra cứu tiếng Anh là [`exact_en`] và [`fts_trigram_en`]
/// (Story 1.11b), ⛔ **không** một nhánh thứ tư ở đây. Story 1.11 viết dòng này khi hai
/// hàm đó chưa tồn tại; chúng ⛔ vẫn không tồn tại **trong nhánh này**, và đó là điểm.
pub(super) fn char_idx(db: ReadHandle<'_>, query: &str, limit: usize) -> SqlResult<(Vec<EntryHit>, bool)> {
    debug_assert!(
        query.chars().count() <= 2,
        "char_idx() expects a query of at most 2 characters (pick_branch() must filter \
         first); calling it directly with a longer query silently truncates to the first \
         two characters"
    );

    let mut chars = query.chars();
    let Some(first) = chars.next() else {
        // Truy vấn rỗng: ⛔ không hàng nào, và ⛔ không một lượt chạm database nào. Một
        // `SELECT` với tham số rỗng ở đây trả về đúng thứ này sau khi quét, chỉ chậm hơn.
        return Ok((Vec::new(), false));
    };

    let Some(second) = chars.next() else {
        // 🔴 **1 ký tự — ⛔ không bước xác minh** (xem doc-comment hàm này ở trên: một ký
        // tự có mặt trong `char_idx` ⇔ nó là chuỗi con). `LIMIT ?2` ở SQL AN TOÀN và CẮT
        // ĐƯỢC THỜI GIAN THẬT — đo (§Debug Log References của story): `char_idx` khai
        // `PRIMARY KEY (ch, entry_id) WITHOUT ROWID`, nên `EXPLAIN QUERY PLAN` cho
        // `LIST SUBQUERY` (driven bởi `SEARCH char_idx USING PRIMARY KEY`, đã sắp theo
        // `entry_id` tăng dần) rồi `SEARCH e USING INTEGER PRIMARY KEY (rowid=?)` — streaming,
        // ⛔ không `USE TEMP B-TREE FOR ORDER BY`. Đo tay: 9–12 ms (⛔ `LIMIT`) → ~1 ms
        // (`LIMIT 20`), ~10×. Đây là nhánh ĐẮT NHẤT của cả sáu — vượt trần NFR1 (`:419`).
        let sql = format!(
            "SELECT {COLUMNS} FROM dict_entry e {JOIN_SOURCE} \
             WHERE e.id IN (SELECT entry_id FROM char_idx WHERE ch = ?1) \
               AND e.lang = 'zh' \
             ORDER BY e.id LIMIT ?2"
        );
        let fetch = fetch_rows(limit);
        let hits = run(db, &sql, &[&first.to_string(), &fetch])?;
        return Ok(cap(hits, limit));
    };

    // 🔴 **2 ký tự — Bẫy 11.** `verify_substring` PHẢI chạy trên TOÀN BỘ ứng viên trước khi
    // cắt: một `LIMIT` ở SQL cắt ứng viên trước khi xác minh cho ra < `limit` mục thật và
    // một dòng "còn M nữa" NÓI DỐI (`verify_substring` loại thêm dương tính giả sau khi đã
    // cắt). ⇒ ⛔ không `LIMIT` ở SQL cho nhánh này — cắt ở RUST, SAU verify. Đo (§Debug Log):
    // kế hoạch của nhánh này (`INTERSECT USING TEMP B-TREE`) ⛔ không `USE TEMP B-TREE FOR
    // ORDER BY` ở outer query nên vốn đã streaming; và nhánh 2-ký-tự vốn dưới trần NFR1
    // (3,451 ms p95, `:419`) nên không cần SQL `LIMIT` để đạt NFR1.
    let sql = format!(
        "SELECT {COLUMNS} FROM dict_entry e {JOIN_SOURCE} \
         WHERE e.id IN ( \
             SELECT entry_id FROM char_idx WHERE ch = ?1 \
             INTERSECT \
             SELECT entry_id FROM char_idx WHERE ch = ?2 \
           ) \
           AND e.lang = 'zh' \
         ORDER BY e.id"
    );
    let candidates = run(db, &sql, &[&first.to_string(), &second.to_string()])?;
    let verified = verify_substring(candidates, query);
    Ok(cap(verified, limit))
}

/// **Nhánh 3** — FTS5 `entry_fts` (`trigram`), cho chuỗi con **≥ 3 ký tự**.
///
/// `entry_fts` là bảng FTS5 **external-content** trên `dict_entry` với
/// `content_rowid = 'id'` (`schema.rs:119-121`), nên phép nối là `f.rowid = e.id`.
///
/// 🔴 **Truy vấn đi vào dạng cụm có ngoặc kép, và ngoặc kép là bắt buộc.** Không bọc,
/// chuỗi của người dùng đi thẳng vào **cú pháp truy vấn FTS5**, và một ký tự như `*` `-`
/// `^` `(` `:` — hay từ `NEAR` — làm SQLite trả `SQLITE_ERROR`. Nghĩa là **tra cứu báo
/// lỗi vì nội dung người dùng bôi đen**: tệ hơn hẳn trả rỗng, và nó chỉ lộ ra ở tay người
/// dùng thật chứ ⛔ không ở CI, nơi fixture chỉ có chữ Hán sạch.
///
/// 🔴 Dấu `"` bên trong truy vấn được **nhân đôi** trước khi bọc — đó là cách thoát của
/// FTS5. ⛔ Không bỏ qua, ⛔ không xoá ký tự: xoá là im lặng trả về kết quả của một truy
/// vấn khác truy vấn người dùng gõ.
pub(super) fn fts_trigram(db: ReadHandle<'_>, query: &str, limit: usize) -> SqlResult<(Vec<EntryHit>, bool)> {
    let phrase = format!("\"{}\"", query.replace('"', "\"\""));

    // 🔴 Cùng Bẫy 11 của `char_idx` 2 ký tự: `verify_substring` phải chạy trên TOÀN BỘ ứng
    // viên trước khi cắt. ⛔ Không `LIMIT` ở SQL. Đo (`EXPLAIN QUERY PLAN`): nhánh này CÓ
    // `USE TEMP B-TREE FOR ORDER BY` — một `LIMIT` ở SQL ⛔ sẽ không cắt được thời gian dù
    // đặt trước hay sau verify — nhưng nhánh 3 vốn dưới trần NFR1 (0,6–2,0 ms mọi ca đo,
    // §Debug Log), nên vô hại: `LIMIT` chỉ mua băng thông IPC ở nhánh này, cắt ở Rust đủ.
    let sql = format!(
        "SELECT {COLUMNS} FROM entry_fts f \
         JOIN dict_entry e ON e.id = f.rowid {JOIN_SOURCE} \
         WHERE entry_fts MATCH ?1 AND e.lang = 'zh' \
         ORDER BY e.id"
    );
    let candidates = run(db, &sql, &[&phrase])?;
    let verified = verify_substring(candidates, query);
    Ok(cap(verified, limit))
}

/// **Nhánh tra chính xác của đường tiếng Anh** — tập khoá `{nguyên văn, hạ chữ thường}`
/// trong **MỘT** truy vấn (AD-44 ③).
///
/// 🔴 **`IN (?1, ?2)`, ⛔ không fallback dây chuyền.** Tra nguyên văn rồi *"rỗng thì tra
/// lại dạng hạ chữ thường"* làm mỗi lượt tra chạy **hai** truy vấn ⇒ số đo NFR1 mất
/// nghĩa, và làm [`super::LookupResult::branch`] **nói dối** về đường đã đi. Một lượt qua
/// B-tree, một truy vấn.
///
/// ⚠️ `IN (?1, ?2)` chứ ⛔ không `?1 OR ?2`: hai cách viết ⛔ **không tương đương về kế
/// hoạch** trên mọi phiên bản SQLite, và AD-44 ③ khai đích danh hình dạng đầu. ⛔ Cũng
/// không `UNION ALL` — nó **sinh trùng** khi hai khoá cùng khớp một hàng, còn `IN` trả
/// mỗi hàng đúng một lần.
///
/// 🔴 **Lỗ mà cả nhánh này tồn tại để bịt:** đo thật trên `dict-core.db`,
/// `headword = 'running'` ⇒ **1** hàng, `headword = 'Running'` ⇒ **0**. Bôi đen một từ ở
/// **đầu câu** là thao tác thường ngày; không có khoá thứ hai, nó trả rỗng ⛔ không lỗi.
///
/// 🔴 **Hạ chữ thường tính ở RUST, ⛔ không bằng `lower()` của SQLite.** Hàm dựng sẵn của
/// SQLite chỉ hạ **ASCII** — nó ⛔ không chạm `É`, `Ü`, `Ø`, và một đầu mục tiếng Anh mượn
/// từ nước ngoài sẽ rơi **im lặng**. [`str::to_lowercase`] của Rust ⛔ cũng **không phụ
/// thuộc locale**: `"I".to_lowercase()` luôn ra `"i"`. Một phép fold theo locale làm
/// **cùng một truy vấn cho hai kết quả trên hai máy** cài ngôn ngữ hệ điều hành khác nhau
/// — một hồi quy ⛔ không tái lập được trên máy người sửa (vá A4 của Reviewer Gate).
///
/// ⚠️ **Bất đối xứng CÓ CHỦ Ý, ⛔ không phải một chỗ bỏ sót:** hạ chữ thường là một khoá
/// **THÊM** phía **truy vấn**, ⛔ **không** phải một phép hạ phía **đầu mục**. `Running`
/// ⇒ `running` (có); `api` ⇒ `API` (⛔ **không**). **1.635** đầu mục tiếng Anh mang chữ
/// hoa **có nghĩa** (`API`, `Wikipedia`, `English`), nên khoá gốc phải được giữ. Khớp hai
/// chiều đòi một **chỉ mục hàm `lower(headword)` lúc build** ⇒ đổi `schema.rs`, dựng lại
/// `dict-core.db`, điền lại `[base].sha256`, đo lại NFR6, và làm **184** nhóm đầu mục
/// *(chỉ phân biệt nhau bằng chữ hoa)* sập vào nhau. Đó là **tầng PRD/kiến trúc**.
///
/// ⚠️ ⛔ **Không** vế `headword_simp`: nó **luôn `NULL`** trên toàn bộ 119.039 mục tiếng
/// Anh, nên một vế `OR e.headword_simp = ?` chỉ thêm một nhánh kế hoạch ⛔ không bao giờ
/// khớp.
pub(super) fn exact_en(db: ReadHandle<'_>, query: &str, limit: usize) -> SqlResult<(Vec<EntryHit>, bool)> {
    let lowered = query.to_lowercase();

    // ⛔ Không bước xác minh (cùng lý do `exact`) ⇒ `LIMIT ?3` ở SQL an toàn.
    let sql = format!(
        "SELECT {COLUMNS} FROM dict_entry e {JOIN_SOURCE} \
         WHERE e.headword IN (?1, ?2) AND e.lang = 'en' \
         ORDER BY e.id LIMIT ?3"
    );
    let fetch = fetch_rows(limit);
    let hits = run(db, &sql, &[&query, &lowered, &fetch])?;
    Ok(cap(hits, limit))
}

/// **Nhánh chuỗi con của đường tiếng Anh** — FTS5 `entry_fts` (`trigram`), **≥ 3 ký tự**.
///
/// Khuôn **y hệt** [`fts_trigram`], đổi đúng một thứ: bộ lọc `lang`. `entry_fts` lập chỉ
/// mục trigram trên `headword` của **MỌI** hàng — cả `zh` lẫn `en` — nên vế `lang` là thứ
/// tách hai đường ra, ⛔ không phải một giả định về tệp.
///
/// ⚠️ **Rủi ro cú pháp FTS5 CAO HƠN NHIỀU ở đây so với đường tiếng Trung:** một truy vấn
/// Latin dễ chứa `'`, `-`, `*`, `:` (`don't`, `state-of-the-art`). Vì thế phép bọc cụm
/// **dùng lại nguyên** cách của nhánh 3 — bọc ngoặc kép + nhân đôi `"` — ⛔ không có bản
/// thứ hai và ⛔ không có một "biến thể cho tiếng Anh".
///
/// 🔴 **Vẫn đi qua [`verify_substring`], và vế `headword_simp` bên trong nó sẽ luôn
/// `None` với tiếng Anh — ⛔ ĐỪNG bỏ hàm vì thế.** Nó là hàng rào chống **dương tính giả**
/// của trigram, và hàng rào đó ⛔ không phụ thuộc ngôn ngữ: FTS5 trả lời *"chứa các
/// trigram này"*, ⛔ không trả lời *"chứa chuỗi này"*.
pub(super) fn fts_trigram_en(db: ReadHandle<'_>, query: &str, limit: usize) -> SqlResult<(Vec<EntryHit>, bool)> {
    let phrase = format!("\"{}\"", query.replace('"', "\"\""));

    // Cùng lý do `fts_trigram`: ⛔ không `LIMIT` ở SQL, cắt ở Rust sau verify.
    let sql = format!(
        "SELECT {COLUMNS} FROM entry_fts f \
         JOIN dict_entry e ON e.id = f.rowid {JOIN_SOURCE} \
         WHERE entry_fts MATCH ?1 AND e.lang = 'en' \
         ORDER BY e.id"
    );
    let candidates = run(db, &sql, &[&phrase])?;
    let verified = verify_substring(candidates, query);
    Ok(cap(verified, limit))
}

// ═════════════════════════════════════════════════════════════════════════════════
// Story 1.17 — ĐẾM ĐẦY ĐỦ THEO NGUỒN (Quyết định #4, §hệ quả ③ đường (a))
// ═════════════════════════════════════════════════════════════════════════════════

/// Gom `hits` thành `(source_code, số đầu mục)`, sắp theo `code` — cùng thứ tự tất định
/// mà tầng gom dùng cho `groups`, nên hai danh sách zip được với nhau ở chỗ gọi.
fn tally(hits: &[EntryHit]) -> Vec<(String, i64)> {
    let mut counts: Vec<(String, i64)> = Vec::new();
    for hit in hits {
        match counts.iter_mut().find(|(code, _)| *code == hit.source_code) {
            Some((_, n)) => *n += 1,
            None => counts.push((hit.source_code.clone(), 1)),
        }
    }
    counts.sort_by(|a, b| a.0.cmp(&b.0));
    counts
}

/// Chạy một câu `COUNT(*) … GROUP BY s.code` và dựng danh sách đã sắp.
fn run_counts(db: ReadHandle<'_>, sql: &str, params: &[&dyn ToSql]) -> SqlResult<Vec<(String, i64)>> {
    let mut stmt = db.prepare_cached(sql)?;
    let rows = stmt.query_map(params, |row| Ok((row.get(0)?, row.get(1)?)))?;
    rows.collect()
}

/// 🔴 **Đếm ĐẦY ĐỦ theo nguồn — số mà thanh nhịp nói ra khi trần đã cắt** (AC12).
///
/// ⚠️ **Hai hình dạng, ⛔ không một.** Ba nhánh ⛔ cần xác minh (`exact` · `exact_en` ·
/// `char_idx` 1 ký tự) đếm bằng **SQL thuần** — rẻ, ⛔ chạm một hàng `dict_entry` nào.
/// Ba nhánh CÓ xác minh (`char_idx` 2 ký tự · cả hai `fts_trigram`) **⛔ đếm được bằng
/// SQL**: `COUNT(*)` ở đó đếm **ứng viên**, mà ứng viên chứa dương tính giả (đo thật:
/// `中國` ⇒ 390 ứng viên, **40** sai). Một `COUNT` trên ứng viên là một con số **to hơn sự
/// thật**, và thanh nhịp khi đó nói dối theo chiều ngược lại — đúng Bẫy 11, chỉ đổi dấu.
/// ⇒ chúng đi qua **cùng** đường lấy-rồi-xác-minh, rồi đếm ở Rust.
///
/// 🔴 Chỗ gọi chỉ được chạy hàm này khi `truncated == true` — xem
/// [`DictionarySource::count_by_source`](crate::ports::DictionarySource::count_by_source).
pub(super) fn count_by_source(
    db: ReadHandle<'_>,
    query: &str,
    route: super::QueryRoute,
    branch: super::QueryBranch,
) -> SqlResult<Vec<(String, i64)>> {
    use super::{QueryBranch, QueryRoute};

    match branch {
        QueryBranch::ExactBtree => match route {
            QueryRoute::Zh => {
                let sql = format!(
                    "SELECT s.code, COUNT(*) FROM dict_entry e {JOIN_SOURCE} \
                     WHERE (e.headword = ?1 OR e.headword_simp = ?1) AND e.lang = 'zh' \
                     GROUP BY s.code ORDER BY s.code"
                );
                run_counts(db, &sql, &[&query])
            }
            QueryRoute::En => {
                let lowered = query.to_lowercase();
                let sql = format!(
                    "SELECT s.code, COUNT(*) FROM dict_entry e {JOIN_SOURCE} \
                     WHERE e.headword IN (?1, ?2) AND e.lang = 'en' \
                     GROUP BY s.code ORDER BY s.code"
                );
                run_counts(db, &sql, &[&query, &lowered])
            }
        },

        QueryBranch::CharIdx => {
            let mut chars = query.chars();
            let Some(first) = chars.next() else {
                return Ok(Vec::new());
            };

            if chars.next().is_none() {
                // 1 ký tự — ⛔ bước xác minh (xem `char_idx`) ⇒ `COUNT` ở SQL ĐÚNG.
                let sql = format!(
                    "SELECT s.code, COUNT(*) FROM dict_entry e {JOIN_SOURCE} \
                     WHERE e.id IN (SELECT entry_id FROM char_idx WHERE ch = ?1) \
                       AND e.lang = 'zh' \
                     GROUP BY s.code ORDER BY s.code"
                );
                return run_counts(db, &sql, &[&first.to_string()]);
            }

            // 2 ký tự — PHẢI xác minh trước khi đếm (xem doc-comment hàm này).
            let (hits, _) = char_idx(db, query, usize::MAX)?;
            Ok(tally(&hits))
        }

        QueryBranch::FtsTrigram => {
            // Cả hai đường: ứng viên trigram PHẢI qua `verify_substring` trước khi đếm.
            let (hits, _) = match route {
                QueryRoute::Zh => fts_trigram(db, query, usize::MAX)?,
                QueryRoute::En => fts_trigram_en(db, query, usize::MAX)?,
            };
            Ok(tally(&hits))
        }

        // ⛔ Không câu SQL nào chạy ở nhánh này (AD-44 ④) ⇒ ⛔ có gì để đếm.
        QueryBranch::NoBranchQueryTooShort => Ok(Vec::new()),
    }
}
