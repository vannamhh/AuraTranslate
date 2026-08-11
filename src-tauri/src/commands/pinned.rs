//! Bề mặt IPC cho **mục đã ghim** — Story 1.20, AC2 · AC3.
//!
//! Cùng khuôn `commands::config`: hàm thuần nhận `Option<&Store>` trước,
//! `#[tauri::command]` chỉ là vỏ mỏng trong [`wire`].
//!
//! ─────────────────────────────────────────────────────────────────────────────
//! 🔴 PHẠM VI TOÀN ỨNG DỤNG (`global.db`) — Ice ký lại 2026-08-11
//! ─────────────────────────────────────────────────────────────────────────────
//! Ngày 2026-08-10 Quyết định #1 chốt `project.db`. Một phép đo hôm sau lật nó: **không
//! tồn tại đường mở lại một `.atproj` từ đĩa** *(11 command IPC, không cái nào đọc một Tác
//! phẩm có sẵn)*, nên với ghim ở `project.db`, đóng app rồi mở lại là không Tác phẩm nào
//! đang mở và bộ ghim **không có đường nào để đọc tới** — AC3 đúng trên đĩa mà không bao
//! giờ đúng trên màn hình. `global.db` mở một lần ở `setup()` và sống suốt vòng đời tiến
//! trình, nên nó là chỗ duy nhất AC3 có nghĩa được hôm nay. Lý lẽ đầy đủ ở doc-comment
//! của [`crate::core::store::PINNED_ENTRY_DDL`].
//!
//! ─────────────────────────────────────────────────────────────────────────────
//! 🔴 GHI NGAY, KHÔNG DEBOUNCE
//! ─────────────────────────────────────────────────────────────────────────────
//! AD-35: một thao tác rời rạc dứt khoát của người dùng **không** được định tuyến qua bộ
//! đệm gõ — *"một thao tác đã hoàn tất nằm chờ tới 5 giây và biến mất nếu app sập, dù
//! người dùng thấy nó đã xong trên màn hình"*. Ghim là đúng loại thao tác đó, nên nó đi
//! thẳng qua `Store::write` (writer nối tiếp, AD-11).
//!
//! ─────────────────────────────────────────────────────────────────────────────
//! 🔴 KHÔNG `MessageKey` MỚI, VÀ NAY MỌI LỖI THUỘC TỪ VỰNG **KHO**
//! ─────────────────────────────────────────────────────────────────────────────
//! Kho vắng mặt ⇒ `store.open_failed`; đường đọc/ghi trượt ⇒
//! `store.read_failed`/`store.write_failed` qua `From<StoreError>`. Cùng ba nhánh mà
//! `commands::config` đã đi qua, và **không** một `MessageKey` mới nào.
//!
//! ⚠️ Lượt đổi phạm vi gỡ luôn nhánh `project.no_work_open` mà bản đầu tái dùng từ Story
//! 1.16 — mục ghim nay không hỏi Tác phẩm nào cả, nên câu đó **sai** ở đây.
//!
//! ⚠️ Mọi chuỗi trong tệp này viết **KHÔNG DẤU** — `scripts/check-i18n.mjs` Kiểm A quét
//! `src-tauri/**/*.rs` và tệp này không nằm trong danh sách miễn trừ.

use crate::core::i18n::IpcError;
use crate::core::store::{Row, SqlResult, Store, StoreError, StoreKind, Transaction};

/// Kho vang mat ⇒ loi *mo kho*, va do la cau dung theo nghia den.
///
/// 🔴 Di qua `From<StoreError> for IpcError`, khong dung `IpcError` bang struct literal —
/// cung ly do va cung khuon `commands::config::store_is_missing`: `From` la cho duy nhat
/// mot `StoreError` chon khoa cua no, va "duy nhat" chi co gia tri neu khong ai di vong.
fn store_is_missing() -> IpcError {
    StoreError::OpenFailed {
        store: StoreKind::Global,
        detail: "the global store was never managed; see lib.rs::open_global_store".to_owned(),
    }
    .into()
}

