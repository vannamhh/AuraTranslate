//! Ranh giới cây nguồn của Story 1.12 — **đúng MỘT** cài đặt khớp ngôn ngữ, và
//! `core/dict/**` **KHÔNG** gọi nó.
//!
//! ⚠️ Tệp riêng có chủ ý, đúng khuôn `store_boundary.rs`/`dict_boundary.rs`:
//! `matching_contract.rs` nghiệm thu **hành vi lúc chạy**; đây là phép kiểm **tĩnh trên
//! cây nguồn**, và trộn hai thứ là làm hỏng đúng thứ khiến cả hai đọc được.
//!
//! ─────────────────────────────────────────────────────────────────────────────
//! 🔴 VÌ SAO MỘT CỔNG CHỨ KHÔNG MỘT LƯỢT ĐỌC BẰNG MẮT
//! ─────────────────────────────────────────────────────────────────────────────
//! `epics.md:1510` viết cho Story 1.12: *"**And** `dict/` **dùng nó**"*. Vế đó **KHÔNG
//! CÒN ĐÚNG** — AD-17 đã được sửa Rule ngày 2026-08-05 (`ARCHITECTURE-SPINE.md:236`) và
//! thân Rule nói thẳng: *"AD này nói mọi nơi cần khớp ngôn ngữ dùng chung MỘT cài đặt —
//! nó KHÔNG nói mọi đường đều phải gọi Matcher. Đường tra cứu **từ điển** tiếng Anh không
//! không gọi"*.
//!
//! Nhưng câu cũ **vẫn còn nguyên trong `epics.md`** (chủ sở hữu John/PM,
//! `deferred-work.md`), và sơ đồ mermaid của AD-13 (`ARCHITECTURE-SPINE.md:189`) **vẫn
//! còn cạnh `dict --> matching`** (chủ sở hữu Winston). Một dev đọc epics — hoặc một lượt
//! review sau — sẽ đọc thấy hai thứ đó **trước** khi đọc thân Rule của AD-17. **Cổng này
//! là chỗ duy nhất mệnh đề đúng sống sót qua một lượt đọc ẩu.**
//!
//! Cái giá của việc mất mệnh đề này không phải một lỗi trả sai: AD-44 ③ đo trên corpus
//! thật rằng mọi biến thể hình thái **đã có sẵn làm đầu mục riêng** (16/16 mẫu thử, gồm
//! cả bất quy tắc), nên một lượt stemming chèn vào đường nóng đổi p95 **0,052–0,961 ms**
//! của Story 1.11b lấy **~0 recall**. Đó là NFR1 bị tiêu ngân sách mà không test hành
//! vi nào đỏ.

use std::fs;
use std::path::{Path, PathBuf};

/// Thư mục sở hữu cài đặt khớp ngôn ngữ. Không phải một danh sách miễn trừ — đây là
/// **phạm vi**.
const MATCHING_DIR: &str = "core/matching";

/// Số tệp `.rs` tối thiểu dưới `src/core/matching/**` để phép quét là thật.
///
/// Số thật lúc dựng (Story 1.12): **1** — `mod.rs`. Sàn **1**, đúng khuôn `DICT_FLOOR`
/// của `dict_boundary.rs:36`: nó bắt một cây **bị cắt**, không bắt việc thêm tệp.
/// *"Cây rỗng đọc thành sạch"* — một đường dẫn gõ sai làm `walk` khớp 0 tệp và mọi phép
/// kiểm dưới đây xanh mà không kiểm gì cả, ngay ngày chúng ra đời.
///
/// ⚠️ Đây là một sàn **THÊM VÀO**, không phải một sàn bị hạ. Story này không nới
/// `RS_FLOOR` của `store_boundary.rs`/`scope_boundary.rs`, không nới `DICT_FLOOR`/
/// `SRC_TAURI_RS_FLOOR` của `dict_boundary.rs`, không nới `FORBIDDEN`/`STORE_DIR` của
/// bất kỳ cổng nào đã có.
const MATCHING_FLOOR: usize = 1;

/// Số tệp `.rs` tối thiểu dưới `src-tauri/src/**` để phép đếm toàn cây là thật.
///
/// Số thật lúc dựng (Story 1.12): **28**. Sàn **20**, cùng khuôn `SRC_TAURI_RS_FLOOR`
/// của `dict_boundary.rs`.
///
/// ⚠️ Story 2.1 (2026-08-12): số thật là **42**; sàn lên **34** (81,0%), nâng cùng lượt với
/// `store_boundary.rs`/`scope_boundary.rs` — một sàn 20 trên 42 tệp để mất hơn nửa cây mà
/// vẫn xanh, tức nó thôi canh được đúng thứ nó tồn tại để canh.
const SRC_RS_FLOOR: usize = 34; // số THẬT 2026-08-12 (sau Story 2.1): 42 tệp `.rs` — 34/42 = 81,0%

