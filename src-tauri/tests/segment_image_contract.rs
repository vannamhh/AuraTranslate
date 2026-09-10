//! Cổng HỢP ĐỒNG của Story 6.14 (FR42 · FR43) — ảnh đúng vị trí, trên ĐÚNG đường sản phẩm
//! (`read_open_chapter_segments`/`read_reading_run`), không gọi thẳng
//! `core::segment::image::resolve_chapter_images`.
//!
//! ─────────────────────────────────────────────────────────────────────────────
//! VÌ SAO SETUP DÙNG SQL TRỰC TIẾP CHO `asset`/`role`/`retired_at`, KHÔNG PHẢI FULL PIPELINE
//! ─────────────────────────────────────────────────────────────────────────────
//! `asset_contract.rs`/`segment_contract.rs` đã lập tiền lệ: `INSERT INTO asset (...)` trực
//! tiếp qua `Store::write` là đường ĐÃ CHẤP NHẬN để dựng một trạng thái CỤ THỂ mà không cần
//! chạy trọn `create_work` + máy chủ TCP giả (`asset_contract.rs:436-444`,
//! `segment_contract.rs:2538` cho `retired_at`). Cổng NÀY không kiểm "ảnh có tải được không"
//! (đó là `asset_contract.rs`/`segment_role_contract.rs`, Story 6.11/6.13) — nó kiểm
//! "`read_open_chapter_segments`/`read_reading_run` có PHÂN GIẢI ĐÚNG một hàng `asset` đã có
//! sẵn hay không", nên "đường sản phẩm thật" ở ĐÂY nghĩa là gọi đúng hai hàm command đó, dữ
//! liệu đầu vào dựng bằng SQL là hợp lệ cho phạm vi này.
//!
//! Mỗi test khớp MỘT hàng của §I/O Matrix spec 6.14, tên hàm nêu đích danh hàng đó.

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use auratranslate_lib::commands::project::create_work_from_text;
use auratranslate_lib::commands::segment::{
    ChapterSegments, ReadingRun, read_open_chapter_segments, read_reading_run, set_segment_omitted,
};
use auratranslate_lib::core::store::Transaction;

static NEXT_DIR: AtomicU64 = AtomicU64::new(0);

fn temp_dir(tag: &str) -> PathBuf {
    let n = NEXT_DIR.fetch_add(1, Ordering::Relaxed);
    let dir = std::env::temp_dir().join(format!("auratranslate-image-{}-{}-{}", std::process::id(), tag, n));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap_or_else(|e| panic!("tao {}: {e}", dir.display()));
    dir
}

fn cleanup(dir: &Path) {
    let _ = fs::remove_dir_all(dir);
}

/// Năm câu — `segments()` (`read_open_chapter_segments`) trả về ĐÚNG NĂM hàng, `ord` 1..=5
/// theo đúng thứ tự tài liệu. Dùng chung cho mọi ca dưới đây cần một dãy segment "sạch".
fn work_with_five_segments(tag: &str) -> (PathBuf, auratranslate_lib::commands::project::OpenWork) {
    let root = temp_dir(tag);
    let opened = create_work_from_text(&root, "Anh 6.14", "zh", "", "一。二。三。四。五。".to_owned())
        .unwrap_or_else(|e| panic!("tao tac pham that bai: {e:?}"));
    (root, opened)
}

/// `id` của segment còn sống, theo `ord` (1-based) — đọc qua ĐÚNG lệnh sản phẩm.
fn segment_id_at(opened: &auratranslate_lib::commands::project::OpenWork, ord_1based: usize) -> i64 {
    let loaded: ChapterSegments =
        read_open_chapter_segments(Some(opened)).expect("nap segment that bai");
    loaded.segments[ord_1based - 1].id
}

/// Chèn một hàng `asset` — khuôn `asset_contract.rs:436-444`.
fn insert_asset(
    opened: &auratranslate_lib::commands::project::OpenWork,
    id: i64,
    file_name: &str,
    source_url: Option<&str>,
    anchor_after_segment_ord: i64,
) {
    let chapter_id = opened.chapter_id;
    let file_name = file_name.to_owned();
    let source_url = source_url.map(str::to_owned);
    opened
        .store
        .write(move |tx: &Transaction<'_>| {
            tx.execute(
                "INSERT INTO asset (id, chapter_id, file_name, source_url, anchor_after_segment_ord, \
                 byte_len, content_type, created_at) VALUES (?1, ?2, ?3, ?4, ?5, 10, 'image/jpeg', \
                 strftime('%Y-%m-%dT%H:%M:%fZ','now'))",
                rusqlite::params![id, chapter_id, file_name, source_url, anchor_after_segment_ord],
            )
        })
        .expect("chen hang asset that bai");
}

