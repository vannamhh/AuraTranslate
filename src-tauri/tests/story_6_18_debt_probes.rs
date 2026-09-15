//! Story 6.18, task 6 — bốn món nợ "chuỗi nhập" của `deferred-work.md`, mỗi món chủ **Story
//! 6.18**: `normalize::normalize` chạy hai lượt `.replace()` không điều kiện; đỉnh RSS thật
//! của TOÀN chuỗi bảy bước trên một Chương ở trần `MAX_IMPORT_BYTES`; `ChapterPattern::match_starts`
//! biên dịch lại regex mỗi lượt gọi. Ba ca ĐẦU (`normalize`/`match_starts`) là `perf_probe_*`
//! THƯỜNG (không `#[ignore]`, cùng khuôn `cleanup_contract.rs::perf_probe_chapter_split_preview_on_two_thousand_chapters`)
//! — nhanh, chạy trong `cargo test --locked` mặc định. Ca "đỉnh RSS toàn chuỗi" nặng (dựng
//! một tệp 100 MB thật trên đĩa cộng một lượt ghi SQLite thật) nên mang `#[ignore]`, cùng lý
//! lẽ `story_6_18_library.rs`.
//!
//! Không con số nào ở đây là một NFR — đây là bằng chứng ĐO cho một món nợ đang mở, không
//! phải một cổng nghiệm thu. `AGENTS.md`: "Never mark something passed by inference."

use std::path::Path;
use std::time::{Duration, Instant};

use auratranslate_lib::commands::project::create_work_from_file;
use auratranslate_lib::core::segment::chapterpattern::ChapterPattern;
use auratranslate_lib::core::segment::normalize::normalize;

