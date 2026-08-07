//! `ScopeKind` · `Semantics` · **bảng ngữ nghĩa của AD-18** — AC2, AC3, AC4.
//!
//! ─────────────────────────────────────────────────────────────────────────────
//! 🔴 VÌ SAO MỘT MACRO, KHÔNG PHẢI HAI BẢNG VIẾT TAY
//! ─────────────────────────────────────────────────────────────────────────────
//! AC4 nguyên văn: *"thêm một loại dữ liệu mới phải khai ngữ nghĩa tường minh, và
//! **không có mặc định ngầm nào**"*. Một `enum` cộng một `fn semantics()` viết tay là
//! hai bản chép phải khớp nhau bằng kỷ luật, và cách chúng trôi khỏi nhau đã biết
//! trước: ai đó thêm một biến thể, trình biên dịch bắt `match` thiếu nhánh, người sửa
//! thêm nhánh **cho hết đỏ** chứ không phải sau khi đọc AD-18 — hoặc tệ hơn, thêm một
//! nhánh `_ =>` và từ đó mọi loại mới im lặng nhận ngữ nghĩa của loại cuối cùng.
//!
//! Macro đóng đúng đường đó: **không tồn tại cú pháp nào khai được một biến thể mà
//! không kèm ngữ nghĩa**. Đây là đúng khuôn `message_keys!` của Story 1.5
//! (`core/i18n/mod.rs:62`), và vì cùng một lý do: *một khai báo, nhiều thứ sinh ra, nên
//! chúng không trôi khỏi nhau được*.
//!
//! **Không `impl Default for Semantics`, không `#[derive(Default)]`.** Một ngữ nghĩa
//! mặc định là chính xác cái lỗ mà AC4 tồn tại để bịt: nó biến *"quên khai"* thành
//! *"khai một thứ hợp lệ mà không ai chọn"*, và lỗi đó không đỏ ở đâu cả.
//!
//! **Không nhánh `_ =>` trong bất kỳ `match` nào trên [`ScopeKind`].** Sức mạnh duy
//! nhất của một `enum` đóng là việc trình biên dịch đỏ khi có biến thể mới; một nhánh
//! bắt-tất-cả đổi nó lấy sự yên tĩnh.
//!
//! ─────────────────────────────────────────────────────────────────────────────
//! ⚠️ CHÍN LOẠI — SÁU CỦA AD-18, BA CỦA FR103
//! ─────────────────────────────────────────────────────────────────────────────
//! Ba loại cuối (`Shortcut` · `LayoutPreset` · `AppConfig`) **không có trong bảng gốc
//! của AD-18**, và đó chính là ca mà AC4 tồn tại để bắt. Chúng mang ngữ nghĩa thứ ba,
//! [`Semantics::GlobalOnly`], vì FR103 đặt chúng ở tầng Global **và không cho chúng đối
//! ứng ở tầng Tác phẩm**; `mockups/settings.html:246` nói thẳng: *"Phím tắt chỉ tồn tại
//! ở tầng Toàn cục — một thao tác không nên đổi phím theo từng Tác phẩm."*
//!
//! Khai chúng là `Override` là **sai im lặng**: nó mở một tầng Tác phẩm mà UX đã cấm,
//! và Story 1.14/1.21 sẽ dựng thanh chuyển phạm vi cho một thứ không nên có.
//! Ba hàng này đã được Ice phê chuẩn 2026-08-04 và ghi vào bảng AD-18.
//!
//! ─────────────────────────────────────────────────────────────────────────────
//! `ngôn ngữ nguồn` KHÔNG PHẢI MỘT LOẠI Ở ĐÂY — và đừng thêm nó vào
//! ─────────────────────────────────────────────────────────────────────────────
//! FR103 liệt kê nó ở tầng Tác phẩm, nên cám dỗ là hiển nhiên. Nhưng Story 5.1 định
//! nghĩa nó là trường **bất biến** trong `meta.json`, đặt lúc tạo Tác phẩm và không đổi
//! được, và bản PRD (`prd.md:765-774`) ghi rõ *"(cố định, đặt lúc tạo)"* — mệnh đề mà
//! `epics.md:296` làm rơi mất. Nó **không có đối ứng ở tầng Global**, nên không có gì
//! để ghi đè và không có gì để hợp nhất. Nó là **thuộc tính của Work**, không phải cấu
//! hình hai tầng. Ghi ở đây để Story 1.15 không phải đoán lại. *(Ice ký 2026-08-04.)*
//!
//! ⚠️ Mọi chuỗi trong tệp này viết KHÔNG DẤU — `scripts/check-i18n.mjs` Kiểm A quét
//! `src-tauri/**/*.rs` tìm ký tự có dấu tiếng Việt ở **vị trí mã**, và `src/core/**`
//! không nằm trong danh sách miễn trừ. Doc-comment thì có dấu thoải mái.

