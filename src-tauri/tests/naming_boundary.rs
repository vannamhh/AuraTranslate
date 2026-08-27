//! Ranh giới ĐẶT TÊN của Story 5.1 — cưỡng chế `Work`/`Chapter` cho tầng Tác phẩm, cấm
//! `Project`/`Book`/`Novel`/`Document` cho cùng khái niệm đó — `AGENTS.md:41`.
//!
//! ⚠️ Tệp riêng có chủ ý, đúng khuôn `ai_boundary.rs`/`scope_boundary.rs`/
//! `matching_boundary.rs`/`glossary_boundary.rs`/`store_boundary.rs`/`segment_boundary.rs`:
//! đây là phép kiểm **tĩnh trên cây nguồn**. `project_contract.rs` nghiệm thu **hành vi lúc
//! chạy** (ba mệnh đề còn lại của Story 5.1: không thực thể tầng thứ ba, `source_lang` bất
//! biến, Glossary/TM phân giải ở `Tier::Work`) — trộn hai thứ là làm hỏng đúng thứ khiến cả
//! hai đọc được.
//!
//! ─────────────────────────────────────────────────────────────────────────────
//! 🔴 AGENTS.md:41 TỰ KHAI *"KHÔNG CỔNG NÀO CANH LUẬT NÀY"* — TỆP NÀY LÀ CỔNG ĐÓ
//! ─────────────────────────────────────────────────────────────────────────────
//! Mô hình hai tầng `Work → Chapter` đã tồn tại gần trọn vẹn trong mã trước cả story này
//! (bảng `work`/`chapter` trong `project.db`, `Tier::Work` của `ScopeResolver`). Thứ không
//! tồn tại là một PHÉP ĐO cho quy ước đặt tên — BA họ định danh THẬT (`ProjectError` ·
//! `ProjectNoWorkOpen` · `ProjectCreateFailed`) đặt tên cho khái niệm tầng Tác phẩm bằng
//! chính từ bị cấm, cộng MỘT chú thích (`ProjectMetaTooNew` — chưa từng là một biến thể khai
//! báo thật; nó chỉ sống trong một dòng ghi nhận sự VẮNG MẶT có chủ ý của chính nó, Ice chốt
//! 2026-08-06, xem `core/i18n/mod.rs`) mà tên cũng phải theo kịp đổi tên để khỏi nói dối. Không
//! gì ngăn một họ định danh MỚI mọc lên theo cùng cách. Story 5.1 đổi ba họ thật đó sang tiền
//! tố `Work` (xem `core/library/mod.rs`, `core/i18n/mod.rs`, `commands/project.rs`,
//! `commands/chapter.rs`) và dựng cổng này để một `pub struct DocumentMeta` hay một
//! `NovelStatus` tương lai không lặng lẽ lọt qua.
//!
//! ─────────────────────────────────────────────────────────────────────────────
//! 🔴 DANH SÁCH MIỄN TRỪ LẤY NGUYÊN VĂN TỪ `AGENTS.md:41` — KHÔNG TỰ NỚI THÊM
//! ─────────────────────────────────────────────────────────────────────────────
//! `AGENTS.md:41`: *"Cấm `Project`/`Book`/`Novel`/`Document` cho `Work` — `StoreKind::Project`
//! và `ProjectStore` đặt tên cho KHO `.atproj`, không cho thực thể."* Tám chuỗi dưới đây
//! ([`STORE_EXEMPT`]) là MỌI hình dạng mà chuỗi `project`/`Project` mang khi nó đặt tên cho
//! cái KHO (`.atproj`/`project.db`) hay cổng của kho đó (`StoreKind::Project` ·
//! `ProjectStore` · `PROJECT_MIGRATIONS`) hoặc chính TỆP thực thi những vai trò đó
//! (`commands/project.rs` · `ports/project_store.rs` · `tests/project_contract.rs`) —
//! không một mục nào đặt tên cho THỰC THỂ. Cổng dưới đây không được tự thêm một mục nào vào
//! danh sách này; một vi phạm bắt được NGOÀI bốn họ định danh đã đổi ở story này là quyết
//! định phạm vi của Ice, không phải một lượt vá tiện tay (xem Completion Notes của story).
//!
//! ─────────────────────────────────────────────────────────────────────────────
//! 🔴 API NỀN TẢNG KHÔNG PHẢI TÊN THỰC THỂ — VÀ VÌ SAO BA HÌNH DẠNG, KHÔNG HAI
//! ─────────────────────────────────────────────────────────────────────────────
//! `document.querySelector(...)` (DOM) và `app.path().document_dir()` (Tauri/OS) không phải
//! vi phạm: [`FORBIDDEN_WORDS`] so khớp **PHÂN BIỆT HOA/THƯỜNG** trên dạng viết hoa đầu chữ
//! (`Document`), và cả hai ví dụ trên đều viết thường (`document`) — chúng không khớp bằng
//! chính thiết kế của phép so, không cần một nhánh miễn trừ riêng. Ca
//! [`platform_apis_are_never_flagged`] chứng minh đúng điều đó bằng phép so **PHÂN BIỆT HOA
//! THƯỜNG**, khác `matching_boundary.rs::contains_forbidden_token` (so KHÔNG phân biệt —
//! đúng cho tên crate viết được nhiều kiểu, sai cho một quy ước đặt tên PascalCase).
//!
//! Hình dạng THỨ BA — `target: Document` (`src/panels/selectionContract.ts:275`) — viết hoa
//! và đứng một mình y hệt hình dạng của một vi phạm THẬT (`struct Book`). Đây là kiểu DOM
//! toàn cục (`lib.dom.d.ts`), không phải một thực thể ta đặt tên, và nó không tự loại bằng
//! phân biệt hoa/thường. [`is_platform_document_type_reference`] nhận diện đúng hình dạng
//! hẹp này — đứng SAU dấu `: ` (vị trí KIỂU trong TypeScript) VÀ không mang hậu tố PascalCase
//! nào theo sau (`DocumentStore` viết ở vị trí đó vẫn bị bắt, xem
//! [`the_platform_document_exemption_does_not_swallow_a_compound_entity_name`]) — hẹp đúng
//! bằng hình dạng thật, không rộng hơn.
//!
//! ─────────────────────────────────────────────────────────────────────────────
//! 🔴 HAI CÂY NGUỒN, HAI CÁCH BỎ COMMENT — VÌ SAO KHÔNG DÙNG CHUNG MỘT HÀM
//! ─────────────────────────────────────────────────────────────────────────────
//! Bốn tệp `*_boundary.rs` khác chỉ bỏ dòng bắt đầu bằng `//`, và điều đó ĐÚNG cho
//! `src-tauri/src/**`: kho không dùng khối `/* … */` một lần nào (đo 2026-08-27, 0 chỗ). Cây
//! `src/**` thì khác — riêng `GridPanel.vue` mang **886** dòng bắt đầu bằng ` * ` (khối
//! JSDoc). Một bộ lọc chỉ-`//` để lọt nguyên văn dòng `GridPanel.vue:578`
//! (*"… của `Document`, nên …"*) vào tập DÒNG MÃ và báo ĐỎ OAN trên một câu giải thích, không
//! một lời gọi. [`strip_ts_comments`] bỏ CẢ BA hình dạng (`//`, `/* … */` xuyên dòng,
//! `<!-- … -->` xuyên dòng — Vue template) cho cây `src/**`; `src-tauri/src/**` giữ bộ lọc
//! chỉ-`//` đã có tiền lệ.
//!
//! ─────────────────────────────────────────────────────────────────────────────
//! 🔴 SÀN SỐ TỆP LÀ BẮT BUỘC — "cây rỗng đọc thành sạch"
//! ─────────────────────────────────────────────────────────────────────────────
//! Bài học kế thừa từ mọi `*_boundary.rs` khác: một gốc quét sai hay một thư mục bị cắt làm
//! `walk` khớp 0 tệp, và khi đó MỌI phép kiểm dưới đây xanh mà không kiểm gì cả.

