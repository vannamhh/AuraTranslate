---
title: 'Story 10.6 — Hướng dẫn cài đặt có ảnh chụp màn hình'
type: 'feature'
ticket: '6'
created: '2026-10-21'
status: 'done'
baseline_revision: '42217f1e82104dc3a4b7187f1e205b9846cc2940'
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

**Problem:** Bản cài `.dmg`/`.msi` không ký số; người dịch phổ thông gặp Gatekeeper hoặc SmartScreen ở lần mở đầu và bỏ cuộc. `.github/release-notes.md` hiện chỉ hứa "Hướng dẫn từng bước kèm ảnh sẽ có ở một bản phát hành sau" (FR108).

**Approach:** Một trang hướng dẫn tiếng Việt trong repo, hai phần macOS và Windows, mỗi bước một ảnh chụp từ bản cài thật do `release.yml` dựng: tải về, đối chiếu SHA-256 (đúng lệnh `releaseBody()` in ra), vượt cảnh báo, mở ứng dụng. Ghi chú phát hành trỏ tới trang này. Một ca Rust giữ cho trang không mất phần nào và không có ảnh trỏ vào tệp không tồn tại.

## Boundaries & Constraints

**Always:** Nói thẳng bản cài không ký số và không notarize, nêu lý do (không có kinh phí cho ký số; Apple notarization và chứng chỉ ký mã Windows đều tốn phí; PRD §Rủi ro). Lệnh checksum trùng chữ với `scripts/release-checksums.mjs` `releaseBody()`. Ảnh chụp từ bản cài do CI dựng, tải bằng trình duyệt (để có cờ quarantine / Mark-of-the-Web thật), không từ bản dựng cục bộ chưa gắn cờ. Tên tệp bản cài trong hướng dẫn lấy từ một lượt `release.yml` thật, không suy từ quy ước Tauri.

**Decisions (Ice, 2026-10-21):**
- Gatekeeper: hướng dẫn cả hai đường — macOS 15 trở lên qua Cài đặt hệ thống → Quyền riêng tư & Bảo mật → "Vẫn mở"; macOS 14 trở xuống chuột phải → Mở. Câu FR108 sửa tại chỗ (🔵 + ngày) qua lượt replan của Sofia, không sửa tay PRD. Ảnh nhánh ≤14 chỉ chụp khi có máy cũ; không có thì là mục nợ `Chủ: Epic 10`.
- Nguồn bản cài: `release.yml` chạy bằng `workflow_dispatch` trên `master`, tải artifact bằng trình duyệt.
- Ảnh: agent chụp phần macOS; Ice chụp phần Windows trên máy Windows. Story chưa sang `built` khi chưa có ảnh Windows.
- Chỗ đặt: `INSTALL.md` ở gốc repo, ảnh trong `install-guide/`.

**Never:** Không thêm ký số, ad-hoc signing hay notarize trong story này (đổi bất biến ⇒ AD mới của Winston). Không đặt hướng dẫn vào `docs/` (đầu vào thô của `tools/dict-build`). Không bảo người dùng tắt Gatekeeper toàn hệ thống (`spctl --master-disable`) hay tắt SmartScreen. Không viết bước mà chưa có ảnh của chính bước đó.

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected Output / Behavior | Error Handling |
|----------|--------------|---------------------------|----------------|
| macOS lần mở đầu | `.dmg` tải bằng trình duyệt, kéo vào Applications, mở | Có bước + ảnh cho đúng hộp thoại máy hiện ra và đường vượt qua | Không có |
| Windows lần chạy `.msi` | `.msi` tải bằng Edge/Chrome, nháy đúp | Bước + ảnh: SmartScreen → More info → Run anyway, rồi trình cài WiX | Không có |
| Checksum lệch | Hash in ra khác dòng trong `SHA256SUMS.txt` | Hướng dẫn nói: đừng mở, tải lại, vẫn lệch thì báo issue | Câu rõ ràng, không lệnh sửa |
| Trình duyệt chặn tải | Edge/Chrome báo "không thường được tải xuống" | Bước + ảnh giữ tệp (Keep) | Không có |

