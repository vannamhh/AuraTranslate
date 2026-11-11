//! CSV/TSV tokenising, quoting, formula guard and file I/O shared by the Glossary, bilingual
//! import and convention exchanges. A RFC 4180 subset written by hand (NFR15: no `csv` crate).
//! Column layout and error vocabulary stay with each caller.

use std::io::Read as _;
use std::path::Path;

use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DelimitedError {
    UnterminatedQuotedField { line: usize },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReadError {
    ReadFailed { path: String, detail: String },
    TooLarge { size: u64, limit: u64 },
    NotUtf8 { path: String },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WriteError {
    pub path: String,
    pub detail: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Delimiter {
    Csv,
    Tsv,
}

impl Delimiter {
    pub const fn as_char(self) -> char {
        match self {
            Delimiter::Csv => ',',
            Delimiter::Tsv => '\t',
        }
    }
}

pub(crate) fn strip_bom(raw: &str) -> &str {
    raw.strip_prefix('\u{feff}').unwrap_or(raw)
}

const FORMULA_TRIGGER_CHARS: [char; 4] = ['=', '+', '-', '@'];

const FORMULA_GUARD_PREFIX: char = '\'';

// Export and import share this predicate so a value that already starts with `'` round-trips.
fn char_after_leading_quotes(field: &str) -> Option<char> {
    field
        .trim_start_matches(FORMULA_GUARD_PREFIX)
        .chars()
        .next()
}

pub(crate) fn needs_formula_guard(field: &str) -> bool {
    matches!(char_after_leading_quotes(field), Some(c) if FORMULA_TRIGGER_CHARS.contains(&c))
}

pub(crate) fn strip_formula_guard(field: &str) -> &str {
    if needs_formula_guard(field)
        && let Some(rest) = field.strip_prefix(FORMULA_GUARD_PREFIX)
    {
        return rest;
    }
    field
}

pub(crate) fn field_needs_quoting(field: &str, delimiter: char) -> bool {
    field.contains(delimiter) || field.contains('"') || field.contains('\n') || field.contains('\r')
}

pub(crate) fn quote_field(field: &str) -> String {
    let mut out = String::with_capacity(field.len() + 2);
    out.push('"');
    for c in field.chars() {
        if c == '"' {
            out.push('"');
        }
        out.push(c);
    }
    out.push('"');
    out
}

pub(crate) fn render_field(field: &str, delimiter: char) -> String {
    let guarded: std::borrow::Cow<'_, str> = if needs_formula_guard(field) {
        std::borrow::Cow::Owned(format!("{FORMULA_GUARD_PREFIX}{field}"))
    } else {
        std::borrow::Cow::Borrowed(field)
    };

    if field_needs_quoting(&guarded, delimiter) {
        quote_field(&guarded)
    } else {
        guarded.into_owned()
    }
}

const ZERO_WIDTH_CHARS: [char; 5] = ['\u{200B}', '\u{200C}', '\u{200D}', '\u{2060}', '\u{FEFF}'];

pub(crate) fn strip_zero_width(s: &str) -> String {
    s.chars()
        .filter(|c| !ZERO_WIDTH_CHARS.contains(c))
        .collect()
}

pub(crate) fn split_first_logical_line(text: &str, delimiter: Option<char>) -> (&str, &str, bool) {
    let is_field_boundary = |c: char| match delimiter {
        Some(d) => c == d,
        None => c == ',' || c == '\t',
    };

    let mut in_quotes = false;
    let mut at_field_start = true;
    let mut chars = text.char_indices().peekable();

    while let Some((i, c)) = chars.next() {
        if in_quotes {
            if c == '"' {
                if chars.peek().map(|&(_, next)| next) == Some('"') {
                    chars.next();
                } else {
                    in_quotes = false;
                }
            }
            continue;
        }

        match c {
            '"' if at_field_start => {
                in_quotes = true;
                at_field_start = false;
            }
            '\n' => return (&text[..i], &text[i + 1..], false),
            '\r' => {
                if let Some(&(j, '\n')) = chars.peek() {
                    return (&text[..i], &text[j + 1..], false);
                }
                return (&text[..i], &text[i + 1..], false);
            }
            c if is_field_boundary(c) => at_field_start = true,
            _ => at_field_start = false,
        }
    }

    (text, "", in_quotes)
}

pub(crate) fn count_line_breaks(s: &str) -> usize {
    let mut count = 0;
    let mut chars = s.chars().peekable();
    while let Some(c) = chars.next() {
        match c {
            '\n' => count += 1,
            '\r' => {
                if chars.peek() == Some(&'\n') {
                    chars.next();
                }
                count += 1;
            }
            _ => {}
        }
    }
    count
}

pub(crate) fn logical_lines(
    text: &str,
    delimiter: Option<char>,
    start_line: usize,
) -> Result<Vec<(usize, &str)>, DelimitedError> {
    let mut out = Vec::new();
    let mut remaining = text;
    let mut line_no = start_line;

    while !remaining.is_empty() {
        let (line, rest, unterminated) = split_first_logical_line(remaining, delimiter);
        if unterminated {
            return Err(DelimitedError::UnterminatedQuotedField { line: line_no });
        }
        out.push((line_no, line));
        line_no += count_line_breaks(line) + 1;
        remaining = rest;
    }

    Ok(out)
}

pub(crate) fn split_fields(line: &str, delimiter: char) -> Vec<String> {
    let mut fields = Vec::new();
    let mut field = String::new();
    let mut in_quotes = false;
    let mut at_field_start = true;
    let mut chars = line.chars().peekable();

    while let Some(c) = chars.next() {
        if in_quotes {
            if c == '"' {
                if chars.peek() == Some(&'"') {
                    field.push('"');
                    chars.next();
                } else {
                    in_quotes = false;
                }
            } else {
                field.push(c);
            }
            continue;
        }

        if c == '"' && at_field_start {
            in_quotes = true;
            at_field_start = false;
            continue;
        }

        if c == delimiter {
            fields.push(std::mem::take(&mut field));
            at_field_start = true;
            continue;
        }

        field.push(c);
        at_field_start = false;
    }

    fields.push(field);
    fields
}

pub(crate) fn unquoted_char_present(text: &str, target: char) -> bool {
    let mut in_quotes = false;
    let mut at_field_start = true;
    let mut chars = text.chars().peekable();

    while let Some(c) = chars.next() {
        if in_quotes {
            if c == '"' {
                if chars.peek() == Some(&'"') {
                    chars.next();
                } else {
                    in_quotes = false;
                }
            }
            continue;
        }

        match c {
            '"' if at_field_start => {
                in_quotes = true;
                at_field_start = false;
            }
            c if c == target => return true,
            ',' | '\t' => at_field_start = true,
            _ => at_field_start = false,
        }
    }

    false
}

pub fn read_text_file(path: &Path, limit: u64) -> Result<String, ReadError> {
    let path_str = path.display().to_string();

    let file = std::fs::File::open(path).map_err(|e| ReadError::ReadFailed {
        path: path_str.clone(),
        detail: e.to_string(),
    })?;

    let mut bytes = Vec::new();
    file.take(limit + 1)
        .read_to_end(&mut bytes)
        .map_err(|e| ReadError::ReadFailed {
            path: path_str.clone(),
            detail: e.to_string(),
        })?;

    if bytes.len() as u64 > limit {
        return Err(ReadError::TooLarge {
            size: bytes.len() as u64,
            limit,
        });
    }

    let text = String::from_utf8(bytes).map_err(|_| ReadError::NotUtf8 { path: path_str })?;

    Ok(strip_bom(&text).to_owned())
}

pub fn write_text_file_atomic(path: &Path, contents: &str) -> Result<(), WriteError> {
    let path_str = path.display().to_string();

    let Some(file_name) = path.file_name() else {
        return Err(WriteError {
            path: path_str,
            detail: "duong dan khong co thanh phan ten tep -- khong doan duoc ten tep tam"
                .to_owned(),
        });
    };

    let mut tmp_name = file_name.to_os_string();
    tmp_name.push(format!(".{}-{}.tmp", std::process::id(), Uuid::new_v4()));
    let tmp = path.with_file_name(tmp_name);

    let write_result = (|| -> std::io::Result<()> {
        use std::io::Write as _;
        let mut file = std::fs::File::create(&tmp)?;
        file.write_all(contents.as_bytes())?;
        file.sync_all()?;
        Ok(())
    })();

    if let Err(e) = write_result {
        let _ = std::fs::remove_file(&tmp);
        return Err(WriteError {
            path: path_str,
            detail: e.to_string(),
        });
    }

    if let Err(e) = std::fs::rename(&tmp, path) {
        let _ = std::fs::remove_file(&tmp);
        return Err(WriteError {
            path: path_str,
            detail: e.to_string(),
        });
    }

    if let Some(dir) = path.parent()
        && let Ok(dir_handle) = std::fs::File::open(dir)
    {
        let _ = dir_handle.sync_all();
    }

    Ok(())
}
