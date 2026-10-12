---
title: 'Story 10.1 — Build công khai qua GitHub Actions'
type: 'feature'
ticket: "1"
created: '2026-10-12'
status: 'in-review'
baseline_revision: '2fe700975c485949aaac5170b1f23ef9daf777f8'
route: 'full'
route_source: 'auto'
risk: 'low'
review: 'quick'
review_source: 'pinned'
lenses_ran: ['quick']
review_loop_iteration: 0
context:
  - '{project-root}/AGENTS.md'
  - '{project-root}/scripts/AGENTS.md'
  - '{project-root}/src-tauri/AGENTS.md'
---

<frozen-after-approval reason="human-owned intent — do not modify unless human renegotiates">

## Intent

**Problem:** Chưa có đường công khai từ một tag tới `.dmg`/`.msi` chứa đủ từ điển: không script nào tải theo `dict-manifest.toml` và đối chiếu SHA-256, cấu hình đóng gói không có `dict`, manifest còn trỏ `dict-v1` (sai sha). Kèm theo là tám mục nợ làm cứng CI và chuỗi cung ứng mà Story 1.3 chuyển sang.

**Approach:** Thêm `release.yml` chạy trên tag `v*`: tải bốn tệp `.db` theo manifest, đối chiếu sha rồi mới `tauri build`, kiểm bốn tệp có mặt trong bản cài, đo NFR6, đẩy `.dmg`/`.msi` làm artifact. Đóng hoặc chuyển chủ từng mục nợ `Chủ: Story 10.1` theo các quyết định dưới đây.

## Decisions (Ice giao Claude quyết, 2026-10-12)

1. **Nối từ điển bằng lớp phủ `src-tauri/tauri.release.conf.json`**, chỉ `release.yml` truyền; dev, `ci.yml`, e2e không đổi. Loại map cơ sở (mọi build phải có 373 MB, job `check` tải mỗi lượt) và map thư mục (đóng gói `README.md`/`.gitkeep`, cấu hình không còn nói gì về bốn tệp).
2. **NFR6 đo, không làm đỏ.** Byte tệp thật đã 400.541.421 > 400.000.000 trước khi cộng binary; định nghĩa và phán quyết là tầng PRD. `release.yml` ghi vào step summary cả byte cài đặt (không tính WebView2) lẫn cỡ tệp cài; nợ mới `Chủ: Ice`.
3. **Cổng `cargo clippy --all-targets -D warnings` cho `src-tauri` và `tools/dict-build`.** Đo: 159 + 3 cảnh báo. Sửa phần cơ học; 21 `too_many_arguments` và 7 `type_complexity` mang `#[expect(lint, reason = "…")]` tại chỗ (tự đỏ khi lint thôi bắn), không allow cấp crate.
4. **`cargo fmt` cả hai crate trong một commit `style` riêng**, thêm `.git-blame-ignore-revs`, rồi cổng `--check`.
5. **CVE: Dependabot alerts** (Ice bật trong cài đặt repo; hôm nay API trả 404). Giữ câu cấm `cargo-deny`/`cargo-audit` ở `ci.yml:139-141`; không thêm `dependabot.yml` vì mọi crate ghim `=` sẽ sinh PR nhiễu.
6. **MSRV:** `rust-version` khai đúng toolchain CI ghim (`1.97.1`) ở cả hai crate.
7. **Triggers:** `push` bỏ tag (`branches: ['**']`), giữ `pull_request`; concurrency gộp nhánh và PR của cùng nhánh.
8. **Type-check toàn bộ `scripts/**/*.mjs`** qua `tsconfig.node.json`, sửa 26 lỗi đo được.

## Boundaries & Constraints

**Always:** Sai sha thì dừng trước `tauri build` và không để tệp hỏng ở `src-tauri/resources/dict/` (AD-25). Action ghim theo SHA commit lấy nguyên văn từ `gh api`. Từ điển đến runtime ở `$RESOURCE/dict/<tệp>.db`. Cổng mới vào đủ ba danh sách. Mỗi mục nợ chạm tới được đóng, nửa đóng hoặc chuyển chủ bằng chữ.

