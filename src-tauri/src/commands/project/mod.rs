//! Bề mặt IPC của một Tác phẩm — Story 1.15 trở đi. `OpenWork` (kho `project.db` đang mở),
//! thư mục thư viện (`resolve_library_root`), hệ xem trước bảng mã/Tầng 2/xuất xứ Chương đan
//! xen (`preview_import_encoding`, `cleanup_and_chapters_preview_for`, `Tier2BlockOverridesState`,
//! `ChapterOriginOverride`), lượt quét Glossary khi nhập (`spawn_import_scan`), và `open_work`/
//! `replace_open_work` (mở lại một `.atproj` đã có).
//!
//! Cùng khuôn `commands::config`: hàm thuần trước, `#[tauri::command]` chỉ là vỏ mỏng
//! trong `wire`.
//!
//! `create_work`/`append_chapters_to_work` và pha tải ảnh nhúng sống ở [`work_creation`]; nhập
//! song ngữ ở [`bilingual`]; nhập từ URL ở [`url_import`]. Ba mối quan tâm đan xen (bảng mã/
//! Tầng 2/xuất xứ Chương) VẪN ở lại đây — xem doc-comment của các mô-đun con và
//! `deferred-work.md` cho lý do không tách tiếp.
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
    ImportError, import_bilingual_file, import_file, import_files, import_text,
    web_import_item_failure_ipc_error,
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
    /// Cảnh báo KHÔNG CHẶN — `true` khi nội dung VỪA ghi/thêm trông như một ngôn ngữ khác
    /// `source_lang` đã khai (heuristic tỉ lệ chữ, xem
    /// [`source_lang_looks_mismatched`]). Không chặn lượt ghi — luôn tính LẠI mỗi lượt gọi,
    /// cùng ngữ nghĩa "của ĐÚNG lượt gọi này" mà `images_saved`/`images_failed` đã theo.
    pub source_lang_mismatch: bool,
    // 🔵 SỬA 2026-09-08 (mục B1 vòng rà đối kháng 3 lớp) — trường `pending_domain_log` đã BỊ
    // GỠ. Bản trước tích luỹ `Vec<DomainLogEntry>` ở đây, CHỈ nối vào `DomainLogState` trên
    // đường THÀNH CÔNG — một lượt trượt (đĩa đầy giữa lúc ghi ảnh) làm nó biến mất, vi phạm
    // §Always "kể cả lượt trượt". `create_work` nay nhận thẳng `&DomainLogState` và mỗi lời
    // gọi `fetch` push NGAY khi hoàn tất — không còn gì để mà "mang" qua `OpenWork` nữa.
    /// `chapter.id` của MỌI Chương vừa ghi bởi lượt [`create_work`]/[`append_chapters_to_work`]
    /// NÀY — cùng ngữ nghĩa "của ĐÚNG lượt gọi này" mà `images_saved`/`source_lang_mismatch` đã
    /// theo. Rỗng cho [`open_work`] (mở lại không ghi Chương nào). Tách khỏi `chapter_id` ở
    /// trên (Chương ĐANG MỞ trong Editor, một khái niệm khác — Story 2.11) vì một lượt nhập
    /// N Chương chỉ mở MỘT trong số đó, còn lượt quét Glossary cần TOÀN BỘ N.
    pub new_chapter_ids: Vec<i64>,
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
/// ĐÚNG MỘT chỗ. Story 6.5 mở nợ `deferred-work.md §*Deferred from: 6-2-pipeline-nhap-mot-chuoi-thu-tu-co-dinh-dung-chung-moi-nguon (2026-09-04)*`: `preview_import_encoding` (mỗi
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
    /// `completed`, `dictionary_inconclusive`, or `scan_failed`. A cancelled worker still
    /// emits nothing; `scan_failed` is only for the six genuine infrastructure branches
    /// (segment read, `app_config` read, missing `global.db`, filter/enqueue, write).
    pub outcome: &'static str,
}

/// Tên sự kiện trên dây — khuôn `EXIT_FLUSH_EVENT` (`lib.rs:161`).
pub const GLOSSARY_IMPORT_SCAN_EVENT: &str = "aura://glossary-import-scan-completed";
const IMPORT_SCAN_COMPLETED: &str = "completed";
const IMPORT_SCAN_DICTIONARY_INCONCLUSIVE: &str = "dictionary_inconclusive";
const IMPORT_SCAN_FAILED: &str = "scan_failed";

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

/// Cancels an in-flight image-download pass, mirroring
/// [`crate::commands::aitranslate::AiTranslateGeneration`]: `next()` captures this pass's
/// generation (retiring any earlier pass); `wire::cancel_image_download` calls `next()`
/// again to bump the counter, so `is_current(generation)` goes `false` for the running
/// pass. `prepare_chapter_images` polls it before each image.
#[derive(Debug, Clone, Default)]
pub struct ImageDownloadGeneration(Arc<AtomicU64>);

