//! Hình dạng trung gian dùng chung cho cả năm parser — Task 4 của Story 1.9.
//!
//! Năm nguồn có năm định dạng thô hoàn toàn khác nhau, nhưng đều phơi ra CÙNG một
//! hàm `parse(reader) -> impl Iterator<Item = Result<RawEntry, ParseIssue>>`. Cùng một
//! hình dạng đầu ra là điều kiện để AC2 vế "không hợp nhất" kiểm được — mã hợp nhất bao
//! giờ cũng xuất hiện ở chỗ năm hình dạng khác nhau phải quy về một, và nếu hình dạng
//! trung gian đã ép năm nguồn về cùng một khuôn `RawEntry` một-nguồn, không ai còn chỗ
//! để lén gộp `sources = "a,b"` vào một hàng (Bẫy 6).
//!
//! ⚠️ `RawEntry` ở đây LUÔN thuộc về ĐÚNG MỘT nguồn — nó chưa mang `source_id` (đó là
//! việc của `build.rs` lúc chèn, sau khi đã biết `dict_source.id` của nguồn đang chạy).

/// Một đầu mục thô, thuộc về đúng một nguồn.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RawEntry {
    /// `'zh'` | `'en'` — ngôn ngữ của đầu mục (không phải ngôn ngữ của nghĩa).
    pub lang: String,
    pub headword: String,
    /// Giản thể, khi nguồn có phân biệt phồn/giản.
    pub headword_simp: Option<String>,
    /// Pinyin hoặc cách đọc khác, khi nguồn có.
    pub reading: Option<String>,
    /// Âm HÁN VIỆT thật — ÂM ĐỌC, không phải NGHĨA. ⛔ Không đẩy vào `senses`; xem
    /// doc-comment `dict_entry.han_viet` ở `schema.rs`.
    ///
    /// 🔴 Story 1.10c AC2: trường này mang ĐÚNG MỘT ngữ nghĩa ở MỌI nguồn — âm Hán Việt
    /// gắn nhãn tường minh (Thiều Chửu, en-wiktionary-vi, Trần Văn Chánh). `Unihan
    /// kVietnamese` KHÔNG còn đổ vào đây (xem `nom_reading`) — nó là âm NÔM, không phải
    /// âm Hán Việt (§Phát hiện của story: 92,4% trùng một âm Nôm đã gắn nhãn).
    pub han_viet: Option<String>,
    /// Âm NÔM — ÂM ĐỌC tiếng Việt của một ký tự khi dùng làm chữ Nôm, ⛔ không phải âm
    /// Hán Việt. Story 1.10c AC1/AC4: `Unihan kVietnamese` đổ vào ĐÂY (đổi vai, không mất
    /// dữ liệu); `en-wiktionary-vi` đổ nhãn `nom-reading` vào ĐÂY.
    pub nom_reading: Option<String>,
    pub senses: Vec<RawSense>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RawSense {
    pub pos: Option<String>,
    /// FR35: bắt buộc khai khi `pos` đến từ một ngôn ngữ khác tiếng Việt — `'vi'` | `'en'`.
    pub pos_lang: Option<String>,
    pub gloss: String,
    pub note: Option<String>,
    pub examples: Vec<RawExample>,
    pub citations: Vec<RawCitation>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RawExample {
    pub text: String,
    pub translation: Option<String>,
    pub translation_lang: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RawCitation {
    pub text: String,
    pub work: Option<String>,
    pub author: Option<String>,
}

/// Một dòng/bản ghi bị bỏ, kèm lý do — DỮ LIỆU, không phải log (§Quyết định #8). Không
/// có bảng này thì "nguồn thứ tư đọc hỏng 90%" trông giống hệt "nguồn thứ tư vốn nhỏ".
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseIssue {
    /// Số dòng/bản ghi trong tệp nguồn (1-based khi nguồn có khái niệm "dòng").
    pub line: usize,
    pub reason: String,
}

impl std::fmt::Display for ParseIssue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "line {}: {}", self.line, self.reason)
    }
}

/// Đếm dòng đọc / dòng bỏ cho một nguồn — in ra bảng cuối lượt (§Quyết định #8).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SourceStats {
    pub source_code: String,
    pub lines_read: usize,
    pub lines_skipped: usize,
    pub entries: usize,
    pub senses: usize,
    pub examples: usize,
    pub citations: usize,
    /// Lý do bỏ → số lần, để một lượt đọc bảng thấy ngay lớp lỗi nào chiếm đa số.
    pub skip_reasons: std::collections::BTreeMap<String, usize>,
}

impl SourceStats {
    pub fn new(source_code: impl Into<String>) -> Self {
        Self {
            source_code: source_code.into(),
            ..Default::default()
        }
    }

    pub fn record_skip(&mut self, reason: &str) {
        self.lines_skipped += 1;
        // dict-build:allow .entry( — đếm lý do bỏ TRONG một nguồn, không hợp nhất nghĩa
        // dict-build:allow or_insert — xuyên nguồn (AD-19); key là lý do bỏ, không phải headword
        *self.skip_reasons.entry(reason.to_string()).or_insert(0) += 1;
    }

    pub fn record_entry(&mut self, entry: &RawEntry) {
        self.entries += 1;
        self.senses += entry.senses.len();
        for s in &entry.senses {
            self.examples += s.examples.len();
            self.citations += s.citations.len();
        }
    }
}
