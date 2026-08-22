//! Nạp và ghi tầng Global qua [`core::store`](crate::core::store) — AC5.
//!
//! ─────────────────────────────────────────────────────────────────────────────
//! 🔴 không TỆP NÀY KHÔNG BAO GIỜ GÕ TÊN CRATE SQLITE — **kể cả trong comment đuôi dòng**
//! ─────────────────────────────────────────────────────────────────────────────
//! `tests/store_boundary.rs:54` cấm hai chuỗi ngoài `core/store/**`, và bộ quét chỉ miễn
//! trừ **dòng bắt đầu bằng `//`**. Story 1.7 đã ghi lại nguyên văn: *"Comment đuôi dòng
//! vẫn bị bắt."* Mọi kiểu cần dùng đã được [`core::store`](crate::core::store) **tái
//! xuất** (`Transaction` · `SqlResult` · `Row` · `ReadHandle`, `store/mod.rs:98-117`), nên
//! không có lý do chính đáng nào để gõ tên crate ở đây.
//!
//! ─────────────────────────────────────────────────────────────────────────────
//! `core::scope` KHÔNG SỞ HỮU KHO CỦA MỌI LOẠI
//! ─────────────────────────────────────────────────────────────────────────────
//! Bảng `config_value` phục vụ **riêng** ba loại [`Semantics::GlobalOnly`]. Glossary, TM,
//! Prompt, Cấu hình AI và Luật làm sạch sẽ mang **bảng riêng của chúng** ở epic của
//! chúng, và tự nạp hai tầng rồi đưa qua [`super::ScopeResolver`] — xem §Quyết định #1 và
//! doc-comment của [`crate::core::store::CONFIG_VALUE_DDL`].
//!
//! **Không gọi [`Store::write`] bên trong một job ghi.** `writer.rs:104` trả
//! `WriteFailed` chứ không xếp hàng, và đó là chốt chống deadlock chứ không phải một
//! lỗi cần lách.
//!
//! ⚠️ Mọi chuỗi trong tệp này viết KHÔNG DẤU — xem doc-comment của [`super::kinds`].

use std::collections::{BTreeMap, BTreeSet};

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
/// ⚠️ Ở [`ScopeKind::AppConfig`] chứ **không** ở `ScopeKind::LayoutPreset`, và ranh giới
/// đó do `kinds.rs` phân xử: `LayoutPreset` mang **preset đã ĐẶT TÊN** — dữ liệu người dùng
/// tự tạo và tự gọi tên, thứ màn hình của Story 1.21 liệt kê ra. Bố cục *đang hiển thị* thì
/// cùng loại với `theme` và `mode`: nó là *"lần cuối ứng dụng ở trạng thái nào"*, không
/// phải một mục trong một danh sách.
///
/// Nhét nó vào `layout_presets` dưới một khoá dành riêng (`__current`) là bẻ nghĩa của
/// *"đã đặt tên"*, và Story 1.21 sẽ hiện `__current` ra màn hình như một preset thật.
///
/// ⚠️ Giá trị là một chuỗi JSON do frontend `stringify` — tầng này **không** phân tích nó
/// và không kiểm hình dạng. `SerializedDockview` là hình dạng của dockview, tức của
/// frontend (AD-1: *"bố cục panel là state UI của frontend"*); kiểm nó ở đây là dựng một bản
/// chép thứ hai của một lược đồ thư viện, và bản chép đó sẽ trôi ở lần nâng đầu tiên. Chốt
/// chống JSON hỏng nằm ở `WorkspaceDock.vue::restore` — `try` → rơi về preset mặc định.
const KEY_LAYOUT: &str = "workspace_layout";

/// Khoá của [`ScopeKind::AppConfig`] mang **các nguồn từ điển đang BỊ TẮT** — Story 1.19,
/// AC5 · §Quyết định #1a.
///
/// ⚠️ Ở [`ScopeKind::AppConfig`], **tầng Global**, chứ không một [`ScopeKind`] thứ mười và
/// không một tầng Tác phẩm. FR103 liệt kê tầng Tác phẩm gồm *"Glossary riêng, prompt riêng,
/// TM riêng, ngôn ngữ nguồn"* — **không** có nguồn từ điển; và `mockups/settings.html:246`
/// đã phân xử đúng lớp câu hỏi này cho phím tắt: *"một thao tác không nên đổi phím theo từng
/// Tác phẩm"*. Một người dịch không tin VietPhrase thì không tin nó ở **mọi** Tác phẩm.
/// Khoá thứ tư của cùng một cửa mà `theme`/`mode`/`workspace_layout` đã đi qua.
///
/// 🔴 **GIÁ TRỊ LÀ TẬP `code` BỊ TẮT, KHÔNG PHẢI TẬP ĐƯỢC BẬT** — và đó là một bất biến, không
/// một quy ước mã hoá. Mặc định là **mọi nguồn đều bật**, nên một nguồn **mới** *(một tệp
/// `.db` thêm ở bản sau)* phải tự động bật. Lưu tập **được bật** làm nguồn mới im lặng **tắt**
/// ngay khi nó xuất hiện: một lớp dữ liệu có mặt trong bản cài mà không ai thấy, đúng lớp lỗi
/// *"rỗng im lặng"* mà AD-44 ④ cấm.
const KEY_DICT_DISABLED: &str = "dict_sources_disabled";

