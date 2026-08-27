//! Hành vi của chỉ mục Library dẫn xuất — Story 5.2, toàn bộ §I/O Matrix.
//!
//! ⚠️ Tệp riêng có chủ ý, đúng khuôn `store_contract.rs`/`project_contract.rs`: đây là
//! **hành vi lúc chạy**; ranh giới cây nguồn của AC2 nằm ở `library_index_boundary.rs`.
//!
//! ─────────────────────────────────────────────────────────────────────────────
//! BỐN LUẬT CỦA TỆP NÀY — cùng khuôn `store_contract.rs:8-21`
//! ─────────────────────────────────────────────────────────────────────────────
//! 1. Mỗi ca một thư mục tạm riêng (pid + `AtomicU64`). **Không thêm `tempfile`** —
//!    §Boundaries của story cấm tường minh, và khuôn dưới đây chép nguyên
//!    `store_contract.rs:51-69`.
//! 2. `Indexer`/`Store` **drop TRƯỚC** khi xoá thư mục — Windows từ chối xoá tệp đang mở.
//! 3. Không đo thời gian bằng `sleep` dài — không ca nào ở đây cần checkpoint/timing.
//! 4. Không ca nào treo khi nó trượt — mọi thao tác ở đây có trần (I/O đồng bộ, không kênh
//!    nào chờ).
//!
//! ─────────────────────────────────────────────────────────────────────────────
//! 🔴 MỌI FIXTURE `.atproj` MANG `project.db` **RÁC** — VÀ ĐÓ LÀ MỘT PHÉP ĐO, KHÔNG PHẢI SƠ SUẤT
//! ─────────────────────────────────────────────────────────────────────────────
//! [`write_atproj`] ghi và văn bản KHÔNG PHẢI SQLite hợp lệ vào `project.db`. AD-9 nói
//! `Indexer` **chỉ đọc `meta.json`**, không bao giờ mở `project.db` — nếu bất kỳ đường nào của
//! `Indexer` lỡ mở tệp đó, con trỏ vào một payload rác sẽ làm CHÍNH XÁC ca đó panic ngay lập
//! tức (không phải một `assert` phải nhớ viết riêng). Mọi ca dưới đây, không chỉ một ca được
//! đặt tên cho AD-9, đều là đối chứng của bất biến đó.

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use auratranslate_lib::core::library::indexer::{IndexError, Indexer};
use auratranslate_lib::core::library::meta::{META_SCHEMA_VERSION, WorkMeta};
use auratranslate_lib::core::store::{Store, StoreSpec, Transaction};

// ═════════════════════════════════════════════════════════════════════════════════
// Hạ tầng dùng chung
// ═════════════════════════════════════════════════════════════════════════════════

static NEXT_DIR: AtomicU64 = AtomicU64::new(0);

/// Một thư mục tạm **của riêng ca này**. Xem luật 1 ở doc-comment đầu tệp.
fn temp_dir(tag: &str) -> PathBuf {
    let n = NEXT_DIR.fetch_add(1, Ordering::Relaxed);
    let dir = std::env::temp_dir().join(format!(
        "auratranslate-library-index-{}-{}-{}",
        std::process::id(),
        tag,
        n
    ));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap_or_else(|e| panic!("tạo {}: {e}", dir.display()));
    dir
}

/// ⚠️ Gọi **sau** khi mọi `Indexer`/`Store` đã drop. Xem luật 2.
fn cleanup(dir: &Path) {
    let _ = fs::remove_dir_all(dir);
}

fn index_path(dir: &Path) -> PathBuf {
    dir.join("library-index.db")
}

fn library_root(dir: &Path) -> PathBuf {
    dir.join("library")
}

fn sidecar(db: &Path, suffix: &str) -> PathBuf {
    let mut raw = db.as_os_str().to_owned();
    raw.push(suffix);
    PathBuf::from(raw)
}

