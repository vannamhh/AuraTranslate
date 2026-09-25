//! Shared source-tree scanner for `tests/*_boundary.rs`.
//!
//! Lives under `tests/support/`, not directly under `tests/`: Cargo only treats files
//! placed directly under `tests/` as their own test binary, so this file never becomes an
//! extra 0-test target. A consumer loads it with
//! `#[path = "support/boundary_scan.rs"] mod boundary_scan;`, the same `#[path]` pattern
//! `tests/fixtures_docx.rs` uses.
//!
//! Not every consumer calls every function here; mark the unused ones
//! `#[allow(dead_code)]` at the `mod` declaration.

use std::fs;
use std::path::{Path, PathBuf};

pub fn src_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src")
}

/// Relative path with `/` separators on both platforms — required for correctness on
/// Windows, not cosmetic: `\`-separated paths never match a POSIX-style prefix check.
pub fn rel_posix(root: &Path, file: &Path) -> String {
    file.strip_prefix(root)
        .unwrap_or(file)
        .to_string_lossy()
        .replace('\\', "/")
}

/// `rel` is inside directory `dir`, matched on a directory boundary rather than a bare
/// string prefix (so `core/scope_legacy` does not match `core/scope`).
pub fn is_inside(rel: &str, dir: &str) -> bool {
    rel == dir || rel.starts_with(&format!("{dir}/"))
}

fn walk(dir: &Path, extensions: Option<&[&str]>, out: &mut Vec<PathBuf>) {
    let entries = fs::read_dir(dir).unwrap_or_else(|e| panic!("đọc {}: {e}", dir.display()));
    for entry in entries {
        let entry = entry.unwrap_or_else(|e| panic!("duyệt {}: {e}", dir.display()));
        let path = entry.path();
        let meta =
            fs::symlink_metadata(&path).unwrap_or_else(|e| panic!("lstat {}: {e}", path.display()));

        // `symlink_metadata`, not `metadata`: resolving a symlink back to a parent
        // directory would recurse forever.
        if meta.file_type().is_symlink() {
            continue;
        }
        if meta.is_dir() {
            walk(&path, extensions, out);
            continue;
        }
        let matches = match extensions {
            None => true,
            Some(exts) => path
                .extension()
                .and_then(|e| e.to_str())
                .is_some_and(|e| exts.contains(&e)),
        };
        if matches {
            out.push(path);
        }
    }
}

/// Every `.rs` file under `root`, with its POSIX-style relative path and content, sorted by
/// relative path.
pub fn rust_sources(root: &Path) -> Vec<(String, String)> {
    let mut files = Vec::new();
    walk(root, Some(&["rs"]), &mut files);
    files.sort();

    files
        .into_iter()
        .map(|file| {
            let rel = rel_posix(root, &file);
            let text =
                fs::read_to_string(&file).unwrap_or_else(|e| panic!("đọc {}: {e}", file.display()));
            (rel, text)
        })
        .collect()
}

/// Every file under `root` whose extension is in `extensions`, as absolute paths sorted by
/// path — for a tree that is not all `.rs` (e.g. the webview's `.ts`/`.vue` tree).
pub fn paths_with_extensions(root: &Path, extensions: &[&str]) -> Vec<PathBuf> {
    let mut files = Vec::new();
    walk(root, Some(extensions), &mut files);
    files.sort();
    files
}

/// Why [`any_sources`] skipped a file — never a panic.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Skip {
    /// File name starts with `.` (`.DS_Store`, `.gitkeep`, …).
    Dotfile,
    /// Read the bytes but they are not valid UTF-8.
    NonUtf8,
}