/// Hai crate mà **chỉ** `core/matching/**` được gõ ở vị trí mã (AD-17).
const MATCHING_ONLY_CRATES: [&str; 2] = ["jieba_rs", "tantivy_stemmers"];

/// Bốn token bị cấm ở **vị trí mã** dưới `core/dict/**` (AD-17 thân Rule + AD-44 ③).
///
/// `"stem("` có ngoặc mở dính liền, cùng lý do với `"instr("` của `dict_boundary.rs:44`:
/// nó là một **lời gọi hàm**, và bản không ngoặc sẽ bắt luôn mọi từ tiếng Anh chứa
/// `stem` (`system`, `stemming`) trong một câu văn — tức một cổng đỏ trên tài liệu, và
/// một cổng như thế bị gỡ trong tuần.
const DICT_FORBIDDEN: [&str; 4] = ["matching", "jieba", "stemmer", "stem("];

/// Bốn tiền tố phụ thuộc ra ngoài mà `core/matching/**` không được gõ — module này là
/// **LÁ** trong đồ thị phụ thuộc (AD-13).
///
/// 🔴 **Dạng TRẦN, không có tiền tố `use `** — vá lúc code review (2026-08-05): bản
/// trước gõ `"use crate::core::"` v.v., và một lời gọi đủ điều kiện viết THẲNG trong thân
/// hàm (vd. `crate::core::dict::foo()`, không qua `use`) lọt qua cổng mà không bị
/// bắt — đúng lớp vi phạm mà cổng này tồn tại để chặn. Cùng khuôn với
/// `core_store_does_not_depend_on_tauri` của `store_boundary.rs`, vốn khớp cả dạng trần
/// `"tauri::"`, và cùng khuôn `contains_forbidden_token` mà AC1/AC2/AC4 của tệp này đã
/// dùng.
const MATCHING_FORBIDDEN_USES: [&str; 4] = [
    "crate::core::",
    "crate::ports",
    "crate::commands",
    "super::",
];

fn is_word_byte(b: u8) -> bool {
    b.is_ascii_alphanumeric() || b == b'_'
}

/// `code` chứa `needle` ở một vị trí KHÔNG dính liền một danh định khác — **không
/// phân biệt hoa/thường**.
///
/// 🔴 Khuôn chép nguyên từ `dict_boundary.rs:64`, và **không** chế khuôn mới. Không
/// phân biệt hoa/thường là bắt buộc: thứ cổng này canh là một **danh định có thể viết
/// nhiều kiểu** (`Jieba` · `jieba_rs` · `JIEBA` · `Stemmer` · `stemmer`), và một phép so
/// khớp phân biệt hoa/thường để lọt đúng biến thể mà người vi phạm tình cờ gõ.
/// `dict_boundary.rs:57` đã phải vá đúng chỗ này ở lượt review 1.11b.
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
/// `store_boundary.rs:68-73`: `starts_with` trên Windows so với `core\matching` và
/// **không bao giờ khớp**, nên cổng quét 0 tệp và chỉ đỏ trên **một** nhánh của ma
/// trận CI.
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

        // ⚠️ `symlink_metadata`, không `metadata`: `metadata` giải symlink, nên một
        // liên kết trỏ về thư mục cha làm đệ quy không dừng.
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

