//! Ba việc THUẦN cho ảnh tải về `.atproj/assets/` — Story 6.11 (FR127).
//!
//! Module này **không đụng đĩa** (không `fs::write`, đó là việc của `commands::project`) và
//! **không quyết định neo vị trí** (đó là `core::segment::anchor`) — đúng ranh giới "một
//! module, một việc" mà `core/webimport/{fetcher,extractor,allowlist}.rs` đã theo.
//!
//! ① [`resolve_absolute_url`] — phân giải `src` của một `<img>` (tuyệt đối/tương đối/rời
//! giao thức `//host/x.jpg`) thành một URL tuyệt đối, theo URL của trang chứa nó.
//! ② [`is_raster_image_mime`] — vị từ MIME ảnh raster, danh mục ĐÓNG bốn kiểu.
//! ③ [`extension_for_mime`] — ánh xạ một MIME ĐÃ CHẤP NHẬN sang đuôi tệp ghi xuống đĩa.
//! ④ [`decode_data_uri_image`] — decodes `data:...;base64,...` images (no network), gated
//! by ②, same closed raster category.
//!
//! `extension_for_mime` is the decision point on the network path
//! (`fetch_and_write_one_asset` reads the HTTP response's `content-type`);
//! `is_raster_image_mime` is the decision point on the `data:` path (no HTTP response to
//! read a `content-type` from) — two paths, two predicates, one [`RASTER_IMAGE_MIMES`]
//! category.
//!
//! `reqwest::Url` hợp lệ ở đây: `webimport_boundary.rs::reqwest_is_named_only_inside_core_webimport_or_core_ai`
//! cho phép `reqwest` trong toàn bộ `core/webimport/`, không riêng `fetcher.rs`/`allowlist.rs`
//! — chỉ `extractor.rs` bị cấm riêng (mệnh đề 2 của tệp đó, AD-40: Extractor không chạm mạng;
//! phân giải một CHUỖI URL không phải "chạm mạng").

/// `src` không phân giải được thành một URL tuyệt đối — trang chứa nó có URL không hợp lệ,
/// hoặc `src` mang một cú pháp URL không hợp lệ (kể cả sau khi đã hợp với URL trang).
#[derive(Debug)]
pub struct ResolveUrlError {
    pub detail: String,
}

