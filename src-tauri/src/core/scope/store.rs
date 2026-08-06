//! Nạp và ghi tầng Global qua [`core::store`](crate::core::store) — AC5.
//!
//! ─────────────────────────────────────────────────────────────────────────────
//! 🔴 ⛔ TỆP NÀY KHÔNG BAO GIỜ GÕ TÊN CRATE SQLITE — **kể cả trong comment đuôi dòng**
//! ─────────────────────────────────────────────────────────────────────────────
//! `tests/store_boundary.rs:54` cấm hai chuỗi ngoài `core/store/**`, và bộ quét chỉ miễn
//! trừ **dòng bắt đầu bằng `//`**. Story 1.7 đã ghi lại nguyên văn: *"Comment đuôi dòng
//! vẫn bị bắt."* Mọi kiểu cần dùng đã được [`core::store`](crate::core::store) **tái
//! xuất** (`Transaction` · `SqlResult` · `Row` · `ReadHandle`, `store/mod.rs:98-117`), nên
//! không có lý do chính đáng nào để gõ tên crate ở đây.
//!
//! ─────────────────────────────────────────────────────────────────────────────
//! ⛔ `core::scope` KHÔNG SỞ HỮU KHO CỦA MỌI LOẠI
//! ─────────────────────────────────────────────────────────────────────────────
//! Bảng `config_value` phục vụ **riêng** ba loại [`Semantics::GlobalOnly`]. Glossary, TM,
//! Prompt, Cấu hình AI và Luật làm sạch sẽ mang **bảng riêng của chúng** ở epic của
//! chúng, và tự nạp hai tầng rồi đưa qua [`super::ScopeResolver`] — xem §Quyết định #1 và
//! doc-comment của [`crate::core::store::CONFIG_VALUE_DDL`].
//!
//! ⛔ **Không gọi [`Store::write`] bên trong một job ghi.** `writer.rs:104` trả
//! `WriteFailed` chứ ⛔ không xếp hàng, và đó là chốt chống deadlock chứ không phải một
//! lỗi cần lách.
//!
//! ⚠️ Mọi chuỗi trong tệp này viết KHÔNG DẤU — xem doc-comment của [`super::kinds`].

use std::collections::BTreeMap;

use crate::core::store::{ReadHandle, Store, StoreError, Transaction};

use super::kinds::{ScopeKind, Semantics};
use super::resolve::Resolved;
use super::{ScopeResolver, Tier};

/// Theme lúc chưa có hàng nào trên đĩa. Nền giấy, đúng hướng *"Bàn viết"* (Story 1.4).
///
/// ⚠️ Mặc định tồn tại ở **hai** tầng, và đó là chủ ý chứ không phải trùng lặp:
/// - **ở đây** — kho mở được nhưng chưa có hàng nào *(lần chạy đầu tiên)*;
/// - **ở `src/main.ts`** — không có Tauri, hoặc kho không mở được, tức không có giá trị
///   nào để rơi về cả.
///
/// Bỏ mặc định ở tầng này và trả chuỗi rỗng thì `cfg?.theme ?? 'light'` phía frontend
/// **không cứu được**: `??` chỉ bắt `null`/`undefined`, còn `''` là một giá trị.
pub const DEFAULT_THEME: &str = "light";

/// Chế độ lúc chưa có hàng nào trên đĩa. PRD §5.2 gọi Library là **điểm vào ứng dụng**.
pub const DEFAULT_MODE: &str = "library";

/// Khoá của [`ScopeKind::AppConfig`] mang theme đang chọn.
const KEY_THEME: &str = "theme";

/// Khoá của [`ScopeKind::AppConfig`] mang chế độ cuối cùng người dùng ở.
const KEY_MODE: &str = "mode";

