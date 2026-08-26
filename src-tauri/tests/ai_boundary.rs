//! Ranh giới cây nguồn của AD-13 — **không module nào ngoài `core/ai/` được phụ thuộc
//! `core/ai/`** — cưỡng chế cho Story 4.1.
//!
//! ⚠️ Tệp riêng có chủ ý, đúng khuôn `scope_boundary.rs`/`matching_boundary.rs`/
//! `glossary_boundary.rs`: đây là phép kiểm **tĩnh trên cây nguồn**. `core/ai/` hôm nay
//! không có `*_contract.rs` nào để trộn vào — chưa có hành vi lúc chạy nào để nghiệm thu.
//!
//! ─────────────────────────────────────────────────────────────────────────────
//! 🔴 VÌ SAO CỔNG NÀY ĐỨNG TRƯỚC KHI `core/ai/` CÓ MỘT DÒNG MÃ
//! ─────────────────────────────────────────────────────────────────────────────
//! `core/ai/mod.rs` (10 dòng, 100% doc-comment) tự khai: *"Test cưỡng chế ranh giới này
//! thuộc Story 4.1."* `project-context.md` đã phải sửa mệnh đề *"Có test cưỡng chế"* thành
//! *"chưa có"* ngày 2026-08-19. AD-13 là điều kiện để FR77 (gỡ sạch cấu hình AI thì mọi
//! năng lực khác vẫn chạy đầy đủ) không thoái hoá thành kỷ luật cá nhân — và một ranh giới
//! kiến trúc dựng SAU khi 32 story của Epic 4 đã viết xong là một ranh giới không còn gì để
//! canh: mọi vi phạm khả dĩ đã kịp mọc rễ. Đó là lý do Story 4.1 chạy ở thứ tự 3½ (ngay sau
//! Epic 3), tách khỏi phần còn lại của Epic 4.
//!
//! ─────────────────────────────────────────────────────────────────────────────
//! 🔴 VÌ SAO ĐỐI CHỨNG DƯƠNG Ở ĐÂY LÀ CA GIEO VI PHẠM TỔNG HỢP, KHÔNG PHẢI KHUÔN
//! `scope_boundary.rs`/`matching_boundary.rs::the_matching_module_actually_uses_...`
//! ─────────────────────────────────────────────────────────────────────────────
//! Khuôn đối chứng dương quen thuộc của bốn tệp `*_boundary.rs` kia khẳng định *"module chủ
//! THẬT SỰ mang từ vựng của nó"* (`core::scope` thật sự gõ `ScopeKind`, `core::matching`
//! thật sự dùng `jieba_rs`/`tantivy_stemmers`, …) — nó chứng minh phép quét không đang canh
//! một chỗ trống. Khuôn đó **dựng không được** ở đây: `core/ai/` có **0 dòng mã**, không một
//! từ vựng nào tồn tại để khẳng định "có thật".
//!
//! Thay vào đó, đối chứng dương ở đây đi theo khuôn THỨ HAI mà chính kho đã có tiền lệ —
//! `matching_boundary.rs::the_warm_jieba_check_would_actually_flag_a_removed_call` và
//! `glossary_boundary.rs::the_non_manual_origin_token_check_catches_term_origin_but_not_candidate_origin`:
//! gọi thẳng vị từ quét trên một **chuỗi dựng tay**, độc lập với cây nguồn hôm nay có gì. Một
//! chuỗi vi phạm phải bị bắt; một chuỗi sạch phải không bị bắt. Đây chính là thứ phân biệt
//! *"không ai vi phạm"* (cây sạch, nhưng phép quét có thể đang mù) với *"không có gì để vi
//! phạm"* (module rỗng, phép quét chưa từng phải chứng minh nó thấy được gì) — hai mệnh đề
//! đọc giống hệt nhau trên `core/ai/` hôm nay nếu thiếu ca này.
//!
//! ─────────────────────────────────────────────────────────────────────────────
//! ⚠️ ĐIỂM MÙ CÓ TÊN — `core/mod.rs` khai `pub mod ai;` TRẦN, không re-export
//! ─────────────────────────────────────────────────────────────────────────────
//! Phép quét bên dưới bắt chuỗi `crate::core::ai` và `super::ai`. Nếu một ngày `core/mod.rs`
//! thêm `pub use ai::Foo;`, module khác viết được `crate::core::Foo` mà KHÔNG đánh vần `ai`
//! một lần nào — cổng này xanh trên một AD-13 đã bị phá. `core_mod_rs_declares_the_ai_module_bare_with_no_reexport`
//! dưới đây khoá lại hình dạng trần hôm nay và ghi rõ điểm mù này; món nợ có chủ nằm ở
//! `deferred-work.md` (chủ: Story 4.2).
//!
//! ─────────────────────────────────────────────────────────────────────────────
//! 🔴 SÀN SỐ TỆP LÀ BẮT BUỘC — "cây rỗng đọc thành sạch"
//! ─────────────────────────────────────────────────────────────────────────────
//! Bài học kế thừa từ `scope_boundary.rs`/`matching_boundary.rs`/`glossary_boundary.rs`: một
//! gốc quét sai hay một thư mục bị cắt làm `walk` khớp 0 tệp, và khi đó MỌI phép kiểm dưới
//! đây xanh mà không kiểm gì cả — kể cả phép kiểm ranh giới thật.

