//! Chỉ mục Library dẫn xuất — `library-index.db`, một đường ghi duy nhất (AD-8) — Story 5.2.
//!
//! ─────────────────────────────────────────────────────────────────────────────
//! 🔴 CỔNG RANH GIỚI: ĐÂY LÀ MODULE DUY NHẤT MỞ KHO NÀY
//! ─────────────────────────────────────────────────────────────────────────────
//! `tests/library_index_boundary.rs` canh: không tệp `.rs` nào dưới `src-tauri/src/**` ngoài
//! module này (và điểm khai `core/store/mod.rs`) được nhắc `StoreSpec::library_index` hay
//! `StoreKind::LibraryIndex`. `lib.rs` chỉ gọi [`Indexer::open`] + `app.manage(...)` — không
//! tự dựng `StoreSpec` cho kho này.
//!
//! ─────────────────────────────────────────────────────────────────────────────
//! NĂM THAO TÁC — Story 5.3 thêm hai (GỠ mồ côi tường minh, LIỆT KÊ mồ côi)
//! ─────────────────────────────────────────────────────────────────────────────
//! - [`Indexer::open`] — mở (hoặc dựng mới) `library-index.db`, hiện thực nhánh KHÔNG-DI-TRÚ
//!   của AD-8: lệch phiên bản lược đồ (cả hai chiều) ⇒ xoá tệp + sidecar rồi dựng lại, KHÔNG
//!   đi qua nhánh từ chối mở mà `project.db`/`global.db` dựa vào (AD-30).
//! - [`Indexer::rebuild`] — quét thư mục gốc Library, đọc **chỉ** `meta.json` của mỗi
//!   `.atproj` (AD-9: không mở `project.db` lần nào), rồi **ĐỐI CHIẾU** kết quả với
//!   `library_work` trong **một** giao dịch qua `store::Writer`. 🔵 **ĐỔI NGỮ NGHĨA (Story
//!   5.3):** trước đây hàm này `DELETE FROM library_work` rồi `INSERT` lại toàn bộ — một
//!   `.atproj` bị xoá/di chuyển ra ngoài gốc biến mất khỏi chỉ mục IM LẶNG. Nay nó UPSERT mọi
//!   mục đọc được (`orphaned = 0`) rồi đánh dấu `orphaned = 1` cho mọi hàng còn lại mà
//!   `atproj_path` KHÔNG nằm trong tập `.atproj` vừa liệt kê được — hàng ĐƯỢC GIỮ, không bị
//!   xoá (§Design Notes "vị từ mồ côi: ba cách viết, hai cách sai"). Toàn bộ scan+ghi chạy
//!   dưới [`Indexer::rebuild_lock`] — hai lượt `rebuild` gọi đồng thời phải NỐI TIẾP, không
//!   xen kẽ giai đoạn quét với giai đoạn ghi (deferred-work.md:8079, chủ Story 5.3).
//!   Đây vẫn là đường ghi DUY NHẤT của module này — không có một đường "chèn một hàng" thứ
//!   hai chạy song song với nó, kể cả khi chỉ một Tác phẩm vừa được tạo (xem
//!   `commands::project::wire::create_work_from_text`, nơi gọi lại đúng hàm này).
//! - [`Indexer::forget_orphan`] — **THÊM Story 5.3.** Xoá đúng MỘT hàng mồ côi khỏi chỉ mục —
//!   đường XOÁ tường minh, có tiền điều kiện `orphaned = 1`, không phải một đường ghi thứ hai
//!   (§Design Notes "vì sao forget_orphan không phải đường ghi thứ hai").
//! - [`Indexer::list_works`] — đường ĐỌC mọi hàng ĐANG SỐNG (`orphaned = 0`), dùng cho Story
//!   5.6/5.9.
//! - [`Indexer::list_orphans`] — **THÊM Story 5.3.** Đường ĐỌC mọi hàng mồ côi
//!   (`orphaned = 1`), dùng cho màn hình tối thiểu của story này.
//!
//! ─────────────────────────────────────────────────────────────────────────────
//! VÌ SAO "một đường ghi duy nhất" LÀ MỘT QUYẾT ĐỊNH, KHÔNG PHẢI SỰ LƯỜI BIẾNG
//! ─────────────────────────────────────────────────────────────────────────────
//! Một thiết kế "nhanh hơn" sẽ thêm một hàm `index_one(work_id, meta)` chèn đúng một hàng khi
//! `create_work` vừa xong, tách khỏi đường quét toàn bộ. Đó là HAI đường ghi cho cùng một
//! bảng — hai chỗ phải giữ đúng cùng logic phát hiện trùng `work_id`, và chúng SẼ trôi khỏi
//! nhau (đúng lớp lỗi mà `AGENTS.md::Known pitfalls` gọi tên: "hai dữ kiện nói cùng một
//! chuyện thì chúng lệch được"). `rebuild` quét lại TOÀN BỘ thư mục gốc mỗi lần — kể cả sau
//! một lượt tạo Tác phẩm — tốn hơn về CPU cho một thư viện lớn, nhưng đó là phép đánh đổi CÓ
//! CHỦ: NFR3–5 chưa nghiệm thu đủ điều kiện ở Epic 5 (chưa có đường tạo 5.000 Chương thật —
//! Story 6.18 mới đo được), còn "một đường ghi duy nhất" là bất biến AC2 của story này đòi
//! NGAY HÔM NAY. Nếu phép đo sau này buộc phải tách đường, đó là một quyết định kiến trúc mới
//! (AD), không phải một lượt tối ưu tiện tay.

