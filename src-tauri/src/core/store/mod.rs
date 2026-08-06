//! Tầng ghi dữ liệu: `Writer` nối tiếp + `Reader` pool + checkpoint (AD-11, AD-12).
//!
//! MỘT writer duy nhất cho mỗi kho ghi được (AD-11). Thời điểm checkpoint là quyết
//! định của ứng dụng, không phó mặc SQLite (AD-12). Lược đồ có phiên bản; mở tiến,
//! không bao giờ mở lùi (AD-30).
//!
//! Đường dẫn `$APPDATA` LUÔN lấy qua `app.path().app_data_dir()` — không viết cứng.
//! Đây là chỗ NFR14 (hành vi tương đương hai nền tảng) hỏng đầu tiên.
//!
//! Crate dành cho module này: `rusqlite` (feature `bundled`) · `libsqlite3-sys`.
//! Story 1.7 sở hữu nội dung.
//!
//! ─────────────────────────────────────────────────────────────────────────────
//! HÌNH DẠNG ĐÃ DỰNG (Story 1.7)
//! ─────────────────────────────────────────────────────────────────────────────
//!
//! Một [`Store`] gói bốn thứ, và mỗi thứ tồn tại vì một AC cụ thể:
//!
//! - **[`writer`]** — MỘT `Connection` ghi sống trong MỘT luồng, nhận việc qua
//!   `std::sync::mpsc`. Đây là toàn bộ AC1 vế ghi và AC2 vế "mọi ghi đi qua
//!   `store::Writer`": không có đường thứ hai để ghi vì không có kết nối ghi thứ hai.
//! - **[`reader`]** — pool `Tuning::pool_size` kết nối, mỗi cái đặt `query_only = 1`.
//!   Cưỡng chế chỉ-đọc là của SQLite, không của kỷ luật người viết (Quyết định #2).
//! - **[`checkpoint`]** — luồng nền trên `Connection` RIÊNG. PASSIVE khi rảnh hoặc khi
//!   `.db-wal` vượt ngưỡng; TRUNCATE chỉ ở [`Store::close`] và ngay trước khi sao lưu
//!   để di trú. ⛔ TRUNCATE **không bao giờ** ở đường nền — nó chờ mọi reader rời đi,
//!   và đặt nó vào nhịp nền là dựng lại đúng cái gai trễ mà `wal_autocheckpoint = 0`
//!   vừa gỡ ra (AD-12 / NFR2).
//! - **[`schema`]** — `PRAGMA user_version`, từ chối mở lùi, di trú chỉ tiến.
//!
//! ─────────────────────────────────────────────────────────────────────────────
//! 🔴 THỨ TỰ TRONG [`Store::open`] LÀ HỢP ĐỒNG, KHÔNG PHẢI SỞ THÍCH
//! ─────────────────────────────────────────────────────────────────────────────
//! ```text
//! mở kết nối
//!   → ĐỌC `PRAGMA user_version`          (chỉ đọc, không ghi một byte nào)
//!   → lớn hơn target ⇒ ĐÓNG và trả lỗi NGAY
//!   → mới đặt ba PRAGMA của AC3
//!   → mới sao lưu
//!   → mới di trú
//! ```
//! Lý do thứ tự này không đảo được: `PRAGMA journal_mode = WAL` **GHI VÀO** database —
//! chuyển một tệp từ `delete` sang `wal` viết lại header. Thứ tự tự nhiên nhất để viết
//! (mở, đặt PRAGMA cho xong, rồi mới xét lược đồ) vi phạm AC7 nguyên văn *"không ghi
//! vào nó một byte nào"*, và nó vi phạm im lặng: không lỗi nào được ném, chỉ có một
//! băm tệp khác đi.
//!
//! ⚠️ Quy ước phiên bản, khai tường minh vì `PRAGMA user_version` mặc định là 0 nên
//! *"database mới tinh"* và *"database ở phiên bản 0"* không phân biệt được:
//! **0 = chưa có lược đồ**, bước di trú đầu tiên đánh số **1**.
//!
//! ─────────────────────────────────────────────────────────────────────────────
//! ⛔ MỌI CHUỖI TRONG MODULE NÀY VIẾT KHÔNG DẤU
//! ─────────────────────────────────────────────────────────────────────────────
//! `scripts/check-i18n.mjs` Kiểm A quét `src-tauri/**/*.rs` tìm ký tự có dấu tiếng Việt
//! **ở vị trí mã**. `src-tauri/tests/**` được miễn trừ có tên; `src/core/store/**` thì
//! **không**. Doc-comment và comment có dấu là hợp lệ — thông báo chẩn đoán, `Display`,
//! `debug_assert!` thì không. (Cổng đã bắt đúng ca này một lần ở `core/i18n/mod.rs`
//! trong lượt review 2026-08-04.)
//!
//! ─────────────────────────────────────────────────────────────────────────────
//! BA + BA CON SỐ `Tuning` LÀ **TẠM** — CHỦ SỞ HỮU LÀ **STORY 2.4**
//! ─────────────────────────────────────────────────────────────────────────────
//! Không con số nào ở [`Tuning::default`] được đo. Chúng không đo được hôm nay vì phép
//! đo cần Editor thật: `wal_threshold_bytes` và nhịp flush của AD-35 **đánh đổi lẫn
//! nhau** — phải đạt NFR18 *(mất ≤ 5 s)* mà không phạm NFR2 *(không frame nào vượt
//! 50 ms)*. `ARCHITECTURE-SPINE.md#Deferred` và `epics.md:454` xếp cả cặp vào Giai đoạn 2.
//! ⛔ Đừng đọc các số này như đã hiệu chỉnh; xem `deferred-work.md`.
//!
//! ─────────────────────────────────────────────────────────────────────────────
//! ⛔ MODULE NÀY KHÔNG `use tauri::…` (Quyết định #1)
//! ─────────────────────────────────────────────────────────────────────────────
//! [`Store::open`] nhận một [`StoreSpec`] mang `PathBuf` đã phân giải. Đường lấy
//! `$APPDATA` sống ở `lib.rs`. Ba lý do, cả ba đo được: test dựng `Store` trên thư mục
//! tạm mà không cần webview (khác biệt giữa 13 ca chạy trong `cargo test` và một bảng
//! nghiệm thu tay); `project.db` của Story 1.15 nằm trong một `.atproj` do người dùng
//! chọn, **không** phải `$APPDATA`; và luật *"đường dẫn `$APPDATA` luôn lấy qua
//! `app.path()`"* áp cho **chỗ gọi**, module thì nhận đường dẫn đã phân giải.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;

