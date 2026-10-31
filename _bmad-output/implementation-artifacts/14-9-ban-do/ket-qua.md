# Bàn đo 14-9: tám nút lẻ bằng chuột thật

Bản dựng: binary debug `cargo build --locked --features wdio`, `npm run build` trước đó, Vite của `tauri dev` (Ice) đang chạy ở 1420 và phục vụ đúng cây việc chưa commit, WKWebView trên máy Ice, 2026-10-31. Chuột và phím thật qua `cliclick` (rê chuột, chờ 200 ms, giữ nút 90 ms) và `osascript` cho Esc, phím mũi tên; `ensureFront()` chạy trước mỗi cú bấm và mỗi phím; wdio chỉ đọc trạng thái. Cửa sổ 1300×900 logic. Lệnh: `npx wdio run e2e/wdio.conf.mjs --spec _bmad-output/implementation-artifacts/14-9-ban-do/chuot-that.e2e.mjs`, không sửa `wdio.conf.mjs`.

Quần thể: một Tác phẩm nhập song ngữ 26 cặp (`Ban dich N.`), 25 câu đã xác nhận thành 25 cặp TM, câu 1 chưa xác nhận để gõ vào; tách thành 4 Chương trước phần Library; một bản sao reviewer 4 hàng (xem Giới hạn).

Tải máy và cú bấm, mỗi lượt xanh 7/7 trên cùng bàn đo: lượt chốt (ghi trong `ket-qua.json`) tải 37 đầu, 11 cuối, 22 cú, 22 lần bấm, 0 mất. Trước đó: tải 3,4 đến 3,0, 22 lần bấm, 0 mất; tải 2,8 đến 3,0, 23 lần bấm, 1 mất (`Ẩn`); tải 5,1 đến 3,6, 25 lần bấm, 3 mất (ô soạn prompt hai lần, Đóng không sửa). Mỗi cú mất có `mousedown`, không có `mouseup` và `click`, bấm lại ngay thì qua. Lượt 21/21 sớm hơn ở tải 4,6 đến 6,9 không còn trong `ket-qua.json`.

| # | Phép đo | Kết quả | Số đọc |
|---|---|---|---|
| 1 | Ẩn | Đạt | lượt chốt: 3 sự kiện ở lần bấm đầu; một lượt trước mất `mouseup` và `click` ở lần đầu; dải biến mất; tiêu điểm vẫn ở ô dịch |
| 1 | Bỏ chọn | Đạt | 3 sự kiện; chọn 1 → 0; nút biến mất |
| 1 | Huỷ | Đạt | 3 sự kiện; nút biến mất; danh sách vẫn 1 dấu |
| 1 | Mục trước | Đạt | 3 sự kiện ở cả hai cú; con trỏ 2 → 1 → 0; nút biến mất ở 0 |
| 1 | Nhập từ file | Đạt | 3 sự kiện; hộp thoại native hiện (sheet 0 → 1), Esc đóng (1 → 0), không để hộp thoại mở |
| 1 | Đóng Prompt library | Đạt | 3 sự kiện ở cả hai cú; lớp phủ đóng |
| 1 | Sửa trên hàng TM | Đạt | 3 sự kiện |
| 1 | Xoá trên hàng TM | Đạt | 3 sự kiện ở cả hai nhịp |
| 2 | Con trỏ ở A, Sửa hàng B | Đạt | con trỏ 0 → 3, ô sửa mang `Ban dich 23.` = đích B |
| 2 | Con trỏ ở A, Xoá B hai nhịp | Đạt | nhịp một: chờ xác nhận ở hàng B (con trỏ 2, 25 hàng); nhịp hai: 24 hàng, B (`Ban dich 24.`) mất, A còn |
| 3 | Hàng bị xén | Đạt | nút cao 28 px, thấy 18 px, cắt 10 px, `elementFromPoint` là nút; `scrollTop` 419 ở `mousedown`, `mouseup` và `click`; 3 sự kiện; ô sửa mang đúng hàng bị xén (`Ban dich 16.`). Sau `click` `scrollTop` 2069 vì form sửa hiện ra |
| 4 | Gõ rồi Đóng ngay | Đạt | gõ `QXZ`; sau Đóng lớp phủ đóng, `prompt_set_list` có `...haiQXZ`; mở lại thì ô thân hiện `...haiQXZ` |
| 5 | Ẩn rồi gõ | Đạt | caret ở 4 trước và sau Ẩn; gõ `q` thành `Ban qdich 1.`, đúng vị trí 4; câu 2 không đổi |
| 6 | `activeElement` sau Huỷ | `DIV.marks-panel` | Esc lần 1 huỷ nhịp chờ (lớp phủ còn mở), Esc lần 2 đóng |
| 6 | `activeElement` sau Mục trước về 0 | `SECTION.al-panel` | ArrowDown: con trỏ 0 → 1 |
| 6 | `activeElement` sau Bỏ chọn | `UL.chapters-list` (sau bản vá) | ArrowDown chọn 0 → 1; trước bản vá `SECTION.wp` và chọn vẫn 0 |

