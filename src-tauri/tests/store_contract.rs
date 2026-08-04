//! Hành vi của tầng ghi dữ liệu — Story 1.7, AC1 tới AC7.
//!
//! ⚠️ Tệp riêng có chủ ý, đúng khuôn `config_invariants.rs` (*bất biến cấu hình*) và
//! `ipc_contract.rs` (*hợp đồng dây*). Tệp này nghiệm thu **hành vi lúc chạy**; ranh giới
//! cây nguồn của AC2 nằm ở `store_boundary.rs`.
//!
//! ─────────────────────────────────────────────────────────────────────────────
//! BỐN LUẬT CỦA TỆP NÀY
//! ─────────────────────────────────────────────────────────────────────────────
//! 1. **Mỗi ca một thư mục tạm riêng.** `cargo test` chạy các ca **song song trong cùng
//!    một tiến trình**; hai ca dùng chung một đường dẫn `.db` sẽ đỏ ngẫu nhiên và bị đọc
//!    thành flaky. Tên gồm pid + một bộ đếm nguyên tử. ⛔ Không thêm `tempfile`.
//! 2. **Drop `Store` TRƯỚC khi xoá thư mục.** Windows từ chối xoá tệp đang mở — một
//!    `remove_dir_all` sớm cho ra một test đỏ **chỉ trên nhánh Windows** của ma trận,
//!    đúng lớp lỗi NFR14 mà Story 1.3 dựng CI để bắt.
//! 3. **Không đo thời gian bằng `sleep` dài.** Các ca của AC4/AC5 lái cơ chế bằng
//!    `Tuning` thu nhỏ (tick và idle tính bằng chục mili-giây), không bằng cách chờ 5
//!    giây thật — nhân với hai nền tảng thì đó là phút CI.
//! 4. **Không ca nào được treo khi nó trượt.** Mọi phép chờ có trần: `recv_timeout`, một
//!    hạn chót trên vòng quay. Một test treo trên CI đắt hơn một test đỏ, và nó không
//!    nói cho ai biết cái gì hỏng.
//!
//! ─────────────────────────────────────────────────────────────────────────────
//! ⚠️ VÌ SAO TỆP NÀY ĐƯỢC PHÉP `use rusqlite`
//! ─────────────────────────────────────────────────────────────────────────────
//! AC2 cấm **module sản phẩm** tự mở kết nối ghi, và `store_boundary.rs` cưỡng chế điều
//! đó trên `src-tauri/src/**`. `tests/**` nằm ngoài — có tên, có lý do, và lý do là:
//! ba ca của AC6/AC7 cần dựng một database ở **một phiên bản lược đồ cho trước, trong
//! chế độ journal cho trước**, tức đúng thứ `core::store` tồn tại để không ai làm được.
//! Đường thay thế duy nhất là thêm một hàm `pub` vào mã sản phẩm mà **chỉ test gọi** —
//! mã không ai dùng, đúng thứ Story 1.5 và 1.6 đã từ chối hai lần.

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::sync::{Arc, mpsc};
use std::thread;
use std::time::{Duration, Instant};

use auratranslate_lib::core::i18n::{IpcError, MessageKey};
use auratranslate_lib::core::store::{
    GLOBAL_MIGRATIONS, Migration, SCHEMA_MIGRATION_LOG_DDL, Store, StoreError, StoreKind,
    StoreSpec, Tuning,
};

// ═════════════════════════════════════════════════════════════════════════════════
// Hạ tầng dùng chung
// ═════════════════════════════════════════════════════════════════════════════════

static NEXT_DIR: AtomicU64 = AtomicU64::new(0);

/// Một thư mục tạm **của riêng ca này**. Xem luật 1 ở doc-comment đầu tệp.
fn temp_dir(tag: &str) -> PathBuf {
    let n = NEXT_DIR.fetch_add(1, Ordering::Relaxed);
    let dir = std::env::temp_dir().join(format!(
        "auratranslate-store-{}-{}-{}",
        std::process::id(),
        tag,
        n
    ));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap_or_else(|e| panic!("tạo {}: {e}", dir.display()));
    dir
}

/// ⚠️ Gọi **sau** khi `Store` đã drop. Xem luật 2.
fn cleanup(dir: &Path) {
    let _ = fs::remove_dir_all(dir);
}

fn db_path(dir: &Path) -> PathBuf {
    dir.join("global.db")
}

fn sidecar(db: &Path, suffix: &str) -> PathBuf {
    let mut raw = db.as_os_str().to_owned();
    raw.push(suffix);
    PathBuf::from(raw)
}

fn file_len(path: &Path) -> u64 {
    fs::metadata(path).map(|m| m.len()).unwrap_or(0)
}

/// `Tuning` cho các ca **không** quan tâm tới checkpoint: nhịp chậm, ngưỡng vô cực, nên
/// luồng nền không chen vào phép đo của ca khác.
fn quiet_tuning() -> Tuning {
    Tuning {
        checkpoint_tick: Duration::from_millis(50),
        idle_before_passive: Duration::from_secs(3600),
        wal_threshold_bytes: u64::MAX,
        close_truncate_budget: Duration::from_secs(5),
        ..Tuning::default()
    }
}

fn spec_with(dir: &Path, tuning: Tuning) -> StoreSpec {
    StoreSpec {
        tuning,
        ..StoreSpec::global(db_path(dir))
    }
}

/// Bảng dò của AC1. Ba cột ghi lại ba thứ khác nhau — xem [`writes_are_serialized`].
const PROBE_DDL: &str = "\
CREATE TABLE probe (
  id          INTEGER PRIMARY KEY,
  worker      INTEGER NOT NULL,
  open_before INTEGER NOT NULL,
  depth_seen  INTEGER NOT NULL,
  thread      TEXT    NOT NULL,
  exited      INTEGER NOT NULL
);
CREATE TABLE depth (id INTEGER PRIMARY KEY CHECK (id = 1), value INTEGER NOT NULL);
INSERT INTO depth (id, value) VALUES (1, 0);";

