//! Luồng nền trên **kết nối riêng**: PASSIVE khi rảnh hoặc quá ngưỡng, TRUNCATE lúc
//! đóng — AD-12, AC4, AC5.
//!
//! ─────────────────────────────────────────────────────────────────────────────
//! 🔴 PASSIVE **KHÔNG** LÀM `.db-wal` NHỎ ĐI — VÀ "SỬA" ĐIỀU ĐÓ LÀ PHÁ CHÍNH AD-12
//! ─────────────────────────────────────────────────────────────────────────────
//! Một lượt checkpoint chép frame từ WAL về database rồi **quay đầu đọc/ghi WAL về đầu
//! tệp để dùng lại**. Tệp `.db-wal` **giữ nguyên cỡ**; nó chỉ **ngừng lớn**. Đó chính là
//! thứ AC5 đòi — *"không phình vô hạn"*, không phải *"co lại"*.
//!
//! Đường hỏng cụ thể, và nó rất dễ đi vào: viết một test assert `.db-wal` nhỏ đi → đỏ →
//! kết luận *"PASSIVE không chạy"* → đổi luồng nền sang TRUNCATE cho xanh. Lúc đó test
//! xanh, AC5 **trông như** đạt, và AD-12 bị vi phạm ở đúng chỗ nó tồn tại để bảo vệ:
//! TRUNCATE **chờ mọi reader rời đi**, nên nó là lượt checkpoint duy nhất có thể **chặn**
//! — đặt nó vào đường chạy nền là dựng lại đúng cái gai trễ mà `wal_autocheckpoint = 0`
//! vừa gỡ ra, và NFR2 mất hiệu lực. Không test nào đỏ, không lỗi nào được ném.
//!
//! → **PASSIVE ở đường nền; TRUNCATE chỉ ở [`Checkpointer::shutdown`] và ngay trước khi
//!   sao lưu để di trú.** Bằng chứng của một lượt PASSIVE là `checkpointed > 0` với
//!   `busy == 0`, **không phải cỡ tệp**.
//!
//! ⚠️ Kéo theo: WAL chỉ được dùng lại khi một lượt checkpoint chép **hết**. Một reader
//! giữ ảnh chụp cũ làm `log > checkpointed`, và tệp **vẫn lớn tiếp** — đó là lý do
//! `Tuning::pool_size` để nhỏ.
//!
//! ─────────────────────────────────────────────────────────────────────────────
//! HAI ĐIỀU KIỆN KÍCH HOẠT, VÀ CHÚNG **KHÔNG** TƯƠNG ĐƯƠNG NHAU
//! ─────────────────────────────────────────────────────────────────────────────
//! - **(a) rảnh** — đã qua `idle_before_passive` kể từ job ghi cuối, **và** có gì đó để
//!   chép (`dirty`). Đây là đường bình thường: người dùng ngừng gõ, ứng dụng dọn dẹp.
//! - **(b) quá ngưỡng** — `.db-wal` vượt `wal_threshold_bytes`. Vế này chạy **kể cả khi
//!   chưa rảnh**, và đó là toàn bộ AC5: một phiên gõ liên tục hàng giờ không bao giờ
//!   "rảnh", nên nếu chỉ có vế (a) thì `.db-wal` phình vô hạn đúng theo định nghĩa.

use std::collections::VecDeque;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Condvar, Mutex, MutexGuard};
use std::thread::JoinHandle;
use std::time::{Duration, Instant};

use rusqlite::Connection;

use super::{StoreError, StoreKind, Tuning, pragmas, wal_len};

/// Trần số dòng chẩn đoán giữ lại. Vòng — dòng cũ nhất bị đẩy ra.
///
/// ⚠️ Có trần chứ không phải một `Vec` lớn dần: một `busy != 0` lặp lại mỗi tick trong
/// một phiên nhiều giờ là một rò rỉ bộ nhớ chậm, và nó rò ở đúng tiến trình mà NFR2 nói
/// về độ trễ.
const DIAGNOSTICS_CAP: usize = 64;

/// Số đếm của luồng checkpoint — bề mặt nghiệm thu của AC4 và AC5.
///
/// AC4 đòi kết quả `(busy, log, checkpointed)` được **đọc và xét**, không vứt đi. Một số
/// đã đọc mà không ai đọc được thì bằng vứt đi, nên nó đọng lại ở đây.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct CheckpointStats {
    /// Số lượt PASSIVE đã chạy.
    pub passive_runs: u64,
    /// Số lượt PASSIVE trả `busy != 0` — **bị chặn**, không phải đã xong.
    pub passive_busy: u64,
    /// Tổng số frame đã chép về database qua mọi lượt.
    pub frames_checkpointed: u64,
    /// Số lượt kích hoạt bởi điều kiện **(a) rảnh**.
    pub idle_triggered: u64,
    /// Số lượt kích hoạt bởi điều kiện **(b) `.db-wal` quá ngưỡng** — bằng chứng của AC5.
    pub threshold_triggered: u64,
    /// Số lượt TRUNCATE (chỉ ở lúc đóng).
    pub truncate_runs: u64,
    /// Số lượt TRUNCATE trả `busy != 0`.
    pub truncate_busy: u64,
    /// Số lượt checkpoint trả lỗi.
    pub errors: u64,
}