/// Tách chuỗi trên đĩa thành tập `code` bị tắt — **hàm thuần, đây là thứ test gọi**.
///
/// Mã hoá: các `code` ngăn nhau bằng `,`, khoảng trắng thừa bị cắt, phần rỗng bị bỏ — cùng
/// quy ước mà `src/main.ts::toBindings` dùng cho hợp âm phím tắt, nên không có quy ước thứ
/// hai để nhớ.
///
/// 🔴 **Một `code` đã lưu mà tệp của nó KHÔNG CÒN đi qua đây nguyên vẹn**, và đó là hành vi
/// đúng: hàm này không biết gì về tập lớp đang gắn, và nó **không được** biết (AD-44 ① vá A2
/// cấm một sổ *"tệp nào chứa gì"*). Phép lọc theo `code` ở [`crate::core::dict::lookup_grouped`]
/// đơn giản không khớp gì cả ⇒ nguồn không còn tồn tại bị **bỏ qua im lặng**, không lỗi và
/// không một chip mồ côi trên màn hình (AC5). Giữ nó trong tập cũng là giữ lời hứa ngược lại:
/// cắm lại tệp đó ⇒ nó vẫn ở trạng thái tắt mà người dùng đã chọn.
pub fn parse_disabled_sources(raw: &str) -> BTreeSet<String> {
    raw.split(',')
        .map(str::trim)
        .filter(|code| !code.is_empty())
        .map(str::to_owned)
        .collect()
}

/// Khoá của [`ScopeKind::AppConfig`] mang **ngưỡng quét ứng viên khi nhập** — Story 3.5,
/// lớp phủ thứ tư (`GlossarySettingsOverlay.vue`).
///
/// ⚠️ Ở [`ScopeKind::AppConfig`], **tầng Global**, chứ không một `ScopeKind` thứ mười và
/// không một tầng Tác phẩm — cùng lý do đã ghi cho [`KEY_DICT_DISABLED`]: ngưỡng quét là
/// một cấu hình ứng dụng, không phải dữ liệu của một Tác phẩm cụ thể (§Never của story:
/// *"Không vẽ thanh chuyển phạm vi… ngưỡng là `AppConfig` ⇒ `GlobalOnly`"*).
const KEY_GLOSSARY_SCAN_THRESHOLD: &str = "glossary_scan_threshold";

/// Ngưỡng mặc định khi chưa ai cấu hình gì, hoặc khi giá trị trên đĩa hỏng — FR47, đo và
/// chốt ở §Boundaries/§Design Notes của story (Chương mẫu thật cho ra một quần thể ứng
/// viên hợp lý ở ngưỡng này; xem §Verification của story cho số đo).
pub const DEFAULT_GLOSSARY_SCAN_THRESHOLD: u32 = 5;

