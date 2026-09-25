//! Ranh giới cây nguồn của Story 4.3 (FR65/FR67/NFR11) — bộ truy cập GIÁ TRỊ THẬT của
//! khoá API (`ApiKeySecret::expose_secret` và `keychain::read`) chỉ được PHÉP GỌI từ bên
//! trong `core/aiconfig/**`.
//!
//! ⚠️ Tệp riêng có chủ ý, đúng khuôn `ai_boundary.rs`/`glossary_boundary.rs`: đây là phép
//! kiểm **tĩnh trên cây nguồn**, và `aiconfig_contract.rs` nghiệm thu **hành vi lúc
//! chạy** — trộn hai thứ làm hỏng đúng thứ khiến cả hai đọc được.
//!
//! ─────────────────────────────────────────────────────────────────────────────
//! VÌ SAO CỔNG NÀY ĐI THEO KHUÔN `ai_boundary.rs`, KHÔNG PHẢI `glossary_boundary.rs`
//! ─────────────────────────────────────────────────────────────────────────────
//! `glossary_boundary.rs::only_entries_eligible_for_injection_may_be_called_from_outside_glossary`
//! có sẵn CHỖ GỌI THẬT trong tree để đối chứng dương "core thật sự khai hàm này" — Story
//! 3.3 đã dựng chỗ gọi sản phẩm đầu tiên trước khi cổng đó ra đời. `expose_secret`/`read`
//! ở đây thì KHÔNG: cả hai đều "dành cho Story 4.8", 0 chỗ gọi nào tồn tại hôm nay ngoài
//! bài test thuần của chính `keychain.rs` — đúng hệt hoàn cảnh `core/ai/` mà
//! `ai_boundary.rs` phải giải quyết (`core/ai/mod.rs` là stub 10 dòng doc-comment, 0 từ
//! vựng để mà đối chứng dương bằng CHỖ GỌI THẬT). Đối chứng dương ở đây vì thế đi theo
//! đúng khuôn `ai_boundary.rs`: một ca gieo vi phạm TỔNG HỢP (chuỗi dựng tay, độc lập
//! với cây nguồn hôm nay có gì) cộng một ca xác nhận ĐỊNH NGHĨA (không phải CHỖ GỌI) của
//! cả hai hàm còn tồn tại dưới `core/aiconfig/**`.
//!
//! ─────────────────────────────────────────────────────────────────────────────
//! ⚠️ GIỚI HẠN THẬT — ghi ra thay vì để người sau tự phát hiện
//! ─────────────────────────────────────────────────────────────────────────────
//! Token `"keychain::read"` bắt được `crate::core::aiconfig::keychain::read(...)` ·
//! `super::keychain::read(...)` · `aiconfig::keychain::read(...)` (mọi hình dạng gọi
//! ĐỦ ĐIỀU KIỆN qua đường dẫn module) VÀ một `use crate::core::aiconfig::keychain::read;`
//! (không cần dấu `(` — câu `use` cũng bị bắt, vì token không neo đuôi `(`).
//! [`line_use_pulls_in_read_via_brace_or_glob`] đóng thêm hai hình dạng `use` mà hai token
//! trần đó KHÔNG bắt được vì chuỗi con `"keychain::read"` không xuất hiện nguyên vẹn trong
//! câu: BRACE (`use …keychain::{read, configured};` — `read` đứng độc lập trong danh sách
//! phẩy, có thể ở bất kỳ vị trí nào) và GLOB (`use …keychain::*;` — kéo `read` vào phạm vi
//! VÔ ĐIỀU KIỆN cùng mọi tên `pub(crate)` khác của module, nên bản thân câu `use` đã là vi
//! phạm, không cần đợi một lời gọi `read(...)` trần xuất hiện sau đó).
//!
//! 🔵 **ĐÓNG 2026-09-21 (spec 4.8, Phase 1, Task 3) — `deferred-work.md:10441`.** Đoạn này
//! từng khẳng định "VẪN KHÔNG bắt được một chỗ gọi đã đặt LẠI TÊN qua `use`
//! (`use …keychain::read as get_secret;` rồi gọi `get_secret()`)". Đo lại, câu đó SAI cho
//! hình dạng BARE mà chính nó nêu làm ví dụ: `"use crate::core::aiconfig::keychain::read as
//! get_secret;"` chứa NGUYÊN VẸN chuỗi con `"keychain::read"` ngay trước khoảng trắng của
//! `as`, và token trần đã có neo biên ĐUÔI (không phải đầu) — nó đã bị bắt từ trước bản sửa
//! này (đối chứng không-hồi-quy:
//! [`the_renamed_import_hole_in_deferred_work_10441_is_closed_for_the_brace_shape`]). Lỗ THẬT
//! nằm ở dạng NGOẶC mà [`line_use_pulls_in_read_via_brace_or_glob`]'s bản trước không đóng:
//! `use …keychain::{read as get_secret, ...};` — vị từ cũ so `item.trim() == "read"`, và
//! `"read as get_secret"` không bằng `"read"`. Sửa: so TOKEN ĐẦU của từng phần tử sau khi
//! tách khoảng trắng, đóng cả hai vị trí trong danh sách. Chủ nợ này giờ đã đóng ở đây, không
//! còn ở Story 4.8's Phase 2/3.
//!
//! ─────────────────────────────────────────────────────────────────────────────
//! 🔴 SÀN SỐ TỆP LÀ BẮT BUỘC — "cây rỗng đọc thành sạch"
//! ─────────────────────────────────────────────────────────────────────────────
//! Bài học kế thừa từ `ai_boundary.rs`/`glossary_boundary.rs`/`scope_boundary.rs`: một
//! gốc quét sai hay một thư mục bị cắt làm `walk` khớp 0 tệp, và khi đó MỌI phép kiểm
//! dưới đây xanh mà không kiểm gì cả — kể cả phép kiểm ranh giới thật.