/// Phân giải `src` của một `<img>` thành URL TUYỆT ĐỐI, theo URL của trang đã tải nó.
///
/// `reqwest::Url::join` xử lý cả ba hình dạng `src` mà một trang thật mang: tuyệt đối
/// (`https://cdn.example/x.jpg` — trả nguyên vẹn, `join` không đổi một URL đã tuyệt đối),
/// tương đối (`/a.jpg`, `../b.png` — hợp với `page_url`), và RỜI GIAO THỨC (`//cdn/x.jpg` —
/// `join` mượn giao thức của `page_url`, đúng ngữ nghĩa RFC 3986 §5.3 mà mọi trình duyệt
/// theo).
///
/// 🔵 **THÊM 2026-09-08 (Story 6.11, D5+D6 vòng rà đối kháng 3 lớp) — hai chặn SỚM.**
/// - `src` RỖNG hoặc chỉ khoảng trắng, hoặc chỉ mang một `#fragment` (`""`, `"   "`,
///   `"#top"`) — `Url::join` phân giải cả ba thành CHÍNH `page_url` (đúng ngữ nghĩa RFC 3986:
///   một tham chiếu rỗng/chỉ-fragment trỏ về TÀI LIỆU HIỆN TẠI). Không chặn ở đây, host của
///   CHÍNH trang đang nhập lọt vào tầng 2 như một "ảnh", và `fetch(..., ResourceKind::Image)`
///   sẽ tải lại TOÀN BỘ trang HTML dưới danh nghĩa ảnh — lãng phí một lượt gọi mạng cho một
///   `<img>` không hề có nguồn.
/// - Scheme khác `http`/`https` (`data:`, `javascript:`, `file:`, …) — `data:` đặc biệt nguy
///   hiểm: một `src` dạng `data:image/svg+xml;base64,...` không đi qua tầng 2 CHÚT NÀO (không
///   host để mà hỏi allowlist) và mang theo byte ĐÃ CÓ SẴN, không phải một cuộc tải — Story
///   6.11 chỉ có nghĩa cho ảnh THẬT SỰ ở xa. Chặn tại nguồn thay vì để nó trôi tới allowlist
///   rồi bị `host_of` trả `None` một cách khó hiểu.
pub fn resolve_absolute_url(src: &str, page_url: &str) -> Result<String, ResolveUrlError> {
    if src.trim().is_empty() {
        return Err(ResolveUrlError { detail: "src rong hoac chi khoang trang".to_owned() });
    }
    if src.trim().starts_with('#') {
        return Err(ResolveUrlError { detail: format!("src chi mang fragment, khong phai mot nguon anh: {src}") });
    }

    let base = reqwest::Url::parse(page_url)
        .map_err(|e| ResolveUrlError { detail: format!("url trang khong hop le ({page_url}): {e}") })?;
    let joined = base
        .join(src)
        .map_err(|e| ResolveUrlError { detail: format!("src khong phan giai duoc ({src}): {e}") })?;

    if joined.scheme() != "http" && joined.scheme() != "https" {
        return Err(ResolveUrlError {
            detail: format!("scheme '{}' khong phai http/https ({src})", joined.scheme()),
        });
    }

    // 🔵 THÊM (vòng rà đối kháng 2, mục D5) — hai hình dạng ĐO ĐƯỢC (không suy diễn, xem
    // `cargo run --example probe_url` lúc vá) mà `Url::join` phân giải thành một tài nguyên
    // KHÔNG PHẢI ảnh, dù cả hai đều KHÔNG rỗng và KHÔNG chỉ mang `#fragment` (chặn ở trên
    // không bắt được):
    // 1. `src` CHỈ mang một query (`"?v=2"`) — `join` giữ NGUYÊN `path` của trang, chỉ thay
    //    query. `joined.path() == base.path()` ⇒ đây là CHÍNH trang, một query khác không
    //    biến nó thành một ảnh.
    // 2. `src` là một đoạn `.`/`./`/`..`/`../ ` (thuần điều hướng thư mục) — `join` luôn cho
    //    ra một `path` KẾT THÚC bằng `/` (một THƯ MỤC, không một tệp) cho các dạng này.
    // Không chặn cả hai, host của CHÍNH trang (hoặc thư mục chứa nó) lọt vào tầng 2 như một
    // "ảnh", và `fetch(..., ResourceKind::Image)` tải lại TOÀN BỘ trang/danh sách thư mục
    // dưới danh nghĩa ảnh.
    if joined.path() == base.path() {
        return Err(ResolveUrlError {
            detail: format!("src ({src}) phan giai ve CHINH trang (cung path, khac query o co) -- khong phai mot anh"),
        });
    }
    if joined.path().ends_with('/') {
        return Err(ResolveUrlError {
            detail: format!("src ({src}) phan giai ve mot THU MUC (path ket thuc bang '/'), khong phai mot tep anh"),
        });
    }

    Ok(joined.to_string())
}

/// Danh mục MIME ảnh raster ĐÓNG — bốn kiểu, đúng khuôn `Fetcher::looks_like_html` (so BẰNG,
/// không `contains`).
///
/// 🔴 **`image/svg+xml` KHÔNG có mặt, có chủ ý (§Never spec 6.11).** SVG là ĐÁNH DẤU (một tài
/// liệu XML có thể mang `<script>`), không phải ảnh raster — nhận nó vào đây mở lại đúng bề
/// mặt mà AD-16 (nội dung ngoài không bao giờ render thành HTML/markup) tồn tại để bịt: một
/// SVG được lưu làm "ảnh" rồi hiển thị (Story 6.14) sẽ là markup ngoài chạy trong webview.
const RASTER_IMAGE_MIMES: [&str; 4] = ["image/jpeg", "image/png", "image/gif", "image/webp"];

