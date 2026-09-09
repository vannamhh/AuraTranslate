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
//!
//! 🔵 **SỬA (vòng rà đối kháng, đo được 2026-09-09) — ② KHÔNG còn là một BƯỚC riêng trên
//! đường sản phẩm.** `commands::project::fetch_and_write_one_asset` (chỗ gọi sản phẩm DUY
//! NHẤT của cả hai hàm) TỪNG gọi `is_raster_image_mime` làm bước gác đứng TRƯỚC
//! `extension_for_mime` — hai vị từ cùng canh đúng MỘT mệnh đề ("MIME này được chấp nhận
//! không") trên đúng một danh mục bốn phần tử. Đo bằng phép GỠ THẬT: gỡ hẳn bước gác đó rồi
//! chạy TRỌN `cargo test --locked` (46 binary) — **0 ca đỏ**, vì `extension_for_mime` đã TỰ
//! đóng vai gác cổng qua nhánh `_ => None` của chính nó. ⇒ Bước gác rời đã bị GỠ khỏi
//! `fetch_and_write_one_asset`; `extension_for_mime` MỘT MÌNH là điểm quyết định DUY NHẤT
//! trên đường ghi ảnh. `is_raster_image_mime` **VẪN ở lại, xuất khẩu công khai** — một vị từ
//! ĐỘC LẬP, tự kiểm bằng test riêng — nhưng KHÔNG còn là một bước của luồng tải ảnh hôm nay,
//! và **0 chỗ gọi sản phẩm nào dùng nó**.
//!
//! ⚠️ **SỬA 2026-09-09 (vòng rà đối kháng 2, mục D9) — câu trước nhường cho `deferred-work.md`.**
//! Bản trước ở đây khai lý do giữ lại là "cho chỗ gọi TƯƠNG LAI, ví dụ Story 6.14" — một khẳng
//! định CHƯA KIỂM ĐƯỢC: Story 6.14 (hiển thị ảnh) đọc byte ảnh TỪ ĐĨA (`assets/<file_name>`),
//! không đọc `content-type` của một phản hồi mạng — tức không có gì bảo đảm 6.14 sẽ THẬT SỰ
//! gọi một hàm nhận đầu vào là chuỗi MIME kiểu HTTP. `deferred-work.md` (mục "Story 6.11")
//! đã ghi đúng hai phương án còn treo (gỡ hẳn, hoặc giữ như một mệnh đề công khai chờ chỗ gọi
//! thật) và để Ice chọn — câu ở đây không lặp lại một lý do chưa kiểm được nữa, chỉ trỏ sang
//! đó.
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
/// ⚠️ **KHÔNG còn nằm trên đường ghi ảnh sản phẩm** (xem SỬA ở doc-comment đầu tệp) —
/// [`extension_for_mime`] một mình đã là điểm quyết định. Hàm này ở lại như một vị từ CÔNG
/// KHAI, ĐỘC LẬP cho chỗ gọi tương lai; tự kiểm bằng bộ test của chính nó, không qua một ca
/// đầu-cuối nào của `create_work`.
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

/// Bóc HOST của một URL TUYỆT ĐỐI đã phân giải (kết quả của [`resolve_absolute_url`]) — dùng
/// để dựng tập host tầng 2 (`Allowlist::with_tier2_hosts`) TRƯỚC khi tải, mà không phải gõ
/// `reqwest` ở tầng gọi (`commands::project`): `webimport_boundary.rs::reqwest_is_named_only_inside_core_webimport_or_core_ai`
/// cấm nguyên chữ `reqwest` xuất hiện ngoài `core/webimport/`/`core/ai/` — đây là chỗ DUY
/// NHẤT tầng gọi cần đọc một host từ một chuỗi URL, nên nó thuộc về MODULE NÀY, không phải
/// một lời gọi `reqwest::Url::parse` rải ở `commands/project.rs`. Cùng khuôn
/// `allowlist::host_of` (RIÊNG TƯ, module đó không cần lộ nó ra ngoài).
pub fn host_of(url: &str) -> Option<String> {
    reqwest::Url::parse(url).ok().and_then(|u| u.host_str().map(str::to_owned))
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

    /// D1 (vòng rà đối kháng 3 lớp) — `RASTER_IMAGE_MIMES` (dùng bởi [`is_raster_image_mime`],
    /// không còn trên đường sản phẩm nhưng vẫn là một nguồn sự thật ĐỘC LẬP) và nhánh `match`
    /// của [`extension_for_mime`] (điểm quyết định THẬT trên đường sản phẩm) phải khớp CHÍNH
    /// XÁC cùng một danh mục — nếu không, "vị từ độc lập" và "điểm quyết định thật" âm thầm
    /// trôi khỏi nhau và không cổng nào bắt được.
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
}
