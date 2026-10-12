---
title: 'Checksum SHA-256'
type: 'feature'
ticket: '3'
created: '2026-10-12'
status: 'built'
baseline_revision: 'e7e88bb2e882c9b07c9192497a64c68d66dcfc79'
route: 'full'
route_source: 'auto'
risk: 'low'
review: 'quick'
review_source: 'pinned'
lenses_ran: ['quick']
review_loop_iteration: 0
context: []
---

<frozen-after-approval reason="human-owned intent — do not modify unless human renegotiates">

## Intent

**Problem:** Job `publish` của `release.yml` (10.2) đính `.dmg` + `.msi` vào GitHub Release mà không công bố checksum nào. Người dùng không có cách đối chiếu tệp tải về với thứ dự án công bố, trong khi bản cài không ký số (FR106).

**Approach:** Trong job `publish`, một script node tính SHA-256 của đúng các tệp sắp đính. Script ghi `SHA256SUMS.txt` (định dạng `shasum -c` đọc được) và dựng thân release: `.github/release-notes.md` cộng một mục sinh ra gồm tên tệp, hash và lệnh đối chiếu cụ thể cho macOS và Windows. `gh release create` dùng thân đó và đính thêm `SHA256SUMS.txt`.

## Decisions (Ice, 2026-10-12)

1. Giữ plan nguyên cỡ (2.167 token tiktoken o200k).

## Boundaries & Constraints

**Always:** Hash sinh trong CI, trên chính tệp được đính vào release, không có giá trị nào chép tay. Hash nằm ngay trong thân release, không chỉ trong một asset. Mọi bất thường làm `publish` đỏ trước `gh release create`: thiếu tệp, không có tệp nào, hay mục checksum thiếu một tệp. Giữ nguyên mọi bất biến của 10.2: `--verify-tag`, `--latest`, không `--clobber`/`--draft`, quyền `contents: write` + `actions: read` chỉ ở `publish`, kiểm CI xanh đứng trước tạo release.

**Never:** Ký số hay GPG. Thêm action ngoài `actions/*`. Hướng dẫn cài có ảnh (10.6, 10.6 sẽ tái dùng lệnh đối chiếu của story này). Đổi job `build`. Push từ trong story.

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected Output / Behavior | Error Handling |
|----------|--------------|---------------------------|----------------|
| Hai bản cài | `x.dmg`, `y.msi` | `SHA256SUMS.txt` hai dòng `<hex64>  <tên>`; thân = ghi chú + mục checksum có cả hai tên, hai hash, lệnh `shasum -a 256` và `Get-FileHash -Algorithm SHA256` | — |
| Tệp không tồn tại | đường dẫn sai | exit ≠ 0, không ghi tệp đầu ra | log đường dẫn |
| Không có tệp | danh sách rỗng | exit ≠ 0 | log usage |
| Tên trùng | hai đường dẫn cùng basename | exit ≠ 0 | log tên trùng |
| Thiếu tệp ghi chú | `--notes` sai | exit ≠ 0 | log đường dẫn |

</frozen-after-approval>

## Code Map