**Never:** Tạo GitHub Release cho ứng dụng (10.2), công bố checksum (10.3). Commit `.db`. Thêm `dict` vào `assetProtocol.scope`. Sửa spine/PRD. Chạy release trên tag `dict-*`. Allow lint cấp crate.

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected Output / Behavior | Error Handling |
|----------|--------------|---------------------------|----------------|
| Tag hợp lệ | push `v0.1.0` | hai job xanh, artifact `.dmg` + `.msi`, bước kiểm thấy đủ bốn `.db` | — |
| Sha lệch | một `sha256` trong manifest bị sửa | job đỏ ở bước tải, `tauri build` không chạy | tên tệp + sha mong đợi + sha thật trong log |
| Tải hỏng | 404 / mạng đứt | job đỏ ở bước tải | không để tệp dở dang |
| Tag từ điển | push `dict-v3` | không workflow nào chạy | — |

</frozen-after-approval>

## Code Map

- `.github/workflows/ci.yml` -- triggers `:46-47`; concurrency `:74-77`; câu cấm `:139-141` (giữ); `uses:` `:112,114,120,125,776,778,783,785,793`; header khẳng định phiên bản `:15-16`; `.dmg` `:366-372`; ba lượt `.msi` `:440-584` với `--config` nofonts `:468` và bootstrapper tạm `:476-478`. Bước gate `:143-278`.
- `.githooks/pre-push:53` · `package.json:14-29` · `scripts/check-gates.mjs` -- ba danh sách cổng và cổng canh chúng.
- `dict-manifest.toml` -- `[base]` + ba `[[detachable]]`; bốn URL `dict-v1` → `dict-v2`; comment nhắc `dict-v1`.
- `scripts/check-dict-manifest.mjs` -- `parseStrictToml` `:77`: tách ra `scripts/lib/` để script tải dùng chung.
- `scripts/check-dict-build.mjs` -- Check B `:178-222` chỉ canh dict-build ↛ src-tauri; thêm chiều ngược.
- `scripts/check-deps.mjs` -- `BANNED_CRATES` `:169-180`, `BANNED_NPM` `:225-233`, `walk` `:130-138`. Plugin đang dùng: `tauri-plugin-dialog`, `tauri-plugin-fs` (bắc cầu), `tauri-plugin-wdio-webdriver` (tuỳ chọn); npm `@tauri-apps/api`, `@tauri-apps/cli` (+ gói nhị phân theo nền tảng).
- `scripts/check-scope-bundled.mjs:7,20` -- hai trích dẫn dòng đã rữa.
- `src-tauri/tauri.conf.json:35-39` -- `bundle.resources` là map; lớp phủ dùng dạng map `"resources/dict/*.db": "dict/"` (dạng mảng sẽ đặt sai chỗ). Glob rỗng làm `tauri-build` lỗi `GlobPathNotFound`.
- `src-tauri/tests/config_invariants.rs` -- `read_json` `:19`; `no_dev_csp_and_no_platform_config_overrides` `:209` (chỉ `.json`; mẫu `json/json5/toml` ở `:386-407`); doc `:313-315`; `nofonts_overlay…` `:488`.
- `src-tauri/src/lib.rs:1476-1477` -- doc comment "chưa mang dict (Story 10.1)".
- `src-tauri/Cargo.toml:7`, `tools/dict-build/Cargo.toml` -- `rust-version`.
- `tsconfig.node.json:14-20` -- chỉ 5 script.
- `deferred-work.md` -- tám mục `:84-113`; bundle `:341-342`; NFR6 `:823`; điều kiện khởi hành `:1696-1705` (đóng được: repo PUBLIC, Actions chạy, `.msi` dựng trên `windows-2025`).
- SHA: `actions/checkout` v7.0.1 `3d3c42e5aac5ba805825da76410c181273ba90b1`; `actions/setup-node` v7.0.0 `820762786026740c76f36085b0efc47a31fe5020`; `Swatinem/rust-cache` v2.9.1 là tag có chú thích (deref lấy commit); `dtolnay/rust-toolchain@1.97.1` là nhánh; `actions/upload-artifact` mới nhất v7.0.2.

## Tasks & Acceptance

