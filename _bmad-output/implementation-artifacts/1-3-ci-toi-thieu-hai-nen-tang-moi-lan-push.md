---
baseline_commit: a2a5612defa545c105f41306e22357269285fdc1
---

# Story 1.3: CI tối thiểu — hai nền tảng, mỗi lần push

Status: in-progress

> **Đã qua một lượt code review 2026-08-03** (ba lớp song song, xem §Review Findings). 16 bản vá đã áp và đã nghiệm thu đỏ-rồi-xanh tại máy; 4 quyết định của Ice đã ghi. **Story vẫn `in-progress`, không lên `review`** — cùng lý do như trước lượt rà soát: AC6, AC7, Task 11 hàng 4 và AC3/Task 4 đều cần một lượt runner thật, và nay thêm hai thứ nữa chỉ runner mới trả lời được: bước `check:scope` mới của D1 có mở được webview trên `macos-26`/`windows-2025` không, và trạng thái cuối của AC8 (D2) phụ thuộc chính câu trả lời đó.

Epic: 1 — Nền móng ứng dụng & Tra cứu ngoại tuyến tức thì
Covers: NFR14 · NFR15 · **NFR6** *(nửa Windows — bàn giao từ Story 1.1, 2026-08-03)* · lưới an toàn cho AC6 của Story 1.2 · **bốn mục Deferred của code review Story 1.2** *(xem `deferred-work.md:13-14`)*

> **Story này không viết một dòng mã sản phẩm nào.** Nó dựng **một** tệp workflow và biến ba lệnh mà Story 1.2 đã để lại thành thứ chạy tự động trên hai nền tảng. Giá trị nằm ở chỗ: từ đây tới Epic 10 (chín epic, ~127 story), **mọi luật cưỡng chế bằng test đều gắn vào đúng tệp này** — AC4 cấm tường minh việc dựng pipeline thứ hai.
>
> **Phần khó nhất không phải viết YAML.** Nó là bốn cái bẫy hỏng-im-lặng ở §Bốn thứ sẽ hỏng im lặng, và **một phát hiện về NFR6 mà story này sẽ là nơi đầu tiên đo được bằng số thật** — xem §NFR6 trên Windows đã vỡ trước khi đo.

---

## Story

As a **chủ dự án**,
I want **mỗi lần push đều được build và chạy test trên cả macOS lẫn Windows**,
So that **một khác biệt nền tảng lọt vào ở Epic 2 không nằm im tới tận Epic 10 mới lộ ra**.

---

## Acceptance Criteria

### AC1 — Hai nền tảng, mỗi lần push, kết quả tách bạch

**Given** một commit bất kỳ được đẩy lên
**When** CI chạy
**Then** `cargo test` và build ứng dụng chạy trên **cả macOS lẫn Windows**
**And** kết quả hai nền tảng hiện **tách bạch**, không gộp thành một trạng thái chung

### AC2 — Trượt là đỏ, không có trạng thái xám

**Given** một test trượt, hoặc một nền tảng build hỏng
**When** CI kết thúc
**Then** trạng thái là **đỏ**
**And** commit đó không được coi là xong

### AC3 — NFR14 cưỡng chế bằng CI, không bằng trí nhớ

**Given** AC hai nền tảng của Story 1.2 (NFR14 — *hành vi tương đương trên macOS và Windows*)
**When** kiểm
**Then** nó được **cưỡng chế bằng CI**, không còn là một phép kiểm tay phải nhớ làm

> Story 1.2 đóng AC6 **không trọn** và ghi đúng như vậy: `cargo check --target x86_64-pc-windows-msvc` gãy trên máy Ice ở ba crate build native C. Trên runner Windows nó là `cargo check`/`cargo test` bình thường. AC3 chỉ được coi là đạt khi **bản Windows chạy thật** đã dựng được, không phải khi `cargo metadata` phân giải xong.

### AC4 — Một pipeline duy nhất, có sẵn chỗ móc cho epic sau

**Given** các luật cưỡng chế bằng test sinh ra ở epic sau — lint cấm màu viết thẳng (AD-34, Story 1.4), test ranh giới `ai/` (AD-13, Story 4.1), bốn test allowlist (AD-41, Epic 6)
**When** chúng tồn tại
**Then** gắn vào **chính pipeline này**
**And** **không dựng pipeline thứ hai**

### AC5 — Không phải FR107

**Given** phạm vi của story này
**When** so với FR107
**Then** đây **không phải** FR107 — không build công khai để người ngoài kiểm chứng, không checksum, không `dict-manifest.toml`
**And** FR107 vẫn đóng ở Story 10.1 với phạm vi nguyên vẹn

### AC6 — Hai phép đo `.msi`, ghi lại ở MỖI lần chạy

*(Nhận bàn giao từ Story 1.1 ngày 2026-08-03. Công thức: [`research/font-spike-results-2026-08-03.md`](../planning-artifacts/research/font-spike-results-2026-08-03.md) §Công thức đo trên Windows.)*

**Given** runner Windows của CI đã chạy được
**When** CI chạy trên một commit bất kỳ
**Then** dung lượng `.msi` **có font** và **không font** được ghi lại thành số cụ thể, và **chênh lệch** được đối chiếu với dải ước **16,0–20,3 MiB** của mũi thăm dò
**And** **chế độ cài WebView2 đang dùng được ghi kèm** — hiện là **`offlineInstaller`** *(Ice chốt 2026-08-03; `epics.md` đã cập nhật cùng ngày)*
**And** dung lượng `.msi` được ghi thành **hai dòng tách bạch** — **payload sản phẩm** (đối chiếu với trần 150–200 MB) và **WebView2 Runtime nhúng ≈ 127 MB** (ghi ra, **không** đối chiếu) — theo NFR6 đã sửa 2026-08-03; xem §NFR6 sau khi Ice nới trần
**And** hai số này ghi lại ở **mỗi lần CI chạy**, không phải một lần rồi thôi — đây là lưới bắt hồi quy khi bộ font hoặc cấu hình WebView2 đổi
**And** phép cộng với dung lượng database **không** thuộc story này — CI ở đây không tải dữ liệu từ điển; phép đối chiếu tổng với trần NFR6 đóng ở **Story 1.9**

### AC7 — Không tải dữ liệu từ điển; chạy đủ ngắn

**Given** dữ liệu từ điển 150–200 MB
**When** CI chạy ở epic này
**Then** **không tải dữ liệu từ điển** — job chỉ biên dịch và chạy các test không phụ thuộc dữ liệu
**And** thời gian chạy **đủ ngắn để không ai muốn tắt nó đi** — nghiệm thu bằng **số đo thật** (thời gian tường + phút tính phí), không bằng cảm nhận

### AC8 — Tổ hợp CSP + asset protocol được một phép kiểm chạm tới

*(Nhận bàn giao từ code review Story 1.2 — `deferred-work.md:13`. Không có trong `epics.md`; đưa vào đây theo đúng tiền lệ Story 1.2 nhận NFR13/UX-DR4.)*

**Given** `npm run check:scope` chạy qua `tauri dev`, nơi Tauri **không** áp CSP (webview nạp HTML từ Vite qua `devUrl`)
**When** ứng dụng chạy ở chế độ **không phải dev** — webview nạp HTML do chính Tauri phục vụ, nên CSP **có** hiệu lực
**Then** cả hai chiều của Kiểm 3 vẫn đúng: tài nguyên **trong** `$RESOURCE/fonts/**` nạp được, `/etc/hosts` · `C:\Windows\win.ini` bị **từ chối**
**And** kết quả này lấy được **trên cả hai nền tảng**, trong chính pipeline này
**And** ⛔ nếu không chạy được trong CI thì **ghi rõ lý do và trả lại cho Ice**, không đánh dấu đạt và không lặng lẽ bỏ

---

## Tasks / Subtasks

- [x] **Task 1 — Đọc trước khi gõ dòng đầu tiên** (AC: 1, 4, 6, 7)
  - [x] Đọc §Bốn thứ sẽ hỏng im lặng. **Ba trong bốn cái đều cho ra một lượt CI XANH với số vô nghĩa** — đó là thứ đắt nhất, đắt hơn một lượt CI đỏ.
  - [x] Đọc §NFR6 trên Windows đã vỡ trước khi đo. Story này là nơi con số đó lộ ra; **cách phản ứng đúng là báo cáo, không phải sửa cấu hình cho số đẹp lại**.
  - [x] Đọc §Ngân sách CI. Repo là **private** (API trả `Not Found` cho người không đăng nhập, 2026-08-03) ⇒ macOS tính **hệ số ×10**. Đây là ràng buộc thật lên AC7.
  - [x] ⛔ **Không** chạm vào `src/`, `src-tauri/src/`, `src-tauri/tauri.conf.json` trừ đúng hai chỗ §Ranh giới phạm vi cho phép.

- [x] **Task 2 — Một tệp workflow, và chỉ một** (AC: 1, 2, 4)
  - [x] Tạo **`.github/workflows/ci.yml`**. ⛔ **Không tạo tệp workflow thứ hai** ở story này và ghi chú tường minh trong tệp rằng AC4 cấm điều đó.
  - [x] Trigger: `on: push` (mọi nhánh) **và** `on: pull_request`. ⚠️ Nhánh mặc định của repo là **`master`**, không phải `main` — đừng viết cứng `branches: [main]`, CI sẽ không bao giờ chạy và **không lỗi nào được ném**.
  - [x] ⛔ **Không** dùng `paths`/`paths-ignore` để bỏ qua commit tài liệu. AC1 nói *"một commit bất kỳ"*; lọc đường dẫn là chỗ một thay đổi thật lọt qua vì nó nằm chung commit với một tệp `.md`.
  - [x] `concurrency: { group: ci-${{ github.ref }}, cancel-in-progress: true }` — huỷ lượt cũ khi push liên tiếp. Đây là đòn bẩy rẻ nhất cho AC7 và cho §Ngân sách CI.
  - [x] `permissions: { contents: read }` — tối thiểu. Story này không ghi gì lên repo (AC5: không phát hành).
  - [x] ⛔ **Không** dùng `tauri-apps/tauri-action`. Nó tồn tại để tạo GitHub Release — đúng thứ AC5 nói **không thuộc story này**; FR107 giữ nguyên phạm vi ở Story 10.1.

- [x] **Task 3 — Job `check`: ma trận hai nền tảng, THỨ TỰ BƯỚC là bắt buộc** (AC: 1, 2, 3)
  - [x] Một job duy nhất, `strategy.matrix.os` hai giá trị → GitHub hiện **hai check run riêng biệt**, đúng AC1 (*"tách bạch, không gộp"*).
  - [x] `fail-fast: false` — một nền tảng đỏ **không được** huỷ nền tảng kia. Gộp lại là mất đúng thứ AC1 đòi.
  - [x] Nhãn runner: **`macos-26`** và **`windows-2025`** (ghim ảnh, không dùng `-latest`). Lý do và đánh đổi ở §Runner. Ghi hai nhãn này vào Completion Notes.
  - [x] ⚠️ **THỨ TỰ BẮT BUỘC — `npm run build` phải chạy TRƯỚC `cargo test`.** `tauri::generate_context!` nhúng frontend **lúc biên dịch**; `../dist` chưa tồn tại thì `cargo test` gãy ở khâu biên dịch chứ không phải ở một assert. Bẫy #2 ở §Bốn thứ sẽ hỏng im lặng.
  - [x] Chuỗi bước: `checkout` → `setup-node` (cache npm) → `rust-toolchain` (ghim) → `rust-cache` → `npm ci` → `npm run check:deps` → `npm run build` → `cargo test --locked --manifest-path src-tauri/Cargo.toml`.
  - [x] Phiên bản action đã kiểm chứng 2026-08-03 ở §Phiên bản action. Dùng đúng bảng đó; nếu một major mới gãy thì **tụt một major và ghi lý do**, đừng thả trôi.
  - [x] `Swatinem/rust-cache@v2` — **bắt buộc**, không phải tối ưu. Không có nó thì `aws-lc-sys` + `libsqlite3-sys` (biên dịch SQLite từ nguồn C) + `zstd-sys` biên dịch lại mỗi lượt và AC7 chết ngay lượt thứ hai.
  - [x] ⚠️ `cargo test` **không** cần `--all-features`; và ⛔ **không** thêm `--release` cho bước test — profile release có `lto = true` + `codegen-units = 1`, biên dịch lâu gấp nhiều lần mà không kiểm thêm được gì.

- [ ] **Task 4 — Bản build ứng dụng thật trên hai nền tảng** (AC: 1, 3)
  - [x] macOS: `npx tauri build --bundles dmg`. Windows: `npx tauri build --bundles msi`.
  - [x] ⚠️ **Truyền `--bundles` tường minh, đừng để `tauri build` trần đọc `bundle.targets`.** `tauri.conf.json` đang khai `["dmg", "msi"]`; `tauri-cli` trên macOS **từ chối** giá trị `msi` (*"possible values: ios, app, dmg"* — đã đo ở Story 1.1). Bẫy #3 ở §Bốn thứ sẽ hỏng im lặng.
  - [x] `CI=true` cho mọi lệnh build. Trên macOS nó tránh bẫy #1 của Story 1.1 (`bundle_dmg.sh` chết ở bước AppleScript). GitHub Actions **đã đặt sẵn `CI=true`** — xác nhận lại, đừng giả định.
  - [x] Ghi **thời gian từng bước** và dung lượng artifact ra `$GITHUB_STEP_SUMMARY`. Story 1.1 đo `.dmg` **22.944.022 byte trên Intel**; runner nay là **arm64** nên số sẽ khác — ghi kèm kiến trúc để không ai đọc thành hồi quy.
  - [x] ⛔ **Không** upload artifact lên GitHub Release, **không** sinh checksum SHA-256, **không** đụng `dict-manifest.toml` (AC5). Dùng `actions/upload-artifact` để giữ `.msi`/`.dmg` cho lượt rà soát thì được — đó là artifact của lượt chạy, không phải bản phát hành.
  - [ ] `cargo check --target x86_64-pc-windows-msvc` mà Story 1.2 **không chạy được trên máy Ice** nay đóng ở đây, dưới dạng `cargo test` + `tauri build` chạy **native** trên runner Windows. `windows-2025` có sẵn VS 2022 Build Tools nên rào biên dịch C của ba crate (`zstd-sys`, `libsqlite3-sys`, `aws-lc-sys`) **không còn**. Ghi xác nhận vào Completion Notes.

