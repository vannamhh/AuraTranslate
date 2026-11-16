# Kiểm chứng website giới thiệu AuraTranslate

Ngày kiểm tra: 2026-11-16. Phạm vi: `website/`, kế hoạch `plan-website-thong-tin-chinh-xac.md`; không dùng kết quả này để nghiệm thu ứng dụng desktop.

## Ảnh app thật

Tạo riêng Tác phẩm “Một buổi sáng bên sông — bản minh họa” từ nội dung Trung–Việt tự soạn: tám câu, năm câu xác nhận, ba thuật ngữ. Phiên app dùng hai thư mục tạm độc lập cho dữ liệu toàn cục và Library, cùng cổng WebDriver 4459. Trước khi tạo có kiểm tra kho toàn cục trong thư mục tạm; sau khi tạo có kiểm tra đường dẫn Tác phẩm trong Library tạm. Hàng rào của `e2e/wdio.conf.mjs` vẫn chạy.

Lượt chụp cuối: 1/1 ca qua, khoảng 9 giây cho bộ chạy; năm ảnh 2560 × 1720 chụp trực tiếp webview của bản debug có feature `wdio`. Đã xem từng ảnh. Nội dung mẫu và xuất xứ ảnh nằm trong `website/assets/screenshots/sample-content.json` và `README.md` cạnh ảnh.

Các lần chuẩn bị lộ hai vấn đề của script chụp: gọi lại fixture reset trong phiên đã có hàng và nhập nhầm bản module không có query HMR. Script tạm đã sửa cách phân giải module thực tế; không sửa app. Thử bổ sung ảnh tra cứu bị timeout và đã bỏ; bộ ảnh sử dụng là lượt chụp cuối đã qua, không tuyên bố trong ảnh có kết quả tra từ.

Các ảnh ban đầu chụp từ Tác phẩm đang test của Ice đã bị xóa khỏi website và các tệp ảnh tạm do lượt làm việc này tạo. Bộ ảnh giao nhận chỉ chứa Tác phẩm minh họa mới.

## Website

- `node --check website/app.js`, kiểm tra whitespace của diff website và liên kết fragment/tệp cục bộ: qua.
- Thực thi chính `website/app.js` trong happy-dom có sẵn: năm tình huống normal, saved-light, invalid, blocked-read, blocked-write qua. Mỗi tình huống kiểm tra đổi theme, hai ví dụ song ngữ, năm nút ảnh, trạng thái ARIA và FAQ/Escape/focus; hai tình huống chặn storage đều chứng minh có chạm nhánh ném lỗi.
- Counter-check: gỡ thật try/catch đọc storage trong tệp sản phẩm, tình huống blocked-read đỏ ở tương tác theme; khôi phục nguyên tệp, năm tình huống xanh.
- Orca browser trên trang HTTP cục bộ: width/scrollWidth lần lượt 375/375, 768/753 và 1440/1425 px. 375 px có ví dụ một cột; không phần tử nào vượt mép phải. Hiệu số 15 px trên desktop/tablet là thanh cuộn.
- Bàn phím thật trong browser: Enter chọn ảnh Glossary, ảnh tải và caption đổi; Enter đổi dark → light; Enter mở FAQ rồi Escape đóng và giữ focus đúng nút. Ảnh xuất tải khi cuộn tới phần xem trước. Đã xem ảnh chụp website dark desktop/mobile và light tablet.
- Browser từng giữ CSS/JS cũ sau HTML mới: đo ảnh rộng 2560 px và scrollWidth 2829 ở viewport 1440. Thêm version query cho hai asset; đo lại ảnh rộng khoảng 1131 px, không tràn trang.
- Review quick độc lập: không phát hiện lỗi; bản vá cache được kiểm tra lại bằng browser và các kiểm tra tập trung.

## Đích tải đã xác minh

Kho công khai `vannamhh/AuraTranslate`. GitHub API `/releases/latest` đang trả `dict-v4`, chỉ có các tệp từ điển. Vì vậy nút tải app dùng bộ cài và checksum đã đọc trực tiếp từ release [v1.1.0](https://github.com/vannamhh/AuraTranslate/releases/tag/v1.1.0): DMG universal, MSI x64 và `SHA256SUMS.txt`. Khi phát hành app mới cần cập nhật ba liên kết có phiên bản và nhãn liên quan.

## CI hiện có

- [Lượt push 43478764582](https://github.com/vannamhh/AuraTranslate/actions/runs/43478764582), 2026-11-13: macOS và Windows xanh; e2e được bỏ qua theo lịch chạy. Đây là revision `7e6b6b1e5f25a196288123ba889f1075b3514c23`, không phải kiểm chứng cho diff website chưa push.
- [Nightly 43769215036](https://github.com/vannamhh/AuraTranslate/actions/runs/43769215036), 2026-11-15 UTC: macOS check và macOS e2e xanh. Windows đỏ ở `quick_translation_contract.rs:1267`, ca `purging_all_rows_shrinks_db_plus_wal_on_disk`: `wal_checkpoint(TRUNCATE) was blocked by a reader`. Đây là lỗi đã có trước thay đổi website; không sửa hoặc kết luận nguyên nhân sâu hơn trong lượt này.
- Đã đọc workflow và kết quả từng job; không có job UTC riêng trong workflow hiện tại. Không chạy lại toàn bộ suite app cho thay đổi website tĩnh. Chưa push nên chưa có CI của commit website.
