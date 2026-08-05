//! Ba PRAGMA của AC3 — **đặt RỒI ĐỌC LẠI**, và mở kết nối bằng cờ tường minh.
//!
//! ─────────────────────────────────────────────────────────────────────────────
//! 🔴 VÌ SAO "ĐẶT RỒI ĐỌC LẠI" LÀ HỢP ĐỒNG CHỨ KHÔNG PHẢI CẨN THẬN THỪA
//! ─────────────────────────────────────────────────────────────────────────────
//! `Connection::pragma_update` gọi `execute_batch` (`rusqlite-0.40.1/src/pragma.rs:227-248`),
//! và `execute_batch` **cố ý nuốt** hàng trả về của PRAGMA — `src/lib.rs:555-560` viết
//! `if !stmt.stmt.is_null() && stmt.step()? { if false { return Err(…) } }`, nhánh
//! `if false` là một no-op có chủ ý của thượng nguồn.
//!
//! `PRAGMA journal_mode = WAL` trả về **chế độ mới** dưới dạng một hàng, và hàng đó bị
//! vứt. Nghĩa là trên một thư mục mà WAL không dùng được, lệnh trả `Ok(())`, database ở
//! lại chế độ `delete`, **mọi bảo đảm của NFR2 và NFR18 biến mất**, và không lỗi nào
//! được ném. Không cổng nào đỏ, không test nào đỏ, không dòng log nào — cho tới khi một
//! người dùng thật gõ và thấy khựng.
//!
//! → Nên mọi PRAGMA ở đây đi qua [`set_and_verify`] hoặc [`verify`]: đặt, đọc lại, so.
//!   Đọc về sai ⇒ lỗi, ⛔ không đi tiếp.
//!
//! ⚠️ **Trạng thái CỦA TỪNG KẾT NỐI** (Bẫy 3): `wal_autocheckpoint`, `busy_timeout` và
//! `query_only` không phải thuộc tính của database. Đặt trên writer rồi tưởng cả kho đã
//! yên là sai hình dạng — writer, **mỗi** kết nối pool, và luồng checkpoint mỗi cái phải
//! tự đặt. Riêng `journal_mode` **là** thuộc tính của database và chỉ writer đặt nó; các
//! kết nối khác chỉ **xác nhận**.

use std::path::Path;

use rusqlite::{Connection, OpenFlags};

use super::{StoreError, StoreKind, Tuning};

/// Mở một kết nối bằng **cờ tường minh**.
///
/// ⚠️ ⛔ Không dùng `OpenFlags::default()`. Nó là
/// `READ_WRITE | CREATE | NO_MUTEX | URI` (`rusqlite-0.40.1/src/lib.rs:1256-1266`) — và
/// `SQLITE_OPEN_URI` là cái bẫy: một thư mục người dùng chứa `?` trong tên
/// (`.../Sach ? tap 2/global.db`) bị SQLite đọc thành URI kèm query string, và kho mở ra
/// ở một chỗ khác chỗ ta nghĩ. Đường dẫn ở đây LUÔN là đường dẫn hệ tệp, không bao giờ
/// là URI.
///
/// `NO_MUTEX` giữ nguyên có chủ ý — chế độ multi-thread là tiền đề của Quyết định #4:
/// `Connection` là `Send` nhưng **không `Sync`**, nên trình biên dịch cưỡng chế việc một
/// kết nối không bị dùng đồng thời từ hai luồng. Đó là hàng rào, không phải phiền nhiễu.
pub(crate) fn open_connection(path: &Path, kind: StoreKind) -> Result<Connection, StoreError> {
    let flags = OpenFlags::SQLITE_OPEN_READ_WRITE
        | OpenFlags::SQLITE_OPEN_CREATE
        | OpenFlags::SQLITE_OPEN_NO_MUTEX;

    Connection::open_with_flags(path, flags).map_err(|e| StoreError::OpenFailed {
        store: kind,
        detail: format!("open {}: {e}", path.display()),
    })
}

