//! `meta.json` — metadata Library đọc được **không cần mở SQLite** (AD-9, AD-33, AC2/AC3).
//!
//! ─────────────────────────────────────────────────────────────────────────────
//! 🔴 GHI NGUYÊN TỬ — QUYẾT ĐỊNH #3 CỦA STORY 1.15
//! ─────────────────────────────────────────────────────────────────────────────
//! `write(<tmp>)` → `sync_all()` → `rename(<tmp>, meta.json)`. **Không tài liệu nào
//! trong PRD/ARCHITECTURE-SPINE yêu cầu điều này** — xem story `1-15…md` §Khoảng trống
//! atomic write. Không có bước này, một lần sập máy giữa lúc ghi để lại `meta.json` cắt
//! cụt, và AC3 (*"đọc được metadata mà không mở SQLite"*) trượt ngay lần đọc kế tiếp.
//!
//! ⚠️ Hàm ghi này **không bao giờ** được gọi bên trong closure của `Store::write` — xem
//! Quyết định #3: `meta.json` được ghi **NGAY SAU KHI** giao dịch `project.db` đã commit,
//! ở tầng THAO TÁC (chỗ gọi ở `commands::project`), không ở tầng giao dịch SQL.
//!
//! ─────────────────────────────────────────────────────────────────────────────
//! 🔴 "DẪN XUẤT" (AD-33) KHÔNG CÓ NGHĨA NẾU KHÔNG CÓ ĐƯỜNG DỰNG LẠI
//! ─────────────────────────────────────────────────────────────────────────────
//! [`WorkMeta::rebuild_from_store`] là bằng chứng của mệnh đề đó — không có nó, "dẫn xuất"
//! chỉ là một câu trong doc-comment mà không ai kiểm chứng được.

use std::io::Write as _;
use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::core::store::{ReadHandle, Store, StoreError};

/// Tên tệp cố định trong một `.atproj/`.
pub const META_FILE: &str = "meta.json";

/// Phiên bản lược đồ **của chính `meta.json`**, độc lập với `PRAGMA user_version` của
/// `project.db` — hai tệp, hai số phiên bản, đúng AC7.
pub const META_SCHEMA_VERSION: u32 = 1;

/// Mọi cách đọc/ghi `meta.json` hỏng.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MetaError {
    /// Đọc hoặc ghi tệp trượt ở tầng I/O / JSON.
    Io {
        /// Đường dẫn `meta.json` liên quan.
        path: String,
        /// Lỗi thô, chỉ để chẩn đoán.
        detail: String,
    },
    /// `meta_schema_version` đọc được **mới hơn** bản ứng dụng hiểu. Không ghi vào.
    SchemaTooNew {
        /// Phiên bản đọc được.
        found: u32,
        /// Phiên bản cao nhất bản ứng dụng này hiểu.
        supported: u32,
    },
}

impl std::fmt::Display for MetaError {
    /// ⚠️ KHÔNG DẤU — chẩn đoán cho log, không phải văn bản hiển thị (NFR16).
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MetaError::Io { path, detail } => write!(f, "meta[{path}] io failed: {detail}"),
            MetaError::SchemaTooNew { found, supported } => write!(
                f,
                "meta schema version {found} is newer than supported {supported}"
            ),
        }
    }
}

impl std::error::Error for MetaError {}

/// Hình dạng `meta.json` — Story 1.15, AC2/AC3/AC7.
///
/// ⚠️ `#[serde(rename_all = ...)]` KHÔNG được đặt — cùng luật với mọi struct qua biên của
/// dự án (xem `commands/config.rs::BootstrapConfig`). Khoá trên đĩa là `snake_case`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorkMeta {
    /// Số phiên bản lược đồ **của tệp này**. Xem [`META_SCHEMA_VERSION`].
    pub meta_schema_version: u32,
    /// UUID v4 (AD-28) — khớp `work.work_id` trong `project.db`.
    pub work_id: String,
    /// Tên Tác phẩm.
    pub name: String,
    /// Ngôn ngữ nguồn — **bất biến** (AD-18, FR3), đặt lúc tạo.
    pub source_lang: String,
    /// Thể loại.
    pub genre: String,
    /// ISO-8601 UTC (Consistency Conventions).
    pub created_at: String,
    /// ISO-8601 UTC.
    pub updated_at: String,
    /// Số Chương — **cache** của FR7, AD-33 nêu đích danh đây là thứ `meta.json` cache lại
    /// để Library đọc tiến độ mà không mở SQLite.
    pub chapter_count: u32,
}

impl WorkMeta {
    /// Đường dẫn `meta.json` bên trong một thư mục `.atproj/`.
    fn path_in(dir: &Path) -> std::path::PathBuf {
        dir.join(META_FILE)
    }

