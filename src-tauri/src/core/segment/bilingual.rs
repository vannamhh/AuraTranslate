//! Bilingual two-column CSV/TSV row parsing — Story 6.16, FR115.
//!
//! ─────────────────────────────────────────────────────────────────────────────
//! PURE `&str` IN, TYPED ROWS OUT — 0 fs, 0 network, 0 panic points
//! ─────────────────────────────────────────────────────────────────────────────
//! Same shape as [`crate::core::segment::import`]'s input step: a module that only turns
//! characters into structured data. `parse_rows` reuses the tokenizer already proven by
//! Story 3.10 ([`crate::core::glossary::exchange`], made `pub(crate)` for this story) —
//! NFR15's precedent, one RFC 4180 subset, not a second copy.
//!
//! ─────────────────────────────────────────────────────────────────────────────
//! ROW IDENTITY SURVIVES TO THE SEGMENT SPLIT
//! ─────────────────────────────────────────────────────────────────────────────
//! `core::segment::pipeline::Step::SplitSegments` needs to know, for every pair of
//! sentence-split lists, which one grouping they came from (to compute the AD-37 row flag
//! and detect a mismatched row) — so a row is never flattened into a single joined column
//! anywhere in this module.

use crate::core::glossary::exchange::{ParseIssue, logical_lines, split_fields};

pub use crate::core::glossary::exchange::Delimiter;

/// One data row of a bilingual CSV/TSV file — the raw cells, split but not trimmed
/// (trimming a cell is `commands::project`'s job, same discipline as every other cell in
/// this module: this file only tokenizes).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BilingualRow {
    /// 1-based row number as a person counts it in a text editor/spreadsheet — counts the
    /// header row too, when one is present. This is the number the I/O Matrix's mismatch
    /// list points at.
    pub row_number: usize,
    /// Every cell of the row, in file order.
    pub cells: Vec<String>,
}

/// Every way [`parse_rows`] refuses a file whole — §I/O Matrix "Unterminated quote"/"Fewer
/// than 2 columns".
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BilingualParseIssue {
    /// A quoted cell opened but never closed — same structural failure
    /// [`crate::core::glossary::exchange::ParseIssue::UnterminatedQuotedField`] names for the
    /// Glossary's CSV/TSV, same reason it stops parsing instead of reporting a
    /// `CellCountMismatch` on the last line: nothing after the open quote resolved into rows.
    UnterminatedQuotedField {
        /// Row where the open quote started (1-based).
        row: usize,
    },
    /// The widest row has fewer than two columns — nothing to pick a source/target role
    /// from. Whole-file refusal, no row number (there is no one row at fault; the shape of
    /// the file itself has no second column).
    TooFewColumns {
        /// Column count found — the widest row's cell count.
        found: usize,
    },
    /// The shared tokenizer reported an issue this module never expects from it — today
    /// `logical_lines` only returns `UnterminatedQuotedField`. Typed instead of panicking: a
    /// file the user picked must never take the process down (spec 6.16: 0 panic points).
    UnexpectedTokenizerIssue {
        /// `ParseIssue`'s own log diagnostic (no diacritics).
        detail: String,
    },
}

impl std::fmt::Display for BilingualParseIssue {
    /// ⚠️ NO diacritics — log diagnostic (NFR16).
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BilingualParseIssue::UnterminatedQuotedField { row } => {
                write!(f, "bilingual import[row {row}]: quoted field opened but never closed")
            }
            BilingualParseIssue::TooFewColumns { found } => {
                write!(f, "bilingual import: {found} column(s) found, need at least 2")
            }
            BilingualParseIssue::UnexpectedTokenizerIssue { detail } => {
                write!(f, "bilingual import: unexpected tokenizer issue: {detail}")
            }
        }
    }
}

impl std::error::Error for BilingualParseIssue {}

/// Parse `text` into rows of cells, split on `delimiter`. Row numbers are 1-based and count
/// every logical row seen on disk — a blank logical row is skipped (same "trailing/interior
/// blank line is not a row" rule the Glossary's `parse` already applies, same reason: an
/// editor's trailing newline must not read as a data row), but its own line number is still
/// consumed so row numbers keep matching what a person counting lines in the file would see.
///
/// Does **not** know about headers or which column is source/target — that is a decision
/// [`commands::project`] makes with the *columns already parsed*, so toggling the header
/// checkbox or swapping columns never re-reads the file (§I/O Matrix "Header checkbox
/// on"/"Swap columns": the preview rebuilds in memory).
pub fn parse_rows(text: &str, delimiter: Delimiter) -> Result<Vec<BilingualRow>, BilingualParseIssue> {
    let d = delimiter.as_char();
    let lines = logical_lines(text, Some(d), 1).map_err(|issue| match issue {
        ParseIssue::UnterminatedQuotedField { line } => {
            BilingualParseIssue::UnterminatedQuotedField { row: line }
        }
        // `logical_lines` only ever returns `UnterminatedQuotedField` today — every other
        // `ParseIssue` variant belongs to the Glossary's column-aware `parse`. Still typed, not
        // `unreachable!`: a future tokenizer change must surface as an error, not a panic.
        other => BilingualParseIssue::UnexpectedTokenizerIssue { detail: other.to_string() },
    })?;

    let mut rows = Vec::with_capacity(lines.len());
    for (row_number, raw) in lines {
        if raw.trim().is_empty() {
            continue;
        }
        rows.push(BilingualRow { row_number, cells: split_fields(raw, d) });
    }
    Ok(rows)
}

