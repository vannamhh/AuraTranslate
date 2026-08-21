//! Glossary hai tầng + bảng chờ ứng viên TÁCH RIÊNG (AD-20, AD-36).
//!
//! Đề xuất tự động luôn vào bảng chờ, KHÔNG BAO GIỜ vào Glossary (AD-20).
//! Vòng đời ba trạng thái; chỉ trạng thái cuối được chèn vào prompt (AD-36).
//!
//! ─────────────────────────────────────────────────────────────────────────────
//! HÌNH DẠNG ĐÃ DỰNG (Story 3.1)
//! ─────────────────────────────────────────────────────────────────────────────
//! - [`entry`] — kiểu THUẦN: [`entry::GlossaryEntry`], [`entry::Category`],
//!   [`entry::TermOrigin`]. [`entry::GlossaryEntry::is_confirmed`] là vị từ DUY NHẤT định
//!   nghĩa "đã chốt" — không cột `status` song song (AD-36).
//! - [`store`] — SQL: `insert_manual_entry` · `confirm_translation` · `load_tier`, và
//!   **đúng MỘT** hàm phơi ra module khác, [`store::entries_eligible_for_injection`], lọc
//!   SAU khi phân giải qua `ScopeResolver::apply_override("glossary", ..)` (AD-18). Điều
//!   kiện chèn sống NGAY TRONG module này (AD-36) — cố ý lệch tiền lệ `core/segment/**`;
//!   tiền lệ đúng là `core/scope/store.rs`.
//! - Bảng riêng, `glossary_entry` — không một hàng `kind = 'glossary'` trong
//!   `config_value` (§Quyết định #1 của `core::store::schema`, `store.rs:283-291` từ chối
//!   đúng ca đó có chủ ý).
//!
//! ─────────────────────────────────────────────────────────────────────────────
//! HÌNH DẠNG ĐÃ DỰNG (Story 3.2) — bảng chờ ứng viên TÁCH HẲN khỏi Glossary
//! ─────────────────────────────────────────────────────────────────────────────
//! - [`candidate`] — kiểu THUẦN: [`candidate::GlossaryCandidate`],
//!   [`candidate::CandidateOrigin`] (**hai** biến thể — `Manual` không biểu diễn được, một
//!   mục nhập tay không đi qua bảng chờ), [`candidate::Resolution`].
//!   [`candidate::GlossaryCandidate::is_pending`] là vị từ DUY NHẤT định nghĩa "chờ duyệt"
//!   — `resolution IS NULL`, cùng khuôn cấu trúc mà Story 3.1 dùng cho `translation IS
//!   NULL`. [`candidate::CandidateOrigin::to_term_origin`] là ánh xạ TOÀN PHẦN sang
//!   [`entry::TermOrigin`] — chỗ DUY NHẤT trong kho sinh ra một `term_origin` khác
//!   `manual`.
//! - [`candidate_store`] — SQL: `insert_candidate` · `pending_candidates` ·
//!   `approve_candidate` (MỘT giao dịch `store.write`: đặt `resolution` VÀ chèn
//!   `glossary_entry`) · `reject_candidate`. `approve_candidate`/`reject_candidate` đọc
//!   `resolution` TRƯỚC khi ghi để phân biệt "id không tồn tại" với "ứng viên đã quyết" —
//!   xem doc-comment đầu tệp đó.
//! - `insert_entry` cũ (Story 3.1) đổi tên [`store::insert_manual_entry`] và MẤT tham số
//!   `term_origin: TermOrigin` — đường ghi phi-manual DUY NHẤT còn lại là
//!   `approve_candidate`, và nó không nhận `term_origin` từ chỗ gọi. Đây là vế CẤU TRÚC
//!   của FR55 ("không cơ chế nào được tự ghi vào Glossary"): trước lượt này, một module
//!   quét/thu hoạch tương lai chỉ cần gọi `insert_entry(.., TermOrigin::ImportScan)` là ghi
//!   thẳng, biên dịch sạch. Sau lượt này, chữ ký đó KHÔNG TỒN TẠI nữa.
//! - Bảng riêng, `glossary_candidate` — **chỉ ở `project.db`** (bước 13), KHÔNG có bước
//!   song sinh ở `GLOBAL_MIGRATIONS`: một ứng viên sinh ra từ việc quét một Tác phẩm cụ
//!   thể (§Never của story: "Bảng ứng viên ở `global.db`").
//!
//! ⚠️ **GIỚI HẠN THẬT — tầng Tác phẩm chưa đọc lại được sau khi khởi động lại ứng dụng.**
//! `ScopeResolver::with_work` chỉ được dựng ở `commands::project::create_work` — tức lúc
//! **TẠO MỚI** một `.atproj` trong phiên hiện tại. Hôm nay **không tồn tại đường mở lại**
//! một `.atproj` đã có trên đĩa (`OpenWorkState` khởi động luôn `None`, và không command
//! IPC nào ngoài `create_work_*` đặt được giá trị vào đó — `deferred-work.md:2465`). Hệ
//! quả cho Glossary: mục tầng Tác phẩm của một Tác phẩm đã đóng rồi mở lại **vẫn nằm
//! nguyên vẹn** trong `project.db` của nó — không mất dữ liệu — nhưng đường Rust để nạp
//! lại `ScopeResolver::with_work` cho phiên mới **chưa tồn tại**, nên
//! [`store::entries_eligible_for_injection`] không phân giải được tầng đó cho tới khi ai
//! đó mở Tác phẩm này lại. **Chủ: Epic 5** (đường mở lại `.atproj` — xem `deferred-work.md`
//! cho mục đóng đầy đủ).
//!
//! ─────────────────────────────────────────────────────────────────────────────
//! HÌNH DẠNG ĐÃ DỰNG (Story 3.3) — bề mặt IPC ĐẦU TIÊN của module này, FR48
//! ─────────────────────────────────────────────────────────────────────────────
//! - [`entry::GlossaryTier`] — `Global`/`Work`, kiểu RIÊNG (không tái dùng
//!   `core::scope::Tier`) vì nó đi tiếp một chặng mà `Tier` không đi: nó là dữ liệu TRÊN
//!   DÂY của `commands::glossary`. `id` của [`entry::GlossaryEntry`] chỉ duy nhất TRONG
//!   một `Store`, nên một `id` trần không đủ để sửa lại đúng hàng — mọi lượt tra/sửa mang
//!   theo cặp `(GlossaryTier, id)`.
//! - [`store::resolve_term_for_quick_add`] — tra hai tầng qua `ScopeResolver::apply_override`,
//!   **không lọc** `is_confirmed` (khác hẳn `entries_eligible_for_injection`, vốn tồn tại
//!   đúng để lọc) — một mục *chờ chốt* bị lọc mất sẽ làm dải "Thêm thuật ngữ" mở nhầm chế
//!   độ THÊM và `UNIQUE` chặn lượt lưu trong im lặng.
//! - [`store::add_manual_term`] / [`store::update_manual_term`] — chọn `&Store` theo
//!   `GlossaryTier` người dùng chọn rồi gọi xuống `insert_manual_entry`/một câu `UPDATE`
//!   trực tiếp; đường ghi phi-manual vẫn chỉ có một cửa (`approve_candidate`), hai hàm này
//!   không mở cửa thứ hai, chỉ định tuyến `&Store`.
//! - [`store::GlossaryError`] có thêm hai biến thể không mang dữ liệu:
//!   [`store::GlossaryError::EntryMissing`] (sửa một `id` đã biến mất) và
//!   [`store::GlossaryError::WorkTierUnavailable`] (chọn tầng Tác phẩm khi chưa mở Tác
//!   phẩm nào) — cộng `impl From<GlossaryError> for IpcError`, khuôn chép từ
//!   `core/store/mod.rs::impl From<StoreError> for IpcError`.
//! - `commands::glossary` (`src-tauri/src/commands/glossary.rs`) là chỗ ĐẦU TIÊN
//!   `OpenWork.scope` được đọc trong mã sản phẩm — trước story này nó chỉ được đặt ở
//!   `create_work` và không command nào khác đọc lại.
//!
//! ⚠️ **CÙNG GIỚI HẠN, ÁP THẲNG LÊN BẢNG CHỜ (Story 3.2).** `glossary_candidate` chỉ tồn
//! tại ở `project.db`, nên `approve_candidate`/`reject_candidate`/`insert_candidate`/
//! `pending_candidates` chỉ ghi/đọc được vào **tầng Tác phẩm** của Tác phẩm ĐANG MỞ trong
//! phiên hiện tại — hai kho (`global.db`/`project.db`) không có giao dịch chung, nên đẩy
//! một quyết định duyệt lên tầng Global không phải việc của story này (đó là Story 3.9,
//! "đẩy một mục từ Tác phẩm lên Global bằng một thao tác"). Không có `&Store` nào của
//! `project.db` ⇒ không gọi được bốn hàm này — cùng đúng giới hạn "chưa mở lại được" ở
//! trên, không phải một giới hạn thứ hai.
//!
//! ─────────────────────────────────────────────────────────────────────────────
//! HÌNH DẠNG ĐÃ DỰNG (Story 3.4) — HÀM PHƠI RA THỨ TƯ, khớp thuật ngữ theo ngôn ngữ
//! ─────────────────────────────────────────────────────────────────────────────
//! - [`entry::GlossaryMark`] — kiểu THUẦN: span ĐIỂM MÃ (`start`/`end`, quy đổi từ byte của
//!   `find_terms` MỘT LẦN trong [`store::marks_for_source_text`]) + `tier` + `is_confirmed`
//!   + `translation`. Không mang `source_term`/`id` — vẽ dấu chỉ cần bốn thứ này.
//! - [`store::marks_for_source_text`] — tra hai tầng qua `ScopeResolver::apply_override`
//!   (**không lọc** `is_confirmed`, cùng lý do [`store::resolve_term_for_quick_add`]), gọi
//!   [`crate::core::matching::find_terms`] (AD-17, không cài lại phép khớp) trên tập thuật
//!   ngữ đã phân giải, rồi phân xử span CHỒNG NHAU (`find_terms` trả chồng nhau được — dài
//!   nhất thắng, hoà thì trái nhất) TRƯỚC khi quy đổi byte → điểm mã.
//! - [`store::warm_jieba_for_source_lang`] — hâm `Jieba` NGOÀI đường gõ, gọi từ đường MỞ
//!   CHƯƠNG (`commands::chapter`), không từ thân `marks_for_source_text` — đóng
//!   `deferred-work.md:413` (179–329 ms khởi tạo lạnh, vượt trần NFR2 3,6–6,6×).
//! - `commands::glossary::glossary_marks_for_chapter` — hàm thuần thứ hai của
//!   `commands::glossary`: nhận `text`/`source_lang` làm THAM SỐ (không tự đọc `chapter`
//!   từ đĩa — frontend đã có `source_text` từ `read_open_chapter`), cùng khuôn
//!   `glossary_lookup_term` (`Option<&Store>` + `Option<&OpenWork>`, chưa mở Tác phẩm ⇒ chỉ
//!   khớp tầng Global, không lỗi).
//! - ⚠️ **NỬA GIAO DIỆN (vẽ dấu ở cột nguyên văn của lưới, dòng `StatusBar`) TÁCH KHỎI
//!   STORY NÀY** — xem `deferred-work.md` §"Deferred from: lượt lập spec Story 3.4", chủ:
//!   Ice, mở qua một lượt `correct-course`. Story 3.4 giao đúng NỬA RUST.

pub mod candidate;
pub mod candidate_store;
pub mod entry;
pub mod store;

pub use candidate::{CandidateOrigin, GlossaryCandidate, Resolution};
pub use candidate_store::{
    approve_candidate, insert_candidate, pending_candidates, reject_candidate,
};
pub use entry::{Category, GlossaryEntry, GlossaryMark, GlossaryTier, TermOrigin};
pub use store::{
    GlossaryError, add_manual_term, confirm_translation, entries_eligible_for_injection,
    insert_manual_entry, load_tier, marks_for_source_text, match_lang_for_source_lang,
    resolve_term_for_quick_add, update_manual_term, warm_jieba_for_source_lang,
};
