//! Cổng HỢP ĐỒNG của Story 6.15 (FR128/AD-43) — xuất xứ tài liệu ở tầng `chapter`, trên ĐÚNG
//! đường sản phẩm (`create_work`/`update_chapter_origin`), không một hàm chỉ-test nào bọc
//! ngoài. Mỗi hàng của §I/O Matrix spec 6.15 có mặt ở đây, cộng hai đối chứng đỏ mà
//! §Verification đòi hỏi:
//!
//! - **Đối chứng đỏ ① — lượt xâu xuất xứ qua pipeline**: gỡ dòng `origin.push(...)` ở
//!   `Step::ExtractMainContent` (`core/segment/pipeline.rs`) phải làm
//!   `an_import_with_a_fully_declared_page_fills_all_four_columns_on_disk` (và các ca đọc
//!   `origin_*` khác) ĐỎ — `ImportedChapter::origin` sẽ luôn `None`.
//! - **Đối chứng đỏ ② — `write_lifecycle_after_change`**: gỡ dòng đó khỏi
//!   `commands::chapter::update_chapter_origin` phải làm
//!   `updating_chapter_origin_refreshes_the_cached_work_meta_updated_at` ĐỎ (`WorkMeta::updated_at`
//!   không được làm mới, chỉ mục Library đọc một dấu thời gian CŨ).
//!
//! URL dùng byte HTML gõ tay trực tiếp (`ChapterInput::RawBytes`/`UrlImportItem::raw`) — 0
//! mạng thật, cùng khuôn `segment_role_contract.rs`.

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Mutex;

use auratranslate_lib::commands::chapter::update_chapter_origin;
use auratranslate_lib::commands::project::{
    ChapterOriginOverride, UrlImportItem, cancel_import_preview, chapters_shape_if_all_ok, create_work,
    create_work_from_text, reset_chapter_origin_overrides, set_chapter_origin_override,
};
use auratranslate_lib::core::i18n::MessageKey;
use auratranslate_lib::core::store::{Migration, PROJECT_MIGRATIONS, Store, StoreSpec, Transaction};

static NEXT_DIR: AtomicU64 = AtomicU64::new(0);

fn temp_dir(tag: &str) -> PathBuf {
    let n = NEXT_DIR.fetch_add(1, Ordering::Relaxed);
    let dir = std::env::temp_dir().join(format!("auratranslate-origin-{}-{}-{}", std::process::id(), tag, n));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap_or_else(|e| panic!("tao {}: {e}", dir.display()));
    dir
}

fn cleanup(dir: &Path) {
    let _ = fs::remove_dir_all(dir);
}

fn url_item(url: &str, html: &str) -> UrlImportItem {
    UrlImportItem { url: url.to_owned(), raw: Some(html.as_bytes().to_vec()), error: None }
}

/// Trang khai ĐỦ cả ba trường HTML (`og:site_name`/`article:published_time`/`author`) — dùng
/// cho hàng I/O Matrix "trang khai đủ".
fn html_full_origin(body_extra: &str) -> String {
    format!(
        "<html><head><title>Bai viet</title>\
         <meta name=\"author\" content=\"Nguyen Van A\">\
         <meta property=\"og:site_name\" content=\"Bao Thi Du\">\
         <meta property=\"article:published_time\" content=\"2026-09-10T08:00:00+07:00\">\
         </head><body><article><h1>Tieu de</h1>\
         <p>Doan mot co du chu de duoc Readability chon lam noi dung chinh cua trang, \
         nhieu chu hon de vuot nguong do dai toi thieu.</p>\
         <p>Doan hai tiep tuc noi dung that su cua bai viet, khong phai menu hay quang cao, \
         du dai de dom_smoothie cham diem cao cho khoi nay.</p>\
         {body_extra}\
         </article></body></html>"
    )
}

/// Trang khai THIẾU tác giả — ba trường kia vẫn đủ.
fn html_missing_author() -> String {
    "<html><head><title>Bai viet</title>\
     <meta property=\"og:site_name\" content=\"Bao Thi Du\">\
     <meta property=\"article:published_time\" content=\"2026-09-10\">\
     </head><body><article><h1>Tieu de</h1>\
     <p>Doan mot co du chu de duoc Readability chon lam noi dung chinh cua trang, \
     nhieu chu hon de vuot nguong do dai toi thieu.</p>\
     <p>Doan hai tiep tuc noi dung that su cua bai viet, khong phai menu hay quang cao, \
     du dai de dom_smoothie cham diem cao cho khoi nay.</p>\
     </article></body></html>"
        .to_owned()
}