use std::collections::{BTreeMap, HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::Mutex;

use crate::core::i18n::{IpcError, MessageKey};
use crate::core::store::{
    LIBRARY_INDEX_MIGRATIONS, ReadHandle, Store, StoreError, StoreKind, StoreSpec, Transaction,
};

use super::meta::WorkMeta;

/// Đuôi thư mục của một Tác phẩm — cùng hằng `WORK_FOLDER_SUFFIX` của [`super::atproj`], chép
/// lại vì hằng đó là `const` riêng tư của module kia và không có lý do lộ ra `pub(crate)` chỉ
/// cho một lần so sánh chuỗi ở đây.
const ATPROJ_EXTENSION: &str = "atproj";

/// Kho `library-index.db` đã mở. Sở hữu một [`Store`] — `Drop` của nó đóng kho (TRUNCATE có
/// trần), cùng khuôn `commands::project::OpenWork` (không có `Drop` thủ công ở đây,
/// `store: Store` tự lo qua field drop).
pub struct Indexer {
    store: Store,
    /// **THÊM Story 5.3.** Nối tiếp TOÀN BỘ một lượt [`Indexer::rebuild`] — cả giai đoạn
    /// QUÉT đĩa lẫn giai đoạn GHI, không chỉ giai đoạn ghi (`store::Writer` đã nối tiếp phần
    /// đó một mình). Hai lượt `rebuild` gọi gần như đồng thời (khởi động + người dùng bấm)
    /// phải hoàn tất TUẦN TỰ; nếu không, lượt A quét được tập `.atproj` CŨ rồi ghi SAU lượt B
    /// sẽ đánh dấu mồ côi một hàng mà lượt B vừa thêm — một ảnh chụp trộn. Xem §Design Notes
    /// "Vì sao một Mutex chứ không một AtomicU64 thế hệ" của story `5-3-quet-lai-thu-muc.md`.
    ///
    /// `()` — khoá không giữ dữ liệu nào, nó chỉ là một chốt chặn. `lock().unwrap_or_else(..)`
    /// theo lệ kho (`AGENTS.md`): một panic ở luồng khác giữ khoá không được lan sang đây.
    rebuild_lock: Mutex<()>,
}

impl Indexer {
    /// Mở (hoặc dựng mới) `library-index.db` tại `path` — nhánh KHÔNG-DI-TRÚ của AD-8.
    ///
    /// Thứ tự: so `PRAGMA user_version` hiện có với đích của [`LIBRARY_INDEX_MIGRATIONS`]
    /// (không đi qua [`Store::open`] để làm việc đó — xem
    /// `crate::core::store::peek_schema_version`); lệch (cả hai chiều) ⇒ xoá tệp + sidecar;
    /// rồi mới [`Store::open`] như bình thường — tại điểm đó tệp luôn hoặc chưa tồn tại hoặc
    /// đã ở đúng phiên bản đích, nên [`Store::open`] không bao giờ chạm nhánh từ chối mở của
    /// nó (bước 3, `StoreError::SchemaTooNew`).
    pub fn open(path: PathBuf) -> Result<Indexer, StoreError> {
        crate::core::store::delete_if_schema_version_differs(
            &path,
            StoreKind::LibraryIndex,
            LIBRARY_INDEX_MIGRATIONS,
        )?;

        let store = Store::open(StoreSpec::library_index(path))?;
        Ok(Indexer {
            store,
            rebuild_lock: Mutex::new(()),
        })
    }

    /// Đóng kho — cùng khuôn `close_global_store`/`close_open_work` ở `lib.rs`. Idempotent
    /// (uỷ quyền cho [`Store::close`], vốn đã idempotent).
    pub fn close(&self) {
        self.store.close();
    }

    /// Quét `root`, đọc `meta.json` của mỗi `<Tên>.atproj/`, và **ghi lại toàn bộ**
    /// `library_work` trong một giao dịch — đường ghi DUY NHẤT của module này.
    ///
    /// `root` không tồn tại ⇒ chỉ mục **rỗng có lý do** ([`RebuildOutcome::root_missing`]),
    /// không tạo thư mục, không lỗi (§I/O Matrix "Thư mục gốc vắng").
    ///
    /// # Lỗi
    /// [`IndexError::Io`] nếu `root` tồn tại nhưng không đọc được (quyền, đĩa hỏng) —
    /// **khác** một `.atproj` con bị hỏng, thứ đó đi vào [`RebuildOutcome::skipped`], không
    /// phải `Err` (§Boundaries: *"một `.atproj` thiếu/hỏng `meta.json` cũng phải phân biệt
    /// được với 'không có Tác phẩm nào', không rơi im lặng"*). [`IndexError::Store`] nếu lượt
    /// ghi trượt.
    pub fn rebuild(&self, root: &Path) -> Result<RebuildOutcome, IndexError> {
        // Khoá TOÀN BỘ scan+ghi ngay từ đây — xem doc-comment của `rebuild_lock`. Giữ khoá
        // xuyên suốt cả hàm (biến `_guard` sống tới cuối scope) là điểm mấu chốt: nó không
        // chỉ khoá lượt ghi (đã nối tiếp qua `store::Writer`), nó khoá cả lượt ĐỌC ĐĨA phía
        // trên, thứ mà hai lượt `rebuild` gọi gần nhau có thể chạy xen kẽ nếu không có nó.
        let _guard = self
            .rebuild_lock
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);

        if !root.exists() {
            return self.mark_all_orphaned_for_missing_root();
        }

        let scan = match scan_atproj_dirs(root)? {
            // 🔴 VÁ (vòng rà ba lớp, P6) — `root.exists()` ngay trên và `read_dir(root)` bên
            // trong `scan_atproj_dirs` là HAI bước tách rời: root biến mất GIỮA hai bước (ổ
            // ngoài rút ra đúng lúc) làm `read_dir` trả `NotFound`. Trước bản vá, `?` biến ca
            // đó thành `IndexError::Io` — một LỖI CỨNG cho đúng tình huống mà nhánh
            // `root_missing` êm ái ngay trên đã tồn tại để canh. `scan_atproj_dirs` tự ánh xạ
            // `NotFound` về [`ScanRootOutcome::RootMissing`]; xử lý y hệt fast-path phía trên.
            ScanRootOutcome::RootMissing => return self.mark_all_orphaned_for_missing_root(),
            ScanRootOutcome::Scanned(scan) => scan,
        };

        // work_id -> đường dẫn ĐẦU TIÊN đã nhận nó, theo đúng thứ tự quét (thứ tự đã SẮP —
        // xem `scan_atproj_dirs` — nên "mục đầu" là tất định, không phụ thuộc thứ tự hệ điều
        // hành trả `read_dir`, thứ **không** đảm bảo ổn định giữa hai lần gọi).
        let mut first_seen: HashMap<String, PathBuf> = HashMap::new();
        let mut kept: Vec<(PathBuf, WorkMeta)> = Vec::new();
        let mut conflicts: Vec<WorkIdConflict> = Vec::new();
        // 🔴 VÁ (vòng rà ba lớp, P5) — bắt đầu từ những entry ĐÃ lỗi khi liệt kê `root` (xem
        // `scan_atproj_dirs`/`partition_dir_entries`), không phải một `Vec` rỗng: một entry hỏng
        // giữa lượt liệt kê không còn huỷ cả lượt quét bằng `?` nữa, nên nó phải có mặt trong
        // kết quả cuối cùng như MỌI `.atproj` bị bỏ qua khác.
        let mut skipped: Vec<SkippedEntry> = scan.skipped;

        // 🔵 SỬA (2026-08-27, vòng rà bốn lớp P3) — vế HAI của vị từ mồ côi ĐÃ SAI, và mệnh đề
        // "vị từ mồ côi: ba cách viết, hai cách sai" của §Design Notes HẾT ĐÚNG: nó thiếu ca
        // thứ tư. Bản trước dựng tập chặn mồ côi từ TOÀN BỘ `scan.dirs` (mọi `.atproj` liệt
        // kê được, không cần đọc được `meta.json`) — sai ở kịch bản "đường dẫn bị Tác phẩm
        // KHÁC chiếm": A từng sống ở `/gốc/Foo.atproj`; người dùng xoá A rồi copy B vào một
        // thư mục CŨNG tên `Foo.atproj`. B đọc được ⇒ `Foo.atproj` vẫn nằm trong `scan.dirs`
        // ⇒ hàng của A (không đọc được ở lượt này) bị coi là "đường dẫn còn đó" nên KHÔNG bị
        // đánh dấu mồ côi — một hàng SỐNG nói dối, trỏ vào thư mục nay thuộc về B.
        //
        // ⇒ Câu đúng không phải *"đường dẫn này có được liệt kê không"* mà là *"đường dẫn
        // này có được liệt kê MÀ KHÔNG đọc được `meta.json` không"* — một đường dẫn ĐỌC ĐƯỢC
        // đã thuộc về ĐÚNG `work_id` nó vừa khai (nằm trong `first_seen`/`kept`), không phải
        // một tấm khiên chung cho MỌI `work_id` từng đứng ở đó. Tập `unreadable_paths` dưới
        // đây chỉ gom những `.atproj` CÒN NẰM ĐÓ nhưng `meta.json` KHÔNG đọc được — đúng và
        // chỉ đúng ca "hỏng nhưng còn" (§Design Notes ca ①) mới được nó che chắn.
        let mut unreadable_paths: HashSet<String> = HashSet::new();

        for dir in scan.dirs {
            match WorkMeta::read(&dir) {
                Ok(meta) => match first_seen.get(&meta.work_id) {
                    Some(kept_path) => conflicts.push(WorkIdConflict {
                        work_id: meta.work_id,
                        kept_path: kept_path.clone(),
                        duplicate_path: dir,
                    }),
                    None => {
                        first_seen.insert(meta.work_id.clone(), dir.clone());
                        kept.push((dir, meta));
                    }
                },
                // `MetaError` đã phân biệt I/O (thiếu tệp) với `SchemaTooNew` (mới hơn ứng
                // dụng hiểu) qua chính kiểu của nó — `Display` của cả hai đủ để chẩn đoán,
                // không cần một trường `kind` thứ hai ở đây (`SkippedEntry::reason` là chuỗi
                // chẩn đoán, KHÔNG DẤU, không phải văn bản hiển thị — cùng luật NFR16).
                Err(err) => {
                    unreadable_paths.insert(dir.display().to_string());
                    skipped.push(SkippedEntry {
                        path: dir,
                        reason: err.to_string(),
                    });
                }
            }
        }

        let indexed = kept.len();
        let orphans = self.store.write(move |tx: &Transaction<'_>| {
            // Vế MỘT của vị từ mồ côi — "work_id không đọc được ở lượt này" — thoả bằng
            // chính việc một hàng KHÔNG nằm trong `kept` (nó không được UPSERT ở đây).
            //
            // 🔴 UPSERT, không `DELETE` + `INSERT` — đây là chỗ đổi ngữ nghĩa CHÍNH của
            // story. `work_id` là khoá chính nên `ON CONFLICT` là cách SQLite tự phân biệt
            // "Tác phẩm này đã có trong chỉ mục" (cập nhật đường dẫn/metadata, gỡ cờ mồ côi)
            // với "Tác phẩm mới" (chèn hàng) — không cần một `SELECT` kiểm trùng ở tầng Rust.
            for (dir, meta) in &kept {
                tx.execute(
                    "INSERT INTO library_work \
                     (work_id, atproj_path, name, source_lang, genre, created_at, \
                      updated_at, chapter_count, orphaned) \
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, 0) \
                     ON CONFLICT (work_id) DO UPDATE SET \
                       atproj_path   = excluded.atproj_path, \
                       name          = excluded.name, \
                       source_lang   = excluded.source_lang, \
                       genre         = excluded.genre, \
                       created_at    = excluded.created_at, \
                       updated_at    = excluded.updated_at, \
                       chapter_count = excluded.chapter_count, \
                       orphaned      = 0",
                    (
                        &meta.work_id,
                        &dir.display().to_string(),
                        &meta.name,
                        &meta.source_lang,
                        &meta.genre,
                        &meta.created_at,
                        &meta.updated_at,
                        meta.chapter_count,
                    ),
                )?;
            }

            // Mọi hàng CÒN LẠI (không vừa UPSERT ở trên): mồ côi khi và chỉ khi `atproj_path`
            // của nó KHÔNG nằm trong `unreadable_paths` — vế HAI của vị từ (P3). Đọc lại toàn
            // bảng trong CÙNG giao dịch (không phải một `Store::read` riêng) để không có cửa
            // sổ đua giữa "đọc trạng thái cũ" và "ghi trạng thái mới".
            let mut stmt = tx.prepare("SELECT work_id, atproj_path FROM library_work")?;
            let existing: Vec<(String, String)> = stmt
                .query_map([], |row| Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?)))?
                .collect::<crate::core::store::SqlResult<Vec<_>>>()?;
            drop(stmt);

            let mut newly_orphaned = 0usize;
            for (work_id, atproj_path) in existing {
                if first_seen.contains_key(&work_id) {
                    continue; // Vừa upsert ở trên -- `orphaned` đã là 0.
                }
                if unreadable_paths.contains(&atproj_path) {
                    continue; // Thư mục còn ĐÓ nhưng meta.json hỏng (ca "hỏng nhưng còn" -- KHÔNG mồ côi).
                }
                let changed = tx.execute(
                    "UPDATE library_work SET orphaned = 1 WHERE work_id = ?1 AND orphaned = 0",
                    [&work_id],
                )?;
                newly_orphaned += changed;
            }

            Ok(newly_orphaned)
        })?;

        Ok(RebuildOutcome {
            indexed,
            root_missing: false,
            conflicts,
            skipped,
            orphans,
        })
    }

    /// Đánh dấu MỌI hàng đang sống thành mồ côi (`orphaned = 1`) và trả một
    /// [`RebuildOutcome`] rỗng-có-lý-do (`root_missing: true`) — dùng CHUNG bởi cả fast-path
    /// (`!root.exists()`) lẫn nhánh đua P6 (`root` biến mất giữa `exists()` và `read_dir`).
    ///
    /// 🔵 **ĐỔI NGỮ NGHĨA (Story 5.3) — trước đây `clear_for_missing_root` XOÁ SẠCH bảng.**
    /// Gốc vắng mặt nghĩa là **tập `.atproj` liệt kê được là rỗng** — đúng vị từ mồ côi ở
    /// [`Indexer::rebuild`] áp cho MỌI hàng đang sống, không phải một nhánh riêng "xoá sạch".
    /// Chỉ mục nói về THƯ VIỆN (thư mục gốc đang cấu hình), không nói về đĩa nói chung: các
    /// `.atproj` của một gốc CŨ vẫn có thể còn nguyên trên đĩa (ca "đổi thư mục gốc"), nhưng
    /// chúng không còn nằm trong thư viện đang quét.
    fn mark_all_orphaned_for_missing_root(&self) -> Result<RebuildOutcome, IndexError> {
        let orphans = self.store.write(|tx: &Transaction<'_>| {
            tx.execute("UPDATE library_work SET orphaned = 1 WHERE orphaned = 0", [])
        })?;
        Ok(RebuildOutcome {
            indexed: 0,
            root_missing: true,
            conflicts: Vec::new(),
            skipped: Vec::new(),
            orphans,
        })
    }

    /// Đường ĐỌC — mọi hàng ĐANG SỐNG (`orphaned = 0`) của `library_work`, sắp theo `work_id`
    /// (tất định; sắp theo tên/ngày là việc của Story 5.6, không phải của story này).
    pub fn list_works(&self) -> Result<Vec<IndexedWork>, StoreError> {
        self.list_rows("WHERE orphaned = 0")
    }

    /// **THÊM Story 5.3.** Đường ĐỌC — mọi hàng MỒ CÔI (`orphaned = 1`), cho màn hình tối
    /// thiểu của story này. Không lọc/sắp theo tiêu chí nào khác `work_id` — 5.6 sở hữu phần
    /// đó.
    pub fn list_orphans(&self) -> Result<Vec<IndexedWork>, StoreError> {
        self.list_rows("WHERE orphaned = 1")
    }

    fn list_rows(&self, predicate: &'static str) -> Result<Vec<IndexedWork>, StoreError> {
        self.store.read(move |conn: ReadHandle<'_>| {
            let mut stmt = conn.prepare(&format!(
                "SELECT work_id, atproj_path, name, source_lang, genre, created_at, \
                 updated_at, chapter_count, orphaned \
                 FROM library_work {predicate} ORDER BY work_id"
            ))?;
            let rows = stmt.query_map([], |row| {
                Ok(IndexedWork {
                    work_id: row.get(0)?,
                    atproj_path: PathBuf::from(row.get::<_, String>(1)?),
                    name: row.get(2)?,
                    source_lang: row.get(3)?,
                    genre: row.get(4)?,
                    created_at: row.get(5)?,
                    updated_at: row.get(6)?,
                    chapter_count: row.get(7)?,
                    orphaned: row.get::<_, i64>(8)? != 0,
                })
            })?;
            rows.collect()
        })
    }

    /// **THÊM Story 5.3.** Xoá đúng MỘT hàng mồ côi khỏi chỉ mục — đường XOÁ tường minh, có
    /// tiền điều kiện `orphaned = 1`. KHÔNG chạm đĩa một byte (§Never của story: "không tự
    /// sửa/di chuyển/xoá bất kỳ `.atproj` nào — `forget_orphan` xoá một hàng chỉ mục").
    ///
    /// # Lỗi
    /// [`IndexError::NotOrphaned`] khi `work_id` không tồn tại HOẶC tồn tại nhưng
    /// `orphaned = 0` — CÙNG một nhánh từ chối cho cả hai ca (§I/O Matrix: "gỡ nhầm một hàng
    /// đang sống" và "gỡ một `work_id` không có" đều phải từ chối, không im lặng thành công
    /// và không mập mờ giữa hai lý do). `WHERE work_id = ?1 AND orphaned = 1` trong MỘT câu
    /// `DELETE` là cách SQLite tự trả về đúng phân biệt đó qua số hàng bị đổi — không cần một
    /// `SELECT` kiểm trước rồi `DELETE` sau (cửa sổ đua giữa hai câu).
    pub fn forget_orphan(&self, work_id: &str) -> Result<(), IndexError> {
        let owned = work_id.to_owned();
        let deleted = self.store.write(move |tx: &Transaction<'_>| {
            tx.execute(
                "DELETE FROM library_work WHERE work_id = ?1 AND orphaned = 1",
                [&owned],
            )
        })?;

        if deleted == 0 {
            return Err(IndexError::NotOrphaned {
                work_id: work_id.to_owned(),
            });
        }
        Ok(())
    }
}

