---
id: 18
type: bug
title: "Xác nhận câu chậm dần theo số câu đã sửa trong phiên"
parent: epic-bien-tap-theo-segment
covers: []
after: []
assignee: ""
refined: true
hitl: false
risk: medium
severity: P2
---

# Xác nhận câu chậm dần theo số câu đã sửa trong phiên

## Description

Người dịch sửa rồi `Mod+Enter` từng câu. Mỗi lượt xác nhận làm webview đứng lâu hơn lượt trước, theo số câu đã sửa từ lúc mở app. Kiểu Song song Hán Việt chịu nặng nhất. Khởi động lại app thì nhanh trở lại. Mỗi lượt xác nhận phải tốn gần như nhau, dù trong phiên đã sửa bao nhiêu câu.

Vé này không phải bản sửa cho lỗi gõ có dấu chậm trên Windows; lỗi đó thuộc spike 17. Hai vé làm song song được.

## Reproduction

1. Mở một Chương 320 câu chữ Hán, bật kiểu Song song Hán Việt.
2. Đưa bản dịch AI vào mọi câu.
3. Lần lượt sửa vài chữ rồi `Mod+Enter` từng câu.
4. Ở các mốc 0, 50, 100 và 300 câu đã xác nhận, đo khoảng từ `Mod+Enter` tới khi con trỏ sang hàng kế, và khoảng luồng chính bị chặn sau khi Rust trả lời.

- Thực tế (2026-11-12, WKWebView, bản debug `--features wdio`, tải máy 2,9–4,2, bàn đo `e2e/measure/confirm-trace.e2e.mjs`): luồng chính bị chặn 11–27 ms ở mốc 0, 50–52 ms ở mốc 100, 108–126 ms ở mốc 300. Rust `confirm_segment` giữ 2–3 ms ở mọi mốc. Bàn đo `e2e/measure/confirm-typing-parallel.e2e.mjs` cho độ dốc khoảng +0,33 ms mỗi câu ở Song song và +0,05 ms mỗi câu ở chữ thường.
- Mong đợi: đường cong phẳng theo số câu đã xác nhận.

## Cause Hypothesis

1. Sau khi Rust trả lời, luồng chính của webview bị chặn một khoảng tăng theo số câu đã xác nhận. *Đã đo.*
2. Lượt chép chữ đang gõ trở lại ô của lưới tra ô bằng một lượt quét cả lưới cho **mỗi** câu trong tập "đã sửa trong phiên", và chạy mỗi lần danh sách câu của Editor đổi. *Một phép gỡ thật vòng này làm đường cong phẳng: còn 19–35 ms ở mốc 300.*
3. Tập "đã sửa trong phiên" chỉ lớn lên. Danh sách câu giữ chữ lúc nạp, và lượt xác nhận chỉ thay trạng thái, nên tập này là bản sống duy nhất của chữ đã gõ kể từ lúc nạp Chương. *Đọc từ mã.*
4. Lượt chép lại không biết Vue vừa viết lại ô nào, nên duyệt hết. Vue chỉ ghi đè chữ của một ô bản dịch khi chữ trong danh sách câu của ô đó đổi, hoặc khi cả lưới được dựng lại. Nếu vậy, ở mỗi lượt `Mod+Enter` vòng này tra hàng trăm ô mà không ghi ô nào. *Suy luận từ mã, chưa đếm số lần ghi thật.*
5. Kiểu Song song làm mỗi lượt quét đắt hơn, vì lưới có thêm hàng nghìn phần tử ruby.

## Acceptance Criteria

1. **Xác nhận không chậm dần**
   **Given** Chương 320 câu ở Song song, mọi câu đã có bản dịch, cùng bản dựng và cùng phiên đo
   **When** sửa rồi xác nhận lần lượt tới 300 câu bằng bàn đo `confirm-trace`
   **Then** khoảng `Mod+Enter` tới con trỏ sang hàng kế ở các mốc 50, 100 và 300 không vượt mốc 0 quá 15 ms
   **And** số đo ghi cho cả Song song lẫn chữ thường, kèm bản dựng, tải máy, số câu, và số lần ghi DOM thật của lượt chép lại trong một lượt xác nhận, trước và sau khi sửa
