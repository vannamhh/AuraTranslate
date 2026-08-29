//! Ranh giới cây nguồn của AC4 (Story 5.5) — cưỡng chế bằng máy mệnh đề *"không thành phần
//! nào khác ghi vào `meta.json`"* (FR7, epic-5-context.md §Technical Decisions).
//!
//! ⚠️ Tệp riêng có chủ ý, đúng khuôn `library_index_boundary.rs`: đây là phép kiểm **tĩnh
//! trên cây nguồn** — hành vi LÚC CHẠY của `WorkMeta::rebuild_from_store` (đếm đúng, ghi đè
//! không đổi tiến độ, ...) sống ở `library_index_contract.rs` · `lifecycle_contract.rs` ·
//! `project_contract.rs`; trộn hai thứ làm hỏng đúng thứ khiến cả hai đọc được.
//!
//! ─────────────────────────────────────────────────────────────────────────────
//! BA MỆNH ĐỀ CƯỠNG CHẾ, CHÉP KHUÔN `library_index_boundary.rs` (Phần 1–4 mỗi mệnh đề)
//! ─────────────────────────────────────────────────────────────────────────────
//! (a) `write_atomic(` trên `WorkMeta` chỉ được NHẮC TỚI Ở VỊ TRÍ MÃ trong BA tệp: nơi hàm
//!     được khai (`core/library/meta.rs`) và hai chỗ gọi sản phẩm đã đóng của FR7
//!     (`commands/project.rs` sau `create_work`, `commands/lifecycle.rs` sau mỗi lượt đổi
//!     trạng thái) — §Never của story: "không thêm một chỗ gọi ghi `meta.json` thứ ba".
//! (b) `META_FILE` và chuỗi `"meta.json"` Ở VỊ TRÍ MÃ chỉ được nhắc tới trong
//!     `core/library/meta.rs` — chỗ DUY NHẤT tên tệp trên đĩa được viết ra; mọi nơi khác phải
//!     đi qua `WorkMeta::read`/`write_atomic`, không tự lắp một chuỗi `"meta.json"` song song.
//! (c) `WorkMeta::read` chỉ được NHẮC TỚI Ở VỊ TRÍ MÃ trong BA tệp: `core/library/meta.rs`
//!     (định nghĩa), `core/library/indexer.rs` (đường quét sản phẩm — AC2 của Story 5.2 đã
//!     đóng việc `Indexer` là nơi duy nhất mở kho dẫn xuất; đây là nửa còn lại, "nơi duy
//!     nhất đọc `meta.json` lúc quét"), và 🔵 **THÊM (2026-08-29, Story 5.7) —
//!     `commands/project.rs`** (`open_work`, đường mở lại một `.atproj` **đã có trên đĩa**
//!     mà story 5.5 từng ghi là "chưa tồn tại" — xem doc-comment của `WorkMeta::read`).
//!     Đây là chỗ đọc thứ BA, không phải một chỗ đi vòng: số chỗ đọc thật sự tăng từ 2 lên
//!     3, và con số trong cổng tăng theo đúng nó (xem §Design Notes "`meta_write_boundary.rs`
//!     — vì sao NỚI danh sách chứ không đi vòng" của
//!     `5-7-danh-sach-chuong-va-mo-chuong-vao-workspace.md`).
//!
//! Mỗi mệnh đề mang bốn phần, đúng khuôn `library_index_boundary.rs`:
//! 1. needle cấm (violations rỗng ngoài `EXEMPT_FILES` tương ứng);
//! 2. đối chứng dương — needle THẬT SỰ xuất hiện ở (ít nhất) chỗ gọi sản phẩm đã biết, không
//!    phải một tập miễn trừ đang canh một tập RỖNG;
//! 3. tự kiểm mỗi `EXEMPT_FILES` khớp một tệp có thật trên đĩa.
//! Sàn quần thể `.rs` (Phần 0) và hai ca tự kiểm vị từ (bắt được/không bắt oan, cuối tệp) chạy
//! CHUNG cho cả ba mệnh đề.

use std::fs;
use std::path::{Path, PathBuf};

// ═════════════════════════════════════════════════════════════════════════════════
// Hạ tầng dùng chung — chép khuôn `library_index_boundary.rs`
// ═════════════════════════════════════════════════════════════════════════════════

