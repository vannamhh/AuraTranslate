---
type: epic
title: "Hoàn thiện giao diện theo mockup"
parent: initiative-auratranslate
covers: [FR136, FR137, FR138, FR139, FR140, FR141, FR142, FR143, FR144, FR145, FR146, FR147, FR148, FR149, FR150, FR151, FR152]
after: []
assignee: ""
risk: high
---

# Hoàn thiện giao diện theo mockup

## Description

App thật làm đúng các FR của Epic 1–11 nhưng lệch xa mockup ở bố cục: thanh tiêu đề dồn 3 tab chế độ cùng 10 nút lối vào, Library xếp mọi khối thành một cột 420 px, các lớp phủ quản lý là flex-wrap thay vì bảng. Epic này sửa hình dạng mọi bề mặt cho giống mockup hết mức có thể, và dựng các chức năng mockup có mà app chưa có (FR136–FR152). Quyết định và phân loại từng dòng nằm ở `change-hoan-thien-giao-dien-theo-mockup.md`; epic này không chép lại.

## Outcome

Người dịch mở app và thấy đúng bố cục mockup — ba tab, Library dải trên + lưới, lưới hai cột với dải gợi ý mọc dưới câu đang sửa — mà không mất lối vào nào; tín hiệu là bảng lệch không còn dòng loại 2 hay loại 5 nào không có đích.

## Requirements

FR136–FR152 ở `prd-auratranslate.md` §6 (thêm 2026-10-13). Dòng lệch loại 2 và loại 5 "làm" ở `bang-lech-{a,b,c}.md` và `phan-loai-loai-5-{a,b,c}.md` là nguồn của từng story; mỗi entry trong `tickets.toml` trích mã dòng của nó: `a:16` là dòng 16 của `bang-lech-a.md` (loại 2), `A10` là hàng `#` 10 của `phan-loai-loai-5-a.md` (loại 5).

## Done when

1. Thanh tiêu đề chỉ còn 3 tab, công tắc bố cục, `Công cụ ▾` và trạng thái Proofreader khi có; mỗi lối vào đã rời thanh tiêu đề mở được từ chỗ §Information Architecture ghi và trả tiêu điểm về nút mở khi đóng (UX-DR17).
2. Mỗi dòng loại 2 của `bang-lech-{a,b,c}.md` đóng bằng một story của epic này hoặc chuyển loại 3 có nguồn; mỗi dòng loại 5 có đúng một đích (đề xuất 2).
3. FR136–FR152 mỗi FR chạy được trong app thật; FR141, FR147, FR149, FR150 chỉ vào build sau khi AD của cửa G2, G1, G4, G3 có trong spine.
4. Thời gian frame khi dải gợi ý mọc/thu trên Chương ≥ 2.000 câu được đo và ghi số (đề xuất 5).
5. Ice chạy một lượt dùng thật trên bản đóng gói macOS và mọi mục `Chủ: Epic 12` trong `deferred-work.md` có dạng đóng trước khi epic lên `done`.

## Boundaries

Hình dạng bề mặt webview và các chức năng loại 5 Ice nhận (D7). Không mở lại Epic 1–9, 11: FR của chúng giữ nghĩa. Không: các cụm loại 3 của đề xuất 2; nội dung mục Cài đặt "Cập nhật" (Story 10.7, epic này chỉ dựng khung); menu gốc hệ điều hành (D10); phím riêng cho từng nguồn từ điển (đề xuất 1).

## References

- parent — _bmad-output/initiative-auratranslate/prd-auratranslate/prd-auratranslate.md, FR136–FR152
- change — _bmad-output/initiative-auratranslate/change-hoan-thien-giao-dien-theo-mockup/change-hoan-thien-giao-dien-theo-mockup.md, đề xuất 1–7
- decisions — _bmad-output/initiative-auratranslate/change-hoan-thien-giao-dien-theo-mockup/handoff-buoc-1-bang-lech.md, D1–D13
- rows — _bmad-output/initiative-auratranslate/change-hoan-thien-giao-dien-theo-mockup/bang-lech-{a,b,c}.md, phan-loai-loai-5-{a,b,c}.md; ảnh app thật ở `anh/`
- design — _bmad-output/initiative-auratranslate/ux-auratranslate/EXPERIENCE.md §Information Architecture, §Component Patterns, §Bề mặt nhập; DESIGN.md §Vạch lề segment; mockups/
- constraint — _bmad-output/initiative-auratranslate/architecture-auratranslate/architecture-auratranslate.md, AD-20, AD-34, AD-36, AD-47, AD-49, AD-53, AD-54, AD-55, AD-56, AD-57; UX-DR17

