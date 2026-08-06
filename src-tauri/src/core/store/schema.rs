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

/// Lược đồ bảng cấu hình khoá-giá trị — **bước 2 của `global.db`**, Story 1.8 AC5.
///
/// ─────────────────────────────────────────────────────────────────────────────
/// 🔴 MỘT BẢNG, KHÔNG PHẢI BA — và ⛔ không phải một bảng cho MỌI loại
/// ─────────────────────────────────────────────────────────────────────────────
/// Hai cám dỗ đối nghịch, cả hai đều sai:
///
/// - **Ba bảng** (`keybinding` + `layout_preset` + `app_config`) là dựng lược đồ cho hai
///   tính năng chưa tồn tại (Story 1.14, Story 1.21). Quy tắc đã khoá ngay trên đây:
///   *mỗi story sở hữu bước di trú của chính nó, cùng lúc với bảng mà nó cần*.
/// - **Một bảng cho tất cả** — tức cả Glossary, TM, Prompt và luật làm sạch cùng nhét vào
///   cột `value TEXT` — là dựng một lược đồ EAV mà bốn epic sau phải bóc ra: Glossary có
///   phân loại/xuất xứ/vòng đời ba trạng thái (Story 3.1), TM có cặp văn bản + xuất xứ
///   (AD-6), luật làm sạch có mẫu regex + cờ bật tắt (Story 6.5).
///
/// **Chốt:** bảng này phục vụ **riêng** ba loại `Semantics::GlobalOnly` của
/// `core::scope::ScopeKind` — `shortcut`, `layout_preset`, `app_config`. Mỗi module miền
/// mang bảng riêng của nó, ở epic của nó.
///
/// ⚠️ Cột `kind` là chuỗi chứ không phải một `CHECK` liệt kê ba giá trị: một `CHECK` biến
/// mọi loại `GlobalOnly` mới thành một bước di trú, trong khi phép cưỡng chế thật đã nằm
/// ở `ScopeKind` phía Rust — nơi trình biên dịch làm việc đó (AC4).
///
/// ⚠️ ⛔ Không cột `tier`. Bảng này **là** tầng Global; một cột tầng ở đây là mời người
/// sau ghi một hàng `tier = 'work'` vào `global.db`, tức đúng thứ
/// `ScopeError::WorkTierForbidden` tồn tại để từ chối.
pub const CONFIG_VALUE_DDL: &str = "\
CREATE TABLE config_value (
  kind       TEXT NOT NULL,
  key        TEXT NOT NULL,
  value      TEXT NOT NULL,
  updated_at TEXT NOT NULL,
  PRIMARY KEY (kind, key)
);";

/// Bộ di trú của `global.db`. Hôm nay **hai** bước.
///
/// ⛔ Không thêm bước cho một lược đồ chưa tồn tại. Mỗi story sở hữu bước di trú của
/// chính nó, cùng lúc với bảng mà nó cần.
///
/// ⚠️ Thêm một bước ở đây làm `tests/store_contract.rs` đỏ ở **đúng một** ca
/// (`a_fresh_database_migrates_up_to_target_and_logs_it`, ca duy nhất chạy trên bộ di trú
/// THẬT), và đó là hành vi đúng: số phiên bản đổi phải là một quyết định có người ký, chứ
/// không phải một hiệu ứng phụ. ⛔ Đừng "sửa cho nhất quán" các con số trong `TWO_STEP` /
/// `BROKEN_STEP_TWO` — chúng là fixture cục bộ và không phụ thuộc hằng này.
pub const GLOBAL_MIGRATIONS: &[Migration] = &[
    Migration {
        to_version: 1,
        sql: SCHEMA_MIGRATION_LOG_DDL,
    },
    Migration {
        to_version: 2,
        sql: CONFIG_VALUE_DDL,
    },
];

/// Lược đồ bảng `work` — **bước 1 của `project.db`**, Story 1.15, AC4.
///
/// ─────────────────────────────────────────────────────────────────────────────
/// 🔴 ĐÚNG MỘT HÀNG, và `CHECK (id = 1)` là cơ chế bắt buộc số đó
/// ─────────────────────────────────────────────────────────────────────────────
/// `project.db` mang **một** Tác phẩm — hình dạng `.atproj/` của AD-9 khoá điều đó ở tầng
/// thư mục. Bảng này phản ánh đúng bất biến ở tầng lược đồ thay vì để nó thành một quy ước
/// không ai canh: một `INSERT` thứ hai vi phạm `CHECK` và **SQLite** từ chối, không phải
/// một `debug_assert!` mà bản release im lặng bỏ qua.
///
/// `work_id` là UUID v4 (AD-28) — sinh **một lần** lúc tạo, ⛔ không đổi được, và là khoá
/// dựng lại `meta.json` (xem [`super::super::readonly`] không áp — đây là `project.db`).
/// `source_lang` là trường **bất biến** (AD-18): AC1 nói *"ngôn ngữ nguồn được đặt lúc tạo
/// và ⛔ không đổi được về sau"* — bất biến này được cưỡng chế ở tầng ứng dụng
/// (`core/segment/import.rs`, ⛔ không có `UPDATE` nào chạm cột này), ⛔ không phải một
/// `CHECK`/trigger SQL, vì SQLite không có cú pháp "cột chỉ ghi một lần".
pub const WORK_DDL: &str = "\
CREATE TABLE work (
  id          INTEGER PRIMARY KEY,
  work_id     TEXT NOT NULL,
  name        TEXT NOT NULL,
  source_lang TEXT NOT NULL,
  genre       TEXT NOT NULL,
  created_at  TEXT NOT NULL,
  updated_at  TEXT NOT NULL,
  CHECK (id = 1)
);";