fn src_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src")
}

/// Đường dẫn tương đối, dùng dấu `/` trên cả hai nền tảng — bắt buộc cho NFR14.
fn rel_posix(root: &Path, file: &Path) -> String {
    file.strip_prefix(root)
        .unwrap_or(file)
        .to_string_lossy()
        .replace('\\', "/")
}

fn walk(dir: &Path, out: &mut Vec<PathBuf>) {
    let entries = fs::read_dir(dir).unwrap_or_else(|e| panic!("đọc {}: {e}", dir.display()));
    for entry in entries {
        let entry = entry.unwrap_or_else(|e| panic!("duyệt {}: {e}", dir.display()));
        let path = entry.path();
        let meta = fs::symlink_metadata(&path)
            .unwrap_or_else(|e| panic!("lstat {}: {e}", path.display()));

        // ⚠️ `symlink_metadata`, không `metadata` — một liên kết trỏ về thư mục cha làm đệ
        // quy không dừng, cùng bài học mọi tệp `*_boundary.rs` khác.
        if meta.file_type().is_symlink() {
            continue;
        }
        if meta.is_dir() {
            walk(&path, out);
        } else if path.extension().and_then(|e| e.to_str()) == Some("rs") {
            out.push(path);
        }
    }
}

fn all_rust_sources() -> (PathBuf, Vec<PathBuf>) {
    let root = src_root();
    let mut files = Vec::new();
    walk(&root, &mut files);
    files.sort();
    (root, files)
}

/// Số tệp `.rs` tối thiểu dưới `src-tauri/src/**` để phép quét là thật.
///
/// Số thật lúc dựng (Story 5.5, 2026-08-28): **57** (56 kế thừa từ
/// `library_index_boundary.rs`, cộng `commands/lifecycle.rs` đã tồn tại từ Story 5.4 — story
/// này không thêm tệp `.rs` mới). Sàn **46** (~80,7%), cùng khuôn tỷ lệ dư địa mà
/// `RS_FLOOR`/`RUST_FLOOR` của các tệp `*_boundary.rs` khác đang giữ.
const RS_FLOOR: usize = 46;

/// Vị từ THUẦN trên một DÒNG MÃ đã biết KHÔNG PHẢI comment — chỗ gọi thật (mỗi Phần 1 dưới
/// đây) lọc dòng `//` TRƯỚC khi gọi hàm này, đúng khuôn
/// `library_index_boundary.rs::line_names_the_library_index_store`.
fn line_names_needle<'a>(code: &str, needle: &'a str) -> Option<&'a str> {
    code.contains(needle).then_some(needle)
}

fn line_names_one_of<'a>(code: &str, needles: &[&'a str]) -> Option<&'a str> {
    needles.iter().find_map(|n| line_names_needle(code, n))
}

