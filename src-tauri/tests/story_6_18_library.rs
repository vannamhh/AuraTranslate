//! Story 6.18, task 3 — dựng thư viện 5.000 Chương THẬT qua ĐÚNG đường sản phẩm, để re-đo
//! NFR3/NFR4/NFR5 (A6/A7/A8, Q4) không còn dựa trên fixture SQL thô của Story 5.14 (Epic 5
//! retro F5: một con số như vậy "không phải một con số về sản phẩm").
//!
//! ─────────────────────────────────────────────────────────────────────────────
//! `#[ignore]` LÀ MỘT QUYẾT ĐỊNH, KHÔNG PHẢI MỘT LƯỢT QUÊN
//! ─────────────────────────────────────────────────────────────────────────────
//! Dựng 50 Tác phẩm × 100 Chương × 10 segment qua `confirm_bilingual_import` (bộ dựng bảng
//! + tách Chương + tách câu THẬT của sản phẩm) tốn nhiều giây hơn một ca thường — không phải
//! một cổng `cargo test --locked` mặc định chạy. Chạy tay:
//!
//!     cargo test --profile bench-release --locked --manifest-path src-tauri/Cargo.toml \
//!       --test story_6_18_library -- --ignored --nocapture
//!
//! ─────────────────────────────────────────────────────────────────────────────
//! CHỈ đường sản phẩm — không một câu SQL ghi nào trong tệp này
//! ─────────────────────────────────────────────────────────────────────────────
//! Toàn bộ Chương/segment được tạo qua `preview_bilingual_import` → `confirm_bilingual_import`
//! (→ `create_work`), và mọi lượt đổi trạng thái Chương đi qua `lifecycle::set_chapter_status`
//! (hàm THUẦN, một `UPDATE` mỗi lần, không `reindex` phụ) — spec 6.18 §Always: "The library
//! is created only by product functions... Chapter status only through
//! `lifecycle::set_chapter_status`". Mọi câu SQL trong tệp này (`conn.prepare`/`query_row`) là
//! ĐỌC, phục vụ đúng một việc: đọc lại để KIỂM quần thể trước khi bất kỳ ai tin số đo dựng
//! trên nó — AC1 grep đúng vế này (0 `INSERT`/`UPDATE` nhắm `project.db`/`library-index.db`
//! trong toàn bộ đường dựng thư viện).
//!
//! ─────────────────────────────────────────────────────────────────────────────
//! Từ vựng NFR3 — CÙNG hình dạng với fixture SQL thô của Story 5.14
//! ─────────────────────────────────────────────────────────────────────────────
//! Mỗi segment mang `天下大势{gc}分久必合{s}, the quick brown fox number {gc}-{s}` (nguồn) và
//! `má của tôi rất hiền ở câu {gc}-{s}, thiên hạ đại thế phân cửu tất hợp` (đích), `gc` là chỉ
//! số Chương TOÀN THƯ VIỆN (0..5000, không phải chỉ số trong một Tác phẩm) và `s` là chỉ số
//! segment trong Chương (0..10) — HỆT bảng `NFR3_CASE` của
//! `library_index_contract.rs::bench_p95_of_a_library_search_over_five_thousand_chapters`, để
//! bàn đo NFR3 mới (đọc thư viện này) tái dùng nguyên năm ca đó, chỉ đổi nguồn dữ liệu.

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Instant;

use auratranslate_lib::commands::lifecycle::set_chapter_status;
use auratranslate_lib::commands::project::{
    OpenWork, PendingImportSourceState, confirm_bilingual_import, stash_pending_import_source,
};
use auratranslate_lib::core::library::indexer::{Indexer, WorkQuery};
use auratranslate_lib::core::segment::chapterpattern::ChapterPattern;
use auratranslate_lib::core::segment::import::import_bilingual_file;
use auratranslate_lib::core::store::{Store, StoreSpec};

const WORKS: usize = 50;
const CHAPTERS_PER_WORK: usize = 100;
const SEGMENTS_PER_CHAPTER: usize = 10;
const TOTAL_CHAPTERS: usize = WORKS * CHAPTERS_PER_WORK;
const TOTAL_SEGMENTS: usize = TOTAL_CHAPTERS * SEGMENTS_PER_CHAPTER;