use std::path::{Path, PathBuf};

#[path = "support/boundary_scan.rs"]
#[allow(dead_code)] // shared module: not every helper is used in this file
mod boundary_scan;
use boundary_scan::{code_lines, rel_posix, src_root};

/// Thư mục DUY NHẤT được phép gọi bộ truy cập giá trị thật của khoá API.
const AICONFIG_DIR: &str = "core/aiconfig";

/// Số tệp `.rs` tối thiểu dưới `core/aiconfig/**` để phép quét là thật.
///
/// Số thật hôm nay (2026-09-17, Story 4.3): **3** (`mod.rs` · `store.rs` · `keychain.rs`).
/// Sàn = số thật, đúng lý lẽ `ai_boundary.rs::AI_FLOOR` (quần thể nhỏ thì không có chỗ
/// "cắt bớt mà vẫn còn tệp").
const AICONFIG_FLOOR: usize = 3;

/// Số tệp `.rs` tối thiểu dưới `src-tauri/src/**` để phép đếm toàn cây là thật.
///
/// Số thật lúc dựng ca này (2026-09-17, Phase 3): **86** tệp (`find src -name '*.rs' |
/// wc -l`). Sàn = **70** (~81,4%), trong dải 80–85% mà `ai_boundary.rs`/
/// `glossary_boundary.rs`/`matching_boundary.rs` đã dùng — bắt một cây bị cắt mất,
/// không bắt việc thêm tệp mới.
const SRC_RS_FLOOR: usize = 70;

/// Hai chuỗi bị cấm ngoài `AICONFIG_DIR`, ở **vị trí mã** — không neo tiền tố `use `,
/// đúng lý lẽ `ai_boundary.rs::FORBIDDEN_BARE_TOKENS`: một bản chỉ so `"use …"` bỏ lọt
/// một lời gọi viết THẲNG trong thân hàm.
///
/// `"expose_secret"` không có dấu `(` — bắt cả `.expose_secret()` (gọi phương thức) lẫn
/// một chỗ VIẾT TÊN nó ra (vd. truyền như một giá trị hàm, `ApiKeySecret::expose_secret`)
/// mà không cần biết cú pháp gọi cụ thể. `"keychain::read"` cũng không có dấu `(` — bắt
/// cả câu `use` đặt tên `read` (xem §GIỚI HẠN THẬT đầu tệp) lẫn một lời gọi đủ điều kiện.
const FORBIDDEN_RAW_VALUE_TOKENS: [&str; 2] = ["expose_secret", "keychain::read"];

/// Mọi tệp `.rs` dưới `src-tauri/src/**`, kèm đường dẫn tương đối kiểu POSIX và nội dung.
fn all_rust_sources() -> Vec<(String, String)> {
    boundary_scan::rust_sources(&src_root())
}

/// `code` mang một trong hai [`FORBIDDEN_RAW_VALUE_TOKENS`] — vị từ THUẦN, dùng bởi CẢ
/// cổng thật lẫn ca gieo vi phạm tổng hợp bên dưới, đúng khuôn
/// `ai_boundary.rs::line_names_a_forbidden_ai_dependency`.
///
/// 🔴 Khớp có NEO BIÊN ĐUÔI — ký tự ngay sau token phải KHÔNG phải ký tự định danh
/// (`[A-Za-z0-9_]`), hoặc hết dòng. Không neo biên thì `expose_secret` cũng khớp một
/// định danh tương lai như `expose_secret_for_debug_only`, và `keychain::read` cũng khớp
/// `keychain::read_only_flag` — cùng lớp lỗi mà `ai_boundary.rs` đã phải sửa ở vòng rà 1
/// của Story 4.1.
///
/// ⚠️ Không neo biên ĐẦU token — cùng lý lẽ `ai_boundary.rs`: cả hai token đã tự mang một
/// hình dạng đủ hiếm; neo đầu sẽ bỏ lọt một lời gọi nằm giữa biểu thức
/// (`foo(secret.expose_secret())`).
fn line_names_a_forbidden_raw_value_access(code: &str) -> Option<&'static str> {
    let bare_token_hit = FORBIDDEN_RAW_VALUE_TOKENS.into_iter().find(|needle| {
        code.match_indices(needle).any(|(at, _)| {
            code[at + needle.len()..]
                .chars()
                .next()
                .is_none_or(|c| !c.is_ascii_alphanumeric() && c != '_')
        })
    });

    bare_token_hit.or_else(|| line_use_pulls_in_read_via_brace_or_glob(code))
}

