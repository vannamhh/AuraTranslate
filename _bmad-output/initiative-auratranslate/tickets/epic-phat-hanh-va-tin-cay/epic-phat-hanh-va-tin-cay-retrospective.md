---
epic: epic-phat-hanh-va-tin-cay
date: 2026-10-22
verdict: accepted-with-open-items
criteria: declared
headless: false
---

# Retrospective — Epic 10: Phát hành & tin cậy

## Epic summary

Epic `epic-phat-hanh-va-tin-cay` (id 10), chín ticket, theo thứ tự `tickets.py status`:

| ref | tên | status / state |
|---|---|---|
| 10.1 | Build công khai qua GitHub Actions | built / review |
| 10.2 | Phát hành cho macOS và Windows | built / review |
| 10.3 | Checksum SHA-256 | built / review |
| 10.4 | Màn hình Attribution | built / review |
| 10.5 | Giấy phép trong bản phát hành | built / review |
| 10.6 | Hướng dẫn cài đặt có ảnh chụp màn hình | done / done |
| 10.7 | Cập nhật chỉ kiểm tra và thông báo | done / done |
| 10.8 | Chính sách gỡ bỏ dữ liệu | built / review |
| 10.9 | Nghiệm thu cuối các ngưỡng phi chức năng | done / done |

`pending_tickets` rỗng. Sáu ticket còn ở `built` (10.1–10.5, 10.8). Ice không nêu trọng tâm nào trước khi phân tích.

### Khoảng diff theo plan

Baseline xếp theo tổ tiên. 10.1–10.4 dựng ngày 2026-10-12, 10.5–10.9 ngày 2026-10-21; Epic 12 chen giữa đúng như Notes của epic khai.

| ref | baseline | khoảng dùng | commit | ghi chú |
|---|---|---|---|---|
| 10.1 | `2fe7009` | `2fe7009..bc43b3b` | 14 | chứa `1a8e6a5` (`cargo fmt` lần đầu, 194 tệp) và `3033e79` (clippy tự sửa): churn cơ học làm phồng 46.476 dòng của khoảng |
| 10.2 | `bc43b3b` | `bc43b3b..e7e88bb` | 8 | |
| 10.3 | `e7e88bb` | `e7e88bb..3dff2dc` | 2 | |
| 10.4 | `3dff2dc` | `3dff2dc..f646a8c` | 2 | cắt: khoảng ghi nhận chạy tới `95936b4`, chứa ~55 commit Epic 12 / Story 2-3 / sổ nợ |
| 10.5 | `95936b4` | `95936b4..42217f1` | 1 | các bản vá 10-5 sau đó nằm trong khoảng của 10.6 (`5858c2f`) và 10.9 (`b553b9d`) |
| 10.6 | `42217f1` | `42217f1..043cd07` | 3 | |
| 10.7 | `043cd07` | `043cd07..2e488d7` | 2 | |
| 10.8 | `2e488d7` | `2e488d7..9b71b27` | 2 | |
| 10.9 | `9b71b27` | `9b71b27..a304dc9` | 12 (1 merge, đã đo) | đầu cuối suy ra = HEAD; chứa `68cd7fa` (cổng của 10-7/10-8), `b553b9d` (10-5), `28f1389` (10-7) |

Commit của Epic 10 nằm ngoài mọi khoảng plan: `a8489d2` (CI `master` đỏ cả hai OS từ sau 10-1) và `b0db152` (26 mục nợ chủ 10-9, vào trước baseline của 10.9).

### Bằng chứng

- Có: tệp epic với Done when 5 tiêu chí; chín plan đều có `baseline_revision`, Review Triage Log và Verification; `tickets.toml`; một story file (`story-build-cong-khai-qua-github-actions.md`), tám entry còn lại chưa từng kéo ra tệp; `git_evidence.py` cho từng khoảng; retro Epic 12 (epic liền trước theo thứ tự `epics`); lượt CI, nightly và `release.yml` đọc qua `gh`; sổ nợ (grep).
- Thiếu: session log. Phân tích quy trình chỉ dựa vào thông điệp commit và Implementation Notes của plan.
- Phạm vi thu hẹp: lượt review mã chạy ba lens inline (bmad-review đã nạp, không tách agent theo lens), chỉ đọc mã, không chạy test, không đối chứng gỡ. Không đọc: `check-i18n.mjs`, `WorkspaceDock.vue`/`WorkspaceMode.vue` trong `b0db152`, `check-panel-refs.mjs`, bộ đo nfr-bench, văn xuôi INSTALL.md và 16 ảnh; layout/a11y của `AttributionOverlay.vue`.

