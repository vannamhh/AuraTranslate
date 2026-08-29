//! Một tệp `.db` mở **CHỈ ĐỌC** — đường của dữ liệu từ điển (AD-7, Story 1.11 AC7).
//!
//! 🔵 **SỬA (2026-08-29, Story 5.9) — không còn chỉ `StoreKind::Dict`.** `Indexer::rebuild`
//! (`core/library/indexer.rs`) nay mở `project.db` của mỗi `.atproj` CHỈ ĐỌC qua đây để thu
//! hoạch văn bản cho tìm kiếm full-text (FR8) — xem doc-comment của [`ReadOnlyDb::open`] cho lý
//! do đó vẫn là một MIỄN TRỪ CÓ TÊN (`{Dict, Project}`), không một cửa mở tuỳ ý.
//!
//! ─────────────────────────────────────────────────────────────────────────────
//! 🔴 VÌ SAO ĐƯỜNG NÀY SỐNG Ở `core/store/` CHỨ KHÔNG Ở `core/dict/`
//! ─────────────────────────────────────────────────────────────────────────────
//! `tests/store_boundary.rs::only_core_store_may_name_rusqlite` cấm hai chuỗi
//! `"rusqlite"` và `"Connection::open"` ở **mọi** tệp ngoài `src/core/store/**`. Cổng đó
//! canh AD-11 (*"không module nào được tự mở kết nối ghi"*). Một tệp từ điển là chỉ đọc
//! nên AD-11 không áp — nhưng cổng **không phân biệt được điều đó**, và nới nó là sai
//! đường: một cổng có hai miễn trừ là một cổng sẽ có ba, và miễn trừ thứ ba sẽ là một
//! module **có** ghi.
//!
//! → Đường mở tệp ở lại đây. `core/dict/` chỉ nhận một [`ReadHandle`] và viết SQL trên
//!   nó, không bao giờ gõ tên crate SQLite. Cổng giữ **đúng một** miễn trừ.
//!
//! ─────────────────────────────────────────────────────────────────────────────
//! VÌ SAO KHÔNG TÁI DÙNG [`Store`]
//! ─────────────────────────────────────────────────────────────────────────────
//! [`Store::open`] mở `READ_WRITE | CREATE`, đặt `journal_mode = WAL`, chạy bộ di trú,
//! rồi dựng một luồng writer và một luồng checkpoint. **Cả bốn đều GHI VÀO tệp**, và một
//! tệp từ điển được giao kèm checksum trong `dict-manifest.toml` (AD-25) — ghi vào nó
//! một byte là làm checksum thành sai, mà không cổng nào bắt được.
//!
//! [`ReadOnlyDb`] là phần còn lại sau khi bỏ hết bốn thứ đó: một pool đọc, và không gì
//! khác.
//!
//! **Không `use tauri::…`** ở tệp này — `store_boundary.rs::core_store_does_not_depend_on_tauri`
//! quét cả `core/store/**`. Đường lấy `$RESOURCE` sống ở chỗ gọi, đúng khuôn `$APPDATA`
//! của [`Store`].

use std::path::{Path, PathBuf};

use super::{ReadHandle, SqlResult, StoreError, StoreKind, Tuning, reader};

/// Một tệp `.db` chỉ đọc: một pool kết nối, không writer, không checkpoint.
///
/// `Send + Sync` cùng lý do với [`Store`] — điều kiện để nó vào `app.manage(…)` ở
/// Story 1.13 mà chỗ gọi không phải bọc thêm `Mutex`.
///
/// ⚠️ [`ReadOnlyDb::close`] chạy trong [`Drop`] nếu chưa ai gọi. Cùng bài học NFR14 của
/// [`Store`]: trên Windows một tệp còn mở là một `remove_dir_all` thất bại, và một tệp
/// từ điển còn mở là một bản cập nhật không thay được tệp đó.
pub struct ReadOnlyDb {
    kind: StoreKind,
    path: PathBuf,
    readers: reader::ReaderPool,
}

