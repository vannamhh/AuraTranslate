# Đo chuột thật Story 14.2

Đo 2026-10-30 trên WKWebView 605.1.15, bản debug `--features wdio`, Vite mới, macOS. `cliclick` phát sự kiện vào cửa sổ app, mọi sự kiện chuột ghi `isTrusted = true`. Chạy lại: `npx wdio run e2e/wdio.conf.mjs --spec _bmad-output/implementation-artifacts/14-2-ban-do/chuot-that.e2e.mjs`.

Ca: caret ở câu A, chuột phải vào câu B.

| Cột | Thứ tự sự kiện | Sau chuột phải | Đóng bằng phím `Q` |
|---|---|---|---|
| Bản dịch | `mousedown` · `focusout` A · `focusin` B · `contextmenu` · `selectionchange` · `mouseup` trên menu | Caret B, tiêu điểm B, WebKit bôi đen `epsilon` | Caret về A, `Q` vào cuối chữ của A, chữ của B nguyên vẹn |
| Nguyên văn | `mousedown` · `contextmenu` · `selectionchange` · `mouseup` trên menu | Caret B, tiêu điểm vẫn ở A, WebKit bôi đen `甲` dù `mousedown` đã chặn | Caret về A, `Q` vào đúng vị trí cũ giữa chữ |
| Số câu | Như nguyên văn | Caret B, tiêu điểm vẫn ở A, vùng chọn là một dấu xuống dòng | Như nguyên văn |

Các ca khác:
- Chọn Cắt bỏ câu từ cột số: `focusout` A · `focusin` B, caret ở đầu ô B, gõ tiếp vào B.
- Chuột phải trong vùng đã bôi đen ở nguyên văn: vùng chọn giữ nguyên, caret sang câu đó, tiêu điểm không đổi.
- `Cmd`+bấm đặt điểm cắt ở câu 4 rồi chuột phải câu 5: Tách câu `disabled`, `title` nói câu 4.
- `Esc` thật qua System Events: menu đóng, caret về câu trước. `cliclick kp:esc` không tạo `keydown` nào.
- Xác nhận chọn từ nguyên văn: tiêu điểm A · B · C, caret sang câu kế. Câu đích lúc đo đã bị cắt bỏ nên bộ đếm không đổi, chưa khẳng định được trạng thái đã ký.

Chưa đo bằng chuột thật: Xác nhận trên câu chưa cắt bỏ · `Esc` giữ điểm cắt · bấm ra ngoài.

⚠️ Cửa sổ app mở dưới cửa sổ đang hoạt động thì `cliclick` rơi vào cửa sổ khác. Khi đo không để app khác giữ tiêu điểm. Lần chạy đầu hiệu chỉnh đúng; các lần sau `frontmost` không ổn định.
