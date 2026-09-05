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
use crate::core::i18n::IpcError;
use crate::core::library::{WorkMeta, create_work_folder, remove_folder};
use crate::core::lifecycle::LifecycleStatus;
use crate::core::scope::load_global_config;
use crate::core::segment::encoding::{
    self, Confidence, EncodingCandidate, EncodingVerdict, NormalizedCandidate,
};
use crate::core::segment::import::{ImportError, import_file, import_text};
use crate::core::segment::pipeline::{ChapterInput, PipelineInput, PipelineShape, run_import};
use crate::core::store::{Store, StoreSpec, Transaction};

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
/// đơn lẻ; ghi N Chương, không còn đúng một.** N = 1 trên đường sản phẩm hôm nay (chuỗi
/// khai `chapter_pattern: None` — Never clause của spec 6.2), nên hành vi quan sát được
/// KHÔNG đổi; đường đi đã tổng quát cho Story 6.6/6.7 (N > 1) mà không cần sửa lại hàm này.
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
pub fn create_work(
    documents_root: &Path,
    name: &str,
    source_lang: &str,
    genre: &str,
    shape: PipelineShape,
    encoding: &'static encoding_rs::Encoding,
    cleanup_rules: Vec<crate::core::cleanup::CleanupRule>,
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
    // 🔴 `chapter_pattern: None` — Never clause của spec 6.2: mẫu phân tách NGƯỜI DÙNG cấu
    // hình được là Story 6.6; sản phẩm hôm nay không có bề mặt nào đưa một mẫu vào, nên
    // bước 5 của chuỗi luôn là no-op và N luôn là 1, đúng hành vi hôm nay.
    //
    // 🔵 SỬA 2026-09-04 (Story 6.3) — `with_encoding`, không còn `default_shaped` cứng
    // UTF-8: `encoding` giờ là tham số của chính `create_work` (xem doc-comment hàm này).
    // 🔵 SỬA 2026-09-05 (Story 6.5) — qua `run_pipeline` (không gọi `run_import` thẳng ở
    // đây nữa — xem doc-comment của hàm đó), cộng `cleanup_rules` đã phân giải.
    let outcome = match run_pipeline(
        PipelineInput::with_encoding(shape, encoding, source_lang_owned.clone())
            .with_cleanup_rules(cleanup_rules),
    ) {
        Ok(outcome) => outcome,
        Err(err) => {
            store.close();
            remove_folder(&dir);
            return Err(err.into());
        }
    };
    let chapters = outcome.chapters;

    // 🔴 SỬA (vòng rà đối kháng 2026-09-04, item 4) — bán kính nổ của một `.expect()` bên
    // TRONG closure ghi là TOÀN TIẾN TRÌNH: `panic = "abort"` giết ngay khi giao dịch đang
    // mở, không unwind, không rollback. Cả hai bất biến dưới đây được validate ở NGOÀI
    // closure, TRƯỚC khi giao dịch mở — không phải vì chúng có thể xảy ra hôm nay (chuỗi
    // sản phẩm luôn N = 1, `chapter_pattern: None`), mà vì `run_import` là một seam CÔNG
    // KHAI (`PipelineShape::Chapters` cho phép N tuỳ ý), và "không thể" là một quan sát về
    // đường sản phẩm HÔM NAY, không phải một hợp đồng kiểu mà trình biên dịch cưỡng chế.
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
        // 🔴 AD-39, N Chương (N = 1 ở story này) — `ord` liên tục từ 1, cùng giao dịch với
        // hàng `work`. `OpenWork::chapter_id` chốt vào Chương ĐẦU TIÊN — Story 2.11 xoá
        // hẳn lối suy-ra-động (`ORDER BY ord LIMIT 1`); N > 1 là mối bận tâm của story sở
        // hữu năng lực đó (6.6/6.7), không phải story này.
        //
        // 🔴 KHÔNG `Option<i64>` + `.expect()` — `chapters` đã được xác nhận KHÔNG RỖNG
        // và `chapters.len()` đã được xác nhận VỪA `i64` ở NGOÀI closure này (vòng rà đối
        // kháng 2026-09-04, item 4). `is_first`/`i as i64` vì thế an toàn theo CẤU TRÚC,
        // không phải theo linh cảm "thực tế không xảy ra".
        let mut first_chapter_id: i64 = 0;
        let mut is_first = true;
        for (i, chapter) in chapters.iter().enumerate() {
            let ord = i as i64 + 1;
            tx.execute(
                "INSERT INTO chapter (ord, title, source_text, status, created_at, updated_at) \
                 VALUES (?1, NULL, ?2, ?3, strftime('%Y-%m-%dT%H:%M:%fZ','now'), \
                 strftime('%Y-%m-%dT%H:%M:%fZ','now'))",
                (ord, &chapter.source_text, LifecycleStatus::NotStarted.as_str()),
            )?;

            // `last_insert_rowid()` đọc **trong** giao dịch, ngay sau lượt chèn của chính
            // nó — `Store::write` giữ một writer duy nhất nối tiếp, nên không lượt chèn
            // nào khác chen được vào giữa hai dòng này.
            let chapter_id = tx.last_insert_rowid();
            crate::commands::segment::insert_segments(tx, chapter_id, &chapter.segments)?;
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

    Ok(OpenWork {
        dir,
        store,
        scope,
        meta,
        chapter_id,
    })
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
    create_work(
        documents_root,
        name,
        source_lang,
        genre,
        import_text(text),
        encoding_rs::UTF_8,
        Vec::new(),
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
/// `.docx` hay định dạng khác ⇒ `import.unsupported_format` (AC8), **trước khi** thư mục
/// `.atproj` được tạo — [`import_file`] từ chối theo phần mở rộng trước khi mở tệp.
pub fn create_work_from_file(
    documents_root: &Path,
    name: &str,
    source_lang: &str,
    genre: &str,
    path: &Path,
) -> Result<OpenWork, IpcError> {
    let shape = import_file(path)?;
    create_work(
        documents_root,
        name,
        source_lang,
        genre,
        shape,
        encoding_rs::UTF_8,
        Vec::new(),
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
    /// Số chỗ khớp trong CẢ lần nhập. 🔵 **Nợ có chủ (Story 6.6/6.7, `deferred-work.md`)** —
    /// hôm nay LUÔN bằng `count_in_chapter`: một lượt xem trước chỉ hiện đúng MỘT Chương
    /// (`PipelineShape::Blob`), nên "cả lần nhập" và "Chương này" là cùng một tập. Con số
    /// KHÔNG sai — nó đúng cho một lần nhập một Chương — nhưng sẽ khác đi khi 6.6/6.7 dựng
    /// lần nhập nhiều Chương.
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
/// được hôm nay. **Chưa đo**: một Chương ở quy mô hàng MB (nếu FR124/Story 6.6+ sau này cho
/// phép một Chương lớn hơn nhiều so với một chương tiểu thuyết điển hình) — nếu ngày đó tới,
/// đo lại trước khi tin, đừng suy tuyến tính từ con số ở đây.
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
fn cleanup_preview_for(
    shape: PipelineShape,
    encoding: &'static encoding_rs::Encoding,
    display_window: &str,
    source_lang: &str,
    cleanup_rules: &[CleanupRule],
    window_truncated: bool,
) -> CleanupPreviewWire {
    let input =
        PipelineInput::with_encoding(shape, encoding, source_lang).with_cleanup_rules(cleanup_rules.to_vec());

    let (final_text_full, report) = match run_pipeline(input) {
        Ok(outcome) => match outcome.chapters.into_iter().next() {
            Some(chapter) => (chapter.source_text, chapter.cleanup_report),
            None => (display_window.to_owned(), None),
        },
        Err(err) => {
            eprintln!("cleanup[preview] chuoi pipeline that bai, roi ve bao cao rong: {err}");
            (display_window.to_owned(), None)
        }
    };

    build_cleanup_preview_wire(display_window, final_text_full, cleanup_rules, report, window_truncated)
}

/// Dựng [`CleanupPreviewWire`] từ một [`crate::core::cleanup::CleanupReport`] ĐÃ CÓ (hoặc
/// `None` khi bước 3 không tạo được báo cáo) — tách khỏi [`cleanup_preview_for`] để chỗ gọi
/// KHÔNG chạy chuỗi (báo cáo rỗng) dùng lại được đúng phép dựng hình dạng dây.
///
/// `display_window`: bản dựng ĐÃ CẮT AN TOÀN của văn bản TRƯỚC khi xoá gì (cùng cửa sổ tầng
/// 1 hiển thị) — trở thành `text` trên dây thẳng, không qua `report` (đóng khuyết tật
/// 2026-09-06: `report.matches`/`.per_rule_counts` đo trên TOÀN văn bản, nhưng `text` hiển thị
/// thì KHÔNG được phép dài hơn cửa sổ). `final_text_full` là văn bản CUỐI CÙNG của TOÀN
/// Chương — cắt xuống cửa sổ CHỈ khi `window_truncated` (giữ nguyên bất biến "không cắt khi
/// không cần cắt" mà `preview_and_confirm_agree_byte_for_byte_on_the_same_input_and_the_same_rules`
/// khoá).
fn build_cleanup_preview_wire(
    display_window: &str,
    final_text_full: String,
    cleanup_rules: &[CleanupRule],
    report: Option<crate::core::cleanup::CleanupReport>,
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

    // 🔵 Nợ có chủ (Story 6.6/6.7) — `count_in_import` LUÔN bằng `count_in_chapter` hôm nay,
    // xem doc-comment `CleanupRuleReportWire::count_in_import`.
    let rules = cleanup_rules
        .iter()
        .map(|rule| {
            let count = counts.get(&(rule.tier, rule.id)).copied().unwrap_or(0);
            CleanupRuleReportWire {
                tier: rule.tier.into(),
                id: rule.id,
                pattern: rule.pattern.clone(),
                kind: rule.kind.as_str().to_owned(),
                enabled: rule.enabled,
                count_in_chapter: count,
                count_in_import: count,
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
/// 🔴 **SỬA 2026-09-06** — nhận thêm `full_bytes` (byte thô TRỌN VẸN của đơn vị đang xem
/// trước, KHÔNG cắt cửa sổ): `cleanup_preview_for` cần TOÀN bộ byte để chạy chuỗi thật với
/// ĐÚNG bảng mã của ứng viên này (`encoding::encoding_for_wire_id(c.wire_id)`), không phải
/// văn bản window đã giải mã sẵn — xem doc-comment `cleanup_preview_for`.
fn encoding_candidate_wire(
    c: EncodingCandidate,
    full_bytes: &[u8],
    source_lang: &str,
    cleanup_rules: &[CleanupRule],
) -> EncodingCandidateWire {
    // `pipeline_window`/`normalized` đồng bộ `Some`/`None` với nhau (cả hai tính từ
    // CÙNG `decoded.as_ref()` bên trong `render_candidates`) — an toàn đọc `window_truncated`
    // từ `normalized` khi `pipeline_window` có giá trị.
    let window_truncated = c.normalized.as_ref().is_some_and(|n| n.window_truncated);
    let cleanup = c.pipeline_window.as_deref().map(|window| {
        match encoding::encoding_for_wire_id(c.wire_id) {
            Some(encoding) => {
                let shape = PipelineShape::Blob(ChapterInput::RawBytes {
                    bytes: full_bytes.to_vec(),
                    label: String::new(),
                });
                cleanup_preview_for(shape, encoding, window, source_lang, cleanup_rules, window_truncated)
            }
            // Không nên xảy ra — `c.wire_id` đến từ `Encoding::name()` của chính một trong
            // năm bảng mã FR126, luôn phân giải lại được. Rơi về báo cáo rỗng thay vì làm vỡ
            // cả dải, giữ đúng khuôn dung thứ lỗi của hàm này.
            None => build_cleanup_preview_wire(window, window.to_owned(), cleanup_rules, None, window_truncated),
        }
    });

    EncodingCandidateWire {
        label: c.label.to_owned(),
        encoding: c.wire_id.to_owned(),
        preview: c.preview,
        normalized: c.normalized.map(NormalizedPreviewWire::from),
        cleanup,
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
pub fn preview_import_encoding(
    shape: &PipelineShape,
    source_lang: &str,
    cleanup_rules: &[CleanupRule],
) -> ImportEncodingPreview {
    let verdict_and_candidates = |bytes: &[u8]| -> (EncodingVerdict, Vec<EncodingCandidateWire>) {
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
                .map(|c| encoding_candidate_wire(c, bytes, source_lang, cleanup_rules))
                .collect()
        };
        (verdict, candidates)
    };

    let self_declared_utf8 = || EncodingVerdict {
        encoding: encoding_rs::UTF_8,
        confidence: Confidence::SelfDeclared,
    };

    let (verdict, candidates) = match shape {
        PipelineShape::Blob(ChapterInput::AlreadyText(_)) => (self_declared_utf8(), Vec::new()),
        PipelineShape::Blob(ChapterInput::RawBytes { bytes, .. }) => verdict_and_candidates(bytes),
        // ⚠️ Sản phẩm hôm nay không có bề mặt nào dựng `PipelineShape::Chapters` TRƯỚC màn
        // xem trước bảng mã (danh sách URL là Story 6.7) — nhánh này chỉ tồn tại để khớp
        // kiểu (`match` cạn hết). Dò trên đơn vị ĐẦU khi nó mang byte thô; tự khai khi rỗng
        // hoặc đơn vị đầu đã là văn bản.
        PipelineShape::Chapters(chapters) => match chapters.first() {
            Some(ChapterInput::RawBytes { bytes, .. }) => verdict_and_candidates(bytes),
            Some(ChapterInput::AlreadyText(_)) | None => (self_declared_utf8(), Vec::new()),
        },
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
    let self_declared_cleanup = self_declared_normalized.as_ref().map(|normalized| {
        let text = self_declared_source_text(shape);
        match encoding::pipeline_window_for_self_declared(text) {
            Some(window) => {
                // 🔴 SỬA 2026-09-06 — `shape` mang văn bản TOÀN VẸN (`text`, không phải
                // `window`) để `cleanup_preview_for` chạy chuỗi trên CẢ Chương; `window` chỉ
                // còn vai trò giới hạn hiển thị. Xem doc-comment `cleanup_preview_for`.
                let full_shape = PipelineShape::Blob(ChapterInput::AlreadyText(text.to_owned()));
                cleanup_preview_for(
                    full_shape,
                    encoding_rs::UTF_8,
                    &window,
                    source_lang,
                    cleanup_rules,
                    normalized.window_truncated,
                )
            }
            // Cùng ca "cửa sổ không đủ một dòng trọn vẹn" của `normalized_self_declared` —
            // `final_text` rỗng đồng bộ với `NormalizedPreviewWire.text == ""` ở đó.
            None => build_cleanup_preview_wire(
                text,
                String::new(),
                cleanup_rules,
                None,
                normalized.window_truncated,
            ),
        }
    });

    ImportEncodingPreview {
        confidence: verdict.confidence.into(),
        selected_encoding: verdict.encoding.name().to_owned(),
        candidates,
        self_declared_normalized,
        self_declared_cleanup,
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
pub fn stash_pending_import_source(state: &PendingImportSourceState, shape: PipelineShape) {
    let mut guard = state.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
    *guard = Some(PendingImportSource { shape });
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
pub fn confirm_import_with_encoding(
    documents_root: &Path,
    state: &PendingImportSourceState,
    name: &str,
    source_lang: &str,
    genre: &str,
    encoding_wire_id: &str,
    cleanup_rules: Vec<CleanupRule>,
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

    let opened =
        create_work(documents_root, name, source_lang, genre, shape, chosen, cleanup_rules)?;

    // Thành công — dọn ô đang chờ, VẪN dưới CÙNG một khoá đã giữ từ đầu hàm.
    *guard = None;

    Ok(opened)
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

    Ok(OpenWork { dir, store, scope, meta, chapter_id })
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
        ImportScanGeneration, ImportScanNextStep, create_work_from_text,
        dictionary_inconclusive_event, dictionary_probe_from_grouped,
        filter_and_enqueue_current_import_scan, guarded_dict_layers, guarded_open_store,
        import_scan_next_step, keep_committed_import_when_scan_spawn_fails,
        read_chapter_segment_texts, swap_locked,
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
}

/// Nhiều vỏ `#[tauri::command]`. **Không một quy tắc nào sống ở đây.**
pub mod wire {
    use super::{
        ImportEncodingPreview, IpcError, OpenWork, OpenWorkState, PendingImportSourceState,
        no_pending_import_source, replace_open_work, resolve_library_root, spawn_import_scan,
    };
    use crate::core::cleanup::CleanupRule;
    use crate::core::i18n::MessageKey;
    use crate::core::library::WorkMeta;
    use crate::core::library::indexer::Indexer;
    use crate::core::scope::ScopeResolver;
    use crate::core::store::Store;

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
    }

    impl CreatedWork {
        /// Gói một [`OpenWork`] thành thứ đi qua dây được — `Store` không `Serialize`.
        fn from_open(open: &OpenWork) -> Self {
            Self {
                meta: open.meta.clone(),
                folder: open.dir.display().to_string(),
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
    #[tauri::command]
    pub fn preview_import_encoding_from_text(
        app: tauri::AppHandle,
        text: String,
        source_lang: String,
    ) -> Result<ImportEncodingPreview, IpcError> {
        use tauri::Manager as _;
        let Some(state) = app.try_state::<PendingImportSourceState>() else {
            return Err(no_pending_import_source());
        };
        let cleanup_rules = resolve_cleanup_rules(&app);
        let shape = super::import_text(text);
        let preview = super::preview_import_encoding(&shape, &source_lang, &cleanup_rules);
        super::stash_pending_import_source(&state, shape);
        Ok(preview)
    }

    /// Vỏ IPC — nhánh TỆP của màn xem trước bảng mã (Story 6.3, FR126). [`import_file`] đọc
    /// byte thô ĐÚNG MỘT LẦN ở đây; `confirm_import_with_encoding` CLONE từ
    /// [`PendingImportSourceState`], không đọc lại đĩa.
    ///
    /// # Lỗi
    /// - [`PendingImportSourceState`] chưa được quản lý ⇒ `import.no_pending_source`, cùng lý
    ///   do nhánh DÁN VĂN BẢN ngay trên.
    #[tauri::command]
    pub fn preview_import_encoding_from_file(
        app: tauri::AppHandle,
        path: String,
        source_lang: String,
    ) -> Result<ImportEncodingPreview, IpcError> {
        use tauri::Manager as _;
        let Some(state) = app.try_state::<PendingImportSourceState>() else {
            return Err(no_pending_import_source());
        };
        let cleanup_rules = resolve_cleanup_rules(&app);
        let shape = super::import_file(std::path::Path::new(&path))?;
        let preview = super::preview_import_encoding(&shape, &source_lang, &cleanup_rules);
        super::stash_pending_import_source(&state, shape);
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
    #[tauri::command]
    pub fn confirm_import_with_encoding(
        app: tauri::AppHandle,
        name: String,
        source_lang: String,
        genre: String,
        encoding: String,
    ) -> Result<CreatedWork, IpcError> {
        use tauri::Manager as _;

        let Some(pending_state) = app.try_state::<PendingImportSourceState>() else {
            return Err(no_pending_import_source());
        };
        // 🔴 Nạp luật NGAY LÚC XÁC NHẬN, không tái dùng bộ đã nạp lúc xem trước — luật có
        // thể đã đổi giữa hai nhịp qua một lượt bật/tắt/soạn khác (§Always spec 6.5).
        let cleanup_rules = resolve_cleanup_rules(&app);
        let root = resolve_library_root(&app, app.try_state::<Store>().as_deref())?;
        let opened = super::confirm_import_with_encoding(
            &root,
            &pending_state,
            &name,
            &source_lang,
            &genre,
            &encoding,
            cleanup_rules,
        )?;

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
}
