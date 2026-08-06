//! **Pha hai** — đọc `dict_sense` · `dict_example` · `dict_citation` cho một tập đầu mục.
//!
//! ⛔ **Tệp này ⛔ không bao giờ gọi vị từ điều phối** — cùng luật với `query.rs`, và
//! `tests/dict_boundary.rs` cưỡng chế nó bằng máy, đếm **tệp**.
//!
//! ─────────────────────────────────────────────────────────────────────────────
//! 🔴 BA LUẬT CỦA TỆP NÀY — CẢ BA HỎNG THÀNH MỘT LƯỢT CI XANH
//! ─────────────────────────────────────────────────────────────────────────────
//! 1. **⛔ Không một truy vấn cho mỗi đầu mục (N+1).** Nhánh `char_idx` một ký tự trả
//!    **3.177** đầu mục trên `dict-core.db` thật; nhân ba tệp, nhân ba bảng, một truy vấn
//!    mỗi đầu mục là **ba bậc độ lớn** — và nó *"chạy đúng"* trên một fixture 20 hàng, nên
//!    ⛔ không ca hành vi nào đỏ. Một tập id đi vào **một** câu SQL theo lô.
//! 2. **Lô cỡ CỐ ĐỊNH, ⛔ không co giãn.** [`run`] của `query.rs` dùng `prepare_cached` có
//!    lý do — *"một lượt tra cứu là đường nóng của NFR1"*. Một câu SQL sinh theo **số phần
//!    tử thật** của lô là một **hình dạng SQL mới mỗi lần** ⇒ cache câu lệnh trống ⇒ SQLite
//!    biên dịch lại câu ở mỗi lượt gõ, trên đúng đường nóng mà cache tồn tại để bảo vệ.
//!    Lô cuối được **đệm bằng một id đã có trong chính lô đó** — `IN` là phép kiểm **tập
//!    hợp**, nên một id lặp lại ⛔ không sinh thêm một hàng nào.
//! 3. **`ORDER BY ord, id` — ⛔ không `ORDER BY ord` trần.**
//!    `tools/dict-build/src/sources/vietphrase.rs` tách `/` **vô điều kiện** và sinh nhiều
//!    `dict_sense` **cùng `ord`** (`deferred-work.md`, Story 1.10). Thiếu khoá phụ, hai
//!    lượt chạy cho hai thứ tự — tức một ca **flaky**, và một ca flaky **bị gỡ** chứ ⛔
//!    không được sửa. `tests/dict_boundary.rs` cưỡng chế luật này bằng máy.
//!
//! ⚠️ Mọi truy vấn dùng **tham số ràng buộc**. Chuỗi `?1, ?2, …` là một hằng dẫn xuất từ
//! [`SENSE_BATCH`], ⛔ không phải dữ liệu của người dùng ghép vào câu.
//!
//! ⚠️ Ba chỉ mục `idx_sense_entry` · `idx_example_sense` · `idx_citation_sense` **đã tồn
//! tại** trong lược đồ (`tools/dict-build/src/schema.rs:106-114`) và chúng tồn tại **chính
//! xác vì đường đọc này**. ⛔ Đừng thêm chỉ mục — chạm `schema.rs` là dựng lại `.db`, là
//! sai `sha256` trong `dict-manifest.toml`.

use std::collections::{HashMap, HashSet};
use std::sync::LazyLock;

use crate::core::store::{ReadHandle, Row, SqlResult, ToSql};

use super::{CitationRecord, ExampleRecord, SenseRecord};

/// Số đầu mục (hoặc số nghĩa) trong **một** lô.
///
/// 🔴 **Hằng, và cỡ cố định là điểm** — xem luật 2 ở doc-comment module. Con số **64** là
/// một đánh đổi giữa hai chi phí ⛔ không cùng đơn vị:
///
/// - **Nhỏ hơn** ⇒ nhiều vòng lặp hơn cho cùng một tập đầu mục *(3.177 đầu mục là 50 lô ở
///   cỡ 64, 199 lô ở cỡ 16)*, mỗi vòng một lượt `query_map` và một lượt duyệt hàng.
/// - **Lớn hơn** ⇒ mỗi lô mang nhiều tham số ràng buộc thừa hơn khi tập id ngắn — và tập
///   id ngắn là **ca thường gặp**, vì Panel Lookup (1.17) hiện một trang chứ ⛔ không hiện
///   3.177 đầu mục.
///
/// ⚠️ `SQLITE_MAX_VARIABLE_NUMBER` mặc định là **32.766** ở SQLite ≥ 3.32, nên 64 ⛔ không
/// ở gần một trần nào — con số này chọn theo hình dạng dữ liệu, ⛔ không theo giới hạn của
/// thư viện.
pub const SENSE_BATCH: usize = 64;

