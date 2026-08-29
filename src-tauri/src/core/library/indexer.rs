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
//! 🔵 SỬA (2026-08-27, phán quyết Ice #1) — CỜ MỒ CÔI KHÔNG CÒN SỐNG Ở KHO NÀY
//! ─────────────────────────────────────────────────────────────────────────────
//! Vòng dựng đầu của Story 5.3 thêm một cột `orphaned` NGAY TRONG `library_work` — tức
//! `library-index.db` không còn dẫn xuất TRỌN VẸN từ đĩa (cờ mồ côi là mẩu trạng thái duy
//! nhất không suy ra được từ `.atproj`). Ice lật quyết định đó: cờ mồ côi là **dữ liệu người
//! dùng** (một quyết định "tôi biết đường dẫn cũ, tôi CHƯA gỡ nó", không phải một cache), nên
//! nó sống ở bảng `library_orphan` của **`global.db`** (xem [`super::orphan_store`]) —
//! `library_work` quay lại đúng nghĩa cũ: "những gì đang có mặt trên đĩa NGAY BÂY GIỜ", dẫn
//! xuất trọn vẹn, không hàng nào sống sót một lượt xoá-dựng-lại. Vì `global.db` và
//! `library-index.db` là HAI kho với HAI `store::Writer` riêng (không giao dịch xuyên kho),
//! [`Indexer::rebuild`]/[`Indexer::forget_orphan`]/[`Indexer::list_orphans`] nay nhận thêm
//! một tham số `global: Option<&Store>` — chỗ gọi (lớp lệnh, `lib.rs`) phải truyền `Store`
//! toàn cục ĐÃ MỞ vào. Xem doc-comment của từng hàm cho THỨ TỰ ghi giữa hai kho (fail-safe,
//! không phải tuỳ ý).
//!
//! ─────────────────────────────────────────────────────────────────────────────
//! NĂM THAO TÁC — Story 5.3 thêm hai (GỠ mồ côi tường minh, LIỆT KÊ mồ côi)
//! ─────────────────────────────────────────────────────────────────────────────
//! - [`Indexer::open`] — mở (hoặc dựng mới) `library-index.db`, hiện thực nhánh KHÔNG-DI-TRÚ
//!   của AD-8: lệch phiên bản lược đồ (cả hai chiều) ⇒ xoá tệp + sidecar rồi dựng lại, KHÔNG
//!   đi qua nhánh từ chối mở mà `project.db`/`global.db` dựa vào (AD-30).
//! - [`Indexer::rebuild`] — quét thư mục gốc Library, đọc `meta.json` của mỗi `.atproj`, rồi
//!   **ĐỐI CHIẾU** kết quả với `library_work` trong **một** giao dịch qua `store::Writer`.
//!   🔵 **SỬA (2026-08-29, Story 5.9) — mệnh đề "đọc CHỈ `meta.json` (AD-9: không mở
//!   `project.db` lần nào)" ở dòng vừa rồi ĐÃ HẾT ĐÚNG.** Cùng giao dịch đó nay CÒN mở
//!   `project.db` của mỗi Tác phẩm trong `kept`, **CHỈ ĐỌC** qua [`crate::core::store::ReadOnlyDb`]
//!   (`StoreKind::Project`, miễn trừ CÓ TÊN ở `core/store/readonly.rs`) để thu hoạch văn bản
//!   vào `library_segment`/ba chỉ mục FTS5 (FR8, xem [`harvest_work_text`]; 🔵 hai → ba,
//!   2026-08-29, Story 5.10: `library_target_fts_nd` — xem [`Indexer::search`]) — KHÔNG BAO GIỜ
//!   qua `Store::open` (đường đó GHI vào tệp: `journal_mode`, bộ di trú, luồng writer). AD-9
//!   ("Indexer chỉ đọc `meta.json`") vẫn đúng cho phần METADATA (`library_work`); nó không còn
//!   đúng cho TOÀN BỘ mô-đun này. Một lượt trượt thu hoạch (project.db vắng mặt/mới hơn/hỏng)
//!   CHỈ bỏ qua văn bản của đúng Tác phẩm đó và đếm vào [`RebuildOutcome::text_skipped`] —
//!   không làm trượt cả lượt `rebuild`, đúng khuôn `conflicts`/`skipped` đã có. 🔵 **ĐỔI NGỮ
//!   NGHĨA (Story 5.3):** trước đây hàm này `DELETE FROM library_work` rồi `INSERT` lại toàn bộ — một
//!   `.atproj` bị xoá/di chuyển ra ngoài gốc biến mất khỏi chỉ mục IM LẶNG. Nay nó UPSERT mọi
//!   mục đọc được vào `library_work`, rồi với mọi hàng CÒN LẠI mà `atproj_path` KHÔNG nằm
//!   trong tập `.atproj` vừa liệt kê được: ghi một bản ghi vào `library_orphan`
//!   (`global.db`) rồi MỚI xoá hàng đó khỏi `library_work` — hàng không biến mất, nó CHUYỂN
//!   KHO (§Design Notes "vị từ mồ côi: bốn cách viết, ba cách sai"; §Rủi ro/§Spec Change Log
//!   của story cho lý do thứ tự ghi). Toàn bộ scan+ghi (CẢ HAI kho) chạy dưới
//!   [`Indexer::rebuild_lock`] — hai lượt `rebuild` gọi đồng thời phải NỐI TIẾP, không xen kẽ
//!   giai đoạn quét với giai đoạn ghi (deferred-work.md:8079, chủ Story 5.3).
//!   Đây vẫn là đường ghi DUY NHẤT của module này — không có một đường "chèn một hàng" thứ
//!   hai chạy song song với nó, kể cả khi chỉ một Tác phẩm vừa được tạo (xem
//!   `commands::project::wire::create_work_from_text`, nơi gọi lại đúng hàm này).
//! - [`Indexer::forget_orphan`] — **THÊM Story 5.3.** Xoá đúng MỘT hàng mồ côi khỏi
//!   `library_orphan` (`global.db`) — đường XOÁ tường minh, có tiền điều kiện "hàng tồn tại",
//!   không phải một đường ghi thứ hai (§Design Notes "vì sao forget_orphan không phải đường
//!   ghi thứ hai").
//! - [`Indexer::list_works`] — đường ĐỌC mọi hàng của `library_work` (nay LUÔN "đang sống",
//!   không còn cờ `orphaned` để lọc — kho này dẫn xuất trọn vẹn trở lại), dùng cho Story
//!   5.6/5.9.
//! - [`Indexer::list_orphans`] — **THÊM Story 5.3.** Đường ĐỌC mọi hàng mồ côi từ
//!   `library_orphan` (`global.db`), dùng cho màn hình tối thiểu của story này.
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
    LIBRARY_INDEX_MIGRATIONS, PROJECT_MIGRATIONS, ReadHandle, ReadOnlyDb, Row, SqlError, SqlResult,
    Store, StoreError, StoreKind, StoreSpec, Transaction, params_from_iter,
};

use super::meta::WorkMeta;
use super::orphan_store::{self, OrphanRecord};

/// Kho `global.db` vắng mặt khi một thao tác Library cần đọc/ghi `library_orphan` — đi qua
/// `StoreError::OpenFailed` cùng khuôn `commands::pinned::store_is_missing`/
/// `commands::library::store_is_missing`, không dựng một biến thể `IndexError` riêng cho
/// đúng một câu này (danh mục `MessageKey` của story chỉ đóng đúng hai khoá mới).
fn global_store_missing(surface: &'static str) -> IndexError {
    IndexError::Store(StoreError::OpenFailed {
        store: StoreKind::Global,
        detail: format!(
            "global store missing while {surface} needs library_orphan -- xem phan quyet Ice #1"
        ),
    })
}

/// Đuôi thư mục của một Tác phẩm — cùng hằng `WORK_FOLDER_SUFFIX` của [`super::atproj`], chép
/// lại vì hằng đó là `const` riêng tư của module kia và không có lý do lộ ra `pub(crate)` chỉ
/// cho một lần so sánh chuỗi ở đây.
const ATPROJ_EXTENSION: &str = "atproj";

/// Danh sách cột dùng chung cho [`Indexer::list_works`] và [`Indexer::find_work`] — MỘT
/// hình dạng hàng, hai mệnh đề `WHERE` khác nhau. Tách ra khỏi thân hàm (Story 5.7) vì
/// closure `map_row` cũ của `list_works` không chia sẻ được sang một hàm thứ hai.
const INDEXED_WORK_COLUMNS: &str = "work_id, atproj_path, name, source_lang, genre, created_at, \
     updated_at, chapter_count, status, status_is_override, chapter_done_count";

