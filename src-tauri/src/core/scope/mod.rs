//! `ScopeResolver` — phân giải hai tầng Global / Tác phẩm, ngữ nghĩa khai báo
//! tường minh (AD-18).
//!
//! Mọi tra cứu hai tầng đi qua đây; ghi đè hay hợp nhất là quyết định đã ghi ở AD-18,
//! không phải thứ mỗi nơi gọi tự chọn.
//!
//! ─────────────────────────────────────────────────────────────────────────────
//! HÌNH DẠNG ĐÃ DỰNG (Story 1.8)
//! ─────────────────────────────────────────────────────────────────────────────
//! - [`kinds`] — `scope_kinds!` sinh [`ScopeKind`] · [`Semantics`] · bảng AD-18 (AC4).
//! - [`resolve`] — ba hàm **thuần**, `pub(crate)`, phơi ra chỉ qua [`ScopeResolver`] (AC1).
//! - [`store`] — nạp/ghi tầng Global qua `core::store` (AC5).
//!
//! ─────────────────────────────────────────────────────────────────────────────
//! 🔴 "ĐÚNG MỘT `ScopeResolver`" ĐƯỢC CƯỠNG CHẾ BẰNG **HAI** CƠ CHẾ, KHÔNG PHẢI MỘT
//! ─────────────────────────────────────────────────────────────────────────────
//! Đúng khuôn AC2 của Story 1.7, và vì cùng lý do — mỗi vế để hở đúng chỗ vế kia đóng:
//!
//! 1. **Kiểu** — [`Semantics`] và ba hàm phân giải chỉ tồn tại trong module này. Không
//!    module nào gọi được [`ScopeResolver::apply_override`] cho một loại đã khai
//!    [`Semantics::Merge`]: gọi sai trả [`ScopeError::WrongSemantics`], ⛔ không im lặng
//!    làm theo ý người gọi.
//! 2. **Test** — `tests/scope_boundary.rs` quét cây nguồn: một danh sách token cấm chỉ
//!    được xuất hiện dưới `src/core/scope/**`.
//!
//! ⚠️ Hôm nay **chưa có consumer nào** — Glossary/TM/Prompt/AI/Luật làm sạch đều là module
//! rỗng. Điều đó **không** làm AC1 thành mệnh đề vòng: cổng quét cấm **token**, nên nó đỏ
//! ngay lần đầu một module Epic 3 tự viết một nhánh `if work.is_some()`. Đó chính là lượt
//! đỏ mà phép kiểm này tồn tại để mua.
//!
//! ─────────────────────────────────────────────────────────────────────────────
//! 🔴 VÌ SAO TẦNG TÁC PHẨM LÀ `Option::None` Ở KHẮP NƠI HÔM NAY
//! ─────────────────────────────────────────────────────────────────────────────
//! `.atproj` và `project.db` là **Story 1.15**; `StoreKind::Project` chưa có `StoreSpec`
//! nào (`core/store/mod.rs:122-134`). Hệ quả, và nó là **thiết kế chứ không phải thiếu
//! sót**:
//! - Mọi chữ ký nhận tầng Tác phẩm là `Option<&…>`, và đường sản phẩm hôm nay luôn `None`.
//! - Nhánh `Some(..)` **vẫn phải đúng và vẫn phải có test** — `tests/scope_contract.rs`
//!   cấp dữ liệu tầng Work bằng tay. Đó là hợp đồng mà Story 1.15 cắm `project.db` vào.
//! - ⛔ Không dựng `StoreSpec::work()`, không mở kho thứ hai, không đoán `meta.json`.
//!
//! Nói cách khác: **story này giao xong hợp đồng hai tầng, và giao xong một tầng.**
//!
//! ─────────────────────────────────────────────────────────────────────────────
//! ⛔ MODULE NÀY KHÔNG `use tauri::…` — cùng lý do Quyết định #1 của Story 1.7
//! ─────────────────────────────────────────────────────────────────────────────
//! Test dựng được trên một thư mục tạm mà không cần webview. Đường lấy `AppHandle`/`State`
//! nằm ở `commands/`, và `tests/scope_boundary.rs` canh mệnh đề này.
//!
//! ⛔ **Không khai trait nào.** AD-2 khoá số cổng ở **ba** và nói rõ `ScopeResolver`
//! **không phải** một cổng; AD-40 đã lập tiền lệ *"hai module Rust thường, không trait
//! hoá"*. `ports/mod.rs` giữ nguyên 5 dòng.
//!
//! ⛔ **Không cache.** Consumer đường nóng duy nhất là khớp Glossary khi gõ (Story 3.4,
//! dưới trần NFR2 *không frame nào vượt 50 ms*), và hôm nay nó chưa tồn tại. Dựng cache
//! bây giờ là dựng một cơ chế vô hiệu hoá mà không có gì để vô hiệu hoá.
//!
//! ⚠️ Mọi chuỗi trong module này viết KHÔNG DẤU — xem doc-comment của [`kinds`].

use std::cmp::Ordering;
use std::collections::BTreeMap;