- [ ] **Task 5 — Hai phép đo `.msi`, và chiều trừ đã ĐẢO** (AC: 6, 7)
  - [x] Tạo **`src-tauri/tauri.nofonts.conf.json`** chứa **đúng** nội dung:
    ```json
    { "bundle": { "resources": null } }
    ```
  - [x] 🔴 **`null`, KHÔNG phải `{}`.** §Công thức đo trên Windows của báo cáo mũi thăm dò viết `{ "bundle": { "resources": {} } }` — **và nó là một no-op**. Tauri merge cấu hình theo **JSON Merge Patch (RFC 7396)** qua `json_patch::merge`; với patch là object rỗng, hàm duyệt 0 khoá và **không đổi gì**. Chỉ `null` mới xoá khoá. Đã đọc mã: `tauri-utils-2.9.3/src/config/parse.rs:7,185` → `json-patch-3.0.1/src/lib.rs:661-681`. Dùng `{}` thì hai bản build **giống hệt nhau**, chênh lệch **bằng 0**, và **không lỗi nào được ném** — đúng thứ §Công thức đã cảnh báo là phải tránh, chỉ là nó tự vấp vào.
  - [x] Thêm một test vào `src-tauri/tests/config_invariants.rs` neo tệp này: nó phải tồn tại, phải là object có `bundle.resources` **là `null` tường minh** (`is_null()`), và **không** là object rỗng. Không có test thì một lượt "dọn dẹp" sau sẽ đổi `null` thành `{}` và phép đo im lặng trả về 0 mãi mãi.
  - [x] ⚠️ **Đặt tên tệp là `tauri.nofonts.conf.json`, không phải `tauri.windows.conf.json`.** Tauri **tự động merge** `tauri.<platform>.conf.json`, và test `no_dev_csp_and_no_platform_config_overrides` (`config_invariants.rs:178-186`) sẽ đỏ đúng chỗ — nó tồn tại chính để chặn việc này.
  - [x] ⚠️ **Chiều trừ ĐẢO so với công thức gốc.** Công thức viết cho app thăm dò (mặc định **không** font, bản B *chồng thêm*). Từ Story 1.2, font **nằm sẵn** trong `tauri.conf.json`. Nên: **bản B = có font = `tauri build` bình thường**; **bản A = không font = `--config src-tauri/tauri.nofonts.conf.json`**.
  - [x] ⚠️ **Hai lệnh ghi ra CÙNG một đường dẫn** `src-tauri/target/release/bundle/msi/*.msi`. Đọc và ghi lại số của bản thứ nhất **trước khi** chạy lệnh thứ hai.
  - [x] Chạy **cả hai bản trong cùng một job** — biên dịch Rust dùng chung, bản thứ hai chỉ tốn khâu đóng gói. Chạy hai job riêng là trả tiền biên dịch hai lần cho đúng một số đo (AC7).
  - [x] Đọc dung lượng bằng `(Get-Item "src-tauri\target\release\bundle\msi\*.msi").Length` (PowerShell) hoặc tương đương trong Node. In **byte**, không in "MB làm tròn".
  - [ ] Đối chiếu chênh lệch với dải **16,0–20,3 MiB**. **Rơi NGOÀI dải mới là phát hiện đáng ghi** — khi đó xem lại mức nén CAB mà Tauri đặt cho WiX (`MSZIP` vs `LZX`), thứ chưa xác minh được từ macOS.
  - [x] Ghi kèm: chế độ WebView2 **đang dùng thật** (đọc từ `tauri.conf.json`, đừng chép từ tài liệu), `rustc --version`, `npx tauri --version`, nhãn ảnh runner. Tất cả vào `$GITHUB_STEP_SUMMARY` để đọc được mà không phải mở log.
  - [x] ⛔ **Đừng đổi `webviewInstallMode` để số đẹp lại.** Ice đã chốt `offlineInstaller` ngày 2026-08-03 sau khi cân với lời hứa *"fully offline"*. Nhiệm vụ của story này là **đo và báo cáo**, xem Task 6.

- [ ] **Task 6 — NFR6: tách hai dòng, đối chiếu đúng một dòng** (AC: 6)
  - [x] Ghi **dung lượng tuyệt đối** của `.msi` bản có font, không chỉ chênh lệch.
  - [x] **Tách con số đó làm hai dòng** — đây là hình dạng nghiệm thu mà NFR6 sửa ngày 2026-08-03 đòi:
    | Dòng | Gồm gì | Đối chiếu trần? |
    |---|---|---|
    | **Payload sản phẩm** | mã + font + *(sau Story 1.9)* dữ liệu từ điển | **Có** — 150–200 MB |
    | **WebView2 Runtime nhúng** | phần `offlineInstaller` nhúng vào, ≈ 127 MB | **Không** — chỉ ghi ra |
  - [x] **Tách bằng phép trừ, không bằng ước lượng, nếu làm được rẻ:** một bản `.msi` thứ ba dựng với `webviewInstallMode = downloadBootstrapper` (qua `--config`, giống hệt cách Task 5 dựng bản không font) trừ khỏi bản chính cho ra **đúng** phần runtime nhúng. Biên dịch đã dùng chung, bản thứ ba chỉ tốn khâu đóng gói. Nếu không làm được thì dùng ≈ 127 MB của tài liệu Tauri và **ghi rõ đó là số mượn từ tài liệu, không phải số đo** — đúng tinh thần cột ✓/⚠️ mà Story 1.2 đưa vào bảng Stack.
  - [x] ⛔ **Không** sửa `webviewInstallMode` trong `tauri.conf.json`. Cấu hình trong repo giữ nguyên `offlineInstaller`; bản thứ ba chỉ tồn tại trong một lượt đo.
  - [ ] Kết luận NFR6 vào Completion Notes theo khuôn Story 1.1: **payload** vượt trần ⇒ **thay đổi tầng PRD cần Ice quyết**. **Runtime nhúng vượt bao nhiêu cũng KHÔNG phải vi phạm NFR6** — Ice đã đưa nó ra ngoài ngân sách ngày 2026-08-03, và `prd.md` §7.2 + `epics.md` §NFR6 đã ghi thành chữ.
  - [x] ⚠️ **Nhưng vẫn nói thẳng con số tổng.** Người dùng tải về thấy dung lượng tổng, không thấy hai dòng của ta. Nếu `.msi` tổng lớn tới mức thành rào cản tải xuống thì ghi thành mục riêng cuối Completion Notes để Ice cân ở **Story 10.2** — nơi đường quay lui còn mở: `downloadBootstrapper` (mất mệnh đề cài offline) hoặc **NSIS** thay `.msi` (chạm hàng Deferred *"chưa khai artifact phát hành chính thức cho Windows"*, `deferred-work.md:6`). ⛔ Không tự chọn.

- [ ] **Task 7 — AC8: Kiểm 3 ngoài chế độ dev, trên cả hai nền tảng** (AC: 8)
  - [x] Hiểu đúng cái đã chặn Story 1.2 trước khi gõ: móc self-check phía Rust là **`#[cfg(debug_assertions)]`** (`src-tauri/src/lib.rs:31,37,55`) nên **không tồn tại trong bản release**; và mã self-check phía frontend chỉ vào bundle khi **build** với `VITE_SCOPE_SELFTEST=1` (`src/App.vue:14`). Bẫy #4 ở §Bốn thứ sẽ hỏng im lặng.
  - [x] **Đường đi được đề xuất — `tauri build --debug`:** profile `dev` ⇒ `debug_assertions` **bật** ⇒ móc còn đó; nhưng webview nạp HTML từ **frontendDist qua asset protocol** ⇒ Tauri **có** chèn CSP. Đó đúng là tổ hợp mà `tauri dev` không bao giờ chạm tới.
    ```bash
    VITE_SCOPE_SELFTEST=1 npx tauri build --debug --bundles app   # macOS
    VITE_SCOPE_SELFTEST=1 npx tauri build --debug --no-bundle     # Windows
    ```
    rồi chạy nhị phân với `AURA_SCOPE_SELFTEST=1` và đọc dòng `VERDICT:`.
  - [x] ⚠️ **`resolveResource()` là chỗ đường này gãy nếu làm ẩu.** Chiều DƯƠNG của Kiểm 3 nạp `fonts/SourceSans3[wght].ttf` qua `resolveResource`. Với bản `.app` trên macOS, tệp nằm ở `Contents/Resources/fonts/` — có sẵn. Với `--no-bundle` trên Windows, **không có** thư mục resource nào cạnh `.exe`; phải **chép `src-tauri/resources/fonts/` sang cạnh nhị phân** trước khi chạy, đúng hình dạng mà `bundle.resources` khai (`"resources/fonts/*.otf" → "fonts/"`). Không chép thì chiều dương trả **404**, và self-check đã phân biệt được *"thiếu tệp"* với *"scope chặn"* nên nó sẽ **FAIL đúng**, không đọc nhầm thành đạt — nhưng lượt chạy vẫn vô nghĩa.
  - [x] Đọc phán quyết **từ dòng `VERDICT:`** như `scripts/check-scope.mjs` đang làm, và **timeout cứng**. Bài học Story 1.2: `tauri dev` **nuốt mã thoát**, và một phép kiểm không bao giờ trả gì thì job chạy tới hạn mức rồi bị huỷ.
  - [x] ⛔ **KHÔNG** bật `debug-assertions = true` trong `[profile.release]` để "làm cho đúng hơn". Profile release đang được **cố ý đóng băng** để số đo NFR6 của Story 1.1 còn so sánh được (`Cargo.toml:56-61`). Đổi nó là làm hỏng chính AC6 của story này.
  - [x] ⛔ **KHÔNG** gỡ `#[cfg(debug_assertions)]` khỏi móc self-check. Story 1.2 đặt nó ở đó có lý do đã ghi thành chữ: *"một móc như vậy không có việc gì trong bản phát hành"*.
  - [x] Ghi **thẳng giới hạn** vào Completion Notes: phép kiểm này chứng minh **tổ hợp CSP + asset protocol**, nó **không** chứng minh hành vi của **nhị phân profile release**. Nói nửa vời ở đây là tái lập đúng lỗi mà mục Defer của Story 1.2 tồn tại để sửa.
  - [ ] Nếu webview không mở được trên runner (không có phiên đồ hoạ, WebView2 vắng mặt, treo): **dừng, ghi lại bằng chứng, và trả lại cho Ice** theo mệnh đề cuối của AC8. Xem §Rủi ro đã biết.

- [x] **Task 8 — NFR15 và cây phụ thuộc: cưỡng chế bằng lock, chạy trên cả hai nền tảng** (AC: 1, 2)
  - [x] `npm ci` (**không** `npm install`) và `cargo …  --locked` ở mọi lệnh cargo. Đây là hình dạng cưỡng chế được của NFR15 trong CI: **không phụ thuộc nào vào được cây mà không hiện thành diff lockfile trong commit**, nên lượt rà giấy phép của Story 1.2 Task 7 luôn có chỗ bám.
  - [x] `npm run check:deps` chạy trên **cả hai** nền tảng. Script đã có **ngưỡng sàn** (Rust ≥ 200, npm ≥ 30) nên cây rỗng không đọc thành "sạch"; số thật trên Windows là **346 crate** (macOS 343) — nếu sàn chạm, đó là lỗi quét chứ không phải đạt.
  - [x] ⛔ **Không thêm `cargo-deny`, `cargo-audit`, `license-checker` hay công cụ rà giấy phép nào.** Chúng nằm ngoài bảng Stack, và NFR15 đòi **đọc tệp giấy phép trong nguồn đã tải** — đúng thứ một bộ nhận dạng tự động làm sai, đã có tiền lệ: `tantivy-stemmers` suýt bị chấm sai BSD-2 ở Story 1.2. Nếu Ice muốn một cổng tự động thì đó là quyết định riêng, không phải việc lặng lẽ thêm ở đây.
  - [x] ⛔ **Không** đưa phép quan sát mạng bằng `lsof` vào CI. Story 1.2 bàn giao **công thức đã sửa** (`pgrep -x auratranslate` + `lsof -nP -a -p …`) để nó không mất, chứ không phải để chạy mỗi push: nó cần một phiên chạy thật có tương tác, và `lsof` thiếu cờ `-a` từng cho ra **274 socket của Lark/AnyDesk/ssh**. Ghi công thức vào `.github/workflows/ci.yml` dưới dạng chú thích, kèm một dòng nói rõ vì sao nó **không** là một bước.

