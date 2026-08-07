//! **Method thứ ba** trên cổng `DictionarySource` — âm Hán Việt theo **LÔ ký tự**.
//! Story 1.16, Quyết định #2.
//!
//! **Tệp này không bao giờ gọi vị từ điều phối** — cùng luật với `query.rs`/`senses.rs`,
//! và `tests/dict_boundary.rs` cưỡng chế nó bằng máy, đếm **tệp**.
//!
//! ─────────────────────────────────────────────────────────────────────────────
//! 🔴 BA LUẬT CỦA TỆP NÀY — CẢ BA HỎNG THÀNH MỘT LƯỢT CI XANH
//! ─────────────────────────────────────────────────────────────────────────────
//! 1. **Không một truy vấn cho mỗi ký tự (N+1).** Một Chương 3.000 ký tự ⇒ ~1.500 ký
//!    tự riêng — cùng bài học `senses.rs`, và cùng lý do `Quyết định #2` của story loại
//!    đường "thêm trường `han_viet` vào `EntryHit` rồi gọi `lookup()` cho từng ký tự".
//! 2. **Lô cỡ CỐ ĐỊNH, không co giãn** — cùng lý do `SENSE_BATCH`: một hình dạng SQL
//!    mới mỗi lần làm trống cache `prepare_cached`.
//! 3. **Câu SQL phủ CẢ `headword` LẪN `headword_simp`, không lọc theo `source.code`** — Bẫy 8
//!    của Story 1.9 (giản thể trả rỗng) cộng AD-10 (runtime không có mã riêng cho từng
//!    nguồn: `WHERE han_viet IS NOT NULL` **là** bộ lọc đúng, năm nguồn nền không mang cột
//!    này đều `NULL` toàn bộ — Quyết định #1 của story đã đo).
//!
//! ⚠️ **Tham số SQLite được đánh số dùng lại được** — `?1` xuất hiện ở CẢ HAI vế `IN (…)`
//! của câu dưới đây và bind **cùng một** danh sách giá trị ở cả hai chỗ (đo thật bằng
//! `sqlite3`/Python trước khi viết dòng này — đúng tinh thần "đo trước khi chốt" của
//! story). Đó là lý do câu này chỉ cần MỘT danh sách placeholder, không hai.

use std::collections::HashSet;
use std::sync::LazyLock;

use crate::core::store::{ReadHandle, Row, SqlResult, ToSql};

use super::HanVietHit;

/// Số ký tự trong **một** lô. Cùng con số với [`super::SENSE_BATCH`] — cùng lý lẽ đánh đổi
/// (nhiều lô nhỏ ↔ tham số thừa trong lô ngắn), và `SQLITE_MAX_VARIABLE_NUMBER` (32.766 ở
/// SQLite ≥ 3.32) không ở gần trần này.
pub const HAN_VIET_BATCH: usize = 64;

/// `?1, ?2, …, ?64` — dựng **một lần**, dùng lại cho mọi lô VÀ cho cả hai vế `IN (…)`.
static PLACEHOLDERS: LazyLock<String> = LazyLock::new(|| {
    (1..=HAN_VIET_BATCH)
        .map(|i| format!("?{i}"))
        .collect::<Vec<_>>()
        .join(", ")
});

/// 🔴 `WHERE (… IN (P) OR … IN (P)) AND han_viet IS NOT NULL` — MỘT câu, mọi tệp trả lời
/// bằng chính câu này (AD-10). **Không** `WHERE source.code = …` — năm nguồn nền còn
/// lại (ngoài lớp gỡ rời Hán Việt) đã đo là `NULL` toàn bộ ở cột `han_viet`, nên bộ lọc
/// `IS NOT NULL` tự nhiên loại chúng mà không cần biết tệp nào chứa gì.
static HAN_VIET_SQL: LazyLock<String> = LazyLock::new(|| {
    format!(
        "SELECT e.headword, e.headword_simp, e.han_viet, s.code \
         FROM dict_entry e JOIN dict_source s ON s.id = e.source_id \
         WHERE (e.headword IN ({p}) OR e.headword_simp IN ({p})) \
           AND e.han_viet IS NOT NULL \
         ORDER BY e.id",
        p = *PLACEHOLDERS
    )
});