fn set_role(opened: &auratranslate_lib::commands::project::OpenWork, segment_id: i64, role: &str) {
    let role = role.to_owned();
    opened
        .store
        .write(move |tx: &Transaction<'_>| {
            tx.execute("UPDATE segment SET role = ?1 WHERE id = ?2", rusqlite::params![role, segment_id])
        })
        .expect("dat role that bai");
}

fn retire(opened: &auratranslate_lib::commands::project::OpenWork, segment_id: i64) {
    opened
        .store
        .write(move |tx: &Transaction<'_>| {
            tx.execute(
                "UPDATE segment SET retired_at = '2026-09-10T00:00:00.000Z' WHERE id = ?1",
                [segment_id],
            )
        })
        .expect("ve huu that bai");
}

fn asset_of<'a>(loaded: &'a ChapterSegments, asset_id: i64) -> &'a auratranslate_lib::commands::segment::ChapterAsset {
    loaded.assets.iter().find(|a| a.asset_id == asset_id).unwrap_or_else(|| panic!("khong thay asset {asset_id}"))
}

fn reading_image_of<'a>(run: &'a ReadingRun, asset_id: i64) -> &'a auratranslate_lib::commands::segment::ReadingImage {
    run.chapters[0]
        .images
        .iter()
        .find(|a| a.asset_id == asset_id)
        .unwrap_or_else(|| panic!("khong thay ReadingImage {asset_id}"))
}

/// Chương phải `Done` để `read_reading_run` trả nội dung (FR120, Story 5.12) — không liên
/// quan tới ảnh, nhưng bắt buộc để tới được nhánh có ảnh.
fn mark_chapter_done(opened: &auratranslate_lib::commands::project::OpenWork) {
    let chapter_id = opened.chapter_id;
    opened
        .store
        .write(move |tx: &Transaction<'_>| {
            tx.execute("UPDATE chapter SET status = 'done' WHERE id = ?1", [chapter_id])
        })
        .expect("dat done that bai");
}

// ═════════════════════════════════════════════════════════════════════════════════
// `assets_dir` — MỘT lần, tuyệt đối, đúng `<dir>/assets`
// ═════════════════════════════════════════════════════════════════════════════════

#[test]
fn assets_dir_is_the_absolute_assets_folder_of_the_open_work_on_both_surfaces() {
    let (root, opened) = work_with_five_segments("assets-dir");
    mark_chapter_done(&opened);
    let expected = opened.dir.join("assets");

    let loaded = read_open_chapter_segments(Some(&opened)).expect("nap segment that bai");
    assert_eq!(PathBuf::from(&loaded.assets_dir), expected);

    let run = read_reading_run(Some(&opened)).expect("nap luot doc that bai");
    assert_eq!(PathBuf::from(&run.assets_dir), expected);

    let dir = opened.dir.clone();
    drop(opened);
    cleanup(&dir);
    cleanup(&root);
}

// ═════════════════════════════════════════════════════════════════════════════════
// I/O Matrix — hàng theo hàng
// ═════════════════════════════════════════════════════════════════════════════════

#[test]
fn an_image_between_two_sentences_shows_after_sentence_k_on_both_surfaces() {
    let (root, opened) = work_with_five_segments("giua-hai-cau");
    mark_chapter_done(&opened);
    let seg2 = segment_id_at(&opened, 2);
    insert_asset(&opened, 900, "giua.jpg", None, 2);

    let loaded = read_open_chapter_segments(Some(&opened)).expect("nap segment that bai");
    assert_eq!(asset_of(&loaded, 900).after_segment_id, Some(seg2));

    let run = read_reading_run(Some(&opened)).expect("nap luot doc that bai");
    assert_eq!(reading_image_of(&run, 900).after_segment_id, Some(seg2));

    let dir = opened.dir.clone();
    drop(opened);
    cleanup(&dir);
    cleanup(&root);
}

