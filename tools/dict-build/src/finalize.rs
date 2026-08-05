//! Hoàn tất tệp: `rebuild` ba chỉ mục FTS5, `ANALYZE`, `VACUUM`, và 🔴
//! `journal_mode = DELETE` (Task 6 của Story 1.9 — Bẫy 1, bẫy đắt nhất của story: một
//! tệp còn ở WAL cần quyền ghi vào thư mục chứa nó để dựng `-shm`, mà `$RESOURCE/dict/`
//! trên máy người dùng chỉ đọc, AD-7/AD-23).

use std::path::{Path, PathBuf};

use rusqlite::Connection;
use sha2::{Digest, Sha256};

/// Đường dẫn "anh em" của `path` — nối THẲNG `suffix` vào tên tệp đầy đủ, ⛔ không thay
/// đuôi `.db` bằng `with_extension` (sai khi `path` không có đúng đuôi `.db`: SQLite đặt
/// tên WAL/SHM bằng cách nối `-wal`/`-shm` vào TÊN TỆP CHÍNH, không phải bằng cách đổi
/// đuôi). `foo.db` → `foo.db-wal`; `foo` (không đuôi) → `foo-wal` — cả hai case đều khớp
/// hành vi thật của SQLite, khác với `foo.with_extension("db-wal")` vốn cho `foo.db-wal`
/// SAI khi input không có `.db`.
pub fn sibling_path(path: &Path, suffix: &str) -> PathBuf {
    let mut s = path.as_os_str().to_os_string();
    s.push(suffix);
    PathBuf::from(s)
}

/// Dọn sạch mọi tệp cũ ở `out_path`/tệp `.tmp` cạnh nó (cộng `-wal`/`-shm`) — LUÔN là một
/// lượt dựng MỚI từ đầu, ⛔ không phải cập nhật tệp cũ. Trả về đường dẫn tệp TẠM để dựng
/// vào. Dùng chung cho CẢ BA đường dựng — base lẫn từng lớp gỡ rời (Task 5, Story 1.10).
pub fn prepare_fresh_output(out_path: &Path) -> std::io::Result<PathBuf> {
    let tmp_path = sibling_path(out_path, ".tmp");
    for p in [out_path, &tmp_path] {
        if p.exists() {
            std::fs::remove_file(p)?;
        }
        for suffix in ["-wal", "-shm"] {
            let sib = sibling_path(p, suffix);
            if sib.exists() {
                std::fs::remove_file(&sib)?;
            }
        }
    }
    Ok(tmp_path)
}

/// `rebuild` cả ba bảng FTS5 external-content — KHÔNG có bước này thì `MATCH` trả 0
/// hàng, không lỗi (Bẫy 3). Chạy TRƯỚC `VACUUM`.
pub fn rebuild_fts(conn: &Connection) -> rusqlite::Result<()> {
    conn.execute_batch(
        "INSERT INTO entry_fts(entry_fts) VALUES('rebuild');
         INSERT INTO sense_fts(sense_fts) VALUES('rebuild');
         INSERT INTO sense_fts_nd(sense_fts_nd) VALUES('rebuild');",
    )
}

/// `ANALYZE` rồi `VACUUM` — điều kiện để số đo AC6 là số thật, không phải số có lỗ.
pub fn analyze_and_vacuum(conn: &Connection) -> rusqlite::Result<()> {
    conn.execute_batch("ANALYZE;")?;
    conn.execute_batch("VACUUM;")?;
    Ok(())
}

/// 🔴 Đặt `journal_mode = DELETE` — LUẬT không thương lượng của story này. `$RESOURCE/
/// dict/` là chỉ đọc trên máy người dùng; WAL cần ghi `-shm` vào thư mục chứa tệp, lỗi
/// đó chạy hoàn hảo suốt lúc phát triển và chỉ lộ ra ở lần tra cứu đầu tiên của người
/// dùng thật.
pub fn set_journal_mode_delete(conn: &Connection) -> rusqlite::Result<String> {
    conn.query_row("PRAGMA journal_mode = DELETE", [], |row| row.get(0))
}

/// Kiểm ngay sau khi đóng: `journal_mode` phải là `delete`, và KHÔNG tệp `-wal`/`-shm`
/// nào còn sót cạnh `.db`. Trả `Err` với thông điệp rõ ràng nếu vi phạm — đây là lưới
/// chặn Bẫy 1 chạy được ở CI, không chỉ ở tài liệu.
pub fn verify_no_wal_artifacts(db_path: &Path) -> Result<(), String> {
    let wal = sibling_path(db_path, "-wal");
    let shm = sibling_path(db_path, "-shm");
    let mut leftovers = Vec::new();
    if wal.exists() {
        leftovers.push(wal.display().to_string());
    }
    if shm.exists() {
        leftovers.push(shm.display().to_string());
    }
    if leftovers.is_empty() {
        Ok(())
    } else {
        Err(format!(
            "journal_mode=DELETE nhưng vẫn còn tệp WAL cạnh .db (Bẫy 1): {}",
            leftovers.join(", ")
        ))
    }
}

