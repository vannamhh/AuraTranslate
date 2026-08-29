//! Bề mặt IPC "Quét lại thư mục" (Story 5.3, FR99) VÀ đường ĐỌC "Bốn trạng thái vòng đời"
//! (Story 5.4, FR5/FR6 — chỉ vế LỌC/LIỆT KÊ; vế GHI sống ở `commands::lifecycle`).
//!
//! Cùng khuôn `commands::config`/`commands::project`: hai lớp, hàm thuần trước
//! ([`rescan`]/[`forget_orphan`]/[`list_works`], nhận `Option<&Indexer>` — thứ `tests/**` gọi
//! được không cần webview), `#[tauri::command]` chỉ là vỏ mỏng trong `mod wire`.
//!
//! ─────────────────────────────────────────────────────────────────────────────
//! 🔴 MODULE NÀY GỌI XUỐNG `Indexer` QUA `try_state` — KHÔNG TỰ DỰNG `StoreSpec`
//! ─────────────────────────────────────────────────────────────────────────────
//! `tests/library_index_boundary.rs` canh: chỉ `core/library/indexer.rs` (chỗ gọi) và
//! `core/store/mod.rs` (điểm khai) được nhắc `StoreSpec::library_index`/
//! `StoreKind::LibraryIndex`. Tệp này KHÔNG bao giờ được nhắc hai định danh đó — cần đọc/ghi
//! `library-index.db` thì gọi qua `Indexer::rebuild`/`Indexer::forget_orphan`, đúng những gì
//! hai hàm thuần dưới đây làm. 🔵 **SỬA (2026-08-27, phán quyết Ice #1)** — cờ mồ côi (bảng
//! `library_orphan`) sống ở `global.db`, KHÔNG ở `library-index.db`; `rescan`/`forget_orphan`
//! nay nhận thêm một `Option<&Store>` (toàn cục) để ghi/đọc bảng đó, truyền tiếp xuống
//! `Indexer::rebuild`/`forget_orphan`. Tệp này vẫn không tự viết SQL cho bảng đó — xem
//! `core::library::orphan_store`.
//!
//! `library_choose_root` là vỏ CHẶN duy nhất của tệp này (hộp thoại chọn thư mục) — cả ba
//! vỏ đều mang `#[tauri::command(async)]` vì `library_rescan` quét đĩa + ghi kho đồng bộ có
//! thể mất thời gian trên một thư viện lớn (AC1: giao diện phải còn bấm/gõ được suốt lượt).
//!
//! ⚠️ Mọi chuỗi trong tệp này viết KHÔNG DẤU — `scripts/check-i18n.mjs` Kiểm A quét
//! `src-tauri/**/*.rs`.

use std::collections::BTreeMap;
use std::path::Path;

use crate::core::i18n::{IpcError, MessageKey};
use crate::core::library::indexer::{
    DEFAULT_SEARCH_LIMIT, IndexError, Indexer, IndexedWork, SearchHit as CoreSearchHit,
    SearchReport as CoreSearchReport, WorkIdConflict, WorkQuery, WorkSortKey,
};
use crate::core::library::orphan_store::OrphanRecord;
use crate::core::lifecycle::LifecycleStatus;
use crate::core::store::{Store, StoreError, StoreKind};

/// Một hàng mồ côi, gói lại cho dây IPC — hình dạng trên dây TRÙNG [`OrphanRecord`] (đường
/// dẫn đã là `String` từ tầng `Indexer`, xem phán quyết Ice #1) nên đây gần như một alias,
/// nhưng vẫn giữ struct RIÊNG ở tầng lệnh: `core::library::orphan_store` là chi tiết triển
/// khai của lõi, còn struct này là HỢP ĐỒNG trên dây — hai vai khác nhau dù hôm nay trùng
/// hình dạng (cùng khuôn `RescanReport`/`RebuildOutcome` không phải cùng một struct).
///
/// ⚠️ `#[serde(rename_all = ...)]` KHÔNG đặt — cùng luật mọi struct qua biên (AD-21):
/// `snake_case` giữ nguyên ở chiều TRẢ VỀ.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct OrphanEntry {
    pub work_id: String,
    pub name: String,
    /// Đường dẫn CŨ mà hàng này từng trỏ tới — giữ nguyên, không xoá/làm rỗng (AC3: "nêu rõ
    /// nó trỏ tới đâu").
    pub atproj_path: String,
}

impl From<OrphanRecord> for OrphanEntry {
    fn from(record: OrphanRecord) -> Self {
        Self {
            work_id: record.work_id,
            name: record.name,
            atproj_path: record.atproj_path,
        }
    }
}

