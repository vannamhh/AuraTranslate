//! Cổng HỢP ĐỒNG của Story 6.11 (FR127) — ảnh tải về `.atproj`, neo vị trí, `source_url`.
//!
//! Đi ĐÚNG con đường sản phẩm: `chapters_shape_if_all_ok` (Story 6.7) → `create_work` — cùng
//! hai hàm mà `commands::project::wire::start_url_import`/`confirm_import_with_encoding` gọi
//! (khuôn `project_contract.rs::n_chapters_from_a_url_list_write_clean_text_ord_and_segments_for_every_chapter`).
//! Server ảnh là một TCP thô tự dựng — không mạng ngoài, cùng khuôn `webimport_contract.rs::spawn_once`.

use std::fs;
use std::io::{Read, Write};
use std::net::TcpListener;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use auratranslate_lib::commands::project::{UrlImportItem, chapters_shape_if_all_ok, create_work, create_work_from_text};
use auratranslate_lib::core::cleanup::{CleanupRule, CleanupRuleKind, CleanupRuleTier};
// D4 (vòng rà đối kháng 3 lớp) — `MessageKey` chỉ được dùng bên trong ca `#[cfg(unix)]`
// `a_disk_write_failure_mid_asset_write_fails_the_whole_import_and_removes_the_atproj_folder`
// (quyền thư mục kiểu Unix không có ý nghĩa trên Windows). Đặt `#[cfg(unix)]` NGAY trên dòng
// `use` này, không để nó vô điều kiện — bản trước gây cảnh báo unused-import trên runner
// Windows CI (ca đó biên dịch ra rỗng ở đó, nhưng `use` vẫn đứng).
#[cfg(unix)]
use auratranslate_lib::core::i18n::MessageKey;
use auratranslate_lib::core::store::Transaction;
use auratranslate_lib::core::webimport::{DomainLogDecision, DomainLogOutcome, DomainLogState, Tier};

static NEXT_DIR: AtomicU64 = AtomicU64::new(0);

/// Một thư mục tạm CỦA RIÊNG ca này — cùng khuôn `project_contract.rs`.
fn temp_dir(tag: &str) -> PathBuf {
    let n = NEXT_DIR.fetch_add(1, Ordering::Relaxed);
    let dir = std::env::temp_dir().join(format!("auratranslate-asset-{}-{}-{}", std::process::id(), tag, n));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap_or_else(|e| panic!("tao {}: {e}", dir.display()));
    dir
}

fn cleanup(dir: &Path) {
    let _ = fs::remove_dir_all(dir);
}

/// Trang bài viết CÓ MỘT ảnh giữa đoạn hai và đoạn ba — ba đoạn văn ĐÚNG NGUYÊN VĂN
/// `webimport_contract.rs::html_page_with_paragraphs`, mỗi đoạn ĐÚNG MỘT câu (một dấu chấm)
/// để mỗi đoạn tách thành ĐÚNG một segment — điều kiện để `anchor_after_segment_ord` đếm
/// được bằng tay: `<h1>` + hai đoạn đứng TRƯỚC ảnh, mỗi khối một segment ⇒ neo = **3** (đo
/// thật, không suy — `<h1>` cũng là một `TEXT_BLOCK_TAGS`, xem `extractor.rs`).
fn html_page_with_one_image(img_src: &str) -> String {
    format!(
        "<html><head><title>Bai viet</title></head><body><article><h1>Tieu de</h1>\
         <p>Doan mot co du chu de duoc Readability chon lam noi dung chinh cua trang, \
         nhieu chu hon de vuot nguong do dai toi thieu.</p>\
         <p>Doan hai tiep tuc noi dung that su cua bai viet, khong phai menu hay quang cao, \
         du dai de dom_smoothie cham diem cao cho khoi nay.</p>\
         <img src=\"{img_src}\">\
         <p>Doan ba dong y nghia, giu cho tong do dai van ban vuot qua nguong toi thieu can \
         thiet de Readability tin day la mot bai viet that.</p>\
         </article></body></html>"
    )
}

/// Server tối giản, chấp nhận TỐI ĐA `max_connections` kết nối rồi tự thoát luồng — đủ để một
/// ca "dedup hỏng, gọi mạng lần hai" TRẢ VỀ NGAY (đếm được `2`) thay vì TREO tới
/// `REQUEST_TIMEOUT` (20 s, `fetcher.rs`) vì không ai lắng nghe kết nối thứ hai.
/// 🔵 **SỬA 2026-09-08 (mục C6 vòng rà đối kháng 3 lớp) — đo được: bộ đếm CŨ đếm KẾT NỐI
/// TCP, không đếm YÊU CẦU HTTP.** `shared_client()` (`fetcher.rs`) dùng một pool giữ-nối
/// (`reqwest` mặc định keep-alive): nếu dedup của `prepare_chapter_images` hỏng và gọi
/// `fetch` HAI lần cho CÙNG một URL, cả hai yêu cầu HOÀN TOÀN CÓ THỂ đi qua đúng MỘT kết nối
/// TCP được pool tái dùng — bộ đếm theo `accept()` khi đó vẫn báo `1`, che mất một lượt tải
/// TRÙNG thật sự. **Đã THỬ** đếm theo yêu cầu HTTP đọc được (ranh giới `\r\n\r\n`) trên một
/// kết nối keep-alive còn mở — đo THẬT: gây treo/`WouldBlock` khó tái lập trên
/// `reqwest::blocking` (client dùng luồng nền + runtime tokio riêng, `fetcher.rs::TASK 0`) mà
/// không đáng chi phí gỡ ở đây. **Đường CHỌN**: buộc máy chủ trả `Connection: close` — client
/// (mọi HTTP client tuân thủ) đóng kết nối SAU MỖI phản hồi thay vì trả nó về pool, nên một
/// kết nối TCP MỚI (đếm được bằng `accept()`) là hệ quả TẤT ĐỊNH của một yêu cầu HTTP mới,
/// không còn khoảng hở "hai yêu cầu, một kết nối" nữa — cùng con số, cơ chế đơn giản hơn hẳn.
fn spawn_counting_image_server(
    max_connections: usize,
    body: &'static [u8],
    content_type: &'static str,
) -> (u16, Arc<AtomicUsize>, thread::JoinHandle<()>) {
    let counter = Arc::new(AtomicUsize::new(0));
    let counter_clone = Arc::clone(&counter);
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind cong tam");
    let port = listener.local_addr().expect("local_addr").port();
    let handle = thread::spawn(move || {
        for _ in 0..max_connections {
            let Ok((mut stream, _)) = listener.accept() else { break };
            counter_clone.fetch_add(1, Ordering::SeqCst);
            let _ = stream.set_read_timeout(Some(Duration::from_secs(5)));
            let mut discard = [0u8; 4096];
            let _ = stream.read(&mut discard);
            // `Connection: close` — xem doc-comment hàm này (mục C6): buộc client đóng kết
            // nối này thay vì trả về pool giữ-nối, để `accept()` (đếm ở trên) tương ứng ĐÚNG
            // một-một với một yêu cầu HTTP thật, không bị pool che một lượt tải TRÙNG.
            let header = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: {content_type}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
                body.len()
            );
            let _ = stream.write_all(header.as_bytes());
            let _ = stream.write_all(body);
        }
    });
    (port, counter, handle)
}

/// Một cổng KHÔNG AI LẮNG NGHE — `bind` rồi `drop` NGAY để giải phóng cổng, mô phỏng tất định
/// "kết nối bị từ chối" trên loopback mà không cần một máy chủ giả tự đóng cổng giữa chừng.
fn unreachable_port() -> u16 {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind cong tam");
    listener.local_addr().expect("local_addr").port()
}

fn url_item(url: &str, html: String) -> UrlImportItem {
    UrlImportItem { url: url.to_owned(), raw: Some(html.into_bytes()), error: None }
}

type AssetRow = (i64, String, Option<String>, i64, i64, String);

fn read_asset_rows(store: &auratranslate_lib::core::store::Store) -> Vec<AssetRow> {
    store
        .read(|conn| {
            let mut stmt = conn.prepare(
                "SELECT chapter_id, file_name, source_url, anchor_after_segment_ord, byte_len, \
                 content_type FROM asset ORDER BY id",
            )?;
            let mut rows = stmt.query([])?;
            let mut out = Vec::new();
            while let Some(row) = rows.next()? {
                out.push((
                    row.get::<_, i64>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, Option<String>>(2)?,
                    row.get::<_, i64>(3)?,
                    row.get::<_, i64>(4)?,
                    row.get::<_, String>(5)?,
                ));
            }
            Ok(out)
        })
        .expect("doc bang asset that bai")
}

// ═════════════════════════════════════════════════════════════════════════════════
// Ảnh GIỮ, host tầng 2, MIME raster — một tệp thật + một hàng `asset` mang neo đúng
// ═════════════════════════════════════════════════════════════════════════════════