/// Widest row's cell count — 0 when `rows` is empty. Used by the command layer to refuse a
/// file with fewer than two columns *before* a preview is built (§I/O Matrix "Fewer than 2
/// columns").
pub fn widest_row_column_count(rows: &[BilingualRow]) -> usize {
    rows.iter().map(|r| r.cells.len()).max().unwrap_or(0)
}

/// A segment paired with its bilingual translation, ready to write — Story 6.16, AD-47 ③.
/// Distinct from [`crate::core::segment::split::SplitSegment`]: that type carries ONE flag
/// (the source flag, AD-37) because on every other import path the target starts out as a
/// mirror of the source (AD-46, AC2 of Story 2.5d) written later, one column at a time. A
/// bilingual row already HAS its own target text at import time, so this type carries both
/// texts and the ALREADY-MIRRORED flag (AD-46: at import, target flag = source flag) in one
/// place — the row-flag rule below computes both together, not by mirroring after the fact.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BilingualSegment {
    pub source_text: String,
    pub target_text: String,
    /// AD-37 flag, mirrored for both `is_paragraph_end`/`is_target_paragraph_end` at
    /// `commands::segment::insert_bilingual_segments` (AD-46: they start equal).
    pub is_paragraph_end: bool,
}

/// One mismatched row — §I/O Matrix "Mismatched row"/"Blank target cell". `chapter_index` is
/// 0-based (position in the import's chapter list); `row_number` is the 1-based file row
/// number [`BilingualRow::row_number`] already carries, echoed here so the preview UI can
/// list "Chapter N, row M" without re-walking rows.
///
/// **THÊM 2026-09-12 (Story 6.17, FR116)** — bốn trường sau carrying đủ dữ kiện cho preview
/// dựng màn regroup KHÔNG cần đi hỏi lại Rust cho mỗi lượt render: `source_sentences` là câu
/// nguồn ĐÃ tách (đơn vị bắt buộc, không đổi bởi regroup — §Never "Target side only");
/// `target_line` là bản dịch ĐÃ tách rồi NỐI LẠI bằng một dấu cách (xem [`join_target_line`]) —
/// đây là chuỗi mà mọi tập `cuts` của [`BilingualRegroupingAction::Cuts`] cắt vào;
/// `candidate_positions` là MỌI điểm hợp lệ để bật/tắt một chỗ cắt hay dừng caret (ranh giới
/// máy cộng ranh giới từ — xem [`candidate_positions`]); `initial_cuts` là tập cắt DUY NHẤT
/// khớp đúng cách máy đã tách `target_line` (== [`machine_boundaries`]), trạng thái ban đầu
/// trước khi người dùng chạm gì; `proposed_cuts` là gợi ý một lượt của [`propose_cuts`].
///
/// **THÊM (vòng rà đối kháng) — `target_sentence_count`.** Số câu đích MÁY đã tách
/// (`target_sentences.len()`), tính Ở ĐÂY chứ không suy từ `initial_cuts.len() + 1` phía
/// TypeScript: hai con số đó CHỈ trùng nhau khi `target_sentences` không rỗng — một hàng
/// "Skip blank target" (0 câu đích) cho `initial_cuts = []`, và `0 + 1 = 1` là một suy diễn
/// SAI (đúng phải là `0`). §I/O Matrix "Skip blank source" cần con số THẬT này để hiện "còn
/// bao nhiêu câu đích bị bỏ" — dựng lại nó ở TypeScript bằng phép cộng trên sẽ chép một luật
/// đã sống ở Rust, và chép sai đúng ca biên đáng lẽ trường này phải tránh.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BilingualMismatch {
    pub chapter_index: usize,
    pub row_number: usize,
    pub source_sentences: Vec<String>,
    pub target_line: String,
    pub target_sentence_count: usize,
    pub candidate_positions: Vec<usize>,
    pub initial_cuts: Vec<usize>,
    pub proposed_cuts: Vec<usize>,
}

