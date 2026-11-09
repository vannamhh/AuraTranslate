/// Replaces several byte ranges of `text` at once. Ranges must be ascending, non-empty, on char
/// boundaries and non-overlapping; anything else returns `None`. Applied right to left on the
/// original text, so a replacement is never matched again.
pub fn apply_spans(text: &str, spans: &[(usize, usize, &str)]) -> Option<String> {
    let mut previous_end = 0usize;
    for &(start, end, _) in spans {
        if start >= end
            || start < previous_end
            || end > text.len()
            || !text.is_char_boundary(start)
            || !text.is_char_boundary(end)
        {
            return None;
        }
        previous_end = end;
    }
    let mut out = text.to_owned();
    for &(start, end, replacement) in spans.iter().rev() {
        out.replace_range(start..end, replacement);
    }
    Some(out)
}

#[cfg(test)]
mod tests {
    use super::apply_spans;

    #[test]
    fn replaces_every_span_without_rematching_replaced_text() {
        let text = "aa bb aa";
        let spans = [(0, 2, "bb aa"), (6, 8, "bb aa")];
        assert_eq!(apply_spans(text, &spans).as_deref(), Some("bb aa bb bb aa"));
    }

    #[test]
    fn no_spans_returns_the_text_unchanged() {
        assert_eq!(apply_spans("abc", &[]).as_deref(), Some("abc"));
    }

    #[test]
    fn refuses_overlap_unsorted_empty_and_out_of_bounds() {
        assert_eq!(apply_spans("abcdef", &[(0, 3, "x"), (2, 4, "y")]), None);
        assert_eq!(apply_spans("abcdef", &[(3, 4, "x"), (0, 1, "y")]), None);
        assert_eq!(apply_spans("abcdef", &[(2, 2, "x")]), None);
        assert_eq!(apply_spans("abcdef", &[(4, 9, "x")]), None);
    }

    #[test]
    fn refuses_a_range_inside_a_multibyte_char() {
        assert_eq!(apply_spans("\u{111}a", &[(1, 2, "x")]), None);
    }
}
