//! Hành vi của chỉ mục Library dẫn xuất — Story 5.2, toàn bộ §I/O Matrix.
//!
//! ⚠️ Tệp riêng có chủ ý, đúng khuôn `store_contract.rs`/`project_contract.rs`: đây là
//! **hành vi lúc chạy**; ranh giới cây nguồn của AC2 nằm ở `library_index_boundary.rs`.
//!
//! 🔵 **SỬA (2026-08-27, Story 5.4) — `Indexer::list_works()` đổi CHỮ KÝ, mọi ca gọi nó sửa
//! CƠ HỌC.** Hàm nay nhận `filter: Option<&[LifecycleStatus]>` và trả [`WorksReport { total,
//! works }`] thay vì thẳng `Vec<IndexedWork>` (bộ lọc bốn trạng thái vòng đời tính TRONG SQL,
//! §Approach của story). Mọi ca CŨ ở tệp này gọi `.list_works(WorkQuery::default())` (giữ ĐÚNG hành vi cũ:
//! không lọc) rồi đọc `.works` để lấy lại `Vec<IndexedWork>` — không ca nào đổi Ý NGHĨA, chỉ
//! đổi CÚ PHÁP gọi. Ca MỚI kiểm bộ lọc thật nằm ở cuối tệp.
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

use auratranslate_lib::core::library::indexer::{IndexError, Indexer, WorkQuery, WorkSortKey};
use auratranslate_lib::core::library::meta::{META_SCHEMA_VERSION, WorkMeta};
use auratranslate_lib::core::lifecycle::LifecycleStatus;
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

/// **THÊM (2026-08-27, phán quyết Ice #1)** — `library_orphan` sống ở `global.db`, KHÔNG ở
/// `library-index.db` nữa, nên `Indexer::rebuild`/`forget_orphan`/`list_orphans` đòi một
/// `&Store` toàn cục. Mỗi ca mở CHÍNH XÁC một `global.db` trong `dir` của ca đó (cùng luật 1
/// của tệp này — một thư mục tạm riêng), và giữ nó sống suốt cả ca kể cả khi `Indexer` bị
/// đóng/mở lại để mô phỏng khởi động lại (dữ liệu `global.db` là tệp, nó SỐNG SÓT qua việc
/// đó mà không cần đóng/mở lại chính `Store`).
fn open_global(dir: &Path) -> Store {
    Store::open(StoreSpec::global(dir.join("global.db")))
        .unwrap_or_else(|e| panic!("mở global.db: {e}"))
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
        // 🔵 THÊM (2026-08-27, Story 5.4) — hai trường mới của `WorkMeta`. Đa số ca ở tệp
        // này không kiểm trạng thái vòng đời (chúng kiểm đường dẫn/xung đột/mồ côi) nên một
        // giá trị "vừa suy ra, không ghi đè" là trung tính; ca CẦN kiểm trạng thái/bộ lọc
        // dùng `write_atproj_with_status` riêng, xem cuối tệp.
        status: Some("not_started".to_owned()),
        status_is_override: false,
        // 🔵 THÊM (2026-08-28, Story 5.5) — cùng lý lẽ ngay trên: giá trị trung tính cho các
        // ca không kiểm tiến độ; ca CẦN kiểm tiến độ dùng `write_atproj_with_progress` riêng.
        chapter_done_count: Some(0),
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
    let global = open_global(&dir);
    let root = library_root(&dir);
    let alpha = write_atproj(&root, "Alpha", "11111111-1111-1111-1111-111111111111", "Alpha", 3);
    let beta = write_atproj(&root, "Beta", "22222222-2222-2222-2222-222222222222", "Beta", 5);
    let gamma = write_atproj(&root, "Gamma", "33333333-3333-3333-3333-333333333333", "Gamma", 0);

    let indexer = Indexer::open(index_path(&dir)).unwrap_or_else(|e| panic!("mở indexer: {e}"));
    let outcome = indexer.rebuild(&root, Some(&global)).unwrap_or_else(|e| panic!("rebuild: {e}"));

    assert_eq!(outcome.indexed, 3, "N=3 .atproj hợp lệ phải cho đúng 3 hàng");
    assert!(!outcome.root_missing);
    assert!(outcome.conflicts.is_empty());
    assert!(outcome.skipped.is_empty());

    let mut works = indexer.list_works(WorkQuery::default()).unwrap_or_else(|e| panic!("list_works: {e}")).works;
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
        // 🔵 THÊM (2026-08-28, Story 5.5) — `write_atproj` ghi `chapter_done_count: Some(0)`
        // (giá trị trung tính, xem doc-comment ở đó); đối chứng nó đi trọn xuống `IndexedWork`.
        assert_eq!(row.chapter_done_count, Some(0));
    }

    drop(indexer);
    drop(global);
    cleanup(&dir);
}

// ═════════════════════════════════════════════════════════════════════════════════
// §I/O Matrix — "Chỉ mục vắng mặt"
// ═════════════════════════════════════════════════════════════════════════════════