/// `Store` phải vào được `app.manage(…)` mà không cần bọc `Mutex` — Quyết định #3.
/// Một phép kiểm lúc **biên dịch**; nó đỏ ở `cargo build`, không ở `cargo test`.
#[test]
fn store_is_send_and_sync() {
    fn require<T: Send + Sync + 'static>() {}
    require::<Store>();
}

// ═════════════════════════════════════════════════════════════════════════════════
// AC1 — một writer nối tiếp, pool đọc song song
// ═════════════════════════════════════════════════════════════════════════════════

/// **Ca 1** — 8 luồng × 50 job ghi đồng thời.
///
/// 🔴 Ba mệnh đề, và cả ba được chứng minh bằng **một bảng đếm ghi vào chính database**,
/// ⛔ không bằng suy luận (AC1 nói nguyên văn như vậy):
///
/// 1. **Đúng MỘT luồng writer** — mỗi job ghi lại `thread::current().id()` của luồng đã
///    chạy nó. Hơn một giá trị khác nhau nghĩa là có hơn một kết nối ghi.
/// 2. **Không job nào lồng nhau** — một bộ đếm `depth` trong database: mỗi job tăng nó,
///    đọc lại, rồi giảm. Đọc về khác 1 nghĩa là một job khác đang dở dang bên trong job
///    này.
/// 3. **Không mất bản ghi nào** — 400 hàng, không hơn không kém.
#[test]
fn writes_are_serialized() {
    let dir = temp_dir("serial-writes");
    let store = Arc::new(Store::open(spec_with(&dir, quiet_tuning())).expect("mở kho"));

    store
        .write(|tx| tx.execute_batch(PROBE_DDL))
        .expect("dựng bảng dò");

    const WORKERS: i64 = 8;
    const PER_WORKER: i64 = 50;

    let mut threads = Vec::new();
    for worker in 0..WORKERS {
        let store = Arc::clone(&store);
        threads.push(thread::spawn(move || {
            for _ in 0..PER_WORKER {
                store
                    .write(move |tx| {
                        // Số job đang dở dang mà giao dịch này NHÌN THẤY.
                        let open_before: i64 = tx.query_row(
                            "SELECT COUNT(*) FROM probe WHERE exited = 0",
                            [],
                            |row| row.get(0),
                        )?;

                        tx.execute("UPDATE depth SET value = value + 1 WHERE id = 1", [])?;
                        let depth_seen: i64 =
                            tx.query_row("SELECT value FROM depth WHERE id = 1", [], |row| {
                                row.get(0)
                            })?;
                        tx.execute("UPDATE depth SET value = value - 1 WHERE id = 1", [])?;

                        let thread_id = format!("{:?}", thread::current().id());
                        tx.execute(
                            "INSERT INTO probe (worker, open_before, depth_seen, thread, exited) \
                             VALUES (?1, ?2, ?3, ?4, 0)",
                            rusqlite::params![worker, open_before, depth_seen, thread_id],
                        )?;
                        tx.execute("UPDATE probe SET exited = 1 WHERE id = last_insert_rowid()", [])?;
                        Ok(())
                    })
                    .expect("job ghi phải thành công");
            }
        }));
    }
    for t in threads {
        t.join().expect("luồng ghi không được panic");
    }

    let (rows, dirty, nested, threads_used, max_depth) = store
        .read(|conn| {
            let rows: i64 = conn.query_row("SELECT COUNT(*) FROM probe", [], |r| r.get(0))?;
            let dirty: i64 =
                conn.query_row("SELECT COUNT(*) FROM probe WHERE exited = 0", [], |r| {
                    r.get(0)
                })?;
            let nested: i64 = conn.query_row(
                "SELECT COUNT(*) FROM probe WHERE open_before <> 0",
                [],
                |r| r.get(0),
            )?;
            let threads_used: i64 =
                conn.query_row("SELECT COUNT(DISTINCT thread) FROM probe", [], |r| r.get(0))?;
            let max_depth: i64 =
                conn.query_row("SELECT MAX(depth_seen) FROM probe", [], |r| r.get(0))?;
            Ok((rows, dirty, nested, threads_used, max_depth))
        })
        .expect("job đọc");

    assert_eq!(
        rows,
        WORKERS * PER_WORKER,
        "mất bản ghi: hàng đợi ghi đánh rơi job, hoặc một giao dịch commit đè lên giao dịch khác"
    );
    assert_eq!(dirty, 0, "một job commit mà không đánh dấu đã ra — giao dịch bị cắt giữa chừng");
    assert_eq!(
        nested, 0,
        "một job nhìn thấy job khác đang dở dang ⇒ hai giao dịch ghi lồng nhau ⇒ AC1 trượt"
    );
    assert_eq!(
        threads_used, 1,
        "{threads_used} luồng khác nhau đã chạy job ghi. AC1 đòi ĐÚNG MỘT kết nối ghi sau \
         một hàng đợi nối tiếp — nhiều hơn một luồng nghĩa là nhiều hơn một kết nối."
    );
    assert_eq!(
        max_depth, 1,
        "bộ đếm độ sâu trong database lên tới {max_depth} ⇒ một job chạy bên trong một job khác"
    );

    drop(store);
    cleanup(&dir);
}

