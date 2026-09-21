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
///
/// 🔵 **CẬP NHẬT 2026-09-16 (Story 4.2) — 44 chống lại một quần thể thật 82 đã ngừng là một
/// tripwire.** Đo lần đầu tại Story 4.2 (trước khi module `core::aiconfig`/`commands::aiconfig`
/// ra đời): **82** tệp `.rs` dưới `src-tauri/src/**`.
///
/// 🔵 **SỬA TẠI CHỖ, cùng ngày — 65 (80% của 82) tính trên quần thể TRƯỚC ba tệp của chính
/// story này, không phải quần thể cây sẽ MANG hằng số này.** Sàn phải đúng 80–85% của cây
/// SAU khi story đóng (nó sống cùng ba tệp mới `core/aiconfig/mod.rs`,
/// `core/aiconfig/store.rs`, `commands/aiconfig.rs`), không phải một ảnh chụp giữa chừng.
/// Đo lại SAU khi ba tệp đó tồn tại: **85** tệp `.rs`. Sàn đúng là **68** (85 × 80% = 68,0 —
/// tròn, không cần làm tròn lên/xuống), không 65 (65/85 = 76,5%, DƯỚI dải 80–85% mà chính
/// đoạn văn này trích dẫn). Đóng nửa đã đo được của món nợ được giao (`deferred-work.md`:
/// "21 hằng floor khác trên 18 tệp `tests/*.rs` đã trôi cùng kiểu — đếm lại 2026-09-16 THẮNG
/// ước lượng '22 trên 16' viết lúc lập kế hoạch — ghi nợ riêng, không sửa ở đây").
const SRC_RS_FLOOR: usize = 68;

/// Hai chuỗi BARE (không tiền tố `use `) mà chỉ `core/ai/**` được phép mang ở **vị trí mã**.
///
/// 🔴 Dạng TRẦN là bắt buộc — đúng khuyết tật `MATCHING_FORBIDDEN_USES`
/// (`matching_boundary.rs:75-87`) đã bị bắt ở một lượt review trước: một bản chỉ so
/// `"use crate::core::ai"` bỏ lọt một lời gọi đủ điều kiện viết THẲNG trong thân hàm
/// (`crate::core::ai::foo()`, không qua `use`). `"super::ai"` phủ một module `core/*` khác
/// gọi sang bằng đường tương đối (`super::ai::foo()`), thứ mà một token chỉ neo ở `crate::`
/// sẽ bỏ lọt.
const FORBIDDEN_BARE_TOKENS: [&str; 2] = ["crate::core::ai", "super::ai"];

/// Tệp DUY NHẤT được Decision 1 (spec 4.7) miễn trừ TRỌN VẸN khỏi cổng bare-token — lớp lệnh
/// bắc cầu qua AD-13 (`src-tauri/src/commands/aiprompt.rs`, Story 4.7 Phase 2). Khớp NGUYÊN
/// VĂN đường dẫn tương đối, không theo tiền tố — đúng bài học `core/aim` mà [`is_inside_ai_module`]
/// đã phải sửa: một hằng số khớp-tiền-tố sẽ tha oan `commands/aiprompt2.rs` hay
/// `commands/aiprompt/mod.rs`. Xem đối chứng ở
/// [`the_approved_ai_prompt_command_file_is_matched_narrowly_and_neighbours_are_not`].
///
/// ⚠️ Tệp này CHƯA TỒN TẠI ở Phase 1 (Story 4.7 chia bốn giai đoạn, tệp này là việc của Phase
/// 2) — cổng thật không đòi nó tồn tại: `all_rust_sources()` chỉ liệt kê tệp CÓ TRÊN ĐĨA, nên
/// một đường dẫn miễn trừ chưa ai tạo đơn giản là KHÔNG BAO GIỜ khớp, không phải một lỗi.
const AI_PROMPT_SEAM_COMMAND_FILE: &str = "commands/aiprompt.rs";

/// Tệp `lib.rs` — chuỗi neo cho bộ dò-và-xoá của ca biên dịch thật
/// ([`lib_rs_without_the_approved_ai_prompt_seam`]), KHÔNG còn là tên tệp của một miễn trừ
/// trên cổng bare-token nào ([`AI_PROMPT_SEAM_LIB_RS_MARKER`] kể dưới đây).
///
/// 🔵 **ĐÓNG 2026-09-21 (spec 4.8, Phase 2) — `deferred-work.md:10840`.** Hằng số này TỪNG
/// đứng cạnh một miễn trừ THEO DÒNG trên cổng bare-token
/// (`no_file_outside_core_ai_names_a_bare_dependency_on_the_ai_module`), dự phòng cho khả năng
/// một dòng `lib.rs` mang CẢ tiền tố `commands::aiprompt` LẪN một trong hai
/// [`FORBIDDEN_BARE_TOKENS`] trên cùng một dòng. Đo được (Phase 2): `grep -n
/// "crate::core::ai" src/lib.rs` cho **0** dòng, không đổi từ lượt đo 2026-09-18 —
/// `lib.rs` chưa từng, và theo Code Map hôm nay không có lý do, viết thẳng một trong hai token
/// đó. Nhánh gate đó đã bị XOÁ (không sửa tại chỗ — một nhánh không bao giờ nổ thì không có gì
/// để sửa, chỉ có chỗ để dọn). Hằng số này VẪN SỐNG vì nó phục vụ một việc HOÀN TOÀN khác, độc
/// lập với cổng đó: xác định TỆP mà bộ dò-và-xoá của ca biên dịch thật FR77 thao tác trên bản
/// chép cây nguồn.
const AI_PROMPT_SEAM_LIB_RS_FILE: &str = "lib.rs";

/// Chuỗi con tiền tố đường dẫn module mà mọi dòng `generate_handler!`/`app.manage` của
/// `commands::aiprompt` trong `lib.rs` mang — khuôn đã có tiền lệ ngay trong `lib.rs` hôm nay
/// (`crate::commands::promptset::wire::…`, `crate::commands::promptset::
/// PendingPromptImportState::new(None)`): mọi lời gọi từ `lib.rs` vào một module `commands::*`
/// đánh vần TRỌN đường dẫn `crate::commands::<module>::…`, không `use` rồi gọi trần.
///
/// 🔵 Xem 🔵 ở doc-comment [`AI_PROMPT_SEAM_LIB_RS_FILE`] — hằng số này (cùng chuỗi, không đổi
/// giá trị) nay chỉ còn MỘT vai: chuỗi neo cho
/// [`text_without_lines_matching_the_ai_prompt_seam_marker`] (bộ dò-và-xoá của ca biên dịch
/// thật), không còn là marker miễn trừ của cổng bare-token — hai vai đó từng dùng CHUNG một
/// hằng số, nhưng chỉ MỘT trong hai còn thật.
///
/// Dấu `::` ĐUÔI là neo biên bắt buộc — không có nó, chuỗi con này khớp cả một module hàng
/// xóm tên dài hơn bắt đầu giống nhau (`crate::commands::aipromptset::…`); có `::` đuôi thì
/// ký tự ngay sau `aiprompt` trong `aipromptset` là `s`, không phải `:`, nên không khớp.
const AI_PROMPT_SEAM_LIB_RS_MARKER: &str = "crate::commands::aiprompt::";

/// `rel` là đúng tệp lệnh được Decision 1 miễn trừ — xem [`AI_PROMPT_SEAM_COMMAND_FILE`]. Dùng
/// để NHẬN DIỆN đúng tệp (đối chứng khớp-tên-tệp, và điểm neo của ca biên dịch), KHÔNG còn
/// nghĩa "miễn trừ TRỌN VẸN khỏi cổng bare-token" nữa kể từ finding V4 (loop 1, Story 4.7) —
/// xem [`line_is_the_approved_ai_prompt_import_in_command_file`] cho miễn trừ THEO DÒNG thật
/// sự dùng bởi cổng đó.
fn is_the_approved_ai_prompt_command_file(rel: &str) -> bool {
    rel == AI_PROMPT_SEAM_COMMAND_FILE
}

/// Chuỗi con Decision 1 miễn trừ THEO DÒNG bên trong seam ① (`commands/aiprompt.rs`) — đúng
/// tiền tố `use` DUY NHẤT tệp này viết vào `core::ai` (`core/ai/mod.rs` chỉ khai đúng một con,
/// `pub mod rag;` — không một module con nào khác của `ai/` để mà nhập). Story 4.7, finding V4
/// (loop 1): trước bản sửa này, TOÀN BỘ tệp bị miễn trừ khỏi cổng bare-token
/// (xem [`is_the_approved_ai_prompt_command_file`]'s doc-comment cũ) — một miễn trừ rộng hơn
/// hẳn thứ Decision 1 thật sự cho phép. Miễn trừ giờ chỉ khớp ĐÚNG dòng mang tiền tố này; mọi
/// dòng KHÁC của tệp (kể cả một `crate::core::ai::` thứ hai không đi qua `rag`, hay một
/// `super::ai` lạc vào) vẫn bị cổng thật quét như mọi tệp khác trong cây.
const AI_PROMPT_SEAM_COMMAND_FILE_MARKER: &str = "crate::core::ai::rag::";

/// `code` là một dòng TRONG seam ① (`rel` phải khớp [`is_the_approved_ai_prompt_command_file`])
/// mang đúng tiền tố nhập hợp lệ [`AI_PROMPT_SEAM_COMMAND_FILE_MARKER`] — cùng khuôn
/// [`line_is_the_approved_ai_translate_import_in_command_file`], chỉ khác tệp và chuỗi neo.
fn line_is_the_approved_ai_prompt_import_in_command_file(rel: &str, code: &str) -> bool {
    is_the_approved_ai_prompt_command_file(rel) && code.contains(AI_PROMPT_SEAM_COMMAND_FILE_MARKER)
}

// ─────────────────────────────────────────────────────────────────────────────
// Spec 4.8, Phase 1, Task 2 — seam THỨ BA: `commands/aitranslate.rs` gọi `core::ai::client`
// ─────────────────────────────────────────────────────────────────────────────
// Rationale (spec 4.8, Phase 1): "today's marker admits `rag` only, and a whole-file
// exemption was already tried and closed (V4)" -- cùng khuôn seam ① ở trên, KHÔNG một
// exemption trọn tệp, một marker THEO DÒNG cho đúng tiền tố module Phase 2 sẽ viết.

/// Tệp DUY NHẤT seam THỨ BA (spec 4.8) sẽ được miễn trừ -- tầng lệnh dịch một segment, chỗ
/// gọi thật ĐẦU TIÊN vào `core::ai::client` (cài đặt `TranslationProvider`, Phase 2 CỦA CHÍNH
/// STORY NÀY). Khớp NGUYÊN VĂN đường dẫn tương đối, không theo tiền tố -- cùng bài học
/// `core/aim` mà [`is_inside_ai_module`] đã phải sửa (xem doc-comment
/// [`AI_PROMPT_SEAM_COMMAND_FILE`] cho lý lẽ đầy đủ, chỉ khác seam).
///
/// ⚠️ Tệp này CHƯA TỒN TẠI ở Phase 1 (spec 4.8 chia bốn phase, tệp này là việc của Phase 2) --
/// cổng thật không đòi nó tồn tại: `all_rust_sources()` chỉ liệt kê tệp CÓ TRÊN ĐĨA, nên đường
/// dẫn miễn trừ này đơn giản KHÔNG BAO GIỜ khớp cho tới khi Phase 2 tạo tệp, không phải một
/// lỗi -- đúng nguyên văn lý lẽ [`AI_PROMPT_SEAM_COMMAND_FILE`] đã ghi cho chính nó ở Story
/// 4.7 Phase 1. Viết `core/ai/client.rs` trước khi cổng `TranslationProvider` tồn tại sẽ đảo
/// ngược đúng thứ tự phụ thuộc spec 4.8's Task 1 rationale nói tới -- đây là lý do Task 2
/// (cổng này) đứng CÙNG Phase với Task 1 (`ports::TranslationProvider`), trước mọi mã của
/// Phase 2.
const AI_TRANSLATE_SEAM_COMMAND_FILE: &str = "commands/aitranslate.rs";

