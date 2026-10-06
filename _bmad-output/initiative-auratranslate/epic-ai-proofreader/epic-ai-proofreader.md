---
type: epic
title: "AI Proofreader — bắt lỗi trước khi bàn giao"
parent: initiative-auratranslate
covers: [FR80, FR81, FR82, FR83, FR84, FR85, FR86]
after: []
assignee: ""
risk: low
---

# AI Proofreader — bắt lỗi trước khi bàn giao

## Description

Người dịch chạy proofreader **theo yêu cầu** trên một segment, một Chương hoặc vùng đang chọn — **không bao giờ chạy nền**. Mỗi phát hiện gồm loại lỗi, vị trí, giải thích ngắn và đề xuất sửa, hiện **gạch chân lượn sóng ngay dưới đúng cụm chữ có vấn đề**. Chấp nhận hoặc bỏ qua từng cái; đánh dấu *"không phải lỗi"* thì lần quét sau **không báo lại trong cùng Tác phẩm**. Và proofreader **không bao giờ tự sửa văn bản**.

## Outcome

Người dịch bắt được lỗi chính tả, ngữ pháp và sai lệch nghĩa trong bản dịch theo yêu cầu, với tỷ lệ báo động giả đủ thấp để họ không tắt tính năng.

## Requirements

- FR80: Quét **chính tả và ngữ pháp tiếng Việt** trên bản dịch của người dùng.
- FR81: **Đối chiếu bản dịch với bản gốc**, đánh dấu đoạn nghi **dịch sai**, **dịch thoát nghĩa quá xa**, hoặc **cấu trúc câu tối nghĩa**. Nghiệm thu bằng **tỷ lệ báo động giả** (số phát hiện bị đánh dấu *"không phải lỗi"* trên tổng số), phải đủ thấp để người dùng không tắt hẳn tính năng.
- FR82: Proofreader chạy **theo yêu cầu** (một segment, một Chương, hoặc vùng đang chọn), **không chạy nền liên tục**.
- FR83: Mỗi phát hiện gồm **loại lỗi · vị trí · giải thích ngắn · đề xuất sửa**. Người dùng chấp nhận hoặc bỏ qua **từng phát hiện một**.
- FR84: **Bỏ qua có ghi nhớ:** đánh dấu một phát hiện là *"không phải lỗi"* thì lần quét sau không báo lại **trong cùng Tác phẩm**.
- FR85: **Proofreader không được tự sửa văn bản.** Mọi thay đổi phải do người dùng chấp nhận.
- FR86: Kết quả proofread hiển thị **ngay tại chỗ trên Editor**, không phải một danh sách rời.

## Done when

1. Chạy proofreader theo yêu cầu trên một segment, một Chương hoặc vùng chọn, không bao giờ chạy nền.
2. Mỗi phát hiện có loại lỗi, vị trí, giải thích ngắn, đề xuất sửa, và hiện bằng gạch chân lượn sóng dưới đúng cụm chữ.
3. Đánh dấu "không phải lỗi" thì lần quét sau không báo lại trong cùng Tác phẩm, kể cả sau gộp/tách segment.
4. Proofreader không tự sửa văn bản; tỷ lệ báo động giả đo được và nằm dưới ngưỡng đã đặt.

## Boundaries

Proofreader theo yêu cầu. Không phải dịch bằng AI (epic AI mở) và không tự sửa.

## References

- parent — _bmad-output/initiative-auratranslate/prd-auratranslate/prd-auratranslate.md, functional requirements FR80, FR81, FR82, FR83, FR84, FR85, FR86
- spec — _bmad-output/initiative-auratranslate/spec-auratranslate/spec-auratranslate.md
- constraint — _bmad-output/initiative-auratranslate/architecture-auratranslate/architecture-auratranslate.md, các AD nêu trong tiêu chí của story

## Notes

- Unknown: ngưỡng đỗ tỷ lệ báo động giả (Story 9.8) chỉ được nêu là "đủ thấp để người dùng không tắt hẳn tính năng", chưa có con số.
