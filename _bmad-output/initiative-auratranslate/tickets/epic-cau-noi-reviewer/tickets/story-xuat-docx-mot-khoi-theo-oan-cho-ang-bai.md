---
id: 4
type: story
title: "Xuất `.docx` một khối theo đoạn cho đăng bài"
parent: epic-cau-noi-reviewer
covers: [FR121]
after: [2, 3, 5]
refined: true
hitl: false
risk: low
---

# Xuất `.docx` một khối theo đoạn cho đăng bài

## Description

Người dịch xuất phạm vi đã chọn thành một `.docx` mà mỗi Chương là một bảng hai cột chỉ có một hàng, không đường kẻ: cột trái là nguyên văn, cột phải là bản dịch, mỗi cột giữ cấu trúc đoạn của chính nó (AD-46). Bôi đen cột phải rồi dán sang trình soạn thảo website sẽ ra văn bản liền mạch để đăng bài. Màn hình xuất nói ngay lúc chọn rằng định dạng này không nhập lại được, và cảnh báo trước lúc xuất khi phạm vi còn câu chưa xác nhận hoặc chưa dịch.

## Acceptance Criteria

1. **Mỗi Chương còn câu là một bảng một hàng**
   **Given** một phạm vi xuất theo FR89 gồm N Chương còn câu thuộc bản dịch và M Chương không còn câu nào
   **When** người dùng xuất định dạng một khối
   **Then** file có đúng N bảng theo thứ tự Chương, mỗi bảng đúng một hàng hai ô
   **And** mỗi Chương có tiêu đề là một đoạn nằm ngoài bảng, ngay trên bảng; M Chương kia chỉ có tiêu đề, không có bảng
2. **Không đường kẻ**
   **Given** file vừa xuất
   **When** kiểm viền bảng
   **Then** không bảng nào có viền: không trên, dưới, trái, phải, giữa hàng hay giữa cột
3. **Cột phải giữ cấu trúc đoạn của bản dịch**
   **Given** một Chương có cờ kết đoạn đích và có ký tự xuống dòng trong bản dịch của một câu
   **When** xuất định dạng một khối
   **Then** ô phải bắt đầu đoạn mới đúng sau mỗi câu bật cờ kết đoạn đích và đúng tại mỗi ký tự xuống dòng; câu trong cùng một đoạn nối nhau bằng một dấu cách
   **And** nhiều ký tự xuống dòng liền nhau tính là một ranh giới đoạn, xuống dòng ở đầu hay cuối câu không tạo đoạn, và ô phải không có đoạn rỗng
   **And** đổi cờ kết đoạn nguồn mà giữ nguyên cờ đích thì ô phải không đổi
4. **Cột trái giữ cấu trúc đoạn của nguyên văn**
   **Given** cùng Chương đó
   **When** xuất định dạng một khối
   **Then** ô trái bắt đầu đoạn mới đúng sau mỗi câu bật cờ kết đoạn nguồn; câu trong cùng một đoạn nối liền, không ký tự chen giữa
   **And** đổi cờ kết đoạn đích mà giữ nguyên cờ nguồn thì ô trái không đổi
5. **Đoạn chỉ đến từ dữ liệu đã lưu**
   **Given** hai Chương có cùng cờ kết đoạn và cùng vị trí ranh giới xuống dòng nhưng khác dấu câu, độ dài câu và số dòng trống liền nhau
   **When** xuất cả hai
   **Then** hai Chương có cùng số đoạn ở mỗi cột (AD-37, AD-46)
6. **Câu bị lược và câu về hưu không ra file (FR133)**
   **Given** một Chương có câu bị lược mang cờ kết đoạn nguồn và cờ kết đoạn đích, có câu bị lược đứng đầu Chương, và có câu đã về hưu
   **When** xuất định dạng một khối
   **Then** các câu đó không xuất hiện ở cột nào
   **And** đoạn vẫn kết tại câu còn lại liền trước câu bị lược, ở cả cột trái lẫn cột phải
   **And** câu bị lược đứng đầu Chương không tạo đoạn rỗng ở cột nào
