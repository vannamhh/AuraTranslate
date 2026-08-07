//! Ba hàm phân giải — **THUẦN**, không chạm đĩa, không chạm [`super::super::store`].
//!
//! AC1 · AC2 · AC3. Chúng nhận **dữ liệu đã nạp** từ chỗ gọi và trả về dữ liệu đã phân
//! giải; mỗi module miền tự sở hữu bảng của nó và tự nạp hai tầng (§Quyết định #1).
//!
//! ─────────────────────────────────────────────────────────────────────────────
//! 🔴 `pub(crate)`, KHÔNG `pub` — và đó là vế "đúng một `ScopeResolver`" ở tầng kiểu
//! ─────────────────────────────────────────────────────────────────────────────
//! Ba hàm dưới đây phơi ra bên ngoài **chỉ** dưới dạng method của
//! [`super::ScopeResolver`]. AC1 nói *"mọi phân giải hai tầng đi qua đúng một
//! `ScopeResolver`"*, và một hàm tự do `pub` là một đường thứ hai — nó biên dịch sạch,
//! chạy đúng, và làm mệnh đề *"đúng một"* thành một lời hứa thay vì một phép cưỡng chế.
//!
//! ⚠️ Mọi chuỗi trong tệp này viết KHÔNG DẤU — xem doc-comment của [`super::kinds`].

use std::cmp::Ordering;
use std::collections::BTreeMap;

use super::kinds::{ScopeKind, Semantics};
use super::{ScopeError, Tier};

/// Một giá trị đã phân giải, **kèm xuất xứ theo từng khoá** (AC2).
///
/// 🔴 `shadowed` **không phải trang trí**, và bỏ nó đi hôm nay là buộc hai màn hình đã
/// vẽ sẵn phải tự truy vấn lại tầng Global:
/// - `mockups/settings.html:172` vẽ *"Ghi đè Toàn cục — ở tầng Toàn cục đang là
///   **Anthropic**"* ngay cạnh giá trị đang thắng;
/// - `mockups/glossary-manage.html:169` vẽ mục toàn cục *"đang bị che"*.
///
/// Tức đúng cái *"một truy vấn riêng"* mà Story 3.1 cấm.
///
/// ─────────────────────────────────────────────────────────────────────────────
/// 🔴 TRẢ VỀ KẾ THỪA = **XOÁ HÀNG TẦNG WORK**, KHÔNG PHẢI CHÉP GIÁ TRỊ GLOBAL XUỐNG
/// ─────────────────────────────────────────────────────────────────────────────
/// Chưa cài hôm nay (chưa có tầng Work), nhưng luật phải đọc được **ở đây** vì hai đường
/// này **không phân biệt được ở khoảnh khắc bấm nút** rồi phân kỳ mãi mãi sau đó: chép
/// giá trị Global xuống làm mục đó **đóng băng** ở giá trị cũ, và lần sau người dùng đổi
/// cấu hình chung thì Tác phẩm này im lặng không theo. `settings.html:228` có nút *"Trả
/// toàn bộ mục về kế thừa"*; ai cài nó phải đọc được luật này trước khi gõ dòng đầu tiên.
///
/// ⚠️ **Trường riêng tư.** Bất biến duy nhất của struct này — `tier == Tier::Global` thì
/// `shadowed` **luôn** là `None`, vì Global là tầng dưới cùng và không có gì dưới nó để
/// che — chỉ giữ được nếu chỗ dựng là [`super::scope`] chứ không phải mọi chỗ gọi.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Resolved<V> {
    value: V,
    tier: Tier,
    shadowed: Option<V>,
}

impl<V> Resolved<V> {
    /// Chỗ DUY NHẤT dựng được một [`Resolved`]. Xem bất biến ở doc-comment của struct.
    pub(crate) fn new(value: V, tier: Tier, shadowed: Option<V>) -> Self {
        debug_assert!(
            !(matches!(tier, Tier::Global) && shadowed.is_some()),
            "Resolved -- tier=Global cannot shadow anything; Global is the bottom tier"
        );
        Self {
            value,
            tier,
            shadowed,
        }
    }

