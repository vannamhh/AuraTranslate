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
#[derive(Debug)]
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

/// Dựng lớp NỀN — `dict-core.db`. `raw_dir` chứa năm thư mục con (`cvdict/`,
/// `cc_cedict/`, `unihan/`, `viwiktionary/`, `en_wiktionary/`) theo quy ước đã ghi ở
/// `tools/dict-build/README.md`. `out_path` là `.db` đích.
///
/// Dựng vào một tệp TẠM cùng thư mục với `out_path`, chỉ đổi tên sang `out_path` SAU KHI
/// mọi bước (rebuild FTS, ANALYZE/VACUUM, journal_mode=DELETE, kiểm no-wal, băm) đã
/// xong — một lượt build hỏng giữa chừng không còn để lại tệp dở dang TẠI `out_path`
/// (Review Findings Group A; trước đây chỉ phân biệt được qua exit code/stderr). Mọi
/// tệp cũ ở `out_path`/`tmp_path` (cộng `-wal`/`-shm` cạnh chúng) bị xoá trước khi dựng,
/// vì đây LUÔN là một lượt dựng MỚI từ đầu, ⛔ không phải cập nhật tệp cũ.
///
/// Đổi tên từ `run` (Story 1.9) → `run_base` (Story 1.10) khi thêm hai đường dựng lớp
/// gỡ rời dùng chung phần đuôi (`finalize::finish`) — nội dung hàm này ⛔ không đổi.
pub fn run_base(raw_dir: &Path, out_path: &Path) -> Result<BuildReport, Box<dyn std::error::Error>> {
    let tmp_path = finalize::prepare_fresh_output(out_path)?;

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
        insert::insert_meta(&tx, "base", &built_at(&[raw_dir]))?;

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

    let (sha256, size_bytes, journal_mode) = finalize::finish(conn, &tmp_path, out_path)?;

    Ok(BuildReport {
        per_source,
        char_idx_pairs,
        sha256,
        size_bytes,
        journal_mode,
    })
}

/// Dựng MỘT lớp gỡ rời — MỘT nguồn, MỘT tệp `.db`, dùng LẠI đúng lược đồ của lớp nền
/// (§Quyết định #1 của Story 1.10, AC4). 🔴 Hàm này ⛔ **không bao giờ** mở
/// `dict-core.db` (§Bẫy 3) — nó chỉ biết `raw_file_path` của CHÍNH nguồn đang dựng.
///
/// Dùng chung `finalize::prepare_fresh_output` + `finalize::finish` với `run_base` —
/// đây LÀ điều kiện của Task 5 (Bẫy 2: một trong hai đường dựng bỏ sót
/// `journal_mode = DELETE` là bẫy đắt nhất, nhân đôi vì giờ có ba đường dựng).
fn run_detachable_layer<F, I>(
    raw_file_path: &Path,
    out_path: &Path,
    meta: &sources_meta::SourceMeta,
    source_version: &str,
    parse_fn: F,
) -> Result<BuildReport, Box<dyn std::error::Error>>
where
    F: FnOnce(BufReader<File>) -> I,
    I: Iterator<Item = Result<crate::model::RawEntry, crate::model::ParseIssue>>,
{
    let tmp_path = finalize::prepare_fresh_output(out_path)?;

    let mut conn = Connection::open(&tmp_path)?;
    conn.execute_batch("PRAGMA foreign_keys = ON;")?;

    insert::create_schema(&conn)?;

    let mut per_source = Vec::new();
    {
        let tx = conn.transaction()?;
        insert::insert_meta(&tx, meta.code, &built_at(&[raw_file_path]))?;

        let source_id = insert::insert_source(&tx, meta, source_version)?;
        let mut stats = SourceStats::new(meta.code);
        let f = File::open(raw_file_path).map_err(|e| {
            format!(
                "không mở được nguồn thô '{}' — {e}",
                raw_file_path.display()
            )
        })?;
        ingest(&tx, source_id, &mut stats, parse_fn(BufReader::new(f)))?;
        require_nonempty(&stats)?;
        per_source.push(stats);

        tx.commit()?;
    }

    let char_idx_pairs: i64 = conn.query_row("SELECT COUNT(*) FROM char_idx", [], |r| r.get(0))?;

    let (sha256, size_bytes, journal_mode) = finalize::finish(conn, &tmp_path, out_path)?;

    Ok(BuildReport {
        per_source,
        char_idx_pairs,
        sha256,
        size_bytes,
        journal_mode,
    })
}