/// Ánh xạ một hàng `library_work` (cột đúng thứ tự [`INDEXED_WORK_COLUMNS`]) sang
/// [`IndexedWork`] — hàm TỰ DO (Story 5.7), không phải closure lồng trong
/// [`Indexer::list_works`], để [`Indexer::find_work`] gọi lại được.
fn map_indexed_work_row(row: &Row<'_>) -> SqlResult<IndexedWork> {
    let status_is_override: i64 = row.get(9)?;
    Ok(IndexedWork {
        work_id: row.get(0)?,
        atproj_path: PathBuf::from(row.get::<_, String>(1)?),
        name: row.get(2)?,
        source_lang: row.get(3)?,
        genre: row.get(4)?,
        created_at: row.get(5)?,
        updated_at: row.get(6)?,
        chapter_count: row.get(7)?,
        status: row.get(8)?,
        status_is_override: status_is_override != 0,
        chapter_done_count: row.get(10)?,
    })
}

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

    /// Quét `root`, đọc `meta.json` của mỗi `<Tên>.atproj/`, và **đối chiếu** kết quả với
    /// `library_work` (`library-index.db`) VÀ `library_orphan` (`global.db`, xem
    /// [`super::orphan_store`]) — đường ghi DUY NHẤT của module này cho cả hai bảng.
    ///
    /// `root` không tồn tại ⇒ chỉ mục **rỗng có lý do** ([`RebuildOutcome::root_missing`]),
    /// không tạo thư mục, không lỗi (§I/O Matrix "Thư mục gốc vắng").
    ///
    /// ─────────────────────────────────────────────────────────────────────────────
    /// 🔴 THỨ TỰ GHI GIỮA HAI KHO — FAIL-SAFE, KHÔNG TUỲ Ý (phán quyết Ice #1)
    /// ─────────────────────────────────────────────────────────────────────────────
    /// `global.db` và `library-index.db` là HAI kho, mỗi kho một `store::Writer` riêng —
    /// KHÔNG có giao dịch nào bọc được cả hai cùng lúc, nên một hàng chuyển từ "đang sống"
    /// sang "mồ côi" LUÔN đi qua HAI lượt ghi tách rời. Chọn: ghi `library_orphan`
    /// (`global.db`) TRƯỚC, rồi mới xoá hàng tương ứng khỏi `library_work`
    /// (`library-index.db`) SAU. Lý do: nếu bước hai (xoá khỏi chỉ mục) trượt sau khi bước
    /// một (ghi global) đã commit, hàng đó tạm thời có mặt ở CẢ HAI kho — lượt `rebuild` kế
    /// tiếp tự sửa (nó vẫn không nằm trong `kept`, vẫn không `unreadable`, nên vẫn được xử lý
    /// lại), không mất gì. Thứ tự NGƯỢC LẠI (xoá khỏi chỉ mục trước, ghi global sau) mà bước
    /// hai trượt thì hàng biến mất khỏi CẢ HAI nơi cùng lúc — lời nhắc mồ côi mất VĨNH VIỄN,
    /// không còn dấu vết nào để một lượt quét sau tự sửa. Ca hợp đồng cho đúng thuộc tính
    /// này: `tests/library_index_contract.rs::orphan_write_order_is_fail_safe_write_global_before_deleting_from_index`.
    ///
    /// Chiều ngược lại (mồ côi QUAY LẠI): `library_work` được UPSERT TRƯỚC (trong cùng giao
    /// dịch với mọi mục `kept` khác), rồi `library_orphan` mới được dọn SAU. Nếu bước dọn dẹp
    /// đó trượt, hàng nằm ở CẢ HAI nơi tạm thời (một mục ĐANG SỐNG trong chỉ mục, MỘT bản ghi
    /// mồ côi cũ còn sót trong `global.db`) — không mất dữ liệu, chỉ là một lời nhắc thừa mà
    /// lượt `rebuild` kế tiếp dọn tiếp (idempotent). Đối xứng với chiều trên: thao tác nào có
    /// thể tự sửa ở lượt sau luôn đứng SAU thao tác không tự sửa được.
    ///
    /// # Lỗi
    /// [`IndexError::Io`] nếu `root` tồn tại nhưng không đọc được (quyền, đĩa hỏng) —
    /// **khác** một `.atproj` con bị hỏng, thứ đó đi vào [`RebuildOutcome::skipped`], không
    /// phải `Err` (§Boundaries: *"một `.atproj` thiếu/hỏng `meta.json` cũng phải phân biệt
    /// được với 'không có Tác phẩm nào', không rơi im lặng"*). [`IndexError::Store`] nếu một
    /// trong hai lượt ghi trượt — bao gồm `global` vắng mặt (`store: StoreKind::Global`).
    pub fn rebuild(&self, root: &Path, global: Option<&Store>) -> Result<RebuildOutcome, IndexError> {
        // Khoá TOÀN BỘ scan+ghi (CẢ HAI kho) ngay từ đây — xem doc-comment của
        // `rebuild_lock`. Giữ khoá xuyên suốt cả hàm (biến `_guard` sống tới cuối scope) là
        // điểm mấu chốt: nó không chỉ khoá lượt ghi (đã nối tiếp qua `store::Writer` của mỗi
        // kho MỘT MÌNH), nó khoá cả lượt ĐỌC ĐĨA phía trên VÀ khoảng hở giữa hai lượt ghi
        // xuyên-kho, thứ mà hai lượt `rebuild` gọi gần nhau có thể chạy xen kẽ nếu không có
        // nó.
        let _guard = self
            .rebuild_lock
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);

        // `global` được đòi NGAY TỪ ĐẦU, kể cả khi lượt này hoá ra không có gì để chuyển kho:
        // mọi nhánh trả về đều cần đọc `RebuildOutcome::current_orphans` từ `global.db`
        // (xem dưới), nên vắng mặt luôn là một lỗi thật, không một chỗ ngầm bỏ qua.
        let global = global.ok_or_else(|| global_store_missing("Indexer::rebuild"))?;

        if !root.exists() {
            return self.mark_all_orphaned_for_missing_root(global);
        }

        let scan = match scan_atproj_dirs(root)? {
            // 🔴 VÁ (vòng rà ba lớp, P6) — `root.exists()` ngay trên và `read_dir(root)` bên
            // trong `scan_atproj_dirs` là HAI bước tách rời: root biến mất GIỮA hai bước (ổ
            // ngoài rút ra đúng lúc) làm `read_dir` trả `NotFound`. Trước bản vá, `?` biến ca
            // đó thành `IndexError::Io` — một LỖI CỨNG cho đúng tình huống mà nhánh
            // `root_missing` êm ái ngay trên đã tồn tại để canh. `scan_atproj_dirs` tự ánh xạ
            // `NotFound` về [`ScanRootOutcome::RootMissing`]; xử lý y hệt fast-path phía trên.
            ScanRootOutcome::RootMissing => return self.mark_all_orphaned_for_missing_root(global),
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
        // `kept_ids` tách RA TRƯỚC khi `first_seen` bị `move` vào closure ngay dưới — dùng
        // cho bước dọn `library_orphan` (chiều "mồ côi quay lại") SAU khi giao dịch dưới đây
        // đã commit.
        let kept_ids: Vec<String> = first_seen.keys().cloned().collect();

        // Bước 1 (library-index.db): UPSERT mọi mục `kept`, rồi xác định tập CHUYỂN SANG mồ
        // côi ở lượt này — nhưng KHÔNG xoá gì khỏi `library_work` trong CHÍNH giao dịch này.
        // Việc xoá phải đợi bước 2 (ghi `global.db`) thành công trước — xem khối 🔴 ở
        // doc-comment hàm này.
        let (to_orphan, text_skipped): (Vec<OrphanRecord>, Vec<TextHarvestSkipped>) = self.store.write(move |tx: &Transaction<'_>| {
            // Vế MỘT của vị từ mồ côi — "work_id không đọc được ở lượt này" — thoả bằng
            // chính việc một hàng KHÔNG nằm trong `kept` (nó không được UPSERT ở đây).
            //
            // 🔴 UPSERT, không `DELETE` + `INSERT` — chỗ đổi ngữ nghĩa CHÍNH của Story 5.3.
            // `work_id` là khoá chính nên `ON CONFLICT` là cách SQLite tự phân biệt "Tác
            // phẩm này đã có trong chỉ mục" (cập nhật đường dẫn/metadata) với "Tác phẩm mới"
            // (chèn hàng) — không cần một `SELECT` kiểm trùng ở tầng Rust.
            //
            // 🔵 THÊM (2026-08-27, Story 5.4) — hai cột `status`/`status_is_override` chở
            // NGUYÊN VẸN giá trị mà `WorkMeta::rebuild_from_store` đã tính (chỗ DUY NHẤT
            // tính giá trị suy ra, §Approach của story) — kho này KHÔNG tự tính lại.
            //
            // 🔵 THÊM (2026-08-28, Story 5.5) — cột `chapter_done_count` chở NGUYÊN VẸN giá trị
            // mà `WorkMeta::rebuild_from_store` đã đếm, cùng lý lẽ trên. `rusqlite` chuyển
            // `Option<u32>` thành `NULL` khi `None` -- không cần chuyển đổi tay như
            // `status_is_override` (đó là `bool` KHÔNG `Option`, SQLite không có kiểu boolean).
            for (dir, meta) in &kept {
                tx.execute(
                    "INSERT INTO library_work \
                     (work_id, atproj_path, name, source_lang, genre, created_at, \
                      updated_at, chapter_count, status, status_is_override, \
                      chapter_done_count) \
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11) \
                     ON CONFLICT (work_id) DO UPDATE SET \
                       atproj_path         = excluded.atproj_path, \
                       name                = excluded.name, \
                       source_lang         = excluded.source_lang, \
                       genre               = excluded.genre, \
                       created_at          = excluded.created_at, \
                       updated_at          = excluded.updated_at, \
                       chapter_count       = excluded.chapter_count, \
                       status              = excluded.status, \
                       status_is_override  = excluded.status_is_override, \
                       chapter_done_count  = excluded.chapter_done_count",
                    (
                        &meta.work_id,
                        &dir.display().to_string(),
                        &meta.name,
                        &meta.source_lang,
                        &meta.genre,
                        &meta.created_at,
                        &meta.updated_at,
                        meta.chapter_count,
                        &meta.status,
                        i64::from(meta.status_is_override),
                        meta.chapter_done_count,
                    ),
                )?;
            }

            // ─────────────────────────────────────────────────────────────────────────
            // Story 5.9 — THU HOẠCH VĂN BẢN, TRONG CÙNG GIAO DỊCH NÀY (§Always của story:
            // "Indexer::rebuild vẫn là đường ghi DUY NHẤT vào library-index.db").
            // ─────────────────────────────────────────────────────────────────────────
            // `library_segment`/ba chỉ mục FTS5 (🔵 hai → ba, Story 5.10) là kho DẪN XUẤT TRỌN VẸN (đúng nghĩa AD-8):
            // không có khái niệm "Tác phẩm mồ côi giữ lại văn bản cũ" ở đây như
            // `library_work`/`library_orphan` phía trên — mỗi lượt `rebuild` XOÁ SẠCH rồi nạp
            // lại từ CHÍNH `kept` (tập `.atproj` đọc được `meta.json` Ở LƯỢT NÀY). Một Tác phẩm
            // vừa mồ côi (`atproj_path` biến mất) mất luôn nội dung tìm kiếm của nó — đúng bản
            // chất "chỉ mục nói về thư viện ĐANG CÓ", không phải một kho lưu trữ.
            tx.execute("DELETE FROM library_segment", [])?;

            let mut text_skipped: Vec<TextHarvestSkipped> = Vec::new();
            for (dir, meta) in &kept {
                match harvest_work_text(dir) {
                    Ok(rows) => {
                        for row in rows {
                            tx.execute(
                                "INSERT INTO library_segment \
                                 (work_id, chapter_id, chapter_ord, chapter_title, segment_id, \
                                  segment_ord, source_text, target_text) \
                                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
                                (
                                    &meta.work_id,
                                    row.chapter_id,
                                    row.chapter_ord,
                                    &row.chapter_title,
                                    row.segment_id,
                                    row.segment_ord,
                                    &row.source_text,
                                    &row.target_text,
                                ),
                            )?;
                        }
                    }
                    // 🔴 Một lượt thu hoạch trượt cho ĐÚNG MỘT Tác phẩm KHÔNG được làm trượt cả
                    // `rebuild` — metadata của nó (`library_work`, vừa UPSERT ở trên) vẫn đúng;
                    // chỉ phần văn bản của riêng nó vắng mặt, và điều đó phải ĐẾM ĐƯỢC
                    // (`RebuildOutcome::text_skipped`), không im lặng.
                    Err(reason) => {
                        eprintln!(
                            "library[index:rebuild] bo qua thu hoach van ban cho work_id={} -- {reason}",
                            meta.work_id
                        );
                        text_skipped.push(TextHarvestSkipped { work_id: meta.work_id.clone(), reason });
                    }
                }
            }

            // Nạp lại TOÀN BỘ ba chỉ mục FTS5 (🔵 hai → ba, Story 5.10) từ nội dung
            // `library_segment` VỪA ghi xong — khuôn 'rebuild' external-content chuẩn của FTS5.
            // Chạy SAU khi mọi `INSERT` ở trên đã xong: một lượt 'rebuild' quét TOÀN BỘ bảng
            // nội dung tại thời điểm nó chạy, không phải một API tăng dần theo từng hàng.
            tx.execute("INSERT INTO library_target_fts(library_target_fts) VALUES('rebuild')", [])?;
            tx.execute("INSERT INTO library_target_fts_nd(library_target_fts_nd) VALUES('rebuild')", [])?;
            tx.execute("INSERT INTO library_source_fts(library_source_fts) VALUES('rebuild')", [])?;

            // Mọi hàng CÒN LẠI (không vừa UPSERT ở trên): mồ côi khi và chỉ khi `atproj_path`
            // của nó KHÔNG nằm trong `unreadable_paths` — vế HAI của vị từ (P3). Đọc lại toàn
            // bảng trong CÙNG giao dịch (không phải một `Store::read` riêng) để không có cửa
            // sổ đua giữa "đọc trạng thái cũ" và "quyết định mồ côi hay không".
            let mut stmt = tx.prepare("SELECT work_id, atproj_path, name FROM library_work")?;
            let existing: Vec<(String, String, String)> = stmt
                .query_map([], |row| {
                    Ok((
                        row.get::<_, String>(0)?,
                        row.get::<_, String>(1)?,
                        row.get::<_, String>(2)?,
                    ))
                })?
                .collect::<crate::core::store::SqlResult<Vec<_>>>()?;
            drop(stmt);

            let mut to_orphan = Vec::new();
            for (work_id, atproj_path, name) in existing {
                if first_seen.contains_key(&work_id) {
                    continue; // Vừa upsert ở trên -- vẫn đang sống.
                }
                if unreadable_paths.contains(&atproj_path) {
                    continue; // Thư mục còn ĐÓ nhưng meta.json hỏng (ca "hỏng nhưng còn" -- KHÔNG mồ côi).
                }
                to_orphan.push(OrphanRecord {
                    work_id,
                    atproj_path,
                    name,
                });
            }

            Ok((to_orphan, text_skipped))
        })?;

        // Bước 2 (global.db TRƯỚC, rồi library-index.db SAU) — chỉ chạy khi có gì để chuyển.
        if !to_orphan.is_empty() {
            orphan_store::upsert_many(global, to_orphan.clone())?;

            let ids: Vec<String> = to_orphan.iter().map(|r| r.work_id.clone()).collect();
            self.store.write(move |tx: &Transaction<'_>| {
                for id in &ids {
                    tx.execute("DELETE FROM library_work WHERE work_id = ?1", [id])?;
                }
                Ok(())
            })?;
        }

        // Chiều ngược lại: mọi work_id CÒN SỐNG ở lượt này không còn lý do gì để có mặt
        // trong `library_orphan` -- dọn dẹp SAU khi `library_work` đã xác nhận nó đang sống
        // (xem khối 🔴 ở doc-comment hàm này). Idempotent, không cần biết trước work_id nào
        // THẬT SỰ có trong bảng mồ côi.
        orphan_store::remove_many(global, kept_ids)?;

        // Ảnh chụp mồ côi lấy NGAY ĐÂY, trong khi `_guard` (rebuild_lock) còn sống: không
        // lượt `rebuild`/`forget_orphan` nào khác chen được vào giữa các giao dịch vừa commit
        // và lượt đọc này. Xem doc-comment của `RebuildOutcome::current_orphans`.
        let current_orphans = orphan_store::list(global)?;

        Ok(RebuildOutcome {
            indexed,
            root_missing: false,
            conflicts,
            skipped,
            orphans: to_orphan.len(),
            current_orphans,
            text_skipped,
        })
    }

    /// Chuyển MỌI hàng đang sống của `library_work` sang mồ côi (`global.db`) và trả một
    /// [`RebuildOutcome`] rỗng-có-lý-do (`root_missing: true`) — dùng CHUNG bởi cả fast-path
    /// (`!root.exists()`) lẫn nhánh đua P6 (`root` biến mất giữa `exists()` và `read_dir`).
    ///
    /// 🔵 **ĐỔI NGỮ NGHĨA (Story 5.3) — trước đây `clear_for_missing_root` XOÁ SẠCH bảng, rồi
    /// đổi thành "đánh dấu `orphaned = 1` tại chỗ" ở vòng dựng đầu.** Phán quyết Ice #1 đổi
    /// nó LẦN NỮA: nay là CHUYỂN KHO — ghi vào `library_orphan` (`global.db`) TRƯỚC, rồi mới
    /// xoá khỏi `library_work` SAU, cùng thứ tự fail-safe mà [`Self::rebuild`] dùng. Gốc vắng
    /// mặt nghĩa là **tập `.atproj` liệt kê được là rỗng** — đúng vị từ mồ côi ở
    /// [`Indexer::rebuild`] áp cho MỌI hàng đang sống, không phải một nhánh riêng "xoá sạch".
    /// Chỉ mục nói về THƯ VIỆN (thư mục gốc đang cấu hình), không nói về đĩa nói chung: các
    /// `.atproj` của một gốc CŨ vẫn có thể còn nguyên trên đĩa (ca "đổi thư mục gốc"), nhưng
    /// chúng không còn nằm trong thư viện đang quét.
    fn mark_all_orphaned_for_missing_root(&self, global: &Store) -> Result<RebuildOutcome, IndexError> {
        let existing: Vec<(String, String, String)> = self.store.read(|conn: ReadHandle<'_>| {
            let mut stmt = conn.prepare("SELECT work_id, atproj_path, name FROM library_work")?;
            stmt.query_map([], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                ))
            })?
            .collect::<crate::core::store::SqlResult<Vec<_>>>()
        })?;

        let orphans = existing.len();
        if !existing.is_empty() {
            let records: Vec<OrphanRecord> = existing
                .iter()
                .map(|(work_id, atproj_path, name)| OrphanRecord {
                    work_id: work_id.clone(),
                    atproj_path: atproj_path.clone(),
                    name: name.clone(),
                })
                .collect();
            // global.db TRƯỚC -- xem khối 🔴 ở doc-comment của `rebuild`.
            orphan_store::upsert_many(global, records)?;

            let ids: Vec<String> = existing.into_iter().map(|(work_id, ..)| work_id).collect();
            self.store.write(move |tx: &Transaction<'_>| {
                for id in &ids {
                    tx.execute("DELETE FROM library_work WHERE work_id = ?1", [id])?;
                }
                Ok(())
            })?;
        }

        // Story 5.9 — gốc vắng mặt ⇒ tập `.atproj` liệt kê được LÀ RỖNG, đúng vị từ mồ côi mà
        // `rebuild()` bình thường áp cho MỌI hàng đang sống (doc-comment hàm này). `library_segment`
        // đi theo đúng lý lẽ đó: không Tác phẩm nào SỐNG ⇒ không hàng văn bản nào có lý do tồn
        // tại. Chạy VÔ ĐIỀU KIỆN (không chỉ khi `existing` không rỗng) — idempotent, và một
        // lượt gọi hai lần liên tiếp trên gốc vắng mặt không được để lại chữ CŨ trong chỉ mục
        // tìm kiếm dù `library_work` đã trống từ trước.
        self.store.write(move |tx: &Transaction<'_>| {
            tx.execute("DELETE FROM library_segment", [])?;
            tx.execute("INSERT INTO library_target_fts(library_target_fts) VALUES('rebuild')", [])?;
            tx.execute("INSERT INTO library_target_fts_nd(library_target_fts_nd) VALUES('rebuild')", [])?;
            tx.execute("INSERT INTO library_source_fts(library_source_fts) VALUES('rebuild')", [])?;
            Ok(())
        })?;

        // P3 -- cùng lý do nhánh `rebuild` bình thường: chụp TRONG khi khoá còn sống.
        let current_orphans = orphan_store::list(global)?;
        Ok(RebuildOutcome {
            indexed: 0,
            root_missing: true,
            conflicts: Vec::new(),
            skipped: Vec::new(),
            orphans,
            current_orphans,
            text_skipped: Vec::new(),
        })
    }

    /// Đường ĐỌC — mọi hàng của `library_work` khớp [`WorkQuery`], sắp theo khoá sắp của
    /// chính `query` (mặc định `updated_at DESC`, luôn kèm `, work_id` làm khoá phụ ỔN ĐỊNH —
    /// AC4), cộng tổng số hàng CHƯA LỌC và hai tập giá trị lựa chọn CHƯA LỌC (`genres`/
    /// `source_langs`). Story 5.6.
    ///
    /// 🔵 **SỬA (2026-08-27, phán quyết Ice #1) — không còn lọc `WHERE orphaned = 0`.** Từ
    /// khi cờ mồ côi chuyển sang `library_orphan` (`global.db`), MỌI hàng còn lại trong
    /// `library_work` đều đang sống theo định nghĩa — không có gì để lọc nữa vì lý do đó.
    /// `library_work` dẫn xuất TRỌN VẸN trở lại (đúng nghĩa gốc trước Story 5.3).
    ///
    /// 🔵 **SỬA (2026-08-27, Story 5.4) — thêm tham số lọc trạng thái, thêm `WorksReport::total`.**
    /// Không lọc trạng thái ⇒ mọi hàng (kể cả `status IS NULL`), `matched == total` — đúng
    /// §I/O Matrix "Không lọc". Lọc trạng thái **trong SQL** bằng `status IN (...)` (AD-1: bộ
    /// lọc tính ở Rust/SQL, không ở TypeScript) — một hàng `status IS NULL` không bao giờ
    /// khớp bất kỳ giá trị nào trong bộ lọc, đúng ngữ nghĩa SQL của `NULL IN (...)` (luôn
    /// `NULL`/không đúng), nên nó tự động bị loại mà không cần một nhánh `WHERE` riêng.
    ///
    /// 🔵 **SỬA (2026-08-28, Story 5.6) — nhận một [`WorkQuery`] thay vì tham số `filter` rời
    /// rạc; thêm lọc `genre`/`source_lang` (`AND`, §I/O Matrix "Ba bộ lọc chồng") và khoá sắp
    /// (`sort`); thêm `WorksReport::genres`/`source_langs`.** `total`, `works` (đã lọc/sắp)
    /// VÀ hai tập lựa chọn đọc trong **CÙNG một lượt `Store::read`** — cùng một kết nối, cùng
    /// một ảnh chụp — để bốn con số/tập không bao giờ đến từ hai thời điểm khác nhau (đúng lý
    /// lẽ mà [`RebuildOutcome::current_orphans`] đã áp cho cặp `indexed`/`orphans` ở
    /// `rebuild()`). Hai tập lựa chọn đọc trên bảng **CHƯA LỌC** (§Always/AD-1: suy từ `works`
    /// đã lọc phía TypeScript làm lựa chọn TEO DẦN — lọc "Tiên hiệp" xong thì mọi lĩnh vực
    /// khác biến mất khỏi ô chọn, người dùng kẹt không đường quay lại).
    ///
    /// ⚠️ **`WorkQuery::status = Some(vec![])` nghĩa là "KHÔNG giá trị nào khớp", không phải
    /// "không lọc"** — và đó là chỗ tầng này CỐ Ý lệch với
    /// [`crate::commands::library::list_works`], nơi doc-comment nói một bộ lọc rỗng đọc là
    /// *không lọc*. Hai câu trả lời đều đúng ở tầng của nó: ở tầng lệnh một bộ lọc rỗng đến từ
    /// *"người dùng chưa bật nút nào"* (⇒ không lọc, và tầng đó chuẩn hoá bộ lọc rỗng thành
    /// `None` TRƯỚC khi gọi xuống); ở tầng này tham số là một **tập giá trị**, và tập rỗng
    /// khớp 0 hàng theo đúng nghĩa đen. Ghi ra ở đây vì một chỗ gọi Rust tương lai đọc
    /// doc-comment của tầng lệnh rồi gọi thẳng xuống đây sẽ nhận kết quả NGƯỢC với thứ nó
    /// chờ, im lặng. (Lượt rà 2026-08-28.)
    pub fn list_works(&self, query: WorkQuery) -> Result<WorksReport, StoreError> {
        self.store.read(move |conn: ReadHandle<'_>| {
            let total: usize = conn.query_row("SELECT COUNT(*) FROM library_work", [], |row| {
                row.get::<_, i64>(0)
            })? as usize;

            // Hai tập lựa chọn -- luôn trên bảng CHƯA LỌC, luôn trong CÙNG lượt đọc này. Xem
            // khối doc-comment ngay trên cho lý do (AD-1, "lựa chọn teo dần").
            let genres = distinct_column(&conn, "genre")?;
            let source_langs = distinct_column(&conn, "source_lang")?;

            // `Some(vec![])` (§I/O Matrix, ⚠️ ngay trên) ⇒ khớp 0 hàng theo nghĩa đen -- ngắn
            // mạch TRƯỚC khi dựng `WHERE`, nhưng SAU khi `total`/`genres`/`source_langs` đã
            // tính, đúng lời hứa "cùng một lượt đọc" cho MỌI nhánh, kể cả nhánh rỗng.
            if let Some(ref statuses) = query.status {
                if statuses.is_empty() {
                    return Ok(WorksReport { total, works: Vec::new(), genres, source_langs });
                }
            }

            // Dựng `WHERE ... AND ...` từ ba bộ lọc ĐỘC LẬP -- §I/O Matrix "Ba bộ lọc chồng":
            // giao của cả ba, không phải hợp. Mỗi mệnh đề vẫn ràng buộc qua `?` (tham số THẬT),
            // `format!` chỉ dựng số lượng dấu hỏi/tên cột -- không mở lỗ tiêm SQL, cùng lý lẽ
            // đã ghi cho nhánh lọc trạng thái ở bản trước.
            let mut clauses: Vec<String> = Vec::new();
            let mut params: Vec<&str> = Vec::new();

            let status_values: Option<Vec<&str>> =
                query.status.as_ref().map(|v| v.iter().map(|s| s.as_str()).collect());
            if let Some(ref statuses) = status_values {
                let placeholders = statuses.iter().map(|_| "?").collect::<Vec<_>>().join(",");
                clauses.push(format!("status IN ({placeholders})"));
                params.extend(statuses.iter().copied());
            }
            if let Some(ref genre) = query.genre {
                clauses.push("genre = ?".to_owned());
                params.push(genre.as_str());
            }
            if let Some(ref source_lang) = query.source_lang {
                clauses.push("source_lang = ?".to_owned());
                params.push(source_lang.as_str());
            }

            let where_clause =
                if clauses.is_empty() { String::new() } else { format!("WHERE {}", clauses.join(" AND ")) };
            // Khoá phụ `, work_id` LUÔN có mặt -- §I/O Matrix "Hai Tác phẩm cùng updated_at":
            // thứ tự phải ỔN ĐỊNH giữa hai lượt tải khi khoá sắp chính trùng nhau. `work_id` là
            // UUID v4 (AD-28) -- ổn định vì nó KHÔNG đổi theo thời gian, không vì nó "đẹp".
            let order_clause = match query.sort {
                WorkSortKey::UpdatedDesc => "ORDER BY updated_at DESC, work_id",
                WorkSortKey::NameAsc => "ORDER BY name COLLATE NOCASE, work_id",
            };

            let mut stmt = conn.prepare(&format!(
                "SELECT {INDEXED_WORK_COLUMNS} FROM library_work {where_clause} {order_clause}"
            ))?;
            let rows = stmt.query_map(params_from_iter(params.iter()), map_indexed_work_row)?;
            let works: Vec<IndexedWork> = rows.collect::<SqlResult<Vec<_>>>()?;

            Ok(WorksReport { total, works, genres, source_langs })
        })
    }

    /// **THÊM Story 5.7.** Đường ĐỌC **một hàng** theo `work_id` — cho `open_work`
    /// (`commands::project.rs`). `atproj_path` phải phân giải ở TẦNG NÀY, không ở tầng lệnh
    /// (`library_index_boundary.rs` cấm module lệnh mang từ vựng chỉ mục). Dùng lại
    /// [`INDEXED_WORK_COLUMNS`]/[`map_indexed_work_row`] của [`Self::list_works`] — cùng
    /// hình dạng hàng, khác mỗi mệnh đề `WHERE`.
    ///
    /// Trả `Ok(None)` khi không có hàng khớp -- chỗ gọi (`commands::project::wire::open_work`)
    /// ánh xạ `None` sang `library.work_not_indexed`, KHÔNG một biến thể lỗi riêng ở đây: đây
    /// là một truy vấn hợp lệ trả 0 hàng, không phải một điều kiện lỗi của tầng chỉ mục.
    pub fn find_work(&self, work_id: &str) -> Result<Option<IndexedWork>, StoreError> {
        self.store.read(move |conn: ReadHandle<'_>| -> SqlResult<Option<IndexedWork>> {
            // ⚠️ `OptionalExtension::optional()` khong duoc `core::store` tai xuat (xem
            // `commands/segment.rs:306`) -- bat `QueryReturnedNoRows` bang tay.
            match conn.query_row(
                &format!("SELECT {INDEXED_WORK_COLUMNS} FROM library_work WHERE work_id = ?1"),
                [work_id],
                map_indexed_work_row,
            ) {
                Ok(work) => Ok(Some(work)),
                Err(SqlError::QueryReturnedNoRows) => Ok(None),
                Err(err) => Err(err),
            }
        })
    }

    /// **THÊM Story 5.3.** Đường ĐỌC — mọi hàng MỒ CÔI, cho màn hình tối thiểu của story
    /// này. Không lọc/sắp theo tiêu chí nào khác `work_id` — 5.6 sở hữu phần đó.
    ///
    /// 🔵 **SỬA (2026-08-27, phán quyết Ice #1) — đọc từ `library_orphan` (`global.db`), không
    /// còn từ `library_work`.** Không giữ `rebuild_lock`: đây là một lượt đọc-nhất-quán bình
    /// thường của `global.db` (WAL cho một ảnh chụp ổn định), cùng khuôn [`Self::list_works`].
    pub fn list_orphans(&self, global: Option<&Store>) -> Result<Vec<OrphanRecord>, IndexError> {
        let global = global.ok_or_else(|| global_store_missing("Indexer::list_orphans"))?;
        Ok(orphan_store::list(global)?)
    }

    /// **THÊM Story 5.3.** Xoá đúng MỘT hàng khỏi `library_orphan` (`global.db`) — đường XOÁ
    /// tường minh, có tiền điều kiện "hàng tồn tại". KHÔNG chạm đĩa một byte, và KHÔNG chạm
    /// `library_work`/`library-index.db` (§Never của story: "không tự sửa/di chuyển/xoá bất
    /// kỳ `.atproj` nào — `forget_orphan` xoá một hàng chỉ mục").
    ///
    /// Trả danh sách mồ côi CÒN LẠI (§I/O Matrix: "trả danh sách mồ côi còn lại") — chụp
    /// TRONG cùng phạm vi đã khoá bởi `rebuild_lock`, cùng lý do và cùng khuôn [`Self::rebuild`].
    ///
    /// 🔴 **Giữ `rebuild_lock`.** Một lượt `rebuild` chạy chen có thể ghi lại đúng `work_id`
    /// này vào `library_orphan` (ca "mồ côi rồi lại mồ côi theo một đường khác") GIỮA lúc
    /// frontend đọc mục này và lúc lệnh xoá này chạy tới — khoá loại bỏ cửa sổ đó, cùng lý do
    /// đã ghi ở vòng rà THỨ HAI P3 khi cờ mồ côi còn sống ở `library_work`.
    ///
    /// # Lỗi
    /// [`IndexError::NotOrphaned`] khi `work_id` không có mặt trong `library_orphan` — CÙNG
    /// một nhánh từ chối cho cả hai ca của §I/O Matrix ("gỡ nhầm một hàng đang sống" và "gỡ
    /// một `work_id` không có"): một hàng ĐANG SỐNG không nằm trong bảng này nữa (nó chỉ có
    /// trong `library_work`), nên cả hai ca đều là "không tìm thấy trong `library_orphan`" —
    /// không im lặng thành công và không mập mờ giữa hai lý do.
    pub fn forget_orphan(&self, work_id: &str, global: Option<&Store>) -> Result<Vec<OrphanRecord>, IndexError> {
        // 🔵 THÊM (2026-08-27, vòng rà THỨ HAI P3) — cùng khoá với `rebuild`, xem lý do ở
        // khối 🔴 ngay trên.
        let _guard = self
            .rebuild_lock
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);

        let global = global.ok_or_else(|| global_store_missing("Indexer::forget_orphan"))?;

        let deleted = orphan_store::forget(global, work_id)?;
        if deleted == 0 {
            return Err(IndexError::NotOrphaned {
                work_id: work_id.to_owned(),
            });
        }

        Ok(orphan_store::list(global)?)
    }

    /// **THÊM Story 5.9, mở rộng Story 5.10 (FR9 — hai chế độ dấu).** Tìm kiếm full-text xuyên
    /// TOÀN BỘ Library — chạy CẢ HAI chỉ mục CHÍNH (`library_target_fts` nửa bản dịch,
    /// `library_source_fts` nửa nguyên văn) MỖI LƯỢT gọi và HỢP kết quả (§Always Story 5.9:
    /// *"một bộ điều phối chọn một nhánh sẽ trả 0 hàng trên một kho CÓ dữ liệu khớp"*, đúng lớp
    /// lỗi AD-44 đã ghi ở đường từ điển). Đọc THUẦN khỏi `library-index.db` — không một truy
    /// vấn nào chạm `.atproj`/`meta.json` (§Never).
    ///
    /// Truy vấn dưới [`MIN_SUBSTRING_QUERY_CHARS`] ký tự ⇒ nhánh nguyên văn (`trigram`, sàn
    /// CỨNG của tokenizer — đo 2026-08-29, SQLite 3.53.2 nhúng) KHÔNG chạy —
    /// [`SearchReport::short_query`] báo ra một trạng thái CÓ TÊN, không phải "không có kết
    /// quả" (§Always). Nhánh bản dịch (`unicode61`) không có sàn đó và VẪN chạy, khớp TRỌN TỪ.
    ///
    /// `limit` áp cho MỖI nhánh RIÊNG — hai chỉ mục trả lời hai câu hỏi khác nhau (nửa nguồn,
    /// nửa dịch), nên một Tác phẩm khớp cả hai không "cướp" hạn mức của nhánh kia.
    ///
    /// ─────────────────────────────────────────────────────────────────────────────
    /// 🔴 STORY 5.10 — HAI CHẾ ĐỘ DẤU (FR9), VÀ VÌ SAO CHÍNH XÁC LUÔN CHẠY TRƯỚC
    /// ─────────────────────────────────────────────────────────────────────────────
    /// `mode` quyết định chỉ mục CHÍNH có đủ hay cần thêm `library_target_fts_nd`
    /// (`unicode61 remove_diacritics 2`) — chỉ mục KHOAN DUNG DẤU chỉ tồn tại ở NỬA BẢN DỊCH
    /// (xem `5-10-hai-che-do-dau.md` §Design Notes "Vì sao nửa nguyên văn không có bản khoan
    /// dung"). Lượt CHÍNH XÁC (target + source, đúng khuôn Story 5.9) LUÔN chạy TRƯỚC, không
    /// ngoại lệ — kể cả khi `mode == Lenient`: đây là bằng chứng cho tập rowid dùng để dán nhãn
    /// [`SearchHit::match_kind`] (xem dưới), và nửa nguyên văn (`source`) không có nhánh `_nd`
    /// nên nó GIỮ NGUYÊN, phân biệt dấu, ở CẢ HAI chế độ — chuyển chế độ không được làm MẤT một
    /// hit đã có (§Always).
    ///
    /// `effective_mode` là [`SearchMode::Lenient`] khi và chỉ khi:
    /// - `mode == Lenient` (người dùng chọn tường minh bằng nút), HOẶC
    /// - `mode == Exact` **và** lượt chính xác trả 0 hàng (`target` VÀ `source` đều rỗng)
    ///   **và** `indexed_segments > 0` — tự NỚI, `widened = true`. Nới trên một chỉ mục RỖNG
    ///   sẽ khai *"đã nới sang khoan dung"* cho một kho chưa có dòng nào — một câu đúng hình
    ///   dạng, sai sự thật (§Always) — nên điều kiện `indexed_segments > 0` là bắt buộc.
    ///
    /// `widened == (mode == Exact && effective_mode == Lenient)` là một BẤT BIẾN của hàm này,
    /// đúng theo cấu tạo (không một nhánh nào gán `widened` ngoài định nghĩa `widened` ở trên).
    ///
    /// Khi `effective_mode == Lenient`, nửa BẢN DỊCH đổi nguồn: thay vì đọc `target` (chỉ mục
    /// CHÍNH), nó đọc `library_target_fts_nd` — vì `_nd` gấp dấu trên CẢ NỘI DUNG lẫn TRUY VẤN
    /// nên tập kết quả của nó là TẬP CHA của tập `target` (mọi hit chính xác cũng khớp `_nd`,
    /// đo 2026-08-29). ⇒ Không cộng gộp `target + nd_target` (sẽ đúp mọi hit chính xác); output
    /// nửa bản dịch ở chế độ khoan dung là ĐÚNG MỘT truy vấn trên `_nd`, dán nhãn theo tập
    /// rowid của `target` vừa lấy được ở lượt chính xác (§Always: *"nhãn `match_kind` phải đến
    /// từ một phép đo CÙNG VỊ TỪ, không từ một phép so chuỗi thứ hai"*) — rowid nằm trong tập
    /// đó ⇒ `Exact`, ngoài ⇒ `Lenient`. Ở lượt TỰ NỚI, `target` vừa đo được RỖNG (đó là lý do
    /// nới), nên tập rowid rỗng và mọi hit `_nd` là `Lenient` theo cấu tạo — không cần một phép
    /// so thứ hai.
    pub fn search(&self, query: &str, limit: usize, mode: SearchMode) -> Result<SearchReport, StoreError> {
        let limit = limit.clamp(1, MAX_SEARCH_LIMIT);
        let trimmed = query.trim().to_owned();
        let short_query = trimmed.chars().count() < MIN_SUBSTRING_QUERY_CHARS;

        self.store.read(move |conn: ReadHandle<'_>| -> SqlResult<SearchReport> {
            // Quần thể THẬT, KHÔNG phụ thuộc truy vấn — đây là con số phân biệt "chỉ mục
            // chưa có dòng nào" với "có dòng mà không khớp" (§I/O Matrix, §Always).
            let indexed_segments: i64 =
                conn.query_row("SELECT COUNT(*) FROM library_segment", [], |row| row.get(0))?;
            let indexed_segments = indexed_segments.max(0) as usize;

            if trimmed.is_empty() {
                // Chỗ gọi (`commands::library::search_library`) không nên gửi một truy vấn
                // rỗng xuống đây (§I/O Matrix: "0 lượt IPC" ở tầng frontend) — nhưng đây vẫn
                // là ranh giới AN TOÀN cuối cùng: một cụm rỗng bọc ngoặc kép (`""`) là một
                // truy vấn FTS5 HỢP LỆ khớp MỌI hàng, đúng thứ không ai muốn ở đây.
                return Ok(SearchReport {
                    hits: Vec::new(),
                    total: 0,
                    indexed_segments,
                    short_query: true,
                    truncated: false,
                    mode,
                    effective_mode: mode,
                    widened: false,
                });
            }

            // ── Lượt CHÍNH XÁC — LUÔN chạy TRƯỚC, mỗi lượt, không ngoại lệ (AD-27 · AC1). ──
            // Lấy `limit + 1` ở MỖI nhánh: hàng thứ `limit + 1` không bao giờ hiển thị, nó chỉ
            // là BẰNG CHỨNG rằng còn nữa. Xem [`SearchReport::truncated`].
            let mut target = search_target_text(&conn, &trimmed, limit + 1)?;
            let target_truncated = target.len() > limit;
            target.truncate(limit);

            let mut source = Vec::new();
            let mut source_truncated = false;
            if !short_query {
                source = search_source_text(&conn, &trimmed, limit + 1)?;
                source_truncated = source.len() > limit;
                source.truncate(limit);
            }

            let exact_is_empty = target.is_empty() && source.is_empty();
            let widened = mode == SearchMode::Exact && exact_is_empty && indexed_segments > 0;
            let effective_mode = if mode == SearchMode::Lenient || widened {
                SearchMode::Lenient
            } else {
                SearchMode::Exact
            };

            if effective_mode == SearchMode::Exact {
                let mut hits = target;
                hits.extend(source);
                let total = hits.len();
                return Ok(SearchReport {
                    hits,
                    total,
                    indexed_segments,
                    short_query,
                    truncated: target_truncated || source_truncated,
                    mode,
                    effective_mode,
                    widened,
                });
            }

            // ── Lượt KHOAN DUNG — chỉ nửa BẢN DỊCH đổi nguồn sang `_nd`; `source` (nguyên
            // văn) ở trên GIỮ NGUYÊN, phân biệt dấu (§Always: "chuyển chế độ không được làm
            // MẤT kết quả"). Xem khối doc-comment ở trên cho lý lẽ dán nhãn theo tập rowid.
            let exact_target_rowids: std::collections::HashSet<i64> =
                target.iter().map(|hit| hit.rowid).collect();
            let mut nd_target = search_target_text_nd(&conn, &trimmed, limit + 1)?;
            let nd_truncated = nd_target.len() > limit;
            nd_target.truncate(limit);
            for hit in &mut nd_target {
                hit.match_kind = if exact_target_rowids.contains(&hit.rowid) {
                    MatchKind::Exact
                } else {
                    MatchKind::Lenient
                };
            }

            let mut hits = nd_target;
            hits.extend(source);
            let total = hits.len();
            Ok(SearchReport {
                hits,
                total,
                indexed_segments,
                short_query,
                truncated: nd_truncated || source_truncated,
                mode,
                effective_mode,
                widened,
            })
        })
    }
}

