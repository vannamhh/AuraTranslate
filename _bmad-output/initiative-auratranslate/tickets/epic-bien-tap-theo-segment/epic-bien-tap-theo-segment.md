---
type: epic
title: "Biên tập theo segment — một vòng dịch tay hoàn chỉnh"
parent: initiative-auratranslate
covers: ["FR16", "FR19", "FR21", "FR23", "FR24", "FR25", "FR26", "FR78", "FR100", "FR101", "FR117", "FR133", "FR134"]
after: []
assignee: ""
status: in-progress
risk: medium
---

# Biên tập theo segment — một vòng dịch tay hoàn chỉnh

## Description

Người dịch dịch trọn một Chương **bằng tay, không cần AI và không cần Glossary**: văn bản tách thành segment cấp câu, gộp hoặc tách khi máy tách sai, xác nhận từng câu với vạch lề đổi màu, điều hướng tới segment chưa dịch kế tiếp, chuyển Chương ngay trong Workspace. Sập ứng dụng giữa phiên gõ **mất tối đa 5 giây công việc**, và không frame nào vượt 50 ms trong lúc auto-save chạy. Mọi phiên bản cũ của một segment xem lại và khôi phục được.

## Outcome

Người dịch dịch trọn một Chương bằng tay mà không mất quá 5 giây công việc khi sập ứng dụng; tín hiệu là NFR18 và NFR2 đạt cùng lúc theo phép đo của Story 2.4.

## Requirements

Nguồn: PRD (prd-auratranslate/prd-auratranslate.md); epics.md §Epic 2 (`FRs covered`). Mỗi dòng giữ mã FR của PRD.

- FR16: Ba panel trong một cửa sổ ứng dụng duy nhất: Lưới đối chiếu, Lookup, AI Translation. Đây là câu trả lời trực tiếp cho nỗi đau "bốn đến năm cửa sổ mở cùng lúc". 🔵 (Sửa 2026-08-14: bản cũ khai bốn, tách Source và Editor. Lưới gộp hai cái đó …
- FR19: Cột nguyên văn của lưới hiển thị văn bản gốc (Anh hoặc Trung) kèm Hán Việt cho tài liệu tiếng Trung — xem ở chế độ chuyển đổi hoặc song song, người dùng tự bật tắt. 🔵 (Sửa 2026-08-14: chữ "tab" rút — Hán Việt sống bên trong ô nguyên văn. …
- FR21: Auto-Lookup: bôi đen một cụm từ ở cột nguyên văn của lưới — chữ gốc hoặc âm Hán Việt — → kết quả tra cứu hiện ngay ở panel Lookup. Không copy, không paste, không chuyển cửa sổ. 🔵 (Sửa 2026-08-14: "Panel Source" → "cột nguyên văn của lưới". …
- FR23: Editor phân đoạn văn bản thành segment ở cấp độ câu. Segment là đơn vị của Translation Memory (C5) và của luồng xác nhận.
- FR24: Người dùng xác nhận từng segment. Segment đã xác nhận được đánh dấu trực quan phân biệt với segment đang dở.
- FR25: Điều hướng nhanh giữa các segment: kế tiếp, trước đó, và segment chưa dịch kế tiếp.
- FR26: Chuyển Chương ngay trong Workspace (Chương trước / Chương sau) mà không phải quay về Library.
- FR78: Người dùng gộp hai segment liền nhau hoặc tách một segment khi máy tách sai. Thao tác này phải có, vì tách câu tự động luôn sai ở một tỷ lệ nhất định — nhất là với dấu chấm trong viết tắt, số thập phân và hội thoại.
- FR100: Auto-save định kỳ, không gián đoạn UI. Không được có gai trễ cảm nhận được khi đang gõ.
- FR101: Versioning: lưu lịch sử các phiên bản dịch của từng segment; xem lại và khôi phục được.
- FR117: Xuất xứ bản dịch ở cấp segment, ba giá trị: tôi dịch · người khác dịch · nhập từ tài liệu song ngữ. Xuất xứ được suy ra tự động từ hành vi, không hỏi người dùng:
- FR133: Cắt bỏ một câu hoặc một dải câu khỏi bản dịch. Cờ đặt trên câu nguồn — đây là quyết định "đoạn này không thuộc bản dịch", không phải một mức độ hoàn thành, nên nó là một trục độc lập với trạng thái segment (khuôn `translate="no"` của XLIFF …
- FR134: Bản dịch ngắt đoạn khác bản gốc. Một đoạn nguồn dài tách được thành hai đoạn trong bản dịch. Cấu trúc đoạn của bản dịch là dữ liệu riêng (AD-46), mặc định soi gương bản gốc cho tới khi người dùng đổi. Trong lưới, `Enter` trong ô bản dịch …

## Done when

1. Dịch trọn một Chương bằng tay trong lưới đối chiếu mà không cần AI hay Glossary: văn bản tách segment cấp câu, xác nhận từng câu, điều hướng tới câu chưa dịch kế tiếp, chuyển Chương trong Workspace (FR23–FR26).
2. Sập ứng dụng giữa lúc gõ mất tối đa 5 giây công việc và không frame nào vượt 50 ms khi auto-save chạy (FR100, NFR18, NFR2), có số đo từ mũi thăm dò Story 2.4.
3. Gộp và tách segment tường minh, gồm gộp bằng `Backspace`, cắt bỏ câu và ngắt đoạn của bản dịch (FR78, FR133, FR134).
4. Mọi phiên bản cũ của một segment xem lại và khôi phục được; xuất xứ bản dịch cấp segment được ghi (FR101, FR117).
5. Lưới hai cột đối chiếu nghiệm thu lại FR16, FR19, FR21 trên cùng một hàng nguồn và bản dịch.
6. Story 2.3 và 2.4 còn `in-progress` đã đóng, và đợt kiểm tay của Ice trên ứng dụng thật đã chạy xong.

## Boundaries

Panel Editor và lưới đối chiếu của một Chương. Không gồm Glossary (epic-glossary), AI (epic-ai-mo-va-smart-rag-injector), Translation Memory (epic-translation-memory).

## References

- parent — ../initiative-auratranslate.md, mục Requirements (FR/NFR của PRD)
- prd — ../prd-auratranslate/prd-auratranslate.md
- spec — ../spec-auratranslate/spec-auratranslate.md
- constraint — ../architecture-auratranslate/architecture-auratranslate.md (AD-xx của từng story nằm trong plan của nó)
- ux — ../ux-auratranslate/ux-auratranslate.md

## Notes

- Retrospective: epic-bien-tap-theo-segment-retrospective.md
- Decision: Story 2.2 và 2.5b thay đổi lời văn FR16, FR19, FR21; ba FR này nghiệm thu lại ở Story 2.5b (epics.md, 2026-08-14); FR20 đã rút.