/// Dựng một `<folder>.atproj/` thật dưới `root`: `meta.json` (qua `WorkMeta::write_atomic`,
/// đúng đường ghi sản phẩm) + `project.db` RÁC (xem doc-comment đầu tệp §AD-9). Trả về đường
/// dẫn thư mục.
fn write_atproj(
    root: &Path,
    folder: &str,
    work_id: &str,
    name: &str,
    chapter_count: u32,
) -> PathBuf {
    let dir = root.join(format!("{folder}.atproj"));
    fs::create_dir_all(&dir).unwrap_or_else(|e| panic!("tạo {}: {e}", dir.display()));

    let meta = WorkMeta {
        meta_schema_version: META_SCHEMA_VERSION,
        work_id: work_id.to_owned(),
        name: name.to_owned(),
        source_lang: "en".to_owned(),
        genre: String::new(),
        created_at: "2026-08-01T00:00:00.000Z".to_owned(),
        updated_at: "2026-08-01T00:00:00.000Z".to_owned(),
        chapter_count,
    };
    meta.write_atomic(&dir)
        .unwrap_or_else(|e| panic!("ghi meta.json ở {}: {e}", dir.display()));

    fs::write(dir.join("project.db"), b"not a real sqlite file -- Indexer khong duoc mo tep nay")
        .unwrap_or_else(|e| panic!("ghi project.db giả ở {}: {e}", dir.display()));

    dir
}

/// Ảnh chụp byte của `meta.json` + `project.db` bên trong mỗi thư mục — dùng để đối chứng
/// "`.atproj` không bị chạm" trước/sau một lượt thao tác trên chỉ mục.
fn snapshot_atproj_bytes(dirs: &[&Path]) -> Vec<(PathBuf, Vec<u8>)> {
    let mut out = Vec::new();
    for dir in dirs {
        for name in ["meta.json", "project.db"] {
            let p = dir.join(name);
            let bytes = fs::read(&p).unwrap_or_else(|e| panic!("đọc {}: {e}", p.display()));
            out.push((p, bytes));
        }
    }
    out
}

// ═════════════════════════════════════════════════════════════════════════════════
// §I/O Matrix — "Dựng lại từ đĩa"
// ═════════════════════════════════════════════════════════════════════════════════

#[test]
fn rebuilding_from_disk_indexes_exactly_n_works_matching_meta_json_field_for_field() {
    let dir = temp_dir("n-works");
    let root = library_root(&dir);
    let alpha = write_atproj(&root, "Alpha", "11111111-1111-1111-1111-111111111111", "Alpha", 3);
    let beta = write_atproj(&root, "Beta", "22222222-2222-2222-2222-222222222222", "Beta", 5);
    let gamma = write_atproj(&root, "Gamma", "33333333-3333-3333-3333-333333333333", "Gamma", 0);

    let indexer = Indexer::open(index_path(&dir)).unwrap_or_else(|e| panic!("mở indexer: {e}"));
    let outcome = indexer.rebuild(&root).unwrap_or_else(|e| panic!("rebuild: {e}"));

    assert_eq!(outcome.indexed, 3, "N=3 .atproj hợp lệ phải cho đúng 3 hàng");
    assert!(!outcome.root_missing);
    assert!(outcome.conflicts.is_empty());
    assert!(outcome.skipped.is_empty());

    let mut works = indexer.list_works().unwrap_or_else(|e| panic!("list_works: {e}"));
    works.sort_by(|a, b| a.name.cmp(&b.name));
    assert_eq!(works.len(), 3);

    let expected = [
        (&alpha, "11111111-1111-1111-1111-111111111111", "Alpha", 3u32),
        (&beta, "22222222-2222-2222-2222-222222222222", "Beta", 5u32),
        (&gamma, "33333333-3333-3333-3333-333333333333", "Gamma", 0u32),
    ];
    for (row, (dir, work_id, name, chapter_count)) in works.iter().zip(expected.iter()) {
        assert_eq!(&row.atproj_path, *dir, "atproj_path phải khớp thư mục thật");
        assert_eq!(&row.work_id, work_id);
        assert_eq!(&row.name, name);
        assert_eq!(row.source_lang, "en");
        assert_eq!(row.genre, "");
        assert_eq!(row.created_at, "2026-08-01T00:00:00.000Z");
        assert_eq!(row.updated_at, "2026-08-01T00:00:00.000Z");
        assert_eq!(row.chapter_count, *chapter_count);
    }

    drop(indexer);
    cleanup(&dir);
}

// ═════════════════════════════════════════════════════════════════════════════════
// §I/O Matrix — "Chỉ mục vắng mặt"
// ═════════════════════════════════════════════════════════════════════════════════