/// `code` là một câu `use` kéo `read` vào phạm vi qua BRACE (`keychain::{read, ...}` — `read`
/// đứng ĐỘC LẬP, ở bất kỳ vị trí nào trong danh sách phẩy) hoặc GLOB (`keychain::*`) — hai
/// hình dạng mà [`FORBIDDEN_RAW_VALUE_TOKENS`] không bắt được vì chuỗi con `"keychain::read"`
/// không xuất hiện nguyên vẹn ở một trong hai câu đó (xem §GIỚI HẠN THẬT đầu tệp cho lỗ KHÁC
/// còn lại — đặt lại tên qua `as`, chủ Story 4.8, không đóng ở đây).
///
/// GLOB kéo `read` vào phạm vi VÔ ĐIỀU KIỆN — mọi tên `pub(crate)` của module, kể cả `read`,
/// đều theo sau `*` — nên bản thân câu `use …keychain::*;` ĐÃ là vi phạm; cổng không đợi một
/// lời gọi `read(...)` trần xuất hiện ở một dòng sau đó.
///
/// Chỉ xét dòng bắt đầu bằng `use `/`pub use ` — tránh một dòng KHÔNG PHẢI `use` tình cờ
/// mang cụm `keychain::{`/`keychain::*` (ví dụ một macro hay một biểu thức generic) bị bắt
/// oan; `code` đã được `code_lines` `trim_start()` nên ký tự đầu dòng là ký tự thật.
fn line_use_pulls_in_read_via_brace_or_glob(code: &str) -> Option<&'static str> {
    if !(code.starts_with("use ") || code.starts_with("pub use ")) {
        return None;
    }

    if let Some(at) = code.find("keychain::{") {
        let after = &code[at + "keychain::{".len()..];
        let inside = after.split('}').next().unwrap_or(after);
        // 🔴 SỬA (spec 4.8, Phase 1, Task 3, đóng deferred-work.md:10441) — so TOKEN ĐẦU của
        // từng phần tử, không so BẰNG cả phần tử: `"read as get_secret"` tách bởi khoảng
        // trắng cho token đầu là `"read"`, nên đặt lại tên qua BRACE
        // (`keychain::{read as get_secret, configured}`) giờ vẫn bị bắt. Bản trước dùng
        // `item.trim() == "read"`, nên đúng hình dạng này lọt qua cổng — chỗ gọi tại nơi khác
        // trong tệp sau đó gọi `get_secret()`, một chuỗi cổng không hề tìm. Hình dạng BARE
        // (`use …keychain::read as get_secret;`, không ngoặc) đã bị bắt từ trước — nó chứa
        // nguyên vẹn chuỗi con `"keychain::read"` trước khoảng trắng của `as`, nên
        // `FORBIDDEN_RAW_VALUE_TOKENS`'s neo biên ĐUÔI đã khớp nó — lỗ THẬT chỉ ở dạng NGOẶC.
        if inside.split(',').any(|item| item.trim().split_whitespace().next() == Some("read")) {
            return Some("keychain::{read}");
        }
    }

    if code.contains("keychain::*") {
        return Some("keychain::*");
    }

    None
}

/// `rel` nằm TRONG `core/aiconfig/**` — khớp theo **biên thư mục**, không theo tiền tố
/// chuỗi. Cùng lỗ hổng mà `ai_boundary.rs::is_inside_ai_module` đã ghi cho `core/ai`:
/// `rel.starts_with(AICONFIG_DIR)` trần sẽ miễn trừ im lặng một thư mục anh em như
/// `core/aiconfig2/` hay `core/aiconfig_legacy/`.
fn is_inside_aiconfig_module(rel: &str) -> bool {
    boundary_scan::is_inside(rel, AICONFIG_DIR)
}

// ─────────────────────────────────────────────────────────────────────────────
// Spec 4.8, Phase 1, Task 3 — miễn trừ tệp DUY NHẤT cho chỗ gọi THẬT đầu tiên
// ─────────────────────────────────────────────────────────────────────────────

