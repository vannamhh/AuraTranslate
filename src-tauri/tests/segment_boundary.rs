//! Ranh giới cây nguồn của Story 2.1 — AC3 · AC8 vế hai · AC12.
//!
//! ⚠️ Tệp riêng có chủ ý, đúng khuôn `store_boundary.rs`/`scope_boundary.rs`:
//! `segment_contract.rs` khai phạm vi của nó ở dòng 1 (*hành vi lúc chạy*); đây là phép kiểm
//! **tĩnh trên cây nguồn**, và trộn hai thứ là làm hỏng đúng thứ khiến cả hai đọc được.
//!
//! ─────────────────────────────────────────────────────────────────────────────
//! BA MỆNH ĐỀ, VÀ MỖI MỆNH ĐỀ HỎNG BẰNG ĐÚNG MỘT DÒNG MÀ MỌI THỨ KHÁC VẪN XANH
//! ─────────────────────────────────────────────────────────────────────────────
//! 1. **AC12** — bộ tách sống ở Rust, trong `core/segment/`, và **không một đường nào** ở
//!    TypeScript. `EXPERIENCE.md:23` khai đích danh: *"Frontend không chứa quy tắc nghiệp vụ
//!    (AD-1). Tách câu, khớp ngôn ngữ, phân giải scope đều nằm ở Rust."*
//! 2. **AC3** — *"không đường mã nào tính lại ranh giới lúc nạp Chương"*. Một lời gọi bộ
//!    tách thêm vào `read_open_chapter` biên dịch sạch, chạy được, cho ra kết quả trông
//!    đúng — và phá AD-4 (ranh giới đóng băng vĩnh viễn lúc ghi) mà không gì báo.
//! 3. **AC8 vế hai** — *"không có đường nào tự động tách lại toàn bộ Thư viện"*. Mệnh đề này
//!    được giao bằng cách một đường như thế **không tồn tại**, nên nó chỉ cưỡng chế được
//!    bằng một phép đếm chỗ gọi.
//!
//! ─────────────────────────────────────────────────────────────────────────────
//! 🔴 SÀN QUẦN THỂ LÀ BẮT BUỘC
//! ─────────────────────────────────────────────────────────────────────────────
//! *"Cây rỗng đọc thành sạch"* — bài học thừa kế từ `check-deps.mjs` và `store_boundary.rs`.
//! Một đường dẫn gõ sai làm `walk` khớp 0 tệp ⇒ mọi vòng lặp dưới đây xanh mà không kiểm gì
//! cả ⇒ cổng chết im lặng ngay ngày nó ra đời.

use std::fs;
use std::path::{Path, PathBuf};

#[path = "support/boundary_scan.rs"]
#[allow(dead_code)] // shared module: not every helper is used in this file
mod boundary_scan;
use boundary_scan::is_inside;

/// Thư mục DUY NHẤT được phép mang bảng chữ cái kết câu.
const SEGMENT_DIR: &str = "core/segment";

/// Đường đọc Chương — nó **không** được gọi bộ tách (AC3).
const CHAPTER_READ_FILE: &str = "commands/chapter.rs";

/// Số tệp `.rs` tối thiểu dưới `src-tauri/src/**` để phép quét là thật.
///
/// 🔴 **Quần thể này KHÁC quần thể của `check-i18n.mjs`** — ở đây là `src-tauri/src/**`,
/// ở đó là `src-tauri/**` sau miễn trừ `tests/**` (gồm cả `build.rs`). Hai con số gần nhau
/// và chúng **không** thay thế nhau được.
const SRC_RS_FLOOR: usize = 84;

/// Số tệp `.ts` + `.vue` tối thiểu dưới `src/**`.
const WEBVIEW_FLOOR: usize = 94;

/// Bảng chữ cái kết câu tiếng Trung — AC1. **Chỉ** `core/segment/**` được mang nó.
///
/// ⚠️ Bốn ký tự này chứ không phải `.`/`!`/`?`: dấu ASCII xuất hiện trong mọi câu tiếng Anh
/// ở mọi doc-comment của kho, nên một phép so trên chúng đỏ ở khắp nơi và bị gỡ trong tuần.
/// Bốn ký tự toàn giác thì **không** xuất hiện ở đâu khác — đo 2026-08-12: **0** lần trong
/// `src-tauri/src/**` trước story này.
const ZH_TERMINATORS: [char; 4] = ['。', '！', '？', '；'];

