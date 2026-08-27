//! Ranh giới cây nguồn của AC2 (Story 5.2) — chỉ `core/library/indexer.rs` (chỗ gọi) và
//! `core/store/mod.rs` (điểm khai) được nhắc `StoreSpec::library_index`/`StoreKind::LibraryIndex`.
//!
//! ⚠️ Tệp riêng có chủ ý, đúng khuôn `store_boundary.rs`: đây là phép kiểm **tĩnh trên cây
//! nguồn** (hành vi LÚC CHẠY của `Indexer` — xoá-và-dựng-lại, phát hiện trùng `work_id`,
//! `.atproj` không bị chạm — sống ở `library_index_contract.rs`; trộn hai thứ làm hỏng đúng
//! thứ khiến cả hai đọc được).
//!
//! ─────────────────────────────────────────────────────────────────────────────
//! `src-tauri/AGENTS.md:29` TỰ KHAI *"CHỈ `Indexer` GHI `library-index.db`"* — TỆP NÀY LÀ CỔNG
//! ─────────────────────────────────────────────────────────────────────────────
//! Trước Story 5.2, câu đó là một quy ước không cổng nào canh — đúng lớp hỏng mà Story 5.1
//! vừa đóng cho quy ước đặt tên (`naming_boundary.rs`). Tệp này biến nó thành một phép đo:
//! một người sau này gọi `Store::open(StoreSpec::library_index(..))` từ một module ngoài
//! `core/library/indexer.rs` làm `cargo test` đỏ và nêu đúng `tệp:dòng` (AC2 của story).
//!
//! ─────────────────────────────────────────────────────────────────────────────
//! CHÉP ĐỦ BỐN PHẦN CỦA `store_boundary.rs` — §Boundaries của story đòi tường minh
//! ─────────────────────────────────────────────────────────────────────────────
//! 1. [`the_scanned_tree_is_large_enough_to_be_real`] — sàn quần thể.
//! 2. [`only_the_named_two_files_may_name_the_library_index_store`] — needle cấm.
//! 3. [`core_library_indexer_actually_opens_the_library_index_store`] — đối chứng dương
//!    (module thật sự dùng, không phải một miễn trừ cho một tập rỗng).
//! 4. [`every_library_index_exemption_still_matches_a_real_file`] — tự kiểm mỗi miễn trừ
//!    khớp ít nhất một tệp thật (một đường dẫn miễn trừ gõ sai/tệp bị xoá làm nhánh miễn trừ
//!    không bao giờ chạy, và phép kiểm chính xanh HÔM NAY mà không kiểm gì cả).

use std::fs;
use std::path::{Path, PathBuf};

/// Hai tệp DUY NHẤT được phép nhắc `FORBIDDEN` — chỗ gọi ([`indexer.rs`]) và điểm khai
/// (`core/store/mod.rs`: biến thể `enum StoreKind`, `as_str()`, và hàm dựng
/// `StoreSpec::library_index`). Xem doc-comment của `StoreSpec::library_index` — nhánh
/// KHÔNG-DI-TRÚ thật sự sống ở `Indexer::open`, không ở điểm khai.
const EXEMPT_FILES: [&str; 2] = ["core/library/indexer.rs", "core/store/mod.rs"];

/// Hai hình dạng mà chỉ hai tệp trên được nhắc tới. `"StoreSpec::library_index"` (có tiền tố
/// đủ, không phải `"library_index"` trần) và `"StoreKind::LibraryIndex"` (có tiền tố đủ) —
/// khớp đúng `grep` của §Verification trong story, không rộng hơn.
const FORBIDDEN: [&str; 2] = ["StoreSpec::library_index", "StoreKind::LibraryIndex"];

/// Số tệp `.rs` tối thiểu dưới `src-tauri/src/**` để phép quét là thật.
///
/// Số thật lúc dựng (Story 5.2, 2026-08-27, sau khi thêm `core/library/indexer.rs`): **56**
/// (55 kế thừa từ `naming_boundary.rs` + 1). Sàn **44** (~78,6%), cùng khuôn tỷ lệ dư địa mà
/// `RS_FLOOR`/`RUST_FLOOR` của các tệp `*_boundary.rs` khác đang giữ.
const RS_FLOOR: usize = 44;

