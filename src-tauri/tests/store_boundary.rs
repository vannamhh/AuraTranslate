//! Ranh giới cây nguồn của AC2 — `rusqlite` chỉ được xuất hiện dưới `src/core/store/**`.
//!
//! ⚠️ Tệp riêng có chủ ý. `store_contract.rs` khai phạm vi của nó ở dòng 1 (*hành vi lúc
//! chạy*); đây là phép kiểm **tĩnh trên cây nguồn**, và trộn hai thứ là làm hỏng đúng
//! thứ khiến cả hai đọc được.
//!
//! ─────────────────────────────────────────────────────────────────────────────
//! VÌ SAO CẦN CẢ HAI VẾ CỦA AC2, KHÔNG PHẢI MỘT
//! ─────────────────────────────────────────────────────────────────────────────
//! AC2 nói *"cưỡng chế bằng test **hoặc** bằng khả năng hiển thị của kiểu"*, và story
//! chốt là **cả hai**, vì mỗi vế để hở đúng chỗ vế kia đóng:
//!
//! - **Kiểu** đóng đường *"lấy được kết nối ghi từ `Store`"*: `Connection` ghi bị `move`
//!   vào luồng writer và `Connection` không `Sync`, nên trình biên dịch giữ phần đó.
//!   Nhưng kiểu **không** ngăn được ai đó gõ `rusqlite::Connection::open(path)` ở một
//!   module hoàn toàn khác — mã đó biên dịch sạch và mở một kết nối ghi thứ hai vào đúng
//!   tệp đó, tức AD-11 bị phá mà không gì báo.
//! - **Test này** đóng đúng đường đó, và chỉ đường đó.
//!
//! ─────────────────────────────────────────────────────────────────────────────
//! 🔴 SÀN SỐ TỆP LÀ BẮT BUỘC
//! ─────────────────────────────────────────────────────────────────────────────
//! *"Cây rỗng đọc thành sạch"* — bài học thừa kế từ `check-deps.mjs:15-17` và
//! `check-i18n.mjs:211-234`. Một đường dẫn gõ sai làm `walk` khớp 0 tệp ⇒ vòng lặp dưới
//! đây xanh mà không kiểm gì cả ⇒ cổng chết im lặng ngay ngày nó ra đời.
//!
//! ⚠️ Phạm vi quét là `src-tauri/src/**`, ⛔ **không** gồm `tests/**`. Miễn trừ này **có
//! tên và có lý do**, đúng khuôn `EXEMPT` của `check-i18n.mjs`: ba ca của AC6/AC7 trong
//! `store_contract.rs` cần dựng một database ở một phiên bản lược đồ và một chế độ
//! journal cho trước — tức đúng thứ `core::store` tồn tại để mã sản phẩm không làm được.
//! Đường thay thế duy nhất là thêm một hàm `pub` vào mã sản phẩm mà chỉ test gọi.

use std::fs;
use std::path::{Path, PathBuf};

/// Thư mục DUY NHẤT được phép nhắc tới `rusqlite`.
const STORE_DIR: &str = "core/store";

/// Số tệp `.rs` tối thiểu dưới `src-tauri/src/**` để phép quét là thật.
///
/// Số thật lúc dựng (Story 1.7): **22** tệp — 17 kế thừa + 5 tệp của `core/store/`. Sàn
/// đặt **dưới** số thật đúng khuôn `RS_FLOOR`/`VUE_FLOOR` của `check-i18n.mjs`: nó bắt
/// một cây bị cắt mất, ⛔ không bắt việc thêm tệp mới.
const RS_FLOOR: usize = 18;

/// Những chuỗi mà **chỉ** `core::store` được mang.
///
/// `"rusqlite"` trần chứ không phải `"rusqlite::Connection"`, có chủ ý: bản hẹp bỏ lọt
/// `use rusqlite::{Connection, Result}` — một cách viết hoàn toàn bình thường mà một
/// phép so chuỗi con hẹp không thấy. Bản rộng cũng chặn luôn `rusqlite::Result` và
/// `rusqlite::params!` ở chỗ gọi, và đó là đúng ý: `core::store` **tái xuất** những kiểu
/// chỗ gọi thật sự cần (`Transaction`, `SqlError`, `SqlResult`, `Row`, `ReadHandle`),
/// nên không module nào có lý do chính đáng để gõ tên crate.
const FORBIDDEN: [&str; 2] = ["rusqlite", "Connection::open"];

fn src_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src")
}

/// Đường dẫn tương đối, dùng dấu `/` trên cả hai nền tảng.
///
/// ⚠️ Chuẩn hoá `\` thành `/` là bắt buộc chứ không phải làm đẹp: `starts_with(STORE_DIR)`
/// trên Windows so với `core\store` và **không bao giờ khớp**, nên miễn trừ biến mất và
/// mọi tệp của chính `core::store` bị báo vi phạm — một test đỏ **chỉ trên một nhánh**
/// của ma trận, đúng lớp lỗi NFR14.
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

        // ⚠️ `symlink_metadata`, không `metadata`: cùng bài học với `check-i18n.mjs:155-160`.
        // `metadata` giải symlink, nên một liên kết trỏ về thư mục cha làm đệ quy không
        // dừng. Symlink bị bỏ qua — cây hiện không có cái nào, và nếu có thì sàn tệp bên
        // dưới là thứ bắt được việc đó.
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