#[test]
fn deleting_the_index_file_then_reopening_rebuilds_all_rows_without_touching_atproj_bytes() {
    let dir = temp_dir("missing-index");
    let global = open_global(&dir);
    let root = library_root(&dir);
    let one = write_atproj(&root, "One", "id-one", "One", 1);
    let two = write_atproj(&root, "Two", "id-two", "Two", 2);

    {
        let indexer = Indexer::open(index_path(&dir)).unwrap_or_else(|e| panic!("mở indexer: {e}"));
        let outcome = indexer.rebuild(&root, Some(&global)).unwrap_or_else(|e| panic!("rebuild: {e}"));
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
    let outcome = indexer.rebuild(&root, Some(&global)).unwrap_or_else(|e| panic!("rebuild lại: {e}"));
    assert_eq!(outcome.indexed, 2, "dựng lại phải cho đủ N hàng như trước");

    let works = indexer.list_works(WorkQuery::default()).unwrap_or_else(|e| panic!("list_works: {e}")).works;
    assert_eq!(works.len(), 2);

    drop(indexer);

    let after = snapshot_atproj_bytes(&[&one, &two]);
    assert_eq!(
        before, after,
        "`.atproj` bị CHẠM trong lượt xoá-rồi-dựng-lại chỉ mục — AD-9 (Indexer chỉ đọc \
         meta.json, không bao giờ mở project.db) đã bị phá"
    );

    drop(global);
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
    let global = open_global(&dir);
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
    let outcome = indexer.rebuild(&root, Some(&global)).unwrap_or_else(|e| panic!("rebuild: {e}"));
    assert_eq!(outcome.indexed, 1);

    let works = indexer.list_works(WorkQuery::default()).unwrap_or_else(|e| panic!("list_works: {e}")).works;
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

    drop(global);
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
    let global = open_global(&dir);
    let root = library_root(&dir);
    write_atproj(&root, "Solo", "id-solo", "Solo", 1);

    let idx = index_path(&dir);
    {
        let conn = rusqlite::Connection::open(&idx).expect("dựng fixture");
        conn.execute_batch("PRAGMA journal_mode = delete;")
            .expect("ghi fixture");
    }

    let indexer = Indexer::open(idx).unwrap_or_else(|e| panic!("mở indexer: {e}"));
    let outcome = indexer.rebuild(&root, Some(&global)).unwrap_or_else(|e| panic!("rebuild: {e}"));
    assert_eq!(outcome.indexed, 1);

    let works = indexer.list_works(WorkQuery::default()).unwrap_or_else(|e| panic!("list_works: {e}")).works;
    assert_eq!(works.len(), 1);

    drop(indexer);
    drop(global);
    cleanup(&dir);
}

/// **THÊM (2026-08-28, vòng rà thứ hai) — AC nói ĐÍCH DANH số 4, và story này vừa nâng
/// `to_version` 4 → 5.** Hai ca ngay trên đã canh CƠ CHẾ chung (một phiên bản lệch bất kỳ, 99
/// hoặc 0, bị xoá-và-dựng-lại), nhưng không ca nào dựng ĐÚNG hình dạng bảng `library_work` của
/// `to_version` 4 (chín cột, KHÔNG có `chapter_done_count`) rồi đối chứng nó lên ĐÚNG 5 kèm cột
/// mới đọc được. Chép tay DDL cũ ở đây thay vì tái dùng `LIBRARY_WORK_DDL` hiện hành — hằng đó
/// đã mang cột mới, tái dùng nó sẽ dựng một fixture SAI (không mô phỏng được tệp thật của
/// người dùng đang ở `to_version` 4).
///
/// 🔴 Đối chứng "ca này đỏ được": hạ `LIBRARY_INDEX_MIGRATIONS`/`LIBRARY_WORK_DDL` về lại
/// `to_version` 4 (gỡ cột `chapter_done_count`) mà KHÔNG gỡ trường tương ứng ở `IndexedWork` ⇒
/// fixture (đã ở `to_version` 4) khớp target hiện hành nên `Indexer::open` KHÔNG xoá-dựng-lại;
/// `library_work` trên đĩa vẫn thiếu cột, và `list_works` ném "no such column" — ca này FAIL.
#[test]
fn an_index_file_at_schema_version_4_is_deleted_and_rebuilt_at_version_5_with_the_new_column() {
    let dir = temp_dir("schema-v4-progress-column");
    let global = open_global(&dir);
    let root = library_root(&dir);
    write_atproj(&root, "Solo", "id-solo", "Solo", 1);

    let idx = index_path(&dir);
    {
        // Hình dạng THẬT của `to_version` 4 (trước Story 5.5) -- mười cột, KHÔNG
        // `chapter_done_count`. Xem lịch sử `LIBRARY_WORK_DDL` (`core/store/schema.rs`).
        let conn = rusqlite::Connection::open(&idx).expect("dựng fixture");
        conn.execute_batch(
            "PRAGMA journal_mode = delete;\n\
             CREATE TABLE schema_migration_log (\n\
               version     INTEGER PRIMARY KEY,\n\
               applied_at  TEXT NOT NULL,\n\
               app_version TEXT NOT NULL\n\
             );\n\
             CREATE TABLE library_work (\n\
               work_id             TEXT PRIMARY KEY,\n\
               atproj_path         TEXT NOT NULL,\n\
               name                TEXT NOT NULL,\n\
               source_lang         TEXT NOT NULL,\n\
               genre               TEXT NOT NULL,\n\
               created_at          TEXT NOT NULL,\n\
               updated_at          TEXT NOT NULL,\n\
               chapter_count       INTEGER NOT NULL,\n\
               status              TEXT,\n\
               status_is_override  INTEGER NOT NULL DEFAULT 0\n\
             );\n\
             PRAGMA user_version = 4;",
        )
        .expect("ghi fixture hinh dang to_version 4");
    }

    let indexer = Indexer::open(idx.clone()).unwrap_or_else(|e| {
        panic!(
            "library-index.db ở to_version 4 phải được XOÁ-VÀ-DỰNG-LẠI, không bị TỪ CHỐI MỞ \
             (AD-8, kho dẫn xuất): {e}"
        )
    });
    let outcome = indexer.rebuild(&root, Some(&global)).unwrap_or_else(|e| panic!("rebuild: {e}"));
    assert_eq!(outcome.indexed, 1);

    let works = indexer.list_works(WorkQuery::default()).unwrap_or_else(|e| panic!("list_works: {e}")).works;
    assert_eq!(works.len(), 1);
    // Cột MỚI phải đọc được (không "no such column") và mang giá trị THẬT -- Chương duy nhất
    // vừa tạo ở `not_started`, nên `Some(0)`, không phải một giá trị mặc định che dấu việc cột
    // không tồn tại.
    assert_eq!(works[0].chapter_done_count, Some(0));

    drop(indexer);

    // Phiên bản trên đĩa phải là 5 -- đối chứng TRỰC TIẾP bằng PRAGMA, không suy luận từ việc
    // `list_works` không lỗi (một `ALTER` trá hình cũng qua được vế đó).
    {
        let conn = rusqlite::Connection::open(&idx).expect("mở lại để kiểm tra");
        let version: i64 = conn
            .query_row("PRAGMA user_version", [], |row| row.get(0))
            .expect("đọc PRAGMA user_version");
        assert_eq!(version, 5, "library-index.db phải ở đúng to_version 5 sau lượt mở lại");
    }

    drop(global);
    cleanup(&dir);
}

// ═════════════════════════════════════════════════════════════════════════════════
// §I/O Matrix — "Trùng `work_id`"
// ═════════════════════════════════════════════════════════════════════════════════

#[test]
fn a_duplicate_work_id_keeps_the_first_entry_and_reports_the_conflict_with_both_paths() {
    let dir = temp_dir("duplicate-work-id");
    let global = open_global(&dir);
    let root = library_root(&dir);
    // Tên thư mục quyết định thứ tự quét (đã SẮP trong `Indexer::rebuild`) -- "A" trước "B".
    let first = write_atproj(&root, "A-First", "dup-id", "First Copy", 1);
    let second = write_atproj(&root, "B-Second", "dup-id", "Second Copy", 2);

    let indexer = Indexer::open(index_path(&dir)).unwrap_or_else(|e| panic!("mở indexer: {e}"));
    let outcome = indexer.rebuild(&root, Some(&global)).unwrap_or_else(|e| panic!("rebuild: {e}"));

    assert_eq!(outcome.indexed, 1, "chỉ một hàng được GIỮ -- không gộp");
    assert_eq!(outcome.conflicts.len(), 1);
    let conflict = &outcome.conflicts[0];
    assert_eq!(conflict.work_id, "dup-id");
    assert_eq!(conflict.kept_path, first);
    assert_eq!(conflict.duplicate_path, second);

    let works = indexer.list_works(WorkQuery::default()).unwrap_or_else(|e| panic!("list_works: {e}")).works;
    assert_eq!(works.len(), 1);
    assert_eq!(works[0].name, "First Copy", "mục ĐẦU được giữ, không ghi đè bằng mục sau");

    drop(indexer);
    drop(global);
    cleanup(&dir);
}

// ═════════════════════════════════════════════════════════════════════════════════
// §I/O Matrix — "`.atproj` hỏng"
// ═════════════════════════════════════════════════════════════════════════════════

#[test]
fn an_atproj_missing_meta_json_is_skipped_with_a_reason_while_others_still_index() {
    let dir = temp_dir("missing-meta");
    let global = open_global(&dir);
    let root = library_root(&dir);
    write_atproj(&root, "Good", "id-good", "Good", 1);
    let broken = root.join("Broken.atproj");
    fs::create_dir_all(&broken).unwrap_or_else(|e| panic!("tạo {}: {e}", broken.display()));

    let indexer = Indexer::open(index_path(&dir)).unwrap_or_else(|e| panic!("mở indexer: {e}"));
    let outcome = indexer.rebuild(&root, Some(&global)).unwrap_or_else(|e| panic!("rebuild: {e}"));

    assert_eq!(outcome.indexed, 1, "Tác phẩm còn lại vẫn vào chỉ mục");
    assert_eq!(outcome.skipped.len(), 1);
    assert_eq!(outcome.skipped[0].path, broken);
    assert!(
        !outcome.skipped[0].reason.is_empty(),
        "lý do bị bỏ qua phải được GHI LẠI, không rỗng"
    );

    let works = indexer.list_works(WorkQuery::default()).unwrap_or_else(|e| panic!("list_works: {e}")).works;
    assert_eq!(works.len(), 1);
    assert_eq!(works[0].name, "Good");

    drop(indexer);
    drop(global);
    cleanup(&dir);
}

#[test]
fn an_atproj_with_unparseable_meta_json_is_skipped_with_a_reason() {
    let dir = temp_dir("bad-json");
    let global = open_global(&dir);
    let root = library_root(&dir);
    let broken = root.join("Broken.atproj");
    fs::create_dir_all(&broken).unwrap_or_else(|e| panic!("tạo {}: {e}", broken.display()));
    fs::write(broken.join("meta.json"), b"{ not valid json")
        .unwrap_or_else(|e| panic!("ghi meta.json hỏng: {e}"));

    let indexer = Indexer::open(index_path(&dir)).unwrap_or_else(|e| panic!("mở indexer: {e}"));
    let outcome = indexer.rebuild(&root, Some(&global)).unwrap_or_else(|e| panic!("rebuild: {e}"));

    assert_eq!(outcome.indexed, 0);
    assert_eq!(outcome.skipped.len(), 1);
    assert_eq!(outcome.skipped[0].path, broken);
    assert!(!outcome.skipped[0].reason.is_empty());

    drop(indexer);
    drop(global);
    cleanup(&dir);
}

// ═════════════════════════════════════════════════════════════════════════════════
// §I/O Matrix — "`meta.json` mới hơn" (cùng nhánh "hỏng" với JSON không phân tích được)
// ═════════════════════════════════════════════════════════════════════════════════

#[test]
fn a_meta_json_newer_than_supported_is_skipped_not_read() {
    let dir = temp_dir("meta-too-new");
    let global = open_global(&dir);
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
    let outcome = indexer.rebuild(&root, Some(&global)).unwrap_or_else(|e| panic!("rebuild: {e}"));

    assert_eq!(outcome.indexed, 0, "meta.json mới hơn KHÔNG được đọc bừa");
    assert_eq!(outcome.skipped.len(), 1);
    assert_eq!(outcome.skipped[0].path, newer);
    assert!(
        outcome.skipped[0].reason.to_lowercase().contains("newer"),
        "lý do bỏ qua phải phân biệt được với 'thiếu tệp' -- nhận: {}",
        outcome.skipped[0].reason
    );

    drop(indexer);
    drop(global);
    cleanup(&dir);
}

// ═════════════════════════════════════════════════════════════════════════════════
// §I/O Matrix — "Thư mục gốc vắng"
// ═════════════════════════════════════════════════════════════════════════════════

#[test]
fn a_missing_library_root_yields_an_empty_index_with_a_reason_and_creates_no_directory() {
    let dir = temp_dir("missing-root");
    let global = open_global(&dir);
    let root = library_root(&dir); // chưa từng tạo

    let indexer = Indexer::open(index_path(&dir)).unwrap_or_else(|e| panic!("mở indexer: {e}"));
    let outcome = indexer.rebuild(&root, Some(&global)).unwrap_or_else(|e| panic!("rebuild: {e}"));

    assert_eq!(outcome.indexed, 0);
    assert!(
        outcome.root_missing,
        "rỗng phải CÓ LÝ DO -- phân biệt được với 'đã quét, thật sự rỗng'"
    );
    assert!(!root.exists(), "rebuild KHÔNG được tự tạo thư mục gốc");

    let works = indexer.list_works(WorkQuery::default()).unwrap_or_else(|e| panic!("list_works: {e}")).works;
    assert!(works.is_empty());

    drop(indexer);
    drop(global);
    cleanup(&dir);
}

/// Vòng rà ba lớp, P8 — bản trên chỉ dựng một root CHƯA TỪNG tồn tại, nên câu đánh dấu mồ côi
/// của nhánh `root_missing` chưa bao giờ chạy trên một bảng CÓ HÀNG. Ca này dựng chỉ mục N=2
/// hàng THẬT trước, rồi mới xoá root, để nhánh đó thật sự có việc để làm.
///
/// 🔵 **ĐỔI NGỮ NGHĨA (Story 5.3) — bản trước khẳng định bảng bị XOÁ SẠCH.** Trước story này,
/// `Indexer::clear_for_missing_root` chạy `DELETE FROM library_work` và ca này khẳng định
/// `list_works(None)` rỗng làm bằng chứng của điều đó. Nay `root_missing` nghĩa là "tập `.atproj`
/// liệt kê được là rỗng" ⇒ MỌI hàng đang sống thành MỒ CÔI (`Indexer::mark_all_orphaned_for_missing_root`),
/// KHÔNG bị xoá — `list_works(None)` (chỉ trả hàng sống) vẫn rỗng, đúng như bản cũ mong đợi, nhưng
/// vì một lý do khác hẳn: hai hàng cũ còn NGUYÊN trong bảng, dưới dạng mồ côi, đọc được qua
/// `list_orphans()`. Đây chính là I/O Matrix "Đổi thư mục gốc": chỉ mục nói về THƯ VIỆN, không
/// nói về đĩa.
#[test]
fn a_root_that_existed_with_rows_then_vanishes_leaves_every_row_orphaned_not_deleted() {
    let dir = temp_dir("root-vanishes-with-rows");
    let global = open_global(&dir);
    let root = library_root(&dir);
    let one = write_atproj(&root, "One", "id-one", "One", 1);
    let two = write_atproj(&root, "Two", "id-two", "Two", 2);

    let indexer = Indexer::open(index_path(&dir)).unwrap_or_else(|e| panic!("mở indexer: {e}"));
    let first = indexer.rebuild(&root, Some(&global)).unwrap_or_else(|e| panic!("rebuild đầu: {e}"));
    assert_eq!(first.indexed, 2, "phải dựng được 2 hàng THẬT trước khi xoá root");
    assert_eq!(
        indexer
            .list_works(WorkQuery::default())
            .unwrap_or_else(|e| panic!("list_works: {e}")).works
            .len(),
        2
    );

    fs::remove_dir_all(&root).unwrap_or_else(|e| panic!("xoá root: {e}"));
    assert!(!root.exists());

    let second = indexer.rebuild(&root, Some(&global)).unwrap_or_else(|e| panic!("rebuild sau khi xoá root: {e}"));
    assert_eq!(second.indexed, 0);
    assert!(
        second.root_missing,
        "rỗng phải CÓ LÝ DO — phân biệt được với 'đã quét, thật sự rỗng'"
    );
    assert_eq!(second.orphans, 2, "cả hai hàng phải CHUYỂN sang mồ côi ở đúng lượt này");

    let works = indexer
        .list_works(WorkQuery::default())
        .unwrap_or_else(|e| panic!("list_works sau khi root biến mất: {e}")).works;
    assert!(
        works.is_empty(),
        "list_works(None) chỉ trả hàng ĐANG SỐNG -- cả hai hàng vừa thành mồ côi, nên nó rỗng: {works:?}"
    );

    let mut orphans = indexer
        .list_orphans(Some(&global))
        .unwrap_or_else(|e| panic!("list_orphans sau khi root biến mất: {e}"));
    orphans.sort_by(|a, b| a.work_id.cmp(&b.work_id));
    assert_eq!(
        orphans.len(),
        2,
        "hai hàng phải CÒN NGUYÊN trong bảng dưới dạng mồ côi -- KHÔNG bị xoá"
    );
    assert_eq!(orphans[0].work_id, "id-one");
    assert_eq!(orphans[0].atproj_path, one.display().to_string(), "đường dẫn CŨ phải giữ nguyên (AC3: nêu rõ nó trỏ tới đâu)");
    assert_eq!(orphans[1].work_id, "id-two");
    assert_eq!(orphans[1].atproj_path, two.display().to_string());

    drop(indexer);
    drop(global);
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
            .rebuild(&root, Some(&global))
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
    let global = open_global(&dir);
    let root = library_root(&dir);
    let work_dir = root.join("Work.atproj");
    fs::create_dir_all(&work_dir).unwrap_or_else(|e| panic!("tạo {}: {e}", work_dir.display()));
    // `project.db` "ghi trước" (giả lập), nhưng `meta.json` CHƯA CÓ -- `.atproj` chưa đầy đủ.
    fs::write(work_dir.join("project.db"), b"not a real sqlite file")
        .unwrap_or_else(|e| panic!("ghi project.db giả: {e}"));

    let indexer = Indexer::open(index_path(&dir)).unwrap_or_else(|e| panic!("mở indexer: {e}"));
    let before = indexer.rebuild(&root, Some(&global)).unwrap_or_else(|e| panic!("rebuild: {e}"));
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
        // 🔵 THÊM (2026-08-27, Story 5.4) — ca này kiểm THỨ TỰ quét, không trạng thái vòng
        // đời; giá trị trung tính, cùng lý do đã ghi ở `write_atproj`.
        status: Some("not_started".to_owned()),
        status_is_override: false,
        // 🔵 THÊM (2026-08-28, Story 5.5) — cùng lý lẽ ngay trên, ca này không kiểm tiến độ.
        chapter_done_count: Some(0),
    };
    meta.write_atomic(&work_dir)
        .unwrap_or_else(|e| panic!("ghi meta.json: {e}"));

    let after = indexer.rebuild(&root, Some(&global)).unwrap_or_else(|e| panic!("rebuild lại: {e}"));
    assert_eq!(
        after.indexed, 1,
        "sau khi meta.json đã trên đĩa, lượt rebuild SAU phải thấy nó"
    );

    drop(indexer);
    drop(global);
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
    let global = open_global(&dir);
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
        .list_works(WorkQuery::default())
        .unwrap_or_else(|e| panic!("list_works ngay sau open phải thành công: {e}")).works;
    assert!(works_before_rebuild.is_empty());

    let outcome = indexer.rebuild(&root, Some(&global)).unwrap_or_else(|e| panic!("rebuild: {e}"));
    assert_eq!(outcome.indexed, 1, "sau rebuild, Tác phẩm thật trên đĩa phải vào chỉ mục");

    let works = indexer.list_works(WorkQuery::default()).unwrap_or_else(|e| panic!("list_works: {e}")).works;
    assert_eq!(works.len(), 1);
    assert_eq!(works[0].work_id, "id-solo");

    drop(indexer);
    drop(global);
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
    let global = open_global(&dir);
    let root = library_root(&dir);
    fs::write(&root, b"day la mot TEP, khong phai thu muc")
        .unwrap_or_else(|e| panic!("ghi fixture: {e}"));
    assert!(root.exists(), "fixture phải tồn tại để bài test có ý nghĩa");
    assert!(!root.is_dir());

    let indexer = Indexer::open(index_path(&dir)).unwrap_or_else(|e| panic!("mở indexer: {e}"));
    let err = indexer
        .rebuild(&root, Some(&global))
        .expect_err("root tồn tại nhưng không phải thư mục PHẢI là lỗi cứng, không phải rỗng im lặng");

    match err {
        IndexError::Io { path, .. } => assert_eq!(path, root),
        other => panic!("kỳ vọng IndexError::Io, nhận {other:?}"),
    }

    drop(indexer);
    drop(global);
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
    let global = open_global(&dir);
    let root = library_root(&dir);
    write_atproj(&root, "Old-Name", "id-move", "Work", 1);

    let indexer = Indexer::open(index_path(&dir)).unwrap_or_else(|e| panic!("mở indexer: {e}"));
    indexer.rebuild(&root, Some(&global)).unwrap_or_else(|e| panic!("rebuild đầu: {e}"));

    let old_dir = root.join("Old-Name.atproj");
    let new_dir = root.join("New-Name.atproj");
    fs::rename(&old_dir, &new_dir).unwrap_or_else(|e| panic!("đổi tên thư mục: {e}"));

    let outcome = indexer.rebuild(&root, Some(&global)).unwrap_or_else(|e| panic!("rebuild sau di chuyển: {e}"));
    assert_eq!(outcome.indexed, 1);
    assert_eq!(outcome.orphans, 0, "di chuyển TRONG gốc không tạo mồ côi nào");

    let works = indexer.list_works(WorkQuery::default()).unwrap_or_else(|e| panic!("list_works: {e}")).works;
    assert_eq!(works.len(), 1, "đúng MỘT hàng, không phải một hàng cũ + một hàng mới");
    assert_eq!(works[0].work_id, "id-move");
    assert_eq!(works[0].atproj_path, new_dir, "atproj_path phải là đường dẫn MỚI");

    assert!(
        indexer.list_orphans(Some(&global)).unwrap_or_else(|e| panic!("list_orphans: {e}")).is_empty()
    );

    drop(indexer);
    drop(global);
    cleanup(&dir);
}

