# Bàn đo 14-4: Rà soát và Lịch sử bằng chuột

Bản dựng: binary debug `--features wdio`, Vite của `tauri dev` đang chạy, WKWebView trên máy Ice, 2026-10-31. Chuột thật qua `cliclick`, `Esc` thật qua `osascript`. `alignment_open` và `review_diff` được trả lời ở `window.fetch` vì đường nhập bản Reviewer đi qua hộp thoại tệp gốc; phần sau lượt trả lời là UI thật. Tổng thể là một lượt mỗi ca, trừ đối chứng.

| Đường | Kết quả |
|---|---|
| Ô Review Mode từ ô đang gõ | Mở, tiêu điểm vào panel, Workspace `inert`: phím gõ không tới ô bản dịch |
| Hai nút diff | Tiêu điểm giữ ở `section.panel`, dòng trạng thái đổi |
| Đóng bằng ô, chuột đã rời ô | Dock đóng, tiêu điểm vào ô, caret về câu cũ, offset 3 giữ nguyên, `abc` gõ tiếp thành `abcxy` |
| Lịch sử từ menu, phím khi lớp phủ mở | Trước khi sửa: `abc` thành `abcQ`, chữ lọt xuống ô dưới lớp phủ. Sau khi sửa: không đổi |
| Lịch sử từ menu, `Esc` rồi gõ | Tiêu điểm về ô, caret về câu, `abc` thành `zzabc`: không đè, nhưng caret về đầu ô, như `Mod+H` |
| `Mod+H`, phím khi lớp phủ mở | Không lọt: WebKit tự xoá vùng chọn khi tiêu điểm đổi không qua chuột |
| Thanh Rà soát | Cao 35px, nút 28px canh baseline, không nới thanh |

Đối chứng: 16 lần bấm hai ô preset có sẵn, không dính Review Mode. Bấm lặp đúng một điểm không rê chuột: `click` bắn 3/8. Chuột dời giữa hai ô: 7/8. Lượt mất có `mousedown` mà không có `mouseup`. Cùng triệu chứng thấy một lần ở nút diff (bấm lặp đúng một điểm, ngoài dải tiêu đề), nên chưa kết luận do thanh tiêu đề hay do `cliclick`; chuột người chưa đo.
