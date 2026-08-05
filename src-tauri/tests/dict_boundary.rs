//! Ranh giới cây nguồn của AC5 — ⛔ không `LIKE`, ⛔ không `GLOB`, ⛔ không `instr(`
//! dưới `src/core/dict/**`.
//!
//! ⚠️ Tệp riêng có chủ ý, đúng khuôn `store_boundary.rs`: `dict_lookup.rs` nghiệm thu
//! **hành vi lúc chạy**; đây là phép kiểm **tĩnh trên cây nguồn**, và trộn hai thứ là làm
//! hỏng đúng thứ khiến cả hai đọc được.
//!
//! ─────────────────────────────────────────────────────────────────────────────
//! VÌ SAO MỘT CỔNG CHỨ KHÔNG MỘT LƯỢT ĐỌC BẰNG MẮT
//! ─────────────────────────────────────────────────────────────────────────────
//! Giai đoạn 0 đo trên `dict-core.db` thật: `LIKE` một ký tự **20,09 ms**, hai ký tự
//! **50,14 ms** — so với `char_idx` **0,15 ms** và **4,49 ms**. Nhanh hơn **134×** và
//! **11×**. Bảng Stack liệt kê `LIKE` đích danh vào danh sách *"Không dùng, đã loại có lý
//! do"*.
//!
//! Cái giá của việc mất mệnh đề này ⛔ không phải một lỗi: nó là một lượt tra cứu **vẫn
//! trả đúng kết quả**, chỉ chậm hơn hai bậc độ lớn — tức NFR1 vỡ mà ⛔ không test hành vi
//! nào đỏ, và nó vỡ ở đúng lúc từ điển đủ lớn để người dùng cảm thấy.
//!
//! ⚠️ `instr(` cũng bị cấm, cùng lý do (quét toàn bảng) — dù nó **được phép** trong SQL
//! nghiệm thu chạy tay ở `sqlite3`, nơi nó là cách tái lập con số 350 của AC4. Phép xác
//! minh chuỗi con của đường sản phẩm chạy ở **Rust** (`str::contains`), ⛔ không ở SQL.

use std::fs;
use std::path::{Path, PathBuf};

/// Thư mục được quét. ⛔ Không phải một danh sách miễn trừ — đây là **phạm vi**.
const DICT_DIR: &str = "core/dict";

/// Số tệp `.rs` tối thiểu dưới `src/core/dict/**` để phép quét là thật.
///
/// Số thật lúc dựng (Story 1.11): **2** — `mod.rs` + `query.rs`. Sàn **1**, đúng khuôn
/// `RS_FLOOR` của `store_boundary.rs`: nó bắt một cây **bị cắt**, ⛔ không bắt việc thêm
/// tệp. *"Cây rỗng đọc thành sạch"* — một đường dẫn gõ sai làm `walk` khớp 0 tệp và cổng
/// này xanh mà ⛔ không kiểm gì cả, ngay ngày nó ra đời.
const DICT_FLOOR: usize = 1;

/// Ba token bị cấm ở **vị trí mã** dưới `core/dict/**`.
///
/// `"instr("` có ngoặc mở dính liền, có chủ ý: nó là một **lời gọi hàm SQL**, và bản
/// không ngoặc sẽ bắt luôn mọi từ tiếng Anh chứa `instr` (`instruction`, `instrument`)
/// trong một câu văn — tức một cổng đỏ trên tài liệu, và một cổng như thế bị gỡ trong
/// tuần.
const FORBIDDEN: [&str; 3] = ["LIKE", "GLOB", "instr("];

fn is_word_byte(b: u8) -> bool {
    b.is_ascii_alphanumeric() || b == b'_'
}

