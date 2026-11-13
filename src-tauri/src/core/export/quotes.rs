use std::borrow::Cow;

const OPENS_AFTER: [char; 9] = ['(', '[', '{', '«', '“', '‘', '—', '–', ':'];

/// Turns each straight `"` into `“` or `”` by the character before it; quotes are never paired,
/// so an odd count only misdirects the quote it concerns.
pub(super) fn curl_double_quotes(text: &str) -> Cow<'_, str> {
    if !text.contains('"') {
        return Cow::Borrowed(text);
    }
    let mut out = String::with_capacity(text.len() + 4);
    let mut previous: Option<char> = None;
    for c in text.chars() {
        if c == '"' {
            let opens = previous.is_none_or(|p| p.is_whitespace() || OPENS_AFTER.contains(&p));
            out.push(if opens { '“' } else { '”' });
        } else {
            out.push(c);
        }
        previous = Some(c);
    }
    Cow::Owned(out)
}

#[cfg(test)]
mod tests {
    use super::curl_double_quotes;

    #[test]
    fn matrix() {
        for (input, expected) in [
            ("says: \"War.\"", "says: “War.”"),
            ("\"War\" often", "“War” often"),
            ("lists \"A\", \"B\" and \"C\"", "lists “A”, “B” and “C”"),
            ("(\"A\")", "(“A”)"),
            ("said \"hello", "said “hello"),
            ("no quotes.", "no quotes."),
            ("“A” and 'B'", "“A” and 'B'"),
        ] {
            assert_eq!(curl_double_quotes(input), expected);
        }
    }
}
