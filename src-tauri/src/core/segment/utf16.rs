//! UTF-16 offsets, the unit a JS `Range` uses, mapped back onto Rust strings (AD-1).

/// Byte offset of UTF-16 offset `target` in `text`; `None` when it falls inside a surrogate pair
/// or past the end.
pub fn utf16_to_byte(text: &str, target: u32) -> Option<usize> {
    let mut units = 0u32;
    for (byte, ch) in text.char_indices() {
        if units == target {
            return Some(byte);
        }
        if units > target {
            return None;
        }
        units += u32::try_from(ch.len_utf16()).unwrap_or(2);
    }
    (units == target).then_some(text.len())
}
