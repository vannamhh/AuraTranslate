# Phép thử ba engine — gõ trong lưới 9.850 hàng

Trang tổng hợp `grid.html` (không Vue, không app): `?v=subgrid|chunk|rows&n=9850`, gọi 20 lần `execCommand('insertText')` ở ô bản dịch hàng giữa, ghi thời gian đồng bộ và tới frame kế.

- Headless WebKit + Chromium: `PW_PATH=<thư mục node_modules/playwright> node run-pw.mjs` → `ket-qua-headless.jsonl`.
- WKWebView của hệ điều hành (mở một cửa sổ): `swift run-wkwebview.swift "$PWD"` (ngoài sandbox) → `ket-qua-wkwebview.jsonl`.

Kết quả và kết luận: `_bmad-output/initiative-auratranslate/research-go-cham-luoi-subgrid-webkit/research-go-cham-luoi-subgrid-webkit.md`, mục D4.
