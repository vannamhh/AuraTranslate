//! Điều phối: đọc raw → parse → chèn → char_idx → rebuild FTS → ANALYZE/VACUUM →
//! journal_mode=DELETE. Task 2–6, 11 của Story 1.9.

use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

use rusqlite::Connection;

use crate::model::SourceStats;
use crate::{finalize, insert, sources, sources_meta};

/// Tổng hợp kết quả một lượt build — in ra bảng cuối, và caller dùng để quyết định mã
/// thoát (một nguồn đọc hỏng nặng vẫn nên dừng và báo, ⛔ không lặng lẽ sinh tệp thiếu —
/// thi hành ở `require_nonempty`, Review Findings Group A).
pub struct BuildReport {
    pub per_source: Vec<SourceStats>,
    pub char_idx_pairs: i64,
    pub sha256: String,
    pub size_bytes: u64,
    pub journal_mode: String,
}

/// Đọc TỐI ĐA `max_lines` dòng đầu của tệp — dùng để dò header (`source_version`), ⛔
/// không đọc trọn tệp chỉ để xem vài dòng đầu (Review Findings Group A). Dòng nào không
/// giải mã được UTF-8 thì BỎ QUA thay vì hỏng cả lượt đọc — đây chỉ là dò header, không
/// phải đường parse chính (đường đó đã có `ParseIssue` riêng cho từng dòng).
fn read_header_lines(path: &Path, max_lines: usize) -> std::io::Result<Vec<String>> {
    let f = File::open(path)?;
    Ok(BufReader::new(f)
        .lines()
        .take(max_lines)
        .filter_map(|l| l.ok())
        .collect())
}

/// Tìm `# Unicode Version X.Y.Z` trong vài dòng đầu của một tệp Unihan — `source_version`
/// của Unihan (§Thông tin kỹ thuật của Story 1.9), ⛔ không viết cứng.
fn unihan_source_version(header_lines: &[String]) -> Option<String> {
    for l in header_lines.iter().take(16) {
        if let Some(rest) = l.strip_prefix("# Unicode Version ") {
            return Some(rest.trim().to_string());
        }
    }
    None
}

/// `source_version` đo được, hay CẢNH BÁO ra stderr rồi rơi về `"unknown"` — âm thầm
/// rơi về `"unknown"` từng làm một nguồn đo hỏng lọt qua `check-dict-manifest.mjs` (chỉ
/// đòi khác rỗng, và `"unknown"` không rỗng) mà không ai biết (Review Findings Group A).
fn version_or_warn(source_code: &str, detected: Option<String>) -> String {
    match detected {
        Some(v) => v,
        None => {
            eprintln!(
                "dict-build: CẢNH BÁO — không đo được source_version của '{source_code}', dùng 'unknown'"
            );
            "unknown".to_string()
        }
    }
}

/// Một nguồn cho ra 0 entry là nguồn đọc hỏng nặng (tệp sai/rỗng/hỏng mã hoá) — dừng và
/// báo, ⛔ không lặng lẽ sinh tệp thiếu nguồn (doc-comment `BuildReport`, Review Findings
/// Group A: trước đây build vẫn `ExitCode::SUCCESS` dù một nguồn đọc ra 0 entry).
fn require_nonempty(stats: &SourceStats) -> Result<(), Box<dyn std::error::Error>> {
    if stats.entries == 0 {
        return Err(format!(
            "nguồn '{}' cho ra 0 entry (đọc {} dòng, bỏ {}) — dừng build, không sinh tệp thiếu nguồn",
            stats.source_code, stats.lines_read, stats.lines_skipped
        )
        .into());
    }
    Ok(())
}

fn ingest<I>(
    conn: &Connection,
    source_id: i64,
    stats: &mut SourceStats,
    iter: I,
) -> rusqlite::Result<()>
where
    I: Iterator<Item = Result<crate::model::RawEntry, crate::model::ParseIssue>>,
{
    for item in iter {
        stats.lines_read += 1;
        match item {
            Ok(entry) => {
                insert::insert_entry(conn, source_id, &entry)?;
                stats.record_entry(&entry);
            }
            Err(issue) => stats.record_skip(&issue.reason),
        }
    }
    Ok(())
}

