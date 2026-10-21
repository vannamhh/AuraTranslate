---
title: 'Story 10.7 — Cập nhật chỉ kiểm tra và thông báo'
type: 'feature'
ticket: '7'
created: '2026-10-21'
status: 'built'
baseline_revision: '043cd0779bf2b24d64a4ca705f4e285b9f2c8029'
route: 'full'
route_source: 'auto'
risk: 'medium'
review: 'quick'
review_source: 'pinned'
lenses_ran: ['quick']
review_loop_iteration: 0
context: []
---

<frozen-after-approval reason="human-owned intent — do not modify unless human renegotiates">

## Intent

**Problem:** Mục Cài đặt → Cập nhật của 12.18 (`src/SettingsUpdateSection.vue`) mới chỉ có chữ. Người dùng không xem được phiên bản đang chạy, không kiểm được có bản mới hay không, và chưa có điểm ra mạng (2) của AD-15 (FR111).

**Approach:** Thêm một lệnh Rust chỉ chạy khi người dùng bấm. Lệnh đọc danh sách bản phát hành từ một host cố định, chọn tag ứng dụng `vX.Y.Z` cao nhất, so với `CARGO_PKG_VERSION`, rồi trả về một trong ba kết quả: đã mới nhất · có bản mới · chưa có bản phát hành ứng dụng nào. Khi có bản mới, một nút mở trang tag trên GitHub Releases bằng trình duyệt; URL dựng trong Rust từ phiên bản đã kiểm. Mục này nhắc người dùng đối chiếu `SHA256SUMS.txt`. Một công tắc `app_config` tắt hẳn việc kiểm tra: khi đã tắt, Rust từ chối lệnh mà không mở kết nối nào.

## Boundaries & Constraints

**Always:** Request gửi từ Rust bằng `reqwest` blocking đã có, tới host cố định trong mã, kèm `User-Agent` và timeout. CSP và ba quyền của `capabilities/main.json` giữ nguyên. Phiên bản lấy từ `CARGO_PKG_VERSION`, đưa lên qua `BootstrapConfig`. Lỗi trả qua `IpcError` với khoá trong `message_keys!`. Chuỗi hiển thị nằm trong `vi.json`. Mỗi `@click` gọi đúng một `dispatch('<id>')`.

**Never:** Kiểm tra lúc khởi động hoặc theo lịch, và không có công tắc "tự kiểm khi mở" (AD-15). Không tải hay ghi tệp nào từ phản hồi, không chạy tiến trình. Không dùng `tauri-plugin-updater` hay `tauri-plugin-http`. Không mở URL do server trả về. Không thêm crate. Không thêm điểm ra mạng thứ tư. Không đụng 10.8, 10.9.

**Decisions (Ice, 2026-10-21):**
- PRD lệch AD-15: PRD:137 và NFR12 (PRD:965, `spec-auratranslate/requirements.md:428`) ghi "đúng hai luồng". Sửa tại chỗ bằng 🔵 thành ba điểm của AD-15, thêm vế kiểm tra phiên bản (FR111).
- Nguồn dữ liệu: REST `https://api.github.com/repos/vannamhh/AuraTranslate/releases?per_page=100`, JSON qua `serde_json`, bỏ `draft` và `prerelease`. Không dùng Atom. Lỗi 403/429 hiện thành "thử lại sau", vì giới hạn là 60 lượt/giờ mỗi IP.
- Công tắc `update_check_enabled` mặc định **bật**: giá trị rỗng hoặc chưa lưu đọc là bật, chỉ `off` mới tắt.

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected Output / Behavior | Error Handling |
|---|---|---|---|
| Có bản mới | tag `v0.2.0` > `0.1.0` | "Có bản 0.2.0", nút mở `…/releases/tag/v0.2.0`, nhắc checksum | — |
| Đã mới nhất | tag cao nhất = hoặc < phiên bản đang chạy | "Đang dùng bản mới nhất 0.1.0" | — |
| Chỉ có tag dữ liệu (trạng thái thật 2026-10-21: `dict-v2`, `dict-raw-v1`, `dict-v1`) | không tag `v\d+.\d+.\d+` nào | trạng thái riêng "chưa có bản phát hành ứng dụng", KHÔNG báo "mới nhất" | — |
| Draft / prerelease / tag lệch mẫu (`v1.2`, `v1.2.3-rc1`) | trộn với tag hợp lệ | bị bỏ qua khi chọn | — |
| Công tắc tắt | `update_check_enabled = off` | nút kiểm tra không bấm được | lệnh trả lỗi `disabled`, server giả nhận 0 kết nối |
| Mất mạng / timeout | không kết nối được | thông báo lỗi, giữ kết quả cũ | khoá connect/timeout |
| Bị giới hạn lượt (403/429) | GitHub chặn | thông báo "thử lại sau" | khoá riêng |
| Phản hồi hỏng | JSON không đọc được | thông báo lỗi | khoá bad-response |
| Mở trang với phiên bản sai mẫu | `"../x"`, `"1.2"` | không mở gì | lỗi validation |

