const LANGUAGE_HEURISTIC_SAMPLE_CHARS: usize = 20_000;

fn is_common_cjk_char(c: char) -> bool {
    matches!(c as u32, 0x4E00..=0x9FFF | 0x3400..=0x4DBF | 0xF900..=0xFAFF)
}

/// `None` when the sample has no letters to count.
pub fn cjk_ratio_sample(text: &str) -> Option<f64> {
    let (mut cjk, mut latin) = (0usize, 0usize);
    for c in text.chars().take(LANGUAGE_HEURISTIC_SAMPLE_CHARS) {
        if is_common_cjk_char(c) {
            cjk += 1;
        } else if c.is_alphabetic() {
            latin += 1;
        }
    }
    let total = cjk + latin;
    if total == 0 {
        None
    } else {
        Some(cjk as f64 / total as f64)
    }
}

pub const CJK_MAJORITY: f64 = 0.5;

/// `zh` at a CJK ratio of at least 0.5, else `en`; `None` when there are no letters.
pub fn guess_source_lang(text: &str) -> Option<&'static str> {
    let ratio = cjk_ratio_sample(text)?;
    Some(if ratio >= CJK_MAJORITY { "zh" } else { "en" })
}