/// JSON-LD SAI CÚ PHÁP đứng cạnh một `<meta author>` hợp lệ — bước bóc phải bỏ qua đúng
/// nguồn hỏng đó và vẫn điền được `author` từ nguồn kế tiếp, không trượt cả lượt nhập.
fn html_broken_json_ld_but_valid_meta() -> String {
    "<html><head><title>Bai viet</title>\
     <script type=\"application/ld+json\">{ khong phai json hop le </script>\
     <meta name=\"author\" content=\"Tac Gia That\">\
     <meta property=\"og:site_name\" content=\"Bao That\">\
     </head><body><article><h1>Tieu de</h1>\
     <p>Doan mot co du chu de duoc Readability chon lam noi dung chinh cua trang, \
     nhieu chu hon de vuot nguong do dai toi thieu.</p>\
     <p>Doan hai tiep tuc noi dung that su cua bai viet, khong phai menu hay quang cao, \
     du dai de dom_smoothie cham diem cao cho khoi nay.</p>\
     </article></body></html>"
        .to_owned()
}

fn read_chapter_origin(
    store: &Store,
    chapter_id: i64,
) -> (Option<String>, Option<String>, Option<String>, Option<String>) {
    store
        .read(move |conn| {
            conn.query_row(
                "SELECT origin_author, origin_site_name, origin_url, origin_published_at \
                 FROM chapter WHERE id = ?1",
                [chapter_id],
                |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?)),
            )
        })
        .expect("doc bon cot xuat xu that bai")
}

fn read_chapter_updated_at(store: &Store, chapter_id: i64) -> String {
    store
        .read(move |conn| {
            conn.query_row("SELECT updated_at FROM chapter WHERE id = ?1", [chapter_id], |r| r.get(0))
        })
        .expect("doc updated_at that bai")
}

// ═════════════════════════════════════════════════════════════════════════════════
// §I/O Matrix — "Nhập URL, trang khai đủ"
// ═════════════════════════════════════════════════════════════════════════════════

#[test]
fn an_import_with_a_fully_declared_page_fills_all_four_columns_on_disk() {
    let root = temp_dir("full-origin");
    let items = vec![url_item("https://example.test/bai-viet", &html_full_origin(""))];
    let shape = chapters_shape_if_all_ok(&items).expect("shape phai dung duoc tu mot muc OK");

    let opened = create_work(
        &root,
        "Bai Viet Du",
        "en",
        "",
        shape,
        encoding_rs::UTF_8,
        Vec::new(),
        None,
        Vec::new(),
        &[],
        &Mutex::new(Vec::new()),
        None,
    )
    .expect("tao tac pham that bai");

    let (author, site_name, url, published_at) = read_chapter_origin(&opened.store, opened.chapter_id);
    assert_eq!(author.as_deref(), Some("Nguyen Van A"));
    assert_eq!(site_name.as_deref(), Some("Bao Thi Du"));
    assert_eq!(url.as_deref(), Some("https://example.test/bai-viet"), "URL bai goc = URL YEU CAU");
    assert_eq!(published_at.as_deref(), Some("2026-09-10"));

    let dir = opened.dir.clone();
    drop(opened);
    cleanup(&dir);
}

// ═════════════════════════════════════════════════════════════════════════════════
// §I/O Matrix — "Nhập URL, trang khai thiếu" ⇒ ô tác giả "không tìm thấy" (NULL), ba ô kia
// vẫn điền
// ═════════════════════════════════════════════════════════════════════════════════

#[test]
fn a_page_missing_the_author_tag_leaves_only_that_column_null() {
    let root = temp_dir("missing-author");
    let items = vec![url_item("https://example.test/thieu-tac-gia", &html_missing_author())];
    let shape = chapters_shape_if_all_ok(&items).expect("shape phai dung duoc");

    let opened = create_work(
        &root,
        "Thieu Tac Gia",
        "en",
        "",
        shape,
        encoding_rs::UTF_8,
        Vec::new(),
        None,
        Vec::new(),
        &[],
        &Mutex::new(Vec::new()),
        None,
    )
    .expect("tao tac pham that bai");

    let (author, site_name, url, published_at) = read_chapter_origin(&opened.store, opened.chapter_id);
    assert_eq!(author, None, "khong the tac gia nao ⇒ NULL, khong phai mot chuoi bia");
    assert_eq!(site_name.as_deref(), Some("Bao Thi Du"));
    assert_eq!(url.as_deref(), Some("https://example.test/thieu-tac-gia"));
    assert_eq!(published_at.as_deref(), Some("2026-09-10"));

    let dir = opened.dir.clone();
    drop(opened);
    cleanup(&dir);
}

// ═════════════════════════════════════════════════════════════════════════════════
// §I/O Matrix — "HTML hỏng / JSON-LD sai cú pháp" ⇒ bỏ qua ĐÚNG nguồn tín hiệu đó, thử nguồn
// kế tiếp; lượt nhập KHÔNG trượt
// ═════════════════════════════════════════════════════════════════════════════════

