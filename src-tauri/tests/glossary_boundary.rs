//! Ranh giới cây nguồn của Story 3.1/3.2 — `glossary_entry`/`glossary_candidate` chỉ tồn
//! tại dưới `core/glossary/**` và `core/store/schema.rs` (nơi hai hằng DDL sống).
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
//! 🔵 CẬP NHẬT 2026-08-20 (Story 3.2) — CHUỖI CỨNG LÀ LỖ HỔNG CÓ TÊN, VÁ TRONG LƯỢT NÀY
//! ─────────────────────────────────────────────────────────────────────────────
//! Cổng gốc của Story 3.1 so **một chuỗi cứng duy nhất** (`"glossary_entry"`) — một bảng
//! khác tên đi lọt hoàn toàn. Story 3.2 tự nó là bằng chứng: `glossary_candidate` ra đời
//! và nếu `FORBIDDEN` không đổi, cổng này XANH GIẢ trên một module Epic 8 viết thẳng `SELECT
//! … FROM glossary_candidate …` để lách qua `approve_candidate`/`reject_candidate`. Vá lỗ
//! đó CÙNG LƯỢT — không để nó chờ tới khi Epic 8 tồn tại rồi mới bị bắt, đúng lý do cổng
//! này được dựng TRƯỚC Epic 4 ở đoạn trên.
//!
//! ─────────────────────────────────────────────────────────────────────────────
//! 🔴 SÀN SỐ TỆP + ĐỐI CHỨNG DƯƠNG LÀ BẮT BUỘC
//! ─────────────────────────────────────────────────────────────────────────────
//! *"Cây rỗng đọc thành sạch"* — bài học kế thừa từ `scope_boundary.rs`/`store_boundary.rs`.
//! Sàn bắt một cây bị **cắt mất**; đối chứng dương bắt ca *"không ai vi phạm"* và *"không
//! có gì để vi phạm"* đọc giống nhau.

use std::fs;
use std::path::{Path, PathBuf};

/// Thư mục DUY NHẤT được phép mang tên bảng `glossary_entry`/`glossary_candidate` bằng mã
/// sản phẩm SQL (`insert`/`select`/`update` thật).
const GLOSSARY_DIR: &str = "core/glossary";

/// Tệp DUY NHẤT khác `GLOSSARY_DIR` được phép — nơi `GLOSSARY_ENTRY_DDL`/
/// `GLOSSARY_CANDIDATE_DDL` khai `CREATE TABLE`, đúng khuôn mọi hằng DDL khác của kho
/// (`schema.rs` sở hữu MỌI tên bảng, không chỉ hai tên này).
const SCHEMA_FILE: &str = "core/store/schema.rs";

/// Số tệp `.rs` tối thiểu dưới `src-tauri/src/**` để phép quét là thật.
///
/// 🔵 **CẬP NHẬT 2026-08-20 (Story 3.3) — đo lại, nâng sàn về dải 80–85%.** Số thật lúc dựng
/// story này: **50** tệp — cây đi từ 49 (Story 3.2) qua đúng MỘT tệp mới,
/// `commands/glossary.rs` (bề mặt IPC đầu tiên của module). Sàn cũ (**38**, ~77,6%) đã tụt
/// dưới dải 80–85% mà tác vụ cổng biên của story này đòi kiểm lại; nâng lên **40** (80%) —
/// vẫn dưới số thật đúng khuôn `RS_FLOOR` của `scope_boundary.rs`/`store_boundary.rs`: nó
/// bắt một cây bị cắt mất, không bắt việc thêm tệp mới.
///
/// ⚠️ **ĐO LẠI 2026-08-21 (Story 3.4) — KHÔNG NÂNG, số thật KHÔNG đổi.** Story này sửa **bảy**
/// tệp `.rs` có sẵn (`core/matching/mod.rs` · `core/glossary/entry.rs` · `…/store.rs` ·
/// `…/mod.rs` · `commands/glossary.rs` · `commands/chapter.rs` · `lib.rs`) và không thêm một
/// tệp `.rs` MỚI nào dưới `src-tauri/src/**` — quần thể vẫn **50**. Một sàn nâng khi số thật
/// đứng nguyên là một sàn nâng theo cảm giác, đúng thứ `check-i18n.mjs::RS_FLOOR` (Story
/// 1.20) đã từ chối một lần.
const RS_FLOOR: usize = 44; // 🔵 SUA 2026-08-22 (ra ba lop) -- 43/53 = 81% VA `check-i18n.mjs::RS_FLOOR`
// = 44/53 = 83% khong khop nhau tren CUNG mot quan the (53 tep .rs duoi src-tauri/src/**,
// da doi chieu: check-i18n.mjs quet ca `src-tauri/**` + `tools/**` roi tru mien tru
// `tests/**`/`tools/**`, con lai dung 53 -- cung so voi cong nay). Hai cong khong co ly do
// chinh dang de lech nhau tren cung mot con so; nang len 44 cho khop.