/// Tệp DUY NHẤT ngoài `core/aiconfig/**` được miễn trừ TRỌN VẸN khỏi cổng raw-value-accessor
/// — chỗ gọi THẬT đầu tiên của `keychain::read`/`expose_secret` (tầng lệnh dịch một segment,
/// spec 4.8: *"a pure function ... that resolves config, reads the key, ... calls 4.7's
/// producer, and streams through a `Channel<T>`"*).
///
/// 🔴 **MIỄN TRỪ TRỌN TỆP có chủ ý ở ĐÂY** — khác bài học V4 của `ai_boundary.rs` ("a
/// whole-file exemption was already tried and closed"), nhưng KHÔNG mâu thuẫn nó: rủi ro V4
/// bắt là một danh sách TÊN MỞ (bất kỳ định danh nào của `core::ai::rag`) lọt qua khi miễn trừ
/// trọn tệp — mở một đường thứ hai vào cả một MODULE. Ở đây [`FORBIDDEN_RAW_VALUE_TOKENS`] là
/// một tập ĐÓNG đúng HAI token cụ thể (`expose_secret`, `keychain::read`), không phải một
/// namespace — miễn trừ trọn tệp cho đúng hai token đó không mở thêm bề mặt nào khác, nó chỉ
/// nói đúng một câu đã ký sẵn ở đầu tệp này: *"Chỗ gọi hợp lệ DUY NHẤT hôm nay là Story 4.8
/// (`core/ai/`)"* — chỉ khác là spec 4.8's Code Map đặt chỗ gọi đó ở TẦNG LỆNH
/// (`commands/aitranslate.rs`), không ở `core/ai/client.rs` (xem doc-comment
/// `ports/translation_provider.rs` §`api_key` cho quyết định cùng nguồn: cổng chỉ nhận `&str`
/// đã lộ, để chỗ gọi — không phải cổng — chịu trách nhiệm expose).
///
/// ⚠️ Tệp này CHƯA TỒN TẠI ở Phase 1 — cùng lý lẽ `AI_PROMPT_SEAM_COMMAND_FILE`
/// (`ai_boundary.rs`, Story 4.7 Phase 1): `all_rust_sources()` chỉ liệt kê tệp CÓ TRÊN ĐĨA,
/// nên miễn trừ này KHÔNG BAO GIỜ khớp cho tới khi Phase 2 tạo tệp — không phải một lỗi. Nếu
/// Phase 2 kết luận `expose_secret`/`keychain::read` cần được gọi từ `core/ai/client.rs` THAY
/// VÌ (hoặc THÊM) `commands/aitranslate.rs`, đó là quyết định của Phase 2 khi có mã thật để
/// đo — nó phải thêm một hằng số miễn trừ THỨ HAI đúng khuôn này, đặt tên tệp mới, không mở
/// rộng hằng số này theo tiền tố.
const AI_TRANSLATE_KEYCHAIN_CALLER_FILE: &str = "commands/aitranslate.rs";

/// `rel` là đúng tệp seam Phase 1 Task 3 miễn trừ — xem [`AI_TRANSLATE_KEYCHAIN_CALLER_FILE`].
/// Khớp NGUYÊN VĂN, không theo tiền tố — cùng bài học `core/aim` mà
/// `ai_boundary.rs::is_inside_ai_module` đã phải sửa.
fn is_the_approved_ai_translate_keychain_caller_file(rel: &str) -> bool {
    rel == AI_TRANSLATE_KEYCHAIN_CALLER_FILE
}

// ═════════════════════════════════════════════════════════════════════════════════
// Sàn quần thể — chạy TRƯỚC mọi phép kiểm khác. Xem doc-comment đầu tệp.
// ═════════════════════════════════════════════════════════════════════════════════

#[test]
fn the_scanned_tree_and_the_aiconfig_module_are_both_large_enough_to_be_real() {
    let files = all_rust_sources();

    assert!(
        files.len() >= SRC_RS_FLOOR,
        "chỉ tìm thấy {} tệp `.rs` dưới `src-tauri/src/**` (sàn {SRC_RS_FLOOR}). Cây quá nhỏ \
         để là thật — một danh sách rỗng làm mọi phép kiểm dưới đây xanh mà không kiểm gì \
         cả. Nghi phạm: gốc quét sai, hoặc một thư mục bị bỏ.",
        files.len()
    );

    let aiconfig_files = files.iter().filter(|(rel, _)| is_inside_aiconfig_module(rel)).count();
    assert!(
        aiconfig_files >= AICONFIG_FLOOR,
        "chỉ tìm thấy {aiconfig_files} tệp `.rs` dưới `src/{AICONFIG_DIR}/**` (sàn \
         {AICONFIG_FLOOR}). Một đường dẫn gõ sai làm `walk` khớp 0 tệp, và khi đó cổng ranh \
         giới bên dưới xanh y hệt trên một thư mục RỖNG."
    );
}

// ═════════════════════════════════════════════════════════════════════════════════
// 🔴 Cổng THẬT — chỉ `core/aiconfig/**` được gọi bộ truy cập giá trị thật của khoá API
// ═════════════════════════════════════════════════════════════════════════════════

