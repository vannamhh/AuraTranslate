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
//!    thành flaky. Tên gồm pid + một bộ đếm nguyên tử. Không thêm `tempfile`.
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
    GLOBAL_MIGRATIONS, Migration, SCHEMA_MIGRATION_LOG_DDL, SqlResult, Store, StoreError,
    StoreKind, StoreSpec, Tuning,
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
/// không bằng suy luận (AC1 nói nguyên văn như vậy):
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
/// `PRAGMA query_only = 1`, tức từ **SQLite**, không từ việc người viết tự nhớ.
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

/// **Ca 5** — `journal_mode` không đặt được ⇒ `open()` **trả Err**, không đi tiếp.
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
/// **Ca này KHÔNG assert `.db-wal` nhỏ đi, và đó là một quyết định, không phải một
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
        "một lượt PASSIVE bị chặn (`busy != 0`) — không đó KHÔNG phải một lượt đã xong. \
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
/// Kỳ vọng là **chững lại**, không phải co lại (xem ca 6).
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

    // ═════════════════════════════════════════════════════════════════════════════
    // 🔵 CODE REVIEW 2026-08-19 — HAI MỆNH ĐỀ ĐỔI CHỖ, và thứ tự LÀ một mệnh đề
    // ═════════════════════════════════════════════════════════════════════════════
    // Bản trước khẳng định *"chững lại"* TRƯỚC *"có trần"*. Hậu quả đo được: lượt CI
    // `32212786258` trên `macos-26` panic ở *"chững lại"*, nên *"có trần"* — mệnh đề **mạnh
    // hơn và ít phụ thuộc nhịp hơn**, theo đúng chữ của chú thích 2026-08-11 ngay dưới —
    // **không bao giờ được đánh giá**. Người đọc lượt đỏ ấy không có đường nào biết bảo đảm
    // thật còn đứng hay không; phải tính bằng tay mới thấy nó đứng, và đứng thoải mái
    // *(210.152 / 327.680 = 64% trần)*.
    //
    // 🔴 ⇒ Mệnh đề YẾU hơn đứng trước sẽ CHE mệnh đề mạnh ở **mọi** lượt đỏ. Thứ tự ở đây
    // không phải gu trình bày — nó quyết định một lượt đỏ nói ra được điều gì. Trần đi trước.
    //
    // ── Mệnh đề 2: CÓ TRẦN ──────────────────────────────────────────────────────
    // Mệnh đề mạnh hơn và ít phụ thuộc nhịp hơn: tổng đã ghi là hai đợt, mà WAL phải giữ
    // ở gần ngưỡng. Không có cơ chế của AC5 thì WAL ≈ toàn bộ lượng đã ghi.
    //
    // ═════════════════════════════════════════════════════════════════════════════
    // 🔴 TRẦN NỚI THEO NỀN TẢNG — Ice chốt 2026-08-11
    // ═════════════════════════════════════════════════════════════════════════════
    // Ca này đỏ trên `windows-2025` ngay lượt CI đầu tiên chạy được tới nó
    // (`31469843146`, sau bản vá `STATUS_ENTRYPOINT_NOT_FOUND` — trước đó nhị phân test
    // tích hợp chết ở khâu NẠP nên ca này chưa từng chạy trên Windows).
    //
    // Số đo, và đọc nó cho đúng vì hai mệnh đề của ca này KHÔNG cùng phán quyết:
    //   `.db-wal` = 889.952 B · tổng đã ghi = 1.310.720 B · trần cũ = 327.680 B
    //   CheckpointStats { threshold_triggered: 51, frames_checkpointed: 6392,
    //                     passive_busy: 0, idle_triggered: 0, errors: 0 }
    // - Mệnh đề 1 (*"chững lại"*) **ĐẠT**. Cơ chế AC5 CÓ chạy: 51 lượt theo ngưỡng,
    //   6.392 frame đã chép, 0 lượt bị chặn, 0 lỗi, và `idle_triggered = 0` chứng minh
    //   vế (a) không hề kích hoạt.
    // - Chỉ mệnh đề 2 trượt: WAL đứng ở 67,9% lượng đã ghi thay vì dưới 25%.
    //
    // Nguyên nhân là `walRestartLog` của SQLite — nó chỉ quay WAL về đầu tệp khi một
    // giao dịch ghi bắt đầu đúng lúc `nBackfill == mxFrame`. Trên Windows nhịp đó không
    // rơi vào nhau; frame vẫn được chép, tệp không bao giờ quay đầu.
    //
    // ⚠️ Trần nới THEO NỀN TẢNG, KHÔNG nới toàn cục. Hạ trần chung xuống 3/4 sẽ vứt luôn
    // bảo đảm chặt của macOS — nền tảng Ice phát triển hằng ngày — cho một khác biệt chỉ
    // tồn tại ở nền tảng kia. Hằng dưới đây nói ra sự khác biệt đó thay vì giấu nó.
    //
    // ⚠️ **Trần Windows hiệu chuẩn trên ĐÚNG MỘT phép đo (n = 1).** 3/4 nằm giữa số đo
    // (67,9%) và ngưỡng của *"cơ chế vắng mặt"* (≈100%), gần số đo hơn để nó còn bắt được
    // hồi quy. Nếu một lượt CI sau vượt 75%, ĐỪNG nới tiếp theo phản xạ: hai số in ở dưới
    // có mặt để lượt đó có dữ liệu thật mà cãi. Đường đóng thật sự là đo trên một máy
    // Windows — món nợ A5 của retrospective Epic 1.
    const WAL_CEILING_NUM: u64 = if cfg!(windows) { 3 } else { 1 };
    const WAL_CEILING_DEN: u64 = 4;

    let written = (2 * ROUNDS * BLOB) as u64;
    let ceiling = written * WAL_CEILING_NUM / WAL_CEILING_DEN;

    // ⚠️ `cargo test` NUỐT stdout của ca xanh, nên dòng dưới chỉ hiện khi ca này đỏ hoặc
    // khi chạy `cargo test --test store_contract -- --nocapture`. Đó là cách lấy điểm đo
    // thứ hai cho trần Windows mà KHÔNG phải chờ một lượt đỏ — ghi ở đây thay vì để người
    // sau tự tìm ra.
    println!(
        "\n  WAL: {after_first} B sau đợt một -> {after_second} B sau đợt hai · tổng đã \
         ghi {written} B · trần {ceiling} B ({WAL_CEILING_NUM}/{WAL_CEILING_DEN}) · \
         {:.1}% lượng ghi",
        (after_second as f64 / written as f64) * 100.0
    );

    assert!(
        after_second < ceiling,
        "`.db-wal` đang giữ {after_second} B (đợt một: {after_first} B) trong khi tổng đã \
         ghi là {written} B, trần {ceiling} B = {WAL_CEILING_NUM}/{WAL_CEILING_DEN} — tức \
         nó lớn theo lượng ghi chứ không theo ngưỡng.\n\n\
         ĐỪNG nới trần theo phản xạ: mệnh đề 1 (*chững lại*) ở ngay DƯỚI đã xanh hay chưa, \
         và `threshold_triggered`/`frames_checkpointed` dưới đây nói cơ chế có chạy hay \
         không. Hai câu đó phân biệt *một hồi quy của tầng Store* với *một trần hiệu chuẩn \
         sai*. Stats: {stats:?}"
    );

    // ── Mệnh đề 1: CHỮNG LẠI ────────────────────────────────────────────────────
    //
    // Đây KHÔNG phải chỗ đòi tệp co lại — PASSIVE chép frame rồi cho SQLite dùng lại chỗ đó,
    // tệp giữ nguyên cỡ và **ngừng lớn**. Xem ca 8.
    //
    // ═════════════════════════════════════════════════════════════════════════════
    // 🔴 HÌNH DẠNG ĐỔI 2026-08-19 — phép so CŨ PHẠT chính cơ chế nó đo
    // ═════════════════════════════════════════════════════════════════════════════
    // Bản cũ: `after_second <= after_first * 2`. Nó **tự tham chiếu**, và đó là khuyết tật:
    // `after_first` được chụp **đúng lúc cơ chế phản ứng lần đầu** *(ngay sau khi
    // `threshold_triggered > 0`)*, nên nó nằm sát `THRESHOLD`. ⇒ **Cơ chế càng phản ứng
    // nhanh, `after_first` càng nhỏ, và trần `×2` càng NGẶT.** Một lượt vá làm checkpoint
    // nhanh hơn sẽ làm ca này ĐỎ. Không cổng nào bắt được kiểu ngược đời đó.
    //
    // ⚠️ **Đo, hai máy, cùng mã, cùng `after_first` = 94.792 B từng byte:**
    //   · máy Ice (macOS): đợt hai -> **94.792 B** — WAL quay đầu trọn vẹn, lớn thêm **0 B**;
    //   · `macos-26` CI:   đợt hai -> **210.152 B** — lớn thêm **115.360 B**.
    //   Cả hai đều dưới trần của mệnh đề 2. Khác biệt là `walRestartLog` của SQLite: nó chỉ
    //   quay WAL về đầu tệp khi một giao dịch ghi bắt đầu đúng lúc `nBackfill == mxFrame`, và
    //   nhịp đó rơi vào nhau hay không là chuyện của MÁY — chú thích 2026-08-11 ngay trên đã
    //   ghi đúng hiện tượng này cho `windows-2025`.
    //
    // ⇒ Phép so mới đo **đợt hai cộng thêm bao nhiêu so với lượng nó GHI** — một hằng số, không
    // một mẫu của chính nó. Không cơ chế nào thì đợt hai cộng gần đủ `ROUND_BYTES`; có cơ chế
    // thì nó cộng một phần nhỏ. Hai điểm đo: **0%** (máy Ice) và **17,6%** (CI).
    //
    // 🔴 **Trần 1/4 KHÔNG phải một lượt nới** — nó là một trục KHÁC. Trần cũ ràng `after_second`
    // vào `after_first`; trần này ràng **mức LỚN THÊM** vào lượng ghi. Mệnh đề 2 *(trần theo
    // tổng lượng ghi)* **không bị chạm một chữ**, và nó vẫn là phép kiểm chặt nhất của ca này.
    // Sức răn còn nguyên: không có cơ chế AC5, mức lớn thêm ≈ `ROUND_BYTES`, tức **4× vượt**.
    //
    // ⚠️ **n = 2 máy.** Hai điểm đo không vẽ được một phân bố. 1/4 nằm giữa số đo cao nhất
    // (17,6%) và ngưỡng *"cơ chế vắng mặt"* (≈100%), gần số đo hơn để nó còn bắt được hồi quy —
    // cùng lý lẽ và cùng tỉ lệ mà Ice đã ký cho mệnh đề 2 ngày 2026-08-11.
    const GROWTH_NUM: u64 = 1;
    const GROWTH_DEN: u64 = 4;
    let round_bytes = (ROUNDS * BLOB) as u64;
    let growth = after_second.saturating_sub(after_first);
    let growth_cap = round_bytes * GROWTH_NUM / GROWTH_DEN;
    assert!(
        growth <= growth_cap,
        "`.db-wal` vẫn phình: đợt hai cộng thêm {growth} B trong khi nó chỉ ghi {round_bytes} B \
         — trần {growth_cap} B ({GROWTH_NUM}/{GROWTH_DEN}). {after_first} B sau đợt một, \
         {after_second} B sau đợt hai.\n\n\
         ĐỌC HAI CÂU NÀY TRƯỚC KHI NỚI: mệnh đề 2 (*có trần*) ở ngay TRÊN đã xanh — nếu nó \
         xanh thì WAL vẫn bị chặn theo ngưỡng, và chỗ hỏng nằm ở `walRestartLog` không rơi \
         nhịp, KHÔNG ở tầng Store. Và `threshold_triggered`/`frames_checkpointed` dưới đây nói \
         cơ chế có chạy hay không. Stats: {stats:?}"
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
/// ⚠️ **Cập nhật Story 1.8.** Câu trước đây ở chỗ này — *"`GLOBAL_MIGRATIONS` hôm nay có
/// đúng một bước, nên `target - 1 == 0`… Ca 10 vì thế không thể nghiệm thu trên bộ di trú
/// thật"* — đã **thôi đúng**: bước 2 (`CONFIG_VALUE_DDL`) làm `target - 1 == 1`, tức Ca 10
/// nay nghiệm thu được trên bộ di trú thật.
///
/// **Nhưng `TWO_STEP` và `spec_with_migrations` GIỮ NGUYÊN**, và lý do không đổi một
/// chữ nào: `StoreSpec.migrations` là một **trường** chứ không phải một hằng tra theo
/// `kind` (Story 1.7 §Completion Notes #2), và fixture cục bộ là cách duy nhất nghiệm thu
/// AC6 vế *"một bước gãy giữa chừng ⇒ rollback"* (`BROKEN_STEP_TWO`) mà **không** phải
/// thêm mã sản phẩm chỉ để test gọi. Story 1.15 dùng đúng trường đó cho `project.db`.
///
/// 🔴 Ba ca dưới đây chạy trên fixture cục bộ, **không** phụ thuộc `GLOBAL_MIGRATIONS`.
/// Sửa các con số của chúng cho *"nhất quán"* với bộ di trú thật là hướng hỏng ĐẮT: hai ca
/// nghiệm thu **sao lưu trước khi di trú** và **rollback khi bước gãy** im lặng mất hiệu
/// lực, và CI vẫn xanh.
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
///
/// 🔴 Ca **DUY NHẤT** ở tệp này chạy trên `GLOBAL_MIGRATIONS` THẬT (qua `spec_with`), nên
/// nó là ca duy nhất phải đổi khi một story thêm một bước di trú. Đó là công việc của nó,
/// không phải một phiền nhiễu: nó canh rằng số phiên bản đổi là một quyết định **có người
/// ký**, chứ không phải một hiệu ứng phụ của một lượt sửa lược đồ.
///
/// ⚠️ Cập nhật Story 1.8: bước 2 thêm bảng `config_value` ⇒ target là **2**.
///
/// 🔴 **Cập nhật Story 1.20 (2026-08-11): bước 3 thêm bảng `pinned_entry` ⇒ target là 3.**
/// Và lượt đỏ này là bằng chứng chạy được cho chính doc-comment ở trên: bản đầu của story
/// đặt bảng ghim vào `PROJECT_MIGRATIONS`, nên ca này **KHÔNG** đỏ — story đã ghi tiền đề
/// *"ca này sẽ đỏ"* mà không kiểm, và phép đo bắt được. Lượt Ice ký lại chuyển bảng sang
/// `global.db`, và nay nó đỏ **đúng như** cơ chế được thiết kế để đỏ.
///
/// 🔵 **CẬP NHẬT 2026-08-19 (Story 3.1): bước 4 thêm bảng `glossary_entry` (tầng Global của
/// Glossary, AD-18/AD-36) ⇒ target là 4.** Mệnh đề *"ba bước, đích là 3"* đã hết đúng, sửa
/// tại chỗ thay vì để nó lặng lẽ sai — đúng cơ chế mà doc-comment ở trên nói câu này ĐƯỢC
/// THIẾT KẾ để đỏ mỗi lần một story thêm một bước.
///
/// 🔵 **CẬP NHẬT 2026-08-24 (Story 3.10): bước 5 dựng lại `glossary_entry` để thêm giá trị
/// `term_origin` thứ tư, `file_import` (FR49/NFR9) ⇒ target là 5.** Câu *"bốn bước, đích là
/// 4"* đã hết đúng, sửa tại chỗ — đúng cơ chế được thiết kế để đỏ.
///
/// 🔵 **CẬP NHẬT 2026-08-27 (phán quyết Ice #1, Story 5.3): bước 6 thêm bảng `library_orphan`
/// (cờ mồ côi của Library chuyển từ `library-index.db` sang `global.db`) ⇒ target là 6.**
/// Câu *"năm bước, đích là 5"* đã hết đúng, sửa tại chỗ — đúng cơ chế được thiết kế để đỏ.
///
/// 🔵 **CẬP NHẬT 2026-09-05 (Story 6.5): bước 7 thêm bảng `import_cleanup_rule` (tầng Global
/// của luật làm sạch, AD-18/FR124) ⇒ target là 7.** Câu *"sáu bước, đích là 6"* đã hết đúng,
/// sửa tại chỗ — đúng cơ chế được thiết kế để đỏ.
#[test]
fn a_fresh_database_migrates_up_to_target_and_logs_it() {
    let dir = temp_dir("fresh-migrate");
    let store = Store::open(spec_with(&dir, quiet_tuning())).expect("mở kho");

    assert_eq!(
        store.schema_version(),
        7,
        "`GLOBAL_MIGRATIONS` có bảy bước (Story 1.7 sổ di trú · Story 1.8 `config_value` · \
         Story 1.20 `pinned_entry` · Story 3.1 `glossary_entry` · Story 3.10 gia tri \
         term_origin thu tu · phan quyet Ice #1 bang library_orphan · Story 6.5 bang \
         import_cleanup_rule), nên một database mới phải kết thúc ở phiên bản 7"
    );

    let (rows, versions, app_version, applied_at) = store
        .read(|conn| {
            let rows: i64 =
                conn.query_row("SELECT COUNT(*) FROM schema_migration_log", [], |r| r.get(0))?;
            let mut stmt =
                conn.prepare("SELECT version FROM schema_migration_log ORDER BY version")?;
            let versions: Vec<i64> = stmt
                .query_map([], |r| r.get(0))?
                .collect::<SqlResult<Vec<i64>>>()?;
            let (app_version, applied_at): (String, String) = conn.query_row(
                "SELECT app_version, applied_at FROM schema_migration_log WHERE version = 1",
                [],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )?;
            Ok((rows, versions, app_version, applied_at))
        })
        .expect("đọc sổ di trú");

    assert_eq!(rows, 7, "sổ di trú phải có đúng một bản ghi cho MỖI bước");
    assert_eq!(
        versions,
        vec![1, 2, 3, 4, 5, 6, 7],
        "cả bảy bước phải có mặt trong sổ — một bước chạy mà không ghi sổ là đúng ca \
         *sổ nói chưa chạy mà lược đồ thì đã*"
    );
    assert_eq!(app_version, env!("CARGO_PKG_VERSION"));
    assert!(
        applied_at.ends_with('Z') && applied_at.contains('T') && applied_at.len() >= 20,
        "`applied_at` phải là ISO-8601 UTC (Consistency Conventions). Nhận: {applied_at}"
    );

    // Bước 2 thật sự đã dựng bảng, không chỉ tăng số phiên bản.
    let config_table: i64 = store
        .read(|conn| {
            conn.query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type = 'table' AND name = 'config_value'",
                [],
                |r| r.get(0),
            )
        })
        .expect("đọc sqlite_master");
    assert_eq!(
        config_table, 1,
        "bước 2 phải dựng bảng `config_value` — một `user_version = 2` mà không có bảng là \
         một lược đồ nói dối"
    );

    // Bước 3 (Story 1.20) — cùng luật, cùng lý do.
    let pinned_table: i64 = store
        .read(|conn| {
            conn.query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type = 'table' AND name = 'pinned_entry'",
                [],
                |r| r.get(0),
            )
        })
        .expect("đọc sqlite_master");
    assert_eq!(
        pinned_table, 1,
        "bước 3 phải dựng bảng `pinned_entry` — một `user_version = 3` mà không có bảng là \
         một lược đồ nói dối"
    );

    // Bước 4 (Story 3.1) — cùng luật, cùng lý do.
    let glossary_table: i64 = store
        .read(|conn| {
            conn.query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type = 'table' AND name = 'glossary_entry'",
                [],
                |r| r.get(0),
            )
        })
        .expect("đọc sqlite_master");
    assert_eq!(
        glossary_table, 1,
        "bước 4 phải dựng bảng `glossary_entry` — một `user_version = 4` mà không có bảng là \
         một lược đồ nói dối"
    );

    // Bước 5 (Story 3.10) — cùng luật, cùng lý do.
    let file_import_step: i64 = store
        .read(|conn| {
            conn.query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type = 'trigger' \
                 AND name = 'glossary_entry_lifecycle_is_one_way'",
                [],
                |r| r.get(0),
            )
        })
        .expect("đọc sqlite_master");
    assert_eq!(
        file_import_step, 1,
        "bước 5 dựng lại `glossary_entry` phải tạo lại trigger một chiều — thiếu nó là AD-36 \
         chết trong im lặng"
    );

    // `PRAGMA user_version` thật sự đã đổi, không chỉ trường trong bộ nhớ.
    let on_disk: i64 = store
        .read(|conn| conn.query_row("PRAGMA user_version", [], |r| r.get(0)))
        .expect("đọc user_version");
    // 🔵 5 → 6 (2026-08-27, phán quyết Ice #1, Story 5.3) — bước 6 thêm `library_orphan`.
    // 🔵 6 → 7 (2026-09-05, Story 6.5) — bước 7 thêm `import_cleanup_rule`.
    assert_eq!(on_disk, 7);

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
/// `.db-wal` / `.db-shm` **không được tạo ra**.
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

        // Không câu nào lọt vào `params`. AD-21: tham số mang DỮ LIỆU. `detail` mang
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
    assert_eq!(StoreKind::Dict.as_str(), "dict");
}