/// Chuỗi con seam THỨ BA miễn trừ THEO DÒNG bên trong [`AI_TRANSLATE_SEAM_COMMAND_FILE`] --
/// tiền tố module Phase 2 sẽ viết để gọi cài đặt `TranslationProvider`. Cùng khuôn
/// [`AI_PROMPT_SEAM_COMMAND_FILE_MARKER`]: miễn trừ khớp NGUYÊN chuỗi con này, không trọn
/// dòng và không trọn tệp -- một `crate::core::ai::` KHÁC không đi qua `client` (vd. một
/// `super::ai` lạc vào, hay một `crate::core::ai::rag::` thứ hai viết nhầm vào tệp này) vẫn bị
/// cổng thật quét như mọi tệp khác trong cây.
///
/// ⚠️ **GIỚI HẠN THẬT, ghi ra thay vì để Phase 2 tự phát hiện:** khác seam ① (aiprompt), seam
/// này KHÔNG có một "control ②" liệt kê tên -- không có
/// `ALLOWED_AI_CLIENT_NAMES_IN_COMMAND_SEAM` đi kèm, đúng khuôn
/// `ALLOWED_AI_RAG_NAMES_IN_COMMAND_SEAM` +
/// `commands_aiprompt_rs_names_nothing_beyond_the_allowed_ai_rag_surface`. Lý do: control ② đó
/// đòi những TÊN CỤ THỂ `core::ai::client` xuất -- kiểu cài đặt `TranslationProvider`, các
/// kiểu mirror nếu có -- và những tên đó CHƯA TỒN TẠI ở Phase 1 (không có tệp nào để mà đặt
/// tên). Đây là món nợ có chủ, không phải một khoảng trống bị giấu: Phase 2 (viết
/// `core/ai/client.rs` VÀ `commands/aitranslate.rs`) phải thêm cặp hằng số + cổng thật đối
/// xứng, đúng khuôn seam ①, MỘT khi có tên thật để mà đóng băng.
const AI_TRANSLATE_SEAM_COMMAND_FILE_MARKER: &str = "crate::core::ai::client::";

/// `rel` là đúng tệp lệnh seam THỨ BA -- xem [`AI_TRANSLATE_SEAM_COMMAND_FILE`]. Cùng khuôn
/// [`is_the_approved_ai_prompt_command_file`].
fn is_the_approved_ai_translate_command_file(rel: &str) -> bool {
    rel == AI_TRANSLATE_SEAM_COMMAND_FILE
}

/// `code` là một dòng TRONG seam THỨ BA (`rel` phải khớp
/// [`is_the_approved_ai_translate_command_file`]) mang đúng tiền tố nhập hợp lệ
/// [`AI_TRANSLATE_SEAM_COMMAND_FILE_MARKER`] -- cùng khuôn
/// [`line_is_the_approved_ai_prompt_import_in_command_file`], chỉ khác tệp và chuỗi neo.
fn line_is_the_approved_ai_translate_import_in_command_file(rel: &str, code: &str) -> bool {
    is_the_approved_ai_translate_command_file(rel)
        && code.contains(AI_TRANSLATE_SEAM_COMMAND_FILE_MARKER)
}

/// Xoá MỌI lần xuất hiện của `marker` khỏi `code` — dùng để quét PHẦN CÒN LẠI của một dòng đã
/// mang chuỗi con được Decision 1 duyệt, thay vì `continue` bỏ qua TRỌN dòng (finding P8, loop
/// 2). Xoá thay vì bỏ qua là bắt buộc, không phải một lựa chọn chặt tay: `marker` (dù là
/// [`AI_PROMPT_SEAM_LIB_RS_MARKER`] hay [`AI_PROMPT_SEAM_COMMAND_FILE_MARKER`]) chính nó có thể
/// CHỨA một trong hai [`FORBIDDEN_BARE_TOKENS`] làm tiền tố con
/// (`AI_PROMPT_SEAM_COMMAND_FILE_MARKER` bắt đầu đúng bằng `"crate::core::ai"`) — quét nguyên
/// dòng mà không xoá phần đã duyệt trước sẽ bắt oan chính chuỗi Decision 1 vừa cho phép.
fn line_with_marker_occurrences_removed(code: &str, marker: &str) -> String {
    code.replace(marker, "")
}

/// Đúng CHÍN tên được phép xuất hiện sau `ai::rag::` bên trong seam ① — Decision 1's control
/// ②: hai hàm AD-14 đã đóng băng ở `core/ai/rag.rs` (`assemble_prompt`, `gather_glossary_context`)
/// CỘNG bảy kiểu mirror-type mà tệp này phải ĐẶT TÊN để viết `impl From<…> for …Wire` (không
/// gọi được các kiểu đó thì không viết được các mirror type Story 4.7 đòi) — không một tên nào
/// khác của `core::ai::rag` (không `expand_prompt_body`, không `render_glossary_pairs`, không
/// bất kỳ hàm/hằng nội bộ nào khác). Bất kỳ tên nào khác lọt qua là một lối vào `core::ai` THỨ
/// HAI mà Decision 1 không hề mở.
const ALLOWED_AI_RAG_NAMES_IN_COMMAND_SEAM: [&str; 9] = [
    "assemble_prompt",
    "gather_glossary_context",
    "GlossaryInjectionStatus",
    "TmInjectionStatus",
    "InjectionLedger",
    "InjectedGlossaryTerm",
    "SuppressedGlossaryTerm",
    "PromptPiece",
    "PromptPieceKind",
];

/// Trích mọi định danh xuất hiện ngay sau [`AI_PROMPT_SEAM_COMMAND_FILE_MARKER`] trong
/// `joined` (đã nối toàn văn qua [`joined_code`]) — cùng khuôn [`glossary_names_named`], chỉ
/// khác chuỗi neo. Xem doc-comment của hàm đó cho các giới hạn thật đã ghi (không theo dõi
/// `use … as alias`, comment đuôi dòng vẫn còn trong `joined`, nhóm LỒNG dừng ở `}` đầu tiên).
fn ai_rag_names_named(joined: &str) -> Vec<String> {
    let anchor = AI_PROMPT_SEAM_COMMAND_FILE_MARKER;
    let mut out = Vec::new();
    let mut search_from = 0usize;
    let bytes = joined.as_bytes();

    while let Some(rel) = joined[search_from..].find(anchor) {
        let after_anchor = search_from + rel + anchor.len();
        let mut i = after_anchor;
        while i < bytes.len() && (bytes[i] as char).is_whitespace() {
            i += 1;
        }
        if i >= bytes.len() {
            break;
        }

        if bytes[i] == b'*' {
            out.push("*".to_owned());
            search_from = i + 1;
        } else if bytes[i] == b'{' {
            let Some(close_rel) = joined[i + 1..].find('}') else {
                break;
            };
            let inner = &joined[i + 1..i + 1 + close_rel];
            for item in inner.split(',') {
                let Some(raw_name) = item.split_whitespace().next() else { continue };
                let name: String = raw_name.chars().filter(|c| is_ident_char(*c)).collect();
                if !name.is_empty() && name != "self" {
                    out.push(name);
                }
            }
            search_from = i + 1 + close_rel + 1;
        } else {
            let start = i;
            let mut j = i;
            while j < bytes.len() && is_ident_char(bytes[j] as char) {
                j += 1;
            }
            let name = &joined[start..j];
            if !name.is_empty() && name != "self" {
                out.push(name.to_owned());
            }
            search_from = if j > after_anchor { j } else { after_anchor + 1 };
        }
    }

    out
}

/// 🔴 Cổng THẬT của Decision 1's control ② (Story 4.7, findings V4/B8) — seam ① không được
/// gọi bất kỳ tên nào khác của `core::ai::rag` ngoài [`ALLOWED_AI_RAG_NAMES_IN_COMMAND_SEAM`].
/// Cộng với [`line_is_the_approved_ai_prompt_import_in_command_file`] (miễn trừ THEO DÒNG của
/// cổng bare-token chính), đây là hai nửa khiến control ② thật sự được canh thay vì chỉ đứng
/// trong doc-comment: một tệp nào đó lọt qua cổng bare-token (vì dòng của nó khớp tiền tố hợp
/// lệ) vẫn phải trả lời được câu hỏi "tên gì đứng sau tiền tố đó" — đúng câu hỏi cổng này hỏi.
#[test]
fn commands_aiprompt_rs_names_nothing_beyond_the_allowed_ai_rag_surface() {
    let files = all_rust_sources();
    let Some((_, text)) = files.iter().find(|(rel, _)| is_the_approved_ai_prompt_command_file(rel))
    else {
        // Đúng khuôn Phase 1's honesty note: tệp có thể chưa tồn tại (Phase sớm hơn) -- không
        // gì để kiểm thì không phải một vi phạm, nhưng CŨNG không phải một PASS im lặng: seam
        // ① hôm nay (Story 4.7 đã đóng Phase 2+) LUÔN tồn tại, nên nhánh này không nên chạy
        // trên cây thật -- ghi rõ để một `walk` lỗi không đọc thành "sạch".
        panic!(
            "`{AI_PROMPT_SEAM_COMMAND_FILE}` khong ton tai trong cay quet -- ca nay khong con \
             gi de kiem, va tren cay that hom nay day la mot dau hieu goc quet sai, khong phai \
             mot trang thai hop le"
        );
    };
    let joined = joined_code(text);
    let names = ai_rag_names_named(&joined);

    // Đối chứng dương: phép quét phải THẬT SỰ thu được ít nhất một tên — cùng đối chứng
    // [`no_core_ai_file_names_a_core_glossary_identifier_outside_the_allowed_four`] đòi.
    assert!(
        !names.is_empty(),
        "phep quet KHONG thu duoc mot ten `core::ai::rag` nao trong `{AI_PROMPT_SEAM_COMMAND_FILE}` \
         -- neu tep that su goi `assemble_prompt`/`gather_glossary_context`, day la vi tu dang mu, \
         khong phai mot cay sach"
    );

    let violations: Vec<&String> =
        names.iter().filter(|name| !ALLOWED_AI_RAG_NAMES_IN_COMMAND_SEAM.contains(&name.as_str())).collect();
    assert!(
        violations.is_empty(),
        "{} ten NGOAI {ALLOWED_AI_RAG_NAMES_IN_COMMAND_SEAM:?} duoc goi tu `core::ai::rag` trong \
         `{AI_PROMPT_SEAM_COMMAND_FILE}`:\n{:?}\n\nDecision 1's control ②: seam nay chi duoc \
         GOI XUONG hai ham AD-14 da dong bang cong bay kieu mirror-type can dat ten -- KHONG \
         lap rap them, KHONG thay the bien, KHONG quet marker lan hai.",
        violations.len(),
        violations
    );
}

