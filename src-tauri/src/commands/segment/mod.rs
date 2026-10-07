//! Bề mặt IPC tách segment **tường minh** — Story 2.1, AC3 · AC8 · AC13 · AC14.
//!
//! Cùng khuôn `commands::chapter`/`commands::project`: hàm thuần trước, `#[tauri::command]`
//! chỉ là vỏ mỏng trong `wire`. Hàm thuần nhận `Option<&OpenWork>` — đây là thứ `tests/**`
//! gọi được **mà không cần webview**.
//!
//! ─────────────────────────────────────────────────────────────────────────────
//! 🔴 VÌ SAO MỘT LỆNH TƯỜNG MINH CHỨ KHÔNG MỘT BƯỚC DI TRÚ DỮ LIỆU — Quyết định #4
//! ─────────────────────────────────────────────────────────────────────────────
//! `deferred-work.md §*Deferred from: 1-12-matcher-dung-chung (2026-08-05)*` để ngỏ đúng hai đường cho **25 Chương Epic 1** đang mang
//! `segment_count = 0`: một thao tác tách tường minh, hoặc một bước di trú dữ liệu. Đường
//! thứ hai bị loại vì ba lý do độc lập:
//!
//! 1. Một bước di trú là **DDL**; chạy một quy tắc nghiệp vụ trong đó trộn hai tầng.
//! 2. Nó chạy **im lặng** lúc mở Tác phẩm — khó phân biệt với đúng cái *"đường tính ngầm
//!    lúc nạp Chương"* mà AC3 cấm bằng chữ.
//! 3. Bản sao lưu trước di trú **không nguyên tử và không xác minh lại**
//!    (`deferred-work.md §*Deferred from: 1-7-tang-ghi-du-lieu-mot-writer-noi-tiep-va-luoc-do-co-phien-ban (2026-08-04)*`, chưa ai vá), và đó sẽ là lượt di trú thật đầu tiên chạy trên
//!    một `project.db` **đã có dữ liệu người dùng**.
//!
//! ⇒ Bước di trú 5 chỉ làm **một việc**: `CREATE TABLE segment`. Chương **mới** nhập được
//! tách tự động trong `create_work` (cùng giao dịch — AC13); Chương **cũ** đi qua lệnh này,
//! một Chương một lượt.
//!
//! ⚠️ Lệnh **từ chối** một Chương đã có segment thay vì ghi đè. AD-4 đóng băng ranh giới
//! vĩnh viễn và AD-3 cấm tái dùng id đã về hưu — một lượt ghi đè im lặng là một lượt về hưu
//! im lặng, và lịch sử của Story 2.6 sẽ trỏ vào những id không ai biết đã mất.
//!
//! ⚠️ **AC8 vế hai** (*"không có đường nào tự động tách lại toàn bộ Thư viện"*) được giao ở
//! đây bằng cách **không tồn tại**, và `tests/segment_boundary.rs` khẳng định điều đó. Vế
//! một (nút tái tách kèm cảnh báo về dữ liệu sẽ về hưu) thuộc **Story 2.8** — hôm nay chưa
//! có `SegmentVersion` để mà giữ lại.
//!
//! ⚠️ Mọi chuỗi trong tệp này viết KHÔNG DẤU — `scripts/check-i18n.mjs` Kiểm A quét
//! `src-tauri/**/*.rs`.

use std::collections::{BTreeMap, BTreeSet};
use std::sync::{Arc, Mutex};

use crate::commands::project::OpenWork;
use crate::core::i18n::{IpcError, MessageKey};
use crate::core::lifecycle::LifecycleStatus;
use crate::core::segment::paragraph::{ParagraphFlags, at_end_of_chapter};
use crate::core::segment::regroup::{NewSegment, SegmentPart, merge, split_at};
use crate::core::segment::role::WovenSegment;
use crate::core::store::{ReadHandle, SqlError, SqlResult, Transaction};

/// Hai giá trị hợp lệ của `segment.status`, và **đúng hai** — Quyết định #5 (Ice ký
/// 2026-08-14).
///
/// ⚠️ Cưỡng chế ở **tầng Rust**, không bằng một `CHECK` trong DDL: cùng khuôn
/// `chapter.status` và `config_value.kind`, và thêm một `CHECK` ở một bảng mà hai bảng anh
/// em không có là dựng hai quy ước cho cùng một việc *(doc-comment `schema.rs`)*.
pub const SEGMENT_STATUS_DRAFT: &str = "draft";
/// Xem [`SEGMENT_STATUS_DRAFT`].
pub const SEGMENT_STATUS_CONFIRMED: &str = "confirmed";

