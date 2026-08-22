# Epic 3 Context: Glossary — chốt thuật ngữ một lần, dùng mãi

<!-- Compiled from planning artifacts. Edit freely. Regenerate with compile-epic-context if planning docs change. -->

## Goal

Epic này tạo Glossary để người dịch lưu và tái dùng một quyết định biên tập cho tên riêng, địa danh và thuật ngữ chuyên ngành. Nó phải hạ gánh nặng khởi đầu của một Tác phẩm dài bằng cách tìm ứng viên lúc nhập, nhưng vẫn giữ người dùng là người duy nhất quyết định mục nào được vào Glossary. Kết quả là một nguồn thuật ngữ đáng tin cho Workspace hôm nay và cho việc ép cách dùng nhất quán ở các năng lực sau này.

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

## Requirements & Constraints

Glossary có tầng Toàn cục và tầng Tác phẩm; khi cùng thuật ngữ xuất hiện ở cả hai, mục của Tác phẩm thắng. Mỗi mục lưu thuật ngữ nguồn, bản dịch có thể chưa có, ghi chú, phân loại, ngày thêm và xuất xứ. Trạng thái đi một chiều từ ứng viên sang chờ chốt bản dịch rồi đã chốt; chỉ mục đã chốt được dùng để ép cách dịch. Người dùng có thể thêm hoặc sửa mục ngay trong ngữ cảnh làm việc, quản lý chúng sau đó, và trao đổi toàn bộ dữ liệu bằng CSV/TSV không cần tài khoản hay server.

Máy chỉ được đề xuất: mọi ứng viên phải vào bảng chờ riêng và cần thao tác duyệt tường minh để thành mục Glossary. Quét lúc nhập tìm chuỗi lặp lại chưa có trong từ điển, với ngưỡng cấu hình được; nhận diện tên người theo dữ liệu phù hợp với tiếng Trung và cụm viết hoa không ở đầu câu cho tiếng Anh. Mỗi ứng viên cần số lần xuất hiện và ví dụ ngữ cảnh, quét nền để không chặn thao tác, và không lặp lại chuỗi đã có hoặc đã bị bỏ trong cùng Tác phẩm.

Khớp Glossary phân theo ngôn ngữ: tiếng Trung chính xác, tiếng Anh theo biến thể hình thái. Dấu Glossary phải hiện ở cột nguyên văn của lưới, kể cả khi hiển thị Hán Việt; mục chờ chốt vẫn hiện nhưng phân biệt rõ với mục đã chốt. Ứng viên tiếng Trung có thể nhận đề xuất âm Hán Việt từ dữ liệu nhúng; khi không có đề xuất phù hợp, người dùng chốt bản dịch lần đầu gặp thuật ngữ trong Workspace. Toàn bộ thao tác phải dùng được bằng bàn phím, đáp ứng tương phản AA ở cả hai theme, và mọi chức năng vẫn hoạt động ngoại tuyến.

## Technical Decisions

Bảng chờ là dữ liệu độc lập, không phải một trạng thái trong Glossary. Cả quét khi nhập và thu hoạch từ review về sau cùng ghi vào bảng này; không tồn tại đường tự ghi sang Glossary. Chỉ thao tác duyệt mới tạo mục Glossary và phải giữ xuất xứ của nó.

Phân giải hai tầng bắt buộc đi qua `ScopeResolver`, với quy tắc ghi đè theo thuật ngữ của tầng Tác phẩm. Khớp ngôn ngữ dùng chung một `Matcher` với các năng lực khác, không tạo bản cài đặt riêng. Mô-đun Glossary chỉ phơi một truy vấn trả về các mục đủ điều kiện chèn; điều kiện đó thuộc chính mô-đun sở hữu dữ liệu, không được sao chép ở nơi gọi. Âm Hán Việt đọc qua `DictionarySource`, không nhân bản dữ liệu hay tạo vòng phụ thuộc.

Mục tầng Tác phẩm nằm trong cơ sở dữ liệu của Tác phẩm, còn tầng Toàn cục nằm trong kho toàn cục. Mọi ghi đi qua writer của kho tương ứng. Lỗi IPC dùng mã, khoá thông báo, tham số và khả năng thử lại; văn bản hiển thị thuộc tài nguyên giao diện. Phụ thuộc mới chỉ được thêm sau khi xác minh giấy phép tương thích GPLv3 và ghi nhận vào Stack.

## UX & Interaction Patterns

Thêm nhanh nhận vùng chọn từ mọi bề mặt đã đăng ký và giữ nguyên vị trí làm việc. Bảng chờ xếp theo tần suất, hiển thị số lần xuất hiện, ngữ cảnh và đề xuất khi có; nhận/bỏ và đổi phân loại phải thao tác bằng phím, đồng thời phiên duyệt dở phải mở lại đúng vị trí. Hàng đã xử lý lùi bằng màu chữ và ký hiệu, không dùng `opacity`.

Dấu thuật ngữ đã chốt dùng token `primary`; mục chờ chốt dùng kiểu gạch chân thay vì giảm tương phản. Rê chuột hoặc đưa focus tới dấu hiển thị bản dịch trong `StatusBar`, không tạo lớp nổi. Việc chốt lần đầu gặp dùng dải nội tuyến đẩy nội dung xuống và thu lại sau khi xử lý; nếu nhiều dải cùng đủ điều kiện, chỉ một dải hiện và việc chốt Glossary có ưu tiên cao nhất. Mọi thao tác giao diện phải được đăng ký trước khi gắn vào chuột hoặc phím, và chỉ dùng token màu đã kiểm tương phản.

## Cross-Story Dependencies

Mô hình hai tầng và bảng chờ là nền cho thêm nhanh, quét, đề xuất, duyệt, quản lý và trao đổi file. Đường khớp dùng `Matcher` và dữ liệu từ điển đã có; phần vẽ dấu phụ thuộc vào bề mặt kết quả khớp. Quét ứng viên được kích hoạt bởi luồng nhập, còn duyệt hàng loạt cần cả trạng thái chờ chốt lẫn đề xuất Hán Việt để xử lý đủ mọi ứng viên. Epic kế tiếp chỉ được đọc các mục đã chốt qua truy vấn đủ điều kiện; Epic review sau này tái sử dụng chính bảng chờ, không tạo cơ chế đề xuất thứ hai.