- [x] **Task 9 — Chỗ móc cho chín epic sau** (AC: 4)
  - [x] Trong `ci.yml`, đặt một khối chú thích **có tên** liệt kê ba luật đã biết sẽ gắn vào: lint cấm màu viết thẳng (AD-34 → Story 1.4) · test ranh giới `ai/` (AD-13 → Story 4.1) · bốn test allowlist (AD-41 → Epic 6), kèm câu *"gắn vào job này, không dựng workflow thứ hai — AC4 Story 1.3"*.
  - [x] Đặt chúng ở nơi một story sau chỉ cần **thêm một bước**, không phải sắp xếp lại job. Ba luật đó đều là *"chạy một lệnh, mã thoát khác 0 là đỏ"* — cùng hình dạng với `check:deps`.
  - [x] Ghi vào Completion Notes **tên job và vị trí bước** để bốn story kia trỏ đúng chỗ.

- [x] **Task 10 — Ranh giới FR107, viết thành chữ** (AC: 5)
  - [x] Chú thích đầu `ci.yml`: *"Đây KHÔNG phải FR107. Không build công khai kiểm chứng được, không checksum, không `dict-manifest.toml`, không GitHub Release. FR107 đóng ở Story 10.1 với phạm vi nguyên vẹn."*
  - [x] Rà lại một lượt cuối: không bước nào tải dữ liệu từ điển, không bước nào tạo release, không bước nào sinh checksum (AC5, AC7).

- [ ] **Task 11 — Nghiệm thu: đỏ thật rồi xanh thật** (AC: 2)
  - [ ] ⛔ **Một pipeline chưa từng đỏ là một pipeline chưa được nghiệm thu.** Cố ý phá rồi khôi phục, ghi kết quả từng lượt:
    | Phá cái gì | Phải đỏ ở đâu |
    |---|---|
    | Thêm một origin từ xa vào `csp` của `tauri.conf.json` | `cargo test` — các test CSP, **cả hai** nền tảng *(Story 1.2 đo: phá cả `csp` lẫn `scope` cho **4/9 FAILED**; ghi số thật của lượt này, đừng chép)* |
    | Đổi `tauri.nofonts.conf.json` từ `null` sang `{}` | test mới ở Task 5 |
    | Tạo `node_modules/@tauri-apps/plugin-fs/` | `npm run check:deps`, exit 1 |
    | Thêm một lỗi biên dịch chỉ ở nhánh Windows (`#[cfg(windows)] compile_error!`) | **chỉ** job Windows đỏ, job macOS **vẫn xanh** — đây là phép kiểm của AC1 (*"tách bạch"*) và của `fail-fast: false` |
  - [ ] Ghi số của AC7: **thời gian tường mỗi job** ở lượt cache lạnh và lượt cache nóng, **phút tính phí** ước tính theo hệ số ở §Ngân sách CI. Không ghi *"chạy nhanh"*.
  - [x] Cập nhật `deferred-work.md`: đóng mục *"Tổ hợp CSP + asset protocol của bản RELEASE"* (Task 7) và mục *"NFR6 phải đo lại"* (Task 5, 6) — hoặc ghi lại chúng với trạng thái mới nếu chưa đóng được.

---

### Review Findings

*Lượt rà soát 2026-08-03 — ba lớp song song (Blind Hunter · Edge Case Hunter · Acceptance Auditor) trên dải `847e933..HEAD` (gồm cả code Story 1.2), spec đối chiếu là chính tệp này. 4 decision-needed (Ice đã giải hết 2026-08-03) · **16 patch — đã áp hết** · 16 defer · 8 loại bỏ.*

##### Nghiệm thu bản vá — đỏ trước, xanh sau, tại máy Ice (macOS arm64, 2026-08-03)

| Phá cái gì | Kỳ vọng | Kết quả thật |
|---|---|---|
| Khai trùng `font-src`, **bản ĐẦU** mở ra `https://cdn.evil.com` | hai test mới đỏ | ✅ `csp_declares_each_directive_exactly_once` **và** `csp_allows_no_remote_origin` FAILED — 2/15. *(Trước bản vá cả hai đều XANH: `BTreeMap` giữ bản cuối, tức bản lành.)* |
| Xoá `base-uri 'self'` khỏi CSP | test mới đỏ | ✅ `csp_declares_the_directives_that_do_not_inherit_default_src` FAILED |
| `capabilities/extra.toml` *(capability hợp lệ)* | test capability đỏ | ✅ FAILED. *(Trước bản vá: XANH — bộ lọc `.ends_with(".json")` loại nó ra.)* |
| `capabilities/sub/extra.json` | test capability đỏ | ✅ FAILED. *(Trước bản vá: XANH — `read_dir` không đệ quy.)* |
| `AURA_SCOPE_TIMEOUT_MS` = `""` · `"5min"` · `"0"` · `"-5"` | rỗng → mặc định; còn lại → từ chối | ✅ `""`/`"   "`/vắng mặt → `300000`; `5min`/`0`/`-5`/`NaN` → exit 1 kèm lý do |
| `AURA_SCOPE_TIMEOUT_MS=3000` cho `check:scope` *(timer nổ khi `tauri dev` còn khởi động — ca TREO của bản cũ)* | giết cả cây, exit 1 | ✅ exit 1 ở **4s** tường; `pgrep -x auratranslate` và `pgrep -f "tauri dev"` đều **rỗng** — không tiến trình mồ côi |

Khôi phục hết, chạy lại xanh. Lượt chạy đầy đủ sau khi vá:

```bash
npm run build                 # vue-tsc ×2 + vite build          → exit 0
cargo test --locked …         # 15 test (13 cũ + 2 mới)          → exit 0
npm run check:deps            # nay có `cargo tree --locked`     → exit 0  (326 crate · 104 gói)
npm run check:scope           # chế độ dev, HAI chiều, 403 thật  → exit 0  (11s)
npm run check:scope:bundled   # mode: bundled-csp                → exit 0  (69s)
python3 -c "import yaml; yaml.safe_load(open('.github/workflows/ci.yml'))"   # parse OK
```

⚠️ **`npm run check:scope` chạy được ở máy này trong 11s và đọc đúng 403 cả hai chiều** — nên bước CI mới của D1 có cơ sở thật, không phải một phép cược. Nhưng **runner** vẫn là ẩn số: đây là macOS có phiên đồ hoạ, còn `macos-26`/`windows-2025` thì chưa ai đo. Nếu nó trượt ở đó, mệnh đề ⛔ của AC8 áp dụng — ghi lý do, trả lại cho Ice, **không** gỡ bước.

⚠️ **Ba số đo sẽ dịch so với mốc cũ**, và đó là hệ quả có chủ ý của bản vá, không phải hồi quy: `bundle.resources` nay mang thêm `license/COPYING.txt` (≈ 35 KB) nên `$fontBytes` của AC6 gồm cả nó; `check:deps` nay báo 326 crate / 104 gói *(so với 343/59 ghi trong story — cây đã đổi từ lúc dựng story, cả hai vẫn trên sàn 200/30)*.

#### Ice đã quyết — 2026-08-03, trong lượt rà soát

| # | Vấn đề | Ice quyết | Thành |
|---|---|---|---|
| D1 | `npm run check:scope` bị bỏ khỏi CI ⇒ không phép kiểm tự động nào còn chứng minh chiều ÂM | **Thêm vào CI.** Nó cần phiên đồ hoạ và có thể trượt trên runner — nếu trượt thì chính lượt chạy đầu là bằng chứng để ghi lý do theo mệnh đề ⛔ của AC8 | **patch** (mục cuối §Cần vá) |
| D2 | AC8 đang được tính là ĐẠT dù chiều âm là `unmeasured` | **Chốt sau lượt CI đầu có D1.** Nếu `check:scope` chạy được trên runner thì chiều âm có lưới tự động và AC8 đóng trọn; nếu không thì hạ xuống "đóng một nửa, đã trả lại cho Ice" | **defer** |
| D3 | `on: push` + `on: pull_request` ⇒ ma trận chạy hai lần trên nhánh có PR | **Giữ cả hai.** AC7 nghiệm thu bằng số thật — để lượt CI đầu đo đúng giá của việc nhân đôi rồi mới quyết, đúng §Ngân sách CI (*"ghi số và dừng"*) | **defer** |
| D4 | `reqwest` default features + `crate-type` thừa `staticlib`/`cdylib` — chi phí AC7 nằm trong `Cargo.toml` | **Không đổi.** §File List ⛔ cấm đụng `Cargo.toml`, và bảng Stack được cài trọn có chủ ý ở Story 1.2. Chờ số AC7 thật rồi mới bàn tối ưu | **defer** |

#### Cần vá

