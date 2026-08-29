//! Bề mặt IPC "Bốn trạng thái vòng đời" — Story 5.4, FR5/FR6.
//!
//! Cùng khuôn `commands::chapter`/`commands::project`: hàm thuần trước, `#[tauri::command]`
//! chỉ là vỏ mỏng trong `mod wire`. Ba hàm thuần nhận `Option<&OpenWork>`/`Option<&mut
//! OpenWork>` — thứ `tests/**` gọi được không cần webview.
//!
//! ─────────────────────────────────────────────────────────────────────────────
//! 🔴 KHUÔN BỐN BƯỚC CỦA MỘT LƯỢT GHI TRẠNG THÁI — chép từ `commands/project.rs:266-340`
//! ─────────────────────────────────────────────────────────────────────────────
//! 1. `open.store.write(|tx| { UPDATE ... })` — giao dịch commit, chỉ SQL.
//! 2. `WorkMeta::rebuild_from_store(&open.store)` — đọc lại từ nguồn sự thật, không dùng số
//!    trong bộ nhớ (đây là chỗ [`crate::core::lifecycle::derive_work_status`] chạy).
//! 3. `meta.write_atomic(&open.dir)` — NGOÀI closure ghi (Quyết định #3, Story 1.15). Lỗi ở
//!    bước này **NÓI RA**, không nuốt (lý lẽ đã ghi ở `project.rs:317-332`).
//! 4. `reindex_library(app, root)` — ở LỚP VỎ (`mod wire`), CHỈ `Indexer` ghi
//!    `library-index.db` (AD-8). Hàm thuần ở tệp này KHÔNG gọi bước 4 — nó không có
//!    `AppHandle`.
//!
//! ⚠️ Mọi chuỗi trong tệp này viết KHÔNG DẤU — `scripts/check-i18n.mjs` Kiểm A quét
//! `src-tauri/**/*.rs`.

use std::collections::BTreeMap;
use std::path::Path;

use crate::commands::chapter::{chapter_not_found, no_work_open};
use crate::commands::project::OpenWork;
use crate::core::i18n::{IpcError, MessageKey};
use crate::core::library::WorkMeta;
use crate::core::library::indexer::Indexer;
use crate::core::lifecycle::LifecycleStatus;
use crate::core::store::{Store, Transaction};

/// Trạng thái vòng đời của Tác phẩm đang mở — thứ đi ra qua dây cho cả ba lệnh của tệp này.
///
/// ⚠️ `#[serde(rename_all = ...)]` KHÔNG đặt — cùng luật với mọi struct qua biên IPC. Hai
/// trường khớp ĐÚNG tên hai trường tương ứng của [`WorkMeta`] (`status`/`status_is_override`)
/// — cùng khái niệm, cùng tên, ở hai nơi khác nhau.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct WorkLifecycle {
    /// Một trong bốn giá trị trên dây của [`LifecycleStatus`], hoặc `None` — **chưa Tác phẩm
    /// nào ghi qua đường này từng để trường này `None`** (nó luôn đọc từ
    /// [`WorkMeta::rebuild_from_store`], nơi trường này LUÔN `Some`); `None` chỉ xảy ra khi
    /// chính `open.meta` đang mang một `meta.json` v1 chưa từng qua lượt dựng lại đó.
    pub status: Option<String>,
    /// `true` ⇔ [`Self::status`] đến từ ghi đè thủ công (`work.status_override`).
    pub status_is_override: bool,
}

impl From<&WorkMeta> for WorkLifecycle {
    fn from(meta: &WorkMeta) -> Self {
        Self { status: meta.status.clone(), status_is_override: meta.status_is_override }
    }
}

/// Giá trị trạng thái vòng đời ngoài danh mục bốn giá trị đóng — dùng chung cho ca
/// `set_chapter_status` LẪN `set_work_status_override` (§Tasks của story: "đúng một danh
/// mục đóng", tái dùng CÙNG một khoá, không đúc khoá thứ hai/ba).
///
/// 🔵 `pub(crate)` — `commands::library::list_works` cũng cần đúng câu này cho ca "giá trị lạ
/// trong bộ lọc" (§Tasks: *"giá trị lạ trong bộ lọc ⇒ err.lifecycle.unknown_status, không
/// im lặng bỏ qua"*), nên hàm này sống Ở ĐÂY (module sở hữu khái niệm) và được tái dùng, chứ
/// không chép lại ở `commands/library.rs`.
pub(crate) fn unknown_status(status: &str) -> IpcError {
    let mut params = BTreeMap::new();
    params.insert("status".to_owned(), status.to_owned());
    IpcError::new("lifecycle.unknown_status", MessageKey::LifecycleUnknownStatus, params, false)
}

