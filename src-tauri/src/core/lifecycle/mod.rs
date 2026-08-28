//! `LifecycleStatus` — bốn trạng thái vòng đời (FR5/FR6), khai MỘT CHỖ DUY NHẤT — Story 5.4.
//!
//! ─────────────────────────────────────────────────────────────────────────────
//! 🔴 VÌ SAO MỘT MACRO, KHÔNG PHẢI MỘT `enum` CỘNG MỘT `match` VIẾT TAY
//! ─────────────────────────────────────────────────────────────────────────────
//! Cùng lý lẽ đã đóng cho `message_keys!` (`core/i18n/mod.rs:62`) và `scope_kinds!`
//! (`core/scope/kinds.rs:96`): một `enum` cộng một hằng `ALL` viết tay là hai bản chép phải
//! khớp nhau bằng kỷ luật, và cách chúng trôi khỏi nhau đã biết trước — thêm một biến thể,
//! trình biên dịch bắt `match` thiếu nhánh ở `as_str()`/`label_key()`, người sửa thêm nhánh
//! **cho hết đỏ** chứ không kèm khoá nhãn `vi.json`. `lifecycle_statuses!` đóng đúng đường
//! đó: **không tồn tại cú pháp khai một giá trị mà không kèm khoá nhãn i18n** — khai tường
//! minh cả ba (định danh Rust, chuỗi trên dây, khoá nhãn) trong CÙNG một dòng.
//!
//! ─────────────────────────────────────────────────────────────────────────────
//! ⚠️ CHỈ BỐN GIÁ TRỊ — KHÔNG THÊM MỘT GIÁ TRỊ THỨ NĂM Ở ĐÂY
//! ─────────────────────────────────────────────────────────────────────────────
//! `5-4-bon-trang-thai-vong-doi.md` §Block If: một trạng thái thứ năm (vd. một trạng thái
//! *hỗn hợp* riêng) là quyết định của Ice, không phải một dòng mã tiện tay. Đọc
//! [`derive_work_status`] trước khi cám dỗ thêm bất cứ điều gì vào bảng này.
//!
//! ⚠️ Mọi chuỗi Ở VỊ TRÍ MÃ trong tệp này (giá trị trên dây, khoá nhãn) viết KHÔNG DẤU —
//! `scripts/check-i18n.mjs` Kiểm A quét `src-tauri/**/*.rs` tìm ký tự có dấu tiếng Việt ở vị
//! trí mã. Doc-comment thì có dấu thoải mái.

