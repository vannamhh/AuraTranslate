//! Pinyin search key written to `dict_entry.reading_key`. A second copy lives in
//! `src-tauri/src/core/dict/pinyin.rs`; both read `tests/fixtures/pinyin_key.tsv`.

const TONE_TABLE: &[(char, char)] = &[
    ('ā', 'a'),
    ('á', 'a'),
    ('ǎ', 'a'),
    ('à', 'a'),
    ('ē', 'e'),
    ('é', 'e'),
    ('ě', 'e'),
    ('è', 'e'),
    ('ê', 'e'),
    ('ī', 'i'),
    ('í', 'i'),
    ('ǐ', 'i'),
    ('ì', 'i'),
    ('ō', 'o'),
    ('ó', 'o'),
    ('ǒ', 'o'),
    ('ò', 'o'),
    ('ū', 'u'),
    ('ú', 'u'),
    ('ǔ', 'u'),
    ('ù', 'u'),
    ('ü', 'v'),
    ('ǖ', 'v'),
    ('ǘ', 'v'),
    ('ǚ', 'v'),
    ('ǜ', 'v'),
    ('ḿ', 'm'),
    ('ń', 'n'),
    ('ň', 'n'),
    ('ǹ', 'n'),
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