fn src_root() -> PathBuf {
    boundary_scan::src_root()
}

/// Gốc của cây webview — `src/` ở **thư mục kho**, không phải `src-tauri/src/`.
fn webview_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("`src-tauri` phải có thư mục cha")
        .join("src")
}

fn rel_posix(root: &Path, file: &Path) -> String {
    boundary_scan::rel_posix(root, file)
}

fn rust_sources() -> (PathBuf, Vec<PathBuf>) {
    let root = src_root();
    let files = boundary_scan::rust_sources(&root)
        .into_iter()
        .map(|(rel, _)| root.join(rel))
        .collect();
    (root, files)
}

fn webview_sources() -> (PathBuf, Vec<PathBuf>) {
    let root = webview_root();
    let files = boundary_scan::paths_with_extensions(&root, &["ts", "vue"]);
    (root, files)
}

fn read(file: &Path) -> String {
    fs::read_to_string(file).unwrap_or_else(|e| panic!("đọc {}: {e}", file.display()))
}

// ═════════════════════════════════════════════════════════════════════════════
// Sàn quần thể — chạy trước mọi phép kiểm khác
// ═════════════════════════════════════════════════════════════════════════════

#[test]
fn the_scanned_trees_are_large_enough_to_be_real() {
    let (_, rust) = rust_sources();
    boundary_scan::assert_population_floor(
        SRC_RS_FLOOR,
        rust.len(),
        "SRC_RS_FLOOR",
        "tệp `.rs` dưới `src-tauri/src/**`",
    );

    let (_, webview) = webview_sources();
    boundary_scan::assert_population_floor(
        WEBVIEW_FLOOR,
        webview.len(),
        "WEBVIEW_FLOOR",
        "tệp `.ts`/`.vue` dưới `src/**`",
    );
}

// ═════════════════════════════════════════════════════════════════════════════
// AC12 — bảng chữ cái kết câu chỉ sống ở `core/segment/**`
// ═════════════════════════════════════════════════════════════════════════════

#[test]
fn only_core_segment_may_name_the_sentence_terminators() {
    let (root, files) = rust_sources();

    let mut violations: Vec<String> = Vec::new();
    let mut segment_files = 0usize;

    for file in &files {
        let rel = rel_posix(&root, file);
        if is_inside(&rel, SEGMENT_DIR) {
            segment_files += 1;
            continue;
        }

        for (index, code) in boundary_scan::code_lines(&read(file)) {
            for needle in ZH_TERMINATORS {
                if code.contains(needle) {
                    violations.push(format!("{rel}:{index}  {needle}  |  {code}"));
                }
            }
        }
    }

    // Miễn trừ phải khớp thứ gì đó. Một `SEGMENT_DIR` gõ sai làm nhánh `continue` không bao
    // giờ chạy — phép kiểm vẫn xanh hôm nay, rồi đỏ sai chỗ vào ngày ai đó đổi tên thư mục.
    assert!(
        segment_files > 0,
        "không tệp nào khớp `{SEGMENT_DIR}` — đường dẫn miễn trừ đã lệch khỏi cây nguồn"
    );

    assert!(
        violations.is_empty(),
        "{} chỗ ngoài `{SEGMENT_DIR}` mang bảng chữ cái kết câu:\n{}\n\n\
         AC12: quy tắc tách câu sống ở **một** chỗ. Một bảng chữ cái thứ hai ở một module \
         khác là hai bộ luật rẽ nhau — và AD-4 đóng băng kết quả của chúng VĨNH VIỄN lúc ghi.",
        violations.len(),
        violations.join("\n")
    );
}

/// Đối chứng dương: `core::segment::split` **có thật sự** mang bảng chữ cái đó.
///
/// ⚠️ Không có ca này thì phép kiểm trên xanh y hệt trên một cây mà bộ tách đã bị xoá —
/// *"không ai vi phạm"* và *"không có gì để vi phạm"* đọc giống nhau.
#[test]
fn core_segment_split_actually_carries_the_terminators() {
    let text = read(&src_root().join("core/segment/split.rs"));

    for needle in ZH_TERMINATORS {
        assert!(
            text.contains(needle),
            "`core/segment/split.rs` không còn mang `{needle}` — AC1 đòi tách theo `。！？；`"
        );
    }
}

// ═════════════════════════════════════════════════════════════════════════════
// AC12 vế TypeScript — không đường tách câu nào ở webview
// ═════════════════════════════════════════════════════════════════════════════

