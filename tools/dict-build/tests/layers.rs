//! Nghiệm thu tích hợp — Task 9 của Story 1.10. Dựng CẢ BA tệp (`dict-core.db` +
//! `dict-thieu-chuu.db` + `dict-vietphrase.db`) từ FIXTURE thật vào một thư mục tạm qua
//! `build::run_all`, rồi kiểm AC1/AC2/AC3/AC4 trên kết quả THẬT.
//!
//! ⚠️ Test parity `sqlite_master` chạy trên FIXTURE, ⛔ không trên tệp thật — phải xanh
//! trên runner CI không có byte dữ liệu từ điển nào (Testing standards).

use std::path::PathBuf;

use dict_build::build;
use rusqlite::Connection;

fn fixtures_raw_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/raw")
}

fn build_all_fixture_dbs() -> (tempfile::TempDir, PathBuf, build::AllLayersReport) {
    let dir = tempfile::tempdir().unwrap();
    let out_dir = dir.path().join("out");
    let report = build::run_all(&fixtures_raw_dir(), &out_dir).expect("all-layers fixture build should succeed");
    (dir, out_dir, report)
}

fn sqlite_master_signature(conn: &Connection) -> Vec<String> {
    let mut stmt = conn
        .prepare("SELECT type, name, tbl_name, sql FROM sqlite_master ORDER BY type, name")
        .unwrap();
    stmt.query_map([], |r| {
        let ty: String = r.get(0)?;
        let name: String = r.get(1)?;
        let tbl_name: String = r.get(2)?;
        let sql: Option<String> = r.get(3)?;
        Ok(format!("{ty}|{name}|{tbl_name}|{}", sql.unwrap_or_default()))
    })
    .unwrap()
    .collect::<Result<_, _>>()
    .unwrap()
}

fn user_version(conn: &Connection) -> i64 {
    conn.query_row("PRAGMA user_version", [], |r| r.get(0)).unwrap()
}

/// 🔴 AC4 — vì đường tra cứu chưa tồn tại (1.11/1.13), AC này nghiệm thu ở TẦNG CẤU
/// TRÚC: `sqlite_master` (kèm `PRAGMA user_version`) của cả ba tệp phải GIỐNG NHAU TỪNG
/// KÝ TỰ. ⛔ Không so `dict_meta` — `built_at` khác nhau theo thiết kế.
#[test]
fn sqlite_master_is_byte_identical_across_all_outputs() {
    let (_dir, out_dir, report) = build_all_fixture_dbs();

    let base_conn = Connection::open(out_dir.join(&report.base.0)).unwrap();
    let base_sig = sqlite_master_signature(&base_conn);
    let base_uv = user_version(&base_conn);

    assert_eq!(report.detachable.len(), 3, "expected exactly three detachable layers built");
    for (name, _) in &report.detachable {
        let conn = Connection::open(out_dir.join(name)).unwrap();
        assert_eq!(
            sqlite_master_signature(&conn),
            base_sig,
            "sqlite_master of '{name}' must be byte-identical to dict-core.db"
        );
        assert_eq!(
            user_version(&conn),
            base_uv,
            "PRAGMA user_version of '{name}' must match dict-core.db"
        );
    }
}

/// AC1 mệnh đề 1: mỗi tệp lớp gỡ rời chứa ĐÚNG MỘT hàng `dict_source`, mang ĐÚNG mã của
/// chính nó.
#[test]
fn each_detachable_file_holds_exactly_one_dict_source_row_with_its_own_code() {
    let (_dir, out_dir, report) = build_all_fixture_dbs();
    for (name, expected_code) in [
        (build::output_file_name("thieu-chuu"), "thieu-chuu"),
        (build::output_file_name("vietphrase"), "vietphrase"),
        (build::output_file_name("tran-van-chanh"), "tran-van-chanh"),
    ] {
        assert!(report.detachable.iter().any(|(n, _)| n == &name));
        let conn = Connection::open(out_dir.join(&name)).unwrap();
        let codes: Vec<String> = conn
            .prepare("SELECT code FROM dict_source")
            .unwrap()
            .query_map([], |r| r.get(0))
            .unwrap()
            .collect::<Result<_, _>>()
            .unwrap();
        assert_eq!(codes, vec![expected_code.to_string()], "{name} must hold exactly one dict_source row with its own code");
    }
}

