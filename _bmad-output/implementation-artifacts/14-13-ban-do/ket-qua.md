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


## Bàn gõ trước và sau

Bàn `../2-15-ban-do/`: 300 phím ở 10 phím/giây rồi `Mod+Enter`, release ký ad-hoc, 9.850 câu, xen kẽ after và before, 3 phiên mỗi bên. `app215-before` dựng từ `baseline_revision`, `app215-after` từ cây làm việc. Tải máy: after 2,63 · 5,44 · 3,04; before 2,76 · 4,37 · 2,96. Số thô: `../2-15-ban-do/ket-qua-t14-<after|before>-<1..3>.json`.

| Phiên | Phím cuối sau | Phím cuối trước | `Mod+Enter` sau | `Mod+Enter` trước |
|---|---|---|---|---|
| 1 | 87 | 92 | 111 | 116 |
| 2 | 77 | 80 | 115 | 119 |
| 3 | 80 | 88 | 119 | 109 |

Mọi phiên đủ 300 phím, một phím chờ lúc phím cuối được gửi, ở cả hai bên. ⇒ Gõ không chậm đi: phím cuối sau thấp hơn trước ở cả 3 cặp, `Mod+Enter` nằm trong dao động của bản trước.

⚠️ Cả hai bản đều chậm hơn nghiệm thu `8260a92` ngày 2026-10-24 (phím cuối 56–77, `Mod+Enter` 68–97), và `Mod+Enter` vượt trần 100 ms. Bản trước cũng vậy nên không do story này; nguyên nhân chưa quy được, ghi ở sổ nợ.

## Chuột thật

`app-mouse` (cây làm việc, `bench-1413m.js`), `run-mouse.sh`, `cliclick` sự kiện tin cậy, 9.850 câu. Kéo từ số câu 3 qua 4 tới 5: tô đúng chỉ số 2, 3, 4, 12 ô. `Shift`+bấm số câu 13: tô chỉ số 2–12, 44 ô, sự kiện mang `shiftKey`. Số thô: `ket-qua-mouse-m1.json`. Không có bản trước để đối chiếu; khớp luật Story 14.3.

Lượt đầu tách cử chỉ thành nhiều tiến trình `cliclick` nên kéo không tới app và `Shift` không bám vào cú bấm; một cử chỉ phải nằm trong một lần gọi `cliclick`.

Hai bàn này chạy trong cửa sổ Ghostty theo yêu cầu của Ice: tiến trình Claude Code treo dưới `launchd` nên macOS không xác định được app chịu trách nhiệm và trả `-1743`.