/// Cắt tại dấu `;` ĐẦU TIÊN (bỏ tham số `charset=...`), `trim`, hạ chữ thường — cùng khuôn
/// `Fetcher::looks_like_html`. Dùng CHUNG cho cả [`is_raster_image_mime`] lẫn
/// [`extension_for_mime`] để hai hàm không thể trôi khỏi nhau về cách đọc `content-type`.
///
/// 🔵 **THÊM `pub` 2026-09-08 (Story 6.11, D1 vòng rà đối kháng 3 lớp).** Bản trước
/// `commands::project::fetch_and_write_one_asset` tự DỰNG LẠI chuỗi MIME ghi vào cột
/// `asset.content_type` bằng `format!("image/{}", ...)` từ ĐUÔI TỆP — ba bảng literal MIME
/// (`RASTER_IMAGE_MIMES`, nhánh `match` của `extension_for_mime`, và bảng dựng-ngược đó) cho
/// CÙNG một danh mục bốn phần tử là ba nguồn có thể trôi khỏi nhau (ví dụ đuôi `"jpg"` dựng
/// ngược lại `"image/jpeg"` bằng một `match` RIÊNG thứ tư nếu ai đó thêm một kiểu MIME mới).
/// Hàm này (đã có sẵn, dùng nội bộ) mở `pub` để tầng gọi lấy THẲNG chuỗi MIME đã CHUẨN HOÁ từ
/// chính phản hồi, không dựng lại từ đuôi.
pub fn normalized_mime(content_type: Option<&str>) -> Option<String> {
    content_type.map(|ct| ct.split(';').next().unwrap_or("").trim().to_ascii_lowercase())
}

/// `content-type` có phải một MIME ảnh raster đã CHẤP NHẬN hay không. `None` (máy chủ không
/// khai) bị coi là KHÔNG PHẢI — cùng luật `looks_like_html`: im lặng đoán "chắc là ảnh" khi
/// máy chủ không nói gì là một phỏng đoán, không phải một sự thật đọc được.
///
/// The MIME gate for [`decode_data_uri_image`]: `data:` images never reach
/// [`extension_for_mime`] (there's no HTTP response to read a `content-type` from), so this
/// is the real decision point for that path.
pub fn is_raster_image_mime(content_type: Option<&str>) -> bool {
    normalized_mime(content_type).is_some_and(|ct| RASTER_IMAGE_MIMES.contains(&ct.as_str()))
}

/// Đuôi tệp cho một MIME ảnh raster — `None` nếu `content_type` không khớp danh mục ĐÓNG
/// [`RASTER_IMAGE_MIMES`].
///
/// 🔴 **Đây là điểm QUYẾT ĐỊNH DUY NHẤT trên đường ghi ảnh sản phẩm** (xem SỬA ở doc-comment
/// đầu tệp) — `commands::project::fetch_and_write_one_asset` đọc `None` từ hàm này làm TOÀN
/// BỘ tín hiệu "bỏ ảnh", không gọi [`is_raster_image_mime`] nữa. `_ => None` ở nhánh cuối vì
/// thế KHÔNG phải một trường hợp thừa/phòng thủ — nó CHÍNH LÀ vế "từ chối" của mệnh đề.
///
/// Đuôi đến từ MIME của PHẢN HỒI, không từ đuôi có sẵn trong URL — một URL `.jpg` trả
/// `text/html` là một ca có thật (trang lỗi/đăng nhập chặn hotlink), và đuôi ghi xuống đĩa
/// phải nói đúng NỘI DUNG THẬT của tệp, không phải một dấu hiệu trên URL.
pub fn extension_for_mime(content_type: Option<&str>) -> Option<&'static str> {
    match normalized_mime(content_type).as_deref() {
        Some("image/jpeg") => Some("jpg"),
        Some("image/png") => Some("png"),
        Some("image/gif") => Some("gif"),
        Some("image/webp") => Some("webp"),
        _ => None,
    }
}

/// A `data:` URI missing `;base64`, missing the comma separating header from payload, whose
/// MIME falls outside [`RASTER_IMAGE_MIMES`] (SVG is still refused, AD-16), or whose payload
/// isn't valid base64.
#[derive(Debug)]
pub struct DataUriError {
    pub detail: String,
}

