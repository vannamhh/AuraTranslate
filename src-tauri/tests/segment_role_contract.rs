//! Cổng HỢP ĐỒNG của Story 6.13 (FR129, AD-42) — alt-text/caption là hai `Segment` mang
//! trường **vai**, trên ĐÚNG đường sản phẩm (`create_work`), không một hàm chỉ-test nào bọc
//! ngoài. Mỗi hàng của I/O Matrix spec 6.13 có mặt ở đây, cộng hai đối chứng đỏ mà §Verification
//! đòi hỏi:
//!
//! - **Đối chứng đỏ ① — dời neo**: `two_adjacent_kept_images_the_second_anchor_shifts_by_the_first_alt`
//!   đỏ nếu lượt dời neo (`role::weave_chapter_segments::shifted_anchor_by_block`) bị gỡ, vì
//!   nó khẳng định neo THẬT ghi xuống `asset` của ảnh THỨ HAI, không chỉ đếm segment.
//! - **Đối chứng đỏ ② — cột `role`**: mọi ca đọc `role='alt'`/`role='caption'` từ `segment`
//!   đỏ nếu `insert_segments` bỏ qua cột đó (rơi về `NULL` của `ALTER TABLE`).
//!
//! Ảnh dùng server TCP thô tự dựng (khuôn `asset_contract.rs::spawn_counting_image_server`) —
//! không mạng ngoài. Quyết định 2 (spec 6.13) nói segment `alt` sinh ĐỘC LẬP với việc tải ảnh
//! có thành công hay không — phần lớn ca dưới đây cố ý dùng một cổng KHÔNG AI LẮNG NGHE
//! (`unreachable_port`) để đo đúng mệnh đề đó mà không cần dựng một máy chủ thật.

use std::fs;
use std::io::{Read, Write};
use std::net::TcpListener;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Mutex;
use std::thread;
use std::time::Duration;

use auratranslate_lib::commands::project::{
    UrlImportItem, chapters_shape_if_all_ok, create_work, create_work_from_text,
};
use auratranslate_lib::commands::segment::{
    confirm_segment, merge_segments, read_open_chapter_segments, save_segment_targets, split_segment,
    SegmentTargetEdit,
};
use auratranslate_lib::core::cleanup::{CleanupRule, CleanupRuleKind, CleanupRuleTier};
use auratranslate_lib::core::store::{Migration, PROJECT_MIGRATIONS, Store, StoreSpec, Transaction};

static NEXT_DIR: AtomicU64 = AtomicU64::new(0);

fn temp_dir(tag: &str) -> PathBuf {
    let n = NEXT_DIR.fetch_add(1, Ordering::Relaxed);
    let dir = std::env::temp_dir().join(format!("auratranslate-role-{}-{}-{}", std::process::id(), tag, n));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap_or_else(|e| panic!("tao {}: {e}", dir.display()));
    dir
}

fn cleanup(dir: &Path) {
    let _ = fs::remove_dir_all(dir);
}

fn url_item(url: &str, html: String) -> UrlImportItem {
    UrlImportItem { url: url.to_owned(), raw: Some(html.into_bytes()), error: None }
}

/// Một cổng KHÔNG AI LẮNG NGHE — mô phỏng tất định "kết nối bị từ chối", KHÔNG cần một máy
/// chủ giả. Ảnh trỏ vào cổng này luôn đếm vào `images_failed`, 0 hàng `asset` — điều kiện
/// đúng để đo Quyết định 2 (alt sinh ĐỘC LẬP với việc tải ảnh).
fn unreachable_port() -> u16 {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind cong tam");
    listener.local_addr().expect("local_addr").port()
}

/// Server tối giản trả về ĐÚNG một ảnh hợp lệ cho `max_connections` kết nối đầu tiên — khuôn
/// `asset_contract.rs::spawn_counting_image_server`, rút gọn (không cần bộ đếm ở đây).
fn spawn_image_server(max_connections: usize, body: &'static [u8]) -> u16 {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind cong tam");
    let port = listener.local_addr().expect("local_addr").port();
    thread::spawn(move || {
        for _ in 0..max_connections {
            let Ok((mut stream, _)) = listener.accept() else { break };
            let _ = stream.set_read_timeout(Some(Duration::from_secs(5)));
            let mut discard = [0u8; 4096];
            let _ = stream.read(&mut discard);
            let header = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: image/jpeg\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
                body.len()
            );
            let _ = stream.write_all(header.as_bytes());
            let _ = stream.write_all(body);
        }
    });
    port
}

/// Trang có `<h1>` + hai đoạn văn (mỗi đoạn ĐÚNG một câu) TRƯỚC một ảnh — cùng fixture
/// `asset_contract.rs::html_page_with_one_image` (neo = **3**, đo được không suy), rồi một
/// khối tuỳ biến ngay sau ảnh (ảnh/caption/không gì), rồi một đoạn kết.
fn html_with_image_and_tail(img_src: &str, alt: Option<&str>, tail_after_image: &str) -> String {
    let alt_attr = alt.map(|a| format!(" alt=\"{a}\"")).unwrap_or_default();
    format!(
        "<html><head><title>Bai viet</title></head><body><article><h1>Tieu de</h1>\
         <p>Doan mot co du chu de duoc Readability chon lam noi dung chinh cua trang, \
         nhieu chu hon de vuot nguong do dai toi thieu.</p>\
         <p>Doan hai tiep tuc noi dung that su cua bai viet, khong phai menu hay quang cao, \
         du dai de dom_smoothie cham diem cao cho khoi nay.</p>\
         <img src=\"{img_src}\"{alt_attr}>\
         {tail_after_image}\
         <p>Doan ba dong y nghia, giu cho tong do dai van ban vuot qua nguong toi thieu can \
         thiet de Readability tin day la mot bai viet that.</p>\
         </article></body></html>"
    )
}

