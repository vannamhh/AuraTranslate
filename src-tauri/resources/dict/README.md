Dữ liệu từ điển ở dạng `dict-core.db` (lớp nền) + **mỗi lớp gỡ rời một file `.db` độc lập** (AD-10).

⛔ **File `.db` KHÔNG nằm trong git.** Chúng là artifact có phiên bản và checksum, tải theo `dict-manifest.toml` ở gốc repo (AD-25). Dòng `*.db` trong `.gitignore` là cố ý — đừng gỡ.

**Tệp nào tồn tại (2026-08-04):** `dict-core.db` — lớp NỀN, năm nguồn (CVDICT · CC-CEDICT · Unihan · viwiktionary · en.wiktionary), sinh bởi `tools/dict-build` (Story 1.9). Checksum + URL thật ghi ở `../../dict-manifest.toml` §`[base]`. **Chưa tải lên GitHub Release** — Ice cần chạy lệnh ở Completion Notes của Story 1.9 trước khi ai tải được. Bốn lớp gỡ rời (Thiều Chửu · Cổ hán văn · VietPhrase · HVTĐTD) **chưa tồn tại** — Story 1.10.

⚠️ **Vùng này KHÔNG còn nằm trong `assetProtocol.scope`** (Story 1.9, Task 10 — Ice phê chuẩn 2026-08-04). Webview không bao giờ đọc tệp từ điển; `rusqlite` mở tệp bằng đường dẫn hệ thống ở tầng Rust (AD-1, AD-11), không đi qua asset protocol. Đóng gói vào `bundle.resources` (và lưới nghiệm thu thay thế) là **Story 10.1**.

**Story sở hữu:** 1.9 (dựng dữ liệu, `tools/dict-build/`) · 10.1 (đưa vào bản phát hành, đóng gói `bundle.resources`).