/// Một dòng của BẢNG PHÂN PHỐI lớp gỡ rời (§Quyết định #2 của Story 1.10): mã lớp,
/// `SourceMeta`, đường dẫn tệp thô TƯƠNG ĐỐI (nối vào `raw_dir` lúc chạy), `source_version`,
/// và hàm parse — đóng gói qua con trỏ hàm KHÔNG bắt biến (`fn(...)`, coerce được từ
/// closure không capture) để nhét được vào MỘT mảng dù chữ ký `parse()` của mỗi module
/// trả về một kiểu iterator cụ thể khác nhau.
///
/// Thêm HVTĐTD/Cổ hán văn ở story nối tiếp = thêm MỘT phần tử vào `DETACHABLE_LAYERS`,
/// ⛔ KHÔNG sửa `run_detachable_layer`/`run_all`/CLI — viết thành `if code == "..." {}`
/// mới là hình dạng "mã riêng cho từng nguồn" mà AC4 cấm, chỉ dịch bẫy đó từ runtime
/// sang phía build.
struct DetachableLayer {
    meta: &'static sources_meta::SourceMeta,
    raw_relative_path: &'static [&'static str],
    source_version: &'static str,
    #[allow(clippy::type_complexity)]
    parse: fn(BufReader<File>) -> Box<dyn Iterator<Item = Result<crate::model::RawEntry, crate::model::ParseIssue>>>,
}

const DETACHABLE_LAYERS: &[DetachableLayer] = &[
    DetachableLayer {
        meta: &sources_meta::THIEU_CHUU,
        raw_relative_path: &["thieu_chuu", "TudienThienChuu.txt"],
        source_version: sources::thieu_chuu::SOURCE_VERSION,
        parse: |r| Box::new(sources::thieu_chuu::parse(r)),
    },
    DetachableLayer {
        meta: &sources_meta::VIETPHRASE,
        raw_relative_path: &["vietphrase", "VietPhrase.txt"],
        source_version: sources::vietphrase::SOURCE_VERSION,
        parse: |r| Box::new(sources::vietphrase::parse(r)),
    },
];

fn raw_path_for(raw_dir: &Path, relative: &[&str]) -> std::path::PathBuf {
    relative.iter().fold(raw_dir.to_path_buf(), |acc, seg| acc.join(seg))
}

/// Tên tệp cố định trong Rust cho MỘT mã lớp — ⛔ không tham số hoá (§Quyết định #3 của
/// Story 1.10): `dict-core.db` cho `"base"`, `dict-<code>.db` cho lớp gỡ rời. `name`
/// trong manifest, tên tệp, và `dict_source.code` LUÔN là cùng một chuỗi.
pub fn output_file_name(layer_code: &str) -> String {
    if layer_code == "base" {
        // dict-build:allow dict-core — tên tệp ĐẦU RA của lớp base, không mở/đọc tệp
        "dict-core.db".to_string()
    } else {
        format!("dict-{layer_code}.db")
    }
}

/// Dựng MỘT lớp gỡ rời theo `code` — tra `DETACHABLE_LAYERS` (bảng, ⛔ không `if`/`match`
/// theo chuỗi từng nguồn). Dùng cho `--layer <code>` đơn lẻ.
pub fn run_detachable_by_code(
    raw_dir: &Path,
    out_dir: &Path,
    code: &str,
) -> Result<(String, BuildReport), Box<dyn std::error::Error>> {
    let layer = DETACHABLE_LAYERS
        .iter()
        .find(|l| l.meta.code == code)
        .ok_or_else(|| format!("mã lớp gỡ rời không xác định: '{code}'"))?;
    // Đối xứng với `run_all` — nếu không, `Connection::open` trả SQLITE_CANTOPEN
    // ("unable to open database file"), một thông điệp ⛔ không hề nhắc tới thư mục thiếu.
    std::fs::create_dir_all(out_dir)?;
    let raw_path = raw_path_for(raw_dir, layer.raw_relative_path);
    let name = output_file_name(layer.meta.code);
    let report = run_detachable_layer(
        &raw_path,
        &out_dir.join(&name),
        layer.meta,
        layer.source_version,
        layer.parse,
    )?;
    Ok((name, report))
}

/// Kết quả một lượt `--layer all`: base + MỌI lớp gỡ rời trong `DETACHABLE_LAYERS`.
#[derive(Debug)]
pub struct AllLayersReport {
    pub base: (String, BuildReport),
    pub detachable: Vec<(String, BuildReport)>,
}