#[test]
fn no_file_outside_core_aiconfig_calls_the_raw_value_accessor_of_the_api_key() {
    let files = all_rust_sources();

    let mut violations: Vec<String> = Vec::new();
    let mut aiconfig_files = 0usize;

    for (rel, text) in &files {
        if is_inside_aiconfig_module(rel) {
            aiconfig_files += 1;
            continue;
        }
        // Spec 4.8, Phase 1, Task 3 -- chỗ gọi THẬT đầu tiên, miễn trừ TRỌN TỆP có chủ ý; xem
        // doc-comment [`AI_TRANSLATE_KEYCHAIN_CALLER_FILE`] cho lý lẽ vì sao trọn-tệp AN TOÀN
        // ở đúng gate này (khác bài học V4 của `ai_boundary.rs`).
        if is_the_approved_ai_translate_keychain_caller_file(rel) {
            continue;
        }
        for (line, code) in code_lines(text) {
            if let Some(needle) = line_names_a_forbidden_raw_value_access(&code) {
                violations.push(format!("{rel}:{line}  {needle}  |  {code}"));
            }
        }
    }

    // Miễn trừ phải khớp thứ gì đó — cùng đối chứng mà `ai_boundary.rs`/`scope_boundary.rs`
    // đòi: một `AICONFIG_DIR` gõ sai làm nhánh `continue` không bao giờ chạy.
    assert!(
        aiconfig_files > 0,
        "không tệp nào khớp `{AICONFIG_DIR}` — đường dẫn miễn trừ đã lệch khỏi cây nguồn"
    );

    assert!(
        violations.is_empty(),
        "{} chỗ NGOÀI `core/aiconfig/**` gọi bộ truy cập giá trị THẬT của khoá API:\n{}\n\n\
         §Always spec 4.3: 'The raw-value accessor is reachable only from inside \
         core/aiconfig/**'. Chỗ gọi hợp lệ DUY NHẤT ngoài đó là `{AI_TRANSLATE_KEYCHAIN_CALLER_FILE}` \
         (Story 4.8, Phase 1 Task 3) — tệp đó chưa tồn tại hôm nay, nên bất kỳ chỗ gọi nào \
         KHÁC là một vi phạm THẬT. DỪNG LẠI, đây là quyết định phạm vi của Ice, không phải một \
         lượt vá tiện tay.",
        violations.len(),
        violations.join("\n")
    );
}

// ═════════════════════════════════════════════════════════════════════════════════
// 🔴 Đối chứng dương — CA GIEO VI PHẠM TỔNG HỢP (bắt buộc, xem Design Notes đầu tệp)
// ═════════════════════════════════════════════════════════════════════════════════

/// 🔴 Chứng minh [`line_names_a_forbidden_raw_value_access`] NỔ ĐƯỢC trên một chuỗi vi
/// phạm và KHÔNG nổ trên một chuỗi sạch, độc lập với việc cây nguồn hôm nay có gì — đúng
/// khuôn `ai_boundary.rs::the_bare_dependency_check_would_actually_flag_a_seeded_violation_and_ignore_clean_code`,
/// cần thiết vì `core/aiconfig/keychain.rs` hôm nay có **0 chỗ gọi thật** từ bên ngoài để
/// tự làm chứng (Story 4.8 chưa tồn tại).
#[test]
fn the_raw_value_accessor_check_would_actually_flag_a_seeded_violation_and_ignore_clean_code() {
    // Ca DƯƠNG THẬT thứ nhất — gọi phương thức `.expose_secret()` từ một module khác.
    assert_eq!(
        line_names_a_forbidden_raw_value_access(
            "        let raw = secret.expose_secret().to_owned();"
        ),
        Some("expose_secret"),
        "ca DUONG THAT: mot loi goi .expose_secret() ngoai core/aiconfig/** phai bi bat"
    );

    // Ca DƯƠNG THẬT thứ hai — lời gọi đủ điều kiện `keychain::read(...)`.
    assert_eq!(
        line_names_a_forbidden_raw_value_access(
            "    let key = crate::core::aiconfig::keychain::read()?;"
        ),
        Some("keychain::read"),
        "ca DUONG THAT: mot loi goi keychain::read(...) du dieu kien phai bi bat"
    );

    // Ca DƯƠNG THẬT thứ ba — `use` đặt tên `read`, không cần dấu `(`.
    assert_eq!(
        line_names_a_forbidden_raw_value_access("use super::keychain::read;"),
        Some("keychain::read"),
        "ca DUONG THAT: mot cau `use` dat ten ham read cung phai bi bat, khong chi loi goi"
    );

    // Ca ÂM — mã sạch không nhắc token bị cấm, không được bị bắt oan.
    assert_eq!(
        line_names_a_forbidden_raw_value_access("    let configured = keychain::configured()?;"),
        None,
        "ca AM: keychain::configured (khong phai read) khong duoc bi bat oan"
    );
    assert_eq!(
        line_names_a_forbidden_raw_value_access("    keychain::set(&validated)?;"),
        None,
        "ca AM: keychain::set khong duoc bi bat oan"
    );

    // Đối chứng thêm: dòng CHÚ THÍCH bị `code_lines` loại trước khi vị từ nhìn thấy nó.
    let synthetic = "// secret.expose_secret() se duoc goi o Story 4.8\n\
                      fn stub() {}\n\
                      // crate::core::aiconfig::keychain::read() -- vi du trong comment\n";
    let code_only: Vec<String> = code_lines(synthetic).map(|(_, code)| code).collect();
    assert!(
        !code_only.iter().any(|code| line_names_a_forbidden_raw_value_access(code).is_some()),
        "hai dong CHU THICH nhac toi token bi cam khong duoc lot vao tap DONG MA"
    );
    assert!(
        code_only.iter().any(|code| code == "fn stub() {}"),
        "dong MA that su (khong phai comment) phai con lai sau code_lines"
    );
}

