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

/// Vòng rà ba lớp, P8 — bản trên chỉ dựng một root CHƯA TỪNG tồn tại, nên câu đánh dấu mồ côi
/// của nhánh `root_missing` chưa bao giờ chạy trên một bảng CÓ HÀNG. Ca này dựng chỉ mục N=2
/// hàng THẬT trước, rồi mới xoá root, để nhánh đó thật sự có việc để làm.
///
/// 🔵 **ĐỔI NGỮ NGHĨA (Story 5.3) — bản trước khẳng định bảng bị XOÁ SẠCH.** Trước story này,
/// `Indexer::clear_for_missing_root` chạy `DELETE FROM library_work` và ca này khẳng định
/// `list_works()` rỗng làm bằng chứng của điều đó. Nay `root_missing` nghĩa là "tập `.atproj`
/// liệt kê được là rỗng" ⇒ MỌI hàng đang sống thành MỒ CÔI (`Indexer::mark_all_orphaned_for_missing_root`),
/// KHÔNG bị xoá — `list_works()` (chỉ trả hàng sống) vẫn rỗng, đúng như bản cũ mong đợi, nhưng
/// vì một lý do khác hẳn: hai hàng cũ còn NGUYÊN trong bảng, dưới dạng mồ côi, đọc được qua
/// `list_orphans()`. Đây chính là I/O Matrix "Đổi thư mục gốc": chỉ mục nói về THƯ VIỆN, không
/// nói về đĩa.
#[test]
fn a_root_that_existed_with_rows_then_vanishes_leaves_every_row_orphaned_not_deleted() {
    let dir = temp_dir("root-vanishes-with-rows");
    let root = library_root(&dir);
    let one = write_atproj(&root, "One", "id-one", "One", 1);
    let two = write_atproj(&root, "Two", "id-two", "Two", 2);

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
    assert_eq!(second.orphans, 2, "cả hai hàng phải CHUYỂN sang mồ côi ở đúng lượt này");

    let works = indexer
        .list_works()
        .unwrap_or_else(|e| panic!("list_works sau khi root biến mất: {e}"));
    assert!(
        works.is_empty(),
        "list_works() chỉ trả hàng ĐANG SỐNG -- cả hai hàng vừa thành mồ côi, nên nó rỗng: {works:?}"
    );

    let mut orphans = indexer
        .list_orphans()
        .unwrap_or_else(|e| panic!("list_orphans sau khi root biến mất: {e}"));
    orphans.sort_by(|a, b| a.work_id.cmp(&b.work_id));
    assert_eq!(
        orphans.len(),
        2,
        "hai hàng phải CÒN NGUYÊN trong bảng dưới dạng mồ côi -- KHÔNG bị xoá"
    );
    assert!(orphans.iter().all(|o| o.orphaned));
    assert_eq!(orphans[0].work_id, "id-one");
    assert_eq!(orphans[0].atproj_path, one, "đường dẫn CŨ phải giữ nguyên (AC3: nêu rõ nó trỏ tới đâu)");
    assert_eq!(orphans[1].work_id, "id-two");
    assert_eq!(orphans[1].atproj_path, two);

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

// ═════════════════════════════════════════════════════════════════════════════════
// Story 5.3 — "Quét lại thư mục" (FR99). Ngữ nghĩa mồ côi mới: `rebuild` ĐỐI CHIẾU thay vì
// xoá-sạch-ghi-lại. Phủ trọn §I/O Matrix của story: di chuyển trong gốc · xoá/chuyển ra
// ngoài · quay lại · gỡ · gỡ nhầm · hỏng-mà-còn-đó · hai lượt quét chồng nhau.
// ═════════════════════════════════════════════════════════════════════════════════

#[test]
fn moving_an_atproj_within_the_root_updates_the_path_and_keeps_exactly_one_row() {
    let dir = temp_dir("move-within-root");
    let root = library_root(&dir);
    write_atproj(&root, "Old-Name", "id-move", "Work", 1);

    let indexer = Indexer::open(index_path(&dir)).unwrap_or_else(|e| panic!("mở indexer: {e}"));
    indexer.rebuild(&root).unwrap_or_else(|e| panic!("rebuild đầu: {e}"));

    let old_dir = root.join("Old-Name.atproj");
    let new_dir = root.join("New-Name.atproj");
    fs::rename(&old_dir, &new_dir).unwrap_or_else(|e| panic!("đổi tên thư mục: {e}"));

    let outcome = indexer.rebuild(&root).unwrap_or_else(|e| panic!("rebuild sau di chuyển: {e}"));
    assert_eq!(outcome.indexed, 1);
    assert_eq!(outcome.orphans, 0, "di chuyển TRONG gốc không tạo mồ côi nào");

    let works = indexer.list_works().unwrap_or_else(|e| panic!("list_works: {e}"));
    assert_eq!(works.len(), 1, "đúng MỘT hàng, không phải một hàng cũ + một hàng mới");
    assert_eq!(works[0].work_id, "id-move");
    assert_eq!(works[0].atproj_path, new_dir, "atproj_path phải là đường dẫn MỚI");
    assert!(!works[0].orphaned);

    assert!(
        indexer.list_orphans().unwrap_or_else(|e| panic!("list_orphans: {e}")).is_empty()
    );

    drop(indexer);
    cleanup(&dir);
}

#[test]
fn deleting_an_atproj_marks_it_orphaned_and_keeps_the_stale_path() {
    let dir = temp_dir("delete-becomes-orphan");
    let root = library_root(&dir);
    let work_dir = write_atproj(&root, "Gone", "id-gone", "Gone", 1);

    let indexer = Indexer::open(index_path(&dir)).unwrap_or_else(|e| panic!("mở indexer: {e}"));
    indexer.rebuild(&root).unwrap_or_else(|e| panic!("rebuild đầu: {e}"));

    fs::remove_dir_all(&work_dir).unwrap_or_else(|e| panic!("xoá .atproj: {e}"));

    let outcome = indexer.rebuild(&root).unwrap_or_else(|e| panic!("rebuild sau khi xoá: {e}"));
    assert_eq!(outcome.indexed, 0);
    assert_eq!(outcome.orphans, 1);

    assert!(indexer.list_works().unwrap_or_else(|e| panic!("list_works: {e}")).is_empty());

    let orphans = indexer.list_orphans().unwrap_or_else(|e| panic!("list_orphans: {e}"));
    assert_eq!(orphans.len(), 1, "hàng phải Ở LẠI dưới dạng mồ côi, không biến mất");
    assert_eq!(orphans[0].work_id, "id-gone");
    assert_eq!(orphans[0].atproj_path, work_dir, "phải NÊU RÕ nó từng trỏ tới đâu (AC3)");
    assert!(orphans[0].orphaned);

    drop(indexer);
    cleanup(&dir);
}

/// AC3 nguyên văn: *"quét lại RỒI KHỞI ĐỘNG LẠI ứng dụng, then hàng đó vẫn ở đó"* — vế "khởi
/// động lại" là phần mà ca ngay trên KHÔNG chạm tới (nó giữ nguyên MỘT `Indexer` từ đầu tới
/// cuối). Ca này đóng `Indexer` (mô phỏng thoát ứng dụng) rồi MỞ LẠI (mô phỏng khởi động lại)
/// trước khi đọc lại danh sách mồ côi — không dựa vào giả định "SQLite tự bền" mà không đo.
#[test]
fn an_orphan_row_survives_closing_and_reopening_the_indexer() {
    let dir = temp_dir("orphan-survives-restart");
    let root = library_root(&dir);
    let work_dir = write_atproj(&root, "Gone", "id-gone", "Gone", 1);

    {
        let indexer = Indexer::open(index_path(&dir)).unwrap_or_else(|e| panic!("mở indexer: {e}"));
        indexer.rebuild(&root).unwrap_or_else(|e| panic!("rebuild đầu: {e}"));
        fs::remove_dir_all(&work_dir).unwrap_or_else(|e| panic!("xoá .atproj: {e}"));
        let outcome = indexer.rebuild(&root).unwrap_or_else(|e| panic!("rebuild sau khi xoá: {e}"));
        assert_eq!(outcome.orphans, 1);
        // Đóng TRƯỜNG ("thoát ứng dụng") -- luật 2 của tệp này (drop trước khi rời scope).
    }

    // "Khởi động lại ứng dụng" -- mở LẠI đúng tệp `library-index.db`, KHÔNG gọi `rebuild()`
    // lần nào ở đây: AC3 đòi hàng còn đó ngay cả TRƯỚC một lượt quét mới, chỉ từ việc mở lại.
    let reopened = Indexer::open(index_path(&dir)).unwrap_or_else(|e| panic!("mở lại indexer: {e}"));
    let orphans = reopened.list_orphans().unwrap_or_else(|e| panic!("list_orphans sau khi mở lại: {e}"));
    assert_eq!(orphans.len(), 1, "hàng mồ côi phải SỐNG SÓT qua một lượt đóng-rồi-mở-lại");
    assert_eq!(orphans[0].work_id, "id-gone");
    assert_eq!(orphans[0].atproj_path, work_dir, "đường dẫn CŨ vẫn phải còn nguyên sau khởi động lại");
    assert!(orphans[0].orphaned);

    drop(reopened);
    cleanup(&dir);
}

#[test]
fn an_orphan_that_reappears_is_restored_without_a_second_row() {
    let dir = temp_dir("orphan-reappears");
    let root = library_root(&dir);
    let work_dir = write_atproj(&root, "Back", "id-back", "Back", 1);

    let indexer = Indexer::open(index_path(&dir)).unwrap_or_else(|e| panic!("mở indexer: {e}"));
    indexer.rebuild(&root).unwrap_or_else(|e| panic!("rebuild đầu: {e}"));

    fs::remove_dir_all(&work_dir).unwrap_or_else(|e| panic!("xoá .atproj: {e}"));
    let orphaned_outcome = indexer.rebuild(&root).unwrap_or_else(|e| panic!("rebuild mồ côi: {e}"));
    assert_eq!(orphaned_outcome.orphans, 1);
    assert_eq!(indexer.list_orphans().unwrap_or_else(|e| panic!("list_orphans: {e}")).len(), 1);

    // "Copy lại vào gốc" -- cùng `work_id`, viết lại y hệt fixture ban đầu.
    write_atproj(&root, "Back", "id-back", "Back", 1);
    let restored = indexer.rebuild(&root).unwrap_or_else(|e| panic!("rebuild sau khi quay lại: {e}"));
    assert_eq!(restored.indexed, 1);
    assert_eq!(restored.orphans, 0);

    let works = indexer.list_works().unwrap_or_else(|e| panic!("list_works: {e}"));
    assert_eq!(works.len(), 1, "KHÔNG tạo hàng thứ hai");
    assert_eq!(works[0].work_id, "id-back");
    assert!(!works[0].orphaned);
    assert!(indexer.list_orphans().unwrap_or_else(|e| panic!("list_orphans: {e}")).is_empty());

    drop(indexer);
    cleanup(&dir);
}

/// §I/O Matrix "Đổi thư mục gốc" — `Indexer::rebuild` không biết gì về "gốc CŨ"/"gốc MỚI", nó
/// chỉ biết `root: &Path` của LƯỢT GỌI này; đổi thư mục gốc ở tầng cấu hình (`AppConfig`,
/// `commands::project::resolve_library_root`) chỉ đổi GIÁ TRỊ được truyền vào lượt `rebuild`
/// kế tiếp — cùng cơ chế mà ca này tái lập trực tiếp ở tầng `Indexer`, không cần chạm tới
/// `ScopeResolver`/dialog. Các `.atproj` của gốc CŨ vẫn NGUYÊN trên đĩa (test không xoá gì),
/// nhưng gốc MỚI không liệt kê được chúng ⇒ chúng phải thành mồ côi — "chỉ mục nói về THƯ
/// VIỆN, không nói về đĩa" (§Design Notes).
#[test]
fn switching_to_a_different_root_orphans_the_old_roots_rows_without_touching_its_files() {
    let dir = temp_dir("switch-root");
    let old_root = dir.join("old-library");
    let new_root = dir.join("new-library");
    let old_work = write_atproj(&old_root, "OldWork", "id-old", "OldWork", 1);

    let indexer = Indexer::open(index_path(&dir)).unwrap_or_else(|e| panic!("mở indexer: {e}"));
    let first = indexer.rebuild(&old_root).unwrap_or_else(|e| panic!("rebuild trên gốc cũ: {e}"));
    assert_eq!(first.indexed, 1);

    // Người dùng "chọn thư mục D qua hộp thoại" -- `new_root` có một Tác phẩm KHÁC hẳn.
    write_atproj(&new_root, "NewWork", "id-new", "NewWork", 1);
    let second = indexer.rebuild(&new_root).unwrap_or_else(|e| panic!("rebuild trên gốc mới: {e}"));
    assert_eq!(second.indexed, 1, "Tác phẩm của gốc MỚI phải được lập chỉ mục");
    assert_eq!(second.orphans, 1, "hàng của gốc CŨ phải thành mồ côi -- nó không có trong gốc mới");

    let works = indexer.list_works().unwrap_or_else(|e| panic!("list_works: {e}"));
    assert_eq!(works.len(), 1);
    assert_eq!(works[0].work_id, "id-new", "chỉ Tác phẩm của gốc MỚI được coi là đang sống");

    let orphans = indexer.list_orphans().unwrap_or_else(|e| panic!("list_orphans: {e}"));
    assert_eq!(orphans.len(), 1);
    assert_eq!(orphans[0].work_id, "id-old");
    assert_eq!(orphans[0].atproj_path, old_work, "giữ nguyên đường dẫn CŨ");

    // "dù thư mục đó vẫn còn trên đĩa" -- test không xoá `old_root`, khẳng định nó còn nguyên.
    assert!(old_work.is_dir(), "gốc CŨ không được ai đụng vào -- chỉ mục không nói về đĩa");
    assert!(old_work.join("meta.json").is_file());

    drop(indexer);
    cleanup(&dir);
}

/// **THÊM 2026-08-27 (vòng rà bốn lớp, P3)** — kịch bản thứ TƯ của vị từ mồ côi: một đường
/// dẫn bị Tác phẩm KHÁC chiếm phải làm hàng CŨ thành mồ côi, không được để lại như một hàng
/// sống trỏ vào thư mục nay thuộc về ai khác.
///
/// A sống ở `/gốc/Foo.atproj`. Người dùng xoá A rồi copy B (work_id KHÁC hẳn) vào một thư
/// mục CŨNG tên `Foo.atproj`. Trước bản vá P3, `Foo.atproj` vẫn nằm trong tập "đã liệt kê
/// được" nên hàng của A bị coi là "đường dẫn còn đó" ⇒ KHÔNG mồ côi — sai, vì cái đang nằm ở
/// đó là B, không phải A.
#[test]
fn a_path_reclaimed_by_a_different_work_orphans_the_old_occupants_row() {
    let dir = temp_dir("path-reclaimed-by-another-work");
    let root = library_root(&dir);
    write_atproj(&root, "Foo", "id-a", "A", 1);

    let indexer = Indexer::open(index_path(&dir)).unwrap_or_else(|e| panic!("mở indexer: {e}"));
    let first = indexer.rebuild(&root).unwrap_or_else(|e| panic!("rebuild đầu: {e}"));
    assert_eq!(first.indexed, 1);
    assert_eq!(
        indexer.list_works().unwrap_or_else(|e| panic!("list_works: {e}"))[0].work_id,
        "id-a"
    );

    // Xoá A, rồi copy B vào MỘT thư mục CÙNG TÊN (`Foo.atproj`) -- work_id KHÁC hẳn.
    fs::remove_dir_all(root.join("Foo.atproj")).unwrap_or_else(|e| panic!("xoá Foo.atproj: {e}"));
    write_atproj(&root, "Foo", "id-b", "B", 1);

    let second = indexer.rebuild(&root).unwrap_or_else(|e| panic!("rebuild sau khi B chiếm chỗ: {e}"));
    assert_eq!(second.indexed, 1, "chỉ B được lập chỉ mục ở đường dẫn đó");
    assert_eq!(second.orphans, 1, "hàng CŨ của A phải thành mồ côi -- đường dẫn nay thuộc về B, không phải A");

    let works = indexer.list_works().unwrap_or_else(|e| panic!("list_works: {e}"));
    assert_eq!(works.len(), 1);
    assert_eq!(works[0].work_id, "id-b", "chỉ B được coi là đang sống ở đường dẫn đó");

    let orphans = indexer.list_orphans().unwrap_or_else(|e| panic!("list_orphans: {e}"));
    assert_eq!(orphans.len(), 1, "A phải Ở LẠI dưới dạng mồ côi, không bị B che khuất");
    assert_eq!(orphans[0].work_id, "id-a");
    assert!(orphans[0].orphaned);

    drop(indexer);
    cleanup(&dir);
}

#[test]
fn an_atproj_whose_meta_json_breaks_after_being_indexed_is_skipped_not_orphaned() {
    let dir = temp_dir("indexed-then-broken");
    let root = library_root(&dir);
    let work_dir = write_atproj(&root, "Breaks", "id-breaks", "Breaks", 1);

    let indexer = Indexer::open(index_path(&dir)).unwrap_or_else(|e| panic!("mở indexer: {e}"));
    let first = indexer.rebuild(&root).unwrap_or_else(|e| panic!("rebuild đầu: {e}"));
    assert_eq!(first.indexed, 1);

    // `meta.json` hỏng, nhưng thư mục `.atproj` VẪN CÒN ĐÓ -- vế HAI của vị từ mồ côi phải
    // giữ hàng này lại, không đánh dấu mồ côi.
    fs::write(work_dir.join("meta.json"), b"{ khong phai json hop le")
        .unwrap_or_else(|e| panic!("ghi meta.json hỏng: {e}"));

    let second = indexer.rebuild(&root).unwrap_or_else(|e| panic!("rebuild sau khi hỏng: {e}"));
    assert_eq!(second.indexed, 0, "meta.json hỏng thì không đọc được nữa");
    assert_eq!(second.skipped.len(), 1, "phải rơi vào SKIPPED");
    assert_eq!(
        second.orphans, 0,
        "KHÔNG được thành mồ côi -- thư mục còn nằm đó, nói 'nó không còn ở đây' là SAI"
    );

    assert!(
        indexer.list_orphans().unwrap_or_else(|e| panic!("list_orphans: {e}")).is_empty(),
        "không hàng nào được đánh dấu mồ côi ở ca này"
    );

    drop(indexer);
    cleanup(&dir);
}

#[test]
fn forget_orphan_removes_exactly_the_named_row_and_returns_the_rest() {
    let dir = temp_dir("forget-orphan-ok");
    let root = library_root(&dir);
    let alpha_dir = write_atproj(&root, "Alpha", "id-alpha", "Alpha", 1);
    write_atproj(&root, "Beta", "id-beta", "Beta", 1);

    let indexer = Indexer::open(index_path(&dir)).unwrap_or_else(|e| panic!("mở indexer: {e}"));
    indexer.rebuild(&root).unwrap_or_else(|e| panic!("rebuild đầu: {e}"));

    fs::remove_dir_all(&alpha_dir).unwrap_or_else(|e| panic!("xoá Alpha: {e}"));
    fs::remove_dir_all(root.join("Beta.atproj")).unwrap_or_else(|e| panic!("xoá Beta: {e}"));
    indexer.rebuild(&root).unwrap_or_else(|e| panic!("rebuild mồ côi: {e}"));
    assert_eq!(indexer.list_orphans().unwrap_or_else(|e| panic!("list_orphans: {e}")).len(), 2);

    indexer
        .forget_orphan("id-alpha")
        .unwrap_or_else(|e| panic!("forget_orphan phải thành công trên một hàng mồ côi: {e}"));

    let remaining = indexer.list_orphans().unwrap_or_else(|e| panic!("list_orphans: {e}"));
    assert_eq!(remaining.len(), 1, "chỉ hàng vừa gỡ biến mất");
    assert_eq!(remaining[0].work_id, "id-beta");

    drop(indexer);
    cleanup(&dir);
}

#[test]
fn forget_orphan_refuses_a_row_that_is_still_alive() {
    let dir = temp_dir("forget-orphan-refuses-live");
    let root = library_root(&dir);
    write_atproj(&root, "Alive", "id-alive", "Alive", 1);

    let indexer = Indexer::open(index_path(&dir)).unwrap_or_else(|e| panic!("mở indexer: {e}"));
    indexer.rebuild(&root).unwrap_or_else(|e| panic!("rebuild: {e}"));

    let err = indexer
        .forget_orphan("id-alive")
        .expect_err("gỡ một hàng ĐANG SỐNG phải bị từ chối, không im lặng thành công");
    match err {
        IndexError::NotOrphaned { work_id } => assert_eq!(work_id, "id-alive"),
        other => panic!("kỳ vọng IndexError::NotOrphaned, nhận {other:?}"),
    }
    assert_eq!(
        indexer.list_works().unwrap_or_else(|e| panic!("list_works: {e}")).len(),
        1,
        "0 lượt xoá -- hàng phải còn nguyên"
    );

    drop(indexer);
    cleanup(&dir);
}

#[test]
fn forget_orphan_refuses_a_work_id_that_does_not_exist() {
    let dir = temp_dir("forget-orphan-refuses-unknown");
    let root = library_root(&dir);

    let indexer = Indexer::open(index_path(&dir)).unwrap_or_else(|e| panic!("mở indexer: {e}"));
    indexer.rebuild(&root).unwrap_or_else(|e| panic!("rebuild: {e}"));

    let err = indexer
        .forget_orphan("id-khong-ton-tai")
        .expect_err("work_id lạ phải bị từ chối trên CÙNG nhánh với 'gỡ nhầm hàng sống'");
    match err {
        IndexError::NotOrphaned { work_id } => assert_eq!(work_id, "id-khong-ton-tai"),
        other => panic!("kỳ vọng IndexError::NotOrphaned, nhận {other:?}"),
    }

    drop(indexer);
    cleanup(&dir);
}

/// Hai lượt `rebuild` gọi từ HAI LUỒNG gần như đồng thời (khởi động + người dùng bấm) phải
/// NỐI TIẾP, không cho ra một trạng thái trộn — deferred-work.md:8079, chủ Story 5.3.
///
/// ⚠️ **Giới hạn thật của ca này, ghi ra thay vì làm tròn lên:** không có một hook tiêm được
/// độ trễ vào giữa lượt QUÉT ĐĨA và lượt GHI của `Indexer::rebuild` (đó là chỗ interleave
/// thật sự nguy hiểm — xem doc-comment của `Indexer::rebuild_lock`), nên ca này không CHỨNG
/// MINH được rằng thiếu Mutex sẽ tạo ra một ảnh chụp trộn cụ thể. Nó đối chứng điều ĐO ĐƯỢC:
/// hàng chục lượt gọi `rebuild` đồng thời từ nhiều luồng, trên một root ổn định, không bao
/// giờ panic/deadlock và LUÔN hội tụ về đúng trạng thái trên đĩa ở cuối cùng — tức Mutex
/// không làm rơi mất một lượt ghi nào và không có race giữa các lượt ghi.
#[test]
fn two_threads_calling_rebuild_concurrently_converge_to_one_consistent_state() {
    let dir = temp_dir("concurrent-rebuild");
    let root = library_root(&dir);
    write_atproj(&root, "One", "id-one", "One", 1);
    write_atproj(&root, "Two", "id-two", "Two", 2);

    let indexer =
        std::sync::Arc::new(Indexer::open(index_path(&dir)).unwrap_or_else(|e| panic!("mở indexer: {e}")));

    let barrier = std::sync::Arc::new(std::sync::Barrier::new(2));
    let mut handles = Vec::new();
    for _ in 0..2 {
        let indexer = std::sync::Arc::clone(&indexer);
        let root = root.clone();
        let barrier = std::sync::Arc::clone(&barrier);
        handles.push(std::thread::spawn(move || {
            for _ in 0..20 {
                barrier.wait();
                indexer.rebuild(&root).unwrap_or_else(|e| panic!("rebuild trên luồng nền: {e}"));
            }
        }));
    }
    for handle in handles {
        handle.join().expect("một luồng rebuild panic");
    }

    let works = indexer.list_works().unwrap_or_else(|e| panic!("list_works: {e}"));
    assert_eq!(works.len(), 2, "trạng thái cuối phải khớp ĐÚNG những gì còn trên đĩa");
    assert!(indexer.list_orphans().unwrap_or_else(|e| panic!("list_orphans: {e}")).is_empty());

    drop(indexer);
    cleanup(&dir);
}