- `.github/workflows/release.yml:133-177`: job `publish`. Hiện chỉ có `checkout`, chưa có node (thêm `actions/setup-node`, cùng SHA `820762…` / v7.0.0 như job `build`, node 22, không cache npm). Bước `create the release` `:167-177` kiểm đúng một `.dmg` + một `.msi` rồi gọi `gh release create … --notes-file .github/release-notes.md`. Header `:1-9` cần một vế về checksum.
- `scripts/lib/dict-fetch.mjs:40`: `sha256OfFile` (stream, hex thường) đang private. Export để dùng lại, không viết hàm hash thứ hai.
- `scripts/check-release-version.mjs`: mẫu cho script mới: hàm thuần export cho vitest cộng nhánh CLI. Nó phải qua `vue-tsc -p tsconfig.node.json` strict (`tsconfig.node.json:16` gồm `scripts/**/*.mjs`), nên kiểu viết bằng JSDoc, không `any` ngầm.
- `tests/frontend/checkReleaseVersion.test.ts`: mẫu vitest cho script, dựng tệp tạm bằng `mkdtempSync`. Test script đặt ở `tests/frontend/`.
- `.github/release-notes.md`: ghi chú tĩnh, giữ nguyên câu "không được ký số và không được notarize" (guard `config_invariants.rs:2839`).
- `src-tauri/tests/config_invariants.rs`: `release_workflow_text` `:2686`, `release_job_lines` `:2695`, `code_lines`. Ca `release_command_pins_notes_file_verify_tag_and_latest_without_clobber` `:2804` đòi `--notes-file .github/release-notes.md` nên phải sửa theo thân mới. Ca `publish_refuses_unless_ci_succeeded…` `:2789` là mẫu kiểm thứ tự bước.

## Tasks & Acceptance

**Execution:**
- [ ] `scripts/lib/dict-fetch.mjs`: export `sha256OfFile`.
- [ ] `scripts/release-checksums.mjs`: tạo mới. `--notes <md> --out-dir <dir> <tệp…>`: ghi `<out-dir>/SHA256SUMS.txt` và `<out-dir>/release-body.md`. Thân gồm ghi chú, rồi mục `### Kiểm tra tệp tải về (SHA-256)` liệt kê từng tệp và hash, kèm lệnh macOS (`shasum -a 256 <tên>` và `shasum -a 256 -c SHA256SUMS.txt`) và Windows PowerShell (`Get-FileHash .\<tên> -Algorithm SHA256`; ghi rõ hoa thường không quan trọng). Mọi lỗi trong ma trận cho exit ≠ 0 trước khi ghi tệp nào.
- [ ] `tests/frontend/releaseChecksums.test.ts`: phủ mọi hàng ma trận. Hash so với `createHash` trên nội dung đã biết, ví dụ chuỗi `abc` cho `ba7816bf…15ad`. Mỗi dòng `SHA256SUMS.txt` khớp `/^[0-9a-f]{64}  [^/\\]+$/`. Thân bắt đầu bằng nguyên văn ghi chú.
- [ ] `.github/workflows/release.yml`: thêm `setup-node` vào `publish`. Trong `create the release`, sau khi kiểm đủ tệp và trước `gh release create`, chạy `node scripts/release-checksums.mjs --notes .github/release-notes.md --out-dir "$RUNNER_TEMP/release" <dmg> <msi>`. `gh release create` đổi sang `--notes-file "$RUNNER_TEMP/release/release-body.md"` và đính thêm `"$RUNNER_TEMP/release/SHA256SUMS.txt"`. Sửa header.
- [ ] `src-tauri/tests/config_invariants.rs`: sửa ca `:2804` theo `--notes-file "$RUNNER_TEMP/release/release-body.md"`. Thêm ca mới: trong `publish`, dòng `node scripts/release-checksums.mjs --notes .github/release-notes.md` đứng trước `gh release create`, và lệnh tạo release đính `SHA256SUMS.txt`.
- [ ] `.github/release-notes.md`: câu "Hướng dẫn từng bước kèm ảnh…" nói thêm rằng mục checksum bên dưới đối chiếu được ngay.

**Acceptance Criteria:**
- Given push tag `v*` trên commit CI xanh, when `publish` xong, then trang Release hiện hash SHA-256 của `.dmg` và `.msi` ngay trong thân, kèm lệnh đối chiếu cho macOS và Windows, và có asset `SHA256SUMS.txt`.
- Given `SHA256SUMS.txt` tải cùng bản cài vào một thư mục, when chạy `shasum -a 256 -c SHA256SUMS.txt` trên macOS, then mọi dòng `OK`.
- Given gỡ bước `release-checksums.mjs`, hoặc trả `--notes-file` về `.github/release-notes.md`, when `cargo test --test config_invariants`, then đỏ đúng ca canh nó.