</frozen-after-approval>

## Code Map

- `.github/release-notes.md:9` -- câu hứa hướng dẫn "ở một bản phát hành sau"; thay bằng liên kết tới trang hướng dẫn. Câu ở `:7` được `release_notes_state_the_build_is_not_signed_or_notarized` (`src-tauri/tests/config_invariants.rs:2915`) giữ — không đổi chữ.
- `scripts/release-checksums.mjs:37-62` `releaseBody()` -- nguồn duy nhất của lệnh checksum: macOS `shasum -a 256 <dmg>` + `shasum -a 256 -c --ignore-missing SHA256SUMS.txt`; Windows `Get-FileHash .\<msi> -Algorithm SHA256`; "Hoa thường trong hash không quan trọng". Tệp `SHA256SUMS.txt`, dòng `<hex64>  <tên>`.
- `.github/workflows/release.yml` -- `workflow_dispatch` chỉ dựng + kiểm và đẩy artifact; tag `v*` mới tạo Release. Chưa có lượt nào (`gh run list --workflow release.yml` rỗng); chưa có release ứng dụng (chỉ `dict-*`).
- `src-tauri/tauri.conf.json:34` `targets: ["dmg","msi"]`; `:51` WebView2 `offlineInstaller`; không có `signingIdentity` ⇒ bundle macOS hoàn toàn không ký.
- `src-tauri/tests/config_invariants.rs` -- chỗ thêm ca giữ trang hướng dẫn (đọc bằng `repo_root()` như ca release-notes).
- `_bmad-output/initiative-auratranslate/prd-auratranslate/prd-auratranslate.md:892` FR108 (ghi đường "chuột phải → Mở"); `:1113` câu lý do không ký số.
- Repo chưa có `README*`, `INSTALL*` hay thư mục ảnh nào ngoài `src-tauri/icons` và `docs/`.

## Tasks & Acceptance

**Execution:**
- [x] Lấy bản cài thật -- chạy `release.yml` bằng `workflow_dispatch` trên `master`, tải artifact `.dmg`/`.msi` bằng trình duyệt, ghi tên tệp và run id vào Implementation Notes -- tên và cờ tải về phải là thật.
- [x] Đo hộp thoại macOS -- mở `.dmg` đã tải trên máy Ice (macOS 15.8.2), ghi nguyên văn hộp thoại hiện ra. Nếu là "bị hỏng, không thể mở" thay vì cảnh báo nhà phát triển ⇒ HALT, đưa Ice các lựa chọn kèm số đo -- đường vượt phụ thuộc trạng thái chữ ký thật.
- [x] `install-guide/macos-*.png` -- agent chụp (`screencapture` + skill `computer-use`) từng bước macOS 15 trên bản đã tải: mở `.dmg`, kéo vào Applications, hộp thoại chặn, Quyền riêng tư & Bảo mật → "Vẫn mở", xác nhận, cửa sổ AuraTranslate đầu tiên; cộng ảnh Terminal chạy lệnh checksum. Đặt tên theo thứ tự bước, PNG, cắt gọn -- ảnh thật của đúng bản phát hành.
- [x] `install-guide/windows-*.png` -- gửi Ice danh sách bước và tên tệp cần chụp (tải trên Edge, Keep, SmartScreen → More info → Run anyway, trình cài WiX, cửa sổ đầu tiên, PowerShell `Get-FileHash`); HALT chờ ảnh -- máy này không có Windows.
- [x] `INSTALL.md` -- trang tiếng Việt: mở đầu không ký số + lý do; phần macOS (cả hai đường Gatekeeper, nhánh ≤14 ghi rõ phiên bản); phần Windows; bước checksum trùng chữ `releaseBody()`; xử lý checksum lệch.
- [x] Sửa FR108 -- gọi `bmad-agent-po` (Sofia) replan để câu FR108 nhận đường macOS 15+ bằng 🔵 + ngày; nếu thiếu ảnh nhánh ≤14, thêm mục nợ `Chủ: Epic 10` vào `deferred-work.md`.
- [x] `.github/release-notes.md` -- câu `:9` thành liên kết `https://github.com/vannamhh/AuraTranslate/blob/master/INSTALL.md`.
- [x] `src-tauri/tests/config_invariants.rs` -- ca mới: trang tồn tại, có hai phần macOS/Windows, có câu không ký số, có đúng lệnh checksum lấy từ `release-checksums.mjs`, mọi `![..](..)` trỏ tới tệp có thật; ghi chú phát hành liên kết tới trang. Đối chứng đỏ: xoá thật một ảnh, ca đỏ đúng tên ảnh.