## Findings

Mỗi phát hiện: nguồn · xử lý vụ việc · bài học ngăn lần sau.

### Đối chiếu spec ↔ mã (Done when, FR105–FR112)

- **S1 — Done when 1 chưa đạt: chưa có GitHub Release nào của ứng dụng.** `gh release list` và API `releases?per_page=100` (đọc 2026-10-22) chỉ có `dict-v1`, `dict-v2`, `dict-raw-v1`; không có tag `v*` trên remote. Job `publish` (checksum + `gh release create`) bị bỏ qua ở mọi lượt dispatch (`39536246718` đỏ, `39539697489`, `39576340751`, `39580744916` xanh, đều chỉ build và inspect). Verification của 10.1, 10.2, 10.3 và triage "quick 3" của 10.9 đều giao lượt thật cho Ice; nợ `deferred-work.md:367` vẫn 🟡 `Chủ: Story 10.1`. FR105/106/107 có mã và test, chưa có sản phẩm. · **Sửa ngay** (Ice đẩy `v0.1.0`). · Bài học: tiêu chí "đã phát hành" chỉ đóng bằng một lần phát hành thật; story mang nó không lên `built` chỉ vì pipeline sẵn sàng.
- **S2 — NFR19 trong `covers` của 10.9 và trong Done when 5 là lỗi chép của lượt di trú, không phải yêu cầu bị rơi.**
  - Nguồn v6 `archive-v6/epics.md:7178` ghi 10.9 phủ "nghiệm thu cuối toàn bộ **NFR1–NFR19**" cùng NFR3, NFR4, NFR5.
  - Lượt di trú `016f2ce` chép khoảng này thành hai id rời `"NFR1", "NFR19"` (`tickets.toml:407`), và Done when 5 của tệp epic sinh ra cùng commit đó.
  - Không AC nào của 10.9, cả ở v6 lẫn ở plan, nhắc NFR19. NFR19 thuộc Story 6.8 "Allowlist mạng hai tầng và nhật ký domain" (`archive-v6/epics.md:4889`, `covers = ["NFR19", "NFR12"]`, đã `done`).
  - Grep "NFR19" trong plan 10.9 và REPORT.md ra 0 là đúng phạm vi của 10.9.
  - · **Đối chiếu spec**: Sofia sửa `covers` của 10.9 và Done when 5. · Bài học: lượt di trú đổi một khoảng id thành danh sách thì phải bung khoảng ra, hoặc giữ nguyên văn; không cắt lấy hai đầu.
- **S3 — NFR4 và NFR5 vượt ngưỡng; Q4 đóng bằng chữ "vượt".** REPORT.md hàng 2: A6 đạt (183,9 < 500 ms), A7 vượt (3.151,7 > 3.000 ms), A8 vượt (700,0 > 300 MB). Đo trên macOS 15.8.2 i9-9980HK với load average 8,70–9,75, không có số Windows. PRD:1236 ghi Q4 "ĐÃ ĐÓNG 2026-10-21 (Story 10.9)". Ice đã ký 10.9 (`e5665e0`), hai mục vượt 🟡 `Chủ: Ice`. · **Chấp nhận** như một sai lệch đã ký, ghi lại để retro sau không gắn cờ lại. · Bài học: không.
- **S4 — Notes của epic sai về nghĩa vụ thông báo tác giả.** Notes viết "không story nào của epic trích nó", nhưng `tickets.toml:372` (AC của 10.9) yêu cầu thông báo cho tác giả Đặng Thế Kiệt. Nghĩa vụ vẫn 🟡 `Chủ: Ice` ở `deferred-work.md:396`, vì dữ liệu HVTĐTD chưa được đóng gói. · **Đối chiếu spec** (Sofia sửa dòng Notes). · Bài học: không.
- **S5 — Mười một kiểm tra trên app thật `Chủ: Epic 10` còn mở.** Các dòng của `deferred-work.md`: 20 (font trên Windows, 🟡), 205 và 13414 (⌘1..⌘3 qua OS, NFR17 trên `.app`), 1160 và 1240 (Attribution trên WKWebView, Escape trả tiêu điểm), 13392 (INSTALL.md thiếu ảnh macOS ≤ 14), 13395 (Gatekeeper trên Apple Silicon), 13398 (thiếu `windows-01-edge-giu-tep.png`, SmartScreen chỉ đo với `Zone.Identifier` đặt tay), 13403 (Cài đặt → Cập nhật chưa thử trên app thật), 13408 (Attribution trên `.app` đã gỡ một lớp), 13417 (tương phản hai theme bằng mắt). `check:debt-owner` hiện xanh chỉ vì epic chưa `done`. · **Sửa ngay** (lượt dùng thật của Ice). · Bài học: không, quy trình đang chạy đúng thiết kế.
- **S6 — Dòng `reqwest` trong Stack table của spine (khoảng dòng 1191) chưa ghi `core::update` là nơi dùng.** Theo AD-15 thì điểm ra mạng này hợp lệ (điểm 2), nên chỉ lệch về chữ. · **Đối chiếu spec** (Winston). · Bài học: không.

