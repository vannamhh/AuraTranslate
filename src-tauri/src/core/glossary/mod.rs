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
//!
//! ─────────────────────────────────────────────────────────────────────────────
//! HÌNH DẠNG ĐÃ DỰNG (Story 3.5) — quét ứng viên khi nhập tài liệu, chỗ gọi sản phẩm ĐẦU
//! TIÊN của bảng chờ (Story 3.2 dựng bốn hàm, 0 chỗ gọi cho tới lượt này)
//! ─────────────────────────────────────────────────────────────────────────────
//! - [`scan`] — module LÁ, thuật toán THUẦN (`scan::scan_candidates_controlled`), không chạm DB, tiêm
//!   vị từ tra từ điển qua tham số (§Boundaries: "lọc tần suất trước, tra sau"). Sinh
//!   n-gram ký tự (`Zh`) hoặc cụm hoa liền nhau (`En`), phân xử "n-gram lồng", nới ngưỡng
//!   cho hình dạng giống họ người ([`surnames`]).
//! - [`surnames`] — mảng hằng *Bách gia tính*, dùng bởi [`scan`].
//! - [`candidate_store::insert_import_scan_candidates`] — hàm ghi LÔ mới, MỘT
//!   `Store::write`, lọc `glossary_entry` NGAY trong câu `INSERT` (`WHERE NOT EXISTS`),
//!   `ON CONFLICT (source_term) DO NOTHING`. **KHÔNG** vào `GLOSSARY_ONLY_SURFACE` —
//!   `commands::project` là chỗ gọi sản phẩm DUY NHẤT và nó nằm NGOÀI `core/glossary/**`
//!   (xem Spec Change Log của story 3-5 cho lý do đầy đủ, và vì sao `insert_candidate`
//!   singular thì có).
//! - `commands::glossary::glossary_pending_candidates` — vỏ IPC CHỈ-ĐỌC thứ năm, gọi
//!   [`pending_candidates`] — chỗ gọi sản phẩm ĐẦU TIÊN của hàm đó (Story 3.2 dựng, 0 chỗ
//!   gọi cho tới lượt này).
//!
//! ─────────────────────────────────────────────────────────────────────────────
//! HÌNH DẠNG ĐÃ DỰNG (Story 3.6) — trạng thái chờ chốt nay có đường CHỐT sản phẩm
//! ─────────────────────────────────────────────────────────────────────────────
//! - [`entry::GlossaryMark`] mang thêm `id`/`source_term` — chốt cần một KHOÁ GHI, và bề
//!   mặt tiếng Anh khớp theo hình thái (`dragons` trên màn hình, `dragon` trong Glossary)
//!   không cho phép suy khoá đó từ chuỗi đã cắt trên màn hình.
//! - [`store::confirm_pending_translation`] — khuôn chép [`store::add_manual_term`]: định
//!   tuyến `&Store` theo `tier` rồi gọi xuống [`store::confirm_translation`] (bị
//!   `GLOSSARY_ONLY_SURFACE` cấm gọi từ `commands/**`).
//! - `commands::glossary::glossary_confirm_pending_translation` +
//!   `commands::glossary::glossary_approve_candidate` — hai vỏ IPC mới, chỗ gọi sản phẩm
//!   ĐẦU TIÊN của [`store::confirm_translation`] (gián tiếp, qua hàm bọc trên) và
//!   [`candidate_store::approve_candidate`].
//!
//! ─────────────────────────────────────────────────────────────────────────────
//! HÌNH DẠNG ĐÃ DỰNG (Story 3.7) — đề xuất bản dịch bằng âm Hán Việt, FR113 (AD-36)
//! ─────────────────────────────────────────────────────────────────────────────
//! - [`han_viet_suggestion`] — module MỚI, `use crate::core::dict::{..}` THẬT (cạnh
//!   `glossary/ → dict/` mà AD-36 chỉ định) — **KHÁC** [`scan`] ngay bên dưới, module đó
//!   TIÊM một closure thay vì gọi thẳng để giữ thuật toán quét THUẦN; ở đây âm Hán Việt LÀ
//!   dữ liệu từ điển, không phải một quy tắc quét, nên đọc trực tiếp là đúng hình dạng. Xem
//!   doc-comment đầu `han_viet_suggestion.rs` cho lý do đầy đủ.
//! - [`han_viet_suggestion::HanVietSuggestion`] — `enum` NĂM nhánh (`Ready(String)` ·
//!   `NotChinese` · `NoReading` · `DictUnavailable` · `NotRequested`) thay cho một
//!   `Option<String>` trần — bốn lý do RỖNG phải phân biệt được trên dây (rỗng im lặng là
//!   lỗi trung tâm của dự án, `AGENTS.md:46`).
//! - [`han_viet_suggestion::suggest_han_viet_batch`] — gọi `crate::core::dict::
//!   lookup_han_viet` ĐÚNG MỘT LẦN cho cả LÔ thuật ngữ (dedupe ký tự trước khi tra); tính
//!   LÚC ĐỌC, **không** một cột `suggested_translation` trong `glossary_candidate`/
//!   `glossary_entry` (một bản chép dữ liệu từ điển là nhân bản dữ liệu, AD-36 cấm — xem
//!   §Design Notes của story cho lý do đầy đủ).
//! - [`entry::GlossaryMark`] mang thêm `han_viet_suggestion: Option<String>` +
//!   `han_viet_status: &'static str` — [`store::marks_for_source_text`] nay nhận thêm
//!   `layers: &DictLayers` + `disabled: &BTreeSet<String>`, gom `source_term` của các mục
//!   CHỜ CHỐT rồi gọi `suggest_han_viet_batch` một lần; mục ĐÃ CHỐT gán thẳng
//!   `HanVietSuggestion::NotRequested`, **0** lượt tra cho chúng.
//! - `commands::glossary::glossary_pending_candidates` — nay cũng nhận `layers`/`disabled`
//!   và gọi `suggest_han_viet_batch` trực tiếp cho tập `source_term` của các ứng viên đang
//!   chờ duyệt (bảng ứng viên không đi qua `marks_for_source_text`).
//!
//! ─────────────────────────────────────────────────────────────────────────────
//! HÌNH DẠNG ĐÃ DỰNG (Story 3.9) — quản lý Glossary: liệt kê cả hai tầng · xoá · đẩy tầng
//! ─────────────────────────────────────────────────────────────────────────────
//! - [`store::list_all_entries`] — HÀM PHƠI RA THỨ CHÍN của module này (cùng bề mặt
//!   `QUICK_ADD_SURFACE`/`GLOSSARY_ONLY_SURFACE` mà Story 3.3 dựng ra để thay
//!   `GLOSSARY_ONLY_SURFACE` cho `commands::glossary`). Khuôn chép
//!   `entries_eligible_for_injection` nhưng KHÔNG lọc `is_confirmed`, và phát cả
//!   `Resolved::shadowed()` thành một hàng thứ hai — đây là chỗ DUY NHẤT trong kho biết một
//!   mục Global có đang bị một mục Work cùng `source_term` che hay không.
//! - [`store::delete_manual_term`] — xoá `(tier, id)`, khuôn định tuyến `&Store` của
//!   [`store::add_manual_term`]. Xoá một mục ĐÃ CHỐT là hợp lệ (trigger một chiều chỉ khớp
//!   `UPDATE OF translation`, không bao giờ khớp `DELETE`).
//! - [`store::promote_to_global`] — đẩy một mục tầng Work lên tầng Global: `INSERT global`
//!   TRƯỚC, `DELETE work` SAU (hai kho không có giao dịch chung, và đây là thứ tự để một
//!   lượt sập giữa chừng để lại trạng thái DƯ chứ không THIẾU). Kiểm tra "đích đã có"
//!   TRƯỚC khi ghi (không bắt lỗi `UNIQUE` sau khi `INSERT` trượt) để trả
//!   [`store::GlossaryError::GlobalTermExists`] — một lỗi CÓ TÊN, không phải một
//!   `store.write_failed` chung.
//! - `commands::glossary::{glossary_list_entries, glossary_delete_term, glossary_promote_
//!   term_to_global}` — ba vỏ IPC mới, chỗ gọi sản phẩm ĐẦU TIÊN của cả ba hàm trên.
//!
//! ─────────────────────────────────────────────────────────────────────────────
//! HÌNH DẠNG ĐÃ DỰNG (Story 3.10) — xuất/nhập CSV/TSV, giá trị `term_origin` THỨ TƯ
//! ─────────────────────────────────────────────────────────────────────────────
//! - [`entry::TermOrigin::FileImport`] — giá trị thứ tư (`"file_import"`), tự đặt bởi
//!   [`store::import_into_tier`], không nhận qua tham số (cùng nguyên tắc FR55 mà
//!   `Manual`/`ImportScan`/`ReviewHarvest` đã giữ). Đòi một bước di trú DỰNG LẠI bảng (một
//!   `CHECK` không `ALTER` được) — hằng MỚI
//!   [`crate::core::store::GLOSSARY_ENTRY_ADD_FILE_IMPORT_ORIGIN_DDL`], bước 5 của
//!   `global.db` VÀ bước 15 của `project.db`, KHÔNG sửa [`crate::core::store::GLOSSARY_ENTRY_DDL`]
//!   tại chỗ.
//! - [`exchange`] — module MỚI, THUẦN (`&str` vào, `String` ra — không `rusqlite`, không
//!   `tauri`, không `std::fs`): [`exchange::render_tier`] (xuất một tầng, 6 cột, không cột
//!   `id`) · [`exchange::parse`] (phân tích TRỌN văn bản, trả MỌI [`exchange::ParseIssue`]
//!   tìm được, không dừng ở lỗi đầu tiên) · [`exchange::classify`] (so với tầng đích, ba
//!   nhánh *mới*/*giống*/*bất đồng*, hàm THUẦN). Một đường bọc nháy kép RFC 4180 DÙNG CHUNG
//!   cho CẢ hai dấu phân cách (CSV **và** TSV) là chỗ duy nhất giữ vòng tròn xuất→nhập khép
//!   kín.
//! - [`store::export_tier`] — gọi [`store::load_tier`] **một tầng** (KHÔNG
//!   [`store::list_all_entries`] — nó phát hàng bị che thành hàng thứ hai, sinh
//!   `source_term` trùng trong tệp).
//! - [`store::import_into_tier`] — **một** `store.write` cho TRỌN lô (§Always: "không ghi
//!   một phần"); `GlossaryError::ImportUniqueConflict` phát hiện SAU khi giao dịch đã
//!   rollback bằng cách nạp lại tầng, không phải một khoá đọc-trước-khi-ghi.
//! - ⚠️ **NỬA CHỌN TỆP (hộp thoại mở/lưu, `#[tauri::command]`) TÁCH KHỎI STORY NÀY** — đang
//!   chờ một `AD` (`ad-brief-2026-08-24-hop-thoai-chon-tep.md`). Story 3.10 giao đúng nửa
//!   định dạng + đường ghi; chỗ nối để lại là một hàm trả `PathBuf`.
//!
//! ─────────────────────────────────────────────────────────────────────────────
//! HÌNH DẠNG ĐÃ DỰNG (Story 3.10b, 2026-08-25) — hộp thoại chọn tệp nối vào, AD-48
//! ─────────────────────────────────────────────────────────────────────────────
//! - [`exchange_io`] — module MỚI, DUY NHẤT của cây này chạm `std::fs`: đọc tệp nhập
//!   (`metadata` ⇒ trần 16 MiB ⇒ `read` ⇒ `from_utf8` ⇒ cắt BOM, khuôn chép
//!   `core/segment/import.rs`) và ghi tệp xuất NGUYÊN TỬ (khuôn chép
//!   `core/library/meta.rs::write_atomic`). `exchange.rs` ở lại THUẦN — AC của Story
//!   3.10 vẫn đúng nguyên vẹn.
//! - `exchange.rs` — hai bản vá TẠI CHỖ: bước cắt DÒNG (`split_first_logical_line`) nay
//!   áp đúng luật "một `"` chỉ mở ô bọc khi đứng NGAY ĐẦU Ô" mà bước cắt Ô đã có, đóng
//!   `deferred-work.md:6776`; `seen`/kiểm `category` không còn `continue` sớm loại nhau,
//!   nên một hàng vừa trùng `source_term` vừa sai `category` báo CẢ HAI lỗi, đóng
//!   `deferred-work.md:6787`.
//! - `commands::glossary` — bốn vỏ IPC mới, gọi thẳng `export_tier`/`import_into_tier`
//!   (KHÔNG nằm trong `GLOSSARY_ONLY_SURFACE`, xem `glossary_boundary.rs`): xuất một
//!   tầng (một nhịp) · mở-và-xem-trước một lượt nhập (nhịp một, kế hoạch ở lại `State`
//!   Rust — AD-48 §Rule ①) · xác nhận lượt nhập (nhịp hai) · huỷ lô đang treo.
//! - `store::GlossaryError` — bảy biến thể mới cho các ca I/O của §I/O Matrix (tệp quá
//!   lớn · phi-UTF-8 · đọc/ghi thất bại · quyết định trỏ thuật ngữ lạ · không có lô
//!   treo · đường dẫn hộp thoại không quy đổi được).
//!
//! ─────────────────────────────────────────────────────────────────────────────
//! 🔵 CẬP NHẬT 2026-08-25 (vòng rà Epic 3, cụm B) — HAI MỆNH ĐỀ TRÊN HẾT ĐÚNG
//! ─────────────────────────────────────────────────────────────────────────────
//! ① Đường ĐỌC của [`exchange_io`] KHÔNG còn là `metadata ⇒ trần ⇒ read`. Hình dạng đó có
//!    một cửa sổ TOCTOU thật (tệp lớn lên GIỮA lúc đo và lúc đọc ⇒ trần bị bỏ qua và toàn
//!    bộ tệp vẫn bị nạp vào bộ nhớ). Nay là `File::open ⇒ take(LIMIT + 1) ⇒ read_to_end`,
//!    và quyết định "quá trần" dựa trên SỐ BYTE THẬT SỰ ĐÃ NẠP. Đường GHI vẫn là khuôn
//!    `write_atomic`, nhưng tên tệp tạm nay mang hậu tố `pid`+`uuid` — khuôn gốc ghi vào
//!    một đường NỘI BỘ CỐ ĐỊNH đã nối tiếp hoá qua `Store`, còn đây là đường NGƯỜI DÙNG
//!    CHỌN, nơi hai lượt xuất cùng đích va nhau được.
//! ② `exchange.rs` KHÔNG còn "hai bản vá TẠI CHỖ" — nay là MƯỜI. Ngoài hai bản vá kể trên,
//!    cụm B thêm: rào công thức CSV/TSV hai chiều · `UnterminatedQuotedField` có tên riêng
//!    thay vì nuốt mọi hàng sau vào một `CellCountMismatch` · đếm ranh giới dòng phủ cả `\r`
//!    trần · dò dấu phân cách trên ô ĐÃ TÁCH thay vì văn bản thô · `DuplicateColumn` cho
//!    cột trùng tên · cắt zero-width khỏi CẢ BA cột văn bản tự do · `seen` ghi nhận độc lập
//!    với `row_ok`.
//! ⇒ Số biến thể `ParseIssue` theo đó lên CHÍN (bảy + `UnterminatedQuotedField` +
//!    `DuplicateColumn`), và `core/i18n/mod.rs` mang đúng chín khoá phân tích tương ứng.

