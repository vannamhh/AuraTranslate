use unicode_normalization::UnicodeNormalization;
use unicode_normalization::char::canonical_combining_class;

/// One normalised char and the UTF-16 range of the stored cluster it came from.
#[derive(Debug, Clone, Copy)]
pub(crate) struct CharOrigin {
    pub byte: u32,
    pub utf16_start: u32,
    pub utf16_end: u32,
}

pub(crate) struct Normalized {
    pub text: String,
    pub chars: Vec<CharOrigin>,
}

/// NFC and Unicode lowercase applied per cluster (a starter plus its combining marks), so each
/// normalised char still maps to a UTF-16 range of the unnormalised input.
pub(crate) fn normalize(input: &str) -> Normalized {
    let mut text = String::with_capacity(input.len());
    let mut chars = Vec::with_capacity(input.len());
    let mut utf16_pos = 0u32;
    let mut iter = input.char_indices().peekable();
    while let Some((start, first)) = iter.next() {
        let mut end = start + first.len_utf8();
        let mut utf16_end = utf16_pos + width16(first);
        while let Some(&(at, next)) = iter.peek() {
            if canonical_combining_class(next) == 0 {
                break;
            }
            end = at + next.len_utf8();
            utf16_end += width16(next);
            iter.next();
        }
        if end - start == 1 {
            chars.push(CharOrigin {
                byte: byte_len(&text),
                utf16_start: utf16_pos,
                utf16_end,
            });
            text.push(first.to_ascii_lowercase());
        } else {
            for lowered in input[start..end].nfc().flat_map(char::to_lowercase) {
                chars.push(CharOrigin {
                    byte: byte_len(&text),
                    utf16_start: utf16_pos,
                    utf16_end,
                });
                text.push(lowered);
            }
        }
        utf16_pos = utf16_end;
    }
    Normalized { text, chars }
}

fn width16(c: char) -> u32 {
    u32::try_from(c.len_utf16()).unwrap_or(2)
}

fn byte_len(text: &str) -> u32 {
    u32::try_from(text.len()).unwrap_or(u32::MAX)
}

/// The rule key of an avoid phrase (AD-67 rule 2).
pub fn rule_key(avoid_phrase: &str) -> String {
    normalize(avoid_phrase).text
}