#[test]
fn a_kept_image_gets_a_real_file_on_disk_and_an_asset_row_with_the_right_anchor() {
    let root = temp_dir("kept-image-anchor");
    let body: &'static [u8] = b"fake-jpeg-bytes-0123456789";
    let (port, counter, _handle) = spawn_counting_image_server(4, body, "image/jpeg; charset=binary");
    let img_url = format!("http://127.0.0.1:{port}/anh.jpg");

    let items = vec![url_item("https://example.test/bai-mot", html_page_with_one_image(&img_url))];
    let shape = chapters_shape_if_all_ok(&items).expect("danh sach toan muc OK");

    let opened = create_work(&root, "Anh Giua Bai", "en", "", shape, encoding_rs::UTF_8, Vec::new(), None, Vec::new(), &std::sync::Mutex::new(Vec::new()), None)
        .expect("tao Tac pham voi anh that bai");

    assert_eq!(opened.images_saved, 1, "dung mot anh GIU trong fixture");
    assert_eq!(opened.images_failed, 0);
    assert_eq!(counter.load(Ordering::SeqCst), 1, "dung MOT lan tai — khong tai trung");

    let rows = read_asset_rows(&opened.store);
    assert_eq!(rows.len(), 1, "dung mot hang asset: {rows:?}");
    let (chapter_id, file_name, source_url, anchor, byte_len, content_type) = &rows[0];
    assert_eq!(*chapter_id, opened.chapter_id, "hang asset phai thuoc Chuong duy nhat vua tao");
    assert!(file_name.ends_with(".jpg"), "duoi tep phai theo MIME phan hoi (.jpg): {file_name}");
    assert_eq!(source_url.as_deref(), Some(img_url.as_str()));
    assert_eq!(*anchor, 3, "<h1> + hai doan van dau phai dung TRUOC anh — neo = 3");
    assert_eq!(*byte_len, body.len() as i64);
    assert_eq!(content_type, "image/jpeg", "content_type ghi xuong phai la MIME DA CHUAN HOA");

    let on_disk = opened.dir.join("assets").join(file_name);
    let written = fs::read(&on_disk).unwrap_or_else(|e| panic!("doc tep vua ghi {}: {e}", on_disk.display()));
    assert_eq!(written, body, "byte tren dia phai TRUNG DUNG than anh may chu tra ve");

    // §Never spec 6.11 — không đụng `tauri.conf.json`/`capabilities`, cùng phép đo cho §Never:
    // không đường dẫn TUYỆT ĐỐI nào bị ghi vào trong project.db (chỉ `file_name`, tương đối).
    assert!(!file_name.contains(std::path::MAIN_SEPARATOR), "file_name phai la TEN TUONG DOI, khong duong dan");

    drop(opened);
    cleanup(&root);
}

#[test]
fn an_image_without_alt_still_keeps_its_position() {
    let root = temp_dir("kept-image-no-alt");
    let body: &'static [u8] = b"no-alt-body";
    let (port, _counter, _handle) = spawn_counting_image_server(4, body, "image/png");
    let img_url = format!("http://127.0.0.1:{port}/anh.png");
    // Fixture dùng chung `html_page_with_one_image` — `<img>` KHÔNG mang `alt` (đúng ca I/O
    // Matrix "Ảnh không có alt").
    let items = vec![url_item("https://example.test/bai-hai", html_page_with_one_image(&img_url))];
    let shape = chapters_shape_if_all_ok(&items).expect("danh sach toan muc OK");

    let opened = create_work(&root, "Anh Khong Alt", "en", "", shape, encoding_rs::UTF_8, Vec::new(), None, Vec::new(), &std::sync::Mutex::new(Vec::new()), None)
        .expect("tao Tac pham that bai");

    assert_eq!(opened.images_saved, 1);
    let rows = read_asset_rows(&opened.store);
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].3, 3, "thieu alt khong duoc doi neo — van la 3");

    drop(opened);
    cleanup(&root);
}

// ═════════════════════════════════════════════════════════════════════════════════
// MIME không phải ảnh raster (kể cả SVG) — bỏ ảnh, KHÔNG ghi tệp, KHÔNG hàng `asset`
// ═════════════════════════════════════════════════════════════════════════════════

#[test]
fn an_svg_response_is_rejected_writes_no_file_and_the_chapter_text_survives_intact() {
    let root = temp_dir("svg-rejected");
    let svg_body: &'static [u8] = b"<svg xmlns='http://www.w3.org/2000/svg'><script>evil()</script></svg>";
    let (port, _counter, _handle) = spawn_counting_image_server(4, svg_body, "image/svg+xml");
    let img_url = format!("http://127.0.0.1:{port}/anh.svg");

    let items = vec![url_item("https://example.test/bai-svg", html_page_with_one_image(&img_url))];
    let shape = chapters_shape_if_all_ok(&items).expect("danh sach toan muc OK");

    // A1 (vòng rà đối kháng 2, 3 lớp) — dùng một DomainLogState CỤC BỘ ĐỌC LẠI ĐƯỢC, thay vì
    // một Mutex thùng rác không bao giờ đọc lại: gỡ hẳn khối gán `MimeRejected` ở
    // `project.rs` vẫn để `outcome` giữ nguyên `Fetched` từ `fetcher.rs` mà KHÔNG một ca nào
    // trước đây bắt được — bảng Quyền riêng tư khi đó khai "Tải thành công" cho một ảnh SVG
    // đã bị TỪ CHỐI.
    let domain_log_state: DomainLogState = Mutex::new(Vec::new());
    let opened = create_work(&root, "Anh SVG", "en", "", shape, encoding_rs::UTF_8, Vec::new(), None, Vec::new(), &domain_log_state, None)
        .expect("mot anh SVG bi tu choi khong duoc lam trot ca luot nhap");

    assert_eq!(opened.images_saved, 0, "SVG la danh dau, khong phai anh raster — phai bi TU CHOI");
    assert_eq!(opened.images_failed, 1);
    assert!(read_asset_rows(&opened.store).is_empty(), "khong hang asset nao cho mot anh bi tu choi");

    let log = domain_log_state.lock().unwrap();
    assert!(
        log.iter().any(|e| e.outcome == Some(DomainLogOutcome::MimeRejected)),
        "mot anh SVG bi TU CHOI MIME phai mang outcome MimeRejected trong nhat ky domain, \
         khong duoc giu nguyen Fetched (mang tinh khai bao SAI 'tai thanh cong'): {:?}",
        *log
    );
    drop(log);

    let assets_dir = opened.dir.join("assets");
    let entries: Vec<_> = fs::read_dir(&assets_dir)
        .unwrap_or_else(|e| panic!("doc {}: {e}", assets_dir.display()))
        .filter_map(|e| e.ok())
        .collect();
    assert!(entries.is_empty(), "KHONG tep nao duoc ghi cho mot anh bi tu choi MIME: {entries:?}");

    let source_text: String = opened
        .store
        .read(|conn| conn.query_row("SELECT source_text FROM chapter WHERE id = ?1", [opened.chapter_id], |r| r.get(0)))
        .expect("doc source_text that bai");
    assert!(source_text.contains("Doan mot"), "van con Doan mot");
    assert!(source_text.contains("Doan hai"), "van con Doan hai");
    assert!(source_text.contains("Doan ba"), "van con Doan ba -- mot anh bi loai khong lam mat chu");

    drop(opened);
    cleanup(&root);
}

// ═════════════════════════════════════════════════════════════════════════════════
// Một ảnh trượt tải KHÔNG làm trượt cả lượt nhập
// ═════════════════════════════════════════════════════════════════════════════════

#[test]
fn a_failed_image_fetch_does_not_fail_the_whole_import() {
    let root = temp_dir("image-fetch-failed");
    let dead_port = unreachable_port();
    let img_url = format!("http://127.0.0.1:{dead_port}/khong-ai-nghe.jpg");

    let items = vec![url_item("https://example.test/bai-hong", html_page_with_one_image(&img_url))];
    let shape = chapters_shape_if_all_ok(&items).expect("danh sach toan muc OK");

    let opened = create_work(&root, "Anh Hong", "en", "", shape, encoding_rs::UTF_8, Vec::new(), None, Vec::new(), &std::sync::Mutex::new(Vec::new()), None)
        .expect("mot anh mang loi KHONG duoc lam trot ca luot nhap");

    assert_eq!(opened.images_saved, 0);
    assert_eq!(opened.images_failed, 1);
    assert!(read_asset_rows(&opened.store).is_empty());

    let source_text: String = opened
        .store
        .read(|conn| conn.query_row("SELECT source_text FROM chapter WHERE id = ?1", [opened.chapter_id], |r| r.get(0)))
        .expect("doc source_text that bai");
    assert!(source_text.contains("Doan ba"), "Chuong van nhap DU chu du mot anh hong");

    drop(opened);
    cleanup(&root);
}

// ═════════════════════════════════════════════════════════════════════════════════
// Ảnh cùng `source_url`, cùng Tác phẩm — 0 lời gọi mạng thứ hai, dùng lại tệp đã có (AD-41)
// ═════════════════════════════════════════════════════════════════════════════════

#[test]
fn the_same_image_url_across_two_chapters_is_fetched_exactly_once_and_shares_one_file() {
    let root = temp_dir("dedup-same-url");
    let body: &'static [u8] = b"shared-image-body";
    let (port, counter, _handle) = spawn_counting_image_server(4, body, "image/gif");
    let img_url = format!("http://127.0.0.1:{port}/chung.gif");

    let items = vec![
        url_item("https://example.test/bai-mot", html_page_with_one_image(&img_url)),
        url_item("https://example.test/bai-hai", html_page_with_one_image(&img_url)),
    ];
    let shape = chapters_shape_if_all_ok(&items).expect("danh sach toan muc OK");

    let opened = create_work(&root, "Anh Trung URL", "en", "", shape, encoding_rs::UTF_8, Vec::new(), None, Vec::new(), &std::sync::Mutex::new(Vec::new()), None)
        .expect("tao Tac pham that bai");

    assert_eq!(opened.images_saved, 2, "hai VI TRI GIU, moi vi tri mot hang asset");
    assert_eq!(opened.images_failed, 0);
    assert_eq!(counter.load(Ordering::SeqCst), 1, "CHI mot lan tai that su — AD-41");

    let rows = read_asset_rows(&opened.store);
    assert_eq!(rows.len(), 2);
    assert_ne!(rows[0].0, rows[1].0, "hai hang phai thuoc HAI Chuong khac nhau");
    assert_eq!(rows[0].1, rows[1].1, "hai hang phai CUNG mot file_name -- dung lai tep da co");

    let assets_dir = opened.dir.join("assets");
    let entries: Vec<_> = fs::read_dir(&assets_dir).expect("doc assets/").filter_map(|e| e.ok()).collect();
    assert_eq!(entries.len(), 1, "CHI mot tep that tren dia du hai hang asset cung tro toi no");

    // D2 (vòng rà đối kháng 3 lớp) -- khoa bat bien BANG SO: `images_saved` dem HANG (khop
    // `rows.len()`), KHONG dem TEP that tren dia (`entries.len()`) -- ca dedup nay la noi hai
    // con so do LECH NHAU that su (2 hang / 1 tep), dung de khoa doc-comment cua `images_saved`
    // dung nghia "so hang asset", khong phai "so anh da tai & ghi".
    assert_eq!(
        opened.images_saved as usize,
        rows.len(),
        "images_saved phai khop CHINH XAC so hang asset that su duoc chen"
    );
    assert_ne!(
        opened.images_saved as usize,
        entries.len(),
        "ca dedup nay PHAI cho images_saved (2) khac so tep that tren dia (1) -- neu bang nhau, \
         fixture nay khong con do dung dieu no khai nua"
    );

    drop(opened);
    cleanup(&root);
}

