Dữ liệu từ điển ở dạng `dict-core.db` (lớp nền) + **mỗi lớp gỡ rời một file `.db` độc lập** (AD-10).

⛔ **File `.db` KHÔNG nằm trong git.** Chúng là artifact có phiên bản và checksum, tải theo `dict-manifest.toml` ở gốc repo (AD-25). Dòng `*.db` trong `.gitignore` là cố ý — đừng gỡ.

**Tệp nào tồn tại (2026-08-05, Story 1.10):** BA tệp *(1 → 3)*, sinh bởi `tools/dict-build`:

| Tệp | Lớp | Nguồn |
|---|---|---|
| `dict-core.db` | NỀN | CVDICT · CC-CEDICT · Unihan · viwiktionary · en.wiktionary *(Story 1.9)* |
| `dict-thieu-chuu.db` | GỠ RỜI | Thiều Chửu — Hán Việt Tự Điển *(Story 1.10)* |
| `dict-vietphrase.db` | GỠ RỜI | VietPhrase *(Story 1.10)* |

Checksum + URL thật ghi ở `../../dict-manifest.toml` (`[base]` + hai khối `[[detachable]]`). **Chưa tải lên GitHub Release** — Ice cần chạy lệnh ở Completion Notes của Story 1.10 để tạo release `dict-v1` với cả ba tệp. Hai lớp gỡ rời còn lại (HVTĐTD · Cổ hán văn) **chưa tồn tại** — chưa có nguồn thô, xem `deferred-work.md`.

⚠️ **Vùng này KHÔNG còn nằm trong `assetProtocol.scope`** (Story 1.9, Task 10 — Ice phê chuẩn 2026-08-04). Webview không bao giờ đọc tệp từ điển; `rusqlite` mở tệp bằng đường dẫn hệ thống ở tầng Rust (AD-1, AD-11), không đi qua asset protocol. Đóng gói vào `bundle.resources` (và lưới nghiệm thu thay thế) là **Story 10.1** — phạm vi giờ là BA tệp, không phải một.

**Story sở hữu:** 1.9 · 1.10 (dựng dữ liệu, `tools/dict-build/`) · 10.1 (đưa vào bản phát hành, đóng gói `bundle.resources`).