7. **Câu chưa dịch không để lại dấu vết ở cột phải**
   **Given** một đoạn có câu chưa có bản dịch, một đoạn mà mọi câu đều chưa có bản dịch, và một Chương chưa câu nào có bản dịch
   **When** xuất định dạng một khối
   **Then** cột phải không có dấu cách thừa và không có đoạn rỗng; ô phải của Chương chưa dịch câu nào để trống
   **And** cột trái vẫn có đủ nguyên văn của các câu đó
8. **Ảnh là một đoạn tại vị trí neo**
   **Given** phạm vi xuất có ảnh, với cách xuất ảnh chọn theo FR130
   **When** xuất định dạng một khối
   **Then** mỗi ảnh là một đoạn riêng ngay sau câu neo của nó, có ở cả hai ô
   **And** chọn *theo link gốc* thì đoạn đó là một liên kết tới URL gốc của ảnh; chọn *theo file ảnh* thì đoạn đó là đường dẫn tương đối tới bản chép của ảnh trong thư mục `<tên tệp>-anh/` cạnh file
   **And** ảnh không có URL gốc khi chọn *theo link gốc* bị bỏ qua và được đếm trên màn hình kết quả, như bản hai cột
9. **Cột phải dán ra văn bản liền mạch**
   **Given** file vừa xuất
   **When** kiểm nội dung ô phải
   **Then** ô chỉ chứa đoạn văn và liên kết ảnh: không bảng lồng, không nền màu, không tô sáng, không ghi chú, không số câu
10. **Không đánh dấu câu chưa xác nhận trong file**
    **Given** một Chương có cả câu đã xác nhận lẫn câu chưa xác nhận
    **When** xuất định dạng một khối
    **Then** hai loại câu ra cùng một định dạng chữ trong file
11. **Màn hình nói "không nhập lại được" ngay lúc chọn**
    **Given** màn hình xuất đang mở
    **When** người dùng chọn định dạng một khối
    **Then** ngay dưới lựa chọn hiện dòng nói định dạng này không nhập lại được và dành cho đăng bài, thay cho dòng "nhập lại được" của bản hai cột
    **And** nút xuất tạo bản một khối, không tạo bản hai cột
12. **Cảnh báo trước lúc xuất, tách câu chưa dịch**
    **Given** phạm vi xuất còn câu có bản dịch nhưng chưa xác nhận, và câu chưa có bản dịch
    **When** người dùng chọn định dạng một khối
    **Then** trước khi bấm xuất, màn hình nêu số câu chưa xác nhận và nói file sẽ không phân biệt chúng
    **And** màn hình nêu riêng số câu chưa dịch và nói chúng sẽ vắng mặt ở cột phải
    **And** câu bị lược và câu về hưu không vào số nào; số nào bằng 0 thì vế cảnh báo đó không hiện; người dùng vẫn xuất được
13. **Tệp mới, không ghi đè**
    **Given** thư mục đích đã có tệp cùng tên
    **When** xuất định dạng một khối
    **Then** tệp mới mang tên `<tên Tác phẩm>-mot-khoi.docx` kèm hậu tố ` (n)` như bản hai cột, và tệp có sẵn còn nguyên

## Boundaries

- Must not change: bản `.docx` hai cột của 8.3 và 8.5 (một hàng mỗi câu, ngắt dòng mềm trong ô, dòng "nhập lại được"); số đếm và cảnh báo phạm vi của 8.2 khi chọn bản hai cột; dữ liệu Tác phẩm (xuất chỉ đọc).
- Không thuộc story này: cổng hình dạng AD-38 khi nhập (8.8), khối ghi nguồn FR131 (8.7), `.md` và text thuần (8.6), tiêu đề hay ghi chú ở đầu bài đăng.

## References