/// **THÊM (2026-08-27, phán quyết Ice #3)** — một cặp `.atproj` cùng `work_id`, gói lại cho
/// dây IPC. Thay thế con số trần `conflicts: usize` mà bản trước gửi — AC4 nói *"phát hiện
/// **VÀ** cảnh báo"*, và một con số trần không đủ dữ kiện để một màn hình nêu ĐÍCH DANH chỗ
/// trùng (AC4 đòi "hai Tác phẩm cùng `work.id`", không chỉ "có N xung đột").
///
/// ⚠️ `#[serde(rename_all = ...)]` KHÔNG đặt — cùng luật mọi struct qua biên (AD-21).
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct ConflictEntry {
    pub work_id: String,
    /// Đường dẫn `.atproj` **đang có mặt** trong chỉ mục (mục ĐẦU, theo thứ tự quét đã sắp).
    pub kept_path: String,
    /// Đường dẫn `.atproj` **trùng `work_id`**, bị loại khỏi lượt ghi này.
    pub duplicate_path: String,
}

impl From<WorkIdConflict> for ConflictEntry {
    fn from(conflict: WorkIdConflict) -> Self {
        Self {
            work_id: conflict.work_id,
            kept_path: conflict.kept_path.display().to_string(),
            duplicate_path: conflict.duplicate_path.display().to_string(),
        }
    }
}

/// Kết quả một lượt [`rescan`] — ba con số của AC1 cộng danh sách mồ côi hiện tại (sau lượt
/// quét này), để một lượt gọi `library.rescan` là đủ cho cả màn hình, không cần một lệnh
/// đọc riêng ngay sau đó.
#[derive(Debug, Clone, serde::Serialize)]
pub struct RescanReport {
    /// Thư mục gốc VỪA quét — màn hình tối thiểu của story này phải hiện được nó (§Never:
    /// "thư mục gốc, nút quét lại, danh sách mục mồ côi, ba con số kết quả"), và chỉ Rust
    /// mới biết bộ phân giải (móc e2e ⇒ cấu hình ⇒ mặc định) đã chọn đường nào.
    pub root: String,
    /// 🔵 **THÊM (2026-08-27, vòng rà bốn lớp P1)** — `true` ⇒ `root` KHÔNG tồn tại trên đĩa
    /// ở lượt quét này. Trước bản vá, `RebuildOutcome::root_missing` (đã tính đúng) bị VỨT ở
    /// đây, nên `indexed == 0` một mình phải gánh CẢ HAI nghĩa: "gốc vắng mặt" VÀ "gốc có
    /// tồn tại nhưng thật sự rỗng" — đúng lớp "rỗng im lặng" mà `AGENTS.md::Known pitfalls`
    /// gọi là lỗi trung tâm của dự án, và ngược §Always của story ("danh sách rỗng phải nói
    /// vì sao rỗng"). Màn hình phải hiện một câu RIÊNG khi trường này `true`.
    pub root_missing: bool,
    pub indexed: usize,
    /// 🔵 **SỬA (2026-08-27, phán quyết Ice #3) — đổi từ `usize` sang `Vec<ConflictEntry>`.**
    /// AC4 viết *"phát hiện **VÀ** cảnh báo"* — hai vế, và một con số nén không đủ dữ kiện
    /// cho vế "cảnh báo": nó không nói được CHỖ NÀO trùng. `.len()` vẫn cho con số cũ khi một
    /// chỗ gọi chỉ cần đếm (dòng ba-con-số của màn hình) — không mất khả năng cũ, chỉ thêm
    /// dữ kiện. Đóng băng ở `tests/ipc_contract.rs` cùng lượt.
    pub conflicts: Vec<ConflictEntry>,
    pub skipped: usize,
    pub orphans: Vec<OrphanEntry>,
}

/// `Indexer` chưa được quản lý (mở `library-index.db` thất bại lúc khởi động) — tái dùng
/// [`MessageKey::StoreOpenFailed`] thay vì đúc một khoá thứ ba (danh mục đóng của story chỉ
/// thêm ĐÚNG HAI khoá: `LibraryNotOrphaned`/`LibraryRootInvalid`).
///
/// 🔴 Không dựng `StoreError::OpenFailed { store: StoreKind::LibraryIndex, .. }` ở đây: viết
/// định danh đó trong tệp này vi phạm chính cổng ranh giới mà doc-comment module vừa nhắc.
fn indexer_is_missing() -> IpcError {
    let mut params = BTreeMap::new();
    params.insert("store".to_owned(), "library_index".to_owned());
    IpcError::new("library.indexer_missing", MessageKey::StoreOpenFailed, params, false)
}

/// Thư mục người dùng chọn qua hộp thoại không dùng được làm gốc Library.
fn root_invalid() -> IpcError {
    IpcError::new("library.root_invalid", MessageKey::LibraryRootInvalid, BTreeMap::new(), false)
}