// ═════════════════════════════════════════════════════════════════════════════════
// D2 (vòng rà đối kháng 2, 3 lớp) — `images_failed` đếm theo VỊ TRÍ, không theo LƯỢT GỌI
// MẠNG: cùng một URL ảnh LỖI dùng lại ở HAI Chương phải cho `images_failed == 2` (hai vị trí
// giữ không có hàng `asset`) trong khi cache dedup (AD-41) chỉ thực hiện ĐÚNG MỘT lượt gọi
// mạng cho URL đó — đối xứng với ca dedup THÀNH CÔNG ngay trên.
// ═════════════════════════════════════════════════════════════════════════════════

#[test]
fn a_failing_image_url_reused_across_two_chapters_counts_two_failed_positions_from_exactly_one_network_call() {
    let root = temp_dir("dedup-same-failing-url");
    // Máy chủ đếm KẾT NỐI (cùng khuôn `spawn_counting_image_server`) nhưng trả 404 — dedup
    // (AD-41) cache CẢ chiều thất bại, nên một URL lỗi dùng lại vẫn chỉ đáng ĐÚNG một kết nối.
    let counter = Arc::new(AtomicUsize::new(0));
    let counter_clone = Arc::clone(&counter);
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind cong tam");
    let port = listener.local_addr().expect("local_addr").port();
    let handle = thread::spawn(move || {
        for _ in 0..4 {
            let Ok((mut stream, _)) = listener.accept() else { break };
            counter_clone.fetch_add(1, Ordering::SeqCst);
            let _ = stream.set_read_timeout(Some(Duration::from_secs(5)));
            let mut discard = [0u8; 4096];
            let _ = stream.read(&mut discard);
            let _ = stream.write_all(b"HTTP/1.1 404 Not Found\r\nConnection: close\r\nContent-Length: 0\r\n\r\n");
        }
    });
    let img_url = format!("http://127.0.0.1:{port}/hong.jpg");

    let items = vec![
        url_item("https://example.test/bai-mot-hong", html_page_with_one_image(&img_url)),
        url_item("https://example.test/bai-hai-hong", html_page_with_one_image(&img_url)),
    ];
    let shape = chapters_shape_if_all_ok(&items).expect("danh sach toan muc OK");

    let opened = create_work(&root, "Anh Hong Trung URL", "en", "", shape, encoding_rs::UTF_8, Vec::new(), None, Vec::new(), &std::sync::Mutex::new(Vec::new()), None)
        .expect("mot anh mang loi khong duoc lam trot ca luot nhap");

    assert_eq!(opened.images_saved, 0);
    assert_eq!(
        opened.images_failed, 2,
        "HAI VI TRI giu (mot moi Chuong) deu khong co hang asset -- dem theo VI TRI, dung nhu \
         doc-comment images_failed da sua"
    );
    assert_eq!(
        counter.load(Ordering::SeqCst),
        1,
        "CHI MOT lan goi mang that su cho URL loi nay -- cache dedup AD-41 cache CA chieu that bai"
    );
    assert!(read_asset_rows(&opened.store).is_empty());

    drop(handle);
    drop(opened);
    cleanup(&root);
}

// ═════════════════════════════════════════════════════════════════════════════════
// Đường nhập tệp/dán tay — 0 ảnh, 0 lời gọi mạng
// ═════════════════════════════════════════════════════════════════════════════════

#[test]
fn the_paste_text_path_has_zero_images_and_touches_the_asset_table_not_at_all() {
    let root = temp_dir("blob-path-zero-images");
    let opened = create_work_from_text(&root, "Dan Tay", "en", "", "Cau mot. Cau hai.".to_owned())
        .expect("tao Tac pham tu van ban dan tay that bai");

    assert_eq!(opened.images_saved, 0);
    assert_eq!(opened.images_failed, 0);
    assert!(read_asset_rows(&opened.store).is_empty(), "bang asset phai RONG tren duong Blob");

    drop(opened);
    cleanup(&root);
}

// ═════════════════════════════════════════════════════════════════════════════════
// `source_url IS NULL` hợp lệ (ảnh người dùng tự thêm sau này — chưa có đường ghi ở story
// này, nhưng CỘT phải chừa chỗ) + rào rỗng `file_name` liệt TRỌN 25 điểm mã `White_Space`
// ═════════════════════════════════════════════════════════════════════════════════

#[test]
fn asset_source_url_allows_null_and_file_name_rejects_every_white_space_code_point() {
    let root = temp_dir("asset-ddl-invariants");
    let opened = create_work_from_text(&root, "DDL Asset", "en", "", "Cau mot.".to_owned())
        .expect("tao Tac pham that bai");
    let chapter_id = opened.chapter_id;

    opened
        .store
        .write(move |tx: &Transaction<'_>| {
            tx.execute(
                "INSERT INTO asset (chapter_id, file_name, source_url, anchor_after_segment_ord, \
                 byte_len, content_type, created_at) VALUES (?1, 'tu-them.jpg', NULL, 0, 10, \
                 'image/jpeg', strftime('%Y-%m-%dT%H:%M:%fZ','now'))",
                [chapter_id],
            )
        })
        .expect("source_url NULL phai la mot hang HOP LE (anh nguoi dung tu them)");

    // U+3000 (KHOẢNG TRẮNG BA-CHỮ-HÁN) — một trong 25 điểm mã `White_Space` — phải bị CHẶN,
    // cùng khuôn `GLOSSARY_ENTRY_DDL` (`src-tauri/AGENTS.md:37`: `trim()` cua SQLite chi cat
    // dau cach ASCII).
    let rejected = opened.store.write(move |tx: &Transaction<'_>| {
        tx.execute(
            "INSERT INTO asset (chapter_id, file_name, source_url, anchor_after_segment_ord, \
             byte_len, content_type, created_at) VALUES (?1, '\u{3000}', NULL, 0, 10, \
             'image/jpeg', strftime('%Y-%m-%dT%H:%M:%fZ','now'))",
            [chapter_id],
        )
    });
    assert!(rejected.is_err(), "file_name toan U+3000 phai bi CHECK cua ASSET_DDL tu choi");

    // D7 (vòng rà đối kháng 3 lớp) — `source_url = ''` (chuoi RONG, khac NULL) phai bi CHAN:
    // truoc sua ca nay bang co BA trang thai phan biet duoc (NULL / '' / URL that) trong khi
    // Y NGHIA chi co hai. U+3000 cung phai bi chan (cung rao 25 diem ma nhu file_name).
    let chapter_id_2 = opened.chapter_id;
    let empty_source_url_rejected = opened.store.write(move |tx: &Transaction<'_>| {
        tx.execute(
            "INSERT INTO asset (chapter_id, file_name, source_url, anchor_after_segment_ord, \
             byte_len, content_type, created_at) VALUES (?1, 'that-bai.jpg', '', 0, 10, \
             'image/jpeg', strftime('%Y-%m-%dT%H:%M:%fZ','now'))",
            [chapter_id_2],
        )
    });
    assert!(empty_source_url_rejected.is_err(), "source_url = '' (chuoi rong) phai bi CHECK tu choi");

    let whitespace_source_url_rejected = opened.store.write(move |tx: &Transaction<'_>| {
        tx.execute(
            "INSERT INTO asset (chapter_id, file_name, source_url, anchor_after_segment_ord, \
             byte_len, content_type, created_at) VALUES (?1, 'that-bai-2.jpg', '\u{3000}', 0, 10, \
             'image/jpeg', strftime('%Y-%m-%dT%H:%M:%fZ','now'))",
            [chapter_id_2],
        )
    });
    assert!(
        whitespace_source_url_rejected.is_err(),
        "source_url toan U+3000 phai bi CHECK tu choi, cung rao voi file_name"
    );

    // D7 — `byte_len` am phai bi CHAN o tang DDL, khong chi phu thuoc duong ghi Rust luon
    // dua vao mot gia tri khong am.
    let negative_byte_len_rejected = opened.store.write(move |tx: &Transaction<'_>| {
        tx.execute(
            "INSERT INTO asset (chapter_id, file_name, source_url, anchor_after_segment_ord, \
             byte_len, content_type, created_at) VALUES (?1, 'am.jpg', NULL, 0, -1, \
             'image/jpeg', strftime('%Y-%m-%dT%H:%M:%fZ','now'))",
            [chapter_id_2],
        )
    });
    assert!(negative_byte_len_rejected.is_err(), "byte_len < 0 phai bi CHECK cua ASSET_DDL tu choi");

    // R4 (vòng rà đối kháng 3, lớp 3) — `content_type` phai chiu CUNG rao rong 25 diem ma
    // voi file_name/source_url, khong duoc bo sot ngay canh hai cot vua sua.
    let chapter_id_3 = opened.chapter_id;
    let empty_content_type_rejected = opened.store.write(move |tx: &Transaction<'_>| {
        tx.execute(
            "INSERT INTO asset (chapter_id, file_name, source_url, anchor_after_segment_ord, \
             byte_len, content_type, created_at) VALUES (?1, 'ct-rong.jpg', NULL, 0, 10, \
             '', strftime('%Y-%m-%dT%H:%M:%fZ','now'))",
            [chapter_id_3],
        )
    });
    assert!(empty_content_type_rejected.is_err(), "content_type = '' (chuoi rong) phai bi CHECK tu choi");

    let whitespace_content_type_rejected = opened.store.write(move |tx: &Transaction<'_>| {
        tx.execute(
            "INSERT INTO asset (chapter_id, file_name, source_url, anchor_after_segment_ord, \
             byte_len, content_type, created_at) VALUES (?1, 'ct-khoang-trang.jpg', NULL, 0, 10, \
             '\u{3000}', strftime('%Y-%m-%dT%H:%M:%fZ','now'))",
            [chapter_id_3],
        )
    });
    assert!(
        whitespace_content_type_rejected.is_err(),
        "content_type toan U+3000 phai bi CHECK tu choi, cung rao voi file_name/source_url"
    );

    drop(opened);
    cleanup(&root);
}

