---
type: epic
title: "Đường nhập — mọi nguồn văn bản vào được, và không hỏng im lặng"
parent: initiative-auratranslate
covers: ["FR13", "FR14", "FR42", "FR43", "FR44", "FR45", "FR115", "FR116", "FR122", "FR123", "FR124", "FR125", "FR126", "FR127", "FR128", "FR129", "FR132"]
after: []
assignee: ""
risk: low
status: in-progress
---

# Đường nhập — mọi nguồn văn bản vào được, và không hỏng im lặng

## Description

Đây là **bề mặt đầu tiên người dùng chạm vào sản phẩm**, và là nơi hai lỗi đắt nhất của ứng dụng có thể xảy ra mà không báo gì cả. Người dịch nhập một bộ 2000 chương từ một file `.txt` 40 MB, hoặc **dán 50 link web**, hoặc một file song ngữ hai cột do người khác dịch — tất cả đi qua **một màn xem trước hợp nhất** cho thấy bảng mã đã đoán, ranh giới nội dung đã bóc, và những gì luật làm sạch **sắp xoá** — trước khi một byte nào ghi xuống đĩa. Ảnh trong bài web tải về nằm trong `.atproj`; alt-text và caption là **hai segment dịch được riêng biệt**. Và ứng dụng **không bao giờ tự quyết định tải cái gì**.

## Outcome

Người dịch đưa văn bản từ mọi nguồn vào Tác phẩm qua một màn xem trước hợp nhất, và không có lần nhập nào hỏng mà không báo; tín hiệu là mọi nguồn đi qua cùng một pipeline và mọi thứ sắp xoá hay sắp tải đều hiện ra trước khi ghi.

## Requirements

- FR13: Đường vào văn bản tối thiểu (dán tay + `.txt`/`.md`); nhánh `.docx` đóng ở Epic 6 (epics.md, FR Coverage Map)
- FR14: Nhập hàng loạt + mẫu phân tách + xem trước (epics.md, FR Coverage Map)
- FR42: Ảnh ở cột nguyên văn của lưới — cần `ASSET` từ đường nhập (epics.md, FR Coverage Map)
- FR43: Ảnh trong Chế độ đọc — cần `ASSET` (epics.md, FR Coverage Map)
- FR44: Alt-text là Segment vai `alt` (AD-42). *Phần cấu trúc ở 6.13; phần nghiệm thu TM ở 7.1* (epics.md, FR Coverage Map)
- FR45: Ảnh lưu trong `.atproj/assets/` (epics.md, FR Coverage Map)
- FR115: Nhập tài liệu song ngữ hai cột (epics.md, FR Coverage Map)
- FR116: Khớp câu trong từng cặp hàng (epics.md, FR Coverage Map)
- FR122: Nhập từ URL bằng danh sách link (epics.md, FR Coverage Map)
- FR123: Bóc nội dung + xem trước + sửa tay (epics.md, FR Coverage Map)
- FR124: Luật làm sạch lộ ra, duyệt trước khi xoá (epics.md, FR Coverage Map)
- FR125: Chuẩn hoá xuống dòng và khoảng trắng (epics.md, FR Coverage Map)
- FR126: Phát hiện và sửa bảng mã (epics.md, FR Coverage Map)
- FR127: Ảnh web tải về `.atproj`, giữ URL gốc (epics.md, FR Coverage Map)
- FR128: Xuất xứ tài liệu ở tầng Chương (epics.md, FR Coverage Map)
- FR129: Caption là Segment vai `caption` (AD-42). *Phần cấu trúc ở 6.13; phần nghiệm thu TM ở 7.1* (epics.md, FR Coverage Map)
- FR132: Bộ lọc "cần xem" trên màn xem trước — *N cần xem · M sạch*. Nền (đường dữ liệu theo từng Chương): Story 6.10a · bộ lọc: Story 6.10 (epics.md, FR Coverage Map)

## Done when

1. Văn bản vào từ dán tay, `.txt`, `.docx`, danh sách link web và bảng song ngữ đều đi qua một pipeline một thứ tự cố định và một màn xem trước hợp nhất, trước khi ghi xuống đĩa (FR13, FR14, FR115, FR122).
2. Màn xem trước nêu bảng mã đã đoán, ranh giới nội dung đã bóc và thứ luật làm sạch sắp xoá, và người dùng sửa được từng thứ (FR123 đến FR126).
3. Ứng dụng chỉ tải đúng những link người dùng đã cho, qua allowlist hai tầng có nhật ký domain (NFR19).
4. Ảnh tải về nằm trong `.atproj`, hiển thị đúng vị trí, alt-text và caption là hai Segment dịch được riêng (FR42 đến FR45, FR127, FR129).
5. Xuất xứ tài liệu ghi ở tầng Chương, và bộ lọc "cần xem" chỉ ra các Chương cần soát, kể cả ở bản xem trước song ngữ (FR128, FR132, FR116).

## Boundaries

Epic theo năng lực người dùng thấy được. Phần nghiệm thu chia đôi với epic khác được nêu ở Notes.

## References

- parent — ../initiative-auratranslate.md, FR Coverage Map của PRD và `epics.md`
- retrospective — epic-duong-nhap-retrospective.md

## Notes

- Retrospective: `epic-duong-nhap-retrospective.md` (verdict ở frontmatter của tệp đó).
- Story 6.18 (entry 18): đã gộp vào Story 10.9 (Epic 10); AC đã ở 10.9. Plan giữ `in-progress` làm đầu vào của 10.9: task 1–6 xong, task 7–8 còn lại ở 10.9.
- FR13 (Epic 1 ⇄ Epic 6), FR44 và FR129 (Epic 6 ⇄ Epic 7): nghiệm thu chia đôi, phần chủ của epic này ở mục Requirements, phần còn lại ở epic kia.