/// Phân giải ngưỡng quét từ giá trị THÔ trên đĩa — **hàm thuần, đây là thứ test gọi**.
///
/// ─────────────────────────────────────────────────────────────────────────────
/// 🔴 GETTER LÀ CHỖ DUY NHẤT BIẾT MỘT GIÁ TRỊ HỎNG — `config_value.value` LÀ TEXT KHÔNG `CHECK`
/// ─────────────────────────────────────────────────────────────────────────────
/// `CONFIG_VALUE_DDL` không ràng buộc hình dạng của `value` (nó phục vụ MỌI khoá của MỌI
/// loại `GlobalOnly` — một `CHECK` riêng cho một khoá là phá vỡ đúng lược đồ dùng-chung mà
/// bảng này tồn tại để giữ). Một giá trị hỏng trên đĩa (`"abc"`/`"0"`/`"-3"`, hay bất cứ gì
/// một bản ứng dụng cũ/một lượt sửa tay `.db` để lại) chỉ có MỘT chỗ để bị bắt: đây.
///
/// I/O Matrix của story: *"`config_value` chứa `"abc"`/`"0"`/`"-3"` ⇒ Rơi về mặc định 5;
/// Ghi chẩn đoán không dấu; KHÔNG ném."* `parse::<u32>` tự chặn `"abc"` (lỗi phân tích) và
/// `"-3"` (dấu trừ không hợp lệ cho `u32`, KHÔNG "phân tích thành số âm rồi ép kiểu" — Rust
/// không có bước ép kiểu ngầm đó). `"0"` phân tích ĐƯỢC thành `0u32` nhưng bị chặn tường
/// minh bằng `== 0` — một ngưỡng 0 chấp nhận MỌI tần suất (kể cả 0 lần lặp), tức tắt hẳn cơ
/// chế lọc mà toàn bộ story này dựng ra để có.
///
/// ⚠️ **Chẩn đoán ra `eprintln!`, không hoảng loạn** — cùng khuôn mọi lớp "hỏng ⇒ rơi về
/// mặc định, nói ra" khác của kho (`decode_category`/`decode_term_origin` TRẢ LỖI vì chúng
/// đọc dữ liệu **do chính ứng dụng ghi** qua một `CHECK`; ở đây `value` KHÔNG có `CHECK` bảo
/// vệ, nên rơi về mặc định — không ném — là lựa chọn ĐÚNG cho một khoá cấu hình mà người
/// dùng vẫn phải dùng được ứng dụng dù giá trị của nó hỏng).
pub fn parse_glossary_scan_threshold(raw: Option<&str>) -> u32 {
    let Some(raw) = raw else {
        return DEFAULT_GLOSSARY_SCAN_THRESHOLD;
    };

    match raw.parse::<u32>() {
        Ok(0) | Err(_) => {
            eprintln!(
                "scope[app_config] glossary_scan_threshold tren dia khong hop le: {raw:?} -- \
                 roi ve mac dinh {DEFAULT_GLOSSARY_SCAN_THRESHOLD}"
            );
            DEFAULT_GLOSSARY_SCAN_THRESHOLD
        }
        Ok(value) => value,
    }
}

