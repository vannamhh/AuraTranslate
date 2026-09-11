//! Bề mặt IPC tạo một Tác phẩm — Story 1.15, AC1/AC8.
//!
//! Cùng khuôn `commands::config`: hàm thuần trước, `#[tauri::command]` chỉ là vỏ mỏng
//! trong `wire`. Khác với `commands::config` (đọc/ghi một kho **đã mở**), hai hàm thuần ở
//! đây **tạo** kho — nên chúng nhận `documents_root: &Path` đã phân giải (qua `app.path()`
//! ở lớp vỏ, Quyết định #5) thay vì `Option<&Store>`: chưa có `Store` nào để nhận trước
//! khi [`create_work`] chạy xong bước đầu tiên.
//!
//! ─────────────────────────────────────────────────────────────────────────────
//! 🔴 BA ĐƯỜNG VÀO CỦA AC1 GẶP NHAU Ở ĐÚNG MỘT HÀM — [`create_work`]
//! ─────────────────────────────────────────────────────────────────────────────
//! Dán văn bản đổ vào [`crate::core::segment::import::import_text`] rồi tới đây; kéo-thả
//! và ô nhập đường dẫn đổ vào [`crate::core::segment::import::import_file`] rồi cũng tới
//! đây. [`create_work`] là chỗ **duy nhất** gọi [`crate::core::store::Store::write`] cho
//! `project.db` — không đường nào khác giữ một bản sao.
//!
//! ⚠️ Mọi chuỗi trong tệp này viết KHÔNG DẤU — `scripts/check-i18n.mjs` Kiểm A quét
//! `src-tauri/**/*.rs`.

use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

use uuid::Uuid;

use crate::core::cleanup::{CleanupRule, CleanupRuleTier};
use crate::core::i18n::{IpcError, MessageKey};
use crate::core::library::{WorkMeta, create_work_folder, remove_folder};
use crate::core::lifecycle::LifecycleStatus;
use crate::core::scope::load_global_config;
use crate::core::segment::chapterpattern::{ChapterPattern, ChapterPatternKind};
use crate::core::segment::encoding::{
    self, Confidence, EncodingCandidate, EncodingVerdict, NormalizedCandidate,
};
use crate::core::segment::import::{
    ImportError, import_bilingual_file, import_file, import_text, web_import_item_failure_ipc_error,
};
use crate::core::segment::pipeline::{ChapterInput, PipelineInput, PipelineShape, run_import};
use crate::core::store::{Store, StoreSpec, Transaction};
use crate::core::webimport::{self, WebImportItemFailureReason};

/// Tên thư mục con dưới `~/Documents/` — AD-23.
const DOCUMENTS_SUBFOLDER: &str = "AuraTranslate";

/// Tác phẩm đang mở — quản lý trong state của `lib.rs` (Task 7).
///
/// 🔴 Sở hữu `Store`: `Drop` của nó chạy `close()` (TRUNCATE có trần) — thay thế giá trị
/// này trong state (mở một Tác phẩm khác) tự đóng Tác phẩm cũ mà không cần mã dọn dẹp
/// riêng.
#[derive(Debug)]
pub struct OpenWork {
    /// Thư mục `<Tên>.atproj/` trên đĩa.
    pub dir: PathBuf,
    /// Kho `project.db` đang mở.
    pub store: Store,
    /// Tầng Tác phẩm thật của `ScopeResolver` (AC9, nợ `deferred-work.md`) — nắm giữ ở
    /// đây để chỗ gọi sau này (Epic 3+) có sẵn một resolver không phải `global_only`.
    pub scope: crate::core::scope::ScopeResolver,
    /// Metadata vừa tạo/đọc — vỏ IPC trả trường này ra ngoài (`Store` không `Serialize`).
    pub meta: WorkMeta,
    /// 🔵 **THÊM 2026-08-18 (Story 2.11 · FR26 · Quyết định #2 đường (a), Ice ký)** —
    /// `chapter.id` của **Chương đang mở**.
    ///
    /// ─────────────────────────────────────────────────────────────────────────
    /// 🔴 VÌ SAO MỘT TRƯỜNG, VÀ VÌ SAO NÓ Ở **RUST** CHỨ KHÔNG Ở WEBVIEW
    /// ─────────────────────────────────────────────────────────────────────────
    /// Trước story này *"Chương đang mở"* **không được lưu ở đâu cả** — nó được **suy ra
    /// động** mỗi lượt gọi bằng `ORDER BY ord LIMIT 1`, ở **hai** chỗ độc lập
    /// (`commands::chapter::read_open_chapter` và
    /// `commands::segment::read_open_chapter_segments`). Hình dạng đó đúng khi một Tác
    /// phẩm có đúng một Chương và **chỉ** khi đó: ngay khi Chương thứ hai tồn tại, hai câu
    /// SQL kia trả về Chương ĐẦU mãi mãi, và không cổng nào đỏ.
    ///
    /// Đường bị loại và lý do (Quyết định #2, 2026-08-18):
    /// - **(b) webview giữ và truyền qua dây** — đụng AD-1. Câu phải trả lời là *"'Chương
    ///   nào đang mở' là state UI hay một quy tắc nghiệp vụ?"*, và nó là quy tắc: nó quyết
    ///   định **hàng nào trên đĩa** được đọc và ghi.
    /// - **(c) lưu xuống đĩa** — kéo theo một bước di trú cho một nghĩa vụ (AC5/FR12) mà
    ///   Quyết định #4(c) vừa giao **trọn** cho Epic 5.
    ///
    /// 🔵 **SỬA 2026-08-18 (code review ba tầng) — ĐOẠN NÀY TỪNG PHÁT BIỂU MỘT PHÉP ĐO SAI.**
    ///
    /// ~~*"`save_segment_targets`/`flush_segment_targets` nhận `chapter_id` từ webview. Một lô
    /// flush đang bay lúc trường này đổi sẽ mang `chapter_id` CŨ ⇒ Rust trả
    /// `segment.unknown_ids` ⇒ bản dịch biến mất im lặng."*~~
    ///
    /// **Đã đọc lại mã và nó không đúng.** `save_segment_targets` (`segment.rs:1171-1193`) kiểm
    /// `SELECT COUNT(*) FROM chapter WHERE id = ?1` rồi ghi bằng
    /// `UPDATE segment … WHERE id = ?2 AND chapter_id = ?3` — cả hai chạy trên **chính
    /// `project.db` đang mở**, và **không đường nào đọc `OpenWork::chapter_id`**. Khác lượt đổi
    /// **Tác phẩm** *(nơi cả `Store` bị trỏ sang một tệp khác)*, Chương cũ **vẫn còn nguyên
    /// trong cùng CSDL** sau một lượt đổi Chương ⇒ một lô tới trễ mang `chapter_id` cũ **ghi
    /// đúng vào Chương cũ**: `touched == expected`, không `unknown_ids`, không mất chữ.
    ///
    /// ⇒ **Kết luận về thứ tự KHÔNG đổi** *(flush → invoke → dọn → nạp)*, nhưng **lý do đổi**:
    /// nó đúng vì tính nhất quán con trỏ/UI, không vì một đường mất chữ qua `unknown_ids`.
    ///
    /// 🔴 **Và mệnh đề sai ấy phải trả giá, ghi ra để lượt sau đừng lặp:** nó hút hết chú ý về
    /// phía một mối nguy **không tồn tại**, trong khi mối nguy **có thật** — người dùng gõ tiếp
    /// trong cửa sổ giữa lượt `invoke` và lượt `resetEditorPanel()`, rồi `flush.reset()` vứt
    /// chữ ấy vô điều kiện — nằm cách đó sáu dòng và không lượt rà nội bộ nào nhìn. Nó được
    /// đóng ở `panels/editorPanelState.ts::noteEditorEdit`, bằng một cửa khoá gõ.
    pub chapter_id: i64,
    /// **THÊM 2026-09-08 (Story 6.11, FR127)** — số HÀNG `asset` đã chèn lúc [`create_work`]
    /// chạy (đúng bằng `saved_assets.len()`, và đúng bằng số lời gọi `INSERT INTO asset`
    /// thực hiện — hai con số này khoá lẫn nhau bằng cấu trúc, không cần một cổng riêng).
    /// `0` cho MỌI đường khác `create_work` (dán văn bản, tệp, mở lại một `.atproj` đã có) —
    /// chúng không bao giờ chạy pha ảnh.
    ///
    /// 🔵 **SỬA 2026-09-09 (D2 vòng rà đối kháng 3 lớp).** Câu cũ khai "số ảnh đã tải & ghi
    /// thành công xuống `assets/`" — SAI trong ca DEDUP: cùng một URL ảnh xuất hiện ở hai
    /// Chương (hoặc hai lần trong một Chương) chỉ được TẢI và GHI TỆP một lần
    /// (`fetch_and_write_one_asset` chạy đúng một lần mỗi URL, xem cache trong
    /// [`prepare_chapter_images`]), nhưng sinh HAI hàng `asset` (mỗi hàng một neo khác
    /// nhau) ⇒ `images_saved == 2` trong khi số TỆP thật trên đĩa là 1. Trường này đếm HÀNG,
    /// không đếm TỆP; không có API nào ở đây trả số tệp thật, vì bề mặt hiển thị (FR127) chỉ
    /// cần biết bao nhiêu tham chiếu ảnh đã có trong CSDL.
    pub images_saved: u32,
    /// **THÊM 2026-09-08 (Story 6.11, FR127)** — số VỊ TRÍ ảnh GIỮ nhưng KHÔNG có hàng
    /// `asset` (host ngoài tầng 2, MIME không phải ảnh raster, mạng lỗi, neo không tính
    /// được, …) — KHÔNG làm trượt `create_work` (§Design Notes spec 6.11). Bề mặt HIỂN THỊ
    /// con số này là nợ có chủ, chưa dựng ở story này — `deferred-work.md`.
    ///
    /// 🔵 **SỬA 2026-09-09 (vòng rà đối kháng 2, mục D2).** Đếm theo VỊ TRÍ (mỗi khối ảnh
    /// KEPT không cho ra một hàng `asset`), KHÔNG đếm theo LƯỢT GỌI MẠNG — cùng URL ảnh lỗi
    /// (404, timeout, …) dùng lại ở TÁM Chương khác nhau cho `images_failed == 8` dù cache
    /// dedup (`prepare_chapter_images::cache`) chỉ thực hiện ĐÚNG MỘT lượt gọi mạng cho URL
    /// đó (kết quả thất bại cũng được cache — xem doc-comment `CachedFetch`). Đối xứng với
    /// `images_saved` (cũng đếm HÀNG/VỊ TRÍ, không đếm TỆP/LƯỢT TẢI — xem doc-comment ở
    /// trên): cả hai trường đếm ĐÚNG những gì chúng khai — vị trí trong Chương, không phải
    /// hoạt động mạng — nhưng câu mô tả ban đầu của trường này không nói rõ điều đó.
    pub images_failed: u32,
    // 🔵 SỬA 2026-09-08 (mục B1 vòng rà đối kháng 3 lớp) — trường `pending_domain_log` đã BỊ
    // GỠ. Bản trước tích luỹ `Vec<DomainLogEntry>` ở đây, CHỈ nối vào `DomainLogState` trên
    // đường THÀNH CÔNG — một lượt trượt (đĩa đầy giữa lúc ghi ảnh) làm nó biến mất, vi phạm
    // §Always "kể cả lượt trượt". `create_work` nay nhận thẳng `&DomainLogState` và mỗi lời
    // gọi `fetch` push NGAY khi hoàn tất — không còn gì để mà "mang" qua `OpenWork` nữa.
}

/// Thư mục gốc mặc định chứa mọi `.atproj` — `~/Documents/AuraTranslate/` (AD-23).
///
/// Không viết cứng `$HOME` — `app.path().document_dir()` là đường duy nhất (NFR14).
///
/// 🔵 **SỬA 2026-08-27 (Story 5.3) — mệnh đề "module này là nơi DUY NHẤT gọi hàm này" HẾT
/// ĐÚNG.** Bản trước (Story 1.15) đúng: không đường sản phẩm nào cho người dùng ĐỔI thư mục
/// gốc, nên `default_library_root` là điểm phân giải DUY NHẤT. Story 5.3 thêm một khoá
/// `AppConfig` (`library_root`, Story 5.3) đọc TRƯỚC hàm này — [`resolve_library_root`] ngay
/// dưới là bộ phân giải MỚI, và nó là hàm này KHÔNG còn gọi được trực tiếp từ bên ngoài
/// module để tạo/tìm `.atproj`; mọi chỗ gọi SẢN PHẨM (`lib.rs::open_library_index`,
/// `wire::create_work_from_text`/`_from_file`) phải đi qua [`resolve_library_root`], không
/// gọi thẳng hàm này. Hàm này ở lại làm **hồi phòng cuối cùng** của bộ phân giải đó.
pub fn default_library_root(app: &tauri::AppHandle) -> Result<PathBuf, IpcError> {
    use tauri::Manager as _;

    // Móc e2e đứng TRƯỚC `document_dir()` và chỉ tồn tại trong bản debug + feature `wdio`
    // (AD-45). Bản phát hành đi thẳng xuống nhánh dưới.
    //
    // 🔴 Vì sao móc này có mặt TRƯỚC khi tồn tại một bàn đo nào tạo Tác phẩm: bộ e2e dựng
    // một cửa sổ THẬT, nên mọi đường ghi của sản phẩm là một đường ghi vào dữ liệu thật của
    // người chạy. `$APPDATA` đã đóng ở AC2; đây là bề mặt THỨ HAI, tìm ra bằng cách đọc mã
    // chứ không bằng cách mất dữ liệu thêm một lần. Xem `crate::E2E_LIBRARY_ROOT_ENV`.
    if let Some(root) = crate::library_root_override() {
        return Ok(root);
    }

    let documents = app.path().document_dir().map_err(|e| {
        crate::core::library::WorkError::CreateFailed {
            detail: format!("resolve document_dir: {e}"),
        }
    })?;

    Ok(documents.join(DOCUMENTS_SUBFOLDER))
}

/// **THÊM Story 5.3.** Bộ phân giải thư mục gốc Library MỚI — móc e2e ⇒ giá trị người dùng
/// đã cấu hình (`AppConfig::library_root`) ⇒ [`default_library_root`]. **Mọi** chỗ gọi sản
/// phẩm phải đi qua hàm này, không gọi thẳng `default_library_root` — nếu không, một Tác
/// phẩm mới có thể sinh ra ở `~/Documents/AuraTranslate/` trong khi màn hình Library đang
/// hiển thị (và quét) một thư mục gốc KHÁC mà người dùng vừa chọn, một "chỗ rỗng im lặng
/// thứ hai" mà AC5 của story tồn tại để chặn.
///
/// 🔴 **Thứ tự ưu tiên là bất biến (§Always của story) — móc e2e ĐỨNG TRƯỚC giá trị người
/// dùng cấu hình.** Bộ e2e dựng cửa sổ THẬT; nếu một giá trị `library_root` sống sót từ một
/// phiên chạy tay trước đó của người phát triển bị đọc TRƯỚC móc e2e, bộ e2e sẽ ghi vào thư
/// mục Library thật của người chạy — đúng lớp lỗi mà `library_root_override()` tồn tại để
/// chặn cho `default_library_root` (xem doc-comment ở đó).
///
/// `store = None` (kho toàn cục chưa được quản lý) rơi thẳng về [`default_library_root`] —
/// không phải một lỗi, cùng khuôn mọi đường đọc cấu hình khác của kho khi `global.db` không
/// mở được (`AGENTS.md`: "mở kho trượt ⇒ ghi chẩn đoán rồi đi tiếp").
pub fn resolve_library_root(
    app: &tauri::AppHandle,
    store: Option<&Store>,
) -> Result<PathBuf, IpcError> {
    resolve_library_root_from(
        crate::library_root_override(),
        resolve_configured_library_root(store),
        || default_library_root(app),
    )
}

/// 🔵 THÊM (2026-08-27, vòng rà THỨ HAI P2) — **hàm thuần**, tách khỏi `resolve_library_root`
/// đúng khuôn hai lớp của `src-tauri/AGENTS.md` (và đúng nước cờ `apply_chosen_root` đã đi).
///
/// ─────────────────────────────────────────────────────────────────────────────
/// 🔴 VÌ SAO TÁCH — `resolve_library_root` KHÔNG CÓ MỘT PHÉP KIỂM HÀNH VI NÀO
/// ─────────────────────────────────────────────────────────────────────────────
/// Trước bản vá, cả ba nhánh ưu tiên nằm trong MỘT hàm đòi `&tauri::AppHandle` — crate này
/// không có `tauri::test`/`MockRuntime` (`src-tauri/Cargo.toml` không khai `test-utils`), nên
/// không ca nào trong `tests/**` gọi được hàm đó. Cổng quét NGUỒN ở `config_invariants.rs`
/// (vòng rà TRƯỚC) chỉ so THỨ TỰ CHUỖI trong mã — nó không chạy hàm, nên đảo nhánh nào cũng
/// không làm ca nào đỏ. Tách phần LÕI (không đụng `AppHandle`) ra hàm này: `override_root`
/// và `configured` là hai mảnh đã phân giải SẴN, và `default` là một closure chỉ được GỌI
/// khi cả hai vế trên đều vắng mặt — test truyền một closure giả (không chạm `document_dir()`)
/// để phủ được đường "rơi về mặc định" mà không cần `AppHandle` thật.
///
/// **Thứ tự là bất biến (không đổi khi tách):** móc e2e (`override_root`) LUÔN thắng, kể cả
/// khi đã có giá trị cấu hình — bộ e2e dựng cửa sổ THẬT, và một `library_root` sống sót từ
/// một phiên chạy tay trước đó không được phép làm nó ghi vào thư mục Library thật của người
/// chạy.
fn resolve_library_root_from(
    override_root: Option<PathBuf>,
    configured: Option<String>,
    default: impl FnOnce() -> Result<PathBuf, IpcError>,
) -> Result<PathBuf, IpcError> {
    if let Some(root) = override_root {
        return Ok(root);
    }
    if let Some(configured) = configured {
        return Ok(PathBuf::from(configured));
    }
    default()
}

/// 🔵 THÊM (2026-08-27, vòng rà THỨ HAI P2) — **hàm thuần**, nhận thẳng `Option<&Store>`
/// (không `AppHandle`) nên test được với một `Store::open` thật, không cần cửa sổ Tauri.
/// Gom cả BA nhánh mà cổng vòng rà trước không với tới: `store = None` ⇒ `None`; đọc
/// `AppConfig::library_root` trượt (`global.db` hỏng) ⇒ chẩn đoán rồi `None`; đọc thành công
/// nhưng chưa ai cấu hình gì ⇒ `None`; đọc thành công VÀ có cấu hình ⇒ `Some(..)`.
fn resolve_configured_library_root(store: Option<&Store>) -> Option<String> {
    let store = store?;
    match load_global_config(store) {
        Ok(config) => config.library_root(),
        // 🔵 THÊM (2026-08-27, vòng rà bốn lớp P4) — nhánh `Err` trước đây bị NUỐT im lặng
        // (`if let Ok(..) = ..`), nên một `global.db` hỏng làm ứng dụng lặng lẽ rơi về gốc
        // mặc định mà không một dòng nào trong log -- ngược lệ đã ghi của kho ("mở kho trượt
        // ⇒ ghi chẩn đoán rồi đi tiếp", `AGENTS.md`). Chẩn đoán KHÔNG DẤU (NFR16/Kiểm A của
        // `check:i18n`), rồi vẫn rơi về mặc định như cũ -- vế "đi tiếp" không đổi, chỉ thêm
        // vế "nói ra".
        Err(err) => {
            eprintln!(
                "library[root] doc AppConfig::library_root that bai, roi ve mac dinh: {err}"
            );
            None
        }
    }
}

/// Chỗ gọi sản phẩm DUY NHẤT của `run_import` — **THÊM 2026-09-05 (Story 6.5)**.
///
/// ─────────────────────────────────────────────────────────────────────────────
/// 🔴 VÌ SAO MỘT HÀM BỌC, KHÔNG GỌI THẲNG `run_import` Ở BA CHỖ
/// ─────────────────────────────────────────────────────────────────────────────
/// `tests/segment_pipeline_boundary.rs::run_import_is_the_one_product_call_site` đếm
/// LITERAL chuỗi `"run_import("` trong `src-tauri/src/**` (ngoài `core/segment/`) và đòi
/// ĐÚNG MỘT chỗ. Story 6.5 mở nợ `deferred-work.md:9359`: `preview_import_encoding` (mỗi
/// ứng viên VÀ đường tự khai) VÀ `confirm_import_with_encoding` đều phải chạy chuỗi thật —
/// ba lời gọi độc lập sẽ là ba dòng mang chuỗi đó, làm cổng đỏ đúng lúc mệnh đề nó canh
/// ("một chỗ gọi sản phẩm thứ hai không âm thầm truyền một thứ tự khác `PIPELINE_ORDER`")
/// vẫn giữ nguyên. Hàm NÀY là chỗ DUY NHẤT chứa literal đó; mọi nơi khác gọi `run_pipeline`.
fn run_pipeline(
    input: PipelineInput,
) -> Result<crate::core::segment::pipeline::PipelineOutput, ImportError> {
    run_import(input)
}

/// Mẫu phân tách Chương trên dây — Story 6.6 (FR14). `None` ⇒ không mẫu, bước 5 no-op (N=1).
///
/// `kind` là [`ChapterPatternKind`] TRỰC TIẾP (không một hàm `from_wire` viết tay ở lớp vỏ) —
/// cùng khuôn `CleanupRuleTier`/`CleanupRuleKind`: `serde::Deserialize` được khai NGAY trên
/// kiểu core, Tauri giải mã tham số thẳng thành nó.
#[derive(Debug, Clone, serde::Deserialize)]
pub struct ChapterPatternWire {
    pub pattern: String,
    pub kind: ChapterPatternKind,
}

/// Phân giải mẫu phân tách Chương đến từ dây — **hàm thuần**. Mẫu `kind = "regex"` được
/// BIÊN DỊCH THỬ ngay ở đây, TRƯỚC khi `run_pipeline` chạy (§I/O Matrix spec 6.6: "Regex
/// không biên dịch được → Từ chối, xem trước giữ kết quả CŨ, hiện thông báo") — cùng luật mà
/// `core::cleanup::store::validate_pattern` đã theo cho luật làm sạch: lưu/dùng một mẫu hỏng
/// KHÔNG BAO GIỜ tới được `run_pipeline`, một cổng biên dịch riêng ở đó là thừa.
pub fn resolve_chapter_pattern(
    wire: Option<ChapterPatternWire>,
) -> Result<Option<ChapterPattern>, ImportError> {
    let Some(wire) = wire else { return Ok(None) };
    if wire.kind == ChapterPatternKind::Regex {
        if let Err(err) = crate::core::segment::chapterpattern::compile(&wire.pattern) {
            return Err(ImportError::InvalidChapterPattern { detail: err.to_string() });
        }
    }
    Ok(Some(ChapterPattern { kind: wire.kind, pattern: wire.pattern }))
}

/// **Hàm thuần** — tạo một Tác phẩm mới trên đĩa từ một [`PipelineShape`] đã có sẵn.
///
/// Thứ tự: dựng thư mục (`core::library::atproj`) → mở `project.db`
/// (`StoreSpec::project`) → chạy TRỌN chuỗi pipeline AD-39 (`run_import`, bước 1-7) →
/// **ghi** hàng `work` + N hàng `chapter` (+ segment của mỗi Chương) trong MỘT giao dịch →
/// dựng lại `meta.json` từ `project.db` vừa commit (Quyết định #3, AD-33) → ghi `meta.json`
/// nguyên tử NGAY SAU giao dịch. Bất kỳ bước nào trượt ⇒ dọn thư mục, không để lại
/// `.atproj/` nửa vời (AC8).
///
/// 🔵 **SỬA 2026-09-04 (Story 6.2, AD-39) — nhận `PipelineShape`, không còn `ImportedChapter`
/// đơn lẻ; ghi N Chương, không còn đúng một.** 🔵 **SỬA 2026-09-05 (Story 6.6) — "N = 1 trên
/// đường sản phẩm hôm nay" đã HẾT ĐÚNG.** Hàm nay nhận thêm tham số `chapter_pattern`
/// (`Option<ChapterPattern>`) và N > 1 là một kết quả THẬT trên đường sản phẩm khi người
/// dùng cấu hình một mẫu khớp được nhiều lần — đường đi đã tổng quát từ Story 6.2, không cần
/// sửa lại hàm này lần nữa.
///
/// 🔵 **SỬA 2026-09-04 (Story 6.3) — thêm tham số `encoding`, ĐIỂM TIÊM DUY NHẤT của bảng
/// mã đã chọn/đã dò.** Trước story này hàm luôn khai UTF-8 cứng qua
/// [`PipelineInput::default_shaped`]; giờ chỗ gọi CHỌN [`PipelineInput::with_encoding`] hay
/// `default_shaped` — `create_work_from_text`/`create_work_from_file` (không đổi chữ ký,
/// dùng bởi `tests/**` và đường sản phẩm KHÔNG đi qua xem trước bảng mã) truyền
/// [`encoding_rs::UTF_8`]; `wire::confirm_import_with_encoding` (Story 6.3, đường CÓ xem
/// trước) truyền bảng mã người dùng đã xác nhận. Đây VẪN là chỗ gọi [`run_import`] DUY NHẤT
/// của cả crate (`segment_pipeline_boundary.rs::run_import_is_the_one_product_call_site`)
/// — không một chỗ gọi thứ hai nào được mở (§Always spec 6.3).
///
/// # Lỗi
/// - dựng thư mục trượt ⇒ `project.create_failed`;
/// - chuỗi pipeline trượt (ví dụ byte không hợp lệ với bảng mã ĐÃ CHỌN) ⇒ lỗi nhập
///   (`import.*`), qua `From<ImportError>`;
/// - mở/ghi `project.db` trượt ⇒ lỗi kho (`store.*`), qua `From<StoreError>`.
///
/// 🔴 **THÊM 2026-09-07 (Story 6.9) — tham số `block_overrides`, đúng cái vòng rà 1 đã hụt.**
/// Trạng thái giữ/loại người dùng đặt bằng `Space`/`[`/`]` ở tầng 2 PHẢI đi tới đây — đây là
/// đường DUY NHẤT ghi Chương xuống `.atproj` (doc-comment ở trên). Không truyền tham số này
/// (hoặc truyền `Vec::new()` một cách sai) làm đĩa nhận phán đoán MÁY trong khi màn hình đã
/// hiện sửa tay — `wire::confirm_import_with_encoding` là chỗ gọi PHẢI đọc
/// `Tier2BlockOverridesState` rồi truyền NGUYÊN VẸN vào đây, reset state đó SAU KHI ghi
/// xong (không phải TRƯỚC — một lượt xác nhận trượt giữ nguyên override để thử lại).
/// 🔴 **THÊM 2026-09-08 (Story 6.11, mục B1 vòng rà đối kháng 3 lớp) — tham số `domain_log_state`.**
/// Trước bản sửa này, pha ảnh tích luỹ `Vec<DomainLogEntry>` cục bộ rồi CHỈ gắn nó vào
/// `OpenWork::pending_domain_log` trên đường THÀNH CÔNG — một lượt nhập trượt (ví dụ đĩa đầy
/// giữa lúc ghi ảnh thứ N) trả `Err` sớm, và không kiểu `Result<OpenWork, IpcError>` nào chở
/// được một `Vec<DomainLogEntry>` kèm theo nhánh `Err`, nên nhật ký của N-1 ảnh ĐÃ tải thành
/// công trước đó biến mất — vi phạm đúng chữ §Always spec 6.11 *"kể cả lượt trượt"*. Sửa bằng
/// cách PUSH THẲNG vào `domain_log_state` ngay khi mỗi lời gọi `fetch` hoàn tất (thành công
/// hay không), thay vì tích luỹ cục bộ rồi trả về SAU CÙNG — một khi đã push, entry đó SỐNG
/// SÓT bất kể phần còn lại của `create_work` có trượt hay không. Test không cần domain log
/// bền qua lượt trượt (đa số) truyền `&Mutex::new(Vec::new())` — một kho tạm, vứt đi sau ca.
pub fn create_work(
    documents_root: &Path,
    name: &str,
    source_lang: &str,
    genre: &str,
    shape: PipelineShape,
    encoding: &'static encoding_rs::Encoding,
    cleanup_rules: Vec<crate::core::cleanup::CleanupRule>,
    chapter_pattern: Option<ChapterPattern>,
    block_overrides: Vec<Option<bool>>,
    // 🔴 **THÊM 2026-09-11 (Story 6.16, FR115)** — vai cột + cờ tiêu đề của đường nhập song
    // ngữ. Tham số MỖI LƯỢT NHẬP, cùng khuôn `chapter_pattern` (không một state thứ ba —
    // §Boundaries spec 6.16). Vô nghĩa (không đọc) khi `shape` không phải
    // `PipelineShape::Bilingual`.
    bilingual_source_column: usize,
    bilingual_target_column: usize,
    bilingual_has_header: bool,
    // 🔴 **THÊM 2026-09-10 (Story 6.15)** — xuất xứ NGƯỜI DÙNG đã gõ đè, theo CHỈ SỐ Chương của
    // lượt nhập này — xem doc-comment [`ChapterOriginOverridesState`]. `&[]` (mọi chỗ gọi
    // KHÔNG đi qua màn xem trước xuất xứ — `tests/**` cũ, `create_work_from_text`/`_from_file`)
    // là "chưa ai sửa gì", CÙNG hành vi trước story này.
    origin_overrides: &[Option<ChapterOriginOverride>],
    domain_log_state: &webimport::DomainLogState,
    // 🔴 **THÊM 2026-09-09 (Story 6.12)** — khối + ảnh nhúng của một `.docx`, đọc SẴN bởi
    // `import_file` (nó không đi qua `Step::ExtractMainContent`, đó là bóc HTML). `None` cho
    // mọi đường khác (dán tay, `.txt`/`.md`, URL). Xem doc-comment
    // [`crate::core::segment::import::DocxSidecar`] cho giới hạn "chỉ Chương đầu tiên".
    docx_sidecar: Option<crate::core::segment::import::DocxSidecar>,
) -> Result<OpenWork, IpcError> {
    let dir = create_work_folder(documents_root, name)?;

    let db_path = dir.join("project.db");
    let store = match Store::open(StoreSpec::project(db_path)) {
        Ok(store) => store,
        Err(err) => {
            remove_folder(&dir);
            return Err(err.into());
        }
    };

    let work_id = Uuid::new_v4().to_string();
    let name_owned = name.to_owned();
    let source_lang_owned = source_lang.to_owned();
    let genre_owned = genre.to_owned();

    // 🔴 AD-39 — TOÀN BỘ chuỗi bảy bước chạy **ở đây, một lần, lúc nhập**, và **NGOÀI**
    // closure ghi bên dưới, có chủ ý — cùng lý do Quyết định #3 cũ của Story 1.15 vẫn giữ:
    // AD-11 giữ **một** writer duy nhất nối tiếp (một `Connection` `move` vào một thread,
    // job đi qua `mpsc::channel`), nên thời gian CPU bên trong closure **chặn mọi lượt ghi
    // khác của tiến trình**. Một Chương dài đi qua chuỗi trong closure là một lượt khoá
    // hàng đợi ghi mà auto-save của Editor (NFR2) phải xếp sau.
    //
    // 🔵 SỬA 2026-09-05 (Story 6.6) — "chapter_pattern: None" (Never clause của spec 6.2) đã
    // HẾT ĐÚNG: `chapter_pattern` giờ là tham số THẬT của chính `create_work`, tới từ màn
    // xem trước nhập. `None` vẫn hợp lệ (không mẫu ⇒ bước 5 no-op, N = 1) — chỉ không còn là
    // giá trị DUY NHẤT có thể tới đây.
    //
    // 🔵 SỬA 2026-09-04 (Story 6.3) — `with_encoding`, không còn `default_shaped` cứng
    // UTF-8: `encoding` giờ là tham số của chính `create_work` (xem doc-comment hàm này).
    // 🔵 SỬA 2026-09-05 (Story 6.5) — qua `run_pipeline` (không gọi `run_import` thẳng ở
    // đây nữa — xem doc-comment của hàm đó), cộng `cleanup_rules` đã phân giải.
    //
    // 🔴 **THÊM 2026-09-06 (Story 6.7)** — `extract_main_content` chốt vào HÌNH DẠNG đầu
    // vào, không phải một tham số riêng của `create_work`. `Chapters(RawBytes, ..)` — byte
    // HTML CHƯA giải mã — là hình dạng DUY NHẤT mà danh sách URL (Story 6.7) dựng ra; không
    // đường sản phẩm nào khác (dán tay, tệp) từng tạo ra nó. 🔴 **Điều kiện KHÔNG ĐƠN GIẢN
    // là "shape là `Chapters`"** — `tests/project_contract.rs::create_work_writes_every_chapter_and_its_segments_when_the_pipeline_yields_more_than_one`
    // dựng tay một `Chapters(AlreadyText, ..)` từ TRƯỚC story này để kiểm thuần cơ chế ghi N
    // Chương, không liên quan gì tới web — một điều kiện chỉ nhìn biến thể `Chapters` sẽ gọi
    // `webimport::extract("Chuong mot...", "")` (nhãn RỖNG, không phải URL tuyệt đối) và làm
    // ca đó ĐỎ OAN. Đúng điều kiện: hình dạng LÀ `Chapters` VÀ đơn vị ĐẦU là `RawBytes` (byte
    // thô CHƯA giải mã — dấu hiệu THẬT của "đến từ mạng", `AlreadyText` không bao giờ cần
    // bóc, dù đứng trong hình dạng nào).
    // ⚠️ **NỢ CÓ CHỦ, ghi ra tại chỗ (vòng rà đối kháng 2, mục D6).** Điều kiện dưới đây đọc
    // MỘT MÌNH `cs.first()` — nếu MỘT `PipelineShape::Chapters` nào đó lỡ TRỘN hình dạng (mục
    // đầu `AlreadyText`, mục SAU `RawBytes`), `extract_main_content` tắt cho TOÀN BỘ danh
    // sách và những mục `RawBytes` phía sau KHÔNG BAO GIỜ được bóc nội dung/tìm ảnh — không
    // panic, không lỗi, `images_failed` vẫn `0` (im lặng, không phải một lượt "thử rồi
    // trượt"). ĐO ĐƯỢC: hai bộ dựng SẢN PHẨM DUY NHẤT của `Chapters` hôm nay
    // (`chapters_shape_if_all_ok`/`chapters_shape_for_view`) luôn dựng một danh sách ĐỒNG
    // NHẤT (toàn `RawBytes`, từ `UrlImportItem` đã tải) —
    // 🔵 SỬA 2026-09-09 (vòng rà đối kháng 3, mục T6): câu trước dẫn `homogeneity_boundary.rs
    // (nếu có)` — tệp đó KHÔNG tồn tại trong `src-tauri/tests/`, một đoạn khai ĐO ĐƯỢC mà
    // mang "(nếu có)" là tự thú chưa kiểm. Test THẬT khoá bất biến này là
    // `webimport_contract.rs::the_two_real_chapters_shape_builders_always_produce_a_homogeneous_list_of_raw_bytes`.
    // Đường KHÁC (test dựng tay `Chapters(AlreadyText, ..)`, xem chú thích trên) cũng đồng
    // nhất. Vì thế điều kiện SAI này chưa từng bị kích hoạt trên đường thật — nhưng
    // `run_import`/`PipelineInput` là một seam CÔNG KHAI, và không gì ở KIỂU ngăn một chỗ gọi
    // tương lai trộn hình dạng. **Chủ: Ice** — sửa đúng cần `extract_main_content` chuyển
    // thành MỘT QUYẾT ĐỊNH THEO TỪNG Chương
    // (không phải một cờ toàn cục), một thay đổi cấu trúc lớn hơn phạm vi lượt vá nhỏ này;
    // xem `deferred-work.md`.
    let extract_main_content = matches!(
        &shape,
        PipelineShape::Chapters(cs) if matches!(cs.first(), Some(ChapterInput::RawBytes { .. }))
    );

    // 🔴 **THÊM 2026-09-08 (Story 6.11, FR127)** — trích URL trang của TỪNG đơn vị TRƯỚC khi
    // `shape` bị `run_pipeline` (ngay dưới) tiêu thụ (Code Map spec 6.11: "URL trang ... đọc
    // được từ `shape` TRƯỚC KHI nó bị run_import nuốt"). Chỉ có ý nghĩa khi `extract_main_content`
    // (đường URL, ảnh cần URL trang để phân giải `src` tương đối) — đường Blob/AlreadyText cho
    // ra một danh sách một phần tử KHÔNG bao giờ được `prepare_chapter_images` đọc tới (nó
    // `continue` ngay ở Chương có `blocks: None`).
    let chapter_urls: Vec<String> = match &shape {
        PipelineShape::Blob(c) => vec![chapter_input_page_url(c)],
        PipelineShape::Chapters(cs) => cs.iter().map(chapter_input_page_url).collect(),
        // 🔴 **THÊM 2026-09-11 (Story 6.16)** — cùng lý do nhánh `Blob` ngay trên: đường song
        // ngữ KHÔNG BAO GIỜ bóc nội dung chính (`extract_main_content` luôn `false` cho hình
        // dạng này — nó không phải `PipelineShape::Chapters`), nên `prepare_chapter_images`
        // không bao giờ đọc tới danh sách này; một phần tử rỗng không mất gì.
        PipelineShape::Bilingual { .. } => vec![String::new()],
    };
    // `cleanup_rules`/`block_overrides` bị DI CHUYỂN vào `PipelineInput` ngay dưới — pha ảnh
    // (sau khi chuỗi chạy xong) cần lại đúng hai giá trị này để tính neo (bước 3/4 AD-39 lặp
    // lại trên TIỀN TỐ, xem `core::segment::anchor::compute_anchor`) và để biết Chương nào
    // đọc `block_overrides` (chỉ Chương đầu — cùng luật `Step::ExtractMainContent`), nên clone
    // TRƯỚC khi di chuyển, không sau.
    let cleanup_rules_for_images = cleanup_rules.clone();
    let block_overrides_for_images = block_overrides.clone();
    // 🔴 THÊM 2026-09-10 (Story 6.15) — bản CHÉP thành `Vec` sở hữu, `move` được vào closure
    // ghi `'static` bên dưới (`origin_overrides` tham số chỉ là `&[..]`, không sống đủ lâu).
    let origin_overrides_owned: Vec<Option<ChapterOriginOverride>> = origin_overrides.to_vec();

    let outcome = match run_pipeline(
        PipelineInput::with_encoding(shape, encoding, source_lang_owned.clone())
            .with_cleanup_rules(cleanup_rules)
            .with_chapter_pattern(chapter_pattern)
            .with_extract_main_content(extract_main_content)
            .with_block_overrides(block_overrides)
            .with_bilingual_columns(bilingual_source_column, bilingual_target_column, bilingual_has_header),
    ) {
        Ok(outcome) => outcome,
        Err(err) => {
            store.close();
            remove_folder(&dir);
            return Err(err.into());
        }
    };
    // 🔴 **THÊM 2026-09-11 (Story 6.16) — từ chối Ở RUST, không chỉ ở nút webview.** §Boundaries:
    // "Mismatched row ⇒ confirm is locked... Rust-side, not only UI". `create_work` là điểm
    // gọi DUY NHẤT ghi `.atproj` (doc-comment đầu tệp) — canh Ở ĐÂY giữ đúng "không hàng nào
    // ghi được khi còn lệch cặp" cho MỌI chỗ gọi, không riêng `confirm_bilingual_import`.
    if !outcome.bilingual_mismatches.is_empty() {
        let count = outcome.bilingual_mismatches.len();
        store.close();
        remove_folder(&dir);
        return Err(crate::core::segment::import::ImportError::BilingualMismatchedRows { count }.into());
    }
    let mut chapters = outcome.chapters;

    // 🔴 **THÊM 2026-09-09 (Story 6.12)** — khối + ảnh `.docx` gắn vào Chương ĐẦU TIÊN ở
    // đây, NGAY SAU khi chuỗi bảy bước đã ổn định số Chương. `.docx` không đi qua
    // `Step::ExtractMainContent` (đó là bóc HTML qua `dom_smoothie`, `.docx` không phải
    // HTML) nên `chapter.blocks` vẫn `None` tới đây cho MỌI đường `.docx` — đây là chỗ DUY
    // NHẤT nó được gán. Cùng giới hạn "chỉ Chương đầu" mà `block_overrides` đã theo cho
    // đường URL (Story 6.9) — xem doc-comment `DocxSidecar`.
    if let Some(sidecar) = &docx_sidecar {
        if let Some(first) = chapters.first_mut() {
            first.blocks = Some(sidecar.blocks.clone());
        }
    }

    // 🔴 THÊM 2026-09-09 (D10 vòng rà đối kháng 3 lớp) — `chapter_urls` (dựng TRƯỚC
    // `run_pipeline`, một phần tử mỗi ĐƠN VỊ đầu vào, xem trên) và `chapters` (SAU khi bảy
    // bước AD-39 chạy) được `prepare_chapter_images` giả định 1:1 THEO CHỈ SỐ
    // (`chapter_urls.get(i)`) — không một dòng nào từng khẳng định điều đó trước sửa này.
    // Giả định ĐÚNG khi `extract_main_content` bật: điều kiện đó buộc `shape` từng là
    // `PipelineShape::Chapters` (khối `matches!` phía trên), tức `already_chaptered = true`,
    // khiến `split_chapters_step` (`core::segment::pipeline`) return SỚM, không đổi số
    // Chương. Đường `Blob` CÓ THỂ nổ ra N > 1 Chương thật (Story 6.6, mẫu phân tách) trong
    // khi `chapter_urls.len() == 1` — nhưng nhánh đó không bao giờ chạy pha ảnh
    // (`extract_main_content = false` ⇒ mọi `chapter.blocks` đều `None`, `prepare_chapter_images`
    // `continue` ngay), nên bất biến chỉ cần đúng trong ĐÚNG nhánh sẽ thật sự đọc
    // `chapter_urls`. Nếu nó SAI, `unwrap_or("")` ở `prepare_chapter_images` khiến mọi `src`
    // tương đối trượt ÂM THẦM, không phân biệt được với "trang không có ảnh" (D10) — một lỗi
    // LẬP TRÌNH thật (không phải điều kiện người dùng có thể gây ra), bắt tại nguồn.
    //
    // 🔵 SỬA (vòng rà đối kháng 2, mục B1) — `assert_eq!` ĐỔI THÀNH `IpcError` + dọn
    // `.atproj`, không còn panic trần. `Cargo.toml` đặt `panic = "abort"` (bán kính nổ TOÀN
    // TIẾN TRÌNH — cùng lý lẽ chú thích "vòng rà đối kháng 2026-09-04, item 4" ngay dưới đây),
    // và tại DÒNG NÀY `create_work_folder`/`Store::open` đã chạy — một panic ở đây giết cả
    // ứng dụng NGƯỜI DÙNG ĐANG DÙNG và để lại một `.atproj` nửa vời, đúng trạng thái mà MỌI
    // nhánh `Err` khác của hàm này dùng `remove_folder` để tránh. Cùng khuôn hai kiểm tra
    // `chapters.is_empty()`/`i64::try_from` ngay dưới đây.
    if extract_main_content && chapters.len() != chapter_urls.len() {
        store.close();
        remove_folder(&dir);
        return Err(crate::core::library::WorkError::CreateFailed {
            detail: format!(
                "bat bien 1:1 giua chapters ({}) va chapter_urls ({}) da vo -- loi lap trinh, \
                 khong phai dieu kien nguoi dung gay ra",
                chapters.len(),
                chapter_urls.len()
            ),
        }
        .into());
    }

    // 🔴 SỬA (vòng rà đối kháng 2026-09-04, item 4) — bán kính nổ của một `.expect()` bên
    // TRONG closure ghi là TOÀN TIẾN TRÌNH: `panic = "abort"` giết ngay khi giao dịch đang
    // mở, không unwind, không rollback. Cả hai bất biến dưới đây được validate ở NGOÀI
    // closure, TRƯỚC khi giao dịch mở.
    // 🔵 SỬA 2026-09-05 (Story 6.6) — "chuỗi sản phẩm luôn N = 1" đã HẾT ĐÚNG (mẫu phân tách
    // cho N > 1 thật trên đường sản phẩm). Hai bất biến dưới vẫn kiểm ở đây vì lý do KHÁC,
    // không đổi: `run_import` là một seam CÔNG KHAI (`PipelineShape::Chapters` cho phép N
    // tuỳ ý), và một Chương RỖNG hay N vượt `i64` là ĐIỀU KIỆN BIÊN của chính N > 1, không
    // phải một điều kiện chỉ tồn tại lúc chuỗi còn luôn N = 1.
    if chapters.is_empty() {
        store.close();
        remove_folder(&dir);
        return Err(crate::core::library::WorkError::CreateFailed {
            detail: "pipeline nhap tra ve 0 Chuong -- khong co gi de ghi".to_owned(),
        }
        .into());
    }
    if i64::try_from(chapters.len()).is_err() {
        store.close();
        remove_folder(&dir);
        return Err(crate::core::library::WorkError::CreateFailed {
            detail: format!("so Chuong ({}) vuot i64 -- khong the ghi cot ord", chapters.len()),
        }
        .into());
    }

    // 🔴 **THÊM 2026-09-08 (Story 6.11, FR127)** — pha ảnh chạy Ở ĐÂY: sau khi chuỗi bảy bước
    // đã ổn định `chapter.source_text`/`segments` (cần cho `anchor::compute_anchor`), TRƯỚC
    // giao dịch ghi SQL bên dưới. Ghi TỆP xảy ra ở đây, NGOÀI closure `store.write` (khuôn
    // `Meta::write_atomic` — job ghi bên dưới CHỈ SQL); một ảnh trượt tải KHÔNG dừng hàm này (đếm
    // vào `images_failed`, log chẩn đoán), nhưng ghi BYTE trượt giữa chừng (đĩa đầy) THÌ
    // dừng — cùng khuôn mọi lỗi khác của hàm này (dọn `.atproj` nửa vời, AC8).
    let docx_images: &[crate::core::docx::DocxImage] =
        docx_sidecar.as_ref().map(|s| s.images.as_slice()).unwrap_or(&[]);
    let image_prep = match prepare_chapter_images(
        &dir,
        &chapters,
        &chapter_urls,
        &block_overrides_for_images,
        &cleanup_rules_for_images,
        &source_lang_owned,
        domain_log_state,
        docx_images,
    ) {
        Ok(prep) => prep,
        Err(err) => {
            store.close();
            remove_folder(&dir);
            return Err(err);
        }
    };

    // 🔴 **THÊM 2026-09-08 (Story 6.11)** — tách `image_prep` thành các trường rời TRƯỚC
    // closure `move` bên dưới: `saved` di chuyển vào closure (ghi `INSERT INTO asset`, CÙNG
    // giao dịch với `chapter`/`segment` — khuôn `atomic asset row` của spec 6.11); hai trường
    // còn lại ở lại đây để lắp vào `OpenWork` SAU khi giao dịch commit. `domain_log` KHÔNG
    // còn là một trường ở đây (mục B1) — mỗi lời gọi `fetch` đã PUSH THẲNG vào
    // `domain_log_state` ngay khi hoàn tất, nên nó sống sót cả trên đường trượt phía trên.
    let ImagePrepOutcome { saved: mut saved_assets, images_saved, images_failed } = image_prep;

    // 🔴 **THÊM 2026-09-09 (Story 6.13)** — dệt segment vai (Quyết định 1/2 spec 6.13) NGAY Ở
    // ĐÂY: `chapter.segments` đã ổn định (bước 7 AD-39, kể cả lượt ép ranh giới caption của
    // `role::force_caption_segment_boundaries` chạy TRONG chuỗi), và `prepare_chapter_images`
    // vừa xong (mọi neo TRƯỚC-KHI-DỆT đã tính). Đây là chỗ DUY NHẤT có cả `chapter.segments`
    // lẫn `saved_assets` — dệt xong TRƯỚC `store.write` để dãy segment cuối cùng và neo ĐÃ DỜI
    // đi vào CÙNG giao dịch, không một đường ghi thứ hai.
    //
    // 🔴 `docx_sidecar.is_some()` ⇒ KHÔNG dệt gì (Quyết định 3 spec 6.13: "Chỉ mô hình +
    // đường web"). `.docx` gắn `blocks` vào Chương đầu (ở trên, "THÊM 2026-09-09 Story 6.12")
    // nhưng đó là ảnh nhúng tài liệu, không phải bề mặt mà spec 6.13 mở — vế `.docx` là nợ có
    // chủ (Ice), ghi ở `deferred-work.md`.
    let weave_this_import = docx_sidecar.is_none();
    let mut final_segments: Vec<Vec<crate::core::segment::role::WovenSegment>> =
        Vec::with_capacity(chapters.len());
    // `(chapter_index, block_index) -> neo ĐÃ DỜI`, chỉ với những Chương THẬT SỰ được dệt.
    let mut shifted_anchors: std::collections::HashMap<(usize, usize), i64> =
        std::collections::HashMap::new();
    for (i, chapter) in chapters.iter().enumerate() {
        match &chapter.blocks {
            Some(blocks) if weave_this_import => {
                // Cùng luật `Step::ExtractMainContent`/`prepare_chapter_images` — chỉ Chương
                // ĐẦU TIÊN đọc `block_overrides` (§Never spec 6.9, kế thừa nguyên vẹn).
                let overrides_for_unit: &[Option<bool>] =
                    if i == 0 { &block_overrides_for_images } else { &[] };
                let effective_kept = crate::core::segment::pipeline::effective_kept_for_blocks(
                    blocks,
                    overrides_for_unit,
                );
                let woven = crate::core::segment::role::weave_chapter_segments(
                    blocks,
                    &effective_kept,
                    &chapter.segments,
                    &chapter.source_text,
                    &cleanup_rules_for_images,
                    &source_lang_owned,
                );
                for (block_index, anchor) in woven.shifted_anchor_by_block {
                    shifted_anchors.insert((i, block_index), anchor);
                }
                final_segments.push(woven.segments);
            }
            // Đường KHÔNG có `blocks` (`.txt`, dán tay) HOẶC đường `.docx` (có `blocks` nhưng
            // Quyết định 3 loại nó khỏi lượt dệt) — mọi hàng `role = NULL`, KHÔNG một byte nào
            // đổi so với trước story này (§Always spec 6.13).
            _ => {
                final_segments.push(
                    chapter
                        .segments
                        .iter()
                        .cloned()
                        .map(crate::core::segment::role::WovenSegment::from)
                        .collect(),
                );
            }
        }
    }
    // Neo của MỌI `SavedAsset` vốn tính TRƯỚC KHI DỆT — cập nhật lại bằng neo ĐÃ DỜI trước khi
    // ghi `INSERT INTO asset`. Chỉ ảnh thuộc một Chương THẬT SỰ được dệt (và tính neo thành
    // công ở CẢ HAI lượt tính, `prepare_chapter_images` lẫn `weave_chapter_segments`) mới có
    // mặt trong `shifted_anchors`; đường `.docx`/không `blocks` không đổi gì (neo giữ nguyên).
    for saved in &mut saved_assets {
        if let Some(&shifted) = shifted_anchors.get(&(saved.chapter_index, saved.block_index)) {
            saved.anchor_after_segment_ord = shifted;
        }
    }
    let final_segments = final_segments;
    let saved_assets = saved_assets;

    // 🔴 Quyết định #3: job ghi CHỈ SQL — không `fs::write` nào bên trong closure này.
    let write_result = store.write(move |tx: &Transaction<'_>| {
        tx.execute(
            "INSERT INTO work (id, work_id, name, source_lang, genre, created_at, updated_at) \
             VALUES (1, ?1, ?2, ?3, ?4, strftime('%Y-%m-%dT%H:%M:%fZ','now'), \
             strftime('%Y-%m-%dT%H:%M:%fZ','now'))",
            (&work_id, &name_owned, &source_lang_owned, &genre_owned),
        )?;

        // 🔴 AC13 (không đổi) — segment ghi xuống **CÙNG** giao dịch với hàng `chapter`
        // sinh ra chúng. Segment đã tính SẴN trong `chapter.segments` (bước 7 của chuỗi,
        // chạy trong `run_import` NGOÀI closure này) — không tính lại ở đây.
        //
        // 🔵 SỬA 2026-09-05 (Story 6.6) — "N = 1 ở story này" đã HẾT ĐÚNG: `ord` liên tục từ
        // 1, cùng giao dịch với hàng `work`, cho N THẬT (mẫu phân tách có thể khớp nhiều
        // lần). `OpenWork::chapter_id` chốt vào Chương ĐẦU TIÊN — Story 2.11 xoá hẳn lối
        // suy-ra-động (`ORDER BY ord LIMIT 1`).
        //
        // 🔴 KHÔNG `Option<i64>` + `.expect()` — `chapters` đã được xác nhận KHÔNG RỖNG
        // và `chapters.len()` đã được xác nhận VỪA `i64` ở NGOÀI closure này (vòng rà đối
        // kháng 2026-09-04, item 4). `is_first`/`i as i64` vì thế an toàn theo CẤU TRÚC,
        // không phải theo linh cảm "thực tế không xảy ra".
        let mut first_chapter_id: i64 = 0;
        let mut is_first = true;
        for (i, chapter) in chapters.iter().enumerate() {
            let ord = i as i64 + 1;
            // 🔵 SỬA 2026-09-05 (Story 6.6) — `title` bơm từ `chapter.title` (dòng khớp mẫu
            // phân tách, `None` cho Chương lời tựa hoặc khi không có mẫu) thay vì `NULL`
            // cứng.
            //
            // 🔴 THÊM 2026-09-10 (Story 6.15, FR128/AD-43) — bốn cột xuất xứ, ÁP override
            // NGƯỜI DÙNG (nếu có, theo chỉ số Chương `i`) lên trên giá trị MÁY đã bóc
            // (`chapter.origin`) — xem [`effective_origin_fields`]. Đường tệp/dán tay có
            // `chapter.origin == None` VÀ `origin_overrides` rỗng ⇒ cả bốn cột `NULL`, KHÔNG
            // BACKFILL, đúng §Always.
            let (origin_author, origin_site_name, origin_url, origin_published_at) =
                effective_origin_fields(chapter.origin.as_ref(), origin_overrides_owned.get(i).and_then(Option::as_ref));
            // 🔴 **THÊM 2026-09-11 (Story 6.16, §Always)** — Chương của đường song ngữ khởi
            // tạo `InProgress`, không `NotStarted`: nó tới với bản dịch SẴN CÓ (dù chưa xác
            // nhận), khác một Chương văn xuôi vừa nhập chưa ai chạm tới.
            let chapter_status = if chapter.bilingual_segments.is_some() {
                LifecycleStatus::InProgress
            } else {
                LifecycleStatus::NotStarted
            };
            tx.execute(
                "INSERT INTO chapter (ord, title, source_text, status, created_at, updated_at, \
                 origin_author, origin_site_name, origin_url, origin_published_at) \
                 VALUES (?1, ?2, ?3, ?4, strftime('%Y-%m-%dT%H:%M:%fZ','now'), \
                 strftime('%Y-%m-%dT%H:%M:%fZ','now'), ?5, ?6, ?7, ?8)",
                (
                    ord,
                    &chapter.title,
                    &chapter.source_text,
                    chapter_status.as_str(),
                    &origin_author,
                    &origin_site_name,
                    &origin_url,
                    &origin_published_at,
                ),
            )?;

            // `last_insert_rowid()` đọc **trong** giao dịch, ngay sau lượt chèn của chính
            // nó — `Store::write` giữ một writer duy nhất nối tiếp, nên không lượt chèn
            // nào khác chen được vào giữa hai dòng này.
            let chapter_id = tx.last_insert_rowid();
            // 🔴 **THÊM 2026-09-11 (Story 6.16, AD-47 ③)** — đường song ngữ ghi `target_text`
            // + `translation_origin = bilingual_import` trong CÙNG một `INSERT`, qua hàm
            // RIÊNG (§Always: "existing path unchanged") — không đi qua dệt vai (Quyết định 3
            // spec 6.13 loại `.docx`; đường này CŨNG không có `blocks` để mà dệt).
            match &chapter.bilingual_segments {
                Some(segments) => {
                    crate::commands::segment::insert_bilingual_segments(tx, chapter_id, segments)?;
                }
                None => {
                    crate::commands::segment::insert_segments(tx, chapter_id, &final_segments[i])?;
                }
            }

            // 🔴 **THÊM 2026-09-08 (Story 6.11, FR127)** — hàng `asset` của CHÍNH Chương này,
            // CÙNG giao dịch với `chapter`/`segment` (§Always spec 6.11: "ghi SQL đi qua
            // `store::Writer` như mọi lệnh ghi khác"). Tệp đã nằm trên đĩa từ TRƯỚC giao dịch
            // này (`prepare_chapter_images`, NGOÀI closure) — ở đây chỉ còn việc ghi HÀNG.
            for saved in saved_assets.iter().filter(|s| s.chapter_index == i) {
                tx.execute(
                    "INSERT INTO asset (chapter_id, file_name, source_url, \
                     anchor_after_segment_ord, byte_len, content_type, created_at) \
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6, strftime('%Y-%m-%dT%H:%M:%fZ','now'))",
                    (
                        chapter_id,
                        &saved.file_name,
                        &saved.source_url,
                        saved.anchor_after_segment_ord,
                        saved.byte_len,
                        &saved.content_type,
                    ),
                )?;
            }

            if is_first {
                first_chapter_id = chapter_id;
                is_first = false;
            }
        }

        Ok(first_chapter_id)
    });

    let chapter_id = match write_result {
        Ok(chapter_id) => chapter_id,
        Err(err) => {
            store.close();
            remove_folder(&dir);
            return Err(err.into());
        }
    };

    // Quyết định #3: `meta.json` ghi NGAY SAU KHI giao dịch commit, ở tầng THAO TÁC —
    // dựng lại từ `project.db` vừa ghi (AD-33), không giữ dữ liệu song song mà trôi.
    let meta = match WorkMeta::rebuild_from_store(&store) {
        Ok(meta) => meta,
        Err(err) => {
            store.close();
            remove_folder(&dir);
            return Err(err.into());
        }
    };

    // 🔴 Loi ghi meta.json PHAI noi ra, KHONG duoc nuot — code review 2026-08-06.
    //
    // Quyet dinh #3 chap nhan **cua so SAP MAY** giua commit va fs::write, va no dung:
    // AD-33 noi meta.json dung lai duoc tu project.db. Nhung no KHONG cho phep di tiep
    // khi ham TRA VE Err. Hai chuyen khac han nhau:
    //   - sap may  ⇒ khong ai chay duoc ma dep, va lan mo sau dung lai duoc;
    //   - Err      ⇒ tien trinh van song, va di tiep nghia la tra ve Ok cho mot .atproj
    //                chi co HAI thanh phan — pha AC2, va pha AC3 (Library doc metadata
    //                ma khong mo SQLite) ngay tu luc tao.
    //
    // Va duong dung lai KHONG TU CHAY: `rebuild_from_store` khong co mot cho goi san
    // pham nao (story nay khong dung man hinh "mo lai mot .atproj"), nen mot meta.json
    // vang mat nam do cho toi Epic 5.
    //
    // ⇒ Cuon lai TRON VEN. An toan vi `create_work_folder` tao DOC QUYEN: `dir` chac chan
    // la thu muc cua chinh luot goi nay, khong phai du lieu co san.
    if let Err(err) = meta.write_atomic(&dir) {
        // ⚠️ Chẩn đoán KHÔNG lặp lại chuỗi "meta.json" viết thẳng (2026-08-28, Story 5.5) --
        // `meta_write_boundary.rs` khoá tên tệp CHỈ ở `core/library/meta.rs`.
        //
        // 🔵 ĐO (2026-08-28, vòng rà thứ hai) -- lượt đổi chữ này KHÔNG làm người vận hành
        // mất đường lần dấu: `{err}` là `MetaError` mà `write_atomic` trả về, và
        // `MetaError::Io::fmt` (`core/library/meta.rs`) in NGUYÊN đường dẫn đầy đủ, luôn kết
        // thúc bằng `meta.json` (`meta[<duong-dan>/meta.json] io failed: <chi tiet>`). Câu
        // log ở đây chỉ đổi phần TIỀN TỐ mô tả thao tác; tên tệp thật vẫn tới log qua `{err}`.
        eprintln!(
            "project[{}] work metadata cache write failed after commit, rolling back: {err}",
            dir.display()
        );
        store.close();
        remove_folder(&dir);
        return Err(crate::core::library::WorkError::from(err).into());
    }

    let scope = crate::core::scope::ScopeResolver::with_work(crate::core::scope::WorkScope {
        work_id: meta.work_id.clone(),
    });

    Ok(OpenWork { dir, store, scope, meta, chapter_id, images_saved, images_failed })
}

/// Trích URL của một [`ChapterInput`] cho [`prepare_chapter_images`] — URL của TRANG chứa
/// ảnh, không phải một nhãn chẩn đoán chung chung.
///
/// 🔵 **SỬA (vòng rà đối kháng 2, mục F1) — gọi THẲNG `pipeline::label_of`, không còn một
/// bản chép riêng.** `label_of` từng là một hàm LỒNG riêng tư bên trong
/// `run_import_with_order`, buộc module này phải giữ một bản chép tay 4 dòng y hệt (khoá
/// đồng bộ bằng một cổng test riêng, `chapter_page_url_drift_boundary.rs`). `label_of` nay
/// `pub(crate)` ở module scope — gọi thẳng xoá bản chép VÀ xoá luôn cổng canh trôi (một
/// nguồn sự thật duy nhất không thể trôi khỏi chính nó).
fn chapter_input_page_url(c: &ChapterInput) -> String {
    crate::core::segment::pipeline::label_of(c)
}

/// Một hàng `asset` ĐÃ SẴN SÀNG ghi — tệp đã nằm trên đĩa, chỉ còn thiếu `chapter_id` (chưa
/// biết tới khi giao dịch ghi chèn hàng `chapter` và đọc `last_insert_rowid()`), nên trường
/// đó thay bằng `chapter_index` (chỉ số trong `chapters`, ổn định xuyên suốt `create_work`).
struct SavedAsset {
    chapter_index: usize,
    /// **THÊM 2026-09-09 (Story 6.13)** — chỉ số của khối `Image` trong `chapter.blocks` mà
    /// hàng này sinh ra từ đó. `anchor_after_segment_ord` ngay dưới được tính TRƯỚC khi dệt
    /// segment vai (Quyết định 2 spec 6.13) — sau khi dệt, `create_work` tra
    /// `WovenChapter::shifted_anchor_by_block` bằng CẶP `(chapter_index, block_index)` để cập
    /// nhật lại neo trước khi ghi `INSERT INTO asset`, đúng khuôn dời neo đã ghi ở
    /// `schema.rs:958-975`.
    block_index: usize,
    anchor_after_segment_ord: i64,
    file_name: String,
    /// ⚠️ **NỢ CÓ CHỦ, ghi ra tại chỗ (vòng rà đối kháng 2, mục D1) — đây là URL YÊU CẦU
    /// (`p.resolved_url`, `src` đã phân giải tuyệt đối), KHÔNG PHẢI chặng CUỐI sau chuyển
    /// hướng.** `webimport::fetch`/[`FetchedPage`] không trả lại URL cuối cùng đã dừng ở đó
    /// (chỉ trả `bytes`/`content_type`) — thêm trường đó là một thay đổi chữ ký xuyên
    /// `fetcher.rs`→`fetch_and_write_one_asset`→ở đây, ngoài phạm vi lượt vá nhỏ này. Cột
    /// `source_url` mang HAI VAI cùng lúc (xuất xứ hiển thị cho người dùng VÀ khoá dedup
    /// AD-41 — xem `cache` bên dưới, khoá bằng CHÍNH `resolved_url` này) — nếu sửa để ghi
    /// chặng cuối, vai KHOÁ DEDUP phải tách khỏi vai HIỂN THỊ (hai cột, không một). Một ảnh
    /// mà máy chủ chuyển hướng sang host khác sẽ hiển thị SAI xuất xứ cho người dùng hôm nay.
    /// **Chủ: Story 6.14** (màn hình đầu tiên THẬT SỰ hiển thị `source_url` cho người dùng
    /// đọc) — `deferred-work.md`.
    ///
    /// 🔵 **SỬA 2026-09-09 (Story 6.12) — kiểu đổi từ `String` sang `Option<String>`.** Ảnh
    /// nhúng `.docx` không có URL nào để mà ghi — `NULL` đã hợp lệ theo lược đồ
    /// (`schema.rs` — `ASSET_DDL`, `source_url` là cột DUY NHẤT cho `NULL`, doc-comment tại
    /// đó nói thẳng nó dành cho ảnh không đến từ mạng). `Some(url)` cho đường mạng (Story
    /// 6.11), `None` cho đường `.docx` (Story 6.12).
    source_url: Option<String>,
    byte_len: i64,
    content_type: String,
}

/// Kết quả pha ảnh (Story 6.11) — xem doc-comment [`create_work`] cho VỊ TRÍ nó chạy.
///
/// 🔵 **SỬA 2026-09-08 (mục B1 vòng rà đối kháng 3 lớp) — KHÔNG còn trường `domain_log`.**
/// Bản trước tích luỹ `Vec<DomainLogEntry>` ở đây rồi CHỈ gắn vào `OpenWork` trên đường thành
/// công — một lượt trượt giữa chừng (ví dụ đĩa đầy) trả `Err` sớm và không có kiểu nào chở
/// được `Vec` đó đi cùng nhánh lỗi, nên nhật ký của những ảnh ĐÃ tải xong trước đó biến mất
/// (vi phạm §Always "kể cả lượt trượt"). Mỗi lời gọi `fetch` nay PUSH THẲNG vào
/// `webimport::DomainLogState` (tham số mới của `prepare_chapter_images`) NGAY khi hoàn tất —
/// entry sống sót bất kể phần còn lại của `create_work` có trượt hay không.
struct ImagePrepOutcome {
    saved: Vec<SavedAsset>,
    images_saved: u32,
    images_failed: u32,
}

/// Một URL ảnh đã thử tải TRONG CHÍNH lượt nhập này — `None` nghĩa là đã thử và TRƯỢT.
///
/// 🔴 Cache CẢ chiều thất bại (không chỉ chiều thành công): AD-41 vế "không tải lại ảnh đã
/// có" áp cho MỌI kết quả trong cùng lượt nhập — một host trả 404 lặp lại ở tám Chương không
/// đáng tám lượt gọi mạng giống hệt nhau, đúng tinh thần I/O Matrix "0 lời gọi mạng; dùng lại
/// (kết quả đã có)".
/// 🔵 **THÊM `Clone` 2026-09-09 (Story 6.12)** — cache tra theo URL (`prepare_chapter_images`)
/// nay trả một bản CLONE thay vì một tham chiếu mượn, để cùng một vòng lặp xử lý được CẢ
/// nhánh `Remote` (tra cache) LẪN nhánh `Local` (không cache, không mượn gì) mà không phải
/// hai kiểu trả về khác nhau — ba trường đều rẻ để clone (hai `String` ngắn, một `i64`).
#[derive(Clone)]
struct CachedFetch {
    file_name: String,
    byte_len: i64,
    content_type: String,
}

/// Pha ảnh của [`create_work`] (Story 6.11, FR127) — chạy TRƯỚC giao dịch ghi SQL, NGOÀI mọi
/// closure `Store::write`.
///
/// Ba việc, theo đúng thứ tự (mỗi ảnh KEPT của mỗi Chương có `blocks: Some(..)`):
/// 1. Tính neo TRƯỚC KHI thử mạng (`core::segment::anchor::compute_anchor`) — một ảnh không
///    tính được neo thì KHÔNG đáng một lượt tải (tránh ghi một tệp mồ côi không có hàng
///    `asset` nào tham chiếu được tới nó).
/// 2. Phân giải `src` thành URL tuyệt đối (`webimport::assets::resolve_absolute_url`), dựng
///    MỘT [`webimport::Allowlist`] tầng 2 từ đúng những host của các ảnh sẽ thử tải (§Always
///    spec 6.11 — tầng 2 dựng từ ĐÚNG ảnh `effective_kept` trả `true`).
/// 3. Tải qua [`webimport::fetch`] (kèm cache dedup theo URL tuyệt đối, cả hai chiều thành
///    công/thất bại), kiểm MIME, ghi byte xuống `assets/` (đường DUY NHẤT `fs::write` của cả
///    hàm).
///
/// # Lỗi
/// Trả `Err` ở HAI nhánh, cả hai đều là điều kiện KHÔNG THỂ NGƯỜI DÙNG GÂY RA (lỗi lập trình
/// hoặc I/O thật, không phải một MIME/host/mạng xấu):
/// 1. Ghi BYTE xuống đĩa thất bại (`std::fs::write`, ví dụ đĩa đầy) — I/O Matrix spec 6.11
///    hàng "Ghi tệp trượt giữa chừng".
/// 2. 🔵 **THÊM (vòng rà đối kháng 2, mục B1)** — bất biến nội bộ vỡ: `extension_for_mime`
///    xác nhận một MIME thuộc danh mục ĐÓNG nhưng `normalized_mime` (đọc CÙNG `content_type`)
///    lại trả `None` — chứng minh được là KHÔNG THỂ xảy ra hôm nay (cùng phép chuẩn hoá nội
///    bộ), nhưng trả `Err` thay vì `unreachable!()` để phòng một lượt tách rời logic sau này
///    (`panic = "abort"` giết cả tiến trình, một `Err` thì không).
///
/// Cả hai nhánh khiến `create_work` dọn SẠCH `.atproj` (không phải hàm này — nó chỉ trả
/// `Err`, chỗ gọi mới `remove_folder`). MỌI lý do khác (host ngoài tầng 2, MIME sai, mạng
/// lỗi, neo không tính được) chỉ đếm vào `images_failed`, KHÔNG BAO GIỜ trả `Err`.
/// **THÊM 2026-09-09 (Story 6.12)** — nguồn byte của một [`PendingImage`]: mạng (Story 6.11)
/// hoặc byte đã đọc SẴN từ zip `.docx` (Story 6.12, không mạng, không `Allowlist`).
enum PendingImageSource {
    /// URL tuyệt đối đã phân giải — đi qua `fetch_and_write_one_asset` (mạng, dedup theo
    /// URL, Allowlist tầng 2).
    Remote { resolved_url: String },
    /// Byte đã có sẵn trong bộ nhớ (ảnh nhúng `.docx`) — đi thẳng
    /// [`write_local_asset_bytes`], không dedup theo URL (không có URL để mà khoá).
    Local { bytes: Vec<u8>, content_type: String },
}

fn prepare_chapter_images(
    dir: &Path,
    chapters: &[crate::core::segment::import::ImportedChapter],
    chapter_urls: &[String],
    block_overrides: &[Option<bool>],
    cleanup_rules: &[crate::core::cleanup::CleanupRule],
    source_lang: &str,
    domain_log_state: &webimport::DomainLogState,
    // 🔴 **THÊM 2026-09-09 (Story 6.12)** — ảnh nhúng `.docx` của Chương ĐẦU TIÊN (rỗng cho
    // mọi đường khác). Xem doc-comment [`crate::core::segment::import::DocxSidecar`].
    docx_images: &[crate::core::docx::DocxImage],
) -> Result<ImagePrepOutcome, IpcError> {
    use crate::core::segment::pipeline::effective_kept_for_blocks;
    use crate::core::webimport::BlockBody;

    /// Một ảnh GIỮ mà neo ĐÃ tính được — sẵn sàng đi vào hàng đợi tải/ghi.
    struct PendingImage {
        chapter_index: usize,
        /// **THÊM 2026-09-09 (Story 6.13)** — xem doc-comment [`SavedAsset::block_index`].
        block_index: usize,
        anchor_after_segment_ord: i64,
        source: PendingImageSource,
    }

    let mut pending: Vec<PendingImage> = Vec::new();
    // 🔵 SỬA 2026-09-09 (vòng rà đối kháng 3, mục R5) — MỌI lượt tăng `images_failed` trong
    // hàm này dùng `saturating_add(1)`, không `+= 1` trần. Trước sửa này `images_saved` được
    // bão hoà có chủ (kèm hẳn một đoạn biện hộ, xem cuối hàm) trong khi `images_failed` +=
    // trần WRAP AROUND về 0 trên release build (Rust chỉ panic-on-overflow ở debug) — cùng
    // MIỀN TRÀN (đếm ảnh trong một lượt nhập) nhưng HAI kỷ luật khác nhau là một sự bất nhất
    // không có lý do. Chọn MỘT: bão hoà ở CẢ HAI, cùng lý lẽ hệt `images_saved` — tràn đòi
    // hơn bốn tỷ ảnh trong MỘT lượt nhập, không một trang thật nào chạm tới, và một con số
    // bão hoà ("ít nhất lớn cỡ này") trung thực hơn một số WRAP về 0 im lặng.
    let mut images_failed: u32 = 0;

    for (i, chapter) in chapters.iter().enumerate() {
        let Some(blocks) = &chapter.blocks else { continue };
        if blocks.is_empty() {
            continue;
        }
        // Cùng luật `Step::ExtractMainContent` (`pipeline.rs`) — chỉ Chương ĐẦU TIÊN đọc
        // `block_overrides` (§Never spec 6.9, kế thừa nguyên vẹn ở đây).
        let overrides_for_unit: &[Option<bool>] = if i == 0 { block_overrides } else { &[] };
        let effective_kept = effective_kept_for_blocks(blocks, overrides_for_unit);
        let page_url = chapter_urls.get(i).map(String::as_str).unwrap_or("");

        for (block_idx, block) in blocks.iter().enumerate() {
            let BlockBody::Image { src, .. } = &block.body else { continue };
            if !effective_kept[block_idx] {
                continue;
            }

            // 🔴 **THÊM 2026-09-09 (Story 6.12)** — ảnh `.docx` không có URL (`src` luôn
            // `None` cho hình dạng đó, xem doc-comment `BlockBody::Image`); byte thật của nó
            // đã được đọc SẴN lúc `import_file` chạy và chỉ sống ở Chương ĐẦU TIÊN (cùng
            // giới hạn `block_overrides`). Tìm theo `block_idx` TRƯỚC khi coi `src: None` là
            // một lỗi.
            let local_image = if i == 0 { docx_images.iter().find(|d| d.block_index == block_idx) } else { None };

            if src.is_none() && local_image.is_none() {
                images_failed = images_failed.saturating_add(1);
                eprintln!(
                    "asset[src] anh khong co thuoc tinh src va khong co byte .docx tuong ung \
                     -- bo qua (chuong {i}, khoi {block_idx})"
                );
                continue;
            }

            // Neo TRƯỚC mạng/ghi tệp — một ảnh không neo được thì không đáng một lượt tải
            // (tránh một tệp mồ côi không hàng `asset` nào trỏ tới, xem doc-comment hàm này).
            let anchor_after_segment_ord = match crate::core::segment::anchor::compute_anchor(
                blocks,
                &effective_kept,
                block_idx,
                &chapter.source_text,
                &chapter.segments,
                cleanup_rules,
                source_lang,
            ) {
                Ok(ord) => ord,
                Err(err) => {
                    images_failed = images_failed.saturating_add(1);
                    eprintln!(
                        "asset[neo] tinh neo that bai (chuong {i}, khoi {block_idx}): {}",
                        err.detail
                    );
                    continue;
                }
            };

            let source = if let Some(img) = local_image {
                // Ảnh `.docx` nhúng — 0 mạng, 0 `Allowlist` (§Always spec 6.12).
                PendingImageSource::Local { bytes: img.bytes.clone(), content_type: img.content_type.clone() }
            } else if let Some(src) = src {
                match webimport::assets::resolve_absolute_url(src, page_url) {
                    Ok(resolved_url) => PendingImageSource::Remote { resolved_url },
                    Err(_err) => {
                        images_failed = images_failed.saturating_add(1);
                        // E4 (vòng rà đối kháng 2, 3 lớp) — `ResolveUrlError::detail` nhúng
                        // NGUYÊN VĂN `src` (URL người dùng đã dán/trang chứa) — KHÔNG in ra
                        // stderr, kể cả bản release. Vị trí (chương/khối) đủ để chẩn đoán cục
                        // bộ mà không lộ URL.
                        eprintln!("asset[url] khong phan giai duoc src (chuong {i}, khoi {block_idx})");
                        continue;
                    }
                }
            } else {
                // KHÔNG THỂ xảy ra: điều kiện ngay trên đã xác nhận `src.is_some() ||
                // local_image.is_some()`, và nhánh `local_image` vừa xử ở trên — trả một
                // lỗi ĐẾM được thay vì `unreachable!()` (`panic = "abort"`), cùng khuôn mọi
                // bất biến nội bộ khác của hàm này (xem `# Lỗi` doc-comment).
                images_failed = images_failed.saturating_add(1);
                eprintln!(
                    "asset[bat_bien] nhanh khong the xay ra: src/local_image deu None sau \
                     khi da kiem co nguon (chuong {i}, khoi {block_idx})"
                );
                continue;
            };

            pending.push(PendingImage { chapter_index: i, block_index: block_idx, anchor_after_segment_ord, source });
        }
    }

    if pending.is_empty() {
        return Ok(ImagePrepOutcome { saved: Vec::new(), images_saved: 0, images_failed });
    }

    // 🔴 Tầng 2 dựng từ ĐÚNG những host MẠNG của ảnh sẽ thử tải — không tầng 1 (pha này KHÔNG
    // BAO GIỜ tải Trang, §Always spec 6.11), và không đọc ảnh `.docx` (0 host — chúng không
    // đi qua mạng, §Always spec 6.12: "Không dựng Allowlist nào trên đường này"). Tầng 1
    // rỗng ⇒ `Allowlist::decide` cho `ResourceKind::Page` luôn `Denied`, vô hại vì hàm này
    // chỉ gọi `fetch(..., ResourceKind::Image)`.
    let tier2_hosts: std::collections::BTreeSet<String> = pending
        .iter()
        .filter_map(|p| match &p.source {
            PendingImageSource::Remote { resolved_url } => webimport::assets::host_of(resolved_url),
            PendingImageSource::Local { .. } => None,
        })
        .collect();
    let allowlist = webimport::Allowlist::default().with_tier2_hosts(tier2_hosts);

    let assets_dir = dir.join("assets");
    let mut cache: std::collections::HashMap<String, Option<CachedFetch>> = std::collections::HashMap::new();
    let mut saved: Vec<SavedAsset> = Vec::new();

    for p in &pending {
        // 🔵 SỬA 2026-09-09 (Story 6.12) — nhánh theo `PendingImageSource`. Dedup theo URL
        // (khuôn Story 6.11) chỉ có nghĩa cho `Remote`; `Local` (ảnh `.docx`) không có URL
        // để mà khoá cache, và ghi thẳng qua [`write_local_asset_bytes`] — 0 mạng, 0
        // `Allowlist`, 0 `DomainLogState` (bảng đó chỉ ghi nhận lượt RA MẠNG, §Always spec 6.8).
        let cached: Option<CachedFetch> = match &p.source {
            PendingImageSource::Remote { resolved_url } => {
                if let Some(existing) = cache.get(resolved_url) {
                    existing.clone()
                } else {
                    // 🔵 B1 — `fetch_and_write_one_asset` PUSH THẲNG vào `domain_log_state`
                    // bên trong nó (trước bước ghi tệp có thể trượt); `?` ở đây không còn
                    // vứt mất một Vec cục bộ nào — cái duy nhất `?` lan ra là `IpcError` của
                    // chính lượt ghi byte trượt, nhật ký thì đã AN TOÀN từ trước đó rồi.
                    let outcome =
                        fetch_and_write_one_asset(resolved_url, &allowlist, &assets_dir, domain_log_state)?;
                    cache.insert(resolved_url.clone(), outcome.clone());
                    outcome
                }
            }
            PendingImageSource::Local { bytes, content_type } => {
                write_local_asset_bytes(bytes, content_type, &assets_dir)?
            }
        };

        match cached {
            Some(c) => saved.push(SavedAsset {
                chapter_index: p.chapter_index,
                block_index: p.block_index,
                anchor_after_segment_ord: p.anchor_after_segment_ord,
                file_name: c.file_name,
                source_url: match &p.source {
                    PendingImageSource::Remote { resolved_url } => Some(resolved_url.clone()),
                    // 🔴 `NULL` — §Always spec 6.12: ảnh `.docx` không đến từ mạng, đúng
                    // nghĩa cột `source_url` đã khai từ Story 6.11 (schema.rs).
                    PendingImageSource::Local { .. } => None,
                },
                byte_len: c.byte_len,
                content_type: c.content_type,
            }),
            None => images_failed = images_failed.saturating_add(1),
        }
    }

    // 🔵 THÊM (vòng rà đối kháng 3, mục R2) — kiểm TRƯỚC giao dịch ghi, không để một `CHECK`
    // của `ASSET_DDL` vỡ BÊN TRONG `store.write` giết TRỌN lượt nhập. Một hàng vi phạm bên
    // trong giao dịch làm SQLite trả lỗi, `create_work` dọn SẠCH `.atproj` — đúng cho lỗi
    // GHI ĐĨA (§Always spec 6.11), nhưng SAI cho một hàng dữ liệu hỏng của MỘT ảnh: cùng
    // triết lý §Always "một ảnh trượt không được dừng cả lượt nhập" mà `images_failed` tồn
    // tại để giữ. Lọc Ở ĐÂY (ngoài giao dịch, có thể đếm/log an toàn) thay vì để SQLite từ
    // chối bên trong.
    let (saved, newly_failed) = saved.into_iter().partition::<Vec<_>, _>(saved_asset_satisfies_asset_check_constraints);
    for offender in &newly_failed {
        eprintln!(
            "asset[check] hang asset vi pham CHECK cua ASSET_DDL, bo qua truoc giao dich (chuong {}, file_name={:?})",
            offender.chapter_index, offender.file_name
        );
    }
    images_failed = images_failed.saturating_add(u32::try_from(newly_failed.len()).unwrap_or(u32::MAX));

    // 🔵 THÊM (vòng rà đối kháng 3, mục R6) — phép kiểm VÉT CẠN cho `chapter_index`, ĐO ĐƯỢC
    // là an toàn hôm nay (mỗi `chapter_index` chép THẲNG từ `i` của vòng lặp
    // `chapters.iter().enumerate()` ngay trên, cùng hàm này — không đường nào khác gán nó),
    // nhưng chỗ gọi (`create_work`) lọc theo `chapter_index == i` bằng một `filter()` KHÔNG
    // VÉT CẠN — một `SavedAsset` mang `chapter_index` không khớp BẤT KỲ vòng lặp Chương nào
    // (một lượt tách rời logic tương lai lỡ gán sai) sẽ không được lọc trúng ở BẤT KỲ chỉ số
    // nào: 0 lần `INSERT`, tệp vẫn nằm trên đĩa (đã ghi ở `fetch_and_write_one_asset`, TRƯỚC
    // giao dịch), và `images_saved` (tính NGAY DƯỚI ĐÂY, từ `saved.len()`) đếm THỪA đúng
    // hàng đó — một tệp mồ côi VÀ một con số hiển thị sai, không lỗi nào báo. Lọc Ở ĐÂY
    // (cùng vị trí R2, ngoài giao dịch) thay vì phát hiện muộn sau khi đã ghi xong.
    let chapters_len = chapters.len();
    let (saved, out_of_range) =
        saved.into_iter().partition::<Vec<_>, _>(|s| saved_asset_chapter_index_is_in_range(s, chapters_len));
    for offender in &out_of_range {
        eprintln!(
            "asset[chapter_index] chapter_index ({}) vuot qua so Chuong ({chapters_len}) -- bo qua truoc giao dich, file_name={:?}",
            offender.chapter_index, offender.file_name
        );
    }
    images_failed = images_failed.saturating_add(u32::try_from(out_of_range.len()).unwrap_or(u32::MAX));

    // ⚠️ NÓI RA (vòng rà đối kháng 2, mục D7) — cùng lý lẽ hệt `byte_len` ở
    // `fetch_and_write_one_asset`: `unwrap_or(u32::MAX)` là một con số CHẨN ĐOÁN (số hàng
    // `asset` đã chèn thành công), không phải một VỊ TRÍ — tràn `u32` đòi hơn bốn tỷ ảnh
    // trong MỘT lượt nhập, một con số không một trang/Tác phẩm thật nào chạm tới được. Bão
    // hoà thay vì `Err`/panic ở một nhánh không ai từng chạm là lựa chọn có chủ, không phải
    // một lối tắt.
    let images_saved = u32::try_from(saved.len()).unwrap_or(u32::MAX);
    Ok(ImagePrepOutcome { saved, images_saved, images_failed })
}

/// R2 (vòng rà đối kháng 3, lớp 3) — bản Rust CỦA CHÍNH các `CHECK` mà `ASSET_DDL` khai, đo
/// TRƯỚC khi một hàng đi vào giao dịch ghi. `char::is_whitespace()` của Rust đã đúng thuộc
/// tính Unicode `White_Space` (không cần liệt tay 25 điểm mã như SQL — `trim()` của SQLite
/// chỉ cắt ASCII, `str::trim()` của Rust thì không có giới hạn đó), nên vế này ĐƠN GIẢN HƠN
/// bản SQL, không phải một bản chép kém trung thành hơn.
fn saved_asset_satisfies_asset_check_constraints(saved: &SavedAsset) -> bool {
    // 🔵 SỬA 2026-09-09 (Story 6.12) — `source_url` nay `Option<String>` (NULL hợp lệ cho
    // ảnh `.docx`, xem doc-comment trường đó). Vị từ khớp ĐÚNG CHECK của `ASSET_DDL`:
    // `source_url IS NULL OR trim(source_url, ...) <> ''`.
    let source_url_ok = match &saved.source_url {
        None => true,
        Some(s) => !s.trim().is_empty(),
    };
    !saved.file_name.trim().is_empty()
        && source_url_ok
        && !saved.content_type.trim().is_empty()
        && saved.byte_len >= 0
        && saved.anchor_after_segment_ord >= 0
}

/// R6 (vòng rà đối kháng 3, lớp 3) — vị từ VÉT CẠN cho `chapter_index`: chỗ gọi
/// (`create_work`) chỉ ghi hàng có `chapter_index == i` cho `i` chạy trong `0..chapters_len`
/// (đúng vòng lặp `chapters.iter().enumerate()`) — một hàng nằm NGOÀI dải đó không được lọc
/// trúng ở BẤT KỲ `i` nào, để lại một tệp mồ côi trên đĩa và một `images_saved` đếm thừa.
fn saved_asset_chapter_index_is_in_range(saved: &SavedAsset, chapters_len: usize) -> bool {
    saved.chapter_index < chapters_len
}

/// Tải + ghi ĐÚNG MỘT ảnh (đã qua dedup ở [`prepare_chapter_images`]) — đường DUY NHẤT
/// `fs::write` của cả pha ảnh.
///
/// `Ok(None)` cho MỌI lý do khiến ảnh này không có tệp: mạng lỗi/host bị chặn/HTTP lỗi
/// (`FetchError`), hoặc MIME không phải ảnh raster (§Never spec 6.11: SVG bị loại).
///
/// 🔵 **SỬA (vòng rà đối kháng 3, mục T4) — `Err` không còn CHỈ cho ghi byte thất bại.** Câu
/// cũ ở đây hết đúng từ lượt B1 (vòng rà đối kháng 2): `Err` còn nổ ở nhánh bất biến nội bộ
/// `normalized_mime` vỡ (xem `# Lỗi` của [`prepare_chapter_images`] cho chi tiết cả hai
/// nhánh) — cả hai đều là điều kiện lập trình/I-O thật, không phải một MIME/host/mạng xấu.
///
/// 🔵 **SỬA (vòng rà đối kháng, đo được — không suy luận).** Bản trước gọi
/// [`webimport::assets::is_raster_image_mime`] làm một bước GÁC RIÊNG đứng TRƯỚC
/// [`webimport::assets::extension_for_mime`] — hai vị từ cùng canh một mệnh đề ("MIME này có
/// được chấp nhận không") trên đúng một danh mục bốn phần tử. Đo bằng phép GỠ THẬT: gỡ hẳn
/// nhánh gọi `is_raster_image_mime` khỏi hàm này rồi chạy TRỌN `cargo test --locked` (46
/// binary, kể cả `asset_contract.rs`/`webimport_contract.rs`) — **0 ca đỏ**. Đúng thứ mã CHẾT
/// trên đường sản phẩm: `extension_for_mime` đã TỰ đóng vai trò gác cổng qua nhánh `_ => None`
/// của chính nó, nên gọi `is_raster_image_mime` trước đó không đổi một hành vi quan sát được
/// nào — đúng lớp lỗi "hai nguồn sự thật cho một mệnh đề" (không phải "ca đầu-cuối gác nhầm
/// chỗ": ca đầu-cuối ĐÚNG chỗ, vị từ ĐẦU mới là chỗ thừa). ⇒ Gỡ HẲN nhánh gọi rời đó —
/// `extension_for_mime` MỘT MÌNH là điểm quyết định DUY NHẤT trên đường này.
/// `is_raster_image_mime` VẪN ở lại `core::webimport::assets` (xuất khẩu công khai, tự kiểm
/// bằng test riêng của nó) như một vị từ ĐỘC LẬP cho chỗ gọi tương lai (Story 6.14 hay bất kỳ
/// nơi nào cần hỏi "MIME này có phải ảnh raster hay không" mà không cần đuôi tệp) — nó không
/// còn là một BƯỚC trong luồng ghi ảnh của story này.
///
/// 🔵 **SỬA 2026-09-08 (mục B1 vòng rà đối kháng 3 lớp) — nhận `&DomainLogState`, không còn
/// `&mut Vec`.** `Vec<DomainLogEntry>` do `fetch()` trả về được PUSH THẲNG vào state ngay tại
/// đây — TRƯỚC bước ghi byte (dòng có thể trả `Err` và khiến hàm này thoát sớm). Nhờ vậy nhật
/// ký của lượt gọi NÀY luôn an toàn trước khi có cơ hội bị `?` cuốn mất ở tầng gọi.
/// 🔵 **SỬA 2026-09-09 (Story 6.12) — tách thành HAI NỬA, đúng Task list spec 6.12.** Bản
/// Story 6.11 làm cả hai việc ("lấy byte qua mạng" + "ghi + dựng hàng") trong MỘT hàm; đường
/// `.docx` cần nửa THỨ HAI (ghi + dựng hàng) mà không đi qua nửa ĐẦU (mạng) — xem
/// [`write_local_asset_bytes`] ngay dưới, hàm DUY NHẤT ghi byte xuống đĩa bây giờ. Nửa NÀY
/// (mạng) trả `Ok(None)` cho MỌI lý do khiến ảnh không có byte: mạng lỗi/host bị chặn/HTTP
/// lỗi (`FetchError`), hoặc MIME không phải ảnh raster (§Never spec 6.11: SVG bị loại) —
/// TRƯỚC khi gọi nửa ghi, nên nửa ghi không cần biết gì về mạng/nhật ký domain.
fn fetch_asset_bytes_over_network(
    url: &str,
    allowlist: &webimport::Allowlist,
    domain_log_state: &webimport::DomainLogState,
) -> Result<Option<(Vec<u8>, String)>, IpcError> {
    let (result, mut log) = webimport::fetch(url, allowlist, webimport::ResourceKind::Image);

    let page = match result {
        Ok(page) => page,
        Err(_err) => {
            // E4 (vòng rà đối kháng 2, 3 lớp) — KHÔNG `eprintln!` URL của người dùng ra
            // stderr (kể cả bản release, không điều kiện) — `domain_log_state` NGAY DƯỚI đây
            // đã là đúng kênh cho đúng loại sự kiện này (NFR19 audit trail, session-lifetime,
            // không phải một tệp log vô thời hạn trên đĩa).
            webimport::append_domain_log_entries(domain_log_state, log);
            return Ok(None);
        }
    };

    let Some(_ext) = webimport::assets::extension_for_mime(page.content_type.as_deref()) else {
        // E4 (vòng rà đối kháng 2, 3 lớp) — KHÔNG in URL người dùng ra stderr; `content_type`
        // (một chuỗi MIME, không phải dữ liệu định danh cá nhân) đủ để chẩn đoán cục bộ.
        eprintln!("asset[mime] mime khong phai anh raster: {:?}", page.content_type);
        // 🔵 Story 6.11, mục A (Ice ký 2026-09-08) — chặng này ĐÃ được `fetch()` gán
        // `Fetched` (mạng thành công); sửa lại thành `MimeRejected` TRƯỚC khi push, vì
        // `fetcher.rs` không biết gì về danh mục MIME ảnh (AD-40) — chỉ tầng gọi này biết.
        if let Some(last) = log.last_mut() {
            last.outcome = Some(webimport::DomainLogOutcome::MimeRejected);
        }
        webimport::append_domain_log_entries(domain_log_state, log);
        return Ok(None);
    };
    // `append_domain_log_entries` chạy TRƯỚC bất kỳ nhánh `Err` nào bên dưới — chặng mạng ĐÃ
    // hoàn tất (log đã có nội dung thật) không được phép biến mất chỉ vì một bước XỬ LÝ sau
    // đó (dù chỉ là một bất biến nội bộ, không phải I/O) gặp trục trặc.
    webimport::append_domain_log_entries(domain_log_state, log);

    // MIME đã CHUẨN HOÁ lấy THẲNG từ content-type của phản hồi (D1 vòng rà đối kháng 3 lớp) —
    // không dựng lại từ đuôi tệp. `_ext` vừa được `extension_for_mime` xác nhận là MỘT TRONG
    // BỐN kiểu ĐÃ CHẤP NHẬN, nên `normalized_mime` của CHÍNH `content_type` này không thể trả
    // `None` — bất biến chứng minh được bằng ĐỌC MÃ. Trả `Err` thay vì `unreachable!()` (xem
    // lý lẽ B1 gốc, giữ nguyên qua lượt tách): `panic = "abort"` giết NGUYÊN tiến trình nếu
    // một lượt sau tách rời hai bảng MIME mà quên đổi đồng bộ.
    let Some(normalized_content_type) = webimport::normalized_mime(page.content_type.as_deref()) else {
        eprintln!(
            "asset[mime] bat bien noi bo vo (extension_for_mime nhan {_ext} nhung normalized_mime tra None)"
        );
        return Err(IpcError::new(
            "work.create_failed",
            MessageKey::WorkCreateFailed,
            std::collections::BTreeMap::new(),
            false,
        ));
    };

    Ok(Some((page.bytes, normalized_content_type)))
}

/// Ghi byte ảnh xuống đĩa + dựng [`CachedFetch`] — NỬA HAI của cả đường mạng (Story 6.11)
/// LẪN đường `.docx` (Story 6.12); đường DUY NHẤT `fs::write` của cả pha ảnh.
///
/// `content_type` PHẢI đã là một trong bốn MIME ảnh raster đã chấp nhận (hoặc một biến thể
/// mà [`webimport::assets::normalized_mime`] chuẩn hoá về đúng một trong bốn đó) —
/// `content_type` KHÔNG thuộc danh mục đó (MIME lạ suy từ đuôi tệp media `.docx`, ví dụ
/// `bmp`/`emf`/`wmf`) trả `Ok(None)`, đúng khuôn "MIME ngoài danh mục 4 ⇒ bỏ ảnh,
/// `images_failed`" của I/O Matrix — KHÔNG một nhánh `Err` riêng cho ca này.
///
/// 🔵 **SỬA (vòng rà đối kháng 3, mục T4, kế thừa qua lượt tách 2026-09-09)** — `Err` không
/// CHỈ cho ghi byte thất bại: bất biến nội bộ `normalized_mime` vỡ (xem `# Lỗi` doc-comment
/// `prepare_chapter_images`) cũng trả `Err`, cùng lý do B1 gốc (`panic = "abort"`).
fn write_local_asset_bytes(
    bytes: &[u8],
    content_type: &str,
    assets_dir: &Path,
) -> Result<Option<CachedFetch>, IpcError> {
    let Some(ext) = webimport::assets::extension_for_mime(Some(content_type)) else {
        eprintln!("asset[mime] mime khong phai anh raster: {content_type:?}");
        return Ok(None);
    };
    let Some(normalized_content_type) = webimport::normalized_mime(Some(content_type)) else {
        eprintln!(
            "asset[mime] bat bien noi bo vo (extension_for_mime nhan {ext} nhung normalized_mime tra None)"
        );
        return Err(IpcError::new(
            "work.create_failed",
            MessageKey::WorkCreateFailed,
            std::collections::BTreeMap::new(),
            false,
        ));
    };

    let file_name = format!("{}.{ext}", Uuid::new_v4());
    let path = assets_dir.join(&file_name);
    // D3 (vòng rà đối kháng 3 lớp) — CỐ Ý KHÔNG dùng khuôn ghi-nguyên-tử của
    // `core::library::meta::Meta::write_atomic`/`core::glossary::exchange_io::write_export_file`
    // (tạm cạnh đích → sync → rename) ở đây — xem lý lẽ đầy đủ tại doc-comment gốc (giữ
    // nguyên qua lượt tách): `path` là một tên UUID hoàn toàn MỚI, KHÔNG ai đọc nó cho tới khi
    // hàng `INSERT INTO asset` được COMMIT; một `fs::write` trượt giữa chừng khiến chỗ gọi
    // (`prepare_chapter_images` → `create_work`) `remove_folder(&dir)` xoá NGUYÊN `.atproj`.
    if let Err(e) = std::fs::write(&path, bytes) {
        eprintln!("asset[ghi] ghi byte anh xuong dia that bai ({}): {e}", path.display());
        return Err(IpcError::new(
            "asset.write_failed",
            MessageKey::AssetWriteFailed,
            std::collections::BTreeMap::new(),
            false,
        ));
    }

    // ⚠️ NÓI RA (vòng rà đối kháng 2, mục D7) — `unwrap_or(i64::MAX)` là một lượt LÀM TRÒN có
    // chủ (giữ nguyên qua lượt tách): một con số CHẨN ĐOÁN (`byte_len`), tràn KHÔNG THỂ XẢY RA
    // trong thực tế trên đường mạng (`fetcher.rs::MAX_RESPONSE_BYTES` 20 MiB) lẫn đường
    // `.docx` (`MAX_IMPORT_BYTES` 100 MB chặn cả tệp `.docx`, một ảnh nhúng không thể lớn hơn
    // chính tệp chứa nó).
    Ok(Some(CachedFetch {
        file_name,
        byte_len: i64::try_from(bytes.len()).unwrap_or(i64::MAX),
        content_type: normalized_content_type,
    }))
}

/// Tải + ghi ĐÚNG MỘT ảnh MẠNG (đã qua dedup ở [`prepare_chapter_images`]) — hợp hai nửa
/// [`fetch_asset_bytes_over_network`] + [`write_local_asset_bytes`]. Đường `.docx` KHÔNG gọi
/// hàm này — nó gọi thẳng [`write_local_asset_bytes`] (§Always spec 6.12: "Không dựng
/// Allowlist nào trên đường này").
fn fetch_and_write_one_asset(
    url: &str,
    allowlist: &webimport::Allowlist,
    assets_dir: &Path,
    domain_log_state: &webimport::DomainLogState,
) -> Result<Option<CachedFetch>, IpcError> {
    let Some((bytes, content_type)) = fetch_asset_bytes_over_network(url, allowlist, domain_log_state)? else {
        return Ok(None);
    };
    write_local_asset_bytes(&bytes, &content_type, assets_dir)
}

/// **Hàm thuần** — nhánh dán văn bản của AC1.
///
/// 🔵 **KHÔNG đổi hành vi (Story 6.3)** — vẫn khai UTF-8 cứng qua `encoding_rs::UTF_8` (văn
/// bản dán tay là `ChapterInput::AlreadyText`, bước giải mã bỏ qua vế transcode cho hình
/// dạng đó dù tham số này là gì — xem doc-comment `pipeline::decode_unit`). Đường sản phẩm
/// CÓ xem trước bảng mã là `wire::confirm_import_with_encoding`; hàm này ở lại cho
/// `tests/**` và mọi chỗ gọi không đi qua màn xem trước.
pub fn create_work_from_text(
    documents_root: &Path,
    name: &str,
    source_lang: &str,
    genre: &str,
    text: String,
) -> Result<OpenWork, IpcError> {
    // Đường Blob KHÔNG BAO GIỜ có ảnh (extract_main_content luôn false cho hình dạng này) —
    // một kho tạm, vứt đi ngay sau lượt gọi, đúng khuôn doc-comment `create_work`.
    create_work(
        documents_root,
        name,
        source_lang,
        genre,
        import_text(text),
        encoding_rs::UTF_8,
        Vec::new(),
        None,
        Vec::new(),
        // Đường dán văn bản không bao giờ là `PipelineShape::Bilingual` — mặc định (0, 1,
        // false) không bao giờ được đọc.
        0,
        1,
        false,
        &[],
        &std::sync::Mutex::new(Vec::new()),
        // Văn bản dán tay không bao giờ có một `DocxSidecar` — xem doc-comment kiểu đó.
        None,
    )
}

// ═════════════════════════════════════════════════════════════════════════════════
// Story 3.5 — quét ứng viên khi nhập tài liệu, chạy NGOÀI luồng giao diện (FR47)
// ═════════════════════════════════════════════════════════════════════════════════

/// Sự kiện phát SAU một lượt quét khi nhập — Story 3.5. Cặp số `(inserted, skipped)`, KỂ
/// CẢ khi cả hai là 0 (§Boundaries: *"Mọi số đếm báo ra, kể cả 0"* — một Chương rỗng vẫn
/// bắn sự kiện, phân biệt được với *"quét chưa chạy"*).
///
/// ⚠️ `#[serde(rename_all = ...)]` KHÔNG đặt — cùng luật với mọi struct qua biên IPC. Sự
/// kiện này hôm nay **0 người tiêu thụ phía frontend** (story chỉ giao vế Rust của cặp số;
/// bề mặt UI đọc lại nó là Story 3.6/3.8) — xem Spec Change Log của story cho lý do.
#[derive(Debug, Clone, serde::Serialize)]
pub struct GlossaryImportScanEvent {
    pub chapter_id: i64,
    pub inserted: i64,
    pub skipped: i64,
    /// `completed` hoặc `dictionary_inconclusive`. Worker bị huỷ KHÔNG phát sự kiện —
    /// một scan cũ không được giả làm một lượt đã hoàn tất.
    pub outcome: &'static str,
}

/// Tên sự kiện trên dây — khuôn `EXIT_FLUSH_EVENT` (`lib.rs:161`).
pub const GLOSSARY_IMPORT_SCAN_EVENT: &str = "aura://glossary-import-scan-completed";
const IMPORT_SCAN_COMPLETED: &str = "completed";
const IMPORT_SCAN_DICTIONARY_INCONCLUSIVE: &str = "dictionary_inconclusive";

/// Generation huỷ lượt quét cũ khi một import mới thay Tác phẩm đang mở.
///
/// Clone chỉ clone `Arc`, nên worker và vỏ IPC đọc cùng một bộ đếm. Không `Arc<Store>`:
/// generation chỉ chở một số, còn mọi quyền ghi vẫn được lấy lại từ `OpenWorkState` theo
/// `work_id` ngay trước enqueue.
#[derive(Debug, Clone, Default)]
pub struct ImportScanGeneration(Arc<AtomicU64>);

impl ImportScanGeneration {
    fn next(&self) -> u64 {
        self.0.fetch_add(1, Ordering::AcqRel).wrapping_add(1)
    }

    fn is_current(&self, generation: u64) -> bool {
        self.0.load(Ordering::Acquire) == generation
    }
}

/// Ánh xạ DUY NHẤT từ kết quả lookup nhiều lớp sang ba trạng thái mà lượt scan hiểu.
/// Worker và unit test cùng gọi hàm này; không test một closure bool chép lại quyết định.
fn dictionary_probe_from_grouped(
    grouped: &crate::core::dict::GroupedLookup,
) -> crate::core::glossary::DictionaryProbe {
    if !grouped.skipped.is_empty() {
        crate::core::glossary::DictionaryProbe::Inconclusive
    } else if !grouped.groups.is_empty() || !grouped.hidden_sources.is_empty() {
        crate::core::glossary::DictionaryProbe::Known
    } else if !grouped.truncated_layers.is_empty() {
        // Trần cấp-layer có thể đã cắt mất một hit mà `groups`/`hidden_sources` không còn
        // chứng minh được. Gọi nó là `Missing` sẽ biến dữ liệu bị cắt trang thành ứng viên
        // giả; chỉ một lượt không chạm trần mới được kết luận dứt khoát là thiếu.
        crate::core::glossary::DictionaryProbe::Inconclusive
    } else {
        crate::core::glossary::DictionaryProbe::Missing
    }
}

/// Payload duy nhất cho nhánh từ điển không kết luận. Tách constructor khỏi `emit` để
/// hình dạng dây (outcome và cả hai số 0) được khóa bằng serialization test mà không phải
/// dựng một `AppHandle` giả.
fn dictionary_inconclusive_event(chapter_id: i64) -> GlossaryImportScanEvent {
    GlossaryImportScanEvent {
        chapter_id,
        inserted: 0,
        skipped: 0,
        outcome: IMPORT_SCAN_DICTIONARY_INCONCLUSIVE,
    }
}

/// Quyết định DUY NHẤT ngay sau thuật toán thuần. Chỉ `Enqueue` mang candidates xuống
/// đường ghi; `DictionaryInconclusive` chỉ cho phép phát outcome chẩn đoán, còn stale/
/// cancelled dừng tuyệt đối — không write và không completed event.
#[derive(Debug, PartialEq, Eq)]
enum ImportScanNextStep {
    Enqueue(Vec<crate::core::glossary::ScanCandidate>),
    EmitDictionaryInconclusive,
    Stop,
}

fn import_scan_next_step(
    outcome: crate::core::glossary::ScanOutcome,
    is_current: bool,
) -> ImportScanNextStep {
    if !is_current {
        return ImportScanNextStep::Stop;
    }
    match outcome {
        crate::core::glossary::ScanOutcome::Completed(candidates) => {
            ImportScanNextStep::Enqueue(candidates)
        }
        crate::core::glossary::ScanOutcome::DictionaryInconclusive => {
            ImportScanNextStep::EmitDictionaryInconclusive
        }
        crate::core::glossary::ScanOutcome::Cancelled => ImportScanNextStep::Stop,
    }
}

/// Import đã commit là sự thật không đảo ngược. Worker scan là hậu xử lý best-effort;
/// seam `spawn` tối thiểu làm lỗi `thread::Builder::spawn` kiểm được mà không tìm cách
/// ép hệ điều hành cạn tài nguyên trong test.
fn keep_committed_import_when_scan_spawn_fails<T>(
    committed: T,
    spawn: impl FnOnce() -> std::io::Result<()>,
) -> T {
    if let Err(err) = spawn() {
        eprintln!("glossary[import_scan] tao worker that bai sau khi import da commit: {err}");
    }
    committed
}

/// Đọc `source_text` của mọi segment CÒN SỐNG (`retired_at IS NULL`) của Chương
/// `chapter_id`, theo `ord` — đúng ranh giới câu mà Story 2.1 đã tách LÚC NHẬP (§Boundaries
/// của story: *"không tự đoán lại"*).
fn read_chapter_segment_texts(
    store: &Store,
    chapter_id: i64,
) -> Result<Vec<String>, crate::core::store::StoreError> {
    store.read(move |conn: crate::core::store::ReadHandle<'_>| {
        let mut stmt = conn.prepare(
            "SELECT source_text FROM segment WHERE chapter_id = ?1 AND retired_at IS NULL \
             ORDER BY ord",
        )?;
        let mut rows = stmt.query([chapter_id])?;
        let mut out = Vec::new();
        while let Some(row) = rows.next()? {
            out.push(row.get::<_, String>(0)?);
        }
        Ok(out)
    })
}

/// **Hàm thuần** — đơn vị QUYẾT ĐỊNH đi tiếp hay dừng, tách khỏi vỏ `AppHandle`/
/// `std::thread` để `tests/**`/`#[cfg(test)]` gọi được TRỰC TIẾP, không cần webview và
/// không cần một luồng nền nào. Đây chính là hàm mà `spawn_import_scan` gọi ở CẢ HAI lần
/// khoá — cùng một quyết định, không hai bản chép tay có thể trôi khỏi nhau.
///
/// Trả `Some(&open.store)` khi và chỉ khi có một Tác phẩm đang mở VÀ nó vẫn là ĐÚNG Tác
/// phẩm đã chốt `work_id` lúc `spawn_import_scan` bắt đầu. `None` ⇒ dừng LẶNG LẼ — đúng
/// I/O Matrix *"Kho đóng giữa lượt quét ⇒ luồng nền kết thúc lặng lẽ, không panic"*, mở
/// rộng cho ca "Tác phẩm đổi" (`OpenWorkState` vẫn `Some` nhưng trỏ một Tác phẩm KHÁC —
/// cùng lớp nguyên nhân: `open`/`work_id` không còn khớp nhau, chỉ khác `open` là `None`
/// hay `Some(sai)`).
fn guarded_open_store<'a>(open: Option<&'a OpenWork>, work_id: &str) -> Option<&'a Store> {
    let open = open?;
    if open.meta.work_id != work_id {
        return None;
    }
    Some(&open.store)
}

/// Khoá `OpenWorkState` đúng một vùng ngắn: xác nhận `work_id`, lọc hai tầng và enqueue;
/// vé trả ra ngoài vùng khoá để caller chờ writer mà không chặn lệnh đổi/đóng Tác phẩm.
/// `is_current` được hỏi ngay trước enqueue để generation cũ không xếp một lượt ghi mới.
fn filter_and_enqueue_current_import_scan(
    work_state: &OpenWorkState,
    work_id: &str,
    global: &Store,
    candidates: &mut Vec<crate::core::glossary::ScanCandidate>,
    is_current: &dyn Fn() -> bool,
) -> Result<
    Option<crate::core::glossary::ImportScanWriteTicket>,
    crate::core::glossary::GlossaryError,
> {
    let guard = work_state
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let Some(open) = guard.as_ref() else {
        return Ok(None);
    };
    let Some(store) = guarded_open_store(Some(open), work_id) else {
        return Ok(None);
    };

    let skipped_by_scope = crate::core::glossary::filter_import_scan_candidates_by_scope(
        &open.scope,
        global,
        store,
        candidates,
    )?;
    if !is_current() {
        return Ok(None);
    }
    let ticket =
        crate::core::glossary::enqueue_import_scan_candidates(store, candidates, skipped_by_scope)?;
    Ok(Some(ticket))
}

/// **Hàm thuần** — cùng lý do [`guarded_open_store`]: tách quyết định ra khỏi thân
/// `spawn_import_scan` để `#[cfg(test)]` gọi được trực tiếp.
///
/// 🔴 **VÁ 2026-08-22 (rà ba lớp) — bản trước NUỐT ca `DictLayers` chưa được quản lý.**
/// `layers_state.as_deref().unwrap_or(&empty_layers)` gộp HAI trạng thái khác hẳn nhau vào
/// một nhánh im lặng: ① `DictLayers` đã quản lý nhưng RỖNG (0 lớp từ điển gắn — trạng thái
/// BÌNH THƯỜNG có tên, AD-25, `src-tauri/resources/dict/` rỗng trong git) và ② `DictLayers`
/// CHƯA TỪNG được `app.manage(...)` (lỗi cấu hình `setup()` — không nên xảy ra, nhưng
/// `lib.rs` luôn `app.manage` một tập lớp dù RỖNG, nên `None` ở đây chỉ có nghĩa là bước đó
/// chưa chạy). Trộn hai ca thành một khiến ca ② — thứ đáng báo — không khác gì ca ① — thứ
/// bình thường: `is_known` luôn `false`, bộ lọc "không có trong từ điển nhúng" vô hiệu HOÀN
/// TOÀN, và bảng chờ ngập từ điển mà không một dòng chẩn đoán nào — đúng ca bàn đo bàn giao
/// đã chạy phải (`DictLayers::empty()`, 969 ứng viên). Sửa: tách hai ca ra, cùng khuôn
/// nhánh `Store` thiếu ngay trên (`eprintln!` rồi dừng) — CHỈ ca ② mới `eprintln!`/dừng; ca
/// ① (đã quản lý, rỗng) đi tiếp lặng lẽ, đúng bản chất "trạng thái bình thường" của nó.
///
/// 🔵 **MỞ PHẠM VI 2026-08-26 (cụm F)** — `pub(crate)`, dùng CHUNG cho
/// `commands::glossary::wire::{glossary_marks_for_chapter, glossary_pending_candidates}`
/// (`glossary.rs` ghi *"THÊM 2026-08-24"* và tái lập ĐÚNG anti-pattern này bằng
/// `unwrap_or(&empty_layers)`, hai ngày SAU khi nó bị gọi tên ở đây). Tham số `surface` mới
/// là nguyên nhân duy nhất khiến hàm không còn "thuần một tham số": chẩn đoán phải nêu đúng
/// bề mặt đang gọi (`import_scan` / `marks_for_chapter` / `pending_candidates`), không in
/// cứng `[import_scan]` cho một chỗ gọi khác hẳn.
pub(crate) fn guarded_dict_layers<'a>(
    layers: Option<&'a crate::core::dict::DictLayers>,
    surface: &str,
) -> Option<&'a crate::core::dict::DictLayers> {
    if layers.is_none() {
        eprintln!(
            "glossary[{surface}] DictLayers chua duoc quan ly -- bo qua (chay tiep se lam is_known LUON false, vo hieu hoan toan bo loc tu dien)"
        );
    }
    layers
}

/// Chạy lượt quét cho Chương `chapter_id` của Tác phẩm `work_id` trên một `std::thread`
/// RIÊNG — spawn TỪ `wire::create_work_from_text`/`wire::create_work_from_file`, **SAU**
/// khi `replace_open_work` đã đặt `OpenWork` vào state (tức sau khi transaction nhập đã
/// commit — spawn TRƯỚC đó là quét một Chương chưa tồn tại).
///
/// ─────────────────────────────────────────────────────────────────────────────
/// 🔴 KHOÁ `OpenWorkState` HAI LẦN NGẮN, KHÔNG MỘT LẦN DÀI SUỐT LƯỢT QUÉT
/// ─────────────────────────────────────────────────────────────────────────────
/// Giữ khoá mutex của `OpenWorkState` suốt pha quét (có thể tới vài giây trên một Chương
/// lớn) sẽ chặn MỌI lệnh khác cần đọc `OpenWorkState` — kể cả `read_open_chapter` mà
/// frontend gọi ngay sau khi tạo Tác phẩm để mở Chương trong Editor. Hàm này khoá ĐÚNG HAI
/// lần, ngắn: một lần để đọc segment (rồi nhả khoá TRƯỚC khi chạy thuật toán quét, tốn CPU
/// nhất), một lần để ghi lô (rồi nhả ngay). Cả hai lần đều gọi ĐÚNG một hàm quyết định —
/// [`guarded_open_store`] — đối chiếu `work_id` với giá trị đã chốt lúc spawn: Tác phẩm đổi
/// giữa hai lần khoá (một lượt tạo Tác phẩm MỚI trong lúc lượt quét của Tác phẩm CŨ còn
/// đang chạy) làm luồng nền kết thúc LẶNG LẼ, không ghi vào kho SAI Tác phẩm — cùng I/O
/// Matrix *"Kho đóng giữa lượt quét ⇒ kết thúc lặng lẽ, không panic"*, mở rộng cho ca "Tác
/// phẩm đổi" (không chỉ ca "kho đóng"). [`guarded_open_store`] là **hàm thuần**, tách khỏi
/// `AppHandle`/`std::thread` — `tests::` canh cả ba ca của hàng I/O Matrix đó trực tiếp,
/// không qua webview/luồng nào.
///
/// 🔴 **Không `unwrap()`/`expect()` nào trên đường này** — `panic = "abort"` giết cả tiến
/// trình (AGENTS.md), và một luồng nền là chỗ tệ nhất để việc đó xảy ra: không ai đang chờ
/// kết quả của nó để thấy màn hình treo, người dùng chỉ thấy ứng dụng biến mất.
fn spawn_import_scan(
    app: tauri::AppHandle,
    work_id: String,
    chapter_id: i64,
    source_lang: String,
) -> std::io::Result<()> {
    use tauri::Manager as _;

    let Some(generation_state) = app.try_state::<ImportScanGeneration>() else {
        eprintln!("glossary[import_scan] generation state chua duoc quan ly -- bo qua luot quet");
        return Ok(());
    };
    let generation_state = generation_state.inner().clone();
    let generation = generation_state.next();

    std::thread::Builder::new()
        .name(format!("aura-import-scan-{generation}"))
        .spawn(move || {
        use tauri::{Emitter as _, Manager as _};

            let current = || generation_state.is_current(generation);
            if !current() {
                return;
            }

        let segments: Vec<String> = {
            let Some(work_state) = app.try_state::<OpenWorkState>() else {
                return;
            };
                let guard = work_state
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner);
            let Some(store) = guarded_open_store(guard.as_ref(), &work_id) else {
                return;
            };
            match read_chapter_segment_texts(store, chapter_id) {
                Ok(rows) => rows,
                Err(err) => {
                    eprintln!("glossary[import_scan] doc segment that bai: {err}");
                    return;
                }
            }
        };

        let Some(global) = app.try_state::<Store>() else {
            eprintln!("glossary[import_scan] global.db chua duoc quan ly -- bo qua luot quet");
            return;
        };
        let config = match crate::core::scope::load_global_config(&global) {
            Ok(c) => c,
            Err(err) => {
                eprintln!("glossary[import_scan] doc app_config that bai: {err}");
                return;
            }
        };
        let threshold = config.glossary_scan_threshold();
        let disabled = config.disabled_source_codes();

        let layers_state = app.try_state::<crate::core::dict::DictLayers>();
        let Some(layers): Option<&crate::core::dict::DictLayers> =
            guarded_dict_layers(layers_state.as_deref(), "import_scan")
        else {
            return;
        };

        let lang = crate::core::glossary::match_lang_for_source_lang(&source_lang);
        let segment_refs: Vec<&str> = segments.iter().map(String::as_str).collect();

            // `skipped` mang CẢ layer hỏng lúc mở lẫn lúc lookup. Một kết quả rỗng kèm
            // `skipped` là KHÔNG KẾT LUẬN, không phải “term không có”.
            let mut probe_dictionary = |term: &str| {
            let result = crate::core::dict::lookup_grouped(
                layers,
                term,
                crate::core::dict::LookupMode::Exact,
                1,
                &disabled,
            );
                dictionary_probe_from_grouped(&result)
        };
            let mut is_cancelled = || !current();
            let scan_outcome = crate::core::glossary::scan_candidates_controlled(
            &segment_refs,
            lang,
            threshold,
            crate::core::glossary::COMMON_SURNAMES,
                &mut probe_dictionary,
                &mut is_cancelled,
        );

            let mut candidates = match import_scan_next_step(scan_outcome, current()) {
                ImportScanNextStep::Enqueue(candidates) => candidates,
                ImportScanNextStep::Stop => return,
                ImportScanNextStep::EmitDictionaryInconclusive => {
                    if let Err(err) = app.emit(
                        GLOSSARY_IMPORT_SCAN_EVENT,
                        dictionary_inconclusive_event(chapter_id),
                    ) {
                        eprintln!("glossary[import_scan] phat su kien that bai: {err}");
                    }
                    return;
                }
            };

            // Chỉ ENQUEUE diễn ra dưới mutex. `ticket.wait()` nằm ngoài khối, nên một
            // writer chậm không giữ `OpenWorkState` qua giao dịch.
            let ticket = {
            let Some(work_state) = app.try_state::<OpenWorkState>() else {
                return;
            };
                match filter_and_enqueue_current_import_scan(
                    &work_state,
                    &work_id,
                    &global,
                    &mut candidates,
                    &current,
                ) {
                    Ok(Some(ticket)) => ticket,
                    Ok(None) => return,
                    Err(err) => {
                        eprintln!("glossary[import_scan] loc/xep lo that bai: {err}");
                return;
                    }
                }
            };

            let (inserted, skipped) = match ticket.wait() {
                Ok(counts) => counts,
                Err(err) => {
                    eprintln!("glossary[import_scan] ghi lo that bai: {err}");
                    return;
                }
        };
            if !current() {
                return;
            }

        if let Err(err) = app.emit(
            GLOSSARY_IMPORT_SCAN_EVENT,
            GlossaryImportScanEvent {
                chapter_id,
                inserted,
                skipped,
                    outcome: IMPORT_SCAN_COMPLETED,
            },
        ) {
            eprintln!("glossary[import_scan] phat su kien that bai: {err}");
        }
        })
        .map(|_| ())
}

/// **Hàm thuần** — nhánh tệp của AC1 (kéo-thả **hoặc** ô nhập đường dẫn; cả hai đã
/// resolve thành một `path` thật ở lớp gọi, xem AD-1/AD-16).
///
/// 🔵 **KHÔNG đổi hành vi (Story 6.3)** — vẫn khai UTF-8 cứng. Đường sản phẩm CÓ xem trước
/// bảng mã là `wire::confirm_import_with_encoding`; hàm này ở lại cho `tests/**` và mọi chỗ
/// gọi không đi qua màn xem trước (cùng lý do `create_work_from_text`).
///
/// # Lỗi
/// Một phần mở rộng chưa nhận (không phải `.txt`/`.md`/`.docx`) ⇒ `import.unsupported_format`
/// (AC8), **trước khi** thư mục `.atproj` được tạo — [`import_file`] từ chối theo phần mở
/// rộng trước khi mở tệp. 🔵 **SỬA 2026-09-09 (Story 6.12)** — `.docx` không còn ví dụ ở đây,
/// nó nay ĐƯỢC nhận.
pub fn create_work_from_file(
    documents_root: &Path,
    name: &str,
    source_lang: &str,
    genre: &str,
    path: &Path,
) -> Result<OpenWork, IpcError> {
    let (shape, docx_sidecar) = import_file(path)?;
    // Đường Blob KHÔNG BAO GIỜ có ảnh MẠNG — `.docx` (Story 6.12) CÓ THỂ có ảnh NHÚNG, mang
    // trong `docx_sidecar`, truyền NGUYÊN VẸN xuống `create_work`.
    create_work(
        documents_root,
        name,
        source_lang,
        genre,
        shape,
        encoding_rs::UTF_8,
        Vec::new(),
        None,
        Vec::new(),
        // `import_file` never yields `PipelineShape::Bilingual` (that shape is
        // `import_bilingual_file`'s alone) — defaults unread.
        0,
        1,
        false,
        &[],
        &std::sync::Mutex::new(Vec::new()),
        docx_sidecar,
    )
}

// ═════════════════════════════════════════════════════════════════════════════════
// Story 6.3 — màn xem trước bảng mã (FR126) — phát hiện, dải đối chiếu, xác nhận
// ═════════════════════════════════════════════════════════════════════════════════
//
// 🔴 BYTE CỦA NGUỒN ĐỌC ĐÚNG MỘT LẦN (§Always spec 6.3)
// ─────────────────────────────────────────────────────────────────────────────
// `wire::preview_import_encoding_from_text`/`_from_file` đọc nguồn (dán tay: nhận thẳng
// `String` qua IPC; tệp: [`import_file`] gọi `std::fs::read` MỘT LẦN) rồi CẤT [`PipelineShape`]
// đã đọc vào [`PendingImportSourceState`] — một Ô DUY NHẤT, cùng khuôn
// `commands::glossary::PendingImportState`. `wire::confirm_import_with_encoding` CLONE
// (không đọc lại từ đĩa/webview) từ ô đó để chạy [`create_work`] — một lượt xác nhận trượt
// (ví dụ `import.undecodable_bytes` vì người dùng chọn nhầm ứng viên) GIỮ NGUYÊN ô đang chờ,
// nên chọn một ứng viên khác rồi xác nhận lại không đòi đọc tệp/dán lại văn bản lần hai. Ô
// chỉ bị THAY khi một lượt xem trước MỚI mở (ghi đè) — không có vỏ dây riêng cho "huỷ": mở
// một lượt xem trước MỚI hoặc khởi động lại tiến trình là hai cách duy nhất ô này trống lại,
// và cả hai đều vô hại (0 byte nào từng xuống đĩa từ ô này — chỉ [`confirm_import_with_encoding`]
// mới gọi [`create_work`]). "Huỷ" thật sự là một quyết định TẦNG GIAO DIỆN (frontend chặn
// `dispatch('import.preview.confirm')` sau khi đóng lớp phủ — xem `src/importPreviewState.ts`).

/// Nguồn ĐANG CHỜ của một lượt xem trước bảng mã.
pub struct PendingImportSource {
    pub shape: PipelineShape,
    /// **THÊM 2026-09-09 (Story 6.12)** — khối + ảnh nhúng nếu `shape` đến từ một `.docx`
    /// (xem doc-comment [`crate::core::segment::import::DocxSidecar`]). `None` cho mọi
    /// đường khác. `confirm_import_with_encoding` CLONE trường này y hệt `shape` — cùng
    /// vòng đời (giữ nguyên trên đường lỗi, dọn chỉ khi `create_work` thành công).
    pub docx_sidecar: Option<crate::core::segment::import::DocxSidecar>,
}

/// Kiểu state Tauri quản lý — `None` == không lượt xem trước nào đang treo, cùng khuôn
/// `commands::glossary::PendingImportState`.
pub type PendingImportSourceState = std::sync::Mutex<Option<PendingImportSource>>;

/// Bản dựng đã CHUẨN HOÁ của một ứng viên, cộng hai số đếm thiệt hại — hình dạng DÂY của
/// [`NormalizedCandidate`] (Story 6.4, FR124/FR125).
///
/// ⚠️ `#[serde(rename_all = ...)]` KHÔNG đặt.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct NormalizedPreviewWire {
    pub text: String,
    pub joined_lines: usize,
    pub blank_lines_removed: usize,
    /// `true` ⇒ `text` không phải TOÀN Chương (nguồn dài hơn cửa sổ bằng chứng, dòng cuối
    /// đã bị bỏ) — frontend nói ra phạm vi cửa sổ bằng chữ khi trường này là `true`.
    pub window_truncated: bool,
}

impl From<NormalizedCandidate> for NormalizedPreviewWire {
    fn from(n: NormalizedCandidate) -> Self {
        Self {
            text: n.text,
            joined_lines: n.joined_lines,
            blank_lines_removed: n.blank_lines_removed,
            window_truncated: n.window_truncated,
        }
    }
}

/// Một ô trong dải năm ứng viên — hình dạng DÂY của [`EncodingCandidate`].
///
/// ⚠️ `#[serde(rename_all = ...)]` KHÔNG đặt — cùng luật với mọi struct qua biên IPC.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct EncodingCandidateWire {
    /// Nhãn FR126 cho MẮT NGƯỜI (`"UTF-8"`, `"GB18030"`, …).
    pub label: String,
    /// Định danh KHÔNG MẤT MÁT (`Encoding::name()`) — gửi lại y nguyên ở lượt xác nhận.
    pub encoding: String,
    /// Bản dựng thật, tối đa 8 ký tự — `null` khi bảng mã này "không ra chữ" trên cửa sổ
    /// bằng chứng.
    pub preview: Option<String>,
    /// Bản dựng ĐÃ CHUẨN HOÁ cộng hai số đếm — `null` đồng bộ với `preview` (Story 6.4).
    /// §Always spec 6.4: bản dựng này đi kèm sẵn trên dây cho CẢ NĂM ô, điều kiện để đổi
    /// ứng viên vẫn là 0 lời gọi IPC (`importPreviewEncoding.test.ts:123,161,192`).
    pub normalized: Option<NormalizedPreviewWire>,
    /// **THÊM 2026-09-05 (Story 6.5)** — khối làm sạch (tầng 3): văn bản đã đánh dấu +
    /// danh sách luật + hai số đếm, tính bằng cách chạy CHÍNH chuỗi pipeline thật trên bản
    /// dựng an toàn của ứng viên này. `null` đồng bộ với `preview`/`normalized` (bảng mã
    /// này "không ra chữ").
    pub cleanup: Option<CleanupPreviewWire>,
    /// **THÊM 2026-09-05 (Story 6.6)** — khối tách Chương (tầng 4): số Chương nhận ra, kèm
    /// `title`/độ dài từng Chương, tính bằng CHÍNH lượt chạy chuỗi thật đã dựng `cleanup` ở
    /// trên (không một lượt `run_pipeline` thứ hai chỉ cho khối này). `null` đồng bộ với
    /// `cleanup` (bảng mã này "không ra chữ").
    pub chapters: Option<ChapterSplitPreviewWire>,
    /// **THÊM 2026-09-07 (Story 6.9)** — khối tầng 2 (ranh giới bóc): dãy khối cả trang của
    /// Chương ĐẦU TIÊN, tính bằng cách chạy CHÍNH chuỗi pipeline thật đã dựng `cleanup`/
    /// `chapters` ở trên (không một lượt `run_pipeline` thứ hai). `null` khi
    /// `extract_main_content == false` (đường tệp/dán tay — I/O Matrix: "rỗng kèm câu nói vì
    /// sao", khoá `tier_empty_story_6_9`) — KHÔNG đồng bộ `null`/`Some` với `cleanup`: một
    /// bảng mã "không ra chữ" (`cleanup == null`) cũng cho `blocks == null`, cùng lý do.
    pub blocks: Option<ChapterBlocksPreviewWire>,
}

/// Nhãn tầng của một luật làm sạch, trên dây — Story 6.5.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CleanupRuleTierWire {
    Global,
    Work,
}

impl From<CleanupRuleTier> for CleanupRuleTierWire {
    fn from(t: CleanupRuleTier) -> Self {
        match t {
            CleanupRuleTier::Global => CleanupRuleTierWire::Global,
            CleanupRuleTier::Work => CleanupRuleTierWire::Work,
        }
    }
}

/// Một luật, kèm hai số đếm — hình dạng DÂY dùng cho danh sách luật của tầng 3.
///
/// 🔴 Danh tính là CẶP `(tier, id)` — không phải `id` trần (§Always spec 6.5: hai tầng
/// đánh số ĐỘC LẬP, luật Toàn cục #1 và luật Tác phẩm #1 cùng tồn tại).
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct CleanupRuleReportWire {
    pub tier: CleanupRuleTierWire,
    pub id: i64,
    pub pattern: String,
    pub kind: String,
    pub enabled: bool,
    /// Số chỗ khớp trong Chương ĐANG XEM TRƯỚC — trên TOÀN văn bản, kể cả khi luật đã tắt
    /// (§Always spec 6.5: "tắt đổi việc xoá, không đổi việc đo").
    pub count_in_chapter: usize,
    /// Số chỗ khớp trong CẢ lần nhập. 🔵 **SỬA 2026-09-05 (Story 6.6) — 🟡 đóng MỘT PHẦN nợ
    /// `deferred-work.md:9535`, không trọn vẹn.** "LUÔN bằng `count_in_chapter`" đã HẾT ĐÚNG
    /// cho [`PipelineShape::Chapters`] (N đơn vị NGAY TỪ ĐẦU — mỗi Chương có báo cáo THẬT
    /// của riêng nó, hai số THỰC SỰ khác nhau khi có ý nghĩa để khác nhau) — nhưng hình dạng
    /// đó CHƯA có đường sản phẩm nào dựng ra (Story 6.7 sẽ là đường đầu tiên). Trên đường
    /// SẢN PHẨM THẬT của CHÍNH story này (`Blob` + `chapter_pattern`), mệnh đề CŨ vẫn đúng:
    /// hai số LUÔN BẰNG NHAU (đúng số, không bịa — xem doc-comment
    /// [`cleanup_and_chapters_preview_for`] mục "GIỚI HẠN THẬT" cho lý do kiến trúc). N = 1
    /// (mặc định, không mẫu) thì hai số vẫn bằng nhau, đúng hành vi cũ.
    pub count_in_import: usize,
}

/// Một chỗ khớp CỦA LUẬT ĐANG BẬT — chỉ luật bật mới xuất hiện ở đây (§I/O Matrix spec
/// 6.5: "Tắt một luật ⇒ chỗ vừa gạch ngang trở về nguyên trạng NGAY"). Điểm mã, nửa-mở.
///
/// ⚠️ **KHÔNG `impl From<CleanupMatch>`** — cố ý gỡ (vòng rà 2026-09-06). Một chỗ khớp có
/// thể vắt qua biên cửa sổ hiển thị (`start < visible_chars < end`), và `end` phải CẮT về
/// biên đó trước khi lên dây (xem chỗ dựng ở `build_cleanup_preview_wire`) — một `From` 1:1
/// mời gọi sao chép `end` nguyên vẹn, đúng lỗi mà vòng rà vừa bắt.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
pub struct CleanupSpanWire {
    pub tier: CleanupRuleTierWire,
    pub id: i64,
    pub start: usize,
    pub end: usize,
}

/// Khối làm sạch của MỘT ứng viên/đường tự khai — tầng 3 (Story 6.5).
///
/// `text` là văn bản ĐÃ GIẢI MÃ, TRƯỚC khi bất kỳ luật nào xoá gì (đứng TRƯỚC bước 4 chuẩn
/// hoá trong `PIPELINE_ORDER`) — đây là văn bản mà `spans` đánh dấu gạch ngang lên. Nó KHÁC
/// văn bản của tầng "chuẩn hoá" (đã chuẩn hoá, KHÔNG áp luật làm sạch — hai tầng hiển thị
/// hai chặng khác nhau của cùng một lượt chạy chuỗi thật).
///
/// 🔴 **`text`/`spans`/`final_text` ĐƯỢC PHÉP cắt ở cửa sổ hiển thị; `rules[].count_in_chapter`/
/// `.count_in_import` THÌ KHÔNG — hai thứ đo trên hai phạm vi KHÁC NHAU, đừng lẫn.** Sửa
/// 2026-09-06, đóng khuyết tật chứng minh bằng ca test
/// `cleanup_contract.rs::counts_cover_the_whole_chapter_even_when_the_rendered_window_is_truncated`:
/// bản trước chạy chuỗi TRÊN CHÍNH `text` đã cắt, nên hai số đếm cũng bị cắt theo — một luật
/// khớp 40 lần trên cả Chương mà chỉ 4 KiB đầu lọt cửa sổ hiện "khớp 2 chỗ". `cleanup_preview_for`
/// nay chạy chuỗi trên TOÀN văn bản; `text`/`spans`/`final_text` ở đây được CẮT RIÊNG cho hiển
/// thị SAU khi đã đếm xong trên bản đầy đủ — xem doc-comment `cleanup_preview_for`.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct CleanupPreviewWire {
    /// Cắt ở cửa sổ hiển thị khi `window_truncated` — KHÔNG phải căn cứ đếm (xem trên).
    pub text: String,
    /// Chỉ những chỗ khớp NẰM TRONG cửa sổ hiển thị (`end <= text.chars().count()`) — một
    /// chỗ khớp sau biên cửa sổ vẫn được ĐẾM ở `rules[]`, chỉ không có gì để mà gạch ngang
    /// trên phần văn bản đang hiện.
    pub spans: Vec<CleanupSpanWire>,
    /// Hai số đếm của MỖI luật đo trên **TOÀN Chương**, không phải trên `text` đã cắt ở trên
    /// — xem doc-comment `CleanupRuleReportWire::count_in_chapter`.
    pub rules: Vec<CleanupRuleReportWire>,
    /// `true` ⇒ `text` không phải TOÀN Chương — cùng nghĩa
    /// `NormalizedPreviewWire::window_truncated`. Áp cho `text`/`spans`/`final_text`, KHÔNG
    /// áp cho `rules[].count_in_chapter`/`.count_in_import` (hai trường đó luôn đo trên TOÀN
    /// Chương, `window_truncated` hay không).
    pub window_truncated: bool,
    /// **THÊM 2026-09-05 (Story 6.5)** — văn bản CUỐI CÙNG (sau cả làm sạch VÀ chuẩn hoá,
    /// tức `PipelineOutput::chapters[0].source_text` của CHÍNH lượt chạy chuỗi vừa tính ra
    /// `rules` ở trên) — chỗ đóng nợ `deferred-work.md:9359` mà một phép đo tìm thấy được:
    /// khi `window_truncated == false`, trường này PHẢI giống hệt từng byte với `source_text`
    /// mà `confirm_import_with_encoding` ghi xuống cho CÙNG đầu vào — hai nhánh preview/confirm
    /// cùng chạy [`run_pipeline`] trên CÙNG văn bản TRỌN VẸN, không phải hai hàm thuần đặt
    /// cạnh nhau, và không phải một lượt chạy trên bản đã cắt. Khi `window_truncated == true`,
    /// trường này CẮT XUỐNG cửa sổ hiển thị CHỈ ĐỂ HIỆN — bản TRỌN VẸN vẫn được tính đúng và
    /// nằm trong hai số đếm của `rules[]` ở trên, không mất đi đâu cả.
    pub final_text: String,
}

/// Nguyên nhân *cần xem* trên dây — khớp `core::segment::review::ReviewCause`, BỐN khoá
/// literal ĐÓNG (§Always spec 6.10: "bốn nhãn nguyên nhân là bốn khoá literal riêng qua một
/// `switch` cạn" — frontend không nội suy khoá, `check:i18n` phải thấy literal).
///
/// ⚠️ `#[serde(rename_all = "snake_case")]` — bốn nhánh, không phải tên trường.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ReviewCauseWire {
    ShortLength,
    HighCleanupMatches,
    HighJoinedLines,
    NotMeasured,
}

impl From<crate::core::segment::review::ReviewCause> for ReviewCauseWire {
    fn from(c: crate::core::segment::review::ReviewCause) -> Self {
        use crate::core::segment::review::ReviewCause;
        match c {
            ReviewCause::ShortLength => ReviewCauseWire::ShortLength,
            ReviewCause::HighCleanupMatches => ReviewCauseWire::HighCleanupMatches,
            ReviewCause::HighJoinedLines => ReviewCauseWire::HighJoinedLines,
            ReviewCause::NotMeasured => ReviewCauseWire::NotMeasured,
        }
    }
}

/// Một Chương trong khối tách Chương (Story 6.6, tầng 4) — số thứ tự, tiêu đề, độ dài, phán
/// quyết *cần xem*/*sạch* (Story 6.10).
///
/// ⚠️ `#[serde(rename_all = ...)]` KHÔNG đặt.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct ChapterSplitPreviewEntryWire {
    /// 1-based, liên tục — cùng quy ước với cột `ord` của bảng `chapter`.
    pub ord: i64,
    /// Dòng khớp mẫu phân tách, `None` cho Chương lời tựa (trước khớp đầu tiên) hoặc khi
    /// mẫu không khớp/không được cấu hình.
    pub title: Option<String>,
    /// Độ dài `source_text` của Chương này, tính bằng ĐIỂM MÃ (không phải byte) — Task list
    /// spec 6.6.
    pub length: usize,
    /// **THÊM 2026-09-08 (Story 6.10a).** 🔵 **SỬA 2026-09-08 (Story 6.10) — `usize` →
    /// `Option<usize>`, doc-comment cũ đã HẾT ĐÚNG.** Bản 6.10a lập luận `0` ở đây là "một SỐ
    /// THẬT (không đo được cho Chương này)" — đúng chỗ HAI NGHĨA bị hàn vào một số `0` mà Story
    /// 6.10 tồn tại để tách: `Some(0)` = *luật thật sự không khớp gì* (đo được, bằng không);
    /// `None` = *không đo được cho Chương này* (bước 3 không tạo được báo cáo, hoặc — đường
    /// `Blob` + `chapter_pattern` — Chương này không phải Chương `ord = 1`, xem "GIỚI HẠN THẬT"
    /// ở doc-comment [`cleanup_and_chapters_preview_for`]). Trộn hai nghĩa vào `0` làm một
    /// Chương CHƯA AI ĐO trông giống một Chương ĐÃ ĐO VÀ SẠCH — đúng lớp lỗi rỗng-im-lặng mà
    /// `AGENTS.md` gọi tên là trung tâm của dự án (AC 2026-09-08). Tổng CỦA CHÍNH Chương này
    /// (`chapter.cleanup_report.per_rule_counts` cộng dồn qua MỌI luật, kể cả luật đã tắt —
    /// cùng quy ước "tắt đổi việc xoá, không đổi việc đo" của
    /// [`CleanupRuleReportWire::count_in_chapter`]) — trục tóm tắt **eager** mà hàng rào Tukey
    /// của Story 6.10 đọc trực tiếp (`core::segment::review::classify`).
    pub cleanup_match_count: Option<usize>,
    /// **THÊM 2026-09-08 (Story 6.10)** — số LẦN bước 4 (chuẩn hoá) đã NỐI hai dòng làm một,
    /// CỦA CHÍNH Chương này (FR125, khớp `ImportedChapter::joined_line_count`). 🔴 Tên
    /// KHÔNG phải `joined_lines` trần — tên đó đã thuộc [`NormalizedPreviewWire::joined_lines`]
    /// với nghĩa KHÁC (theo ứng viên bảng mã, có cửa sổ `EVIDENCE_WINDOW_BYTES`); tên này theo
    /// quy ước `count_in_chapter` đã có ở [`CleanupRuleReportWire`]. `None` = *không đo được
    /// cho Chương này* — trên đường `Blob` con số đo được TRƯỚC khi tách Chương thuộc về TOÀN
    /// TÀI LIỆU, không quy về Chương nào được, kể cả `ord = 1` (xem doc-comment
    /// `core::segment::pipeline::Flow::joined_line_counts`); trên đường `Chapters` (URL) con
    /// số của mỗi Chương là THẬT.
    pub joined_line_count_in_chapter: Option<usize>,
    /// **THÊM 2026-09-08 (Story 6.10)** — phán quyết của hàng rào Tukey
    /// (`core::segment::review::classify`) trên CHÍNH Chương này. Đây LÀ phán quyết tính LÚC
    /// CHẠY, không lưu xuống đĩa (§Never spec 6.10) — `needs_review == !review_causes.is_empty()`
    /// là một bất biến giữ bởi phía Rust, không phải hai trường độc lập.
    pub needs_review: bool,
    /// Danh mục nguyên nhân *cần xem* — RỖNG khi và chỉ khi `needs_review == false`.
    pub review_causes: Vec<ReviewCauseWire>,
    /// **THÊM 2026-09-10 (Story 6.15, FR128/AD-43)** — bốn trường xuất xứ HIỆU LỰC (đã áp
    /// [`ChapterOriginOverride`] của CHÍNH Chương này, nếu có) — xem [`ChapterOriginWire`].
    pub origin: ChapterOriginWire,
}

/// Bốn trường xuất xứ trên dây, cộng bốn cờ `*_confirmed` — cùng triết lý [`BlockWire::confirmed`]:
/// `true` ⇔ NGƯỜI DÙNG đã chạm ô đó ở màn xem trước (giá trị hiện ra là giá trị họ gõ, kể cả
/// khi họ xoá trắng ⇒ `None` + `confirmed = true`); `false` ⇔ giá trị máy bóc (hoặc `None` nếu
/// máy cũng chưa từng chạy).
#[derive(Debug, Clone, Default, PartialEq, Eq, serde::Serialize)]
pub struct ChapterOriginWire {
    pub author: Option<String>,
    pub site_name: Option<String>,
    pub url: Option<String>,
    pub published_at: Option<String>,
    pub author_confirmed: bool,
    pub site_name_confirmed: bool,
    pub url_confirmed: bool,
    pub published_at_confirmed: bool,
}

impl ChapterOriginWire {
    fn from_machine_and_override(
        machine: Option<&crate::core::webimport::ChapterOrigin>,
        over: Option<&ChapterOriginOverride>,
    ) -> Self {
        let (author, site_name, url, published_at) = effective_origin_fields(machine, over);
        ChapterOriginWire {
            author,
            site_name,
            url,
            published_at,
            author_confirmed: over.is_some_and(|o| o.author.is_some()),
            site_name_confirmed: over.is_some_and(|o| o.site_name.is_some()),
            url_confirmed: over.is_some_and(|o| o.url.is_some()),
            published_at_confirmed: over.is_some_and(|o| o.published_at.is_some()),
        }
    }
}

/// Khối tách Chương của MỘT ứng viên/đường tự khai — tầng 4 (Story 6.6, FR14).
///
/// 🔴 **Mang TOÀN BỘ N Chương, không chỉ ba đầu/ba cuối.** §Never spec 6.6 cấm mọi ngưỡng/cờ
/// "đáng ngờ" — AC5 ("chỗ bắt nhầm nhìn thấy được") đạt bằng đường YẾU HƠN nhưng KHÔNG NÓI
/// DỐI: mọi Chương mang `title`/`length`, và người dùng SẮP XẾP theo `length` để tự phát
/// hiện chỗ bất thường (§Design Notes "Vì sao KHÔNG có cờ đáng ngờ"). Sắp theo độ dài đòi
/// nhìn thấy MỌI Chương, không chỉ một cửa sổ cố định — tầng hiển thị
/// (`ImportPreviewOverlay.vue`) tự co gọn về "ba đầu, `⋯`, ba cuối" làm khung nhìn MẶC ĐỊNH,
/// và mở rộng khi người dùng bấm sắp xếp.
///
/// 🔵 **SỬA 2026-09-08 (Story 6.10) — lệnh cấm "cờ đáng ngờ" ở trên là một lượt HOÃN, không
/// phải một lượt CẤM VĨNH VIỄN, và bản sửa này chính là lúc nó được thi hành.**
/// `deferred-work.md` (mục "Cờ 'đáng ngờ' + nút lọc cho danh sách Chương") ghi rõ lý do hoãn:
/// *"Cả hai vế đòi một hằng số ngưỡng CHƯA ĐO ĐƯỢC trước khi có một kho truyện thật"* — đó là
/// lý do loại một NGƯỠNG TUYỆT ĐỐI (một độ dài ký tự cụ thể, một số lần khớp cụ thể), KHÔNG
/// phải lý do loại mọi phép so. `needs_review`/`review_causes` dưới đây (Story 6.10) là đúng
/// cờ đó, dựng bằng hàng rào Tukey — một phép so TƯƠNG ĐỐI giữa các Chương trong CÙNG lượt
/// nhập (`core::segment::review::classify`), không một hằng số tuyệt đối nào. Chip
/// `title`/`length` SẮP-XẾP-ĐƯỢC ở trên VẪN giữ nguyên — đây là một lớp bổ sung, không phải
/// một lượt thay thế.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct ChapterSplitPreviewWire {
    pub chapter_count: usize,
    pub chapters: Vec<ChapterSplitPreviewEntryWire>,
    /// **THÊM 2026-09-08 (Story 6.10)** — số mục URL HỎNG của CẢ lượt nhập (`error.is_some()`
    /// trên `UrlImportItemWire`) — `0` trên đường tệp/dán tay (không có khái niệm "mục hỏng").
    /// Đi vào [`build_chapter_split_preview_wire`] qua THAM SỐ, KHÔNG tính lại ở đây (§Always
    /// spec 6.10: "hai con số do RUST cộng, kể cả vế link hỏng"). Link hỏng KHÔNG BAO GIỜ trở
    /// thành một hàng trong `chapters` (§Always spec 6.10, phương án C) — con số này đứng
    /// TÁCH BIỆT để tầng hiển thị ghép câu *"X Chương + Y link hỏng"* mà không phải tự cộng gì.
    pub broken_item_count: usize,
    /// **THÊM 2026-09-08 (Story 6.10)** — `N` của chip *"N cần xem · M sạch"`. BẰNG số Chương
    /// mang `needs_review == true` CỘNG `broken_item_count` — một link hỏng LUÔN LUÔN cần chú
    /// ý (nó hỏng), nên nó được cộng vào vế CẦN XEM (§Always spec 6.10: "kể cả vế link hỏng").
    /// Rust cộng con số này — không phải một phép cộng ở tầng hiển thị (AD-1).
    pub needs_review_count: usize,
    /// **THÊM 2026-09-08 (Story 6.10)** — `M` của chip — số Chương mang `needs_review ==
    /// false`. KHÔNG BAO GIỜ cộng `broken_item_count` (link hỏng không phải Chương, và không
    /// bao giờ "sạch" — nó hỏng).
    pub clean_count: usize,
    /// **THÊM 2026-09-08 (Story 6.10)** — `true` khi ÍT NHẤT một trong ba tín hiệu so-tương-đối
    /// (`length`/`cleanup_match_count`/`joined_line_count`) có hàng rào tồn tại cho lượt nhập
    /// này (`core::segment::review::SignalParticipation::any`). `false` ⇒ KHÔNG tín hiệu nào
    /// tham gia (dưới bốn giá trị đo được cho CẢ ba, hoặc mọi hàng rào đều suy biến) —
    /// `needs_review_count`/`clean_count` khi đó KHÔNG được đọc như "đã đo và sạch": tầng hiển
    /// thị phải nói *"chưa đủ Chương để so"* thay vì khai `0 cần xem` (§Always spec 6.10, AC
    /// 2026-09-08 — "không đo được không bao giờ rơi vào nhánh sạch", áp cho CẢ LƯỢT NHẬP khi
    /// trường này là `false`).
    pub any_signal_participated: bool,
}

/// Dựng khối tách Chương (tầng 4) — TÍNH LUÔN phán quyết *cần xem*/*sạch* bằng hàng rào Tukey
/// (`core::segment::review::classify`, Story 6.10) trên chính ba số tóm tắt vừa đọc từ
/// `chapters`. `broken_item_count` đi vào qua THAM SỐ — chỗ gọi trên đường tệp/dán tay truyền
/// `0` (§Always spec 6.10: "đường tệp/dán tay truyền 0"), đường URL truyền số mục
/// `error.is_some()` của CẢ danh sách (xem `url_import_encoding_preview`).
fn build_chapter_split_preview_wire(
    chapters: &[crate::core::segment::import::ImportedChapter],
    broken_item_count: usize,
    origin_overrides: &[Option<ChapterOriginOverride>],
) -> ChapterSplitPreviewWire {
    let metrics: Vec<crate::core::segment::review::ChapterMetrics> = chapters
        .iter()
        .map(|c| crate::core::segment::review::ChapterMetrics {
            length: c.source_text.chars().count(),
            cleanup_match_count: c.cleanup_report.as_ref().map(|r| r.per_rule_counts.values().sum()),
            joined_line_count: c.joined_line_count,
        })
        .collect();
    let outcome = crate::core::segment::review::classify(&metrics);

    let entries: Vec<ChapterSplitPreviewEntryWire> = chapters
        .iter()
        .zip(metrics.iter())
        .zip(outcome.verdicts.iter())
        .enumerate()
        .map(|(i, ((c, m), verdict))| ChapterSplitPreviewEntryWire {
            ord: i as i64 + 1,
            title: c.title.clone(),
            length: m.length,
            cleanup_match_count: m.cleanup_match_count,
            joined_line_count_in_chapter: m.joined_line_count,
            needs_review: verdict.needs_review,
            review_causes: verdict.causes.iter().map(|&cause| ReviewCauseWire::from(cause)).collect(),
            origin: ChapterOriginWire::from_machine_and_override(
                c.origin.as_ref(),
                origin_overrides.get(i).and_then(Option::as_ref),
            ),
        })
        .collect();

    let needs_review_chapters = entries.iter().filter(|e| e.needs_review).count();
    ChapterSplitPreviewWire {
        chapter_count: entries.len(),
        clean_count: entries.len() - needs_review_chapters,
        needs_review_count: needs_review_chapters + broken_item_count,
        broken_item_count,
        any_signal_participated: outcome.participation.any(),
        chapters: entries,
    }
}

// ═════════════════════════════════════════════════════════════════════════════════
// Story 6.9 — tầng 2 (bóc nội dung chính + sửa ranh giới bằng bàn phím, FR123)
// ═════════════════════════════════════════════════════════════════════════════════

/// Thân một khối trên dây — khớp `webimport::BlockBody`, gắn thẻ `kind` (AD-21: dữ liệu định
/// danh máy, không một câu). Trường thân đặt tên `body` ở [`BlockWire`], không ở đây — kiểu
/// này CHÍNH LÀ nội dung `body`.
///
/// ⚠️ `#[serde(rename_all = ...)]` KHÔNG đặt — cùng luật mọi kiểu qua biên IPC; `kind` dùng
/// `rename_all = "snake_case"` RIÊNG (ba giá trị "paragraph"/"image"/"caption", không phải
/// tên trường).
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum BlockBodyWire {
    Paragraph { text: String },
    Image { src: Option<String>, alt: Option<String> },
    Caption { text: String },
}

impl From<&webimport::BlockBody> for BlockBodyWire {
    fn from(b: &webimport::BlockBody) -> Self {
        match b {
            webimport::BlockBody::Paragraph(t) => BlockBodyWire::Paragraph { text: t.clone() },
            webimport::BlockBody::Image { src, alt } => {
                BlockBodyWire::Image { src: src.clone(), alt: alt.clone() }
            }
            webimport::BlockBody::Caption(t) => BlockBodyWire::Caption { text: t.clone() },
        }
    }
}

/// Một khối trên dây — thân CỘNG hai cờ trực giao suy ra đủ ba vạch lề hiển thị (§Spec Change
/// Log spec 6.9, KEEP mục ①): `kept == false` ⇒ "Đã loại"; `kept && !confirmed` ⇒ "Giữ · máy
/// đoán"; `kept && confirmed` ⇒ "Giữ". `kept`/`confirmed` là giá trị HIỆU LỰC (đã áp
/// `Tier2BlockOverridesState`, xem [`build_chapter_blocks_preview_wire`]) — KHÔNG phải
/// `Block::machine_kept` trần.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct BlockWire {
    pub body: BlockBodyWire,
    pub kept: bool,
    /// `true` ⇔ người dùng đã ĐẶT TAY trạng thái này (`Tier2BlockOverridesState` mang
    /// `Some(_)` ở đúng chỉ số này) — phân biệt "Giữ · máy đoán" khỏi "Giữ" đã xác nhận.
    pub confirmed: bool,
}

/// Khối tầng 2 của MỘT ứng viên — Story 6.9, FR123. `None` (ở [`EncodingCandidateWire::blocks`])
/// khi tầng không áp dụng (`extract_main_content == false`, đường tệp/dán tay — I/O Matrix:
/// "Tầng 2 rỗng KÈM CÂU NÓI VÌ SAO", khoá `tier_empty_story_6_9`); `Some` với `blocks` RỖNG khi
/// áp dụng nhưng trang không có khối nào (I/O Matrix "Trang 0 khối" — cực hiếm, xem lưới an
/// toàn ở `extractor.rs::build_blocks`, trong thực tế trang thật LUÔN có ít nhất một khối).
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct ChapterBlocksPreviewWire {
    pub blocks: Vec<BlockWire>,
}

/// Dựng [`ChapterBlocksPreviewWire`] từ MỘT `chapter` (chỗ gọi chọn Chương nào — trước Story
/// 6.10a LUÔN là `chapters.first()`; nay là Chương con trỏ đang chọn) VÀ `block_overrides`,
/// dùng ĐÚNG [`crate::core::segment::pipeline::effective_kept_for_blocks`] mà bước 2 của
/// chuỗi đã gọi để ghép `source_text` — hai nơi PHẢI thấy cùng một trạng thái "giữ" (xem
/// doc-comment hàm đó). `None` khi Chương không tồn tại hoặc `extract_main_content == false`
/// (`chapter.blocks.is_none()`).
///
/// 🔴 **`block_overrides` là trạng thái của ĐÚNG đơn vị 0 của hình dạng GỐC — hàm này KHÔNG
/// tự biết `chapter` truyền vào có phải đơn vị đó hay không.** Đây là một hàm THUẦN áp override
/// THEO CHỈ SỐ lên bất kỳ danh sách khối nào được đưa tới — nó không cầm `detail_chapter_index`
/// nên không thể tự chặn. Chỗ gọi (`cleanup_and_chapters_preview_for`) chịu trách nhiệm CHỈ
/// truyền `block_overrides` thật khi `chapter` đúng là đơn vị 0, và truyền một lát RỖNG cho
/// mọi Chương khác — xem doc-comment tại chỗ gọi đó (vòng rà đối kháng bước 4, P1).
fn build_chapter_blocks_preview_wire(
    chapter: Option<&crate::core::segment::import::ImportedChapter>,
    block_overrides: &[Option<bool>],
) -> Option<ChapterBlocksPreviewWire> {
    let blocks = chapter?.blocks.as_ref()?;
    let effective_kept =
        crate::core::segment::pipeline::effective_kept_for_blocks(blocks, block_overrides);
    let wire_blocks = blocks
        .iter()
        .enumerate()
        .map(|(i, b)| BlockWire {
            body: BlockBodyWire::from(&b.body),
            kept: effective_kept.get(i).copied().unwrap_or(b.machine_kept),
            confirmed: block_overrides.get(i).is_some_and(|o| o.is_some()),
        })
        .collect();
    Some(ChapterBlocksPreviewWire { blocks: wire_blocks })
}

/// Khuôn `(start, end, machine_kept) -> Vec<Option<bool>>` — hàm THUẦN cạnh
/// `wire::tier2_block_confirm_range` (`src-tauri/AGENTS.md:11` cấm luật trong vỏ). Trả một
/// patch ĐẦY ĐỦ `machine_kept.len()` phần tử: `Some(true)` trong dải `[start, end]` (bao gồm
/// hai đầu) — I/O Matrix spec 6.9: "khối 3–9 confirmed, MỌI khối NGOÀI dải ornament, MỘT
/// LƯỢT". `start > end` (không nên tới đây — frontend chặn `]` trước `[`) được XỬ AN TOÀN
/// bằng cách hoán đổi, không panic và không âm thầm bỏ qua.
///
/// 🔴 **SỬA 2026-09-07 (vòng rà bước 4, mục 6) — tham số `machine_kept: &[bool]`, KHÔNG
/// `total: usize` trần.** Bản trước ép NGOÀI dải luôn `Some(false)` — kể cả một khối máy đã
/// ĐÚNG loại từ đầu (`machine_kept[i] == false`). `Some(false)` mang nghĩa "người dùng ĐÃ
/// XÁC NHẬN loại" (`confirmed == true`, xem [`BlockWire`]) — gán nó cho một khối máy chưa hề
/// sai là một lời khai KHÔNG THẬT ("người xác nhận" một điều không ai chạm tới), và làm mất
/// đúng phân biệt "máy đoán" khỏi "người xác nhận" mà toàn bộ `Tier2BlockOverridesState`
/// dựng lên để giữ. Quy tắc ĐÚNG, đo trên chính AC ("mọi khối ngoài dải bị loại"): ngoài dải,
/// một khối máy ĐANG giữ (`machine_kept[i] == true`) cần bị ép — đó là sửa THẬT một quyết
/// định sai của máy, nên `Some(false)`; một khối máy ĐÃ loại (`machine_kept[i] == false`)
/// không cần ép gì — giữ `None` (chưa ai xác nhận, đúng với sự thật) mới đúng, không phải vì
/// tiện mà vì đó là "đủ ép trạng thái ĐÚNG với AC, không thừa một xác nhận không ai làm".
pub fn block_overrides_for_range(start: usize, end: usize, machine_kept: &[bool]) -> Vec<Option<bool>> {
    let (lo, hi) = if start <= end { (start, end) } else { (end, start) };
    machine_kept
        .iter()
        .enumerate()
        .map(|(i, &was_kept)| {
            let in_range = i >= lo && i <= hi;
            if in_range {
                Some(true)
            } else if was_kept {
                // Ngoài dải, máy ĐANG giữ — AC buộc loại, đây là SỬA thật ⇒ xác nhận.
                Some(false)
            } else {
                // Ngoài dải, máy ĐÃ loại từ đầu — không có gì để "xác nhận", giữ None.
                None
            }
        })
        .collect()
}

/// Đặt/gỡ override của MỘT khối theo chỉ số — hàm THUẦN cạnh `wire::tier2_block_set_kept`.
/// Vector NGẮN HƠN `total_blocks` được nới bằng `None` (chưa ai sửa khối đó) trước khi ghi.
///
/// 🔴 **SỬA 2026-09-07 (vòng rà bước 4, mục 5) — tham số `total_blocks: usize` cộng
/// `Result<(), ()>`.** Bản trước nới vector tới `index + 1` KHÔNG so `index` với tổng số
/// khối thật của trang — một `index` từ một lượt hiển thị CŨ (trang đã tải lại, số khối đổi)
/// vẫn được ghi lặng lẽ, tạo một override "ma" không khối nào trên trang MỚI trỏ tới, hoặc
/// ngược lại đọc-nhầm sang một khối KHÁC nếu tổng số khối co lại rồi phình ra trùng chỉ số.
/// Chỗ gọi (`wire::tier2_block_set_kept`) giờ PHẢI tự đo `total_blocks` THẬT (qua
/// [`current_tier2_machine_kept`]) trước khi gọi hàm này, và `Err(())` nghĩa là `index` không
/// còn khớp trang hiện hành — chỗ gọi từ chối ghi thay vì đoán.
pub fn set_block_override(
    overrides: &mut Vec<Option<bool>>,
    index: usize,
    kept: bool,
    total_blocks: usize,
) -> Result<(), ()> {
    if index >= total_blocks {
        return Err(());
    }
    if overrides.len() < total_blocks {
        overrides.resize(total_blocks, None);
    }
    overrides[index] = Some(kept);
    Ok(())
}

/// Đo `machine_kept` THẬT của Chương ĐẦU TIÊN đường URL, HIỆN HÀNH — dùng chung bởi hai vỏ
/// `tier2_block_*` để có một TỔNG SỐ KHỐI đáng tin trước khi ghi override (mục 5/6, vòng rà
/// bước 4). Chạy [`url_import_encoding_preview`] với override RỖNG (`&[]`) — ở đó `kept`
/// của mỗi khối CHÍNH LÀ `machine_kept` trần (không override nào áp) — rồi lấy `blocks` của
/// ứng viên ĐẦU TIÊN mang `Some(blocks)` (năm ứng viên override cùng Chương nên
/// `machine_kept`/tổng số khối giống hệt nhau ở bất kỳ ứng viên nào có `Some`; lấy ứng viên
/// đầu tránh phải biết trước bảng mã nào "ra chữ"). `None` khi danh sách còn mục hỏng/rỗng
/// hoặc `extract_main_content == false` (không có gì để mà đếm) — chỗ gọi coi đó là lỗi lắp
/// dây/trạng thái cũ, từ chối ghi thay vì đoán một tổng số khối.
fn current_tier2_machine_kept(
    items: &[UrlImportItem],
    source_lang: &str,
    cleanup_rules: &[CleanupRule],
) -> Option<Vec<bool>> {
    let preview = url_import_encoding_preview(items, source_lang, cleanup_rules, &[], &[])?;
    let blocks = preview.candidates.iter().find_map(|c| c.blocks.as_ref())?;
    Some(blocks.blocks.iter().map(|b| b.kept).collect())
}

/// Ba trạng thái tin cậy trên dây — DỮ LIỆU (AD-21: Rust không gửi câu). Frontend tự dịch
/// qua `t()` bằng ba khoá cố định (`mode.library.preview.confidence_*`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ConfidenceWire {
    SelfDeclared,
    High,
    Low,
}

impl From<Confidence> for ConfidenceWire {
    fn from(c: Confidence) -> Self {
        match c {
            Confidence::SelfDeclared => ConfidenceWire::SelfDeclared,
            Confidence::HighGuess => ConfidenceWire::High,
            Confidence::LowGuess => ConfidenceWire::Low,
        }
    }
}

/// Kết quả một lượt xem trước bảng mã — trả về từ hai vỏ `preview_import_encoding_from_*`.
///
/// ⚠️ `#[serde(rename_all = ...)]` KHÔNG đặt.
#[derive(Debug, Clone, serde::Serialize)]
pub struct ImportEncodingPreview {
    pub confidence: ConfidenceWire,
    /// Bảng mã đang CHỌN — `EncodingCandidateWire::encoding` của ô mặc định.
    pub selected_encoding: String,
    /// Dải năm ô — RỖNG khi `confidence != low` (dải KHÔNG mở, §Always spec 6.3: "không có
    /// trạng thái lỗi cho bảng mã đoán sai" áp dụng SAU khi người dùng đã thấy dải, không
    /// phải một lý do để giấu nó khi tin cậy cao/tự khai — dải RỖNG ở ca đó vì không có gì
    /// để mắt chọn, không phải vì bị che).
    ///
    /// 🔴 **LUÔN đủ NĂM ô khi có byte thô để dò** (`RawBytes`/`Chapters` mang byte), BẤT KỂ
    /// `confidence` — I/O Matrix spec 6.3 hàng "Tệp thuần ASCII": *"năm bản dựng cho CÙNG
    /// một chuỗi, không có gì để chọn"* — năm bản dựng ĐÃ TỒN TẠI ở ca đó, chỉ trùng nhau.
    /// Việc dải có MỞ hay không (hiện strip cho người dùng thấy) là quyết định của TẦNG HIỂN
    /// THỊ dựa trên `confidence` (`src/importPreviewState.ts`), không phải một quyết định
    /// Rust đưa ra bằng cách giấu dữ liệu — giữ dữ liệu luôn sẵn sàng là điều kiện để người
    /// dùng ép mở dải thủ công (`E`) kể cả khi tin cậy cao, mà không cần một lượt gọi Rust
    /// thứ hai. Rỗng CHỈ xảy ra ở nhánh tự khai thật (`AlreadyText`) — ở đó không có gì để
    /// mà dò, không phải "có nhưng bị giấu".
    pub candidates: Vec<EncodingCandidateWire>,
    /// 🔴 **THÊM 2026-09-04 (Story 6.4, vá vòng rà 1, mục 1).** Bản dựng chuẩn hoá cộng hai
    /// số đếm cho nhánh **TỰ KHAI** — `Some(..)` chính xác khi `candidates` RỖNG (không có
    /// ứng viên nào để mà đọc `.normalized` từ đó), `None` khi `candidates` không rỗng (năm
    /// ô đã tự mang bản dựng riêng, đọc từ đó — không lặp dữ liệu ở đây).
    ///
    /// AC6 của `epics.md` ("màn xem trước hiện văn bản đã chuẩn hoá — đúng thứ sẽ được ghi")
    /// áp cho MỌI đường qua xem trước, không riêng đường có ứng viên bảng mã — cơ chế
    /// theo-ứng-viên (`EncodingCandidateWire::normalized`) không phủ được đường DÁN VĂN BẢN
    /// TAY (`ChapterInput::AlreadyText`, 0 ứng viên): không có trường này, luật gộp dòng vẫn
    /// chạy và AD-4 đóng băng kết quả, mà người dùng không thấy gì (§Spec Change Log, Vòng
    /// rà 1). Dựng từ [`encoding::normalized_self_declared`].
    pub self_declared_normalized: Option<NormalizedPreviewWire>,
    /// **THÊM 2026-09-05 (Story 6.5)** — khối làm sạch (tầng 3) cho nhánh TỰ KHAI, cùng
    /// điều kiện `Some`/`None` với [`Self::self_declared_normalized`].
    pub self_declared_cleanup: Option<CleanupPreviewWire>,
    /// **THÊM 2026-09-05 (Story 6.6)** — khối tách Chương (tầng 4) cho nhánh TỰ KHAI, cùng
    /// điều kiện `Some`/`None` với [`Self::self_declared_normalized`].
    pub self_declared_chapters: Option<ChapterSplitPreviewWire>,
}

/// Chạy chuỗi pipeline thật trên `shape` — **TOÀN Chương, KHÔNG cắt cửa sổ** — để tính khối
/// làm sạch của MỘT ứng viên/đường tự khai, chỗ đóng nợ `deferred-work.md:9359`.
///
/// 🔴 **SỬA 2026-09-06 — khuyết tật chứng minh bằng ca test
/// `cleanup_contract.rs::counts_cover_the_whole_chapter_even_when_the_rendered_window_is_truncated`.**
/// Bản trước nhận THẲNG `window` (bản dựng an toàn đã cắt) làm đầu vào của `run_pipeline`,
/// nên `per_rule_counts`/`matches` sinh ra từ CỬA SỔ — một luật khớp ở cả trong VÀ ngoài cửa
/// sổ chỉ được đếm phần trong cửa sổ, trái thẳng §Always spec 6.5 ("hai con số ... cả hai đo
/// trên TOÀN văn bản, không trên cửa sổ hiển thị"). CPU của `regex`/`normalize` rẻ trên một
/// Chương — cửa sổ `EVIDENCE_WINDOW_BYTES` (Story 6.3/6.4) tồn tại để giới hạn TẢI TRỌNG TRÊN
/// DÂY của bản dựng hiển thị, không phải để giới hạn việc CHẠY CHUỖI.
///
/// 🔴 **ĐO, KHÔNG CHỈ KHAI (vòng rà 2026-09-06).** Một lượt mở màn xem trước chạy TỐI ĐA
/// SÁU lượt `run_pipeline` trên TOÀN văn bản — năm ứng viên FR126 (`encoding_candidate_wire`)
/// HOẶC một lượt tự khai (`self_declared_cleanup`), không bao giờ cả sáu trong CÙNG một lệnh
/// (hai nhánh loại trừ nhau — xem `match shape` ở `preview_import_encoding`). Đo thật bằng
/// `cleanup_contract.rs::perf_probe_six_full_pipeline_runs_on_one_large_chapter` trên một
/// Chương 440.000 byte (5.000 lần lặp một câu tiếng Trung, VƯỢT XA chương thật lớn nhất từng
/// thấy — xem `deferred-work.md`: 351 ký tự) cộng năm luật (ba literal, hai regex): đường 5
/// ứng viên (5 lượt `run_pipeline`) tốn **~63-83 ms TOÀN BỘ** (ba lượt đo lặp lại, máy phát
/// triển của Ice, không tải nền) — **~13-17 ms/lượt**; đường tự khai (1 lượt) tốn cùng cỡ độ
/// lớn cho MỘT lượt (~63-81 ms — biến thiên giữa các lần đo lớn hơn phần chia đều cho 5 lượt
/// kia, chưa tách được bao nhiêu là chi phí khởi động một lần/lượt cố định so với chi phí
/// tuyến tính theo kích cỡ văn bản). Ở quy mô này, tổng chi phí một lượt mở màn xem trước
/// nằm dưới một khung hình ở 60 Hz (~16 ms) TÍNH TRÊN MỖI lượt `run_pipeline`, và dưới một
/// phần mười giây cho TOÀN BỘ dải năm ứng viên — không cần ghi nợ hiệu năng ở quy mô đo
/// được hôm nay.
///
/// 🔵 **ĐO LẠI 2026-09-05 (Story 6.6) — nguồn NHIỀU CHƯƠNG THẬT, không suy tuyến tính từ số
/// cũ.** Mốc trên đo TRÊN MỘT Chương; story này thêm khối tách Chương (tầng 4,
/// [`ChapterSplitPreviewWire`]) và số Chương thật có thể lên tới 2.000 (trần I/O Matrix spec
/// 6.6, hàng "Xác nhận N Chương"). `cleanup_contract.rs::perf_probe_chapter_split_preview_on_two_thousand_chapters`
/// dựng đúng 2.000 Chương (~125.000 byte, một mẫu regex + một luật literal) và đo đường tự
/// khai (1 lượt `run_pipeline`, CỘNG THÊM việc dựng khối tách cho cả 2.000 Chương): **~42-45
/// ms** (ba lượt đo lặp lại, máy phát triển của Ice, không tải nền, build KHÔNG tối ưu —
/// `cargo test` mặc định, không `--release`). Chi phí THÊM của khối tách Chương so với một
/// lượt `run_pipeline` không-tách-Chương cùng cỡ dữ liệu là NHỎ so với tổng — số đo không
/// tách riêng được hai phần vì chúng chạy trong CÙNG một lượt gọi, và tách riêng đòi hai lượt
/// đo trên hai bản build khác nhau (không làm ở đây, không cần thiết ở quy mô này).
///
/// 🔴 **SỬA (vòng rà đối kháng 3, mục 7) — "45 ms vẫn dưới một phần mười giây" đã bị RÚT LẠI,
/// KHÔNG suy tuyến tính từ số trên.** Số ~42-45 ms ngay trên chỉ đo đường TỰ KHAI
/// (`ChapterInput::AlreadyText`, ĐÚNG MỘT lượt `run_pipeline`) — đường FR126 THẬT (`RawBytes`,
/// dò bảng mã) gọi [`encoding_candidate_wire`] → [`cleanup_and_chapters_preview_for`] MỘT LẦN
/// CHO MỖI ứng viên trong NĂM ([`ImportEncodingPreview::candidates`] luôn đủ năm khi có byte
/// để dò), tức CÙNG khối lượng việc (2.000 Chương, một luật) chạy tối đa NĂM LẦN mỗi lượt tải
/// màn xem trước — không phải một. Suy tuyến tính (`5 × 45 ms ≈ 225 ms`) bị CẤM tự ý tin mà
/// không đo (Ice, 2026-09-05); `cleanup_contract.rs::perf_probe_chapter_split_preview_on_five_candidates_with_two_thousand_chapters`
/// đo THẲNG đường năm ứng viên trên CÙNG khối lượng: **~242-286 ms TOÀN BỘ** (bốn lượt đo lặp
/// lại, máy phát triển của Ice, không tải nền, build DEBUG không tối ưu) — cùng cỡ độ lớn với
/// suy tuyến tính (không lệch bậc), nhưng là số ĐO ĐƯỢC, không phải số SUY RA. Ở quy mô 2.000
/// Chương, đường năm ứng viên KHÔNG còn dưới một phần mười giây — dưới một phần BA giây. Đây
/// vẫn là biên TRÊN hiếm gặp (I/O Matrix spec 6.6 gọi 2.000 Chương là TRẦN, không phải trung
/// vị); không ghi nợ hiệu năng mới ở mức đo được hôm nay, chỉ SỬA câu khẳng định cho khớp con
/// số đã đo. **Chưa đo**: một Chương ĐƠN LẺ ở quy mô hàng MB (nếu FR124/Epic
/// 6 sau này cho phép một Chương lớn hơn nhiều so với một chương tiểu thuyết điển hình), và
/// build `--release` (số đo trên là DEBUG, chậm hơn đáng kể so với bản phát hành thật) — nếu
/// ngày đó tới, đo lại trước khi tin, đừng suy tuyến tính từ con số ở đây.
///
/// Nay `shape` mang
/// TOÀN VĂN BẢN thật (byte thô CẢ tệp cho ứng viên, hoặc chuỗi tự khai CẢ Chương), và
/// `display_window` (bản dựng ĐÃ CẮT, cùng cửa sổ tầng 1 hiển thị) chỉ còn dùng để GIỚI HẠN
/// những gì hiện trên màn hình — không tham gia phép đo.
///
/// Mẫu `regex` không biên dịch được (không nên xảy ra — đã biên dịch thử lúc lưu, xem
/// `core::cleanup::store::validate_pattern`) rơi về báo cáo RỖNG thay vì làm vỡ cả màn xem
/// trước: người dùng vẫn thấy văn bản, chỉ mất phần đánh dấu của LƯỢT NÀY. Byte không giải mã
/// được với bảng mã của MỘT ứng viên (`Err(ImportError::UndecodableBytes)`, có thể xảy ra ở
/// một chỗ SAU cửa sổ bằng chứng — trước lượt sửa này không đường nào chạm tới đó để mà lộ
/// ra) rơi về CÙNG một báo cáo rỗng, không làm vỡ dải năm ô.
/// 🔵 **SỬA 2026-09-05 (Story 6.6) — nhận thêm `chapter_pattern`, trả kèm khối tách Chương.**
/// `outcome.chapters` giờ đọc TOÀN BỘ (không còn `.into_iter().next()`) — mẫu phân tách có
/// thể cho ra N Chương, và khối tách Chương (tầng 4) cần cả N để hiện title/độ dài từng
/// Chương. Vẫn ĐÚNG MỘT lượt `run_pipeline` cho cả hai khối (làm sạch + tách Chương).
///
/// ─────────────────────────────────────────────────────────────────────────────
/// 🔴 VÌ SAO `count_in_import` = TỔNG `per_rule_counts` QUA MỌI CHƯƠNG, KHÔNG MỘT LƯỢT
/// `cleanup::apply` THỨ HAI
/// ─────────────────────────────────────────────────────────────────────────────
/// 🔵 **SỬA 2026-09-09 (Story 6.13) — tên cổng đổi, "một" đã hai lần hết đúng.** Cổng nay tên
/// `cleanup_boundary.rs::the_cleanup_apply_function_has_exactly_four_named_product_call_sites`
/// (Story 6.11 thêm `anchor::compute_anchor`, Story 6.13 thêm `anchor::compute_block_prefix_len`
/// — cả hai đều là bản chạy lại CÙNG bước 3 trên một TIỀN TỐ, không phải một lượt tính lại cho
/// CHÍNH đoạn văn ở đây). Lý lẽ nguyên văn dưới đây (2026-09-06) không đổi: nó vẫn khoá đúng
/// `core::cleanup::apply` ở một TẬP ĐÃ ĐẶT TÊN — một bản nháp sớm của hàm này gọi `apply` LẦN
/// THỨ HAI ở đây để tính lại số khớp riêng từng Chương, và cổng đó ĐỎ NGAY (đo 2026-09-06:
/// `cargo test --test cleanup_boundary` báo 2 chỗ gọi, đòi đúng 1). Thay vào đó, `count_in_import`
/// CHỈ CỘNG DỒN các `per_rule_counts` mà
/// CHÍNH `Step::CleanByRules` đã tính — không tính lại gì. Với [`PipelineShape::Chapters`]
/// (N đơn vị NGAY TỪ ĐẦU, `already_chaptered = true`), bước 3 lặp `apply` một lần cho MỖI
/// đơn vị (một chỗ gọi nguồn, N lần chạy) nên `outcome.chapters[i].cleanup_report` là báo cáo
/// THẬT của riêng Chương đó — tổng của chúng đúng là `count_in_import`, đóng nợ
/// `deferred-work.md:9535`.
///
/// ─────────────────────────────────────────────────────────────────────────────
/// ⚠️ GIỚI HẠN THẬT — mẫu phân tách (`Blob` + `chapter_pattern`) KHÔNG đạt độ chi tiết đó
/// ─────────────────────────────────────────────────────────────────────────────
/// Bước 3 (`CleanByRules`) đứng TRƯỚC bước 5 (`SplitChapters`) trong `PIPELINE_ORDER` — với
/// hình dạng `Blob`, luật làm sạch chạy trên TOÀN blob dưới dạng MỘT đơn vị, TRƯỚC KHI mẫu
/// phân tách cắt nó ra N Chương ở bước 5. Không có cách nào — trong ĐÚNG một chỗ gọi
/// `apply`, với `PIPELINE_ORDER` không đổi — để biết TRƯỚC bước 3 rằng văn bản sẽ tách thành
/// N Chương. `pipeline::split_chapters_step` vì thế GIỮ LẠI báo cáo của TOÀN blob và gán nó
/// cho Chương ĐẦU (`ord = 1`) thay vì vứt nó (bản trước RESET nó về `None`, làm
/// `count_in_chapter` hiện SAI SỐ 0 cho một luật thật sự có khớp — một hồi quy tệ hơn cả
/// "yếu"). Kết quả: `count_in_chapter`/`count_in_import` LUÔN BẰNG NHAU khi N Chương đến từ
/// `chapter_pattern` (đúng số TOÀN tài liệu, không sai, chỉ không CHI TIẾT theo Chương) —
/// khác `PipelineShape::Chapters` ở trên, nơi hai số THẬT SỰ khác nhau khi có ý nghĩa để khác
/// nhau. **Chủ của việc chi tiết hoá theo Chương cho `chapter_pattern`:** cần một cơ chế
/// theo dõi vị trí xuyên bước chuẩn hoá (bước 4) chưa tồn tại — ghi vào `deferred-work.md`,
/// không phải việc của story này.
/// 🔴 **`pub`, không riêng tư (vòng nghiệm thu 2026-09-06)** — bản đầu để hàm này riêng tư và
/// đối chứng nợ `deferred-work.md:9535` tự cộng `per_rule_counts` NGAY TRONG ca test, so với
/// số tính tay: ca đó khẳng định phép cộng CỦA CHÍNH CA TEST đúng, không khẳng định phép cộng
/// mà HÀM NÀY (chỗ SẢN PHẨM thật sinh `count_in_import`) làm ra — một đột biến đổi dòng gán
/// `count_in_import` bên dưới đi thẳng vào một hằng số vẫn để ca đó XANH. Cùng khuôn hai lớp
/// `src-tauri/AGENTS.md` (hàm thuần `pub`, `tests/**` gọi trực tiếp không cần webview) mà
/// `resolve_chapter_pattern` đã theo — xem `cleanup_contract.rs::count_in_import_equals_the_hand_counted_sum_of_count_in_chapter_across_n_chapters_with_different_match_counts`.
/// **THÊM tham số `extract_main_content` 2026-09-06 (Story 6.7).** `shape` ở đây thường là
/// một `PipelineShape::Blob` được GÓI LẠI từ byte của MỘT đơn vị (xem chỗ gọi ở
/// [`encoding_candidate_wire`]) — kể cả khi shape GỐC (trước khi gói) là
/// `PipelineShape::Chapters` (đường URL). Vì vậy hàm này KHÔNG suy `extract_main_content` từ
/// hình dạng `shape` nhận được (luôn `Blob` sau khi gói) — chỗ gọi phải truyền tường minh,
/// tính từ hình dạng GỐC (`preview_import_encoding`, tham số cùng tên).
/// 🔴 **THÊM tham số `block_overrides` 2026-09-07 (Story 6.9).** Cùng lý do `extract_main_content`
/// ở trên: `shape` GÓI LẠI thành `Blob` bên trong [`encoding_candidate_wire`], nhưng override
/// là trạng thái của người dùng cho ĐƠN VỊ ĐẦU TIÊN của hình dạng GỐC — chỗ gọi truyền tường
/// minh từ `Tier2BlockOverridesState`, không suy từ `shape` đã gói.
/// 🔴 **THÊM tham số `detail_chapter_index` 2026-09-08 (Story 6.10a) — thay `chapters.first()`
/// bằng một CON TRỎ.** Trước bản sửa này, chi tiết tầng 2/3 (`blocks_wire`/`final_text`/báo
/// cáo làm sạch dựng `text`+`spans`) LUÔN lấy từ Chương ĐẦU (`chapters.first()`), bất kể
/// `shape` mang bao nhiêu Chương — dữ liệu của Chương 2..N đã được `run_pipeline` tính RỒI
/// (nó chạy trên TOÀN `shape`) nhưng bị VỨT ngay tại đây (nguyên nhân ① của spec 6.10a).
/// `chapters_wire` (tóm tắt — MỌI Chương) và `import_totals` (tổng CẢ lần nhập) KHÔNG đổi,
/// vẫn tính trên TOÀN `chapters` — chỉ phần CHI TIẾT (`blocks_wire`/`final_text_full`/
/// `chapter_report`) đổi từ `chapters.first()` sang `chapters.get(detail_chapter_index)`.
/// `detail_chapter_index` ngoài phạm vi (Chương không tồn tại — mẫu phân tách vừa đổi làm N
/// đổi, hoặc `run_pipeline` trả rỗng) ⇒ chi tiết RỖNG (`blocks_wire = None`,
/// `chapter_report = None`, `final_text` rơi về `display_window`) — CÙNG hình dạng "không có
/// gì để hiện" mà chỗ gọi vốn đã xử lý cho ca `chapters.first() == None`, không một nhánh lỗi
/// mới.
/// 🔴 **THÊM tham số `broken_item_count` 2026-09-08 (Story 6.10).** Số mục URL hỏng của CẢ
/// lượt nhập — chỉ có nghĩa cho đường URL; mọi chỗ gọi khác (đường tệp/dán tay, chi tiết LAZY
/// của `chapter_detail_for_index`) truyền `0` (§Always spec 6.10: "đường tệp/dán tay truyền
/// 0"). Đi thẳng vào [`build_chapter_split_preview_wire`] — hàm này không cộng/trừ gì lên nó.
pub fn cleanup_and_chapters_preview_for(
    shape: PipelineShape,
    encoding: &'static encoding_rs::Encoding,
    chapter_pattern: Option<&ChapterPattern>,
    display_window: &str,
    source_lang: &str,
    cleanup_rules: &[CleanupRule],
    window_truncated: bool,
    extract_main_content: bool,
    block_overrides: &[Option<bool>],
    detail_chapter_index: usize,
    broken_item_count: usize,
    // 🔴 THÊM 2026-09-10 (Story 6.15) — cùng khuôn `block_overrides` ngay trên, nhưng theo
    // CHỈ SỐ CHƯƠNG (không giới hạn Chương đầu) — xem `ChapterOriginOverridesState`.
    origin_overrides: &[Option<ChapterOriginOverride>],
) -> (CleanupPreviewWire, ChapterSplitPreviewWire, Option<ChapterBlocksPreviewWire>) {
    let input = PipelineInput::with_encoding(shape, encoding, source_lang)
        .with_cleanup_rules(cleanup_rules.to_vec())
        .with_chapter_pattern(chapter_pattern.cloned())
        .with_extract_main_content(extract_main_content)
        .with_block_overrides(block_overrides.to_vec());

    let chapters = match run_pipeline(input) {
        Ok(outcome) => outcome.chapters,
        Err(err) => {
            eprintln!("import[preview] chuoi pipeline that bai, roi ve bao cao rong: {err}");
            Vec::new()
        }
    };

    let chapters_wire = build_chapter_split_preview_wire(&chapters, broken_item_count, origin_overrides);
    let detail_chapter = chapters.get(detail_chapter_index);
    // 🔴 SỬA (vòng rà đối kháng bước 4, P1) — `block_overrides` CHỈ có nghĩa cho đơn vị 0 của
    // `shape` GỐC (`PipelineInput::block_overrides` doc-comment: "Chỉ `units[0]` đọc trường
    // này"), nhưng `build_chapter_blocks_preview_wire` áp nó THEO CHỈ SỐ lên bất kỳ `chapter`
    // nào được truyền vào — không tự biết đó có phải đơn vị 0 hay không. Trước bản vá này,
    // con trỏ ở Chương k > 0 mà đã có override từ Chương 0 (`Space`/`[`/`]`) sẽ làm `kept` của
    // Chương k bị bẻ theo override đó, và tệ hơn, `confirmed` báo "người dùng ĐÃ XÁC NHẬN"
    // cho một khối chưa ai từng chạm — một lời khai KHÔNG THẬT trên màn hình. Quyết định
    // "override có áp được không" phải sống Ở ĐÂY (nơi biết `detail_chapter_index`), không
    // nhét thêm một tham số chỉ số vào `build_chapter_blocks_preview_wire` (hàm đó không cần
    // biết NGỮ CẢNH gọi, chỉ cần biết override nào được phép dùng).
    let effective_block_overrides: &[Option<bool>] =
        if detail_chapter_index == 0 { block_overrides } else { &[] };
    let blocks_wire = build_chapter_blocks_preview_wire(detail_chapter, effective_block_overrides);

    // 🔵 SỬA (vòng rà đối kháng bước 4, P8) — đổi tên từ `chapter0_report`: biến này nay giữ
    // báo cáo làm sạch của Chương BẤT KỲ mà `detail_chapter_index` (con trỏ) trỏ tới, không
    // còn LUÔN là Chương 0 (tên cũ hết đúng từ khi Story 6.10a thêm chỉ số con trỏ ở trên).
    let (final_text_full, detail_chapter_report) = match detail_chapter {
        Some(chapter) => (chapter.source_text.clone(), chapter.cleanup_report.clone()),
        None => (display_window.to_owned(), None),
    };

    let mut import_totals: std::collections::BTreeMap<crate::core::cleanup::CleanupRuleKey, usize> =
        std::collections::BTreeMap::new();
    for chapter in &chapters {
        if let Some(report) = &chapter.cleanup_report {
            for (key, count) in &report.per_rule_counts {
                *import_totals.entry(*key).or_insert(0) += count;
            }
        }
    }

    let cleanup_wire = build_cleanup_preview_wire(
        display_window,
        final_text_full,
        cleanup_rules,
        detail_chapter_report,
        import_totals,
        window_truncated,
    );
    (cleanup_wire, chapters_wire, blocks_wire)
}

/// Dựng [`CleanupPreviewWire`] từ một [`crate::core::cleanup::CleanupReport`] ĐÃ CÓ (hoặc
/// `None` khi bước 3 không tạo được báo cáo) — tách khỏi
/// [`cleanup_and_chapters_preview_for`] để chỗ gọi KHÔNG chạy chuỗi (báo cáo rỗng) dùng lại
/// được đúng phép dựng hình dạng dây.
///
/// `display_window`: bản dựng ĐÃ CẮT AN TOÀN của văn bản TRƯỚC khi xoá gì (cùng cửa sổ tầng
/// 1 hiển thị) — trở thành `text` trên dây thẳng, không qua `report` (đóng khuyết tật
/// 2026-09-06: `report.matches`/`.per_rule_counts` đo trên TOÀN văn bản, nhưng `text` hiển thị
/// thì KHÔNG được phép dài hơn cửa sổ). `final_text_full` là văn bản CUỐI CÙNG của TOÀN
/// Chương — cắt xuống cửa sổ CHỈ khi `window_truncated` (giữ nguyên bất biến "không cắt khi
/// không cần cắt" mà `preview_and_confirm_agree_byte_for_byte_on_the_same_input_and_the_same_rules`
/// khoá). `import_totals` — **THÊM 2026-09-05 (Story 6.6)** — tổng số khớp MỖI luật qua TOÀN
/// lần nhập; CÓ THỂ khác `report`'s per-chapter counts khi N > 1 đến từ
/// [`PipelineShape::Chapters`] (🟡 đóng MỘT PHẦN nợ `deferred-work.md:9535` — hình dạng đó
/// chưa có đường sản phẩm nào dựng ra). Trên đường sản phẩm THẬT (`Blob` + `chapter_pattern`)
/// và khi N = 1, chỗ gọi truyền CÙNG map với `report.per_rule_counts` nên hai số bằng nhau
/// như trước story (không đổi hành vi đường cũ).
fn build_cleanup_preview_wire(
    display_window: &str,
    final_text_full: String,
    cleanup_rules: &[CleanupRule],
    report: Option<crate::core::cleanup::CleanupReport>,
    import_totals: std::collections::BTreeMap<crate::core::cleanup::CleanupRuleKey, usize>,
    window_truncated: bool,
) -> CleanupPreviewWire {
    // Chỉ luật ĐANG BẬT được phép xuất hiện trong `spans` — luật tắt vẫn đếm (§Always spec
    // 6.5), nhưng KHÔNG được đánh dấu gạch ngang trong văn bản (I/O Matrix: "Tắt một luật ⇒
    // chỗ vừa gạch ngang trở về nguyên trạng NGAY").
    let enabled_keys: std::collections::BTreeSet<(CleanupRuleTier, i64)> =
        cleanup_rules.iter().filter(|r| r.enabled).map(|r| (r.tier, r.id)).collect();

    let (matches, counts) = match report {
        Some(r) => (r.matches, r.per_rule_counts),
        None => (Vec::new(), std::collections::BTreeMap::new()),
    };

    // `matches` đo trên TOÀN văn bản (xem doc-comment hàm này). Một chỗ khớp có thể đứng ở
    // BA vị trí so với biên cửa sổ hiển thị (`visible_chars`): ① trọn TRONG cửa sổ
    // (`end <= visible_chars`) ⇒ giữ nguyên; ② trọn NGOÀI cửa sổ (`start >= visible_chars`)
    // ⇒ không có gì để mà gạch ngang, bỏ khỏi `spans` — nó vẫn được ĐẾM ở `rules[]` bên dưới
    // (đọc thẳng từ `counts`, không đi qua bộ lọc này); ③ **VẮT QUA BIÊN**
    // (`start < visible_chars < end`) ⇒ 🔴 SỬA vòng rà (2026-09-06) — bản trước lọc theo
    // `m.end <= visible_chars`, nên ca ③ bị coi như ca ② và loại BỎ HẲN, dù phần đầu chỗ khớp
    // vẫn đang HIỆN trên màn hình. Hậu quả: chữ đang hiện, sẽ bị xoá lúc xác nhận, mà KHÔNG
    // mang gạch ngang — thủng đúng lời hứa cốt lõi FR124 ("hiện thứ sắp xoá"). Nay CẮT `end`
    // về đúng biên (`end.min(visible_chars)`) rồi VẪN gạch ngang phần còn nằm trong cửa sổ.
    let visible_chars = display_window.chars().count();
    let spans = matches
        .into_iter()
        .filter(|m| enabled_keys.contains(&(m.rule_tier, m.rule_id)) && m.start < visible_chars)
        .map(|m| CleanupSpanWire {
            tier: m.rule_tier.into(),
            id: m.rule_id,
            start: m.start,
            end: m.end.min(visible_chars),
        })
        .collect();

    // 🔵 SỬA 2026-09-05 (Story 6.6) — `count_in_import` đọc từ `import_totals` (tổng qua N
    // Chương khi mẫu phân tách cho N > 1), KHÔNG còn LUÔN bằng `count_in_chapter` — đóng nợ
    // `deferred-work.md:9535`. Xem doc-comment `cleanup_and_chapters_preview_for` cho cách
    // `import_totals` được tính.
    let rules = cleanup_rules
        .iter()
        .map(|rule| {
            let key = (rule.tier, rule.id);
            let count_in_chapter = counts.get(&key).copied().unwrap_or(0);
            let count_in_import = import_totals.get(&key).copied().unwrap_or(0);
            CleanupRuleReportWire {
                tier: rule.tier.into(),
                id: rule.id,
                pattern: rule.pattern.clone(),
                kind: rule.kind.as_str().to_owned(),
                enabled: rule.enabled,
                count_in_chapter,
                count_in_import,
            }
        })
        .collect();

    // `final_text_full` là văn bản CUỐI CÙNG của TOÀN Chương — cắt xuống cửa sổ CHỈ khi
    // `window_truncated`; khi KHÔNG cắt, trả nguyên vẹn (bất biến byte-for-byte với đường
    // `confirm_import_with_encoding` — không được nới ở đây).
    let final_text = if window_truncated {
        crate::core::segment::normalize::window_safe_prefix(&final_text_full, display_window.len())
            .unwrap_or_default()
    } else {
        final_text_full
    };

    CleanupPreviewWire { text: display_window.to_owned(), spans, rules, window_truncated, final_text }
}

/// Dựng [`EncodingCandidateWire`] TRỌN VẸN (thay `impl From<EncodingCandidate>` cũ — khối
/// làm sạch cần `source_lang`/`cleanup_rules`, hai tham số một `From` không nhận được).
///
/// 🔵 **SỬA 2026-09-08 (Story 6.10a) — nhận `shape: &PipelineShape` (hình dạng GỐC NGUYÊN
/// VẸN), KHÔNG còn `full_bytes`/`label` bị gói lại thành MỘT đơn vị `Blob`.** Đây LÀ nguyên
/// nhân ② của spec 6.10a: bản trước LUÔN gói byte của đơn vị ĐẦU (`chapters.first()`) thành
/// `PipelineShape::Blob` cho MỌI hình dạng ngoài, kể cả khi hình dạng NGOÀI là
/// `PipelineShape::Chapters` mang N > 1 đơn vị (đường URL) — Chương 2..N vì thế KHÔNG BAO GIỜ
/// đi qua `run_pipeline` ở lượt xem trước. Nay hàm này CLONE nguyên `shape` (đã mang ĐỦ N đơn
/// vị khi là `Chapters`) và nạp THẲNG vào [`cleanup_and_chapters_preview_for`] — `label` không
/// còn là tham số riêng (mỗi `ChapterInput` trong `shape` đã tự mang nhãn của nó). Dò bảng mã
/// (`c` — kết quả `encoding::render_candidates`) GIỮ NGUYÊN chốt từ đơn vị ĐẦU (§Always: "một
/// bảng mã cho cả danh sách") — chỗ gọi (`preview_import_encoding`) không đổi vế đó.
/// 🔵 **SỬA 2026-09-05 (Story 6.6)** — nhận thêm `chapter_pattern`, trả kèm `chapters`.
/// 🔴 **THÊM tham số `broken_item_count` 2026-09-08 (Story 6.10)** — cùng lý do tham số cùng
/// tên ở [`cleanup_and_chapters_preview_for`]; chỗ gọi (`preview_import_encoding`) truyền
/// tường minh, không suy từ `shape`.
fn encoding_candidate_wire(
    c: EncodingCandidate,
    shape: &PipelineShape,
    source_lang: &str,
    cleanup_rules: &[CleanupRule],
    chapter_pattern: Option<&ChapterPattern>,
    extract_main_content: bool,
    block_overrides: &[Option<bool>],
    broken_item_count: usize,
    origin_overrides: &[Option<ChapterOriginOverride>],
) -> EncodingCandidateWire {
    // `pipeline_window`/`normalized` đồng bộ `Some`/`None` với nhau (cả hai tính từ
    // CÙNG `decoded.as_ref()` bên trong `render_candidates`) — an toàn đọc `window_truncated`
    // từ `normalized` khi `pipeline_window` có giá trị.
    let window_truncated = c.normalized.as_ref().is_some_and(|n| n.window_truncated);
    let (cleanup, chapters, blocks) = match c.pipeline_window.as_deref() {
        Some(window) => match encoding::encoding_for_wire_id(c.wire_id) {
            Some(encoding) => {
                // 🔴 Chi tiết (tầng 2/3) LUÔN của Chương 0 ở lượt tải EAGER này — chi tiết
                // của một Chương KHÁC đến từ lệnh IPC lazy `preview_chapter_detail` (Story
                // 6.10a, §Design Notes "tóm tắt eager, chi tiết lazy") khi con trỏ dời, KHÔNG
                // từ việc lặp lại năm ứng viên này cho mọi Chương.
                let (cleanup_wire, chapters_wire, blocks_wire) = cleanup_and_chapters_preview_for(
                    shape.clone(),
                    encoding,
                    chapter_pattern,
                    window,
                    source_lang,
                    cleanup_rules,
                    window_truncated,
                    extract_main_content,
                    block_overrides,
                    0,
                    broken_item_count,
                    origin_overrides,
                );
                (Some(cleanup_wire), Some(chapters_wire), blocks_wire)
            }
            // Không nên xảy ra — `c.wire_id` đến từ `Encoding::name()` của chính một trong
            // năm bảng mã FR126, luôn phân giải lại được. Rơi về báo cáo rỗng thay vì làm vỡ
            // cả dải, giữ đúng khuôn dung thứ lỗi của hàm này.
            None => (
                Some(build_cleanup_preview_wire(
                    window,
                    window.to_owned(),
                    cleanup_rules,
                    None,
                    std::collections::BTreeMap::new(),
                    window_truncated,
                )),
                Some(build_chapter_split_preview_wire(&[], broken_item_count, &[])),
                None,
            ),
        },
        None => (None, None, None),
    };

    EncodingCandidateWire {
        label: c.label.to_owned(),
        encoding: c.wire_id.to_owned(),
        preview: c.preview,
        normalized: c.normalized.map(NormalizedPreviewWire::from),
        cleanup,
        chapters,
        blocks,
    }
}

/// **Hàm thuần** — dò bảng mã cho `shape` VỪA ĐỌC (không tự đọc gì, không tự lưu state —
/// chỗ gọi ở `mod wire` chịu trách nhiệm cả hai việc đó, đúng khuôn hai lớp
/// `src-tauri/AGENTS.md`).
///
/// I/O Matrix spec 6.3: `AlreadyText` (văn bản dán tay) ⇒ tự khai, KHÔNG byte nào để dò
/// ⇒ dải rỗng thật (không phải bị giấu). `RawBytes` ⇒ [`encoding::detect`] CỘNG
/// [`encoding::render_candidates`] LUÔN LUÔN — xem doc-comment [`ImportEncodingPreview::candidates`]
/// cho lý do "luôn đủ năm ô" bất kể `confidence`.
///
/// 🔵 **THÊM 2026-09-04 (Story 6.4) — tham số `source_lang`.** [`encoding::render_candidates`]
/// cần nó để dựng bản chuẩn hoá của mỗi ứng viên (vị từ kết câu + dấu nối rẽ nhánh
/// Trung/Anh). KHÔNG phải một lượt đọc thêm: `source_lang` đã có sẵn ở tầng frontend trước
/// khi màn xem trước mở (`sourceLang` của form nhập, `src/modes/libraryImport.ts`).
/// 🔵 **THÊM 2026-09-05 (Story 6.5) — tham số `cleanup_rules`.** Luật làm sạch ĐÃ PHÂN GIẢI
/// (hai tầng đã hợp nhất ở `mod wire`, xem `core::cleanup::store::resolve_two_tiers`) —
/// mỗi ứng viên VÀ đường tự khai nay chạy qua chuỗi pipeline thật (`run_pipeline`) để tính
/// khối làm sạch (tầng 3), đóng nợ `deferred-work.md:9359`.
/// 🔴 **THÊM tham số `block_overrides` 2026-09-07 (Story 6.9).** Chỉ có nghĩa cho đơn vị ĐẦU
/// của `PipelineShape::Chapters` (đường URL); nhánh `Blob`/tự khai truyền `&[]` (không bao
/// giờ đọc tới).
/// 🔴 **THÊM tham số `broken_item_count` 2026-09-08 (Story 6.10).** Số mục URL hỏng của CẢ
/// lượt nhập — chảy vào `encoding_candidate_wire` cho MỌI ứng viên trên nhánh CÓ byte để dò
/// (`Blob(RawBytes)`/`Chapters`); nhánh TỰ KHAI (`AlreadyText`) luôn hardcode `0` bên trong
/// hàm này (§Always spec 6.10: nhánh đó CHÍNH LÀ "đường dán tay" — không bao giờ là đường
/// URL, xem doc-comment `PipelineInput::extract_main_content`). Chỗ gọi trên đường tệp/dán
/// tay (`wire::preview_import_encoding_from_text`/`_from_file`) truyền `0`; đường URL
/// (`url_import_encoding_preview`) truyền số mục `error.is_some()` của `UrlImportItemsState`.
pub fn preview_import_encoding(
    shape: &PipelineShape,
    source_lang: &str,
    cleanup_rules: &[CleanupRule],
    chapter_pattern: Option<&ChapterPattern>,
    block_overrides: &[Option<bool>],
    broken_item_count: usize,
    origin_overrides: &[Option<ChapterOriginOverride>],
) -> ImportEncodingPreview {
    // 🔵 **SỬA 2026-09-08 (Story 6.10a) — bỏ tham số `label` riêng, thêm `shape` (hình dạng
    // GỐC nguyên vẹn).** `label` từng cần thiết vì hình dạng nạp vào pipeline bị GÓI LẠI
    // thành một đơn vị `Blob` bên trong `encoding_candidate_wire` (mất `label` gốc); nay hàm
    // đó nhận thẳng `shape` (mỗi `ChapterInput` tự mang nhãn của nó), nên tham số `label` rời
    // không còn cần. `extract_main_content` chốt vào HÌNH DẠNG NGOÀI (`shape` của CHÍNH
    // `preview_import_encoding`) — không đổi.
    let verdict_and_candidates = |bytes: &[u8],
                                   extract_main_content: bool|
     -> (EncodingVerdict, Vec<EncodingCandidateWire>) {
        let verdict = encoding::detect(bytes);
        // 🔴 SỬA (vòng rà đối kháng 2, mục 7) — bản trước ép `candidates` RỖNG cho MỌI
        // `SelfDeclared`, gộp CHUNG hai ca khác hẳn nhau dưới MỘT nhãn tin cậy: ① byte RỖNG
        // (không có gì để mà dò — "tự khai THẬT", đúng như doc-comment hàm này ĐÃ khai:
        // *"AlreadyText ⇒ KHÔNG byte nào để dò ⇒ dải rỗng THẬT"*) và ② một BOM đứng trước
        // byte THẬT (`sniff_bom` trả `Some`, `bytes` không rỗng) — case NÀY có đủ byte để dò
        // y hệt ca tin cậy cao/thấp, chỉ là ta CHỌN tin BOM làm mặc định. Ép rỗng ở ca ② biến
        // "dải KHÔNG MỞ mặc định" (quyết định HIỂN THỊ, đúng I/O Matrix) thành "dải KHÔNG
        // TỒN TẠI" (một MẤT MÁT DỮ LIỆU) — khi bảng mã BOM khai KHÔNG giải mã được thật (BOM
        // UTF-16LE đứng trước byte hỏng, ví dụ), lượt xác nhận trượt bằng `UndecodableBytes`
        // và người dùng bấm `E` (`openImportPreviewCandidatePicker`) THẤY NO-OP vì
        // `candidates.length === 0` — NGÕ CỤT, không đường lùi nào ngoài đóng lớp phủ, bỏ cả
        // lượt nhập. Đúng câu doc-comment hàm này đã tự khai ("RawBytes ⇒ detect CỘNG
        // render_candidates LUÔN LUÔN") và đúng doc-comment [`ImportEncodingPreview::candidates`]
        // ("Rỗng CHỈ xảy ra ở nhánh tự khai THẬT (AlreadyText)... không phải 'có nhưng bị
        // giấu'") — mã bây giờ khớp lời khai của chính nó: CHỈ byte RỖNG mới cho dải rỗng
        // thật; một BOM đứng trước byte thật vẫn có đủ năm ô, `E` vẫn mở được nó làm lối
        // thoát khi bảng mã BOM khai hoá ra sai.
        let candidates = if bytes.is_empty() {
            Vec::new()
        } else {
            encoding::render_candidates(bytes, source_lang)
                .into_iter()
                .map(|c| {
                    encoding_candidate_wire(
                        c,
                        shape,
                        source_lang,
                        cleanup_rules,
                        chapter_pattern,
                        extract_main_content,
                        block_overrides,
                        broken_item_count,
                        origin_overrides,
                    )
                })
                .collect()
        };
        (verdict, candidates)
    };

    let self_declared_utf8 = || EncodingVerdict {
        encoding: encoding_rs::UTF_8,
        confidence: Confidence::SelfDeclared,
    };

    let (verdict, candidates) = match shape {
        // `Blob` = đường tệp/dán tay — KHÔNG BAO GIỜ bóc nội dung chính (§Always spec 6.7).
        PipelineShape::Blob(ChapterInput::AlreadyText(_)) => (self_declared_utf8(), Vec::new()),
        PipelineShape::Blob(ChapterInput::RawBytes { bytes, .. }) => {
            verdict_and_candidates(bytes, false)
        }
        // 🔵 **SỬA 2026-09-08 (Story 6.10a) — "nuôi vào pipeline" đổi, "dò bảng mã" GIỮ
        // NGUYÊN ở `chapters.first()`.** §Always story 6.10a: dò bảng mã cho CẢ danh sách vẫn
        // chốt từ đơn vị ĐẦU (không đổi) — vế đổi là `verdict_and_candidates` giờ nạp THẲNG
        // `shape` (mang ĐỦ N đơn vị) vào `encoding_candidate_wire`, không còn gói lại chỉ đơn
        // vị đầu thành `Blob` (nguyên nhân ② của spec 6.10a — Chương 2..N từng KHÔNG BAO GIỜ
        // đi qua `run_pipeline` ở lượt xem trước đường URL).
        PipelineShape::Chapters(chapters) => match chapters.first() {
            Some(ChapterInput::RawBytes { bytes, .. }) => verdict_and_candidates(bytes, true),
            Some(ChapterInput::AlreadyText(_)) | None => (self_declared_utf8(), Vec::new()),
        },
        // 🔴 **THÊM 2026-09-11 (Story 6.16)** — đường song ngữ có màn xem trước RIÊNG
        // (`preview_bilingual_import`), KHÔNG đi qua hàm này — cùng lý do `PipelineShape::Bilingual`
        // không xuất hiện trên đường sản phẩm gọi `preview_import_encoding`. Phòng thủ kiểu,
        // không phải một trạng thái người dùng gây ra được.
        PipelineShape::Bilingual { .. } => (self_declared_utf8(), Vec::new()),
    };

    // 🔴 THÊM 2026-09-04 (Story 6.4, vá vòng rà 1, mục 1) — `candidates` RỖNG (tự khai
    // thật, HOẶC byte rỗng) vẫn phải chở một bản chuẩn hoá. Văn bản nguồn cho nó là chuỗi
    // dán tay THẬT khi có (`AlreadyText`), hoặc chuỗi rỗng khi không có gì để mà tự khai —
    // `normalize::normalize("", ..)` hợp lệ, không phải một ca đặc biệt phải né.
    let self_declared_normalized = candidates.is_empty().then(|| {
        let text = self_declared_source_text(shape);
        NormalizedPreviewWire::from(encoding::normalized_self_declared(text, source_lang))
    });

    // 🔴 THÊM 2026-09-05 (Story 6.5) — cùng điều kiện `Some`/`None` với `self_declared_normalized`
    // ở trên (đọc `window_truncated` từ đó thay vì tính lại — cùng phép đo, một chỗ).
    // 🔵 SỬA 2026-09-05 (Story 6.6) — cũng dựng `self_declared_chapters` CÙNG một lượt.
    let self_declared_pair = self_declared_normalized.as_ref().map(|normalized| {
        let text = self_declared_source_text(shape);
        match encoding::pipeline_window_for_self_declared(text) {
            Some(window) => {
                // 🔴 SỬA 2026-09-06 — `shape` mang văn bản TOÀN VẸN (`text`, không phải
                // `window`) để `cleanup_and_chapters_preview_for` chạy chuỗi trên CẢ Chương;
                // `window` chỉ còn vai trò giới hạn hiển thị.
                let full_shape = PipelineShape::Blob(ChapterInput::AlreadyText(text.to_owned()));
                let (cleanup, chapters, _blocks) = cleanup_and_chapters_preview_for(
                    full_shape,
                    encoding_rs::UTF_8,
                    chapter_pattern,
                    &window,
                    source_lang,
                    cleanup_rules,
                    normalized.window_truncated,
                    // Nhánh TỰ KHAI luôn là văn bản dán tay (`AlreadyText`) — KHÔNG BAO GIỜ
                    // là đường URL (đường đó luôn mang `RawBytes`) — `extract_main_content`
                    // luôn `false` ở đây, cùng lý do `Blob` ở nhánh trên. `_blocks` luôn
                    // `None` (bước 2 không chạy) — nhánh tự khai không có tầng 2, §Never spec
                    // 6.9.
                    false,
                    &[],
                    0,
                    // Nhánh TỰ KHAI không bao giờ có mục URL để mà hỏng (xem doc-comment
                    // tham số `broken_item_count` ở `preview_import_encoding`) — `0` cố định.
                    0,
                    // Nhánh TỰ KHAI luôn `AlreadyText` — `chapter.origin` luôn `None`, không
                    // có gì để mà áp override lên (§Never spec 6.15: tầng 2 chưa mở ở đây).
                    &[],
                );
                (cleanup, chapters)
            }
            // Cùng ca "cửa sổ không đủ một dòng trọn vẹn" của `normalized_self_declared` —
            // `final_text` rỗng đồng bộ với `NormalizedPreviewWire.text == ""` ở đó.
            None => (
                build_cleanup_preview_wire(
                    text,
                    String::new(),
                    cleanup_rules,
                    None,
                    std::collections::BTreeMap::new(),
                    normalized.window_truncated,
                ),
                build_chapter_split_preview_wire(&[], 0, &[]),
            ),
        }
    });
    let (self_declared_cleanup, self_declared_chapters) = match self_declared_pair {
        Some((cleanup, chapters)) => (Some(cleanup), Some(chapters)),
        None => (None, None),
    };

    ImportEncodingPreview {
        confidence: verdict.confidence.into(),
        selected_encoding: verdict.encoding.name().to_owned(),
        candidates,
        self_declared_normalized,
        self_declared_cleanup,
        self_declared_chapters,
    }
}

/// Văn bản THẬT của nhánh tự khai, nếu có — chỗ gọi DUY NHẤT là `preview_import_encoding`,
/// đúng lúc `candidates` đã RỖNG. `RawBytes` không có văn bản (chưa giải mã, và nếu tới đây
/// thì `bytes` đã rỗng — không có gì để mà giải mã) ⇒ chuỗi rỗng, không phải `None`: một
/// chuỗi rỗng qua `normalize::normalize` là một giá trị HỢP LỆ (xem ca ma trận I/O "Chỉ
/// khoảng trắng"), không phải một trường hợp phải tránh gọi.
fn self_declared_source_text(shape: &PipelineShape) -> &str {
    match shape {
        PipelineShape::Blob(ChapterInput::AlreadyText(text)) => text,
        PipelineShape::Chapters(chapters) => match chapters.first() {
            Some(ChapterInput::AlreadyText(text)) => text,
            _ => "",
        },
        _ => "",
    }
}

// ═════════════════════════════════════════════════════════════════════════════════
// Story 6.10a — con trỏ *Chương đang chọn*: chi tiết LAZY (tầng 2 + tầng 3) cho một Chương
// KHÁC Chương 0, qua một lệnh IPC MỚI khi con trỏ dời (`⌥←`/`⌥→`) — xem §Design Notes "Vì sao
// tóm tắt eager mà chi tiết lazy".
// ═════════════════════════════════════════════════════════════════════════════════

/// Cửa sổ hiển thị (`display_window`) cộng cờ cắt cho Chương thứ `chapter_index` của `shape`
/// — cùng con số mà `encoding_candidate_wire` tính cho Chương 0 (qua `EncodingCandidate::
/// pipeline_window`, chính là `render_candidates` gọi trên đơn vị ĐẦU), nhưng cho MỘT bảng mã
/// ĐÃ BIẾT (không dò lại) và một Chương BẤT KỲ trong `shape`.
///
/// `None` khi `chapter_index` ngoài phạm vi, hoặc bảng mã đã chọn "không ra chữ" trên cửa sổ
/// bằng chứng của CHÍNH Chương đó (byte hỏng/chỉ toàn khoảng trắng) — chỗ gọi coi đó là
/// "không có gì để hiện", không đoán.
fn display_window_for_chapter(
    shape: &PipelineShape,
    chapter_index: usize,
    encoding: &'static encoding_rs::Encoding,
    source_lang: &str,
) -> Option<(String, bool)> {
    fn window_for_unit(
        unit: &ChapterInput,
        encoding: &'static encoding_rs::Encoding,
        source_lang: &str,
    ) -> Option<(String, bool)> {
        match unit {
            ChapterInput::RawBytes { bytes, .. } => {
                encoding::pipeline_window_for_known_encoding(bytes, encoding)
            }
            ChapterInput::AlreadyText(text) => {
                let window_truncated = encoding::normalized_self_declared(text, source_lang).window_truncated;
                encoding::pipeline_window_for_self_declared(text).map(|w| (w, window_truncated))
            }
        }
    }

    match shape {
        // 🔴 SỬA (vòng rà đối kháng bước 4, P5) — `Blob` mang ĐÚNG MỘT đơn vị; `chapter_index`
        // khác 0 ở đây không có gì để mà trỏ tới. Bản trước NUỐT tham số này (bỏ qua, luôn
        // đọc `unit` bất kể `chapter_index`) — vô hại HÔM NAY vì chỗ gọi duy nhất
        // (`chapter_detail_for_index`, qua `preview_chapter_detail`) chỉ nhận `chapter_index`
        // từ đường URL (`PipelineShape::Chapters`), nhưng hàm này là `pub`/tổng quát và chữ ký
        // hứa một chỉ số bất kỳ — một chỗ gọi TƯƠNG LAI với `Blob` + `chapter_index > 0` sẽ
        // lặng lẽ nhận cửa sổ của đơn vị DUY NHẤT thay vì một lỗi rõ ràng. Trả `None` cho
        // `chapter_index != 0` — "Chương đó không tồn tại trong `Blob`" là ĐÚNG NGHĨA
        // ("không có gì để hiện", không đoán), khớp quy ước "chỉ số ngoài phạm vi ⇒ `None`"
        // mà chính hàm này đã theo ở nhánh `Chapters`.
        PipelineShape::Blob(unit) => {
            if chapter_index != 0 {
                return None;
            }
            window_for_unit(unit, encoding, source_lang)
        }
        PipelineShape::Chapters(units) => {
            window_for_unit(units.get(chapter_index)?, encoding, source_lang)
        }
        // 🔴 **THÊM 2026-09-11 (Story 6.16)** — đường song ngữ không dùng cơ chế con trỏ
        // *Chương đang chọn* của Story 6.10a (hàng/Chương của nó không có "cửa sổ hiển thị"
        // bảng mã theo nghĩa này) — phòng thủ kiểu, cùng lý do các nhánh `PipelineShape::Bilingual`
        // khác trong tệp này.
        PipelineShape::Bilingual { .. } => None,
    }
}

/// Chi tiết LAZY (tầng 2 + tầng 3) cho Chương thứ `chapter_index` — **Story 6.10a**, lệnh IPC
/// mới đứng cạnh (`wire::preview_chapter_detail`). Chạy LẠI TRỌN VẸN chuỗi pipeline trên
/// `shape` với bảng mã ĐÃ CHỌN (không dò lại, không lặp năm ứng viên FR126) — MỘT lượt
/// `run_pipeline`, không năm — rồi lấy riêng chi tiết của `chapter_index` từ kết quả.
///
/// ⚠️ **Chưa cắt còn một đơn vị — chạy trên TOÀN `shape`.** Một phương án nhanh hơn là cắt
/// `shape` xuống còn ĐÚNG đơn vị `chapter_index` trước khi nạp — nhưng
/// `PipelineInput::block_overrides` CHỈ áp cho `units[0]` (§Story 6.9), và cắt sẽ đặt đơn vị
/// ĐƯỢC YÊU CẦU vào vị trí 0, làm override THẬT của Chương 0 (nếu có) bị áp NHẦM sang bất kỳ
/// Chương nào người dùng đang xem — một lượt ghi SAI Ý NGHĨA không lần ngược được (đúng lớp
/// lỗi Ask First của spec 6.10a cấm). Chạy trên TOÀN `shape` giữ nguyên vị trí thật của mọi
/// đơn vị, nên `block_overrides` vẫn chỉ chạm đúng Chương 0 như bản chất của nó — cái giá là
/// O(N Chương) mỗi lượt dời con trỏ thay vì O(1); đây là một đánh đổi CÓ CHỦ cho story này,
/// không phải một sơ suất — xem Design Notes.
///
/// `None` khi `chapter_index` không còn khớp số Chương thật của lượt chạy MỚI (mẫu phân tách
/// vừa đổi làm N đổi), hoặc bảng mã đã chọn "không ra chữ" cho Chương này — chỗ gọi coi đó là
/// trạng thái CŨ/không hợp lệ, từ chối thay vì đoán.
pub fn chapter_detail_for_index(
    shape: &PipelineShape,
    chapter_index: usize,
    encoding: &'static encoding_rs::Encoding,
    chapter_pattern: Option<&ChapterPattern>,
    source_lang: &str,
    cleanup_rules: &[CleanupRule],
    extract_main_content: bool,
    block_overrides: &[Option<bool>],
) -> Option<(CleanupPreviewWire, Option<ChapterBlocksPreviewWire>)> {
    let (window, window_truncated) =
        display_window_for_chapter(shape, chapter_index, encoding, source_lang)?;
    let (cleanup_wire, chapters_wire, blocks_wire) = cleanup_and_chapters_preview_for(
        shape.clone(),
        encoding,
        chapter_pattern,
        &window,
        source_lang,
        cleanup_rules,
        window_truncated,
        extract_main_content,
        block_overrides,
        chapter_index,
        // Chi tiết LAZY của MỘT Chương — `chapters_wire` ở đây chỉ dùng để đọc
        // `chapter_count` (dòng ngay dưới), không lộ ra ngoài hàm này, nên số mục hỏng không
        // có ý nghĩa để mà truyền (0 an toàn, xem doc-comment tham số `broken_item_count` ở
        // `cleanup_and_chapters_preview_for`).
        0,
        // `chapters_wire` ở đây chỉ đọc `chapter_count`, không lộ ra ngoài hàm này (xem doc-
        // comment ngay trên) — xuất xứ không có nơi để mà hiện, `&[]` không mất gì.
        &[],
    );
    if chapter_index >= chapters_wire.chapter_count {
        return None;
    }
    Some((cleanup_wire, blocks_wire))
}

/// Hình dạng DÂY của kết quả [`chapter_detail_for_index`] — trả về từ `wire::preview_chapter_detail`.
///
/// ⚠️ `#[serde(rename_all = ...)]` KHÔNG đặt — cùng luật mọi kiểu qua biên IPC.
#[derive(Debug, Clone, serde::Serialize)]
pub struct ChapterDetailWire {
    pub cleanup: CleanupPreviewWire,
    pub blocks: Option<ChapterBlocksPreviewWire>,
}

/// `confirm_import_with_encoding` gọi khi [`PendingImportSourceState`] rỗng — hộp thoại xem
/// trước đã bị dọn (huỷ ở tầng giao diện, đóng Tác phẩm, hoặc một lượt xem trước KHÁC đã ghi
/// đè) trước khi lượt xác nhận này tới nơi. Cùng khuôn `GlossaryNoPendingImport`.
fn no_pending_import_source() -> IpcError {
    IpcError::new(
        "import.no_pending_source",
        crate::core::i18n::MessageKey::ImportNoPendingSource,
        std::collections::BTreeMap::new(),
        false,
    )
}

/// **Hàm thuần** — ghi `shape` vào `state`, ghi ĐÈ lượt xem trước cũ nếu có. Đúng khuôn hai
/// lớp `src-tauri/AGENTS.md`: nhận thẳng `&PendingImportSourceState`, không `AppHandle`, để
/// `tests::` gọi được không cần webview. Gọi bởi `wire::preview_import_encoding_from_text`/
/// `_from_file`, NGAY SAU [`preview_import_encoding`] — đúng thứ tự "đọc rồi mới cất" (§Always
/// spec 6.3: byte đọc đúng một lần).
pub fn stash_pending_import_source(
    state: &PendingImportSourceState,
    shape: PipelineShape,
    // 🔴 **THÊM 2026-09-09 (Story 6.12).** `.docx` là đường DUY NHẤT truyền `Some` —
    // `preview_import_encoding_from_text` và mọi đường KHÁC truyền `None`.
    docx_sidecar: Option<crate::core::segment::import::DocxSidecar>,
) {
    let mut guard = state.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
    *guard = Some(PendingImportSource { shape, docx_sidecar });
}

/// **Hàm thuần** — dọn ô đang chờ.
///
/// ─────────────────────────────────────────────────────────────────────────────
/// ⚠️ VÌ SAO KHÔNG CÓ VỎ IPC RIÊNG — "huỷ" LÀ MỘT QUYẾT ĐỊNH TẦNG GIAO DIỆN
/// ─────────────────────────────────────────────────────────────────────────────
/// Task list spec 6.3 chỉ đòi ĐÚNG BA vỏ dây (hai `preview_import_encoding_from_*` cộng
/// `confirm_import_with_encoding`) — không một vỏ thứ tư cho "huỷ". Sản phẩm hôm nay chặn
/// một lượt `confirm` sau khi huỷ hoàn toàn ở TẦNG GIAO DIỆN: `dispatch('import.preview.confirm')`
/// không bao giờ được gọi sau khi `src/importPreviewState.ts::cancelImportPreview()` đã xoá
/// state cục bộ và đóng lớp phủ (defect #5 của vòng rà 1 nằm ở CHÍNH chỗ đó, không ở Rust).
///
/// Hàm này ở lại như một hàm THUẦN, không có chỗ gọi sản phẩm nào (0 chỗ gọi từ `mod wire`),
/// vì đó là điều kiện DUY NHẤT để hàng ma trận I/O *"huỷ rồi xác nhận ⇒ 0 Tác phẩm được
/// tạo"* kiểm được TRONG `tests/**` — không có cách nào khác để một `tests/**` (chỉ gọi
/// hàm thuần, không webview) mô phỏng "người dùng đã huỷ" mà không có một hàm dọn state
/// tường minh. [`tests`] gọi hàm này rồi khẳng định [`confirm_import_with_encoding`] từ chối
/// và 0 thư mục `.atproj` được tạo.
pub fn cancel_import_preview(state: &PendingImportSourceState) {
    let mut guard = state.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
    *guard = None;
}

/// **Hàm thuần** — lõi của lượt xác nhận: giải `encoding_wire_id`, CLONE (không `take`, xem
/// doc-comment [`PendingImportSourceState`]) nguồn đang chờ, gọi [`create_work`], rồi dọn ô
/// đang chờ khi và chỉ khi THÀNH CÔNG. Tách khỏi `wire::confirm_import_with_encoding` đúng
/// khuôn hai lớp — `tests::` gọi được không cần `tauri::AppHandle`.
///
/// # Lỗi
/// - `encoding_wire_id` không giải ngược được ⇒ `import.unrecognized_encoding` (KHÔNG âm
///   thầm rơi về UTF-8, §Design Notes spec 6.3);
/// - `state` rỗng ⇒ `import.no_pending_source`;
/// - [`create_work`] trượt (ví dụ byte không giải mã được với CHÍNH bảng mã đã chọn) ⇒ lỗi
///   của nó, đi thẳng — ô đang chờ GIỮ NGUYÊN (không dọn trên đường lỗi), để một lượt xác
///   nhận KẾ TIẾP với một ứng viên khác không đòi đọc nguồn lần hai.
/// 🔵 **THÊM 2026-09-05 (Story 6.5) — tham số `cleanup_rules`.** CÙNG tập luật (ĐÃ PHÂN
/// GIẢI ở `mod wire`, nạp NGAY LÚC XÁC NHẬN chứ không phải bộ đã dùng lúc xem trước — luật
/// có thể đã đổi giữa hai nhịp qua một lượt bật/tắt/soạn khác) mà [`preview_import_encoding`]
/// vừa dùng để hiện — đây là chỗ đóng nợ `deferred-work.md:9359` cho NỬA GHI: `create_work`
/// nhận đúng luật đó, không một bộ luật thứ hai.
/// 🔴 **THÊM 2026-09-07 (Story 6.9) — tham số `block_overrides`, đúng cái vòng rà 1 đã hụt.**
/// `wire::confirm_import_with_encoding` đọc `Tier2BlockOverridesState` NGAY LÚC XÁC NHẬN
/// (cùng kỷ luật "đọc lại lúc xác nhận, không tái dùng bộ lúc xem trước" mà `cleanup_rules`
/// đã theo) rồi truyền vào đây; state đó chỉ được RESET ở lớp vỏ SAU KHI hàm này trả `Ok`.
/// 🔴 **THÊM 2026-09-08 (Story 6.11, mục B1) — tham số `domain_log_state`.** Đọc doc-comment
/// `create_work` — thread thẳng xuống đó, không tự tích luỹ gì ở tầng này.
pub fn confirm_import_with_encoding(
    documents_root: &Path,
    state: &PendingImportSourceState,
    name: &str,
    source_lang: &str,
    genre: &str,
    encoding_wire_id: &str,
    cleanup_rules: Vec<CleanupRule>,
    chapter_pattern: Option<ChapterPattern>,
    block_overrides: Vec<Option<bool>>,
    // 🔴 **THÊM 2026-09-10 (Story 6.15)** — cùng kỷ luật `block_overrides` ngay trên: đọc
    // `ChapterOriginOverridesState` NGAY LÚC XÁC NHẬN, reset chỉ ở lớp vỏ SAU KHI hàm này trả
    // `Ok`.
    origin_overrides: Vec<Option<ChapterOriginOverride>>,
    domain_log_state: &webimport::DomainLogState,
) -> Result<OpenWork, IpcError> {
    let chosen = encoding::encoding_for_wire_id(encoding_wire_id).ok_or_else(|| {
        IpcError::from(ImportError::UnrecognizedEncoding { wire_id: encoding_wire_id.to_owned() })
    })?;

    // 🔴 SỬA (vòng rà đối kháng 2, mục 14) — GIỮ NGUYÊN một `MutexGuard` cho TRỌN vẹn phần
    // đọc-rồi-ghi, không thả khoá giữa lúc đọc `shape` (clone) và lúc gọi [`create_work`].
    // Bản trước thả khoá ngay sau khi đọc xong `shape` rồi mới gọi `create_work` — hai lượt
    // gọi hàm này CHỒNG NHAU (hai lời gọi IPC đến gần như cùng lúc, ví dụ một bộ fixture
    // e2e gọi thẳng `internals.invoke('confirm_import_with_encoding', …)` hai lần liền,
    // vòng chặn `confirming` ở `importPreviewState.ts` là JS-side, không chắn được một lời
    // gọi thô bỏ qua tầng đó) đều đọc được CÙNG một `shape` (chưa ai xoá), đều
    // `create_work` THÀNH CÔNG ⇒ HAI Tác phẩm từ MỘT nguồn đang chờ. Giữ khoá xuyên suốt
    // biến `confirm_import_with_encoding` thành một đoạn TỚI HẠN đúng nghĩa: lượt THỨ HAI
    // phải đợi lượt thứ nhất xong (kể cả phần ghi đĩa của `create_work`) rồi mới đọc được
    // `state`, lúc đó `guard` đã là `None` (đã dọn ở cuối lượt thứ nhất) ⇒ trả
    // `no_pending_import_source` — TỪ CHỐI SẠCH, không ghi đè, không Tác phẩm thứ hai.
    //
    // Không đổi hành vi CA THƯỜNG (một lượt xác nhận duy nhất, không chồng): `shape` vẫn chỉ
    // bị dọn khi và chỉ khi `create_work` thành công — GIỮ NGUYÊN trên đường lỗi, đúng doc-
    // comment ở trên (lượt xác nhận lại với một ứng viên khác không đòi đọc nguồn lần hai).
    let mut guard = state.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
    let shape = guard.as_ref().map(|p| p.shape.clone()).ok_or_else(no_pending_import_source)?;
    // 🔴 **THÊM 2026-09-09 (Story 6.12)** — CÙNG khoá, CÙNG lượt đọc với `shape` ngay trên
    // (không một `MutexGuard` thứ hai) — `docx_sidecar` phải sống sót/biến mất ĐÚNG lúc
    // `shape` sống sót/biến mất, hai trường của CÙNG MỘT `PendingImportSource`.
    let docx_sidecar = guard.as_ref().and_then(|p| p.docx_sidecar.clone());

    let opened = create_work(
        documents_root,
        name,
        source_lang,
        genre,
        shape,
        chosen,
        cleanup_rules,
        chapter_pattern,
        block_overrides,
        // `confirm_import_with_encoding` phục vụ đường văn xuôi/URL — không bao giờ mang
        // `PipelineShape::Bilingual` (đường đó đi qua `confirm_bilingual_import`, hàm RIÊNG).
        0,
        1,
        false,
        &origin_overrides,
        domain_log_state,
        docx_sidecar,
    )?;

    // Thành công — dọn ô đang chờ, VẪN dưới CÙNG một khoá đã giữ từ đầu hàm.
    *guard = None;

    Ok(opened)
}

// ═════════════════════════════════════════════════════════════════════════════════
// Story 6.16 — Nhập tài liệu song ngữ hai cột (FR115, AD-39 · AD-37/46 · AD-47 ③)
// ═════════════════════════════════════════════════════════════════════════════════
//
// ─────────────────────────────────────────────────────────────────────────────
// 🔴 STATE TÁI DÙNG, KHÔNG MỘT HỘP THỨ BA
// ─────────────────────────────────────────────────────────────────────────────
// Vai cột (`source_column`/`target_column`) và cờ tiêu đề là tham số MỖI LƯỢT xem trước/xác
// nhận, cùng khuôn `chapter_pattern` — KHÔNG một `Mutex<...>` mới cạnh
// `Tier2BlockOverridesState`/`ChapterOriginOverridesState`. `PendingImportSourceState` (đã
// có, Story 6.3) giữ nguyên vai trò: `stash_pending_import_source`/`cancel_import_preview`
// dùng ĐƯỢC NGUYÊN cho `PipelineShape::Bilingual` — chỉ byte thô của tệp được cất, đổi vai
// cột/tiêu đề không đọc lại đĩa (§I/O Matrix "Swap columns"/"Header checkbox on": "counts
// rebuild"/"rebuilds the preview in memory").

/// Một hàng lệch cặp trên dây — Story 6.16.
#[derive(Debug, Clone, serde::Serialize)]
pub struct BilingualMismatchWire {
    pub chapter_index: usize,
    pub row_number: usize,
    pub source_sentence_count: usize,
    pub target_sentence_count: usize,
}

impl From<&crate::core::segment::bilingual::BilingualMismatch> for BilingualMismatchWire {
    fn from(m: &crate::core::segment::bilingual::BilingualMismatch) -> Self {
        BilingualMismatchWire {
            chapter_index: m.chapter_index,
            row_number: m.row_number,
            source_sentence_count: m.source_sentence_count,
            target_sentence_count: m.target_sentence_count,
        }
    }
}

/// Kết quả chạy TRỌN chuỗi bảy bước cho MỘT ứng viên bảng mã — Story 6.16. `chapter_count`/
/// `pair_count`/`mismatches` đều RỖNG/0 khi ứng viên này "không ra chữ" trong cửa sổ bằng
/// chứng (`preview == None`, cùng khuôn `EncodingCandidateWire`).
#[derive(Debug, Clone, serde::Serialize)]
pub struct BilingualEncodingCandidateWire {
    pub label: String,
    pub encoding: String,
    pub preview: Option<String>,
    pub row_count: usize,
    pub chapter_count: usize,
    pub pair_count: usize,
    pub mismatches: Vec<BilingualMismatchWire>,
}

/// Dải năm ứng viên trên dây — Story 6.16, cùng khuôn [`ImportEncodingPreview`].
#[derive(Debug, Clone, serde::Serialize)]
pub struct BilingualImportEncodingPreview {
    pub confidence: ConfidenceWire,
    pub selected_encoding: String,
    pub candidates: Vec<BilingualEncodingCandidateWire>,
    /// Tối đa [`crate::core::segment::pipeline::BILINGUAL_SAMPLE_ROW_CAP`] hàng đầu tiên
    /// của bảng mã ĐANG CHỌN — mọi cột, để webview dựng thẻ chọn cột nguồn/đích với dữ liệu
    /// THẬT thay vì tên cột suông. Rỗng khi tệp rỗng hoặc bảng mã đang chọn không ra chữ.
    pub sample_rows: Vec<Vec<String>>,
    /// Tổng số hàng của bảng mã ĐANG CHỌN — §I/O Matrix "Fewer than 2 columns" cũng lộ ra ở
    /// đây khi chuỗi từ chối: `0` cùng `candidates` rỗng thân trong (chuỗi trả `Err`, không
    /// một ứng viên nào chạy được tới cuối).
    pub row_count: usize,
    /// Số cột rộng nhất đếm được ở bảng mã ĐANG CHỌN — webview dùng để dựng danh sách lựa
    /// chọn cột nguồn/đích (0-based, `0..column_count`). `0` khi tệp rỗng.
    pub column_count: usize,
}

/// **Hàm thuần** — dò bảng mã VÀ chạy TRỌN chuỗi bảy bước cho MỖI ứng viên (AD-39: "table
/// parsing happens right after decode, inside the chain, and re-runs on every encoding
/// candidate"). Cùng khuôn [`preview_import_encoding`]: KHÔNG tự đọc gì, KHÔNG tự lưu
/// state — vỏ `mod wire` cấp `shape` (đọc tệp MỘT LẦN ở `preview_bilingual_import_from_file`,
/// hoặc clone từ ô đang chờ ở `rebuild_bilingual_import_preview`) rồi gọi hàm này.
///
/// `bilingual_source_column`/`bilingual_target_column`/`bilingual_has_header` là tham số MỖI
/// LƯỢT gọi (xem §Design Notes đầu mục) — đổi cột/tiêu đề chỉ đòi gọi lại hàm này với
/// CÙNG `shape` (byte thô không đổi), 0 lượt đọc đĩa thêm.
///
/// `cleanup_rules` — luật làm sạch đã phân giải hai tầng, cùng nguồn mà lượt xác nhận đọc lại
/// lúc xác nhận (§Always spec 6.16: "Cleanup and normalize run per cell, both columns").
///
/// # Lỗi
/// 🔵 **SỬA 2026-09-11 (Story 6.16, bước nghiệm thu)** — bản đầu nuốt MỌI `Err` của chuỗi
/// bằng `.ok()`: một tệp một cột, hay một ô mở ngoặc kép không đóng, vẫn hiện một màn xem
/// trước toàn số 0 với nút xác nhận BẬT, và lỗi chỉ lộ ra khi bấm xác nhận — trái hàng I/O
/// Matrix "Fewer than 2 columns: Refused before preview". Nay hai lỗi HÌNH DẠNG BẢNG của ứng
/// viên ĐANG CHỌN được trả lên (`import.bilingual_too_few_columns`,
/// `import.bilingual_unterminated_quoted_field`); lỗi của các ứng viên KHÁC vẫn chỉ làm ứng
/// viên đó rỗng, cùng khuôn dung thứ của [`preview_import_encoding`].
pub fn preview_bilingual_import(
    shape: &PipelineShape,
    source_lang: &str,
    cleanup_rules: &[CleanupRule],
    chapter_pattern: Option<&ChapterPattern>,
    bilingual_source_column: usize,
    bilingual_target_column: usize,
    bilingual_has_header: bool,
) -> Result<BilingualImportEncodingPreview, IpcError> {
    let PipelineShape::Bilingual { input, .. } = shape else {
        // Chỉ `mod wire` dựng `shape` cho hàm này, luôn từ `import_bilingual_file` — nhánh
        // này là phòng thủ kiểu (một lỗi lập trình, không một đường sản phẩm), không phải
        // một trạng thái người dùng gây ra được.
        return Ok(BilingualImportEncodingPreview {
            confidence: ConfidenceWire::SelfDeclared,
            selected_encoding: encoding_rs::UTF_8.name().to_owned(),
            candidates: Vec::new(),
            sample_rows: Vec::new(),
            row_count: 0,
            column_count: 0,
        });
    };
    let bytes: &[u8] = match input {
        ChapterInput::RawBytes { bytes, .. } => bytes,
        // `import_bilingual_file` luôn dựng `RawBytes` — nhánh này không nên chạm trên
        // đường sản phẩm, cùng lý lẽ nhánh `PipelineShape` không khớp ở trên.
        ChapterInput::AlreadyText(_) => &[],
    };
    let verdict = encoding::detect(bytes);

    let mut selected_sample_rows: Vec<Vec<String>> = Vec::new();
    let mut selected_row_count = 0usize;
    let mut selected_column_count = 0usize;
    let mut selected_seen = false;
    // Lỗi hình dạng bảng của ứng viên ĐANG CHỌN — xem mục "# Lỗi" của doc-comment.
    let mut selected_refusal: Option<ImportError> = None;

    let candidates: Vec<BilingualEncodingCandidateWire> = if bytes.is_empty() {
        Vec::new()
    } else {
        encoding::render_candidates(bytes, source_lang)
            .into_iter()
            .map(|c| {
                let is_selected = c.wire_id == verdict.encoding.name();
                let result = encoding::encoding_for_wire_id(c.wire_id).map(|enc| {
                    run_pipeline(
                        PipelineInput::with_encoding(shape.clone(), enc, source_lang)
                            .with_cleanup_rules(cleanup_rules.to_vec())
                            .with_chapter_pattern(chapter_pattern.cloned())
                            .with_bilingual_columns(
                                bilingual_source_column,
                                bilingual_target_column,
                                bilingual_has_header,
                            ),
                    )
                });
                let outcome = match result {
                    Some(Ok(o)) => Some(o),
                    Some(Err(err)) => {
                        if is_selected && selected_refusal.is_none() && is_bilingual_table_refusal(&err) {
                            selected_refusal = Some(err);
                        }
                        None
                    }
                    None => None,
                };

                let (chapter_count, pair_count, mismatches, row_count, column_count) = match &outcome {
                    Some(o) => {
                        let pair_count: usize = o
                            .chapters
                            .iter()
                            .filter_map(|c| c.bilingual_segments.as_ref())
                            .map(|s| s.len())
                            .sum();
                        let mismatches: Vec<BilingualMismatchWire> =
                            o.bilingual_mismatches.iter().map(BilingualMismatchWire::from).collect();
                        let column_count =
                            o.bilingual_sample_rows.iter().map(Vec::len).max().unwrap_or(0);
                        (o.chapters.len(), pair_count, mismatches, o.bilingual_row_count, column_count)
                    }
                    None => (0, 0, Vec::new(), 0, 0),
                };

                if is_selected && !selected_seen {
                    selected_seen = true;
                    if let Some(o) = &outcome {
                        selected_sample_rows = o.bilingual_sample_rows.clone();
                        selected_row_count = row_count;
                        selected_column_count = column_count;
                    }
                }

                BilingualEncodingCandidateWire {
                    label: c.label.to_owned(),
                    encoding: c.wire_id.to_owned(),
                    preview: c.preview,
                    row_count,
                    chapter_count,
                    pair_count,
                    mismatches,
                }
            })
            .collect()
    };

    if let Some(err) = selected_refusal {
        return Err(err.into());
    }

    Ok(BilingualImportEncodingPreview {
        confidence: verdict.confidence.into(),
        selected_encoding: verdict.encoding.name().to_owned(),
        candidates,
        sample_rows: selected_sample_rows,
        row_count: selected_row_count,
        column_count: selected_column_count,
    })
}

/// Hai lỗi HÌNH DẠNG BẢNG mà màn xem trước song ngữ phải TỪ CHỐI thay vì hiện một dải số 0 —
/// xem mục "# Lỗi" của [`preview_bilingual_import`].
fn is_bilingual_table_refusal(err: &ImportError) -> bool {
    matches!(
        err,
        ImportError::BilingualTooFewColumns { .. } | ImportError::BilingualUnterminatedQuotedField { .. }
    )
}

/// **Hàm thuần** — lõi lượt xác nhận song ngữ: CLONE nguồn đang chờ từ
/// [`PendingImportSourceState`] (tái dùng, xem §Design Notes đầu mục), gọi [`create_work`],
/// dọn ô đang chờ khi và chỉ khi THÀNH CÔNG — cùng khuôn
/// [`confirm_import_with_encoding`].
///
/// # Lỗi
/// - `state` rỗng ⇒ `import.no_pending_source`;
/// - `encoding_wire_id` không giải ngược được ⇒ `import.unrecognized_encoding`;
/// - còn hàng lệch cặp ⇒ `import.bilingual_mismatched_rows` — [`create_work`] tự kiểm lại
///   (Rust-side, §Boundaries), nên đây LUÔN đúng dù webview có quên gọi
///   [`preview_bilingual_import`] lại sau lượt sửa cuối hay không.
pub fn confirm_bilingual_import(
    documents_root: &Path,
    state: &PendingImportSourceState,
    name: &str,
    source_lang: &str,
    genre: &str,
    encoding_wire_id: &str,
    cleanup_rules: Vec<CleanupRule>,
    chapter_pattern: Option<ChapterPattern>,
    bilingual_source_column: usize,
    bilingual_target_column: usize,
    bilingual_has_header: bool,
) -> Result<OpenWork, IpcError> {
    let chosen = encoding::encoding_for_wire_id(encoding_wire_id).ok_or_else(|| {
        IpcError::from(ImportError::UnrecognizedEncoding { wire_id: encoding_wire_id.to_owned() })
    })?;

    let mut guard = state.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
    let shape = guard.as_ref().map(|p| p.shape.clone()).ok_or_else(no_pending_import_source)?;

    let opened = create_work(
        documents_root,
        name,
        source_lang,
        genre,
        shape,
        chosen,
        // 🔵 **SỬA 2026-09-11 (Story 6.16, bước nghiệm thu)** — bản đầu truyền `Vec::new()`
        // ở đây VÀ ở màn xem trước, nên nhánh làm sạch theo ô của `Step::CleanByRules` chạy
        // trên 0 luật: luật người dùng đã bật không bao giờ tới đường song ngữ (§Always spec
        // 6.16: "Cleanup and normalize run per cell, both columns"). Vỏ `wire` phân giải hai
        // tầng LÚC XÁC NHẬN, cùng kỷ luật `confirm_import_with_encoding`. Override khối và
        // override xuất xứ vẫn rỗng: hai bề mặt đó không có trên màn xem trước song ngữ.
        cleanup_rules,
        chapter_pattern,
        Vec::new(),
        bilingual_source_column,
        bilingual_target_column,
        bilingual_has_header,
        &[],
        &std::sync::Mutex::new(Vec::new()),
        None,
    )?;

    *guard = None;
    Ok(opened)
}

// ═════════════════════════════════════════════════════════════════════════════════
// Story 6.7 — Nhập từ URL bằng danh sách link (AD-15 · AD-40 · AD-41 · FR122)
// ═════════════════════════════════════════════════════════════════════════════════
//
// ─────────────────────────────────────────────────────────────────────────────
// 🔴 KHÔNG MỘT ĐƯỜNG `create_work` THỨ HAI — TÁI DÙNG TOÀN BỘ MÁY XEM TRƯỚC BẢNG MÃ CŨ
// ─────────────────────────────────────────────────────────────────────────────
// N link đã tải thành công dựng ĐÚNG một [`PipelineShape::Chapters`] — hình dạng
// `preview_import_encoding`/`confirm_import_with_encoding` (Story 6.3) đã biết xử từ
// `chapters.first()` (xem `PipelineShape::Chapters` trong `preview_import_encoding` —
// "danh sách URL là Story 6.7", nhánh đó viết sẵn từ Story 6.3). [`start_url_import`] vì
// thế KHÔNG gọi [`create_work`] trực tiếp: nó `stash_pending_import_source` đúng như hai
// nhánh dán-văn-bản/tệp, và màn xác nhận CUỐI CÙNG đi qua [`confirm_import_with_encoding`]
// KHÔNG ĐỔI MỘT DÒNG — dải năm ứng viên bảng mã, khối làm sạch (tầng 3), khối tách Chương
// (tầng 4, ở đây luôn hiện ĐÚNG N Chương vì `already_chaptered = true` khiến bước 5 bỏ qua)
// đều MIỄN PHÍ. Điều kiện DUY NHẤT: `PendingImportSourceState` phải được ĐỒNG BỘ với danh
// sách mục hiện tại SAU MỌI thao tác (tải, tải lại một mục, bỏ một mục) — xem
// [`sync_pending_from_url_items`].

/// Một MỤC trong danh sách URL đang chờ — byte thô THÀNH CÔNG, HOẶC một [`IpcError`] mang
/// đúng MỘT trong tám lý do của [`WebImportItemFailureReason`]. Đúng MỘT trong hai, không
/// cả hai — không kiểu Rust nào ép được bất biến "đúng một trong hai trường" ở ĐÂY mà không
/// một `enum` (giữ `struct` phẳng để `UrlImportItemWire::from` không phải `match`), nên
/// mọi hàm DỰNG giá trị này (không phải `derive`) đều đặt ĐÚNG MỘT trong `raw`/`error`.
#[derive(Debug, Clone)]
pub struct UrlImportItem {
    /// URL đã dán — TRIM, không rỗng (dòng rỗng bị lọc TRƯỚC khi tới đây).
    pub url: String,
    /// Byte HTML thô — `Some` khi tải THÀNH CÔNG và `content-type` là HTML.
    pub raw: Option<Vec<u8>>,
    /// Lý do thất bại — `Some` khi `raw` là `None`.
    pub error: Option<IpcError>,
}

/// Trạng thái từng mục, sống CẠNH [`PendingImportSourceState`] trong bộ nhớ (§Task list spec
/// 6.7) — `None` == chưa có lượt dán URL nào đang treo, cùng khuôn `PendingImportSourceState`.
pub type UrlImportItemsState = std::sync::Mutex<Option<Vec<UrlImportItem>>>;

/// **THÊM 2026-09-07 (Story 6.9)** — trạng thái giữ/loại người dùng đã ĐẶT BẰNG TAY cho khối
/// của Chương đầu tiên, sống CẠNH [`UrlImportItemsState`] trong bộ nhớ. `overrides[i] ==
/// Some(v)` ⇒ khối `i` giữ (`v`); `None`/ngoài phạm vi ⇒ dùng `machine_kept` — cùng ngữ nghĩa
/// [`crate::core::segment::pipeline::PipelineInput::block_overrides`], mà trường này CHÍNH LÀ
/// nguồn cấp giá trị.
///
/// 🔴 **Rỗng (Vec trần, KHÔNG một `Option` bọc ngoài) là trạng thái "chưa ai sửa gì" — khác
/// [`PendingImportSourceState`]/[`UrlImportItemsState`] (nơi `None` phân biệt "chưa mở lượt
/// nào" khỏi "đã mở, danh sách rỗng").** Tầng 2 không cần phân biệt đó: một vector rỗng LUÔN
/// nghĩa là "không có override nào", dù đó là vì chưa mở lượt xem trước, hay vì đã mở nhưng
/// người dùng chưa bấm `Space`/`[`/`]` lần nào — hai ca đó xử lý GIỐNG HỆT nhau (dùng nguyên
/// `machine_kept`), nên một `Option` bọc ngoài không thêm được thông tin nào.
///
/// **Reset khi nào (cả bốn ở lớp vỏ `wire`, KHÔNG ở các hàm thuần — reset là một quyết định
/// VÒNG ĐỜI của lượt xem trước, không phải của phép tính):**
/// `preview_import_encoding_from_text`/`_from_file` (mở một lượt MỚI — text/file không bao
/// giờ đọc tầng 2, nhưng dọn sạch cho nhất quán vòng đời) và `start_url_import` (danh sách
/// HOÀN TOÀN MỚI — cấu trúc khối của Chương đầu chắc chắn khác, một override cũ áp nhầm chỉ
/// số sẽ SAI Ý NGHĨA, không chỉ lỗi thời) LUÔN reset. `reload_url_import_item`/
/// `remove_url_import_item` CHỈ reset khi `index == 0` (mục 0 chính là Chương tầng 2 đang
/// hiển thị) — sửa/bỏ một mục KHÁC không đụng tới cấu trúc khối của mục 0, giữ nguyên override
/// là đúng, không phải một giản lược.
pub type Tier2BlockOverridesState = std::sync::Mutex<Vec<Option<bool>>>;

/// Dọn sạch [`Tier2BlockOverridesState`] — cùng khuôn
/// [`clear_url_import_items_after_successful_confirm`]. **Hàm thuần, `pub`** để
/// `tests/project_contract.rs` gọi được không cần `tauri::AppHandle`.
pub fn reset_block_overrides(state: &Tier2BlockOverridesState) {
    let mut guard = state.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
    guard.clear();
}

/// **THÊM 2026-09-10 (Story 6.15)** — bốn trường xuất xứ người dùng đã gõ đè cho MỘT Chương ở
/// màn xem trước, mỗi trường `Some(v)` ⇔ người dùng đã CHẠM ô đó (`v` rỗng sau khi cắt ⇒ họ đã
/// xoá trắng, cột ghi `NULL`); `None` ⇔ chưa ai chạm, dùng nguyên giá trị máy bóc
/// ([`crate::core::webimport::ChapterOrigin`]). Khác [`Tier2BlockOverridesState`] (chỉ đơn vị
/// 0) — xuất xứ áp được cho MỌI Chương của lượt nhập URL, nên state bên dưới đánh chỉ số theo
/// CHƯƠNG, không theo khối.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ChapterOriginOverride {
    pub author: Option<String>,
    pub site_name: Option<String>,
    pub url: Option<String>,
    pub published_at: Option<String>,
}

/// `field(value)` — cắt hai đầu, rỗng sau khi cắt ⇒ `None` (ô đã bị xoá trắng, cột ghi
/// `NULL`) — khuôn dùng chung cho cả bốn trường của [`ChapterOriginOverride`].
fn trimmed_or_none(value: &str) -> Option<String> {
    let t = value.trim();
    if t.is_empty() { None } else { Some(t.to_owned()) }
}

/// Áp một [`ChapterOriginOverride`] (nếu có) lên một
/// [`crate::core::webimport::ChapterOrigin`] máy đã bóc (nếu có) — trả bốn `Option<String>`
/// SẴN SÀNG bind vào câu `INSERT`. `override_field == Some(v)` LUÔN thắng (kể cả khi `v` rỗng
/// ⇒ `NULL`, người dùng đã xoá trắng); `None` ⇒ dùng nguyên giá trị máy (hoặc `None` nếu máy
/// cũng chưa từng chạy — đường tệp/dán tay).
fn effective_origin_fields(
    machine: Option<&crate::core::webimport::ChapterOrigin>,
    over: Option<&ChapterOriginOverride>,
) -> (Option<String>, Option<String>, Option<String>, Option<String>) {
    // 🔴 SỬA (đo được, bàn đo `chapter_origin_contract.rs::a_hand_typed_override_at_preview_time_wins_over_the_machine_extracted_value`)
    // — bản đầu bọc `over.map(|o| &o.<field>)` dựng một `Option` LỒNG SAI: lớp NGOÀI chỉ nói
    // "có một BẢN GHI override cho Chương này", không nói "TRƯỜNG NÀY đã bị chạm" — nên MỘT
    // trường bị chạm (`author`) kéo theo BA trường còn lại (`site_name`/`url`/`published_at`,
    // đang `None` = "chưa chạm") bị đọc NHẦM thành "đã chạm, giá trị rỗng" ⇒ mất giá trị MÁY
    // của chúng. `over_field` ở đây PHẢI là `Option<String>` ĐÃ LÀM PHẲNG (một lớp DUY NHẤT):
    // `None` ⇔ chưa chạm (dù `over` có mặt hay không) — chỉ khi đó mới rơi về giá trị máy.
    let pick = |over_field: Option<String>, machine_field: Option<&String>| -> Option<String> {
        match over_field {
            Some(v) => trimmed_or_none(&v),
            None => machine_field.cloned(),
        }
    };
    (
        pick(over.and_then(|o| o.author.clone()), machine.and_then(|m| m.author.as_ref())),
        pick(over.and_then(|o| o.site_name.clone()), machine.and_then(|m| m.site_name.as_ref())),
        pick(over.and_then(|o| o.url.clone()), machine.and_then(|m| m.url.as_ref())),
        pick(
            over.and_then(|o| o.published_at.clone()),
            machine.and_then(|m| m.published_at.as_ref()),
        ),
    )
}

/// **THÊM 2026-09-10 (Story 6.15)** — cùng khuôn [`Tier2BlockOverridesState`], nhưng đánh chỉ
/// số theo CHƯƠNG (vị trí trong `chapters: &[ImportedChapter]` của lượt nhập đang xem trước),
/// không theo khối — xuất xứ là quyết định per-Chương (§Tasks spec 6.15), không giới hạn ở
/// Chương đầu như tầng 2 khối. Rỗng == "chưa ai sửa gì", cùng lý lẽ `Tier2BlockOverridesState`.
///
/// **Reset khi nào — SÁU điểm, tất cả ở lớp vỏ `wire`:**
/// `preview_import_encoding_from_text`/`_from_file` (mở một lượt XEM TRƯỚC MỚI — đường tệp/
/// dán tay không bao giờ đọc override, nhưng dọn NGAY vẫn bắt buộc: không dòng này, một
/// override còn treo từ một lượt nhập URL đã HUỶ [huỷ không tự dọn state Rust — xem
/// `cancel_import_preview`] sống sót và bị `confirm_import_with_encoding` đọc nhầm vào Chương
/// của lượt dán tay/tệp MỚI, đúng lớp lỗi "rỗng/hỏng ngầm" mà AGENTS.md gọi tên là trung tâm —
/// bắt ở lượt rà 2026-09-10); `start_url_import` (danh sách HOÀN TOÀN MỚI);
/// `reload_url_import_item`/`remove_url_import_item` (chỉ số Chương có thể đã dời — dọn TOÀN
/// BỘ, xem §Spec Change Log spec 6.15 mục 2); và ngay SAU KHI [`confirm_import_with_encoding`]
/// trả `Ok` (đã ghi xong, giữ lại là rác cho lượt kế) — KHÔNG TRƯỚC (một lượt xác nhận trượt
/// giữ nguyên override để thử lại, cùng kỷ luật `PendingImportSourceState`).
pub type ChapterOriginOverridesState = std::sync::Mutex<Vec<Option<ChapterOriginOverride>>>;

/// Dọn sạch [`ChapterOriginOverridesState`] — cùng khuôn [`reset_block_overrides`]. **Hàm
/// thuần, `pub`** để `tests/**` gọi được không cần `tauri::AppHandle`.
pub fn reset_chapter_origin_overrides(state: &ChapterOriginOverridesState) {
    let mut guard = state.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
    guard.clear();
}

/// Ghi một [`ChapterOriginOverride`] vào chỉ số `chapter_index` của
/// [`ChapterOriginOverridesState`], mở rộng vector (đệm `None`) nếu cần — **hàm thuần**, đây
/// là thứ `tests/**` gọi; vỏ IPC ở `mod wire`.
pub fn set_chapter_origin_override(
    state: &ChapterOriginOverridesState,
    chapter_index: usize,
    over: ChapterOriginOverride,
) {
    let mut guard = state.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
    if guard.len() <= chapter_index {
        guard.resize(chapter_index + 1, None);
    }
    guard[chapter_index] = Some(over);
}

/// Vị từ THUẦN — mục vừa sửa/tải lại/bỏ ở `index` có làm cấu trúc khối của Chương tầng 2
/// (LUÔN là mục 0 của danh sách URL, xem doc-comment `Tier2BlockOverridesState` mục "Reset
/// khi nào") SAI Ý NGHĨA hay không. **THÊM 2026-09-07 (vòng rà bước 4, mục 14)** — tách khỏi
/// hai chỗ gọi `if index == 0 { reset_tier2_block_overrides(&app); }` bên trong
/// `wire::reload_url_import_item`/`wire::remove_url_import_item` để `tests/**` gọi được quy
/// tắc này KHÔNG cần dựng một `tauri::AppHandle` (`src-tauri/AGENTS.md:11`: quy tắc sống ở
/// hàm thuần, vỏ chỉ chuyển tiếp).
pub fn mutated_index_invalidates_tier2_blocks(index: usize) -> bool {
    index == 0
}

/// P6 (vòng rà đối kháng bước 4) — dọn [`UrlImportItemsState`] khi và CHỈ KHI `create_work`
/// vừa THÀNH CÔNG. Trước bản vá này, `wire::confirm_import_with_encoding` chỉ dọn
/// [`PendingImportSourceState`] (`*guard = None` trong [`confirm_import_with_encoding`] ở
/// trên) — `UrlImportItemsState` không hề bị chạm ở bất kỳ đâu ngoài ba lệnh dây
/// `start_url_import`/`reload_url_import_item`/`remove_url_import_item`. Hệ quả: byte HTML
/// thô của N link nằm lại trong bộ nhớ sau khi Tác phẩm đã tạo xong, và một danh sách CŨ vẫn
/// còn đó nếu người dùng mở lại màn nhập URL. **Hàm thuần, `pub`** — không có `tauri::test`/
/// `MockRuntime` trong crate này (`Cargo.toml` không khai `test-utils`), nên đây là cách DUY
/// NHẤT để `tests/project_contract.rs` phủ được đường dọn này mà không cần dựng một
/// `tauri::AppHandle` thật.
pub fn clear_url_import_items_after_successful_confirm(items_state: &UrlImportItemsState) {
    let mut guard = items_state.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
    *guard = None;
}

/// Hình dạng DÂY của [`UrlImportItem`] — vị trí là INDEX trong `Vec` (frontend giữ nguyên
/// thứ tự, không sắp lại), `error` mang `IpcError` ĐẦY ĐỦ (khoá i18n + tham số) để frontend
/// dịch bằng `tError()`, cùng khuôn mọi lỗi IPC khác của dự án.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct UrlImportItemWire {
    pub url: String,
    pub ok: bool,
    pub error: Option<IpcError>,
}

impl From<&UrlImportItem> for UrlImportItemWire {
    fn from(item: &UrlImportItem) -> Self {
        UrlImportItemWire { url: item.url.clone(), ok: item.error.is_none(), error: item.error.clone() }
    }
}

/// Kết quả trả về của cả ba lệnh (tải danh sách, tải lại một mục, bỏ một mục) — danh sách
/// mục HIỆN TẠI cộng xem trước bảng mã khi TOÀN BỘ đã OK. `encoding_preview: None` là điều
/// kiện đủ để frontend biết nút xác nhận phải khoá (đồng bộ với
/// [`sync_pending_from_url_items`] — cùng điều kiện, không suy luận riêng ở tầng hiển thị).
#[derive(Debug, Clone, serde::Serialize)]
pub struct UrlImportBatchWire {
    pub items: Vec<UrlImportItemWire>,
    pub encoding_preview: Option<ImportEncodingPreview>,
    /// 🔵 **THÊM Story 6.8** — số domain PHÂN BIỆT trong nhật ký của CẢ PHIÊN CHẠY (không chỉ
    /// lượt gọi vừa rồi) tại thời điểm trả lời — chân màn xem trước đọc trực tiếp trường này
    /// (mockup *"Đã gọi **N** domain · xem"*), không một lệnh IPC thứ hai chỉ để có một số.
    /// `wire::start_url_import`/`reload_url_import_item`/`remove_url_import_item` cùng tính
    /// qua `wire::domain_log_domain_count` — ba vỏ, một nguồn.
    pub domain_log_domain_count: usize,
}

/// Hình dạng DÂY của một [`webimport::DomainLogEntry`] — Story 6.8, NFR19. `kind`/`decision`
/// đi qua như DỮ LIỆU (chuỗi định danh máy, AD-21), KHÔNG một câu — cùng khuôn
/// `CleanupRuleTierWire` (`Global`/`Work`, `cleanupTierLabelKey` phía TS ánh xạ sang câu).
/// Frontend dựng câu "vì sao được phép/từ chối" bằng một hàm THUẦN có nhánh mặc định, không
/// nhận nguyên văn từ Rust.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct DomainLogEntryWire {
    pub at_epoch_ms: u64,
    pub domain: String,
    /// `"page"` | `"image"` — khớp [`webimport::ResourceKind`] (biến thể `Page`/`Image`),
    /// `serde(rename_all = "snake_case")` trên chính kiểu đó (định nghĩa ở
    /// `core::webimport::allowlist`).
    ///
    /// 🔵 SỬA 2026-09-09 (vòng rà đối kháng 2, mục D8) — câu cũ khai `"document"`, một chuỗi
    /// KHÔNG BAO GIỜ thật sự đi qua dây (lỗi từ Story 6.8): biến thể Rust là `Page`, và
    /// `snake_case` của `Page` là `"page"`, không phải `"document"`.
    pub kind: webimport::ResourceKind,
    pub allowed: bool,
    /// `"tier1"` | `"tier2"` | `"denied"` — tầng đã CẤP PHÉP khi `allowed == true`; luôn
    /// `"denied"` khi `allowed == false`. Một trường DUY NHẤT (không `Option<Tier>` +
    /// `bool` rời) để phía TS không phải tự đối chiếu hai trường có nhất quán không.
    pub tier: &'static str,
    /// **THÊM 2026-09-08 (Story 6.11)** — `null` khi `allowed == false` (0 kết nối, không có
    /// kết quả mạng nào để mà báo); một trong tám chuỗi `snake_case` của
    /// [`webimport::DomainLogOutcome`] khi `allowed == true`. Đây là trường trả lời "và rồi
    /// SAO" cho một chặng đã cho phép — thứ hai ô `Error Handling` của I/O Matrix spec 6.11
    /// đòi mà `tier`/`allowed` một mình không nói được.
    pub outcome: Option<webimport::DomainLogOutcome>,
}

impl From<&webimport::DomainLogEntry> for DomainLogEntryWire {
    fn from(entry: &webimport::DomainLogEntry) -> Self {
        let (allowed, tier) = match entry.decision {
            webimport::DomainLogDecision::Allowed(webimport::Tier::One) => (true, "tier1"),
            webimport::DomainLogDecision::Allowed(webimport::Tier::Two) => (true, "tier2"),
            webimport::DomainLogDecision::Denied => (false, "denied"),
        };
        DomainLogEntryWire {
            at_epoch_ms: entry.at_epoch_ms,
            domain: entry.domain.clone(),
            kind: entry.kind,
            allowed,
            tier,
            outcome: entry.outcome,
        }
    }
}

/// Tải MỘT URL và phân loại kết quả thành [`UrlImportItem`] — **0 dòng phân tích nội dung
/// ngoài việc đọc header `content-type`** (kiểm giao thức, không phải nội dung; xem
/// doc-comment [`webimport::looks_like_html`]). Trả kèm nhật ký domain phát sinh từ CHÍNH
/// lượt gọi này (§Always spec 6.8: "một bản ghi cho mọi lời gọi") — chỗ gọi nối vào
/// [`webimport::DomainLogState`] của phiên chạy; hàm này (và [`webimport::fetch`] bên dưới
/// nó) không tự ghi vào state Tauri nào — cả hai đều là hàm THUẦN.
///
/// 🔴 **`ResourceKind::Page` LUÔN LUÔN** — đây là chỗ gọi sản phẩm DUY NHẤT của
/// [`webimport::fetch`] hôm nay, và nó tải đúng những gì người dùng đã dán (một bài viết),
/// không bao giờ một ảnh. `ResourceKind::Image` chỉ có mặt trong kiểu để bốn mệnh đề bắt
/// buộc của AD-41 phát biểu được (xem doc-comment đầu `core::webimport::allowlist`) — Story
/// 6.11 sẽ là chỗ gọi sản phẩm ĐẦU TIÊN của nó.
fn fetch_url_import_item(url: &str, allowlist: &webimport::Allowlist) -> (UrlImportItem, Vec<webimport::DomainLogEntry>) {
    let (result, log) = webimport::fetch(url, allowlist, webimport::ResourceKind::Page);
    let item = match result {
        Ok(page) => {
            if webimport::looks_like_html(page.content_type.as_deref()) {
                UrlImportItem { url: url.to_owned(), raw: Some(page.bytes), error: None }
            } else {
                UrlImportItem {
                    url: url.to_owned(),
                    raw: None,
                    error: Some(web_import_item_failure_ipc_error(url, WebImportItemFailureReason::NotHtml, None)),
                }
            }
        }
        Err(e) => {
            let http_status = match &e {
                webimport::FetchError::HttpStatus { status } => Some(*status),
                _ => None,
            };
            let reason = WebImportItemFailureReason::from(e);
            UrlImportItem {
                url: url.to_owned(),
                raw: None,
                error: Some(web_import_item_failure_ipc_error(url, reason, http_status)),
            }
        }
    };
    (item, log)
}

/// Bước ĐẦU VÀO — TRIM mỗi dòng, BỎ dòng rỗng khi đếm (I/O Matrix spec 6.7: "Dòng rỗng bỏ
/// khi đếm; dòng rác thành mục hỏng"), rồi tải TUẦN TỰ ĐÚNG THỨ TỰ đã dán. **Hàm thuần** —
/// không `tauri::`, `tests/**` gọi được trực tiếp.
/// Cắt hai đầu một dòng dán vào **đúng như `String.prototype.trim()` của JS làm** — thứ
/// `src/modes/libraryImport.ts::pastedUrlLines` dùng để đếm hai con số *N link · N Chương*.
///
/// 🔴 **ĐO 2026-09-07 (vòng rà bước 4) — `str::trim()` của Rust KHÔNG bằng `trim()` của JS,
/// và chỗ lệch nằm đúng trên một ký tự người dùng hay dán phải.** Đo trực tiếp cả hai bên trên
/// bốn ký tự: `U+00A0`, `U+2028`, `U+200B` cho kết quả GIỐNG nhau, nhưng **`U+FEFF`** (BOM /
/// zero-width no-break space) thì JS coi là khoảng trắng còn Rust **không** (`char::is_whitespace`
/// theo thuộc tính Unicode `White_Space`, và `U+FEFF` không có thuộc tính đó).
///
/// Hai ca hỏng THẬT mà chỗ lệch này sinh ra, cả hai đều đánh vào bất biến trung tâm của story:
/// ① một dòng CHỈ có `U+FEFF` — JS cắt thành rỗng nên KHÔNG đếm, Rust giữ nên sinh THÊM một
/// mục ⇒ *N link* trên màn hình khác số Chương sắp tạo, đúng thứ AC4 dựng một test để bắt.
/// ② một URL hợp lệ mang BOM ở ĐẦU (dán từ Windows/Excel/trang web là ca thường gặp) — JS cắt
/// nên màn hình đếm nó là link hợp lệ, Rust giữ nên `Url::parse` trượt ⇒ mục hỏng oan, nút xác
/// nhận khoá, và người dùng không có cách nào nhìn ra một ký tự vô hình.
///
/// ⇒ Cắt thêm `U+FEFF`. Không mở rộng gì khác: ba ký tự còn lại đã khớp, và một phép cắt RỘNG
/// HƠN JS lại tạo ra chỗ lệch theo chiều ngược lại.
fn trim_like_the_paste_box(line: &str) -> &str {
    line.trim_matches(|c: char| c.is_whitespace() || c == '\u{FEFF}')
}

/// 🔵 **Story 6.8** — chữ ký đổi: trả kèm nhật ký domain của CẢ lượt (`log`), và tự dựng
/// [`webimport::Allowlist`] từ CHÍNH `urls` đã TRIM/lọc rỗng — allowlist một-lần-nhập đúng
/// theo CẤU TẠO (§Design Notes spec 6.8): không có `Allowlist` nào tồn tại NGOÀI thân hàm
/// này, nên không có gì để mà rò rỉ sang lượt nhập kế tiếp.
pub fn fetch_url_import_items(urls: Vec<String>) -> (Vec<UrlImportItem>, Vec<webimport::DomainLogEntry>) {
    let trimmed: Vec<String> =
        urls.into_iter().map(|s| trim_like_the_paste_box(&s).to_owned()).filter(|s| !s.is_empty()).collect();
    let allowlist = webimport::Allowlist::from_urls(trimmed.iter().map(String::as_str));

    let mut log = Vec::new();
    let items = trimmed
        .into_iter()
        .map(|u| {
            let (item, entries) = fetch_url_import_item(&u, &allowlist);
            log.extend(entries);
            item
        })
        .collect();
    (items, log)
}

/// Allowlist cho một lượt TẢI LẠI một mục — dựng từ URL của **TOÀN BỘ** danh sách hiện tại
/// (`items`), không chỉ URL của mục đang tải lại: allowlist là một-lần-NHẬP, và tải lại vẫn
/// thuộc CÙNG lần nhập với lượt [`fetch_url_import_items`] ban đầu (`UrlImportItemsState`
/// giữ nguyên danh sách đó giữa các lượt gọi — xem doc-comment [`UrlImportItemsState`]).
fn allowlist_from_items(items: &[UrlImportItem]) -> webimport::Allowlist {
    webimport::Allowlist::from_urls(items.iter().map(|it| it.url.as_str()))
}

/// Dựng [`PipelineShape::Chapters`] từ `items` khi và CHỈ KHI danh sách KHÔNG rỗng, KHÔNG mục
/// nào mang lỗi, VÀ mọi mục không-lỗi thật sự mang `raw` — `None` khi còn một mục hỏng (đúng
/// lúc nút xác nhận phải KHOÁ, §Always spec 6.7), danh sách rỗng (đóng vế còn mở của nợ
/// `:9067`), hoặc một mục vỡ bất biến "đúng một trong `raw`/`error` là `Some`" (không kiểu nào
/// cưỡng chế bất biến đó — 🔴 vỡ thì hàm này KHÔNG được lặng lẽ dựng một Chương RỖNG từ
/// `unwrap_or_default()`, đó chính là lớp lỗi "rỗng im lặng" mà `AGENTS.md` gọi là trung tâm
/// của dự án; trả `None` giống hệt đường "còn mục hỏng" là lựa chọn AN TOÀN duy nhất). **Hàm
/// thuần, `pub`** để `tests/project_contract.rs` gọi trực tiếp — chứng minh "còn MỘT mục hỏng
/// ⇒ 0 hàng ghi xuống" mà không cần dựng một `tauri::AppHandle`.
pub fn chapters_shape_if_all_ok(items: &[UrlImportItem]) -> Option<PipelineShape> {
    if items.is_empty() || items.iter().any(|it| it.error.is_some()) {
        return None;
    }
    let mut chapters = Vec::with_capacity(items.len());
    for it in items {
        let bytes = it.raw.clone()?;
        chapters.push(ChapterInput::RawBytes { bytes, label: it.url.clone() });
    }
    Some(PipelineShape::Chapters(chapters))
}

/// **THÊM Story 6.10a** — vị từ **XEM**, tách khỏi [`chapters_shape_if_all_ok`] (vị từ **GHI**,
/// GIỮ NGUYÊN Ở TRÊN, không đụng — đường `create_work` vẫn khoá cho tới khi MỌI mục sạch).
///
/// ─────────────────────────────────────────────────────────────────────────────
/// 🔴 VÌ SAO CẦN MỘT HÀM THỨ HAI — "XEM ĐƯỢC" VÀ "GHI ĐƯỢC" LÀ HAI MỆNH ĐỀ KHÁC NHAU
/// ─────────────────────────────────────────────────────────────────────────────
/// Trước story này, `url_import_encoding_preview` dùng CHÍNH [`chapters_shape_if_all_ok`] —
/// nên "một mục hỏng" đồng thời làm CẢ nút xác nhận khoá LẪN màn xem trước biến mất hoàn
/// toàn, cho 4 Chương ĐÃ TẢI được rất tốt. Hàm này dựng [`PipelineShape::Chapters`] từ CÁC
/// MỤC OK — bỏ QUA mục hỏng, không đợi TOÀN BỘ danh sách sạch mới có gì đó để mà xem — và
/// `items` KHÔNG hề bị đụng vào (mục hỏng vẫn đứng NGUYÊN vị trí của nó trong danh sách,
/// hàm này chỉ ĐỌC để lọc, không sắp lại/xoá gì ở `items`).
///
/// `None` khi KHÔNG mục OK nào (mọi mục đều hỏng — I/O Matrix: "rỗng CÓ LÝ DO", không có gì
/// để mà xem, khác hẳn "sẵn sàng ghi").
///
/// 🔴 Kết quả hàm này KHÔNG BAO GIỜ được dùng để quyết định nút xác nhận khoá hay không — chỉ
/// [`chapters_shape_if_all_ok`] mới có quyền đó (xem chỗ gọi ở `sync_pending_from_url_items`,
/// KHÔNG đổi). Phía frontend, tín hiệu khoá nút đọc thẳng `UrlImportItemWire::ok` của TỪNG
/// mục (`items.every(ok)`), KHÔNG còn đọc `encoding_preview === null` — trộn hai tín hiệu đó
/// lại đúng là lỗi mà §Always story 6.10a cấm.
pub fn chapters_shape_for_view(items: &[UrlImportItem]) -> Option<PipelineShape> {
    let chapters: Vec<ChapterInput> = items
        .iter()
        .filter_map(|it| it.raw.clone().map(|bytes| ChapterInput::RawBytes { bytes, label: it.url.clone() }))
        .collect();
    if chapters.is_empty() { None } else { Some(PipelineShape::Chapters(chapters)) }
}

/// Đồng bộ [`PendingImportSourceState`] với `items` HIỆN TẠI — gọi lại sau MỌI thao tác đổi
/// danh sách (tải lần đầu, tải lại một mục, bỏ một mục). Còn mục hỏng hoặc danh sách rỗng ⇒
/// DỌN ô đang chờ (một lượt `confirm_import_with_encoding` kế tiếp trả `no_pending_source`
/// — nút xác nhận khoá THẬT, không chỉ khoá ở tầng hiển thị). Toàn bộ mục OK ⇒ GHI ĐÈ ô đang
/// chờ bằng [`PipelineShape::Chapters`] mới dựng từ CHÍNH danh sách này.
fn sync_pending_from_url_items(pending: &PendingImportSourceState, items: &[UrlImportItem]) {
    match chapters_shape_if_all_ok(items) {
        // Đường URL không bao giờ mang một `DocxSidecar` (đó là đường tệp `.docx`, Story
        // 6.12) — `None` cố định.
        Some(shape) => stash_pending_import_source(pending, shape, None),
        None => cancel_import_preview(pending),
    }
}

/// Xem trước bảng mã cho danh sách URL — `None` khi KHÔNG mục OK nào/danh sách rỗng (I/O
/// Matrix: "rỗng có lý do"). Chốt từ đơn vị ĐẦU — [`PipelineInput::encoding`] doc-comment "một
/// bảng mã cho cả danh sách".
///
/// 🔵 **SỬA 2026-09-08 (Story 6.10a) — đọc [`chapters_shape_for_view`] (vị từ XEM), KHÔNG còn
/// [`chapters_shape_if_all_ok`] (vị từ GHI).** Trước bản sửa này, một mục hỏng làm CẢ hàm này
/// trả `None` — bốn Chương ĐÃ TẢI tốt biến mất khỏi màn xem trước chỉ vì một mục THỨ NĂM
/// hỏng. Nút xác nhận khoá KHÔNG còn phụ thuộc kết quả hàm này (xem doc-comment
/// [`chapters_shape_for_view`]) — `sync_pending_from_url_items` (đường GHI) vẫn gọi
/// [`chapters_shape_if_all_ok`] y nguyên.
/// 🔴 **THÊM 2026-09-08 (Story 6.10)** — đếm số mục `error.is_some()` của CẢ `items` rồi
/// truyền vào [`preview_import_encoding`] — đây là chỗ DUY NHẤT tính `broken_item_count` cho
/// đường URL, đọc thẳng từ `items` (không từ danh sách Chương — link hỏng không bao giờ vào
/// đó, §Always spec 6.10).
fn url_import_encoding_preview(
    items: &[UrlImportItem],
    source_lang: &str,
    cleanup_rules: &[CleanupRule],
    block_overrides: &[Option<bool>],
    origin_overrides: &[Option<ChapterOriginOverride>],
) -> Option<ImportEncodingPreview> {
    let shape = chapters_shape_for_view(items)?;
    let broken_item_count = items.iter().filter(|it| it.error.is_some()).count();
    Some(preview_import_encoding(
        &shape,
        source_lang,
        cleanup_rules,
        None,
        block_overrides,
        broken_item_count,
        origin_overrides,
    ))
}

/// Dựng [`UrlImportBatchWire`] từ trạng thái HIỆN TẠI — dùng chung bởi cả ba lệnh
/// (tải/tải lại/bỏ một mục) VÀ hai lệnh MỚI Story 6.9 (đặt/gỡ override một khối, đặt dải) để
/// không có năm lượt lắp dây khác nhau cho CÙNG một hình dạng. `domain_log_domain_count` đi
/// vào từ THAM SỐ (đọc từ [`webimport::DomainLogState`] ở lớp vỏ) — hàm này ở lại **thuần**,
/// không tự cầm `AppHandle`/`State` nào.
///
/// 🔴 **THÊM 2026-09-07 (Story 6.9) — tham số `block_overrides`.** Đường URL là đường DUY
/// NHẤT `extract_main_content == true` đi qua được (§Always spec 6.7/6.9) — tầng 2 CHỈ có
/// nghĩa ở đây, nên đây CŨNG là chỗ DUY NHẤT override cần chảy vào lượt xem trước.
fn url_import_batch_wire(
    items: &[UrlImportItem],
    source_lang: &str,
    cleanup_rules: &[CleanupRule],
    block_overrides: &[Option<bool>],
    domain_log_domain_count: usize,
    // 🔴 THÊM 2026-09-10 (Story 6.15) — cùng lý do `block_overrides` ngay trên: đường URL là
    // đường DUY NHẤT `extract_main_content == true`, nên đây CŨNG là chỗ DUY NHẤT xuất xứ có
    // gì để mà hiện.
    origin_overrides: &[Option<ChapterOriginOverride>],
) -> UrlImportBatchWire {
    UrlImportBatchWire {
        items: items.iter().map(UrlImportItemWire::from).collect(),
        encoding_preview: url_import_encoding_preview(
            items,
            source_lang,
            cleanup_rules,
            block_overrides,
            origin_overrides,
        ),
        domain_log_domain_count,
    }
}

/// Lỗi dự phòng cho một nhánh KHÔNG NÊN xảy ra trên đường sản phẩm — state Tauri chưa được
/// `.manage(...)` (lỗi lắp dây ở `lib.rs`), hoặc một `index` ngoài phạm vi danh sách hiện tại
/// (frontend luôn gửi một index đọc từ CHÍNH mảng nó đang hiện). Cùng khuôn
/// [`ImportError::InvalidPipelineOrder`]/`InvalidCleanupPattern` — [`MessageKey::Unknown`],
/// không tự đúc một khoá mới cho một nhánh không chỗ gọi SẢN PHẨM nào đi qua.
fn url_import_internal_error() -> IpcError {
    IpcError::new(
        "import.web_internal_error",
        crate::core::i18n::MessageKey::Unknown,
        std::collections::BTreeMap::new(),
        false,
    )
}

/// `work_id` không có hàng trong `library-index.db` (`Indexer::find_work` trả `None`) —
/// hàm dựng lỗi **tách riêng** để [`open_work`] và `wire::open_work` dùng chung MỘT nguồn
/// sự thật cho câu này (đúng khuôn `no_work_open`/`chapter_not_found` của
/// `commands::chapter`).
fn work_not_indexed(work_id: &str) -> IpcError {
    IpcError::new(
        "library.work_not_indexed",
        crate::core::i18n::MessageKey::LibraryWorkNotIndexed,
        std::collections::BTreeMap::from([("work_id".to_owned(), work_id.to_owned())]),
        false,
    )
}

/// **Hàm thuần** — mở lại một `.atproj` **đã có trên đĩa** (Story 5.7, FR12). Khuôn thứ
/// tự chép NGUYÊN VĂN của [`create_work`], chỉ thay bước *tạo* bằng bước *đọc*: `WorkMeta::
/// read` → `Store::open` → chọn `chapter_id` → `ScopeResolver::with_work`.
///
/// 🔴 **`indexed: Option<&IndexedWork>`, không `&IndexedWork` trần** — đúng khuôn hai lớp
/// của `src-tauri/AGENTS.md` ("① một hàm thuần nhận `Option<&Store>`... đây là thứ
/// `tests/**` gọi được không cần webview"): quyết định *"`work_id` lạ ⇒
/// `library.work_not_indexed`"* là một QUY TẮC, và `mod wire` bên dưới **không một quy tắc
/// nào sống ở đó** — nó chỉ gọi `Indexer::find_work` rồi chuyển tiếp `Option` xuống đây
/// nguyên vẹn, để `tests/project_contract.rs` gọi được ca "work_id lạ" mà không cần một
/// `tauri::AppHandle` thật (crate này không khai `tauri = { features = ["test-utils"] }`).
///
/// 🔴 **KHÔNG `remove_folder` ở BẤT KỲ nhánh lỗi nào** — khác hẳn [`create_work`]: `dir` ở
/// đây là **dữ liệu có sẵn của người dùng** (một `.atproj` đã tồn tại từ trước, được liệt
/// vào `library-index.db`), không phải một thư mục mà chính lượt gọi này vừa dựng. Một lỗi
/// đọc `meta.json`/`project.db` giữa chừng không được phép xoá dữ liệu người dùng — nó chỉ
/// được phép TỪ CHỐI MỞ.
///
/// # Lỗi
/// - `work_id` không có trong chỉ mục ⇒ `library.work_not_indexed`, `OpenWorkState` không
///   đổi (chỗ gọi chưa từng thấy `OpenWork` nào để đổi);
/// - `meta.json` mới hơn bản ứng dụng hiểu ⇒ [`crate::core::library::WorkError::MetaTooNew`]
///   (`work.meta_too_new`) — không một byte nào bị ghi (AC8);
/// - `meta.json` đọc trượt vì lý do KHÁC (thư mục biến mất, quyền đọc, …) ⇒
///   [`crate::core::library::WorkError::OpenFailed`] (`work.open_failed`);
/// - `project.db` mở trượt (kể cả `SchemaTooNew`) ⇒ lỗi kho (`store.*`), qua
///   `From<StoreError>`.
pub fn open_work(
    work_id: &str,
    indexed: Option<&crate::core::library::indexer::IndexedWork>,
) -> Result<OpenWork, IpcError> {
    let indexed = indexed.ok_or_else(|| work_not_indexed(work_id))?;
    let dir = indexed.atproj_path.clone();

    let meta = match WorkMeta::read(&dir) {
        Ok(meta) => meta,
        Err(crate::core::library::MetaError::SchemaTooNew { found, supported }) => {
            return Err(crate::core::library::WorkError::MetaTooNew { found, supported }.into());
        }
        Err(crate::core::library::MetaError::Io { detail, .. }) => {
            return Err(
                crate::core::library::WorkError::OpenFailed { name: indexed.name.clone(), detail }
                    .into(),
            );
        }
    };

    let db_path = dir.join("project.db");

    // ─────────────────────────────────────────────────────────────────────────────
    // 🔴 KIỂM TỆP CÓ MẶT **TRƯỚC** `Store::open` — NẾU KHÔNG, ĐƯỜNG NÀY GHI VÀO
    //    DỮ LIỆU NGƯỜI DÙNG Ở ĐÚNG NHÁNH LẼ RA PHẢI TỪ CHỐI
    // ─────────────────────────────────────────────────────────────────────────────
    // 🔵 **THÊM 2026-08-29 (lượt review)** — [`Store::open`] đi qua
    // `pragmas::open_connection`, hàm này mang **`SQLITE_OPEN_CREATE`**
    // (`core/store/pragmas.rs:45-47`). ⇒ Một `.atproj` còn `meta.json` lành lặn nhưng **mất**
    // `project.db` (xoá tay, ổ mạng chưa gắn, một lượt đồng bộ dở dang) **không** rơi vào
    // nhánh `OpenFailed` nào cả: nó ÂM THẦM TẠO một `project.db` RỖNG, chạy trọn 17 bước di
    // trú lên tệp mới ấy, rồi mới trượt ở câu `SELECT id FROM chapter` phía dưới bằng một lỗi
    // kho CHUNG CHUNG.
    //
    // 🔴 Hai điều sai cùng lúc, và điều thứ nhất nặng hơn: ① hàm này **GHI** vào thư mục của
    // người dùng ở đúng nhánh mà doc-comment của chính nó tuyên bố *"chỉ được phép TỪ CHỐI
    // MỞ"* — và tệp rỗng đó ở lại trên đĩa sau khi lượt mở trượt, cạnh một `meta.json` vẫn
    // khai `chapter_count > 0`, tức hai nửa của một `.atproj` nói ngược nhau; ② câu báo cho
    // người dùng là một lỗi kho, trong khi sự thật là *"Tác phẩm này thiếu mất `project.db`"*
    // — đúng lớp *"một câu SAI VỀ LOẠI"* mà Story 2.11 đã phải sửa một lần ở
    // `commands/chapter.rs::chapter_not_found`.
    //
    // ⚠️ Kiểm `exists()` là một phép kiểm CÓ CỬA SỔ ĐUA (tệp có thể biến mất ngay sau đó), và
    // nó **không** cần đóng cửa sổ ấy để đáng giá: ca thật ở đây là một tệp đã vắng mặt **từ
    // trước** lượt gọi, không phải một lượt xoá xảy ra đúng trong micro-giây đó. Cùng lý lẽ và
    // cùng khuôn `Store::peek_schema_version` (`core/store/mod.rs`), nơi vòng rà ba lớp P2 đã
    // phân xử đúng mệnh đề này cho `library-index.db`.
    if !db_path.exists() {
        return Err(crate::core::library::WorkError::OpenFailed {
            name: indexed.name.clone(),
            detail: format!("project.db khong ton tai trong {}", dir.display()),
        }
        .into());
    }

    let store = Store::open(StoreSpec::project(db_path))?;

    // Chương đầu theo `(ord, id)` -- §Design Notes "Vì sao KHÔNG có Chương mở gần nhất":
    // hôm nay mọi Tác phẩm có ĐÚNG một Chương, nên đây luôn là hàng duy nhất; câu SQL vẫn
    // viết đúng cho khi Chương thứ hai tồn tại (Epic 6/Story 5.8), không đoán trước hình
    // dạng UX của lượt đó.
    //
    // 🔵 **SỬA 2026-08-29 (lượt review)** — `query_row` biến "0 hàng" thành
    // `QueryReturnedNoRows`, tức một **lỗi KHO**, và người dùng đọc *"khong mo duoc kho du
    // lieu"* cho một `.atproj` mà tệp không hỏng gì cả. Đây là nguyên văn lớp lỗi mà Story
    // 2.11 đã sửa ở `commands/chapter.rs::chapter_not_found` (xem doc-comment hàm đó). Một
    // Tác phẩm **không Chương nào** là một trạng thái của DỮ LIỆU, không của kho ⇒ nó đi ra
    // bằng `work.open_failed`, mang TÊN Tác phẩm để người dùng nhận ra mình đang nói về cái
    // nào.
    //
    // ⚠️ **Vì sao TÁI DÙNG `OpenFailed` chứ không đúc một `MessageKey` thứ tư.** Hôm nay
    // **0** đường sản phẩm nào tạo được một Tác phẩm không Chương: `create_work` luôn chèn
    // đúng một hàng `chapter` trong cùng giao dịch, và **0** đường nào xoá Chương (FR15 là
    // Story 5.8). Ca này chỉ tới được từ một `project.db` bị sửa tay hoặc hỏng. Một khoá
    // riêng cho nó hôm nay là *"một khoá cho một nhánh không chỗ gọi nào đi qua"* — đúng thứ
    // Story 1.7 §Completion Notes #3 cấm. ⇒ Khi **Story 5.8** mở đường xoá Chương, ca này có
    // một chỗ gọi SẢN PHẨM và **lúc đó** nó đáng một khoá riêng.
    // 🔴 `query_map().next()` chứ KHÔNG `query_row` — chép nguyên khuôn
    // `commands/chapter.rs::read_open_chapter`, và vì đúng lý do đã ghi ở đó: `query_row`
    // biến "0 hàng" thành một `QueryReturnedNoRows`, tức một lỗi KHO. `Option` ở đây là một
    // trạng thái DỮ LIỆU bình thường, và tầng này đổi nó thành một lỗi CÓ TÊN.
    //
    // ⚠️ Và **không** bắt nó bằng cách so chuỗi trong `detail` của `StoreError`: `Store::read`
    // gói mọi `Err` thành một `detail: String`, nên đoán lại lý do từ chuỗi đó là một chẩn
    // đoán SAI cho hai ca khác hẳn nhau (0 hàng / kho hỏng thật) — cùng lý lẽ định lượng mà
    // `commands/segment.rs::save_segment_targets` đã ghi cho ô CÓ KIỂU của nó.
    let found = match store.read(|conn| {
        let mut stmt = conn.prepare("SELECT id FROM chapter ORDER BY ord, id LIMIT 1")?;
        let mut rows = stmt.query_map([], |row| row.get::<_, i64>(0))?;
        rows.next().transpose()
    }) {
        Ok(found) => found,
        Err(err) => {
            store.close();
            return Err(err.into());
        }
    };

    let Some(chapter_id) = found else {
        store.close();
        return Err(crate::core::library::WorkError::OpenFailed {
            name: indexed.name.clone(),
            detail: "project.db khong co hang chapter nao".to_owned(),
        }
        .into());
    };

    let scope = crate::core::scope::ScopeResolver::with_work(crate::core::scope::WorkScope {
        work_id: meta.work_id.clone(),
    });

    // Story 6.11 -- mo lai KHONG bao gio chay pha anh (chi `create_work` chay), nen ca HAI
    // truong moi deu la gia tri "chua tung chay" -- 0/0.
    // 🔵 SUA 2026-09-09 (vong ra doi khang 2, muc D8) -- cau cu khai BA truong; truong thu ba
    // (`pending_domain_log`) da bi GO hoan toan khoi OpenWork tu luot sua B1 (domain_log_state
    // nay la mot tham so ngoai, khong con la mot truong tra ve) -- chi con HAI truong that.
    Ok(OpenWork { dir, store, scope, meta, chapter_id, images_saved: 0, images_failed: 0 })
}

/// Kiểu state Tauri quản lý — Tác phẩm đang mở, hoặc chưa mở gì (Task 7).
///
/// ⚠️ `Mutex`, không `RwLock`: đúng một Tác phẩm mở tại một thời điểm, và mọi thao tác
/// đọc/ghi field của nó (thay Tác phẩm khác, đóng lúc thoát) đều là **thao tác độc quyền**
/// — không có nhánh "nhiều reader cùng lúc" nào ở tầng state này (khác hẳn `Store::read`
/// bên trong, nơi pool nhiều kết nối đã lo phần đó).
pub type OpenWorkState = std::sync::Mutex<Option<OpenWork>>;

/// Thay Tác phẩm đang mở (nếu có) bằng `new_work` — **Store cũ tự đóng qua `Drop`**.
///
/// ⚠️ Nếu `OpenWorkState` chưa từng được `app.manage(...)` (lỗi cấu hình `setup()`, không
/// phải đường sản phẩm bình thường), `new_work` bị drop ngay khi hàm này return — Tác
/// phẩm vừa tạo đóng lại tức thì. Đây là im lặng có chủ ý: cùng khuôn
/// `close_global_store`/`try_state`, không panic khi state vắng mặt.
///
/// ─────────────────────────────────────────────────────────────────────────────
/// 🔴 AC10 (Story 1.16) — `Store` CŨ THẢ **NGOÀI** VÙNG KHOÁ, KHÔNG bên trong
/// ─────────────────────────────────────────────────────────────────────────────
/// Lượt review 2026-08-06 dự báo đúng: `*guard = Some(new_work)` chạy `Drop` của giá trị
/// CŨ (đóng `Store` — TRUNCATE có trần, `core::store::Store::close`) **trong khi `guard`
/// vẫn giữ khoá**, vì Rust drop giá trị bị ghi đè ngay tại chỗ gán, và `guard` chỉ nhả khoá
/// ở cuối khối. Hôm nay vô hại (chưa command nào khác đọc `OpenWorkState`), nhưng story
/// này thêm command **đầu tiên đọc** nó (`commands::chapter::wire::read_open_chapter`) —
/// đóng một `Store` giữ khoá mutex chặn mọi lượt đọc đó trong lúc TRUNCATE chạy.
///
/// Khuôn đúng: `Mutex::replace` trả **giá trị cũ**, gán trong một khối con để `guard` nhả
/// khoá ngay khi khối đó kết thúc, RỒI mới `drop(old)` — Store cũ đóng khi không ai còn
/// giữ khoá.
fn replace_open_work(app: &tauri::AppHandle, new_work: OpenWork) {
    use tauri::Manager as _;

    // ─────────────────────────────────────────────────────────────────────────────
    // THEM Story 6.14 (FR42/FR43, AD-23) -- CAP SCOPE DONG cho dung thu muc `.atproj` dang mo
    // ─────────────────────────────────────────────────────────────────────────────
    // Anh cua Story 6.11 nam trong `<dir>/assets/`; webview doc chung qua `asset://` (CSP da
    // cho san, `tauri.conf.json` KHONG doi mot byte -- nut that DUY NHAT la scope). AD-23 da
    // chot san ve nay: "Scope dong cap luc chay chi khi nguoi dung chon qua hop thoai -- thu
    // muc goc Library" -- day la NANG LUC CHUA DUNG cua ve do, khong phai mot bat bien bi doi,
    // nen khong can AD moi (Design Notes spec 6.14).
    //
    // Cho nay la nut that CHUNG cua CA BON duong mo mot Tac pham (`create_work_from_text`,
    // `create_work_from_file`, `confirm_import_with_encoding`, `open_work`) -- xem bon cho goi
    // `replace_open_work(...)` trong `wire` ben duoi. Cap scope O DAY, mot lan, thay vi rai
    // lai bon lan, dung khuon "mot nut that" da co cho `PendingImportState`/glossary ngay
    // duoi day.
    //
    // Cap HEP hon muc AD-23 cho phep (ca thu muc goc Library) la co y: chi thu muc `.atproj`
    // CU THE dang mo, khong phai ca goc Library -- hep hon thi khong can xin them.
    //
    // Loi cap scope KHONG chan luot mo Tac pham: day la mot cai gia da can (mot Tac pham mo
    // duoc ma anh khong hien la mot khiem khuyet HIEN THI, khac han mot Tac pham khong mo
    // duoc). Chi ghi chan doan (KHONG DAU, NFR16) -- `<img>` se truot va webview doi sang
    // khung giu cho mang danh tinh (FR43), khong throw, khong trang trang.
    // 🔴 HEP toi `<dir>/assets`, KHONG ca thu muc `.atproj` -- vong ra 2026-09-10. Cap ca
    // `.atproj` phoi luon `project.db` ra `asset://` cho webview doc duoc, trong khi AD-1/AD-11
    // dat MOI truy cap du lieu o Rust va `SECURITY-NOTES.md` da khai cung mot le do cho
    // `$APPDATA` ("frontend khong co viec gi voi global.db"). Anh cua FR42/FR43 chi nam trong
    // `assets/`, va do dung bang `ChapterSegments::assets_dir`/`ReadingRun::assets_dir` ma
    // webview ghep duong dan -- nen cap dung chung mot thu muc do la du, va hep hon thi khong
    // can xin them.
    let assets_dir = new_work.dir.join("assets");
    if let Err(err) = app.asset_protocol_scope().allow_directory(&assets_dir, true) {
        eprintln!(
            "project[scope] khong cap duoc asset_protocol_scope cho {}: {err}",
            assets_dir.display()
        );
    }

    // Story 3.10b (AD-48) -- mo mot Tac pham KHAC lam `project.db` cua no doi hoan toan; mot
    // lo nhap Glossary dang TREO o tang Work (neu co) tro toi kho CU, va `RowPlanKind::
    // Conflict::existing_id` cua no khong con dung nghia o kho MOI. Don TRUOC khi swap, cung
    // khuon `close_open_work` (`lib.rs`) -- ca hai duong deu lam mot `project.db` bien mat
    // khoi tam voi cua lo dang treo.
    if let Some(pending) = app.try_state::<crate::commands::glossary::PendingImportState>() {
        crate::commands::glossary::clear_pending_import_for_tier(
            &pending,
            crate::core::glossary::GlossaryTier::Work,
        );
    }

    if let Some(state) = app.try_state::<OpenWorkState>() {
        drop(swap_locked(&state, new_work));
    }
}

/// Thay giá trị bên trong `mutex` bằng `new`, trả về giá trị **CŨ** — **không** tự
/// `drop` nó ở đây. Đó là toàn bộ điểm của hàm này (AC10): `guard` nhả khoá ở cuối khối
/// `lock()`/`replace()`, và giá trị cũ chỉ bị drop **sau đó**, ở chỗ gọi
/// ([`replace_open_work`]) — chứ không trong khi khoá vẫn còn giữ.
///
/// Tách thành một hàm **thuần theo kiểu** (`T` bất kỳ, không riêng `OpenWork`) là điều
/// kiện để [`tests::swap_locked_drops_the_old_value_after_the_lock_is_released`] kiểm
/// được đúng thuộc tính đó bằng một kiểu dò tự khoá lại chính `mutex` trong `Drop` của nó
/// — dựng một `OpenWork` thật (mở `Store`) chỉ để kiểm thứ tự khoá/drop là một chi phí
/// không cần thiết cho một mệnh đề thuần về **thứ tự**.
fn swap_locked<T>(mutex: &std::sync::Mutex<Option<T>>, new: T) -> Option<T> {
    let mut guard = mutex
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    guard.replace(new)
}

#[cfg(test)]
mod tests {
    use super::{
        ImportScanGeneration, ImportScanNextStep, SavedAsset, create_work_from_text,
        dictionary_inconclusive_event, dictionary_probe_from_grouped,
        filter_and_enqueue_current_import_scan, guarded_dict_layers, guarded_open_store,
        import_scan_next_step, keep_committed_import_when_scan_spawn_fails,
        read_chapter_segment_texts, saved_asset_chapter_index_is_in_range,
        saved_asset_satisfies_asset_check_constraints, swap_locked,
    };
    use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
    use std::sync::{Arc, Mutex};

    /// Thư mục tạm CỦA RIÊNG ca này — pid + `AtomicU64`, cùng luật bốn điều của
    /// `glossary_contract.rs`/`glossary_commands_contract.rs` (mỗi ca một thư mục riêng;
    /// `Store` drop TRƯỚC khi xoá; không `sleep` dài; không ca nào treo khi trượt).
    static NEXT_GUARD_DIR: AtomicU64 = AtomicU64::new(0);

    fn guard_test_dir(tag: &str) -> std::path::PathBuf {
        let n = NEXT_GUARD_DIR.fetch_add(1, Ordering::Relaxed);
        let dir = std::env::temp_dir().join(format!(
            "auratranslate-project-guard-{}-{}-{}",
            std::process::id(),
            tag,
            n
        ));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap_or_else(|e| panic!("tao {}: {e}", dir.display()));
        dir
    }

    fn guard_test_cleanup(dir: &std::path::Path) {
        let _ = std::fs::remove_dir_all(dir);
    }

    /// Adapter TEST `bool -> DictionaryProbe`, giữ CỤC BỘ ở bàn test này — 🔵 2026-08-26
    /// (cụm F). `core::glossary::scan::scan_candidates` (vỏ `bool` công khai) đã bị xoá:
    /// nó có 0 chỗ gọi sản phẩm và biến một layer LỖI thành "không có trong từ điển". Đường
    /// sản phẩm thật của `spawn_import_scan` tiêm closure gọi `dictionary_probe_from_grouped`
    /// thẳng vào `scan_candidates_controlled`; ca test dưới đây chỉ cần một vị từ `bool` tất
    /// định nên tự giữ đúng phần thân adapter đã xoá, không phục hồi một API sản phẩm.
    fn scan_candidates_bool_probe(
        segments: &[&str],
        lang: crate::core::matching::MatchLang,
        threshold: u32,
        surnames: &[char],
        is_known: &mut dyn FnMut(&str) -> bool,
    ) -> Vec<crate::core::glossary::ScanCandidate> {
        let mut probe = |term: &str| {
            if is_known(term) {
                crate::core::glossary::DictionaryProbe::Known
            } else {
                crate::core::glossary::DictionaryProbe::Missing
            }
        };
        let mut never_cancelled = || false;
        match crate::core::glossary::scan_candidates_controlled(
            segments,
            lang,
            threshold,
            surnames,
            &mut probe,
            &mut never_cancelled,
        ) {
            crate::core::glossary::ScanOutcome::Completed(out) => out,
            crate::core::glossary::ScanOutcome::DictionaryInconclusive
            | crate::core::glossary::ScanOutcome::Cancelled => Vec::new(),
        }
    }

    // ═════════════════════════════════════════════════════════════════════════════════
    // I/O Matrix — "Kho đóng giữa lượt quét ⇒ luồng nền kết thúc lặng lẽ, không panic",
    // mở rộng cho ca "Tác phẩm đổi" — Story 3.5, rà bảng I/O phát hiện hàng này KHÔNG có
    // test nào canh (`spawn_import_scan` nhận `AppHandle` nên `tests/**` không gọi tới nó
    // được). `guarded_open_store` là đơn vị quyết định được TÁCH RA đúng luật hai lớp của
    // `src-tauri/AGENTS.md` để ba ca dưới đây canh được TRỰC TIẾP, không cần webview/luồng.
    // ═════════════════════════════════════════════════════════════════════════════════

    /// Ca ① — không có Tác phẩm nào đang mở (`OpenWorkState` là `None`, hoặc — như ở đây,
    /// nơi hàm nhận thẳng `Option<&OpenWork>` — chỗ gọi truyền `None`) ⇒ dừng lặng lẽ.
    #[test]
    fn guarded_open_store_returns_none_when_no_work_is_open() {
        assert!(
            guarded_open_store(None, "bat-ky-work-id-nao").is_none(),
            "khong co Tac pham nao dang mo -- phai tra None, khong panic"
        );
    }

    /// Nối trọn hàng Matrix "B thay A": generation B sinh NGAY TRONG pha đếm của A;
    /// hook mà worker thật dùng thấy A stale, scan trả `Cancelled`, lookup = 0 và helper
    /// hậu-scan chọn `Stop` — biến thể duy nhất không enqueue/không phát completion.
    #[test]
    fn a_new_import_generation_cancels_the_old_scan_before_lookup_write_or_completion() {
        let dir = guard_test_dir("generation-cancels-scan");
        let opened = create_work_from_text(&dir, "Generation", "en", "", "source".to_owned())
            .unwrap_or_else(|e| panic!("tao Tac pham that bai: {e:?}"));

        let generation = ImportScanGeneration::default();
        let generation_a = generation.next();
        let segments: Vec<String> = (0..500)
            .map(|i| format!("a beast called Fire Dragon appeared at hour {i}."))
            .collect();
        let refs: Vec<&str> = segments.iter().map(String::as_str).collect();
        let mut lookup_calls = 0usize;
        let mut probe = |_term: &str| {
            lookup_calls += 1;
            crate::core::glossary::DictionaryProbe::Missing
        };
        let mut cancellation_checks = 0usize;
        let mut current_generation = || {
            cancellation_checks += 1;
            if cancellation_checks == 3 {
                let _generation_b = generation.next();
            }
            !generation.is_current(generation_a)
        };

        let outcome = crate::core::glossary::scan_candidates_controlled(
            &refs,
            crate::core::matching::MatchLang::En,
            5,
            crate::core::glossary::COMMON_SURNAMES,
            &mut probe,
            &mut current_generation,
        );
        let next = import_scan_next_step(outcome, generation.is_current(generation_a));

        assert_eq!(next, ImportScanNextStep::Stop);
        assert_eq!(lookup_calls, 0, "generation cu phai dung ngay trong count");
        let pending = crate::core::glossary::pending_candidates(&opened.store)
            .expect("doc bang cho doi chung 0 write");
        assert!(pending.is_empty(), "Stop khong duoc xep bat ky batch nao");

        drop(opened.store);
        guard_test_cleanup(&dir);
    }

    /// Lái chính mapping mà worker gọi bằng một `GroupedLookup` có layer lỗi thật về MẶT
    /// kiểu dữ liệu. Outcome phải là `dictionary_inconclusive`; next-step không mang batch,
    /// và bảng Work vẫn rỗng — không chỉ kiểm một predicate thuần tách rời.
    #[test]
    fn a_skipped_dictionary_layer_maps_through_the_worker_decision_to_zero_batch_writes() {
        let dir = guard_test_dir("dictionary-inconclusive");
        let opened =
            create_work_from_text(&dir, "Dict Inconclusive", "en", "", "source".to_owned())
                .unwrap_or_else(|e| panic!("tao Tac pham that bai: {e:?}"));
        let grouped = crate::core::dict::GroupedLookup {
            route: crate::core::dict::QueryRoute::En,
            branch: crate::core::dict::QueryBranch::ExactBtree,
            groups: Vec::new(),
            skipped: vec![crate::core::dict::SkippedLayer {
                path: std::path::PathBuf::from("broken-layer.db"),
                reason: crate::core::dict::SkipReason::OpenFailed {
                    detail: "fixture open failure".to_owned(),
                },
            }],
            truncated_layers: Vec::new(),
            // `skipped` phải thắng cả bằng chứng hit bị cắt trang: một layer hỏng làm
            // toàn lượt không kết luận, không cho hit ở layer khác che mất lỗi.
            hidden_sources: vec![("hidden source".to_owned(), 1)],
            layers_loaded: false,
        };
        let segments: Vec<String> = (0..5)
            .map(|i| format!("a beast called Fire Dragon appeared at hour {i}."))
            .collect();
        let refs: Vec<&str> = segments.iter().map(String::as_str).collect();
        let mut probe = |_term: &str| dictionary_probe_from_grouped(&grouped);
        let mut never_cancelled = || false;

        let outcome = crate::core::glossary::scan_candidates_controlled(
            &refs,
            crate::core::matching::MatchLang::En,
            5,
            crate::core::glossary::COMMON_SURNAMES,
            &mut probe,
            &mut never_cancelled,
        );
        let next = import_scan_next_step(outcome, true);

        assert_eq!(next, ImportScanNextStep::EmitDictionaryInconclusive);
        let pending = crate::core::glossary::pending_candidates(&opened.store)
            .expect("doc bang cho doi chung 0 write");
        assert!(
            pending.is_empty(),
            "dictionary inconclusive khong mang batch de enqueue"
        );

        drop(opened.store);
        guard_test_cleanup(&dir);
    }

    #[test]
    fn a_hidden_source_is_a_known_hit_when_no_dictionary_layer_was_skipped() {
        let grouped = crate::core::dict::GroupedLookup {
            route: crate::core::dict::QueryRoute::En,
            branch: crate::core::dict::QueryBranch::ExactBtree,
            groups: Vec::new(),
            skipped: Vec::new(),
            truncated_layers: vec!["base".to_owned()],
            hidden_sources: vec![("source cut cleanly by limit".to_owned(), 2)],
            layers_loaded: true,
        };

        assert_eq!(
            dictionary_probe_from_grouped(&grouped),
            crate::core::glossary::DictionaryProbe::Known,
            "hidden_sources da chung minh co hit, nen Known thang truncated"
        );
    }

    #[test]
    fn a_truncated_layer_without_a_visible_or_hidden_hit_is_inconclusive_not_missing() {
        let grouped = crate::core::dict::GroupedLookup {
            route: crate::core::dict::QueryRoute::En,
            branch: crate::core::dict::QueryBranch::ExactBtree,
            groups: Vec::new(),
            skipped: Vec::new(),
            truncated_layers: vec!["base".to_owned()],
            hidden_sources: Vec::new(),
            layers_loaded: true,
        };

        assert_eq!(
            dictionary_probe_from_grouped(&grouped),
            crate::core::glossary::DictionaryProbe::Inconclusive,
            "truncated khong du bang chung de ket luan Missing"
        );
    }

    #[test]
    fn the_dictionary_inconclusive_payload_serializes_the_reviewed_outcome_and_zero_counts() {
        let payload = dictionary_inconclusive_event(42);

        assert_eq!(
            serde_json::to_value(payload).expect("serialize payload"),
            serde_json::json!({
                "chapter_id": 42,
                "inserted": 0,
                "skipped": 0,
                "outcome": "dictionary_inconclusive",
            })
        );
    }

    /// `thread::Builder::spawn` lỗi được tiêm thẳng qua seam mà hai wire command dùng.
    /// Import đã commit vẫn đọc được từ SQLite và thư mục không bị đảo ngược/xoá.
    #[test]
    fn a_spawn_failure_after_commit_preserves_the_import_and_returns_normally() {
        let dir = guard_test_dir("spawn-failure-after-commit");
        let opened = create_work_from_text(
            &dir,
            "Spawn Failure",
            "en",
            "",
            "a committed source sentence.".to_owned(),
        )
        .unwrap_or_else(|e| panic!("tao Tac pham that bai: {e:?}"));
        let project_dir = opened.dir.clone();
        let chapter_id = opened.chapter_id;

        let opened = keep_committed_import_when_scan_spawn_fails(opened, || {
            Err(std::io::Error::other("injected thread spawn failure"))
        });

        let rows = read_chapter_segment_texts(&opened.store, chapter_id)
            .expect("import da commit phai con doc duoc sau spawn Err");
        assert_eq!(rows, vec!["a committed source sentence."]);
        assert!(project_dir.join("project.db").is_file());
        // 🔵 SỬA (2026-08-28, Story 5.5) — dùng `WorkMeta::path_in` thay vì chuỗi `"meta.json"`
        // viết thẳng (hay nhắc thẳng `META_FILE`): `meta_write_boundary.rs` khoá cả hai hình
        // dạng đó CHỈ ở `core/library/meta.rs`, và một bản chép tay ở đây là đúng thứ cổng đó
        // tồn tại để bắt.
        assert!(crate::core::library::meta::WorkMeta::path_in(&project_dir).is_file());

        drop(opened.store);
        guard_test_cleanup(&dir);
    }

    /// Ca ② — `work_id` đã đổi giữa hai lần khoá (Tác phẩm CŨ đã bị thay bằng một Tác
    /// phẩm KHÁC trong `OpenWorkState`) ⇒ dừng lặng lẽ, **0 ghi**. Đối chứng bằng `SELECT`
    /// qua `pending_candidates` — không chỉ tin giá trị trả về `None` — bằng cách lái qua
    /// ĐÚNG hình dạng mà `spawn_import_scan` dùng: chỉ ghi khi `guarded_open_store` trả
    /// `Some`. Nếu vệ bảo vệ bị gỡ (hoặc hỏng), ứng viên GIẢ ở dưới sẽ lọt vào bảng chờ và
    /// ca này đỏ.
    #[test]
    fn guarded_open_store_returns_none_and_blocks_every_write_when_the_work_id_has_changed_mid_scan()
    {
        let dir = guard_test_dir("work-id-changed");
        let opened = create_work_from_text(
            &dir,
            "Doi Tac Pham Giua Chung",
            "zh",
            "",
            "萧炎登场".to_owned(),
        )
            .unwrap_or_else(|e| panic!("tao Tac pham that bai: {e:?}"));

        // `work_id` CHỐT LÚC SPAWN không còn khớp `opened.meta.work_id` -- mô phỏng đúng
        // ca "Tác phẩm đổi giữa hai lần khoá": `OpenWorkState` nay trỏ một Tác phẩm KHÁC.
        let stale_work_id = "khong-con-la-tac-pham-nay";
        assert_ne!(
            opened.meta.work_id, stale_work_id,
            "fixture phai thuc su lech work_id"
        );

        let fake_candidates = vec![crate::core::glossary::ScanCandidate {
            source_term: "萧炎".to_owned(),
            occurrence_count: 99,
            context_example: "cau gia.".to_owned(),
        }];

        // Đúng khuôn production ở `spawn_import_scan`: chỉ ghi khi `guarded_open_store`
        // trả `Some`.
        if let Some(store) = guarded_open_store(Some(&opened), stale_work_id) {
            let _ = crate::core::glossary::insert_import_scan_candidates(store, &fake_candidates);
        }

        let pending = crate::core::glossary::pending_candidates(&opened.store)
            .expect("doc bang cho de doi chung -- day la ve SELECT, khong chi tin gia tri tra ve");
        assert!(
            pending.is_empty(),
            "work_id lech ⇒ 0 hang duoc phep ghi vao bang cho ung vien. Nhan: {pending:?}"
        );

        drop(opened.store);
        guard_test_cleanup(&dir);
    }

    /// Ca ③ — ca THƯỜNG: `work_id` khớp ở CẢ HAI lần khoá ⇒ lượt quét chạy hết và ghi.
    /// Đối chứng dương của ca ② — không có nó thì "0 ghi" ở ca ② có thể xanh vì thuật toán
    /// quét/ghi tự nó hỏng, không phải vì vệ bảo vệ đúng.
    #[test]
    fn guarded_open_store_returns_the_store_and_a_normal_scan_runs_to_completion_and_writes() {
        let dir = guard_test_dir("normal-run");
        let text: String = (0..6).map(|i| format!("萧炎在第{i}章登场")).collect();
        let opened = create_work_from_text(&dir, "Ca Thuong", "zh", "", text)
            .unwrap_or_else(|e| panic!("tao Tac pham that bai: {e:?}"));
        let work_id = opened.meta.work_id.clone();

        // Lần khoá THỨ NHẤT (đọc segment) -- cùng `work_id` đã chốt lúc spawn.
        let store = guarded_open_store(Some(&opened), &work_id)
            .expect("work_id khop o lan khoa thu nhat -- phai tra Some(&store)");
        let segments = read_chapter_segment_texts(store, opened.chapter_id).expect("doc segment");
        let segment_refs: Vec<&str> = segments.iter().map(String::as_str).collect();

        let mut is_known = |_: &str| false;
        let candidates = scan_candidates_bool_probe(
            &segment_refs,
            crate::core::matching::MatchLang::Zh,
            5,
            crate::core::glossary::COMMON_SURNAMES,
            &mut is_known,
        );
        assert!(
            !candidates.is_empty(),
            "van ban mau (6 lan '萧炎') phai sinh it nhat mot ung vien"
        );

        // Lần khoá THỨ HAI (ghi lô) -- cùng `work_id`, đúng hình dạng hai-lần-khoá-ngắn.
        let store_for_write = guarded_open_store(Some(&opened), &work_id)
            .expect("work_id van khop o lan khoa thu hai -- phai tra Some(&store)");
        let (inserted, _skipped) =
            crate::core::glossary::insert_import_scan_candidates(store_for_write, &candidates)
                .expect("ghi lo");
        assert!(inserted > 0, "ca thuong phai ghi duoc it nhat mot hang");

        let pending =
            crate::core::glossary::pending_candidates(&opened.store).expect("doc lai bang cho");
        assert!(
            !pending.is_empty(),
            "bang cho phai co hang sau mot luot quet binh thuong"
        );

        drop(opened.store);
        guard_test_cleanup(&dir);
    }

    /// Một term ở Global phải biến mất TRƯỚC câu `INSERT` Work, nhưng vẫn cộng vào
    /// `skipped`. Đối chứng giữ một term khác để chứng minh lô không bị xoá trắng.
    #[test]
    fn global_and_work_glossary_terms_are_resolved_before_the_batch_and_counted_as_skipped() {
        let dir = guard_test_dir("global-filter");
        let global = crate::core::store::Store::open(crate::core::store::StoreSpec::global(
            dir.join("global.db"),
        ))
        .expect("mo global.db");
        crate::commands::glossary::glossary_add_term(
            Some(&global),
            None,
            crate::core::glossary::GlossaryTier::Global,
            "Fire Dragon",
            Some("Hoa Long"),
            "",
            crate::core::glossary::Category::Other,
        )
        .expect("chen term global");

        let opened = create_work_from_text(&dir, "Loc Hai Tang", "en", "", "source".to_owned())
            .unwrap_or_else(|e| panic!("tao Tac pham that bai: {e:?}"));
        crate::commands::glossary::glossary_add_term(
            Some(&global),
            Some(&opened),
            crate::core::glossary::GlossaryTier::Work,
            "Ice Phoenix",
            Some("Bang Phuong"),
            "",
            crate::core::glossary::Category::Other,
        )
        .expect("chen term Work");
        let work_id = opened.meta.work_id.clone();
        let state = Mutex::new(Some(opened));
        let mut candidates = vec![
            crate::core::glossary::ScanCandidate {
                source_term: "Fire Dragon".to_owned(),
                occurrence_count: 7,
                context_example: "A beast called Fire Dragon arrived.".to_owned(),
            },
            crate::core::glossary::ScanCandidate {
                source_term: "Ice Phoenix".to_owned(),
                occurrence_count: 6,
                context_example: "A beast called Ice Phoenix arrived.".to_owned(),
            },
            crate::core::glossary::ScanCandidate {
                source_term: "Storm Tiger".to_owned(),
                occurrence_count: 5,
                context_example: "A beast called Storm Tiger arrived.".to_owned(),
            },
        ];

        let ticket = filter_and_enqueue_current_import_scan(
            &state,
            &work_id,
            &global,
            &mut candidates,
            &|| true,
        )
        .expect("loc va enqueue")
        .expect("work_id dang mo phai cho phep enqueue");
        let (inserted, skipped) = ticket.wait().expect("writer tra loi");
        assert_eq!((inserted, skipped), (1, 2));

        let guard = state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let pending =
            crate::core::glossary::pending_candidates(&guard.as_ref().expect("work con mo").store)
                .expect("doc bang cho");
        assert_eq!(pending.len(), 1);
        assert_eq!(pending[0].source_term, "Storm Tiger");
        drop(guard);

        let opened = state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .take();
        drop(opened);
        drop(global);
        guard_test_cleanup(&dir);
    }

    /// Generation đổi đúng SAU khi scope filter đã chạy nhưng TRƯỚC enqueue. Callback là
    /// điểm kiểm chính worker dùng, nên `None` ở đây đồng nghĩa không có write-ticket nào
    /// được tạo; bảng chờ là đối chứng SQL cho vế 0 write.
    #[test]
    fn a_generation_that_turns_stale_after_scope_filtering_creates_no_ticket_and_writes_nothing() {
        let dir = guard_test_dir("late-cancellation-after-scope");
        let global = crate::core::store::Store::open(crate::core::store::StoreSpec::global(
            dir.join("global.db"),
        ))
        .expect("mo global.db");
        crate::commands::glossary::glossary_add_term(
            Some(&global),
            None,
            crate::core::glossary::GlossaryTier::Global,
            "Fire Dragon",
            Some("Hoa Long"),
            "",
            crate::core::glossary::Category::Other,
        )
        .expect("chen term Global de chung minh scope filter da chay");

        let opened = create_work_from_text(&dir, "Late Cancel", "en", "", "source".to_owned())
            .unwrap_or_else(|e| panic!("tao Tac pham that bai: {e:?}"));
        let work_id = opened.meta.work_id.clone();
        let state = Mutex::new(Some(opened));
        let mut candidates = vec![
            crate::core::glossary::ScanCandidate {
                source_term: "Fire Dragon".to_owned(),
                occurrence_count: 5,
                context_example: "A beast called Fire Dragon arrived.".to_owned(),
            },
            crate::core::glossary::ScanCandidate {
                source_term: "Ice Phoenix".to_owned(),
                occurrence_count: 5,
                context_example: "A beast called Ice Phoenix arrived.".to_owned(),
            },
        ];
        let generation = ImportScanGeneration::default();
        let generation_a = generation.next();
        let checks = AtomicUsize::new(0);
        let current = || {
            checks.fetch_add(1, Ordering::Relaxed);
            let _generation_b = generation.next();
            generation.is_current(generation_a)
        };

        let ticket = filter_and_enqueue_current_import_scan(
            &state,
            &work_id,
            &global,
            &mut candidates,
            &current,
        )
        .expect("scope filter thanh cong truoc cancellation");

        assert!(
            ticket.is_none(),
            "stale sau filter khong duoc tao write-ticket"
        );
        assert_eq!(
            checks.load(Ordering::Relaxed),
            1,
            "mot check tat dinh ngay truoc enqueue"
        );
        assert_eq!(
            candidates
                .iter()
                .map(|c| c.source_term.as_str())
                .collect::<Vec<_>>(),
            vec!["Ice Phoenix"],
            "Global term da bi loc, chung minh cancellation xay ra SAU scope filtering"
        );
        let guard = state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let pending =
            crate::core::glossary::pending_candidates(&guard.as_ref().expect("work con mo").store)
                .expect("doc bang cho doi chung");
        assert!(pending.is_empty(), "0 ticket phai tuong ung 0 write");
        drop(guard);

        let opened = state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .take();
        drop(opened);
        drop(global);
        guard_test_cleanup(&dir);
    }

    /// Writer bị chặn bằng kênh tất định (không sleep/timing). Helper phải trả ticket và
    /// `OpenWorkState::try_lock` phải thành công TRƯỚC khi job cản được thả.
    #[test]
    fn a_slow_writer_never_keeps_open_work_state_locked_while_the_ticket_waits() {
        let dir = guard_test_dir("writer-ticket-unlocks-state");
        let global = crate::core::store::Store::open(crate::core::store::StoreSpec::global(
            dir.join("global.db"),
        ))
        .expect("mo global.db");
        let opened = create_work_from_text(&dir, "Writer Cham", "en", "", "source".to_owned())
            .unwrap_or_else(|e| panic!("tao Tac pham that bai: {e:?}"));
        let work_id = opened.meta.work_id.clone();

        let (started_tx, started_rx) = std::sync::mpsc::channel();
        let (release_tx, release_rx) = std::sync::mpsc::channel();
        let blocker = opened
            .store
            .write_ticket(move |_tx| {
                let _ = started_tx.send(());
                let _ = release_rx.recv();
                Ok(())
            })
            .expect("xep job can writer");
        started_rx.recv().expect("writer phai vao job can");

        let state = Mutex::new(Some(opened));
        let mut candidates = vec![crate::core::glossary::ScanCandidate {
            source_term: "Fire Dragon".to_owned(),
            occurrence_count: 5,
            context_example: "A beast called Fire Dragon arrived.".to_owned(),
        }];
        let scan_ticket = filter_and_enqueue_current_import_scan(
            &state,
            &work_id,
            &global,
            &mut candidates,
            &|| true,
        )
        .expect("loc va enqueue")
        .expect("work dang mo");

        assert!(
            state.try_lock().is_ok(),
            "ticket da xep sau writer cham nhung OpenWorkState phai duoc nha truoc wait"
        );
        release_tx.send(()).expect("tha writer");
        blocker.wait().expect("job can ket thuc");
        scan_ticket.wait().expect("lo scan ket thuc");

        let opened = state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .take();
        drop(opened);
        drop(global);
        guard_test_cleanup(&dir);
    }

    // ═════════════════════════════════════════════════════════════════════════════════
    // Rà ba lớp 2026-08-22 — `guarded_dict_layers` KHÔNG được nuốt ca "chưa quản lý" thành
    // ca "rỗng bình thường".
    // ═════════════════════════════════════════════════════════════════════════════════

    /// Bản lỗi (`unwrap_or(&DictLayers::empty())`) coi `None` và `Some(rỗng)` là MỘT — ca
    /// này canh đúng vế bị nuốt: `None` (chưa quản lý) phải LAN RA `None`, không âm thầm
    /// đổi thành "rỗng nhưng hợp lệ".
    #[test]
    fn guarded_dict_layers_returns_none_and_does_not_silently_fall_back_to_empty_when_not_managed()
    {
        assert!(
            guarded_dict_layers(None, "import_scan").is_none(),
            "DictLayers chua duoc quan ly -- phai lan None ra ngoai, khong tu doi thanh rong"
        );
    }

    /// Đối chứng dương: một `DictLayers` ĐÃ quản lý (kể cả khi rỗng — trạng thái bình
    /// thường, AD-25) phải đi qua NGUYÊN VẸN, không hàm này tự tráo bằng một bản khác.
    #[test]
    fn guarded_dict_layers_passes_the_managed_layers_through_unchanged() {
        let layers = crate::core::dict::DictLayers::empty();
        let out = guarded_dict_layers(Some(&layers), "import_scan");
        assert!(
            out.is_some(),
            "DictLayers da quan ly (du rong) van phai di qua -- day la trang thai binh thuong"
        );
        assert!(
            std::ptr::eq(out.expect("da kiem is_some o tren"), &layers),
            "phai tra ve DUNG tham chieu da nhan, khong dung mot ban thay the nao khac"
        );
    }

    /// 🔴 **AC10 (Story 1.16)** — kiểm bằng chính cơ chế mà lỗi biểu hiện: giá trị CŨ,
    /// lúc bị drop, tự khoá LẠI cùng một mutex. Bản lỗi (`*guard = Some(new)`) drop giá
    /// trị cũ trong khi `guard` vẫn sống ⇒ `try_lock()` bên dưới trả `Err` và test đỏ.
    /// Bản đã vá nhả khoá trước, nên `try_lock()` thành công.
    #[test]
    fn swap_locked_drops_the_old_value_after_the_lock_is_released() {
        struct ReentrantProbe(Arc<Mutex<Option<ReentrantProbe>>>);

        impl Drop for ReentrantProbe {
            fn drop(&mut self) {
                assert!(
                    self.0.try_lock().is_ok(),
                    "gia tri CU dang bi drop trong khi mutex van con khoa -- AC10 vo hieu"
                );
            }
        }

        let mutex: Arc<Mutex<Option<ReentrantProbe>>> = Arc::new(Mutex::new(None));

        let first = swap_locked(&mutex, ReentrantProbe(Arc::clone(&mutex)));
        assert!(
            first.is_none(),
            "mutex rong luc dau ⇒ khong co gia tri CU nao"
        );

        let second = swap_locked(&mutex, ReentrantProbe(Arc::clone(&mutex)));
        assert!(second.is_some());
        drop(second); // Drop cua ReentrantProbe tu assert ⇒ day la phep kiem that su.

        // 🔴 Lay gia tri CON LAI ra roi tha NGOAI khoa — hai viec trong mot dong.
        //
        // (1) Pha chu trinh `Arc`: gia tri cuoi nam TRONG chinh mutex ma no giu mot `Arc`
        //     toi, nen refcount khong bao gio ve 0 ⇒ `Drop` cua no khong bao gio chay
        //     va bo nho ro o cuoi test. Bat o luot code review 2026-08-06.
        // (2) Cho phep chinh phep kiem chay them mot lan nua: `take()` trong mot khoi rieng
        //     nha `guard` TRUOC, roi `drop(last)` chay `try_lock()` khi mutex da ranh.
        let last = { mutex.lock().unwrap().take() };
        assert!(
            last.is_some(),
            "mutex phai con dung mot gia tri sau ca hai luot swap"
        );
        drop(last);
    }

    // ═════════════════════════════════════════════════════════════════════════════
    // P2 (vòng rà THỨ HAI, 2026-08-27) — `resolve_library_root_from`/
    // `resolve_configured_library_root` KHÔNG có một phép kiểm HÀNH VI nào trước bản vá:
    // cả ba nhánh sống trong `resolve_library_root(app, store)`, đòi `&tauri::AppHandle` mà
    // crate này không có cách dựng giả (không `test-utils`). Tách hai hàm THUẦN để phủ được
    // BA nhánh: giá trị đã cấu hình thắng · `load_global_config` lỗi ⇒ rơi về mặc định ·
    // `store = None` ⇒ rơi về mặc định — cộng ca "override thắng giá trị cấu hình" (nay
    // kiểm được vì không cần `AppHandle`).
    // ═════════════════════════════════════════════════════════════════════════════

    static NEXT_ROOT_DIR: AtomicU64 = AtomicU64::new(0);

    fn root_test_dir(tag: &str) -> std::path::PathBuf {
        let n = NEXT_ROOT_DIR.fetch_add(1, Ordering::Relaxed);
        let dir = std::env::temp_dir().join(format!(
            "auratranslate-resolve-library-root-{}-{}-{}",
            std::process::id(),
            tag,
            n
        ));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap_or_else(|e| panic!("tao {}: {e}", dir.display()));
        dir
    }

    fn root_test_cleanup(dir: &std::path::Path) {
        let _ = std::fs::remove_dir_all(dir);
    }

    /// §Always của story 5.3: móc e2e LUÔN thắng, kể cả khi đã có giá trị cấu hình.
    #[test]
    fn resolve_library_root_from_override_wins_even_when_configured_is_present() {
        let result = super::resolve_library_root_from(
            Some(std::path::PathBuf::from("/override")),
            Some("/da-cau-hinh".to_owned()),
            || panic!("default KHONG duoc goi khi override co mat"),
        );
        assert_eq!(result.unwrap(), std::path::PathBuf::from("/override"));
    }

    #[test]
    fn resolve_library_root_from_uses_the_configured_value_when_override_is_absent() {
        let result = super::resolve_library_root_from(
            None,
            Some("/da-cau-hinh".to_owned()),
            || panic!("default KHONG duoc goi khi da co gia tri cau hinh"),
        );
        assert_eq!(result.unwrap(), std::path::PathBuf::from("/da-cau-hinh"));
    }

    #[test]
    fn resolve_library_root_from_calls_the_default_only_when_both_are_absent() {
        let result =
            super::resolve_library_root_from(None, None, || Ok(std::path::PathBuf::from("/mac-dinh")));
        assert_eq!(result.unwrap(), std::path::PathBuf::from("/mac-dinh"));
    }

    #[test]
    fn resolve_library_root_from_propagates_a_default_error() {
        let result = super::resolve_library_root_from(None, None, || {
            Err(crate::core::store::StoreError::OpenFailed {
                store: crate::core::store::StoreKind::Global,
                detail: "gia lap".to_owned(),
            }
            .into())
        });
        assert!(result.is_err(), "loi tu default phai duoc truyen nguyen ven, khong bi nuot");
    }

    #[test]
    fn resolve_configured_library_root_with_no_store_is_not_configured() {
        assert_eq!(super::resolve_configured_library_root(None), None);
    }

    #[test]
    fn resolve_configured_library_root_with_nothing_saved_is_not_configured() {
        let dir = root_test_dir("fresh");
        let store = crate::core::store::Store::open(crate::core::store::StoreSpec::global(
            dir.join("global.db"),
        ))
        .unwrap_or_else(|e| panic!("mo global.db: {e}"));

        assert_eq!(super::resolve_configured_library_root(Some(&store)), None);

        drop(store);
        root_test_cleanup(&dir);
    }

    #[test]
    fn resolve_configured_library_root_returns_a_saved_value() {
        let dir = root_test_dir("configured");
        let store = crate::core::store::Store::open(crate::core::store::StoreSpec::global(
            dir.join("global.db"),
        ))
        .unwrap_or_else(|e| panic!("mo global.db: {e}"));
        crate::core::scope::save_value(&store, "app_config", "library_root", "/tu-cau-hinh")
            .unwrap_or_else(|e| panic!("ghi cau hinh: {e}"));

        assert_eq!(
            super::resolve_configured_library_root(Some(&store)),
            Some("/tu-cau-hinh".to_owned())
        );

        drop(store);
        root_test_cleanup(&dir);
    }

    /// `ReaderPool::close()` (doc-comment của chính nó): "Sau lời gọi này, `read()` trả
    /// `StoreError::PoolClosed`" — cách TẤT ĐỊNH duy nhất để dựng một `load_global_config`
    /// trượt mà không cần một `global.db` hỏng thật trên đĩa.
    #[test]
    fn resolve_configured_library_root_falls_back_to_not_configured_when_the_read_fails() {
        let dir = root_test_dir("read-fails");
        let store = crate::core::store::Store::open(crate::core::store::StoreSpec::global(
            dir.join("global.db"),
        ))
        .unwrap_or_else(|e| panic!("mo global.db: {e}"));
        store.close();

        assert_eq!(
            super::resolve_configured_library_root(Some(&store)),
            None,
            "doc cau hinh truot khong duoc lam ung dung nga -- phai roi ve 'chua cau hinh'"
        );

        drop(store);
        root_test_cleanup(&dir);
    }

    /// R2 (vòng rà đối kháng 3, lớp 3) — vị từ Rust phải khớp ĐÚNG những `CHECK` mà
    /// `ASSET_DDL` khai: một hàng hợp lệ (fixture thật) qua được, một hàng vi phạm MỖI cột
    /// một lượt phải bị bắt.
    fn well_formed_saved_asset() -> SavedAsset {
        SavedAsset {
            chapter_index: 0,
            block_index: 0,
            anchor_after_segment_ord: 1,
            file_name: "abc123.jpg".to_owned(),
            source_url: Some("https://example.test/a.jpg".to_owned()),
            byte_len: 10,
            content_type: "image/jpeg".to_owned(),
        }
    }

    #[test]
    fn a_well_formed_saved_asset_satisfies_every_check_constraint() {
        assert!(saved_asset_satisfies_asset_check_constraints(&well_formed_saved_asset()));
    }

    /// **THÊM 2026-09-09 (Story 6.12)** — `source_url: None` (ảnh `.docx` nhúng) phải THOẢ
    /// mãn CHECK, đúng khuôn `ASSET_DDL` (`source_url IS NULL OR trim(...) <> ''`) — ca ÂM
    /// đi kèm với "source_url rỗng phải bị bắt" ngay dưới, chứng minh vị từ phân biệt được
    /// `None` (hợp lệ) khỏi `Some("")` (không hợp lệ).
    #[test]
    fn a_saved_asset_with_no_source_url_still_satisfies_the_check_constraint() {
        let mut docx_asset = well_formed_saved_asset();
        docx_asset.source_url = None;
        assert!(
            saved_asset_satisfies_asset_check_constraints(&docx_asset),
            "source_url: None (anh .docx) phai duoc CHAP NHAN, dung nghia NULL cua ASSET_DDL"
        );
    }

    #[test]
    fn saved_asset_check_constraints_catch_a_violation_on_each_field_one_at_a_time() {
        let mut bad = well_formed_saved_asset();
        bad.file_name = "   ".to_owned();
        assert!(!saved_asset_satisfies_asset_check_constraints(&bad), "file_name toan khoang trang phai bi bat");

        let mut bad = well_formed_saved_asset();
        bad.source_url = Some(String::new());
        assert!(!saved_asset_satisfies_asset_check_constraints(&bad), "source_url la Some(\"\") phai bi bat");

        let mut bad = well_formed_saved_asset();
        bad.content_type = "\u{3000}".to_owned();
        assert!(!saved_asset_satisfies_asset_check_constraints(&bad), "content_type toan U+3000 phai bi bat");

        let mut bad = well_formed_saved_asset();
        bad.byte_len = -1;
        assert!(!saved_asset_satisfies_asset_check_constraints(&bad), "byte_len am phai bi bat");

        let mut bad = well_formed_saved_asset();
        bad.anchor_after_segment_ord = -1;
        assert!(!saved_asset_satisfies_asset_check_constraints(&bad), "anchor am phai bi bat");
    }

    /// R6 (vòng rà đối kháng 3, lớp 3) — `chapter_index` trong dải `0..chapters_len` phải
    /// qua được vị từ; NGOÀI dải (kể cả ĐÚNG BẰNG `chapters_len`, biên trên không hợp lệ)
    /// phải bị bắt.
    #[test]
    fn saved_asset_chapter_index_in_range_accepts_valid_indices_and_rejects_out_of_range_ones() {
        let mut a = well_formed_saved_asset();
        a.chapter_index = 0;
        assert!(saved_asset_chapter_index_is_in_range(&a, 3), "chapter_index 0 trong dai 0..3 phai qua");

        let mut b = well_formed_saved_asset();
        b.chapter_index = 2;
        assert!(saved_asset_chapter_index_is_in_range(&b, 3), "chapter_index 2 trong dai 0..3 phai qua");

        let mut c = well_formed_saved_asset();
        c.chapter_index = 3;
        assert!(
            !saved_asset_chapter_index_is_in_range(&c, 3),
            "chapter_index == chapters_len (bien tren, KHONG hop le) phai bi bat"
        );

        let mut d = well_formed_saved_asset();
        d.chapter_index = 99;
        assert!(!saved_asset_chapter_index_is_in_range(&d, 3), "chapter_index vuot xa dai phai bi bat");
    }

}

/// Nhiều vỏ `#[tauri::command]`. **Không một quy tắc nào sống ở đây.**
pub mod wire {
    use super::{
        ImportEncodingPreview, IpcError, OpenWork, OpenWorkState, PendingImportSourceState,
        Tier2BlockOverridesState, no_pending_import_source, replace_open_work,
        resolve_library_root, spawn_import_scan,
    };
    use crate::core::cleanup::CleanupRule;
    use crate::core::i18n::MessageKey;
    use crate::core::library::WorkMeta;
    use crate::core::library::indexer::Indexer;
    use crate::core::scope::ScopeResolver;
    use crate::core::store::Store;
    use crate::core::webimport;

    /// Luật làm sạch ĐÃ PHÂN GIẢI (hai tầng hợp nhất qua `ScopeResolver::apply_merge`) cho
    /// lượt gọi HIỆN TẠI — Story 6.5. Đọc CẢ HAI tầng MỖI LƯỢT gọi (không cache): `global.db`
    /// luôn có; `project.db` chỉ có khi một Tác phẩm đang mở (`OpenWorkState`).
    ///
    /// Lỗi (kho vắng mặt, `ScopeResolver::apply_merge` từ chối) rơi về **0 luật** kèm chẩn
    /// đoán — cùng khuôn `resolve_configured_library_root`: luật làm sạch là một tiện ích
    /// bổ trợ, một sự cố ở đây không được phép làm cả màn xem trước sập.
    fn resolve_cleanup_rules(app: &tauri::AppHandle) -> Vec<CleanupRule> {
        use tauri::Manager as _;

        let global_state = app.try_state::<Store>();
        let Some(global) = global_state.as_deref() else {
            eprintln!("cleanup[rules] global.db chua duoc quan ly, roi ve 0 luat");
            return Vec::new();
        };

        let Some(work_state) = app.try_state::<OpenWorkState>() else {
            return resolve_cleanup_rules_against(&ScopeResolver::global_only(), global, None);
        };
        let guard = work_state.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
        match guard.as_ref() {
            Some(open) => resolve_cleanup_rules_against(&open.scope, global, Some(&open.store)),
            None => resolve_cleanup_rules_against(&ScopeResolver::global_only(), global, None),
        }
    }

    fn resolve_cleanup_rules_against(
        resolver: &ScopeResolver,
        global: &Store,
        work: Option<&Store>,
    ) -> Vec<CleanupRule> {
        match crate::core::cleanup::resolve_two_tiers(resolver, global, work) {
            Ok(rules) => rules,
            Err(err) => {
                eprintln!("cleanup[rules] phan giai that bai, roi ve 0 luat: {err}");
                Vec::new()
            }
        }
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Story 6.9 — trạng thái giữ/loại khối tầng 2, `Tier2BlockOverridesState`.
    // ─────────────────────────────────────────────────────────────────────────

    /// Đọc bản sao HIỆN HÀNH của `Tier2BlockOverridesState` — best-effort RỖNG khi state chưa
    /// được `.manage(...)` (lỗi lắp dây ở `lib.rs`), cùng triết lý `resolve_cleanup_rules`:
    /// một tiện ích bổ trợ trượt không được làm sập cả lượt IPC chính.
    fn resolve_tier2_block_overrides(app: &tauri::AppHandle) -> Vec<Option<bool>> {
        use tauri::Manager as _;

        match app.try_state::<Tier2BlockOverridesState>() {
            Some(state) => {
                let guard = state.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
                guard.clone()
            }
            None => {
                eprintln!("webimport[tier2] Tier2BlockOverridesState chua duoc quan ly, roi ve 0 override");
                Vec::new()
            }
        }
    }

    /// Dọn `Tier2BlockOverridesState` — best-effort (state vắng mặt không phải một lỗi để mà
    /// ném, cùng lý do [`resolve_tier2_block_overrides`]).
    fn reset_tier2_block_overrides(app: &tauri::AppHandle) {
        use tauri::Manager as _;

        if let Some(state) = app.try_state::<Tier2BlockOverridesState>() {
            super::reset_block_overrides(&state);
        }
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Story 6.15 — trạng thái xuất xứ NGƯỜI DÙNG gõ đè, theo Chương —
    // `ChapterOriginOverridesState`. Cùng khuôn hai hàm ngay trên.
    // ─────────────────────────────────────────────────────────────────────────

    /// Đọc bản sao HIỆN HÀNH của `ChapterOriginOverridesState` — best-effort RỖNG khi state
    /// chưa được `.manage(...)`.
    fn resolve_chapter_origin_overrides(app: &tauri::AppHandle) -> Vec<Option<super::ChapterOriginOverride>> {
        use tauri::Manager as _;

        match app.try_state::<super::ChapterOriginOverridesState>() {
            Some(state) => {
                let guard = state.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
                guard.clone()
            }
            None => {
                eprintln!(
                    "webimport[origin] ChapterOriginOverridesState chua duoc quan ly, roi ve 0 override"
                );
                Vec::new()
            }
        }
    }

    /// Dọn `ChapterOriginOverridesState` — best-effort.
    fn reset_chapter_origin_overrides(app: &tauri::AppHandle) {
        use tauri::Manager as _;

        if let Some(state) = app.try_state::<super::ChapterOriginOverridesState>() {
            super::reset_chapter_origin_overrides(&state);
        }
    }

    /// Vỏ IPC — ghi một lượt sửa tay xuất xứ (một hoặc nhiều trong bốn ô) cho Chương thứ
    /// `chapter_index` (0-based, vị trí trong danh sách Chương của lượt nhập ĐANG XEM TRƯỚC)
    /// vào `ChapterOriginOverridesState`. **Không một quy tắc nào sống ở đây** — đọc
    /// [`super::set_chapter_origin_override`].
    ///
    /// ⚠️ Bốn tham số Option: `None` ⇔ ô đó CHƯA bị chạm (dùng nguyên giá trị máy);
    /// `Some(v)` ⇔ đã chạm, `v` rỗng sau khi cắt ⇒ cột ghi `NULL` lúc xác nhận. Frontend GỌI
    /// LẠI lệnh này với TOÀN BỘ bốn trường mỗi lần một ô đổi (form "sửa tại chỗ" giữ trạng
    /// thái bốn ô trong một draft duy nhất — xem `src/ChapterOrigin.vue`).
    #[tauri::command]
    pub fn set_chapter_origin_override(
        app: tauri::AppHandle,
        chapter_index: usize,
        author: Option<String>,
        site_name: Option<String>,
        url: Option<String>,
        published_at: Option<String>,
    ) {
        use tauri::Manager as _;

        let Some(state) = app.try_state::<super::ChapterOriginOverridesState>() else {
            eprintln!("webimport[origin] ChapterOriginOverridesState chua duoc quan ly, bo qua luot sua");
            return;
        };
        super::set_chapter_origin_override(
            &state,
            chapter_index,
            super::ChapterOriginOverride { author, site_name, url, published_at },
        );
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Story 6.8 (NFR19, AD-41) — nhật ký domain: nối bản ghi + đọc số domain phân biệt.
    // ─────────────────────────────────────────────────────────────────────────
    //
    // 🔴 Cả hai hàm dưới đây coi `webimport::DomainLogState` VẮNG MẶT là một lỗi LẮP DÂY
    // (`lib.rs::open_work_slot` không `.manage` nó), không phải một lý do để chặn thao tác
    // sản phẩm chính (tải/tải lại một URL) — cùng triết lý `resolve_cleanup_rules` ngay
    // trên: một tiện ích bổ trợ (ở đây là TÍNH MINH BẠCH, không phải đường ghi chính) trượt
    // thì rơi về giá trị AN TOÀN kèm chẩn đoán, không làm sập cả lượt IPC.

    /// Nối `entries` vào nhật ký của phiên chạy — best-effort, xem lý do ở trên.
    fn append_domain_log(app: &tauri::AppHandle, entries: Vec<webimport::DomainLogEntry>) {
        use tauri::Manager as _;

        if entries.is_empty() {
            return;
        }
        match app.try_state::<webimport::DomainLogState>() {
            Some(state) => webimport::append_domain_log_entries(&state, entries),
            None => eprintln!(
                "webimport[domain_log] DomainLogState chua duoc quan ly - bo qua {} ban ghi",
                entries.len()
            ),
        }
    }

    /// Số domain PHÂN BIỆT trong nhật ký của cả phiên chạy — best-effort, xem lý do ở trên.
    fn domain_log_domain_count(app: &tauri::AppHandle) -> usize {
        use tauri::Manager as _;

        match app.try_state::<webimport::DomainLogState>() {
            Some(state) => webimport::distinct_domain_count(&state),
            None => {
                eprintln!("webimport[domain_log] DomainLogState chua duoc quan ly - dem tra ve 0");
                0
            }
        }
    }

    /// Thứ hai lệnh trả về — [`WorkMeta`] **cộng đường dẫn thư mục trên đĩa**.
    ///
    /// ─────────────────────────────────────────────────────────────────────────────
    /// 🔴 VÌ SAO `folder` PHẢI ĐI RA — AC6 KHÔNG GIAO ĐƯỢC NẾU THIẾU NÓ
    /// ─────────────────────────────────────────────────────────────────────────────
    /// AC6 hứa với người dùng *"copy thư mục là đủ để sao lưu"*. Một lời hứa về **một
    /// thư mục cụ thể** mà không nói thư mục đó ở đâu thì không thực hiện được.
    /// Và tên thư mục **không** suy ra được từ `meta.name`: `sanitize_name` thay ký tự
    /// cấm (`Tập 1: Khởi đầu` → `Tập 1_ Khởi đầu`), và trùng tên thì thêm hậu tố
    /// ` (2)` — nên chỉ Rust mới biết tên thật. Code review 2026-08-06.
    ///
    /// ⚠️ `#[serde(rename_all = ...)]` KHÔNG đặt — cùng luật với mọi struct qua biên.
    #[derive(Debug, Clone, serde::Serialize)]
    pub struct CreatedWork {
        /// Metadata vừa ghi xuống `meta.json`.
        pub meta: WorkMeta,
        /// Đường dẫn **tuyệt đối** tới `<Tên>.atproj/` trên máy này.
        ///
        /// ⚠️ Đây là một giá trị **qua IPC**, không phải một giá trị **ghi xuống đĩa** —
        /// AC5 cấm đường dẫn tuyệt đối bên trong `meta.json`/`project.db`, không cấm
        /// nói cho người dùng biết Tác phẩm của họ nằm ở đâu.
        pub folder: String,
        /// **THÊM 2026-09-08 (Story 6.11, FR127)** — số HÀNG `asset` đã chèn. `0` cho mọi
        /// lượt tạo KHÔNG đi qua đường URL (dán văn bản, tệp) — không `#[serde(rename_all
        /// = "camelCase")]` (cùng luật mọi struct qua dây, `project_contract.rs` khoá dây).
        ///
        /// 🔵 **SỬA 2026-09-09 (vòng rà đối kháng 3, mục T1).** Câu cũ khai "số ảnh đã tải &
        /// ghi thành công" — SAI trong ca DEDUP (D2, `commands::project::OpenWork::images_saved`
        /// đã sửa CÙNG câu này ở bản `OpenWork`, nhưng bỏ sót đúng bản SONG SINH này mà
        /// frontend thật sự đọc qua dây): cùng một URL ảnh dùng lại ở hai Chương chỉ tải/ghi
        /// MỘT tệp nhưng sinh HAI hàng `asset` ⇒ `images_saved == 2` trong khi số TỆP thật là
        /// 1. Trường này đếm HÀNG, không đếm TỆP.
        pub images_saved: u32,
        /// **THÊM 2026-09-08 (Story 6.11, FR127)** — số ảnh GIỮ nhưng không có hàng `asset`.
        /// Bề mặt HIỂN THỊ con số này là nợ có chủ (`deferred-work.md`) — trường có mặt ở đây
        /// để KHÔNG bị bịa lại một lần nữa khi bề mặt đó được dựng.
        pub images_failed: u32,
    }

    impl CreatedWork {
        /// Gói một [`OpenWork`] thành thứ đi qua dây được — `Store` không `Serialize`.
        fn from_open(open: &OpenWork) -> Self {
            Self {
                meta: open.meta.clone(),
                folder: open.dir.display().to_string(),
                images_saved: open.images_saved,
                images_failed: open.images_failed,
            }
        }
    }

    /// **THÊM Story 5.7.** Kết quả của [`open_work`] (vỏ IPC) — hình dạng [`CreatedWork`]
    /// MỞ RỘNG thêm `chapter_id`: mở một Tác phẩm đã có luôn kèm Chương nó sẽ mở (Chương
    /// đầu theo `(ord, id)`, xem §Design Notes "Vì sao KHÔNG có Chương mở gần nhất" của
    /// `5-7-danh-sach-chuong-va-mo-chuong-vao-workspace.md`), nên trả cả hai trong MỘT lượt
    /// IPC thay vì bắt webview gọi thêm `read_open_chapter` ngay sau khi mở.
    ///
    /// ⚠️ `#[serde(rename_all = ...)]` KHÔNG đặt — cùng luật với mọi struct qua biên.
    #[derive(Debug, Clone, serde::Serialize)]
    pub struct OpenedWork {
        /// Metadata của Tác phẩm vừa mở lại.
        pub meta: WorkMeta,
        /// Đường dẫn **tuyệt đối** tới `<Tên>.atproj/` trên máy này — cùng lý do
        /// [`CreatedWork::folder`].
        pub folder: String,
        /// `chapter.id` của Chương đầu theo `(ord, id)` — đúng nguồn sự thật
        /// [`OpenWork::chapter_id`], không suy lại ở tầng vỏ.
        pub chapter_id: i64,
    }

    impl OpenedWork {
        /// Gói một [`OpenWork`] thành thứ đi qua dây được — cùng khuôn `CreatedWork::from_open`.
        fn from_open(open: &OpenWork) -> Self {
            Self {
                meta: open.meta.clone(),
                folder: open.dir.display().to_string(),
                chapter_id: open.chapter_id,
            }
        }
    }

    /// `Indexer` chưa được quản lý (mở `library-index.db` thất bại lúc khởi động) — tái dùng
    /// [`MessageKey::StoreOpenFailed`] thay vì đúc một khoá thứ ba, đúng khuôn
    /// `commands::library::indexer_is_missing` (danh mục đóng của story này chỉ thêm ĐÚNG BA
    /// khoá: `WorkMetaTooNew`/`WorkOpenFailed`/`LibraryWorkNotIndexed`).
    fn indexer_is_missing() -> IpcError {
        let mut params = std::collections::BTreeMap::new();
        params.insert("store".to_owned(), "library_index".to_owned());
        IpcError::new("library.indexer_missing", MessageKey::StoreOpenFailed, params, false)
    }

    /// Đưa Tác phẩm/trạng thái vòng đời vừa ghi vào `library-index.db` — Story 5.2, AD-8
    /// "`.atproj` ghi trước, chỉ mục ghi sau".
    ///
    /// ─────────────────────────────────────────────────────────────────────────────
    /// 🔴 VÌ SAO Ở LỚP VỎ, KHÔNG BÊN TRONG MỘT HÀM THUẦN
    /// ─────────────────────────────────────────────────────────────────────────────
    /// `Indexer` sống trong state của Tauri — chỉ có ở lớp vỏ, không trong các hàm thuần của
    /// `super::create_work`/`commands::lifecycle` (nhận `&Path`/`Option<&OpenWork>`, không
    /// `AppHandle`, để `tests::` gọi được không cần webview, cùng khuôn `src-tauri/AGENTS.md`).
    /// Gọi tới đây **chỉ khi** hàm thuần tương ứng đã trả `Ok` — tức `write_atomic` đã chạy
    /// xong và `.atproj` đã đầy đủ trên đĩa (§Boundaries "Thứ tự ghi") — nên đặt lời gọi này
    /// ở lớp vỏ, ngay sau khi hàm thuần trả về, giữ NGUYÊN thứ tự mà đặt nó bên trong hàm
    /// thuần sẽ cho ra.
    ///
    /// 🔴 Lỗi chỉ mục **KHÔNG** được làm hỏng lượt ghi đã commit vào `.atproj` — chẩn đoán
    /// rồi ĐI TIẾP, trả `Ok` cho người dùng như bình thường. Đây chính là "chỉ mục lỗi không
    /// làm hỏng `.atproj`" viết bằng mã.
    ///
    /// 🔵 **SỬA (2026-08-27, Story 5.4) — đổi tên từ `reindex_after_create_work`, và mệnh đề
    /// "chỗ gọi thứ hai" đã HẾT ĐÚNG.** Tên cũ chỉ đúng khi có ĐÚNG hai chỗ gọi
    /// (`lib.rs::open_library_index` lúc khởi động, và chính hàm này sau khi tạo Tác phẩm).
    /// Story này thêm chỗ gọi THỨ BA: `commands::lifecycle::wire` gọi lại đúng hàm này sau
    /// MỖI lượt ghi trạng thái Chương/ghi đè Tác phẩm (§Always: *"vỏ IPC gọi lại đúng hàm
    /// reindex đã có, không tự UPDATE library_work"*) — tên mới nói đúng vai trò CHUNG của
    /// nó (đưa mọi thay đổi vào chỉ mục), không còn khoá vào MỘT sự kiện cụ thể.
    pub(crate) fn reindex_library(app: &tauri::AppHandle, root: &std::path::Path) {
        use tauri::Manager as _;

        let Some(indexer) = app.try_state::<crate::core::library::indexer::Indexer>() else {
            eprintln!("library[index] Indexer chua duoc quan ly -- bo qua luot dua vao chi muc");
            return;
        };
        // Phan quyet Ice #1 (2026-08-27) -- co mo coi song o `library_orphan` (global.db),
        // nen `rebuild` can mot `&Store` toan cuc. `open_global_store` chay TRUOC setup nay
        // (xem `resolve_library_root` ngay tren chinh ham goi), nen Store da (co the) duoc
        // `app.manage()`.
        let global = app.try_state::<Store>();
        match indexer.rebuild(root, global.as_deref()) {
            // Vòng rà ba lớp, P7 — `RebuildOutcome` không còn bị vứt: xung đột `work_id`/
            // entry bị bỏ qua phải có ÍT NHẤT một dòng chẩn đoán, cùng khuôn `lib.rs::open_library_index`.
            Ok(outcome) => outcome.log_if_notable("reindex"),
            Err(err) => {
                eprintln!("library[index] rebuild that bai: {err}");
            }
        }
    }

    /// Vỏ IPC của [`super::create_work_from_text`].
    ///
    /// ⚠️ Trả về [`CreatedWork`] — vỏ **không** trả `OpenWork` ra ngoài (nó mang `Store`,
    /// không `Serialize`); quản lý `OpenWork` trong state qua [`replace_open_work`].
    ///
    /// 🔴 **VÌ SAO VỎ NÀY Ở LẠI, DÙ 0 CHỖ GỌI SẢN PHẨM (vòng rà đối kháng 2, mục 5)** —
    /// `src/**` không còn chỗ nào gọi tới đây kể từ Story 6.3 (nộp form đi qua màn xem
    /// trước bảng mã, xem `wire::confirm_import_with_encoding`), và adapter TS
    /// `createWorkFromText` (`src/config/project.ts`) đã bị XOÁ vì lý do đó. Vỏ RUST này
    /// KHÔNG bị xoá theo: 15+ tệp `e2e/specs/**` gọi thẳng
    /// `internals.invoke('create_work_from_text', {...})` để dựng fixture NHANH, cố ý đi
    /// đường IPC trực tiếp — bỏ qua UI, bỏ qua màn xem trước, đúng ý đồ của một bàn fixture
    /// (xem `e2e/support/workspace.mjs`). Xoá vỏ này phá TOÀN BỘ hạ tầng đó.
    ///
    /// Bất biến "không byte nào xuống đĩa trước khi xác nhận" (§Always spec 6.3) vì thế là
    /// một mệnh đề về ĐƯỜNG SẢN PHẨM (`src/**`), không phải về TOÀN BỘ bề mặt IPC — cổng
    /// canh nó là `tests/frontend/noProductPathBypassesEncodingPreview.test.ts` (quét
    /// `src/**`, không quét `e2e/**` — `e2e/**` được PHÉP đi tắt có chủ ý).
    #[tauri::command]
    pub fn create_work_from_text(
        app: tauri::AppHandle,
        name: String,
        source_lang: String,
        genre: String,
        text: String,
    ) -> Result<CreatedWork, IpcError> {
        use tauri::Manager as _;
        // 🔴 Story 5.3 — resolve_library_root, KHÔNG default_library_root: một Tác phẩm
        // mới phải sinh ra trong đúng thư mục người dùng đã chọn, nếu không AC5 của story
        // mở ra một chỗ rỗng im lặng thứ hai (xem doc-comment của resolve_library_root).
        let root = resolve_library_root(&app, app.try_state::<Store>().as_deref())?;
        let opened = super::create_work_from_text(&root, &name, &source_lang, &genre, text)?;
        let created = CreatedWork::from_open(&opened);
        reindex_library(&app, &root);
        // 🔴 Chốt `work_id`/`chapter_id`/`source_lang` TRƯỚC khi `opened` bị `move` vào
        // `replace_open_work` — Story 3.5, spawn lượt quét SAU khi Tác phẩm đã vào state.
        let work_id = opened.meta.work_id.clone();
        let chapter_id = opened.chapter_id;
        let scan_source_lang = source_lang.clone();
        replace_open_work(&app, opened);
        // Import đã commit và `OpenWorkState` đã thay xong. Spawn lỗi chỉ làm mất lượt
        // quét nền; biến một thành công đã ghi xuống đĩa thành lỗi IPC sẽ khiến người
        // dùng thử lại và tạo một Tác phẩm trùng.
        Ok(super::keep_committed_import_when_scan_spawn_fails(
            created,
            || spawn_import_scan(app, work_id, chapter_id, scan_source_lang),
        ))
    }

    /// Vỏ IPC của [`super::create_work_from_file`]. Cùng lý do "ở lại dù 0 chỗ gọi sản
    /// phẩm" với `create_work_from_text` ngay trên — đọc doc-comment ở đó.
    #[tauri::command]
    pub fn create_work_from_file(
        app: tauri::AppHandle,
        name: String,
        source_lang: String,
        genre: String,
        path: String,
    ) -> Result<CreatedWork, IpcError> {
        use tauri::Manager as _;
        // 🔴 Cùng lý do nhánh `create_work_from_text` ngay trên.
        let root = resolve_library_root(&app, app.try_state::<Store>().as_deref())?;
        let opened = super::create_work_from_file(
            &root,
            &name,
            &source_lang,
            &genre,
            std::path::Path::new(&path),
        )?;
        let created = CreatedWork::from_open(&opened);
        reindex_library(&app, &root);
        // 🔴 Cùng lý do nhánh `create_work_from_text` ngay trên — chốt trước khi `move`.
        let work_id = opened.meta.work_id.clone();
        let chapter_id = opened.chapter_id;
        let scan_source_lang = source_lang.clone();
        replace_open_work(&app, opened);
        Ok(super::keep_committed_import_when_scan_spawn_fails(
            created,
            || spawn_import_scan(app, work_id, chapter_id, scan_source_lang),
        ))
    }

    /// Vỏ IPC — nhánh DÁN VĂN BẢN của màn xem trước bảng mã (Story 6.3, FR126). Văn bản dán
    /// tay đã LÀ `String` từ lúc rời webview — không byte thô nào để đọc lại, nên đây KHÔNG
    /// thể trượt bằng chính lượt đọc (`Result` vẫn cần: xem lỗi "state chưa quản lý" dưới
    /// đây).
    /// **Không một quy tắc nào sống ở đây** — đọc [`super::preview_import_encoding`] và
    /// [`super::stash_pending_import_source`].
    ///
    /// 🔵 **THÊM 2026-09-04 (Story 6.4) — tham số `source_lang`.** KHÔNG phải một command
    /// mới, KHÔNG một lượt đọc thêm: `sourceLang` đã có sẵn ở form phía frontend TRƯỚC khi
    /// lệnh này được gọi (`src/importPreviewState.ts::openWith` đã nhận nó làm tham số từ
    /// trước Story 6.3) — chỉ là trước story này chưa có lý do để gửi nó xuống.
    ///
    /// # Lỗi
    /// - [`PendingImportSourceState`] chưa được `app.manage(...)` (lỗi cấu hình `setup()`) ⇒
    ///   `import.no_pending_source`, TƯỜNG MINH. 🔴 SỬA (vòng rà đối kháng 2, mục 1) — bản
    ///   trước `eprintln!` rồi vẫn trả `Ok(preview)`: người dùng thấy màn xem trước chạy
    ///   BÌNH THƯỜNG, chọn một ứng viên, bấm "Xác nhận" — rồi mọi lượt xác nhận đều trượt
    ///   với `no_pending_source`, không một manh mối nào giải thích vì sao. Đây là một lượt
    ///   XUỐNG CẤP IM LẶNG đúng lớp lỗi mà AGENTS.md gọi tên là trung tâm của dự án.
    /// 🔵 **THÊM 2026-09-05 (Story 6.6) — tham số `chapter_pattern`.** Mẫu phân tách Chương
    /// là tham số MỖI LƯỢT NHẬP (§Always spec 6.6) — KHÔNG lưu ở đâu cả giữa hai lượt gọi,
    /// frontend gửi lại nó ở MỌI lượt xem trước/xác nhận (`src/importPreviewState.ts`).
    #[tauri::command]
    pub fn preview_import_encoding_from_text(
        app: tauri::AppHandle,
        text: String,
        source_lang: String,
        chapter_pattern: Option<super::ChapterPatternWire>,
    ) -> Result<ImportEncodingPreview, IpcError> {
        use tauri::Manager as _;
        let Some(state) = app.try_state::<PendingImportSourceState>() else {
            return Err(no_pending_import_source());
        };
        let pattern = super::resolve_chapter_pattern(chapter_pattern)?;
        let cleanup_rules = resolve_cleanup_rules(&app);
        let shape = super::import_text(text);
        // Story 6.9 — đường dán văn bản KHÔNG BAO GIỜ bóc nội dung chính (`extract_main_content
        // == false`), nên `&[]` không mất gì; dọn `Tier2BlockOverridesState` cho nhất quán
        // VÒNG ĐỜI của một lượt xem trước MỚI (xem doc-comment kiểu đó).
        reset_tier2_block_overrides(&app);
        // 🔴 SỬA 2026-09-10 (Story 6.15, lượt rà) — dọn `ChapterOriginOverridesState` CÙNG lượt.
        // Thiếu dòng này, một lượt gõ đè xuất xứ ở màn URL bị HUỶ rồi mở lại bằng dán tay/tệp
        // vẫn ĐỌC ĐƯỢC override CŨ lúc `confirm_import_with_encoding` (state đó chỉ được đọc,
        // không được RESET, ở lượt HUỶ — xem doc-comment `ChapterOriginOverridesState` mục
        // "Reset khi nào") — bốn ô của một Chương dán tay/tệp thật sự sẽ mang xuất xứ của một
        // lượt nhập URL đã bỏ, đúng lớp lỗi "rỗng/hỏng ngầm" mà AGENTS.md gọi tên là trung tâm.
        reset_chapter_origin_overrides(&app);
        let preview = super::preview_import_encoding(
            &shape,
            &source_lang,
            &cleanup_rules,
            pattern.as_ref(),
            &[],
            // Đường tệp/dán tay — 0 mục URL để mà hỏng (§Always spec 6.10: "đường tệp/dán
            // tay truyền 0").
            0,
            // Đường tệp/dán tay không bao giờ bóc xuất xứ (`extract_main_content == false`)
            // — Story 6.15, cùng lý lẽ `&[]` của `block_overrides` ngay trên.
            &[],
        );
        // Văn bản dán tay không bao giờ có một `DocxSidecar` (xem doc-comment kiểu đó).
        super::stash_pending_import_source(&state, shape, None);
        Ok(preview)
    }

    /// Vỏ IPC — nhánh TỆP của màn xem trước bảng mã (Story 6.3, FR126). [`import_file`] đọc
    /// byte thô ĐÚNG MỘT LẦN ở đây; `confirm_import_with_encoding` CLONE từ
    /// [`PendingImportSourceState`], không đọc lại đĩa.
    ///
    /// # Lỗi
    /// - [`PendingImportSourceState`] chưa được quản lý ⇒ `import.no_pending_source`, cùng lý
    ///   do nhánh DÁN VĂN BẢN ngay trên.
    /// 🔵 **THÊM 2026-09-05 (Story 6.6) — tham số `chapter_pattern`**, cùng lý do nhánh DÁN
    /// VĂN BẢN ngay trên.
    #[tauri::command]
    pub fn preview_import_encoding_from_file(
        app: tauri::AppHandle,
        path: String,
        source_lang: String,
        chapter_pattern: Option<super::ChapterPatternWire>,
    ) -> Result<ImportEncodingPreview, IpcError> {
        use tauri::Manager as _;
        let Some(state) = app.try_state::<PendingImportSourceState>() else {
            return Err(no_pending_import_source());
        };
        let pattern = super::resolve_chapter_pattern(chapter_pattern)?;
        let cleanup_rules = resolve_cleanup_rules(&app);
        let (shape, docx_sidecar) = super::import_file(std::path::Path::new(&path))?;
        // Story 6.9 — cùng lý do nhánh DÁN VĂN BẢN ở trên: đường tệp KHÔNG BAO GIỜ bóc nội
        // dung chính (kể cả `.docx` — Story 6.12: nó không đi qua `dom_smoothie`).
        reset_tier2_block_overrides(&app);
        // 🔴 SỬA 2026-09-10 (Story 6.15, lượt rà) — cùng lý do nhánh DÁN VĂN BẢN ở trên.
        reset_chapter_origin_overrides(&app);
        let preview = super::preview_import_encoding(
            &shape,
            &source_lang,
            &cleanup_rules,
            pattern.as_ref(),
            &[],
            // Đường tệp/dán tay — 0 mục URL để mà hỏng (§Always spec 6.10: "đường tệp/dán
            // tay truyền 0").
            0,
            // Đường tệp/dán tay không bao giờ bóc xuất xứ (`extract_main_content == false`)
            // — Story 6.15, cùng lý lẽ `&[]` của `block_overrides` ngay trên.
            &[],
        );
        super::stash_pending_import_source(&state, shape, docx_sidecar);
        Ok(preview)
    }

    /// Vỏ IPC — xác nhận lượt nhập với bảng mã đã chọn (Story 6.3, FR126). **Không một quy
    /// tắc nào sống ở đây** — lõi là [`super::confirm_import_with_encoding`] (hàm thuần,
    /// điểm gọi [`super::create_work`] DUY NHẤT của đường CÓ xem trước bảng mã — không một
    /// chỗ gọi `run_import` thứ hai, §Always spec 6.3); vỏ này chỉ phân giải thư mục gốc,
    /// gói kết quả thành [`CreatedWork`], và nối tiếp lượt tái lập chỉ mục + quét Glossary
    /// đúng khuôn `create_work_from_text`/`create_work_from_file`.
    ///
    /// # Lỗi
    /// - `encoding` không giải ngược được thành một bảng mã ⇒ `import.unrecognized_encoding`,
    ///   TƯỜNG MINH, không âm thầm rơi về UTF-8 (§Design Notes spec 6.3);
    /// - không có lượt xem trước nào đang treo ⇒ `import.no_pending_source`;
    /// - byte không giải mã được với CHÍNH bảng mã đã chọn ⇒ `import.undecodable_bytes`, nêu
    ///   đích danh bảng mã đó — ô đang chờ GIỮ NGUYÊN, chọn một ứng viên khác rồi xác nhận
    ///   lại không đòi đọc nguồn lần hai.
    /// 🔵 **THÊM 2026-09-05 (Story 6.6) — tham số `chapter_pattern`.** Xem trước và xác nhận
    /// phải trùng từng byte (§Always spec 6.6) — frontend gửi lại CÙNG mẫu đã dùng ở lượt
    /// xem trước gần nhất, không một cơ chế "nhớ mẫu" nào ở tầng Rust.
    #[tauri::command]
    pub fn confirm_import_with_encoding(
        app: tauri::AppHandle,
        name: String,
        source_lang: String,
        genre: String,
        encoding: String,
        chapter_pattern: Option<super::ChapterPatternWire>,
    ) -> Result<CreatedWork, IpcError> {
        use tauri::Manager as _;

        let Some(pending_state) = app.try_state::<PendingImportSourceState>() else {
            return Err(no_pending_import_source());
        };
        let pattern = super::resolve_chapter_pattern(chapter_pattern)?;
        // 🔴 Nạp luật NGAY LÚC XÁC NHẬN, không tái dùng bộ đã nạp lúc xem trước — luật có
        // thể đã đổi giữa hai nhịp qua một lượt bật/tắt/soạn khác (§Always spec 6.5).
        let cleanup_rules = resolve_cleanup_rules(&app);
        // 🔴 **THÊM 2026-09-07 (Story 6.9) — đúng cái vòng rà 1 đã hụt.** Đọc
        // `Tier2BlockOverridesState` NGAY LÚC XÁC NHẬN (cùng kỷ luật `cleanup_rules` ở trên),
        // truyền NGUYÊN VẸN xuống `create_work` — state đó chỉ RESET ở dưới, SAU KHI `?` đã
        // xác nhận thành công (đường lỗi giữ nguyên override để người dùng thử lại một ứng
        // viên bảng mã khác mà không mất lượt sửa tay vừa làm).
        let block_overrides = resolve_tier2_block_overrides(&app);
        // 🔴 **THÊM 2026-09-10 (Story 6.15)** — cùng kỷ luật `block_overrides` ngay trên.
        let origin_overrides = resolve_chapter_origin_overrides(&app);
        let root = resolve_library_root(&app, app.try_state::<Store>().as_deref())?;
        // 🔴 **THÊM 2026-09-08 (Story 6.11, mục B1 vòng rà đối kháng 3 lớp)** — nhật ký domain
        // của PHA ẢNH nay được PUSH THẲNG vào state THẬT của phiên chạy TỪ BÊN TRONG
        // `create_work` (mỗi lời gọi `fetch` một lần, ngay khi hoàn tất) — KỂ CẢ khi
        // `confirm_import_with_encoding` sau đó trả `Err` (đĩa đầy giữa lúc ghi ảnh, §Always
        // "kể cả lượt trượt"). Không còn một bước `append_domain_log` RỜI sau `?` — bước đó
        // sẽ KHÔNG BAO GIỜ chạy trên đường lỗi (chính vấn đề mục B1 sửa), và giữ nó trên
        // đường thành công sẽ nối trùng hai lần cùng một lô entry.
        //
        // ⚠️ Vắng `DomainLogState` (lỗi lắp dây ở `lib.rs`) ⇒ rơi về một kho TẠM cục bộ,
        // best-effort — cùng triết lý mọi state Tauri vắng mặt khác trong tệp này (chẩn đoán,
        // không chặn nghiệp vụ). Kho tạm này chỉ SỐNG trong lượt gọi này rồi mất — chấp nhận
        // được vì đây là đường KHÔNG BAO GIỜ xảy ra trên một app đã khởi động đúng
        // (`open_work_slot` luôn `.manage()` nó).
        let fallback_domain_log_state: webimport::DomainLogState = std::sync::Mutex::new(Vec::new());
        let domain_log_state = app.try_state::<webimport::DomainLogState>();
        let domain_log_state_ref = domain_log_state.as_deref().unwrap_or_else(|| {
            eprintln!("webimport[domain_log] DomainLogState chua duoc quan ly - dung kho tam");
            &fallback_domain_log_state
        });
        let opened = super::confirm_import_with_encoding(
            &root,
            &pending_state,
            &name,
            &source_lang,
            &genre,
            &encoding,
            cleanup_rules,
            pattern,
            block_overrides,
            origin_overrides,
            domain_log_state_ref,
        )?;
        reset_tier2_block_overrides(&app);
        reset_chapter_origin_overrides(&app);

        // P6 (vòng rà đối kháng bước 4) — `create_work` VỪA thành công (dòng trên đã `?`
        // sớm trên lỗi): dọn `UrlImportItemsState` CÙNG kỷ luật với `PendingImportSourceState`
        // (chỉ dọn khi thành công). Best-effort: state vắng mặt là một lỗi lắp dây ở
        // `lib.rs`, không phải lý do để báo hỏng một Tác phẩm VỪA tạo xong thành công.
        if let Some(items_state) = app.try_state::<super::UrlImportItemsState>() {
            super::clear_url_import_items_after_successful_confirm(&items_state);
        }

        let created = CreatedWork::from_open(&opened);
        reindex_library(&app, &root);
        let work_id = opened.meta.work_id.clone();
        let chapter_id = opened.chapter_id;
        let scan_source_lang = source_lang.clone();
        replace_open_work(&app, opened);
        Ok(super::keep_committed_import_when_scan_spawn_fails(
            created,
            || spawn_import_scan(app, work_id, chapter_id, scan_source_lang),
        ))
    }

    /// Vỏ IPC — màn xem trước bảng mã của đường nhập song ngữ (Story 6.16, FR115). Cùng
    /// khuôn `preview_import_encoding_from_file`: **không một quy tắc nào sống ở đây** — đọc
    /// [`super::preview_bilingual_import`] và [`super::stash_pending_import_source`] (tái
    /// dùng, KHÔNG một state thứ ba — §Design Notes đầu mục Story 6.16 của `commands::project`).
    ///
    /// Chỉ lượt MỞ đi qua vỏ này. Mọi lượt đổi cột, đảo vai, bật/tắt tiêu đề, sửa mẫu phân
    /// tách sau đó đi qua `rebuild_bilingual_import_preview` — không đọc lại tệp.
    ///
    /// # Lỗi
    /// - [`PendingImportSourceState`] chưa được quản lý ⇒ `import.no_pending_source`;
    /// - đuôi tệp không phải `.csv`/`.tsv` ⇒ `import.bilingual_unsupported_format`;
    /// - ứng viên bảng mã đang chọn gặp một ô mở ngoặc kép không bao giờ đóng, hoặc tệp có ít
    ///   hơn hai cột ⇒ `import.bilingual_unterminated_quoted_field` /
    ///   `import.bilingual_too_few_columns`, TỪ CHỐI trước khi có gì để xem trước, và ô đang
    ///   chờ được dọn để không một nguồn CŨ nào nằm lại sau một lượt mở MỚI bị từ chối.
    #[tauri::command]
    pub fn preview_bilingual_import_from_file(
        app: tauri::AppHandle,
        path: String,
        source_lang: String,
        chapter_pattern: Option<super::ChapterPatternWire>,
        source_column: usize,
        target_column: usize,
        has_header: bool,
    ) -> Result<super::BilingualImportEncodingPreview, IpcError> {
        use tauri::Manager as _;
        let Some(state) = app.try_state::<PendingImportSourceState>() else {
            return Err(no_pending_import_source());
        };
        let pattern = super::resolve_chapter_pattern(chapter_pattern)?;
        let shape = super::import_bilingual_file(std::path::Path::new(&path))?;
        // Cùng lý do `preview_import_encoding_from_file` dọn hai state này cho MỌI lượt xem
        // trước MỚI, kể cả khi đường hiện tại không đọc chúng — vòng đời NHẤT QUÁN quan
        // trọng hơn một lượt dọn thừa (xem doc-comment `Tier2BlockOverridesState`/
        // `ChapterOriginOverridesState` mục "Reset khi nào").
        reset_tier2_block_overrides(&app);
        reset_chapter_origin_overrides(&app);
        let cleanup_rules = resolve_cleanup_rules(&app);
        let preview = match super::preview_bilingual_import(
            &shape,
            &source_lang,
            &cleanup_rules,
            pattern.as_ref(),
            source_column,
            target_column,
            has_header,
        ) {
            Ok(preview) => preview,
            Err(err) => {
                super::cancel_import_preview(&state);
                return Err(err);
            }
        };
        super::stash_pending_import_source(&state, shape, None);
        Ok(preview)
    }

    /// Vỏ IPC — DỰNG LẠI màn xem trước song ngữ trên nguồn ĐANG CHỜ (Story 6.16, FR115), cho
    /// mọi lượt đổi cột nguồn/đích, đảo vai, bật/tắt tiêu đề, sửa mẫu phân tách.
    ///
    /// 🔴 **KHÔNG đọc tệp, KHÔNG cất lại nguồn, KHÔNG dọn override.** Quyết định Ice
    /// 2026-09-11: *"toggling rebuilds the preview in memory"*. Bản đầu gọi lại
    /// `preview_bilingual_import_from_file` với CÙNG `path` cho mọi lượt đổi — tức ĐỌC LẠI TỆP
    /// mỗi lần, và một tệp bị sửa trên đĩa giữa hai lượt đổi cột sẽ lặng lẽ thay nguồn đang
    /// xem trước. Vỏ này clone `shape` từ [`PendingImportSourceState`] — đúng byte mà lượt mở
    /// đã đọc — rồi gọi lại [`super::preview_bilingual_import`]. Canh bằng
    /// `ipc_contract.rs::the_three_bilingual_import_wires_are_registered_read_cleanup_rules_and_rebuild_never_reads_the_file`.
    ///
    /// # Lỗi
    /// - không có nguồn đang chờ ⇒ `import.no_pending_source`;
    /// - lỗi hình dạng bảng của ứng viên đang chọn ⇒ như [`super::preview_bilingual_import`].
    #[tauri::command]
    pub fn rebuild_bilingual_import_preview(
        app: tauri::AppHandle,
        source_lang: String,
        chapter_pattern: Option<super::ChapterPatternWire>,
        source_column: usize,
        target_column: usize,
        has_header: bool,
    ) -> Result<super::BilingualImportEncodingPreview, IpcError> {
        use tauri::Manager as _;
        let Some(state) = app.try_state::<PendingImportSourceState>() else {
            return Err(no_pending_import_source());
        };
        let pattern = super::resolve_chapter_pattern(chapter_pattern)?;
        let shape = {
            let guard = state.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
            guard.as_ref().map(|p| p.shape.clone()).ok_or_else(no_pending_import_source)?
        };
        let cleanup_rules = resolve_cleanup_rules(&app);
        super::preview_bilingual_import(
            &shape,
            &source_lang,
            &cleanup_rules,
            pattern.as_ref(),
            source_column,
            target_column,
            has_header,
        )
    }

    /// Vỏ IPC — xác nhận lượt nhập song ngữ (Story 6.16, FR115). Cùng khuôn
    /// `confirm_import_with_encoding`: lõi là [`super::confirm_bilingual_import`] (điểm gọi
    /// [`super::create_work`] cho đường này), vỏ chỉ phân giải thư mục gốc, gói kết quả, và
    /// nối tiếp lượt tái lập chỉ mục.
    ///
    /// ⚠️ **KHÔNG spawn quét Glossary** — khác `confirm_import_with_encoding`: một Chương
    /// song ngữ đã có bản dịch, quét ứng viên Glossary (Story 3.5, FR47) là nghĩa vụ của
    /// đường văn xuôi (câu chưa dịch cần gợi ý thuật ngữ); không nằm trong Task list spec
    /// 6.16, và spawn nó vào đây sẽ là một bề mặt MỚI không AC nào của story này canh.
    #[tauri::command]
    pub fn confirm_bilingual_import(
        app: tauri::AppHandle,
        name: String,
        source_lang: String,
        genre: String,
        encoding: String,
        chapter_pattern: Option<super::ChapterPatternWire>,
        source_column: usize,
        target_column: usize,
        has_header: bool,
    ) -> Result<CreatedWork, IpcError> {
        use tauri::Manager as _;

        let Some(pending_state) = app.try_state::<PendingImportSourceState>() else {
            return Err(no_pending_import_source());
        };
        let pattern = super::resolve_chapter_pattern(chapter_pattern)?;
        let root = resolve_library_root(&app, app.try_state::<Store>().as_deref())?;
        // Đọc hai tầng luật LÚC XÁC NHẬN — cùng kỷ luật `confirm_import_with_encoding`.
        let cleanup_rules = resolve_cleanup_rules(&app);
        let opened = super::confirm_bilingual_import(
            &root,
            &pending_state,
            &name,
            &source_lang,
            &genre,
            &encoding,
            cleanup_rules,
            pattern,
            source_column,
            target_column,
            has_header,
        )?;

        let created = CreatedWork::from_open(&opened);
        reindex_library(&app, &root);
        replace_open_work(&app, opened);
        Ok(created)
    }

    /// Vỏ IPC — mở lại một `.atproj` **đã có trên đĩa** (Story 5.7, FR12).
    ///
    /// 🔴 Tham số là `work_id`, KHÔNG một đường dẫn hệ tệp (§Never của story):
    /// `atproj_path` phân giải Ở RUST, từ `library-index.db`, qua [`Indexer::find_work`] —
    /// webview không bao giờ tự dựng hay truyền một đường dẫn.
    #[tauri::command]
    pub fn open_work(app: tauri::AppHandle, work_id: String) -> Result<OpenedWork, IpcError> {
        use tauri::Manager as _;

        let Some(indexer) = app.try_state::<Indexer>() else {
            return Err(indexer_is_missing());
        };

        // 🔴 **Không một quy tắc nào sống ở đây** — `indexed` (kể cả `None`) chuyển thẳng
        // xuống hàm thuần [`super::open_work`], nơi quyết định *"`None` ⇒
        // `library.work_not_indexed`"* thật sự sống (xem doc-comment của hàm đó).
        let indexed = indexer.find_work(&work_id)?;
        let opened = super::open_work(&work_id, indexed.as_ref())?;
        let result = OpenedWork::from_open(&opened);
        replace_open_work(&app, opened);
        Ok(result)
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Story 6.7 — Nhập từ URL bằng danh sách link (AD-15 · AD-40 · AD-41 · FR122)
    // ─────────────────────────────────────────────────────────────────────────

    /// Vỏ IPC — tải TUẦN TỰ đúng thứ tự đã dán, dựng trạng thái từng mục, ĐỒNG BỘ
    /// [`PendingImportSourceState`] (khoá/mở xác nhận tự động qua máy Story 6.3 — xem
    /// doc-comment [`super::sync_pending_from_url_items`]).
    ///
    /// `#[tauri::command(async)]` trên một hàm ĐỒNG BỘ — khuôn đã có 17 tiền lệ
    /// (`library.rs:640`) để `reqwest::blocking` (gọi tuần tự, có thể mất tới N × 20 giây)
    /// không chặn luồng chính (Task 0 — xem `core::webimport` doc-comment cho phép đo).
    #[tauri::command(async)]
    pub fn start_url_import(
        app: tauri::AppHandle,
        urls: Vec<String>,
        source_lang: String,
    ) -> Result<super::UrlImportBatchWire, IpcError> {
        use tauri::Manager as _;

        let Some(items_state) = app.try_state::<super::UrlImportItemsState>() else {
            return Err(super::url_import_internal_error());
        };
        let Some(pending_state) = app.try_state::<PendingImportSourceState>() else {
            return Err(no_pending_import_source());
        };
        let cleanup_rules = resolve_cleanup_rules(&app);

        let (items, log_entries) = super::fetch_url_import_items(urls);
        append_domain_log(&app, log_entries);
        super::sync_pending_from_url_items(&pending_state, &items);
        // Story 6.9 — danh sách HOÀN TOÀN MỚI, dọn override CŨ trước khi dựng dây (xem
        // doc-comment `Tier2BlockOverridesState` mục "Reset khi nào").
        reset_tier2_block_overrides(&app);
        // Story 6.15 — cùng lý do: danh sách MỚI, chỉ số Chương cũ không còn khớp gì.
        reset_chapter_origin_overrides(&app);
        let block_overrides = resolve_tier2_block_overrides(&app);
        let origin_overrides = resolve_chapter_origin_overrides(&app);
        let wire = super::url_import_batch_wire(
            &items,
            &source_lang,
            &cleanup_rules,
            &block_overrides,
            domain_log_domain_count(&app),
            &origin_overrides,
        );

        let mut guard = items_state.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
        *guard = Some(items);
        Ok(wire)
    }

    /// Vỏ IPC — tải lại ĐÚNG MỘT mục hỏng (I/O Matrix spec 6.7: "đúng 1 lời gọi mạng, chỉ
    /// tới URL của mục k"). Trượt lần nữa ⇒ mục mang lý do MỚI (có thể khác lý do cũ).
    #[tauri::command(async)]
    pub fn reload_url_import_item(
        app: tauri::AppHandle,
        index: usize,
        source_lang: String,
    ) -> Result<super::UrlImportBatchWire, IpcError> {
        use tauri::Manager as _;

        let Some(items_state) = app.try_state::<super::UrlImportItemsState>() else {
            return Err(super::url_import_internal_error());
        };
        let Some(pending_state) = app.try_state::<PendingImportSourceState>() else {
            return Err(no_pending_import_source());
        };
        let cleanup_rules = resolve_cleanup_rules(&app);

        let mut guard = items_state.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
        let Some(items) = guard.as_mut() else {
            return Err(super::url_import_internal_error());
        };
        let Some(url) = items.get(index).map(|it| it.url.clone()) else {
            return Err(super::url_import_internal_error());
        };
        // Đúng MỘT lời gọi mạng — `fetch_url_import_item` gọi thẳng `webimport::fetch`,
        // không có vòng lặp nào bọc quanh nó ở đây. Allowlist dựng từ TOÀN BỘ danh sách hiện
        // tại (§Always spec 6.8) — xem doc-comment [`super::allowlist_from_items`].
        let allowlist = super::allowlist_from_items(items.as_slice());
        let log_entries = if let Some(slot) = items.get_mut(index) {
            let (new_item, log_entries) = super::fetch_url_import_item(&url, &allowlist);
            *slot = new_item;
            log_entries
        } else {
            Vec::new()
        };
        append_domain_log(&app, log_entries);
        super::sync_pending_from_url_items(&pending_state, items);
        // Story 6.9 — CHỈ mục 0 (Chương tầng 2 đang hiển thị) làm override cũ SAI Ý NGHĨA khi
        // tải lại — xem doc-comment `Tier2BlockOverridesState` mục "Reset khi nào".
        if super::mutated_index_invalidates_tier2_blocks(index) {
            reset_tier2_block_overrides(&app);
        }
        // Story 6.15 — một lượt tải LẠI đổi nội dung (và có thể cả xuất xứ) của ĐÚNG mục
        // `index`; khác Tier2 (chỉ mục 0 có ý nghĩa), xuất xứ áp cho MỌI Chương — dọn TOÀN BỘ
        // là lựa chọn AN TOÀN, không giữ một override có thể đã sai cho đúng chỉ số đó.
        reset_chapter_origin_overrides(&app);
        let block_overrides = resolve_tier2_block_overrides(&app);
        let origin_overrides = resolve_chapter_origin_overrides(&app);
        Ok(super::url_import_batch_wire(
            items,
            &source_lang,
            &cleanup_rules,
            &block_overrides,
            domain_log_domain_count(&app),
            &origin_overrides,
        ))
    }

    /// Vỏ IPC — bỏ MỘT mục khỏi danh sách (I/O Matrix spec 6.7: "N−1 link · N−1 Chương — hai
    /// số cùng giảm"). **0 lời gọi mạng.** Không `(async)` — chỉ đổi state trong bộ nhớ.
    #[tauri::command]
    pub fn remove_url_import_item(
        app: tauri::AppHandle,
        index: usize,
        source_lang: String,
    ) -> Result<super::UrlImportBatchWire, IpcError> {
        use tauri::Manager as _;

        let Some(items_state) = app.try_state::<super::UrlImportItemsState>() else {
            return Err(super::url_import_internal_error());
        };
        let Some(pending_state) = app.try_state::<PendingImportSourceState>() else {
            return Err(no_pending_import_source());
        };
        let cleanup_rules = resolve_cleanup_rules(&app);

        let mut guard = items_state.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
        let Some(items) = guard.as_mut() else {
            return Err(super::url_import_internal_error());
        };
        if index >= items.len() {
            return Err(super::url_import_internal_error());
        }
        items.remove(index);
        super::sync_pending_from_url_items(&pending_state, items);
        // Story 6.9 — cùng lý do `reload_url_import_item`: chỉ mục 0 làm cấu trúc khối của
        // Chương tầng 2 khác đi.
        if super::mutated_index_invalidates_tier2_blocks(index) {
            reset_tier2_block_overrides(&app);
        }
        // Story 6.15 — bỏ một mục dời chỉ số của MỌI mục đứng sau nó; dọn TOÀN BỘ override
        // xuất xứ thay vì cố dịch chuyển từng chỉ số, cùng lý lẽ `reload_url_import_item`.
        reset_chapter_origin_overrides(&app);
        let block_overrides = resolve_tier2_block_overrides(&app);
        let origin_overrides = resolve_chapter_origin_overrides(&app);
        Ok(super::url_import_batch_wire(
            items,
            &source_lang,
            &cleanup_rules,
            &block_overrides,
            domain_log_domain_count(&app),
            &origin_overrides,
        ))
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Story 6.9 — sửa ranh giới BÓC bằng bàn phím (FR123): hai lệnh ghi override, cả hai trả
    // lại `UrlImportBatchWire` TƯƠI, cùng khuôn ba lệnh URL ngay trên (đặt/gỡ trạng thái rồi
    // dựng lại dây từ CHÍNH state đó — không một lệnh "làm mới" riêng).
    // ─────────────────────────────────────────────────────────────────────────

    /// Vỏ IPC — đổi trạng thái giữ/loại của MỘT khối (`Space`). **0 lời gọi mạng**: chỉ đổi
    /// `Tier2BlockOverridesState` trong bộ nhớ rồi dựng lại xem trước từ byte ĐÃ TẢI
    /// (`UrlImportItemsState`). Không một quy tắc nào sống ở đây — [`super::set_block_override`]
    /// là hàm thuần (`src-tauri/AGENTS.md:11`).
    ///
    /// 🔴 **SỬA 2026-09-07 (vòng rà bước 4, mục 5) — đo `total_blocks` THẬT qua
    /// [`super::current_tier2_machine_kept`] trước khi ghi**, KHÔNG tin `index` một cách mù.
    /// `index` ngoài phạm vi trang HIỆN HÀNH (trang vừa tải lại, số khối đổi từ dưới lượt
    /// hiển thị cũ của frontend) ⇒ [`super::url_import_internal_error`] — từ chối ghi một
    /// override "ma", thay vì lặng lẽ nới vector tới một chỉ số không khối nào trỏ tới.
    #[tauri::command]
    pub fn tier2_block_set_kept(
        app: tauri::AppHandle,
        index: usize,
        kept: bool,
        source_lang: String,
    ) -> Result<super::UrlImportBatchWire, IpcError> {
        use tauri::Manager as _;

        let Some(items_state) = app.try_state::<super::UrlImportItemsState>() else {
            return Err(super::url_import_internal_error());
        };
        let Some(overrides_state) = app.try_state::<Tier2BlockOverridesState>() else {
            return Err(super::url_import_internal_error());
        };
        let cleanup_rules = resolve_cleanup_rules(&app);

        {
            let guard = items_state.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
            let Some(items) = guard.as_ref() else {
                return Err(super::url_import_internal_error());
            };
            let Some(machine_kept) =
                super::current_tier2_machine_kept(items, &source_lang, &cleanup_rules)
            else {
                return Err(super::url_import_internal_error());
            };
            let mut overrides_guard =
                overrides_state.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
            if super::set_block_override(&mut overrides_guard, index, kept, machine_kept.len())
                .is_err()
            {
                return Err(super::url_import_internal_error());
            }
        }

        let guard = items_state.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
        let Some(items) = guard.as_ref() else {
            return Err(super::url_import_internal_error());
        };
        let block_overrides = resolve_tier2_block_overrides(&app);
        let origin_overrides = resolve_chapter_origin_overrides(&app);
        Ok(super::url_import_batch_wire(
            items,
            &source_lang,
            &cleanup_rules,
            &block_overrides,
            domain_log_domain_count(&app),
            &origin_overrides,
        ))
    }

    /// Vỏ IPC — đặt CẢ MỘT DẢI `[start, end]` thành giữ, MỌI khối NGOÀI dải thành loại, một
    /// lượt (`[`/`]`, I/O Matrix spec 6.9).
    ///
    /// 🔴 **SỬA 2026-09-07 (vòng rà bước 4, mục 5/6) — `total` frontend gửi lên giờ chỉ là
    /// một CHỮ KÝ để đối chiếu, KHÔNG còn là nguồn sự thật.** Vỏ tự đo `machine_kept` THẬT
    /// qua [`super::current_tier2_machine_kept`] (đọc byte ĐÃ TẢI, không tin bất kỳ số nào
    /// frontend gửi lên); `total != machine_kept.len()` ⇒ trạng thái frontend đang hiện đã
    /// CŨ (trang vừa tải lại từ dưới tay) ⇒ [`super::url_import_internal_error`], từ chối
    /// dựng một patch trên một tổng số khối không còn đúng với trang hiện hành.
    #[tauri::command]
    pub fn tier2_block_confirm_range(
        app: tauri::AppHandle,
        start: usize,
        end: usize,
        total: usize,
        source_lang: String,
    ) -> Result<super::UrlImportBatchWire, IpcError> {
        use tauri::Manager as _;

        let Some(items_state) = app.try_state::<super::UrlImportItemsState>() else {
            return Err(super::url_import_internal_error());
        };
        let Some(overrides_state) = app.try_state::<Tier2BlockOverridesState>() else {
            return Err(super::url_import_internal_error());
        };
        let cleanup_rules = resolve_cleanup_rules(&app);

        {
            let guard = items_state.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
            let Some(items) = guard.as_ref() else {
                return Err(super::url_import_internal_error());
            };
            let Some(machine_kept) =
                super::current_tier2_machine_kept(items, &source_lang, &cleanup_rules)
            else {
                return Err(super::url_import_internal_error());
            };
            if total != machine_kept.len() {
                return Err(super::url_import_internal_error());
            }
            let mut overrides_guard =
                overrides_state.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
            *overrides_guard = super::block_overrides_for_range(start, end, &machine_kept);
        }

        let guard = items_state.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
        let Some(items) = guard.as_ref() else {
            return Err(super::url_import_internal_error());
        };
        let block_overrides = resolve_tier2_block_overrides(&app);
        let origin_overrides = resolve_chapter_origin_overrides(&app);
        Ok(super::url_import_batch_wire(
            items,
            &source_lang,
            &cleanup_rules,
            &block_overrides,
            domain_log_domain_count(&app),
            &origin_overrides,
        ))
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Story 6.10a — con trỏ *Chương đang chọn*: chi tiết LAZY (tầng 2/3) cho Chương thứ k khi
    // con trỏ dời (`⌥←`/`⌥→`). KHÔNG `(async)`: byte đã tải sống trong `UrlImportItemsState`
    // (AD-41 — 0 lời gọi mạng khi người dùng không bấm), chỉ chạy lại chuỗi pipeline trong bộ
    // nhớ.
    // ─────────────────────────────────────────────────────────────────────────

    /// Vỏ IPC — dựng lại tầng 2/3 cho Chương thứ `chapter_index`, với bảng mã ĐÃ CHỌN (không
    /// dò lại, không lặp năm ứng viên). Đọc [`super::UrlImportItemsState`] TRỰC TIẾP, KHÔNG
    /// [`PendingImportSourceState`] (trạng thái GHI — `None` khi còn mục hỏng, trong khi vẫn
    /// có thể có Chương HỢP LỆ để mà xem, đúng tách vị từ XEM/GHI §Always story 6.10a) — dựng
    /// hình dạng XEM qua [`super::chapters_shape_for_view`], cùng nguồn mà
    /// `url_import_encoding_preview` dùng.
    ///
    /// # Lỗi
    /// - [`super::UrlImportItemsState`] chưa được quản lý, chưa mở lượt URL nào, hoặc KHÔNG
    ///   mục OK nào ⇒ `import.web_internal_error` — con trỏ không có gì để mà dựng lại;
    /// - `encoding` không giải ngược được thành một bảng mã đã biết ⇒ cùng lỗi trên;
    /// - `chapter_index` ngoài phạm vi Chương thật của lượt chạy MỚI, hoặc bảng mã đã chọn
    ///   "không ra chữ" cho Chương này ⇒ cùng lỗi trên — frontend coi đây là trạng thái CŨ
    ///   (mẫu phân tách vừa đổi làm N đổi dưới chân), không phải một lỗi mạng.
    #[tauri::command]
    pub fn preview_chapter_detail(
        app: tauri::AppHandle,
        chapter_index: usize,
        encoding: String,
        source_lang: String,
        chapter_pattern: Option<super::ChapterPatternWire>,
    ) -> Result<super::ChapterDetailWire, IpcError> {
        use tauri::Manager as _;

        let Some(items_state) = app.try_state::<super::UrlImportItemsState>() else {
            return Err(super::url_import_internal_error());
        };
        let Some(resolved_encoding) = crate::core::segment::encoding::encoding_for_wire_id(&encoding)
        else {
            return Err(super::url_import_internal_error());
        };
        let pattern = super::resolve_chapter_pattern(chapter_pattern)?;
        let cleanup_rules = resolve_cleanup_rules(&app);

        let guard = items_state.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
        let Some(items) = guard.as_ref() else {
            return Err(super::url_import_internal_error());
        };
        let Some(shape) = super::chapters_shape_for_view(items) else {
            return Err(super::url_import_internal_error());
        };
        let block_overrides = resolve_tier2_block_overrides(&app);
        // Đường URL là đường DUY NHẤT dựng `PipelineShape::Chapters` (§Always spec 6.7) —
        // `extract_main_content = true` luôn đúng ở đây, cùng lý do `start_url_import`.
        match super::chapter_detail_for_index(
            &shape,
            chapter_index,
            resolved_encoding,
            pattern.as_ref(),
            &source_lang,
            &cleanup_rules,
            true,
            &block_overrides,
        ) {
            Some((cleanup, blocks)) => Ok(super::ChapterDetailWire { cleanup, blocks }),
            None => Err(super::url_import_internal_error()),
        }
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Story 6.8 (NFR19) — đọc TOÀN BỘ nhật ký domain của phiên chạy, cho Cài đặt › Quyền
    // riêng tư. KHÔNG `(async)` — chỉ đọc một `Mutex<Vec<_>>` trong bộ nhớ, 0 mạng, 0 đĩa.
    // ─────────────────────────────────────────────────────────────────────────

    /// Vỏ IPC — đọc TOÀN BỘ nhật ký domain THÔ của phiên chạy hiện tại (§Always spec 6.8:
    /// "không phân trang, không xem thêm che bớt hàng" — trả nguyên mảng, gộp là việc của
    /// tầng trình bày). Thiếu [`webimport::DomainLogState`] (lỗi lắp dây) ⇒ mảng RỖNG, cùng
    /// khuôn best-effort của `append_domain_log`/`domain_log_domain_count` — một màn Cài đặt
    /// trống vẫn tốt hơn một lỗi chặn cả lớp phủ vì một sự cố ở một tính năng phụ trợ.
    #[tauri::command]
    pub fn list_domain_log(app: tauri::AppHandle) -> Vec<super::DomainLogEntryWire> {
        use tauri::Manager as _;

        match app.try_state::<webimport::DomainLogState>() {
            Some(state) => webimport::read_domain_log(&state).iter().map(super::DomainLogEntryWire::from).collect(),
            None => {
                eprintln!("webimport[domain_log] DomainLogState chua duoc quan ly - tra ve mang rong");
                Vec::new()
            }
        }
    }
}
