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
//! → Đừng "giải quyết" bằng cách sửa `[profile.release]` — `deferred-work.md`, và
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

/// Vé của một job đã **xếp vào** writer nhưng có thể chưa chạy xong.
///
/// `pub(crate)` là ranh giới cố ý của Story 3.5 review: một consumer cần xếp job trong
/// vùng khoá state rồi NHẢ khoá trước khi chờ. Không mở sender, connection hay transaction
/// ra ngoài `core/store/**`; vé chỉ cho đúng một thao tác [`WriteTicket::wait`].
///
/// 🔴 **THÊM 2026-08-25 (Cụm C, C5) — `#[must_use]` đặt trên KIỂU, không trên hàm sinh ra
/// nó.** Thả một vé mà không gọi [`WriteTicket::wait`] nghĩa là thả TRỌN kết quả commit/
/// rollback và mọi [`StoreError`] đi cùng nó — job vẫn CHẠY (đã enqueue), chỉ không ai còn
/// biết nó thành công hay thất bại. `Result` trả về từ [`Writer::enqueue`]/
/// [`super::Store::write_ticket`] đã tự `#[must_use]` (mọi `Result` đều vậy), nên đánh dấu
/// trên HÀM là dư — ca thật sự lọt lưới là sau khi đã `?`/`.expect()` mở `Result` đó ra,
/// `let ticket = store.write_ticket(job)?;` rồi để `ticket` rơi khỏi phạm vi mà không
/// `.wait()`. Chỉ một `#[must_use]` trên CHÍNH kiểu `WriteTicket<T>` mới bắt được câu đó.
///
/// ⚠️ **Giới hạn thật, không phải một lỗ hổng bị bỏ sót:** `#[must_use]` chỉ nổ khi giá trị
/// bị thả NGAY TẠI một câu lệnh (`store.write_ticket(job)?;` viết trần). Một biến cục bộ giữ
/// `ticket` rồi rơi khỏi phạm vi ở cuối hàm KHÔNG bị lint này bắt — không lint nào trong Rust
/// hôm nay bắt được ca đó. Hôm nay (2026-08-25) **0 chỗ đang thả vé** trong toàn kho (đã rà
/// mọi chỗ sinh/tiêu thụ), nên bản vá này không sửa một lỗi đang sống — nó chặn lỗi KẾ TIẾP.
///
/// 🔴 **THÊM 2026-08-25 (Cụm C, Ice chốt) — `[lints.rust] unused_must_use = "deny"` trong
/// `Cargo.toml` nâng cảnh báo này thành LỖI BIÊN DỊCH cho cả crate** (đo trước khi nâng:
/// `RUSTFLAGS="-D warnings" cargo check --locked` → exit 0, 0 cảnh báo trong toàn crate
/// 2026-08-25 — nâng không làm gãy build nào đang xanh). §I/O Matrix ca ⑧ giờ là một CỔNG
/// COMPILE THẬT, không còn một lượt bấm tay: bất kỳ chỗ nào trong crate thả một vé bằng một
/// câu lệnh trần đều làm `cargo build`/`cargo check`/`cargo test` ĐỎ ngay, không cần một ca
/// test riêng mới bắt được.
///
/// ⚠️ **Vì sao doctest dưới đây KHÔNG gọi được `WriteTicket` thật.** `WriteTicket` — và mọi
/// hàm sinh ra nó ([`Writer::enqueue`]/[`super::Store::write_ticket`]) — đều `pub(crate)`, cố
/// ý (xem đoạn đầu doc-comment này). Một doctest luôn biên dịch như MỘT CRATE NGOÀI (đúng luật
/// riêng tư thường của Rust cho `pub(crate)`), nên không đường nào gọi được `WriteTicket`
/// thật từ đây — đã đo bằng BA thực nghiệm độc lập 2026-08-25: nhập thẳng tên ⇒
/// `E0425 cannot find type`; nhập qua đường dẫn đủ
/// (`auratranslate_lib::core::store::writer::WriteTicket`) ⇒ `E0603 module is private`; và
/// một hàm bọc đánh dấu `#[cfg(doctest)]` cũng KHÔNG lọt vào rlib mà doctest liên kết (`cfg
/// (doctest)` chỉ áp cho CHÍNH đoạn doctest, không áp cho crate thư viện đang được biên dịch
/// để doctest liên kết tới) ⇒ vẫn `E0603`. Mở rộng tầm nhìn của `WriteTicket` ra `pub` để một
/// doctest gọi được là ĐỔI một bất biến kiến trúc (đúng câu mà chính đoạn trên vừa khoá:
/// "Không mở sender, connection hay transaction ra ngoài `core/store/**`") — đó là việc của
/// một `AD` mới do Ice ký, không phải một dòng mã tự quyết ở đây.
///
/// Khối dưới đây vì vậy dựng một kiểu THAY THẾ CÙNG HÌNH DẠNG (một `#[must_use]` trên KIỂU,
/// một hàm sinh nó, một câu lệnh trần thả nó) để chứng minh ĐÚNG cơ chế mà `[lints.rust]` áp
/// dụng cho `WriteTicket` thật — Cargo áp `[lints]` cho các TARGET của gói (lib/bin/test/
/// bench/example), KHÔNG áp cho chương trình riêng mà `rustdoc` biên dịch cho một doctest
/// (đo được: bỏ dòng `#![deny(unused_must_use)]` dưới đây thì khối này chỉ CẢNH BÁO, không
/// còn lỗi, và ca `compile_fail` tự đỏ vì "biên dịch thành công" — nên dòng đó PHẢI có mặt để
/// mô phỏng đúng mức nghiêm mà `Cargo.toml` áp cho crate thật).
///
/// ```compile_fail
/// #![deny(unused_must_use)]
///
/// #[must_use]
/// struct SameShapeAsWriteTicket;
///
/// fn enqueue_like_write_ticket() -> SameShapeAsWriteTicket {
///     SameShapeAsWriteTicket
/// }
///
/// fn main() {
///     enqueue_like_write_ticket(); // cau lenh tran -- dung ca I/O Matrix ⑧, khong `.wait()`
/// }
/// ```
///
/// Đối chứng NGƯỢC cho giới hạn ⚠️ ở trên — cùng khối, chỉ đổi câu lệnh trần thành một `let`
/// rồi để biến rơi khỏi phạm vi: KHÔNG lỗi, KHÔNG cảnh báo, biên dịch sạch. Đây LÀ khối
/// `compile_fail` phải thất bại theo nghĩa "không được đỏ" — dùng `ignore` thay vì
/// `compile_fail` vì mục đích của khối này là chứng minh nó BIÊN DỊCH ĐƯỢC, không phải nó
/// gãy; `ignore` chặn nó khỏi bị chạy như một ca "phải biên dịch" bắt buộc trong khi vẫn hiện
/// trong tài liệu như một ví dụ đọc được.
///
/// ```ignore
/// #![deny(unused_must_use)]
///
/// #[must_use]
/// struct SameShapeAsWriteTicket;
///
/// fn enqueue_like_write_ticket() -> SameShapeAsWriteTicket {
///     SameShapeAsWriteTicket
/// }
///
/// fn does_not_wait() {
///     let ticket = enqueue_like_write_ticket();
///     // `ticket` roi khoi pham vi o day ma khong `.wait()` -- KHONG lint nao bat duoc.
/// }
///
/// fn main() {
///     does_not_wait();
/// }
/// ```
#[must_use]
pub(crate) struct WriteTicket<T> {
    reply_rx: mpsc::Receiver<SqlResult<T>>,
    kind: StoreKind,
}