#[test]
fn deleting_an_atproj_marks_it_orphaned_and_keeps_the_stale_path() {
    let dir = temp_dir("delete-becomes-orphan");
    let global = open_global(&dir);
    let root = library_root(&dir);
    let work_dir = write_atproj(&root, "Gone", "id-gone", "Gone", 1);

    let indexer = Indexer::open(index_path(&dir)).unwrap_or_else(|e| panic!("mở indexer: {e}"));
    indexer.rebuild(&root, Some(&global)).unwrap_or_else(|e| panic!("rebuild đầu: {e}"));

    fs::remove_dir_all(&work_dir).unwrap_or_else(|e| panic!("xoá .atproj: {e}"));

    let outcome = indexer.rebuild(&root, Some(&global)).unwrap_or_else(|e| panic!("rebuild sau khi xoá: {e}"));
    assert_eq!(outcome.indexed, 0);
    assert_eq!(outcome.orphans, 1);

    assert!(indexer.list_works(WorkQuery::default()).unwrap_or_else(|e| panic!("list_works: {e}")).works.is_empty());

    let orphans = indexer.list_orphans(Some(&global)).unwrap_or_else(|e| panic!("list_orphans: {e}"));
    assert_eq!(orphans.len(), 1, "hàng phải Ở LẠI dưới dạng mồ côi, không biến mất");
    assert_eq!(orphans[0].work_id, "id-gone");
    assert_eq!(orphans[0].atproj_path, work_dir.display().to_string(), "phải NÊU RÕ nó từng trỏ tới đâu (AC3)");

    drop(indexer);
    drop(global);
    cleanup(&dir);
}

/// AC3 nguyên văn: *"quét lại RỒI KHỞI ĐỘNG LẠI ứng dụng, then hàng đó vẫn ở đó"* — vế "khởi
/// động lại" là phần mà ca ngay trên KHÔNG chạm tới (nó giữ nguyên MỘT `Indexer` từ đầu tới
/// cuối). Ca này đóng `Indexer` (mô phỏng thoát ứng dụng) rồi MỞ LẠI (mô phỏng khởi động lại)
/// trước khi đọc lại danh sách mồ côi — không dựa vào giả định "SQLite tự bền" mà không đo.
#[test]
fn an_orphan_row_survives_closing_and_reopening_the_indexer() {
    let dir = temp_dir("orphan-survives-restart");
    let global = open_global(&dir);
    let root = library_root(&dir);
    let work_dir = write_atproj(&root, "Gone", "id-gone", "Gone", 1);

    {
        let indexer = Indexer::open(index_path(&dir)).unwrap_or_else(|e| panic!("mở indexer: {e}"));
        indexer.rebuild(&root, Some(&global)).unwrap_or_else(|e| panic!("rebuild đầu: {e}"));
        fs::remove_dir_all(&work_dir).unwrap_or_else(|e| panic!("xoá .atproj: {e}"));
        let outcome = indexer.rebuild(&root, Some(&global)).unwrap_or_else(|e| panic!("rebuild sau khi xoá: {e}"));
        assert_eq!(outcome.orphans, 1);
        // Đóng TRƯỜNG ("thoát ứng dụng") -- luật 2 của tệp này (drop trước khi rời scope).
    }

    // "Khởi động lại ứng dụng" -- mở LẠI đúng tệp `library-index.db`, KHÔNG gọi `rebuild()`
    // lần nào ở đây: AC3 đòi hàng còn đó ngay cả TRƯỚC một lượt quét mới, chỉ từ việc mở lại.
    let reopened = Indexer::open(index_path(&dir)).unwrap_or_else(|e| panic!("mở lại indexer: {e}"));
    let orphans = reopened.list_orphans(Some(&global)).unwrap_or_else(|e| panic!("list_orphans sau khi mở lại: {e}"));
    assert_eq!(orphans.len(), 1, "hàng mồ côi phải SỐNG SÓT qua một lượt đóng-rồi-mở-lại");
    assert_eq!(orphans[0].work_id, "id-gone");
    assert_eq!(orphans[0].atproj_path, work_dir.display().to_string(), "đường dẫn CŨ vẫn phải còn nguyên sau khởi động lại");

    drop(reopened);
    drop(global);
    cleanup(&dir);
}