// ═════════════════════════════════════════════════════════════════════════════════
// Story 6.17 (FR116) — quy nhóm lại câu đích trong một hàng lệch cặp, TRƯỚC khi ghi
// ═════════════════════════════════════════════════════════════════════════════════
//
// ─────────────────────────────────────────────────────────────────────────────
// 🔴 HÀM THUẦN — 0 fs, 0 network, 0 điểm panic, cùng kỷ luật đầu tệp
// ─────────────────────────────────────────────────────────────────────────────
// Một lượt quy nhóm chỉ là: tách lại (đo trước khi tin), so khớp với ảnh chụp người dùng đã
// thấy (staleness), rồi cắt `target_line` tại các chỉ số KÝ TỰ UNICODE — cùng kỷ luật
// `regroup::split_at` đã theo cho gộp/tách segment: `chars()` không phải byte, `get` thay chỉ
// số tràn, sắp lại tại chỗ, từ chối 0/≥ độ dài/trùng nhau. Khác `regroup::split_at` ở một
// điểm cấu trúc: 0 segment nào tồn tại lúc này (§Always AD-5 "0 segments exist yet, so no
// retire + create") nên không cờ kết đoạn/xuất xứ nào cần tính ở đây — `pipeline.rs` gán
// chúng theo VỊ TRÍ HÀNG sau khi hàm này trả về, đúng cách nó đã làm cho hàng cặp được.

/// Câu nguồn/đích ĐÃ TÁCH LẠI của một hàng, ngay lúc đọc — nguyên liệu cho cả staleness lẫn
/// đề xuất. `target_line` là NỐI của `target_sentences` bằng một dấu cách (Design Notes:
/// "Canonical target line").
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DerivedRow {
    pub source_sentences: Vec<String>,
    pub target_sentences: Vec<String>,
    pub target_line: String,
}

/// Nối các câu đích MÁY đã tách thành MỘT dòng — dấu nối LUÔN là một dấu cách, không hỏi
/// `source_lang`: `target_line` luôn là tiếng Việt (`crate::core::dict::NATIVE_LANG`), một
/// ngôn ngữ dùng dấu cách, và nó là bản dịch — không phải văn bản nguồn mà `regroup::source_joiner`
/// phục vụ.
#[must_use]
pub fn join_target_line(target_sentences: &[String]) -> String {
    target_sentences.join(" ")
}

/// Tách lại CẢ HAI ô của một hàng — nguồn theo `source_lang` (AD-18: trường bất biến của Tác
/// phẩm), đích luôn tiếng Việt (cùng nhánh `NATIVE_LANG` mà `pipeline::split_segments_step`
/// đã dùng cho hàng cặp được). Đo trước khi tin — không nhớ lại kết quả cũ.
#[must_use]
pub fn derive_row(source_cell: &str, target_cell: &str, source_lang: &str) -> DerivedRow {
    let source_sentences: Vec<String> = super::split::split_source_text(source_cell, source_lang)
        .into_iter()
        .map(|s| s.text)
        .collect();
    let target_sentences: Vec<String> =
        super::split::split_source_text(target_cell, crate::core::dict::NATIVE_LANG)
            .into_iter()
            .map(|s| s.text)
            .collect();
    let target_line = join_target_line(&target_sentences);
    DerivedRow { source_sentences, target_sentences, target_line }
}

/// Ranh giới MÁY của `target_line` — chỉ số ký tự nơi hai câu đích máy đã tách liền nhau
/// (KHÔNG gồm hai đầu `0`/độ dài). Đây là bộ cắt DUY NHẤT khớp đúng cách
/// [`super::split::split_source_text`] đã tách `target_sentences`: cắt đúng các vị trí này rồi
/// trim mỗi mảnh trả lại nguyên `target_sentences` — [`initial_cuts`] của một
/// [`BilingualMismatch`] LÀ chính danh sách này.
#[must_use]
pub fn machine_boundaries(target_sentences: &[String]) -> Vec<usize> {
    let mut out = Vec::new();
    let mut pos = 0usize;
    for (i, s) in target_sentences.iter().enumerate() {
        pos += s.chars().count();
        if i + 1 < target_sentences.len() {
            out.push(pos);
            pos += 1; // dấu cách nối, xem `join_target_line`.
        }
    }
    out
}

/// Ranh giới TỪ của `target_line` — mọi điểm chuyển giữa một run khoảng trắng và một run
/// không-khoảng-trắng (hai đầu `0`/độ dài không tính, không có gì để mà chuyển tại đó).
///
/// ⚠️ **Không mượn `unicode-segmentation`** dù crate đó đã có sẵn trong cây phụ thuộc
/// (`split.rs` đã ghi lý do cho ranh giới CÂU) — thêm nó vào `Cargo.toml` làm trực tiếp là một
/// dependency MỚI, thứ spec 6.17 cấm (§Never "No new crate"). `target_line` LUÔN là tiếng Việt
/// (`join_target_line`) — một ngôn ngữ dùng dấu cách phân tách âm tiết — nên một ranh giới
/// khoảng trắng ĐÃ LÀ một ranh giới từ thật, không phải một phép gần đúng vá cho ngôn ngữ
/// không dấu cách.
#[must_use]
pub fn word_boundaries(target_line: &str) -> Vec<usize> {
    let chars: Vec<char> = target_line.chars().collect();
    let mut out = Vec::new();
    for i in 1..chars.len() {
        if chars[i - 1].is_whitespace() != chars[i].is_whitespace() {
            out.push(i);
        }
    }
    out
}