/// **THÊM (2026-08-27, vòng rà bốn lớp P8)** — kho `global.db` vắng mặt ⇒ lỗi *mở kho*, cùng
/// khuôn `commands::glossary::store_is_missing`/`commands::pinned::store_is_missing`: đi qua
/// `From<StoreError> for IpcError`, không dựng `IpcError` bằng struct literal.
fn store_is_missing() -> IpcError {
    StoreError::OpenFailed {
        store: StoreKind::Global,
        detail: "the global store was never managed; see lib.rs::open_global_store".to_owned(),
    }
    .into()
}

/// **Hàm thuần** — quét lại `root`, trả ba con số của AC1 cộng danh sách mồ côi hiện tại.
///
/// 🔵 **THÊM tham số `global` (2026-08-27, phán quyết Ice #1).** Cờ mồ côi nay sống ở
/// `library_orphan` (`global.db`) — `Indexer::rebuild` cần một `&Store` đã mở tới kho đó để
/// ghi/đọc. Vỏ [`wire::library_rescan`] đã fetch `Store` sẵn cho [`crate::commands::project::resolve_library_root`],
/// nên đây không phải một chỗ gọi `try_state` thứ hai — chỉ là truyền tiếp cùng một giá trị.
///
/// # Lỗi
/// `indexer = None` ⇒ `library.indexer_missing`; `global = None` hoặc quét/ghi trượt ⇒ lỗi của
/// [`crate::core::library::indexer::IndexError`] (qua `From<IndexError> for IpcError`).
pub fn rescan(
    indexer: Option<&Indexer>,
    global: Option<&Store>,
    root: &Path,
) -> Result<RescanReport, IpcError> {
    let indexer = indexer.ok_or_else(indexer_is_missing)?;

    let outcome = indexer.rebuild(root, global)?;
    // Cùng đường chẩn đoán CHUNG mà `lib.rs::open_library_index` và
    // `commands::project::wire::reindex_library` (đổi tên ở Story 5.4) đã dùng — một chỗ gọi
    // KHÁC (người dùng bấm "Quét lại") không được đứng ngoài quy ước đó.
    outcome.log_if_notable("rescan");

    // `outcome.current_orphans` là ảnh chụp lấy TRONG cùng phạm vi khoá của
    // `Indexer::rebuild` (nay đọc từ `global.db`) — dùng thẳng nó, không một lượt đọc riêng.
    Ok(RescanReport {
        root: root.display().to_string(),
        // P1 -- không vứt `root_missing` nữa, chuyển thẳng nguyên vẹn.
        root_missing: outcome.root_missing,
        indexed: outcome.indexed,
        // Phán quyết Ice #3 -- chở NGUYÊN dữ liệu xung đột, không nén thành `.len()`.
        conflicts: outcome.conflicts.into_iter().map(ConflictEntry::from).collect(),
        skipped: outcome.skipped.len(),
        orphans: outcome.current_orphans.into_iter().map(OrphanEntry::from).collect(),
    })
}

/// **THÊM (2026-08-27, vòng rà THỨ HAI P9)** — `err.library.not_orphaned` phải nói được
/// TÊN mục, không chỉ `work_id` (một UUID trần không phải thứ người dùng nhận ra).
fn not_orphaned(work_id: String, name: String) -> IpcError {
    let mut params = BTreeMap::new();
    params.insert("work_id".to_owned(), work_id);
    params.insert("name".to_owned(), name);
    IpcError::new("library.not_orphaned", MessageKey::LibraryNotOrphaned, params, false)
}

/// **Hàm thuần** — gỡ đúng một hàng mồ côi, trả danh sách mồ côi CÒN LẠI (§I/O Matrix:
/// "trả danh sách mồ côi còn lại").
///
/// 🔵 **THÊM tham số `name` (2026-08-27, vòng rà THỨ HAI P9).** `Indexer::forget_orphan`
/// không biết "tên mà người dùng đang thấy" — đó là dữ liệu của TẦNG GỌI
/// (`LibraryMode.vue` đang hiển thị `currentLibraryOrphan.name` ngay lúc người dùng bấm),
/// nên hàm này nhận nó qua tham số và tự dựng `IpcError` cho ca từ chối thay vì đi qua
/// `From<IndexError>` chung (nó chỉ có `work_id`).
///
/// # Lỗi
/// `indexer = None` ⇒ `library.indexer_missing`; `global = None` ⇒ lỗi kho toàn cục;
/// `work_id` không tồn tại hoặc đang sống ⇒ `library.not_orphaned` (mang CẢ `work_id` LẪN
/// `name` vừa truyền vào).
pub fn forget_orphan(
    indexer: Option<&Indexer>,
    global: Option<&Store>,
    work_id: &str,
    name: &str,
) -> Result<Vec<OrphanEntry>, IpcError> {
    let indexer = indexer.ok_or_else(indexer_is_missing)?;

    // `Indexer::forget_orphan` tự trả danh sách mồ côi còn lại (chụp trong cùng phạm vi
    // khoá) — không một lượt `list_orphans()` riêng, không khoá, chạy SAU.
    match indexer.forget_orphan(work_id, global) {
        Ok(orphans) => Ok(orphans.into_iter().map(OrphanEntry::from).collect()),
        // P9 -- KHÔNG uỷ quyền cho `From<IndexError> for IpcError` ở ca này: hàm đó chỉ biết
        // `work_id`. Bắt riêng để nạp thêm `name` do chỗ gọi cung cấp.
        Err(IndexError::NotOrphaned { work_id }) => Err(not_orphaned(work_id, name.to_owned())),
        Err(other) => Err(other.into()),
    }
}

