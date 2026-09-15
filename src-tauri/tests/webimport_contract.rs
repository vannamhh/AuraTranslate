//! Cổng HỢP ĐỒNG của Story 6.7 + 6.8 — hành vi thật của `Fetcher`/`Extractor`/`Allowlist`,
//! cộng đường "N link ⇒ N Chương" ở tầng `commands::project`. Chép `spawn_once` từ bàn đo
//! 6.1 (`webimport_probe.rs`) — server HTTP thô tự dựng, không framework, không mạng ngoài.
//!
//! Bảy ca THẬT của Story 6.7 (không `#[ignore]`), đúng Task list spec 6.7:
//! - chặn chuyển hướng khác host (server bị chặn nhận **0** kết nối)
//! - cắt theo dòng chảy (`CAP ≤ đọc ≪ quảng cáo`, không một con số)
//! - lỗi kết nối phân loại đúng
//! - bóc ra văn bản **không chứa** `<` của thẻ
//! - trang bóc rỗng thành mục hỏng
//! - N link giữ đúng thứ tự
//! - mục hỏng giữ đúng **vị trí**
//!
//! 🔵 **Story 6.8 — bốn ca AD-41 bắt buộc (spine `:542`) cộng ca tầng 2**, ở cuối tệp. Ca
//! "chặn chuyển hướng khác host" của 6.7 (`a_cross_host_redirect_is_blocked...`) đổi TÊN và
//! MỆNH ĐỀ — nó xanh nhờ *"không trong allowlist"*, không còn nhờ *"khác host"* (§Design
//! Notes spec 6.8) — và một ca MỚI phủ chiều ngược lại (hai host CÙNG tầng 1 ⇒ theo được).

use std::fs;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

use auratranslate_lib::commands::project::{
    UrlImportItem, chapters_shape_for_view, chapters_shape_if_all_ok, create_work, fetch_url_import_items,
    preview_import_encoding,
};
use auratranslate_lib::core::segment::import::{ImportError, web_import_item_failure_ipc_error};
use auratranslate_lib::core::segment::chapterpattern::ChapterPattern;
use auratranslate_lib::core::segment::pipeline::{ChapterInput, PipelineInput, PipelineShape, run_import};
use auratranslate_lib::core::webimport::{
    Allowlist, BlockBody, DomainLogDecision, DomainLogOutcome, FetchError, ResourceKind,
    WebImportItemFailureReason, extract, fetch, looks_like_html,
};

/// Server tối giản: chấp nhận ĐÚNG MỘT kết nối, đọc và bỏ qua request, gọi `respond` để viết
/// response thô, rồi tự thoát luồng. Trả về cổng đã cấp và `JoinHandle`. Chép nguyên khuôn
/// `webimport_probe.rs::spawn_once`.
fn spawn_once<F>(respond: F) -> (u16, thread::JoinHandle<()>)
where
    F: FnOnce(TcpStream) + Send + 'static,
{
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind cổng tạm");
    let port = listener.local_addr().expect("local_addr").port();
    let handle = thread::spawn(move || {
        if let Ok((mut stream, _)) = listener.accept() {
            let _ = stream.set_read_timeout(Some(Duration::from_secs(5)));
            let mut discard = [0u8; 4096];
            let _ = stream.read(&mut discard);
            respond(stream);
        }
    });
    (port, handle)
}

fn html_page_with_paragraphs() -> String {
    "<html><head><title>Bai viet</title></head><body><article><h1>Tieu de</h1>\
     <p>Doan mot co du chu de duoc Readability chon lam noi dung chinh cua trang, \
     nhieu chu hon de vuot nguong do dai toi thieu.</p>\
     <p>Doan hai tiep tuc noi dung that su cua bai viet, khong phai menu hay quang cao, \
     du dai de dom_smoothie cham diem cao cho khoi nay.</p>\
     <p>Doan ba dong y nghia, giu cho tong do dai van ban vuot qua nguong toi thieu can \
     thiet de Readability tin day la mot bai viet that.</p>\
     </article></body></html>"
        .to_owned()
}

fn ok_html_response(body: &str) -> String {
    format!(
        "HTTP/1.1 200 OK\r\nContent-Type: text/html; charset=utf-8\r\nContent-Length: {}\r\n\r\n{}",
        body.len(),
        body
    )
}

// ═════════════════════════════════════════════════════════════════════════════════
// Ca 1 (Story 6.8: MỆNH ĐỀ ĐỔI) — chặn chuyển hướng ra host NGOÀI allowlist, 0 kết nối
// ═════════════════════════════════════════════════════════════════════════════════
//
// 🔵 **Story 6.8 — tên cũ `a_cross_host_redirect_is_blocked_and_...`, mệnh đề đổi từ "khác
// host" sang "không trong allowlist" (§Design Notes spec 6.8).** `127.0.0.1` và `localhost`
// là HAI CHUỖI HOST KHÁC NHAU theo `Url::host_str()`, dù cùng trỏ về loopback — đủ để mô
// phỏng "host ngoài allowlist" thật mà không cần DNS/mạng ngoài. `allowlist` ở đây chỉ chứa
// host của `port_a` (URL gốc) — `localhost` (đích chuyển hướng) không hề có mặt, đúng ca
// "ngoài allowlist", KHÔNG chỉ "khác host" (xem ca kế tiếp: hai host khác nhau NHƯNG cùng
// allowlist thì ĐƯỢC theo).
#[test]
fn a_redirect_to_a_host_outside_the_allowlist_is_blocked_and_the_target_host_receives_zero_connections() {
    let reached_b = Arc::new(AtomicUsize::new(0));
    let reached_b_clone = Arc::clone(&reached_b);
    let (port_b, _handle_b) = spawn_once(move |mut stream| {
        reached_b_clone.fetch_add(1, Ordering::SeqCst);
        let _ = stream.write_all(ok_html_response("hi").as_bytes());
    });

    let location = format!("http://localhost:{port_b}/final");
    let location_for_server = location.clone();
    let (port_a, _handle_a) = spawn_once(move |mut stream| {
        let body = format!(
            "HTTP/1.1 301 Moved Permanently\r\nLocation: {location_for_server}\r\nContent-Length: 0\r\n\r\n"
        );
        let _ = stream.write_all(body.as_bytes());
    });

    let start_url = format!("http://127.0.0.1:{port_a}/start");
    let allowlist = Allowlist::from_urls([start_url.as_str()]);
    let (result, _log) = fetch(&start_url, &allowlist, ResourceKind::Page);

    assert!(
        matches!(result, Err(FetchError::NotAllowlisted)),
        "kỳ vọng `NotAllowlisted`, nhận: {result:?}"
    );
    assert_eq!(
        reached_b.load(Ordering::SeqCst),
        0,
        "server đích của chuyển hướng bị chặn phải nhận ĐÚNG 0 kết nối"
    );
}

/// **MỚI (Story 6.8)** — chiều NGƯỢC LẠI của ca trên: hai host KHÁC NHAU nhưng CÙNG có mặt
/// trong allowlist (cả hai đều được coi là "URL người dùng đã dán" ở đây) ⇒ chuyển hướng
/// được PHÉP THEO. Đóng đúng khoảng trống mà §Design Notes spec 6.8 nêu — trước bản vá,
/// KHÔNG ca nào phủ chiều này (mọi chuyển hướng khác host đều bị chặn tuyệt đối).
#[test]
fn a_redirect_between_two_hosts_both_inside_the_allowlist_is_followed() {
    let (port_b, _handle_b) = spawn_once(move |mut stream| {
        let _ = stream.write_all(ok_html_response("noi dung that o host B").as_bytes());
    });
    let location = format!("http://localhost:{port_b}/final");
    let location_for_server = location.clone();
    let (port_a, _handle_a) = spawn_once(move |mut stream| {
        let body = format!(
            "HTTP/1.1 301 Moved Permanently\r\nLocation: {location_for_server}\r\nContent-Length: 0\r\n\r\n"
        );
        let _ = stream.write_all(body.as_bytes());
    });

    let start_url = format!("http://127.0.0.1:{port_a}/start");
    // Cả HAI host — `127.0.0.1` (gốc) VÀ `localhost` (đích chuyển hướng) — đều trong danh
    // sách dán, đúng khuôn "hai host tầng 1" của I/O Matrix spec 6.8.
    let allowlist = Allowlist::from_urls([start_url.as_str(), location.as_str()]);
    let (result, _log) = fetch(&start_url, &allowlist, ResourceKind::Page);

    let page = result.expect("hai host cung allowlist -- chuyen huong phai duoc theo");
    assert!(
        String::from_utf8_lossy(&page.bytes).contains("noi dung that o host B"),
        "phai nhan duoc noi dung THAT tu host B, khong dung lai o response 301 cua host A"
    );
}

// ═════════════════════════════════════════════════════════════════════════════════
// Ca 2 — cắt theo dòng chảy: CAP ≤ đọc ≪ quảng cáo, không một con số cụ thể
// ═════════════════════════════════════════════════════════════════════════════════

#[test]
fn a_response_advertising_far_more_than_the_cap_is_cut_off_mid_stream() {
    // 200 MiB quảng cáo — gấp 10 lần trần sản phẩm (20 MiB, `webimport::MAX_RESPONSE_BYTES`),
    // đủ lớn để "đọc dừng sớm" là kết luận không thể chối cãi từ số byte server SẢN XUẤT được
    // trước khi client đóng kết nối.
    const ADVERTISED_LEN: usize = 200 * 1024 * 1024;

    let sent_total = Arc::new(AtomicUsize::new(0));
    let sent_clone = Arc::clone(&sent_total);
    let (port, _handle) = spawn_once(move |mut stream| {
        let _ = stream.set_write_timeout(Some(Duration::from_secs(10)));
        let header = format!(
            "HTTP/1.1 200 OK\r\nContent-Type: text/html\r\nContent-Length: {ADVERTISED_LEN}\r\n\r\n"
        );
        if stream.write_all(header.as_bytes()).is_err() {
            return;
        }
        let chunk = vec![b'x'; 256 * 1024];
        let mut sent = 0usize;
        while sent < ADVERTISED_LEN {
            let remaining = ADVERTISED_LEN - sent;
            let this_write = remaining.min(chunk.len());
            if stream.write_all(&chunk[..this_write]).is_err() {
                // Client đã đóng kết nối sớm — kết quả ĐÚNG, không phải lỗi hạ tầng.
                break;
            }
            sent += this_write;
            sent_clone.store(sent, Ordering::SeqCst);
        }
    });

    let url = format!("http://127.0.0.1:{port}/big");
    let allowlist = Allowlist::from_urls([url.as_str()]);
    let (result, _log) = fetch(&url, &allowlist, ResourceKind::Page);
    assert!(matches!(result, Err(FetchError::TooLarge)), "kỳ vọng `TooLarge`, nhận: {result:?}");

    // Chờ luồng server ghi nốt vài khối cuối (đóng kết nối phía client không đồng bộ tức thì
    // với việc server nhận biết broken pipe) — đọc lại `sent_total` sau một khoảng ngắn.
    thread::sleep(Duration::from_millis(200));
    let produced = sent_total.load(Ordering::SeqCst);

    assert!(
        produced < ADVERTISED_LEN,
        "server đã sản xuất HẾT {ADVERTISED_LEN} byte quảng cáo — client không hề dừng sớm"
    );
    // Bất biến nghiệm thu là "dừng ngay sau trần", không một con số cụ thể (phụ thuộc cỡ gói
    // TCP/độ trễ loopback của máy đang chạy — xem `webimport_probe.rs::size_cap_case`). Biên
    // dưới rộng rãi: server không có lý do gì để dừng ở dưới một khối 256 KiB.
    assert!(produced > 0, "server chưa kịp gửi byte nào — phép đo hạ tầng vô nghĩa");
}

// ═════════════════════════════════════════════════════════════════════════════════
// Ca 3 — lỗi kết nối phân loại đúng
// ═════════════════════════════════════════════════════════════════════════════════

#[test]
fn a_dead_connection_is_classified_as_connect_failed_not_some_other_error() {
    const MAX_ATTEMPTS: usize = 5;
    for _ in 0..MAX_ATTEMPTS {
        let port = {
            let listener = TcpListener::bind("127.0.0.1:0").expect("bind cổng tạm");
            listener.local_addr().expect("local_addr").port()
        };
        let url = format!("http://127.0.0.1:{port}/nope");
        let allowlist = Allowlist::from_urls([url.as_str()]);
        match fetch(&url, &allowlist, ResourceKind::Page).0 {
            Err(FetchError::ConnectFailed { .. }) => return,
            Err(FetchError::Timeout { .. }) => return, // hệ điều hành có thể trả timeout thay vì refused
            other => {
                // TOCTOU thật (ai đó vừa bind đúng cổng) — thử cổng khác thay vì báo đỏ oan.
                eprintln!("connect_failed retry: {other:?}");
                continue;
            }
        }
    }
    panic!("{MAX_ATTEMPTS} lần thử liên tiếp đều không cho kết quả phân loại đúng — TOCTOU thật");
}

// ═════════════════════════════════════════════════════════════════════════════════
// Ca 4 — bóc ra văn bản KHÔNG chứa `<` của thẻ
// ═════════════════════════════════════════════════════════════════════════════════