/// **THÊM Story 5.6.** `SELECT DISTINCT <column> FROM library_work`, sắp theo chính giá trị
/// đó (không dấu phân biệt hoa/thường) để tập lựa chọn hiện ra ổn định giữa hai lượt gọi.
///
/// ⚠️ `column` **không phải** dữ liệu người dùng — hai chỗ gọi DUY NHẤT truyền literal
/// `"genre"`/`"source_lang"` (xem [`Indexer::list_works`]), nên `format!` ở đây dựng TÊN CỘT
/// từ mã nguồn, không từ dây IPC — không mở lỗ tiêm SQL, cùng lý lẽ đã ghi cho `COLUMNS`/
/// `placeholders` ở `list_works`.
fn distinct_column(conn: &ReadHandle<'_>, column: &str) -> SqlResult<Vec<String>> {
    let mut stmt =
        conn.prepare(&format!("SELECT DISTINCT {column} FROM library_work ORDER BY {column} COLLATE NOCASE"))?;
    let rows = stmt.query_map([], |row| row.get::<_, String>(0))?;
    rows.collect::<SqlResult<Vec<_>>>()
}

// ═════════════════════════════════════════════════════════════════════════════════
// Story 5.9 — TÌM KIẾM FULL-TEXT (FR8). Xem [`Indexer::search`] cho hợp đồng đầy đủ.
// ═════════════════════════════════════════════════════════════════════════════════