use std::fs;
use std::path::{Path, PathBuf};

/// Thư mục DUY NHẤT được phép mang từ vựng của module `ai/` (AD-13).
const AI_DIR: &str = "core/ai";

/// Số tệp `.rs` tối thiểu dưới `core/ai/**` để phép quét là thật.
///
/// Số thật hôm nay: **1** (`mod.rs`, stub 10 dòng doc-comment). Sàn = số thật, không phải
/// một tỉ lệ dưới nó — không có chỗ nào để "cắt bớt mà vẫn còn tệp" khi quần thể chỉ có một
/// phần tử; sàn 1 vẫn bắt đúng ca `walk` khớp 0 tệp (gốc quét sai / thư mục bị xoá).
const AI_FLOOR: usize = 1;

/// Số tệp `.rs` tối thiểu dưới `src-tauri/src/**` để phép đếm toàn cây là thật.
///
/// Số thật lúc dựng story này (2026-08-26): **55** tệp. Bốn tệp `*_boundary.rs` cũ (dựng ở
/// các story trước) còn ghi **53** trong hằng số riêng của chúng — số đã trôi qua các lượt
/// thêm tệp không ai nâng lại sàn cũ, đúng bài học mà chính các tệp đó đã ghi lại. Sàn ở đây
/// đo LẠI, không chép: **44** (80%, dưới khuôn 80–85% mà `scope_boundary.rs`/
/// `matching_boundary.rs`/`glossary_boundary.rs` đã dùng) — bắt một cây bị cắt mất, không
/// bắt việc thêm tệp mới.
const SRC_RS_FLOOR: usize = 44;

/// Hai chuỗi BARE (không tiền tố `use `) mà chỉ `core/ai/**` được phép mang ở **vị trí mã**.
///
/// 🔴 Dạng TRẦN là bắt buộc — đúng khuyết tật `MATCHING_FORBIDDEN_USES`
/// (`matching_boundary.rs:75-87`) đã bị bắt ở một lượt review trước: một bản chỉ so
/// `"use crate::core::ai"` bỏ lọt một lời gọi đủ điều kiện viết THẲNG trong thân hàm
/// (`crate::core::ai::foo()`, không qua `use`). `"super::ai"` phủ một module `core/*` khác
/// gọi sang bằng đường tương đối (`super::ai::foo()`), thứ mà một token chỉ neo ở `crate::`
/// sẽ bỏ lọt.
const FORBIDDEN_BARE_TOKENS: [&str; 2] = ["crate::core::ai", "super::ai"];

fn src_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src")
}

