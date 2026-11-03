---
type: epic
title: "Quy ước chữ Việt — bắt và thay cụm cần tránh trong bản dịch, không gọi AI"
parent: initiative-auratranslate
covers: [FR159]
sources: [_bmad-output/initiative-auratranslate/prd-auratranslate/prd-auratranslate.md]
companions: [_bmad-output/initiative-auratranslate/architecture-auratranslate/architecture-auratranslate.md]
after: []
refined: true
risk: high
---

# Quy ước chữ Việt — bắt và thay cụm cần tránh trong bản dịch, không gọi AI

## Description

Người dịch có những quy ước về chính chữ Việt (*đảm bảo → bảo đảm*, *thập kỷ → thập niên*) mà Glossary không bắt được vì chúng không neo vào câu nguồn, Proofreader thì tốn token và không tất định. Epic dựng một danh sách luật hai tầng, một bộ quét cục bộ trên bản dịch, dấu tại chỗ, nhận thay từng chỗ hoặc cả Chương, ghi nhớ "không phải lỗi", và xuất nhập để chia sẻ. Bề mặt đánh dấu, dải nhận và bộ nhớ bỏ qua của Proofreader (Epic 9) có sẵn để dùng lại.

## Outcome

Người dịch thấy mọi chỗ vi phạm quy ước của mình ngay trong Editor và sửa cả Chương trong một lượt có kiểm, với 0 lời gọi AI và không chỗ nào bị sửa khi chưa được nhận.

## Requirements

- R1 (FR159): một luật gồm cụm cần tránh và cụm thay thế nếu có; luật không có cụm thay thế cho ra dấu và không có thao tác thay.
- R2 (FR159, FR103): luật ở hai tầng Global và Tác phẩm; tầng Tác phẩm thêm được luật và tắt được luật Global mà không xoá nó; trùng cụm cần tránh thì luật Tác phẩm thắng. Kiểm: luật Global tắt ở Tác phẩm A cho 0 dấu trong A và vẫn có dấu ở Tác phẩm B.
- R3 (FR159): khớp cục bộ, tất định, 0 lời gọi AI; NFC chỉ để so, văn bản lưu không đổi; không phân biệt hoa thường; không gộp *i/y*; chỗ khớp nằm cạnh đầu, cuối văn bản hoặc ký tự không phải chữ cái hay chữ số. Kiểm: *kỷ* không khớp *kỉ*, *an* không khớp trong *toàn*, hai lượt quét cùng đầu vào ra cùng tập chỗ khớp.
- R4 (FR159, FR86): quét bản dịch trong Editor kể cả chữ đang gõ, và gợi ý ở panel AI Translation (chỉ đánh dấu, không thay); câu đã cắt (FR133) không bị quét; dấu hiện tại chỗ.
- R5 (FR159): nhận thay từng chỗ; cụm thay lấy kiểu hoa của chỗ bị thay (*Đảm bảo → Bảo đảm*, *ĐẢM BẢO → BẢO ĐẢM*, kiểu khác giữ cách viết trong luật).
- R6 (FR159): thay cả Chương qua danh sách mọi chỗ khớp kèm câu chứa nó, chọn sẵn, bỏ chọn được; chạy một lượt trên văn bản trước khi thay; chỗ bị bỏ chọn giữ nguyên từng ký tự.
- R7 (FR159, FR117, FR56, FR101): nhận thay giữ xuất xứ của câu; câu đã xác nhận giữ xác nhận, cặp TM mang chữ mới, lịch sử phiên bản ghi lại.
- R8 (FR159, FR84): đánh dấu một chỗ là "không phải lỗi" thì chỗ đó không hiện lại trong cùng Tác phẩm, kể cả sau khi đóng và mở lại.
- R9 (FR159, NFR9, FR49): xuất và nhập danh sách luật dưới dạng CSV/TSV hai cột, cột hai trống là "không dùng"; xuất rồi nhập vào danh sách trống cho lại đúng các luật đó.
- R10 (FR159, FR85): không đường nào đổi bản dịch khi người dùng chưa nhận.
- R11 (NFR17, NFR20): mọi thao tác của epic làm được bằng bàn phím và bằng chuột.
- R12 (FR159, FR46): chỗ khớp nằm trọn trong bản dịch của một mục Glossary đã chốt mà câu nguồn chứa thuật ngữ đó thì không bị đánh dấu.

## Constraints