#[test]
fn deleting_the_index_file_then_reopening_rebuilds_all_rows_without_touching_atproj_bytes() {
    let dir = temp_dir("missing-index");
    let root = library_root(&dir);
    let one = write_atproj(&root, "One", "id-one", "One", 1);
    let two = write_atproj(&root, "Two", "id-two", "Two", 2);

    {
        let indexer = Indexer::open(index_path(&dir)).unwrap_or_else(|e| panic!("mở indexer: {e}"));
        let outcome = indexer.rebuild(&root).unwrap_or_else(|e| panic!("rebuild: {e}"));
        assert_eq!(outcome.indexed, 2);
        // ⚠️ Drop TRƯỚC khi xoá tệp bằng tay ngay dưới — luật 2.
    }

    let before = snapshot_atproj_bytes(&[&one, &two]);

    // "Xoá $APPDATA/library-index.db bằng tay" — đúng §Manual checks của story.
    let idx = index_path(&dir);
    let _ = fs::remove_file(&idx);
    let _ = fs::remove_file(sidecar(&idx, "-wal"));
    let _ = fs::remove_file(sidecar(&idx, "-shm"));
    assert!(!idx.exists());

    let indexer = Indexer::open(index_path(&dir)).unwrap_or_else(|e| panic!("mở lại indexer: {e}"));
    let outcome = indexer.rebuild(&root).unwrap_or_else(|e| panic!("rebuild lại: {e}"));
    assert_eq!(outcome.indexed, 2, "dựng lại phải cho đủ N hàng như trước");

    let works = indexer.list_works().unwrap_or_else(|e| panic!("list_works: {e}"));
    assert_eq!(works.len(), 2);

    drop(indexer);

    let after = snapshot_atproj_bytes(&[&one, &two]);
    assert_eq!(
        before, after,
        "`.atproj` bị CHẠM trong lượt xoá-rồi-dựng-lại chỉ mục — AD-9 (Indexer chỉ đọc \
         meta.json, không bao giờ mở project.db) đã bị phá"
    );

    cleanup(&dir);
}

// ═════════════════════════════════════════════════════════════════════════════════
// §I/O Matrix — "Lược đồ lệch" (cả hai chiều)
// ═════════════════════════════════════════════════════════════════════════════════

/// Chiều "mới hơn ứng dụng" — hình dạng đối xứng với
/// `store_contract.rs::a_newer_schema_is_refused_without_touching_a_single_byte`, nhưng KẾT
/// LUẬN NGƯỢC LẠI: ở đó AC7 đòi TỪ CHỐI MỞ (AD-30); ở đây AD-8 đòi XOÁ-VÀ-DỰNG-LẠI. Đây chính
/// là "chỗ dễ chép nhầm khuôn nhất trong story" mà §Design Notes cảnh báo.
#[test]
fn a_schema_version_newer_than_supported_is_deleted_and_rebuilt_not_refused() {
    let dir = temp_dir("schema-too-new");
    let root = library_root(&dir);
    write_atproj(&root, "Solo", "id-solo", "Solo", 1);

    let idx = index_path(&dir);
    {
        let conn = rusqlite::Connection::open(&idx).expect("dựng fixture");
        conn.execute_batch(
            "PRAGMA journal_mode = delete;\n\
             CREATE TABLE from_the_future (id INTEGER PRIMARY KEY);\n\
             INSERT INTO from_the_future (id) VALUES (1);\n\
             PRAGMA user_version = 99;",
        )
        .expect("ghi fixture");
    }

    let indexer = Indexer::open(idx.clone()).unwrap_or_else(|e| {
        panic!(
            "library-index.db LỆCH PHIÊN BẢN phải được XOÁ-VÀ-DỰNG-LẠI, không bị TỪ CHỐI MỞ \
             (khác AD-30 của project.db/global.db, đúng AD-8 của kho dẫn xuất): {e}"
        )
    });
    let outcome = indexer.rebuild(&root).unwrap_or_else(|e| panic!("rebuild: {e}"));
    assert_eq!(outcome.indexed, 1);

    let works = indexer.list_works().unwrap_or_else(|e| panic!("list_works: {e}"));
    assert_eq!(works.len(), 1);
    assert_eq!(works[0].work_id, "id-solo");

    drop(indexer);

    // Bảng "từ tương lai" phải biến mất — tệp thật sự đã bị XOÁ, không phải một `ALTER`
    // chạy trên tệp cũ (thứ sẽ giữ nguyên bảng đó).
    {
        let conn = rusqlite::Connection::open(&idx).expect("mở lại để kiểm tra");
        let has_future_table: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE name = 'from_the_future'",
                [],
                |row| row.get(0),
            )
            .unwrap_or(0);
        assert_eq!(
            has_future_table, 0,
            "bảng cũ còn sống sau lượt xoá-và-dựng-lại — tệp KHÔNG bị xoá thật, đây là một \
             ALTER trá hình"
        );
    }

    cleanup(&dir);
}