/// Đường dẫn tương đối, dùng dấu `/` trên cả hai nền tảng.
///
/// ⚠️ Chuẩn hoá `\` thành `/` là bắt buộc chứ không phải làm đẹp — cùng bài học NFR14 mà
/// `scope_boundary.rs::rel_posix`/`matching_boundary.rs::rel_posix` đã ghi: `starts_with(AI_DIR)`
/// trên Windows so với `core\ai` và KHÔNG BAO GIỜ khớp, nên miễn trừ biến mất và chính
/// `core/ai/mod.rs` tự tố cáo mình là vi phạm — một test đỏ chỉ trên MỘT nhánh của ma trận CI.
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
        // kết trỏ về thư mục cha làm đệ quy không dừng. Cùng bài học các tệp `*_boundary.rs`
        // khác đã ghi.
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

/// Mọi tệp `.rs` dưới `src-tauri/src/**`, kèm đường dẫn tương đối kiểu POSIX và nội dung.
fn all_rust_sources() -> Vec<(String, String)> {
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

/// Dòng **mã** của một khối văn bản: `(số dòng 1-based, nội dung đã trim đầu)`.
///
/// ⚠️ Chỉ dòng bắt đầu bằng `//` được bỏ qua, đúng luật mà mọi tệp `*_boundary.rs` khác áp:
/// một doc-comment GIẢI THÍCH một ranh giới không phải một lời gọi vượt qua nó. Nhận `&str`
/// thay vì một đường dẫn tệp (khác `scope_boundary.rs`, giống `matching_boundary.rs`) có chủ
/// ý: đây là hàm DUY NHẤT mà cả phép quét cây thật LẪN ca gieo vi phạm tổng hợp bên dưới
/// cùng gọi — hai bên không thể lệch nhau bằng cách tự lặp lại logic đọc-dòng ở hai chỗ.
///
/// **Comment đuôi dòng vẫn bị bắt** — phần mã vẫn ở đầu dòng.
fn code_lines(text: &str) -> impl Iterator<Item = (usize, &str)> {
    text.lines()
        .enumerate()
        .map(|(index, line)| (index + 1, line.trim_start()))
        .filter(|(_, code)| !code.starts_with("//"))
}

/// `code` mang một trong hai [`FORBIDDEN_BARE_TOKENS`] — vị từ THUẦN, dùng bởi CẢ cổng thật
/// lẫn ca gieo vi phạm tổng hợp. Trả về token khớp đầu tiên, hoặc `None`.
///
/// Tách ra khỏi thân test — đúng khuôn `glossary_boundary.rs::line_calls_a_glossary_only_surface_function`
/// — để hai bên không thể lệch nhau bằng cách trùng lặp phép so chuỗi ở hai chỗ khác nhau.
/// 🔴 **Khớp có NEO BIÊN, không phải `contains` trần** — vòng rà 1 (2026-08-26) bắt được:
/// bản đầu dùng `code.contains(needle)`, nên một module tương lai tên `core::aiven` hay
/// `super::aisle` bị báo ĐỎ OAN (chuỗi `crate::core::ai` là tiền tố thật của
/// `crate::core::aiven`). Đây đúng lớp lỗi mà `matching_boundary.rs::contains_forbidden_token`
/// + `is_word_byte` tồn tại để chống, và Code Map của story đã trỏ vào đó — bản đầu chỉ mượn
/// bài học "quét BARE token" mà bỏ quên bài học "neo biên" nằm ngay cạnh.
///
/// Ký tự ngay sau token phải KHÔNG phải ký tự định danh (`[A-Za-z0-9_]`). Hết dòng cũng là
/// một biên hợp lệ (`use crate::core::ai;` → ký tự kế là `;`).
///
/// ⚠️ **GIỚI HẠN THẬT:** không neo biên ĐẦU token, và không cần — cả hai token đã tự mang
/// một tiền tố đủ hiếm (`crate::core::` / `super::`); neo đầu sẽ bỏ lọt một lời gọi nằm giữa
/// biểu thức (`foo(crate::core::ai::bar())`).
fn line_names_a_forbidden_ai_dependency(code: &str) -> Option<&'static str> {
    FORBIDDEN_BARE_TOKENS.into_iter().find(|needle| {
        code.match_indices(needle).any(|(at, _)| {
            code[at + needle.len()..]
                .chars()
                .next()
                .is_none_or(|c| !c.is_ascii_alphanumeric() && c != '_')
        })
    })
}

