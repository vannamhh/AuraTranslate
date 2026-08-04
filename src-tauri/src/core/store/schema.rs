//! Phiên bản lược đồ, từ chối mở lùi, di trú **chỉ tiến** — AD-30, AC6, AC7.
//!
//! ─────────────────────────────────────────────────────────────────────────────
//! QUY ƯỚC PHIÊN BẢN, KHAI TƯỜNG MINH
//! ─────────────────────────────────────────────────────────────────────────────
//! Phiên bản nằm ở `PRAGMA user_version`, mặc định là **0** — nên *"database mới tinh"*
//! và *"database ở phiên bản 0"* **không phân biệt được**. Quy ước vì thế phải nói ra
//! thay vì để mỗi người đọc tự suy:
//!
//! - **0 = chưa có lược đồ.** Không có gì để sao lưu, và không có gì mất khi di trú.
//! - Bước di trú đầu tiên đánh số **1**.
//! - `to_version` tăng dần nghiêm ngặt. ⛔ Không có bước lùi, ⛔ không có bước
//!   *"sửa cho vừa"* — một bước như vậy là hai đường lược đồ khác nhau cho cùng một số,
//!   và chúng sẽ rẽ nhau ở máy người dùng chứ không ở đây.
//!
//! ─────────────────────────────────────────────────────────────────────────────
//! 🔴 SAO LƯU BẰNG `fs::copy` TỆP `.db` TRẦN LÀ MỘT BẢN SAO **KHÔNG ĐẦY ĐỦ**
//! ─────────────────────────────────────────────────────────────────────────────
//! Khi WAL đang bật, dữ liệu **đã commit nhưng chưa checkpoint** sống trong `.db-wal`,
//! không trong `.db`. Copy mình tệp `.db` cho ra một bản sao **thiếu đúng những thay đổi
//! gần nhất** — và bản sao đó trông hoàn toàn hợp lệ: mở được, không lỗi, chỉ thiếu.
//! Đây là bản sao lưu mà AC6 dựa vào để **cho phép** di trú, nên nó hỏng ở đúng chỗ đắt
//! nhất: chỗ người dùng tin là mình có đường lui.
//!
//! → [`backup_before_migration`]: `wal_checkpoint(TRUNCATE)` → **xác nhận `busy == 0`** →
//!   rồi mới `fs::copy`.
//!
//! ⚠️ Feature `backup` của `rusqlite` đang **TẮT** (`Cargo.toml:75`) ⇒
//! `Connection::backup` **không tồn tại**. ⛔ Bật nó là thêm bề mặt API mới vào một crate
//! đã ghim — ngoài phạm vi story này, và `check-deps.mjs` sẽ đỏ.

use std::path::Path;

use rusqlite::Connection;

use super::{StoreError, StoreKind, pragmas};

/// Lược đồ của bảng nhật ký di trú — **bước 1 của `global.db`**.
///
/// Vì sao một bảng nhật ký chứ không phải một bảng nghiệp vụ: `global.db` **chưa có**
/// nghiệp vụ nào ở story này (cấu hình là Story 1.8, phím tắt là 1.21). Nhưng AC6 nói
/// *"chạy các bước di trú chỉ tiến trong một giao dịch, sau khi đã sao lưu"*, và **không
/// có bước nào thì AC6 không có gì để nghiệm thu trên đường sản phẩm** — chỉ nghiệm thu
/// được bằng một bộ di trú giả trong test, tức lại đúng hình dạng *"mệnh đề vòng"* mà
/// lượt review Story 1.5 đã bắt (`deferred-work.md:38`).
///
/// - `applied_at` lấy bằng `strftime` **của chính SQLite** — ISO-8601 UTC theo
///   Consistency Conventions, và ⛔ không phải thêm `chrono`/`time` cho một dòng.
/// - `app_version` lấy từ `env!("CARGO_PKG_VERSION")`.
/// - Bản ghi được chèn **trong cùng giao dịch** với bước sinh ra nó. Ghi ngoài giao dịch
///   là mở đúng ca *"sổ nói đã chạy mà lược đồ thì chưa"*.
pub const SCHEMA_MIGRATION_LOG_DDL: &str = "\
CREATE TABLE schema_migration_log (
  version     INTEGER PRIMARY KEY,
  applied_at  TEXT NOT NULL,
  app_version TEXT NOT NULL
);";