/// Every regular file under `root` (no extension filter), skipping symlinks and dotfiles,
/// read as UTF-8. Returns `(readable files, skipped files with a reason)` — never panics on
/// a dotfile or a non-UTF-8 file.
pub fn any_sources(root: &Path) -> (Vec<(String, String)>, Vec<(String, Skip)>) {
    let mut files = Vec::new();
    walk(root, None, &mut files);
    files.sort();

    let mut sources = Vec::new();
    let mut skipped = Vec::new();
    for file in files {
        let rel = rel_posix(root, &file);
        let is_dotfile = file
            .file_name()
            .and_then(|n| n.to_str())
            .is_some_and(|n| n.starts_with('.'));
        if is_dotfile {
            skipped.push((rel, Skip::Dotfile));
            continue;
        }
        match fs::read_to_string(&file) {
            Ok(text) => sources.push((rel, text)),
            Err(e) if e.kind() == std::io::ErrorKind::InvalidData => {
                skipped.push((rel, Skip::NonUtf8));
            }
            Err(e) => panic!("đọc {}: {e}", file.display()),
        }
    }
    (sources, skipped)
}

/// Length, in chars, of an escape sequence starting at `chars[at]` (the char right after a
/// `\`) — covers `\xHH`, `\u{…}` and a single escaped char.
fn skip_escape(chars: &[char], at: usize) -> usize {
    let n = chars.len();
    if at >= n {
        return at;
    }
    if chars[at] == 'u' && at + 1 < n && chars[at + 1] == '{' {
        let mut j = at + 2;
        while j < n && chars[j] != '}' {
            j += 1;
        }
        if j < n {
            j += 1;
        }
        return j;
    }
    if chars[at] == 'x' && at + 2 < n {
        return at + 3;
    }
    at + 1
}

/// If `chars[i]` opens a raw string (`r"…"`/`r#"…"#`/…, optionally `b`-prefixed), the index
/// just past the closing `"`; `None` otherwise.
fn raw_string_end(chars: &[char], i: usize) -> Option<usize> {
    let n = chars.len();
    let mut j = i;
    if j < n && chars[j] == 'b' {
        j += 1;
    }
    if j >= n || chars[j] != 'r' {
        return None;
    }
    j += 1;
    let mut hashes = 0usize;
    while j < n && chars[j] == '#' {
        hashes += 1;
        j += 1;
    }
    if j >= n || chars[j] != '"' {
        return None;
    }
    j += 1;
    loop {
        if j >= n {
            return Some(j);
        }
        if chars[j] == '"' {
            let close_quote = j;
            let mut k = j + 1;
            let mut matched = 0usize;
            while k < n && matched < hashes && chars[k] == '#' {
                matched += 1;
                k += 1;
            }
            if matched == hashes {
                return Some(k);
            }
            j = close_quote + 1;
            continue;
        }
        j += 1;
    }
}

/// If `chars[i]` opens a valid char literal (`'x'`/`'\''`/`'\u{2014}'`/…, optionally
/// `b`-prefixed), the index just past the closing `'`; `None` otherwise — including for a
/// lifetime (`'a`), which is never immediately followed by a second `'`.
fn char_literal_end(chars: &[char], i: usize) -> Option<usize> {
    let n = chars.len();
    let mut j = i;
    if j < n && chars[j] == 'b' {
        j += 1;
    }
    if j >= n || chars[j] != '\'' {
        return None;
    }
    j += 1;
    if j >= n {
        return None;
    }
    if chars[j] == '\\' {
        let after_escape = skip_escape(chars, j + 1);
        if after_escape < n && chars[after_escape] == '\'' {
            return Some(after_escape + 1);
        }
        return None;
    }
    if j + 1 < n && chars[j + 1] == '\'' {
        return Some(j + 2);
    }
    None
}

