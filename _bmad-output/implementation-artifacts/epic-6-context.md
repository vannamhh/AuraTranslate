# Epic 6 Context: Đường nhập — mọi nguồn văn bản vào được, và không hỏng im lặng

<!-- Compiled from planning artifacts. Edit freely. Regenerate with compile-epic-context if planning docs change. -->

## Goal

Bề mặt đầu tiên người dùng chạm vào sản phẩm, nơi hai lỗi đắt nhất — bảng mã sai, ranh giới bóc nội dung sai — có thể xảy ra mà không báo gì. Epic dựng một pipeline nhập duy nhất, thứ tự cố định, dùng chung mọi nguồn (file, URL, song ngữ), bắt buộc qua xem trước hợp nhất trước khi ghi đĩa. Ảnh web nằm trong `.atproj`; alt-text/caption là segment riêng. Ứng dụng không bao giờ tự quyết định tải gì ngoài danh sách người dùng cấp — kiểm chứng bằng mắt.

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

- Mọi nguồn qua đúng một pipeline, cùng thứ tự, chỉ khác bước đầu vào; không byte ghi đĩa trước khi xác nhận ở xem trước. Bảng mã sai/bóc sai không có trạng thái "lỗi" cứng — chỉ hiện bằng mắt, sửa tay được.
- URL: phạm vi/thứ tự tải do đúng danh sách người dùng dán quyết định, không tự quét mục lục hay lần "chương sau"; hai số N link/N Chương bằng nhau là bằng chứng quan sát được; chưa bấm thì không gọi mạng. Link hỏng: giữ chỗ đánh dấu tại vị trí, một trong 8 lý do phân biệt được; xác nhận khoá tới khi bỏ mục đó hoặc tải lại riêng.
- Bộ lọc "cần xem" áp cho mọi đường nhập — điều kiện để dò bảng mã/bóc nội dung/luật làm sạch còn tác dụng ở quy mô hàng chục Chương.
- Ảnh web là file thật trong `.atproj/assets/`, qua asset protocol, không bao giờ URL từ xa. Xuất xứ tài liệu (tác giả, nguồn, URL, ngày đăng) ở tầng Chương, tự điền khi có thể, không để trống im lặng.
- Song ngữ: lệch số câu hai cột không khớp im lặng — đẩy ra nối tay trước khi ghi; segment vào trạng thái chưa xác nhận dù trông hoàn chỉnh. Alt-text/caption là segment dịch được để tự động vào TM/Glossary.
- Mạng chỉ chủ động do người dùng kích hoạt; không tải nền/prefetch/tải lại ảnh đã có; danh sách domain đã gọi phải xem được. Ngoài phạm vi v1: bộ đọc riêng theo site.
- Ba ngưỡng hiệu năng Library đo lại bằng số thật trên thư viện 5.000 Chương dựng bằng chính đường nhập hàng loạt.

## Technical Decisions