/// 🔴 `rel` nằm TRONG `core/ai/**` — khớp theo **biên thư mục**, không theo tiền tố chuỗi.
///
/// Vòng rà 1 (2026-08-26) bắt được, và đây là phát hiện nặng nhất của cả lượt vì nó là một
/// **XANH GIẢ**, không phải một đỏ oan: bản đầu dùng `rel.starts_with(AI_DIR)`, nên
/// `core/aim/mod.rs` · `core/ai_providers/foo.rs` · `core/ai2/x.rs` đều được **miễn trừ im
/// lặng** khỏi cổng ranh giới. Một module như thế viết `use crate::core::ai;` thì cổng vẫn
/// xanh trên một AD-13 đã bị phá — và trong đúng một epic tên `ai`, một thư mục anh em bắt
/// đầu bằng "ai" là chuyện dễ xảy ra, không phải một khả năng lý thuyết.
fn is_inside_ai_module(rel: &str) -> bool {
    rel == AI_DIR || rel.starts_with(&format!("{AI_DIR}/"))
}

/// Phần CÂU LỆNH của một dòng — bỏ chú thích đuôi dòng.
///
/// ⚠️ Cắt thô ở `//` đầu tiên: đủ cho `core/mod.rs` (một tệp chỉ chứa khai báo `pub mod`,
/// không chuỗi literal nào mang `//`), và giới hạn đó ghi ra ở đây thay vì để người sau
/// tưởng nó là một parser.
fn statement_of(code: &str) -> &str {
    code.split("//").next().unwrap_or(code).trim()
}

/// Câu lệnh re-export một thứ gì đó TỪ module `ai` — neo theo ĐOẠN định danh, không theo
/// chuỗi con. Xem chú thích tại chỗ gọi để biết vì sao.
fn statement_reexports_the_ai_module(stmt: &str) -> bool {
    stmt.contains("pub use")
        && stmt
            .split(|c: char| !c.is_ascii_alphanumeric() && c != '_')
            .any(|seg| seg == "ai")
}

// ═════════════════════════════════════════════════════════════════════════════════
// Sàn quần thể — chạy TRƯỚC mọi phép kiểm khác. Xem doc-comment đầu tệp.
// ═════════════════════════════════════════════════════════════════════════════════

/// Xem [`SRC_RS_FLOOR`] và [`AI_FLOOR`].
#[test]
fn the_scanned_tree_and_the_ai_module_are_both_large_enough_to_be_real() {
    let files = all_rust_sources();

    assert!(
        files.len() >= SRC_RS_FLOOR,
        "chỉ tìm thấy {} tệp `.rs` dưới `src-tauri/src/**` (sàn {SRC_RS_FLOOR}). Cây quá nhỏ \
         để là thật — một danh sách rỗng làm mọi phép kiểm dưới đây xanh mà không kiểm gì \
         cả. Nghi phạm: gốc quét sai, hoặc một thư mục bị bỏ.",
        files.len()
    );

    let ai_files = files.iter().filter(|(rel, _)| is_inside_ai_module(rel)).count();
    assert!(
        ai_files >= AI_FLOOR,
        "chỉ tìm thấy {ai_files} tệp `.rs` dưới `src/{AI_DIR}/**` (sàn {AI_FLOOR}). Một \
         đường dẫn gõ sai làm `walk` khớp 0 tệp, và khi đó cổng ranh giới bên dưới xanh y \
         hệt trên một thư mục RỖNG — \"không ai vi phạm\" và \"không có gốc quét\" đọc giống \
         nhau."
    );
}

// ═════════════════════════════════════════════════════════════════════════════════
// 🔴 AD-13 — không tệp nào ngoài `core/ai/**` được gõ một token phụ thuộc BARE
// ═════════════════════════════════════════════════════════════════════════════════