/// Đối chứng dương + âm cho [`ai_rag_names_named`] trên văn bản DỰNG TAY — độc lập với
/// `commands/aiprompt.rs` thật, cùng khuôn
/// [`the_bare_dependency_check_would_actually_flag_a_seeded_violation_and_ignore_clean_code`].
#[test]
fn ai_rag_names_named_collects_a_multiline_use_group_and_a_seeded_forbidden_name() {
    let text = "use crate::core::ai::rag::{\n    assemble_prompt, gather_glossary_context,\n    InjectionLedger,\n};\n";
    assert_eq!(
        ai_rag_names_named(&joined_code(text)),
        vec!["assemble_prompt", "gather_glossary_context", "InjectionLedger"],
        "phai thu duoc CA BA ten, du nhom `use` trai qua nhieu dong"
    );

    // Ca DƯƠNG THẬT — một tên KHÔNG nằm trong danh sách cho phép phải bị gieo được, độc lập
    // với cây nguồn hôm nay có gì.
    let seeded = "use crate::core::ai::rag::{assemble_prompt, expand_prompt_body};\n";
    let names = ai_rag_names_named(&joined_code(seeded));
    assert!(names.contains(&"expand_prompt_body".to_owned()), "phai gieo bat duoc mot ten LA");
    assert!(
        !ALLOWED_AI_RAG_NAMES_IN_COMMAND_SEAM.contains(&"expand_prompt_body"),
        "tien de: 'expand_prompt_body' khong duoc nam trong danh sach cho phep"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// Spec 4.8, Phase 2 (carried Phase 1 task) — control ② của seam THỨ BA: đóng băng TÊN được
// phép gọi từ `core::ai::client` bên trong `commands/aitranslate.rs` — cùng khuôn control ②
// của seam ① ngay trên (`ALLOWED_AI_RAG_NAMES_IN_COMMAND_SEAM` +
// `commands_aiprompt_rs_names_nothing_beyond_the_allowed_ai_rag_surface`), giờ mới dựng được
// vì `core/ai/client.rs` không tồn tại ở Phase 1 (xem doc-comment
// `AI_TRANSLATE_SEAM_COMMAND_FILE_MARKER` §GIỚI HẠN THẬT ngay phía trên).
// ─────────────────────────────────────────────────────────────────────────────

/// Đúng HAI tên được phép xuất hiện sau `ai::client::` bên trong seam THỨ BA — đo trên mã
/// thật của `commands/aitranslate.rs`: `use crate::core::ai::client::{OpenAiChatClient,
/// OpenAiClientError};`, không một tên nào khác. Bất kỳ tên nào khác lọt qua là một lối vào
/// `core::ai::client` THỨ HAI mà tầng lệnh không cần: nó chỉ gọi cài đặt DUY NHẤT của cổng
/// (`OpenAiChatClient`) và đóng gói lỗi CỦA NÓ (`OpenAiClientError`) thành `IpcError` — không
/// lắp lại request, không thay biến, không một kiểu nội bộ khác của module đó (kể cả
/// `build_request_body`/`ChatCompletionsRequestBody`/`split_sse_frames`, những tên chỉ
/// `tests/ai_translate_contract.rs` — một crate KHÁC — mới cần gọi).
const ALLOWED_AI_CLIENT_NAMES_IN_COMMAND_SEAM: [&str; 2] = ["OpenAiChatClient", "OpenAiClientError"];

/// Trích mọi định danh xuất hiện ngay sau [`AI_TRANSLATE_SEAM_COMMAND_FILE_MARKER`] trong
/// `joined` (đã nối toàn văn qua [`joined_code`]) — cùng khuôn [`ai_rag_names_named`], chỉ
/// khác chuỗi neo. Xem doc-comment của hàm đó cho các giới hạn thật đã ghi (không theo dõi
/// `use … as alias`, comment đuôi dòng vẫn còn trong `joined`, nhóm LỒNG dừng ở `}` đầu tiên).
fn ai_client_names_named(joined: &str) -> Vec<String> {
    let anchor = AI_TRANSLATE_SEAM_COMMAND_FILE_MARKER;
    let mut out = Vec::new();
    let mut search_from = 0usize;
    let bytes = joined.as_bytes();

    while let Some(rel) = joined[search_from..].find(anchor) {
        let after_anchor = search_from + rel + anchor.len();
        let mut i = after_anchor;
        while i < bytes.len() && (bytes[i] as char).is_whitespace() {
            i += 1;
        }
        if i >= bytes.len() {
            break;
        }

        if bytes[i] == b'*' {
            out.push("*".to_owned());
            search_from = i + 1;
        } else if bytes[i] == b'{' {
            let Some(close_rel) = joined[i + 1..].find('}') else {
                break;
            };
            let inner = &joined[i + 1..i + 1 + close_rel];
            for item in inner.split(',') {
                let Some(raw_name) = item.split_whitespace().next() else { continue };
                let name: String = raw_name.chars().filter(|c| is_ident_char(*c)).collect();
                if !name.is_empty() && name != "self" {
                    out.push(name);
                }
            }
            search_from = i + 1 + close_rel + 1;
        } else {
            let start = i;
            let mut j = i;
            while j < bytes.len() && is_ident_char(bytes[j] as char) {
                j += 1;
            }
            let name = &joined[start..j];
            if !name.is_empty() && name != "self" {
                out.push(name.to_owned());
            }
            search_from = if j > after_anchor { j } else { after_anchor + 1 };
        }
    }

    out
}

/// 🔴 Cổng THẬT của control ② của seam THỨ BA — `commands/aitranslate.rs` không được gọi bất
/// kỳ tên nào khác của `core::ai::client` ngoài [`ALLOWED_AI_CLIENT_NAMES_IN_COMMAND_SEAM`].
/// Cộng với [`line_is_the_approved_ai_translate_import_in_command_file`] (miễn trừ THEO DÒNG
/// của cổng bare-token chính), đây là hai nửa khiến control ② thật sự được canh — cùng vai trò
/// [`commands_aiprompt_rs_names_nothing_beyond_the_allowed_ai_rag_surface`], trên seam THỨ BA.
#[test]
fn commands_aitranslate_rs_names_nothing_beyond_the_allowed_ai_client_surface() {
    let files = all_rust_sources();
    let Some((_, text)) =
        files.iter().find(|(rel, _)| is_the_approved_ai_translate_command_file(rel))
    else {
        panic!(
            "`{AI_TRANSLATE_SEAM_COMMAND_FILE}` khong ton tai trong cay quet -- ca nay khong con \
             gi de kiem, va tren cay that hom nay day la mot dau hieu goc quet sai, khong phai \
             mot trang thai hop le"
        );
    };
    let joined = joined_code(text);
    let names = ai_client_names_named(&joined);

    assert!(
        !names.is_empty(),
        "phep quet KHONG thu duoc mot ten `core::ai::client` nao trong \
         `{AI_TRANSLATE_SEAM_COMMAND_FILE}` -- neu tep that su goi `OpenAiChatClient`/\
         `OpenAiClientError`, day la vi tu dang mu, khong phai mot cay sach"
    );

    let violations: Vec<&String> = names
        .iter()
        .filter(|name| !ALLOWED_AI_CLIENT_NAMES_IN_COMMAND_SEAM.contains(&name.as_str()))
        .collect();
    assert!(
        violations.is_empty(),
        "{} ten NGOAI {ALLOWED_AI_CLIENT_NAMES_IN_COMMAND_SEAM:?} duoc goi tu `core::ai::client` \
         trong `{AI_TRANSLATE_SEAM_COMMAND_FILE}`:\n{:?}\n\nControl ② cua seam THU BA: tang lenh \
         chi duoc goi CAI DAT DUY NHAT cua cong (`OpenAiChatClient`) va kieu loi CUA NO \
         (`OpenAiClientError`) -- KHONG lap rap them, KHONG thay the bien, KHONG quet marker lan \
         hai.",
        violations.len(),
        violations
    );
}

/// Đối chứng dương + âm cho [`ai_client_names_named`] trên văn bản DỰNG TAY — độc lập với
/// `commands/aitranslate.rs` thật, cùng khuôn
/// [`ai_rag_names_named_collects_a_multiline_use_group_and_a_seeded_forbidden_name`].
#[test]
fn ai_client_names_named_collects_a_multiline_use_group_and_a_seeded_forbidden_name() {
    let text = "use crate::core::ai::client::{\n    OpenAiChatClient, OpenAiClientError,\n};\n";
    assert_eq!(
        ai_client_names_named(&joined_code(text)),
        vec!["OpenAiChatClient", "OpenAiClientError"],
        "phai thu duoc CA HAI ten, du nhom `use` trai qua nhieu dong"
    );

    // Ca DƯƠNG THẬT — một tên KHÔNG nằm trong danh sách cho phép phải bị gieo được, độc lập
    // với cây nguồn hôm nay có gì.
    let seeded = "use crate::core::ai::client::{OpenAiChatClient, split_sse_frames};\n";
    let names = ai_client_names_named(&joined_code(seeded));
    assert!(names.contains(&"split_sse_frames".to_owned()), "phai gieo bat duoc mot ten LA");
    assert!(
        !ALLOWED_AI_CLIENT_NAMES_IN_COMMAND_SEAM.contains(&"split_sse_frames"),
        "tien de: 'split_sse_frames' khong duoc nam trong danh sach cho phep"
    );
}

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
            // 🔵 **ĐÓNG 2026-09-21 (spec 4.8, Phase 2) — `deferred-work.md:10840`, đo được: chỗ
            // miễn trừ THEO DÒNG cho `lib.rs` (nửa "seam ②" của Decision 1, spec 4.7) từng đứng
            // ở đây đã bị XOÁ, không sửa tại chỗ, vì `grep -n "crate::core::ai" src/lib.rs` cho
            // **0** dòng — `lib.rs` chỉ bao giờ đánh vần `crate::commands::aiprompt::…`/
            // `crate::commands::aitranslate::…`, không bao giờ viết thẳng một trong hai
            // `FORBIDDEN_BARE_TOKENS`. Nhánh cũ (gọi `line_is_the_approved_ai_prompt_seam_in_lib_rs`)
            // vì thế không đổi kết quả của cổng thật trên cây hôm nay hay bất kỳ cây nào `lib.rs`
            // còn giữ kỷ luật đó — xoá nó không mở một lỗ hổng, nó xoá một nhánh chưa từng có cơ
            // hội tự bào chữa. `AI_PROMPT_SEAM_LIB_RS_FILE`/`AI_PROMPT_SEAM_LIB_RS_MARKER` VẪN
            // còn (xem doc-comment của chúng) — chúng phục vụ MỘT việc khác, độc lập: bộ dò-và-
            // xoá của ca biên dịch thật bên dưới
            // ([`lib_rs_without_the_approved_ai_prompt_seam`]), thứ không liên quan gì tới cổng
            // bare-token này.
            //
            // 🔴 Story 4.7, finding V4 (loop 1) -- SỬA: trước bản này, seam ① miễn trừ TRỌN
            // VẸN cả tệp `commands/aiprompt.rs` khỏi phép quét (dòng đã xoá phía trên) --
            // Decision 1's control ② ("KHÔNG gọi bất kỳ thứ gì khác của `core::ai` ngoài đúng
            // hai hàm AD-14 đã đóng băng") khi đó chỉ là LỜI HỨA trong doc-comment: MỌI dòng
            // khác của tệp (kể cả một `crate::core::ai::` thứ hai, một `super::ai` lạc vào)
            // không bị cổng này nhìn thấy nữa. Miễn trừ giờ THEO DÒNG, đúng khuôn dòng ngay
            // trên: chỉ dòng THẬT SỰ mang tiền tố nhập hợp lệ
            // (`AI_PROMPT_SEAM_COMMAND_FILE_MARKER`) được bỏ qua; cổng riêng
            // [`commands_aiprompt_rs_names_nothing_beyond_the_allowed_ai_rag_surface`] ngay
            // dưới kiểm PHẦN TÊN của đúng dòng này khớp
            // [`ALLOWED_AI_RAG_NAMES_IN_COMMAND_SEAM`] -- hai cổng cộng lại mới thật sự đóng
            // control ②, không phải một trong hai riêng lẻ.
            //
            // 🔴 finding P8 (loop 2), cùng lý luận với nhánh `lib.rs` ở trên -- xoá CHUỖI CON đã
            // duyệt rồi quét PHẦN CÒN LẠI, không `continue` bỏ qua trọn dòng.
            if line_is_the_approved_ai_prompt_import_in_command_file(rel, code) {
                let remainder =
                    line_with_marker_occurrences_removed(code, AI_PROMPT_SEAM_COMMAND_FILE_MARKER);
                if let Some(needle) = line_names_a_forbidden_ai_dependency(&remainder) {
                    violations.push(format!("{rel}:{line}  {needle}  |  {code}"));
                }
                continue;
            }
            // Spec 4.8, Phase 1, Task 2 -- seam THỨ BA (`commands/aitranslate.rs` ->
            // `core::ai::client`). Cùng lý luận finding P8 ở seam ① ngay trên: XOÁ chuỗi con
            // đã duyệt rồi quét PHẦN CÒN LẠI, không `continue` bỏ qua TRỌN dòng -- một token
            // cấm THỨ HAI đứng cùng dòng với tiền tố đã duyệt vẫn phải bị bắt.
            if line_is_the_approved_ai_translate_import_in_command_file(rel, code) {
                let remainder = line_with_marker_occurrences_removed(
                    code,
                    AI_TRANSLATE_SEAM_COMMAND_FILE_MARKER,
                );
                if let Some(needle) = line_names_a_forbidden_ai_dependency(&remainder) {
                    violations.push(format!("{rel}:{line}  {needle}  |  {code}"));
                }
                continue;
            }
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

// ═════════════════════════════════════════════════════════════════════════════════
// Story 4.6 — bề mặt `core::glossary` ĐƯỢC PHÉP dưới `core/ai/**`: đúng BỐN tên
// ═════════════════════════════════════════════════════════════════════════════════
//
// ─────────────────────────────────────────────────────────────────────────────
// 🔴 VÌ SAO GATE NÀY THAY THẾ MỘT VỊ TỪ CŨ ĐỌC TỪNG DÒNG
// ─────────────────────────────────────────────────────────────────────────────
// Pass 1 (2026-09-17, rà soát loop 0 của spec 4.6) đo được ba khuyết tật trên một vị từ đầu
// tiên (`names_after_glossary_double_colon`, đã xoá): nó quét TỪNG DÒNG, nên
// `use crate::core::glossary::{` với dấu `}` đóng KHÔNG nằm trên cùng dòng làm nhánh
// `else { continue }` bỏ qua toàn bộ nhóm — **0 tên được thu** trên chính hình dạng mà
// `rustfmt` xuống dòng cho một nhóm `use` từ ba tên trở lên (đo được: seed một tên cấm bên
// TRONG nhóm nhiều dòng của `rag.rs` để `cargo test --test ai_boundary --test glossary_boundary`
// vẫn 100% xanh). Gate dưới đây nối MỌI dòng MÃ (bỏ comment) của một tệp thành MỘT khối
// (`joined_code`, cùng khuôn `glossary_boundary.rs::commands_glossary_calls_the_new_quick_add_surface_not_the_forbidden_one`)
// RỒI MỚI quét — một `\n` bên trong `{...}` không còn là vấn đề vì nó chỉ là MỘT ký tự nữa
// trong chuỗi đã nối.

/// Năm tên `core::glossary` được PHÉP xuất hiện dưới `core/ai/**` — Decision 5 của spec 4.6:
/// cửa DUY NHẤT (`confirmed_terms_for_injection`), kiểu nó trả về
/// (`GlossaryInjectionOutcome`), kiểu lỗi của nó (`GlossaryError`), hàm suy `MatchLang`
/// (`match_lang_for_source_lang` — không nằm trên `GLOSSARY_ONLY_SURFACE` của
/// `glossary_boundary.rs`, cùng khuôn `commands::glossary` đã dùng), và `GlossaryTier` (rà
/// soát 2026-09-18 — ledger của `core::ai::rag` phải mang lại `tier` mà cửa đã tính, để Story
/// 4.7 và AC "lưới/ledger cùng span" đọc được nó tại ledger, không chỉ tại cửa). Bất kỳ tên
/// nào khác — kể cả nằm TRONG một nhóm `use …glossary::{` nhiều dòng, kể cả chỉ xuất hiện
/// trong chính lời `use` mà không có lời gọi bare nào theo sau, kể cả một glob `glossary::*`
/// — là một đường thứ hai vào dữ liệu Glossary, đúng thứ Decision 5 cấm.
const ALLOWED_GLOSSARY_NAMES_UNDER_AI: [&str; 5] = [
    "confirmed_terms_for_injection",
    "GlossaryInjectionOutcome",
    "GlossaryError",
    "match_lang_for_source_lang",
    "GlossaryTier",
];

/// Nối mọi dòng MÃ (bỏ dòng bắt đầu bằng `//`) của một khối văn bản thành MỘT chuỗi.
///
/// 🔴 **BẮT BUỘC cho gate này**: xem §Vì sao ở đầu cụm — một nhóm `use …glossary::{` nhiều
/// dòng đọc TỪNG DÒNG độc lập không bao giờ thấy hết nhóm.
fn joined_code(text: &str) -> String {
    let mut out = String::new();
    for (_, line) in code_lines(text) {
        out.push_str(line);
        out.push('\n');
    }
    out
}

/// Định danh hợp lệ về CÚ PHÁP Rust: chữ/số ASCII hoặc `_`.
fn is_ident_char(c: char) -> bool {
    c.is_ascii_alphanumeric() || c == '_'
}

/// Trích MỌI định danh xuất hiện ngay sau `glossary::` trong `joined` (đã nối toàn văn qua
/// [`joined_code`] — KHÔNG còn ranh giới dòng nào để mà mất dấu) — cả hình dạng MỘT tên
/// (`glossary::confirmed_terms_for_injection(..)`, `use …glossary::load_tier;`), hình dạng
/// NHÓM `{a, b, c}` của `use` (KỂ CẢ khi nhóm đó có `\n` THẬT bên trong `{...}`, đúng khuyết
/// tật Pass 1 đo được), lẫn glob `glossary::*` (được ghi lại thành tên `"*"`, một tên KHÔNG
/// BAO GIỜ khớp bốn tên hợp lệ — một glob TỰ NÓ đã là vi phạm, vì nó phơi MỌI tên tương lai
/// của module kia, không riêng bốn tên đã ký).
///
/// ⚠️ **GIỚI HẠN THẬT, ghi ra thay vì giấu:**
/// - Không theo dõi `use … as alias;` rồi `alias::name(..)` — bí danh cho cả MODULE (không
///   phải bí danh một tên) làm chuỗi `glossary::` biến mất khỏi chính lời gọi sau đó. Không
///   tệp nào trong cây hôm nay viết vậy; nếu có ngày nào đó viết, đây là một khoảng trống có
///   tên, không phải một khoảng trống bị giấu.
/// - `joined` (qua [`joined_code`]/[`code_lines`]) chỉ bỏ dòng MÃ bắt đầu bằng `//` — một
///   chú thích ĐUÔI DÒNG (`let x = 1; // glossary::load_tier`) hay một khối `/* … */` nhắc
///   tên cấm vẫn còn nguyên trong `joined`, nên gate THẬT ([`no_core_ai_file_names_a_core_glossary_identifier_outside_the_allowed_four`])
///   sẽ đỏ KHÔNG VÌ MỘT VI PHẠM THẬT — một đỏ OAN, không phải một đỏ bỏ sót. Cùng giới hạn
///   `code_lines` đã mang từ mọi tệp `*_boundary.rs` khác trong kho; sửa đòi một bộ tách
///   chú thích thật (không còn là so chuỗi), ngoài phạm vi rà soát này.
/// - Một nhóm LỒNG (`use …::{a, b::{c}}`) dừng ở dấu `}` ĐẦU TIÊN gặp được — `inner` sẽ chỉ
///   là `"a, b::{c"`, thiếu dấu đóng của nhóm ngoài; hình dạng này không xuất hiện trong kho
///   hôm nay (Glossary tái xuất phẳng ở gốc module, không lồng theo `store::{...}`), nên đây
///   là một khoảng trống có tên, không phải một khoảng trống đã kiểm.
fn glossary_names_named(joined: &str) -> Vec<String> {
    const ANCHOR: &str = "glossary::";
    let mut out = Vec::new();
    let mut search_from = 0usize;
    let bytes = joined.as_bytes();

    while let Some(rel) = joined[search_from..].find(ANCHOR) {
        let after_anchor = search_from + rel + ANCHOR.len();
        let mut i = after_anchor;
        while i < bytes.len() && (bytes[i] as char).is_whitespace() {
            i += 1;
        }
        if i >= bytes.len() {
            break;
        }

        if bytes[i] == b'*' {
            // Glob `use …glossary::*;` -- tu no la vi pham (phoi MOI ten tuong lai cua module
            // kia), ghi lai thanh mot "ten" khong bao gio khop bon ten hop le.
            out.push("*".to_owned());
            search_from = i + 1;
        } else if bytes[i] == b'{' {
            let Some(close_rel) = joined[i + 1..].find('}') else {
                break; // nhom khong khep -- dung quet, dung khuon `scan_markers` cho `{{` mo coi
            };
            let inner = &joined[i + 1..i + 1 + close_rel];
            for item in inner.split(',') {
                let Some(raw_name) = item.split_whitespace().next() else { continue };
                let name: String = raw_name.chars().filter(|c| is_ident_char(*c)).collect();
                if !name.is_empty() && name != "self" {
                    out.push(name);
                }
            }
            search_from = i + 1 + close_rel + 1;
        } else {
            let start = i;
            let mut j = i;
            while j < bytes.len() && is_ident_char(bytes[j] as char) {
                j += 1;
            }
            let name = &joined[start..j];
            if !name.is_empty() && name != "self" {
                out.push(name.to_owned());
            }
            search_from = if j > after_anchor { j } else { after_anchor + 1 };
        }
    }

    out
}

/// 🔴 Cổng THẬT — chỉ năm tên trong [`ALLOWED_GLOSSARY_NAMES_UNDER_AI`] được phép xuất hiện
/// sau `glossary::` dưới `core/ai/**`.
#[test]
fn no_core_ai_file_names_a_core_glossary_identifier_outside_the_allowed_four() {
    let files = all_rust_sources();
    let mut violations: Vec<String> = Vec::new();
    let mut total_names_collected = 0usize;
    let mut ai_files = 0usize;

    for (rel, text) in &files {
        if !is_inside_ai_module(rel) {
            continue;
        }
        ai_files += 1;
        let joined = joined_code(text);
        let names = glossary_names_named(&joined);
        total_names_collected += names.len();
        for name in names {
            if !ALLOWED_GLOSSARY_NAMES_UNDER_AI.contains(&name.as_str()) {
                violations.push(format!("{rel}  {name}"));
            }
        }
    }

    assert!(ai_files > 0, "không tệp nào khớp `{AI_DIR}` — đường dẫn miễn trừ đã lệch");

    // Đối chứng dương: phép quét phải THẬT SỰ thu được ít nhất một tên — cùng đối chứng mà
    // Pass 1 đòi ("assert the scan actually collected a non-zero number of names"). `rag.rs`
    // gọi `confirmed_terms_for_injection` thật; 0 tên nghĩa là phép quét đang mù, đúng khuyết
    // tật vị từ cũ mắc phải.
    assert!(
        total_names_collected > 0,
        "phép quét KHÔNG thu được một tên `core::glossary` nào dưới `core/ai/**` — đây đúng \
         khuyết tật Pass 1 đo được (vị từ cũ đọc từng dòng, không bao giờ thấy hết một nhóm \
         `use` nhiều dòng). `core::ai::rag` phải gọi `confirmed_terms_for_injection` thật."
    );

    assert!(
        violations.is_empty(),
        "{} tên `core::glossary` NGOÀI năm tên được phép xuất hiện dưới `core/ai/**`:\n{}\n\n\
         Decision 5 của spec 4.6: `core::ai::rag` chỉ được gọi ĐÚNG MỘT cửa vào Glossary \
         (`confirmed_terms_for_injection`), kiểu nó trả về (`GlossaryInjectionOutcome`), \
         `GlossaryError`, `match_lang_for_source_lang`, và `GlossaryTier`. Bất kỳ tên nào \
         khác là một đường thứ hai vào dữ liệu Glossary.",
        violations.len(),
        violations.join("\n")
    );
}

/// 🔴 Đối chứng dương — vị từ [`glossary_names_named`] thu được TÊN CẤM bên trong một nhóm
/// `use …glossary::{` THẬT NHIỀU DÒNG (dấu `\n` thật bên trong `{...}`, đúng hình dạng
/// `rustfmt` xuống dòng cho một nhóm từ ba tên trở lên) — chính khuyết tật Pass 1 đo được.
#[test]
fn the_glossary_names_scan_actually_collects_a_genuine_multi_line_use_group() {
    let joined = joined_code(
        "use crate::core::glossary::{\n    confirmed_terms_for_injection,\n    load_tier,\n};\n",
    );
    assert!(
        joined.contains('\n'),
        "tien de: `joined` phai con giu mot `\\n' THAT ben trong nhom `{{...}}` -- neu khong \
         ca nay khong con nghiem thu dung khuyet tat Pass 1"
    );

    let names = glossary_names_named(&joined);
    assert_eq!(
        names,
        vec!["confirmed_terms_for_injection".to_owned(), "load_tier".to_owned()],
        "mot nhom `use` NHIEU DONG phai duoc quet HET -- day dung khuyet tat Pass 1: vi tu \
         dong cu khong bao gio thay het mot nhom nhu the nay"
    );
}

/// Đối chứng ÂM — một dòng sạch (không `glossary::` nào) và một lời gọi CÙNG tên trong CHÍNH
/// module `core::glossary` (tự mình gọi mình, không mang tiền tố `glossary::`) không bị bắt
/// oan; và bốn tên hợp lệ không bị báo vi phạm khi chúng LÀ bốn tên được phép.
#[test]
fn the_glossary_names_scan_does_not_flag_allowed_names_or_unrelated_code() {
    let clean = joined_code("let x = 1 + 2;\nfn stub() {}\n");
    assert!(glossary_names_named(&clean).is_empty());

    let allowed = joined_code(
        "use crate::core::glossary::{\n    confirmed_terms_for_injection,\n    GlossaryInjectionOutcome,\n    GlossaryError,\n    match_lang_for_source_lang,\n    GlossaryTier,\n};\n",
    );
    let names = glossary_names_named(&allowed);
    for name in &names {
        assert!(
            ALLOWED_GLOSSARY_NAMES_UNDER_AI.contains(&name.as_str()),
            "{name:?} phai la mot trong nam ten duoc phep"
        );
    }
    assert_eq!(names.len(), 5);
}

/// Đối chứng dương — glob `use crate::core::glossary::*;` phải bị bắt là một vi phạm (tên
/// `"*"`, không khớp bất kỳ tên nào trong năm tên hợp lệ).
#[test]
fn a_glob_import_of_the_glossary_module_is_flagged_as_a_violation() {
    let joined = joined_code("use crate::core::glossary::*;\n");
    let names = glossary_names_named(&joined);
    assert_eq!(names, vec!["*".to_owned()]);
    assert!(
        !ALLOWED_GLOSSARY_NAMES_UNDER_AI.contains(&"*"),
        "\"*\" khong duoc phep khop bat ky ten hop le nao -- glob tu no la vi pham"
    );
}

// ═════════════════════════════════════════════════════════════════════════════════
// Story 4.6 — cửa Glossary được GỌI (invocation) tối đa MỘT LẦN dưới `core/ai/**`
// ═════════════════════════════════════════════════════════════════════════════════

/// Đếm số LỜI GỌI (tên đứng ngay trước `(`, sau khi bỏ khoảng trắng, và không đứng liền sau
/// một ký tự định danh khác — neo biên trái) của `name` trong `joined`. Một lời `use …name;`
/// KHÔNG mang `(` ngay sau tên nên KHÔNG được tính — "gọi tối đa một lần" nói về LỜI GỌI,
/// không phải lời NHẬP.
fn count_calls(joined: &str, name: &str) -> usize {
    let mut count = 0usize;
    let mut search_from = 0usize;
    let bytes = joined.as_bytes();

    while let Some(rel) = joined[search_from..].find(name) {
        let at = search_from + rel;
        let before_ok = at == 0 || !is_ident_char(bytes[at - 1] as char);
        let after = &joined[at + name.len()..];
        let after_ok = before_ok && after.trim_start().starts_with('(');
        if after_ok {
            count += 1;
        }
        search_from = at + name.len();
    }

    count
}

/// 🔴 Cổng THẬT — `confirmed_terms_for_injection` được GỌI tối đa MỘT LẦN trên toàn bộ
/// `core/ai/**`. Rationale (spec 4.6): *"exactly one query" là một AC mà không gì cưỡng chế
/// hôm nay; một chỗ gọi thứ hai để mọi bộ test khác xanh*.
#[test]
fn the_glossary_injection_door_is_called_at_most_once_under_core_ai() {
    const DOOR: &str = "confirmed_terms_for_injection";
    let files = all_rust_sources();
    let mut total_calls = 0usize;

    for (rel, text) in &files {
        if !is_inside_ai_module(rel) {
            continue;
        }
        total_calls += count_calls(&joined_code(text), DOOR);
    }

    // 🔴 `== 1`, không `<= 1` — rà soát 2026-09-18. `<= 1` cho `0` qua: xoá chỗ gọi DUY NHẤT
    // (`gather_glossary_context`) làm gate này XANH trong khi cả module `rag` mất khả năng
    // gom dữ liệu Glossary — một hồi quy nặng hơn "gọi hai lần" mà `<= 1` không thấy.
    assert_eq!(
        total_calls, 1,
        "`{DOOR}` được GỌI {total_calls} lần dưới `core/ai/**` -- Decision 5 đòi ĐÚNG MỘT \
         truy vấn Glossary cho mỗi câu (không nhiều hơn MỘT, và không ÍT hơn MỘT — 0 lần \
         nghĩa là chỗ gọi sản phẩm duy nhất đã biến mất)."
    );
}

/// Đối chứng dương + âm cho [`count_calls`] — chứng minh nó đếm LỜI GỌI, không đếm lời NHẬP,
/// và sẽ bắt được một lần gọi thứ hai nếu ai đó thêm vào.
#[test]
fn the_call_counter_counts_invocations_not_imports_and_would_flag_a_second_call_site() {
    let import_only = joined_code("use crate::core::glossary::confirmed_terms_for_injection;\n");
    assert_eq!(
        count_calls(&import_only, "confirmed_terms_for_injection"),
        0,
        "mot loi NHAP khong mang `(` ngay sau ten -- khong duoc tinh la mot loi GOI"
    );

    let one_call = "let a = confirmed_terms_for_injection(r, g, w, s, l)?;\n";
    assert_eq!(count_calls(&joined_code(one_call), "confirmed_terms_for_injection"), 1);

    let two_calls = format!("{one_call}let b = confirmed_terms_for_injection(r, g, w, s, l)?;\n");
    assert_eq!(
        count_calls(&joined_code(&two_calls), "confirmed_terms_for_injection"),
        2,
        "ca DUONG THAT: hai loi GOI phai duoc dem la hai, khong phai mot"
    );
}

// ═════════════════════════════════════════════════════════════════════════════════
// Story 4.7, Decision 1 — tên tệp seam ① được miễn trừ khỏi cổng bare-token
// ═════════════════════════════════════════════════════════════════════════════════
//
// 🔵 **ĐÓNG 2026-09-21 (spec 4.8, Phase 2) — `deferred-work.md:10840`.** Nửa "seam ②"
// (`lib.rs`, khớp qua `line_is_the_approved_ai_prompt_seam_in_lib_rs`) của ca dưới đây đã bị
// XOÁ cùng chính vị từ đó — xem 🔵 ở doc-comment [`AI_PROMPT_SEAM_LIB_RS_FILE`]. Ca này giờ chỉ
// còn đối chứng seam ① (tên tệp lệnh).

/// 🔴 Đối chứng dương + âm cho tên tệp seam ① (`commands/aiprompt.rs`) Decision 1 miễn trừ —
/// khớp CHÍNH XÁC đường dẫn đã ký, và KHÔNG khớp một hàng xóm gần giống. Đây đúng lớp lỗi
/// XANH GIẢ mà `is_inside_ai_module` đã bị bắt một lần (`core/aim` khớp lọt qua một hằng số
/// theo tiền tố) — ca này khoá lại rằng vị từ ở đây không lặp lại cùng khuyết tật.
#[test]
fn the_approved_ai_prompt_command_file_is_matched_narrowly_and_neighbours_are_not() {
    assert!(
        is_the_approved_ai_prompt_command_file(AI_PROMPT_SEAM_COMMAND_FILE),
        "ca dương thật: đúng đường dẫn Decision 1 ký phải khớp"
    );
    for hang_xom in [
        "commands/aiprompt2.rs",
        "commands/aipromptset.rs",
        "commands/aiprompt/mod.rs",
        "commands/aiprompt_test.rs",
        "core/commands/aiprompt.rs",
        "commands/aiprompt.rs.bak",
    ] {
        assert!(
            !is_the_approved_ai_prompt_command_file(hang_xom),
            "XANH OAN: {hang_xom:?} là một tệp HÀNG XÓM, không phải đúng seam Decision 1 ký \
             -- một hằng số khớp theo tiền tố sẽ tha oan nó, đúng khuyết tật `core/aim`"
        );
    }
}

/// 🔴 Story 4.7 loop 2, finding P8 -- một dòng mang chuỗi con đã duyệt (Decision 1) không được
/// BỎ QUA TRỌN: nó vẫn phải bị quét cho một token cấm THỨ HAI đứng CÙNG DÒNG. Chứng minh cho
/// [`line_with_marker_occurrences_removed`] (hàm cổng thật ở trên gọi), độc lập với cây nguồn
/// hôm nay có gì -- cùng khuôn ca dương thật của [`ai_rag_names_named_collects_a_multiline_use_group_and_a_seeded_forbidden_name`].
///
/// 🔵 **RÚT GỌN 2026-09-21 (spec 4.8, Phase 2) — `deferred-work.md:10840`.** Ca này từng mang
/// thêm một nửa "Seam ② (lib.rs)" test cùng lý luận trên marker
/// [`AI_PROMPT_SEAM_LIB_RS_MARKER`]; nửa đó bị xoá cùng lượt xoá nhánh miễn trừ lib.rs khỏi
/// cổng thật — xem 🔵 ở doc-comment [`AI_PROMPT_SEAM_LIB_RS_FILE`]. Chỉ còn seam ①
/// (`commands/aiprompt.rs`) là nhánh THẬT của cổng bare-token hôm nay.
#[test]
fn a_line_carrying_the_approved_prefix_is_still_scanned_for_a_second_forbidden_token() {
    // Dòng SẠCH, chỉ mang tiền tố đã duyệt -- không được bắt oan sau khi xoá tiền tố.
    let clean = "use crate::core::ai::rag::{assemble_prompt, gather_glossary_context};";
    assert_eq!(
        line_names_a_forbidden_ai_dependency(&line_with_marker_occurrences_removed(
            clean,
            AI_PROMPT_SEAM_COMMAND_FILE_MARKER
        )),
        None,
        "dòng CHỈ mang tiền tố đã duyệt không được bị bắt oan sau khi gỡ tiền tố"
    );

    // Ca DƯƠNG THẬT -- cùng dòng, CỘNG một token cấm KHÁC đứng kề, phải vẫn bị bắt. Trước bản
    // sửa P8, dòng này bị `continue` bỏ qua TRỌN chỉ vì nó cũng mang tiền tố đã duyệt.
    let seeded = "use crate::core::ai::rag::{assemble_prompt}; let _ = super::ai::warm_up();";
    assert_eq!(
        line_names_a_forbidden_ai_dependency(&line_with_marker_occurrences_removed(
            seeded,
            AI_PROMPT_SEAM_COMMAND_FILE_MARKER
        )),
        Some("super::ai"),
        "một token cấm THỨ HAI đứng cùng dòng với tiền tố đã duyệt phải vẫn bị bắt, không được \
         bỏ qua TRỌN theo dòng chỉ vì dòng đó cũng mang tiền tố hợp lệ"
    );
}

// ═════════════════════════════════════════════════════════════════════════════════
// Spec 4.8, Phase 1, Task 2 — đối chứng dương + âm cho seam THỨ BA (`commands/aitranslate.rs`)
// ═════════════════════════════════════════════════════════════════════════════════

/// Cùng khuôn [`the_approved_ai_prompt_command_file_is_matched_narrowly_and_neighbours_are_not`], trên
/// seam THỨ BA -- khớp CHÍNH XÁC hình dạng spec 4.8 Task 2 đặt tên, và KHÔNG khớp một hàng
/// xóm gần giống (cùng lớp lỗi XANH GIẢ mà `core/aim` đã bị bắt một lần cho seam ①/module
/// `ai`).
#[test]
fn the_third_approved_seam_is_matched_narrowly_and_neighbours_are_not() {
    // ── ① Tệp lệnh -- khớp NGUYÊN VĂN đường dẫn, không theo tiền tố ───────────────
    assert!(
        is_the_approved_ai_translate_command_file(AI_TRANSLATE_SEAM_COMMAND_FILE),
        "ca dương thật: đúng đường dẫn Task 2 đặt tên phải khớp"
    );
    for hang_xom in [
        "commands/aitranslate2.rs",
        "commands/aitranslateset.rs",
        "commands/aitranslate/mod.rs",
        "commands/aitranslate_test.rs",
        "core/commands/aitranslate.rs",
        "commands/aitranslate.rs.bak",
        "commands/aiprompt.rs",
    ] {
        assert!(
            !is_the_approved_ai_translate_command_file(hang_xom),
            "XANH OAN: {hang_xom:?} là một tệp HÀNG XÓM (hoặc seam KHÁC), không phải đúng seam              THỨ BA -- một hằng số khớp theo tiền tố sẽ tha oan nó, đúng khuyết tật `core/aim`"
        );
    }

    // ── ② Dòng trong tệp lệnh -- khớp tiền tố module, không trọn dòng ─────────────
    assert!(
        line_is_the_approved_ai_translate_import_in_command_file(
            AI_TRANSLATE_SEAM_COMMAND_FILE,
            "use crate::core::ai::client::TranslationClient;"
        ),
        "ca dương thật: một dòng nhập đúng tiền tố `core::ai::client::` trong đúng tệp phải khớp"
    );

    // Đối chứng âm ① -- cùng chuỗi con nhưng ở MỘT TỆP KHÁC (kể cả seam ① của aiprompt) không
    // được miễn trừ gì.
    for hang_xom_file in ["commands/other.rs", AI_PROMPT_SEAM_COMMAND_FILE, "lib.rs"] {
        assert!(
            !line_is_the_approved_ai_translate_import_in_command_file(
                hang_xom_file,
                "use crate::core::ai::client::TranslationClient;"
            ),
            "XANH OAN: chuỗi con đúng nhưng TỆP sai ({hang_xom_file:?}) -- miễn trừ seam THỨ BA              chỉ dành cho `{AI_TRANSLATE_SEAM_COMMAND_FILE}`"
        );
    }

    // Đối chứng âm ② -- đúng tệp, nhưng dòng mang tiền tố seam ① (`core::ai::rag::`), không
    // phải seam THỨ BA (`core::ai::client::`) -- hai seam không được lẫn vào nhau.
    assert!(
        !line_is_the_approved_ai_translate_import_in_command_file(
            AI_TRANSLATE_SEAM_COMMAND_FILE,
            "use crate::core::ai::rag::assemble_prompt;"
        ),
        "XANH OAN: tiền tố seam ① (`core::ai::rag::`) không được khớp miễn trừ của seam THỨ BA          (`core::ai::client::`) dù cùng tệp"
    );

    // Đối chứng âm ③ -- đúng tệp, một dòng sạch không mang chuỗi con nào -- không được khớp oan.
    assert!(
        !line_is_the_approved_ai_translate_import_in_command_file(
            AI_TRANSLATE_SEAM_COMMAND_FILE,
            "    let resolved = resolve_two_tiers(resolver, global, work);"
        ),
        "một dòng sạch, thật, của chính tệp seam THỨ BA không được khớp oan"
    );
}

/// Cùng khuôn [`a_line_carrying_the_approved_prefix_is_still_scanned_for_a_second_forbidden_token`]
/// (finding P8, Story 4.7) -- một dòng mang chuỗi con seam THỨ BA đã duyệt vẫn phải bị quét
/// cho một token cấm THỨ HAI đứng CÙNG DÒNG, không được `continue` bỏ qua TRỌN.
#[test]
fn a_line_carrying_the_approved_ai_translate_prefix_is_still_scanned_for_a_second_forbidden_token()
{
    // Dòng SẠCH, chỉ mang tiền tố đã duyệt -- không được bắt oan sau khi xoá tiền tố.
    let clean = "use crate::core::ai::client::TranslationClient;";
    assert_eq!(
        line_names_a_forbidden_ai_dependency(&line_with_marker_occurrences_removed(
            clean,
            AI_TRANSLATE_SEAM_COMMAND_FILE_MARKER
        )),
        None,
        "dòng CHỈ mang tiền tố seam THỨ BA đã duyệt không được bị bắt oan sau khi gỡ tiền tố"
    );

    // Ca DƯƠNG THẬT -- cùng dòng, CỘNG một token cấm KHÁC đứng kề, phải vẫn bị bắt.
    let seeded =
        "use crate::core::ai::client::TranslationClient; let _ = super::ai::warm_up();";
    assert_eq!(
        line_names_a_forbidden_ai_dependency(&line_with_marker_occurrences_removed(
            seeded,
            AI_TRANSLATE_SEAM_COMMAND_FILE_MARKER
        )),
        Some("super::ai"),
        "một token cấm THỨ HAI đứng cùng dòng với tiền tố seam THỨ BA đã duyệt phải vẫn bị bắt,          không được bỏ qua TRỌN theo dòng chỉ vì dòng đó cũng mang tiền tố hợp lệ"
    );
}

// ═════════════════════════════════════════════════════════════════════════════════
// Story 4.7, Decision 1 — đối chứng dương THẬT: xoá `core/ai/` + hai seam, cây còn lại vẫn
// biên dịch. FR77's real content — xem doc-comment đầu tệp §VÌ SAO ĐỐI CHỨNG DƯƠNG.
// ═════════════════════════════════════════════════════════════════════════════════
//
// Decision 1 tự ghi: AD-13 offers hai cơ chế "tương đương" — một test tự động, hoặc `ai/` là
// crate riêng để trình biên dịch cưỡng chế — và "hai cơ chế đó không tương đương NHƯ ĐÃ CÀI".
// Nghĩa là phép quét TĨNH ở trên (bare-token) không tự nó chứng minh "cây vẫn biên dịch nếu
// xoá `core/ai/`" — nó chỉ chứng minh "không ai gõ hai token cấm". Hai mệnh đề khác nhau, và
// mệnh đề FR77 cần là mệnh đề THỨ HAI. Ca dưới đây gọi THẬT `cargo check` trên một bản sao cây
// nguồn đã bị xoá `core/ai/` cộng hai seam — đây là bằng chứng biên dịch, không phải suy diễn
// từ một phép quét chuỗi.

/// Xoá đúng dòng khai `pub mod <module_name>;` (dạng TRẦN, kể cả mang comment đuôi dòng) khỏi
/// văn bản của một `mod.rs` — mô hình hoá "xoá thư mục module thì khai báo của nó (sống ở
/// `mod.rs` CHA, ngoài chính thư mục) cũng phải đi theo"; xoá thư mục một mình để lại một
/// `mod` treo mà lỗi biên dịch không nói gì về câu hỏi AD-13/FR77 đang hỏi cả — sẽ làm ca biên
/// dịch bên dưới đỏ vì một lý do KHÔNG liên quan.
///
/// Dùng CHUNG cho cả hai khai báo phải đi theo hai lượt xoá của ca này: `core/mod.rs`'s
/// `pub mod ai;` (đi theo `core/ai/`) VÀ `commands/mod.rs`'s `pub mod aiprompt;` (đi theo
/// `commands/aiprompt.rs` — bắt được THẬT bằng chính ca biên dịch dưới đây trước khi hàm này
/// tồn tại: xoá tệp mà không xoá khai báo cho `error[E0583]: file not found for module
/// aiprompt` tại `commands/mod.rs`, không phải một đoán trước).
fn mod_rs_without_declaration(text: &str, module_name: &str) -> String {
    let target = format!("pub mod {module_name};");
    let mut out = String::new();
    for line in text.lines() {
        if statement_of(line.trim_start()) == target {
            continue;
        }
        out.push_str(line);
        out.push('\n');
    }
    out
}

/// Vỏ mỏng của [`mod_rs_without_declaration`] cho đúng module `ai` — giữ TÊN CŨ vì ca đơn
/// [`the_declaration_stripper_removes_only_the_bare_ai_line_and_keeps_its_neighbours`] đã gọi
/// tên này; không đổi tên một hàm test đã có để tránh một sửa không cần thiết.
fn core_mod_rs_without_the_ai_declaration(text: &str) -> String {
    mod_rs_without_declaration(text, "ai")
}

/// Số dư dấu ngoặc nhọn của MỘT dòng — `{` đếm dương, `}` đếm âm. Đếm KÝ TỰ thô, không phân
/// tích cú pháp thật (không phân biệt trong chuỗi/comment) — chấp nhận được ở đây vì đầu vào
/// là `lib.rs` của CHÍNH kho này, một văn bản đã biết hình dạng, không phải văn bản người
/// dùng gõ tuỳ ý.
fn brace_delta(line: &str) -> i32 {
    let opens = i32::try_from(line.matches('{').count()).unwrap_or(i32::MAX);
    let closes = i32::try_from(line.matches('}').count()).unwrap_or(i32::MAX);
    opens - closes
}

/// Xoá mọi dòng của một văn bản mang `marker` (một chuỗi tiền tố module, ví dụ
/// [`AI_PROMPT_SEAM_LIB_RS_MARKER`] = `crate::commands::aiprompt::`) — để mô phỏng "seam bị
/// xoá cùng `core/ai/`" cho ca biên dịch bên dưới. **Không kiểm `rel`** — hàm này THUẦN theo
/// văn bản, chỉ được gọi khi chỗ gọi đã biết chắc `text` là nội dung của một tệp THẬT SỰ tham
/// chiếu module tương ứng theo hình dạng đó (xem các vỏ mỏng ngay dưới:
/// [`lib_rs_without_the_approved_ai_prompt_seam`]/[`project_mod_rs_without_the_ai_prompt_seam`]/
/// [`lib_rs_without_the_approved_ai_translate_seam`]).
///
/// 🔴 **THAM SỐ HOÁ 2026-09-21 (spec 4.8, Phase 2, carried task) — `marker` giờ là tham số,
/// không còn khoá cứng [`AI_PROMPT_SEAM_LIB_RS_MARKER`].** Seam THỨ BA (`commands::aitranslate`,
/// tiền tố `crate::commands::aitranslate::`) cần đúng cơ chế này trên một chuỗi neo KHÁC —
/// nhân đôi toàn bộ thân hàm (số dư ngoặc, khối nhiều dòng, `assert_eq!` cân ngoặc) cho một
/// marker thứ hai là đúng lớp trôi mà việc tách một hàm chung tồn tại để chặn. Ba vỏ mỏng gọi
/// hàm này với marker riêng của chúng; hành vi trên hai marker cũ (`AI_PROMPT_SEAM_LIB_RS_MARKER`)
/// không đổi.
///
/// 🔴 **SỬA 2026-09-18 (rà soát, finding V1) — TỔNG QUÁT HOÁ từ một hàm CHỈ dành cho `lib.rs`.**
/// Trước bản này, hàm chỉ xử lý ĐÚNG `lib.rs` (qua `line_is_the_approved_ai_prompt_seam_in_lib_rs`,
/// bản thân nó kiểm `rel == "lib.rs"`) — đúng cho tới khi finding V1 (spec 4.7, loop 1) đóng:
/// `commands/project/mod.rs::replace_open_work` được thêm MỘT tham chiếu THẬT vào
/// `crate::commands::aiprompt::LastAssembledPromptState`/`clear_last_assembled_prompt_on_work_close`
/// — seam ① giờ có HAI điểm gọi thật (`lib.rs` lẫn `commands/project/mod.rs`), không còn một.
/// Đo được thật (không phải suy đoán): chạy ca biên dịch dưới đây SAU khi thêm dòng V1 mà
/// KHÔNG tổng quát hoá hàm này cho `error[E0433]: cannot find aiprompt in commands` tại
/// `commands/project/mod.rs:5247` — seam ② bị xoá nhưng điểm gọi THỨ HAI này không đi theo,
/// đúng lớp lỗi mà `mod_rs_without_declaration` đã được tổng quát hoá một lần cho `commands/
/// mod.rs`'s khai báo, giờ lặp lại ở TẦNG THÂN HÀM thay vì tầng khai báo module.
///
/// ⚠️ **Dòng khớp marker có thể MỞ một khối nhiều dòng** (vẫn đúng như bản trước, không đổi
/// logic đó): Phase 2 (Story 4.7) thêm một nhánh `if let Some(record) = handle.try_state::<
/// crate::commands::aiprompt::LastAssembledPromptState>() { … }` ba dòng trong
/// `close_open_work` (`lib.rs`) — và V1's nhánh mới trong `project/mod.rs` mang CÙNG hình
/// dạng ba dòng. Xoá chỉ đúng dòng khớp để lại một `}` treo không còn `{` ghép cặp ⇒ văn bản
/// kết quả không còn hợp lệ cú pháp Rust — bắt được bằng chính ca biên dịch thật ở
/// [`deleting_core_ai_and_its_three_approved_seams_leaves_the_rest_of_the_tree_compiling`],
/// KHÔNG phải một đoán trước. Khi dòng khớp marker cũng MỞ một khối ([`brace_delta`] dương),
/// tiếp tục bỏ các dòng sau cho tới khi số dư ngoặc trở lại `<= 0` — bỏ CẢ KHỐI, không chỉ
/// dòng đầu. Hai hình dạng một-dòng cũ (`generate_handler!`'s mục, `app.manage(...)`) có
/// `brace_delta == 0` nên hành vi của chúng không đổi.
fn text_without_lines_matching_the_ai_prompt_seam_marker(text: &str, marker: &str) -> String {
    let mut out = String::new();
    let mut skipping_block_depth: i32 = 0;
    for line in text.lines() {
        if skipping_block_depth > 0 {
            skipping_block_depth += brace_delta(line);
            continue;
        }
        if line.contains(marker) {
            let delta = brace_delta(line);
            if delta > 0 {
                skipping_block_depth = delta;
            }
            continue;
        }
        out.push_str(line);
        out.push('\n');
    }
    // 🔴 Story 4.7, finding E7 (loop 1) — nếu số dư ngoặc CHƯA về 0 ở cuối văn bản, một dòng
    // marker đã MỞ một khối mà KHÔNG BAO GIỜ đóng lại trong phần còn lại (vd. `{` của marker
    // nằm trong một chuỗi/comment, hoặc văn bản đầu vào không cân ngoặc) — nhánh
    // `skipping_block_depth > 0 { continue; }` ở trên khi đó ÂM THẦM nuốt trọn phần còn lại
    // của tệp, và kết quả đọc như "mọi seam khác đã bị xoá sạch" trong khi thật ra là "hàm
    // này cắt cụt văn bản". `assert!` — không `debug_assert!` — vì hàm này chạy trong CẢ hai
    // đường: ca đối chứng dương biên dịch thật (nơi một kết quả sai câm lặng đọc như FR77 đã
    // qua) lẫn các ca đơn vị ngay dưới.
    assert_eq!(
        skipping_block_depth, 0,
        "so du ngoac cuoi van ban phai VE 0 -- neu khong, mot dong marker da MO mot khoi KHONG \
         BAO GIO dong lai trong phan con lai (vi du `{{` nam trong mot chuoi/comment), va ham \
         nay se AM THAM cat cut phan con lai cua van ban thay vi tra ve mot ket qua can ngoac \
         that -- xem finding E7 (Story 4.7, loop 1)"
    );
    out
}

/// Vỏ mỏng cho `lib.rs` — giữ TÊN CŨ vì các ca đơn vị đã có
/// ([`the_lib_rs_stripper_removes_only_the_approved_seam_lines`], v.v.) gọi đúng tên này.
fn lib_rs_without_the_approved_ai_prompt_seam(text: &str) -> String {
    text_without_lines_matching_the_ai_prompt_seam_marker(text, AI_PROMPT_SEAM_LIB_RS_MARKER)
}

/// Vỏ mỏng cho `commands/project/mod.rs` — điểm gọi THẬT thứ hai vào `commands::aiprompt`
/// (finding V1, loop 1: `replace_open_work` dọn [`LastAssembledPromptState`] cùng hai người
/// láng giềng `PendingImportState`/`PendingPromptImportState` khi Tác phẩm được THAY THẾ,
/// không chỉ khi ĐÓNG HẲN như `lib.rs::close_open_work`).
fn project_mod_rs_without_the_ai_prompt_seam(text: &str) -> String {
    text_without_lines_matching_the_ai_prompt_seam_marker(text, AI_PROMPT_SEAM_LIB_RS_MARKER)
}

/// Chuỗi con seam THỨ BA cần xoá KHỎI `lib.rs` để mô phỏng "seam bị xoá cùng `core/ai/`" cho
/// ca biên dịch thật bên dưới — tiền tố module mà HAI dòng `generate_handler!`
/// (`ai_translate_segment`/`ai_translate_cancel`) VÀ dòng `app.manage(...AiTranslateGeneration
/// ::default())` đều mang (đo trên `lib.rs` thật, spec 4.8 Phase 2: cả ba dòng đánh vần
/// `crate::commands::aitranslate::…`). Khác [`AI_TRANSLATE_SEAM_COMMAND_FILE_MARKER`]
/// (`crate::core::ai::client::`, tiền tố mà `commands/aitranslate.rs` dùng để gọi CÀI ĐẶT của
/// cổng) — hai chuỗi neo canh hai seam KHÁC NHAU trên hai TỆP khác nhau, không được lẫn.
const AI_TRANSLATE_SEAM_LIB_RS_MARKER: &str = "crate::commands::aitranslate::";

/// Vỏ mỏng cho `lib.rs`, seam THỨ BA — cùng khuôn [`lib_rs_without_the_approved_ai_prompt_seam`],
/// chỉ khác chuỗi neo. Cả ba dòng seam THỨ BA mang trong `lib.rs` hôm nay (`brace_delta == 0`
/// từng dòng — không dòng nào mở một khối nhiều dòng, khác `close_open_work`'s nhánh
/// `if let Some(record) = … { … }` của seam ①) nên không cần đối chứng "khối nhiều dòng" riêng.
fn lib_rs_without_the_approved_ai_translate_seam(text: &str) -> String {
    text_without_lines_matching_the_ai_prompt_seam_marker(text, AI_TRANSLATE_SEAM_LIB_RS_MARKER)
}

/// Đối chứng dương + âm cho [`core_mod_rs_without_the_ai_declaration`] trên văn bản DỰNG TAY —
/// độc lập với `core/mod.rs` thật, đúng khuôn mọi vị từ khác của tệp này.
#[test]
fn the_declaration_stripper_removes_only_the_bare_ai_line_and_keeps_its_neighbours() {
    let text = "pub mod ai;\npub mod aiconfig;\npub mod ai; // AD-13\n";
    let got = core_mod_rs_without_the_ai_declaration(text);
    assert_eq!(
        got, "pub mod aiconfig;\n",
        "phải xoá CẢ HAI dòng `pub mod ai;` (một trần, một mang comment đuôi dòng) và giữ \
         nguyên `pub mod aiconfig;`"
    );
}

/// Đối chứng dương + âm cho [`lib_rs_without_the_approved_ai_prompt_seam`] trên văn bản DỰNG
/// TAY — độc lập với `lib.rs` thật.
#[test]
fn the_lib_rs_stripper_removes_only_the_approved_seam_lines() {
    let text = "\
            crate::commands::promptset::wire::prompt_set_list,
            crate::commands::aiprompt::wire::assemble_and_record_prompt,
    app.manage(crate::commands::aiprompt::LastAssembledPromptState::new(None));
    app.manage(crate::commands::promptset::PendingPromptImportState::new(None));
";
    let got = lib_rs_without_the_approved_ai_prompt_seam(text);
    assert!(!got.contains("aiprompt"), "không còn dòng nào nhắc `aiprompt` sau khi xoá");
    assert!(
        got.contains("promptset::wire::prompt_set_list") && got.contains("PendingPromptImportState"),
        "hai dòng KHÔNG liên quan seam Decision 1 phải còn nguyên"
    );
}

/// Đối chứng dương + âm cho [`lib_rs_without_the_approved_ai_translate_seam`] trên văn bản
/// DỰNG TAY — cùng khuôn [`the_lib_rs_stripper_removes_only_the_approved_seam_lines`], seam
/// THỨ BA. Ba dòng đây là NGUYÊN VĂN hình dạng thật của `lib.rs` (hai `generate_handler!` +
/// một `app.manage`, xem doc-comment [`AI_TRANSLATE_SEAM_LIB_RS_MARKER`]).
#[test]
fn the_lib_rs_stripper_removes_only_the_approved_ai_translate_seam_lines() {
    let text = "\
            crate::commands::aiprompt::wire::ai_prompt_assemble,
            crate::commands::aitranslate::wire::ai_translate_segment,
            crate::commands::aitranslate::wire::ai_translate_cancel,
    app.manage(crate::commands::aitranslate::AiTranslateGeneration::default());
    app.manage(crate::commands::promptset::PendingPromptImportState::new(None));
";
    let got = lib_rs_without_the_approved_ai_translate_seam(text);
    assert!(!got.contains("aitranslate"), "không còn dòng nào nhắc `aitranslate` sau khi xoá");
    assert!(
        got.contains("aiprompt::wire::ai_prompt_assemble") && got.contains("PendingPromptImportState"),
        "hai dòng KHÔNG liên quan seam THỨ BA phải còn nguyên"
    );
}

/// 🔴 Đối chứng dương THẬT của khuyết tật vừa bắt được: một dòng khớp marker MỞ một khối
/// nhiều dòng (đúng hình dạng thật `close_open_work` — dòng đầu khớp marker và mở `{`, thân +
/// `}` đóng nằm ở các dòng SAU không mang marker). Hành vi CŨ (xoá đúng một dòng khớp) để lại
/// `}` treo — văn bản kết quả không hợp lệ cú pháp. Ca này khẳng định CẢ KHỐI biến mất, và
/// một khối KHÔNG liên quan seam (cũng mang `if let … {` ba dòng, hình dạng giống nhưng không
/// khớp marker) phải còn nguyên — đối chứng dương lẫn âm trên đúng một bề mặt.
#[test]
fn the_lib_rs_stripper_removes_the_whole_multi_line_block_a_marker_line_opens() {
    let text = "\
    if let Some(pending) = handle.try_state::<crate::commands::promptset::PendingPromptImportState>() {
        crate::commands::promptset::clear_pending_prompt_import_work_tier(&pending);
    }
    if let Some(record) = handle.try_state::<crate::commands::aiprompt::LastAssembledPromptState>() {
        let mut guard = record.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
        *guard = None;
    }

    if let Some(state) = handle.try_state::<crate::commands::project::OpenWorkState>() {
        let mut guard = state.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
    }
";
    let got = lib_rs_without_the_approved_ai_prompt_seam(text);

    assert!(!got.contains("aiprompt"), "không còn dòng nào nhắc `aiprompt` sau khi xoá");
    // Không `}` treo: số `{` và `}` còn lại phải bằng nhau -- nếu khuyết tật cũ tái phát (chỉ
    // xoá dòng đầu của khối), vế này đỏ vì thiếu một `{` đã bị xoá cùng dòng marker.
    assert_eq!(
        got.matches('{').count(),
        got.matches('}').count(),
        "văn bản còn lại phải CÂN NGOẶC -- một dòng bị xoá dở dang sẽ để lại `}}` treo:\n{got}"
    );
    // Khối KHÔNG liên quan (PendingPromptImportState) và khối KHÔNG liên quan còn lại
    // (OpenWorkState) phải còn NGUYÊN VẸN, kể cả thân của chúng -- không bị cuốn theo.
    assert!(
        got.contains("clear_pending_prompt_import_work_tier(&pending)"),
        "thân khối PendingPromptImportState (không liên quan seam) phải còn nguyên:\n{got}"
    );
    assert!(
        got.contains("OpenWorkState") && got.contains("PoisonError::into_inner"),
        "khối OpenWorkState (không liên quan seam) phải còn nguyên:\n{got}"
    );
}

/// 🔴 Story 4.7, finding V1 (loop 1) — cùng ca đối chứng dương/âm trên, chạy trên hình dạng
/// THẬT của `commands/project/mod.rs::replace_open_work` (điểm gọi THỨ HAI vào
/// `commands::aiprompt`, phát sinh khi V1 được đóng — xem doc-comment
/// [`text_without_lines_matching_the_ai_prompt_seam_marker`]). Không dựng lại hình dạng ba
/// dòng mới — copy NGUYÊN VĂN từ chính đoạn đã thêm vào `project/mod.rs`, để một lượt đổi
/// hình dạng thật ở đó (ví dụ gộp về một dòng) tự động phản ánh vào ca này thay vì lệch âm
/// thầm.
#[test]
fn the_project_mod_rs_stripper_removes_the_whole_multi_line_block_the_v1_clearing_branch_opens() {
    let text = "\
    if let Some(pending) = app.try_state::<crate::commands::glossary::PendingImportState>() {
        crate::commands::glossary::clear_pending_import_for_tier(
            &pending,
            crate::core::glossary::GlossaryTier::Work,
        );
    }
    if let Some(pending) = app.try_state::<crate::commands::promptset::PendingPromptImportState>() {
        crate::commands::promptset::clear_pending_prompt_import_work_tier(&pending);
    }
    if let Some(record) = app.try_state::<crate::commands::aiprompt::LastAssembledPromptState>() {
        crate::commands::aiprompt::clear_last_assembled_prompt_on_work_close(&record);
    }

    if let Some(state) = app.try_state::<OpenWorkState>() {
        drop(swap_locked(&state, new_work));
    }
";
    let got = project_mod_rs_without_the_ai_prompt_seam(text);

    assert!(!got.contains("aiprompt"), "không còn dòng nào nhắc `aiprompt` sau khi xoá");
    assert_eq!(
        got.matches('{').count(),
        got.matches('}').count(),
        "văn bản còn lại phải CÂN NGOẶC -- một dòng bị xoá dở dang sẽ để lại `}}` treo:\n{got}"
    );
    // Ba khối KHÔNG liên quan seam (glossary, promptset, OpenWorkState) phải còn NGUYÊN VẸN.
    assert!(
        got.contains("clear_pending_import_for_tier") && got.contains("GlossaryTier::Work"),
        "khối glossary (không liên quan seam) phải còn nguyên:\n{got}"
    );
    assert!(
        got.contains("clear_pending_prompt_import_work_tier(&pending)"),
        "khối promptset (không liên quan seam) phải còn nguyên:\n{got}"
    );
    assert!(
        got.contains("swap_locked(&state, new_work)"),
        "khối OpenWorkState (không liên quan seam) phải còn nguyên:\n{got}"
    );
}

