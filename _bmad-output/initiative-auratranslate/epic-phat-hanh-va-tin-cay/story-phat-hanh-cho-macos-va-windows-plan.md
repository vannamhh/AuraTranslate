---
title: 'Story 10.2 — Phát hành cho macOS và Windows'
type: 'feature'
ticket: "2"
created: '2026-10-12'
status: 'ready-for-dev'
baseline_revision: ''
route: 'full'
route_source: 'auto'
risk: 'medium'
review: ''
review_source: ''
lenses_ran: []
review_loop_iteration: 0
context:
  - '{project-root}/AGENTS.md'
  - '{project-root}/scripts/AGENTS.md'
  - '{project-root}/src-tauri/AGENTS.md'
---

<frozen-after-approval reason="human-owned intent — do not modify unless human renegotiates">

## Intent

**Problem:** `release.yml` (Story 10.1) dựng `.dmg`/`.msi` chỉ làm artifact của lượt chạy, hết hạn sau 30 ngày; chưa có GitHub Release, chưa có ghi chú phát hành nói rõ không ký số, không gì buộc tag khớp phiên bản ứng dụng, và không gì kiểm hai bản cài cùng mang bộ font đóng gói.

**Approach:** Thêm job `publish` vào `release.yml`, chỉ chạy khi push tag `v*`, sau khi cả hai job dựng xanh: tạo GitHub Release bằng `gh` có sẵn trên runner (không thêm action bên thứ ba), đính `.dmg` + `.msi`, thân là tệp ghi chú phát hành trong repo. Mở rộng `inspect-release.mjs` kiểm bốn tệp font như đang kiểm bốn `.db`; thêm bước đối chiếu tag ↔ phiên bản.

## Decisions (Ice, 2026-10-12)

1. **macOS: một `.dmg` universal** (`--target universal-apple-darwin`). Số đo khi chọn: binary release x86_64 16.334.844 byte (máy Ice, Homebrew rustc, `cargo build --release`, không qua `tauri build`); arm64 chưa đo; dữ liệu 400.541.421 byte không nhân lên.
2. **Công bố ngay thành Latest** khi mọi kiểm xanh; không nháp.
3. **Giữ plan nguyên cỡ** (3.101 token tiktoken) với đủ guard.

## Boundaries & Constraints

**Always:** Định dạng phát hành là `.dmg` và `.msi` (sơ đồ triển khai của spine); WebView2 giữ `offlineInstaller`. `contents: write` chỉ ở job `publish`, các job khác giữ `contents: read`. Release ứng dụng là Latest; `dict-v2`, `dict-raw-v1` không bao giờ thành Latest. Release chỉ tạo khi lượt `ci.yml` trên đúng commit đó đã `success` (bằng chứng hành vi tương đương: `check` chạy đủ cổng, vitest, `cargo test` trên `macos-26` và `windows-2025`). Ghi chú phát hành nói thẳng: không ký số, không notarize, và lý do.

**Never:** Checksum (10.3), giấy phép trong bản phát hành (10.5), hướng dẫn cài có ảnh (10.6), cơ chế cập nhật (10.7), số NFR6 trong ghi chú (10.9). Thêm action bên thứ ba ngoài `actions/*`. Ký số. Đổi `[profile.release]`. Tạo release khi chạy bằng `workflow_dispatch` (lượt đó chỉ dựng và kiểm). Push từ trong story.

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected Output / Behavior | Error Handling |
|----------|--------------|---------------------------|----------------|
| Tag hợp lệ | push `v0.1.0`, ba tệp phiên bản đều `0.1.0`, CI commit xanh | Release `v0.1.0` Latest, đủ bản cài đã chốt, thân = ghi chú phát hành | — |
| Tag lệch phiên bản | push `v0.2.0`, `tauri.conf.json` còn `0.1.0` | đỏ trước `tauri build`, không release | log tên tệp + phiên bản thấy + phiên bản mong đợi |
| CI commit chưa xanh | `ci.yml` trên SHA đó đỏ / đang chạy / không có | `publish` đỏ, không release | log trạng thái run tìm thấy |
| Một OS dựng đỏ | job Windows đỏ | `publish` không chạy | — |
| Tag đã có release | chạy lại trên tag cũ | đỏ, release cũ giữ nguyên | không `--clobber` |
| `workflow_dispatch` | chạy tay trên `master` | hai job dựng + kiểm, `publish` bị bỏ qua | — |
| Thiếu font trong bản cài | một tệp font vắng ở `.app` hoặc bản giải nén `.msi` | bước kiểm đỏ | tên tệp thiếu |

