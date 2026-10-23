# Handoff Pha 1 -> Pha 2 (story 2-16)

Pha 1 xong. Bàn đo + số "trước" ở thư mục này (README có bảng số). Không đụng `src/`.

## Dùng lại bàn đo cho Pha 2 (đo bản thử chia khối)
- Dựng: `./build-216.sh <repo> <out>` (~11 phút, release). Chạy: `./run-move.sh <nhãn> <out> <122|9850>` (ngoài sandbox, màn hình mở khoá, KHÔNG chạm máy khi chạy). 3 phiên mỗi cỡ cho so sánh; `python3 analyse.py ket-qua-*.json`.
- Bàn này chỉ đo bố cục đầu, chuột thật và `segment.next`. Phiên gõ 30 s là bàn `../2-15-ban-do/` (`build-215.sh`, `run-session.sh`).
- 🔴 Hợp âm số (Ctrl+Alt+Shift+1..5) của bàn 2-15 đụng phím chụp màn hình của macshot (mã 19/20/21/23, đang chạy trên máy này): một lần phím 4 làm mất tiêu điểm cửa sổ app. Bàn 2-16 đã chuyển sang Ctrl+Alt+Shift+Q/W/E/R/T/U. Trước khi chạy `run-session.sh` 2-15 phải đổi hợp âm tương tự (trong `bench-215.js` và `run-session.sh`) hoặc tắt macshot; bản 2-15 chưa sửa.
- `segment.next` không có phím mặc định; `bench-216.js` gán `Mod+Alt+Y` qua `put_config` kind `shortcut` khi tạo Tác phẩm.
- Bộ chọn ô chuột: ba ô thứ 2, 4, 6 đang hiện (`.cell-tgt`, `data-segment-id`). Nếu khối đổi DOM, `.cell-tgt` vẫn phải tồn tại.

## Số "trước" (9.850 câu, baseline 5f25df6)
- Bố cục đầu 4,38–4,71 s (ô đích đầu có mặt ở 3,2–3,4 s).
- Chuột thật: handler->2 frame 114–142 ms, gửi->2 frame 233–306 ms.
- `segment.next`: handler->f 59–110 ms, gửi->f 117–183 ms.
- Cỡ 122 câu: bố cục 0,51–0,59 s, chuột 43–112 / 155–215 ms, phím 41–65 / 46–76 ms.
- Số gõ 10 phím/giây ở 9.850 câu: lấy từ bàn 2-15 (không đo lại ở Pha 1): ~0,5 s mỗi phím.

## Việc còn lại / rủi ro
- Ceiling cho 2.14 tiêu chí 2 ở thân story, không có trong plan này: so với số trên.
- `mouseup` của cliclick không ghi được; chỉ mousedown có số.
- Plan có `context: []` rỗng; Pha 2 đọc Code Map trong plan, không có tệp context để nạp.
- Phiên mỗi cỡ đo khi máy tải 5–10 (loadavg), ghi trong JSON.
