# Story 10.9 — Báo cáo nghiệm thu các ngưỡng phi chức năng

Mỗi hàng là một AC của plan `tickets/epic-phat-hanh-va-tin-cay/tickets/story-nghiem-thu-cuoi-cac-nguong-phi-chuc-nang-plan.md`. Mọi số dưới đây là số đo; không hàng nào được điền bằng suy luận. Không có số Windows nào cho NFR3/4/5.

| AC | Số | Nguồn | Kết luận |
|---|---|---|---|
| 1. Bộ đo 6.18 chạy trọn: p95 NFR3 theo kiểu truy vấn, NFR4 lạnh/ấm, NFR5 B/MB/MiB | p95 xấu nhất 3 phiên: `source_han_truncated` 183,9 ms · `target_explicit_lenient_truncated` 103,6 · `target_auto_widen_truncated` 103,6 · `target_exact_truncated` 103,3 · `source_latin_unique` 7,2. NFR4 (30 lần ok): lạnh trung vị 3.151,7 / max 5.770,6 ms; ấm trung vị 3.331,7 / max 6.145,0 ms. NFR5 (600 mẫu ok): `phys_footprint` 700.018.688 B · 700,0 MB · 667,6 MiB; RSS 462.450.688 B | `6-18-ban-do/REPORT.md`, `environment.txt`, `*-raw.tsv`; `03376ab`, MacBookPro16,1 i9-9980HK, macOS 15.8.2, rustc 1.97.1, `release`, 4 lớp từ điển, 2026-10-21 09:55–10:36 UTC | đạt: `summarize.mjs` in đủ ma trận. ⚠️ Tải máy 8,70 → 9,75 suốt lượt đo (một trình duyệt chạy nền) |
| 2. Ice ký số; A6/A7/A8 mỗi cái một kết luận "macOS", Q4 đóng, không còn nhãn ngưỡng tạm | A6 đạt (183,9 < 500 ms) · A7 vượt (3.151,7 > 3.000 ms) · A8 vượt (700,0 > 300 MB) | PRD hàng NFR3–5, A6–A8, Q4; `spec-auratranslate.md` A6–A8, Q4; `requirements.md` NFR3–5 | đạt: Ice ký 2026-10-21, kể cả A7 dù tải máy có thể đã đội số (các phiên yên nhất chỉ trên ngưỡng 0,2–1,7%). Số 5.14 ghi "sơ bộ, thay bởi 10.9" |
| 3. `workflow_dispatch` của `release.yml`: 4 số byte (payload + WebView2 × macOS, Windows) | macOS payload 432.363.050 B, WebView2 không nhúng (WKWebView); Windows payload 415.937.424 B, WebView2 214.156.024 B | run `39580744916` (`03376ab`), artifact `nfr6-sizes-macOS` và `nfr6-sizes-Windows`; bản xem trước ghi chú dựng cục bộ bằng `release-checksums.mjs --sizes` | đạt: bản xem trước có mục `### Dung lượng (NFR6)` với ba số và câu cố định cho WebView2 macOS. Run đầu `39576340751` ra WebView2 Windows 0 và payload 630.621.072 B (xem Plan Change Log) |
| 4. Bảng kích thước thiếu một số thì bước nối ghi chú đỏ, nêu tên số thiếu | 5 ca đỏ: payload 0, thiếu payload, WebView2 Windows 0, thiếu hệ điều hành, tệp số không có | `tests/frontend/releaseChecksums.test.ts`, `tests/frontend/msiStreams.test.ts` | đạt |
| 5. Rà giấy phép: mỗi gói có SPDX kèm kết luận GPLv3; mọi phụ thuộc trực tiếp có một hàng Stack; mọi hàng Stack có phụ thuộc thật | xem mục NFR15 bên dưới | `cargo metadata`, `package-lock.json`, spine §Stack | đạt: 610 gói Rust + 27 gói npm đóng gói, 0 gói thiếu `license`, 0 hàng Stack lệch; sửa câu "37 hàng" ở spine thành 50 |
| 6. `check-debt-owner` không còn mục mở hoặc 🟡 có chủ cuối Story 10.9 | 17 → 0 | `deferred-work.md`, `node scripts/check-debt-owner.mjs` exit 0 | đạt (xem mục Sổ nợ) |
| 7. PRD: NFR1 không còn đòi đo đầu-cuối; NFR6 mang trần mới Ice ký kèm số hai dòng; cả hai sửa tại chỗ kèm dòng cập nhật có ngày | NFR1: p95 đường Rust 1,8–2,4 ms (1.17/1.18). NFR6: trần 500.000.000 B, macOS 432.363.050 B, Windows 415.937.424 B + WebView2 214.156.024 B | PRD NFR1, A1, NFR6 (hàng bảng, sửa lần ba, A2, dự phóng, tiêu chí thành công) | đạt: Ice ký trần 500.000.000 B ngày 2026-10-21 (Q1, chọn giữa 450 và 500 triệu byte) |
| NFR16 (i18n) | `check:i18n` exit 0: 147 `.rs` + 60 `.vue`, 2065 chuỗi `vi.json` | `npm run check:i18n`, 2026-10-21 | đạt; vế tệp `.ts` giữ ngoài AC (Q4), mục nợ `Chủ: Ice` |
| WCAG / tương phản | `check:tokens` exit 0: light 35 cặp đạt AA, thấp nhất 4,542:1; dark 35 cặp, thấp nhất 4,505:1; 42 tổ hợp = 35 kiểm + 11 loại trừ đóng băng | `npm run check:tokens`, 2026-10-21 | đạt phần máy chấm; duyệt bằng mắt hai theme là mục nợ `Chủ: Epic 10` (Q5) |

