---
type: epic
title: "Dịch nhanh — hiểu câu nguồn bằng mô hình rẻ, không chạm văn bản"
parent: initiative-auratranslate
covers: [FR153, FR154, FR155, FR156, FR157, FR158]
sources: [_bmad-output/initiative-auratranslate/prd-auratranslate/prd-auratranslate.md, _bmad-output/initiative-auratranslate/ad-brief-dich-nhanh/ad-brief-dich-nhanh.md]
companions: [_bmad-output/initiative-auratranslate/architecture-auratranslate/architecture-auratranslate.md, _bmad-output/initiative-auratranslate/ux-auratranslate/EXPERIENCE.md, _bmad-output/initiative-auratranslate/ux-auratranslate/DESIGN.md, _bmad-output/initiative-auratranslate/ux-auratranslate/mockups/dich-nhanh.html]
after: []
refined: true
risk: high
---

# Dịch nhanh — hiểu câu nguồn bằng mô hình rẻ, không chạm văn bản

## Description

Bảng chờ Glossary và màn xem trước nhập URL đầy câu tiếng Trung, tiếng Anh mà người dùng phải hiểu mới quyết được nhận hay bỏ. Epic thay đường `Dịch thử` đang có ở `abfd6026` (cấu hình chính, kết quả trong bộ nhớ) bằng một cấu hình giá rẻ riêng, một kho lưu tới khi người dùng dọn, tuỳ chọn tự dịch chữ đang hiện, và mở rộng sang các khối ở màn xem trước.

## Outcome

Người dịch đọc hiểu được câu nguồn ở hai màn quyết định mà không tốn tiền của mô hình chính và không trả tiền hai lần cho một câu, thể hiện bằng 0 lời gọi mạng khi trúng kho và 0 lời gọi khi tuỳ chọn tắt.

## Requirements

- R1 (FR153; AD-64 mục 2): người dùng khai nhà cung cấp, endpoint, mô hình và khoá riêng, chỉ ở tầng Toàn cục. Trống cả endpoint lẫn mô hình thì cú bấm dùng trọn cấu hình chính ở tầng Toàn cục; khai một nửa thì báo chưa cấu hình, không đọc khoá chính.
- R2 (FR154; AD-64 mục 6): dưới Ngữ cảnh của hàng đang mở, thứ tự đọc là bản dịch trong Tác phẩm, kho, rồi mới gọi mạng; trúng kho thì 0 lời gọi tới `TranslationProvider`.
- R3 (FR155; AD-64 mục 7): ở tầng Giữ/Loại khối, kể cả khi chưa có Tác phẩm, người dùng bấm để dịch các khối đang thấy mà kho chưa có, qua `guard_text` và dấu khối của AD-60, trong trần số khối và số ký tự của một lượt.
- R4 (FR156; AD-64 mục 1, 9): tuỳ chọn mặc định tắt, chỉ có tác dụng khi cấu hình nhanh đã khai riêng; Rust đọc lại hai điều kiện ngay lúc gọi và trả `auto_off`; chỉ dịch chữ đang hiện; không bắt đầu khi lô dịch thật đang chạy; chỗ đã lỗi chỉ thử lại khi bấm.
- R5 (FR156; AD-64 mục 8): cả app có một lượt dịch nhanh, huỷ theo đúng `run_id`, bề mặt khác nhận `busy`, cuộn không huỷ lô, và không lượt nào chạm `AiTranslateGeneration`.
- R6 (FR157; AD-64 mục 3, 4): kết quả nằm ở `quick-translation.db`; một chuỗi nguồn sau `quick_key` là một hàng, chung mọi Tác phẩm, đổi mô hình không dịch lại; kho hỏng thì app vẫn chạy và dịch nhanh báo lỗi.
- R7 (FR157; AD-64 mục 5): chỉ lưu kết quả trọn vẹn; lô bị huỷ giữ các khối đã đóng; lượt đã huỷ hoặc đã bị dọn không ghi được; không đường nào ghi `target_text`, TM hay Glossary.
- R8 (FR157; EXPERIENCE §Dịch nhanh): mỗi câu dịch hiện nhãn nguồn và mô hình thật, `Dịch lại` thay bản cũ.
- R9 (FR158; AD-64 mục 10): Cài đặt → Dữ liệu hiện số câu, dung lượng, số câu theo mô hình; dọn tất cả hoặc dọn quá N ngày tính từ lúc dịch; dọn huỷ lượt đang chạy và dung lượng tệp giảm thật. Không tự dọn, không trần dung lượng.
- R10 (FR76; AD-64 mục 11): mỗi bề mặt hiện token và chi phí ước tính của lượt vừa chạy; không ghi `AiCallRecords`, không đổi `LastAssembledPromptState`.
- R11 (NFR12; AD-64 mục 1): tuỳ chọn tắt hoặc cấu hình nhanh trống thì 0 lời gọi AI nào không có cú bấm, đo ở cả hai bề mặt.
- R12 (NFR17, NFR20; EXPERIENCE §Dịch nhanh): mọi nút có đường chuột và `Tab`; đang có lượt chạy thì `Esc` đầu huỷ lượt; `J` `K` `Space` ở tầng khối không đổi; chiều cao mọc không dời nút dưới chuột.
- R13 (AD-64 mục 12): mỗi đối chứng (a)–(h) là một phép gỡ thật và đỏ đúng lý do.

