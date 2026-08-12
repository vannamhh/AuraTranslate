//! Bề mặt IPC đọc **Chương đang mở** — Story 1.16, AC8.
//!
//! Cùng khuôn `commands::config`/`commands::project`: hàm thuần trước, `#[tauri::command]`
//! chỉ là vỏ mỏng trong `wire`. Hàm thuần nhận `Option<&OpenWork>` — đúng khuôn
//! `Option<&Store>` của `commands::config` — chứ không nhận `Store` trực tiếp, vì Chương
//! sống trong Tác phẩm **đang mở** (`OpenWorkState`), không phải một kho được `app.manage`
//! thẳng như `global.db`.
//!
//! ─────────────────────────────────────────────────────────────────────────────
//! 🔴 STORY NÀY KHÔNG DỰNG "MỞ LẠI MỘT `.atproj` TỪ ĐĨA"
//! ─────────────────────────────────────────────────────────────────────────────
//! Đọc `OpenWorkState` — Tác phẩm mà `create_work_from_text`/`create_work_from_file` (Story
//! 1.15) vừa đặt vào, hoặc `chưa có` khi webview vừa mở lại từ đầu. Không có đường
//! `WorkMeta::read` nào ở đây.
//!
//! ⚠️ Mọi chuỗi trong tệp này viết KHÔNG DẤU — `scripts/check-i18n.mjs` Kiểm A quét
//! `src-tauri/**/*.rs`.

use crate::commands::project::OpenWork;
use crate::core::i18n::{IpcError, MessageKey};

/// Chương **đang mở**, đọc từ `OpenWorkState` — không phải một hàng chọn được (Epic 2).
///
/// ⚠️ `#[serde(rename_all = ...)]` KHÔNG đặt — cùng luật với mọi struct qua biên IPC.
#[derive(Debug, Clone, serde::Serialize)]
pub struct OpenChapter {
    /// `chapter.id` — định danh Chương, `dict_entry.id`-style: chỉ có nghĩa trong
    /// `project.db` của chính Tác phẩm đang mở.
    pub chapter_id: i64,
    /// `chapter.source_text` — nguyên khối, không tách câu/đoạn (AD-4, Story 2.1 sở hữu
    /// việc tách).
    pub source_text: String,
    /// `work.source_lang` — `"zh"` hoặc `"en"`, trường **bất biến** ghi lúc tạo (AD-18).
    /// Panel Source dùng đúng trường này để quyết có tab Hán Việt hay không (AC3), không
    /// không đoán từ nội dung `source_text`.
    pub source_lang: String,
}

/// Tác phẩm đang mở ⇒ lỗi *chưa mở Tác phẩm nào*, và đó là câu đúng theo nghĩa đen.
///
/// Đi qua `IpcError::new` với `MessageKey::ProjectNoWorkOpen` — không phải một lỗi kho
/// (`StoreError`), nên nó không thuộc từ vựng `store.*` (§Quyết định của Story 1.16).
///
/// ⚠️ Riêng tư trở lại từ 2026-08-11: Story 1.20 từng nâng nó lên `pub(crate)` để
/// `commands::pinned` tái dùng, nhưng lượt Ice ký lại chuyển mục ghim sang `global.db` —
/// nơi *"chưa mở Tác phẩm nào"* **không phải** một câu có nghĩa. Đọc Chương lại là chỗ duy
/// nhất nói câu đó, nên nó về đúng phạm vi cũ.
///
/// ⚠️ `pub(crate)` **trở lại** từ 2026-08-11 (Story 2.1), và điều kiện hạ phạm vi ở trên
/// không còn đúng: `commands::segment` tách một Chương **của Tác phẩm đang mở**, nên với nó
/// câu *"chưa mở Tác phẩm nào"* là câu đúng theo nghĩa đen — khác hẳn ca mục ghim. Hai chỗ
/// gọi, **một** khoá; một khoá thứ hai cho cùng câu là hai chuỗi phải giữ khớp nhau bằng
/// kỷ luật.
pub(crate) fn no_work_open() -> IpcError {
    IpcError::new(
        "project.no_work_open",
        MessageKey::ProjectNoWorkOpen,
        std::collections::BTreeMap::new(),
        false,
    )
}

/// Đọc Chương đang mở — **hàm thuần, đây là thứ test gọi**.
///
/// # Lỗi
/// - chưa Tác phẩm nào mở ⇒ `project.no_work_open`;
/// - đường đọc trượt (hàng `chapter` vắng mặt, kho hỏng) ⇒ `store.read_failed` (qua
///   `From<StoreError>`).
pub fn read_open_chapter(open: Option<&OpenWork>) -> Result<OpenChapter, IpcError> {
    let open = open.ok_or_else(no_work_open)?;

    // Epic 1 tạo ĐÚNG MỘT Chương cho mỗi Tác phẩm, `ord = 1` (Story 1.15,
    // `commands::project::create_work`). Chọn Chương / chuyển Chương là Epic 2 — không
    // thuộc phạm vi story này (xem §Ranh giới phạm vi của story).
    let (chapter_id, source_text) = open.store.read(|conn| {
        conn.query_row(
            "SELECT id, source_text FROM chapter ORDER BY ord LIMIT 1",
            [],
            |row| {
                let id: i64 = row.get(0)?;
                let source_text: String = row.get(1)?;
                Ok((id, source_text))
            },
        )
    })?;

    Ok(OpenChapter {
        chapter_id,
        source_text,
        source_lang: open.meta.source_lang.clone(),
    })
}

/// Một vỏ `#[tauri::command]`. **Không một quy tắc nào sống ở đây.**
pub mod wire {
    use super::{IpcError, OpenChapter};
    use crate::commands::project::OpenWorkState;

    /// Vỏ IPC của [`super::read_open_chapter`].
    ///
    /// ⚠️ `try_state`, không `state()` — cùng lý do `commands::config::wire`: state có
    /// thể chưa từng được `app.manage` (lỗi cấu hình `setup()`), và `panic = "abort"` giết
    /// tiến trình nếu ta thẳng tay `.unwrap()`.
    #[tauri::command]
    pub fn read_open_chapter(app: tauri::AppHandle) -> Result<OpenChapter, IpcError> {
        use tauri::Manager as _;

        let Some(state) = app.try_state::<OpenWorkState>() else {
            return super::read_open_chapter(None);
        };
        let guard = state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        super::read_open_chapter(guard.as_ref())
    }
}
