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
- Decision: 2026-10-10 — incept lại epic từ bản chuyển v6: tám story chia theo giá trị người dùng, `after` rỗng, thành chuỗi lát triển khai có tracer, AD và sweep.
- Decision: 2026-10-10 — tracer là 9.1: quét chính tả/ngữ pháp một segment, gạch chân trong ô; đo CSS Custom Highlight API trên WKWebView ngay trong tracer, không làm được thì dừng và trình hai phương án.
- Decision: 2026-10-10 — story mở đầu 9.9 vá C1, C2 (điều kiện Ice đặt để ký Epic 4) và C4, C6 (nằm trên đường gọi AI sắp rút chung) của retro Epic 4, cùng action #8 rút hàm chung đơn/lô. C3, C5, C7, C8 không thuộc Epic 9.
- Decision: 2026-10-10 — AD-53 (9.10) chỉ quyết lượt ghi khi chấp nhận đề xuất, chữ ký ghi nhớ và phát hiện cũ đi khi gõ sửa; seam lệnh và provider theo AD-13/AD-15, hình dạng phát hiện do 9.1 chốt, phát hiện chỉ sống trong bộ nhớ webview.
- Decision: 2026-10-10 — có story Refactor sweep (9.11). Ngưỡng của 9.8 để `unknown` tới khi Ice thấy 9.2 chạy thật.
- Dropped: 2026-10-10 — 9.7 "Proofreader không tự sửa văn bản": story chỉ gồm cổng canh, setup lớn hơn việc; ba vế chia về 9.1 (quét không đổi ký tự hay trạng thái), 9.3 (quét cả Chương chưa xử lý thì Chương nguyên vẹn), 9.4 (chỉ chấp nhận tường minh mới ghi, cổng nguồn canh đường ghi duy nhất).
- Source conflict: FR86 / tiêu chí gốc 9.5 — vạch lề "đã dùng hết sáu giá trị" vs `SEGMENT_RULE_VALUES` có năm giá trị (`src/panels/editorSegments.ts:81`, `ornament` đã bỏ).
- Source conflict: spine AD-3 (:95) — ghi nhớ proofreader "tham chiếu `segment.id`" vs spine :918 và FR84 khoá theo `(work, chữ ký)`; AD-53 sửa.
- Source conflict: tiêu chí gốc 9.5 — Editor "trang văn bản liền mạch, không chia ô" vs Editor là lưới mỗi segment một ô `contenteditable` từ Story 2.5b (`src/panels/GridPanel.vue`).
- Source conflict: tiêu chí gốc 9.4 — dải "mọc dưới câu" vs khe dải chung nằm trên StatusBar trong `App.vue`, phân xử bởi `src/panels/inlineStripPriority.ts`.
- Source conflict: epic và spine gọi `RagInjector` như một kiểu vs mã là các hàm `core::ai::rag::{gather_*, assemble_prompt}`, và `assemble_prompt` không nhận văn bản bản dịch mà FR80/FR81 cần.
- Source conflict: tiêu chí gốc 9.5 (WCAG AA cả hai theme) vs `tm-rule` 2,48:1 ở theme sáng, DESIGN.md ghi chỉ dùng làm màu nét; là `unknown` của 9.5.
- Source conflict: spine AD-47 ③ — bảng lượt ghi không-phải-người-dùng đã đóng, không có hàng cho chấp nhận đề xuất proofreader; AD-53 (9.10) quyết. Spine AD-51 còn trỏ `commands/segment.rs::write_non_user_target`, nay ở `commands/segment/targets.rs`.
