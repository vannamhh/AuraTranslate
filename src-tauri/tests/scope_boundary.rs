//! Ranh giới cây nguồn của AC1 — phân giải hai tầng chỉ tồn tại dưới `src/core/scope/**`.
//!
//! ⚠️ Tệp riêng có chủ ý, đúng khuôn `store_boundary.rs` / `store_contract.rs`:
//! `scope_contract.rs` nghiệm thu **hành vi lúc chạy**, đây là phép kiểm **tĩnh trên cây
//! nguồn**, và trộn hai thứ là làm hỏng đúng thứ khiến cả hai đọc được.
//!
//! ─────────────────────────────────────────────────────────────────────────────
//! 🔴 VÌ SAO CẦN CẢ HAI VẾ CỦA AC1, KHÔNG PHẢI MỘT
//! ─────────────────────────────────────────────────────────────────────────────
//! AC1 nói *"mọi phân giải hai tầng đi qua **đúng một** `ScopeResolver`"*, và story chốt
//! là **cả hai** cơ chế, vì mỗi vế để hở đúng chỗ vế kia đóng:
//!
//! - **Kiểu** đóng đường *"gọi sai hàm cho ngữ nghĩa của loại"*: ba hàm phân giải là
//!   `pub(crate)` và phơi ra chỉ qua `ScopeResolver`, và `require()` trả
//!   `ScopeError::WrongSemantics` chứ không im lặng làm theo ý người gọi. Nhưng kiểu
//!   **không** ngăn được ai đó viết `if work.is_some() { … } else { … }` bằng tay ở một
//!   module hoàn toàn khác — mã đó biên dịch sạch, chạy được, và cài lại AD-18 sai.
//! - **Test này** đóng đúng đường đó, và chỉ đường đó.
//!
//! ─────────────────────────────────────────────────────────────────────────────
//! ⚠️ HÔM NAY CHƯA CÓ CONSUMER NÀO — VÀ ĐÓ KHÔNG LÀM PHÉP KIỂM NÀY THÀNH VÒNG
//! ─────────────────────────────────────────────────────────────────────────────
//! Glossary (Epic 3) · TM (Epic 7) · Prompt và Cấu hình AI (Epic 4) · Luật làm sạch
//! (Epic 6) đều là module rỗng. Cổng dưới đây cấm **token**, không đo consumer — nên nó
//! đỏ ngay **lần đầu** một module Epic 3 gõ `ScopeKind` hay tự viết một nhánh
//! `if work.is_some()`. Đó chính là lượt đỏ mà phép kiểm này tồn tại để mua, và nó xảy ra
//! ở story sau chứ không phải hôm nay.
//!
//! ─────────────────────────────────────────────────────────────────────────────
//! 🔴 SÀN SỐ TỆP + ĐỐI CHỨNG DƯƠNG LÀ BẮT BUỘC
//! ─────────────────────────────────────────────────────────────────────────────
//! *"Cây rỗng đọc thành sạch"* — bài học thừa kế từ `check-deps.mjs:15-17`,
//! `check-i18n.mjs:211-234` và `store_boundary.rs`. Sàn bắt một cây bị **cắt mất**; đối
//! chứng dương bắt ca *"không ai vi phạm"* và *"không có gì để vi phạm"* đọc giống nhau.

use std::fs;
use std::path::{Path, PathBuf};

/// Thư mục DUY NHẤT được phép mang từ vựng phân giải hai tầng.
const SCOPE_DIR: &str = "core/scope";

/// Thư mục DUY NHẤT được phép nhắc tới tầng SQLite — dùng lại ở phép kiểm ngược chiều.
const STORE_DIR: &str = "core/store";

/// Số tệp `.rs` tối thiểu dưới `src-tauri/src/**` để phép quét là thật.
///
/// Số thật lúc dựng (Story 1.8): **26** tệp — 22 kế thừa + 3 tệp của `core/scope/` +
/// `commands/config.rs`. Sàn đặt **dưới** số thật đúng khuôn `RS_FLOOR` của
/// `store_boundary.rs`: nó bắt một cây bị cắt mất, ⛔ không bắt việc thêm tệp mới.
const RS_FLOOR: usize = 20;