/// **Hàm thuần** — mọi thứ xảy ra SAU khi hộp thoại chọn thư mục đã đóng.
///
/// `picked = None` (người dùng HUỶ) ⇒ `Ok(None)`: **không** ghi cấu hình, **không** quét
/// lại, và **không** một biến thể lỗi nào (§I/O Matrix "Huỷ hộp thoại").
///
/// 🔴 **Vì sao tách khỏi vỏ, chứ không để inline như bản dựng đầu.** `blocking_pick_folder()`
/// cần một cửa sổ THẬT, nên nhánh huỷ nằm trong vỏ là một nhánh **không ca nào chạy được** —
/// và §I/O Matrix của story có một hàng đòi đúng nhánh đó. `Option<&Path>` là ranh giới XA
/// NHẤT còn viết được một ca hợp đồng cho nó, đúng cùng lý lẽ mà
/// `core/library/indexer.rs::partition_dir_entries` đã tách ra khỏi `scan_atproj_dirs`.
/// Đo: `tests/library_commands_contract.rs::cancelling_the_folder_dialog_writes_no_config_and_leaves_the_index_alone`.
///
/// Thứ tự CÓ CHỦ: ghi cấu hình TRƯỚC, quét sau. Một lượt quét trượt vẫn để lại lựa chọn của
/// người dùng trên đĩa — ngược lại thì người dùng chọn xong thư mục, thấy lỗi, và lần mở sau
/// vẫn ở gốc cũ mà không hiểu vì sao.
///
/// # Lỗi
/// đường dẫn không phải thư mục ⇒ `library.root_invalid`; ghi cấu hình trượt ⇒ lỗi kho;
/// `indexer = None` ⇒ `library.indexer_missing` (SAU khi cấu hình đã ghi — xem trên).
pub fn apply_chosen_root(
    store: Option<&Store>,
    indexer: Option<&Indexer>,
    picked: Option<&Path>,
) -> Result<Option<RescanReport>, IpcError> {
    let Some(path) = picked else {
        return Ok(None);
    };
    if !path.is_dir() {
        return Err(root_invalid());
    }

    crate::commands::config::put_config(
        store,
        "app_config",
        "library_root",
        &path.display().to_string(),
    )?;

    // `store` LÀ `global.db` -- cùng giá trị `rescan`/`Indexer::rebuild` cần cho
    // `library_orphan` (phán quyết Ice #1). Không một `try_state` thứ hai.
    let report = rescan(indexer, store, path)?;
    Ok(Some(report))
}

// ═════════════════════════════════════════════════════════════════════════════════
// Story 5.4 — đường ĐỌC "bốn trạng thái vòng đời": danh sách Tác phẩm + bộ lọc.
// ═════════════════════════════════════════════════════════════════════════════════

/// Một hàng của `library_work`, gói lại cho dây IPC — hình dạng `snake_case`, đóng băng ở
/// `tests/ipc_contract.rs`.
///
/// ⚠️ `#[serde(rename_all = ...)]` KHÔNG đặt — cùng luật mọi struct qua biên (AD-21).
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct WorkRow {
    pub work_id: String,
    pub atproj_path: String,
    pub name: String,
    pub source_lang: String,
    pub genre: String,
    pub created_at: String,
    pub updated_at: String,
    pub chapter_count: u32,
    pub status: Option<String>,
    pub status_is_override: bool,
    /// 🔵 **THÊM (2026-08-28, Story 5.5)** — số Chương đã xong (FR7), hoặc `None` (*"chưa
    /// biết"*). Xem doc-comment của `IndexedWork::chapter_done_count`.
    pub chapter_done_count: Option<u32>,
}

impl From<IndexedWork> for WorkRow {
    fn from(work: IndexedWork) -> Self {
        Self {
            work_id: work.work_id,
            atproj_path: work.atproj_path.display().to_string(),
            name: work.name,
            source_lang: work.source_lang,
            genre: work.genre,
            created_at: work.created_at,
            updated_at: work.updated_at,
            chapter_count: work.chapter_count,
            status: work.status,
            status_is_override: work.status_is_override,
            chapter_done_count: work.chapter_done_count,
        }
    }
}