fn html_two_adjacent_images(img_a: &str, alt_a: Option<&str>, img_b: &str, alt_b: Option<&str>) -> String {
    let alt_attr_a = alt_a.map(|a| format!(" alt=\"{a}\"")).unwrap_or_default();
    let alt_attr_b = alt_b.map(|a| format!(" alt=\"{a}\"")).unwrap_or_default();
    format!(
        "<html><head><title>Bai viet</title></head><body><article><h1>Tieu de</h1>\
         <p>Doan mot co du chu de duoc Readability chon lam noi dung chinh cua trang, \
         nhieu chu hon de vuot nguong do dai toi thieu.</p>\
         <p>Doan hai tiep tuc noi dung that su cua bai viet, khong phai menu hay quang cao, \
         du dai de dom_smoothie cham diem cao cho khoi nay.</p>\
         <img src=\"{img_a}\"{alt_attr_a}>\
         <img src=\"{img_b}\"{alt_attr_b}>\
         <p>Doan ba dong y nghia, giu cho tong do dai van ban vuot qua nguong toi thieu can \
         thiet de Readability tin day la mot bai viet that.</p>\
         </article></body></html>"
    )
}

/// Đọc `(ord, role, source_text)` của mọi segment SỐNG của Chương, theo `ord` — hình dạng THÔ
/// đủ cho mọi ca ở tệp này.
fn read_role_rows(store: &auratranslate_lib::core::store::Store, chapter_id: i64) -> Vec<(i64, Option<String>, String)> {
    store
        .read(move |conn| {
            let mut stmt = conn.prepare(
                "SELECT ord, role, source_text FROM segment WHERE chapter_id = ?1 \
                 AND retired_at IS NULL ORDER BY ord",
            )?;
            let rows = stmt.query_map([chapter_id], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)))?;
            rows.collect::<Result<Vec<_>, _>>()
        })
        .expect("doc segment that bai")
}

/// Mọi neo `anchor_after_segment_ord` đã ghi, sắp tăng dần. `file_name` là một UUID ngẫu
/// nhiên (`Uuid::new_v4()`, `commands/project.rs`) — KHÔNG mang lại dấu vết nào của URL nguồn,
/// nên đọc THEO NEO (thứ tự tài liệu, không suy từ tên tệp) là cách đúng để phân biệt ảnh A
/// với ảnh B trong ca này.
fn read_asset_anchors_sorted(store: &auratranslate_lib::core::store::Store) -> Vec<i64> {
    store
        .read(|conn| {
            let mut stmt =
                conn.prepare("SELECT anchor_after_segment_ord FROM asset ORDER BY anchor_after_segment_ord, id")?;
            let rows = stmt.query_map([], |r| r.get::<_, i64>(0))?;
            rows.collect::<Result<Vec<_>, _>>()
        })
        .expect("doc asset that bai")
}

// ═════════════════════════════════════════════════════════════════════════════════
// AC1 — Ảnh có alt + figcaption: đúng một hàng 'alt', đúng một hàng 'caption', đúng thứ tự,
// 0 cột text nào thêm vào `asset`.
// ═════════════════════════════════════════════════════════════════════════════════

#[test]
fn an_image_with_alt_and_a_following_caption_get_role_segments_right_after_the_anchor_in_order() {
    let root = temp_dir("ac1-alt-caption");
    let img_url = format!("http://127.0.0.1:{}/anh.jpg", unreachable_port());
    let html = html_with_image_and_tail(
        &img_url,
        Some("mo ta anh minh hoa"),
        "<figcaption>Chu thich cho anh nay.</figcaption>",
    );
    let items = vec![url_item("https://example.test/bai-1", html)];
    let shape = chapters_shape_if_all_ok(&items).expect("danh sach toan muc OK");

    let opened = create_work(&root, "AC1", "en", "", shape, encoding_rs::UTF_8, Vec::new(), None, Vec::new(), 0, 1, false, &[], &Mutex::new(Vec::new()), None)
        .expect("tao Tac pham that bai");

    let rows = read_role_rows(&opened.store, opened.chapter_id);
    // h1, doan1, doan2 (neo=3) -- ROLE:alt -- ROLE:caption -- doan3.
    assert_eq!(rows.len(), 6, "phai co dung 6 hang (3 van xuoi truoc + alt + caption + 1 van xuoi sau): {rows:?}");
    assert_eq!(rows[3].1.as_deref(), Some("alt"), "hang thu 4 phai mang role='alt': {rows:?}");
    assert_eq!(rows[3].2, "mo ta anh minh hoa");
    assert_eq!(rows[4].1.as_deref(), Some("caption"), "hang thu 5 phai mang role='caption': {rows:?}");
    assert_eq!(rows[4].2, "Chu thich cho anh nay.");
    assert!(rows[0].1.is_none() && rows[1].1.is_none() && rows[2].1.is_none() && rows[5].1.is_none(), "sau hang van xuoi khong mang vai nao: {rows:?}");

    let alt_count = rows.iter().filter(|(_, r, _)| r.as_deref() == Some("alt")).count();
    let caption_count = rows.iter().filter(|(_, r, _)| r.as_deref() == Some("caption")).count();
    assert_eq!(alt_count, 1, "dung MOT hang role='alt'");
    assert_eq!(caption_count, 1, "dung MOT hang role='caption'");

    // 🔴 Doc lai qua DTO `ChapterSegment` that su (`read_open_chapter_segments`, day IPC
    // `select_chapter_segments`'s `row.get(9)`) -- khong chi qua SQL tho o tren. Chua ca nao
    // trong tep nay doc `.role` qua duong nay truoc lan sua nay, nen chi so cot cua bo doc DTO
    // chua tung duoc do voi mot gia tri KHAC NULL.
    let wire = read_open_chapter_segments(Some(&opened)).expect("doc segment qua day IPC");
    assert_eq!(wire.segments.len(), 6);
    assert_eq!(wire.segments[3].role.as_deref(), Some("alt"), "DTO qua day phai mang dung role='alt': {:?}", wire.segments[3]);
    assert_eq!(wire.segments[4].role.as_deref(), Some("caption"), "DTO qua day phai mang dung role='caption': {:?}", wire.segments[4]);
    assert!(
        wire.segments[0].role.is_none()
            && wire.segments[1].role.is_none()
            && wire.segments[2].role.is_none()
            && wire.segments[5].role.is_none(),
        "cac hang van xuoi phai mang role = None qua day IPC: {:?}",
        wire.segments
    );

    // 0 cot text nao them vao `asset` -- bang van chi co dung TAM cot cua ASSET_DDL (Story
    // 6.11: id, chapter_id, file_name, source_url, anchor_after_segment_ord, byte_len,
    // content_type, created_at), khong `alt`/`caption` nao ca.
    let asset_columns: i64 = opened
        .store
        .read(|conn| conn.query_row("SELECT COUNT(*) FROM pragma_table_info('asset')", [], |r| r.get(0)))
        .expect("dem cot asset");
    assert_eq!(asset_columns, 8, "bang `asset` phai giu dung tam cot cua ASSET_DDL, khong them cot text nao");

    drop(opened);
    cleanup(&root);
}