/// The extent of the syntax atom starting at `chars[i]`: a line comment (`//`), a nestable
/// block comment (`/* … */`), a raw string, a normal/byte string, or a char literal. Returns
/// `(index just past it, whether it is a comment)`, or `None` if `chars[i]` opens none of
/// those (plain code).
fn atom_end(chars: &[char], i: usize) -> Option<(usize, bool)> {
    let n = chars.len();

    if i + 1 < n && chars[i] == '/' && chars[i + 1] == '/' {
        let mut j = i;
        while j < n && chars[j] != '\n' {
            j += 1;
        }
        return Some((j, true));
    }

    if i + 1 < n && chars[i] == '/' && chars[i + 1] == '*' {
        let mut depth = 1usize;
        let mut j = i + 2;
        while j < n && depth > 0 {
            if j + 1 < n && chars[j] == '/' && chars[j + 1] == '*' {
                depth += 1;
                j += 2;
            } else if j + 1 < n && chars[j] == '*' && chars[j + 1] == '/' {
                depth -= 1;
                j += 2;
            } else {
                j += 1;
            }
        }
        return Some((j, true));
    }

    if let Some(end) = raw_string_end(chars, i) {
        return Some((end, false));
    }

    if chars[i] == '"' || (chars[i] == 'b' && i + 1 < n && chars[i + 1] == '"') {
        let mut j = i + if chars[i] == 'b' { 2 } else { 1 };
        while j < n {
            if chars[j] == '\\' && j + 1 < n {
                j += 2;
                continue;
            }
            if chars[j] == '"' {
                j += 1;
                break;
            }
            j += 1;
        }
        return Some((j, false));
    }

    if let Some(end) = char_literal_end(chars, i) {
        return Some((end, false));
    }

    None
}

/// Code lines of `text`: `(1-based line number, trimmed content with comments stripped)`.
///
/// Strips `//` to end of line and nestable `/* … */` (via [`atom_end`]), walking through
/// strings/raw strings/chars so a `/*` inside `"src/**/*.rs"` or `r#"/*"#`, or a `'"'` char
/// literal, is not mistaken for a comment delimiter. A line left empty after stripping
/// (including a line that was entirely a comment) is dropped from the result.
pub fn code_lines(text: &str) -> impl Iterator<Item = (usize, String)> {
    let chars: Vec<char> = text.chars().collect();
    let n = chars.len();
    let mut out = String::with_capacity(text.len());
    let mut i = 0usize;

    while i < n {
        if let Some((end, is_comment)) = atom_end(&chars, i) {
            if is_comment {
                for &ch in &chars[i..end] {
                    out.push(if ch == '\n' { '\n' } else { ' ' });
                }
            } else {
                out.extend(&chars[i..end]);
            }
            i = end;
            continue;
        }
        out.push(chars[i]);
        i += 1;
    }

    let lines: Vec<(usize, String)> = out
        .lines()
        .enumerate()
        .map(|(index, line)| (index + 1, line.trim().to_string()))
        .filter(|(_, code)| !code.is_empty())
        .collect();
    lines.into_iter()
}

/// The char index (not byte index) of the `}` matching a `{` already open at depth 1, in
/// `chars[start..]`, skipping any brace inside a comment/string/char literal (via
/// [`atom_end`]). `None` if unbalanced.
fn matching_close_brace(chars: &[char], start: usize) -> Option<usize> {
    let n = chars.len();
    let mut depth = 1usize;
    let mut i = start;
    while i < n {
        if let Some((end, _)) = atom_end(chars, i) {
            i = end;
            continue;
        }
        match chars[i] {
            '{' => depth += 1,
            '}' => {
                depth -= 1;
                if depth == 0 {
                    return Some(i);
                }
            }
            _ => {}
        }
        i += 1;
    }
    None
}

