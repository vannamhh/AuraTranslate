//! Nghiệm thu tích hợp trên FIXTURE (Task 4/11 của Story 1.9) — chạy TRỌN pipeline
//! `build::run` (schema → 5 parser → chèn → char_idx → rebuild FTS → VACUUM →
//! journal_mode=DELETE) trên dữ liệu fixture thật (trích từ CVDICT/CC-CEDICT/Unihan/
//! kaikki.org thật, không phải bịa).
//!
//! ⚠️ Đây là test CHỨNG MINH MÃ ĐÚNG. Test CHỨNG MINH DỮ LIỆU đúng chạy trên
//! `dict-core.db` dựng từ NĂM NGUỒN THẬT ở Task 11, ghi vào Debug Log References của
//! story — không lẫn hai loại nghiệm thu này (Testing standards, Story 1.9).

use std::path::PathBuf;

use dict_build::build;
use rusqlite::Connection;

fn fixtures_raw_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/raw")
}

fn build_fixture_db() -> (tempfile::TempDir, PathBuf, build::BuildReport) {
    let dir = tempfile::tempdir().unwrap();
    let out = dir.path().join("dict-core-fixture.db");
    let report = build::run(&fixtures_raw_dir(), &out).expect("fixture build should succeed");
    (dir, out, report)
}

#[test]
fn all_five_sources_produce_at_least_one_entry() {
    let (_dir, _out, report) = build_fixture_db();
    assert_eq!(report.per_source.len(), 5);
    for s in &report.per_source {
        assert!(
            s.entries > 0,
            "source {} produced zero entries from its fixture",
            s.source_code
        );
    }
}

/// `en_wiktionary/Chinese.jsonl` fixture chứa MỘT dòng cố ý hỏng cú pháp (thiếu
/// `word`) — `skip_reasons` phải ghi nhận đúng lý do đó, ⛔ không chỉ "entries > 0" mù
/// mờ (Review Findings Group B — trước đây không test nào đọc `skip_reasons`/
/// `lines_skipped`, nên dòng cố ý hỏng của fixture chưa từng thật sự được nghiệm thu
/// qua pipeline).
#[test]
fn malformed_en_wiktionary_line_is_recorded_with_its_real_skip_reason() {
    let (_dir, _out, report) = build_fixture_db();
    let en = report
        .per_source
        .iter()
        .find(|s| s.source_code == "en-wiktionary")
        .expect("en-wiktionary must be present");
    assert!(en.lines_skipped > 0, "expected at least one skipped line");
    assert!(
        en.skip_reasons.keys().any(|r| r.contains("missing 'word' field")),
        "expected a \"missing 'word' field\" skip reason, got {:?}",
        en.skip_reasons
    );
}

/// Group A vá lỗi "nhiều dòng JSONL cùng headword ⇒ nhiều `dict_entry`" bằng cách gộp
/// theo headword TRONG một nguồn (`wiktextract_common::parse`). Test này khoá lại hành
/// vi đó ở tầng TÍCH HỢP, qua `build::run` THẬT, trên dữ liệu THẬT: fixture
/// `en_wiktionary/Chinese.jsonl` có `馬` ở BA dòng JSONL thật — hai dòng nghĩa dùng
/// được cộng một dòng `tags:["no-gloss"]` thật (Review Findings Group B).
#[test]
fn en_wiktionary_same_headword_ma_merges_into_one_entry_with_multiple_senses() {
    let (_dir, out, _report) = build_fixture_db();
    let conn = Connection::open(&out).unwrap();

    let entry_count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM dict_entry de
               JOIN dict_source src ON src.id = de.source_id
             WHERE de.headword = '馬' AND src.code = 'en-wiktionary'",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(
        entry_count, 1,
        "馬 across 3 real en-wiktionary JSONL lines (same headword) must merge into exactly 1 dict_entry"
    );

    let sense_count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM dict_sense ds
               JOIN dict_entry de ON de.id = ds.entry_id
               JOIN dict_source src ON src.id = ds.source_id
             WHERE de.headword = '馬' AND src.code = 'en-wiktionary'",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert!(
        sense_count >= 2,
        "the merged entry must carry senses from BOTH real usable-gloss lines, got {sense_count}"
    );
}

