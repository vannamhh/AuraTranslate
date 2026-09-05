//! Luật làm sạch văn bản lúc nhập — Story 6.5, AD-18 (`ScopeKind::ImportCleanupRule`,
//! `Semantics::Merge`), FR124.
//!
//! ─────────────────────────────────────────────────────────────────────────────
//! 🔴 VÌ SAO MODULE NÀY THUẦN, KHÔNG SỐNG TRONG `core::segment::`
//! ─────────────────────────────────────────────────────────────────────────────
//! `core::segment::` phải ở lại thuần văn bản (`segment_boundary.rs::the_splitter_stays_pure`).
//! Bảng luật phụ thuộc `Store` + `ScopeResolver` (trạng thái NGOÀI chuỗi đã giải mã) — đúng
//! điểm phân biệt mà §Design Notes của spec 6.5 dùng để từ chối đặt kho ở `core/segment/`.
//! [`apply`] ở tệp NÀY thuần tuyệt đối (văn bản + luật ĐÃ phân giải vào, không đọc gì khác);
//! [`store`] là nơi duy nhất module này chạm `Store`/`ScopeResolver`.
//!
//! ─────────────────────────────────────────────────────────────────────────────
//! 🔴 DANH TÍNH MỘT LUẬT LÀ CẶP `(tier, id)` — KHÔNG PHẢI `id` TRẦN
//! ─────────────────────────────────────────────────────────────────────────────
//! `import_cleanup_rule` dùng `id INTEGER PRIMARY KEY AUTOINCREMENT` ở CẢ HAI kho
//! (`global.db`/`project.db`) — hai tầng đánh số ĐỘC LẬP, nên luật Toàn cục #1 và luật Tác
//! phẩm #1 cùng tồn tại. Bật/tắt/sửa/xoá một luật LUÔN đi kèm [`CleanupRuleTier`], không
//! bao giờ chỉ một `id`.
//!
//! 🔴 **Kiểu nhãn tầng RIÊNG — không tái dùng `core::scope::Tier`.** Cùng lý lẽ
//! `core::glossary::GlossaryTier`: kiểu này còn đi tiếp một chặng mà `Tier` không đi (dữ
//! liệu TRÊN DÂY của `commands::cleanup`/`commands::project`), nên nó không được phép phụ
//! thuộc vào cách `core::scope` biểu diễn hai tầng.
//!
//! ─────────────────────────────────────────────────────────────────────────────
//! 🔴 ĐẾM CHO MỌI LUẬT, XOÁ CHỈ LUẬT ĐANG BẬT
//! ─────────────────────────────────────────────────────────────────────────────
//! [`apply`] tính `per_rule_counts` cho **MỌI** luật (kể cả luật đã tắt) trên **TOÀN** văn
//! bản — "tắt đổi việc xoá, không đổi việc đo" (§Always spec 6.5). Chỉ luật `enabled` mới
//! góp phần vào việc xoá khỏi `text` trả về. Khớp chồng nhau giữa nhiều luật xoá đúng MỘT
//! lần (hợp các dải byte, không khử trùng lặp phép ĐẾM — AD-18 hợp nhất).

pub mod store;

use std::collections::BTreeMap;
use std::fmt;

pub use store::{
    CleanupRuleRow, CleanupStoreError, add_rule, delete_rule, edit_rule, list_tier,
    resolve_two_tiers, set_enabled,
};

/// Nhãn tầng của MỘT luật làm sạch — xem doc-comment đầu module cho lý do kiểu RIÊNG.
///
/// 🔵 **`serde::Deserialize`, `rename` TỪNG biến thể** — cùng khuôn `GlossaryTier`: tham số
/// `tier` của các lệnh `cleanup.*` được Tauri giải mã trực tiếp thành kiểu này, không qua
/// một hàm `from_wire` viết tay.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Deserialize)]
pub enum CleanupRuleTier {
    /// `global.db` — áp cho mọi Tác phẩm.
    #[serde(rename = "global")]
    Global,
    /// `project.db` của Tác phẩm đang mở.
    #[serde(rename = "work")]
    Work,
}