/// Mọi tệp `.rs` dưới `src-tauri/src/**`, kèm đường dẫn tương đối kiểu POSIX.
fn src_sources() -> Vec<(String, String)> {
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

/// Dòng mã, không phải dòng comment.
///
/// ⚠️ Chỉ dòng bắt đầu bằng `//` được bỏ qua — cùng luật với `store_boundary.rs:155` và
/// `dict_boundary.rs:172`, và vì cùng một lý do: doc-comment của `core/matching/mod.rs`
/// **giải thích** vì sao `dict/` không gọi nó, kèm số đo, và một cổng đỏ trên chính
/// câu giải thích luật nó canh là một cổng bị gỡ trong tuần.
///
/// Comment đuôi dòng (`… jieba …; // ghi chú`) vẫn bị bắt, vì phần mã vẫn ở đầu dòng.
fn code_lines(text: &str) -> impl Iterator<Item = (usize, &str)> {
    text.lines()
        .enumerate()
        .map(|(index, line)| (index + 1, line.trim_start()))
        .filter(|(_, code)| !code.starts_with("//"))
}

// ═════════════════════════════════════════════════════════════════════════════════
// Sàn quần thể — chạy TRƯỚC mọi phép kiểm khác
// ═════════════════════════════════════════════════════════════════════════════════

/// Xem [`MATCHING_FLOOR`] và [`SRC_RS_FLOOR`].
#[test]
fn the_scanned_tree_is_large_enough_to_be_real() {
    let files = src_sources();

    assert!(
        files.len() >= SRC_RS_FLOOR,
        "chỉ tìm thấy {} tệp `.rs` dưới `src-tauri/src/**` (sàn {SRC_RS_FLOOR}). Cây quá \
         nhỏ để là thật — một danh sách rỗng làm mọi phép kiểm dưới đây xanh mà không \
         kiểm gì cả. Nghi phạm: gốc quét sai, hoặc một thư mục bị bỏ.",
        files.len()
    );

    let matching = files
        .iter()
        .filter(|(rel, _)| rel.starts_with(MATCHING_DIR))
        .count();
    assert!(
        matching >= MATCHING_FLOOR,
        "chỉ tìm thấy {matching} tệp `.rs` dưới `src/{MATCHING_DIR}/**` (sàn \
         {MATCHING_FLOOR}). Một đường dẫn gõ sai làm `walk` khớp 0 tệp, và khi đó cổng \
         *\"chỉ `core/matching/**` được gõ hai crate\"* xanh y hệt trên một thư mục RỖNG."
    );
}

// ═════════════════════════════════════════════════════════════════════════════════
// AC1 — ĐÚNG MỘT cài đặt, và nó nằm ở `core/matching/`
// ═════════════════════════════════════════════════════════════════════════════════

/// 🔴 **AC1** — **chỉ** `core/matching/**` gõ `jieba_rs` và `tantivy_stemmers`.
#[test]
fn only_the_matching_module_ever_names_the_two_language_crates() {
    let files = src_sources();

    let mut violations: Vec<String> = Vec::new();
    for (rel, text) in &files {
        if rel.starts_with(MATCHING_DIR) {
            continue;
        }
        for (line_no, code) in code_lines(text) {
            for needle in MATCHING_ONLY_CRATES {
                if contains_forbidden_token(code, needle) {
                    violations.push(format!("{rel}:{line_no}  {needle}  |  {code}"));
                }
            }
        }
    }

    assert!(
        violations.is_empty(),
        "{} chỗ NGOÀI `core/matching/**` gõ tên một crate khớp ngôn ngữ:\n{}\n\n\
         AD-17: tồn tại ĐÚNG MỘT cài đặt khớp ngôn ngữ, và nó sống ở `core/matching/`. \
         Một lời gọi `jieba_rs`/`tantivy_stemmers` ở nơi khác là một cài đặt THỨ HAI đang \
         mọc — và lớp lỗi mà AD-17 tồn tại để chặn là *\"Glossary bắt được một biến thể \
         mà TM không bắt được, và không ai hiểu vì sao\"*.\n\n\
         Đường đúng: gọi `auratranslate_lib::core::matching::{{tokenize, normalize, \
         ngrams, find_terms}}`.",
        violations.len(),
        violations.join("\n")
    );
}

/// 🔴 **AC1 — đối chứng dương.** `core/matching/**` **có thật sự** gõ **cả hai** crate.
///
/// ⚠️ Không có ca này thì phép kiểm trên xanh y hệt trên một `core/matching/` **rỗng**
/// — *"không ai vi phạm"* và *"không có gì để vi phạm"* đọc giống hệt nhau. Đây là cùng
/// khuôn `core_store_actually_uses_rusqlite` của `store_boundary.rs:209` và
/// `the_lookup_path_actually_uses_the_two_indexes` của `dict_boundary.rs:203`.
#[test]
fn the_matching_module_actually_uses_both_language_crates() {
    let all: String = src_sources()
        .into_iter()
        .filter(|(rel, _)| rel.starts_with(MATCHING_DIR))
        .map(|(_, text)| text)
        .collect::<Vec<_>>()
        .join("\n");

    for needle in MATCHING_ONLY_CRATES {
        assert!(
            all.contains(needle),
            "`core/matching/**` KHÔNG nhắc tới `{needle}`. Cổng cấm ở trên đang canh \
             một chỗ trống: AD-17 đòi MỘT cài đặt, không đòi KHÔNG cài đặt nào. Hai \
             nhánh phải sống — `jieba-rs` cho tiếng Trung, `tantivy-stemmers` cho tiếng \
             Anh."
        );
    }
}

// ═════════════════════════════════════════════════════════════════════════════════
// 🔴 AC2 — `core/dict/**` KHÔNG gọi Matcher
// ═════════════════════════════════════════════════════════════════════════════════

/// 🔴 **AC2** — **0** lời gọi tới `core::matching` từ `core/dict/**`.
#[test]
fn the_dictionary_lookup_path_never_calls_the_matcher() {
    let files = src_sources();

    let dict_files = files
        .iter()
        .filter(|(rel, _)| rel.starts_with("core/dict"))
        .count();
    assert!(
        dict_files >= 1,
        "không tìm thấy tệp `.rs` nào dưới `src/core/dict/**`. Cổng này xanh y hệt trên \
         một thư mục rỗng — nghi phạm: gốc quét sai."
    );

    let mut violations: Vec<String> = Vec::new();
    for (rel, text) in &files {
        if !rel.starts_with("core/dict") {
            continue;
        }
        for (line_no, code) in code_lines(text) {
            for needle in DICT_FORBIDDEN {
                if contains_forbidden_token(code, needle) {
                    violations.push(format!("{rel}:{line_no}  {needle}  |  {code}"));
                }
            }
        }
    }

    assert!(
        violations.is_empty(),
        "{} chỗ dưới `core/dict/**` chạm tới Matcher:\n{}\n\n\
         ══════════════════════════════════════════════════════════════════════════\n\
         🔴 ĐỌC TRƯỚC KHI GỠ CỔNG NÀY — `epics.md:1510` ĐANG LỆCH, KHÔNG PHẢI CỔNG\n\
         ══════════════════════════════════════════════════════════════════════════\n\
         `epics.md:1510` viết cho Story 1.12: *\"And `dict/` dùng nó\"*. Vế đó KHÔNG \
         CÒN ĐÚNG, và chủ sở hữu lượt sửa là John (PM) — xem `deferred-work.md`.\n\n\
         **AD-17, thân Rule (`ARCHITECTURE-SPINE.md:236`)**: *\"AD này nói mọi nơi cần \
         khớp ngôn ngữ dùng chung MỘT cài đặt — nó KHÔNG nói mọi đường đều phải gọi \
         Matcher. Đường tra cứu TỪ ĐIỂN tiếng Anh không gọi […] Glossary (FR51) và TM \
         (FR61) thì CÓ.\"*\n\n\
         **AD-44 ③ (`ARCHITECTURE-SPINE.md:604-618`)**, dữ kiện mạnh: corpus đã có sẵn \
         MỌI dạng biến thể làm đầu mục riêng — **16/16** mẫu thử, gồm cả bất quy tắc. Một \
         lượt stemming trên đường nóng đổi p95 **0,052–0,961 ms** (đo thật ở Story 1.11b) \
         lấy **~0 recall**. NFR1 cho backend ≤ 10 ms.\n\n\
         ⚠️ Sơ đồ mermaid của AD-13 (`ARCHITECTURE-SPINE.md:189`) còn cạnh \
         `dict --> matching`. Nó vẽ TRƯỚC lượt sửa Rule của AD-17 và nay mâu thuẫn với \
         chính thân Rule ở `:236`. Chủ sở hữu: Winston. **Theo thân Rule, không theo \
         mũi tên.**",
        violations.len(),
        violations.join("\n")
    );
}

// ═════════════════════════════════════════════════════════════════════════════════
// AC3 — `core/matching/` là LÁ trong đồ thị phụ thuộc (AD-13)
// ═════════════════════════════════════════════════════════════════════════════════

/// **AC3** — module không phụ thuộc một module miền nào, không `ports`, không
/// `commands`.
///
/// 🔴 Đặc biệt **không** phụ thuộc `ai/`: AD-13 nói *"không module nào ngoài `ai/`
/// được phụ thuộc `ai/`"*.
#[test]
fn the_matching_module_is_a_leaf_in_the_dependency_graph() {
    let files = src_sources();

    let mut violations: Vec<String> = Vec::new();
    for (rel, text) in &files {
        if !rel.starts_with(MATCHING_DIR) {
            continue;
        }
        for (line_no, code) in code_lines(text) {
            for needle in MATCHING_FORBIDDEN_USES {
                if contains_forbidden_token(code, needle) {
                    violations.push(format!("{rel}:{line_no}  {needle}  |  {code}"));
                }
            }
        }
    }

    assert!(
        violations.is_empty(),
        "{} chỗ dưới `core/matching/**` phụ thuộc ra ngoài:\n{}\n\n\
         AD-13: `core/matching/` là LÁ. Nó nhận `&str` + `MatchLang` và trả dữ liệu \
         thuần, nên nó dùng được từ `core::glossary` VÀ `core::tm` mà không cần một \
         lớp bọc nào (AC2). Một phụ thuộc ngược lên một module miền là đảo chiều mũi tên \
         — và với `ai/` thì AD-13 nói thẳng: *\"không module nào ngoài `ai/` được phụ \
         thuộc `ai/`\"*.\n\n\
         Module này cũng KHÔNG chạm filesystem, không chạm database, không ra \
         mạng (AD-15: đúng ba điểm ra mạng, không có điểm thứ tư).",
        violations.len(),
        violations.join("\n")
    );
}

// ═════════════════════════════════════════════════════════════════════════════════
// AC4 — ngôn ngữ là THAM SỐ, không đoán từ nội dung
// ═════════════════════════════════════════════════════════════════════════════════

/// 🔴 **AC4** — **không tồn tại** một vị từ dò script nào trong `core/matching/**`.
///
/// ⚠️ Cổng `exactly_one_definition_of_is_han_exists_under_src_tauri`
/// (`dict_boundary.rs`, Story 1.11b) đã canh riêng `is_han` trên **toàn** `src-tauri/**`.
/// Ca này canh **họ** vị từ còn lại, và nó canh ở **phạm vi module này** — nơi cám dỗ
/// xuất hiện.
///
/// 🔴 Chuỗi cần tìm dựng bằng [`concat!`] chứ không viết liền một mạch: viết liền, tệp
/// này tự khớp chính nó và cổng đỏ ngay ngày nó ra đời — rồi người sửa tiếp theo sẽ gỡ
/// nó bằng một danh sách miễn trừ.
#[test]
fn the_matching_module_never_guesses_the_language_from_the_content() {
    let needles: [String; 4] = [
        concat!("fn ", "is_han").to_string(),
        concat!("fn ", "is_cjk").to_string(),
        concat!("fn ", "detect_lang").to_string(),
        concat!("fn ", "detect_script").to_string(),
    ];

    let files = src_sources();
    let mut violations: Vec<String> = Vec::new();
    for (rel, text) in &files {
        if !rel.starts_with(MATCHING_DIR) {
            continue;
        }
        for (line_no, code) in code_lines(text) {
            for needle in &needles {
                if code.contains(needle.as_str()) {
                    violations.push(format!("{rel}:{line_no}  {needle}  |  {code}"));
                }
            }
            // Một dải Unicode viết cứng là cùng một vị từ, chỉ không có tên hàm.
            if code.contains("\\u{4E00}") || code.contains("\\u{9FFF}") {
                violations.push(format!("{rel}:{line_no}  dai Unicode viet cung  |  {code}"));
            }
        }
    }

    assert!(
        violations.is_empty(),
        "{} chỗ dưới `core/matching/**` tự đoán ngôn ngữ:\n{}\n\n\
         AC4: ngôn ngữ là THAM SỐ từ chỗ gọi (`MatchLang`), KHÔNG đoán từ nội dung.\n\n\
         1. Một định nghĩa `is_han` thứ hai làm cổng \
            `exactly_one_definition_of_is_han_exists_under_src_tauri` \
            (`dict_boundary.rs`) ĐỎ. Đừng nới cổng đó — đừng tạo ra thứ làm nó đỏ.\n\
         2. `MatchLang` KHÔNG phải `core::dict::QueryRoute`. `QueryRoute` trả lời \
            *\"tra vào bảng nào của tệp `.db` nào\"* — một thuộc tính của HÌNH DẠNG CHUỖI \
            TRUY VẤN (AD-44 ①). `MatchLang` trả lời *\"khớp thuật ngữ trong văn bản của \
            MỘT Tác phẩm\"*, và ngôn ngữ nguồn của Tác phẩm là một trường BẤT BIẾN trong \
            `meta.json`, đặt lúc tạo (`prd.md:765-774`). Đoán lại từ nội dung là bỏ đi \
            một dữ kiện đã có và thay bằng một phỏng đoán.\n\
         3. Cùng luật đã đặt ở `core::dict::LookupMode`: *\"chế độ do CHỖ GỌI quyết, không \
            không đoán từ nội dung\"*.",
        violations.len(),
        violations.join("\n")
    );
}

// ═════════════════════════════════════════════════════════════════════════════════
// AC9 — `Jieba` khởi tạo ĐÚNG MỘT LẦN
// ═════════════════════════════════════════════════════════════════════════════════

/// 🔴 **AC9** — chuỗi khởi tạo `Jieba` xuất hiện ở **đúng một** vị trí mã dưới
/// `src-tauri/src/**`.
///
/// ⚠️ Cổng đếm **vị trí mã**, không đếm tệp: hai lời gọi trong cùng một tệp vẫn là hai
/// lần giải nén 5.071.843 byte.
///
/// 🔴 Chuỗi cần tìm dựng bằng [`concat!`] — xem lý do ở
/// [`the_matching_module_never_guesses_the_language_from_the_content`].
#[test]
fn the_jieba_dictionary_is_constructed_at_exactly_one_place() {
    const NEEDLE: &str = concat!("Jieba", "::", "new");

    let files = src_sources();
    let sites: Vec<String> = files
        .iter()
        .flat_map(|(rel, text)| {
            code_lines(text)
                .filter(|(_, code)| code.contains(NEEDLE))
                .map(move |(line_no, code)| format!("{rel}:{line_no}  |  {code}"))
                .collect::<Vec<_>>()
        })
        .collect();

    assert_eq!(
        sites.len(),
        1,
        "{} vị trí mã dựng `Jieba` dưới `src-tauri/src/**`, chờ ĐÚNG MỘT:\n{}\n\n\
         Feature `default-dict` nhúng `dict.txt` — **5.071.843 byte thô** — qua \
         `include_flate::flate!`. Dựng instance là GIẢI NÉN cộng nạp từng dòng vào một \
         cây `cedar`; đó không phải một hằng số biên dịch mà là công việc chạy lúc \
         chạy.\n\n\
         Một lời gọi nằm trong thân một hàm bị gọi lặp là một hồi quy NFR2 (không frame \
         nào vượt 50 ms) mà KHÔNG TEST NÀO THẤY: test chạy một lần, người dùng gõ một \
         nghìn lần.\n\n\
         Đường đúng: `static JIEBA: LazyLock<Jieba>` trong `core/matching/mod.rs`.",
        sites.len(),
        sites.join("\n")
    );

    assert!(
        sites[0].starts_with(MATCHING_DIR),
        "điểm khởi tạo `Jieba` nằm ở {} — chờ nó ở dưới `{MATCHING_DIR}/`",
        sites[0]
    );
}

/// **AC9 vế `LazyLock`** — đối chứng dương: điểm khởi tạo duy nhất **thật sự** là một
/// `static` lười, không phải một lời gọi trần trong thân hàm.
///
/// ⚠️ Không có ca này thì cổng trên vẫn xanh khi ai đó chuyển lời gọi duy nhất đó
/// **vào trong** thân `tokenize` — vẫn "đúng một vị trí mã", và vẫn là một lượt giải nén
/// mỗi lần gõ phím.
#[test]
fn the_single_jieba_instance_is_actually_lazily_initialised_once() {
    let all: String = src_sources()
        .into_iter()
        .filter(|(rel, _)| rel.starts_with(MATCHING_DIR))
        .map(|(_, text)| text)
        .collect::<Vec<_>>()
        .join("\n");

    for needle in ["LazyLock", "static JIEBA"] {
        assert!(
            all.contains(needle),
            "`core/matching/**` KHÔNG chứa `{needle}`. Cổng \
             `the_jieba_dictionary_is_constructed_at_exactly_one_place` đếm vị trí mã và \
             vẫn xanh khi lời gọi DUY NHẤT đó nằm trong thân một hàm bị gọi lặp — tức vẫn \
             một lượt giải nén 5.071.843 byte MỖI LẦN gõ phím. Điểm khởi tạo phải là một \
             `static` lười."
        );
    }
}

// ═════════════════════════════════════════════════════════════════════════════════
// AC10 — hâm nóng Jieba (Story 3.4, `deferred-work.md:413`) mắc THẬT vào đường mở Chương
// ═════════════════════════════════════════════════════════════════════════════════
//
// ⚠️ Tệp riêng có chủ ý vẫn đúng: đây là phép kiểm TĨNH trên cây nguồn (đúng vai của
// `matching_boundary.rs`), không phải hành vi lúc chạy — và nó tiếp nối đúng mạch AC9 ngay
// trên: AC9 giữ cho chi phí khởi tạo `Jieba` (179–329 ms bản release) không NHÂN LÊN; AC10
// giữ cho lượt HÂM chi phí đó ra khỏi đường gõ không bị XOÁ hay DỜI đi trong im lặng.
//
// 🔴 VÌ SAO CỔNG NÀY, KHÔNG PHẢI MỘT LƯỢT ĐỌC BẰNG MẮT — đo được, không suy luận: `grep -rn
// "warm_jieba_for_source_lang\|core::matching::warm" src-tauri/tests` = 0 kết quả trước cổng
// này, và xoá thẳng dòng gọi `warm_jieba_for_source_lang(...)` khỏi `read_open_chapter`
// (`commands/chapter.rs`) để `project_contract.rs`/`segment_contract.rs` chạy `cargo test
// --locked` — TOÀN BỘ vẫn xanh. Tức chính hồi quy NFR2 mà Story 3.4 dựng ra để chặn (khởi
// tạo lạnh 179–329 ms rơi đúng phím đầu người dùng gõ) bò lại được mà không một cổng nào đỏ.

/// Tệp sở hữu hai đường mở Chương mà lượt hâm phải mắc vào.
const CHAPTER_COMMAND_FILE: &str = "commands/chapter.rs";

/// Chuỗi cần tìm — hàm hâm nóng `Jieba` của `core::glossary` (`deferred-work.md:413`).
const WARM_NEEDLE: &str = "warm_jieba_for_source_lang";

/// Lát THÂN của một hàm top-level `pub fn NAME(` trong `text`, tính từ chữ ký của nó tới chữ
/// ký `pub fn`/`pub(crate) fn`/`pub mod` TIẾP THEO ở cột 0 (hoặc hết tệp).
///
/// ⚠️ Đủ thô để không cần một bộ phân tích cú pháp Rust thật, và đủ chặt để phân biệt
/// `read_open_chapter` với `open_adjacent_chapter` trong CÙNG một tệp — cả hai đều là hàm
/// top-level, không lồng nhau, đúng hình dạng `commands/chapter.rs` thật sự có.
fn top_level_fn_body<'a>(text: &'a str, name: &str) -> &'a str {
    let sig = format!("pub fn {name}(");
    let start = text
        .find(&sig)
        .unwrap_or_else(|| panic!("khong tim thay `{sig}` -- ham da doi ten hay bi xoa?"));
    let after = start + sig.len();
    let next_boundary = ["\npub fn ", "\npub(crate) fn ", "\npub mod "]
        .iter()
        .filter_map(|marker| text[after..].find(marker).map(|i| after + i))
        .min()
        .unwrap_or(text.len());
    &text[start..next_boundary]
}