/// Mở một tệp **CHỈ ĐỌC** bằng cờ tường minh — đường của [`StoreKind::Dict`] (AC7).
///
/// `SQLITE_OPEN_READ_ONLY | SQLITE_OPEN_NO_MUTEX`. Ba thứ **vắng mặt** ở đây đều là
/// quyết định, không phải sơ suất:
///
/// - ⛔ **Không `SQLITE_OPEN_URI`** — cùng nguyên lý lẽ với [`open_connection`]: một thư
///   mục người dùng chứa `?` trong tên (`.../Sach ? tap 2/dict-core.db`) bị SQLite đọc
///   thành URI kèm query string, và tệp mở ra ở một chỗ khác chỗ ta nghĩ. Đường dẫn ở
///   đây LUÔN là đường dẫn hệ tệp.
/// - 🔴 ⛔ **Không `SQLITE_OPEN_CREATE`** — và đây là lý do MỚI, riêng của đường chỉ đọc.
///   Với `CREATE`, một đường dẫn gõ sai (hoặc một tệp `$RESOURCE` chưa được đóng gói)
///   không trả lỗi: SQLite **dựng một tệp rỗng mới toanh**, mọi truy vấn sau đó trả
///   rỗng, ⛔ không lỗi nào được ném, và người dùng chỉ thấy *"tra từ không ra kết quả"*.
///   Không `CREATE` thì đường dẫn sai là một `Err` ngay tại chỗ mở.
/// - ⛔ **Không `SQLITE_OPEN_READ_WRITE`** — AD-7: dữ liệu từ điển chỉ đọc, luôn luôn.
///
/// `NO_MUTEX` giữ nguyên vì cùng lý do với [`open_connection`]: `Connection` là `Send`
/// nhưng ⛔ không `Sync`, nên trình biên dịch tự canh việc một kết nối không bị dùng
/// đồng thời từ hai luồng.
pub(crate) fn open_readonly_connection(
    path: &Path,
    kind: StoreKind,
) -> Result<Connection, StoreError> {
    let flags = OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_NO_MUTEX;

    Connection::open_with_flags(path, flags).map_err(|e| StoreError::OpenFailed {
        store: kind,
        detail: format!("open readonly {}: {e}", path.display()),
    })
}

/// Đọc một PRAGMA trả **một hàng một cột** thành chuỗi.
///
/// ⚠️ Đọc thành `String` cho mọi PRAGMA, kể cả những cái trả số: `PRAGMA busy_timeout`
/// trả INTEGER, `PRAGMA journal_mode` trả TEXT, và một hàm đọc duy nhất tránh được việc
/// mỗi chỗ gọi tự chọn kiểu rồi lệch nhau. So sánh làm trên chuỗi đã chuẩn hoá.
fn read_pragma(conn: &Connection, name: &str, kind: StoreKind) -> Result<String, StoreError> {
    conn.query_row(&format!("PRAGMA {name}"), [], |row| {
        // `get::<_, String>` gãy trên cột INTEGER; đọc qua `ValueRef` rồi tự đổi.
        let value = row.get_ref(0)?;
        Ok(match value {
            rusqlite::types::ValueRef::Integer(i) => i.to_string(),
            rusqlite::types::ValueRef::Real(f) => f.to_string(),
            rusqlite::types::ValueRef::Text(t) => String::from_utf8_lossy(t).into_owned(),
            rusqlite::types::ValueRef::Blob(_) | rusqlite::types::ValueRef::Null => {
                String::new()
            }
        })
    })
    .map_err(|e| StoreError::OpenFailed {
        store: kind,
        detail: format!("read PRAGMA {name}: {e}"),
    })
}

