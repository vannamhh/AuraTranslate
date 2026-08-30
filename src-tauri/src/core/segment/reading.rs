//! Gom segment **thuộc bản dịch** thành đoạn cho Chế độ đọc — Story 5.11, FR11.
//!
//! ─────────────────────────────────────────────────────────────────────────────
//! VÌ SAO HÀM NÀY Ở RUST, KHÔNG MỘT `computed` Ở VUE — cùng kỷ luật `paragraph.rs`
//! ─────────────────────────────────────────────────────────────────────────────
//! Cắt đoạn *trông như* việc render: duyệt danh sách, gặp `is_target_paragraph_end` thì
//! xuống đoạn. Ca biên phá cách đọc đó — đo được, không suy:
//!
//! ```text
//! câu 1  is_omitted=0  is_target_paragraph_end=0
//! câu 2  is_omitted=1  is_target_paragraph_end=1   ← người dùng đã CẮT BỎ đúng câu kết đoạn
//! câu 3  is_omitted=0  is_target_paragraph_end=0
//! ```
//!
//! Lọc trước rồi cắt sau ⇒ `[1, 3]` **một đoạn** — hai đoạn của người dùng nhập làm một,
//! im lặng. Cắt trước rồi lọc sau ⇒ đúng hai đoạn nhưng đoạn đầu phải mang cờ của một câu
//! **không còn tồn tại**. Quy tắc thật là *"cờ kết đoạn chuyển cho câu còn sống liền trước
//! trong cùng đoạn"*, và một quy tắc có ca biên thật thì không được sống ở webview (AD-1)
//! — đúng lý lẽ mà `core/segment/paragraph.rs` đã viết cho ba ca biên của AD-37.
//!
//! ─────────────────────────────────────────────────────────────────────────────
//! 🔴 CHỐT LỌC LÀ [`super::omit::segments_in_translation`], KHÔNG MỘT VỊ TỪ THỨ HAI
//! ─────────────────────────────────────────────────────────────────────────────
//! Doc-comment của `omit.rs` cấm thẳng một `v-if`/vị từ `!is_omitted` rải ở nơi khác:
//! *"chốt lọc phải có đúng một vị từ"*. Hàm dưới đây gọi lại chính chốt đó rồi ĐỐI CHIẾU
//! kết quả với dãy gốc theo con trỏ phần tử (`std::ptr::eq`) — không viết lại `!s.is_omitted`
//! một lần thứ hai ở tệp này.

use crate::commands::segment::ChapterSegment;

use super::omit;

/// Gom `segments` thành các đoạn **thuộc bản dịch** — Story 5.11.
///
/// Mỗi đoạn trả về là một dãy `&ChapterSegment` liên tiếp theo `(ord, id)`. Không đoạn
/// nào rỗng lọt ra ngoài: một đoạn mà mọi câu của nó đều bị cắt bỏ đơn giản không xuất
/// hiện trong kết quả, cùng khuôn *"không bịa"* của [`super::paragraph::merged`].
///
/// ⚠️ Nhận `&[ChapterSegment]`, trả `Vec<Vec<&ChapterSegment>>` — không nhân bản
/// `source_text`/`target_text`, cùng lý do hiệu năng mà [`omit::segments_in_translation`]
/// đã ghi cho Chương lớn nhất có thật (9.850 câu).
///
/// # Thuật toán
/// Duyệt `segments` theo đúng thứ tự đã lưu (một lượt, không chia hai lượt lọc-rồi-cắt):
/// - Câu **còn sống** (có mặt trong [`omit::segments_in_translation`]) được đẩy vào đoạn
///   đang gom; cờ `is_target_paragraph_end` của CHÍNH câu đó đóng đoạn lại.
/// - Câu **đã cắt bỏ** không được đẩy vào đâu cả — nó vắng mặt hoàn toàn. Nhưng nếu nó
///   mang cờ kết đoạn VÀ đoạn đang gom không rỗng, cờ ấy đóng đoạn đang gom ngay tại đó:
///   đây chính là phép *"chuyển cờ cho câu còn sống liền trước"*. Đoạn đang gom rỗng
///   (mọi câu trước đó trong đoạn cũng đã bị cắt bỏ, hoặc đoạn trước vừa đóng xong) thì
///   không có gì để đóng — đúng ca *"cả một đoạn bị cắt bỏ"*.
#[must_use]
pub fn paragraphs_in_translation(segments: &[ChapterSegment]) -> Vec<Vec<&ChapterSegment>> {
    // 🔴 Chốt lọc DUY NHẤT — xem doc-comment đầu tệp. `surviving` giữ THAM CHIẾU vào chính
    // các phần tử của `segments`, nên `std::ptr::eq` dưới đây so được từng phần tử một mà
    // không cần một vị từ `!is_omitted` thứ hai.
    let surviving = omit::segments_in_translation(segments);

    let mut result: Vec<Vec<&ChapterSegment>> = Vec::new();
    let mut current: Vec<&ChapterSegment> = Vec::new();
    let mut next_surviving = 0usize;

    for segment in segments {
        let is_surviving =
            next_surviving < surviving.len() && std::ptr::eq(surviving[next_surviving], segment);

        if is_surviving {
            next_surviving += 1;
            current.push(segment);
            if segment.is_target_paragraph_end {
                result.push(std::mem::take(&mut current));
            }
            continue;
        }

        // Câu đã CẮT BỎ — không đẩy vào đâu cả (AC5 · FR133: vắng mặt hoàn toàn, không
        // chỗ trống, không `[…]`). Cờ kết đoạn của nó, nếu có, chuyển cho câu còn sống
        // liền trước trong `current` bằng cách đóng đoạn ĐANG GOM ngay tại đây.
        if segment.is_target_paragraph_end && !current.is_empty() {
            result.push(std::mem::take(&mut current));
        }
    }

    // Đoạn cuối Chương không mang cờ kết đoạn (AD-37, ca biên "segment cuối Chương ⇒ tắt,
    // luôn luôn") nên nó không tự đóng trong vòng lặp — đóng nó ở đây nếu còn sót lại.
    if !current.is_empty() {
        result.push(current);
    }

    result
}