/// Chép ĐỆ QUY toàn bộ `src` sang `dst` bằng `std::fs` thuần (không `git`, không lệnh shell
/// ngoài) — cây con nào tên khớp một phần tử của `skip_dir_names` bị BỎ QUA hoàn toàn (dùng
/// cho `target/`, thư mục build nặng và không liên quan tới câu hỏi "cây nguồn còn biên dịch
/// không"). Portable trên Windows lẫn macOS/Linux — không gọi `cp -r`/`git`, hai thứ đối
/// chứng dương TRƯỚC đây phụ thuộc và mỗi thứ mang một khuyết tật riêng (xem doc-comment
/// [`deleting_core_ai_and_its_three_approved_seams_leaves_the_rest_of_the_tree_compiling`] về vì
/// sao `git worktree` bị bỏ).
fn copy_dir_recursive_skipping(src: &Path, dst: &Path, skip_dir_names: &[&str]) -> std::io::Result<()> {
    fs::create_dir_all(dst)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let file_type = entry.file_type()?;
        let name = entry.file_name();
        let dst_path = dst.join(&name);
        if file_type.is_dir() {
            if skip_dir_names.iter().any(|skip| name.to_string_lossy() == *skip) {
                continue;
            }
            copy_dir_recursive_skipping(&entry.path(), &dst_path, skip_dir_names)?;
        } else if file_type.is_file() {
            fs::copy(entry.path(), &dst_path)?;
        }
        // Symlink: không tệp/thư mục nào dưới `src-tauri/` của kho này là symlink hôm nay —
        // bỏ qua có chủ ý thay vì theo đuổi một nhánh chưa từng cần tới.
    }
    Ok(())
}