/// Một hàng `pinned_entry` đã đọc ra — hình dạng trên dây.
///
/// ⚠️ `#[serde(rename_all = ...)]` KHÔNG đặt — cùng luật với mọi struct qua biên IPC:
/// `source_code` phải tới webview đúng tên đó, không `sourceCode`.
///
/// 🔴 `headword`/`gloss` là **ảnh chụp** lúc ghim, không một khoá ngoại vào từ điển: tệp
/// `.db` nguồn có thể được thay ở một bản phát hành sau, và một hàng ghim vẫn phải hiện ra
/// được mà không cần một lượt tra thứ hai (xem doc-comment `PINNED_ENTRY_DDL`).
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct PinnedEntry {
    /// `pinned_entry.id` — khoa thay the trong `global.db`. Khong di ra ngoai may nay.
    pub id: i64,
    /// `dict_source.code` của nguồn sinh ra mục từ.
    pub source_code: String,
    /// `dict_entry.id` **trong tệp `.db` của nguồn đó** — không duy nhất xuyen tep.
    pub entry_id: i64,
    /// Dau muc THAT trong tu dien (`EntryHit.headword`), khong phai truy van nguoi dung.
    pub headword: String,
    /// Nghia rut gon — `NULL` khi luot tra khong lay ve nghia nao.
    pub gloss: Option<String>,
    /// ISO-8601 UTC, lay bang `strftime` cua chinh SQLite.
    pub pinned_at: String,
}

/// Cau lenh doc — dung o ca hai cho, nen thu tu sap xep chi co MOT ban.
///
/// 🔴 `pinned_at DESC, id DESC`: moi nhat truoc. Khoa phu `id` la thu pha hoa khi hai luot
/// ghim roi vao cung mot mili-giay — `strftime('%f')` chi co ba chu so thap phan, va hai
/// luot bam lien tiep trong cung mot mili-giay cho ra hai chuoi BANG NHAU. Khong co khoa
/// phu thi thu tu cua chung do SQLite tu quyet, tuc doi giua hai luot doc.
const SELECT_PINNED: &str = "\
SELECT id, source_code, entry_id, headword, gloss, pinned_at \
FROM pinned_entry ORDER BY pinned_at DESC, id DESC";

/// Mot hang thanh mot [`PinnedEntry`] — **mot ban duy nhat** cho ca ba duong doc.
///
/// ⚠️ Kieu cua tang SQLite di qua ba ten [`Row`]/[`SqlResult`]/[`Transaction`] **tai xuat
/// tu `core::store`**, khong qua ten crate: `tests/store_boundary.rs::
/// only_core_store_may_name_rusqlite` cam moi module ngoai `core::store` nhac toi no
/// (AD-11), va no bat ca `params!`.
fn row_to_entry(row: &Row<'_>) -> SqlResult<PinnedEntry> {
    Ok(PinnedEntry {
        id: row.get(0)?,
        source_code: row.get(1)?,
        entry_id: row.get(2)?,
        headword: row.get(3)?,
        gloss: row.get(4)?,
        pinned_at: row.get(5)?,
    })
}

/// Doc toan bo muc ghim — **ham thuan, day la thu test goi**. Pham vi toan ung dung.
///
/// # Loi
/// - kho vang mat ⇒ `store.open_failed`;
/// - duong doc truot ⇒ `store.read_failed` (qua `From<StoreError>`).
pub fn list_pinned_entries(store: Option<&Store>) -> Result<Vec<PinnedEntry>, IpcError> {
    let store = store.ok_or_else(store_is_missing)?;

    let rows = store.read(|conn| {
        let mut stmt = conn.prepare(SELECT_PINNED)?;
        let found = stmt
            .query_map([], |row| row_to_entry(row))?
            .collect::<SqlResult<Vec<PinnedEntry>>>()?;
        Ok(found)
    })?;

    Ok(rows)
}

/// Ghim mot muc tu — **ham thuan**. Tra ve bo ghim MOI, da sap xep.
///
/// 🔴 `INSERT OR IGNORE`, khong `INSERT`: `UNIQUE (source_code, entry_id)` la hop dong o
/// tang luoc do, va ghim hai lan cung mot muc phai la mot thao tac **VO HAI** chu khong
/// mot loi — cung luat `selectSourceTab`/`toggleDictSource` khi thao tac khong ap dung.
/// Mot loi o day nghia la mot cu bam kep tra ve `store.write_failed`.
///
/// ⚠️ Tra ve **ca bo** thay vi hang vua chen: chi goi can bo da sap xep de ve lai danh
/// sach, va mot luot doc thu hai ngay sau luot ghi la mot vong IPC thua cho cung mot su
/// that. Doc trong **cung giao dich** voi luot ghi, nen khong luot ghi nao khac chen vao
/// giua va cho ra mot bo da lac hau ngay luc nhan duoc.
///
/// # Loi
/// - kho vang mat ⇒ `store.open_failed`;
/// - duong ghi truot ⇒ `store.write_failed` (qua `From<StoreError>`).
pub fn pin_entry(
    store: Option<&Store>,
    source_code: &str,
    entry_id: i64,
    headword: &str,
    gloss: Option<&str>,
) -> Result<Vec<PinnedEntry>, IpcError> {
    let store = store.ok_or_else(store_is_missing)?;

    // ⚠️ `to_owned()` truoc khi vao closure: `Store::write` doi `F: Send + 'static`, nen
    // job khong muon duoc mot `&str` cua chi goi.
    let source_code = source_code.to_owned();
    let headword = headword.to_owned();
    let gloss = gloss.map(str::to_owned);

    let rows = store.write(move |tx: &Transaction<'_>| {
        tx.execute(
            "INSERT OR IGNORE INTO pinned_entry \
             (source_code, entry_id, headword, gloss, pinned_at) \
             VALUES (?1, ?2, ?3, ?4, strftime('%Y-%m-%dT%H:%M:%fZ','now'))",
            (&source_code, entry_id, &headword, &gloss),
        )?;

        let mut stmt = tx.prepare(SELECT_PINNED)?;
        let found = stmt
            .query_map([], |row| row_to_entry(row))?
            .collect::<SqlResult<Vec<PinnedEntry>>>()?;
        Ok(found)
    })?;

    Ok(rows)
}