#[test]
fn a_syntactically_broken_json_ld_block_does_not_fail_the_import_and_the_next_signal_source_still_fills() {
    let root = temp_dir("broken-json-ld");
    let items = vec![url_item("https://example.test/json-ld-hong", &html_broken_json_ld_but_valid_meta())];
    let shape = chapters_shape_if_all_ok(&items).expect("shape phai dung duoc");

    let opened = create_work(
        &root,
        "Json Ld Hong",
        "en",
        "",
        shape,
        encoding_rs::UTF_8,
        Vec::new(),
        None,
        Vec::new(),
        &[],
        &Mutex::new(Vec::new()),
        None,
    )
    .expect("mot khoi JSON-LD sai cu phap KHONG duoc lam trot ca luot nhap");

    let (author, site_name, _url, _published_at) = read_chapter_origin(&opened.store, opened.chapter_id);
    assert_eq!(author.as_deref(), Some("Tac Gia That"), "nguon ke tiep (<meta author>) van phai dien duoc");
    assert_eq!(site_name.as_deref(), Some("Bao That"));

    let dir = opened.dir.clone();
    drop(opened);
    cleanup(&dir);
}

// ═════════════════════════════════════════════════════════════════════════════════
// §I/O Matrix — "Nhập từ file / dán tay" ⇒ cả bốn ô NULL (không chuỗi rỗng), khối vẫn nhập
// tay được từ danh sách Chương (kiểm ở test `update_chapter_origin` bên dưới)
// ═════════════════════════════════════════════════════════════════════════════════

#[test]
fn importing_from_pasted_text_leaves_all_four_origin_columns_null_not_empty_string() {
    let root = temp_dir("paste-no-origin");
    let opened = create_work_from_text(&root, "Dan Tay", "en", "", "Mot doan van ban dan tay.".to_owned())
        .expect("tao tac pham that bai");

    let (author, site_name, url, published_at) = read_chapter_origin(&opened.store, opened.chapter_id);
    assert_eq!(author, None);
    assert_eq!(site_name, None);
    assert_eq!(url, None);
    assert_eq!(published_at, None);

    let dir = opened.dir.clone();
    drop(opened);
    cleanup(&dir);
}

// ═════════════════════════════════════════════════════════════════════════════════
// §I/O Matrix — "Một trang tách thành nhiều Chương" ⇒ cả N Chương nhận CÙNG bộ bốn trường
// ═════════════════════════════════════════════════════════════════════════════════
//
// 🔴 Kiến trúc HÔM NAY không cho `chapter_pattern` khớp trên hình dạng `Chapters` (URL) —
// `already_chaptered` làm `Step::SplitChapters` bỏ qua HOÀN TOÀN (xem
// `webimport_contract.rs::a_single_link_import_is_exactly_one_chapter_even_with_a_chapter_pattern_configured`).
// Ca này vì thế xâu ĐÚNG cơ chế broadcast của `Flow::origins` (`pipeline.rs`) qua
// `run_import` trực tiếp — công khai chính để `tests/**` dựng đối chứng cho AD-39 — với hình
// dạng `Blob` + `extract_main_content: true` + `chapter_pattern` khớp 3 lần. Đây KHÔNG phải
// một đường sản phẩm hôm nay (không chỗ gọi `commands::project` nào dựng ra tổ hợp này), mà
// là cơ chế TẦNG PIPELINE phải đúng bất kể tổ hợp nào `tests/**` xâu vào — cùng triết lý mà
// `pipeline.rs::run_import_with_order` đã theo cho mọi đối chứng AD-39 khác.
#[test]
fn one_page_split_into_three_chapters_makes_every_chapter_carry_the_same_origin() {
    use auratranslate_lib::core::segment::chapterpattern::ChapterPattern;
    use auratranslate_lib::core::segment::pipeline::{ChapterInput, PipelineInput, PipelineShape, run_import};

    let body_extra = "<h2>===Chuong 2===</h2><p>Noi dung Chuong hai, cung du dai de vuot qua \
                       nguong toi thieu can thiet cho lam sach.</p>\
                       <h2>===Chuong 3===</h2><p>Noi dung Chuong ba, cung du dai de vuot qua \
                       nguong toi thieu can thiet cho lam sach nua.</p>";
    let html = html_full_origin(body_extra);

    // `RawBytes` (không `AlreadyText`) — `label_of(AlreadyText)` luôn trả CHUỖI RỖNG, và
    // `dom_smoothie` từ chối một URL rỗng ("the document URL must be absolute"). `RawBytes`
    // mang nhãn (URL) thật, cùng hình dạng đường sản phẩm dùng cho `.txt`/dán tay CÓ nhãn.
    let input = PipelineInput::default_shaped(
        PipelineShape::Blob(ChapterInput::RawBytes {
            bytes: html.into_bytes(),
            label: "https://example.test/bai-viet".to_owned(),
        }),
        "en",
    )
    .with_extract_main_content(true)
    .with_chapter_pattern(Some(ChapterPattern::literal("===Chuong")));

    let outcome = run_import(input).expect("pipeline phai chay duoc");
    assert!(outcome.chapters.len() >= 2, "mau phan tach phai khop it nhat mot lan de co N > 1 Chuong that");

    let first_origin = outcome.chapters[0].origin.clone().expect("Chuong dau phai co xuat xu (Some)");
    assert_eq!(first_origin.author.as_deref(), Some("Nguyen Van A"));
    assert_eq!(first_origin.site_name.as_deref(), Some("Bao Thi Du"));

    for (i, chapter) in outcome.chapters.iter().enumerate() {
        assert_eq!(
            chapter.origin, Some(first_origin.clone()),
            "Chuong chi so {i}: phai mang DUNG bo bon truong cua trang, giong het Chuong dau"
        );
    }
}