/// Marker scratch dùng CHUNG với `nfr-bench` (Story 6.18 Quyết định 5, đổi tên từ
/// `story-5-14-bench`) — một quy ước HOME nháp duy nhất cho toàn chuỗi đo NFR, không một
/// tiền tố thứ hai phải nhớ riêng.
const SCRATCH_MARKER: &str = "auratranslate-nfr-bench-";

fn scratch_documents_root() -> PathBuf {
    // Cho phép bàn đo release trỏ builder này thẳng vào HOME nháp mà `nfr-bench` sẽ dùng
    // (né một lượt copy cây thư mục 5.000 Chương) — cùng khuôn
    // `AURA_5_14_EXPORT_LIBRARY_ROOT` của `segment_contract.rs`. Hàng rào tên thư mục ngăn
    // một biến môi trường gõ nhầm ghi đè dữ liệu thật.
    if let Some(export_root) = std::env::var_os("AURA_6_18_EXPORT_LIBRARY_ROOT").map(PathBuf::from) {
        let raw = export_root.to_string_lossy();
        assert!(
            raw.contains(SCRATCH_MARKER),
            "đích export phải là HOME nháp có marker {SCRATCH_MARKER}, nhận {raw}"
        );
        if export_root.exists() {
            assert!(
                fs::read_dir(&export_root).expect("đọc đích export").next().is_none(),
                "đích export phải rỗng: {}",
                export_root.display()
            );
        } else {
            fs::create_dir_all(&export_root).expect("tạo đích export");
        }
        return export_root;
    }
    let dir = std::env::temp_dir().join(format!(
        "{SCRATCH_MARKER}story-6-18-{}",
        std::process::id()
    ));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap_or_else(|e| panic!("tạo {}: {e}", dir.display()));
    dir
}

fn cleanup(dir: &Path) {
    // Chỉ tự xoá khi KHÔNG chạy dưới một đích export tường minh — người gọi `run.sh` (Story
    // 6.18 task 5) cần cây thư mục sống sót sau khi tiến trình test thoát.
    if std::env::var_os("AURA_6_18_EXPORT_LIBRARY_ROOT").is_none() {
        let _ = fs::remove_dir_all(dir);
    }
}

/// Dựng đúng một hàng CSV, bọc nháy kép (RFC4180) vì cả hai cột đều mang dấu phẩy trong
/// thân câu — thiếu bọc nháy thì trình phân bảng cắt tách thành nhiều cột hơn, và cột đích
/// thật bị trôi sang một chỉ số cột khác (đo tay lúc soạn tệp này, xem `zz_scratch_618_probe`
/// đã xoá khỏi cây — dạng lỗi CHÍNH XÁC này).
fn push_row(csv: &mut String, source: &str, target: &str) {
    csv.push('"');
    csv.push_str(&source.replace('"', "\"\""));
    csv.push_str("\",\"");
    csv.push_str(&target.replace('"', "\"\""));
    csv.push_str("\"\n");
}

/// Toàn bộ nội dung `.csv` của MỘT Tác phẩm: `CHAPTERS_PER_WORK` Chương, mỗi Chương
/// `SEGMENTS_PER_CHAPTER` hàng. Hàng `s == 0` mang thêm tiền tố `HOI {c}` (mẫu phân tách
/// Chương literal, chỉ cần DUY NHẤT trong phạm vi MỘT Tác phẩm — 100 giá trị `c` phân biệt
/// là đủ) — hàng đó VẪN là một segment thật (xem doc-comment đầu tệp), không một hàng tiêu
/// đề bị loại riêng: trình song ngữ không có khái niệm "hàng chỉ làm tiêu đề".
fn bilingual_csv_for_work(work_idx: usize) -> String {
    let mut csv = String::new();
    for c in 0..CHAPTERS_PER_WORK {
        let global_chapter = work_idx * CHAPTERS_PER_WORK + c;
        for s in 0..SEGMENTS_PER_CHAPTER {
            let prefix = if s == 0 { format!("HOI {c} ") } else { String::new() };
            let source = format!(
                "{prefix}天下大势{global_chapter}分久必合{s}, the quick brown fox number {global_chapter}-{s}"
            );
            let target = format!(
                "má của tôi rất hiền ở câu {global_chapter}-{s}, thiên hạ đại thế phân cửu tất hợp"
            );
            push_row(&mut csv, &source, &target);
        }
    }
    csv
}

