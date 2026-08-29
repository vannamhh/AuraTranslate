//! DANH MỤC `message_key` mà Rust được phép phát ra (AD-21).
//!
//! **KHÔNG chứa văn bản hiển thị.** AD-21 nguyên văn: *"Rust không bao giờ trả về
//! văn bản hiển thị"*. Toàn bộ chuỗi giao diện sống ở `src/i18n/vi.json` và chỉ ở đó
//! (NFR16). Ở đây chỉ có khoá.
//!
//! Vì sao cần một danh mục tập trung: hình dạng lỗi qua IPC là
//! `{ code, message_key, params, retryable }`. Không có danh mục thì mỗi module tự
//! gõ khoá của mình, và một khoá gõ sai chỉ lộ ra khi người dùng gặp đúng lỗi đó —
//! frontend không phân giải được, hiện ra khoá trần hoặc chuỗi rỗng. Cùng hình dạng
//! hỏng im lặng mà `CommandRegistry` (AD-34) tồn tại để chặn.
//!
//! ─────────────────────────────────────────────────────────────────────────────
//! HÌNH DẠNG ĐÃ CHỐT (Story 1.5): **enum, khai qua một `macro_rules!`, KHÔNG sinh mã
//! lúc build.** Ba đường mà bản scaffold để ngỏ, và vì sao hai đường kia bị loại:
//!
//! - **Sinh mã từ `vi.json` lúc build** ⇒ thêm một `build.rs` và biến `vi.json` thành
//!   đầu vào biên dịch Rust. Nghĩa là sửa một dấu phẩy trong một chuỗi giao diện làm
//!   biên dịch lại nửa cây Rust, mỗi lần, trên cả hai nền tảng CI. Giá đó trả cho một
//!   thứ mà **một test đọc file lúc chạy** bắt được y hệt —
//!   `tests/ipc_contract.rs::every_message_key_exists_in_vi_json`.
//! - **Hằng viết tay** ⇒ `ALL` và `as_str()` là hai bản chép phải khớp nhau bằng kỷ
//!   luật, mà test đồng bộ với `vi.json` chạy TRÊN `ALL`. Thêm một biến thể rồi quên
//!   thêm vào `ALL` cho ra một test xanh giả và một khoá thiếu chỉ lộ ở tay người dùng.
//!
//! Macro giải đúng vế đó: **một khai báo, ba thứ sinh ra** (`enum` · `ALL` · `as_str`),
//! nên chúng không trôi khỏi nhau được.
//!
//! Bốn trường của `IpcError` là hợp đồng nguyên văn của AD-21 và bị
//! `tests/ipc_contract.rs` khoá lại — đọc doc-comment của struct trước khi đổi bất cứ
//! gì ở đó. Cổng `npm run check:i18n` canh nửa còn lại: không chuỗi hiển thị nào ở vị
//! trí mã, và mọi khoá của `MessageKey` có mặt trong `vi.json`.

use std::collections::BTreeMap;

use serde::{Serialize, Serializer};