/// Đối chứng dương/âm riêng cho [`line_use_pulls_in_read_via_brace_or_glob`] — bổ sung
/// 2026-09-17 (rà soát): `use …keychain::{read, ...}` (BRACE) và `use …keychain::*;` (GLOB)
/// không chứa nguyên vẹn chuỗi con `"keychain::read"`, nên `FORBIDDEN_RAW_VALUE_TOKENS` bỏ
/// lọt cả hai — một cơ chế lách KHÁC với lỗ đặt-lại-tên-qua-`as` đã ghi ở đầu tệp, không
/// cùng một lỗ. Cùng khuôn ca ngay trên: dương thật + âm, gọi qua đúng vị từ mà cổng thật
/// dùng ([`line_names_a_forbidden_raw_value_access`]).
#[test]
fn the_brace_or_glob_use_check_would_actually_flag_a_seeded_violation_and_ignore_clean_code() {
    // Ca DƯƠNG THẬT thứ nhất — BRACE, `read` đứng ĐẦU danh sách.
    assert_eq!(
        line_names_a_forbidden_raw_value_access(
            "use crate::core::aiconfig::keychain::{read, configured};"
        ),
        Some("keychain::{read}"),
        "ca DUONG THAT: use dang BRACE dat ten read (dau danh sach) phai bi bat"
    );

    // Ca DƯƠNG THẬT thứ hai — BRACE, `read` đứng CUỐI danh sách.
    assert_eq!(
        line_names_a_forbidden_raw_value_access("use super::keychain::{configured, delete, read};"),
        Some("keychain::{read}"),
        "ca DUONG THAT: use dang BRACE dat ten read o CUOI danh sach cung phai bi bat"
    );

    // Ca DƯƠNG THẬT thứ ba — GLOB: bản thân câu `use` đã là vi phạm, không cần một lời gọi
    // `read(...)` trần xuất hiện ở dòng sau.
    assert_eq!(
        line_names_a_forbidden_raw_value_access("use crate::core::aiconfig::keychain::*;"),
        Some("keychain::*"),
        "ca DUONG THAT: use dang GLOB keo read vao pham vi VO DIEU KIEN, ban than cau use da la \
         vi pham"
    );

    // Ca ÂM thứ nhất — BRACE không đặt tên `read`, không được bắt oan.
    assert_eq!(
        line_names_a_forbidden_raw_value_access("use super::keychain::{configured, delete};"),
        None,
        "ca AM: BRACE khong dat ten read khong duoc bi bat oan"
    );

    // Ca ÂM thứ hai — một dòng KHÔNG PHẢI `use` tình cờ mang cụm ký tự giống nhau.
    assert_eq!(
        line_names_a_forbidden_raw_value_access(
            "    let keychain_placeholder = format!(\"keychain::{{read}}\");"
        ),
        None,
        "ca AM: mot dong khong bat dau bang `use `/`pub use ` khong duoc xet boi vi tu nay, du \
         no tinh co mang chuoi con giong het"
    );

    // Ca ÂM thứ ba — BRACE đặt tên một định danh DÀI HƠN mang `read` làm tiền tố, không được
    // bắt oan (so BẰNG từng phần tử đã tách dấu phẩy, không phải `contains`).
    assert_eq!(
        line_names_a_forbidden_raw_value_access("use super::keychain::{ready_flag, configured};"),
        None,
        "DO OAN: `ready_flag` la mot dinh danh KHAC voi `read`, khong duoc bi bat"
    );
}

// ═════════════════════════════════════════════════════════════════════════════════
// Spec 4.8, Phase 1, Task 3 — đóng lỗ đặt-lại-tên qua BRACE (`deferred-work.md:10441`)
// ═════════════════════════════════════════════════════════════════════════════════

