//! Pool đọc song song trên WAL, **chỉ-đọc do SQLite cưỡng chế** — AD-11, AC1, AC2.
//!
//! ─────────────────────────────────────────────────────────────────────────────
//! ⛔ KHÔNG `r2d2` / `deadpool` / `bb8` / `parking_lot`
//! ─────────────────────────────────────────────────────────────────────────────
//! Một pool gồm: một `Vec` kết nối, một `Mutex`, một `Condvar`, và một guard trả kết nối
//! về trong `Drop`. Đó là toàn bộ tệp này. Đổi lại, mỗi phụ thuộc mới phải rà giấy phép
//! **bằng cách mở tệp trong nguồn đã tải mà đọc** (NFR15) và vào bảng Stack — đó là
//! quyết định của Ice, không phải hệ quả phụ của một story tầng dữ liệu.
//!
//! ─────────────────────────────────────────────────────────────────────────────
//! 🔴 CHỈ-ĐỌC LÀ CỦA SQLITE, KHÔNG PHẢI CỦA KỶ LUẬT NGƯỜI VIẾT
//! ─────────────────────────────────────────────────────────────────────────────
//! Mỗi kết nối trong pool đặt `PRAGMA query_only = 1` **và đọc lại để xác nhận**. Một
//! `INSERT` gửi qua [`ReaderPool::read`] không "được coi là sai" — nó **thất bại**, với
//! lỗi của SQLite. Đó là bằng chứng của AC2 vế *"khả năng hiển thị của kiểu"*: người viết
//! không phải nhớ gì cả.
//!
//! Vì sao `query_only` chứ không `SQLITE_OPEN_READ_ONLY` — xem Quyết định #2 trong
//! `pragmas.rs`.

use std::path::Path;
use std::sync::{Condvar, Mutex, MutexGuard};

use rusqlite::Connection;

use super::{ReadHandle, SqlResult, StoreError, StoreKind, Tuning, pragmas};

fn lock<T>(m: &Mutex<T>) -> MutexGuard<'_, T> {
    m.lock().unwrap_or_else(|poisoned| poisoned.into_inner())
}

struct PoolState {
    idle: Vec<Connection>,
    closed: bool,
}

/// Pool kết nối đọc.
pub(crate) struct ReaderPool {
    state: Mutex<PoolState>,
    available: Condvar,
    kind: StoreKind,
}

impl ReaderPool {
    /// Mở `tuning.pool_size` kết nối, mỗi cái tự đặt bộ PRAGMA của nó.
    ///
    /// ⚠️ *"Mỗi cái tự đặt"* là Bẫy 3 và nó không rút gọn được: `busy_timeout`,
    /// `wal_autocheckpoint` và `query_only` là trạng thái **của từng kết nối**, không
    /// phải của database. Quên `busy_timeout` trên pool nghĩa là reader nhận
    /// `SQLITE_BUSY` **ngay lập tức** trong lúc TRUNCATE chạy — biểu hiện thành "thỉnh
    /// thoảng tra cứu lỗi", và không tái lập được.
    pub(crate) fn open(
        path: &Path,
        kind: StoreKind,
        tuning: &Tuning,
    ) -> Result<ReaderPool, StoreError> {
        Self::open_with(
            path,
            kind,
            tuning,
            pragmas::open_connection,
            pragmas::apply_reader_pragmas,
        )
    }

    /// Pool đọc trên một tệp **CHỈ ĐỌC** — đường của [`StoreKind::Dict`] (Story 1.11).
    ///
    /// Khác [`ReaderPool::open`] **đúng hai hàm**: cờ mở (`READ_ONLY`, ⛔ không `CREATE`)
    /// và bộ pragma (⛔ không `verify_wal`, ⛔ không `wal_autocheckpoint`). Mọi thứ còn
    /// lại — `Mutex` + `Condvar` + `Lease` + `Drop` trả kết nối về — **dùng lại nguyên**.
    ///
    /// ⛔ Đó là lý do tệp này ⛔ không có bản sao thứ hai của thân pool: hai bản sẽ trôi
    /// khỏi nhau, và bản ít được đọc hơn sẽ là bản mang lỗi rò kết nối.
    pub(crate) fn open_readonly(
        path: &Path,
        kind: StoreKind,
        tuning: &Tuning,
    ) -> Result<ReaderPool, StoreError> {
        Self::open_with(
            path,
            kind,
            tuning,
            pragmas::open_readonly_connection,
            pragmas::apply_dict_reader_pragmas,
        )
    }