fn chapter_ids_in_order(store: &Store) -> Vec<i64> {
    store
        .read(|conn| {
            let mut stmt = conn.prepare("SELECT id FROM chapter ORDER BY ord")?;
            let rows = stmt.query_map([], |row| row.get::<_, i64>(0))?;
            rows.collect::<auratranslate_lib::core::store::SqlResult<Vec<_>>>()
        })
        .expect("đọc danh sách Chương vừa dựng")
}

/// Chuyển TOÀN BỘ Chương của một Tác phẩm đang mở sang `status`, qua đúng hàm THUẦN
/// `lifecycle::set_chapter_status` — một lời gọi một Chương, không một câu `UPDATE` gộp nào
/// viết tay ở đây (§Always spec 6.18).
fn set_all_chapters(opened: &mut OpenWork, status: &str) {
    let ids = chapter_ids_in_order(&opened.store);
    assert_eq!(ids.len(), CHAPTERS_PER_WORK, "Tác phẩm phải mang đúng {CHAPTERS_PER_WORK} Chương trước khi đổi trạng thái");
    for chapter_id in ids {
        set_chapter_status(Some(opened), chapter_id, status).unwrap_or_else(|e| {
            panic!("set_chapter_status({chapter_id}, {status:?}) thất bại: {e:?}")
        });
    }
}

struct Population {
    works: usize,
    chapters: i64,
    live_segments: i64,
    done_chapters: i64,
    not_started_chapters: i64,
}

/// Đọc lại quần thể từ CHÍNH `documents_root` trên đĩa — mỗi `.atproj` mở READ-ONLY qua
/// `Store::open`, KHÔNG qua `Indexer` (một phép đọc độc lập với chỉ mục dẫn xuất, để một lỗi
/// trong `Indexer::rebuild` không thể tự che một quần thể sai bằng chính con số nó tính ra).
fn read_back_population(documents_root: &Path) -> Population {
    let mut atproj_dirs: Vec<PathBuf> = fs::read_dir(documents_root)
        .unwrap_or_else(|e| panic!("đọc {}: {e}", documents_root.display()))
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.path())
        .filter(|path| path.is_dir() && path.extension().and_then(|e| e.to_str()) == Some("atproj"))
        .collect();
    atproj_dirs.sort();

    let mut chapters = 0i64;
    let mut live_segments = 0i64;
    let mut done_chapters = 0i64;
    let mut not_started_chapters = 0i64;
    for dir in &atproj_dirs {
        let store = Store::open(StoreSpec::project(dir.join("project.db")))
            .unwrap_or_else(|e| panic!("mở {}: {e}", dir.display()));
        let (c, live, done, not_started): (i64, i64, i64, i64) = store
            .read(|conn| {
                Ok((
                    conn.query_row("SELECT COUNT(*) FROM chapter", [], |row| row.get(0))?,
                    conn.query_row(
                        "SELECT COUNT(*) FROM segment WHERE retired_at IS NULL",
                        [],
                        |row| row.get(0),
                    )?,
                    conn.query_row(
                        "SELECT COUNT(*) FROM chapter WHERE status = 'done'",
                        [],
                        |row| row.get(0),
                    )?,
                    conn.query_row(
                        "SELECT COUNT(*) FROM chapter WHERE status = 'not_started'",
                        [],
                        |row| row.get(0),
                    )?,
                ))
            })
            .unwrap_or_else(|e| panic!("đếm {}: {e}", dir.display()));
        chapters += c;
        live_segments += live;
        done_chapters += done;
        not_started_chapters += not_started;
    }

    Population {
        works: atproj_dirs.len(),
        chapters,
        live_segments,
        done_chapters,
        not_started_chapters,
    }
}