/// 🔴 **SỬA 2026-09-07 (Story 6.9) — mệnh đề đổi nghĩa khi `extract` trả một mô hình KHỐI,
/// không còn một `String` phẳng.** Không nhánh `BlockBody` nào (kể cả `Image`, kể cả một khối
/// `machine_kept == false` — "ornament" cũng đi qua đúng dây, đó là điểm của story này) được
/// mang `<` của thẻ HTML gốc — AD-16 §Rule mục 1/2 áp cho TOÀN mô hình, không riêng phần
/// "giữ".
#[test]
fn extracted_text_never_contains_an_angle_bracket_from_the_source_markup() {
    let html = "<html><head><title>Bai viet</title></head><body>\
        <nav><a href=\"/menu\">Menu</a></nav>\
        <article><h1>Tieu de bai viet</h1>\
        <p>Doan mot co du chu de duoc Readability chon lam noi dung chinh cua trang, \
        nhieu chu hon de vuot nguong do dai toi thieu can thiet.</p>\
        <p>Doan hai tiep tuc noi dung that su cua bai viet, khong phai menu hay quang cao, \
        du dai de dom_smoothie cham diem cao cho khoi nay mot cach ro rang.</p>\
        </article>\
        <aside><p>Binh luan cua doc gia, khong lien quan noi dung bai viet chinh, day chi la \
        rac quanh bai de kiem tra bo loc co loai duoc no khong.</p></aside>\
        </body></html>"
        .to_owned();
    let blocks = extract(&html, "https://example.com/bai-viet").expect("bóc thành công");
    assert!(!blocks.is_empty(), "trang có nội dung thật phải cho ít nhất một khối");

    let mut saw_kept_text = false;
    for block in &blocks {
        match &block.body {
            BlockBody::Paragraph(text) | BlockBody::Caption(text) => {
                assert!(!text.contains('<'), "thân khối không được chứa `<` của thẻ HTML: {text:?}");
                if block.machine_kept {
                    saw_kept_text = true;
                }
            }
            BlockBody::Image { src, alt } => {
                if let Some(src) = src {
                    assert!(!src.contains('<'), "src ảnh không được chứa `<`: {src:?}");
                }
                if let Some(alt) = alt {
                    assert!(!alt.contains('<'), "alt ảnh không được chứa `<`: {alt:?}");
                }
            }
        }
    }
    assert!(saw_kept_text, "phải có ít nhất một khối `machine_kept` mang nội dung thật");
    assert!(
        blocks.iter().any(|b| !b.machine_kept),
        "trang có `<nav>`/`<aside>` phải cho ít nhất một khối bị loại (ornament) — nếu không, \
         ca này không kiểm được điều nó tuyên bố kiểm (phủ cả nhánh ornament)"
    );
}

// ═════════════════════════════════════════════════════════════════════════════════
// Ca 5 — trang bóc ra rỗng thành mục hỏng
// ═════════════════════════════════════════════════════════════════════════════════

#[test]
fn a_page_with_no_extractable_content_fails_extraction_instead_of_falling_back_to_raw_html() {
    // Trang gần như trống — không đủ để `dom_smoothie` chấm điểm ra một khối nội dung nào.
    let html = "<html><head><title>Trong</title></head><body></body></html>".to_owned();
    let result = extract(&html, "https://example.com/trong");
    assert!(
        result.is_err(),
        "một trang không bóc được nội dung chính phải trả `Err`, không rơi về Ok(rỗng) hay HTML thô"
    );
}

// ═════════════════════════════════════════════════════════════════════════════════
// Ca 6 + 7 — N link giữ đúng thứ tự; mục hỏng giữ đúng VỊ TRÍ
// ═════════════════════════════════════════════════════════════════════════════════

#[test]
fn n_links_are_fetched_sequentially_and_a_broken_item_keeps_its_position() {
    let (port_ok_1, _h1) = spawn_once(|mut stream| {
        let _ = stream.write_all(ok_html_response(&html_page_with_paragraphs()).as_bytes());
    });
    // Cổng KHÔNG ai lắng nghe — mục thứ hai (giữa danh sách) hỏng có chủ ý.
    let dead_port = {
        let listener = TcpListener::bind("127.0.0.1:0").expect("bind cổng tạm");
        listener.local_addr().expect("local_addr").port()
    };
    let (port_ok_2, _h2) = spawn_once(|mut stream| {
        let _ = stream.write_all(ok_html_response(&html_page_with_paragraphs()).as_bytes());
    });

    let urls = vec![
        format!("http://127.0.0.1:{port_ok_1}/a"),
        format!("http://127.0.0.1:{dead_port}/b"),
        format!("http://127.0.0.1:{port_ok_2}/c"),
    ];
    let (items, _log) = fetch_url_import_items(urls.clone());

    assert_eq!(items.len(), 3, "phải giữ đúng N mục, không rơi rớt cái nào");
    for (i, item) in items.iter().enumerate() {
        assert_eq!(item.url, urls[i], "URL ở vị trí {i} phải khớp đúng thứ tự đã dán");
    }
    assert!(items[0].error.is_none() && items[0].raw.is_some(), "mục 1 (tốt) phải thành công");
    assert!(
        items[1].error.is_some() && items[1].raw.is_none(),
        "mục 2 (cổng chết) phải thất bại, GIỮ VỊ TRÍ thứ hai — không bị đẩy xuống cuối"
    );
    assert!(items[2].error.is_none() && items[2].raw.is_some(), "mục 3 (tốt) phải thành công");
}

/// **THÊM (Story 6.10a)** — 5 link, link #3 (index 2) hỏng ⇒ màn xem trước vẫn DỰNG ĐƯỢC với
/// 4 Chương (vị từ XEM, `chapters_shape_for_view`), mục #3 GIỮ CHỖ TẠI VỊ TRÍ 3, VÀ nút xác
/// nhận vẫn KHOÁ (vị từ GHI, `chapters_shape_if_all_ok` vẫn trả `None`) — hai khẳng định
/// trong MỘT ca, đúng khuyến cáo Task list spec 6.10a: đây là chỗ dễ trộn hai vị từ nhất.
///
/// 🔴 Đối chứng đỏ ② của §Verification spec 6.10a: cho `chapters_shape_for_view` BỎ QUA mục
/// hỏng (đã đúng, không đổi) **và** cho `chapters_shape_if_all_ok` CŨNG bỏ qua mục hỏng (một
/// đột biến giả định trộn hai vị từ) ⇒ assert `chapters_shape_if_all_ok(&items).is_none()`
/// dưới đây phải ĐỎ. Nếu nó XANH trên một sản phẩm đã trộn, hai vị từ đang bị lẫn.
#[test]
fn a_broken_item_at_position_three_still_lets_the_other_four_chapters_preview_while_the_write_predicate_stays_locked()
 {
    let (port_ok_1, _h1) = spawn_once(|mut stream| {
        let _ = stream.write_all(ok_html_response(&html_page_with_paragraphs()).as_bytes());
    });
    let (port_ok_2, _h2) = spawn_once(|mut stream| {
        let _ = stream.write_all(ok_html_response(&html_page_with_paragraphs()).as_bytes());
    });
    // Cổng KHÔNG ai lắng nghe — mục thứ BA (index 2, "#3" 1-based) hỏng có chủ ý.
    let dead_port = {
        let listener = TcpListener::bind("127.0.0.1:0").expect("bind cổng tạm");
        listener.local_addr().expect("local_addr").port()
    };
    let (port_ok_3, _h3) = spawn_once(|mut stream| {
        let _ = stream.write_all(ok_html_response(&html_page_with_paragraphs()).as_bytes());
    });
    let (port_ok_4, _h4) = spawn_once(|mut stream| {
        let _ = stream.write_all(ok_html_response(&html_page_with_paragraphs()).as_bytes());
    });

    let urls = vec![
        format!("http://127.0.0.1:{port_ok_1}/a"),
        format!("http://127.0.0.1:{port_ok_2}/b"),
        format!("http://127.0.0.1:{dead_port}/c"),
        format!("http://127.0.0.1:{port_ok_3}/d"),
        format!("http://127.0.0.1:{port_ok_4}/e"),
    ];
    let (items, _log) = fetch_url_import_items(urls);
    assert_eq!(items.len(), 5, "phải giữ đúng 5 mục");
    assert!(items[2].error.is_some(), "mục #3 (cổng chết) phải hỏng");
    for i in [0usize, 1, 3, 4] {
        assert!(items[i].error.is_none() && items[i].raw.is_some(), "mục {i} phải OK");
    }

    // Vị từ GHI — KHÔNG đổi, vẫn khoá vì còn MỘT mục hỏng.
    assert!(
        chapters_shape_if_all_ok(&items).is_none(),
        "chapters_shape_if_all_ok (vị từ GHI) phải vẫn trả None khi còn mục #3 hỏng -- nút \
         xác nhận phải KHOÁ"
    );

    // Vị từ XEM — MỚI, bỏ qua mục hỏng, dựng được 4 Chương từ 4 mục OK.
    let view_shape = chapters_shape_for_view(&items)
        .expect("chapters_shape_for_view (vị từ XEM) phải dựng được từ 4 mục OK còn lại");
    let preview = preview_import_encoding(&view_shape, "en", &[], None, &[], 0, &[]);
    assert!(!preview.candidates.is_empty(), "còn byte OK để dò -- dải ứng viên không được rỗng");
    let chapters_summary = preview.candidates[0]
        .chapters
        .as_ref()
        .expect("ứng viên đầu phải mang khối tách Chương (tầng 4)");
    assert_eq!(
        chapters_summary.chapter_count, 4,
        "màn xem trước phải hiện ĐỦ 4 Chương từ 4 mục OK, không đợi mục #5 (thứ năm, thật ra \
         là mục hỏng thứ ba) sạch mới có gì để xem"
    );
}

/// Danh sách rỗng/toàn dòng trắng ⇒ 0 mục (đóng vế I/O Matrix "Danh sách rỗng").
#[test]
fn blank_and_empty_lines_are_dropped_before_counting_and_never_produce_an_item() {
    let (items, _log) = fetch_url_import_items(vec!["   ".to_owned(), "".to_owned(), "\t".to_owned()]);
    assert!(items.is_empty(), "dòng rỗng/toàn khoảng trắng không được sinh ra một mục nào");
}

/// Dòng RÁC (không rỗng, không phải URL hợp lệ) ⇒ MỘT mục hỏng, **0 lời gọi mạng** cho dòng
/// đó — không có server nào lắng nghe ở cổng của test này, và test không dựng server nào cả:
/// nếu `fetch_url_import_items` cố kết nối, nó sẽ trả một lỗi mạng (không phải `InvalidUrl`)
/// hoặc treo — cả hai đều làm assert dưới đây SAI.
#[test]
fn a_garbage_line_becomes_one_broken_item_with_zero_network_calls() {
    let (items, log) = fetch_url_import_items(vec!["day khong phai url".to_owned()]);
    assert_eq!(items.len(), 1);
    assert!(items[0].error.is_some(), "dòng rác phải thành một mục hỏng");
    assert!(items[0].raw.is_none());

    // R1 (vòng rà đối kháng 3, lớp 3) — 0 kết nối mạng KHÔNG được nghĩa là 0 hàng KIỂM TOÁN.
    // Trước khi vá, `fetch()` trả `log` RỖNG cho một URL không phân giải được — một lượt
    // trượt không để lại DẤU VẾT nào, phá đúng §Always "kể cả lượt trượt" (`domain_log.rs`).
    assert_eq!(log.len(), 1, "mot URL khong phan giai duoc van phai de lai DUNG MOT hang kiem toan: {log:?}");
    assert_eq!(log[0].decision, DomainLogDecision::Denied, "0 ket noi -- dung hinh dang Denied");
    assert_eq!(
        log[0].outcome,
        Some(DomainLogOutcome::Other),
        "ly do trượt KHONG PHAI chinh sach allowlist -- Other phan biet no khoi mot Denied \
         THAT (bi chinh sach chan)"
    );
}