/// Bước 2 + bước 3 của khuôn bốn bước — DÙNG CHUNG bởi [`set_chapter_status`] và
/// [`set_work_status_override`] sau khi bước 1 (giao dịch ghi) đã commit thành công. Cập
/// nhật `open.meta` tại chỗ, và trả [`WorkLifecycle`] đọc được ngay từ giá trị vừa dựng lại.
///
/// # Lỗi
/// Lỗi ghi `meta.json` **NÓI RA**, không nuốt — cùng lý lẽ `project.rs:317-332`: đường dựng
/// lại KHÔNG tự chạy lần nữa, nên một `meta.json` vắng mặt/cũ nằm đó cho tới lượt ghi kế
/// tiếp nếu lỗi này bị nuốt.
///
/// ⚠️ `pub(crate)` từ 2026-08-29 (Story 5.8) — **hôm nay riêng tư, tên GIỮ NGUYÊN.** Bốn
/// thao tác tổ chức Chương mới (`commands::chapter::rename_chapter` ·
/// `move_chapter` · `merge_chapter_into_previous` · `split_chapter_at_segment`) đi qua ĐÚNG
/// khuôn bốn bước mà hàm này chở (bước 2 + bước 3), và tái dùng nó là cách duy nhất không
/// đúc một bản chép thứ hai của cùng một khuôn — đo 2026-08-27 đã cho thấy chỗ nối bước 4
/// **chưa có ai canh** khi khuôn bốn bước bị chép tay một lần rồi (xem
/// `reindex_after_lifecycle_write`). Đổi TÊN ở đây sẽ mồ côi mọi tham chiếu trong ba tệp
/// story (`commands/chapter.rs`, `commands/lifecycle.rs`, tài liệu review), nên tên giữ
/// nguyên, chỉ độ hiển thị đổi.
pub(crate) fn write_lifecycle_after_change(open: &mut OpenWork) -> Result<WorkLifecycle, IpcError> {
    let meta = WorkMeta::rebuild_from_store(&open.store)?;

    if let Err(err) = meta.write_atomic(&open.dir) {
        // ⚠️ Chẩn đoán KHÔNG lặp lại chuỗi "meta.json" viết thẳng (2026-08-28, Story 5.5) --
        // `meta_write_boundary.rs` khoá tên tệp CHỈ ở `core/library/meta.rs`; "work metadata
        // cache" mô tả đúng vai trò của tệp mà không đúc thêm một bản chép tên tệp thứ hai.
        //
        // 🔵 ĐO (2026-08-28, vòng rà thứ hai) -- lượt đổi chữ này KHÔNG làm người vận hành
        // mất đường lần dấu: `{err}` là `MetaError` mà `write_atomic` trả về, và
        // `MetaError::Io::fmt` (`core/library/meta.rs`) in NGUYÊN đường dẫn đầy đủ, luôn kết
        // thúc bằng `meta.json` (`meta[<duong-dan>/meta.json] io failed: <chi tiet>`). Câu
        // log ở đây chỉ đổi phần TIỀN TỐ mô tả thao tác; tên tệp thật vẫn tới log qua `{err}`.
        eprintln!(
            "lifecycle[{}] work metadata cache write failed after commit: {err}",
            open.dir.display()
        );
        return Err(crate::core::library::WorkError::from(err).into());
    }

    let lifecycle = WorkLifecycle::from(&meta);
    open.meta = meta;
    Ok(lifecycle)
}

/// Đọc trạng thái vòng đời hiện thời của Tác phẩm **đang mở** — hàm thuần, đây là thứ test
/// gọi. Không ghi gì; đọc thẳng `open.meta`, thứ [`WorkMeta::rebuild_from_store`] đã tính sẵn
/// ở lượt tạo/ghi gần nhất.
///
/// # Lỗi
/// chưa Tác phẩm nào mở ⇒ `work.none_open` (tái dùng [`no_work_open`]).
pub fn read_work_lifecycle(open: Option<&OpenWork>) -> Result<WorkLifecycle, IpcError> {
    let open = open.ok_or_else(no_work_open)?;
    Ok(WorkLifecycle::from(&open.meta))
}