/// `code` chứa `needle` ở một vị trí KHÔNG dính liền một danh định khác — **không phân
/// biệt hoa/thường**.
///
/// 🔴 **Không phân biệt hoa/thường là bắt buộc:** từ khoá và tên hàm SQLite *(`LIKE`,
/// `Like`, `like`, `GLOB`, `glob`, `INSTR(`, `instr(`)* đều chạy giống hệt nhau bất kể
/// hoa/thường. So khớp phân biệt hoa/thường ở đây để lọt một `e.headword like ?1` viết
/// thường — đúng lượt quét toàn bảng 134×/11× chậm hơn mà cổng này tồn tại để chặn — mà
/// cổng vẫn xanh.
///
/// ⚠️ Ranh giới chỉ xét ở đầu KIA của `needle` là một ký tự "từ" (chữ/số/`_`) —
/// `instr(` kết ở dấu `(`, ⛔ không phải một ký tự "từ", nên vế phải bỏ qua có chủ ý
/// *(một lời gọi thật luôn có tham số dính ngay sau dấu mở ngoặc)*. `LIKE`/`GLOB` cả
/// hai đầu đều là chữ, nên cả hai vế đều xét — để một hằng số kiểu `GLOBAL_X` (đúng quy
/// ước SCREAMING_SNAKE_CASE của chính tệp này) ⛔ không làm cổng đỏ nhầm.
fn contains_forbidden_token(code: &str, needle: &str) -> bool {
    let code_upper = code.to_ascii_uppercase();
    let needle_upper = needle.to_ascii_uppercase();
    let bytes = code_upper.as_bytes();
    let needle_bytes = needle_upper.as_bytes();
    let check_before = needle_bytes.first().copied().is_some_and(is_word_byte);
    let check_after = needle_bytes.last().copied().is_some_and(is_word_byte);

    let mut start = 0;
    while let Some(rel) = code_upper[start..].find(&needle_upper) {
        let idx = start + rel;
        let end = idx + needle_upper.len();

        let before_ok = !check_before || idx == 0 || !is_word_byte(bytes[idx - 1]);
        let after_ok = !check_after || end >= bytes.len() || !is_word_byte(bytes[end]);

        if before_ok && after_ok {
            return true;
        }
        start = idx + 1;
    }
    false
}

fn src_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src")
}

/// Đường dẫn tương đối, dùng dấu `/` trên cả hai nền tảng.
///
/// ⚠️ Chuẩn hoá `\` thành `/` là bắt buộc chứ ⛔ không phải làm đẹp — bài học NFR14 ở
/// `store_boundary.rs:68-73`: `starts_with` trên Windows so với `core\dict` và **không
/// bao giờ khớp**, nên cổng quét 0 tệp và chỉ đỏ trên **một** nhánh của ma trận CI.
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

        // ⚠️ `symlink_metadata`, ⛔ không `metadata`: `metadata` giải symlink, nên một liên
        // kết trỏ về thư mục cha làm đệ quy ⛔ không dừng.
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

fn dict_sources() -> Vec<(String, String)> {
    let root = src_root();
    let mut files = Vec::new();
    walk(&root.join("core").join("dict"), &mut files);
    files.sort();

    files
        .into_iter()
        .map(|file| {
            let rel = rel_posix(&root, &file);
            let text =
                fs::read_to_string(&file).unwrap_or_else(|e| panic!("đọc {}: {e}", file.display()));
            (rel, text)
        })
        .collect()
}

/// Sàn quần thể — chạy trước mọi phép kiểm khác. Xem [`DICT_FLOOR`].
#[test]
fn the_scanned_tree_is_large_enough_to_be_real() {
    let files = dict_sources();
    assert!(
        files.len() >= DICT_FLOOR,
        "chỉ tìm thấy {} tệp `.rs` dưới `src/{DICT_DIR}/**` (sàn {DICT_FLOOR}). Cây quá \
         nhỏ để là thật — một danh sách rỗng làm mọi phép kiểm dưới đây xanh mà ⛔ không \
         kiểm gì cả. Nghi phạm: gốc quét sai, hoặc một thư mục bị bỏ.",
        files.len()
    );
}