### Review mã (adversarial · edge-case · verification-gap), trọng tâm ranh giới giữa các ticket

- **R1 — `NOTICE.txt` liệt kê lớp dữ liệu bằng tay, không cổng nào nối nó với manifest hay `DETACHABLE_ALL`** (ranh giới 10.5 ↔ 10.8, medium). `NOTICE.txt:12-13` ghi cứng `dict-core.db, dict-thieu-chuu.db, dict-vietphrase.db, dict-tran-van-chanh.db` là "shipped in dict/". `DATA-REMOVAL.md` bước 3 bắt sửa tay. Nếu bỏ sót bước này, mọi cổng vẫn xanh: `data_removal_guide_keeps_the_files…` ở `config_invariants.rs:3132-3171` chỉ kiểm chuỗi có mặt. Hệ quả: bản cài mang một tuyên bố pháp lý sai, đi ngược ý Done when 3 "gỡ một nguồn không đổi gì khác". Nguồn: `42217f1`, `9b71b27`. · **Sửa ngay**. · Bài học: văn bản phát hành mà nội dung phụ thuộc dữ liệu phải sinh ra từ dữ liệu, hoặc có cổng so khớp.
- **R2 — `release.yml` truyền `--sizes` cho `release-checksums.mjs`, nhưng không gì canh việc đó** (10.3 ↔ 10.9, medium, đúng lớp "im lặng rỗng"). `release-checksums.mjs:125`: `sizes.length > 0 ? checkSizes(…) : []`. Bỏ hai tham số `--sizes` ở `release.yml:194` thì script vẫn exit 0 và ghi chú phát hành mất mục NFR6. Không ca nào trong `config_invariants.rs` nhắc `sizes`/`nfr6`. Nguồn: `d65dacd`. · **Sửa ngay**. · Bài học: không.
- **R3 — Bất biến "checksum phủ mọi asset tải lên" chỉ được canh cho hai bản cài** (10.3, medium-low). `config_invariants.rs:2926-2949` kiểm `.dmg`/`.msi` có trong lệnh sums và `SHA256SUMS.txt` có trong lệnh create. Thêm asset thứ ba vào `gh release create` (`release.yml:195`) thì test vẫn xanh. Nguồn: `ea5f3eb`. · **Hoãn** (nối vào story sửa R1/R2). · Bài học: không.
- **R4 — Kiểm tra cập nhật thất bại vẫn để kết quả cũ trên màn hình** (10.7, low-medium). `src/updateCheckState.ts:44-49` trả về khi có lỗi mà không xoá `outcome`; `SettingsUpdateSection.vue:47-51` hiện cả khối `role=status` "Đã là bản mới nhất" lẫn cảnh báo lỗi. Doc comment dòng 37 cho thấy đây là chủ ý. · **Ice quyết** (giữ, hoặc làm mờ/xoá kết quả cũ). · Bài học: không.
- **R5 — Kiểm tra cập nhật chỉ đọc trang đầu `per_page=100`** (`core/update/mod.rs:7`), không xử lý `Link`. Release `dict-*` dùng chung danh sách này. Có hơn 100 release mới hơn bản ứng dụng cuối thì kết quả thành `UpToDate`/`NoAppRelease` mà không báo lỗi. Nguồn: `a4778db`. Hiện mới có 3 release. · **Hoãn**. · Bài học: không.
- **R6 — `gh release create --latest` vô điều kiện** (`release.yml:195`). Tag hotfix trên nhánh cũ (`v0.1.1` sau `v0.2.0`) sẽ thành Latest; prerelease bị chặn chỉ nhờ `TAG_RE` (`check-release-version.mjs:11`). Kiểm tra cập nhật của 10.7 không đọc cờ Latest nên không bị ảnh hưởng. Nguồn: `dfa8ea8`. · **Hoãn**. · Bài học: không.
- **R7 — Ghi chú phát hành tĩnh nói "Ứng dụng không tải thêm gì sau khi cài"** (`.github/release-notes.md:3,15`). Câu này viết trước khi 10.7 thêm lời gọi GitHub API do người dùng bấm, và 10.7 không xem lại. · **Ice quyết** câu chữ. · Bài học: story mở thêm điểm ra mạng thì grep các lời hứa ngoại tuyến trong văn bản phát hành.
- **R8 — Sàn `MIN_BUNDLED_LICENSES = 9` không bắt được việc mất một phần tệp giấy phép** (`dict-fetch.mjs:200-205`, `inspect-release.mjs:23-29`). Tập kỳ vọng lấy từ chính thư mục nguồn, nên mất một tệp nguồn thì tệp đó cũng ra khỏi tập kỳ vọng; 12 xuống 9 vẫn xanh. Nguồn: `42217f1`. · **Hoãn**. · Bài học: không.
- **R9 — Guard INSTALL.md ghim tên tệp theo phiên bản** (`config_invariants.rs:3025-3041`), nên lần đầu nâng phiên bản sẽ đỏ trong khi 16 ảnh vẫn mang 0.1.0. Ghi chú phát hành trỏ `blob/master/INSTALL.md`, không trỏ bản theo tag. Nguồn: `da0f733`. · **Hoãn**. · Bài học: không.
- **R10 — Bốn mục thấp, hoãn chung:**
  - `release-checksums.mjs:82` rơi về `entries[0]` khi thiếu `.dmg`/`.msi`.
  - Cổng "CI xanh" của `publish` chỉ được canh theo thứ tự dòng (`config_invariants.rs:2876-2889`).
  - `compile_cached` một ô toàn cục trong `chapterpattern.rs:101-115` (`b0db152`).
  - `fetchDicts` không có timeout (`dict-fetch.mjs:84-95`).