/// Chiều "cũ hơn ứng dụng" — hôm nay `LIBRARY_INDEX_MIGRATIONS` chỉ có MỘT phiên bản (v1, xem
/// `core/store/schema.rs::LIBRARY_WORK_DDL`), nên `found < target` với một lược đồ THỰC SỰ
/// khác hình dạng chưa tái lập được bằng fixture (sẽ tái lập được khi lần rewrite tiếp theo
/// bump `to_version` lên 2). Ca dưới đây tái lập trạng thái GẦN NHẤT có thật hôm nay:
/// `found = 0` — một tệp SQLite CÓ THẬT nhưng chưa từng chạy bước di trú (mô phỏng một lần
/// sập máy giữa `Store::open` bước 1 và bước 6). `found = 0 != target = 1` vẫn đi qua ĐÚNG
/// nhánh so sánh mà `a_schema_version_newer_than_supported_is_deleted_and_rebuilt_not_refused`
/// canh ở chiều kia — cùng một hàm, hai chiều của cùng một phép so sánh.
#[test]
fn an_index_file_stuck_at_schema_version_zero_is_deleted_and_rebuilt_not_left_half_migrated() {
    let dir = temp_dir("schema-stuck-zero");
    let root = library_root(&dir);
    write_atproj(&root, "Solo", "id-solo", "Solo", 1);

    let idx = index_path(&dir);
    {
        let conn = rusqlite::Connection::open(&idx).expect("dựng fixture");
        conn.execute_batch("PRAGMA journal_mode = delete;")
            .expect("ghi fixture");
    }

    let indexer = Indexer::open(idx).unwrap_or_else(|e| panic!("mở indexer: {e}"));
    let outcome = indexer.rebuild(&root).unwrap_or_else(|e| panic!("rebuild: {e}"));
    assert_eq!(outcome.indexed, 1);

    let works = indexer.list_works().unwrap_or_else(|e| panic!("list_works: {e}"));
    assert_eq!(works.len(), 1);

    drop(indexer);
    cleanup(&dir);
}

// ═════════════════════════════════════════════════════════════════════════════════
// §I/O Matrix — "Trùng `work_id`"
// ═════════════════════════════════════════════════════════════════════════════════

#[test]
fn a_duplicate_work_id_keeps_the_first_entry_and_reports_the_conflict_with_both_paths() {
    let dir = temp_dir("duplicate-work-id");
    let root = library_root(&dir);
    // Tên thư mục quyết định thứ tự quét (đã SẮP trong `Indexer::rebuild`) -- "A" trước "B".
    let first = write_atproj(&root, "A-First", "dup-id", "First Copy", 1);
    let second = write_atproj(&root, "B-Second", "dup-id", "Second Copy", 2);

    let indexer = Indexer::open(index_path(&dir)).unwrap_or_else(|e| panic!("mở indexer: {e}"));
    let outcome = indexer.rebuild(&root).unwrap_or_else(|e| panic!("rebuild: {e}"));

    assert_eq!(outcome.indexed, 1, "chỉ một hàng được GIỮ -- không gộp");
    assert_eq!(outcome.conflicts.len(), 1);
    let conflict = &outcome.conflicts[0];
    assert_eq!(conflict.work_id, "dup-id");
    assert_eq!(conflict.kept_path, first);
    assert_eq!(conflict.duplicate_path, second);

    let works = indexer.list_works().unwrap_or_else(|e| panic!("list_works: {e}"));
    assert_eq!(works.len(), 1);
    assert_eq!(works[0].name, "First Copy", "mục ĐẦU được giữ, không ghi đè bằng mục sau");

    drop(indexer);
    cleanup(&dir);
}

// ═════════════════════════════════════════════════════════════════════════════════
// §I/O Matrix — "`.atproj` hỏng"
// ═════════════════════════════════════════════════════════════════════════════════