/// Đặt trạng thái vòng đời của MỘT Chương — hàm thuần, đây là thứ test gọi. Story 5.4, FR5.
///
/// Giá trị được cưỡng chế ở TẦNG RUST (§Always) TRƯỚC khi chạm SQL: một giá trị ngoài danh
/// mục bị từ chối tại đây, nên [`open.store.write`](crate::core::store::Store::write) không
/// bao giờ chạy cho một giá trị lạ — đúng §I/O Matrix "không một lượt ghi nào chạy".
///
/// # Lỗi
/// - chưa Tác phẩm nào mở ⇒ `work.none_open`;
/// - `status` ngoài danh mục bốn giá trị ⇒ `err.lifecycle.unknown_status` `{status}`, KHÔNG
///   một lượt ghi nào chạy;
/// - `chapter_id` không có trong Tác phẩm đang mở ⇒ `segment.chapter_not_found` (tái dùng
///   [`chapter_not_found`]) — câu `UPDATE` chạy nhưng khớp 0 hàng, tức không một byte nào đổi;
/// - lỗi ghi `meta.json` sau khi giao dịch Chương đã commit ⇒ `work.create_failed` (tái dùng
///   `WorkError`, xem [`write_lifecycle_after_change`]) — Chương ĐÃ đổi trạng thái trên đĩa,
///   lỗi này chỉ nói chỉ mục/`meta.json` chưa theo kịp.
pub fn set_chapter_status(
    open: Option<&mut OpenWork>,
    chapter_id: i64,
    status: &str,
) -> Result<WorkLifecycle, IpcError> {
    let open = open.ok_or_else(no_work_open)?;

    let parsed = LifecycleStatus::from_wire(status).ok_or_else(|| unknown_status(status))?;
    let wire_status = parsed.as_str();

    // Bước 1 — CHỈ SQL, cùng khuôn `create_work`. `UPDATE ... WHERE id = ?` khớp 0 hàng khi
    // `chapter_id` không tồn tại trong Tác phẩm đang mở -- KHÔNG một byte nào đổi, đúng câu
    // "không một lượt ghi nào chạy" của §I/O Matrix dù câu SQL vẫn được gửi đi.
    let touched: usize = open.store.write(move |tx: &Transaction<'_>| {
        tx.execute(
            "UPDATE chapter SET status = ?1, updated_at = strftime('%Y-%m-%dT%H:%M:%fZ','now') \
             WHERE id = ?2",
            (wire_status, chapter_id),
        )
    })?;

    if touched == 0 {
        return Err(chapter_not_found(chapter_id));
    }

    write_lifecycle_after_change(open)
}

/// Ghi đè (hoặc bỏ ghi đè) trạng thái vòng đời của TÁC PHẨM đang mở — hàm thuần, đây là thứ
/// test gọi. Story 5.4, FR6.
///
/// `status = None` ⇒ bỏ ghi đè (`status_override` về `NULL`, đọc lại ra giá trị SUY RA hiện
/// thời); `status = Some(raw)` ⇒ ghi đè bằng giá trị đó sau khi cưỡng chế qua
/// [`LifecycleStatus::from_wire`].
///
/// # Lỗi
/// - chưa Tác phẩm nào mở ⇒ `work.none_open`;
/// - `status` là `Some` mang một giá trị ngoài danh mục ⇒ `err.lifecycle.unknown_status`
///   `{status}`, KHÔNG một lượt ghi nào chạy;
/// - lỗi ghi `meta.json` sau khi giao dịch đã commit ⇒ `work.create_failed`, cùng lý lẽ
///   [`set_chapter_status`].
pub fn set_work_status_override(
    open: Option<&mut OpenWork>,
    status: Option<&str>,
) -> Result<WorkLifecycle, IpcError> {
    let open = open.ok_or_else(no_work_open)?;

    // Cưỡng chế TRƯỚC khi chạm SQL (§Always) -- một giá trị lạ dừng ở đây, `open.store.write`
    // không bao giờ chạy.
    let validated: Option<&'static str> = match status {
        Some(raw) => Some(LifecycleStatus::from_wire(raw).ok_or_else(|| unknown_status(raw))?.as_str()),
        None => None,
    };

    // Bước 1 -- CHỈ SQL. `?1` nhận `NULL` khi `validated` là `None` (rusqlite ánh xạ
    // `Option<&str>` sang NULL/TEXT) -- đúng §Always "NULL-hoặc-giá-trị, không một cờ boolean
    // riêng".
    open.store.write(move |tx: &Transaction<'_>| {
        tx.execute("UPDATE work SET status_override = ?1 WHERE id = 1", (validated,))
    })?;

    write_lifecycle_after_change(open)
}