/// Mọi thư mục con của `root` mang đuôi `.atproj`, **sắp xếp** — thứ tự quét phải tất định để
/// "giữ mục đầu" (§Boundaries, phát hiện trùng `work_id`) không phụ thuộc thứ tự trả về của
/// hệ điều hành, thứ `std::fs::read_dir` **không** đảm bảo ổn định.
///
/// 🔴 **VÁ (vòng rà ba lớp, P5 + P6).** Bản trước dùng `?` ở HAI chỗ: một trên chính
/// `read_dir(root)`, một trên MỖI `entry` khi liệt kê. Cả hai đều sai theo cùng một kiểu — một
/// entry hỏng GIỮA lượt liệt kê vứt luôn mọi `.atproj` đã đọc được TRƯỚC nó (ngược nguyên tắc
/// chính của story: *"các Tác phẩm còn lại vẫn vào chỉ mục"*), còn `read_dir(root)` trả
/// `NotFound` (root biến mất giữa lúc `Indexer::rebuild` kiểm `root.exists()` và lúc hàm này
/// chạy — TOCTOU thật, không lý thuyết) bị biến thành một `IndexError::Io` CỨNG thay vì nhánh
/// `root_missing` êm ái đã tồn tại sẵn cho đúng tình huống đó.
fn scan_atproj_dirs(root: &Path) -> Result<ScanRootOutcome, IndexError> {
    let entries = match std::fs::read_dir(root) {
        Ok(entries) => entries,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            return Ok(ScanRootOutcome::RootMissing);
        }
        Err(e) => {
            return Err(IndexError::Io {
                path: root.to_path_buf(),
                detail: e.to_string(),
            });
        }
    };

    let (paths, entry_errors) = partition_dir_entries(entries.map(|r| r.map(|entry| entry.path())));

    let mut dirs: Vec<PathBuf> = paths
        .into_iter()
        // ⚠️ `is_dir()` theo symlink — chấp nhận được ở đây: không có lý do nghiệp vụ nào một
        // `.atproj` là symlink, và nếu có, quét nó cũng vô hại (đọc `meta.json`, không ghi).
        .filter(|path| {
            path.is_dir() && path.extension().and_then(|ext| ext.to_str()) == Some(ATPROJ_EXTENSION)
        })
        .collect();
    dirs.sort();

    // Không có đường dẫn CỤ THỂ cho một entry hỏng — chính `DirEntry` không đọc được, nên
    // `SkippedEntry::path` neo vào `root` (điểm gần nhất còn biết được) thay vì bịa một đường
    // dẫn con.
    let skipped = entry_errors
        .into_iter()
        .map(|detail| SkippedEntry {
            path: root.to_path_buf(),
            reason: format!("read_dir entry: {detail}"),
        })
        .collect();

    Ok(ScanRootOutcome::Scanned(AtprojScan { dirs, skipped }))
}

