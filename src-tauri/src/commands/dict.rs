//! Bề mặt IPC đọc âm Hán Việt cho tab Hán Việt — Story 1.16, AC4. Bề mặt IPC tra cứu từ
//! điển có cấu trúc cho Panel Lookup — Story 1.17, AC8.
//!
//! ─────────────────────────────────────────────────────────────────────────────
//! 🔴 VÌ SAO MỘT COMMAND RIÊNG, ⛔ KHÔNG GỘP VÀO `read_open_chapter`
//! ─────────────────────────────────────────────────────────────────────────────
//! `commands::chapter::read_open_chapter` trả `source_text` — dữ liệu bắt buộc cho MỌI
//! Chương. Gom âm Hán Việt vào cùng lượt trả về đó buộc MỌI lần mở Chương (kể cả nguồn
//! **tiếng Anh**, AC3 — ⛔ không tab Hán Việt) tính toán qua tập ba tệp `.db` từ điển. Tách
//! thành một command riêng để webview chỉ gọi nó **khi `source_lang == "zh"`** — cùng
//! nguyên tắc "adapter mỏng, chỗ gọi quyết khi nào cần" đã áp cho `bootstrap_config`/
//! `put_config` (Story 1.8).
//!
//! ⛔ **Không cổng thứ tư** (AD-2): command này chỉ là vỏ IPC gọi xuống
//! [`crate::core::dict::lookup_han_viet`] — cổng vẫn đúng ba: `DictionarySource` (+ `Store`
//! + `ReadOnlyDb`, hai cổng còn lại của bộ ba AD-2).
//!
//! ⚠️ Mọi chuỗi trong tệp này viết KHÔNG DẤU — `scripts/check-i18n.mjs` Kiểm A quét
//! `src-tauri/**/*.rs`.

use std::collections::BTreeMap;

use crate::core::dict::{DictLayers, GroupedLookup, HanVietLookup, LookupMode, SenseRecord, lookup_grouped, lookup_han_viet};
use crate::ports::DictionarySource;

/// Đọc âm Hán Việt cho `chars` — **hàm thuần, đây là thứ test gọi**.
///
/// `layers = None` đối xử **giống hệt** một tập lớp rỗng — `DictLayers` luôn được
/// `app.manage(...)` ở `setup()` (kể cả khi rỗng, xem `lib.rs::open_dict_layers`), nên
/// nhánh `None` chỉ xảy ra nếu cấu hình `setup()` sai, và hành vi đúng của nó là **giống
/// ca "0 lớp"** — một trạng thái BÌNH THƯỜNG có tên (AD-25), ⛔ không phải một lỗi.
///
/// ⛔ **Không có nhánh lỗi**: một lớp hỏng lúc tra được [`lookup_han_viet`] xử lý bằng cách
/// coi lớp đó không đóng góp gì (cùng luật `lookup_grouped`), ⛔ không làm hỏng cả lượt.
pub fn read_han_viet(layers: Option<&DictLayers>, chars: &[String]) -> HanVietLookup {
    let refs: Vec<&str> = chars.iter().map(String::as_str).collect();
    let empty = DictLayers::empty();
    lookup_han_viet(layers.unwrap_or(&empty), &refs)
}

/// 🔴 **Quyết định #4 (Story 1.17)** — cỡ trang pha một, **quyết ở ĐÂY** (Panel Lookup),
/// ⛔ không phải một hằng chôn trong `core/dict/**` (`ports/dict_source.rs` viết sẵn lý do:
/// một `LIMIT` là **chính sách sản phẩm**, ⛔ không phải quy tắc dữ liệu).
///
/// ✅ **CHỐT ở Task 8** (2026-08-06), từ số đo đầu-cuối thật trên `tools/dict-build/out/*.db`
/// (`--release`, 4 lớp thật): `20` là giá trị đã đo — cắt nhánh `char_idx` một ký tự từ p95
/// **20,836 ms** (không `LIMIT`) xuống **5,109 ms** (`LIMIT 20`), và đường sản phẩm thật
/// (`commands::dict::lookup`, `LookupMode::Exact` cố định) đo được p95 **6,535 ms** cho ca
/// xấu nhất tìm được (`"山"`, 7 nhóm/10 hàng) — dưới xa trần đầu-cuối 100 ms của NFR1. Xem
/// §Debug Log References của story cho bảng đầy đủ.
const LOOKUP_PAGE_LIMIT: usize = 20;