#[test]
fn an_orphan_that_reappears_is_restored_without_a_second_row() {
    let dir = temp_dir("orphan-reappears");
    let global = open_global(&dir);
    let root = library_root(&dir);
    let work_dir = write_atproj(&root, "Back", "id-back", "Back", 1);

    let indexer = Indexer::open(index_path(&dir)).unwrap_or_else(|e| panic!("mở indexer: {e}"));
    indexer.rebuild(&root, Some(&global)).unwrap_or_else(|e| panic!("rebuild đầu: {e}"));

    fs::remove_dir_all(&work_dir).unwrap_or_else(|e| panic!("xoá .atproj: {e}"));
    let orphaned_outcome = indexer.rebuild(&root, Some(&global)).unwrap_or_else(|e| panic!("rebuild mồ côi: {e}"));
    assert_eq!(orphaned_outcome.orphans, 1);
    assert_eq!(indexer.list_orphans(Some(&global)).unwrap_or_else(|e| panic!("list_orphans: {e}")).len(), 1);

    // "Copy lại vào gốc" -- cùng `work_id`, viết lại y hệt fixture ban đầu.
    write_atproj(&root, "Back", "id-back", "Back", 1);
    let restored = indexer.rebuild(&root, Some(&global)).unwrap_or_else(|e| panic!("rebuild sau khi quay lại: {e}"));
    assert_eq!(restored.indexed, 1);
    assert_eq!(restored.orphans, 0);

    let works = indexer.list_works(WorkQuery::default()).unwrap_or_else(|e| panic!("list_works: {e}")).works;
    assert_eq!(works.len(), 1, "KHÔNG tạo hàng thứ hai");
    assert_eq!(works[0].work_id, "id-back");
    assert!(indexer.list_orphans(Some(&global)).unwrap_or_else(|e| panic!("list_orphans: {e}")).is_empty());

    drop(indexer);
    drop(global);
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
    let global = open_global(&dir);
    let old_root = dir.join("old-library");
    let new_root = dir.join("new-library");
    let old_work = write_atproj(&old_root, "OldWork", "id-old", "OldWork", 1);

    let indexer = Indexer::open(index_path(&dir)).unwrap_or_else(|e| panic!("mở indexer: {e}"));
    let first = indexer.rebuild(&old_root, Some(&global)).unwrap_or_else(|e| panic!("rebuild trên gốc cũ: {e}"));
    assert_eq!(first.indexed, 1);

    // Người dùng "chọn thư mục D qua hộp thoại" -- `new_root` có một Tác phẩm KHÁC hẳn.
    write_atproj(&new_root, "NewWork", "id-new", "NewWork", 1);
    let second = indexer.rebuild(&new_root, Some(&global)).unwrap_or_else(|e| panic!("rebuild trên gốc mới: {e}"));
    assert_eq!(second.indexed, 1, "Tác phẩm của gốc MỚI phải được lập chỉ mục");
    assert_eq!(second.orphans, 1, "hàng của gốc CŨ phải thành mồ côi -- nó không có trong gốc mới");

    let works = indexer.list_works(WorkQuery::default()).unwrap_or_else(|e| panic!("list_works: {e}")).works;
    assert_eq!(works.len(), 1);
    assert_eq!(works[0].work_id, "id-new", "chỉ Tác phẩm của gốc MỚI được coi là đang sống");

    let orphans = indexer.list_orphans(Some(&global)).unwrap_or_else(|e| panic!("list_orphans: {e}"));
    assert_eq!(orphans.len(), 1);
    assert_eq!(orphans[0].work_id, "id-old");
    assert_eq!(orphans[0].atproj_path, old_work.display().to_string(), "giữ nguyên đường dẫn CŨ");

    // "dù thư mục đó vẫn còn trên đĩa" -- test không xoá `old_root`, khẳng định nó còn nguyên.
    assert!(old_work.is_dir(), "gốc CŨ không được ai đụng vào -- chỉ mục không nói về đĩa");
    assert!(old_work.join("meta.json").is_file());

    drop(indexer);
    drop(global);
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
    let global = open_global(&dir);
    let root = library_root(&dir);
    write_atproj(&root, "Foo", "id-a", "A", 1);

    let indexer = Indexer::open(index_path(&dir)).unwrap_or_else(|e| panic!("mở indexer: {e}"));
    let first = indexer.rebuild(&root, Some(&global)).unwrap_or_else(|e| panic!("rebuild đầu: {e}"));
    assert_eq!(first.indexed, 1);
    assert_eq!(
        indexer.list_works(WorkQuery::default()).unwrap_or_else(|e| panic!("list_works: {e}")).works[0].work_id,
        "id-a"
    );

    // Xoá A, rồi copy B vào MỘT thư mục CÙNG TÊN (`Foo.atproj`) -- work_id KHÁC hẳn.
    fs::remove_dir_all(root.join("Foo.atproj")).unwrap_or_else(|e| panic!("xoá Foo.atproj: {e}"));
    write_atproj(&root, "Foo", "id-b", "B", 1);

    let second = indexer.rebuild(&root, Some(&global)).unwrap_or_else(|e| panic!("rebuild sau khi B chiếm chỗ: {e}"));
    assert_eq!(second.indexed, 1, "chỉ B được lập chỉ mục ở đường dẫn đó");
    assert_eq!(second.orphans, 1, "hàng CŨ của A phải thành mồ côi -- đường dẫn nay thuộc về B, không phải A");

    let works = indexer.list_works(WorkQuery::default()).unwrap_or_else(|e| panic!("list_works: {e}")).works;
    assert_eq!(works.len(), 1);
    assert_eq!(works[0].work_id, "id-b", "chỉ B được coi là đang sống ở đường dẫn đó");

    let orphans = indexer.list_orphans(Some(&global)).unwrap_or_else(|e| panic!("list_orphans: {e}"));
    assert_eq!(orphans.len(), 1, "A phải Ở LẠI dưới dạng mồ côi, không bị B che khuất");
    assert_eq!(orphans[0].work_id, "id-a");

    drop(indexer);
    drop(global);
    cleanup(&dir);
}

#[test]
fn an_atproj_whose_meta_json_breaks_after_being_indexed_is_skipped_not_orphaned() {
    let dir = temp_dir("indexed-then-broken");
    let global = open_global(&dir);
    let root = library_root(&dir);
    let work_dir = write_atproj(&root, "Breaks", "id-breaks", "Breaks", 1);

    let indexer = Indexer::open(index_path(&dir)).unwrap_or_else(|e| panic!("mở indexer: {e}"));
    let first = indexer.rebuild(&root, Some(&global)).unwrap_or_else(|e| panic!("rebuild đầu: {e}"));
    assert_eq!(first.indexed, 1);

    // `meta.json` hỏng, nhưng thư mục `.atproj` VẪN CÒN ĐÓ -- vế HAI của vị từ mồ côi phải
    // giữ hàng này lại, không đánh dấu mồ côi.
    fs::write(work_dir.join("meta.json"), b"{ khong phai json hop le")
        .unwrap_or_else(|e| panic!("ghi meta.json hỏng: {e}"));

    let second = indexer.rebuild(&root, Some(&global)).unwrap_or_else(|e| panic!("rebuild sau khi hỏng: {e}"));
    assert_eq!(second.indexed, 0, "meta.json hỏng thì không đọc được nữa");
    assert_eq!(second.skipped.len(), 1, "phải rơi vào SKIPPED");
    assert_eq!(
        second.orphans, 0,
        "KHÔNG được thành mồ côi -- thư mục còn nằm đó, nói 'nó không còn ở đây' là SAI"
    );

    assert!(
        indexer.list_orphans(Some(&global)).unwrap_or_else(|e| panic!("list_orphans: {e}")).is_empty(),
        "không hàng nào được đánh dấu mồ côi ở ca này"
    );

    drop(indexer);
    drop(global);
    cleanup(&dir);
}

#[test]
fn forget_orphan_removes_exactly_the_named_row_and_returns_the_rest() {
    let dir = temp_dir("forget-orphan-ok");
    let global = open_global(&dir);
    let root = library_root(&dir);
    let alpha_dir = write_atproj(&root, "Alpha", "id-alpha", "Alpha", 1);
    write_atproj(&root, "Beta", "id-beta", "Beta", 1);

    let indexer = Indexer::open(index_path(&dir)).unwrap_or_else(|e| panic!("mở indexer: {e}"));
    indexer.rebuild(&root, Some(&global)).unwrap_or_else(|e| panic!("rebuild đầu: {e}"));

    fs::remove_dir_all(&alpha_dir).unwrap_or_else(|e| panic!("xoá Alpha: {e}"));
    fs::remove_dir_all(root.join("Beta.atproj")).unwrap_or_else(|e| panic!("xoá Beta: {e}"));
    indexer.rebuild(&root, Some(&global)).unwrap_or_else(|e| panic!("rebuild mồ côi: {e}"));
    assert_eq!(indexer.list_orphans(Some(&global)).unwrap_or_else(|e| panic!("list_orphans: {e}")).len(), 2);

    // 🔵 SỬA (2026-08-27, vòng rà THỨ HAI P3) — đọc TRỰC TIẾP giá trị TRẢ VỀ của
    // `forget_orphan`, không gọi `list_orphans()` một lần nữa: đây chính là ảnh chụp mà
    // `Indexer::forget_orphan` phải tự trả trong CÙNG phạm vi khoá (P3), và ca này phải kiểm
    // đúng CÁI ĐÓ, không phải một lượt đọc riêng biệt tình cờ cho cùng kết quả.
    let remaining = indexer
        .forget_orphan("id-alpha", Some(&global))
        .unwrap_or_else(|e| panic!("forget_orphan phải thành công trên một hàng mồ côi: {e}"));
    assert_eq!(remaining.len(), 1, "chỉ hàng vừa gỡ biến mất");
    assert_eq!(remaining[0].work_id, "id-beta");

    // Đối chứng: một lượt đọc riêng sau đó phải khớp với ảnh chụp đã trả.
    let reread = indexer.list_orphans(Some(&global)).unwrap_or_else(|e| panic!("list_orphans: {e}"));
    assert_eq!(reread, remaining);

    drop(indexer);
    drop(global);
    cleanup(&dir);
}

