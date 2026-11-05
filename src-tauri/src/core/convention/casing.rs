/// The replacement spelled in the case of the text it replaces: all lower, Capitalised and
/// ALL UPPER follow the matched text; any other mix keeps the rule's own spelling.
pub fn transfer_case(matched: &str, replacement: &str) -> String {
    let mut letters = matched
        .chars()
        .filter(|c| c.is_lowercase() || c.is_uppercase());
    let Some(first) = letters.next() else {
        return replacement.to_owned();
    };
    let rest: Vec<char> = letters.collect();
    let rest_lower = rest.iter().all(|c| c.is_lowercase());
    let rest_upper = rest.iter().all(|c| c.is_uppercase());
    if first.is_lowercase() && rest_lower {
        replacement.to_lowercase()
    } else if first.is_uppercase() && rest.is_empty() || first.is_uppercase() && rest_lower {
        capitalise(replacement)
    } else if first.is_uppercase() && rest_upper {
        replacement.to_uppercase()
    } else {
        replacement.to_owned()
    }
}

fn capitalise(text: &str) -> String {
    let lower = text.to_lowercase();
    let mut chars = lower.chars();
    match chars.next() {
        Some(head) => head.to_uppercase().chain(chars).collect(),
        None => lower,
    }
}