/// SHA-256 và kích thước byte của tệp cuối — in ra để `dict-manifest.toml` chép-dán
/// (AC3), và là dữ liệu bảng kế toán AC6.
///
/// Băm theo LUỒNG (khối 64 KiB), ⛔ không nạp trọn tệp vào bộ nhớ qua `std::fs::read` —
/// tệp này đã ~155 MB và sẽ còn lớn hơn sau khi Story 1.10 thêm bốn lớp gỡ rời.
pub fn sha256_and_size(path: &Path) -> std::io::Result<(String, u64)> {
    use std::io::Read;

    let mut file = std::fs::File::open(path)?;
    let mut hasher = Sha256::new();
    let mut buf = [0u8; 65536];
    let mut size: u64 = 0;
    loop {
        let n = file.read(&mut buf)?;
        if n == 0 {
            break;
        }
        hasher.update(&buf[..n]);
        size += n as u64;
    }
    let digest = hasher.finalize();
    let hex: String = digest.iter().map(|b| format!("{b:02x}")).collect();
    Ok((hex, size))
}

/// 🔴 Phần ĐUÔI dùng chung cho MỌI lượt dựng — base lẫn TỪNG lớp gỡ rời (Task 5, Bẫy 2:
/// một trong hai đường dựng bỏ `journal_mode = DELETE` là bẫy đắt nhất, nhân đôi so với
/// Story 1.9 vì giờ có ba đường dựng). rebuild FTS → ANALYZE/VACUUM →
/// `journal_mode = DELETE` → kiểm no-wal → băm → `rename` từ `.tmp` sang `out_path`.
/// Trả `(sha256, size_bytes, journal_mode)` — caller (`build.rs`) đóng gói vào
/// `BuildReport` cùng `per_source`/`char_idx_pairs` mà nó tự biết.
pub fn finish(
    conn: Connection,
    tmp_path: &Path,
    out_path: &Path,
) -> Result<(String, u64, String), Box<dyn std::error::Error>> {
    rebuild_fts(&conn)?;
    analyze_and_vacuum(&conn)?;
    let journal_mode = set_journal_mode_delete(&conn)?;
    // PRAGMA journal_mode không báo lỗi khi bị SQLite âm thầm từ chối đổi — nó chỉ trả
    // về chế độ ĐANG thật sự có hiệu lực. Không xác nhận ở đây thì một lượt chuyển chế
    // độ bị từ chối vẫn cho ra ExitCode::SUCCESS (Bẫy 1).
    if journal_mode.to_lowercase() != "delete" {
        return Err(format!(
            "journal_mode vẫn là '{journal_mode}' sau khi yêu cầu DELETE — Bẫy 1 chưa được khép kín"
        )
        .into());
    }
    drop(conn); // đóng kết nối TRƯỚC khi kiểm tệp -wal/-shm cạnh .db

    verify_no_wal_artifacts(tmp_path).map_err(|e| -> Box<dyn std::error::Error> { e.into() })?;
    let (sha256, size_bytes) = sha256_and_size(tmp_path)?;

    std::fs::rename(tmp_path, out_path)?;

    Ok((sha256, size_bytes, journal_mode))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn sha256_matches_a_known_vector() {
        let dir = std::env::temp_dir().join(format!("dict-build-test-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("empty.bin");
        std::fs::File::create(&path).unwrap().write_all(b"").unwrap();
        let (hex, size) = sha256_and_size(&path).unwrap();
        // SHA-256 của chuỗi rỗng — vector chuẩn NIST, không phải số tự bịa.
        assert_eq!(
            hex,
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
        assert_eq!(size, 0);
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn no_wal_artifacts_passes_when_clean() {
        let dir = std::env::temp_dir().join(format!("dict-build-test2-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let db_path = dir.join("dict-core.db");
        assert!(verify_no_wal_artifacts(&db_path).is_ok());
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn detects_leftover_wal_file() {
        let dir = std::env::temp_dir().join(format!("dict-build-test3-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let db_path = dir.join("dict-core.db");
        std::fs::write(dir.join("dict-core.db-wal"), b"x").unwrap();
        assert!(verify_no_wal_artifacts(&db_path).is_err());
        std::fs::remove_dir_all(&dir).ok();
    }

    /// `--out` không có đúng đuôi `.db` — `with_extension` từng tính SAI đường dẫn WAL
    /// thật (`foo-wal`, SQLite nối thẳng hậu tố vào TÊN TỆP CHÍNH), khiến lưới chặn Bẫy 1
    /// kiểm nhầm tệp và bỏ lọt leftover thật. `sibling_path` phải tính đúng bất kể đuôi.
    #[test]
    fn detects_leftover_wal_file_when_out_path_has_no_dot_db_extension() {
        let dir = std::env::temp_dir().join(format!("dict-build-test4-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let db_path = dir.join("dict-core"); // không đuôi .db, kiểu --out hợp lệ nhưng khác quy ước
        assert_eq!(sibling_path(&db_path, "-wal"), dir.join("dict-core-wal"));
        assert_eq!(sibling_path(&db_path, "-shm"), dir.join("dict-core-shm"));

        assert!(verify_no_wal_artifacts(&db_path).is_ok());
        std::fs::write(dir.join("dict-core-wal"), b"x").unwrap();
        assert!(
            verify_no_wal_artifacts(&db_path).is_err(),
            "leftover WAL file at the REAL SQLite-derived path must be detected"
        );
        std::fs::remove_dir_all(&dir).ok();
    }
}
