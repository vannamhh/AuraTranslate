# Bàn đo 14-10: rời cửa sổ khi menu câu mở, và caret sau Esc ở Lịch sử

Bản dựng: binary debug `cargo build --locked --features wdio` với `TAURI_CONFIG` đặt `devUrl` về Vite riêng ở cổng 1421, WKWebView trên máy Ice, 2026-10-31. Vite của `tauri dev` (Ice) ở 1420 đã giữ tem `?t=` của nhiều lượt sửa trong ngày (`editorPanelState.ts?t=…`), nên cầu reset của e2e trượt ở Tác phẩm thứ hai; bàn đo chạy trên Vite 1421 dựng mới, wdio vẫn dùng lại 1420 chỉ để kiểm đồ thị. Chuột phải qua `cliclick rc:`, bấm trái qua `dd:`/`du:` giữ 90 ms, `osascript` cho Cmd-Tab, `Esc` (key code 53) và `⌘H`; `ensureFront()` trước mỗi cú; wdio chỉ đọc trạng thái. Cửa sổ 2600×1800 vật lý. Cách chạy: dựng `npx vite --port 1421 --strictPort` ở gốc repo; đặt `TAURI_CONFIG='{"build":{"devUrl":"http://localhost:1421"}}'` rồi `cargo build --locked --features wdio --manifest-path src-tauri/Cargo.toml`; chạy `npx wdio run e2e/wdio.conf.mjs --spec _bmad-output/implementation-artifacts/14-10-ban-do/chuot-that.e2e.mjs`, không sửa `wdio.conf.mjs`. Số thô ở `ket-qua.json`.

Quần thể: một Tác phẩm nhập song ngữ 9 cặp (`Ban dich N.`), một Chương, không câu nào xác nhận. Tải máy 2,7 đến 3,0 (`uptime`) lúc chạy lượt chốt; lượt đầu cùng tải, hỏng ở bàn đo (xem Giới hạn) chứ không ở sản phẩm.

| # | Phép đo | Kết quả | Số đọc |
|---|---|---|---|
| A | Menu câu mở (chuột phải thật ở câu 7, 8, 9), Cmd-Tab đi rồi về, bấm `Cắt bỏ câu` | Đạt 3/3, KHÔNG tái hiện | `document.hasFocus()` false lúc ở ngoài; quay về sau 340–363 ms; menu còn mở, caret vẫn ở đúng câu nhắm (7, 8, 9); cú bấm có đủ `mousedown`, `mouseup`, `click`; đúng câu bị cắt bỏ (`omitted` chỉ đổi ở câu nhắm), menu đóng |
| A | Sự kiện tiêu điểm trong cả lượt | `focusout` có một lần, KHÔNG phải do Cmd-Tab | `blur`+`focusout` ở ô câu 1 và `focusin` ở ô câu nhắm đến từ lúc mở menu (menu đưa tiêu điểm vào ô câu nhắm); lúc Cmd-Tab chỉ có `blur` của ô câu nhắm rồi `blur` của cửa sổ, không `focusout` |
| B | Không có menu, Cmd-Tab đi rồi về, gõ `q` | Đạt | caret 4 trước, 4 sau; `Ban dich 2.` thành `Ban qdich 2.`; `activeElement` là ô dịch |
| C | `⌘H` thật, `Esc`, gõ `zz` | Đạt | caret 4 → 4; `Ban dich 4.` thành `Ban zzdich 4.`; lớp phủ đóng, `activeElement` là ô dịch |
| D | Chuột phải thật trong ô (WebKit tự bôi `dich`, 4–8), mục `Lịch sử phiên bản`, `Esc`, gõ `zz` | Đạt | lúc mở vùng chọn `dich` (không thu về); sau `Esc` caret thu về một điểm ở 8; `Ban dich 5.` thành `Ban dichzz 5.`, không chữ nào bị đè; cú bấm mục đủ ba sự kiện |

Kết luận phát hiện 14.2 (rời cửa sổ khi menu mở): WebKit không bắn `focusout` khi cửa sổ mất tiêu điểm, nên `onColumnFocusOut` không đặt caret `null` và menu vẫn chạy đúng câu; không thêm nghe `blur` của cửa sổ. Phạm vi: macOS 15, WKWebView, 3 lượt, một cú Cmd-Tab mỗi lượt; WebView2 chưa đo (`B7`).

Kết luận phát hiện 14.4 (`Esc` đưa caret về đầu ô): đã sửa, bàn đo C và D đọc đúng chỗ. Trước bản vá e2e đọc `zzabc` ở đường `⌘H` (xem `grid-sentence-menu.e2e.mjs`).

## Giới hạn và điều chưa đo

- Lượt bàn đo đầu (Tác phẩm 6 cặp) hỏng ở bàn đo: sau `Cắt bỏ câu` ở lượt A1 câu 3 giữ cờ cắt bỏ nên vòng A2, A3 tìm nhãn `Cắt bỏ câu` mà menu hiện `Đưa câu trở lại`. Đã đổi sang mỗi vòng một câu khác (9 cặp). Không phải lỗi sản phẩm.
- Cmd-Tab đo bằng `osascript` (`key code 48 using command down`), hai cú liên tiếp; Cmd-Tab của người, và đổi sang một ứng dụng khác chứ không về Finder, chưa đo. Menu bị đóng bởi cuộn hay bấm ngoài khi quay về thì thuộc ca khác.
- Chuột phải trong ô lúc chưa có caret (lần đầu vào ô) chưa đo.
- `typeText` đi qua `cliclick t:`; `Esc` và `⌘H` qua `osascript`, vì `kp:esc` của `cliclick` không tạo `keydown` (xem `14-9-ban-do/ket-qua.md`).