/// Chuỗi bị cấm ngoài hai vị trí ở trên — **tên bảng thật**, chữ thường nguyên văn như nó
/// nằm trong SQL (`CREATE TABLE glossary_entry`, `FROM glossary_candidate`, …).
///
/// ⚠️ Không phải `"Glossary"` (tên module/kiểu Rust, viết hoa) và không phải `"glossary"`
/// (khoá dây của `ScopeKind::Glossary`, `core/scope/kinds.rs:162`) — cả hai token đó xuất
/// hiện hợp lệ ở khắp nơi (`core/scope/**`, doc-comment, `deferred-work.md`). Chỉ CHUỖI
/// TÊN BẢNG mới là thứ cổng này canh: đó là chỗ duy nhất "chạm dữ liệu Glossary" có nghĩa
/// theo đúng câu mà `epic-3-context.md` dùng.
///
/// 🔵 **CẬP NHẬT 2026-08-20 (Story 3.2) — từ MỘT chuỗi trần thành DANH SÁCH.** Bản Story
/// 3.1 so đúng một hằng `&str`. `glossary_candidate` ra đời cùng lược đồ khác hẳn
/// `glossary_entry`, và một chuỗi trần thứ hai không so được — mảng là hình dạng đúng cho
/// "MỘT TẬP tên bảng bị cấm", không phải "một tên bảng".
const FORBIDDEN_TABLES: [&str; 2] = ["glossary_entry", "glossary_candidate"];

/// Vế THỨ HAI của AD-36 — *"`ai/` không có đường nào khác chạm dữ liệu Glossary"*.
///
/// `core::glossary::mod` tái xuất công khai `insert_manual_entry` · `confirm_translation` ·
/// `load_tier` · `entries_eligible_for_injection`. Ba cái đầu phơi dữ liệu THÔ —
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
///
/// 🔵 **CẬP NHẬT 2026-08-20 (Story 3.2) — `insert_entry` đổi tên `insert_manual_entry`.**
/// Câu tương ứng vẫn đúng: chữ ký mới không nhận `term_origin` nữa nhưng vẫn phơi dữ liệu
/// THÔ (ghi thẳng, không qua điều kiện chèn), nên vẫn chỉ được gọi trong
/// `core/glossary/**`. ⚠️ **Bốn hàm mới của `candidate_store` (`insert_candidate` ·
/// `pending_candidates` · `approve_candidate` · `reject_candidate`) CỐ Ý CHƯA vào danh sách
/// này** — chưa có chỗ gọi sản phẩm nào ngoài `core/glossary/**` để nghiệm thu một quyết
/// định (có nên hạn chế `pending_candidates`/`approve_candidate` hay không phụ thuộc vào
/// hình dạng bề mặt Story 3.3/3.5/3.8 dựng). Món nợ có chủ ở `deferred-work.md`.
/// 🔵 **CẬP NHẬT 2026-08-22 (Story 3.5) — hàm THỨ TƯ, `insert_candidate`.** Nửa món nợ
/// `deferred-work.md:5630-5643` đóng ở đây: bốn hàm `candidate_store` (Story 3.2) chờ
/// đúng "story dựng chỗ gọi sản phẩm đầu tiên" quyết định — ứng viên gần nhất ghi trong
/// sổ nợ là *"Story 3.5 (`insert_candidate`)"*. Đo lại (`grep insert_candidate
/// src-tauri/src/**` ngoài `core/glossary/**`): **0** chỗ gọi. Story này KHÔNG gọi hàm
/// đơn lẻ đó — nó dựng một hàm ghi LÔ mới (`insert_import_scan_candidates`, `WHERE NOT
/// EXISTS` + `ON CONFLICT DO NOTHING`) mà `commands::project` gọi thay. `insert_candidate`
/// vì thế khoá lại giống `insert_manual_entry`/`confirm_translation`/`load_tier`: phơi dữ
/// liệu THÔ (một `INSERT` trần, không lọc `glossary_entry`, không đếm `(đã chèn, đã bỏ
/// qua)`), và một chỗ gọi sản phẩm tương lai dùng nó thay vì hàm lô mới sẽ lách qua đúng
/// luật *"lọc `glossary_entry` trước khi ghi"* mà story này vừa dựng. `pending_candidates`/
/// `approve_candidate`/`reject_candidate` GIỮ NGUYÊN ngoài danh sách này — `pending_
/// candidates` nay có chỗ gọi sản phẩm thật (`commands::glossary::glossary_pending_
/// candidates`, xem `QUICK_ADD_SURFACE`); `approve_candidate`/`reject_candidate` vẫn là
/// nợ MỞ, chủ Story 3.8.
const GLOSSARY_ONLY_SURFACE: [&str; 4] =
    ["insert_manual_entry", "confirm_translation", "load_tier", "insert_candidate"];