- [x] [Review][Patch] 🔴 **`npm` được spawn KHÔNG qua shell ⇒ job Windows chết ở bước thứ 5** [`scripts/check-deps.mjs:85`] — `execFileSync('npm', ['ls','--all','--json'], {...})` không truyền `shell`. Trên Windows `npm` là `npm.cmd`; libuv chỉ dò `.com`/`.exe` khi tìm PATH ⇒ **ENOENT**. Lỗi bị bắt ở `:91-93` (`raw = err.stdout` → rỗng → `throw`) → `abort()` → `exit 1`. Kéo theo: `cargo test`, AC8, và **toàn bộ ba phép đo `.msi` của AC6** không bao giờ chạy trên Windows — tức AC1, AC3, AC6, AC7 đều mất nửa nền tảng. Chính tác giả đã xử đúng ở hai script anh em (`check-scope.mjs:46`, `check-scope-bundled.mjs:74` đều có `shell: IS_WIN`); tệp này bị bỏ sót. *(Kèm: dòng tick `[x]` ở Task 8 khai "số thật trên Windows là **346 crate**" — chưa từng có lượt chạy Windows nào.)*
- [x] [Review][Patch] 🔴 **`BTreeMap` giữ chỉ thị CSP TRÙNG bản cuối, trình duyệt cưỡng chế bản ĐẦU** [`src-tauri/tests/config_invariants.rs:28-36`] — `csp.split(';').filter_map(...).collect::<BTreeMap<_,_>>()`: khoá trùng thì insert sau ghi đè insert trước. Theo spec CSP, trong **một** policy trình duyệt dùng lần xuất hiện **đầu tiên** và bỏ qua các lần sau. Nên `"… font-src https://cdn.evil.com; font-src 'self' asset: …"` đi qua sạch `csp_allows_no_remote_origin` (`:87`), `csp_scheme_sources_are_pinned…` (`:130`) và cả `starts_with("default-src 'self'")` — trong khi webview thật thi hành `https://cdn.evil.com`. Doc-comment `:23-27` khoe đã bịt "bốn lối lách"; đây là lối thứ năm, do chính bản viết lại tạo ra. Fix: assert không có tên chỉ thị nào lặp lại (hoặc fold theo first-wins).
- [x] [Review][Patch] 🔴 **Bước AC8 đứng TRƯỚC bước đo `.msi` và không có `always()` ⇒ AC6 mất trắng ở mọi lượt AC8 hỏng** [`.github/workflows/ci.yml:123-124` vs `:203-204`] — bước `check:scope:bundled` không có `continue-on-error`; bước đo `.msi` chỉ có `if: runner.os == 'Windows'`. Mặc định GitHub Actions là bước sau chỉ chạy khi bước trước `success()`. §Rủi ro đã biết #1 nói thẳng đường AC8 trên runner *"chưa ai trong dự án này đo"* và có thể treo — tức kịch bản dễ xảy ra nhất ở lượt chạy đầu vô hiệu hoá đúng AC mà story tồn tại để đóng, và AC6 đòi ghi số ở **MỖI** lần chạy. Fix: chuyển bước AC8 xuống **sau** hai bước đo, hoặc gắn `if: !cancelled() && runner.os == '…'` cho các bước đo.
- [x] [Review][Patch] **Test `capabilities/` chỉ lọc `.json` và KHÔNG đệ quy — Tauri nạp cả `.toml`/`.json5`, đệ quy** [`src-tauri/tests/config_invariants.rs:279-283`] — `fs::read_dir` + `.filter(|n| n.ends_with(".json"))`. Nguồn Tauri: `tauri-utils/src/acl/build.rs` khai `CAPABILITY_FILE_EXTENSIONS = ["json","json5","toml"]` và nạp bằng glob `"{capabilities}/**/*"`. Nên `capabilities/extra.toml` hay `capabilities/sub/extra.json` với `permissions = ["fs:default"]` được Tauri cấp quyền thật mà **không test nào đỏ** — đúng hình dạng hỏng im lặng mà comment `:275-277` tuyên bố đã đóng. *(Cùng test: `main_capability_grants_the_minimum…` `:239-268` chỉ đọc `windows` và `permissions`; thêm khoá `remote.urls` / `webviews` / `platforms` cũng không làm đỏ.)*
- [x] [Review][Patch] **Dò chế độ CSP bằng cửa sổ đua 100 ms — sai cả hai chiều** [`src/selftest/scopeCheck.ts:250-257`] — `cspApplies` chỉ bật khi `fetch` **throw** VÀ sự kiện `securitypolicyviolation` kịp tới trong 100 ms. (a) **Dương tính giả:** ở chế độ dev, `fetch` ném vì lý do bất kỳ cộng một violation từ nguồn khác ⇒ vào nhánh `bundled-csp` ⇒ chiều âm thành `unmeasured` ⇒ `VERDICT: PASS`, hàng rào 403 bị bỏ qua im lặng. (b) **Âm tính giả:** ở bundled, WebView2 phát violation muộn hơn 100 ms (cold start trên `windows-2025`) ⇒ rơi vào nhánh dev ⇒ `checkOutOfScopeDenied` bị CSP chặn ⇒ `'fail'` ⇒ **CI đỏ oan** với chẩn đoán sai hoàn toàn. Nhánh này mới chỉ chạy trên macOS. Fix: chờ theo sự kiện (race giữa violation và một deadline dài hơn) thay vì `setTimeout` cố định, và coi "fetch ném + không có violation" là một trạng thái lỗi tường minh, không phải "dev mode".
- [x] [Review][Patch] **`cargo tree` chạy KHÔNG `--locked`, ngay trước `cargo test --locked` trong cùng job** [`scripts/check-deps.mjs:61-65` · `ci.yml:93` vs `:107`] — `cargo tree` được phép giải lại và **ghi lại `Cargo.lock`**. Hai hệ quả: cây được quét có thể không phải cây được test; và nếu lock bị ghi lại thì `cargo test --locked` ở bước sau đỏ vì một lý do không liên quan tới commit. Comment `ci.yml:111` khẳng định *"`--locked` là nửa Rust của NFR15"* — nửa đó bị vô hiệu bởi bước đứng ngay trước.
- [x] [Review][Patch] **CSP thiếu bốn chỉ thị KHÔNG kế thừa `default-src`** [`src-tauri/tauri.conf.json` — `app.security.csp`] — chuỗi hiện tại không khai `base-uri`, `form-action`, `object-src`, `frame-ancestors`; cả bốn đều không rơi về `default-src` theo spec CSP. `base-uri` đáng lo nhất: AD-16 tồn tại vì nội dung nhập từ web là không tin được, mà một điểm chèn DOM đủ để ghi `<base href>` và trỏ lại mọi đường dẫn tương đối — một đường ra mạng nằm ngoài ba điểm của AD-15. Và `csp_allows_no_remote_origin` chỉ duyệt các chỉ thị **đang có mặt**, nên sự vắng mặt là vô hình với toàn bộ suite. ⚠️ Vá kèm: thêm `'none'` vào `ALLOWED_LOCAL_SOURCES` (`config_invariants.rs:49-55`), nếu không test sẽ đỏ.
- [x] [Review][Patch] **Khai `GPL-3.0-or-later` nhưng repo không có tệp giấy phép nào** [`package.json:7` · `src-tauri/Cargo.toml:8`] — không có `LICENSE*`/`COPYING*` ở gốc, cũng không trong `bundle.resources`. GPL-3.0 đòi văn bản giấy phép đi kèm bản phân phối. Bản build đóng gói **ba** tệp OFL của font nhưng không đóng gói giấy phép của chính sản phẩm — khó biện minh nhất trong một story mà toàn bộ NFR15 là rà giấy phép thủ công từng crate. ⚠️ Thêm vào `bundle.resources` sẽ cộng ~35 KB vào `$fontBytes` của AC6 (lớp phủ `nofonts` gỡ **mọi** resource) — ghi chú lại khi đo.
- [x] [Review][Patch] **`SIGKILL` chỉ giết wrapper `npx`/`cmd.exe`, không giết cây `tauri dev` ⇒ nhánh timeout có thể KHÔNG BAO GIỜ chạy** [`scripts/check-scope.mjs:42-47,62-65`] — `spawn('npx', ['tauri','dev'], { shell: win32 })` rồi `child.kill('SIGKILL')`. Tiến trình cháu (`cargo run` → `auratranslate` → vite) không nhận tín hiệu và vẫn giữ đầu ghi của cùng ống stdout/stderr; sự kiện `'close'` chỉ phát khi tiến trình đã thoát **và** mọi stdio đã đóng ⇒ `if (timedOut)` ở `:77` có thể không bao giờ thực thi và script treo vô hạn. Đúng chế độ hỏng mà doc-comment `:15-20` nói *"nay có TIMEOUT cứng"* để chặn — cơ chế mới chỉ chuyển chỗ hỏng. Fix: `detached: true` + `process.kill(-child.pid)` trên POSIX, `taskkill /T /F /PID` trên Windows. *(⚠️ `check-scope-bundled.mjs` KHÔNG dính lỗi này — nó `spawn(binPath)` trực tiếp, không qua wrapper.)*
- [x] [Review][Patch] **Handler `'close'` vứt bỏ mã thoát và tín hiệu của tiến trình con** [`scripts/check-scope-bundled.mjs:135`] — `child.on('close', () => {` không nhận `code`/`signal`. Self-check in `VERDICT: PASS`, Rust gọi `app.exit(0)`, nhưng tiến trình sau đó chết vì panic ở luồng khác hoặc SIGSEGV của webview lúc dọn dẹp ⇒ vẫn `process.exit(0)`. Một nhị phân crash lúc thoát được ghi nhận "ĐẠT" trên cả hai nền tảng.
- [x] [Review][Patch] **`Number(process.env.… ?? 300_000)` không lọc chuỗi rỗng / NaN** [`scripts/check-scope.mjs:35` · `scripts/check-scope-bundled.mjs:37`] — `??` chỉ bắt `undefined`/`null`. `AURA_SCOPE_TIMEOUT_MS=""` (rất thường gặp khi biến khai trong `env:` của workflow mà giá trị rỗng) ⇒ `Number('') === 0`; `=5min` ⇒ `NaN`. `setTimeout` ép cả hai về ~1 ms ⇒ SIGKILL bắn tức thì, luôn exit 1 kèm *"Hết 0s / NaNs mà self-check chưa phát VERDICT — nhiều khả năng webview không mở được"* — chẩn đoán sai vĩnh viễn.
- [x] [Review][Patch] **Nhánh `catch` của `App.vue` tự nó có thể ném, và viết cứng tên event** [`src/App.vue:21-28`] — trong `catch` lại `await import('@tauri-apps/api/event')`: nếu nguyên nhân gãy ban đầu chính là import động (bundle lỗi, CSP chặn chunk) thì lệnh này cũng reject và **không event nào được phát**; nếu `emit` là thứ reject thì `catch` gọi lại đúng `emit` đó. Không có lớp bọc thứ hai ⇒ unhandled rejection trong `onMounted` ⇒ treo tới timeout, và báo cáo ra là *"webview không mở được"* trong khi webview mở bình thường. Kèm: `:27` viết thẳng `'selftest:scope-check'` thay vì hằng (`scopeCheck.ts:66` khai `const SELFTEST_EVENT` nhưng **không export**), và payload thiếu trường `mode` mà `ScopeCheckReport` khai.
- [x] [Review][Patch] **Regex đọc tên nhị phân không neo vào section `[package]`** [`scripts/check-scope-bundled.mjs:50`] — `/^\s*name\s*=\s*"([^"]+)"/m` lấy match **đầu tiên trong tệp**. Hôm nay đúng vì `[package] name = "auratranslate"` (`Cargo.toml:2`) đứng trước `[lib] name = "auratranslate_lib"` (`:15`). Thêm một khối `[workspace]`/`[[bin]]` lên trên hay sắp xếp lại manifest là script đi tìm `auratranslate_lib.exe` rồi chết với *"Không tìm thấy nhị phân đã dựng"* — thông báo trỏ sai hoàn toàn nguyên nhân, **sau khi** đã trả trọn chi phí một lượt biên dịch debug.
- [x] [Review][Patch] **Bước macOS thiếu cổng chống ô rỗng mà bước Windows có** [`.github/workflows/ci.yml:160-161` vs `:277-278`] — Windows kiểm `if (-not $rustcV -or -not $tauriV) { throw … }`; macOS chỉ hoist `RUSTC_V=$(rustc --version)` / `TAURI_V=$(npx tauri --version)`. `set -e` bắt được **mã thoát khác 0** nhưng không bắt được lệnh exit 0 in ra chuỗi rỗng ⇒ bảng `$GITHUB_STEP_SUMMARY` của macOS có ô trống mà bước vẫn xanh. Chính "số đo biến mất mà bước vẫn xanh" mà comment `:157-159` tuyên bố đã sửa — sửa mới nửa nền tảng.
- [x] [Review][Patch] **`$payloadBytes` là một đồng nhất thức, không phải phép đo — và nó lái một cổng cứng tầng PRD** [`.github/workflows/ci.yml:249-251`] — `$runtimeBytes = $withFonts − $noRuntime` rồi `$payloadBytes = $withFonts − $runtimeBytes` rút gọn **đúng bằng** `$noRuntime.Bytes`, tức dung lượng bản `downloadBootstrapper` (vẫn còn stub bootstrapper trong đó) chứ không phải "mã + font" như nhãn ghi. Thêm nữa MSI nén theo cabinet **toàn cục**: bỏ 27 MB font không làm cabinet nhỏ đi đúng 27 MB, và bỏ runtime đổi luôn tỉ lệ nén phần còn lại — nên cả `$fontBytes` lẫn `$runtimeBytes` là hiệu số của kho nén, không phải dung lượng thành phần. Vậy mà `$payloadMB -gt 200` phát *"🔴 VƯỢT trần — cần Ice quyết"*. Fix: đổi nhãn thành *"bản không nhúng runtime (proxy cho payload)"* và ghi rõ giới hạn của phép trừ ngay trong bảng summary.

- [x] [Review][Patch] **[D1] Gắn `npm run check:scope` vào pipeline — chiều ÂM phải có lưới tự động** [`.github/workflows/ci.yml`] — Ice chốt 2026-08-03. Thêm một bước chạy `npm run check:scope` (chế độ dev, nơi `fetch` đọc được **HTTP 403** thật) vào job `check`, chạy trên cả hai nền tảng. Hôm nay chỉ có `check:scope:bundled`, mà ở chế độ bundled chiều âm là `unmeasured` (`scopeCheck.ts:260-263`) và `unmeasured` **không** làm đỏ verdict (`:271`) ⇒ mở toang `assetProtocol.scope` lúc chạy vẫn cho CI xanh ở mọi bước. Script cần **phiên đồ hoạ** (`tauri dev`) nên có thể trượt trên runner — nếu trượt, đó chính là bằng chứng để ghi lý do và trả lại cho Ice theo mệnh đề ⛔ của AC8, **không** phải lý do lặng lẽ bỏ bước. ⚠️ Vá kèm hai thứ: bước này phải chịu chung cách xếp thứ tự của patch *"AC8 đứng trước bước đo `.msi`"* (đừng để nó chặn AC6), và nó phụ thuộc patch *"SIGKILL chỉ giết wrapper `npx`"* — không sửa chỗ đó thì timeout của `check-scope.mjs` có thể không bao giờ chạy và job treo tới `timeout-minutes: 60`.

#### Hoãn — đã ghi vào `deferred-work.md`

- [x] [Review][Defer] **[D2] Trạng thái AC8 chốt sau lượt CI đầu có D1** — deferred, Ice chốt 2026-08-03: nếu `check:scope` chạy được trên runner thì chiều âm có lưới tự động và AC8 đóng trọn; nếu không thì hạ `deferred-work.md:14` xuống *"đóng một nửa, đã trả lại cho Ice"* và thêm AC8 vào danh sách "còn thiếu" thành mục thứ năm
- [x] [Review][Defer] **[D3] `on: push` + `on: pull_request` nhân đôi lượt chạy trên nhánh có PR** [`ci.yml:26-27,34`] — deferred, Ice chốt giữ cả hai: AC7 nghiệm thu bằng số thật, để lượt CI đầu đo đúng giá của việc nhân đôi (macOS ×10, repo private) rồi mới quyết — đúng §Ngân sách CI *"ghi số và dừng"*
- [x] [Review][Defer] **[D4] `reqwest` default features (kéo `aws-lc-sys`) + `crate-type` thừa `staticlib`/`cdylib`** [`src-tauri/Cargo.toml:16,52`] — deferred, Ice chốt không đổi: §File List ⛔ cấm đụng `Cargo.toml` và bảng Stack được cài trọn có chủ ý ở Story 1.2; chờ số AC7 thật rồi mới bàn tối ưu
- [x] [Review][Defer] `timeout-minutes: 60` nhiều khả năng không đủ cho nhánh Windows cache lạnh [`ci.yml:59`] — deferred, cần số đo từ lượt chạy thật
- [x] [Review][Defer] `--config` vô hiệu hoá mọi bất biến của `config_invariants.rs`, và danh sách chặn lớp phủ nền tảng chỉ liệt kê biến thể `.json` [`config_invariants.rs:166-190` vs `ci.yml:237,247`] — deferred
- [x] [Review][Defer] Cổng phụ thuộc dùng **danh sách cấm** trong khi `config_invariants.rs:92-94` lập luận danh sách cấm là sai; thiếu `tauri-plugin-shell` · `-http` · `-process` · `-opener` [`check-deps.mjs:121-142`] — deferred
- [x] [Review][Defer] `walk()` đệ quy không có bộ nhớ đã-thăm [`check-deps.mjs:95-99`] — deferred
- [x] [Review][Defer] `deferred-work.md:7` (*"đường nạp font chưa từng chạy trên Windows"*) nay đã lỗi thời và không được cập nhật; §File List ⛔ khai *"không đụng `planning-artifacts/**`"* nhưng dải commit có sửa `epics.md` (+14/−) và `prd.md` (+6/−) — deferred
- [x] [Review][Defer] Không có clippy · rustfmt · ESLint · Prettier · test runner frontend · quét CVE; `scripts/*.mjs` (chính tầng cưỡng chế) không được type-check vì `tsconfig.json` chỉ include `src/**` — deferred
- [x] [Review][Defer] `dict-manifest.toml:9-18` đặt luật "ba trường BẮT BUỘC" và cảnh báo checksum sai *"hỏng im lặng đúng kiểu tệ nhất"* nhưng không parser/test nào đọc nó — deferred, chủ sở hữu Story 1.9/10.1
- [x] [Review][Defer] Trích dẫn dòng trong comment cưỡng chế đã rữa: `check-scope-bundled.mjs:20` trỏ `Cargo.toml:56-61` cho `[profile.release]` nhưng khối đó ở `:61-66`; các trích `deferred-work.md:5,13` không phân giải được từ gốc repo — deferred
- [x] [Review][Defer] Nhánh nền tảng chỉ có `win32`/không-`win32`: Linux nhận `--bundles app` và `binPath` trỏ vào `.app` của macOS [`check-scope-bundled.mjs:60-62,80-82`]; đồng thời không bước build nào khớp một OS thứ ba trong matrix [`ci.yml:138,204`] — deferred
- [x] [Review][Defer] Header `ci.yml:15-17` khẳng định đã kiểm chứng `v7.0.1`/`v7.0.0`/`v2.9.1` nhưng mã dùng tag major trôi `@v7`/`@v7`/`@v2` [`ci.yml:62,64,75`] — deferred
- [x] [Review][Defer] `rust-version = "1.85"` không có gì kiểm (CI chỉ chạy `1.97.1`) [`Cargo.toml:7`] — deferred
- [x] [Review][Defer] `vite.config.ts:9-19` không nối `build.target`/`minify`/`sourcemap` với `TAURI_ENV_*` ⇒ bản `--debug` mà `check-scope-bundled.mjs` chẩn đoán bằng `String(err)` được build **không sourcemap** — deferred
- [x] [Review][Defer] `stdout` + `stderr` gộp vào cùng chuỗi `log` không phân tách theo dòng ⇒ một chunk stderr chen giữa có thể phá anchor `^VERDICT: …$` [`check-scope-bundled.mjs:112-123` · `check-scope.mjs:52-60`] — deferred