- **Đã kiểm và sạch:**
  - Không có danh sách nguồn từ điển ghi cứng trong `src/`, `src-tauri/src`; tên nguồn chỉ còn trong comment. Attribution dựng từ `dict_source` của các `.db` đính kèm. Ca `dict_sources.rs` `deleting_a_file_removes_its_whole_attribution_block_and_leaves_no_orphan` có mặt.
  - Không có `tauri-plugin-updater`; `check-deps.mjs:199,268` cấm nó.
  - `update_check` chỉ chạy khi bấm: GET, `redirect::Policy::none()`, trần 4 MiB / 15 s, 403/429 → `RateLimited`, không 2xx → `BadResponse`, không bao giờ thành `UpToDate`.
  - Opener không có trong `capabilities/main.json`.
  - `contents: write` chỉ ở `publish`.
  - Tag `dict-v*` không khởi động `release.yml`.
  - Việc đổi `NOTICE` → `NOTICE.txt` nhất quán ở mọi nơi.
  - Probe `b553b9d` đọc `bundle.resources` thay vì ghi cứng.
  - Phép duyệt FAT/DIFAT của `msi-streams.mjs` đúng (tên `END_OF_CHAIN` gây hiểu nhầm).

### Góc nhìn tổng hợp

- **A1 — `config_invariants.rs` tăng từ 2.355 lên 3.171 dòng (70 `fn`), qua 14 commit của bốn ticket (10.2, 10.3, 10.6, 10.8).** Repo không có cổng trần số dòng mỗi tệp: `check-layout.mjs:117 FILE_FLOOR` là sàn đếm tệp. Tệp này chưa phải lớn nhất (`segment_contract.rs` 12.164 dòng), nhưng gom bất biến phát hành, cài đặt và dữ liệu vào một chỗ. · **Hoãn** (tách `release_invariants.rs`). · Bài học: không.
- **A2 — `scripts/test-story.mjs:71` vẫn dùng `new URL('..', import.meta.url).pathname`**, đúng dạng đã làm đỏ `summarize.mjs` trên Windows (`841b181`). Đây là chỗ `.pathname` duy nhất còn lại; 18 tệp khác đã dùng `fileURLToPath`. Hiện chưa lộ vì script không chạy trên Windows CI. · **Sửa ngay** (một dòng). · Bài học: một bản vá theo lớp lỗi phải grep cùng lớp đó trên toàn repo.
- **A3 — Trùng lặp nhẹ.**
  - `sha256OfFile` nằm trong `scripts/lib/dict-fetch.mjs:41` nhưng các script phát hành cũng dùng (`release-checksums.mjs:9`).
  - Luật "đúng một `.dmg` / một `.msi`" được mã hoá ở bốn chỗ (`release.yml:83-85,107-108,187-190`, `release-checksums.mjs:82-84`).
  - `src/config/update.ts` chép thêm một bản `isIpcError` (repo có 21 bản; epic này không mở đầu mẫu đó).
  - Danh sách lớp, parser TOML và hash đều đã gom về một nơi.
  - · **Chấp nhận / hoãn**.