// ═════════════════════════════════════════════════════════════════════════════════
// I/O Matrix hàng 6 — "Phản hồi vượt trần | thân > 20 MiB | Cắt giữa chừng, bỏ ảnh |
// FetchError::TooLarge vào nhật ký" — CHƯA có ca nào phủ đường ẢNH trước phiên này
// (`TooLarge` chỉ được nghiệm thu trên đường Trang, `webimport_contract.rs:193`/`:748`).
// ═════════════════════════════════════════════════════════════════════════════════

/// Server phản hồi `Content-Length: 200 MiB` rồi STREAM cho tới khi client cắt kết nối —
/// đúng khuôn `webimport_contract.rs::an_oversized_body_reaches_the_user_as_the_too_large_reason`,
/// chép riêng vào tệp này vì `spawn_once` không công khai xuyên hai tệp test.
fn spawn_oversized_image_server() -> (u16, thread::JoinHandle<()>) {
    const ADVERTISED_LEN: usize = 200 * 1024 * 1024;
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind cong tam");
    let port = listener.local_addr().expect("local_addr").port();
    let handle = thread::spawn(move || {
        let Ok((mut stream, _)) = listener.accept() else { return };
        let _ = stream.set_read_timeout(Some(Duration::from_secs(5)));
        let mut discard = [0u8; 4096];
        let _ = stream.read(&mut discard);
        let header = format!(
            "HTTP/1.1 200 OK\r\nContent-Type: image/jpeg\r\nContent-Length: {ADVERTISED_LEN}\r\n\r\n"
        );
        if stream.write_all(header.as_bytes()).is_err() {
            return;
        }
        let chunk = vec![b'x'; 256 * 1024];
        let mut sent = 0usize;
        while sent < ADVERTISED_LEN {
            let this_write = (ADVERTISED_LEN - sent).min(chunk.len());
            if stream.write_all(&chunk[..this_write]).is_err() {
                // Client (fetcher.rs) da dong ket noi som sau khi vuot MAX_RESPONSE_BYTES —
                // dung ket qua mong doi, khong phai loi ha tang.
                break;
            }
            sent += this_write;
        }
    });
    (port, handle)
}

/// 🔵 **ĐỔI TÊN 2026-09-08 (Ice ký, vòng rà đối kháng 3 lớp) — tên cũ khai `denied`, thân
/// assert `Allowed(Tier::Two)`.** Sau khi `DomainLogEntry` nới thêm `outcome` (mục A), thân
/// ca dưới đây khẳng định ĐÚNG lý do trượt (`DomainLogOutcome::TooLarge`), không còn phải
/// dừng ở mức "đã cho phép" — tên hàm đổi theo cho khớp.
#[test]
fn an_oversized_image_response_is_cut_off_with_a_too_large_outcome_no_file_and_the_chapter_still_imports() {
    let root = temp_dir("image-too-large");
    let (port, _handle) = spawn_oversized_image_server();
    let img_url = format!("http://127.0.0.1:{port}/khong-lo.jpg");

    let items = vec![url_item("https://example.test/bai-qua-lon", html_page_with_one_image(&img_url))];
    let shape = chapters_shape_if_all_ok(&items).expect("danh sach toan muc OK");

    let domain_log_state: DomainLogState = Mutex::new(Vec::new());
    let opened = create_work(
        &root,
        "Anh Qua Lon",
        "en",
        "",
        shape,
        encoding_rs::UTF_8,
        Vec::new(),
        None,
        Vec::new(),
        &domain_log_state,
        None,
    )
    .expect("mot anh vuot tran KHONG duoc lam trot ca luot nhap");

    // (a)+(c) 0 anh duoc luu, images_failed tang.
    assert_eq!(opened.images_saved, 0, "anh vuot MAX_RESPONSE_BYTES khong duoc luu");
    assert_eq!(opened.images_failed, 1);

    // (b) 0 hang `asset`.
    assert!(read_asset_rows(&opened.store).is_empty(), "khong hang asset nao cho mot anh bi cat giua chung");

    // (a) 0 tep trong assets/.
    let assets_dir = opened.dir.join("assets");
    let entries: Vec<_> = fs::read_dir(&assets_dir)
        .unwrap_or_else(|e| panic!("doc {}: {e}", assets_dir.display()))
        .filter_map(|e| e.ok())
        .collect();
    assert!(entries.is_empty(), "KHONG tep nao duoc ghi cho mot anh vuot tran: {entries:?}");

    // (d) Chuong van nhap DU chu.
    let source_text: String = opened
        .store
        .read(|conn| conn.query_row("SELECT source_text FROM chapter WHERE id = ?1", [opened.chapter_id], |r| r.get(0)))
        .expect("doc source_text that bai");
    assert!(source_text.contains("Doan mot"));
    assert!(source_text.contains("Doan hai"));
    assert!(source_text.contains("Doan ba"), "Chuong van nhap DU chu du anh minh hoa vuot tran");

    // (e) mot ban ghi nhat ky domain mang DUNG LY DO -- Muc A (Ice ky 2026-09-08) noi
    // `DomainLogEntry` them truong `outcome`: host nay phai duoc `Allowed(Tier::Two)` VA
    // mang outcome `TooLarge` -- phan biet duoc voi mot anh tai XONG TRON VEN (`Fetched`).
    let log = domain_log_state.lock().unwrap();
    assert!(
        log.iter().any(|e| matches!(e.decision, DomainLogDecision::Allowed(Tier::Two))
            && e.outcome == Some(DomainLogOutcome::TooLarge)),
        "phai co it nhat mot ban ghi ALLOWED tang 2 mang outcome TooLarge cho host nay: {:?}",
        *log
    );
    drop(log);

    drop(opened);
    cleanup(&root);
}

// ═════════════════════════════════════════════════════════════════════════════════
// I/O Matrix hàng 10 — "Ghi tệp trượt giữa chừng | đĩa đầy | Lượt nhập trượt sạch, thư mục
// `.atproj` bị dọn | IpcError có message_key" — CHƯA có ca nào ép lỗi này chạy thật trước
// phiên này (`MessageKey::AssetWriteFailed` có đúng MỘT tham chiếu trong `src/`, 0 trong
// `tests/`). Ép bằng một CHƯỚNG NGẠI THẬT trên đường tệp đích (đổi quyền thư mục `assets/`
// còn RỖNG), không gọi thẳng `IpcError::new(...)`/`WorkError::CreateFailed` để mô phỏng.
// 🔴 `#[cfg(unix)]` — quyền thư mục kiểu Unix (bit `w` trên chính thư mục) là đường DUY NHẤT
// chặn `File::create` bên trong nó một cách tất định; thuộc tính "chỉ đọc" của Windows trên
// một THƯ MỤC không chặn việc tạo tệp mới bên trong, nên kỹ thuật này không dịch được sang đó.
// ═════════════════════════════════════════════════════════════════════════════════