#[test]
fn anchor_zero_resolves_to_none_on_both_surfaces() {
    let (root, opened) = work_with_five_segments("neo-khong");
    mark_chapter_done(&opened);
    insert_asset(&opened, 901, "dau.jpg", None, 0);

    let loaded = read_open_chapter_segments(Some(&opened)).expect("nap segment that bai");
    assert_eq!(asset_of(&loaded, 901).after_segment_id, None);

    let run = read_reading_run(Some(&opened)).expect("nap luot doc that bai");
    assert_eq!(reading_image_of(&run, 901).after_segment_id, None);

    let dir = opened.dir.clone();
    drop(opened);
    cleanup(&dir);
    cleanup(&root);
}

#[test]
fn an_anchor_pointing_at_a_cut_sentence_stays_on_the_grid_but_moves_back_for_reading() {
    let (root, opened) = work_with_five_segments("cat-bo");
    mark_chapter_done(&opened);
    let seg2 = segment_id_at(&opened, 2);
    let seg3 = segment_id_at(&opened, 3);
    set_segment_omitted(Some(&opened), seg3, true).expect("cat bo cau 3 that bai");
    insert_asset(&opened, 902, "sau-cat.jpg", None, 3);

    let loaded = read_open_chapter_segments(Some(&opened)).expect("nap segment that bai");
    assert_eq!(
        asset_of(&loaded, 902).after_segment_id,
        Some(seg3),
        "luoi KHONG loc cat bo -- anh van hien sau chinh cau da cat bo (FR44)"
    );

    let run = read_reading_run(Some(&opened)).expect("nap luot doc that bai");
    assert_eq!(
        reading_image_of(&run, 902).after_segment_id,
        Some(seg2),
        "Che do doc doi ve cau CON SONG lien truoc"
    );

    let dir = opened.dir.clone();
    drop(opened);
    cleanup(&dir);
    cleanup(&root);
}

#[test]
fn an_anchor_pointing_at_a_retired_sentence_moves_back_to_the_nearest_surviving_one_on_both_surfaces() {
    let (root, opened) = work_with_five_segments("ve-huu");
    mark_chapter_done(&opened);
    let seg2 = segment_id_at(&opened, 2);
    let seg3 = segment_id_at(&opened, 3);
    retire(&opened, seg3);
    // Neo van tro vao ord=3 (cau da ve huu) -- `select_chapter_segments` da loc no khoi day
    // truyen vao `resolve_chapter_images`, nen `ord=3` khong con ton tai trong tap do.
    insert_asset(&opened, 903, "ve-huu.jpg", None, 3);

    let loaded = read_open_chapter_segments(Some(&opened)).expect("nap segment that bai");
    assert_eq!(asset_of(&loaded, 903).after_segment_id, Some(seg2));

    let run = read_reading_run(Some(&opened)).expect("nap luot doc that bai");
    assert_eq!(reading_image_of(&run, 903).after_segment_id, Some(seg2));

    let dir = opened.dir.clone();
    drop(opened);
    cleanup(&dir);
    cleanup(&root);
}

#[test]
fn an_anchor_with_nothing_surviving_before_it_falls_back_to_the_start_of_the_chapter() {
    let (root, opened) = work_with_five_segments("dau-chuong-ve-huu");
    mark_chapter_done(&opened);
    let seg1 = segment_id_at(&opened, 1);
    retire(&opened, seg1);
    insert_asset(&opened, 904, "khong-doc-duoc.jpg", None, 1);

    let loaded = read_open_chapter_segments(Some(&opened)).expect("nap segment that bai");
    assert_eq!(asset_of(&loaded, 904).after_segment_id, None, "anchor khong doc duoc -- ve dau Chuong");

    let dir = opened.dir.clone();
    drop(opened);
    cleanup(&dir);
    cleanup(&root);
}

#[test]
fn two_images_sharing_an_anchor_both_show_ordered_by_ascending_asset_id() {
    let (root, opened) = work_with_five_segments("cung-neo");
    mark_chapter_done(&opened);
    let seg1 = segment_id_at(&opened, 1);
    // Chen id LON truoc, id NHO sau -- thu tu tra ve phai theo id TANG DAN, khong theo thu
    // tu chen.
    insert_asset(&opened, 950, "hai.jpg", None, 1);
    insert_asset(&opened, 910, "mot.jpg", None, 1);

    let loaded = read_open_chapter_segments(Some(&opened)).expect("nap segment that bai");
    let ids: Vec<i64> = loaded.assets.iter().map(|a| a.asset_id).collect();
    assert_eq!(ids, vec![910, 950]);
    assert!(loaded.assets.iter().all(|a| a.after_segment_id == Some(seg1)));

    let dir = opened.dir.clone();
    drop(opened);
    cleanup(&dir);
    cleanup(&root);
}

