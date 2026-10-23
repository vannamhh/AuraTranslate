---
id: 14
type: bug
title: "Dời con trỏ chậm ở Chương lớn"
parent: epic-bien-tap-theo-segment
covers: []
after: [16]
assignee: ""
refined: true
hitl: false
risk: medium
severity: P2
---

# Dời con trỏ chậm ở Chương lớn

## Description

Trên một Chương 9.850 câu, mỗi lần người dịch bấm sang câu khác trong lưới, con trỏ mất 706–770 ms mới tới, tức khoảng 15 lần trần 50 ms của một frame. Đây là thao tác người dịch làm nhiều nhất. Ở 2.000 câu, con số này vẫn là 161–226 ms. Một lượt dời con trỏ phải xong trong một frame, dù Chương lớn tới đâu.

## Reproduction

1. Dựng bản ứng dụng thật trên macOS (WKWebView), rồi mở một Chương 9.850 câu trong lưới đối chiếu.
2. Chờ lượt bố cục đầu tiên của lưới chạy xong. Bàn đo của 2.5b từng đọc ra 26.927 ms ở lượt đầu, vì nó bấm đồng hồ khi mới có ô đầu tiên.
3. Bấm vào ô bản dịch của một câu, rồi bấm sang ô bản dịch của một câu khác. Đo từ cú bấm tới hết hai frame tiếp theo. Lặp lại ít nhất ba lượt.

- Thực tế (2026-08-15, WKWebView 605.1.15, macOS 15.6, bản dựng thật, Story 2.5b): 770 / 706 / 767 ms ở 9.850 câu; 226 / 173 / 195 / 189 / 161 ms ở 2.000 câu. Lưới có 49.256 node DOM ở 9.850 câu.
- Mong đợi: ≤ 50 ms mỗi lượt.
- Ngày 2026-10-20, lượt rà nợ xác nhận cơ chế gây chậm vẫn còn ở `bf3f47f` nhưng không đo lại. Từ đó tới nay 2.5c, 2.5d và 2.11 đều đổi lưới mà không ai đo lại, nên 706–770 ms vẫn là số mới nhất.

## Cause Hypothesis

Lưới chưa ảo hoá, và cả lưới là một component dựng mọi hàng: Chương có bao nhiêu câu thì có bấy nhiêu hàng, mỗi hàng năm ô. Template đọc câu đang có con trỏ và chữ đang sửa ngay trên từng hàng, nên mỗi lần con trỏ đổi câu, Vue chạy lại hàm render của toàn lưới. Cùng lượt đó, quy tắc vạch lề (`ruleById`, `ruleClassById`) được tính lại trên toàn danh sách; đây là một phần của chi phí, không phải toàn bộ. Vì vậy chi phí một lượt dời con trỏ tăng theo số câu của Chương, chứ không theo số câu đổi trạng thái (luôn là hai câu: câu con trỏ vừa rời và câu con trỏ tới).

Đường gõ phím có thể chung cơ chế: mỗi phím gõ thay cả bảng chữ đang sửa, mà bảng này cả quy tắc vạch lề lẫn template đều đọc. Chưa ai đo một lượt gõ ở 9.850 câu; số 24–33 ms của 2.5b là `selectionchange` bên trong cùng một ô, lúc con trỏ không đổi câu.

🔵 2026-10-22 — đo tách trong Story 2.14 (plan, Implementation Notes) cho thấy giả thuyết trên chỉ đúng một phần. ~225 ms là `PanelFrame` dựng lại cả slot lưới. ~390 ms là WebKit tính lại bố cục cả lưới `subgrid`, vì vạch lề của câu nhận con trỏ là một phần tử được chèn mới vào lưới. Số ô sửa được, vùng chọn, bộ gõ và CSS của lưới không phải nguyên nhân.

## Acceptance Criteria

1. **Dời con trỏ xong trong một frame ở Chương lớn**
   **Given** một Chương 9.850 câu mở trong bản dựng thật trên macOS, và lượt bố cục đầu tiên đã xong
   **When** người dùng dời con trỏ sang câu khác, ít nhất ba lượt cho mỗi đường: bấm chuột vào ô bản dịch của câu khác, và phím điều hướng tới câu kế tiếp (Story 2.10)
   **Then** mỗi lượt, tính từ cú bấm hoặc phím tới hết hai frame tiếp theo, ≤ 50 ms; số đo được ghi kèm bản dựng, phiên bản WebKit và số câu
2. **Có số trước khi sửa, cùng điều kiện với số sau khi sửa**
   **Given** mã hiện tại, chưa sửa gì
   **When** chạy bước tái hiện, và gõ một phím vào ô bản dịch ở cùng Chương 9.850 câu
   **Then** số dời con trỏ và số một lượt gõ phím trước khi sửa được ghi lại trên cùng máy, cùng cỡ Chương, cùng tải máy với số sau khi sửa; số gõ phím được ghi cả trước lẫn sau, không có ngưỡng
3. **Có test bắt được lỗi này**
   **Given** bộ test
   **When** nó chạy
   **Then** có một ca cho thấy công việc của một lượt dời con trỏ không tăng theo số câu của Chương: ca này đếm công việc (không đo thời gian) ở hai cỡ Chương khác nhau, đỏ trên mã hiện tại và xanh sau khi sửa