use crate::core::i18n::{IpcError, MessageKey};

pub(crate) mod checkpoint;
pub(crate) mod pragmas;
pub(crate) mod reader;
pub mod readonly;
pub(crate) mod schema;
pub(crate) mod writer;

pub use checkpoint::CheckpointStats;
pub use readonly::ReadOnlyDb;
pub use schema::{
    CHAPTER_DDL, CONFIG_VALUE_DDL, GLOBAL_MIGRATIONS, Migration, PROJECT_MIGRATIONS,
    SCHEMA_MIGRATION_LOG_DDL, WORK_DDL,
};

/// Kiểu giao dịch mà một job ghi nhận được. Tái xuất để chỗ gọi **không phải gõ
/// `rusqlite`** — xem [`ReadHandle`] cho cùng lý do.
pub use rusqlite::Transaction;
/// Lỗi thô của SQLite, cho chỗ gọi cần rẽ nhánh trên nó bên trong một job.
pub use rusqlite::Error as SqlError;
/// `Result` của một job ghi/đọc, trước khi nó được bọc thành [`StoreError`].
pub use rusqlite::Result as SqlResult;
/// Một hàng kết quả, cho các hàm trợ giúp nhận `&Row` ở chỗ gọi.
pub use rusqlite::Row;
/// Trait ràng buộc tham số, cho chỗ gọi cần một danh sách tham số **không đồng nhất kiểu**.
///
/// ⚠️ Tái xuất vì cùng lý do với bốn kiểu trên: không có nó, một module muốn viết
/// `&[&dyn ToSql]` phải gõ tên crate, và `store_boundary.rs` — đúng như nó phải làm —
/// sẽ gọi đó là một vi phạm.
pub use rusqlite::ToSql;

/// Kết nối đọc đã đặt `PRAGMA query_only = 1`.
///
/// 🔴 **Bí danh chứ không phải kiểu mới, và đó là chủ ý.** AC2 đòi
/// `rusqlite::Connection` không xuất hiện trong chữ ký `pub` nào thoát khỏi module;
/// bí danh này giữ đúng điều đó *(chỗ gọi viết `|conn| …` và không bao giờ gõ tên
/// `rusqlite`)* mà không phải bọc lại từng phương thức của `Connection` — một lớp bọc
/// như vậy là hàng trăm dòng chuyển tiếp mà **không thêm một phép cưỡng chế nào**.
///
/// Phép cưỡng chế thật nằm ở hai chỗ khác, và cả hai đều là máy chứ không phải kỷ luật:
/// 1. `query_only = 1` — **SQLite** từ chối mọi lệnh ghi trên kết nối này (Quyết định #2).
/// 2. `tests/store_boundary.rs` — `rusqlite` chỉ được xuất hiện dưới `src/core/store/**`.
pub type ReadHandle<'a> = &'a rusqlite::Connection;

