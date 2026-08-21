# Epic 3 Context: Glossary — chốt thuật ngữ một lần, dùng mãi

<!-- Compiled from planning artifacts. Edit freely. Regenerate with compile-epic-context if planning docs change. -->

## Goal

Epic này dựng Glossary — nơi người dịch chốt cách dịch cho tên riêng, địa danh và thuật ngữ chuyên ngành một lần, để Smart RAG Injector (Epic 4) có dữ liệu đáng tin ép AI dùng đúng cách gọi đã chốt. Hai trụ chính: thêm nhanh một thuật ngữ ngay tại chỗ đang dịch mà không rời màn hình, và duyệt hàng loạt hàng trăm ứng viên do máy quét ra chỉ bằng phím tắt, không gõ chữ. Mọi đề xuất tự động (quét khi nhập, thu hoạch từ bản review ở Epic 8) dừng ở một bảng chờ tách biệt; không cơ chế nào được tự ghi vào Glossary — chỉ người dùng mới chốt.

## Stories

- Story 3.1: Mô hình Glossary hai tầng và vòng đời ba trạng thái
- Story 3.2: Bảng chờ ứng viên tách hẳn khỏi Glossary
- Story 3.3: Thêm nhanh thuật ngữ từ bất kỳ panel nào
- Story 3.4: Khớp thuật ngữ theo ngôn ngữ qua Matcher dùng chung 🔵 *(thu hẹp 2026-08-21)*
- Story 3.4b: Đánh dấu thuật ngữ ở cột nguyên văn của lưới 🔵 *(thêm 2026-08-21 qua `correct-course`)*
- Story 3.5: Quét ứng viên khi nhập tài liệu
- Story 3.6: Trạng thái chờ chốt và dải mọc chốt lần đầu gặp
- Story 3.7: Đề xuất bản dịch bằng âm Hán Việt
- Story 3.8: Duyệt hàng loạt một phím
- Story 3.9: Quản lý Glossary
- Story 3.10: Xuất và nhập Glossary qua CSV/TSV

## Requirements & Constraints

- Hai tầng: toàn cục (mọi Tác phẩm) và theo Tác phẩm; trùng thuật ngữ thì tầng Tác phẩm thắng. Mỗi mục mang thuật ngữ nguồn, bản dịch, ghi chú, phân loại (người/địa danh/chuyên ngành/khác), ngày thêm, xuất xứ (nhập tay / đề xuất khi nhập tài liệu / thu hoạch từ bản review).
- Vòng đời một chiều: ứng viên → chờ chốt bản dịch → đã chốt; bản dịch nullable tới khi chốt. Chỉ mục đã chốt được đưa vào ép AI — một trường trống trong prompt gợi ý sai cho mô hình rằng thuật ngữ không có bản dịch quy ước.
- Thêm nhanh dùng được từ mọi bề mặt vùng chọn đã đăng ký (cột nguyên văn của lưới, Panel Lookup, Panel AI Translation, cột bản dịch của lưới), không rời màn hình, toàn bộ bằng bàn phím. Thuật ngữ đã chốt đánh dấu ở cột nguyên văn của lưới; mục chờ chốt cũng đánh dấu nhưng phải phân biệt được.
- Khớp theo ngôn ngữ: tiếng Trung khớp chính xác, tiếng Anh khớp mờ qua stemming.
- Quét khi nhập: chuỗi lặp ≥5 lần (ngưỡng cấu hình được) và không có trong từ điển nhúng, ghi vào bảng chờ kèm số lần xuất hiện + ví dụ ngữ cảnh, chạy nền không chặn người dùng.
- Thu hoạch từ bản review (Epic 8, FR54/FR95): đổi thuật ngữ nhất quán trong bản Reviewer được đề xuất bổ sung kèm tỉ lệ nhất quán, kích hoạt độc lập với việc có mở Review Mode hay không.
- Nguyên tắc xuyên suốt: mọi đề xuất tự động phải qua duyệt tường minh của người dùng; không cơ chế nào tự ghi vào Glossary.
- Ứng viên tiếng Trung được đề xuất sẵn bản dịch bằng âm Hán Việt (dữ liệu nhúng, chạy ngoại tuyến hoàn toàn); nhận một phím thì vào thẳng trạng thái đã chốt. Tiếng Anh, hoặc tiếng Trung không tra được âm Hán Việt phù hợp, không có đề xuất, đi theo đường chờ chốt.
- Chờ chốt: lần đầu gặp thuật ngữ trong Workspace, hệ thống hỏi đúng một lần rồi khoá thành đã chốt.
- Duyệt hàng loạt: sắp theo tần suất giảm dần, một phím cho nhận/bỏ mỗi mục, phím số đổi phân loại, đóng mở lại phải quay đúng vị trí đang duyệt.
- Quản lý: tìm theo nguồn lẫn bản dịch, lọc theo phân loại/xuất xứ/trạng thái, sửa có hiệu lực ngay, xoá thì mất luôn đánh dấu ở lưới, đẩy một mục từ Tác phẩm lên Global bằng một thao tác.
- Xuất/nhập CSV/TSV: round-trip đủ trường; xung đột khi nhập phải hỏi người dùng, không im lặng ghi đè; file sai định dạng báo rõ dòng/cột thiếu và không ghi một phần; mục nhập từ file mang xuất xứ riêng; mục không có bản dịch vào trạng thái chờ chốt, không phải đã chốt rỗng.