/// `?1, ?2, …, ?64` — dựng **một lần**, dùng lại cho mọi lô.
///
/// 🔴 Dựng một lần là điều kiện của luật 2: `prepare_cached` khoá theo **văn bản câu**, nên
/// hai chuỗi khác nhau là hai câu khác nhau trong cache.
static PLACEHOLDERS: LazyLock<String> = LazyLock::new(|| {
    (1..=SENSE_BATCH)
        .map(|i| format!("?{i}"))
        .collect::<Vec<_>>()
        .join(", ")
});

/// `dict_sense` — sáu trường của FR28 cộng `entry_id` để gắn ngược về đầu mục.
static SENSE_SQL: LazyLock<String> = LazyLock::new(|| {
    format!(
        "SELECT s.id, s.entry_id, s.pos, s.pos_lang, s.gloss, s.note, s.ord \
         FROM dict_sense s WHERE s.entry_id IN ({}) \
         ORDER BY s.entry_id, s.ord, s.id",
        *PLACEHOLDERS
    )
});

/// `dict_example` — treo vào **`sense_id`** (FR30), ⛔ không vào `entry_id`.
///
/// 🔴 Lược đồ đã cưỡng chế vế đó (`dict_example.sense_id REFERENCES dict_sense(id)`), và
/// đọc bằng một `JOIN` vòng qua `entry_id` là **tự đánh mất** nó: ví dụ sẽ treo vào cả đầu
/// mục thay vì vào đúng **từ loại** sinh ra nó.
static EXAMPLE_SQL: LazyLock<String> = LazyLock::new(|| {
    format!(
        "SELECT x.sense_id, x.text, x.translation, x.translation_lang, x.ord \
         FROM dict_example x WHERE x.sense_id IN ({}) \
         ORDER BY x.sense_id, x.ord, x.id",
        *PLACEHOLDERS
    )
});

/// `dict_citation` — **bảng RIÊNG** với ví dụ, vì nó mang **xuất xứ** (`work`, `author`).
///
/// ⛔ Trộn hai bảng vào một danh sách là làm mất đúng thứ FR30 phân biệt: một *ví dụ* do
/// người biên soạn đặt ra, một *trích dẫn* đến từ một tác phẩm có tên và có tác giả.
static CITATION_SQL: LazyLock<String> = LazyLock::new(|| {
    format!(
        "SELECT c.sense_id, c.text, c.work, c.author, c.ord \
         FROM dict_citation c WHERE c.sense_id IN ({}) \
         ORDER BY c.sense_id, c.ord, c.id",
        *PLACEHOLDERS
    )
});

/// Một lô **đủ [`SENSE_BATCH`] phần tử**: phần thiếu đệm bằng phần tử đầu của lô.
///
/// ⚠️ Đệm bằng một id **đã có trong lô** chứ ⛔ không bằng một id giả (`-1`, `0`): một id
/// giả là một giá trị đi vào câu SQL mà ⛔ không ai kiểm được nó có va vào dữ liệu thật
/// ⛔ không. Lặp một id đã hỏi thì `IN` trả đúng cùng tập hàng — đó là ngữ nghĩa **tập
/// hợp**, ⛔ không phải một phép nối.
fn pad(chunk: &[i64]) -> [i64; SENSE_BATCH] {
    let fill = chunk[0];
    let mut padded = [fill; SENSE_BATCH];
    padded[..chunk.len()].copy_from_slice(chunk);
    padded
}