// ═════════════════════════════════════════════════════════════════════════════════
// ĐO HIỆU NĂNG — N link thật qua server cục bộ (Task list spec 6.7, "ĐO hiệu năng")
// ═════════════════════════════════════════════════════════════════════════════════
//
// Mốc TRƯỚC story (Story 6.6): 5 ứng viên × 2.000 Chương ~242-286 ms (`project.rs:1288-1300`,
// debug) — đo trên đường KHÔNG bóc nội dung chính (`extract_main_content = false`, mọi hình
// dạng trước Story 6.7 luôn đi qua `Blob`). Suy tuyến tính từ số đó sang đường URL bị CẤM
// (Ice, 2026-09-05) vì bóc nội dung chính là một chi phí MỚI, không có mặt trong mốc cũ.
//
// 🔴 Đo THẲNG dưới đây, N = 20 (đủ nhanh để không kéo `cargo test --locked` toàn cục, đủ lớn
// để không phải một điểm dữ liệu duy nhất). Server cục bộ (`spawn_once` lặp lại N lần trong
// N luồng riêng) — không phụ thuộc mạng ngoài, lặp lại được.
#[test]
fn perf_probe_twenty_links_end_to_end_fetch_plus_extract_plus_pipeline() {
    const N: usize = 20;

    fn html_page(i: usize) -> String {
        let mut body = String::new();
        body.push_str(&format!("<html><head><title>Bai {i}</title></head><body><article><h1>Tieu de {i}</h1>"));
        for p in 0..40 {
            body.push_str(&format!(
                "<p>Doan thu {p} cua bai {i}, du dai de vuot qua nguong toi thieu can thiet cho \
                 mot khoi noi dung duoc cham diem cao, khong phai menu hay quang cao xung quanh no.</p>"
            ));
        }
        body.push_str("</article></body></html>");
        body
    }

    let mut ports = Vec::with_capacity(N);
    for i in 0..N {
        let (port, _handle) = spawn_once(move |mut stream| {
            let _ = stream.write_all(ok_html_response(&html_page(i)).as_bytes());
        });
        ports.push(port);
    }
    // ⚠️ Đo được (2026-09-06): mục ĐẦU TIÊN của một loạt `spawn_once` liên tiếp trượt
    // `ConnectFailed` MỘT CÁCH TẤT ĐỊNH trên máy đo này — dù `TcpListener::bind` đã đặt socket
    // vào trạng thái LISTEN trước khi `thread::spawn` chạy (hàng đợi backlog của kernel đáng lẽ
    // đủ để nhận một `connect()` dù `.accept()` CHƯA được luồng server gọi tới). Nghi nhiều luồng
    // server dựng LIÊN TIẾP trong một vòng lặp chặt cần một khoảng ngắn để hệ điều hành THẬT SỰ
    // lên lịch chạy luồng đầu tiên — không phải một khiếm khuyết của `webimport::fetch`
    // (`webimport_boundary.rs`/`a_dead_connection_is_classified...` không dựng NHIỀU server liên
    // tiếp trong MỘT vòng lặp, nên không chạm bẫy này). Một khoảng nghỉ ngắn ở ĐÂY (không sửa
    // `spawn_once` dùng chung — các ca khác không cần) là đủ để đo ổn định, không phải một cách
    // che triệu chứng: bản chất phép đo này là THỜI GIAN của `fetch`+`pipeline`, không phải thời
    // gian dựng hạ tầng test.
    thread::sleep(Duration::from_millis(100));
    let urls: Vec<String> = ports.iter().map(|p| format!("http://127.0.0.1:{p}/a")).collect();

    let t0 = std::time::Instant::now();
    let (items, _log) = fetch_url_import_items(urls);
    let fetch_elapsed = t0.elapsed();
    assert!(items.iter().all(|it| it.error.is_none()), "ca do khong duoc phep co muc hong: {items:?}");

    let shape = chapters_shape_if_all_ok(&items).expect("toan bo OK phai cho Some(shape)");
    let t1 = std::time::Instant::now();
    let input = PipelineInput::default_shaped(shape, "en").with_extract_main_content(true);
    let outcome = run_import(input).expect("pipeline phai chay duoc tren N trang HTML that");
    let pipeline_elapsed = t1.elapsed();

    assert_eq!(outcome.chapters.len(), N);
    println!(
        "PERF_PROBE_URL_IMPORT\tN={N}\tfetch_ms={:.1}\tpipeline_ms={:.1}\ttotal_ms={:.1}",
        fetch_elapsed.as_secs_f64() * 1000.0,
        pipeline_elapsed.as_secs_f64() * 1000.0,
        (fetch_elapsed + pipeline_elapsed).as_secs_f64() * 1000.0,
    );
    // ⚠️ **KHÔNG một con số cụ thể nào ở đây đáng tin** — máy đo phiên này (2026-09-06) đo
    // được `load average` 15 phút > 100 trên 16 lõi (`uptime`), tức MÁY BỊ CHIA SẺ NẶNG với
    // nhiều tiến trình khác, không phải "máy phát triển, không tải nền" như mọi
    // `perf_probe_*` khác của kho (`cleanup_contract.rs`). Ba lượt đo LIÊN TIẾP của CHÍNH
    // N=20 này cho ba con số lệch nhau ~30 LẦN (150 ms · 5.000 ms · 7.000 ms cho `fetch_ms`
    // một mình, cùng một mã, cùng một máy, cách nhau vài phút) — biến thiên đó đến từ TẢI HỆ
    // THỐNG tại thời điểm đo (mỗi lượt `webimport::fetch` dựng một `reqwest::blocking::Client`
    // MỚI, tức một luồng hệ điều hành + một runtime tokio mới cho MỖI link — chi phí khởi tạo
    // đó nhạy với tranh chấp lịch CPU hơn hẳn phần việc thật), KHÔNG đến từ một thay đổi mã
    // nào giữa ba lượt. Bài học AGENTS.md "hai lượt đo phải cùng tải máy" áp dụng NGƯỢC ở
    // đây: máy này không giữ tải ổn định giữa hai lượt, nên không con số ms nào từ phiên đo
    // này được phép chép vào một tài liệu khác như một sự thật về hiệu năng sản phẩm.
    // **Việc bài test này THẬT SỰ nghiệm thu**: N link được tải + bóc + đưa qua chuỗi pipeline
    // ĐÚNG (outcome.chapters.len() == N, không mục nào hỏng) trong một thời gian HỮU HẠN
    // (trần rộng bên dưới chỉ để bắt một treo vô hạn thật, không để khẳng định một tốc độ).
    // Đo lại trên một máy KHÔNG chia sẻ tải trước khi tin bất kỳ con số ms nào ở đây.
    //
    // 🔵 **ĐO LẠI 2026-09-07 — lần này TÁI LẬP ĐƯỢC, nên có số để ghi.** Ba lượt liên tiếp,
    // cùng phiên, `uptime` ngay trước và ngay sau đều cho `load average` 1 phút ≈ **26-28
    // trên 16 lõi**:
    //
    //   | lượt | `fetch_ms` | `pipeline_ms` | `total_ms` |
    //   |------|-----------:|--------------:|-----------:|
    //   | 1    |     4745,5 |         228,4 |     4974,0 |
    //   | 2    |     4857,8 |         221,1 |     5079,0 |
    //   | 3    |     4922,9 |         234,6 |     5157,5 |
    //
    // Chênh lệch giữa ba lượt: **≤ 4% cho `fetch_ms`, ≤ 6% cho `pipeline_ms`** — khác hẳn ~30
    // LẦN của phiên 2026-09-06 ghi ngay trên. ⇒ Con số dùng được, VỚI ĐÚNG hai điều kiện phải
    // đọc kèm, không tách rời:
    //   ① **Máy VẪN không nhàn** (load 26-28/16 lõi, không do lượt đo này gây ra). Đây là cận
    //      TRÊN dưới tải, không phải hiệu năng của một máy rảnh — số thật sẽ THẤP HƠN.
    //   ② `fetch_ms` KHÔNG phải chi phí mạng: server là `127.0.0.1`, và bàn dựng ngủ 100 ms
    //      mỗi luồng trước khi phục vụ, tức ~2.000 ms trong ~4.800 ms là SLEEP CỦA CHÍNH BÀN
    //      ĐO. Phần còn lại chủ yếu là dựng một `reqwest::blocking::Client` MỚI cho mỗi link.
    //      🔴 Đừng chép `fetch_ms` ra ngoài như "tốc độ tải 20 link".
    //
    // Con số ĐÁNG mang đi là **`pipeline_ms` ≈ 221-235 ms cho 20 Chương** đã bóc nội dung —
    // đây là lần đầu chuỗi AD-39 chạy có bước 2 thật. ⚠️ KHÔNG so trực tiếp với mốc trước
    // story (5 ứng viên × 2.000 Chương ~242-286 ms, `project.rs:1288-1300`): hai phép đo khác
    // cả hình dạng đầu vào lẫn số bước, và suy tuyến tính bị CẤM (Ice, 2026-09-05).
    assert!(
        fetch_elapsed.as_secs_f64() < 60.0,
        "fetch {N} link cuc bo mat qua 60s — nghi treo that (khong phai chi tai may), can dieu tra"
    );
}

// ═════════════════════════════════════════════════════════════════════════════════
// BỐN CA BỔ SUNG 2026-09-07 — bốn hàng I/O Matrix đo được là CHƯA có ca nào chạm
// ═════════════════════════════════════════════════════════════════════════════════
//
// Phép đo dẫn tới lượt bổ sung này (2026-09-07, đếm số lần mỗi tên lý do xuất hiện trong
// TOÀN BỘ `src-tauri/tests/**` + `tests/frontend/**`): `Timeout` 24 · `TooLarge` 5 ·
// `RedirectBlocked` 2 · `ConnectFailed` 2 · `InvalidUrl` 1 — nhưng `HttpStatus` **0**,
// `NotHtml` **0**, `ExtractionEmpty` **0**. Ba lý do đó có mã sản phẩm đầy đủ và một khoá
// `MessageKey` riêng, mà không ca nào chạy qua chúng: một bộ test xanh KHÔNG chứng minh chỗ
// nối mới được canh (`AGENTS.md` §Known pitfalls). Hàng "đúng MỘT link" cũng vậy — doc-comment
// `PipelineShape::Chapters` nhấn mạnh "KỂ CẢ khi danh sách chỉ có ĐÚNG MỘT link", và đó chính
// là mệnh đề chưa ai canh.

/// I/O Matrix hàng *"`content-type` không phải HTML"* — `Extractor` KHÔNG được gọi trên byte
/// đó, mục giữ vị trí và mang đúng lý do `NotHtml`.
#[test]
fn a_response_that_is_not_html_becomes_one_broken_item_carrying_the_not_html_reason() {
    let (port, _h) = spawn_once(|mut stream| {
        let body = b"%PDF-1.7 khong phai mot trang HTML";
        let head = format!(
            "HTTP/1.1 200 OK\r\nContent-Type: application/pdf\r\nContent-Length: {}\r\n\r\n",
            body.len()
        );
        let _ = stream.write_all(head.as_bytes());
        let _ = stream.write_all(body);
    });

    let url = format!("http://127.0.0.1:{port}/tai-lieu.pdf");
    let (items, _log) = fetch_url_import_items(vec![url.clone()]);

    assert_eq!(items.len(), 1, "mục hỏng vẫn phải GIỮ CHỖ — hai con số N link · N Chương bằng nhau");
    assert!(items[0].raw.is_none(), "byte không phải HTML không được đi tiếp vào `Extractor`");
    assert_eq!(
        items[0].error.as_ref().expect("phải có lý do"),
        &web_import_item_failure_ipc_error(&url, WebImportItemFailureReason::NotHtml, None),
        "lý do phải là NotHtml — không phải một lỗi mạng chung chung"
    );
}

/// I/O Matrix hàng *"link hỏng — 404"* — vế `HttpStatus` của hàng đó, chưa ca nào chạm.
/// `status` đi vào `params` dưới dạng DỮ LIỆU (AD-21), không phải một câu.
#[test]
fn a_404_becomes_one_broken_item_carrying_the_http_status_reason_and_the_numeric_code() {
    let (port, _h) = spawn_once(|mut stream| {
        let _ = stream.write_all(b"HTTP/1.1 404 Not Found\r\nContent-Length: 0\r\n\r\n");
    });

    let url = format!("http://127.0.0.1:{port}/khong-ton-tai");
    let (items, log) = fetch_url_import_items(vec![url.clone()]);

    assert_eq!(items.len(), 1);
    assert!(items[0].raw.is_none());
    assert_eq!(
        items[0].error.as_ref().expect("phải có lý do"),
        &web_import_item_failure_ipc_error(&url, WebImportItemFailureReason::HttpStatus, Some(404)),
        "lý do phải là HttpStatus MANG ĐÚNG mã 404 — một mã sai làm người dùng đi tìm nhầm \
         nguyên nhân, và `?` (mã vắng) cũng là một lời nói dối nhẹ hơn"
    );
    // A2 (vòng rà đối kháng 2, 3 lớp) — nhật ký domain cũng phải mang đúng outcome HttpStatus
    // cho chặng ĐÃ ĐƯỢC PHÉP này, không chỉ IpcError của tầng hiển thị.
    assert!(
        log.iter().any(|e| e.outcome == Some(DomainLogOutcome::HttpStatus)),
        "nhat ky domain phai co it nhat mot ban ghi mang outcome HttpStatus cho lan 404 nay: {log:?}"
    );
}

/// I/O Matrix hàng *"trang bóc ra rỗng"* — 🟡 vế **RỖNG TUYỆT ĐỐI** của hàng đó, và CHỈ vế
/// đó. Vế *"quá ngắn"* mà hàng Matrix cũng nêu KHÔNG được dựng ở story này: nó đòi một hằng
/// số ngưỡng mà không phép đo nào hiện có đỡ nổi (bảy mẫu bàn đo 6.1 đều một site, bài thật
/// ngắn nhất 192 ký tự). Ice chốt 2026-09-07: ghi nợ, **Chủ Story 6.10** — story sở hữu bộ
/// lọc "cần xem", nơi tín hiệu là *so với TRUNG VỊ các Chương khác*, không cần hằng số.
/// ⚠️ Hệ quả còn hở, đo được: một trang chỉ có `<nav><a>m</a></nav>` cho ra một Chương
/// `source_text: "m"` — xem `deferred-work.md` §"Deferred from: 6-7… vòng rà bước 3".
///
/// Vế ÁNH XẠ LÝ DO. Ca `..._fails_extraction_instead_of_
/// falling_back_to_raw_html` phía trên canh `extract()` trả `Err`; ca này canh chuỗi biến `Err`
/// đó thành đúng `ExtractionEmpty` chứ không phải một lý do khác — và KHÔNG rơi về HTML thô.
#[test]
fn a_page_with_no_main_content_maps_to_the_extraction_empty_reason_inside_the_pipeline() {
    let barren = "<html><head><title>t</title></head><body></body></html>";
    let shape = PipelineShape::Chapters(vec![ChapterInput::RawBytes {
        bytes: barren.as_bytes().to_vec(),
        label: "http://127.0.0.1:1/barren".to_owned(),
    }]);
    let err = run_import(
        PipelineInput::default_shaped(shape, "zh".to_owned()).with_extract_main_content(true),
    )
    .expect_err("trang không bóc được nội dung chính phải TRƯỢT, không rơi về HTML thô");

    match err {
        ImportError::WebImportItemFailed { reason, url, .. } => {
            assert_eq!(
                reason,
                WebImportItemFailureReason::ExtractionEmpty,
                "phải là ExtractionEmpty — rơi về HTML thô là rỗng im lặng ĐỔI HÌNH DẠNG"
            );
            assert_eq!(url, "http://127.0.0.1:1/barren", "lý do phải chỉ đúng mục nào hỏng");
        }
        other => panic!("phải là WebImportItemFailed, nhận: {other:?}"),
    }
}