/// 🔴 Kiểm MỌI nguồn thô của lớp gỡ rời TỒN TẠI trước khi xoá một tệp `.db` nào.
///
/// `finalize::prepare_fresh_output` xoá `out_path` **trước** khi mở nguồn thô, nên một
/// lượt `--layer all` thiếu nguồn ở lớp thứ hai sẽ: dựng xong hai tệp mới, **xoá mất**
/// tệp tốt của lớp thứ ba từ lượt trước, rồi thoát 1 để lại một `.tmp` mồ côi. Kết quả
/// là `out/` chứa một bộ KHÔNG đầy đủ và trộn thế hệ mà ⛔ không dấu hiệu nào — trong khi
/// §Quyết định #6 đòi ba tệp thuộc **một** thế hệ dữ liệu. Kiểm trước là cách rẻ nhất
/// biến ca đó thành một lỗi sạch, ⛔ không đụng tệp nào.
fn require_all_raw_sources_present(raw_dir: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let missing: Vec<String> = DETACHABLE_LAYERS
        .iter()
        .map(|l| raw_path_for(raw_dir, l.raw_relative_path))
        .filter(|p| !p.is_file())
        .map(|p| p.display().to_string())
        .collect();
    if !missing.is_empty() {
        return Err(format!(
            "thiếu {} nguồn thô của lớp gỡ rời, ⛔ không tệp .db nào bị đụng tới: {}",
            missing.len(),
            missing.join(", ")
        )
        .into());
    }
    Ok(())
}

/// `--layer all` (mặc định): dựng ĐÚNG base + hai lớp gỡ rời hôm nay — hỏng nếu BẤT KỲ
/// lớp nào thiếu nguồn thô. ⛔ Không có chế độ "bỏ qua lớp thiếu nguồn" (§Bẫy 7).
pub fn run_all(raw_dir: &Path, out_dir: &Path) -> Result<AllLayersReport, Box<dyn std::error::Error>> {
    std::fs::create_dir_all(out_dir)?;
    require_all_raw_sources_present(raw_dir)?;

    let base_name = output_file_name("base");
    let base_report = run_base(raw_dir, &out_dir.join(&base_name))?;

    let mut detachable = Vec::new();
    for layer in DETACHABLE_LAYERS {
        detachable.push(run_detachable_by_code(raw_dir, out_dir, layer.meta.code)?);
    }

    Ok(AllLayersReport {
        base: (base_name, base_report),
        detachable,
    })
}

#[cfg(test)]
mod distribution_table_tests {
    use super::*;

    /// 🔴 `DETACHABLE_LAYERS` (đường dựng THẬT) và `sources_meta::DETACHABLE_ALL` (thứ
    /// Kiểm D của `check-dict-build.mjs` canh, và thứ hai test `sources_meta` khoá) phải
    /// khớp nhau ĐÚNG TỪNG MÃ, đúng thứ tự.
    ///
    /// Không có test này, story nối tiếp thêm HVTĐTD vào `DETACHABLE_ALL` + manifest mà
    /// quên `DETACHABLE_LAYERS` sẽ cho: Kiểm D xanh, cổng manifest xanh, test
    /// `sources_meta` xanh — nhưng `--layer all` **im lặng không dựng** `dict-hvtdtd.db`,
    /// và manifest công bố một tệp không tồn tại ⇒ mọi máy khách 404. Đó đúng là lớp lỗi
    /// "một lớp BỊ RƠI MẤT" mà AC5 nói cổng phải bắt được.
    #[test]
    fn distribution_table_matches_detachable_all_exactly() {
        let table: Vec<&str> = DETACHABLE_LAYERS.iter().map(|l| l.meta.code).collect();
        let declared: Vec<&str> = sources_meta::DETACHABLE_ALL.iter().map(|s| s.code).collect();
        assert_eq!(
            table, declared,
            "DETACHABLE_LAYERS (build.rs) phải khớp DETACHABLE_ALL (sources_meta.rs) từng mã, đúng thứ tự"
        );
    }

    /// `.find()` theo `code` là cách điều phối duy nhất — mã trùng làm một lớp bị dựng
    /// hai lần (đè lên chính nó) còn lớp kia ⛔ không bao giờ chạy, với exit code SUCCESS.
    #[test]
    fn distribution_table_has_no_duplicate_codes() {
        let codes: Vec<&str> = DETACHABLE_LAYERS.iter().map(|l| l.meta.code).collect();
        let distinct: std::collections::HashSet<&str> = codes.iter().copied().collect();
        assert_eq!(codes.len(), distinct.len(), "DETACHABLE_LAYERS có mã lớp trùng: {codes:?}");
    }