/// Đặt một PRAGMA rồi **đọc lại và so**.
///
/// ⚠️ Giá trị nội suy bằng `format!` chứ không bằng tham số ràng buộc, và đó là bắt buộc
/// chứ không phải lười: SQLite **không** nhận tham số ràng buộc trong câu `PRAGMA`. Mọi
/// giá trị đi qua đây là hằng của chính chương trình hoặc số từ [`Tuning`] — ⛔ không bao
/// giờ là dữ liệu người dùng.
fn set_and_verify(
    conn: &Connection,
    name: &str,
    value: &str,
    expected: &str,
    kind: StoreKind,
) -> Result<(), StoreError> {
    conn.execute_batch(&format!("PRAGMA {name} = {value}"))
        .map_err(|e| StoreError::OpenFailed {
            store: kind,
            detail: format!("set PRAGMA {name} = {value}: {e}"),
        })?;

    let got = read_pragma(conn, name, kind)?;
    if got.eq_ignore_ascii_case(expected) {
        return Ok(());
    }

    Err(StoreError::OpenFailed {
        store: kind,
        detail: format!("PRAGMA {name} read back as {got:?}, expected {expected:?}"),
    })
}

/// `journal_mode = WAL`, đặt rồi đọc lại. Đọc về khác `"wal"` ⇒
/// [`StoreError::WalUnavailable`], ⛔ không đi tiếp.
///
/// 🔴 Biến thể lỗi RIÊNG chứ không dùng chung `OpenFailed`, vì đây là ca duy nhất trong
/// cả module mà *"lệnh chạy thành công"* và *"kết quả đúng"* là hai chuyện khác nhau —
/// và là ca duy nhất người dùng có thể tự sửa được (đổi chỗ để kho, ra khỏi ổ mạng).
pub(crate) fn set_and_verify_wal(conn: &Connection, kind: StoreKind) -> Result<(), StoreError> {
    // ⚠️ `execute_batch` cũng được ở đây vì ta đọc lại ngay dưới — nhưng đọc lại bằng
    // MỘT câu `PRAGMA journal_mode` riêng, KHÔNG tin hàng trả về của câu đặt.
    conn.execute_batch("PRAGMA journal_mode = WAL")
        .map_err(|e| StoreError::OpenFailed {
            store: kind,
            detail: format!("set PRAGMA journal_mode = WAL: {e}"),
        })?;

    let mode = read_pragma(conn, "journal_mode", kind)?;
    if mode.eq_ignore_ascii_case("wal") {
        return Ok(());
    }

    Err(StoreError::WalUnavailable { store: kind, mode })
}

/// Xác nhận (không đặt) rằng database đang ở WAL.
///
/// Dùng cho pool đọc và luồng checkpoint: `journal_mode` là thuộc tính của **database**,
/// writer đã đặt nó; các kết nối khác đặt lại là thừa, mà không xác nhận thì một kết nối
/// mở nhầm vào một tệp khác sẽ không lộ ra.
fn verify_wal(conn: &Connection, kind: StoreKind) -> Result<(), StoreError> {
    let mode = read_pragma(conn, "journal_mode", kind)?;
    if mode.eq_ignore_ascii_case("wal") {
        return Ok(());
    }
    Err(StoreError::WalUnavailable { store: kind, mode })
}

/// `wal_autocheckpoint = 0` + `busy_timeout` — bộ chung của **mọi** kết nối (Bẫy 3).
fn apply_connection_pragmas(
    conn: &Connection,
    kind: StoreKind,
    tuning: &Tuning,
) -> Result<(), StoreError> {
    // 0 = TẮT autocheckpoint của SQLite. AD-12: thời điểm checkpoint là quyết định của
    // ứng dụng. Để nguyên mặc định 1000 trang là để SQLite chèn một lượt checkpoint vào
    // giữa một lượt gõ, tức đúng cái gai trễ mà NFR2 cấm.
    set_and_verify(conn, "wal_autocheckpoint", "0", "0", kind)?;

    let ms = tuning.busy_timeout.as_millis().to_string();
    set_and_verify(conn, "busy_timeout", &ms, &ms, kind)?;

    Ok(())
}