- parent — _bmad-output/initiative-auratranslate/epic-cau-noi-reviewer/epic-cau-noi-reviewer.md, Requirements FR121 và FR130, Notes
- constraint — _bmad-output/initiative-auratranslate/architecture-auratranslate/architecture-auratranslate.md, AD-37, AD-38, AD-43, AD-46
- source — _bmad-output/initiative-auratranslate/prd-auratranslate/prd-auratranslate.md, FR121, FR133, FR134
- design — _bmad-output/initiative-auratranslate/ux-auratranslate/mockups/export-share.html, §2b
- design — _bmad-output/initiative-auratranslate/ux-auratranslate/EXPERIENCE.md, §Yêu cầu mới đã vào PRD
- sibling — _bmad-output/initiative-auratranslate/epic-cau-noi-reviewer/story-xuat-docx-bang-hai-cot-theo-segment-plan.md
- sibling — _bmad-output/initiative-auratranslate/epic-cau-noi-reviewer/story-chon-cach-xuat-hinh-anh-plan.md
- debt — _bmad-output/initiative-auratranslate/deferred-work.md, §nghĩa vụ FR133 chỉ phát biểu MỘT CHIỀU và §AC4 vế "đường xuất đọc cả hai nguồn"

## Notes

- Decision (2026-10-07, Ice, epic Notes): vế "hai ô giữ đúng số lần xuống đoạn như nhau" hết đúng; tiêu chí theo AD-46, nghiệm thu chỉ đọc cột phải.
- Decision (2026-10-08, Ice): ký tự xuống dòng trong bản dịch thành đoạn riêng, không thành ngắt dòng mềm như 8.3; dòng trống gộp lại, không có đoạn rỗng.
- Decision (2026-10-08, Ice): ảnh là một đoạn tại vị trí neo ở cả hai ô, theo cách chọn của 8.5.
- Decision (2026-10-08, Ice): bỏ mọi đường kẻ, kể cả đường dọc giữa hai cột.
- Source conflict: mockup §2b — bảng mẫu vẽ viền ngoài và chỉ bỏ đường giữa vs Decision "bỏ mọi đường kẻ" ở trên; theo Decision, không theo mockup.
- Decision (2026-10-08, Ice): 8.4 mang tiêu chí FR133 (tiêu chí 6), trả lời câu hỏi chờ Ice trong sổ nợ cho phần của 8.4.
- Decision (2026-10-08, Ice): cột trái nối câu không ký tự chen giữa vì nguyên văn là tiếng Trung (mockup §2b); cột phải nối bằng một dấu cách.
- Decision (2026-10-08, Ice): câu chưa dịch không để lại gì ở cột phải; Chương chưa dịch câu nào vẫn có bảng với ô phải trống; Chương không còn câu chỉ có tiêu đề.
- Decision (2026-10-08, Ice): cảnh báo trước lúc xuất nêu riêng số câu chưa dịch, vì câu vắng khỏi bài đăng mà không báo là lỗi rỗng im lặng.
- Decision (2026-10-08, Ice): hậu tố tên tệp là `-mot-khoi`, theo `-hai-cot` của 8.3.
- Decision (2026-10-08, Ice): `after` gồm 8.3 và 8.5 vì 8.4 dùng lại cách ghi ảnh, cách chọn ảnh và cách đặt tên tệp của hai story đó.
- Assumption: phía nguồn chưa có phép gom đoạn tương ứng với phép gom đoạn bản dịch (dời cờ của câu bị lược về câu liền trước); 8.4 dựng nó. Mã mới vào tệp `.rs` riêng và nâng sàn quần thể cùng lượt, theo epic Notes 2026-10-07.
- Open question: nguyên văn có thể chứa ký tự xuống dòng không; nếu có, cột trái có tách đoạn tại đó như cột phải không.
- Decision (2026-10-08, Ice): khi xong, ghi phần của 8.4 vào hai mục sổ nợ trong References, và ghi phép dán thật sang trình soạn thảo website thành mục nợ `Chủ: Epic 8`.