/// Trạng thái dùng chung giữa luồng writer, luồng checkpoint và chỗ gọi.
///
/// ⚠️ `Mutex` khoá bằng `unwrap_or_else(|e| e.into_inner())` ở **mọi** chỗ, không bằng
/// `unwrap()`. Lý do là Bẫy 6: `panic = "abort"` làm một panic ở đây giết cả tiến trình,
/// và một mutex bị nhiễm độc là thứ duy nhất trong module này có thể panic mà không phải
/// lỗi lập trình. Đọc tiếp một trạng thái nhiễm độc là lựa chọn đúng: nó chỉ chứa số
/// đếm và dấu thời gian.
pub(crate) struct Shared {
    epoch: Instant,
    last_write_ms: AtomicU64,
    dirty: AtomicBool,

    stop: Mutex<bool>,
    stop_cv: Condvar,
    done: Mutex<bool>,
    done_cv: Condvar,

    passive_runs: AtomicU64,
    passive_busy: AtomicU64,
    frames_checkpointed: AtomicU64,
    idle_triggered: AtomicU64,
    threshold_triggered: AtomicU64,
    truncate_runs: AtomicU64,
    truncate_busy: AtomicU64,
    errors: AtomicU64,

    diagnostics: Mutex<VecDeque<String>>,
}

/// Khoá một `Mutex` mà **không panic** kể cả khi nó đã nhiễm độc. Xem [`Shared`].
fn lock<T>(m: &Mutex<T>) -> MutexGuard<'_, T> {
    m.lock().unwrap_or_else(|poisoned| poisoned.into_inner())
}

impl Shared {
    pub(crate) fn new() -> Self {
        Self {
            epoch: Instant::now(),
            last_write_ms: AtomicU64::new(0),
            dirty: AtomicBool::new(false),
            stop: Mutex::new(false),
            stop_cv: Condvar::new(),
            done: Mutex::new(false),
            done_cv: Condvar::new(),
            passive_runs: AtomicU64::new(0),
            passive_busy: AtomicU64::new(0),
            frames_checkpointed: AtomicU64::new(0),
            idle_triggered: AtomicU64::new(0),
            threshold_triggered: AtomicU64::new(0),
            truncate_runs: AtomicU64::new(0),
            truncate_busy: AtomicU64::new(0),
            errors: AtomicU64::new(0),
            diagnostics: Mutex::new(VecDeque::new()),
        }
    }

    /// Luồng writer gọi sau **mỗi** job: mốc "lần ghi cuối" và cờ "có gì để chép".
    pub(crate) fn touch_write(&self) {
        // `as u64` an toàn: `Instant::elapsed` không âm, và 2^64 ms là ~584 triệu năm.
        self.last_write_ms
            .store(self.epoch.elapsed().as_millis() as u64, Ordering::Relaxed);
        self.dirty.store(true, Ordering::Relaxed);
    }

    fn since_last_write(&self) -> Duration {
        let now = self.epoch.elapsed().as_millis() as u64;
        let last = self.last_write_ms.load(Ordering::Relaxed);
        Duration::from_millis(now.saturating_sub(last))
    }

    /// Ghi một dòng chẩn đoán — vòng, có trần, và **cũng in ra stderr**.
    ///
    /// ⚠️ Chuỗi truyền vào phải KHÔNG DẤU: `check-i18n` Kiểm A quét `src/core/store/**`.
    pub(crate) fn note(&self, line: String) {
        eprintln!("{line}");
        let mut log = lock(&self.diagnostics);
        if log.len() == DIAGNOSTICS_CAP {
            log.pop_front();
        }
        log.push_back(line);
    }

    pub(crate) fn diagnostics(&self) -> Vec<String> {
        lock(&self.diagnostics).iter().cloned().collect()
    }

    pub(crate) fn stats(&self) -> CheckpointStats {
        CheckpointStats {
            passive_runs: self.passive_runs.load(Ordering::Relaxed),
            passive_busy: self.passive_busy.load(Ordering::Relaxed),
            frames_checkpointed: self.frames_checkpointed.load(Ordering::Relaxed),
            idle_triggered: self.idle_triggered.load(Ordering::Relaxed),
            threshold_triggered: self.threshold_triggered.load(Ordering::Relaxed),
            truncate_runs: self.truncate_runs.load(Ordering::Relaxed),
            truncate_busy: self.truncate_busy.load(Ordering::Relaxed),
            errors: self.errors.load(Ordering::Relaxed),
        }
    }
}

