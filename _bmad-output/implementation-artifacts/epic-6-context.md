# Epic 6 Context: Đường nhập — mọi nguồn văn bản vào được, và không hỏng im lặng

<!-- Compiled from planning artifacts. Edit freely. Regenerate with compile-epic-context if planning docs change. -->

## Goal

Bề mặt đầu tiên người dùng chạm vào sản phẩm — nơi bảng mã sai và ranh giới bóc nội dung sai có thể hỏng dữ liệu mà không báo gì. Epic dựng một pipeline nhập duy nhất, thứ tự cố định, dùng chung cho file, URL và tài liệu song ngữ, bắt buộc qua một màn xem trước hợp nhất trước khi ghi đĩa. Ảnh web thành file thật trong `.atproj`, alt-text/caption là segment dịch được riêng, và ứng dụng không bao giờ tự ý tải gì ngoài danh sách người dùng cấp.

## Stories

- Story 6.1: Mũi thăm dò ba lựa chọn thư viện
- Story 6.2: Pipeline nhập một chuỗi thứ tự cố định, dùng chung mọi nguồn
- Story 6.3: Bảng mã — phát hiện và dải đối chiếu năm bản dựng thật
- Story 6.4: Chuẩn hoá xuống dòng và khoảng trắng
- Story 6.5: Luật làm sạch lộ ra và hiện thứ sắp xoá
- Story 6.6: Tách Chương theo mẫu phân tách
- Story 6.7: Nhập từ URL bằng danh sách link
- Story 6.8: Allowlist mạng hai tầng và nhật ký domain
- Story 6.9: Bóc nội dung chính và sửa ranh giới bằng bàn phím
- Story 6.10a: Xem trước theo từng Chương và điều hướng Chương
- Story 6.10: Bộ lọc "cần xem"
- Story 6.11: Ảnh tải về `.atproj`, neo vị trí, và URL gốc
- Story 6.12: Đọc `.docx`
- Story 6.13: Alt-text và caption là hai `Segment` mang trường vai
- Story 6.14: Hiển thị ảnh đúng vị trí
- Story 6.15: Xuất xứ tài liệu ở tầng Chương
- Story 6.16: Nhập tài liệu song ngữ hai cột
- Story 6.17: Khớp câu trong từng cặp hàng
- Story 6.18: Đo lại NFR3, NFR4, NFR5 trên thư viện 5.000 Chương thật

## Requirements & Constraints

- Mọi nguồn qua đúng một pipeline, cùng thứ tự, chỉ khác bước đầu vào; không byte nào ghi đĩa trước khi xác nhận ở xem trước. Bảng mã/bóc sai không phải "lỗi" — chỉ hiện bằng mắt, sửa tay được.
- URL: phạm vi/thứ tự tải do đúng danh sách link người dùng dán quyết định, không tự quét mục lục hay lần "chương sau"; số link và số Chương phải bằng nhau; chưa bấm thì không gọi mạng. Link hỏng giữ chỗ đúng vị trí kèm một trong tám lý do; xác nhận khoá tới khi xử lý xong mục đó.
- Xem trước nhiều Chương: mỗi Chương mang dữ liệu của chính nó ở mọi tầng, không mượn số Chương khác — điều kiện để bộ lọc "cần xem" còn tác dụng ở quy mô hàng chục Chương. "Cần xem" gom năm nguyên nhân, so theo một hàng rào thống kê chung; dấu hiệu chưa đo được phải phân biệt với "đã đo và sạch".
- Ảnh web là file thật trong `assets/`, không bao giờ tham chiếu URL từ xa; xuất xứ tài liệu (tác giả, nguồn, URL, ngày đăng) ghi ở tầng Chương, tự điền khi có thể, không để trống im lặng.
- Song ngữ: lệch số câu hai cột không khớp im lặng — đẩy ra nối tay trước khi ghi; segment nhập theo đường này luôn chưa xác nhận dù trông hoàn chỉnh.
- Mạng chỉ chủ động do người dùng kích hoạt, không tải nền/prefetch, không tải lại ảnh đã có; nhật ký domain đã gọi phải xem được.
- Ba ngưỡng hiệu năng Library phải đo lại bằng số thật trên thư viện 5.000 Chương dựng bằng chính đường nhập hàng loạt, thay số đo sơ bộ trước.

## Technical Decisions

