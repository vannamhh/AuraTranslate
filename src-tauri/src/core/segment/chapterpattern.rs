//! Mẫu phân tách Chương — Story 6.6, FR14, AD-39 bước 5 ([`pipeline::Step::SplitChapters`]).
//!
//! ─────────────────────────────────────────────────────────────────────────────
//! 🔴 THUẦN TUYỆT ĐỐI — cùng luật `segment_boundary.rs::the_splitter_stays_pure`
//! ─────────────────────────────────────────────────────────────────────────────
//! Không `Store`, không `ScopeKind`, không I/O. Mẫu phân tách là tham số MỖI LƯỢT NHẬP
//! (`pipeline::PipelineInput::chapter_pattern`), KHÔNG một bảng — xem §Design Notes "Vì sao
//! mẫu phân tách KHÔNG là cấu hình hai tầng" của spec 6.6.
//!
//! ─────────────────────────────────────────────────────────────────────────────
//! Cùng khuôn biên dịch regex với `core::cleanup::compile_cleanup_regex`
//! ─────────────────────────────────────────────────────────────────────────────
//! `multi_line(true)` LUÔN bật — `^第.*章` vô nghĩa nếu không có nó, và thiếu cờ này thì mẫu
//! khớp 0 lần TRONG IM LẶNG (`cleanup/mod.rs:230-232` đã khai đúng lý lẽ này).

use std::fmt;

/// Hai hình dạng mẫu — cùng khuôn [`crate::core::cleanup::CleanupRuleKind`].
///
/// `serde::Deserialize` trực tiếp trên kiểu NÀY (không một hàm `from_wire` viết tay ở lớp
/// vỏ) — cùng khuôn `CleanupRuleKind`: tham số `chapter_pattern` của các lệnh IPC được Tauri
/// giải mã thẳng thành kiểu này.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Deserialize)]
pub enum ChapterPatternKind {
    #[serde(rename = "literal")]
    Literal,
    #[serde(rename = "regex")]
    Regex,
}

impl ChapterPatternKind {
    pub const fn as_str(self) -> &'static str {
        match self {
            ChapterPatternKind::Literal => "literal",
            ChapterPatternKind::Regex => "regex",
        }
    }
}

impl fmt::Display for ChapterPatternKind {
    /// KHÔNG DẤU (NFR16) — chẩn đoán cho log, không phải văn bản hiển thị.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Mẫu phân tách Chương ĐÃ PHÂN GIẢI — kiểu của
/// [`crate::core::segment::pipeline::PipelineInput::chapter_pattern`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChapterPattern {
    pub kind: ChapterPatternKind,
    pub pattern: String,
}

impl ChapterPattern {
    /// Khuôn dựng tiện cho `tests/**` và nội bộ — cùng lý do
    /// `core::cleanup::mod::tests::literal`/`regex_rule`.
    pub fn literal(pattern: impl Into<String>) -> Self {
        ChapterPattern { kind: ChapterPatternKind::Literal, pattern: pattern.into() }
    }

    pub fn regex(pattern: impl Into<String>) -> Self {
        ChapterPattern { kind: ChapterPatternKind::Regex, pattern: pattern.into() }
    }

    /// Vị trí BYTE bắt đầu của MỌI khớp KHÔNG-RỖNG trên `text`, theo thứ tự tăng dần — luôn
    /// rơi đúng một ranh giới ký tự thật (đầu một khớp `str`/`regex`), an toàn để cắt lát
    /// `&text[start..end]` trực tiếp.
    ///
    /// Mẫu RỖNG ⇒ `Ok(vec![])` — no-op, giữ hành vi cũ của `pipeline.rs:667`. Mẫu regex
    /// khớp ĐỘ DÀI 0 (ví dụ `x*`) bị LỌC ở nguồn — cùng lý do
    /// `core::cleanup::byte_ranges_for` đã lọc cho luật làm sạch. Mẫu regex không biên dịch
    /// được ⇒ `Err`, không `panic!` (`panic = "abort"`).
    pub fn match_starts(&self, text: &str) -> Result<Vec<usize>, regex::Error> {
        if self.pattern.is_empty() {
            return Ok(Vec::new());
        }
        match self.kind {
            ChapterPatternKind::Literal => {
                Ok(text.match_indices(self.pattern.as_str()).map(|(i, _)| i).collect())
            }
            ChapterPatternKind::Regex => {
                let re = compile(&self.pattern)?;
                Ok(re.find_iter(text).filter(|m| !m.range().is_empty()).map(|m| m.start()).collect())
            }
        }
    }
}

/// Biên dịch một mẫu `regex` cho mẫu phân tách Chương — LUÔN đa dòng (`multi_line(true)`),
/// cùng khuôn `core::cleanup::compile_cleanup_regex` (`cleanup/mod.rs:230-232`). Chỗ gọi thử
/// biên dịch TRƯỚC khi chạy pipeline sống ở `commands::project` (mẫu hỏng bị từ chối Ở
/// NGUỒN, không tới đây trên đường sản phẩm với một mẫu hỏng).
pub fn compile(pattern: &str) -> Result<regex::Regex, regex::Error> {
    regex::RegexBuilder::new(pattern).multi_line(true).build()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_pattern_matches_nothing() {
        let p = ChapterPattern::literal("");
        assert_eq!(p.match_starts("bat ky van ban nao").unwrap(), Vec::<usize>::new());
    }

    #[test]
    fn literal_pattern_finds_every_occurrence() {
        let p = ChapterPattern::literal("第一章");
        let starts = p.match_starts("第一章 mot第一章 hai第一章 ba").unwrap();
        assert_eq!(starts.len(), 3);
    }

    #[test]
    fn regex_pattern_is_multi_line_and_matches_per_line() {
        let p = ChapterPattern::regex(r"^Chuong\s+\d+.*$");
        let starts = p.match_starts("Chuong 1 Mo Dau\nnoi dung\nChuong 2 Tiep Theo").unwrap();
        assert_eq!(starts.len(), 2, "^/$ phai neo theo TUNG DONG, khong theo toan van ban");
    }

    #[test]
    fn a_zero_length_regex_match_is_filtered_at_the_source() {
        let p = ChapterPattern::regex("x*");
        assert_eq!(p.match_starts("abcd").unwrap(), Vec::<usize>::new());
    }

    #[test]
    fn an_invalid_regex_is_rejected_not_panicking() {
        let p = ChapterPattern::regex("[unclosed");
        assert!(p.match_starts("bat ky van ban nao").is_err());
    }
}
