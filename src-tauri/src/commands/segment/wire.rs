    use super::{
        ChapterSegments, ConfirmOutcome, IpcError, OmitOutcome, ParagraphEndOutcome,
        PromoteAiTranslationOutcome, SaveOutcome, ReadingMark, ReadingRun, RegroupOutcome,
        RestoreOutcome, SegmentTargetEdit, SegmentVersionRow, SplitOutcome,
    };
    use crate::commands::project::OpenWorkState;

    /// Vỏ IPC của [`super::split_chapter_into_segments`].
    ///
    /// ⚠️ `try_state`, không `state()` — cùng lý do `commands::chapter::wire`: state có thể
    /// chưa từng được `app.manage` (lỗi cấu hình `setup()`), và `panic = "abort"` giết cả
    /// tiến trình nếu ta thẳng tay `.unwrap()`.
    ///
    /// ⚠️ `chapter_id` đi trên dây dưới tên **`chapterId`** — `invoke()` gửi tham số ở dạng
    /// camelCase. `src/config/segment.ts` là chỗ duy nhất gõ cái tên đó.
    #[tauri::command]
    pub fn split_chapter_into_segments(
        app: tauri::AppHandle,
        chapter_id: i64,
    ) -> Result<SplitOutcome, IpcError> {
        use tauri::Manager as _;

        let Some(state) = app.try_state::<OpenWorkState>() else {
            return super::split_chapter_into_segments(None, chapter_id);
        };
        let guard = state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        super::split_chapter_into_segments(guard.as_ref(), chapter_id)
    }

    /// Vỏ IPC của [`super::read_open_chapter_segments`].
    ///
    /// ⚠️ **Không tham số nào đi trên dây** — xem doc-comment của hàm thuần. `invoke()` phía
    /// webview gọi nó với một payload rỗng.
    #[tauri::command]
    pub fn read_open_chapter_segments<R: tauri::Runtime>(
        app: tauri::AppHandle<R>,
        prefill: Option<bool>,
    ) -> Result<ChapterSegments, IpcError> {
        use tauri::Manager as _;

        let prefill = prefill.unwrap_or(true);
        let global = app.try_state::<crate::core::store::Store>();
        let Some(state) = app.try_state::<OpenWorkState>() else {
            return super::load_open_chapter_segments(global.as_deref(), None, prefill);
        };
        let guard = state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        super::load_open_chapter_segments(global.as_deref(), guard.as_ref(), prefill)
    }

    /// Vỏ IPC của [`super::read_reading_run`] — Story 5.12 (FR120).
    ///
    /// 🔵 **Đổi tên tại chỗ 2026-08-30 (Story 5.12)** — vỏ này trước mang tên
    /// `read_reading_chapter` (Story 5.11); tên trên dây LÀ tên hàm, và một lệnh đọc cùng bề
    /// mặt Chế độ đọc dưới hai tên là hai nguồn sự thật. `src/config/reading.ts` đổi tên gọi
    /// theo CÙNG lượt.
    ///
    /// ⚠️ **Không tham số nào đi trên dây**, cùng khuôn [`read_open_chapter_segments`] ngay
    /// trên. ⚠️ **Không** `(async)`: cùng hạng đọc-thuần với vỏ nó thay thế —
    /// `commands/segment.rs` không nằm trong bảng `count_async_attrs` của
    /// `config_invariants.rs` (xem §Design Notes của story cho phép đo quy mô đứng sau quyết
    /// định này, và món nợ có chủ khi quy mô đó đổi — Story 5.14).
    #[tauri::command]
    pub fn read_reading_run(app: tauri::AppHandle) -> Result<ReadingRun, IpcError> {
        use tauri::Manager as _;

        let Some(state) = app.try_state::<OpenWorkState>() else {
            return super::read_reading_run(None);
        };
        let guard = state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        super::read_reading_run(guard.as_ref())
    }

    /// Vỏ IPC mỏng cho marker Chế độ đọc. `segmentId` là tên camelCase phía webview.
    #[tauri::command]
    pub fn mark_reading_segment(
        app: tauri::AppHandle,
        segment_id: i64,
    ) -> Result<ReadingMark, IpcError> {
        use tauri::Manager as _;
        let Some(state) = app.try_state::<OpenWorkState>() else {
            return super::mark_reading_segment(None, segment_id);
        };
        let guard = state.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
        super::mark_reading_segment(guard.as_ref(), segment_id)
    }

    /// Vỏ IPC mỏng cho danh sách marker của Tác phẩm đang mở.
    #[tauri::command]
    pub fn list_reading_marks(app: tauri::AppHandle) -> Result<Vec<ReadingMark>, IpcError> {
        use tauri::Manager as _;
        let Some(state) = app.try_state::<OpenWorkState>() else {
            return super::list_reading_marks(None);
        };
        let guard = state.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
        super::list_reading_marks(guard.as_ref())
    }

    /// Vỏ IPC của [`super::save_segment_targets`] — đường flush của AD-35. Story 2.3.
    ///
    /// ⚠️ `chapter_id` đi trên dây dưới tên **`chapterId`**, và `edits` dưới tên **`edits`** —
    /// `invoke()` gửi tham số ở dạng camelCase. `src/config/segment.ts` là chỗ duy nhất gõ
    /// hai cái tên đó.
    ///
    /// 🔴 `MutexGuard` giữ **XUYÊN SUỐT** lời gọi, cùng lý do và cùng đường hỏng mà
    /// [`super::split_chapter_into_segments`] đã ghi ở doc-comment của nó. Nhả sớm để "tối ưu"
    /// là mở lại một cuộc đua ghi: `replace_open_work` có thể trỏ `OpenWorkState` sang một Tác
    /// phẩm **khác** giữa lúc lô này đang bay, và lúc đó lô ghi vào `project.db` của Tác phẩm
    /// vừa bị thay.
    #[tauri::command]
    pub fn save_segment_targets(
        app: tauri::AppHandle,
        chapter_id: i64,
        edits: Vec<SegmentTargetEdit>,
    ) -> Result<SaveOutcome, IpcError> {
        use tauri::Manager as _;

        let Some(state) = app.try_state::<OpenWorkState>() else {
            return super::save_segment_targets(None, chapter_id, &edits);
        };
        let guard = state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);

        // ⚠️ **KHÔNG một quyết định nào ở đây** — kể cả thứ tự hai lượt ghi của AD-31 hàng 3.
        // Nó sống ở `super::flush_segment_targets`, và lý do là một phép đo: đặt nó ở vỏ này
        // thì đảo thứ tự đi qua **54/54 ca xanh**, vì `tests/**` gọi một vỏ cần `AppHandle`
        // không được. Xem doc-comment của hàm thuần đó.
        //
        // ⚠️ Cả hai lượt ghi bên trong đi dưới **cùng một** `MutexGuard`: nhả giữa chừng là mở
        // lại đúng cuộc đua đã ghi ở dưới *(`replace_open_work` trỏ sang Tác phẩm khác giữa
        // lúc lô đang bay)*.
        super::flush_segment_targets(guard.as_ref(), chapter_id, &edits).map(|(_, saved)| saved)
    }

    /// Vỏ IPC của [`super::save_chapter_position`] — Story 5.7 (AC4/AC6).
    ///
    /// ⚠️ `chapter_id`/`segment_id` đi trên dây dưới tên **`chapterId`**/**`segmentId`** —
    /// `invoke()` gửi tham số ở dạng camelCase.
    #[tauri::command]
    pub fn save_chapter_position(
        app: tauri::AppHandle,
        chapter_id: i64,
        segment_id: i64,
    ) -> Result<(), IpcError> {
        use tauri::Manager as _;

        let Some(state) = app.try_state::<OpenWorkState>() else {
            return super::save_chapter_position(None, chapter_id, segment_id);
        };
        let guard = state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        super::save_chapter_position(guard.as_ref(), chapter_id, segment_id)
    }

    /// Vỏ IPC của [`super::confirm_segment`] — Story 2.5, FR24 · AD-31.
    ///
    /// ⚠️ `segment_id` đi trên dây dưới tên **`segmentId`** — `invoke()` gửi tham số ở dạng
    /// camelCase. `src/config/segment.ts` là chỗ duy nhất gõ cái tên đó.
    ///
    /// 🔴 `try_state`, **không** `state()` — mở kho có thể đã thất bại và `app.manage()` chưa
    /// từng chạy ⇒ `state()` panic ⇒ `panic = "abort"` giết cả tiến trình.
    ///
    /// 🔴 `MutexGuard` giữ **XUYÊN SUỐT** lời gọi, cùng lý do [`save_segment_targets`] đã ghi.
    ///
    /// ⚠️ **AD-35 vế (c) — flush TRƯỚC, xác nhận SAU — KHÔNG được cưỡng chế ở đây, và đó là
    /// một giới hạn thật, ghi ra thay vì để người sau tự phát hiện.** Lệnh này chỉ đọc thứ
    /// **đã ở trên đĩa**; nó không biết gì về văn bản đang gõ trong webview. Vế *"flush xong
    /// rồi mới ghi trạng thái"* được giao ở **tầng TypeScript**, tại chỗ command được gọi
    /// (`src/commands/index.ts` — `await flushEditorNow()` rồi mới `invoke`). Cái giá: một
    /// chỗ gọi tương lai quên `await` sẽ ký một văn bản **cũ hơn** thứ người dùng đang nhìn,
    /// và **không cổng nào ở tầng Rust bắt được**. Lưới ở đây là một test frontend, không một
    /// hợp đồng Rust.
    /// 🔴 Vỏ này **không** phân xử một chữ nào — nó chuyển nguyên văn tham số xuống hàm thuần.
    /// Cùng luật đã ghi cho `flush_segment_targets`: đo 2026-08-14 cho thấy một quyết định đặt
    /// ở vỏ đi qua **54/54 xanh** vì `tests/**` gọi vỏ không được *(nó cần `AppHandle`)*.
    #[tauri::command]
    pub fn confirm_segment<R: tauri::Runtime>(
        app: tauri::AppHandle<R>,
        segment_id: i64,
    ) -> Result<ConfirmOutcome, IpcError> {
        use tauri::Manager as _;

        let Some(state) = app.try_state::<OpenWorkState>() else {
            return super::confirm_segment(None, segment_id);
        };
        let guard = state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        super::confirm_segment(guard.as_ref(), segment_id)
    }

    /// Vỏ IPC của [`super::set_segment_omitted`] — Story 2.5c, FR133.
    ///
    /// ⚠️ `try_state`, không `state()` — cùng lý do mọi vỏ khác trong module này.
    ///
    /// ⚠️ Hai tham số đi trên dây dưới tên **`segmentId`** và **`omitted`** — `invoke()` gửi
    /// tham số ở dạng camelCase dù hàm Rust nhận `snake_case`. `src/config/segment.ts` là
    /// chỗ duy nhất gõ hai cái tên đó.
    ///
    /// ─────────────────────────────────────────────────────────────────────────────
    /// 🔴 **MỘT** VỎ CHO **HAI** LỆNH, VÀ ĐÓ KHÔNG MÂU THUẪN QUYẾT ĐỊNH #3
    /// ─────────────────────────────────────────────────────────────────────────────
    /// Ice ký đường (b) ngày 2026-08-15: *"hai lệnh `editor.omit_segment` +
    /// `editor.restore_segment` — hai phím, hai nhãn, trạng thái đọc được từ tên lệnh"*.
    /// Quyết định đó nói về tầng **`CommandRegistry`**, tức bề mặt người dùng chạm vào — và
    /// hai lệnh đó **có thật**, đăng ký riêng, nhãn riêng, hợp âm riêng
    /// (`src/commands/index.ts`).
    ///
    /// Tầng dây thì khác: hai lệnh ấy khác nhau ở **đúng một giá trị boolean**, và một vỏ
    /// thứ hai ở đây sẽ là một bề mặt IPC nữa phải cấp quyền, phải canh, phải giữ đồng bộ —
    /// để nói cùng một điều. *"Thêm một quyền là một quyết định kiến trúc, không phải một
    /// dòng cấu hình"* (`project-context.md` §Tauri). ⇒ Một vỏ, một tham số `omitted`.
    #[tauri::command]
    pub fn set_segment_omitted(
        app: tauri::AppHandle,
        segment_id: i64,
        omitted: bool,
    ) -> Result<OmitOutcome, IpcError> {
        use tauri::Manager as _;

        let Some(state) = app.try_state::<OpenWorkState>() else {
            return super::set_segment_omitted(None, segment_id, omitted);
        };
        let guard = state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        super::set_segment_omitted(guard.as_ref(), segment_id, omitted)
    }

    /// Vỏ mỏng cho [`super::set_segment_paragraph_end`] — Story 2.5d.
    ///
    /// 🔴 `try_state`, **không** `state()`: mở kho có thể đã thất bại và `app.manage()` chưa
    /// từng chạy ⇒ `state()` panic ⇒ `panic = "abort"` giết cả tiến trình.
    /// ⚠️ **Không một quyết định nào ở đây** — vỏ chỉ lấy `State` rồi gọi xuống hàm thuần.
    #[tauri::command]
    pub fn set_segment_paragraph_end(
        app: tauri::AppHandle,
        segment_id: i64,
        ends_paragraph: bool,
    ) -> Result<ParagraphEndOutcome, IpcError> {
        use tauri::Manager as _;

        let Some(state) = app.try_state::<OpenWorkState>() else {
            return super::set_segment_paragraph_end(None, segment_id, ends_paragraph);
        };
        let guard = state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        super::set_segment_paragraph_end(guard.as_ref(), segment_id, ends_paragraph)
    }

    /// Vỏ IPC của [`super::read_segment_history`]. Story 2.6 · FR101.
    ///
    /// ⚠️ `segment_id` đi trên dây dưới tên **`segmentId`** — `invoke()` gửi tham số ở dạng
    /// camelCase. ⚠️ Nhưng trường của [`SegmentVersionRow`] **trả về** giữ `snake_case`. Hai
    /// chiều khác nhau; `src/config/segment.ts` là chỗ duy nhất gõ cả hai cái tên.
    ///
    /// ⚠️ `try_state`, không `state()` — mở kho có thể đã thất bại và `app.manage()` chưa từng
    /// chạy ⇒ `state()` panic ⇒ `panic = "abort"` giết cả tiến trình.
    #[tauri::command]
    pub fn read_segment_history(
        app: tauri::AppHandle,
        segment_id: i64,
    ) -> Result<Vec<SegmentVersionRow>, IpcError> {
        use tauri::Manager as _;

        let Some(state) = app.try_state::<OpenWorkState>() else {
            return super::read_segment_history(None, segment_id);
        };
        let guard = state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        super::read_segment_history(guard.as_ref(), segment_id)
    }

    /// Vỏ IPC của [`super::restore_segment_version`]. Story 2.6 · FR101 · AC2.
    ///
    /// ⚠️ Ba tham số đi trên dây dưới tên **`segmentId`** · **`versionId`** · **`force`** —
    /// `invoke()` gửi tham số ở dạng camelCase. Trường của [`RestoreOutcome`] **trả về** thì
    /// giữ `snake_case`.
    ///
    /// 🔴 `force = false` là lượt gọi **thứ nhất**; nếu nó về với `needs_confirmation = true`
    /// thì **không một byte nào đã được ghi** và webview phải hỏi lại người dùng trước khi
    /// gọi lại với `force = true`. Xem doc-comment của hàm thuần.
    ///
    /// ⚠️ **Nghĩa vụ của tầng gọi:** flush mọi ký tự đang chờ **trước** lượt gọi này — chốt
    /// chống mất bản nháp so trên **đĩa**, và `editorEditedText` có thể còn giữ ký tự chưa
    /// xuống WAL (AD-35).
    ///
    /// ⚠️ `try_state`, không `state()` — cùng lý do năm vỏ trên.
    #[tauri::command]
    pub fn restore_segment_version(
        app: tauri::AppHandle,
        segment_id: i64,
        version_id: i64,
        force: bool,
    ) -> Result<RestoreOutcome, IpcError> {
        use tauri::Manager as _;

        let Some(state) = app.try_state::<OpenWorkState>() else {
            return super::restore_segment_version(None, segment_id, version_id, force);
        };
        let guard = state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        super::restore_segment_version(guard.as_ref(), segment_id, version_id, force)
    }

    /// Vỏ IPC của [`super::promote_ai_translation`]. Story 4.8 · FR72 · AD-47①/③; `force`
    /// theo khuôn AD-49 iii.
    ///
    /// ⚠️ Ba tham số đi trên dây dưới tên **`segmentId`** · **`targetText`** · **`force`** —
    /// `invoke()` gửi tham số ở dạng camelCase. Trường của [`PromoteAiTranslationOutcome`]
    /// **trả về** giữ `snake_case`.
    ///
    /// 🔴 `force = false` là lượt gọi **thứ nhất**; nếu nó về với `needs_confirmation = true`
    /// thì **không một byte nào đã được ghi** và webview phải hỏi lại người dùng trước khi
    /// gọi lại với `force = true` — cùng khuôn [`restore_segment_version`] ngay trên.
    ///
    /// ⚠️ **Nghĩa vụ của tầng gọi:** đây là lượt ghi non-user (AD-47①) — nó KHÔNG đọc bộ đệm
    /// gõ dở của Editor, nên không có "flush trước" nào cần đợi ở đây (khác
    /// `restore_segment_version`/`merge_segments`). Phía webview mirror kết quả bằng
    /// `replaceEditorSegment` NGAY sau lượt gọi này thành công (`needs_confirmation = false`)
    /// — đó là nửa còn lại của AD-47①(a) (§Code Map spec 4.8), không phải việc của vỏ Rust.
    ///
    /// ⚠️ `try_state`, không `state()` — cùng lý do mọi vỏ khác của kho.
    #[tauri::command]
    pub fn promote_ai_translation<R: tauri::Runtime>(
        app: tauri::AppHandle<R>,
        segment_id: i64,
        target_text: String,
        force: bool,
    ) -> Result<PromoteAiTranslationOutcome, IpcError> {
        use tauri::Manager as _;

        let Some(state) = app.try_state::<OpenWorkState>() else {
            return super::promote_ai_translation(None, segment_id, &target_text, force);
        };
        let guard = state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        super::promote_ai_translation(guard.as_ref(), segment_id, &target_text, force)
    }

    /// Wire shell of [`super::tm_fuzzy_matches`]; `segmentId` on the wire. Async so the scan
    /// runs off the main thread, and the `OpenWorkState` lock is released before scoring.
    #[tauri::command(async)]
    pub fn tm_fuzzy_matches<R: tauri::Runtime>(
        app: tauri::AppHandle<R>,
        segment_id: i64,
    ) -> Result<super::TmFuzzyMatches, IpcError> {
        use tauri::Manager as _;

        let global = app.try_state::<crate::core::store::Store>();
        let prepared = match app.try_state::<OpenWorkState>() {
            None => super::prepare_tm_fuzzy(global.as_deref(), None, segment_id)?,
            Some(state) => {
                let guard = state.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
                super::prepare_tm_fuzzy(global.as_deref(), guard.as_ref(), segment_id)?
            }
        };
        super::score_tm_fuzzy(prepared)
    }

    /// Wire shell of [`super::tm_concordance`]; `query` on the wire. Async, and the
    /// `OpenWorkState` lock is released before the substring scan.
    #[tauri::command(async)]
    pub fn tm_concordance<R: tauri::Runtime>(
        app: tauri::AppHandle<R>,
        query: String,
    ) -> Result<super::TmConcordance, IpcError> {
        use tauri::Manager as _;

        let global = app.try_state::<crate::core::store::Store>();
        let scan = match app.try_state::<OpenWorkState>() {
            None => super::prepare_tm_concordance(global.as_deref(), None, &query)?,
            Some(state) => {
                let guard = state.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
                super::prepare_tm_concordance(global.as_deref(), guard.as_ref(), &query)?
            }
        };
        super::score_tm_concordance(scan)
    }

    /// Wire shell of [`super::accept_tm_fuzzy`]; `segmentId`, `tier`, `unitId`, `expectedTarget`, `force` on the wire.
    #[tauri::command]
    pub fn accept_tm_fuzzy<R: tauri::Runtime>(
        app: tauri::AppHandle<R>,
        segment_id: i64,
        tier: String,
        unit_id: i64,
        expected_target: String,
        force: bool,
    ) -> Result<PromoteAiTranslationOutcome, IpcError> {
        use tauri::Manager as _;

        let global = app.try_state::<crate::core::store::Store>();
        let Some(state) = app.try_state::<OpenWorkState>() else {
            return super::accept_tm_fuzzy(global.as_deref(), None, segment_id, &tier, unit_id, &expected_target, force);
        };
        let guard = state.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
        super::accept_tm_fuzzy(global.as_deref(), guard.as_ref(), segment_id, &tier, unit_id, &expected_target, force)
    }

    /// Wire shell of [`super::review_accept_change`]; `chapterId`, `groupId`, `expectedTarget`, `force` on the wire.
    #[tauri::command]
    pub fn review_accept_change<R: tauri::Runtime>(
        app: tauri::AppHandle<R>,
        chapter_id: i64,
        group_id: i64,
        expected_target: String,
        force: bool,
    ) -> Result<PromoteAiTranslationOutcome, IpcError> {
        use tauri::Manager as _;

        let Some(state) = app.try_state::<OpenWorkState>() else {
            return super::review_accept_change(None, chapter_id, group_id, &expected_target, force);
        };
        let guard = state.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
        super::review_accept_change(guard.as_ref(), chapter_id, group_id, &expected_target, force)
    }

    /// Wire shell of [`super::accept_tm_exact`]; `segmentId`, `tier`, `unitId`, `expectedTarget`, `force` on the wire.
    #[tauri::command]
    pub fn accept_tm_exact<R: tauri::Runtime>(
        app: tauri::AppHandle<R>,
        segment_id: i64,
        tier: String,
        unit_id: i64,
        expected_target: String,
        force: bool,
    ) -> Result<PromoteAiTranslationOutcome, IpcError> {
        use tauri::Manager as _;

        let global = app.try_state::<crate::core::store::Store>();
        let Some(state) = app.try_state::<OpenWorkState>() else {
            return super::accept_tm_exact(global.as_deref(), None, segment_id, &tier, unit_id, &expected_target, force);
        };
        let guard = state.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
        super::accept_tm_exact(global.as_deref(), guard.as_ref(), segment_id, &tier, unit_id, &expected_target, force)
    }

    /// Vỏ IPC của [`super::merge_segments`]. Story 2.8 · FR78 · AD-5 · AC1.
    ///
    /// ⚠️ `segment_id` đi trên dây dưới tên **`segmentId`** — `invoke()` gửi tham số ở dạng
    /// camelCase. ⚠️ Nhưng trường của [`RegroupOutcome`] **trả về** giữ `snake_case`. Hai
    /// chiều khác nhau; `src/config/segment.ts` là chỗ duy nhất gõ cả hai cái tên.
    ///
    /// ⚠️ **Nghĩa vụ của tầng gọi:** flush mọi ký tự đang chờ **trước** lượt gọi này — bản
    /// dịch đi vào hàng mới đọc từ **đĩa**, và `editorEditedText` có thể còn giữ ký tự chưa
    /// xuống WAL (AD-35). Cùng nghĩa vụ mà `restore_segment_version` đã mang.
    ///
    /// ⚠️ `try_state`, không `state()` — mở kho có thể đã thất bại và `app.manage()` chưa
    /// từng chạy ⇒ `state()` panic ⇒ `panic = "abort"` giết cả tiến trình.
    #[tauri::command]
    pub fn merge_segments(
        app: tauri::AppHandle,
        segment_id: i64,
    ) -> Result<RegroupOutcome, IpcError> {
        use tauri::Manager as _;

        let Some(state) = app.try_state::<OpenWorkState>() else {
            return super::merge_segments(None, segment_id);
        };
        let guard = state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        super::merge_segments(guard.as_ref(), segment_id)
    }

    /// Vỏ IPC của [`super::split_segment`]. Story 2.8 · FR78 · AD-5 · AC2.
    ///
    /// ⚠️ Hai tham số đi trên dây dưới tên **`segmentId`** · **`cuts`**. Mỗi phần tử của
    /// `cuts` đếm **ký tự Unicode**, không byte — tầng thuần chịu được một số bất kỳ, và đây
    /// là bề mặt duy nhất của story này nhận một chỉ số từ webview.
    ///
    /// 🔵 **2026-08-17 — `cut: usize` thành `cuts: Vec<usize>`**, chữ ký của Ice cho AC7 vế
    /// *"nhiều mảnh"* sau code review. `n` chỗ cắt cho `n + 1` mảnh trong **một** lượt ghi.
    /// 🔴 Đây là một lượt **đổi hình dạng dây**: test Rust và vitest dựng fixture chép tay
    /// luôn có sẵn trường. ⇒ Lưới duy nhất là **e2e**, và ca của nó phải gửi một mảng thật.
    ///
    /// ⚠️ Cùng nghĩa vụ flush của tầng gọi như [`merge_segments`], và cùng lý do `try_state`.
    #[tauri::command]
    pub fn split_segment(
        app: tauri::AppHandle,
        segment_id: i64,
        cuts: Vec<usize>,
    ) -> Result<RegroupOutcome, IpcError> {
        use tauri::Manager as _;

        let Some(state) = app.try_state::<OpenWorkState>() else {
            return super::split_segment(None, segment_id, cuts);
        };
        let guard = state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        super::split_segment(guard.as_ref(), segment_id, cuts)
    }