/// **Hàm thuần** — bước 4 của khuôn bốn bước: đưa lượt ghi vừa commit vào `library-index.db`.
///
/// ─────────────────────────────────────────────────────────────────────────────
/// 🔴 VÌ SAO NÓ Ở ĐÂY CHỨ KHÔNG Ở `mod wire` — MỘT ĐỐI CHỨNG ĐÃ CHẠY VÀ ĐÃ ĐỎ
/// ─────────────────────────────────────────────────────────────────────────────
/// Bản dựng đầu của Story 5.4 đặt bước 4 **bên trong** `mod wire` (một hàm riêng tư
/// `reindex_after_lifecycle_write(&AppHandle)`). Nó chạy đúng, nhưng **không phép kiểm nào
/// với tới được**: `tests/**` không dựng được một `tauri::AppHandle`, nên mọi thứ sống trong
/// `mod wire` chỉ được so CHUỖI, không được chạy. ⚠️ **Đo 2026-08-27:** gỡ hẳn hai lời gọi
/// bước 4 rồi chạy `cargo test --locked` cho **0 failed** trên toàn bộ 34 binary — chỗ nối
/// giữa "đã ghi trạng thái" và "chỉ mục Library đã biết" **chưa có ai canh**, đúng lớp lỗi
/// mà `AGENTS.md::Known pitfalls` gọi tên (*"một bộ test xanh KHÔNG chứng minh chỗ nối mới
/// được canh"*, Epic 3 dính năm lần trong bảy ngày).
///
/// ⇒ Quy tắc *"ghi thành công thì đưa vào chỉ mục"* chuyển xuống tầng hàm thuần, nhận
/// `Option<&Indexer>` + `Option<&Store>` + `&Path` đúng khuôn
/// [`crate::commands::library::rescan`]. `mod wire` quay về đúng vai của nó: lấy `State`,
/// không giữ quy tắc nào (`src-tauri/AGENTS.md`).
///
/// ⚠️ **Không trả `Result`, có chủ ý.** Một lượt reindex trượt KHÔNG được biến một lượt ghi
/// trạng thái **đã thành công trên đĩa** thành lỗi IPC — `.atproj` mới là dữ liệu người dùng,
/// chỉ mục là dẫn xuất (AD-8). Mọi đường trượt đi ra chẩn đoán, cùng khuôn
/// [`crate::commands::project::wire::reindex_library`].
pub fn reindex_after_lifecycle_write(indexer: Option<&Indexer>, global: Option<&Store>, root: &Path) {
    let Some(indexer) = indexer else {
        eprintln!("lifecycle[reindex] Indexer chua duoc quan ly -- bo qua luot dua vao chi muc");
        return;
    };
    match indexer.rebuild(root, global) {
        // Cùng đường chẩn đoán CHUNG mà `lib.rs::open_library_index`,
        // `project::wire::reindex_library` và `library::rescan` đã dùng — chỗ gọi thứ TƯ không
        // được đứng ngoài quy ước đó.
        Ok(outcome) => outcome.log_if_notable("lifecycle"),
        Err(err) => eprintln!("lifecycle[reindex] rebuild that bai: {err}"),
    }
}

/// **Vị từ bước 4, khai ĐÚNG MỘT CHỖ** — *"chỉ reindex sau một lượt ghi THÀNH CÔNG"*.
///
/// 🔴 Tồn tại vì lượt rà 2026-08-28 bắt được: `mod wire` từng **chép lại** trình tự
/// *"gọi hàm thuần → nhả khoá → `if result.is_ok()` → reindex"* một lần thứ hai, nên hai ca
/// hợp đồng canh đường `*_indexed` còn sản phẩm chạy một đường viết tay khác. Hai bản cài đặt
/// của cùng một quy tắc là hai thứ phải giữ khớp nhau **bằng kỷ luật** — đúng lớp lỗi mà
/// `AGENTS.md::Known pitfalls` ghi. Nay cả hai đường đi qua hàm này.
///
/// ⚠️ Nhận `Result` **đã tính sẵn** chứ không nhận một closure: `mod wire` phải chạy phần ghi
/// **bên trong** vùng khoá `OpenWorkState` rồi **nhả khoá** trước bước 4 (không giữ khoá qua
/// một lượt quét đĩa), nên nó không thể đưa phần ghi vào đây dưới dạng closure. Hình dạng này
/// là hình dạng DUY NHẤT dùng chung được cho cả hai chỗ gọi.
///
/// Vì sao lỗi ở bước 4 không đổi `result`: xem [`reindex_after_lifecycle_write`].
pub fn finish_lifecycle_write<T>(
    result: Result<T, IpcError>,
    indexer: Option<&Indexer>,
    global: Option<&Store>,
    root: &Path,
) -> Result<T, IpcError> {
    if result.is_ok() {
        reindex_after_lifecycle_write(indexer, global, root);
    }
    result
}