/// I/O Matrix hàng *"đúng MỘT link"* — `already_chaptered = true` nên bước 5 BỎ QUA **kể cả
/// khi** một mẫu phân tách khớp nhiều chỗ trong chính trang đó. Không có ca này thì một lượt
/// sửa làm bước 5 chạy lại trên đường URL sẽ cắt một Chương thành nhiều, và không cổng nào đỏ.
#[test]
fn exactly_one_link_yields_exactly_one_chapter_even_when_a_chapter_pattern_would_match_inside_it() {
    let html = "<html><body><article>\
        <p>Chuong 1 mo dau cua bai viet nay du dai de Readability giu lai lam noi dung chinh \
        cua trang, khong phai menu hay quang cao gi ca.</p>\
        <p>Chuong 2 tiep theo trong CUNG mot trang — neu buoc 5 chay, no se cat cho nay thanh \
        mot Chuong thu hai, va do la dieu ca test nay ton tai de chan.</p>\
        <p>Chuong 3 dong lai, van trong cung mot don vi dau vao, du dai de tong so ky tu vuot \
        nguong toi thieu ma Readability doi hoi.</p>\
        </article></body></html>";
    let shape = PipelineShape::Chapters(vec![ChapterInput::RawBytes {
        bytes: html.as_bytes().to_vec(),
        label: "http://127.0.0.1:1/mot-link".to_owned(),
    }]);
    let out = run_import(
        PipelineInput::default_shaped(shape, "zh".to_owned())
            .with_extract_main_content(true)
            .with_chapter_pattern(Some(ChapterPattern::literal("Chuong "))),
    )
    .expect("một link tốt phải đi hết chuỗi");

    assert_eq!(
        out.chapters.len(),
        1,
        "MỘT link ⇒ ĐÚNG MỘT Chương. `already_chaptered` phải làm bước 5 bỏ qua dù mẫu \
         `Chuong ` khớp ba chỗ bên trong trang — nếu ra 3, bước 5 đã chạy trên đường URL"
    );
}

/// I/O Matrix hàng *"link hỏng"*, vế **TIMEOUT** — phép ÁNH XẠ, không phải phép chờ.
///
/// ⚠️ **Vì sao KHÔNG gieo một lượt timeout thật:** `REQUEST_TIMEOUT` là **20 giây**
/// (`fetcher.rs:38`, con số chép từ bàn đo 6.1), nên một ca dựng server-không-bao-giờ-trả-lời
/// tốn 20 s mỗi lượt và phán quyết của nó phụ thuộc **wall-clock** — trên máy đo hiện tại
/// (`load average` 19-30 trên 16 lõi) đó là một ca giòn theo tải, đúng lớp lỗi `AGENTS.md`
/// §"hai lượt đo phải cùng tải máy" cảnh báo. Hành vi MẠNG đã được bàn đo 6.1 đo thật
/// (`reqwest::Error::is_timeout()` phân biệt được với `is_connect()`); thứ CHƯA ai canh — đo
/// 2026-09-07, `grep` cho **0** dòng — là nhánh biến lỗi đó thành lý do người dùng đọc. Ca này
/// canh đúng nhánh ấy, tất định, 0 giây.
#[test]
fn a_timeout_maps_to_the_timeout_reason_and_its_own_message_key_not_a_generic_failure() {
    let reason = WebImportItemFailureReason::from(FetchError::Timeout {
        detail: "qua han doc".to_owned(),
    });
    assert_eq!(
        reason,
        WebImportItemFailureReason::Timeout,
        "một lỗi quá hạn phải thành lý do Timeout — gộp nó vào ConnectFailed làm người dùng \
         đi tìm nhầm nguyên nhân (mạng chết vs máy chủ chậm là hai việc khác nhau)"
    );

    let url = "http://127.0.0.1:1/cham";
    assert_ne!(
        web_import_item_failure_ipc_error(url, WebImportItemFailureReason::Timeout, None),
        web_import_item_failure_ipc_error(url, WebImportItemFailureReason::ConnectFailed, None),
        "Timeout và ConnectFailed phải cho HAI `IpcError` khác nhau — cùng một khoá cho cả hai \
         là AD-21 bị thủng: người dùng đọc một câu không nói đúng chuyện đã xảy ra"
    );
}

// ═════════════════════════════════════════════════════════════════════════════════
// BỐN CA BỔ SUNG 2026-09-07 (vòng rà bước 4) — ánh xạ FetchError → lý do NGƯỜI DÙNG ĐỌC
// ═════════════════════════════════════════════════════════════════════════════════
//
// 🔴 **Lỗ được chứng minh bằng PHÉP ĐỘT BIẾN, không bằng suy luận.** Gieo hai đột biến vào
// `impl From<FetchError> for WebImportItemFailureReason` (`webimport/mod.rs`) —
// `RedirectBlockedCrossHost => TooLarge` và `InvalidUrl => ConnectFailed` — rồi chạy
// `cargo test --locked`: **1204 xanh, 0 đỏ**. Tức người dùng có thể được báo "phản hồi quá
// lớn" cho một chuyển hướng bị chặn, hoặc "không kết nối được" cho một dòng rác không phải
// URL, mà không cổng nào đỏ.
//
// Vì sao các ca CŨ không bắt được: `a_cross_host_redirect_is_blocked...` và
// `a_response_advertising_far_more_than_the_cap...` gọi `fetch()` TRẦN nên dừng ở
// `FetchError`, chưa đi qua phép ánh xạ; `n_links_are_fetched_sequentially...` và
// `a_garbage_line_becomes_one_broken_item...` có đi qua nhưng chỉ khẳng định `error.is_some()`
// — đúng lớp lỗi `AGENTS.md` §Known pitfalls gọi tên: một assert đúng ở CẢ HAI nhánh thì
// không canh nhánh nào.

/// Chuyển hướng khác host, ĐI QUA `fetch_url_import_items` — lý do phải là `RedirectBlocked`.
#[test]
fn a_blocked_cross_host_redirect_reaches_the_user_as_the_redirect_blocked_reason() {
    let (port_b, _hb) = spawn_once(|mut s| {
        let _ = s.write_all(ok_html_response("hi").as_bytes());
    });
    let location = format!("http://localhost:{port_b}/final");
    let (port_a, _ha) = spawn_once(move |mut s| {
        let _ = s.write_all(
            format!("HTTP/1.1 301 Moved Permanently\r\nLocation: {location}\r\nContent-Length: 0\r\n\r\n")
                .as_bytes(),
        );
    });

    let url = format!("http://127.0.0.1:{port_a}/start");
    let (items, _log) = fetch_url_import_items(vec![url.clone()]);

    assert_eq!(items.len(), 1);
    assert_eq!(
        items[0].error.as_ref().expect("phải có lý do"),
        &web_import_item_failure_ipc_error(&url, WebImportItemFailureReason::RedirectBlocked, None),
        "chuyển hướng bị chặn phải tới người dùng dưới lý do RedirectBlocked — báo một lý do \
         khác là nói sai chuyện đã xảy ra"
    );
}

/// Thân vượt trần, ĐI QUA `fetch_url_import_items` — lý do phải là `TooLarge`.
#[test]
fn an_oversized_body_reaches_the_user_as_the_too_large_reason() {
    // 🔴 200 MiB, KHÔNG phải 20: `MAX_RESPONSE_BYTES` = 20 MiB và điều kiện cắt là
    // `out.len() > MAX`, nên một thân đúng 20 MiB KHÔNG vượt trần. Cùng con số
    // `ADVERTISED_LEN` mà `a_response_advertising_far_more_than_the_cap...` đã dùng.
    let (port, _h) = spawn_once(|mut s| {
        const ADVERTISED_LEN: usize = 200 * 1024 * 1024;
        let _ = s.write_all(
            format!("HTTP/1.1 200 OK\r\nContent-Type: text/html\r\nContent-Length: {ADVERTISED_LEN}\r\n\r\n")
                .as_bytes(),
        );
        let chunk = vec![b'x'; 256 * 1024];
        let mut sent = 0usize;
        // Ghi tới khi client cắt kết nối — `write_all` trượt lúc đó, và đó là kết thúc mong đợi.
        while sent < ADVERTISED_LEN {
            let this_write = (ADVERTISED_LEN - sent).min(chunk.len());
            if s.write_all(&chunk[..this_write]).is_err() {
                break;
            }
            sent += this_write;
        }
    });

    let url = format!("http://127.0.0.1:{port}/qua-lon");
    let (items, _log) = fetch_url_import_items(vec![url.clone()]);

    assert_eq!(items.len(), 1);
    assert_eq!(
        items[0].error.as_ref().expect("phải có lý do"),
        &web_import_item_failure_ipc_error(&url, WebImportItemFailureReason::TooLarge, None),
        "thân vượt trần phải tới người dùng dưới lý do TooLarge"
    );
}

/// Dòng rác, ĐI QUA `fetch_url_import_items` — lý do phải là `InvalidUrl`, KHÔNG phải một lỗi
/// mạng. Đây là ca bắt đúng đột biến `InvalidUrl => ConnectFailed`.
#[test]
fn a_garbage_line_reaches_the_user_as_the_invalid_url_reason_not_a_network_failure() {
    let url = "day khong phai url";
    let (items, _log) = fetch_url_import_items(vec![url.to_owned()]);

    assert_eq!(items.len(), 1);
    assert_eq!(
        items[0].error.as_ref().expect("phải có lý do"),
        &web_import_item_failure_ipc_error(url, WebImportItemFailureReason::InvalidUrl, None),
        "một dòng không phải URL phải nói ĐÚNG là URL không hợp lệ — báo 'không kết nối được' \
         đẩy người dùng đi kiểm mạng cho một lỗi gõ nhầm"
    );
}

/// Cổng chết, ĐI QUA `fetch_url_import_items`.
///
/// ⚠️ Ca này KHÔNG ghim đúng một lý do: hệ điều hành được phép trả `timeout` thay cho
/// `refused` (cùng nhượng bộ mà `a_dead_connection_is_classified_as_connect_failed...` đã ghi).
/// Nó ghim phần ĐO ĐƯỢC một cách tất định — lý do phải nằm trong đúng hai nhánh mạng, và
/// KHÔNG được là một trong bốn lý do KHÔNG phải mạng.
#[test]
fn a_dead_port_reaches_the_user_as_a_network_reason_and_never_as_a_content_reason() {
    let dead_port = {
        let l = TcpListener::bind("127.0.0.1:0").expect("bind");
        l.local_addr().expect("addr").port()
    };
    let url = format!("http://127.0.0.1:{dead_port}/chet");
    let (items, log) = fetch_url_import_items(vec![url.clone()]);
    let got = items[0].error.as_ref().expect("phải có lý do");

    let is_network = got
        == &web_import_item_failure_ipc_error(&url, WebImportItemFailureReason::ConnectFailed, None)
        || got
            == &web_import_item_failure_ipc_error(&url, WebImportItemFailureReason::Timeout, None);
    assert!(is_network, "cổng chết phải cho một lý do MẠNG, nhận: {got:?}");

    // A2 (vòng rà đối kháng 2, 3 lớp) — nhật ký domain phải mang ĐÚNG MỘT trong hai outcome
    // mạng tương ứng (ConnectFailed/Timeout), khớp đúng lý do người dùng đọc được ở trên.
    assert!(
        log.iter().any(|e| matches!(e.outcome, Some(DomainLogOutcome::ConnectFailed) | Some(DomainLogOutcome::Timeout))),
        "nhat ky domain phai co it nhat mot ban ghi mang outcome ConnectFailed hoac Timeout: {log:?}"
    );

    for wrong in [
        WebImportItemFailureReason::InvalidUrl,
        WebImportItemFailureReason::TooLarge,
        WebImportItemFailureReason::RedirectBlocked,
        WebImportItemFailureReason::NotHtml,
    ] {
        assert_ne!(
            got,
            &web_import_item_failure_ipc_error(&url, wrong, None),
            "cổng chết KHÔNG được báo {wrong:?} — đó là một chẩn đoán sai hẳn tầng"
        );
    }
}

/// AC4 — *"hai con số lệch nhau thì một test tự động phải thất bại"*. Ca này là test đó, cho
/// chỗ lệch ĐO ĐƯỢC giữa `String.prototype.trim()` (JS, đếm trên màn hình) và `str::trim()`
/// (Rust, tải thật): JS cắt `U+FEFF`, Rust thì không. Xem
/// `commands::project::trim_like_the_paste_box`.
#[test]
fn a_zero_width_no_break_space_never_makes_the_fetched_count_differ_from_the_on_screen_count() {
    // ① Dòng CHỈ có BOM — JS đếm 0, Rust phải cũng cho 0 mục.
    let (items, _log) = fetch_url_import_items(vec!["\u{FEFF}".to_owned(), "  \u{FEFF} ".to_owned()]);
    assert!(
        items.is_empty(),
        "một dòng chỉ có U+FEFF không được sinh ra mục nào — màn hình đếm 0, nếu Rust đếm 1 thì \
         `N link` khác `N Chương` và đó đúng là dấu hiệu AC4 tồn tại để bắt. Nhận: {items:?}"
    );

    // ② URL hợp lệ mang BOM ở ĐẦU — JS coi là một link hợp lệ, Rust phải cũng vậy (không
    // được biến nó thành một mục hỏng oan bằng `InvalidUrl`).
    let dead_port = {
        let l = TcpListener::bind("127.0.0.1:0").expect("bind");
        l.local_addr().expect("addr").port()
    };
    let bare = format!("http://127.0.0.1:{dead_port}/co-bom");
    let (items, _log) = fetch_url_import_items(vec![format!("\u{FEFF}{bare}")]);
    assert_eq!(items.len(), 1);
    assert_eq!(items[0].url, bare, "BOM đầu dòng phải được cắt, URL còn lại nguyên vẹn");
    assert_ne!(
        items[0].error.as_ref().expect("cổng chết nên phải có lý do"),
        &web_import_item_failure_ipc_error(&bare, WebImportItemFailureReason::InvalidUrl, None),
        "một URL hợp lệ mang BOM KHÔNG được báo là URL không hợp lệ — người dùng không nhìn \
         thấy được một ký tự vô hình"
    );
}