fn total_bytes(root: &Path) -> u64 {
    fn walk(dir: &Path, total: &mut u64) {
        let Ok(entries) = fs::read_dir(dir) else { return };
        for entry in entries.filter_map(|e| e.ok()) {
            let path = entry.path();
            if path.is_dir() {
                walk(&path, total);
            } else if let Ok(meta) = entry.metadata() {
                *total += meta.len();
            }
        }
    }
    let mut total = 0u64;
    walk(root, &mut total);
    total
}

/// Đỉnh RSS tiến trình trong lúc build — cùng công cụ `ps -o rss=` (KB) mỗi 20 ms mà
/// `story_6_18_debt_probes.rs::peak_rss_of_the_whole_seven_step_import_chain_on_a_max_import_bytes_chapter`
/// đã dùng (§Never spec 6.18: "no new crate"), lấy `fetch_max` trên toàn thời lượng build.
/// Matrix row "Build library" (§I/O Matrix spec 6.18): "build time + peak RSS recorded" — chưa
/// ai đo trước bản vá này (audit 2026-09-14 P3).
fn spawn_rss_sampler() -> (Arc<AtomicU64>, Arc<AtomicBool>, std::thread::JoinHandle<()>) {
    let pid = std::process::id();
    let peak_kb = Arc::new(AtomicU64::new(0));
    let stop = Arc::new(AtomicBool::new(false));
    let handle = {
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
                std::thread::sleep(std::time::Duration::from_millis(20));
            }
        })
    };
    (peak_kb, stop, handle)
}

