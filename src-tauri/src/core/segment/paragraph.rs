//! Ba ca biên của AD-37, áp cho **một CẶP cờ kết đoạn** (FR134, AD-46, Story 2.5d, AC3).
//!
//! ─────────────────────────────────────────────────────────────────────────────
//! 🔴 VÌ SAO MODULE NÀY TỒN TẠI HÔM NAY, KHI CHƯA BỀ MẶT NÀO GỌI NÓ
//! ─────────────────────────────────────────────────────────────────────────────
//! AC3 đòi: *"ba ca biên của AD-37 — gộp → theo câu cuối · tách → theo mảnh cuối, các mảnh
//! trước tắt · segment cuối Chương → tắt, luôn luôn — áp **y nguyên** cho cờ đích"*.
//!
//! **Hai trong ba ca không có bề mặt để áp** (đo 2026-08-15, và đo lại 2026-08-16):
//! `grep "fn merge_segments\|merge_segment\|MergeSegment"` trên `src-tauri/src/**` cho **0**
//! kết quả, và Story 2.8 *(gộp và tách segment tường minh)* là `backlog`. Hôm nay:
//!
//! | Ca | Có mã thi hành? | Ở đâu |
//! |---|---|---|
//! | Segment cuối Chương → tắt, luôn luôn | **Có** | [`super::split`], `mark_paragraph_end` |
//! | Tách → theo mảnh cuối | **Có, chỉ ở đường NHẬP** | [`super::split`] |
//! | Gộp → theo câu cuối | **Không** — mới là một bảng trong doc-comment | — |
//!
//! ⚠️ **Và có một khoảng hở thật, khác hẳn khoảng hở của [`super::omit`]:** chừng nào cờ đích
//! còn **bằng** cờ nguồn *(AC2, lúc nhập)*, ba ca biên đúng cho cờ đích **theo dẫn xuất** —
//! không cần một dòng mã thứ hai. Vế còn hở là ngày người dùng đã **đổi** cờ đích rồi mới
//! gộp hoặc tách: lúc đó hai cờ khác nhau, và bảng ba ca phải chạy **hai lần, độc lập**.
//! Người viết Story 2.8 đọc AC của chính nó, thấy *"cờ đi theo câu cuối"*, cài **một** lượt
//! cho `is_paragraph_end` — và cờ đích của người dùng biến mất. Không cổng nào đỏ.
//!
//! ⇒ **Quyết định #6 đường (b), Ice ký 2026-08-15:** dựng **hàm thuần** hôm nay kèm test hợp
//! đồng, đúng khuôn [`super::omit`] đã dựng cho một nghĩa vụ chưa có bề mặt tiêu thụ. Story
//! 2.8 chỉ việc gọi.
//!
//! 🔴 **Đây KHÔNG phải "dựng gộp/tách"**, và AC3 **vẫn không đóng trọn** ở story này: không
//! có đường gộp nào để nghiệm thu. Phần đó là 🟡, ghi nợ có chủ **Story 2.8** ở
//! `deferred-work.md`. Đừng đọc module này thành *"AC3 xong"*.
//!
//! ─────────────────────────────────────────────────────────────────────────────
//! 🔴 LOGIC Ở **RUST**, KHÔNG MỘT `v-if` Ở VUE — AD-1
//! ─────────────────────────────────────────────────────────────────────────────
//! Cấu trúc đoạn là **dữ liệu đã lưu** (AD-37, AD-46, và AC4 của story này nói thẳng:
//! *"đọc dữ liệu đã lưu, KHÔNG suy ra"*). Một phép tính lại ở webview là một nguồn sự thật
//! thứ hai, và nó rẽ khỏi đĩa đúng vào ngày người dùng đổi cờ đầu tiên.

/// Một cặp cờ kết đoạn của **một** segment: `(nguyên văn, bản dịch)`.
///
/// ⚠️ Một `struct` chứ không một `(bool, bool)` trần, và lý do là một lớp lỗi **đo được ở
/// mọi kho**: hai `bool` cạnh nhau không phân biệt được khi gọi, nên một lượt đảo tham số
/// biên dịch sạch, chạy sạch, và cho một kết quả sai **đối xứng** — thứ khó thấy nhất khi
/// đọc lại. Hai trường có tên thì một lượt đảo là một lỗi biên dịch.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ParagraphFlags {
    /// AD-37 — cờ của **nguyên văn** (`segment.is_paragraph_end`).
    pub source: bool,
    /// AD-46 — cờ của **bản dịch** (`segment.is_target_paragraph_end`), bước di trú 9.
    pub target: bool,
}