/// AI-7, D1 — chỗ lệch NGƯỢC CHIỀU: một dòng dán CHỈ có `U+0085` (NEL) phải sinh ĐÚNG MỘT
/// mục, khớp `pastedUrlLines` phía JS (`.trim()` gốc của JS KHÔNG cắt NEL, nên dòng đó không
/// rỗng, `N link` đếm nó là 1). Trước bản vá `trim_like_the_paste_box` cắt bằng
/// `char::is_whitespace()`, mà `U+0085` NẰM trong `White_Space` của Unicode nên dòng bị cắt
/// thành rỗng và bị lọc bỏ — `N link` (JS) và `N Chương` (Rust) lệch nhau đúng MỘT, đối xứng
/// ngược với ca BOM ngay trên.
#[test]
fn a_next_line_character_only_pasted_line_still_produces_exactly_one_item_matching_pasted_url_lines() {
    let (items, _log) = fetch_url_import_items(vec!["\u{0085}".to_owned()]);
    assert_eq!(
        items.len(),
        1,
        "một dòng CHỈ có U+0085 phải sinh ĐÚNG một mục — JS `.trim()` không cắt NEL nên \
         `pastedUrlLines` đếm nó là 1 dòng; Rust đếm khác đi là chính chỗ lệch NGƯỢC CHIỀU mà \
         D1 đóng. Nhận: {items:?}"
    );
}

// ═════════════════════════════════════════════════════════════════════════════════
// P2 (vòng rà đối kháng bước 4) — trần số chặng chuyển hướng, dưới một host ĐƯỢC allowlist
// ═════════════════════════════════════════════════════════════════════════════════
//
// `redirect::Policy::custom` THAY TRỌN chính sách mặc định của `reqwest` — trần ~10 chặng
// mặc định biến mất nếu không tự thêm. Trước bản vá này, một vòng lặp chuyển hướng CÙNG host
// (không bị AD-41 chặn vì host không đổi) chỉ dừng lại nhờ `REQUEST_TIMEOUT` (20 s) và bị báo
// SAI cho người dùng là `Timeout`. Server dưới đây chấp nhận NHIỀU kết nối liên tiếp, mỗi lần
// trả một 302 trỏ VỀ CHÍNH nó — một vòng lặp không bao giờ tự dừng nếu không có trần.
//
// 🔵 **Story 6.8** — `allowlist` PHẢI chứa host này, nếu không hop ĐẦU TIÊN đã bị AD-41 chặn
// (`NotAllowlisted`) trước khi vòng lặp có cơ hội chạy, và ca này không còn kiểm được đúng
// thứ nó dựng ra để canh (trần SỐ CHẶNG, không phải chuyện allowlist).
#[test]
fn a_same_host_redirect_loop_is_capped_and_not_misreported_as_a_timeout() {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind cổng tạm");
    let port = listener.local_addr().expect("local_addr").port();
    let handle = thread::spawn(move || {
        // Đủ vòng lặp để vượt trần sản phẩm (10) — bản thân test không cần biết đúng con số,
        // chỉ cần server SẴN SÀNG lặp nhiều hơn bất kỳ trần hợp lý nào.
        for _ in 0..30 {
            match listener.accept() {
                Ok((mut stream, _)) => {
                    let _ = stream.set_read_timeout(Some(Duration::from_secs(5)));
                    let mut discard = [0u8; 4096];
                    let _ = stream.read(&mut discard);
                    let body = format!(
                        "HTTP/1.1 302 Found\r\nLocation: http://127.0.0.1:{port}/loop\r\nContent-Length: 0\r\n\r\n"
                    );
                    if stream.write_all(body.as_bytes()).is_err() {
                        break;
                    }
                }
                Err(_) => break,
            }
        }
    });

    let url = format!("http://127.0.0.1:{port}/loop");
    let allowlist = Allowlist::from_urls([url.as_str()]);
    let (result, log) = fetch(&url, &allowlist, ResourceKind::Page);

    assert!(
        !matches!(result, Err(FetchError::Timeout { .. })),
        "vòng lặp chuyển hướng CÙNG HOST không được báo sai là `Timeout` — trần số chặng phải \
         bắt được nó trước khi `REQUEST_TIMEOUT` (20s) kịp hết hạn: {result:?}"
    );
    assert!(result.is_err(), "vòng lặp chuyển hướng phải là một lỗi, không phải `Ok`: {result:?}");

    // A2/D4 (vòng rà đối kháng 2, 3 lớp) — MỌI chặng đã thật sự theo (tất cả trừ chặng CUỐI)
    // phải mang outcome Redirected; và chặng CUỐI (nơi vòng lặp bị huỷ vì vượt trần) phải
    // mang `Other`, KHÔNG được giữ nguyên `Redirected` — nếu không, bản ghi cuối đọc lên y
    // hệt một chuyển hướng bình thường, không phân biệt được với một fetch còn giữa chừng.
    assert!(log.len() >= 2, "vòng lặp phải sinh ra nhiều hơn một bản ghi domain: {log:?}");
    let (last, earlier) = log.split_last().expect("log khong rong");
    assert_eq!(
        last.outcome,
        Some(DomainLogOutcome::Other),
        "ban ghi CUOI CUNG cua mot vong lap vuot tran phai mang outcome Other, khong duoc giu \
         nguyen Redirected: {log:?}"
    );
    assert!(
        earlier.iter().all(|e| e.outcome == Some(DomainLogOutcome::Redirected)),
        "moi ban ghi TRUOC ban ghi cuoi phai mang outcome Redirected (da that su theo chuyen \
         huong): {log:?}"
    );

    drop(handle);
}

// ═════════════════════════════════════════════════════════════════════════════════
// P3 (vòng rà đối kháng bước 4) — một 3xx LÀNH không phải `NotAllowlisted`
// ═════════════════════════════════════════════════════════════════════════════════
//
// `status().is_redirection()` một mình không phân biệt được "bị chính sách chặn" khỏi "máy
// chủ tự trả một 3xx làm phản hồi CUỐI" (không có `Location`, ví dụ 304 Not Modified). Trước
// bản vá này, ca dưới đây bị báo SAI là `NotAllowlisted` dù chưa từng có một chặng nào bị
// chặn — server chỉ nhận ĐÚNG MỘT kết nối. `allowlist` chứa CHÍNH host này để loại trừ khả
// năng "bị chặn vì không allowlist" khỏi ca kiểm — thứ ca này canh là chuyện KHÁC hẳn.
#[test]
fn a_benign_3xx_with_no_location_header_is_not_reported_as_a_blocked_redirect() {
    let (port, _h) = spawn_once(|mut stream| {
        let _ = stream.write_all(b"HTTP/1.1 304 Not Modified\r\nContent-Length: 0\r\n\r\n");
    });

    let url = format!("http://127.0.0.1:{port}/khong-doi");
    let allowlist = Allowlist::from_urls([url.as_str()]);
    let (result, _log) = fetch(&url, &allowlist, ResourceKind::Page);

    assert!(
        !matches!(result, Err(FetchError::NotAllowlisted)),
        "một 304 KHÔNG có `Location` chưa từng bị chính sách chặn — không được báo \
         `NotAllowlisted`: {result:?}"
    );
    assert!(
        matches!(result, Err(FetchError::HttpStatus { status: 304 })),
        "một 3xx LÀNH còn lại phải thành `HttpStatus` mang đúng mã số: {result:?}"
    );
}

// ═════════════════════════════════════════════════════════════════════════════════
// P4 (vòng rà đối kháng bước 4) — `looks_like_html` so BẰNG kiểu MIME, không khớp chuỗi con
// ═════════════════════════════════════════════════════════════════════════════════

#[test]
fn looks_like_html_matches_the_exact_mime_type_not_a_substring() {
    assert!(looks_like_html(Some("text/html; charset=utf-8")), "kiểu HTML kèm charset phải khớp");
    assert!(looks_like_html(Some("TEXT/HTML")), "so sánh không phân biệt hoa/thường");
    assert!(looks_like_html(Some("application/xhtml+xml")), "xhtml cũng được coi là HTML");

    assert!(
        !looks_like_html(Some("text/htmlfoo")),
        "`text/htmlfoo` KHÔNG được khớp — trước bản vá `contains(\"text/html\")` khớp nhầm nó"
    );
    assert!(
        !looks_like_html(Some("application/json")),
        "một kiểu MIME không liên quan không được khớp"
    );
    assert!(
        !looks_like_html(Some("multipart/form-data; boundary=text/html")),
        "chuỗi `text/html` xuất hiện trong MỘT THAM SỐ không được làm cả kiểu MIME khớp"
    );
    assert!(!looks_like_html(None), "máy chủ không khai content-type phải bị coi là KHÔNG PHẢI HTML");
}

// ═════════════════════════════════════════════════════════════════════════════════
// STORY 6.8 — BỐN CA AD-41 BẮT BUỘC (spine `:542`) + CA TẦNG 2
// ═════════════════════════════════════════════════════════════════════════════════
//
// AD-41 (`ARCHITECTURE-SPINE.md:542`) đòi nguyên văn: *"từ chối host ngoài hai tầng; từ
// chối chuyển hướng ra ngoài; từ chối tài liệu ở tầng 2; không lời gọi nào khi người dùng
// không bấm"*. Bốn `#[test]` dưới đây ứng ĐÚNG bốn mệnh đề đó, theo thứ tự; ca "chuyển
// hướng ra ngoài" chính là `a_redirect_to_a_host_outside_the_allowlist_is_blocked_and_...`
// đã viết lại ở đầu tệp (Ca 1) — không lặp một bản thứ hai ở đây.

/// AD-41, mệnh đề 1 — **"từ chối host ngoài hai tầng"**. Allowlist RỖNG (không tier1, không
/// tier2) ⇒ `fetch` từ chối NGAY host của chính URL gốc, TRƯỚC khi `send()` chạy — máy chủ
/// đích nhận ĐÚNG 0 kết nối (đo bằng `AtomicUsize`, không bằng mã trả về — đúng kỷ luật
/// Acceptance Criteria đầu tiên của spec 6.8).
#[test]
fn ad41_case_1_a_host_outside_both_tiers_is_denied_before_any_connection_opens() {
    let reached = Arc::new(AtomicUsize::new(0));
    let reached_clone = Arc::clone(&reached);
    let (port, _handle) = spawn_once(move |mut stream| {
        reached_clone.fetch_add(1, Ordering::SeqCst);
        let _ = stream.write_all(ok_html_response("khong duoc phep toi day").as_bytes());
    });

    let url = format!("http://127.0.0.1:{port}/ngoai-allowlist");
    // Allowlist KHÔNG được dựng từ chính `url` — mô phỏng đúng "host chưa từng có trong danh
    // sách dán", khác các test khác trong tệp này (chúng luôn tự allowlist host mình gọi).
    let allowlist = Allowlist::default();
    let (result, log) = fetch(&url, &allowlist, ResourceKind::Page);

    assert!(matches!(result, Err(FetchError::NotAllowlisted)), "ky vong NotAllowlisted, nhan: {result:?}");
    assert_eq!(reached.load(Ordering::SeqCst), 0, "may chu dich phai nhan DUNG 0 ket noi");
    assert_eq!(log.len(), 1, "mot ban ghi nhat ky cho lan bi tu choi nay");
    assert!(matches!(log[0].decision, DomainLogDecision::Denied), "ban ghi phai la TU CHOI, khong phai cho phep");
}

/// AD-41, mệnh đề 3 — **"từ chối tài liệu ở tầng 2"**. Luật *"tầng 2 chỉ ảnh, không bao giờ
/// tài liệu"* phải sống trong KIỂU (`Allowlist::decide` — một `match` cạn trên
/// [`ResourceKind`]), không trong một `if` rời ở chỗ gọi (§Always spec 6.8). Host CHỈ có mặt
/// ở tầng 2 (không tier1) + xin `Document` ⇒ từ chối, 0 kết nối — dù CHÍNH host đó, xin
/// `Image`, lẽ ra được phép (xem ca ngay dưới).
#[test]
fn ad41_case_3_a_document_request_to_a_tier_2_only_host_is_denied() {
    let reached = Arc::new(AtomicUsize::new(0));
    let reached_clone = Arc::clone(&reached);
    let (port, _handle) = spawn_once(move |mut stream| {
        reached_clone.fetch_add(1, Ordering::SeqCst);
        let _ = stream.write_all(ok_html_response("tai lieu khong duoc phep o tang 2").as_bytes());
    });

    let url = format!("http://127.0.0.1:{port}/bai-viet");
    let host = reqwest::Url::parse(&url).expect("URL hop le").host_str().expect("co host").to_owned();
    let allowlist = Allowlist::default().with_tier2_hosts([host]);
    let (result, _log) = fetch(&url, &allowlist, ResourceKind::Page);

    assert!(
        matches!(result, Err(FetchError::NotAllowlisted)),
        "tai lieu tu mot host CHI o tang 2 phai bi tu choi: {result:?}"
    );
    assert_eq!(reached.load(Ordering::SeqCst), 0, "may chu dich phai nhan DUNG 0 ket noi");
}

/// Ca đối chứng cho mệnh đề 3 — CÙNG host tầng 2, xin `Image` thay vì `Document` ⇒ ĐƯỢC PHÉP
/// (I/O Matrix spec 6.8: *"Ảnh từ host tầng 2 → Cho phép"*). Không có ca này, mệnh đề 3 ở
/// trên có thể xanh vì một lý do SAI (ví dụ: allowlist rỗng bị đọc nhầm là "luôn từ chối",
/// không phải "đúng tầng 2 thì từ chối Document").
#[test]
fn an_image_request_to_a_tier_2_only_host_is_allowed() {
    let (port, _handle) = spawn_once(|mut stream| {
        let _ = stream.write_all(ok_html_response("anh duoc phep o tang 2").as_bytes());
    });

    let url = format!("http://127.0.0.1:{port}/anh.jpg");
    let host = reqwest::Url::parse(&url).expect("URL hop le").host_str().expect("co host").to_owned();
    let allowlist = Allowlist::default().with_tier2_hosts([host]);
    let (result, _log) = fetch(&url, &allowlist, ResourceKind::Image);

    result.expect("anh tu mot host tang 2 phai duoc CHO PHEP");
}

