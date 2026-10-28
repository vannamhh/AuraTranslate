---
type: epic
title: "Library — kho tác phẩm, tìm kiếm, và đọc lại thành quả"
parent: initiative-auratranslate
covers: ["FR1", "FR2", "FR3", "FR4", "FR5", "FR6", "FR7", "FR8", "FR9", "FR10", "FR11", "FR12", "FR15", "FR98", "FR99", "FR119", "FR120"]
after: []
assignee: ""
risk: low
status: in-progress
---

# Library — kho tác phẩm, tìm kiếm, và đọc lại thành quả

## Description

Mở ứng dụng là vào Library, không phải vào màn hình dịch. Người dịch **nắm được mình đang có những gì**: lưới Tác phẩm với bìa, tiến độ và bốn trạng thái vòng đời; lọc theo trạng thái, lĩnh vực, ngôn ngữ nguồn, ngày sửa; **tìm full-text xuyên toàn thư viện phân biệt dấu**. Mở một Chương đưa thẳng vào Workspace **đúng câu đang dở lần trước**. Và Chế độ đọc để người dịch quay lại **thưởng thức thành quả**: đọc liên tục qua các Chương đã dịch, dừng ở biên tường minh tại Chương chưa bắt đầu, đánh dấu chỗ cần sửa bằng một phím rồi **đọc tiếp ngay**. Xoá chỉ mục Library rồi quét lại phục hồi đầy đủ, không mất một byte dữ liệu nào.

## Outcome

Người dịch mở ứng dụng là nắm được mình đang có gì, tìm lại được bất cứ câu nào đã dịch và đọc lại thành quả liên tục; tín hiệu là Library dựng lại đầy đủ từ thư mục Tác phẩm và ba ngưỡng NFR3, NFR4, NFR5 có số đo thật.

## Requirements

- FR1: Hai tầng Tác phẩm → Chương (epics.md, FR Coverage Map)
- FR2: Tài liệu đơn lẻ = Tác phẩm một Chương (epics.md, FR Coverage Map)
- FR3: Metadata Tác phẩm (epics.md, FR Coverage Map)
- FR4: Glossary/TM gắn tầng Tác phẩm (epics.md, FR Coverage Map)
- FR5: Bốn trạng thái vòng đời; Chương tự chuyển Chưa bắt đầu → Đang dịch ở lần ghi bản dịch đầu tiên do người dùng, Đã xong chỉ do người đặt, đặt được trong Workspace, có gợi ý khi câu cuối được xác nhận (PRD FR5 🔵 2026-10-27)
- FR6: Suy ra tự động + ghi đè tay (epics.md, FR Coverage Map)
- FR7: Tiến độ Tác phẩm (epics.md, FR Coverage Map)
- FR8: Full-text search xuyên Library (epics.md, FR Coverage Map)
- FR9: Hai chế độ dấu (epics.md, FR Coverage Map)
- FR10: Lọc và sắp xếp (epics.md, FR Coverage Map)
- FR11: Chế độ đọc (epics.md, FR Coverage Map)
- FR12: Mở Chương → Workspace đúng vị trí (epics.md, FR Coverage Map)
- FR15: Đổi tên, sắp xếp, gộp/tách Chương (AD-32) (epics.md, FR Coverage Map)
- FR98: Chỉ mục Library dựng lại được (epics.md, FR Coverage Map)
- FR99: Quét lại thư mục, mục mồ côi (epics.md, FR Coverage Map)
- FR119: Đánh dấu chỗ cần sửa khi đang đọc (epics.md, FR Coverage Map)
- FR120: Chỉ đọc phần đã dịch (Chương Đã xong, Đang dịch, Tạm ngưng), dừng ở biên tại Chương Chưa bắt đầu, câu chưa dịch thành một dòng mờ ghi số câu (PRD FR120 🔵 2026-10-27)

## Done when