/// Kết quả [`scan_atproj_dirs`] khi `read_dir(root)` **thành công** — danh sách `.atproj` tìm
/// được (đã sắp) cộng mọi entry lỗi trong lúc liệt kê (P5), gộp thẳng vào
/// [`RebuildOutcome::skipped`] ở chỗ gọi.
struct AtprojScan {
    dirs: Vec<PathBuf>,
    skipped: Vec<SkippedEntry>,
}

/// Kết quả [`scan_atproj_dirs`] — thêm nhánh `RootMissing` cho ca đua P6, tách khỏi
/// `Result<_, IndexError>` vì đây KHÔNG phải một lỗi (đúng khuôn `root_missing` mà
/// [`Indexer::rebuild`] đã dùng cho fast-path `!root.exists()`).
enum ScanRootOutcome {
    Scanned(AtprojScan),
    RootMissing,
}

/// Vị từ THUẦN — tách các entry LỖI ra khỏi các entry ĐỌC ĐƯỢC mà KHÔNG dùng `?`, nên một entry
/// hỏng không vứt mất những entry đã đọc thành công trước hay sau nó (P5). Tách khỏi
/// `scan_atproj_dirs` để test được TRỰC TIẾP trên một chuỗi dựng tay — `DirEntry` không có
/// constructor công khai, nên đây là ranh giới xa nhất còn viết được một ca hợp đồng thuần cho
/// đúng thuộc tính này.
fn partition_dir_entries<I>(entries: I) -> (Vec<PathBuf>, Vec<String>)
where
    I: IntoIterator<Item = std::io::Result<PathBuf>>,
{
    let mut oks = Vec::new();
    let mut errs = Vec::new();
    for entry in entries {
        match entry {
            Ok(path) => oks.push(path),
            Err(e) => errs.push(e.to_string()),
        }
    }
    (oks, errs)
}