</frozen-after-approval>

## Code Map

- `.github/workflows/release.yml` -- 105 dòng. Header `:1-8` nói "không phát hành gì" (phải sửa); triggers `:12-14`; `concurrency` `:16-18`; `permissions: contents: read` `:20-21`; job `build` ma trận `[macos-26, windows-2025]` `:24-31`; fetch `:54`; `tauri build` `:57,:62`; kiểm `.app`/`.dmg` `:67`, kiểm `.msi` qua `msiexec /a` `:82`; `upload-artifact` tên `aura-translate-${{ runner.os }}` `:98`. Mẫu ghim: `uses: owner/action@<SHA 40 ký tự> # vX.Y.Z`; `download-artifact` cần SHA lấy nguyên văn từ `gh api`.
- `scripts/inspect-release.mjs` -- 61 dòng, `parseArgs` `:18`; đang kiểm bốn `.db` theo manifest + ghi cỡ ra step summary. Thêm kiểm bốn tệp font (`resources/fonts/*.{otf,ttf}` theo `tauri.conf.json:35-40`) có mặt dưới `fonts/` và sha khớp tệp nguồn.
- `scripts/ci-previous-verdict.mjs` -- đọc `gh run list --workflow ci.yml`; tham khảo cách gọi `gh` cho bước "CI commit đã xanh", không sửa.
- Phiên bản: `src-tauri/tauri.conf.json:4`, `src-tauri/Cargo.toml:3`, `package.json:3` đều `0.1.0`; chưa gì so với tag.
- `src-tauri/tests/config_invariants.rs` -- quét chuỗi, không parser YAML. `workflow_files()` `:567`, `code_lines()` `:579`; `only_release_workflow_listens_to_tags_and_only_to_v_tags` `:686`; `release_workflow_passes_the_release_overlay_to_every_tauri_build` `:716` đòi đúng 2 dòng `npx tauri build` (job `publish` không dựng nên không đụng); `every_workflow_config_overlay_is_an_allowed_one` `:742`.
- `scripts/check-deps.mjs` -- `BANNED_CRATES` `:188-198`, `BANNED_NPM` `:259-267`; `tauri-plugin-updater` / `@tauri-apps/plugin-updater` chưa bị cấm (tự tải + tự cài, trái "không cơ chế tải thêm sau khi cài" và FR111).
- `src-tauri/src/core/dict/` -- không dùng `reqwest`; `reqwest` chỉ ở `core/webimport`, `core/ai`, `core/aiconfig`, `core/i18n`, `commands/project/wire.rs`, `ports/translation_provider.rs` (AD-15: ba lối ra). Mẫu test ranh giới: `src-tauri/tests/matching_boundary.rs:80`.
- `src/tokens/fonts.ts` -- bốn `FontFace` qua `resolveResource` + `convertFileSrc`; dãy dự phòng hệ thống trong `reset.css:56` là dự phòng khi nạp lỗi, không đổi.
- `deferred-work.md` -- hai mục `Chủ: Story 10.2`: định dạng Windows `:16-17` (đóng: `.msi` theo spine); phần dư NFR6 `downloadBootstrapper`/NSIS `:29-31` (chuyển `Chủ: Ice` cạnh mục NFR6 `:833`, vì lối thoát đó chỉ có nghĩa sau phán quyết NFR6). Mục font chưa nhìn thấy trên Windows `:22-23` giữ `Chủ: Epic 10`; mục universal `:15` thêm dòng 🟡: bản phát hành nay universal, cỡ thật ở step summary lượt `release.yml` đầu, Windows ARM64 vẫn chưa đo, giữ `Chủ: B7`.

## Tasks & Acceptance