/// 🔴 **AC5** — ⛔ không một câu `LIKE` / `GLOB` / `instr(` nào trên đường tra cứu nóng.
#[test]
fn the_hot_lookup_path_never_scans_the_table() {
    let files = dict_sources();

    let mut violations: Vec<String> = Vec::new();
    for (rel, text) in &files {
        for (index, line) in text.lines().enumerate() {
            let code = line.trim_start();

            // ⚠️ Chỉ dòng bắt đầu bằng `//` được bỏ qua — cùng luật với
            // `store_boundary.rs:155` và vì cùng một lý do: doc-comment của `query.rs`
            // **giải thích** vì sao `LIKE` bị loại, kèm số đo, và một cổng đỏ trên câu
            // giải thích chính luật nó canh là một cổng bị gỡ trong tuần.
            //
            // ⛔ Comment đuôi dòng (`… LIKE …; // ghi chú`) vẫn bị bắt, vì phần mã vẫn ở
            // đầu dòng.
            if code.starts_with("//") {
                continue;
            }

            for needle in FORBIDDEN {
                if contains_forbidden_token(code, needle) {
                    violations.push(format!("{rel}:{}  {needle}  |  {code}", index + 1));
                }
            }
        }
    }

    assert!(
        violations.is_empty(),
        "{} chỗ dưới `core/dict/**` quét toàn bảng:\n{}\n\n\
         Giai đoạn 0 đo `LIKE` một ký tự 20,09 ms và hai ký tự 50,14 ms, so với `char_idx` \
         0,15 ms và 4,49 ms — chậm hơn 134× và 11×. NFR1 cho 100 ms ĐẦU-CUỐI, trong đó \
         backend giữ ≤ 10 ms.\n\n\
         Đường đúng: `char_idx` cho chuỗi con 1–2 ký tự, `entry_fts` (trigram) cho 3+, và \
         phép xác minh chuỗi con chạy ở **Rust** (`str::contains`), ⛔ KHÔNG ở SQL.",
        violations.len(),
        violations.join("\n")
    );
}

/// Đối chứng dương: `core/dict/**` **có thật sự** dùng hai chỉ mục mà AC5 đòi.
///
/// ⚠️ Không có ca này thì phép kiểm trên xanh y hệt trên một `core/dict/` **rỗng** —
/// *"không ai vi phạm"* và *"không có gì để vi phạm"* đọc giống hệt nhau. Đây là cùng
/// khuôn `core_store_actually_uses_rusqlite` của `store_boundary.rs`.
#[test]
fn the_lookup_path_actually_uses_the_two_indexes() {
    let files = dict_sources();
    let all: String = files
        .iter()
        .map(|(_, text)| text.as_str())
        .collect::<Vec<_>>()
        .join("\n");

    for needle in ["char_idx", "entry_fts", "INTERSECT"] {
        assert!(
            all.contains(needle),
            "`core/dict/**` ⛔ KHÔNG nhắc tới `{needle}`. Cổng cấm `LIKE` ở trên đang canh \
             một chỗ trống: ba nhánh của AD-26 dựng trên `char_idx` (chuỗi con 1–2 ký tự, \
             qua `INTERSECT` khi hai ký tự) và `entry_fts` (3+ ký tự)."
        );
    }

    // 🔴 Story 1.11b nới **quần thể**, ⛔ không nới **luật**: hai chuỗi phải có mặt CẢ
    // HAI. Đó là đối chứng dương của AD-44 ① vá A2 — ⛔ KHÔNG sổ đăng ký *"tệp `.db` nào
    // chứa ngôn ngữ nào"*; mọi tệp đang gắn đều được tra, và `lang` lọc TRONG SQL, ở CẢ
    // HAI đường.
    for needle in ["lang = 'zh'", "lang = 'en'"] {
        assert!(
            all.contains(needle),
            "`core/dict/**` ⛔ KHÔNG chứa `{needle}`. MỌI nhánh phải lọc `dict_entry.lang` \
             TƯỜNG MINH trong SQL — ⛔ không giả định *\"tệp này chỉ có một ngôn ngữ\"*. \
             `entry_fts` lập chỉ mục trigram trên `headword` của MỌI hàng, cả zh lẫn en: \
             đo được `entry_fts MATCH '\"dic\"'` ⇒ 572 hàng, 100% `lang='en'`."
        );
    }
}

