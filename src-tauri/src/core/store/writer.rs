//! MỘT kết nối ghi, MỘT luồng, MỘT hàng đợi nối tiếp — AD-11, AC1, AC2.
//!
//! ─────────────────────────────────────────────────────────────────────────────
//! VÌ SAO ĐÂY LÀ CẢ AC2, KHÔNG CHỈ MỘT NỬA
//! ─────────────────────────────────────────────────────────────────────────────
//! AC2 nói *"không module nào tự mở được kết nối ghi"*. Cách cưỡng chế rẻ nhất là một
//! quy ước trong tài liệu, và nó thất bại ở story thứ ba. Cách ở đây là: **không có kết
//! nối ghi thứ hai để mở**. `Connection` ghi duy nhất được `move` vào luồng này lúc
//! [`Writer::spawn`] và không bao giờ rời khỏi nó — `rusqlite::Connection` là `Send`
//! nhưng **không `Sync`** (`rusqlite-0.40.1/src/lib.rs:364`), nên trình biên dịch tự
//! cưỡng chế phần còn lại. Nửa kia của AC2 là `tests/store_boundary.rs`.
//!
//! ─────────────────────────────────────────────────────────────────────────────
//! 🔴 LUỒNG NÀY KHÔNG ĐƯỢC PANIC. LỖI LÀ **GIÁ TRỊ**.
//! ─────────────────────────────────────────────────────────────────────────────
//! `Cargo.toml` `[profile.release]` đặt `panic = "abort"` (cố ý đóng băng để giữ số đo
//! NFR6 của Story 1.1 so sánh được). Hệ quả: một `panic!` ở đây **chấm dứt tiến trình
//! ngay** — không unwind, không `Drop`, không cơ hội flush WAL; và trên Windows release
//! `windows_subsystem = "windows"` khiến nó cũng không in ra đâu.
//!
//! → `catch_unwind` **vô dụng** ở đây: không có unwind để bắt.
//! → Mọi `unwrap()` / `expect()` trong module này là một **lỗi thiết kế**, không phải
//!   một lối tắt. Mutex khoá qua `unwrap_or_else(|e| e.into_inner())`; kênh phản hồi gửi
//!   bằng `let _ = tx.send(…)`.
//! → ⛔ Đừng "giải quyết" bằng cách sửa `[profile.release]` — `deferred-work.md`, và
//!   quyết định đó thuộc Story 1.9 / 10.9 cùng lượt đo lại NFR6.

use std::cell::Cell;
use std::sync::{Arc, Mutex, MutexGuard, mpsc};
use std::thread::JoinHandle;

use rusqlite::{Connection, Transaction};

use super::checkpoint::Shared;
use super::{SqlResult, StoreError, StoreKind};

/// Một việc đã đóng gói. `FnOnce` vì mỗi job chạy đúng một lần và mang theo kênh phản
/// hồi **riêng của lời gọi sinh ra nó** — không có kênh phản hồi dùng chung, nên không
/// có ca "trả kết quả nhầm người". Trả `bool`: job có thật sự COMMIT hay không, để
/// luồng writer chỉ `touch_write()` khi có gì mới trong WAL.
type Task = Box<dyn FnOnce(&mut Connection) -> bool + Send + 'static>;

fn lock<T>(m: &Mutex<T>) -> MutexGuard<'_, T> {
    m.lock().unwrap_or_else(|poisoned| poisoned.into_inner())
}

std::thread_local! {
    /// `true` trong suốt vòng đời của chính luồng writer, `false` ở mọi luồng khác.
    ///
    /// ⚠️ Cờ này tồn tại để chặn một lớp deadlock cụ thể: một job ghi (chạy TRÊN luồng
    /// writer) gọi lại [`Writer::write`] — task lồng vào hàng đợi, rồi chờ đúng luồng
    /// đang bận chạy job ngoài xử lý nó. Không ai còn rảnh để dequeue, `reply_rx.recv()`
    /// của lời gọi lồng chặn mãi. Bắt sớm ở đây rẻ hơn hẳn một trần thời gian.
    static ON_WRITER_THREAD: Cell<bool> = const { Cell::new(false) };
}

/// Hàng đợi ghi nối tiếp.
pub(crate) struct Writer {
    /// `Option` chứ không phải `Sender` trần, và `Mutex` chứ không phải trần.
    ///
    /// 🔴 Đây là cơ chế đóng: [`Writer::shutdown`] **lấy** `Sender` ra và thả nó, kênh
    /// đứt, `recv()` của luồng trả `Err`, luồng thoát. Mọi [`Writer::write`] sau đó thấy
    /// `None` và trả [`StoreError::WriterGone`] **ngay**, ⛔ không treo.
    ///
    /// ⚠️ Khoá được giữ **trong suốt lời `send`**, có chủ ý. Bản dùng `Sender::clone()`
    /// rồi thả khoá trước khi gửi có một khe hở thật: bản sao giữ kênh sống, nên luồng
    /// writer không thoát, nên `shutdown()` `join` mãi.
    jobs: Mutex<Option<mpsc::Sender<Task>>>,
    handle: Mutex<Option<JoinHandle<()>>>,
    kind: StoreKind,
}