/// Kết quả một lượt [`list_works`] — `total` (tổng số hàng CHƯA LỌC) VÀ `matched`
/// (`works.len()`, tường minh trên dây) trong **MỘT** lượt đọc, để hai con số không bao giờ
/// đến từ hai ảnh chụp khác nhau (§Always: *"một danh sách rỗng phải nói vì sao rỗng"*).
///
/// ⚠️ `matched` LÀ `works.len()` — trường tường minh trên dây có chủ ý (không suy luận từ
/// `.length` phía TypeScript, AD-1), khác [`crate::core::library::indexer::WorksReport`] ở
/// tầng dưới, nơi một trường thứ hai mang cùng con số là thứ có thể trôi khỏi nhau nên bị bỏ.
///
/// 🔵 **THÊM (2026-08-28, Story 5.6)** — `genres`/`source_langs`: hai tập giá trị CÓ THẬT,
/// chép NGUYÊN VẸN từ [`crate::core::library::indexer::WorksReport`] (đã `DISTINCT` trên bảng
/// CHƯA LỌC ở tầng dưới) — giao diện dựng `<option>` từ ĐÂY, không tự suy từ `works` đã lọc
/// (AD-1, §Always: suy vậy làm lựa chọn TEO DẦN theo mỗi lượt lọc).
#[derive(Debug, Clone, serde::Serialize)]
pub struct WorkListReport {
    pub total: usize,
    pub matched: usize,
    pub works: Vec<WorkRow>,
    pub genres: Vec<String>,
    pub source_langs: Vec<String>,
}

/// Giá trị khoá sắp ngoài danh mục hai giá trị đóng của [`WorkSortKey`] — chép khuôn
/// [`crate::commands::lifecycle::unknown_status`] (Story 5.6, §Tasks).
fn unknown_sort(sort: &str) -> IpcError {
    let mut params = BTreeMap::new();
    params.insert("sort".to_owned(), sort.to_owned());
    IpcError::new("library.unknown_sort", MessageKey::LibraryUnknownSort, params, false)
}

/// **Hàm thuần** — liệt kê + lọc + sắp Tác phẩm cho Library, Story 5.4 (bộ lọc trạng thái) +
/// Story 5.6 (lĩnh vực · ngôn ngữ nguồn · sắp xếp).
///
/// Bộ lọc trạng thái là danh sách chuỗi trên dây (0 hoặc nhiều trong bốn giá trị của
/// [`LifecycleStatus`]); `filter = None` hoặc `Some(&[])` ⇒ không lọc trạng thái — mọi hàng,
/// kể cả hàng `status IS NULL`. `genre`/`source_lang`: `None` ⇒ không lọc lĩnh vực/ngôn ngữ
/// tương ứng — KHÔNG chuẩn hoá chuỗi rỗng thành `None` ở đây (đó là việc của tầng gọi, nếu
/// nó muốn); một `Some("")` lọc đúng nghĩa đen "lĩnh vực RỖNG". `sort = None` ⇒ mặc định
/// [`WorkSortKey::UpdatedDesc`] (§I/O Matrix "Sắp mặc định").
///
/// # Lỗi
/// `indexer = None` ⇒ `library.indexer_missing`; một giá trị trong `filter` ngoài danh mục
/// bốn giá trị đóng ⇒ `err.lifecycle.unknown_status` `{status}` (tái dùng
/// [`crate::commands::lifecycle::unknown_status`], KHÔNG im lặng bỏ qua giá trị lạ); `sort`
/// ngoài danh mục hai giá trị đóng ⇒ `err.library.unknown_sort` `{sort}`, KHÔNG rơi về mặc
/// định (§Always).
pub fn list_works(
    indexer: Option<&Indexer>,
    filter: Option<&[String]>,
    genre: Option<&str>,
    source_lang: Option<&str>,
    sort: Option<&str>,
) -> Result<WorkListReport, IpcError> {
    let indexer = indexer.ok_or_else(indexer_is_missing)?;

    let parsed_filter: Option<Vec<LifecycleStatus>> = match filter {
        None => None,
        Some(raw) if raw.is_empty() => None,
        Some(raw) => {
            let mut statuses = Vec::with_capacity(raw.len());
            for value in raw {
                let status = LifecycleStatus::from_wire(value)
                    .ok_or_else(|| crate::commands::lifecycle::unknown_status(value))?;
                statuses.push(status);
            }
            Some(statuses)
        }
    };

    let parsed_sort = match sort {
        None => WorkSortKey::default(),
        Some(raw) => WorkSortKey::from_wire(raw).ok_or_else(|| unknown_sort(raw))?,
    };

    let query = WorkQuery {
        status: parsed_filter,
        genre: genre.map(str::to_owned),
        source_lang: source_lang.map(str::to_owned),
        sort: parsed_sort,
    };

    let report = indexer.list_works(query)?;
    let works: Vec<WorkRow> = report.works.into_iter().map(WorkRow::from).collect();
    Ok(WorkListReport {
        total: report.total,
        matched: works.len(),
        works,
        genres: report.genres,
        source_langs: report.source_langs,
    })
}