/// Năm loại kho của AD-7 mà story này chạm tới — **khai hết, dựng đúng một**.
///
/// ⛔ [`StoreKind::Global`] và [`StoreKind::Project`] có mã khởi tạo hôm nay — Story 1.15
/// dựng vế thứ hai (`project.db`, nằm trong một `.atproj` do người dùng chọn, ⛔ không phải
/// `$APPDATA`). `library-index.db` là **Epic 5**, và AD-8 còn nói nó **không di trú** — xoá
/// rồi dựng lại — tức nó cần một nhánh khác mà story đó phải tự quyết. Viết sẵn mã cho loại
/// kia hôm nay là mã không ai gọi, và nó sẽ sai theo đúng cách mà không test nào bắt.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum StoreKind {
    /// `$APPDATA/global.db` — cấu hình, glossary chung, TM chung, phím tắt.
    Global,
    /// `<tác phẩm>.atproj/project.db` — **Story 1.15**.
    Project,
    /// `$APPDATA/library-index.db` — chỉ mục dẫn xuất, **Epic 5**, ⛔ không di trú (AD-8).
    LibraryIndex,

    /// Một tệp từ điển `.db` — **CHỈ ĐỌC, LUÔN LUÔN** (AD-7). Story 1.11.
    ///
    /// 🔴 Loại này khác hẳn ba loại trên và cái khác nằm ở chỗ nó **không có** gì:
    /// ⛔ không [`StoreSpec`], ⛔ không writer, ⛔ không luồng checkpoint, ⛔ không bộ
    /// di trú, ⛔ không `journal_mode = WAL`. Nó đi qua [`ReadOnlyDb`], ⛔ không qua
    /// [`Store`] — vì cả bốn thứ vừa kể đều **GHI VÀO** tệp, và một tệp từ điển được
    /// giao kèm checksum trong `dict-manifest.toml` (AD-25). Ghi vào nó một byte là
    /// làm checksum thành sai, và ⛔ không cổng nào bắt được điều đó
    /// (`check-dict-manifest.mjs` cố ý ⛔ không đọc `.db`).
    ///
    /// ⚠️ Cả ba tệp từ điển ở `journal_mode = delete` — `tools/dict-build` đặt thế có
    /// chủ ý (`finalize.rs`). Nên `apply_reader_pragmas` (nó gọi `verify_wal`) ⛔ không
    /// dùng được ở đây; đường của loại này là `apply_dict_reader_pragmas`.
    Dict,
}

impl StoreKind {
    /// Định danh máy đọc. ⛔ Không phải nhãn hiển thị — nó đi vào `params` của
    /// [`IpcError`], nơi AD-21 chỉ cho phép **dữ liệu**, không cho phép câu.
    pub const fn as_str(self) -> &'static str {
        match self {
            StoreKind::Global => "global",
            StoreKind::Project => "project",
            StoreKind::LibraryIndex => "library-index",
            StoreKind::Dict => "dict",
        }
    }
}

/// Sáu con số điều khiển tầng ghi. **Tất cả đều TẠM** — xem doc-comment của module.
///
/// ⛔ Đừng chôn chúng thành số trần rải rác trong `writer.rs` / `checkpoint.rs`. Một
/// `struct` với `Default` là điều kiện để Story 2.4 hiệu chỉnh được bằng **một** lượt
/// sửa thay vì một lượt đi săn; và là điều kiện để test lái cơ chế bằng `Tuning` thu
/// nhỏ *(tick và idle tính bằng chục mili-giây)* thay vì chờ 5 giây thật — nhân với hai
/// nền tảng CI thì đó là phút, và §Testing standards cấm.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Tuning {
    /// Số kết nối đọc song song. **Tạm: 4.** Đủ để quan sát được đọc chồng nhau (AC1);
    /// nhỏ để TRUNCATE ở [`Store::close`] không phải chờ nhiều reader rời đi.
    ///
    /// ⚠️ Kéo theo của Bẫy 8: WAL chỉ được dùng lại khi một lượt checkpoint chép **hết**.
    /// Một reader giữ ảnh chụp cũ làm `log > checkpointed` và tệp vẫn lớn tiếp — nên pool
    /// lớn hơn không "nhanh hơn", nó chỉ làm ngưỡng của AC5 khó đúng hơn.
    pub pool_size: usize,

    /// `PRAGMA busy_timeout`. **Tạm: 5 000 ms.** Dài hơn hẳn một lượt checkpoint bình
    /// thường, ngắn hơn ngưỡng mà người dùng cho là treo.
    ///
    /// ⚠️ Là trạng thái CỦA TỪNG KẾT NỐI, không phải của database — writer, mỗi kết nối
    /// pool, và luồng checkpoint đều phải tự đặt. Quên trên pool nghĩa là reader nhận
    /// `SQLITE_BUSY` **ngay lập tức** trong lúc TRUNCATE chạy, biểu hiện thành "thỉnh
    /// thoảng tra cứu lỗi" và không tái lập được.
    pub busy_timeout: Duration,

    /// Nhịp thức dậy của luồng checkpoint. **Tạm: 1 s.** Độ phân giải của cả hai điều
    /// kiện kích hoạt.
    pub checkpoint_tick: Duration,

    /// Rảnh bao lâu thì chạy PASSIVE. **Tạm: 5 s.**
    ///
    /// ⚠️ **Cố ý dài hơn** nhịp flush 2 s của AD-35, để checkpoint không đánh nhau với
    /// đường gõ. Đây là con số nhạy nhất của cả sáu với cặp NFR2/NFR18.
    pub idle_before_passive: Duration,

    /// `.db-wal` vượt cỡ này thì checkpoint chạy **kể cả khi chưa rảnh** (AC5).
    /// **Tạm: 4 MiB.**
    ///
    /// Bằng đúng ngưỡng autocheckpoint mặc định của SQLite *(1000 trang × 4096 B)* mà
    /// AC3 vừa tắt — lấy lại đúng số nó bỏ lại, tức không đổi hành vi theo một hướng
    /// chưa ai đo.
    pub wal_threshold_bytes: u64,

    /// Trần thời gian cho lượt TRUNCATE ở [`Store::close`]. **Tạm: 2 s.**
    ///
    /// 🔴 Trần này không phải để cho đẹp. `scripts/check-scope.mjs` và
    /// `check-scope-bundled.mjs` chạy nhị phân **với timeout cứng** rồi đọc dòng
    /// `VERDICT:`; một `close()` chờ hết `busy_timeout` biến thành *"self-check chưa
    /// chạy tới nơi"* và làm **hai cổng của Story 1.2/1.3 đỏ vì tầng ghi dữ liệu**,
    /// không vì phạm vi mà chúng canh.
    pub close_truncate_budget: Duration,
}