/// 🔵 **THÊM 2026-08-20 (Story 3.3)** — vị từ THUẦN dùng bởi cổng thật
/// ([`only_entries_eligible_for_injection_may_be_called_from_outside_glossary`]) **VÀ**
/// đối chứng mới bên dưới, cùng lý do [`line_spells_a_non_manual_term_origin_token`]: hai
/// bên không thể lệch nhau bằng cách trùng lặp logic so chuỗi ở hai chỗ khác nhau.
fn line_calls_a_glossary_only_surface_function(code: &str) -> Option<&'static str> {
    GLOSSARY_ONLY_SURFACE.into_iter().find(|needle| code.contains(needle))
}

/// Bốn hàm mà `commands::glossary` ĐƯỢC PHÉP gọi xuống `core::glossary` — bề mặt Story 3.3
/// dựng đúng để thay thế `GLOSSARY_ONLY_SURFACE` cho chỗ gọi sản phẩm đầu tiên (Ice ký ở
/// `glossary_boundary.rs:80-88`, tiền lệ Story 3.1: sửa CHỮ KÝ thay vì nới cổng).
///
/// 🔵 **CẬP NHẬT 2026-08-21 (Story 3.4) — hàm THỨ TƯ, `marks_for_source_text`.** Tên hằng
/// giữ nguyên `QUICK_ADD_SURFACE` dù nó nay phục vụ CẢ hai bề mặt của `commands::glossary`
/// (dải "Thêm thuật ngữ" 3.3 + đánh dấu 3.4) — đổi tên sẽ chạm hai test đã có mà không đổi
/// mệnh đề chúng canh; giữ khuôn hiện tại (một danh sách CÁC HÀM ĐƯỢC PHÉP, không phải một
/// danh sách CHỈ-CHO-QUICK-ADD) là đúng thứ Story 3.1 đã ký ở `:80-88`.
/// 🔵 **CẬP NHẬT 2026-08-22 (Story 3.5) — hàm THỨ NĂM, `pending_candidates`.** Vỏ IPC
/// CHỈ-ĐỌC mới `glossary_pending_candidates` (`commands/glossary.rs`) là chỗ gọi sản phẩm
/// ĐẦU TIÊN của hàm này (Story 3.2 dựng nó, 0 chỗ gọi cho tới lượt này) — đóng nốt nửa
/// còn lại của `deferred-work.md:5630-5643`. `pending_candidates` KHÔNG lọc gì (trả TRỌN
/// `resolution IS NULL`), nhưng đó đúng là việc một bề mặt "phơi để nghiệm thu bằng mắt"
/// (§Intent của story) cần — không có điều kiện chèn nào để tách ra như
/// `entries_eligible_for_injection` đã làm cho `glossary_entry`.
const QUICK_ADD_SURFACE: [&str; 5] = [
    "resolve_term_for_quick_add",
    "add_manual_term",
    "update_manual_term",
    "marks_for_source_text",
    "pending_candidates",
];