/// **THÊM 2026-08-27 (vòng rà THỨ HAI, P3 ②)** — gỡ một mục mồ côi VÀ một lượt `rebuild`
/// khác chạy gần như đồng thời không được sinh ra một `library.not_orphaned` SAI nguyên
/// nhân. Trước bản vá, `forget_orphan` không lấy `rebuild_lock`, nên một `rebuild` xen vào
/// đúng lúc `forget_orphan` đang đọc/ghi có thể lật cờ `orphaned` giữa chừng.
///
/// ⚠️ Cùng giới hạn đã ghi ở `two_threads_calling_rebuild_concurrently_converge_to_one_consistent_state`:
/// không có hook tiêm độ trễ, nên ca này không CHỨNG MINH một cửa sổ đua cụ thể — nó đối
/// chứng điều ĐO ĐƯỢC: hàng chục lượt `rebuild` xen với đúng MỘT lượt `forget_orphan` không
/// bao giờ panic/deadlock, và trạng thái cuối nhất quán (đúng đang-sống, không mồ côi trôi
/// nổi).
#[test]
fn forget_orphan_running_alongside_concurrent_rebuilds_never_reports_a_false_not_orphaned() {
    let dir = temp_dir("forget-orphan-concurrent-rebuild");
    // `Arc` -- hai luồng dưới đây cần cùng một `&Store` toàn cục, đúng khuôn `Arc<Indexer>`
    // ngay dưới (phán quyết Ice #1: `global.db` giữ `library_orphan`).
    let global = std::sync::Arc::new(open_global(&dir));
    let root = library_root(&dir);
    let alive_dir = write_atproj(&root, "Alive", "id-alive", "Alive", 1);
    let orphan_dir = write_atproj(&root, "Ghost", "id-ghost", "Ghost", 1);

    let indexer =
        std::sync::Arc::new(Indexer::open(index_path(&dir)).unwrap_or_else(|e| panic!("mở indexer: {e}")));
    indexer.rebuild(&root, Some(&global)).unwrap_or_else(|e| panic!("rebuild đầu: {e}"));
    fs::remove_dir_all(&orphan_dir).unwrap_or_else(|e| panic!("xoá Ghost: {e}"));
    indexer.rebuild(&root, Some(&global)).unwrap_or_else(|e| panic!("rebuild mồ côi: {e}"));
    assert_eq!(indexer.list_orphans(Some(&global)).unwrap_or_else(|e| panic!("list_orphans: {e}")).len(), 1);
    assert!(alive_dir.is_dir(), "Alive phải còn nguyên trên đĩa suốt ca này");

    let barrier = std::sync::Arc::new(std::sync::Barrier::new(2));

    let rebuild_indexer = std::sync::Arc::clone(&indexer);
    let rebuild_root = root.clone();
    let rebuild_global = std::sync::Arc::clone(&global);
    let rebuild_barrier = std::sync::Arc::clone(&barrier);
    let rebuild_handle = std::thread::spawn(move || {
        rebuild_barrier.wait();
        for _ in 0..20 {
            rebuild_indexer
                .rebuild(&rebuild_root, Some(&rebuild_global))
                .unwrap_or_else(|e| panic!("rebuild trên luồng nền: {e}"));
        }
    });

    let forget_indexer = std::sync::Arc::clone(&indexer);
    let forget_global = std::sync::Arc::clone(&global);
    let forget_barrier = std::sync::Arc::clone(&barrier);
    let forget_handle = std::thread::spawn(move || {
        forget_barrier.wait();
        forget_indexer.forget_orphan("id-ghost", Some(&forget_global))
    });

    rebuild_handle.join().expect("luồng rebuild panic");
    let forget_result = forget_handle.join().expect("luồng forget_orphan panic");

    // Đúng MỘT kết quả hợp lệ: hoặc gỡ THÀNH CÔNG (hàng còn mồ côi khi lệnh chạy tới), hoặc
    // bị từ chối vì KHÔNG CÒN mồ côi nữa (một `rebuild` xen vào đã đưa "Ghost" trở lại NẾU
    // nó tái xuất hiện -- ở đây nó không tái xuất hiện, nên nhánh từ chối không nên xảy ra
    // trong ca CỤ THỂ này, nhưng cả hai đều là kết quả HỢP LỆ về mặt LOGIC, không phải một
    // lỗi giả). Điều KHÔNG được xảy ra là panic hoặc deadlock -- đã được đảm bảo bằng việc
    // `.join()` ở trên trả về.
    match forget_result {
        Ok(remaining) => assert!(
            remaining.iter().all(|o| o.work_id != "id-ghost"),
            "gỡ thành công thì work_id đó không còn trong danh sách mồ côi"
        ),
        Err(IndexError::NotOrphaned { work_id }) => assert_eq!(work_id, "id-ghost"),
        Err(other) => panic!("kỳ vọng thành công hoặc NotOrphaned, nhận {other:?}"),
    }

    // Trạng thái CUỐI phải nhất quán: "Alive" vẫn sống, không hàng mồ côi trôi nổi ngoài dự
    // kiến (Ghost đã bị xoá khỏi đĩa suốt ca này, không quay lại).
    let works = indexer.list_works(WorkQuery::default()).unwrap_or_else(|e| panic!("list_works: {e}")).works;
    assert_eq!(works.len(), 1);
    assert_eq!(works[0].work_id, "id-alive");

    drop(indexer);
    drop(global);
    cleanup(&dir);
}

#[test]
fn forget_orphan_refuses_a_row_that_is_still_alive() {
    let dir = temp_dir("forget-orphan-refuses-live");
    let global = open_global(&dir);
    let root = library_root(&dir);
    write_atproj(&root, "Alive", "id-alive", "Alive", 1);

    let indexer = Indexer::open(index_path(&dir)).unwrap_or_else(|e| panic!("mở indexer: {e}"));
    indexer.rebuild(&root, Some(&global)).unwrap_or_else(|e| panic!("rebuild: {e}"));

    let err = indexer
        .forget_orphan("id-alive", Some(&global))
        .expect_err("gỡ một hàng ĐANG SỐNG phải bị từ chối, không im lặng thành công");
    match err {
        IndexError::NotOrphaned { work_id } => assert_eq!(work_id, "id-alive"),
        other => panic!("kỳ vọng IndexError::NotOrphaned, nhận {other:?}"),
    }
    assert_eq!(
        indexer.list_works(WorkQuery::default()).unwrap_or_else(|e| panic!("list_works: {e}")).works.len(),
        1,
        "0 lượt xoá -- hàng phải còn nguyên"
    );

    drop(indexer);
    drop(global);
    cleanup(&dir);
}

#[test]
fn forget_orphan_refuses_a_work_id_that_does_not_exist() {
    let dir = temp_dir("forget-orphan-refuses-unknown");
    let global = open_global(&dir);
    let root = library_root(&dir);

    let indexer = Indexer::open(index_path(&dir)).unwrap_or_else(|e| panic!("mở indexer: {e}"));
    indexer.rebuild(&root, Some(&global)).unwrap_or_else(|e| panic!("rebuild: {e}"));

    let err = indexer
        .forget_orphan("id-khong-ton-tai", Some(&global))
        .expect_err("work_id lạ phải bị từ chối trên CÙNG nhánh với 'gỡ nhầm hàng sống'");
    match err {
        IndexError::NotOrphaned { work_id } => assert_eq!(work_id, "id-khong-ton-tai"),
        other => panic!("kỳ vọng IndexError::NotOrphaned, nhận {other:?}"),
    }

    drop(indexer);
    drop(global);
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
    // `Arc` -- vòng lặp dưới đây spawn HAI luồng, mỗi luồng cần `move` riêng một bản; một
    // `Store` trần chỉ `move` được MỘT lần.
    let global = std::sync::Arc::new(open_global(&dir));
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
        let global = std::sync::Arc::clone(&global);
        let barrier = std::sync::Arc::clone(&barrier);
        handles.push(std::thread::spawn(move || {
            for _ in 0..20 {
                barrier.wait();
                indexer.rebuild(&root, Some(&global)).unwrap_or_else(|e| panic!("rebuild trên luồng nền: {e}"));
            }
        }));
    }
    for handle in handles {
        handle.join().expect("một luồng rebuild panic");
    }

    let works = indexer.list_works(WorkQuery::default()).unwrap_or_else(|e| panic!("list_works: {e}")).works;
    assert_eq!(works.len(), 2, "trạng thái cuối phải khớp ĐÚNG những gì còn trên đĩa");
    assert!(indexer.list_orphans(Some(&global)).unwrap_or_else(|e| panic!("list_orphans: {e}")).is_empty());

    drop(indexer);
    drop(global);
    cleanup(&dir);
}

// ═════════════════════════════════════════════════════════════════════════════════
// Phán quyết Ice #1 (2026-08-27) — thứ tự ghi fail-safe giữa `global.db` và
// `library-index.db` khi một hàng chuyển sang mồ côi. Không có giao dịch xuyên hai kho.
// ═════════════════════════════════════════════════════════════════════════════════

/// **Đối chứng bắt buộc** cho khối 🔴 "THỨ TỰ GHI GIỮA HAI KHO" ở doc-comment của
/// `Indexer::rebuild`: `global.db` (bảng `library_orphan`) phải được ghi TRƯỚC khi hàng
/// tương ứng bị xoá khỏi `library_work` (`library-index.db`). Không giao dịch nào bọc được
/// cả hai kho, nên khi bước ghi `global.db` THẤT BẠI, hàng phải CÒN NGUYÊN trong
/// `library_work` — không bị xoá "lỡ tay" trước khi lời nhắc mồ côi đã ghi xong ở nơi khác.
///
/// Mô phỏng "ghi `global.db` thất bại" bằng cách ĐÓNG `global` (`Store::close`) trước khi gọi
/// `rebuild` lượt hai — `store::Writer::write` sau khi đóng luôn trả `StoreError::WriterGone`
/// (`core/store/writer.rs`), một cách THẬT để mô phỏng một lượt ghi trượt, không phải suy
/// luận trên giấy.
///
/// 🔴 **Đối chứng GỠ đã chạy tay (xem báo cáo nghiệm thu của phán quyết):** đảo thứ tự trong
/// `Indexer::rebuild` (xoá khỏi `library_work` TRƯỚC, ghi `global.db` SAU) rồi chạy lại ca
/// này ⇒ ĐỎ đúng ở khẳng định "hàng còn nguyên trong `library_work`" — với thứ tự đảo đó, hàng
/// đã bị xoá TRƯỚC khi lượt ghi global (thất bại) kịp chạy, nên nó biến mất khỏi CẢ HAI kho
/// cùng lúc — đúng ca mất dữ liệu VĨNH VIỄN mà phán quyết cấm.
#[test]
fn orphan_write_order_is_fail_safe_write_global_before_deleting_from_index() {
    let dir = temp_dir("orphan-write-order-fail-safe");
    let global = open_global(&dir);
    let root = library_root(&dir);
    let work_dir = write_atproj(&root, "Gone", "id-gone", "Gone", 1);

    let indexer = Indexer::open(index_path(&dir)).unwrap_or_else(|e| panic!("mở indexer: {e}"));
    let first = indexer
        .rebuild(&root, Some(&global))
        .unwrap_or_else(|e| panic!("rebuild đầu: {e}"));
    assert_eq!(first.indexed, 1, "phải dựng được 1 hàng THẬT trước khi mô phỏng lỗi");

    fs::remove_dir_all(&work_dir).unwrap_or_else(|e| panic!("xoá .atproj: {e}"));

    // Mô phỏng "ghi global.db thất bại": đóng writer của nó TRƯỚC khi lượt rebuild kế tiếp
    // chạy tới bước ghi `library_orphan`.
    global.close();

    let err = indexer
        .rebuild(&root, Some(&global))
        .expect_err("global.db đã đóng -- lượt ghi library_orphan PHẢI trượt, không được xanh");
    // Không kiểm biến thể CỤ THỂ của lỗi kho (WriterGone hôm nay, có thể đổi nếu cơ chế đóng
    // kho đổi) -- điều ca này đối chứng là TRẠNG THÁI SAU lỗi, không phải hình dạng của lỗi.
    match err {
        IndexError::Store(_) => {}
        other => panic!("kỳ vọng IndexError::Store (global.db đã đóng), nhận {other:?}"),
    }

    // ⇒ ĐIỂM MẤU CHỐT của ca này: hàng phải CÒN NGUYÊN trong `library_work` -- bước xoá
    // KHÔNG được chạy trước khi bước ghi `global.db` đã xác nhận thành công.
    let works = indexer
        .list_works(WorkQuery::default())
        .unwrap_or_else(|e| panic!("list_works sau lỗi: {e}")).works;
    assert_eq!(
        works.len(),
        1,
        "hàng phải CÒN NGUYÊN trong library_work sau khi bước ghi global.db trượt -- xoá nó \
         TRƯỚC khi global.db xác nhận là mất lời nhắc mồ côi VĨNH VIỄN (không còn ở kho nào)"
    );
    assert_eq!(works[0].work_id, "id-gone");

    drop(indexer);
    drop(global);
    cleanup(&dir);
}