#[test]
fn an_image_without_a_caption_segment_yields_no_caption_text() {
    let (root, opened) = work_with_five_segments("khong-caption");
    mark_chapter_done(&opened);
    let seg1 = segment_id_at(&opened, 1);
    let seg2 = segment_id_at(&opened, 2);
    set_role(&opened, seg2, "alt");
    insert_asset(&opened, 920, "khong-caption.jpg", None, 1);

    let loaded = read_open_chapter_segments(Some(&opened)).expect("nap segment that bai");
    let asset = asset_of(&loaded, 920);
    assert!(asset.alt_text.is_some());
    assert_eq!(asset.caption_text, None);
    assert_eq!(asset.after_segment_id, Some(seg1), "neo van tro dung cau 1");

    let dir = opened.dir.clone();
    drop(opened);
    cleanup(&dir);
    cleanup(&root);
}

#[test]
fn an_image_without_an_alt_segment_yields_no_alt_text_and_the_frontend_treats_it_as_decorative() {
    let (root, opened) = work_with_five_segments("khong-alt");
    mark_chapter_done(&opened);
    let seg2 = segment_id_at(&opened, 2);
    set_role(&opened, seg2, "caption");
    insert_asset(&opened, 921, "khong-alt.jpg", None, 1);

    let loaded = read_open_chapter_segments(Some(&opened)).expect("nap segment that bai");
    let asset = asset_of(&loaded, 921);
    assert_eq!(asset.alt_text, None, "khong segment alt -- Rust KHONG bia chu; alt=\"\" la viec cua webview");
    assert!(asset.caption_text.is_some());

    let dir = opened.dir.clone();
    drop(opened);
    cleanup(&dir);
    cleanup(&root);
}

#[test]
fn an_untranslated_caption_segment_still_returns_its_raw_empty_target_text() {
    // "Khong chua dich -- khong khoi chu thich" la mot luat HIEN THI (webview), khong phai
    // luat cua ham Rust nay -- no tra NGUYEN VAN `target_text`, kha ca khi rong. Ca nay khoa
    // dung phan cua Rust: khong tu quyet dinh "rong thi tra None".
    let (root, opened) = work_with_five_segments("caption-chua-dich");
    mark_chapter_done(&opened);
    let seg2 = segment_id_at(&opened, 2);
    set_role(&opened, seg2, "caption"); // target_text mac dinh la '' (chua dich).
    insert_asset(&opened, 922, "caption-rong.jpg", None, 1);

    let loaded = read_open_chapter_segments(Some(&opened)).expect("nap segment that bai");
    assert_eq!(asset_of(&loaded, 922).caption_text.as_deref(), Some(""));

    let dir = opened.dir.clone();
    drop(opened);
    cleanup(&dir);
    cleanup(&root);
}

#[test]
fn a_chapter_that_is_not_done_yet_contributes_zero_images_to_the_reading_run() {
    let (root, opened) = work_with_five_segments("chua-done");
    // KHONG goi mark_chapter_done -- Chuong con "in_progress" (mac dinh cua create_work).
    insert_asset(&opened, 930, "khong-hien.jpg", None, 1);

    let run = read_reading_run(Some(&opened)).expect("nap luot doc that bai");
    assert!(run.chapters.is_empty(), "Chuong chua Done khong duoc vao dãy doc -- khong duong vong");

    let dir = opened.dir.clone();
    drop(opened);
    cleanup(&dir);
    cleanup(&root);
}

#[test]
fn a_chapter_with_zero_assets_renders_identically_to_before_this_story_on_both_surfaces() {
    let (root, opened) = work_with_five_segments("khong-anh");
    mark_chapter_done(&opened);
    // KHONG chen hang `asset` nao.

    let loaded = read_open_chapter_segments(Some(&opened)).expect("nap segment that bai");
    assert_eq!(loaded.assets, Vec::new(), "Chuong 0 anh -- mang assets phai RONG, khong None/loi");
    assert_eq!(loaded.segments.len(), 5, "segment van nap binh thuong, khong bi anh huong");

    let run = read_reading_run(Some(&opened)).expect("nap luot doc that bai");
    assert_eq!(run.chapters[0].images, Vec::new());
    assert_eq!(
        run.chapters[0].paragraphs[0].segments.len(),
        5,
        "van dung nam cau nhu truoc story nay -- khong segment nao bi anh huong boi tinh nang anh"
    );

    let dir = opened.dir.clone();
    drop(opened);
    cleanup(&dir);
    cleanup(&root);
}