---

## Dev Notes

### Ranh giới phạm vi — đọc trước khi gõ dòng đầu tiên

| Story này **có** làm | Story này **KHÔNG** làm |
|---|---|
| Một tệp `.github/workflows/ci.yml` | Bất kỳ mã sản phẩm nào (`src/`, `src-tauri/src/`) |
| Chạy ba lệnh Story 1.2 để lại, trên hai nền tảng | Build công khai / checksum / Release — **đó là FR107, Story 10.1** |
| Hai phép đo `.msi` + chế độ WebView2 | Tải dữ liệu từ điển hay cộng với dung lượng database — **đó là Story 1.9** |
| `src-tauri/tauri.nofonts.conf.json` (đúng một khoá) | Sửa `tauri.conf.json` — ⛔ kể cả `webviewInstallMode` |
| Một test mới neo tệp cấu hình đó | Bộ token màu / lint màu — **đó là Story 1.4** |
| Chỗ móc **có tên** cho luật của epic sau | Cài đặt các luật đó — chúng chưa tồn tại |
| Đo NFR6 thành **hai dòng** và báo cáo cả số tổng | Đổi `webviewInstallMode` · chọn NSIS thay `.msi` — **Story 10.2** |

> **Hai ngoại lệ duy nhất chạm vào cây nguồn có sẵn:** thêm `src-tauri/tauri.nofonts.conf.json`, và thêm **một** test vào `src-tauri/tests/config_invariants.rs` để neo tệp đó. Cả hai đều phục vụ AC6 và không cái nào đổi hành vi lúc chạy.

### Trạng thái repo hiện tại — số, không phải mô tả

Đọc lúc dựng story, `HEAD = a2a5612`:

| | |
|---|---|
| Remote | `https://github.com/vannamhh/AuraTranslate.git` |
| Nhánh mặc định | **`master`** *(không phải `main`)* |
| Tình trạng repo | **private** — `api.github.com/repos/vannamhh/AuraTranslate` trả `Not Found` cho người chưa đăng nhập |
| `.github/` hiện có | **chỉ** `agents/*.agent.md` — **chưa có workflow nào** |
| `.gitignore` với `.github/` | không chặn ⇒ `ci.yml` vào git bình thường |

**Ba lệnh Story 1.2 để lại** (`§Debug Log References` của story đó — chép đúng, đừng phát minh lại):

```bash
npm run check:deps                                 # 13 phép kiểm — cây phụ thuộc, AC2/AC5 của 1.2
npm run check:scope                                # Kiểm 3, hai chiều — chạy qua `tauri dev`
cargo test --manifest-path src-tauri/Cargo.toml    # 12 test bất biến cấu hình
```

Điều kiện chạy của từng lệnh, vì chúng **không giống nhau** và đây là chỗ dễ xếp sai thứ tự:

| Lệnh | Cần gì trước | Cần cửa sổ đồ hoạ? |
|---|---|---|
| `check:deps` | `npm ci` xong; `cargo` chạy được | không |
| `cargo test` | **`dist/` phải tồn tại** — `generate_context!` nhúng frontend lúc biên dịch | không |
| `check:scope` | `dist/` + toolchain đầy đủ; nó tự chạy `npx tauri dev` | **có** |

### Bốn thứ sẽ hỏng im lặng

Ba trong bốn cái **cho ra một lượt CI XANH** với kết quả vô nghĩa. Đó là lý do chúng đứng đầu Dev Notes.

**1. 🔴 `{ "bundle": { "resources": {} } }` là một NO-OP — chênh lệch `.msi` sẽ bằng 0 và không lỗi nào được ném.**

§Công thức đo trên Windows của báo cáo mũi thăm dò viết đúng chữ đó. Đọc mã thì nó không làm gì cả:

- `tauri-utils-2.9.3/src/config/parse.rs:7` → `use json_patch::merge;`, dùng ở `:185`.
- `json-patch-3.0.1/src/lib.rs:661-681` — đây là **JSON Merge Patch, RFC 7396**: với mỗi khoá trong patch, `null` ⇒ `map.remove(key)`, ngược lại ⇒ merge đệ quy. Patch là object **rỗng** ⇒ vòng lặp chạy **0 lần** ⇒ `doc` **không đổi**.

⇒ Bản "không font" sẽ **vẫn có font**, hai số bằng nhau, chênh lệch 0 MiB, CI xanh. Dùng **`null`**. Test ở Task 5 tồn tại để khoá điều này lại.

**2. `cargo test` gãy nếu `dist/` chưa tồn tại — và gãy ở khâu biên dịch, không ở một assert.**

`tauri::generate_context!` nhúng `frontendDist: "../dist"` **lúc biên dịch**. Trên máy Ice thư mục đó đã có sẵn từ lượt build trước nên bẫy này vô hình. Trên runner sạch nó là lỗi đầu tiên. **`npm run build` trước, `cargo test` sau.**

**3. `bundle.targets` đang là `["dmg", "msi"]` — build trần trên macOS đụng `msi`.**

`tauri-cli` 2.11.4 trên macOS trả `error: invalid value 'msi' for '--bundles'` (đo ở Story 1.1, `font-spike-results-2026-08-03.md:110-113`). Truyền `--bundles` tường minh cho **cả hai** nền tảng và không phụ thuộc vào việc bundler có tự lọc theo hệ điều hành hay không — đó là hành vi chưa ai đo, và AC1 không đáng phải cược vào nó.

**4. Móc self-check chỉ tồn tại trong bản debug — chạy nó trên bản release là chờ một event không bao giờ tới.**

Hai đầu phải đối xứng, và cả hai đều có điều kiện:

| Đầu | Điều kiện | Ở đâu |
|---|---|---|
| Rust — listener + quyết mã thoát | `#[cfg(debug_assertions)]` **và** `AURA_SCOPE_SELFTEST=1` lúc chạy | `src-tauri/src/lib.rs:31,37,55` |
| Frontend — self-check | `VITE_SCOPE_SELFTEST === '1'` **lúc build** (`import()` động) | `src/App.vue:14,19` |

Thiếu một trong hai ⇒ **treo**, không phải chạy sai. `scripts/check-scope.mjs` đã có timeout cứng cho đúng ca này; bản chạy mới ở Task 7 phải có tương đương.

### NFR6 sau khi Ice nới trần — đo hai dòng, đối chiếu một dòng

**Ice quyết ngày 2026-08-03, và quyết định đã vào tài liệu:** trần 150–200 MB là trần của **payload sản phẩm** (mã + font + dữ liệu từ điển); **bản WebView2 Runtime nhúng nằm ngoài ngân sách**. Đã sửa `prd.md` §7.2 (NFR6 + giả định A2), `epics.md` §NFR6 · §bản đồ NFR · §ghi chú Epic 1 · AC6 Story 1.3 · AC Story 1.9 · AC Story 10.9.

Vì sao phát biểu như vậy chứ không đặt một trần riêng cho Windows: runtime đó là **thành phần của hệ điều hành**, nhúng vào chỉ để giữ lời hứa *cài được khi không có mạng* — cùng lời hứa NFR7 và NFR12 đang mang. Đưa nó ra ngoài ngân sách giữ được **một** con số chung cho hai nền tảng, và **dư địa ~47 MB không đổi** — nên Story 1.9 vẫn đo đúng một thứ.

Dưới đây là phép cộng cho thấy vì sao quyết định đó là cần thiết, và con số nào rơi vào dòng nào.

| Thành phần | Số | Nguồn |
|---|---|---|
| WebView2 `offlineInstaller` | **≈ 127 MB** | tài liệu Tauri v2 §Windows Installer — *"increases the installer size by around 127MB"*, tải **lúc build** |
| Bộ font | 20,3 MiB ≈ **21,29 MB** | Story 1.1, đo thật trên `.dmg` |
| Baseline ứng dụng rỗng | ≈ **1,40 MB** | Story 1.1 |
| Dữ liệu từ điển ba nguồn đầu | **130 MB** | Story 1.9 sẽ thêm |
| **Trần NFR6** | **150–200 MB** | PRD — **payload sản phẩm**, sau lượt sửa 2026-08-03 |

Chia theo hai dòng nghiệm thu:

| Dòng | Hôm nay | Sau Story 1.9 | Đối chiếu trần |
|---|---|---|---|
| **Payload sản phẩm** | ≈ 22,7 MB *(font 21,29 + baseline 1,40)* | ≈ **152,7 MB** | **có** — vẫn trong dải 150–200, dư địa ~47 MB |
| **WebView2 Runtime nhúng** | ≈ 127 MB | ≈ 127 MB | **không** |
| `.msi` tổng người dùng tải về | ≈ **150 MB** | ≈ **280 MB** | *(chỉ ghi ra)* |

⇒ Không có lượt sửa NFR6 thì `.msi` **vượt trần khi chưa có một byte từ điển nào**, và vượt **không phải vì font** — đó chính là lý do Ice nới trần theo hướng loại trừ runtime thay vì cắt tính năng.

**Chuỗi sự kiện, để không ai đọc thành lỗi của một người:** Story 1.1 cảnh báo nguyên văn rằng chế độ này *"một mình nó đủ đẩy tổng vượt trần NFR6"*; code review Story 1.2 phát hiện `downloadBootstrapper` mâu thuẫn với lời hứa *"fully offline"*; Ice chốt **ưu tiên lời hứa offline** ngày 2026-08-03 và ghi kèm *"NFR6 phải đo lại trên bản `.msi` thật"*; cùng ngày, sau khi story này chỉ ra phép cộng ở trên, Ice chốt **nới trần theo hướng loại trừ runtime**. Story 1.3 là chỗ phép đo xảy ra.

> **Hai hệ quả dễ chịu của cách phát biểu này:** chế độ WebView2 **triệt tiêu trong phép trừ** (cả hai bản build đều mang nó) nên **chênh lệch do font** vẫn đối chiếu sạch với dải 16,0–20,3 MiB; và dòng *payload sản phẩm* đo **giống hệt nhau** trên macOS lẫn Windows, nên Story 1.9 không phải nghiệm thu hai lần theo hai trần.

### Ngân sách CI — ràng buộc thật lên AC7

**Ice chốt 2026-08-03: repo giữ **private** trong suốt quá trình dựng.** Nên hệ số nhân phút GitHub Actions là ràng buộc thật, không phải giả định: **Linux ×1 · Windows ×2 · macOS ×10**. Một job macOS 20 phút tiêu **200 phút** hạn mức. Lối thoát *"mở repo public thì Actions miễn phí"* **đã bị loại** — đừng đề xuất lại; nó chỉ mở lại ở Epic 10 cùng FR107 (*build công khai kiểm chứng được*), và đó là quyết định của Ice ở thời điểm đó.

⇒ **Ba đòn bẩy dưới đây là toàn bộ ngân sách kỹ thuật của AC7.** Dùng cả ba, theo thứ tự:

1. **`concurrency` + `cancel-in-progress`** — rẻ nhất, không mất gì. Làm ở Task 2.
2. **`Swatinem/rust-cache@v2`** — bắt buộc. Cache lạnh phải biên dịch `aws-lc-sys`, `libsqlite3-sys` (SQLite từ nguồn C) và `zstd-sys`; cache nóng thì chỉ crate của chính dự án + khâu link LTO.
3. **Mọi bản `.msi` trong CÙNG một job** — bản có font, bản không font, và (nếu làm ở Task 6) bản `downloadBootstrapper`: biên dịch dùng chung, các bản sau chỉ tốn khâu đóng gói.

**Nếu sau khi đo, AC7 vẫn không đạt:** ⛔ đừng tự cắt một nền tảng, đừng tự chuyển sang `schedule`, đừng tự thêm `paths-ignore`. Ghi **số thật** vào Completion Notes — thời gian tường và phút tính phí ước tính của cả hai nền tảng, cache lạnh và cache nóng — rồi để Ice quyết. Ba lựa chọn còn lại đều là đánh đổi về phạm vi chứ không phải kỹ thuật: giảm tần suất job nặng · bỏ bản build release khỏi mỗi push · nâng hạn mức trả phí.