- **A4 — Kiến trúc.** `core/update` + `commands/update.rs` là điểm 2 của AD-15, nên không cần AD mới. Allow-list `reqwest` được canh hai chiều (`webimport_boundary.rs:200,245-250`). Không thêm phụ thuộc mới: chỉ nâng `rust-version` lên 1.97.1 và dùng lại `tauri-plugin-opener` có sẵn từ Epic 12. Lệch chữ duy nhất là S6.
- **A5 — Lệch mẫu nhỏ.** `updateCheckState.ts`, `config/update.ts` và `dictSourcesState.ts` không có predicate `…HasLoaded`. Trạng thái "chưa biết" vẫn là `null`, tức đúng luật Option/NULL, nên không có lỗi. · **Chấp nhận**.

### Quy trình (từ commit và lượt CI; không có session log)

- **P1 — Đỏ muộn, lặp đúng bài học P1 của Epic 12.**
  - Push CI trên `master` đỏ ở `42217f1` (`39536205940`) và `5858c2f` (`39539696497`). Thông điệp của `b553b9d` tự ghi: "hai lượt CI push của 10-5 trên `master` đỏ mà chưa ai đọc".
  - 10-7 và 10-8 chưa từng qua pre-push: lượt push đầu của nhánh lộ hai cổng đỏ (`68cd7fa`).
  - Hai lỗi chỉ có trên Windows lộ ra sau khi 10.9 đã `built` (`841b181`, `28f1389`).
  - · Bài học: xem hành động 9.
- **P2 — Ký done trước khi đọc CI.**
  - 10.6 (`043cd07`, 05:29Z) và 10.7 (`2e488d7`, 06:46Z) được ký khi lượt push mới nhất trên `master` (`5858c2f`, 03:38Z) đang đỏ.
  - 10.9 được ký ở `e5665e0` (12:26Z). Lượt PR CI trên chính commit đó (`39599967844`, 12:29Z) đỏ; ba bản vá đến sau; push CI trên `a304dc9` (`39637282641`) đỏ ở Windows.
  - AGENTS.md §Tests đòi đọc CI trước `done`.
  - · Bài học: commit "Ice ký done" ghi id lượt CI xanh mà nó dựa vào.
- **P3 — `master` chưa xanh lại từ `0501f3e` (2026-10-20).**
  - `a304dc9` đỏ ở `store_contract::the_wal_stops_growing_once_it_crosses_the_threshold` (+247.200 B, dung sai 0). Đây là ca đỏ lâu năm, nợ `deferred-work.md:13054`, `Chủ: Ice`, nhưng con số mới này chưa được ghi vào mục đó.
  - Nightly `39676620290` trên `a304dc9`: `e2e (macos-26)` xanh, Windows bị huỷ sau khoảng một giờ.
  - · **Sửa ngay** (ghi lý do). · Bài học: không thêm.

## Behavior verification