/// Nghiệm thu lại kịch bản patch WAL-path của Group A: `verify_no_wal_artifacts` phải
/// đúng bất kể `--out` có đuôi `.db` hay không — trước đây test tích hợp duy nhất luôn
/// dựng vào một đường dẫn CÓ đuôi `.db`, nên không bao giờ tự exercise được lời hứa đó
/// (Review Findings Group B).
#[test]
fn wal_check_survives_output_path_without_dot_db_extension() {
    let dir = tempfile::tempdir().unwrap();
    let out = dir.path().join("dict-core-fixture"); // KHÔNG có đuôi .db, cố ý
    let report = build::run(&fixtures_raw_dir(), &out).expect("build must succeed regardless of extension");
    assert_eq!(report.journal_mode.to_lowercase(), "delete");
    assert!(!dict_build::finalize::sibling_path(&out, "-wal").exists());
    assert!(!dict_build::finalize::sibling_path(&out, "-shm").exists());
    assert!(out.exists(), "output file itself must exist at the exact requested path");
}

/// Nghiệm thu lại kịch bản patch tệp-tạm-rồi-rename của Group A: một lượt build phải
/// thành công dù `out_path` (cộng `-wal`/`-shm` cạnh nó) đã có rác từ lượt trước — trước
/// đây không test nào dựng vào một đích ĐÃ CÓ tệp (Review Findings Group B).
#[test]
fn rebuilding_over_a_stale_output_and_wal_artifacts_still_succeeds() {
    let dir = tempfile::tempdir().unwrap();
    let out = dir.path().join("dict-core-fixture.db");
    std::fs::write(&out, b"stale garbage from a previous run").unwrap();
    std::fs::write(dict_build::finalize::sibling_path(&out, "-wal"), b"stale wal").unwrap();
    std::fs::write(dict_build::finalize::sibling_path(&out, "-shm"), b"stale shm").unwrap();

    let report = build::run(&fixtures_raw_dir(), &out).expect("build must succeed over stale artifacts");
    assert!(report.per_source.iter().all(|s| s.entries > 0));
    assert!(!dict_build::finalize::sibling_path(&out, "-wal").exists());
    assert!(!dict_build::finalize::sibling_path(&out, "-shm").exists());

    let conn = Connection::open(&out).unwrap();
    let table_count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM sqlite_master WHERE type = 'table' AND name = 'dict_meta'",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(table_count, 1, "rebuilt file must be a fresh, well-formed database, not the stale garbage");
}

/// Đúng lỗi "rơi về `unknown` âm thầm" mà Group A vá (`version_or_warn`) — nếu tái phát,
/// bộ test này phải bắt được, ⛔ không chỉ dựa vào việc đọc log console bằng mắt (Review
/// Findings Group B).
#[test]
fn all_sources_have_a_real_non_unknown_source_version() {
    let (_dir, out, _report) = build_fixture_db();
    let conn = Connection::open(&out).unwrap();
    let mut stmt = conn
        .prepare("SELECT code, source_version FROM dict_source ORDER BY code")
        .unwrap();
    let rows: Vec<(String, String)> = stmt
        .query_map([], |r| Ok((r.get(0)?, r.get(1)?)))
        .unwrap()
        .collect::<Result<_, _>>()
        .unwrap();
    assert_eq!(rows.len(), 5);
    for (code, version) in rows {
        assert!(!version.is_empty(), "{code} has an empty source_version");
        assert_ne!(version, "unknown", "{code} silently fell back to 'unknown' source_version");
    }
}