// ═════════════════════════════════════════════════════════════════════════════════
// Đối chứng đỏ ① — dời neo: ảnh THỨ HAI phải cộng thêm đúng 1 vì `alt` của ảnh ĐẦU đã chen
// một segment vào TRƯỚC nó.
// ═════════════════════════════════════════════════════════════════════════════════

#[test]
fn two_adjacent_kept_images_the_second_anchor_shifts_by_the_first_alt() {
    let root = temp_dir("shift-two-images");
    let body: &'static [u8] = b"jpeg-bytes-anh-a";
    let port_a = spawn_image_server(4, body);
    let port_b = spawn_image_server(4, body);
    let img_a = format!("http://127.0.0.1:{port_a}/a.jpg");
    let img_b = format!("http://127.0.0.1:{port_b}/b.jpg");

    let html = html_two_adjacent_images(&img_a, Some("mo ta A"), &img_b, None);
    let items = vec![url_item("https://example.test/bai-2", html)];
    let shape = chapters_shape_if_all_ok(&items).expect("danh sach toan muc OK");

    let opened = create_work(&root, "Doi chung do 1", "en", "", shape, encoding_rs::UTF_8, Vec::new(), None, Vec::new(), 0, 1, false, &[], &Mutex::new(Vec::new()), None)
        .expect("tao Tac pham that bai");

    assert_eq!(opened.images_saved, 2, "ca hai anh phai tai duoc");

    // Hai file_name la UUID ngau nhien, khong mang dau vet URL nguon -- doc THEO NEO (thu tu
    // tai lieu) la cach dung de phan biet anh A (dung TRUOC, co alt) voi anh B (dung SAU).
    let anchors = read_asset_anchors_sorted(&opened.store);
    assert_eq!(
        anchors,
        vec![3, 4],
        "neo cua anh A (co alt) khong doi (van la 3, khong anh nao dung TRUOC no de chen \
         them segment); neo cua anh B PHAI cong them 1 vi alt cua A da chen dung MOT segment \
         truoc no -- day la doi chung do cho loi dat neo: go luot doi neo thi ca hai se cung \
         mang gia tri 3"
    );

    let rows = read_role_rows(&opened.store, opened.chapter_id);
    let alt_rows: Vec<_> = rows.iter().filter(|(_, r, _)| r.as_deref() == Some("alt")).collect();
    assert_eq!(alt_rows.len(), 1, "chi anh A co alt -- dung MOT hang role='alt': {rows:?}");
    assert_eq!(alt_rows[0].2, "mo ta A");

    drop(opened);
    cleanup(&root);
}

// ═════════════════════════════════════════════════════════════════════════════════
// Quyết định 2 — alt sinh ĐỘC LẬP với việc tải ảnh có thành công hay không.
// ═════════════════════════════════════════════════════════════════════════════════

#[test]
fn an_image_whose_asset_fetch_fails_still_gets_its_alt_segment() {
    let root = temp_dir("alt-independent-of-fetch");
    let img_url = format!("http://127.0.0.1:{}/khong-ai-nghe.jpg", unreachable_port());
    let html = html_with_image_and_tail(&img_url, Some("mo ta du anh khong tai duoc"), "");
    let items = vec![url_item("https://example.test/bai-3", html)];
    let shape = chapters_shape_if_all_ok(&items).expect("danh sach toan muc OK");

    let opened = create_work(&root, "Quyet dinh 2", "en", "", shape, encoding_rs::UTF_8, Vec::new(), None, Vec::new(), 0, 1, false, &[], &Mutex::new(Vec::new()), None)
        .expect("tao Tac pham that bai");

    assert_eq!(opened.images_saved, 0, "cong khong ai nghe -- 0 anh tai duoc");
    assert_eq!(opened.images_failed, 1);

    let rows = read_role_rows(&opened.store, opened.chapter_id);
    let alt_rows: Vec<_> = rows.iter().filter(|(_, r, _)| r.as_deref() == Some("alt")).collect();
    assert_eq!(
        alt_rows.len(),
        1,
        "Quyet dinh 2: segment alt sinh cho MOI anh giu co alt khac rong, DOC LAP voi viec \
         tai anh co thanh cong hay khong: {rows:?}"
    );
    assert_eq!(alt_rows[0].2, "mo ta du anh khong tai duoc");

    drop(opened);
    cleanup(&root);
}

// ═════════════════════════════════════════════════════════════════════════════════
// I/O Matrix — alt rỗng/chỉ khoảng trắng ⇒ 0 segment; ảnh không có alt ⇒ 0 segment.
// ═════════════════════════════════════════════════════════════════════════════════

#[test]
fn a_whitespace_only_alt_generates_no_role_segment() {
    let root = temp_dir("alt-whitespace-only");
    let img_url = format!("http://127.0.0.1:{}/anh.jpg", unreachable_port());
    let html = html_with_image_and_tail(&img_url, Some("   "), "");
    let items = vec![url_item("https://example.test/bai-4", html)];
    let shape = chapters_shape_if_all_ok(&items).expect("danh sach toan muc OK");

    let opened = create_work(&root, "Alt trang", "en", "", shape, encoding_rs::UTF_8, Vec::new(), None, Vec::new(), 0, 1, false, &[], &Mutex::new(Vec::new()), None)
        .expect("tao Tac pham that bai");

    let rows = read_role_rows(&opened.store, opened.chapter_id);
    assert!(rows.iter().all(|(_, r, _)| r.is_none()), "alt chi khoang trang -- 0 segment vai nao: {rows:?}");
    assert_eq!(rows.len(), 4, "van dung 4 segment van xuoi (h1 + 3 doan), khong hut them hang nao");

    drop(opened);
    cleanup(&root);
}