impl CleanupRuleTier {
    /// Định danh máy đọc — thứ đi trên dây. Không phải nhãn hiển thị (AD-21, NFR16).
    pub const fn as_str(self) -> &'static str {
        match self {
            CleanupRuleTier::Global => "global",
            CleanupRuleTier::Work => "work",
        }
    }

    /// Phân giải một giá trị đến từ dây. `None` cho mọi chuỗi không khớp — không đoán.
    pub fn from_wire(raw: &str) -> Option<Self> {
        match raw {
            "global" => Some(CleanupRuleTier::Global),
            "work" => Some(CleanupRuleTier::Work),
            _ => None,
        }
    }
}

impl fmt::Display for CleanupRuleTier {
    /// KHÔNG DẤU — chẩn đoán cho log, không phải văn bản hiển thị (NFR16).
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Hai hình dạng mẫu — chuỗi con nguyên văn hay biểu thức chính quy (crate `regex`, rà
/// NFR15 ở spine §Stack).
///
/// 🔵 `serde::Deserialize`, cùng lý do [`CleanupRuleTier`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Deserialize)]
pub enum CleanupRuleKind {
    #[serde(rename = "literal")]
    Literal,
    #[serde(rename = "regex")]
    Regex,
}

impl CleanupRuleKind {
    pub const fn as_str(self) -> &'static str {
        match self {
            CleanupRuleKind::Literal => "literal",
            CleanupRuleKind::Regex => "regex",
        }
    }

    pub fn from_wire(raw: &str) -> Option<Self> {
        match raw {
            "literal" => Some(CleanupRuleKind::Literal),
            "regex" => Some(CleanupRuleKind::Regex),
            _ => None,
        }
    }
}

impl fmt::Display for CleanupRuleKind {
    /// KHÔNG DẤU (NFR16).
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Một luật làm sạch ĐÃ PHÂN GIẢI (tầng đã gán, từ [`store::resolve_two_tiers`]) — đầu vào
/// của [`apply`] và của [`crate::core::segment::pipeline::PipelineInput::cleanup_rules`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CleanupRule {
    pub tier: CleanupRuleTier,
    /// Khoá hàng TRONG kho của chính `tier` — không duy nhất xuyên hai tầng.
    pub id: i64,
    pub pattern: String,
    pub kind: CleanupRuleKind,
    pub enabled: bool,
}

/// Khoá của một luật trên dây và trong [`CleanupMatch`] — cặp, không phải `id` trần (xem
/// doc-comment đầu module).
pub type CleanupRuleKey = (CleanupRuleTier, i64);

/// Một chỗ khớp — điểm mã, nửa-mở `[start, end)`, cùng quy ước
/// `core::glossary::entry::GlossaryMark`/`GlossaryMarksMap` phía frontend.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CleanupMatch {
    pub rule_tier: CleanupRuleTier,
    pub rule_id: i64,
    pub start: usize,
    pub end: usize,
}

/// Báo cáo làm sạch của MỘT Chương — gắn vào
/// `core::segment::import::ImportedChapter::cleanup_report` (Story 6.5, đóng nợ
/// `deferred-work.md:9359`: xem trước và xác nhận nay đọc CÙNG một lượt chạy chuỗi).
///
/// `matches`/`per_rule_counts` tính trên văn bản NGAY TRƯỚC khi luật xoá gì — đã giải mã
/// ([`Step::DecodeEncoding`] đã chạy), CHƯA chuẩn hoá (đứng TRƯỚC [`Step::NormalizeParagraphsAndWhitespace`]
/// trong `PIPELINE_ORDER`).
///
/// 🔴 **KHÔNG `source_before_removal`** — vòng rà 2026-09-06 gỡ nó: một bản CHÉP TOÀN VĂN BẢN
/// của mỗi Chương, ghi ra ở MỌI lượt chạy chuỗi, mà không một chỗ nào đọc lại (`grep -rn
/// "source_before_removal" src-tauri/src/` chỉ còn khớp chính định nghĩa nó, trước lượt gỡ
/// này). `commands::project::cleanup_preview_for`/`build_cleanup_preview_wire` lấy văn bản
/// "trước khi xoá" từ chính tham số ĐANG CÓ SẴN ở tay người gọi (`full_text`/`display_window`),
/// không cần đọc lại từ báo cáo — nên trường này chưa từng có một chỗ gọi thật. Nếu một chỗ
/// gọi thật xuất hiện sau này (một tầng khác cần đọc lại văn bản trước-khi-xoá TỪ CHÍNH báo
/// cáo, không có sẵn nó ở tay), thêm lại trường này KÈM chỗ gọi đó trong CÙNG một lượt — không
/// thêm trước để "phòng khi cần".
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CleanupReport {
    pub matches: Vec<CleanupMatch>,
    pub per_rule_counts: BTreeMap<CleanupRuleKey, usize>,
}