/// AC2, hình dạng fixture: `山` có mặt ở CẢ `cvdict` LẪN `cc-cedict` (đã kiểm thật khi
/// dựng fixture — dòng 35089-35090 của `CVDICT.u8` và dòng 35485-35486 của
/// `cedict.txt`, cùng thời điểm 2026-08-04). Phải cho ra ≥ 2 hàng `dict_sense` với
/// `source_id` KHÁC NHAU, và KHÔNG một hàng nào bị nuốt bởi một bước hợp nhất.
#[test]
fn ac2_shan_appears_under_two_different_source_ids_not_merged() {
    let (_dir, out, _report) = build_fixture_db();
    let conn = Connection::open(&out).unwrap();

    let mut stmt = conn
        .prepare(
            "SELECT DISTINCT ds.source_id, src.code
             FROM dict_sense ds
             JOIN dict_entry de ON de.id = ds.entry_id
             JOIN dict_source src ON src.id = ds.source_id
             WHERE de.headword = '山'
             ORDER BY src.code",
        )
        .unwrap();
    let source_codes: Vec<String> = stmt
        .query_map([], |r| r.get::<_, String>(1))
        .unwrap()
        .collect::<Result<_, _>>()
        .unwrap();

    assert!(
        source_codes.len() >= 2,
        "expected 山 under >= 2 distinct source_id, got {source_codes:?}"
    );
    assert!(source_codes.contains(&"cvdict".to_string()));
    assert!(source_codes.contains(&"cc-cedict".to_string()));

    // Siết chính xác theo dữ liệu thật (Review Findings Group B — bound `>= 2` cũ lỏng
    // hơn dữ liệu thật hỗ trợ và không bắt được ca gộp lầm TRONG một nguồn): fixture
    // CVDICT.u8 dòng 10-11 và cedict.txt dòng 19-20 mỗi nguồn có ĐÚNG hai dòng cho "山"
    // (nghĩa "họ Sơn" và "núi") — nếu build gộp nhầm hai dòng CÙNG nguồn thành một, số
    // này tụt xuống 1 mà bound `>= 2` cũ (trên tổng, không theo từng nguồn) không bắt
    // được.
    for src in ["cvdict", "cc-cedict"] {
        let per_source_entry_count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM dict_entry de
                   JOIN dict_source s ON s.id = de.source_id
                 WHERE de.headword = '山' AND s.code = ?1",
                [src],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(
            per_source_entry_count, 2,
            "expected exactly 2 separate dict_entry rows for 山 within source '{src}' (one per real fixture line), got {per_source_entry_count}"
        );
    }
}

/// AC5 hai chiều trên dữ liệu fixture: chỉ mục CHÍNH phân biệt dấu, chỉ mục PHỤ xoá
/// dấu. `mountain` (gloss thật của Unihan `U+5C71`) không đủ để thử tiếng Việt có dấu,
/// nên test này dùng `note`/`gloss` gốc từ CVDICT — cụm "núi; đồi" (dòng thật đã tải).
#[test]
fn ac5_primary_fts_is_diacritic_sensitive_secondary_is_not() {
    let (_dir, out, _report) = build_fixture_db();
    let conn = Connection::open(&out).unwrap();

    // "núi" có dấu — cả hai chỉ mục phải bắt được nó (đây không phải cặp đối lập dấu).
    let hits_primary: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM sense_fts WHERE sense_fts MATCH 'núi'",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert!(hits_primary > 0, "primary FTS should find accented 'núi'");

    // Chỉ mục phụ (remove_diacritics=2) phải khớp cả bản KHÔNG dấu "nui" — vì nó xoá
    // dấu của CHÍNH VĂN BẢN ĐÃ LẬP CHỈ MỤC, không chỉ của truy vấn.
    let hits_secondary_no_diacritics: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM sense_fts_nd WHERE sense_fts_nd MATCH 'nui'",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert!(
        hits_secondary_no_diacritics > 0,
        "secondary FTS (remove_diacritics=2) should match the undiacritized query 'nui'"
    );

    // Và chỉ mục CHÍNH KHÔNG được khớp bản không dấu — đó là toàn bộ ý nghĩa của AD-27.
    let hits_primary_no_diacritics: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM sense_fts WHERE sense_fts MATCH 'nui'",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(
        hits_primary_no_diacritics, 0,
        "primary FTS must NOT match the undiacritized query — remove_diacritics 0 is the whole point"
    );
}

/// AD-26 dữ liệu (không phải đường tra cứu — Story 1.11): `entry_fts` (trigram trên
/// ĐẦU MỤC) và `char_idx` phải trả khác rỗng cho các đầu mục có trong fixture.
///
/// ⚠️ AD-26 nhánh 3 là *"chuỗi con 3+ ký tự"* — trigram không sinh token nào cho một
/// headword MỘT ký tự như `山` (đúng lý do §Thông tin kỹ thuật Task 11 dùng `中國人`,
/// KHÔNG dùng `山`, để thử `entry_fts`). `屯溪區` (3 ký tự, có thật trong fixture
/// CC-CEDICT) là ứng viên đúng tầm cho trigram; `山` thử bằng `char_idx`, không bằng
/// `entry_fts`.
#[test]
fn ad26_entry_trigram_and_char_idx_are_populated_and_queryable() {
    let (_dir, out, _report) = build_fixture_db();
    let conn = Connection::open(&out).unwrap();

    let trigram_hits: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM entry_fts WHERE entry_fts MATCH '屯溪區'",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert!(trigram_hits > 0, "entry_fts trigram should find 屯溪區 (3+ chars)");

    let char_idx_hits: i64 = conn
        .query_row("SELECT COUNT(*) FROM char_idx WHERE ch = '山'", [], |r| {
            r.get(0)
        })
        .unwrap();
    assert!(char_idx_hits > 0, "char_idx should have a row for 山");
}