/// AC1 đối chứng âm: `dict-core.db` chứa 0 hàng `dict_source` có `code` thuộc mã lớp gỡ
/// rời — một hợp nhất lén giữa base và lớp gỡ rời sẽ bị bắt ở đây.
#[test]
fn dict_core_holds_zero_rows_for_any_detachable_code() {
    let (_dir, out_dir, report) = build_all_fixture_dbs();
    let conn = Connection::open(out_dir.join(&report.base.0)).unwrap();
    let count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM dict_source WHERE code IN ('thieu-chuu', 'vietphrase', 'tran-van-chanh')",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(count, 0, "dict-core.db must hold zero dict_source rows for any detachable code");
}

/// AC1 đối chứng âm (chiều còn lại): một tệp lớp gỡ rời không được chứa hàng của LỚP GỠ
/// RỜI CÒN LẠI — cách ly TỪNG lớp, không chỉ cách ly với base.
#[test]
fn detachable_files_do_not_contain_each_others_rows() {
    let (_dir, out_dir, report) = build_all_fixture_dbs();
    let thieu_chuu_path = out_dir.join(build::output_file_name("thieu-chuu"));
    let vietphrase_path = out_dir.join(build::output_file_name("vietphrase"));
    let _ = &report;

    let tc_conn = Connection::open(&thieu_chuu_path).unwrap();
    let has_vietphrase: i64 = tc_conn
        .query_row("SELECT COUNT(*) FROM dict_source WHERE code = 'vietphrase'", [], |r| r.get(0))
        .unwrap();
    assert_eq!(has_vietphrase, 0);

    let vp_conn = Connection::open(&vietphrase_path).unwrap();
    let has_thieu_chuu: i64 = vp_conn
        .query_row("SELECT COUNT(*) FROM dict_source WHERE code = 'thieu-chuu'", [], |r| r.get(0))
        .unwrap();
    assert_eq!(has_thieu_chuu, 0);

    // Story 1.10c — lớp gỡ rời thứ ba, cùng phép kiểm cách ly cả hai chiều.
    let tvc_path = out_dir.join(build::output_file_name("tran-van-chanh"));
    let tvc_conn = Connection::open(&tvc_path).unwrap();
    for code in ["thieu-chuu", "vietphrase"] {
        let has: i64 = tvc_conn
            .query_row(
                &format!("SELECT COUNT(*) FROM dict_source WHERE code = '{code}'"),
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(has, 0, "dict-tran-van-chanh.db must not contain {code}'s rows");
    }
    let has_tvc_in_tc: i64 = tc_conn
        .query_row("SELECT COUNT(*) FROM dict_source WHERE code = 'tran-van-chanh'", [], |r| r.get(0))
        .unwrap();
    assert_eq!(has_tvc_in_tc, 0);
    let has_tvc_in_vp: i64 = vp_conn
        .query_row("SELECT COUNT(*) FROM dict_source WHERE code = 'tran-van-chanh'", [], |r| r.get(0))
        .unwrap();
    assert_eq!(has_tvc_in_vp, 0);
}

/// AC2: mỗi tệp lớp gỡ rời tự mang metadata giấy phép/ghi công của chính nó — cả bốn
/// trường khác rỗng, và `license_text` là văn bản thật (đủ dài để không phải placeholder).
#[test]
fn each_detachable_source_declares_non_empty_license_text_and_attribution() {
    let (_dir, out_dir, report) = build_all_fixture_dbs();
    for (name, _) in &report.detachable {
        let conn = Connection::open(out_dir.join(name)).unwrap();
        let (license_kind, license_text, attribution, source_url): (String, String, String, String) = conn
            .query_row(
                "SELECT license_kind, license_text, attribution, source_url FROM dict_source",
                [],
                |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?)),
            )
            .unwrap();
        assert!(!license_kind.is_empty(), "{name}: license_kind is empty");
        assert!(license_text.len() > 200, "{name}: license_text too short to be real legal/declaration text ({} chars)", license_text.len());
        assert!(!attribution.is_empty(), "{name}: attribution is empty");
        assert!(!source_url.is_empty(), "{name}: source_url is empty");
    }
}