/// 🔴 Vế test của AC1 — những chuỗi mà **chỉ** `core::scope` được mang.
///
/// Bốn token, và mỗi cái đóng một đường hỏng khác nhau:
/// - `"Semantics"` — bảng ngữ nghĩa AD-18. Một bản chép thứ hai ở module khác là hai bảng
///   sẽ trôi khỏi nhau, và chúng trôi im lặng.
/// - `"resolve_override"` / `"resolve_merge"` — tên hai hàm **nội bộ** của [`resolve`]
///   (`pub(crate)`). Một chỗ gọi ngoài crate không biên dịch được, nhưng **trong** crate
///   thì được, và đó chính là chỗ Epic 3 tới Epic 7 sẽ đứng nếu lách qua `ScopeResolver`.
///   ⚠️ Lượt review kiến trúc bắt được rằng nếu phương thức công khai của `ScopeResolver`
///   mang đúng hai tên này thì chính lời gọi ĐÚNG (`resolver.resolve_override(...)`) cũng
///   tự đỏ — nên `ScopeResolver` phơi ra dưới tên `apply_override` / `apply_merge`, khác
///   hẳn hai token bị cấm ở đây. Xem doc-comment của `ScopeResolver::apply_merge`.
/// - `"ScopeKind"` — danh mục loại. Gõ được nó nghĩa là đăng ký được một loại mới ở ngoài
///   bảng, tức đúng lỗ mà AC4 tồn tại để bịt.
const FORBIDDEN_OUTSIDE_SCOPE: [&str; 4] = [
    "Semantics",
    "resolve_override",
    "resolve_merge",
    "ScopeKind",
];

/// Cấm ngược chiều: `core::scope` ⛔ không được gõ tên tầng SQLite.
///
/// ⚠️ `store_boundary.rs` đã bao quần thể này *(nó quét cả `src-tauri/src/**`)*, và khẳng
/// định lại ở đây là **có chủ ý**: `core/scope/**` là quần thể **mới**, nó đọc/ghi thật
/// qua `Store`, và nó là chỗ đầu tiên trong dự án bị cám dỗ mở một kết nối riêng *"cho
/// nhanh"*. Một mệnh đề được canh ở hai chỗ vẫn rẻ hơn một mệnh đề được canh ở chỗ mà
/// người sửa `core/scope/` không đọc.
const FORBIDDEN_INSIDE_SCOPE: [&str; 2] = ["rusqlite", "Connection::open"];

fn src_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src")
}

/// Đường dẫn tương đối, dùng dấu `/` trên cả hai nền tảng.
///
/// ⚠️ Chuẩn hoá `\` thành `/` là bắt buộc chứ không phải làm đẹp: `starts_with(SCOPE_DIR)`
/// trên Windows so với `core\scope` và **không bao giờ khớp**, nên miễn trừ biến mất và
/// mọi tệp của chính `core::scope` bị báo vi phạm — một test đỏ **chỉ trên một nhánh** của
/// ma trận, đúng lớp lỗi NFR14.
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
        let meta =
            fs::symlink_metadata(&path).unwrap_or_else(|e| panic!("lstat {}: {e}", path.display()));

        // ⚠️ `symlink_metadata`, không `metadata`: `metadata` giải symlink, nên một liên
        // kết trỏ về thư mục cha làm đệ quy không dừng. Cùng bài học `check-i18n.mjs:155`.
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