## Implementation Notes

- Hash được tính trong `publish` bằng `scripts/release-checksums.mjs`, dùng lại `sha256OfFile` của `scripts/lib/dict-fetch.mjs`, nay đã export.
- Job `publish` (`ubuntu-24.04`) thêm `setup-node` cùng SHA với job `build`, không cache npm vì job này không chạy `npm ci`.
- Lệnh của mỗi hệ điều hành chọn tệp theo đuôi: `.dmg` cho macOS, `.msi` cho Windows. `shasum -c` dùng `--ignore-missing` vì `SHA256SUMS.txt` liệt kê cả hai bản cài, mà mỗi người chỉ tải một bản.
- 10.6 lấy lệnh đối chiếu trong `releaseBody()` làm bước checksum của hướng dẫn có ảnh.
- Còn do người làm: chạy `release.yml` thật trên tag `v*` để thấy mục checksum và asset `SHA256SUMS.txt` trên trang Release. Lượt này chạy chung với lượt còn nợ của 10.2.

## Plan Change Log

## Review Triage Log

Lượt 1 (quick): high 0 · medium 3 · low 2 · false 0 · maybe-false 0. Cả năm phát hiện đều vá.
- medium · patch: lệnh `Get-FileHash` trỏ vào `.dmg` (`entries[0]`) trong khi người dùng Windows tải `.msi`, và test khoá luôn hành vi sai đó. Sửa: chọn tệp theo đuôi cho từng hệ điều hành.
- medium · patch: `shasum -c SHA256SUMS.txt` báo "FAILED open" cho `.msi` vắng mặt và exit ≠ 0. Sửa: thêm `--ignore-missing`. Đã đo trên macOS 15.8.1, shasum 6.02: tệp vắng bị bỏ qua, exit 0.
- medium · patch: không gì canh việc cả hai bản cài đều được đưa vào script. Gỡ `"${MSIS[0]}"` thì mọi test vẫn xanh, trái ràng buộc "mục checksum thiếu một tệp làm `publish` đỏ". Sửa: ca Rust đòi cả hai biến trên dòng `node`.
- low · patch: ca trùng tên truyền cùng một đường dẫn hai lần. Sửa: dùng hai thư mục khác nhau.
- low · patch: dòng `Usage:` trong header lặp lại hằng `USAGE`. Sửa: xoá dòng đó.

## Design Notes

Hash tính ở `publish` trên đúng tệp sắp đính, không tính ở job `build`. Người dùng đối chiếu tệp mình tải với asset của release, nên tệp đem hash phải là tệp được upload. Tính thêm ở `build` rồi so lại thì chỉ bắt được hỏng khi chuyển artifact, và `download-artifact` đã tự kiểm digest phần đó. GitHub cũng tự hiện digest cho asset, nhưng con số đó do GitHub sinh lúc upload chứ không phải CI của dự án, và không có tệp nào cho `shasum -c` đọc. Vì vậy nó không thay được AC "sinh trong CI".

## Verification

**Commands:**
- `npx vitest run tests/frontend/releaseChecksums.test.ts`: xanh.
- `cargo test --test config_invariants`: xanh. Đối chứng đỏ bằng phép gỡ thật cho ca mới và ca đã sửa.
- `npm run build`: xanh (vue-tsc strict trên script mới).
- Tay: `node scripts/release-checksums.mjs --notes .github/release-notes.md --out-dir <tạm> <hai tệp bất kỳ>`, rồi `cd <tạm>` cùng hai tệp và chạy `shasum -a 256 -c SHA256SUMS.txt`: mọi dòng `OK`.

**Manual checks (if no CLI):**
- Lượt `release.yml` thật trên tag `v*` (cần push, người làm): trang Release có mục checksum và asset `SHA256SUMS.txt`.