#[test]
#[ignore = "ban do dung tay Story 6.18: dung thu vien 5.000 Chuong that, khong phai mot cong"]
fn builds_the_6_18_library_through_product_import_and_lifecycle_code() {
    let documents_root = scratch_documents_root();
    let global_store = Store::open(StoreSpec::global(documents_root.join("..").join("global.db")))
        .unwrap_or_else(|e| panic!("mở global.db: {e}"));
    let index_path = documents_root.join("..").join("library-index.db");
    let indexer = Indexer::open(index_path.clone()).expect("mở chỉ mục");

    let state: PendingImportSourceState = Mutex::new(None);

    // Matrix row "Build library" (§I/O Matrix): "Read-back equals declaration; build time +
    // peak RSS recorded". Bọc đúng đoạn "dựng" (import 50 Tác phẩm + rebuild full dẫn tới
    // quần thể `full` đã kiểm) — KHÔNG bọc lượt khám phá frontier/phục hồi phía dưới, vì đó là
    // một mối quan tâm khác (chi phí chuyển trạng thái, đã có `transition-raw.tsv` riêng của
    // task 5) chứ không phải chi phí DỰNG thư viện.
    let build_t0 = Instant::now();
    let (build_peak_rss_kb, build_rss_stop, build_rss_sampler) = spawn_rss_sampler();

    println!("STORY_6_18_BUILD\tphase=import\tworks={WORKS}");
    for work_idx in 0..WORKS {
        let csv_text = bilingual_csv_for_work(work_idx);
        let source_path = documents_root.join(format!(".src-{work_idx:02}.csv"));
        fs::write(&source_path, csv_text.as_bytes())
            .unwrap_or_else(|e| panic!("ghi {}: {e}", source_path.display()));

        let shape = import_bilingual_file(&source_path).expect("đọc tệp nguồn hợp lệ");
        stash_pending_import_source(&state, shape, None);

        let work_name = format!("NFR Story 6.18 Work {work_idx:02}");
        let mut opened = confirm_bilingual_import(
            &documents_root,
            &state,
            &work_name,
            "zh",
            "",
            "UTF-8",
            Vec::new(),
            Some(ChapterPattern::literal("HOI")),
            0,
            1,
            false,
            Vec::new(),
        )
        .unwrap_or_else(|e| panic!("confirm_bilingual_import({work_name}) thất bại: {e:?}"));

        // Nguồn CSV tạm chỉ phục vụ lượt đọc của `import_bilingual_file` — không phải một
        // phần của thư viện đã dựng; xoá ngay để `documents_root` chỉ còn đúng các
        // `.atproj`.
        let _ = fs::remove_file(&source_path);

        let ids = chapter_ids_in_order(&opened.store);
        assert_eq!(
            ids.len(),
            CHAPTERS_PER_WORK,
            "Tác phẩm {work_name} phải mang đúng {CHAPTERS_PER_WORK} Chương từ mẫu phân tách"
        );

        // Cả 100 Chương của Tác phẩm này sang `done` — trạng thái QUẦN THỂ CHUẨN "full"
        // (spec 6.18 Quyết định 3). Mỗi lời gọi là MỘT `UPDATE` qua hàm thuần, không gộp.
        set_all_chapters(&mut opened, "done");

        drop(opened);
    }

    println!("STORY_6_18_BUILD\tphase=rebuild_full");
    indexer.rebuild(&documents_root, Some(&global_store)).expect("rebuild full");

    let full = read_back_population(&documents_root);
    assert_eq!(full.works, WORKS, "quần thể full: sai số Tác phẩm");
    assert_eq!(full.chapters, TOTAL_CHAPTERS as i64, "quần thể full: sai số Chương");
    assert_eq!(full.live_segments, TOTAL_SEGMENTS as i64, "quần thể full: sai số segment sống");
    assert_eq!(full.done_chapters, TOTAL_CHAPTERS as i64, "quần thể full: không phải mọi Chương đều done");
    assert_eq!(full.not_started_chapters, 0, "quần thể full: vẫn còn Chương not_started");

    let full_bytes = total_bytes(&documents_root);
    println!(
        "STORY_6_18_POPULATION\tstate=full\tworks={}\tchapters={}\tsegments={}\tdone_chapters={}\tbytes={}\tverdict=matches_declaration",
        full.works, full.chapters, full.live_segments, full.done_chapters, full_bytes
    );

    // Đóng cửa sổ đo NGAY sau khi quần thể `full` vừa được xác nhận khớp khai báo — thời
    // điểm sớm nhất "đã dựng xong, đã kiểm xong" là đúng, không lẫn thêm lượt frontier/phục
    // hồi phía dưới.
    build_rss_stop.store(true, Ordering::Relaxed);
    build_rss_sampler.join().expect("join luong lay mau RSS");
    let build_wall = build_t0.elapsed();
    let build_wall_ms = build_wall.as_millis();
    let build_peak_kb = build_peak_rss_kb.load(Ordering::Relaxed);
    let build_peak_mb = build_peak_kb as f64 / 1024.0;
    println!(
        "STORY_6_18_BUILD_STATS\tbuild_wall_ms={build_wall_ms}\tpeak_rss_kb={build_peak_kb}\tpeak_rss_mb={build_peak_mb:.1}\tmau_moi_20ms_qua_ps_khong_phai_footprint"
    );

    // ─────────────────────────────────────────────────────────────────────────
    // "Cả hai trạng thái qua `set_chapter_status`" — spec 6.18 task 3.
    // ─────────────────────────────────────────────────────────────────────────
    // Đưa ĐÚNG Tác phẩm đầu tiên (Work 00) về `not_started` toàn bộ — hình dạng "frontier"
    // (Chương ĐẦU của Tác phẩm đang mở chưa `done`, spec 6.18 Quyết định 3), rồi đọc lại để
    // chứng minh quần thể works/chapters/segments KHÔNG đổi, chỉ trạng thái đổi. Sau đó
    // phục hồi lại `done` — nghệ thuật xuất ra đĩa vẫn là quần thể "full" sạch, phù hợp
    // dùng làm gốc mặc định cho mọi bàn đo khác (NFR3 cần `indexed_segments == 50_000` bất
    // kể trạng thái Chương; NFR4/NFR5 tự chuyển trạng thái lúc chạy — Story 6.18 task 5).
    // `open_work` thuần (đúng đường `commands::project::wire::open_work` đi qua) đòi một
    // `IndexedWork` từ CHÍNH chỉ mục vừa rebuild — không một `OpenWork` tự ráp tay nào ở
    // đây, cùng kỷ luật "chỉ hàm sản phẩm" của toàn tệp này.
    let indexed_before_frontier = indexer.list_works(WorkQuery::default()).expect("liệt kê trước frontier");
    let frontier_target = indexed_before_frontier
        .works
        .iter()
        .min_by(|a, b| a.name.cmp(&b.name))
        .expect("ít nhất một Tác phẩm đã lập chỉ mục")
        .clone();
    let frontier_work_dir = frontier_target.atproj_path.clone();
    let mut frontier_open = auratranslate_lib::commands::project::open_work(
        &frontier_target.work_id,
        Some(&frontier_target),
    )
    .unwrap_or_else(|e| panic!("open_work({}) thất bại: {e:?}", frontier_target.work_id));

    println!("STORY_6_18_BUILD\tphase=frontier_transition\twork={}", frontier_work_dir.display());
    set_all_chapters(&mut frontier_open, "not_started");
    drop(frontier_open);

    indexer.rebuild(&documents_root, Some(&global_store)).expect("rebuild frontier");
    let frontier = read_back_population(&documents_root);
    assert_eq!(frontier.works, WORKS, "quần thể frontier: sai số Tác phẩm");
    assert_eq!(frontier.chapters, TOTAL_CHAPTERS as i64, "quần thể frontier: sai số Chương");
    assert_eq!(frontier.live_segments, TOTAL_SEGMENTS as i64, "quần thể frontier: sai số segment sống");
    assert_eq!(
        frontier.not_started_chapters,
        CHAPTERS_PER_WORK as i64,
        "quần thể frontier: phải đúng {CHAPTERS_PER_WORK} Chương not_started (một Tác phẩm)"
    );
    assert_eq!(
        frontier.done_chapters,
        (TOTAL_CHAPTERS - CHAPTERS_PER_WORK) as i64,
        "quần thể frontier: 49 Tác phẩm còn lại phải vẫn done"
    );
    println!(
        "STORY_6_18_POPULATION\tstate=frontier\tworks={}\tchapters={}\tsegments={}\tdone_chapters={}\tnot_started_chapters={}\tverdict=matches_declaration",
        frontier.works, frontier.chapters, frontier.live_segments, frontier.done_chapters, frontier.not_started_chapters
    );

    // Phục hồi — export cuối cùng trên đĩa là quần thể "full". Chỉ mục đã đổi (Chương vừa
    // sang `not_started`) nên phải `find_work` LẠI, không tái dùng `frontier_target` cũ.
    let indexed_after_frontier = indexer
        .find_work(&frontier_target.work_id)
        .expect("tìm lại Tác phẩm frontier")
        .expect("Tác phẩm frontier vẫn phải còn trong chỉ mục");
    let mut restore_open = auratranslate_lib::commands::project::open_work(
        &frontier_target.work_id,
        Some(&indexed_after_frontier),
    )
    .unwrap_or_else(|e| panic!("open_work({}) thất bại lúc phục hồi: {e:?}", frontier_target.work_id));
    set_all_chapters(&mut restore_open, "done");
    drop(restore_open);
    indexer.rebuild(&documents_root, Some(&global_store)).expect("rebuild phục hồi full");

    let restored = read_back_population(&documents_root);
    assert_eq!(restored.done_chapters, TOTAL_CHAPTERS as i64, "phục hồi full thất bại");
    assert_eq!(restored.not_started_chapters, 0, "phục hồi full thất bại: vẫn còn not_started");

    // Chỉ mục vừa dựng phải thấy ĐÚNG quần thể sống — bàn đo NFR3 (task 4) đọc THẲNG chỉ mục
    // này, không tự `rebuild` lại.
    let indexed = indexer.list_works(WorkQuery::default()).expect("liệt kê Tác phẩm đã lập chỉ mục");
    assert_eq!(indexed.works.len(), WORKS, "chỉ mục không thấy đủ {WORKS} Tác phẩm");

    println!(
        "STORY_6_18_POPULATION\tstate=final_export\tworks={}\tchapters={}\tsegments={}\tdocuments_root={}\tindex_path={}",
        restored.works,
        restored.chapters,
        restored.live_segments,
        documents_root.display(),
        index_path.display()
    );

    drop(indexer);
    drop(global_store);
    cleanup(&documents_root);
}
