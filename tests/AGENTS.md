<!-- bmad:context -->
<!-- Verified 2026-08-25 against 69b19a8. Managed by bmad-project-context; edits inside this block are replaced on refresh. -->

## tests/frontend/ — vitest

Vai: hành vi của module thuần, mã đụng DOM, và `.vue`. Bốn đường nghiệm thu không chồng nhau — cổng tĩnh `scripts/check-*.mjs` (mệnh đề khai báo trên toàn cây) · `src-tauri/tests/**` (hợp đồng, ranh giới, bất biến cấu hình) · vitest · `e2e/**` (WKWebView/WebView2 thật). Trước khi viết một phép kiểm mới, hỏi: mệnh đề này đã có chủ ở đường nào chưa. Hai đường cùng canh một mệnh đề là hai nguồn sự thật.

## Conventions that differ from defaults

- `happy-dom` KHÔNG phải WebKit. Mọi mệnh đề về hình học, bố cục, hay engine thật thuộc bàn đo/e2e — không thuộc vitest.
- Test sống ở `tests/frontend/**`, KHÔNG đồng vị trí trong `src/**`: bốn cổng đếm quần thể `src/**` và một tệp test đổ vào đó thổi phồng mẫu số, cộng hai va chạm (`check-i18n` Kiểm A đỏ với chữ tiếng Việt, `check-tokens` Kiểm B đỏ với màu viết thẳng).
- `tsconfig.json` phải `include` cây test — một cây test không được kiểm kiểu là một cây test sẽ mục: nó vẫn chạy xanh trong khi kiểu của thứ nó kiểm đã đổi dưới chân.
- Mọi vá `happy-dom` sống ở `tests/frontend/support/setup.ts`, mỗi mục kèm một dòng nói nó thiếu gì và AI ĐỌC NÓ. Danh sách đó là một món nợ đo được — hôm nay 3 mục, hai trong số đó đã không còn ai đọc.
- Không `vi.useFakeTimers()` khi hàm đã nhận thời điểm qua tham số: bọc đồng hồ giả là đổi một bảo đảm lấy một thói quen.

## Known pitfalls

- 🔴 Đường sai rất rẻ và phải chặn bằng tay: thêm một `?.` vào MÃ SẢN PHẨM cho hết đỏ. Đó là một nhánh mà kiểu nói không bao giờ chạy — mã chết vĩnh viễn trong sản phẩm để phục vụ một bản mô phỏng. Khoảng thiếu của bản mô phỏng vá ở `setup.ts`; khuyết tật sản phẩm vá trong `src/`.

<!-- /bmad:context -->
