use std::fs::{self, File, OpenOptions};
use std::io::ErrorKind;
use std::path::{Path, PathBuf};

use super::new_file::{MAX_ATTEMPTS, candidate_stem, create_and_write};
use super::table_rows::ExportImage;

pub const IMAGE_DIR_SUFFIX: &str = "-anh";

#[derive(Debug)]
pub enum ImageFilesError {
    /// Hàng `asset` có nhưng tệp trong `assets/` không còn (hoặc tên tệp không phải tên trần).
    SourceMissing { chapter_ord: i64, file_name: String },
    Write(String),
}

/// Tên tệp ảnh trong thư mục xuất; tiền tố `asset.id` giữ hai ảnh trùng tên không đè nhau.
pub fn copied_name(image: &ExportImage) -> String {
    format!("{}-{}", image.asset_id, image.file_name)
}

#[derive(Debug)]
pub struct WrittenWithImages {
    pub file_path: PathBuf,
    pub images_dir: PathBuf,
}

fn source_of(assets_dir: &Path, image: &ExportImage) -> Result<PathBuf, ImageFilesError> {
    let missing = || ImageFilesError::SourceMissing { chapter_ord: image.chapter_ord, file_name: image.file_name.clone() };
    let plain = Path::new(&image.file_name).file_name().and_then(|n| n.to_str()) == Some(image.file_name.as_str());
    let path = assets_dir.join(&image.file_name);
    if plain && path.is_file() { Ok(path) } else { Err(missing()) }
}

fn copy_all(sources: &[(PathBuf, String)], dir: &Path) -> std::io::Result<()> {
    for (source, name) in sources {
        let mut from = File::open(source)?;
        let mut to = OpenOptions::new().write(true).create_new(true).open(dir.join(name))?;
        std::io::copy(&mut from, &mut to)?;
        to.sync_all()?;
    }
    Ok(())
}

/// Chọn stem cuối cùng mà cả `<stem>.<extension>` lẫn `<stem>-anh/` đều chưa có, tạo thư mục
/// ảnh và chép ảnh vào, rồi mới ghi tệp do `build(stem)` dựng. Lỗi giữa chừng xoá thư mục vừa tạo;
/// không bao giờ ghi đè thứ có sẵn.
pub fn write_file_with_images(
    folder: &Path,
    stem: &str,
    extension: &str,
    assets_dir: &Path,
    images: &[&ExportImage],
    build: impl Fn(&str) -> Result<Vec<u8>, String>,
) -> Result<WrittenWithImages, ImageFilesError> {
    let sources = images
        .iter()
        .map(|image| source_of(assets_dir, image).map(|path| (path, copied_name(image))))
        .collect::<Result<Vec<_>, _>>()?;
    let io = |e: std::io::Error| ImageFilesError::Write(e.to_string());
    for attempt in 1..=MAX_ATTEMPTS {
        let final_stem = candidate_stem(stem, attempt);
        let file_path = folder.join(format!("{final_stem}.{extension}"));
        let images_dir = folder.join(format!("{final_stem}{IMAGE_DIR_SUFFIX}"));
        if file_path.exists() || images_dir.exists() {
            continue;
        }
        match fs::create_dir(&images_dir) {
            Ok(()) => {}
            Err(e) if e.kind() == ErrorKind::AlreadyExists => continue,
            Err(e) => return Err(io(e)),
        }
        let outcome = copy_all(&sources, &images_dir)
            .map_err(io)
            .and_then(|()| build(&final_stem).map_err(ImageFilesError::Write))
            .and_then(|bytes| create_and_write(&file_path, &bytes).map_err(io));
        match outcome {
            Ok(()) => return Ok(WrittenWithImages { file_path, images_dir }),
            Err(ImageFilesError::Write(_)) if file_path.exists() => {
                let _ = fs::remove_dir_all(&images_dir);
            }
            Err(e) => {
                let _ = fs::remove_dir_all(&images_dir);
                return Err(e);
            }
        }
    }
    Err(ImageFilesError::Write("no free file name".to_owned()))
}