pub mod kinds;
pub mod resolve;
pub mod store;

pub use kinds::{ScopeKind, Semantics};
pub use resolve::{Resolved, Tiered};
pub use store::{DEFAULT_MODE, DEFAULT_THEME, GlobalConfig, load_global_config, save_value};

/// Hai tầng của AD-18. **`Work`, ⛔ không phải `Project`.**
///
/// Consistency Conventions (`ARCHITECTURE-SPINE.md:538`) cấm `Project` cho thực thể Tác
/// phẩm. `StoreKind::Project` đặt tên cho **tệp** `project.db`, ⛔ không cho tầng — hai
/// khái niệm khác nhau tình cờ nằm cạnh nhau, và gộp tên chúng là mở đúng loại nhầm lẫn
/// mà một quy ước đặt tên tồn tại để chặn.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Tier {
    /// `$APPDATA/global.db` — cấu hình chung, dùng cho mọi Tác phẩm.
    Global,
    /// `<tác phẩm>.atproj/project.db` — riêng một Tác phẩm. **Story 1.15**.
    Work,
}

impl Tier {
    /// Định danh máy đọc. ⛔ Không phải nhãn hiển thị (AD-21, NFR16).
    pub const fn as_str(self) -> &'static str {
        match self {
            Tier::Global => "global",
            Tier::Work => "work",
        }
    }

    /// Thứ hạng khi tầng làm **khoá phụ**: Work trước Global (AD-18).
    ///
    /// 🔴 Một hàm chứ không phải `#[derive(Ord)]` trên thứ tự khai báo, và đó là chủ ý:
    /// thứ tự khai báo đọc tự nhiên là *Global rồi Work* (tầng dưới trước tầng trên), còn
    /// thứ tự **sắp xếp** thì ngược lại. Buộc hai thứ đó vào một `derive` nghĩa là đảo
    /// một trong hai sẽ im lặng đảo cái kia.
    pub(crate) const fn rank(self) -> u8 {
        match self {
            Tier::Work => 0,
            Tier::Global => 1,
        }
    }
}

/// Mọi cách phân giải hai tầng hỏng.
///
/// 🔴 **Lỗi LẬP TRÌNH, không phải lỗi người dùng** — và đó là lý do nó ⛔ **không**
/// `impl From<ScopeError> for IpcError` và ⛔ **không bao giờ** vượt ranh giới IPC.
///
/// Story 1.7 §Completion Notes #3 khoá quy tắc: *"⛔ Không khoá `MessageKey` nào cho tính
/// năng chưa tồn tại."* Cả hai biến thể dưới đây chỉ xảy ra khi mã gọi sai hàm cho loại
/// của nó — không có gì để nói với người dùng, và một câu tiếng Việt cho một lỗi mà chỉ
/// người viết mã gây ra được là một khoá chuỗi không ai nghiệm thu được.
///
/// ⚠️ Mọi lỗi mà story này thật sự phát ra qua IPC đều là lỗi **kho**, và cả năm khoá của
/// chúng đã có từ Story 1.7 kèm `From<StoreError> for IpcError`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScopeError {
    /// Gọi hàm phân giải sai với ngữ nghĩa mà `kind` đã khai.
    WrongSemantics {
        /// Loại dữ liệu bị gọi sai.
        kind: ScopeKind,
        /// Ngữ nghĩa mà bảng AD-18 khai cho nó.
        declared: Semantics,
        /// Ngữ nghĩa của hàm vừa được gọi.
        called: Semantics,
    },

    /// Dữ liệu tầng Work đi vào một loại [`Semantics::GlobalOnly`].
    WorkTierForbidden {
        /// Loại dữ liệu chỉ tồn tại ở tầng Global.
        kind: ScopeKind,
    },
}

impl std::fmt::Display for ScopeError {
    /// ⚠️ KHÔNG DẤU — chẩn đoán cho log, không phải văn bản hiển thị (NFR16).
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ScopeError::WrongSemantics {
                kind,
                declared,
                called,
            } => write!(
                f,
                "scope[{}] declares {declared:?} but was resolved as {called:?}",
                kind.as_str()
            ),
            ScopeError::WorkTierForbidden { kind } => write!(
                f,
                "scope[{}] is global-only; a work tier was supplied",
                kind.as_str()
            ),
        }
    }
}

impl std::error::Error for ScopeError {}

/// Chỗ **DUY NHẤT** phân giải được hai tầng (AC1).
///
/// ⚠️ Hôm nay `work` **luôn** là `None` và [`ScopeResolver::global_only`] là hàm dựng duy
/// nhất tồn tại — xem *"vì sao tầng Tác phẩm là `Option::None`"* ở doc-comment của module.
/// Story 1.15 thêm hàm dựng thứ hai, và ba method dưới đây không phải đổi chữ ký.
#[derive(Debug, Clone, Default)]
pub struct ScopeResolver {
    /// `None` = *"chưa mở Tác phẩm nào"*, trạng thái **duy nhất** tồn tại hôm nay.
    work: Option<WorkScope>,
}