impl Writer {
    /// Trao quyền sở hữu kết nối ghi cho một luồng mới.
    ///
    /// ⚠️ `conn` đi vào bằng giá trị. Đó là chữ ký nói ra hợp đồng: sau lời gọi này,
    /// không ai — kể cả [`super::Store`] — còn cầm được kết nối ghi nữa.
    pub(crate) fn spawn(
        conn: Connection,
        kind: StoreKind,
        shared: Arc<Shared>,
    ) -> Result<Writer, StoreError> {
        let (tx, rx) = mpsc::channel::<Task>();

        let handle = std::thread::Builder::new()
            .name(format!("aura-store-writer-{}", kind.as_str()))
            .spawn(move || {
                ON_WRITER_THREAD.with(|flag| flag.set(true));
                let mut conn = conn;
                // `recv()` trả `Err` khi mọi `Sender` đã bị thả — đó là tín hiệu dừng, và
                // nó là tín hiệu DUY NHẤT. ⛔ Không có cờ dừng thứ hai để hai thứ lệch nhau.
                while let Ok(task) = rx.recv() {
                    // Chỉ mốc "lần ghi cuối" khi job THẬT SỰ commit — một job rollback
                    // không thêm gì vào WAL, và coi nó là "vừa ghi" trì hoãn PASSIVE vô cớ.
                    if task(&mut conn) {
                        shared.touch_write();
                    }
                }
                // Thả kết nối TRƯỚC khi luồng kết thúc, tường minh: `close()` chạy
                // TRUNCATE ngay sau khi `join` luồng này, và một kết nối ghi còn mở là
                // một reader mà TRUNCATE phải chờ.
                drop(conn);
            })
            .map_err(|e| StoreError::OpenFailed {
                store: kind,
                detail: format!("spawn writer thread: {e}"),
            })?;

        Ok(Writer {
            jobs: Mutex::new(Some(tx)),
            handle: Mutex::new(Some(handle)),
            kind,
        })
    }

    /// Xếp một job vào hàng đợi và **chặn** cho tới khi nó chạy xong.
    ///
    /// Mỗi job là một giao dịch: `Ok` ⇒ commit, `Err` ⇒ rollback. Hình dạng này đến từ
    /// §Quyết định #3 của story và ⛔ không phải chỗ để sáng tạo.
    pub(crate) fn write<T, F>(&self, job: F) -> Result<T, StoreError>
    where
        F: FnOnce(&Transaction<'_>) -> SqlResult<T> + Send + 'static,
        T: Send + 'static,
    {
        let kind = self.kind;

        // ⚠️ Gọi lồng: job đang chạy TRÊN luồng writer gọi lại `Store::write`/`Writer::write`.
        // Task lồng vào hàng đợi rồi chờ đúng luồng đang bận chạy job ngoài xử lý nó —
        // không ai còn rảnh để dequeue, và `reply_rx.recv()` dưới đây sẽ chặn mãi. Bắt ở
        // đây, ⛔ không để `recv()` tự lộ ra thành một treo không giải thích được.
        if ON_WRITER_THREAD.with(Cell::get) {
            return Err(StoreError::WriteFailed {
                store: kind,
                detail: "reentrant Store::write() called from within a write job \
                         on the writer thread"
                    .to_owned(),
            });
        }

        let (reply_tx, reply_rx) = mpsc::channel::<SqlResult<T>>();

        let task: Task = Box::new(move |conn: &mut Connection| {
            let outcome = (|| {
                let tx = conn.transaction()?;
                let value = job(&tx)?;
                tx.commit()?;
                Ok(value)
            })();

            let committed = outcome.is_ok();

            // ⚠️ `let _ =` là CỐ Ý, không phải cẩu thả: chỗ gọi có thể đã bỏ đi (bị huỷ,
            // hết thời gian chờ ở một tầng trên), và khi đó `send` trả `Err`. Một
            // `unwrap()` ở dòng này giết luồng writer — và với `panic = "abort"` là giết
            // cả tiến trình. Xem doc-comment của module.
            let _ = reply_tx.send(outcome);

            committed
        });

        {
            let guard = lock(&self.jobs);
            match guard.as_ref() {
                Some(tx) => tx
                    .send(task)
                    .map_err(|_| StoreError::WriterGone { store: kind })?,
                None => return Err(StoreError::WriterGone { store: kind }),
            }
        }

        // Ba nhánh, và nhánh thứ ba là lý do hàm này không bao giờ treo: nếu luồng writer
        // biến mất giữa chừng thì `reply_tx` nằm trong `task` bị thả theo, kênh đứt, và
        // `recv()` trả `Err` thay vì chờ mãi.
        match reply_rx.recv() {
            Ok(Ok(value)) => Ok(value),
            Ok(Err(err)) => Err(StoreError::WriteFailed {
                store: kind,
                detail: err.to_string(),
            }),
            Err(_) => Err(StoreError::WriterGone { store: kind }),
        }
    }

    /// Đóng hàng đợi và **chờ** luồng writer xử lý hết việc đã xếp.
    ///
    /// Idempotent. Sau lời gọi này mọi [`Writer::write`] trả [`StoreError::WriterGone`].
    ///
    /// ⚠️ `join` ở đây **không** cần trần thời gian: hàng đợi hữu hạn, mỗi job là một
    /// giao dịch cục bộ trên một kết nối mà không ai khác giữ, và job không gọi ra ngoài.
    /// Trần thời gian nằm ở lượt TRUNCATE — nơi SQLite thật sự **chờ reader** — chứ
    /// không ở đây. Đặt trần ở cả hai chỗ là bỏ dở một giao dịch đang commit để tiết kiệm
    /// mili-giây trên đường thoát.
    pub(crate) fn shutdown(&self) {
        // Thả `Sender`: đây là toàn bộ tín hiệu dừng.
        lock(&self.jobs).take();

        if let Some(handle) = lock(&self.handle).take() {
            let _ = handle.join();
        }
    }
}

impl Drop for Writer {
    fn drop(&mut self) {
        self.shutdown();
    }
}
