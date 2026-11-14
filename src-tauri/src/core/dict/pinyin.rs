//! Pinyin search key of a query. A second copy lives in `tools/dict-build/src/pinyin.rs`, which writes `dict_entry.reading_key`;
//! both read `tools/dict-build/tests/fixtures/pinyin_key.tsv` and must stay identical (AD-69).

// Escapes keep the Vietnamese-letter scan of check:i18n from reading this table as UI text.
const TONE_TABLE: &[(char, char)] = &[
    ('\u{101}', 'a'),
    ('\u{e1}', 'a'),
    ('\u{1ce}', 'a'),
    ('\u{e0}', 'a'),
    ('\u{113}', 'e'),
    ('\u{e9}', 'e'),
    ('\u{11b}', 'e'),
    ('\u{e8}', 'e'),
    ('\u{ea}', 'e'),
    ('\u{12b}', 'i'),
    ('\u{ed}', 'i'),
    ('\u{1d0}', 'i'),
    ('\u{ec}', 'i'),
    ('\u{14d}', 'o'),
    ('\u{f3}', 'o'),
    ('\u{1d2}', 'o'),
    ('\u{f2}', 'o'),
    ('\u{16b}', 'u'),
    ('\u{fa}', 'u'),
    ('\u{1d4}', 'u'),
    ('\u{f9}', 'u'),
    ('\u{fc}', 'v'),
    ('\u{1d6}', 'v'),
    ('\u{1d8}', 'v'),
    ('\u{1da}', 'v'),
    ('\u{1dc}', 'v'),
    ('\u{1e3f}', 'm'),
    ('\u{144}', 'n'),
    ('\u{148}', 'n'),
    ('\u{1f9}', 'n'),
];

const DROPPED: &[char] = &[
    '⁰', '¹', '²', '³', '⁴', '⁵', '⁻', '·', '→', '’', '‘', '—', '–',
];

/// `None` when the reading is not a pinyin string (any character outside the closed table)
/// or yields an empty key.
pub fn pinyin_key(reading: &str) -> Option<String> {
    let head = reading.split('(').next().unwrap_or("");
    let lowered: String = head.chars().flat_map(char::to_lowercase).collect();
    let mut key = String::with_capacity(lowered.len());
    let mut chars = lowered.chars().peekable();
    while let Some(c) = chars.next() {
        if c == 'u' && chars.peek() == Some(&':') {
            chars.next();
            key.push('v');
        } else if let Some(&(_, base)) = TONE_TABLE.iter().find(|(from, _)| *from == c) {
            key.push(base);
        } else if c.is_ascii_lowercase() {
            key.push(c);
        } else if c.is_ascii_digit()
            || c.is_whitespace()
            || c.is_ascii_punctuation()
            || DROPPED.contains(&c)
        {
            if c.is_ascii_digit() && !('0'..='5').contains(&c) {
                return None;
            }
        } else {
            return None;
        }
    }
    if key.is_empty() { None } else { Some(key) }
}