/// Mọi điểm HỢP LỆ để bật/tắt một chỗ cắt hay dừng caret trên `target_line` — hợp của
/// [`machine_boundaries`] và [`word_boundaries`], sắp tăng dần và không trùng. Đây là danh
/// sách DUY NHẤT webview cần để di caret bằng `← →` (AD-1: không luật kinh doanh nào sống ở
/// TypeScript — kể cả một luật "ranh giới từ" tưởng vô hại).
#[must_use]
pub fn candidate_positions(target_sentences: &[String], target_line: &str) -> Vec<usize> {
    let mut set: Vec<usize> = machine_boundaries(target_sentences);
    for wb in word_boundaries(target_line) {
        if !set.contains(&wb) {
            set.push(wb);
        }
    }
    set.sort_unstable();
    set.dedup();
    set
}

/// Đề xuất MỘT tập cắt cho `target_line` — `source_sentences.len() - 1` chỗ cắt, mỗi chỗ là
/// điểm ứng viên ([`machine_boundaries`] trước, [`word_boundaries`] sau — cùng thứ tự ưu tiên
/// [`candidate_positions`] dựng) GẦN NHẤT vị trí mà độ dài các câu nguồn ngụ ý (câu nguồn dài
/// hơn ⇒ mảnh đích tương ứng CŨNG dài hơn theo cùng tỉ lệ). Cùng MỘT luật cho gộp lẫn tách
/// (Design Notes: "the same rule for a join and for a split") — không nhánh `if` nào phân biệt
/// hai chiều, chỉ số lượng cắt cần khác nhau.
///
/// Chỉ là một điểm KHỞI ĐẦU hiện trong preview — không tự áp dụng (§Always "applied only by an
/// explicit act"). Khi `target_line` không đủ điểm ứng viên (câu tiếng Việt không dấu cách nào,
/// hiếm nhưng có thể), kết quả có thể NGẮN HƠN số cắt cần — người dùng tự bù nốt bằng caret.
#[must_use]
pub fn propose_cuts(source_sentences: &[String], target_sentences: &[String]) -> Vec<usize> {
    let needed = source_sentences.len().saturating_sub(1);
    if needed == 0 {
        return Vec::new();
    }
    let target_line = join_target_line(target_sentences);
    let target_len = target_line.chars().count();
    if target_len < 2 {
        return Vec::new();
    }
    let total_source_len: usize =
        source_sentences.iter().map(|s| s.chars().count()).sum::<usize>().max(1);

    let mut candidates: Vec<usize> = machine_boundaries(target_sentences);
    for wb in word_boundaries(&target_line) {
        if !candidates.contains(&wb) {
            candidates.push(wb);
        }
    }

    let mut used: Vec<usize> = Vec::with_capacity(needed);
    let mut cum_source = 0usize;
    for source_sentence in source_sentences.iter().take(needed) {
        cum_source += source_sentence.chars().count();
        let ideal_f = (target_len as f64) * (cum_source as f64) / (total_source_len as f64);
        let ideal = (ideal_f.round() as isize).clamp(1, (target_len - 1) as isize) as usize;
        let pick = candidates
            .iter()
            .copied()
            .filter(|c| !used.contains(c))
            .min_by_key(|&c| (c as isize - ideal as isize).unsigned_abs());
        if let Some(p) = pick {
            used.push(p);
        }
    }
    used.sort_unstable();
    used
}

/// Cắt `target_line` tại `cuts` (chỉ số KÝ TỰ UNICODE) — trả `n + 1` mảnh ĐÃ TRIM, hoặc `None`
/// khi bất kỳ chỗ cắt nào KHÔNG hợp lệ: `0`, ở cuối/ngoài chuỗi, trùng nhau, hay để lại một
/// mảnh RỖNG sau khi trim. Cùng kỷ luật `regroup::split_at`: `chars()` không byte (một chỉ số
/// byte rơi giữa một ký tự nhiều byte làm `str::split_at` panic, mà `panic = "abort"` giết cả
/// tiến trình), sắp `cuts` tại chỗ, `get` thay chỉ số tràn.
///
/// `cuts` RỖNG là một lượt hợp lệ (một mảnh duy nhất = `target_line` đã trim) — khác
/// `regroup::split_at`, nơi rỗng nghĩa là *"không có lượt tách nào được yêu cầu"*: ở đây một
/// hàng nguồn CHỈ MỘT câu hợp lệ khi đích cũng gộp về một mảnh, và đó chính là `cuts = []`.
#[must_use]
pub fn apply_cuts(target_line: &str, cuts: &[usize]) -> Option<Vec<String>> {
    let chars: Vec<char> = target_line.chars().collect();
    let mut sorted: Vec<usize> = cuts.to_vec();
    sorted.sort_unstable();
    if sorted.windows(2).any(|w| w[0] == w[1]) {
        return None;
    }
    if sorted.first().is_some_and(|&c| c == 0) || sorted.last().is_some_and(|&c| c >= chars.len()) {
        return None;
    }

    let mut bounds: Vec<usize> = Vec::with_capacity(sorted.len() + 2);
    bounds.push(0);
    bounds.extend_from_slice(&sorted);
    bounds.push(chars.len());

    let mut out = Vec::with_capacity(bounds.len().saturating_sub(1));
    for w in bounds.windows(2) {
        let (Some(&start), Some(&end)) = (w.first(), w.get(1)) else { return None };
        let piece: String = chars.get(start..end)?.iter().collect();
        let trimmed = piece.trim().to_owned();
        if trimmed.is_empty() {
            return None;
        }
        out.push(trimmed);
    }
    Some(out)
}