#[test]
fn an_image_with_no_alt_attribute_generates_no_role_segment_and_does_not_shift_anchors() {
    let root = temp_dir("no-alt-attribute");
    let img_url = format!("http://127.0.0.1:{}/anh.jpg", unreachable_port());
    let html = html_with_image_and_tail(&img_url, None, "");
    let items = vec![url_item("https://example.test/bai-5", html)];
    let shape = chapters_shape_if_all_ok(&items).expect("danh sach toan muc OK");

    let opened = create_work(&root, "Khong alt", "en", "", shape, encoding_rs::UTF_8, Vec::new(), None, Vec::new(), 0, 1, false, &[], &Mutex::new(Vec::new()), None)
        .expect("tao Tac pham that bai");

    let rows = read_role_rows(&opened.store, opened.chapter_id);
    assert!(rows.iter().all(|(_, r, _)| r.is_none()));
    assert_eq!(rows.len(), 4);

    drop(opened);
    cleanup(&root);
}

// ═════════════════════════════════════════════════════════════════════════════════
// Đối chứng đỏ ③ — khối `figcaption` BA CÂU vẫn cho ĐÚNG MỘT segment `role='caption'`.
// ═════════════════════════════════════════════════════════════════════════════════

#[test]
fn a_three_sentence_figcaption_with_an_owning_image_becomes_exactly_one_caption_segment() {
    let root = temp_dir("caption-three-sentences");
    let img_url = format!("http://127.0.0.1:{}/anh.jpg", unreachable_port());
    let html = html_with_image_and_tail(
        &img_url,
        None,
        "<figcaption>Cau mot cua chu thich. Cau hai cua chu thich. Cau ba cua chu thich.</figcaption>",
    );
    let items = vec![url_item("https://example.test/bai-6", html)];
    let shape = chapters_shape_if_all_ok(&items).expect("danh sach toan muc OK");

    let opened = create_work(&root, "Caption ba cau", "en", "", shape, encoding_rs::UTF_8, Vec::new(), None, Vec::new(), 0, 1, false, &[], &Mutex::new(Vec::new()), None)
        .expect("tao Tac pham that bai");

    let rows = read_role_rows(&opened.store, opened.chapter_id);
    let caption_rows: Vec<_> = rows.iter().filter(|(_, r, _)| r.as_deref() == Some("caption")).collect();
    assert_eq!(
        caption_rows.len(),
        1,
        "mot khoi figcaption BA CAU van phai cho DUNG MOT segment role='caption' -- day la \
         doi chung do cho phep ep ranh gioi (Quyet dinh 1): go no thi ca nay do vi caption \
         se tach thanh BA hang khong hang nao mang vai: {rows:?}"
    );
    assert_eq!(caption_rows[0].2, "Cau mot cua chu thich. Cau hai cua chu thich. Cau ba cua chu thich.");

    drop(opened);
    cleanup(&root);
}

// ═════════════════════════════════════════════════════════════════════════════════
// I/O Matrix — caption không có ảnh trước nó ⇒ không gán vai, Chương vẫn nhập.
// ═════════════════════════════════════════════════════════════════════════════════

#[test]
fn a_caption_with_no_preceding_kept_image_gets_no_role_and_the_chapter_still_imports() {
    let root = temp_dir("caption-no-image-before");
    let html = "<html><head><title>Bai viet</title></head><body><article>\
        <figcaption>Chu thich mo dau, khong anh nao dung truoc no ca.</figcaption>\
        <p>Doan mot co du chu de duoc Readability chon lam noi dung chinh cua trang, \
        nhieu chu hon de vuot nguong do dai toi thieu.</p>\
        <p>Doan hai tiep tuc noi dung that su cua bai viet, khong phai menu hay quang cao, \
        du dai de dom_smoothie cham diem cao cho khoi nay.</p>\
        </article></body></html>"
        .to_owned();
    let items = vec![url_item("https://example.test/bai-7", html)];
    let shape = chapters_shape_if_all_ok(&items).expect("danh sach toan muc OK");

    let opened = create_work(&root, "Caption mo coi", "en", "", shape, encoding_rs::UTF_8, Vec::new(), None, Vec::new(), 0, 1, false, &[], &Mutex::new(Vec::new()), None)
        .expect("Chuong van phai nhap duoc du caption khong co anh so huu");

    let rows = read_role_rows(&opened.store, opened.chapter_id);
    assert!(rows.iter().all(|(_, r, _)| r.is_none()), "khong anh nao truoc no -- 0 hang mang vai: {rows:?}");
    assert!(
        rows.iter().any(|(_, _, t)| t.contains("Chu thich mo dau")),
        "noi dung caption van phai con trong Chuong, di tiep nhu van xuoi: {rows:?}"
    );

    drop(opened);
    cleanup(&root);
}

// ═════════════════════════════════════════════════════════════════════════════════
// I/O Matrix — Chương 0 ảnh (`.txt`/dán tay): `role` là `NULL` ở mọi hàng.
// ═════════════════════════════════════════════════════════════════════════════════

#[test]
fn a_plain_text_import_with_no_images_leaves_every_role_null() {
    let root = temp_dir("no-images-role-null");
    let opened = create_work_from_text(&root, "Khong anh", "zh", "", "一。二。\n三。".to_owned())
        .expect("tao tac pham that bai");

    let rows = read_role_rows(&opened.store, opened.chapter_id);
    assert_eq!(rows.len(), 3);
    assert!(rows.iter().all(|(_, r, _)| r.is_none()), "duong .txt/dan tay: moi hang role = NULL: {rows:?}");

    drop(opened);
    cleanup(&root);
}

// ═════════════════════════════════════════════════════════════════════════════════
// AC — xác nhận một segment vai 'caption' chuyển `confirmed` + đúng MỘT `segment_version`,
// y hệt một segment văn xuôi (ca đối chiếu cạnh nhau).
// ═════════════════════════════════════════════════════════════════════════════════

