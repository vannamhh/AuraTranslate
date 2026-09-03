//! Mũi thăm dò Story 6.1 — spec `spec-6-1-mui-tham-do-ba-lua-chon-thu-vien.md`.
//!
//! ─────────────────────────────────────────────────────────────────────────────
//! 🔴 ĐÂY LÀ MỘT BÀN ĐO, KHÔNG PHẢI `Fetcher`/`Extractor` THẬT
//! ─────────────────────────────────────────────────────────────────────────────
//! Ba hàm `#[ignore]` dưới đây đo ba giả định của Epic 6 trên dữ liệu thật: `dom_smoothie`
//! bóc được nội dung chính hay không (FR123), `chardetng` dò đúng bảng mã hay không (FR126),
//! và `reqwest` có ba năng lực mà `Fetcher` cần hay không (chặn chuyển hướng theo từng chặng,
//! giới hạn kích thước theo dòng chảy, báo lỗi mạng). File này KHÔNG chạm `core/webimport/`
//! ngoài việc đọc doc-comment của nó, không dựng `Fetcher`/`Extractor`, và không viết một
//! dòng nào của pipeline nhập — đó là Story 6.2/6.3/6.9. Mã dưới đây gọi thẳng ba crate ứng
//! viên bằng cách rẻ nhất có thể, chỉ để lấy số đo.
//!
//! Ba hàm `#[ignore]` vì đây là số đo trên mạng thật/dữ liệu thật, không phải một cổng
//! đúng/sai — đúng khuôn ba tiền lệ đã có: `library_index_contract.rs`, `segment_contract.rs`,
//! `dict_sources.rs`. Chạy tay:
//!
//!     cargo test --locked --manifest-path src-tauri/Cargo.toml --test webimport_probe \
//!       -- --ignored --nocapture
//!
//! ─────────────────────────────────────────────────────────────────────────────
//! VÌ SAO KHÔNG `use dom_query::...` HAY `use tendril::...`
//! ─────────────────────────────────────────────────────────────────────────────
//! Cả hai crate là phụ thuộc BẮC CẦU của `dom_smoothie` — không khai tường minh trong
//! `Cargo.toml` — nên `tests/**` không thấy được tên của chúng (chỉ thấy `[dependencies]`
//! khai tường minh, đúng Design Notes của spec này). `dom_smoothie::Article::content`/
//! `text_content` có kiểu `StrTendril`, nhưng nó triển khai `Deref<Target = str>`, nên gọi
//! phương thức của `str` qua auto-deref (`.chars()`, `.matches(...)`) biên dịch được mà
//! không cần đặt tên `tendril` ở đâu cả. Đếm đoạn dùng đúng cách này — đếm số lần khớp
//! `"<p"` trong `content` — là một XẤP XỈ (đếm cả `<pre`), đủ cho một mũi thăm dò, không
//! đủ cho một cổng nghiệm thu; ghi rõ ở cột tiêu đề TSV.
//!
//! ─────────────────────────────────────────────────────────────────────────────
//! VÌ SAO REQWEST DÙNG SERVER LOCALHOST TỰ DỰNG, KHÔNG PHẢI EPOCHTIMES.COM
//! ─────────────────────────────────────────────────────────────────────────────
//! Ba năng lực cần đo (chặn chuyển hướng, cắt theo dòng chảy, báo lỗi mạng) là hành vi
//! GIAO THỨC của `reqwest`, không phụ thuộc nội dung trang — một server thật không cho
//! phép ta ĐIỀU KHIỂN được chuỗi chuyển hướng hay kích thước thân trả về một cách tất
//! định. Server cục bộ (`127.0.0.1`, cổng hệ điều hành cấp) làm phép đo lặp lại được và
//! không phụ thuộc một site ngoài còn sống hay không.

use std::fs;
use std::io::{Read as _, Write as _};
use std::net::{TcpListener, TcpStream};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use chardetng::{EncodingDetector, Iso2022JpDetection, Utf8Detection};
use dom_smoothie::Readability;

/// Thư mục bàn giao của story — khuôn `5-14-ban-do/`. `CARGO_MANIFEST_DIR` của crate này
/// là `src-tauri/`, nên bàn giao nằm một cấp lên rồi vào `_bmad-output/...`.
fn ban_do_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("_bmad-output/implementation-artifacts/6-1-ban-do")
}

fn write_tsv(name: &str, header: &str, rows: &[String]) {
    let path = ban_do_dir().join(name);
    let mut out = String::new();
    out.push_str(header);
    out.push('\n');
    for row in rows {
        out.push_str(row);
        out.push('\n');
    }
    fs::write(&path, out).unwrap_or_else(|e| panic!("ghi {}: {e}", path.display()));
    println!("BAN_DO_TSV\t{}\t{} hàng", path.display(), rows.len());
}

