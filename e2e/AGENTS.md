<!-- bmad:context -->
<!-- Verified 2026-08-24 against b290336. Managed by bmad-project-context; edits inside this block are replaced on refresh. -->

## e2e/ — WebdriverIO trong webview thật

Vai duy nhất: hành vi trong WKWebView/WebView2 THẬT. Bộ này chạy ở NHỊP ĐÊM trên macOS (`schedule` 18:00 UTC = 01:00 giờ Ice, cộng `workflow_dispatch`) — KHÔNG ở `push`, không trong `pre-push`, và nửa Windows/WebView2 chưa từng chạy. Chạy tay: `npm run test:e2e`.

## Known pitfalls

- 🔴 Cấm `.click()` của driver, dùng `realClick()` ở `e2e/support/pointer.mjs`. Driver bắn `click` TRƯỚC `focusin` — ngược chuột thật — nên nó vừa cho ĐỎ sai nguyên nhân, vừa cho XANH trên một sản phẩm đang hỏng. Cưỡng chế bằng `no-restricted-syntax` trong `eslint.config.js`.
- Mỗi spec mở một cửa sổ thật (~1,5 phút) và nó ghi vào `global.db` cùng thư mục gốc Library THẬT của người chạy nếu hai biến môi trường chuyển hướng không xuống được tiến trình con. `wdio.conf.mjs` có một phép tự kiểm DƯƠNG TÍNH — `global.db` phải nằm trong thư mục tạm — chạy trước khi xoá bất cứ gì. Đừng gỡ phép kiểm đó.
- 🔴 Nhịp đêm ĐỎ không có nghĩa là sản phẩm hồi quy. Bốn đêm đầu (20–23/08) đỏ 2, và cả hai lượt đỏ chết ở CẦU IPC — `core.invoke not available after 5s` ⇒ fixture không tạo được Tác phẩm ⇒ lưới 0 hàng sau 30 s — trong khi job `check` xanh cả hai nền tảng cả bốn đêm. Đọc lỗi trước khi sửa một dòng sản phẩm; và đừng vá bằng `continue-on-error` hay một vòng chạy lại, cả hai biến job thành thứ không bao giờ đỏ. (Chủ: Story 3.9, `deferred-work.md`.)

<!-- /bmad:context -->