// ═════════════════════════════════════════════════════════════════════════════════
// §I/O Matrix — "Sửa tay ở xem trước rồi xác nhận" ⇒ giá trị ĐÃ GÕ xuống đĩa, không phải giá
// trị máy bóc
// ═════════════════════════════════════════════════════════════════════════════════

#[test]
fn a_hand_typed_override_at_preview_time_wins_over_the_machine_extracted_value() {
    let root = temp_dir("override-wins");
    let items = vec![url_item("https://example.test/bai-viet", &html_full_origin(""))];
    let shape = chapters_shape_if_all_ok(&items).expect("shape phai dung duoc");

    let overrides = vec![Some(ChapterOriginOverride {
        author: Some("Nguoi Dung Go Tay".to_owned()),
        site_name: None,
        url: None,
        published_at: None,
    })];

    let opened = create_work(
        &root,
        "Sua Tay Xac Nhan",
        "en",
        "",
        shape,
        encoding_rs::UTF_8,
        Vec::new(),
        None,
        Vec::new(),
        &overrides,
        &Mutex::new(Vec::new()),
        None,
    )
    .expect("tao tac pham that bai");

    let (author, site_name, _url, _published_at) = read_chapter_origin(&opened.store, opened.chapter_id);
    assert_eq!(author.as_deref(), Some("Nguoi Dung Go Tay"), "gia tri NGUOI DUNG go phai thang gia tri may");
    assert_eq!(site_name.as_deref(), Some("Bao Thi Du"), "truong KHONG bi cham van giu gia tri may");

    let dir = opened.dir.clone();
    drop(opened);
    cleanup(&dir);
}

/// Ô bị người dùng XOÁ TRẮNG (override `Some("")`) phải ghi `NULL`, không phải một chuỗi rỗng
/// — cùng luật `str::trim()` của `rename_chapter`.
#[test]
fn an_override_cleared_to_an_empty_string_stores_null_not_a_blank_string() {
    let root = temp_dir("override-cleared");
    let items = vec![url_item("https://example.test/bai-viet", &html_full_origin(""))];
    let shape = chapters_shape_if_all_ok(&items).expect("shape phai dung duoc");

    let overrides = vec![Some(ChapterOriginOverride {
        author: Some("   ".to_owned()),
        site_name: None,
        url: None,
        published_at: None,
    })];

    let opened = create_work(
        &root,
        "Xoa Trang Luc Xac Nhan",
        "en",
        "",
        shape,
        encoding_rs::UTF_8,
        Vec::new(),
        None,
        Vec::new(),
        &overrides,
        &Mutex::new(Vec::new()),
        None,
    )
    .expect("tao tac pham that bai");

    let (author, _site_name, _url, _published_at) = read_chapter_origin(&opened.store, opened.chapter_id);
    assert_eq!(author, None, "o da xoa trang phai ve NULL, khong phai chuoi rong/khoang trang");

    let dir = opened.dir.clone();
    drop(opened);
    cleanup(&dir);
}

// ═════════════════════════════════════════════════════════════════════════════════
// §I/O Matrix — "Sửa tay rồi huỷ xem trước" ⇒ 0 Tác phẩm được tạo, state ghi đè bị dọn
// ═════════════════════════════════════════════════════════════════════════════════

#[test]
fn cancelling_preview_after_typing_an_override_leaves_zero_atproj_and_clears_the_override_state() {
    use auratranslate_lib::commands::project::PendingImportSourceState;

    let root = temp_dir("cancel-after-typed");
    let state: PendingImportSourceState = Mutex::new(None);
    let overrides_state: auratranslate_lib::commands::project::ChapterOriginOverridesState = Mutex::new(Vec::new());

    set_chapter_origin_override(
        &overrides_state,
        0,
        ChapterOriginOverride {
            author: Some("Se Bi Huy".to_owned()),
            site_name: None,
            url: None,
            published_at: None,
        },
    );
    cancel_import_preview(&state);
    reset_chapter_origin_overrides(&overrides_state);

    let entries_before = fs::read_dir(&root).map(|it| it.count()).unwrap_or(0);
    assert_eq!(entries_before, 0, "huy truoc khi xac nhan khong duoc tao bat ky thu muc .atproj nao");

    let guard = overrides_state.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
    assert!(guard.is_empty(), "state ghi de xuat xu phai duoc don sach sau mot lot huy");
    drop(guard);

    cleanup(&root);
}