/// 🔴 Cổng THẬT — quét toàn cây `src-tauri/src/**` trừ `core/ai/**`, tìm hai token bare.
#[test]
fn no_file_outside_core_ai_names_a_bare_dependency_on_the_ai_module() {
    let files = all_rust_sources();

    let mut violations: Vec<String> = Vec::new();
    let mut ai_files = 0usize;

    for (rel, text) in &files {
        if is_inside_ai_module(rel) {
            ai_files += 1;
            continue;
        }
        for (line, code) in code_lines(text) {
            if let Some(needle) = line_names_a_forbidden_ai_dependency(code) {
                violations.push(format!("{rel}:{line}  {needle}  |  {code}"));
            }
        }
    }

    // Miễn trừ phải khớp thứ gì đó — cùng đối chứng mà `scope_boundary.rs`/`glossary_boundary.rs`
    // đòi: một `AI_DIR` gõ sai làm nhánh `continue` không bao giờ chạy, phép kiểm vẫn xanh
    // hôm nay (mã ngoài `core/ai/` sạch thật), rồi đỏ sai chỗ vào ngày ai đó đổi tên thư mục.
    assert!(
        ai_files > 0,
        "không tệp nào khớp `{AI_DIR}` — đường dẫn miễn trừ đã lệch khỏi cây nguồn"
    );

    assert!(
        violations.is_empty(),
        "{} chỗ NGOÀI `core/ai/**` gõ một token phụ thuộc BARE vào module `ai`:\n{}\n\n\
         AD-13: KHÔNG module nào ngoài `ai/` được phụ thuộc `ai/`. Đây là điều kiện để FR77 \
         (gỡ sạch cấu hình AI thì mọi năng lực khác vẫn chạy đầy đủ) không thoái hoá thành \
         kỷ luật cá nhân. Nếu đây là một vi phạm THẬT đang tồn tại trong cây — DỪNG LẠI, đây \
         là quyết định phạm vi của Ice, không phải một lượt vá tiện tay.",
        violations.len(),
        violations.join("\n")
    );
}

// ═════════════════════════════════════════════════════════════════════════════════
// 🔴 Đối chứng dương — CA GIEO VI PHẠM TỔNG HỢP (bắt buộc, xem Design Notes đầu tệp)
// ═════════════════════════════════════════════════════════════════════════════════

/// 🔴 Chứng minh [`line_names_a_forbidden_ai_dependency`] — vị từ mà cổng thật ở trên gọi —
/// NỔ ĐƯỢC trên một chuỗi vi phạm và KHÔNG nổ trên một chuỗi sạch, độc lập với việc cây
/// nguồn hôm nay có gì.
///
/// ⚠️ Không có ca này thì cổng thật ở trên xanh y hệt trên một vị từ luôn trả `None` — "không
/// ai vi phạm" và "vị từ hỏng, luôn nói không có" đọc giống hệt nhau trên một cây mà `core/ai/`
/// hôm nay chưa có consumer thật nào để tự làm chứng.
#[test]
fn the_bare_dependency_check_would_actually_flag_a_seeded_violation_and_ignore_clean_code() {
    // Ca DƯƠNG THẬT thứ nhất — `use crate::core::ai;` ở một module khác.
    assert_eq!(
        line_names_a_forbidden_ai_dependency("use crate::core::ai;"),
        Some("crate::core::ai"),
        "ca DUONG THAT: `use crate::core::ai;` phai bi vi tu bat -- day chinh la hinh dang \
         `use` ma AD-13 cam"
    );

    // Ca DƯƠNG THẬT thứ hai — lời gọi đủ điều kiện viết THẲNG trong thân hàm, không `use`.
    // Đây đúng khuyết tật `MATCHING_FORBIDDEN_USES` đã bị bắt ở một lượt review trước: một
    // bản chỉ so tiền tố `"use "` sẽ bỏ lọt hình dạng này.
    assert_eq!(
        line_names_a_forbidden_ai_dependency("    let p = crate::core::ai::build_prompt();"),
        Some("crate::core::ai"),
        "ca DUONG THAT: mot loi goi BARE trong than ham (khong qua `use`) van phai bi bat"
    );

    // Ca DƯƠNG THẬT thứ ba — `super::ai::foo()` từ một module `core/*` khác, đường tương đối.
    assert_eq!(
        line_names_a_forbidden_ai_dependency("        super::ai::warm_up();"),
        Some("super::ai"),
        "ca DUONG THAT: `super::ai::...` (duong tuong doi tu mot module core/* khac) van \
         phai bi bat"
    );

    // Ca ÂM — mã sạch không nhắc `ai` theo hai hình dạng bị cấm, không được bị bắt oan.
    assert_eq!(
        line_names_a_forbidden_ai_dependency("    let entries = core::glossary::load_tier(g);"),
        None,
        "ca AM: mot dong khong dinh gi den module `ai` khong duoc bi bat oan"
    );
    assert_eq!(
        line_names_a_forbidden_ai_dependency("pub mod ai;"),
        None,
        "ca AM: khai bao `pub mod ai;` o core/mod.rs (khong phai `crate::core::ai` hay \
         `super::ai`) khong duoc bi bat -- day la khai bao HOP LE duy nhat ma AD-13 cho phep"
    );

    // Đối chứng thêm: `code_lines` bỏ dòng CHÚ THÍCH — đúng I/O Matrix "Nhắc `core::ai`
    // trong một dòng chú thích ⇒ KHÔNG đỏ". Dựng một khối văn bản TỔNG HỢP nhiều dòng, đi
    // qua chính `code_lines` (không đọc tệp trên đĩa) để chứng minh dòng `//` biến mất khỏi
    // tập dòng-mã trước khi vị từ ở trên có cơ hội nhìn thấy nó.
    let synthetic = "// core::ai sẽ được gọi ở đây khi Epic 4 tới lượt\n\
                      fn stub() {}\n\
                      // crate::core::ai::build_prompt() -- vi du trong comment, khong phai ma\n";
    let code_only: Vec<&str> = code_lines(synthetic).map(|(_, code)| code).collect();
    assert!(
        !code_only.iter().any(|code| line_names_a_forbidden_ai_dependency(code).is_some()),
        "hai dong CHU THICH nhac toi token bi cam khong duoc lot vao tap DONG MA -- \
         `code_lines` phai loai chung TRUOC khi vi tu quet chay, dung I/O Matrix cua spec"
    );
    // Và một chuỗi sạch còn lại sau khi lọc (dòng `fn stub() {}`) đúng là thứ vẫn ở lại.
    assert!(
        code_only.contains(&"fn stub() {}"),
        "dong MA that su (khong phai comment) phai con lai sau `code_lines`"
    );
}