/// Sàn CỨNG của tokenizer `trigram` — dưới 3 ký tự nó không lập chỉ mục được token nào (đo
/// 2026-08-29, SQLite 3.53.2 nhúng — 🔵 sửa tại chỗ 2026-08-29, Story 5.10, xem doc-comment
/// `LIBRARY_WORK_DDL` ở `core/store/schema.rs`: `"天下"` trả **0** hàng trên chính văn bản
/// chứa nó). Danh mục ĐÓNG một hằng số, không một "gần đúng" viết tay ở chỗ gọi.
pub const MIN_SUBSTRING_QUERY_CHARS: usize = 3;

/// Cỡ trang MẶC ĐỊNH của một lượt [`Indexer::search`] khi chỗ gọi không truyền `limit` —
/// dùng ở tầng lệnh (`commands::library::search_library`).
pub const DEFAULT_SEARCH_LIMIT: usize = 50;

/// Trần TRÊN của `limit` — cùng lý lẽ `candidate_ceiling` của `core/dict/query.rs`: không để
/// một chỗ gọi (hoặc một tham số IPC bất thường) kéo cả `library_segment` vào bộ nhớ.
const MAX_SEARCH_LIMIT: usize = 500;

/// Hệ số AN TOÀN cho tập ứng viên `trigram` TRƯỚC khi xác minh — cùng lý lẽ
/// `core/dict/query.rs::candidate_ceiling`: cắt ứng viên TRƯỚC khi xác minh làm số hàng cuối
/// cùng ít hơn `limit` thật, và một dòng "còn nữa" sẽ nói dối (Bẫy 11 của đường từ điển). 50
/// là hệ số đã đo đủ rộng cho tỉ lệ dương tính giả tệ nhất từng thấy ở một chỉ mục trigram
/// trong kho này (`中國` ⇒ 390 ứng viên, 40 sai ≈ 10,3%, `core/dict/query.rs`).
fn search_candidate_ceiling(limit: usize) -> i64 {
    const SAFETY_FACTOR: usize = 50;
    i64::try_from(limit.saturating_mul(SAFETY_FACTOR)).unwrap_or(i64::MAX)
}