/// **Ca 2** — 4 luồng đọc đồng thời trong khi writer đang chạy.
///
/// ⚠️ Bằng chứng là **đỉnh số reader cùng lúc**, đo bằng một bộ đếm nguyên tử ngay bên
/// trong closure đọc. Vòng chờ có **hạn chót** chứ không phải `Barrier`: một pool bị
/// serialize hoá sẽ làm `Barrier::wait` treo mãi, mà một test treo trên CI thì không nói
/// cho ai biết cái gì hỏng.
#[test]
fn reads_run_in_parallel_while_the_writer_works() {
    let dir = temp_dir("parallel-reads");
    let tuning = Tuning {
        pool_size: 4,
        ..quiet_tuning()
    };
    let store = Arc::new(Store::open(spec_with(&dir, tuning)).expect("mở kho"));

    store
        .write(|tx| tx.execute_batch(PROBE_DDL))
        .expect("dựng bảng dò");

    let keep_writing = Arc::new(AtomicUsize::new(1));
    let writer = {
        let store = Arc::clone(&store);
        let keep_writing = Arc::clone(&keep_writing);
        thread::spawn(move || {
            let mut n = 0i64;
            while keep_writing.load(Ordering::SeqCst) == 1 {
                store
                    .write(move |tx| {
                        tx.execute(
                            "INSERT INTO probe (worker, open_before, depth_seen, thread, exited) \
                             VALUES (?1, 0, 0, 'writer', 1)",
                            rusqlite::params![n],
                        )?;
                        Ok(())
                    })
                    .expect("job ghi nền");
                n += 1;
            }
            n
        })
    };

    const READERS: usize = 4;
    let live = Arc::new(AtomicUsize::new(0));
    let peak = Arc::new(AtomicUsize::new(0));

    let mut threads = Vec::new();
    for _ in 0..READERS {
        let store = Arc::clone(&store);
        let live = Arc::clone(&live);
        let peak = Arc::clone(&peak);
        threads.push(thread::spawn(move || {
            store
                .read(|conn| {
                    let now = live.fetch_add(1, Ordering::SeqCst) + 1;
                    peak.fetch_max(now, Ordering::SeqCst);

                    let deadline = Instant::now() + Duration::from_secs(5);
                    while live.load(Ordering::SeqCst) < READERS && Instant::now() < deadline {
                        thread::yield_now();
                    }

                    let seen: i64 =
                        conn.query_row("SELECT COUNT(*) FROM probe", [], |r| r.get(0))?;
                    live.fetch_sub(1, Ordering::SeqCst);
                    Ok(seen)
                })
                .expect("job đọc phải chạy trong khi writer đang ghi")
        }));
    }

    for t in threads {
        t.join().expect("luồng đọc không được panic");
    }
    keep_writing.store(0, Ordering::SeqCst);
    let written = writer.join().expect("luồng ghi nền không được panic");

    assert_eq!(
        peak.load(Ordering::SeqCst),
        READERS,
        "đỉnh số reader cùng lúc là {} chứ không phải {READERS} ⇒ đường đọc đang bị nối tiếp hoá. \
         AC1 đòi pool NHIỀU kết nối chạy song song trên WAL.",
        peak.load(Ordering::SeqCst)
    );
    assert!(
        written > 0,
        "luồng ghi nền không ghi được bản ghi nào trong lúc bốn reader đang mở — writer bị reader chặn"
    );

    drop(store);
    cleanup(&dir);
}

/// **Ca 13** — writer đã dừng thì `write()` trả lỗi **trong thời gian hữu hạn**.
///
/// 🔴 Trần thời gian là cả điểm của ca này. Bản cài đặt sai không trả về `Err` — nó
/// **treo** trên `recv()`, và một `write()` treo trên đường gõ là NFR2 chết mà không lỗi
/// nào được ném. `recv_timeout` biến một ca treo thành một ca đỏ.
#[test]
fn write_after_close_fails_instead_of_hanging() {
    let dir = temp_dir("writer-gone");
    let store = Arc::new(Store::open(spec_with(&dir, quiet_tuning())).expect("mở kho"));

    store.close();

    let (tx, rx) = mpsc::channel();
    {
        let store = Arc::clone(&store);
        thread::spawn(move || {
            let outcome = store.write(|tx| tx.execute_batch("CREATE TABLE late (id INTEGER)"));
            let _ = tx.send(outcome);
        });
    }

    let outcome = rx
        .recv_timeout(Duration::from_secs(5))
        .expect("`write()` sau `close()` phải TRẢ VỀ, không được treo");

    match outcome {
        Err(StoreError::WriterGone { store }) => assert_eq!(store, StoreKind::Global),
        other => panic!("kỳ vọng StoreError::WriterGone, nhận {other:?}"),
    }

    drop(store);
    cleanup(&dir);
}

// ═════════════════════════════════════════════════════════════════════════════════
// AC2 — không đường ghi thứ hai
// ═════════════════════════════════════════════════════════════════════════════════

/// **Ca 3** — một `INSERT` qua đường `read()` phải **thất bại**, với lỗi của SQLite.
///
/// 🔴 Đây là bằng chứng của AC2 vế *"khả năng hiển thị của kiểu"*: cưỡng chế đến từ
/// `PRAGMA query_only = 1`, tức từ **SQLite**, ⛔ không từ việc người viết tự nhớ.
#[test]
fn writing_through_the_read_path_is_refused_by_sqlite() {
    let dir = temp_dir("read-only");
    let store = Store::open(spec_with(&dir, quiet_tuning())).expect("mở kho");

    store
        .write(|tx| tx.execute_batch(PROBE_DDL))
        .expect("dựng bảng dò");

    let err = store
        .read(|conn| {
            conn.execute(
                "INSERT INTO probe (worker, open_before, depth_seen, thread, exited) \
                 VALUES (1, 0, 0, 'reader', 1)",
                [],
            )
        })
        .expect_err("một INSERT qua đường đọc PHẢI thất bại — nếu nó thành công thì AC2 trượt");

    match err {
        StoreError::ReadFailed { store, detail } => {
            assert_eq!(store, StoreKind::Global);
            assert!(
                detail.to_lowercase().contains("readonly")
                    || detail.to_lowercase().contains("read-only"),
                "lỗi phải đến TỪ SQLITE (\"attempt to write a readonly database\"), \
                 không phải từ một phép kiểm tự viết. Nhận: {detail}"
            );
        }
        other => panic!("kỳ vọng StoreError::ReadFailed, nhận {other:?}"),
    }

    // Và bảng vẫn rỗng — lỗi được ném RA chứ không phải ghi rồi mới than.
    let rows: i64 = store
        .read(|conn| conn.query_row("SELECT COUNT(*) FROM probe", [], |r| r.get(0)))
        .expect("đọc lại");
    assert_eq!(rows, 0, "hàng đã lọt vào database qua đường đọc");

    drop(store);
    cleanup(&dir);
}

// ═════════════════════════════════════════════════════════════════════════════════
// AC3 — ba PRAGMA, đặt rồi ĐỌC LẠI
// ═════════════════════════════════════════════════════════════════════════════════