// ═════════════════════════════════════════════════════════════════════════════════
// Story 5.4 — bốn trạng thái vòng đời, bộ lọc tính TRONG SQL của `Indexer::list_works`.
// ═════════════════════════════════════════════════════════════════════════════════

/// Dựng một `<folder>.atproj/` với `status`/`status_is_override` TƯỜNG MINH — biến thể của
/// [`write_atproj`] cho các ca cần kiểm CHÍNH trạng thái vòng đời (33 chỗ gọi khác của
/// `write_atproj` không cần việc đó, xem doc-comment ở đó).
fn write_atproj_with_status(
    root: &Path,
    folder: &str,
    work_id: &str,
    name: &str,
    status: Option<&str>,
    status_is_override: bool,
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
        chapter_count: 1,
        status: status.map(str::to_owned),
        status_is_override,
        // 🔵 THÊM (2026-08-28, Story 5.5) — hàm này kiểm trạng thái vòng đời, không tiến độ;
        // ca CẦN kiểm tiến độ (bao gồm "ghi đè thủ công vẫn có tiến độ") dùng
        // `write_atproj_with_progress` riêng, xem cuối tệp.
        chapter_done_count: Some(0),
    };
    meta.write_atomic(&dir)
        .unwrap_or_else(|e| panic!("ghi meta.json ở {}: {e}", dir.display()));
    fs::write(dir.join("project.db"), b"not a real sqlite file -- AD-9")
        .unwrap_or_else(|e| panic!("ghi project.db gia: {e}"));

    dir
}

/// Dựng một `<folder>.atproj/` với `meta.json` **HÌNH DẠNG V1 THẬT** — tám khoá gốc, THIẾU
/// HẲN `status`/`status_is_override` (không phải `WorkMeta { status: None, .. }` rồi
/// serialize, thứ LUÔN ghi `"status":null` tường minh). Đây là bằng chứng của
/// `#[serde(default)]`: một tệp thật sự KHÔNG BAO GIỜ nhắc tới khoá đó, không phải một tệp
/// mới nhắc tới khoá đó với giá trị `null`.
fn write_v1_atproj_missing_lifecycle_fields(root: &Path, folder: &str, work_id: &str, name: &str) -> PathBuf {
    let dir = root.join(format!("{folder}.atproj"));
    fs::create_dir_all(&dir).unwrap_or_else(|e| panic!("tạo {}: {e}", dir.display()));

    let raw = format!(
        "{{\n  \"meta_schema_version\": 1,\n  \"work_id\": {work_id:?},\n  \"name\": {name:?},\n  \
         \"source_lang\": \"en\",\n  \"genre\": \"\",\n  \
         \"created_at\": \"2026-08-01T00:00:00.000Z\",\n  \
         \"updated_at\": \"2026-08-01T00:00:00.000Z\",\n  \"chapter_count\": 1\n}}"
    );
    fs::write(dir.join("meta.json"), raw).unwrap_or_else(|e| panic!("ghi meta.json v1 gia: {e}"));
    fs::write(dir.join("project.db"), b"not a real sqlite file -- AD-9")
        .unwrap_or_else(|e| panic!("ghi project.db gia: {e}"));

    dir
}

/// **Ca DUY NHẤT chứng minh phương án `Option<String>` khác phương án
/// mặc-định-`"not_started"`** (§Design Notes của story) — một `meta.json` v1 thật (thiếu hẳn
/// hai khoá mới) vào chỉ mục với `status IS NULL`, và hàng đó KHÔNG khớp bất kỳ giá trị nào
/// trong bốn bộ lọc, kể cả khi bộ lọc mang đủ cả bốn.
///
/// 🔴 Nếu `WorkMeta::status` từng là `String` với `#[serde(default)]` thay vì
/// `Option<String>`, giá trị mặc định của `String` (`""`, KHÔNG phải `"not_started"` như một
/// người đọc lướt có thể đoán) sẽ đi vào `library_work.status`; `""` không khớp
/// `LifecycleStatus::from_wire` ở bất kỳ đâu trong tầng lệnh, nhưng SQL `status IN (...)`
/// KHÔNG loại nó theo cùng cơ chế `NULL IN (...)` — nó là một chuỗi thật, không phải `NULL`.
/// Ca dưới đây khẳng định đúng cơ chế `NULL`, và vì thế phân biệt được hai phương án.
#[test]
fn a_true_v1_meta_json_indexes_with_status_null_and_matches_no_filter() {
    let dir = temp_dir("v1-meta-json-missing-fields");
    let global = open_global(&dir);
    let root = library_root(&dir);
    write_v1_atproj_missing_lifecycle_fields(&root, "Old", "id-old", "Old Work");

    let indexer = Indexer::open(index_path(&dir)).unwrap_or_else(|e| panic!("mo indexer: {e}"));
    let outcome = indexer.rebuild(&root, Some(&global)).unwrap_or_else(|e| panic!("rebuild: {e}"));
    assert_eq!(outcome.indexed, 1, "meta.json v1 phai doc duoc va vao chi muc, khong bi skip");

    let no_filter = indexer.list_works(WorkQuery::default()).unwrap_or_else(|e| panic!("list_works: {e}"));
    assert_eq!(no_filter.total, 1);
    assert_eq!(no_filter.works.len(), 1, "khong loc thi hang status IS NULL van phai co mat");
    assert_eq!(no_filter.works[0].status, None, "meta.json v1 phai doc ra status = None (chua biet)");
    assert!(!no_filter.works[0].status_is_override);
    // 🔵 THÊM (2026-08-28, Story 5.5) — cùng lý lẽ: khoá `chapter_done_count` cũng vắng mặt
    // ở một `meta.json` v1 thật, phải đọc ra `None`, KHÔNG `Some(0)`.
    assert_eq!(
        no_filter.works[0].chapter_done_count, None,
        "meta.json v1 phai doc ra chapter_done_count = None (chua biet), khong phai Some(0)"
    );

    let filtered = indexer
        .list_works(WorkQuery { status: Some(LifecycleStatus::ALL.to_vec()), ..Default::default() })
        .unwrap_or_else(|e| panic!("list_works loc du bon gia tri: {e}"));
    assert_eq!(filtered.total, 1, "tong so hang KHONG doi theo bo loc");
    assert!(
        filtered.works.is_empty(),
        "hang status IS NULL khong duoc khop bat ky gia tri nao trong bon bo loc: {:?}",
        filtered.works
    );

    drop(indexer);
    drop(global);
    cleanup(&dir);
}

/// UPSERT chở đúng `status`/`status_is_override` của `meta.json` — không giá trị nào bị bỏ
/// rơi giữa `WorkMeta` và `library_work`.
#[test]
fn rebuild_upserts_the_status_and_override_flag_from_meta_json() {
    let dir = temp_dir("status-roundtrip");
    let global = open_global(&dir);
    let root = library_root(&dir);
    write_atproj_with_status(&root, "Paused", "id-paused", "Paused Work", Some("paused"), true);

    let indexer = Indexer::open(index_path(&dir)).unwrap_or_else(|e| panic!("mo indexer: {e}"));
    indexer.rebuild(&root, Some(&global)).unwrap_or_else(|e| panic!("rebuild: {e}"));

    let works = indexer.list_works(WorkQuery::default()).unwrap_or_else(|e| panic!("list_works: {e}")).works;
    assert_eq!(works.len(), 1);
    assert_eq!(works[0].status.as_deref(), Some("paused"));
    assert!(works[0].status_is_override, "ghi de thu cong phai duoc giu nguyen qua UPSERT");

    drop(indexer);
    drop(global);
    cleanup(&dir);
}

/// Mỗi giá trị trong bốn giá trị lọc RIÊNG RẼ được — bốn Tác phẩm, bốn trạng thái khác nhau,
/// lọc theo từng giá trị chỉ trả đúng một hàng.
#[test]
fn each_of_the_four_lifecycle_statuses_filters_independently() {
    let dir = temp_dir("filter-each-status");
    let global = open_global(&dir);
    let root = library_root(&dir);
    write_atproj_with_status(&root, "A", "id-a", "A", Some("not_started"), false);
    write_atproj_with_status(&root, "B", "id-b", "B", Some("in_progress"), false);
    write_atproj_with_status(&root, "C", "id-c", "C", Some("paused"), true);
    write_atproj_with_status(&root, "D", "id-d", "D", Some("done"), false);

    let indexer = Indexer::open(index_path(&dir)).unwrap_or_else(|e| panic!("mo indexer: {e}"));
    indexer.rebuild(&root, Some(&global)).unwrap_or_else(|e| panic!("rebuild: {e}"));

    for (status, expected_id) in [
        (LifecycleStatus::NotStarted, "id-a"),
        (LifecycleStatus::InProgress, "id-b"),
        (LifecycleStatus::Paused, "id-c"),
        (LifecycleStatus::Done, "id-d"),
    ] {
        let report = indexer
            .list_works(WorkQuery { status: Some(vec![status]), ..Default::default() })
            .unwrap_or_else(|e| panic!("list_works loc {status:?}: {e}"));
        assert_eq!(report.total, 4, "tong so hang KHONG LOC luon la 4, bat ke bo loc nao");
        assert_eq!(
            report.works.len(),
            1,
            "loc theo {status:?} phai tra dung MOT hang, nhan {:?}",
            report.works
        );
        assert_eq!(report.works[0].work_id, expected_id);
    }

    drop(indexer);
    drop(global);
    cleanup(&dir);
}

/// **THÊM (lượt rà 2026-08-28)** — bộ lọc HAI-TRÊN-BỐN, tổ hợp mà người dùng bấm nhiều nhất
/// (*"chưa xong"* = `not_started` + `in_progress`). Bốn nút lọc bật/tắt độc lập nhau nên MỌI
/// tập con đều bấm được, nhưng bộ test chỉ có ca một-giá-trị và ca cả-bốn: một `IN (...)` dựng
/// sai với đúng hai tham số (thừa/thiếu một dấu `?`) lọt qua cả hai ca đó.
#[test]
fn a_two_of_four_filter_returns_exactly_the_two_matching_rows() {
    let dir = temp_dir("filter-two-of-four");
    let global = open_global(&dir);
    let root = library_root(&dir);
    write_atproj_with_status(&root, "A", "id-a", "A", Some("not_started"), false);
    write_atproj_with_status(&root, "B", "id-b", "B", Some("in_progress"), false);
    write_atproj_with_status(&root, "C", "id-c", "C", Some("paused"), true);
    write_atproj_with_status(&root, "D", "id-d", "D", Some("done"), false);

    let indexer = Indexer::open(index_path(&dir)).unwrap_or_else(|e| panic!("mo indexer: {e}"));
    indexer.rebuild(&root, Some(&global)).unwrap_or_else(|e| panic!("rebuild: {e}"));

    let report = indexer
        .list_works(WorkQuery { status: Some(vec![LifecycleStatus::NotStarted, LifecycleStatus::InProgress]), ..Default::default() })
        .unwrap_or_else(|e| panic!("list_works loc hai gia tri: {e}"));

    assert_eq!(report.total, 4, "tong so hang KHONG LOC van la 4");
    assert_eq!(report.works.len(), 2, "dung HAI hang khop, khong phai 1 va khong phai 4");
    let mut ids: Vec<&str> = report.works.iter().map(|w| w.work_id.as_str()).collect();
    ids.sort_unstable();
    assert_eq!(ids, vec!["id-a", "id-b"], "dung hai hang do, khong phai hai hang bat ky");

    drop(indexer);
    drop(global);
    cleanup(&dir);
}