/// Kết nối GHI: `journal_mode = WAL` + `wal_autocheckpoint = 0` + `busy_timeout`.
/// Cả ba đặt rồi đọc lại — đây là AC3 nguyên văn.
pub(crate) fn apply_writer_pragmas(
    conn: &Connection,
    kind: StoreKind,
    tuning: &Tuning,
) -> Result<(), StoreError> {
    set_and_verify_wal(conn, kind)?;
    apply_connection_pragmas(conn, kind, tuning)
}

/// Kết nối ĐỌC: bộ chung **cộng `query_only = 1`**, và xác nhận database ở WAL.
///
/// 🔴 `query_only` thay vì `SQLITE_OPEN_READ_ONLY` (Quyết định #2). Cả hai đều là cưỡng
/// chế của SQLite chứ không phải kỷ luật, nhưng `READ_ONLY` mang một ràng buộc phụ trên
/// database WAL: kết nối chỉ-đọc cần tệp `-shm` **đã tồn tại** và cần quyền phù hợp trên
/// **thư mục**, nên nó gãy ở những ca biên mà `query_only` không có. `query_only` là
/// trạng thái kết nối, đọc lại xác nhận được, và SQLite trả lỗi cho mọi lệnh ghi.
///
/// ⚠️ `query_only` đặt **CUỐI CÙNG**, sau hai PRAGMA kia — thứ tự này có lý do: hai
/// PRAGMA kia là trạng thái kết nối chứ không phải thay đổi database nên vẫn đặt được
/// dưới `query_only`, nhưng dựa vào điều đó là dựa vào một chi tiết của SQLite mà không
/// gì trong repo này cưỡng chế. Đặt cuối thì không phải dựa vào gì cả.
pub(crate) fn apply_reader_pragmas(
    conn: &Connection,
    kind: StoreKind,
    tuning: &Tuning,
) -> Result<(), StoreError> {
    verify_wal(conn, kind)?;
    apply_connection_pragmas(conn, kind, tuning)?;
    set_and_verify(conn, "query_only", "1", "1", kind)?;
    Ok(())
}

/// Kết nối ĐỌC trên một tệp **từ điển**: `busy_timeout` + `query_only = 1`. Hết. (AC7)
///
/// ─────────────────────────────────────────────────────────────────────────────
/// 🔴 VÌ SAO KHÔNG TÁI DÙNG [`apply_reader_pragmas`] — HAI ĐƯỜNG HỎNG NỐI TIẾP
/// ─────────────────────────────────────────────────────────────────────────────
/// [`apply_reader_pragmas`] gọi `verify_wal`. Cả ba tệp từ điển ở `journal_mode = delete`
/// — `tools/dict-build/src/finalize.rs::set_journal_mode_delete` đặt thế **có chủ ý**, và
/// tệp đi kèm một checksum trong `dict-manifest.toml` (AD-25). Nên:
///
/// 1. Tái dùng thẳng ⇒ [`StoreError::WalUnavailable`] `{ mode: "delete" }` ngay lượt mở
///    đầu tiên. Hỏng **ồn ào** — đây là đường ít tệ hơn.
/// 2. Cám dỗ tiếp theo, *"thì đặt WAL cho nó"* ⇒ `PRAGMA journal_mode = WAL` **GHI VÀO**
///    database. SHA-256 của tệp đổi, `dict-manifest.toml` thành sai, AD-25 vỡ, và ⛔
///    không cổng nào bắt (`check-dict-manifest.mjs` cố ý ⛔ không đọc `.db`). Trên một
///    `$RESOURCE` chỉ-đọc thật thì lệnh chỉ **trượt** — tức hành vi khác nhau giữa máy
///    dev và bản phát hành.
///
/// → Nên đường của tệp từ điển ⛔ **không** chạm `journal_mode` theo bất kỳ chiều nào:
///   ⛔ không đặt, ⛔ không xác nhận.
///
/// ⛔ **Không `wal_autocheckpoint`** — nó chỉ có nghĩa trên một database WAL, và ở đây
/// ⛔ không có WAL nào. Đặt nó là khai một ý định sai cho người đọc sau.
///
/// ⚠️ `query_only` đặt **CUỐI CÙNG**, cùng thứ tự và cùng lý do với
/// [`apply_reader_pragmas`]. Nó chồng lên `SQLITE_OPEN_READ_ONLY` chứ ⛔ không thay thế:
/// cờ mở là cưỡng chế của tầng tệp, `query_only` là cưỡng chế của tầng câu lệnh và nó
/// **đọc lại xác nhận được** — mà một bất biến đọc lại được là một bất biến test được.
pub(crate) fn apply_dict_reader_pragmas(
    conn: &Connection,
    kind: StoreKind,
    tuning: &Tuning,
) -> Result<(), StoreError> {
    let ms = tuning.busy_timeout.as_millis().to_string();
    set_and_verify(conn, "busy_timeout", &ms, &ms, kind)?;
    set_and_verify(conn, "query_only", "1", "1", kind)?;
    Ok(())
}