/// 🔴 **Đối chứng đỏ THỨ BA — rò rỉ `ChapterOriginOverridesState` giữa hai lượt nhập KHÁC
/// NGUỒN (lượt rà 2026-09-10).** Trước bản vá này, `wire::preview_import_encoding_from_text`/
/// `_from_file` dọn `Tier2BlockOverridesState` khi mở một lượt xem trước MỚI nhưng KHÔNG dọn
/// `ChapterOriginOverridesState` cạnh nó — một override còn treo từ một lượt nhập URL đã HUỶ
/// (huỷ không tự dọn state Rust, xem ca ngay trên) sống sót và bị đọc nhầm vào Chương của một
/// lượt DÁN TAY/TỆP hoàn toàn khác, đúng lớp lỗi "rỗng/hỏng ngầm" mà AGENTS.md gọi tên là
/// trung tâm — vi phạm thẳng §I/O Matrix "Nhập từ file / dán tay ⇒ cả bốn ô 'không tìm
/// thấy'".
///
/// ⚠️ **Vì sao ca này KHÔNG gọi thẳng `wire::preview_import_encoding_from_text`/
/// `confirm_import_with_encoding`** — kho không có `tauri::test`/`MockRuntime` (xem ghi chú
/// tương tự ở `tests/project_contract.rs:925`), nên hai `#[tauri::command]` đó không gọi được
/// từ `tests/**` mà không dựng một `tauri::AppHandle` thật. Ca này vì thế TÁI DIỄN ĐÚNG chuỗi
/// hàm THUẦN mà mỗi vỏ gọi, THEO ĐÚNG THỨ TỰ, trên CÙNG một `ChapterOriginOverridesState`:
/// bước "mở lượt xem trước MỚI" gọi [`reset_chapter_origin_overrides`] TRƯỚC
/// [`auratranslate_lib::commands::project::preview_import_encoding`] — đúng vị trí dòng vá vừa
/// thêm vào cả hai vỏ — rồi bước "xác nhận" đọc LẠI CHÍNH state đó (cùng khuôn
/// `resolve_chapter_origin_overrides` riêng tư của `mod wire`) để truyền vào [`create_work`].
/// Gỡ dòng `reset_chapter_origin_overrides(&overrides_state)` ở bước mở lượt xem trước MỚI
/// (mô phỏng đúng lượt gỡ dòng vá khỏi vỏ thật) làm ca này ĐỎ.
#[test]
fn a_leftover_override_from_a_cancelled_url_preview_never_reaches_a_pasted_text_confirm() {
    use auratranslate_lib::commands::project::{
        ChapterOriginOverridesState, PendingImportSourceState, preview_import_encoding,
    };
    use auratranslate_lib::core::segment::import::import_text;

    let root = temp_dir("origin-leak-across-sources");
    let overrides_state: ChapterOriginOverridesState = Mutex::new(Vec::new());
    let pending_state: PendingImportSourceState = Mutex::new(None);

    // Lượt URL TRƯỚC — người dùng gõ đè tác giả cho Chương 0, rồi HUỶ (không xuống đĩa, và
    // KHÔNG tự dọn `overrides_state` — xem ca ngay trên).
    set_chapter_origin_override(
        &overrides_state,
        0,
        ChapterOriginOverride {
            author: Some("Ro Ri Tu Luot URL Da Huy".to_owned()),
            site_name: None,
            url: None,
            published_at: None,
        },
    );
    cancel_import_preview(&pending_state);

    // Lượt DÁN TAY MỚI — tái diễn chuỗi hàm thuần của `wire::preview_import_encoding_from_text`
    // sau bản vá: dọn `ChapterOriginOverridesState` TRƯỚC khi dựng xem trước.
    //
    // 🔵 SỬA 2026-09-10 (vòng rà 1, mục 2) — câu cũ ở đây viết *"gỡ dòng dưới đây là gỡ ĐÚNG
    // dòng vá vừa thêm vào vỏ thật"*, và nó SAI. ĐO bằng phép gỡ thật: gỡ cả sáu lời gọi
    // `reset_chapter_origin_overrides(&app);` khỏi `mod wire` rồi chạy lại tệp này ⇒ **15/15
    // vẫn XANH**. Ca này gọi lượt dọn bằng CHÍNH TAY NÓ, nên nó canh chính nó, không canh vỏ.
    // Vai THẬT của nó hẹp hơn: chứng minh rằng KHI lượt dọn đã chạy thì bốn cột ra `NULL` —
    // một mệnh đề về `create_work`, không về vỏ. Thứ canh vỏ là
    // `ipc_contract.rs::both_preview_wires_reset_the_chapter_origin_overrides_before_building_a_preview`
    // (quét THÂN vỏ theo DÒNG MÃ; đỏ dưới cả phép xoá lẫn phép chú thích — đã đo cả hai).
    reset_chapter_origin_overrides(&overrides_state);
    let shape = import_text("Noi dung dan tay, khong xuat xu nao ca.".to_owned());
    let _preview = preview_import_encoding(&shape, "en", &[], None, &[], 0, &[]);

    // Lượt XÁC NHẬN — đọc LẠI đúng `overrides_state` (cùng khuôn `resolve_chapter_origin_overrides`
    // riêng tư của `mod wire`) rồi truyền vào `create_work`, đúng thứ `wire::confirm_import_with_encoding`
    // làm.
    let origin_overrides_at_confirm: Vec<Option<ChapterOriginOverride>> = overrides_state
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .clone();

    let opened = create_work(
        &root,
        "Dan Tay Sau Luot URL Bi Huy",
        "en",
        "",
        shape,
        encoding_rs::UTF_8,
        Vec::new(),
        None,
        Vec::new(),
        &origin_overrides_at_confirm,
        &Mutex::new(Vec::new()),
        None,
    )
    .expect("tao tac pham that bai");

    let (author, site_name, url, published_at) = read_chapter_origin(&opened.store, opened.chapter_id);
    assert_eq!(
        author, None,
        "override CON TREO cua lot URL da huy khong duoc ro ri sang Chuong cua lot dan tay MOI"
    );
    assert_eq!(site_name, None);
    assert_eq!(url, None);
    assert_eq!(published_at, None);

    let dir = opened.dir.clone();
    drop(opened);
    cleanup(&dir);
}