/// User-Agent thật — nhiều site (kể cả epochtimes.com) trả 403 cho UA mặc định của
/// `reqwest` (`reqwest/x.y.z`).
const UA: &str = "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 \
     (KHTML, like Gecko) Chrome/120.0 Safari/537.36";

// ═════════════════════════════════════════════════════════════════════════════════
// Bàn đo 1 — `dom_smoothie` bóc nội dung chính từ bài báo THẬT (FR123).
// ═════════════════════════════════════════════════════════════════════════════════
//
// Nguồn: `urls.txt` (commit được) — danh sách bài báo epochtimes.com thật, cộng một trang
// KHÔNG phải bài (trang chủ ấn bản Phồn thể) cho ca "trang không phải bài" của ma trận I/O.
// HTML tải về nằm trong `fixtures/html/` (gitignore) — tự tải nếu chưa có cache, để lượt
// chạy lại không phụ thuộc mạng nữa. Chân lý nền cho "bóc đúng hay sai" là một lượt xử của
// người trên `extraction-raw.tsv`, ghi vào `REPORT.md` — bài kiểm này KHÔNG assert một
// ngưỡng tỉ lệ, đúng Design Notes của spec ("chân lý nền là một lượt xử của người").
#[test]
#[ignore = "ban do mang that: doc bai bao epochtimes.com that, khong phai mot cong"]
fn dom_smoothie_records_one_tsv_row_of_extraction_measurements_per_real_fetched_article() {
    let urls_path = ban_do_dir().join("urls.txt");
    let urls_raw = fs::read_to_string(&urls_path)
        .unwrap_or_else(|e| panic!("đọc {}: {e}", urls_path.display()));
    let urls: Vec<&str> = urls_raw
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty() && !l.starts_with('#'))
        .collect();
    assert!(
        !urls.is_empty(),
        "0 URL trong {} — lỗi hạ tầng của bàn đo (tệp phải commit kèm ít nhất một bài \
         báo thật), không phải một phép đo tỉ lệ 0%",
        urls_path.display()
    );

    let fixtures_dir = ban_do_dir().join("fixtures/html");
    fs::create_dir_all(&fixtures_dir)
        .unwrap_or_else(|e| panic!("tạo {}: {e}", fixtures_dir.display()));

    let client = reqwest::blocking::Client::builder()
        .user_agent(UA)
        .timeout(Duration::from_secs(20))
        .build()
        .expect("dựng client reqwest");

    let mut rows = Vec::new();
    let mut fetch_ok = 0usize;
    let mut fetch_err = 0usize;
    let mut extract_ok = 0usize;
    let mut extract_err = 0usize;

    for (idx, url) in urls.iter().enumerate() {
        let id = format!("a{:02}", idx + 1);
        let cache_path = fixtures_dir.join(format!("{id}.html"));

        let html = if cache_path.is_file() {
            fs::read_to_string(&cache_path).ok()
        } else {
            match client.get(*url).send().and_then(|r| r.error_for_status()) {
                Ok(resp) => match resp.text() {
                    Ok(body) => {
                        let _ = fs::write(&cache_path, &body);
                        Some(body)
                    }
                    Err(e) => {
                        println!("EXTRACT_SAMPLE\t{id}\tfetch_err\tđọc thân trả về: {e}");
                        None
                    }
                },
                Err(e) => {
                    println!("EXTRACT_SAMPLE\t{id}\tfetch_err\t{e}");
                    None
                }
            }
        };

        let Some(html) = html else {
            fetch_err += 1;
            rows.push(format!(
                "{id}\t{url}\t\t\t\t\t\t\tfetch_err"
            ));
            continue;
        };
        fetch_ok += 1;

        let mut readability = Readability::new(html, Some(url), None)
            .unwrap_or_else(|e| panic!("{id}: URL không tuyệt đối ({url}): {e}"));
        let is_readable = readability.is_probably_readable();

        match readability.parse() {
            Ok(article) => {
                extract_ok += 1;
                let char_count = article.text_content.chars().count();
                // Xấp xỉ: đếm khớp "<p" trong HTML đã bóc — xem doc-comment đầu file.
                let paragraph_count = article.content.matches("<p").count();
                let text: &str = &article.text_content;
                let chars: Vec<char> = text.chars().collect();
                let head: String = chars.iter().take(80).collect();
                let tail: String = chars.iter().rev().take(80).collect::<Vec<_>>().into_iter().rev().collect();
                let note = if is_readable { "" } else { "khong_giong_bai_viet" };
                rows.push(format!(
                    "{id}\t{url}\t{is_readable}\t{char_count}\t{paragraph_count}\t{}\t{}\t{}\t{note}",
                    tsv_escape(&article.title),
                    tsv_escape(&head),
                    tsv_escape(&tail),
                ));
            }
            Err(e) => {
                extract_err += 1;
                rows.push(format!(
                    "{id}\t{url}\t{is_readable}\t\t\t\t\t\textract_err: {e}"
                ));
            }
        }
    }

    write_tsv(
        "extraction-raw.tsv",
        "id\turl\tis_probably_readable\tchar_count\tparagraph_count_approx\ttitle\tfirst_80_chars\tlast_80_chars\tnote",
        &rows,
    );
    println!(
        "EXTRACT_SUMMARY\tsamples={}\tfetch_ok={fetch_ok}\tfetch_err={fetch_err}\textract_ok={extract_ok}\textract_err={extract_err}",
        urls.len()
    );

    assert_eq!(rows.len(), urls.len(), "phải ghi đúng một hàng cho mỗi URL, không bỏ sót");
}

