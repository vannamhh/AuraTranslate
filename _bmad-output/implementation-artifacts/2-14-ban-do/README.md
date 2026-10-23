# Bàn đo Story 2.14 — dời con trỏ và một phím gõ ở Chương 9.850 câu

Tạo tác của một lượt đo, không vào `package.json`. Khuôn của `../2-5b-ban-do/`.

Chạy ngoài sandbox, máy rảnh:

```sh
TAURI_WEBDRIVER_PORT=4467 npm run test:e2e -- \
  --spec _bmad-output/implementation-artifacts/2-14-ban-do/do-doi-con-tro.e2e.mjs
```

`TAURI_WEBDRIVER_PORT=4467` tránh cổng 4445 bị tiến trình khác giữ (xem README của 2-5b). Spec tự kiểm
`location.href` và số hàng trước khi đo. Ba phép: bấm chuột thật (`realClick`; mousedown và mouseup
đo riêng), `Mod+Alt+ArrowDown` tổng hợp, một `InputEvent` tổng hợp. Mỗi phép bỏ lượt đầu, n = 4.
Bản dựng là debug nạp Vite dev.
- Bàn đối chứng gỡ vùng sửa (gỡ `contenteditable`, thang K, `content-visibility`): `TAURI_WEBDRIVER_PORT=4467 npm run test:e2e -- --spec _bmad-output/implementation-artifacts/2-14-ban-do/doi-chung-vung-sua.e2e.mjs`
- Bàn đối chứng kích thước DOM (giữ N hàng đầu, gỡ phần còn lại ở cả bốn cột): `TAURI_WEBDRIVER_PORT=4467 npm run test:e2e -- --spec _bmad-output/implementation-artifacts/2-14-ban-do/doi-chung-kich-thuoc-dom.e2e.mjs`
- Bàn tách pha (JS ứng dụng so với render WebKit): `TAURI_WEBDRIVER_PORT=4467 npm run test:e2e -- --spec _bmad-output/implementation-artifacts/2-14-ban-do/tach-pha.e2e.mjs`
- Bàn đo Vue theo component (config.performance, render/patch từng lần dời con trỏ): `TAURI_WEBDRIVER_PORT=4467 npm run test:e2e -- --spec _bmad-output/implementation-artifacts/2-14-ban-do/do-vue.e2e.mjs`
- Bàn quy khoảng trống (layout, IPC, timer, handler, khối chặn luồng chính; ba biến thể gắn/gỡ dụng cụ/bấm thật): `TAURI_WEBDRIVER_PORT=4467 npm run test:e2e -- --spec _bmad-output/implementation-artifacts/2-14-ban-do/quy-khoang-trong.e2e.mjs`
- Bàn sao tách app (nhân bản `#app`, gỡ Vue, rồi bisect CSS từng thuộc tính trên bản sao): `TAURI_WEBDRIVER_PORT=4467 npm run test:e2e -- --spec _bmad-output/implementation-artifacts/2-14-ban-do/ban-sao-tach-app.e2e.mjs`
- Bàn phát lại đột biến (ghi mutation Vue mỗi lần dời con trỏ rồi phát lại bằng DOM thuần trên bản sao đã gỡ Vue): `TAURI_WEBDRIVER_PORT=4467 npm run test:e2e -- --spec _bmad-output/implementation-artifacts/2-14-ban-do/phat-lai-dot-bien.e2e.mjs`
