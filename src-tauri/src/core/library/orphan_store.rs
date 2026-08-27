//! Bảng `library_orphan` — **global.db**, phán quyết Ice #1 (2026-08-27) trên Story 5.3.
//!
//! ─────────────────────────────────────────────────────────────────────────────
//! CỜ MỒ CÔI LÀ DỮ LIỆU NGƯỜI DÙNG, KHÔNG PHẢI TRẠNG THÁI CỦA CHỈ MỤC DẪN XUẤT
//! ─────────────────────────────────────────────────────────────────────────────
//! Vòng dựng đầu của Story 5.3 giữ cờ mồ côi làm một cột (`orphaned`) ngay trong
//! `library_work` (`library-index.db`) — hẹp hơn, không kho mới. Ice lật quyết định đó
//! 2026-08-27: một hàng mồ côi chỉ biến mất khi người dùng **chủ động** gọi `forget_orphan`,
//! không đường tự động nào xoá nó — đó là một quyết định người dùng đã ghi lại ("tôi biết
//! đường dẫn cũ, tôi CHƯA gỡ nó"), không phải một mẩu cache có thể mất vô hại khi
//! `library-index.db` bị xoá tay hay lệch phiên bản. Xem doc-comment của
//! `core::store::schema::LIBRARY_ORPHAN_DDL` cho lý lẽ đầy đủ và §Design Notes/§Spec Change
//! Log của `5-3-quet-lai-thu-muc.md` cho lịch sử quyết định.
//!
//! ─────────────────────────────────────────────────────────────────────────────
//! 🔴 MODULE NÀY SỞ HỮU MỌI CÂU SQL CỦA `library_orphan` — `Indexer` GỌI XUỐNG ĐÂY
//! ─────────────────────────────────────────────────────────────────────────────
//! Mọi hàm dưới đây nhận một `&Store` đã MỞ SẴN (đó là `global.db`, được quản lý ở nơi khác
//! — `lib.rs::open_global_store`) — module này không tự mở kho, đúng khuôn mọi module ngoài
//! `core/store` (`commands::pinned`, `commands::config`, …). `Indexer::rebuild`/
//! `forget_orphan`/`list_orphans` gọi xuống các hàm này thay vì tự viết SQL, nên có đúng MỘT
//! chỗ biết hình dạng của bảng `library_orphan`.
//!
//! ⚠️ Không dùng `rusqlite::` trực tiếp — mọi kiểu tầng SQLite đi qua các tên tái xuất từ
//! `core::store` (`Row`/`SqlResult`/`Transaction`/`ReadHandle`), đúng luật
//! `tests/store_boundary.rs::only_core_store_may_name_rusqlite` áp cho MỌI module ngoài
//! `core/store` — module này không phải một miễn trừ.
//!
//! ─────────────────────────────────────────────────────────────────────────────
//! 🔴 KHÔNG CÓ GIAO DỊCH XUYÊN HAI KHO — THỨ TỰ GỌI Ở `Indexer` MỚI LÀ THỨ CHỊU TRÁCH NHIỆM
//! ─────────────────────────────────────────────────────────────────────────────
//! `global.db` và `library-index.db` là HAI kho, mỗi kho một `store::Writer` riêng — không
//! có SQLite `ATTACH`/giao dịch chung nào bọc cả hai. Các hàm ở đây chỉ làm ĐÚNG MỘT việc mỗi
//! lần gọi (ghi/xoá/đọc bảng này) và không biết gì về `library_work` — chỗ quyết định GỌI
//! HÀM NÀO TRƯỚC, HÀM NÀO SAU (ghi `global.db` TRƯỚC khi xoá khỏi `library-index.db`, để một
//! bước hai trượt không làm mất lời nhắc vĩnh viễn) là `Indexer::rebuild`/`forget_orphan`,
//! không phải module này. Xem doc-comment của các hàm đó cho lý do đầy đủ.

use crate::core::store::{Row, SqlResult, Store, StoreError, Transaction};

