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

/// Thư mục DUY NHẤT được phép mang bảng chữ cái kết câu.
const SEGMENT_DIR: &str = "core/segment";

/// Đường đọc Chương — nó **không** được gọi bộ tách (AC3).
const CHAPTER_READ_FILE: &str = "commands/chapter.rs";

/// Số tệp `.rs` tối thiểu dưới `src-tauri/src/**` để phép quét là thật.
///
/// Số thật lúc dựng (Story 2.1): **42** tệp — 40 kế thừa sau Story 1.21, cộng
/// `core/segment/split.rs` và `commands/segment.rs`. Sàn đặt **dưới** số thật đúng khuôn
/// `RS_FLOOR` của `store_boundary.rs`: nó bắt một cây **bị cắt**, không bắt việc thêm tệp mới.
///
/// 🔴 **Quần thể này KHÁC quần thể của `check-i18n.mjs`** — ở đây là `src-tauri/src/**`,
/// ở đó là `src-tauri/**` sau miễn trừ `tests/**` (gồm cả `build.rs`). Hai con số gần nhau
/// và chúng **không** thay thế nhau được.
const SRC_RS_FLOOR: usize = 34;

/// Số tệp `.ts` + `.vue` tối thiểu dưới `src/**`.
///
/// Số thật lúc dựng (Story 2.1): **47** — 31 tệp `.ts` (30 kế thừa + `config/segment.ts`)
/// và 15 tệp `.vue` … cộng lại 47. Sàn **38** (~81%), cùng tỷ lệ dư địa mà `TS_FLOOR`/
/// `VUE_FLOOR` của `check-commands.mjs` đang giữ.
const WEBVIEW_FLOOR: usize = 38;

/// Bảng chữ cái kết câu tiếng Trung — AC1. **Chỉ** `core/segment/**` được mang nó.
///
/// ⚠️ Bốn ký tự này chứ không phải `.`/`!`/`?`: dấu ASCII xuất hiện trong mọi câu tiếng Anh
/// ở mọi doc-comment của kho, nên một phép so trên chúng đỏ ở khắp nơi và bị gỡ trong tuần.
/// Bốn ký tự toàn giác thì **không** xuất hiện ở đâu khác — đo 2026-08-12: **0** lần trong
/// `src-tauri/src/**` trước story này.
const ZH_TERMINATORS: [char; 4] = ['。', '！', '？', '；'];

fn src_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src")
}

/// Gốc của cây webview — `src/` ở **thư mục kho**, không phải `src-tauri/src/`.
fn webview_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("`src-tauri` phải có thư mục cha")
        .join("src")
}

/// Đường dẫn tương đối, dùng dấu `/` trên cả hai nền tảng.
///
/// ⚠️ Chuẩn hoá `\` thành `/` là bắt buộc chứ không phải làm đẹp: `starts_with(SEGMENT_DIR)`
/// trên Windows so với `core\segment` và **không bao giờ khớp**, nên miễn trừ biến mất và
/// mọi tệp của chính `core::segment` bị báo vi phạm — một test đỏ **chỉ trên một nhánh** của
/// ma trận, đúng lớp lỗi NFR14.
fn rel_posix(root: &Path, file: &Path) -> String {
    file.strip_prefix(root)
        .unwrap_or(file)
        .to_string_lossy()
        .replace('\\', "/")
}

fn walk(dir: &Path, extensions: &[&str], out: &mut Vec<PathBuf>) {
    let entries = fs::read_dir(dir).unwrap_or_else(|e| panic!("đọc {}: {e}", dir.display()));
    for entry in entries {
        let entry = entry.unwrap_or_else(|e| panic!("duyệt {}: {e}", dir.display()));
        let path = entry.path();
        let meta =
            fs::symlink_metadata(&path).unwrap_or_else(|e| panic!("lstat {}: {e}", path.display()));

        // ⚠️ `symlink_metadata`, không `metadata`: `metadata` giải symlink, nên một liên kết
        // trỏ về thư mục cha làm đệ quy không dừng. Cùng bài học `store_boundary.rs`.
        if meta.file_type().is_symlink() {
            continue;
        }
        if meta.is_dir() {
            walk(&path, extensions, out);
        } else if path
            .extension()
            .and_then(|e| e.to_str())
            .is_some_and(|e| extensions.contains(&e))
        {
            out.push(path);
        }
    }
}

fn sources(root: PathBuf, extensions: &[&str]) -> (PathBuf, Vec<PathBuf>) {
    let mut files = Vec::new();
    walk(&root, extensions, &mut files);
    files.sort();
    (root, files)
}

fn rust_sources() -> (PathBuf, Vec<PathBuf>) {
    sources(src_root(), &["rs"])
}

fn webview_sources() -> (PathBuf, Vec<PathBuf>) {
    sources(webview_root(), &["ts", "vue"])
}

fn read(file: &Path) -> String {
    fs::read_to_string(file).unwrap_or_else(|e| panic!("đọc {}: {e}", file.display()))
}