#[cfg(unix)]
#[test]
fn a_disk_write_failure_mid_asset_write_fails_the_whole_import_and_removes_the_atproj_folder() {
    use std::os::unix::fs::PermissionsExt;

    let root = temp_dir("asset-write-failure");

    // Server CHỦ Ý trễ 80 ms trước khi trả lời — cho luồng canh (dưới) đủ thời gian PHÁT
    // HIỆN thư mục `assets/` vừa được `create_work_folder` dựng lên RỒI đổi quyền của nó
    // TRƯỚC KHI `std::fs::write` của pha ảnh chạm tới, thay vì đua theo mili-giây.
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind cong tam");
    let port = listener.local_addr().expect("local_addr").port();
    let server = thread::spawn(move || {
        let Ok((mut stream, _)) = listener.accept() else { return };
        let _ = stream.set_read_timeout(Some(Duration::from_secs(5)));
        let mut discard = [0u8; 4096];
        let _ = stream.read(&mut discard);
        thread::sleep(Duration::from_millis(80));
        let body = b"anh that nhung se khong bao gio duoc ghi";
        let header = format!(
            "HTTP/1.1 200 OK\r\nContent-Type: image/jpeg\r\nContent-Length: {}\r\n\r\n",
            body.len()
        );
        let _ = stream.write_all(header.as_bytes());
        let _ = stream.write_all(body);
    });

    // Luồng canh — dò `root` cho một `.atproj/assets` MỚI xuất hiện rồi khoá bit ghi (0o500 =
    // r-x, KHÔNG w) trên chính thư mục đó. Thư mục vẫn RỖNG lúc bị khoá (ảnh chưa tải xong,
    // nhờ độ trễ 80 ms ở trên) — `File::create` bên trong nó sau đó trượt THẬT với
    // `PermissionDenied`, không phải một lỗi giả lập.
    let root_for_watcher = root.clone();
    let locked_path = Arc::new(std::sync::Mutex::new(None::<PathBuf>));
    let locked_path_clone = Arc::clone(&locked_path);
    let watcher = thread::spawn(move || {
        for _ in 0..2000 {
            if let Ok(entries) = fs::read_dir(&root_for_watcher) {
                for entry in entries.flatten() {
                    let p = entry.path();
                    if p.extension().and_then(|e| e.to_str()) == Some("atproj") {
                        let assets = p.join("assets");
                        if assets.is_dir() {
                            let _ = fs::set_permissions(&assets, fs::Permissions::from_mode(0o500));
                            *locked_path_clone.lock().unwrap() = Some(assets);
                            return;
                        }
                    }
                }
            }
            thread::sleep(Duration::from_millis(1));
        }
    });

    let img_url = format!("http://127.0.0.1:{port}/anh-khong-bao-gio-duoc-ghi.jpg");
    let items = vec![url_item("https://example.test/bai-dia-day", html_page_with_one_image(&img_url))];
    let shape = chapters_shape_if_all_ok(&items).expect("danh sach toan muc OK");

    let result = create_work(&root, "Dia Day", "en", "", shape, encoding_rs::UTF_8, Vec::new(), None, Vec::new(), &std::sync::Mutex::new(Vec::new()), None);

    watcher.join().expect("luong canh panic");
    let _ = server.join();

    // C3 (vòng rà đối kháng 2, 3 lớp) — ca này ĐUA VỚI TẢI MÁY (luồng canh phải khoá
    // `assets/` trước khi ảnh kịp ghi). Một ca ĐỎ vì luồng canh KHÔNG KỊP khoá (điều kiện đo
    // không đạt) phải nói ra ĐÚNG điều đó, không giả dạng một hồi quy sản phẩm — kiểm điều
    // kiện TRƯỚC khi kiểm hành vi.
    assert!(
        locked_path.lock().unwrap().is_some(),
        "luong canh KHONG kip khoa assets/ truoc khi anh ghi xong -- ca nay khong do gi ca, \
         khong phai mot hoi quy san pham (thu chay lai, hoac tren mot may it tai hon)"
    );

    // (a) create_work PHẢI trả `Err` mang ĐÚNG `message_key`.
    let err = result.expect_err(
        "ghi byte anh xuong mot thu muc chi-doc PHAI lam TRUOT ca luot nhap, khong duoc \
         thanh cong am tham bo qua anh",
    );
    assert_eq!(
        err.message_key(),
        MessageKey::AssetWriteFailed,
        "loi phai mang dung message_key cho duong ghi tep trong (khong phai mot loi kho chung chung)"
    );

    // (b) Thư mục `.atproj` KHÔNG còn trên đĩa — trước khi kiểm, TRẢ LẠI quyền ghi cho
    // `assets/` (nếu luồng canh đã khoá nó VÀ `remove_folder` chưa xoá xong nó) để một lượt
    // dọn TAY (`cleanup`) không tự trượt vì chính khoá quyền của ca test này.
    if let Some(path) = locked_path.lock().unwrap().take() {
        let _ = fs::set_permissions(&path, fs::Permissions::from_mode(0o700));
    }
    let atproj_dirs: Vec<PathBuf> = fs::read_dir(&root)
        .into_iter()
        .flatten()
        .flatten()
        .map(|e| e.path())
        .filter(|p| p.extension().and_then(|e| e.to_str()) == Some("atproj"))
        .collect();
    assert!(
        atproj_dirs.is_empty(),
        "mot luot nhap TRUOT phai don SACH .atproj — con lai: {atproj_dirs:?}"
    );

    cleanup(&root);
}

// ═════════════════════════════════════════════════════════════════════════════════
// B1 (vòng rà đối kháng 3 lớp) — nhật ký domain của một ảnh ĐÃ tải xong PHẢI SỐNG SÓT trong
// `domain_log_state` dù `create_work` sau đó trả `Err` vì MỘT ảnh khác ghi tệp thất bại giữa
// chừng. Trước sửa B1, `domain_log` là một `Vec` CỤC BỘ chỉ được gắn vào `OpenWork` trên
// đường THÀNH CÔNG — dấu `?` ở nhánh lỗi vứt sạch nó (`project.rs:791` cũ).
// ═════════════════════════════════════════════════════════════════════════════════

// 🔴 P0 (vòng rà đối kháng 2, 3 lớp) — THIẾU `#[cfg(unix)]` ở đây làm TOÀN BỘ binary
// `asset_contract` không biên dịch được trên Windows: `std::os::unix::fs::PermissionsExt` và
// `fs::Permissions::from_mode` không tồn tại ngoài Unix. Cùng khuôn hai chỗ khác trong CHÍNH
// tệp này (`use MessageKey` ở đầu tệp, ca `a_disk_write_failure_mid_asset_write_fails_...`)
// — đây là chỗ SÓT thứ ba, và nó nặng hơn hai chỗ kia: một `use` sai `cfg` chỉ tạo CẢNH BÁO,
// còn một hàm gọi `PermissionsExt`/`from_mode` không tồn tại là LỖI BIÊN DỊCH, giết cả BINARY
// — tức 0/16 ca của tệp này (không chỉ ca B1) từng chạy trên nhánh windows-2025 của CI.
#[cfg(unix)]
#[test]
fn a_later_image_write_failure_does_not_erase_the_domain_log_entry_of_an_earlier_successful_image() {
    use std::os::unix::fs::PermissionsExt;

    let root = temp_dir("b1-domain-log-survives-failure");

    // Anh THU NHAT -- tra loi NGAY (khong tre): phai tai/ghi/vao log TRUOC KHI luong canh
    // kip khoa thu muc.
    let body1: &'static [u8] = b"anh-thu-nhat-thanh-cong";
    let (port1, _counter1, _h1) = spawn_counting_image_server(4, body1, "image/jpeg");
    let img1 = format!("http://127.0.0.1:{port1}/thanh-cong.jpg");

    // Anh THU HAI -- may chu tre 80 ms truoc khi tra loi, cho luong canh du thoi gian khoa
    // thu muc `assets/` NGAY SAU KHI anh thu nhat da ghi xong tep (thu muc co DUNG 1 muc).
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind cong tam");
    let port2 = listener.local_addr().expect("local_addr").port();
    let server2 = thread::spawn(move || {
        let Ok((mut stream, _)) = listener.accept() else { return };
        let _ = stream.set_read_timeout(Some(Duration::from_secs(5)));
        let mut discard = [0u8; 4096];
        let _ = stream.read(&mut discard);
        thread::sleep(Duration::from_millis(80));
        let body = b"anh thu hai se khong bao gio duoc ghi";
        let header = format!(
            "HTTP/1.1 200 OK\r\nContent-Type: image/jpeg\r\nContent-Length: {}\r\n\r\n",
            body.len()
        );
        let _ = stream.write_all(header.as_bytes());
        let _ = stream.write_all(body);
    });
    let img2 = format!("http://127.0.0.1:{port2}/that-bai.jpg");

    // 🔵 ĐO 2026-09-09 -- chạy MỘT MÌNH tệp này (khởi động lạnh `shared_client()` của
    // `fetcher.rs`, thread nền + runtime tokio riêng), độ trễ TRƯỚC lượt ghi tệp ĐẦU TIÊN đo
    // được tới ~5,7 s -- chi phí khởi động MỘT LẦN của tầng mạng, không phải thời gian xử lý
    // ảnh thật (hai lượt ghi của CHÍNH ca này cách nhau ĐÚNG ~90 ms, đúng như kỳ vọng từ độ
    // trễ 80 ms của server2). Một hạn canh gác 3 s (giá trị ban đầu) đo được là NGẮN HƠN chi
    // phí khởi động đó -- canh gác hết hạn TRƯỚC KHI ảnh thứ nhất kịp ghi, khoá SAI mục tiêu
    // (0 khoá nào áp dụng) và làm `create_work` luôn trả `Ok`. Nâng hạn lên 30 s (khuôn "đo
    // trước khi tin", MEMORY "Hai lượt đo phải cùng tải máy") để canh gác chắc chắn còn sống
    // qua chi phí khởi động một-lần đó, dù ca này chạy một mình hay giữa cả bộ.
    const WATCHDOG_MAX_MILLIS: u64 = 30_000;

    let root_for_watcher = root.clone();
    let locked_path: Arc<Mutex<Option<PathBuf>>> = Arc::new(Mutex::new(None));
    let locked_path_clone = Arc::clone(&locked_path);
    let watcher = thread::spawn(move || {
        for _ in 0..WATCHDOG_MAX_MILLIS {
            if let Ok(entries) = fs::read_dir(&root_for_watcher) {
                for entry in entries.flatten() {
                    let p = entry.path();
                    if p.extension().and_then(|e| e.to_str()) == Some("atproj") {
                        let assets = p.join("assets");
                        if let Ok(assets_entries) = fs::read_dir(&assets) {
                            if assets_entries.count() >= 1 {
                                let _ = fs::set_permissions(&assets, fs::Permissions::from_mode(0o500));
                                *locked_path_clone.lock().unwrap() = Some(assets);
                                return;
                            }
                        }
                    }
                }
            }
            thread::sleep(Duration::from_millis(1));
        }
    });

    let html = format!(
        "<html><head><title>Bai viet</title></head><body><article><h1>Tieu de</h1>\
         <p>Doan mot co du chu de duoc Readability chon lam noi dung chinh cua trang, \
         nhieu chu hon de vuot nguong do dai toi thieu.</p>\
         <img src=\"{img1}\">\
         <p>Doan hai tiep tuc noi dung that su cua bai viet, khong phai menu hay quang cao, \
         du dai de dom_smoothie cham diem cao cho khoi nay.</p>\
         <img src=\"{img2}\">\
         <p>Doan ba dong y nghia, giu cho tong do dai van ban vuot qua nguong toi thieu can \
         thiet de Readability tin day la mot bai viet that.</p>\
         </article></body></html>"
    );
    let items = vec![url_item("https://example.test/b1-hai-anh-mot-that-bai", html)];
    let shape = chapters_shape_if_all_ok(&items).expect("danh sach toan muc OK");

    let domain_log_state: DomainLogState = Mutex::new(Vec::new());
    let result = create_work(
        &root,
        "B1 Nhat Ky Song Sot",
        "en",
        "",
        shape,
        encoding_rs::UTF_8,
        Vec::new(),
        None,
        Vec::new(),
        &domain_log_state,
        None,
    );

    watcher.join().expect("luong canh panic");
    let _ = server2.join();

    // C3 (vòng rà đối kháng 2, 3 lớp) — cùng lý do ca `a_disk_write_failure_...`: kiểm ĐIỀU
    // KIỆN đua trước khi kiểm HÀNH VI. Một ĐỎ vì luồng canh không kịp khoá phải nói ra đúng
    // điều đó, không giả dạng một hồi quy của B1.
    assert!(
        locked_path.lock().unwrap().is_some(),
        "luong canh KHONG kip khoa assets/ truoc khi anh thu nhat ghi xong -- ca nay khong do \
         gi ca, khong phai mot hoi quy B1 (thu chay lai, hoac tren mot may it tai hon)"
    );

    assert!(result.is_err(), "anh thu hai ghi tep that bai phai lam TRUOT ca luot nhap");

    if let Some(path) = locked_path.lock().unwrap().take() {
        let _ = fs::set_permissions(&path, fs::Permissions::from_mode(0o700));
    }

    // Cốt lõi B1: dù `create_work` trả `Err`, domain_log_state (sở hữu bởi CHỖ GỌI, không
    // phải bởi `create_work`) vẫn phải mang lại bản ghi của ảnh ĐÃ tải xong TRƯỚC lượt trượt.
    let log = domain_log_state.lock().unwrap();
    let fetched_count = log.iter().filter(|e| e.outcome == Some(DomainLogOutcome::Fetched)).count();
    assert_eq!(
        fetched_count, 2,
        "CẢ HAI ảnh đã tải mạng THÀNH CÔNG trước khi ảnh thứ hai ghi tệp trượt -- B1 đòi push \
         vào domain_log_state NGAY khi fetch xong, TRƯỚC bước ghi đĩa có thể trượt; log hiện \
         có: {:?}",
        *log
    );

    cleanup(&root);
}