**Execution:**
- [ ] Phase 1 (`style`) -- `cargo fmt` hai crate; `.git-blame-ignore-revs` ghi SHA commit đó.
- [ ] Phase 2 (`refactor`) -- sửa clippy hai crate (`cargo clippy --fix` trước, tay sau); `#[expect]` có `reason` cho lint thiết kế; `rust-version = "1.97.1"`.
- [ ] `dict-manifest.toml` -- URL sang `dict-v2`; sửa comment sai.
- [ ] `scripts/fetch-dicts.mjs` + `scripts/lib/` -- tải vào tệp tạm, đối chiếu sha-256, chỉ đổi tên vào đích khi khớp; `--manifest`, `--dest`; vitest cho khớp / lệch / lỗi tải.
- [ ] `src-tauri/tauri.release.conf.json` -- chỉ thêm khoá `dict` vào `bundle.resources`.
- [ ] `config_invariants.rs` -- lớp phủ release chứa đúng bốn tên lấy từ manifest đi tới `dict/` và không đụng khoá nào khác; `release.yml` truyền nó; mọi `--config` trong `.github/workflows/*.yml` thuộc danh sách lớp phủ cho phép; không đâu đặt `TAURI_CONFIG`; cấm biến thể `.json5`/`.toml`; sửa doc `:313-315`.
- [ ] `.github/workflows/release.yml` -- `tags: ['v*']` + `workflow_dispatch`; `macos-26`/`windows-2025`; `permissions: contents: read`; fetch → `tauri build --bundles dmg|msi --config src-tauri/tauri.release.conf.json` → liệt kê bốn `.db` trong `.app` / bản giải nén `msiexec /a` (thiếu ⇒ đỏ) → step summary NFR6 → `upload-artifact`.
- [ ] `scripts/check-dict-build.mjs` -- chiều ngược Check B.
- [ ] `scripts/check-deps.mjs` -- danh sách CHO PHÉP cho `tauri-plugin-*` / `@tauri-apps/*`; tập đã thăm trong `walk`.
- [ ] `ci.yml` -- ghim SHA, sửa header; triggers + concurrency; cổng clippy và fmt (ba danh sách).
- [ ] `tsconfig.node.json` + `scripts/*.mjs` -- include toàn bộ, sửa lỗi kiểu (sửa kiểu, không `@ts-expect-error`).
- [ ] `scripts/check-scope-bundled.mjs`, `src-tauri/src/lib.rs` -- xoá trích dẫn dòng, sửa doc comment.
- [ ] `deferred-work.md` -- đóng / nửa đóng / chuyển chủ từng mục `Chủ: Story 10.1`; nợ mới NFR6 `Chủ: Ice`; mục Dependabot chờ Ice bật.

**Acceptance Criteria:**
- Given `release.yml` chạy bằng `workflow_dispatch`, when xong, then hai job xanh, log công khai, `.dmg` và `.msi` chứa đủ bốn `.db` có sha khớp manifest.
- Given manifest sửa một sha, when chạy `fetch-dicts` (local và trên runner), then thoát khác 0 trước `tauri build`.
- Given gỡ khoá `dict` khỏi lớp phủ release, hoặc gỡ `--config` khỏi `release.yml`, when chạy `config_invariants`, then đỏ đúng ca canh nó.
- Given CI trên commit cuối, when xong, then clippy, fmt, type-check script và mọi cổng cũ xanh trên cả hai OS.

## Implementation Notes