## Lỗi tìm thấy

Sau Bỏ chọn bằng chuột, tiêu điểm nằm ở gốc `SECTION.wp` chứ không ở `UL.chapters-list`, nên các phím mũi tên của danh sách Chương chết: `keydown ArrowDown` có `target=SECTION.wp`, số hàng chọn sau phím vẫn 0. Đối chứng ngay sau đó: bấm thật một hàng Chương (tiêu điểm về `UL.chapters-list`) rồi ArrowDown thì hàng chọn đi 1 → 2. Nguyên nhân là `mousedown` lên nút không focus được trong WKWebView đẩy tiêu điểm lên tổ tiên focus được gần nhất; `onListKeydown` nằm trên danh sách. Plan chỉ nêu "về panel nếu rơi về `body`" cho Huỷ và Mục trước, không cho Bỏ chọn, nên đây là mục nợ `Chủ: Epic 14` hoặc sửa trước khi đóng story, do Ice quyết.

🔵 2026-10-31: đã sửa ở `LibraryWorkPage.vue`, số chọn về 0 mà tiêu điểm ở `body` hoặc gốc `.wp` thì danh sách nhận tiêu điểm. Sau bản vá tiêu điểm ở `UL.chapters-list`, ArrowDown chọn 0 → 1.

🔵 2026-10-31: review tìm thêm một lỗi có từ trước story, bàn đo xác nhận: sau cú Xoá thứ hai hàng đang giữ tiêu điểm bị gỡ, tiêu điểm về `BODY`, ArrowDown không dời con trỏ (2 → 2) và mọi phím của lớp phủ chết. Đã sửa ở `TmManageOverlay.vue`; sau bản vá tiêu điểm ở `UL.tm-list`, ArrowDown dời 2 → 3.

## Giới hạn và điều chưa đo

- Cliclick `kp:esc` không tạo `keydown` nào tới trang (8 lần, `keyLog` rỗng, tải 2,6); Esc và mũi tên trong bàn đo đi qua `osascript` `key code` (53, 125) và cho `keydown` thật. Gõ chữ vẫn qua `cliclick t:`. Nguyên nhân cliclick chưa tìm.
- Bản sao reviewer chèn thẳng bằng `sqlite3` vào `project.db` của Tác phẩm bàn đo (thư mục tạm của e2e) vì nhập reviewer đi qua hộp thoại tệp native; hộp thoại đó chỉ đo ở nút Nhập từ file của Prompt library.
- Dải Proofreader ở chỗ `track` cần một kết quả quét; quét thật cần keychain, nên kết quả được đặt vào đúng instance `proofreadState.ts` mà app đang dùng (URL có tem `?t=` đọc từ resource timing; dải hiện ra chứng minh cùng instance). Nút Ẩn tắt ở trạng thái quét không đo ở đây.
- Mục "tap-to-click trackpad", phím Enter/Space trên nút (không có đường tự động theo `e2e/AGENTS.md`) và Windows không đo.
- Các lượt trước ở tải 25 đến 70 (Blender của Ice chạy dài) mất cú bấm nhiều (khoảng 4 trên 10) và không dùng; không lượt nào ở tải đó vào bảng trên. Ở tải 2 đến 3, một cú đầu `Sửa` và một cú `Ẩn` từng mất `mouseup`, cùng triệu chứng `14-4-ban-do/ket-qua.md`; bấm lại thì qua.
- `man-hinh.png` không giữ.
