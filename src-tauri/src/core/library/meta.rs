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
///
/// 🔵 **NÂNG 1 → 2 (2026-08-27, Story 5.4)** — hai trường `status`/`status_is_override`
/// thêm vào [`WorkMeta`]. `WorkMeta::read` chỉ từ chối bản **mới hơn** (xem doc-comment của
/// hàm đó), nên mọi `meta.json` v1 viết TRƯỚC story này vẫn đọc được sau lượt nâng: cả hai
/// trường mới đọc ra giá trị mặc định của kiểu (`None`/`false`) qua `#[serde(default)]` —
/// đúng CHỦ Ý, không phải một khoảng trống. Xem §Design Notes "Vì sao `Option<String>`" của
/// `5-4-bon-trang-thai-vong-doi.md`.
///
/// 🔵 **NÂNG 2 → 3 (2026-08-28, Story 5.5)** — trường `chapter_done_count` thêm vào
/// [`WorkMeta`], cùng lý lẽ `Option` đã áp cho `status` ở lượt nâng trên: một `meta.json`
/// v1/v2 (viết TRƯỚC story này) không mang khoá này, đọc ra `None` qua `#[serde(default)]` —
/// `None` nói *CHƯA BIẾT*, không phải `Some(0)`. Xem doc-comment của trường và §Design Notes
/// "Vì sao `Option<u32>` chứ không `u32`" của `5-5-tien-do-tac-pham.md`.
pub const META_SCHEMA_VERSION: u32 = 3;

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
///
/// 🔵 **NÂNG (2026-08-28, Story 5.5)** — trường [`Self::chapter_done_count`] thêm vào, xem
/// doc-comment của trường đó và [`META_SCHEMA_VERSION`].
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
    /// 🔵 **THÊM (2026-08-27, Story 5.4)** — trạng thái vòng đời hiển thị của Tác phẩm, một
    /// trong bốn giá trị trên dây của [`crate::core::lifecycle::LifecycleStatus`], hoặc
    /// `None`.
    ///
    /// `#[serde(default)]` — **KHÔNG** `#[serde(default = "...")]` về một giá trị mặc định
    /// cụ thể: một `meta.json` v1 (viết trước story này) không mang khoá này, và đọc nó ra
    /// `None` phải nói *CHƯA BIẾT*, không được lặng lẽ trở thành *"Chưa bắt đầu"*. Một Tác
    /// phẩm đã dịch xong mà hiện "Chưa bắt đầu" trong Library, không một lỗi nào được ném,
    /// là đúng lớp *"rỗng im lặng"* mà `AGENTS.md::Known pitfalls` gọi tên. [`Self::rebuild_from_store`]
    /// LUÔN đặt trường này thành `Some(..)` — `None` chỉ tồn tại trên những `meta.json` mà
    /// đường dựng lại chưa từng chạm tới.
    #[serde(default)]
    pub status: Option<String>,
    /// 🔵 **THÊM (2026-08-27, Story 5.4)** — `true` ⇔ [`Self::status`] đến từ
    /// `work.status_override` (ghi đè thủ công), `false` ⇔ giá trị suy ra tự động (hoặc
    /// `status` là `None`, tức chưa biết — cờ này không mang nghĩa gì trong ca đó).
    ///
    /// `#[serde(default)]` — `meta.json` v1 không mang khoá này ⇒ `false`, đúng nghĩa "không
    /// biết có ghi đè hay không" cho một Tác phẩm chưa từng qua đường dựng lại của story này.
    #[serde(default)]
    pub status_is_override: bool,
    /// 🔵 **THÊM (2026-08-28, Story 5.5)** — số Chương ở `chapter.status = 'done'`, FR7. Kiểu
    /// `Option<u32>`, KHÔNG `u32`: `0` là một giá trị **hợp lệ và thường gặp** (Tác phẩm chưa
    /// dịch Chương nào), nên một `#[serde(default)]` về `0` làm một `meta.json` v1/v2 chưa
    /// từng qua [`Self::rebuild_from_store`] của story này **không phân biệt được** với một
    /// Tác phẩm thật sự chưa xong Chương nào — đúng lớp *"rỗng im lặng"* mà
    /// `AGENTS.md::Known pitfalls` gọi tên. `None` nói *CHƯA BIẾT*; [`Self::rebuild_from_store`]
    /// LUÔN đặt trường này thành `Some(..)`, kể cả khi `chapter_count = 0` (⇒ `Some(0)`).
    ///
    /// ⚠️ Ghi đè thủ công trạng thái Tác phẩm (`status_is_override = true`) KHÔNG BAO GIỜ đổi
    /// trường này — tiến độ đếm từ `chapter.status` thật, độc lập với `work.status_override`
    /// (§Never của story: "ghi đè thủ công KHÔNG BAO GIỜ đổi tiến độ").
    #[serde(default)]
    pub chapter_done_count: Option<u32>,
}