/// 🔴 AC2 vế test — không module nào ngoài `core::store` được nhắc tới `rusqlite`.
#[test]
fn only_core_store_may_name_rusqlite() {
    let (root, files) = all_rust_sources();

    let mut violations: Vec<String> = Vec::new();
    let mut store_files = 0usize;

    for file in &files {
        let rel = rel_posix(&root, file);
        if rel.starts_with(STORE_DIR) {
            store_files += 1;
            continue;
        }

        let text = fs::read_to_string(file)
            .unwrap_or_else(|e| panic!("đọc {}: {e}", file.display()));

        for (index, line) in text.lines().enumerate() {
            let code = line.trim_start();

            // ⚠️ Dòng comment **không** phải vi phạm — cùng luật mà `check-i18n.mjs` Kiểm A
            // áp cho chuỗi tiếng Việt, và vì cùng một lý do: `core/dict/mod.rs:6` ghi
            // *"crate dành cho module này: `rusqlite` — dùng chung cài đặt với
            // `core::store`"*, và đó là **tài liệu về một ranh giới**, không phải một lời
            // gọi vượt qua nó. Một cổng đỏ trên câu giải thích chính luật nó canh là một
            // cổng bị gỡ trong tuần.
            //
            // ⛔ Chỉ dòng bắt đầu bằng `//` được bỏ qua. Comment đuôi dòng
            // (`Connection::open(p); // ghi chú`) vẫn bị bắt, vì phần mã vẫn ở đầu dòng.
            if code.starts_with("//") {
                continue;
            }

            for needle in FORBIDDEN {
                if code.contains(needle) {
                    violations.push(format!("{rel}:{}  {needle}  |  {code}", index + 1));
                }
            }
        }
    }

    // Miễn trừ phải khớp thứ gì đó. Một `STORE_DIR` gõ sai làm nhánh `continue` không
    // bao giờ chạy — phép kiểm vẫn xanh hôm nay (mã sản phẩm ngoài store sạch thật),
    // rồi đỏ sai chỗ vào ngày ai đó đổi tên thư mục.
    assert!(
        store_files > 0,
        "không tệp nào khớp `{STORE_DIR}` — đường dẫn miễn trừ đã lệch khỏi cây nguồn"
    );

    assert!(
        violations.is_empty(),
        "{} chỗ ngoài `core::store` nhắc tới tầng SQLite:\n{}\n\n\
         AD-11: MỌI ghi đi qua `store::Writer` của kho tương ứng. Một \
         `Connection::open` ở một module khác mở một kết nối ghi THỨ HAI vào cùng tệp — \
         mã đó biên dịch sạch, chạy được, và phá đúng bất biến mà cả tầng này tồn tại để \
         giữ. Cần đọc/ghi thì gọi `Store::read` / `Store::write`; cần một kiểu của \
         `rusqlite` thì `core::store` đã tái xuất (`Transaction`, `SqlError`, `SqlResult`, \
         `Row`, `ReadHandle`).",
        violations.len(),
        violations.join("\n")
    );
}

/// Đối chứng dương: `core::store` **có thật sự** dùng `rusqlite`.
///
/// ⚠️ Không có ca này thì phép kiểm trên xanh y hệt trên một cây mà toàn bộ tầng dữ liệu
/// đã bị xoá — *"không ai vi phạm"* và *"không có gì để vi phạm"* đọc giống nhau.
#[test]
fn core_store_actually_uses_rusqlite() {
    let (root, files) = all_rust_sources();

    let hits = files
        .iter()
        .filter(|f| rel_posix(&root, f).starts_with(STORE_DIR))
        .filter(|f| {
            fs::read_to_string(f)
                .map(|t| t.contains("rusqlite"))
                .unwrap_or(false)
        })
        .count();

    assert!(
        hits >= 3,
        "chỉ {hits} tệp dưới `{STORE_DIR}` nhắc tới `rusqlite`. Tầng ghi dữ liệu gồm \
         writer · reader · pragmas · checkpoint · schema; ít hơn 3 nghĩa là cây đã bị cắt \
         và phép kiểm ranh giới đang canh một chỗ trống."
    );
}

/// `core::store` ⛔ **không** `use tauri::…` — Quyết định #1.
///
/// Vì sao thành một test chứ không một comment: mệnh đề này hỏng bằng **một dòng `use`**
/// mà mọi thứ khác vẫn xanh, và cái giá chỉ hiện ra ở Story 1.15 (`project.db` nằm trong
/// một `.atproj` do người dùng chọn, không phải `$APPDATA`) và ở mọi ca test phải dựng
/// một `AppHandle` để chạm vào tầng dữ liệu.
#[test]
fn core_store_does_not_depend_on_tauri() {
    let (root, files) = all_rust_sources();

    let mut violations = Vec::new();
    for file in &files {
        let rel = rel_posix(&root, file);
        if !rel.starts_with(STORE_DIR) {
            continue;
        }
        let text = fs::read_to_string(file)
            .unwrap_or_else(|e| panic!("đọc {}: {e}", file.display()));
        for (index, line) in text.lines().enumerate() {
            let code = line.trim_start();
            // Chỉ vị trí MÃ. Doc-comment của module nói về `app.path().app_data_dir()`
            // đúng như nó phải nói, và đó không phải một phụ thuộc.
            if code.starts_with("//") {
                continue;
            }
            if code.contains("use tauri") || code.contains("tauri::") {
                violations.push(format!("{rel}:{}  {}", index + 1, code));
            }
        }
    }

    assert!(
        violations.is_empty(),
        "`core::store` đã phụ thuộc `tauri`:\n{}\n\n\
         Quyết định #1: `Store::open` nhận một `StoreSpec` mang `PathBuf` đã phân giải; \
         đường lấy `$APPDATA` sống ở `lib.rs`. Mất mệnh đề này là mất khả năng test tầng \
         dữ liệu mà không cần webview.",
        violations.join("\n")
    );
}