impl Default for Tuning {
    fn default() -> Self {
        Self {
            pool_size: 4,
            busy_timeout: Duration::from_millis(5_000),
            checkpoint_tick: Duration::from_secs(1),
            idle_before_passive: Duration::from_secs(5),
            wal_threshold_bytes: 4 * 1024 * 1024,
            close_truncate_budget: Duration::from_secs(2),
        }
    }
}

/// Mô tả một kho: nó là kho gì, nằm ở đâu, chạy với con số nào, và **bộ di trú nào**.
///
/// ⚠️ `migrations` là một trường chứ không phải một hằng tra theo `kind`, và đó là một
/// quyết định chứ không phải một chỗ để ngỏ: nó là cách duy nhất nghiệm thu được AC6 vế
/// *"một bước di trú ném lỗi giữa chừng ⇒ rollback"* mà **không** phải thêm mã sản phẩm
/// chỉ để test gọi. Story 1.15 sẽ dùng đúng trường này cho `project.db`.
#[derive(Debug, Clone)]
pub struct StoreSpec {
    /// Loại kho — đi vào `params` của lỗi, nên chẩn đoán nói được *kho nào* hỏng.
    pub kind: StoreKind,
    /// Đường dẫn tệp `.db` **đã phân giải**. ⛔ Module này không tự phân giải `$APPDATA`.
    pub path: PathBuf,
    /// Sáu con số tạm. Xem [`Tuning`].
    pub tuning: Tuning,
    /// Các bước di trú, **thứ tự tăng dần theo `to_version`**.
    pub migrations: &'static [Migration],
}

impl StoreSpec {
    /// Kho `global.db` với bộ di trú và `Tuning` mặc định.
    ///
    /// ⛔ Không có `StoreSpec::library_index` hôm nay — xem [`StoreKind`].
    pub fn global(path: PathBuf) -> Self {
        Self {
            kind: StoreKind::Global,
            path,
            tuning: Tuning::default(),
            migrations: GLOBAL_MIGRATIONS,
        }
    }

    /// Kho `project.db` của một `.atproj` — **Story 1.15**, bộ di trú [`PROJECT_MIGRATIONS`]
    /// và `Tuning` mặc định.
    ///
    /// `path` **đã phân giải** — chỗ gọi (`core::library::atproj`) quyết định
    /// `<Tên>.atproj/project.db` nằm ở đâu; module này không tự đoán.
    pub fn project(path: PathBuf) -> Self {
        Self {
            kind: StoreKind::Project,
            path,
            tuning: Tuning::default(),
            migrations: PROJECT_MIGRATIONS,
        }
    }
}

