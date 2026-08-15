//! Cắt bỏ câu khỏi bản dịch — **chốt lọc cho MỌI đầu ra** (FR133, Story 2.5c, AC5).
//!
//! ─────────────────────────────────────────────────────────────────────────────
//! 🔴 VÌ SAO MODULE NÀY TỒN TẠI HÔM NAY, KHI CHƯA BỀ MẶT NÀO GỌI NÓ
//! ─────────────────────────────────────────────────────────────────────────────
//! AC5 đòi: *"Chế độ đọc và **mọi** bản xuất — ẩn hoàn toàn, không dấu vết, không `[…]`,
//! không chỗ trống"*. **Cả hai bề mặt đó là khung rỗng** (đo 2026-08-15):
//! `core::export` có sáu dòng và **không một dòng mã**; `ReadingMode.vue` tự ghi *"khung
//! rỗng có chủ ý"*.
//!
//! ⚠️ Và có một khoảng hở đặc tả **thật**: nghĩa vụ *"ẩn hoàn toàn"* chỉ được phát biểu
//! **một chiều**, từ FR133 áp xuống. Các FR xuất bản *(FR87 · FR88 · FR89 · FR121 · FR130 ·
//! FR131)* và các Story của Epic 5 *(5.11–5.13)* / Epic 8 *(8.3 · 8.4 · 8.6)* — **không AC
//! nào tham chiếu ngược lại FR133**. Một nghĩa vụ chỉ có chiều đi xuống thì **không có ai
//! canh ở phía tiêu thụ**: người viết Story 8.3 đọc AC của chính nó, thấy đủ, và xuất ra một
//! tệp mang nguyên câu người dùng đã quyết định bỏ. Không cổng nào đỏ, không test nào đỏ —
//! đúng tiêu chí *"vi phạm được mà không cổng nào đỏ"*.
//!
//! ⇒ **Quyết định #2 đường (b), Ice ký 2026-08-15:** dựng **cái chốt** hôm nay, kèm test hợp
//! đồng, thay vì giao nghĩa vụ cho trí nhớ của người sau. Hai bề mặt kia chỉ việc gọi.
//!
//! 🔴 **Đây KHÔNG phải "dựng Chế độ đọc"**, và AC5 **vẫn không đóng trọn** ở story này: không
//! có bề mặt nào để nghiệm thu *"không dấu vết, không `[…]`, không chỗ trống"*. Phần đó là
//! 🟡, ghi nợ có chủ ở `deferred-work.md`. Đừng đọc module này thành *"AC5 xong"*.
//!
//! ─────────────────────────────────────────────────────────────────────────────
//! ⚠️ MỘT CÁI BẪY ĐỌC: LOẠI SUY XLIFF CHỈ ĐÚNG MỘT NỬA
//! ─────────────────────────────────────────────────────────────────────────────
//! `epics.md` và `EXPERIENCE.md:126` đều viết *"đúng khuôn `translate="no"` của XLIFF"*.
//! Trong XLIFF 2.0, `translate="no"` **khoá** một unit và **GIỮ NGUYÊN nội dung trong bản
//! xuất** — nó là *"đừng dịch cái này"*, không phải *"bỏ cái này đi"*. Loại suy đúng ở vế
//! *"trục độc lập, không phải một mức độ hoàn thành"*; **sai** ở vế hành vi đầu ra.
//! ⇒ Ai viết phần xuất bản: **đừng** chép ngữ nghĩa XLIFF vào đó. AC5 đòi ẩn, XLIFF thì giữ.
//!
//! ─────────────────────────────────────────────────────────────────────────────
//! 🔴 LỌC Ở **RUST**, KHÔNG Ở MỘT `v-if` RẢI RÁC TRONG VUE — AD-1
//! ─────────────────────────────────────────────────────────────────────────────
//! *"Frontend chỉ render và giữ state UI"*. Một `v-if="!s.is_omitted"` ở `ReadingMode.vue`
//! sẽ lo được **một** bề mặt và bỏ trống bề mặt xuất bản — và bề mặt xuất bản là chỗ hậu
//! quả **vĩnh viễn**: một tệp `.docx` đã gửi cho reviewer thì không rút lại được.

use crate::commands::segment::ChapterSegment;

/// Danh sách segment **thuộc bản dịch** — đã bỏ mọi câu người dùng cắt bỏ (FR133).
///
/// Đây là chốt mà **mọi** đường ra phải đi qua: Chế độ đọc (Epic 5) và mọi bản xuất
/// (Epic 8).
///
/// ⚠️ Nhận `&[ChapterSegment]` và trả `Vec<&ChapterSegment>` chứ không nhân bản dữ liệu:
/// Chương lớn nhất có thật là **9.850** câu, và một lượt `clone()` toàn bộ `source_text` +
/// `target_text` chỉ để bỏ vài hàng là một phép sao vô cớ trên đường nóng của bản xuất.
///
/// 🔴 **KHÔNG lọc `retired_at` ở đây, và đó là một quyết định.** Segment về hưu (AD-5) là một
/// khái niệm **khác**: nó là hàng đã bị gộp/tách thay thế, tức nó không nằm trong lượt nạp
/// của một Chương ngay từ đầu. Trộn hai phép lọc vào một hàm là dựng một hàm hai nghĩa, và
/// chỗ gọi sẽ không đọc được nó lọc gì. Đường lọc về hưu có chủ riêng: **Story 2.8**.
pub fn segments_in_translation(segments: &[ChapterSegment]) -> Vec<&ChapterSegment> {
    segments.iter().filter(|s| !s.is_omitted).collect()
}

/// Số câu **thuộc bản dịch** — cùng vị từ với [`segments_in_translation`], không một phép
/// đếm thứ hai.
///
/// ⚠️ Tồn tại vì chỗ gọi thường nhất của một bản xuất là *"có gì để xuất không"*, và cách
/// rẻ là viết `segments.iter().filter(|s| !s.is_omitted).count()` tại chỗ — tức **một bản
/// sao của vị từ**. Hai bản sao là hai chỗ phải sửa vào ngày vị từ đổi, và chúng sẽ lệch
/// nhau trong im lặng.
pub fn count_in_translation(segments: &[ChapterSegment]) -> usize {
    segments_in_translation(segments).len()
}