/// Quyết định người dùng cho MỘT hàng lệch cặp — cắt, hoặc bỏ qua.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BilingualRegroupingAction {
    /// Chỉ số KÝ TỰ UNICODE vào [`BilingualRegrouping::target_line`] — xem [`apply_cuts`].
    Cuts(Vec<usize>),
    /// "Bỏ qua hàng này" — chỉ hợp lệ khi một trong hai phía có 0 câu (§Never: "No skip on a
    /// row whose two sides both have at least one sentence").
    Skip,
}

/// Một lượt quy nhóm người dùng đã làm cho MỘT hàng — mang theo ẢNH CHỤP của hai ô lúc lượt
/// quy nhóm được tạo, để [`resolve`] so khớp trước khi áp (staleness — Design Notes: "Echoing
/// the two texts is the staleness check").
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BilingualRegrouping {
    pub row_number: usize,
    pub source_sentences: Vec<String>,
    pub target_line: String,
    pub action: BilingualRegroupingAction,
}

/// Kết quả MỘT hàng đã được [`resolve`] chấp nhận.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RegroupResolution {
    /// Cặp được — `(source_text, target_text)` theo đúng thứ tự câu nguồn; `target_text` rỗng
    /// cho một mảnh "chưa dịch" (đường Skip nguồn > 0/đích = 0).
    Paired(Vec<(String, String)>),
    /// Bỏ qua — 0 segment đóng góp từ hàng này (đường Skip nguồn = 0).
    Skipped,
}

/// Lỗi TYPED của [`resolve`] — khác "hàng vẫn ở lại danh sách lệch cặp" ([`resolve`] trả
/// `Ok(None)` cho ca đó): đây LÀ một vi phạm hợp đồng thật của webview (bỏ qua một hàng mà cả
/// hai phía đều có câu), và §I/O Matrix "Skip refused" đòi một lỗi TYPED, không phải một hàng
/// lặng lẽ ở lại danh sách.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BilingualSkipNotAllowed {
    pub row_number: usize,
}