/// Mọi cách tầng ghi dữ liệu hỏng — và **mỗi biến thể mang sẵn một [`MessageKey`]**.
///
/// Vì sao khoá nằm ở đây từ hôm nay chứ không đợi tới lúc có gì hiển thị nó: Story 1.8
/// là đường IPC thật đầu tiên có nhu cầu đọc/ghi qua ranh giới, và nếu `StoreError`
/// không mang khoá thì story đó phải **phát minh một từ vựng lỗi thứ hai ở chỗ gọi** —
/// đúng hình dạng mà `core/i18n` tồn tại để chặn. Ở đây nó chỉ phải nối dây.
///
/// ⚠️ `detail` **không bao giờ** đi vào `params`. Nó là văn bản lỗi thô của SQLite —
/// một câu, tiếng Anh, cho người đang chẩn đoán. AD-21 nói `params` mang **dữ liệu**;
/// một `params: {"reason": "<câu>"}` là AD-21 bị thủng qua cửa sau.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StoreError {
    /// Không mở được tệp, hoặc di trú gãy. `detail` là lỗi thô, chỉ để chẩn đoán.
    OpenFailed {
        /// Kho nào.
        store: StoreKind,
        /// Lỗi thô. ⛔ Không đi lên giao diện.
        detail: String,
    },

    /// 🔴 `PRAGMA journal_mode = WAL` đặt xong mà **đọc lại ra chế độ khác**.
    ///
    /// Đây là cả lý do tồn tại của luật "đặt rồi ĐỌC LẠI": `pragma_update` gọi
    /// `execute_batch`, và `execute_batch` trong `rusqlite` 0.40.1 **cố ý nuốt** hàng trả
    /// về của PRAGMA (`src/lib.rs:555-560`, nhánh `if false` là no-op của thượng nguồn).
    /// `journal_mode` trả về chế độ mới dưới dạng một hàng, và hàng đó bị vứt — nên trên
    /// một thư mục mà WAL không dùng được, lệnh trả `Ok(())`, database ở lại `delete`,
    /// **mọi bảo đảm của NFR2 và NFR18 biến mất**, và không lỗi nào được ném.
    WalUnavailable {
        /// Kho nào.
        store: StoreKind,
        /// Chế độ ĐỌC ĐƯỢC VỀ — dữ liệu, không phải câu.
        mode: String,
    },

    /// Database mang phiên bản lược đồ **mới hơn** ứng dụng. ⛔ Không ghi một byte nào.
    SchemaTooNew {
        /// Kho nào.
        store: StoreKind,
        /// `PRAGMA user_version` đọc được.
        found: u32,
        /// Phiên bản cao nhất bản ứng dụng này hiểu.
        supported: u32,
    },

    /// Job ghi chạy nhưng trả lỗi ⇒ giao dịch đã rollback, ⛔ không có nửa ghi nào.
    WriteFailed {
        /// Kho nào.
        store: StoreKind,
        /// Lỗi thô. ⛔ Không đi lên giao diện.
        detail: String,
    },

    /// Luồng writer không còn nhận việc — kho đã đóng, hoặc luồng đã chết.
    ///
    /// 🔴 Biến thể này tồn tại để [`Store::write`] **trả về trong thời gian hữu hạn**
    /// thay vì chặn mãi. Một `recv()` treo trên đường ghi là NFR2 chết mà không lỗi nào
    /// được ném.
    WriterGone {
        /// Kho nào.
        store: StoreKind,
    },

    /// Job đọc chạy nhưng trả lỗi. ⚠️ Một `INSERT` qua [`Store::read`] rơi vào đây —
    /// và đó là **bằng chứng dương** của AC2, không phải một sự cố.
    ReadFailed {
        /// Kho nào.
        store: StoreKind,
        /// Lỗi thô. ⛔ Không đi lên giao diện.
        detail: String,
    },

    /// Pool đọc đã đóng ([`Store::close`] đã chạy).
    PoolClosed {
        /// Kho nào.
        store: StoreKind,
    },
}

impl StoreError {
    /// Kho mà lỗi này nói về.
    pub const fn store(&self) -> StoreKind {
        match self {
            StoreError::OpenFailed { store, .. }
            | StoreError::WalUnavailable { store, .. }
            | StoreError::SchemaTooNew { store, .. }
            | StoreError::WriteFailed { store, .. }
            | StoreError::WriterGone { store }
            | StoreError::ReadFailed { store, .. }
            | StoreError::PoolClosed { store } => *store,
        }
    }

    /// Khoá chuỗi mà [`IpcError`] sẽ mang. Xem [`StoreError`] về vì sao nó ở đây.
    pub const fn message_key(&self) -> MessageKey {
        match self {
            StoreError::OpenFailed { .. } => MessageKey::StoreOpenFailed,
            StoreError::WalUnavailable { .. } => MessageKey::StoreWalUnavailable,
            StoreError::SchemaTooNew { .. } => MessageKey::StoreSchemaTooNew,
            StoreError::WriteFailed { .. } | StoreError::WriterGone { .. } => {
                MessageKey::StoreWriteFailed
            }
            StoreError::ReadFailed { .. } | StoreError::PoolClosed { .. } => {
                MessageKey::StoreReadFailed
            }
        }
    }

    /// `code` — định danh máy đọc để frontend rẽ nhánh. ⛔ Không bao giờ ra màn hình.
    ///
    /// ⚠️ Hẹp hơn [`StoreError::message_key`] có chủ ý: `WriterGone` và `WriteFailed`
    /// dùng **chung** một câu cho người dùng *(thay đổi vừa rồi chưa được lưu)* nhưng là
    /// **hai** tình huống khác hẳn nhau khi chẩn đoán. AD-21 cho phép đúng điều đó —
    /// `code` và `message_key` là hai trường, không phải một trường hai tên.
    pub const fn code(&self) -> &'static str {
        match self {
            StoreError::OpenFailed { .. } => "store.open_failed",
            StoreError::WalUnavailable { .. } => "store.wal_unavailable",
            StoreError::SchemaTooNew { .. } => "store.schema_too_new",
            StoreError::WriteFailed { .. } => "store.write_failed",
            StoreError::WriterGone { .. } => "store.writer_gone",
            StoreError::ReadFailed { .. } => "store.read_failed",
            StoreError::PoolClosed { .. } => "store.pool_closed",
        }
    }

    /// Quyền hiển thị một nút thử lại. ⛔ Không phải lệnh tự thử lại (AD-22).
    ///
    /// Một lượt ghi/đọc trượt vì `SQLITE_BUSY` **có thể** thành công lần sau, nên người
    /// dùng được quyền bấm lại. Lược đồ quá mới thì không — bấm bao nhiêu lần cũng vậy,
    /// và một nút thử lại ở đó là nói dối.
    pub const fn retryable(&self) -> bool {
        matches!(
            self,
            StoreError::WriteFailed { .. } | StoreError::ReadFailed { .. }
        )
    }
}

