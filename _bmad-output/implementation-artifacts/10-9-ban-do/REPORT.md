# Story 10.9 — Báo cáo nghiệm thu các ngưỡng phi chức năng

Mỗi hàng là một AC của plan `tickets/epic-phat-hanh-va-tin-cay/tickets/story-nghiem-thu-cuoi-cac-nguong-phi-chuc-nang-plan.md`. Hàng ghi "chờ Phase M" hoặc "chờ dispatch" chưa có số; không hàng nào được điền bằng suy luận.

| AC | Số | Nguồn | Kết luận |
|---|---|---|---|
| 1. Bộ đo 6.18 chạy trọn: p95 NFR3 theo kiểu truy vấn, NFR4 lạnh/ấm, NFR5 B/MB/MiB | chờ Phase M | `6-18-ban-do/REPORT.md` sinh bởi `run.sh` + `summarize.mjs` | chờ Phase M |
| 2. Ice ký số; A6/A7/A8 mỗi cái một kết luận "macOS", Q4 đóng, không còn nhãn ngưỡng tạm | chờ Phase M | PRD :916–918, :1177–1179, :1222 | chờ Phase M, rồi Phase P sau khi Ice ký |
| 3. `workflow_dispatch` của `release.yml`: 4 số byte (payload + WebView2 × macOS, Windows) | chờ dispatch | artifact của lượt `release.yml` | chờ dispatch (Phase R viết mã, cần push) |
| 4. Bảng kích thước thiếu một số thì bước nối ghi chú đỏ, nêu tên số thiếu | chờ Phase R | `scripts/inspect-release.mjs`, test cạnh `release-checksums` | chờ Phase R |
| 5. Rà giấy phép: mỗi gói có SPDX kèm kết luận GPLv3; mọi phụ thuộc trực tiếp có một hàng Stack; mọi hàng Stack có phụ thuộc thật | xem mục NFR15 bên dưới | `cargo metadata`, `package-lock.json`, spine §Stack | đạt: 610 gói Rust + 27 gói npm đóng gói, 0 gói thiếu `license`, 0 hàng Stack lệch; sửa câu "37 hàng" ở spine thành 50 |
| 6. `check-debt-owner` không còn mục mở hoặc 🟡 có chủ cuối Story 10.9 | 17 → 12 mục còn chủ Story 10.9 | `deferred-work.md` | một phần: 5 mục đã đóng hoặc chuyển chủ (xem mục Sổ nợ); 12 mục chờ số của Phase M. Cổng `check:debt-owner` xanh |
| 7. PRD: NFR1 không còn đòi đo đầu-cuối; NFR6 mang trần mới Ice ký kèm số hai dòng; cả hai sửa tại chỗ kèm dòng cập nhật có ngày | NFR1: p95 đường Rust 1,8–2,4 ms (đã đo ở 1.17/1.18) | PRD :914 và A1 | NFR1 đạt (sửa 2026-10-21); NFR6 chờ dispatch + Ice ký (Phase P) |
| NFR16 (i18n) | `check:i18n` exit 0: 147 `.rs` + 60 `.vue`, 2065 chuỗi `vi.json` | `npm run check:i18n`, 2026-10-21, cây ở `9b71b27` + sửa của Phase S | đạt; vế tệp `.ts` giữ ngoài AC (Q4), mục nợ `Chủ: Ice` |
| WCAG / tương phản | `check:tokens` exit 0: light 35 cặp đạt AA, thấp nhất 4,542:1; dark 35 cặp, thấp nhất 4,505:1; 42 tổ hợp = 35 kiểm + 11 loại trừ đóng băng | `npm run check:tokens`, 2026-10-21 | đạt phần máy chấm; duyệt bằng mắt hai theme là mục nợ `Chủ: Epic 10` (Q5) |

## NFR15 — rà giấy phép một lần (Q3, không thêm cổng)

- Rust: `cargo metadata --locked --offline`, cây `dependencies` + `build-dependencies` của gói gốc, mọi nền tảng, tính năng mặc định: 610 gói. Nhóm: MIT OR Apache-2.0 283 · MIT 139 · Apache-2.0 OR MIT 52 · MIT/Apache-2.0 26 · Unicode-3.0 18 · Zlib OR Apache-2.0 OR MIT 17 · Apache-2.0 8 · MPL-2.0 8 · còn lại là biểu thức dễ dãi (ISC, BSD-2/3-Clause, Unlicense, 0BSD, LLVM-exception, CC0/MIT-0) với 1 gói `CDLA-Permissive-2.0` (`webpki-root-certs`) và 2 gói `r-efi` có nhánh `LGPL-2.1-or-later` bên cạnh MIT. Trường `license` rỗng: 0.
- MPL-2.0 (8 gói: `cssparser` 0.36.0/0.37.0, `cssparser-macros` 0.6.1/0.7.0, `selectors` 0.36.1/0.38.0, `dtoa-short` 0.3.5, `option-ext` 0.2.0): `grep "Exhibit B"` trong nguồn đã tải chỉ trúng văn bản `LICENSE` chuẩn của sáu gói có tệp; hai bản `selectors` không có `LICENSE*` và không tệp nguồn nào mang Exhibit B. Kết luận: tương thích GPLv3.
- `tauri-plugin-wdio-webdriver` 1.3.0 (optional, tắt mặc định, không nằm trong 610): `Cargo.toml` đã tải khai MIT.
- npm: `package-lock.json`, gói không `dev`: 27 gói (MIT 23 · Apache-2.0 OR MIT 1 · BSD-2-Clause 1 · BSD-3-Clause 1 · ISC 1), 0 gói thiếu `license`. Phần còn lại của 713 mục là công cụ build, đã rà ở lượt 10.5.
- Đối chiếu Stack: 25 tên phụ thuộc trực tiếp trong `Cargo.toml` (gồm dev và build) và 20 trong `package.json` đều có đúng một hàng, phiên bản trùng `Cargo.lock`/`package-lock.json`. Hàng Stack không phải phụ thuộc trực tiếp (`libsqlite3-sys`, `open`, `is-wsl`, `is-docker`, `displayplacer`, ba font) đều có thật. Sửa duy nhất: câu "37 hàng (34+3)" ở spine, nay ghi 50 hàng kèm dòng cập nhật.

## Sổ nợ — 17 mục `Chủ: Story 10.9`

Đã xử lý (Phase S): `:945`, `:1178`, `:1187` → KHÔNG LÀM (Q2); `:203` → 🟡 `Chủ: Epic 10`; `:36` → 🟡 `Chủ: Ice`. Mục `:50` (`.ts`) → 🟡 `Chủ: Ice`. Ba mục mới cuối sổ: vòng không chạm chuột macOS và tương phản hai theme (`Chủ: Epic 10`), nửa Windows của NFR14/NFR17 (`Chủ: B7`). Mục thông báo tác giả (`:393`) giữ `Chủ: Ice`.

Còn chờ số của Phase M hoặc Phase P (12 mục, số dòng ở cây hiện tại): `:413` (dư địa NFR6), `:5780` và `:5906` (độ trễ mở Chương, `Jieba` đồng bộ), `:8017` (NFR3), `:8072` ("hàng trăm `.atproj`"), `:8200` (NFR4), `:8946` (đỉnh RSS, NFR5), `:9286` (sao chép kép khi dời con trỏ), `:9315` (trần số Chương một mẫu), `:9978` (lọc ép hiện trọn danh sách), `:11998` (clone Glossary khi mở Chương), `:12012` (bộ đếm truy vấn Glossary).
