---
id: 13
type: story
title: "Lỗi nhà cung cấp AI nói đúng nguyên nhân"
parent: epic-ai-mo-va-smart-rag-injector
covers: ["FR65", "FR66", "FR67", "FR75"]
after: []
assignee: ""
refined: true
hitl: false
risk: medium
---

# Lỗi nhà cung cấp AI nói đúng nguyên nhân

## Description

Khi nhà cung cấp AI từ chối một lượt dịch hay một lượt kiểm tra kết nối, người dùng thấy câu tiếng Việt đúng với mã trạng thái, cộng nguyên văn lời nhà cung cấp đã che khoá, theo AD-59. Lý do: hôm nay mọi mã không phải 2xx ra cùng một câu "kiểm lại endpoint và khoá API", nên một tài khoản Gemini hết tiền nạp trước (402, đo 2026-10-27) bị báo như sai khoá. Cùng story sửa hai chỗ cấu hình lộ ra trong lượt rà đó: preset Gemini và phép so mô hình của "Kiểm tra kết nối".

## Requirements

1. **Câu chính theo mã trạng thái** (FR75)
   **Given** nhà cung cấp trả một mã không phải 2xx cho lượt dịch một câu, lượt dịch lô, hoặc lượt kiểm tra kết nối
   **When** lỗi hiện ra
   **Then** câu tiếng Việt nêu đúng họ nguyên nhân của mã đó: 401/403 khoá bị từ chối, 402 thanh toán hoặc số dư, 404 endpoint hoặc mô hình, 429 hạn mức hoặc tốc độ gọi, 5xx sự cố phía nhà cung cấp, mã khác câu chung kèm mã
   **And** lượt dịch lô vẫn nêu câu đã dừng, công việc đang làm không mất, và không có lượt thử lại tự động (AD-22)
2. **Lời nhà cung cấp hiện nguyên văn** (FR75)
   **Given** thân phản hồi lỗi là JSON ở một trong ba hình dạng của AD-59 Rule 2, gồm dạng mảng một phần tử của Gemini
   **When** lỗi hiện ra
   **Then** dưới câu chính có một dòng riêng mang lời nhà cung cấp, nguyên văn, tối đa 300 ký tự, dưới nhãn cho biết đó là lời của nhà cung cấp
   **And** khi thân lỗi là HTML, JSON hỏng, không có `message`, hay dài quá 16 KiB, chỉ câu chính hiện ra, không nhãn rỗng
3. **Khoá không bao giờ lộ** (FR67)
   **Given** lời nhà cung cấp chứa khoá đã cấu hình, nguyên khoá hoặc một đoạn từ 12 ký tự trở lên của khoá
   **When** lỗi đi qua IPC
   **Then** không trường nào của lỗi chứa khoá hay đoạn đó: khoá đầy đủ được thay bằng `***`, còn đoạn sót thì bỏ cả lời nhà cung cấp (AD-29, AD-59 Rule 3)
   **And** lời nhà cung cấp không nằm trong dòng log nào, cũng không nằm trong `Display`/`Debug` của lỗi client
4. **Có test canh, và test đỏ được** (FR67, FR75)
   **Given** bộ test
   **When** nó chạy
   **Then** có đủ bốn ca của AD-59 Rule 6, trong đó ca endpoint giả trả 402 kèm khoá đi qua đường dịch thật, không gọi thẳng hàm che
   **And** gỡ thật lời gọi hàm che trên đường dịch làm ca đó đỏ, vì đúng lý do khoá lộ
5. **Preset Gemini ghi cố định mô hình** (FR65)
   **Given** màn Cài đặt AI
   **When** chọn chip Gemini
   **Then** mô hình được ghi là `gemini-3.8-flash`; endpoint, nhiệt độ và số token của preset như cũ