impl std::fmt::Display for StoreError {
    /// ⚠️ KHÔNG DẤU — xem doc-comment của module. Đây là chẩn đoán cho log, không phải
    /// văn bản hiển thị; văn bản hiển thị sống ở `src/i18n/vi.json` và chỉ ở đó (NFR16).
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StoreError::OpenFailed { store, detail } => {
                write!(f, "store[{}] open failed: {detail}", store.as_str())
            }
            StoreError::WalUnavailable { store, mode } => write!(
                f,
                "store[{}] journal_mode read back as {mode:?}, expected \"wal\"",
                store.as_str()
            ),
            StoreError::SchemaTooNew {
                store,
                found,
                supported,
            } => write!(
                f,
                "store[{}] schema version {found} is newer than supported {supported}",
                store.as_str()
            ),
            StoreError::WriteFailed { store, detail } => {
                write!(f, "store[{}] write failed: {detail}", store.as_str())
            }
            StoreError::WriterGone { store } => {
                write!(f, "store[{}] writer thread is gone", store.as_str())
            }
            StoreError::ReadFailed { store, detail } => {
                write!(f, "store[{}] read failed: {detail}", store.as_str())
            }
            StoreError::PoolClosed { store } => {
                write!(f, "store[{}] reader pool is closed", store.as_str())
            }
        }
    }
}

impl std::error::Error for StoreError {}

/// 🔴 Đi **qua [`IpcError::new`]**, ⛔ không dựng struct literal.
///
/// `IpcError::new` là chỗ DUY NHẤT `message_key` gặp `params`, và đó là chỗ duy nhất
/// một khoá thiếu tham số bị bắt. Một struct literal đi vòng qua nó biên dịch sạch, qua
/// mọi cổng hiện có, rồi đặt nguyên văn `{store}` lên màn hình người dùng — xem
/// doc-comment của `IpcError`.
impl From<StoreError> for IpcError {
    fn from(err: StoreError) -> Self {
        let mut params = BTreeMap::new();
        params.insert("store".to_owned(), err.store().as_str().to_owned());

        match &err {
            StoreError::WalUnavailable { mode, .. } => {
                params.insert("mode".to_owned(), mode.clone());
            }
            StoreError::SchemaTooNew {
                found, supported, ..
            } => {
                // ⚠️ Số đi trên dây dưới dạng CHUỖI, kể cả khi nó là số: định dạng số và
                // ngày giờ chỉ ở frontend (Consistency Conventions). Hợp đồng của `params`
                // là `chuỗi -> chuỗi`.
                params.insert("found".to_owned(), found.to_string());
                params.insert("supported".to_owned(), supported.to_string());
            }
            _ => {}
        }

        IpcError::new(err.code(), err.message_key(), params, err.retryable())
    }
}

/// Một kho ghi được: một writer nối tiếp, một pool đọc, một luồng checkpoint.
///
/// `Send + Sync` mà **không cần bọc `Mutex` ở chỗ gọi** — điều kiện để nó vào
/// `app.manage(…)`: `std::sync::mpsc::Sender<T>` là `Sync` kể từ Rust 1.72, toolchain CI
/// là 1.97.1 và `rust-version` khai 1.85.
///
/// ⚠️ [`Store::close`] chạy trong [`Drop`] nếu chưa ai gọi. Đó không phải phép lịch sự:
/// chỉ TRUNCATE mới cắt `.db-wal` về 0, và trên Windows một tệp còn mở là một
/// `remove_dir_all` thất bại (NFR14) — đúng lớp lỗi mà CI hai nền tảng của Story 1.3
/// dựng ra để bắt.
pub struct Store {
    kind: StoreKind,
    path: PathBuf,
    schema_version: u32,
    writer: writer::Writer,
    readers: reader::ReaderPool,
    checkpoint: checkpoint::Checkpointer,
    shared: Arc<checkpoint::Shared>,
}