/// Khoá của [`ScopeKind::AppConfig`] mang **bố cục panel đang hiển thị** — Story 1.14, AC4.
///
/// ⚠️ Ở [`ScopeKind::AppConfig`] chứ ⛔ **không** ở `ScopeKind::LayoutPreset`, và ranh giới
/// đó do `kinds.rs` phân xử: `LayoutPreset` mang **preset đã ĐẶT TÊN** — dữ liệu người dùng
/// tự tạo và tự gọi tên, thứ màn hình của Story 1.21 liệt kê ra. Bố cục *đang hiển thị* thì
/// cùng loại với `theme` và `mode`: nó là *"lần cuối ứng dụng ở trạng thái nào"*, ⛔ không
/// phải một mục trong một danh sách.
///
/// ⛔ Nhét nó vào `layout_presets` dưới một khoá dành riêng (`__current`) là bẻ nghĩa của
/// *"đã đặt tên"*, và Story 1.21 sẽ hiện `__current` ra màn hình như một preset thật.
///
/// ⚠️ Giá trị là một chuỗi JSON do frontend `stringify` — tầng này ⛔ **không** phân tích nó
/// và ⛔ không kiểm hình dạng. `SerializedDockview` là hình dạng của dockview, tức của
/// frontend (AD-1: *"bố cục panel là state UI của frontend"*); kiểm nó ở đây là dựng một bản
/// chép thứ hai của một lược đồ thư viện, và bản chép đó sẽ trôi ở lần nâng đầu tiên. Chốt
/// chống JSON hỏng nằm ở `WorkspaceDock.vue::restore` — `try` → rơi về preset mặc định.
const KEY_LAYOUT: &str = "workspace_layout";

/// Ba loại `GlobalOnly` đã phân giải, sẵn sàng cho tầng adapter.
///
/// ⚠️ Giữ nguyên [`Resolved`] thay vì làm phẳng ngay, vì đó là **bằng chứng** rằng đường
/// đọc này đi qua [`ScopeResolver`] thật chứ không phải một truy vấn tắt chạy song song
/// với nó — tức đúng vế *"đúng một `ScopeResolver`"* của AC1, quan sát được ở kiểu trả về.
/// Hôm nay mọi mục mang `tier: Global` và `shadowed: None`; ⛔ đừng đọc điều đó thành
/// *"trường tier là thừa"* — Story 1.15 không phải đổi chữ ký nào.
#[derive(Debug, Clone)]
pub struct GlobalConfig {
    app: BTreeMap<String, Resolved<String>>,
    shortcuts: BTreeMap<String, Resolved<String>>,
    layout_presets: BTreeMap<String, Resolved<String>>,
}

impl GlobalConfig {
    /// Theme đang chọn, hoặc [`DEFAULT_THEME`] nếu chưa ai chọn gì.
    pub fn theme(&self) -> &str {
        self.app
            .get(KEY_THEME)
            .map_or(DEFAULT_THEME, |r| r.value().as_str())
    }

    /// Chế độ cuối cùng, hoặc [`DEFAULT_MODE`].
    ///
    /// ⚠️ ⛔ Không kiểm tính hợp lệ ở đây: một `global.db` sửa tay có thể mang `"lbrary"`,
    /// và chốt tương ứng đã nằm ở `src/modes/modeState.ts` — nơi nó rơi về mặc định và
    /// **kêu to**. Kiểm hai lần ở hai tầng với hai danh sách chép tay là cách hai danh
    /// sách đó trôi khỏi nhau.
    pub fn mode(&self) -> &str {
        self.app
            .get(KEY_MODE)
            .map_or(DEFAULT_MODE, |r| r.value().as_str())
    }

    /// Bố cục panel đang hiển thị, ở dạng chuỗi JSON của frontend — Story 1.14, AC4.
    ///
    /// ⚠️ **Chuỗi rỗng** khi chưa ai lưu gì, ⛔ không `Option`: nó đi qua IPC tới một
    /// TypeScript `string`, và ở đó `''` với `undefined` phải dẫn về **cùng một** nhánh
    /// *"dựng preset mặc định"*. Hai đại diện cho một trạng thái là hai nhánh phải nhớ
    /// giữ đồng bộ — cùng lý lẽ với [`DEFAULT_THEME`] ở đầu tệp này.
    ///
    /// ⛔ Tầng này ⛔ không kiểm chuỗi có phải JSON hợp lệ không. Xem [`KEY_LAYOUT`].
    pub fn workspace_layout(&self) -> &str {
        self.app.get(KEY_LAYOUT).map_or("", |r| r.value().as_str())
    }