    /// Thân dùng chung. Hai tham số hàm là **toàn bộ** khác biệt giữa hai đường mở.
    fn open_with(
        path: &Path,
        kind: StoreKind,
        tuning: &Tuning,
        open_one: fn(&Path, StoreKind) -> Result<Connection, StoreError>,
        apply_pragmas: fn(&Connection, StoreKind, &Tuning) -> Result<(), StoreError>,
    ) -> Result<ReaderPool, StoreError> {
        // Sàn 1: một pool rỗng làm `read()` chờ mãi trên `Condvar` — một cách treo mà
        // không lỗi nào được ném. `Tuning` là dữ liệu, kể cả trong test, nên chặn ở đây.
        let size = tuning.pool_size.max(1);

        let mut idle = Vec::with_capacity(size);
        for _ in 0..size {
            let conn = open_one(path, kind)?;
            apply_pragmas(&conn, kind, tuning)?;
            idle.push(conn);
        }

        Ok(ReaderPool {
            state: Mutex::new(PoolState {
                idle,
                closed: false,
            }),
            available: Condvar::new(),
            kind,
        })
    }

    /// Mượn một kết nối, chạy closure, **trả kết nối về dù closure kết thúc kiểu gì**.
    pub(crate) fn read<T, F>(&self, job: F) -> Result<T, StoreError>
    where
        F: FnOnce(ReadHandle<'_>) -> SqlResult<T>,
    {
        let lease = self.acquire()?;

        // ⚠️ Kết quả được tính TRƯỚC khi `lease` ra khỏi scope, và `Lease` trả kết nối về
        // trong `Drop` — nên đường `Err` và đường panic đều không rò kết nối. Một bản
        // "trả về bằng tay sau khi xong" rò đúng ở hai đường đó, và pool cạn dần cho tới
        // khi `read()` chờ mãi.
        //
        // ⚠️ Nhánh `None` không đạt tới được (`Lease::conn` chỉ bị lấy đi trong `Drop`),
        // và nó vẫn là **một giá trị lỗi** chứ không phải một `expect`. Cùng luật với
        // luồng writer: `panic = "abort"` biến mọi panic trong tầng này thành một tiến
        // trình chết không kịp flush WAL — kể cả panic ở một nhánh "không thể xảy ra".
        let Some(conn) = lease.conn() else {
            return Err(StoreError::PoolClosed { store: self.kind });
        };

        let outcome = job(conn);

        outcome.map_err(|err| StoreError::ReadFailed {
            store: self.kind,
            detail: err.to_string(),
        })
    }

    fn acquire(&self) -> Result<Lease<'_>, StoreError> {
        let mut state = lock(&self.state);

        loop {
            if state.closed {
                return Err(StoreError::PoolClosed { store: self.kind });
            }
            if let Some(conn) = state.idle.pop() {
                return Ok(Lease {
                    pool: self,
                    conn: Some(conn),
                });
            }
            // ⚠️ Chờ **không** trần thời gian là an toàn ở đây, và chỉ vì [`ReaderPool::close`]
            // đánh thức mọi người chờ. Không có nó thì đây là một chỗ treo lúc thoát.
            state = self
                .available
                .wait(state)
                .unwrap_or_else(|poisoned| poisoned.into_inner());
        }
    }

    fn release(&self, conn: Connection) {
        let mut state = lock(&self.state);
        if state.closed {
            // Pool đã đóng: thả kết nối thay vì cất lại. Giữ lại là giữ một tệp đang mở,
            // và Windows từ chối xoá tệp đang mở (NFR14).
            drop(conn);
            return;
        }
        state.idle.push(conn);
        drop(state);
        self.available.notify_one();
    }

    /// Đóng pool: thả mọi kết nối rảnh, đánh thức mọi người đang chờ.
    ///
    /// Idempotent. Sau lời gọi này [`ReaderPool::read`] trả [`StoreError::PoolClosed`].
    ///
    /// 🔴 Phải chạy **trước** lượt TRUNCATE cuối: TRUNCATE chờ mọi reader rời đi, và một
    /// kết nối pool còn mở là một reader.
    pub(crate) fn close(&self) {
        let mut state = lock(&self.state);
        state.closed = true;
        state.idle.clear();
        drop(state);
        self.available.notify_all();
    }
}

impl Drop for ReaderPool {
    fn drop(&mut self) {
        self.close();
    }
}

/// Kết nối đang được mượn. `Drop` trả nó về pool — kể cả trên đường `Err` hay panic.
struct Lease<'p> {
    pool: &'p ReaderPool,
    conn: Option<Connection>,
}

impl Lease<'_> {
    /// `None` chỉ xảy ra sau `Drop`, tức không bao giờ với một `&self` còn sống. Trả
    /// `Option` thay vì `expect` để module này giữ đúng lời hứa **không panic**.
    fn conn(&self) -> Option<ReadHandle<'_>> {
        self.conn.as_ref()
    }
}

impl Drop for Lease<'_> {
    fn drop(&mut self) {
        if let Some(conn) = self.conn.take() {
            self.pool.release(conn);
        }
    }
}
