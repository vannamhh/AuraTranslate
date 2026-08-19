//! Ranh giới cây nguồn của Story 3.1 — `glossary_entry` chỉ tồn tại dưới `core/glossary/**`
//! và `core/store/schema.rs` (nơi hằng `GLOSSARY_ENTRY_DDL` sống).
//!
//! ⚠️ Tệp riêng có chủ ý, đúng khuôn `scope_boundary.rs`/`store_boundary.rs`: đây là phép
//! kiểm **tĩnh trên cây nguồn**, và `glossary_contract.rs` nghiệm thu **hành vi lúc chạy**
//! — trộn hai thứ làm hỏng đúng thứ khiến cả hai đọc được.
//!
//! ─────────────────────────────────────────────────────────────────────────────
//! VÌ SAO CỔNG NÀY TỒN TẠI TRƯỚC KHI EPIC 4 CÓ MỘT DÒNG MÃ
//! ─────────────────────────────────────────────────────────────────────────────
//! `deferred-work.md` (Story 3.1) đóng lại mệnh đề của Epic 3 context: *"Epic 4
//! (`RagInjector`) phụ thuộc trực tiếp vào truy vấn 'mục đủ điều kiện chèn' mà Story 3.1
//! dựng — không có đường nào khác để chạm dữ liệu Glossary."* Cổng này cưỡng chế đúng vế
//! *"không có đường nào khác"*: một module Epic 4 viết thẳng
//! `"SELECT … FROM glossary_entry …"` để lách qua `entries_eligible_for_injection` vẫn
//! biên dịch sạch — cổng dưới đây là phép kiểm duy nhất bắt được nó, và nó phải đứng sẵn
//! **trước** khi Epic 4 tồn tại, đúng lý do `scope_boundary.rs` đứng sẵn trước Epic 3.
//!
//! ─────────────────────────────────────────────────────────────────────────────
//! 🔴 SÀN SỐ TỆP + ĐỐI CHỨNG DƯƠNG LÀ BẮT BUỘC
//! ─────────────────────────────────────────────────────────────────────────────
//! *"Cây rỗng đọc thành sạch"* — bài học kế thừa từ `scope_boundary.rs`/`store_boundary.rs`.
//! Sàn bắt một cây bị **cắt mất**; đối chứng dương bắt ca *"không ai vi phạm"* và *"không
//! có gì để vi phạm"* đọc giống nhau.

use std::fs;
use std::path::{Path, PathBuf};

/// Thư mục DUY NHẤT được phép mang tên bảng `glossary_entry` bằng mã sản phẩm SQL
/// (`insert`/`select`/`update` thật).
const GLOSSARY_DIR: &str = "core/glossary";

/// Tệp DUY NHẤT khác `GLOSSARY_DIR` được phép — nơi hằng `GLOSSARY_ENTRY_DDL` khai `CREATE
/// TABLE glossary_entry`, đúng khuôn mọi hằng DDL khác của kho (`schema.rs` sở hữu MỌI tên
/// bảng, không chỉ tên này).
const SCHEMA_FILE: &str = "core/store/schema.rs";

/// Số tệp `.rs` tối thiểu dưới `src-tauri/src/**` để phép quét là thật.
///
/// Số thật lúc dựng (Story 3.1, 2026-08-19): **47** tệp — cây đã đi xa khỏi 42 (Story 2.1)
/// qua các bước Epic 2 còn lại cộng hai tệp mới của module này
/// (`core/glossary/entry.rs` · `core/glossary/store.rs`). Sàn đặt **dưới** số thật đúng
/// khuôn `RS_FLOOR` của `scope_boundary.rs`/`store_boundary.rs`: nó bắt một cây bị cắt
/// mất, không bắt việc thêm tệp mới. Sàn **38** (~80,9%).
const RS_FLOOR: usize = 38; // số THẬT 2026-08-19: 47 tệp .rs -- 38/47 = 80,9%

/// Chuỗi bị cấm ngoài hai vị trí ở trên — **tên bảng thật**, chữ thường nguyên văn như nó
/// nằm trong SQL (`CREATE TABLE glossary_entry`, `FROM glossary_entry`, …).
///
/// ⚠️ Không phải `"Glossary"` (tên module/kiểu Rust, viết hoa) và không phải `"glossary"`
/// (khoá dây của `ScopeKind::Glossary`, `core/scope/kinds.rs:162`) — cả hai token đó xuất
/// hiện hợp lệ ở khắp nơi (`core/scope/**`, doc-comment, `deferred-work.md`). Chỉ CHUỖI
/// TÊN BẢNG mới là thứ cổng này canh: đó là chỗ duy nhất "chạm dữ liệu Glossary" có nghĩa
/// theo đúng câu mà `epic-3-context.md` dùng.
const FORBIDDEN: &str = "glossary_entry";