/// AC2, nghĩa vụ quyền nhân thân: attribution của Thiều Chửu nêu đích danh tên tác giả,
/// trên DỮ LIỆU THẬT vừa dựng (không chỉ ở tầng khai báo `sources_meta`).
#[test]
fn thieu_chuu_attribution_names_the_author_in_the_built_file() {
    let (_dir, out_dir, report) = build_all_fixture_dbs();
    let name = build::output_file_name("thieu-chuu");
    assert!(report.detachable.iter().any(|(n, _)| n == &name));
    let conn = Connection::open(out_dir.join(&name)).unwrap();
    let attribution: String = conn
        .query_row("SELECT attribution FROM dict_source WHERE code = 'thieu-chuu'", [], |r| r.get(0))
        .unwrap();
    assert!(attribution.contains("Thiều Chửu"));
    assert!(attribution.contains("Nguyễn Hữu Kha"));
}

/// AC3 đối chứng âm — lỗi gán nhãn dễ mắc nhất của story: `vietphrase` phải là `unknown`,
/// ⛔ KHÔNG `public-domain`, trên DỮ LIỆU THẬT vừa dựng.
#[test]
fn vietphrase_is_unknown_not_public_domain_in_the_built_file() {
    let (_dir, out_dir, report) = build_all_fixture_dbs();
    let name = build::output_file_name("vietphrase");
    assert!(report.detachable.iter().any(|(n, _)| n == &name));
    let conn = Connection::open(out_dir.join(&name)).unwrap();
    let (license_kind, license_id): (String, Option<String>) = conn
        .query_row(
            "SELECT license_kind, license_id FROM dict_source WHERE code = 'vietphrase'",
            [],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .unwrap();
    assert_eq!(license_kind, "unknown");
    assert_ne!(license_kind, "public-domain");
    // 🔴 AC3 chốt cứng `NULL`, ⛔ không phải chuỗi rỗng. Mệnh đề `||` cũ chấp nhận cả
    // hai, tức nó xanh cho đúng giá trị mà AC cấm (Review Findings 1.10).
    assert_eq!(license_id, None, "AC3: license_id của vietphrase phải là NULL");
}

/// AC3 — trường `license_kind` PHẢI giữ nguyên kiểu TEXT (chuỗi mở): chèn thẳng một hàng
/// với `license_kind = 'author-grant'` (giá trị HVTĐTD sẽ dùng ở story nối tiếp) và
/// khẳng định THÀNH CÔNG — cách duy nhất chứng minh "biểu diễn được phép riêng của tác
/// giả" khi HVTĐTD chưa có mặt (Testing standards).
#[test]
fn license_kind_column_accepts_a_value_outside_the_open_license_set() {
    let conn = Connection::open_in_memory().unwrap();
    dict_build::insert::create_schema(&conn).unwrap();
    let result = conn.execute(
        "INSERT INTO dict_source (code, display_name, license_kind, license_id, license_text, attribution, source_version, source_url)
         VALUES ('hvtdtd-test', 'HVTĐTD (test)', 'author-grant', NULL, 'placeholder', 'test', 'test', 'https://example.invalid')",
        [],
    );
    assert!(result.is_ok(), "license_kind must accept a value outside the open-license enum: {result:?}");
}

/// §Bẫy 2: một trong hai đường dựng lớp gỡ rời bỏ sót `journal_mode = DELETE` — chạy cho
/// CẢ HAI lớp, không chỉ một.
#[test]
fn every_layer_uses_delete_journal_mode_with_no_wal_artifacts() {
    let (_dir, out_dir, report) = build_all_fixture_dbs();

    let base_path = out_dir.join(&report.base.0);
    assert_eq!(report.base.1.journal_mode.to_lowercase(), "delete");
    assert!(!dict_build::finalize::sibling_path(&base_path, "-wal").exists());
    assert!(!dict_build::finalize::sibling_path(&base_path, "-shm").exists());

    for (name, r) in &report.detachable {
        let path = out_dir.join(name);
        assert_eq!(r.journal_mode.to_lowercase(), "delete", "{name}: journal_mode must be DELETE");
        assert!(!dict_build::finalize::sibling_path(&path, "-wal").exists(), "{name}: leftover -wal file");
        assert!(!dict_build::finalize::sibling_path(&path, "-shm").exists(), "{name}: leftover -shm file");
    }
}

/// 🔴 Bẫy 3 cưỡng chế ở tầng dữ liệu: đường dựng lớp gỡ rời không bao giờ mở
/// `dict-core.db` — nếu nó có, một chữ Hán trùng giữa base và lớp gỡ rời sẽ cho ra bản
/// ghi RIÊNG ở mỗi tệp, ⛔ không bị lọc trùng xuyên tệp (AD-19: không hợp nhất nguồn).
#[test]
fn a_headword_shared_with_base_still_gets_its_own_row_in_the_detachable_file() {
    let (_dir, out_dir, report) = build_all_fixture_dbs();
    // "山" (núi) có mặt ở CẢ fixture CVDICT/CC-CEDICT (lớp nền, dòng thật đã dùng bởi
    // ac2_shan_appears_under_two_different_source_ids_not_merged của tests/parse.rs)
    // LẪN fixture Thiều Chửu (dòng 1790 thật của TudienThienChuu.txt).
    let base_conn = Connection::open(out_dir.join(&report.base.0)).unwrap();
    let base_count: i64 = base_conn
        .query_row("SELECT COUNT(*) FROM dict_entry WHERE headword = '山'", [], |r| r.get(0))
        .unwrap();
    assert!(base_count > 0, "fixture setup assumption: '山' must exist in the base fixture");

    let tc_conn = Connection::open(out_dir.join(build::output_file_name("thieu-chuu"))).unwrap();
    let tc_count: i64 = tc_conn
        .query_row("SELECT COUNT(*) FROM dict_entry WHERE headword = '山'", [], |r| r.get(0))
        .unwrap();
    assert!(tc_count > 0, "'山' must have its own row in dict-thieu-chuu.db, not be filtered out by a base lookup");
}

/// 🔴 §Bẫy 7 — `--layer all` hỏng khi BẤT KỲ lớp nào thiếu nguồn thô. ⛔ Không có chế độ
/// "bỏ qua lớp thiếu nguồn": nó sống sót vào lúc phát hành và cho ra một bản cài thiếu
/// một lớp với lượt build XANH. Lời hứa này trước đây ⛔ không có test nào.
#[test]
fn run_all_fails_when_any_layer_is_missing_its_raw_source() {
    let dir = tempfile::tempdir().unwrap();
    let raw_dir = dir.path().join("raw");
    copy_dir_all(&fixtures_raw_dir(), &raw_dir);
    std::fs::remove_dir_all(raw_dir.join("vietphrase")).unwrap();

    let err = build::run_all(&raw_dir, &dir.path().join("out"))
        .expect_err("--layer all phải HỎNG khi thiếu nguồn thô của một lớp");
    let msg = err.to_string();
    assert!(msg.contains("vietphrase"), "thông điệp lỗi phải nêu đích danh lớp thiếu: {msg}");
}

/// 🔴 Một lượt `--layer all` hỏng giữa chừng ⛔ KHÔNG được đụng tới tệp `.db` của lượt
/// trước. `prepare_fresh_output` xoá tệp đích TRƯỚC khi mở nguồn thô, nên nếu ⛔ không
/// kiểm nguồn từ đầu, out-dir còn lại một bộ KHÔNG đầy đủ và trộn thế hệ — trong khi
/// §Quyết định #6 đòi ba tệp thuộc MỘT thế hệ dữ liệu.
#[test]
fn a_failed_run_all_leaves_the_previous_output_untouched() {
    let dir = tempfile::tempdir().unwrap();
    let raw_dir = dir.path().join("raw");
    let out_dir = dir.path().join("out");
    copy_dir_all(&fixtures_raw_dir(), &raw_dir);

    let first = build::run_all(&raw_dir, &out_dir).expect("lượt dựng đầu phải thành công");
    let mut before: Vec<(String, u64)> = Vec::new();
    for name in std::iter::once(first.base.0.clone()).chain(first.detachable.iter().map(|(n, _)| n.clone())) {
        let len = std::fs::metadata(out_dir.join(&name)).unwrap().len();
        before.push((name, len));
    }
    assert_eq!(before.len(), 4);

    std::fs::remove_dir_all(raw_dir.join("vietphrase")).unwrap();
    assert!(build::run_all(&raw_dir, &out_dir).is_err());

    for (name, len) in &before {
        let path = out_dir.join(name);
        assert!(path.exists(), "'{name}' bị XOÁ bởi một lượt build hỏng");
        assert_eq!(&std::fs::metadata(&path).unwrap().len(), len, "'{name}' bị ghi đè bởi một lượt build hỏng");
    }
    assert!(
        !out_dir.join(format!("{}.tmp", build::output_file_name("vietphrase"))).exists(),
        "một lượt hỏng ⛔ không được để lại tệp .tmp mồ côi"
    );
}

/// 🔴 AD-25 — cùng một cây nguồn thô ⇒ cùng một SHA-256. Trước đây `dict_meta('built_at')`
/// lấy từ `strftime('now')` với độ phân giải mili-giây, nên hai lượt build liên tiếp từ
/// CÙNG một fixture ra hai tệp khác byte: mọi giá trị `sha256` trong `dict-manifest.toml`
/// chỉ đúng cho đúng một lượt chạy, và ⛔ không cổng nào bắt được.
#[test]
fn two_builds_from_the_same_raw_tree_produce_identical_checksums() {
    let raw = fixtures_raw_dir();
    let dir_a = tempfile::tempdir().unwrap();
    let dir_b = tempfile::tempdir().unwrap();

    let a = build::run_all(&raw, &dir_a.path().join("out")).unwrap();
    let b = build::run_all(&raw, &dir_b.path().join("out")).unwrap();

    assert_eq!(a.base.1.sha256, b.base.1.sha256, "dict-core.db phải tái lập được");
    assert_eq!(a.detachable.len(), b.detachable.len());
    for ((name_a, ra), (name_b, rb)) in a.detachable.iter().zip(b.detachable.iter()) {
        assert_eq!(name_a, name_b);
        assert_eq!(ra.sha256, rb.sha256, "'{name_a}' phải tái lập được");
    }
}

fn copy_dir_all(src: &std::path::Path, dst: &std::path::Path) {
    std::fs::create_dir_all(dst).unwrap();
    for entry in std::fs::read_dir(src).unwrap() {
        let entry = entry.unwrap();
        let to = dst.join(entry.file_name());
        if entry.file_type().unwrap().is_dir() {
            copy_dir_all(&entry.path(), &to);
        } else {
            std::fs::copy(entry.path(), &to).unwrap();
        }
    }
}