// ═════════════════════════════════════════════════════════════════════════════════
// Story 2.3 · AC5 · Task 4.4 — VẾ "CHỈ XONG SAU KHI ĐÃ GHI VÀO WAL" CỦA AD-35
// ═════════════════════════════════════════════════════════════════════════════════

/// AD-35 nói flush *"chỉ được coi là xong **sau khi đã ghi vào WAL** — nếu chỉ vào hàng đợi
/// trong bộ nhớ thì ngưỡng 5 giây của NFR18 không bảo đảm gì"*.
///
/// `Store::write` **chặn** tới khi job chạy xong và mỗi job là một giao dịch, nên nửa
/// *"không phải khi mới vào hàng đợi trong bộ nhớ"* đã đứng từ Story 1.7. Nửa còn lại —
/// *"đã ghi vào WAL"* — phụ thuộc **`PRAGMA synchronous`**:
///
/// | `synchronous` | commit trên WAL |
/// |---|---|
/// | `2` FULL | `fsync` WAL ở **mỗi** commit ⇒ `Ok` **là** bằng chứng đã ghi |
/// | `1` NORMAL | **không** `fsync`; WAL có thể mất khi mất điện ⇒ AC5 **chưa thoả** |
///
/// 🔴 Ca này tồn tại vì *"mặc định biên dịch của SQLite là FULL"* là một mệnh đề về **giá
/// trị mặc định của một thư viện C được ghim**, không phải một lời khai trong mã của dự án
/// này. Cùng luật *"đặt rồi ĐỌC LẠI"* mà [`StoreError::WalUnavailable`] tồn tại để dạy: một
/// lượt nâng `libsqlite3-sys`, hay một cờ biên dịch đổi, hạ nó xuống NORMAL **im lặng** và
/// NFR18 mất bảo đảm mà không cổng nào đỏ.
///
/// 🔵 **ĐÍNH CHÍNH 2026-08-13 (code review) — nay `pragmas.rs` ĐẶT nó, và ca này vẫn ĐỌC LẠI.**
///
/// Bản đầu của ca này ghi *"đọc, không đặt"* và giao lượt đặt cho **Story 2.4**, với lý lẽ
/// rằng con số đánh đổi với NFR2. Lý lẽ đó đúng cho việc **đổi** giá trị, nhưng nó không phủ
/// việc **ghim** giá trị đang chạy: phép đo cho ra `2 (FULL)`, nên `apply_writer_pragmas` đặt
/// đúng `FULL` ⇒ hành vi đổi **0**, hiệu năng đổi **0**, và Story 2.4 vẫn tự do hiệu chỉnh.
/// Cái đổi là AD-35 thôi đứng trên một **mặc định biên dịch của một thư viện C được ghim** và
/// bắt đầu đứng trên một lời khai của chương trình này — nửa còn thiếu của chính luật *"đặt
/// rồi ĐỌC LẠI"*.
///
/// ⚠️ Ca này vì thế **không** trở nên thừa: nó là vế **đọc lại**. Một lượt nâng
/// `libsqlite3-sys` làm `FULL` không đặt được nữa sẽ đỏ **ở đây**, không im lặng.
#[test]
fn the_write_connection_fsyncs_the_wal_on_every_commit() {
    let dir = temp_dir("synchronous");
    let store = Store::open(spec_with(&dir, quiet_tuning())).expect("mở kho");

    let (sync_writer, mode) = store
        .write(|tx| {
            let sync: i64 = tx.query_row("PRAGMA synchronous", [], |r| r.get(0))?;
            let mode: String = tx.query_row("PRAGMA journal_mode", [], |r| r.get(0))?;
            Ok((sync, mode))
        })
        .expect("đọc PRAGMA synchronous trên kết nối ghi");

    assert_eq!(
        mode.to_lowercase(),
        "wal",
        "ca này chỉ có nghĩa trên WAL — `synchronous` mang ngữ nghĩa khác ở journal khác"
    );
    assert_eq!(
        sync_writer, 2,
        "PRAGMA synchronous = {sync_writer} (2 = FULL, 1 = NORMAL). Ở NORMAL, một commit \
         trên WAL KHÔNG fsync, nên `Store::write` trả Ok TRƯỚC khi dữ liệu chạm đĩa — và \
         AC5 của Story 2.3 (AD-35) CHƯA THOẢ. Đây là một phát hiện phải BÁO, không phải \
         một chỗ để chèn một PRAGMA: con số này đánh đổi với NFR2 và chủ là Story 2.4."
    );

    drop(store);
    cleanup(&dir);
}