**Acceptance Criteria:**
- Given trang hướng dẫn, when người dùng macOS đọc, then mỗi bước từ tải về tới cửa sổ AuraTranslate đầu tiên có một ảnh chụp, và bước vượt Gatekeeper khớp hộp thoại đã đo.
- Given trang hướng dẫn, when người dùng Windows đọc, then mỗi bước từ tải về tới cửa sổ đầu tiên có một ảnh, và bước SmartScreen ghi tường minh More info → Run anyway.
- Given trang hướng dẫn, when đọc phần mở đầu, then nó nói thẳng bản cài không ký số, không notarize, và vì sao.
- Given bước đối chiếu checksum, when so với `releaseBody()`, then lệnh trùng chữ cho cả hai hệ điều hành.
- Given ghi chú phát hành, when dựng release body, then có liên kết tới trang hướng dẫn và không còn câu "sẽ có ở một bản phát hành sau".

## Implementation Notes

- Lượt `release.yml` đầu tiên (`39536246718`) đỏ ở bước kiểm bản cài cả hai nền tảng; sửa ở `5858c2f` (ngoài phạm vi 10.6, commit riêng `fix(release): 10-5`). Lượt xanh dùng cho ảnh: `39539697489` ở `5858c2f2fe5862b347a34cd70eb00208886f5676`. Tên tệp thật: `AuraTranslate_0.1.0_universal.dmg`, `AuraTranslate_0.1.0_x64_en-US.msi`.
- `.dmg` tải bằng Brave (artifact zip, giải nén bằng Archive Utility), mang `com.apple.quarantine`; bản trong `/Applications` chép bằng Finder vẫn mang cờ; `codesign -dv`: "code object is not signed at all"; `spctl`: rejected.
- macOS 15.8.2 Intel (i9-9980HK): hộp thoại đầu là "“AuraTranslate.app” Not Opened" với Done / Move to Bin (không phải "is damaged"); Privacy & Security hiện "Open Anyway"; xác nhận "Open “AuraTranslate.app”?" với Move to Bin / Open Anyway / Done, rồi mật khẩu. Máy Intel chạy lát x86_64 ⇒ Apple Silicon chưa đo.
- Kéo thả bằng `orca computer drag` không có hiệu lực (synthetic input); bản sao vào `/Applications` làm bằng `Finder duplicate` qua AppleScript, cùng đường chép của Finder.
- Ảnh hộp thoại Gatekeeper và xác nhận do Ice chụp (agent không thấy cửa sổ `CoreServicesUIAgent`); ảnh có tên/ảnh đại diện/tab Finder của Ice đã cắt bỏ; cửa sổ app mở trên dữ liệu dev của Ice nên chỉ giữ phần trên.
- `SHA256SUMS.txt` cho ảnh checksum dựng bằng chính `scripts/release-checksums.mjs` trên tệp đã tải (lượt `workflow_dispatch` không tạo Release); dấu nhắc Terminal tạm `$ ` qua `zsh -f`.
- Ảnh macOS 14 trở xuống: không có máy ⇒ nợ `Chủ: Epic 10`.
- Windows (Ice, máy ảo VMware Windows 11): chạy `.msi` từ thư mục chia sẻ `\\vmware-host\…` thì SmartScreen KHÔNG hiện (UAC ghi "File origin: Network drive"). Chép `.msi` vào ổ C và gắn tay `Zone.Identifier` `ZoneId=3` thì SmartScreen hiện "Windows protected your PC" → More info → Run anyway; UAC "Publisher: Unknown". `Get-FileHash` cùng `2AE80CD9…C65C9` ở cả hai lượt. Trình cài WiX: Welcome → Destination Folder → Ready (Install có khiên UAC) → Installing → Completed với ô "Launch AuraTranslate" đánh sẵn.
- Ca `install_guide_keeps_its_sections_checksum_commands_and_every_image` đọc phiên bản từ `tauri.conf.json`, không cứng `0.1.0`; giữ hai chiều ảnh ↔ tham chiếu. Đối chứng: gỡ thật `macos-02-gatekeeper-chan.png` thì đỏ đúng tên ảnh.
- Ảnh Edge "Keep": Ice chọn ghi nợ (`Chủ: Epic 10`), bước còn chữ, không ảnh. Nợ macOS ≤14 và Apple Silicon cùng chủ; mục `license/` Windows `Chủ: B7` đóng bằng lượt `39539697489`.