- **Đã chạy:**
  - Gọi thật `GET https://api.github.com/repos/vannamhh/AuraTranslate/releases?per_page=100` (2026-10-22): HTTP 200, 3 release, không có tag `vX.Y.Z`. Theo `highest_app_version` và `decide` (`core/update/mod.rs:64-85`), kết quả là `NoAppRelease`; quan sát này đối chiếu với mã, không chạy qua app.
  - Nightly `39676620290` trên `a304dc9` (chứa mọi mã của Epic 10): `e2e (macos-26)` xanh, gồm `story-12-18-settings` (mục Cập nhật có đúng công tắc và nút kiểm tra, không có liên kết) và `attribution-focus`.
  - Dispatch `release.yml` `39580744916` (`03376ab`): build và inspect xanh trên cả hai OS. Payload NFR6 là 432.363.050 B trên macOS; 415.937.424 B cộng WebView2 214.156.024 B trên Windows.
- **Chưa chạy:**
  - Bấm kiểm tra cập nhật trên app đóng gói.
  - Mở Attribution, mở văn bản giấy phép, sao chép ghi công trên `.app`.
  - Gỡ một lớp rồi mở Attribution trên `.app`.
  - Cài qua Gatekeeper/SmartScreen.
  - Job `publish`.
  - Các mục này là S1 và S5.

## Previous-retro follow-through

Retro Epic 12 (`epic-hoan-thien-giao-dien-theo-mockup-retrospective.md`, phán quyết `rejected`) có 12 hành động. Mục Previous-retro follow-through của nó không ghi mục nào chưa xong (Epic 9 không có retro).

1. Push và đọc CI, kể cả Windows (Ice): **một phần**.
   - Đã push; PR #1 merge thành `a304dc9`. Lý do đỏ được ghi ở `eb97bbf`, `0501f3e`, `b553b9d`, `841b181`, `28f1389`.
   - Không có nightly nào chạy trên `e93c5ff`.
   - Chỗ đỏ trên `a304dc9` chưa có lý do ghi lại (P3).
2. Sửa R1 PROMOTE (Epic 12): **đã xong**, `fa44774`.
3. Story dọn teardown (Epic 12): không tìm thấy bằng chứng.
4. Các mục code nhỏ trong sổ (Epic 12): không tìm thấy bằng chứng.
5. Xác minh S5 (Epic 12): không tìm thấy bằng chứng.
6. Các quyết định của Ice: không tìm thấy bằng chứng.
7. Lượt dùng thật trên bản đóng gói macOS (Ice): không tìm thấy bằng chứng. Sổ vẫn còn mục `Chủ: Epic 12` ở dòng 13362, 13366, 13370.
8. Đóng `dw:671` và `dw:6064` (Epic 12): không tìm thấy bằng chứng.
9. Đối chiếu spec (Winston, Sofia): không tìm thấy bằng chứng.
10. Hoãn thành nợ có chủ, phần còn lại giao "Epic 10": **không tìm thấy mục nợ `Chủ: Epic 10` nào cho A-S2, A-S3, A-D4, A-D5 hay cổng R4**.
11. Bài học quy trình (Ice): không tìm thấy bằng chứng trong AGENTS.md. Bài học P1 lặp lại ở Epic 10 (P1 ở trên).
12. Sửa R5 FR77 (Epic 12): **đã xong**, `1fc0a10`, `5883601`. `src-tauri/src/lib.rs:1362` đăng ký `ai_config_test_connection` qua `aitranslate::wire`.

## Action items

Mọi mục là **đề xuất**; retro không sửa mã, spec hay sổ nợ.

