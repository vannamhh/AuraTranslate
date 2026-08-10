//! Ranh giới cây nguồn của AC5 — không `LIKE`, không `GLOB`, không `instr(`
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
//! Cái giá của việc mất mệnh đề này không phải một lỗi: nó là một lượt tra cứu **vẫn
//! trả đúng kết quả**, chỉ chậm hơn hai bậc độ lớn — tức NFR1 vỡ mà không test hành vi
//! nào đỏ, và nó vỡ ở đúng lúc từ điển đủ lớn để người dùng cảm thấy.
//!
//! ⚠️ `instr(` cũng bị cấm, cùng lý do (quét toàn bảng) — dù nó **được phép** trong SQL
//! nghiệm thu chạy tay ở `sqlite3`, nơi nó là cách tái lập con số 350 của AC4. Phép xác
//! minh chuỗi con của đường sản phẩm chạy ở **Rust** (`str::contains`), không ở SQL.

use std::fs;
use std::path::{Path, PathBuf};

/// Thư mục được quét. Không phải một danh sách miễn trừ — đây là **phạm vi**.
const DICT_DIR: &str = "core/dict";

/// Số tệp `.rs` tối thiểu dưới `src/core/dict/**` để phép quét là thật.
///
/// Số thật lúc dựng (Story 1.11): **2** — `mod.rs` + `query.rs`. Sàn **1**, đúng khuôn
/// `RS_FLOOR` của `store_boundary.rs`: nó bắt một cây **bị cắt**, không bắt việc thêm
/// tệp. *"Cây rỗng đọc thành sạch"* — một đường dẫn gõ sai làm `walk` khớp 0 tệp và cổng
/// này xanh mà không kiểm gì cả, ngay ngày nó ra đời.
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
/// `instr(` kết ở dấu `(`, không phải một ký tự "từ", nên vế phải bỏ qua có chủ ý
/// *(một lời gọi thật luôn có tham số dính ngay sau dấu mở ngoặc)*. `LIKE`/`GLOB` cả
/// hai đầu đều là chữ, nên cả hai vế đều xét — để một hằng số kiểu `GLOBAL_X` (đúng quy
/// ước SCREAMING_SNAKE_CASE của chính tệp này) không làm cổng đỏ nhầm.
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
/// ⚠️ Chuẩn hoá `\` thành `/` là bắt buộc chứ không phải làm đẹp — bài học NFR14 ở
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

        // ⚠️ `symlink_metadata`, không `metadata`: `metadata` giải symlink, nên một liên
        // kết trỏ về thư mục cha làm đệ quy không dừng.
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
         nhỏ để là thật — một danh sách rỗng làm mọi phép kiểm dưới đây xanh mà không \
         kiểm gì cả. Nghi phạm: gốc quét sai, hoặc một thư mục bị bỏ.",
        files.len()
    );
}

/// 🔴 **AC5** — không một câu `LIKE` / `GLOB` / `instr(` nào trên đường tra cứu nóng.
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
            // Comment đuôi dòng (`… LIKE …; // ghi chú`) vẫn bị bắt, vì phần mã vẫn ở
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
         phép xác minh chuỗi con chạy ở **Rust** (`str::contains`), KHÔNG ở SQL.",
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
            "`core/dict/**` KHÔNG nhắc tới `{needle}`. Cổng cấm `LIKE` ở trên đang canh \
             một chỗ trống: ba nhánh của AD-26 dựng trên `char_idx` (chuỗi con 1–2 ký tự, \
             qua `INTERSECT` khi hai ký tự) và `entry_fts` (3+ ký tự)."
        );
    }

    // 🔴 Story 1.11b nới **quần thể**, không nới **luật**: hai chuỗi phải có mặt CẢ
    // HAI. Đó là đối chứng dương của AD-44 ① vá A2 — KHÔNG sổ đăng ký *"tệp `.db` nào
    // chứa ngôn ngữ nào"*; mọi tệp đang gắn đều được tra, và `lang` lọc TRONG SQL, ở CẢ
    // HAI đường.
    for needle in ["lang = 'zh'", "lang = 'en'"] {
        assert!(
            all.contains(needle),
            "`core/dict/**` KHÔNG chứa `{needle}`. MỌI nhánh phải lọc `dict_entry.lang` \
             TƯỜNG MINH trong SQL — không giả định *\"tệp này chỉ có một ngôn ngữ\"*. \
             `entry_fts` lập chỉ mục trigram trên `headword` của MỌI hàng, cả zh lẫn en: \
             đo được `entry_fts MATCH '\"dic\"'` ⇒ 572 hàng, 100% `lang='en'`."
        );
    }
}

// ═════════════════════════════════════════════════════════════════════════════════
// Story 1.11b · AC3 — vị từ điều phối chạy ĐÚNG MỘT LẦN, và chạy TRÊN adapter
// ═════════════════════════════════════════════════════════════════════════════════