/// Cắt bỏ một chú thích CUỐI DÒNG (`code // ghi chú`) khỏi phần MÃ THẬT của dòng — bổ sung
/// cho luật "dòng bắt đầu bằng `//` không phải vi phạm" mà `library_index_boundary.rs` đã có:
/// tệp này còn gặp dạng `continue; // meta.json hỏng ...`, nơi `//` đứng GIỮA dòng, không ở
/// đầu.
///
/// 🔵 **SỬA (2026-08-28, vòng rà thứ hai) — mệnh đề "không tệp nào ở đây có `//` bên trong một
/// chuỗi" đã SAI, không phải "hết đúng theo thời gian" mà sai NGAY TỪ ĐẦU.** Đếm lại cùng ngày:
/// **5** dòng trong `src-tauri/src` mang một hằng `"aura://…"` — ví dụ
/// `commands/project.rs:388`:
/// `pub const GLOSSARY_IMPORT_SCAN_EVENT: &str = "aura://glossary-import-scan-completed";`
/// (bốn dòng còn lại: `lib.rs:160,164,247,250`, cùng họ hằng sự kiện `aura://…`). Bản CŨ của
/// hàm này cắt tại dấu `//` ĐẦU TIÊN trên dòng bất kể nó nằm trong hay ngoài một chuỗi ký tự —
/// nên bất kỳ needle nào (kể cả `write_atomic(`) đứng SAU một hằng `aura://…` trên CÙNG một
/// dòng sẽ bị cắt khỏi tầm quét một cách IM LẶNG. Đây là cổng cưỡng chế AC4, nên một âm tính
/// giả ở đúng chỗ này phá chính mệnh đề cổng tồn tại để bảo đảm.
///
/// ⇒ Hàm nay là một MÁY TRẠNG THÁI nhỏ, duyệt theo **char** (không phải byte — cắt giữa một
/// ký tự UTF-8 nhiều byte sẽ panic ở lượt lập chỉ mục chuỗi): theo dõi có đang NẰM TRONG một
/// chuỗi `"..."` hay không; trong chuỗi, bỏ qua ký tự ngay sau `\` (escape, kể cả `\"` — dấu
/// nháy kép thoát không được phép đóng chuỗi); ngoài chuỗi, `//` mới được nhận là mở đầu một
/// chú thích. Vẫn KHÔNG phải một trình phân tích cú pháp Rust đầy đủ (không xử lý chuỗi thô
/// `r"..."`, không xử lý ký tự `'"'`) — đủ cho quần thể THỰC TẾ của `src-tauri/src/**` hôm
/// nay, không hơn.
fn code_without_trailing_comment(code: &str) -> &str {
    let mut in_string = false;
    let mut chars = code.char_indices().peekable();
    while let Some((idx, ch)) = chars.next() {
        if in_string {
            if ch == '\\' {
                // Escape trong chuỗi -- bỏ qua ký tự KẾ TIẾP dù nó là gì (kể cả `\"`, thứ
                // KHÔNG được phép đóng chuỗi). `chars.next()` tự động không panic ở cuối
                // dòng: `None` chỉ dừng vòng lặp, không index quá cuối.
                chars.next();
                continue;
            }
            if ch == '"' {
                in_string = false;
            }
            continue;
        }
        if ch == '"' {
            in_string = true;
            continue;
        }
        if ch == '/' {
            if let Some(&(_, next_ch)) = chars.peek() {
                if next_ch == '/' {
                    return code[..idx].trim_end();
                }
            }
        }
    }
    code
}

/// Áp ĐÚNG chuỗi lọc mà CẢ HAI chỗ gọi thật (`violations_outside`/`file_names_one_of_in_code`)
/// dùng — tách ra một hàm riêng để ca tự kiểm ở cuối tệp gọi được chính đường THẬT, không một
/// bản chép tay của nó: cắt khoảng trắng đầu dòng; dòng COMMENT TOÀN PHẦN (bắt đầu bằng `//`)
/// trả `None` — không có "mã" nào để kiểm; mọi dòng khác trả `Some(code)` sau khi cắt một chú
/// thích CUỐI DÒNG nếu có (`code_without_trailing_comment`).
fn code_position_of(line: &str) -> Option<&str> {
    let trimmed = line.trim_start();
    if trimmed.starts_with("//") {
        return None;
    }
    Some(code_without_trailing_comment(trimmed))
}

/// Mọi VI PHẠM: tệp NGOÀI `exempt` mà mang MỘT TRONG `needles` Ở VỊ TRÍ MÃ (không tính dòng
/// `//`), gộp theo `tệp:dòng  needle  |  mã`, đúng khuôn Phần 2 của `library_index_boundary.rs`.
fn violations_outside<'a>(
    root: &Path,
    files: &[PathBuf],
    exempt: &[&str],
    needles: &[&'a str],
) -> Vec<String> {
    let mut violations = Vec::new();
    for file in files {
        let rel = rel_posix(root, file);
        if exempt.contains(&rel.as_str()) {
            continue;
        }
        let text = fs::read_to_string(file).unwrap_or_else(|e| panic!("đọc {}: {e}", file.display()));
        for (index, line) in text.lines().enumerate() {
            let Some(code) = code_position_of(line) else { continue };
            if let Some(needle) = line_names_one_of(code, needles) {
                violations.push(format!("{rel}:{}  {needle}  |  {code}", index + 1));
            }
        }
    }
    violations
}

/// `true` ⇔ `file` (tương đối, chuẩn hoá `/`) nhắc MỘT TRONG `needles` Ở VỊ TRÍ MÃ.
fn file_names_one_of_in_code(file: &Path, needles: &[&str]) -> bool {
    let text = fs::read_to_string(file).unwrap_or_default();
    text.lines()
        .filter_map(code_position_of)
        .any(|code| line_names_one_of(code, needles).is_some())
}