// ═════════════════════════════════════════════════════════════════════════════════
// C1 (vòng rà đối kháng 3 lớp) — neo đi qua `create_work` với LUẬT LÀM SẠCH THẬT SỰ BẬT,
// không `Vec::new()`. `core::segment::anchor::tests` đã khoá cơ chế ở tầng ĐƠN VỊ; ca này
// khoá ĐẦU-CUỐI: `create_work` phải THẬT SỰ truyền `cleanup_rules` xuống `compute_anchor`,
// không chỉ xuống bước 3 của chuỗi AD-39.
// ═════════════════════════════════════════════════════════════════════════════════

#[test]
fn create_work_computes_the_anchor_with_a_real_enabled_cleanup_rule_not_an_empty_slice() {
    let root = temp_dir("anchor-with-real-cleanup-rule");
    let body: &'static [u8] = b"anh-voi-luat-lam-sach";
    let (port, _counter, _handle) = spawn_counting_image_server(4, body, "image/jpeg");
    let img_url = format!("http://127.0.0.1:{port}/anh.jpg");

    let items = vec![url_item("https://example.test/bai-co-luat-lam-sach", html_page_with_one_image(&img_url))];
    let shape = chapters_shape_if_all_ok(&items).expect("danh sach toan muc OK");

    // Luật BẬT khớp một chuỗi con NẰM TRỌN trong Đoạn hai (đứng TRƯỚC ảnh) — không vắt qua
    // ranh giới ảnh (§Ask First spec 6.11).
    let cleanup_rules = vec![CleanupRule {
        tier: CleanupRuleTier::Global,
        id: 1,
        pattern: "quang cao, ".to_owned(),
        kind: CleanupRuleKind::Literal,
        enabled: true,
    }];

    let opened = create_work(
        &root,
        "Anh Voi Luat Lam Sach",
        "en",
        "",
        shape,
        encoding_rs::UTF_8,
        cleanup_rules,
        None,
        Vec::new(),
        &DomainLogState::new(Vec::new()),
        None,
    )
    .expect("tao Tac pham voi luat lam sach BAT that bai");

    let source_text: String = opened
        .store
        .read(|conn| conn.query_row("SELECT source_text FROM chapter WHERE id = ?1", [opened.chapter_id], |r| r.get(0)))
        .expect("doc source_text that bai");
    assert!(
        !source_text.contains("quang cao"),
        "luat lam sach phai THAT SU chay tren chapter.source_text -- fixture van con \"quang cao\": {source_text:?}"
    );

    assert_eq!(opened.images_saved, 1, "luat lam sach BAT khong duoc lam anh trot tai");
    let rows = read_asset_rows(&opened.store);
    assert_eq!(rows.len(), 1, "dung mot hang asset: {rows:?}");
    assert_eq!(
        rows[0].3, 3,
        "neo phai VAN dung -- <h1> + hai doan van dau (luat lam sach chi rut ngan chu, khong xoa nguyen cau)"
    );

    drop(opened);
    cleanup(&root);
}

// ═════════════════════════════════════════════════════════════════════════════════
// C4 (vòng rà đối kháng 2, 3 lớp) — `compute_anchor` trả `Err` ĐI QUA `create_work`, không
// chỉ ở ca đơn vị `anchor.rs::tests`. Đo được (không suy luận): với fixture
// `html_page_with_one_image`, văn bản ghép THẬT giữa hai đoạn quanh ảnh là
// "...cho khoi nay.\n\nDoan ba..." — một luật làm sạch khớp ĐÚNG chuỗi vắt qua đúng ranh giới
// đó phá giả định phân tách-theo-tiền-tố (§Ask First spec 6.11), khiến tự kiểm của
// `compute_anchor` trả `Err` — CHÍNH XÁC cơ chế mà
// `anchor.rs::a_cleanup_rule_matching_across_the_image_boundary_is_caught_by_the_self_check_not_silently_wrong`
// đã đo ở tầng đơn vị.
// ═════════════════════════════════════════════════════════════════════════════════

#[test]
fn create_work_counts_the_image_as_failed_not_a_panic_when_compute_anchor_self_check_rejects_the_prefix() {
    let root = temp_dir("compute-anchor-err-through-create-work");
    let body: &'static [u8] = b"anh-vat-qua-ranh-gioi";
    let (port, counter, _handle) = spawn_counting_image_server(4, body, "image/jpeg");
    let img_url = format!("http://127.0.0.1:{port}/anh.jpg");

    let items = vec![url_item("https://example.test/bai-vat-ranh-gioi", html_page_with_one_image(&img_url))];
    let shape = chapters_shape_if_all_ok(&items).expect("danh sach toan muc OK");

    // Luật BẬT khớp ĐÚNG chuỗi vắt qua ranh giới nơi ảnh từng đứng (nối liền cuối Đoạn hai
    // với đầu Đoạn ba) — đo được ở trên bằng cách in `chapter.source_text` KHÔNG luật, không
    // suy diễn hình dạng "\n\n".
    let cleanup_rules = vec![CleanupRule {
        tier: CleanupRuleTier::Global,
        id: 1,
        pattern: "cho khoi nay.\n\nDoan ba".to_owned(),
        kind: CleanupRuleKind::Literal,
        enabled: true,
    }];

    let opened = create_work(
        &root,
        "Anh Vat Ranh Gioi",
        "en",
        "",
        shape,
        encoding_rs::UTF_8,
        cleanup_rules,
        None,
        Vec::new(),
        &DomainLogState::new(Vec::new()),
        None,
    )
    .expect("mot anh tinh neo that bai (compute_anchor Err) KHONG duoc lam trot ca luot nhap");

    // Cốt lõi C4: `compute_anchor` trả `Err` ⇒ `prepare_chapter_images` đếm nó vào
    // `images_failed`, KHÔNG panic, KHÔNG dừng cả lượt nhập (đúng doc-comment
    // `prepare_chapter_images`: "MỌI lý do khác [ngoài ghi đĩa] chỉ đếm vào `images_failed`").
    assert_eq!(opened.images_saved, 0, "anh khong tinh duoc neo thi khong duoc luu");
    assert_eq!(opened.images_failed, 1, "anh khong tinh duoc neo phai dem vao images_failed");
    assert!(read_asset_rows(&opened.store).is_empty(), "khong hang asset nao cho mot anh tinh neo that bai");

    // Neo KHÔNG tính được nghĩa là hàm KHÔNG được thử tải — 0 kết nối mạng, cùng khuôn
    // doc-comment `prepare_chapter_images`: "một ảnh không neo được thì không đáng một lượt
    // tải (tránh ghi một tệp mồ côi)".
    assert_eq!(
        counter.load(Ordering::SeqCst),
        0,
        "anh khong tinh duoc neo thi KHONG duoc thu tai mang -- neo tinh TRUOC mang"
    );

    drop(opened);
    cleanup(&root);
}

// ═════════════════════════════════════════════════════════════════════════════════
// C2 (vòng rà đối kháng 3 lớp) — `block_overrides` trên đường ảnh: 0 ca trước phiên này.
// ═════════════════════════════════════════════════════════════════════════════════