- Pipeline ở `core/segment/`: giải mã bảng mã → bóc nội dung → làm sạch → chuẩn hoá đoạn/khoảng trắng → tách Chương theo mẫu → xem trước+sửa tay → tách segment → ghi `.atproj`. Tách Chương áp theo hình dạng đầu vào, không theo loại nguồn. `.docx` bỏ qua giải mã bảng mã (đã khai encoding trong XML).
- `Fetcher`/`Extractor` là hai module Rust tách rời, không nâng thành cổng thứ tư: `Fetcher` chỉ lấy byte, không phân tích nội dung; `Extractor` dựng mô hình đoạn/câu/ảnh/caption có cấu trúc, không chạm mạng — ranh giới này là điều kiện để allowlist kiểm chứng được bằng test. Không byte HTML thô nào qua IPC.
- Allowlist mạng hai tầng sống đúng một lần nhập: tầng 1 là host link vừa dán (tải tài liệu), tầng 2 là host tài nguyên trang tầng 1 tham chiếu (chỉ tải ảnh). Host ngoài hai tầng hoặc chuyển hướng ra ngoài bị từ chối. Capabilities khai tĩnh lúc build không diễn đạt được ràng buộc runtime này, nên hàng rào nằm ở mã ứng dụng, bắt buộc có bộ test riêng.
- Đọc `.docx` dùng bộ đọc OOXML tự viết trên `zip`+`quick-xml`, không gọi thư viện ghi `.docx` sẵn có trên đường đọc: thư viện đó có hàng trăm điểm có thể làm tiến trình dừng đột ngột khi build ở chế độ mặc định của ứng dụng, khiến ca "tệp hỏng" không đóng được bằng cơ chế bắt lỗi thông thường. Thư viện đó vẫn giữ vai bộ ghi cho Epic 8 và làm bộ sinh dữ liệu test; bộ đọc mới là module riêng, không thừa kế quyền ra mạng của đường URL. Ô bảng `.docx` tách thành đoạn riêng để bộ tách câu không cắt vắt qua ranh giới hàng/ô.
- Alt-text/caption là `Segment` mang thêm trường vai, không phải cột text riêng trên ảnh — để tự động vào Translation Memory/Glossary sau này. `ASSET` mang neo vị trí độc lập với segment đi kèm.
- Luật làm sạch là danh sách mẫu ở hai tầng (Toàn cục, Tác phẩm), áp hợp nhất chứ không ghi đè.

## UX & Interaction Patterns

- Xem trước xếp ba tầng nhân quả — bảng mã, ranh giới bóc, luật làm sạch — không nút "Tiếp theo"; đổi bảng mã dựng lại ngay trong bộ nhớ. Tin cậy dò thấp mở dải năm ứng viên bảng mã ở cỡ chữ đọc, kèm đoạn dựng thật đầu Chương.
- Dưới ô dán link hiện cặp số N link/N Chương; đầu xem trước nhiều Chương hiện cặp số N cần xem/M sạch. Phím tắt: lọc về nhóm cần xem, đổi Chương đang xem (dừng ở hai đầu danh sách), xác nhận toàn bộ — đều là command đăng ký chung.
- Khối nội dung mang một trong ba trạng thái ở vạch lề: đã xác nhận, máy đoán chưa xác nhận, đã loại (chìm xuống, chữ nhỏ lại, phân biệt bằng độ lùi chứ không màu). Bàn phím: đi giữa khối, giữ/bỏ khối, đặt vùng giữ, bật/tắt luật khớp khối đang chọn.
- Nhật ký domain: dòng tóm tắt ở chân xem trước, bảng đầy đủ trong Cài đặt; hai tầng phân biệt bằng nhãn chữ, không màu.
- Ảnh hiển thị đúng vị trí theo neo riêng, không suy từ thứ tự segment; caption dịch hiện dưới ảnh, alt-text dịch không hiện trên trang.

## Cross-Story Dependencies

- Story 6.1 đi trước 6.3, 6.9 và phần HTTP của 6.7/6.8. Story 6.2 là nền cho mọi story nguồn khác.
- Story 6.7 là điều kiện cho 6.8 (allowlist) và 6.11 (tải ảnh); 6.8 lại là điều kiện để 6.11 tải ảnh qua tầng 2 hợp lệ.
- Story 6.9, 6.6, 6.3, 6.5 nuôi dữ liệu vào 6.10a, và 6.10a là nền bắt buộc cho 6.10. 6.11 là nền cho 6.13, 6.14; 6.12 cấp năng lực đếm đoạn trong ô bảng — điều kiện cho một cổng kiểm ở Epic 8.
- Story 6.16 phụ thuộc mẫu phân tách của 6.6, là điều kiện cho 6.17; 6.18 phụ thuộc đường nhập hàng loạt 6.6/6.7; 6.13 là điều kiện để alt-text/caption tự động vào TM ở Epic 7 và Glossary.