/// Bo ghim mot muc tu — **ham thuan**. Tra ve bo ghim MOI, da sap xep.
///
/// Bo ghim mot muc chua tung ghim la thao tac **VO HAI**, khong mot loi — cung luat
/// [`pin_entry`].
///
/// # Loi
/// - kho vang mat ⇒ `store.open_failed`;
/// - duong ghi truot ⇒ `store.write_failed` (qua `From<StoreError>`).
pub fn unpin_entry(
    store: Option<&Store>,
    source_code: &str,
    entry_id: i64,
) -> Result<Vec<PinnedEntry>, IpcError> {
    let store = store.ok_or_else(store_is_missing)?;

    let source_code = source_code.to_owned();

    let rows = store.write(move |tx: &Transaction<'_>| {
        tx.execute(
            "DELETE FROM pinned_entry WHERE source_code = ?1 AND entry_id = ?2",
            (&source_code, entry_id),
        )?;

        let mut stmt = tx.prepare(SELECT_PINNED)?;
        let found = stmt
            .query_map([], |row| row_to_entry(row))?
            .collect::<SqlResult<Vec<PinnedEntry>>>()?;
        Ok(found)
    })?;

    Ok(rows)
}

/// Ba vo `#[tauri::command]`. **Khong mot quy tac nao song o day.**
///
/// ⚠️ Ba vo nay gon han ban dau: pham vi doi sang `global.db` nghia la chung lay thang
/// `State<Store>` — khong con mot `Mutex<Option<OpenWork>>` de muon, nen khong con mot
/// `MutexGuard` phai giu song toi khi loi goi xuong xong. Cung khuon `commands::config`.
pub mod wire {
    use super::{IpcError, PinnedEntry};
    use crate::core::store::Store;

    /// Vo IPC cua [`super::list_pinned_entries`].
    ///
    /// ⚠️ `try_state`, khong `state()` — `lib.rs::open_global_store` ghi chan doan roi **di
    /// tiep** khi mo kho that bai, nen `app.manage(store)` co the chua tung chay. Mot
    /// `state::<Store>()` thang tay panic, va `panic = "abort"` giet ca tien trinh.
    #[tauri::command]
    pub fn list_pinned_entries(app: tauri::AppHandle) -> Result<Vec<PinnedEntry>, IpcError> {
        use tauri::Manager as _;

        let managed = app.try_state::<Store>();
        super::list_pinned_entries(managed.as_deref())
    }

    /// Vo IPC cua [`super::pin_entry`].
    #[tauri::command]
    pub fn pin_entry(
        app: tauri::AppHandle,
        source_code: String,
        entry_id: i64,
        headword: String,
        gloss: Option<String>,
    ) -> Result<Vec<PinnedEntry>, IpcError> {
        use tauri::Manager as _;

        let managed = app.try_state::<Store>();
        super::pin_entry(
            managed.as_deref(),
            &source_code,
            entry_id,
            &headword,
            gloss.as_deref(),
        )
    }

    /// Vo IPC cua [`super::unpin_entry`].
    #[tauri::command]
    pub fn unpin_entry(
        app: tauri::AppHandle,
        source_code: String,
        entry_id: i64,
    ) -> Result<Vec<PinnedEntry>, IpcError> {
        use tauri::Manager as _;

        let managed = app.try_state::<Store>();
        super::unpin_entry(managed.as_deref(), &source_code, entry_id)
    }
}