/// Chạy trọn lượt build. `raw_dir` chứa năm thư mục con (`cvdict/`, `cc_cedict/`,
/// `unihan/`, `viwiktionary/`, `en_wiktionary/`) theo quy ước đã ghi ở
/// `tools/dict-build/README.md`. `out_path` là `.db` đích.
///
/// Dựng vào một tệp TẠM cùng thư mục với `out_path`, chỉ đổi tên sang `out_path` SAU KHI
/// mọi bước (rebuild FTS, ANALYZE/VACUUM, journal_mode=DELETE, kiểm no-wal, băm) đã
/// xong — một lượt build hỏng giữa chừng không còn để lại tệp dở dang TẠI `out_path`
/// (Review Findings Group A; trước đây chỉ phân biệt được qua exit code/stderr). Mọi
/// tệp cũ ở `out_path`/`tmp_path` (cộng `-wal`/`-shm` cạnh chúng) bị xoá trước khi dựng,
/// vì đây LUÔN là một lượt dựng MỚI từ đầu, ⛔ không phải cập nhật tệp cũ.
pub fn run(raw_dir: &Path, out_path: &Path) -> Result<BuildReport, Box<dyn std::error::Error>> {
    let tmp_path = finalize::sibling_path(out_path, ".tmp");

    for p in [out_path, &tmp_path] {
        if p.exists() {
            std::fs::remove_file(p)?;
        }
        for suffix in ["-wal", "-shm"] {
            let sib = finalize::sibling_path(p, suffix);
            if sib.exists() {
                std::fs::remove_file(&sib)?;
            }
        }
    }

    let mut conn = Connection::open(&tmp_path)?;
    conn.execute_batch("PRAGMA foreign_keys = ON;")?;

    insert::create_schema(&conn)?;

    let mut per_source = Vec::new();

    // Mọi lượt chèn (dict_meta + năm nguồn + char_idx) chạy trong MỘT transaction —
    // trước đây mỗi INSERT tự autocommit, nên một lỗi giữa chừng để lại hàng mồ côi
    // (Review Findings Group A; đây cũng là điều kiện để doc-comment "cùng giao dịch"
    // của `char_idx::insert_for_entry` đúng nghĩa đen). `VACUUM`/thay `journal_mode`
    // không chạy được TRONG transaction nên nằm NGOÀI khối này, sau khi đã `commit`.
    {
        let tx = conn.transaction()?;
        insert::insert_meta(&tx)?;

        // ── CVDICT ──────────────────────────────────────────────────────────────
        {
            let dir = raw_dir.join("cvdict");
            let version = version_or_warn(
                sources::cvdict::SOURCE_CODE,
                std::fs::read_to_string(dir.join("SOURCE_VERSION.txt"))
                    .ok()
                    .map(|s| s.trim().to_string()),
            );
            let source_id = insert::insert_source(&tx, &sources_meta::CVDICT, &version)?;
            let mut stats = SourceStats::new(sources::cvdict::SOURCE_CODE);
            let f = File::open(dir.join("CVDICT.u8"))?;
            ingest(
                &tx,
                source_id,
                &mut stats,
                sources::cvdict::parse(BufReader::new(f)),
            )?;
            require_nonempty(&stats)?;
            per_source.push(stats);
        }

        // ── CC-CEDICT ───────────────────────────────────────────────────────────
        {
            let dir = raw_dir.join("cc_cedict");
            let path = dir.join("cedict.txt");
            let header = read_header_lines(&path, 64)?;
            let version = version_or_warn(
                sources::cc_cedict::SOURCE_CODE,
                sources::cc_cedict::source_version(&header),
            );
            let source_id = insert::insert_source(&tx, &sources_meta::CC_CEDICT, &version)?;
            let mut stats = SourceStats::new(sources::cc_cedict::SOURCE_CODE);
            let f = File::open(&path)?;
            ingest(
                &tx,
                source_id,
                &mut stats,
                sources::cc_cedict::parse(BufReader::new(f)),
            )?;
            require_nonempty(&stats)?;
            per_source.push(stats);
        }

        // ── Unihan ──────────────────────────────────────────────────────────────
        {
            let dir = raw_dir.join("unihan");
            let readings_header = read_header_lines(&dir.join("Unihan_Readings.txt"), 16)?;
            let version = version_or_warn(
                sources::unihan::SOURCE_CODE,
                unihan_source_version(&readings_header),
            );
            let source_id = insert::insert_source(&tx, &sources_meta::UNIHAN, &version)?;
            let mut stats = SourceStats::new(sources::unihan::SOURCE_CODE);

            // §Bẫy: kProperty tự mô tả, không phụ thuộc tên tệp gốc — nối Readings +
            // Variants thành MỘT reader (module doc-comment của `sources::unihan`).
            // Đọc BYTE THÔ (`std::fs::read`), ⛔ không `read_to_string` — một byte
            // UTF-8 lỗi ở BẤT KỲ ĐÂU trong tệp trước đây hỏng cả lượt build (`?` ném
            // trước khi bất kỳ `ParseIssue` nào được đếm); đọc byte thô rồi để
            // `BufRead::lines()` giải mã TỪNG DÒNG cho phép lỗi cục bộ rơi đúng vào
            // `ParseIssue` như `sources::unihan::parse` đã viết sẵn cho ca này
            // (Review Findings Group A).
            let mut combined: Vec<u8> = Vec::new();
            for fname in ["Unihan_Readings.txt", "Unihan_Variants.txt"] {
                combined.extend_from_slice(&std::fs::read(dir.join(fname))?);
                combined.push(b'\n');
            }
            ingest(
                &tx,
                source_id,
                &mut stats,
                sources::unihan::parse(std::io::Cursor::new(combined)),
            )?;
            require_nonempty(&stats)?;
            per_source.push(stats);
        }

        // ── viwiktionary ────────────────────────────────────────────────────────
        {
            let dir = raw_dir.join("viwiktionary");
            let path = dir.join("vi-extract.jsonl");
            let version =
                version_or_warn(sources::viwiktionary::SOURCE_CODE, file_mtime_date(&path));
            let source_id = insert::insert_source(&tx, &sources_meta::VIWIKTIONARY, &version)?;
            let mut stats = SourceStats::new(sources::viwiktionary::SOURCE_CODE);
            let f = File::open(&path)?;
            ingest(
                &tx,
                source_id,
                &mut stats,
                sources::viwiktionary::parse(BufReader::new(f)),
            )?;
            require_nonempty(&stats)?;
            per_source.push(stats);
        }

        // ── en.wiktionary (mục tiếng Trung) ────────────────────────────────────
        {
            let dir = raw_dir.join("en_wiktionary");
            let path = dir.join("Chinese.jsonl");
            let version =
                version_or_warn(sources::en_wiktionary::SOURCE_CODE, file_mtime_date(&path));
            let source_id = insert::insert_source(&tx, &sources_meta::EN_WIKTIONARY, &version)?;
            let mut stats = SourceStats::new(sources::en_wiktionary::SOURCE_CODE);
            let f = File::open(&path)?;
            ingest(
                &tx,
                source_id,
                &mut stats,
                sources::en_wiktionary::parse(BufReader::new(f)),
            )?;
            require_nonempty(&stats)?;
            per_source.push(stats);
        }

        tx.commit()?;
    }

    let char_idx_pairs: i64 = conn.query_row("SELECT COUNT(*) FROM char_idx", [], |r| r.get(0))?;

    finalize::rebuild_fts(&conn)?;
    finalize::analyze_and_vacuum(&conn)?;
    let journal_mode = finalize::set_journal_mode_delete(&conn)?;
    // PRAGMA journal_mode không báo lỗi khi bị SQLite âm thầm từ chối đổi — nó chỉ trả
    // về chế độ ĐANG thật sự có hiệu lực. Không xác nhận ở đây thì một lượt chuyển chế
    // độ bị từ chối vẫn cho ra `ExitCode::SUCCESS` (Review Findings Group A, Bẫy 1).
    if journal_mode.to_lowercase() != "delete" {
        return Err(format!(
            "journal_mode vẫn là '{journal_mode}' sau khi yêu cầu DELETE — Bẫy 1 chưa được khép kín"
        )
        .into());
    }
    drop(conn); // đóng kết nối TRƯỚC khi kiểm tệp -wal/-shm cạnh .db

    finalize::verify_no_wal_artifacts(&tmp_path)
        .map_err(|e| -> Box<dyn std::error::Error> { e.into() })?;
    let (sha256, size_bytes) = finalize::sha256_and_size(&tmp_path)?;

    std::fs::rename(&tmp_path, out_path)?;

    Ok(BuildReport {
        per_source,
        char_idx_pairs,
        sha256,
        size_bytes,
        journal_mode,
    })
}

