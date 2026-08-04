//! Nghiệm thu lược đồ độc lập với parser — Task 3 của Story 1.9. Dựng schema trên một
//! kết nối trong bộ nhớ và kiểm các bất biến mà §Quyết định #2/#3 chốt cứng.

use dict_build::{insert, schema};
use rusqlite::Connection;

fn open_with_schema() -> Connection {
    let conn = Connection::open_in_memory().unwrap();
    conn.execute_batch("PRAGMA foreign_keys = ON;").unwrap();
    insert::create_schema(&conn).expect("schema DDL must apply cleanly");
    conn
}

#[test]
fn every_table_and_virtual_table_from_the_decision_exists() {
    let conn = open_with_schema();
    let mut stmt = conn
        .prepare("SELECT name FROM sqlite_master WHERE type IN ('table','view') ORDER BY name")
        .unwrap();
    let names: Vec<String> = stmt
        .query_map([], |r| r.get(0))
        .unwrap()
        .collect::<Result<_, _>>()
        .unwrap();

    for expected in [
        "dict_meta",
        "dict_source",
        "dict_entry",
        "dict_sense",
        "dict_example",
        "dict_citation",
        "char_idx",
        "entry_fts",
        "sense_fts",
        "sense_fts_nd",
    ] {
        assert!(
            names.iter().any(|n| n == expected),
            "missing table/virtual table '{expected}', got {names:?}"
        );
    }
}

/// §Bẫy 3: một bảng FTS5 external-content KHÔNG `rebuild` trả 0 hàng, KHÔNG lỗi. Test
/// này chứng minh hành vi đó tồn tại — để `finalize::rebuild_fts` có gì để sửa.
#[test]
fn fts_without_rebuild_silently_returns_zero_rows_not_an_error() {
    let conn = open_with_schema();
    conn.execute(
        "INSERT INTO dict_source (code, display_name, license_kind, license_text, attribution, source_version, source_url)
         VALUES ('t','t','open','t','t','t','t')",
        [],
    )
    .unwrap();
    conn.execute(
        "INSERT INTO dict_entry (source_id, lang, headword) VALUES (1, 'zh', '山')",
        [],
    )
    .unwrap();
    conn.execute(
        "INSERT INTO dict_sense (entry_id, source_id, gloss, ord) VALUES (1, 1, 'mountain', 0)",
        [],
    )
    .unwrap();

    // ⛔ Không rebuild — đúng hình dạng lỗi của Bẫy 3.
    let hits: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM sense_fts WHERE sense_fts MATCH 'mountain'",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(hits, 0, "without rebuild, external-content FTS must be empty, not error");

    conn.execute_batch("INSERT INTO sense_fts(sense_fts) VALUES('rebuild');")
        .unwrap();
    let hits_after: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM sense_fts WHERE sense_fts MATCH 'mountain'",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(hits_after, 1, "after rebuild, the row must be found");
}