## NFR15 — rà giấy phép một lần (Q3, không thêm cổng)

- Rust: `cargo metadata --locked --offline`, cây `dependencies` + `build-dependencies` của gói gốc, mọi nền tảng, tính năng mặc định: 610 gói. Nhóm: MIT OR Apache-2.0 283 · MIT 139 · Apache-2.0 OR MIT 52 · MIT/Apache-2.0 26 · Unicode-3.0 18 · Zlib OR Apache-2.0 OR MIT 17 · Apache-2.0 8 · MPL-2.0 8 · còn lại là biểu thức dễ dãi (ISC, BSD-2/3-Clause, Unlicense, 0BSD, LLVM-exception, CC0/MIT-0) với 1 gói `CDLA-Permissive-2.0` (`webpki-root-certs`) và 2 gói `r-efi` có nhánh `LGPL-2.1-or-later` bên cạnh MIT. Trường `license` rỗng: 0.
- MPL-2.0 (8 gói: `cssparser` 0.36.0/0.37.0, `cssparser-macros` 0.6.1/0.7.0, `selectors` 0.36.1/0.38.0, `dtoa-short` 0.3.5, `option-ext` 0.2.0): `grep "Exhibit B"` trong nguồn đã tải chỉ trúng văn bản `LICENSE` chuẩn của sáu gói có tệp; hai bản `selectors` không có `LICENSE*` và không tệp nguồn nào mang Exhibit B. Kết luận: tương thích GPLv3.
- `tauri-plugin-wdio-webdriver` 1.3.0 (optional, tắt mặc định, không nằm trong 610): `Cargo.toml` đã tải khai MIT.
- npm: `package-lock.json`, gói không `dev`: 27 gói (MIT 23 · Apache-2.0 OR MIT 1 · BSD-2-Clause 1 · BSD-3-Clause 1 · ISC 1), 0 gói thiếu `license`. Phần còn lại của 713 mục là công cụ build, đã rà ở lượt 10.5.
- Đối chiếu Stack: 25 tên phụ thuộc trực tiếp trong `Cargo.toml` (gồm dev và build) và 20 trong `package.json` đều có đúng một hàng, phiên bản trùng `Cargo.lock`/`package-lock.json`. Hàng Stack không phải phụ thuộc trực tiếp (`libsqlite3-sys`, `open`, `is-wsl`, `is-docker`, `displayplacer`, ba font) đều có thật. Sửa duy nhất: câu "37 hàng (34+3)" ở spine, nay ghi 50 hàng kèm dòng cập nhật. `scripts/lib/msi-streams.mjs` mới chỉ dùng `node:fs`, không thêm phụ thuộc.

## Sổ nợ — 17 mục `Chủ: Story 10.9`

Lượt đầu: `:945`, `:1178`, `:1187` → KHÔNG LÀM (Q2); `:203` → 🟡 `Chủ: Epic 10`; `:36` → 🟡 `Chủ: Ice`; `:50` (`.ts`) → 🟡 `Chủ: Ice`. Ba mục mới cuối sổ: vòng không chạm chuột macOS và tương phản hai theme (`Chủ: Epic 10`), nửa Windows của NFR14/NFR17 (`Chủ: B7`). Mục thông báo tác giả (`:393`) giữ `Chủ: Ice`.

Sau lượt đo (tên mục, vì số dòng trôi):

- ✅ NFR3 sơ bộ của 5.9 — đóng bằng p95 183,9 ms.
- 🟡 `Chủ: Ice`: dư địa NFR6 (trần mới, HVTĐTD và Cổ hán văn vẫn phải đo trước khi đóng gói) · NFR4 vượt và máy chưa nhàn · NFR5 vượt và đỉnh RSS chuỗi nhập chưa đo lại · "hàng trăm `.atproj`" (bộ đo vẫn là 50 Tác phẩm).
- Chuyển `Chủ: Ice` vì bộ đo 10.9 không đo đường đó: độ trễ mở Chương với `Jieba` · khung hình lúc mở Chương · sao chép kép khi dời con trỏ xem trước URL · trần số Chương một mẫu · lọc ép hiện trọn danh sách trên WKWebView · clone Glossary khi mở Chương · bộ đếm truy vấn Glossary.
