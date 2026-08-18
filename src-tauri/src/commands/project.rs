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

use uuid::Uuid;

use crate::core::i18n::IpcError;
use crate::core::library::{WorkMeta, create_work_folder, remove_folder};
use crate::core::segment::import::{ImportedChapter, import_file, import_text};
use crate::core::segment::split::split_source_text;
use crate::core::store::{Store, StoreSpec, Transaction};

/// Trạng thái vòng đời ban đầu của mọi Chương mới (FR5) — **tạm**, chờ Story 2.5 dựng
/// máy trạng thái đầy đủ. Chuỗi tự do ở tầng SQL, cùng khuôn `config_value.kind`.
const CHAPTER_STATUS_NOT_STARTED: &str = "not_started";

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
/// ⚠️ Scope động của AD-23 hôm nay được cưỡng chế bằng **kỷ luật mã Rust** (module này là
/// nơi DUY NHẤT gọi hàm này), không phải bởi framework — xem Completion Notes của story
/// `1-15…md`.
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
        crate::core::library::ProjectError::CreateFailed {
            detail: format!("resolve document_dir: {e}"),
        }
    })?;

    Ok(documents.join(DOCUMENTS_SUBFOLDER))
}

/// **Hàm thuần** — tạo một Tác phẩm mới trên đĩa từ một [`ImportedChapter`] đã có sẵn.
///
/// Thứ tự: dựng thư mục (`core::library::atproj`) → mở `project.db`
/// (`StoreSpec::project`) → **ghi** hàng `work` + hàng `chapter` trong MỘT giao dịch →
/// dựng lại `meta.json` từ `project.db` vừa commit (Quyết định #3, AD-33) → ghi `meta.json`
/// nguyên tử NGAY SAU giao dịch. Bất kỳ bước nào trượt ⇒ dọn thư mục, không để lại
/// `.atproj/` nửa vời (AC8).
///
/// # Lỗi
/// - dựng thư mục trượt ⇒ `project.create_failed`;
/// - mở/ghi `project.db` trượt ⇒ lỗi kho (`store.*`), qua `From<StoreError>`.
pub fn create_work(
    documents_root: &Path,
    name: &str,
    source_lang: &str,
    genre: &str,
    imported: ImportedChapter,
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
    let source_text = imported.source_text;

    // 🔴 Story 2.1, AC3 + AD-39: ranh giới segment tính **ở đây, một lần, lúc nhập** — và
    // không đường mã nào tính lại lúc nạp Chương.
    //
    // 🔴 Và nó chạy **NGOÀI** closure ghi, có chủ ý. AD-11 giữ **một** writer duy nhất nối
    // tiếp (một `Connection` `move` vào một thread, job đi qua `mpsc::channel`), nên thời
    // gian CPU bên trong closure **chặn mọi lượt ghi khác của tiến trình**. Một Chương dài
    // đi qua bộ tách trong closure là một lượt khoá hàng đợi ghi mà auto-save của Editor
    // (NFR2) phải xếp sau — cùng Quyết định #3 của Story 1.15 đã cấm `fs::write` ở đó.
    let segments = split_source_text(&source_text, source_lang);

    // 🔴 Quyết định #3: job ghi CHỈ SQL — không `fs::write` nào bên trong closure này.
    let write_result = store.write(move |tx: &Transaction<'_>| {
        tx.execute(
            "INSERT INTO work (id, work_id, name, source_lang, genre, created_at, updated_at) \
             VALUES (1, ?1, ?2, ?3, ?4, strftime('%Y-%m-%dT%H:%M:%fZ','now'), \
             strftime('%Y-%m-%dT%H:%M:%fZ','now'))",
            (&work_id, &name_owned, &source_lang_owned, &genre_owned),
        )?;
        tx.execute(
            "INSERT INTO chapter (ord, title, source_text, status, created_at, updated_at) \
             VALUES (1, NULL, ?1, ?2, strftime('%Y-%m-%dT%H:%M:%fZ','now'), \
             strftime('%Y-%m-%dT%H:%M:%fZ','now'))",
            (&source_text, CHAPTER_STATUS_NOT_STARTED),
        )?;

        // 🔴 AC13 — segment ghi xuống **CÙNG** giao dịch với hàng `chapter` sinh ra chúng.
        // Một Chương tồn tại mà segment của nó chưa tồn tại là **đúng** trạng thái
        // `segment_count = 0` mà `deferred-work.md:542` bắt story này dọn; dựng lại nó ở
        // đường nhập mới là dựng lại chính món nợ.
        //
        // `last_insert_rowid()` đọc **trong** giao dịch, ngay sau lượt chèn của chính nó —
        // `Store::write` giữ một writer duy nhất nối tiếp, nên không lượt chèn nào khác
        // chen được vào giữa hai dòng này.
        let chapter_id = tx.last_insert_rowid();
        crate::commands::segment::insert_segments(tx, chapter_id, &segments)?;
        // 🔵 2026-08-18 (Story 2.11) — `chapter_id` đi RA khỏi closure thay vì bị bỏ.
        // `OpenWork::chapter_id` phải được đặt bằng chính hàng vừa chèn; đọc lại nó bằng
        // một câu `ORDER BY ord LIMIT 1` sau giao dịch là dựng lại đúng lối suy-ra-động mà
        // trường ấy tồn tại để xoá.
        Ok(chapter_id)
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
        eprintln!(
            "project[{}] meta.json write failed after commit, rolling back: {err}",
            dir.display()
        );
        store.close();
        remove_folder(&dir);
        return Err(crate::core::library::ProjectError::from(err).into());
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
pub fn create_work_from_text(
    documents_root: &Path,
    name: &str,
    source_lang: &str,
    genre: &str,
    text: String,
) -> Result<OpenWork, IpcError> {
    create_work(documents_root, name, source_lang, genre, import_text(text))
}

/// **Hàm thuần** — nhánh tệp của AC1 (kéo-thả **hoặc** ô nhập đường dẫn; cả hai đã
/// resolve thành một `path` thật ở lớp gọi, xem AD-1/AD-16).
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
    let imported = import_file(path)?;
    create_work(documents_root, name, source_lang, genre, imported)
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
    let mut guard = mutex.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
    guard.replace(new)
}

#[cfg(test)]
mod tests {
    use super::swap_locked;
    use std::sync::{Arc, Mutex};

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
        assert!(first.is_none(), "mutex rong luc dau ⇒ khong co gia tri CU nao");

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
        assert!(last.is_some(), "mutex phai con dung mot gia tri sau ca hai luot swap");
        drop(last);
    }
}

