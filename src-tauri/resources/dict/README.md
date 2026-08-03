Dữ liệu từ điển ở dạng `dict-core.db` (lớp nền) + **mỗi lớp gỡ rời một file `.db` độc lập** (AD-10).

⛔ **File `.db` KHÔNG nằm trong git.** Chúng là artifact có phiên bản và checksum, tải theo `dict-manifest.toml` ở gốc repo (AD-25). Dòng `*.db` trong `.gitignore` là cố ý — đừng gỡ.

Vùng này được khai `$RESOURCE/dict/**` **chỉ đọc** trong `assetProtocol.scope` (AD-23).

**Story sở hữu:** 1.9 (dựng dữ liệu, `tools/dict-build/`) · 10.1 (đưa vào bản phát hành).