/// Một hàng mồ côi — đúng BA cột của `library_orphan`, đủ để màn hình nêu "nó trỏ tới đâu"
/// (AC3) mà không cần đọc `library-index.db`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OrphanRecord {
    pub work_id: String,
    /// Đường dẫn CŨ, tuyệt đối trên máy này — giữ NGUYÊN VĂN, không cắt/chuẩn hoá.
    pub atproj_path: String,
    /// Ảnh chụp tên Tác phẩm lúc thành mồ côi.
    pub name: String,
}

const SELECT_ORPHANS: &str =
    "SELECT work_id, atproj_path, name FROM library_orphan ORDER BY work_id";

fn row_to_record(row: &Row<'_>) -> SqlResult<OrphanRecord> {
    Ok(OrphanRecord {
        work_id: row.get(0)?,
        atproj_path: row.get(1)?,
        name: row.get(2)?,
    })
}

/// Đọc TOÀN BỘ hàng mồ côi hiện có, sắp theo `work_id` (tất định — cùng khuôn
/// `Indexer::list_rows`; sắp theo tên/ngày là việc của Story 5.6).
pub fn list(store: &Store) -> Result<Vec<OrphanRecord>, StoreError> {
    store.read(|conn| {
        let mut stmt = conn.prepare(SELECT_ORPHANS)?;
        stmt.query_map([], |row| row_to_record(row))?
            .collect::<SqlResult<Vec<_>>>()
    })
}

/// Ghi (UPSERT) NHIỀU hàng mồ côi trong MỘT giao dịch — dùng khi một lượt `rebuild` phát
/// hiện nhiều Tác phẩm cùng biến mất một lượt (đổi thư mục gốc, hay gốc biến mất).
///
/// `records` rỗng ⇒ no-op, không mở giao dịch nào (tránh một round-trip `store::Writer` vô
/// nghĩa trên đường nóng "không có gì mồ côi ở lượt này", là đa số các lượt `rebuild`).
///
/// `ON CONFLICT DO UPDATE`: một `work_id` đã mồ côi từ trước (ví dụ do gốc biến mất rồi biến
/// mất theo một đường khác) ghi ĐÈ đường dẫn/tên mới nhất, không sinh hàng thứ hai — cùng
/// nguyên tắc UPSERT mà `library_work` dùng cho các hàng đang sống.
///
/// 🔴 **Chỗ gọi PHẢI đợi hàm này trả `Ok` trước khi xoá hàng tương ứng khỏi
/// `library-index.db`** — xem doc-comment module. Hàm này không tự làm việc đó; nó chỉ ghi.
pub fn upsert_many(store: &Store, records: Vec<OrphanRecord>) -> Result<(), StoreError> {
    if records.is_empty() {
        return Ok(());
    }
    store.write(move |tx: &Transaction<'_>| {
        for record in &records {
            tx.execute(
                "INSERT INTO library_orphan (work_id, atproj_path, name) VALUES (?1, ?2, ?3) \
                 ON CONFLICT (work_id) DO UPDATE SET \
                   atproj_path = excluded.atproj_path, \
                   name        = excluded.name",
                (&record.work_id, &record.atproj_path, &record.name),
            )?;
        }
        Ok(())
    })
}

/// Xoá NHIỀU `work_id` khỏi bảng mồ côi trong MỘT giao dịch — dùng khi các Tác phẩm mồ côi
/// QUAY LẠI (đường dẫn xuất hiện lại trong lượt quét này). Idempotent: một `work_id` không có
/// mặt không phải lỗi, không panic.
///
/// `work_ids` rỗng ⇒ no-op, không mở giao dịch nào.
pub fn remove_many(store: &Store, work_ids: Vec<String>) -> Result<(), StoreError> {
    if work_ids.is_empty() {
        return Ok(());
    }
    store.write(move |tx: &Transaction<'_>| {
        for work_id in &work_ids {
            tx.execute("DELETE FROM library_orphan WHERE work_id = ?1", [work_id])?;
        }
        Ok(())
    })
}

