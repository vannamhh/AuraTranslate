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
}

/// Thư mục gốc mặc định chứa mọi `.atproj` — `~/Documents/AuraTranslate/` (AD-23).
///
/// ⛔ Không viết cứng `$HOME` — `app.path().document_dir()` là đường duy nhất (NFR14).
/// ⚠️ Scope động của AD-23 hôm nay được cưỡng chế bằng **kỷ luật mã Rust** (module này là
/// nơi DUY NHẤT gọi hàm này), ⛔ không phải bởi framework — xem Completion Notes của story
/// `1-15…md`.
pub fn default_library_root(app: &tauri::AppHandle) -> Result<PathBuf, IpcError> {
    use tauri::Manager as _;

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
/// nguyên tử NGAY SAU giao dịch. Bất kỳ bước nào trượt ⇒ dọn thư mục, ⛔ không để lại
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

    // 🔴 Quyết định #3: job ghi CHỈ SQL — ⛔ không `fs::write` nào bên trong closure này.
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
        Ok(())
    });

    if let Err(err) = write_result {
        store.close();
        remove_folder(&dir);
        return Err(err.into());
    }

    // Quyết định #3: `meta.json` ghi NGAY SAU KHI giao dịch commit, ở tầng THAO TÁC —
    // dựng lại từ `project.db` vừa ghi (AD-33), ⛔ không giữ dữ liệu song song mà trôi.
    let meta = match WorkMeta::rebuild_from_store(&store) {
        Ok(meta) => meta,
        Err(err) => {
            store.close();
            remove_folder(&dir);
            return Err(err.into());
        }
    };

    // 🔴 Loi ghi meta.json PHAI noi ra, ⛔ KHONG duoc nuot — code review 2026-08-06.
    //
    // Quyet dinh #3 chap nhan **cua so SAP MAY** giua commit va fs::write, va no dung:
    // AD-33 noi meta.json dung lai duoc tu project.db. Nhung no ⛔ KHONG cho phep di tiep
    // khi ham TRA VE Err. Hai chuyen khac han nhau:
    //   - sap may  ⇒ ⛔ khong ai chay duoc ma dep, va lan mo sau dung lai duoc;
    //   - Err      ⇒ tien trinh van song, va di tiep nghia la tra ve Ok cho mot .atproj
    //                chi co HAI thanh phan — pha AC2, va pha AC3 (Library doc metadata
    //                ma khong mo SQLite) ngay tu luc tao.
    //
    // Va duong dung lai KHONG TU CHAY: `rebuild_from_store` ⛔ khong co mot cho goi san
    // pham nao (story nay ⛔ khong dung man hinh "mo lai mot .atproj"), nen mot meta.json
    // vang mat nam do cho toi Epic 5.
    //
    // ⇒ Cuon lai TRON VEN. An toan vi `create_work_folder` tao DOC QUYEN: `dir` chac chan
    // la thu muc cua chinh luot goi nay, ⛔ khong phai du lieu co san.
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
/// ⚠️ `Mutex`, ⛔ không `RwLock`: đúng một Tác phẩm mở tại một thời điểm, và mọi thao tác
/// đọc/ghi field của nó (thay Tác phẩm khác, đóng lúc thoát) đều là **thao tác độc quyền**
/// — không có nhánh "nhiều reader cùng lúc" nào ở tầng state này (khác hẳn `Store::read`
/// bên trong, nơi pool nhiều kết nối đã lo phần đó).
pub type OpenWorkState = std::sync::Mutex<Option<OpenWork>>;

/// Thay Tác phẩm đang mở (nếu có) bằng `new_work` — **Store cũ tự đóng qua `Drop`**.
///
/// ⚠️ Nếu `OpenWorkState` chưa từng được `app.manage(...)` (lỗi cấu hình `setup()`, không
/// phải đường sản phẩm bình thường), `new_work` bị drop ngay khi hàm này return — Tác
/// phẩm vừa tạo đóng lại tức thì. Đây là im lặng có chủ ý: cùng khuôn
/// `close_global_store`/`try_state`, ⛔ không panic khi state vắng mặt.
fn replace_open_work(app: &tauri::AppHandle, new_work: OpenWork) {
    use tauri::Manager as _;

    if let Some(state) = app.try_state::<OpenWorkState>() {
        let mut guard = state.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
        *guard = Some(new_work);
    }
}

/// Hai vỏ `#[tauri::command]`. ⛔ **Không một quy tắc nào sống ở đây.**
pub mod wire {
    use super::{IpcError, OpenWork, default_library_root, replace_open_work};
    use crate::core::library::WorkMeta;

    /// Thứ hai lệnh trả về — [`WorkMeta`] **cộng đường dẫn thư mục trên đĩa**.
    ///
    /// ─────────────────────────────────────────────────────────────────────────────
    /// 🔴 VÌ SAO `folder` PHẢI ĐI RA — AC6 ⛔ KHÔNG GIAO ĐƯỢC NẾU THIẾU NÓ
    /// ─────────────────────────────────────────────────────────────────────────────
    /// AC6 hứa với người dùng *"copy thư mục là đủ để sao lưu"*. Một lời hứa về **một
    /// thư mục cụ thể** mà ⛔ không nói thư mục đó ở đâu thì ⛔ không thực hiện được.
    /// Và tên thư mục ⛔ **không** suy ra được từ `meta.name`: `sanitize_name` thay ký tự
    /// cấm (`Tập 1: Khởi đầu` → `Tập 1_ Khởi đầu`), và trùng tên thì thêm hậu tố
    /// ` (2)` — nên chỉ Rust mới biết tên thật. Code review 2026-08-06.
    ///
    /// ⚠️ `#[serde(rename_all = ...)]` ⛔ KHÔNG đặt — cùng luật với mọi struct qua biên.
    #[derive(Debug, Clone, serde::Serialize)]
    pub struct CreatedWork {
        /// Metadata vừa ghi xuống `meta.json`.
        pub meta: WorkMeta,
        /// Đường dẫn **tuyệt đối** tới `<Tên>.atproj/` trên máy này.
        ///
        /// ⚠️ Đây là một giá trị **qua IPC**, ⛔ không phải một giá trị **ghi xuống đĩa** —
        /// AC5 cấm đường dẫn tuyệt đối bên trong `meta.json`/`project.db`, ⛔ không cấm
        /// nói cho người dùng biết Tác phẩm của họ nằm ở đâu.
        pub folder: String,
    }

    impl CreatedWork {
        /// Gói một [`OpenWork`] thành thứ đi qua dây được — `Store` ⛔ không `Serialize`.
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