    /// Đọc `meta.json` — **không chạm `project.db`** (AC3).
    ///
    /// # Lỗi
    /// [`MetaError::SchemaTooNew`] nếu `meta_schema_version` vượt bản ứng dụng hiểu — không
    /// **không** ghi gì, kể cả khi chỗ gọi định làm vậy sau đó.
    pub fn read(dir: &Path) -> Result<WorkMeta, MetaError> {
        let path = Self::path_in(dir);
        let raw = std::fs::read_to_string(&path).map_err(|e| MetaError::Io {
            path: path.display().to_string(),
            detail: e.to_string(),
        })?;

        let meta: WorkMeta = serde_json::from_str(&raw).map_err(|e| MetaError::Io {
            path: path.display().to_string(),
            detail: e.to_string(),
        })?;

        if meta.meta_schema_version > META_SCHEMA_VERSION {
            return Err(MetaError::SchemaTooNew {
                found: meta.meta_schema_version,
                supported: META_SCHEMA_VERSION,
            });
        }

        Ok(meta)
    }

    /// Ghi `meta.json` **nguyên tử** — `write(<tmp>)` → `sync_all()` → `rename`.
    ///
    /// 🔴 **Không bao giờ** gọi hàm này bên trong closure của `Store::write` — xem
    /// doc-comment của module.
    pub fn write_atomic(&self, dir: &Path) -> Result<(), MetaError> {
        let target = Self::path_in(dir);
        let mut tmp = target.clone();
        tmp.set_extension("json.tmp");

        let json = serde_json::to_string_pretty(self).map_err(|e| MetaError::Io {
            path: target.display().to_string(),
            detail: e.to_string(),
        })?;

        let write_result = (|| -> std::io::Result<()> {
            let mut file = std::fs::File::create(&tmp)?;
            file.write_all(json.as_bytes())?;
            file.sync_all()?;
            Ok(())
        })();

        if let Err(e) = write_result {
            let _ = std::fs::remove_file(&tmp);
            return Err(MetaError::Io {
                path: tmp.display().to_string(),
                detail: e.to_string(),
            });
        }

        if let Err(e) = std::fs::rename(&tmp, &target) {
            // ⚠️ Dọn tệp tạm ở CẢ nhánh này — không có nó, một `rename` trượt để lại
            // `meta.json.tmp` nằm cạnh một `meta.json` vắng mặt, và lần sau lại thêm một
            // cái nữa. Đường quét của Epic 5 sẽ gặp rác này.
            let _ = std::fs::remove_file(&tmp);
            return Err(MetaError::Io {
                path: target.display().to_string(),
                detail: e.to_string(),
            });
        }

        // 🔴 fsync THƯ MỤC CHA — không phải thừa, và không phải cùng thứ với
        // `sync_all()` ở trên. `file.sync_all()` làm bền **nội dung** tệp tạm;
        // `rename` sửa **thư mục**, và mục thư mục đó nằm trong cache của hệ tệp cho tới
        // khi chính thư mục được fsync. Thiếu bước này, một lần mất điện ngay sau
        // `rename` có thể để lại thư mục không có `meta.json` nào — đúng thứ
        // doc-comment ở đầu module tuyên bố là đã chặn.
        //
        // ⚠️ Im lặng khi trượt có chủ ý: `File::open` một thư mục không hợp lệ trên
        // Windows (trả `Err`), và ở đó tính bền của `rename` do hệ tệp lo. Ghi được thì
        // ghi, không biến một khác biệt nền tảng thành một lỗi cho người dùng.
        if let Ok(dir_handle) = std::fs::File::open(dir) {
            let _ = dir_handle.sync_all();
        }

        Ok(())
    }

    /// Dựng lại `meta.json` từ **`project.db`** — bằng chứng của mệnh đề "dẫn xuất" (AD-33).
    ///
    /// Đọc hàng `work` (đúng một, `CHECK (id = 1)`) và đếm `chapter`, qua [`Store::read`]
    /// — không giao dịch ghi nào chạy ở đây.
    pub fn rebuild_from_store(store: &Store) -> Result<WorkMeta, StoreError> {
        store.read(|conn: ReadHandle<'_>| {
            let (work_id, name, source_lang, genre, created_at, updated_at): (
                String,
                String,
                String,
                String,
                String,
                String,
            ) = conn.query_row(
                "SELECT work_id, name, source_lang, genre, created_at, updated_at \
                 FROM work WHERE id = 1",
                [],
                |row| {
                    Ok((
                        row.get(0)?,
                        row.get(1)?,
                        row.get(2)?,
                        row.get(3)?,
                        row.get(4)?,
                        row.get(5)?,
                    ))
                },
            )?;

            let chapter_count: u32 =
                conn.query_row("SELECT COUNT(*) FROM chapter", [], |row| row.get(0))?;

            Ok(WorkMeta {
                meta_schema_version: META_SCHEMA_VERSION,
                work_id,
                name,
                source_lang,
                genre,
                created_at,
                updated_at,
                chapter_count,
            })
        })
    }
}