/// C2, chiều TẮT — người dùng ĐÃ LOẠI một ảnh mà máy (`machine_kept`) đang GIỮ. Ảnh đó phải
/// biến mất HOÀN TOÀN khỏi pha tải: 0 kết nối mạng, 0 tệp, 0 hàng `asset` — KHÔNG đếm vào
/// `images_failed` (nó không hề được THỬ, không phải một lượt thử thất bại).
#[test]
fn block_overrides_force_off_an_otherwise_kept_image_zero_connections_zero_file_zero_row() {
    let root = temp_dir("block-override-force-off");
    let counter = Arc::new(AtomicUsize::new(0));
    let counter_clone = Arc::clone(&counter);
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind cong tam");
    let port = listener.local_addr().expect("local_addr").port();
    let _server = thread::spawn(move || {
        if let Ok((mut stream, _)) = listener.accept() {
            counter_clone.fetch_add(1, Ordering::SeqCst);
            let _ = stream.set_read_timeout(Some(Duration::from_secs(5)));
            let mut discard = [0u8; 4096];
            let _ = stream.read(&mut discard);
            let _ = stream.write_all(b"HTTP/1.1 200 OK\r\nContent-Type: image/jpeg\r\nContent-Length: 3\r\n\r\nabc");
        }
    });
    let img_url = format!("http://127.0.0.1:{port}/khong-duoc-goi.jpg");

    let items = vec![url_item("https://example.test/bai-force-off", html_page_with_one_image(&img_url))];
    let shape = chapters_shape_if_all_ok(&items).expect("danh sach toan muc OK");

    // Khối theo THỨ TỰ tài liệu trong `html_page_with_one_image`: 0=<h1>, 1=Doan mot,
    // 2=Doan hai, 3=<img>, 4=Doan ba. Ép TẮT đúng chỉ số 3 -- `machine_kept` cua no la `true`
    // (do o giua hai doan van GIU), nen day la mot phep EP THAT, khong trung voi may.
    let block_overrides: Vec<Option<bool>> = vec![None, None, None, Some(false)];

    let opened = create_work(
        &root,
        "Force Off",
        "en",
        "",
        shape,
        encoding_rs::UTF_8,
        Vec::new(),
        None,
        block_overrides,
        &DomainLogState::new(Vec::new()),
        None,
    )
    .expect("ep tat mot anh khong duoc lam trot ca luot nhap");

    // Cho luong server mot khoang ngan de lo ra mot ket noi NEU CO (khong nen co).
    thread::sleep(Duration::from_millis(50));
    assert_eq!(counter.load(Ordering::SeqCst), 0, "anh bi EP TAT khong duoc nhan MOT ket noi mang nao");
    assert_eq!(opened.images_saved, 0);
    assert_eq!(opened.images_failed, 0, "anh bi EP TAT khong duoc THU, nen khong duoc dem la mot lan THAT BAI");
    assert!(read_asset_rows(&opened.store).is_empty(), "0 hang asset cho mot anh bi EP TAT");

    drop(opened);
    cleanup(&root);
}

/// C2, chiều BẬT — người dùng ĐÃ GIỮ một ảnh mà máy (`machine_kept`) đang LOẠI. Fixture đo
/// THẬT (không suy luận): một `<div id="header">` ĐỨNG NGOÀI `<article>` bị chính
/// `dom_smoothie::Readability::parse()` loại khỏi nội dung chính (`grabArticle`), nên
/// `text_content` không hề chứa "Menu ngan" — khối đó (và ảnh liền sau nó, theo
/// `infer_image_kept_state`: không có khối chữ GIỮ nào đứng trước, khối chữ GIỮ đầu tiên
/// đứng SAU cũng không giúp vì bản thân "Menu ngan" đã `machine_kept = false`) nhận
/// `machine_kept = false` một cách TỰ NHIÊN — đo bằng `cargo run` trực tiếp trên
/// `webimport::extract` trước khi viết ca này, không phải suy diễn từ tài liệu.
#[test]
fn block_overrides_force_on_an_otherwise_excluded_image_gets_fetched_and_gets_a_row() {
    let root = temp_dir("block-override-force-on");
    let body: &'static [u8] = b"logo-bytes";
    let (port, counter, _handle) = spawn_counting_image_server(4, body, "image/png");
    let img_url = format!("http://127.0.0.1:{port}/logo.png");

    let html = format!(
        "<html><head><title>T</title></head><body>\
         <div id=\"header\"><p>Menu ngan</p><img src=\"{img_url}\"></div>\
         <article><h1>Tieu de</h1>\
         <p>Doan mot co du chu de duoc Readability chon lam noi dung chinh cua trang, \
         nhieu chu hon de vuot nguong do dai toi thieu.</p>\
         <p>Doan hai tiep tuc noi dung that su cua bai viet, khong phai menu hay quang cao, \
         du dai de dom_smoothie cham diem cao cho khoi nay.</p>\
         <p>Doan ba dong y nghia, giu cho tong do dai van ban vuot qua nguong toi thieu can \
         thiet de Readability tin day la mot bai viet that.</p>\
         </article></body></html>"
    );
    let items = vec![url_item("https://example.test/bai-force-on", html)];
    let shape = chapters_shape_if_all_ok(&items).expect("danh sach toan muc OK");

    // Chi so 1 (anh trong <div id="header">) -- xem doc-comment ham nay cho phep do xac nhan
    // machine_kept TU NHIEN cua no la `false`. Ep BAT bang tay.
    let block_overrides: Vec<Option<bool>> = vec![None, Some(true)];

    let opened = create_work(
        &root,
        "Force On",
        "en",
        "",
        shape,
        encoding_rs::UTF_8,
        Vec::new(),
        None,
        block_overrides,
        &DomainLogState::new(Vec::new()),
        None,
    )
    .expect("ep bat mot anh khong duoc lam trot ca luot nhap");

    assert_eq!(
        opened.images_saved, 1,
        "anh bi may LOAI tu nhien nhung nguoi dung EP GIU phai duoc tai va luu"
    );
    assert_eq!(opened.images_failed, 0);
    assert_eq!(counter.load(Ordering::SeqCst), 1, "dung MOT lan tai that su cho anh vua duoc EP BAT");
    let rows = read_asset_rows(&opened.store);
    assert_eq!(rows.len(), 1, "dung mot hang asset cho anh vua duoc EP BAT: {rows:?}");
    assert_eq!(rows[0].3, 0, "khong khoi chu GIU nao dung truoc anh nay -- neo = 0");

    drop(opened);
    cleanup(&root);
}

// ═════════════════════════════════════════════════════════════════════════════════
// C3 (vòng rà đối kháng 3 lớp) — ca ÂM cho §Always "tầng 2 dựng từ ĐÚNG những ảnh
// `effective_kept` trả `true`": một ảnh KHÔNG giữ (TỰ NHIÊN, không ép tay) không được đưa
// host của nó vào tầng 2 -- 0 kết nối, khác hẳn C2 (ảnh bị ÉP tắt bằng tay).
// ═════════════════════════════════════════════════════════════════════════════════

#[test]
fn a_naturally_excluded_image_host_never_enters_tier_two_and_receives_zero_connections() {
    let root = temp_dir("naturally-excluded-image");
    let counter = Arc::new(AtomicUsize::new(0));
    let counter_clone = Arc::clone(&counter);
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind cong tam");
    let port = listener.local_addr().expect("local_addr").port();
    let _server = thread::spawn(move || {
        // Toi da 4 ket noi -- neu tang 2 lo mo cho host nay, ca nay se BAT duoc thay vi treo.
        for _ in 0..4 {
            let Ok((mut stream, _)) = listener.accept() else { break };
            counter_clone.fetch_add(1, Ordering::SeqCst);
            let _ = stream.set_read_timeout(Some(Duration::from_secs(5)));
            let mut discard = [0u8; 4096];
            let _ = stream.read(&mut discard);
            let _ = stream.write_all(b"HTTP/1.1 200 OK\r\nContent-Type: image/png\r\nContent-Length: 3\r\n\r\nabc");
        }
    });
    let img_url = format!("http://127.0.0.1:{port}/logo-khong-duoc-goi.png");

    // Cùng fixture đo THẬT ở `block_overrides_force_on_...` -- ảnh trong `<div id="header">`
    // (NGOÀI `<article>`) bị chính `Readability::parse()` loại, `machine_kept = false` một
    // cách TỰ NHIÊN, KHÔNG một `block_overrides` nào ở ca này.
    let html = format!(
        "<html><head><title>T</title></head><body>\
         <div id=\"header\"><p>Menu ngan</p><img src=\"{img_url}\"></div>\
         <article><h1>Tieu de</h1>\
         <p>Doan mot co du chu de duoc Readability chon lam noi dung chinh cua trang, \
         nhieu chu hon de vuot nguong do dai toi thieu.</p>\
         <p>Doan hai tiep tuc noi dung that su cua bai viet, khong phai menu hay quang cao, \
         du dai de dom_smoothie cham diem cao cho khoi nay.</p>\
         <p>Doan ba dong y nghia, giu cho tong do dai van ban vuot qua nguong toi thieu can \
         thiet de Readability tin day la mot bai viet that.</p>\
         </article></body></html>"
    );
    let items = vec![url_item("https://example.test/bai-loai-tu-nhien", html)];
    let shape = chapters_shape_if_all_ok(&items).expect("danh sach toan muc OK");

    let opened = create_work(
        &root,
        "Loai Tu Nhien",
        "en",
        "",
        shape,
        encoding_rs::UTF_8,
        Vec::new(),
        None,
        Vec::new(),
        &DomainLogState::new(Vec::new()),
        None,
    )
    .expect("mot anh bi may loai tu nhien khong duoc lam trot ca luot nhap");

    thread::sleep(Duration::from_millis(50));
    assert_eq!(
        counter.load(Ordering::SeqCst),
        0,
        "host cua mot anh KHONG GIU (tu nhien) khong duoc vao tang 2 -- 0 ket noi"
    );
    assert_eq!(opened.images_saved, 0);
    assert_eq!(opened.images_failed, 0, "anh khong GIU khong duoc THU tai, nen khong dem la mot lan THAT BAI");
    assert!(read_asset_rows(&opened.store).is_empty());

    drop(opened);
    cleanup(&root);
}

// ═════════════════════════════════════════════════════════════════════════════════
// C5 (vòng rà đối kháng 3 lớp) — ảnh ở CUỐI Chương, và NHIỀU ảnh trong MỘT Chương (hình dạng
// phổ biến nhất trên trang thật). Task list spec 6.11 đòi đủ đầu/giữa/cuối; "giữa" đã có ở
// `a_kept_image_gets_a_real_file_on_disk_and_an_asset_row_with_the_right_anchor`.
// ═════════════════════════════════════════════════════════════════════════════════