/// `Intl.Segmenter` với `granularity: 'word'` là **hợp lệ** (Story 1.18b, tách TỪ tiếng
/// Trung cho tab Hán Việt). Với `granularity: 'sentence'` thì **không**: đó là tách câu ở
/// webview, và nó chạy **mỗi lần Chương nạp** — đúng thứ AC3 cấm.
#[test]
fn no_webview_file_asks_for_sentence_granularity() {
    let (root, files) = webview_sources();

    let mut violations: Vec<String> = Vec::new();
    let mut segmenter_hits = 0usize;

    for file in &files {
        let rel = rel_posix(&root, file);
        for (index, line) in read(file).lines().enumerate() {
            if line.contains("Intl.Segmenter") {
                segmenter_hits += 1;
            }
            // So trên dòng đã bỏ khoảng trắng để bắt cả `granularity:'sentence'` viết liền.
            let squeezed: String = line.chars().filter(|c| !c.is_whitespace()).collect();
            if squeezed.contains("granularity:'sentence'")
                || squeezed.contains("granularity:\"sentence\"")
            {
                violations.push(format!("{rel}:{}  {}", index + 1, line.trim()));
            }
        }
    }

    assert!(
        segmenter_hits > 0,
        "không tệp webview nào nhắc tới `Intl.Segmenter` — cây quét đã lệch, vì \
         `src/panels/wordBoundary.ts` (Story 1.18b) chắc chắn mang nó"
    );

    assert!(
        violations.is_empty(),
        "{} chỗ ở webview xin `granularity: 'sentence'`:\n{}\n\n\
         AC12 + AD-1: tách câu nằm ở Rust. Và AC3 cấm mọi đường tính lại ranh giới lúc nạp \
         Chương — `Intl.Segmenter` chạy đúng mỗi lần đó.",
        violations.len(),
        violations.join("\n")
    );
}

// ═════════════════════════════════════════════════════════════════════════════
// AC3 — đường đọc Chương không gọi bộ tách
// ═════════════════════════════════════════════════════════════════════════════

#[test]
fn the_chapter_read_path_never_calls_the_splitter() {
    let text = read(&src_root().join(CHAPTER_READ_FILE));

    let offenders: Vec<String> = boundary_scan::code_lines(&text)
        .filter(|(_, code)| code.contains("split_source_text") || code.contains("segment::split"))
        .map(|(index, code)| format!("{CHAPTER_READ_FILE}:{index}  {code}"))
        .collect();

    assert!(
        offenders.is_empty(),
        "đường đọc Chương đã gọi bộ tách:\n{}\n\n\
         AC3: *\"không đường mã nào tính lại ranh giới lúc nạp Chương\"*. AD-4 đóng băng ranh \
         giới lúc GHI; tính lại lúc đọc là hai bộ ranh giới cho cùng một Chương, và `segment.id` \
         của Story 2.6 sẽ trỏ vào bộ nào không ai biết.",
        offenders.join("\n")
    );
}

// ═════════════════════════════════════════════════════════════════════════════
// AC8 vế hai — không đường nào tự động tách lại toàn bộ Thư viện
// ═════════════════════════════════════════════════════════════════════════════