/// Khai MỘT CHỖ DUY NHẤT, sinh ra ba thứ phải khớp nhau: `enum MessageKey`,
/// `MessageKey::ALL` và `MessageKey::as_str()`.
///
/// ⚠️ Vì sao macro chứ không viết tay hai chỗ: test `every_message_key_exists_in_vi_json`
/// chạy TRÊN `ALL`. Thêm một biến thể mà quên thêm vào `ALL` ⇒ test vẫn xanh, khoá vẫn
/// thiếu trong `vi.json`, và nó chỉ lộ ra ở tay người dùng thật. Một danh sách kiểm
/// tự rút gọn để cho xanh là đúng thứ phép kiểm tồn tại để chặn.
///
/// ⚠️ `$(#[$meta:meta])*` không phải trang trí: doc-comment nở ra thành `#[doc = "…"]`,
/// nên một macro không khai chỗ nhận attribute sẽ từ chối biên dịch ngay khi ai đó viết
/// một dòng `///` cho biến thể đầu tiên. Danh mục này tồn tại để được chú thích.
/// 🔴 Bảng THAM SỐ BẮT BUỘC nằm trong CÙNG khai báo, và đó là điểm của cả macro này.
///
/// Lỗ hổng nó đóng: `message_key` có kiểu nên một khoá ngoài danh mục không biên dịch
/// được — nhưng `params` là một `BTreeMap` tự do, và **không gì** nối nó với các
/// placeholder mà chuỗi trong `vi.json` đòi. Một chỗ gọi viết
/// `IpcError { message_key: MessageKey::IoReadFailed, params: BTreeMap::new(), .. }`
/// biên dịch sạch, qua `every_message_key_exists_in_vi_json` (khoá CÓ mặt), qua Kiểm C
/// của cổng (placeholder ĐÚNG hình dạng) — rồi người dùng đọc được nguyên văn
/// *"Không đọc được tệp tại {path} — nội dung chưa được nạp."*
///
/// Khai tham số cạnh khoá thì `required_params()` không trôi khỏi `as_str()` được, đúng
/// cùng lý lẽ đã chọn macro cho `ALL`. `check_message_key_params_match_vi_json` đối
/// chiếu bảng này với placeholder bóc từ `vi.json` theo **cả hai chiều**.
macro_rules! message_keys {
    ($($(#[$meta:meta])* $variant:ident => $key:literal [$($param:literal),* $(,)?]),+ $(,)?) => {
        /// Mọi khoá mà Rust được phép phát ra. Không mang văn bản hiển thị.
        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        pub enum MessageKey {
            $($(#[$meta])* $variant),+
        }

        impl MessageKey {
            /// Mọi biến thể. Sinh từ CÙNG khai báo với `as_str` nên không trôi được.
            pub const ALL: &'static [MessageKey] = &[$(MessageKey::$variant),+];

            /// Khoá chấm nguyên văn — thứ đi trên dây và thứ tra trong `vi.json`.
            pub const fn as_str(self) -> &'static str {
                match self {
                    $(MessageKey::$variant => $key),+
                }
            }

            /// Tên các tham số mà chuỗi của khoá này BẮT BUỘC phải nhận đủ.
            ///
            /// Nguồn duy nhất cho `IpcError::new` và cho phép kiểm đối chiếu với `vi.json`.
            pub const fn required_params(self) -> &'static [&'static str] {
                match self {
                    $(MessageKey::$variant => &[$($param),*]),+
                }
            }
        }
    };
}

// ─────────────────────────────────────────────────────────────────────────────
// BỘ KHOÁ MỒI — đúng hai, và đó là một quyết định, không phải một chỗ chưa làm xong.
//
// Story 1.5 sở hữu CƠ CHẾ, không sở hữu TỪ VỰNG. Mỗi story sau tự thêm khoá của nó
// cùng lúc với tính năng cần nó. Một danh mục 200 khoá dựng sẵn cho panel chưa ai
// dựng là 200 chuỗi không ai nghiệm thu được, và chúng sẽ sai.
// ─────────────────────────────────────────────────────────────────────────────
message_keys! {
    /// Khoá dự phòng cuối cùng của AD-21: mọi lỗi chưa phân loại được rơi vào đây
    /// thay vì rơi vào một chuỗi viết tay ở chỗ ném.
    ///
    /// ⚠️ KHÔNG tham số — và đó là điều kiện để nó làm được việc dự phòng: `IpcError::new`
    /// rơi về đây khi tham số thiếu, nên chuỗi của nó không được đòi tham số nào.
    Unknown => "err.unknown" [],
    /// Mang tham số `path`. Tồn tại để chứng minh đường nội suy tham số chạy thật
    /// từ Rust qua dây tới `createResolver` phía frontend.
    IoReadFailed => "err.io.read_failed" ["path"],

    // ─────────────────────────────────────────────────────────────────────────
    // TẦNG GHI DỮ LIỆU — Story 1.7 (AD-11 · AD-12 · AD-30)
    //
    // Năm khoá, và đúng năm: mỗi cái ứng với một cách `core::store::StoreError` hỏng
    // thật ở story này. Không khoá nào cho một tính năng chưa tồn tại — `project.db`
    // (Story 1.15) và `library-index.db` (Epic 5) sẽ tự mang khoá của chúng, cùng lúc
    // với mã cần chúng.
    //
    // ⚠️ `params` mang DỮ LIỆU: tên kho (`global`), phiên bản lược đồ, chế độ journal
    // đọc được về. Không mang câu — xem doc-comment của `IpcError`.
    // ─────────────────────────────────────────────────────────────────────────
    /// Database mang phiên bản lược đồ **mới hơn** bản ứng dụng đang chạy (AD-30, AC7).
    /// Ứng dụng từ chối mở và không ghi vào nó một byte nào.
    StoreSchemaTooNew => "err.store.schema_too_new" ["store", "found", "supported"],
    /// Không mở được kho — tệp hỏng, không có quyền, hoặc một bước di trú gãy.
    StoreOpenFailed => "err.store.open_failed" ["store"],
    /// `PRAGMA journal_mode = WAL` đọc lại ra chế độ khác. `mode` là chế độ ĐỌC ĐƯỢC VỀ.
    /// Không có WAL thì NFR2 và NFR18 mất bảo đảm, nên đây là lỗi chứ không phải cảnh báo.
    StoreWalUnavailable => "err.store.wal_unavailable" ["store", "mode"],
    /// Một job ghi trượt ⇒ giao dịch đã rollback, không có nửa ghi nào trên đĩa.
    StoreWriteFailed => "err.store.write_failed" ["store"],
    /// Một job đọc trượt.
    StoreReadFailed => "err.store.read_failed" ["store"],

    // ─────────────────────────────────────────────────────────────────────────
    // TẦNG TÁC PHẨM + ĐƯỜNG NHẬP — Story 1.15 (AD-9 · AD-33 · AD-39)
    //
    // Bốn khoá, và đúng bốn: `.docx`/bảng mã lạ bị từ chối TRƯỚC khi chạm đĩa
    // (AC8) là lỗi ĐƯỜNG NHẬP, không phải lỗi kho — `StoreError` không có biến
    // thể nào mô tả đúng "định dạng chưa nhận". `meta.json` mang số phiên bản
    // RIÊNG của chính nó (AC7), độc lập với `PRAGMA user_version` của `project.db`.
    // ─────────────────────────────────────────────────────────────────────────
    /// Định dạng tệp đưa vào chưa được nhận ở phiên bản hiện tại (`.docx`, v.v.) — AC8.
    /// `format` là phần mở rộng đọc được, dữ liệu chứ không phải câu.
    ImportUnsupportedFormat => "err.import.unsupported_format" ["format"],
    /// Nội dung tệp không giải mã được bằng UTF-8 — Quyết định #6, cùng khuôn với `.docx`.
    ImportNotUtf8 => "err.import.not_utf8" ["path"],
    /// Tệp đưa vào không có phần mở rộng nào — hạng RIÊNG, không phải
    /// `ImportUnsupportedFormat` với `format` rỗng (nó cho ra một câu vỡ). Code review
    /// 2026-08-06.
    ImportMissingExtension => "err.import.missing_extension" ["path"],
    /// Tệp vượt trần kích thước nhập (100 MB — Ice chốt 2026-08-06). `size`/`limit` là
    /// **số byte thô**: dữ liệu, không phải câu (AD-21).
    ImportTooLarge => "err.import.too_large" ["size", "limit"],
    /// Không dựng được `<Tên>.atproj/` trên đĩa (AC2, AC8).
    WorkCreateFailed => "err.work.create_failed" [],
    /// Đường đọc Chương gọi trước khi có Tác phẩm nào mở (Story 1.16, AC8) —
    /// `OpenWorkState` rỗng, không phải một lỗi kho.
    WorkNoneOpen => "err.work.none_open" [],

    // ─────────────────────────────────────────────────────────────────────────
    // TẦNG SEGMENT — Story 2.1 (AD-3 · AD-4 · AD-5 · AD-21 · AD-37)
    //
    // Hai khoá, và đúng hai: lệnh tách tường minh có đúng hai cách từ chối RIÊNG của
    // nó. *"Chưa mở Tác phẩm nào"* tái dùng `WorkNoneOpen` — cùng câu, cùng
    // nghĩa, và một khoá thứ hai cho nó là hai chuỗi phải giữ khớp nhau bằng kỷ luật.
    //
    // **KHÔNG** khoá nào cho "tái tách một Chương đã có segment": thao tác đó là
    // Story 2.8 (nó cần ngữ nghĩa VỀ HƯU của AD-5, mà hôm nay chưa có `SegmentVersion`
    // để giữ lại). Một khoá cho một tính năng chưa tồn tại là đúng thứ Story 1.7
    // §Completion Notes #3 cấm.
    // ─────────────────────────────────────────────────────────────────────────
    /// `chapter_id` đưa vào không có trong `project.db` của Tác phẩm đang mở.
    SegmentChapterNotFound => "err.segment.chapter_not_found" ["chapter_id"],
    /// Chương đã có segment ⇒ lệnh tách **từ chối**, không ghi đè. AD-4 đóng băng ranh
    /// giới vĩnh viễn; một lượt ghi đè im lặng là một lượt cho về hưu im lặng.
    /// `count` là số segment hiện có — dữ liệu, không phải câu.
    SegmentAlreadySplit => "err.segment.already_split" ["chapter_id", "count"],

    // ── Story 2.3 (AD-35 · AD-31 · FR100) — MỘT khoá, và đúng một ────────────────
    //
    // Lệnh ghi bản dịch tái dùng `SegmentChapterNotFound` cho ca *"Chương không thuộc
    // Tác phẩm đang mở"* và `WorkNoneOpen` cho ca *"chưa mở Tác phẩm nào"* — cùng
    // câu, cùng nghĩa, và một khoá thứ hai cho chúng là hai chuỗi phải giữ khớp bằng
    // kỷ luật. Chỉ ca dưới đây là RIÊNG của nó.
    //
    // **KHÔNG** khoá nào cho *"segment đã về hưu, không ghi được"*: `retired_at` chưa có
    // đường nào đặt (Story 2.8). Một khoá cho một nhánh không chỗ gọi nào đi qua là
    // đúng thứ Story 1.7 §Completion Notes #3 cấm.
    /// Một `segment.id` trong lô ghi không thuộc Chương được chỉ — lô bị **từ chối trọn**,
    /// không ghi một phần. `count` là số id lạ; `chapter_id` là Chương đã chỉ.
    ///
    /// 🔴 Vì sao từ chối trọn lô chứ không bỏ qua id lạ: một lô ghi **một phần** để lại
    /// đúng trạng thái mà không ai quan sát được — người dùng thấy chữ trên màn hình,
    /// đĩa giữ một phần, và không dấu hiệu nào báo. AD-35 nói flush *"chỉ được coi là
    /// xong sau khi đã ghi vào WAL"*; một lô nửa vời không thoả mệnh đề đó cho nửa còn lại.
    SegmentUnknownIds => "err.segment.unknown_ids" ["chapter_id", "count"],

    // ── Story 2.5 (AD-31 · AD-5 · FR24 · FR56 · FR101) — BA khoá, và đúng ba ────────
    //
    // Lệnh xác nhận tái dùng `WorkNoneOpen` cho ca *"chưa mở Tác phẩm nào"*. Ba ca
    // dưới đây là RIÊNG của nó, và AC14 đòi cả ba **phân biệt được**: *"không trả 'đã
    // xong' cho một lượt không ghi gì"*.
    //
    // ⚠️ `SegmentNotFound` KHÔNG gộp được vào `SegmentUnknownIds`: cái kia nói về một **lô**
    // và mang `chapter_id` + số id lạ; cái này nói về **một** segment được chỉ đích danh.
    // Mượn chung sẽ cho ra một câu nói về một lô mà người dùng vừa bấm một phím.
    /// `segment_id` không có trong `project.db` của Tác phẩm đang mở.
    SegmentNotFound => "err.segment.not_found" ["segment_id"],
    /// Segment đã **về hưu** (AD-5) ⇒ không xác nhận được. Một câu đã về hưu không còn là
    /// câu người dùng đang làm việc trên đó.
    ///
    /// 🔴 **HÀNG RÀO VIẾT TRƯỚC, và đó là một ngoại lệ CÓ CHỮ KÝ.** Luật của kho là *"không
    /// khoá nào cho một nhánh không chỗ gọi nào đi qua"* (Story 1.7 §CN #3), và hôm nay
    /// **chưa đường sản phẩm nào cho segment về hưu** — `retired_at` chỉ đặt được bằng SQL
    /// trực tiếp, chủ là **Story 2.8**. Ngoại lệ này do AC14 của Story 2.5 đòi bằng chữ, và
    /// nó có lưới thật: `segment_contract.rs::every_refusal_of_confirm_carries_its_own_...`
    /// dựng trạng thái về hưu bằng SQL trong fixture, nên nhánh **được chạy**, không chỉ
    /// được biên dịch.
    SegmentRetired => "err.segment.retired" ["segment_id"],
    /// Xác nhận một câu **chưa dịch** (`target_text` rỗng) ⇒ từ chối — Quyết định #7,
    /// Ice ký 2026-08-14.
    ///
    /// 🔴 Vì sao từ chối chứ không cho qua: một `SegmentVersion` mang chuỗi rỗng đi vào
    /// lịch sử FR101 *(người dùng khôi phục về "không có gì")*, và ở Epic 7 FR56 ghi **một
    /// cặp TM có vế đích rỗng**. Cặp đó sẽ khớp 100% ở một Chương sau rồi **điền sẵn một
    /// bản dịch rỗng** (FR58). Dữ liệu hỏng **vĩnh viễn** trong một kho dùng chung, sinh ra
    /// bởi một thao tác trông vô hại.
    SegmentNothingToConfirm => "err.segment.nothing_to_confirm" ["segment_id"],

    // ── Story 2.5d (AD-46 · AD-37 · FR134) — MỘT khoá, và đúng một ──────────────────
    //
    // Lệnh đổi cờ kết đoạn của bản dịch tái dùng `WorkNoneOpen` · `SegmentNotFound` ·
    // `SegmentRetired` cho ba ca đầu — cùng câu, cùng nghĩa. Chỉ ca dưới đây là RIÊNG.
    //
    // 🔴 **Vì sao một khoá THỨ BA chứ không mượn `SegmentRetired`** — code review
    // 2026-08-16, Ice ký đường (a). Hai khoá cũ nói *"câu không tồn tại"* và *"câu đã về
    // hưu"*; khoá này nói một sự thật **khác hẳn**: câu **tồn tại**, **còn sống**, và vẫn
    // không mang cờ được. Mượn một trong hai câu kia là nói dối về trạng thái của segment
    // — đúng thứ AC14 của Story 2.5 cấm khi nó đòi mọi lượt từ chối **phân biệt được**.
    /// Segment là câu **cuối Chương** ⇒ không đặt được cờ kết đoạn cho bản dịch — AC3 của
    /// Story 2.5d, ca ① của AD-37: *"segment cuối Chương → tắt, LUÔN LUÔN"*.
    ///
    /// 🔴 *"Luôn luôn"* nghĩa là **kể cả khi người dùng bấm phím tường minh**: một đoạn
    /// không thể kết thúc sau câu cuối cùng, vì không có gì đứng sau nó để tách khỏi.
    /// Ca này là ca biên duy nhất **không** hỏi cờ cũ — xem
    /// [`crate::core::segment::paragraph::at_end_of_chapter`], hàm thuần phát biểu nó.
    SegmentEndsChapter => "err.segment.ends_chapter" ["segment_id"],

    // ── Story 2.8 (FR78 · AD-5) — HAI khoá, và cùng một phép thử cho cả hai ─────────
    //
    // Lệnh gộp và lệnh tách tái dùng `WorkNoneOpen` · `SegmentNotFound` ·
    // `SegmentRetired` cho ba ca đầu — cùng câu, cùng nghĩa. Hai khoá dưới đây là RIÊNG vì
    // cả hai nói một sự thật **không khoá nào đang có nói được**: câu **tồn tại**, **còn
    // sống**, và thao tác vẫn không chạy được. Cùng phép thử đã dựng khoá thứ ba ở 2.5d.
    /// Gộp mà segment đang chọn là câu **ĐẦU** Chương ⇒ không có câu nào liền trên nó.
    ///
    /// ⚠️ Chữ ký #1(a) của Ice (2026-08-17) chốt *"gộp đúng hai — câu đang có caret và câu
    /// **liền trên** nó"*, nên *"không có câu liền trên"* là một ca **thường nhật**, không
    /// một ca biên: nó xảy ra mỗi lần người dùng bấm `⌘M` ở câu đầu Chương.
    SegmentNoPrevious => "err.segment.no_previous" ["segment_id"],
    /// Chỗ cắt để lại một mảnh **rỗng**, hoặc nằm ngoài `source_text`.
    ///
    /// 🔴 Một hàng `segment` không có văn bản nguồn là *"rỗng im lặng"* ở dạng tệ nhất —
    /// không đường mã nào phía sau biết xử lý nó, và nó nằm trên đĩa vĩnh viễn. Chặn ở
    /// tầng thuần ([`crate::core::segment::regroup::split_at`]) và nói ra ở đây.
    SegmentCutLeavesEmptyPiece => "err.segment.cut_leaves_empty_piece" ["segment_id"],

    // **KHÔNG có `WorkMetaTooNew` ở đây, và đó là một quyết định** (Ice, code review
    // 2026-08-06). Cơ chế từ chối một `meta.json` mới hơn vẫn còn nguyên và vẫn có test
    // (`MetaError::SchemaTooNew` + `WorkMeta::read`), nhưng không đường sản phẩm nào
    // gọi `WorkMeta::read` — story này không dựng màn hình "mở lại một `.atproj`".
    // Một khoá cho một tính năng chưa tồn tại là đúng thứ Story 1.7 §CN #3 cấm. 🔴 Story
    // nào dựng đường mở lại (ứng viên: Epic 5) thêm lại khoá này CÙNG LƯỢT với màn hình.

    // ── Story 3.3 (FR48 · AD-18 · AD-36) — BA khoá, và đúng ba ─────────────────────
    //
    // Bề mặt IPC ĐẦU TIÊN của `core/glossary/**` (`commands::glossary`, dải "Thêm thuật
    // ngữ"). Lệnh đọc/ghi tái dùng khoá kho hiện có (`StoreOpenFailed`/`StoreWriteFailed`/
    // `StoreReadFailed`) qua `impl From<GlossaryError> for IpcError` — ba khoá dưới đây chỉ
    // cho ba ca RIÊNG của module Glossary mà không khoá kho nào diễn đạt được.
    /// Sửa một mục Glossary bằng `(tier, id)` mà `id` không khớp hàng nào — mục đã bị xoá
    /// giữa chừng (đua với Story 3.9, hoặc một `id` cũ còn kẹt ở webview).
    GlossaryEntryMissing => "err.glossary.entry_missing" [],
    /// Chọn tầng Tác phẩm cho thêm/sửa một mục Glossary khi chưa có Tác phẩm nào đang mở.
    GlossaryWorkTierUnavailable => "err.glossary.work_tier_unavailable" [],
    /// `ScopeResolver::apply_override` từ chối bên trong `core::glossary::GlossaryError::
    /// Scope` — lỗi LẬP TRÌNH, không nên xảy ra trên đường gọi đúng (xem doc-comment của
    /// `core::scope::ScopeError` và `core::glossary::store::GlossaryError`).
    ///
    /// ⚠️ KHÔNG tham số, cùng lý do `Unknown`: `Display` của `ScopeError` là một câu chẩn
    /// đoán, và tham số `IpcError` phải mang DỮ LIỆU chứ không mang CÂU (AD-21).
    GlossaryScopeError => "err.glossary.scope_error" [],

    // ── Story 3.9 (FR49 · AD-18 · AD-36) — MỘT khoá, và đúng một ────────────────────
    //
    // Màn hình "Quản lý Glossary" tái dùng `GlossaryEntryMissing` (Xoá/Sửa một `id` đã biến
    // mất) và `WorkNoneOpen` (đẩy tầng khi chưa mở Tác phẩm nào). Chỉ ca dưới đây là
    // RIÊNG của thao tác "đẩy một mục lên tầng Toàn cục" — không khoá lỗi nào hiện có nói
    // đúng "đích đã có `source_term` này rồi, 0 lượt ghi".
    /// Đẩy một mục tầng Tác phẩm lên tầng Toàn cục mà `source_term` đã tồn tại sẵn ở
    /// `global.db` — **0 lượt ghi**, cả hai mục giữ nguyên. Không tham số: `source_term` là
    /// dữ liệu người dùng vừa gõ/thấy trên màn hình, không cần lặp lại qua `params`.
    GlossaryGlobalTermExists => "err.glossary.global_term_exists" [],

    // ── Story 3.10 (FR49 · NFR9) — MƯỜI khoá, và đúng mười ──────────────────────────
    //
    // 🔵 **CẬP NHẬT 2026-08-25 (vòng rà ba lớp, cụm B) — "TÁM khoá, và đúng tám" HẾT ĐÚNG.**
    // Hai biến thể `ParseIssue` mới (mục ②, ⑤ của cụm B) thêm hai khoá PHÂN TÍCH nữa: từ
    // bảy lên CHÍN. Tổng vẫn cộng thêm đúng một khoá GHI như trước ⇒ tám thành mười.
    //
    // Chín khoá đầu là lỗi PHÂN TÍCH (`core::glossary::exchange::ParseIssue`, trước khi có
    // Store nào được chạm tới) — mỗi cái ứng với đúng một hàng "0 lượt ghi" của §I/O Matrix.
    // Khoá cuối là lỗi GHI (`core::glossary::store::GlossaryError::ImportUniqueConflict`) —
    // hàng "Va UNIQUE giữa chừng". `params` mang DỮ LIỆU (số dòng, tên cột, giá trị đọc
    // được), không mang CÂU (AD-21).
    //
    // 🔴 Cặp `ParseIssue` (chín biến thể) ↔ chín khoá đầu ở đây ĐƯỢC CANH bằng
    // `glossary_exchange_contract.rs::every_parse_issue_variant_maps_to_a_declared_message_key`
    // (P7, vòng rà ba lớp 2026-08-25) — `ipc_contract.rs` chỉ canh `MessageKey` ↔ `vi.json`,
    // KHÔNG canh `ParseIssue` ↔ `MessageKey`; ba danh sách lệch nhau không cổng nào đỏ trước
    // khi cổng đó có mặt.
    /// Hàng tiêu đề chứa CẢ hai dấu phân cách (`,` và TAB) hoặc KHÔNG cái nào.
    GlossaryImportDelimiterUnresolved => "err.glossary.import_delimiter_unresolved" [],
    /// Hàng tiêu đề thiếu cột bắt buộc. `column` là tên cột — dữ liệu, không phải câu.
    GlossaryImportMissingColumn => "err.glossary.import_missing_column" ["column"],
    /// Một hàng dữ liệu có số ô khác số cột của hàng tiêu đề.
    GlossaryImportCellCountMismatch =>
        "err.glossary.import_cell_count_mismatch" ["line", "expected", "found"],
    /// `category` của một hàng không khớp bốn giá trị đã biết.
    GlossaryImportUnknownCategory => "err.glossary.import_unknown_category" ["line", "value"],
    /// `source_term` của một hàng rỗng hoặc chỉ toàn khoảng trắng sau khi cắt.
    GlossaryImportBlankSourceTerm => "err.glossary.import_blank_source_term" ["line"],
    /// Cùng `source_term` xuất hiện ở hai hàng dữ liệu trong CHÍNH tệp — không "dòng sau
    /// thắng" im lặng.
    GlossaryImportDuplicateSourceTerm =>
        "err.glossary.import_duplicate_source_term" ["first_line", "second_line"],
    /// 🔵 **THÊM 2026-08-25 (vòng rà ba lớp, P3).** Cột `created_at` có mặt, không rỗng,
    /// nhưng không khớp hình dạng ISO-8601 UTC — cột này không phải văn bản tự do.
    GlossaryImportInvalidCreatedAt => "err.glossary.import_invalid_created_at" ["line", "value"],
    /// 🔵 **THÊM 2026-08-25 (vòng rà ba lớp, cụm B, mục ②).** Một ô mở dấu ngoặc kép nhưng
    /// không bao giờ đóng — trước bản vá, phần còn lại của tệp bị nuốt vào MỘT hàng, và lỗi
    /// nổi lên là `CellCountMismatch` ở dòng CUỐI tệp, trỏ sai chỗ.
    GlossaryImportUnterminatedQuotedField =>
        "err.glossary.import_unterminated_quoted_field" ["line"],
    /// 🔵 **THÊM 2026-08-25 (vòng rà ba lớp, cụm B, mục ⑤).** Hàng tiêu đề mang hai cột
    /// cùng một tên ĐÃ BIẾT — trước bản vá, cột thứ hai (và giá trị của nó) mất im lặng.
    GlossaryImportDuplicateColumn => "err.glossary.import_duplicate_column" ["column"],
    /// Một hàng phân loại *mới* lúc `classify()` nhưng `source_term` đã bị một lượt ghi
    /// KHÁC chèn vào tầng đích trước khi giao dịch nhập kịp mở — giao dịch rollback trọn.
    /// 🔵 **SỬA 2026-08-25 (P6)** — `value` nay có thể mang NHIỀU thuật ngữ, nối bằng `", "`
    /// (xem `GlossaryError::ImportUniqueConflict::source_terms`, một `Vec` chứ không một
    /// `String` — hàm ý chỗ gọi TƯƠNG LAI dựng UI tách lại bằng `", "` nếu cần liệt riêng
    /// từng thuật ngữ; hôm nay chưa vỏ IPC nào tiêu thụ khoá này).
    GlossaryImportUniqueConflict => "err.glossary.import_unique_conflict" ["value"],

    // ── Story 3.10b (AD-48 · FR49/NFR9) — BỐN khoá MỚI, ba tái dùng ─────────────────
    //
    // Hộp thoại chọn tệp nối vào xuất/nhập Glossary. Ba ca I/O mượn khoá CHUNG với
    // `core::segment::import` vì câu ĐÚNG là câu chung, không câu riêng của Glossary:
    // `ImportTooLarge` (["size","limit"], `core::glossary::store::GlossaryError::
    // ImportFileTooLarge` dùng trần 16 MiB thay vì 100 MiB — cùng khoá, hai con số
    // khác), `ImportNotUtf8` (["path"], `GlossaryError::ImportNotUtf8`), `IoReadFailed`
    // (["path"], `GlossaryError::ImportReadFailed` — lỗi mở/đọc tệp KHÁC hai ca trên,
    // ví dụ quyền truy cập). Bốn khoá dưới đây RIÊNG vì không khoá nào hiện có nói đúng
    // bốn sự thật này.
    /// Ghi tệp xuất thất bại (hệ điều hành từ chối, hết dung lượng, …) — §Ask First của
    /// spec: "ghi nguyên tử bị hệ điều hành từ chối ở một thư mục người dùng chọn".
    /// `path` là đường dẫn người dùng vừa chọn — dữ liệu, không phải câu.
    GlossaryExportWriteFailed => "err.glossary.export_write_failed" ["path"],
    /// `FilePath::into_path()` của `tauri-plugin-dialog` trả lỗi (`InvalidPathUrl`) —
    /// hộp thoại trả về một giá trị không quy đổi được thành `PathBuf`. Không tham số:
    /// không có dữ liệu người dùng đọc được nào để mang theo, chỉ một chẩn đoán nội bộ.
    GlossaryDialogPathInvalid => "err.glossary.dialog_path_invalid" [],
    /// Bản đồ quyết định (nhịp hai) mang một khoá `source_term` KHÔNG có trong lô đã
    /// phân tích ở nhịp một — §Always: "một quyết định trỏ tới `source_term` không có
    /// trong lô là một lỗi tường minh". `value` là thuật ngữ lạ đó.
    GlossaryImportDecisionUnknownTerm => "err.glossary.import_decision_unknown_term" ["value"],
    /// Xác nhận lượt nhập (nhịp hai) khi chưa qua nhịp một, hoặc lô đã bị dọn (huỷ, đổi
    /// tệp, đóng Tác phẩm ở tầng Work) — §I/O Matrix "Xác nhận khi không có lô nào".
    GlossaryNoPendingImport => "err.glossary.no_pending_import" [],

    // ── Cụm C (`spec-epic-3-review-cum-c-dong-thoi-duong-commit-nhap.md`, C1) — MỘT khoá ────
    //
    // `TakeTheirs` của `import_into_tier` nay so LẠC QUAN với giá trị người dùng đã thấy ở
    // nhịp preview; một lượt ghi khác chen vào giữa hai nhịp làm phép so đó trượt.
    /// Bản dịch của một hàng *bất đồng* đã bị đổi ở nơi khác trong lúc người dùng xem trước
    /// (hoặc, cho một mục *chờ chốt*, đã bị người khác chốt) — §I/O Matrix ①/③b. `value` là
    /// mọi thuật ngữ va, nối bằng `", "` — cùng hình dạng `GlossaryImportUniqueConflict`.
    GlossaryImportStaleConflict => "err.glossary.import_stale_conflict" ["value"],

    // ── Story 5.3 (AD-8 · FR99) — ĐÚNG HAI khoá ─────────────────────────────────────
    //
    // Bề mặt IPC "Quét lại thư mục". Danh mục ĐÓNG: `IndexError::Io` tái dùng
    // `IoReadFailed` thay vì đúc một khoá thứ ba (xem `commands::library`/
    // `core::library::indexer::IndexError`), và `Indexer` chưa được quản lý tái dùng
    // `StoreOpenFailed` (params `{"store": "library_index"}`) thay vì một khoá thứ ba nữa —
    // `commands/library.rs` KHÔNG được nhắc `StoreKind::LibraryIndex` (cổng ranh giới
    // `library_index_boundary.rs`), nên nó dựng `IpcError` bằng khoá CHUNG này, không bằng
    // một `StoreError::OpenFailed { store: StoreKind::LibraryIndex, .. }` thật.
    /// [`crate::core::library::indexer::Indexer::forget_orphan`] gọi trên một `work_id`
    /// không tồn tại, hoặc tồn tại nhưng đang SỐNG (`orphaned = 0`) — cùng một câu cho cả
    /// hai ca (§I/O Matrix: không im lặng thành công, không mập mờ giữa hai lý do).
    ///
    /// 🔵 **THÊM tham số `name` (2026-08-27, vòng rà THỨ HAI P9).** `work_id` một mình là
    /// một UUID trần — không phải thứ người dùng NHẬN RA. `name` của mục mồ côi đã có sẵn
    /// ở CHỖ GỌI (`LibraryMode.vue` đang hiển thị nó ngay lúc người dùng bấm "Gỡ khỏi chỉ
    /// mục"), nên `commands::library::forget_orphan` nhận nó qua tham số và tự dựng
    /// `IpcError` cho ca này thay vì đi qua `From<IndexError>` chung (`Indexer` không biết
    /// "tên mà người dùng đang thấy" — đó là dữ liệu của TẦNG GỌI, không phải của chỉ mục).
    LibraryNotOrphaned => "err.library.not_orphaned" ["work_id", "name"],
    /// Thư mục gốc người dùng chọn qua hộp thoại không dùng được (ví dụ: một tệp thường,
    /// không phải thư mục). Không tham số: đường dẫn cụ thể không phải dữ liệu cần thiết cho
    /// câu này, và hộp thoại chọn thư mục của hệ điều hành đã tự lọc phần lớn ca sai hình
    /// dạng trước khi trả về.
    LibraryRootInvalid => "err.library.root_invalid" [],
    // ── Story 5.9 (FR8) — KHÔNG một khoá mới ─────────────────────────────────────────
    //
    // Bề mặt IPC "Tìm kiếm full-text xuyên Library". Danh mục ĐÓNG: `commands::library::search_library`
    // gọi trên `indexer = None` ("chưa mở chỉ mục") tái dùng ĐÚNG khoá `StoreOpenFailed` mà
    // `list_works`/`rescan` đã dùng (qua `indexer_is_missing()`, params `{"store":
    // "library_index"}`) — không một biến thể thứ ba cho cùng một sự thật "Indexer chưa được
    // quản lý". Bốn ca RỖNG còn lại của §I/O Matrix story này (chưa gõ gì · đang tìm · chỉ mục
    // chưa có dòng nào · truy vấn dưới 3 ký tự · có dòng mà không khớp) KHÔNG một khoá nào ở
    // đây: chúng là TRẠNG THÁI trong `SearchReport` (`indexed_segments`/`short_query`/`hits`
    // rỗng), không phải lỗi — mọi câu hiển thị của chúng sống trong `mode.library.search_*`
    // (`vi.json`), phía Rust chỉ trả DỮ LIỆU (AD-21).
    /// **THÊM (2026-08-28, Story 5.6)** — khoá sắp xếp ngoài danh mục hai giá trị đóng của
    /// [`crate::core::library::indexer::WorkSortKey`] — đi qua `commands::library::list_works`
    /// trên dây (`sort`). Không một lượt SQL nào chạy trước khi lỗi này được trả (§Always: "một
    /// khoá lạ trên dây ⇒ IpcError, không im lặng rơi về mặc định").
    LibraryUnknownSort => "err.library.unknown_sort" ["sort"],

    // ── Story 5.4 (FR5/FR6) — ĐÚNG MỘT khoá ─────────────────────────────────────────
    //
    // Bề mặt IPC "Bốn trạng thái vòng đời". Danh mục ĐÓNG: ca "chưa Tác phẩm nào mở" tái
    // dùng `WorkNoneOpen`, ca "chapter_id không tồn tại" tái dùng `SegmentChapterNotFound`
    // — cả hai đã có ở trên, không đúc khoá thứ hai/ba cho cùng câu.
    /// Giá trị trạng thái vòng đời ngoài danh mục bốn giá trị đóng của
    /// [`crate::core::lifecycle::LifecycleStatus`] — đi qua `set_chapter_status` hoặc
    /// `set_work_status_override`. Không một lượt ghi nào chạy trước khi lỗi này được trả.
    LifecycleUnknownStatus => "err.lifecycle.unknown_status" ["status"],

    // ── Story 5.7 (FR12) — mở lại `.atproj` + danh sách Chương ───────────────────────
    //
    // Bề mặt IPC "Mở Tác phẩm" + "Danh sách Chương" + "Mở Chương". Danh mục ĐÓNG: ca
    // "chưa Tác phẩm nào mở" tái dùng `WorkNoneOpen`, ca "chapter_id không tồn tại" tái
    // dùng `SegmentChapterNotFound` — cả hai đã có ở trên, không đúc khoá thứ hai/ba cho
    // cùng câu. `store.schema_too_new` (đã có, `StoreSchemaTooNew`) phủ nhánh `project.db`
    // mới hơn ứng dụng; ba khoá dưới đây phủ phần còn lại.
    /// `meta.json` của một `.atproj` đang mở lại mang `meta_schema_version` mới hơn bản
    /// ứng dụng hiểu — [`crate::core::library::WorkError::MetaTooNew`]. Không một byte nào
    /// trong `.atproj` bị ghi trước khi lỗi này được trả (AC8).
    WorkMetaTooNew => "err.work.meta_too_new" ["found", "supported"],
    /// Mở lại một `.atproj` đã có trên đĩa thất bại vì một lý do KHÁC `meta.json` quá mới
    /// (thư mục biến mất, quyền đọc, …) — [`crate::core::library::WorkError::OpenFailed`].
    WorkOpenFailed => "err.work.open_failed" ["name"],
    /// `open_work` nhận một `work_id` không có hàng trong `library-index.db`
    /// (`Indexer::find_work` trả `None`). `OpenWorkState` không đổi.
    LibraryWorkNotIndexed => "err.library.work_not_indexed" ["work_id"],

    // ── Story 5.8 (FR15 · AD-32) — ĐÚNG BA khoá mới ─────────────────────────────────
    //
    // Bề mặt IPC "Tổ chức lại Chương": đổi tên · dời lên/xuống · gộp vào Chương liền trước ·
    // tách tại câu đang có caret. Danh mục ĐÓNG: ca "chưa Tác phẩm nào mở" tái dùng
    // `WorkNoneOpen`, ca "`chapter_id` không tồn tại" tái dùng `SegmentChapterNotFound`, ca
    // "segment_id lạ / cặp lệch" tái dùng `SegmentNotFound` — cả ba đã có ở trên, không đúc
    // khoá thứ hai/ba/tư cho cùng câu. Ba khoá dưới đây phủ đúng ba sự thật KHÔNG khoá nào
    // hiện có nói được: đã ở biên khi dời/gộp, và một lượt tách sẽ để lại một Chương rỗng.
    /// Dời một Chương LÊN khi nó đã ở vị trí ĐẦU, hoặc gộp một Chương khi không có Chương nào
    /// liền trước nó — cả hai đều là *"không có hàng liền trước theo `(ord, id)`"*, cùng một
    /// sự thật cho hai lệnh khác nhau. `chapter_id` là Chương vừa được chỉ. **0 hàng bị chạm.**
    ChapterAtFirst => "err.chapter.at_first" [],
    /// Dời một Chương XUỐNG khi nó đã ở vị trí CUỐI — không có hàng liền sau theo `(ord, id)`.
    /// **0 hàng bị chạm.**
    ChapterAtLast => "err.chapter.at_last" [],
    /// Tách tại một câu là câu ĐẦU Chương (hoặc không còn hàng SỐNG nào đứng trước nó) ⇒
    /// Chương mới sẽ RỖNG — một kết quả không có nghĩa, bị từ chối trước khi chạm SQL ghi nào.
    /// `chapter_id` là Chương sẽ bị để rỗng (Chương đang mở, phía trước điểm cắt).
    ChapterSplitLeavesEmpty => "err.chapter.split_leaves_empty" [],
}

/// 🔴 `Serialize` VIẾT TAY, và đây là chỗ dễ hỏng im lặng nhất của cả story.
///
/// `#[derive(Serialize)]` trần trên một unit variant cho ra **TÊN BIẾN THỂ**:
/// `MessageKey::IoReadFailed` → `"IoReadFailed"`. Đó là một chuỗi, JSON hợp lệ,
/// không lỗi nào được ném, `cargo build` sạch. Frontend tra `"IoReadFailed"` trong
/// `vi.json`, không thấy, và theo đúng AC4 nó **hiện khoá nguyên văn rồi ghi cảnh
/// báo** — nghĩa là hỏng đúng kiểu *"trông như đang chạy"*.
///
/// Đừng thay bằng `#[derive(Serialize)]` + `#[serde(rename = "…")]` trên từng
/// biến thể: khoá khi đó có hai nguồn (`rename` và `as_str`) và chúng sẽ trôi khỏi
/// nhau. Một nguồn duy nhất là `message_keys!`.
impl Serialize for MessageKey {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.as_str())
    }
}