fn tsv_escape(s: &str) -> String {
    s.replace('\t', " ").replace('\n', " ").replace('\r', " ")
}

// ═════════════════════════════════════════════════════════════════════════════════
// Bàn đo 2 — `chardetng` dò bảng mã trên `.txt` GBK/Big5 THẬT (FR126).
// ═════════════════════════════════════════════════════════════════════════════════
//
// Fixture do Ice tự cấp vào `fixtures/encoding/` (gitignore), KHÔNG tự sinh bằng cách mã
// hoá ngược từ UTF-8 — xem "Never" của spec: mã hoá bằng `encoding_rs` rồi bảo `chardetng`
// đọc lại là một vòng tròn. Quy ước tên tệp: `<mô-tả>__<NHÃN>.txt`, `NHÃN` là một trong năm
// bảng của FR126 (`UTF-8` · `GB18030` · `GBK` · `BIG5` · `UTF-16`, không phân biệt hoa
// thường) — xem `README.md` của thư mục này. 0 tệp ⇒ bàn đo THOÁT KHÁC 0 (assert đỏ) —
// phân biệt tường minh với "đã đo, tỉ lệ 0%", đúng ma trận I/O của spec.
#[test]
#[ignore = "ban do can fixture that cua Ice trong fixtures/encoding/, khong phai mot cong"]
fn chardetng_records_the_true_and_guessed_label_of_every_encoding_fixture_or_fails_loudly_on_zero_samples(
) {
    let dir = ban_do_dir().join("fixtures/encoding");
    let mut files: Vec<PathBuf> = fs::read_dir(&dir)
        .into_iter()
        .flatten()
        .filter_map(Result::ok)
        .map(|e| e.path())
        .filter(|p| p.extension().map(|e| e == "txt").unwrap_or(false))
        .collect();
    files.sort();

    println!("ENCODING_SAMPLES\t{}", files.len());
    assert!(
        !files.is_empty(),
        "0 mẫu trong {} — Ice CHƯA cấp fixture GBK/Big5 thật. Đây là LỖI HẠ TẦNG của bàn \
         đo, không phải một phép đo với tỉ lệ đúng 0% — ghi nợ vào deferred-work.md, chủ \
         Ice, không suy phán quyết FR126 từ đây.",
        dir.display()
    );

    let mut rows = Vec::new();
    let mut matched = 0usize;
    for path in &files {
        let file_name = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("")
            .to_string();
        let Some((sample_id, true_label)) = file_name.rsplit_once("__") else {
            rows.push(format!(
                "{file_name}\t\tKHONG_DOC_DUOC_TEN\t\t\tten_tep_thieu_dau___LABEL"
            ));
            continue;
        };

        let bytes = fs::read(path).unwrap_or_else(|e| panic!("đọc {}: {e}", path.display()));
        let mut detector = EncodingDetector::new(Iso2022JpDetection::Deny);
        detector.feed(&bytes, true);
        let guess = detector.guess(None, Utf8Detection::Allow);

        let norm_true = normalize_label(true_label);
        let norm_guess = normalize_label(guess.name());
        let is_match = norm_true == norm_guess
            || (norm_true == "UTF16" && norm_guess.starts_with("UTF16"));
        if is_match {
            matched += 1;
        }

        rows.push(format!(
            "{sample_id}\t{true_label}\t{}\t{}\t{}\t{}",
            guess.name(),
            is_match,
            bytes.len(),
            path.file_name().and_then(|n| n.to_str()).unwrap_or(""),
        ));
    }

    write_tsv(
        "encoding-raw.tsv",
        "id\ttrue_label\tguessed_label\tmatch\tbyte_len\tfile_name",
        &rows,
    );
    println!(
        "ENCODING_SUMMARY\tsamples={}\tmatched={matched}",
        files.len()
    );
}