/// Bộ lọc quét sạch: `matched = 0` MÀ `total > 0` — hai con số đến từ CÙNG một lượt đọc, và
/// một `total > 0` cùng `matched == 0` phải phân biệt được với "Library trống thật" (nơi cả
/// hai đều 0).
#[test]
fn a_filter_matching_nothing_still_reports_the_unfiltered_total() {
    let dir = temp_dir("filter-matches-nothing");
    let global = open_global(&dir);
    let root = library_root(&dir);
    write_atproj_with_status(&root, "A", "id-a", "A", Some("not_started"), false);
    write_atproj_with_status(&root, "B", "id-b", "B", Some("not_started"), false);
    write_atproj_with_status(&root, "C", "id-c", "C", Some("not_started"), false);

    let indexer = Indexer::open(index_path(&dir)).unwrap_or_else(|e| panic!("mo indexer: {e}"));
    indexer.rebuild(&root, Some(&global)).unwrap_or_else(|e| panic!("rebuild: {e}"));

    let report = indexer
        .list_works(WorkQuery { status: Some(vec![LifecycleStatus::Done]), ..Default::default() })
        .unwrap_or_else(|e| panic!("list_works: {e}"));
    assert_eq!(report.total, 3, "tong so hang trong chi muc, khong lien quan bo loc");
    assert!(report.works.is_empty(), "khong hang nao mang trang thai Done");

    drop(indexer);
    drop(global);
    cleanup(&dir);
}

/// Không lọc (`filter = None`): mọi hàng, KỂ CẢ hàng `status IS NULL` (mô phỏng một
/// `meta.json` v1 chưa từng qua `rebuild_from_store`) — `matched == total`.
#[test]
fn no_filter_returns_every_row_including_a_row_with_unknown_status() {
    let dir = temp_dir("no-filter-includes-unknown");
    let global = open_global(&dir);
    let root = library_root(&dir);
    write_atproj_with_status(&root, "Known", "id-known", "Known", Some("done"), false);
    write_atproj_with_status(&root, "Unknown", "id-unknown", "Unknown", None, false);

    let indexer = Indexer::open(index_path(&dir)).unwrap_or_else(|e| panic!("mo indexer: {e}"));
    indexer.rebuild(&root, Some(&global)).unwrap_or_else(|e| panic!("rebuild: {e}"));

    let report = indexer.list_works(WorkQuery::default()).unwrap_or_else(|e| panic!("list_works: {e}"));
    assert_eq!(report.total, 2);
    assert_eq!(report.works.len(), 2, "khong loc thi ca hang status IS NULL cung phai co mat");

    drop(indexer);
    drop(global);
    cleanup(&dir);
}

/// Một hàng `status IS NULL` (chưa biết) KHÔNG khớp bất kỳ giá trị nào trong bốn -- kể cả khi
/// bộ lọc mang ĐỦ cả bốn giá trị. `NULL IN (...)` luôn không đúng trong SQL, và đây là ca duy
/// nhất chứng minh phương án `Option<String>` (thay vì mặc định `"not_started"`) tạo ra khác
/// biệt QUAN SÁT ĐƯỢC: một mặc định sẽ làm hàng này lọt vào bộ lọc `NotStarted`.
#[test]
fn a_row_with_unknown_status_never_matches_any_filter_even_all_four_values_at_once() {
    let dir = temp_dir("unknown-status-matches-no-filter");
    let global = open_global(&dir);
    let root = library_root(&dir);
    write_atproj_with_status(&root, "Unknown", "id-unknown", "Unknown", None, false);

    let indexer = Indexer::open(index_path(&dir)).unwrap_or_else(|e| panic!("mo indexer: {e}"));
    indexer.rebuild(&root, Some(&global)).unwrap_or_else(|e| panic!("rebuild: {e}"));

    let report = indexer
        .list_works(WorkQuery { status: Some(LifecycleStatus::ALL.to_vec()), ..Default::default() })
        .unwrap_or_else(|e| panic!("list_works: {e}"));
    assert_eq!(report.total, 1);
    assert!(
        report.works.is_empty(),
        "mot hang status IS NULL khong duoc phep khop bat ky gia tri nao trong bo loc, ke ca \
         khi bo loc mang du bon gia tri: {:?}",
        report.works
    );

    drop(indexer);
    drop(global);
    cleanup(&dir);
}

// ═════════════════════════════════════════════════════════════════════════════════
// Story 5.6 — lọc lĩnh vực/ngôn ngữ nguồn, sắp xếp, tập lựa chọn (`WorkQuery`).
// ═════════════════════════════════════════════════════════════════════════════════

/// Dựng một `<folder>.atproj/` với `genre`/`source_lang`/`updated_at` TƯỜNG MINH — biến thể
/// của [`write_atproj_with_status`] cho các ca lọc/sắp của Story 5.6, vốn cần kiểm soát ba
/// trường đó chứ không phải trạng thái vòng đời.
#[allow(clippy::too_many_arguments)]
fn write_atproj_full(
    root: &Path,
    folder: &str,
    work_id: &str,
    name: &str,
    genre: &str,
    source_lang: &str,
    status: &str,
    updated_at: &str,
) -> PathBuf {
    let dir = root.join(format!("{folder}.atproj"));
    fs::create_dir_all(&dir).unwrap_or_else(|e| panic!("tạo {}: {e}", dir.display()));

    let meta = WorkMeta {
        meta_schema_version: META_SCHEMA_VERSION,
        work_id: work_id.to_owned(),
        name: name.to_owned(),
        source_lang: source_lang.to_owned(),
        genre: genre.to_owned(),
        created_at: "2026-08-01T00:00:00.000Z".to_owned(),
        updated_at: updated_at.to_owned(),
        chapter_count: 1,
        status: Some(status.to_owned()),
        status_is_override: false,
        chapter_done_count: Some(0),
    };
    meta.write_atomic(&dir)
        .unwrap_or_else(|e| panic!("ghi meta.json ở {}: {e}", dir.display()));
    fs::write(dir.join("project.db"), b"not a real sqlite file -- AD-9")
        .unwrap_or_else(|e| panic!("ghi project.db gia: {e}"));

    dir
}

/// §I/O Matrix "Lọc lĩnh vực": `WHERE genre = ?`, `matched` ≤ `total`.
#[test]
fn filtering_by_genre_matches_only_rows_with_that_exact_genre() {
    let dir = temp_dir("filter-genre");
    let global = open_global(&dir);
    let root = library_root(&dir);
    write_atproj_full(&root, "A", "id-a", "A", "Tien hiep", "zh", "not_started", "2026-08-01T00:00:00.000Z");
    write_atproj_full(&root, "B", "id-b", "B", "Do thi", "zh", "not_started", "2026-08-01T00:00:00.000Z");

    let indexer = Indexer::open(index_path(&dir)).unwrap_or_else(|e| panic!("mo indexer: {e}"));
    indexer.rebuild(&root, Some(&global)).unwrap_or_else(|e| panic!("rebuild: {e}"));

    let report = indexer
        .list_works(WorkQuery { genre: Some("Tien hiep".to_owned()), ..Default::default() })
        .unwrap_or_else(|e| panic!("list_works: {e}"));
    assert_eq!(report.total, 2, "total KHONG doi theo bo loc");
    assert_eq!(report.works.len(), 1);
    assert_eq!(report.works[0].work_id, "id-a");

    drop(indexer);
    drop(global);
    cleanup(&dir);
}

/// §I/O Matrix "Lọc ngôn ngữ": `WHERE source_lang = ?`.
#[test]
fn filtering_by_source_lang_matches_only_rows_with_that_exact_language() {
    let dir = temp_dir("filter-source-lang");
    let global = open_global(&dir);
    let root = library_root(&dir);
    write_atproj_full(&root, "A", "id-a", "A", "", "zh", "not_started", "2026-08-01T00:00:00.000Z");
    write_atproj_full(&root, "B", "id-b", "B", "", "en", "not_started", "2026-08-01T00:00:00.000Z");

    let indexer = Indexer::open(index_path(&dir)).unwrap_or_else(|e| panic!("mo indexer: {e}"));
    indexer.rebuild(&root, Some(&global)).unwrap_or_else(|e| panic!("rebuild: {e}"));

    let report = indexer
        .list_works(WorkQuery { source_lang: Some("zh".to_owned()), ..Default::default() })
        .unwrap_or_else(|e| panic!("list_works: {e}"));
    assert_eq!(report.total, 2);
    assert_eq!(report.works.len(), 1);
    assert_eq!(report.works[0].work_id, "id-a");

    drop(indexer);
    drop(global);
    cleanup(&dir);
}

/// §I/O Matrix "Ba bộ lọc chồng": trạng thái ∧ lĩnh vực ∧ ngôn ngữ — GIAO của cả ba, không
/// phải hợp. Bốn hàng, chỉ một hàng khớp cả ba tiêu chí.
#[test]
fn three_filters_stacked_intersect_instead_of_union() {
    let dir = temp_dir("filter-three-stacked");
    let global = open_global(&dir);
    let root = library_root(&dir);
    // Khớp CẢ BA: not_started + Tien hiep + zh.
    write_atproj_full(&root, "Match", "id-match", "Match", "Tien hiep", "zh", "not_started", "2026-08-01T00:00:00.000Z");
    // Đúng trạng thái + lĩnh vực nhưng SAI ngôn ngữ.
    write_atproj_full(&root, "WrongLang", "id-wrong-lang", "WrongLang", "Tien hiep", "en", "not_started", "2026-08-01T00:00:00.000Z");
    // Đúng trạng thái + ngôn ngữ nhưng SAI lĩnh vực.
    write_atproj_full(&root, "WrongGenre", "id-wrong-genre", "WrongGenre", "Do thi", "zh", "not_started", "2026-08-01T00:00:00.000Z");
    // Đúng lĩnh vực + ngôn ngữ nhưng SAI trạng thái.
    write_atproj_full(&root, "WrongStatus", "id-wrong-status", "WrongStatus", "Tien hiep", "zh", "done", "2026-08-01T00:00:00.000Z");

    let indexer = Indexer::open(index_path(&dir)).unwrap_or_else(|e| panic!("mo indexer: {e}"));
    indexer.rebuild(&root, Some(&global)).unwrap_or_else(|e| panic!("rebuild: {e}"));

    let report = indexer
        .list_works(WorkQuery {
            status: Some(vec![LifecycleStatus::NotStarted]),
            genre: Some("Tien hiep".to_owned()),
            source_lang: Some("zh".to_owned()),
            sort: WorkSortKey::default(),
        })
        .unwrap_or_else(|e| panic!("list_works: {e}"));
    assert_eq!(report.total, 4);
    assert_eq!(report.works.len(), 1, "chi hang khop CA BA tieu chi moi duoc tra ve: {:?}", report.works);
    assert_eq!(report.works[0].work_id, "id-match");

    drop(indexer);
    drop(global);
    cleanup(&dir);
}