/// Token xuất xứ TỰ ĐỘNG — chỉ được sinh ra (biến thể enum, chuỗi `as_str()`) bên trong
/// `core/glossary/**` (AD-20: chỉ `approve_candidate` được phép suy ra một `term_origin`
/// khác `manual`, và nó suy TỪ `candidate_origin` của chính hàng ứng viên, không nhận từ
/// một tham số mà chỗ gọi ngoài tự đặt).
///
/// ⚠️ Đây là vế mà Story 3.1 chỉ NHẮC TỚI trong doc-comment (`entry.rs::TermOrigin`) chứ
/// chưa CANH: trước Story 3.2, không gì ngăn một module khác viết
/// `"import_scan".to_owned()` thẳng và truyền vào `insert_entry` cũ. Chữ ký mới của
/// `insert_manual_entry` (không tham số `term_origin`) đã đóng phần lớn lỗ đó, nhưng cổng
/// này đóng phần còn lại: không tệp nào ngoài `core/glossary/**` được PHÉP GÕ hai token
/// này, dù có gọi được hàm nào hay không.
///
/// 🔵 **CẬP NHẬT 2026-08-20 (lượt rà soát ba lớp) — CÓ ĐỊNH TÍNH `TermOrigin::`, không còn
/// tên biến thể TRẦN.** Bản trước dùng `"ImportScan"`/`"ReviewHarvest"` trần, và
/// `CandidateOrigin` (Story 3.2, `candidate.rs`) khai **hai biến thể TRÙNG TÊN**
/// (`CandidateOrigin::ImportScan`/`CandidateOrigin::ReviewHarvest`) — một chuỗi con trần
/// khớp CẢ HAI kiểu. Tái hiện được: một tệp ngoài `core/glossary/**` gọi
/// `insert_candidate(store, term, CandidateOrigin::ImportScan)` — đúng hình dạng Story 3.5
/// CẦN GỌI để đưa một ứng viên vào bảng chờ — làm cổng đỏ OAN. Có định tính
/// `TermOrigin::` thu hẹp đúng về ý ban đầu: chỉ bắt cách VIẾT biến thể của `TermOrigin`,
/// không bắt việc dùng `CandidateOrigin` — xem bài tự kiểm
/// [`the_non_manual_origin_token_check_catches_term_origin_but_not_candidate_origin`] cho
/// bằng chứng cả hai chiều.
const NON_MANUAL_ORIGIN_TOKENS: [&str; 2] = ["TermOrigin::ImportScan", "TermOrigin::ReviewHarvest"];

/// Vị từ THUẦN dùng bởi cổng thật ([`only_core_glossary_may_spell_the_non_manual_term_origin_tokens`])
/// **VÀ** bài tự kiểm ngay dưới — tách ra để hai bên không thể lệch nhau bằng cách trùng
/// lặp logic so chuỗi ở hai chỗ khác nhau.
fn line_spells_a_non_manual_term_origin_token(code: &str) -> Option<&'static str> {
    NON_MANUAL_ORIGIN_TOKENS
        .into_iter()
        .find(|needle| code.contains(needle))
}

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
/// `glossary_entry` HOẶC `glossary_candidate`.
///
/// 🔵 **CẬP NHẬT 2026-08-20 (Story 3.2) — đổi tên từ
/// `only_glossary_and_schema_may_name_the_glossary_entry_table`, `FORBIDDEN` thành
/// `FORBIDDEN_TABLES`.** Vòng lặp ngoài mới (`for needle in FORBIDDEN_TABLES`) là toàn bộ
/// khác biệt về hành vi; mệnh đề không đổi.
#[test]
fn only_glossary_and_schema_may_name_glossary_tables() {
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
            for needle in FORBIDDEN_TABLES {
                if code.contains(needle) {
                    violations.push(format!("{rel}:{line}  {needle}  |  {code}"));
                }
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
        "{} chỗ ngoài `core/glossary/**` và `{SCHEMA_FILE}` mang tên một bảng Glossary:\n{}\n\n\
         `epic-3-context.md`: 'Epic 4 (RagInjector) phụ thuộc trực tiếp vào truy vấn mục đủ \
         điều kiện chèn mà Story 3.1 dựng — không có đường nào khác để chạm dữ liệu \
         Glossary.' Story 3.2 nói thêm: 'không cơ chế nào được tự ghi vào Glossary' (AD-20) \
         áp CẢ cho bảng chờ ứng viên. Một module khác gõ tên bảng thẳng là đúng đường tắt \
         đó. Cần dữ liệu Glossary thì gọi `core::glossary::entries_eligible_for_injection` \
         (hoặc `load_tier`/`insert_manual_entry`/`confirm_translation` cho phần còn lại của \
         vòng đời `glossary_entry`, hoặc `insert_candidate`/`pending_candidates`/ \
         `approve_candidate`/`reject_candidate` cho `glossary_candidate`).",
        violations.len(),
        violations.join("\n")
    );
}

