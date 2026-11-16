# Ảnh giao diện AuraTranslate

Các ảnh PNG là ảnh chụp webview của app Tauri thật trên macOS, không dựng lại bằng HTML và không chỉnh sửa nội dung ảnh.

Tác phẩm “Một buổi sáng bên sông — bản minh họa” được tạo riêng từ nội dung tự soạn trong [sample-content.json](sample-content.json). Tác phẩm có tám câu Trung–Việt, năm câu đã xác nhận và ba thuật ngữ minh họa. Ảnh không sử dụng dữ liệu tác phẩm đang test của Ice.

Phiên chụp dùng bản debug dựng bằng `cargo build --locked --features wdio --manifest-path src-tauri/Cargo.toml`, bộ chạy `e2e/wdio.conf.mjs`, cổng WebDriver riêng 4459 và cả hai thư mục tạm `AURATRANSLATE_E2E_DATA_DIR` / `AURATRANSLATE_E2E_LIBRARY_ROOT`. Trước khi ghi, script kiểm tra kho toàn cục nằm trong thư mục tạm; sau khi tạo, kiểm tra Tác phẩm mới nằm trong Library tạm. Bộ chạy giữ hàng rào dữ liệu thật của dự án.

| Tệp | Màn hình |
| --- | --- |
| `workspace.png` | Lưới đối chiếu nguyên văn và bản dịch |
| `glossary.png` | Ba thuật ngữ của riêng tác phẩm minh họa |
| `translation-memory.png` | Các cặp câu được tạo từ những lần xác nhận |
| `reading.png` | Đọc bản dịch và phân biệt câu chưa xác nhận |
| `export.png` | Xem trước xuất DOCX hai cột |

Ảnh chụp ngày 2026-11-16; các số trong ảnh thuộc tác phẩm minh họa, không phải số đo hiệu năng hay quy mô dữ liệu sản phẩm.