/// 🔴 `deferred-work.md:363` — *"⛔ Không giới hạn độ dài truy vấn — validate thuộc tầng
/// IPC/UI của 1.13/**1.17**"*. Một sàn TRÊN có tên, ⛔ **không** một `panic`: một lượt bôi
/// đen vô tình kéo qua nhiều đoạn văn (hàng nghìn ký tự) vẫn phải trả lời, ⛔ không đơ máy
/// hay ném lỗi — nó chỉ bị CẮT trước khi vào đường tra, vì một truy vấn dài hơn ngần này
/// vô nghĩa với `LookupMode::Exact` (Quyết định #3): ⛔ đầu mục nào trong từ điển dài đến
/// thế, nên phần vượt sàn chỉ tốn công so khớp mà ⛔ bao giờ khớp.
const QUERY_LENGTH_CEILING: usize = 200;

/// Kết quả một lượt tra Panel Lookup — pha một GOM cộng pha hai HYDRATE, **một lượt IPC**.
///
/// ⚠️ **`senses_by_layer` khoá theo LỚP**, ⛔ không theo `entry_id` phẳng: `entry_id` chỉ
/// duy nhất **trong một tệp**, nên một khoá phẳng sẽ trộn nghĩa của hai lớp mang cùng số
/// `entry_id` — đúng lỗi mà [`crate::core::dict::SourceGroup::layer`] tồn tại để ngăn.
/// Webview zip `senses_by_layer[group.layer]` lại với `group.entries` theo `entry_id`.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct LookupResponse {
    /// Pha một — nhóm theo nguồn, ⛔ hợp nhất.
    pub grouped: GroupedLookup,
    /// Pha hai — nghĩa đã hydrate, **chỉ cho tập đầu mục pha một vừa trả về** (đã bị
    /// `LIMIT` của Quyết định #4 cắt bớt nếu cần) — ⛔ không hydrate toàn bộ từ điển.
    pub senses_by_layer: BTreeMap<String, Vec<SenseRecord>>,

    /// 🔴 **Truy vấn đã bị [`QUERY_LENGTH_CEILING`] CẮT trước khi vào đường tra.**
    ///
    /// ⚠️ Cùng nguyên tắc `truncated_layers` của Quyết định #4, áp cho trần **độ dài** thay
    /// vì trần **số hàng**: một lượt bôi đen dài hơn trần bị cắt rồi tra `Exact` ⇒ chắc
    /// chắn 0 kết quả ⇒ panel hiện *"⛔ tìm thấy trong từ điển"*, một câu **SAI** — hệ
    /// thống ⛔ hề tra thứ người dùng chọn. Bản đầu của 1.17 cắt im lặng (bắt ở code review
    /// 2026-08-07). Panel đọc cờ này để nói ra rằng vùng chọn quá dài, ⛔ im.
    pub query_truncated: bool,
}