/// AD-41, mệnh đề 4 — **"không lời gọi nào khi người dùng không bấm"**. Dựng một
/// [`Allowlist`] từ danh sách URL KHÔNG tự phát sinh bất kỳ kết nối mạng nào — `from_urls`
/// chỉ phân giải host bằng `Url::parse` (thuần, không DNS/socket). Ca này là một đối chứng
/// CHỐNG HỒI QUY: một lượt sửa sau này lỡ thêm một bước xác thực host qua mạng vào
/// `Allowlist::from_urls` (ví dụ "thử kết nối trước để biết host có tồn tại") sẽ làm ca này
/// đỏ — chính điều mệnh đề 4 cấm.
#[test]
fn ad41_case_4_building_an_allowlist_makes_no_network_call_on_its_own() {
    let reached = Arc::new(AtomicUsize::new(0));
    let reached_clone = Arc::clone(&reached);
    let (port, handle) = spawn_once(move |mut stream| {
        reached_clone.fetch_add(1, Ordering::SeqCst);
        let _ = stream.write_all(ok_html_response("khong ai duoc goi toi day").as_bytes());
    });

    let url = format!("http://127.0.0.1:{port}/chua-bam");
    let _allowlist = Allowlist::from_urls([url.as_str()]);

    // KHÔNG một lời gọi `fetch(...)` nào ở đây — đúng mô phỏng "N dòng dán vào ô, chưa bấm
    // nút tải" (I/O Matrix spec 6.8, hàng 1). Cho server một khoảng ngắn để lộ ra một kết
    // nối NẾU CÓ (nó không nên có) trước khi kiểm bộ đếm.
    thread::sleep(Duration::from_millis(50));
    assert_eq!(
        reached.load(Ordering::SeqCst),
        0,
        "dung Allowlist tu danh sach URL KHONG duoc tu phat sinh mot ket noi mang nao"
    );

    drop(handle);
}

/// Đối chứng — [`fetch_url_import_items`] cũng phải NỐI ĐƯỢC nhật ký domain của MỌI mục,
/// không chỉ mục đầu (mỗi mục là một lượt `fetch` riêng, allowlist dùng CHUNG cho cả danh
/// sách — xem doc-comment [`fetch_url_import_items`]).
#[test]
fn fetch_url_import_items_returns_one_domain_log_entry_per_item() {
    let (port_a, _ha) = spawn_once(|mut s| {
        let _ = s.write_all(ok_html_response(&html_page_with_paragraphs()).as_bytes());
    });
    let (port_b, _hb) = spawn_once(|mut s| {
        let _ = s.write_all(ok_html_response(&html_page_with_paragraphs()).as_bytes());
    });

    let urls = vec![format!("http://127.0.0.1:{port_a}/a"), format!("http://127.0.0.1:{port_b}/b")];
    let (items, log) = fetch_url_import_items(urls);

    assert_eq!(items.len(), 2);
    assert!(items.iter().all(|it| it.error.is_none()), "hai host deu trong allowlist tu chinh danh sach");
    assert_eq!(log.len(), 2, "mot ban ghi nhat ky cho MOI muc, ca hai deu CHO PHEP");
}

// ═════════════════════════════════════════════════════════════════════════════════
// Story 6.11 — tầng 2 đi ĐẦU-CUỐI qua `create_work` (Code Map spec 6.11: "đừng chép lại bốn
// ca AD-41 đã có ở :986-1057" — ca dưới đây kiểm WIRING của `create_work`, không lặp lại
// mệnh đề CƠ CHẾ mà bốn ca kia đã khoá ở tầng `Allowlist`/`fetch`).
// ═════════════════════════════════════════════════════════════════════════════════

static NEXT_WEBIMPORT_DIR: AtomicUsize = AtomicUsize::new(0);

fn webimport_temp_dir(tag: &str) -> PathBuf {
    let n = NEXT_WEBIMPORT_DIR.fetch_add(1, Ordering::Relaxed);
    let dir = std::env::temp_dir().join(format!(
        "auratranslate-webimport-e2e-{}-{tag}-{n}",
        std::process::id()
    ));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap_or_else(|e| panic!("tao {}: {e}", dir.display()));
    dir
}

/// Trang bài viết với MỘT ảnh — cùng khuôn `html_page_with_paragraphs`, thêm một `<img>` giữa
/// hai đoạn cuối.
fn html_page_with_paragraphs_and_one_image(img_src: &str) -> String {
    format!(
        "<html><head><title>Bai viet</title></head><body><article><h1>Tieu de</h1>\
         <p>Doan mot co du chu de duoc Readability chon lam noi dung chinh cua trang, \
         nhieu chu hon de vuot nguong do dai toi thieu.</p>\
         <img src=\"{img_src}\">\
         <p>Doan hai tiep tuc noi dung that su cua bai viet, khong phai menu hay quang cao, \
         du dai de dom_smoothie cham diem cao cho khoi nay.</p>\
         <p>Doan ba dong y nghia, giu cho tong do dai van ban vuot qua nguong toi thieu can \
         thiet de Readability tin day la mot bai viet that.</p>\
         </article></body></html>"
    )
}

/// **AD-41, mệnh đề "chuyển hướng ra ngoài" — đi qua ĐÚNG `create_work`, không gọi `fetch`
/// trực tiếp.** Ảnh tầng 2 (host `port_a`, đúng host mà `create_work` tự thêm vào allowlist
/// tầng 2 vì đó là host của CHÍNH `<img src>`) chuyển hướng 301 sang một host KHÁC
/// (`port_b`) — host đó KHÔNG BAO GIỜ được thêm vào allowlist (nó không phải host của bất kỳ
/// `src` nào trong trang), nên phải bị `NotAllowlisted` chặn TRƯỚC khi mở kết nối. Đây là
/// bằng chứng WIRING: nếu `prepare_chapter_images`/`fetch_and_write_one_asset` lỡ dựng một
/// allowlist rộng hơn cần thiết (ví dụ cho phép MỌI host), ca này đỏ ngay.
///
/// 🔴 **SỬA TÊN 2026-09-09 (vòng rà đối kháng 2, mục C2) — phạm vi HẸP HƠN tên cũ.** Tên cũ
/// (`create_work_blocks_an_image_redirect_to_a_host_outside_tier_two_...`) đọc như một mệnh
/// đề TỔNG QUÁT ("mọi host ngoài tầng 2 đều bị chặn"), nhưng `prepare_chapter_images` dựng
/// tầng 2 MỘT LẦN từ host của MỌI ảnh giữ trong MỌI Chương rồi dùng CHUNG cho cả Tác phẩm
/// (xem doc-comment hàm đó) — một ảnh ở Chương 1 chuyển hướng sang host của một ảnh Ở CHƯƠNG
/// KHÁC (ví dụ Chương 5) sẽ ĐƯỢC CHO QUA, không bị chặn. Ca này chỉ phủ đúng MỘT trường hợp
/// hẹp hơn: host đích không phải host của BẤT KỲ `src` nào trong TOÀN Tác phẩm. Chưa có ca
/// nào phủ trường hợp rộng hơn (chuyển hướng CHÉO Chương) — ghi nợ tại
/// `deferred-work.md` thay vì để tên ca nói quá thứ nó đo.
#[test]
fn create_work_blocks_an_image_redirect_to_a_host_matching_no_src_anywhere_in_the_work() {
    let reached_b = Arc::new(AtomicUsize::new(0));
    let reached_b_clone = Arc::clone(&reached_b);
    let (port_b, _handle_b) = spawn_once(move |mut stream| {
        reached_b_clone.fetch_add(1, Ordering::SeqCst);
        let _ = stream.write_all(ok_html_response("khong duoc phep toi day").as_bytes());
    });

    // `localhost`, KHÔNG `127.0.0.1` — hai CHUỖI HOST khác nhau theo `Url::host_str()` dù
    // cùng trỏ về loopback (cùng mẹo `a_redirect_to_a_host_outside_the_allowlist_is_blocked_...`
    // ở đầu tệp này): `Allowlist::decide` so bằng CHUỖI HOST, không phân giải DNS, nên
    // `127.0.0.1:port_a` và `127.0.0.1:port_b` sẽ là CÙNG một host (cổng không phải một phần
    // của `host_str()`) và không mô phỏng được ca "host lạ".
    let location = format!("http://localhost:{port_b}/anh-that.jpg");
    let location_for_server = location.clone();
    let (port_a, _handle_a) = spawn_once(move |mut stream| {
        let body = format!(
            "HTTP/1.1 301 Moved Permanently\r\nLocation: {location_for_server}\r\nContent-Length: 0\r\n\r\n"
        );
        let _ = stream.write_all(body.as_bytes());
    });

    let img_src = format!("http://127.0.0.1:{port_a}/anh.jpg");
    let items = vec![UrlImportItem {
        url: "https://example.test/bai-mot".to_owned(),
        raw: Some(html_page_with_paragraphs_and_one_image(&img_src).into_bytes()),
        error: None,
    }];
    let shape = chapters_shape_if_all_ok(&items).expect("danh sach toan muc OK");

    let root = webimport_temp_dir("tier2-redirect-blocked");
    let domain_log_state: auratranslate_lib::core::webimport::DomainLogState = std::sync::Mutex::new(Vec::new());
    let opened = create_work(
        &root,
        "Tier2 Redirect",
        "en",
        "",
        shape,
        encoding_rs::UTF_8,
        Vec::new(),
        None,
        Vec::new(), 0, 1, false, &[],
        &domain_log_state,
        None, &[])
    .expect("mot anh bi chan KHONG duoc lam trot ca luot nhap");

    assert_eq!(opened.images_saved, 0, "chuyen huong ra ngoai tang 2 phai bi chan, khong co tep nao duoc luu");
    assert_eq!(opened.images_failed, 1);
    assert_eq!(reached_b.load(Ordering::SeqCst), 0, "host DICH cua chuyen huong phai nhan DUNG 0 ket noi");

    // §Always spec 6.11 — "mọi Vec<DomainLogEntry> mà fetch trả về phải nối vào DomainLogState
    // — kể cả lượt trượt": ca Denied của host b PHẢI có mặt trong nhật ký.
    let log = domain_log_state.lock().unwrap();
    assert!(
        log.iter().any(|e| e.domain == "localhost" && matches!(e.decision, DomainLogDecision::Denied)),
        "nhat ky domain phai co it nhat mot ban ghi TU CHOI cho lan chuyen huong nay: {:?}",
        *log
    );
    drop(log);

    drop(opened);
    let _ = fs::remove_dir_all(&root);
}

// ═════════════════════════════════════════════════════════════════════════════════
// Story 6.9 — Đối chứng đỏ ② (§Verification spec 6.9): phủ khối trên BẢY mẫu bàn đo 6.1
// ═════════════════════════════════════════════════════════════════════════════════

/// `a07.html` có **0** thẻ `<p>` — ca quyết định của Task list spec 6.9 (bộ chọn khối vòng 1
/// chỉ phủ `p, img, figcaption`, mất trắng nội dung dạng tiêu đề/danh sách của trang này).
/// Đường dẫn TƯƠNG ĐỐI với `CARGO_MANIFEST_DIR` (`src-tauri/`).
fn fixture_html(name: &str) -> String {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("_bmad-output")
        .join("implementation-artifacts")
        .join("6-1-ban-do")
        .join("fixtures")
        .join("html")
        .join(name);
    fs::read_to_string(&path).unwrap_or_else(|err| panic!("khong doc duoc {}: {err}", path.display()))
}

/// Như trước — trả CHÍNH chuỗi `text_content` (cấu hình `TextMode::Formatted`, ĐÚNG khuôn
/// `extractor::extract`), dùng làm mốc so BẰNG TỪNG KÝ TỰ, độc lập với mô hình khối —
/// **THÊM 2026-09-07 (vòng rà bước 4, mục 12)** để so BẰNG TỪNG KÝ TỰ với văn bản ghép từ
/// khối, không chỉ so ĐỘ DÀI.
fn fixture_text_content(html: &str, url: &str) -> String {
    let config = dom_smoothie::Config { text_mode: dom_smoothie::TextMode::Formatted, ..Default::default() };
    let mut readability = dom_smoothie::Readability::new(html.to_owned(), Some(url), Some(config))
        .expect("Readability::new");
    let article = readability.parse().expect("parse");
    article.text_content.to_string()
}