/// **Ca 4** — mở mới rồi đọc lại ba PRAGMA, **trên cả hai loại kết nối**.
///
/// ⚠️ Cả hai, vì Bẫy 3: `wal_autocheckpoint` và `busy_timeout` là trạng thái **của từng
/// kết nối**, không phải của database. Một bản chỉ đặt trên writer đi qua một ca chỉ
/// kiểm writer, rồi reader nhận `SQLITE_BUSY` ngay lập tức trong lúc TRUNCATE chạy.
#[test]
fn the_three_pragmas_read_back_on_every_connection() {
    let dir = temp_dir("pragmas");
    let tuning = Tuning {
        busy_timeout: Duration::from_millis(4321),
        ..quiet_tuning()
    };
    let store = Store::open(spec_with(&dir, tuning)).expect("mở kho");

    let (mode, autockpt, busy) = store
        .write(|tx| {
            let mode: String = tx.query_row("PRAGMA journal_mode", [], |r| r.get(0))?;
            let autockpt: i64 = tx.query_row("PRAGMA wal_autocheckpoint", [], |r| r.get(0))?;
            let busy: i64 = tx.query_row("PRAGMA busy_timeout", [], |r| r.get(0))?;
            Ok((mode, autockpt, busy))
        })
        .expect("đọc PRAGMA trên kết nối ghi");

    assert_eq!(mode.to_lowercase(), "wal", "kết nối ghi không ở WAL");
    assert_eq!(autockpt, 0, "autocheckpoint của SQLite chưa tắt — AD-12 nói thời điểm checkpoint là quyết định của ỨNG DỤNG");
    assert_eq!(busy, 4321, "busy_timeout trên kết nối ghi không khớp Tuning");

    let (mode, autockpt, busy, query_only) = store
        .read(|conn| {
            let mode: String = conn.query_row("PRAGMA journal_mode", [], |r| r.get(0))?;
            let autockpt: i64 = conn.query_row("PRAGMA wal_autocheckpoint", [], |r| r.get(0))?;
            let busy: i64 = conn.query_row("PRAGMA busy_timeout", [], |r| r.get(0))?;
            let query_only: i64 = conn.query_row("PRAGMA query_only", [], |r| r.get(0))?;
            Ok((mode, autockpt, busy, query_only))
        })
        .expect("đọc PRAGMA trên kết nối đọc");

    assert_eq!(mode.to_lowercase(), "wal", "kết nối đọc không thấy WAL");
    assert_eq!(autockpt, 0, "Bẫy 3: quên `wal_autocheckpoint` trên pool đọc");
    assert_eq!(busy, 4321, "Bẫy 3: quên `busy_timeout` trên pool đọc");
    assert_eq!(query_only, 1, "kết nối pool không đặt `query_only` — AC2 mất vế cưỡng chế");

    drop(store);
    cleanup(&dir);
}

/// **Ca 5** — `journal_mode` không đặt được ⇒ `open()` **trả Err**, ⛔ không đi tiếp.
///
/// 🔴 Đây là ca đối chứng âm của Bẫy 1, và nó là ca quan trọng nhất của AC3: một bản chỉ
/// *đặt* mà không *đọc lại* đi qua ca 4 (trên đĩa WAL bật được) và **xanh ở đây luôn** —
/// vì `pragma_update` trả `Ok(())` kể cả khi database ở lại `delete`.
///
/// `:memory:` là database WAL không dùng được — `journal_mode` đọc về `"memory"`.
#[test]
fn open_fails_when_wal_cannot_be_enabled() {
    let spec = StoreSpec {
        path: PathBuf::from(":memory:"),
        tuning: quiet_tuning(),
        ..StoreSpec::global(PathBuf::from(":memory:"))
    };

    let err = Store::open(spec).expect_err(
        "một database không dùng được WAL PHẢI làm `open()` trả Err. \
         Nhận `Ok` nghĩa là PRAGMA được đặt mà không ai đọc lại (Bẫy 1).",
    );

    match err {
        StoreError::WalUnavailable { store, mode } => {
            assert_eq!(store, StoreKind::Global);
            assert_ne!(
                mode.to_lowercase(),
                "wal",
                "biến thể lỗi nói WAL không dùng được nhưng chế độ đọc về lại là wal"
            );
        }
        other => panic!("kỳ vọng StoreError::WalUnavailable, nhận {other:?}"),
    }
}

// ═════════════════════════════════════════════════════════════════════════════════
// AC4 / AC5 — luồng checkpoint
// ═════════════════════════════════════════════════════════════════════════════════

/// Ghi `rounds` job, mỗi job một blob `blob_bytes` byte, cách nhau `gap`.
fn write_blobs(store: &Store, rounds: usize, blob_bytes: usize, gap: Duration) {
    for _ in 0..rounds {
        store
            .write(move |tx| {
                tx.execute(
                    "INSERT INTO bulk (payload) VALUES (?1)",
                    rusqlite::params![vec![0u8; blob_bytes]],
                )?;
                Ok(())
            })
            .expect("job ghi khối");
        if !gap.is_zero() {
            thread::sleep(gap);
        }
    }
}

/// Chờ tới khi `check` đúng, có **hạn chót**. Trả về `true` nếu đạt.
fn wait_until(deadline: Duration, mut check: impl FnMut() -> bool) -> bool {
    let stop = Instant::now() + deadline;
    while Instant::now() < stop {
        if check() {
            return true;
        }
        thread::sleep(Duration::from_millis(10));
    }
    check()
}