**Execution:**
- [ ] `.github/release-notes.md` -- tạo, tiếng Việt: không ký số, không notarize, lý do (PRD §9.1); Gatekeeper/SmartScreen sẽ cảnh báo, hướng dẫn ở 10.6; macOS universal (Apple Silicon và Intel); Windows nhúng WebView2 cài được không mạng; từ điển nhúng sẵn, không tải thêm.
- [ ] `scripts/check-release-version.mjs` + vitest -- nhận tag, đọc ba tệp phiên bản; lệch ⇒ exit ≠ 0 với tên tệp và hai giá trị; tag không dạng `v<semver>` ⇒ exit ≠ 0.
- [ ] `scripts/inspect-release.mjs` + vitest -- kiểm bốn tệp font như trên.
- [ ] `.github/workflows/release.yml` -- sửa header; bước kiểm phiên bản trước `tauri build` (chỉ khi tag); job `publish` (`ubuntu-24.04`, `needs: build`, `if` tag `v*`, `permissions: contents: write`): xác nhận `ci.yml` trên `github.sha` `success` → `download-artifact` → `gh release create "$GITHUB_REF_NAME" --verify-tag --latest --notes-file .github/release-notes.md <tệp>` không `--draft`. macOS: `dtolnay/rust-toolchain` thêm `targets: aarch64-apple-darwin,x86_64-apple-darwin`; `tauri build --target universal-apple-darwin`; đường dẫn bundle macOS ở bước kiểm và `upload-artifact` đổi sang `src-tauri/target/universal-apple-darwin/release/bundle/`; bước kiểm macOS chạy `lipo -archs` trên binary trong `.app`, thiếu `x86_64` hoặc `arm64` ⇒ đỏ.
- [ ] `src-tauri/tests/config_invariants.rs` -- ca mới: chỉ job `publish` có `contents: write`, mọi job khác và cấp workflow `read`; `publish` `needs: build` và bị chặn trên tag `v*`; lệnh tạo release truyền `--notes-file .github/release-notes.md`, `--verify-tag`, `--latest`, không `--clobber`; tệp ghi chú chứa câu không ký số / không notarize; bước kiểm phiên bản đứng trước mọi `npx tauri build`; dòng `tauri build` macOS mang `--target universal-apple-darwin`.
- [ ] `scripts/check-deps.mjs` -- cấm `tauri-plugin-updater` và `@tauri-apps/plugin-updater`.
- [ ] `src-tauri/tests/dict_boundary.rs` -- mã `core/dict/` không nhắc `reqwest`/`http`/`ureq`, theo mẫu `matching_boundary.rs`; dòng code, không dòng comment.
- [ ] `deferred-work.md` -- hai mục `Chủ: Story 10.2` đóng / chuyển chủ bằng chữ như Code Map.

**Acceptance Criteria:**
- Given push tag `v*` trên commit CI xanh, when `release.yml` xong, then GitHub Releases có release tag đó là Latest, đính `.dmg` universal và `.msi`, thân là ghi chú phát hành; `dict-v2` không Latest.
- Given hai bản cài, when bước kiểm chạy trên runner, then cả hai có đủ bốn `.db` và bốn tệp font sha khớp nguồn, và binary macOS mang cả `x86_64` lẫn `arm64`.
- Given gỡ `--notes-file`, gỡ `permissions` của `publish`, hoặc đưa bước kiểm phiên bản ra sau `tauri build`, when `cargo test --test config_invariants`, then đỏ đúng ca canh nó.
- Given thêm `tauri-plugin-updater` vào `Cargo.toml`, when `npm run check:deps`, then đỏ.

## Implementation Notes

## Plan Change Log

## Review Triage Log

## Design Notes

`gh release create` thay cho action bên thứ ba: `gh` có sẵn trên runner GitHub, không thêm hàng Stack/NFR15, quyền đi qua `GH_TOKEN: ${{ github.token }}`. `--verify-tag` cấm `gh` tự tạo tag khi tag chưa có trên remote. Release Latest được đặt tường minh vì GitHub chọn Latest theo ngày và semver: tag `v0.1.0` mới hơn `dict-v2` nên mặc định cũng thường đúng, nhưng không để mặc định quyết.

## Verification

**Commands:**
- `npx vitest run scripts/` (các tệp test của hai script) -- xanh.
- `cargo test --test config_invariants` · `cargo test --test dict_boundary` -- xanh; đối chứng đỏ bằng phép gỡ thật cho từng ca mới.
- `npm run check:deps` -- xanh; thêm tạm `tauri-plugin-updater` ⇒ đỏ.
- `gh workflow run release.yml --ref master` -- hai job dựng xanh, `publish` bị bỏ qua (cần push, do người làm).

**Manual checks:**
- Ice đẩy tag `v0.1.0` sau khi story vào `master` và CI commit đó xanh: release xuất hiện, tải được cả hai bản cài, mở `.dmg` trên máy Intel của mình, đọc ghi chú phát hành. Lượt này cũng đóng AC1 của 10.1.