/// 🔴 **Khuôn bọc cụm FTS5 — CHÉP NGUYÊN cách của `core/dict/query.rs:310`, không một "biến
/// thể" thứ hai.** Truy vấn người dùng đi vào FTS5 dạng CỤM có ngoặc kép, dấu `"` bên trong
/// NHÂN ĐÔI trước khi bọc (cách thoát của FTS5). Không bọc thì `*` `-` `^` `(` `:` hay từ
/// `NEAR` làm SQLite trả `SQLITE_ERROR` — tra cứu báo lỗi vì CHÍNH chữ người dùng gõ. Dùng ở
/// CẢ HAI nhánh (`search_target_text`/`search_source_text`), không có bản thứ hai.
fn fts_phrase(query: &str) -> String {
    format!("\"{}\"", query.replace('"', "\"\""))
}

/// Nửa nào của một segment một hit khớp — danh mục ĐÓNG, hai giá trị. Story 5.9.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SearchField {
    /// Khớp ở `target_text` — nửa BẢN DỊCH (`library_target_fts`, `unicode61
    /// remove_diacritics 0`, phân biệt dấu, khớp TRỌN TỪ).
    Target,
    /// Khớp ở `source_text` — nửa NGUYÊN VĂN (`library_source_fts`, `trigram`, đã xác minh
    /// chuỗi con ở Rust).
    Source,
}

impl SearchField {
    /// Định danh máy đọc — thứ đi trên dây. Không phải nhãn hiển thị (AD-21).
    pub const fn as_str(self) -> &'static str {
        match self {
            SearchField::Target => "target",
            SearchField::Source => "source",
        }
    }
}

/// Chế độ một lượt [`Indexer::search`] — danh mục ĐÓNG, hai giá trị. Story 5.10 (FR9). Khuôn
/// TRỰC TIẾP của [`WorkSortKey`]: `as_str`/`from_wire`/`Default`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SearchMode {
    /// Chỉ chạy hai chỉ mục CHÍNH (`library_target_fts`/`library_source_fts`), PHÂN BIỆT dấu
    /// tiếng Việt ở cả hai (AD-27). **Mặc định** — khoan dung KHÔNG BAO GIỜ là mặc định.
    Exact,
    /// Nửa BẢN DỊCH đọc thêm `library_target_fts_nd` (`unicode61 remove_diacritics 2`) — do
    /// người dùng chọn tường minh, HOẶC do một lượt tự nới (xem [`SearchReport::widened`]).
    /// Nửa NGUYÊN VĂN không đổi — nó không có chỉ mục `_nd` (§Design Notes của
    /// `5-10-hai-che-do-dau.md`, "Vì sao nửa nguyên văn không có bản khoan dung").
    Lenient,
}

impl SearchMode {
    /// **THÊM (vòng rà bốn lớp, mục 9)** — danh mục ĐÓNG, cùng khuôn [`WorkSortKey::ALL`]: chỗ
    /// kiểm khứ hồi (`as_str` ⇄ `from_wire`) chạy TRÊN hằng số này, không viết tay một danh
    /// sách song song sẽ trôi khỏi enum thật.
    pub const ALL: &'static [SearchMode] = &[SearchMode::Exact, SearchMode::Lenient];

    /// Định danh máy đọc — thứ đi trên dây. Không phải nhãn hiển thị (AD-21).
    pub const fn as_str(self) -> &'static str {
        match self {
            SearchMode::Exact => "exact",
            SearchMode::Lenient => "lenient",
        }
    }

    /// Phân giải một giá trị đến từ dây IPC. `None` ⇒ giá trị ngoài danh mục đóng, chỗ gọi tự
    /// dựng lỗi (không đoán, không rơi về mặc định) — cùng khuôn [`WorkSortKey::from_wire`].
    pub fn from_wire(raw: &str) -> Option<SearchMode> {
        match raw {
            "exact" => Some(SearchMode::Exact),
            "lenient" => Some(SearchMode::Lenient),
            _ => None,
        }
    }
}

impl Default for SearchMode {
    /// §Always: "khoan dung KHÔNG BAO GIỜ là mặc định" (AD-27 · AC4) — `mode = None` trên dây
    /// ⇒ `Exact`.
    fn default() -> Self {
        SearchMode::Exact
    }
}