/// **Ca 6** — ghi rồi để rảnh quá `idle` ⇒ một lượt PASSIVE với `busy == 0` và
/// `checkpointed > 0`.
///
/// ⛔ **Ca này KHÔNG assert `.db-wal` nhỏ đi, và đó là một quyết định, không phải một
/// chỗ bỏ sót.** PASSIVE chép frame về database rồi cho SQLite **dùng lại** chỗ đó —
/// tệp giữ nguyên cỡ và **ngừng lớn**. Một assert "nhỏ đi" ở đây đỏ, và cách "sửa" tự
/// nhiên nhất là đổi luồng nền sang TRUNCATE — lúc đó test xanh, AC5 trông như đạt, và
/// AD-12 bị phá ở đúng chỗ nó tồn tại để bảo vệ (TRUNCATE **chặn**, chờ mọi reader rời
/// đi). Xem doc-comment của `core::store::checkpoint`.
#[test]
fn an_idle_pause_triggers_one_passive_checkpoint() {
    let dir = temp_dir("idle-passive");
    let tuning = Tuning {
        checkpoint_tick: Duration::from_millis(20),
        idle_before_passive: Duration::from_millis(60),
        wal_threshold_bytes: u64::MAX, // ⇒ chỉ điều kiện (a) rảnh mới kích hoạt được
        ..Tuning::default()
    };
    let store = Store::open(spec_with(&dir, tuning)).expect("mở kho");

    store
        .write(|tx| tx.execute_batch("CREATE TABLE bulk (id INTEGER PRIMARY KEY, payload BLOB)"))
        .expect("dựng bảng");
    write_blobs(&store, 20, 4096, Duration::ZERO);

    let reached = wait_until(Duration::from_secs(5), || {
        let s = store.checkpoint_stats();
        s.idle_triggered > 0 && s.frames_checkpointed > 0
    });

    let stats = store.checkpoint_stats();
    assert!(
        reached,
        "sau khi rảnh quá `idle_before_passive` mà không lượt PASSIVE nào chạy được: {stats:?}"
    );
    assert_eq!(
        stats.passive_busy, 0,
        "một lượt PASSIVE bị chặn (`busy != 0`) — ⛔ đó KHÔNG phải một lượt đã xong. \
         Chẩn đoán: {:?}",
        store.diagnostics()
    );
    assert_eq!(
        stats.threshold_triggered, 0,
        "ngưỡng đặt vô cực mà vẫn kích hoạt được ⇒ điều kiện (b) đang đọc sai"
    );
    assert_eq!(stats.errors, 0, "checkpoint lỗi: {:?}", store.diagnostics());

    drop(store);
    cleanup(&dir);
}

/// **Ca 7** — ghi liên tục cho `.db-wal` vượt ngưỡng, rồi ghi tiếp cùng lượng nữa.
///
/// 🔴 Đây là AC5 nguyên văn. `idle_before_passive` để **một giờ** có chủ ý: nếu vế (b)
/// không tồn tại thì không lượt checkpoint nào chạy được trong cả ca này, và `.db-wal`
/// sẽ lớn gấp đôi ở đợt hai. Một phiên gõ liên tục hàng giờ không bao giờ "rảnh" — đó
/// chính là ca mà AC5 nói tới.
///
/// Kỳ vọng là **chững lại**, ⛔ không phải co lại (xem ca 6).
#[test]
fn the_wal_stops_growing_once_it_crosses_the_threshold() {
    let dir = temp_dir("wal-threshold");
    const THRESHOLD: u64 = 64 * 1024;
    const ROUNDS: usize = 20;
    const BLOB: usize = 32 * 1024;

    // ⚠️ Ba con số dưới đây là **điều kiện để phép đo có nghĩa**, không phải sở thích:
    //
    // - `BLOB` lớn hơn nửa `THRESHOLD` ⇒ chỉ hai lượt ghi là WAL vượt ngưỡng. Không có
    //   điều đó thì ca này đo một cái ngưỡng chưa bao giờ chạm tới.
    // - `checkpoint_tick` **ngắn hơn hẳn** khoảng cách giữa hai lượt ghi ⇒ lượt checkpoint
    //   chép xong TRƯỚC lượt ghi kế tiếp. Đây là điều kiện để SQLite quay WAL về đầu tệp:
    //   `walRestartLog` chỉ chạy khi giao dịch ghi bắt đầu ở đúng lúc `nBackfill ==
    //   mxFrame`. Đảo lại (ghi dày, checkpoint thưa) thì mỗi lượt ghi bắt đầu trên một
    //   WAL còn tồn đọng, WAL không bao giờ quay đầu, và ca này đỏ vì một lý do **không
    //   phải** lỗi của cơ chế.
    // - `idle_before_passive` để một giờ ⇒ vế (a) tuyệt đối không kích hoạt được, nên mọi
    //   lượt checkpoint quan sát được ở đây đều là bằng chứng của vế (b).
    let tuning = Tuning {
        checkpoint_tick: Duration::from_millis(3),
        idle_before_passive: Duration::from_secs(3600),
        wal_threshold_bytes: THRESHOLD,
        ..Tuning::default()
    };
    let db = db_path(&dir);
    let store = Store::open(spec_with(&dir, tuning)).expect("mở kho");

    store
        .write(|tx| tx.execute_batch("CREATE TABLE bulk (id INTEGER PRIMARY KEY, payload BLOB)"))
        .expect("dựng bảng");

    let gap = Duration::from_millis(10);

    write_blobs(&store, ROUNDS, BLOB, gap);
    let crossed = wait_until(Duration::from_secs(5), || {
        store.checkpoint_stats().threshold_triggered > 0
    });
    thread::sleep(Duration::from_millis(100));
    let after_first = file_len(&sidecar(&db, "-wal"));

    write_blobs(&store, ROUNDS, BLOB, gap);
    thread::sleep(Duration::from_millis(100));
    let after_second = file_len(&sidecar(&db, "-wal"));

    let stats = store.checkpoint_stats();
    assert!(
        crossed,
        "`.db-wal` vượt ngưỡng {THRESHOLD} B mà không lượt checkpoint nào chạy TRƯỚC lúc \
         rảnh. Đó là AC5 trượt: một phiên gõ liên tục không bao giờ rảnh, nên chỉ có vế \
         (a) thì WAL phình vô hạn. Stats: {stats:?}"
    );
    assert_eq!(
        stats.idle_triggered, 0,
        "`idle_before_passive` đặt một giờ mà vế (a) vẫn kích hoạt ⇒ điều kiện rảnh đang đọc sai"
    );

    // ── Mệnh đề 1: CHỮNG LẠI ────────────────────────────────────────────────────
    // Đợt hai ghi đúng lượng bằng đợt một, nên một WAL "phình vô hạn" sẽ xấp xỉ gấp đôi.
    // ⛔ Đây KHÔNG phải chỗ đòi tệp co lại — PASSIVE chép frame rồi cho SQLite dùng lại
    // chỗ đó, tệp giữ nguyên cỡ và ngừng lớn. Xem ca 6.
    assert!(
        after_second <= after_first * 2,
        "`.db-wal` vẫn phình: {after_first} B sau đợt một, {after_second} B sau đợt hai \
         (cùng lượng ghi). Stats: {stats:?}"
    );

    // ── Mệnh đề 2: CÓ TRẦN ──────────────────────────────────────────────────────
    // Mệnh đề mạnh hơn và ít phụ thuộc nhịp hơn: tổng đã ghi là hai đợt, mà WAL phải giữ
    // ở gần ngưỡng. Không có cơ chế của AC5 thì WAL ≈ toàn bộ lượng đã ghi.
    let written = (2 * ROUNDS * BLOB) as u64;
    assert!(
        after_second < written / 4,
        "`.db-wal` đang giữ {after_second} B trong khi tổng đã ghi là {written} B — \
         tức nó lớn theo lượng ghi chứ không theo ngưỡng. Stats: {stats:?}"
    );

    drop(store);
    cleanup(&dir);
}

