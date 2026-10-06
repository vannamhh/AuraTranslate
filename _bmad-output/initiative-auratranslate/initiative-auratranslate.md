---
type: initiative
title: "AuraTranslate: nơi bản dịch của người dịch sống"
parent: none
covers: [FR1, FR2, FR3, FR4, FR5, FR6, FR7, FR8, FR9, FR10, FR11, FR12, FR13, FR14, FR15, FR16, FR17, FR18, FR19, FR21, FR22, FR23, FR24, FR25, FR26, FR27, FR28, FR29, FR30, FR31, FR32, FR33, FR34, FR35, FR36, FR37, FR38, FR39, FR40, FR41, FR42, FR43, FR44, FR45, FR46, FR47, FR48, FR49, FR50, FR51, FR52, FR53, FR54, FR55, FR56, FR57, FR58, FR59, FR60, FR61, FR62, FR63, FR64, FR65, FR66, FR67, FR68, FR69, FR70, FR71, FR72, FR73, FR74, FR75, FR76, FR77, FR78, FR79, FR80, FR81, FR82, FR83, FR84, FR85, FR86, FR87, FR88, FR89, FR90, FR91, FR92, FR93, FR94, FR95, FR96, FR97, FR98, FR99, FR100, FR101, FR102, FR103, FR104, FR105, FR106, FR107, FR108, FR109, FR110, FR111, FR112, FR113, FR114, FR115, FR116, FR117, FR118, FR119, FR120, FR121, FR122, FR123, FR124, FR125, FR126, FR127, FR128, FR129, FR130, FR131, FR132, FR133, FR134, FR135, NFR1, NFR2, NFR3, NFR4, NFR5, NFR6, NFR7, NFR8, NFR9, NFR10, NFR11, NFR12, NFR13, NFR14, NFR15, NFR16, NFR17, NFR18, NFR19]
after: []
assignee: ""
status: in-progress
risk: medium
---

# AuraTranslate: nơi bản dịch của người dịch sống

## Description

AuraTranslate là translation workstation chạy local-first, hoàn toàn ngoại tuyến, cho người dịch Anh/Trung sang Việt coi trọng chất lượng hơn tốc độ, trên macOS và Windows. Library giữ mọi Tác phẩm đã dịch; Workspace biên tập theo segment; từ điển nhúng tra cứu tức thì kèm nguồn; Glossary và Translation Memory tích luỹ quyết định của người dịch; AI chỉ đề xuất và người biên tập quyết định. Dự án mã nguồn mở GPL-3.0-or-later. PRD giữ yêu cầu và thuật ngữ, spec giữ chuỗi xây dựng.

## Outcome

Người dịch Việt Nam có một công cụ thay thế QuickTranslator trên mọi hệ điều hành, nơi công sức dịch đọng lại thành Library, Glossary và Translation Memory dùng lại được, và ở đó không bản dịch nào hoàn tất mà không qua tay người.

## Requirements

Yêu cầu nằm ở PRD (FR1–FR135 trừ FR20 đã rút, NFR1–NFR19); mỗi epic trích mã của nó ở `covers`.

## Done when

1. Mọi yêu cầu chức năng còn hiệu lực (FR1–FR135 trừ FR20) và NFR1–NFR19 của PRD đạt ngưỡng nghiệm thu của chính nó, không còn nợ mang tên Ice mở ở epic cuối.
2. Tra cứu hoạt động 100% khi ngoại tuyến, mọi mục từ hiển thị nguồn định nghĩa, và độ trễ Auto-Lookup đầu-cuối p95 dưới 100 ms (NFR1).
3. Một vòng dịch trọn một Chương làm được không chạm chuột (NFR17), và sập ứng dụng mất không quá 5 giây công việc (NFR18).
4. Có thể mang dữ liệu đi: xuất được TMX; bản cài dưới trần 400.000.000 byte và không tải thêm sau khi cài.
5. Chỉ số kết quả công việc (tuân thủ Glossary, mật độ lỗi reviewer sửa, consistency drift) đo được, và hai counter-metric (chấp nhận thẳng bản dịch AI, thời gian quản lý công cụ) được theo dõi, không có dấu hiệu trượt về auto-translate.
6. Bản phát hành chạy được trên macOS và Windows không ký số, với đường dẫn vượt rào cản cài đặt được ghi rõ cho người dùng.

## Boundaries

Ứng dụng desktop Tauri v2 (Rust và Vue 3), SQLite đóng gói, dữ liệu từ điển phát hành qua GitHub Release. Không tài khoản, không cloud sync, không telemetry; không dịch xong mà không qua tay người; không đặt mục tiêu tốc độ. Tracer path: bôi đen một cụm trong Chương nhập từ tệp, thấy tra cứu tức thì, dịch tay theo segment, lưu vào Library, xuất ra.

## References

- prd — _bmad-output/initiative-auratranslate/prd-auratranslate/prd-auratranslate.md, section 4 Mục tiêu và 6 Yêu cầu chức năng
- spec — _bmad-output/initiative-auratranslate/spec-auratranslate/spec-auratranslate.md
- architecture — _bmad-output/initiative-auratranslate/architecture-auratranslate/architecture-auratranslate.md
- ux — _bmad-output/initiative-auratranslate/ux-auratranslate/ux-auratranslate.md
- brief — _bmad-output/initiative-auratranslate/brief-auratranslate/brief-auratranslate.md, for history only

## Notes

- Decision: FR20 (Sync Scrolling) bị rút 2026-08-14 qua đề xuất đổi hướng, Ice ký; vì vậy `covers` bỏ FR20.
- Decision: thứ tự epic trong `tickets.toml` là thứ tự thực thi 1, 2, 3, 5, 6, 4, 11, 7, 8, 9, 10 (Ice chốt 2026-10-06, Q9); id epic giữ số v6.
- Source conflict: bốn FR nghiệm thu chia đôi giữa hai epic (FR13 giữa Epic 1 và 6; FR44 và FR129 giữa Epic 6 và 7; FR70 giữa Epic 4 và 7); không ghi `after` cấp epic vì `epics.md` nêu thứ tự chạy chứ không nêu phụ thuộc (Q23).
- Open question: entry 7.1 chờ phần cấu trúc ở 6.13; `after` cấp entry chỉ ghi nơi tiêu đề story nêu rõ phụ thuộc.