// ═════════════════════════════════════════════════════════════════════════════════
// Phần 0 — sàn quần thể. Chạy trước mọi phép kiểm khác.
// ═════════════════════════════════════════════════════════════════════════════════

#[test]
fn the_scanned_tree_is_large_enough_to_be_real() {
    let (_, files) = all_rust_sources();
    assert!(
        files.len() >= RS_FLOOR,
        "chỉ tìm thấy {} tệp `.rs` dưới `src-tauri/src/**` (sàn {RS_FLOOR}). Cây quá nhỏ để \
         là thật — một danh sách rỗng làm mọi phép kiểm dưới đây xanh mà không kiểm gì cả.",
        files.len()
    );
}

// ═════════════════════════════════════════════════════════════════════════════════
// Mệnh đề (a) — `write_atomic(` trên `WorkMeta` chỉ ở BA tệp đã đóng.
// ═════════════════════════════════════════════════════════════════════════════════

/// Ba tệp DUY NHẤT được phép nhắc `write_atomic(` — nơi hàm được khai, cộng hai chỗ gọi sản
/// phẩm mà §Code Map của story đã liệt kê là "danh mục đóng".
const WRITE_ATOMIC_EXEMPT: [&str; 3] =
    ["core/library/meta.rs", "commands/project.rs", "commands/lifecycle.rs"];

/// Cố ý KHÔNG kèm dấu `.` phía trước: needle khớp CẢ chữ ký hàm
/// (`pub fn write_atomic(&self, ...)`, chỗ hàm được KHAI) LẪN mọi lời gọi
/// (`meta.write_atomic(...)`) — đúng nghĩa "ba tệp DUY NHẤT được phép NHẮC TỚI cái tên này ở
/// vị trí mã" mà Task 5(a) của story mô tả, không chỉ ba tệp GỌI nó.
const WRITE_ATOMIC_NEEDLE: &str = "write_atomic(";

#[test]
fn only_the_three_closed_sites_may_name_write_atomic() {
    let (root, files) = all_rust_sources();
    let violations = violations_outside(&root, &files, &WRITE_ATOMIC_EXEMPT, &[WRITE_ATOMIC_NEEDLE]);
    assert!(
        violations.is_empty(),
        "{} chỗ ngoài danh mục đóng {:?} nhắc `write_atomic(`:\n{}\n\n\
         §Never của story 5.5: 'không thêm một chỗ gọi ghi meta.json thứ ba' — hai chỗ đang có \
         (`commands/project.rs`, `commands/lifecycle.rs`) là danh mục ĐÓNG.",
        violations.len(),
        WRITE_ATOMIC_EXEMPT,
        violations.join("\n")
    );
}

#[test]
fn commands_project_and_lifecycle_actually_call_write_atomic() {
    let root = src_root();
    for rel in ["commands/project.rs", "commands/lifecycle.rs"] {
        let file = root.join(rel);
        assert!(
            file_names_one_of_in_code(&file, &[WRITE_ATOMIC_NEEDLE]),
            "`{rel}` không còn gọi `write_atomic(` — miễn trừ ở mệnh đề (a) đang canh một tập \
             RỖNG, tức không còn kiểm gì cả."
        );
    }
}

#[test]
fn every_write_atomic_exemption_matches_a_real_file() {
    let root = src_root();
    for rel in WRITE_ATOMIC_EXEMPT {
        assert!(
            root.join(rel).is_file(),
            "miễn trừ {rel:?} của mệnh đề (a) không khớp một tệp thật — sửa `WRITE_ATOMIC_EXEMPT` \
             trước khi tin cổng chính là kín."
        );
    }
}

// ═════════════════════════════════════════════════════════════════════════════════
// Mệnh đề (b) — `META_FILE` và chuỗi `"meta.json"` ở vị trí mã chỉ ở `core/library/meta.rs`.
// ═════════════════════════════════════════════════════════════════════════════════

const META_FILE_EXEMPT: [&str; 1] = ["core/library/meta.rs"];