/// Dòng bắt đầu bằng `//` là **tài liệu về một ranh giới**, không phải một lời gọi vượt qua
/// nó — cùng luật mà `store_boundary.rs` và `check-i18n.mjs` Kiểm A đều áp, và vì cùng lý do:
/// một cổng đỏ trên câu giải thích chính luật nó canh là một cổng bị gỡ trong tuần.
fn is_comment(line: &str) -> bool {
    let code = line.trim_start();
    code.starts_with("//") || code.starts_with("* ") || code.starts_with("*/")
}

// ═════════════════════════════════════════════════════════════════════════════
// Sàn quần thể — chạy trước mọi phép kiểm khác
// ═════════════════════════════════════════════════════════════════════════════

#[test]
fn the_scanned_trees_are_large_enough_to_be_real() {
    let (_, rust) = rust_sources();
    assert!(
        rust.len() >= SRC_RS_FLOOR,
        "chỉ tìm thấy {} tệp `.rs` dưới `src-tauri/src/**` (sàn {SRC_RS_FLOOR}). Cây quá nhỏ \
         để là thật — một danh sách rỗng làm mọi phép kiểm dưới đây xanh mà không kiểm gì cả.",
        rust.len()
    );

    let (_, webview) = webview_sources();
    assert!(
        webview.len() >= WEBVIEW_FLOOR,
        "chỉ tìm thấy {} tệp `.ts`/`.vue` dưới `src/**` (sàn {WEBVIEW_FLOOR}). Gốc quét sai, \
         hoặc một thư mục bị bỏ.",
        webview.len()
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
        if rel.starts_with(SEGMENT_DIR) {
            segment_files += 1;
            continue;
        }

        for (index, line) in read(file).lines().enumerate() {
            if is_comment(line) {
                continue;
            }
            for needle in ZH_TERMINATORS {
                if line.contains(needle) {
                    violations.push(format!("{rel}:{}  {needle}  |  {}", index + 1, line.trim()));
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

    let offenders: Vec<String> = text
        .lines()
        .enumerate()
        .filter(|(_, line)| !is_comment(line))
        .filter(|(_, line)| line.contains("split_source_text") || line.contains("segment::split"))
        .map(|(index, line)| format!("{CHAPTER_READ_FILE}:{}  {}", index + 1, line.trim()))
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

/// Bộ tách có **đúng hai** chỗ gọi trên đường sản phẩm, và cả hai đều đã đặt tên:
/// `create_work` (Chương mới, cùng giao dịch — AC13) và lệnh tách tường minh (Chương cũ,
/// một Chương một lượt — Quyết định #4).
///
/// Một chỗ gọi **thứ ba** là thứ phải có người ký: nó hoặc là một đường tính ngầm (AC3), hoặc
/// là một đường tách hàng loạt (AC8 vế hai). Ca này bắt cả hai bằng một phép đếm.
#[test]
fn the_splitter_has_exactly_two_product_call_sites() {
    let (root, files) = rust_sources();

    let mut call_sites: Vec<String> = Vec::new();
    for file in &files {
        let rel = rel_posix(&root, file);
        // Chính module định nghĩa bộ tách không phải một chỗ gọi.
        if rel.starts_with(SEGMENT_DIR) {
            continue;
        }
        for (index, line) in read(file).lines().enumerate() {
            if is_comment(line) {
                continue;
            }
            if line.contains("split_source_text") {
                call_sites.push(format!("{rel}:{}  {}", index + 1, line.trim()));
            }
        }
    }

    // Hai lời gọi + hai dòng `use` mang tên hàm vào phạm vi = 4 dòng khớp.
    let expected: [&str; 2] = ["commands/project.rs", "commands/segment.rs"];
    for file in expected {
        assert!(
            call_sites.iter().any(|c| c.starts_with(file)),
            "`{file}` phải gọi bộ tách — nếu nó không còn gọi, một trong hai đường nhập đã mất \
             phép tách. Các chỗ gọi tìm thấy:\n{}",
            call_sites.join("\n")
        );
    }

    let unexpected: Vec<&String> = call_sites
        .iter()
        .filter(|c| !expected.iter().any(|f| c.starts_with(f)))
        .collect();

    assert!(
        unexpected.is_empty(),
        "bộ tách có chỗ gọi ngoài hai đường đã ký:\n{unexpected:#?}\n\n\
         AC3 cấm một đường tính ngầm lúc nạp Chương; AC8 vế hai cấm một đường tự động tách lại \
         toàn bộ Thư viện. Một chỗ gọi thứ ba phải có người ký, không phải một hiệu ứng phụ."
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

    let offenders: Vec<String> = text
        .lines()
        .enumerate()
        .filter(|(_, line)| !is_comment(line))
        .filter(|(_, line)| {
            line.contains("Transaction")
                || line.contains("std::fs")
                || line.contains("Store")
                || line.contains("use tauri")
        })
        .map(|(index, line)| format!("split.rs:{}  {}", index + 1, line.trim()))
        .collect();

    assert!(
        offenders.is_empty(),
        "bộ tách đã chạm tầng I/O hoặc tầng ghi:\n{}\n\n\
         Nó phải ở lại một hàm thuần — cùng khuôn `core::segment::import::import_text`.",
        offenders.join("\n")
    );
}
