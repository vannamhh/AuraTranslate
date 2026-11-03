---
id: 16
type: story
title: "Mọi lượt dịch mang ba câu đứng trước làm ngữ cảnh, đường dịch một câu đọc theo khối"
parent: epic-ai-mo-va-smart-rag-injector
covers: [FR70, FR71, FR72, FR73, FR74, FR75]
after: []
refined: true
hitl: false
risk: high
---

# Mọi lượt dịch mang ba câu đứng trước làm ngữ cảnh, đường dịch một câu đọc theo khối

## Description

Hôm nay chỉ dịch cả Chương mới có câu kề, và chỉ ở khe giữa hai câu cần dịch. Dịch một câu tại con trỏ, dịch vùng chọn và thử lại không mang câu kề nào. Vì thử lại luôn chạy như một vùng chọn, câu được thử lại còn có ít ngữ cảnh hơn lần dịch đầu.

Story này thi hành AD-65: mọi lượt dịch mang tối đa 3 câu đứng ngay trước làm khối ngữ cảnh chỉ đọc.

Đường dịch một câu chuyển sang ngữ pháp khối của AD-60, để chữ ngữ cảnh mà mô hình chép lại không lọt vào panel. Cờ `maybe_truncated` của đường này được giữ.

## Requirements

1. **Ba câu đứng trước ở mọi chế độ** (FR70, FR73)
   **Given** một câu cần dịch có ít nhất 3 câu đứng trước nó trong cùng Chương
   **When** người dùng dịch câu đó tại con trỏ, dịch một vùng chọn bắt đầu từ câu đó, hoặc thử lại câu đó
   **Then** prompt có đúng 3 khối ngữ cảnh đứng trước khối của câu đó, theo thứ tự trong văn bản, và không có khối ngữ cảnh nào đứng sau khối cuối của lượt
   **And** câu đã cắt bỏ khỏi bản dịch không bao giờ là câu kề; câu đứng trước nó nữa lấp chỗ
   **And** câu kề không bao giờ lấy từ Chương khác: câu đầu Chương có 0 khối ngữ cảnh, câu thứ hai có 1 khối
2. **Nội dung khối ngữ cảnh** (FR70)
   **Given** một câu kề
   **When** lượt được lắp
   **Then** câu kề đã có bản dịch mang nguồn cùng bản dịch đã lưu; câu kề có bản dịch rỗng sau khi cắt khoảng trắng chỉ mang nguồn, không có mũi tên và không có dòng đích rỗng
   **And** một dòng của câu kề trùng dấu khối thì không làm khối nào đóng sớm
   **And** câu kề không được quét Glossary hay TM, không tính vào trần 100 câu, và không có hàng nào của panel đổi vì nó
3. **Dịch cả Chương** (FR70, FR73)
   **Given** dịch cả Chương
   **When** kế hoạch lượt được lập
   **Then** lượt đầu tiên của Chương mang tối đa 3 câu kề, và lượt bị cắt vì chạm trần 100 câu mang 3 câu kề chỉ có nguồn
   **And** cách cắt lượt theo khe câu đã dịch giữ nguyên như AD-60 mục 2
4. **Vùng chọn gồm nhiều đoạn rời** (FR73)
   **Given** một vùng chọn gồm hai đoạn rời
   **When** thao tác chạy
   **Then** mỗi đoạn đi lượt riêng và mang câu kề riêng, và không lượt nào có khối cần dịch của cả hai đoạn
5. **Dịch một câu chỉ hiện chữ của câu đó** (FR72, FR74)
   **Given** người dùng dịch một câu tại con trỏ
   **When** mô hình trả chữ, kể cả khi nó chép lại một khối ngữ cảnh trước hoặc sau khối của câu
   **Then** chữ hiện dần trên panel chỉ gồm chữ của khối câu đó, không có dấu khối và không có chữ ngữ cảnh, kể cả trong lúc đang chảy
   **And** khối đã đóng thì kết quả là chữ của khối; cờ `maybe_truncated` bật khi lượt kết thúc không có `finish_reason`, hoặc với `finish_reason` khác `stop`
6. **Dịch một câu khi khối hỏng** (FR72, FR75)
   **Given** người dùng dịch một câu tại con trỏ
   **When** luồng kết thúc giữa khối của câu, hoặc mô hình trả chữ mà không có khối nào, hoặc khối xuất hiện hai lần, hoặc chữ của khối rỗng
   **Then** luồng dừng giữa khối thì panel giữ chữ đã nhận của khối với cờ `maybe_truncated` bật, và người dùng vẫn tự quyết có đưa sang Editor hay không
   **And** ba trường hợp còn lại, và luồng dừng giữa khối khi khối chưa có chữ nào, thì panel hiện *thiếu kết quả AI*, không hiện chữ nào, và không bao giờ lấy chữ nằm ngoài khối thay vào
   **And** mô hình không trả chữ nào thì panel hiện câu báo trả lời rỗng hiện có, không hiện *thiếu kết quả AI*
   **And** không có lượt nào tự gửi lại