### Runner — vì sao ghim ảnh, và một số sẽ khác Story 1.1

Kiểm chứng trên tài liệu GitHub ngày **2026-08-03**:

| Nhãn | Là gì |
|---|---|
| `macos-latest` | **arm64 (Apple Silicon)**, 3 CPU / 7 GB |
| `macos-26` | arm64, ảnh macOS 26 — GA từ 2026-02-26 |
| `macos-15-intel` · `macos-26-intel` | Intel x64 |
| `windows-latest` | = `windows-2025`, x64, 4 CPU / 16 GB |

**Ghim `macos-26` + `windows-2025`** thay vì `-latest`: story này ghi số dung lượng **ở mỗi lần chạy** (AC6), nên ảnh runner đổi dưới chân là một hồi quy giả. Cái giá: khi ảnh về hưu phải ghim lại — rẻ, và nó hiện thành lỗi tường minh chứ không phải một con số trôi.

> ⚠️ **Số `.dmg` từ CI sẽ KHÔNG khớp Story 1.1.** Story 1.1 và 1.2 đo trên **Intel x86_64** (`.dmg` = 22.944.022 byte); runner macOS nay là **arm64**. Baseline khác kiến trúc thì khác dung lượng. **Ghi kèm kiến trúc vào mỗi số**, nếu không thì lượt rà soát sau sẽ đọc thành hồi quy. *(Tiện thể: đây cũng là lần đầu dự án có bằng chứng trên Apple Silicon — một hàng trong `deferred-work.md:5` được thu hẹp lại, dù chưa đóng vì chưa ai đo universal binary.)*

Rào biên dịch C mà Story 1.2 đâm phải **được kỳ vọng biến mất** trên `windows-2025`: ảnh Windows của GitHub đi kèm Visual Studio 2022 (gồm workload C++), nên `zstd-sys`, `libsqlite3-sys`, `aws-lc-sys` biên dịch native bình thường thay vì cross-compile. **Xác nhận ở lượt chạy đầu và ghi vào Completion Notes** — đây chính là mệnh đề mà Story 1.2 không kiểm được và bàn giao sang đây; đừng khẳng định nó từ tài liệu.

### Phiên bản action — kiểm chứng qua GitHub API ngày 2026-08-03

| Action | Dùng | Ghi chú |
|---|---|---|
| `actions/checkout` | `@v7` | latest = **v7.0.1**, 2026-07-20 |
| `actions/setup-node` | `@v7` | latest = **v7.0.0**, 2026-07-14 — major **mới ba tuần**; gãy thì tụt `@v6` và ghi lý do |
| `Swatinem/rust-cache` | `@v2` | latest = **v2.9.1**, 2026-03-12 |
| `dtolnay/rust-toolchain` | `@1.97.1` | nhánh tồn tại, đã kiểm. **Ghim đúng số máy Ice đang chạy** để hai nền tảng và máy Ice cùng một toolchain; `@stable` hôm nay là 1.100.0 và sẽ trôi |

Node cho `setup-node`: **22** (máy Ice v22.22.2; Vite 8 đòi `^20.19.0 \|\| >=22.12.0`). Dùng `cache: 'npm'`.

### Khung `ci.yml` — hình dạng, không phải bản chép

Đây là **thứ tự và ràng buộc**, không phải mã dán thẳng. Mỗi dòng `⚠️` là một chỗ đã có bằng chứng.

```yaml
# Đây KHÔNG phải FR107 — xem Task 10.
on:
  push:          # ⚠️ mọi nhánh. Nhánh mặc định là `master`, đừng viết cứng `main`.
  pull_request:

concurrency:
  group: ci-${{ github.ref }}
  cancel-in-progress: true

permissions:
  contents: read

jobs:
  check:
    strategy:
      fail-fast: false          # ⚠️ AC1 — hai nền tảng tách bạch
      matrix:
        os: [macos-26, windows-2025]
    runs-on: ${{ matrix.os }}
    steps:
      - uses: actions/checkout@v7
      - uses: actions/setup-node@v7        # node-version: 22, cache: npm
      - uses: dtolnay/rust-toolchain@1.97.1
      - uses: Swatinem/rust-cache@v2       # ⚠️ bắt buộc, không phải tối ưu (AC7)
      - run: npm ci                        # ⚠️ ci, không phải install (NFR15)
      - run: npm run check:deps
      - run: npm run build                 # ⚠️ PHẢI trước cargo test — generate_context! nhúng dist/
      - run: cargo test --locked --manifest-path src-tauri/Cargo.toml
      - # Task 7 — Kiểm 3 ngoài chế độ dev, hai nền tảng
      - # Task 4 — tauri build --bundles dmg | msi   (⚠️ --bundles tường minh)
      - # Task 5 — (chỉ Windows) bản không font + hai số + $GITHUB_STEP_SUMMARY
      #
      # ── Chỗ móc cho epic sau (AC4) — gắn vào ĐÂY, không dựng workflow thứ hai ──
      #   Story 1.4  · lint cấm màu viết thẳng trong component (AD-34)
      #   Story 4.1  · test cưỡng chế ranh giới `ai/` (AD-13)
      #   Epic 6     · bốn test allowlist mạng (AD-41)
```

**Vì sao một job làm tất cả thay vì tách nhiều job:** hai bản `.msi` phải dùng chung khâu biên dịch (§Ngân sách CI), và `cargo test` + `tauri build` cũng vậy. Tách job là trả tiền biên dịch nhiều lần. AC1 vẫn đạt vì **matrix** đã cho hai check run riêng.

### Bốn mục Deferred của Story 1.2 — cái nào đóng ở đây, cái nào không

| Mục (`deferred-work.md`) | Ở story này |
|---|---|
| `:13` Tổ hợp CSP + asset protocol bản release | **Đóng** — Task 7, kèm giới hạn ghi thẳng |
| `:14` NFR6 phải đo lại vì `offlineInstaller` | **Đóng** — Ice đã nới trần 2026-08-03 (`prd.md` §7.2, `epics.md` §NFR6); story này đo **hai dòng** ở Task 5, 6 |
| `:15` `$RESOURCE/dict/**` trong scope nhưng không trong `bundle.resources` | **Không** — chủ sở hữu là Story 1.9 / 10.1 |
| `:16` `panic = "abort"` giết đường checkpoint AD-12 | **Không** — chủ sở hữu là Story 1.7 |
| `:17` NFR16 không có cơ chế cưỡng chế | **Không** — chủ sở hữu là Story 1.5 |
| `:18` `.shell` sinh thanh cuộn | **Không** — chủ sở hữu là Story 1.4 |
| `:6` Chưa khai artifact phát hành chính thức cho Windows (`.msi` vs NSIS) | **Không đóng**, nhưng Task 6 chạm tới nó — nếu Ice chọn NSIS thì con số `.msi` của story này không áp cho thứ người dùng tải về. Ghi nhận, đừng tự quyết |

### Rủi ro đã biết — cả hai đều thuộc Task 7

1. **Runner có mở được cửa sổ webview không.** Tài liệu Tauri v2 §Tests **không** nói gì về chạy headless trên CI; nó chỉ đề cập WebDriver (và ghi rõ macOS **không có** desktop WebDriver client). Runner GitHub cho macOS và Windows đều có phiên đồ hoạ và WebView2 Runtime cài sẵn trên ảnh Windows, nên đường này **nhiều khả năng chạy** — nhưng chưa ai trong dự án này đo. Lượt chạy đầu là phép đo. Treo hay không mở được cửa sổ ⇒ **timeout cứng, exit 1, ghi lý do** — đừng để job chạy tới hạn mức.
2. **`.msi` cần tính năng VBSCRIPT của Windows.** Tài liệu Tauri v2 §Windows Installer nêu tường minh; mặc định là bật, nhưng nếu lượt chạy đầu gãy ở khâu WiX thì đây là chỗ nhìn trước tiên. *(WiX v3: §Công thức đo trên Windows của mũi thăm dò ghi *"Tauri CLI tự tải lần build đầu"*, còn tài liệu Tauri nói phải cài sẵn — **hai nguồn nói khác nhau**. Lượt chạy đầu phân xử; ghi kết quả vào Completion Notes để không ai phải tra lại.)*

### Testing standards — thừa kế nguyên từ Story 1.2

- **Mã thoát là phán quyết.** Một script in cảnh báo rồi trả 0 là một phép kiểm không cưỡng chế được gì. Story 1.2 đã đâm vào đúng chỗ này: `tauri dev` **nuốt mã thoát**.
- **Cây rỗng không phải cây sạch.** `check-deps.mjs` có ngưỡng sàn; đừng gỡ.
- **Phép kiểm phải có cả hai chiều.** Chỉ kiểm chiều từ chối thì một cấu hình chặn sạch mọi thứ vẫn "qua".
- **Nghiệm thu bằng đỏ trước, xanh sau.** Task 11 là bắt buộc, không phải nice-to-have.
- **Số, không phải tính từ.** *"Chạy nhanh"* không nghiệm thu được AC7; `14 phút / cache nóng / macOS ×10 = 140 phút tính phí` thì có.

### References

- [Source: `_bmad-output/planning-artifacts/epics.md#Story 1.3` — bảy AC nguyên văn, `:1080-1128`]
- [Source: `_bmad-output/planning-artifacts/epics.md#Epic 1 · Ghi chú cài đặt` — vì sao CI đứng ngay sau scaffold, `:812`]
- [Source: `_bmad-output/planning-artifacts/architecture/.../ARCHITECTURE-SPINE.md#Stack` — 19 hàng đã ghim, `scripts/check-deps.sh` (nay `.mjs`) *"Story 1.3 gắn script này vào CI"*, `:611`]
- [Source: `_bmad-output/planning-artifacts/architecture/.../ARCHITECTURE-SPINE.md#AD-15` — ba điểm ra mạng; không có điểm thứ tư]
- [Source: `_bmad-output/planning-artifacts/research/font-spike-results-2026-08-03.md#Công thức đo trên Windows` — `:417-451`, kèm cảnh báo chiều trừ đảo]
- [Source: `_bmad-output/planning-artifacts/research/font-spike-results-2026-08-03.md#Phép đo 2` — dải ước 16,0–20,3 MiB và giới hạn của phép tự kiểm, `:105-148`]
- [Source: `_bmad-output/implementation-artifacts/1-2-…-pham-vi-mang.md#Debug Log References` — ba lệnh, bảng đỏ/xanh, `:597-615`]
- [Source: `_bmad-output/implementation-artifacts/1-2-…-pham-vi-mang.md#AC6` — vì sao `cargo check` cho Windows gãy trên máy Ice, `:716-742`]
- [Source: `_bmad-output/implementation-artifacts/1-2-…-pham-vi-mang.md#Bàn giao tường minh` — ba việc giao cho 1.3, `:773-784`]
- [Source: `_bmad-output/implementation-artifacts/deferred-work.md` — `:6`, `:13-18`]
- [Source: `src-tauri/tauri.conf.json` — `bundle.targets`, `webviewInstallMode: offlineInstaller`, `bundle.resources`]
- [Source: `src-tauri/src/lib.rs:14-88` — móc self-check, `#[cfg(debug_assertions)]`]
- [Source: `src/App.vue:14,19` — `VITE_SCOPE_SELFTEST` gate lúc build]
- [Source: `scripts/check-scope.mjs` — timeout cứng, đọc `VERDICT:`, tự quyết mã thoát]
- [Source: `src-tauri/tests/config_invariants.rs:166-187` — test cấm `tauri.<platform>.conf.json`]
- [Web 2026-08-03] `tauri-utils-2.9.3/src/config/parse.rs:7,185` + `json-patch-3.0.1/src/lib.rs:661-681` — merge là **RFC 7396**; patch object rỗng là no-op, chỉ `null` mới xoá khoá
- [Web 2026-08-03] Tauri v2 §Windows Installer — `offlineInstaller` ≈ **+127 MB**, tải lúc build; `.msi` chỉ dựng được trên Windows; cần tính năng VBSCRIPT
- [Web 2026-08-03] GitHub Docs §GitHub-hosted runners — `macos-latest` = **arm64**; `windows-latest` = `windows-2025`
- [Web 2026-08-03] GitHub API — `actions/checkout` **v7.0.1** · `actions/setup-node` **v7.0.0** · `Swatinem/rust-cache` **v2.9.1** · `dtolnay/rust-toolchain` có nhánh **1.97.1**

---

## Ba quyết định của Ice — 2026-08-03, đã áp vào story và vào tài liệu quy hoạch

| # | Câu hỏi | Ice quyết | Áp ở đâu |
|---|---|---|---|
| 1 | `.msi` vượt trần NFR6 vì `offlineInstaller` — xử lý thế nào? | **Nới trần**: trần 150–200 MB là trần của **payload sản phẩm**; **WebView2 Runtime nhúng nằm ngoài ngân sách**, ghi thành dòng riêng trong mọi phép đo | AC6 · Task 5, 6 · §NFR6 sau khi Ice nới trần · **`prd.md` §7.2 + giả định A2** · **`epics.md`** §NFR6, bản đồ NFR, ghi chú Epic 1, AC6 Story 1.3, AC Story 1.9, AC Story 10.9 |
| 2 | Repo private hay public? | **Private** trong suốt quá trình dựng | §Ngân sách CI — lối thoát *"public thì Actions miễn phí"* **đã loại**, ba đòn bẩy kỹ thuật là toàn bộ ngân sách của AC7 |
| 3 | Có cập nhật `epics.md` không? | **Có** — khác tiền lệ Story 1.1/1.2 vì đây là thay đổi **tầng PRD**, không phải ghi nhận cấp story | `epics.md` và `prd.md` đã sửa cùng ngày; dev **không** cần sửa thêm hai tệp đó |

