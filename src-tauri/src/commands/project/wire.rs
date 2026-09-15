    use super::{
        FileImportBatchWire, ImportEncodingPreview, IpcError, OpenWork, OpenWorkState,
        PendingImportSourceState, Tier2BlockOverridesState, cancel_import_preview,
        no_pending_import_source, replace_open_work, resolve_library_root, spawn_import_scan,
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
    ///
    /// 🔴 **`#[tauri::command(async)]` trên một hàm ĐỒNG BỘ — CƠ CHẾ, viết một lần ở đây và
    /// năm vỏ dưới trỏ về.** Macro sinh `body_async`
    /// (`tauri-macros-2.6.3/src/command/wrapper.rs:361-396`), bọc lời gọi đồng bộ trong
    /// `async move { … }` rồi giao cho `respond_async_serialized`
    /// (`tauri-2.11.5/src/ipc/mod.rs:343`) → `respond_async_serialized_inner` (`:371`) →
    /// `async_runtime::spawn` (`:375`) → **`tokio::spawn` trên runtime ĐA LUỒNG**
    /// (`async_runtime.rs:103-113`; `default_runtime` ở `:222` dựng `TokioRuntime::new()`).
    /// ⇒ thân hàm chạy trên một **luồng worker tokio**, không phải luồng chính — đó là thứ gỡ
    /// lượt treo cửa sổ. **KHÔNG** phải một "sync threadpool": `spawn_blocking`
    /// (`async_runtime.rs:290`) không nằm trên đường này, và chuỗi `"sync_threadpool"` ở
    /// `wrapper.rs:264` chỉ được `tracing::debug_span!` ở `:278` đọc — một nhãn log, không
    /// điều khiển gì. Không đổi một dòng thân hàm; chỉ tham số phải `Send`, và vỏ này nhận
    /// `AppHandle` cùng các giá trị sở hữu.
    ///
    /// **Vỏ này CHẶN vì:** `reindex_library` ngay dưới là một lượt quét TOÀN BỘ thư mục gốc
    /// Library (`Indexer::rebuild(root)`), chạy sau mỗi lượt tạo. Không phải mạng: hình `Blob`
    /// để `blocks` rỗng nên `prepare_chapter_images` không với tới `webimport::fetch` từ đây.
    /// 🔵 **SỬA 2026-09-15 (vòng rà 3)** — câu này từng trỏ `core/segment/pipeline.rs:702`.
    /// Dòng ấy là `blocks: vec![None; n]` trong khởi tạo `Flow` DÙNG CHUNG, mà cả ba hình
    /// (`Blob`, `Chapters`, `Bilingual`) đều đi qua như nhau (`match` ở `:680-692`) — nên nó
    /// không phân biệt được `Blob` với đường URL và không đỡ được kết luận. Lý do ĐÚNG: bước
    /// rót `blocks` chỉ chạy trên đường HTML/URL (`Step::ExtractMainContent`), nói rõ tại
    /// `pipeline.rs:1554-1560`. Kết luận không đổi, chỉ chỗ trỏ đổi.
    /// Cổng canh: `config_invariants.rs::the_blocking_wires_run_off_the_main_thread`.
    #[tauri::command(async)]
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
    ///
    /// `#[tauri::command(async)]` trên một hàm ĐỒNG BỘ ⇒ thân hàm chạy trên một **luồng
    /// worker tokio** (`tokio::spawn`, KHÔNG phải `spawn_blocking`, không phải luồng chính) —
    /// chuỗi dẫn chứng đầy đủ ở doc-comment `create_work_from_text` ngay trên.
    ///
    /// **Vỏ này CHẶN vì:** `import_file` đọc TRỌN tệp vào bộ nhớ, trần `MAX_IMPORT_BYTES` =
    /// 100 MB (`core/segment/import.rs:82`, kiểm ở `:684`), cộng một lượt giải nén `.docx`;
    /// rồi mới tới trọn pipeline và `reindex_library`. Đây là chi phí theo KÍCH THƯỚC TỆP,
    /// khác hẳn lý do của `create_work_from_text` (theo kích thước THƯ VIỆN).
    #[tauri::command(async)]
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

    /// Vỏ IPC — nhánh TỆP của màn xem trước bảng mã (Story 6.3, FR126; mở rộng N tệp Story
    /// 6.6b, FR14). [`super::import_files`] đọc byte thô ĐÚNG MỘT LẦN mỗi tệp ở đây;
    /// `confirm_import_with_encoding` CLONE từ [`PendingImportSourceState`], không đọc lại
    /// đĩa.
    ///
    /// 🔵 **SỬA 2026-09-15 (Story 6.6b) — tham số đổi từ `path: String` sang `paths:
    /// Vec<String>`, kiểu trả đổi từ `ImportEncodingPreview` sang [`FileImportBatchWire`].**
    /// Đây là hình dạng DÂY MỚI của vỏ này — một envelope PER-ITEM cho MỌI N (kể cả N = 1,
    /// §Always spec 6.6b: "one shape to reason about"), thay vì trả thẳng
    /// `ImportEncodingPreview`. `paths` rỗng ⇒ [`super::import_files`] trả
    /// `Err(ImportError::EmptyFileList)`, đi thẳng qua `?` (I/O Matrix "Empty list"). Một mục
    /// KHÔNG đọc được (quyền, quá cỡ, `.docx`/`.csv`/`.tsv` bên trong một batch N > 1) không
    /// làm hỏng cả lượt — nó ở lại trong `items` với lý do riêng, và `encoding_preview`
    /// chuyển `None` (điều kiện ĐỦ để khoá xác nhận, §Decisions).
    ///
    /// # Lỗi
    /// - [`PendingImportSourceState`] chưa được quản lý ⇒ `import.no_pending_source`, cùng lý
    ///   do nhánh DÁN VĂN BẢN ngay trên.
    /// - `paths` rỗng ⇒ `import.empty_file_list` (`?` từ [`super::import_files`]).
    /// - `paths.len() == 1` và tệp đó không đọc được ⇒ lỗi TOÀN CỤC của chính tệp đó (không
    ///   có "phần còn lại" để mà giữ — §Always spec 6.6b: N = 1 y hệt hôm nay).
    /// 🔵 **THÊM 2026-09-05 (Story 6.6) — tham số `chapter_pattern`**, cùng lý do nhánh DÁN
    /// VĂN BẢN ngay trên.
    ///
    /// `#[tauri::command(async)]` trên một hàm ĐỒNG BỘ ⇒ thân hàm chạy trên một **luồng
    /// worker tokio** (`tokio::spawn`, KHÔNG phải `spawn_blocking`, không phải luồng chính) —
    /// chuỗi dẫn chứng đầy đủ ở doc-comment `create_work_from_text`.
    ///
    /// **Vỏ này CHẶN vì:** cùng trần 100 MB của `import_file`
    /// (`core/segment/import.rs:82`/`:684`) — đọc TRỌN MỖI tệp cộng giải nén `.docx` khi
    /// N = 1 — nhưng ở lượt XEM TRƯỚC, tức TRƯỚC khi người dùng xác nhận bất cứ điều gì; lượt
    /// treo rơi vào đúng nhịp người dùng còn đang cân nhắc, không phải nhịp họ đã chấp nhận
    /// chờ.
    #[tauri::command(async)]
    pub fn preview_import_encoding_from_file(
        app: tauri::AppHandle,
        paths: Vec<String>,
        source_lang: String,
        chapter_pattern: Option<super::ChapterPatternWire>,
    ) -> Result<FileImportBatchWire, IpcError> {
        use tauri::Manager as _;
        let Some(state) = app.try_state::<PendingImportSourceState>() else {
            return Err(no_pending_import_source());
        };
        let pattern = super::resolve_chapter_pattern(chapter_pattern)?;
        let cleanup_rules = resolve_cleanup_rules(&app);
        // 🔴 SỬA 2026-09-16 (phản biện) — `import_files` trượt (danh sách rỗng, hoặc MỘT
        // đường dẫn không đọc được ở N = 1) KHÔNG còn thoát bằng `?` trần. `?` trả lỗi TRƯỚC
        // khi chạm `state` — một `PipelineShape` đã cất từ một lượt xem trước TRƯỚC ĐÓ (còn
        // hợp lệ, `PendingImportSourceState` chưa từng dọn) sống sót qua lượt gọi TRƯỢT này và
        // vẫn xác nhận được, dù màn hình vừa báo lỗi. Dọn Ô ĐANG CHỜ trên đường lỗi này, cùng
        // khuôn `cancel_import_preview` ở nhánh "còn mục hỏng" ngay dưới.
        let outcome = match super::import_files(&paths) {
            Ok(outcome) => outcome,
            Err(err) => {
                cancel_import_preview(&state);
                return Err(err.into());
            }
        };
        // Story 6.9 — cùng lý do nhánh DÁN VĂN BẢN ở trên: đường tệp KHÔNG BAO GIỜ bóc nội
        // dung chính (kể cả `.docx` — Story 6.12: nó không đi qua `dom_smoothie`).
        reset_tier2_block_overrides(&app);
        // 🔴 SỬA 2026-09-10 (Story 6.15, lượt rà) — cùng lý do nhánh DÁN VĂN BẢN ở trên.
        reset_chapter_origin_overrides(&app);

        // 🔵 SỬA 2026-09-16 (phản biện) — quyết định "mọi mục OK / khoá / cất" nay sống trong
        // [`super::build_file_import_batch_wire`] (hàm THUẦN, `tests/**` gọi được không cần
        // `tauri::AppHandle`) thay vì chỉ ở đây.
        let (batch, all_ok) =
            super::build_file_import_batch_wire(&outcome, &source_lang, &cleanup_rules, pattern.as_ref());
        if all_ok {
            super::stash_pending_import_source(&state, outcome.shape, outcome.docx_sidecar);
        } else {
            // Một mục hỏng: không có gì hợp lệ để mà cất — dọn ô đang chờ, cùng khuôn
            // `sync_pending_from_url_items` (đường URL) khi còn mục hỏng.
            cancel_import_preview(&state);
        }
        Ok(batch)
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
    ///
    /// `#[tauri::command(async)]` trên một hàm ĐỒNG BỘ ⇒ thân hàm chạy trên một **luồng
    /// worker tokio** (`tokio::spawn`, KHÔNG phải `spawn_blocking`, không phải luồng chính) —
    /// chuỗi dẫn chứng đầy đủ ở doc-comment `create_work_from_text`.
    ///
    /// 🔴 **Vỏ này CHẶN vì MẠNG, TUẦN TỰ — vỏ nặng nhất của cả tệp.** `create_work` →
    /// [`super::prepare_chapter_images`] (`:975`) → `fetch_and_write_one_asset` (`:1402`) →
    /// `webimport::fetch` (`:1282`): một vòng lặp TUẦN TỰ, mỗi ảnh chờ tới `REQUEST_TIMEOUT`
    /// = 20 giây (`core/webimport/fetcher.rs:85`). ⇒ một lượt nhập từ URL có N ảnh trên một
    /// host chết đứng cửa sổ tới **N × 20 giây**, và suốt khoảng đó vỏ này còn GIỮ khoá
    /// [`PendingImportSourceState`].
    ///
    /// ⚠️ `(async)` **không rút ngắn** lượt chờ đó — nó chỉ dời chỗ chờ khỏi luồng giao diện.
    /// Ngân sách thời gian, tiến độ và huỷ giữa chừng cho vòng lặp ảnh là nợ CÓ CHỦ riêng
    /// (`deferred-work.md`, Story 6.11, chủ Ice).
    #[tauri::command(async)]
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
    ///
    /// `#[tauri::command(async)]` trên một hàm ĐỒNG BỘ ⇒ thân hàm chạy trên một **luồng
    /// worker tokio** (`tokio::spawn`, KHÔNG phải `spawn_blocking`, không phải luồng chính) —
    /// chuỗi dẫn chứng đầy đủ ở doc-comment `create_work_from_text`.
    ///
    /// **Vỏ này CHẶN vì:** [`super::import_bilingual_file`] (`core/segment/import.rs:737`)
    /// gọi `std::fs::read` ở `:763` — đọc TRỌN tệp `.csv`/`.tsv` vào bộ nhớ, cùng trần
    /// `MAX_IMPORT_BYTES` = 100 MB. Chỉ lượt MỞ đi qua đây; các lượt đổi cột/vai/tiêu đề sau
    /// đó đi qua `rebuild_bilingual_import_preview` và KHÔNG đọc lại đĩa, nên đây là điểm
    /// duy nhất của đường song ngữ trả giá đọc tệp.
    #[tauri::command(async)]
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
            // Lượt MỞ luôn bắt đầu 0 quy nhóm — người dùng chưa thấy hàng lệch cặp nào để mà
            // sửa.
            &[],
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
        // 🔴 **THÊM 2026-09-12 (Story 6.17, FR116)** — quy nhóm ĐANG có (mỗi lượt gõ một chỗ
        // cắt gọi lại vỏ này, cùng khuôn `chapter_pattern`) — Rust re-validate TOÀN BỘ danh
        // sách này ngay ở đây, đúng chữ "Rust re-validates every regrouping at rebuild".
        regroupings: Vec<super::BilingualRegroupingWire>,
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
        let resolved: Vec<crate::core::segment::bilingual::BilingualRegrouping> =
            regroupings.into_iter().map(Into::into).collect();
        super::preview_bilingual_import(
            &shape,
            &source_lang,
            &cleanup_rules,
            pattern.as_ref(),
            source_column,
            target_column,
            has_header,
            &resolved,
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
    ///
    /// `#[tauri::command(async)]` trên một hàm ĐỒNG BỘ ⇒ thân hàm chạy trên một **luồng
    /// worker tokio** (`tokio::spawn`, KHÔNG phải `spawn_blocking`, không phải luồng chính) —
    /// chuỗi dẫn chứng đầy đủ ở doc-comment `create_work_from_text`.
    ///
    /// **Vỏ này CHẶN vì:** trọn pipeline cộng một lô chèn `segment` hàng loạt cộng các lượt
    /// ghi đĩa, rồi `reindex_library`.
    ///
    /// 🔵 **KHÔNG phải mạng** — một bản ghi trước đó nói ngược lại. Nhánh song ngữ dựng mọi
    /// `ImportedChapter` với `blocks: None` (`core/segment/pipeline.rs:1011`, nhánh từ
    /// `:986`), và [`super::prepare_chapter_images`] bỏ qua đúng những Chương đó (`:1011`,
    /// `let Some(blocks) = &chapter.blocks else { continue }` — KHÔNG phải `:636-637`, chỗ đó
    /// là vòng dệt của `create_work`, có thêm điều kiện `weave_this_import`)
    /// ⇒ `webimport::fetch` không với tới được từ đây.
    #[tauri::command(async)]
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
        // 🔴 **THÊM 2026-09-12 (Story 6.17, FR116)** — quy nhóm ĐANG có, cùng khuôn
        // `rebuild_bilingual_import_preview` — "Rust re-validates every regrouping ... at
        // confirm".
        regroupings: Vec<super::BilingualRegroupingWire>,
    ) -> Result<CreatedWork, IpcError> {
        use tauri::Manager as _;

        let Some(pending_state) = app.try_state::<PendingImportSourceState>() else {
            return Err(no_pending_import_source());
        };
        let pattern = super::resolve_chapter_pattern(chapter_pattern)?;
        let root = resolve_library_root(&app, app.try_state::<Store>().as_deref())?;
        // Đọc hai tầng luật LÚC XÁC NHẬN — cùng kỷ luật `confirm_import_with_encoding`.
        let cleanup_rules = resolve_cleanup_rules(&app);
        let resolved: Vec<crate::core::segment::bilingual::BilingualRegrouping> =
            regroupings.into_iter().map(Into::into).collect();
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
            resolved,
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