use std::fs;
use std::path::{Path, PathBuf};

/// Bốn từ cấm cho khái niệm tầng Tác phẩm — `AGENTS.md:41`. So khớp **PHÂN BIỆT HOA/THƯỜNG**
/// trên đúng dạng viết hoa đầu chữ này; xem doc-comment đầu tệp §API NỀN TẢNG.
const FORBIDDEN_WORDS: [&str; 4] = ["Project", "Book", "Novel", "Document"];

/// Miễn trừ NGUYÊN VĂN từ `AGENTS.md:41` — mọi chuỗi đặt tên cho KHO `.atproj`, không cho
/// thực thể `Work`. Xem doc-comment đầu tệp §DANH SÁCH MIỄN TRỪ.
const STORE_EXEMPT: [&str; 8] = [
    ".atproj",
    "project.db",
    "StoreKind::Project",
    "ProjectStore",
    "PROJECT_MIGRATIONS",
    "commands/project.rs",
    "ports/project_store.rs",
    "tests/project_contract.rs",
];

/// Số tệp `.rs` tối thiểu dưới `src-tauri/src/**` để phép quét là thật.
///
/// Số thật lúc dựng (2026-08-27, sau đổi tên của story này): **55**. Sàn **44** (80%),
/// cùng khuôn `RS_FLOOR`/`SRC_RS_FLOOR` của các tệp `*_boundary.rs` khác.
const RUST_FLOOR: usize = 44;

/// Số tệp `.ts`/`.vue` tối thiểu dưới `src/**` để phép quét là thật.
///
/// Số thật lúc dựng (2026-08-27): **73** (51 `.ts` + 22 `.vue`). Sàn **58** (~79%).
const FRONTEND_FLOOR: usize = 58;

fn is_word_byte(b: u8) -> bool {
    b.is_ascii_alphanumeric() || b == b'_'
}

fn rust_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src")
}

/// `CARGO_MANIFEST_DIR` là `src-tauri/` — thư mục `src/` của frontend là thư mục anh em,
/// một cấp lên rồi xuống `src`.
fn frontend_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap_or_else(|| panic!("CARGO_MANIFEST_DIR khong co thu muc cha"))
        .join("src")
}

/// Đường dẫn tương đối, dùng dấu `/` trên cả hai nền tảng — bắt buộc cho NFR14, cùng lý do
/// mọi tệp `*_boundary.rs` khác đã ghi: `starts_with`/nối chuỗi trên Windows so với `core\ai`
/// và không bao giờ khớp nếu không chuẩn hoá.
fn rel_posix(root: &Path, file: &Path) -> String {
    file.strip_prefix(root)
        .unwrap_or(file)
        .to_string_lossy()
        .replace('\\', "/")
}

fn walk(dir: &Path, exts: &[&str], out: &mut Vec<PathBuf>) {
    let entries = fs::read_dir(dir).unwrap_or_else(|e| panic!("đọc {}: {e}", dir.display()));
    for entry in entries {
        let entry = entry.unwrap_or_else(|e| panic!("duyệt {}: {e}", dir.display()));
        let path = entry.path();
        let meta =
            fs::symlink_metadata(&path).unwrap_or_else(|e| panic!("lstat {}: {e}", path.display()));

        // ⚠️ `symlink_metadata`, không `metadata`: một liên kết trỏ về thư mục cha làm đệ
        // quy không dừng — cùng bài học mọi tệp `*_boundary.rs` khác đã ghi.
        if meta.file_type().is_symlink() {
            continue;
        }
        if meta.is_dir() {
            walk(&path, exts, out);
        } else if path
            .extension()
            .and_then(|e| e.to_str())
            .is_some_and(|e| exts.contains(&e))
        {
            out.push(path);
        }
    }
}

/// Một tệp nguồn đã đọc: đường dẫn HIỂN THỊ (bắt đầu bằng `src-tauri/src/` hoặc `src/`, đúng
/// hình dạng Code Map của story dùng), nội dung ĐÃ BỎ COMMENT (để quét vi phạm theo dòng) và
/// nội dung NGUYÊN VĂN (để [`every_store_exemption_still_matches_something_real_in_the_tree`]
/// kiểm một chuỗi miễn trừ còn thật — kể cả khi nó chỉ còn sống trong một dòng chú thích, vd.
/// `commands/project.rs`/`ports/project_store.rs`/`tests/project_contract.rs`, cả ba chỉ xuất
/// hiện dưới dạng TÊN TỆP được nhắc trong doc-comment, không một lần nào ở vị trí mã).
struct ScannedFile {
    display: String,
    scan_text: String,
}

/// Mọi tệp `.rs` dưới `src-tauri/src/**`, bỏ dòng comment theo khuôn `//` đã có tiền lệ.
///
/// ⚠️ KHÔNG bỏ khối `/* … */` — cây này không dùng hình dạng đó (đo 2026-08-27: 0 chỗ, xem
/// doc-comment đầu tệp §HAI CÂY NGUỒN). Nếu số đo đó đổi, [`the_rust_tree_still_has_zero_block_comments`]
/// bên dưới sẽ đỏ trước, chứ không phải cổng chính đỏ oan.
fn rust_sources() -> Vec<ScannedFile> {
    let root = rust_root();
    let mut files = Vec::new();
    walk(&root, &["rs"], &mut files);
    files.sort();

    files
        .into_iter()
        .map(|file| {
            let rel = rel_posix(&root, &file);
            let text =
                fs::read_to_string(&file).unwrap_or_else(|e| panic!("đọc {}: {e}", file.display()));
            let scan_text: String = text
                .lines()
                .map(|line| {
                    if line.trim_start().starts_with("//") {
                        String::new()
                    } else {
                        line.to_owned()
                    }
                })
                .collect::<Vec<_>>()
                .join("\n");
            ScannedFile { display: format!("src-tauri/src/{rel}"), scan_text }
        })
        .collect()
}