/// Vế THỨ HAI của AD-36 — *"`ai/` không có đường nào khác chạm dữ liệu Glossary"*.
///
/// `core::glossary::mod` tái xuất công khai bốn hàm: `insert_entry` · `confirm_translation`
/// · `load_tier` · `entries_eligible_for_injection`. Ba cái đầu phơi dữ liệu THÔ —
/// `load_tier` trả **cả** mục *chờ chốt*, không lọc gì cả — nên một module khác gọi thẳng
/// chúng để tự lọc lấy sẽ tự cài lại (hoặc cài SAI) đúng luật mà
/// `entries_eligible_for_injection` đã đóng gói. Ba tên này chỉ được PHÉP xuất hiện dưới
/// `core/glossary/**`, nơi chúng được `pub fn` khai và tái xuất.
///
/// 🔵 **CẬP NHẬT 2026-08-19 (vá cuối) — `entries_eligible_for_injection` nay nhận `&Store`
/// và tự gọi `load_tier` bên trong** (`store.rs`), nên chỗ gọi ngoài module KHÔNG còn cần
/// tự `load_tier` gì cả — nó chỉ đưa `&Store` đã mở. Câu *"chỗ gọi ngoài tự nạp hai tầng
/// bằng `load_tier` rồi truyền kết quả vào `entries_eligible_for_injection`"* mà bản trước
/// của đoạn này viết đã **hết đúng**: đó chính là hình dạng mà cổng dưới đây phát hiện là
/// bất khả thi (đường DUY NHẤT dựng tham số cho hàm phơi ra DUY NHẤT lại bị chính cổng đó
/// cấm) — Ice ký sửa chữ ký thay vì nới cổng. Ba tên vẫn chỉ được PHÉP xuất hiện dưới
/// `core/glossary/**`; không có ca hợp lệ nào gọi chúng từ bên ngoài.
const GLOSSARY_ONLY_SURFACE: [&str; 3] = ["insert_entry", "confirm_translation", "load_tier"];

fn src_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src")
}

/// Đường dẫn tương đối, dùng dấu `/` trên cả hai nền tảng.
///
/// ⚠️ Chuẩn hoá `\` thành `/` là bắt buộc chứ không phải làm đẹp — cùng lý do
/// `scope_boundary.rs::rel_posix`: `starts_with(GLOSSARY_DIR)` trên Windows so với
/// `core\glossary` và **không bao giờ khớp**, nên miễn trừ biến mất và mọi tệp của chính
/// `core::glossary` bị báo vi phạm — một test đỏ **chỉ trên một nhánh** của ma trận, đúng
/// lớp lỗi NFR14.
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

        // ⚠️ `symlink_metadata`, không `metadata` — cùng lý do `scope_boundary.rs::walk`:
        // `metadata` giải symlink, nên một liên kết trỏ về thư mục cha làm đệ quy không dừng.
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
/// ⚠️ Chỉ dòng bắt đầu bằng `//` được bỏ qua, đúng luật `scope_boundary.rs::code_lines` áp
/// và `check-i18n.mjs` Kiểm A áp — Comment đuôi dòng vẫn bị bắt vì phần mã vẫn ở đầu dòng.
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

/// 🔴 Vế chính — chỉ `core/glossary/**` và `core/store/schema.rs` được mang chuỗi
/// `glossary_entry`.
#[test]
fn only_glossary_and_schema_may_name_the_glossary_entry_table() {
    let (root, files) = all_rust_sources();

    let mut violations: Vec<String> = Vec::new();
    let mut allowed_files = 0usize;

    for file in &files {
        let rel = rel_posix(&root, file);
        if rel.starts_with(GLOSSARY_DIR) || rel == SCHEMA_FILE {
            allowed_files += 1;
            continue;
        }

        for (line, code) in code_lines(file) {
            if code.contains(FORBIDDEN) {
                violations.push(format!("{rel}:{line}  {code}"));
            }
        }
    }

    // Miễn trừ phải khớp thứ gì đó — cùng đối chứng mà `scope_boundary.rs` đòi: một
    // `GLOSSARY_DIR`/`SCHEMA_FILE` gõ sai làm nhánh `continue` không bao giờ chạy, phép
    // kiểm vẫn xanh hôm nay (mã ngoài hai vị trí đó sạch thật), rồi đỏ sai chỗ vào ngày ai
    // đó đổi tên tệp/thư mục.
    assert!(
        allowed_files > 0,
        "không tệp nào khớp `{GLOSSARY_DIR}` hay `{SCHEMA_FILE}` — đường dẫn miễn trừ đã \
         lệch khỏi cây nguồn"
    );

    assert!(
        violations.is_empty(),
        "{} chỗ ngoài `core/glossary/**` và `{SCHEMA_FILE}` mang chuỗi `glossary_entry`:\n{}\n\n\
         `epic-3-context.md`: 'Epic 4 (RagInjector) phụ thuộc trực tiếp vào truy vấn mục đủ \
         điều kiện chèn mà Story 3.1 dựng — không có đường nào khác để chạm dữ liệu \
         Glossary.' Một module khác gõ tên bảng thẳng là đúng đường tắt đó. Cần dữ liệu \
         Glossary thì gọi `core::glossary::entries_eligible_for_injection` (hoặc \
         `load_tier`/`insert_entry`/`confirm_translation` cho phần còn lại của vòng đời).",
        violations.len(),
        violations.join("\n")
    );
}

