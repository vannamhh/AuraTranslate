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
//! - [`Indexer::rebuild`] — quét thư mục gốc Library, đọc **chỉ** `meta.json` của mỗi
//!   `.atproj` (AD-9: không mở `project.db` lần nào), rồi **ĐỐI CHIẾU** kết quả với
//!   `library_work` trong **một** giao dịch qua `store::Writer`. 🔵 **ĐỔI NGỮ NGHĨA (Story
//!   5.3):** trước đây hàm này `DELETE FROM library_work` rồi `INSERT` lại toàn bộ — một
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
    LIBRARY_INDEX_MIGRATIONS, ReadHandle, Row, SqlResult, Store, StoreError, StoreKind, StoreSpec,
    Transaction, params_from_iter,
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
        let to_orphan: Vec<OrphanRecord> = self.store.write(move |tx: &Transaction<'_>| {
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

            Ok(to_orphan)
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

        // P3 -- cùng lý do nhánh `rebuild` bình thường: chụp TRONG khi khoá còn sống.
        let current_orphans = orphan_store::list(global)?;
        Ok(RebuildOutcome {
            indexed: 0,
            root_missing: true,
            conflicts: Vec::new(),
            skipped: Vec::new(),
            orphans,
            current_orphans,
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

            const COLUMNS: &str = "work_id, atproj_path, name, source_lang, genre, created_at, \
                 updated_at, chapter_count, status, status_is_override, chapter_done_count";

            let map_row = |row: &Row<'_>| -> SqlResult<IndexedWork> {
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
            };

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

            let mut stmt = conn.prepare(&format!("SELECT {COLUMNS} FROM library_work {where_clause} {order_clause}"))?;
            let rows = stmt.query_map(params_from_iter(params.iter()), map_row)?;
            let works: Vec<IndexedWork> = rows.collect::<SqlResult<Vec<_>>>()?;

            Ok(WorksReport { total, works, genres, source_langs })
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