#[test]
fn confirming_a_caption_segment_behaves_exactly_like_confirming_a_prose_segment() {
    let root = temp_dir("confirm-caption-segment");
    let img_url = format!("http://127.0.0.1:{}/anh.jpg", unreachable_port());
    let html = html_with_image_and_tail(&img_url, None, "<figcaption>Chu thich can dich.</figcaption>");
    let items = vec![url_item("https://example.test/bai-8", html)];
    let shape = chapters_shape_if_all_ok(&items).expect("danh sach toan muc OK");

    let opened = create_work(&root, "Xac nhan caption", "en", "", shape, encoding_rs::UTF_8, Vec::new(), None, Vec::new(), 0, 1, false, &[], &Mutex::new(Vec::new()), None)
        .expect("tao Tac pham that bai");

    let (caption_id, prose_id): (i64, i64) = opened
        .store
        .read(move |conn| {
            let caption: i64 = conn.query_row(
                "SELECT id FROM segment WHERE chapter_id = ?1 AND role = 'caption'",
                [opened.chapter_id],
                |r| r.get(0),
            )?;
            let prose: i64 = conn.query_row(
                "SELECT id FROM segment WHERE chapter_id = ?1 AND role IS NULL ORDER BY ord LIMIT 1",
                [opened.chapter_id],
                |r| r.get(0),
            )?;
            Ok((caption, prose))
        })
        .expect("doc id segment that bai");

    let chapter_id = opened.chapter_id;
    save_segment_targets(
        Some(&opened),
        chapter_id,
        &[
            SegmentTargetEdit { id: caption_id, target_text: "Ban dich chu thich.".to_owned() },
            SegmentTargetEdit { id: prose_id, target_text: "Ban dich van xuoi.".to_owned() },
        ],
    )
    .expect("ghi ban dich that bai");

    let caption_outcome = confirm_segment(Some(&opened), caption_id, "").expect("xac nhan caption that bai");
    let prose_outcome = confirm_segment(Some(&opened), prose_id, "").expect("xac nhan van xuoi that bai");

    assert_eq!(caption_outcome.status, "confirmed");
    assert!(caption_outcome.version_created);
    assert_eq!(prose_outcome.status, "confirmed");
    assert!(prose_outcome.version_created);

    let (caption_status, caption_versions, caption_role): (String, i64, Option<String>) = opened
        .store
        .read(move |conn| {
            let status: String =
                conn.query_row("SELECT status FROM segment WHERE id = ?1", [caption_id], |r| r.get(0))?;
            let versions: i64 = conn.query_row(
                "SELECT COUNT(*) FROM segment_version WHERE segment_id = ?1",
                [caption_id],
                |r| r.get(0),
            )?;
            let role: Option<String> =
                conn.query_row("SELECT role FROM segment WHERE id = ?1", [caption_id], |r| r.get(0))?;
            Ok((status, versions, role))
        })
        .expect("doc trang thai caption that bai");
    assert_eq!(caption_status, "confirmed");
    assert_eq!(caption_versions, 1, "dung MOT segment_version, y het mot segment van xuoi");
    assert_eq!(caption_role.as_deref(), Some("caption"), "xac nhan KHONG duoc lam mat vai da gan");

    let (prose_status, prose_versions): (String, i64) = opened
        .store
        .read(move |conn| {
            let status: String =
                conn.query_row("SELECT status FROM segment WHERE id = ?1", [prose_id], |r| r.get(0))?;
            let versions: i64 = conn.query_row(
                "SELECT COUNT(*) FROM segment_version WHERE segment_id = ?1",
                [prose_id],
                |r| r.get(0),
            )?;
            Ok((status, versions))
        })
        .expect("doc trang thai van xuoi that bai");
    assert_eq!(prose_status, caption_status, "hai duong phai hoi tu ve cung mot trang thai");
    assert_eq!(prose_versions, caption_versions, "hai duong phai sinh cung so segment_version");

    drop(opened);
    cleanup(&root);
}

// ═════════════════════════════════════════════════════════════════════════════════
// AC — một `project.db` ở phiên bản 20 (trước story này) mở được, di trú lên 21, mọi hàng cũ
// mang `role = NULL`.
// ═════════════════════════════════════════════════════════════════════════════════

#[test]
fn a_version_20_project_database_migrates_to_21_and_every_existing_row_gets_role_null() {
    // Đúng MƯỜI CHÍN bước THẬT của `PROJECT_MIGRATIONS` trước bước 21 (role) -- cùng khuôn
    // `segment_contract.rs::a_project_database_stranded_at_the_burned_version_four_...`:
    // dùng lát cắt của CHÍNH hằng thật, không một fixture chép tay sẽ trôi khỏi sự thật.
    static OLD_STEPS: [Migration; 19] = [
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
    ];

    let root = temp_dir("migrate-20-to-21");
    let db = root.join("project.db");

    let old = Store::open(StoreSpec { migrations: &OLD_STEPS, ..StoreSpec::project(db.clone()) })
        .expect("dung fixture o phien ban 20");
    assert_eq!(old.schema_version(), 20, "fixture phai dung o dung phien ban 20 -- khong thi ca nay khong kiem gi ca");

    // Mot hang segment THAT, ghi truoc khi cot `role` ton tai -- dung DDL cua chinh phien ban
    // 20 (khong the nhac cot `role`, no chua sinh ra).
    let chapter_id: i64 = old
        .write(|tx: &Transaction<'_>| {
            tx.execute(
                "INSERT INTO work (id, work_id, name, source_lang, genre, created_at, updated_at) \
                 VALUES (1, 'w', 'Fixture v20', 'zh', '', strftime('%Y-%m-%dT%H:%M:%fZ','now'), \
                 strftime('%Y-%m-%dT%H:%M:%fZ','now'))",
                (),
            )?;
            tx.execute(
                "INSERT INTO chapter (ord, title, source_text, status, created_at, updated_at) \
                 VALUES (1, NULL, '一。', 'not_started', strftime('%Y-%m-%dT%H:%M:%fZ','now'), \
                 strftime('%Y-%m-%dT%H:%M:%fZ','now'))",
                (),
            )?;
            let chapter_id = tx.last_insert_rowid();
            tx.execute(
                "INSERT INTO segment (chapter_id, ord, source_text, is_paragraph_end, \
                 is_target_paragraph_end, translation_origin, created_at, updated_at) \
                 VALUES (?1, 1, '一。', 0, 0, '', strftime('%Y-%m-%dT%H:%M:%fZ','now'), \
                 strftime('%Y-%m-%dT%H:%M:%fZ','now'))",
                [chapter_id],
            )?;
            Ok(chapter_id)
        })
        .expect("ghi fixture v20 that bai");
    drop(old);

    // Day la dong menh de THAT: mo lai bang bo di tru DAY DU (dich 22 -- Story 6.15 them buoc
    // 22 sau story nay, `Store::open` luon di tru toi dich MOI NHAT cua PROJECT_MIGRATIONS).
    let migrated = Store::open(StoreSpec::project(db)).expect("mot project.db o phien ban 20 phai mo duoc");
    assert_eq!(migrated.schema_version(), 22, "di tru phai chay het toi dich moi nhat (qua ca buoc 21 segment.role)");

    let role: Option<String> = migrated
        .read(move |conn| conn.query_row("SELECT role FROM segment WHERE chapter_id = ?1", [chapter_id], |r| r.get(0)))
        .expect("doc role that bai");
    assert_eq!(role, None, "hang cu tao truoc story nay phai mang role = NULL sau di tru -- khong hang nao bi viet lai noi dung");

    drop(migrated);
    cleanup(&root);
}