> **Vì sao chọn *loại trừ runtime* thay vì đặt một trần riêng cho Windows:** một trần thứ hai buộc Story 1.9 nghiệm thu hai lần theo hai con số, và dư địa ~47 MB sẽ phải tính lại cho từng nền tảng. Cách phát biểu đã chốt giữ **một** con số chung, giữ nguyên dư địa, và nói đúng bản chất — WebView2 là runtime của hệ điều hành, không phải payload của AuraTranslate.
>
> ⚠️ **Cái giá đã biết và đã chấp nhận:** `.msi` người dùng tải về vẫn ≈ 150 MB hôm nay và ≈ 280 MB sau Story 1.9. NFR6 không còn bị vi phạm **theo định nghĩa**, nhưng con số tổng vẫn là con số thật — nên Task 6 bắt buộc phải ghi nó ra, và đường quay lui (`downloadBootstrapper` · NSIS) giữ mở ở **Story 10.2**.

---

## Dev Agent Record

### Agent Model Used

Claude Opus 5 (`claude-opus-5`) — 2026-08-03.

### Debug Log References

Lệnh chạy được ở máy Ice (macOS, Intel x86_64, `rustc 1.97.1`, `tauri-cli 2.11.4`, Node v22.22.2):

```bash
npm run check:deps            # 13 phép kiểm cây phụ thuộc      → exit 0
npm run build                 # vue-tsc ×2 + vite build          → exit 0
cargo test --locked --manifest-path src-tauri/Cargo.toml   # 13 test (12 cũ + 1 mới) → exit 0
npm run check:scope           # Kiểm 3, chế độ dev, HAI chiều    → exit 0   (hồi quy Story 1.2)
npm run check:scope:bundled   # Kiểm 3, chế độ bundled, AC8      → exit 0   (MỚI)
```

Kiểm YAML/PowerShell/bash **trước khi** tốn một lượt runner:

```bash
python3 -c "import yaml; yaml.safe_load(open('.github/workflows/ci.yml'))"      # parse OK
bash -n <khối bash của bước macOS>                                              # parse OK
docker run --rm mcr.microsoft.com/powershell:lts-alpine-3.17 \
  pwsh -c '[Parser]::ParseFile(...)'                                            # parse OK, 0 lỗi
# + chạy khô cả hai bước sinh $GITHUB_STEP_SUMMARY với số giả → markdown ra đúng
# + chạy khô hai cổng hỏng-im-lặng (chênh lệch = 0) → cả hai throw, exit 1
```

### Completion Notes List

#### Quy ước đánh dấu ở story này

Ice chốt 2026-08-03: **chưa đẩy lên remote**, để CI chạy sau. Nên checkbox theo đúng một luật:

> **Tick** khi sản phẩm giao nộp của mục đó *đã tồn tại và đã kiểm chứng tới hết mức máy này cho phép*.
> **Không tick** khi sản phẩm giao nộp *chính là một con số từ runner*.

Bốn task còn mở (**4, 5, 6, 7, 11**) đều thuộc loại thứ hai. Cơ chế của chúng đã dựng xong và đã chạy khô; thứ còn thiếu là lượt chạy thật. Danh sách chính xác nằm ở `deferred-work.md` §Story 1.3.

#### Ba phát hiện đo được, không phải suy từ tài liệu

**1. 🔴 `{ "resources": {} }` là NO-OP — và nay có số đo, không chỉ có lý lẽ.**
Story đã đọc mã (`tauri-utils` → `json-patch`, RFC 7396) và kết luận đúng. Tôi kiểm chứng lại **bằng ba bản `.app` debug thật** trên macOS, vì một mệnh đề suy từ mã nguồn vẫn là một mệnh đề chưa đo:

| `--config` | `Contents/Resources/fonts/` | `.app` |
|---|---|---:|
| *(không có)* | có | 54.712 KiB |
| `{"bundle":{"resources":null}}` | **biến mất** | **28.068 KiB** |
| `{"bundle":{"resources":{}}}` | **vẫn còn** | **54.712 KiB** ← y hệt bản đầu |

Chênh lệch `null` = 26.644 KiB ≈ 26,02 MiB (`.app` không nén; dải ước 16,0–20,3 MiB là cho bản đã nén).
Đồng thời đo thêm: `--config` trỏ vào **đường dẫn không tồn tại** thì `tauri-cli` **báo lỗi ngay**, không im lặng bỏ qua — nên đó *không* phải một bẫy nữa. Đường dẫn phân giải theo **thư mục làm việc**, không theo `src-tauri/`.

**2. 🔴 `connect-src` thiếu `asset:` — `fetch()` tới asset protocol chạy ở dev, GÃY khi đóng gói.**
Đây là câu trả lời thật cho AC8, và nó lớn hơn AC8. CSP hiện là `connect-src 'self' ipc: http://ipc.localhost`; `font-src` và `img-src` **có** `asset:`, riêng `connect-src` thì không. Chạy bản `.app` debug (có CSP), bốn sự kiện `securitypolicyviolation` nêu đích danh `connect-src`, `blockedURI` là chính URL `asset://`:

```
fetch IN : THROW TypeError: Load failed      ← CSP chặn, KHÔNG phải scope chặn
fetch OUT: THROW TypeError: Load failed      ← CSP chặn, KHÔNG phải scope chặn
xhr  IN/OUT: status=0                        ← cùng chỉ thị
font IN  : LOADED                    ✅      ← `font-src asset:` cho qua
font OUT : THROW NetworkError
img  IN/OUT: onerror                         ← .ttf không phải ảnh, không nói lên gì
```

⚠️ **Phần đắt không nằm ở phép kiểm.** `$RESOURCE/dict/**` **đang nằm trong** `assetProtocol.scope` — tức là ta đã tuyên bố webview *được phép* đọc từ điển — trong khi CSP cấm `fetch` nó. **Hai khai báo đang mâu thuẫn nhau**, và story đầu tiên chạm vào sẽ chạy tốt suốt lúc phát triển rồi hỏng ở bản người dùng cài. Đã ghi thành mục riêng ở `deferred-work.md`, nối với mục `$RESOURCE/dict/**` có sẵn từ Story 1.2. Thuộc **Story 1.9 / 10.1**.

**3. `FontFace` KHÔNG phân biệt được ba ca khác hẳn nhau — nên chiều âm không đo được dưới CSP.**
Đây là lý do phương án B tồn tại, và nó là số đo chứ không phải phỏng đoán:

| Đích | Kết quả |
|---|---|
| ngoài scope, có thật (`/etc/hosts`) | `NetworkError` |
| **trong scope, có thật, không phải font** (`OFL-sourcesans3.txt`) | **`NetworkError`** |
| trong scope, không tồn tại (404) | `NetworkError` |

Ca giữa là ca giết phép kiểm: nếu `scope` mở toang thì `/etc/hosts` được phục vụ 200 rồi `FontFace` **vẫn** ném `NetworkError` vì nó không phải font — **y hệt** khi bị chặn. Một phép kiểm cho cùng kết quả dù hàng rào còn hay mất thì không kiểm gì cả. Nên chiều âm ghi `[----] unmeasured` kèm lý do, **không** ghi PASS.

#### AC8 — đóng được tới đâu, và giới hạn ghi thẳng

`npm run check:scope:bundled` (mới) dựng `tauri build --debug` rồi chạy nhị phân với timeout cứng và đọc `VERDICT:`. Profile `dev` giữ `debug_assertions` ⇒ móc self-check còn; webview nạp HTML qua asset protocol ⇒ **CSP có áp**. Đó đúng là tổ hợp `tauri dev` không bao giờ chạm tới.

- ✅ **Chứng minh:** tổ hợp CSP + asset protocol — tài nguyên trong `$RESOURCE/fonts/**` nạp được dưới CSP qua `font-src`, **đúng đường Story 1.4 sẽ dùng thật**.
- ⛔ **KHÔNG chứng minh:** hành vi của nhị phân profile **release**. Móc là `#[cfg(debug_assertions)]`, và profile release đang bị cố ý đóng băng để giữ số đo NFR6 so sánh được. Không gỡ `cfg`, không bật `debug-assertions` trong `[profile.release]` — cả hai đều làm hỏng thứ khác.
- ⛔ **KHÔNG chứng minh:** chiều âm dưới CSP (xem phát hiện 3). Chiều âm vẫn có bằng chứng **403** từ chế độ dev (Story 1.2), trên **cùng mã Rust** cưỡng chế scope — CSP chỉ chồng thêm một lớp.
- ⚠️ Mới chạy trên **macOS**. Trên Windows đường đi là `--no-bundle` + tự chép `resources/fonts/` sang cạnh nhị phân; đã cài đặt, **chưa ai đo**.

#### Nghiệm thu đỏ-rồi-xanh (Task 11) — 3/4 hàng, tại chỗ

| Phá cái gì | Kỳ vọng | Kết quả thật |
|---|---|---|
| Thêm `https://cdn.example.com` vào `img-src` của `csp` | `cargo test` đỏ | ✅ `csp_allows_no_remote_origin` FAILED — **1/13**. *(Story 1.2 đo 4/9 khi phá **cả** `csp` lẫn `scope`; lượt này chỉ phá `csp`, nên một test là đúng.)* |
| `tauri.nofonts.conf.json`: `null` → `{}` | test mới đỏ | ✅ FAILED tại `:389` — *"`bundle.resources` phải là `null` tường minh"* |
| `tauri.nofonts.conf.json`: xoá hẳn khoá `resources` | test mới đỏ | ✅ FAILED tại `:378` — *"đúng một khoá. Thấy: []"* |
| `node_modules/@tauri-apps/plugin-fs/` | `check:deps` exit 1 | ✅ exit 1, *"gói npm `@tauri-apps/plugin-fs` CÓ MẶT"* |
| Giấu `SourceSans3[wght].ttf` khỏi `resources/fonts/` | `check:scope:bundled` đỏ | ✅ exit 1, `[FAIL] in-scope qua font-src` |
| `#[cfg(windows)] compile_error!` | **chỉ** Windows đỏ, macOS xanh | ⏳ **CHƯA CHẠY** — cần runner |

Mọi lượt phá đều đã khôi phục và kiểm lại xanh.

#### Vị trí chỗ móc cho epic sau (Task 9 yêu cầu ghi lại)

- **Tệp:** `.github/workflows/ci.yml` · **job:** `check` · **bước:** khối chú thích `CHỖ MÓC CHO EPIC SAU — AC4` ở **cuối danh sách `steps`**.
- Thêm **một** bước ngay dưới khối đó là đủ; không phải sắp xếp lại job. Ba luật đã biết (AD-34 Story 1.4 · AD-13 Story 4.1 · AD-41 Epic 6) đều cùng hình dạng *"chạy một lệnh, mã thoát khác 0 là đỏ"*, y hệt bước `check:deps`.
- Chúng chạy trên **cả hai** nền tảng mà không phải khai gì thêm — đó là lý do job được viết thành **một job có matrix** thay vì hai job song song.
- **Nhãn runner đã ghim:** `macos-26` (arm64) và `windows-2025` (x64).

#### Ba quyết định của tôi, và vì sao

1. **Không `actions/upload-artifact`.** AC5 cho phép (artifact của lượt chạy ≠ bản phát hành), nhưng repo **private** và `.msi` ≈ 150 MB **mỗi lần push** — một hoá đơn lưu trữ đổi lấy tệp gần như không ai mở, trong khi AC7 đang đếm từng phút. Con số — thứ story thật sự cần — đã nằm ở `$GITHUB_STEP_SUMMARY`. Ghi thành chú thích trong `ci.yml` kèm cách bật lại.
2. **Bản `downloadBootstrapper` của Task 6 sinh ra lúc chạy, không commit.** Nó ghi vào `$RUNNER_TEMP`. Lý do: story nói bản này *"chỉ tồn tại trong một lượt đo"*, và một tệp cấu hình `downloadBootstrapper` nằm trong repo là thứ người sau nhặt nhầm thành cấu hình thật — đúng lúc Ice vừa chốt `offlineInstaller` để giữ lời hứa *fully offline*. Ngược lại `tauri.nofonts.conf.json` **phải** commit và **phải** có test neo, vì chế độ hỏng của nó (`null` → `{}`) là im lặng.
3. **`timeout-minutes: 60` ở cấp job.** Mặc định của GitHub là 360 phút; một job treo trên macOS tiêu **3.600 phút** hạn mức trước khi ai kịp nhận ra. Treo phải là một lỗi tường minh, không phải một hoá đơn. Đây là lưới an toàn thật cho AC7.

#### Hai chỗ tôi vượt ra ngoài §Ranh giới phạm vi — cả hai đều có lý do, một cái do Ice chốt