/// 🔴 Đóng `deferred-work.md:10441` cho ĐÚNG hình dạng còn hở: `use
/// …keychain::{read as get_secret, ...};` — một `use` BRACE đặt lại tên `read` qua `as`. Hình
/// dạng BARE tương đương (`use …keychain::read as get_secret;`, không ngoặc) đã bị bắt từ
/// trước bản sửa này — nó chứa nguyên vẹn chuỗi con `"keychain::read"` ngay trước khoảng
/// trắng của `as`, và `FORBIDDEN_RAW_VALUE_TOKENS`'s neo biên ĐUÔI (không phải đầu) đã khớp
/// nó; đo được bằng ca đầu tiên dưới đây, giữ nguyên KHÔNG đổi qua bản sửa. Lỗ THẬT nằm ở
/// dạng NGOẶC: `item.trim() == "read"` (bản trước [`line_use_pulls_in_read_via_brace_or_glob`])
/// không khớp chuỗi `"read as get_secret"`, nên nếu chỗ gọi thật viết vậy rồi gọi
/// `get_secret()` ở một dòng khác, KHÔNG dòng nào trong cả tệp còn mang một trong hai
/// [`FORBIDDEN_RAW_VALUE_TOKENS`] — cổng xanh trong khi vi phạm tồn tại, đúng nguyên văn tóm
/// tắt của `deferred-work.md:10441`.
#[test]
fn the_renamed_import_hole_in_deferred_work_10441_is_closed_for_the_brace_shape() {
    // Ca DƯƠNG THẬT thứ nhất — hình dạng BARE (không ngoặc) ĐÃ bị bắt từ trước bản sửa này;
    // giữ lại như một đối chứng KHÔNG HỒI QUY, không phải một khẳng định mới.
    assert_eq!(
        line_names_a_forbidden_raw_value_access(
            "use crate::core::aiconfig::keychain::read as get_secret;"
        ),
        Some("keychain::read"),
        "hình dạng BARE (không ngoặc) của use...as đã bị bắt từ trước bản sửa 10441 — không          được hồi quy"
    );

    // Ca DƯƠNG THẬT thứ hai — hình dạng NGOẶC, `read as alias` đứng ĐẦU danh sách. Đây là ca
    // mà bản TRƯỚC bản sửa này bỏ lọt.
    assert_eq!(
        line_names_a_forbidden_raw_value_access(
            "use crate::core::aiconfig::keychain::{read as get_secret, configured};"
        ),
        Some("keychain::{read}"),
        "10441: use...as dang NGOAC (read as get_secret) phai bi bat -- day dung hinh \
         dang cong bo bi lot truoc ban sua nay"
    );

    // Ca DƯƠNG THẬT thứ ba — hình dạng NGOẶC, `read as alias` đứng CUỐI danh sách.
    assert_eq!(
        line_names_a_forbidden_raw_value_access(
            "use super::keychain::{configured, delete, read as fetch};"
        ),
        Some("keychain::{read}"),
        "10441: use...as dang NGOAC o CUOI danh sach cung phai bi bat"
    );

    // Ca ÂM — một định danh KHÁC mang `read` làm tiền tố, dù có `as`, không được bắt oan (so
    // TOKEN ĐẦU sau khi tách khoảng trắng, không phải `contains`/`starts_with`).
    assert_eq!(
        line_names_a_forbidden_raw_value_access(
            "use super::keychain::{ready_flag as rf, configured};"
        ),
        None,
        "ĐỎ OAN: `ready_flag` là một định danh KHÁC với `read`, dù đổi tên qua `as`, không          được bị bắt"
    );
}

// ═════════════════════════════════════════════════════════════════════════════════
// Spec 4.8, Phase 1, Task 3 — miễn trừ tệp mới khớp HẸP, không khớp hàng xóm
// ═════════════════════════════════════════════════════════════════════════════════

/// Đối chứng dương + âm cho [`is_the_approved_ai_translate_keychain_caller_file`] — cùng
/// khuôn `ai_boundary.rs::the_third_approved_seam_is_matched_narrowly_and_neighbours_are_not`:
/// khớp CHÍNH XÁC đường dẫn Task 3 đặt tên, và KHÔNG khớp một hàng xóm gần giống (cùng lớp
/// lỗi XANH GIẢ mà `core/aim` đã bị bắt một lần cho `ai_boundary.rs::is_inside_ai_module`).
#[test]
fn the_new_caller_exemption_is_matched_narrowly_and_neighbours_are_not() {
    assert!(
        is_the_approved_ai_translate_keychain_caller_file(AI_TRANSLATE_KEYCHAIN_CALLER_FILE),
        "ca dương thật: đúng đường dẫn Task 3 đặt tên phải khớp"
    );
    for hang_xom in [
        "commands/aitranslate2.rs",
        "commands/aitranslateset.rs",
        "commands/aitranslate/mod.rs",
        "commands/aitranslate_test.rs",
        "core/commands/aitranslate.rs",
        "commands/aitranslate.rs.bak",
        "commands/aiprompt.rs",
        "core/ai/client.rs",
    ] {
        assert!(
            !is_the_approved_ai_translate_keychain_caller_file(hang_xom),
            "XANH OAN: {hang_xom:?} là một tệp HÀNG XÓM, không phải đúng tệp Task 3 đặt tên --              một hằng số khớp theo tiền tố sẽ tha oan nó, đúng khuyết tật `core/aim`"
        );
    }
}

// ═════════════════════════════════════════════════════════════════════════════════
// Đối chứng dương — `core/aiconfig/keychain.rs` thật sự ĐỊNH NGHĨA cả hai hàm bị canh
// ═════════════════════════════════════════════════════════════════════════════════

