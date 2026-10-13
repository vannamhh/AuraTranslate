# Bảng lệch mockup ↔ app thật — tổng hợp

Bước 1 của `handoff-buoc-1-bang-lech.md`. Chi tiết từng dòng nằm ở ba tệp phần; tệp này chỉ gom số, chủ đề và câu hỏi cho bước 2.

| Phần | Phạm vi | Tệp |
|---|---|---|
| A | Library, nhập (tệp · URL · song ngữ), danh sách Chương, tìm kiếm, dữ liệu/mồ côi, trạng thái rỗng, màn hẹp, Cài đặt, 10 lối vào thanh tiêu đề | `bang-lech-a.md` |
| B | Workspace sáng/tối, Tra cứu, nguồn từ điển, lịch sử/ghim, dải TM, dải chốt Glossary, AI hàng loạt, xem prompt | `bang-lech-b.md` |
| C | Bảng chờ Glossary, Quản lý Glossary, Quản lý TM, Thư viện prompt, Proofreader, Chế độ đọc, Review Mode, Xuất | `bang-lech-c.md` |

## Số dòng theo loại

Đếm bằng `awk` trên cột Loại của các bảng bề mặt, 2026-10-13. Số tự khai của agent lệch tối đa 3 dòng mỗi ô vì vài ô chứa `|`; tin số dưới đây khi đếm lại cùng lệnh.

| Loại | A | B | C | Tổng |
|---|---|---|---|---|
| 1 khớp | 42 | 41 | 52 | 135 |
| 2 lệch cần sửa | 47 | 17 | 54 | 118 |
| 3 lệch có chủ đích | 4 | 18 | 8 | 30 |
| 4 app có, mockup không vẽ | 17 | 17 | 17 | 51 |
| 5 mockup có, app chưa có | 26 | 31 | 19 | 76 |
| cần ảnh, chưa xếp | 8 | 5 | 4 | 17 |

Bằng chứng: mã nguồn và bốn ảnh Ice chụp (Library, Workspace, cột Hán Việt, panel Tra cứu). Không agent nào chạy app.

## Chủ đề lệch loại 2 (để cắt story)

1. **Khung app.** Thanh tiêu đề dồn 10 lối vào (D1). Không mockup nào có nút ở thanh tiêu đề — xem bảng lối vào cuối `bang-lech-a.md`.
2. **Library.** Một cột 420 px xếp dọc thư mục gốc, mồ côi, tìm kiếm, lưới, form nhập; hai phần ba bên phải trống. Mockup: một dải trên cùng + lưới 4 cột.
3. **Danh sách Chương.** Danh sách phẳng chọn một. Mockup: nhóm theo trạng thái, chọn nhiều, thanh hành động gộp/tách/đổi tên/xuất.
4. **Màn xem trước nhập** (tách Chương, URL, song ngữ). Lớp phủ xếp tầng dọc; mockup là màn hai cột. Song ngữ khoá xác nhận khi còn hàng lệch, mockup có "Nhập luôn, nối sau".
5. **Panel Tra cứu.** D2, D5, D6; nút Ghim ở từng mục từ, mockup đặt ở đầu panel.
6. **Lớp phủ quản lý.** Bảng chờ Glossary thiếu thanh tiến độ, bộ lọc, thẻ nở, phím Space. Quản lý Glossary/TM xếp flex-wrap; mockup là bảng có đầu cột, công tắc tầng, cột Xuất xứ, vạch lề trạng thái.
7. **Xem prompt.** Một khối `<pre>` trong hộp thoại; mockup là màn đầy đủ tách tầng, có số token.
8. **Xuất, Proofreader.** Màn xuất thiếu công tắc Nội dung, bốn trường khối ghi nguồn, xem trước. Proofreader thiếu chi phí ước tính trước khi quét và đếm "đã chặn N lần".

## Câu hỏi cho Ice ở bước 2

- **Q1 Chỗ đặt 10 lối vào.** Mockup/IA chỉ đặt chỗ cho #3 bảng chờ Glossary và #8 Review Mode (trong Workspace), #6 Xuất (Library, thanh hành động Chương). #4 Quản lý Glossary, #5 Quản lý TM, #10 Cài đặt là cửa sổ có phím, không vẽ nút. #1 Quét chính tả, #2 Chỗ đã bỏ qua, #7 Nhập bản reviewer, #9 Nối tay không có lối mở nào. Phím mặc định trong mã (`src/commands/index.ts`): `Mod+Alt+Q` bảng chờ, `Mod+Alt+M` Quản lý Glossary, `Mod+Alt+3` Review Mode, `Mod+Comma` Cài đặt; sáu lối còn lại không có phím. Mockup tự va phím: `⌘M` vừa Quản lý TM vừa gộp segment (mã giữ `Mod+M` cho gộp), `⌘3` vừa Review Mode vừa Chế độ đọc.
- **Q2 Nguồn Hán Việt.** D4 bỏ dòng trên từng câu; đó là chỗ render duy nhất. Hiện một lần ở chỗ khác hay không hiện?
- **Q3 Dải gợi ý.** Dải TM, dải chốt Glossary và dải Proofreader dùng chung một slot ngay trên thanh trạng thái (`App.vue`, `topmostStrip`, plan Story 3.6 và 7.5). Mockup cho chúng mọc ở đáy panel / dưới câu đang sửa. Theo mockup, hay giữ slot chung?
- **Q4 Loại 5.** 76 dòng là chức năng mockup vẽ mà app chưa có. Đưa vào epic giao diện, hay ghi nợ có chủ để epic này chỉ làm bố cục?
- **Q5 Quyết định nằm trong plan story, không qua `change-*`.** Bỏ bộ lọc phạm vi Lịch sử (plan lịch sử tra cứu, Quyết định #5); thu hoạch thuật ngữ vào bảng chờ thay vì hộp riêng (plan thu hoạch, AD-20). Agent xếp loại 3; Ice xác nhận.
- **Q6 Hình dạng popup nguồn (D2).** Sally đưa phương án.

## Cần ảnh (gộp, bỏ trùng; dpr=2)

Library: cuộn hết trang có Tác phẩm · Library rỗng · sau quét lại có mồ côi · kết quả tìm kiếm · danh sách Chương. Nhập: xem trước txt · URL · song ngữ · màn nối lệch. Workspace: nền tối có một hàng đang sửa và một hàng TM điền sẵn · 1440×790 và 1180×740 · thanh trạng thái sau khi lưu. Tra cứu: tab Lịch sử có ghim · một chữ nhiều nguồn (打). Dải: TM khớp mờ và khớp 100% · chốt Glossary · Proofreader. AI: đang sinh · lô chạy dở có lỗi · xem prompt. Lớp phủ: bảng chờ Glossary · Quản lý Glossary · Quản lý TM · Thư viện prompt · Proofreader chọn phạm vi và chỗ đã bỏ qua · Lịch sử phiên bản · Cài đặt (AI tầng Tác phẩm, Phím tắt, Quyền riêng tư) · Attribution. Chế độ đọc: ba mức, sáng/tối, cuối trang, Chỗ cần sửa, Mục lục. Review Mode: hai panel · nhập reviewer · nối tay. Xuất: từng định dạng · ảnh thiếu link · khối ghi nguồn. Trạng thái rỗng/lỗi: Lookup, TM, AI.