/// Dòng **mã** của một tệp: `(số dòng 1-based, nội dung đã trim đầu)`.
///
/// ⚠️ Chỉ dòng bắt đầu bằng `//` được bỏ qua, đúng luật mà `store_boundary.rs:147` áp và
/// `check-i18n.mjs` Kiểm A áp: một doc-comment **giải thích** một ranh giới không phải một
/// lời gọi vượt qua nó, và một cổng đỏ trên câu giải thích chính luật nó canh là một cổng
/// bị gỡ trong tuần.
///
/// ⛔ **Comment đuôi dòng vẫn bị bắt** — phần mã vẫn ở đầu dòng. Story 1.7 đã ghi lại
/// nguyên văn điều này sau khi nó cắn một lần.
fn code_lines(file: &Path) -> Vec<(usize, String)> {
    let text =
        fs::read_to_string(file).unwrap_or_else(|e| panic!("đọc {}: {e}", file.display()));
    text.lines()
        .enumerate()
        .map(|(i, l)| (i + 1, l.trim_start().to_owned()))
        .filter(|(_, code)| !code.starts_with("//"))
        .collect()
}

/// Sàn quần thể — chạy trước mọi phép kiểm khác. Xem doc-comment đầu tệp.
#[test]
fn the_scanned_tree_is_large_enough_to_be_real() {
    let (_, files) = all_rust_sources();
    assert!(
        files.len() >= RS_FLOOR,
        "chỉ tìm thấy {} tệp `.rs` dưới `src-tauri/src/**` (sàn {RS_FLOOR}). \
         Cây quá nhỏ để là thật — một danh sách rỗng làm mọi phép kiểm dưới đây xanh mà \
         không kiểm gì cả. Nghi phạm: gốc quét sai, hoặc một thư mục bị bỏ.",
        files.len()
    );
}

/// 🔴 AC1 vế test — từ vựng phân giải hai tầng chỉ sống dưới `core::scope`.
#[test]
fn only_core_scope_may_name_the_two_tier_vocabulary() {
    let (root, files) = all_rust_sources();

    let mut violations: Vec<String> = Vec::new();
    let mut scope_files = 0usize;

    for file in &files {
        let rel = rel_posix(&root, file);
        if rel.starts_with(SCOPE_DIR) {
            scope_files += 1;
            continue;
        }

        for (line, code) in code_lines(file) {
            for needle in FORBIDDEN_OUTSIDE_SCOPE {
                if code.contains(needle) {
                    violations.push(format!("{rel}:{line}  {needle}  |  {code}"));
                }
            }
        }
    }

    // Miễn trừ phải khớp thứ gì đó. Một `SCOPE_DIR` gõ sai làm nhánh `continue` không bao
    // giờ chạy — phép kiểm vẫn xanh hôm nay (mã ngoài `core::scope` sạch thật), rồi đỏ sai
    // chỗ vào ngày ai đó đổi tên thư mục.
    assert!(
        scope_files > 0,
        "không tệp nào khớp `{SCOPE_DIR}` — đường dẫn miễn trừ đã lệch khỏi cây nguồn"
    );

    assert!(
        violations.is_empty(),
        "{} chỗ ngoài `core::scope` mang từ vựng phân giải hai tầng:\n{}\n\n\
         AD-18 + AC1: MỌI phân giải hai tầng đi qua ĐÚNG MỘT `ScopeResolver`. Một nhánh \
         `if work.is_some()` viết tay ở một module khác biên dịch sạch, chạy được, và cài \
         lại AD-18 sai — thường theo hướng *ghi đè cả tập* thay vì *ghi đè theo từng khoá*, \
         thứ làm 411 mục Glossary toàn cục biến mất mà không test cẩu thả nào bắt.\n\
         Cần phân giải thì gọi `ScopeResolver::apply_override` / `apply_merge` / \
         `resolve_global_only`; cần cấu hình khởi động thì gọi \
         `core::scope::load_global_config`.",
        violations.len(),
        violations.join("\n")
    );
}