#[cfg(test)]
mod tests {
    use super::partition_dir_entries;
    use std::path::PathBuf;

    /// P5, đối chứng DƯƠNG trên chuỗi dựng tay: một entry LỖI đứng GIỮA hai entry đọc được
    /// không được vứt mất entry ĐỨNG TRƯỚC nó — đúng lỗi mà bản `?` trần mắc phải (huỷ cả lượt
    /// bằng lỗi đầu tiên gặp phải, mất luôn phần đã tích luỹ).
    #[test]
    fn an_error_in_the_middle_does_not_discard_entries_already_collected() {
        let a = PathBuf::from("/root/A.atproj");
        let b = PathBuf::from("/root/B.atproj");
        let entries = vec![
            Ok(a.clone()),
            Err(std::io::Error::other("entry hong gia lap")),
            Ok(b.clone()),
        ];

        let (oks, errs) = partition_dir_entries(entries);

        assert_eq!(
            oks,
            vec![a, b],
            "CA HAI entry doc duoc (truoc VA sau entry loi) phai con nguyen trong ket qua"
        );
        assert_eq!(errs.len(), 1, "dung MOT loi duoc gom lai, khong bi nuot va khong bi nhan doi");
        assert!(errs[0].contains("entry hong gia lap"));
    }

    /// Đối chứng ÂM: không entry nào lỗi ⇒ danh sách lỗi rỗng, mọi đường dẫn giữ nguyên thứ tự
    /// đưa vào (sắp xếp là việc của `scan_atproj_dirs`, không phải của vị từ thuần này).
    #[test]
    fn no_errors_means_every_path_survives_and_the_error_list_is_empty() {
        let a = PathBuf::from("/root/A.atproj");
        let b = PathBuf::from("/root/B.atproj");
        let (oks, errs) = partition_dir_entries(vec![Ok(a.clone()), Ok(b.clone())]);

        assert_eq!(oks, vec![a, b]);
        assert!(errs.is_empty());
    }