/// Kết nối của luồng CHECKPOINT: bộ chung, ⛔ **không** `query_only` — checkpoint ghi.
pub(crate) fn apply_checkpoint_pragmas(
    conn: &Connection,
    kind: StoreKind,
    tuning: &Tuning,
) -> Result<(), StoreError> {
    verify_wal(conn, kind)?;
    apply_connection_pragmas(conn, kind, tuning)
}

/// Kết quả một lượt `PRAGMA wal_checkpoint(<mode>)` — **ba cột, và cả ba đều được xét**.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct CheckpointOutcome {
    /// `busy` — khác 0 nghĩa là lượt này **bị chặn** và có thể chưa chép được frame nào.
    pub(crate) busy: i64,
    /// `log` — số frame đang có trong WAL.
    pub(crate) log: i64,
    /// `checkpointed` — số frame đã chép về database.
    pub(crate) checkpointed: i64,
}

/// Chạy `PRAGMA wal_checkpoint(<mode>)` và **đọc cả ba cột**.
///
/// ─────────────────────────────────────────────────────────────────────────────
/// 🔴 CẢ HAI CÁCH GỌI "TỰ NHIÊN" ĐỀU SAI — và một trong hai sai IM LẶNG
/// ─────────────────────────────────────────────────────────────────────────────
/// - `conn.execute("PRAGMA wal_checkpoint(PASSIVE)", [])` ⇒
///   **`Error::ExecuteReturnedResults`** (`rusqlite-0.40.1/src/statement.rs:682`).
///   Sai ồn ào, dễ phát hiện.
/// - `conn.execute_batch("PRAGMA wal_checkpoint(PASSIVE)")` ⇒ **`Ok(())`** và ba cột bị
///   vứt. Sai im lặng, và đây là đường hỏng thật: một lượt PASSIVE bị một reader chặn
///   trả `busy = 1` và **không chép được frame nào**; mã đọc nó thành *"đã checkpoint
///   xong"*, `.db-wal` cứ phình, và ngưỡng của AC5 không bao giờ có cơ hội đúng.
///
/// → `query_row`, ba cột, và `busy != 0` là một **trạng thái phải ghi lại**, không phải
///   một thành công.
///
/// ⚠️ Feature `hooks` của `rusqlite` đang **TẮT** (`Cargo.toml:114`, không nằm trong
/// `default` hay `bundled`) ⇒ `Wal::checkpoint_v2` và `CheckpointMode` **không tồn tại**.
/// Checkpoint **phải** đi qua SQL. ⛔ Bật feature đó là thêm bề mặt API mới vào một crate
/// đã ghim, ngoài phạm vi story này.
pub(crate) fn wal_checkpoint(
    conn: &Connection,
    mode: &str,
    kind: StoreKind,
) -> Result<CheckpointOutcome, StoreError> {
    conn.query_row(&format!("PRAGMA wal_checkpoint({mode})"), [], |row| {
        Ok(CheckpointOutcome {
            busy: row.get(0)?,
            log: row.get(1)?,
            checkpointed: row.get(2)?,
        })
    })
    .map_err(|e| StoreError::OpenFailed {
        store: kind,
        detail: format!("wal_checkpoint({mode}): {e}"),
    })
}