/// **Ba** ngữ nghĩa phân giải, và đúng ba. Không `Default`.
///
/// Mỗi biến thể ứng với đúng một trong ba hàm phân giải của [`super::resolve`], và
/// [`super::ScopeResolver`] từ chối gọi chéo — xem [`super::ScopeError::WrongSemantics`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Semantics {
    /// **Ghi đè theo TỪNG KHOÁ** — tầng Tác phẩm thắng trên khoá trùng, và chỉ trên
    /// khoá trùng.
    ///
    /// 🔴 **Không phải ghi đè theo cả tập.** AD-18 viết *"tầng Tác phẩm thắng theo
    /// từng thuật ngữ"*; Story 3.4 nói cùng luật ở chiều ngược lại: *"áp cả hai, tầng
    /// Tác phẩm thắng khi trùng"*. Một cài đặt *"work rỗng thì dùng global, work không
    /// rỗng thì dùng work"* làm 412 mục Glossary toàn cục biến mất ngay khi người dùng
    /// thêm một mục riêng cho Tác phẩm — xem [`super::resolve::resolve_override`].
    Override,

    /// **Cả hai tầng cùng áp**, không khử trùng lặp *(AD-19: giữ nguyên bất đồng)*.
    ///
    /// Mỗi mục mang nhãn tầng của **chính nó**, không phải cả tập mang một nhãn — Story
    /// 6.5 đòi *"mỗi luật mang nhãn tầng"*, và màn quản lý Glossary đòi hiện mục toàn
    /// cục *"đang bị che"*.
    Merge,

    /// Chỉ tồn tại ở tầng Global; một tầng Tác phẩm cho loại này là **lỗi lập trình**.
    ///
    /// Hàng thứ ba, **mở rộng bảng AD-18** — xem doc-comment của module. Một giá trị
    /// tầng Work đi vào đây trả [`super::ScopeError::WorkTierForbidden`] chứ không
    /// bị bỏ qua im lặng: bỏ qua im lặng là cách một tầng bị cấm vẫn được ghi vào đĩa
    /// rồi không bao giờ có tác dụng, tức đúng lớp lỗi *"trông như đang chạy"*.
    GlobalOnly,
}

