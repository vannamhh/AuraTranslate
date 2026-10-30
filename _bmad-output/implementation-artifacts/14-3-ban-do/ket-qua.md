# Đo chuột thật Story 14.3

Đo 2026-10-30 trên WKWebView 605.1.15, binary debug `--features wdio`, Vite của `tauri dev`, tải máy 6–10. Bàn đo: `chuot-that.e2e.mjs`, `cliclick` gửi sự kiện tin cậy vào cửa sổ app, wdio chỉ đọc lại.

## Chuột thật, Chương 5 câu

| Thao tác | Đo được |
|---|---|
| Caret câu 1, nhấn số 2, kéo qua số 3, thả trên ô bản dịch câu 4 | `buttons=1` ở mọi `mouseover`, tô đúng 2, 3, 4; caret vẫn 1, tiêu điểm vẫn ô câu 1 |
| Sau khi thả | `click` bắn lên `DIV` chung của hai ô, không handler nào đọc |
| Gõ trong ô câu 1, bấm › ở Chương cuối | `mousedown → mouseup → click` trên nút, không `focusout`; caret 1, tiêu điểm vẫn ô, thanh trạng thái báo biên Chương |

## Chi phí một lượt dời `focus`, Chương 9.850 câu

Bắn `mouseover` trong trang rồi chờ một khung hình. Tải 6–7.

| Hình dạng | Trung vị | Tối đa | Lượt trên 50 ms |
|---|---|---|---|
| 60 hàng liền nhau, đúng kéo thật | 116 ms | 145 ms | 60/60 |
| 59 bước, mỗi bước 167 hàng | 137 ms | 186 ms | 59/59 |

Hai dòng gần bằng nhau nên chi phí theo cỡ Chương, không theo số hàng đổi. Gộp theo khung hình không giảm được giá mỗi khung, nên không thêm. Chương 2.000 câu, 58 bước mỗi bước 34 hàng: trung vị 29 ms, tối đa 68 ms, 3/58 lượt trên 50 ms. Chưa đo trên bản phát hành.