2. **Có số trước khi sửa, cùng điều kiện với số sau khi sửa**
   **Given** mã ở commit bắt đầu vé này
   **When** chạy bước tái hiện
   **Then** các số của tiêu chí 1 được ghi trên cùng máy, cùng cỡ Chương, cùng tải máy với số sau khi sửa
3. **Có test bắt được lỗi, ở cả hai vế**
   **Given** bộ test
   **When** nó chạy
   **Then** một ca đếm số lần tra ô sau một lượt đổi danh sách câu chỉ đổi trạng thái, ở 20 và 200 câu đã sửa; ca này đỏ trên mã cũ vì số lần tra tăng theo số câu
   **And** một ca dựng lại cả lưới với 20 câu đã sửa chưa lưu, và cả 20 ô hiện đúng chữ đã gõ; ca này đỏ nếu gỡ hẳn lượt chép lại
4. **Hoặc: không cần sửa, có bằng chứng**
   **Given** bước tái hiện
   **When** chạy trên mã ở commit bắt đầu vé này
   **Then** tiêu chí 1 đã đạt, với số đo ghi trong Notes. Tiêu chí này thay cho tiêu chí 1–3

## Boundaries

- Chữ đang gõ mà chưa lưu vẫn sống sót qua một lượt dựng lại cả lưới: kéo panel sang chỗ khác, gộp hoặc tách câu, mở lại Chương.
- Khi AI hoặc TM điền vào một câu mà người dịch đang gõ dở chưa lưu, ô vẫn hiện chữ đang gõ.
- Con trỏ không bị đá ra khỏi ô đang gõ.
- Hợp đồng flush và auto-save giữ nguyên (AD-35, FR100).
- Gạch chân quy ước chữ và soát lỗi vẫn vẽ như trước.

## References

- parent — _bmad-output/initiative-auratranslate/tickets/epic-bien-tap-theo-segment/epic-bien-tap-theo-segment.md
- measurement — _bmad-output/initiative-auratranslate/research-go-cham-luoi-subgrid-webkit/handoff-xac-nhan-song-song.md, Pha 1 và Pha 2
- bench — e2e/measure/confirm-trace.e2e.mjs
- bench — e2e/measure/confirm-typing-parallel.e2e.mjs
- constraint — _bmad-output/initiative-auratranslate/architecture-auratranslate/architecture-auratranslate.md, AD-35
- prd — _bmad-output/initiative-auratranslate/prd-auratranslate/prd-auratranslate.md, FR100

## Notes

- Decision: 2026-11-12, Ice — tách khỏi lỗi gõ có dấu trên Windows; vé này sửa cơ chế đã đo trên Mac.
- Decision: 2026-11-12, Ice — ngưỡng: các mốc 50, 100 và 300 không vượt mốc 0 quá 15 ms, trên bản debug.
- Winston đề xuất hai hướng sửa; người dựng đo cả hai và trình Ice chọn. (a) Chỉ chép lại các câu mà lượt cập nhật thật sự thay chữ, cộng toàn bộ khi cả lưới dựng lại. (b) Tỉa tập "đã sửa trong phiên" sau khi lưu.
- ⚠️ Hướng (b) đụng hợp đồng "DOM sở hữu chữ" của Story 2.3. Chữ đã lưu không về lại danh sách câu, nên tỉa là mất bản sống duy nhất. Đi hướng này thì phải đo và trình Ice trước, và có thể phải chia vé.
- Lượt dùng thật trên Windows thuộc spike 17 và bug mở từ spike đó, không thuộc vé này.