/// `text` with every `#[cfg(test)] mod NAME { … }` block removed (line count preserved: a
/// removed char other than `\n` becomes nothing, `\n` stays, so line numbers after the block
/// are unchanged). Product code after the block is still scanned.
///
/// Anchors on a line (`line.trim() == "#[cfg(test)]"`), not `str::find` over the whole text,
/// so a comment that only mentions that string is not mistaken for the real attribute.
///
/// Three shapes follow a `#[cfg(test)]` line (blank lines in between are skipped):
/// - `mod NAME { … }` — the block is removed.
/// - `mod NAME;` — the module body lives in another file, so there is nothing to remove
///   here; the `#[cfg(test)]` line is left in place and scanning continues right after it.
/// - anything else (no `mod `, or `mod NAME {` with unbalanced braces) — panics; an
///   unsupported shape is a loud failure, not a silent skip.
pub fn without_test_modules(text: &str) -> String {
    let mut result = String::with_capacity(text.len());
    let mut remaining = text;

    loop {
        let mut attr_start: Option<usize> = None;
        let mut offset = 0usize;
        for line in remaining.split_inclusive('\n') {
            if line.trim() == "#[cfg(test)]" {
                attr_start = Some(offset);
                break;
            }
            offset += line.len();
        }

        let Some(attr_start) = attr_start else {
            result.push_str(remaining);
            break;
        };

        result.push_str(&remaining[..attr_start]);
        let after_attr = &remaining[attr_start..];

        let mut lines_iter = after_attr.split_inclusive('\n');
        let attr_line = lines_iter
            .next()
            .unwrap_or_else(|| panic!("vừa khớp dòng `#[cfg(test)]` nên phải tồn tại"));

        let mut probe_offset = attr_line.len();
        let mut mod_line: Option<&str> = None;
        let mut mod_line_offset = probe_offset;
        for line in after_attr[probe_offset..].split_inclusive('\n') {
            if line.trim().is_empty() {
                probe_offset += line.len();
                continue;
            }
            mod_line = Some(line);
            mod_line_offset = probe_offset;
            break;
        }

        let mod_line = mod_line.unwrap_or_else(|| {
            panic!(
                "`#[cfg(test)]` không theo sau bởi một dòng `mod …` nào (hết tệp) -- hình \
                 dạng ngoài phạm vi hỗ trợ, xem doc-comment `without_test_modules`"
            )
        });
        let trimmed_mod_line = mod_line.trim_start();

        if !trimmed_mod_line.starts_with("mod ") {
            panic!(
                "`#[cfg(test)]` không theo sau bởi một dòng `mod …` -- hình dạng ngoài phạm \
                 vi hỗ trợ, xem doc-comment `without_test_modules`: {mod_line:?}"
            );
        }

        if trimmed_mod_line.trim_end().ends_with(';') && !mod_line.contains('{') {
            // `mod NAME;`: nothing to remove here, the body lives elsewhere.
            result.push_str(after_attr[..mod_line_offset + mod_line.len()].as_ref());
            remaining = &after_attr[mod_line_offset + mod_line.len()..];
            continue;
        }

        let Some(brace_rel) = mod_line.find('{') else {
            panic!(
                "`#[cfg(test)]` theo sau bởi `mod …` không phải `mod NAME {{ … }}` cũng không \
                 phải `mod NAME;` -- hình dạng ngoài phạm vi hỗ trợ: {mod_line:?}"
            );
        };

        let open_brace_offset = mod_line_offset + brace_rel;
        let after_open = &after_attr[open_brace_offset + 1..];
        let chars: Vec<char> = after_open.chars().collect();

        let Some(close_char_idx) = matching_close_brace(&chars, 0) else {
            panic!(
                "`#[cfg(test)] mod … {{ … }}` không cân dấu ngoặc -- hình dạng ngoài phạm vi \
                 hỗ trợ, xem doc-comment `without_test_modules`"
            );
        };

        let close_byte_offset: usize = after_open
            .char_indices()
            .nth(close_char_idx)
            .map(|(b, _)| b)
            .unwrap_or(after_open.len());
        let after_close_byte_len = after_open[close_byte_offset..]
            .chars()
            .next()
            .map(char::len_utf8)
            .unwrap_or(0);
        let after_close_byte = close_byte_offset + after_close_byte_len;

        let removed_span = &after_attr[..open_brace_offset + 1 + after_close_byte];
        for ch in removed_span.chars() {
            if ch == '\n' {
                result.push('\n');
            }
        }

        remaining = &after_attr[open_brace_offset + 1 + after_close_byte..];
    }

    result
}