/// Đối chứng dương: `core::glossary` **có thật sự** mang CẢ HAI tên bảng — không có ca này
/// thì phép kiểm trên xanh y hệt trên một cây mà `candidate_store.rs` (hoặc toàn bộ
/// `core/glossary/`) đã bị xoá.
#[test]
fn core_glossary_actually_names_both_glossary_tables() {
    let (root, files) = all_rust_sources();

    for needle in FORBIDDEN_TABLES {
        let hits = files
            .iter()
            .filter(|f| rel_posix(&root, f).starts_with(GLOSSARY_DIR))
            .filter(|f| {
                fs::read_to_string(f)
                    .map(|t| t.contains(needle))
                    .unwrap_or(false)
            })
            .count();

        assert!(
            hits >= 1,
            "0 tệp dưới `{GLOSSARY_DIR}` nhắc tới `{needle}`. `store.rs`/`candidate_store.rs` \
             phải gõ tên bảng để đọc/ghi nó; 0 nghĩa là cây đã bị cắt và phép kiểm ranh giới \
             đang canh một chỗ trống."
        );
    }
}

/// Đối chứng dương thứ hai: `core::store::schema` cũng thật sự mang CẢ HAI tên — hai hằng
/// `GLOSSARY_ENTRY_DDL`/`GLOSSARY_CANDIDATE_DDL` sống ở đây, đúng như [`SCHEMA_FILE`] khai.
///
/// 🔵 **CẬP NHẬT 2026-08-20 (Story 3.2) — đổi tên từ
/// `schema_rs_actually_declares_the_glossary_entry_table`**, tên nêu ở §KEEP của Spec
/// Change Log lượt rà soát #1.
#[test]
fn schema_rs_actually_declares_both_glossary_tables() {
    let (root, files) = all_rust_sources();

    for needle in FORBIDDEN_TABLES {
        let hit = files.iter().any(|f| {
            rel_posix(&root, f) == SCHEMA_FILE
                && fs::read_to_string(f)
                    .map(|t| t.contains(needle))
                    .unwrap_or(false)
        });

        assert!(
            hit,
            "`{SCHEMA_FILE}` không nhắc tới `{needle}` — hằng DDL tương ứng (bước 4/12 cho \
             `glossary_entry`, bước 13 cho `glossary_candidate`) phải khai `CREATE TABLE \
             {needle}` ở đúng tệp này."
        );
    }
}
// ═════════════════════════════════════════════════════════════════════════════════
// Vế thứ hai của AD-36 — chỉ `entries_eligible_for_injection` gọi được từ module khác
// ═════════════════════════════════════════════════════════════════════════════════

/// 🔴 `insert_manual_entry`/`confirm_translation`/`load_tier` chỉ được GỌI (và được KHAI)
/// dưới `core/glossary/**`. Một module Epic 4 gọi `core::glossary::load_tier(global)` thẳng
/// để tự lọc "đã chốt" biên dịch sạch và qua cả mười một cổng hôm nay — cổng này là phép
/// kiểm DUY NHẤT bắt được nó.
///
/// ⚠️ Trước ca này, ba tên đó chỉ xuất hiện trong một câu thông báo `assert!` của
/// `only_glossary_and_schema_may_name_glossary_tables` — tức được NHẮC TỚI, không được
/// CANH.
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
            if let Some(needle) = line_calls_a_glossary_only_surface_function(&code) {
                violations.push(format!("{rel}:{line}  {needle}  |  {code}"));
            }
        }
    }

    assert!(
        violations.is_empty(),
        "{} chỗ ngoài `{GLOSSARY_DIR}` gọi thẳng một hàm phơi dữ liệu THÔ của Glossary:\n{}\n\n\
         `load_tier` trả CẢ mục chờ chốt; `insert_manual_entry`/`confirm_translation` ghi \
         thẳng không qua điều kiện chèn. Module khác chỉ được gọi \
         `core::glossary::entries_eligible_for_injection` — đúng MỘT hàm phơi ra, theo AD-36.",
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
            "không tệp nào dưới `{GLOSSARY_DIR}` khai `{needle}` — cây đã bị cắt và phép \
             kiểm ranh giới đang canh một chỗ trống."
        );
    }
}