impl ImageDownloadGeneration {
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

/// Same shape as [`dictionary_inconclusive_event`]: kept separate from `emit` so the wire
/// shape is testable without an `AppHandle`.
fn scan_failed_event(chapter_id: i64) -> GlossaryImportScanEvent {
    GlossaryImportScanEvent {
        chapter_id,
        inserted: 0,
        skipped: 0,
        outcome: IMPORT_SCAN_FAILED,
    }
}

/// The one place all six infrastructure-failure branches of [`spawn_import_scan`] emit
/// `scan_failed`, so they can't drift from each other. Swallows its own emit failure —
/// already on an error path, so a second error here only goes to `stderr`.
fn emit_import_scan_failed(app: &tauri::AppHandle, chapter_id: i64) {
    use tauri::Emitter as _;
    if let Err(err) = app.emit(GLOSSARY_IMPORT_SCAN_EVENT, scan_failed_event(chapter_id)) {
        eprintln!("glossary[import_scan] phat su kien that bai: {err}");
    }
}

/// Pulled out of `spawn_import_scan`'s thread closure so the threshold read and its use in
/// `scan_candidates_controlled` are testable together without a background thread or an
/// `AppHandle`; `config` comes in as a plain parameter instead of `app.try_state`.
fn scan_with_configured_threshold(
    config: &crate::core::scope::GlobalConfig,
    segments: &[&str],
    lang: crate::core::matching::MatchLang,
    probe_dictionary: &mut dyn FnMut(&str) -> crate::core::glossary::DictionaryProbe,
    is_cancelled: &mut dyn FnMut() -> bool,
) -> crate::core::glossary::ScanOutcome {
    let threshold = config.glossary_scan_threshold();
    crate::core::glossary::scan_candidates_controlled(
        segments,
        lang,
        threshold,
        crate::core::glossary::COMMON_SURNAMES,
        probe_dictionary,
        is_cancelled,
    )
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
             ORDER BY ord, id",
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

/// Chạy lượt quét Glossary cho TOÀN BỘ `chapter_ids` của Tác phẩm `work_id`, trên một
/// `std::thread` RIÊNG — spawn TỪ `wire::create_work_from_text`/`wire::create_work_from_file`/
/// `wire::confirm_import_with_encoding` (cả nhánh Tác phẩm MỚI lẫn nhánh APPEND), **SAU** khi
/// `replace_open_work` đã đặt `OpenWork` vào state (tức sau khi transaction nhập đã commit —
/// spawn TRƯỚC đó là quét những Chương chưa tồn tại).
///
/// 🔴 **MỘT generation cho CẢ LÔ, không một generation mỗi Chương.** [`ImportScanGeneration`]
/// là một bộ đếm DUY NHẤT toàn ứng dụng: gọi `.next()` một lần nữa sẽ làm generation TRƯỚC ĐÓ
/// hết hiện hành ngay lập tức. Gọi hàm này N lần (một lần mỗi Chương) cho một lượt nhập N
/// Chương sẽ tự huỷ (N-1) lượt quét đầu của CHÍNH lượt nhập đó. Một lời gọi hàm này ⇒ một
/// `.next()` ⇒ một luồng quét nguyên vẹn toàn bộ danh sách.
///
/// `config`/`layers`/`disabled`/`lang` được nạp ĐÚNG MỘT LẦN cho cả lô, không phải mỗi Chương —
/// chúng không đổi giữa chừng một lượt quét, và nạp lại `app_config` từ `global.db` N lần chỉ
/// để đọc cùng một giá trị là chi phí nhân N vô ích.
///
/// ─────────────────────────────────────────────────────────────────────────────
/// 🔴 KHOÁ `OpenWorkState` HAI LẦN NGẮN MỖI CHƯƠNG, KHÔNG MỘT LẦN DÀI SUỐT CẢ LÔ
/// ─────────────────────────────────────────────────────────────────────────────
/// Giữ khoá mutex của `OpenWorkState` suốt pha quét (có thể tới vài giây trên một Chương lớn,
/// nhân lên qua N Chương) sẽ chặn MỌI lệnh khác cần đọc `OpenWorkState` — kể cả `read_open_chapter`
/// mà frontend gọi ngay sau khi tạo Tác phẩm để mở Chương trong Editor. [`run_one_chapter_import_scan`]
/// khoá ĐÚNG HAI lần MỖI Chương, ngắn: một lần để đọc segment (rồi nhả khoá TRƯỚC khi chạy thuật
/// toán quét, tốn CPU nhất), một lần để ghi lô (rồi nhả ngay) — vòng lặp dưới đây gọi lại nó cho
/// từng `chapter_id`, nên tổng thời gian giữ khoá tại một thời điểm bất kỳ vẫn chỉ là MỘT Chương,
/// không phải cả N. Cả hai lần khoá đều gọi ĐÚNG một hàm quyết định — [`guarded_open_store`] —
/// đối chiếu `work_id` với giá trị đã chốt lúc spawn: Tác phẩm đổi giữa hai lần khoá (một lượt
/// tạo Tác phẩm MỚI trong lúc lượt quét của Tác phẩm CŨ còn đang chạy) làm luồng nền kết thúc
/// LẶNG LẼ, không ghi vào kho SAI Tác phẩm — cùng I/O Matrix *"Kho đóng giữa lượt quét ⇒ kết
/// thúc lặng lẽ, không panic"*, mở rộng cho ca "Tác phẩm đổi". `current()` được hỏi lại ở ĐẦU
/// mỗi vòng lặp Chương — một Tác phẩm đổi/đóng giữa lô dừng các Chương CÒN LẠI ngay, không quét
/// nốt phần dở trên một Tác phẩm đã không còn là Tác phẩm lúc spawn.
///
/// 🔴 **Không `unwrap()`/`expect()` nào trên đường này** — `panic = "abort"` giết cả tiến
/// trình (AGENTS.md), và một luồng nền là chỗ tệ nhất để việc đó xảy ra: không ai đang chờ
/// kết quả của nó để thấy màn hình treo, người dùng chỉ thấy ứng dụng biến mất.
fn spawn_import_scan(
    app: tauri::AppHandle,
    work_id: String,
    chapter_ids: Vec<i64>,
    source_lang: String,
) -> std::io::Result<()> {
    use tauri::Manager as _;

    let Some(generation_state) = app.try_state::<ImportScanGeneration>() else {
        eprintln!("glossary[import_scan] generation state chua duoc quan ly -- bo qua luot quet");
        for &chapter_id in &chapter_ids {
            emit_import_scan_failed(&app, chapter_id);
        }
        return Ok(());
    };
    let generation_state = generation_state.inner().clone();
    let generation = generation_state.next();

    std::thread::Builder::new()
        .name(format!("aura-import-scan-{generation}"))
        .spawn(move || {
            let current = || generation_state.is_current(generation);
            if !current() {
                return;
            }

            let Some(global) = app.try_state::<Store>() else {
                eprintln!("glossary[import_scan] global.db chua duoc quan ly -- bo qua luot quet");
                for &chapter_id in &chapter_ids {
                    emit_import_scan_failed(&app, chapter_id);
                }
                return;
            };
            let config = match crate::core::scope::load_global_config(&global) {
                Ok(c) => c,
                Err(err) => {
                    eprintln!("glossary[import_scan] doc app_config that bai: {err}");
                    for &chapter_id in &chapter_ids {
                        emit_import_scan_failed(&app, chapter_id);
                    }
                    return;
                }
            };
            let disabled = config.disabled_source_codes();

            let layers_state = app.try_state::<crate::core::dict::DictLayers>();
            let Some(layers): Option<&crate::core::dict::DictLayers> =
                guarded_dict_layers(layers_state.as_deref(), "import_scan")
            else {
                return;
            };

            let lang = crate::core::glossary::match_lang_for_source_lang(&source_lang);

            for chapter_id in chapter_ids {
                if !current() {
                    return;
                }
                run_one_chapter_import_scan(
                    &app,
                    &work_id,
                    chapter_id,
                    &global,
                    &config,
                    &disabled,
                    layers,
                    lang,
                    &current,
                );
            }
        })
        .map(|_| ())
}

/// Thân của MỘT Chương trong vòng lặp [`spawn_import_scan`] — tách ra để hàm cha chỉ còn việc
/// nạp một lần những gì KHÔNG đổi giữa các Chương (`config`/`disabled`/`layers`/`lang`) rồi lặp.
/// Nhận `&tauri::AppHandle` (không `move` sở hữu) vì được gọi LẶP LẠI trong cùng một luồng.
fn run_one_chapter_import_scan(
    app: &tauri::AppHandle,
    work_id: &str,
    chapter_id: i64,
    global: &Store,
    config: &crate::core::scope::GlobalConfig,
    disabled: &std::collections::BTreeSet<String>,
    layers: &crate::core::dict::DictLayers,
    lang: crate::core::matching::MatchLang,
    current: &dyn Fn() -> bool,
) {
    use tauri::{Emitter as _, Manager as _};

    let segments: Vec<String> = {
        let Some(work_state) = app.try_state::<OpenWorkState>() else {
            return;
        };
        let guard = work_state.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
        let Some(store) = guarded_open_store(guard.as_ref(), work_id) else {
            return;
        };
        match read_chapter_segment_texts(store, chapter_id) {
            Ok(rows) => rows,
            Err(err) => {
                eprintln!("glossary[import_scan] doc segment that bai: {err}");
                emit_import_scan_failed(app, chapter_id);
                return;
            }
        }
    };

    let segment_refs: Vec<&str> = segments.iter().map(String::as_str).collect();
    // `skipped` mang CẢ layer hỏng lúc mở lẫn lúc lookup. Một kết quả rỗng kèm
    // `skipped` là KHÔNG KẾT LUẬN, không phải “term không có”.
    let mut probe_dictionary = |term: &str| {
        let result =
            crate::core::dict::lookup_grouped(layers, term, crate::core::dict::LookupMode::Exact, 1, disabled);
        dictionary_probe_from_grouped(&result)
    };
    let mut is_cancelled = || !current();
    let scan_outcome = scan_with_configured_threshold(
        config,
        &segment_refs,
        lang,
        &mut probe_dictionary,
        &mut is_cancelled,
    );

    let mut candidates = match import_scan_next_step(scan_outcome, current()) {
        ImportScanNextStep::Enqueue(candidates) => candidates,
        ImportScanNextStep::Stop => return,
        ImportScanNextStep::EmitDictionaryInconclusive => {
            if let Err(err) =
                app.emit(GLOSSARY_IMPORT_SCAN_EVENT, dictionary_inconclusive_event(chapter_id))
            {
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
        match filter_and_enqueue_current_import_scan(&work_state, work_id, global, &mut candidates, current) {
            Ok(Some(ticket)) => ticket,
            Ok(None) => return,
            Err(err) => {
                eprintln!("glossary[import_scan] loc/xep lo that bai: {err}");
                emit_import_scan_failed(app, chapter_id);
                return;
            }
        }
    };

    let (inserted, skipped) = match ticket.wait() {
        Ok(counts) => counts,
        Err(err) => {
            eprintln!("glossary[import_scan] ghi lo that bai: {err}");
            emit_import_scan_failed(app, chapter_id);
            return;
        }
    };
    if !current() {
        return;
    }

    if let Err(err) = app.emit(
        GLOSSARY_IMPORT_SCAN_EVENT,
        GlossaryImportScanEvent { chapter_id, inserted, skipped, outcome: IMPORT_SCAN_COMPLETED },
    ) {
        eprintln!("glossary[import_scan] phat su kien that bai: {err}");
    }
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
    /// **THÊM 2026-09-16 (Story 6.7b, FR122 nửa hai)** — `work_id` của Tác phẩm ĐÍCH,
    /// `None` ⇔ đích Tác phẩm MỚI. Cất tại lượt MỞ một phiên xem trước (ba đường đơn ngữ:
    /// dán/tệp/URL — Quyết định 1), sống suốt phiên (kể cả qua `reload`/`remove một mục URL`,
    /// đổi override khối tầng 2, dời con trỏ Chương xem trước — mọi thao tác đó gọi lại
    /// [`stash_pending_import_source`]/[`sync_pending_from_url_items`] với CÙNG giá trị này,
    /// không phải một giá trị mới), và đọc lại y nguyên lúc xác nhận qua
    /// [`current_pending_destination`].
    ///
    /// 🔴 **VÌ SAO Ở ĐÂY, KHÔNG PHẢI MỘT THAM SỐ MỖI-LƯỢT-GỌI RIÊNG (đổi ý sau lượt rà đầu).**
    /// Bản đầu của story này truyền đích như một tham số riêng cho MỖI vỏ cần nó — đúng khuôn
    /// `chapter_pattern`/`cleanup_rules` (nạp lại lúc xác nhận, không tin cache). Khuôn đó gãy
    /// đúng lúc NĂM vỏ giữa phiên (`reload_url_import_item`/`remove_url_import_item`/
    /// `tier2_block_set_kept`/`tier2_block_confirm_range`/`preview_chapter_detail`) cũng cần
    /// đúng giá trị đó để dựng LẠI màn xem trước đang hiện — không vỏ nào trong năm vỏ đó
    /// nhận một tham số "đích" từ frontend hôm nay, và việc thêm nó vào cả năm chỉ để chuyển
    /// tiếp một giá trị KHÔNG ĐỔI trong suốt phiên là chép năm lần một thứ vốn chỉ có MỘT giá
    /// trị đúng cho cả phiên. Khác `cleanup_rules` (luật có thể đổi GIỮA hai lượt gọi qua một
    /// cài đặt khác), đích của MỘT phiên xem trước là bất biến theo cấu trúc: không màn hình
    /// nào cho phép đổi đích giữa chừng một phiên (chọn đích là bước ĐẦU, trước khi xem
    /// trước bắt đầu). Vì thế nó thuộc về state CỦA PHIÊN, không phải tham số của LƯỢT GỌI —
    /// cùng lớp với `shape` chính nó.
    pub destination_work_id: Option<String>,
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
    /// `deferred-work.md §*Deferred from: 6-2-pipeline-nhap-mot-chuoi-thu-tu-co-dinh-dung-chung-moi-nguon (2026-09-04)*`, không trọn vẹn.** "LUÔN bằng `count_in_chapter`" đã HẾT ĐÚNG
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
    /// `rules` ở trên) — chỗ đóng nợ `deferred-work.md §*Deferred from: 6-2-pipeline-nhap-mot-chuoi-thu-tu-co-dinh-dung-chung-moi-nguon (2026-09-04)*` mà một phép đo tìm thấy được:
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
    /// **THÊM 2026-09-15 (Story 6.6b)** — tên/đường dẫn tệp NGUỒN của Chương này, echo thẳng
    /// từ [`crate::core::segment::import::ImportedChapter::source_file`]. `None` cho MỌI
    /// hình dạng KHÁC [`PipelineShape::Files`] hôm nay — xem doc-comment trường đó cho lý do
    /// đầy đủ. Tầng hiển thị (`ImportPreviewOverlay.vue`) hiện trường này làm nhãn "tệp nguồn"
    /// của mỗi hàng Chương khi có mặt.
    pub source_file: Option<String>,
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
    ///
    /// Always safe to display regardless of `any_signal_participated`: when no signal
    /// participates, `classify()` pushes no cause into any Chapter, so this count is
    /// always `0` and collapses to `broken_item_count`. Only
    /// [`ChapterSplitPreviewWire::clean_count`] needs to be gated on `any_signal_participated`.
    pub needs_review_count: usize,
    /// **THÊM 2026-09-08 (Story 6.10)** — `M` của chip — số Chương mang `needs_review ==
    /// false`. KHÔNG BAO GIỜ cộng `broken_item_count` (link hỏng không phải Chương, và không
    /// bao giờ "sạch" — nó hỏng).
    ///
    /// ⚠️ The one field gated by `any_signal_participated`: when `false`, this value
    /// ("every Chapter is clean") would be a false claim, since nothing was actually
    /// measured. `needs_review_count` needs no such gate — see its doc comment.
    pub clean_count: usize,
    /// **THÊM 2026-09-08 (Story 6.10)** — `true` khi ÍT NHẤT một trong ba tín hiệu so-tương-đối
    /// (`length`/`cleanup_match_count`/`joined_line_count`) có hàng rào tồn tại cho lượt nhập
    /// này (`core::segment::review::SignalParticipation::any`). `false` ⇒ KHÔNG tín hiệu nào
    /// tham gia (dưới bốn giá trị đo được cho CẢ ba, hoặc mọi hàng rào đều suy biến) — only
    /// `clean_count` must then not be read as "measured and clean" (the display layer must
    /// say "not enough Chapters to compare" instead of claiming `M clean`). `needs_review_count`
    /// is unaffected — see its own doc comment.
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
            source_file: c.source_file.clone(),
        })
        .collect();

    let needs_review_chapters = entries.iter().filter(|e| e.needs_review).count();
    // Guards the contract on `ChapterSplitPreviewWire::needs_review_count`: with no signal
    // participating, `classify()` must push no cause into any Chapter.
    debug_assert!(
        outcome.participation.any() || needs_review_chapters == 0,
        "khong tin hieu nao tham gia nhung van co Chuong needs_review=true -- vo hop dong needs_review_count"
    );
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

// ═════════════════════════════════════════════════════════════════════════════════
// Story 6.6b — nhập N tệp cùng lúc (FR14 mở rộng)
// ═════════════════════════════════════════════════════════════════════════════════
//
// Hình dạng DÂY copy KHUÔN của `UrlImportItemWire`/`UrlImportBatchWire` (§Decisions: "Copy
// the URL item shape") — nhưng KHÔNG một state thứ hai (`FileImportItemsState`): một tệp có
// thể ĐỌC LẠI (khác byte HTML đã tải qua mạng), nên `items` là DẪN XUẤT ở mỗi lượt gọi
// `wire::preview_import_encoding_from_file`, không phải một Mutex sống giữa hai lượt gọi (xem
// §Design Notes "Vì sao không state thứ hai cho mục tệp").

/// Hình dạng DÂY của một [`crate::core::segment::import::FileImportItem`] — vị trí là INDEX
/// trong `Vec` (frontend giữ nguyên thứ tự đã gửi), `error` mang `IpcError` ĐẦY ĐỦ để frontend
/// dịch bằng `tError()`, cùng khuôn [`UrlImportItemWire`].
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct FileImportItemWire {
    pub path: String,
    pub ok: bool,
    pub error: Option<IpcError>,
}

impl From<&crate::core::segment::import::FileImportItem> for FileImportItemWire {
    fn from(item: &crate::core::segment::import::FileImportItem) -> Self {
        FileImportItemWire {
            path: item.path.clone(),
            ok: item.error.is_none(),
            error: item.error.clone().map(IpcError::from),
        }
    }
}

/// Kết quả trả về của `wire::preview_import_encoding_from_file` — danh sách mục theo ĐÚNG thứ
/// tự `paths` cộng xem trước bảng mã, cho MỌI N (kể cả N = 1, §Always spec 6.6b: "one shape to
/// reason about"). `encoding_preview: None` là điều kiện ĐỦ để khoá nút xác nhận — khác
/// [`UrlImportBatchWire`] (ở đó `items` hỏng vẫn để lại `encoding_preview: Some(..)` cho các
/// mục CÒN TỐT), đường tệp đơn giản hơn: MỘT mục hỏng khoá TOÀN BỘ lượt xem trước (§Decisions:
/// "encoding_preview: None as the single sufficient condition for a locked confirm — the
/// display tier never derives that itself").
#[derive(Debug, Clone, serde::Serialize)]
pub struct FileImportBatchWire {
    pub items: Vec<FileImportItemWire>,
    pub encoding_preview: Option<ImportEncodingPreview>,
}

/// **THÊM 2026-09-16 (phản biện) — hàm THUẦN, tách khỏi `wire::preview_import_encoding_from_file`.**
/// Toàn bộ quyết định "mọi mục OK / khoá / cất" mà §Decisions spec 6.6b đòi (`encoding_preview:
/// None` là điều kiện ĐỦ để khoá xác nhận) từng chỉ sống TRONG vỏ IPC — không `tests/**` nào
/// gọi được nó không qua `tauri::AppHandle`, và mọi ca test đặt tên vỏ đó chỉ so văn bản mã
/// nguồn (không CHẠY nó). Đúng khuôn hai lớp `src-tauri/AGENTS.md`: hàm này nhận
/// `&FilesImportOutcome` VỪA ĐỌC (không tự đọc đĩa, không tự cất/dọn state) — chỗ gọi ở `mod
/// wire` chịu trách nhiệm CẢ HAI việc đó.
///
/// Trả `(FileImportBatchWire, bool)` — `bool` thứ hai là *"outcome.shape có nên được CẤT vào
/// PendingImportSourceState hay không"*: `true` ⇔ mọi mục `items[]` đều `ok` (chính điều kiện
/// mà `encoding_preview` cũng dùng, tính CHUNG một lần — không hai chỗ hỏi CÙNG một câu bằng
/// hai cách).
pub fn build_file_import_batch_wire(
    outcome: &crate::core::segment::import::FilesImportOutcome,
    source_lang: &str,
    cleanup_rules: &[CleanupRule],
    chapter_pattern: Option<&ChapterPattern>,
) -> (FileImportBatchWire, bool) {
    let all_ok = outcome.items.iter().all(|it| it.error.is_none());
    let encoding_preview = all_ok.then(|| {
        preview_import_encoding(&outcome.shape, source_lang, cleanup_rules, chapter_pattern, &[], 0, &[])
    });
    let items = outcome.items.iter().map(FileImportItemWire::from).collect();
    (FileImportBatchWire { items, encoding_preview }, all_ok)
}

/// Chạy chuỗi pipeline thật trên `shape` — **TOÀN Chương, KHÔNG cắt cửa sổ** — để tính khối
/// làm sạch của MỘT ứng viên/đường tự khai, chỗ đóng nợ `deferred-work.md §*Deferred from: 6-2-pipeline-nhap-mot-chuoi-thu-tu-co-dinh-dung-chung-moi-nguon (2026-09-04)*`.
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
/// `deferred-work.md §*Deferred from: 6-2-pipeline-nhap-mot-chuoi-thu-tu-co-dinh-dung-chung-moi-nguon (2026-09-04)*`.
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
/// đối chứng nợ `deferred-work.md §*Deferred from: 6-2-pipeline-nhap-mot-chuoi-thu-tu-co-dinh-dung-chung-moi-nguon (2026-09-04)*` tự cộng `per_rule_counts` NGAY TRONG ca test, so với
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
/// [`PipelineShape::Chapters`] (🟡 đóng MỘT PHẦN nợ `deferred-work.md §*Deferred from: 6-2-pipeline-nhap-mot-chuoi-thu-tu-co-dinh-dung-chung-moi-nguon (2026-09-04)*` — hình dạng đó
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
    // `deferred-work.md §*Deferred from: 6-2-pipeline-nhap-mot-chuoi-thu-tu-co-dinh-dung-chung-moi-nguon (2026-09-04)*`. Xem doc-comment `cleanup_and_chapters_preview_for` cho cách
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
/// khối làm sạch (tầng 3), đóng nợ `deferred-work.md §*Deferred from: 6-2-pipeline-nhap-mot-chuoi-thu-tu-co-dinh-dung-chung-moi-nguon (2026-09-04)*`.
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
    // Tham số `url_label` chỉ có nghĩa khi `extract_main_content`: `encoding::render_candidates`
    // cần nó để gọi `webimport::extract` cho MỖI ứng viên trên đường URL (bóc nội dung chính
    // TRƯỚC khi chuẩn hoá, xem doc-comment hàm đó) — nhánh `extract_main_content == false`
    // không đọc tới, luôn truyền `""`.
    let verdict_and_candidates = |bytes: &[u8],
                                   extract_main_content: bool,
                                   url_label: &str|
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
            encoding::render_candidates(bytes, source_lang, extract_main_content, url_label)
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
            verdict_and_candidates(bytes, false, "")
        }
        // 🔵 **SỬA 2026-09-08 (Story 6.10a) — "nuôi vào pipeline" đổi, "dò bảng mã" GIỮ
        // NGUYÊN ở `chapters.first()`.** §Always story 6.10a: dò bảng mã cho CẢ danh sách vẫn
        // chốt từ đơn vị ĐẦU (không đổi) — vế đổi là `verdict_and_candidates` giờ nạp THẲNG
        // `shape` (mang ĐỦ N đơn vị) vào `encoding_candidate_wire`, không còn gói lại chỉ đơn
        // vị đầu thành `Blob` (nguyên nhân ② của spec 6.10a — Chương 2..N từng KHÔNG BAO GIỜ
        // đi qua `run_pipeline` ở lượt xem trước đường URL).
        PipelineShape::Chapters(chapters) => match chapters.first() {
            Some(ChapterInput::RawBytes { bytes, label }) => {
                verdict_and_candidates(bytes, true, label)
            }
            Some(ChapterInput::AlreadyText(_)) | None => (self_declared_utf8(), Vec::new()),
        },
        // 🔴 **THÊM 2026-09-11 (Story 6.16)** — đường song ngữ có màn xem trước RIÊNG
        // (`preview_bilingual_import`), KHÔNG đi qua hàm này — cùng lý do `PipelineShape::Bilingual`
        // không xuất hiện trên đường sản phẩm gọi `preview_import_encoding`. Phòng thủ kiểu,
        // không phải một trạng thái người dùng gây ra được.
        PipelineShape::Bilingual { .. } => (self_declared_utf8(), Vec::new()),
        // **THÊM 2026-09-15 (Story 6.6b) — dò TRÊN TỪNG tệp, so sánh phán quyết.** §Decisions:
        // "One encoding for the whole batch, detected on EVERY file." Dải năm ô/bản dựng thật
        // vẫn dựng từ ĐÚNG MỘT tệp đại diện (tệp ĐẦU — cùng khuôn `Chapters` ngay trên: một
        // bảng mã cho cả danh sách, `verdict_and_candidates` đã nạp THẲNG `shape` đủ N đơn vị
        // vào `encoding_candidate_wire` qua closure bên ngoài) — vế ĐỔI là phán quyết tin cậy:
        // khi MỘT tệp bất kỳ (kể cả tệp đầu) tự dò ra một bảng mã KHÁC bảng mã đại diện, phán
        // quyết bị ép xuống `LowGuess` (⇒ `ConfidenceWire::Low`, mở dải năm ứng viên) — không
        // bao giờ âm thầm chọn bảng mã của tệp #1 mà không cho người dùng biết các tệp còn lại
        // bất đồng.
        PipelineShape::Files(units) => {
            // 🔴 SỬA 2026-09-16 (phản biện) — đại diện là đơn vị ĐẦU TIÊN CÓ BYTE, không
            // literally `units.first()`. Đo được: một batch 3 tệp mà tệp ĐẦU 0 byte cho
            // `candidates = 0`, `confidence = SelfDeclared`, KHÔNG tầng 4, và
            // `self_declared_chapters = Some(1)` — xem trước khai MỘT Chương cho BA tệp trong
            // khi xác nhận ghi BA, phá đúng hàng ma trận đông lạnh "Preview equals confirm"
            // bằng một lượt đếm THIẾU trong im lặng. `unwrap_or(0)` (không tệp nào có byte —
            // mọi tệp 0 byte) giữ nguyên hành vi CŨ (đơn vị 0), đúng ca duy nhất mà "0 byte" là
            // sự thật của CẢ batch, không phải của MỘT tệp lẻ loi.
            let representative_index = units
                .iter()
                .position(|u| matches!(u, ChapterInput::RawBytes { bytes, .. } if !bytes.is_empty()))
                .unwrap_or(0);
            let representative_bytes: &[u8] = match units.get(representative_index) {
                Some(ChapterInput::RawBytes { bytes, .. }) => bytes,
                // Bất khả trên đường sản phẩm (`import_files` chỉ đưa `.txt`/`.md` vào
                // `Files`, luôn `RawBytes`) — phòng thủ kiểu cho `tests/**`.
                Some(ChapterInput::AlreadyText(_)) | None => &[],
            };
            let (mut verdict, candidates) =
                verdict_and_candidates(representative_bytes, false, "");
            // 🔴 SỬA 2026-09-16 (phản biện) — MỘT nguồn sự thật cho bảng mã của đơn vị đại
            // diện: `verdict.encoding` (từ `verdict_and_candidates` ngay trên, tức
            // `encoding::detect(representative_bytes)`) — không hỏi lại `encoding::detect`
            // trên CHÍNH đơn vị đó lần thứ hai bằng cách lặp nó trong vòng `disagreement`
            // (bản trước lặp qua CẢ `units`, kể cả đơn vị đại diện, hỏi lại đúng câu
            // `verdict_and_candidates` vừa trả lời). Chỉ những đơn vị KHÁC đơn vị đại diện mới
            // cần một lượt `detect` của riêng chúng để so.
            let disagreement = units.iter().enumerate().any(|(i, u)| {
                if i == representative_index {
                    return false;
                }
                match u {
                    ChapterInput::RawBytes { bytes, .. } => encoding::detect(bytes).encoding != verdict.encoding,
                    ChapterInput::AlreadyText(_) => false,
                }
            });
            if disagreement {
                verdict.confidence = encoding::Confidence::LowGuess;
            }
            (verdict, candidates)
        }
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
///
/// 🔵 **THÊM 2026-09-16 (Story 6.6b, phản biện) — tham số `chapter_pattern`.** Nhánh `Files`
/// ngay dưới đọc nó để biết ánh xạ `chapter_index` → đơn vị có phải PHÉP ĐỒNG NHẤT hay không
/// — xem doc-comment nhánh đó.
fn display_window_for_chapter(
    shape: &PipelineShape,
    chapter_index: usize,
    encoding: &'static encoding_rs::Encoding,
    source_lang: &str,
    chapter_pattern: Option<&ChapterPattern>,
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
        // 🔵 **SỬA 2026-09-16 (Story 6.6b, phản biện) — không còn `None` VÔ ĐIỀU KIỆN.**
        //
        // ─────────────────────────────────────────────────────────────────────────────
        // 🔴 KHÔNG MẪU ⇒ ÁNH XẠ `chapter_index` ↔ ĐƠN VỊ LÀ PHÉP ĐỒNG NHẤT, KHÔNG PHẢI ĐOÁN
        // ─────────────────────────────────────────────────────────────────────────────
        // `split_unit_for_files` (bước 5 CỦA `Files`) trả ĐÚNG MỘT mảnh cho MỖI đơn vị khi
        // `chapter_pattern` là `None` — không nhánh nào khác thay đổi số đơn vị TRƯỚC bước 5
        // (bước 1-4 lặp per-unit, giữ nguyên N). Vì thế, trên đường KHÔNG mẫu, Chương thứ
        // `chapter_index` CHÍNH LÀ đơn vị thứ `chapter_index` — `units.get(chapter_index)` là
        // MỘT PHÉP TRA CỨU ĐÚNG, cùng khuôn `PipelineShape::Chapters` ngay trên (nơi ánh xạ
        // luôn đồng nhất, N đơn vị = N Chương). Đây chính là hành trình CHÍNH của story: N tệp,
        // không mẫu, mỗi tệp một Chương (§Always: "Story 6.10a holds per chapter: no Chapter
        // borrows another unit's number, at any tier") — trả `None` vô điều kiện ở đây từng
        // phá đúng dòng đó cho hành trình chính, không chỉ một cạnh hiếm.
        //
        // CÓ mẫu: một đơn vị CÓ THỂ bị tách thành k > 1 mảnh (`split_on_positions` bên trong
        // `split_unit_for_files`), và ranh giới đó chỉ tính được SAU khi giải mã + so mẫu —
        // hàm này chạy TRƯỚC pipeline, trên byte thô của TỪNG tệp, nên ánh xạ `chapter_index`
        // (chỉ số Chương ĐẦU RA) về đúng tệp/mảnh đòi lặp lại chính phép so mẫu đó Ở TẦNG NÀY.
        // Nhánh này VẪN `None` — nợ CÓ CHỦ hẹp lại đúng ca này (`deferred-work.md`), không phải
        // phạm vi story này. Tier 4 (title/length/review) không đọc hàm này — nó đã EAGER cho
        // MỌI Chương, kể cả khi có mẫu.
        PipelineShape::Files(units) => match chapter_pattern {
            None => window_for_unit(units.get(chapter_index)?, encoding, source_lang),
            Some(_) => None,
        },
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
        display_window_for_chapter(shape, chapter_index, encoding, source_lang, chapter_pattern)?;
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
    stash_pending_import_source_for(state, shape, docx_sidecar, None);
}

/// **Hàm thuần** — [`stash_pending_import_source`] cộng đích. **THÊM 2026-09-16 (Story
/// 6.7b)** — xem doc-comment [`PendingImportSource::destination_work_id`]. Tách khỏi
/// [`stash_pending_import_source`] (thay vì thêm tham số vào chính nó) để giữ chữ ký GỐC ổn
/// định cho mọi chỗ gọi chưa mang khái niệm đích — 35+ lời gọi trực tiếp trong `tests/**`
/// (`cleanup_contract.rs`/`bilingual_import_contract.rs`/`segment_contract.rs`/
/// `project_contract.rs`/`story_6_18_library.rs`) đi thẳng vào hàm thuần này (đúng khuôn hai
/// lớp — không cần `tauri::AppHandle`), và thêm một tham số bắt buộc vào nó sẽ là một lượt
/// đổi DÂY phá vỡ TOÀN BỘ những chỗ gọi đó, không liên quan gì tới đích của story này. Cùng
/// lý lẽ đúng khuôn `resolve_cleanup_rules`/`resolve_cleanup_rules_for` (`wire.rs`): hàm GỐC
/// giữ nguyên chữ ký, hàm MỚI cộng thêm.
///
/// Ba lượt MỞ phiên (dán/tệp/URL) truyền giá trị người dùng vừa chọn; mọi lượt gọi lại TRONG
/// một phiên đã mở (đổi mục URL, đổi override) phải truyền LẠI giá trị ĐÃ CÓ trong state —
/// đọc qua [`current_pending_destination`] trước khi gọi hàm này — không phải một giá trị
/// mới, nếu không đích sẽ "quên" giữa hai lượt gọi giữa phiên.
pub fn stash_pending_import_source_for(
    state: &PendingImportSourceState,
    shape: PipelineShape,
    docx_sidecar: Option<crate::core::segment::import::DocxSidecar>,
    destination_work_id: Option<String>,
) {
    let mut guard = state.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
    *guard = Some(PendingImportSource { shape, docx_sidecar, destination_work_id });
}

/// **Hàm thuần** — đích ĐANG CẤT của lượt xem trước hiện tại (Story 6.7b), nếu có. `None` khi
/// không có lượt xem trước nào đang treo, hoặc phiên đang treo nhắm Tác phẩm MỚI. Chỉ ĐỌC,
/// không đổi gì trong `state`. Dùng bởi các vỏ giữa phiên (dựng lại dây sau reload/remove một
/// mục URL, đổi override khối tầng 2, dời con trỏ Chương xem trước) và bởi
/// `confirm_import_with_encoding` để biết đích mà không cần frontend gửi lại một tham số
/// riêng cho lượt xác nhận (xem doc-comment [`PendingImportSource::destination_work_id`] cho
/// lý do đổi từ "tham số mỗi lượt gọi" sang "state của phiên").
pub fn current_pending_destination(state: &PendingImportSourceState) -> Option<String> {
    let guard = state.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
    guard.as_ref().and_then(|p| p.destination_work_id.clone())
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
/// vừa dùng để hiện — đây là chỗ đóng nợ `deferred-work.md §*Deferred from: 6-2-pipeline-nhap-mot-chuoi-thu-tu-co-dinh-dung-chung-moi-nguon (2026-09-04)*` cho NỬA GHI: `create_work`
/// nhận đúng luật đó, không một bộ luật thứ hai.
/// 🔴 **THÊM 2026-09-07 (Story 6.9) — tham số `block_overrides`, đúng cái vòng rà 1 đã hụt.**
/// `wire::confirm_import_with_encoding` đọc `Tier2BlockOverridesState` NGAY LÚC XÁC NHẬN
/// (cùng kỷ luật "đọc lại lúc xác nhận, không tái dùng bộ lúc xem trước" mà `cleanup_rules`
/// đã theo) rồi truyền vào đây; state đó chỉ được RESET ở lớp vỏ SAU KHI hàm này trả `Ok`.
/// 🔴 **THÊM 2026-09-08 (Story 6.11, mục B1) — tham số `domain_log_state`.** Đọc doc-comment
/// `create_work` — thread thẳng xuống đó, không tự tích luỹ gì ở tầng này.
pub fn confirm_import_with_encoding_with_progress(
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
    on_image_progress: &mut dyn FnMut(usize, usize),
    should_cancel: &dyn Fn() -> bool,
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

    let opened = create_work_with_progress(
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
        // Đường văn xuôi/URL không bao giờ mang `PipelineShape::Bilingual` — 0 regrouping.
        &[],
        on_image_progress,
        should_cancel,
    )?;

    // Thành công — dọn ô đang chờ, VẪN dưới CÙNG một khoá đã giữ từ đầu hàm.
    *guard = None;

    Ok(opened)
}

/// Thin wrapper over [`confirm_import_with_encoding_with_progress`] with a no-op
/// progress/cancel pair, for callers that don't need either.
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
    origin_overrides: Vec<Option<ChapterOriginOverride>>,
    domain_log_state: &webimport::DomainLogState,
) -> Result<OpenWork, IpcError> {
    confirm_import_with_encoding_with_progress(
        documents_root,
        state,
        name,
        source_lang,
        genre,
        encoding_wire_id,
        cleanup_rules,
        chapter_pattern,
        block_overrides,
        origin_overrides,
        domain_log_state,
        &mut |_, _| {},
        &|| false,
    )
}

/// **Hàm thuần** — lõi của lượt xác nhận đường APPEND (Story 6.7b, FR122 nửa hai). Cùng
/// khuôn [`confirm_import_with_encoding`] ngay trên (giải `encoding_wire_id`, CLONE nguồn
/// đang chờ, dọn ô đang chờ CHỈ KHI thành công), nhưng gọi [`append_chapters_to_work`] +
/// [`crate::commands::lifecycle::write_lifecycle_after_change`] — bước 1-2-3 của khuôn bốn
/// bước AD-8 (`ARCHITECTURE-SPINE.md`) — thay vì [`create_work`]. Bước 4
/// (`reindex_after_lifecycle_write`) chạy Ở LỚP VỎ, sau khi khoá `OpenWorkState` (nếu đích
/// trùng Tác phẩm đang mở) đã nhả — cùng kỷ luật
/// `commands::lifecycle::wire::set_chapter_status`.
///
/// `open` là đích ĐÃ PHÂN GIẢI — chỗ gọi (`wire::confirm_import_with_encoding`) chịu trách
/// nhiệm lấy nó ra (tái dùng `OpenWorkState` hoặc [`open_work`], xem doc-comment
/// [`append_chapters_to_work`]).
///
/// # Lỗi
/// Y hệt [`confirm_import_with_encoding`] cộng lỗi của [`append_chapters_to_work`]/
/// `write_lifecycle_after_change`; KHÔNG một nhánh nào xoá `open` (§Never spec 6.7b) — một
/// lỗi để `open` NGUYÊN VẸN, ô đang chờ GIỮ NGUYÊN để thử lại với một ứng viên bảng mã khác.
pub fn confirm_append_import_with_encoding_with_progress(
    open: &mut OpenWork,
    state: &PendingImportSourceState,
    source_lang: &str,
    encoding_wire_id: &str,
    cleanup_rules: Vec<CleanupRule>,
    chapter_pattern: Option<ChapterPattern>,
    block_overrides: Vec<Option<bool>>,
    origin_overrides: Vec<Option<ChapterOriginOverride>>,
    domain_log_state: &webimport::DomainLogState,
    on_image_progress: &mut dyn FnMut(usize, usize),
    should_cancel: &dyn Fn() -> bool,
) -> Result<(), IpcError> {
    let chosen = encoding::encoding_for_wire_id(encoding_wire_id).ok_or_else(|| {
        IpcError::from(ImportError::UnrecognizedEncoding { wire_id: encoding_wire_id.to_owned() })
    })?;

    // Cùng lý lẽ khoá xuyên suốt của `confirm_import_with_encoding` ngay trên (vòng rà đối
    // kháng 2, mục 14) — giữ NGUYÊN một `MutexGuard` cho trọn phần đọc-rồi-ghi.
    let mut guard = state.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
    let shape = guard.as_ref().map(|p| p.shape.clone()).ok_or_else(no_pending_import_source)?;
    let docx_sidecar = guard.as_ref().and_then(|p| p.docx_sidecar.clone());

    append_chapters_to_work_with_progress(
        open,
        source_lang,
        shape,
        chosen,
        cleanup_rules,
        chapter_pattern,
        block_overrides,
        &origin_overrides,
        domain_log_state,
        docx_sidecar,
        on_image_progress,
        should_cancel,
    )?;

    // Thanh cong -- don o dang cho, VAN duoi CUNG mot khoa da giu tu dau ham.
    *guard = None;
    drop(guard);

    // Buoc 2-3 cua khuon bon buoc AD-8 -- cap nhat open.meta tai cho.
    crate::commands::lifecycle::write_lifecycle_after_change(open)?;
    Ok(())
}

/// Thin wrapper over [`confirm_append_import_with_encoding_with_progress`] with a no-op
/// progress/cancel pair.
pub fn confirm_append_import_with_encoding(
    open: &mut OpenWork,
    state: &PendingImportSourceState,
    source_lang: &str,
    encoding_wire_id: &str,
    cleanup_rules: Vec<CleanupRule>,
    chapter_pattern: Option<ChapterPattern>,
    block_overrides: Vec<Option<bool>>,
    origin_overrides: Vec<Option<ChapterOriginOverride>>,
    domain_log_state: &webimport::DomainLogState,
) -> Result<(), IpcError> {
    confirm_append_import_with_encoding_with_progress(
        open,
        state,
        source_lang,
        encoding_wire_id,
        cleanup_rules,
        chapter_pattern,
        block_overrides,
        origin_overrides,
        domain_log_state,
        &mut |_, _| {},
        &|| false,
    )
}

/// **THÊM 2026-09-16 (Story 6.7b, Phase 4 — vòng rà của coordinator, đối chứng đỏ ① của
/// đường APPEND)** — [`confirm_append_import_with_encoding`] CỘNG bước 4, đúng khuôn
/// [`crate::commands::lifecycle::set_chapter_status_indexed`]/
/// `crate::commands::chapter::merge_chapter_into_previous_indexed`: hàm THUẦN, không
/// `AppHandle`, để `tests/**` gọi được thẳng và chứng minh bước 4 THẬT SỰ chạy sau một lượt
/// xác nhận append thành công — không phải một lượt `Indexer::rebuild` do chính test tự gọi
/// rời (đúng cái bẫy "tự reindex thì không canh gì" mà correction của Phase 1 đã gọi tên,
/// áp lại cho bước 4 thay vì AC3).
///
/// Uỷ thác TRỌN quyết định "chỉ reindex khi ghi thành công" cho
/// [`crate::commands::lifecycle::finish_lifecycle_write`] — vị từ `is_ok()` khai ĐÚNG MỘT
/// CHỖ, dùng chung với mọi `*_indexed` khác trong kho, không một bản chép tay thứ hai.
///
/// ⚠️ **Giới hạn ghi thẳng, không giấu**: đây là seam THUẦN cho `tests/**`, cùng vai trò
/// `set_chapter_status_indexed` đã giữ từ trước.
///
/// 🔵 **SỬA 2026-09-16 (vòng rà, mục B9) — câu dưới đây từng đúng, hết đúng từ bản sửa này.**
/// Bản viết đầu nói hàm này "KHÔNG phải hàm mà `wire::confirm_import_with_encoding` gọi theo
/// đúng tên" ở CẢ HAI nhánh — đo lại: điều đó chỉ đúng cho nhánh Tác phẩm ĐANG MỞ (giữ một
/// `MutexGuard` của `OpenWorkState` suốt lượt ghi, nên gọi hàm NÀY ở đó sẽ kéo dài thời gian
/// giữ khoá qua một lượt quét đĩa toàn Library, đúng lớp rủi ro mà `wire::set_chapter_status`'s
/// doc-comment đã ghi tên — nhánh đó VẪN tự soạn `confirm_append_import_with_encoding` rồi một
/// lời gọi `reindex_library` RIÊNG, SAU khi khoá đã nhả). Nhánh CÒN LẠI (đích chưa mở — `Store`
/// vừa mở riêng từ chỉ mục, không một `MutexGuard` nào đang giữ) không mang rủi ro đó, và từ
/// bản sửa 2026-09-16 (B9: hàm này trước đó có 1 định nghĩa + 6 chỗ gọi test + **0** chỗ gọi
/// sản phẩm — case canh nó chỉ chứng minh một hàm song song, không chứng minh gì về đường sản
/// phẩm thật) gọi ĐÚNG hàm này, xem `wire::confirm_import_with_encoding`. Một mutation xoá dòng
/// gọi `reindex_library(&app, &root)` ở nhánh Tác phẩm ĐANG MỞ vẫn KHÔNG bị hàm này/case dùng
/// nó bắt được — không đường nào trong `tests/**` gọi được thẳng một vỏ `#[tauri::command]`
/// (không `tauri::test`/`MockRuntime` trong kho, đo lại ở Phase 1). Cái hàm này ĐÓNG là seam
/// "bước 4 của đường append CÓ TỒN TẠI và THẬT SỰ cập nhật chỉ mục" — seam mà trước Phase 4
/// KHÔNG tồn tại ở BẤT KỲ hình dạng nào cho đường append (khác hẳn
/// `set_chapter_status`/`merge_chapter_into_previous`, cả hai đã có `*_indexed` từ trước).
pub fn confirm_append_import_with_encoding_indexed(
    open: &mut OpenWork,
    state: &PendingImportSourceState,
    source_lang: &str,
    encoding_wire_id: &str,
    cleanup_rules: Vec<CleanupRule>,
    chapter_pattern: Option<ChapterPattern>,
    block_overrides: Vec<Option<bool>>,
    origin_overrides: Vec<Option<ChapterOriginOverride>>,
    domain_log_state: &webimport::DomainLogState,
    indexer: Option<&crate::core::library::indexer::Indexer>,
    global: Option<&Store>,
    root: &std::path::Path,
) -> Result<(), IpcError> {
    confirm_append_import_with_encoding_indexed_with_progress(
        open,
        state,
        source_lang,
        encoding_wire_id,
        cleanup_rules,
        chapter_pattern,
        block_overrides,
        origin_overrides,
        domain_log_state,
        indexer,
        global,
        root,
        &mut |_, _| {},
        &|| false,
    )
}

/// Same shape as [`confirm_append_import_with_encoding_indexed`] plus real image
/// progress/cancel wiring; production callers that need it use this one, tests keep
/// calling the no-op wrapper.
#[allow(clippy::too_many_arguments)]
pub fn confirm_append_import_with_encoding_indexed_with_progress(
    open: &mut OpenWork,
    state: &PendingImportSourceState,
    source_lang: &str,
    encoding_wire_id: &str,
    cleanup_rules: Vec<CleanupRule>,
    chapter_pattern: Option<ChapterPattern>,
    block_overrides: Vec<Option<bool>>,
    origin_overrides: Vec<Option<ChapterOriginOverride>>,
    domain_log_state: &webimport::DomainLogState,
    indexer: Option<&crate::core::library::indexer::Indexer>,
    global: Option<&Store>,
    root: &std::path::Path,
    on_image_progress: &mut dyn FnMut(usize, usize),
    should_cancel: &dyn Fn() -> bool,
) -> Result<(), IpcError> {
    crate::commands::lifecycle::finish_lifecycle_write(
        confirm_append_import_with_encoding_with_progress(
            open,
            state,
            source_lang,
            encoding_wire_id,
            cleanup_rules,
            chapter_pattern,
            block_overrides,
            origin_overrides,
            domain_log_state,
            on_image_progress,
            should_cancel,
        ),
        indexer,
        global,
        root,
    )
}



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
            Some(v) => webimport::chapter_origin_trim_or_none(&v),
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

/// Từ chối mở một `project.db` mà cột `segment.translation_origin` mang một giá trị NGOÀI
/// danh mục đóng [`TRANSLATION_ORIGINS`].
///
/// [`SEGMENT_DDL`] không có `CHECK` trên cột này (cùng lý do cột `status`, xem doc-comment
/// của nó), và [`Store::open`]'s [`StoreError::SchemaTooNew`] chỉ so SỐ BƯỚC di trú: thêm
/// một giá trị thứ năm không cần một bước di trú nào, nên `PRAGMA user_version` không đổi
/// và ca đó trượt sạch qua vế đó. Đây là lớp chặn DUY NHẤT ở tầng đọc, gọi ngay sau
/// [`Store::open`] trước khi bất kỳ hàng nào khác được đọc/ghi.
///
/// [`TRANSLATION_ORIGINS`]: crate::commands::segment::TRANSLATION_ORIGINS
/// [`SEGMENT_DDL`]: crate::core::store::SEGMENT_DDL
fn reject_unknown_translation_origin(
    store: &Store,
) -> Result<(), crate::core::store::StoreError> {
    use crate::commands::segment::TRANSLATION_ORIGINS;
    use crate::core::store::{StoreError, StoreKind, params_from_iter};

    let placeholders = (1..=TRANSLATION_ORIGINS.len())
        .map(|i| format!("?{i}"))
        .collect::<Vec<_>>()
        .join(", ");
    let found: Option<String> = store.read(|conn| {
        let mut stmt = conn.prepare(&format!(
            "SELECT DISTINCT translation_origin FROM segment \
             WHERE translation_origin NOT IN ({placeholders}) LIMIT 1",
        ))?;
        let mut rows = stmt.query(params_from_iter(TRANSLATION_ORIGINS.iter()))?;
        match rows.next()? {
            Some(row) => Ok(Some(row.get::<_, String>(0)?)),
            None => Ok(None),
        }
    })?;

    match found {
        Some(value) => Err(StoreError::UnknownTranslationOrigin { store: StoreKind::Project, value }),
        None => Ok(()),
    }
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
///   `From<StoreError>`;
/// - `project.db` mở được nhưng cột `segment.translation_origin` mang một giá trị NGOÀI
///   danh mục đóng `TRANSLATION_ORIGINS` ⇒ `store.unknown_translation_origin`, không một
///   byte nào bị ghi — xem doc-comment của [`reject_unknown_translation_origin`].
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

    // 🔴 TỪ CHỐI MỞ một `project.db` mang một `translation_origin` NGOÀI
    // danh mục đóng `TRANSLATION_ORIGINS`. `SEGMENT_DDL` không có `CHECK` trên cột này
    // (cùng lý do cột `status`, xem doc-comment của nó), và `SchemaTooNew` chỉ so SỐ BƯỚC
    // di trú — thêm một giá trị thứ năm không cần một bước di trú nào, nên nó không đổi
    // `PRAGMA user_version` và trượt sạch qua vế đó. Đây là lớp chặn DUY NHẤT ở tầng đọc.
    if let Err(err) = reject_unknown_translation_origin(&store) {
        store.close();
        return Err(err.into());
    }

    // Mở lại đúng `work.last_chapter_id` nếu hàng đó CÒN SỐNG trong `chapter` (một Chương đã
    // bị xoá/gộp sau khi được lưu ở đó để lại một giá trị "cũ", đọc được nhưng không hợp lệ --
    // xem doc-comment `WORK_LAST_CHAPTER_ID_DDL`); `NULL` hoặc cũ ⇒ rơi về Chương đầu theo
    // `(ord, id)`, hành vi gốc trước khi cột này tồn tại.
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
        let last_chapter_id: Option<i64> =
            conn.query_row("SELECT last_chapter_id FROM work WHERE id = 1", [], |row| row.get(0))?;

        if let Some(last_id) = last_chapter_id {
            let mut stmt = conn.prepare("SELECT id FROM chapter WHERE id = ?1")?;
            let mut rows = stmt.query_map([last_id], |row| row.get::<_, i64>(0))?;
            if let Some(id) = rows.next().transpose()? {
                return Ok(Some(id));
            }
        }

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
    Ok(OpenWork {
        dir,
        store,
        scope,
        meta,
        chapter_id,
        images_saved: 0,
        images_failed: 0,
        // Mở lại một Work đã có không ghi/thêm chữ mới -- không gì để mà so voi source_lang.
        source_lang_mismatch: false,
        // Mở lại không ghi Chương nào -- rỗng, cùng lý do ngay trên.
        new_chapter_ids: Vec::new(),
    })
}

/// Giải đích cho đường APPEND (Story 6.7b) — bọc [`open_work`], KHÔNG thay hành vi của nó.
///
/// 🔵 **THÊM 2026-09-16 (Ice, qua vòng rà) — sửa MÃ, không sửa ma trận.** Đo được: một
/// `project.db` HỎNG (tệp còn đó, nội dung không còn là một CSDL SQLite hợp lệ) đi qua
/// `Store::open`'s `PRAGMA user_version` trước khi `open_work` kịp phân biệt loại lỗi, nên
/// nó trả về `StoreError::OpenFailed`/`store.open_failed`/`MessageKey::StoreOpenFailed` — một
/// mã tầng-kho, không nêu tên Tác phẩm. Ice chốt: trên đường APPEND, câu báo phải gọi đúng
/// tên Tác phẩm người dùng VỪA CHỌN làm đích, nên một đích hỏng phải nổi lên bằng
/// [`crate::core::library::WorkError::OpenFailed`] (`work.open_failed`) — không phải mã kho.
///
/// Đây là lý do hàm này tồn tại thay vì sửa `open_work`: `open_work` còn một chỗ gọi KHÁC
/// (`wire::open_work`, Story 5.7, "mở lại một `.atproj` đã có trên đĩa" từ Library) mà mã
/// kho vẫn đúng ngữ cảnh — đổi tại nguồn sẽ đổi cả câu báo của đường đó, thứ Ice không yêu
/// cầu. Chỉ đường APPEND đổi.
///
/// `meta.json` schema quá mới vẫn đi qua KHÔNG ĐỔI — `open_work` đã trả
/// [`crate::core::library::WorkError::MetaTooNew`] cho ca đó trước khi `Store::open` bao giờ
/// chạy, nên hàm này không có gì để bọc lại ở đó.
pub fn open_destination_for_append(
    work_id: &str,
    indexed: Option<&crate::core::library::indexer::IndexedWork>,
) -> Result<OpenWork, IpcError> {
    open_work(work_id, indexed).map_err(|err| {
        if err.code() == "store.open_failed" {
            let name = indexed.map(|w| w.name.clone()).unwrap_or_else(|| work_id.to_owned());
            crate::core::library::WorkError::OpenFailed { name, detail: format!("{err:?}") }.into()
        } else {
            err
        }
    })
}

/// Kiểu state Tauri quản lý — Tác phẩm đang mở, hoặc chưa mở gì (Task 7).
///
/// ⚠️ `Mutex`, không `RwLock`: đúng một Tác phẩm mở tại một thời điểm, và mọi thao tác
/// đọc/ghi field của nó (thay Tác phẩm khác, đóng lúc thoát) đều là **thao tác độc quyền**
/// — không có nhánh "nhiều reader cùng lúc" nào ở tầng state này (khác hẳn `Store::read`
/// bên trong, nơi pool nhiều kết nối đã lo phần đó).
pub type OpenWorkState = std::sync::Mutex<Option<OpenWork>>;

/// `work_id`s with an APPEND currently writing into a Work that is not the open editor
/// Work, so `replace_open_work`'s orphan sweep can skip them instead of racing the write.
pub type AppendInProgressState = std::sync::Mutex<std::collections::HashSet<String>>;

/// Pure decision over an ALREADY-LOCKED set — the caller holds the `AppendInProgressState`
/// lock across both this check and whatever it gates, so the answer cannot go stale before
/// the caller acts on it.
pub fn should_skip_orphan_sweep(in_progress: &std::collections::HashSet<String>, work_id: &str) -> bool {
    in_progress.contains(work_id)
}

/// Removes `work_id` from [`AppendInProgressState`] on every exit path, including `?`.
pub struct AppendInProgressGuard<'a> {
    state: &'a AppendInProgressState,
    work_id: String,
}

impl<'a> AppendInProgressGuard<'a> {
    pub fn new(state: &'a AppendInProgressState, work_id: String) -> Self {
        let mut set = state.lock().unwrap_or_else(|e| e.into_inner());
        set.insert(work_id.clone());
        drop(set);
        Self { state, work_id }
    }
}

impl Drop for AppendInProgressGuard<'_> {
    fn drop(&mut self) {
        let mut set = self.state.lock().unwrap_or_else(|e| e.into_inner());
        set.remove(&self.work_id);
    }
}