// ═════════════════════════════════════════════════════════════════════════════════
// Story 5.9 — "Tìm kiếm full-text xuyên Library" (FR8).
// ═════════════════════════════════════════════════════════════════════════════════

/// Một kết quả tìm kiếm, gói lại cho dây IPC — hình dạng `snake_case`, đóng băng ở
/// `tests/ipc_contract.rs`. Story 5.9.
///
/// ⚠️ `#[serde(rename_all = ...)]` KHÔNG đặt — cùng luật mọi struct qua biên (AD-21).
/// `field` là `String` ("target"/"source"), KHÔNG một enum Rust lộ ra dây — [`SearchField`]
/// (`core::library::indexer`) là chi tiết TRIỂN KHAI của lõi; hình dạng trên dây tự nó là hợp
/// đồng, đúng khuôn `WorkRow`/`IndexedWork` ngay trên.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct SearchHit {
    pub work_id: String,
    pub work_name: String,
    pub chapter_id: i64,
    pub chapter_ord: i64,
    pub chapter_title: Option<String>,
    /// `null` ⇒ hit CẤP CHƯƠNG (Chương chưa tách segment sống nào). Lượt mở kết quả
    /// (`src/modes/librarySearch.ts::openSearchHit`) truyền nó nguyên vẹn xuống
    /// `openChapterById(chapterId, segmentId)` — `undefined` khi `null`, để Rust quyết con trỏ.
    pub segment_id: Option<i64>,
    pub field: String,
    /// Đoạn trích văn bản THUẦN, cặp dấu `‹…›` bao quanh phần khớp — KHÔNG một thẻ HTML nào
    /// (AD-16). Render bằng nội suy Vue thường, không `v-html`.
    pub snippet: String,
}

impl From<CoreSearchHit> for SearchHit {
    fn from(hit: CoreSearchHit) -> Self {
        Self {
            work_id: hit.work_id,
            work_name: hit.work_name,
            chapter_id: hit.chapter_id,
            chapter_ord: hit.chapter_ord,
            chapter_title: hit.chapter_title,
            segment_id: hit.segment_id,
            field: hit.field.as_str().to_owned(),
            snippet: hit.snippet,
        }
    }
}

/// Kết quả một lượt [`search_library`] — Story 5.9. Ba trường ngoài `hits` cho
/// `src/modes/librarySearch.ts` đủ dữ kiện phân biệt NĂM ca rỗng của §I/O Matrix mà không cần
/// một lượt IPC thứ hai: `indexed_segments == 0` ⇒ chỉ mục chưa có gì; `short_query` ⇒ dưới
/// sàn trigram; `hits` rỗng mà cả hai trường kia không rơi vào hai ca trên ⇒ không khớp thật.
#[derive(Debug, Clone, serde::Serialize)]
pub struct SearchReport {
    pub hits: Vec<SearchHit>,
    /// `hits.len()` — trường TƯỜNG MINH, không suy từ `.length` phía TypeScript (AD-1, cùng lý
    /// lẽ `WorkListReport::matched`).
    pub total: usize,
    pub indexed_segments: usize,
    pub short_query: bool,
    /// 🔴 `true` ⇔ danh sách đã bị trần `limit` CẮT — `total` khi đó là *"số hàng đang hiện"*,
    /// không phải *"số hàng khớp"*. Xem [`crate::core::library::indexer::SearchReport::truncated`]
    /// cho lý do trường này tồn tại (một trần cắt trong im lặng là một câu khẳng định sai).
    pub truncated: bool,
}

impl From<CoreSearchReport> for SearchReport {
    fn from(report: CoreSearchReport) -> Self {
        Self {
            hits: report.hits.into_iter().map(SearchHit::from).collect(),
            total: report.total,
            indexed_segments: report.indexed_segments,
            short_query: report.short_query,
            truncated: report.truncated,
        }
    }
}

