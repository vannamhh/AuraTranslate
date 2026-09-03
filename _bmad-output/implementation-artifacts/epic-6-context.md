# Epic 6 Context: Đường nhập — mọi nguồn văn bản vào được, và không hỏng im lặng

<!-- Compiled from planning artifacts. Edit freely. Regenerate with compile-epic-context if planning docs change. -->

## Goal

Đây là bề mặt đầu tiên người dùng chạm vào sản phẩm, nơi hai lỗi đắt nhất có thể xảy ra mà không báo gì: bảng mã sai và ranh giới nội dung bóc sai. Epic này dựng một pipeline nhập duy nhất, thứ tự cố định, dùng chung cho mọi nguồn — file (`.txt`/`.md`/`.docx`), danh sách URL, tài liệu song ngữ hai cột — và bắt buộc mọi nguồn đi qua một màn xem trước hợp nhất trước khi bất kỳ byte nào ghi xuống đĩa. Màn xem trước lộ ra bảng mã đã đoán, ranh giới đã bóc, và những gì luật làm sạch sắp xoá. Ảnh tải từ web nằm trong `.atproj`; alt-text và caption là hai segment dịch được riêng biệt. Ứng dụng không bao giờ tự quyết định tải cái gì ngoài danh sách người dùng đã cấp.

## Stories

- Story 6.1: Mũi thăm dò ba lựa chọn thư viện (trích xuất nội dung, dò bảng mã, HTTP client)
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

- Nhập hàng loạt: nhiều file cùng lúc, hoặc tách một file lớn theo mẫu phân tách (tiêu đề/regex) cấu hình được, luôn kèm xem trước trước khi xác nhận.
- Nhập URL bằng danh sách link (mỗi dòng một Chương, đúng thứ tự). Ứng dụng tuyệt đối không tự tìm thêm link — không quét mục lục, không lần "chương sau"; ràng buộc phải đếm được bằng mắt, không chỉ là cam kết bằng lời.
- Bóc nội dung dùng một thuật toán chung (không bộ đọc riêng theo site), kèm xem trước bắt buộc và sửa ranh giới bằng tay — thiếu đường sửa tay thì chưa đạt nghiệm thu.
- Luật làm sạch rác là danh sách mẫu (chuỗi/regex) xem/sửa/tắt được; xem trước phải hiện thứ sắp xoá trước khi xoá — loại làm sạch duy nhất có thể xoá nhầm nội dung thật.
- Chuẩn hoá xuống dòng/khoảng trắng chạy trước tách segment; kết quả chuẩn hoá là thứ được lưu, không phải lớp hiển thị đắp lên bản gốc.
- Phát hiện/sửa bảng mã áp cho nguồn không tự khai bảng mã (`.txt`, `.md`, HTTP) trong năm bảng UTF-8/GB18030/GBK/Big5/UTF-16; đổi tay được, thấy kết quả ngay, không phải nhập lại từ đầu.
- Bộ lọc "cần xem" áp cho mọi đường nhập qua xem trước (không riêng URL) — điều kiện để bảng mã/bóc nội dung/làm sạch còn tác dụng thật ở quy mô hàng chục Chương.
- Ảnh tải từ web lưu trong `.atproj/assets/` (không có loại ảnh chỉ mang link); URL gốc lưu kèm metadata, không tham chiếu URL từ xa.
- Xuất xứ tài liệu (tác giả, báo/website, URL gốc, ngày đăng) là bốn trường ở tầng Chương, tự điền khi nhập URL, sửa/nhập tay được; không lưu chuỗi ghi nguồn đã định dạng.
- Nhập song ngữ (`.docx`/`.md`/`.csv`/`.tsv` hai cột) tạo Tác phẩm hoàn chỉnh, segment nguồn-đích đã khớp cặp; khớp câu giới hạn trong từng cặp hàng, số câu lệch nhau phải hiện ra cho người dùng nối tay, không khớp im lặng.
- Alt-text và caption là hai segment dịch được riêng biệt, tham gia TM/Glossary/luồng xác nhận như mọi segment khác.
- Mạng của epic này chỉ kích hoạt chủ động bởi người dùng; không telemetry. Ngoài phạm vi v1: bộ đọc riêng theo site, tự lần trang tiếp theo, tự quét mục lục — bị loại để giữ nguyên tắc "phạm vi và thứ tự do người dùng cấp". Rủi ro chấp nhận có ý thức: không kiểm điều khoản sử dụng website trước khi mở đường nhập URL.
- Ba ngưỡng hiệu năng Library (p95 tìm kiếm, khởi động, bộ nhớ nhàn rỗi) đang tạm; epic này đo lại bằng số thật trên thư viện 5.000 Chương dựng bằng chính đường nhập của sản phẩm, ghi kết luận vào PRD.