    /// Giá trị đang thắng.
    pub fn value(&self) -> &V {
        &self.value
    }

    /// Tầng sinh ra giá trị đang thắng.
    pub const fn tier(&self) -> Tier {
        self.tier
    }

    /// Giá trị **bị che** ở tầng dưới, nếu có. `None` khi khoá chỉ tồn tại ở một tầng.
    pub fn shadowed(&self) -> Option<&V> {
        self.shadowed.as_ref()
    }
}

/// Một mục của kết quả hợp nhất, **mang nhãn tầng của chính nó** (AC3 mệnh đề 2).
///
/// ⚠️ Nhãn nằm trên **từng mục**, không phải cả tập mang một nhãn. Story 6.5 đòi
/// *"mỗi luật mang nhãn tầng — Toàn cục hoặc Tác phẩm"*, và một `(Tier, Vec<V>)` không
/// diễn đạt được một danh sách đã trộn thứ tự theo xuất xứ.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Tiered<V> {
    tier: Tier,
    value: V,
}

impl<V> Tiered<V> {
    /// Chỗ DUY NHẤT dựng được một [`Tiered`] — cùng lý do với [`Resolved::new`].
    pub(crate) const fn new(tier: Tier, value: V) -> Self {
        Self { tier, value }
    }

    /// Tầng sinh ra mục này.
    pub const fn tier(&self) -> Tier {
        self.tier
    }

    /// Mục.
    pub fn value(&self) -> &V {
        &self.value
    }
}

/// Cổng ngữ nghĩa: gọi sai hàm cho `kind` ⇒ `Err`, **không im lặng làm theo ý người gọi**.
///
/// 🔴 `Err` chứ không `panic!`/`unwrap()`, và ở **cả debug lẫn release**: `Cargo.toml`
/// ghim `panic = "abort"` ở `[profile.release]`, nên một panic ở đây giết cả tiến trình
/// và cuốn theo writer nối tiếp của AD-11/AD-12.
fn require(kind: ScopeKind, called: Semantics) -> Result<(), ScopeError> {
    let declared = kind.semantics();
    if declared == called {
        return Ok(());
    }
    Err(ScopeError::WrongSemantics {
        kind,
        declared,
        called,
    })
}

/// **Ghi đè theo TỪNG KHOÁ** — AC2.
///
/// ```text
/// kết quả = tất cả khoá của Global ∪ tất cả khoá của Work
/// với mỗi khoá:  có ở Work   ⇒ value = Work,   tier = Work,   shadowed = Global (nếu có)
///                chỉ ở Global ⇒ value = Global, tier = Global, shadowed = None
/// ```
///
/// ─────────────────────────────────────────────────────────────────────────────
/// 🔴 BẪY SỐ MỘT CỦA CẢ STORY — hợp nhất khoá TRƯỚC, rồi mới cho Work thắng
/// ─────────────────────────────────────────────────────────────────────────────
/// Cài đặt sai trông rất hợp lý:
/// ```ignore
/// if let Some(w) = work { if !w.is_empty() { return w.clone(); } }
/// global.clone()
/// ```
/// Một test viết cẩu thả *(global có `a`, work có `a` khác giá trị, khẳng định kết quả là
/// giá trị của work)* **xanh**. Hành vi thật: người dùng có 412 mục Glossary toàn cục,
/// thêm **một** mục riêng cho Tác phẩm, và **411 mục kia biến mất**.
///
/// AD-18: *"tầng Tác phẩm thắng **theo từng thuật ngữ**"*. Story 3.4 cùng luật, rõ hơn:
/// *"áp **cả hai**, tầng Tác phẩm thắng khi trùng"*. Mệnh đề nghiệm thu bắt buộc: **một
/// khoá chỉ có ở Global phải còn trong kết quả, mang `tier: Global`.**
///
/// # Lỗi
/// [`ScopeError::WrongSemantics`] nếu `kind` không khai [`Semantics::Override`].
pub(crate) fn resolve_override<K, V>(
    kind: ScopeKind,
    global: &BTreeMap<K, V>,
    work: Option<&BTreeMap<K, V>>,
) -> Result<BTreeMap<K, Resolved<V>>, ScopeError>
where
    K: Ord + Clone,
    V: Clone,
{
    require(kind, Semantics::Override)?;
    Ok(merge_by_key(global, work))
}

