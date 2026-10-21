---
title: 'Story 10.5 — Giấy phép trong bản phát hành'
type: 'feature'
ticket: 5
created: '2026-10-21'
status: 'built'
baseline_revision: '95936b4db17b91929abbce14701ea1ed7c70feb8'
route: 'full'
route_source: 'auto'
risk: 'medium'
review: 'quick'
review_source: 'pinned'
lenses_ran: [quick]
review_loop_iteration: 0
context:
  - '{project-root}/scripts/AGENTS.md'
  - '{project-root}/tools/dict-build/AGENTS.md'
---

<frozen-after-approval reason="human-owned intent — do not modify unless human renegotiates">

## Intent

**Problem:** Bản cài chỉ mang `license/COPYING.txt` (GPL v3) và ba giấy phép font. Giấy phép của bảy bộ dữ liệu chỉ nằm trong cột `dict_source.license_text` của từng `.db`, nên người nhận bản cài không có chúng ở dạng tệp. Repo không có `LICENSE`/`NOTICE` nói phần nào không thuộc GPL v3. Bảng Stack thiếu `tauri-plugin-dialog` và khai "mọi giấy phép permissive", dù có ba crate bắc cầu MPL-2.0. Trong khi đó màn hình Attribution đã hứa "Toàn bộ giấy phép kèm trong bản phát hành" (`vi.json:1427`).

**Approach:** Đưa văn bản giấy phép của từng bộ dữ liệu vào `license/` của bản cài. Viết `NOTICE` ở gốc repo ánh xạ nguồn → giấy phép → tệp, và cũng đóng gói nó. Thêm một kiểm trong `inspect-release` để bản cài thiếu tệp nào thì đỏ. Rà lại toàn bộ phụ thuộc trực tiếp và bắc cầu, ghi kết quả có ngày vào bảng Stack của spine.

## Boundaries & Constraints

**Always:** Lấy văn bản giấy phép từ `tools/dict-build/assets/licenses/`, chính nguồn mà `licenses.rs` nhúng vào `.db`, không viết lại. `NOTICE` nói mỗi nguồn theo đúng `license_kind`/`license_id` của `sources_meta.rs` (`unknown`, `copyrighted` ghi đúng như vậy, trỏ tệp tuyên bố của nguồn đó). Ghi rõ nghĩa vụ share-alike cho dữ liệu phái sinh từ nguồn CC-BY-SA-4.0.

**Decisions (Ice, 2026-10-21):**
1. HVTĐTD: `NOTICE` không viết điều khoản HVTĐTD, vì dữ liệu đó không tới (Ice chốt 2026-08-08, `dictSourcesState.ts:256`). Vế AC này ghi KHÔNG LÀM có lý do.
2. Tệp giấy phép dữ liệu: thêm một mục map trong `bundle.resources` trỏ thẳng `../tools/dict-build/assets/licenses/*.txt` → `license/`, không chép bản sao.
3. Cây bắc cầu: quét một lần, ghi kết quả có ngày vào spine §Stack, không thêm cổng.
4. Phạm vi: giữ trọn năm việc trong một plan.

**Never:** Không đổi giấy phép của mã nguồn. Không thêm phụ thuộc (không `cargo-deny`/`cargo-about`/`license-checker`). Không sửa `sources_meta.rs`, các `.db`, `dict-manifest.toml`, hay chính sách đưa nguồn nào vào bản cài. Không viết điều khoản pháp lý mà `assets/licenses/` không nói.

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected Output / Behavior | Error Handling |
|----------|--------------|---------------------------|----------------|
| Bản cài đủ | `.app`/`msiexec /a` của build phát hành | `license/` có `COPYING.txt`, `NOTICE.txt` và đủ bảy tệp dữ liệu, mỗi tệp byte-giống nguồn | `inspect-release` exit 0 |
| Thiếu một tệp | Gỡ một tệp giấy phép khỏi cây đã cài | `inspect-release` nêu đúng tên tệp thiếu | exit 1 |
| Tệp lệch nguồn | Một byte khác ở tệp trong bản cài | `inspect-release` nêu tệp lệch SHA-256 | exit 1 |

</frozen-after-approval>

## Code Map