/// Kết quả một lượt [`apply`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Cleaned {
    /// Văn bản sau khi xoá mọi chỗ khớp của luật ĐANG BẬT (dải hợp nhất, xoá đúng một lần
    /// trên phần chồng nhau).
    pub text: String,
    /// MỌI chỗ khớp của MỌI luật — kể cả luật đã tắt (§Always: "vẫn báo cáo chỗ khớp").
    /// Không sắp theo tầng/`id` — sắp theo `(start, rule_tier, rule_id)` để hiển thị theo
    /// thứ tự xuất hiện trong văn bản.
    pub matches: Vec<CleanupMatch>,
    /// Số chỗ khớp của MỖI luật, trên TOÀN văn bản — kể cả luật đã tắt.
    pub per_rule_counts: BTreeMap<CleanupRuleKey, usize>,
}

/// Luật không thi hành được lúc chạy — mẫu regex không biên dịch được.
///
/// ⚠️ **Không nên xảy ra trên đường sản phẩm.** `core::cleanup::store::add_rule`/`edit_rule`
/// biên dịch thử một mẫu `regex` TRƯỚC khi ghi (§Always spec 6.5: "Lưu bị từ chối, bảng
/// không đổi một hàng") — biến thể này chỉ có thể chạm tới nếu dữ liệu trên đĩa đã trôi
/// khỏi lời hứa đó (một bản ứng dụng cũ/hỏng ghi ra). Trả `Err`, không `panic!` — `panic =
/// "abort"` giết cả tiến trình (`src-tauri/AGENTS.md:10`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CleanupError {
    pub tier: CleanupRuleTier,
    pub id: i64,
}

impl fmt::Display for CleanupError {
    /// KHÔNG DẤU (NFR16).
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "cleanup[{}#{}] pattern does not compile as regex", self.tier, self.id)
    }
}

impl std::error::Error for CleanupError {}

/// Điểm mã cho MỖI vị trí byte của `text` — chỉ những chỉ số TRÙNG một ranh giới ký tự thật
/// mang giá trị có nghĩa; mọi chỗ gọi trong [`apply`] chỉ tra cứu ở các vị trí đó (đầu/cuối
/// một chỗ khớp `str`/`regex`, luôn là ranh giới ký tự hợp lệ).
fn byte_to_char_index(text: &str) -> Vec<usize> {
    let mut index = vec![0usize; text.len() + 1];
    let mut count = 0usize;
    for (byte_pos, _) in text.char_indices() {
        index[byte_pos] = count;
        count += 1;
    }
    index[text.len()] = count;
    index
}

/// Biên dịch một mẫu `regex` cho luật làm sạch — **LUÔN đa dòng** (`(?m)`, cờ `multi_line`
/// của crate `regex`): `^`/`$` neo theo TỪNG DÒNG, không theo toàn văn bản. Đây KHÔNG phải
/// một lựa chọn tuỳ ý — mẫu mẫu của chính mockup (`web-import.html:355`,
/// `^本章由.*整理$`, "dòng quảng cáo cuối chương") chỉ có nghĩa khi neo theo dòng: một
/// Chương nhiều dòng mà không bật cờ này sẽ khớp **0 lần** trong im lặng (đúng lớp lỗi
/// "rỗng im lặng" — `AGENTS.md`), vì `^`/`$` mặc định neo vào ĐẦU/CUỐI TOÀN chuỗi.
///
/// 🔴 Chỗ gọi DUY NHẤT khác ngoài [`byte_ranges_for`]: `core::cleanup::store::validate_pattern`
/// biên dịch thử BẰNG ĐÚNG hàm này trước khi ghi — hai lớp (lưu và chạy) phải nói cùng một
/// ngôn ngữ regex, nếu không một mẫu biên dịch được lúc lưu (cờ khác) có thể vỡ lúc chạy.
pub fn compile_cleanup_regex(pattern: &str) -> Result<regex::Regex, regex::Error> {
    regex::RegexBuilder::new(pattern).multi_line(true).build()
}