#[test]
fn an_atproj_missing_meta_json_is_skipped_with_a_reason_while_others_still_index() {
    let dir = temp_dir("missing-meta");
    let root = library_root(&dir);
    write_atproj(&root, "Good", "id-good", "Good", 1);
    let broken = root.join("Broken.atproj");
    fs::create_dir_all(&broken).unwrap_or_else(|e| panic!("tạo {}: {e}", broken.display()));

    let indexer = Indexer::open(index_path(&dir)).unwrap_or_else(|e| panic!("mở indexer: {e}"));
    let outcome = indexer.rebuild(&root).unwrap_or_else(|e| panic!("rebuild: {e}"));

    assert_eq!(outcome.indexed, 1, "Tác phẩm còn lại vẫn vào chỉ mục");
    assert_eq!(outcome.skipped.len(), 1);
    assert_eq!(outcome.skipped[0].path, broken);
    assert!(
        !outcome.skipped[0].reason.is_empty(),
        "lý do bị bỏ qua phải được GHI LẠI, không rỗng"
    );

    let works = indexer.list_works().unwrap_or_else(|e| panic!("list_works: {e}"));
    assert_eq!(works.len(), 1);
    assert_eq!(works[0].name, "Good");

    drop(indexer);
    cleanup(&dir);
}

#[test]
fn an_atproj_with_unparseable_meta_json_is_skipped_with_a_reason() {
    let dir = temp_dir("bad-json");
    let root = library_root(&dir);
    let broken = root.join("Broken.atproj");
    fs::create_dir_all(&broken).unwrap_or_else(|e| panic!("tạo {}: {e}", broken.display()));
    fs::write(broken.join("meta.json"), b"{ not valid json")
        .unwrap_or_else(|e| panic!("ghi meta.json hỏng: {e}"));

    let indexer = Indexer::open(index_path(&dir)).unwrap_or_else(|e| panic!("mở indexer: {e}"));
    let outcome = indexer.rebuild(&root).unwrap_or_else(|e| panic!("rebuild: {e}"));

    assert_eq!(outcome.indexed, 0);
    assert_eq!(outcome.skipped.len(), 1);
    assert_eq!(outcome.skipped[0].path, broken);
    assert!(!outcome.skipped[0].reason.is_empty());

    drop(indexer);
    cleanup(&dir);
}

// ═════════════════════════════════════════════════════════════════════════════════
// §I/O Matrix — "`meta.json` mới hơn" (cùng nhánh "hỏng" với JSON không phân tích được)
// ═════════════════════════════════════════════════════════════════════════════════

#[test]
fn a_meta_json_newer_than_supported_is_skipped_not_read() {
    let dir = temp_dir("meta-too-new");
    let root = library_root(&dir);
    let newer = root.join("Newer.atproj");
    fs::create_dir_all(&newer).unwrap_or_else(|e| panic!("tạo {}: {e}", newer.display()));
    let raw = format!(
        "{{\"meta_schema_version\":{},\"work_id\":\"id-newer\",\"name\":\"Newer\",\
         \"source_lang\":\"en\",\"genre\":\"\",\"created_at\":\"2026-08-01T00:00:00.000Z\",\
         \"updated_at\":\"2026-08-01T00:00:00.000Z\",\"chapter_count\":1}}",
        META_SCHEMA_VERSION + 1
    );
    fs::write(newer.join("meta.json"), raw).unwrap_or_else(|e| panic!("ghi meta.json: {e}"));

    let indexer = Indexer::open(index_path(&dir)).unwrap_or_else(|e| panic!("mở indexer: {e}"));
    let outcome = indexer.rebuild(&root).unwrap_or_else(|e| panic!("rebuild: {e}"));

    assert_eq!(outcome.indexed, 0, "meta.json mới hơn KHÔNG được đọc bừa");
    assert_eq!(outcome.skipped.len(), 1);
    assert_eq!(outcome.skipped[0].path, newer);
    assert!(
        outcome.skipped[0].reason.to_lowercase().contains("newer"),
        "lý do bỏ qua phải phân biệt được với 'thiếu tệp' -- nhận: {}",
        outcome.skipped[0].reason
    );

    drop(indexer);
    cleanup(&dir);
}