/// **Hàm thuần** — tìm kiếm full-text xuyên TOÀN BỘ Library (FR8), Story 5.9. `query` KHÔNG
/// được `trim()`/chuẩn hoá ở đây — đó là việc của tầng gọi (frontend không phát IPC cho một ô
/// tìm rỗng, §I/O Matrix "Truy vấn rỗng"); [`Indexer::search`] tự an toàn với một chuỗi rỗng
/// nếu nó lỡ tới đây.
///
/// `limit = None` ⇒ [`DEFAULT_SEARCH_LIMIT`] — chỗ gọi (webview) không phải biết con số đó.
///
/// # Lỗi
/// `indexer = None` ⇒ `library.indexer_missing` (tái dùng [`indexer_is_missing`], cùng khuôn
/// mọi lệnh khác của tệp này — danh mục `MessageKey` ĐÓNG của story chỉ tái dùng khoá đã có,
/// không đúc khoá mới).
pub fn search_library(
    indexer: Option<&Indexer>,
    query: &str,
    limit: Option<u32>,
) -> Result<SearchReport, IpcError> {
    let indexer = indexer.ok_or_else(indexer_is_missing)?;
    let limit = limit.map(|l| l as usize).unwrap_or(DEFAULT_SEARCH_LIMIT);
    let report = indexer.search(query, limit)?;
    Ok(SearchReport::from(report))
}

/// Năm vỏ `#[tauri::command]` — ba của Story 5.3, [`library_list_works`] của Story 5.4, cộng
/// [`library_search`] của Story 5.9.
/// **Không một quy tắc nào sống ở đây.**
pub mod wire {
    use tauri::Manager as _;
    use tauri_plugin_dialog::DialogExt as _;

    use super::{
        OrphanEntry, RescanReport, WorkListReport, indexer_is_missing, root_invalid,
        store_is_missing,
    };
    use crate::core::i18n::IpcError;
    use crate::core::library::indexer::Indexer;
    use crate::core::store::Store;

    /// Vỏ IPC của [`super::rescan`] — nhìn CÙNG thư mục gốc mà
    /// [`crate::commands::project::resolve_library_root`] cho lượt tạo Tác phẩm nhìn.
    ///
    /// 🔴 **`(async)` KHÔNG PHẢI TRANG TRÍ.** Một lượt quét chạm mọi `.atproj` trong gốc —
    /// trên một thư viện lớn đó là I/O đồng bộ đáng kể cộng một lượt ghi qua
    /// `store::Writer`; chạy trên luồng chính sẽ chặn đúng vòng lặp sự kiện mà AC1 đòi giữ
    /// mượt ("giao diện còn bấm/gõ được suốt lượt"). `#[tauri::command(async)]` đưa THÂN HÀM
    /// ra `sync_threadpool`, không đổi một dòng thân hàm — cùng khuôn năm vỏ CHẶN của
    /// `commands/glossary.rs`. Cổng canh: `config_invariants.rs::the_blocking_wires_run_off_the_main_thread`.
    #[tauri::command(async)]
    pub fn library_rescan(app: tauri::AppHandle) -> Result<RescanReport, IpcError> {
        let indexer = app.try_state::<Indexer>();
        let store = app.try_state::<Store>();
        let root =
            crate::commands::project::resolve_library_root(&app, store.as_deref())?;
        // Cùng `store` vừa dùng cho `resolve_library_root` -- `Indexer::rebuild` cần nó để
        // ghi/đọc `library_orphan` (phán quyết Ice #1). Một `try_state` DUY NHẤT cho cả hai.
        super::rescan(indexer.as_deref(), store.as_deref(), &root)
    }

    /// Vỏ IPC của [`super::forget_orphan`].
    ///
    /// 🔴 `(async)` — cùng lý do [`library_rescan`]: một lượt ghi qua `store::Writer` là một
    /// chờ đồng bộ trên luồng gọi, và cổng `the_blocking_wires_run_off_the_main_thread` canh
    /// TẤT CẢ ba vỏ của tệp này như một nhóm, không phân biệt "nặng"/"nhẹ".
    #[tauri::command(async)]
    pub fn library_forget_orphan(
        app: tauri::AppHandle,
        work_id: String,
        // 🔵 THÊM (2026-08-27, vòng rà THỨ HAI P9) — frontend gửi `name` (đã hiển thị sẵn
        // trên màn hình) để một lượt từ chối nói được TÊN, không chỉ UUID trần.
        name: String,
    ) -> Result<Vec<OrphanEntry>, IpcError> {
        let indexer = app.try_state::<Indexer>();
        // `library_orphan` sống ở `global.db` (phán quyết Ice #1) -- fetch cùng khuôn hai vỏ
        // kia của tệp này.
        let store = app.try_state::<Store>();
        super::forget_orphan(indexer.as_deref(), store.as_deref(), &work_id, &name)
    }