// ═════════════════════════════════════════════════════════════════════════════════
// §I/O Matrix — "Sửa ở danh sách Chương" ⇒ UPDATE đúng bốn cột cộng `updated_at`;
// `chapter_not_found` nếu id sai
// ═════════════════════════════════════════════════════════════════════════════════

#[test]
fn updating_chapter_origin_from_the_chapter_list_writes_exactly_the_four_columns_and_bumps_updated_at() {
    let root = temp_dir("update-from-list");
    let mut opened = create_work_from_text(&root, "Sua Tu Danh Sach", "en", "", "Noi dung.".to_owned())
        .expect("tao tac pham that bai");
    let chapter_id = opened.chapter_id;
    let updated_at_before = read_chapter_updated_at(&opened.store, chapter_id);

    std::thread::sleep(std::time::Duration::from_millis(5));

    let rows = update_chapter_origin(
        Some(&mut opened),
        chapter_id,
        "Tac Gia Nhap Tay",
        "Website Nhap Tay",
        "https://example.test/nhap-tay",
        "2026-01-01",
    )
    .expect("cap nhat xuat xu that bai");
    let row = rows.iter().find(|r| r.chapter_id == chapter_id).expect("phai co hang cho Chuong vua sua");
    assert_eq!(row.origin_author.as_deref(), Some("Tac Gia Nhap Tay"));
    assert_eq!(row.origin_site_name.as_deref(), Some("Website Nhap Tay"));
    assert_eq!(row.origin_url.as_deref(), Some("https://example.test/nhap-tay"));
    assert_eq!(row.origin_published_at.as_deref(), Some("2026-01-01"));

    let updated_at_after = read_chapter_updated_at(&opened.store, chapter_id);
    assert_ne!(updated_at_before, updated_at_after, "updated_at phai nhich sau lot sua xuat xu");

    let dir = opened.dir.clone();
    drop(opened);
    cleanup(&dir);
}

#[test]
fn updating_origin_with_an_unknown_chapter_id_reuses_the_named_error_and_touches_nothing() {
    let root = temp_dir("update-unknown-id");
    let mut opened = create_work_from_text(&root, "Id La", "en", "", "Noi dung.".to_owned())
        .expect("tao tac pham that bai");
    let real_id = opened.chapter_id;
    let updated_at_before = read_chapter_updated_at(&opened.store, real_id);

    let err = update_chapter_origin(Some(&mut opened), real_id + 999, "A", "B", "C", "D")
        .expect_err("chapter_id la phai la mot loi");
    assert_eq!(err.code(), "segment.chapter_not_found");
    assert_eq!(err.message_key(), MessageKey::SegmentChapterNotFound);

    let updated_at_after = read_chapter_updated_at(&opened.store, real_id);
    assert_eq!(updated_at_before, updated_at_after, "0 hang bi cham khi chapter_id sai");

    let dir = opened.dir.clone();
    drop(opened);
    cleanup(&dir);
}

/// Ô xoá trắng ở danh sách Chương cũng phải về `NULL`, không chuỗi rỗng — cùng luật màn xem
/// trước.
#[test]
fn clearing_a_field_to_empty_from_the_chapter_list_stores_null() {
    let root = temp_dir("clear-from-list");
    let mut opened = create_work_from_text(&root, "Xoa Trang Danh Sach", "en", "", "Noi dung.".to_owned())
        .expect("tao tac pham that bai");
    let chapter_id = opened.chapter_id;

    update_chapter_origin(Some(&mut opened), chapter_id, "Tac Gia", "", "", "")
        .expect("dat tac gia lan dau that bai");
    let (author, ..) = read_chapter_origin(&opened.store, chapter_id);
    assert_eq!(author.as_deref(), Some("Tac Gia"));

    update_chapter_origin(Some(&mut opened), chapter_id, "   ", "", "", "")
        .expect("xoa trang tac gia that bai");
    let (author_after, ..) = read_chapter_origin(&opened.store, chapter_id);
    assert_eq!(author_after, None, "xoa trang (chi khoang trang) phai ve NULL");

    let dir = opened.dir.clone();
    drop(opened);
    cleanup(&dir);
}