/// Một bước di trú. `sql` chạy **trọn trong một giao dịch**; trả `Err` ⇒ rollback.
///
/// ⚠️ `sql` là `&'static str` có chủ ý: một bước di trú là **hằng của bản ứng dụng**, đọc
/// được cạnh mã, và không bao giờ được sinh ra lúc chạy. Một bước di trú dựng bằng
/// `format!` từ trạng thái là một lược đồ khác nhau trên mỗi máy.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Migration {
    /// Phiên bản mà database đạt được **sau khi** bước này commit. Tăng dần nghiêm ngặt.
    pub to_version: u32,
    /// Câu (hoặc nhiều câu) SQL của bước.
    pub sql: &'static str,
}

/// Bộ di trú của `global.db`. Hôm nay đúng **một** bước — xem [`SCHEMA_MIGRATION_LOG_DDL`].
///
/// ⛔ Không thêm bước cho một lược đồ chưa tồn tại. Mỗi story sở hữu bước di trú của
/// chính nó, cùng lúc với bảng mà nó cần.
pub const GLOBAL_MIGRATIONS: &[Migration] = &[Migration {
    to_version: 1,
    sql: SCHEMA_MIGRATION_LOG_DDL,
}];

/// Phiên bản cao nhất mà một bộ di trú đạt tới. Bộ rỗng ⇒ 0.
///
/// 🔴 Chỉ đáng tin **sau** [`validate_strictly_increasing`]: hàm này tin `.last()` là
/// lớn nhất, và một danh sách khai lộn thứ tự làm giả định đó sai mà không gì báo.
pub(crate) fn target_version(migrations: &[Migration]) -> u32 {
    migrations.last().map(|m| m.to_version).unwrap_or(0)
}

/// Xác nhận bộ di trú **tăng dần nghiêm ngặt** — bất biến của chính bộ di trú.
///
/// ⚠️ Phải chạy TRƯỚC khi [`target_version`] được tin, không chỉ trước khi [`migrate`]
/// chạy: [`super::Store::open`] dùng `target` để quyết định từ chối mở (AC7) trước cả
/// bước sao lưu và di trú — một `target` tính sai từ một danh sách lộn thứ tự làm quyết
/// định đó sai ở đúng bước không được phép sai.
pub(crate) fn validate_strictly_increasing(
    migrations: &[Migration],
    kind: StoreKind,
) -> Result<(), StoreError> {
    let mut previous = 0u32;
    for m in migrations {
        if m.to_version <= previous {
            return Err(StoreError::OpenFailed {
                store: kind,
                detail: format!(
                    "migration list is not strictly increasing: {} follows {}",
                    m.to_version, previous
                ),
            });
        }
        previous = m.to_version;
    }
    Ok(())
}

/// Đọc `PRAGMA user_version`.
///
/// 🔴 **Chỉ đọc**, và nó là bước thứ hai của [`super::Store::open`] chứ không phải bước
/// thứ tư — xem doc-comment của `core::store`. Đảo thứ tự là AC7 trượt im lặng.
pub(crate) fn read_user_version(conn: &Connection, kind: StoreKind) -> Result<u32, StoreError> {
    let raw: i64 = conn
        .query_row("PRAGMA user_version", [], |row| row.get(0))
        .map_err(|e| StoreError::OpenFailed {
            store: kind,
            detail: format!("read PRAGMA user_version: {e}"),
        })?;

    // `user_version` là INTEGER 32-bit có dấu trong header SQLite. Một số âm ở đó nghĩa
    // là tệp không do ứng dụng này viết ra; ⛔ đừng ép kiểu im lặng thành một số dương
    // khổng lồ rồi kết luận "lược đồ quá mới".
    u32::try_from(raw).map_err(|_| StoreError::OpenFailed {
        store: kind,
        detail: format!("PRAGMA user_version is {raw}, expected a non-negative integer"),
    })
}