/// Xuất xứ *"chưa có bản dịch"* — Quyết định #3 đường (b′) (Ice ký 2026-08-16).
///
/// 🔴 **Không** phải một giá trị thứ tư của FR117; nó là **sự vắng mặt** của một câu trả lời,
/// và nó ánh xạ sang *"không cặp TM nào được ghi"* trên trục nhị phân FR118 (AD-47 ⑥). Cùng
/// hình dạng và cùng lý do với `target_text = ""` — *"chưa dịch"* là một chuỗi **rỗng**, không
/// một giá trị **vắng mặt** *(doc-comment của `SEGMENT_TARGET_TEXT_DDL`)*.
pub const TRANSLATION_ORIGIN_NONE: &str = "";
/// FR117 *"tôi dịch"*. Trục nhị phân FR118: **của tôi**.
pub const TRANSLATION_ORIGIN_SELF: &str = "self";
/// FR117 *"người khác dịch"*. Trục nhị phân FR118: **của người khác**.
///
/// ⚠️ Đây cũng là giá trị mà AD-47 ③ giao cho ca **chấp nhận thay đổi từ Review Mode** (FR94,
/// Epic 8) và cho ca **bất đồng khi gộp/tách** (AD-47 ④, Story 2.8). Story 2.7 **không** cài
/// hai đường đó — nó chỉ khai giá trị chúng sẽ dùng, để hai Epic sau không tự đặt tên riêng.
pub const TRANSLATION_ORIGIN_OTHER: &str = "other";
/// FR117 *"nhập từ tài liệu song ngữ"* (FR115, Epic 6). Trục nhị phân FR118: **của người khác**.
///
/// ⚠️ Chưa đường mã nào **ghi** giá trị này hôm nay, và nó vẫn ở đây có chủ ý: nó là cái tên
/// mà Epic 6 (FR115) sẽ dùng, khai sẵn để Epic đó không tự đặt một tên riêng.
///
/// 🔵 **SỬA 2026-08-16 (code review) — bản trước khai một lớp bảo vệ KHÔNG TỒN TẠI.** Nguyên
/// văn nó viết: *"[`is_translation_origin`] phải nhận nó, nếu không một `.atproj` do một bản
/// tương lai ghi sẽ bị chính bản này gọi là hỏng"*. Không có `fn is_translation_origin` nào
/// trong kho — `grep` toàn cây trả về đúng dòng chú thích đó và không gì khác. ⇒ Câu ấy mô tả
/// một vế **đọc** chưa ai viết, và nó sai ngay từ lúc được gõ ra.
///
/// 🔴 **Vế đọc đó hôm nay KHÔNG có, và đây là hình dạng thật của khoảng hở:** [`TRANSLATION_ORIGINS`]
/// chỉ được một chỗ duy nhất đọc — ca `the_translation_origin_catalogue_matches_ad_47_row_by_row`
/// — nên nó canh **giá trị khai trong mã nguồn**, không canh **giá trị đi vào cột**. Đường đọc
/// sản phẩm (`SELECT ... translation_origin` trong [`confirm_segment`]) lấy thẳng ra `String`,
/// không đối chiếu danh mục. Một `.atproj` mang giá trị lạ đi qua sạch. Ghi nợ có chủ thay vì
/// dựng một hàm ở đây: một phép kiểm lúc chạy phải khai **nó làm gì khi gặp giá trị lạ**
/// *(từ chối mở? hạ về `''`? báo lỗi?)*, và đó là một quyết định lược đồ, không một dòng mã.
///
/// ⚠️ Và không cổng nào bắt được chính lỗi vừa sửa: kho **không** chạy `cargo doc` với
/// `rustdoc::broken_intra_doc_links` ở bất kỳ đâu trong 11 cổng, CI, hay `pre-push`.
pub const TRANSLATION_ORIGIN_BILINGUAL_IMPORT: &str = "bilingual_import";

/// Danh mục **ĐÓNG** của `segment.translation_origin` — AD-47 ⑥.
///
/// 🔴 Thêm một giá trị vào đây là một lượt **nới FR117**, và AD-47 ⑥ đặt hai điều kiện cho nó:
/// giá trị mới phải khai nó rơi về vế nào của **trục nhị phân FR118** *(của tôi / của người
/// khác)*, và vì tập giá trị nằm trên **đĩa người dùng** nên lượt nới là **một bước di trú
/// nữa**. Không lượt nào trong hai lượt đó là một dòng mã.
/// ⚠️ **Ai đọc mảng này, ghi ra vì "một hằng chỉ test đọc" là một hằng đáng ngờ:** nó là
/// **cổng** — `segment_contract.rs::the_translation_origin_catalogue_matches_ad_47_row_by_row`
/// khẳng định **đúng bốn** phần tử và **đúng bốn** cái tên đó. Một giá trị thứ năm lặng lẽ
/// thêm vào làm ca đó **đỏ**, và đó là đường duy nhất của kho bắt được một lượt nới danh mục
/// mà không ai viết `AD`. Cùng khuôn và cùng vai với `SEGMENT_RULE_VALUES` ↔ Kiểm I.
pub const TRANSLATION_ORIGINS: [&str; 4] = [
    TRANSLATION_ORIGIN_NONE,
    TRANSLATION_ORIGIN_SELF,
    TRANSLATION_ORIGIN_OTHER,
    TRANSLATION_ORIGIN_BILINGUAL_IMPORT,
];

// ═════════════════════════════════════════════════════════════════════════════════
// Story 4.8, Phase 2 — lượt PROMOTE một kết quả AI vào Editor (AD-47①, AD-47③, `⌘⇧↵`)
// ═════════════════════════════════════════════════════════════════════════════════

/// Một vỏ `#[tauri::command]`. **Không một quy tắc nào sống ở đây.**
pub mod wire;

mod chapter_read;
mod confirm;
mod history;
mod import;
mod reading;
mod regroup;
mod targets;
mod tm_match;
pub use chapter_read::*;
pub use confirm::*;
pub use history::*;
pub use import::*;
pub use reading::*;
pub use regroup::*;
pub use targets::*;
pub use tm_match::*;
