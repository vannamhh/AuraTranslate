//! Đề xuất bản dịch bằng âm Hán Việt cho một thuật ngữ Glossary CHỜ CHỐT — Story 3.7, FR113.
//!
//! ─────────────────────────────────────────────────────────────────────────────
//! 🔴 CẠNH `glossary/ → dict/` LÀ CẠNH THẬT — AD-36 chỉ định đích danh
//! ─────────────────────────────────────────────────────────────────────────────
//! `use crate::core::dict::{..}` ngay dưới đây LÀ cạnh đó: `core/glossary/**` trước module
//! này có **0** dòng gọi thẳng `core::dict` (`scan.rs` cố ý TIÊM một closure thay vì `use`
//! thẳng, để giữ thuật toán quét THUẦN và test được không cần một `Store`/`DictLayers` nào —
//! xem doc-comment đầu `scan.rs`). Module này chọn NGƯỢC LẠI có chủ ý: âm Hán Việt **là**
//! dữ liệu từ điển, không phải một quy tắc quét, và AD-36 chỉ định đích danh "âm Hán Việt cho
//! FR113 đọc qua cổng `DictionarySource`" — tức đọc TRỰC TIẾP, không qua một lớp tiêm gián
//! tiếp thứ hai chỉ để giữ một quy ước module không áp cho trường hợp này. Chiều ngược lại
//! (`core::dict` gọi `core::glossary`) vẫn là **0** — giữ nguyên, xem cổng đồ thị phụ thuộc
//! ở `tests/dict_boundary.rs`.
//!
//! ─────────────────────────────────────────────────────────────────────────────
//! 🔴 MODULE THUẦN — KHÔNG CHẠM `Store`, KHÔNG BIẾT SQL
//! ─────────────────────────────────────────────────────────────────────────────
//! [`suggest_han_viet_batch`] chỉ nhận [`DictLayers`] (đã mở sẵn, do `commands::dict` quản
//! lý) + tập nguồn đã tắt + danh sách thuật ngữ — không tự đọc `Store`/`ScopeResolver` nào.
//! Chỗ gọi ([`super::store::marks_for_source_text`], `commands::glossary::
//! glossary_pending_candidates`) chịu trách nhiệm gom `source_term` của các mục CHỜ CHỐT
//! trước khi gọi hàm này.
//!
//! ─────────────────────────────────────────────────────────────────────────────
//! 🔴 BỐN LÝ DO RỖNG PHẢI PHÂN BIỆT ĐƯỢC TRÊN DÂY — một `Option<String>` trần là RỖNG IM LẶNG
//! ─────────────────────────────────────────────────────────────────────────────
//! `AGENTS.md` gọi rỗng im lặng là lỗi trung tâm của dự án, và `resources/dict/` RỖNG trong
//! cây git (AD-25) làm ca *"chưa gắn lớp từ điển nào"* thành ca THƯỜNG GẶP NHẤT ở máy dev —
//! nó phải có một nhãn RIÊNG (`DictUnavailable`), khác hẳn *"đã tra mà thiếu âm"*
//! (`NoReading`). [`HanVietSuggestion`] là một `enum` NĂM nhánh, danh mục ĐÓNG (khuôn
//! `Resolution`/`CandidateOrigin` của `core::glossary`), thay cho một `Option<String>` trần.
//!
//! ⚠️ Mọi chuỗi trong `src-tauri/src/**` viết KHÔNG DẤU; doc-comment có dấu là hợp lệ.

use std::collections::{BTreeSet, HashMap, HashSet};

use crate::core::dict::{DictLayers, HanVietReading, is_han, lookup_han_viet};

/// Kết quả tra âm Hán Việt cho MỘT thuật ngữ — danh mục ĐÓNG, năm nhánh.
///
/// 🔴 Không thêm nhánh thứ sáu mà không sửa `as_status_str`/`src/config/glossary.ts` (type
/// guard năm chuỗi) CÙNG LƯỢT — hai bên là MỘT hợp đồng, không hai.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HanVietSuggestion {
    /// Mọi ký tự của thuật ngữ đều có âm — đề xuất đã ghép sẵn (hoa đầu mỗi âm tiết, nối
    /// bằng một khoảng trắng, đúng thứ tự ký tự nguồn).
    Ready(String),
    /// Không phải một chuỗi Hán — vị từ là **HÌNH DẠNG CHUỖI** (mọi ký tự là Hán), không
    /// phải `source_lang` của Tác phẩm (xem doc-comment đầu module cho lý do đầy đủ).
    NotChinese,
    /// Là chữ Hán nhưng ÍT NHẤT một ký tự thiếu âm ở mọi lớp đang gắn — không đề xuất một
    /// phần: một đề xuất thiếu chữ còn tệ hơn không đề xuất, vì nó ĐÚNG hình dạng của một
    /// gợi ý hoàn chỉnh.
    NoReading,
    /// **0** lớp từ điển đang gắn (AD-25) — khác hẳn [`Self::NoReading`]: đây là "chưa từng
    /// hỏi được", không phải "đã hỏi mà không có".
    DictUnavailable,
    /// Dấu ĐÃ CHỐT — **0** lượt tra Hán Việt chạy cho dấu này (mục đã có `translation` thật,
    /// một đề xuất cho nó không có chỗ tiêu thụ). Nhãn RIÊNG, không mượn [`Self::NotChinese`]:
    /// một thuật ngữ chữ Hán đã chốt vẫn LÀ chữ Hán, gắn cho nó nhãn "không phải tiếng Trung"
    /// là một lời nói dối nhỏ mà lượt sau sẽ đọc như sự thật. Chỗ gọi (không phải hàm này)
    /// gán nhãn này trực tiếp cho mọi mục `is_confirmed() == true`, TRƯỚC khi vào
    /// [`suggest_han_viet_batch`] — hàm này không bao giờ tự trả về nhánh này.
    NotRequested,
}