/// Vị từ nào đã tìm ra một hit — danh mục ĐÓNG, hai giá trị. Story 5.10.
///
/// 🔴 **Nhãn này đến từ một PHÉP ĐO CÙNG VỊ TỪ (tập rowid của nhánh chính xác), không từ một
/// phép so chuỗi thứ hai** — xem doc-comment [`Indexer::search`] cho lý lẽ đầy đủ: một
/// `raw.contains(query)` nói KHÁC vị từ `unicode61` (khớp TRỌN TỪ) đang dùng để tìm.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MatchKind {
    /// Hit khớp cả ở chỉ mục CHÍNH (`library_target_fts`, phân biệt dấu).
    Exact,
    /// Hit CHỈ khớp ở chỉ mục KHOAN DUNG (`library_target_fts_nd`) — rowid của nó KHÔNG nằm
    /// trong tập rowid mà chỉ mục chính vừa trả về ở CÙNG lượt gọi.
    Lenient,
}

impl MatchKind {
    /// **THÊM (vòng rà bốn lớp, mục 9)** — danh mục ĐÓNG, cùng khuôn [`SearchMode::ALL`].
    /// `MatchKind` không có `from_wire` (nó chỉ đi RA dây, không bao giờ được PHÂN GIẢI TỪ dây
    /// — client không gửi `match_kind`), nên hằng số này phục vụ ca kiểm "hai biến thể mang
    /// hai chuỗi PHÂN BIỆT nhau, không đứa nào rỗng" thay vì một phép khứ hồi `from_wire`.
    pub const ALL: &'static [MatchKind] = &[MatchKind::Exact, MatchKind::Lenient];

    /// Định danh máy đọc — thứ đi trên dây. Không phải nhãn hiển thị (AD-21).
    pub const fn as_str(self) -> &'static str {
        match self {
            MatchKind::Exact => "exact",
            MatchKind::Lenient => "lenient",
        }
    }
}

/// Một kết quả tìm kiếm — một hàng của `library_segment` khớp truy vấn ở MỘT nửa (`field` nói
/// nửa nào). Story 5.9, mở rộng Story 5.10.
#[derive(Debug, Clone)]
pub struct SearchHit {
    pub work_id: String,
    pub work_name: String,
    pub chapter_id: i64,
    pub chapter_ord: i64,
    pub chapter_title: Option<String>,
    /// `None` ⇒ hit CẤP CHƯƠNG — Chương chưa tách segment sống nào (xem
    /// [`harvest_work_text`]); lượt mở kết quả để Rust quyết con trỏ (`open_chapter`), không
    /// đi qua đường dời con trỏ theo segment.
    pub segment_id: Option<i64>,
    pub field: SearchField,
    /// Đoạn trích văn bản THUẦN, cặp dấu `‹…›` bao quanh phần khớp — KHÔNG một thẻ HTML nào
    /// (AD-16, §Always: *"`snippet()` dùng cặp dấu văn bản thuần"*). Render bằng nội suy Vue
    /// thường (`{{ }}`), không bao giờ `v-html`.
    pub snippet: String,
    /// **THÊM Story 5.10.** Vị từ nào đã tìm ra hit này — `Exact` cho mọi hit của lượt chính
    /// xác (kể cả nửa nguyên văn, luôn `Exact`); phân biệt `Exact`/`Lenient` chỉ có ý nghĩa cho
    /// nửa bản dịch ở [`SearchMode::Lenient`]. Đi lên dây thành `match_kind` (`commands::library::SearchHit`).
    pub match_kind: MatchKind,
    /// **THÊM Story 5.10.** `rowid` của `library_segment` — dùng NỘI BỘ để dán nhãn
    /// [`Self::match_kind`] theo tập thành viên (§Always). 🔴 KHÔNG LÊN DÂY: `From<SearchHit>`
    /// ở `commands::library` không chép trường này — nó là chi tiết TRIỂN KHAI của lõi, không
    /// phải hợp đồng IPC.
    pub rowid: i64,
}

/// Kết quả một lượt [`Indexer::search`] — Story 5.9, mở rộng Story 5.10. Ba trường đầu ngoài
/// `hits` cho tầng trên (`commands::library::search_library`, rồi `LibraryMode.vue`) đủ dữ
/// kiện phân biệt NĂM ca rỗng của §I/O Matrix Story 5.9: `indexed_segments == 0` ⇒ chỉ mục
/// chưa có gì; `short_query` ⇒ dưới sàn trigram; `hits` rỗng mà cả hai trường kia không rơi vào
/// hai ca trên ⇒ không khớp thật. Ba trường cuối (`mode`/`effective_mode`/`widened`) là của
/// Story 5.10 — xem doc-comment [`Indexer::search`] cho hợp đồng đầy đủ giữa chúng.
#[derive(Debug, Clone)]
pub struct SearchReport {
    pub hits: Vec<SearchHit>,
    /// `hits.len()` — trường TƯỜNG MINH, không suy từ `.length` phía TypeScript (AD-1, cùng lý
    /// lẽ `WorksReport`/`WorkListReport`).
    pub total: usize,
    /// Tổng số hàng CÓ THẬT trong `library_segment`, KHÔNG phụ thuộc truy vấn.
    pub indexed_segments: usize,
    /// `true` ⇔ truy vấn dưới [`MIN_SUBSTRING_QUERY_CHARS`] ký tự — nửa nguyên văn (trigram)
    /// KHÔNG chạy ở lượt này.
    pub short_query: bool,
    /// 🔴 `true` ⇔ **ít nhất một nhánh đã chạm trần `limit` và danh sách bị CẮT** — tức
    /// [`Self::total`] là *"số hàng ĐANG HIỆN"*, **không** phải *"số hàng khớp"*.
    ///
    /// ⚠️ Trường này ra đời từ một lượt rà: bản đầu không có nó, và giao diện đọc `total`
    /// thành *"{total} kết quả"* — một câu khẳng định DỨT KHOÁT trên một con số đã bị trần cắt.
    /// Một từ thường gặp trong một thư viện thật khớp hàng nghìn hàng và màn hình nói *"100
    /// kết quả"*, không một dấu hiệu nào cho biết còn nữa. Đó là đúng hình dạng *"một câu trả
    /// lời đúng về hình dạng nhưng sai về sự thật"* mà `AGENTS.md::Known pitfalls` gọi tên, và
    /// nó vi phạm chính luật *"không trần nào được cắt trong im lặng"* của story này.
    ///
    /// Phát hiện bằng cách lấy `limit + 1` hàng rồi cắt lại còn `limit` — không một truy vấn
    /// `COUNT(*)` thứ hai trên cùng một `MATCH` (đắt gần bằng chính lượt tìm).
    pub truncated: bool,
    /// **THÊM Story 5.10.** Chế độ NGƯỜI DÙNG (hoặc chỗ gọi) yêu cầu — chép nguyên tham số đầu
    /// vào [`Indexer::search`].
    pub mode: SearchMode,
    /// **THÊM Story 5.10.** Chế độ THỰC SỰ đã chạy — có thể khác `mode` khi một lượt TỰ NỚI
    /// xảy ra (xem [`Self::widened`]).
    pub effective_mode: SearchMode,
    /// **THÊM Story 5.10.** `true` ⇔ đây là một lượt TỰ NỚI: `mode == Exact` nhưng lượt chính
    /// xác trả 0 hàng trên một chỉ mục KHÔNG rỗng, nên hệ thống tự chạy thêm `_nd`. Bất biến:
    /// `widened == (mode == Exact && effective_mode == Lenient)`.
    pub widened: bool,
}

/// **Nhánh bản dịch** — `library_target_fts` (`unicode61 remove_diacritics 0`), khớp TRỌN TỪ,
/// PHÂN BIỆT dấu. Không bước xác minh: `unicode61` là tokenizer theo TỪ, một kết quả MATCH đã
/// là một khớp thật (khác `trigram`, nơi MATCH chỉ nói "chứa các trigram này").
fn search_target_text(conn: &ReadHandle<'_>, query: &str, limit: usize) -> SqlResult<Vec<SearchHit>> {
    let phrase = fts_phrase(query);
    let limit_i64 = i64::try_from(limit).unwrap_or(i64::MAX);
    let sql = "\
        SELECT s.rowid, s.work_id, w.name, s.chapter_id, s.chapter_ord, s.chapter_title, s.segment_id, \
               snippet(library_target_fts, 0, '\u{2039}', '\u{203a}', '\u{2026}', 10) \
        FROM library_target_fts f \
        JOIN library_segment s ON s.rowid = f.rowid \
        JOIN library_work w ON w.work_id = s.work_id \
        WHERE library_target_fts MATCH ?1 \
        ORDER BY s.work_id, s.chapter_ord, s.segment_ord \
        LIMIT ?2";
    let mut stmt = conn.prepare_cached(sql)?;
    let rows = stmt.query_map((&phrase, limit_i64), |row| {
        Ok(SearchHit {
            rowid: row.get(0)?,
            work_id: row.get(1)?,
            work_name: row.get(2)?,
            chapter_id: row.get(3)?,
            chapter_ord: row.get(4)?,
            chapter_title: row.get(5)?,
            segment_id: row.get(6)?,
            field: SearchField::Target,
            // Nhánh CHÍNH XÁC -- mọi hit của nó là `Exact` theo cấu tạo (Story 5.10).
            match_kind: MatchKind::Exact,
            snippet: row.get(7)?,
        })
    })?;
    rows.collect()
}

/// **Nhánh khoan dung của nửa bản dịch** — Story 5.10 (FR9). Khuôn TRỰC TIẾP của
/// [`search_target_text`] ngay trên, chỉ đổi tên bảng FTS sang `library_target_fts_nd`
/// (`unicode61 remove_diacritics 2`) và đọc THÊM `s.rowid` (đã có sẵn ở bản khuôn, không phải
/// một cột mới). `match_kind` gán TẠM `Lenient` ở đây — [`Indexer::search`] sửa lại theo tập
/// rowid của lượt chính xác TRƯỚC khi trả ra (xem doc-comment của nó), không phải một hằng số
/// cuối cùng.
fn search_target_text_nd(conn: &ReadHandle<'_>, query: &str, limit: usize) -> SqlResult<Vec<SearchHit>> {
    let phrase = fts_phrase(query);
    let limit_i64 = i64::try_from(limit).unwrap_or(i64::MAX);
    let sql = "\
        SELECT s.rowid, s.work_id, w.name, s.chapter_id, s.chapter_ord, s.chapter_title, s.segment_id, \
               snippet(library_target_fts_nd, 0, '\u{2039}', '\u{203a}', '\u{2026}', 10) \
        FROM library_target_fts_nd f \
        JOIN library_segment s ON s.rowid = f.rowid \
        JOIN library_work w ON w.work_id = s.work_id \
        WHERE library_target_fts_nd MATCH ?1 \
        ORDER BY s.work_id, s.chapter_ord, s.segment_ord \
        LIMIT ?2";
    let mut stmt = conn.prepare_cached(sql)?;
    let rows = stmt.query_map((&phrase, limit_i64), |row| {
        Ok(SearchHit {
            rowid: row.get(0)?,
            work_id: row.get(1)?,
            work_name: row.get(2)?,
            chapter_id: row.get(3)?,
            chapter_ord: row.get(4)?,
            chapter_title: row.get(5)?,
            segment_id: row.get(6)?,
            field: SearchField::Target,
            match_kind: MatchKind::Lenient,
            snippet: row.get(7)?,
        })
    })?;
    rows.collect()
}