/// Hai hình dạng: hằng `META_FILE` (khớp bất kể tiền tố đường dẫn gọi nó — một chỗ gọi
/// `crate::core::library::meta::META_FILE` cũng khớp vì chuỗi con `META_FILE` có mặt) và
/// chuỗi ký tự `meta.json` viết thẳng (không kèm dấu ngoặc kép để bắt được cả bên trong một
/// `format!`/nội suy — hẹp hơn thì bỏ lọt `format!("{{}}/meta.json", ..)`).
const META_FILE_NEEDLES: [&str; 2] = ["META_FILE", "meta.json"];

#[test]
fn only_meta_rs_may_name_the_meta_file() {
    let (root, files) = all_rust_sources();
    let violations = violations_outside(&root, &files, &META_FILE_EXEMPT, &META_FILE_NEEDLES);
    assert!(
        violations.is_empty(),
        "{} chỗ ngoài `core/library/meta.rs` nhắc tên tệp `meta.json` (hằng `META_FILE` hoặc \
         chuỗi viết thẳng):\n{}\n\n\
         AC4 đòi ĐÚNG MỘT nơi biết tên tệp trên đĩa; chỗ khác phải đi qua \
         `WorkMeta::read`/`write_atomic`, không tự lắp một chuỗi `\"meta.json\"` song song.",
        violations.len(),
        violations.join("\n")
    );
}

#[test]
fn core_library_meta_actually_names_the_meta_file() {
    let file = src_root().join("core/library/meta.rs");
    for needle in META_FILE_NEEDLES {
        assert!(
            file_names_one_of_in_code(&file, &[needle]),
            "`core/library/meta.rs` không còn nhắc `{needle}` — miễn trừ ở mệnh đề (b) đang \
             canh một tập RỖNG, tức không còn kiểm gì cả."
        );
    }
}

#[test]
fn every_meta_file_exemption_matches_a_real_file() {
    let root = src_root();
    for rel in META_FILE_EXEMPT {
        assert!(
            root.join(rel).is_file(),
            "miễn trừ {rel:?} của mệnh đề (b) không khớp một tệp thật — sửa `META_FILE_EXEMPT` \
             trước khi tin cổng chính là kín."
        );
    }
}

// ═════════════════════════════════════════════════════════════════════════════════
// Mệnh đề (c) — `WorkMeta::read` chỉ ở `core/library/meta.rs` + `core/library/indexer.rs`.
// ═════════════════════════════════════════════════════════════════════════════════

// 🔵 SUA (2026-08-29, Story 5.7) — tu HAI len BA phan tu. `commands/project.rs::open_work`
// la cho doc `meta.json` THU BA, va no la mot chu MOI cua mon no ma Story 5.5 da ghi bang
// chu ("story nay khong dung mot duong mo lai .atproj"). Xem khoi doc-comment dau tep.
const WORK_META_READ_EXEMPT: [&str; 3] =
    ["core/library/meta.rs", "core/library/indexer.rs", "commands/project.rs"];
const WORK_META_READ_NEEDLE: &str = "WorkMeta::read";

#[test]
fn only_meta_rs_indexer_rs_and_project_rs_may_call_work_meta_read() {
    let (root, files) = all_rust_sources();
    let violations =
        violations_outside(&root, &files, &WORK_META_READ_EXEMPT, &[WORK_META_READ_NEEDLE]);
    assert!(
        violations.is_empty(),
        "{} chỗ ngoài danh mục đóng {:?} nhắc `WorkMeta::read`:\n{}\n\n\
         `Indexer::rebuild` đọc `meta.json` lúc quét, và `commands::project::open_work` đọc nó \
         lúc MỞ LẠI (Story 5.7) — một chỗ đọc thứ TƯ là một đường vào chưa được xét.",
        violations.len(),
        WORK_META_READ_EXEMPT,
        violations.join("\n")
    );
}