    /// Đối chứng biên: MỌI entry đều lỗi ⇒ danh sách đường dẫn rỗng (không panic, không `Err`
    /// đẩy lên) — vị từ thuần luôn trả `Ok`-shape, quyết định "đây có phải lỗi cứng không" là
    /// việc của chỗ gọi.
    #[test]
    fn every_entry_failing_still_returns_cleanly_with_an_empty_path_list() {
        let (oks, errs) = partition_dir_entries(vec![
            Err(std::io::Error::other("e1")),
            Err(std::io::Error::other("e2")),
        ]);

        assert!(oks.is_empty());
        assert_eq!(errs.len(), 2);
    }
}

/// Kết quả một lượt [`Indexer::rebuild`] — rỗng phải có LÝ DO, không rỗng im lặng
/// (`AGENTS.md::Known pitfalls`).
#[derive(Debug, Clone)]
pub struct RebuildOutcome {
    /// Số hàng đã ghi vào `library_work` ở lượt này.
    pub indexed: usize,
    /// `true` ⇒ thư mục gốc Library **chưa tồn tại**. Phân biệt với `indexed == 0 &&
    /// root_missing == false` (gốc có tồn tại nhưng không chứa `.atproj` nào — "đã quét, thật
    /// sự rỗng") — hai trạng thái khác nhau mà một con số `0` một mình không nói được.
    pub root_missing: bool,
    /// Hai (hoặc nhiều) `.atproj` cùng `work_id`. Mục ĐẦU (theo thứ tự quét đã sắp) được giữ
    /// trong chỉ mục; mọi mục SAU liệt ở đây — không gộp, không ghi đè im lặng.
    pub conflicts: Vec<WorkIdConflict>,
    /// `.atproj` bị bỏ qua: thiếu/hỏng `meta.json`, hoặc `meta_schema_version` mới hơn bản
    /// ứng dụng hiểu. Các Tác phẩm còn lại vẫn vào chỉ mục bình thường.
    pub skipped: Vec<SkippedEntry>,
    /// **THÊM Story 5.3.** Số hàng vừa CHUYỂN sang `orphaned = 1` ở lượt này — không phải
    /// tổng số hàng mồ côi hiện có (đó là `Indexer::list_orphans().len()`). `0` là bình
    /// thường (mọi hàng đã ở đúng vị trí, hoặc đây là lượt quét đầu tiên trên một chỉ mục
    /// rỗng); khác `root_missing`, một `orphans > 0` không tự nó là một lỗi — nó là bằng
    /// chứng đối chiếu đang hoạt động.
    pub orphans: usize,
}

