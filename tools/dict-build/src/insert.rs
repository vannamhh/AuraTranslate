//! Chèn dữ liệu vào `dict-core.db` — cầu nối giữa `model::RawEntry` (hình dạng trung
//! gian của parser) và lược đồ SQL (`schema.rs`). Story 1.9, Task 3/4/5.

use rusqlite::{Connection, params};

use crate::char_idx;
use crate::model::RawEntry;
use crate::sources_meta::SourceMeta;

/// Dựng schema — mọi hằng DDL của `schema.rs`, theo đúng thứ tự khai (tôn trọng FK).
pub fn create_schema(conn: &Connection) -> rusqlite::Result<()> {
    for ddl in crate::schema::ALL_TABLE_DDL {
        conn.execute_batch(ddl)?;
    }
    Ok(())
}

/// Chèn một hàng `dict_source`, trả về `id` vừa sinh. `source_version` được ĐO lúc
/// chạy (không phải hằng biên dịch) — xem `sources_meta::SourceMeta`.
pub fn insert_source(
    conn: &Connection,
    meta: &SourceMeta,
    source_version: &str,
) -> rusqlite::Result<i64> {
    conn.execute(
        "INSERT INTO dict_source
           (code, display_name, license_kind, license_id, license_text, attribution, source_version, source_url)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        params![
            meta.code,
            meta.display_name,
            meta.license_kind,
            meta.license_id,
            meta.license_text(),
            meta.attribution,
            source_version,
            meta.source_url,
        ],
    )?;
    Ok(conn.last_insert_rowid())
}

/// Chèn MỘT `RawEntry` — `dict_entry` + mọi `dict_sense`/`dict_example`/`dict_citation`
/// của nó + cặp `char_idx` phủ cả `headword` lẫn `headword_simp` (Task 5), tất cả gắn
/// `source_id` đã biết. 🔴 AC2: `source_id` là `NOT NULL` ở cả `dict_entry` lẫn
/// `dict_sense` — không hàng nào chèn được nếu thiếu.
pub fn insert_entry(conn: &Connection, source_id: i64, entry: &RawEntry) -> rusqlite::Result<i64> {
    conn.execute(
        "INSERT INTO dict_entry (source_id, lang, headword, headword_simp, reading, han_viet)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        params![
            source_id,
            entry.lang,
            entry.headword,
            entry.headword_simp,
            entry.reading,
            entry.han_viet,
        ],
    )?;
    let entry_id = conn.last_insert_rowid();

    char_idx::insert_for_entry(
        conn,
        entry_id,
        &entry.headword,
        entry.headword_simp.as_deref(),
    )?;

    for (ord, sense) in entry.senses.iter().enumerate() {
        conn.execute(
            "INSERT INTO dict_sense (entry_id, source_id, pos, pos_lang, gloss, note, ord)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                entry_id,
                source_id,
                sense.pos,
                sense.pos_lang,
                sense.gloss,
                sense.note,
                ord as i64,
            ],
        )?;
        let sense_id = conn.last_insert_rowid();

        for (ex_ord, ex) in sense.examples.iter().enumerate() {
            conn.execute(
                "INSERT INTO dict_example (sense_id, text, translation, translation_lang, ord)
                 VALUES (?1, ?2, ?3, ?4, ?5)",
                params![sense_id, ex.text, ex.translation, ex.translation_lang, ex_ord as i64],
            )?;
        }

        for (cit_ord, cit) in sense.citations.iter().enumerate() {
            conn.execute(
                "INSERT INTO dict_citation (sense_id, text, work, author, ord)
                 VALUES (?1, ?2, ?3, ?4, ?5)",
                params![sense_id, cit.text, cit.work, cit.author, cit_ord as i64],
            )?;
        }
    }

    Ok(entry_id)
}

/// `dict_meta` — `schema_version` cộng dấu vết dựng tệp (§Quyết định #2: hai chỗ ghi
/// phiên bản vì `user_version` là đường đọc kiểm rẻ nhất, `dict_meta` là thứ người đọc
/// tệp bằng tay thấy được).
///
/// `layer` là `"base"` hoặc mã lớp gỡ rời (vd. `"thieu-chuu"`) — Story 1.10, §Quyết định
/// #5: một HÀNG trong bảng khoá/giá trị ĐÃ CÓ, ⛔ không phải cột mới ⇒ `sqlite_master`
/// không đổi ⇒ AC4 vẫn đạt. Story 1.13 đọc hàng này để biết mình vừa mở tệp nào TRƯỚC
/// khi đọc `dict_source`.
///
/// 🔴 `built_at` là THAM SỐ, ⛔ không phải `strftime('now')` (Ice chốt 2026-08-05, Review
/// Findings). Đồng hồ với độ phân giải mili-giây làm **mọi** lượt build ra một tệp khác
/// byte, tức mọi giá trị `sha256` trong `dict-manifest.toml` chỉ đúng cho đúng một lượt
/// chạy — một lần `cargo run` lại trước khi upload là mọi máy khách fail checksum, và
/// ⛔ không cổng nào bắt được (cổng manifest cố ý không đọc `.db`). AD-25 đòi artifact có
/// checksum; checksum chỉ có nghĩa khi build tái lập được. Giá trị do `build::built_at`
/// dẫn xuất từ CHÍNH nguồn thô, ⛔ không từ đồng hồ.
pub fn insert_meta(conn: &Connection, layer: &str, built_at: &str) -> rusqlite::Result<()> {
    conn.execute(
        "INSERT INTO dict_meta (key, value) VALUES ('schema_version', ?1)",
        params![crate::schema::SCHEMA_VERSION.to_string()],
    )?;
    conn.execute(
        "INSERT INTO dict_meta (key, value) VALUES ('built_at', ?1)",
        params![built_at],
    )?;
    conn.execute(
        "INSERT INTO dict_meta (key, value) VALUES ('builder_version', ?1)",
        params![env!("CARGO_PKG_VERSION")],
    )?;
    conn.execute(
        "INSERT INTO dict_meta (key, value) VALUES ('layer', ?1)",
        params![layer],
    )?;
    conn.execute_batch(&format!(
        "PRAGMA user_version = {};",
        crate::schema::SCHEMA_VERSION
    ))?;
    Ok(())
}
