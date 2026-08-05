Dữ liệu từ điển ở dạng `dict-core.db` (lớp nền) + **mỗi lớp gỡ rời một file `.db` độc lập** (AD-10).

⛔ **File `.db` KHÔNG nằm trong git.** Chúng là artifact có phiên bản và checksum, tải theo `dict-manifest.toml` ở gốc repo (AD-25). Dòng `*.db` trong `.gitignore` là cố ý — đừng gỡ.

**Tệp nào tồn tại (2026-08-05, Story 1.10b):** BA tệp, sinh bởi `tools/dict-build`:

| Tệp | Lớp | Nguồn |
|---|---|---|
| `dict-core.db` | NỀN | **SÁU** nguồn: CVDICT · CC-CEDICT · Unihan · viwiktionary *(vai B, mục tiếng Trung)* · en.wiktionary *(Story 1.9)* · **viwiktionary-en** *(vai A, mục tiếng Anh — Story 1.10b, FR34)* |
| `dict-thieu-chuu.db` | GỠ RỜI | Thiều Chửu — Hán Việt Tự Điển *(Story 1.10)* |
| `dict-vietphrase.db` | GỠ RỜI | VietPhrase *(Story 1.10)* |

Checksum + URL thật ghi ở `../../dict-manifest.toml` (`[base]` + hai khối `[[detachable]]`). **Chưa tải lên GitHub Release** — Ice cần chạy lệnh ở Completion Notes của Story 1.10b để tạo release `dict-v1` với cả ba tệp. Hai lớp gỡ rời còn lại (HVTĐTD · Cổ hán văn) **chưa tồn tại** — chưa có nguồn thô, xem `deferred-work.md`.

🔴 **`dict-core.db` đã đổi LẦN HAI ở Story 1.10b** — `154.464.256` → **`194.998.272` byte**, sha `741e1666…` → **`2145c7ae…`**. Nguồn nền thứ sáu `viwiktionary-en` mang **119.039 đầu mục tiếng Anh** *(FR34)*; trước đó tệp này **100% `lang='zh'`**. Nếu bản cũ đã được tải lên release, cần `--clobber` **cả ba** tệp để ba checksum thuộc **một thế hệ dữ liệu**.

⚠️ **Vùng này KHÔNG còn nằm trong `assetProtocol.scope`** (Story 1.9, Task 10 — Ice phê chuẩn 2026-08-04). Webview không bao giờ đọc tệp từ điển; `rusqlite` mở tệp bằng đường dẫn hệ thống ở tầng Rust (AD-1, AD-11), không đi qua asset protocol. Đóng gói vào `bundle.resources` (và lưới nghiệm thu thay thế) là **Story 10.1** — phạm vi giờ là BA tệp, không phải một.

**Story sở hữu:** 1.9 · 1.10 (dựng dữ liệu, `tools/dict-build/`) · 10.1 (đưa vào bản phát hành, đóng gói `bundle.resources`).