/// Xoá ĐÚNG MỘT `work_id` — trả SỐ HÀNG bị xoá (0 hoặc 1, `work_id` là khoá chính), để chỗ
/// gọi (`Indexer::forget_orphan`) tự quyết định "not orphaned" khi không hàng nào bị xoá,
/// đúng khuôn `DELETE ... WHERE work_id = ?1 AND orphaned = 1` mà bản trước dùng trên
/// `library_work` — cùng nguyên tắc, chỉ đổi BẢNG/KHO đang giữ hàng đó.
pub fn forget(store: &Store, work_id: &str) -> Result<usize, StoreError> {
    let owned = work_id.to_owned();
    store.write(move |tx: &Transaction<'_>| {
        tx.execute("DELETE FROM library_orphan WHERE work_id = ?1", [&owned])
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::store::StoreSpec;
    use std::path::PathBuf;
    use std::sync::atomic::{AtomicU64, Ordering};

    static NEXT_DIR: AtomicU64 = AtomicU64::new(0);

    fn temp_dir() -> PathBuf {
        let n = NEXT_DIR.fetch_add(1, Ordering::Relaxed);
        let dir = std::env::temp_dir().join(format!(
            "auratranslate-orphan-store-{}-{}",
            std::process::id(),
            n
        ));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap_or_else(|e| panic!("tao {}: {e}", dir.display()));
        dir
    }

    fn open_global(dir: &std::path::Path) -> Store {
        Store::open(StoreSpec::global(dir.join("global.db"))).expect("mo global.db")
    }

    /// Ca hợp đồng cho chính thuộc tính mà phán quyết Ice #1 đòi: một lượt `upsert_many` rỗng
    /// không mở giao dịch nào (đối chứng gián tiếp qua việc nó không panic/không lỗi trên một
    /// kho vừa mở, chưa có bảng nào bị chạm).
    #[test]
    fn upserting_an_empty_list_is_a_no_op() {
        let dir = temp_dir();
        let store = open_global(&dir);
        upsert_many(&store, Vec::new()).expect("danh sach rong phai la no-op");
        assert!(list(&store).expect("list").is_empty());
        drop(store);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn upsert_then_list_then_forget_round_trips() {
        let dir = temp_dir();
        let store = open_global(&dir);

        upsert_many(
            &store,
            vec![OrphanRecord {
                work_id: "id-1".to_owned(),
                atproj_path: "/tmp/Foo.atproj".to_owned(),
                name: "Foo".to_owned(),
            }],
        )
        .expect("upsert");

        let rows = list(&store).expect("list");
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].work_id, "id-1");
        assert_eq!(rows[0].atproj_path, "/tmp/Foo.atproj");
        assert_eq!(rows[0].name, "Foo");

        // UPSERT lần hai trên CÙNG work_id phải GHI ĐÈ, không sinh hàng thứ hai.
        upsert_many(
            &store,
            vec![OrphanRecord {
                work_id: "id-1".to_owned(),
                atproj_path: "/tmp/Foo-Moved.atproj".to_owned(),
                name: "Foo".to_owned(),
            }],
        )
        .expect("upsert lan hai");
        let rows = list(&store).expect("list sau upsert lan hai");
        assert_eq!(rows.len(), 1, "UPSERT khong duoc sinh hang thu hai");
        assert_eq!(rows[0].atproj_path, "/tmp/Foo-Moved.atproj");

        let deleted = forget(&store, "id-1").expect("forget");
        assert_eq!(deleted, 1);
        assert!(list(&store).expect("list sau forget").is_empty());

        // forget lần hai trên work_id đã gỡ -- idempotent, 0 hàng, không lỗi.
        let deleted_again = forget(&store, "id-1").expect("forget lan hai");
        assert_eq!(deleted_again, 0);

        drop(store);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn remove_many_is_idempotent_for_unknown_work_ids() {
        let dir = temp_dir();
        let store = open_global(&dir);
        remove_many(&store, vec!["khong-ton-tai".to_owned()]).expect("remove_many rong du lieu");
        remove_many(&store, Vec::new()).expect("danh sach rong phai la no-op");
        drop(store);
        let _ = std::fs::remove_dir_all(&dir);
    }
}
