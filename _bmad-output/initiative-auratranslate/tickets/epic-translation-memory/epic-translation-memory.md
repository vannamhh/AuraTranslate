---
type: epic
title: "Translation Memory — không dịch lại, không tra lại thứ đã dịch"
parent: initiative-auratranslate
covers: [FR44, FR56, FR57, FR58, FR59, FR60, FR61, FR62, FR63, FR64, FR70, FR118, FR129]
after: []
assignee: ""
status: in-progress
risk: low
---

# Translation Memory — không dịch lại, không tra lại thứ đã dịch

## Description

Mỗi lần người dịch xác nhận một segment, cặp *(nguồn → đích)* **tự vào Translation Memory, không một thao tác thủ công nào**. Từ đó về sau: câu y hệt được **điền sẵn nhưng vẫn ở trạng thái chưa xác nhận**; câu tương tự hiện kèm phần trăm khớp và diff phần khác biệt; và Concordance trả lời *"cụm này trước đây tôi dịch thế nào?"* ngay trong Panel Lookup. TM xuất được TMX mở ở CAT tool khác. Và vì chủ dự án làm **cả hai vai**, mỗi cặp TM mang **xuất xứ**, và Smart RAG Injector **ưu tiên cặp của chính người dùng**.

## Outcome

Người dịch không phải dịch lại hay tra lại điều đã dịch: cặp nguồn-đích tự vào TM, câu y hệt hoặc tương tự được gợi lại, và AI học văn phong của chính người dùng.

## Requirements

- FR44: Alt-text là Segment vai `alt` (AD-42). *Phần cấu trúc ở 6.13; phần nghiệm thu TM ở 7.1* (epics.md, FR Coverage Map)
- FR56: **Ghi tự động:** mỗi khi người dùng xác nhận một segment, cặp *(nguồn → đích)* được ghi vào TM. **Không có thao tác thủ công nào.** Cặp TM mang **xuất xứ** kế thừa từ segment (FR117).
- FR57: TM có **phạm vi kép**: TM riêng theo Tác phẩm và TM chung toàn cục.
- FR58: **Khớp tuyệt đối (100%):** segment y hệt đã dịch trước đây được **điền sẵn** và **đánh dấu là gợi ý cần xác nhận**. Hệ thống **không** tự coi segment đó là đã hoàn thành.
- FR59: **Khớp mờ:** hiển thị các bản dịch cũ tương tự kèm **phần trăm khớp** và **diff phần khác biệt**.
- FR60: **Concordance:** tra ngược *"cụm từ này trước đây tôi dịch thế nào?"* trên toàn bộ TM. Kết quả đưa vào **Panel Lookup**, cùng chỗ với kết quả từ điển.
- FR61: Thuật toán khớp **phân theo ngôn ngữ**: tiếng Trung dùng n-gram ký tự; tiếng Anh dùng token n-gram sau stemming.
- FR62: Xem, sửa và xoá từng mục TM. Danh sách hiển thị **xuất xứ** của từng cặp và **lọc được theo xuất xứ**.
- FR63: Khi cùng một segment nguồn có **nhiều bản dịch khác nhau**, hệ thống **giữ lại tất cả** và hiển thị tất cả kèm ngày, thay vì ghi đè.
- FR64: **Xuất và nhập TMX.**
- FR70: Smart RAG Injector — nửa TM đóng ở Epic 7 (epics.md, FR Coverage Map: FR70 Epic 4 ⇄ Epic 7; Story 7.11)
- FR118: **Translation Memory không được trộn phong cách.** Mỗi cặp mang xuất xứ *của tôi* hoặc *của người khác*; **Smart RAG Injector ưu tiên cặp *của tôi***, cặp xuất xứ khác chỉ chèn khi không đủ và phải **đánh dấu rõ trong prompt là văn phong tham khảo**.
- FR129: Caption là Segment vai `caption` (AD-42). *Phần cấu trúc ở 6.13; phần nghiệm thu TM ở 7.1* (epics.md, FR Coverage Map)

## Done when

1. Xác nhận một segment tự ghi cặp (nguồn → đích) vào TM của Tác phẩm, khoá theo cặp văn bản, không thao tác thủ công.
2. Câu y hệt được điền sẵn nhưng vẫn ở trạng thái chưa xác nhận; câu tương tự hiện kèm phần trăm khớp và diff; Concordance trả lời trong Panel Lookup.
3. Mỗi cặp TM mang xuất xứ, và Smart RAG Injector ưu tiên cặp của chính người dùng, đánh dấu rõ cặp xuất xứ khác là văn phong tham khảo.
4. Quản lý TM (phạm vi Tác phẩm và toàn cục) và xuất/nhập TMX hoạt động trên dữ liệu thật.
5. Nửa TM của FR44, FR70 và FR129 được nghiệm thu ở epic này.

## Boundaries

Translation Memory và phần TM của Smart RAG. Không phải đường nhập cấu trúc (epic Đường nhập) và không phải gọi AI (epic AI mở).

## References

- parent — _bmad-output/initiative-auratranslate/prd-auratranslate/prd-auratranslate.md, functional requirements FR56, FR57, FR58, FR59, FR60, FR61, FR62, FR63, FR64, FR118
- spec — _bmad-output/initiative-auratranslate/spec-auratranslate/spec-auratranslate.md
- constraint — _bmad-output/initiative-auratranslate/architecture-auratranslate/architecture-auratranslate.md, các AD nêu trong tiêu chí của story
- change — _bmad-output/initiative-auratranslate/change-thu-tu-thuc-thi-epic/change-thu-tu-thuc-thi-epic.md, thứ tự thực thi epic

## Notes

- Parked: FR44 và FR129 chia nghiệm thu với epic Đường nhập (Story 6.13 nghiệm thu phần cấu trúc, phần TM đóng ở Story 7.1); FR70 chia với epic AI mở.
- Decision: Epic 7 chạy sau epic Trả nợ nền (thứ tự thực thi do Ice chốt 2026-10-06).
- Retrospective: epic-translation-memory-retrospective.md
