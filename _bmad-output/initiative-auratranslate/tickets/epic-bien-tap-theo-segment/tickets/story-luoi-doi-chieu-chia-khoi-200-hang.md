---
id: 16
type: story
title: "Lưới đối chiếu chia khối 200 hàng"
parent: epic-bien-tap-theo-segment
covers: ["FR16", "FR21"]
after: []
assignee: ""
refined: true
hitl: false
risk: high
---

# Lưới đối chiếu chia khối 200 hàng

## Description

Lưới đối chiếu chia thành các khối khoảng 200 hàng nối tiếp, mỗi khối là một lưới riêng có cột subgrid riêng, thay cho một subgrid trải cả Chương (quyết định #1 của Story 2.5b). Lý do: WebKit tính lại bố cục cả lưới ngay trong mỗi lệnh gõ, nên ở Chương 9.850 câu mỗi phím tốn ~0,5 s và phím xếp hàng (bug 2.15); dời con trỏ có thể chậm vì cùng lượt bố cục lại (giả thuyết của bug 2.14). Người dùng vẫn thấy một lưới liền mạch, mọi hàng vẫn ở trong DOM.

## Requirements

1. **Bản thử đạt số trước khi chuyển cả lưới** (FR16)
   **Given** bản release trên macOS chạy WKWebView của hệ điều hành, một Chương 9.850 câu, lưới đã chia khối, lượt bố cục đầu đã xong
   **When** gõ liên tục 30 giây ở 10 phím/giây vào ô bản dịch, rồi dời con trỏ sang câu khác ba lượt bằng chuột và ba lượt bằng phím điều hướng segment (Story 2.10)
   **Then** chữ của phím cuối hiện ≤ 100 ms sau phím đó, không còn phím chờ, và mỗi lượt dời con trỏ ≤ 50 ms tới hết hai frame
   **And** nếu không đạt, việc dừng ở đây, số được trình Ice, các tiêu chí 2–6 chưa làm
2. **Vùng chọn nhiều câu qua ranh giới khối** (FR16)
   **Given** một Chương nhiều khối
   **When** kéo chọn từ ô bản dịch của một khối sang ô bản dịch của khối kế, hoặc chọn trọn Chương
   **Then** vùng chọn nằm trọn trong cột bắt đầu kéo, gồm đúng các câu trong khoảng theo segment id; không ô nào của cột khác được tô; mọi lệnh dùng vùng chọn nhiều câu (Story 4.9) cho kết quả như trước
3. **Tra cứu từ cột nguyên văn ở mọi khối** (FR21)
   **Given** một câu ở bất kỳ khối nào
   **When** bôi đen một cụm ở cột nguyên văn
   **Then** panel Lookup tra ngay, như ở khối đầu
4. **Lưới vẫn trông như một**
   **Given** mọi khối của Chương
   **When** lưới hiện
   **Then** mỗi hàng nguyên văn và bản dịch thẳng nhau như nay; ở hai phía mỗi ranh giới khối, vị trí và chiều cao hàng lệch ≤ 1 px so với bản hiện tại, đo trong một ca e2e đã có; và lượt bố cục đầu không chậm hơn bản hiện tại trên cùng máy
5. **Điều hướng và e2e đi qua mọi khối**
   **Given** một câu đích ở khối khác khối đang có con trỏ
   **When** dùng điều hướng segment (Story 2.10), hoặc chạy các spec e2e của lưới
   **Then** con trỏ tới đúng câu đó và được cuộn vào; các spec e2e của lưới xanh, chỉ đổi cách tìm hàng trong helper, không đổi điều kiện
6. **Có test giữ hình dạng chia khối**
   **Given** bộ test
   **When** nó chạy
   **Then** có một ca đỏ nếu lưới quay về một subgrid trải cả Chương hoặc một khối vượt cỡ khối; cỡ khối là một hằng có tên mà ca test đọc; có ca cho Chương ít hơn một khối và cho khối cuối lẻ; và cổng đếm bề mặt vùng chọn của lưới (Kiểm F) đếm đúng thiết kế mới, kèm lý do ghi trong cổng

## Boundaries

- Mọi hàng vẫn ở trong DOM; không ảo hoá.
- `⌘Z` gốc trong ô đang gõ, gõ bằng bộ gõ tiếng Việt, hợp đồng flush (Story 2.3) giữ nguyên.
- Ngoài lưới chỉ đọc vùng chọn nhiều câu theo segment id, như hiện nay.
- Gộp bằng `Backspace` ở đầu ô (Story 2.9), tách, cắt bỏ câu và ngắt đoạn giữ nguyên hành vi ở hàng sát ranh giới khối, và cả khi số hàng đổi làm một khối vượt hoặc hụt cỡ.
- Vạch lề luôn có mặt và dày 3 px (DESIGN.md 🔵 2026-10-23); dải Glossary/Proofreader neo theo câu, hàng ảnh, gộp, tách, cắt bỏ câu và ngắt đoạn giữ nguyên hành vi.

## References

- parent — _bmad-output/initiative-auratranslate/tickets/epic-bien-tap-theo-segment/epic-bien-tap-theo-segment.md, Requirements
- research — _bmad-output/initiative-auratranslate/research-go-cham-luoi-subgrid-webkit/research-go-cham-luoi-subgrid-webkit.md, Executive summary và D4
- architecture — _bmad-output/initiative-auratranslate/architecture-auratranslate/architecture-auratranslate.md, §Deferred, hàng "Chiến lược ảo hoá danh sách dài" (🔵 2026-10-23)
- bug — _bmad-output/initiative-auratranslate/tickets/epic-bien-tap-theo-segment/tickets/bug-go-lien-tuc-thi-ung-dung-tut-lai-sau-phim-go.md, tiêu chí 1
- bug — _bmad-output/initiative-auratranslate/tickets/epic-bien-tap-theo-segment/tickets/bug-doi-con-tro-cham-o-chuong-lon.md, tiêu chí 1
- ux — _bmad-output/initiative-auratranslate/ux-auratranslate/DESIGN.md, vạch lề 3 px (🔵 2026-10-23)
- bench — _bmad-output/implementation-artifacts/2-15-ban-do/ (bàn đo gõ 10 phím/giây trên bản release) và thu-engine/ (trang tổng hợp)

## Notes

- Decision: 2026-10-23, Ice — chia khối ~200 hàng, giữ subgrid trong khối, không ảo hoá; ghi ở hàng Deferred của spine, không thành AD.
- Decision: 2026-10-23, Ice — đo ít, đúng mục tiêu: bản thử của tiêu chí 1 chạy một phiên gõ và ba lượt dời con trỏ. Lượt đo nghiệm thu cuối chạy một lần, đúng theo tiêu chí 1 của bug 2.14 và 2.15, rồi đóng hai bug đó bằng chính số ấy.
- Decision: 2026-10-23, Ice — bản thử của tiêu chí 1 là bước đầu trên `GridPanel` thật: chỉ chia khối phần bố cục, chưa sửa vùng chọn và helper e2e, rồi đo. Đạt thì làm tiếp tiêu chí 2–6 trên cùng mã; không đạt thì dừng và không commit phần chia khối.
- Decision: 2026-10-23, Ice — trước khi chia khối, đo thêm ba lượt dời con trỏ mỗi đường (chuột, phím điều hướng) trên bản release hiện tại ở 9.850 câu, làm số "trước" cho tiêu chí 2 của bug 2.14. Số "trước" của 2.15 đã có trên bản release (`043f3bc`). Tiêu chí 3 của hai bug đã có ca test (`gridCaretMoveCost`, `gridTypingCost`).
- Open question: cỡ khối 200 hàng mới đo trên trang tổng hợp và chỉ cho đường gõ; dời con trỏ và vùng chọn qua ranh giới chưa đo ở cỡ nào. Bản thử của tiêu chí 1 ghi cỡ đã dùng.
- Decision: 2026-10-23, Ice — vùng chọn luôn nằm trong một cột, kể cả khi kéo qua ranh giới khối. Vùng chọn đi theo thứ tự DOM nên sẽ đi qua các cột nguyên văn của khối kế nếu không được giữ lại; dùng một bề mặt bọc cả lưới hay nhiều bề mặt là việc của thiết kế story này.
- Risk high vì story đổi cấu trúc lưới chính và bề mặt vùng chọn dùng chung. Kiểm thêm ngoài tiêu chí: Ice dùng thật một Chương lớn, gõ và chọn qua ranh giới khối, trước khi Epic 2 đóng (mục nợ `Chủ: Epic 2` khi story xong).
- Nếu việc vượt một phiên làm việc, người dựng tách tiêu chí 2–6 thành story kế trước khi làm tiếp.
