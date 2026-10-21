# Gỡ một nguồn từ điển khỏi bản phát hành

Quy trình này thực thi FR112: gỡ một lớp gỡ rời là xoá một `.db` khỏi `dict-manifest.toml`, không đổi mã. Ứng dụng quét `$RESOURCE/dict/*.db` lúc chạy (`DictLayers::open`) và dựng Attribution từ `dict_source` của các tệp có mặt, nên một tệp vắng mặt là một nguồn vắng mặt (FR36). Dữ liệu từ điển không nằm trong git (AD-25).

Chỉ lớp gỡ rời gỡ được theo cách này. Lớp nền `dict-core.db` không gỡ được. Lớp HVTĐTD chưa có nguồn nên chưa tồn tại; nếu về sau có và bị gỡ, tra Hán Việt rơi về nhãn tiếng Anh (`pos_lang='en'`), phần này được ca fixture trong `src-tauri/tests/dict_sources.rs` phủ.

## Các bước

1. Trong `dict-manifest.toml`, chuyển khối `[[detachable]]` của lớp thành khối `[[removed]]` với `name`, `reason`, `date`; bỏ `url`, `sha256`, `source_version`. Chỉ xoá khối mà không ghi `[[removed]]` thì cổng báo lớp bị rơi mất.
2. Xoá `src-tauri/resources/dict/dict-<tên>.db` cục bộ. `scripts/fetch-dicts.mjs` không dọn thư mục này; để lại tệp thì `inspect-release` đỏ vì có `.db` ngoài manifest.
3. Sửa tay `NOTICE.txt`: bỏ tên tệp khỏi danh sách và ghi lớp đã gỡ.
4. Giấy phép riêng của lớp trong `tools/dict-build/assets/licenses/` vẫn được đóng gói (`licenses.rs` nhúng chúng bằng `include_str!` và `tauri.conf.json` đóng gói glob `assets/licenses/*.txt`); không xoá chúng.

## Kiểm

- `npm run check:dict-manifest`
- `npm run check:dict`
- `cargo test --test config_invariants`
- `npx vitest run tests/frontend/fetchDicts.test.ts`
- Bản đóng gói: `npx tauri build --config src-tauri/tauri.release.conf.json --bundles app`, rồi `node scripts/inspect-release.mjs`; mở Attribution và xác nhận ghi công của lớp đã gỡ không còn.

Sau các bước trên, `git diff` chỉ chạm `dict-manifest.toml`, `NOTICE.txt` và tài liệu.