/// Runs `sweep` while holding the `AppendInProgressState` lock for the whole check-then-sweep
/// span — `AppendInProgressGuard::new` locks the same `Mutex`, so it cannot register (and let
/// its APPEND start writing into `assets_dir`) until `sweep` has returned. Skips `sweep`
/// (without running it) when `append_state` is `None` (fails CLOSED: an unmanaged state means
/// an in-flight APPEND cannot be ruled out) or when `work_id` is already registered.
fn sweep_orphans_holding_the_append_lock(
    append_state: Option<&AppendInProgressState>,
    work_id: &str,
    assets_dir: &std::path::Path,
    sweep: impl FnOnce(),
) {
    let Some(state) = append_state else {
        eprintln!(
            "project[orphan] AppendInProgressState chua duoc quan ly, bo qua luot quet mo coi cho {}",
            assets_dir.display()
        );
        return;
    };
    let in_progress = state.lock().unwrap_or_else(|e| e.into_inner());
    if should_skip_orphan_sweep(&in_progress, work_id) {
        eprintln!(
            "project[orphan] bo qua luot quet mo coi cho {} - mot luot APPEND dang do dang",
            assets_dir.display()
        );
        return;
    }
    sweep();
}

// A missing assets_dir (no images ever downloaded) returns 0, not an error. Only regular
// files are removed; a failed removal is logged and skipped, not fatal to the sweep.
fn sweep_orphaned_asset_files(assets_dir: &std::path::Path, referenced: &std::collections::BTreeSet<String>) -> usize {
    let Ok(entries) = std::fs::read_dir(assets_dir) else { return 0 };
    let mut removed = 0usize;
    for entry in entries.flatten() {
        let Ok(file_type) = entry.file_type() else { continue };
        if !file_type.is_file() {
            continue;
        }
        let name = entry.file_name();
        let Some(name_str) = name.to_str() else { continue };
        if referenced.contains(name_str) {
            continue;
        }
        match std::fs::remove_file(entry.path()) {
            Ok(()) => removed += 1,
            Err(err) => eprintln!("project[orphan] khong xoa duoc {}: {err}", entry.path().display()),
        }
    }
    removed
}

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

    // Sweeps assets/ for files no `asset` row names, before the Work accepts any write.
    //
    // 🔴 A failed read of the `asset` table skips the sweep entirely — sweeping with an
    // empty referenced set would delete every real asset file.
    let append_state = app.try_state::<AppendInProgressState>();
    sweep_orphans_holding_the_append_lock(append_state.as_deref(), &new_work.meta.work_id, &assets_dir, || {
        match new_work.store.read(|conn| {
            let mut stmt = conn.prepare("SELECT file_name FROM asset")?;
            let rows = stmt.query_map([], |r| r.get::<_, String>(0))?;
            rows.collect::<crate::core::store::SqlResult<std::collections::BTreeSet<String>>>()
        }) {
            Ok(referenced) => {
                sweep_orphaned_asset_files(&assets_dir, &referenced);
            }
            Err(err) => {
                eprintln!(
                    "project[orphan] khong doc duoc bang asset cho {}, bo qua luot quet mo coi: {err:?}",
                    assets_dir.display()
                );
            }
        }
    });

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
    // Story 4.5 -- cung ly do ngay tren, cho lo nhap bo prompt dang treo (neu co): chi ha
    // `work_kind` ve `None`, khong xoa TRON lo (`lib.rs::close_open_work` giai thich vi sao).
    if let Some(pending) = app.try_state::<crate::commands::promptset::PendingPromptImportState>() {
        crate::commands::promptset::clear_pending_prompt_import_work_tier(&pending);
    }
    // 🔴 Story 4.7, finding V1 (loop 1) -- cung ly do BA nguoi lang gieng ngay tren, cho ban
    // ghi prompt da lap cua Tac pham CU (neu co): `segment_id`/`chapter_id` la khoa hang cua
    // CHINH `project.db` sap bi thay the boi `new_work`, va hai `.atproj` khac nhau deu danh
    // so lai tu 1 -- de ban ghi song qua lan swap nay se doc sai "dung Chuong/segment nay" cho
    // mot Tac pham no chua tung thay (xem doc-comment day du o
    // `commands::aiprompt::clear_last_assembled_prompt_on_work_close`, cung mot ham, chi khac
    // DIEM GOI: `close_open_work` la dong HAN TOAN khong con Tac pham nao mo; day la THAY THE
    // mot Tac pham nay bang mot Tac pham KHAC ma khong di qua trang thai "khong Tac pham" o
    // giua -- ca hai duong deu phai xoa ban ghi cu.
    if let Some(record) = app.try_state::<crate::commands::aiprompt::LastAssembledPromptState>() {
        crate::commands::aiprompt::clear_last_assembled_prompt_on_work_close(&record);
    }

    if let Some(state) = app.try_state::<OpenWorkState>() {
        let old = swap_locked(&state, new_work);
        // Revoke the outgoing Work's `asset://` scope (AD-23: one Work at a time).
        // `forbid_directory` runs after the new Work's `allow_directory` above.
        let old_assets_dir_owned = old.as_ref().map(|w| w.dir.join("assets"));
        if let Some(old_assets_dir) = assets_dir_to_forbid(old_assets_dir_owned.as_deref(), &assets_dir) {
            if let Err(err) = app.asset_protocol_scope().forbid_directory(old_assets_dir, true) {
                eprintln!(
                    "project[scope] khong thu hoi duoc asset_protocol_scope cho {}: {err}",
                    old_assets_dir.display()
                );
            }
        }
        drop(old);
    }
}

