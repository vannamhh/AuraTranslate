//! Diacritic folding and query-shape classification shared by `library` and `dict` (AD-61, AD-69).

use unicode_normalization::char::decompose_canonical;

/// Gấp `đ`/`Đ` về `d`/`D` — chữ DUY NHẤT mà `remove_diacritics` của SQLite (mọi tham số, đo
/// trực tiếp: `unicode61` mức 1/2 lẫn `trigram` mức 1) không gấp được, vì đây là hai ký tự ĐỘC
/// LẬP, không phải `d`/`D` cộng dấu tổ hợp mà bảng gấp dấu built-in xử lý. Dùng khi ghi
/// `library_segment.target_text_fold` (xem [`harvest_work_text`]) và truy vấn
/// [`search_target_text_nd`] — `unicode61 remove_diacritics 2` tự gấp phần dấu tổ hợp còn lại
/// của nửa BẢN DỊCH, nên riêng nửa đó chỉ cần đúng phép gấp này. Nửa NGUYÊN VĂN cần thêm
/// [`fold_diacritics_case_preserving`] — xem hàm đó cho lý do.
pub fn fold_dd_letter(input: &str) -> String {
    input
        .chars()
        .map(|c| match c {
            '\u{0111}' => 'd',
            '\u{0110}' => 'D',
            other => other,
        })
        .collect()
}

/// Gấp dấu ĐẦY ĐỦ, GIỮ NGUYÊN hoa/thường — [`fold_dd_letter`] cộng phép tách NFD rồi bỏ mọi
/// dấu tổ hợp (`U+0300..=U+036F`). Dùng cho nửa NGUYÊN VĂN (`library_segment.source_text_fold`
/// và truy vấn của [`search_source_text_nd`]): **đo trực tiếp (SQLite 3.53.2 nhúng)**, tokenizer
/// `trigram` — dù đặt `remove_diacritics 1` hay không — KHÔNG gấp dấu tổ hợp tiếng Việt (`à`,
/// `ẵ`, …), khác hẳn `unicode61 remove_diacritics 2` của nửa bản dịch; xem chú thích
/// [`LIBRARY_WORK_DDL`] cho số đo đầy đủ. ⇒ Nửa nguyên văn phải tự gấp TRỌN VẸN trước khi ghi/
/// truy vấn, không dựa được vào tokenizer. Giữ hoa/thường nguyên vẹn vì `trigram` đã tự phân
/// biệt-hoa-thường KHÔNG cần tham số nào (xem [`search_source_text`]) — phép gấp này không
/// phải chỗ để lo chuyện đó.
pub fn fold_diacritics_case_preserving(input: &str) -> String {
    fold_dd_letter(input)
        .chars()
        .map(fold_one_char_case_preserving)
        .collect()
}

/// Gấp ĐÚNG MỘT ký tự -- luôn trả về ĐÚNG MỘT ký tự, giữ bất biến một-ký-tự-một-ký-tự mà
/// [`original_text_snippet`] dựa vào để ánh xạ chỉ số ký tự giữa cột đã gấp và `original`.
///
/// NFD trên MỘT ký tự Việt có dấu tổ hợp (`ẵ` → `a` + hai dấu tổ hợp U+0300..=U+036F) luôn ra
/// ĐÚNG một ký tự nền sau khi bỏ dấu -- nhưng NFD KHÔNG luôn vậy cho mọi ký tự: `が` (U+304C)
/// tách thành `か` (U+304B) + U+3099 (dấu đục tiếng Nhật, NGOÀI dải U+0300..=U+036F), và một
/// âm tiết Hangul dựng sẵn (`가`) tách thành HAI/BA Jamo rời, không dấu tổ hợp nào cả. Cả hai
/// ca đó: giữ NGUYÊN ký tự gốc, không tách -- một ký tự "chưa gấp" còn đúng vị trí hơn hai ký
/// tự lệch vị trí.
fn fold_one_char_case_preserving(c: char) -> char {
    let mut parts: Vec<char> = Vec::new();
    decompose_canonical(c, |part| parts.push(part));
    let Some((&base, marks)) = parts.split_first() else {
        return c;
    };
    if marks.iter().all(|m| ('\u{0300}'..='\u{036f}').contains(m)) {
        base
    } else {
        c
    }
}

/// Chế độ một lượt [`Indexer::search`] — danh mục ĐÓNG, hai giá trị, suy từ truy vấn bởi
/// [`classify_query`] (AD-61).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SearchMode {
    /// Chỉ chạy hai chỉ mục CHÍNH (`library_target_fts`/`library_source_fts`), PHÂN BIỆT dấu
    /// tiếng Việt ở cả hai (AD-27).
    Exact,
    /// Nửa BẢN DỊCH đọc thêm `library_target_fts_nd` (`unicode61 remove_diacritics 2`) và nửa
    /// NGUYÊN VĂN đọc thêm `library_source_fts_nd` (`trigram`) — do truy vấn không dấu
    /// (AD-61), HOẶC do một lượt tự nới (xem [`SearchReport::widened`]). Cả hai chỉ mục PHỤ đọc
    /// một cột ĐÃ GẤP riêng cho mình trước khi đưa vào tokenizer — [`fold_dd_letter`] (chỉ
    /// `đ/Đ`) cho nửa bản dịch, [`fold_diacritics_case_preserving`] (thêm dấu tổ hợp) cho nửa
    /// nguyên văn.
    Lenient,
}

impl SearchMode {
    /// Định danh máy đọc — thứ đi trên dây. Không phải nhãn hiển thị (AD-21).
    pub const fn as_str(self) -> &'static str {
        match self {
            SearchMode::Exact => "exact",
            SearchMode::Lenient => "lenient",
        }
    }
}

/// Phân loại truy vấn (AD-61 mục 1) — chỗ DUY NHẤT suy chế độ dấu. `Lenient` khi và chỉ khi có
/// ít nhất một chữ ASCII `[A-Za-z]` và không có `đ`/`Đ` hay dấu nào (dựng sẵn lẫn tổ hợp).
pub fn classify_query(query: &str) -> SearchMode {
    let query = query.trim();
    let has_latin = query.chars().any(|c| c.is_ascii_alphabetic());
    let has_combining_mark = query
        .chars()
        .any(|c| ('\u{0300}'..='\u{036f}').contains(&c));
    if has_latin && !has_combining_mark && fold_diacritics_case_preserving(query) == query {
        SearchMode::Lenient
    } else {
        SearchMode::Exact
    }
}