impl Store {
    /// Mở (hoặc tạo) kho. **Thứ tự các bước là hợp đồng** — xem doc-comment của module.
    pub fn open(spec: StoreSpec) -> Result<Store, StoreError> {
        let StoreSpec {
            kind,
            path,
            tuning,
            migrations,
        } = spec;

        // ── 0. `Dict` đi qua `ReadOnlyDb`, ⛔ không bao giờ qua đây ───────────────
        //
        // 🔴 `StoreSpec` mọi trường đều `pub`, nên hệ kiểu không tự ngăn ai đó dựng
        // `StoreSpec { kind: StoreKind::Dict, .. }` rồi gọi `Store::open`. Nếu lọt qua,
        // bước 4 dưới đây đặt `journal_mode = WAL` — GHI VÀO tệp, làm checksum của
        // `dict-manifest.toml` (AD-25) thành sai. Chặn ở đây, sớm hơn cả bước 1, để
        // ⛔ không byte nào của tệp từ điển bị chạm dù chỉ bằng cách mở kết nối ghi.
        if kind == StoreKind::Dict {
            return Err(StoreError::OpenFailed {
                store: kind,
                detail: "Store::open refuses StoreKind::Dict; dictionary files are read-only \
                         and must open through ReadOnlyDb::open instead"
                    .to_string(),
            });
        }

        // ── 1. Mở kết nối ghi bằng CỜ TƯỜNG MINH ─────────────────────────────────
        let mut conn = pragmas::open_connection(&path, kind)?;

        // ── 2. ĐỌC phiên bản. Chỉ đọc. Chưa một byte nào được ghi. ────────────────
        let found = schema::read_user_version(&conn, kind)?;

        // Kiểm bất biến của bộ di trú TRƯỚC khi tin `target`: bước 3 ngay dưới đây dùng
        // `target` để quyết định từ chối mở (AC7). `schema::migrate` kiểm lại bất biến
        // này lần nữa trước khi chạy — xem doc-comment của `validate_strictly_increasing`
        // về vì sao một lần kiểm ở đó là không đủ.
        schema::validate_strictly_increasing(migrations, kind)?;
        let target = schema::target_version(migrations);

        // ── 3. Mới hơn ứng dụng ⇒ ĐÓNG và trả lỗi NGAY (AC7) ─────────────────────
        //
        // 🔴 `drop` tường minh chứ không để cuối scope: bước 4 ngay dưới đây GHI VÀO
        // database, và một `return` sau nó là AC7 trượt im lặng.
        if found > target {
            drop(conn);
            return Err(StoreError::SchemaTooNew {
                store: kind,
                found,
                supported: target,
            });
        }

        // ── 4. Ba PRAGMA của AC3 — đặt RỒI ĐỌC LẠI ───────────────────────────────
        pragmas::apply_writer_pragmas(&conn, kind, &tuning)?;

        // ── 5. Sao lưu TRƯỚC bước di trú đầu tiên, và chỉ khi đã có lược đồ ───────
        //
        // `found == 0` nghĩa là chưa có lược đồ (xem `schema.rs`) — không có gì để sao
        // lưu, và một tệp `.bak-v0` rỗng chỉ làm người đọc tưởng mình có đường lui.
        if found >= 1 && found < target {
            schema::backup_before_migration(&conn, &path, kind, found)?;
        }

        // ── 6. Di trú chỉ tiến, mỗi bước một giao dịch ────────────────────────────
        let schema_version = schema::migrate(&mut conn, kind, found, migrations)?;

        // ── 7. Pool đọc — mỗi kết nối tự đặt PRAGMA của nó (Bẫy 3) ───────────────
        let shared = Arc::new(checkpoint::Shared::new());
        let readers = reader::ReaderPool::open(&path, kind, &tuning)?;

        // ── 8. Luồng checkpoint trên kết nối RIÊNG ───────────────────────────────
        let checkpoint = checkpoint::Checkpointer::spawn(&path, kind, tuning, Arc::clone(&shared))?;

        // ── 9. Luồng writer nhận quyền sở hữu kết nối ghi ─────────────────────────
        let writer = writer::Writer::spawn(conn, kind, Arc::clone(&shared))?;

        Ok(Store {
            kind,
            path,
            schema_version,
            writer,
            readers,
            checkpoint,
            shared,
        })
    }