## Technical Decisions

- Một pipeline duy nhất, thứ tự cố định, sống ở `core/segment/`: giải mã bảng mã → bóc nội dung → làm sạch theo luật → chuẩn hoá đoạn/khoảng trắng → tách Chương theo mẫu → xem trước + sửa tay → tách segment + cờ kết đoạn → ghi `.atproj`. Mỗi nguồn chỉ khác ở bước đầu vào và việc bỏ qua bước không áp dụng. Áp bước tách Chương theo hình dạng đầu vào (một dòng chưa chia Chương → tách; mỗi link đã là một Chương → không tách), không theo loại đường nhập. `.docx` bỏ qua giải mã bảng mã; đường song ngữ áp mẫu phân tách lên cột nguồn, không áp cột đích.
- `Fetcher` (URL → byte + content-type + charset) và `Extractor` (byte → mô hình nội dung có cấu trúc) là hai module Rust tách rời, không trait hoá. `Fetcher` không phân tích nội dung; `Extractor` không chạm mạng. Bóc HTML chạy trọn ở Rust — không byte HTML thô qua IPC, chỉ mô hình đã bóc.
- Allowlist mạng hai tầng, sống đúng một lần nhập: tầng 1 (host các link đã dán) tải tài liệu; tầng 2 (host tài nguyên tham chiếu từ trang tầng 1) chỉ tải ảnh, không bao giờ tài liệu, kể cả khi chuyển hướng. Ghi `(thời điểm, domain, tầng, kết quả)` vào nhật ký. Tauri capabilities khai tĩnh lúc build nên không cưỡng chế được ràng buộc này bằng framework — bắt buộc bộ test riêng. Không tải lại ảnh đã có, so theo `source_url` trong cùng Tác phẩm.
- Nhập `.docx` phân biệt hai hình dạng bảng trước khi alignment: bảng một hàng với ô chứa nhiều đoạn (bản xuất một khối để đăng bài) bị từ chối tường minh, không ghi gì — nhận dạng bằng hình dạng, không bằng metadata/tên file.
- Alt-text/caption là `Segment` mang thêm trường vai (`alt`|`caption`), không phải cột text trên `ASSET`. `ASSET` mang neo vị trí riêng, độc lập với việc có segment đi kèm; `alt` treo tại neo ảnh, `caption` treo ngay sau. Nhiều nhất một segment mỗi vai; không sinh segment rỗng.
- Xuất xứ là dữ liệu bốn trường trên `CHAPTER` (tầng Chương) — nguồn sự thật duy nhất; khối ghi nguồn dựng lúc xuất, không lưu chuỗi định dạng sẵn.
- Ba thư viện nền cần rà trước khi ghim vào Stack: trích xuất nội dung, dò bảng mã, HTTP client cho `Fetcher` (ưu tiên dùng lại `reqwest` sẵn có); mọi thư viện mới phải rà GPL v3 trước khi thêm. Tỉ lệ bóc/dò sai cao không chặn tiến độ, không đổi kiến trúc — sửa tay ở xem trước là dự phòng theo thiết kế.