/// `text` (thân một hàm) THẬT SỰ gọi [`WARM_NEEDLE`] ở vị trí MÃ, không phải trong comment.
fn fn_body_calls_warm(body: &str) -> bool {
    code_lines(body).any(|(_, code)| code.contains(WARM_NEEDLE))
}

/// 🔴 **AC10** — cả `read_open_chapter` lẫn `open_adjacent_chapter` đều gọi
/// `warm_jieba_for_source_lang` — hai điểm sản phẩm DUY NHẤT đưa một `source_lang` mới lên
/// webview (`commands/chapter.rs`).
///
/// ─────────────────────────────────────────────────────────────────────────────
/// ⚠️ GIỚI HẠN THẬT — ĐỌC TRƯỚC KHI COI CỔNG NÀY LÀ ĐỦ
/// ─────────────────────────────────────────────────────────────────────────────
/// Cổng này bắt được lời gọi bị **XOÁ** hoặc **DỜI** khỏi hai hàm dưới đây — nó **KHÔNG**
/// chứng minh lượt hâm **thật sự có tác dụng** (không đo thời gian: một ngưỡng mili-giây sẽ
/// chập chờn trên một runner CI đang tải, đúng lý do các phép kiểm khác trong kho tránh
/// assert theo thời gian). Bằng chứng "có tác dụng" là số đo TAY ở `deferred-work.md:413`
/// (179–329 ms khởi tạo lạnh, ~1 µs lượt gọi ấm kế tiếp), không phải một cổng tự động.
#[test]
fn warm_jieba_for_source_lang_is_called_from_both_chapter_opening_functions() {
    let files = src_sources();
    let chapter_source = files
        .iter()
        .find(|(rel, _)| rel == CHAPTER_COMMAND_FILE)
        .map(|(_, text)| text.as_str())
        .unwrap_or_else(|| {
            panic!("khong tim thay `{CHAPTER_COMMAND_FILE}` -- cay nguon da bi cat mat")
        });

    for func in ["read_open_chapter", "open_adjacent_chapter"] {
        let body = top_level_fn_body(chapter_source, func);
        assert!(
            fn_body_calls_warm(body),
            "`{func}` ({CHAPTER_COMMAND_FILE}) KHÔNG còn gọi `{WARM_NEEDLE}` -- đây chính là \
             đường MỞ CHƯƠNG mà `deferred-work.md:413` đòi lượt hâm `Jieba` mắc vào. Xoá hoặc \
             dời lời gọi này ra khỏi hàm là mở lại đúng hồi quy NFR2 mà Story 3.4 dựng ra để \
             chặn: khởi tạo lạnh 179–329 ms rơi đúng vào phím đầu tiên người dùng gõ.\n\n\
             GIỚI HẠN THẬT của cổng này: nó bắt lời gọi bị XOÁ/DỜI, nó KHÔNG chứng minh lượt \
             hâm thật sự có tác dụng — xem số đo tay ở `deferred-work.md:413`."
        );
    }
}