pub mod candidate;
pub mod candidate_store;
pub mod entry;
pub mod exchange;
pub mod exchange_io;
pub mod han_viet_suggestion;
pub mod scan;
pub mod store;
pub mod surnames;

pub use candidate::{CandidateOrigin, GlossaryCandidate, Resolution};
pub use candidate_store::{
    approve_candidate, insert_candidate, insert_import_scan_candidates, pending_candidates,
    reject_candidate,
};
pub use entry::{Category, GlossaryEntry, GlossaryMark, GlossaryTier, TermOrigin};
pub use exchange::{
    ConflictDecision, Delimiter, ImportRow, ImportSummary, ParseIssue, ParsedImport, RowPlan,
    RowPlanKind, classify, parse, render_tier,
};
pub use exchange_io::{MAX_GLOSSARY_IMPORT_BYTES, read_import_file, write_export_file};
pub use han_viet_suggestion::{HanVietSuggestion, suggest_han_viet_batch};
pub use scan::{DictionaryProbe, ScanCandidate, ScanOutcome, scan_candidates_controlled};

pub(crate) use candidate_store::{ImportScanWriteTicket, enqueue_import_scan_candidates};
pub(crate) use store::filter_import_scan_candidates_by_scope;
pub use store::{
    GlossaryError, add_manual_term, classify_import_rows, confirm_pending_translation,
    confirm_translation, delete_manual_term, entries_eligible_for_injection, export_tier,
    import_into_tier, insert_manual_entry, list_all_entries, load_tier, marks_for_source_text,
    match_lang_for_source_lang, promote_to_global, resolve_term_for_quick_add,
    update_manual_term, warm_jieba_for_source_lang,
};
pub use surnames::COMMON_SURNAMES;