/// Chạy một câu theo lô trên toàn bộ `ids`, gom hàng qua `to_row`.
fn run_batched<T, F>(db: ReadHandle<'_>, sql: &str, ids: &[i64], to_row: F) -> SqlResult<Vec<T>>
where
    F: Fn(&Row<'_>) -> SqlResult<T> + Copy,
{
    let mut out = Vec::new();

    for chunk in ids.chunks(SENSE_BATCH) {
        let padded = pad(chunk);
        let params: Vec<&dyn ToSql> = padded.iter().map(|id| id as &dyn ToSql).collect();

        let mut stmt = db.prepare_cached(sql)?;
        let rows = stmt.query_map(params.as_slice(), to_row)?;
        for row in rows {
            out.push(row?);
        }
    }

    Ok(out)
}

/// Đọc **toàn bộ** nghĩa · ví dụ · trích dẫn cho `entry_ids`.
///
/// Kết quả sắp theo `(entry_id, ord, sense_id)` — tất định **⛔ không phụ thuộc thứ tự
/// `entry_ids` chỗ gọi truyền vào, ⛔ cũng không phụ thuộc cách chia lô**. Hai thứ tự khác
/// nhau cho cùng một tập id là một ca flaky chờ sẵn.
pub(super) fn read_senses(db: ReadHandle<'_>, entry_ids: &[i64]) -> SqlResult<Vec<SenseRecord>> {
    if entry_ids.is_empty() {
        // ⛔ Không một lượt chạm database nào. Một `SELECT` với lô rỗng trả đúng thứ này
        // sau khi đã đi qua cả pool, chỉ chậm hơn.
        return Ok(Vec::new());
    }

    // Loại trùng **trước khi chia lô**: `IN (...)` khử trùng bên trong MỘT lô (luật 2 ở
    // doc-comment module), nhưng một `entry_id` lặp lại ở HAI lô khác nhau chạy qua
    // `SENSE_SQL` hai lần và sinh hai `SenseRecord` giống hệt nhau — rồi bước gộp
    // ví dụ/trích dẫn theo `sense_id` chỉ còn nạp được cho bản đầu, bản lặp nhận danh
    // sách rỗng một cách im lặng.
    let mut seen = HashSet::new();
    let entry_ids: Vec<i64> = entry_ids
        .iter()
        .copied()
        .filter(|id| seen.insert(*id))
        .collect();
    let entry_ids = entry_ids.as_slice();

    let mut senses = run_batched(db, &SENSE_SQL, entry_ids, |row| {
        let pos_lang: Option<String> = row.get(3)?;
        Ok(SenseRecord {
            sense_id: row.get(0)?,
            entry_id: row.get(1)?,
            pos: row.get(2)?,
            // FR35 — vị từ quyết ở RUST, ⛔ ở webview (AD-1). Xem `is_foreign_lang`.
            pos_is_foreign: super::is_foreign_lang(pos_lang.as_deref()),
            pos_lang,
            gloss: row.get(4)?,
            note: row.get(5)?,
            ord: row.get(6)?,
            examples: Vec::new(),
            citations: Vec::new(),
        })
    })?;

    if senses.is_empty() {
        // Đầu mục ⛔ không có nghĩa nào là một trạng thái **hợp lệ** (một đầu mục chỉ mang
        // âm đọc, chẳng hạn), ⛔ không phải một lỗi.
        return Ok(senses);
    }

    let sense_ids: Vec<i64> = senses.iter().map(|sense| sense.sense_id).collect();

    let mut examples: HashMap<i64, Vec<ExampleRecord>> = HashMap::new();
    for (sense_id, example) in run_batched(db, &EXAMPLE_SQL, &sense_ids, |row| {
        let translation_lang: Option<String> = row.get(3)?;
        Ok((
            row.get::<_, i64>(0)?,
            ExampleRecord {
                text: row.get(1)?,
                translation: row.get(2)?,
                // AC4 — "cùng luật" nghĩa là cùng MỘT hàm, ⛔ hai bản chép.
                translation_is_foreign: super::is_foreign_lang(translation_lang.as_deref()),
                translation_lang,
                ord: row.get(4)?,
            },
        ))
    })? {
        examples.entry(sense_id).or_default().push(example);
    }

    let mut citations: HashMap<i64, Vec<CitationRecord>> = HashMap::new();
    for (sense_id, citation) in run_batched(db, &CITATION_SQL, &sense_ids, |row| {
        Ok((
            row.get::<_, i64>(0)?,
            CitationRecord {
                text: row.get(1)?,
                work: row.get(2)?,
                author: row.get(3)?,
                ord: row.get(4)?,
            },
        ))
    })? {
        citations.entry(sense_id).or_default().push(citation);
    }

    for sense in &mut senses {
        if let Some(found) = examples.remove(&sense.sense_id) {
            sense.examples = found;
        }
        if let Some(found) = citations.remove(&sense.sense_id) {
            sense.citations = found;
        }
    }

    // Một tập id trải trên nhiều lô ra nhiều khối đã sắp riêng; sắp lại một lượt để thứ tự
    // là thuộc tính của **kết quả**, ⛔ không phải của cách chia lô.
    senses.sort_by_key(|sense| (sense.entry_id, sense.ord, sense.sense_id));

    Ok(senses)
}