/// Đối chứng dương: `core::scope` **có thật sự** mang từ vựng đó.
///
/// ⚠️ Không có ca này thì phép kiểm trên xanh y hệt trên một cây mà toàn bộ `core/scope/`
/// đã bị xoá — *"không ai vi phạm"* và *"không có gì để vi phạm"* đọc giống nhau.
#[test]
fn core_scope_actually_owns_the_two_tier_vocabulary() {
    let (root, files) = all_rust_sources();

    let hits = files
        .iter()
        .filter(|f| rel_posix(&root, f).starts_with(SCOPE_DIR))
        .filter(|f| {
            fs::read_to_string(f)
                .map(|t| t.contains("ScopeKind"))
                .unwrap_or(false)
        })
        .count();

    assert!(
        hits >= 3,
        "chỉ {hits} tệp dưới `{SCOPE_DIR}` nhắc tới `ScopeKind`. Module gồm `mod` · `kinds` \
         · `resolve` · `store`; ít hơn 3 nghĩa là cây đã bị cắt và phép kiểm ranh giới đang \
         canh một chỗ trống."
    );
}

/// Ngược chiều — `core::scope` ⛔ không gõ tên tầng SQLite.
///
/// Nó đọc và ghi thật, nhưng **qua `Store::read` / `Store::write`** và bằng các kiểu đã
/// **tái xuất** từ `core::store` (`Transaction` · `SqlError` · `SqlResult` · `Row` ·
/// `ReadHandle`, `store/mod.rs:98-117`). AD-11: một kết nối ghi thứ hai vào cùng tệp là
/// bất biến của cả tầng dữ liệu bị phá mà không gì báo.
#[test]
fn core_scope_never_names_the_sqlite_layer() {
    let (root, files) = all_rust_sources();

    let mut violations: Vec<String> = Vec::new();
    for file in &files {
        let rel = rel_posix(&root, file);
        if !rel.starts_with(SCOPE_DIR) {
            continue;
        }
        for (line, code) in code_lines(file) {
            for needle in FORBIDDEN_INSIDE_SCOPE {
                if code.contains(needle) {
                    violations.push(format!("{rel}:{line}  {needle}  |  {code}"));
                }
            }
        }
    }

    assert!(
        violations.is_empty(),
        "`core::scope` đã gõ tên tầng SQLite:\n{}\n\n\
         ⛔ Kể cả trong một comment ĐUÔI DÒNG — bộ quét chỉ miễn trừ dòng bắt đầu bằng \
         `//`, và Story 1.7 đã ghi lại nguyên văn điều đó sau khi nó cắn một lần. \
         `core::store` đã tái xuất mọi kiểu chỗ gọi thật sự cần; `{STORE_DIR}` là thư mục \
         duy nhất được phép nhắc tên crate.",
        violations.join("\n")
    );
}

/// `core::scope` ⛔ **không** `use tauri::…` — cùng khuôn `core_store_does_not_depend_on_tauri`.
///
/// Vì sao thành một test chứ không một comment: mệnh đề này hỏng bằng **một dòng `use`**
/// mà mọi thứ khác vẫn xanh, và cái giá chỉ hiện ra ở mọi ca test phải dựng một `AppHandle`
/// để chạm vào bộ phân giải — tức đúng khác biệt giữa `scope_contract.rs` chạy trong
/// `cargo test` và một bảng nghiệm thu tay.
///
/// Đường lấy `AppHandle`/`State` sống ở `commands/config.rs`, và nó là một **vỏ mỏng**.
#[test]
fn core_scope_does_not_depend_on_tauri() {
    let (root, files) = all_rust_sources();

    let mut violations = Vec::new();
    for file in &files {
        let rel = rel_posix(&root, file);
        if !rel.starts_with(SCOPE_DIR) {
            continue;
        }
        for (line, code) in code_lines(file) {
            if code.contains("use tauri") || code.contains("tauri::") {
                violations.push(format!("{rel}:{line}  {code}"));
            }
        }
    }

    assert!(
        violations.is_empty(),
        "`core::scope` đã phụ thuộc `tauri`:\n{}\n\n\
         Quyết định #1 của Story 1.7, áp nguyên vẹn ở đây: bộ phân giải nhận dữ liệu đã \
         nạp và một `&Store` đã mở; đường lấy `State` sống ở `commands/`. Mất mệnh đề này \
         là mất khả năng test phân giải hai tầng mà không cần webview.",
        violations.join("\n")
    );
}