/// Lược đồ bảng `chapter` — **bước 1 của `project.db`**, Story 1.15, AC4.
///
/// ─────────────────────────────────────────────────────────────────────────────
/// 🔴 `AUTOINCREMENT`, ⛔ KHÔNG `INTEGER PRIMARY KEY` TRẦN
/// ─────────────────────────────────────────────────────────────────────────────
/// `INTEGER PRIMARY KEY` trần là bí danh của `rowid`, và SQLite **tái dùng** rowid đã xoá
/// khi nó là rowid lớn nhất từng cấp — cụ thể, xoá hàng cuối rồi chèn hàng mới sẽ nhận
/// lại đúng `id` vừa mất. AD-3 nói id đã về hưu ⛔ **không bao giờ** được tái dùng.
/// `AUTOINCREMENT` giữ một sổ riêng (`sqlite_sequence`) và không bao giờ phát lại một giá
/// trị đã dùng, đổi lại chi phí ghi nhỏ mà không ai đo được ở quy mô một cuốn sách.
///
/// `ord` là **cột riêng** cho thứ tự hiển thị (AD-3, AD-32) — sắp lại được (Epic 2 gộp/tách
/// Chương) mà ⛔ không đụng `id`. ⛔ **Không** `UNIQUE` trên `ord` ở story này: Epic 2 tự
/// quyết cơ chế sắp lại (có thể để hở tạm thời trong một giao dịch nhiều bước).
///
/// `status` mang trạng thái vòng đời ban đầu *Chưa bắt đầu* (FR5) — chuỗi tự do ở tầng
/// SQL, cưỡng chế giá trị hợp lệ là việc của tầng Rust gọi nó (cùng khuôn với
/// `config_value.kind` ở `CONFIG_VALUE_DDL`, xem doc-comment ở trên).
///
/// ⛔ **Không** bảng `segment` — Quyết định #4 của story: AD-4 đóng băng ranh giới segment
/// tính một lần lúc nhập; một bộ tách "tạm" ở đây là đóng băng vĩnh viễn ranh giới sai.
/// `source_text` mang **nguyên khối** văn bản nguồn của Chương; Story 2.1 sở hữu bước tách
/// tường minh biến nó thành các hàng `segment`.
pub const CHAPTER_DDL: &str = "\
CREATE TABLE chapter (
  id          INTEGER PRIMARY KEY AUTOINCREMENT,
  ord         INTEGER NOT NULL,
  title       TEXT,
  source_text TEXT NOT NULL,
  status      TEXT NOT NULL,
  created_at  TEXT NOT NULL,
  updated_at  TEXT NOT NULL
);";

/// Bộ di trú của `project.db`. Hôm nay **ba** bước — Story 1.15.
///
/// ⚠️ **Ba bước, ⛔ không phải một** — và đó là hệ quả của một ràng buộc kỹ thuật, ghi ra
/// thay vì giấu: `Migration::sql` là `&'static str`, và `concat!` (thứ duy nhất nối được
/// hai chuỗi ở **compile time** mà không thêm phụ thuộc) chỉ nhận **literal**, ⛔ không
/// nhận một `const` đặt tên. Nối [`SCHEMA_MIGRATION_LOG_DDL`] (hằng **tái dùng** từ
/// `global.db`) với [`WORK_DDL`]/[`CHAPTER_DDL`] thành một chuỗi duy nhất buộc phải chép
/// lại nguyên văn của hằng kia — đúng thứ *"tái dùng, ⛔ đừng viết lại"* cấm. Ba bước tách
/// rời, mỗi bước một hằng, giữ **mỗi** DDL có **đúng một** nguồn sự thật, cùng khuôn
/// [`GLOBAL_MIGRATIONS`] đã tách `SCHEMA_MIGRATION_LOG_DDL` (bước 1) khỏi
/// `CONFIG_VALUE_DDL` (bước 2). "Mỗi bước một giao dịch" là bất biến sẵn có của
/// [`migrate`] — ⛔ không AC nào của story này đòi `work`/`chapter` phải cùng một giao dịch
/// SQL với nhật ký di trú.
///
/// ⛔ Không thêm bước cho một lược đồ chưa tồn tại — cùng luật với [`GLOBAL_MIGRATIONS`].
/// ⛔ **Không** bảng `segment`/Glossary/TM/prompt/asset ở đây; mỗi epic mang bảng riêng của
/// nó cùng lúc với bước di trú cần nó.
pub const PROJECT_MIGRATIONS: &[Migration] = &[
    Migration {
        to_version: 1,
        sql: SCHEMA_MIGRATION_LOG_DDL,
    },
    Migration {
        to_version: 2,
        sql: WORK_DDL,
    },
    Migration {
        to_version: 3,
        sql: CHAPTER_DDL,
    },
];

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