fn temp_dir(tag: &str) -> std::path::PathBuf {
    let dir = std::env::temp_dir().join(format!("auratranslate-6-18-debt-probes-{tag}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap_or_else(|e| panic!("tao {}: {e}", dir.display()));
    dir
}

fn cleanup(dir: &Path) {
    let _ = std::fs::remove_dir_all(dir);
}

/// Văn bản KHÔNG mang một ký tự `\r` nào — hình dạng ĐA SỐ (§Never đoạn evidence của món nợ:
/// "MỌI Chương từ một nguồn KHÔNG PHẢI Windows"), câu tiếng Việt + tiếng Trung xen kẽ để giống
/// văn bản thật, xuống dòng `\n` trần mỗi câu, đoạn cách nhau một dòng trống.
fn build_crlf_free_text(target_bytes: usize) -> String {
    let mut out = String::with_capacity(target_bytes + 256);
    let mut i = 0usize;
    while out.len() < target_bytes {
        out.push_str(&format!(
            "Đây là câu thứ {i} của một đoạn văn dài, mang cả tiếng Việt lẫn 一些中文字符 để giống \
             một Chương thật.\n"
        ));
        i += 1;
        if i % 8 == 0 {
            out.push('\n'); // ranh giới đoạn
        }
    }
    out
}

// ═════════════════════════════════════════════════════════════════════════════════
// `normalize::normalize` — hai lượt `.replace()` không điều kiện (deferred-work.md, cụm
// "Deferred from: 6-4…", Chủ Story 6.18)
// ═════════════════════════════════════════════════════════════════════════════════
#[test]
fn perf_probe_normalize_two_pass_replace_cost_on_a_ten_megabyte_crlf_free_chapter() {
    const TARGET_BYTES: usize = 10 * 1024 * 1024; // 10 MB -- đủ để đo, đủ nhanh cho cong mac dinh
    let text = build_crlf_free_text(TARGET_BYTES);
    let source_bytes = text.len();
    assert!(!text.contains('\r'), "tien de: van ban khong duoc mang \\r nao");

    // Chi phi CÔ LẬP của hai lượt `.replace()` (đúng dòng `normalize.rs:86`, chép lại y
    // nguyên ở đây để đo RIÊNG bước đó, không lẫn với split/join của toàn `normalize`).
    let t0 = Instant::now();
    let unified = text.replace("\r\n", "\n").replace('\r', "\n");
    let two_pass_replace = t0.elapsed();
    assert_eq!(unified.len(), text.len(), "0 khop thi do dai phai giu nguyen");
    drop(unified);

    // Chi phi của MỘT lượt gác `contains('\r')` trước hai lượt `.replace()` (đề xuất của
    // chính món nợ) -- nếu gác RẺ HƠN nhiều so với hai lượt replace, gác trước là một tối ưu
    // có that; nếu suýt soát, gác không đáng (thêm một lượt quét cho lợi ích gần 0).
    let t1 = Instant::now();
    let has_cr = text.contains('\r');
    let guard_check = t1.elapsed();
    assert!(!has_cr);

    // Toàn `normalize()` thật (bước 4 sản phẩm nguyên vẹn) trên CÙNG văn bản, để so sánh
    // TỈ TRỌNG: hai lượt replace chiếm bao nhiêu phần trăm của toàn bước 4.
    let t2 = Instant::now();
    let result = normalize(&text, "vi");
    let whole_normalize = t2.elapsed();
    assert_eq!(result.joined_lines, 0, "van ban moi dong da tron cau -- khong dong nao bi noi oan");

    let ratio_pct = if whole_normalize.as_nanos() > 0 {
        100.0 * two_pass_replace.as_secs_f64() / whole_normalize.as_secs_f64()
    } else {
        0.0
    };
    let guard_saves_pct = if two_pass_replace.as_nanos() > 0 {
        100.0 * (1.0 - guard_check.as_secs_f64() / two_pass_replace.as_secs_f64())
    } else {
        0.0
    };

    eprintln!(
        "[perf_probe_normalize_two_pass] nguon={source_bytes}B (0 \\r) \
         two_pass_replace={two_pass_replace:?} guard_check_contains_cr={guard_check:?} \
         whole_normalize={whole_normalize:?} replace_ti_trong_trong_normalize={ratio_pct:.1}% \
         guard_re_hon_bao_nhieu={guard_saves_pct:.1}% \
         verdict=guard_truoc_hai_lan_replace_{}",
        if guard_check < two_pass_replace { "co_loi" } else { "khong_dang" }
    );
}

// ═════════════════════════════════════════════════════════════════════════════════
// `ChapterPattern::match_starts` — regex biên dịch lại mỗi lượt gọi, không cache
// (deferred-work.md, cụm "Deferred from: 6-10a…", Chủ Story 6.18)
// ═════════════════════════════════════════════════════════════════════════════════
#[test]
fn perf_probe_match_starts_recompiles_the_regex_every_call() {
    const CHAPTER_COUNT: usize = 2_000; // cùng cỡ với perf_probe_chapter_split_preview_on_two_thousand_chapters
    const CALLS_PER_PREVIEW_RELOAD: usize = 7; // doc-comment mon no: "toi da 6-7 lan" moi luot tai lai xem truoc

    let mut text = String::new();
    for i in 0..CHAPTER_COUNT {
        text.push_str(&format!("Chuong {i}: Tieu De\n\nnoi dung ngan cua chuong nay.\n\n"));
    }

    let pattern = ChapterPattern::regex(r"^Chuong \d+:.*$");

    // MỘT lượt `match_starts` (chi phí biên dịch regex + quét toàn văn MỘT lần).
    let t0 = Instant::now();
    let starts_once = pattern.match_starts(&text).expect("mau hop le");
    let one_call = t0.elapsed();
    assert_eq!(starts_once.len(), CHAPTER_COUNT);

    // BẢY lượt liên tiếp trên CÙNG `pattern`/CÙNG văn bản -- hình dạng thật một lượt tải lại
    // xem trước (năm ứng viên bảng mã + đường tự khai + lượt xác nhận, doc-comment món nợ).
    let t1 = Instant::now();
    for _ in 0..CALLS_PER_PREVIEW_RELOAD {
        let starts = pattern.match_starts(&text).expect("mau hop le");
        assert_eq!(starts.len(), CHAPTER_COUNT);
    }
    let seven_calls = t1.elapsed();

    // Đối chứng: một Regex biên dịch MỘT LẦN rồi quét bảy lần (đường "nếu có cache") --
    // dùng thẳng `regex::Regex` qua `chapterpattern::compile`, không qua `match_starts`, để
    // cô lập đúng phần "biên dịch lại" mà món nợ nói tới.
    let compiled = auratranslate_lib::core::segment::chapterpattern::compile(r"^Chuong \d+:.*$")
        .expect("bien dich mot lan");
    let t2 = Instant::now();
    for _ in 0..CALLS_PER_PREVIEW_RELOAD {
        let count = compiled.find_iter(&text).filter(|m| !m.is_empty()).count();
        assert_eq!(count, CHAPTER_COUNT);
    }
    let seven_scans_cached = t2.elapsed();

    let recompile_overhead = seven_calls.saturating_sub(seven_scans_cached);
    let recompile_pct = if seven_calls.as_nanos() > 0 {
        100.0 * recompile_overhead.as_secs_f64() / seven_calls.as_secs_f64()
    } else {
        0.0
    };

    eprintln!(
        "[perf_probe_match_starts_recompile] {CHAPTER_COUNT} Chuong, {CALLS_PER_PREVIEW_RELOAD} lan goi/luot tai. \
         mot_lan_goi={one_call:?} bay_lan_goi_that={seven_calls:?} bay_lan_QUET_neu_da_cache_regex={seven_scans_cached:?} \
         chi_phi_bien_dich_lai_uoc_tinh={recompile_overhead:?} ({recompile_pct:.1}% cua bay lan goi that) \
         verdict=chi_phi_bien_dich_lai_{}",
        if recompile_pct > 10.0 { "dang_ke" } else { "nho" }
    );
}

// ═════════════════════════════════════════════════════════════════════════════════
// Đỉnh RSS thật của TOÀN chuỗi bảy bước trên một Chương ở trần `MAX_IMPORT_BYTES` (100 MB)
// (hai món nợ gộp: "chưa ai đo đỉnh RSS thật" của Story 1.15 cũ + "bước 4 thêm ít nhất hai
// lượt quét mới" — cả hai Chủ Story 6.18, cùng cụm "Deferred from: 6-4…").
//
// Chạy tay:
//   cargo test --locked --release --test story_6_18_debt_probes \
//     -- --ignored --nocapture peak_rss
// ═════════════════════════════════════════════════════════════════════════════════
#[test]
#[ignore = "nang: dung mot tep 100 MB that tren dia cong mot luot ghi SQLite that -- khong phai mot cong mac dinh"]
fn peak_rss_of_the_whole_seven_step_import_chain_on_a_max_import_bytes_chapter() {
    use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
    use std::sync::Arc;

    // MAX_IMPORT_BYTES thật (100 MB) trừ một biên an toàn nhỏ -- vòng ghi theo dòng có thể
    // vượt quá đích một chút ở dòng CUỐI, và `import_file` từ chối NGHIÊM > MAX_IMPORT_BYTES.
    const TARGET_BYTES: u64 = 100 * 1024 * 1024 - 4096;

    let dir = temp_dir("peak-rss");
    let source_path = dir.join("mot-chuong-100mb.txt");

    // Dựng tệp TRÊN ĐĨA (không giữ 100 MB thứ hai trong bộ nhớ tiến trình lúc dựng) --
    // ghi theo khối để tiền RSS của CHÍNH bước dựng fixture không lẫn vào số đo.
    {
        use std::io::Write;
        let mut f = std::fs::File::create(&source_path).unwrap_or_else(|e| panic!("tao tep: {e}"));
        let mut written = 0u64;
        let mut i = 0usize;
        while written < TARGET_BYTES {
            let line = format!(
                "Đây là câu thứ {i} của Chương duy nhất, mang cả tiếng Việt lẫn 一些中文字符.\n"
            );
            f.write_all(line.as_bytes()).unwrap_or_else(|e| panic!("ghi tep: {e}"));
            written += line.len() as u64;
            i += 1;
        }
        f.flush().unwrap_or_else(|e| panic!("flush: {e}"));
    }
    let actual_size = std::fs::metadata(&source_path).expect("doc kich thuoc tep vua tao").len();
    eprintln!("[peak_rss] tep nguon {actual_size} byte tai {}", source_path.display());

    // Lấy mẫu RSS NỀN (`ps -o rss=`, KB) mỗi 20 ms trong khi luồng chính chạy chuỗi nhập thật
    // -- cùng công cụ `ps` mà `5-14-ban-do/run.sh::measure_memory_session` đã dùng cho tập
    // WebKit, không một crate mới (§Never spec 6.18: "no new crate").
    let pid = std::process::id();
    let peak_kb = Arc::new(AtomicU64::new(0));
    let stop = Arc::new(AtomicBool::new(false));
    let sampler = {
        let peak_kb = Arc::clone(&peak_kb);
        let stop = Arc::clone(&stop);
        std::thread::spawn(move || {
            while !stop.load(Ordering::Relaxed) {
                if let Ok(out) = std::process::Command::new("ps")
                    .args(["-o", "rss=", "-p", &pid.to_string()])
                    .output()
                {
                    if let Ok(text) = String::from_utf8(out.stdout) {
                        if let Ok(kb) = text.trim().parse::<u64>() {
                            peak_kb.fetch_max(kb, Ordering::Relaxed);
                        }
                    }
                }
                std::thread::sleep(Duration::from_millis(20));
            }
        })
    };

    let documents_root = dir.join("library");
    std::fs::create_dir_all(&documents_root).expect("tao library root");

    let t0 = Instant::now();
    let opened = create_work_from_file(&documents_root, "Peak RSS Probe", "vi", "", &source_path)
        .unwrap_or_else(|e| panic!("create_work_from_file that bai: {e:?}"));
    let elapsed = t0.elapsed();

    let chapter_count: i64 = opened
        .store
        .read(|conn| conn.query_row("SELECT COUNT(*) FROM chapter", [], |row| row.get(0)))
        .expect("dem Chuong vua tao");
    let segment_count: i64 = opened
        .store
        .read(|conn| conn.query_row("SELECT COUNT(*) FROM segment", [], |row| row.get(0)))
        .expect("dem segment vua tao");
    drop(opened);

    stop.store(true, Ordering::Relaxed);
    sampler.join().expect("join luong lay mau");

    let peak_kb = peak_kb.load(Ordering::Relaxed);
    let peak_mb = peak_kb as f64 / 1024.0;
    let ratio_to_source = peak_mb / (actual_size as f64 / (1024.0 * 1024.0));

    eprintln!(
        "[peak_rss_of_the_whole_seven_step_import_chain] nguon={actual_size}B (~{:.1} MB) \
         chapters={chapter_count} segments={segment_count} thoi_gian={elapsed:?} \
         dinh_RSS_tien_trinh={peak_kb} KB (~{peak_mb:.1} MB) ti_le_dinh_RSS/nguon={ratio_to_source:.2}x \
         mau_moi_20ms_qua_`ps`_khong_phai_footprint -- can duoi ude UOC LUONG, khong phai tran chinh xac",
        actual_size as f64 / (1024.0 * 1024.0)
    );

    assert!(chapter_count >= 1, "phai tao duoc it nhat mot Chuong");
    assert!(segment_count > 0, "phai tach duoc segment");

    cleanup(&dir);
}