/// **Ca 8** — `close()` cắt `.db-wal` về 0 byte (hoặc xoá hẳn).
///
/// 🔴 **Chỉ TRUNCATE làm được điều này.** Ca này là phép kiểm duy nhất phân biệt được
/// một `close()` thật với một `close()` chỉ dừng luồng — và nó là nửa cơ chế của NFR10.
#[test]
fn close_truncates_the_wal_to_nothing() {
    let dir = temp_dir("close-truncate");
    let db = db_path(&dir);
    let store = Store::open(spec_with(&dir, quiet_tuning())).expect("mở kho");

    store
        .write(|tx| tx.execute_batch("CREATE TABLE bulk (id INTEGER PRIMARY KEY, payload BLOB)"))
        .expect("dựng bảng");
    write_blobs(&store, 40, 4096, Duration::ZERO);

    assert!(
        file_len(&sidecar(&db, "-wal")) > 0,
        "ca này vô nghĩa nếu `.db-wal` rỗng ngay từ đầu — kiểm lại rằng WAL thật sự bật"
    );

    store.close();

    let wal = sidecar(&db, "-wal");
    let len = file_len(&wal);
    assert!(
        !wal.exists() || len == 0,
        "`.db-wal` còn {len} byte sau `close()`. Chỉ `wal_checkpoint(TRUNCATE)` cắt được \
         tệp về 0; một `close()` chỉ dừng luồng thì để nguyên nó. Chẩn đoán: {:?}",
        store.diagnostics()
    );
    assert!(
        store.checkpoint_stats().truncate_runs >= 1,
        "không lượt TRUNCATE nào được ghi nhận lúc đóng"
    );

    drop(store);
    cleanup(&dir);
}

// ═════════════════════════════════════════════════════════════════════════════════
// AC6 / AC7 — phiên bản lược đồ
// ═════════════════════════════════════════════════════════════════════════════════

/// Bộ di trú **hai bước** để nghiệm thu AC6 vế *"đúng một bước chạy"* và vế sao lưu.
///
/// ⚠️ `GLOBAL_MIGRATIONS` hôm nay có **đúng một** bước, nên `target - 1 == 0` — mà 0 là
/// *"chưa có lược đồ"*, tức không có gì để sao lưu. Ca 10 của story vì thế **không thể**
/// nghiệm thu trên bộ di trú thật, và đó chính là lý do `StoreSpec::migrations` là một
/// trường chứ không phải một hằng tra theo `kind`.
static TWO_STEP: [Migration; 2] = [
    Migration {
        to_version: 1,
        sql: SCHEMA_MIGRATION_LOG_DDL,
    },
    Migration {
        to_version: 2,
        sql: "CREATE TABLE step_two (id INTEGER PRIMARY KEY);",
    },
];

/// Bộ di trú mà bước 2 **gãy giữa chừng**: câu đầu chạy được, câu sau tham chiếu một bảng
/// không tồn tại. Ca 11 dựa vào việc câu đầu ĐÃ chạy để chứng minh giao dịch rollback
/// thật, chứ không phải bước bị bỏ qua từ đầu.
static BROKEN_STEP_TWO: [Migration; 2] = [
    Migration {
        to_version: 1,
        sql: SCHEMA_MIGRATION_LOG_DDL,
    },
    Migration {
        to_version: 2,
        sql: "CREATE TABLE half_applied (id INTEGER PRIMARY KEY);\n\
              INSERT INTO table_that_does_not_exist (id) VALUES (1);",
    },
];

fn spec_with_migrations(dir: &Path, migrations: &'static [Migration]) -> StoreSpec {
    StoreSpec {
        migrations,
        ..spec_with(dir, quiet_tuning())
    }
}

/// **Ca 9** — database mới tinh (`user_version = 0`) di trú lên target và ghi sổ.
#[test]
fn a_fresh_database_migrates_up_to_target_and_logs_it() {
    let dir = temp_dir("fresh-migrate");
    let store = Store::open(spec_with(&dir, quiet_tuning())).expect("mở kho");

    assert_eq!(
        store.schema_version(),
        1,
        "`GLOBAL_MIGRATIONS` có một bước, nên một database mới phải kết thúc ở phiên bản 1"
    );

    let (rows, version, app_version, applied_at) = store
        .read(|conn| {
            let rows: i64 =
                conn.query_row("SELECT COUNT(*) FROM schema_migration_log", [], |r| r.get(0))?;
            let (version, app_version, applied_at): (i64, String, String) = conn.query_row(
                "SELECT version, app_version, applied_at FROM schema_migration_log",
                [],
                |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
            )?;
            Ok((rows, version, app_version, applied_at))
        })
        .expect("đọc sổ di trú");

    assert_eq!(rows, 1, "sổ di trú phải có đúng một bản ghi");
    assert_eq!(version, 1);
    assert_eq!(app_version, env!("CARGO_PKG_VERSION"));
    assert!(
        applied_at.ends_with('Z') && applied_at.contains('T') && applied_at.len() >= 20,
        "`applied_at` phải là ISO-8601 UTC (Consistency Conventions). Nhận: {applied_at}"
    );

    // `PRAGMA user_version` thật sự đã đổi, không chỉ trường trong bộ nhớ.
    let on_disk: i64 = store
        .read(|conn| conn.query_row("PRAGMA user_version", [], |r| r.get(0)))
        .expect("đọc user_version");
    assert_eq!(on_disk, 1);

    drop(store);
    cleanup(&dir);
}