- Bốn pha, mỗi pha một commit trở lên: `style` (fmt), `refactor(rust)` (clippy, MSRV), `feat(release)` (đường phát hành), `refactor(scripts)` + `ci` (làm cứng CI). Ghi chú bàn giao giữa các pha nằm ngoài repo.
- ⚠️ Fmt cần hai lượt mới hội tụ. Nó làm đỏ chín chỗ quét mã nguồn theo dòng: comment miễn trừ máy đọc chỉ che dòng ngay dưới, nên phải dời vào trong biểu thức; needle chữ ký và `app.manage(...)` đổi theo văn bản đã định dạng. Hai bộ cắt thân hàm trên `\n    pub fn ` vẫn XANH nhưng yếu đi, vì không tìm thấy dấu cắt nên thân hàm chạy tới hết tệp.
- ⚠️ `#[expect]` đặt giữa `#[tauri::command]` và `fn` làm đỏ các test đọc dòng ngay trước chữ ký; phải đặt trên `#[tauri::command]`. Sáu `#[allow(clippy::…)]` có từ trước story cũng đổi sang `#[expect(…, reason)]`.
- `SegmentRole::from_str` đổi thành `parse`; `set_block_override` trả `Result<(), BlockIndexOutOfRange>` thay cho `Result<(), ()>`.
- Type-check toàn bộ `scripts/**/*.mjs` ở chế độ strict của `tsconfig.node.json` ra 328 lỗi, không phải 26 như lượt đo không strict. Đã sửa hết bằng JSDoc, không dùng `@ts-expect-error`.
- Bộ duyệt cây `walk` chặn vòng bằng chuỗi tổ tiên, không dùng tập đã-thăm toàn cục: tập toàn cục làm số gói npm rơi 522 → 305, dưới sàn 444.
- Concurrency của CI đổi thành `ci-${{ github.head_ref || github.ref_name }}`. Lượt `schedule` trên `master` vẫn chung nhóm với push `master`, y như với `github.ref` trước đây.
- Bản `.app` debug dựng với lớp phủ release nặng 498.562.207 byte (binary debug, không phải số NFR6). Số thật sẽ có ở lượt `release.yml` đầu tiên.
- Còn do người làm: chạy `release.yml` lần đầu (nửa Windows chưa từng chạy), và Ice bật Dependabot alerts.

## Plan Change Log

## Review Triage Log

Lượt 1 (quick): high 0 · medium 4 · low 3 · false 0 · maybe-false 0; hai low bị bác.
- medium · patch — `dict-fetch.mjs` giữ tệp sai sha dưới tên thật khi lần tải thay thế thất bại (chỉ `.part` bị xoá); xoá đích trước khi tải, thêm ca vitest.
- medium · patch — `.part` không bị gitignore (`git check-ignore` exit 1); thêm mẫu.
- medium · patch (sổ nợ) — mục `bundle.resources` và điều kiện khởi hành đóng ✅ bằng suy luận về `release.yml` chưa chạy; mục đầu hạ 🟡 `Chủ: Story 10.1`, mục sau chỉ dẫn run `ci.yml` 37446210896.
- medium · chờ người — AC1/AC4 cần push và một lượt `release.yml`/CI thật; step 3 cấm push, chuyển sang bước trình bày.
- low · patch — comment mang mã story/banner/lịch sử review ở `release.yml:3`, `config_invariants.rs:550`, `strict-toml.mjs:35,49,110`.
- medium · patch — nhóm concurrency `ci-${{ github.head_ref || github.ref_name }}` gộp PR từ fork tên `master` với `master` thật và huỷ lượt của nó; thêm repo đầu nhánh vào khoá.
- low · patch — `tsconfig.node.json` đổi định dạng `"lib"` ngoài phạm vi; trả về một dòng.
- low · bác — cổng `check:clippy`/`check:fmt` không tự kiểm: phán quyết là mã thoát của cargo, cùng tiền lệ `check:lint`; đối chứng bằng tay chạy cho cả hai.
- low · bác — chiều ngược Check B bắt mọi chuỗi `dict-build` trong `Cargo.toml`: chỉ khắt khe hơn, vô hại; đối chứng `[patch]` đã chạy ở Phase 3a.

## Verification

**Commands:**
- `node scripts/fetch-dicts.mjs` rồi `shasum -a 256 src-tauri/resources/dict/*.db` -- khớp manifest; với `--manifest <bản sửa sha>` -- thoát khác 0.
- `cargo test --test config_invariants` · `npm run check:deps` · `npm run check:dict` · `npm run check:dict-manifest` · `npm run check:gates` -- xanh.
- Full suite một lần sau Phase 2 (`Cargo.toml` đổi).
- `gh workflow run release.yml --ref master` rồi `gh run view` -- hai job xanh, artifact có `.dmg`/`.msi`.

**Manual checks:**
- Ice tải artifact `.dmg`, thấy bốn tệp ở `.app/Contents/Resources/dict/`.