    /// Hợp âm phím tắt theo id thao tác. Rỗng nghĩa là *"dùng hợp âm mặc định"*.
    pub fn shortcuts(&self) -> BTreeMap<String, String> {
        flatten(&self.shortcuts)
    }

    /// Preset bố cục đã đặt tên. Nội dung của chúng là **Story 1.14**.
    pub fn layout_presets(&self) -> BTreeMap<String, String> {
        flatten(&self.layout_presets)
    }

    /// Tầng sinh ra mọi mục — hôm nay **luôn** [`Tier::Global`].
    ///
    /// Không phải test hook: nó là cách duy nhất một chỗ gọi khẳng định được rằng thứ nó
    /// vừa nhận đến từ đâu, và AC5 nghiệm thu trên đúng mệnh đề đó.
    pub fn tiers(&self) -> Vec<Tier> {
        self.app
            .values()
            .chain(self.shortcuts.values())
            .chain(self.layout_presets.values())
            .map(Resolved::tier)
            .collect()
    }
}

fn flatten(map: &BTreeMap<String, Resolved<String>>) -> BTreeMap<String, String> {
    map.iter()
        .map(|(k, v)| (k.clone(), v.value().clone()))
        .collect()
}

/// Nạp **cả ba** loại `GlobalOnly` rồi phân giải chúng qua [`ScopeResolver`] — AC5.
///
/// Vòng chạy end-to-end mà AC5 đòi: `global.db` *(bước di trú 2)* → [`Store::read`] →
/// [`ScopeResolver::resolve_global_only`] → chỗ gọi ở `commands/`.
///
/// ⚠️ Tầng Tác phẩm là `None` ở cả ba lời gọi, và hôm nay đó là trạng thái **duy nhất**
/// tồn tại — `.atproj` là Story 1.15. Xem doc-comment của [`super`].
///
/// # Lỗi
/// [`StoreError::ReadFailed`] / [`StoreError::PoolClosed`] từ đường đọc. ⛔ Không lỗi nào
/// của [`super::ScopeError`] thoát ra được: ba lời gọi dưới đây dùng đúng hàm cho ngữ
/// nghĩa của chúng, và `the_three_global_only_kinds_are_exactly_three` canh mệnh đề đó.
pub fn load_global_config(store: &Store) -> Result<GlobalConfig, StoreError> {
    let resolver = ScopeResolver::global_only();

    let resolve_one = |kind: ScopeKind| -> Result<BTreeMap<String, Resolved<String>>, StoreError> {
        let global = load_kind(store, kind)?;
        // 🔴 `unwrap_or_default()` chứ không `?`: `resolve_global_only` chỉ trả `Err` khi
        // mã gọi sai hàm cho loại của nó, và ba loại dưới đây khai `GlobalOnly` ngay trong
        // `scope_kinds!`. Một `Err` ở đây là lỗi lập trình, không phải một lỗi để báo cho
        // người dùng — và `ScopeError` KHÔNG BAO GIỜ vượt ranh giới IPC (§Quyết định #7).
        // Rơi về map rỗng giữ cho ứng dụng lên được bằng mặc định thay vì dựng một từ vựng
        // lỗi thứ hai ở tầng adapter.
        let resolved = resolver.resolve_global_only(kind, &global, None);
        debug_assert!(
            resolved.is_ok(),
            "scope[{}] must declare GlobalOnly semantics -- see scope_kinds!",
            kind.as_str()
        );
        Ok(resolved.unwrap_or_default())
    };

    Ok(GlobalConfig {
        app: resolve_one(ScopeKind::AppConfig)?,
        shortcuts: resolve_one(ScopeKind::Shortcut)?,
        layout_presets: resolve_one(ScopeKind::LayoutPreset)?,
    })
}