/// Bộ tách có **đúng một** chỗ gọi sản phẩm NGOÀI `core/segment/`: lệnh tách tường minh
/// (Chương cũ, một Chương một lượt — Quyết định #4 của `commands::segment`).
///
/// 🔵 **SỬA 2026-09-04 (Story 6.2, AD-39) — từ "đúng HAI" xuống "đúng MỘT".** Trước story
/// này, `commands/project.rs::create_work` cũng gọi thẳng `split_source_text` (Chương MỚI,
/// cùng giao dịch — AC13 cũ). Story 6.2 dời lời gọi đó vào `core/segment/pipeline.rs`
/// (bước 7 của chuỗi AD-39, "tách segment + cờ kết đoạn"), nơi `SEGMENT_DIR` miễn trừ nó
/// khỏi phép quét này — [`the_pipeline_module_actually_calls_the_splitter`] ngay dưới là
/// đối chứng dương rằng chỗ gọi đó THẬT SỰ có mặt, không phải biến mất khỏi cả hai nơi.
/// `commands/project.rs` giờ chỉ gọi `run_import` (`core::segment::pipeline`), không còn tự
/// tách gì — xem `segment_pipeline_boundary.rs::run_import_is_the_one_product_call_site`.
///
/// Một chỗ gọi **thứ hai** ngoài `core/segment/` là thứ phải có người ký: nó hoặc là một
/// đường tính ngầm (AC3), hoặc là một đường tách hàng loạt (AC8 vế hai). Ca này bắt cả hai
/// bằng một phép đếm.
#[test]
fn the_splitter_has_exactly_one_product_call_site_outside_core_segment() {
    let (root, files) = rust_sources();

    let mut call_sites: Vec<String> = Vec::new();
    for file in &files {
        let rel = rel_posix(&root, file);
        // Chính module định nghĩa bộ tách, và pipeline dùng nó — không phải một "chỗ gọi
        // ngoài" theo nghĩa của phép kiểm này.
        if is_inside(&rel, SEGMENT_DIR) {
            continue;
        }
        for (index, code) in boundary_scan::code_lines(&read(file)) {
            if code.contains("split_source_text") {
                call_sites.push(format!("{rel}:{index}  {code}"));
            }
        }
    }

    // 🔵 SỬA (vòng rà đối kháng 2026-09-04, item 11) — khôi phục ĐẾM CHÍNH XÁC. Bản đầu chỉ
    // kiểm THÀNH VIÊN ("có mặt ở tệp đã ký, không có mặt ở tệp khác") — một phép kiểm thành
    // viên thuần xanh y hệt dù `commands/segment.rs` mọc từ 2 chỗ khớp (1 `use` + 1 lời gọi)
    // lên 5, 10, hay bất kỳ số nào, miễn là KHÔNG có chỗ khớp ở một tệp khác. Task của spec
    // 6.2 ghi rõ "cổng dời theo mã, không được nới" — số thật hôm nay là 2
    // (`commands/segment.rs:45` dòng `use`, `:334` lời gọi), và đó là con số cổng này phải
    // giữ, không phải một tập hợp tên tệp.
    assert_eq!(
        call_sites.len(),
        2,
        "số dòng khớp `split_source_text` ngoài `core/segment/` đã đổi — kỳ vọng ĐÚNG 2 (một          dòng `use` + một lời gọi, cả hai trong `commands/segment.rs`), tìm thấy {}:
{}",
        call_sites.len(),
        call_sites.join("
")
    );
    for site in &call_sites {
        assert!(
            site.starts_with("commands/segment.rs"),
            "chỗ khớp phải ở `commands/segment.rs` — tìm thấy: {site}"
        );
    }
}

/// Đối chứng dương: `core/segment/pipeline.rs` **thật sự** gọi bộ tách — không có ca này,
/// phép kiểm trên xanh y hệt trên một cây mà bước 7 của chuỗi ĐÃ MẤT lời gọi (xoá tay, hoặc
/// một lượt viết lại bằng tay thay vì "Gọi, đừng viết lại" — Code Map của spec 6.2).
#[test]
fn the_pipeline_module_actually_calls_the_splitter() {
    // 🔵 SỬA (vòng rà đối kháng 2026-09-04, item 12) — bản đầu dùng `text.contains(..)` trên
    // TOÀN VĂN BẢN tệp; một lời gọi bị comment hoặc một dòng doc-comment nhắc chuỗi này vẫn
    // giữ ca xanh. Lọc chú thích qua `boundary_scan::code_lines`.
    let text = read(&src_root().join("core/segment/pipeline.rs"));
    let has_call =
        boundary_scan::code_lines(&text).any(|(_, code)| code.contains("split_source_text("));
    assert!(
        has_call,
        "`core/segment/pipeline.rs` không còn gọi `split_source_text` (ngoài chú thích) — \
         bước 7 của chuỗi AD-39 (\"tách segment + cờ kết đoạn\") phải GỌI bộ tách sẵn có, \
         không viết lại nó."
    );
}

// ═════════════════════════════════════════════════════════════════════════════
// Bộ tách là một hàm THUẦN — không I/O, không `Connection`
// ═════════════════════════════════════════════════════════════════════════════

/// Cùng khuôn `import_text`: bộ tách nhận `&str`, trả `Vec`, và không chạm đĩa hay database.
///
/// Vì sao thành một test: mệnh đề này hỏng bằng **một dòng `use`**, và cái giá chỉ hiện ra ở
/// mọi ca test phải dựng một `Store` để kiểm một luật tách câu — tức đúng lúc không ai muốn
/// viết thêm ca nữa.
#[test]
fn the_splitter_stays_pure() {
    let text = read(&src_root().join("core/segment/split.rs"));

    let offenders: Vec<String> = boundary_scan::code_lines(&text)
        .filter(|(_, code)| {
            code.contains("Transaction")
                || code.contains("std::fs")
                || code.contains("Store")
                || code.contains("use tauri")
        })
        .map(|(index, code)| format!("split.rs:{index}  {code}"))
        .collect();

    assert!(
        offenders.is_empty(),
        "bộ tách đã chạm tầng I/O hoặc tầng ghi:\n{}\n\n\
         Nó phải ở lại một hàm thuần — cùng khuôn `core::segment::import::import_text`.",
        offenders.join("\n")
    );
}

// ═════════════════════════════════════════════════════════════════════════════════
// Story 2.3 · AC12 — LỆNH GHI MỚI ĐI QUA ĐÚNG `store::Writer`, KHÔNG MỞ ĐƯỜNG THỨ HAI
// ═════════════════════════════════════════════════════════════════════════════════

/// AC12 của Story 2.3 đòi bằng chữ *"một ca ở `store_boundary.rs` **hoặc**
/// `segment_boundary.rs` khẳng định lệnh ghi mới không mở kết nối nào và không nhận `&Store`
/// ở tầng dưới"*. Code review 2026-08-13 tìm ra rằng mệnh đề đó chỉ được phủ **gián tiếp** —
/// bởi `store_boundary.rs::only_core_store_may_name_rusqlite`, một cổng quét cả cây và vì thế
/// phủ `commands/segment.rs` mà không ai phải viết gì.
///
/// 🔴 Gián tiếp là **chưa đủ**, và lý do không phải hình thức: cổng kia canh *"ai được nhắc
/// tên `rusqlite`"*. Nó **không** canh vế thứ hai của AC12 — *"không nhận `&Store` ở tầng
/// dưới"* — vì `&Store` là một kiểu **được phép** ở `commands/**`. Một lượt refactor đổi
/// `fn write_targets(tx: &Transaction)` thành `fn write_targets(store: &Store)` biên dịch
/// sạch, không nhắc `rusqlite` một chữ, đi qua trọn cổng kia — và mở một giao dịch **thứ hai**
/// trên writer nối tiếp của AD-11, đúng đường hỏng mà doc-comment của `insert_segments`
/// (`commands/segment.rs:60-62`) đã ghi tên từ Story 2.1.
///
/// ⚠️ Ca này đọc **bản đã che** *(bỏ dòng chú thích)*, cùng khuôn mọi phép kiểm tĩnh khác ở
/// tệp này: doc-comment của chính hàm ghi có chuỗi `&Store` trong câu giải thích *vì sao*
/// nó không nhận `&Store`, và một bộ đọc thô sẽ đỏ vì đúng đoạn văn dạy nó điều phải kiểm.
#[test]
fn the_target_write_path_never_takes_a_store_below_the_command_shell() {
    let text = read(&src_root().join("commands/segment.rs"));

    // Thân closure trao cho `Store::write` nhận `&Transaction` — đó là hình dạng ĐÚNG, và nó
    // phải còn đó. Mất nó nghĩa là đường ghi đã đổi hình dạng dưới chân AC12.
    assert!(
        text.contains("write(move |tx: &Transaction<'_>|"),
        "không còn một closure `Store::write(move |tx: &Transaction<'_>| …)` nào trong \
         `commands/segment.rs`. AC12 của Story 2.3 đứng trên hình dạng đó: một lô, một giao \
         dịch, và tầng dưới nhận `&Transaction` chứ không `&Store`."
    );

    // Không hàm nào Ở TẦNG DƯỚI được nhận `&Store`. Vỏ lệnh nhận `Option<&OpenWork>` và đọc
    // `open.store` — đó là tầng TRÊN, và nó được phép.
    let offenders: Vec<String> = boundary_scan::code_lines(&text)
        .filter(|(_, code)| code.contains("&Store") || code.contains("Connection::open"))
        .map(|(index, code)| format!("segment.rs:{index}  {code}"))
        .collect();

    assert!(
        offenders.is_empty(),
        "đường ghi bản dịch đã nhận `&Store` ở tầng dưới, hoặc đã mở một kết nối riêng:\n{}\n\n\
         AD-11 có MỘT writer nối tiếp. Một hàm dưới vỏ lệnh cầm `&Store` sẽ mở một giao dịch \
         thứ hai — cùng đường hỏng mà `insert_segments` đã ghi từ Story 2.1.",
        offenders.join("\n")
    );
}
