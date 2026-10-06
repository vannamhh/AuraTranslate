# Epic 3 Context: Glossary — chốt thuật ngữ một lần, dùng mãi

<!-- Compiled from planning artifacts. Edit freely. Regenerate with compile-epic-context if planning docs change. -->

## Goal

Epic này tạo Glossary để người dịch lưu và tái dùng một quyết định biên tập cho tên riêng, địa danh và thuật ngữ chuyên ngành. Nó phải hạ gánh nặng khởi đầu của một Tác phẩm dài bằng cách tìm ứng viên lúc nhập, nhưng vẫn giữ người dùng là người duy nhất quyết định mục nào được vào Glossary. Kết quả là một nguồn thuật ngữ đáng tin cho Workspace hôm nay, cho việc ép cách dùng nhất quán ở các năng lực sau này, và cho việc chia sẻ bộ thuật ngữ với người dịch khác qua file — không cần server hay tài khoản.

## Stories

- Story 3.1: Mô hình Glossary hai tầng và vòng đời ba trạng thái
- Story 3.2: Bảng chờ ứng viên tách hẳn khỏi Glossary
- Story 3.3: Thêm nhanh thuật ngữ từ bất kỳ panel nào
- Story 3.4: Khớp thuật ngữ theo ngôn ngữ qua Matcher dùng chung
- Story 3.4b: Đánh dấu thuật ngữ ở cột nguyên văn của lưới
- Story 3.5: Quét ứng viên khi nhập tài liệu
- Story 3.6: Trạng thái chờ chốt và dải mọc chốt lần đầu gặp
- Story 3.7: Đề xuất bản dịch bằng âm Hán Việt
- Story 3.8: Duyệt hàng loạt một phím
- Story 3.9: Quản lý Glossary
- Story 3.10: Xuất và nhập Glossary qua CSV/TSV
- Story 3.10b: Nối hộp thoại chọn tệp vào xuất/nhập Glossary

## Requirements & Constraints

Glossary có tầng Toàn cục và tầng Tác phẩm; khi cùng thuật ngữ xuất hiện ở cả hai, mục của Tác phẩm thắng. Mỗi mục lưu thuật ngữ nguồn, bản dịch có thể chưa có, ghi chú, phân loại, ngày thêm và xuất xứ. Trạng thái đi một chiều từ ứng viên sang chờ chốt bản dịch rồi đã chốt; chỉ mục đã chốt được dùng để ép cách dịch. Người dùng thêm/sửa mục ngay trong ngữ cảnh làm việc, quản lý chúng sau đó, và trao đổi toàn bộ dữ liệu bằng CSV/TSV không cần tài khoản hay server — vế xuất/nhập gồm ba phần tách nhau (quản lý, định dạng và đường ghi, hộp thoại chọn tệp của hệ điều hành); nay cả ba đã đóng.

Máy chỉ được đề xuất: mọi ứng viên vào bảng chờ riêng, cần thao tác duyệt tường minh mới thành mục Glossary. Quét lúc nhập tìm chuỗi lặp ≥5 lần chưa có trong từ điển (ngưỡng cấu hình được), nhận diện tên người theo dữ liệu phù hợp tiếng Trung và cụm viết hoa không đầu câu cho tiếng Anh; chạy nền, không chặn thao tác, không lặp lại chuỗi đã có hoặc đã bỏ.

Khớp phân theo ngôn ngữ: tiếng Trung chính xác, tiếng Anh theo biến thể hình thái. Dấu Glossary hiện ở cột nguyên văn của lưới kể cả khi hiển thị Hán Việt; mục chờ chốt vẫn hiện nhưng phân biệt rõ với mục đã chốt. Ứng viên tiếng Trung có thể nhận đề xuất âm Hán Việt; khi không có đề xuất phù hợp, người dùng chốt bản dịch lần đầu gặp thuật ngữ trong Workspace. Toàn bộ thao tác dùng được bằng bàn phím, đáp ứng tương phản AA ở cả hai theme, và mọi chức năng hoạt động ngoại tuyến.

## Technical Decisions

Bảng chờ là dữ liệu độc lập, không phải một trạng thái trong Glossary. Cả quét khi nhập và thu hoạch từ review về sau cùng ghi vào bảng này; không có đường tự ghi sang Glossary — chỉ thao tác duyệt mới tạo mục, và phải giữ xuất xứ.