impl RebuildOutcome {
    /// Ghi chẩn đoán KHÔNG DẤU ra stderr khi [`Self::conflicts`]/[`Self::skipped`] không rỗng.
    ///
    /// 🔴 **THÊM (vòng rà ba lớp, P7)** — AD-28 đòi `Indexer` *"phát hiện VÀ CẢNH BÁO hai Tác
    /// phẩm trùng `work.id`"*. Trước bản vá, cả hai chỗ gọi sản phẩm (`lib.rs::open_library_index`
    /// lúc khởi động, `commands::project::wire::reindex_after_create_work` sau khi tạo Tác
    /// phẩm) đều VỨT `RebuildOutcome` bằng `if let Err(err) = indexer.rebuild(..) { .. }` —
    /// nghĩa là vế "phát hiện" có giá trị trả về THẬT (có test), nhưng vế "CẢNH BÁO" không có
    /// một đầu ra nào: không ai từng đọc `conflicts`/`skipped` ngoài `tests/**`. Hàm này là
    /// đường CHUNG cho cả hai chỗ gọi — tách ra để chúng không thể trôi khỏi nhau (đúng khuôn
    /// `commands::project::guarded_dict_layers`, vốn cũng nhận `surface: &str` để chẩn đoán
    /// nêu đúng chỗ gọi).
    ///
    /// ⚠️ Đây CHỈ là chẩn đoán cho log (NFR16) — bề mặt HIỂN THỊ cho người dùng vẫn là Story
    /// 5.6, không dựng ở đây (§Never của story 5.2).
    pub fn log_if_notable(&self, surface: &str) {
        if let Some(first) = self.conflicts.first() {
            eprintln!(
                "library[index:{surface}] {} work_id trung nhau -- vd. work_id={} giu {} bo qua {}",
                self.conflicts.len(),
                first.work_id,
                first.kept_path.display(),
                first.duplicate_path.display()
            );
        }
        if let Some(first) = self.skipped.first() {
            eprintln!(
                "library[index:{surface}] {} .atproj bi bo qua khi dung chi muc -- vd. {} ({})",
                self.skipped.len(),
                first.path.display(),
                first.reason
            );
        }
        // **THÊM Story 5.3** -- ve mo coi vao CUNG duong chan doan nay, khong dung mot
        // duong thu hai (doc-comment cua ham nay da noi ro ly do: hai cho goi san pham
        // khong duoc trooi khoi nhau).
        if self.orphans > 0 {
            eprintln!(
                "library[index:{surface}] {} .atproj thanh mo coi o luot quet nay -- \
                 atproj_path khong con nam trong tap vua liet ke duoc",
                self.orphans
            );
        }
    }
}

/// Một cặp `.atproj` cùng `work_id` — mục ĐẦU được giữ, mục SAU bị loại khỏi chỉ mục.
#[derive(Debug, Clone)]
pub struct WorkIdConflict {
    pub work_id: String,
    /// Đường dẫn `.atproj` **đang có mặt** trong chỉ mục.
    pub kept_path: PathBuf,
    /// Đường dẫn `.atproj` **trùng `work_id`**, bị loại khỏi lượt ghi này.
    pub duplicate_path: PathBuf,
}