/// Xoá (thay bằng khoảng trắng, GIỮ NGUYÊN vị trí byte và `\n`) mọi vùng comment
/// TypeScript/Vue: `//` tới hết dòng, `/* … */` xuyên nhiều dòng, `<!-- … -->` xuyên nhiều
/// dòng. Xem doc-comment đầu tệp §HAI CÂY NGUỒN cho lý do cây này cần nhiều hơn một bộ lọc
/// `//`.
///
/// ⚠️ Không phân biệt được comment khỏi một chuỗi ký tự chứa `//`/`/*` — cùng giới hạn mọi
/// tệp `*_boundary.rs` khác đã chấp nhận (so chuỗi con, không phải một parser). Không phải
/// vấn đề ở đây: bốn từ cấm là định danh PascalCase tiếng Anh, không phải nội dung chuỗi
/// hiển thị tiếng Việt của dự án.
///
/// An toàn UTF-8: mọi dấu mốc (`//`, `/*`, `*/`, `<!--`, `-->`) là ASCII thuần, và byte
/// tiếp diễn UTF-8 (0x80–0xBF) không bao giờ trùng giá trị của chúng — nên biên vùng bị xoá
/// không bao giờ rơi giữa một ký tự nhiều byte, và thay mỗi byte bị xoá bằng `b' '` (một byte
/// ASCII hợp lệ) không bao giờ tạo ra một chuỗi UTF-8 hỏng.
fn strip_ts_comments(text: &str) -> String {
    let bytes = text.as_bytes();
    let n = bytes.len();
    let mut out = bytes.to_vec();
    let mut i = 0;

    while i < n {
        if i + 1 < n && bytes[i] == b'/' && bytes[i + 1] == b'/' {
            while i < n && bytes[i] != b'\n' {
                out[i] = b' ';
                i += 1;
            }
        } else if i + 1 < n && bytes[i] == b'/' && bytes[i + 1] == b'*' {
            out[i] = b' ';
            out[i + 1] = b' ';
            i += 2;
            while i + 1 < n && !(bytes[i] == b'*' && bytes[i + 1] == b'/') {
                if bytes[i] != b'\n' {
                    out[i] = b' ';
                }
                i += 1;
            }
            if i + 1 < n {
                out[i] = b' ';
                out[i + 1] = b' ';
                i += 2;
            } else if i < n && bytes[i] != b'\n' {
                out[i] = b' ';
                i += 1;
            }
        } else if i + 3 < n && &bytes[i..i + 4] == b"<!--" {
            for k in 0..4 {
                out[i + k] = b' ';
            }
            i += 4;
            while i + 2 < n && &bytes[i..i + 3] != b"-->" {
                if bytes[i] != b'\n' {
                    out[i] = b' ';
                }
                i += 1;
            }
            if i + 2 < n {
                for k in 0..3 {
                    out[i + k] = b' ';
                }
                i += 3;
            } else {
                i = n;
            }
        } else {
            i += 1;
        }
    }

    String::from_utf8(out).unwrap_or_else(|_| text.to_owned())
}

/// Mọi tệp `.ts`/`.vue` dưới `src/**`, đã bỏ comment qua [`strip_ts_comments`].
fn frontend_sources() -> Vec<ScannedFile> {
    let root = frontend_root();
    let mut files = Vec::new();
    walk(&root, &["ts", "vue"], &mut files);
    files.sort();

    files
        .into_iter()
        .map(|file| {
            let rel = rel_posix(&root, &file);
            let text =
                fs::read_to_string(&file).unwrap_or_else(|e| panic!("đọc {}: {e}", file.display()));
            let scan_text = strip_ts_comments(&text);
            ScannedFile { display: format!("src/{rel}"), scan_text }
        })
        .collect()
}

/// Xoá mọi lần xuất hiện của các chuỗi [`STORE_EXEMPT`] khỏi `line` — thay bằng khoảng
/// trắng, GIỮ NGUYÊN vị trí byte, để một vi phạm THẬT đứng cạnh một chuỗi miễn trừ trên
/// cùng dòng vẫn bị bắt (không "nuốt" cả dòng).
///
/// 🔴 **VÁ (vòng rà đối kháng — P3)** — bản trước che theo CHUỖI CON, không neo biên: một
/// định danh MỚI thật sự vi phạm mà CHỨA một chuỗi miễn trừ làm tiền tố (vd. `ProjectStoreView`
/// chứa trọn `ProjectStore`) bị che mất đúng phần đó TRƯỚC khi [`FORBIDDEN_WORDS`] kịp soi —
/// đỏ thật thành xanh giả. Chỉ mask khi chuỗi miễn trừ khớp Ở BIÊN ĐỊNH DANH cả hai phía
/// (trước và sau): `StoreKind::Project` đứng trước dấu `;` (biên) thì bị che; giả định
/// `StoreKind::ProjectX` (biên SAU thất bại — `X` là word byte) thì KHÔNG bị che, và
/// `FORBIDDEN_WORDS` soi thấy `Project` đứng sau `::` (biên TRƯỚC hợp lệ) như bình thường.
fn mask_store_exemptions(line: &str) -> String {
    let bytes_ref = line.as_bytes();
    let mut out = bytes_ref.to_vec();
    for exempt in STORE_EXEMPT {
        let mut start = 0usize;
        while start <= line.len() {
            let Some(rel) = line.get(start..).and_then(|s| s.find(exempt)) else { break };
            let at = start + rel;
            let end = at + exempt.len();
            let before_ok = at == 0 || !is_word_byte(bytes_ref[at - 1]);
            let after_ok = end >= bytes_ref.len() || !is_word_byte(bytes_ref[end]);
            if before_ok && after_ok {
                for b in &mut out[at..end] {
                    *b = b' ';
                }
            }
            start = end;
        }
    }
    String::from_utf8(out).unwrap_or_else(|_| line.to_owned())
}

/// `code` mang `Document` tại byte-offset `at` như một tham chiếu KIỂU DOM toàn cục
/// (`: Document`, đúng hình dạng `src/panels/selectionContract.ts:275`) — API NỀN TẢNG,
/// không phải tên thực thể. Xem doc-comment đầu tệp §API NỀN TẢNG cho lý do hẹp đúng bằng
/// hình dạng thật: đòi CẢ hai — đứng sau dấu `:` (khoảng trắng tuỳ ý ở giữa) VÀ không mang
/// hậu tố PascalCase (`DocumentStore` ở đúng vị trí đó vẫn bị bắt, xem ca đối chứng bên dưới).
///
/// 🔴 **VÁ (vòng rà đối kháng — P10)** — bản trước khoá cứng ĐÚNG một byte pattern `": "`
/// (dấu hai chấm rồi ĐÚNG MỘT khoảng trắng). `target:Document` (Prettier/rustfmt tắt, không
/// khoảng trắng) hay `target:  Document` (hai khoảng trắng) báo ĐỎ OAN trên một tham chiếu
/// kiểu DOM hợp lệ. Nay lùi qua khoảng trắng/tab TUỲ Ý trước `Document` rồi mới đòi ký tự
/// liền trước đó là `:` — vế chặt (hậu tố PascalCase) giữ nguyên không đổi.
fn is_platform_document_type_reference(code: &str, at: usize) -> bool {
    let bytes = code.as_bytes();
    let end = at + "Document".len();

    let after_is_compound_suffix = bytes.get(end).copied().is_some_and(is_word_byte);
    if after_is_compound_suffix {
        return false;
    }

    let mut i = at;
    while i > 0 && matches!(bytes[i - 1], b' ' | b'\t') {
        i -= 1;
    }
    i > 0 && bytes[i - 1] == b':'
}

/// Vị từ THUẦN, dùng bởi CẢ cổng thật lẫn mọi ca đối chứng bên dưới — hai bên không thể lệch
/// nhau bằng cách trùng lặp phép so chuỗi ở hai chỗ khác nhau (đúng khuôn
/// `ai_boundary.rs::line_names_a_forbidden_ai_dependency`).
///
/// Trả về từ cấm khớp ĐẦU TIÊN, hoặc `None`. Khớp có NEO BIÊN TRƯỚC (`Project` trong
/// `ProjectError` bị bắt vì trước nó là biên định danh; `Project` trong `StoreKind::Project`
/// cũng đứng sau một biên — nó bị THA vì đã bị [`mask_store_exemptions`] xoá trước khi vị từ
/// này chạy, không phải vì thiếu neo). KHÔNG neo biên SAU — đó chính là điều kiện để bắt được
/// hình dạng tiền tố ghép (`NovelMeta`, `DocumentStore`), đúng I/O Matrix "Từ cấm còn lại".
fn line_names_a_forbidden_entity(code: &str) -> Option<&'static str> {
    let masked = mask_store_exemptions(code);
    let bytes = masked.as_bytes();

    for word in FORBIDDEN_WORDS {
        let mut start = 0usize;
        while start <= masked.len() {
            let Some(rel) = masked.get(start..).and_then(|s| s.find(word)) else { break };
            let at = start + rel;
            let before_ok = at == 0 || !is_word_byte(bytes[at - 1]);

            if before_ok {
                let is_platform =
                    word == "Document" && is_platform_document_type_reference(&masked, at);
                if !is_platform {
                    return Some(word);
                }
            }
            start = at + word.len();
        }
    }
    None
}