/// **Ca 10** — `user_version = target - 1` ⇒ **đúng một** bước chạy, và tệp `.bak-v…` có
/// mặt.
#[test]
fn one_step_runs_and_a_backup_is_written_first() {
    let dir = temp_dir("one-step");
    let db = db_path(&dir);

    // Đưa kho tới phiên bản 1 rồi đóng hẳn.
    {
        let store = Store::open(spec_with_migrations(&dir, &TWO_STEP[..1])).expect("mở lần một");
        assert_eq!(store.schema_version(), 1);
    }

    let backup = dir.join("global.db.bak-v1");
    assert!(!backup.exists(), "chưa di trú mà đã có bản sao lưu");

    let store = Store::open(spec_with_migrations(&dir, &TWO_STEP)).expect("mở lần hai");
    assert_eq!(store.schema_version(), 2, "phải đi tiếp đúng một bước");

    assert!(
        backup.exists(),
        "AC6 đòi sao lưu TRƯỚC bước di trú đầu tiên. Không thấy {}",
        backup.display()
    );
    assert!(
        file_len(&backup) > 0,
        "bản sao lưu rỗng — `fs::copy` chạy trước khi `wal_checkpoint(TRUNCATE)` chép xong (Bẫy 5)"
    );

    let (rows, versions) = store
        .read(|conn| {
            let rows: i64 =
                conn.query_row("SELECT COUNT(*) FROM schema_migration_log", [], |r| r.get(0))?;
            let versions: i64 = conn.query_row(
                "SELECT COUNT(*) FROM schema_migration_log WHERE version = 2",
                [],
                |r| r.get(0),
            )?;
            Ok((rows, versions))
        })
        .expect("đọc sổ");
    assert_eq!(rows, 2, "sổ phải có bản ghi của cả hai bước");
    assert_eq!(versions, 1, "bước 2 phải được ghi đúng một lần");

    // Bảng của bước 2 tồn tại thật.
    store
        .read(|conn| conn.query_row("SELECT COUNT(*) FROM step_two", [], |r| r.get::<_, i64>(0)))
        .expect("bảng của bước 2 phải tồn tại");

    drop(store);
    let _ = db;
    cleanup(&dir);
}

/// **Ca 11** — một bước di trú ném lỗi giữa chừng ⇒ rollback, `user_version` **không đổi**.
///
/// ⚠️ Bước hỏng có **hai** câu, và câu đầu chạy được. Nếu giao dịch không thật, bảng
/// `half_applied` sẽ còn lại trên đĩa — nên ca này phân biệt được *"rollback thật"* với
/// *"bước bị bỏ qua ngay từ đầu"*.
#[test]
fn a_failing_migration_rolls_back_and_leaves_the_version_alone() {
    let dir = temp_dir("migration-rollback");

    {
        let store =
            Store::open(spec_with_migrations(&dir, &BROKEN_STEP_TWO[..1])).expect("mở lần một");
        assert_eq!(store.schema_version(), 1);
    }

    let err = Store::open(spec_with_migrations(&dir, &BROKEN_STEP_TWO))
        .expect_err("một bước di trú gãy PHẢI làm `open()` trả Err");
    match err {
        StoreError::OpenFailed { store, detail } => {
            assert_eq!(store, StoreKind::Global);
            assert!(
                detail.contains("migration 2"),
                "chẩn đoán phải nêu đích danh bước gãy. Nhận: {detail}"
            );
        }
        other => panic!("kỳ vọng StoreError::OpenFailed, nhận {other:?}"),
    }

    let store = Store::open(spec_with_migrations(&dir, &BROKEN_STEP_TWO[..1])).expect("mở lại");
    assert_eq!(
        store.schema_version(),
        1,
        "`user_version` đã đổi dù bước di trú trượt ⇒ `PRAGMA user_version` nằm ngoài giao dịch"
    );

    let (log_rows, half): (i64, i64) = store
        .read(|conn| {
            let log_rows: i64 =
                conn.query_row("SELECT COUNT(*) FROM schema_migration_log", [], |r| r.get(0))?;
            let half: i64 = conn.query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type = 'table' AND name = 'half_applied'",
                [],
                |r| r.get(0),
            )?;
            Ok((log_rows, half))
        })
        .expect("đọc sổ");

    assert_eq!(log_rows, 1, "sổ ghi thêm một bước chưa bao giờ commit");
    assert_eq!(
        half, 0,
        "bảng `half_applied` của câu ĐẦU trong bước hỏng vẫn còn ⇒ bước di trú KHÔNG chạy \
         trong một giao dịch"
    );

    drop(store);
    cleanup(&dir);
}