/// `"GB18030"` == `"gb18030"` == `"gb-18030"`; `"UTF-16"` khớp riêng ở chỗ gọi (LE/BE).
fn normalize_label(s: &str) -> String {
    s.chars()
        .filter(|c| c.is_ascii_alphanumeric())
        .map(|c| c.to_ascii_uppercase())
        .collect()
}

// ═════════════════════════════════════════════════════════════════════════════════
// Bàn đo 3 — ba năng lực `reqwest` cần cho `Fetcher` (chặn chuyển hướng theo chặng, cắt
// thân theo dòng chảy, báo lỗi mạng). Server cục bộ — xem doc-comment đầu file.
// ═════════════════════════════════════════════════════════════════════════════════
#[test]
#[ignore = "ban do dung server localhost, chay duoc nhung tach khoi cargo test mac dinh"]
fn reqwest_blocks_a_cross_host_redirect_caps_a_streamed_body_and_reports_a_dead_connection() {
    let mut rows = Vec::new();

    rows.push(redirect_case());
    rows.push(size_cap_case());
    rows.push(network_failure_case());

    write_tsv(
        "reqwest-raw.tsv",
        "scenario\tresult\tdetail",
        &rows,
    );
}

/// Server A trả 301 sang server B (cổng khác = "host khác" giả lập). Chính sách chặn
/// dựa trên PORT đích (đứng cho domain) và server B không bao giờ được nối tới.
fn redirect_case() -> String {
    let reached_b = Arc::new(AtomicU64::new(0));
    let reached_b_clone = Arc::clone(&reached_b);
    let (port_b, _handle_b) = spawn_once(move |mut stream| {
        reached_b_clone.fetch_add(1, Ordering::SeqCst);
        let _ = stream.write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 2\r\n\r\nhi");
    });

    let location = format!("http://127.0.0.1:{port_b}/final");
    let location_for_server = location.clone();
    let (port_a, _handle_a) = spawn_once(move |mut stream| {
        let body = format!(
            "HTTP/1.1 301 Moved Permanently\r\nLocation: {location_for_server}\r\nContent-Length: 0\r\n\r\n"
        );
        let _ = stream.write_all(body.as_bytes());
    });

    let chain: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));
    let chain_clone = Arc::clone(&chain);
    // "Allowlist" của phép đo: chỉ port của chính server A được đi tiếp — bất kỳ port
    // nào khác (đứng cho domain khác) bị `stop()`, đúng hành vi AD-41 cần từ `Fetcher`.
    let policy = reqwest::redirect::Policy::custom(move |attempt| {
        chain_clone.lock().unwrap_or_else(|e| e.into_inner()).push(attempt.url().to_string());
        if attempt.url().port() == Some(port_a) {
            attempt.follow()
        } else {
            attempt.stop()
        }
    });

    let client = reqwest::blocking::Client::builder()
        .redirect(policy)
        .timeout(Duration::from_secs(5))
        .build()
        .expect("dựng client reqwest");

    let resp = client
        .get(format!("http://127.0.0.1:{port_a}/start"))
        .send()
        .expect("gửi yêu cầu ban đầu");

    let status = resp.status();
    let has_location = resp.headers().contains_key("location");
    let chain_recorded = chain.lock().unwrap_or_else(|e| e.into_inner()).clone();
    let b_reached = reached_b.load(Ordering::SeqCst);

    assert!(status.is_redirection(), "chặn đúng chỗ phải trả nguyên 3xx, không đi tiếp");
    assert!(has_location, "chặn đúng chỗ vẫn phải giữ header Location cho người gọi đọc");
    assert_eq!(chain_recorded, vec![location.clone()], "chuỗi chuyển hướng phải ghi lại được đúng một chặng bị chặn");
    assert_eq!(b_reached, 0, "server bị chặn (khác host/port) TUYỆT ĐỐI không được nhận kết nối");

    format!(
        "redirect_blocked_cross_host\tOK\tstatus={status}; chain={chain_recorded:?}; server_b_reached={b_reached}"
    )
}