- Pipeline `core/segment/`: giải mã bảng mã → bóc nội dung → làm sạch theo luật → chuẩn hoá đoạn/khoảng trắng → tách Chương theo mẫu → xem trước+sửa tay → tách segment+cờ kết đoạn → ghi `.atproj`. Tách Chương theo hình dạng đầu vào (một dòng chưa chia → tách; một link một Chương → không). `.docx` bỏ qua giải mã bảng mã.
- **`Fetcher`/`Extractor` tách rời, không nâng cổng thứ tư (AD-40):** hai module Rust trong `core/webimport/`, không trait hoá. `Fetcher` (URL → byte+content-type+charset khai báo) đúng một cài đặt mãi mãi — điểm ra mạng thứ ba của kiến trúc. `Extractor` (byte → mô hình đoạn/câu/ảnh/caption có cấu trúc, không nhánh nào mang chuỗi HTML) một cài đặt dùng chung. `Fetcher` không bao giờ phân tích nội dung; `Extractor` không bao giờ chạm mạng — ranh giới này là điều kiện allowlist mạng kiểm chứng được bằng test. Không byte HTML thô qua IPC.
- **Allowlist mạng hai tầng, sống đúng một lần nhập (AD-41):** tầng 1 = host link vừa dán, tải tài liệu; tầng 2 = host tài nguyên trang tầng 1 tham chiếu (cùng lần nhập), chỉ tải ảnh, không bao giờ tài liệu. Host ngoài hai tầng, hoặc chuyển hướng dẫn ra ngoài, bị từ chối. Ghi (thời điểm, domain, tầng, kết quả) vào nhật ký xem được. Không tải lại ảnh đã có (so `source_url`, cùng Tác phẩm). Tauri capabilities khai tĩnh lúc build, không diễn đạt được ràng buộc runtime này — hàng rào ở mã ứng dụng, **bắt buộc bộ test riêng**: từ chối host ngoài hai tầng, từ chối chuyển hướng ra ngoài, từ chối tài liệu ở tầng 2, không lời gọi khi chưa bấm.
- Cổng ghi mẫu đã triển khai thật ở Story 6.7, tham chiếu cho cổng ghi của 6.8: `commands::project::chapters_shape_if_all_ok` trả `None` khi còn mục lỗi, đồng bộ `PendingImportSourceState`. `.docx`: bảng một hàng ô nhiều đoạn (bản đăng bài) bị từ chối ở cổng vào trước mọi ghi, nhận theo hình dạng dữ liệu, không theo tên file.
- Alt-text/caption là `Segment` mang thêm trường vai, không phải cột text trên ảnh (cột text không vào TM, hỏng im lặng). `ASSET` mang neo vị trí độc lập với segment đi kèm; `alt` treo tại neo, `caption` ngay sau neo. Luật làm sạch hai tầng là hợp nhất (Toàn cục + Tác phẩm cùng áp), khác ghi đè của Glossary/prompt.

## UX & Interaction Patterns

- Xem trước xếp ba tầng nhân quả, không nút "Tiếp theo": bảng mã → ranh giới bóc → luật làm sạch; đổi bảng mã dựng lại ngay trong bộ nhớ. Tin cậy dò bảng mã thấp mở dải năm ứng viên, cỡ chữ đọc, kèm bản dựng thật 6–8 ký tự đầu Chương.
- Dưới ô dán link hiện *N link · sẽ tạo N Chương* — lệch nhau là dấu hiệu lỗi. Đầu xem trước hiện *N Chương cần xem*/*M Chương sạch*; `⌥W` lọc cần xem, `⌥←`/`⌥→` đi Chương trước/sau, `⌘↵` xác nhận toàn bộ.
- Khối nội dung: trạng thái vạch lề `confirmed`/`tm-rule`/`ornament` (đã loại, phân biệt bằng độ lùi không phải màu). `J`/`K` điều hướng, `Space` giữ/bỏ, `[`/`]` đặt vùng giữ, `R` bật/tắt luật khớp — mọi phím là command chung. Chỗ bị luật khớp hiện gạch ngang kèm nhãn, hai số (Chương này/cả lần nhập) và nhãn tầng.
- Nhật ký domain: dòng tóm tắt chân xem trước + bảng đầy đủ ở Cài đặt › Quyền riêng tư; hai tầng phân biệt bằng nhãn chữ (`Tài liệu`/`Ảnh`), không màu; mỗi hàng ghi vì sao được phép.
- Ảnh hiển thị đúng vị trí lấy từ neo, không suy từ thứ tự segment; caption dịch hiện dưới ảnh, alt-text dịch không hiện trên trang.

## Cross-Story Dependencies

- Story 6.1 đi trước 6.3, 6.9, và HTTP client của 6.7/6.8. Story 6.2 là nền cho mọi story nguồn còn lại.
- Story 6.7 là điều kiện cho 6.8 (allowlist) và 6.11 (tải ảnh); 6.8 lại là điều kiện để 6.11 tải ảnh qua tầng 2 hợp lệ.
- Story 6.9, 6.6, 6.3, 6.5 nuôi trực tiếp vào 6.10; 6.11 là nền cho 6.13, 6.14; 6.12 cấp năng lực đọc số đoạn trong ô bảng — điều kiện cho một cổng kiểm ở Epic 8.
- Story 6.16 phụ thuộc mẫu phân tách của 6.6, là điều kiện cho 6.17; 6.18 phụ thuộc đường nhập hàng loạt 6.6/6.7, thay số đo sơ bộ ở Epic 5; 6.13 là điều kiện để alt-text/caption tự động vào TM ở Epic 7 (nghiệm thu ở Story 7.1) và Glossary.