</frozen-after-approval>

## Code Map

- `src/SettingsUpdateSection.vue` -- khung 12.18, đang chỉ có chữ; gắn ở `SettingsOverlay.vue:664-666`. Giữ `settings.update.lead`/`why_*` (`vi.json:1999-2001`).
- `src/aiUsageVisibleState.ts` + `src/config/bootstrap.ts:98,218,231,279,316` + `core/scope/store.rs:166-168,322-327` + `commands/config.rs:116,162` -- mẫu đầy đủ của một công tắc `app_config` Global (không migration); chép theo.
- `src-tauri/src/core/ai/client.rs`, `commands/aitranslate` `ai_config_test_connection` (`lib.rs:1362`), `tests/connection_check_contract.rs` -- mẫu lệnh mạng async và server HTTP giả cục bộ.
- `commands/library.rs:914,1066-1071` -- mẫu `app.opener()` gọi từ Rust; dùng `open_url` với URL dựng trong Rust.
- `src-tauri/tests/webimport_boundary.rs:197-229` -- `path_is_allowed_for_reqwest` chỉ cho `core/webimport/`, `core/ai/`; thêm `core/update/`.
- `src-tauri/tests/ipc_contract.rs:182,199` (các trường của `BootstrapConfig`) · `ipc_argument_contract.rs` (đọc danh sách lệnh từ `lib.rs`).
- `core/i18n/mod.rs:100,629-631` -- `message_keys!`, có sẵn các biến thể connect/timeout.
- `tests/frontend/settingsFrame.test.ts:119-137` và `e2e/specs/story-12-18-settings.e2e.mjs:51-68` -- đang khẳng định mục Cập nhật có 0 nút/input; sửa cho khớp nội dung mới.
- `scripts/check-deps.mjs:197-199,349` đã cấm updater/http và nêu 10.7 là điểm (2): giữ nguyên.
- Đích: repo `vannamhh/AuraTranslate`. `release.yml:183` tạo tag `v*` với cờ `--latest`. Đo ngày 2026-10-21: `releases/latest` trả 200 cho **`dict-v2`** ⇒ không dùng được, phải đọc cả danh sách và lọc.
- Chuỗi ràng buộc đã ghi đủ ở PRD §9.2 (PRD:1124-1134). `INSTALL.md:3-7` chưa có vế "⇒ cấm tự cập nhật".

## Tasks & Acceptance