/// Hình dạng lỗi vượt ranh giới IPC — AD-21, **hợp đồng nguyên văn bốn trường**.
///
/// **KHÔNG đặt `#[serde(rename_all = "camelCase")]` lên struct này.** Thói quen
/// viết Tauri là đặt nó lên mọi struct qua IPC cho hợp phong cách JS; ở đây nó biến
/// `message_key` thành `messageKey`. Rust biên dịch sạch, không test nào đỏ trừ khi
/// test so ĐÚNG CHÍNH TẢ KHOÁ, và mọi chỗ đọc theo AD-21 nhận `undefined` rồi hiển
/// thị chuỗi rỗng. Bốn tên trường là DÂY, không phải sở thích — `tests/ipc_contract.rs`
/// khoá chúng lại.
///
/// Hai điều kiện nữa, cả hai đều có lý do đo được:
///
/// - `params` là `BTreeMap`, **không phải `HashMap`**: thứ tự khoá ổn định thì test
///   so JSON mới ổn định qua từng lượt chạy.
/// - Giá trị của `params` là `String`, **kể cả số**. Định dạng số và ngày giờ chỉ ở
///   frontend (cùng nguyên tắc với *"lưu ISO-8601 UTC, định dạng hiển thị chỉ ở
///   frontend"* của Consistency Conventions).
///
/// **`params` cũng không được mang văn bản hiển thị.** Một
/// `params: {"reason": "Nhà cung cấp không phản hồi"}` là AD-21 bị thủng qua cửa sau.
/// Tham số mang **dữ liệu** (đường dẫn, số đếm, tên nhà cung cấp), không mang **câu**.
///
/// **TRƯỜNG RIÊNG TƯ, dựng CHỈ qua [`IpcError::new`].** Không phải để giấu dữ liệu —
/// bốn trường vẫn đọc được qua bốn accessor và vẫn đi nguyên vẹn trên dây. Lý do là
/// `new` là chỗ DUY NHẤT nối `message_key` với `params`, và một struct literal đi vòng
/// qua nó (`IpcError { params: BTreeMap::new(), .. }`) biên dịch sạch, qua mọi phép
/// kiểm hiện có, rồi đặt nguyên văn `{path}` lên màn hình người dùng.
/// ⚠️ `#[non_exhaustive]` KHÔNG làm được việc này: nó chỉ chặn struct literal từ crate
/// KHÁC, mà các `#[tauri::command]` của Story 1.6 nằm trong CÙNG crate.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct IpcError {
    /// Định danh máy đọc, ổn định qua mọi lần sửa lời văn — frontend **rẽ nhánh** trên
    /// trường này.
    ///
    /// ⚠️ `code` và `message_key` được phép 1:1 hôm nay (chưa nhánh nào cần rẽ), nhưng
    /// chúng là HAI trường chứ không phải một trường hai tên, và được phép rời nhau về
    /// sau mà không phải đổi hợp đồng. không `code` không bao giờ được đưa ra màn hình.
    code: String,

    /// Khoá tra trong `vi.json`. Kiểu là `MessageKey` chứ không phải `String`, nên một
    /// khoá không có trong danh mục không biên dịch được — thay vì lộ ra lúc chạy.
    message_key: MessageKey,

    /// Tham số nội suy cho chuỗi. Dữ liệu, không phải câu. Xem doc-comment của struct.
    params: BTreeMap<String, String>,

    /// **Chỉ là quyền hiển thị một nút thử lại.**
    ///
    /// Không mã nào được tự thử lại khi thấy `true`: AD-22 cấm auto-retry, và với
    /// BYOK (người dùng trả tiền API của chính họ) một lượt tự thử lại là tính tiền
    /// hai lần cho một thao tác họ chưa yêu cầu lần thứ hai.
    retryable: bool,
}