impl ReadOnlyDb {
    /// Mở một tệp `.db` **đã tồn tại**, chỉ đọc.
    ///
    /// 🔴 Đường dẫn không tồn tại ⇒ [`StoreError::OpenFailed`], và **không một tệp
    /// nào được tạo ra** — cờ mở không có `SQLITE_OPEN_CREATE`. Xem
    /// `pragmas::open_readonly_connection` về vì sao vế đó là hợp đồng chứ không phải
    /// cẩn thận thừa.
    ///
    /// **Không đọc `PRAGMA user_version`, không di trú, không kiểm phiên bản
    /// lược đồ ở đây.** Việc từ chối một tệp mới hơn ứng dụng là quyết định của tầng gọi
    /// (Story 1.13, nơi biết mình đang mở *lớp* nào và làm gì khi một lớp bị từ chối);
    /// đặt nó ở đây là chôn một chính sách vào một cơ chế.
    pub fn open(path: PathBuf, kind: StoreKind) -> Result<ReadOnlyDb, StoreError> {
        // 🔵 SỬA (2026-08-29, Story 5.9) — danh sách cho phép mở rộng từ `{Dict}` sang
        // `{Dict, Project}`, một MIỄN TRỪ CÓ TÊN, không một cửa mở tuỳ ý.
        //
        // Vì sao `Project` được thêm: `Indexer::rebuild` (`core/library/indexer.rs`) phải THU
        // HOẠCH văn bản từ `project.db` của mỗi `.atproj` để dựng `library_segment`/hai chỉ
        // mục FTS5 (Story 5.9, FR8) — và nó đọc một Tác phẩm mà chính lượt quét **không sở
        // hữu** (người dùng có thể đang MỞ đúng Tác phẩm đó ở một Store khác cùng lúc). Bốn
        // thứ `Store::open` sẽ ghi vào tệp (`readonly.rs:19-24`: `READ_WRITE | CREATE`,
        // `journal_mode = WAL`, bộ di trú, luồng writer) đều SAI ở đây — ba trong bốn là GHI
        // vào một `.atproj` mà lượt quét không sở hữu, và cái thứ ba (bộ di trú) còn nguy hiểm
        // riêng: nó sẽ DI TRÚ HÀNG LOẠT cả thư viện chỉ vì người dùng mở Library.
        //
        // Vì sao vẫn `debug_assert_eq!`-shape (không nới thành `StoreKind` bất kỳ): một kind
        // THỨ BA lọt vào đây (`Global`/`Project` dùng sai chỗ/`LibraryIndex`) là một lỗi lập
        // trình, không một trường hợp hợp lệ chưa tính tới — miễn trừ phải CÓ TÊN và phải CHẾT
        // ĐƯỢC (khi Story nào đó thật sự cần mở chỉ-đọc một kind thứ ba, danh sách này lại mở
        // rộng CÓ CHỦ, không tự nó nới ra).
        debug_assert!(
            matches!(kind, StoreKind::Dict | StoreKind::Project),
            "ReadOnlyDb::open is for StoreKind::Dict or StoreKind::Project only; every other \
             kind goes through Store::open"
        );

        // ⚠️ `Tuning::default()` dùng nguyên, nhưng **chỉ hai trường của nó có nghĩa ở
        // đây**: `pool_size` (số kết nối) và `busy_timeout` (trạng thái từng kết nối).
        // Bốn trường còn lại — `checkpoint_tick`, `idle_before_passive`,
        // `wal_threshold_bytes`, `close_truncate_budget` — đều nói về WAL và về luồng
        // checkpoint, và một tệp chỉ đọc không có cả hai. Chúng bị bỏ qua, không
        // phải bị quên.
        let tuning = Tuning::default();

        let readers = reader::ReaderPool::open_readonly(&path, kind, &tuning)?;

        Ok(ReadOnlyDb {
            kind,
            path,
            readers,
        })
    }

    /// Chạy một job ĐỌC trên một kết nối mượn từ pool. Chữ ký y hệt [`Store::read`].
    ///
    /// 🔴 Chỉ-đọc ở đây được **SQLite** cưỡng chế hai lần: cờ mở `SQLITE_OPEN_READ_ONLY`
    /// ở tầng tệp, và `PRAGMA query_only = 1` ở tầng câu lệnh. Một `INSERT` gửi qua
    /// đường này trả [`StoreError::ReadFailed`] — đó là **bằng chứng dương** của AC7,
    /// không phải một sự cố.
    pub fn read<T, F>(&self, job: F) -> Result<T, StoreError>
    where
        F: FnOnce(ReadHandle<'_>) -> SqlResult<T>,
    {
        self.readers.read(job)
    }

    /// Đóng pool: thả mọi kết nối. Idempotent.
    ///
    /// Không TRUNCATE, không checkpoint — không có WAL nào để cắt. Đó là toàn bộ
    /// khác biệt với [`Store::close`], và nó là hệ quả của việc tệp này chưa bao giờ
    /// được ghi.
    pub fn close(&self) {
        self.readers.close();
    }

    /// Loại kho — [`StoreKind::Dict`] cho một tệp từ điển, [`StoreKind::Project`] cho một
    /// `project.db` mở chỉ-đọc để thu hoạch (Story 5.9).
    pub const fn kind(&self) -> StoreKind {
        self.kind
    }

    /// Đường dẫn tệp `.db`.
    pub fn path(&self) -> &Path {
        &self.path
    }
}

impl Drop for ReadOnlyDb {
    fn drop(&mut self) {
        self.close();
    }
}

impl std::fmt::Debug for ReadOnlyDb {
    /// Không in `Connection` — nó không `Debug` và cũng không có gì đọc được.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ReadOnlyDb")
            .field("kind", &self.kind.as_str())
            .field("path", &self.path)
            .finish_non_exhaustive()
    }
}
