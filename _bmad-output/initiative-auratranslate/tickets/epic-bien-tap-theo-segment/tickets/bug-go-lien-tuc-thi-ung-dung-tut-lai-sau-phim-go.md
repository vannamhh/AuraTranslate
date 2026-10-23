---
id: 15
type: bug
title: "Gõ liên tục thì ứng dụng tụt lại sau phím gõ"
parent: epic-bien-tap-theo-segment
covers: []
after: [16]
assignee: ""
refined: true
hitl: false
risk: medium
severity: P2
---

# Gõ liên tục thì ứng dụng tụt lại sau phím gõ

## Description

Khi người dịch gõ liên tục vào ô bản dịch, chữ hiện ra chậm dần so với phím gõ. Phím không mất mà bị xếp hàng, và ứng dụng chỉ đuổi kịp sau khi người dịch ngừng gõ. Trong lúc đó, một phím tắt bấm ngay sau phiên gõ, như `Mod+Enter` để xác nhận câu, không có tác dụng gì trong nhiều giây, nên người dịch tưởng ứng dụng bị treo. Ở nhịp người gõ nhanh, chữ của mỗi phím và phím tắt bấm sau đó phải có tác dụng ngay, dù đã gõ bao lâu và Chương lớn tới đâu.

## Reproduction

1. Dựng bản ứng dụng thật trên macOS (WKWebView).
2. Mở một Chương 122 câu (thang `m` của bàn đo Story 2.4). Chờ lượt bố cục đầu tiên của lưới chạy xong.
3. Bấm vào ô bản dịch của một câu rồi gõ liên tục 30 giây ở nhịp 10 phím/giây, tức một phím mỗi 100 ms, không ngừng. Ngay sau phím cuối, bấm `Mod+Enter` khi con trỏ vẫn ở ô đó, để xác nhận đúng câu vừa gõ.
4. Với mọi phím, đo khoảng từ `event.timeStamp` của phím đó tới frame đầu tiên có chữ của phím đó trên màn hình. Đếm số phím còn chờ xử lý lúc gửi phím cuối. Đo khoảng từ `Mod+Enter` tới lúc câu đổi sang trạng thái đã xác nhận trên màn hình.
5. Lặp lại bước 2–4 trên một Chương 9.850 câu (Chương của bàn đo Story 2.5b). Mỗi cỡ chạy ít nhất ba phiên.

- Thực tế (2026-08-19, bản release, mã tháng 8, Story 2.4): Chương 122 câu, bàn đo gõ bằng `osascript` khoảng 40 phím/giây (suy ra từ 1.004–1.233 sự kiện `input` trong 30 giây, bảng §"Số của bốn phiên chạy trọn"), bốn phiên `n=1`, máy chạy lẫn `cargo build`. Phiên gõ 30 giây kéo dài 40,7 / 58,8 / 70,1 / 45,4 giây. Sau khi ngừng gõ, bộ đếm phím của bàn đo nhảy 3 → 13 ở một phiên và tới 7 ở phiên khác, tức phím bị xếp hàng chứ không mất.
- Thực tế (2026-10-22, bản debug nạp Vite dev, WKWebView 605.1.15, Chương 9.850 câu, Story 2.14): một phím gõ rời mất 761 / 661 / 711 / 762 ms tính tới hết hai frame. Sau thử nghiệm `v-memo` của 2.14 còn 577–795 ms. Bản debug chạy Vue bản dev, chưa ai đo bản release chênh bao nhiêu; số này cũng không đo một phiên gõ liên tục.
- Mong đợi: chữ của phím cuối hiện ≤ 100 ms sau phím đó, và `Mod+Enter` có tác dụng ≤ 100 ms, ở cả hai cỡ Chương.
- Chưa ai đo ở nhịp 10 phím/giây. Số tháng 8 đo ở nhịp gấp bốn lần nhịp người và trên lưới cũ.

## Cause Hypothesis

Phím xếp hàng khi việc làm cho mỗi sự kiện phím tốn nhiều thời gian luồng chính hơn khoảng cách giữa hai phím. WebKit không bỏ sự kiện phím nào, nên hàng chờ dài ra theo độ dài phiên gõ. Chi phí một phím gõ rời (~700 ms ở 9.850 câu, bản debug) không cộng dồn được theo số phím, vì nó gồm hai loại việc:

- Việc theo sự kiện, chạy một lần cho mỗi phím: Vue cập nhật sau từng sự kiện `input`, vì mỗi sự kiện là một task riêng. Story 2.14 đo được `PanelFrame` dựng lại cả slot lưới (~225 ms ở 9.850 câu) vì template đọc bảng chữ đang sửa trên từng hàng, mà mỗi phím gõ thay cả bảng đó. Loại việc này là nghi can chính của hàng chờ: 225 ms mỗi phím so với 100 ms giữa hai phím.
- Việc theo frame, chạy một lần cho cả lô phím chờ giữa hai frame: WebKit tính lại bố cục cả lưới `subgrid` (~390 ms khi dời con trỏ). Loại này làm tụt số frame mỗi giây nhưng tự gộp lại, nên nghi là không làm hàng chờ dài ra. Chữ gõ thêm có thể đổi chiều cao hàng, và phím đầu tiên ở một câu rỗng đổi trạng thái vạch lề.

Chưa ai tách hai loại này trên đường gõ. Bản sửa vạch lề của 2.14 đánh vào loại thứ hai, nên có thể không đổi gì cho đường gõ. Ở 122 câu, frame giữa (p50) tháng 8 là 25–44 ms; nếu cỡ này không đạt ở nhịp người, còn một chi phí cố định mỗi phím chưa có tên. Story 2.4 đo được `busy = 0` (không lượt checkpoint nào bị chặn), nên auto-save chưa phải nghi can đầu, nhưng chưa bị loại ở nhịp người.

## Acceptance Criteria

1. **Không còn xếp hàng phím ở nhịp người gõ nhanh**
   **Given** bản dựng thật trên macOS, mở một Chương 122 câu, rồi một Chương 9.850 câu, lượt bố cục đầu tiên đã xong
   **When** gõ liên tục 30 giây vào ô bản dịch ở nhịp 10 phím/giây rồi bấm `Mod+Enter`, ít nhất ba phiên mỗi cỡ
   **Then** mỗi phiên, chữ của phím cuối hiện trên màn hình ≤ 100 ms sau phím đó, không còn phím nào chờ xử lý, và câu đổi sang trạng thái đã xác nhận ≤ 100 ms sau `Mod+Enter`
   **And** độ trễ của mọi phím được ghi kèm p50, p95 và độ trễ theo thời gian trong phiên, cùng bản dựng, phiên bản WebKit, số câu, cách gửi phím và tải máy
2. **Có số trước khi sửa, cùng điều kiện với số sau khi sửa**
   **Given** mã ở commit bắt đầu vé này, chưa sửa gì
   **When** chạy bước tái hiện
   **Then** các số của tiêu chí 1 và chi phí một phím gõ rời ở cả hai cỡ Chương được ghi lại trên cùng máy, cùng cỡ Chương, cùng tải máy với số sau khi sửa; chi phí một phím gõ rời được ghi cả trước lẫn sau, không có ngưỡng
3. **Có test bắt được lỗi này**
   **Given** bộ test
   **When** nó chạy
   **Then** có một ca đếm số hàng mà lưới phải xét lại sau một sự kiện `input`, gồm hàng dựng lại và hàng chỉ so lại `v-memo` khi cả slot lưới render lại (đếm công việc, không đo thời gian), ở Chương 20 câu và 200 câu; ca này đỏ trên mã trước khi sửa vì số đó tăng theo số câu, và xanh sau khi sửa
4. **Hoặc: không cần sửa, có bằng chứng**
   **Given** bước tái hiện
   **When** chạy trên mã ở commit bắt đầu vé này
   **Then** cả hai cỡ Chương đã đạt tiêu chí 1, với số đo ghi trong Notes. Tiêu chí này thay cho tiêu chí 1–3

## Boundaries

- Không phím nào mất, và chữ hiện đúng thứ tự gõ.
- Hợp đồng flush và auto-save của Story 2.3 giữ nguyên (FR100).
- Lượt gõ không đổi mốc so xuất xứ (AD-47).
- Gõ bằng bộ gõ tiếng Việt (chuỗi sự kiện composition) vẫn ra đúng chữ.
- Vạch lề của câu đang gõ đổi theo trạng thái như trước. Các Boundaries của Story 2.14 (vạch lề, vùng chọn nhiều segment, điều hướng, tra cứu ở cột nguyên văn) giữ nguyên.

## References