impl HanVietSuggestion {
    /// Chuỗi đề xuất, hoặc `None` cho bốn nhánh còn lại — đây là thứ đi vào
    /// `GlossaryMark::han_viet_suggestion`/`GlossaryMarkWire::han_viet_suggestion`.
    pub fn suggestion_text(&self) -> Option<&str> {
        match self {
            HanVietSuggestion::Ready(text) => Some(text.as_str()),
            HanVietSuggestion::NotChinese
            | HanVietSuggestion::NoReading
            | HanVietSuggestion::DictUnavailable
            | HanVietSuggestion::NotRequested => None,
        }
    }

    /// Định danh máy đọc trên dây — **năm** chuỗi đóng, khớp NGUYÊN VĂN §I/O Matrix của
    /// spec. Không phải nhãn hiển thị (AD-21, NFR16) — nhãn hiển thị sống ở `vi.json` phía
    /// frontend, chọn theo giá trị chuỗi này.
    pub const fn as_status_str(&self) -> &'static str {
        match self {
            HanVietSuggestion::Ready(_) => "ok",
            HanVietSuggestion::NotChinese => "not_chinese",
            HanVietSuggestion::NoReading => "no_reading",
            HanVietSuggestion::DictUnavailable => "dict_unavailable",
            HanVietSuggestion::NotRequested => "not_requested",
        }
    }
}

/// Hoa CHỮ CÁI ĐẦU của một âm tiết Hán Việt (`"tây"` → `"Tây"`) — âm Hán Việt luôn là MỘT
/// âm tiết tiếng Việt (không dấu `|`/`,`/khoảng trắng bên trong, `split_readings` của
/// `core::dict` đã tách sạch), nên "hoa chữ cái đầu" và "hoa từ đầu" là MỘT thao tác ở đây.
fn capitalize_first(syllable: &str) -> String {
    let mut chars = syllable.chars();
    match chars.next() {
        None => String::new(),
        Some(first) => first.to_uppercase().chain(chars).collect(),
    }
}

/// Tính đề xuất cho MỘT thuật ngữ, từ bảng tra `readings` đã gom cho cả LÔ — hàm THUẦN nội
/// bộ, không I/O.
///
/// 🔴 Thứ tự xét: HÌNH DẠNG trước ([`is_han`] cho MỌI ký tự — độc lập với việc từ điển đã
/// gắn hay chưa), rồi mới tới TÌNH TRẠNG TỪ ĐIỂN ([`DictUnavailable`](HanVietSuggestion::DictUnavailable)),
/// rồi mới tới TỪNG KÝ TỰ ([`NoReading`](HanVietSuggestion::NoReading)). Một chuỗi tiếng Anh
/// vẫn là `NotChinese` bất kể `resources/dict/` có rỗng hay không — dict rỗng không đổi
/// HÌNH DẠNG của chuỗi đầu vào.
fn suggest_for_term(
    term: &str,
    layers_loaded: bool,
    readings: &HashMap<char, Option<&HanVietReading>>,
) -> HanVietSuggestion {
    if term.is_empty() || !term.chars().all(is_han) {
        return HanVietSuggestion::NotChinese;
    }
    if !layers_loaded {
        return HanVietSuggestion::DictUnavailable;
    }

    let mut syllables: Vec<String> = Vec::with_capacity(term.chars().count());
    for ch in term.chars() {
        match readings.get(&ch) {
            Some(Some(reading)) => syllables.push(capitalize_first(&reading.primary)),
            // `None` (ký tự ngoài lô đã tra -- bất khả trên đường gọi đúng) VÀ
            // `Some(None)` (đã tra, không lớp nào có âm) đều rơi về CÙNG một kết luận: đề
            // xuất KHÔNG đầy đủ, không đề xuất một phần.
            _ => return HanVietSuggestion::NoReading,
        }
    }
    HanVietSuggestion::Ready(syllables.join(" "))
}