// ═════════════════════════════════════════════════════════════════════════════════
// 🔴 P2 (vòng rà đối kháng) — chuỗi `code` của `IpcError::new(` phải bị soi RIÊNG
// ═════════════════════════════════════════════════════════════════════════════════
// `code` (`core/i18n/mod.rs::IpcError`) là "định danh máy đọc" đi qua IPC, viết THƯỜNG có dấu
// chấm (`glossary.entry_missing`, `import.too_large`, …) — khác hẳn định danh Rust PascalCase
// mà [`FORBIDDEN_WORDS`]/[`line_names_a_forbidden_entity`] canh. `naming_boundary.rs` so PHÂN
// BIỆT HOA/THƯỜNG có chủ ý (xem doc-comment đầu tệp §API NỀN TẢNG), nên một chuỗi `code` viết
// thường mang từ cấm (`"project.create_failed"`) không bao giờ khớp `FORBIDDEN_WORDS` — cổng
// chính MÙ với đúng lỗi mà `MessageKey`/`vi.json` đã đổi tên nhưng `code` thì quên đổi. Đây là
// cổng THỨ HAI, dùng CHUNG [`mask_store_exemptions`] (để `project.db` vẫn hợp lệ trong một
// chuỗi `code` giả định) nhưng soi bằng vỏ chữ thường riêng.

/// Bốn từ cấm dạng CHỮ THƯỜNG — dùng riêng cho chuỗi `code`.
const FORBIDDEN_WORDS_LOWER: [&str; 4] = ["project", "book", "novel", "document"];

/// `code_string` (tham số ĐẦU của một lời gọi `IpcError::new(`) mang một từ cấm dạng thường —
/// cùng khuôn neo biên TRƯỚC và mask [`STORE_EXEMPT`] mà [`line_names_a_forbidden_entity`]
/// dùng, chỉ đổi bảng từ sang [`FORBIDDEN_WORDS_LOWER`].
fn ipc_error_code_names_a_forbidden_entity(code_string: &str) -> Option<&'static str> {
    let masked = mask_store_exemptions(code_string);
    let bytes = masked.as_bytes();

    for word in FORBIDDEN_WORDS_LOWER {
        let mut start = 0usize;
        while start <= masked.len() {
            let Some(rel) = masked.get(start..).and_then(|s| s.find(word)) else { break };
            let at = start + rel;
            let before_ok = at == 0 || !is_word_byte(bytes[at - 1]);
            if before_ok {
                return Some(word);
            }
            start = at + word.len();
        }
    }
    None
}

/// Trích chuỗi `code` (tham số ĐẦU) của mọi lời gọi `IpcError::new(` trong `text` — cuộc gọi
/// thường xuống dòng (khuôn `IpcError::new(\n    "…",\n    MessageKey::…, …)`, xem
/// `core/library/mod.rs`/`commands/chapter.rs`), nên vị từ này quét TỚI chuỗi literal `"..."`
/// kế tiếp thay vì đòi cùng dòng với `IpcError::new(`.
///
/// Trả về `(số dòng 1-based nơi "IpcError::new(" xuất hiện, chuỗi code)`.
///
/// ⚠️ **GIỚI HẠN, ghi ra thay vì để người sau tự phát hiện:** không xử escape (`\"`) bên
/// trong chuỗi `code` — mọi `code` thật trong kho hôm nay là một literal đơn giản kiểu
/// `"work.create_failed"`, không dấu nháy kép lồng. Không phân biệt được `IpcError::new(` là
/// một LỜI GỌI THẬT hay một chuỗi/comment nhắc tới chuỗi đó — cùng giới hạn so-chuỗi-con mà
/// mọi vị từ khác trong tệp này đã chấp nhận (xem doc-comment đầu tệp §HAI CÂY NGUỒN).
fn ipc_error_new_code_arguments(text: &str) -> Vec<(usize, String)> {
    const NEEDLE: &str = "IpcError::new(";
    let mut out = Vec::new();
    let mut search_from = 0usize;

    while let Some(rel) = text.get(search_from..).and_then(|s| s.find(NEEDLE)) {
        let call_at = search_from + rel;
        let line_no = text[..call_at].matches('\n').count() + 1;
        let after = call_at + NEEDLE.len();
        if let Some(open_rel) = text.get(after..).and_then(|s| s.find('"')) {
            let open = after + open_rel + 1;
            if let Some(close_rel) = text.get(open..).and_then(|s| s.find('"')) {
                out.push((line_no, text[open..open + close_rel].to_owned()));
            }
        }
        search_from = after;
    }

    out
}

/// Vi phạm từ chuỗi `code` của mọi lời gọi `IpcError::new(` trong MỘT tệp — cùng hình dạng
/// báo lỗi `"{display}:{line}  {word}  |  …"` mà [`violations_in_file`] dùng, để hai loại vi
/// phạm đọc giống nhau trong một danh sách gộp.
fn ipc_error_code_violations_in_file(file: &ScannedFile) -> Vec<String> {
    ipc_error_new_code_arguments(&file.scan_text)
        .into_iter()
        .filter_map(|(line_no, code_string)| {
            let word = ipc_error_code_names_a_forbidden_entity(&code_string)?;
            Some(format!("{}:{}  {word}  |  IpcError::new({code_string:?}, …)", file.display, line_no))
        })
        .collect()
}

// ═════════════════════════════════════════════════════════════════════════════════
// Sàn quần thể — chạy TRƯỚC mọi phép kiểm khác. Xem doc-comment đầu tệp.
// ═════════════════════════════════════════════════════════════════════════════════

#[test]
fn the_scanned_trees_are_both_large_enough_to_be_real() {
    let rust_files = rust_sources();
    assert!(
        rust_files.len() >= RUST_FLOOR,
        "chỉ tìm thấy {} tệp `.rs` dưới `src-tauri/src/**` (sàn {RUST_FLOOR}). Cây quá nhỏ để \
         là thật — một danh sách rỗng làm mọi phép kiểm dưới đây xanh mà không kiểm gì cả.",
        rust_files.len()
    );

    let frontend_files = frontend_sources();
    assert!(
        frontend_files.len() >= FRONTEND_FLOOR,
        "chỉ tìm thấy {} tệp `.ts`/`.vue` dưới `src/**` (sàn {FRONTEND_FLOOR}). Cây quá nhỏ để \
         là thật.",
        frontend_files.len()
    );
}