#[test]
fn an_image_at_the_end_of_the_chapter_anchors_after_every_segment() {
    let root = temp_dir("image-at-the-end");
    let body: &'static [u8] = b"anh-cuoi-chuong";
    let (port, counter, _handle) = spawn_counting_image_server(4, body, "image/jpeg");
    let img_url = format!("http://127.0.0.1:{port}/anh-cuoi.jpg");

    // <h1> + BA doan van, anh o SAU CUNG -- khong doan nao dung sau no.
    let html = format!(
        "<html><head><title>Bai viet</title></head><body><article><h1>Tieu de</h1>\
         <p>Doan mot co du chu de duoc Readability chon lam noi dung chinh cua trang, \
         nhieu chu hon de vuot nguong do dai toi thieu.</p>\
         <p>Doan hai tiep tuc noi dung that su cua bai viet, khong phai menu hay quang cao, \
         du dai de dom_smoothie cham diem cao cho khoi nay.</p>\
         <p>Doan ba dong y nghia, giu cho tong do dai van ban vuot qua nguong toi thieu can \
         thiet de Readability tin day la mot bai viet that.</p>\
         <img src=\"{img_url}\">\
         </article></body></html>"
    );
    let items = vec![url_item("https://example.test/bai-anh-cuoi", html)];
    let shape = chapters_shape_if_all_ok(&items).expect("danh sach toan muc OK");

    let opened = create_work(
        &root,
        "Anh Cuoi Chuong",
        "en",
        "",
        shape,
        encoding_rs::UTF_8,
        Vec::new(),
        None,
        Vec::new(),
        &DomainLogState::new(Vec::new()),
        None,
    )
    .expect("tao Tac pham voi anh o cuoi Chuong that bai");

    assert_eq!(opened.images_saved, 1);
    assert_eq!(counter.load(Ordering::SeqCst), 1);
    let rows = read_asset_rows(&opened.store);
    assert_eq!(rows.len(), 1, "dung mot hang asset: {rows:?}");

    let segment_count: i64 = opened
        .store
        .read(|conn| conn.query_row("SELECT COUNT(*) FROM segment WHERE chapter_id = ?1", [opened.chapter_id], |r| r.get(0)))
        .expect("dem segment that bai");
    assert_eq!(
        rows[0].3, segment_count,
        "anh o CUOI Chuong phai neo SAU segment CUOI CUNG -- anchor phai bang TRON so segment"
    );

    drop(opened);
    cleanup(&root);
}

#[test]
fn two_images_in_one_chapter_each_get_their_own_file_and_their_own_correctly_ordered_anchor() {
    let root = temp_dir("two-images-one-chapter");
    let body1: &'static [u8] = b"anh-thu-nhat";
    let body2: &'static [u8] = b"anh-thu-hai-khac-noi-dung";
    let (port1, counter1, _h1) = spawn_counting_image_server(4, body1, "image/jpeg");
    let (port2, counter2, _h2) = spawn_counting_image_server(4, body2, "image/png");
    let img1 = format!("http://127.0.0.1:{port1}/anh-mot.jpg");
    let img2 = format!("http://127.0.0.1:{port2}/anh-hai.png");

    // <h1> + Doan mot, ANH THU NHAT, Doan hai, ANH THU HAI, Doan ba -- hinh dang PHO BIEN
    // NHAT tren mot trang that (nhieu anh minh hoa xen giua cac doan).
    let html = format!(
        "<html><head><title>Bai viet</title></head><body><article><h1>Tieu de</h1>\
         <p>Doan mot co du chu de duoc Readability chon lam noi dung chinh cua trang, \
         nhieu chu hon de vuot nguong do dai toi thieu.</p>\
         <img src=\"{img1}\">\
         <p>Doan hai tiep tuc noi dung that su cua bai viet, khong phai menu hay quang cao, \
         du dai de dom_smoothie cham diem cao cho khoi nay.</p>\
         <img src=\"{img2}\">\
         <p>Doan ba dong y nghia, giu cho tong do dai van ban vuot qua nguong toi thieu can \
         thiet de Readability tin day la mot bai viet that.</p>\
         </article></body></html>"
    );
    let items = vec![url_item("https://example.test/bai-hai-anh", html)];
    let shape = chapters_shape_if_all_ok(&items).expect("danh sach toan muc OK");

    let opened = create_work(
        &root,
        "Hai Anh Mot Chuong",
        "en",
        "",
        shape,
        encoding_rs::UTF_8,
        Vec::new(),
        None,
        Vec::new(),
        &DomainLogState::new(Vec::new()),
        None,
    )
    .expect("tao Tac pham voi hai anh trong mot Chuong that bai");

    assert_eq!(opened.images_saved, 2, "ca hai anh deu phai duoc luu");
    assert_eq!(opened.images_failed, 0);
    assert_eq!(counter1.load(Ordering::SeqCst), 1, "anh thu nhat: dung mot lan tai");
    assert_eq!(counter2.load(Ordering::SeqCst), 1, "anh thu hai: dung mot lan tai");

    let rows = read_asset_rows(&opened.store);
    assert_eq!(rows.len(), 2, "dung hai hang asset, moi anh mot hang: {rows:?}");
    assert_ne!(rows[0].1, rows[1].1, "hai anh KHAC NOI DUNG phai cho hai file_name KHAC nhau");
    assert!(
        rows.iter().any(|r| r.2.as_deref() == Some(img1.as_str())),
        "phai co mot hang mang source_url la anh THU NHAT: {rows:?}"
    );
    assert!(
        rows.iter().any(|r| r.2.as_deref() == Some(img2.as_str())),
        "phai co mot hang mang source_url la anh THU HAI: {rows:?}"
    );

    // Neo phai TANG DAN theo dung thu tu tai lieu -- anh thu hai dung SAU anh thu nhat trong
    // trang, nen neo cua no phai LON HON (nhieu segment hon dung truoc no).
    let anchor_for =
        |url: &str| rows.iter().find(|r| r.2.as_deref() == Some(url)).map(|r| r.3).expect("hang phai co mat");
    assert!(
        anchor_for(&img1) < anchor_for(&img2),
        "anh thu nhat (dung TRUOC trong tai lieu) phai co neo NHO HON anh thu hai: {:?} >= {:?}",
        anchor_for(&img1),
        anchor_for(&img2)
    );

    drop(opened);
    cleanup(&root);
}

// ═════════════════════════════════════════════════════════════════════════════════
// M6 (vòng rà đối kháng 2, lớp 3) — ca ĐẦU-CUỐI: một ẢNH NHẬP THẬT (đi qua trọn pha ảnh của
// `create_work`, không phải `insert_asset_directly` gieo tay) phải SỐNG SÓT nguyên vẹn qua
// một lượt tổ chức lại Chương KHÔNG liên quan tới nó.
// ═════════════════════════════════════════════════════════════════════════════════

#[test]
fn a_real_imported_image_survives_a_chapter_reorganisation_of_an_unrelated_chapter_byte_for_byte() {
    let root = temp_dir("m6-end-to-end-reorg");
    let body1: &'static [u8] = b"anh-chuong-mot-that";
    let body2: &'static [u8] = b"anh-chuong-hai-that-khac-noi-dung";
    let (port1, _c1, _h1) = spawn_counting_image_server(4, body1, "image/jpeg");
    let (port2, _c2, _h2) = spawn_counting_image_server(4, body2, "image/png");
    let img1 = format!("http://127.0.0.1:{port1}/anh1.jpg");
    let img2 = format!("http://127.0.0.1:{port2}/anh2.png");

    let items = vec![
        url_item("https://example.test/chuong-mot", html_page_with_one_image(&img1)),
        url_item("https://example.test/chuong-hai", html_page_with_one_image(&img2)),
    ];
    let shape = chapters_shape_if_all_ok(&items).expect("danh sach toan muc OK");

    let opened = create_work(
        &root,
        "M6 Dau Cuoi",
        "en",
        "",
        shape,
        encoding_rs::UTF_8,
        Vec::new(),
        None,
        Vec::new(),
        &DomainLogState::new(Vec::new()),
        None,
    )
    .expect("tao Tac pham voi hai Chuong, moi Chuong mot anh that bai");

    assert_eq!(opened.images_saved, 2, "ca hai Chuong deu phai co anh duoc luu THAT");

    let chapter_ids: Vec<i64> = opened
        .store
        .read(|conn| {
            let mut stmt = conn.prepare("SELECT id FROM chapter ORDER BY ord")?;
            let rows = stmt.query_map([], |row| row.get(0))?;
            rows.collect::<auratranslate_lib::core::store::SqlResult<Vec<i64>>>()
        })
        .expect("doc danh sach Chuong that bai");
    assert_eq!(chapter_ids.len(), 2, "phai co dung hai Chuong: {chapter_ids:?}");
    let (chapter1, chapter2) = (chapter_ids[0], chapter_ids[1]);

    // Chụp NGUYÊN VẸN hàng asset THẬT của Chương 2 (ảnh KHÔNG liên quan) trước lượt tách.
    let before_c2 = read_asset_rows(&opened.store).into_iter().find(|r| r.0 == chapter2).expect("asset that cua Chuong 2 truoc");

    // Tách Chương 1 tại segment ord=2 (segment thu hai) cua no.
    let seg_in_c1: i64 = opened
        .store
        .read(|conn| {
            conn.query_row(
                "SELECT id FROM segment WHERE chapter_id = ?1 AND ord = 2",
                [chapter1],
                |row| row.get(0),
            )
        })
        .expect("doc segment ord=2 cua Chuong 1 that bai");

    let mut opened = opened;
    auratranslate_lib::commands::chapter::split_chapter_at_segment(Some(&mut opened), seg_in_c1)
        .expect("tach Chuong 1 that bai");

    let after_c2 = read_asset_rows(&opened.store).into_iter().find(|r| r.0 == chapter2).expect("asset that cua Chuong 2 sau");
    assert_eq!(
        after_c2, before_c2,
        "hang asset THAT cua Chuong 2 (khong lien quan toi lan tach o Chuong 1) phai giu \
         NGUYEN TUNG BYTE (chapter_id, file_name, source_url, anchor, byte_len, content_type) \
         -- truoc: {before_c2:?}, sau: {after_c2:?}"
    );

    drop(opened);
    cleanup(&root);
}
