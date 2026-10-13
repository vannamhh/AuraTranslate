# Ảnh app đang chạy (macOS, dev build, nền sáng, dpr=2, cửa sổ 1280×860 điểm = 2560×1720)

Chụp 2026-10-13 bằng `screencapture -l`, điều khiển qua Orca. Chỉ mở Tác phẩm `test (12).atproj`. Đã đối chiếu: các `project.db` khác không đổi mtime.

| Tệp | Bề mặt · trạng thái | Đã thấy gì liên quan bảng lệch |
|---|---|---|
| library-da-tai-dau-trang.png | Library, đầu trang, 47 Tác phẩm đã tải, chưa mở Tác phẩm | Thư mục gốc, mồ côi, tìm kiếm, bộ lọc xếp dọc một cột ~420 px, hai phần ba bên phải trống |
| library-luoi-the-giua-trang.png | Library, giữa lưới thẻ | Thẻ bìa chữ cái đầu, "chưa biết", "Chưa biết tiến độ" (0 thẻ có tiến độ thật) |
| library-khoi-chuong-chua-mo-va-form-nhap.png | Library, khối Chương khi chưa mở Tác phẩm + đầu form nhập | Câu "Bấm Tải danh sách…" và lỗi đỏ khi chưa có Tác phẩm đang mở |
| library-form-nhap-cuoi-trang.png | Library, cuối trang | Form nhập: văn bản, tệp, song ngữ, URL xếp dọc |
| library-tim-kiem-co-ket-qua.png | Library, tìm `布拉姆斯`, 50 kết quả | Kết quả là thẻ chữ thuần, từ khớp bọc ‹ ›, không tô màu; dòng "35/47 chưa có văn bản trong chỉ mục" |
| library-trang-thai-tac-pham-va-chuong-1-chuong.png | Library, đã mở `test (12)`: khối Trạng thái + Chương | Nút Dời lên/Dời xuống/Gộp, ô Tên Chương |
| library-xuat-xu-tai-lieu-va-chuong-cuoi.png | Library, khối Xuất xứ tài liệu + hàng Chương | Hàng Chương duy nhất "1 · Chương 1 · Chưa bắt đầu · 7", không tay kéo, không tiến độ câu, không thời điểm |
| workspace-chua-mo-tac-pham.png | Workspace khi chưa mở Tác phẩm | Hai panel Đối chiếu và Tra cứu/Đề xuất AI ở trạng thái rỗng |
| workspace-test12-nen-sang.png | Workspace, `test (12)`, nền sáng | Bảng Đối chiếu: số câu, nguyên văn, ô dịch, chip "đã xác nhận"/"chưa dịch"; tab Tra cứu / Đề xuất AI |
| workspace-cot-han-viet.png | Workspace, tab cột Hán Việt | Dòng "NGUỒN: …" lặp trên từng câu (D4 của bảng lệch) |
| tra-cuu-chua-tra.png | Tra cứu, chưa tra | Dải Nguồn + tab Từ điển/Concordance/Lịch sử; CC-CEDICT gạch ngang (đang tắt) |
| tra-cuu-tu-dien-nhieu-nguon.png | Tra cứu `小提琴`, tab Từ điển | Ba nguồn, dòng "Có nhiều hơn một nguồn", nút Ghim ở từng mục từ |
| tra-cuu-concordance-co-ket-qua.png | Tra cứu, tab Concordance sau khi bôi chọn | Câu hướng dẫn Concordance (chưa có kết quả, chưa bấm phím tắt) |
| tra-cuu-lich-su-1-muc-chua-ghim-sau-tra-tieu-de-cam.png | Tra cứu, tab Lịch sử có 1 mục, 0 ghim | Đã ghim (0) + Tra gần đây (1 mục) |
| tra-cuu-tab-concordance-chua-tra.png · tra-cuu-tab-lich-su-rong.png | Hai tab trước khi tra | Trạng thái rỗng |
| de-xuat-ai-chua-cau-hinh-co-tac-pham.png | Đề xuất AI, AI chưa cấu hình | Một đoạn duy nhất, không tiêu đề, không Ollama/LM Studio, không `⌘,` |
| xem-prompt-chua-lap.png | Xem prompt, chưa lắp ráp | Khung ~68% bề ngang, không tab; chỉ một câu "Chưa lắp ráp prompt nào" |
| prompt-thu-vien-chua-mo-tac-pham.png | Thư viện prompt (chưa mở Tác phẩm) | Hai tiêu đề tầng, "Bộ mới", "Nhập từ file" |
| prompt-bo-moi-form.png | Thư viện prompt, form "Tạo bộ mới" | Ô nội dung prompt là textarea thuần, không tô biến |
| glossary-quan-ly-chua-mo-tac-pham.png · glossary-quan-ly-co-tac-pham.png | Quản lý Glossary, không/có Tác phẩm | Có Tác phẩm: 2 hàng (姆斯, 这首) có badge "Tác phẩm này", không vạch lề trạng thái |
| tm-quan-ly-chua-mo-tac-pham.png · tm-quan-ly-co-tac-pham.png | Quản lý TM, trống | Dải sức khoẻ 0%, không có hàng nào để thấy vạch lề |
| glossary-bang-cho-chua-mo-tac-pham.png · glossary-bang-cho-co-tac-pham.png | Bảng chờ Glossary | Không có hàng chờ nào |
| cho-da-bo-qua-chua-mo-tac-pham.png · cho-da-bo-qua-co-tac-pham.png | Chỗ đã bỏ qua | Danh sách trống |
| quet-chinh-ta-chua-mo-tac-pham.png · quet-chinh-ta-co-tac-pham.png | Quét chính tả | Có Tác phẩm: popover ba lựa chọn Câu này / Vùng chọn / Cả Chương (không phải lớp phủ), không chi phí ước tính |
| xuat-chua-mo-tac-pham.png · xuat-co-tac-pham.png · xuat-co-tac-pham-cuon-xem-truoc.png | Xuất | Có Tác phẩm: phạm vi, định dạng, ảnh, khối ghi nguồn một ô, xem trước hai cột; không có công tắc Nội dung |
| nhap-reviewer-chua-mo-tac-pham.png | Nhập bản reviewer | Mở hộp thoại tệp macOS gốc (đã Huỷ bằng Escape), không có lớp phủ của app |
| review-mode-thong-bao-chua-co-ban-reviewer.png | Mở Review Mode | Chỉ hiện lời báo "Chương này chưa có bản Reviewer", không vào Review Mode |
| noi-tay-chua-mo-tac-pham.png · noi-tay-co-tac-pham.png | Nối tay | Chỉ hiện lỗi "chưa có bản reviewer" |
| attribution-nguon-du-lieu-dau.png | Nguồn dữ liệu và ghi công (nút trong panel Tra cứu) | Khung ~86% bề ngang, bảng Nguồn · Giấy phép · Vai trò · Ghi công, nút Mở giấy phép / Sao chép bản ghi công |
| che-do-doc-mo-dau.png · che-do-doc-muc-luc.png · che-do-doc-cho-can-sua.png | Chế độ đọc | Chương chưa dịch xong nên chỉ có "Điểm dừng của lượt đọc", không có cột đọc |
| cai-dat-ai-mo-hinh.png · cai-dat-ai-tang-tac-pham.png · cai-dat-prompt.png · cai-dat-glossary.png · cai-dat-tm.png · cai-dat-xuat.png · cai-dat-phim-tat.png · cai-dat-quyen-rieng-tu.png | Cài đặt, từng mục | AI ở tầng Tác phẩm chưa có trường ghi đè; Phím tắt là bảng Thao tác · Mã lệnh · Phím; Quyền riêng tư là nhật ký mạng trống |