6. **Kiểm tra kết nối nhận id có tiền tố `models/`** (FR66)
   **Given** endpoint trả danh sách mô hình có id dạng `models/<id>`, như Gemini
   **When** kiểm tra kết nối với mô hình đã cấu hình là `<id>` trần
   **Then** kết quả là đã kết nối; một mô hình thật sự không có trong danh sách vẫn báo thiếu
   **And** quy tắc `:latest` của Ollama vẫn đúng
7. **Ghi lại những gì chưa đo** (FR75)
   **Given** mục nợ "Chưa ai bấm Thử kết nối và dịch một câu trên từng preset có mạng" trong sổ nợ
   **When** story xong
   **Then** mục đó có thêm một dòng 🔵 nêu các điểm của lượt rà 2026-10-27 ở Notes; mục nợ mới cho lượt dùng thật của story có `Chủ: Epic 4`

## Boundaries

- Hình dạng lỗi qua IPC `{ code, message_key, params, retryable }` không đổi; mọi khoá lỗi khác vẫn chỉ mang dữ liệu (AD-21).
- Lỗi không tới được nhà cung cấp, luồng đứt giữa chừng, hết thời gian chờ: câu và hành vi như cũ.
- Cấu hình người dùng đã lưu không bị sửa; preset chỉ ghi khi bấm chip. Preset của tám nhà cung cấp còn lại không đổi.
- Đường dịch 2xx (thân request, đọc luồng, số token) không đổi.

## References

- parent — _bmad-output/initiative-auratranslate/tickets/epic-ai-mo-va-smart-rag-injector/epic-ai-mo-va-smart-rag-injector.md, Requirements FR65, FR66, FR67, FR75
- architecture — _bmad-output/initiative-auratranslate/architecture-auratranslate/architecture-auratranslate.md, AD-59 (cùng AD-21, AD-22, AD-29)
- plan — _bmad-output/initiative-auratranslate/tickets/epic-ai-mo-va-smart-rag-injector/tickets/story-loi-mang-va-loi-api-plan.md, họ lỗi của Story 4.10 mà story này tách theo mã trạng thái
- debt — _bmad-output/initiative-auratranslate/deferred-work.md, §"Deferred from: preset nhà cung cấp AI (2026-10-25, plan-preset-cau-hinh-ai)"
- docs — https://ai.google.dev/gemini-api/docs/models (cập nhật 2026-10-22), https://ai.google.dev/gemini-api/docs/api-errors, https://ai.google.dev/gemini-api/docs/billing

## Notes

- Decision: 2026-10-27, Ice — hiện lời nhà cung cấp (phương án b), không chỉ đổi câu theo mã; AD-59 do Winston soạn cùng ngày.
- Decision: 2026-10-27, Ice — preset Gemini ghi cố định `gemini-3.8-flash`, không dùng bí danh `gemini-flash-latest`.
- Lượt rà 2026-10-27 (nội dung cho dòng 🔵 của tiêu chí 7): tài liệu xAI nói mô hình suy luận không hỗ trợ `stream_options` và `max_tokens`, trong khi app luôn gửi `stream_options`, nên mọi lượt dịch qua xAI có thể đỏ; tài liệu OpenAI về `max_tokens` không đọc được (403); `/models` của lớp tương thích Anthropic không có trong tài liệu; Mistral không còn nêu bí danh `mistral-large-latest`; LM Studio không nói gì về `stream_options`. Đã đo bằng khoá Gemini: 402 `RESOURCE_EXHAUSTED` trên cả đường tương thích OpenAI lẫn đường gốc, `/models` trả id tiền tố `models/`.
- Lượt dùng thật cho mục nợ `Chủ: Epic 4`: Ice gọi dịch một câu bằng khoá Gemini khi tài khoản còn hết tiền nạp trước, thấy câu 402 và lời của Google; rồi kiểm tra kết nối với `gemini-3.8-flash`.
- Risk medium vì story mở một đường cho chữ do bên ngoài viết đi qua IPC, ngay cạnh khoá API.
