//! Cổng **thứ nhất** trong đúng ba của AD-2 — một tệp `.db` từ điển nhìn qua một trait.
//!
//! ─────────────────────────────────────────────────────────────────────────────
//! 🔴 ĐƠN VỊ LÀ **MỘT TỆP**, ⛔ KHÔNG BAO GIỜ MỘT **NGÔN NGỮ**
//! ─────────────────────────────────────────────────────────────────────────────
//! AD-44 ⑤ nói thẳng: *"Cổng `DictionarySource`: ⛔ **Cấm** một adapter cho mỗi ngôn
//! ngữ"*. Hai hệ quả, và cả hai đều đắt hơn vẻ ngoài của chúng:
//!
//! 1. Một adapter theo **ngôn ngữ** phá mệnh đề *"gỡ một lớp = xoá một file"* của AD-10 —
//!    một lớp gỡ rời là **một tệp**, ⛔ không phải một ngôn ngữ, và `dict-core.db` mang
//!    **cả hai** ngôn ngữ trong cùng một tệp.
//! 2. Nó làm **FR36 ⛔ không nghiệm thu được bằng test thật nữa**: phép thử của FR36 là
//!    *xoá một tệp rồi chạy lại bộ test tra cứu*, và một adapter theo ngôn ngữ ⛔ không có
//!    tệp nào để xoá.
//!
//! `lang` đi qua đường này như một **TRƯỜNG** của bản ghi
//! ([`crate::core::dict::EntryHit::lang`]), ⛔ không phải một **KIỂU** — ⛔ không tồn tại
//! một bản ghi kết quả thứ hai dành riêng cho tiếng Anh (AD-44 ⑤).
//!
//! ─────────────────────────────────────────────────────────────────────────────
//! ⛔ KIỂU BẢN GHI SỐNG Ở `core::dict`, ⛔ KHÔNG Ở ĐÂY
//! ─────────────────────────────────────────────────────────────────────────────
//! [`crate::core::dict::LookupMode`] · [`QueryRoute`] · [`LookupResult`] có từ Story
//! 1.11/1.11b; [`SenseRecord`] · [`SourceInfo`] · [`QueryBranch`] thêm ở Story 1.13. Một
//! bản sao thứ hai trong `ports/` là **hai từ vựng cho một khái niệm**, và hai từ vựng sẽ
//! trôi khỏi nhau. Trait này **tham chiếu** chúng.
//!
//! ⛔ **Và tệp này ⛔ không gõ tên crate SQLite, ⛔ không `Connection::open`, ⛔ không chạm
//! filesystem.** Nó khai **hình dạng**, ⛔ không mang **cài đặt** — cài đặt là
//! [`crate::core::dict::DictLayer`], và đường mở tệp ở lại `core/store/**`
//! (`store_boundary.rs::only_core_store_may_name_rusqlite`).

use crate::core::dict::{HanVietHit, LookupResult, QueryBranch, QueryRoute, SenseRecord, SourceInfo};
use crate::core::store::StoreError;

/// Một tệp `.db` từ điển, nhìn qua cổng.
///
/// 🔴 **Hai pha, ⛔ không phải một** (Story 1.13 §Quyết định #1B). Lý do là một **số đo**,
/// ⛔ không phải một sở thích: nhánh `char_idx` một ký tự (`山`) trả **3.177** đầu mục với
/// p95 **7,324 ms** trên bản release — và đó là chi phí của **một** tệp, **⛔ chưa đọc một
/// hàng `dict_sense` nào**. Đọc nghĩa cho từng đó đầu mục × ba tệp ngay trong [`Self::lookup`]
/// vượt trần 10 ms một cách chắc chắn, và đường ra duy nhất là một `LIMIT` — tức cổng này
/// sẽ tự quyết một **chính sách sản phẩm** thuộc về Panel Lookup (Story 1.17).
///
/// ⇒ [`Self::lookup`] trả **đầu mục**; [`Self::senses`] đọc nghĩa cho một tập **do chỗ gọi
/// chọn**.
pub trait DictionarySource {
    /// `dict_meta('layer')` — `"base"` hoặc mã lớp gỡ rời. Đọc từ **CHÍNH tệp**.
    ///
    /// ⛔ **Không** suy từ tên tệp, và ⛔ **không** tra một sổ đăng ký: AD-44 ① vá A2 cấm
    /// một *"sổ tệp `.db` nào chứa gì"* vì nó là **nguồn sự thật thứ hai cho một dữ kiện
    /// đã nằm trong dữ liệu**, và nó sai **im lặng** vào đúng ngày một lớp được thêm hay
    /// gỡ đi (FR112).
    fn layer(&self) -> &str;