// ═════════════════════════════════════════════════════════════════════════════════
// §I/O Matrix — "Thư mục gốc vắng"
// ═════════════════════════════════════════════════════════════════════════════════

#[test]
fn a_missing_library_root_yields_an_empty_index_with_a_reason_and_creates_no_directory() {
    let dir = temp_dir("missing-root");
    let root = library_root(&dir); // chưa từng tạo

    let indexer = Indexer::open(index_path(&dir)).unwrap_or_else(|e| panic!("mở indexer: {e}"));
    let outcome = indexer.rebuild(&root).unwrap_or_else(|e| panic!("rebuild: {e}"));

    assert_eq!(outcome.indexed, 0);
    assert!(
        outcome.root_missing,
        "rỗng phải CÓ LÝ DO -- phân biệt được với 'đã quét, thật sự rỗng'"
    );
    assert!(!root.exists(), "rebuild KHÔNG được tự tạo thư mục gốc");

    let works = indexer.list_works().unwrap_or_else(|e| panic!("list_works: {e}"));
    assert!(works.is_empty());

    drop(indexer);
    cleanup(&dir);
}

/// Vòng rà ba lớp, P8 — bản trên chỉ dựng một root CHƯA TỪNG tồn tại, nên câu `DELETE FROM
/// library_work` của nhánh `root_missing` (`Indexer::clear_for_missing_root`) chưa bao giờ
/// chạy trên một bảng CÓ HÀNG: xoá một bảng rỗng "thành công" không chứng minh được gì. Ca này
/// dựng chỉ mục N=2 hàng THẬT trước, rồi mới xoá root, để câu `DELETE` đó thật sự có việc để
/// làm.
#[test]
fn a_root_that_existed_with_rows_then_vanishes_leaves_the_index_empty_not_stale() {
    let dir = temp_dir("root-vanishes-with-rows");
    let root = library_root(&dir);
    write_atproj(&root, "One", "id-one", "One", 1);
    write_atproj(&root, "Two", "id-two", "Two", 2);

    let indexer = Indexer::open(index_path(&dir)).unwrap_or_else(|e| panic!("mở indexer: {e}"));
    let first = indexer.rebuild(&root).unwrap_or_else(|e| panic!("rebuild đầu: {e}"));
    assert_eq!(first.indexed, 2, "phải dựng được 2 hàng THẬT trước khi xoá root");
    assert_eq!(
        indexer
            .list_works()
            .unwrap_or_else(|e| panic!("list_works: {e}"))
            .len(),
        2
    );

    fs::remove_dir_all(&root).unwrap_or_else(|e| panic!("xoá root: {e}"));
    assert!(!root.exists());

    let second = indexer.rebuild(&root).unwrap_or_else(|e| panic!("rebuild sau khi xoá root: {e}"));
    assert_eq!(second.indexed, 0);
    assert!(
        second.root_missing,
        "rỗng phải CÓ LÝ DO — phân biệt được với 'đã quét, thật sự rỗng'"
    );

    let works = indexer
        .list_works()
        .unwrap_or_else(|e| panic!("list_works sau khi root biến mất: {e}"));
    assert!(
        works.is_empty(),
        "bảng phải THẬT SỰ được xoá sạch (DELETE chạy trên một bảng CÓ hàng), không còn 2 hàng \
         cũ trôi nổi: {works:?}"
    );

    drop(indexer);
    cleanup(&dir);
}

// ═════════════════════════════════════════════════════════════════════════════════
// §I/O Matrix — "Xoá chỉ mục ≠ mất dữ liệu"
// ═════════════════════════════════════════════════════════════════════════════════