fn src_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src")
}

/// Đường dẫn tương đối, dùng dấu `/` trên cả hai nền tảng — bắt buộc cho NFR14, cùng lý do
/// mọi tệp `*_boundary.rs` khác đã ghi: `starts_with`/so sánh trên Windows so với `core\library`
/// và không bao giờ khớp nếu không chuẩn hoá.
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

        // ⚠️ `symlink_metadata`, không `metadata` — cùng bài học mọi tệp `*_boundary.rs` khác:
        // một liên kết trỏ về thư mục cha làm đệ quy không dừng.
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

/// Vị từ THUẦN trên một DÒNG MÃ đã biết KHÔNG PHẢI comment — chỗ gọi thật (Phần 2) lọc dòng
/// `//` TRƯỚC khi gọi hàm này, đúng khuôn `store_boundary.rs::only_core_store_may_name_rusqlite`.
/// Tách ra để cổng thật VÀ mọi ca đối chứng dương/âm dưới đây gọi CHUNG một hàm — hai bên
/// không thể trôi khỏi nhau bằng cách chép lại phép so hai lần (cùng khuôn
/// `naming_boundary.rs::line_names_a_forbidden_entity`).
fn line_names_the_library_index_store(code: &str) -> Option<&'static str> {
    FORBIDDEN.into_iter().find(|needle| code.contains(needle))
}

/// **Phần 1 — sàn quần thể.** Chạy trước mọi phép kiểm khác; xem doc-comment đầu tệp.
#[test]
fn the_scanned_tree_is_large_enough_to_be_real() {
    let (_, files) = all_rust_sources();
    assert!(
        files.len() >= RS_FLOOR,
        "chỉ tìm thấy {} tệp `.rs` dưới `src-tauri/src/**` (sàn {RS_FLOOR}). Cây quá nhỏ để \
         là thật — một danh sách rỗng làm mọi phép kiểm dưới đây xanh mà không kiểm gì cả. \
         Nghi phạm: gốc quét sai, hoặc một thư mục bị bỏ.",
        files.len()
    );
}

/// **Phần 2 — needle cấm.** AC2 của story: không module nào ngoài hai tệp miễn trừ được nhắc
/// `StoreSpec::library_index`/`StoreKind::LibraryIndex`.
#[test]
fn only_the_named_two_files_may_name_the_library_index_store() {
    let (root, files) = all_rust_sources();

    let mut violations: Vec<String> = Vec::new();

    for file in &files {
        let rel = rel_posix(&root, file);
        if EXEMPT_FILES.contains(&rel.as_str()) {
            continue;
        }

        let text =
            fs::read_to_string(file).unwrap_or_else(|e| panic!("đọc {}: {e}", file.display()));

        for (index, line) in text.lines().enumerate() {
            let code = line.trim_start();

            // Dòng comment KHÔNG phải vi phạm — cùng luật mà `store_boundary.rs` áp cho
            // `rusqlite`: một chỗ giải thích ranh giới không phải một lượt vượt qua nó.
            if code.starts_with("//") {
                continue;
            }

            if let Some(needle) = line_names_the_library_index_store(code) {
                violations.push(format!("{rel}:{}  {needle}  |  {code}", index + 1));
            }
        }
    }

    assert!(
        violations.is_empty(),
        "{} chỗ ngoài `core/library/indexer.rs`/`core/store/mod.rs` nhắc tới kho \
         `library-index.db`:\n{}\n\n\
         `src-tauri/AGENTS.md:29`: chỉ `Indexer` ghi `library-index.db`. Cần mở/đọc kho này thì \
         gọi qua `Indexer::open`/`Indexer::rebuild`/`Indexer::list_works` — không tự dựng \
         `StoreSpec`/`Store::open` cho nó ở nơi khác.",
        violations.len(),
        violations.join("\n")
    );
}

// ═════════════════════════════════════════════════════════════════════════════════
// Đối chứng dương/âm trên CHUỖI DỰNG TAY — độc lập với nội dung cây nguồn hôm nay,
// gọi thẳng [`line_names_the_library_index_store`], đúng khuôn `naming_boundary.rs`.
// ═════════════════════════════════════════════════════════════════════════════════

