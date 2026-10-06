---
type: epic
title: "Trả nợ nền — đóng nợ đã hoãn của Epic 1–6 trước khi xây tiếp"
parent: initiative-auratranslate
covers: []
after: []
assignee: ""
status: in-progress
risk: low
---

# Trả nợ nền — đóng nợ đã hoãn của Epic 1–6 trước khi xây tiếp

## Description

Lượt rà sổ nợ 2026-09-23 để lại 199 mục còn đúng trên mã, nằm trong phần nền Epic 1–6 đã dựng — cổng kiểm, e2e, tra cứu, Glossary, editor, tầng ghi, đường nhập, Library. Không epic tính năng nào còn lại chạm tới chúng. Epic này không thêm năng lực người dùng thấy được; nó trả nợ trước khi Epic 7 dựng TM lên cùng phần nền đó.

## Outcome

Nợ đã hoãn của phần nền (cổng kiểm, e2e, tra cứu, Glossary, editor, tầng ghi, đường nhập, Library) được đóng hoặc quyết dứt điểm trước khi epic Translation Memory dựng lên cùng phần nền đó.

## Requirements

Không có FR (epics.md: "FRs covered: không"). Nguồn: `epics.md` §Epic 11, nay ở archive-v6.

## Done when

1. Mỗi mục `deferred-work.md` mang `Chủ: Story 11.N` kết thúc bằng một trong ba dạng: đã đóng kèm bằng chứng mã hoặc test, không làm kèm lý do, hoặc chuyển chủ cụ thể mới kèm lý do.
2. `check:debt-owner` Kiểm C xanh: không mục mở nào còn trỏ vào story của epic này đã `done`.
3. Không mục nào treo điều kiện bị dựng mã phòng trước; không bất biến kiến trúc nào bị đổi trong story (mục đòi đổi chuyển `Chủ: Winston`).
4. Lỗi ranh giới do chính các story của epic sinh ra (Story 11.8) được sửa trước lượt dùng thật của Ice.

## Boundaries

Trả nợ phần nền của các epic đã dựng; không thêm năng lực người dùng thấy được.

## References

- parent — _bmad-output/initiative-auratranslate/prd-auratranslate/prd-auratranslate.md, functional requirements (không có)
- spec — _bmad-output/initiative-auratranslate/spec-auratranslate/spec-auratranslate.md
- constraint — _bmad-output/initiative-auratranslate/architecture-auratranslate/architecture-auratranslate.md, các AD nêu trong tiêu chí của story
- change — _bmad-output/initiative-auratranslate/change-epic-tra-no-nen/change-epic-tra-no-nen.md, thêm Epic 11

## Notes

- Decision: epic chạy ngay sau epic AI mở, dù khối `epic-11` nằm cuối `sprint-status.yaml` (thứ tự thực thi do Ice chốt 2026-10-06).
- AC chung cho mọi story (mỗi story thừa kế cả bốn khối): (1) mọi mục `Chủ: Story 11.N` được Task 0 đọc lại trên mã HEAD vì có thể đã tự đóng hoặc đổi dạng; (2) story sang `done` thì mỗi mục kết thúc bằng đúng một trong ba dạng đóng và Kiểm C đỏ nếu còn mục mở mang tên story; (3) mục treo điều kiện chưa xảy ra thì không dựng mã phòng trước, chốt `KHÔNG LÀM` hoặc chuyển `Chủ: Ice`; (4) mục đòi đổi bất biến kiến trúc thì dừng và chuyển `Chủ: Winston`.
- Unknown: danh sách mục của mỗi story là `grep 'Chủ: Story 11.N' deferred-work.md`, không chép vào đây.
- Retrospective: epic-tra-no-nen-retrospective.md