/// Mọi dải byte `[start, end)` một luật khớp trên `text` — literal dùng `str::match_indices`
/// (không chồng lấn TRONG một luật, SQLite/Rust tách nghĩa theo lượt gọi này); regex dùng
/// `Regex::find_iter` (cùng tính chất không chồng lấn nội bộ một luật, và an toàn với một
/// mẫu có thể khớp chuỗi rỗng — bộ lặp của crate `regex` tự tiến ít nhất một ký tự sau một
/// khớp rỗng, không vòng lặp vô hạn).
fn byte_ranges_for(text: &str, rule: &CleanupRule) -> Result<Vec<(usize, usize)>, CleanupError> {
    if rule.pattern.is_empty() {
        // Phòng thủ — DDL + tầng lệnh đã cấm mẫu rỗng/chỉ khoảng trắng từ trước khi ghi
        // (§Always spec 6.5), nên nhánh này không nên chạm được trên đường sản phẩm.
        return Ok(Vec::new());
    }
    match rule.kind {
        CleanupRuleKind::Literal => {
            Ok(text.match_indices(rule.pattern.as_str()).map(|(i, m)| (i, i + m.len())).collect())
        }
        CleanupRuleKind::Regex => {
            let re = compile_cleanup_regex(&rule.pattern)
                .map_err(|_| CleanupError { tier: rule.tier, id: rule.id })?;
            // 🔴 SỬA vòng rà (2026-09-06) — một mẫu như `x*` khớp được chuỗi RỖNG ở mọi vị
            // trí không có `x`; xoá 0 ký tự không xoá gì, nhưng vẫn được ĐẾM và vẫn sinh một
            // `CleanupMatch` — số đếm phồng lên (một Chương dài ra vài nghìn ký tự ra "khớp
            // vài nghìn lần") và một dấu gạch ngang KHÔNG CÓ ĐỘ RỘNG xuất hiện trên màn hình
            // (vô hình với mắt, nhưng vẫn là một `<span>` thật). Lọc chúng ở NGUỒN — literal
            // không cần lọc (pattern rỗng đã bị chặn ở đầu hàm, và một pattern KHÔNG rỗng
            // không thể khớp chuỗi rỗng qua `match_indices`).
            Ok(re
                .find_iter(text)
                .filter(|m| !m.range().is_empty())
                .map(|m| (m.start(), m.end()))
                .collect())
        }
    }
}

