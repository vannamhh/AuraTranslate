# Bàn đo: `delete` ở đầu ô bản dịch không gộp câu

Bản dựng: binary debug `cargo build --locked --features wdio` với `TAURI_CONFIG` đặt `devUrl` về Vite riêng ở cổng 1421, WKWebView trên máy Ice, 2026-11-01. Mã cũ chạy từ một worktree tạm ở `48ea2d4bdd7038a17a3b9fa186c0505ea17d1e30`, không đụng phiên `tauri dev` đang mở. Rust không đổi giữa hai lượt.

Quần thể: Tác phẩm nhập song ngữ 9 cặp (`Ban dich N.`), một Chương. Phím `delete` thật qua `osascript key code 51`, `⌘M` qua `keystroke`, chuột thật `cliclick`, `ensureFront()` trước mỗi lượt. Đĩa đọc bằng `read_open_chapter_segments` sau 3,5 s. Tải máy lúc đầu lượt: 4,09 (sau), 4,61 (trước).

| Ca | Mã cũ (`ket-qua-truoc.json`) | Mã mới (`ket-qua-sau.json`) |
|---|---|---|
| Caret offset 3 của câu 5, 6 cú `delete` (3 cú xoá chữ, 3 cú thừa) | 9 hàng còn 6, câu 2 đến 5 gộp thành một ở lưới và đĩa | 9 hàng, chỉ mất 3 ký tự ở câu 5 |
| Câu 8: chọn hết, 1 cú xoá, 5 cú thừa | 6 hàng còn 2, gộp dây chuyền tới câu đầu Chương | 9 hàng, câu 8 rỗng ở lưới và đĩa |
| `⌘M` thật ở hàng cuối | gộp được | 9 hàng còn 8, có dòng báo |
| Mục menu chuột phải | không đo: mã cũ chỉ còn 1 hàng sau các ca trên | 8 hàng còn 7, có dòng báo |

Mọi `keydown` tới ô (6/6 ở cả hai ca). Mã mới 0/5 cú bấm mất `click`. Lượt `sau` đầu tiên đỏ 5/5 vì `System Events` chưa chạy (`-600`), không phải sản phẩm; lượt lại cùng binary xanh 5/5.

Ngoài phạm vi: xoá tới sát khoảng trắng đầu câu để lại `\xa0dich 5.` ở cả hai bản; đó là WebKit đổi khoảng trắng đầu ô thành NBSP, có từ trước.