## Plan Change Log

## Review Triage Log

Lượt 1 (quick): high 0 · medium 3 · low 2 · false 2 · rejected-by-decision 3.
- medium · patch — `INSTALL.md` Windows Bước 1 khẳng định cảnh báo Edge "chỉ hiện khi tải thẳng" mà chưa đo; câu được thu về "chưa có ảnh".
- medium · patch — `INSTALL.md` "các bước vẫn vậy" trên Apple Silicon chưa đo; thay bằng "chưa kiểm, gặp 'is damaged' thì báo Issues".
- medium · patch — ca `install_guide_…` xanh khi xoá dòng tham chiếu ảnh; thêm chiều mọi `.png` trong `install-guide/` phải được tham chiếu.
- low · rejected — `matches("đừng mở tệp").count() == 2` là đếm câu chữ: đỏ khi đổi lời là đúng việc cần, sửa khác chỉ thêm độ phức tạp.
- low · patch — một gạch Implementation Notes kể theo ngôi thứ nhất; viết lại.
- rejected · Ice quyết — bước Edge "Keep" không ảnh (Ice chọn (b), nợ `Chủ: Epic 10`).
- rejected · Ice quyết — nhánh macOS ≤14 không ảnh (quyết định 1a, nợ `Chủ: Epic 10`).
- rejected · Ice quyết — SmartScreen đo với `Zone.Identifier` gắn tay (Ice làm theo cách b; nợ đo lại ở bản phát hành đầu).
- false — bộ lọc `l.contains("dmg")` chọn nhầm dòng: `assert_eq!(mac.len(), 1)` đỏ ngay khi có dòng thứ hai, và chỉ dòng macOS mang `dmg`.
- false — diff lẫn bản sửa 10-5: nó nằm ở commit riêng `5858c2f`, diff chỉ tính từ baseline; "Tải từ trang Releases" đúng vào lúc hướng dẫn được dùng (sau tag `v*`).

## Verification

**Commands:**
- `cargo test --manifest-path src-tauri/Cargo.toml --test config_invariants` -- expected: ca mới xanh; xanh trước khi `npm run build` có `dist/`.
- `node scripts/release-checksums.mjs` theo cách ca hiện có gọi (nếu có) -- expected: release body mang liên kết tới trang.

**Manual checks (if no CLI):**
- Mở trang trên GitHub (nhánh đẩy lên) và xem mọi ảnh hiện ra.