// ═════════════════════════════════════════════════════════════════════════════════
// Điểm mù có tên — `core/mod.rs` khai `ai` TRẦN, không re-export
// ═════════════════════════════════════════════════════════════════════════════════

/// `core/mod.rs` khai `pub mod ai;` TRẦN và KHÔNG re-export gì từ nó.
///
/// ⚠️ Xem doc-comment đầu tệp §ĐIỂM MÙ CÓ TÊN. Ca này khoá lại hình dạng AN TOÀN hôm nay
/// (khai trần, không `pub use ai::…`) — nó KHÔNG chứng minh không ai re-export trong tương
/// lai (một `pub use` mới không nhất thiết đứng ngay cạnh dòng `pub mod ai;`, và một cổng cố
/// quét "không có `pub use ai::` ở bất kỳ đâu trong cả cây" là một cổng khác, chưa dựng —
/// món nợ có chủ ở `deferred-work.md`, chủ Story 4.2).
#[test]
fn core_mod_rs_declares_the_ai_module_bare_with_no_reexport() {
    const CORE_MOD_FILE: &str = "core/mod.rs";

    let files = all_rust_sources();
    let (_, text) = files
        .iter()
        .find(|(rel, _)| rel == CORE_MOD_FILE)
        .unwrap_or_else(|| panic!("khong tim thay `{CORE_MOD_FILE}` -- cay nguon da bi cat mat"));

    let mut found_bare_decl = false;
    for (line, code) in code_lines(text) {
        // ⚠️ Cắt chú thích ĐUÔI DÒNG trước khi so bằng — vòng rà 1 (2026-08-26) bắt được:
        // bản đầu so `code == "pub mod ai;"` nguyên văn, nên một lượt sửa vô hại như
        // `pub mod ai; // AD-13` làm `found_bare_decl` ở lại `false` và ca báo ĐỎ OAN rằng
        // khai bao da bien mat.
        let stmt = statement_of(code);
        if stmt == "pub mod ai;" {
            found_bare_decl = true;
        }
        // ⚠️ Neo theo ĐOẠN đường dẫn, không theo chuỗi con — vòng rà 1 bắt được: bản đầu so
        // `code.contains("ai::")`, nên `pub use domain_ai::Config;` hay `pub use foo::Chai::Bar;`
        // báo ĐỎ OAN (chuỗi `ai::` nằm trong `_ai::` và `Chai::`). Đúng lớp lỗi "thiếu neo
        // biên" mà vị từ token ở trên vừa phải sửa, chỉ ở một bề mặt khác.
        assert!(
            !statement_reexports_the_ai_module(stmt),
            "`{CORE_MOD_FILE}:{line}` re-export tu `ai::` ({code}) -- day chinh la DIEM MU \
             ghi o doc-comment dau tep: mot `pub use ai::Foo;` cho module khac viet duoc \
             `crate::core::Foo` ma khong danh van `ai` mot lan nao, lam cong ranh gioi xanh \
             tren mot AD-13 da bi pha"
        );
    }

    assert!(
        found_bare_decl,
        "`{CORE_MOD_FILE}` khong con khai `pub mod ai;` -- cay da bi cat hoac khai bao da \
         doi hinh dang ma phep kiem nay chua theo kip"
    );
}