/// 🔵 **ĐÓNG 2026-09-21 (spec 4.8, Phase 2, carried task) — nợ Phase 1 đã mở rộng xong.**
/// Đoạn dưới đây từng ghi rằng Phase 1 chỉ xoá `core/ai/` + HAI seam (aiprompt), vì seam THỨ
/// BA (`commands/aitranslate.rs`, [`AI_TRANSLATE_SEAM_COMMAND_FILE`]) chưa tồn tại. Tệp đó nay
/// đã có (Phase 2), nên ca dưới đây giờ xoá CẢ BA seam trong cùng một lượt: (a) xoá
/// `commands/aitranslate.rs` sau một `assert!(...is_file())` cứng, cùng khuôn finding B8 đã
/// áp cho seam ①/② — để một lượt đổi tên tệp trong tương lai không làm nhánh xoá âm thầm
/// thành NO-OP; (b) xoá khai báo `pub mod aitranslate;` ở `commands/mod.rs` qua
/// [`mod_rs_without_declaration`]; (c) xoá cả ba dòng `lib.rs` mang seam THỨ BA
/// (`crate::commands::aitranslate::…`, hai `generate_handler!` + một `app.manage`) qua
/// [`lib_rs_without_the_approved_ai_translate_seam`]. `commands/aitranslate.rs` không có điểm
/// gọi THẬT thứ hai kiểu finding V1 (`AiTranslateGeneration` là state TOÀN PHIÊN, không theo
/// Tác phẩm — không module nào khác dọn nó khi một Tác phẩm đóng/thay), nên không cần một bước
/// tương đương `project_mod_rs_without_the_ai_prompt_seam` cho seam này — đo bằng `grep -rn
/// "commands::aitranslate" src/` trước khi viết dòng này: đúng ba chỗ, cả ba trong `lib.rs`.
///
/// ⚠️ `#[ignore]` có chủ ý, đúng tiền lệ `check:scope`/`check:scope:bundled` (root
/// `AGENTS.md`: *"sit outside `pre-push` on purpose... run them by hand"*) — ca này dựng một
/// bản CHÉP RIÊNG của `src-tauri/` rồi gọi `cargo check` THẬT: tốn giây tới phút, không phải
/// milli-giây như mọi ca tĩnh khác của tệp này, và không nên nằm trong đường `cargo test
/// --locked` mặc định mà `pre-push` chạy mỗi lần push. Chạy tay:
/// `cargo test --test ai_boundary --locked -- --ignored --exact
/// deleting_core_ai_and_its_three_approved_seams_leaves_the_rest_of_the_tree_compiling`
///
/// Dọn dẹp qua `Drop` (`ProbeDirGuard`) để một panic giữa ca vẫn gỡ được thư mục tạm — nhị
/// phân test biên dịch với `unwind` (`src-tauri/AGENTS.md`), nên `Drop` vẫn chạy trên một
/// panic dù crate sản phẩm dùng `panic = "abort"`.
///
/// ⚠️ **SỬA 2026-09-18 (Phase 4 → hậu-Phase-4) — vì sao KHÔNG còn `git worktree add ... HEAD`:**
/// Phase 1 dựng ca này trên `git worktree add --detach <dir> HEAD` — `HEAD` là một COMMIT, và
/// story này (đúng khuôn mọi story khác trong kho: một commit MỖI story, ở CUỐI) chưa commit
/// lúc Phase 1–4 chạy. Kết quả: nhánh xoá `commands/aiprompt.rs`/seam của `lib.rs` bên trong
/// ca luôn là NO-OP trên `HEAD` cũ, bất kể Phase 2/3 đã viết seam thật vào ĐĨA — `PASS` đo
/// được chỉ tái lập kịch bản Phase 1, không chứng minh gì về hai tệp thật (xem
/// `deferred-work.md`, mục "Đối chứng dương của seam ... KHÔNG THỂ nghiệm thu"). Sửa bằng
/// cách bỏ `git` hoàn toàn: [`copy_dir_recursive_skipping`] chép THẲNG cây làm việc hiện tại
/// (bao gồm mọi thay đổi CHƯA commit) sang một thư mục tạm, rồi xoá/sửa trên bản chép đó —
/// nghiệm thu đúng CÂY ĐANG CÓ TRÊN ĐĨA, không phụ thuộc trạng thái commit.
#[test]
#[ignore = "copies the whole `src-tauri/` tree + runs a real `cargo check`; run by hand, see doc-comment"]
fn deleting_core_ai_and_its_three_approved_seams_leaves_the_rest_of_the_tree_compiling() {
    let src_tauri_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));

    let probe_dir =
        std::env::temp_dir().join(format!("ai_boundary_seam_probe_{}", std::process::id()));
    if probe_dir.exists() {
        let _ = fs::remove_dir_all(&probe_dir);
    }

    struct ProbeDirGuard {
        probe_dir: PathBuf,
    }
    impl Drop for ProbeDirGuard {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.probe_dir);
        }
    }
    let _guard = ProbeDirGuard { probe_dir: probe_dir.clone() };

    let probe_src_tauri_dir = probe_dir.join("src-tauri");
    // `target/` bị BỎ QUA có chủ ý -- nặng (build artifacts), không liên quan câu hỏi đang
    // hỏi, và `cargo check` dưới đây được truyền `--target-dir` trỏ THẲNG vào `target/` THẬT
    // của crate gốc để tái sử dụng mọi crate phụ thuộc đã biên dịch sẵn (xem chú thích ở lệnh
    // gọi `cargo check`).
    copy_dir_recursive_skipping(&src_tauri_dir, &probe_src_tauri_dir, &["target"]).unwrap_or_else(
        |e| panic!("khong chep duoc {} sang {}: {e}", src_tauri_dir.display(), probe_src_tauri_dir.display()),
    );

    let probe_src = probe_src_tauri_dir.join("src");

    let ai_dir = probe_src.join(AI_DIR);
    assert!(
        ai_dir.is_dir(),
        "`{}` khong ton tai trong ban chep -- cay lam viec hien tai khong con core/ai/, khong \
         con gi de xoa cho ca nay nghiem thu",
        ai_dir.display()
    );
    fs::remove_dir_all(&ai_dir).unwrap_or_else(|e| panic!("khong xoa duoc {}: {e}", ai_dir.display()));

    let core_mod_path = probe_src.join("core/mod.rs");
    let core_mod_text = fs::read_to_string(&core_mod_path)
        .unwrap_or_else(|e| panic!("khong doc duoc {}: {e}", core_mod_path.display()));
    fs::write(&core_mod_path, core_mod_rs_without_the_ai_declaration(&core_mod_text))
        .unwrap_or_else(|e| panic!("khong ghi duoc {}: {e}", core_mod_path.display()));

    // 🔴 Story 4.7, finding B8 (loop 1) — seam ① (`core/ai/`) ngay TRÊN đòi TỒN TẠI bằng một
    // `assert!` cứng; seam ② (đây) trước bản sửa chỉ xoá `if aiprompt_existed`, KHÔNG một
    // `assert!` nào ép nó phải có mặt. Một lượt đổi tên tệp trong tương lai (vd. sang
    // `commands/aiprompt/mod.rs`, đúng lớp lỗi "false-exemption" `is_inside_ai_module`'s
    // `core/aim` fix đã bắt một lần) làm `aiprompt_path.exists()` thành `false` một cách ÂM
    // THẦM — nhánh xoá bị BỎ QUA, seam ② quay lại đúng NO-OP mà toàn bộ lượt sửa away-from-
    // `git worktree` này tồn tại để loại bỏ, và ca vẫn `PASS` như không có chuyện gì. Đối
    // xứng với seam ①: TỒN TẠI là một đòi hỏi CỨNG, không một nhánh `if` che giấu một lần
    // xoá bị bỏ sót.
    let aiprompt_path = probe_src.join(AI_PROMPT_SEAM_COMMAND_FILE);
    assert!(
        aiprompt_path.is_file(),
        "`{}` khong ton tai trong ban chep -- cay lam viec hien tai khong con seam ② \
         (`commands/aiprompt.rs`), khong con gi de xoa cho ca nay nghiem thu vi giu nguyen \
         no-op cua ban cu; xem finding B8 (Story 4.7, loop 1)",
        aiprompt_path.display()
    );
    fs::remove_file(&aiprompt_path)
        .unwrap_or_else(|e| panic!("khong xoa duoc {}: {e}", aiprompt_path.display()));

    // Xoá tệp `commands/aiprompt.rs` một mình để lại `pub mod aiprompt;` treo ở `commands/
    // mod.rs` — CÙNG lớp lỗi `core/mod.rs`'s `pub mod ai;` ngay trên, chỉ khác cây con. Bắt
    // được THẬT bằng chính ca này trước khi dòng dưới đây tồn tại: `error[E0583]: file not
    // found for module aiprompt` tại `commands/mod.rs`, không phải một đoán trước.
    let commands_mod_path = probe_src.join("commands/mod.rs");
    let commands_mod_text = fs::read_to_string(&commands_mod_path)
        .unwrap_or_else(|e| panic!("khong doc duoc {}: {e}", commands_mod_path.display()));
    fs::write(&commands_mod_path, mod_rs_without_declaration(&commands_mod_text, "aiprompt"))
        .unwrap_or_else(|e| panic!("khong ghi duoc {}: {e}", commands_mod_path.display()));

    let lib_rs_path = probe_src.join(AI_PROMPT_SEAM_LIB_RS_FILE);
    let lib_rs_text = fs::read_to_string(&lib_rs_path)
        .unwrap_or_else(|e| panic!("khong doc duoc {}: {e}", lib_rs_path.display()));
    fs::write(&lib_rs_path, lib_rs_without_the_approved_ai_prompt_seam(&lib_rs_text))
        .unwrap_or_else(|e| panic!("khong ghi duoc {}: {e}", lib_rs_path.display()));

    // 🔴 Story 4.7, finding V1 (loop 1) -- diem goi THAT thu HAI vao `commands::aiprompt`
    // (`replace_open_work` dep them ba dong dua qua nhieu Tac pham). Bo dong nay thi ca bien
    // dich duoi day do THAT -- `error[E0433]: cannot find aiprompt in commands` tai
    // `commands/project/mod.rs` -- vi seam ② da bi xoa nhung diem goi THU HAI khong di theo.
    let project_mod_rs_path = probe_src.join("commands/project/mod.rs");
    let project_mod_rs_text = fs::read_to_string(&project_mod_rs_path)
        .unwrap_or_else(|e| panic!("khong doc duoc {}: {e}", project_mod_rs_path.display()));
    fs::write(&project_mod_rs_path, project_mod_rs_without_the_ai_prompt_seam(&project_mod_rs_text))
        .unwrap_or_else(|e| panic!("khong ghi duoc {}: {e}", project_mod_rs_path.display()));

    // 🔵 spec 4.8, Phase 2 (carried task) -- seam THU BA (`commands/aitranslate.rs`), cung
    // khuon finding B8 da ap cho seam ②: TON TAI la mot doi hoi CUNG truoc khi xoa, khong mot
    // nhanh `if` che giau mot lan xoa bi bo sot khi tep doi ten trong tuong lai.
    let aitranslate_path = probe_src.join(AI_TRANSLATE_SEAM_COMMAND_FILE);
    assert!(
        aitranslate_path.is_file(),
        "`{}` khong ton tai trong ban chep -- cay lam viec hien tai khong con seam THU BA \
         (`commands/aitranslate.rs`), khong con gi de xoa cho ca nay nghiem thu vi giu nguyen \
         no-op cua ban cu",
        aitranslate_path.display()
    );
    fs::remove_file(&aitranslate_path)
        .unwrap_or_else(|e| panic!("khong xoa duoc {}: {e}", aitranslate_path.display()));

    // Xoa tep `commands/aitranslate.rs` mot minh de lai `pub mod aitranslate;` treo o
    // `commands/mod.rs` -- CUNG lop loi hai buoc `mod_rs_without_declaration` ngay tren.
    let commands_mod_text_after_aiprompt = fs::read_to_string(&commands_mod_path)
        .unwrap_or_else(|e| panic!("khong doc duoc {}: {e}", commands_mod_path.display()));
    fs::write(
        &commands_mod_path,
        mod_rs_without_declaration(&commands_mod_text_after_aiprompt, "aitranslate"),
    )
    .unwrap_or_else(|e| panic!("khong ghi duoc {}: {e}", commands_mod_path.display()));

    // Ba dong seam THU BA trong `lib.rs` (hai `generate_handler!` + mot `app.manage`) -- doc
    // LAI tu dia vi buoc tren da ghi de chinh tep nay cho seam ① (`lib_rs_without_the_approved_
    // ai_prompt_seam`); khong dung `lib_rs_text` cu, no khong con khop noi dung tren dia.
    let lib_rs_text_after_aiprompt = fs::read_to_string(&lib_rs_path)
        .unwrap_or_else(|e| panic!("khong doc duoc {}: {e}", lib_rs_path.display()));
    fs::write(
        &lib_rs_path,
        lib_rs_without_the_approved_ai_translate_seam(&lib_rs_text_after_aiprompt),
    )
    .unwrap_or_else(|e| panic!("khong ghi duoc {}: {e}", lib_rs_path.display()));

    // Dung LAI target dir cua cay chinh -- moi crate phu thuoc (bang thu ba) da bien dich san
    // o do, khong phu thuoc duong dan crate goc, nen duoc TAI SU DUNG; chi crate cua chinh du
    // an (source vua doi) can bien dich lai. Khong dung target-dir MOI: mot lan build day du
    // lai toan bo cay phu thuoc la chi phi khong can thiet cho cau hoi dang hoi.
    let real_target_dir = src_tauri_dir.join("target");
    let check = std::process::Command::new("cargo")
        .args(["check", "--locked"])
        .arg("--target-dir")
        .arg(&real_target_dir)
        .current_dir(probe_dir.join("src-tauri"))
        .output()
        .unwrap_or_else(|e| panic!("khong goi duoc `cargo check`: {e}"));

    assert!(
        check.status.success(),
        "`cargo check` THAT BAI sau khi xoa `core/ai/` + ba seam duoc phe duyet -- FR77 (go \
         sach cau hinh AI thi moi nang luc khac van chay day du) khong con dung, hoac mot noi \
         NGOAI ba seam van dang phu thuoc `core/ai/`:\nSTDOUT:\n{}\nSTDERR:\n{}",
        String::from_utf8_lossy(&check.stdout),
        String::from_utf8_lossy(&check.stderr)
    );
}