## Technical Decisions

- Bảng ứng viên là bảng riêng, tách hẳn khỏi Glossary — không phải cột trạng thái. Không đường ghi nào từ cơ chế đề xuất tự động thẳng vào Glossary; chỉ qua thao tác duyệt tường minh (AD-20). Cùng một bảng chờ này sẽ được Epic 8 (thu hoạch từ bản review) ghi vào sau, không cần bảng thứ hai.
- Vòng đời ba trạng thái một chiều (AD-36). Module `glossary/` phơi đúng một truy vấn trả về mục đủ điều kiện chèn prompt; điều kiện chèn (chỉ trạng thái đã chốt) nằm trong `glossary/`, không ở nơi gọi — `ai/` không có đường nào khác chạm dữ liệu Glossary.
- Phân giải hai tầng qua đúng một `ScopeResolver` dùng chung (AD-18); ngữ nghĩa Glossary là "ghi đè" (tầng Tác phẩm thắng), khác với "hợp nhất" của Translation Memory.
- Khớp thuật ngữ dùng đúng một component `Matcher` dùng chung với từ điển và TM (AD-17), không cài lại riêng. Tiếng Trung: khớp chính xác + n-gram ký tự (tách từ qua `jieba-rs` khi cần). Tiếng Anh: stemming + token n-gram.
- Âm Hán Việt cho đề xuất (FR113) đọc qua cổng `DictionarySource`, không cài lại dữ liệu bên trong `glossary/`. Thêm cạnh phụ thuộc `glossary/ → dict/`, không tạo chu trình ngược.
- Lưu trữ: mục tầng Tác phẩm nằm trong `project.db` của Tác phẩm đó; mục tầng Global nằm trong `global.db`. Mỗi kho ghi được có đúng một kết nối ghi sau hàng đợi nối tiếp; đọc dùng pool song song (WAL); không module nào tự mở kết nối ghi riêng.
- Không thêm port thứ tư ngoài `DictionarySource`/`TranslationProvider`/`ProjectStore` mà thiếu một Architecture Decision mới. Chuỗi hiển thị không viết cứng trong Rust — lỗi/thông báo qua IPC dạng `{ code, message_key, params, retryable }`.

## UX & Interaction Patterns

- "Dải mọc" dưới câu đang sửa là mẫu dùng chung thay hộp thoại — cho chốt Glossary lần đầu gặp (Story 3.6), phát hiện Proofreader (Epic 9), gợi ý TM khớp mờ (Epic 7). Đẩy văn bản xuống, không phủ lên; thu lại ngay khi xong; dải kế tiếp mọc đúng chỗ vừa thu, vị trí không nhảy.
- Chỉ một dải mọc tại một thời điểm: cái nào chặn thật thắng, cái nào chỉ gợi ý thì nhường. Chốt Glossary luôn thắng vì thuật ngữ chưa chốt không tham gia ép AI — để treo là để một lỗ hổng chạy tiếp qua mọi câu sau.
- Bảng chờ duyệt hàng loạt: mỗi dòng hiện số lần xuất hiện, ví dụ ngữ cảnh, bản dịch đề xuất khi có. Hàng đã duyệt/đã bỏ lùi ra sau bằng đổi màu chữ sang `on-surface-variant` cộng dấu tick/x — tuyệt đối không dùng `opacity` (một đợt kiểm toán trước từng để lọt lỗi tương phản chính vì dùng opacity ở đúng màn hình này).
- Màu nhấn `primary` (xanh mực) chỉ dành cho đúng ba việc toàn ứng dụng, gồm đánh dấu thuật ngữ Glossary đã chốt — không dùng cho nút bấm hay tiêu đề thường.
- Toàn bộ luồng thêm nhanh, duyệt hàng loạt, quản lý Glossary phải làm được hoàn toàn bằng bàn phím.

## Cross-Story Dependencies

- Story 3.4 phải dùng lại `Matcher` đã dựng ở Epic 1, không cài đặt khớp ngôn ngữ riêng cho Glossary.
- Story 3.4b phụ thuộc bề mặt IPC `glossary_marks_for_chapter` mà Story 3.4 dựng. Nó gọi **MỘT** lượt mỗi lần mở Chương, cộng một lượt làm mới khi Glossary đổi hoặc khi segment gộp/tách — **không một lượt nào trên đường gõ** (Ice ký 2026-08-21). Đó là thứ giữ **214 ms** *(Chương 48.640 ký tự, Glossary 5.000 mục)* nằm NGOÀI trần NFR2 **50 ms**, và là lý do không story nào ở đây thêm cache hay chỉ mục ngược.
- Story 3.7 phụ thuộc dữ liệu Hán Việt nhúng và cổng `DictionarySource` từ Epic 1.
- Story 3.6 chia sẻ cơ chế "chỉ một dải mọc" với Epic 7 (TM khớp mờ) và Epic 9 (Proofreader); Glossary luôn thắng khi cùng kích hoạt trên một câu.
- Epic 4 (Smart RAG Injector) phụ thuộc trực tiếp vào truy vấn "mục đủ điều kiện chèn" mà Story 3.1 dựng — không có đường nào khác để chạm dữ liệu Glossary.