    /// Chạy một job GHI trên writer nối tiếp. **Chặn** cho tới khi có kết quả.
    ///
    /// Mỗi job là **một giao dịch**: trả `Ok` ⇒ commit, trả `Err` ⇒ rollback. Không có
    /// đường để một job commit nửa chừng.
    ///
    /// 🔴 Trả về trong thời gian hữu hạn kể cả khi luồng writer đã đi mất — xem
    /// [`StoreError::WriterGone`]. ⛔ Không bao giờ treo.
    pub fn write<T, F>(&self, job: F) -> Result<T, StoreError>
    where
        F: FnOnce(&Transaction<'_>) -> SqlResult<T> + Send + 'static,
        T: Send + 'static,
    {
        self.writer.write(job)
    }

    /// Chạy một job ĐỌC trên một kết nối mượn từ pool.
    ///
    /// 🔴 Kết nối đã đặt `query_only = 1`, nên **SQLite** từ chối mọi lệnh ghi ở đây —
    /// không phải người viết tự nhớ. Một `INSERT` qua đường này trả
    /// [`StoreError::ReadFailed`], và đó là bằng chứng dương của AC2.
    ///
    /// Kết nối được trả lại pool **kể cả khi closure trả `Err`**, qua `Drop` của `Lease`.
    ///
    /// ⚠️ Vế "kể cả khi panic" chỉ đúng nếu có unwind (`cargo test`/dev): bản release ghim
    /// `panic = "abort"` (xem Bẫy 6 của module), nên một panic trong `job` ở đây chấm dứt
    /// tiến trình ngay — `Lease::drop` không kịp chạy để trả kết nối về. Đừng viết `job`
    /// có đường panic.
    pub fn read<T, F>(&self, job: F) -> Result<T, StoreError>
    where
        F: FnOnce(ReadHandle<'_>) -> SqlResult<T>,
    {
        self.readers.read(job)
    }

    /// Đóng kho: dừng writer → đóng pool → **TRUNCATE có trần thời gian** → dừng luồng nền.
    ///
    /// Idempotent. Gọi lại là một no-op.
    ///
    /// ⚠️ Trần thời gian không phải phép lịch sự — xem [`Tuning::close_truncate_budget`].
    /// Hết trần thì ghi chẩn đoán rồi thoát; ⛔ **không treo tiến trình**.
    pub fn close(&self) {
        // Thứ tự này là hợp đồng. ⚠️ `readers.close()` KHÔNG tự chờ một `Lease` đang mượn
        // dở — nó chỉ thả các kết nối RẢNH và đánh thức người đang chờ. Việc chờ thật sự
        // diễn ra bên trong `wal_checkpoint(TRUNCATE)` ngay dưới đây, nơi SQLite tự
        // busy-wait (có trần `busy_timeout`, khác `close_truncate_budget`). Writer và pool
        // phải đóng trước để không còn kết nối RẢNH nào giữ WAL khi TRUNCATE chạy; đảo lại
        // là tự dựng ra đúng ca mà cái trần đó tồn tại để cứu.
        self.writer.shutdown();
        self.readers.close();
        self.checkpoint.shutdown();
    }

    /// Loại kho.
    pub const fn kind(&self) -> StoreKind {
        self.kind
    }

    /// Đường dẫn tệp `.db`.
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Phiên bản lược đồ **sau** khi di trú xong.
    pub const fn schema_version(&self) -> u32 {
        self.schema_version
    }

    /// Số đếm của luồng checkpoint.
    ///
    /// Không phải "test hook": AC4 đòi kết quả `(busy, log, checkpointed)` được **đọc và
    /// xét**, không vứt đi — và một số đọc mà không ai đọc được thì bằng vứt đi. Đây là
    /// bề mặt chẩn đoán, và nó cũng là thứ AC4/AC5 nghiệm thu trên.
    pub fn checkpoint_stats(&self) -> CheckpointStats {
        self.shared.stats()
    }

    /// Nhật ký chẩn đoán gần đây (vòng, có trần) — `busy != 0`, TRUNCATE hết trần, …
    ///
    /// ⚠️ Đây là nơi *"`busy != 0` ⇒ ghi chẩn đoán, ⛔ không coi là đã xong"* của AC4
    /// thật sự đọng lại. Không có nó thì mệnh đề đó chỉ là một comment.
    pub fn diagnostics(&self) -> Vec<String> {
        self.shared.diagnostics()
    }
}

impl Drop for Store {
    fn drop(&mut self) {
        self.close();
    }
}

impl std::fmt::Debug for Store {
    /// ⛔ Không in `Connection` — nó không `Debug` và cũng không có gì đọc được.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Store")
            .field("kind", &self.kind.as_str())
            .field("path", &self.path)
            .field("schema_version", &self.schema_version)
            .finish_non_exhaustive()
    }
}

/// Đường dẫn tệp `-wal` cạnh một tệp `.db`.
///
/// ⚠️ `Path::with_extension` là SAI ở đây: `global.db` → `global.wal`, trong khi SQLite
/// dùng `global.db-wal`. Nối vào `OsString` thô là cách duy nhất đúng trên cả hai nền
/// tảng, và nó không giả định đường dẫn là UTF-8 hợp lệ (Windows).
pub(crate) fn wal_path(db: &Path) -> PathBuf {
    let mut raw = db.as_os_str().to_owned();
    raw.push("-wal");
    PathBuf::from(raw)
}

/// Cỡ tệp `-wal` tính bằng byte; `Ok(0)` nếu tệp chưa tồn tại.
///
/// ⚠️ Chỉ `NotFound` được coi là "0 byte". Mọi lỗi I/O khác (quyền truy cập, khoá tệp
/// tạm thời…) được trả ra cho chỗ gọi tự chẩn đoán, ⛔ không bị nuốt thành cùng một giá
/// trị với "tệp chưa tồn tại" — nuốt im lặng ở đây nghĩa là ngưỡng của AC5 có thể không
/// bao giờ kích hoạt trong đúng phiên gõ liên tục hàng giờ mà nó tồn tại để canh.
pub(crate) fn wal_len(db: &Path) -> std::io::Result<u64> {
    match std::fs::metadata(wal_path(db)) {
        Ok(m) => Ok(m.len()),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(0),
        Err(e) => Err(e),
    }
}