// ═════════════════════════════════════════════════════════════════════════════════
// I/O Matrix hàng "Gộp/tách một segment vai" — vai KHÔNG nhân bản sang hàng MỚI.
//
// 🔴 **VÌ SAO CA NÀY PHẢI SỐNG Ở ĐÂY, DÙ `segment_contract.rs` ĐÃ CÓ MỘT DÒNG `None` CHO
// `role`.** Ca `a_row_born_from_regroup_has_every_column_set_on_purpose_not_by_default` dựng
// Tác phẩm bằng `create_work_from_text` — một Chương **0 ảnh**, nên CẢ HAI câu bị gộp vốn đã
// mang `role = NULL`. Khẳng định "hàng mới mang `role = NULL`" ở đó vì thế đúng ở **CẢ HAI**
// nhánh: nó xanh y hệt nếu `write_regroup` CÓ chép vai sang hàng mới, vì không có vai nào để
// mà chép. Đo 2026-09-09 (vòng nghiệm thu bước 3): đó là một assert không canh nhánh nào —
// đúng lớp lỗi mà `AGENTS.md` §Known pitfalls gọi tên ("một bộ test xanh KHÔNG chứng minh chỗ
// nối mới được canh").
//
// Ca này gộp một segment **THẬT SỰ mang `role='caption'`** với câu liền trên nó. Nếu câu
// `INSERT` của `write_regroup` chép `role` từ hàng nguồn, hàng mới sẽ mang `'caption'` và ca
// này ĐỎ — đó là điều làm nó khác ca kia.
// ═════════════════════════════════════════════════════════════════════════════════

#[test]
fn merging_a_caption_segment_gives_a_new_row_with_no_role_at_all() {
    let root = temp_dir("merge-vai-khong-nhan-ban");
    let img_url = format!("http://127.0.0.1:{}/anh.jpg", unreachable_port());
    let html = html_with_image_and_tail(
        &img_url,
        Some("mo ta anh minh hoa"),
        "<figcaption>Chu thich cho anh nay.</figcaption>",
    );
    let items = vec![url_item("https://example.test/bai-gop-vai", html)];
    let shape = chapters_shape_if_all_ok(&items).expect("danh sach toan muc OK");

    let opened = create_work(&root, "Gop segment vai", "en", "", shape, encoding_rs::UTF_8, Vec::new(), None, Vec::new(), 0, 1, false, &[], &Mutex::new(Vec::new()), None)
        .expect("tao Tac pham that bai");

    // ── Tien de cua ca nay, khang dinh TRUOC khi gop ────────────────────────────────
    // Khong co dong nay, mot thay doi tuong lai lam caption thoi mang vai se bien ca nay
    // thanh mot phep gop hai cau van xuoi -- xanh, va khong canh gi (dung cai bay ma ca o
    // `segment_contract.rs` da dinh).
    let chapter_id = opened.chapter_id;
    let truoc = read_role_rows(&opened.store, chapter_id);
    let (cap_ord, _, cap_text) = truoc
        .iter()
        .find(|(_, r, _)| r.as_deref() == Some("caption"))
        .cloned()
        .expect("phai co mot hang role='caption' TRUOC khi gop -- day la tien de cua ca nay");
    assert!(cap_ord > 1, "hang caption phai co mot cau dung TREN no de ma gop vao");

    // `merge_segments` nhan `segment_id`, khong phai `ord` -- doc id that cua hang caption.
    let cap_id: i64 = opened
        .store
        .read(move |conn| {
            conn.query_row(
                "SELECT id FROM segment WHERE chapter_id = ?1 AND role = 'caption' AND retired_at IS NULL",
                [chapter_id],
                |r| r.get(0),
            )
        })
        .expect("doc id cua hang caption that bai");

    let out = merge_segments(Some(&opened), cap_id).expect("gop hang caption voi cau lien tren no");
    assert_eq!(out.new_segments.len(), 1, "mot luot gop sinh dung MOT hang moi");
    let moi_id = out.new_segments[0].id;
    // 🔴 Doc TRUC TIEP qua DTO `ChapterSegment` (day IPC that su, `read_fresh_rows` cot thu
    // 10, `row.get(9)`) -- khong chi doc lai bang SQL tho nhu ben duoi. Neu chi so cot lech
    // (mot cot moi chen VAO GIUA thay vi noi cuoi cau SELECT), phep doc THO co the vo tinh
    // van dung nham cot khac ma van ra None -- doc qua DTO la cho DUY NHAT do dung hinh dang
    // day that.
    assert_eq!(
        out.new_segments[0].role, None,
        "DTO `ChapterSegment.role` cua hang MOI (tu `RegroupOutcome::new_segments`) phai la \
         None -- day la phep do TRUC TIEP qua day IPC, khong qua SQL tho"
    );

    // ── Menh de duoc do ────────────────────────────────────────────────────────────
    let sau = read_role_rows(&opened.store, chapter_id);
    let moi = sau
        .iter()
        .find(|(ord, _, _)| *ord == cap_ord - 1)
        .expect("hang moi chiem `ord` cua cau DAU nhom (AD-5: ve huu + tao moi)");
    assert!(
        moi.2.contains(&cap_text),
        "hang moi phai chua van ban cua caption da gop vao: {moi:?}"
    );
    assert_eq!(
        moi.1, None,
        "🔴 vai KHONG nhan ban: mot hang MOI sinh tu gop/tach (AD-5 ve huu + tao moi) khong \
         thua ke `role` cua hang nguon. Ca nay DO neu `write_regroup` chep cot do -- va do la \
         phep do ma ca o `segment_contract.rs` (Chuong 0 anh) khong lam duoc."
    );
    assert_eq!(
        sau.iter().filter(|(_, r, _)| r.as_deref() == Some("caption")).count(),
        0,
        "sau luot gop, khong hang SONG nao con mang vai caption: {sau:?}"
    );

    // 🔵 **SUA 2026-09-09, sau khi DO thay vi doan** — ban dau ca nay khang dinh "hang alt giu
    // nguyen vai", tren gia dinh rang cau dung TREN caption la mot cau van xuoi. Luot chay dau
    // tien bac gia dinh do: `alt` va `caption` cua CUNG mot anh nam LIEN NHAU (alt tai neo,
    // caption ngay sau -- dung AD-42), nen `merge_segments` tren hang caption gop no voi hang
    // ALT. Ket qua do duoc con MANH hon menh de ban dau: CA HAI hang nguon deu mang vai, va
    // hang moi van `role = NULL` -- neu `write_regroup` chep vai, no da phai chep MOT trong
    // HAI vai do, va ca nay DO.
    assert_eq!(
        sau.iter().filter(|(_, r, _)| r.as_deref() == Some("alt")).count(),
        0,
        "luot gop nay nuot CA hang alt lan hang caption (chung lien nhau theo AD-42), nen \
         khong hang SONG nao con mang vai: {sau:?}"
    );
    assert!(
        moi.2.contains("mo ta anh minh hoa"),
        "hang moi phai chua ca van ban cua hang alt da bi gop vao: {moi:?}"
    );

    let _ = moi_id;
    drop(opened);
    cleanup(&root);
}