impl IpcError {
    /// Chỗ DUY NHẤT dựng được một `IpcError` — và chỗ duy nhất `message_key` gặp `params`.
    ///
    /// # Thiếu tham số thì sao
    ///
    /// Một khoá đòi `["path"]` mà `params` không có `path` là **lỗi lập trình**, và nó
    /// được xử theo hai chế độ khác nhau vì hai hoàn cảnh khác nhau:
    ///
    /// - **Debug và `cargo test`** ⇒ `debug_assert!` nổ ngay, nêu đích danh khoá và tham
    ///   số thiếu. Đây là nơi lỗi phải chết: sớm, ồn, cạnh chỗ gây ra nó.
    /// - **Release** ⇒ KHÔNG panic. `Cargo.toml` đặt `panic = "abort"`, nên một panic
    ///   trong đường **báo lỗi** giết luôn tiến trình và cuốn theo cả `core::store`
    ///   (AD-11/AD-12, xem `deferred-work.md`). Thay vào đó khoá rơi về
    ///   [`MessageKey::Unknown`] — người dùng đọc một câu hoàn chỉnh thay vì một
    ///   placeholder thô, `code` giữ nguyên nên chẩn đoán không mất, và hành vi này
    ///   không bao giờ tệ hơn hôm nay.
    ///
    /// ⚠️ Tham số THỪA được giữ nguyên, có chủ ý: chúng vô hại lúc hiển thị
    /// (`resolve.ts` chỉ thay placeholder nó gặp) và hữu ích lúc chẩn đoán.
    pub fn new(
        code: impl Into<String>,
        message_key: MessageKey,
        params: BTreeMap<String, String>,
        retryable: bool,
    ) -> Self {
        let missing: Vec<&str> = message_key
            .required_params()
            .iter()
            .copied()
            .filter(|p| !params.contains_key(*p))
            .collect();

        if missing.is_empty() {
            return Self { code: code.into(), message_key, params, retryable };
        }

        // ⚠️ Thông báo viết KHÔNG DẤU, và đó không phải một lựa chọn thẩm mỹ: `src/**` nằm
        // trong phạm vi Kiểm A của `npm run check:i18n`, và một chuỗi tiếng Việt có dấu ở
        // vị trí mã là vi phạm AC2 — kể cả khi nó chỉ tồn tại ở profile debug. Lời văn
        // đầy đủ nằm ở doc-comment của hàm này; chỗ này chỉ mang DỮ LIỆU.
        // (Cổng đã bắt đúng ca này một lần trong lượt review 2026-08-04.)
        debug_assert!(
            missing.is_empty(),
            "IpcError::new -- message_key={} required={:?} missing={:?} \
             -- see the IpcError::new doc-comment",
            message_key.as_str(),
            message_key.required_params(),
            missing,
        );

        Self {
            code: code.into(),
            message_key: MessageKey::Unknown,
            params,
            retryable,
        }
    }

    /// Định danh máy đọc để frontend rẽ nhánh. Không bao giờ đưa ra màn hình.
    pub fn code(&self) -> &str {
        &self.code
    }

    /// Khoá tra trong `vi.json`.
    pub fn message_key(&self) -> MessageKey {
        self.message_key
    }

    /// Tham số nội suy. Dữ liệu, không phải câu.
    pub fn params(&self) -> &BTreeMap<String, String> {
        &self.params
    }

    /// Quyền hiển thị một nút thử lại. Không phải lệnh tự thử lại (AD-22).
    pub fn retryable(&self) -> bool {
        self.retryable
    }
}
