# 14-13 — số "trước" trên bản phát hành

Bản dựng: `baseline_revision` 27574847 (cây sạch, chỉ có thư mục này và plan chưa theo dõi), release ký ad-hoc, `app-before/`. macOS 15.8.2, WebKit.framework 20621.3.11.11.3 (UA đông cứng 605.1.15), cửa sổ 1200×900, màn hình mở khoá. Quần thể: một Tác phẩm `'句子。'.repeat(9850)`, 9.850 hàng `[data-num-segment-id]`. Tải máy (`loadavg` 1 phút lúc bắt đầu): 5,30 · 4,98 · 4,27; máy đang chạy việc khác.

Bàn: `bench-1413.js` nối vào bundle, `run-drag.sh`, `analyse.py`. Vòng đo theo 14.3: `mouseover` tổng hợp trong trang, `t0` trong `rAF`, đo tới `rAF` kế. `adjacent` = 60 hàng liền nhau; `step167` = 59 bước 167 hàng (bước cuối kẹp ở hàng 9.849, vì 59×167 > 9.850). Mỗi phiên chạy `adjacent`, `step167`, `adjacent`, `step167`. Ba vế: `total` (khoảng giữa hai frame, gồm style/layout/paint), `js` (đồng bộ trong `dispatchEvent`), `microtask` (tới sau hai microtask, tức Vue đã flush render + patch DOM). Phần sau JS = `total` − `microtask`.

Trung vị mỗi lượt 60 hàng (ms), từng lượt trong từng phiên:

| Phiên | adjacent total | adjacent js / microtask | step167 total | step167 js / microtask |
|---|---|---|---|---|
| 1 | 63 · 64 | 5 / 31 · 5 / 31 | 73 · 73 | 5 / 33 · 5 / 33 |
| 2 | 64 · 62 | 5 / 30 · 4 / 29 | 72 · 71 | 4 / 32 · 4 / 32 |
| 3 | 65 · 61 | 5 / 31 · 4 / 28 | 67 · 73 | 5 / 32 · 4 / 32 |

Tối đa của `total`: 82–127 ms. Số lượt trên 50 ms: 60/60 ở mọi lượt `adjacent`, 59/59 ở mọi lượt `step167`. Số thô: `ket-qua-before-<1..3>.json`.

## Kết luận Pha 1: TRƯỢT

Trung vị 61–73 ms, cao hơn 50 ms ở cả 3 phiên, mọi lượt. Hàng I/O "Bản phát hành trượt" ⇒ Pha 2 chạy.

Chi phí nằm đâu (bản phát hành, 9.850 hàng):
- JS đồng bộ trong handler: 4–5 ms.
- Vue flush + patch DOM (tới hết microtask): ~30 ms — đây là phần chỉ-render-khối-đổi có thể cắt.
- Sau JS (style, layout, paint tới frame kế): ~30–40 ms, không quy được cho một chỗ khi chưa có profile; tương tự trần 53–66 ms của dời caret ở 2-16. Nếu sau khi cắt phần ~30 ms mà `total` vẫn ≥ 50 ms thì rơi hàng "Sửa rồi vẫn trượt".
- Trung vị `step167` cao hơn `adjacent` ~5–10 ms dù cả chương bị tô: chi phí theo cỡ Chương, không theo số hàng đổi.

## Ghi chú cho người sau

- 🔵 2026-11-01: lỗi dưới đây đã sửa, `npm run build` xanh; hai `build-1413*.sh` giữ `npx vite build` để bản dựng đo được giống bản đã đo.
- ⚠️ `npm run build` đỏ ở `baseline_revision`: `vue-tsc -p tsconfig.node.json` báo lỗi kiểu trong `scripts/gen-zh-fold.mjs` (commit afdc4205). `build-1413.sh` vì vậy gọi `npx vite build` (cùng bundle, bỏ bước kiểm kiểu). Plan nhắc `npm run build` lại sau bản dựng để `cargo test` không nhúng bench; dùng `npx vite build` cho tới khi lỗi kiểu được sửa (không thuộc story này; mục nợ `Chủ: Ice` đã có ở `deferred-work.md`).
- `app-before/` (67 MB) giữ để Pha 3 xen kẽ trước/sau, không có trong `.gitignore`: đừng commit. `dist/` đã dựng lại sạch (0 dấu bench).

## Số sau, xen kẽ với số trước

Bản dựng: `app-after` = `baseline_revision` 27574847 cộng cây làm việc chưa commit (`app-after/revision.txt`), `app-before` như trên; release ký ad-hoc, cùng máy, cùng cửa sổ, cùng quần thể 9.850 hàng. Xen kẽ after-1, before-b1, after-2, before-b2, after-3, before-b3. Tải máy (`loadavg` 1 phút lúc bắt đầu): after 4,78 · 9,10 · 6,57; before 4,86 · 8,60 · 6,52. Số thô: `ket-qua-after-<1..3>.json`, `ket-qua-before-b<1..3>.json`.

Trung vị mỗi phiên, mỗi phiên hai lượt (ms):

| Hình dạng | Vế | Trước | Sau |
|---|---|---|---|
| 60 hàng liền nhau | `total` | 66–69 | 40–41 |
| 60 hàng liền nhau | `microtask` | 31–33 | 6–7 |
| 59 bước 167 hàng | `total` | 73–79 | 45–48 |
| 59 bước 167 hàng | `microtask` | 34–36 | 9–10 |

Số lượt trên 50 ms: trước 60/60 và 59/59 ở mọi lượt; sau 1–10/60 và 22–26/59. Tối đa `total` sau: 52–98 ms. Vế `js` giữ 5 ms ở cả hai bản. Phần sau JS (`total` − `microtask`) không đổi, khoảng 34–38 ms sau sửa.

⇒ Trung vị 60 hàng kề đạt dưới 50 ms ở cả 3 phiên. Hình dạng 167 hàng cũng dưới 50 ms nhưng gần ngưỡng, gần nửa số bước vẫn vượt.

Chưa đo: bàn gõ trước và sau (`../2-15-ban-do/`), chuột thật trên `app-after`. Lượt Pha 3 bị dừng khi bản trước của bàn gõ đang dịch.

## Chưa đo ở Pha 3 (bị chặn bởi môi trường, không phải bởi mã)

- Bàn gõ `2-15-ban-do` trước và sau: hai bản `app215-before` (worktree `baseline_revision`) và `app215-after` (cây làm việc) đã dựng xong, ký ad-hoc, nhưng không chạy được phiên nào. Host agent lúc này báo `Not authorised to send Apple events to System Events (-1743)` và cliclick báo thiếu Accessibility, nên `require_front` đỏ (rc=5) ở 4 phiên thử, trước phím đầu tiên; không có số nào được ghi. Chạy lại: `./run-typing.sh` ở thư mục này, từ terminal có quyền Automation/Accessibility, màn hình mở khoá.
- Chuột thật (AC cuối): `app-mouse` (cây làm việc, `bench-1413m.js` đọc `.row-selected` qua phím nóng Ctrl+Alt+Shift+M) đã dựng, `run-mouse.sh <nhãn> app-mouse` viết xong, chưa chạy lần nào, cùng lý do. Bàn chưa được kiểm chứng; không có đối chứng bản trước.
- `.gitignore` của thư mục này giữ các bản dựng, HOME nháp và log ngoài git. `dist/` đã dựng lại sạch.