/// [`set_chapter_status`] **cộng bước 4** — hàm thuần, đây là thứ test gọi để chứng minh
/// hàng `library_work` đã mang giá trị mới sau một lượt ghi.
///
/// Bước 4 chạy **CHỈ** sau một lượt ghi thành công: một `status` ngoài danh mục hoặc một
/// `chapter_id` không tồn tại không được kéo theo một lượt quét đĩa vô ích.
pub fn set_chapter_status_indexed(
    open: Option<&mut OpenWork>,
    indexer: Option<&Indexer>,
    global: Option<&Store>,
    root: &Path,
    chapter_id: i64,
    status: &str,
) -> Result<WorkLifecycle, IpcError> {
    finish_lifecycle_write(set_chapter_status(open, chapter_id, status), indexer, global, root)
}

/// [`set_work_status_override`] **cộng bước 4** — cùng lý lẽ [`set_chapter_status_indexed`].
pub fn set_work_status_override_indexed(
    open: Option<&mut OpenWork>,
    indexer: Option<&Indexer>,
    global: Option<&Store>,
    root: &Path,
    status: Option<&str>,
) -> Result<WorkLifecycle, IpcError> {
    finish_lifecycle_write(set_work_status_override(open, status), indexer, global, root)
}

/// Ba vỏ `#[tauri::command]`. **Không một quy tắc nào sống ở đây.**
pub mod wire {
    use super::{IpcError, WorkLifecycle};
    use crate::commands::project::OpenWorkState;
    use crate::core::library::indexer::Indexer;
    use crate::core::store::Store;

    /// Vỏ IPC của [`super::read_work_lifecycle`]. Đọc thuần -- không ghi, không cần
    /// `(async)` (cùng khuôn `commands::chapter::wire::read_open_chapter`).
    #[tauri::command]
    pub fn read_work_lifecycle(app: tauri::AppHandle) -> Result<WorkLifecycle, IpcError> {
        use tauri::Manager as _;

        let Some(state) = app.try_state::<OpenWorkState>() else {
            return super::read_work_lifecycle(None);
        };
        let guard = state.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
        super::read_work_lifecycle(guard.as_ref())
    }

    /// Vỏ IPC của [`super::set_chapter_status`].
    ///
    /// 🔴 `(async)` KHÔNG PHẢI TRANG TRÍ — bước 4 của khuôn bốn bước (`reindex_library`) là
    /// một lượt quét TOÀN BỘ thư mục gốc Library; chạy nó trên luồng chính chặn đúng vòng
    /// lặp sự kiện mà Story 3.10b đã đo là bế tắc. Cổng canh:
    /// `config_invariants.rs::the_blocking_wires_run_off_the_main_thread`.
    #[tauri::command(async)]
    pub fn set_chapter_status(
        app: tauri::AppHandle,
        chapter_id: i64,
        status: String,
    ) -> Result<WorkLifecycle, IpcError> {
        use tauri::Manager as _;

        let Some(state) = app.try_state::<OpenWorkState>() else {
            return super::set_chapter_status(None, chapter_id, &status);
        };
        // 🔴 Khoá `OpenWorkState` NHẢ trước bước 4: giữ nó qua một lượt quét đĩa sẽ chặn mọi
        // lệnh khác đọc Tác phẩm đang mở (khuôn "khoá hai lần ngắn" của `commands::project`).
        // Vì thế vỏ chạy phần ghi TRONG vùng khoá rồi đưa `Result` đã tính sẵn qua
        // [`super::finish_lifecycle_write`] -- vị từ "chỉ reindex khi ghi thành công" khai
        // ĐÚNG MỘT CHỖ, dùng chung với `super::set_chapter_status_indexed` mà hợp đồng canh.
        let result = {
            let mut guard = state.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
            super::set_chapter_status(guard.as_mut(), chapter_id, &status)
        };
        finish_with_reindex(&app, result)
    }