impl<T> WriteTicket<T> {
    /// Chờ job đã xếp trả lời. Kênh đứt là `WriterGone`, không một đường treo vô hạn.
    pub(crate) fn wait(self) -> Result<T, StoreError> {
        match self.reply_rx.recv() {
            Ok(Ok(value)) => Ok(value),
            Ok(Err(err)) => Err(StoreError::WriteFailed {
                store: self.kind,
                detail: err.to_string(),
            }),
            Err(_) => Err(StoreError::WriterGone { store: self.kind }),
        }
    }
}

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
    /// `None` và trả [`StoreError::WriterGone`] **ngay**, không treo.
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
                // nó là tín hiệu DUY NHẤT. Không có cờ dừng thứ hai để hai thứ lệch nhau.
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
    /// §Quyết định #3 của story và không phải chỗ để sáng tạo.
    pub(crate) fn write<T, F>(&self, job: F) -> Result<T, StoreError>
    where
        F: FnOnce(&Transaction<'_>) -> SqlResult<T> + Send + 'static,
        T: Send + 'static,
    {
        self.enqueue(job)?.wait()
    }

    /// Xếp job và trả vé NGAY SAU `send`, không chờ giao dịch chạy xong.
    ///
    /// Hình dạng này tách đúng hai pha vốn đã có trong [`Writer::write`]; nó không dựng
    /// writer hay hàng đợi thứ hai. Consumer bình thường tiếp tục gọi `write`; chỉ đường
    /// cần nhả mutex state trước khi chờ mới dùng vé qua `Store::write_ticket`.
    pub(crate) fn enqueue<T, F>(&self, job: F) -> Result<WriteTicket<T>, StoreError>
    where
        F: FnOnce(&Transaction<'_>) -> SqlResult<T> + Send + 'static,
        T: Send + 'static,
    {
        let kind = self.kind;

        // ⚠️ Gọi lồng: job đang chạy TRÊN luồng writer gọi lại `Store::write`/`Writer::write`.
        // Task lồng vào hàng đợi rồi chờ đúng luồng đang bận chạy job ngoài xử lý nó —
        // không ai còn rảnh để dequeue, và `reply_rx.recv()` dưới đây sẽ chặn mãi. Bắt ở
        // đây, không để `recv()` tự lộ ra thành một treo không giải thích được.
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

        // 🔵 CẬP NHẬT 2026-08-22 — `recv()` không còn nằm ngay dưới sau khi enqueue/reply
        // được tách thành ticket. Ba nhánh cũ vẫn nguyên vẹn trong `WriteTicket::wait`:
        // writer trả `Ok`/`Err`, hoặc `reply_tx` bị thả làm kênh đứt và `recv()` trả `Err`
        // thay vì chờ mãi.
        Ok(WriteTicket { reply_rx, kind })
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