// ═════════════════════════════════════════════════════════════════════════════════
// Story 3.2 — chỉ `core/glossary/**` được PHÉP GÕ token xuất xứ tự động
// ═════════════════════════════════════════════════════════════════════════════════

/// 🔴 `TermOrigin::ImportScan`/`TermOrigin::ReviewHarvest` chỉ được PHÉP xuất hiện dưới
/// `core/glossary/**`.
///
/// Trước Story 3.2, `insert_entry` cũ nhận `term_origin: TermOrigin` từ NƠI GỌI, nên một
/// module ở `core/segment/**` (Story 3.5, quét khi nhập) hay `core/review/**` (Epic 8, thu
/// hoạch) chỉ cần viết `TermOrigin::ImportScan` và gọi thẳng là ghi vào Glossary — biên
/// dịch sạch. Chữ ký mới của `insert_manual_entry` (không tham số `term_origin`) đóng phần
/// LỚN lỗ đó, nhưng không đóng việc một tệp khác GÕ được `TermOrigin::ImportScan`/
/// `TermOrigin::ReviewHarvest` (ví dụ để tự dựng một chuỗi `"import_scan"` bằng tay, lách
/// hẳn qua kiểu Rust). Cổng này đóng phần còn lại.
///
/// 🔵 **CẬP NHẬT 2026-08-20 (lượt rà soát ba lớp) — dùng vị từ DÙNG CHUNG
/// [`line_spells_a_non_manual_term_origin_token`], không tự lặp lại phép so ở đây.** Bản
/// trước tự viết vòng lặp so chuỗi trần TẠI CHỖ, và tách rời khỏi
/// [`NON_MANUAL_ORIGIN_TOKENS`] khiến bài tự kiểm không thể chứng minh ĐÚNG logic mà cổng
/// này chạy. Gọi chung một hàm xoá khả năng hai bên lệch nhau.
#[test]
fn only_core_glossary_may_spell_the_non_manual_term_origin_tokens() {
    let (root, files) = all_rust_sources();

    let mut violations: Vec<String> = Vec::new();

    for file in &files {
        let rel = rel_posix(&root, file);
        if rel.starts_with(GLOSSARY_DIR) {
            continue;
        }

        for (line, code) in code_lines(file) {
            if let Some(needle) = line_spells_a_non_manual_term_origin_token(&code) {
                violations.push(format!("{rel}:{line}  {needle}  |  {code}"));
            }
        }
    }

    assert!(
        violations.is_empty(),
        "{} chỗ ngoài `{GLOSSARY_DIR}` gõ token xuất xứ tự động:\n{}\n\n\
         `TermOrigin::ImportScan`/`TermOrigin::ReviewHarvest` chỉ được PHÉP sinh ra bên \
         trong `core::glossary::candidate::CandidateOrigin::to_term_origin` — chỗ DUY NHẤT \
         trong kho suy một `term_origin` khác `manual`, và nó suy TỪ dữ liệu đã có trên đĩa \
         (`candidate_origin` của chính hàng ứng viên), không nhận từ một tham số mà chỗ gọi \
         ngoài tự đặt (AD-20). Gọi `CandidateOrigin::ImportScan`/`CandidateOrigin::ReviewHarvest` \
         (ví dụ qua `insert_candidate`) là HỢP LỆ và KHÔNG bị cổng này bắt.",
        violations.len(),
        violations.join("\n")
    );
}

