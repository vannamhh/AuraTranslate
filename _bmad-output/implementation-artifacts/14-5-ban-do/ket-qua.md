# Bàn đo 14-5: Thêm thuật ngữ bằng chuột thật

Bản dựng: binary debug `--features wdio`, Vite của `tauri dev` đang chạy, WKWebView trên máy Ice, 2026-10-31. Chuột và phím thật qua `cliclick`, `Esc` thật qua `osascript`. Mỗi ca một lượt, mỗi lượt một phiên wdio. Chạy lại: `npx wdio run e2e/wdio.conf.mjs --spec _bmad-output/implementation-artifacts/14-5-ban-do/chuot-that.e2e.mjs --mochaOpts.grep "<tên ca>"`.

| Ca | Ô thuật ngữ | Sau phím |
|---|---|---|
| Bôi `五` ở nguyên văn, chuột phải trong vùng bôi | `五`, tiêu điểm ở ô, menu đã đóng | `q` thành `五q` |
| Bôi `alpha` ở bản dịch, chuột phải trong vùng bôi | `alpha`, tiêu điểm ở ô, menu đã đóng | `q` thành `alphaq` |
| Từ bôi ở câu 1, chuột phải ô số câu 3 | Trống, tiêu điểm ở ô | `q` thành `q` |
| Không bôi trước, chuột phải lên chữ ở nguyên văn | `八九`, từ WebKit tự bôi | Không đo phím ở ca này |

Đóng dải: caret ở câu 1, chữ `abcd`, offset 2, chuột phải câu 3 ở từng cột, chọn mục, `Esc`, gõ `z`.

| Cột nhắm | Sau `Esc` | Sau phím |
|---|---|---|
| Nguyên văn | Tiêu điểm ô câu 1, caret offset 2 | `abzcd` |
| Số câu | Tiêu điểm ô câu 1, caret offset 2 | `abzcd` |
| Bản dịch | Tiêu điểm qua ô câu 3 rồi về ô câu 1, caret offset 2 | `abzcd` |

Lượt đo đầu của ca ô số lộ lỗi Vitest và e2e không thấy: với cú bấm tin cậy, Vue render giữa listener của mục và listener ở `document`, nên `:disabled` theo `quickAddIsOpen` tắt mục trước khi `onDocumentClick` đọc `button.disabled`. Menu không đóng, phím đầu tiên vào nhánh đóng không chọn, trả trạng thái cũ và mất ký tự: ô thuật ngữ vẫn trống, vùng chọn quay về `一二三`, caret `null`. Trạng thái tắt nay đọc một lần lúc menu mở; các hàng trên đo sau bản sửa.

Chưa đo bằng chuột thật: đường Lưu (Vitest có ca cho cả ba cột).

Điều kiện đo:
- App vừa mở nuốt cú bấm đơn và cú chuột phải dù đã frontmost theo System Events. Cmd-Tab thật hai lần (`key code 48 using command down`) rồi bấm: cú bấm đơn qua sau 1,3–1,4 s, ba lượt đo.
- Rê chuột tới điểm, chờ 200 ms rồi mới bấm; không có bước này, cú chuột phải vào ô số mất cả bốn lần thử trong một lượt.
- Bản đầu của bàn đo gửi input không kiểm app nào đang ở trước. `cli()` nay từ chối khi app đo không phải frontmost.
