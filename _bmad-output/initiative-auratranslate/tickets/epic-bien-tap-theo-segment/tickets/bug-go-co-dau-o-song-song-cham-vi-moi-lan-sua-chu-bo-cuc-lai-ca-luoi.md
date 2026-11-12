---
id: 19
type: bug
title: "Gõ có dấu ở Song song chậm vì mỗi lần sửa chữ bố cục lại cả lưới"
parent: epic-bien-tap-theo-segment
covers: []
after: [17]
assignee: ""
refined: true
hitl: true
risk: high
severity: P1
---

# Gõ có dấu ở Song song chậm vì mỗi lần sửa chữ bố cục lại cả lưới

## Description

Trên Windows, người dịch gõ tiếng Việt có dấu bằng EVKey, Telex hay VNI, ở kiểu Song song Hán Việt. Sau một lúc, chữ hiện chậm hơn phím gõ, và chậm dần. Gõ không dấu thì vẫn theo kịp. Máy Boot Camp của người dùng chậm hơn máy đo. Mỗi chữ gõ ở Song song phải hiện kịp như ở kiểu Nguyên văn, kể cả khi gõ có dấu liên tục.

Bước nghiệm thu cuối cần Ice: cài bản đã sửa trên máy Windows, ghi lại vệt Performance theo cách của spike 17, và gõ thật một lượt.

## Reproduction

1. Trên Windows, cài MSI v1.0.2. Đặt `WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS=--remote-debugging-port=9222 --remote-allow-origins=*`.
2. Mở một Chương ở kiểu Song song. Gõ có dấu bằng EVKey vào ô bản dịch cho tới khi chậm.
3. Từ Edge, vào `edge://inspect`, rồi ghi vệt Performance khoảng 10 giây khi gõ có dấu. Ghi thêm một vệt ở kiểu Nguyên văn để đối chứng.
4. Đọc vệt bằng `layout_trace.py` trong thư mục `windows-trace/` của tài liệu research.

- Thực tế (2026-11-12, máy ảo VMware, vẽ bằng phần mềm, spike 17): mỗi lần sửa chữ có một lần bố cục với 37 đối tượng bẩn nhưng 8.562 đối tượng được bố cục, trung vị 67 ms, p90 101 ms. 145 chữ có dấu sinh 192 lần bố cục. Tác vụ dài nhất 865 ms. Kiểu Nguyên văn: 827 đối tượng, 7 ms.
- Mong đợi: một lần sửa chữ ở Song song chỉ bố cục ô đang gõ và những gì thật sự phải đổi theo nó, nên chi phí không tăng theo cỡ lưới.

## Cause Hypothesis

Blink bố cục lại toàn bộ cây ở mỗi lần sửa chữ, thay vì chỉ ô đang gõ. Ở Song song, mỗi hàng nguồn có hàng chục phần tử ruby, nên cây lớn gấp khoảng mười lần kiểu Nguyên văn, và mỗi lần bố cục đắt gấp khoảng mười lần. EVKey làm ra chữ có dấu bằng phím xoá rồi gõ lại, nên một chữ thành hai lần bố cục trở lên. Khi nhịp gõ vượt sức máy, phím xếp hàng.

Vì sao Blink không giới hạn lượt bố cục vào ô đang gõ thì chưa biết. D11 đo 1,7 ms ở một trang tổng hợp cùng hình dạng lưới trên Chrome macOS, nên điều kiện khiến app thật bố cục lại toàn bộ chưa được tái hiện ngoài app. Các nghi can, chưa đo: cách lưới chia khối và `subgrid` truyền thay đổi chiều cao của một ô lên các track của cả lưới; CSS riêng của app trên ô bản dịch, ô nguồn và ruby; phông CJK.

## Acceptance Criteria

1. **Gõ có dấu ở Song song theo kịp**
   **Given** bản đã sửa, cài từ MSI trên máy ảo Windows đã đo ở spike 17, cùng Chương, kiểu Song song
   **When** Ice gõ có dấu bằng EVKey khoảng 10 giây và ghi vệt Performance như spike 17
   **Then** số đối tượng được bố cục ở mỗi lần sửa chữ không còn bằng cả lưới, và một lần bố cục có trung vị ≤ 16 ms, p90 ≤ 25 ms
   **And** không còn tác vụ nào trên 100 ms do gõ