// ═════════════════════════════════════════════════════════════════════════════════
// Cách ly theo Chương — sửa Chương 2 không chạm Chương 1 và 3
// ═════════════════════════════════════════════════════════════════════════════════

#[test]
fn editing_chapter_two_of_three_never_touches_chapter_one_or_three() {
    let root = temp_dir("isolation-three-chapters");
    let items = vec![
        url_item("https://example.test/chuong-1", &html_full_origin("")),
        url_item("https://example.test/chuong-2", &html_missing_author()),
        url_item("https://example.test/chuong-3", &html_broken_json_ld_but_valid_meta()),
    ];
    let shape = chapters_shape_if_all_ok(&items).expect("shape phai dung duoc tu ba muc OK");

    let mut opened = create_work(
        &root,
        "Ba Chuong Cach Ly",
        "en",
        "",
        shape,
        encoding_rs::UTF_8,
        Vec::new(),
        None,
        Vec::new(),
        &[],
        &Mutex::new(Vec::new()),
        None,
    )
    .expect("tao tac pham that bai");

    let chapter_ids: Vec<i64> = opened
        .store
        .read(|conn| {
            let mut stmt = conn.prepare("SELECT id FROM chapter ORDER BY ord")?;
            let rows = stmt.query_map([], |r| r.get::<_, i64>(0))?;
            rows.collect::<Result<Vec<_>, _>>()
        })
        .expect("doc danh sach chapter_id that bai");
    assert_eq!(chapter_ids.len(), 3, "ba muc URL OK phai dung ba Chuong");

    let before: Vec<_> =
        chapter_ids.iter().map(|&id| read_chapter_origin(&opened.store, id)).collect();
    let updated_before: Vec<_> = chapter_ids.iter().map(|&id| read_chapter_updated_at(&opened.store, id)).collect();

    std::thread::sleep(std::time::Duration::from_millis(5));
    update_chapter_origin(
        Some(&mut opened),
        chapter_ids[1],
        "Tac Gia Chuong Hai",
        "",
        "",
        "",
    )
    .expect("sua Chuong 2 that bai");

    let after: Vec<_> = chapter_ids.iter().map(|&id| read_chapter_origin(&opened.store, id)).collect();
    let updated_after: Vec<_> = chapter_ids.iter().map(|&id| read_chapter_updated_at(&opened.store, id)).collect();

    assert_eq!(after[0], before[0], "Chuong 1 khong duoc doi mot byte nao");
    assert_eq!(after[2], before[2], "Chuong 3 khong duoc doi mot byte nao");
    assert_ne!(after[1].0.as_deref(), before[1].0.as_deref(), "Chuong 2 phai doi");

    assert_eq!(updated_after[0], updated_before[0], "updated_at cua Chuong 1 khong duoc nhich");
    assert_eq!(updated_after[2], updated_before[2], "updated_at cua Chuong 3 khong duoc nhich");
    assert_ne!(updated_after[1], updated_before[1], "updated_at cua Chuong 2 phai nhich");

    let dir = opened.dir.clone();
    drop(opened);
    cleanup(&dir);
}

// ═════════════════════════════════════════════════════════════════════════════════
// §I/O Matrix — mở `project.db` v21 bằng bản mới ⇒ lên v22, bốn cột NULL, 0 hàng mất
// ═════════════════════════════════════════════════════════════════════════════════