/// **Hợp nhất hai tầng** — AC3.
///
/// Ba mệnh đề, và cả ba đều cưỡng chế được ở đây:
/// 1. Kết quả chứa mục của **cả hai** tầng, không khử trùng lặp *(AD-19: giữ nguyên
///    bất đồng — hai nguồn nói khác nhau là **thông tin**, không phải nhiễu)*.
/// 2. **Mỗi mục mang nhãn tầng** — xem [`Tiered`].
/// 3. **Tầng là khoá PHỤ, không bao giờ là khoá chính.**
///
/// ─────────────────────────────────────────────────────────────────────────────
/// 🔴 VÌ SAO `primary` ĐẾN TỪ CHỖ GỌI, VÀ VÌ SAO TẦNG KHÔNG TẮT ĐƯỢC
/// ─────────────────────────────────────────────────────────────────────────────
/// AD-18 khai khoá **chính** là *xuất xứ* (FR118), khoá **phụ** là *tầng* (Work trước
/// Global), và giải thích: *"một cặp TM toàn cục do **chính người dùng** dịch vẫn giống
/// văn phong của họ hơn một cặp Tác phẩm do người khác dịch."*
///
/// `core::scope` **không được biết** *xuất xứ* là gì — đó là dữ liệu trên bản ghi TM
/// (Story 7.2). Kéo nó vào đây là kéo cả miền TM vào bộ phân giải. Nên khoá chính đi vào
/// bằng tham số, còn khoá phụ thì **luôn** được áp và chỗ gọi **không có cách nào tắt**
/// — đó chính là vế của AD-18 mà story này cưỡng chế. AD-18 còn nói trước hậu quả của
/// việc đảo hai khoá: *"Không khai thứ tự này thì Giai đoạn 4 và Giai đoạn 6 sẽ cài lệch
/// nhau."*
///
/// ⚠️ `sort_by`, **không** `sort_unstable_by`: bản unstable phá thứ tự nguồn trong
/// nhóm bằng nhau, tức hai lượt chạy trên cùng dữ liệu cho hai danh sách khác nhau. Với
/// `primary = None` thì tầng là khoá **duy nhất**, nên toàn bộ thứ tự trong mỗi tầng đến
/// từ tính ổn định của phép sắp xếp.
///
/// # Lỗi
/// [`ScopeError::WrongSemantics`] nếu `kind` không khai [`Semantics::Merge`].
pub(crate) fn resolve_merge<V>(
    kind: ScopeKind,
    global: &[V],
    work: Option<&[V]>,
    primary: Option<&dyn Fn(&V, &V) -> Ordering>,
) -> Result<Vec<Tiered<V>>, ScopeError>
where
    V: Clone,
{
    require(kind, Semantics::Merge)?;

    // Thứ tự nạp không phải thứ tự kết quả — phép sắp xếp ngay dưới đây quyết định. Nạp
    // Global trước chỉ để người đọc thấy tầng dưới đi trước tầng trên trong mã.
    let mut out: Vec<Tiered<V>> = Vec::with_capacity(global.len() + work.map_or(0, <[V]>::len));
    for v in global {
        out.push(Tiered::new(Tier::Global, v.clone()));
    }
    for v in work.unwrap_or(&[]) {
        out.push(Tiered::new(Tier::Work, v.clone()));
    }

    out.sort_by(|a, b| {
        let by_tier = a.tier().rank().cmp(&b.tier().rank());
        match primary {
            // 🔴 `then` chứ không `then_with(|| …)` đảo chỗ: khoá chính chạy TRƯỚC, và
            // tầng chỉ phân xử khi khoá chính hoà. Đảo hai dòng này là Bẫy 2.
            Some(f) => f(a.value(), b.value()).then(by_tier),
            None => by_tier,
        }
    });

    Ok(out)
}