/// Đối chứng DƯƠNG: cả hai hình dạng khai/gọi thật đều bị vị từ bắt.
#[test]
fn a_hand_built_call_or_declaration_line_is_caught() {
    assert_eq!(
        line_names_the_library_index_store(
            "let store = Store::open(StoreSpec::library_index(path))?;"
        ),
        Some("StoreSpec::library_index"),
        "ca DUONG THAT: goi ham dung StoreSpec::library_index phai bi bat"
    );
    assert_eq!(
        line_names_the_library_index_store("if kind == StoreKind::LibraryIndex {"),
        Some("StoreKind::LibraryIndex"),
        "ca DUONG THAT: so sanh voi StoreKind::LibraryIndex phai bi bat"
    );
}

/// Đối chứng ÂM: dòng nhắc tới một kho KHÁC (`StoreSpec::project`/`StoreKind::Project`) —
/// không phải hình dạng needle của story này — không bị bắt oan.
#[test]
fn a_hand_built_line_naming_a_different_store_is_not_caught() {
    assert_eq!(
        line_names_the_library_index_store("let store = Store::open(StoreSpec::project(path))?;"),
        None,
        "ca AM: StoreSpec::project la kho KHAC, khong duoc bat oan"
    );
    assert_eq!(
        line_names_the_library_index_store("if kind == StoreKind::Project {"),
        None,
        "ca AM: StoreKind::Project la kho KHAC, khong duoc bat oan"
    );
}

/// ⚠️ Vị từ này THUẦN trên một dòng đã biết KHÔNG PHẢI comment — nó KHÔNG tự lọc `//`. Cổng
/// thật (Phần 2) lọc dòng đó TRƯỚC khi gọi, đúng khuôn `store_boundary.rs`. Ca này ghi lại hợp
/// đồng đó bằng chữ: gọi thẳng vị từ trên một chuỗi comment vẫn trả `Some` — trách nhiệm lọc
/// comment nằm ở CHỖ GỌI, không nằm ở vị từ.
#[test]
fn the_predicate_itself_does_not_filter_comment_lines_the_caller_does() {
    assert_eq!(
        line_names_the_library_index_store("// xem StoreKind::LibraryIndex ở trên"),
        Some("StoreKind::LibraryIndex"),
        "vi tu KHONG tu loc comment -- do la trach nhiem cua cho goi (Phan 2), da kiem rieng \
         bang `code.starts_with(\"//\")` truoc khi goi ham nay"
    );
}

/// **Phần 3 — đối chứng dương.** `core/library/indexer.rs` **thật sự** dùng cả hai needle —
/// không có ca này, phép kiểm ở Phần 2 xanh y hệt trên một cây mà `Indexer` đã bị xoá sạch
/// (*"không ai vi phạm"* và *"không có gì để vi phạm"* đọc giống nhau).
#[test]
fn core_library_indexer_actually_opens_the_library_index_store() {
    let indexer_path = src_root().join("core/library/indexer.rs");
    let text = fs::read_to_string(&indexer_path)
        .unwrap_or_else(|e| panic!("đọc {}: {e}", indexer_path.display()));

    for needle in FORBIDDEN {
        assert!(
            text.contains(needle),
            "`core/library/indexer.rs` không còn nhắc `{needle}` — miễn trừ ở Phần 2 đang canh \
             một tập RỖNG, tức không còn kiểm gì cả. Cây đã bị cắt, hoặc `Indexer` không còn tự \
             mở kho của nó."
        );
    }
}

/// **Phần 4 — tự kiểm mỗi miễn trừ khớp ≥ 1 tệp thật.** Một mục [`EXEMPT_FILES`] gõ sai (đổi
/// tên, xoá tệp) làm nhánh miễn trừ ở Phần 2 không bao giờ chạy — phép kiểm chính vẫn xanh HÔM
/// NAY (không ai đụng nó) rồi đỏ sai chỗ vào ngày một vi phạm thật xuất hiện đúng ở tệp có tên
/// đó.
#[test]
fn every_library_index_exemption_still_matches_a_real_file() {
    let root = src_root();
    for rel in EXEMPT_FILES {
        let candidate = root.join(rel);
        assert!(
            candidate.is_file(),
            "miễn trừ {rel:?} không khớp một tệp thật ({}). Nó đã lệch khỏi thực tế (đổi tên, \
             xoá tệp, …) — sửa `EXEMPT_FILES` trước khi tin cổng chính là kín.",
            candidate.display()
        );
    }
}