**Execution:**
- [x] `src-tauri/src/core/update/mod.rs` (mới) -- hàm thuần: phân tích danh sách, chọn tag ứng dụng cao nhất, so phiên bản, dựng URL tag; hàm fetch nhận base URL để test thay được -- logic test được, không cần mạng thật.
- [x] `src-tauri/src/commands/update.rs` (mới) + `lib.rs` -- `update_check` (async, `spawn_blocking`, đọc công tắc trước khi mở kết nối) và `update_open_release_page(version)` (kiểm mẫu, rồi `open_url`) -- hai bề mặt IPC.
- [x] `core/scope/store.rs`, `commands/config.rs`, `core/i18n/mod.rs` -- khoá `update_check_enabled`, thêm `app_version` vào `BootstrapConfig`, khoá lỗi mới.
- [x] `src/config/update.ts`, `src/updateCheckState.ts`, `src/SettingsUpdateSection.vue`, `src/main.ts`, `src/i18n/vi.json` -- adapter, trạng thái, dòng phiên bản, công tắc, nút, ba kết quả, câu nhắc checksum.
- [x] `src-tauri/tests/update_check_contract.rs` (mới) -- chạy đủ các hàng của ma trận trên server giả, kể cả "tắt ⇒ 0 kết nối". Thêm ca quét mã `core/update/` + `commands/update.rs`, dòng mã thật (không comment), cấm `std::fs`, `std::process`, `open_path`. Sửa `webimport_boundary.rs` và `ipc_contract.rs`.
- [x] `tests/frontend/settingsUpdateSection.test.ts` (mới), `settingsFrame.test.ts`, `e2e/specs/story-12-18-settings.e2e.mjs` -- các trạng thái giao diện; thay các khẳng định "0 nút".
- [x] `INSTALL.md` -- một câu: vì không ký số nên ứng dụng không tự cập nhật, chỉ kiểm tra ở Cài đặt → Cập nhật.
- [x] `prd-auratranslate/prd-auratranslate.md:137,965`, `spec-auratranslate/requirements.md:428` -- sửa tại chỗ bằng 🔵 kèm ngày: ba điểm ra mạng theo AD-15 -- quyết định 1 của Ice.

**Acceptance Criteria:**
- Given toàn bộ mã nguồn, when rà, then không có đường nào ghi tệp, chạy tiến trình hay cài đặt từ phản hồi kiểm tra. Ca quét đỏ khi thêm thật một `std::fs::write` vào `core/update/`.
- Given app vừa mở, when chưa bấm gì, then không có request nào tới host cập nhật.
- Given có bản mới, when thông báo, then dẫn tới GitHub Releases và nhắc đối chiếu SHA-256.
- Given `vi.json`, `.rs`, `.vue`, when chạy `check:i18n` và `check:tokens`, then xanh.

## Implementation Notes

- Đo ngày 2026-10-21: `GET /releases/latest` của GitHub trả `dict-v2`, vì repo chưa có bản phát hành `v*` nào. Lệnh vì vậy đọc cả danh sách và lọc tag `vX.Y.Z`; bấm kiểm tra hôm nay phải ra "chưa có bản phát hành ứng dụng".
- `src/main.ts` không đổi: không có gì cần đăng ký lúc khởi động. Ba trình xử lý `@submit` vào `HANDLER_TABLE` của `check-commands.mjs` với `R_SUBMIT_DIRECT`, theo khuôn công tắc `ai_usage_visible`.
- `update_check` là `async fn` literal chạy `spawn_blocking`, nên bảng điều tra lệnh của `config_invariants.rs` ghi 2 plain / 0 `(async)`.
- `SRC_TAURI_RS_FLOOR` của `dict_boundary.rs` lên 208 = ceil(0,85 × 244 tệp), vì ba tệp `.rs` mới làm sàn tụt dưới 80%.
- `config_invariants.rs` ở `043cd07` đã đỏ `rustfmt --check` (từ ca INSTALL.md của 10-6), và `cargo fmt` đã sửa luôn.
- Đối chứng bằng phép gỡ thật: một `std::fs::write` thật gài vào `core/update/mod.rs` ⇒ ca quét đỏ; gỡ `.take` và khối kiểm trần ⇒ ca body quá trần ra `Ok(NoAppRelease)`; một `runUpdateCheck()` thật thêm vào `main.ts` ⇒ ca quét `src/**` đỏ với `main.ts`.
- Chưa chạy e2e 12.18 (cần app thật, ngoài sandbox) và chưa thử tay trên app thật. Hai việc này vào phần xem lại thật của Epic 10.

## Review Triage Log

Lượt 1 (quick): low 9 · medium 1 · false 7 · maybe-false 0. Patch 5, reject 12, loopback 0.