// ═════════════════════════════════════════════════════════════════════════════════
// Story 1.11b · AC3 — vị từ điều phối chạy ĐÚNG MỘT LẦN, và chạy TRÊN adapter
// ═════════════════════════════════════════════════════════════════════════════════

/// 🔴 **AC3** — chuỗi `pick_route` xuất hiện ở **đúng một** tệp dưới `core/dict/**`, và
/// ⛔ **không** ở `query.rs`.
///
/// Vì sao đây là một cổng: để vị từ chạy **trong** adapter là để mỗi tệp `.db` tự trả lời
/// một câu hỏi thuộc về **cả lượt tra** — và hai tệp sẽ trả lời **khác nhau** ngay khi
/// định nghĩa `is_han` của chúng lệch. Tầng gom thật (Story 1.13) chưa tồn tại;
/// `pick_route` được khai **công khai** chính là để 1.13 gọi nó **một lần** rồi truyền
/// **cùng một** `route` xuống mọi tệp.
///
/// ⚠️ Cổng đếm **tệp**, ⛔ không đếm *"vị trí mã"* — nên một dòng chú thích nhắc tên vị từ
/// ở `query.rs` cũng làm nó đỏ, **có chủ ý**: một cổng có ngoại lệ *"trừ khi là comment"*
/// là một cổng chờ ngoại lệ thứ hai. Vế *"⛔ không ở vị trí mã của `query.rs`"* vẫn được
/// khẳng định **riêng**, để khi nó đỏ vì một lời gọi thật, thông điệp nói đúng thứ đã
/// hỏng thay vì nói *"đếm tệp sai"*.
#[test]
fn the_routing_predicate_lives_in_exactly_one_file_and_the_adapter_never_calls_it() {
    let files = dict_sources();

    let carriers: Vec<&str> = files
        .iter()
        .filter(|(_, text)| text.contains("pick_route"))
        .map(|(rel, _)| rel.as_str())
        .collect();

    assert_eq!(
        carriers.len(),
        1,
        "chuỗi `pick_route` có mặt ở {} tệp dưới `src/{DICT_DIR}/**` ({carriers:?}), chờ \
         ĐÚNG MỘT (`core/dict/mod.rs`, nơi khai nó).\n\n\
         Vị từ điều phối chạy ĐÚNG MỘT LẦN cho mỗi lượt tra, ở tầng gom (Story 1.13) — ⛔ \
         KHÔNG bên trong `lookup`, ⛔ KHÔNG bên trong `query.rs`. Một adapter tự phân xử \
         lại đường đi là một tệp `.db` tự trả lời một câu hỏi của CẢ lượt tra, và hai tệp \
         sẽ trả lời khác nhau ngay khi định nghĩa `is_han` của chúng lệch.",
        carriers.len()
    );
    assert_eq!(
        carriers.first().copied(),
        Some("core/dict/mod.rs"),
        "tệp mang `pick_route` là {carriers:?}, chờ `core/dict/mod.rs`"
    );

    for (rel, text) in &files {
        if rel != "core/dict/query.rs" {
            continue;
        }
        for (index, line) in text.lines().enumerate() {
            let code = line.trim_start();
            if code.starts_with("//") {
                continue;
            }
            assert!(
                !code.contains("pick_route"),
                "{rel}:{}  gọi vị từ điều phối ở một vị trí mã:  {code}\n\n\
                 Adapter ⛔ KHÔNG tự phân xử lại đường đi — `route` là THAM SỐ, đi xuống \
                 từ chỗ gọi (AD-44 ①, vá A1).",
                index + 1
            );
        }
    }
}