// ═════════════════════════════════════════════════════════════════════════════════
// Vòng rà 2026-09-10 — AD-42 "nhiều nhất một segment mỗi vai" áp cho CẢ khối `Caption` thứ
// hai trở đi của CÙNG một ảnh (bắt được: hai `<figcaption>` liên tiếp từng cho HAI hàng
// `role='caption'`).
// ═════════════════════════════════════════════════════════════════════════════════

#[test]
fn two_adjacent_figcaptions_for_one_image_produce_exactly_one_caption_role_row() {
    let root = temp_dir("two-figcaptions-one-image");
    let img_url = format!("http://127.0.0.1:{}/anh.jpg", unreachable_port());
    let html = html_with_image_and_tail(
        &img_url,
        None,
        "<figcaption>Chu thich thu nhat.</figcaption><figcaption>Chu thich thu hai.</figcaption>",
    );
    let items = vec![url_item("https://example.test/bai-hai-figcaption", html)];
    let shape = chapters_shape_if_all_ok(&items).expect("danh sach toan muc OK");

    let opened = create_work(&root, "Hai figcaption mot anh", "en", "", shape, encoding_rs::UTF_8, Vec::new(), None, Vec::new(), 0, 1, false, &[], &Mutex::new(Vec::new()), None)
        .expect("tao Tac pham that bai");

    let rows = read_role_rows(&opened.store, opened.chapter_id);
    let caption_rows: Vec<_> = rows.iter().filter(|(_, r, _)| r.as_deref() == Some("caption")).collect();
    assert_eq!(
        caption_rows.len(),
        1,
        "AD-42: nhieu nhat MOT segment role='caption' cho MOT anh, du trang co HAI \
         <figcaption> lien tiep: {rows:?}"
    );
    assert_eq!(caption_rows[0].2, "Chu thich thu nhat.", "khoi DAU TIEN moi duoc chon mang vai");
    assert!(
        rows.iter().any(|(_, r, t)| t == "Chu thich thu hai." && r.is_none()),
        "khoi THU HAI phai di tiep NHU VAN XUOI (role = NULL), khong bi gan vai rieng: {rows:?}"
    );

    drop(opened);
    cleanup(&root);
}

// ═════════════════════════════════════════════════════════════════════════════════
// Vòng rà 2026-09-10 — một `CleanupRule` xoá TRẮNG văn bản caption không được để đoạn văn
// xuôi kế tiếp thừa hưởng `role='caption'` theo chỉ số.
// ═════════════════════════════════════════════════════════════════════════════════

#[test]
fn a_cleanup_rule_erasing_the_caption_text_tags_no_row_and_leaves_the_next_prose_row_bare() {
    let root = temp_dir("cleanup-erases-caption");
    let img_url = format!("http://127.0.0.1:{}/anh.jpg", unreachable_port());
    let caption_text = "Chu thich se bi mot luat lam sach xoa trang.";
    let html = html_with_image_and_tail(
        &img_url,
        None,
        &format!("<figcaption>{caption_text}</figcaption>"),
    );
    let items = vec![url_item("https://example.test/bai-cleanup-xoa-caption", html)];
    let shape = chapters_shape_if_all_ok(&items).expect("danh sach toan muc OK");

    let rules = vec![CleanupRule {
        tier: CleanupRuleTier::Global,
        id: 1,
        pattern: caption_text.to_owned(),
        kind: CleanupRuleKind::Literal,
        enabled: true,
    }];

    let opened = create_work(&root, "Cleanup xoa caption", "en", "", shape, encoding_rs::UTF_8, rules, None, Vec::new(), 0, 1, false, &[], &Mutex::new(Vec::new()), None)
        .expect("tao Tac pham that bai");

    let rows = read_role_rows(&opened.store, opened.chapter_id);
    assert!(
        !rows.iter().any(|(_, _, t)| t.contains("Chu thich")),
        "van ban caption phai THAT SU bi luat lam sach xoa trang, khong con segment nao mang \
         no: {rows:?}"
    );
    assert_eq!(
        rows.iter().filter(|(_, r, _)| r.as_deref() == Some("caption")).count(),
        0,
        "caption da bi xoa trang -- 0 hang role='caption': {rows:?}"
    );
    let prose = rows
        .iter()
        .find(|(_, _, t)| t.contains("Doan ba dong y nghia"))
        .expect("doan van xuoi sau caption phai con nguyen");
    assert_eq!(
        prose.1, None,
        "doan van xuoi KE TIEP caption da bi xoa khong duoc thua huong role='caption' theo \
         CHI SO: {rows:?}"
    );

    drop(opened);
    cleanup(&root);
}