/// Server C khai `Content-Length` rất lớn rồi stream thân trả về; client đọc qua
/// `std::io::Read` (không `.bytes()`/`.text()`) và DỪNG khi vượt trần, không nạp trọn.
fn size_cap_case() -> String {
    const ADVERTISED_LEN: usize = 20 * 1024 * 1024; // 20 MiB — đủ lớn để "nạp trọn" là sai lầm rõ ràng.
    const CAP: usize = 1024 * 1024; // 1 MiB — trần giả lập của phép đo.

    let (port, _handle) = spawn_once(move |mut stream| {
        let _ = stream.set_write_timeout(Some(Duration::from_secs(5)));
        let header = format!(
            "HTTP/1.1 200 OK\r\nContent-Type: application/octet-stream\r\nContent-Length: {ADVERTISED_LEN}\r\n\r\n"
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
                // Client đóng sớm (đúng thứ phép đo này cần) ⇒ broken pipe là kết quả
                // ĐÚNG, không phải một lỗi hạ tầng.
                break;
            }
            sent += this_write;
        }
    });

    let client = reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(10))
        .build()
        .expect("dựng client reqwest");
    let mut resp = client
        .get(format!("http://127.0.0.1:{port}/big"))
        .send()
        .expect("gửi yêu cầu");

    let mut buf = [0u8; 64 * 1024];
    let mut read_total = 0usize;
    loop {
        let n = resp.read(&mut buf).unwrap_or(0);
        if n == 0 {
            break;
        }
        read_total += n;
        if read_total >= CAP {
            break; // Cắt theo dòng chảy — drop `resp` ngay sau vòng lặp, không đọc tiếp.
        }
    }
    drop(resp);

    assert!(
        read_total < ADVERTISED_LEN,
        "phải dừng TRƯỚC khi đọc hết {ADVERTISED_LEN} byte quảng cáo"
    );
    assert!(
        read_total >= CAP,
        "phải đọc được ít nhất tới trần {CAP} byte trước khi cắt"
    );

    format!(
        "size_capped_streamed_read\tOK\tadvertised={ADVERTISED_LEN}; cap={CAP}; actually_read={read_total}"
    )
}

/// Không server nào lắng nghe ở cổng này — "mạng hỏng" tất định trên loopback.
fn network_failure_case() -> String {
    // Bind rồi drop ngay để lấy một cổng chắc chắn KHÔNG có ai lắng nghe khi ta gọi tới.
    let port = {
        let listener = TcpListener::bind("127.0.0.1:0").expect("bind cổng tạm");
        listener.local_addr().expect("local_addr").port()
    };

    let client = reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(2))
        .build()
        .expect("dựng client reqwest");
    let result = client.get(format!("http://127.0.0.1:{port}/nope")).send();

    let err = result.expect_err("kết nối phải thất bại — không ai lắng nghe ở cổng này");
    assert!(
        err.is_connect() || err.is_timeout(),
        "lỗi phải được nhận diện là lỗi KẾT NỐI/hết giờ, không phải một loại khác: {err}"
    );

    format!("fetch_err_connection_refused\tOK\t{err}")
}

/// Server tối giản: chấp nhận ĐÚNG MỘT kết nối, đọc và bỏ qua request, gọi `respond` để
/// viết response thô, rồi tự thoát luồng. Trả về cổng đã cấp và `JoinHandle`.
///
/// ⚠️ Chỗ gọi KHÔNG join handle này ở ca chặn chuyển hướng: server "bị chặn" đúng nghĩa
/// là không bao giờ nhận kết nối, nên `.accept()` của nó treo vĩnh viễn — join sẽ treo cả
/// test. Không cần join để lấy đúng: `client.get(...).send()` chỉ trả về sau khi ĐÃ đọc
/// trọn response headers, nên với server THẬT SỰ được gọi tới, `write_all` của nó đã chạy
/// xong trước khi `send()` trả về — đồng bộ hoá tự nhiên qua chính giao thức TCP, không
/// cần một `join()` thêm.
fn spawn_once<F>(respond: F) -> (u16, thread::JoinHandle<()>)
where
    F: FnOnce(TcpStream) + Send + 'static,
{
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind cổng tạm");
    let port = listener.local_addr().expect("local_addr").port();
    let handle = thread::spawn(move || {
        if let Ok((mut stream, _)) = listener.accept() {
            let _ = stream.set_read_timeout(Some(Duration::from_secs(5)));
            // Đọc (và vứt) request tới byte cuối của header — không cần phân tích, ta chỉ
            // cần giải phóng buffer nhận trước khi ghi phản hồi.
            let mut discard = [0u8; 4096];
            let _ = stream.read(&mut discard);
            respond(stream);
        }
    });
    (port, handle)
}