/// Bẫy 8 trên fixture: `char_idx` phủ CẢ `國` (phồn) LẪN `国` (giản) — Unihan fixture
/// khai `U+570B kSimplifiedVariant U+56FD` thật.
#[test]
fn char_idx_covers_both_traditional_and_simplified_forms() {
    let (_dir, out, _report) = build_fixture_db();
    let conn = Connection::open(&out).unwrap();

    let trad: i64 = conn
        .query_row("SELECT COUNT(*) FROM char_idx WHERE ch = '國'", [], |r| {
            r.get(0)
        })
        .unwrap();
    let simp: i64 = conn
        .query_row("SELECT COUNT(*) FROM char_idx WHERE ch = '国'", [], |r| {
            r.get(0)
        })
        .unwrap();
    assert!(trad > 0, "char_idx must cover the traditional form 國");
    assert!(simp > 0, "char_idx must cover the simplified form 国");
}

/// AC2, cơ chế 1 (lược đồ): `dict_sense.source_id` là `NOT NULL` — không hàng nào chèn
/// được nếu thiếu, cưỡng chế bởi chính SQLite chứ không phải người viết.
#[test]
fn dict_sense_source_id_rejects_null_by_schema() {
    let (_dir, out, _report) = build_fixture_db();
    let conn = Connection::open(&out).unwrap();
    conn.execute_batch("PRAGMA foreign_keys = ON;").unwrap();

    let entry_id: i64 = conn
        .query_row("SELECT id FROM dict_entry LIMIT 1", [], |r| r.get(0))
        .unwrap();
    let result = conn.execute(
        "INSERT INTO dict_sense (entry_id, source_id, gloss, ord) VALUES (?1, NULL, 'x', 0)",
        rusqlite::params![entry_id],
    );
    assert!(result.is_err(), "inserting a NULL source_id must fail");
}

/// Task 6: tệp giao ra ở `journal_mode = DELETE`, không `-wal`/`-shm` sót lại (Bẫy 1).
#[test]
fn built_file_uses_delete_journal_mode_with_no_wal_artifacts() {
    let (_dir, out, report) = build_fixture_db();
    assert_eq!(report.journal_mode.to_lowercase(), "delete");
    assert!(!out.with_extension("db-wal").exists());
    assert!(!out.with_extension("db-shm").exists());

    let conn = Connection::open(&out).unwrap();
    let mode: String = conn
        .query_row("PRAGMA journal_mode", [], |r| r.get(0))
        .unwrap();
    assert_eq!(mode.to_lowercase(), "delete");
}

/// AC3 vế cơ chế: SHA-256 và kích thước byte được in ra và khớp tệp thật.
#[test]
fn build_report_includes_a_real_sha256_and_matching_size() {
    let (_dir, out, report) = build_fixture_db();
    assert_eq!(report.sha256.len(), 64);
    assert!(report.sha256.chars().all(|c| c.is_ascii_hexdigit() && !c.is_ascii_uppercase()));
    let real_size = std::fs::metadata(&out).unwrap().len();
    assert_eq!(report.size_bytes, real_size);

    // Băm lại tệp output THẬT và so với `report.sha256` — trước đây chỉ kiểm HÌNH DẠNG
    // (64 hex thường), nên một hash tính từ nội dung stale (vd. băm trước VACUUM cuối)
    // vẫn qua được test miễn chuỗi đúng khuôn (Review Findings Group B).
    use sha2::{Digest, Sha256};
    let bytes = std::fs::read(&out).unwrap();
    let mut hasher = Sha256::new();
    hasher.update(&bytes);
    let real_hash: String = hasher.finalize().iter().map(|b| format!("{b:02x}")).collect();
    assert_eq!(report.sha256, real_hash, "reported SHA-256 must match the real file's actual content");
}