/// Sao lưu **trước bước di trú đầu tiên**, và chỉ khi đã có lược đồ (`from >= 1`).
///
/// Trình tự là hợp đồng, xem doc-comment của module: TRUNCATE → xác nhận `busy == 0` →
/// `fs::copy`. Tệp đích là `<tên>.db.bak-v<n>` **cạnh tệp gốc**, với `n` là phiên bản
/// **trước** khi di trú — cái tên nói được nó là bản sao của cái gì.
pub(crate) fn backup_before_migration(
    conn: &Connection,
    path: &Path,
    kind: StoreKind,
    from: u32,
) -> Result<(), StoreError> {
    let outcome = pragmas::wal_checkpoint(conn, "TRUNCATE", kind)?;

    // 🔴 `busy != 0` nghĩa là TRUNCATE **không chép hết** — tức `.db` vẫn thiếu phần nằm
    // trong WAL, tức bản sao sắp tạo ra là bản sao không đầy đủ. ⛔ Không đi tiếp: một
    // bản sao lưu sai còn tệ hơn không có, vì nó làm người ta dám di trú.
    if outcome.busy != 0 {
        return Err(StoreError::OpenFailed {
            store: kind,
            detail: format!(
                "backup aborted: wal_checkpoint(TRUNCATE) reported busy={} log={} checkpointed={}",
                outcome.busy, outcome.log, outcome.checkpointed
            ),
        });
    }

    let mut name = path.file_name().unwrap_or_default().to_owned();
    name.push(format!(".bak-v{from}"));
    let target = path.with_file_name(name);

    std::fs::copy(path, &target).map_err(|e| StoreError::OpenFailed {
        store: kind,
        detail: format!("copy backup to {}: {e}", target.display()),
    })?;

    Ok(())
}

/// Chạy các bước di trú **chỉ tiến**, mỗi bước trong **một** giao dịch.
///
/// Trả về phiên bản sau khi xong. Một bước ném lỗi ⇒ giao dịch của **chính bước đó**
/// rollback và `user_version` giữ nguyên giá trị trước bước đó; các bước đã commit trước
/// nó thì ở lại — đúng nghĩa "chỉ tiến", và đó là lý do mỗi bước một giao dịch chứ không
/// phải cả loạt một giao dịch.
pub(crate) fn migrate(
    conn: &mut Connection,
    kind: StoreKind,
    from: u32,
    migrations: &[Migration],
) -> Result<u32, StoreError> {
    // ⚠️ Idempotent với lần kiểm ở `Store::open` (xem `validate_strictly_increasing`):
    // `migrate` không có cách nào biết chỗ gọi đã kiểm chưa, và cái giá của kiểm lại một
    // danh sách nhỏ mỗi lần mở là không đáng kể so với cái giá của một lần bỏ sót.
    validate_strictly_increasing(migrations, kind)?;

    let app_version = env!("CARGO_PKG_VERSION");
    let mut current = from;

    for m in migrations.iter().filter(|m| m.to_version > from) {
        let tx = conn.transaction().map_err(|e| StoreError::OpenFailed {
            store: kind,
            detail: format!("begin transaction for migration {}: {e}", m.to_version),
        })?;

        tx.execute_batch(m.sql).map_err(|e| StoreError::OpenFailed {
            store: kind,
            detail: format!("migration {} failed: {e}", m.to_version),
        })?;

        // ⚠️ `strftime` của SQLite, không phải đồng hồ của Rust: ISO-8601 UTC theo
        // Consistency Conventions mà ⛔ không phải kéo `chrono`/`time` về cho một dòng.
        tx.execute(
            "INSERT INTO schema_migration_log (version, applied_at, app_version) \
             VALUES (?1, strftime('%Y-%m-%dT%H:%M:%fZ','now'), ?2)",
            rusqlite::params![m.to_version, app_version],
        )
        .map_err(|e| StoreError::OpenFailed {
            store: kind,
            detail: format!("log migration {}: {e}", m.to_version),
        })?;

        // ⚠️ `PRAGMA` không nhận tham số ràng buộc. Giá trị là `u32` của chính chương
        // trình, ⛔ không bao giờ là dữ liệu người dùng.
        tx.execute_batch(&format!("PRAGMA user_version = {}", m.to_version))
            .map_err(|e| StoreError::OpenFailed {
                store: kind,
                detail: format!("set user_version to {}: {e}", m.to_version),
            })?;

        tx.commit().map_err(|e| StoreError::OpenFailed {
            store: kind,
            detail: format!("commit migration {}: {e}", m.to_version),
        })?;

        current = m.to_version;
    }

    Ok(current)
}