/// Không có ca này thì cổng thật ở trên xanh y hệt trên một cây mà `keychain.rs` đã bị
/// xoá sạch cả hai hàm — "không ai vi phạm" và "không có gì để vi phạm" đọc giống nhau.
///
/// ⚠️ Đây là đối chứng trên **định nghĩa**, không phải trên **chỗ gọi** — khác
/// `glossary_boundary.rs::core_glossary_actually_defines_the_restricted_surface`, vì
/// không có chỗ gọi sản phẩm nào hôm nay (xem doc-comment đầu tệp).
#[test]
fn core_aiconfig_keychain_actually_defines_both_raw_value_accessors() {
    let files = all_rust_sources();
    let (_, text) = files
        .iter()
        .find(|(rel, _)| rel == "core/aiconfig/keychain.rs")
        .unwrap_or_else(|| panic!("khong tim thay core/aiconfig/keychain.rs -- cay da bi cat"));

    assert!(
        text.contains("fn expose_secret"),
        "`core/aiconfig/keychain.rs` khong con dinh nghia `expose_secret` -- cay da bi cat va \
         cong ranh gioi phia tren dang canh mot cho trong"
    );
    assert!(
        text.contains("fn read()"),
        "`core/aiconfig/keychain.rs` khong con dinh nghia `read` -- cay da bi cat va cong ranh \
         gioi phia tren dang canh mot cho trong"
    );
}

// ═════════════════════════════════════════════════════════════════════════════════
// 🔴 NFR14 — miễn trừ `core/aiconfig` phải khớp trên CẢ hình dạng đường dẫn Windows
// ═════════════════════════════════════════════════════════════════════════════════

/// Cùng lý lẽ `ai_boundary.rs::the_core_ai_exemption_still_matches_when_the_path_arrives_windows_shaped`.
#[test]
fn the_core_aiconfig_exemption_still_matches_when_the_path_arrives_windows_shaped() {
    let root = Path::new("/repo/src-tauri/src");

    let windows_shaped = PathBuf::from("/repo/src-tauri/src").join("core\\aiconfig\\keychain.rs");
    let rel = rel_posix(root, &windows_shaped);

    assert_eq!(rel, "core/aiconfig/keychain.rs", "rel_posix phải đổi `\\` thành `/`; nhận được {rel:?}");
    assert!(
        is_inside_aiconfig_module(&rel),
        "đường dẫn hình dạng Windows {rel:?} phải khớp miễn trừ `{AICONFIG_DIR}`"
    );

    let other = PathBuf::from("/repo/src-tauri/src").join("core\\ai\\mod.rs");
    assert!(
        !is_inside_aiconfig_module(&rel_posix(root, &other)),
        "miễn trừ `{AICONFIG_DIR}` khớp quá rộng — một module ngoài `aiconfig/` sẽ được tha oan"
    );
}

// ═════════════════════════════════════════════════════════════════════════════════
// 🔴 Neo biên — token KHÔNG khớp hàng xóm tiền tố; thư mục KHÔNG khớp hàng xóm tên riêng
// ═════════════════════════════════════════════════════════════════════════════════

/// Cùng bài học vòng rà 1 của Story 4.1 (`ai_boundary.rs`) — chép lại tại đây cho hai vị
/// từ riêng của tệp này thay vì tin rằng "đã học một lần thì áp dụng khắp nơi".
#[test]
fn the_two_predicates_anchor_on_boundaries_and_do_not_fire_on_prefix_neighbours() {
    // ── ① Vị từ token: không khớp một định danh dài hơn có cùng tiền tố ────────────
    for hang_xom in [
        "let x = secret.expose_secret_for_debug_only();",
        "keychain::read_only_flag()",
        "keychain::reader::new()",
    ] {
        assert_eq!(
            line_names_a_forbidden_raw_value_access(hang_xom),
            None,
            "ĐỎ OAN: {hang_xom:?} mang mot dinh danh DAI HON co cung tien to, khong phai chinh \
             ham bi cam. Mot phep khop khong neo bien se bat nham no."
        );
    }
    assert_eq!(
        line_names_a_forbidden_raw_value_access("secret.expose_secret()"),
        Some("expose_secret"),
        "ca duong phai giu nguyen sau luot them neo bien"
    );

    // ── ② Miễn trừ thư mục: không khớp một thư mục anh em cùng tiền tố ─────────────
    assert!(is_inside_aiconfig_module("core/aiconfig"), "chinh thu muc phai duoc mien tru");
    assert!(is_inside_aiconfig_module("core/aiconfig/keychain.rs"), "tep trong core/aiconfig/ phai duoc mien tru");
    for anh_em in ["core/aiconfig2/x.rs", "core/aiconfig_legacy/y.rs", "core/aiconfigx.rs"] {
        assert!(
            !is_inside_aiconfig_module(anh_em),
            "XANH GIA: {anh_em:?} la mot thu muc ANH EM, khong nam trong core/aiconfig/. Mien \
             tru theo tien to chuoi se bo qua no khoi cong ranh gioi."
        );
    }
}
