---
type: epic
title: "Glossary — chốt thuật ngữ một lần, dùng mãi"
parent: initiative-auratranslate
covers: ["FR46", "FR47", "FR48", "FR49", "FR50", "FR51", "FR52", "FR53", "FR55", "FR113", "FR114"]
after: []
assignee: ""
risk: low
status: in-progress
---

# Glossary — chốt thuật ngữ một lần, dùng mãi

## Description

Người dịch bôi đen một cụm từ ở bất kỳ panel nào và thêm vào Glossary mà **không rời màn hình đang làm việc**; thuật ngữ đã chốt hiện ngay dấu trực quan ở cột nguyên văn của lưới. Sau một lần nhập lớn, hàng trăm ứng viên do máy quét ra hiện thành bảng chờ xếp theo tần suất, **duyệt bằng một phím mỗi mục, không phải gõ chữ nào** — ứng viên tiếng Trung còn kèm sẵn bản dịch âm Hán Việt, chạy hoàn toàn ngoại tuyến. Glossary xuất/nhập round-trip qua CSV/TSV để chia sẻ trong cộng đồng.

## Outcome

Người dịch chốt một thuật ngữ một lần và thấy nó được dùng lại ở mọi nơi sau đó, không bao giờ có mục nào tự chui vào Glossary; tín hiệu là mọi mục Glossary đều do người dùng duyệt và bảng chờ ứng viên duyệt được bằng một phím mỗi mục.

## Requirements

- FR46: Glossary hai tầng (epics.md, FR Coverage Map)
- FR47: Trường của một mục Glossary (epics.md, FR Coverage Map)
- FR48: Thêm nhanh từ bất kỳ panel nào (epics.md, FR Coverage Map)
- FR49: Quản lý + xuất/nhập CSV/TSV — quản lý: Story 3.9 · định dạng + đường ghi: Story 3.10 · hộp thoại chọn tệp: Story 3.10b (epics.md, FR Coverage Map)
- FR50: Đánh dấu thuật ngữ ở cột nguyên văn của lưới — khớp + bề mặt IPC: Story 3.4 · vẽ dấu + `StatusBar`: Story 3.4b (epics.md, FR Coverage Map)
- FR51: Khớp thuật ngữ theo ngôn ngữ (epics.md, FR Coverage Map)
- FR52: Quét ứng viên khi nhập tài liệu (epics.md, FR Coverage Map)
- FR53: Duyệt hàng loạt một phím (epics.md, FR Coverage Map)
- FR55: Không cơ chế nào tự ghi vào Glossary (epics.md, FR Coverage Map)
- FR113: Đề xuất bản dịch bằng âm Hán Việt (epics.md, FR Coverage Map)
- FR114: Trạng thái chờ chốt bản dịch (epics.md, FR Coverage Map)

## Done when

1. Mọi mục Glossary đến từ hành động của người dùng: không đường nào, kể cả quét ứng viên và đề xuất Hán Việt, ghi thẳng vào Glossary (FR55).
2. Thuật ngữ đã chốt ở tầng Tác phẩm đè tầng Global, phân giải qua một đường duy nhất, và mục chờ chốt không bao giờ được chèn (FR46, FR47, FR114).
3. Quét ứng viên sau khi nhập cho ra bảng chờ xếp theo tần suất, duyệt hàng loạt bằng một phím mỗi mục, ứng viên tiếng Trung có sẵn bản dịch âm Hán Việt ngoại tuyến (FR52, FR53, FR113).
4. Thêm nhanh từ bất kỳ panel nào mà không rời màn hình đang làm việc, và thuật ngữ đã chốt hiện dấu ở cột nguyên văn của lưới (FR48, FR50, FR51).
5. Quản lý Glossary và xuất rồi nhập lại qua CSV/TSV cho ra đúng tập mục ban đầu, kể cả khi chọn tệp bằng hộp thoại hệ điều hành (FR49).

## Boundaries

Epic theo năng lực người dùng thấy được. Phần nghiệm thu chia đôi với epic khác được nêu ở Notes.

## References

- parent — ../initiative-auratranslate.md, FR Coverage Map của PRD và `epics.md`
- retrospective — epic-glossary-retrospective.md

## Notes

- Retrospective: `epic-glossary-retrospective.md` (verdict ở frontmatter của tệp đó).
