---
id: 17
type: spike
title: "Gõ có dấu bằng EVKey ở Song song trên Windows tốn thời gian ở đâu"
parent: epic-bien-tap-theo-segment
covers: []
after: []
assignee: ""
refined: true
hitl: true
risk: medium
---

# Gõ có dấu bằng EVKey ở Song song trên Windows tốn thời gian ở đâu

## Description

Trên Windows (MSI v1.0.2), Ice gõ tiếng Việt có dấu bằng EVKey, cả Telex lẫn VNI, ở kiểu Song song Hán Việt. Sau một lúc gõ thì chậm. Gõ không dấu ở cùng chỗ vẫn nhanh. Ice đã báo đúng hình dạng này ngày 2026-11-06. Research D6–D11 khi đó loại được cỡ DOM và ruby trên Blink 156, nhưng không tìm ra nguyên nhân. Mọi bàn đo từ đó tới nay đều chạy trên Mac và gõ ASCII không qua bộ gõ, nên chưa bàn nào tái hiện được lỗi.

Câu hỏi: trên chính máy đó, mỗi phím có dấu sinh bao nhiêu sự kiện, và thời gian dồn vào đâu? Bug sửa lỗi Windows sẽ được mở từ câu trả lời này. Bug 18 không chờ spike này.

## Approach

1. Ice đặt biến môi trường người dùng `WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS=--remote-debugging-port=9222 --remote-allow-origins=*`, rồi mở bản MSI v1.0.2 đang cài. Không cần dựng bản mới.
2. Trong Edge trên cùng máy, Ice mở `edge://inspect`, chọn trang AuraTranslate, mở tab Performance.
3. Mở một Chương ở Song song và gõ có dấu cho tới khi chậm. Tại cùng chỗ đó, Ice ghi ba vệt, mỗi vệt khoảng 10 giây:
   - (a) có dấu ở Song song;
   - (b) không dấu ở Song song;
   - (c) có dấu ở kiểu chữ thường.
4. Ice chụp `edge://gpu`, bốn hàng: Compositing, Rasterization, Problems Detected, Active GPU.
5. Ice thêm `--disable-renderer-accessibility` vào biến môi trường, khởi động lại app, và ghi lại xem gõ có dấu còn chậm không.
6. Một agent đọc ba vệt. Với mỗi vệt, nó ghi: số sự kiện mỗi phím (`keydown`, `beforeinput`, `input`, `selectionchange`, composition), ms mỗi phím, và ba khoản tốn nhất (hàm JS có tên, Recalculate Style, Layout, Paint, Composite).

Hộp thời gian: khoảng 45 phút của Ice trên máy Windows, cộng một phiên agent đọc vệt. Đủ tốt khi vệt (a) và vệt (b) khác nhau ở một khoản có tên, và khoản đó giải thích được độ chậm.

## Acceptance Criteria

1. **Ba vệt và ảnh `edge://gpu` được lưu**
   Verify: thư mục `_bmad-output/initiative-auratranslate/research-go-cham-luoi-subgrid-webkit/windows-trace/`, hoặc chỗ Ice chọn theo Open question bên dưới
2. **Có bảng so ba ô (a), (b), (c)**, kèm kết quả của lượt `--disable-renderer-accessibility`
   Verify: một mục mới trong `_bmad-output/initiative-auratranslate/research-go-cham-luoi-subgrid-webkit/research-go-cham-luoi-subgrid-webkit.md`
3. **Nguyên nhân có tên, và bug sửa đã được mở trong Epic 2**, hoặc ghi rõ vì sao chưa mở được
   Verify: `tickets.toml` của Epic 2
4. **Ice chấp nhận câu trả lời**
   Verify: một dòng `Decision:` trong Notes của vé này

## References

- parent — _bmad-output/initiative-auratranslate/tickets/epic-bien-tap-theo-segment/epic-bien-tap-theo-segment.md
- research — _bmad-output/initiative-auratranslate/research-go-cham-luoi-subgrid-webkit/research-go-cham-luoi-subgrid-webkit.md, D8–D11 và khuyến nghị 12–14
- measurement — _bmad-output/initiative-auratranslate/research-go-cham-luoi-subgrid-webkit/handoff-xac-nhan-song-song.md, Pha 1 và Pha 2 (số đo trên Mac, chỉ phím ASCII)

## Notes

- Decision: 2026-11-12, Ice — lỗi chỉ khi gõ có dấu bằng EVKey (Telex và VNI) ở Song song trên Windows, sau một lúc gõ; gõ không dấu vẫn nhanh.
- Decision: 2026-11-12, Ice — đo trên Windows trước, sửa sau (phương án Y), vì lỗi đã lặp lại nhiều lần mà chưa từng có số đo trên máy đó.
- Nghi can từ mã, chưa phải kết luận: `onEditKeydown` hẹn một lượt vẽ lại gạch chân toàn lưới sau mỗi `keydown` (`src/panels/GridPanel.vue`, ~1723); EVKey làm ra một chữ có dấu bằng phím xoá rồi gõ lại, nên một phím thành nhiều sự kiện; DOM ruby của Song song làm mỗi lượt quét lưới đắt hơn.
- Open question: một vệt có thể nặng hàng chục MB. Commit cả vệt vào repo, hay chỉ commit bảng tóm tắt và giữ vệt ngoài repo?