/// Đối chứng dương: `core/library/indexer.rs` **thật sự** dùng needle — không có ca này, phép
/// kiểm chính xanh y hệt trên một cây mà `Indexer` đã ngừng đọc `meta.json`.
///
/// ⚠️ `core/library/meta.rs` KHÔNG kiểm ở đây: định nghĩa (`impl WorkMeta { pub fn read(..) }`)
/// không tự nhắc chuỗi ĐỦ ĐỦ ĐIỀU KIỆN `WorkMeta::read` (nó chỉ viết `read`, không
/// `WorkMeta::read`, bên trong chính `impl` của nó) — miễn trừ nó là ĐƯỢC PHÉP im lặng, không
/// BẮT BUỘC phải khớp needle, khác hẳn `commands/project.rs`/`commands/lifecycle.rs` ở mệnh đề
/// (a) hay chính `core/library/meta.rs` ở mệnh đề (b).
#[test]
fn core_library_indexer_actually_calls_work_meta_read() {
    let file = src_root().join("core/library/indexer.rs");
    assert!(
        file_names_one_of_in_code(&file, &[WORK_META_READ_NEEDLE]),
        "`core/library/indexer.rs` không còn gọi `WorkMeta::read` — miễn trừ ở mệnh đề (c) đang \
         canh một tập RỖNG, tức không còn kiểm gì cả."
    );
}

/// **THÊM (2026-08-29, Story 5.7)** — đối chứng dương cho chỗ đọc thứ BA:
/// `commands/project.rs::open_work` thật sự gọi `WorkMeta::read`, không phải một miễn trừ
/// canh một tập RỖNG.
#[test]
fn commands_project_actually_calls_work_meta_read() {
    let file = src_root().join("commands/project.rs");
    assert!(
        file_names_one_of_in_code(&file, &[WORK_META_READ_NEEDLE]),
        "`commands/project.rs` không còn gọi `WorkMeta::read` — miễn trừ ở mệnh đề (c) đang \
         canh một tập RỖNG, tức không còn kiểm gì cả."
    );
}

#[test]
fn every_work_meta_read_exemption_matches_a_real_file() {
    let root = src_root();
    for rel in WORK_META_READ_EXEMPT {
        assert!(
            root.join(rel).is_file(),
            "miễn trừ {rel:?} của mệnh đề (c) không khớp một tệp thật — sửa \
             `WORK_META_READ_EXEMPT` trước khi tin cổng chính là kín."
        );
    }
}

// ═════════════════════════════════════════════════════════════════════════════════
// Tự kiểm vị từ — bắt được một dòng dựng tay, và không bắt oan (dùng CHUNG cho cả ba mệnh đề).
// ═════════════════════════════════════════════════════════════════════════════════

/// Đối chứng DƯƠNG: mọi hình dạng thật của cả ba mệnh đề đều bị vị từ bắt.
#[test]
fn a_hand_built_line_matching_each_shape_is_caught() {
    assert_eq!(
        line_names_needle("meta.write_atomic(&dir)?;", WRITE_ATOMIC_NEEDLE),
        Some(WRITE_ATOMIC_NEEDLE),
        "ca DUONG THAT: goi meta.write_atomic(...) phai bi bat"
    );
    assert_eq!(
        line_names_needle(
            "pub fn write_atomic(&self, dir: &Path) -> Result<(), MetaError> {",
            WRITE_ATOMIC_NEEDLE
        ),
        Some(WRITE_ATOMIC_NEEDLE),
        "ca DUONG THAT: chinh chu ky ham (noi no duoc khai) cung phai bi bat -- needle cua (a) \
         rong hon mot loi goi phuong thuc"
    );
    assert_eq!(
        line_names_one_of("let target = dir.join(META_FILE);", &META_FILE_NEEDLES),
        Some("META_FILE"),
        "ca DUONG THAT: nhac hang META_FILE phai bi bat"
    );
    assert_eq!(
        line_names_one_of("fs::write(dir.join(\"meta.json\"), bytes)?;", &META_FILE_NEEDLES),
        Some("meta.json"),
        "ca DUONG THAT: chuoi \"meta.json\" viet thang phai bi bat"
    );
    assert_eq!(
        line_names_needle("let m = WorkMeta::read(&dir)?;", WORK_META_READ_NEEDLE),
        Some(WORK_META_READ_NEEDLE),
        "ca DUONG THAT: goi WorkMeta::read(...) phai bi bat"
    );
}