7. **Xem và gửi lại prompt** (FR71)
   **Given** một câu vừa được dịch tại con trỏ
   **When** người dùng mở màn Xem prompt của câu đó, rồi bấm gửi lại
   **Then** màn hiện đủ các khối ngữ cảnh và đánh dấu đâu là câu kề
   **And** gửi lại gửi đúng prompt đã ghi, kể cả câu kề, và đọc kết quả theo tiêu chí 5 và 6
8. **Không có công tắc câu kề** (FR70)
   **Given** một bộ prompt có chữ `{{chapter_context}}` trong thân
   **When** lượt được lắp
   **Then** chữ đó đứng nguyên văn và được báo là marker lạ như hôm nay, và câu kề vẫn được gửi
9. **Test canh đỏ được** (FR70, FR73)
   **Given** bộ test
   **When** nó chạy
   **Then** có đủ các ca (a)–(l) của AD-65 mục 8, mọi ca đi qua cổng nghe cục bộ với khung viết tay
   **And** mỗi ca canh một luật có một phép gỡ thật ghi trong plan, kể cả những ca AD-65 chưa nêu phép gỡ (plan tự chọn); phép gỡ đó làm ca đỏ đúng vì lý do được đo
10. **Ghi lại những gì chưa đo** (FR70)
    **Given** sổ nợ
    **When** story xong
    **Then** có một mục nợ `Chủ: Epic 4` cho hai việc: Ice dùng thật dịch một câu bằng một mô hình cục bộ (Ollama hoặc LM Studio) và bằng mô hình preset của app; và đối chiếu số token vào của một lượt dịch một câu trước và sau story

## Boundaries

- Must not change: cách cắt lượt theo khe câu đã dịch (AD-60 mục 2) cùng ca (k) của AD-60; luật đối chiếu khối của dịch lô (AD-60 mục 7–8).
- Must not change: bộ ba biến prompt và system prompt mặc định; Proofreader; dịch nhanh (AD-64).
- Must not change: một lượt dịch không ghi gì vào `target_text`; chỉ thao tác đưa sang của người dùng mới ghi (FR72, AD-47).
- Must not change: lệnh dịch một câu, trạng thái của panel AI và hình dạng kết quả trả về webview; huỷ giữa chừng vẫn để lại chữ đã hiện.
- Must not change: một lượt trả lời không có chữ nào vẫn hiện câu báo trả lời rỗng hiện có (bug 4.14).
- Must not change: không tự thử lại (AD-22); hình dạng lỗi qua IPC (AD-21); không thêm điểm ra mạng nào (AD-15).

## References

- parent — _bmad-output/initiative-auratranslate/tickets/epic-ai-mo-va-smart-rag-injector/epic-ai-mo-va-smart-rag-injector.md, Requirements FR70, FR71, FR73, FR74 và Done when 6
- architecture — _bmad-output/initiative-auratranslate/architecture-auratranslate/architecture-auratranslate.md, AD-65 (cùng AD-60 mục 2–4, 7, 12–13, và các dòng 🔵 2026-11-03 dưới AD-14 và AD-60)
- prd — _bmad-output/initiative-auratranslate/prd-auratranslate/prd-auratranslate.md, FR70
- research — _bmad-output/initiative-auratranslate/research-llm-translation-cost-and-quality/research-llm-translation-cost-and-quality.md, R2
- sibling — _bmad-output/initiative-auratranslate/tickets/epic-ai-mo-va-smart-rag-injector/tickets/story-dich-lo-gom-cau-thanh-luot.md, ngữ pháp khối và bộ đọc mà story này dùng lại
- sibling — _bmad-output/initiative-auratranslate/tickets/epic-ai-mo-va-smart-rag-injector/tickets/bug-dich-bang-gemini-ra-ket-qua-trong.md, câu báo trả lời rỗng phải giữ

## Notes

- Decision: 2026-11-03, Ice — chỉ phương án B (câu kề dưới dạng khối, không có biến `{{chapter_context}}`); 3 câu đứng trước, không có câu sau; giữ câu kề ở chế độ cả Chương; không đo trước. Ghi ở AD-65.
- Decision: 2026-11-03, Ice — một lượt dịch một câu không có chữ nào vẫn hiện câu báo trả lời rỗng của bug 4.14; một lượt có chữ mà không có khối dùng được thì panel hiện *thiếu kết quả AI*, vì nói "không có chữ nào" khi mô hình có trả chữ là sai.
- ⚠️ Mô hình cục bộ kém tuân định dạng có thể ra *thiếu kết quả AI* ở câu mà hôm nay vẫn dịch được; đường dịch một câu không có đường lùi về chữ thường (AD-65 mục 5).
- Kiểm thêm ngoài tiêu chí, vì risk high: Ice dịch một câu giữa Chương bằng một mô hình cục bộ thật, mở màn Xem prompt thấy đủ 3 câu kề, và kết quả không chứa chữ của câu kề. Đây là mục nợ ở tiêu chí 10, không chặn story xong.
- Risk high vì story đổi cách đọc kết quả của đường dịch được dùng nhiều nhất, và hỏng ở đây có thể im lặng (chữ ngữ cảnh lọt vào bản dịch, hoặc câu dịch được hôm nay thành thiếu kết quả).