impl ParagraphFlags {
    /// Cặp cờ lúc một Chương **vừa nhập**: bản dịch **soi gương** bản gốc (AC2).
    ///
    /// ⚠️ Đây là chỗ DUY NHẤT trong Rust phát biểu phép soi gương thành một hàm có tên.
    /// `insert_segments` viết nó thẳng trong câu `INSERT` *(cùng một giá trị vào hai cột)*
    /// vì nó ở tầng SQL; mọi đường **logic** dùng hàm này để phép soi gương có đúng một tên.
    #[must_use]
    pub const fn mirrored(source: bool) -> Self {
        Self {
            source,
            target: source,
        }
    }
}

/// **Ca ① — segment CUỐI Chương: cả hai cờ TẮT, luôn luôn.** (AD-37, AC7 của Story 2.1)
///
/// 🔴 *"Luôn luôn"* nghĩa là **kể cả khi người dùng đã tự bật cờ đích**: một đoạn không thể
/// kết thúc sau câu cuối cùng, vì không có gì đứng sau nó để tách khỏi. Đây là ca biên duy
/// nhất **không** hỏi cờ cũ.
///
/// ⚠️ Vì sao nó nhận `ParagraphFlags` rồi vứt đi thay vì không nhận gì: chỗ gọi của Story
/// 2.8 sẽ áp **cùng một bảng** cho mọi segment của nhóm và chỉ phân nhánh theo **vị trí**.
/// Một hàm không nhận gì buộc chỗ gọi phải viết một `if` riêng cho ca cuối — tức bảng ba ca
/// bị tách làm hai chỗ.
#[must_use]
pub const fn at_end_of_chapter(_current: ParagraphFlags) -> ParagraphFlags {
    ParagraphFlags {
        source: false,
        target: false,
    }
}

/// **Ca ② — GỘP: cặp cờ đi theo CÂU CUỐI của nhóm gộp.** (AD-37; chủ thi hành: Story 2.8)
///
/// 🔴 **Hai cờ đi theo câu cuối MỘT CÁCH ĐỘC LẬP.** Đó là cả lý do hàm này tồn tại: cách
/// viết tự nhiên ở Story 2.8 là lấy `is_paragraph_end` của câu cuối rồi *"cờ đích chắc cũng
/// vậy"* — và lượt suy đó **xoá quyết định ngắt đoạn của người dùng** ở mọi câu mà hai cờ
/// đã khác nhau.
///
/// ⚠️ Nhóm rỗng ⇒ `None`. Một nhóm gộp rỗng là một lỗi của chỗ gọi, và trả một cặp cờ tắt
/// cho nó là **bịa ra một câu trả lời** cho một câu hỏi vô nghĩa — đúng lớp *"rỗng im lặng"*
/// mà `project-context.md` cấm.
#[must_use]
pub fn merged(group: &[ParagraphFlags]) -> Option<ParagraphFlags> {
    group.last().copied()
}

/// **Ca ③ — TÁCH: mảnh CUỐI giữ cặp cờ; mọi mảnh TRƯỚC nhận cả hai cờ TẮT.** (AD-37)
///
/// 🔴 Cùng lý do với [`merged`], và hỏng theo cách ngược lại: một lượt tách cài đúng cho cờ
/// nguồn mà để cờ đích **nguyên ở mọi mảnh** sẽ sinh ra `n` ranh giới đoạn từ chỗ trước đó
/// chỉ có một.
///
/// ⚠️ `pieces == 0` ⇒ danh sách rỗng. Cùng luật với [`merged`]: không bịa.
#[must_use]
pub fn split_into(current: ParagraphFlags, pieces: usize) -> Vec<ParagraphFlags> {
    if pieces == 0 {
        return Vec::new();
    }
    let mut out = vec![
        ParagraphFlags {
            source: false,
            target: false,
        };
        pieces
    ];
    // `pieces >= 1` da duoc bao dam o tren, nen chi so nay luon hop le.
    out[pieces - 1] = current;
    out
}