/// Đối chứng ÂM: các dòng gần giống nhưng KHÔNG khớp needle không bị bắt oan — một hàm ghi
/// KHÁC dùng chung tiền tố `write_` (`file.write_all(...)`, chính dòng có thật ở
/// `core/library/meta.rs::write_atomic`) không bị nhầm với `write_atomic(`, tên tệp khác không
/// bị nhầm với `meta.json`, và `WorkMeta::rebuild_from_store` (khác `WorkMeta::read`) không bị
/// nhầm lẫn.
#[test]
fn a_hand_built_line_naming_something_else_is_not_caught() {
    assert_eq!(
        line_names_needle("file.write_all(json.as_bytes())?;", WRITE_ATOMIC_NEEDLE),
        None,
        "ca AM: write_all la HAM KHAC write_atomic (chi chung tien to `write_`), khong duoc bat oan"
    );
    assert_eq!(
        line_names_one_of("let target = dir.join(\"project.db\");", &META_FILE_NEEDLES),
        None,
        "ca AM: \"project.db\" la ten tep KHAC, khong duoc bat oan"
    );
    assert_eq!(
        line_names_needle("let meta = WorkMeta::rebuild_from_store(&store)?;", WORK_META_READ_NEEDLE),
        None,
        "ca AM: WorkMeta::rebuild_from_store la HAM KHAC WorkMeta::read, khong duoc bat oan"
    );
}

/// ⚠️ Vị từ này THUẦN trên một dòng đã biết KHÔNG PHẢI comment — nó KHÔNG tự lọc `//`. Cổng
/// thật lọc dòng đó TRƯỚC khi gọi (`violations_outside`/`file_names_one_of_in_code`), đúng
/// khuôn `library_index_boundary.rs`. Ghi lại hợp đồng đó bằng chữ: gọi thẳng vị từ trên một
/// chuỗi comment vẫn trả `Some` — trách nhiệm lọc comment nằm ở CHỖ GỌI.
#[test]
fn the_predicate_itself_does_not_filter_comment_lines_the_caller_does() {
    assert_eq!(
        line_names_needle("// xem WorkMeta::read ở trên", WORK_META_READ_NEEDLE),
        Some(WORK_META_READ_NEEDLE),
        "vi tu KHONG tu loc comment -- do la trach nhiem cua ham violations_outside/\
         file_names_one_of_in_code, da loc rieng bang `code.starts_with(\"//\")` truoc khi goi ham nay"
    );
}

/// **THÊM (2026-08-28, vòng rà thứ hai) — neo đúng hình dạng của lỗi VÁ 1.** Một lời gọi
/// `write_atomic(` đứng CÙNG DÒNG, SAU một hằng chuỗi `"aura://…"` (hình dạng có thật ở
/// `commands/project.rs:388`/`lib.rs:160,164,247,250`, xem doc-comment của
/// `code_without_trailing_comment`) PHẢI vẫn bị bắt — bản CŨ của hàm đó cắt tại `//` bên
/// TRONG chuỗi, nuốt mất `write_atomic(` đứng sau. Đối chứng ÂM đứng cạnh: một dòng COMMENT
/// THẬT SỰ mang đúng hình dạng đó (bắt đầu bằng `//`) không được để lại "mã" nào để kiểm — nó
/// bị lọc bởi `code_position_of` TRƯỚC khi chạm `code_without_trailing_comment`, không phải vì
/// bộ cắt chú thích cuối dòng "khôn" tới mức tự nhận ra cả dòng là comment.
#[test]
fn a_write_atomic_call_sharing_a_line_with_an_aura_uri_constant_is_still_caught() {
    let call_line = "let _ = emit(\"aura://x\", m.write_atomic(d));";
    let code = code_position_of(call_line)
        .expect("day la mot dong MA THAT (khong bat dau bang //) -- phai co code de kiem");
    assert_eq!(
        line_names_one_of(code, &[WRITE_ATOMIC_NEEDLE]),
        Some(WRITE_ATOMIC_NEEDLE),
        "mot loi goi write_atomic( dung dong voi mot hang \"aura://...\" PHAI bi bat -- truoc \
         luot va, `//` trong chuoi lam ham cat nham va nuot mat `write_atomic(` dung sau"
    );

    let comment_line = "// aura://x write_atomic(";
    assert_eq!(
        code_position_of(comment_line),
        None,
        "mot dong COMMENT THAT (bat dau bang //) khong duoc co 'code' de kiem -- no bi loc o \
         buoc dau (code_position_of), khong lien quan gi toi viec cat chu thich cuoi dong"
    );
}
