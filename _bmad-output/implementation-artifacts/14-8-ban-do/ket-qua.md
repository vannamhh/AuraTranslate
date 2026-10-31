# Bàn đo 14-8: bật tắt panel bằng chuột thật

Bản dựng: binary debug `--features wdio`, Vite của `tauri dev` đang chạy, WKWebView trên máy Ice, 2026-10-31. Chuột thật qua `cliclick`, giữ nút 90 ms, rê chuột trước mỗi cú. Cửa sổ 1300×900 logic, tầng `full`, preset mặc định. Quần thể: một Tác phẩm 120 câu, `scrollHeight` 5414. Lượt ghi ở đây chạy sau khi lưới nhớ vị trí cuộn, tải máy khoảng 8.

| Công tắc | Sau khi tắt | Sau khi bật | `scrollTop` trước → sau |
|---|---|---|---|
| Đối chiếu | 2 nhóm, chỉ công tắc này tắt | 3 nhóm, cả ba bật | 600 → 600 |
| Tra cứu | 2 nhóm, chỉ công tắc này tắt | 3 nhóm, cả ba bật | 600 → 600 |
| Đề xuất AI | 2 nhóm, chỉ công tắc này tắt | 3 nhóm, cả ba bật | 600 → 600 |

Cú bấm: 26/26 có đủ `mousedown`, `mouseup`, `click`. `aria-checked` đọc lúc menu mở, qua hai cú bấm thật nữa cho mỗi lần đọc.

Hình học của chỗ cũ và cỡ cũ nằm ở `e2e/specs/panel-toggle-mouse.e2e.mjs`: x, y, w, h của cả ba nhóm trong sai số 2 px sau mỗi cặp tắt rồi bật.

Mất cú bấm: một lượt trước, chưa rê chuột và bấm lặp đúng một điểm trên `Công cụ ▾`, tải máy khoảng 8, 1 trong 20 cú có `mousedown` mà không có `mouseup` và `click`. Cùng triệu chứng với `14-4-ban-do/ket-qua.md`. Rê chuột trước mỗi cú thì 0 trong 26. Một lượt không đủ để kết luận nguyên nhân; mục nợ Epic 14 của 14-4 đã giữ triệu chứng này.

Bàn đo không dùng `ensureFullLayoutTier`: hàm đó nới cửa sổ lên 5000 px nên `Công cụ ▾` nằm ở x=5083, ngoài màn hình 1536 px, và cả ba cú đầu không tới trang.

🔵 2026-10-31: trước khi `GridPanel` nhớ vị trí cuộn, hàng Đối chiếu đo 600 → 0. Sau bản vá có một lượt ở tải máy khoảng 40 dừng giữa chừng: 1 trong 15 cú có `mousedown` mà không có `mouseup` và `click`, dù đã rê chuột, nên việc rê chuột không loại hết triệu chứng. Lượt ở tải khoảng 8 mất 0 trong 26.