/// Xem doc-comment của [`rust_sources`] — mệnh đề "cây Rust không dùng `/* … */`" là điều
/// kiện để bộ lọc chỉ-`//` của cây đó AN TOÀN. Ca này canh mệnh đề đó tự đỏ trước khi cổng
/// chính lặng lẽ bỏ sót một khối comment thật.
///
/// ⚠️ Chỉ nhìn `scan_text` (đã xoá trắng dòng `//`/`///`/`//!`), KHÔNG nhìn nội dung tệp
/// nguyên văn — đo 2026-08-27: nội dung nguyên văn của 35 tệp chứa chuỗi con `"/*"`, và TOÀN
/// BỘ là dương giả — glob kiểu `src/**/*.rs`/`core/store/**` bên trong một dòng `///` (bản
/// thân dòng đó đã bị bộ lọc `//` loại), cộng một chỗ `*mới*/*giống*` (nhấn mạnh kiểu
/// Markdown) cũng nằm trong một dòng `//!`. Cả hai hình dạng dương giả đó đều có `*` hoặc
/// `.` NGAY SAU `/*` — một khối comment C thật gần như luôn có khoảng trắng hoặc `!`/`*`
/// (biến thể doc) ngay sau, nên loại hai hình dạng đó là đủ hẹp mà vẫn không bỏ sót.
#[test]
fn the_rust_tree_still_has_zero_block_comments() {
    let hits: Vec<String> = rust_sources()
        .into_iter()
        .filter(|f| {
            let bytes = f.scan_text.as_bytes();
            bytes.windows(2).enumerate().any(|(i, w)| {
                w == b"/*" && bytes.get(i + 2).is_some_and(|&b| b != b'*' && b != b'.')
            })
        })
        .map(|f| f.display)
        .collect();

    assert!(
        hits.is_empty(),
        "cây `src-tauri/src/**` nay có khối comment `/* … */` THẬT (ngoài vị trí `//`) ở {} \
         tệp: {:?}. Bộ lọc chỉ-`//` của `rust_sources()` không bỏ được hình dạng này — hoặc \
         thêm xử lý `/* … */` (đúng khuôn `strip_ts_comments`) hoặc xác nhận đây không phải \
         một vùng văn bản sẽ làm cổng chính đỏ oan.",
        hits.len(),
        hits
    );
}

// ═════════════════════════════════════════════════════════════════════════════════
// 🔴 Đối chứng dương/âm — I/O Matrix của story, gọi thẳng vị từ trên chuỗi dựng tay
// ═════════════════════════════════════════════════════════════════════════════════

/// I/O Matrix hàng "Vi phạm thật".
#[test]
fn a_real_violation_string_is_caught() {
    assert_eq!(
        line_names_a_forbidden_entity("pub enum ProjectError {"),
        Some("Project"),
        "ca DUONG THAT: `pub enum ProjectError {{` phai bi vi tu bat"
    );
}