/// **Nhánh nguyên văn** — `library_source_fts` (`trigram`), chuỗi CON thật, phủ được chữ Hán.
///
/// 🔴 **PHẢI qua bước xác minh chuỗi con Ở RUST** (§Always) — FTS5 trigram trả lời *"chứa các
/// trigram này"*, không trả lời *"chứa chuỗi này"* (`core/dict/query.rs:392-394` ghi cùng bài
/// học ở đường từ điển; `verify_substring` của tệp đó là khuôn). `to_lowercase()` CẢ HAI vế —
/// trigram KHÔNG phân biệt hoa/thường lúc tìm ứng viên (đo 2026-08-29: `"BROWN"` khớp
/// `the quick brown fox`), nên xác minh phân biệt hoa/thường sẽ ÂM THẦM LOẠI đúng hàng vừa
/// tìm được — một hàng rào chống dương-tính-giả biến thành một cỗ máy sinh âm-tính-giả.
///
/// Ứng viên lấy tới [`search_candidate_ceiling`] (KHÔNG `LIMIT limit` ở SQL — cùng Bẫy 11 của
/// `core/dict/query.rs`: cắt trước khi xác minh cho ra ít hơn `limit` mục thật), xác minh, RỒI
/// mới cắt còn `limit` ở Rust.
///
/// 🔴 **`max_tokens` của `snippet()` là 64 ở nhánh này chứ KHÔNG phải 10 như nhánh bản dịch, và
/// con số đó bị ÉP bởi tokenizer chứ không phải một sở thích.** Một "token" của `trigram` là
/// một cụm BA KÝ TỰ trượt từng ký tự một, nên 10 token ≈ 12 ký tự. Đo 2026-08-29 (SQLite
/// 3.53.2 nhúng — 🔵 sửa tại chỗ 2026-08-29, Story 5.10) trên một câu dài mang từ khoá
/// `zzqqmarker` ở giữa:
/// - `trigram`, `max_tokens = 10` ⇒ `"… ‹zzqqmarker› …"` — **không một chữ ngữ cảnh nào**;
/// - `trigram`, `max_tokens = 64` ⇒ `"…, roi toi doan chua tu khoa ‹zzqqmarker› o giua, va sau do con rat n…"`;
/// - `unicode61`, `max_tokens = 10` ⇒ `"…doan chua tu khoa ‹zzqqmarker› o giua, va sau do…"` — đủ.
///
/// ⇒ AC của story đòi kết quả kèm **đoạn văn bản khớp**; một đoạn chỉ chứa đúng từ khoá không
/// phải một đoạn văn bản. Hai nhánh vì thế mang HAI con số, và đừng "đồng bộ" chúng.
fn search_source_text(conn: &ReadHandle<'_>, query: &str, limit: usize) -> SqlResult<Vec<SearchHit>> {
    let phrase = fts_phrase(query);
    let ceiling = search_candidate_ceiling(limit);
    let sql = "\
        SELECT s.rowid, s.work_id, w.name, s.chapter_id, s.chapter_ord, s.chapter_title, s.segment_id, \
               s.source_text, \
               snippet(library_source_fts, 0, '\u{2039}', '\u{203a}', '\u{2026}', 64) \
        FROM library_source_fts f \
        JOIN library_segment s ON s.rowid = f.rowid \
        JOIN library_work w ON w.work_id = s.work_id \
        WHERE library_source_fts MATCH ?1 \
        ORDER BY s.work_id, s.chapter_ord, s.segment_ord \
        LIMIT ?2";
    let mut stmt = conn.prepare_cached(sql)?;
    let rows = stmt.query_map((&phrase, ceiling), |row| {
        let hit = SearchHit {
            rowid: row.get(0)?,
            work_id: row.get(1)?,
            work_name: row.get(2)?,
            chapter_id: row.get(3)?,
            chapter_ord: row.get(4)?,
            chapter_title: row.get(5)?,
            segment_id: row.get(6)?,
            field: SearchField::Source,
            // Nửa NGUYÊN VĂN không có nhánh `_nd` (§Design Notes của story) -- luôn `Exact`,
            // kể cả trong một lượt khoan dung (§Always: "chuyển chế độ không làm mất kết quả").
            match_kind: MatchKind::Exact,
            snippet: row.get(8)?,
        };
        let source_text: String = row.get(7)?;
        Ok((hit, source_text))
    })?;
    let candidates: Vec<(SearchHit, String)> = rows.collect::<SqlResult<Vec<_>>>()?;

    let needle = query.to_lowercase();
    let mut verified: Vec<SearchHit> = candidates
        .into_iter()
        .filter(|(_, source_text)| source_text.to_lowercase().contains(&needle))
        .map(|(hit, _)| hit)
        .collect();
    verified.truncate(limit);
    Ok(verified)
}

// ═════════════════════════════════════════════════════════════════════════════════
// Story 5.9 — THU HOẠCH VĂN BẢN. Xem [`harvest_work_text`] cho hợp đồng đầy đủ.
// ═════════════════════════════════════════════════════════════════════════════════

/// Tên tệp `project.db` bên trong một thư mục `.atproj` — cùng literal đã dùng ở
/// `commands/project.rs` (`dir.join("project.db")`); chưa đủ chỗ dùng xuyên module để đáng một
/// hằng `pub(crate)` dùng chung.
const PROJECT_DB_FILE: &str = "project.db";

/// Một hàng văn bản thu hoạch được từ MỘT `project.db` — cấp SEGMENT (`segment_id = Some`) hoặc
/// cấp CHƯƠNG khi Chương đó chưa có segment SỐNG nào (`segment_id = None`, dùng
/// `chapter.source_text`; `target_text` rỗng vì bảng `chapter` không có cột bản dịch — AD-32
/// giữ nguyên ranh giới đó).
struct HarvestedRow {
    chapter_id: i64,
    chapter_ord: i64,
    chapter_title: Option<String>,
    segment_id: Option<i64>,
    segment_ord: i64,
    source_text: String,
    target_text: String,
}

/// Story 5.9 — thu hoạch TOÀN BỘ văn bản (nguyên văn + bản dịch) của MỘT Tác phẩm, đọc CHỈ ĐỌC
/// qua [`ReadOnlyDb`] (`StoreKind::Project`, miễn trừ CÓ TÊN ở `readonly.rs`) — KHÔNG BAO GIỜ
/// qua [`Store::open`] (§Always của story: bốn thứ `Store::open` ghi vào tệp, kể cả chạy bộ di
/// trú, và một lượt quét không sở hữu Tác phẩm này).
///
/// `Err(String)` — chẩn đoán KHÔNG DẤU nêu ĐÍCH DANH lý do (`project.db` vắng mặt / phiên bản
/// lược đồ mới hơn ứng dụng / mở-đọc thất bại) — chỗ gọi ([`Indexer::rebuild`]) đếm nó vào
/// [`RebuildOutcome::text_skipped`] cùng `work_id`, KHÔNG làm trượt cả lượt `rebuild`.
///
/// 🔴 **Kiểm phiên bản lược đồ TRƯỚC khi mở đọc thật** (AD-30): một `project.db` ở phiên bản
/// MỚI HƠN ứng dụng hiểu có thể mang một hình dạng `chapter`/`segment` mà hai câu SQL dưới đây
/// KHÔNG biết — đọc mù vào đó là đọc SAI CỘT một cách im lặng, không phải một lỗi ồn ào. Bỏ qua
/// phần văn bản của đúng Tác phẩm này an toàn hơn.
fn harvest_work_text(dir: &Path) -> Result<Vec<HarvestedRow>, String> {
    let project_db_path = dir.join(PROJECT_DB_FILE);

    let found = match crate::core::store::peek_schema_version(&project_db_path, StoreKind::Project) {
        Ok(None) => return Err("project.db vang mat".to_owned()),
        Ok(Some(found)) => found,
        Err(err) => return Err(format!("khong doc duoc phien ban luoc do cua project.db: {err}")),
    };
    let target = crate::core::store::schema::target_version(PROJECT_MIGRATIONS);
    if found > target {
        return Err(format!(
            "project.db o schema version {found}, ung dung chi hieu toi {target} -- AD-30 cam \
             doc sau vao mot luoc do chua biet"
        ));
    }

    let db = ReadOnlyDb::open(project_db_path, StoreKind::Project)
        .map_err(|err| format!("mo project.db chi doc that bai: {err}"))?;

    let result = db.read(|conn: ReadHandle<'_>| -> SqlResult<Vec<HarvestedRow>> {
        let mut chapters_stmt =
            conn.prepare("SELECT id, ord, title, source_text FROM chapter ORDER BY ord")?;
        let chapters: Vec<(i64, i64, Option<String>, String)> = chapters_stmt
            .query_map([], |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)))?
            .collect::<SqlResult<Vec<_>>>()?;
        drop(chapters_stmt);

        let mut out = Vec::new();
        for (chapter_id, chapter_ord, chapter_title, chapter_source_text) in chapters {
            // 🔴 **`is_omitted` LỌC RIÊNG NỬA BẢN DỊCH, không lọc cả hàng** — thêm ở lượt rà
            // 2026-08-29. `core/segment/omit.rs` khai `is_omitted` là *"chốt lọc cho MỌI đầu
            // ra"* (FR133/AC5: câu bị cắt phải *"ẩn hoàn toàn, không dấu vết"*), và doc-comment
            // của chính module đó dự đoán ĐÚNG lỗi này: *"người viết Story 8.3 đọc AC của chính
            // nó, thấy đủ, và xuất ra một tệp mang nguyên câu người dùng đã quyết định bỏ"*.
            // Tìm kiếm là một bề mặt tiêu thụ MỚI, nên nghĩa vụ đó áp cho nó.
            //
            // ⇒ `target_text` của một câu đã cắt đi vào chỉ mục là chuỗi RỖNG: người dùng đã
            // quyết định nó không thuộc bản dịch, nên nó không được hiện lại trong một đoạn
            // trích. `source_text` thì GIỮ NGUYÊN — FR133 cắt câu khỏi BẢN DỊCH, không xoá nó
            // khỏi nguyên tác, và FR8 hứa tìm được trong nguyên văn của mọi Tác phẩm. Lọc cả
            // hàng sẽ làm một câu CÓ THẬT trong nguyên tác biến mất khỏi tìm kiếm — một lớp
            // rỗng im lặng khác, đổi lỗi này lấy lỗi kia.
            let mut seg_stmt = conn.prepare(
                "SELECT id, ord, source_text, \
                        CASE WHEN is_omitted = 1 THEN '' ELSE target_text END \
                 FROM segment \
                 WHERE chapter_id = ?1 AND retired_at IS NULL ORDER BY ord",
            )?;
            let segments: Vec<(i64, i64, String, String)> = seg_stmt
                .query_map([chapter_id], |row| {
                    Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?))
                })?
                .collect::<SqlResult<Vec<_>>>()?;
            drop(seg_stmt);

            if segments.is_empty() {
                // §I/O Matrix "Chương chưa tách segment" — một hit CẤP CHƯƠNG, chỉ nửa
                // NGUYÊN VĂN (không cột `target_text` ở `chapter`).
                out.push(HarvestedRow {
                    chapter_id,
                    chapter_ord,
                    chapter_title: chapter_title.clone(),
                    segment_id: None,
                    segment_ord: 0,
                    source_text: chapter_source_text,
                    target_text: String::new(),
                });
            } else {
                for (segment_id, segment_ord, source_text, target_text) in segments {
                    out.push(HarvestedRow {
                        chapter_id,
                        chapter_ord,
                        chapter_title: chapter_title.clone(),
                        segment_id: Some(segment_id),
                        segment_ord,
                        source_text,
                        target_text,
                    });
                }
            }
        }
        Ok(out)
    });

    db.close();
    result.map_err(|err| format!("doc du lieu tu project.db that bai: {err}"))
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
    /// tổng số hàng mồ côi hiện có (đó là [`Self::current_orphans`]`.len()`). `0` là bình
    /// thường (mọi hàng đã ở đúng vị trí, hoặc đây là lượt quét đầu tiên trên một chỉ mục
    /// rỗng); khác `root_missing`, một `orphans > 0` không tự nó là một lỗi — nó là bằng
    /// chứng đối chiếu đang hoạt động.
    pub orphans: usize,
    /// 🔵 **THÊM (2026-08-27, vòng rà THỨ HAI P3)** — ảnh chụp TOÀN BỘ hàng mồ côi hiện có,
    /// lấy TRONG cùng phạm vi đã khoá bởi `rebuild_lock` — KHÔNG phải một lượt
    /// `list_orphans()` riêng gọi SAU khi hàm này đã trả về.
    ///
    /// ─────────────────────────────────────────────────────────────────────────
    /// 🔴 VÌ SAO — BA PHÁT HIỆN, MỘT GỐC
    /// ─────────────────────────────────────────────────────────────────────────
    /// Trước bản vá, `commands::library::rescan` gọi `rebuild()` (có khoá) RỒI
    /// `list_orphans()` (KHÔNG khoá) như hai lời gọi tách rời. Một lượt `rebuild`/
    /// `forget_orphan` khác chen vào đúng khe hở giữa hai lời gọi đó làm `orphans` trên dây
    /// phản ánh một THẾ HỆ KHÁC với `indexed`/`conflicts`/`skipped` trong CÙNG một báo cáo —
    /// ngược đúng lời hứa của doc-comment `RescanReport` ("một lượt gọi là đủ cho cả màn
    /// hình"). Trường này đóng lỗ đó: nó được đọc trong khi `_guard` (khoá `rebuild_lock`)
    /// còn sống, nên không lượt `rebuild`/`forget_orphan` nào khác có thể chen vào giữa lúc
    /// giao dịch vừa commit và lúc ảnh chụp này được chụp.
    ///
    /// 🔵 **SỬA (2026-08-27, phán quyết Ice #1) — kiểu đổi từ `Vec<IndexedWork>` sang
    /// `Vec<OrphanRecord>`.** Từ khi cờ mồ côi chuyển sang `library_orphan` (`global.db`),
    /// một hàng mồ côi không còn mang các trường của `library_work` (`source_lang`, `genre`,
    /// `created_at`, `updated_at`, `chapter_count`) — nó chỉ có `work_id`/`atproj_path`/`name`,
    /// đúng ba cột của bảng mới. Xem [`super::orphan_store::OrphanRecord`].
    pub current_orphans: Vec<OrphanRecord>,
    /// **THÊM Story 5.9.** Mọi Tác phẩm (trong `kept` của lượt này) mà lượt thu hoạch VĂN BẢN
    /// bị bỏ qua — `library_work` của nó vẫn UPSERT bình thường từ `meta.json`; chỉ phần
    /// `library_segment`/ba chỉ mục FTS5 (🔵 hai → ba, Story 5.10) của đúng Tác phẩm này vắng mặt (`project.db` vắng
    /// mặt, phiên bản lược đồ mới hơn ứng dụng hiểu, hoặc mở/đọc trượt). Rỗng là bình thường
    /// (mọi Tác phẩm thu hoạch được); khác `orphans`, một `text_skipped` không rỗng không tự
    /// nó là một lỗi hệ thống — nó là bằng chứng CÓ TÊN cho một Tác phẩm cụ thể.
    pub text_skipped: Vec<TextHarvestSkipped>,
}

