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
    ProjectCreateFailed => "err.project.create_failed" [],
    /// Đường đọc Chương gọi trước khi có Tác phẩm nào mở (Story 1.16, AC8) —
    /// `OpenWorkState` rỗng, không phải một lỗi kho.
    ProjectNoWorkOpen => "err.project.no_work_open" [],

    // ─────────────────────────────────────────────────────────────────────────
    // TẦNG SEGMENT — Story 2.1 (AD-3 · AD-4 · AD-5 · AD-21 · AD-37)
    //
    // Hai khoá, và đúng hai: lệnh tách tường minh có đúng hai cách từ chối RIÊNG của
    // nó. *"Chưa mở Tác phẩm nào"* tái dùng `ProjectNoWorkOpen` — cùng câu, cùng
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
    // Tác phẩm đang mở"* và `ProjectNoWorkOpen` cho ca *"chưa mở Tác phẩm nào"* — cùng
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

    // **KHÔNG có `ProjectMetaTooNew` ở đây, và đó là một quyết định** (Ice, code review
    // 2026-08-06). Cơ chế từ chối một `meta.json` mới hơn vẫn còn nguyên và vẫn có test
    // (`MetaError::SchemaTooNew` + `WorkMeta::read`), nhưng không đường sản phẩm nào
    // gọi `WorkMeta::read` — story này không dựng màn hình "mở lại một `.atproj`".
    // Một khoá cho một tính năng chưa tồn tại là đúng thứ Story 1.7 §CN #3 cấm. 🔴 Story
    // nào dựng đường mở lại (ứng viên: Epic 5) thêm lại khoá này CÙNG LƯỢT với màn hình.
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