#[test]
fn deleting_the_library_index_file_leaves_global_db_rows_untouched() {
    let dir = temp_dir("global-untouched");
    let global_path = dir.join("global.db");
    let global = Store::open(StoreSpec::global(global_path))
        .unwrap_or_else(|e| panic!("mở global.db: {e}"));
    global
        .write(|tx: &Transaction<'_>| {
            tx.execute(
                "INSERT INTO pinned_entry (source_code, entry_id, headword, gloss, pinned_at) \
                 VALUES (?1, ?2, ?3, ?4, strftime('%Y-%m-%dT%H:%M:%fZ','now'))",
                ("cvdict", 1i64, "test-headword", Option::<&str>::None),
            )?;
            Ok(())
        })
        .unwrap_or_else(|e| panic!("ghi pinned_entry: {e}"));

    let root = library_root(&dir);
    write_atproj(&root, "Work", "id-work", "Work", 1);
    {
        let indexer = Indexer::open(index_path(&dir)).unwrap_or_else(|e| panic!("mở indexer: {e}"));
        indexer
            .rebuild(&root)
            .unwrap_or_else(|e| panic!("rebuild: {e}"));
        // Drop TRƯỚC khi xoá tệp -- luật 2.
    }

    let idx = index_path(&dir);
    fs::remove_file(&idx).expect("xoá library-index.db bằng tay");
    assert!(!idx.exists());

    let count: i64 = global
        .read(|conn| conn.query_row("SELECT COUNT(*) FROM pinned_entry", [], |row| row.get(0)))
        .unwrap_or_else(|e| panic!("đọc pinned_entry: {e}"));
    assert_eq!(
        count, 1,
        "mọi hàng `global.db` phải NGUYÊN VẸN sau khi xoá `library-index.db`"
    );

    drop(global);
    cleanup(&dir);
}

// ═════════════════════════════════════════════════════════════════════════════════
// §I/O Matrix — "Thứ tự ghi"
// ═════════════════════════════════════════════════════════════════════════════════

#[test]
fn index_rows_only_appear_after_meta_json_is_already_on_disk() {
    let dir = temp_dir("write-order");
    let root = library_root(&dir);
    let work_dir = root.join("Work.atproj");
    fs::create_dir_all(&work_dir).unwrap_or_else(|e| panic!("tạo {}: {e}", work_dir.display()));
    // `project.db` "ghi trước" (giả lập), nhưng `meta.json` CHƯA CÓ -- `.atproj` chưa đầy đủ.
    fs::write(work_dir.join("project.db"), b"not a real sqlite file")
        .unwrap_or_else(|e| panic!("ghi project.db giả: {e}"));

    let indexer = Indexer::open(index_path(&dir)).unwrap_or_else(|e| panic!("mở indexer: {e}"));
    let before = indexer.rebuild(&root).unwrap_or_else(|e| panic!("rebuild: {e}"));
    assert_eq!(before.indexed, 0, "chưa có meta.json thì chưa vào chỉ mục");
    assert_eq!(before.skipped.len(), 1);

    // `meta.json` GHI SAU -- đúng khuôn AD-8 "`.atproj` ghi trước, chỉ mục ghi sau".
    let meta = WorkMeta {
        meta_schema_version: META_SCHEMA_VERSION,
        work_id: "id-order".to_owned(),
        name: "Work".to_owned(),
        source_lang: "en".to_owned(),
        genre: String::new(),
        created_at: "2026-08-01T00:00:00.000Z".to_owned(),
        updated_at: "2026-08-01T00:00:00.000Z".to_owned(),
        chapter_count: 1,
    };
    meta.write_atomic(&work_dir)
        .unwrap_or_else(|e| panic!("ghi meta.json: {e}"));

    let after = indexer.rebuild(&root).unwrap_or_else(|e| panic!("rebuild lại: {e}"));
    assert_eq!(
        after.indexed, 1,
        "sau khi meta.json đã trên đĩa, lượt rebuild SAU phải thấy nó"
    );

    drop(indexer);
    cleanup(&dir);
}

// ═════════════════════════════════════════════════════════════════════════════════
// Vòng rà ba lớp — P1 [CAO]: tệp chỉ mục KHÔNG ĐỌC ĐƯỢC (rác/hỏng) không tự lành
// ═════════════════════════════════════════════════════════════════════════════════

