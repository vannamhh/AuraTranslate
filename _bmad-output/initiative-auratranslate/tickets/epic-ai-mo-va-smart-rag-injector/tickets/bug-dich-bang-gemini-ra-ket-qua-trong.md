---
id: 14
type: bug
title: "Dịch bằng Gemini ra kết quả trống, không báo lỗi"
parent: epic-ai-mo-va-smart-rag-injector
covers: ["FR74", "FR75", "FR76"]
after: []
assignee: ""
refined: true
hitl: false
risk: medium
severity: P1
---

# Dịch bằng Gemini ra kết quả trống, không báo lỗi

## Description

Bấm "Dịch câu này" với nhà cung cấp Gemini: panel Đề xuất AI hiện số token đã dùng nhưng ô kết quả trống, và không có câu báo lỗi nào. Gemini thật ra đã trả bản dịch. Kết quả phải hiện ra; và khi một lượt dịch kết thúc mà thật sự không có chữ nào, người dùng phải được báo, không phải nhìn một ô trống.

## Reproduction

1. Cài đặt AI tầng Toàn cục: provider `gemini`, endpoint `https://generativelanguage.googleapis.com/v1beta/openai`, mô hình `gemini-3.5-flash-lite`, nhiệt độ và số token tối đa để mặc định, system prompt mặc định.
2. Mở một Tác phẩm có bộ prompt "Bình luận – Chính luận", chọn câu 「一切都好得很。」, bấm "Dịch câu này".
3. Thực tế: dòng "933 token — mô hình này chưa có giá trong bảng giá…", ô kết quả trống, không báo lỗi (ảnh của Ice, 2026-10-27). Mong đợi: bản dịch tiếng Việt hiện ra.

Đo 2026-10-27, cùng khoá trong Keychain của app, cùng system prompt và bộ prompt, bốn lượt: lần nào Gemini cũng trả HTTP 200, `finish_reason: stop`, có bản dịch (vd. "Mọi việc đều rất tốt đẹp."). Luồng thô cho thấy mỗi khung mang cả `choices[0].delta.content` lẫn `usage`:

```
data: {"choices":[{"delta":{"content":"Mọi","role":"assistant"},"index":0}],…,"usage":{"completion_tokens":2,"prompt_tokens":233,"total_tokens":235}}
data: {"choices":[{"delta":{"content":" việc đều tốt đẹp hoàn toàn.","role":"assistant"},"index":0}],…,"usage":{"completion_tokens":9,…}}
data: {"choices":[{"delta":{"extra_content":{"google":{"thought_signature":"…"}},"role":"assistant"},"finish_reason":"stop","index":0}],…,"usage":{…}}
data: [DONE]
```

## Cause Hypothesis

Bộ đọc luồng trả về đúng một kết quả cho mỗi khung, và một khung có `usage` mang số token thì được đọc thành số token trước khi `content` được xét, nên chữ trong cùng khung bị bỏ (Story 4.11 thêm nhánh `usage`). Dữ liệu mẫu của test hiện có theo kiểu OpenAI: `usage` chỉ ở khung cuối, cùng `choices: []`, nên không chạm tới trường hợp này. Ngoài ra, hết luồng mà không có chữ nào thì không đường nào báo, và `finish_reason` không được đọc.

## Acceptance Criteria

1. **Chữ và số token trong cùng một khung đều được giữ** (FR74, FR76)
   **Given** nhà cung cấp gửi các khung mang cả `delta.content` lẫn `usage`, như luồng đo được ở Reproduction
   **When** dịch một câu hoặc dịch lô
   **Then** toàn bộ chữ hiện dần trong panel, đúng thứ tự, và số token hiện sau lượt gọi là số của khung `usage` cuối cùng
   **And** luồng kiểu OpenAI (`usage` chỉ ở khung cuối với `choices: []`) và luồng không có `usage` cho kết quả như trước
2. **Lượt dịch không có chữ nào thì được báo** (FR75)
   **Given** một luồng kết thúc đúng cách (`[DONE]`) mà không có chữ nào
   **When** lượt dịch một câu hoặc một câu trong lô kết thúc
   **Then** người dùng thấy một câu báo rằng mô hình trả về rỗng, kèm lý do dừng nhà cung cấp gửi (`finish_reason`) khi có, thay cho ô trống; không có gì được ghi vào Editor và không tự thử lại (AD-22)
   **And** lượt dịch lô dừng ở câu đó và nêu tên câu, như các lỗi khác của lô
3. **Có test bắt đúng lỗi này, và test đỏ được**
   **Given** bộ test
   **When** nó chạy
   **Then** một ca dùng đúng hình dạng khung Gemini ở Reproduction, qua đường dịch thật trên máy chủ giả, thấy đủ chữ; một ca luồng `[DONE]` không chữ thấy câu báo rỗng
   **And** đưa lại cách đọc "có `usage` thì bỏ qua `content`" làm ca Gemini đỏ vì mất chữ
4. **Hoặc: không cần sửa, có bằng chứng**
   **Given** Reproduction
   **When** chạy trên mã hiện tại
   **Then** bản dịch đã hiện ra, bằng chứng ghi trong Notes — thay cho 1 và 2

## References

- parent — _bmad-output/initiative-auratranslate/tickets/epic-ai-mo-va-smart-rag-injector/epic-ai-mo-va-smart-rag-injector.md, Requirements FR74, FR75, FR76
- plan — _bmad-output/initiative-auratranslate/tickets/epic-ai-mo-va-smart-rag-injector/tickets/story-so-token-va-uoc-tinh-chi-phi-plan.md, cách đọc khung `usage` của Story 4.11
- architecture — _bmad-output/initiative-auratranslate/architecture-auratranslate/architecture-auratranslate.md, AD-21, AD-22

## Notes

- Decision: 2026-10-27, Ice — mở ticket lỗi này và dựng ngay, gồm cả câu báo khi lượt dịch rỗng.
- Lượt đo đọc khoá từ Keychain của app mà không in ra; con số 933 token trong app khác 242 token của lượt đo dựng lại, chưa rõ phần chênh (ngữ cảnh khác trong prompt thật) — không ảnh hưởng nguyên nhân.
- Risk medium vì bộ đọc luồng dùng chung cho mọi nhà cung cấp.