/// Tay cầm của luồng nền. [`Checkpointer::shutdown`] là nơi TRUNCATE duy nhất chạy.
pub(crate) struct Checkpointer {
    shared: Arc<Shared>,
    handle: Mutex<Option<JoinHandle<()>>>,
    budget: Duration,
    kind: StoreKind,
}

impl Checkpointer {
    /// Dựng kết nối RIÊNG rồi cho luồng nền sở hữu nó.
    ///
    /// 🔴 *"Kết nối riêng"* là chữ trong AC4, không phải một cách nói: mượn của writer
    /// thì checkpoint phải xếp hàng sau mọi job ghi (và `Connection` không `Sync` nên nó
    /// thậm chí không biên dịch được); mượn của pool thì nó chạy dưới `query_only = 1`
    /// và **không checkpoint được gì**.
    pub(crate) fn spawn(
        path: &Path,
        kind: StoreKind,
        tuning: Tuning,
        shared: Arc<Shared>,
    ) -> Result<Checkpointer, StoreError> {
        let conn = pragmas::open_connection(path, kind)?;
        pragmas::apply_checkpoint_pragmas(&conn, kind, &tuning)?;

        let thread_shared = Arc::clone(&shared);
        let db_path: PathBuf = path.to_path_buf();

        let handle = std::thread::Builder::new()
            .name(format!("aura-store-checkpoint-{}", kind.as_str()))
            .spawn(move || run(conn, db_path, kind, tuning, thread_shared))
            .map_err(|e| StoreError::OpenFailed {
                store: kind,
                detail: format!("spawn checkpoint thread: {e}"),
            })?;

        Ok(Checkpointer {
            shared,
            handle: Mutex::new(Some(handle)),
            budget: tuning.close_truncate_budget,
            kind,
        })
    }

    /// Ra hiệu dừng, rồi **chờ lượt TRUNCATE cuối trong trần thời gian**.
    ///
    /// 🔴 Hết trần ⇒ ghi chẩn đoán rồi **thoát**, không `join`, không treo tiến
    /// trình. Xem [`Tuning::close_truncate_budget`]: một `close()` chậm làm
    /// `check:scope` và `check:scope:bundled` đỏ vì tầng ghi dữ liệu, không vì phạm vi
    /// mà chúng canh.
    ///
    /// Idempotent — gọi lần thứ hai là no-op.
    pub(crate) fn shutdown(&self) {
        let Some(handle) = lock(&self.handle).take() else {
            return;
        };

        *lock(&self.shared.stop) = true;
        self.shared.stop_cv.notify_all();

        let finished = {
            let guard = lock(&self.shared.done);
            let (guard, _timeout) = self
                .shared
                .done_cv
                .wait_timeout_while(guard, self.budget, |done| !*done)
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            *guard
        };

        if finished {
            let _ = handle.join();
            return;
        }

        self.shared.note(format!(
            "store[{}] close: final wal_checkpoint(TRUNCATE) exceeded the {} ms budget; \
             leaving the checkpoint thread detached instead of blocking process exit",
            self.kind.as_str(),
            self.budget.as_millis()
        ));
    }
}

impl Drop for Checkpointer {
    fn drop(&mut self) {
        self.shutdown();
    }
}