/// I/O Matrix hàng "Miễn trừ có tên".
#[test]
fn named_store_exemptions_are_not_caught() {
    assert_eq!(
        line_names_a_forbidden_entity("    use crate::ports::project_store::StoreKind::Project;"),
        None,
        "ca AM: `StoreKind::Project` dat ten cho KHO, khong phai thuc the -- khong duoc bat oan"
    );
    assert_eq!(
        line_names_a_forbidden_entity(r#"    let db_path = dir.join("project.db");"#),
        None,
        "ca AM: `project.db` la ten TEP cua KHO -- khong duoc bat oan"
    );
    assert_eq!(
        line_names_a_forbidden_entity("pub trait ProjectStore {"),
        None,
        "ca AM: `ProjectStore` dat ten cho cong cua KHO -- khong duoc bat oan"
    );
}

/// 🔴 **Vòng rà đối kháng — P3.** Một định danh MỚI thật sự vi phạm mà CHỨA một chuỗi miễn
/// trừ làm tiền tố (`ProjectStoreView` chứa trọn `ProjectStore`) vẫn phải bị bắt — che theo
/// chuỗi con không neo biên sẽ làm phần `Project` biến mất trước khi vị từ kịp soi.
#[test]
fn a_token_that_merely_contains_an_exempt_string_as_a_prefix_is_still_caught() {
    assert_eq!(
        line_names_a_forbidden_entity("pub struct ProjectStoreView {"),
        Some("Project"),
        "ca DUONG THAT: `ProjectStoreView` la mot thuc the MOI chua tron `ProjectStore` lam \
         tien to -- mask khong neo bien se xoa mat `Project` truoc khi vi tu kip soi"
    );
    // Đối chứng ÂM giữ nguyên: chính `ProjectStore` (không hậu tố) vẫn được miễn trừ.
    assert_eq!(
        line_names_a_forbidden_entity("pub trait ProjectStore {"),
        None,
        "ca AM: `ProjectStore` (khong hau to) van la mien tru dat ten cho KHO"
    );
}

/// I/O Matrix hàng "API nền tảng" — cộng hình dạng thứ ba (`target: Document`) mà cây
/// `src/**` hôm nay thật sự có. Xem doc-comment đầu tệp §API NỀN TẢNG.
#[test]
fn platform_apis_are_never_flagged() {
    assert_eq!(
        line_names_a_forbidden_entity("  document.querySelector('.cell')"),
        None,
        "ca AM: `document.querySelector` la DOM toan cuc viet THUONG -- so phan biet hoa/thuong \
         phai loai no ma khong can mot nhanh mien tru rieng"
    );
    assert_eq!(
        line_names_a_forbidden_entity("    let documents = app.path().document_dir()?;"),
        None,
        "ca AM: `document_dir()` la ham OS (Tauri) viet THUONG -- cung ly do tren"
    );
    assert_eq!(
        line_names_a_forbidden_entity(
            "export function attachSelectionWatcher(target: Document, cb: () => void) {"
        ),
        None,
        "ca AM: `target: Document` la kieu DOM toan cuc (lib.dom.d.ts) o vi tri KIEU cua \
         TypeScript, khong phai mot thuc the ta dat ten -- xem is_platform_document_type_reference"
    );
}

/// 🔴 **Vòng rà đối kháng — P10.** `is_platform_document_type_reference` không được khoá
/// cứng đúng một byte pattern `": "` — khoảng trắng quanh dấu `:` là tuỳ ý về mặt cú pháp
/// TypeScript.
#[test]
fn the_platform_document_exemption_accepts_any_amount_of_space_around_the_colon() {
    assert_eq!(
        line_names_a_forbidden_entity("function f(target:Document): void {"),
        None,
        "ca AM: `target:Document` (KHONG khoang trang) van la tham chieu kieu DOM hop le"
    );
    assert_eq!(
        line_names_a_forbidden_entity("function f(target:  Document): void {"),
        None,
        "ca AM: `target:  Document` (HAI khoang trang) van la tham chieu kieu DOM hop le"
    );
    // Vế chặt giữ nguyên: hậu tố PascalCase vẫn bị bắt dù không khoảng trắng.
    assert_eq!(
        line_names_a_forbidden_entity("function f(target:DocumentStore): void {"),
        Some("Document"),
        "DocumentStore o vi tri KIEU (khong khoang trang) van la mot thuc the GHEP TEN, \
         khong duoc tha oan"
    );
}

/// I/O Matrix hàng "Từ cấm còn lại" — cả ba phải bị bắt.
#[test]
fn remaining_forbidden_words_are_all_caught() {
    assert_eq!(line_names_a_forbidden_entity("struct Book {"), Some("Book"));
    assert_eq!(line_names_a_forbidden_entity("  novel: NovelMeta,"), Some("Novel"));
    assert_eq!(
        line_names_a_forbidden_entity("pub struct DocumentStore {"),
        Some("Document")
    );
}

/// Đúng AC của story: "một người sau này thêm `pub struct DocumentMeta`" — vị từ bắt được,
/// và khi dòng này nằm trong một tệp thật, `scan_all_violations` nêu đúng `file:line` (định
/// dạng `"{display}:{line}  {word}  |  {code}"`, xem [`the_real_source_tree_has_zero_naming_violations`]).
#[test]
fn a_future_document_meta_struct_would_be_caught() {
    assert_eq!(
        line_names_a_forbidden_entity("pub struct DocumentMeta {"),
        Some("Document"),
        "AC cua story: mot DocumentMeta tuong lai phai bi cong bat"
    );
}

/// 🔴 **Vòng rà đối kháng — P2, đối chứng dương.** Chuỗi `code` của một `IpcError::new(`
/// mang từ cấm dạng thường phải bị bắt — độc lập với cây nguồn hôm nay có gì. Cuộc gọi xuống
/// dòng, đúng khuôn thật của `core/library/mod.rs`/`commands/chapter.rs`.
#[test]
fn the_ipc_error_code_check_would_actually_flag_a_seeded_violation() {
    let seeded = "IpcError::new(\n    \"project.foo\",\n    MessageKey::Unknown,\n    \
                  BTreeMap::new(),\n    false,\n)";

    let args = ipc_error_new_code_arguments(seeded);
    assert_eq!(args.len(), 1, "phai trich duoc dung MOT chuoi code tu loi goi gieo tay: {args:?}");
    assert_eq!(
        ipc_error_code_names_a_forbidden_entity(&args[0].1),
        Some("project"),
        "ca DUONG THAT: `IpcError::new(\"project.foo\"` phai bi vi tu bat"
    );
}

/// 🔴 **Vòng rà đối kháng — P2, đối chứng âm.** Khoá ĐÃ ĐỔI TÊN (`work.none_open`) và một
/// mục miễn trừ KHO (`project.db`) không được bắt oan trong vị trí `code`.
#[test]
fn the_ipc_error_code_check_does_not_flag_the_renamed_key_or_a_store_filename() {
    assert_eq!(
        ipc_error_code_names_a_forbidden_entity("work.none_open"),
        None,
        "ca AM: `work.none_open` la khoa DA DOI TEN, khong duoc bat oan"
    );
    assert_eq!(
        ipc_error_code_names_a_forbidden_entity("project.db"),
        None,
        "ca AM: `project.db` dat ten cho KHO qua STORE_EXEMPT, khong duoc bat oan trong vi tri code"
    );
}

/// 🔴 Miễn trừ DOM hẹp đúng bằng hình dạng thật — KHÔNG rộng tới mức nuốt một thực thể ghép
/// tên bằng `Document` đứng ở CÙNG vị trí cú pháp. Không có ca này, một `foo: DocumentStore`
/// viết trong một chữ ký hàm sẽ lọt qua như một tham chiếu DOM.
#[test]
fn the_platform_document_exemption_does_not_swallow_a_compound_entity_name() {
    assert_eq!(
        line_names_a_forbidden_entity("function f(target: DocumentStore): void {"),
        Some("Document"),
        "DocumentStore o vi tri KIEU van la mot thuc the GHEP TEN bang tu cam -- khong duoc \
         tha oan chi vi no dung sau `: `"
    );
}

// ═════════════════════════════════════════════════════════════════════════════════
// 🔴 Cổng THẬT — quét toàn bộ `src-tauri/src/**` + `src/**`
// ═════════════════════════════════════════════════════════════════════════════════

/// Mọi vi phạm trong MỘT tệp — tách khỏi [`scan_all_violations`] để
/// [`the_store_kind_enum_declaration_site_of_project_is_recognized_as_the_same_named_exemption`]
/// gọi được trực tiếp trên nội dung dựng tay.
///
/// 🔴 **NGOẠI LỆ HẸP, KHÔNG PHẢI MỘT MIỄN TRỪ MỚI** — điểm KHAI BÁO biến thể `Project` của
/// `enum StoreKind` (`core/store/mod.rs`, hôm nay dòng 171: chỉ `    Project,`, không mang
/// tiền tố) là chính thực thể mà `STORE_EXEMPT` gọi tên bằng chuỗi `"StoreKind::Project"` —
/// Rust không lặp lại tên enum tại điểm khai một biến thể, nên chuỗi có tiền tố đó không bao
/// giờ khớp tại ĐÚNG điểm sinh ra nó. Đây KHÔNG tự nới danh sách [`STORE_EXEMPT`]: không
/// thêm chuỗi nào vào đó. Đây là áp CÙNG một miễn trừ đã đặt tên vào điểm khai báo thay vì chỉ
/// điểm gọi, và phạm vi bị khoá CHẶT vào đúng thân `enum StoreKind { … }` — một biến thể
/// `Project` bên trong một enum KHÁC (vd. `enum WorkTierChoice { Project, … }`) vẫn bị bắt,
/// xem [`a_bare_project_variant_outside_the_store_kind_enum_is_still_caught`].
fn violations_in_file(file: &ScannedFile) -> Vec<String> {
    let mut violations = Vec::new();
    let mut inside_store_kind_enum = false;

    for (line_no, code) in file.scan_text.lines().enumerate() {
        let trimmed = code.trim();

        // 🔴 VÁ (vòng rà đối kháng — P4): `trimmed.contains("enum StoreKind")` khớp CẢ
        // `enum StoreKindLegacy {` (hay bất kỳ tên nào chứa "enum StoreKind" làm tiền tố) —
        // miễn trừ oan cho biến thể `Project` thật của một enum KHÁC. Neo bằng `ends_with`
        // trên đúng đuôi `"enum StoreKind {"`: không còn khớp một tên dài hơn, vì phần đuôi
        // của dòng đó sẽ là `"enum StoreKindLegacy {"`, không phải chuỗi này.
        if trimmed.ends_with("enum StoreKind {") {
            inside_store_kind_enum = true;
        }

        if let Some(word) = line_names_a_forbidden_entity(code) {
            let is_store_kind_project_declaration =
                word == "Project" && inside_store_kind_enum && trimmed == "Project,";
            if !is_store_kind_project_declaration {
                violations.push(format!("{}:{}  {word}  |  {trimmed}", file.display, line_no + 1));
            }
        }

        if inside_store_kind_enum && trimmed == "}" {
            inside_store_kind_enum = false;
        }
    }

    violations
}

fn scan_all_violations() -> Vec<String> {
    let rust = rust_sources();

    // `ipc_error_code_violations_in_file` chỉ áp cho cây Rust — `IpcError::new(` là một hình
    // dạng Rust, `src/**` không bao giờ mang chuỗi đó.
    let ipc_code_violations: Vec<String> =
        rust.iter().flat_map(ipc_error_code_violations_in_file).collect();

    rust.iter()
        .flat_map(violations_in_file)
        .chain(ipc_code_violations)
        .chain(frontend_sources().iter().flat_map(violations_in_file))
        .collect()
}

/// 🔴 AC chính của story: cây nguồn sau khi đổi tên có 0 vi phạm.
#[test]
fn the_real_source_tree_has_zero_naming_violations() {
    let violations = scan_all_violations();

    assert!(
        violations.is_empty(),
        "{} chỗ dùng `Project`/`Book`/`Novel`/`Document` để đặt tên cho khái niệm tầng Tác \
         phẩm:\n{}\n\n\
         AGENTS.md:41: thuật ngữ cố định là `Work`/`Chapter`. Nếu đây là một vi phạm THẬT — \
         DỪNG LẠI nếu nó không thuộc ba họ định danh thật mà Story 5.1 đã đổi \
         (`ProjectError`/`ProjectNoWorkOpen`/`ProjectCreateFailed`) hay chú thích đồng đổi tên \
         cùng lượt (`ProjectMetaTooNew`): đây là quyết định phạm vi của Ice, không phải một \
         lượt vá tiện tay.",
        violations.len(),
        violations.join("\n")
    );
}

/// Đúng doc-comment của [`violations_in_file`]: điểm khai báo KHÔNG mang tiền tố của chính
/// `StoreKind::Project` không bị bắt oan — nó là cùng một miễn trừ đã đặt tên, áp cho điểm
/// khai thay vì điểm gọi.
#[test]
fn the_store_kind_enum_declaration_site_of_project_is_recognized_as_the_same_named_exemption() {
    let file = ScannedFile {
        display: "src-tauri/src/core/store/mod.rs".to_owned(),
        scan_text: "pub enum StoreKind {\n    Global,\n    Project,\n    LibraryIndex,\n    Dict,\n}"
            .to_owned(),
    };

    assert!(
        violations_in_file(&file).is_empty(),
        "diem khai bao (khong tien to `StoreKind::`) cua bien the da duoc AGENTS.md:41 mien tru \
         (`StoreKind::Project`) khong duoc bi bat oan: {:?}",
        violations_in_file(&file)
    );
}

/// Đối chứng ÂM của ngoại lệ trên: một biến thể `Project` bên trong một enum KHÁC —
/// `StoreKind` không phải "mọi enum nào cũng được miễn trừ biến thể `Project`" — vẫn phải bị
/// bắt. Không có ca này, ngoại lệ ở trên rộng hơn hình dạng thật của nó cho phép.
#[test]
fn a_bare_project_variant_outside_the_store_kind_enum_is_still_caught() {
    let file = ScannedFile {
        display: "src-tauri/src/some_other_module.rs".to_owned(),
        scan_text: "pub enum WorkTierChoice {\n    Project,\n    Something,\n}".to_owned(),
    };

    let violations = violations_in_file(&file);
    assert_eq!(
        violations.len(),
        1,
        "mot bien the `Project` NGOAI than `enum StoreKind` van phai bi bat: {violations:?}"
    );
}

/// 🔴 **Vòng rà đối kháng — P4.** `enum StoreKindLegacy` (hay bất kỳ tên nào CHỨA
/// `"enum StoreKind"` làm tiền tố) không phải `enum StoreKind` — biến thể `Project` của nó
/// vẫn phải bị bắt, không được miễn trừ oan bằng một phép so `contains`.
#[test]
fn an_enum_whose_name_merely_contains_store_kind_as_a_prefix_does_not_borrow_its_exemption() {
    let file = ScannedFile {
        display: "src-tauri/src/some_other_module.rs".to_owned(),
        scan_text: "pub enum StoreKindLegacy {\n    Project,\n    Something,\n}".to_owned(),
    };

    let violations = violations_in_file(&file);
    assert_eq!(
        violations.len(),
        1,
        "`enum StoreKindLegacy` KHONG phai `enum StoreKind` -- bien the `Project` cua no van \
         phai bi bat, khong duoc mien tru oan qua mot phep so `contains`: {violations:?}"
    );
}

/// Miễn trừ phải khớp thứ gì đó THẬT trong kho — cùng đối chứng mà mọi tệp `*_boundary.rs`
/// khác đòi: một chuỗi trong [`STORE_EXEMPT`] gõ sai (đã đổi tên/xoá) làm nhánh miễn trừ
/// không bao giờ chạy, và phép kiểm chính ở trên vẫn xanh HÔM NAY (không ai đụng nó) rồi đỏ
/// sai chỗ vào ngày một vi phạm thật xuất hiện y hệt hình dạng cái miễn trừ đã chết.
///
/// ⚠️ Phạm vi tìm ở đây RỘNG HƠN cây bị cổng chính quét (`src-tauri/src` + `src`) — thêm
/// `src-tauri/tests/**` (trừ CHÍNH tệp này, nếu không phép kiểm tự thoả mãn vô nghĩa bằng
/// chính hằng [`STORE_EXEMPT`] nó khai) cho NĂM miễn trừ CHUỖI. Cây bị cổng chính quét không
/// có lý do nghiệp vụ nào tự nhắc tên tệp `tests/**` của chính nó, và mọi tệp `*_boundary.rs`
/// khác cố ý loại `tests/**` khỏi phạm vi quét (`store_boundary.rs` đầu tệp) nên mở rộng
/// phạm vi CỔNG CHÍNH để thoả ca này là sai hướng. Đây CHỈ là một sanity-check cho danh sách
/// miễn trừ, không phải cổng.
///
/// ⚠️ Ba miễn trừ ĐƯỜNG DẪN TỆP (`commands/project.rs`/`ports/project_store.rs`/
/// `tests/project_contract.rs`) được kiểm bằng SỰ TỒN TẠI CỦA CHÍNH TỆP ĐÓ, không phải bằng
/// việc một tệp KHÁC nhắc tới nó bằng chuỗi — đo 2026-08-27: `ports/project_store.rs` không
/// được bất kỳ tệp `.rs`/`.ts`/`.vue` nào khác trong kho nhắc tới bằng chuỗi (chỉ các hồ sơ
/// story ở `_bmad-output/implementation-artifacts/**` nhắc, và đó không phải cây nguồn). Bản
/// thân tệp vẫn tồn tại và vẫn đặt tên cho một cổng của KHO (`pub trait ProjectStore` —
/// xem [`named_store_exemptions_are_not_caught`]) — tồn tại-trên-đĩa là bằng chứng đúng cho
/// một miễn trừ ĐƯỜNG DẪN, khác bằng chứng cho một miễn trừ CHUỖI xuất hiện trong mã.
#[test]
fn every_store_exemption_still_matches_something_real_in_the_repo() {
    let file_path_exemptions: [&str; 3] =
        ["commands/project.rs", "ports/project_store.rs", "tests/project_contract.rs"];
    let string_exemptions: [&str; 5] =
        [".atproj", "project.db", "StoreKind::Project", "ProjectStore", "PROJECT_MIGRATIONS"];

    for rel in file_path_exemptions {
        let candidate = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(if rel.starts_with("tests/")
        {
            rel.to_owned()
        } else {
            format!("src/{rel}")
        });
        assert!(
            candidate.is_file(),
            "miễn trừ đường dẫn {rel:?} không còn khớp một tệp thật ({}). Nó đã lệch khỏi \
             thực tế (đổi tên, xoá tệp, …) — sửa danh sách miễn trừ trước khi tin cổng chính \
             là kín.",
            candidate.display()
        );
    }

    let mut files = Vec::new();
    walk(&rust_root(), &["rs"], &mut files);
    walk(&PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests"), &["rs"], &mut files);
    walk(&frontend_root(), &["ts", "vue"], &mut files);

    let all_text: String = files
        .into_iter()
        .filter(|f| f.file_name().and_then(|n| n.to_str()) != Some("naming_boundary.rs"))
        .map(|f| fs::read_to_string(&f).unwrap_or_default())
        .collect::<Vec<_>>()
        .join("\n");

    for exempt in string_exemptions {
        assert!(
            all_text.contains(exempt),
            "miễn trừ {exempt:?} không khớp bất kỳ đâu trong kho. Nó đã lệch khỏi thực tế \
             (đổi tên, xoá tệp, …) — sửa danh sách miễn trừ trước khi tin cổng chính là kín."
        );
    }
}

/// Cụm đánh dấu đầu danh sách miễn trừ trong dòng luật của `AGENTS.md`.
const EXEMPTION_CLAUSE_MARKER: &str = "Miễn trừ đủ tám mục";

/// Mọi đoạn giữa hai dấu backtick trong `clause` — dùng để trích danh sách mà một CÂU của
/// `AGENTS.md` thực sự liệt kê, tách khỏi thân test để
/// [`the_exemption_clause_parser_would_actually_flag_a_seeded_ninth_item`] gọi được trên một
/// chuỗi dựng tay, độc lập với nội dung thật của `AGENTS.md` hôm nay.
fn quoted_items(clause: &str) -> Vec<&str> {
    clause.split('`').enumerate().filter_map(|(i, seg)| (i % 2 == 1).then_some(seg)).collect()
}

/// Cắt `rule_line` xuống đúng MỆNH ĐỀ liệt kê miễn trừ — từ [`EXEMPTION_CLAUSE_MARKER`] tới
/// dấu `". "` kế tiếp (ranh giới câu) — rồi trả `quoted_items` của đúng mệnh đề đó. Tách khỏi
/// `rule_line` đầy đủ vì dòng luật còn quote nhiều thứ KHÁC không thuộc danh sách miễn trừ
/// (`Work`, `Chapter`, bốn từ cấm, `STORE_EXEMPT`, tên chính tệp cổng, …) — so nguyên dòng sẽ
/// luôn lệch.
fn exemption_items_named_in_rule_line(rule_line: &str) -> Vec<&str> {
    let Some(start) = rule_line.find(EXEMPTION_CLAUSE_MARKER) else {
        panic!(
            "dong luat khong con cum \"{EXEMPTION_CLAUSE_MARKER}\" -- khong biet doan nao la \
             danh sach mien tru"
        )
    };
    let after_marker = &rule_line[start..];
    let clause_end = after_marker.find(". ").unwrap_or(after_marker.len());
    quoted_items(&after_marker[..clause_end])
}

/// `AGENTS.md` là NGUỒN của quy ước; [`STORE_EXEMPT`] chỉ là bản thi hành của nó. Ca này đối
/// chiếu HAI CHIỀU: mọi mục của `STORE_EXEMPT` phải được dòng luật nêu tên, VÀ mọi mục dòng
/// luật nêu tên phải có mặt trong `STORE_EXEMPT`, cộng đúng SỐ LƯỢNG — không chỉ một chiều.
///
/// 🔴 **VÁ (vòng rà đối kháng — P9)** — bản trước chỉ kiểm MỘT chiều (mọi mục `STORE_EXEMPT`
/// có mặt trong dòng luật) cộng một phép ĐẾM backtick lỏng lẻo (`listed >= STORE_EXEMPT.len()`)
/// mà không đối chiếu TỪNG MỤC. Một mục THỨ CHÍN thêm vào dòng luật (không có trong
/// `STORE_EXEMPT`) làm `listed` tăng lên 9, phép `>=` vẫn đúng, và mục lạ đó lọt qua hoàn
/// toàn — đúng khi cả tên ca lẫn `AGENTS.md` đều hứa "khớp nhau từng mục".
#[test]
fn the_written_rule_and_the_enforced_exemption_list_name_the_same_eight_things() {
    let agents = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap_or_else(|| panic!("CARGO_MANIFEST_DIR khong co thu muc cha"))
        .join("AGENTS.md");
    let text = fs::read_to_string(&agents)
        .unwrap_or_else(|e| panic!("khong doc duoc {}: {e}", agents.display()));

    let rule_line = text
        .lines()
        .find(|l| l.contains(EXEMPTION_CLAUSE_MARKER))
        .unwrap_or_else(|| {
            panic!("AGENTS.md khong con dong luat dat ten nao nhac danh sach mien tru")
        });

    let quoted = exemption_items_named_in_rule_line(rule_line);

    for item in STORE_EXEMPT {
        assert!(
            quoted.contains(&item),
            "STORE_EXEMPT mien tru `{item}` nhung dong luat trong AGENTS.md khong neu no — \
             cong dang rong hon luat da viet"
        );
    }

    for item in &quoted {
        assert!(
            STORE_EXEMPT.contains(item),
            "dong luat trong AGENTS.md neu `{item}` nhung STORE_EXEMPT khong co no — luat da \
             viet rong hon cong dang thi hanh (mot muc THEM VAO da lot qua neu chi dem so \
             luong thay vi doi chieu tung muc)"
        );
    }

    assert_eq!(
        quoted.len(),
        STORE_EXEMPT.len(),
        "so muc trong menh de mien tru cua AGENTS.md ({}) khac so muc cua STORE_EXEMPT ({}) — \
         hai ben phai khop TUNG MUC, khong chi khop nhau ve tap hop khi co trung lap",
        quoted.len(),
        STORE_EXEMPT.len()
    );
}

/// 🔴 Đối chứng dương của vá P9: chứng minh [`exemption_items_named_in_rule_line`] THẬT SỰ
/// bắt được một mục thứ chín gieo tay — độc lập với nội dung thật của `AGENTS.md` hôm nay,
/// đúng khuôn đối chứng dương/âm mà mọi vị từ quét tĩnh khác trong tệp này phải có.
#[test]
fn the_exemption_clause_parser_would_actually_flag_a_seeded_ninth_item() {
    let seeded_nine = "Miễn trừ đủ tám mục, tất cả đặt tên cho KHO: `.atproj` · `project.db` · \
                        `StoreKind::Project` · `ProjectStore` · `PROJECT_MIGRATIONS` · \
                        `commands/project.rs` · `ports/project_store.rs` · \
                        `tests/project_contract.rs` · `một-mục-thứ-chín-gieo-tay`. Cổng canh.";

    let quoted = exemption_items_named_in_rule_line(seeded_nine);

    assert_eq!(quoted.len(), 9, "ca DUONG THAT: phai trich duoc du CHIN muc, nhan {quoted:?}");
    assert!(
        quoted.contains(&"một-mục-thứ-chín-gieo-tay"),
        "ca DUONG THAT: muc gieo tay phai co mat trong danh sach trich ra: {quoted:?}"
    );
    assert_ne!(
        quoted.len(),
        STORE_EXEMPT.len(),
        "muc gieo tay phai lam so luong LECH khoi STORE_EXEMPT — day chinh la dieu kien de \
         cong chinh phai do khi ai do them mot muc ma khong sua STORE_EXEMPT"
    );

    // Đối chứng ÂM: mệnh đề TÁM mục thật (không gieo) phải khớp đúng `STORE_EXEMPT`.
    let clean_eight = "Miễn trừ đủ tám mục, tất cả đặt tên cho KHO: `.atproj` · `project.db` · \
                        `StoreKind::Project` · `ProjectStore` · `PROJECT_MIGRATIONS` · \
                        `commands/project.rs` · `ports/project_store.rs` · \
                        `tests/project_contract.rs`. Cổng canh.";
    let clean_quoted = exemption_items_named_in_rule_line(clean_eight);
    assert_eq!(
        clean_quoted.len(),
        STORE_EXEMPT.len(),
        "ca AM: mot menh de TAM muc THAT (khong gieo) phai khop dung so luong STORE_EXEMPT"
    );
}