// ═════════════════════════════════════════════════════════════════════════════════
// 🔴 NFR14 — miễn trừ `core/ai` phải khớp trên CẢ hình dạng đường dẫn Windows
// ═════════════════════════════════════════════════════════════════════════════════

/// 🔴 [`rel_posix`] chuẩn hoá `\` thành `/`, nên nhánh `rel.starts_with(AI_DIR)` của cổng thật
/// khớp y hệt trên Windows.
///
/// ⚠️ **Vì sao ca này tồn tại thay vì một dòng chú thích.** Máy chạy `pre-push` là macOS, nên
/// hàng *"chạy trên Windows"* của bảng I/O không có phép kiểm nào chạm tới cho tới khi CI dựng
/// job `windows-2025` — tức SAU khi push. Thiếu phép chuẩn hoá, mọi tệp dưới `core/ai/` báo
/// đường dẫn `core\ai\mod.rs`, nhánh miễn trừ **không bao giờ chạy**, và `core/ai/mod.rs` tự tố
/// cáo chính mình: cổng đỏ trên Windows, xanh trên macOS, và không ai đọc ra vì sao.
///
/// Ca này kiểm vị từ trên một đầu vào DỰNG TAY mang hình dạng Windows — cùng kỹ thuật mà
/// `the_bare_dependency_check_would_actually_flag_a_seeded_violation_and_ignore_clean_code`
/// dùng, và vì cùng một lý do: nghiệm thu một mệnh đề mà cây hôm nay không tự làm chứng được.
///
/// ⚠️ **GIỚI HẠN THẬT, ghi ra thay vì để người sau tự phát hiện:** ca này nghiệm thu phép
/// CHUẨN HOÁ, không nghiệm thu `walk` trên một hệ tệp Windows thật. Vế đó vẫn thuộc job
/// `windows-2025` của CI.
#[test]
fn the_core_ai_exemption_still_matches_when_the_path_arrives_windows_shaped() {
    let root = Path::new("/repo/src-tauri/src");

    // Hình dạng Windows: `walk` trả `\` giữa các thành phần.
    let windows_shaped = PathBuf::from("/repo/src-tauri/src").join("core\\ai\\mod.rs");
    let rel = rel_posix(root, &windows_shaped);

    assert_eq!(
        rel, "core/ai/mod.rs",
        "rel_posix phải đổi `\\` thành `/`; nhận được {rel:?}"
    );
    assert!(
        is_inside_ai_module(&rel),
        "đường dẫn hình dạng Windows {rel:?} phải khớp miễn trừ `{AI_DIR}` — nếu không, chính \
         `core/ai/mod.rs` bị cổng đếm là một chỗ vi phạm trên Windows trong khi macOS xanh"
    );

    // Đối chứng ÂM: một thư mục KHÁC không được vô tình khớp miễn trừ.
    let other = PathBuf::from("/repo/src-tauri/src").join("core\\dict\\mod.rs");
    assert!(
        !is_inside_ai_module(&rel_posix(root, &other)),
        "miễn trừ `{AI_DIR}` khớp quá rộng — một module ngoài `ai/` sẽ được tha oan"
    );
}

