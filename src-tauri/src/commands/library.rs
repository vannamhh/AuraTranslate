//! Bề mặt IPC "Quét lại thư mục" — Story 5.3, FR99.
//!
//! Cùng khuôn `commands::config`/`commands::project`: hai lớp, hàm thuần trước
//! ([`rescan`]/[`forget_orphan`], nhận `Option<&Indexer>` — thứ `tests/**` gọi được không
//! cần webview), `#[tauri::command]` chỉ là vỏ mỏng trong `mod wire`.
//!
//! ─────────────────────────────────────────────────────────────────────────────
//! 🔴 MODULE NÀY GỌI XUỐNG `Indexer` QUA `try_state` — KHÔNG TỰ DỰNG `StoreSpec`
//! ─────────────────────────────────────────────────────────────────────────────
//! `tests/library_index_boundary.rs` canh: chỉ `core/library/indexer.rs` (chỗ gọi) và
//! `core/store/mod.rs` (điểm khai) được nhắc `StoreSpec::library_index`/
//! `StoreKind::LibraryIndex`. Tệp này KHÔNG bao giờ được nhắc hai định danh đó — cần đọc/ghi
//! `library-index.db` thì gọi qua `Indexer::rebuild`/`Indexer::forget_orphan`/
//! `Indexer::list_orphans`, đúng những gì hai hàm thuần dưới đây làm.
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
use crate::core::library::indexer::{Indexer, IndexedWork};
use crate::core::store::{Store, StoreError, StoreKind};

/// Một hàng mồ côi, gói lại cho dây IPC — `PathBuf` không `Serialize` trực tiếp thành thứ
/// frontend đọc được thuận tiện, nên đường dẫn đi qua dưới dạng chuỗi hiển thị được.
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

impl From<IndexedWork> for OrphanEntry {
    fn from(work: IndexedWork) -> Self {
        Self {
            work_id: work.work_id,
            name: work.name,
            atproj_path: work.atproj_path.display().to_string(),
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
    pub conflicts: usize,
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
/// # Lỗi
/// `indexer = None` ⇒ `library.indexer_missing`; quét/ghi trượt ⇒ lỗi của
/// [`crate::core::library::indexer::IndexError`] (qua `From<IndexError> for IpcError`).
pub fn rescan(indexer: Option<&Indexer>, root: &Path) -> Result<RescanReport, IpcError> {
    let indexer = indexer.ok_or_else(indexer_is_missing)?;

    let outcome = indexer.rebuild(root)?;
    // Cùng đường chẩn đoán CHUNG mà `lib.rs::open_library_index` và
    // `commands::project::wire::reindex_after_create_work` đã dùng — chỗ gọi thứ BA
    // (người dùng bấm) không được đứng ngoài quy ước đó.
    outcome.log_if_notable("rescan");

    let orphans = indexer.list_orphans()?;
    Ok(RescanReport {
        root: root.display().to_string(),
        // P1 -- không vứt `root_missing` nữa, chuyển thẳng nguyên vẹn.
        root_missing: outcome.root_missing,
        indexed: outcome.indexed,
        conflicts: outcome.conflicts.len(),
        skipped: outcome.skipped.len(),
        orphans: orphans.into_iter().map(OrphanEntry::from).collect(),
    })
}

/// **Hàm thuần** — gỡ đúng một hàng mồ côi, trả danh sách mồ côi CÒN LẠI (§I/O Matrix:
/// "trả danh sách mồ côi còn lại").
///
/// # Lỗi
/// `indexer = None` ⇒ `library.indexer_missing`; `work_id` không tồn tại hoặc đang sống ⇒
/// `library.not_orphaned` (qua `IndexError::NotOrphaned`).
pub fn forget_orphan(indexer: Option<&Indexer>, work_id: &str) -> Result<Vec<OrphanEntry>, IpcError> {
    let indexer = indexer.ok_or_else(indexer_is_missing)?;

    indexer.forget_orphan(work_id)?;

    let orphans = indexer.list_orphans()?;
    Ok(orphans.into_iter().map(OrphanEntry::from).collect())
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

    let report = rescan(indexer, path)?;
    Ok(Some(report))
}

/// Ba vỏ `#[tauri::command]`. **Không một quy tắc nào sống ở đây.**
pub mod wire {
    use tauri::Manager as _;
    use tauri_plugin_dialog::DialogExt as _;

    use super::{OrphanEntry, RescanReport, indexer_is_missing, root_invalid, store_is_missing};
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
        super::rescan(indexer.as_deref(), &root)
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
    ) -> Result<Vec<OrphanEntry>, IpcError> {
        let indexer = app.try_state::<Indexer>();
        super::forget_orphan(indexer.as_deref(), &work_id)
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
}