/// Đọc mọi cặp `khoá -> giá trị` của **một** loại từ tầng Global.
///
/// `pub(crate)` chứ không `pub`: chỗ gọi ngoài `core::scope` không được đặt tên
/// [`ScopeKind`] *(`tests/scope_boundary.rs` cưỡng chế)*, nên đường vào của chúng là
/// [`load_global_config`].
///
/// ⚠️ `ORDER BY key` không phải để cho đẹp: nó làm hai lượt chạy trên cùng dữ liệu cho
/// cùng một thứ tự, tức test so sánh mới ổn định. `BTreeMap` cũng tự sắp, nhưng để SQLite
/// làm nghĩa là thứ tự đúng ngay cả khi kiểu đích đổi.
pub(crate) fn load_kind(
    store: &Store,
    kind: ScopeKind,
) -> Result<BTreeMap<String, String>, StoreError> {
    let wire = kind.as_str();

    store.read(move |conn: ReadHandle<'_>| {
        let mut stmt =
            conn.prepare("SELECT key, value FROM config_value WHERE kind = ?1 ORDER BY key")?;
        let mut rows = stmt.query([wire])?;

        let mut out = BTreeMap::new();
        while let Some(row) = rows.next()? {
            out.insert(row.get::<_, String>(0)?, row.get::<_, String>(1)?);
        }
        Ok(out)
    })
}

/// Ghi (hoặc cập nhật) **một** giá trị ở tầng Global.
///
/// `kind` là chuỗi vì nó đến từ bên kia ranh giới IPC — dữ liệu không tin được. Một khoá
/// lạ ⇒ [`StoreError::WriteFailed`], ⛔ không đoán và ⛔ không ghi gì cả.
///
/// ─────────────────────────────────────────────────────────────────────────────
/// ⚠️ VÌ SAO MỘT `kind` LẠ TRẢ LỖI **KHO** CHỨ KHÔNG PHẢI MỘT LOẠI LỖI MỚI
/// ─────────────────────────────────────────────────────────────────────────────
/// §Quyết định #7 khoá hai điều: story này ⛔ không thêm khoá `MessageKey` nào, và
/// [`super::ScopeError`] ⛔ không bao giờ vượt ranh giới IPC. Nên câu duy nhất còn lại để
/// nói với người dùng là câu thật: **thay đổi vừa rồi chưa được lưu** — đó chính là
/// `store.write_failed`. Nó không phải một cách nói tránh: không byte nào được ghi, và
/// `detail` mang lý do đầy đủ cho người đang chẩn đoán *(⛔ và `detail` không bao giờ đi
/// lên giao diện — `From<StoreError> for IpcError` đã chặn)*.
///
/// ⚠️ Một loại **không** phải [`Semantics::GlobalOnly`] cũng bị từ chối ở đây: bảng
/// `config_value` phục vụ riêng ba loại đó, và một hàng `kind = 'glossary'` trong đó là
/// mầm của đúng lược đồ EAV mà §Quyết định #1 loại bỏ.
pub fn save_value(store: &Store, kind: &str, key: &str, value: &str) -> Result<(), StoreError> {
    let Some(parsed) = ScopeKind::from_wire(kind) else {
        return Err(StoreError::WriteFailed {
            store: store.kind(),
            detail: format!("unknown scope kind {kind:?}; nothing was written"),
        });
    };

    if !matches!(parsed.semantics(), Semantics::GlobalOnly) {
        return Err(StoreError::WriteFailed {
            store: store.kind(),
            detail: format!(
                "scope kind {:?} is {:?}, and config_value serves GlobalOnly kinds only; \
                 nothing was written",
                parsed.as_str(),
                parsed.semantics()
            ),
        });
    }

    // Sở hữu tường minh: job ghi chạy trên luồng writer nên nó phải `Send + 'static`.
    let kind = parsed.as_str().to_owned();
    let key = key.to_owned();
    let value = value.to_owned();

    store.write(move |tx: &Transaction<'_>| {
        // `strftime` của CHÍNH SQLite — ISO-8601 UTC theo Consistency Conventions, và ⛔
        // không phải thêm một phụ thuộc ngày giờ cho một cột (NFR15 đòi rà GPLv3 trước).
        tx.execute(
            "INSERT INTO config_value (kind, key, value, updated_at)
             VALUES (?1, ?2, ?3, strftime('%Y-%m-%dT%H:%M:%fZ', 'now'))
             ON CONFLICT (kind, key) DO UPDATE SET
               value      = excluded.value,
               updated_at = excluded.updated_at",
            (&kind, &key, &value),
        )?;
        Ok(())
    })?;

    Ok(())
}