/// Áp MỘT lượt quy nhóm lên hàng hiện tại của tệp (`source_cell`/`target_cell` ĐANG SỐNG,
/// không phải ảnh chụp) — hàm thuần DUY NHẤT mà cả `rebuild`/`confirm` VÀ chuỗi pipeline gọi
/// (Task list: "one rule for rebuild and confirm").
///
/// # Trả về
/// - `Ok(Some(Paired(..)))` — quy nhóm hợp lệ VÀ khớp ảnh chụp, hàng cặp được.
/// - `Ok(Some(Skipped))` — bỏ qua hợp lệ (nguồn = 0 câu).
/// - `Ok(None)` — hàng VẪN lệch cặp: ảnh chụp đã CŨ (cột đổi/bảng mã đổi giữa chừng — §I/O
///   Matrix "Stale regrouping"), hoặc `Cuts` không hợp lệ (chỗ cắt tồi, hay không ra đúng số
///   mảnh cần — §I/O Matrix "Bad cut on wire"/"Still unequal").
/// - `Err(BilingualSkipNotAllowed)` — `Skip` được gửi cho một hàng mà CẢ HAI phía đều có câu.
#[must_use]
pub fn resolve(
    source_cell: &str,
    target_cell: &str,
    source_lang: &str,
    regrouping: &BilingualRegrouping,
) -> Result<Option<RegroupResolution>, BilingualSkipNotAllowed> {
    let derived = derive_row(source_cell, target_cell, source_lang);
    if derived.source_sentences != regrouping.source_sentences || derived.target_line != regrouping.target_line
    {
        return Ok(None); // ảnh chụp cũ — xem §I/O Matrix "Stale regrouping".
    }

    match &regrouping.action {
        BilingualRegroupingAction::Skip => {
            let source_empty = derived.source_sentences.is_empty();
            let target_empty = derived.target_sentences.is_empty();
            if source_empty && !target_empty {
                Ok(Some(RegroupResolution::Skipped))
            } else if !source_empty && target_empty {
                Ok(Some(RegroupResolution::Paired(
                    derived.source_sentences.into_iter().map(|s| (s, String::new())).collect(),
                )))
            } else {
                Err(BilingualSkipNotAllowed { row_number: regrouping.row_number })
            }
        }
        BilingualRegroupingAction::Cuts(cuts) => {
            if derived.source_sentences.is_empty() {
                return Ok(None); // 0 câu nguồn — `Cuts` vô nghĩa, chỉ `Skip` giải quyết được.
            }
            let Some(pieces) = apply_cuts(&derived.target_line, cuts) else {
                return Ok(None);
            };
            if pieces.len() != derived.source_sentences.len() {
                return Ok(None);
            }
            Ok(Some(RegroupResolution::Paired(derived.source_sentences.into_iter().zip(pieces).collect())))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn splits_csv_rows_by_comma() {
        let rows = parse_rows("a,b\nc,d\n", Delimiter::Csv).unwrap();
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0].row_number, 1);
        assert_eq!(rows[0].cells, vec!["a".to_owned(), "b".to_owned()]);
        assert_eq!(rows[1].row_number, 2);
        assert_eq!(rows[1].cells, vec!["c".to_owned(), "d".to_owned()]);
    }

    #[test]
    fn splits_tsv_rows_by_tab() {
        let rows = parse_rows("a\tb\nc\td\n", Delimiter::Tsv).unwrap();
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0].cells, vec!["a".to_owned(), "b".to_owned()]);
    }

    #[test]
    fn blank_rows_are_skipped_but_still_count_line_numbers() {
        let rows = parse_rows("a,b\n\nc,d\n", Delimiter::Csv).unwrap();
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0].row_number, 1);
        assert_eq!(rows[1].row_number, 3);
    }

    #[test]
    fn unterminated_quote_is_refused_with_its_row_number() {
        let err = parse_rows("a,b\n\"c,d\n", Delimiter::Csv).unwrap_err();
        assert_eq!(err, BilingualParseIssue::UnterminatedQuotedField { row: 2 });
    }

    #[test]
    fn widest_row_wins_the_column_count() {
        let rows = parse_rows("a,b\nc,d,e\n", Delimiter::Csv).unwrap();
        assert_eq!(widest_row_column_count(&rows), 3);
    }

    #[test]
    fn empty_text_has_0_columns() {
        assert_eq!(widest_row_column_count(&[]), 0);
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // Story 6.17 — quy nhóm câu đích
    // ═══════════════════════════════════════════════════════════════════════════

    fn reg(row_number: usize, source: &[&str], target_line: &str, action: BilingualRegroupingAction) -> BilingualRegrouping {
        BilingualRegrouping {
            row_number,
            source_sentences: source.iter().map(|s| s.to_string()).collect(),
            target_line: target_line.to_owned(),
            action,
        }
    }

    #[test]
    fn split_target_resolves_a_2_vs_1_row_at_a_word_boundary() {
        // Source 2, target 1; one cut at a word boundary.
        let r = reg(1, &["He left.", "She smiled."], "He left she smiled", BilingualRegroupingAction::Cuts(vec![8]));
        let out = resolve("He left. She smiled.", "He left she smiled", "en", &r).unwrap();
        assert_eq!(
            out,
            Some(RegroupResolution::Paired(vec![
                ("He left.".to_owned(), "He left".to_owned()),
                ("She smiled.".to_owned(), "she smiled".to_owned()),
            ]))
        );
    }

    #[test]
    fn join_target_resolves_a_2_vs_3_row_by_removing_a_machine_boundary() {
        // Source 2, target 3; boundary t1/t2 removed -> first target "t1 t2".
        let source_cell = "One. Two.";
        let target_cell = "Mot. Hai. Ba.";
        let derived = derive_row(source_cell, target_cell, "en");
        assert_eq!(derived.target_sentences, vec!["Mot.", "Hai.", "Ba."]);
        let boundaries = machine_boundaries(&derived.target_sentences);
        assert_eq!(boundaries.len(), 2);
        // Chi giu chỗ cắt THỨ HAI (giữa "Hai." và "Ba.") -- gộp "Mot." + "Hai." thành mảnh đầu.
        let r = reg(1, &["One.", "Two."], &derived.target_line, BilingualRegroupingAction::Cuts(vec![boundaries[1]]));
        let out = resolve(source_cell, target_cell, "en", &r).unwrap();
        assert_eq!(
            out,
            Some(RegroupResolution::Paired(vec![
                ("One.".to_owned(), "Mot. Hai.".to_owned()),
                ("Two.".to_owned(), "Ba.".to_owned()),
            ]))
        );
    }

    #[test]
    fn still_unequal_after_edits_stays_a_mismatch() {
        // Source 2, target pieces 3 after edits -> Ok(None), row still listed.
        let derived = derive_row("One. Two.", "Mot. Hai. Ba.", "en");
        let r = reg(1, &["One.", "Two."], &derived.target_line, BilingualRegroupingAction::Cuts(vec![3, 8]));
        let out = resolve("One. Two.", "Mot. Hai. Ba.", "en", &r).unwrap();
        assert_eq!(out, None);
    }

    #[test]
    fn a_bad_cut_on_the_wire_never_panics_and_leaves_the_row_unresolved() {
        let derived = derive_row("One. Two.", "Mot hai", "en");
        for bad in [vec![0usize], vec![100usize], vec![3, 3]] {
            let r = reg(1, &["One.", "Two."], &derived.target_line, BilingualRegroupingAction::Cuts(bad));
            assert_eq!(resolve("One. Two.", "Mot hai", "en", &r).unwrap(), None);
        }
    }

    #[test]
    fn a_cut_landing_inside_a_multi_byte_character_never_panics() {
        // Moi ky tu Han duoi day chiem 3 byte UTF-8 -- mot chi so BYTE (thay vi KY TU) roi giua
        // mot ky tu se lam `str::split_at` panic (`panic = "abort"` giet ca tien trinh). Test
        // nay khoa rang KHONG panic voi moi chi so hop le/khong hop le, vi `apply_cuts` luon
        // gom ky tu qua `chars()` truoc khi cat.
        let derived = derive_row("One. Two.", "你好嗎再見", "en");
        for cut in 0..=10usize {
            let r = reg(1, &["One.", "Two."], &derived.target_line, BilingualRegroupingAction::Cuts(vec![cut]));
            let _ = resolve("One. Two.", "你好嗎再見", "en", &r);
        }
    }

    #[test]
    fn a_stale_regrouping_after_a_column_swap_is_dropped_and_the_row_stays_listed() {
        let derived = derive_row("One. Two.", "Mot hai", "en");
        let r = reg(1, &["One.", "Two."], &derived.target_line, BilingualRegroupingAction::Cuts(vec![3]));
        // Encoding/column change alters the row underneath the snapshot.
        let out = resolve("One. Two. Three.", "Mot hai ba", "en", &r).unwrap();
        assert_eq!(out, None, "anh chup cu phai bi tha, khong ap oan len hang da doi");
    }

    #[test]
    fn a_stale_regrouping_whose_cut_count_still_fits_is_refused_by_the_snapshot_alone() {
        // Ca DUY NHAT chi phep so ANH CHUP bac duoc: hang da doi van co 2 cau nguon, va cho cat
        // cu (3) van cho dung 2 manh khong rong tren dong dich MOI. Moi hang rao khac -- so manh
        // != so cau nguon, manh rong, chi so tran -- deu IM LANG o day, nen neu phep so anh chup
        // bi go, ban dich cua nguoi dung se bi cat theo mot quyet dinh lam tren VAN BAN KHAC.
        let r = reg(1, &["One.", "Two."], "Mot hai", BilingualRegroupingAction::Cuts(vec![3]));
        let derived = derive_row("Aaa. Bbb.", "Xyz qwe", "en");
        assert_eq!(derived.source_sentences.len(), 2, "hang MOI van co dung 2 cau nguon");
        assert_eq!(apply_cuts(&derived.target_line, &[3]).map(|p| p.len()), Some(2), "cho cat cu van cho 2 manh");

        let out = resolve("Aaa. Bbb.", "Xyz qwe", "en", &r).unwrap();
        assert_eq!(out, None, "van ban da doi thi quy nhom cu phai bi tha, du so manh van khop");
    }

    #[test]
    fn an_unaffected_regrouping_still_resolves_after_an_unrelated_header_toggle() {
        let derived = derive_row("One. Two.", "Mot hai", "en");
        let r = reg(1, &["One.", "Two."], &derived.target_line, BilingualRegroupingAction::Cuts(vec![3]));
        // Cung hai o dau vao -- mot header toggle o hang KHAC khong lam anh chup cu.
        let out = resolve("One. Two.", "Mot hai", "en", &r).unwrap();
        assert!(out.is_some());
    }

    #[test]
    fn a_proposal_is_shown_for_a_2_vs_1_row_and_nothing_pairs_until_accepted() {
        let derived = derive_row("He left. She smiled.", "He left she smiled", "en");
        let proposal = propose_cuts(&derived.source_sentences, &derived.target_sentences);
        assert_eq!(proposal.len(), 1, "source 2, target 1 -> dung 1 cho cat de xuat");
        // Chinh no la mot regrouping HOP LE -- nhung chua duoc ap (khong goi resolve o day).
        let r = reg(1, &derived.source_sentences.iter().map(String::as_str).collect::<Vec<_>>(), &derived.target_line, BilingualRegroupingAction::Cuts(proposal));
        assert!(resolve("He left. She smiled.", "He left she smiled", "en", &r).unwrap().is_some());
    }

    #[test]
    fn skip_blank_target_yields_one_untranslated_segment_per_source_sentence() {
        // Heading row 1-vs-0, skipped.
        let r = reg(1, &["Chuong Mot"], "", BilingualRegroupingAction::Skip);
        let out = resolve("Chuong Mot", "", "en", &r).unwrap();
        assert_eq!(out, Some(RegroupResolution::Paired(vec![("Chuong Mot".to_owned(), String::new())])));
    }

    // 🔴 THÊM (vòng rà đối kháng) — moi ca Skip blank-target khac trong tep nay chi dung MOT
    // cau nguon, nen mot bien the noi het ca nhom lai thanh MOT cap van xanh. Ca nay dung HAI
    // cau nguon de khoa dung mENH DE "moi cau nguon MOT cap rieng", khong phai "gop lai".
    #[test]
    fn skip_blank_target_with_a_two_sentence_source_fans_out_two_untranslated_pairs() {
        let r = reg(1, &["One.", "Two."], "", BilingualRegroupingAction::Skip);
        let out = resolve("One. Two.", "", "en", &r).unwrap();
        assert_eq!(
            out,
            Some(RegroupResolution::Paired(vec![
                ("One.".to_owned(), String::new()),
                ("Two.".to_owned(), String::new()),
            ])),
            "hai cau nguon phai cho HAI cap rieng, khong gop thanh mot"
        );
    }

    #[test]
    fn skip_blank_source_drops_the_translation_entirely() {
        // 0-vs-2 row, skipped -> 0 segments from the row.
        let derived = derive_row("", "Mot. Hai.", "en");
        let r = reg(1, &[], &derived.target_line, BilingualRegroupingAction::Skip);
        let out = resolve("", "Mot. Hai.", "en", &r).unwrap();
        assert_eq!(out, Some(RegroupResolution::Skipped));
    }

    #[test]
    fn skip_is_refused_with_a_typed_error_when_both_sides_have_sentences() {
        let r = reg(1, &["One.", "Two."], "Mot.", BilingualRegroupingAction::Skip);
        let err = resolve("One. Two.", "Mot.", "en", &r).unwrap_err();
        assert_eq!(err, BilingualSkipNotAllowed { row_number: 1 });
    }

    #[test]
    fn candidate_positions_merge_machine_and_word_boundaries_sorted_and_deduped() {
        let target_sentences = vec!["Mot".to_owned(), "hai ba".to_owned()];
        let target_line = join_target_line(&target_sentences);
        assert_eq!(target_line, "Mot hai ba");
        let positions = candidate_positions(&target_sentences, &target_line);
        // "Mot hai ba" -- ranh gioi may: chi so 3 (ngay sau "Mot"). Ranh gioi tu: hai dau cua
        // MOI khoang trang (vao va ra) -- 3/4 quanh khoang trang dau, 7/8 quanh khoang trang
        // sau. Hop + sap + khu trung: [3, 4, 7, 8].
        assert_eq!(positions, vec![3, 4, 7, 8]);
    }

    #[test]
    fn a_proposal_cuts_at_the_candidate_boundary_nearest_the_source_length_ratio() {
        // Cau nguon DAU rat ngan, cau sau rat dai => cho cat de xuat phai nam SAT dau dong dich,
        // tai mot ranh gioi TU that. Mot luat "cat giua dong" cho ra cho cat khac han, va mot
        // luat "cat bat ky dau" cat roi VAO GIUA mot tu -- ca hai deu bi ca nay bat.
        let long_b = "b".repeat(22);
        let source_sentences = vec!["A.".to_owned(), format!("B{}.", "b".repeat(21))];
        let target_sentences = vec![format!("aa {long_b}")];

        let cuts = propose_cuts(&source_sentences, &target_sentences);
        assert_eq!(cuts, vec![2], "cho cat phai la ranh gioi tu ngay sau \"aa\", khong phai trung diem dong");

        let pieces = apply_cuts(&join_target_line(&target_sentences), &cuts).expect("cho cat de xuat phai hop le");
        assert_eq!(pieces, vec!["aa".to_owned(), long_b]);
    }

    #[test]
    fn apply_cuts_with_no_cuts_yields_the_whole_trimmed_line_as_one_piece() {
        assert_eq!(apply_cuts("  ca cau  ", &[]), Some(vec!["ca cau".to_owned()]));
    }

    #[test]
    fn apply_cuts_rejects_a_cut_that_leaves_an_empty_piece() {
        assert_eq!(apply_cuts("ab cd", &[0]), None);
        assert_eq!(apply_cuts("ab cd", &[5]), None);
        assert_eq!(apply_cuts("ab cd", &[2, 2]), None);
        assert_eq!(apply_cuts("ab  cd", &[2, 3]), None, "manh giua hai cho cat lien nhau rong sau trim");
    }
}