1. **`src/selftest/scopeCheck.ts` (mã sản phẩm, `src/`) — Ice chốt phương án B ngày 2026-08-03.** Bảng ranh giới cấm chạm `src/`, nhưng phương án B *định nghĩa* là "đo chiều dương qua `FontFace` dưới CSP", và điều đó không tồn tại được nếu self-check không nhận biết được chế độ. Thay đổi: thêm nhánh `bundled-csp` (phát hiện bằng `securitypolicyviolation`, không đoán từ `import.meta.env`) và trạng thái thứ ba `unmeasured`. **Nhánh dev giữ nguyên logic Story 1.2 từng dòng** — đã chạy `npm run check:scope` để xác nhận không hồi quy: vẫn PASS cả hai chiều, vẫn đọc **403**.
2. **`scripts/check-scope-bundled.mjs` (tệp mới).** Không nằm trong bảng ranh giới, nhưng cũng không phải mã sản phẩm, và có tiền lệ trực tiếp: `check-deps.mjs` và `check-scope.mjs` tồn tại vì *"`npm run` trên Windows chạy qua `cmd.exe` — không có bash"*. Task 7 cần đúng cùng thứ đó (timeout cứng, đọc `VERDICT:`, hai nền tảng). Viết inline hai lần trong YAML — một bản bash, một bản PowerShell — là nhân đôi thứ khó nhất và không chạy thử được ở máy. **Không** phải pipeline thứ hai: nó là **một bước** trong job `check`.

#### Một chỗ tôi tự làm hỏng im lặng, và đã sửa

Lượt chạy khô bước macOS lộ ra chính lỗi mà story cảnh báo, ở mã của tôi: `echo "… $(npx tauri --version) …"` — `npx` gãy, `echo` **vẫn thành công**, `set -e` không bắt được, và bảng chỉ còn một ô rỗng. **Số đo biến mất mà bước vẫn xanh.** Đã hoist cả hai lệnh phiên bản ra biến ở cả bash lẫn PowerShell, và bên PowerShell thêm `throw` khi rỗng: *"số đo không truy nguyên được thì không phải số đo"*.

#### Còn thiếu gì để đóng story — đúng bốn thứ, tất cả cần runner

1. **AC6** — ba số `.msi` và hai dòng nghiệm thu NFR6. `.msi` **chỉ dựng được trên Windows**.
2. **AC7** — thời gian tường + phút tính phí, cache lạnh và nóng, hai nền tảng. ⚠️ **Rủi ro đã biết trước:** job biên dịch Rust **hai profile** (`dev` cho AC8, `release` cho AC1/AC6) và dựng **ba** bản `.msi`. Story giả định *"bản thứ hai chỉ tốn khâu đóng gói"* — điều đó đúng với **bundler**, nhưng `tauri-build` nhúng config vào nhị phân lúc biên dịch, nên đổi `--config` **có thể** kích hoạt một lượt link lại, mà profile release có `lto = true` + `codegen-units = 1`. Nếu AC7 trượt, đây là chỗ nhìn trước tiên. ⛔ Theo §Ngân sách CI: **ghi số và dừng**, không tự cắt nền tảng, không tự thêm `paths-ignore`.
3. **Task 11 hàng 4** — `#[cfg(windows)] compile_error!` phải làm **chỉ** job Windows đỏ, macOS **vẫn xanh**. Đây là phép kiểm thật của `fail-fast: false` và của AC1 *"tách bạch"*.
4. **AC3 / Task 4** — mệnh đề *rào biên dịch C biến mất trên `windows-2025`* mới là kỳ vọng đọc từ tài liệu. Cùng chỗ: **WiX v3** — mũi thăm dò nói *"Tauri CLI tự tải lần build đầu"*, tài liệu Tauri nói phải cài sẵn; **hai nguồn nói khác nhau**, lượt chạy đầu phân xử. Và `.msi` cần tính năng **VBSCRIPT** của Windows — nếu gãy ở khâu WiX thì nhìn chỗ này trước.

Chạy được ngay khi có `gh`: `git push` → CI tự chạy (`on: push`, mọi nhánh) → đọc `$GITHUB_STEP_SUMMARY`.

### File List

**Mới**

| Tệp | Là gì |
|---|---|
| `.github/workflows/ci.yml` | Toàn bộ pipeline. Một tệp, một job có matrix hai nền tảng (AC4) |
| `src-tauri/tauri.nofonts.conf.json` | Lớp phủ đo bộ font — `{ "bundle": { "resources": null } }` |
| `scripts/check-scope-bundled.mjs` | Harness AC8: dựng `tauri build --debug`, chạy, timeout cứng, đọc `VERDICT:` |
| `COPYING` · `src-tauri/resources/license/COPYING.txt` | *(lượt rà soát)* Văn bản GPL-3.0 lấy từ `gnu.org`. Cả hai manifest khai `GPL-3.0-or-later` nhưng repo không có tệp giấy phép nào, trong khi bản build đóng gói **ba** tệp OFL của font. Bản trong `resources/` vào `bundle.resources` để đi kèm bản phân phối |
| `src/selftest/eventName.ts` | *(lượt rà soát)* Hằng tên event, tách riêng để `App.vue` dùng được ở nhánh `catch` mà **không** phải import tĩnh `scopeCheck.ts` — import tĩnh sẽ kéo mã self-check vào bundle release |

**Sửa**

| Tệp | Thay đổi |
|---|---|
| `src-tauri/tests/config_invariants.rs` | +1 test `nofonts_overlay_drops_resources_with_an_explicit_null` (12 → **13** test) |
| `src/selftest/scopeCheck.ts` | Nhận biết chế độ CSP; thêm trạng thái `unmeasured`; **nhánh dev giữ nguyên hành vi Story 1.2** |
| `package.json` | +script `check:scope:bundled` |
| `_bmad-output/implementation-artifacts/deferred-work.md` | Đóng mục `:13`; thêm mục `connect-src`; thêm mục bốn phép nghiệm thu chờ runner |
| `_bmad-output/implementation-artifacts/sprint-status.yaml` | `1-3-…` → `in-progress` |
| `_bmad-output/implementation-artifacts/1-3-…-moi-lan-push.md` | Chính tệp này |
| `scripts/check-deps.mjs` | *(lượt rà soát)* `shell: IS_WIN` cho `npm ls` — thiếu nó là job Windows chết ở bước 5; `cargo tree --locked` |
| `src-tauri/tests/config_invariants.rs` | *(lượt rà soát)* `csp_directives` giữ bản ĐẦU + `csp_directive_counts`; test `capabilities/` đệ quy và mọi phần mở rộng; +2 test CSP (13 → **15**) |
| `src-tauri/tauri.conf.json` | *(lượt rà soát)* CSP thêm `base-uri`·`form-action`·`object-src`·`frame-ancestors`; `bundle.resources` thêm `license/` |
| `src/selftest/scopeCheck.ts` | *(lượt rà soát)* Dò CSP theo **sự kiện `connect-src`** thay vì `setTimeout(100)`; thêm chế độ `undetermined` (luôn FAIL); dùng hằng từ `eventName.ts` |
| `src/App.vue` | *(lượt rà soát)* Bọc `catch` lần hai; dùng hằng tên event; payload thêm `mode` |
| `scripts/check-scope.mjs` · `check-scope-bundled.mjs` | *(lượt rà soát)* `readTimeoutMs` từ chối `""`/NaN; `killTree()` giết cả cây + lưới an toàn 5s; kiểm `code`/`signal`; đọc `[package] name` đúng section |

⛔ **Không** đụng tới: `src-tauri/src/**` · `src-tauri/Cargo.toml` · `_bmad-output/planning-artifacts/**`.

> ⚠️ **Bảng trên đã sửa hai chỗ khai sai mà lượt rà soát bắt được.** (1) `src-tauri/tauri.conf.json` **có** bị sửa (CSP + `bundle.resources`) nên nó rời khỏi danh sách ⛔ — thay đổi là hệ quả của hai patch đã được duyệt, không phải một lượt sửa lén. (2) Dòng ⛔ cũ khai *"không đụng `_bmad-output/planning-artifacts/**`"* là **sai sự thật** ngay từ trước lượt rà soát: dải commit của story có sửa `epics.md` (+14/−) và `prd.md` (+6/−) theo đúng quyết định #3 của Ice. Đã ghi vào `deferred-work.md` §lượt rà soát.

## Change Log

| Ngày | Thay đổi |
|---|---|
| 2026-08-03 | Story dựng bằng `bmad-create-story`. Phân tích `epics.md` §Story 1.3 + §Epic 1 Ghi chú cài đặt · `ARCHITECTURE-SPINE.md` (Stack, AD-15, AD-23, Consistency Conventions) · Story 1.2 trọn vẹn (Review Findings, Completion Notes, bàn giao) · `deferred-work.md` · `font-spike-results-2026-08-03.md` §Phép đo 2 + §Công thức đo trên Windows · trạng thái repo thật (`tauri.conf.json`, `Cargo.toml`, hai script, `config_invariants.rs`, `lib.rs`, `App.vue`). **Ba phát hiện mà tài liệu nguồn chưa có:** (1) công thức `--config {"bundle":{"resources":{}}}` của báo cáo mũi thăm dò là **no-op** theo RFC 7396 — đọc mã `tauri-utils` → `json-patch` để xác nhận, phải dùng `null`; (2) `cargo test` gãy nếu `dist/` chưa tồn tại vì `generate_context!` nhúng frontend lúc biên dịch; (3) `offlineInstaller` **+127 MB tải lúc build** ⇒ `.msi` vượt trần NFR6 **trước khi** có một byte từ điển nào — story này là nơi con số đó lộ ra. Kiểm chứng phiên bản action và nhãn runner qua GitHub API/Docs ngày 2026-08-03 (`macos-latest` nay là arm64 ⇒ số `.dmg` không so được với Story 1.1) |
| 2026-08-03 | **Cài đặt.** Dựng `.github/workflows/ci.yml` (một tệp, một job matrix `macos-26` + `windows-2025`, `fail-fast: false`, `concurrency`, `timeout-minutes: 60`), `src-tauri/tauri.nofonts.conf.json`, `scripts/check-scope-bundled.mjs`, +1 test neo lớp phủ (12 → 13 test). **Ba phát hiện đo được:** (1) mệnh đề `{}` là no-op nay có **số đo** — ba bản `.app` debug cho 54.712 / **28.068** / 54.712 KiB, xác nhận chỉ `null` mới xoá khoá; (2) 🔴 **`connect-src` thiếu `asset:`** ⇒ `fetch()` tới asset protocol chạy ở dev, **gãy khi đóng gói** — và `$RESOURCE/dict/**` đang nằm trong `assetProtocol.scope` nên **hai khai báo đang mâu thuẫn**, một lỗi chờ sẵn cho Story 1.9/10.1; (3) `FontFace` trả **cùng** `NetworkError` cho "403 scope chặn", "tệp không phải font" và "404" ⇒ chiều âm **không đo được** dưới CSP, ghi `unmeasured` thay vì đoán. Ice chốt **phương án B**: giữ nguyên CSP. Nghiệm thu đỏ-rồi-xanh **5/6** hàng tại chỗ; hàng còn lại (`#[cfg(windows)] compile_error!`) cần runner. Kiểm YAML + PowerShell (qua container) + bash **trước** khi tốn lượt runner; chạy khô cả hai bước sinh `$GITHUB_STEP_SUMMARY` và cả hai cổng hỏng-im-lặng. Tự phát hiện và sửa một chỗ hỏng-im-lặng trong chính mã mình: `$(…)` gọi thẳng trong `echo` làm số đo biến mất mà bước vẫn xanh |
| 2026-08-03 | **Ice chốt: chưa đẩy lên remote, để CI chạy sau.** Story giữ `in-progress`, KHÔNG lên `review`. Bốn thứ chỉ runner mới trả lời được vẫn để **chưa tick**: AC6 (ba số `.msi` — chỉ dựng được trên Windows) · AC7 (thời gian tường + phút tính phí, cache lạnh/nóng) · Task 11 hàng 4 (chỉ Windows đỏ, macOS vẫn xanh) · AC3/Task 4 (rào biên dịch C và WiX v3 trên `windows-2025` — hai nguồn tài liệu nói khác nhau). Đã ghi cả bốn vào `deferred-work.md` §Story 1.3 |
| 2026-08-03 | **Ice quyết ba điểm, và lần này quyết định chạm tới tầng PRD.** (1) **Nới trần NFR6 theo hướng loại trừ**: trần 150–200 MB là trần của **payload sản phẩm**, bản WebView2 Runtime nhúng (`offlineInstaller`, ≈ 127 MB) **nằm ngoài ngân sách** và ghi thành dòng riêng trong mọi phép đo — giữ được **một** con số chung cho hai nền tảng và giữ nguyên dư địa ~47 MB, thay vì đặt trần thứ hai buộc Story 1.9 nghiệm thu hai lần. AC6 và Task 6 viết lại quanh **hai dòng nghiệm thu**; Task 6 nay còn một việc mới: tách phần runtime **bằng phép trừ** một bản `--config downloadBootstrapper` chứ không mượn số của tài liệu, nếu làm được rẻ. (2) **Repo giữ private** — §Ngân sách CI loại hẳn lối thoát *"public thì Actions miễn phí"* và ghi rõ ba đòn bẩy kỹ thuật là toàn bộ ngân sách của AC7; nếu vẫn không đạt thì dev **ghi số và dừng**, không tự cắt nền tảng. (3) **Có cập nhật tài liệu quy hoạch** — khác tiền lệ Story 1.1/1.2 vì đây là thay đổi tầng PRD: đã sửa `prd.md` §7.2 (NFR6 + giả định A2) và `epics.md` ở sáu chỗ (§NFR6 · bản đồ NFR · ghi chú Epic 1 · AC6 Story 1.3 · AC Story 1.9 · AC Story 10.9). Mục *Câu hỏi cho Ice* thay bằng bảng ba quyết định |