/// Decodes a `data:image/...;base64,...` URI into raw bytes plus a normalized MIME. `data:`
/// images never touch the network or the `Allowlist`; they're gated by the same
/// [`is_raster_image_mime`] predicate `fetch_and_write_one_asset` uses, so SVG is refused
/// through the same gate (AD-16).
pub fn decode_data_uri_image(src: &str) -> Result<(Vec<u8>, String), DataUriError> {
    let rest = src
        .strip_prefix("data:")
        .ok_or_else(|| DataUriError { detail: "khong phai mot data: URI".to_owned() })?;
    let (header, payload) = rest
        .split_once(',')
        .ok_or_else(|| DataUriError { detail: "data: URI thieu dau phay phan cach header/payload".to_owned() })?;
    if !header.split(';').any(|part| part.trim().eq_ignore_ascii_case("base64")) {
        return Err(DataUriError { detail: "data: URI khong mang tham so base64".to_owned() });
    }
    let mime = header.split(';').next().unwrap_or("").trim().to_ascii_lowercase();
    if !is_raster_image_mime(Some(&mime)) {
        return Err(DataUriError { detail: format!("MIME '{mime}' khong thuoc danh muc anh raster (data: URI)") });
    }
    let cleaned: String = payload.chars().filter(|c| !c.is_ascii_whitespace()).collect();
    let bytes = {
        use base64::Engine as _;
        base64::engine::general_purpose::STANDARD
            .decode(cleaned)
            .map_err(|e| DataUriError { detail: format!("giai ma base64 that bai: {e}") })?
    };
    Ok((bytes, mime))
}

/// Bóc HOST của một URL TUYỆT ĐỐI đã phân giải (kết quả của [`resolve_absolute_url`]) — dùng
/// để dựng tập host tầng 2 (`Allowlist::with_tier2_hosts`) TRƯỚC khi tải, mà không phải gõ
/// `reqwest` ở tầng gọi (`commands::project`): `webimport_boundary.rs::reqwest_is_named_only_inside_core_webimport_or_core_ai`
/// cấm nguyên chữ `reqwest` xuất hiện ngoài `core/webimport/`/`core/ai/` — đây là chỗ DUY
/// NHẤT tầng gọi cần đọc một host từ một chuỗi URL, nên nó thuộc về MODULE NÀY, không phải
/// một lời gọi `reqwest::Url::parse` rải ở `commands/project/`. Cùng khuôn
/// `allowlist::host_of` (RIÊNG TƯ, module đó không cần lộ nó ra ngoài).
pub fn host_of(url: &str) -> Option<String> {
    reqwest::Url::parse(url).ok().and_then(|u| u.host_str().map(str::to_owned))
}

