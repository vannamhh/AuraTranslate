---
id: 23
type: story
title: "Màn xem trước nhập: nút Nhập luôn trong tầm tay, cột cấu hình dính, khối loại gộp thành dải"
parent: epic-duong-nhap
covers: ["FR14", "FR122", "FR123", "FR126", "FR128", "FR132"]
after: []
hitl: false
risk: medium
---

# Màn xem trước nhập: nút Nhập luôn trong tầm tay, cột cấu hình dính, khối loại gộp thành dải

## Description

Nhập một bài báo epochtimes.com, người dùng phải cuộn qua 301 khối mới tới nút xác nhận (mockup đo 18 955 px ở 1280×860). Story này dựng bố cục Ice chọn ngày 2026-11-02: cột cấu hình dính bên trái, cột văn bản bên phải, chân dính mang nút Nhập. Đường URL thôi bị ép một cột.

Khối loại liền nhau gộp thành một dải mở được. Bảng mã tin cậy cao thu còn một dòng. Lượt một Chương ẩn khối Tách Chương. Xuất xứ in một tiêu đề, và câu "chưa ghi gì xuống đĩa" chỉ còn ở chân.

Ô mẫu phân tách Chương chỉ hiện khi đầu vào chưa chia Chương (AD-39). Rust gửi một cờ hình dạng trong khối tách Chương, webview đọc cờ đó (cách A, Ice chọn 2026-11-02).

Văn bản của tầng luật làm sạch và chỗ gạch ngang tại chỗ thuộc story 24, không thuộc story này.

## Acceptance Criteria

Verify: Một spec e2e và các tệp vitest của màn xem trước, trên một lượt nhập URL có ≥ 200 khối loại và một lượt nhập `.txt` nhiều Chương.

1. **Chân dính.** Ở mọi vị trí cuộn, chân hiện `Huỷ` và `Nhập N Chương ⌥⌘↵` trong khung. Chân ghi số Chương sẽ tạo, số ký tự, ước lượng ứng viên Glossary, trạng thái ban đầu, "chưa ghi gì xuống đĩa" và dòng nhật ký domain. `⌥⌘↵` xác nhận giống bấm nút. Khi nào nút tắt thì giữ nguyên như hôm nay.
2. **Hai cột cho mọi đường nhập.** URL, tệp và dán tay đều có cột cấu hình bên trái, cột văn bản bên phải. Cột cấu hình đứng yên khi cột văn bản cuộn. Cột cấu hình dài hơn khung thì tự cuộn riêng.
3. **Tên Tác phẩm cao theo nội dung.** Tên dài n dòng thì ô hiện đủ n dòng, không có thanh cuộn. Gõ thêm thì ô giãn theo.
4. **Khối loại gộp thành dải.** Mỗi dãy khối loại liền nhau hiện thành một dải. Dải ghi số khối và vài chữ đầu. Mở dải thì từng khối hiện với `Giữ khối này`. Khối giữ luôn hiện đủ. Giữ một khối trong dải thì dải tách quanh khối đó và số trên các dải đúng.
5. **Bàn phím giữ nghĩa.** `J`/`K` đi qua một dải như một khối. `Space` trên dải mở hoặc thu dải, trên khối thì đổi giữ/loại như hôm nay. `[` `]`, `⌥←` `⌥→`, `⌥W`, `E` chạy như trước khi có dải.
6. **Bảng mã một dòng khi tin cậy cao.** Tin cậy cao thì cột cấu hình hiện "tên bảng mã · tin cậy cao · Đổi E". Bấm `Đổi` hoặc `E` thì dải năm ứng viên mở. Tin cậy thấp thì dải năm ứng viên mở sẵn, như FR126.
7. **Tách Chương theo số Chương.** Lượt có một Chương không hiện khối Tách Chương, bộ đếm cần xem/sạch, hay `⌥W`. Từ hai Chương, con trỏ Chương, bộ đếm hai con số và `⌥W` đứng ở đầu cột văn bản (FR132).
8. **Ô mẫu phân tách theo hình dạng đầu vào.** Đầu vào đã chia Chương (danh sách link) không có ô mẫu phân tách. Đầu vào chưa chia Chương (`.txt`, `.md`, `.docx`, dán tay, nhiều tệp) có ô, và gõ mẫu thì số Chương đổi như hôm nay. Webview quyết theo cờ Rust gửi, không theo tên đường nhập.
9. **Một tiêu đề, một câu.** "Xuất xứ tài liệu" in một lần. "Chưa ghi gì xuống đĩa" chỉ in ở chân.

## Boundaries

- Must not change: thứ tự pipeline AD-39 và việc không ghi gì xuống đĩa trước khi xác nhận.
- Must not change: kết quả nhập. Cùng đầu vào, cùng thao tác sửa, Tác phẩm tạo ra giống hệt trước story này.
- Must not change: bốn trường xuất xứ, nhật ký domain, danh sách link với `Bỏ`/tải lại, chọn Tác phẩm đã có.
- Touch point: màn xem trước song ngữ (FR115) đọc chung khối tách Chương; nó giữ bố cục hôm nay, chỉ nhận cờ mới.

## References

- parent — _bmad-output/initiative-auratranslate/tickets/epic-duong-nhap/epic-duong-nhap.md, Requirements
- design — _bmad-output/initiative-auratranslate/ux-auratranslate/EXPERIENCE.md, §Bố cục: cột cấu hình, cột văn bản, chân dính
- design — _bmad-output/initiative-auratranslate/ux-auratranslate/mockups/import-preview-gon.html, Phương án 1
- design — _bmad-output/initiative-auratranslate/ux-auratranslate/EXPERIENCE.md, đoạn "Sửa ranh giới bóc — bàn phím là đường chính" (bảng phím)
- architecture — _bmad-output/initiative-auratranslate/architecture-auratranslate/architecture-auratranslate.md, AD-39 (bảng tiêu chí hình dạng)

## Notes

- Decision: Phương án 1 (hai cột) thay vì một cột thu gọn; tên Tác phẩm cao theo nội dung (Ice, 2026-11-02).
- Decision: cờ hình dạng gửi từ Rust, không so tên đường nhập ở webview (Ice, 2026-11-02, cách A của Winston).
- Decision: một khối loại đứng riêng giữa hai khối giữ cũng thành dải một khối, để luật gộp chỉ có một nhánh (Ice, 2026-11-02).
- Decision: story đặt ở Epic 6, không mở lại Epic 12 đã xong; văn bản một lần và gạch ngang tại chỗ tách sang story 24 vì có thể cần Rust ánh xạ chỗ khớp luật vào khối (Ice, 2026-11-02).
- Open question: dải hiện "vài chữ đầu" của bao nhiêu khối. Build đề xuất số và đưa Ice xem trên bài thật.
- Bàn đo thật: nhiều tệp vitest `importPreview*` và hai spec e2e (`story-12-6-import-preview`, `import-preview-mouse`) đọc cấu trúc màn này; build đếm và sửa chúng ở pha Tests.
- Kiểm ngoài tiêu chí (risk medium): đối chứng đỏ ở criterion 8 phải là phép gỡ cờ thật ở Rust, ca e2e đỏ vì ô mẫu hiện lại trên đường URL.
- Kiểm ngoài tiêu chí: Ice nhập lại đúng bài epochtimes.com trong app thật và nút Nhập hiện mà không phải cuộn.