/// **THÊM Story 5.9.** Một Tác phẩm mà lượt thu hoạch văn bản của lượt `rebuild` này bị bỏ
/// qua — xem [`RebuildOutcome::text_skipped`].
#[derive(Debug, Clone)]
pub struct TextHarvestSkipped {
    pub work_id: String,
    /// Chẩn đoán, KHÔNG DẤU (NFR16) — không phải văn bản hiển thị. Nêu ĐÍCH DANH lý do:
    /// `project.db` vắng mặt / phiên bản lược đồ mới hơn ứng dụng / mở-đọc thất bại.
    pub reason: String,
}

impl RebuildOutcome {
    /// Ghi chẩn đoán KHÔNG DẤU ra stderr khi [`Self::conflicts`]/[`Self::skipped`] không rỗng.
    ///
    /// 🔴 **THÊM (vòng rà ba lớp, P7)** — AD-28 đòi `Indexer` *"phát hiện VÀ CẢNH BÁO hai Tác
    /// phẩm trùng `work.id`"*. Trước bản vá, cả hai chỗ gọi sản phẩm (`lib.rs::open_library_index`
    /// lúc khởi động, `commands::project::wire::reindex_library` — đổi tên ở Story 5.4, khi
    /// đó còn tên `reindex_after_create_work` — sau khi tạo Tác phẩm) đều VỨT `RebuildOutcome`
    /// bằng `if let Err(err) = indexer.rebuild(..) { .. }` —
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
        // **THÊM Story 5.9** -- cùng đường chẩn đoán, cùng lý do đã ghi ở khối trên: hai chỗ
        // gọi sản phẩm không được trôi khỏi nhau. Mỗi Tác phẩm bị bỏ qua ĐÃ được ghi riêng
        // ngay lúc `harvest_work_text` thất bại (`Indexer::rebuild`); dòng tổng hợp này chỉ
        // cho một chỗ gọi lười biếng đọc `RebuildOutcome` mà không quan tâm dòng log chi tiết.
        if let Some(first) = self.text_skipped.first() {
            eprintln!(
                "library[index:{surface}] {} Tac pham bi bo qua phan van ban khi thu hoach -- \
                 vd. work_id={} ({})",
                self.text_skipped.len(),
                first.work_id,
                first.reason
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

/// Một hàng của `library_work`, cho đường đọc [`Indexer::list_works`].
///
/// 🔵 **SỬA (2026-08-27, phán quyết Ice #1) — gỡ trường `orphaned`.** `library_work` dẫn
/// xuất TRỌN VẸN trở lại: mọi hàng còn trong bảng này đều đang sống theo định nghĩa (một
/// hàng chuyển sang mồ côi bị XOÁ khỏi đây, không còn đánh dấu tại chỗ — xem
/// [`super::orphan_store::OrphanRecord`] cho hình dạng của một hàng MỒ CÔI, nay sống ở một
/// bảng/kho khác).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IndexedWork {
    pub work_id: String,
    /// Đường dẫn TUYỆT ĐỐI trên máy này — khác `meta.json`, nơi đường tuyệt đối bị cấm
    /// (AC5, Story 1.15). Xem doc-comment của `LIBRARY_WORK_DDL` (`core/store/schema.rs`).
    pub atproj_path: PathBuf,
    pub name: String,
    pub source_lang: String,
    pub genre: String,
    pub created_at: String,
    pub updated_at: String,
    pub chapter_count: u32,
    /// 🔵 **THÊM (2026-08-27, Story 5.4)** — một trong bốn giá trị trên dây của
    /// [`crate::core::lifecycle::LifecycleStatus`], hoặc `None` (*"chưa biết"* — hàng đến từ
    /// một `meta.json` v1 chưa từng qua `WorkMeta::rebuild_from_store`).
    pub status: Option<String>,
    /// 🔵 **THÊM (2026-08-27, Story 5.4)** — `true` ⇔ [`Self::status`] đến từ ghi đè thủ
    /// công. Vô nghĩa khi `status` là `None`.
    pub status_is_override: bool,
    /// 🔵 **THÊM (2026-08-28, Story 5.5)** — số Chương ở `chapter.status = 'done'` (FR7), hoặc
    /// `None` (*"chưa biết"* — hàng đến từ một `meta.json` v1/v2 chưa từng qua
    /// `WorkMeta::rebuild_from_store` của story này). Độc lập với [`Self::status_is_override`]:
    /// ghi đè thủ công trạng thái Tác phẩm không bao giờ đổi trường này.
    pub chapter_done_count: Option<u32>,
}

/// Khoá sắp xếp — danh mục ĐÓNG, Story 5.6, FR10. Hai giá trị hôm nay; phân giải qua
/// [`Self::from_wire`] ở tầng lệnh (`commands::library::list_works`), cùng khuôn
/// `LifecycleStatus::from_wire` (`core/lifecycle/mod.rs`) — một khoá lạ trên dây ⇒ `IpcError`,
/// KHÔNG im lặng rơi về mặc định (§Always của story).
///
/// ⚠️ **Không một `enum` thứ hai/biến thể thứ ba tiện tay** — thêm một khoá sắp là một quyết
/// định sản phẩm (thêm một `<option>` + một khoá nhãn `vi.json`), không một dòng mã.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WorkSortKey {
    /// Ngày sửa gần nhất trước — **mặc định** (FR10 nêu nó trước).
    UpdatedDesc,
    /// Tên, không phân biệt hoa/thường (`COLLATE NOCASE`).
    NameAsc,
}

impl WorkSortKey {
    pub const ALL: &'static [WorkSortKey] = &[WorkSortKey::UpdatedDesc, WorkSortKey::NameAsc];

    /// Định danh máy đọc — thứ đi trên dây. Không phải nhãn hiển thị (AD-21).
    pub const fn as_str(self) -> &'static str {
        match self {
            WorkSortKey::UpdatedDesc => "updated_desc",
            WorkSortKey::NameAsc => "name_asc",
        }
    }

    /// Phân giải một giá trị đến từ dây IPC. `None` ⇒ giá trị ngoài danh mục đóng, chỗ gọi tự
    /// dựng lỗi (không đoán, không rơi về mặc định).
    pub fn from_wire(raw: &str) -> Option<WorkSortKey> {
        match raw {
            "updated_desc" => Some(WorkSortKey::UpdatedDesc),
            "name_asc" => Some(WorkSortKey::NameAsc),
            _ => None,
        }
    }
}

impl Default for WorkSortKey {
    /// §I/O Matrix "Sắp mặc định": `sort=None` ⇒ `updated_desc`.
    fn default() -> Self {
        WorkSortKey::UpdatedDesc
    }
}

/// Tham số một lượt [`Indexer::list_works`] — bộ lọc trạng thái (đã có từ Story 5.4) cộng
/// `genre`/`source_lang`/khoá sắp (Story 5.6), tất cả tính TRONG SQL ở tầng này (AD-1).
///
/// `Default` ⇒ không lọc gì, sắp `updated_desc` — đúng "Không lọc" của §I/O Matrix.
#[derive(Debug, Clone, Default)]
pub struct WorkQuery {
    /// `None` ⇒ không lọc trạng thái. `Some(vec![])` ⇒ khớp 0 hàng theo NGHĨA ĐEN — xem ⚠️ ở
    /// doc-comment của [`Indexer::list_works`].
    pub status: Option<Vec<crate::core::lifecycle::LifecycleStatus>>,
    /// `None` ⇒ không lọc lĩnh vực.
    pub genre: Option<String>,
    /// `None` ⇒ không lọc ngôn ngữ nguồn.
    pub source_lang: Option<String>,
    pub sort: WorkSortKey,
}

/// Kết quả một lượt [`Indexer::list_works`] — `total` LUÔN là tổng số hàng CHƯA LỌC,
/// `works.len()` là số hàng KHỚP bộ lọc (hoặc bằng `total` khi không lọc). Story 5.4.
///
/// 🔴 **KHÔNG một trường `matched: usize` riêng** — nó luôn bằng `works.len()`, và một
/// trường thứ hai mang cùng con số là hai dữ kiện có thể trôi khỏi nhau (`AGENTS.md::Known
/// pitfalls`). `commands::library::WorkListReport` (tầng lệnh) mới là nơi trường `matched`
/// tường minh xuất hiện — ở đó nó đi qua dây IPC, nơi suy luận từ `.length` phía TypeScript
/// đúng là điều AD-1 cấm.
///
/// 🔵 **THÊM (2026-08-28, Story 5.6) — `genres`/`source_langs`.** Hai tập giá trị CÓ THẬT,
/// `DISTINCT` trên bảng CHƯA LỌC, cùng một lượt đọc với `total`/`works` (xem doc-comment của
/// [`Indexer::list_works`]) — để giao diện dựng `<option>` mà không tự suy diễn từ `works` đã
/// lọc (AD-1: suy vậy làm lựa chọn TEO DẦN theo mỗi lượt lọc).
#[derive(Debug, Clone)]
pub struct WorksReport {
    pub total: usize,
    pub works: Vec<IndexedWork>,
    pub genres: Vec<String>,
    pub source_langs: Vec<String>,
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
    /// **THÊM Story 5.3.** [`Indexer::forget_orphan`] gọi trên một `work_id` không có mặt
    /// trong `library_orphan` — dù chưa từng mồ côi, dù đang SỐNG trong `library_work`, hay
    /// dù là một cái tên lạ hoàn toàn — cùng một nhánh từ chối cho mọi ca (§I/O Matrix của
    /// story).
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