/// Đối chứng dương: `core::glossary` **có thật sự** mang chuỗi `glossary_entry` — không có
/// ca này thì phép kiểm trên xanh y hệt trên một cây mà toàn bộ `core/glossary/` đã bị xoá.
#[test]
fn core_glossary_actually_names_its_own_table() {
    let (root, files) = all_rust_sources();

    let hits = files
        .iter()
        .filter(|f| rel_posix(&root, f).starts_with(GLOSSARY_DIR))
        .filter(|f| {
            fs::read_to_string(f)
                .map(|t| t.contains(FORBIDDEN))
                .unwrap_or(false)
        })
        .count();

    assert!(
        hits >= 1,
        "0 tệp dưới `{GLOSSARY_DIR}` nhắc tới `glossary_entry`. Module gồm `mod` · `entry` \
         (kiểu thuần, không SQL) · `store` (SQL) — `store.rs` phải gõ tên bảng để đọc/ghi \
         nó; 0 nghĩa là cây đã bị cắt và phép kiểm ranh giới đang canh một chỗ trống."
    );
}

/// Đối chứng dương thứ hai: `core::store::schema` cũng thật sự mang chuỗi đó — hằng
/// `GLOSSARY_ENTRY_DDL` sống ở đây, đúng như [`SCHEMA_FILE`] khai.
#[test]
fn schema_rs_actually_declares_the_glossary_entry_table() {
    let (root, files) = all_rust_sources();

    let hit = files.iter().any(|f| {
        rel_posix(&root, f) == SCHEMA_FILE
            && fs::read_to_string(f)
                .map(|t| t.contains(FORBIDDEN))
                .unwrap_or(false)
    });

    assert!(
        hit,
        "`{SCHEMA_FILE}` không nhắc tới `glossary_entry` — hằng `GLOSSARY_ENTRY_DDL` (bước 4 \
         của `GLOBAL_MIGRATIONS`, bước 12 của `PROJECT_MIGRATIONS`) phải khai `CREATE TABLE \
         glossary_entry` ở đúng tệp này."
    );
}
// ═════════════════════════════════════════════════════════════════════════════════
// Vế thứ hai của AD-36 — chỉ `entries_eligible_for_injection` gọi được từ module khác
// ═════════════════════════════════════════════════════════════════════════════════

/// 🔴 `insert_entry`/`confirm_translation`/`load_tier` chỉ được GỌI (và được KHAI) dưới
/// `core/glossary/**`. Một module Epic 4 gọi `core::glossary::load_tier(global)` thẳng để
/// tự lọc "đã chốt" biên dịch sạch và qua cả mười một cổng hôm nay — cổng này là phép kiểm
/// DUY NHẤT bắt được nó.
///
/// ⚠️ Trước ca này, ba tên đó chỉ xuất hiện trong một câu thông báo `assert!` của
/// `only_glossary_and_schema_may_name_the_glossary_entry_table` — tức được NHẮC TỚI, không
/// được CANH.
#[test]
fn only_entries_eligible_for_injection_may_be_called_from_outside_glossary() {
    let (root, files) = all_rust_sources();

    let mut violations: Vec<String> = Vec::new();

    for file in &files {
        let rel = rel_posix(&root, file);
        if rel.starts_with(GLOSSARY_DIR) {
            continue;
        }

        for (line, code) in code_lines(file) {
            for needle in GLOSSARY_ONLY_SURFACE {
                if code.contains(needle) {
                    violations.push(format!("{rel}:{line}  {needle}  |  {code}"));
                }
            }
        }
    }

    assert!(
        violations.is_empty(),
        "{} chỗ ngoài `{GLOSSARY_DIR}` gọi thẳng một hàm phơi dữ liệu THÔ của Glossary:
{}

         `load_tier` trả CẢ mục chờ chốt; `insert_entry`/`confirm_translation` ghi thẳng          không qua điều kiện chèn. Module khác chỉ được gọi          `core::glossary::entries_eligible_for_injection` — đúng MỘT hàm phơi ra, theo AD-36.",
        violations.len(),
        violations.join("\n")
    );
}

/// Đối chứng dương: `core::glossary` **có thật sự** khai cả ba tên bị hạn chế — không có ca
/// này thì phép kiểm trên xanh y hệt trên một cây mà `store.rs` đã bị xoá.
#[test]
fn core_glossary_actually_defines_the_restricted_surface() {
    let (root, files) = all_rust_sources();

    for needle in GLOSSARY_ONLY_SURFACE {
        let hit = files
            .iter()
            .filter(|f| rel_posix(&root, f).starts_with(GLOSSARY_DIR))
            .any(|f| {
                fs::read_to_string(f)
                    .map(|t| t.contains(needle))
                    .unwrap_or(false)
            });

        assert!(
            hit,
            "không tệp nào dưới `{GLOSSARY_DIR}` khai `{needle}` — cây đã bị cắt và phép              kiểm ranh giới đang canh một chỗ trống."
        );
    }
}