- `src-tauri/tauri.conf.json:35-40` -- `bundle.resources`: `resources/license/*.txt` → `license/`, `resources/fonts/*` → `fonts/`. Đây là wiring dùng chung, chạm vào thì phải chạy trọn bộ.
- `src-tauri/resources/license/COPYING.txt` -- bản sao byte-giống `COPYING` ở gốc, không có cổng nào giữ đồng bộ.
- `tools/dict-build/assets/licenses/*.txt` -- bảy tệp, 60 452 B: CC-BY-SA-4.0, CC0-1.0, GFDL-1.3, Unicode-License-v3, thieu-chuu, tran-van-chanh, vietphrase. `tools/dict-build/src/licenses.rs:15-23` nhúng chúng vào `.db`.
- `tools/dict-build/src/sources_meta.rs:58-187` -- `license_kind`/`license_id` của từng nguồn (cvdict, cc-cedict, bốn wiktionary, unihan, thieu-chuu, vietphrase, tran-van-chanh). Chỉ đọc.
- `scripts/inspect-release.mjs` (64 dòng) và `scripts/lib/dict-fetch.mjs:169` (`inspectInstalledFonts`) -- khuôn kiểm cây đã cài theo SHA-256. Kiểm giấy phép làm theo đúng khuôn này. `release.yml` đã gọi nó.
- `.github/workflows/release.yml:64-183` -- `build` gọi `tauri build`; `publish` tải lên `.dmg`, `.msi`, `SHA256SUMS.txt`.
- `_bmad-output/initiative-auratranslate/architecture-auratranslate/architecture-auratranslate.md:1163-1321` -- §Stack: bảng 3 cột (`Name | Version | Giấy phép`), 49 hàng. Thiếu `tauri-plugin-dialog =2.7.2` (chỉ có ở bảng con AD-48 :1235). Câu :1225 "mọi giấy phép permissive" sai với `cssparser`, `cssparser-macros`, `selectors` (MPL-2.0). Câu :1227 ghi đã quét 530 gói npm. Dùng offset/limit khi đọc.
- `src/i18n/vi.json:1427` -- câu hứa của Attribution. Không đổi, story này làm nó thành đúng.
- `_bmad-output/initiative-auratranslate/deferred-work.md:26-27` -- nợ `Chủ: Story 10.5`: chưa đối chiếu name ID 13/14 trong ba tệp font với `OFL.txt`/`LICENSE`.

## Tasks & Acceptance

**Execution:**
- [x] `NOTICE` (gốc, mới) -- tên dự án, GPL-3.0-or-later cho mã (trỏ `COPYING`). Dữ liệu từ điển **không** thuộc GPL v3: bảng nguồn → `license_kind`/`license_id` → tệp trong `license/`. Câu share-alike cho dữ liệu phái sinh CC-BY-SA. Font: trỏ ba tệp trong `fonts/`. Không nhắc HVTĐTD (Decision 1).
- [x] `src-tauri/tauri.conf.json` -- thêm mục map `../tools/dict-build/assets/licenses/*.txt` → `license/` và đưa `NOTICE` ở gốc vào `license/NOTICE.txt` (Decision 2). Kiểm trên bản build thật rằng Tauri nhận đường `../` và tên tệp không đụng `COPYING.txt`. Chạy trọn bộ một lần vì đây là wiring dùng chung.
- [x] `_bmad-output/initiative-auratranslate/deferred-work.md` -- một mục mới đóng ngay bằng `→ KHÔNG LÀM 2026-10-21 (Story 10.5)`: vế HVTĐTD của FR110 không viết vì dữ liệu không tới, trỏ quyết định 2026-08-08.
- [x] `scripts/inspect-release.mjs` (+ `scripts/lib/dict-fetch.mjs` nếu tách hàm) -- kiểm `license/`: đủ tệp, mỗi tệp SHA-256 khớp nguồn, thiếu hoặc lệch thì exit 1 và nêu tên tệp. Ca test thêm vào `tests/frontend/fetchDicts.test.ts`, nơi các ca `inspectInstalledFonts` đang sống, phủ ba hàng của Matrix.
- [x] Spine §Stack -- thêm hàng `tauri-plugin-dialog`; sửa tại chỗ câu :1225; quét một lần cây bắc cầu (`cargo metadata --locked` trường `license`; `node_modules/**/package.json` trường `license`), ghi một dòng kết quả có ngày, nêu tên mọi giấy phép không permissive (Decision 3).
- [x] Font name ID 13/14 -- đọc bảng `name` của ba tệp `src-tauri/resources/fonts/*` bằng một script dùng một lần trong scratchpad, không phụ thuộc mới. Đối chiếu với tệp giấy phép đi kèm, rồi đóng mục `deferred-work.md:26` bằng số đo.

