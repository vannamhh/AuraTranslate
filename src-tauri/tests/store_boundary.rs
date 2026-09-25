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
//! ⚠️ Phạm vi quét là `src-tauri/src/**`, **không** gồm `tests/**`. Miễn trừ này **có
//! tên và có lý do**, đúng khuôn `EXEMPT` của `check-i18n.mjs`: ba ca của AC6/AC7 trong
//! `store_contract.rs` cần dựng một database ở một phiên bản lược đồ và một chế độ
//! journal cho trước — tức đúng thứ `core::store` tồn tại để mã sản phẩm không làm được.
//! Đường thay thế duy nhất là thêm một hàm `pub` vào mã sản phẩm mà chỉ test gọi.

use std::fs;
use std::path::{Path, PathBuf};

#[path = "support/boundary_scan.rs"]
#[allow(dead_code)] // shared module: not every helper is used in this file
mod boundary_scan;
use boundary_scan::is_inside;

/// Thư mục DUY NHẤT được phép nhắc tới `rusqlite`.
const STORE_DIR: &str = "core/store";

/// Số tệp `.rs` tối thiểu dưới `src-tauri/src/**` để phép quét là thật.
///
/// Số thật lúc dựng (Story 1.7): **22** tệp — 17 kế thừa + 5 tệp của `core/store/`. Sàn
/// đặt **dưới** số thật đúng khuôn `RS_FLOOR`/`VUE_FLOOR` của `check-i18n.mjs`: nó bắt
/// một cây bị cắt mất, không bắt việc thêm tệp mới.
///
/// ⚠️ Story 1.8: số thật là **26** — thêm 3 tệp `core/scope/` và `commands/config.rs`. Sàn
/// lên **20** (~77%).
///
/// ⚠️ Story 2.1 (2026-08-12): số thật là **42** — cây đã đi xa khỏi 26, và một sàn 20 trên
/// 42 tệp (47,6%) không còn canh được *"cây bị cắt"* nữa: mất hơn nửa cây vẫn xanh. Sàn lên
/// **34** (81,0%), cùng tỷ lệ dư địa mà `RS_FLOOR` của `check-i18n.mjs` đang giữ. Hai tệp
/// mới của story này là `core/segment/split.rs` và `commands/segment.rs`.
///
/// 🔴 **Quần thể này KHÁC quần thể của `check-i18n.mjs`** — ở đây là `src-tauri/src/**`
/// (26 tệp), ở đó là `src-tauri/**` sau miễn trừ `tests/**` (27 tệp, gồm `build.rs`). Hai
/// con số gần nhau và chúng **không** thay thế nhau được; chép số của tệp kia sang đây là
/// đặt một cái sàn cho một cây khác.
const RS_FLOOR: usize = 43; // 🔵 NÂNG 2026-08-24 (Story 3.7) — số THẬT: 53 tệp `.rs` dưới
// `src-tauri/src/**` (+`core/glossary/han_viet_suggestion.rs`) — 43/53 = 81,1%. Sàn cũ (34,
// đặt 2026-08-12) đã trôi xuống 34/53 = 64,2% qua nhiều story không ai nâng lại.

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
    boundary_scan::src_root()
}

fn rel_posix(root: &Path, file: &Path) -> String {
    boundary_scan::rel_posix(root, file)
}

fn all_rust_sources() -> (PathBuf, Vec<PathBuf>) {
    let root = src_root();
    let files = boundary_scan::rust_sources(&root)
        .into_iter()
        .map(|(rel, _)| root.join(rel))
        .collect();
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
        if is_inside(&rel, STORE_DIR) {
            store_files += 1;
            continue;
        }

        let text = fs::read_to_string(file)
            .unwrap_or_else(|e| panic!("đọc {}: {e}", file.display()));

        for (index, code) in boundary_scan::code_lines(&text) {
            for needle in FORBIDDEN {
                if code.contains(needle) {
                    violations.push(format!("{rel}:{index}  {needle}  |  {code}"));
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
        .filter(|f| is_inside(&rel_posix(&root, f), STORE_DIR))
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

/// `core::store` **không** `use tauri::…` — Quyết định #1.
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
        if !is_inside(&rel, STORE_DIR) {
            continue;
        }
        let text = fs::read_to_string(file)
            .unwrap_or_else(|e| panic!("đọc {}: {e}", file.display()));
        for (index, code) in boundary_scan::code_lines(&text) {
            if code.contains("use tauri") || code.contains("tauri::") {
                violations.push(format!("{rel}:{index}  {code}"));
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