/// 🔴 TỰ KIỂM — chứng minh cổng ngay trên đỏ ĐƯỢC và không đỏ OAN, cả hai vế trên CHÍNH vị
/// từ [`line_spells_a_non_manual_term_origin_token`] mà cổng thật gọi.
///
/// 🔵 **THÊM 2026-08-20 (lượt rà soát ba lớp).** Trước lượt vá này, `NON_MANUAL_ORIGIN_TOKENS`
/// mang hai chuỗi TRẦN (`"ImportScan"`, `"ReviewHarvest"`) — và `CandidateOrigin`
/// (`candidate.rs`, Story 3.2) khai hai biến thể TRÙNG TÊN. Tái hiện được: một tệp ngoài
/// `core/glossary/**` gọi `insert_candidate(store, term, CandidateOrigin::ImportScan)` —
/// đúng hình dạng Story 3.5 CẦN GỌI để nạp một ứng viên vào bảng chờ — bị cổng bắt NHẦM
/// thành vi phạm AD-20. Vá bằng cách có định tính `TermOrigin::` (xem
/// [`NON_MANUAL_ORIGIN_TOKENS`]); ca này khoá cả hai chiều để lượt vá không âm thầm hỏng
/// theo hướng ngược lại (thu hẹp quá tay, không còn bắt được ca thật).
#[test]
fn the_non_manual_origin_token_check_catches_term_origin_but_not_candidate_origin() {
    assert_eq!(
        line_spells_a_non_manual_term_origin_token("let x = TermOrigin::ImportScan;"),
        Some("TermOrigin::ImportScan"),
        "ca DUONG THAT: TermOrigin::ImportScan phai bi bat -- day chinh la vi pham AD-20 \
         ma cong nay dung ra de chan"
    );
    assert_eq!(
        line_spells_a_non_manual_term_origin_token(
            "let entry = TermOrigin::ReviewHarvest;"
        ),
        Some("TermOrigin::ReviewHarvest"),
        "ca DUONG THAT thu hai: TermOrigin::ReviewHarvest phai bi bat"
    );
    assert_eq!(
        line_spells_a_non_manual_term_origin_token(
            "insert_candidate(store, term, CandidateOrigin::ImportScan)"
        ),
        None,
        "ca AM: goi insert_candidate voi CandidateOrigin::ImportScan la HINH DANG HOP LE \
         ma Story 3.5 can dung tu ngoai core/glossary/** -- KHONG duoc bi bat. Truoc luot va \
         2026-08-20, mot chuoi con tran \"ImportScan\" khop CA HAI ten kieu va lam cong nay \
         do OAN dung ca nay"
    );
    assert_eq!(
        line_spells_a_non_manual_term_origin_token(
            "let o = CandidateOrigin::ReviewHarvest;"
        ),
        None,
        "ca AM thu hai: CandidateOrigin::ReviewHarvest cung KHONG duoc bi bat, cung ly do \
         tren"
    );
}

/// Đối chứng dương: `core::glossary` **có thật sự** đánh vần CẢ HAI token — không có ca
/// này thì phép kiểm trên xanh y hệt trên một cây mà `entry.rs`/`candidate.rs` đã bị xoá
/// hết biến thể `ImportScan`/`ReviewHarvest`.
#[test]
fn core_glossary_actually_spells_both_non_manual_origin_tokens() {
    let (root, files) = all_rust_sources();

    for needle in NON_MANUAL_ORIGIN_TOKENS {
        let hits = files
            .iter()
            .filter(|f| rel_posix(&root, f).starts_with(GLOSSARY_DIR))
            .filter(|f| {
                fs::read_to_string(f)
                    .map(|t| t.contains(needle))
                    .unwrap_or(false)
            })
            .count();

        assert!(
            hits >= 1,
            "0 tệp dưới `{GLOSSARY_DIR}` nhắc tới `{needle}`. `entry.rs` khai biến thể đó \
             của `TermOrigin`, `candidate.rs` khai biến thể `CandidateOrigin` tương ứng — 0 \
             nghĩa là cây đã bị cắt và phép kiểm ranh giới đang canh một chỗ trống."
        );
    }
}

// ═════════════════════════════════════════════════════════════════════════════════
// Story 3.3 — `commands::glossary` gọi ĐÚNG bề mặt mới, không lách qua bề mặt cấm
// ═════════════════════════════════════════════════════════════════════════════════