/// **Chỉ tầng Global** — AC5, và ngữ nghĩa thứ ba của bảng AD-18 mở rộng.
///
/// Kết quả có hình dạng y hệt [`resolve_override`] *(cùng `BTreeMap<K, Resolved<V>>`)*
/// để chỗ tiêu thụ không phải rẽ nhánh theo ngữ nghĩa — nhưng mọi mục **luôn** mang
/// `tier: Tier::Global` và `shadowed: None`, vì không có tầng nào dưới Global.
///
/// ⚠️ Tham số `work` tồn tại **để từ chối**, không phải để dùng: một chỗ gọi lỡ truyền
/// dữ liệu tầng Work cho một loại `GlobalOnly` phải nghe *"không"* ngay tại đây. Bỏ qua
/// im lặng là cách một tầng bị cấm vẫn được ghi xuống đĩa rồi không bao giờ có tác dụng
/// — hỏng đúng kiểu *"trông như đang chạy"*.
///
/// `Some(<rỗng>)` là hợp lệ và tương đương `None`: chỗ gọi hôm nay luôn ở trạng thái
/// *"chưa mở Tác phẩm nào"* và một map rỗng không khai một tầng nào cả.
///
/// # Lỗi
/// - [`ScopeError::WrongSemantics`] nếu `kind` không khai [`Semantics::GlobalOnly`];
/// - [`ScopeError::WorkTierForbidden`] nếu `work` là `Some(..)` và **không rỗng**.
pub(crate) fn resolve_global_only<K, V>(
    kind: ScopeKind,
    global: &BTreeMap<K, V>,
    work: Option<&BTreeMap<K, V>>,
) -> Result<BTreeMap<K, Resolved<V>>, ScopeError>
where
    K: Ord + Clone,
    V: Clone,
{
    require(kind, Semantics::GlobalOnly)?;

    if work.is_some_and(|w| !w.is_empty()) {
        return Err(ScopeError::WorkTierForbidden { kind });
    }

    Ok(global
        .iter()
        .map(|(k, v)| (k.clone(), Resolved::new(v.clone(), Tier::Global, None)))
        .collect())
}

/// Hợp nhất khoá của hai tầng rồi cho Work thắng **trên khoá trùng**.
///
/// Tách khỏi [`resolve_override`] để phép cưỡng chế ngữ nghĩa và phép hợp nhất đọc được
/// riêng — và để chỗ này không có đường nào trả về sớm với chỉ một tầng.
fn merge_by_key<K, V>(global: &BTreeMap<K, V>, work: Option<&BTreeMap<K, V>>) -> BTreeMap<K, Resolved<V>>
where
    K: Ord + Clone,
    V: Clone,
{
    // 🔴 TẦNG DƯỚI VÀO TRƯỚC, NGUYÊN VẸN. Đây là dòng phân biệt cài đặt đúng với cài
    // đặt làm 411 mục Glossary toàn cục biến mất — xem doc-comment của `resolve_override`.
    // Không có nhánh nào trả về sớm với chỉ một tầng, và đó là chủ ý.
    let mut out: BTreeMap<K, Resolved<V>> = global
        .iter()
        .map(|(k, v)| (k.clone(), Resolved::new(v.clone(), Tier::Global, None)))
        .collect();

    // Tầng trên ghi đè, và **chỉ trên khoá nó thật sự có**. Giá trị vừa bị đẩy ra không
    // biến mất — nó đi vào `shadowed`, thứ mà `settings.html:172` và
    // `glossary-manage.html:169` hiển thị.
    //
    // ⚠️ `global.get(k)` chứ không phải giá trị vừa lấy ra khỏi `out`: đọc từ nguồn giữ
    // cho `shadowed` luôn là giá trị TẦNG GLOBAL, kể cả nếu ai đó về sau chèn một tầng
    // thứ ba vào giữa.
    for (k, w) in work.into_iter().flatten() {
        out.insert(
            k.clone(),
            Resolved::new(w.clone(), Tier::Work, global.get(k).cloned()),
        );
    }

    out
}