/// Dedup key for a pasted URL-import link: strips `#fragment`, lowercases the host. `None`
/// when `url` doesn't parse — an unparseable URL is treated as unique rather than merged
/// with another one that also failed to parse.
pub fn normalize_url_for_dedup(url: &str) -> Option<String> {
    let mut parsed = reqwest::Url::parse(url).ok()?;
    parsed.set_fragment(None);
    if let Some(host) = parsed.host_str() {
        let lower = host.to_ascii_lowercase();
        if lower != host {
            parsed.set_host(Some(&lower)).ok()?;
        }
    }
    Some(parsed.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolve_absolute_url_handles_absolute_relative_and_protocol_relative_src() {
        let page = "https://example.test/bai-viet/chuong-1.html";

        assert_eq!(
            resolve_absolute_url("https://cdn.example/x.jpg", page).unwrap(),
            "https://cdn.example/x.jpg",
            "src da tuyet doi thi giu nguyen"
        );
        assert_eq!(
            resolve_absolute_url("/a.jpg", page).unwrap(),
            "https://example.test/a.jpg",
            "src tuong doi tu GOC phai hop voi host cua trang"
        );
        assert_eq!(
            resolve_absolute_url("../b.png", page).unwrap(),
            "https://example.test/b.png",
            "src tuong doi TU THU MUC phai hop dung duong dan (cat het thu muc chua trang)"
        );
        assert_eq!(
            resolve_absolute_url("//cdn.example/x.jpg", page).unwrap(),
            "https://cdn.example/x.jpg",
            "src ROI GIAO THUC phai muon giao thuc cua trang (https)"
        );
    }

    #[test]
    fn resolve_absolute_url_rejects_an_unparsable_page_url_or_src() {
        assert!(resolve_absolute_url("/a.jpg", "khong-phai-url").is_err(), "URL trang hong phai bi tu choi");
        assert!(
            resolve_absolute_url("http://[::1", "https://example.test/").is_err(),
            "src hong (IPv6 khong dong ngoac) phai bi tu choi"
        );
    }

    /// D5 (vòng rà đối kháng 3 lớp) — `src` rỗng/khoảng trắng/chỉ `#fragment` KHÔNG được phép
    /// phân giải thành CHÍNH `page_url` (đúng ngữ nghĩa RFC 3986 mà `Url::join` theo) — nếu
    /// không, host của trang đang nhập lọt vào tầng 2 như một "ảnh".
    #[test]
    fn resolve_absolute_url_rejects_empty_whitespace_and_fragment_only_src() {
        let page = "https://example.test/bai-viet.html";
        for bad in ["", "   ", "#top", "  #muc-luc"] {
            let result = resolve_absolute_url(bad, page);
            assert!(result.is_err(), "src {bad:?} phai bi TU CHOI, khong duoc phan giai thanh page_url");
        }
    }

    /// D5 (vòng rà đối kháng 2, 3 lớp) — `src` chỉ mang QUERY, hoặc chỉ mang một đoạn điều
    /// hướng THƯ MỤC thuần (`.`/`./`/`..`/`../`), phải bị TỪ CHỐI — cả hai không rỗng, không
    /// chỉ `#fragment`, nên chặn D5 vòng trước KHÔNG bắt được; đo được bằng
    /// `cargo run --example probe_url` lúc vá: cả hai phân giải ra CHÍNH trang hoặc một thư
    /// mục, không phải một tệp ảnh.
    #[test]
    fn resolve_absolute_url_rejects_query_only_and_bare_directory_navigation_src() {
        let page = "https://example.test/bai-viet/chuong-1.html";
        for bad in ["?v=2", "?", ".", "./", "..", "../"] {
            let result = resolve_absolute_url(bad, page);
            assert!(
                result.is_err(),
                "src {bad:?} phai bi TU CHOI -- phan giai ve CHINH trang hoac mot thu muc, khong phai mot anh"
            );
        }
        // Ca ÂM đi kèm: một `src` tương đối THẬT SỰ trỏ tới một TỆP (có phần mở rộng, không
        // kết thúc bằng `/`) trong một thư mục CHA vẫn phải được CHẤP NHẬN — chặn trên không
        // được bắt oan một ảnh thật chỉ vì đường dẫn đi qua `..`.
        assert!(
            resolve_absolute_url("../anh/x.jpg", page).is_ok(),
            "mot src tuong doi THAT (co ten tep, khong ket thuc bang '/') qua thu muc cha van phai duoc chap nhan"
        );
    }

    /// D6 (vòng rà đối kháng 3 lớp) — chỉ `http`/`https` được phép; `data:`/`javascript:`
    /// không đi qua tầng 2 (không host để mà hỏi allowlist) và không phải một cuộc TẢI thật.
    #[test]
    fn resolve_absolute_url_rejects_every_scheme_except_http_and_https() {
        let page = "https://example.test/bai-viet.html";
        assert!(
            resolve_absolute_url("data:image/svg+xml;base64,PHN2Zz48L3N2Zz4=", page).is_err(),
            "data: phai bi TU CHOI -- khong phai mot cuoc tai qua mang"
        );
        assert!(resolve_absolute_url("javascript:alert(1)", page).is_err(), "javascript: phai bi TU CHOI");
        assert!(resolve_absolute_url("file:///etc/passwd", page).is_err(), "file: phai bi TU CHOI");
        assert!(resolve_absolute_url("https://cdn.example/x.jpg", page).is_ok(), "https: phai duoc chap nhan");
        assert!(
            resolve_absolute_url("http://cdn.example/x.jpg", page).is_ok(),
            "http: (khong TLS) van phai duoc chap nhan"
        );
    }

    #[test]
    fn is_raster_image_mime_accepts_exactly_the_closed_set_and_rejects_svg() {
        for good in ["image/jpeg", "image/png", "image/gif", "image/webp"] {
            assert!(is_raster_image_mime(Some(good)), "{good} phai duoc chap nhan");
            assert!(
                is_raster_image_mime(Some(&format!("{good}; charset=binary"))),
                "{good} kem tham so van phai duoc chap nhan (cat tai dau ';')"
            );
            let upper = good.to_ascii_uppercase();
            assert!(is_raster_image_mime(Some(&upper)), "{upper} (hoa) phai duoc chap nhan");
        }

        assert!(
            !is_raster_image_mime(Some("image/svg+xml")),
            "SVG la danh dau, khong phai anh raster -- phai bi TU CHOI (Never clause spec 6.11)"
        );
        assert!(!is_raster_image_mime(Some("text/html")), "mot trang HTML khong phai anh");
        assert!(!is_raster_image_mime(None), "may chu khong khai content-type phai bi coi la KHONG PHAI anh");
        assert!(
            !is_raster_image_mime(Some("image/jpegfoo")),
            "so BANG, khong `contains` -- mot MIME gan giong khong duoc khop nham"
        );
    }

    #[test]
    fn extension_for_mime_maps_the_closed_set_and_rejects_everything_else() {
        assert_eq!(extension_for_mime(Some("image/jpeg")), Some("jpg"));
        assert_eq!(extension_for_mime(Some("image/png; charset=binary")), Some("png"));
        assert_eq!(extension_for_mime(Some("image/gif")), Some("gif"));
        assert_eq!(extension_for_mime(Some("IMAGE/WEBP")), Some("webp"));
        assert_eq!(extension_for_mime(Some("image/svg+xml")), None);
        assert_eq!(extension_for_mime(None), None);
    }

    #[test]
    fn decode_data_uri_image_decodes_a_raster_mime_and_rejects_svg() {
        // A 1x1 PNG, base64-encoded.
        let png_b64 = "iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAQAAAC1HAwCAAAAC0lEQVR42mNk+A8AAQUBAScY42YAAAAASUVORK5CYII=";
        let (bytes, mime) =
            decode_data_uri_image(&format!("data:image/png;base64,{png_b64}")).expect("PNG data: URI phai giai ma duoc");
        assert_eq!(mime, "image/png");
        assert!(!bytes.is_empty());

        let svg = "data:image/svg+xml;base64,PHN2Zz48L3N2Zz4=";
        assert!(
            decode_data_uri_image(svg).is_err(),
            "SVG data: URI phai bi TU CHOI -- danh dau, khong phai anh raster (AD-16)"
        );

        let (bytes_with_space, mime_with_space) =
            decode_data_uri_image(&format!("data:image/png; base64,{png_b64}"))
                .expect("khoang trang sau dau ';' van phai duoc chap nhan");
        assert_eq!(mime_with_space, "image/png");
        assert!(!bytes_with_space.is_empty());
    }

    #[test]
    fn decode_data_uri_image_rejects_non_data_scheme_missing_base64_and_bad_payload() {
        assert!(decode_data_uri_image("https://cdn.example/x.png").is_err());
        assert!(decode_data_uri_image("data:image/png,not-base64-encoded").is_err(), "thieu tham so base64 phai bi tu choi");
        assert!(
            decode_data_uri_image("data:image/png;base64,!!!not-valid-base64!!!").is_err(),
            "payload khong phai base64 hop le phai bi tu choi"
        );
    }

    /// `RASTER_IMAGE_MIMES` (the `data:` path's decision point) and `extension_for_mime`'s
    /// `match` arms (the network path's decision point) must match exactly the same
    /// category, or the two paths silently drift apart with nothing to catch it.
    #[test]
    fn raster_image_mimes_matches_extension_for_mimes_branches_exactly() {
        for mime in RASTER_IMAGE_MIMES {
            assert!(
                extension_for_mime(Some(mime)).is_some(),
                "{mime} co trong RASTER_IMAGE_MIMES nhung extension_for_mime khong nhan dien"
            );
        }
        let known_extensions = ["jpg", "png", "gif", "webp"];
        assert_eq!(
            RASTER_IMAGE_MIMES.len(),
            known_extensions.len(),
            "hai danh sach phai cung so luong phan tu"
        );
        for ext in known_extensions {
            let mime = format!("image/{}", if ext == "jpg" { "jpeg" } else { ext });
            assert!(
                RASTER_IMAGE_MIMES.contains(&mime.as_str()),
                "{mime} duoc extension_for_mime nhan dien nhung khong co trong RASTER_IMAGE_MIMES"
            );
        }
    }

    #[test]
    fn normalize_url_for_dedup_strips_fragment_and_lowercases_host_only() {
        assert_eq!(
            normalize_url_for_dedup("https://Example.TEST/Bai-Viet#muc-luc"),
            normalize_url_for_dedup("https://example.test/Bai-Viet"),
            "fragment bi bo, host ha chu thuong -- hai URL nay phai cung mot khoa"
        );
        assert_ne!(
            normalize_url_for_dedup("https://example.test/Bai-Viet"),
            normalize_url_for_dedup("https://example.test/bai-viet"),
            "duong dan (path) VAN phan biet hoa/thuong -- khong bi ha chu cung host"
        );
        assert!(normalize_url_for_dedup("khong-phai-url").is_none());
    }
}
