//! The only file I/O of `core/tm`: reading a TMX file and writing an export atomically.

use std::io::Read as _;
use std::path::Path;

use uuid::Uuid;

use super::tmx::{MAX_TMX_BYTES, TmxError, decode_tmx_bytes};

/// Reads at most `MAX_TMX_BYTES + 1` bytes, so an oversized file is refused without loading it.
pub fn read_tmx_file(path: &Path) -> Result<String, TmxError> {
    let read_failed = |e: std::io::Error| TmxError::ReadFailed { detail: format!("{}: {e}", path.display()) };
    let file = std::fs::File::open(path).map_err(read_failed)?;
    let mut bytes = Vec::new();
    file.take(MAX_TMX_BYTES + 1).read_to_end(&mut bytes).map_err(read_failed)?;
    decode_tmx_bytes(&bytes)
}

/// Writes `contents` to a unique temp file beside `path`, then renames it over `path`. Contents
/// the importer would refuse (`MAX_TMX_BYTES`) are refused before any file is created.
pub fn write_tmx_file(path: &Path, contents: &str) -> Result<(), TmxError> {
    if contents.len() as u64 > MAX_TMX_BYTES {
        return Err(TmxError::TooLarge { limit: MAX_TMX_BYTES });
    }
    let failed = |detail: String| TmxError::WriteFailed { detail: format!("{}: {detail}", path.display()) };
    let Some(file_name) = path.file_name() else {
        return Err(failed("no file name".to_owned()));
    };
    let mut tmp_name = file_name.to_os_string();
    tmp_name.push(format!(".{}-{}.tmp", std::process::id(), Uuid::new_v4()));
    let tmp = path.with_file_name(tmp_name);

    let written = (|| -> std::io::Result<()> {
        use std::io::Write as _;
        let mut file = std::fs::File::create(&tmp)?;
        file.write_all(contents.as_bytes())?;
        file.sync_all()
    })();
    if let Err(e) = written {
        let _ = std::fs::remove_file(&tmp);
        return Err(failed(e.to_string()));
    }
    if let Err(e) = std::fs::rename(&tmp, path) {
        let _ = std::fs::remove_file(&tmp);
        return Err(failed(e.to_string()));
    }
    if let Some(dir) = path.parent()
        && let Ok(handle) = std::fs::File::open(dir)
    {
        let _ = handle.sync_all();
    }
    Ok(())
}