Phân giải hai tầng đi qua `ScopeResolver` (tầng Tác phẩm ghi đè theo thuật ngữ). Khớp ngôn ngữ dùng chung một `Matcher` với các năng lực khác, không cài bản riêng. Mô-đun Glossary chỉ phơi một truy vấn trả về mục đủ điều kiện chèn; điều kiện đó thuộc chính mô-đun sở hữu dữ liệu. Âm Hán Việt đọc qua `DictionarySource`, không nhân bản dữ liệu hay tạo vòng phụ thuộc. Mục tầng Tác phẩm nằm trong DB của Tác phẩm, tầng Toàn cục nằm trong kho toàn cục; mọi ghi đi qua writer tương ứng. Lỗi IPC dùng mã, khoá thông báo, tham số và khả năng thử lại; văn bản hiển thị thuộc tài nguyên giao diện.

Hộp thoại chọn tệp (xuất/nhập Glossary) là **API phía Rust** (`AD-48`) — JavaScript không chạm tới nó, chỉ dispatch một command đã đăng ký. `capabilities/main.json` giữ đúng ba quyền, không cấp quyền plugin filesystem/dialog nào ra JS; `tauri_plugin_fs::init()` không được gọi dù crate có mặt trong cây phụ thuộc (có mặt trong cây khác hẳn có mặt trên bề mặt IPC). Cổng `check:deps` canh mã trong nhị phân (bốn tên cấm còn lại), tách biệt với cổng canh bề mặt IPC — hai cổng, hai mệnh đề khác nhau. Payload nhị phân do crate mới thêm được đo bằng byte và ghi lại; vượt 1 MB thì đổi sang crate hộp thoại nhẹ hơn thay vì nới lỏng quy tắc gọi từ Rust. Tệp nhập vào phải có trần kích thước (từ chối tường minh thay vì đọc trọn tệp lớn vào bộ nhớ trên luồng invoke) và phải là UTF-8 (từ chối tường minh, không đoán bảng mã — dò bảng mã là việc của Epic 6). Phụ thuộc mới chỉ thêm sau khi xác minh giấy phép tương thích GPLv3 và ghi nhận vào Stack.

## UX & Interaction Patterns

Thêm nhanh nhận vùng chọn từ mọi bề mặt đã đăng ký và giữ nguyên vị trí làm việc. Bảng chờ xếp theo tần suất, hiển thị số lần xuất hiện, ngữ cảnh và đề xuất khi có; nhận/bỏ và đổi phân loại thao tác bằng phím, phiên duyệt dở mở lại đúng vị trí. Hàng đã xử lý lùi bằng màu chữ và ký hiệu, không dùng `opacity`.

Dấu thuật ngữ đã chốt dùng token `primary`; mục chờ chốt dùng kiểu gạch chân thay vì giảm tương phản. Rê chuột hoặc đưa focus tới dấu hiển thị bản dịch trong `StatusBar`, không tạo lớp nổi. Chốt lần đầu gặp dùng dải nội tuyến đẩy nội dung xuống và thu lại sau khi xử lý; nếu nhiều dải cùng đủ điều kiện, chỉ một dải hiện và việc chốt Glossary có ưu tiên cao nhất. Mọi thao tác giao diện phải đăng ký trước khi gắn vào chuột hoặc phím, chỉ dùng token màu đã kiểm tương phản. Màn hình quản lý Glossary có sẵn hai nút xuất/nhập CSV; huỷ hộp thoại chọn tệp là một lựa chọn hợp lệ (không tệp ghi, không lỗi hiện ra), không phải trường hợp lỗi.

## Cross-Story Dependencies

Mô hình hai tầng và bảng chờ là nền cho thêm nhanh, quét, đề xuất, duyệt, quản lý và trao đổi file. Đường khớp dùng `Matcher` và dữ liệu từ điển đã có; vẽ dấu phụ thuộc vào bề mặt kết quả khớp. Quét ứng viên được kích hoạt bởi luồng nhập, còn duyệt hàng loạt cần cả trạng thái chờ chốt lẫn đề xuất Hán Việt để xử lý đủ mọi ứng viên. Epic kế tiếp chỉ được đọc các mục đã chốt qua truy vấn đủ điều kiện; Epic review sau này tái sử dụng chính bảng chờ, không tạo cơ chế đề xuất thứ hai.

Story 3.10b chỉ nối hộp thoại chọn tệp vào hai hàm đọc/ghi định dạng mà Story 3.10 đã dựng — không mở lại và không nhân bản bước đọc định dạng. Cả ba story của mảng xuất/nhập (3.9, 3.10, 3.10b) nay đã đóng; không story nào ở Epic 4 trở đi bị chặn bởi nhóm này.