/// §I/O Matrix "Lĩnh vực không tồn tại": `matched = 0`, `total` giữ nguyên — 0 kết quả là một
/// kết quả HỢP LỆ, không phải một lỗi.
#[test]
fn filtering_by_a_genre_that_does_not_exist_matches_nothing_but_keeps_the_total() {
    let dir = temp_dir("filter-genre-missing");
    let global = open_global(&dir);
    let root = library_root(&dir);
    write_atproj_full(&root, "A", "id-a", "A", "Tien hiep", "zh", "not_started", "2026-08-01T00:00:00.000Z");

    let indexer = Indexer::open(index_path(&dir)).unwrap_or_else(|e| panic!("mo indexer: {e}"));
    indexer.rebuild(&root, Some(&global)).unwrap_or_else(|e| panic!("rebuild: {e}"));

    let report = indexer
        .list_works(WorkQuery { genre: Some("Khong co".to_owned()), ..Default::default() })
        .unwrap_or_else(|e| panic!("list_works: {e}"));
    assert_eq!(report.total, 1, "total van la so hang trong chi muc");
    assert!(report.works.is_empty());

    drop(indexer);
    drop(global);
    cleanup(&dir);
}

/// §I/O Matrix "Sắp theo ngày sửa" (mặc định) — `ORDER BY updated_at DESC, work_id`.
#[test]
fn sorting_by_updated_at_orders_the_most_recently_touched_work_first() {
    let dir = temp_dir("sort-updated-desc");
    let global = open_global(&dir);
    let root = library_root(&dir);
    write_atproj_full(&root, "Old", "id-old", "Old", "", "en", "not_started", "2026-08-01T00:00:00.000Z");
    write_atproj_full(&root, "New", "id-new", "New", "", "en", "not_started", "2026-08-20T00:00:00.000Z");
    write_atproj_full(&root, "Mid", "id-mid", "Mid", "", "en", "not_started", "2026-08-10T00:00:00.000Z");

    let indexer = Indexer::open(index_path(&dir)).unwrap_or_else(|e| panic!("mo indexer: {e}"));
    indexer.rebuild(&root, Some(&global)).unwrap_or_else(|e| panic!("rebuild: {e}"));

    let report = indexer
        .list_works(WorkQuery { sort: WorkSortKey::UpdatedDesc, ..Default::default() })
        .unwrap_or_else(|e| panic!("list_works: {e}"));
    let ids: Vec<&str> = report.works.iter().map(|w| w.work_id.as_str()).collect();
    assert_eq!(ids, vec!["id-new", "id-mid", "id-old"], "moi nhat truoc, cu nhat sau: {ids:?}");

    drop(indexer);
    drop(global);
    cleanup(&dir);
}

/// §I/O Matrix "Sắp theo tên" — `ORDER BY name COLLATE NOCASE, work_id`, không phân biệt
/// hoa/thường.
#[test]
fn sorting_by_name_orders_case_insensitively() {
    let dir = temp_dir("sort-name-asc");
    let global = open_global(&dir);
    let root = library_root(&dir);
    write_atproj_full(&root, "Zebra", "id-zebra", "zebra", "", "en", "not_started", "2026-08-01T00:00:00.000Z");
    write_atproj_full(&root, "Apple", "id-apple", "Apple", "", "en", "not_started", "2026-08-01T00:00:00.000Z");
    write_atproj_full(&root, "middle", "id-middle", "middle", "", "en", "not_started", "2026-08-01T00:00:00.000Z");

    let indexer = Indexer::open(index_path(&dir)).unwrap_or_else(|e| panic!("mo indexer: {e}"));
    indexer.rebuild(&root, Some(&global)).unwrap_or_else(|e| panic!("rebuild: {e}"));

    let report = indexer
        .list_works(WorkQuery { sort: WorkSortKey::NameAsc, ..Default::default() })
        .unwrap_or_else(|e| panic!("list_works: {e}"));
    let ids: Vec<&str> = report.works.iter().map(|w| w.work_id.as_str()).collect();
    assert_eq!(
        ids,
        vec!["id-apple", "id-middle", "id-zebra"],
        "'zebra' (thuong) phai dung SAU 'Apple' (hoa) -- COLLATE NOCASE: {ids:?}"
    );

    drop(indexer);
    drop(global);
    cleanup(&dir);
}

/// §I/O Matrix "Hai Tác phẩm cùng `updated_at`" — thứ tự ỔN ĐỊNH nhờ khoá phụ `work_id`,
/// giống hệt nhau giữa hai lượt gọi liên tiếp.
#[test]
fn two_works_with_the_same_updated_at_get_a_stable_order_via_the_work_id_tiebreaker() {
    let dir = temp_dir("sort-stable-tiebreak");
    let global = open_global(&dir);
    let root = library_root(&dir);
    let same_moment = "2026-08-15T00:00:00.000Z";
    // 🔴 VÌ SAO TÊN THƯ MỤC NGƯỢC VỚI `work_id` — đối chứng đã ĐO, không suy luận.
    //
    // `scan_atproj_dirs` sắp thư mục theo TÊN (`dirs.sort()`), và `library_work` là bảng
    // thường (KHÔNG `WITHOUT ROWID`) nên nó mang `rowid` ẩn theo thứ tự UPSERT. Nếu tên thư
    // mục ("A" < "Z") CÙNG chiều với `work_id` ("id-a" < "id-z"), một `ORDER BY` THIẾU khoá
    // phụ `work_id` vẫn có thể trả về ĐÚNG thứ tự — không phải nhờ khoá phụ, mà nhờ SQLite
    // quét bảng theo `rowid` (== thứ tự UPSERT == thứ tự quét thư mục) TRÙNG HỢP với thứ tự
    // `work_id` mong đợi. Đã đo: gỡ `, work_id` khỏi `ORDER BY` với tên thư mục "A"/"B" khớp
    // `work_id` "id-a"/"id-b" ⇒ ca này vẫn XANH — đúng cái bẫy §Verification của story cảnh
    // báo ("nếu vẫn xanh thì ca đó chưa dựng được hai hàng trùng mốc — sửa ca, không bỏ qua").
    // ⇒ Thư mục "A" mang `id-z`, thư mục "Z" mang `id-a`: thứ tự QUÉT/UPSERT (theo tên thư
    // mục) là id-z RỒI id-a — NGƯỢC thứ tự `work_id` mong đợi (id-a < id-z). Giờ chỉ MỘT
    // trong hai cơ chế cho đúng kết quả: khoá phụ `work_id` tường minh. Gỡ nó ⇒ ca này ĐỎ
    // (đã tự đối chứng khi viết bản vá này).
    write_atproj_full(&root, "A", "id-z", "Z", "", "en", "not_started", same_moment);
    write_atproj_full(&root, "Z", "id-a", "A", "", "en", "not_started", same_moment);

    let indexer = Indexer::open(index_path(&dir)).unwrap_or_else(|e| panic!("mo indexer: {e}"));
    indexer.rebuild(&root, Some(&global)).unwrap_or_else(|e| panic!("rebuild: {e}"));

    let first = indexer
        .list_works(WorkQuery { sort: WorkSortKey::UpdatedDesc, ..Default::default() })
        .unwrap_or_else(|e| panic!("list_works lan 1: {e}"));
    let second = indexer
        .list_works(WorkQuery { sort: WorkSortKey::UpdatedDesc, ..Default::default() })
        .unwrap_or_else(|e| panic!("list_works lan 2: {e}"));

    let ids_first: Vec<&str> = first.works.iter().map(|w| w.work_id.as_str()).collect();
    let ids_second: Vec<&str> = second.works.iter().map(|w| w.work_id.as_str()).collect();
    assert_eq!(ids_first, ids_second, "hai lam KHONG duoc doi thu tu khi khoa sap chinh trung nhau");
    // `id-a` < `id-z` theo khoá phụ `work_id` -- NGƯỢC thứ tự quét thư mục (xem khối lý lẽ
    // trên). Chỉ đúng khi `ORDER BY` THẬT SỰ mang `, work_id`.
    assert_eq!(ids_first, vec!["id-a", "id-z"]);

    drop(indexer);
    drop(global);
    cleanup(&dir);
}

/// §I/O Matrix "Tập lựa chọn" — `genres`/`source_langs` là `DISTINCT` trên bảng CHƯA LỌC:
/// dù lọc theo lĩnh vực "Tien hiep", cả hai tập lựa chọn vẫn liệt kê MỌI giá trị có trong chỉ
/// mục, không teo dần theo bộ lọc đang bật (AD-1, §Always).
#[test]
fn selection_sets_are_distinct_over_the_unfiltered_table_even_while_a_filter_is_active() {
    let dir = temp_dir("selection-sets-unfiltered");
    let global = open_global(&dir);
    let root = library_root(&dir);
    write_atproj_full(&root, "A", "id-a", "A", "Tien hiep", "zh", "not_started", "2026-08-01T00:00:00.000Z");
    write_atproj_full(&root, "B", "id-b", "B", "Do thi", "en", "not_started", "2026-08-01T00:00:00.000Z");
    write_atproj_full(&root, "C", "id-c", "C", "Tien hiep", "en", "not_started", "2026-08-01T00:00:00.000Z");

    let indexer = Indexer::open(index_path(&dir)).unwrap_or_else(|e| panic!("mo indexer: {e}"));
    indexer.rebuild(&root, Some(&global)).unwrap_or_else(|e| panic!("rebuild: {e}"));

    // Lọc chỉ còn "Tien hiep" -- nếu tập lựa chọn suy từ `works` ĐÃ LỌC, "Do thi" sẽ biến mất.
    let report = indexer
        .list_works(WorkQuery { genre: Some("Tien hiep".to_owned()), ..Default::default() })
        .unwrap_or_else(|e| panic!("list_works: {e}"));
    assert_eq!(report.works.len(), 2, "hai hang khop bo loc lĩnh vực");

    let mut genres = report.genres.clone();
    genres.sort_unstable();
    assert_eq!(
        genres,
        vec!["Do thi".to_owned(), "Tien hiep".to_owned()],
        "genres phai liet ke CA HAI gia tri, ke ca gia tri khong khop bo loc dang bat: {genres:?}"
    );
    let mut source_langs = report.source_langs.clone();
    source_langs.sort_unstable();
    assert_eq!(
        source_langs,
        vec!["en".to_owned(), "zh".to_owned()],
        "source_langs phai liet ke CA HAI ngon ngu, khong teo theo bo loc dang bat: {source_langs:?}"
    );

    drop(indexer);
    drop(global);
    cleanup(&dir);
}