/// **AC10 — đối chứng dương.** Chứng minh vị từ [`fn_body_calls_warm`] mà cổng thật ở trên
/// gọi thật sự ĐỎ ĐƯỢC khi lời gọi biến mất, trên hai đầu vào TỔNG HỢP — không mượn cây
/// nguồn thật (không có cách nào "xoá tạm" một dòng của `commands/chapter.rs` trong một
/// `#[test]` mà không ghi đè tệp trên đĩa).
///
/// ⚠️ Không có ca này thì cổng thật ở trên xanh y hệt trên một vị từ luôn trả `true` — *"có
/// lời gọi"* và *"vị từ hỏng, luôn nói có"* đọc giống hệt nhau trên một cây nguồn hôm nay vẫn
/// sạch.
#[test]
fn the_warm_jieba_check_would_actually_flag_a_removed_call() {
    let with_call = "pub fn read_open_chapter(open: Option<&OpenWork>) -> Result<OpenChapter, IpcError> {\n    \
        let open = open.ok_or_else(no_work_open)?;\n    \
        crate::core::glossary::warm_jieba_for_source_lang(&open.meta.source_lang);\n    \
        Ok(OpenChapter {})\n}\n\n\
        pub fn open_adjacent_chapter(open: Option<&mut OpenWork>) -> Result<ChapterSwitch, IpcError> {\n    \
        Ok(ChapterSwitch {})\n}\n";
    assert!(
        fn_body_calls_warm(top_level_fn_body(with_call, "read_open_chapter")),
        "ca DUONG THAT: mot loi goi that phai duoc vi tu nhan ra"
    );

    let without_call = "pub fn read_open_chapter(open: Option<&OpenWork>) -> Result<OpenChapter, IpcError> {\n    \
        let open = open.ok_or_else(no_work_open)?;\n    \
        Ok(OpenChapter {})\n}\n\n\
        pub fn open_adjacent_chapter(open: Option<&mut OpenWork>) -> Result<ChapterSwitch, IpcError> {\n    \
        Ok(ChapterSwitch {})\n}\n";
    assert!(
        !fn_body_calls_warm(top_level_fn_body(without_call, "read_open_chapter")),
        "ca AM: mot loi goi da bi XOA phai lam vi tu tra ve false -- day chinh la hinh dang ma \
         cong that se do neu ai xoa loi goi khoi read_open_chapter"
    );

    // Doi chung them: `top_level_fn_body` phai CAT DUNG tai `open_adjacent_chapter`, khong
    // "nuot" ca no vao than `read_open_chapter` -- neu khong, mot loi goi CHI nam trong
    // `open_adjacent_chapter` se lam ca tren do OAN cho `read_open_chapter`.
    let call_only_in_second_fn = "pub fn read_open_chapter(open: Option<&OpenWork>) -> Result<OpenChapter, IpcError> {\n    \
        Ok(OpenChapter {})\n}\n\n\
        pub fn open_adjacent_chapter(open: Option<&mut OpenWork>) -> Result<ChapterSwitch, IpcError> {\n    \
        crate::core::glossary::warm_jieba_for_source_lang(&open.meta.source_lang);\n    \
        Ok(ChapterSwitch {})\n}\n";
    assert!(
        !fn_body_calls_warm(top_level_fn_body(call_only_in_second_fn, "read_open_chapter")),
        "lat than cua read_open_chapter khong duoc tran sang open_adjacent_chapter"
    );
    assert!(
        fn_body_calls_warm(top_level_fn_body(call_only_in_second_fn, "open_adjacent_chapter")),
        "loi goi trong open_adjacent_chapter phai duoc chinh ham do nhan ra"
    );
}