## Notes

- Decision: một epic gom cả loại 2 và loại 5 Ice nhận, đứng trước Epic 10; 10.6 và 10.9 chờ epic này (Ice duyệt đề xuất 7, 2026-10-13).
- Decision: Story 1 (khung app) đi trước mọi story khác (đề xuất 7).
- Decision: giữ nguyên khung 20 story; 14, 15, 19, 20 không tách phần không cần cửa, chờ trọn AD G1–G4 (Ice, 2026-10-13).
- Decision: ba dòng Attribution (b:99, b:103, B11) nằm ngoài khung 20, thành entry 21 riêng (Ice, 2026-10-13).
- Decision: giữ một epic dù 22 entry, quá khoảng 8–12 thường gặp (Ice, 2026-10-13).
- Decision: entry 1 là tracer bullet. Chuỗi `after` 14→15→19→20 là thứ tự đánh số migration trong `PROJECT_MIGRATIONS`, không phải phụ thuộc chức năng.
- Decision: entry 21 chờ Story 10.4 (đang review), dù Epic 12 đứng trước Epic 10: 21 sửa chính `AttributionOverlay.vue` mà 10.4 dựng. Hai epic khai `after` lẫn nhau trong `tickets.toml` của initiative, chỉ ở mức entry (12.21 → 10.4; 10.6, 10.9 → 12.22; 10.7 → 12.18), nên không có vòng; `order_conflict` mà `tickets.py` báo là cố ý (Ice duyệt 2026-10-13).
- Waits on Epic 10 because: 12.21 sửa `AttributionOverlay.vue` mà Story 10.4 dựng.
- Source conflict: `bang-lech.md` ghi 118 dòng loại 2 (A 47) và đề xuất 2 ghi "khoảng 55" dòng loại 5 vào epic; đếm lại cột Loại cho 122 (A 51) và 63 dòng loại 5 "làm" (tính cả dòng làm một phần). Entry trích dòng theo số dòng tệp, không theo số đếm.
- Open question: B22 (đưa cả lô sang), A10 (dấu thời gian chỉ mục), A19 (mở thư mục Tác phẩm) mỗi chỗ hai phương án hợp lệ; story đo rồi đưa Ice chọn.
- Decision: AD-54 (G1, FR147), AD-55 (G2, FR141), AD-56 (G3, FR150), AD-57 (G4, FR149) vào spine; Ice ký 2026-10-13. Entry 14, 15, 19, 20 hết `unknown`.
- Decision: nhãn từ AI · từ TM · từ bản review của A17 thành lệch có chủ đích; entry 19 dựng ba giá trị xuất xứ cộng không rõ (AD-55, Ice duyệt 2026-10-13).
- Decision: Xuất nằm tạm trong `Công cụ ▾` từ story 1 tới story 4, vì `export.open` không có phím và thanh hành động danh sách Chương chưa có; story 4 gỡ mục đó (Ice, 2026-10-13).
- Decision: nhãn chế độ dấu là `Chính xác dấu | Không dấu` theo `library-and-import.html`, không phải `Khoan dung không dấu` của `library-search-results.html`; story 2 và story 5 dùng cùng nhãn (Ice, 2026-10-13).
- Open question: Bảng chờ Glossary và Review Mode không còn lối mở nhìn thấy sau story 1, chỉ còn `Mod+Alt+Q` và `Mod+Alt+3`. IA ghi chỗ của chúng là "Workspace", nhưng không entry nào đặt nút ở đó. Done when 1 cần một story nhận việc này.
- Unknown: chi phí track hàng chèn vào lưới `subgrid` năm cột (`src/panels/GridPanel.vue`) trên Chương lớn chưa đo.