/// 🔴 **AC3** — chuỗi `pick_route` xuất hiện ở **đúng một** tệp dưới `core/dict/**`, và
/// **không** ở `query.rs`.
///
/// Vì sao đây là một cổng: để vị từ chạy **trong** adapter là để mỗi tệp `.db` tự trả lời
/// một câu hỏi thuộc về **cả lượt tra** — và hai tệp sẽ trả lời **khác nhau** ngay khi
/// định nghĩa `is_han` của chúng lệch. Tầng gom thật (Story 1.13) chưa tồn tại;
/// `pick_route` được khai **công khai** chính là để 1.13 gọi nó **một lần** rồi truyền
/// **cùng một** `route` xuống mọi tệp.
///
/// ⚠️ Cổng đếm **tệp**, không đếm *"vị trí mã"* — nên một dòng chú thích nhắc tên vị từ
/// ở `query.rs` cũng làm nó đỏ, **có chủ ý**: một cổng có ngoại lệ *"trừ khi là comment"*
/// là một cổng chờ ngoại lệ thứ hai. Vế *"không ở vị trí mã của `query.rs`"* vẫn được
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
         Vị từ điều phối chạy ĐÚNG MỘT LẦN cho mỗi lượt tra, ở tầng gom (Story 1.13) — không \
         KHÔNG bên trong `lookup`, KHÔNG bên trong `query.rs`. Một adapter tự phân xử \
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
                 Adapter KHÔNG tự phân xử lại đường đi — `route` là THAM SỐ, đi xuống \
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
/// `store_boundary.rs`: nó bắt một cây **bị cắt**, không bắt việc thêm tệp.
const SRC_TAURI_RS_FLOOR: usize = 20;

/// 🔴 **AC2 vế cuối** — trong toàn bộ `src-tauri/**` chỉ còn **MỘT** định nghĩa `is_han`.
///
/// Bản sao chỉ-BMP (3 dải) từng nằm ở `tests/dict_lookup.rs` **đã lệch thật** so với bảy
/// dải của `tools/dict-build`. Hai định nghĩa lệch nhau định tuyến một truy vấn sang
/// đường tiếng Trung rồi tra nó vào một `char_idx` **chưa bao giờ lập chỉ mục ký tự đó**
/// ⇒ **rỗng**, **không lỗi** — đúng lớp lỗi AD-26 ra đời để chặn.
///
/// ⚠️ Chuỗi cần tìm dựng bằng [`concat!`] chứ không viết liền một mạch: viết liền, tệp
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
         dưới đây ra 0 và cổng xanh mà không kiểm gì cả.",
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
         *\"không phải chữ Hán\"* ⇒ truy vấn đi SANG đường tiếng Anh và trả RỖNG, KHÔNG \
         lỗi.\n\n\
         Đường sửa: gọi `auratranslate_lib::core::dict::is_han`. ĐỪNG chép bảng dải.",
        carriers.len()
    );
}

// ═════════════════════════════════════════════════════════════════════════════════
// Story 1.13 · AC2 — RUNTIME KHÔNG CÓ MÃ RIÊNG CHO TỪNG NGUỒN
// ═════════════════════════════════════════════════════════════════════════════════

/// Số tệp `.rs` tối thiểu dưới `src-tauri/src/**` để bốn cổng dưới đây là thật.
///
/// Số thật lúc dựng (Story 1.13): **31**. Sàn **20**, cùng khuôn mọi sàn khác của dự án:
/// nó bắt một cây **bị cắt**, không bắt việc thêm tệp. *"Cây rỗng đọc thành sạch"* — một
/// gốc quét gõ sai làm `walk` khớp 0 tệp và **cả bốn** cổng xanh mà không kiểm gì cả.
const SRC_ONLY_RS_FLOOR: usize = 20;

/// **Mười `code` THẬT**, đo trên bốn tệp `.db` ở `tools/dict-build/out/` ngày 2026-08-08.
///
/// 🔴 **Không một chuỗi nào trong đây được xuất hiện ở VỊ TRÍ MÃ dưới `src-tauri/src/**`
/// hay `src/**`.** AD-10 nói *"Runtime **không có mã riêng cho từng nguồn**"* và
/// `epics.md:1543` lặp lại — cả hai là **văn xuôi**. Đây là chỗ mệnh đề đó thành máy.
///
/// 🔴 **MỘT danh sách cho CẢ BA cổng, và đó là một sửa lỗi có bằng chứng.** Tới lượt code
/// review 2026-08-10 đây là **hai** danh sách: một `SOURCE_CODES` chín phần tử phục vụ cổng
/// Story 1.13 *(quét toàn `src-tauri/src/**`)*, và một `REAL_SOURCE_CODES` mười phần tử phục
/// vụ hai cổng Story 1.19 *(quét `core/dict/**` và `src/**`)*. Danh sách chín **thiếu**
/// `en-wiktionary-vi` và `tran-van-chanh`, lại **thừa** `hvtdtd` — một mã chưa từng tồn tại
/// trong dữ liệu thật *(Ice chốt 2026-08-08: không tìm được nguồn)*. Hệ quả **đo được**: cấy
/// `pub const PROBE_CODE: &str = "tran-van-chanh";` vào `src-tauri/src/lib.rs` rồi chạy
/// `cargo test --test dict_boundary` cho **14/14 xanh**, cổng không bắt. Hai danh sách cho
/// cùng một phép cấm là một nguồn sự thật thứ hai, và nó đã sai đúng như AD-44 ① tiên liệu.
///
/// 🔴 Danh sách này **không** phải một sổ đăng ký *"tệp nào chứa gì"* (thứ AD-44 ① vá A2
/// cấm): nó là **quần thể của một phép cấm**, sống trong `tests/` và không một dòng mã sản
/// phẩm nào đọc nó. Đường sản phẩm luôn dẫn xuất danh sách nguồn từ `DictLayers` đang gắn.
///
/// ⚠️ Hình dạng vi phạm **rẻ nhất** là một `if code == "vietphrase"` để *"sửa cho gọn"*
/// đúng 18 đầu mục trùng của §Quyết định #2 — và nó sẽ được viết bởi một người thật lòng
/// nghĩ mình đang vá một lỗi. `tools/dict-build/src/build.rs:365` đã đặt đúng luật này cho
/// phía **dựng** (*"KHÔNG viết thành `if code == \"...\"`"*); đây là phía **đọc**.
///
/// ⚠️ Comment **được phép** nhắc tên nguồn, và đó không phải một ngoại lệ mà là chính
/// điểm: doc-comment của `query.rs` **giải thích** phân bố dữ liệu bằng số đo có tên nguồn,
/// và một cổng đỏ trên câu giải thích chính luật nó canh là một cổng **bị gỡ trong tuần**.
///
/// ⚠️ **Một nguồn mới được dựng ⇒ thêm `code` của nó vào đây.** Đó là một việc phải làm tay,
/// và nó phải ở lại như vậy: đọc danh sách từ chính bốn tệp `.db` là để cổng **im lặng tự
/// tắt** trên một máy chưa dựng dữ liệu — đúng thứ `SRC_ONLY_RS_FLOOR` tồn tại để chặn.
const REAL_SOURCE_CODES: [&str; 10] = [
    "cc-cedict",
    "cvdict",
    "en-wiktionary",
    "en-wiktionary-vi",
    "unihan",
    "viwiktionary",
    "viwiktionary-en",
    "thieu-chuu",
    "tran-van-chanh",
    "vietphrase",
];