/// Ba loại `GlobalOnly` đã phân giải, sẵn sàng cho tầng adapter.
///
/// ⚠️ Giữ nguyên [`Resolved`] thay vì làm phẳng ngay, vì đó là **bằng chứng** rằng đường
/// đọc này đi qua [`ScopeResolver`] thật chứ không phải một truy vấn tắt chạy song song
/// với nó — tức đúng vế *"đúng một `ScopeResolver`"* của AC1, quan sát được ở kiểu trả về.
/// Hôm nay mọi mục mang `tier: Global` và `shadowed: None`; đừng đọc điều đó thành
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
    /// ⚠️ Không kiểm tính hợp lệ ở đây: một `global.db` sửa tay có thể mang `"lbrary"`,
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
    /// ⚠️ **Chuỗi rỗng** khi chưa ai lưu gì, không `Option`: nó đi qua IPC tới một
    /// TypeScript `string`, và ở đó `''` với `undefined` phải dẫn về **cùng một** nhánh
    /// *"dựng preset mặc định"*. Hai đại diện cho một trạng thái là hai nhánh phải nhớ
    /// giữ đồng bộ — cùng lý lẽ với [`DEFAULT_THEME`] ở đầu tệp này.
    ///
    /// Tầng này không kiểm chuỗi có phải JSON hợp lệ không. Xem [`KEY_LAYOUT`].
    pub fn workspace_layout(&self) -> &str {
        self.app.get(KEY_LAYOUT).map_or("", |r| r.value().as_str())
    }

    /// Các nguồn từ điển **đang bị TẮT**, ở dạng chuỗi trên đĩa — Story 1.19, AC5.
    ///
    /// ⚠️ **Chuỗi rỗng** khi chưa ai tắt gì, không `Option` — cùng luật
    /// [`Self::workspace_layout`]: nó đi qua IPC tới một TypeScript `string`, và ở đó `''`
    /// với `undefined` phải dẫn về **cùng một** nhánh *"mọi nguồn đều bật"*.
    pub fn dict_sources_disabled(&self) -> &str {
        self.app
            .get(KEY_DICT_DISABLED)
            .map_or("", |r| r.value().as_str())
    }

    /// Cùng giá trị, đã tách thành tập — thứ mà đường tra cứu nhận (§Quyết định #2a).
    pub fn disabled_source_codes(&self) -> BTreeSet<String> {
        parse_disabled_sources(self.dict_sources_disabled())
    }

    /// Ngưỡng quét ứng viên khi nhập, ĐÃ QUA [`parse_glossary_scan_threshold`] — Story 3.5.
    /// Không bao giờ `0`, không bao giờ hỏng: chỗ gọi nhận thẳng một số dùng được ngay.
    pub fn glossary_scan_threshold(&self) -> u32 {
        parse_glossary_scan_threshold(
            self.app.get(KEY_GLOSSARY_SCAN_THRESHOLD).map(|r| r.value().as_str()),
        )
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
/// [`StoreError::ReadFailed`] / [`StoreError::PoolClosed`] từ đường đọc. Không lỗi nào
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
        let resolved = resolver.resolve_global_only(kind.as_str(), &global, None);
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
/// lạ ⇒ [`StoreError::WriteFailed`], không đoán và không ghi gì cả.
///
/// ─────────────────────────────────────────────────────────────────────────────
/// ⚠️ VÌ SAO MỘT `kind` LẠ TRẢ LỖI **KHO** CHỨ KHÔNG PHẢI MỘT LOẠI LỖI MỚI
/// ─────────────────────────────────────────────────────────────────────────────
/// §Quyết định #7 khoá hai điều: story này không thêm khoá `MessageKey` nào, và
/// [`super::ScopeError`] không bao giờ vượt ranh giới IPC. Nên câu duy nhất còn lại để
/// nói với người dùng là câu thật: **thay đổi vừa rồi chưa được lưu** — đó chính là
/// `store.write_failed`. Nó không phải một cách nói tránh: không byte nào được ghi, và
/// `detail` mang lý do đầy đủ cho người đang chẩn đoán *(không và `detail` không bao giờ đi
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
        // `strftime` của CHÍNH SQLite — ISO-8601 UTC theo Consistency Conventions, và không
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

/// Xoa **mot** hang cau hinh o tang Global. Story 1.21 · AC8.
///
/// ─────────────────────────────────────────────────────────────────────────────
/// 🔴 VI SAO XOA LA MOT THAO TAC RIENG, KHONG PHAI MOT LUOT GHI CHUOI RONG
/// ─────────────────────────────────────────────────────────────────────────────
/// Duong doc phan biet ba trang thai bang **su co mat cua khoa**, khong bang gia tri cua
/// no: khoa vang mat nghia la *"chua ai dat gi"* ⇒ dung mac dinh cua san pham; khoa co mat
/// voi gia tri rong nghia la *"thao tac nay co y khong co phim"*. Hai cau tra loi khac
/// nhau, va man hinh phim tat phai toi duoc ca hai.
///
/// ⚠️ Ghi chinh hop am mac dinh xuong dia **khong** thay duoc ham nay: hang do thanh mot
/// gia tri dong bang, nen mot story sau doi hop am mac dinh thi nguoi da bam *"tra ve mac
/// dinh"* mot lan mac ket o gia tri cu mai mai, khong dau hieu nao. Ice chot 2026-08-11.
///
/// Cung luat voi [`save_value`], va do khong phai mot su trung lap: mot `kind` la ⇒
/// [`StoreError::WriteFailed`]; mot loai khong phai [`Semantics::GlobalOnly`] cung vay.
/// Bang `config_value` phuc vu rieng ba loai do, va mot lenh `DELETE` khong kiem loai la
/// mot cua sau vao chinh cai lenh `INSERT` da khoa.
///
/// ⚠️ Xoa mot khoa **khong ton tai** la **thanh cong**, khong mot loi. Nut *"tra ve mac
/// dinh"* bam duoc o moi hang, ke ca hang chua ai dong toi, va bat nguoi goi phai biet
/// truoc hang do co ton tai khong la dung thu doi hoi mot vong doc thua truoc moi luot ghi.
pub fn delete_value(store: &Store, kind: &str, key: &str) -> Result<(), StoreError> {
    let Some(parsed) = ScopeKind::from_wire(kind) else {
        return Err(StoreError::WriteFailed {
            store: store.kind(),
            detail: format!("unknown scope kind {kind:?}; nothing was deleted"),
        });
    };

    if !matches!(parsed.semantics(), Semantics::GlobalOnly) {
        return Err(StoreError::WriteFailed {
            store: store.kind(),
            detail: format!(
                "scope kind {:?} is {:?}, and config_value serves GlobalOnly kinds only; \
                 nothing was deleted",
                parsed.as_str(),
                parsed.semantics()
            ),
        });
    }

    // So huu tuong minh: job ghi chay tren luong writer nen no phai `Send + 'static`.
    let kind = parsed.as_str().to_owned();
    let key = key.to_owned();

    store.write(move |tx: &Transaction<'_>| {
        tx.execute(
            "DELETE FROM config_value WHERE kind = ?1 AND key = ?2",
            (&kind, &key),
        )?;
        Ok(())
    })?;

    Ok(())
}