/// §Quyết định #7: `PRAGMA user_version` VÀ `dict_meta('schema_version', …)` — hai chỗ,
/// vì `user_version` là đường đọc rẻ, `dict_meta` là thứ người đọc tệp bằng tay thấy.
#[test]
fn schema_version_is_recorded_in_both_places() {
    let conn = open_with_schema();
    insert::insert_meta(&conn).unwrap();

    let user_version: i64 = conn
        .query_row("PRAGMA user_version", [], |r| r.get(0))
        .unwrap();
    assert_eq!(user_version, schema::SCHEMA_VERSION as i64);

    let meta_version: String = conn
        .query_row(
            "SELECT value FROM dict_meta WHERE key = 'schema_version'",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(meta_version, schema::SCHEMA_VERSION.to_string());
}

/// AC1 mệnh đề 5: `dict_example`/`dict_citation` treo vào `sense_id`, KHÔNG có cột nào
/// tham chiếu `entry_id` trực tiếp — kiểm bằng cách khẳng định DDL không khai nó.
#[test]
fn examples_and_citations_reference_sense_not_entry() {
    assert!(schema::DICT_EXAMPLE_DDL.contains("sense_id"));
    assert!(!schema::DICT_EXAMPLE_DDL.contains("entry_id"));
    assert!(schema::DICT_CITATION_DDL.contains("sense_id"));
    assert!(!schema::DICT_CITATION_DDL.contains("entry_id"));
}

/// Bổ sung cho test trên: chuỗi DDL đúng khuôn không chứng minh ràng buộc CÓ HIỆU LỰC
/// lúc chạy — chèn thật với `sense_id` trỏ vào một hàng không tồn tại phải bị SQLite từ
/// chối (Review Findings Group B).
#[test]
fn dict_example_and_dict_citation_sense_id_foreign_key_is_enforced_live() {
    let conn = open_with_schema();
    conn.execute(
        "INSERT INTO dict_source (code, display_name, license_kind, license_text, attribution, source_version, source_url)
         VALUES ('t','t','open','t','t','t','t')",
        [],
    )
    .unwrap();
    conn.execute(
        "INSERT INTO dict_entry (source_id, lang, headword) VALUES (1, 'zh', '山')",
        [],
    )
    .unwrap();
    // ⛔ Không hàng `dict_sense` nào được chèn — `sense_id = 999` không tồn tại.
    let example_result = conn.execute(
        "INSERT INTO dict_example (sense_id, text, ord) VALUES (999, 'x', 0)",
        [],
    );
    assert!(example_result.is_err(), "dict_example.sense_id must reject a non-existent sense_id");

    let citation_result = conn.execute(
        "INSERT INTO dict_citation (sense_id, text, ord) VALUES (999, 'x', 0)",
        [],
    );
    assert!(citation_result.is_err(), "dict_citation.sense_id must reject a non-existent sense_id");
}

/// AC5 mệnh đề 1/2: hai bảng trên `gloss`, tên trần = chính, hậu tố `_nd` = phụ.
#[test]
fn primary_sense_fts_is_diacritic_sensitive_by_tokenizer_declaration() {
    assert!(schema::SENSE_FTS_DDL.contains("remove_diacritics 0"));
    assert!(schema::SENSE_FTS_ND_DDL.contains("remove_diacritics 2"));
}

/// AC5 mệnh đề 4 / Bẫy 5: trigram nằm trên `headword` của `dict_entry`, KHÔNG trên
/// `gloss` của `dict_sense`.
#[test]
fn trigram_index_targets_headword_not_gloss() {
    assert!(schema::ENTRY_FTS_DDL.contains("headword"));
    assert!(schema::ENTRY_FTS_DDL.contains("trigram"));
    assert!(!schema::ENTRY_FTS_DDL.contains("gloss"));
}

#[test]
fn char_idx_is_without_rowid_with_composite_primary_key() {
    assert!(schema::CHAR_IDX_DDL.contains("WITHOUT ROWID"));
    assert!(schema::CHAR_IDX_DDL.contains("PRIMARY KEY (ch, entry_id)"));
}

/// Bổ sung cho test trên: chèn thật một cặp `(ch, entry_id)` TRÙNG phải bị khoá chính
/// tổng hợp từ chối — chuỗi DDL đúng khuôn không tự chứng minh điều đó lúc chạy (Review
/// Findings Group B).
#[test]
fn char_idx_composite_primary_key_rejects_a_duplicate_pair_live() {
    let conn = open_with_schema();
    conn.execute(
        "INSERT INTO dict_source (code, display_name, license_kind, license_text, attribution, source_version, source_url)
         VALUES ('t','t','open','t','t','t','t')",
        [],
    )
    .unwrap();
    conn.execute(
        "INSERT INTO dict_entry (source_id, lang, headword) VALUES (1, 'zh', '山')",
        [],
    )
    .unwrap();
    conn.execute("INSERT INTO char_idx (ch, entry_id) VALUES ('山', 1)", [])
        .unwrap();
    let dup = conn.execute("INSERT INTO char_idx (ch, entry_id) VALUES ('山', 1)", []);
    assert!(dup.is_err(), "a duplicate (ch, entry_id) pair must be rejected by the composite primary key");
}