/// Mọi tệp `.rs` dưới `src-tauri/src/**`, kèm đường dẫn tương đối POSIX.
fn src_only_sources() -> Vec<(String, String)> {
    let root = src_root();
    let mut files = Vec::new();
    walk(&root, &mut files);
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

/// Sàn quần thể của bốn cổng Story 1.13 — chạy trước chúng.
#[test]
fn the_whole_src_tree_is_large_enough_to_be_real() {
    let files = src_only_sources();
    assert!(
        files.len() >= SRC_ONLY_RS_FLOOR,
        "chỉ tìm thấy {} tệp `.rs` dưới `src-tauri/src/**` (sàn {SRC_ONLY_RS_FLOOR}). Cây \
         quá nhỏ để là thật.",
        files.len()
    );

    // Đối chứng dương: quần thể **thật sự** chứa vùng mã mà bốn cổng canh.
    for expected in [
        "core/dict/layer.rs",
        "core/dict/senses.rs",
        "ports/dict_source.rs",
    ] {
        assert!(
            files.iter().any(|(rel, _)| rel == expected),
            "quần thể quét KHÔNG chứa `{expected}` — bốn cổng của Story 1.13 đang canh \
             một chỗ trống"
        );
    }
}

/// 🔴 **AC2** — không một literal mã nguồn nào ở **vị trí mã** dưới `src-tauri/src/**`.
///
/// Vì sao AD-10 đáng một cổng: tập lớp đến từ **quét thư mục** và danh tính lớp đến từ
/// **dữ liệu trong chính tệp**. Một `if code == "…"` ở đường đọc là một **nguồn sự thật thứ
/// hai** cho một dữ kiện đã nằm trong dữ liệu (AD-44 ① vá A2), và nó sai **im lặng** vào
/// đúng ngày một lớp gỡ rời được thêm hay gỡ đi (FR112).
#[test]
fn the_runtime_never_names_a_single_dictionary_source() {
    // 🔴 **Đối chứng dương thường trực** — không phải một lượt đột biến chạy tay rồi
    // quên. Phép so khớp phải chứng minh nó **bắt được** hình dạng vi phạm, ngay ở đây,
    // mỗi lượt CI. Không có nó, cổng dưới đây xanh y hệt trên một bộ so khớp hỏng.
    for code in REAL_SOURCE_CODES {
        let planted = format!("    let _ = \"{code}\";");
        assert!(
            contains_forbidden_token(&planted, code),
            "bộ so khớp KHÔNG bắt được `{code}` trong `{planted}` — cổng dưới đây đang \
             canh một chỗ trống"
        );
    }

    let files = src_only_sources();
    let mut violations: Vec<String> = Vec::new();

    for (rel, text) in &files {
        for (index, line) in text.lines().enumerate() {
            let code = line.trim_start();
            if code.starts_with("//") {
                continue;
            }
            for needle in REAL_SOURCE_CODES {
                if contains_forbidden_token(code, needle) {
                    violations.push(format!("{rel}:{}  {needle}  |  {code}", index + 1));
                }
            }
        }
    }

    assert!(
        violations.is_empty(),
        "{} chỗ dưới `src-tauri/src/**` viết mã riêng cho một nguồn cụ thể:\n{}\n\n\
         AD-10: *\"Runtime KHÔNG có mã riêng cho từng nguồn\"*. Tập lớp đến từ QUÉT THƯ MỤC \
         (AC3), danh tính lớp từ `dict_meta('layer')` của CHÍNH tệp, và nguồn từ \
         `dict_source` của CHÍNH tệp.\n\n\
         Nếu bạn vừa định viết `if code == \"…\"` để dọn 18 đầu mục trùng của VietPhrase: \
         §Quyết định #2 của Story 1.13 chốt **để nguyên**, và lý do chính là dòng bạn vừa \
         định viết.",
        violations.len(),
        violations.join("\n")
    );
}

/// 🔴 **AC2 vế thứ hai** — không một **tên tệp `.db`** nào viết cứng ở vị trí mã.
///
/// *"Gỡ một lớp = xoá một file"* (FR36) là **sai** ngay khi một danh sách tên tệp tồn tại
/// trong mã: xoá tệp thì danh sách vẫn còn, và nó nói dối về thứ đang có.
#[test]
fn the_layer_set_never_hardcodes_a_db_filename() {
    // Đối chứng dương thường trực — cùng lý do với cổng trên.
    assert!(
        mentions_a_dict_db_file("    let path = dir.join(\"dict-core.db\");"),
        "bộ so khớp KHÔNG bắt được một tên tệp `.db` viết cứng"
    );
    assert!(
        !mentions_a_dict_db_file("const GLOBAL_DB_FILE: &str = \"global.db\";"),
        "bộ so khớp đỏ nhầm trên `global.db` — đó là kho ghi của Story 1.7, không phải \
         một lớp từ điển"
    );

    let files = src_only_sources();
    let mut violations: Vec<String> = Vec::new();

    for (rel, text) in &files {
        for (index, line) in text.lines().enumerate() {
            let code = line.trim_start();
            if code.starts_with("//") {
                continue;
            }
            if mentions_a_dict_db_file(code) {
                violations.push(format!("{rel}:{}  |  {code}", index + 1));
            }
        }
    }

    assert!(
        violations.is_empty(),
        "{} chỗ viết cứng một tên tệp `.db` của từ điển:\n{}\n\n\
         Tập lớp đến từ `DictLayers::open(dir)` — nó thử mở MỌI tệp `*.db` trong thư mục. \
         Một danh sách tên tệp là một sổ đăng ký (AD-44 ① vá A2), và nó làm FR36 *\"xoá một \
         file\"* thành một mệnh đề sai.",
        violations.len(),
        violations.join("\n")
    );
}

/// Một dòng mã có nhắc tới một tên tệp `.db` của **từ điển** không.
fn mentions_a_dict_db_file(code: &str) -> bool {
    let lowered = code.to_ascii_lowercase();
    lowered.contains("dict-") && lowered.contains(".db")
}

// ═════════════════════════════════════════════════════════════════════════════════
// Story 1.13 · AC6 — KHÔNG TỒN TẠI MỘT HÀM HỢP NHẤT NGHĨA GIỮA CÁC NGUỒN
// ═════════════════════════════════════════════════════════════════════════════════

/// Động từ của một phép hợp nhất.
///
/// ⚠️ **Biến cách bất quy tắc phải liệt kê RIÊNG**, không suy ra từ dạng gốc bằng
/// `contains`: `"merging"` **không** chứa chuỗi con `"merge"` (bỏ `e` câm trước `-ing`),
/// và `"unified"`/`"unifies"` không chứa `"unify"` (`y → i` trước `-ed`/`-es`) — hai lỗi
/// chính tả tiếng Anh này từng làm `fn merge_entries(...)`/`fn unified_glosses(...)` lọt
/// qua cổng mà không ai biết. Đối chứng dương của bài test khoá cả bốn dạng.
const MERGE_VERBS: [&str; 13] = [
    "merge",
    "merging",
    "unify",
    "unified",
    "unifies",
    "dedup",
    "coalesce",
    "coalescing",
    "consolidate",
    "consolidating",
    "combine",
    "combining",
    "flatten",
];

/// Danh từ của thứ bị hợp nhất — **thứ AD-19 cấm động vào**.
///
/// 🔴 Cổng ghép **động từ + danh từ**, không cấm động từ một mình: `core/scope/resolve.rs`
/// có `merge_by_key` và `resolve_merge` **hợp lệ** — chúng hợp nhất **cấu hình hai tầng**
/// (Story 1.8), không hợp nhất nghĩa giữa các nguồn. Một cổng cấm chữ `merge` trần sẽ đỏ
/// trên chúng, và người sửa tiếp theo sẽ gỡ nó bằng một **danh sách miễn trừ** — rồi miễn
/// trừ thứ hai sẽ là một hàm thật sự vi phạm.
///
/// ⚠️ `"entry"` **không** chứa được `"entries"` (số nhiều bất quy tắc `y → ies`) — liệt kê
/// riêng, cùng lý do với `MERGE_VERBS`.
const MERGE_NOUNS: [&str; 7] = [
    "sense", "gloss", "source", "meaning", "entry", "entries", "dict",
];

/// 🔴 **AC6 mệnh đề cuối** — **không tồn tại** một hàm hợp nhất nghĩa giữa các nguồn.
#[test]
fn no_function_merges_meanings_across_sources() {
    // Đối chứng dương thường trực, và nó khai luôn **ranh giới** của cổng.
    for planted in [
        "fn merge_senses(",
        "fn dedupe_source_groups(",
        "    pub fn unify_dict_entries(&self) {",
        // 🔴 Ba dạng biến cách bất quy tắc — xem cảnh báo ở `MERGE_VERBS`/`MERGE_NOUNS`.
        "fn merge_entries(",
        "fn unified_glosses(",
        "    pub fn coalescing_dict_sources(&self) {",
    ] {
        assert!(
            merging_function_name(planted).is_some(),
            "bộ so khớp KHÔNG bắt được `{planted}`"
        );
    }
    for allowed in [
        "fn merge_by_key<K, V>(",
        "pub(crate) fn resolve_merge<V>(",
        "    pub fn apply_merge<V>(",
    ] {
        assert!(
            merging_function_name(allowed).is_none(),
            "bộ so khớp đỏ nhầm trên `{allowed}` — hợp nhất CẤU HÌNH hai tầng (Story 1.8) \
             không phải hợp nhất NGHĨA giữa các nguồn (AD-19)"
        );
    }

    let files = src_only_sources();
    let mut violations: Vec<String> = Vec::new();

    for (rel, text) in &files {
        for (index, line) in text.lines().enumerate() {
            let code = line.trim_start();
            if code.starts_with("//") {
                continue;
            }
            if let Some(name) = merging_function_name(code) {
                violations.push(format!("{rel}:{}  {name}  |  {code}", index + 1));
            }
        }
    }

    assert!(
        violations.is_empty(),
        "{} hàm dưới `src-tauri/src/**` mang hình dạng một phép HỢP NHẤT nguồn:\n{}\n\n\
         🔴 **AD-19** — KHÔNG tồn tại bước hợp nhất nguồn, ở bất kỳ đâu. **FR31**: mỗi \
         định nghĩa mang xuất xứ của nó. **FR32**: hai nguồn bất đồng ⇒ CẢ HAI có mặt, không \
         không nguồn nào được chọn làm *câu trả lời*.\n\n\
         Đó là toàn bộ lời hứa của Epic 1 với người dịch: *tôi tự phán xét thay vì tin một \
         câu trả lời đã bị gộp lại*. Một hàm gộp ở đây lấy đúng thứ đó đi.\n\n\
         ⚠️ Trùng đầu mục TRONG một nguồn (18 đầu mục của VietPhrase) là một câu hỏi KHÁC, \
         và §Quyết định #2 của Story 1.13 chốt **để nguyên** — xem `deferred-work.md`.",
        violations.len(),
        violations.join("\n")
    );
}

/// Tên hàm của `code` nếu nó mang **cả** một động từ hợp nhất **lẫn** một danh từ bị cấm.
fn merging_function_name(code: &str) -> Option<String> {
    let after_fn = code.split("fn ").nth(1)?;
    let name: String = after_fn
        .chars()
        .take_while(|c| c.is_ascii_alphanumeric() || *c == '_')
        .collect();
    if name.is_empty() {
        return None;
    }

    let lowered = name.to_ascii_lowercase();
    let has_verb = MERGE_VERBS.iter().any(|verb| lowered.contains(verb));
    let has_noun = MERGE_NOUNS.iter().any(|noun| lowered.contains(noun));

    (has_verb && has_noun).then_some(name)
}

// ═════════════════════════════════════════════════════════════════════════════════
// Story 1.13 · AC1 — `ports/` KHAI HÌNH DẠNG, KHÔNG MANG CÀI ĐẶT
// ═════════════════════════════════════════════════════════════════════════════════

/// 🔴 **AC1** — `ports/**` không gõ `rusqlite`, không `Connection::open`, không
/// chạm filesystem.
///
/// ⚠️ `store_boundary.rs::only_core_store_may_name_rusqlite` **đã** quét `src/**` và sẽ đỏ
/// trên hai chuỗi đầu; cổng này **không** thay nó và **không** nới nó. Nó thêm vế
/// **filesystem** — thứ cổng kia không canh — và nó nói bằng ngôn ngữ của AD-2 khi đỏ,
/// nên người đọc biết mình vừa phá **cổng** chứ không phải vừa phá *"một quy ước về
/// rusqlite"*.
#[test]
fn ports_declare_shape_and_never_open_anything() {
    // ⚠️ `"fs::"` **không** `"std::fs"`: `use std::{fs, path::Path};` — một kiểu import
    // gộp hoàn toàn bình thường — không mang chuỗi con `"std::fs"` ở đâu cả, nhưng lời
    // gọi `fs::read(…)`/`fs::metadata(…)` sau đó vẫn chạm filesystem y hệt. `"fs::"` bắt
    // được lời gọi đó bất kể nó được `use` theo kiểu nào — kể cả đường đủ `std::fs::…`, vì
    // `"fs::"` là chuỗi con của nó.
    const FORBIDDEN_IN_PORTS: [&str; 6] = [
        "rusqlite",
        "Connection::open",
        "fs::",
        "File::open",
        "read_dir",
        "PathBuf",
    ];

    let root = src_root();
    let mut files = Vec::new();
    walk(&root.join("ports"), &mut files);
    files.sort();

    assert!(
        files.len() >= 2,
        "chỉ {} tệp `.rs` dưới `src/ports/**` — cây quá nhỏ để là thật",
        files.len()
    );

    let mut violations: Vec<String> = Vec::new();
    let mut declares_the_port = false;

    for file in &files {
        let rel = rel_posix(&root, file);
        let text =
            fs::read_to_string(file).unwrap_or_else(|e| panic!("đọc {}: {e}", file.display()));

        if text.contains("trait DictionarySource") {
            declares_the_port = true;
        }

        for (index, line) in text.lines().enumerate() {
            let code = line.trim_start();
            if code.starts_with("//") {
                continue;
            }
            for needle in FORBIDDEN_IN_PORTS {
                if contains_forbidden_token(code, needle) {
                    violations.push(format!("{rel}:{}  {needle}  |  {code}", index + 1));
                }
            }
        }
    }

    assert!(
        violations.is_empty(),
        "{} chỗ dưới `src/ports/**` mang CÀI ĐẶT thay vì HÌNH DẠNG:\n{}\n\n\
         AD-2 khai ĐÚNG BA cổng, và một cổng khai *cái gì*, không khai *bằng cách nào*. \
         Đường mở tệp sống ở `core/store/**` (xem doc-comment `readonly.rs:1-14` về vì sao \
         nó ở đấy chứ không ở `core/dict/`); cài đặt của cổng này là \
         `core::dict::DictLayer`.",
        violations.len(),
        violations.join("\n")
    );

    // Đối chứng dương: `ports/**` **có thật** khai cổng thứ nhất. Không có vế này, cổng
    // trên xanh y hệt trên một `ports/` chỉ còn `mod.rs`.
    assert!(
        declares_the_port,
        "`src/ports/**` KHÔNG khai `trait DictionarySource`. Cổng ở trên đang canh một \
         chỗ trống — AD-2 nói cổng thứ nhất tồn tại, và Story 1.13 là story dựng nó."
    );
}

// ═════════════════════════════════════════════════════════════════════════════════
// Story 1.13 · AC7 — `ORDER BY ord` TRẦN LÀ MỘT CA FLAKY CHỜ SẴN
// ═════════════════════════════════════════════════════════════════════════════════

/// 🔴 **AC7** — mọi `ORDER BY` nhắc `ord` dưới `core/dict/**` phải mang **khoá phụ `id`**.
///
/// `tools/dict-build/src/sources/vietphrase.rs` tách `/` **vô điều kiện** và sinh nhiều
/// `dict_sense` **cùng `ord`** (`deferred-work.md`, Story 1.10). Với `ORDER BY ord` trần,
/// SQLite **không hứa** một thứ tự ổn định giữa các hàng bằng nhau — hai lượt chạy cho
/// hai thứ tự, tức một ca **flaky**, và một ca flaky **bị gỡ** chứ không được sửa.
///
/// ⚠️ Đây là cổng bổ sung cho ca hành vi
/// `dict_sources.rs::senses_sharing_one_ord_are_still_ordered_deterministically`, và **cả
/// hai đều cần**: trên một fixture nhỏ SQLite thường trả đúng thứ tự **do may mắn**, nên ca
/// hành vi một mình không phân biệt được may mắn với đúng.
#[test]
fn every_ord_ordering_carries_its_tiebreaker() {
    // Đối chứng dương thường trực.
    assert!(
        ordering_lacks_a_tiebreaker("ORDER BY s.ord\""),
        "bộ so khớp KHÔNG bắt được một `ORDER BY ord` trần"
    );
    assert!(
        !ordering_lacks_a_tiebreaker("ORDER BY s.entry_id, s.ord, s.id\","),
        "bộ so khớp đỏ nhầm trên một câu ĐÃ có khoá phụ"
    );
    assert!(
        !ordering_lacks_a_tiebreaker("ORDER BY e.id\""),
        "bộ so khớp đỏ nhầm trên `ORDER BY e.id` — không nhắc `ord` thì không có gì \
         để phá vỡ thế hoà"
    );

    let files = dict_sources();
    let mut violations: Vec<String> = Vec::new();

    for (rel, text) in &files {
        for (index, line) in text.lines().enumerate() {
            let code = line.trim_start();
            if code.starts_with("//") {
                continue;
            }
            if ordering_lacks_a_tiebreaker(code) {
                violations.push(format!("{rel}:{}  |  {code}", index + 1));
            }
        }
    }

    assert!(
        violations.is_empty(),
        "{} câu `ORDER BY` dưới `core/dict/**` sắp theo `ord` mà KHÔNG có khoá phụ \
         `id`:\n{}\n\n\
         `ord` KHÔNG duy nhất: VietPhrase sinh nhiều `dict_sense` cùng `ord` cho một đầu \
         mục. Thiếu khoá phụ ⇒ hai lượt chạy cho hai thứ tự ⇒ một ca flaky, và một ca flaky \
         BỊ GỠ chứ không được sửa.\n\n\
         Đường sửa: `ORDER BY … , x.ord, x.id`.",
        violations.len(),
        violations.join("\n")
    );
}

/// Câu `ORDER BY` của `code` có sắp theo `ord` mà thiếu khoá phụ `id` không.
fn ordering_lacks_a_tiebreaker(code: &str) -> bool {
    let Some(clause) = code.split("ORDER BY").nth(1) else {
        return false;
    };
    contains_forbidden_token(clause, "ord") && !contains_forbidden_token(clause, "id")
}

// ═════════════════════════════════════════════════════════════════════════════════
// STORY 1.19 — AC1 · AC4 · AC9: KHÔNG MỘT DANH TÍNH NGUỒN NÀO VIẾT CỨNG
// ═════════════════════════════════════════════════════════════════════════════════

// Quần thể của cả ba cổng dưới đây là [`REAL_SOURCE_CODES`], khai một lần duy nhất ở §Story
// 1.13. Tới lượt code review 2026-08-10 chỗ này còn giữ một **bản sao** mười phần tử đứng
// cạnh một bản chín phần tử ở §1.13, và bản chín ấy chính là chỗ cổng `src-tauri/src/**`
// thủng — xem doc-comment của hằng đó để biết phép cấy lỗi đã đo ra điều gì.

/// 🔴 **AC4** — `core/dict/**` **không gõ tên một `code` nguồn cụ thể nào**.
///
/// Bộ lọc nguồn là một **THAM SỐ từ chỗ gọi** (§Quyết định #2a), cùng doctrine
/// `route`/`branch`/`limit`. Một tên nguồn viết cứng ở tầng gom là một quy tắc nghiệp vụ
/// chôn vào tầng dữ liệu: nó sai **im lặng** vào đúng ngày một lớp được thêm hay gỡ đi.
#[test]
fn the_grouping_layer_never_names_a_single_source_code() {
    let files = dict_sources();

    let mut violations: Vec<String> = Vec::new();
    for (rel, text) in &files {
        for (index, line) in text.lines().enumerate() {
            let code = line.trim_start();
            if code.starts_with("//") {
                continue;
            }
            for needle in REAL_SOURCE_CODES {
                if code.contains(needle) {
                    violations.push(format!("{rel}:{}  {needle}  |  {code}", index + 1));
                }
            }
        }
    }

    assert!(
        violations.is_empty(),
        "{} chỗ dưới `core/dict/**` gõ tên một nguồn cụ thể:\n{}\n\n\
         Tầng gom biết `code` là một CHUỖI KHOÁ, không biết chuỗi nào có nghĩa gì. Tập bị \
         tắt đi vào bằng THAM SỐ (`lookup_grouped`/`lookup_han_viet`), và nó đọc từ `Store` \
         ở tầng `commands/` — AC4 của Story 1.19.",
        violations.len(),
        violations.join("\n")
    );
}

/// 🔴 **AC4, vế thứ hai** — `core/dict/**` **không đọc `Store`** một lời nào.
///
/// Đối xứng với `store_boundary.rs::core_store_does_not_depend_on_tauri`: một lượt đọc cấu
/// hình bên trong tầng gom là một đường thứ hai mà chỗ gọi không kiểm soát được, và nó làm
/// hai lượt tra cùng tham số cho hai kết quả khác nhau.
#[test]
fn the_grouping_layer_never_reads_the_config_store() {
    let files = dict_sources();

    let mut violations: Vec<String> = Vec::new();
    for (rel, text) in &files {
        for (index, line) in text.lines().enumerate() {
            let code = line.trim_start();
            if code.starts_with("//") {
                continue;
            }
            for needle in ["core::scope", "load_global_config", "GlobalConfig"] {
                if code.contains(needle) {
                    violations.push(format!("{rel}:{}  {needle}  |  {code}", index + 1));
                }
            }
        }
    }

    assert!(
        violations.is_empty(),
        "{} chỗ dưới `core/dict/**` đọc tầng cấu hình:\n{}\n\n\
         `disabled` là một GIÁ TRỊ đi xuống từ `commands/dict.rs`, không một thứ tầng gom \
         tự đi lấy (AC4, §Quyết định #2a).",
        violations.len(),
        violations.join("\n")
    );
}

/// 🔴 **AC1 · AC9** — cây webview và catalog chuỗi **không** viết cứng một danh tính nguồn nào.
///
/// Hai vế, và vế thứ hai là điều kiện để chỗ giữ `author-grant` dùng lại được:
/// - **AC1** — dải chip và bảng Attribution dẫn xuất từ `list_dict_sources`, nên **0** `code`
///   thật được phép xuất hiện ở vị trí mã hay trong `vi.json`;
/// - **AC9** — **danh tính TÁC GIẢ** đọc từ `dict_source.attribution` của **chính tệp**. Một
///   cái tên viết vào `vi.json` khoá chỗ giữ đó vào **một** nguồn, và nguồn ấy (HVTĐTD) sẽ
///   không tới — Ice chốt 2026-08-08.
///
/// ⚠️ Quét cả `src/**` (không chỉ `.vue`) và `src/i18n/vi.json`. Sàn quần thể đi kèm: một
/// đường dẫn gõ sai làm cổng xanh mà không đọc gì cả.
#[test]
fn the_webview_and_the_string_catalog_hardcode_no_source_identity() {
    let webview = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("src");

    let mut files: Vec<PathBuf> = Vec::new();
    walk_any(&webview, &mut files);
    files.sort();

    assert!(
        files.len() >= 30,
        "chỉ tìm thấy {} tệp dưới `src/**` — quần thể quá nhỏ để là thật, và một danh sách \
         rỗng làm phép cấm dưới đây xanh mà không kiểm gì cả",
        files.len()
    );

    // 🔴 Tên tác giả gắn với chỗ giữ `author-grant` trong mockup và trong AC gốc của epic.
    // Nó **không** được xuất hiện ở đâu trong cây webview: `attribution` của chính tệp `.db`
    // là nguồn sự thật duy nhất cho danh tính tác giả.
    const FORBIDDEN_AUTHOR: &str = "Đặng Thế Kiệt";

    let mut violations: Vec<String> = Vec::new();
    for file in &files {
        let rel = rel_posix(&webview, file);
        let text = fs::read_to_string(file).unwrap_or_else(|e| panic!("đọc {rel}: {e}"));
        // 🔴 **Che chú thích TRƯỚC khi quét, không lọc theo dòng đầu.** Một khối `<!-- … -->`
        // nhiều dòng có dòng thứ hai **không** bắt đầu bằng `<!--`, nên phép lọc theo dòng
        // để lọt đúng thứ nó định bỏ qua — và cổng đỏ trên chính câu GIẢI THÍCH luật nó
        // canh. Cổng `LIKE` ở trên đã ghi bằng chữ vì sao đó là đường ngắn nhất tới việc
        // cổng bị gỡ.
        let text = mask_comments(&text);
        for (index, line) in text.lines().enumerate() {
            let code = line.trim_start();
            for needle in REAL_SOURCE_CODES {
                if code.contains(needle) {
                    violations.push(format!("{rel}:{}  {needle}  |  {code}", index + 1));
                }
            }
            if code.contains(FORBIDDEN_AUTHOR) {
                violations.push(format!("{rel}:{}  <ten tac gia>  |  {code}", index + 1));
            }
        }
    }

    assert!(
        violations.is_empty(),
        "{} chỗ trong cây webview viết cứng một danh tính nguồn:\n{}\n\n\
         Dải chip và bảng Attribution DẪN XUẤT từ `list_dict_sources` — thêm một tệp `.db` \
         phải làm chúng hiện ra mà KHÔNG sửa một dòng mã (AC1). Và danh tính tác giả đọc từ \
         `dict_source.attribution` của chính tệp, không từ `vi.json` (AC9).",
        violations.len(),
        violations.join("\n")
    );
}

/// Mọi tệp thường dưới `dir`, bỏ symlink — cùng luật [`walk`], khác ở chỗ không lọc đuôi.
fn walk_any(dir: &Path, out: &mut Vec<PathBuf>) {
    let entries = fs::read_dir(dir).unwrap_or_else(|e| panic!("đọc {}: {e}", dir.display()));
    for entry in entries {
        let entry = entry.unwrap_or_else(|e| panic!("duyệt {}: {e}", dir.display()));
        let path = entry.path();
        let meta =
            fs::symlink_metadata(&path).unwrap_or_else(|e| panic!("lstat {}: {e}", path.display()));
        if meta.file_type().is_symlink() {
            continue;
        }
        if meta.is_dir() {
            walk_any(&path, out);
        } else {
            out.push(path);
        }
    }
}

/// Thay mọi ký tự **trong chú thích** bằng khoảng trắng, **giữ nguyên số dòng**.
///
/// Ba dạng: `<!-- … -->` (`.vue` template) · `/* … */` · `// …` tới hết dòng.
///
/// 🔴 Giữ nguyên xuống dòng là bắt buộc: số dòng trong thông báo vi phạm phải trỏ đúng chỗ,
/// và một bản cắt bỏ hẳn chú thích sẽ trỏ lệch đúng bằng số dòng đã cắt.
///
/// ⚠️ **Giới hạn đã biết, ghi ra chứ không giấu:** máy trạng thái này **không** theo dõi
/// chuỗi ký tự, nên một `//` bên trong một chuỗi (`'https://…'`) che nốt phần còn lại của
/// dòng đó. Hướng lệch là **âm tính giả** — cổng bỏ sót, không đỏ oan — và với một phép
/// **cấm** thì đó là hướng phải cân nhắc. Chấp nhận ở đây vì mười `code` thật đều là danh
/// định kebab-case, không cái nào sống được trong một URL của cây nguồn này.
fn mask_comments(text: &str) -> String {
    #[derive(Clone, Copy, PartialEq)]
    enum State {
        Code,
        Html,
        Block,
        Line,
    }

    let bytes: Vec<char> = text.chars().collect();
    let mut out = String::with_capacity(text.len());
    let mut state = State::Code;
    let mut i = 0usize;

    let starts = |b: &[char], at: usize, needle: &str| -> bool {
        let n: Vec<char> = needle.chars().collect();
        at + n.len() <= b.len() && b[at..at + n.len()] == n[..]
    };

    while i < bytes.len() {
        let c = bytes[i];
        match state {
            State::Code => {
                if starts(&bytes, i, "<!--") {
                    state = State::Html;
                    out.push_str("    ");
                    i += 4;
                    continue;
                }
                if starts(&bytes, i, "/*") {
                    state = State::Block;
                    out.push_str("  ");
                    i += 2;
                    continue;
                }
                if starts(&bytes, i, "//") {
                    state = State::Line;
                    out.push_str("  ");
                    i += 2;
                    continue;
                }
                out.push(c);
                i += 1;
            }
            State::Html => {
                if starts(&bytes, i, "-->") {
                    state = State::Code;
                    out.push_str("   ");
                    i += 3;
                    continue;
                }
                out.push(if c == '\n' { '\n' } else { ' ' });
                i += 1;
            }
            State::Block => {
                if starts(&bytes, i, "*/") {
                    state = State::Code;
                    out.push_str("  ");
                    i += 2;
                    continue;
                }
                out.push(if c == '\n' { '\n' } else { ' ' });
                i += 1;
            }
            State::Line => {
                if c == '\n' {
                    state = State::Code;
                    out.push('\n');
                } else {
                    out.push(' ');
                }
                i += 1;
            }
        }
    }
    out
}