/// Khai MỘT CHỖ DUY NHẤT, sinh ra bốn thứ phải khớp nhau: `enum LifecycleStatus`,
/// `LifecycleStatus::ALL`, `as_str()` và `label_key()`, cộng `from_wire()`.
///
/// Cú pháp: `Variant => "gia_tri_tren_day" : "khoa.nhan"`. Khoá nhãn nằm **trong cùng khai
/// báo** với biến thể, nên không tồn tại đường khai một giá trị mà quên khoá nhãn.
///
/// ⚠️ `$(#[$meta:meta])*` không phải trang trí: doc-comment nở ra thành `#[doc = "…"]`, nên
/// một macro không khai chỗ nhận attribute sẽ từ chối biên dịch ngay khi ai đó viết một dòng
/// `///` cho biến thể đầu tiên — cùng khuôn `scope_kinds!`.
macro_rules! lifecycle_statuses {
    ($($(#[$meta:meta])* $variant:ident => $wire:literal : $label:literal),+ $(,)?) => {
        /// Bốn trạng thái vòng đời của FR5/FR6 — dùng chung cho tầng Chương VÀ tầng Tác
        /// phẩm (§Approach của story: "khai một chỗ duy nhất ... cho cả hai tầng dùng
        /// chung").
        #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
        pub enum LifecycleStatus {
            $($(#[$meta])* $variant),+
        }

        impl LifecycleStatus {
            /// Mọi biến thể. Sinh từ CÙNG khai báo với `as_str()`/`label_key()` nên không
            /// trôi được.
            ///
            /// 🔴 `lifecycle_contract.rs` đối chiếu độ dài này với một hằng **viết tay**
            /// `= 4` — con số viết tay là chỗ một con người phải ký, cùng lý lẽ
            /// `scope_kinds!::ALL`.
            pub const ALL: &'static [LifecycleStatus] = &[$(LifecycleStatus::$variant),+];

            /// Định danh máy đọc — thứ đi trên dây, thứ nằm ở `chapter.status`/
            /// `work.status_override`/`library_work.status`. Không phải nhãn hiển thị.
            pub const fn as_str(self) -> &'static str {
                match self {
                    $(LifecycleStatus::$variant => $wire),+
                }
            }

            /// Khoá tra trong `vi.json` cho nhãn HIỂN THỊ của trạng thái này.
            pub const fn label_key(self) -> &'static str {
                match self {
                    $(LifecycleStatus::$variant => $label),+
                }
            }

            /// Phân giải một giá trị đến **từ bên ngoài** (dây IPC, cột trên đĩa).
            ///
            /// ⚠️ Nhánh `_ => None` ở đây **không** vi phạm luật cấm `_ =>` trên
            /// `LifecycleStatus`: đây là `match` trên `&str` — một tập vô hạn, không tin
            /// được — nên nhánh cuối là bắt buộc và nó trả `None` chứ không đoán.
            pub fn from_wire(raw: &str) -> Option<LifecycleStatus> {
                match raw {
                    $($wire => Some(LifecycleStatus::$variant),)+
                    _ => None,
                }
            }
        }
    };
}

lifecycle_statuses! {
    /// Chưa Chương nào bắt đầu dịch — 0 Chương, hoặc mọi Chương đều `NotStarted`.
    NotStarted => "not_started" : "lifecycle.not_started",
    /// Ít nhất một Chương đang dở, và tập Chương không rơi vào ca `Done`/`NotStarted` toàn
    /// phần — xem [`derive_work_status`].
    InProgress => "in_progress" : "lifecycle.in_progress",
    /// **CHỈ đến từ ghi đè thủ công ở tầng Tác phẩm.** [`derive_work_status`] KHÔNG BAO GIỜ
    /// trả giá trị này — FR6 nguyên văn: *"Tạm ngưng ở tầng Tác phẩm là quyết định của
    /// người, hệ thống không suy ra được"*. Ở tầng Chương, giá trị này là một trạng thái
    /// bình thường người dùng tự đặt.
    Paused => "paused" : "lifecycle.paused",
    /// Mọi Chương đều `Done`.
    Done => "done" : "lifecycle.done",
}

/// Bảng suy ra trạng thái Tác phẩm từ tập trạng thái Chương — §Always của story 5.4, đúng
/// bốn hàng: 0 Chương ⇒ `NotStarted`; mọi Chương `Done` ⇒ `Done`; mọi Chương `NotStarted` ⇒
/// `NotStarted`; **mọi ca còn lại** ⇒ `InProgress`.
///
/// 🔴 **KHÔNG BAO GIỜ trả [`LifecycleStatus::Paused`].** Xem doc-comment của biến thể đó, và
/// §Design Notes của `5-4-bon-trang-thai-vong-doi.md` cho phương án bị loại (một Chương
/// `Paused` từng được cân nhắc để lan sang tầng Tác phẩm — bị loại vì nó làm `Paused` không
/// còn là bằng chứng tự thân của một quyết định con người).
pub fn derive_work_status(chapters: &[LifecycleStatus]) -> LifecycleStatus {
    if chapters.is_empty() {
        return LifecycleStatus::NotStarted;
    }
    if chapters.iter().all(|status| *status == LifecycleStatus::Done) {
        return LifecycleStatus::Done;
    }
    if chapters.iter().all(|status| *status == LifecycleStatus::NotStarted) {
        return LifecycleStatus::NotStarted;
    }
    LifecycleStatus::InProgress
}

#[cfg(test)]
mod tests {
    use super::{LifecycleStatus, derive_work_status};

    /// Hàng đầu của bảng: 0 Chương ⇒ `NotStarted` — một Tác phẩm chưa có Chương nào không
    /// được phép suy ra bất kỳ giá trị nào khác.
    #[test]
    fn zero_chapters_derive_not_started() {
        assert_eq!(derive_work_status(&[]), LifecycleStatus::NotStarted);
    }

    /// Mọi Chương `Done` ⇒ `Done`.
    #[test]
    fn every_chapter_done_derives_done() {
        let chapters = [LifecycleStatus::Done, LifecycleStatus::Done, LifecycleStatus::Done];
        assert_eq!(derive_work_status(&chapters), LifecycleStatus::Done);
    }

    /// Bất biến trung tâm của bảng: KHÔNG tổ hợp nào — kể cả một tổ hợp CHỨA `Paused` ở tầng
    /// Chương — được phép suy ra `Paused` ở tầng Tác phẩm. Một Chương `Paused` trộn với các
    /// trạng thái khác vẫn chỉ suy ra `InProgress`.
    #[test]
    fn no_combination_of_chapters_ever_derives_paused() {
        let combos: [&[LifecycleStatus]; 4] = [
            &[LifecycleStatus::Done, LifecycleStatus::NotStarted, LifecycleStatus::Paused],
            &[LifecycleStatus::Paused],
            &[LifecycleStatus::Paused, LifecycleStatus::Paused],
            &[LifecycleStatus::Done, LifecycleStatus::Paused],
        ];
        for combo in combos {
            assert_ne!(
                derive_work_status(combo),
                LifecycleStatus::Paused,
                "to hop {combo:?} khong duoc suy ra Paused"
            );
        }
    }
}