/// Một `.atproj` bị bỏ qua trong lượt quét — thiếu/hỏng `meta.json`, hoặc lược đồ `meta.json`
/// mới hơn bản ứng dụng hiểu (`MetaError::SchemaTooNew`, cùng nhánh xử lý).
#[derive(Debug, Clone)]
pub struct SkippedEntry {
    pub path: PathBuf,
    /// Chẩn đoán, KHÔNG DẤU (`Display` của [`super::meta::MetaError`]) — không phải văn bản
    /// hiển thị (NFR16); Story sở hữu bề mặt hiển thị của trường này là Story 5.6.
    pub reason: String,
}

/// Một hàng của `library_work`, cho đường đọc [`Indexer::list_works`]/[`Indexer::list_orphans`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IndexedWork {
    pub work_id: String,
    /// Đường dẫn TUYỆT ĐỐI trên máy này — khác `meta.json`, nơi đường tuyệt đối bị cấm
    /// (AC5, Story 1.15). Xem doc-comment của `LIBRARY_WORK_DDL` (`core/store/schema.rs`).
    ///
    /// Trên một hàng MỒ CÔI, đây là đường dẫn CŨ — cố ý giữ nguyên, không xoá/làm rỗng: AC3
    /// đòi hàng mồ côi "nêu rõ nó trỏ tới đâu".
    pub atproj_path: PathBuf,
    pub name: String,
    pub source_lang: String,
    pub genre: String,
    pub created_at: String,
    pub updated_at: String,
    pub chapter_count: u32,
    /// **THÊM Story 5.3.** `true` ⇒ hàng mồ côi (`.atproj` không còn nằm trong tập vừa quét
    /// được). [`Indexer::list_works`] chỉ trả hàng `false`; [`Indexer::list_orphans`] chỉ trả
    /// hàng `true` — trường này có mặt trên cả hai đường đọc để một chỗ gọi tương lai gộp cả
    /// hai (nếu có) vẫn phân biệt được, không đoán từ đường gọi.
    pub orphaned: bool,
}

/// Mọi cách một lượt [`Indexer::rebuild`] hỏng mà KHÔNG phải một `.atproj` con bị bỏ qua (đó
/// đi vào [`SkippedEntry`], không phải `Err`) — quét thư mục gốc trượt (quyền, đĩa hỏng), hoặc
/// kho ghi trượt.
#[derive(Debug)]
pub enum IndexError {
    /// Đọc thư mục GỐC trượt. **Khác** "gốc chưa tồn tại"
    /// ([`RebuildOutcome::root_missing`], không phải lỗi) — đây là gốc CÓ tồn tại nhưng không
    /// đọc được (quyền, đĩa hỏng).
    Io { path: PathBuf, detail: String },
    /// Kho `library-index.db` ghi trượt.
    Store(StoreError),
    /// **THÊM Story 5.3.** [`Indexer::forget_orphan`] gọi trên một `work_id` không tồn tại,
    /// hoặc tồn tại nhưng đang SỐNG (`orphaned = 0`) — cùng một nhánh từ chối cho cả hai ca
    /// (§I/O Matrix của story).
    NotOrphaned { work_id: String },
}

impl std::fmt::Display for IndexError {
    /// ⚠️ KHÔNG DẤU — chẩn đoán cho log (NFR16), cùng luật mọi kiểu lỗi khác của
    /// `src-tauri/src/**`.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            IndexError::Io { path, detail } => {
                write!(f, "index[{}] io failed: {detail}", path.display())
            }
            IndexError::Store(err) => write!(f, "index[store] {err}"),
            IndexError::NotOrphaned { work_id } => {
                write!(f, "index[forget_orphan] work_id={work_id} is not an orphaned row")
            }
        }
    }
}

impl std::error::Error for IndexError {}

impl From<StoreError> for IndexError {
    fn from(err: StoreError) -> Self {
        IndexError::Store(err)
    }
}

/// 🔴 Đi qua [`IpcError::new`], không dựng struct literal — cùng luật với
/// `From<StoreError> for IpcError` (`core/store/mod.rs`).
///
/// ⚠️ `IndexError::Io` tái dùng [`MessageKey::IoReadFailed`] thay vì đúc một khoá thứ ba
/// (§Tasks của story `5-3-quet-lai-thu-muc.md`: "danh mục đóng, và `IndexError::Io` tái
/// dùng `IoReadFailed`").
impl From<IndexError> for IpcError {
    fn from(err: IndexError) -> Self {
        match err {
            IndexError::Io { path, detail } => {
                let mut params = BTreeMap::new();
                params.insert("path".to_owned(), path.display().to_string());
                let _ = detail; // Chẩn đoán thô -- không đi vào `params` (AD-21: params mang dữ liệu, không mang câu).
                IpcError::new("library.io_read_failed", MessageKey::IoReadFailed, params, false)
            }
            IndexError::Store(err) => err.into(),
            IndexError::NotOrphaned { work_id } => {
                let mut params = BTreeMap::new();
                params.insert("work_id".to_owned(), work_id);
                IpcError::new(
                    "library.not_orphaned",
                    MessageKey::LibraryNotOrphaned,
                    params,
                    false,
                )
            }
        }
    }
}
