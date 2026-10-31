# Bàn đo 14-7: cắt cặp ở nhập song ngữ bằng chuột thật

Bản dựng: binary debug `--features wdio`, Vite của `tauri dev` đang chạy, WKWebView trên máy Ice, 2026-10-31. Chuột và phím thật qua `cliclick` (giữ nút 90 ms) và `osascript` (phím mũi tên). Quần thể: một hàng CSV, 3 câu nguồn, một dòng đích tiếng Việt không dấu 48 từ, 94 đích bấm, dòng xuống 4 hàng. Số liệu bên dưới lấy từ lượt 3. Số thô ở `ket-qua.json`. Chưa đo tap-to-click.

| Ca | Kết quả |
|---|---|
| Bấm đích thứ 3 | `click` tới đúng nút, `¦` thành `‖`, hai mảnh `Ong lao` / `ngoi duoi goc ...`, hàng còn lệch, tiêu điểm ở `SECTION.bip-panel` |
| Bấm đích thứ 6 | Hàng thành `Đã khớp`, ba mảnh, ở lại trên màn, hai nút cặp lệch không hiện |
| Bấm lại đích thứ 6 | Hàng về `Chỗ lệch 1 / 1`, `‖` thứ 6 thành `¦`, caret còn ở đích 6 |
| `←` rồi `→` sau cú bấm | Caret 5 → 4 → 5, tiêu điểm vẫn ở `panel` |

Hình học: đích rộng 21 px, cao 28 px. Tâm cách tâm nhỏ nhất trên cùng một hàng chữ là 23,6 px. Ở 14 px con số này sẽ là 16,6 px, dưới 24, nên `DESIGN.md` §Layout & Spacing áp 21 px.

Mất cú bấm, ba lượt, cùng cách bấm, giữ nút 90 ms:

- Lượt 1, tải máy khoảng 76, ô đích bị dấu phẩy cắt còn 14 từ: 1 trong 4 cú mất ở lần cắt đích thứ 6. `mousedown` tới, `mouseup` và `click` không tới; caret đã đổi nên cú này có render lại lúc nhấn. Bấm lại lần hai thì tới.
- Lượt 2, tải máy khoảng 6, cùng ô 14 từ: 0 trong 3 cú mất ở ca trên, rồi 12 vòng không bấm lại, mỗi vòng một cú trên đích khác (caret đổi mỗi cú) xen một cú đối chứng trên dòng tiêu đề. Đích 12/12 đủ `mousedown`, `mouseup`, `click`; đối chứng 12/12.
- Lượt 3, tải máy khoảng 10–17, dòng 48 từ, sau các bản vá của review: 0 trong 3, đích 12/12, đối chứng 12/12.

⇒ Cú mất ở lượt 1 không tái hiện, nguyên nhân chưa xác định. Ba lượt khác tải máy nên không kết luận được do tải. Một lượt bàn đo khác dừng ở bước hiệu chuẩn (cú nhấp đúp đầu không tới trang), trước màn song ngữ, chạy lại thì qua. `ket-qua.json` là số thô của lượt 3.