- parent — _bmad-output/initiative-auratranslate/tickets/epic-bien-tap-theo-segment/epic-bien-tap-theo-segment.md
- measurement — _bmad-output/initiative-auratranslate/deferred-work.md, mục "App tụt lại ~15 giây sau một phiên gõ 30 giây"
- measurement — _bmad-output/initiative-auratranslate/tickets/epic-bien-tap-theo-segment/tickets/story-mui-tham-do-do-nfr18-va-nfr2-dong-thoi-plan.md, §"BÀN ĐO NFR2 NAY CHẠY TRỌN VÒNG VÀ LẶP LẠI ĐƯỢC" và §"Số của bốn phiên chạy trọn"
- measurement — _bmad-output/initiative-auratranslate/tickets/epic-bien-tap-theo-segment/tickets/bug-doi-con-tro-cham-o-chuong-lon-plan.md, Implementation Notes (số một phím gõ ở 9.850 câu, đo tách `PanelFrame` và bố cục `subgrid`)
- constraint — _bmad-output/initiative-auratranslate/architecture-auratranslate/architecture-auratranslate.md, AD-47
- bench — _bmad-output/implementation-artifacts/2-4-ban-do/build-bench.sh (bản release kèm script đo)
- sample data — _bmad-output/implementation-artifacts/2-4-ban-do/chapters/ladder-m.txt, Chương 122 câu
- bench — _bmad-output/implementation-artifacts/2-14-ban-do/do-doi-con-tro.e2e.mjs, phép đo một phím gõ
- prd — _bmad-output/initiative-auratranslate/prd-auratranslate/prd-auratranslate.md, §7.1 Hiệu năng, NFR2; FR100

## Notes

- Decision: 2026-10-23, Ice — báo cáo lấy từ mục nợ của Story 2.4, không phải từ một lượt dùng thật.
- Decision: 2026-10-23, Ice — đường gõ là vé riêng, không gộp vào 2.14. Mục nợ "App tụt lại ~15 giây…" đổi Chủ từ Story 2.4 sang vé này; Story 2.4 vẫn chỉ đo gõ kèm auto-save.
- Decision: 2026-10-23, Ice — "hết tụt" nghĩa là ở nhịp 10 phím/giây trong 30 giây, chữ của phím cuối và `Mod+Enter` bấm sau đó có tác dụng ≤ 100 ms, ở cả 122 và 9.850 câu. Chi phí mỗi phím được ghi, không có ngưỡng.
- Decision: 2026-10-23, Ice — 2.14 đã `blocked` và ngừng sửa lưới, sẽ đo lại sau; vé này không chờ 2.14, nên `after` để rỗng.
- Decision: 2026-10-23, Ice — 10 phím/giây là nhịp của người gõ nhanh (khoảng 100 từ/phút). Nhịp ~40 phím/giây của bàn đo tháng 8 không phải mốc nghiệm thu.
- Decision: 2026-10-23, Ice — như 2.14, vé này không nghiệm thu NFR2 (NFR2 chỉ nói về lúc auto-save chạy), nên `covers` để rỗng; phiên NFR2 30 phút vẫn thuộc Story 2.4.
- Decision: 2026-10-23, Ice — risk medium vì gõ là thao tác chính và phím xếp hàng cũng là chữ chưa vào bộ đệm, nhưng không phím nào mất; severity P2 vì chưa ai thấy lỗi ở nhịp người trên Chương nhỏ.
- Chương 122 câu là cỡ đối chứng: ở nhịp người nó có lẽ đã đạt. Nó đạt không có nghĩa là đã sửa được một nửa; nó không đạt thì có một chi phí cố định mỗi phím, và đó là một phát hiện cần ghi.
- Chương 9.850 câu là Chương tổng hợp (khoảng 4,9 ký tự mỗi câu, plan Story 2.4), dùng làm trần chịu tải để so với số của 2.14, không phải hình dạng một Chương thật.
- Open question: mọi số gõ của Story 2.4 là chữ ASCII, vì `osascript keystroke` làm hỏng chữ có dấu; đường IME là một mục nợ riêng của 2.4. Một chữ có dấu gõ qua Telex sinh nhiều sự kiện hơn một phím ASCII, nên chưa rõ nhịp 10 phím/giây ASCII nặng hơn hay nhẹ hơn nhịp gõ tiếng Việt thật. Vé này đo bằng phím ASCII và ghi cách gửi phím kèm số đo; số cho chữ có dấu chờ mục nợ IME của Story 2.4.
- Decision: 2026-10-23, Ice — phần sửa lưới của 2.14 được commit trước, ở `43729b3`; số "trước" của tiêu chí 2 đo trên commit bắt đầu vé này, tức đã gồm phần đó.
- Nếu cách sửa phải đổi cấu trúc lưới và vượt một phiên làm việc, người dựng báo lại để chia vé trước khi làm tiếp. Nếu cách sửa là ảo hoá hàng, câu hỏi mở của 2.14 về hàng §Deferred "Chiến lược ảo hoá danh sách dài" của spine áp dụng ở đây.
- Vế Blink/WebView2 chưa đo; theo luật của dự án, phần kiểm chỉ làm được trên Windows đi vào B7.