/// 🔴 **Đối chứng đỏ ② — phủ khối, bảy mẫu bàn đo 6.1.**
///
/// 🔴 **SỬA 2026-09-07 (vòng rà bước 4, mục 12) — sàn 60% ĐO YẾU HƠN điều AC thật sự đòi, và
/// SAI CẢ CHO a07.** Bản trước chỉ hỏi "văn bản ghép có KHÔNG NGẮN HƠN ĐÁNG KỂ `text_content`
/// không" (sàn 60% độ dài, áp cho SÁU mẫu a01-a06) — một ngưỡng GẦN ĐÚNG cho một cơ chế mà
/// chính doc-comment đầu `extractor.rs` khai là SO KHỚP CHÍNH XÁC, không khoan dung. Đo LẠI
/// cả sáu mẫu đó (không override — đúng điều kiện AC "trùng đúng đầu ra Story 6.7" áp):
/// `joined == text_content` **THẬT SỰ đúng TỪNG KÝ TỰ trên cả sáu**, không chỉ "đủ gần" — sàn
/// 60% vì vậy che mất một hồi quy thật (ví dụ một khối bị rớt/gán sai vị trí nhưng đủ ngắn để
/// vẫn qua sàn 60%). Ca này giờ so BẰNG (`assert_eq!`) cho a01-a06, đúng độ mạnh mà AC đòi.
///
/// 🔴 `a07.html` (0 thẻ `<p>`) là ca QUYẾT ĐỊNH của Task list spec 6.9: trước bản vá gốc nó
/// cho **0 khối** ⇒ Chương ghi xuống RỖNG. Ca ĐÓ đã đóng (assert "không rỗng" áp cho cả bảy
/// mẫu, giữ NGUYÊN). 🔴 **Sàn 60% KHÔNG áp cho `a07.html` — đo thật (2026-09-07) cho ra
/// 107/218 ký tự = 49%, DƯỚI 60%, một cách CHÍNH ĐÁNG, không phải một hồi quy cần vá.**
/// `a07.html` là trang chủ (không phải bài viết) — chính doc-comment Task 1 của
/// `extractor.rs` (Story 6.7) đã ghi nhận: nhãn điều hướng/thời gian đăng ("3小時"…)/tiêu đề
/// mục ("熱門排行"…) chiếm gần hết `text_content` của CHÍNH Readability, và những nhãn NGẮN,
/// LẶP LẠI đó (ví dụ ba khối cùng là "2小時") khớp CHÍNH XÁC ở NHIỀU vị trí — DP tối đa hoá
/// tổng ký tự đôi khi phải BỎ một khớp ngắn để giữ trật tự cho một khớp khác nặng hơn (đúng
/// cơ chế), và trên một trang TOÀN nhãn ngắn như thế này, phần "bỏ" đó cộng dồn thành gần một
/// nửa. Không một hằng ngưỡng nào (60% hay khác) có nghĩa thật ở đây — ca này giữ ĐÚNG hai
/// đối chứng mà spec đòi cho `a07` (không 0 khối, văn bản ghép không rỗng), không hơn.
///
/// ⚠️ **`#[ignore]` từ 2026-09-11 — ca này KHÔNG chạy được trên CI.** Bảy mẫu nằm trong
/// `6-1-ban-do/fixtures/html/`, thư mục `6-1-ban-do/.gitignore` loại có chủ ý (README: "nội
/// dung có bản quyền, không commit"). Lượt push đầu tiên đưa ca này lên CI (run `34553274876`)
/// đỏ trên `macos-26` với `No such file or directory`; trên máy dev nó xanh chỉ vì cache có sẵn.
/// Mệnh đề nó canh nay chạy MẶC ĐỊNH trên fixture tự viết ở
/// [`extract_covers_headings_list_items_and_a_zero_paragraph_page_on_hand_written_fixtures`];
/// ca này ở lại làm bàn đo trên dữ liệu THẬT. Chạy tay:
/// `cargo test --test webimport_contract -- --ignored extract_covers_all_seven`
#[test]
#[ignore = "ban do can bay trang HTML THAT o 6-1-ban-do/fixtures/html (gitignore vi co ban quyen), khong co tren CI"]
fn extract_covers_all_seven_bench_fixtures_without_losing_headings_or_list_items() {
    for name in ["a01.html", "a02.html", "a03.html", "a04.html", "a05.html", "a06.html", "a07.html"] {
        let html = fixture_html(name);
        let url = format!("https://example.com/{name}");
        let blocks = extract(&html, &url).unwrap_or_else(|e| panic!("{name}: extract that bai: {e:?}"));
        assert!(!blocks.is_empty(), "{name}: phải cho ít nhất một khối");

        let effective_kept: Vec<bool> = blocks.iter().map(|b| b.machine_kept).collect();
        let joined = auratranslate_lib::core::segment::pipeline::join_kept_blocks(&blocks, &effective_kept);
        assert!(!joined.trim().is_empty(), "{name}: văn bản ghép từ khối đang giữ không được rỗng");

        if name == "a07.html" {
            // Ngoại lệ CÓ CHỦ, xem doc-comment hàm này — hai đối chứng ngay trên (không rỗng
            // khối/văn bản) là TẤT CẢ những gì có nghĩa thật cho riêng mẫu này.
            continue;
        }

        let text_content = fixture_text_content(&html, &url);
        assert_eq!(
            joined, text_content,
            "{name}: văn bản ghép từ khối (0 override) phải trùng ĐÚNG TỪNG KÝ TỰ với \
             text_content -- đây là AC 'trùng đúng đầu ra Story 6.7', không phải một sàn %"
        );
    }
}

// ═════════════════════════════════════════════════════════════════════════════════
// Đối chứng đỏ ② trên fixture TỰ VIẾT, commit được — chạy mặc định, kể cả trên CI
// ═════════════════════════════════════════════════════════════════════════════════

/// Trang BÀI VIẾT tự viết, không chép từ nguồn nào — cùng hình dạng a01-a06 mà ca `#[ignore]`
/// ở trên đo: `h2`/`h3`, danh sách `<li>`, `blockquote`, `pre` xen giữa các `<p>`, cộng điều
/// hướng và chân trang mà Readability phải loại.
const HAND_WRITTEN_ARTICLE_HTML: &str = r#"<!DOCTYPE html>
<html lang="vi">
<head><meta charset="utf-8"><title>Ghi chép về cách ủ trà xanh tại nhà</title></head>
<body>
<header>
  <nav>
    <ul>
      <li><a href="/">Trang chủ</a></li>
      <li><a href="/chuyen-muc/do-uong">Đồ uống</a></li>
      <li><a href="/lien-he">Liên hệ</a></li>
    </ul>
  </nav>
</header>
<main>
  <article>
    <h1>Ghi chép về cách ủ trà xanh tại nhà</h1>
    <p>Bài ghi chép này được viết riêng làm dữ liệu kiểm thử cho bộ bóc nội dung. Nội dung không chép từ bất kỳ trang nào; mọi con số trong bài chỉ là ví dụ minh hoạ, không phải lời khuyên.</p>
    <h2>Chuẩn bị nước và ấm</h2>
    <p>Nước dùng để ủ trà nên được đun sôi rồi để nguội bớt trong vài phút. Ấm được tráng qua bằng nước nóng để giữ nhiệt đều, tránh làm lá trà bị sốc nhiệt ở lượt rót đầu tiên.</p>
    <ul>
      <li>Một ấm sứ hoặc ấm đất nhỏ, dung tích vừa đủ cho hai chén.</li>
      <li>Khoảng một thìa đầy lá trà khô cho mỗi lượt ủ.</li>
      <li>Nước đã đun sôi và để nguội bớt chừng ba phút.</li>
    </ul>
    <h3>Vì sao không dùng nước đang sôi</h3>
    <p>Nước quá nóng làm nước trà nhanh chát và mất mùi thơm nhẹ vốn có của lá. Người mới tập ủ trà thường chỉ nhận ra khác biệt này khi pha thử hai ấm cạnh nhau.</p>
    <blockquote>Uống chậm một chén trà ngon còn hơn uống vội cả một ấm lớn.</blockquote>
    <h2>Ghi lại từng lượt ủ</h2>
    <p>Mỗi lượt ủ nên được ghi lại theo cùng một khuôn để về sau dễ so sánh. Bảng ghi dưới đây dùng ba cột đơn giản: lượt, thời gian ủ và nhận xét ngắn.</p>
    <pre>luot 1 | 40 giay | thom nhe
luot 2 | 60 giay | dam hon</pre>
    <p>Sau vài tuần ghi chép đều đặn, người pha sẽ tự tìm được thời gian ủ hợp với khẩu vị của mình mà không cần dựa vào bất kỳ công thức cố định nào.</p>
  </article>
</main>
<footer>
  <p>Trang ghi chép cá nhân, dữ liệu kiểm thử tự viết.</p>
</footer>
</body>
</html>
"#;

/// Trang CHỦ tự viết — cùng hình dạng a07: **0** thẻ `<p>`, nội dung chỉ gồm tiêu đề, mục danh
/// sách và `div` lá mang nhãn ngắn.
const HAND_WRITTEN_ZERO_PARAGRAPH_HOME_HTML: &str = r#"<!DOCTYPE html>
<html lang="vi">
<head><meta charset="utf-8"><title>Sổ tay đồ uống</title></head>
<body>
<div id="khung">
  <div class="dau-trang">
    <ul class="menu">
      <li><a href="/">Trang chủ</a></li>
      <li><a href="/tra">Trà</a></li>
      <li><a href="/ca-phe">Cà phê</a></li>
    </ul>
  </div>
  <div class="noi-dung">
    <h2>Mới cập nhật</h2>
    <ul class="danh-sach">
      <li><a href="/tra/u-tra-xanh">Ghi chép về cách ủ trà xanh tại nhà</a> <span>2 giờ trước</span></li>
      <li><a href="/tra/tra-o-long">Ba lượt ủ đầu tiên của một ấm trà ô long</a> <span>5 giờ trước</span></li>
      <li><a href="/ca-phe/pha-phin">Pha cà phê phin chậm cho buổi sáng cuối tuần</a> <span>1 ngày trước</span></li>
      <li><a href="/tra/tra-lanh">Làm trà lạnh từ lá trà khô trong tủ mát</a> <span>2 ngày trước</span></li>
    </ul>
    <h2>Đọc nhiều</h2>
    <div class="the">
      <h3>Cách chọn ấm cho người mới bắt đầu</h3>
      <div class="tom-tat">Một chiếc ấm nhỏ bằng sứ trắng giúp nhìn rõ màu nước trà ở từng lượt ủ, nhờ vậy dễ nhận ra lúc nước bắt đầu đậm quá mức mong muốn.</div>
    </div>
    <div class="the">
      <h3>Nước máy, nước lọc hay nước suối</h3>
      <div class="tom-tat">So sánh ngắn ba loại nước quen thuộc khi dùng để ủ cùng một loại lá trà, với cùng nhiệt độ và cùng thời gian ủ cho mỗi lượt.</div>
    </div>
    <div class="the">
      <h3>Bảo quản lá trà khô qua mùa nồm</h3>
      <div class="tom-tat">Đựng lá trà trong hộp kín, tránh ánh nắng trực tiếp và tránh để gần các loại gia vị có mùi mạnh trong bếp.</div>
    </div>
  </div>
  <div class="chan-trang">Sổ tay đồ uống, dữ liệu kiểm thử tự viết.</div>
</div>
</body>
</html>
"#;

/// 🔴 **Đối chứng đỏ ② trên fixture tự viết** — cùng mệnh đề với
/// [`extract_covers_all_seven_bench_fixtures_without_losing_headings_or_list_items`] (bản đó
/// cần bảy trang THẬT không commit được, nên `#[ignore]`): bộ chọn khối phủ heading/`li`/
/// `blockquote`/`pre`, và một trang **0** `<p>` không cho 0 khối.
///
/// Tiền điều kiện được assert TRƯỚC, không mặc định: nếu chữ của heading/`li`/`blockquote`/`pre`
/// KHÔNG có trong `text_content` thì phép so `joined == text_content` đúng sẵn ngay cả với bộ
/// chọn vòng 1 (`p, img, figcaption`) — ca xanh mà không canh gì.
#[test]
fn extract_covers_headings_list_items_and_a_zero_paragraph_page_on_hand_written_fixtures() {
    for tag in ["<h2", "<h3", "<li", "<blockquote", "<pre"] {
        assert!(HAND_WRITTEN_ARTICLE_HTML.contains(tag), "fixture bai viet phai co {tag}");
    }
    assert!(
        !HAND_WRITTEN_ZERO_PARAGRAPH_HOME_HTML.contains("<p"),
        "fixture trang chu phai co DUNG 0 the <p> -- do chinh la hinh dang a07"
    );

    // ── Trang bài viết — trùng TỪNG KÝ TỰ với `text_content`, như a01-a06.
    let url = "https://example.com/bai-viet-tu-viet";
    let blocks = extract(HAND_WRITTEN_ARTICLE_HTML, url).unwrap_or_else(|e| panic!("bai viet: extract that bai: {e:?}"));
    assert!(!blocks.is_empty(), "bai viet: phai cho it nhat mot khoi");

    let text_content = fixture_text_content(HAND_WRITTEN_ARTICLE_HTML, url);
    for must_survive in [
        "Chuẩn bị nước và ấm",
        "Vì sao không dùng nước đang sôi",
        "Khoảng một thìa đầy lá trà khô cho mỗi lượt ủ.",
        "Uống chậm một chén trà ngon còn hơn uống vội cả một ấm lớn.",
        "luot 2 | 60 giay | dam hon",
    ] {
        assert!(
            text_content.contains(must_survive),
            "tien dieu kien: text_content cua Readability phai giu {must_survive:?} -- neu khong, \
             phep so ben duoi dung san ca voi bo chon chi co <p>: {text_content:?}"
        );
    }

    let effective_kept: Vec<bool> = blocks.iter().map(|b| b.machine_kept).collect();
    let joined = auratranslate_lib::core::segment::pipeline::join_kept_blocks(&blocks, &effective_kept);
    assert_eq!(
        joined, text_content,
        "bai viet: van ban ghep tu khoi (0 override) phai trung DUNG TUNG KY TU voi text_content"
    );

    // 🔴 Phép so `joined == text_content` ở trên MÙ với bộ chọn khối. `exact_gap_before` là TRỌN
    // đoạn `text_content` giữa hai khối giữ liền nhau, không chỉ khoảng trắng, nên chữ của một
    // heading/`li` KHÔNG được chọn làm khối vẫn lọt vào `joined` qua khoảng đệm của `<p>` kế
    // tiếp. Đo 2026-09-11 bằng phép GỠ: đưa `TEXT_BLOCK_TAGS`/`BLOCK_SELECTOR` về vòng 1
    // (`p, img, figcaption`) thì ca này XANH khi chỉ có phép so văn bản — cả ca `#[ignore]` bảy
    // mẫu thật ở trên cũng xanh. Mệnh đề về bộ chọn phải hỏi ở tầng KHỐI.
    let kept = kept_text_bodies(&blocks);
    for own_block in [
        "Chuẩn bị nước và ấm",
        "Vì sao không dùng nước đang sôi",
        "Khoảng một thìa đầy lá trà khô cho mỗi lượt ủ.",
        "Uống chậm một chén trà ngon còn hơn uống vội cả một ấm lớn.",
    ] {
        assert!(
            kept.contains(&own_block),
            "bai viet: {own_block:?} phai la MOT khoi dang giu cua CHINH NO -- bo chon khoi khong phu the do: {kept:?}"
        );
    }
    assert!(
        kept.iter().any(|t| t.starts_with("luot 1 | 40 giay")),
        "bai viet: khoi <pre> phai la MOT khoi dang giu cua chinh no: {kept:?}"
    );

    // ── Trang chủ 0 `<p>` — hai đối chứng spec đòi cho a07: không 0 khối, văn bản không rỗng.
    let url = "https://example.com/trang-chu-tu-viet";
    let blocks = extract(HAND_WRITTEN_ZERO_PARAGRAPH_HOME_HTML, url)
        .unwrap_or_else(|e| panic!("trang chu: extract that bai: {e:?}"));
    assert!(!blocks.is_empty(), "trang chu 0 <p>: phai cho it nhat mot khoi -- truoc ban va 6.9 no cho 0 khoi");
    let effective_kept: Vec<bool> = blocks.iter().map(|b| b.machine_kept).collect();
    let joined = auratranslate_lib::core::segment::pipeline::join_kept_blocks(&blocks, &effective_kept);
    assert!(!joined.trim().is_empty(), "trang chu 0 <p>: van ban ghep tu khoi dang giu khong duoc rong");

    // 🔴 Hai assert trên cũng MÙ: khi không còn khối giữ nào, lưới an toàn của `build_blocks` đẩy
    // NGUYÊN `text_content` thành một khối — "không 0 khối" đúng sẵn. Hỏi thẳng: lưới đó KHÔNG
    // được là thứ đỡ trang này.
    let home_text_content = fixture_text_content(HAND_WRITTEN_ZERO_PARAGRAPH_HOME_HTML, url);
    let kept = kept_text_bodies(&blocks);
    assert!(
        kept.len() >= 2 && !kept.contains(&home_text_content.as_str()),
        "trang chu 0 <p>: phai co it nhat HAI khoi dang giu tu chinh bo chon, khong phai mot khoi luoi an toan chua nguyen text_content: {kept:?}"
    );
}