    /// Tên tệp đầu ra của MỌI lớp phải rời nhau — một lớp gỡ rời mang `code = "core"`
    /// sẽ ghi đè `dict-core.db` mà `base_and_detachable_code_sets_are_disjoint` ⛔ không
    /// bắt được (nó chỉ so tập `code`, không so tập TÊN TỆP).
    #[test]
    fn every_layer_writes_to_a_distinct_output_file() {
        let mut names: Vec<String> = vec![output_file_name("base")];
        names.extend(DETACHABLE_LAYERS.iter().map(|l| output_file_name(l.meta.code)));
        let distinct: std::collections::HashSet<&String> = names.iter().collect();
        assert_eq!(names.len(), distinct.len(), "tên tệp đầu ra bị trùng: {names:?}");
    }
}

/// `source_version` cho hai nguồn Wiktionary — không có header ngày tháng trong nội
/// dung, nên dùng thời điểm sửa đổi cuối của tệp đã tải (ngày dump, xấp xỉ đúng tinh
/// thần "phiên bản nguồn thô" khi nguồn không tự khai).
/// 🔴 `built_at` TẤT ĐỊNH — dẫn xuất từ chính NGUỒN THÔ, ⛔ không từ đồng hồ hệ thống.
///
/// Cùng một cây nguồn thô ⇒ cùng một `built_at` ⇒ cùng một tệp `.db` byte-for-byte ⇒
/// cùng một SHA-256. Đây là điều kiện để mọi giá trị `sha256` trong `dict-manifest.toml`
/// còn đúng sau một lượt `cargo run` lại — trước đây `strftime('now')` với độ phân giải
/// mili-giây làm hai lượt build liên tiếp từ CÙNG một cây fixture ra sáu hash khác nhau.
///
/// Thứ tự ưu tiên:
/// 1. `SOURCE_DATE_EPOCH` (quy ước reproducible-builds) — cho phép ghim cứng khi phát hành.
/// 2. `mtime` MỚI NHẤT trong số các nguồn thô đã đọc — một thuộc tính của ĐẦU VÀO.
/// 3. Epoch 0, kèm cảnh báo — chỉ xảy ra khi ⛔ không đọc được metadata nào.
fn built_at(inputs: &[&Path]) -> String {
    let secs = std::env::var("SOURCE_DATE_EPOCH")
        .ok()
        .and_then(|v| v.trim().parse::<i64>().ok())
        .or_else(|| newest_mtime_secs(inputs))
        .unwrap_or_else(|| {
            eprintln!(
                "dict-build: ⚠️  không đọc được mtime của nguồn thô và ⛔ không có SOURCE_DATE_EPOCH — dict_meta('built_at') dùng epoch 0"
            );
            0
        });
    iso8601_utc(secs).unwrap_or_else(|| "1970-01-01T00:00:00Z".to_string())
}

/// `mtime` lớn nhất tìm được dưới `inputs` (đệ quy cho thư mục). `None` khi ⛔ không đọc
/// được gì.
fn newest_mtime_secs(inputs: &[&Path]) -> Option<i64> {
    fn walk(path: &Path, best: &mut Option<i64>) {
        if path.is_dir() {
            if let Ok(entries) = std::fs::read_dir(path) {
                let mut children: Vec<std::path::PathBuf> =
                    entries.filter_map(|e| e.ok()).map(|e| e.path()).collect();
                // Sắp xếp để lượt duyệt ⛔ không phụ thuộc thứ tự trả về của hệ tệp.
                children.sort();
                for child in children {
                    walk(&child, best);
                }
            }
            return;
        }
        let secs = std::fs::metadata(path)
            .ok()
            .and_then(|m| m.modified().ok())
            .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
            .map(|d| d.as_secs() as i64);
        if let Some(s) = secs {
            *best = Some(best.map_or(s, |b: i64| b.max(s)));
        }
    }
    let mut best = None;
    for p in inputs {
        walk(p, &mut best);
    }
    best
}

/// Epoch giây → ISO-8601 UTC. Dùng chính SQLite để đổi, ⛔ không kéo `chrono`/`time`
/// (§Quyết định #6 của Story 1.9: 0 crate mới).
fn iso8601_utc(secs: i64) -> Option<String> {
    let conn = Connection::open_in_memory().ok()?;
    conn.query_row(
        "SELECT strftime('%Y-%m-%dT%H:%M:%SZ', ?1, 'unixepoch')",
        [secs],
        |r| r.get::<_, String>(0),
    )
    .ok()
}

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
