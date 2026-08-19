<!-- bmad:context -->
<!-- Verified 2026-08-19 against 705d17a. Managed by bmad-project-context; edits inside this block are replaced on refresh. -->

## e2e/ — WebdriverIO trong webview thật

Vai duy nhất: hành vi trong WKWebView/WebView2 THẬT. Bộ này không chạy ở cổng tự động nào — không trong `pre-push`, và 0 lần trong `ci.yml`. Chạy tay: `npm run test:e2e`.

## Known pitfalls

- 🔴 Cấm `.click()` của driver, dùng `realClick()` ở `e2e/support/pointer.mjs`. Driver bắn `click` TRƯỚC `focusin` — ngược chuột thật — nên nó vừa cho ĐỎ sai nguyên nhân, vừa cho XANH trên một sản phẩm đang hỏng. Cưỡng chế bằng `no-restricted-syntax` trong `eslint.config.js`.
- Mỗi spec mở một cửa sổ thật (~1,5 phút) và nó ghi vào `global.db` cùng thư mục gốc Library THẬT của người chạy nếu hai biến môi trường chuyển hướng không xuống được tiến trình con. `wdio.conf.mjs` có một phép tự kiểm DƯƠNG TÍNH — `global.db` phải nằm trong thư mục tạm — chạy trước khi xoá bất cứ gì. Đừng gỡ phép kiểm đó.

<!-- /bmad:context -->
