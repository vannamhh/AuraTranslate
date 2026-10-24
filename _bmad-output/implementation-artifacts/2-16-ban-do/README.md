# Bàn đo story 2-16 — bố cục đầu, dời con trỏ bằng chuột thật, dời con trỏ bằng `editor.next_segment`

Tạo tác của một lượt đo, không vào `package.json`. Bản release thật, sự kiện thật ở tầng hệ điều hành. Khuôn của `../2-15-ban-do/`.

- `build-216.sh <repo> <out>`: dựng release của `<repo>`, nối `bench-216.js` vào `dist/assets/index-*.js`, ký ad-hoc, chép `.app` ra `<out>`.
- `run-move.sh <nhãn> <app-dir> <122|9850>`: một phiên, màn hình mở khoá, ngoài sandbox. Cửa sổ đặt {200,25,1200,900}. Phím nóng Ctrl+Alt+Shift+chữ (Q/W tạo Tác phẩm 122/9.850 câu (và gán `editor.next_segment` = `Mod+Alt+Y` qua `put_config` kind `shortcut`, vì lệnh này không có phím mặc định), E mở Workspace và đo bố cục đầu, R chọn 3 ô đích đang hiện trong khung nhìn, T bật bộ ghi, U đổ số).
  - Chuột: `cliclick c:x,y` (mousedown/mouseup thật), 3 lượt cách 4 s, mỗi lượt tính `perl time` ngay trước lệnh.
  - Phím: `keys-3.applescript` gửi `Cmd+Opt+Y` 3 lần cách 4 s, in giờ ngay trước mỗi `keystroke`.
- `analyse.py ket-qua-*.json`.

Các số:
- `layout_ms`: từ lúc handler mở tab Workspace tới hết hai frame sau khi `.cell-tgt` đầu tiên có mặt trong DOM. `cell_present_ms`: tới lúc ô có mặt.
- `sent->f`: từ giờ gửi (đồng hồ shell) tới hết hai frame (`rAF` lồng nhau), gồm cả thời gian khởi động `cliclick`/`osascript` và hàng đợi sự kiện. `h->f`: từ lúc handler capture chạy tới hết hai frame, không gồm hàng đợi. Số thật nằm giữa; `event.timeStamp` của WebKit là giờ chạy handler (xem README 2-15) nên không dùng làm giờ gửi.
- `hit` / `moved`: ô đang focus sau hai frame có đúng ô đích (chuột) hoặc đã đổi ô (phím).

Giới hạn: một phiên ba lượt mỗi đường trên máy đang chạy việc khác; số kèm `load_before`.

⚠️ Không dùng phím số trong hợp âm của bàn đo: macshot (đang chạy trên máy này) gắn phím số 2/3/4/5 (mã 19/20/21/23) cho chụp màn hình và phản ứng cả với Ctrl+Alt+Shift+số, ảnh hưởng tiêu điểm cửa sổ. Bàn 2-15 vẫn dùng chúng.

## Số "trước" — `baseline_revision` 5f25df6, bản release ký ad-hoc, macOS 15.8.2, WebKit.framework 20621 (UA đông cứng 605.1.15), cửa sổ 1200×900

Ba phiên mỗi cỡ (`ket-qua-base-<122|9850>-<1..3>.json`), `loadavg` 1 phút lúc bắt đầu 4,9–5,1 (122 câu) và 6,5–10,3 (9.850 câu); máy đang chạy việc khác. Đơn vị ms, ba lượt mỗi phiên.

| 9.850 câu | phiên 1 | phiên 2 | phiên 3 |
|---|---|---|---|
| bố cục đầu (mở tab → ô đích đầu + 2 frame) | 4711 | 4383 | 4541 |
| chuột thật, mousedown → 2 frame (handler→f) | 142 · 137 · 124 | 126 · 125 · 127 | 117 · 114 · 121 |
| chuột thật, gửi → 2 frame (sent→f) | 293 · 287 · 249 | 306 · 248 · 260 | 233 · 233 · 237 |
| `segment.next` (handler→f) | 110 · 59 · 63 | 85 · 62 · 72 | 73 · 59 · 59 |
| `segment.next` (sent→f) | 183 · 117 · 124 | 152 · 125 · 142 | 131 · 120 · 124 |

| 122 câu | bố cục đầu | chuột handler→f | chuột sent→f | `segment.next` handler→f | `segment.next` sent→f |
|---|---|---|---|---|---|
| phiên 1 | 512 | 60 · 53 · 60 | 177 · 168 · 170 | 65 · 56 · 48 | 76 · 62 · 53 |
| phiên 2 | 590 | 112 · 44 · 43 | 215 · 155 · 157 | 47 · 41 · 49 | 56 · 46 · 54 |
| phiên 3 | 574 | 44 · 64 · 46 | 174 · 207 · 166 | 49 · 56 · 45 | 58 · 62 · 50 |

Đọc số:
- Cả hai đường ở 9.850 câu nhanh hơn nhiều so với 0,5 s mỗi phím gõ, nhưng chuột chạm 114–142 ms handler→f (233–306 ms từ lúc gửi) và vượt cỡ 122 câu khoảng 2–3 lần; đó là "số trước" cho 2.14 tiêu chí 2.
- `mouseup` của `cliclick c:` hầu như không được ghi (bộ ghi chỉ thấy 1 trong 9 lượt ở một phiên thử), nên chỉ có số mousedown.
- `segment.next` đổi ô trong cùng handler `keydown` (focusin ngay lúc `keydown`, `before` trong JSON đã là ô mới); đừng đọc `before` như ô cũ.
- `sent→f` gồm thời gian khởi động `cliclick`/`osascript` (vài chục ms); `handler→f` bỏ hàng đợi sự kiện. Số thật nằm giữa.

Nghiệm thu story 2-16 (release `24bb1e2`, 2026-10-24): số và bảng ở `HANDOFF.md` §Nghiệm thu; số thô `ket-qua-acc-*.json` ở thư mục này (dời con trỏ) và ở `../2-15-ban-do/` (gõ).