## Không chụp được khi chỉ xem

- Nền tối (Workspace, Đối chiếu, Chế độ đọc, xem trước nhập): theme ghi qua `putConfig` (`src/tokens/themeState.ts::setTheme`), bỏ qua.
- Xem trước nhập (txt, URL, song ngữ, màn nối lệch): phải bấm Nhập/Tạo, ghi dữ liệu.
- Panel Đề xuất AI đang sinh, lô có lỗi, lỗi AI retryable, xem prompt đủ Glossary + TM: cần gọi AI và cấu hình nhà cung cấp.
- Dải TM mờ/100%, dải chốt Glossary, dải Proofreader, câu gạch chân, phạm vi/chỗ bỏ qua có dữ liệu: cần xác nhận câu hoặc chạy quét.
- Thanh trạng thái sau khi lưu: cần lưu.
- Lịch sử phiên bản ≥3 phiên bản, bảng chờ Glossary có hàng, Quản lý TM có cặp, ca Glossary tầng Toàn cục: dữ liệu không có và dựng nó là ghi.
- Chế độ đọc ba mức, cuối trang, Mục lục có Chương, Chỗ cần sửa có dữ liệu: Chương duy nhất chưa dịch xong; đổi mức Thoáng/Đặc và Đổi giao diện ghi cấu hình.
- Review Mode hai panel, hộp Nhập reviewer sau khi nhập, màn nối tay: cần nhập tệp reviewer.
- Xuất từng định dạng, ảnh thiếu link, khối ghi nguồn bật: cần đổi lựa chọn rồi xuất; không có ảnh trong Tác phẩm.
- Library rỗng, Library sau Quét lại có mồ côi: cấm Quét lại và đổi thư mục gốc.
- 1440×790 và 1180×740: đổi kích thước cửa sổ có thể ghi hình học; ghi đè bố cục: ghi cấu hình.
- Tra cứu 打 và tra chữ không có kết quả: không có chữ đó trong 7 câu; chỉ thử bôi chọn `小提琴`.
- Trường ghi đè AI ở tầng Tác phẩm (nhãn kế thừa/ghi đè): chưa có ghi đè nào, tạo ra là Lưu.

## Mâu thuẫn với bảng

- `bang-lech-a.md` dòng "AI chưa cấu hình": đang chờ ảnh với Loại 1 tạm; ảnh cho thấy thiếu tiêu đề, dòng Ollama/LM Studio và `⌘,`, nên đã đổi thành 2.
- `bang-lech.md`: bản tóm tắt ghi "Library: một cột 420 px xếp dọc"; ảnh xác nhận. Ảnh còn cho thấy mọi thẻ Tác phẩm hiện "chưa biết / Chưa biết tiến độ" (35/47 chưa có chỉ mục), ngoài phạm vi bảng.
- `bang-lech-c.md` nói "Quản lý Glossary xếp flex-wrap": ảnh cho thấy hàng danh sách, không phải flex-wrap; chưa sửa dòng nào (bố cục chưa được xếp loại trên dòng "cần ảnh").
- Review Mode không mở được khi Chương chưa có bản reviewer: lối vào chỉ cho lời báo (ghi nhận, không phải mâu thuẫn).