/// Tra `query` — **hàm thuần, đây là thứ test gọi** (khuôn `read_han_viet`).
///
/// 🔴 `LookupMode::Exact` **cố định** — Quyết định #3: một lượt bôi đen là một câu hỏi
/// *"cụm này nghĩa gì"*, ⛔ không *"đầu mục nào chứa cụm này"* (câu đó là Concordance,
/// Story 7.7). ⛔ **Không** tham số `mode` trên chữ ký — cùng doctrine `route`/`branch`:
/// một quyết định sản phẩm ⛔ không phải thứ chỗ gọi (webview) tự chọn lại mỗi lượt.
///
/// ⛔ **Không nhánh lỗi cho ca "0 lớp"** — cùng luật [`read_han_viet`] (AD-25).
///
/// 🔴 Pha hai đi **đúng lớp của từng nhóm** — [`DictLayers::layer`] nhận `group.layer`,
/// ⛔ không một tập `entry_id` gộp xuyên lớp (Bẫy 3 của story: trộn `entry_id` giữa các
/// lớp đọc nhầm nghĩa mà ⛔ lỗi nào được ném). Một lượt gọi `senses()` cho **mỗi lớp có
/// nhóm**, ⛔ không một lượt cho mỗi nhóm — một lớp mang nhiều nhóm (nhiều nguồn) dùng
/// chung đúng một lượt gọi.
pub fn lookup(layers: Option<&DictLayers>, query: &str) -> LookupResponse {
    let empty = DictLayers::empty();
    let layers = layers.unwrap_or(&empty);

    let truncated_query: String = query.chars().take(QUERY_LENGTH_CEILING).collect();
    // ⚠️ Đếm ký tự chứ ⛔ so độ dài byte: `truncated_query.len() < query.len()` đúng theo
    // byte nhưng ⛔ trả lời được câu "đã cắt chưa" cho một truy vấn thuần Hán.
    let query_truncated = query.chars().count() > QUERY_LENGTH_CEILING;

    let grouped = lookup_grouped(layers, &truncated_query, LookupMode::Exact, LOOKUP_PAGE_LIMIT);

    let mut entry_ids_by_layer: BTreeMap<&str, Vec<i64>> = BTreeMap::new();
    for group in &grouped.groups {
        entry_ids_by_layer
            .entry(group.layer.as_str())
            .or_default()
            .extend(group.entries.iter().map(|hit| hit.entry_id));
    }

    let mut senses_by_layer = BTreeMap::new();
    for (layer_name, entry_ids) in entry_ids_by_layer {
        let Some(layer) = layers.layer(layer_name) else {
            // Bất khả trong một tập lớp toàn vẹn: `group.layer` đến từ chính
            // `layers.layers()` mà `lookup_grouped` vừa duyệt qua. Rỗng còn hơn panic.
            continue;
        };
        // Lớp hỏng lúc hydrate pha hai ⛔ không được làm hỏng cả lượt tra — pha một của
        // nó đã trả lời được, nên rỗng ở đây chỉ là "chưa hydrate xong", ⛔ không phải
        // "lớp đó không tồn tại". Cùng tinh thần rỗng-có-lý-do của `lookup_grouped`.
        let senses = layer.senses(&entry_ids).unwrap_or_default();
        senses_by_layer.insert(layer_name.to_owned(), senses);
    }

    LookupResponse { grouped, senses_by_layer, query_truncated }
}

/// Một vỏ `#[tauri::command]`. ⛔ **Không một quy tắc nào sống ở đây.**
pub mod wire {
    use super::{DictLayers, HanVietLookup, LookupResponse};

    /// Vỏ IPC của [`super::read_han_viet`].
    #[tauri::command]
    pub fn read_han_viet(app: tauri::AppHandle, chars: Vec<String>) -> HanVietLookup {
        use tauri::Manager as _;

        let managed = app.try_state::<DictLayers>();
        super::read_han_viet(managed.as_deref(), &chars)
    }

    /// Vỏ IPC của [`super::lookup`].
    ///
    /// ⚠️ `try_state`, ⛔ không `state()` — cùng lý do [`read_han_viet`]: state có thể
    /// chưa từng được `app.manage` (lỗi cấu hình `setup()`), và `panic = "abort"` giết
    /// tiến trình nếu ta thẳng tay `.unwrap()`.
    #[tauri::command]
    pub fn lookup_dictionary(app: tauri::AppHandle, query: String) -> LookupResponse {
        use tauri::Manager as _;

        let managed = app.try_state::<DictLayers>();
        super::lookup(managed.as_deref(), &query)
    }
}