## UX & Interaction Patterns

- Xem trước xếp ba tầng theo thứ tự nhân quả trên một màn hình, không nút "Tiếp theo": bảng mã → ranh giới nội dung → luật làm sạch. Đổi bảng mã dựng lại tầng sau ngay trong bộ nhớ.
- Độ tin cậy dò bảng mã thấp → mở dải năm ứng viên trên văn bản, mỗi ứng viên kèm bản dựng thật 6-8 ký tự đầu Chương, cỡ chữ đọc — biến phán đoán kỹ thuật thành câu hỏi thị giác, thay vì hộp thoại cảnh báo.
- Dưới ô dán link luôn hiện `N` link · sẽ tạo `N` Chương, kèm "Chỉ tải đúng N link này. Không tìm thêm link nào khác."
- Đầu xem trước luôn hiện `N` Chương cần xem · `M` Chương sạch. Phím `⌥W` lọc nhóm cần xem; `⌥←`/`⌥→` đi Chương trước/sau; `⌘↵` xác nhận toàn bộ.
- Khối nội dung mang trạng thái vạch lề `confirmed`/`tm-rule`/`ornament` (đã loại, phân biệt bằng độ lùi chứ không màu). Phím `J`/`K`/mũi tên di chuyển, `Space` bật/tắt giữ, `[`/`]` đặt vùng giữ, `R` bật/tắt luật khớp khối chọn — mọi phím đăng ký trong `CommandRegistry`.
- Chỗ bị luật khớp hiện gạch ngang nét `ornament` kèm nhãn luật và hai con số (trong Chương này / cả lần nhập); mỗi luật mang nhãn tầng Toàn cục hoặc Tác phẩm.
- Bốn trường xuất xứ gom một khối đầu mỗi Chương, sửa tại chỗ; trường không tìm thấy hiện chữ nghiêng "không tìm thấy" thay vì để trống.
- Nhật ký domain: dòng tóm tắt chân xem trước cộng bảng đầy đủ ở Cài đặt › Quyền riêng tư; hai tầng phân biệt bằng nhãn chữ (`Tài liệu`/`Ảnh`), không bằng màu.
- Chế độ đọc: ảnh đúng vị trí theo neo `ASSET`; caption đã dịch hiện dưới ảnh, alt-text không hiện trên trang; ảnh không caption không chừa chỗ trống.

## Cross-Story Dependencies

- Story 6.1 (thư viện nền) đi trước 6.3, 6.9 và phần HTTP client của 6.7/6.8.
- Story 6.2 (pipeline dùng chung) là nền cho mọi story nguồn còn lại.
- Story 6.7 (nhập URL) là điều kiện cho 6.8 (allowlist) và 6.11 (tải ảnh); 6.8 lại là điều kiện để 6.11 tải qua tầng 2 hợp lệ.
- Story 6.9 và 6.6 nuôi trực tiếp vào 6.10 (bộ lọc "cần xem"), cùng 6.3 và 6.5.
- Story 6.11 (ảnh + neo) là nền cho 6.13 (alt-text/caption) và 6.14 (hiển thị ảnh).
- Story 6.12 (đọc `.docx`) cấp khả năng đọc số đoạn trong ô bảng — tiên quyết cho một cổng kiểm ở Epic 8, ngoài phạm vi epic này.
- Story 6.16 phụ thuộc mẫu phân tách của 6.6 và là điều kiện cho 6.17.
- Story 6.18 phụ thuộc đường nhập hàng loạt (6.6/6.7) để dựng thư viện 5.000 Chương; kết quả thay số đo sơ bộ ở Epic 5 (Story 5.14).
- Story 6.13 là điều kiện để caption/alt-text tự động vào TM khi TM được dựng ở Epic 7 (nghiệm thu thật ở Story 7.1) và vào Glossary.