/// Hai vỏ `#[tauri::command]`. **Không một quy tắc nào sống ở đây.**
pub mod wire {
    use super::{IpcError, OpenWork, default_library_root, replace_open_work};
    use crate::core::library::WorkMeta;

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

    /// Vỏ IPC của [`super::create_work_from_text`].
    ///
    /// ⚠️ Trả về [`CreatedWork`] — vỏ **không** trả `OpenWork` ra ngoài (nó mang `Store`,
    /// không `Serialize`); quản lý `OpenWork` trong state qua [`replace_open_work`].
    #[tauri::command]
    pub fn create_work_from_text(
        app: tauri::AppHandle,
        name: String,
        source_lang: String,
        genre: String,
        text: String,
    ) -> Result<CreatedWork, IpcError> {
        let root = default_library_root(&app)?;
        let opened = super::create_work_from_text(&root, &name, &source_lang, &genre, text)?;
        let created = CreatedWork::from_open(&opened);
        replace_open_work(&app, opened);
        Ok(created)
    }

    /// Vỏ IPC của [`super::create_work_from_file`].
    #[tauri::command]
    pub fn create_work_from_file(
        app: tauri::AppHandle,
        name: String,
        source_lang: String,
        genre: String,
        path: String,
    ) -> Result<CreatedWork, IpcError> {
        let root = default_library_root(&app)?;
        let opened = super::create_work_from_file(
            &root,
            &name,
            &source_lang,
            &genre,
            std::path::Path::new(&path),
        )?;
        let created = CreatedWork::from_open(&opened);
        replace_open_work(&app, opened);
        Ok(created)
    }
}