- `updateCheckState.ts` `resetUpdateCheck` không ai gọi — low · patch: xoá.
- `setUpdateCheckEnabled` đổi công tắc trước khi ghi, không lùi lại khi ghi lỗi — low · reject: y hệt khuôn `aiUsageVisibleState.ts:15-18`; lỗi hiện qua `updateSaveError`, và nếu Rust vẫn đọc giá trị cũ thì nó trả `update.disabled` thành lỗi hiện ra, không âm thầm.
- Dòng phiên bản rỗng khi bootstrap lỗi — low · patch: chuỗi riêng `settings.update.version_unknown`.
- AC "chưa bấm thì không request" chỉ được chứng minh khi mount riêng component — medium · patch: thêm ca quét nguồn `src/**`, đối chứng bằng một lời gọi thật trong `main.ts`.
- Ca quét Rust chỉ thử trên chuỗi gài — false: phase Rust đã gài thật `std::fs::write` vào `core/update/mod.rs` và ca đỏ. Thiếu `tokio::process`/`OpenOptions` — low · patch: thêm token. URL từ server tới `open_url` — false: lớp `wire` chỉ trả chuỗi phiên bản, và ca `the_release_page_is_built…` khẳng định đúng URL đã mở.
- Trần 4 MB áp sau khi đã đọc hết body — low · patch: đọc tối đa trần + 1 byte.
- 301/302 bị gộp vào bad_response — low · reject: cố ý `Policy::none()` để giữ host cố định, hiếm gặp, sửa thì phải thêm nhánh.
- Mọi 403 coi là bị giới hạn lượt — false: đúng quyết định của Ice (403/429 ⇒ "thử lại sau").
- `Version::parse(current)` lỗi thì báo bad_response — false: không chạy tới được; `check-release-version.mjs` ép `X.Y.Z`.
- Hai khối rustfmt trong `config_invariants.rs` không liên quan — false: bản ở baseline đỏ `rustfmt --check`, và `check:fmt` là một cổng của pre-push.
- `SRC_TAURI_RS_FLOOR` 195→208 không có số đo — false: `assert_population_floor` đòi ceil(0,85 × 244) = 208. Cột async của bảng điều tra lệnh đếm thuộc tính `(async)`, giống hàng `ai_proofread_segment`.
- Comment đầu module nhắc AD/FR — low · reject: chúng nêu bất biến không hiển nhiên (chỉ chạy khi bấm, không ghi tệp/không chạy tiến trình) và không mang id story.
- Nút công tắc không gắn `aria-labelledby` — low · reject: y hệt khuôn `SettingsOverlay.vue:553-555`.
- Vitest thiếu nhánh open_failed và hình dạng lạ — low · reject: cùng đường render `tError` như ca rate_limited; mọi khoá đã có ca Rust.
- Sổ sách plan / chưa chạy toàn bộ suite — false: step 5 lo phần này; toàn bộ suite chạy sau khi vá.

## Design Notes

So sánh phiên bản bằng bộ ba `(u64,u64,u64)` tách từ `^v(\d+)\.(\d+)\.(\d+)$`, không cần crate `semver`. URL mở luôn có dạng `https://github.com/vannamhh/AuraTranslate/releases/tag/v{a}.{b}.{c}`, dựng từ bộ ba đã parse, không lấy từ trường `html_url`/`link` mà server trả.

## Verification

**Commands:**
- `npm run build && cargo test --manifest-path src-tauri/Cargo.toml --test update_check_contract --test webimport_boundary --test ipc_contract --test ipc_argument_contract` -- expected: xanh.
- `npx vitest run tests/frontend/settingsUpdateSection.test.ts tests/frontend/settingsFrame.test.ts` -- expected: xanh.
- Chạy toàn bộ suite một lần vì thay đổi chạm `lib.rs` (đăng ký lệnh) -- expected: xanh.

**Manual checks (if no CLI):**
- Mở app thật: Cài đặt → Cập nhật hiện `0.1.0`. Bấm kiểm tra với dữ liệu thật hôm nay phải ra "chưa có bản phát hành ứng dụng". Tắt công tắc thì nút không bấm được.