/// **Ca 12** — phiên bản mới hơn ứng dụng ⇒ `Err`, **byte-for-byte không đổi**, và
/// ⛔ `.db-wal` / `.db-shm` **không được tạo ra**.
///
/// 🔴 Đây là AC dễ trượt nhất của cả story, và nó trượt **im lặng**. Thứ tự tự nhiên
/// nhất để viết `open()` — mở, đặt PRAGMA cho xong, rồi mới xét lược đồ — vi phạm AC7,
/// vì `PRAGMA journal_mode = WAL` **ghi vào** database: nó viết lại header của tệp.
/// Không lỗi nào được ném; chỉ có một tệp khác đi.
///
/// ⚠️ Fixture dựng ở chế độ `delete` có chủ ý: đó là hình dạng cho phép khẳng định
/// *"không tệp sidecar nào được tạo"* một cách sạch sẽ.
#[test]
fn a_newer_schema_is_refused_without_touching_a_single_byte() {
    let dir = temp_dir("schema-too-new");
    let db = db_path(&dir);
    let target = GLOBAL_MIGRATIONS
        .last()
        .map(|m| m.to_version)
        .unwrap_or(0);

    {
        let conn = rusqlite::Connection::open(&db).expect("dựng fixture");
        conn.execute_batch(&format!(
            "PRAGMA journal_mode = delete;\n\
             CREATE TABLE from_the_future (id INTEGER PRIMARY KEY);\n\
             PRAGMA user_version = {};",
            target + 1
        ))
        .expect("ghi fixture");
    }

    let before = fs::read(&db).expect("đọc fixture");

    let err = Store::open(spec_with(&dir, quiet_tuning()))
        .expect_err("một lược đồ mới hơn PHẢI bị từ chối");
    match err {
        StoreError::SchemaTooNew {
            store,
            found,
            supported,
        } => {
            assert_eq!(store, StoreKind::Global);
            assert_eq!(found, target + 1);
            assert_eq!(supported, target);
        }
        other => panic!("kỳ vọng StoreError::SchemaTooNew, nhận {other:?}"),
    }

    let after = fs::read(&db).expect("đọc lại");
    assert_eq!(
        before.len(),
        after.len(),
        "cỡ tệp `.db` đổi sau một lần mở bị từ chối"
    );
    assert!(
        before == after,
        "tệp `.db` KHÁC ĐI byte-for-byte sau một lần mở bị từ chối. \
         Nghi phạm số một: `PRAGMA journal_mode = WAL` được đặt TRƯỚC khi đọc \
         `PRAGMA user_version` — nó viết lại header. Xem hợp đồng thứ tự trong `Store::open`."
    );

    for suffix in ["-wal", "-shm"] {
        let side = sidecar(&db, suffix);
        assert!(
            !side.exists(),
            "{} được tạo ra trong một lần mở đáng lẽ không chạm vào database",
            side.display()
        );
    }

    cleanup(&dir);
}

// ═════════════════════════════════════════════════════════════════════════════════
// Hình dạng lỗi — bàn giao cho Story 1.8 (Quyết định #6)
// ═════════════════════════════════════════════════════════════════════════════════

/// `From<StoreError> for IpcError` đi **qua `IpcError::new`** và mang đủ tham số.
///
/// Vì sao nghiệm thu hôm nay khi chưa gì hiển thị nó: đây là toàn bộ phần story này đóng
/// góp cho món nợ `ipc_error_wire_shape` (`deferred-work.md:38-40`). Story 1.8 chỉ phải
/// **nối dây**; nếu bảng tham số ở đây sai thì nó sai từ hôm nay, không phải từ hôm đó.
///
/// ⚠️ Ca này cũng là lưới cho `IpcError::new`: khoá thiếu tham số rơi về
/// `MessageKey::Unknown` ở release và `debug_assert!` nổ ở debug — `cargo test` chạy ở
/// debug, nên một `params` thiếu làm ca này panic chứ không lặng lẽ đổi khoá.
#[test]
fn every_store_error_converts_to_a_complete_ipc_error() {
    let cases: Vec<(StoreError, MessageKey, &str)> = vec![
        (
            StoreError::OpenFailed {
                store: StoreKind::Global,
                detail: "disk on fire".to_owned(),
            },
            MessageKey::StoreOpenFailed,
            "store.open_failed",
        ),
        (
            StoreError::WalUnavailable {
                store: StoreKind::Global,
                mode: "delete".to_owned(),
            },
            MessageKey::StoreWalUnavailable,
            "store.wal_unavailable",
        ),
        (
            StoreError::SchemaTooNew {
                store: StoreKind::Global,
                found: 9,
                supported: 1,
            },
            MessageKey::StoreSchemaTooNew,
            "store.schema_too_new",
        ),
        (
            StoreError::WriteFailed {
                store: StoreKind::Global,
                detail: "constraint".to_owned(),
            },
            MessageKey::StoreWriteFailed,
            "store.write_failed",
        ),
        (
            StoreError::WriterGone {
                store: StoreKind::Global,
            },
            MessageKey::StoreWriteFailed,
            "store.writer_gone",
        ),
        (
            StoreError::ReadFailed {
                store: StoreKind::Global,
                detail: "readonly".to_owned(),
            },
            MessageKey::StoreReadFailed,
            "store.read_failed",
        ),
        (
            StoreError::PoolClosed {
                store: StoreKind::Global,
            },
            MessageKey::StoreReadFailed,
            "store.pool_closed",
        ),
    ];

    for (err, expected_key, expected_code) in cases {
        let debug = format!("{err:?}");
        let ipc: IpcError = err.into();

        assert_eq!(ipc.code(), expected_code, "code sai cho {debug}");
        assert_eq!(
            ipc.message_key(),
            expected_key,
            "message_key rơi về `Unknown` cho {debug} ⇒ `IpcError::new` thấy thiếu tham số"
        );

        let declared: Vec<&str> = expected_key.required_params().to_vec();
        let given: Vec<&str> = ipc.params().keys().map(String::as_str).collect();
        for name in &declared {
            assert!(
                given.contains(name),
                "{debug} → thiếu tham số `{name}`; khoá đòi {declared:?}, nhận {given:?}"
            );
        }

        // ⛔ Không câu nào lọt vào `params`. AD-21: tham số mang DỮ LIỆU. `detail` mang
        // văn bản lỗi thô của SQLite và nó phải ở lại trong `Debug`, không lên dây.
        let params: &BTreeMap<String, String> = ipc.params();
        for (key, value) in params {
            assert!(
                !value.contains(' ') || key == "mode",
                "tham số `{key}` mang một câu (`{value}`) — AD-21 chỉ cho phép dữ liệu"
            );
        }
    }
}

/// Tên kho đi vào `params` là **định danh máy đọc**, ổn định qua mọi lần sửa lời văn.
#[test]
fn store_kind_names_are_stable_machine_identifiers() {
    assert_eq!(StoreKind::Global.as_str(), "global");
    assert_eq!(StoreKind::Project.as_str(), "project");
    assert_eq!(StoreKind::LibraryIndex.as_str(), "library-index");
}