    /// Vỏ IPC mở hộp thoại CHỌN THƯ MỤC rồi đổi `AppConfig::library_root` + quét lại ngay
    /// trên thư mục vừa chọn — AD-48.
    ///
    /// 🔴 **P1 (khuôn chép từ `commands::glossary::wire::glossary_export_tier`) — kiểm
    /// `Indexer` có mặt TRƯỚC khi mở hộp thoại.** Không lãng phí một lượt tương tác người
    /// dùng cho một thao tác chắc chắn trượt vì `Indexer` chưa được quản lý.
    ///
    /// Huỷ hộp thoại ⇒ `Ok(None)` — không ghi cấu hình, không quét, không một biến thể lỗi
    /// (§I/O Matrix "Huỷ hộp thoại").
    ///
    /// 🔴 **`(async)` KHÔNG PHẢI TRANG TRÍ — thiếu nó là TREO ỨNG DỤNG.**
    /// `blocking_pick_folder()` chặn vòng lặp sự kiện mà chính hộp thoại đang chờ, đúng lớp
    /// lỗi đã ĐO ở Story 3.10b (macOS báo "Not Responding"). `#[tauri::command(async)]` cho
    /// `sync_threadpool` chạy nó ngoài luồng chính.
    #[tauri::command(async)]
    pub fn library_choose_root(app: tauri::AppHandle) -> Result<Option<RescanReport>, IpcError> {
        // 🔵 THÊM (2026-08-27, vòng rà bốn lớp P8) — kiểm CẢ `Store` lẫn `Indexer` TRƯỚC khi
        // mở hộp thoại, không chỉ `Indexer`. `apply_chosen_root` sẽ tự trả lỗi qua
        // `put_config` nếu `Store` vắng mặt, nhưng CHỈ SAU KHI người dùng đã duyệt xong một
        // thư mục -- đúng thứ khuôn P1 (chép từ `commands::glossary::wire::glossary_export_tier`)
        // tồn tại để tránh: "không lãng phí một lượt tương tác người dùng cho một thao tác
        // chắc chắn trượt".
        if app.try_state::<Store>().is_none() {
            return Err(store_is_missing());
        }
        let Some(indexer) = app.try_state::<Indexer>() else {
            return Err(indexer_is_missing());
        };
        drop(indexer); // Chỉ kiểm SỰ CÓ MẶT trước dialog (P1) -- khoá LẠI sau khi đóng.

        let picked = app.dialog().file().blocking_pick_folder();
        // Huy hop thoai di qua CUNG mot ham thuan voi nhanh chon that -- vo nay khong tu
        // quyet dinh gi. Xem doc-comment cua `super::apply_chosen_root`.
        let path = match picked {
            Some(picked) => Some(picked.into_path().map_err(|_| root_invalid())?),
            None => None,
        };

        // Khoa MOI, sau khi hop thoai da dong (P1) -- khong tai dung gia tri doc truoc dialog.
        let store = app.try_state::<Store>();
        let indexer = app.try_state::<Indexer>();
        super::apply_chosen_root(store.as_deref(), indexer.as_deref(), path.as_deref())
    }

    /// Vỏ IPC của [`super::list_works`] — Story 5.4 (bộ lọc trạng thái) + Story 5.6 (lĩnh vực
    /// · ngôn ngữ nguồn · sắp xếp). `invoke()` gửi camelCase: `sourceLang` trên dây, không
    /// `source_lang` (`src/AGENTS.md`).
    ///
    /// Đọc thuần khỏi `library-index.db` -- không ghi, không cần `(async)` (khác ba vỏ trên
    /// của tệp này: chúng chặn vì I/O đồng bộ trên `.atproj`/hộp thoại, còn đây là một câu
    /// `SELECT` duy nhất qua kết nối đọc của pool).
    #[tauri::command]
    pub fn library_list_works(
        app: tauri::AppHandle,
        filter: Option<Vec<String>>,
        genre: Option<String>,
        source_lang: Option<String>,
        sort: Option<String>,
    ) -> Result<WorkListReport, IpcError> {
        let indexer = app.try_state::<Indexer>();
        super::list_works(
            indexer.as_deref(),
            filter.as_deref(),
            genre.as_deref(),
            source_lang.as_deref(),
            sort.as_deref(),
        )
    }

    /// Vỏ IPC của [`super::search_library`] — Story 5.9, FR8.
    ///
    /// 🔴 **`(async)`** — khác [`library_list_works`] (một `SELECT` duy nhất): mỗi lượt tìm
    /// chạy CẢ HAI chỉ mục FTS5 (§Always: không một bộ điều phối chọn một nhánh), và nhánh
    /// `trigram` còn kéo theo một bước xác minh chuỗi con Ở RUST trên TOÀN BỘ tập ứng viên
    /// (`search_candidate_ceiling`, tới 50× `limit`) trước khi cắt còn `limit` — nặng hơn hẳn
    /// một lượt liệt kê `library_work`. Cổng canh:
    /// `config_invariants.rs::the_blocking_wires_run_off_the_main_thread`.
    #[tauri::command(async)]
    pub fn library_search(
        app: tauri::AppHandle,
        query: String,
        limit: Option<u32>,
    ) -> Result<super::SearchReport, IpcError> {
        let indexer = app.try_state::<Indexer>();
        super::search_library(indexer.as_deref(), &query, limit)
    }
}