#[test]
fn a_version_21_project_database_migrates_to_22_with_all_origin_columns_null_and_zero_rows_lost() {
    // Đúng HAI MƯƠI bước THẬT của `PROJECT_MIGRATIONS` trước bước 22 (xuất xứ) — chỉ số 20
    // (bước THỨ 21, `SEGMENT_ROLE_DDL`) là phần tử CUỐI của fixture v21; chỉ số 20 trở đi (bước
    // 22, `CHAPTER_ORIGIN_DDL`) chính là bước đang được đo, KHÔNG được có mặt trong fixture.
    static OLD_STEPS: [Migration; 20] = [
        PROJECT_MIGRATIONS[0],
        PROJECT_MIGRATIONS[1],
        PROJECT_MIGRATIONS[2],
        PROJECT_MIGRATIONS[3],
        PROJECT_MIGRATIONS[4],
        PROJECT_MIGRATIONS[5],
        PROJECT_MIGRATIONS[6],
        PROJECT_MIGRATIONS[7],
        PROJECT_MIGRATIONS[8],
        PROJECT_MIGRATIONS[9],
        PROJECT_MIGRATIONS[10],
        PROJECT_MIGRATIONS[11],
        PROJECT_MIGRATIONS[12],
        PROJECT_MIGRATIONS[13],
        PROJECT_MIGRATIONS[14],
        PROJECT_MIGRATIONS[15],
        PROJECT_MIGRATIONS[16],
        PROJECT_MIGRATIONS[17],
        PROJECT_MIGRATIONS[18],
        PROJECT_MIGRATIONS[19],
    ];

    let root = temp_dir("migrate-21-to-22");
    let db = root.join("project.db");

    let old = Store::open(StoreSpec { migrations: &OLD_STEPS, ..StoreSpec::project(db.clone()) })
        .expect("dung fixture o phien ban 21");
    assert_eq!(old.schema_version(), 21, "fixture phai dung o dung phien ban 21");

    let chapter_id: i64 = old
        .write(|tx: &Transaction<'_>| {
            tx.execute(
                "INSERT INTO work (id, work_id, name, source_lang, genre, created_at, updated_at) \
                 VALUES (1, 'w', 'Fixture v21', 'en', '', strftime('%Y-%m-%dT%H:%M:%fZ','now'), \
                 strftime('%Y-%m-%dT%H:%M:%fZ','now'))",
                (),
            )?;
            tx.execute(
                "INSERT INTO chapter (ord, title, source_text, status, created_at, updated_at) \
                 VALUES (1, NULL, 'Noi dung cu.', 'not_started', strftime('%Y-%m-%dT%H:%M:%fZ','now'), \
                 strftime('%Y-%m-%dT%H:%M:%fZ','now'))",
                (),
            )?;
            let chapter_id = tx.last_insert_rowid();
            tx.execute(
                "INSERT INTO segment (chapter_id, ord, source_text, is_paragraph_end, \
                 is_target_paragraph_end, translation_origin, created_at, updated_at) \
                 VALUES (?1, 1, 'Noi dung cu.', 1, 0, '', strftime('%Y-%m-%dT%H:%M:%fZ','now'), \
                 strftime('%Y-%m-%dT%H:%M:%fZ','now'))",
                [chapter_id],
            )?;
            Ok(chapter_id)
        })
        .expect("ghi fixture v21 that bai");
    drop(old);

    let migrated = Store::open(StoreSpec::project(db)).expect("mot project.db o phien ban 21 phai mo duoc");
    assert_eq!(migrated.schema_version(), 22, "di tru phai chay het toi buoc 22 (chapter.origin_*)");

    let (author, site_name, url, published_at) = read_chapter_origin(&migrated, chapter_id);
    assert_eq!(author, None, "KHONG backfill -- hang CU phai giu ca bon cot NULL");
    assert_eq!(site_name, None);
    assert_eq!(url, None);
    assert_eq!(published_at, None);

    let (chapter_count, segment_count): (i64, i64) = migrated
        .read(|conn| {
            let c: i64 = conn.query_row("SELECT COUNT(*) FROM chapter", [], |r| r.get(0))?;
            let s: i64 = conn.query_row("SELECT COUNT(*) FROM segment", [], |r| r.get(0))?;
            Ok((c, s))
        })
        .expect("dem hang sau di tru that bai");
    assert_eq!(chapter_count, 1, "0 hang chapter nao duoc mat");
    assert_eq!(segment_count, 1, "0 hang segment nao duoc mat");

    drop(migrated);
    cleanup(&root);
}

// ═════════════════════════════════════════════════════════════════════════════════
// 🔴 Đối chứng đỏ ② — write_lifecycle_after_change
// ═════════════════════════════════════════════════════════════════════════════════

/// `WorkMeta::updated_at` là `MAX(work.created_at, MAX(chapter.updated_at),
/// MAX(segment.updated_at))` (`core/library/meta.rs::rebuild_from_store`) — một lượt sửa
/// xuất xứ bơm `chapter.updated_at` mới, nên `open.meta.updated_at` PHẢI đổi theo NẾU
/// `write_lifecycle_after_change` thật sự chạy. Gỡ dòng đó khỏi
/// `commands::chapter::update_chapter_origin` làm ca này ĐỎ — `opened.meta` đứng yên ở giá
/// trị CŨ dù đĩa đã đổi, đúng lớp lỗi "chỉ mục Library nói dối trong im lặng" mà §Boundaries
/// spec 6.15 gọi tên.
#[test]
fn updating_chapter_origin_refreshes_the_cached_work_meta_updated_at() {
    let root = temp_dir("origin-lifecycle-meta");
    let mut opened = create_work_from_text(&root, "Xuat Xu Lifecycle", "en", "", "Noi dung.".to_owned())
        .expect("tao tac pham that bai");
    let chapter_id = opened.chapter_id;
    let before = opened.meta.updated_at.clone();

    // Do phan giai `strftime('%f')` la mili-giay -- doi it nhat 5ms de dam bao dau thoi gian
    // MOI thuc su khac dau thoi gian CU, khong phai mot ca dong khung tinh co trung nhau.
    std::thread::sleep(std::time::Duration::from_millis(5));

    update_chapter_origin(Some(&mut opened), chapter_id, "Tac Gia Lifecycle", "", "", "")
        .expect("cap nhat xuat xu that bai");

    assert_ne!(
        opened.meta.updated_at, before,
        "write_lifecycle_after_change phai lam moi WorkMeta.updated_at sau lot sua xuat xu"
    );

    let dir = opened.dir.clone();
    drop(opened);
    cleanup(&dir);
}