/// **Hàm phơi ra** của module này — Story 3.7, FR113. Tính đề xuất âm Hán Việt cho MỖI
/// thuật ngữ trong `terms`, ĐÚNG vị trí và ĐÚNG số lượng (khuôn [`crate::core::dict::
/// lookup_han_viet`]).
///
/// 🔴 **Gọi [`lookup_han_viet`] ĐÚNG MỘT LẦN cho toàn bộ ký tự đã dedupe của CẢ LÔ** — không
/// một lượt cho mỗi thuật ngữ. `ports/dict_source.rs:120-126` đã viết sẵn lý do cấm N+1 cho
/// đúng phương thức `han_viet` mà `lookup_han_viet` gọi xuống: một Chương thật có thể mang
/// hàng chục thuật ngữ chờ chốt, và tra riêng từng thuật ngữ nhân số lượt gọi `DictionarySource::
/// han_viet` lên đúng bấy nhiêu lần cho một tập ký tự phần lớn TRÙNG NHAU giữa các thuật ngữ.
///
/// `disabled` đi thẳng xuống [`lookup_han_viet`] — **cùng bộ lọc nguồn đã tắt** với tab Hán
/// Việt (Story 1.19 §Quyết định #3a), không đúc phép đọc thứ hai.
pub fn suggest_han_viet_batch(
    layers: &DictLayers,
    disabled: &BTreeSet<String>,
    terms: &[&str],
) -> Vec<HanVietSuggestion> {
    // Dedupe MỌI ký tự Hán của CẢ LÔ, giữ thứ tự gặp lần đầu -- các ký tự không-Hán không
    // đáng một lượt tra (chúng chỉ khiến thuật ngữ chứa chúng thành `NotChinese`, quyết định
    // đó không cần dữ liệu từ điển).
    let mut seen: HashSet<char> = HashSet::new();
    let mut unique_chars: Vec<String> = Vec::new();
    for term in terms {
        for ch in term.chars() {
            if is_han(ch) && seen.insert(ch) {
                unique_chars.push(ch.to_string());
            }
        }
    }
    let char_refs: Vec<&str> = unique_chars.iter().map(String::as_str).collect();

    let lookup = lookup_han_viet(layers, &char_refs, disabled);
    let layers_loaded = lookup.layers_loaded;

    let mut readings: HashMap<char, Option<&HanVietReading>> =
        HashMap::with_capacity(lookup.characters.len());
    for entry in &lookup.characters {
        if let Some(ch) = entry.character.chars().next() {
            readings.insert(ch, entry.reading.as_ref());
        }
    }

    terms
        .iter()
        .map(|term| suggest_for_term(term, layers_loaded, &readings))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeSet;

    // ⚠️ Ca hành vi ĐẦY ĐỦ (dữ liệu Hán thật, nhiều lớp, nguồn tắt, ...) sống ở
    // `tests/glossary_han_viet_suggestion_contract.rs` -- test unit ở đây chỉ canh các hàm
    // THUẦN nội bộ mà tệp `tests/` không chạm tới trực tiếp (`capitalize_first`,
    // `as_status_str`), cùng khuôn `core/dict/mod.rs::split_readings_tests`.

    #[test]
    fn capitalize_first_uppercases_only_the_first_character() {
        assert_eq!(capitalize_first("tay"), "Tay");
        assert_eq!(capitalize_first(""), "");
    }

    #[test]
    fn as_status_str_spells_all_five_closed_variants() {
        assert_eq!(HanVietSuggestion::Ready("Bac Luong".to_owned()).as_status_str(), "ok");
        assert_eq!(HanVietSuggestion::NotChinese.as_status_str(), "not_chinese");
        assert_eq!(HanVietSuggestion::NoReading.as_status_str(), "no_reading");
        assert_eq!(HanVietSuggestion::DictUnavailable.as_status_str(), "dict_unavailable");
        assert_eq!(HanVietSuggestion::NotRequested.as_status_str(), "not_requested");
    }

    #[test]
    fn suggestion_text_is_some_only_for_ready() {
        assert_eq!(HanVietSuggestion::Ready("x".to_owned()).suggestion_text(), Some("x"));
        assert_eq!(HanVietSuggestion::NotChinese.suggestion_text(), None);
        assert_eq!(HanVietSuggestion::NoReading.suggestion_text(), None);
        assert_eq!(HanVietSuggestion::DictUnavailable.suggestion_text(), None);
        assert_eq!(HanVietSuggestion::NotRequested.suggestion_text(), None);
    }

    #[test]
    fn a_latin_term_is_not_chinese_even_with_zero_layers_loaded() {
        let layers = DictLayers::empty();
        let disabled = BTreeSet::new();
        let out = suggest_han_viet_batch(&layers, &disabled, &["dragon"]);
        assert_eq!(out, vec![HanVietSuggestion::NotChinese]);
    }

    #[test]
    fn an_empty_term_is_not_chinese() {
        let layers = DictLayers::empty();
        let disabled = BTreeSet::new();
        let out = suggest_han_viet_batch(&layers, &disabled, &[""]);
        assert_eq!(out, vec![HanVietSuggestion::NotChinese]);
    }

    #[test]
    fn a_chinese_term_with_zero_layers_loaded_is_dict_unavailable_not_no_reading() {
        let layers = DictLayers::empty();
        let disabled = BTreeSet::new();
        let out = suggest_han_viet_batch(&layers, &disabled, &["北涼"]);
        assert_eq!(out, vec![HanVietSuggestion::DictUnavailable]);
    }
}