- AD-47 ①: nhận thay là lệnh rời, tức lượt ghi không-phải-người-dùng, ghi cả mốc lẫn cột xuất xứ; cần một hàng mới ở AD-47 ③ do Winston viết.
- AD-53: khuôn chấp nhận đề xuất (phép kiểm văn bản chưa đổi, khoảng `[start, end)`) và chữ ký ghi nhớ; tập loại phát hiện đóng, thêm loại là sửa spine.
- AD-1: luật khớp và luật thay nằm ở Rust; webview chỉ vẽ dấu.
- AD-13 và FR77: bộ quét không phụ thuộc `core::ai` và chạy khi không cấu hình AI.
- AD-49: thao tác rời mang đường lui của chính nó.

## Done when

1. Mỗi R có test canh xanh, và mỗi guard mới đã được đối chứng bằng một phép gỡ thật.
2. Spec e2e trên WKWebView: gõ *đảm bảo* vào một ô thì dấu hiện; thay cả Chương với một chỗ bỏ chọn thì chỗ đó giữ nguyên từng ký tự và xuất xứ các câu không đổi.
3. `npm run check:commands` xanh và `POINTER_DEBT` không tăng.
4. Mục nợ `Chủ: Epic 16` đã đóng: Ice chạy một lượt dùng thật với danh sách quy ước của mình trên một Tác phẩm thật, kể cả thay cả Chương trên một Chương đã xác nhận.
5. CI xanh cả nửa Windows.

## Boundaries

- Ngoài epic: luật dạng biểu thức chính quy; gộp *i/y*; tự sửa; quét ngoài Chương đang mở; tự chèn danh sách luật vào prompt (FR69 vẫn là đường tay); định dạng gói hay hạ tầng phân phối.
- Must not change: chữ ký ghi nhớ và lượt ghi của Proofreader (AD-53); xuất xứ của câu sau một lần thay (R7).
- Touch point: `src/panels/proofreadHighlight.ts`, `src/ProofreaderStrip.vue` — vẽ dấu và dải nhận; owner: epic-ai-proofreader.
- Touch point: `src/panels/GridPanel.vue` — ô bản dịch; owner: epic-bien-tap-theo-segment.
- Touch point: `src/panels/AiTranslationPanel.vue` — dấu trên gợi ý; owner: epic-ai-mo-va-smart-rag-injector.

## References

- parent — _bmad-output/initiative-auratranslate/initiative-auratranslate.md, section Requirements
- prd — _bmad-output/initiative-auratranslate/prd-auratranslate/prd-auratranslate.md, §6.4 Quy ước chữ Việt, §13 Q10 và Q11
- architecture — _bmad-output/initiative-auratranslate/architecture-auratranslate/architecture-auratranslate.md, §AD-47, §AD-53
- review — _bmad-output/initiative-auratranslate/prd-auratranslate/review-rubric-fr159.md

## Notes

- Decision: 2026-11-03, Ice — nhận thay giữ xuất xứ của câu như chấp nhận đề xuất chính tả của AD-53, nhưng câu đã xác nhận giữ xác nhận và cặp TM mang chữ mới; PRD FR159 sửa theo.
- Decision: 2026-11-03, Ice — một epic mới thay vì gộp vào Epic 9: FR159 thuộc C4 và phải chạy không cần AI, Epic 9 là C7 và đang sắp đóng.
- Decision: 2026-11-03, Ice — ghi epic và entry ngay, entry bị chặn mang `unknown`, thay vì chờ AD và UX như Epic 15; cắt lại nếu AD đổi hình.
- Decision: 2026-11-03, Sofia đề xuất, Ice duyệt — Story 16.1 là tracer bullet; 16.2 thay cả Chương đi ngay sau vì là phần ghi hàng loạt và rủi ro ghi hỏng chữ.
- Unknown: AD của Winston cho bộ quét — hàng AD-47 ③, giữ xác nhận và ghi lại TM khác AD-53 ra sao, loại phát hiện mới hay thực thể riêng, quét ở đâu và ngân sách hiệu năng khi gõ; chặn 16.1, 16.2, 16.4.
- Unknown: UX của Sally — dấu quy ước phân biệt với dấu Proofreader, màn xác nhận thay cả Chương; chặn 16.1, 16.2.
- Decision: 2026-11-03, Ice — Q10 PRD: Glossary thắng (R12). Q11 PRD: CSV/TSV hai cột như Glossary (R9).