impl WorkMeta {
    /// Đường dẫn `meta.json` bên trong một thư mục `.atproj/`.
    ///
    /// 🔵 **`pub(crate)` (2026-08-28, Story 5.5)** — trước đó `private`.
    ///
    /// 🔵 **SỬA (2026-08-28, vòng rà thứ hai) — vế `tests/**` của câu trước SAI, không chỉ
    /// diễn đạt kém.** `src-tauri/tests/**` biên dịch thành các crate TEST RIÊNG (mỗi tệp một
    /// crate nhị phân, liên kết vào `auratranslate_lib` như một phụ thuộc ngoài) — `pub(crate)`
    /// KHÔNG thấy được từ đó dù có nâng tầm nhìn hay không; nâng lên hẳn `pub` mới đổi được gì
    /// cho `tests/**`. Người nâng tầm nhìn đúng lý do CHỈ có một chỗ gọi thật:
    /// `commands/project.rs:1123`, một `#[cfg(test)] mod tests` sống TRONG CÙNG crate `lib`
    /// (biên dịch cùng `auratranslate_lib`, không phải một crate test riêng) — `pub(crate)` là
    /// đủ và đúng tầm cho đúng một chỗ gọi đó, thay vì tự lắp chuỗi `"meta.json"`/nhắc
    /// `META_FILE` mà `meta_write_boundary.rs` khoá CHỈ ở module này (AC4). KHÔNG `pub`: đây
    /// vẫn là chi tiết nội bộ, không phải một phần bề mặt IPC. Hai chỗ gọi sản phẩm còn lại
    /// (`Self::path_in` ở dòng 162/188 ngay dưới) đã ở TRONG `impl WorkMeta`, không cần tầm
    /// nhìn `pub(crate)` để thấy nhau.
    pub(crate) fn path_in(dir: &Path) -> std::path::PathBuf {
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
    /// Đọc hàng `work` (đúng một, `CHECK (id = 1)`, cộng `status_override`) và cả đếm lẫn
    /// đọc trạng thái từng `chapter`, qua [`Store::read`] — không giao dịch ghi nào chạy ở
    /// đây.
    ///
    /// 🔵 **THÊM (2026-08-27, Story 5.4)** — `status`/`status_is_override` tính TẠI ĐÂY, chỗ
    /// DUY NHẤT tính giá trị suy ra (§Approach của story): `work.status_override IS NOT
    /// NULL` ⇒ giữ nguyên giá trị đó, `is_override = true`; ngược lại gọi
    /// [`crate::core::lifecycle::derive_work_status`] trên tập trạng thái của MỌI Chương,
    /// `is_override = false`. Trường này LUÔN `Some(..)` sau một lượt dựng lại — `None` chỉ
    /// còn tồn tại trên những `meta.json` cũ mà đường này chưa từng chạm tới.
    pub fn rebuild_from_store(store: &Store) -> Result<WorkMeta, StoreError> {
        store.read(|conn: ReadHandle<'_>| {
            let (work_id, name, source_lang, genre, created_at, updated_at, status_override): (
                String,
                String,
                String,
                String,
                String,
                String,
                Option<String>,
            ) = conn.query_row(
                "SELECT work_id, name, source_lang, genre, created_at, updated_at, \
                 status_override FROM work WHERE id = 1",
                [],
                |row| {
                    Ok((
                        row.get(0)?,
                        row.get(1)?,
                        row.get(2)?,
                        row.get(3)?,
                        row.get(4)?,
                        row.get(5)?,
                        row.get(6)?,
                    ))
                },
            )?;

            let chapter_count: u32 =
                conn.query_row("SELECT COUNT(*) FROM chapter", [], |row| row.get(0))?;

            // §Never của story: "không tự suy trạng thái Chương từ trạng thái segment" -- đọc
            // thẳng `chapter.status`, không đi vòng qua `segment`.
            let mut stmt = conn.prepare("SELECT status FROM chapter")?;
            let chapter_status_rows: Vec<String> = stmt
                .query_map([], |row| row.get::<_, String>(0))?
                .collect::<crate::core::store::SqlResult<Vec<_>>>()?;
            drop(stmt);

            // 🔵 NÂNG RA NGOÀI `match status_override` (2026-08-28, Story 5.5, §Always +
            // §Design Notes "Cái bẫy ở `match status_override`"). TRƯỚC lượt này, vòng phân
            // giải nằm TRONG nhánh `None` -- nhánh `Some(raw)` (ghi đè thủ công) không bao giờ
            // chạm `chapter_status_rows`. Đặt phép đếm tiến độ bên trong nhánh đó sẽ làm MỌI
            // Tác phẩm có ghi đè thủ công mất tiến độ (biên dịch sạch, qua mọi cổng CŨ). Nâng
            // ra ngoài để cả hai nhánh dùng CHUNG một tập `chapters` đã phân giải -- nhánh nào
            // xử lý `status`/`status_is_override` không còn liên quan tới việc đếm.
            //
            // Mọi giá trị ghi qua `commands::lifecycle`/`create_work` đã được `LifecycleStatus`
            // cưỡng chế ở tầng Rust trước khi chạm SQL (§Always) -- vòng lọc dưới đây chỉ là
            // lớp phòng thủ cho một hàng cũ/hỏng, không phải đường vào bình thường.
            //
            // 🔴 NHƯNG BỎ QUA IM LẶNG THÌ CẤM (lượt rà 2026-08-28). Nếu MỌI hàng `chapter` đều
            // hỏng, `chapters` rỗng và `derive_work_status(&[])` trả `NotStarted` -- tức một
            // Tác phẩm CÓ Chương bị khai là "Chua bat dau", và hỏng dữ liệu đội lốt một trạng
            // thái hợp lệ. Đúng lớp lỗi trung tâm mà `AGENTS.md::Known pitfalls` gọi tên. Không
            // đường nào ở đây trả lỗi ra người dùng được (`meta.json` là cache dẫn xuất, và một
            // Tác phẩm không mở được vì MỘT hàng hỏng thì tệ hơn), nên nó phải để lại VẾT ở
            // chẩn đoán.
            let mut chapters: Vec<crate::core::lifecycle::LifecycleStatus> = Vec::new();
            for raw in &chapter_status_rows {
                match crate::core::lifecycle::LifecycleStatus::from_wire(raw) {
                    Some(parsed) => chapters.push(parsed),
                    None => eprintln!(
                        "meta[{work_id}] chapter.status khong nam trong danh muc bon gia tri: {raw:?}                                  -- hang nay bi bo qua khi suy ra trang thai Tac pham VA khi dem tien do"
                    ),
                }
            }
            if chapters.is_empty() && !chapter_status_rows.is_empty() {
                eprintln!(
                    "meta[{work_id}] KHONG hang chapter nao doc duoc ({} hang tren dia)                              -- trang thai suy ra duoi day la not_started vi THIEU du lieu,                              khong phai vi Tac pham chua bat dau",
                    chapter_status_rows.len()
                );
            }

            // Tiến độ (FR7) -- đếm trên tập ĐÃ PHÂN GIẢI, độc lập hoàn toàn với
            // `status_override`: một Chương hỏng không được tính là đã xong (§Always), và một
            // Tác phẩm ghi đè thủ công vẫn hiện đúng số Chương đã xong thật (§Never).
            let chapter_done_count = chapters
                .iter()
                .filter(|s| **s == crate::core::lifecycle::LifecycleStatus::Done)
                .count() as u32;

            let (status, status_is_override) = match status_override {
                Some(raw) => (Some(raw), true),
                None => {
                    let derived = crate::core::lifecycle::derive_work_status(&chapters);
                    (Some(derived.as_str().to_owned()), false)
                }
            };

            Ok(WorkMeta {
                meta_schema_version: META_SCHEMA_VERSION,
                work_id,
                name,
                source_lang,
                genre,
                created_at,
                updated_at,
                chapter_count,
                status,
                status_is_override,
                chapter_done_count: Some(chapter_done_count),
            })
        })
    }
}