## Constraints

- AD-15, AD-64 mục 1: không thêm điểm ra mạng; điểm ra vẫn là adapter `TranslationProvider`.
- AD-13: `core/quick_translation` không phụ thuộc `core::ai`.
- AD-16: câu dịch hiển thị như chữ thuần.
- AD-14: prompt do hàm thuần lắp.
- AD-11, AD-30: một kết nối ghi; kho này xoá luôn an toàn nên không dùng di trú.

## Done when

1. Mọi đối chứng (a)–(h) của AD-64 mục 12 đã chạy và đỏ đúng lý do; test canh của từng story xanh.
2. Trên WKWebView thật, một spec e2e cho mỗi bề mặt: trúng kho không có lời gọi mạng; tuỳ chọn tắt không có lời gọi nào.
3. `npm run check:commands` xanh và `POINTER_DEBT` không tăng.
4. Mục nợ `Chủ: Epic 15` đã đóng: Ice chạy một lượt dùng thật bằng khoá thật trên một Tác phẩm `zh`, một Tác phẩm `en` và một lần nhập URL, kể cả `Dọn` thu nhỏ tệp.
5. Phần chỉ kiểm được trên Windows (keychain ô thứ hai, `VACUUM` khi kho đang có kết nối ghi) nằm trong một mục `B7`.
6. CI xanh cả nửa Windows.

## Boundaries

- Ngoài epic: dịch trước cả danh sách, dịch toàn bộ Chương xem trước, dịch khối ảnh, trần dung lượng, tự dọn, ghi đè cấu hình theo Tác phẩm.
- Ngoài epic: đưa câu dịch nhanh vào Editor, TM hay Glossary; xem lần gọi ở Prompt Inspector (FR71).
- Ngoài epic: phím tắt mặc định cho `Dịch thử` và `Tự dịch`.
- Must not change: lô dịch thật và `AiTranslateGeneration`; đường `resolve_call_config` của cấu hình chính; hành vi `J` `K` `Space` ở tầng khối.
- Touch point: `src/ImportPreviewOverlay.vue` — tầng khối; owner: epic-duong-nhap.
- Touch point: `src/GlossaryQueueOverlay.vue` — khối Ngữ cảnh; owner: epic-glossary.

## References

- parent — _bmad-output/initiative-auratranslate/initiative-auratranslate.md, section Requirements
- prd — _bmad-output/initiative-auratranslate/prd-auratranslate/prd-auratranslate.md, §6.6 Dịch nhanh
- architecture — _bmad-output/initiative-auratranslate/architecture-auratranslate/architecture-auratranslate.md, §AD-64
- ux — _bmad-output/initiative-auratranslate/ux-auratranslate/EXPERIENCE.md, §Dịch nhanh
- design — _bmad-output/initiative-auratranslate/ux-auratranslate/DESIGN.md, §Câu dịch tham khảo
- design — _bmad-output/initiative-auratranslate/ux-auratranslate/mockups/dich-nhanh.html

## Notes

- Decision: 2026-11-01, Ice — N mặc định 30 ngày, khối đã loại vẫn dịch khi đang thấy, khối ảnh không dịch, `Dọn tất cả` xác nhận tại chỗ (`ux-auratranslate/.memlog.md`). Spine §AD-64 còn ghi N chưa chốt; Winston thêm 🔵 khi sửa spine lần tới.
- Decision: 2026-11-01, Ice duyệt — Story 15.1 là tracer bullet; 15.3 đi ngay sau cấu hình vì là phần chưa có số đo; 15.4 đứng trước tự dịch để dữ liệu tự sinh luôn có đường dọn. 15.3 giữ một story với plan hai pha Rust và Webview, theo AGENTS.md.
- Decision: 2026-11-01, Ice chọn một epic mới thay vì gộp vào Epic 4: một năng lực giao trọn, và Epic 4 đã xong hết story, gộp sẽ thành 22 story.
- Decision: 2026-11-01, Ice — mở Cài đặt không huỷ lượt đang chạy, chỉ tắt `Tự dịch` mới huỷ. Nếu Story 15.1 đo thấy mở Cài đặt làm đóng lớp phủ thì AD-64 mục 8 vẫn thắng: đóng lớp phủ là huỷ.
- Open question: `QUICK_BATCH_MAX_BLOCKS`, `QUICK_BATCH_MAX_CHARS` (Story 15.3) và thời gian chờ của tự chạy (Story 15.5) chưa có số đo; story đo trên trang thật rồi mới chốt (AD-64 §Chưa chốt).