/// AD-8: kho DẪN XUẤT thì xoá luôn là thao tác AN TOÀN — kể cả khi lý do "cần xoá" là
/// "không đọc được" (đĩa hỏng nửa chừng, byte rác ghi đè, …), không chỉ "lệch phiên bản". Trước
/// bản vá của vòng rà ba lớp, `peek_schema_version` trả `Err` cho một tệp không phải SQLite hợp
/// lệ, `?` đẩy lỗi đó xuyên suốt `delete_if_schema_version_differs` → `Indexer::open`, và
/// `lib.rs::open_library_index` (nhánh "mở trượt ⇒ log rồi ĐI TIẾP") không bao giờ quản lý
/// `Indexer` — Library RỖNG VĨNH VIỄN mà không một dòng nào nói vì sao, cho tới khi người dùng
/// tự tay xoá tệp. Đúng lớp "rỗng im lặng" mà `AGENTS.md::Known pitfalls` gọi tên.
#[test]
fn an_unreadable_index_file_is_deleted_and_rebuilt_not_left_broken_forever() {
    let dir = temp_dir("unreadable-index");
    let root = library_root(&dir);
    write_atproj(&root, "Solo", "id-solo", "Solo", 1);

    let idx = index_path(&dir);
    fs::write(&idx, b"khong phai mot tep SQLite hop le -- rac thuan tuy, mo phong dia hong")
        .unwrap_or_else(|e| panic!("ghi fixture rác: {e}"));

    let indexer = Indexer::open(idx).unwrap_or_else(|e| {
        panic!(
            "mot tep chi muc RAC/HONG phai duoc XOA-VA-DUNG-LAI (kho dan xuat, AD-8), khong \
             duoc lam Indexer::open that bai VINH VIEN: {e}"
        )
    });

    // `list_works` phải THÀNH CÔNG ngay sau `open` (trước cả `rebuild`) và trả danh sách
    // RỖNG-ĐÃ-QUÉT — bằng chứng lược đồ đã được dựng lại thật, không phải một kho còn treo.
    let works_before_rebuild = indexer
        .list_works()
        .unwrap_or_else(|e| panic!("list_works ngay sau open phải thành công: {e}"));
    assert!(works_before_rebuild.is_empty());

    let outcome = indexer.rebuild(&root).unwrap_or_else(|e| panic!("rebuild: {e}"));
    assert_eq!(outcome.indexed, 1, "sau rebuild, Tác phẩm thật trên đĩa phải vào chỉ mục");

    let works = indexer.list_works().unwrap_or_else(|e| panic!("list_works: {e}"));
    assert_eq!(works.len(), 1);
    assert_eq!(works[0].work_id, "id-solo");

    drop(indexer);
    cleanup(&dir);
}

// ═════════════════════════════════════════════════════════════════════════════════
// Vòng rà ba lớp — P9 [THẤP]: root TỒN TẠI nhưng không đọc được ⇒ `IndexError::Io`, không
// phải rỗng im lặng
// ═════════════════════════════════════════════════════════════════════════════════

/// Root **tồn tại** (`Path::exists()` đúng) nhưng KHÔNG PHẢI một thư mục ⇒ `read_dir` bên
/// trong `scan_atproj_dirs` trượt với một lỗi KHÁC `NotFound` (không phải nhánh `root_missing`
/// êm ái của P6) — đây là ca `IndexError::Io` chưa từng có bàn đo, và trước bản vá này không
/// gì canh việc nó không lặng lẽ trở thành "chỉ mục rỗng".
///
/// ⚠️ **Thủ thuật TẤT ĐỊNH trên CẢ macOS lẫn Windows**: trỏ `root` vào một TỆP thường thay vì
/// một thư mục. `read_dir` trên một tệp thường luôn lỗi trên mọi nền tảng (không cần quyền
/// đặc biệt, không cần một hệ tệp lạ) — khác các thủ thuật "bỏ quyền đọc thư mục"
/// (`chmod 000`) vốn KHÔNG hoạt động trên Windows và không tất định trên CI chạy bằng root.
#[test]
fn a_root_that_exists_but_cannot_be_read_as_a_directory_is_a_hard_error_not_a_silent_empty() {
    let dir = temp_dir("root-is-a-file");
    let root = library_root(&dir);
    fs::write(&root, b"day la mot TEP, khong phai thu muc")
        .unwrap_or_else(|e| panic!("ghi fixture: {e}"));
    assert!(root.exists(), "fixture phải tồn tại để bài test có ý nghĩa");
    assert!(!root.is_dir());

    let indexer = Indexer::open(index_path(&dir)).unwrap_or_else(|e| panic!("mở indexer: {e}"));
    let err = indexer
        .rebuild(&root)
        .expect_err("root tồn tại nhưng không phải thư mục PHẢI là lỗi cứng, không phải rỗng im lặng");

    match err {
        IndexError::Io { path, .. } => assert_eq!(path, root),
        other => panic!("kỳ vọng IndexError::Io, nhận {other:?}"),
    }

    drop(indexer);
    cleanup(&dir);
}
