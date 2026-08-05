//! Sinh `char_idx` — cặp `(ch, entry_id)` cho MỌI ký tự Hán trong `headword` VÀ trong
//! `headword_simp` (Task 5 của Story 1.9, Bẫy 8). Phủ mỗi phồn thể thì `国` trả rỗng mà
//! không lỗi nào được ném — đúng lớp lỗi FR39 tồn tại để chặn.

use rusqlite::Connection;

/// Một ký tự được coi là "Hán" nếu nằm trong một trong các khối CJK Unified Ideographs
/// (kể cả các khối mở rộng) — dải rộng có chủ ý hơn dải BMP đơn thuần, vì cả CVDICT lẫn
/// Unihan đều có đầu mục ngoài BMP (ví dụ `𠧜` đã thấy thật trong mẫu en.wiktionary).
///
/// `pub(crate)` vì `sources::thieu_chuu` cần đúng bộ dải này để nhận ra chữ Hán trong
/// trích dẫn — MỘT nguồn sự thật, ⛔ không sao chép bảng dải sang module khác (Review
/// Findings 1.10: hai bản sao cùng tên sẽ trôi khỏi nhau khi bổ sung CJK Ext H/I).
pub(crate) fn is_han(c: char) -> bool {
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

/// Chèn mọi cặp `(ch, entry_id)` cho MỘT đầu mục — gọi ngay sau khi `dict_entry` đã có
/// hàng đó, trong CÙNG giao dịch với lượt chèn đầu mục (nhất quán, ⛔ không tách pha).
///
/// Trả về số cặp đã chèn (SAU khi loại trùng trong cùng đầu mục — `headword` và
/// `headword_simp` trùng ký tự thì chỉ một cặp, nhờ `INSERT OR IGNORE` cộng khoá chính
/// `(ch, entry_id)`; đây là khử trùng lặp TRONG một đầu mục, hợp lệ theo AC2, ⛔ không
/// phải hợp nhất xuyên nguồn).
pub fn insert_for_entry(
    conn: &Connection,
    entry_id: i64,
    headword: &str,
    headword_simp: Option<&str>,
) -> rusqlite::Result<usize> {
    let mut stmt = conn.prepare_cached(
        "INSERT OR IGNORE INTO char_idx (ch, entry_id) VALUES (?1, ?2)",
    )?;
    let mut count = 0usize;
    let mut seen = std::collections::HashSet::new();
    for c in headword.chars().chain(headword_simp.unwrap_or("").chars()) {
        if is_han(c) && seen.insert(c) {
            count += stmt.execute(rusqlite::params![c.to_string(), entry_id])?;
        }
    }
    Ok(count)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn covers_both_traditional_and_simplified() {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch(
            "CREATE TABLE dict_entry (id INTEGER PRIMARY KEY);
             CREATE TABLE char_idx (ch TEXT NOT NULL, entry_id INTEGER NOT NULL, PRIMARY KEY (ch, entry_id)) WITHOUT ROWID;",
        )
        .unwrap();
        conn.execute("INSERT INTO dict_entry (id) VALUES (1)", [])
            .unwrap();

        let n = insert_for_entry(&conn, 1, "國", Some("国")).unwrap();
        assert_eq!(n, 2);

        let trad: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM char_idx WHERE ch = '國' AND entry_id = 1",
                [],
                |r| r.get(0),
            )
            .unwrap();
        let simp: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM char_idx WHERE ch = '国' AND entry_id = 1",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(trad, 1);
        assert_eq!(simp, 1);
    }

    #[test]
    fn same_character_in_both_fields_is_not_duplicated() {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch(
            "CREATE TABLE dict_entry (id INTEGER PRIMARY KEY);
             CREATE TABLE char_idx (ch TEXT NOT NULL, entry_id INTEGER NOT NULL, PRIMARY KEY (ch, entry_id)) WITHOUT ROWID;",
        )
        .unwrap();
        conn.execute("INSERT INTO dict_entry (id) VALUES (1)", [])
            .unwrap();

        let n = insert_for_entry(&conn, 1, "山", Some("山")).unwrap();
        assert_eq!(n, 1);
    }

    #[test]
    fn non_han_characters_are_not_indexed() {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch(
            "CREATE TABLE dict_entry (id INTEGER PRIMARY KEY);
             CREATE TABLE char_idx (ch TEXT NOT NULL, entry_id INTEGER NOT NULL, PRIMARY KEY (ch, entry_id)) WITHOUT ROWID;",
        )
        .unwrap();
        conn.execute("INSERT INTO dict_entry (id) VALUES (1)", [])
            .unwrap();

        let n = insert_for_entry(&conn, 1, "GDP", None).unwrap();
        assert_eq!(n, 0);
    }

    #[test]
    fn recognizes_characters_outside_the_bmp() {
        assert!(is_han('\u{20000}'));
        assert!(!is_han('a'));
        assert!(!is_han('あ'));
    }
}