// Returns None both when there is no old Work and when the old Work is the one just
// re-allowed (reopening itself), so the just-granted allow is never immediately revoked.
fn assets_dir_to_forbid<'a>(
    old_assets_dir: Option<&'a std::path::Path>,
    newly_allowed_assets_dir: &std::path::Path,
) -> Option<&'a std::path::Path> {
    old_assets_dir.filter(|old| *old != newly_allowed_assets_dir)
}

#[cfg(test)]
mod scope_revocation_tests {
    use super::assets_dir_to_forbid;
    use std::path::Path;

    #[test]
    fn no_old_work_means_nothing_to_forbid() {
        assert_eq!(assets_dir_to_forbid(None, Path::new("/lib/B.atproj/assets")), None);
    }

    #[test]
    fn a_different_old_work_is_forbidden() {
        let old = Path::new("/lib/A.atproj/assets");
        assert_eq!(assets_dir_to_forbid(Some(old), Path::new("/lib/B.atproj/assets")), Some(old));
    }

    #[test]
    fn reopening_the_same_work_forbids_nothing() {
        let dir = Path::new("/lib/A.atproj/assets");
        assert_eq!(assets_dir_to_forbid(Some(dir), dir), None, "khong duoc huy ngay luot allow vua cap cho CHINH no");
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
mod tests;

/// Nhiều vỏ `#[tauri::command]`. **Không một quy tắc nào sống ở đây.**
pub mod wire;

/// Ba khối tự chứa nhấc ra khỏi thân tệp này, không đổi hành vi. `pub use` bên dưới giữ
/// nguyên mọi đường dẫn `super::X`/`crate::commands::project::X` mà
/// `wire.rs`/`tests.rs`/`tests/project_contract.rs` đã dùng trước lượt tách.
mod bilingual;
mod url_import;
mod work_creation;
pub use bilingual::*;
pub use url_import::*;
pub use work_creation::*;