/// Thân chữ của mọi khối `Paragraph`/`Caption` đang `machine_kept`, theo thứ tự tài liệu.
fn kept_text_bodies(blocks: &[auratranslate_lib::core::webimport::Block]) -> Vec<&str> {
    blocks
        .iter()
        .filter(|b| b.machine_kept)
        .filter_map(|b| match &b.body {
            BlockBody::Paragraph(t) | BlockBody::Caption(t) => Some(t.as_str()),
            BlockBody::Image { .. } => None,
        })
        .collect()
}

// ═════════════════════════════════════════════════════════════════════════════════
// A2 (vòng rà đối kháng 2, 3 lớp) — bất biến trung tâm của `outcome`: MỌI bản ghi `Allowed`
// đã HOÀN TẤT (fetch() đã trả về) phải mang `outcome.is_some()` — chính lời tuyên bố của
// `domain_log.rs:107-112`. Quét qua NHIỀU tình huống thật (thành công/404/cổng chết/vòng lặp
// chuyển hướng) thay vì một fixture đơn — một vị từ đúng trên MỘT tình huống không chứng
// minh được gì cho các tình huống khác.
// ═════════════════════════════════════════════════════════════════════════════════

#[test]
fn every_completed_allowed_domain_log_entry_carries_a_non_none_outcome_across_several_real_scenarios() {
    let mut all_entries: Vec<auratranslate_lib::core::webimport::DomainLogEntry> = Vec::new();

    // (a) Thành công thẳng.
    let (port_ok, _h_ok) = spawn_once(|mut s| {
        let _ = s.write_all(ok_html_response("hi").as_bytes());
    });
    let url_ok = format!("http://127.0.0.1:{port_ok}/ok");
    let allowlist_ok = Allowlist::from_urls([url_ok.as_str()]);
    let (_r, log_ok) = fetch(&url_ok, &allowlist_ok, ResourceKind::Page);
    all_entries.extend(log_ok);

    // (b) 404.
    let (port_404, _h_404) = spawn_once(|mut s| {
        let _ = s.write_all(b"HTTP/1.1 404 Not Found\r\nContent-Length: 0\r\n\r\n");
    });
    let url_404 = format!("http://127.0.0.1:{port_404}/khong-co");
    let allowlist_404 = Allowlist::from_urls([url_404.as_str()]);
    let (_r, log_404) = fetch(&url_404, &allowlist_404, ResourceKind::Page);
    all_entries.extend(log_404);

    // (c) Cổng chết (ConnectFailed/Timeout).
    let dead_port = {
        let l = TcpListener::bind("127.0.0.1:0").expect("bind");
        l.local_addr().expect("addr").port()
    };
    let url_dead = format!("http://127.0.0.1:{dead_port}/chet");
    let allowlist_dead = Allowlist::from_urls([url_dead.as_str()]);
    let (_r, log_dead) = fetch(&url_dead, &allowlist_dead, ResourceKind::Page);
    all_entries.extend(log_dead);

    // (d) Chuyển hướng ĐƯỢC theo tới một host khác, thành công ở chặng cuối.
    let (port_b, _h_b) = spawn_once(|mut s| {
        let _ = s.write_all(ok_html_response("dich").as_bytes());
    });
    let location = format!("http://localhost:{port_b}/final");
    let (port_a, _h_a) = spawn_once(move |mut s| {
        let _ = s.write_all(
            format!("HTTP/1.1 301 Moved Permanently\r\nLocation: {location}\r\nContent-Length: 0\r\n\r\n")
                .as_bytes(),
        );
    });
    let url_a = format!("http://127.0.0.1:{port_a}/dau");
    let allowlist_ab = Allowlist::from_urls([url_a.as_str(), format!("http://localhost:{port_b}/final").as_str()]);
    let (_r, log_redirect) = fetch(&url_a, &allowlist_ab, ResourceKind::Page);
    all_entries.extend(log_redirect);

    assert!(
        all_entries.len() >= 4,
        "phai gop duoc it nhat bon ban ghi tu bon tinh huong khac nhau: {all_entries:?}"
    );

    let offenders: Vec<_> = all_entries
        .iter()
        .filter(|e| matches!(e.decision, DomainLogDecision::Allowed(_)) && e.outcome.is_none())
        .collect();
    assert!(
        offenders.is_empty(),
        "MOI ban ghi Allowed da HOAN TAT (fetch() da tra ve) phai mang outcome.is_some() -- \
         day la bat bien domain_log.rs tuyen bo; vi pham: {offenders:?}"
    );

    // Ca ÂM đi kèm — một bản ghi Denied (0 kết nối) HỢP LỆ giữ outcome None, không bị vị từ
    // trên bắt oan.
    let denied_present = all_entries.iter().any(|e| matches!(e.decision, DomainLogDecision::Denied));
    if denied_present {
        assert!(
            all_entries
                .iter()
                .filter(|e| matches!(e.decision, DomainLogDecision::Denied))
                .all(|e| e.outcome.is_none()),
            "mot ban ghi Denied PHAI giu outcome None (0 ket noi, khong co gi de bao cao)"
        );
    }
}

// ═════════════════════════════════════════════════════════════════════════════════
// D6 (vòng rà đối kháng 2, 3 lớp) — hai bộ dựng SẢN PHẨM DUY NHẤT của `PipelineShape::Chapters`
// phải luôn cho ra một danh sách ĐỒNG NHẤT (toàn `RawBytes`) — `commands/project.rs` chỉ đọc
// `cs.first()` để quyết định `extract_main_content` cho CẢ danh sách (§Ask First — sửa đúng
// cần một cờ THEO TỪNG Chương, ngoài phạm vi lượt vá này, xem `deferred-work.md`); nếu một
// trong hai hàm dựng này lỡ trộn hình dạng, mục sau `RawBytes` đầu tiên sẽ bị bỏ qua pha ảnh
// ÂM THẦM. Ca này khoá bất biến "luôn đồng nhất" tại đúng hai điểm dựng, để một lượt sửa sau
// này lỡ phá nó bị bắt ở NGUỒN, không phải đoán qua hành vi `extract_main_content`.
// ═════════════════════════════════════════════════════════════════════════════════

#[test]
fn the_two_real_chapters_shape_builders_always_produce_a_homogeneous_list_of_raw_bytes() {
    let items = vec![
        UrlImportItem { url: "https://a.example/1".to_owned(), raw: Some(b"<html>a</html>".to_vec()), error: None },
        UrlImportItem { url: "https://a.example/2".to_owned(), raw: Some(b"<html>b</html>".to_vec()), error: None },
        UrlImportItem {
            url: "https://a.example/3".to_owned(),
            raw: None,
            error: Some(web_import_item_failure_ipc_error(
                "https://a.example/3",
                WebImportItemFailureReason::InvalidUrl,
                None,
            )),
        },
    ];

    fn assert_homogeneous_raw_bytes(shape: &PipelineShape) {
        match shape {
            PipelineShape::Chapters(cs) => {
                assert!(!cs.is_empty(), "danh sach khong duoc rong");
                assert!(
                    cs.iter().all(|c| matches!(c, ChapterInput::RawBytes { .. })),
                    "MOI don vi phai la RawBytes -- mot danh sach TRON HINH DANG lam \
                     `extract_main_content` (doc `cs.first()`, commands/project/mod.rs) doc SAI \
                     cho cac muc sau: {cs:?}"
                );
            }
            PipelineShape::Blob(_) => panic!("hai ham dung nay phai cho Chapters, khong Blob"),
            // 🔵 THÊM 2026-09-11 (Story 6.16) — `PipelineShape` co them mot bien the moi
            // (`Bilingual`); hai ham dung nay chi con duoc goi voi `Chapters` hom nay va se
            // van vay sau story 6.16 (duong song ngu khong di qua `chapters_shape_*`).
            PipelineShape::Bilingual { .. } => {
                panic!("hai ham dung nay phai cho Chapters, khong Bilingual")
            }
            // 🔵 THÊM 2026-09-15 (Story 6.6b) — `PipelineShape` co them mot bien the moi
            // (`Files`); hai ham dung nay chi con duoc goi voi `Chapters` hom nay (duong N
            // tep khong di qua `chapters_shape_*` -- no di qua `import_files`).
            PipelineShape::Files(_) => panic!("hai ham dung nay phai cho Chapters, khong Files"),
        }
    }

    let write_shape = chapters_shape_if_all_ok(&items[..2]).expect("hai muc OK phai dung duoc shape GHI");
    assert_homogeneous_raw_bytes(&write_shape);

    let view_shape = chapters_shape_for_view(&items).expect("vi tu XEM phai dung duoc voi muc hong xen giua");
    assert_homogeneous_raw_bytes(&view_shape);
}

// ═════════════════════════════════════════════════════════════════════════════════
// Debt probe — Story 6.18 task 6: "Mỗi link dựng một `reqwest::blocking::Client` MỚI"
// (deferred-work.md, cụm "Deferred from: 6-7…", Chủ Story 6.18) — chi phí đo TRÊN danh sách
// LỚN (100 và 1.000 link), không suy tuyến tính từ N=20 của `perf_probe_twenty_links…`
// (Ice cấm suy tuyến tính, ghi ngay tại ca đó).
// ═════════════════════════════════════════════════════════════════════════════════
//
// `#[ignore]` — cùng lý lẽ `perf_probe_twenty_links…` phía trên nhưng nặng hơn 5×/50× số
// luồng server cục bộ; không phải một cổng `cargo test --locked` mặc định. Chạy tay:
//   cargo test --locked --test webimport_contract -- --ignored --nocapture client_per_link
fn perf_probe_client_per_link_cost(n: usize) {
    fn tiny_html_page(i: usize) -> String {
        format!(
            "<html><head><title>Bai {i}</title></head><body><article><h1>Tieu de {i}</h1>\
             <p>Mot doan van ban ngan de vuot nguong do dai toi thieu cho khoi noi dung, \
             khong phai menu hay quang cao xung quanh no, danh cho lien ket thu {i}.</p>\
             </article></body></html>"
        )
    }

    let mut ports = Vec::with_capacity(n);
    for i in 0..n {
        let (port, _handle) = spawn_once(move |mut stream| {
            let _ = stream.write_all(ok_html_response(&tiny_html_page(i)).as_bytes());
        });
        ports.push(port);
    }
    // Cùng khoảng nghỉ đã đo cần thiết ở `perf_probe_twenty_links…` (luồng server đầu tiên
    // của một loạt `spawn_once` liên tiếp cần thời gian THẬT để hệ điều hành lên lịch).
    thread::sleep(Duration::from_millis(200));
    let urls: Vec<String> = ports.iter().map(|p| format!("http://127.0.0.1:{p}/a")).collect();

    let t0 = std::time::Instant::now();
    let (items, _log) = fetch_url_import_items(urls);
    let elapsed = t0.elapsed();

    let ok_count = items.iter().filter(|it| it.error.is_none()).count();
    let per_link_ms = elapsed.as_secs_f64() * 1000.0 / n as f64;

    println!(
        "PERF_PROBE_CLIENT_PER_LINK\tn={n}\tok={ok_count}\ttotal_ms={:.1}\tper_link_ms={per_link_ms:.2}",
        elapsed.as_secs_f64() * 1000.0
    );
    // ⚠️ Cùng cảnh báo tải máy của `perf_probe_twenty_links…` áp dụng ở đây — con số `ms`
    // không đáng tin như một "tốc độ", chỉ đáng tin như bằng chứng CHIỀU TĂNG (n=100 so
    // n=1.000) của chi phí dựng-Client-mới-mỗi-link. Xem §Implementation Notes spec 6.18 cho
    // con số thật đo được và verdict.
    assert_eq!(ok_count, n, "moi trong so {n} link phai tai OK -- mot muc hong lam do sai lech");
    assert!(elapsed.as_secs_f64() < 300.0, "{n} link cuc bo mat qua 300s -- nghi treo that");
}

#[test]
#[ignore = "nang: dung n luong server cuc bo + n lan goi Client moi -- khong phai mot cong mac dinh"]
fn perf_probe_client_per_link_cost_on_one_hundred_links() {
    perf_probe_client_per_link_cost(100);
}

#[test]
#[ignore = "nang: dung 1000 luong server cuc bo + 1000 lan goi Client moi -- khong phai mot cong mac dinh"]
fn perf_probe_client_per_link_cost_on_one_thousand_links() {
    perf_probe_client_per_link_cost(1_000);
}