// ═════════════════════════════════════════════════════════════════════════════════
// Story 1.11b · AC2 — MỘT định nghĩa `is_han` trong toàn `src-tauri/**`
// ═════════════════════════════════════════════════════════════════════════════════

/// Số tệp `.rs` tối thiểu dưới `src-tauri/{src,tests}/**` để phép đếm dưới đây là thật.
///
/// Số thật lúc dựng (Story 1.11b): **36**. Sàn **20**, cùng khuôn `RS_FLOOR` của
/// `store_boundary.rs`: nó bắt một cây **bị cắt**, ⛔ không bắt việc thêm tệp.
const SRC_TAURI_RS_FLOOR: usize = 20;

/// 🔴 **AC2 vế cuối** — trong toàn bộ `src-tauri/**` chỉ còn **MỘT** định nghĩa `is_han`.
///
/// Bản sao chỉ-BMP (3 dải) từng nằm ở `tests/dict_lookup.rs` **đã lệch thật** so với bảy
/// dải của `tools/dict-build`. Hai định nghĩa lệch nhau định tuyến một truy vấn sang
/// đường tiếng Trung rồi tra nó vào một `char_idx` **chưa bao giờ lập chỉ mục ký tự đó**
/// ⇒ **rỗng**, ⛔ **không lỗi** — đúng lớp lỗi AD-26 ra đời để chặn.
///
/// ⚠️ Chuỗi cần tìm dựng bằng [`concat!`] chứ ⛔ không viết liền một mạch: viết liền, tệp
/// này tự khớp chính nó và cổng đỏ ngay ngày nó ra đời — rồi người sửa tiếp theo sẽ gỡ nó
/// bằng một danh sách miễn trừ. Cổng quét **cả** `src/**` **lẫn** `tests/**`, vì bản sao
/// đã bị xoá sống ở `tests/**`.
#[test]
fn exactly_one_definition_of_is_han_exists_under_src_tauri() {
    const NEEDLE: &str = concat!("fn ", "is_han(");

    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let mut files = Vec::new();
    walk(&manifest.join("src"), &mut files);
    walk(&manifest.join("tests"), &mut files);
    files.sort();

    assert!(
        files.len() >= SRC_TAURI_RS_FLOOR,
        "chỉ tìm thấy {} tệp `.rs` dưới `src-tauri/src/**` và `src-tauri/tests/**` (sàn \
         {SRC_TAURI_RS_FLOOR}). Cây quá nhỏ để là thật — một danh sách rỗng làm phép đếm \
         dưới đây ra 0 và cổng xanh mà ⛔ không kiểm gì cả.",
        files.len()
    );

    let carriers: Vec<String> = files
        .iter()
        .filter(|file| {
            fs::read_to_string(file)
                .unwrap_or_else(|e| panic!("đọc {}: {e}", file.display()))
                .contains(NEEDLE)
        })
        .map(|file| rel_posix(&manifest, file))
        .collect();

    assert_eq!(
        carriers.len(),
        1,
        "{} định nghĩa `is_han` dưới `src-tauri/**` ({carriers:?}), chờ ĐÚNG MỘT \
         (`src/core/dict/mod.rs`).\n\n\
         Hai bản sẽ TRÔI khỏi nhau — bản cũ ở `tests/dict_lookup.rs` chỉ có 3 dải BMP \
         trong khi `tools/dict-build` có 7, và bản hẹp hơn đọc `𠧜` (U+209DC) thành \
         *\"không phải chữ Hán\"* ⇒ truy vấn đi SANG đường tiếng Anh và trả RỖNG, ⛔ KHÔNG \
         lỗi.\n\n\
         Đường sửa: gọi `auratranslate_lib::core::dict::is_han`. ⛔ ĐỪNG chép bảng dải.",
        carriers.len()
    );
}