2. **Có số trước và sau, cùng điều kiện**
   **Given** vệt của spike 17 là số "trước"
   **When** đo lại sau khi sửa trên cùng máy
   **Then** bảng so trước/sau (đối tượng được bố cục, trung vị và p90 một lần bố cục, tác vụ dài) được ghi vào tài liệu research, kèm phiên bản WebView2
3. **Có test hoặc bàn đo bắt được lỗi**
   **Given** bộ test hoặc một bàn đo trong `e2e/measure/`
   **When** nó chạy
   **Then** nó cho thấy số đối tượng được bố cục mỗi lần sửa chữ ở Song song trên mã cũ (tăng theo cỡ lưới) và trên mã mới (không tăng). Nếu không đo được ngoài Windows, ghi lý do và để vệt Windows làm bằng chứng duy nhất
4. **Lượt gõ thật trên máy người dùng**
   **Given** máy Boot Camp nơi lỗi được báo
   **When** Ice gõ có dấu ở Song song quá 100 câu
   **Then** chữ theo kịp phím gõ. Nếu chưa làm được trước khi đóng vé, ghi một mục nợ `Chủ: B7`
5. **Hoặc: không cần sửa, có bằng chứng**
   **Given** bước tái hiện
   **When** chạy trên mã ở commit bắt đầu vé này
   **Then** vệt đã đạt tiêu chí 1, với số đo ghi trong Notes. Tiêu chí này thay cho 1–4

## Boundaries

- Hàng nguồn và hàng bản dịch vẫn thẳng hàng. Chiều cao hàng vẫn theo nội dung dài hơn của hai cột.
- Ruby Hán Việt vẫn hiện đầy đủ, đúng âm, không đè lên hàng kế.
- Vùng chọn nhiều câu, điều hướng, tra cứu ở cột nguyên văn và Auto-Lookup giữ nguyên.
- Chữ gõ có dấu ra đúng ký tự. Không phím nào mất.
- Kiểu Nguyên văn và kiểu xem thứ ba không chậm đi.
- Hợp đồng "DOM sở hữu chữ" và flush (AD-35) giữ nguyên.

## References

- parent — _bmad-output/initiative-auratranslate/tickets/epic-bien-tap-theo-segment/epic-bien-tap-theo-segment.md
- spike — _bmad-output/initiative-auratranslate/tickets/epic-bien-tap-theo-segment/tickets/spike-go-co-dau-bang-evkey-o-song-song-tren-windows.md
- measurement — _bmad-output/initiative-auratranslate/research-go-cham-luoi-subgrid-webkit/research-go-cham-luoi-subgrid-webkit.md, D12 (vệt Windows) và D11 (trang tổng hợp trên Blink 156)
- tool — _bmad-output/initiative-auratranslate/research-go-cham-luoi-subgrid-webkit/windows-trace/layout_trace.py
- constraint — _bmad-output/initiative-auratranslate/architecture-auratranslate/architecture-auratranslate.md, AD-35
- prior — _bmad-output/initiative-auratranslate/tickets/epic-bien-tap-theo-segment/tickets/story-luoi-doi-chieu-chia-khoi-200-hang.md

## Notes

- Decision: 2026-11-12, Ice — hướng sửa là (c): tìm vì sao Blink bố cục lại toàn bộ cây thay vì chỉ ô đang gõ, rồi sửa đúng chỗ đó. Không chọn (a) thu nhỏ khối ở Song song hay (b) `content-visibility` cho ruby ngoài màn hình làm bước đầu.
- Decision: 2026-11-12, Ice — làm vé này trước bug 18.
- Assumption: ngưỡng ở tiêu chí 1 (trung vị ≤ 16 ms, p90 ≤ 25 ms trên máy ảo) do Sofia đề xuất, chờ Ice xác nhận.
- Open question: bản MSI chứa bản sửa đến tay Ice bằng đường nào? (a) ra tag `v1.0.3`; (b) thêm bước tải `.msi` lên ở lượt chạy tay `release.yml`.
- risk high vì vé có thể đổi cách lưới truyền thay đổi chiều cao hàng, chạm vạch lề, vùng chọn và ruby. Kiểm thêm ngoài tiêu chí: chạy lại `e2e/specs/hanviet-parallel-row-height.e2e.mjs` và các ca vùng chọn qua ranh giới khối.
- Nếu tìm ra rằng sửa đúng chỗ phải đổi một bất biến của lưới (subgrid, chia khối 200 hàng), đó là một AD mới do Winston soạn, không phải một dòng mã. Báo lại để chia vé.
