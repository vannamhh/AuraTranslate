use std::fs::OpenOptions;
use std::io::{ErrorKind, Write as _};
use std::path::{Path, PathBuf};

const STEM_MAX_CHARS: usize = 100;
const MAX_ATTEMPTS: u32 = 1000;

/// Tên tệp an toàn trên Windows, macOS và Linux; rỗng thì về `fallback`.
pub fn safe_stem(raw: &str, fallback: &str) -> String {
    let cleaned: String = raw
        .chars()
        .map(|c| if c.is_control() || "/\\:*?\"<>|".contains(c) { '_' } else { c })
        .take(STEM_MAX_CHARS)
        .collect();
    let trimmed = cleaned.trim_matches(|c: char| c == '.' || c.is_whitespace());
    if trimmed.is_empty() { fallback.to_owned() } else { trimmed.to_owned() }
}

/// Ghi `bytes` vào `folder/<stem>.<extension>`; tên đã có thì thử `<stem> (2)`, `(3)`, …
/// và không bao giờ ghi đè. Trả đường dẫn đã ghi.
pub fn write_new_file(folder: &Path, stem: &str, extension: &str, bytes: &[u8]) -> std::io::Result<PathBuf> {
    for attempt in 1..=MAX_ATTEMPTS {
        let name = if attempt == 1 {
            format!("{stem}.{extension}")
        } else {
            format!("{stem} ({attempt}).{extension}")
        };
        let path = folder.join(name);
        match OpenOptions::new().write(true).create_new(true).open(&path) {
            Ok(mut file) => {
                let written = file.write_all(bytes).and_then(|()| file.sync_all());
                if let Err(e) = written {
                    drop(file);
                    let _ = std::fs::remove_file(&path);
                    return Err(e);
                }
                return Ok(path);
            }
            Err(e) if e.kind() == ErrorKind::AlreadyExists => {}
            Err(e) => return Err(e),
        }
    }
    Err(std::io::Error::new(ErrorKind::AlreadyExists, "no free file name"))
}