// ═════════════════════════════════════════════════════════════════════════════════
// I/O Matrix "Gộp/tách một segment vai" — chiều TÁCH: hàng mới cũng phải mang role = NULL.
// ═════════════════════════════════════════════════════════════════════════════════

#[test]
fn splitting_a_role_bearing_segment_gives_new_rows_with_no_role_at_all() {
    let root = temp_dir("split-vai-khong-nhan-ban");
    let img_url = format!("http://127.0.0.1:{}/anh.jpg", unreachable_port());
    let caption_text = "Chu thich se bi tach lam doi.";
    let html = html_with_image_and_tail(&img_url, None, &format!("<figcaption>{caption_text}</figcaption>"));
    let items = vec![url_item("https://example.test/bai-tach-vai", html)];
    let shape = chapters_shape_if_all_ok(&items).expect("danh sach toan muc OK");

    let opened = create_work(&root, "Tach segment vai", "en", "", shape, encoding_rs::UTF_8, Vec::new(), None, Vec::new(), 0, 1, false, &[], &Mutex::new(Vec::new()), None)
        .expect("tao Tac pham that bai");

    let chapter_id = opened.chapter_id;
    let cap_id: i64 = opened
        .store
        .read(move |conn| {
            conn.query_row(
                "SELECT id FROM segment WHERE chapter_id = ?1 AND role = 'caption' AND retired_at IS NULL",
                [chapter_id],
                |r| r.get(0),
            )
        })
        .expect("doc id cua hang caption that bai");

    let cut = caption_text.chars().count() / 2;
    let out = split_segment(Some(&opened), cap_id, vec![cut]).expect("tach hang caption lam doi");
    assert_eq!(out.new_segments.len(), 2, "tach mot diem cat sinh dung HAI manh");
    for piece in &out.new_segments {
        assert_eq!(
            piece.role, None,
            "🔴 vai KHONG nhan ban: hang MOI sinh tu tach (AD-5 ve huu + tao moi) khong duoc \
             thua ke `role` cua hang nguon: {piece:?}"
        );
    }

    let rows = read_role_rows(&opened.store, chapter_id);
    assert_eq!(
        rows.iter().filter(|(_, r, _)| r.as_deref() == Some("caption")).count(),
        0,
        "sau luot tach, khong hang SONG nao con mang vai caption: {rows:?}"
    );

    drop(opened);
    cleanup(&root);
}

// ═════════════════════════════════════════════════════════════════════════════════
// `block_overrides` chỉ áp cho Chương ĐẦU TIÊN trên đường dệt MỚI (role.rs) — một lượt nhập
// URL nhiều Chương không được để override của Chương 0 rò sang Chương khác.
// ═════════════════════════════════════════════════════════════════════════════════

#[test]
fn block_overrides_only_apply_to_chapter_zero_on_the_weave_path() {
    let root = temp_dir("block-overrides-chapter-zero-only");
    let img_url_a = format!("http://127.0.0.1:{}/a.jpg", unreachable_port());
    let img_url_b = format!("http://127.0.0.1:{}/b.jpg", unreachable_port());
    // Hai Chuong CUNG mot hinh dang khoi (h1 + 2 doan + anh + 1 doan) -- anh o CUNG chi so
    // khoi (3) o ca hai Chuong, chi khac URL/alt.
    let html_a = html_with_image_and_tail(&img_url_a, Some("mo ta A"), "");
    let html_b = html_with_image_and_tail(&img_url_b, Some("mo ta B"), "");
    let items = vec![
        url_item("https://example.test/nhieu-chuong-1", html_a),
        url_item("https://example.test/nhieu-chuong-2", html_b),
    ];
    let shape = chapters_shape_if_all_ok(&items).expect("danh sach toan muc OK");

    // EP LOAI anh o chi so khoi 3 -- CHI cho Chuong 0 (block_overrides khong mang chi so
    // Chuong, chi ap dung cho don vi DAU TIEN theo dung luat da co tu Story 6.9).
    let overrides = vec![None, None, None, Some(false)];

    let opened = create_work(&root, "Nhieu chuong block_overrides", "en", "", shape, encoding_rs::UTF_8, Vec::new(), None, overrides, 0, 1, false, &[], &Mutex::new(Vec::new()), None)
        .expect("tao Tac pham that bai");

    let chapter_a_id = opened.chapter_id;
    let chapter_b_id: i64 = opened
        .store
        .read(|conn| conn.query_row("SELECT id FROM chapter WHERE ord = 2", [], |r| r.get(0)))
        .expect("doc id Chuong thu hai that bai");

    let rows_a = read_role_rows(&opened.store, chapter_a_id);
    assert!(
        rows_a.iter().all(|(_, r, _)| r.as_deref() != Some("alt")),
        "Chuong 0: anh bi EP LOAI boi block_overrides -- 0 hang role='alt': {rows_a:?}"
    );

    let rows_b = read_role_rows(&opened.store, chapter_b_id);
    let alt_rows_b: Vec<_> = rows_b.iter().filter(|(_, r, _)| r.as_deref() == Some("alt")).collect();
    assert_eq!(
        alt_rows_b.len(),
        1,
        "Chuong 1: block_overrides cua Chuong 0 KHONG duoc ro sang -- anh cua Chuong nay van \
         GIU binh thuong (machine_kept mac dinh) va phai co dung MOT hang role='alt': {rows_b:?}"
    );
    assert_eq!(alt_rows_b[0].2, "mo ta B");

    drop(opened);
    cleanup(&root);
}