    /// Các nguồn tệp này mang. `dict-core.db` mang **sáu**; mỗi lớp gỡ rời mang một.
    ///
    /// 🔴 Khoá là [`SourceInfo::code`] (**chuỗi**), ⛔ không phải `id` (**số**): mỗi tệp có
    /// bảng `dict_source` RIÊNG, nên `id = 1` tồn tại ở cả ba tệp và trỏ ba nguồn khác nhau.
    fn sources(&self) -> &[SourceInfo];

    /// **Pha một** — tra một truy vấn, trả các đầu mục khớp.
    ///
    /// 🔴 `route` **nhận từ chỗ gọi** (AD-44 ①): adapter ⛔ không tự phân xử lại một câu
    /// hỏi thuộc về **CẢ LƯỢT TRA**. Để mỗi tệp tự tính là để hai tệp trả lời khác nhau
    /// ngay khi định nghĩa [`crate::core::dict::is_han`] của chúng lệch nhau.
    ///
    /// 🔴 `branch` **cũng nhận từ chỗ gọi** (Task 4.1 của Story 1.13), ⛔ **không** tính lại
    /// từ `mode` ở đây: tầng gom tính nó **ĐÚNG MỘT LẦN** cho cả lượt tra (qua
    /// [`crate::core::dict::pick_branch`]) và truyền cùng giá trị xuống **mọi** tệp. Để mỗi
    /// tệp tự tính lại là để chỉ còn một `debug_assert_eq!` — vô tác dụng ở bản release —
    /// giữ chúng khớp nhau.
    fn lookup(
        &self,
        query: &str,
        route: QueryRoute,
        branch: QueryBranch,
    ) -> Result<LookupResult, StoreError>;

    /// **Pha hai** — đọc nghĩa · ví dụ · trích dẫn cho một tập đầu mục **do chỗ gọi chọn**.
    ///
    /// 🔴 Đọc theo **LÔ**, ⛔ **không** một truy vấn cho mỗi đầu mục: 3.177 đầu mục × 3 tệp
    /// × 3 bảng là ba bậc độ lớn, và một cài đặt N+1 *"chạy đúng"* trên một fixture 20 hàng.
    ///
    /// ⚠️ `entry_ids` là `dict_entry.id` **của chính tệp này** — chúng chỉ duy nhất **trong
    /// một tệp `.db`**. Truyền id của tệp khác vào đây là đọc nhầm nghĩa mà ⛔ không lỗi
    /// nào được ném; đó là lý do pha hai đi **qua một lớp cụ thể** chứ ⛔ không qua tập lớp.
    fn senses(&self, entry_ids: &[i64]) -> Result<Vec<SenseRecord>, StoreError>;

    /// **Method thứ ba** — âm Hán Việt cho một **LÔ ký tự**. Story 1.16, Quyết định #2.
    ///
    /// 🔴 Đọc theo **LÔ**, ⛔ **không** một lời gọi cho mỗi ký tự — cùng luật [`Self::senses`],
    /// và cùng lý do: một Chương 3.000 ký tự ⇒ ~1.500 ký tự riêng × ba tệp là ba bậc độ lớn
    /// nếu gọi từng ký tự một.
    ///
    /// ⚠️ **⛔ Không** phải đường nóng NFR1 (Auto-Lookup, Story 1.18) — method này chạy
    /// **một lần cho mỗi lượt nạp Chương**, ⛔ không hàng trăm lần mỗi phiên.
    ///
    /// Trả về **các hàng khớp thô**, chưa tách nhiều âm và chưa chọn ưu tiên giữa các lớp
    /// — cả hai là việc của tầng gom ([`crate::core::dict::lookup_han_viet`]), ⛔ không
    /// phải việc của adapter (cùng doctrine `route`/`branch` của [`Self::lookup`]).
    fn han_viet(&self, chars: &[&str]) -> Result<Vec<HanVietHit>, StoreError>;
}