/// Positive: `commands/glossary.rs` thực sự gọi cả BA hàm mới, và KHÔNG gọi bất kỳ tên nào
/// trong `GLOSSARY_ONLY_SURFACE`. Đối chứng dương của chính story này — không có ca này thì
/// cổng `only_entries_eligible_for_injection_may_be_called_from_outside_glossary` xanh y hệt
/// trên một `commands/glossary.rs` không gọi gì cả (một bề mặt IPC rỗng, vô dụng).
#[test]
fn commands_glossary_calls_the_new_quick_add_surface_not_the_forbidden_one() {
    let path = src_root().join("commands").join("glossary.rs");
    assert!(
        path.is_file(),
        "commands/glossary.rs phai ton tai o {}",
        path.display()
    );

    // ⚠️ `code_lines`, KHÔNG `fs::read_to_string(..).contains(..)` trần: chính doc-comment
    // đầu tệp của `commands/glossary.rs` NHẮC TỚI cả ba tên bị cấm (để giải thích LÝ DO
    // chúng bị cấm) — một phép so toàn văn bản sẽ tự bắt nhầm chính lời giải thích đó. Cùng
    // kỷ luật mà `only_glossary_and_schema_may_name_glossary_tables` đã dùng cho
    // `FORBIDDEN_TABLES` ở trên: chỉ MÃ mới đáng canh, không phải mọi dòng trong tệp.
    let mut code = String::new();
    for (_, line) in code_lines(&path) {
        code.push_str(&line);
        code.push('\n');
    }

    for needle in QUICK_ADD_SURFACE {
        assert!(
            code.contains(needle),
            "commands/glossary.rs khong goi `{needle}` (trong PHAN MA, khong tinh comment) \
             -- day la dung MOT trong ba ham ma Story 3.3 dung ra de thay the \
             GLOSSARY_ONLY_SURFACE cho be mat IPC nay"
        );
    }
    for needle in GLOSSARY_ONLY_SURFACE {
        assert!(
            !code.contains(needle),
            "commands/glossary.rs goi thang `{needle}` (trong PHAN MA) -- day la mot trong \
             ba ham bi cam ngoai core/glossary/**; dung ba ham moi \
             (resolve_term_for_quick_add / add_manual_term / update_manual_term) thay vi \
             lach qua day"
        );
    }
}

/// 🔴 Đối chứng dương thứ hai — chứng minh cổng THẬT (dùng cùng vị từ
/// [`line_calls_a_glossary_only_surface_function`]) vẫn đỏ nếu ai đó viết một lời gọi
/// `load_tier(...)` bên trong `commands/glossary.rs`: cổng không phải "chưa bao giờ đỏ",
/// nó chỉ đang đỏ đúng ZERO lần trên cây hiện tại vì cây hiện tại sạch.
#[test]
fn the_glossary_only_surface_check_would_still_flag_a_forbidden_call_from_commands_glossary() {
    assert_eq!(
        line_calls_a_glossary_only_surface_function("let rows = load_tier(&global)?;"),
        Some("load_tier"),
        "mot loi goi load_tier gia lap phai bi vi tu nay bat -- day chinh la hinh dang ma \
         cong that se do neu commands/glossary.rs lach qua GLOSSARY_ONLY_SURFACE"
    );
    assert_eq!(
        line_calls_a_glossary_only_surface_function("insert_manual_entry(store, term, ..)"),
        Some("insert_manual_entry"),
    );
    assert_eq!(
        line_calls_a_glossary_only_surface_function("confirm_translation(store, id, t)"),
        Some("confirm_translation"),
    );
    assert_eq!(
        line_calls_a_glossary_only_surface_function("resolve_term_for_quick_add(r, g, w, t)"),
        None,
        "ba ham MOI cua Story 3.3 khong nam trong GLOSSARY_ONLY_SURFACE -- do la diem cua \
         chung"
    );
    assert_eq!(
        line_calls_a_glossary_only_surface_function("marks_for_source_text(r, g, w, text, lang)"),
        None,
        "ham THU TU cua Story 3.4 cung khong nam trong GLOSSARY_ONLY_SURFACE -- cung diem \
         voi ba ham cua Story 3.3"
    );
}