/// Tập tệp mang một needle **Ở VỊ TRÍ MÃ** (không tính dòng `//`) — đúng vị từ mà Phần 2 dùng
/// từng dòng, gọi lại đây trên TOÀN VĂN một tệp. Tách khỏi ca test để không hai bản chép tay
/// của cùng phép lọc trôi khỏi nhau.
fn files_naming_the_library_index_store_in_code(root: &Path, files: &[PathBuf]) -> Vec<String> {
    let mut out: Vec<String> = files
        .iter()
        .filter(|f| {
            let text = fs::read_to_string(f).unwrap_or_default();
            text.lines()
                .map(str::trim_start)
                .filter(|code| !code.starts_with("//"))
                .any(|code| line_names_the_library_index_store(code).is_some())
        })
        .map(|f| rel_posix(root, f))
        .collect();
    out.sort();
    out
}

/// **Đối chứng thứ hai, ĐỘC LẬP với Phần 2** — cùng vị từ [`line_names_the_library_index_store`]
/// và cùng luật lọc comment (`//`), nhưng gộp kết quả theo TỆP thay vì theo DÒNG rồi so trực
/// tiếp với [`EXEMPT_FILES`], thay vì chỉ khẳng định "danh sách vi phạm rỗng" như Phần 2. Bắt
/// được hai lớp lỗi mà Phần 2 một mình không bắt: (a) một mục `EXEMPT_FILES` không còn cần
/// (tệp đó không thật sự nhắc needle nào ở vị trí mã nữa) — Phần 4 chỉ kiểm tệp có TỒN TẠI,
/// không kiểm nó có THẬT SỰ dùng needle; (b) một tệp thứ ba lọt qua vì `EXEMPT_FILES` bị gõ
/// sai tên (khớp `.contains(&rel.as_str())` không bao giờ đúng ở Phần 2, nên nhánh miễn trừ
/// không chạy và MỌI dòng của tệp thật đều bị báo — tức Phần 2 tự đỏ trước ca này kịp chạy;
/// ca này là lớp phòng thủ thứ hai, không phải lớp DUY NHẤT).
///
/// ⚠️ **Sửa (vòng rà ba lớp, P3)** — bản trước quét TOÀN VĂN không lọc comment
/// (`t.contains(n)`), mâu thuẫn với chính luật "dòng comment KHÔNG phải vi phạm" mà Phần 2
/// tuyên bố ngay phía trên trong CÙNG tệp này. Hệ quả đã nằm trên cây: doc-comment của
/// `open_library_index` (`lib.rs`) từng bị viết MỜ ĐI có chủ ý để tránh chạm needle — uốn tài
/// liệu sản phẩm để giữ một test xanh, đúng thứ luật (a) của kho cấm. Nay ca này DÙNG LẠI đúng
/// vị từ Phần 2 (đã lọc comment), nên `lib.rs` được viết lại cho nói thẳng — xem
/// `open_library_index`.
#[test]
fn the_forbidden_needles_appear_in_exactly_the_two_exempt_files_and_nowhere_else() {
    let (root, files) = all_rust_sources();

    let files_with_a_needle = files_naming_the_library_index_store_in_code(&root, &files);

    let mut expected: Vec<String> = EXEMPT_FILES.iter().map(|s| s.to_string()).collect();
    expected.sort();

    assert_eq!(
        files_with_a_needle, expected,
        "tập tệp THẬT SỰ mang một needle ({files_with_a_needle:?}) lệch khỏi `EXEMPT_FILES` \
         ({expected:?}) — hoặc một tệp cần miễn trừ đã không còn cần (xoá khỏi danh sách), hoặc \
         một tệp thứ ba đã bắt đầu nhắc tới kho này (điều tra ngay, đây có thể là vi phạm THẬT)."
    );
}