// ═════════════════════════════════════════════════════════════════════════════════
// 🔴 CHOT LOC VAI TREN DUONG DOC -- CHO NOI, khong phai ham thuan
// ═════════════════════════════════════════════════════════════════════════════════
//
// Ca nay ra doi tu mot phep DOT BIEN o vong ra 2026-09-10: go `image::strip_role_segments`
// khoi `read_reading_run` (de lai `paragraphs_in_translation` tran) thi CA BO `cargo test`
// van XANH -- 12/12 o chinh tep nay. Hai ca don vi cua `strip_role_segments` trong
// `core/segment/image.rs` kiem HAM do tach roi, nen go LOI GOI thi chung khong thay gi; con
// test frontend thi gia o bien IPC, nen fixture cua no khong bao gio chua segment vai.
//
// ⇒ Menh de dau bang cua Story 6.14 -- "alt-text KHONG hien tren trang" (FR43, EXPERIENCE.md:357)
// -- co ZERO phep do tren duong san pham. Ca duoi day canh dung cho noi do: mot segment mang
// vai THAT, di qua `read_reading_run` THAT.
#[test]
fn a_role_bearing_segment_never_reaches_the_reading_page_as_prose() {
    let (root, opened) = work_with_five_segments("vai-khong-len-trang");
    mark_chapter_done(&opened);
    let seg2 = segment_id_at(&opened, 2);
    let seg3 = segment_id_at(&opened, 3);
    set_role(&opened, seg2, "alt");
    set_role(&opened, seg3, "caption");

    let run = read_reading_run(Some(&opened)).expect("nap luot doc that bai");

    let on_page: Vec<i64> = run.chapters[0]
        .paragraphs
        .iter()
        .flat_map(|p| p.segments.iter().map(|s| s.id))
        .collect();
    assert!(
        !on_page.contains(&seg2),
        "segment role='alt' LEN TRANG doc nhu van xuoi -- chot loc vai da bi go khoi `read_reading_run`. \
         Tren trang: {on_page:?}, alt la {seg2}"
    );
    assert!(
        !on_page.contains(&seg3),
        "segment role='caption' LEN TRANG doc nhu van xuoi -- caption phai di vao <figcaption>, \
         khong vao dong van. Tren trang: {on_page:?}, caption la {seg3}"
    );
    assert_eq!(on_page.len(), 3, "ba cau van xuoi con lai van len trang du");

    let dir = opened.dir.clone();
    drop(opened);
    cleanup(&dir);
    cleanup(&root);
}

// ═════════════════════════════════════════════════════════════════════════════════
// Cach ly theo CHUONG -- `WHERE chapter_id = ?1` la mot cau SQL, va cau SQL nao cung
// co the bi sua. Vong ra 2026-09-10: 0 ca nao dung Tac pham NHIEU Chuong.
// ═════════════════════════════════════════════════════════════════════════════════
#[test]
fn an_asset_row_of_another_chapter_never_leaks_into_this_chapters_images() {
    let root = temp_dir("cach-ly-chuong");
    let opened = create_work_from_text(&root, "Anh 6.14 hai chuong", "zh", "###", "Mot. Hai.\n###\nBa. Bon.".to_owned())
        .unwrap_or_else(|e| panic!("tao tac pham that bai: {e:?}"));

    // Hang `asset` gan cho mot chapter_id KHAC hoan toan (999) -- khong Chuong nao cua Tac
    // pham nay mang id do.
    let chapter_id_khac = 999_i64;
    opened
        .store
        .write(move |tx: &Transaction<'_>| {
            tx.execute(
                "INSERT INTO asset (id, chapter_id, file_name, source_url, anchor_after_segment_ord, \
                 byte_len, content_type, created_at) VALUES (930, ?1, 'chuong-khac.jpg', NULL, 1, 10, \
                 'image/jpeg', strftime('%Y-%m-%dT%H:%M:%fZ','now'))",
                [chapter_id_khac],
            )
        })
        .expect("chen hang asset that bai");

    let loaded = read_open_chapter_segments(Some(&opened)).expect("nap segment that bai");
    assert!(
        loaded.assets.iter().all(|a| a.asset_id != 930),
        "anh cua Chuong KHAC lot vao day: {:?}",
        loaded.assets
    );

    let dir = opened.dir.clone();
    drop(opened);
    cleanup(&dir);
    cleanup(&root);
}