4. **Hoặc: không cần sửa, có bằng chứng**
   **Given** bước tái hiện
   **When** chạy trên mã hiện tại
   **Then** một lượt dời con trỏ đã ≤ 50 ms ở 9.850 câu, hoặc báo cáo gốc đo sai, với bằng chứng ghi trong Notes. Tiêu chí này thay cho tiêu chí 1–3

## Boundaries

- Câu nhận con trỏ có vạch lề `primary`, câu con trỏ vừa rời trở về vạch của trạng thái nó, đúng như trước, và vạch đổi trong cùng lượt vẽ mà con trỏ tới, không trễ sau.
- Vùng chọn nhiều segment (Story 4.9) giữ nguyên hành vi, kể cả khi chọn trọn một Chương.
- Chữ đang chờ ghi của mọi câu không mất và vẫn được ghi, kể cả câu đã cuộn ra khỏi màn hình.
- Điều hướng segment (Story 2.10) tới một câu ở ngoài màn hình vẫn đưa con trỏ vào đúng câu đó.
- Bôi đen ở cột nguyên văn vẫn tra ngay (FR21). Gộp, tách, cắt bỏ câu và ngắt đoạn của bản dịch giữ nguyên hành vi.

## References

- parent — _bmad-output/initiative-auratranslate/tickets/epic-bien-tap-theo-segment/epic-bien-tap-theo-segment.md
- measurement — _bmad-output/initiative-auratranslate/deferred-work.md, mục "TASK 8 ĐÃ ĐO — SỐ GIAO CHO STORY 2.4. MỘT ĐƯỜNG VƯỢT TRẦN NFR2 15 LẦN."
- bench — _bmad-output/implementation-artifacts/2-5b-ban-do/do-hang-va-hieu-nang.e2e.mjs, phép đo 9.850 câu và lượt chờ bố cục đầu
- prd — _bmad-output/initiative-auratranslate/prd-auratranslate/prd-auratranslate.md, §7.1 Hiệu năng, NFR2
- constraint — _bmad-output/initiative-auratranslate/architecture-auratranslate/architecture-auratranslate.md, §Deferred, hàng "Chiến lược ảo hoá danh sách dài"
- story — _bmad-output/initiative-auratranslate/tickets/epic-bien-tap-theo-segment/tickets/story-mui-tham-do-do-nfr18-va-nfr2-dong-thoi-plan.md, AC13 (bảng ngưỡng "đòi mở sớm")

## Notes

- Decision: 2026-10-22, Ice — vé này nhận hẳn đường dời con trỏ, tách khỏi Task 6 và AC13 của Story 2.4. Story 2.4 vẫn đo lúc gõ kèm auto-save. Mục nợ ở `deferred-work.md` đổi Chủ từ Story 2.4 sang vé này.
- Decision: 2026-10-22, Ice — trần 50 ms là trần frame mượn từ NFR2. Theo lời PRD, NFR2 chỉ nói về lúc auto-save chạy, nên vé này không nghiệm thu NFR2 và `covers` để rỗng.
- Decision: 2026-10-22, Ice — sau sáu phép đo, hướng sửa là: vạch lề luôn có mặt, `.strip-pad` sửa cùng khuôn nếu đắt, làm lại `v-memo` kèm ca test cho từng giá trị, đo trên bản dựng thật; không ảo hoá hàng.
- Decision: 2026-10-23, Ice — mọi vạch lề dày 3 px, chỉ khác kiểu nét và màu (`confirmed` thành nét đôi), vì đổi bề dày vạch khi dời con trỏ ép tính lại bố cục cả lưới; DESIGN.md mang 🔵 2026-10-23.
- Decision: 2026-10-22, Ice — đặt ở Epic 2; risk medium vì đây là thao tác thường nhất, chậm 15 lần nhưng không mất dữ liệu; severity P2 vì Chương nhỏ vẫn dùng được.
- Mục nợ đã nêu ba hướng thấy được: ảo hoá hàng, tính quy tắc vạch lề tại chỗ thay vì dựng một `Map` cho toàn danh sách, và tách lớp `editing` ra khỏi lượt tính lại. Chưa hướng nào được đo. Quyết định #7 do Ice ký là "không tối ưu mù".
- Open question: nếu cách sửa là ảo hoá hàng, hàng §Deferred "Chiến lược ảo hoá danh sách dài" của spine mở sớm hơn Giai đoạn 3. Spine ghi đây là quyết định trình bày trong một module, nên chưa rõ việc này cần một AD mới do Winston soạn hay chỉ cần sửa hàng đó.
- Đường gõ phím vẫn thuộc Story 2.4 theo quyết định ngày 2026-10-22. Người dựng vé này tự đo và ghi số gõ phím của tiêu chí 2, trước và sau khi sửa, trên cùng bản dựng với số dời con trỏ; 2.4 đọc số đó, không đo thay; nếu số đó vượt 50 ms thì Ice quyết có đưa đường gõ vào vé này hay không.
- Decision: 2026-10-23, Ice — số gõ phím đã vượt 50 ms (Implementation Notes của plan); đường gõ thành bug 2.15, không gộp vào đây và không chờ vé này.
- Nếu cách sửa phải đổi cấu trúc lưới và vượt một phiên làm việc, người dựng báo lại để chia vé trước khi làm tiếp.
- Vế Blink/WebView2 chưa đo; theo luật của dự án, phần kiểm chỉ làm được trên Windows đi vào B7.