1. Mở ứng dụng vào Library: lưới Tác phẩm hiện bìa, tiến độ và bốn trạng thái vòng đời, lọc và sắp xếp được (FR1, FR5, FR6, FR7, FR10).
2. Xoá chỉ mục Library rồi quét lại phục hồi đầy đủ, không mất dữ liệu; thư mục `.atproj` thả vào được nhận (FR98, FR99).
3. Mở một Chương đưa thẳng vào Workspace đúng câu đang dở, và đổi tên, sắp xếp, gộp, tách Chương giữ nguyên mọi tham chiếu (FR12, FR15).
4. Tìm full-text xuyên Library trên cả nguyên văn lẫn bản dịch, có hai chế độ dấu (FR8, FR9).
5. Chế độ đọc đọc liên tục qua các Chương đã dịch, dừng ở biên tường minh tại Chương chưa bắt đầu, đánh dấu chỗ cần sửa bằng một phím và đọc tiếp (FR11, FR119, FR120).
6. NFR3, NFR4, NFR5 có số đo thật kèm quy mô thư viện, và trạng thái ba ngưỡng tạm được ghi lại.

## Boundaries

Epic theo năng lực người dùng thấy được. Phần nghiệm thu chia đôi với epic khác được nêu ở Notes.

## References

- parent — ../initiative-auratranslate.md, FR Coverage Map của PRD và `epics.md`
- retrospective — epic-library-retrospective.md

## Notes

- Retrospective: `epic-library-retrospective.md` (verdict ở frontmatter của tệp đó).
- Story 5.14 là `done` theo `sprint-status`; bản ghi v6 ghi `review`, plan lấy theo sprint (Q12 của kế hoạch di trú).
- Decision: 2026-10-27, Ice — trạng thái Chương theo phương án A: Workspace có nút đặt Chương đang mở thành Đã xong; xác nhận câu cuối chưa xác nhận (không tính `is_omitted`) chỉ hiện gợi ý, không tự đặt Done; lần ghi bản dịch đầu tiên do người dùng tự chuyển Chưa bắt đầu → Đang dịch; một migration `project.db` sửa một lần Chương cũ đã có bản dịch mà vẫn ghi Chưa bắt đầu; gỡ nút "Đặt Chương này là Đã xong" ở trang Tác phẩm Library vì thanh hành động Đổi trạng thái đã phủ. Lý do: gõ, xác nhận và dịch AI không đổi trạng thái Chương nên tiến độ FR7 đứng 0 và Chế độ đọc không có gì để đọc; món nợ "Workspace không có nút đổi trạng thái Chương" chỉ ghi chủ Story 5.7 trong chú thích hai e2e, chưa từng vào `deferred-work.md`, nên không ai theo dõi khi 5.7 đóng. Thành entry 18.
- Decision: 2026-10-27, Ice — FR120 mở phạm vi: Chế độ đọc đọc Chương Đã xong, Đang dịch và Tạm ngưng, dừng tại Chương Chưa bắt đầu đầu tiên; mỗi dãy câu chưa dịch liền nhau là một dòng mờ ghi số câu dẫn sang Workspace câu đầu dãy, không hiện nguyên văn; dấu câu chưa xác nhận giữ ở mọi Chương được đọc. PRD FR120 sửa kèm 🔵. Thành entry 19, sau entry 18 vì thiếu bước tự chuyển Đang dịch thì không Chương nào được đọc thêm.
- Assumption: điền sẵn TM khớp 100% (FR58) không tính là lần ghi bản dịch đầu tiên — nó chạy ngay khi mở Chương (`commands/segment/chapter_read.rs`), tính nó thì mở Chương là đổi trạng thái. 🔵 2026-10-28, Ice: xác nhận câu thì tính, kể cả câu có chữ do TM điền sẵn (PRD FR5).
- Source conflict: `ux-auratranslate/EXPERIENCE.md` KF-2 bước 7 ghi "Hết Chương, trạng thái Chương tự chuyển Đã xong", trái PRD FR5/FR6 và quyết định 2026-10-27; đoạn §Phạm vi và thao tác — FR119, FR120 còn ghi chỉ đọc Chương Đã xong. Sửa qua `bmad-ux` (Sally); entry 18 và 19 theo PRD.