**Acceptance Criteria:**
- Given bản cài phát hành, when mở `license/`, then có GPL v3 đầy đủ, `NOTICE` và văn bản giấy phép của mọi bộ dữ liệu có trong `dict/`.
- Given `NOTICE`, when đọc, then thấy dữ liệu từ điển không thuộc GPL v3, từng nguồn đúng `license_kind` như `sources_meta.rs`, và nghĩa vụ share-alike.
- Given bảng Stack, when so với `src-tauri/Cargo.toml` và `package.json`, then mọi phụ thuộc trực tiếp có hàng, và có một dòng kết quả rà cây bắc cầu ghi ngày, nêu tên mọi giấy phép không permissive.
- Given `deferred-work.md:26`, when đọc dòng cuối, then nó đã đóng bằng giá trị name ID 13/14 đo thật.

## Implementation Notes

- Tauri nhận đường `../` trong map `bundle.resources`. `license/` của `.app` release có `COPYING.txt`, `NOTICE.txt` và bảy tệp dữ liệu, không đụng tên. Bước `.dmg` không chạy được trong sandbox, `.msi` chưa thử.
- `config_invariants.rs::base_config_never_carries_the_dictionary` đỏ vì đường mới chứa chuỗi `dict-build`. Đã thu hẹp về: nguồn `.db`, một đoạn đường đúng bằng `dict`, hoặc đích bắt đầu bằng `dict`. Đối chứng: đưa `"resources/dict/*.db": "dict/"` vào cấu hình gốc thì đỏ đúng thông báo.
- Đo bảng `name` của font thấy ID 0 của Source Serif 4 khai Reserved Font Name 'Source', trái với `fonts/README.md` và spine. Ghi thành mục nợ mở `Chủ: Ice`.
- Trọn bộ chạy một lần sau khi chạm `tauri.conf.json`: vitest 208 tệp / 3 001 ca xanh; `cargo test --no-fail-fast` không ca đỏ (lượt cuối chỉ in dòng lỗi, không in số ca đạt).
- Quét bắc cầu: 611 gói Rust, 522 gói npm. Giấy phép ngoài nhóm dễ dãi ghi ở spine §Stack, dòng 🔵 2026-10-21.

## Plan Change Log

## Review Triage Log

Lượt 1, 2026-10-21, lens `quick`: 2 medium · 1 low vá · 1 low bác · 1 false.

- Câu "chỉ Source Sans 3 khai RFN" còn ở spine và `fonts/README.md` — `medium`, patch: đọc lại name ID 0 độc lập (cả hai `SourceSerif4` khai RFN 'Source'), spine sửa tại chỗ bằng 🔵, README thay câu, mục nợ RFN đóng.
- Nhánh "found 2 copies" của `inspectInstalledLicenses` không có ca — `low`, patch: thêm ca bản trùng, `fetchDicts.test.ts` 22/22.
- `.msi`/`.dmg` chưa kiểm, trọn bộ không ghi lại — `medium`, patch: mục nợ mới `Chủ: B7`; kết quả trọn bộ ghi ở Implementation Notes.
- `base_config_never_carries_the_dictionary` yếu đi — `low`, bác: ví dụ `→ dictionary/` vẫn đỏ vì đích bắt đầu bằng `dict`; chỉ còn lọt một tệp từ điển không `.db` map tới đích không mang `dict`, không có đường nào trong repo; đối chứng `resources/dict/*.db` đỏ đúng lý do.
- Mục HVTĐTD đóng KHÔNG LÀM không có chủ để mở lại — `false`: mục đã đóng thì không cần chủ, `check:debt-owner` xanh.

## Verification

**Commands:**
- `npx vitest run tests/frontend/fetchDicts.test.ts` -- expected: các ca Matrix xanh.
- `npm run build && npm run tauri build -- --config src-tauri/tauri.release.conf.json`, rồi `node scripts/inspect-release.mjs --root <.app> --installer <.dmg>` -- expected: exit 0. Gỡ một tệp trong `license/` của `.app` rồi chạy lại -- expected: exit 1 nêu tên tệp.
- Trọn bộ một lần vì chạm `tauri.conf.json` (AGENTS.md §Tests).