/// Áp `rules` lên `text` — hàm THUẦN, thân THẬT của [`crate::core::segment::pipeline::Step::CleanByRules`]
/// (bước 3 chuỗi AD-39). Biên dịch mỗi mẫu `regex` ĐÚNG MỘT LẦN cho lượt gọi này (mỗi luật
/// gọi [`byte_ranges_for`] đúng một lần).
///
/// # Lỗi
/// [`CleanupError`] nếu một luật `regex` không biên dịch được — xem doc-comment của kiểu đó
/// cho lý do đây không nên xảy ra trên đường sản phẩm.
pub fn apply(text: &str, rules: &[CleanupRule]) -> Result<Cleaned, CleanupError> {
    let byte_to_char = byte_to_char_index(text);

    let mut matches: Vec<CleanupMatch> = Vec::new();
    let mut per_rule_counts: BTreeMap<CleanupRuleKey, usize> = BTreeMap::new();
    let mut delete_ranges: Vec<(usize, usize)> = Vec::new();

    for rule in rules {
        let byte_ranges = byte_ranges_for(text, rule)?;
        let key: CleanupRuleKey = (rule.tier, rule.id);
        per_rule_counts.insert(key, byte_ranges.len());

        for &(start, end) in &byte_ranges {
            matches.push(CleanupMatch {
                rule_tier: rule.tier,
                rule_id: rule.id,
                start: byte_to_char[start],
                end: byte_to_char[end],
            });
        }

        // 🔴 Chỉ luật ĐANG BẬT góp vào việc xoá — luật tắt vẫn đã được ĐẾM ở trên (đúng vế
        // "tắt đổi việc xoá, không đổi việc đo" của §Always spec 6.5).
        if rule.enabled {
            delete_ranges.extend(byte_ranges);
        }
    }

    // Thứ tự hiển thị: theo vị trí xuất hiện trong văn bản, tầng rồi id phân xử khi hai
    // luật khớp cùng một điểm bắt đầu (ổn định, không phụ thuộc thứ tự nạp `rules`).
    matches.sort_by(|a, b| {
        a.start
            .cmp(&b.start)
            .then(a.rule_tier.cmp(&b.rule_tier))
            .then(a.rule_id.cmp(&b.rule_id))
    });

    // Hợp các dải xoá chồng nhau — AD-18 hợp nhất: hai luật khớp cùng một chỗ thì chỗ đó
    // biến mất MỘT LẦN, dù cả hai luật đều đã đếm nó riêng ở trên.
    delete_ranges.sort_by_key(|r| r.0);
    let mut merged: Vec<(usize, usize)> = Vec::new();
    for (start, end) in delete_ranges {
        match merged.last_mut() {
            Some(last) if start <= last.1 => {
                if end > last.1 {
                    last.1 = end;
                }
            }
            _ => merged.push((start, end)),
        }
    }

    let mut out = String::with_capacity(text.len());
    let mut cursor = 0usize;
    for &(start, end) in &merged {
        out.push_str(&text[cursor..start]);
        cursor = end;
    }
    out.push_str(&text[cursor..]);

    Ok(Cleaned { text: out, matches, per_rule_counts })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn literal(tier: CleanupRuleTier, id: i64, pattern: &str, enabled: bool) -> CleanupRule {
        CleanupRule { tier, id, pattern: pattern.to_owned(), kind: CleanupRuleKind::Literal, enabled }
    }

    fn regex_rule(tier: CleanupRuleTier, id: i64, pattern: &str, enabled: bool) -> CleanupRule {
        CleanupRule { tier, id, pattern: pattern.to_owned(), kind: CleanupRuleKind::Regex, enabled }
    }

    #[test]
    fn no_rules_leaves_text_byte_for_byte_unchanged() {
        let out = apply("mot doan van ban", &[]).expect("0 luat khong the loi");
        assert_eq!(out.text, "mot doan van ban");
        assert!(out.matches.is_empty());
        assert!(out.per_rule_counts.is_empty());
    }

    #[test]
    fn a_literal_rule_removes_every_occurrence_and_counts_them() {
        let rules = [literal(CleanupRuleTier::Global, 1, "qiu shou cang", true)];
        let text = "A qiu shou cang B qiu shou cang C";
        let out = apply(text, &rules).expect("luat literal hop le");
        // Chỉ đúng CHUỖI KHỚP bị xoá — khoảng trắng hai bên nó (không thuộc mẫu) ở lại,
        // nên chỗ vừa xoá để lại HAI dấu cách liền nhau, không phải một.
        assert_eq!(out.text, "A  B  C");
        assert_eq!(out.per_rule_counts[&(CleanupRuleTier::Global, 1)], 2);
        assert_eq!(out.matches.len(), 2);
    }

    #[test]
    fn a_disabled_rule_is_counted_but_not_removed() {
        let rules = [literal(CleanupRuleTier::Global, 1, "rac", false)];
        let out = apply("rac o day rac o kia", &rules).expect("luat tat van hop le");
        assert_eq!(out.text, "rac o day rac o kia", "luat tat khong duoc xoa gi");
        assert_eq!(out.per_rule_counts[&(CleanupRuleTier::Global, 1)], 2, "van phai dem");
        assert_eq!(out.matches.len(), 2, "van phai bao cao cho khop");
    }

    #[test]
    fn two_tiers_matching_the_same_spot_delete_it_once_but_both_rules_count_it() {
        let rules = [
            literal(CleanupRuleTier::Global, 1, "X", true),
            literal(CleanupRuleTier::Work, 1, "X", true),
        ];
        let out = apply("aXb", &rules).expect("hai luat trung nhau van hop le");
        assert_eq!(out.text, "ab", "cho khop chi bien mat MOT LAN");
        assert_eq!(out.per_rule_counts[&(CleanupRuleTier::Global, 1)], 1, "luat Global van dem");
        assert_eq!(out.per_rule_counts[&(CleanupRuleTier::Work, 1)], 1, "luat Work van dem");
        assert_eq!(out.matches.len(), 2, "ca hai luat deu bao cao cho khop, khong khu trung lap");
    }

    #[test]
    fn a_regex_rule_matches_by_pattern() {
        let rules = [regex_rule(CleanupRuleTier::Work, 7, "^ghi chu.*$", true)];
        let out = apply("dong dau\nghi chu cua tac gia\ndong cuoi", &rules).expect("regex hop le");
        assert_eq!(out.text, "dong dau\n\ndong cuoi");
    }

    #[test]
    fn an_invalid_regex_is_rejected_not_panicking() {
        let rules = [regex_rule(CleanupRuleTier::Global, 1, "[unclosed", true)];
        let err = apply("bat ky van ban nao", &rules).expect_err("mau regex hong phai la Err");
        assert_eq!(err.tier, CleanupRuleTier::Global);
        assert_eq!(err.id, 1);
    }

    #[test]
    fn matches_are_codepoints_not_bytes() {
        // "萧" la mot ky tu, ba byte UTF-8. Luat khop chinh no phai cho span [1, 2), khong
        // [1, 4) hay mot con so tinh theo byte.
        let rules = [literal(CleanupRuleTier::Global, 1, "萧", true)];
        let out = apply("A萧B", &rules).expect("literal Han hop le");
        assert_eq!(out.matches.len(), 1);
        let m = out.matches[0];
        assert_eq!((m.start, m.end), (1, 2), "span phai la DIEM MA, khong phai byte");
    }

    #[test]
    fn a_rule_that_matches_the_entire_text_yields_an_empty_string() {
        let rules = [literal(CleanupRuleTier::Global, 1, "toan bo", true)];
        let out = apply("toan bo", &rules).expect("literal hop le");
        assert_eq!(out.text, "", "khop toan van ban phai cho chuoi rong");
    }

    #[test]
    fn a_zero_length_regex_match_is_not_counted_and_yields_no_span() {
        // "x*" khop chuoi RONG o MOI vi tri khong co "x" -- neu khong loc, mot van ban 4 ky
        // tu khong co "x" nao se cho 5 cho khop RONG (truoc/giua/sau moi ky tu).
        let rules = [regex_rule(CleanupRuleTier::Global, 1, "x*", true)];
        let out = apply("abcd", &rules).expect("regex hop le");
        assert_eq!(out.text, "abcd", "khop do dai 0 khong xoa gi");
        assert_eq!(out.matches.len(), 0, "khop rong khong duoc sinh span");
        assert_eq!(out.per_rule_counts[&(CleanupRuleTier::Global, 1)], 0, "khop rong khong duoc dem");
    }

    #[test]
    fn a_regex_alternation_still_counts_the_real_non_empty_matches_alongside_a_zero_length_pattern() {
        // Doi chung: mot mau vua khop RONG (o vi tri khong co "x") vua khop THAT (khi co
        // "x") tren CUNG mot luat -- chi cho khop THAT moi con lai.
        let rules = [regex_rule(CleanupRuleTier::Global, 1, "x*", true)];
        let out = apply("axxbxc", &rules).expect("regex hop le");
        assert_eq!(out.text, "abc", "chi cho khop THAT (co do dai) bi xoa");
        // "x*" tren "axxbxc": cho khop THAT chi co "xx" (vi tri 1..3) va "x" (vi tri 4..5) --
        // moi cho khop RONG khac (truoc 'a', truoc 'b', truoc 'c', cuoi chuoi) bi loc.
        assert_eq!(out.per_rule_counts[&(CleanupRuleTier::Global, 1)], 2, "hai cho khop THAT: xx, x");
    }
}