    /// Vỏ IPC của [`super::set_work_status_override`]. `status: Option<String>` -- `invoke()`
    /// gửi `null` khi webview muốn BỎ ghi đè (xem `config/lifecycle.ts`).
    ///
    /// 🔴 `(async)` — cùng lý do [`set_chapter_status`].
    #[tauri::command(async)]
    pub fn set_work_status_override(
        app: tauri::AppHandle,
        status: Option<String>,
    ) -> Result<WorkLifecycle, IpcError> {
        use tauri::Manager as _;

        let Some(state) = app.try_state::<OpenWorkState>() else {
            return super::set_work_status_override(None, status.as_deref());
        };
        let result = {
            let mut guard = state.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
            super::set_work_status_override(guard.as_mut(), status.as_deref())
        };
        finish_with_reindex(&app, result)
    }

    /// Bước 4 cho cả hai vỏ ghi ở trên: lấy `State` rồi giao TRỌN quyết định cho
    /// [`super::finish_lifecycle_write`]. **Không một quy tắc nào sống ở đây** — hàm này chỉ
    /// biết cách lấy `Indexer`/`Store`/`root` ra khỏi `AppHandle`.
    ///
    /// 🔵 **SỬA (2026-08-28, lượt rà) — đổi tên từ `reindex_after_lifecycle_write`.** Tên cũ
    /// TRÙNG với hàm thuần cùng tên ở module cha, chỉ khác chữ ký; `grep` và "go to definition"
    /// đều nhập nhằng giữa hai thứ khác vai. Tên mới nói đúng vai của nó (giải quyết `State`
    /// rồi *kết thúc* lượt ghi), và vị từ `is_ok()` đã chuyển hẳn sang module cha.
    ///
    /// Không giải quyết được `root` ⇒ ghi chẩn đoán rồi trả `result` NGUYÊN VẸN: một lượt ghi
    /// trạng thái đã thành công trên đĩa không được biến thành lỗi IPC vì chỉ mục (AD-8).
    fn finish_with_reindex(
        app: &tauri::AppHandle,
        result: Result<WorkLifecycle, IpcError>,
    ) -> Result<WorkLifecycle, IpcError> {
        use tauri::Manager as _;

        let store = app.try_state::<Store>();
        let root = match crate::commands::project::resolve_library_root(app, store.as_deref()) {
            Ok(root) => root,
            Err(err) => {
                eprintln!("lifecycle[reindex] khong giai quyet duoc thu muc goc Library: {err:?}");
                return result;
            }
        };
        let indexer = app.try_state::<Indexer>();
        super::finish_lifecycle_write(result, indexer.as_deref(), store.as_deref(), &root)
    }
}

#[cfg(test)]
mod tests {
    use super::{unknown_status, WorkLifecycle};
    use crate::core::library::meta::WorkMeta;
    use crate::core::lifecycle::LifecycleStatus;

    #[test]
    fn work_lifecycle_from_meta_copies_both_fields() {
        let meta = WorkMeta {
            meta_schema_version: 2,
            work_id: "id".to_owned(),
            name: "N".to_owned(),
            source_lang: "en".to_owned(),
            genre: String::new(),
            created_at: "2026-08-01T00:00:00.000Z".to_owned(),
            updated_at: "2026-08-01T00:00:00.000Z".to_owned(),
            chapter_count: 1,
            // ⚠️ Qua `LifecycleStatus::Paused.as_str()`, không chuỗi "paused" viết thẳng —
            // §Verification của story: "mọi lần xuất hiện ở vị trí mã nằm trong
            // core/lifecycle/mod.rs; chỗ khác chỉ được nhắc qua LifecycleStatus::…".
            status: Some(LifecycleStatus::Paused.as_str().to_owned()),
            status_is_override: true,
            chapter_done_count: Some(0),
        };
        let lifecycle = WorkLifecycle::from(&meta);
        assert_eq!(lifecycle.status.as_deref(), Some(LifecycleStatus::Paused.as_str()));
        assert!(lifecycle.status_is_override);
    }

    #[test]
    fn unknown_status_carries_the_offending_value_as_a_param() {
        let err = unknown_status("archived");
        // `IpcError` không lộ `params` công khai bên ngoài serialize -- kiểm qua JSON, đúng
        // khuôn các ca khác của kho kiểm `IpcError` bằng serialize thay vì accessor riêng.
        let json = serde_json::to_value(&err).expect("IpcError serialize duoc");
        assert_eq!(json["code"], "lifecycle.unknown_status");
        assert_eq!(json["params"]["status"], "archived");
    }
}