/// `source_version` cho hai nguồn Wiktionary — không có header ngày tháng trong nội
/// dung, nên dùng thời điểm sửa đổi cuối của tệp đã tải (ngày dump, xấp xỉ đúng tinh
/// thần "phiên bản nguồn thô" khi nguồn không tự khai).
fn file_mtime_date(path: &Path) -> Option<String> {
    let meta = std::fs::metadata(path).ok()?;
    let modified = meta.modified().ok()?;
    let secs = modified
        .duration_since(std::time::UNIX_EPOCH)
        .ok()?
        .as_secs();
    // Không kéo `chrono`/`time` cho một lần đổi định dạng — dùng chính SQLite để đổi
    // epoch giây sang ISO-8601 UTC, khớp Consistency Conventions.
    let conn = Connection::open_in_memory().ok()?;
    conn.query_row(
        "SELECT strftime('%Y-%m-%d', ?1, 'unixepoch')",
        [secs as i64],
        |r| r.get::<_, String>(0),
    )
    .ok()
}

pub fn print_report(report: &BuildReport) {
    println!("\n=== Bảng dựng dữ liệu — nguồn → dòng đọc / dòng bỏ / bản ghi ===");
    println!(
        "{:14} {:>10} {:>10} {:>10} {:>10} {:>10} {:>10}",
        "nguồn", "đọc", "bỏ", "entry", "sense", "example", "citation"
    );
    for s in &report.per_source {
        println!(
            "{:14} {:>10} {:>10} {:>10} {:>10} {:>10} {:>10}",
            s.source_code, s.lines_read, s.lines_skipped, s.entries, s.senses, s.examples, s.citations
        );
        for (reason, count) in &s.skip_reasons {
            println!("    [{count:>8}] {reason}");
        }
    }
    println!("\nchar_idx cặp (ch, entry_id): {}", report.char_idx_pairs);
    println!("journal_mode sau khi đóng:    {}", report.journal_mode);
    println!("SHA-256:                      {}", report.sha256);
    println!(
        "Kích thước:                   {} byte ({:.2} MB thập phân)",
        report.size_bytes,
        report.size_bytes as f64 / 1_000_000.0
    );
}