/// Vòng đời của luồng nền.
fn run(conn: Connection, path: PathBuf, kind: StoreKind, tuning: Tuning, shared: Arc<Shared>) {
    loop {
        let stop = {
            let guard = lock(&shared.stop);
            // ⚠️ `wait_timeout_while`, không `wait_timeout` trần: một `notify_all()` xảy
            // ra trong khi luồng này đang CHẠY một lượt checkpoint (không phải đang chờ
            // trên condvar) không được đệm lại — condvar không nhớ thông báo đã bỏ lỡ. Một
            // `wait_timeout` trần sẽ vẫn ngủ đủ `checkpoint_tick` ở vòng kế trước khi đọc
            // lại cờ, ăn vào đúng `close_truncate_budget` mà `Checkpointer::shutdown` cố
            // giữ. `wait_timeout_while` đọc cờ TRƯỚC khi ngủ nên không mất tín hiệu.
            let (guard, _timeout) = shared
                .stop_cv
                .wait_timeout_while(guard, tuning.checkpoint_tick, |stop| !*stop)
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            *guard
        };
        if stop {
            break;
        }

        let over_threshold = match wal_len(&path) {
            Ok(len) => len > tuning.wal_threshold_bytes,
            Err(e) => {
                // Không suy ra "quá ngưỡng" từ một lỗi đọc: nói dối theo hướng đó có thể
                // ép TRUNCATE chạy nhầm chỗ. Ghi chẩn đoán rồi để tick sau thử lại — không
                // không nuốt im lặng như trước.
                shared.note(format!(
                    "store[{}] cannot read -wal file size: {e}",
                    kind.as_str()
                ));
                false
            }
        };
        let idle = shared.dirty.load(Ordering::Relaxed)
            && shared.since_last_write() >= tuning.idle_before_passive;

        if !over_threshold && !idle {
            continue;
        }

        // ⚠️ Ngưỡng được đếm TRƯỚC vì nó là vế đặc trưng của AC5 — *"chạy kể cả khi chưa
        // rảnh"*. Một lượt vừa quá ngưỡng vừa rảnh vẫn là bằng chứng cho vế (b).
        if over_threshold {
            shared.threshold_triggered.fetch_add(1, Ordering::Relaxed);
        } else {
            shared.idle_triggered.fetch_add(1, Ordering::Relaxed);
        }

        passive(&conn, kind, &shared);
    }

    truncate(&conn, kind, &shared);

    // Đóng kết nối TRƯỚC khi báo xong: chỗ gọi `close()` được phép xoá thư mục ngay sau
    // đó, và Windows từ chối xoá một tệp đang mở (NFR14). Đây là đúng lớp lỗi mà CI hai
    // nền tảng của Story 1.3 dựng ra để bắt, và nó chỉ đỏ trên một nhánh của ma trận.
    drop(conn);

    *lock(&shared.done) = true;
    shared.done_cv.notify_all();
}

/// Một lượt PASSIVE — ba cột đọc hết, `busy != 0` ghi chẩn đoán.
fn passive(conn: &Connection, kind: StoreKind, shared: &Shared) {
    shared.passive_runs.fetch_add(1, Ordering::Relaxed);

    match pragmas::wal_checkpoint(conn, "PASSIVE", kind) {
        Ok(outcome) => {
            if outcome.busy != 0 {
                shared.passive_busy.fetch_add(1, Ordering::Relaxed);
                shared.note(format!(
                    "store[{}] wal_checkpoint(PASSIVE) blocked: busy={} log={} checkpointed={}",
                    kind.as_str(),
                    outcome.busy,
                    outcome.log,
                    outcome.checkpointed
                ));
                // `dirty` KHÔNG được xoá: lượt này chưa xong, và xoá cờ ở đây là tự
                // nói với chính mình rằng WAL đã sạch.
                return;
            }

            if outcome.checkpointed > 0 {
                shared
                    .frames_checkpointed
                    .fetch_add(outcome.checkpointed as u64, Ordering::Relaxed);
            }

            // Chỉ khi chép ĐỦ mới coi là sạch. `log > checkpointed` nghĩa là một reader
            // còn giữ ảnh chụp cũ và WAL chưa được dùng lại (xem doc-comment module).
            if outcome.checkpointed >= outcome.log {
                shared.dirty.store(false, Ordering::Relaxed);
            }
        }
        Err(err) => {
            shared.errors.fetch_add(1, Ordering::Relaxed);
            // ⚠️ Không thêm tiền tố `store[{kind}]` ở đây: `StoreError::Display` (xem
            // `mod.rs`) đã tự mang tiền tố đó, và lặp lại cho ra
            // `store[global] store[global] open failed: …`.
            shared.note(format!("{err}"));
        }
    }
}

/// Lượt TRUNCATE cuối cùng — **chỉ ở đây**, và chỉ vì `.db-wal` phải về 0 khi thoát.
fn truncate(conn: &Connection, kind: StoreKind, shared: &Shared) {
    shared.truncate_runs.fetch_add(1, Ordering::Relaxed);

    match pragmas::wal_checkpoint(conn, "TRUNCATE", kind) {
        Ok(outcome) => {
            if outcome.checkpointed > 0 {
                shared
                    .frames_checkpointed
                    .fetch_add(outcome.checkpointed as u64, Ordering::Relaxed);
            }
            if outcome.busy != 0 {
                shared.truncate_busy.fetch_add(1, Ordering::Relaxed);
                shared.note(format!(
                    "store[{}] wal_checkpoint(TRUNCATE) blocked at close: busy={} log={} checkpointed={}",
                    kind.as_str(),
                    outcome.busy,
                    outcome.log,
                    outcome.checkpointed
                ));
            }
        }
        Err(err) => {
            shared.errors.fetch_add(1, Ordering::Relaxed);
            // ⚠️ Không thêm tiền tố `store[{kind}]` ở đây: `StoreError::Display` (xem
            // `mod.rs`) đã tự mang tiền tố đó, và lặp lại cho ra
            // `store[global] store[global] open failed: …`.
            shared.note(format!("{err}"));
        }
    }
}