/// Khai MỘT CHỖ DUY NHẤT, sinh ra bốn thứ phải khớp nhau: `enum ScopeKind`,
/// `ScopeKind::ALL`, `ScopeKind::as_str()` và `ScopeKind::semantics()`.
///
/// Cú pháp: `Variant => "khoa_tren_day" : Semantics::X`. Ngữ nghĩa nằm **trong cùng
/// khai báo** với biến thể, nên AC4 được cưỡng chế bằng trình biên dịch chứ không bằng
/// một dòng trong tài liệu.
///
/// ⚠️ `$(#[$meta:meta])*` không phải trang trí: doc-comment nở ra thành `#[doc = "…"]`,
/// nên một macro không khai chỗ nhận attribute sẽ từ chối biên dịch ngay khi ai đó viết
/// một dòng `///` cho biến thể đầu tiên. Bảng này tồn tại để được chú thích — mỗi hàng
/// mang nguồn (AD/FR) và chủ sở hữu dữ liệu của nó.
macro_rules! scope_kinds {
    ($($(#[$meta:meta])* $variant:ident => $wire:literal : $semantics:ident),+ $(,)?) => {
        /// Mọi loại dữ liệu phân giải hai tầng. Bảng AD-18, cưỡng chế bằng kiểu.
        ///
        /// ⚠️ `Ord` có chủ ý: nó là điều kiện để `ALL` sắp xếp ổn định trong test và để
        /// một `BTreeMap` khoá theo loại có thứ tự đọc được.
        #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
        pub enum ScopeKind {
            $($(#[$meta])* $variant),+
        }

        impl ScopeKind {
            /// Mọi biến thể. Sinh từ CÙNG khai báo với `semantics()` nên không trôi được.
            ///
            /// 🔴 Test `the_kind_table_has_every_variant` so độ dài này với một hằng số
            /// **viết tay**: thêm một biến thể mà quên nó thì không đỏ ở đây (macro tự
            /// sinh), nhưng con số viết tay là chỗ một con người phải ký.
            pub const ALL: &'static [ScopeKind] = &[$(ScopeKind::$variant),+];

            /// Định danh máy đọc — thứ đi trên dây và thứ nằm ở cột `kind` của
            /// `config_value`. Không phải nhãn hiển thị (AD-21, NFR16).
            pub const fn as_str(self) -> &'static str {
                match self {
                    $(ScopeKind::$variant => $wire),+
                }
            }

            /// Ngữ nghĩa phân giải của loại này. **Bảng AD-18, đọc được bằng máy.**
            ///
            /// Không nhánh `_ =>`: xem doc-comment của module.
            pub const fn semantics(self) -> Semantics {
                match self {
                    $(ScopeKind::$variant => Semantics::$semantics),+
                }
            }

            /// Phân giải một khoá đến **từ bên ngoài** (dây IPC, cột `kind` trên đĩa).
            ///
            /// ⚠️ Nhánh `_ => None` ở đây **không** vi phạm luật cấm `_ =>`: luật đó nói
            /// về `match` **trên `ScopeKind`**, nơi một nhánh bắt-tất-cả nuốt mất biến
            /// thể mới. Đây là `match` trên `&str` — một tập vô hạn và không tin được —
            /// nên nhánh cuối là bắt buộc, và nó trả `None` chứ không đoán.
            pub fn from_wire(raw: &str) -> Option<ScopeKind> {
                match raw {
                    $($wire => Some(ScopeKind::$variant),)+
                    _ => None,
                }
            }
        }
    };
}

// ─────────────────────────────────────────────────────────────────────────────────
// BẢNG AD-18 — chín hàng. Sáu hàng gốc + ba hàng `GlobalOnly` của FR103.
//
// Cột "Chủ sở hữu dữ liệu" là một mệnh đề về PHẠM VI: story này khai NGỮ NGHĨA của cả
// chín loại, và không khai BẢNG của loại nào ngoài ba loại `GlobalOnly` cuối. Xem
// §Quyết định #1: nhét Glossary (phân loại/xuất xứ/vòng đời ba trạng thái) và TM (cặp
// văn bản + xuất xứ) vào một cột `value TEXT` là dựng một lược đồ EAV mà bốn epic sau
// phải bóc ra.
// ─────────────────────────────────────────────────────────────────────────────────
scope_kinds! {
    /// Thuật ngữ đã chốt. **AD-18 · FR46** — dữ liệu thuộc **Epic 3**.
    ///
    /// Ghi đè **theo từng thuật ngữ**, và `mockups/glossary-manage.html:169` vẽ mục toàn
    /// cục bị che hiện inline cạnh mục thắng — tức `shadowed` phải có mặt trong kết quả.
    Glossary => "glossary" : Override,

    /// Bộ prompt theo thể loại. **AD-18 · FR69** — dữ liệu thuộc **Epic 4**.
    Prompt => "prompt" : Override,

    /// Cấu hình nhà cung cấp AI. **AD-18 · FR68** — dữ liệu thuộc **Epic 4**.
    ///
    /// 🔴 Ghi đè **theo TỪNG TRƯỜNG**, không theo cả struct. AD-18 chỉ ghi *"Cấu hình AI
    /// | ghi đè"* và Story 4.2 chỉ nói *"ghi đè được theo Tác phẩm đó"*; chỉ
    /// `mockups/settings.html` lộ ra rằng trong **cùng một** cấu hình có trường `ghi đè`
    /// và trường `kế thừa` cùng lúc (`:172`, `:188`, `:200`). Nghĩa là Epic 4 phân giải
    /// nó như một map `khoá trường -> giá trị`, y hệt Glossary — **không** như một
    /// giá trị nguyên khối. *(Ice ký 2026-08-04; chốt ở đây rẻ hơn chốt ở Epic 4.)*
    AiConfig => "ai_config" : Override,

    /// Tên người dịch cho khối ghi nguồn. **AD-18 · FR131 · AD-43** — **Epic 8**.
    ///
    /// ⚠️ Hàng này tới AD-18 muộn: lượt review kiến trúc 2026-08-03b §F4 bắt được nó đã
    /// đi qua `ScopeResolver` mà **không có hàng ngữ nghĩa nào**.
    TranslatorName => "translator_name" : Override,

    /// Cặp câu nguồn/đích đã dịch. **AD-18 · FR57** — dữ liệu thuộc **Epic 7**.
    ///
    /// 🔴 Hợp nhất, và **tầng là khoá PHỤ**. AD-18 khai khoá chính là *xuất xứ* (FR118)
    /// và giải thích vì sao: *"một cặp TM toàn cục do chính người dùng dịch vẫn giống văn
    /// phong của họ hơn một cặp Tác phẩm do người khác dịch."* Đảo hai khoá cho ra một
    /// danh sách trông có thứ tự và hỏng đúng mục đích của FR70 — xem
    /// [`super::resolve::resolve_merge`].
    TranslationMemory => "translation_memory" : Merge,

    /// Luật làm sạch văn bản lúc nhập. **AD-18 · FR124** — dữ liệu thuộc **Epic 6**.
    ///
    /// Hợp nhất, và `EXPERIENCE.md:216` *(UX-DR40)* đòi **mỗi luật mang nhãn tầng**
    /// — Toàn cục hoặc Tác phẩm. Nhãn nằm trên **từng mục**, không trên cả tập.
    ///
    /// ⚠️ Hàng thứ hai mà lượt review §F4 phải vá vào bảng AD-18.
    ImportCleanupRule => "import_cleanup_rule" : Merge,

    /// Hợp âm phím tắt. **FR103 · AC5** — màn gán phím là **Story 1.21**.
    ///
    /// `GlobalOnly`: `mockups/settings.html:246` — *"một thao tác không nên đổi phím theo
    /// từng Tác phẩm"*.
    Shortcut => "shortcut" : GlobalOnly,

    /// Preset bố cục panel đã đặt tên. **FR103 · AC5** — nội dung là **Story 1.14**.
    ///
    /// ⚠️ AD-1 gọi *"bố cục panel"* là state UI của frontend, còn FR103 đặt preset ở
    /// `global.db`. Hai câu không mâu thuẫn nếu tách đúng: **bố cục đang hiển thị** là
    /// của frontend; **preset đã đặt tên và lưu lại** là dữ liệu Rust, đọc qua
    /// `ScopeResolver`, ghi qua `store::Writer`. Ghi ở đây vì Story 1.14 sẽ phải chọn,
    /// và cách đọc kia dẫn thẳng tới `localStorage`.
    LayoutPreset => "layout_preset" : GlobalOnly,

    /// Lựa chọn ứng dụng: theme, chế độ cuối cùng. **AC5** — sở hữu bởi **story này**.
    ///
    /// Loại duy nhất trong bảng mà story này vừa khai ngữ nghĩa vừa khai dữ liệu.
    AppConfig => "app_config" : GlobalOnly,
}