/// Một lô **đủ [`HAN_VIET_BATCH`] phần tử** — phần thiếu đệm bằng phần tử đầu của lô.
/// Cùng lý lẽ `senses.rs::pad`: đệm bằng một ký tự **đã có trong lô** không bằng một
/// giá trị giả, vì `IN` là phép kiểm tập hợp — lặp lại không sinh thêm hàng nào.
fn pad(chunk: &[&str]) -> Vec<String> {
    let fill = chunk[0].to_owned();
    let mut padded = vec![fill; HAN_VIET_BATCH];
    for (i, c) in chunk.iter().enumerate() {
        padded[i] = (*c).to_owned();
    }
    padded
}

fn row_to_tuple(row: &Row<'_>) -> SqlResult<(String, Option<String>, String, String)> {
    Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?))
}

/// Đọc âm Hán Việt cho `chars` — **một tệp, một lượt**, theo lô.
///
/// 🔴 `chars` **nhận từ chỗ gọi**, không dedupe ở tầng gom trước khi gọi hàm này là một
/// kỳ vọng, không một điều kiện — hàm này tự dedupe **trước khi chia lô**, cùng lý do
/// `senses.rs::read_senses`: một ký tự lặp ở hai lô khác nhau chạy hai lượt SQL cho cùng
/// một câu hỏi.
///
/// Mỗi hàng khớp sinh ra **một hoặc hai** [`HanVietHit`]: một cho `headword` nếu nó nằm
/// trong tập của **lô đang chạy**, một cho `headword_simp` nếu nó KHÁC `headword` VÀ cũng
/// nằm trong tập của lô đó — ca giản thể/phồn thể khớp ở hai trường khác nhau của CÙNG
/// một hàng.
///
/// 🔴 **VÌ SAO tập lọc phải theo LÔ, không phải tập truy vấn đầy đủ** *(lỗi thật, bắt ở
/// lượt code review 2026-08-06)*: với một tập đầy đủ, một hàng có `headword` ở lô A và
/// `headword_simp` ở lô B bị **cả hai** lô trả về, và **cả hai** lô đẩy **cả hai**
/// [`HanVietHit`] ⇒ **4 hit thay vì 2**. Tệ hơn hàng trùng: `out` nối theo thứ tự LÔ, nên
/// một ký tự có thể nhận hit phát ra từ lô của một ký tự KHÁC — và `or_insert` ở tầng gom
/// lấy hit đến trước. Hệ quả: **âm được chọn phụ thuộc VỊ TRÍ ký tự trong Chương**, cùng
/// chữ và cùng tệp `.db` cho hai kết quả khác nhau.
///
/// ⚠️ Đo trên bốn tệp `.db` thật: **0** hàng mang `headword_simp` khác `headword` ở cột
/// `han_viet` ⇒ hôm nay **chưa chạm tới được**. Vá vì đây là một cổng **công khai** mà
/// Story 1.17/3.7 sẽ tiêu thụ trực tiếp, không vì nó đang cháy.
pub(super) fn read_han_viet(db: ReadHandle<'_>, chars: &[&str]) -> SqlResult<Vec<HanVietHit>> {
    if chars.is_empty() {
        return Ok(Vec::new());
    }

    let mut seen = HashSet::new();
    let chars: Vec<&str> = chars.iter().copied().filter(|c| seen.insert(*c)).collect();

    let mut out = Vec::new();

    for chunk in chars.chunks(HAN_VIET_BATCH) {
        // 🔴 Tập lọc là tập của **CHÍNH LÔ NÀY**, không phải tập truy vấn đầy đủ — xem
        // doc-comment của hàm. Dựng lại cho mỗi lô: `HAN_VIET_BATCH` phần tử, không đáng
        // kể so với một lượt SQL.
        let query_set: HashSet<&str> = chunk.iter().copied().collect();

        let padded = pad(chunk);
        let params: Vec<&dyn ToSql> = padded.iter().map(|s| s as &dyn ToSql).collect();

        let mut stmt = db.prepare_cached(&HAN_VIET_SQL)?;
        let rows = stmt.query_map(params.as_slice(), row_to_tuple)?;

        for row in rows {
            let (headword, headword_simp, han_viet, source_code) = row?;

            if query_set.contains(headword.as_str()) {
                out.push(HanVietHit {
                    character: headword.clone(),
                    reading: han_viet.clone(),
                    source_code: source_code.clone(),
                });
            }

            if let Some(simp) = headword_simp.as_deref() {
                if simp != headword.as_str() && query_set.contains(simp) {
                    out.push(HanVietHit {
                        character: simp.to_owned(),
                        reading: han_viet.clone(),
                        source_code: source_code.clone(),
                    });
                }
            }
        }
    }

    Ok(out)
}
