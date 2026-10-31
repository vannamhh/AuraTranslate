# Bàn đo 14-6: tầng khối nhập URL bằng chuột thật

Bản dựng: binary debug `--features wdio`, Vite của `tauri dev` đang chạy, WKWebView trên máy Ice, 2026-10-31. Chuột và phím thật qua `cliclick`. Quần thể: hai trang `http://127.0.0.1`, mỗi trang 31 khối, đi đường URL thật qua Rust. Số thô ở `ket-qua.json`.

| Ca | Kết quả |
|---|---|
| Bấm khối 14 trong danh sách đã cuộn | Nhắm khối 14, `scrollTop` 1997 không đổi, khối dời 0 px, tiêu điểm ở `OL`, `j` tới khối 15 |
| Bấm khối đang nhắm khi tiêu điểm ở `panel` | Tiêu điểm sang `OL`, `j` chạy tiếp |
| `Loại khối này` | Khối chìm, nhãn thành `Giữ khối này`, tiêu điểm còn ở `OL`, `j` chạy tiếp |
| `Đầu vùng giữ` ở 16, nhắm 14, `Cuối vùng giữ` | Dải đảo chiều giữ 14–16, nút biến mất, tiêu điểm còn ở `OL`, `j` chạy tiếp |
| Kéo trên chữ khối 5 | Nhắm khối 5, vùng bôi `bai viet co du chu de Readab` còn nguyên |
| › rồi ‹ | Chương đổi, `scrollTop` 358 không đổi, tiêu điểm ở `panel`, `j` chạy trên Chương 2 |

Hai lỗi Vitest và e2e không thấy, chỉ lộ ở bàn đo:

- Bấm › kéo tiêu điểm và cuộn sang danh sách Chương ở tầng 4, nút rời khỏi con trỏ. Đã sửa: đường chuột giữ nguyên tầm nhìn, đường `⌥→` không đổi.
- `<ol>` bị huỷ trong lúc Chương nạp nên tiêu điểm rơi về `BODY` và `J`/`K` chết. Đã sửa: tiêu điểm sang `panel`.

Mất cú bấm: 12 vòng xen kẽ trong một lượt, ngay sau một phím thật, cùng toạ độ.

| Giữ nút | Cụm nút có `click` | Checkbox có sẵn có `click` |
|---|---|---|
| 90 ms | 6/6 | 6/6 |
| 0 ms (`cliclick c:`) | 0/6 | 0/6 |

`cliclick c:` nhả ngay nên `mouseup` không tới trang, trên cả điều khiển có sẵn. Bàn đo vì thế giữ 90 ms. Chưa đo: tap-to-click trên trackpad, thời gian giữ ngắn hơn 90 ms. Hai lượt đầu của phép đo này (15 và 40 ms) đã bỏ vì nhiễu: cú bấm vào checkbox lấy tiêu điểm nên `J` không dời khối và nút trôi khỏi màn hình.

Điều kiện đo:
- HTML một dòng dài hơn 4 KiB làm Chương k > 0 báo lỗi, xem mục nợ cùng ngày; trang đo có xuống dòng.
- Trần 120 s của mocha không ghi đè được bằng `this.timeout`, nên phép đo mất cú bấm là một `it` riêng.