/// Chỗ Story 1.15 điền tầng Tác phẩm vào.
///
/// ⚠️ Rỗng có chủ ý, và ⛔ **đừng đoán trước hình dạng của nó**: `.atproj` là một thư mục
/// (AD-9), `project.db` nằm trong đó chứ không ở `$APPDATA`, và `meta.json` chưa được
/// thiết kế. Viết sẵn trường hôm nay là viết mã không ai gọi, và nó sẽ sai theo đúng cách
/// mà không test nào bắt (`core/store/mod.rs:122-134`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct WorkScope;

impl ScopeResolver {
    /// Hàm dựng **duy nhất** hôm nay: chưa mở Tác phẩm nào.
    ///
    /// AC5 nguyên văn: *"tầng Global phân giải được khi ứng dụng chạy mà chưa mở Tác phẩm
    /// nào"* — và đó không phải một ca biên, đó là trạng thái duy nhất tồn tại.
    pub const fn global_only() -> Self {
        Self { work: None }
    }

    /// Đã mở một Tác phẩm chưa. Hôm nay **luôn** `false`.
    pub const fn has_work_tier(&self) -> bool {
        self.work.is_some()
    }

    /// **Ghi đè theo từng khoá** (AC2). Xem [`resolve::resolve_override`].
    ///
    /// ⚠️ Tên **khác** hàm nội bộ nó bọc (`apply_` chứ không `resolve_`) — có chủ đích, xem
    /// doc-comment của [`resolve::resolve_override`] và [`Self::apply_merge`].
    ///
    /// # Lỗi
    /// [`ScopeError::WrongSemantics`] nếu `kind` không khai [`Semantics::Override`].
    pub fn apply_override<K, V>(
        &self,
        kind: ScopeKind,
        global: &BTreeMap<K, V>,
        work: Option<&BTreeMap<K, V>>,
    ) -> Result<BTreeMap<K, Resolved<V>>, ScopeError>
    where
        K: Ord + Clone,
        V: Clone,
    {
        resolve::resolve_override(kind, global, work)
    }

    /// **Hợp nhất hai tầng**, tầng là khoá phụ (AC3). Xem [`resolve::resolve_merge`].
    ///
    /// ─────────────────────────────────────────────────────────────────────────────
    /// ⚠️ VÌ SAO `apply_*`, KHÔNG PHẢI `resolve_*` — lượt review kiến trúc bắt được
    /// ─────────────────────────────────────────────────────────────────────────────
    /// `tests/scope_boundary.rs::only_core_scope_may_name_the_two_tier_vocabulary` cấm
    /// token `"resolve_override"`/`"resolve_merge"` ngoài `core/scope/**` — chốt vì hai
    /// hàm đó là `pub(crate)` trong [`resolve`], nên một module cùng crate (Epic 3–7) gọi
    /// thẳng chúng để **lách** `ScopeResolver` vẫn biên dịch được. Nếu phương thức công
    /// khai của `ScopeResolver` mang **đúng tên** hai hàm bị cấm đó, thì chính lời gọi
    /// SẢN PHẨM, ĐÚNG ĐẮN (`resolver.resolve_override(...)`) cũng mang cùng chuỗi và tự
    /// làm cổng đỏ — cổng không phân biệt được "gọi đúng cửa" với "lách qua cửa sau".
    ///
    /// Đặt tên khác (`apply_override`/`apply_merge`) tách hai lời gọi đó ra: gọi qua
    /// `ScopeResolver` — con đường được phép — không còn mang token bị cấm; gọi thẳng
    /// `resolve::resolve_override`/`resolve_merge` — con đường lách — vẫn mang, và vẫn đỏ
    /// đúng như AC1 đòi.
    ///
    /// # Lỗi
    /// [`ScopeError::WrongSemantics`] nếu `kind` không khai [`Semantics::Merge`].
    pub fn apply_merge<V>(
        &self,
        kind: ScopeKind,
        global: &[V],
        work: Option<&[V]>,
        primary: Option<&dyn Fn(&V, &V) -> Ordering>,
    ) -> Result<Vec<Tiered<V>>, ScopeError>
    where
        V: Clone,
    {
        resolve::resolve_merge(kind, global, work, primary)
    }

    /// **Chỉ tầng Global** (AC5). Xem [`resolve::resolve_global_only`].
    ///
    /// # Lỗi
    /// - [`ScopeError::WrongSemantics`] nếu `kind` không khai [`Semantics::GlobalOnly`];
    /// - [`ScopeError::WorkTierForbidden`] nếu `work` là `Some(..)` và không rỗng.
    pub fn resolve_global_only<K, V>(
        &self,
        kind: ScopeKind,
        global: &BTreeMap<K, V>,
        work: Option<&BTreeMap<K, V>>,
    ) -> Result<BTreeMap<K, Resolved<V>>, ScopeError>
    where
        K: Ord + Clone,
        V: Clone,
    {
        resolve::resolve_global_only(kind, global, work)
    }
}