// ═════════════════════════════════════════════════════════════════════════════════
// 🔴 Vòng rà 1 (2026-08-26) — ba vị từ đều THIẾU NEO BIÊN. Ca dưới đây khoá cả ba.
// ═════════════════════════════════════════════════════════════════════════════════

/// 🔴 Ba vị từ của tệp này neo theo BIÊN, không theo chuỗi con.
///
/// Cả ba khuyết tật do vòng rà đối kháng của Story 4.1 tìm ra, và **hai lớp rà độc lập cùng
/// chỉ vào khuyết tật thứ hai** — nó là ca nặng nhất vì nó là một XANH GIẢ, không phải đỏ oan.
///
/// Không có ca này thì cả ba bản vá đều là mã không ai canh: gỡ neo ra, bộ test cũ vẫn xanh.
#[test]
fn the_three_predicates_anchor_on_boundaries_and_do_not_fire_on_prefix_neighbours() {
    // ── ① Vị từ token: `crate::core::aiven` KHÔNG phải một vi phạm ─────────────────
    assert_eq!(
        line_names_a_forbidden_ai_dependency("use crate::core::ai;"),
        Some("crate::core::ai"),
        "ca dương phải giữ nguyên sau lượt thêm neo biên"
    );
    for hang_xom in [
        "use crate::core::aiven::Client;",
        "let x = crate::core::ai2::foo();",
        "use super::aisle::Row;",
        "use crate::core::ai_providers::Cfg;",
    ] {
        assert_eq!(
            line_names_a_forbidden_ai_dependency(hang_xom),
            None,
            "ĐỎ OAN: {hang_xom:?} là một module HÀNG XÓM tên bắt đầu bằng `ai`, không phải \
             một lời gọi vào `core/ai/`. Một `contains` trần bắt nhầm nó."
        );
    }

    // ── ② Miễn trừ thư mục: XANH GIẢ nếu khớp theo tiền tố chuỗi ──────────────────
    assert!(is_inside_ai_module("core/ai"), "chính thư mục phải được miễn trừ");
    assert!(is_inside_ai_module("core/ai/mod.rs"), "tệp trong `core/ai/` phải được miễn trừ");
    for anh_em in ["core/aim/mod.rs", "core/ai_providers/foo.rs", "core/ai2/x.rs", "core/aix.rs"] {
        assert!(
            !is_inside_ai_module(anh_em),
            "XANH GIẢ: {anh_em:?} là một thư mục ANH EM, không nằm trong `core/ai/`. Miễn trừ \
             theo tiền tố chuỗi sẽ bỏ qua nó khỏi cổng ranh giới, và một `use crate::core::ai;` \
             viết trong đó đi lọt hoàn toàn — cổng xanh trên một AD-13 đã bị phá."
        );
    }

    // ── ③ Vị từ re-export: `domain_ai::` / `Chai::` KHÔNG phải re-export từ `ai` ───
    assert!(
        statement_reexports_the_ai_module("pub use ai::Foo;"),
        "ca dương phải giữ nguyên — đây chính là điểm mù mà ca `core/mod.rs` canh"
    );
    assert!(
        statement_reexports_the_ai_module("pub use self::ai::Foo;"),
        "đường vòng qua `self::` cũng là re-export từ `ai`"
    );
    for vo_can in ["pub use domain_ai::Config;", "pub use foo::Chai::Bar;", "pub use aiven::X;"] {
        assert_eq!(
            statement_reexports_the_ai_module(vo_can),
            false,
            "ĐỎ OAN: {vo_can:?} không re-export gì từ module `ai`; chuỗi `ai` chỉ nằm LỌT \
             trong một định danh dài hơn."
        );
    }

    // ── ④ Cắt chú thích đuôi dòng trước khi so khai báo ───────────────────────────
    assert_eq!(statement_of("pub mod ai; // AD-13"), "pub mod ai;");
    assert_eq!(statement_of("pub mod ai;"), "pub mod ai;");
}