1. **Phát hành thật `v0.1.0`** rồi đọc job `publish`: release có `.dmg`, `.msi`, `SHA256SUMS.txt`, mục NFR6 trong thân. Xong thì đóng `dw:367` và Done when 1 (S1). Chủ: Ice.
2. **Lượt dùng thật** cho 11 mục `Chủ: Epic 10` (S5), gộp với hành động 7 của Epic 12 nếu tiện. Mục Windows (dòng 20) chuyển sang `B7`. Chủ: Ice.
3. **Sửa lỗi chép NFR19 của lượt di trú** (S2): `covers` của 10.9 và Done when 5 của tệp epic trở về đúng nghĩa v6, tức NFR1–NFR19 là phạm vi rà, NFR3/4/5 là ba phép đo; NFR19 không còn đứng như một mục riêng. Chủ: Sofia.
4. **Ghi lý do chỗ đỏ Windows trên `a304dc9`** (+247.200 B) vào `dw:13054`, và đưa `master` xanh lại (P3). Chủ: Ice.
5. **Story 10.10 "Sửa lỗi ranh giới giữa các story của Epic 10"**: R1 (NOTICE.txt theo manifest), R2 (canh `--sizes`), R3 (checksum phủ mọi asset), A2 (`fileURLToPath`). Mỗi guard có đối chứng gỡ thật. Ice chọn Epic 10 ngày 2026-10-22; entry `id = 10` đã thêm vào `tickets.toml` của epic. Chủ: Story 10.10.
6. **Quyết định của Ice:** R4 (kết quả cũ sau khi kiểm tra lỗi) và R7 (câu "không tải thêm gì" trong ghi chú phát hành). Chủ: Ice.
7. **Đối chiếu spec**, chờ người áp dụng:
   - Sofia sửa dòng Notes của epic về nghĩa vụ thông báo tác giả (S4).
   - Winston thêm `core::update` vào dòng `reqwest` của Stack table (S6).
8. **Ghi nợ có chủ cho phần hành động 10 của Epic 12 từng giao "Epic 10"** (A-S2, A-S3, A-D4, A-D5, cổng R4). Epic 10 sắp đóng, nên chủ phải là một epic còn mở. Hoãn cùng chỗ: R5, R6, R8, R9, R10, A1. Chủ: Ice chọn chủ, Sofia ghi.
9. **Bài học quy trình** (P1, P2, lần thứ hai liên tiếp): commit ký `done` phải nêu id lượt CI xanh trên commit được ký, và mỗi story push xong mới ký. Chủ: Ice (đưa vào AGENTS.md qua `bmad-project-context` nếu muốn; không dựng cổng trên quy trình BMad).

## Acceptance verdict

**Phán quyết: accepted-with-open-items**, do Ice quyết ngày 2026-10-22, ghi đè máy chấm. Tiêu chí là tiêu chí epic tự khai.

- Máy chấm ra **rejected** vì hai lý do: Done when 1 chưa có release thật, và Done when 5 thiếu NFR19.
- Ice chấp nhận Done when 1 khi pipeline đã sẵn. Release `v0.1.0` thật vẫn là hành động 1, mục mở.
- Kiểm tra lại cho thấy NFR19 ở Done when 5 là lỗi chép của lượt di trú (S2), không phải phép đo bị bỏ.
- Mục mở: hành động 1, 2, 4 và Story 10.10. Story 10.10 thêm vào epic sau retro, nên epic chưa đóng được trước khi nó `done`.

| Done when | Kết luận | Bằng chứng |
|---|---|---|
| 1. Build công khai, phát hành trên GitHub Releases kèm SHA-256 | Ice chấp nhận pipeline; release thật còn mở | S1 |
| 2. Hướng dẫn có ảnh, xử lý Gatekeeper và SmartScreen | đạt, còn lỗ hổng có chủ | `INSTALL.md`, 16 ảnh; S5 dòng 13392/13395/13398 |
| 3. Attribution dựng từ tệp dữ liệu có mặt | đạt ở mã và test; chưa thử trên `.app` | `dict_sources.rs`, R1, S5 dòng 13408 |
| 4. Cập nhật chỉ kiểm tra và thông báo | đạt ở mã, test và e2e; chưa bấm trên app thật | đã kiểm và sạch; S5 dòng 13403 |
| 5. NFR1, NFR3, NFR4, NFR5, NFR19 đo lại, Q4 đóng | đạt: NFR3/4/5 đo, NFR4/NFR5 vượt và đã ký, NFR1 giải bằng sửa PRD, Q4 đóng; NFR19 là lỗi chép | S2, S3 |

## Open questions

- Danh sách domain "xem được trong ứng dụng" của NFR19 có phải ghi cả lời gọi kiểm tra cập nhật không? Hiện `core/update` và `commands/update.rs` không ghi gì vào domain log. NFR19 chỉ nói về đường nhập từ URL, nên đây chỉ là câu hỏi, không phải phát hiện.
- Sáu ticket còn `built` (10.1–10.5, 10.8) có được ký `done` trước khi đóng S1 và S5 không?
