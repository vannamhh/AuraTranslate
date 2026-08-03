---
baseline_commit: 754f0f9a1a4f1da5b297cdbfa20bc9596a304139
---

# Story 1.2: Scaffold dự án và khoá phạm vi filesystem, phạm vi mạng

Status: ready-for-dev

Epic: 1 — Nền móng ứng dụng & Tra cứu ngoại tuyến tức thì
Covers: FR104 · NFR12 · NFR14 · **NFR13** *(nghiệm thu bằng vắng mặt — xem `implementation-readiness-report-2026-08-03.md:631`)* · **UX-DR4** *(Ice gộp vào đây 2026-08-03 — trước đó không nằm trong Covers của story nào)*

> **Đây là commit mã nguồn đầu tiên của dự án.** Trước story này repo chỉ có tài liệu. Mọi quy ước dựng ở đây sẽ được chín epic sau chép lại — sai một quy ước ở đây là sửa ở 300 chỗ về sau.
>
> **Phần cứng nhất của story không phải "dựng được app", mà là ba hàng rào:** phạm vi filesystem khai tĩnh (AD-23), CSP không nới (AD-15), và cây phụ thuộc sạch (FR104/NFR12/NFR13). Ba thứ này rẻ khi làm ở commit đầu và rất đắt khi vá sau.

## Story

As a **người dựng**,
I want **một khung dự án dựng theo đúng cây nguồn đã chốt với phạm vi filesystem và CSP khoá từ commit đầu tiên**,
So that ***"không ai đọc được tài liệu của bạn"* là ràng buộc do framework cưỡng chế chứ không phải một lời hứa**.

## Acceptance Criteria

### AC1 — Cây nguồn đúng Structural Seed, không starter template

**Given** cây nguồn ở Structural Seed
**When** scaffold hoàn tất
**Then** tồn tại `src-tauri/src/{commands,core/{segment,matching,glossary,tm,dict,library,export,webimport,ai,scope,store},ports}`, `src-tauri/capabilities/`, `src-tauri/resources/dict/`, `src/{modes,panels,layout,commands,tokens,i18n}`, `tools/dict-build/`, `dict-manifest.toml`
**And** không dùng bất kỳ starter template cộng đồng nào

### AC2 — Phiên bản khớp bảng Stack; ba phụ thuộc đã loại vắng mặt

**Given** bảng Stack ghim phiên bản
**When** cài đặt phụ thuộc
**Then** phiên bản khớp đúng bảng
**And** `tauri-plugin-stronghold`, `tauri-plugin-keyring`, `tauri-wire` không có mặt trong cây phụ thuộc

### AC3 — Phạm vi filesystem tĩnh, cưỡng chế bởi Tauri

**Given** capabilities khai tĩnh
**When** ứng dụng chạy
**Then** chỉ `$RESOURCE/dict/**` và `$RESOURCE/fonts/**` đọc được, `$APPDATA/**` đọc và ghi được
**And** một thử nghiệm đọc file ngoài scope bị Tauri từ chối

### AC4 — CSP chặn mọi origin từ xa

**Given** CSP mặc định của Tauri
**When** frontend nạp
**Then** mọi origin từ xa bị chặn — không CDN, không font ngoài, không ảnh ngoài
**And** không có mã nào nới CSP

### AC5 — Không một lời gọi ra ngoài nào

**Given** ứng dụng chạy trọn một phiên làm việc
**When** quan sát lưu lượng mạng
**Then** không có lời gọi ra ngoài nào (FR104, NFR12)
**And** không có crash reporter hay thư viện analytics trong cây phụ thuộc

### AC6 — Hai nền tảng, hành vi tương đương

**Given** cùng một commit
**When** build trên macOS và trên Windows
**Then** cả hai ra bản chạy được với hành vi tương đương (NFR14)

---

## Tasks / Subtasks

- [ ] **Task 1 — Dựng khung Tauri v2 bằng tay, không qua `create-tauri-app`** (AC: 1)
  - [ ] ⛔ **Không chạy `create-tauri-app`, không chạy `npm create tauri-app`, không copy app thăm dò của Story 1.1.** AC1 mang mệnh đề nguyên văn *"không dùng bất kỳ starter template cộng đồng nào"*, và §Ranh giới phạm vi của Story 1.1 đã cấm tường minh việc dùng app thăm dò làm scaffold.
  - [ ] Tạo `package.json` ở gốc repo (npm — máy đã có npm 10.9.7, Node v22.22.2, đủ cho Vite 8).
  - [ ] Tạo `src-tauri/Cargo.toml`, `src-tauri/build.rs`, `src-tauri/tauri.conf.json`.
  - [ ] **Bố cục crate: `src/lib.rs` (crate root, giữ `pub fn run()`) + `src/main.rs` (chỉ gọi `run()`).** Đây là quy ước của **chính framework Tauri v2**, không phải starter template cộng đồng — và nó là điều kiện để `tests/` có thể `use` được mã sản phẩm (Task 8 cần).
  - [ ] Khai `[lib] name = "auratranslate_lib"`, `crate-type = ["staticlib", "cdylib", "rlib"]`. ⚠️ **Khai `[lib]` mà không có `src/lib.rs` là bẫy #2 của Story 1.1** — `cargo metadata` gãy trước cả khi biên dịch. Tạo tệp trước, khai sau.
  - [ ] Cửa sổ duy nhất mang label `main` (AD-24 — một cửa sổ OS, ba chế độ). Label này bị `capabilities` tham chiếu ở Task 4.
  - [ ] `productName` = `AuraTranslate`. Tên này quyết định tên tiến trình — công thức quan sát mạng ở §Nghiệm thu AC5 `pgrep` theo đúng chuỗi này.
  - [ ] ⛔ **Không** để `identifier` mặc định kiểu `com.tauri.dev` — Tauri từ chối build. Dùng `com.auratranslate.desktop`. ⚠️ **Đừng kết thúc identifier bằng `.app`** — nó đụng phần mở rộng bundle của macOS.
  - [ ] Khối `build` của `tauri.conf.json`: `beforeDevCommand: "npm run dev"` · `beforeBuildCommand: "npm run build"` · `devUrl: "http://localhost:1420"` · `frontendDist: "../dist"`. Bốn trường này sai một cái là `tauri dev` treo hoặc `tauri build` đóng gói một thư mục rỗng — **và bản rỗng vẫn build thành công**, hỏng im lặng.
  - [ ] `vite.config.ts` cần bốn thiết lập cho Tauri: `server.port = 1420` + `server.strictPort = true` (nếu Vite tự nhảy cổng thì `devUrl` trỏ sai) · `server.host` để trống cho desktop · `clearScreen: false` (giữ lại lỗi Rust trên terminal) · `envPrefix: ['VITE_', 'TAURI_']`.

- [ ] **Task 2 — Dựng đúng cây nguồn Rust, mỗi thư mục là module thật** (AC: 1)
  - [ ] `src-tauri/src/commands/mod.rs` · `src-tauri/src/ports/mod.rs` · `src-tauri/src/core/mod.rs`.
  - [ ] `src-tauri/src/core/<x>/mod.rs` cho **đúng mười hai** module: `segment` `matching` `glossary` `tm` `dict` `library` `export` `webimport` `ai` `scope` `store` **`i18n`**.
  - [ ] **Dùng `mod.rs` rỗng, KHÔNG dùng `.gitkeep`.** Lý do: `mod.rs` khai trong `lib.rs` thì trình biên dịch đi qua chúng — cây nguồn trở thành thứ `cargo check` xác nhận, không phải thứ mắt người phải soát. `.gitkeep` không cưỡng chế được gì.
  - [ ] Mỗi `mod.rs` mang **một dòng doc-comment** ghi module sở hữu khái niệm gì + AD ràng buộc nó (chép từ bảng Cây nguồn ở §Cây nguồn phải dựng). Đây là chỗ rẻ nhất để giữ ranh giới khỏi trôi.
  - [ ] `ports/mod.rs` ghi chú **đúng ba cổng** `DictionarySource` · `TranslationProvider` · `ProjectStore` và mệnh đề *"cổng thứ tư phải là một AD mới"* (AD-2). Chưa khai trait nào ở story này.
  - [ ] `core/ai/mod.rs` ghi chú AD-13: *không module nào ngoài `ai/` được import `ai/`*; test cưỡng chế thuộc Story 4.1.
  - [ ] `core/i18n/mod.rs` — **có tạo** *(Ice chốt 2026-08-03: theo bảng Consistency Conventions)*. Doc-comment phải nói rõ nó **không** chứa văn bản hiển thị: xem §`core/i18n/` là gì.

- [ ] **Task 3 — Dựng cây nguồn frontend và các tệp gốc còn lại** (AC: 1)
  - [ ] `src/{modes,panels,layout,commands,tokens}` — mỗi thư mục một `.gitkeep` kèm `README.md` một dòng ghi **story nào sở hữu** nó (`modes`/`panels`/`layout` → 1.14 · `commands` → 1.6 · `tokens` → 1.4).
  - [ ] `src/i18n/vi.json` — tệp thật với nội dung `{}`, không phải `.gitkeep`. Story 1.5 sở hữu nội dung; sự tồn tại của tệp là AC của story này.
  - [ ] `src/main.ts`, `src/App.vue`, `index.html`, `vite.config.ts`, `tsconfig.json`, `tsconfig.node.json`, `env.d.ts`.
  - [ ] `src-tauri/resources/dict/.gitkeep` + `README.md` ghi: *file `.db` không nằm trong git, tải theo `dict-manifest.toml` (AD-25); Story 1.9 và 10.1 sở hữu.*
  - [ ] `src-tauri/resources/fonts/` — **thư mục này không có trong danh sách AC1 nhưng AC3 đòi scope `$RESOURCE/fonts/**`**, và AD-23 khai nó tường minh. Tạo thư mục; nội dung đặt ở Task 9.
  - [ ] `tools/dict-build/` + `README.md` ghi: *parser sống ở đây, không vào bản phát hành (AD-25); Story 1.9 dựng nội dung.* **Chưa dựng thành crate Rust** — chưa có `Cargo.toml`, chưa vào workspace.
  - [ ] `dict-manifest.toml` ở **gốc repo** — khung rỗng có chú thích mô tả ba trường bắt buộc mỗi tệp: `url`, `sha256`, `phiên bản nguồn thô` (AD-25). Không điền giá trị giả.
  - [ ] Bổ sung `.gitignore`: `/src-tauri/target/`, `/src-tauri/gen/schemas/`, `/dist/`, `*.tsbuildinfo`. ⚠️ Dòng `*.db` **đã có sẵn** và đúng ý — nó giữ file từ điển ra ngoài git theo AD-25; **đừng gỡ nó**.
  - [ ] ℹ️ `src-tauri/gen/schemas/` do `tauri-build` sinh ra ở lần build đầu. Trước lần build đầu, dòng `"$schema": "../gen/schemas/desktop-schema.json"` trong capabilities **chưa phân giải được** — đó là chuyện của editor tooling, **không phải lỗi**. Đừng gỡ dòng `$schema` để "sửa" nó.

- [ ] **Task 4 — Phạm vi filesystem tĩnh theo AD-23, KHÔNG qua plugin `fs`** (AC: 3)
  - [ ] ⛔ **Không cài `tauri-plugin-fs`, `tauri-plugin-sql`, `tauri-plugin-dialog`, `tauri-plugin-store`.** Lý do đầy đủ ở §Vì sao không có plugin `fs` — đọc trước khi làm task này.
  - [ ] Tạo `src-tauri/capabilities/main.json` khai `"windows": ["main"]`, permissions **tối thiểu**: `core:default` (cần cho `resolveResource`/`convertFileSrc`) và không gì khác. Không thêm quyền plugin nào.
  - [ ] Khai phạm vi tĩnh trong `app.security.assetProtocol.scope` của `tauri.conf.json` — **đúng hai mục, không hơn**: `$RESOURCE/dict/**` và `$RESOURCE/fonts/**`, cả hai **chỉ đọc theo bản chất giao thức**. Khung JSON ở §Phạm vi tĩnh.
  - [ ] ⛔ **Không đưa `$APPDATA` vào `assetProtocol.scope`.** Frontend không có việc gì với `global.db` hay `library-index.db` (AD-1, AD-11). Nửa `$APPDATA/**` của AD-23 là phạm vi của **mã Rust**, xem §Phạm vi tĩnh.
  - [ ] Bật feature `protocol-asset` trong `Cargo.toml` — **bắt buộc** khi bật `assetProtocol`; `tauri-build` báo lỗi nếu thiếu (bẫy #4 của Story 1.1).
  - [ ] Phía Rust: mọi đường dẫn `$APPDATA` lấy qua `app.path().app_data_dir()`, **không viết cứng** `~/Library/Application Support/…`. Đường dẫn viết cứng là chỗ NFR14 (hành vi tương đương hai nền tảng) hỏng đầu tiên.

- [ ] **Task 5 — CSP và ba hàng rào mạng** (AC: 4, 5)
  - [ ] Chép nguyên văn khối `security` mà Story 1.1 đã kiểm chứng trên bản build release (§Cấu hình Tauri đã kiểm chứng của báo cáo mũi thăm dò). Khung ở §CSP dưới.
  - [ ] Khai CSP **tường minh trong `tauri.conf.json`**, không để `null`. `csp: null` là **tắt CSP**, không phải "dùng mặc định" — đây là chỗ AC4 hỏng im lặng dễ nhất.
  - [ ] Ghi một chú thích ngay cạnh khối `security` nêu: *`font-src asset:` và `img-src asset:` KHÔNG phải nới CSP theo nghĩa AD-15 cấm — AD-15 cấm origin **từ xa**; asset protocol là tài nguyên cục bộ đã nằm trong bản cài.* Chú thích này tồn tại để một giai đoạn sau không gỡ nhầm.
  - [ ] Thử hạ `style-src` xuống `'self'` trước; chỉ giữ `'unsafe-inline'` nếu bản build **release** thật sự cần, và **ghi lý do vào Completion Notes**. Xem §Một quyết định phải cân, không được chép máy móc.
  - [ ] ⛔ Không thêm bất kỳ `devCsp` nào nới ra ngoài `'self'` + kênh HMR cục bộ.
  - [ ] ⛔ Không thêm `http`/`https` client nào ở story này. Ba điểm ra mạng của AD-15 thuộc Story 4.x, 6.7, 10.7 — **không có điểm thứ tư**.

- [ ] **Task 6 — Cài TRỌN bảng Stack, ghim chính xác** *(Ice chốt 2026-08-03)* (AC: 2)
  - [ ] Cài **toàn bộ** các hàng của bảng Stack ngay ở commit này, không đợi story cần tới. Bảng đầy đủ kèm bẫy kênh phát hành ở §Bảng ghim phiên bản.
  - [ ] Ghim **chính xác**, không dùng dải rộng (`^`, `~`, `*`). Cả mười hai hàng đều tồn tại đúng số đã ghim — xác minh lại crates.io/npm ngày 2026-08-03.
  - [ ] ⚠️ **`typescript` phải ghim `5.9.3`.** `npm i -D typescript` hôm nay kéo về **7.0.2** — bảng Stack ghi *TypeScript 5.x*, nên cài mặc định là **vi phạm AC2** ngay ở lệnh đầu tiên.
  - [ ] ⚠️ **`@tauri-apps/cli` ghim `2.11.4`, không phải `2.11.5`.** Crate `tauri` và CLI npm đánh số riêng; `2.11.5` **không tồn tại** trên npm (bẫy #3 của Story 1.1, xác minh lại 2026-08-03: `dist-tags.latest = 2.11.4`).
  - [ ] ⚠️ **`similar` / `dissimilar` là hàng DUY NHẤT không cài được** — xem §Một hàng của bảng Stack chưa cài được. Ghi tường minh vào Completion Notes, đừng bỏ im lặng.
  - [ ] ⚠️ **`rusqlite` feature `bundled` biên dịch SQLite từ nguồn C** — lần build đầu chậm hơn hẳn (vài phút). Đó là bình thường, không phải treo. `libsqlite3-sys 0.38.1` là phụ thuộc bắc cầu của `rusqlite 0.40.1`; khai tường minh để lock ghim đúng số bảng Stack.
  - [ ] Crate cài mà chưa dùng **không** sinh cảnh báo của `cargo` — nên đừng chờ trình biên dịch nhắc. Ghi vào doc-comment của module sở hữu (`core/dict/` cho `jieba-rs`, `core/store/` cho `rusqlite`…) rằng crate nào dành cho nó, để story sau không cài trùng bằng tên khác.
  - [ ] Commit **cả** `Cargo.lock` **và** `package-lock.json`. Không có lock thì "ghim phiên bản" chỉ đúng trên máy người dựng đầu tiên, và AC6 (*cùng một commit → hai nền tảng*) mất nghĩa.
  - [ ] Ghi vào Completion Notes **phiên bản đã giải quyết thật** của từng phụ thuộc (đọc từ lock), không chép lại con số trong Dev Notes. `reqwest` đặc biệt cần ghi — bảng Stack chỉ ghi *"mới nhất lúc dựng"*, nên **số thật phải quay ngược vào bảng**.

- [ ] **Task 7 — Rà giấy phép và cập nhật bảng Stack** (AC: 2)
  - [ ] NFR15: **mỗi** phụ thuộc phải rà tương thích GPL v3 **trước khi** thêm, và ghi vào bảng Stack (Consistency Conventions).
  - [ ] Rà **bằng cách đọc tệp `LICENSE` trong nguồn đã tải** (`~/.cargo/registry/src/…`, `node_modules/…`), đúng tiền lệ Story 1.1 — nhãn của registry là dẫn xuất, không phải nguồn sự thật. Story 1.1 đã bắt được `source-han-serif` bị GitHub gắn `NOASSERTION` trong khi văn bản nói rõ OFL 1.1.
  - [ ] Mười hai hàng đã có sẵn cột giấy phép (kiểm chứng 2026-08-02) → việc ở đây là **xác minh lại bằng tệp**, không phải tra lại từ đầu. Đánh dấu hàng nào đã tự tay mở tệp mà đọc.
  - [ ] Thêm hàng **mới** vào bảng Stack cho các phụ thuộc chưa có: `tauri-build` · `serde` · `serde_json` · `@vitejs/plugin-vue` · `vue-tsc` · `@tauri-apps/api` · `@tauri-apps/cli` — theo đúng khuôn ba cột `Name` · `Version` · `Giấy phép`.
  - [ ] Điền **số thật** của `reqwest` vào bảng, thay chuỗi *"mới nhất lúc dựng"*.
  - [ ] Thêm **`tauri-plugin-fs`** vào danh sách *"Không dùng, đã loại có lý do"* của bảng Stack, kèm lý do một dòng (AD-1 + AD-29 — plugin tồn tại để phơi API ra JS; webview mỏng không có việc gì với filesystem). Ice chốt 2026-08-03. Xem §Vì sao không có plugin `fs`.
  - [ ] Thêm một dòng `(decision)` vào `.memlog.md` của architecture ghi quyết định này — đúng khuôn dòng `:48` đã ghi cho `tauri-plugin-keyring`.

- [ ] **Task 8 — Ba phép kiểm cưỡng chế, chạy được bằng lệnh** (AC: 2, 3, 5)
  - [ ] **Kiểm 1 — ba phụ thuộc đã loại vắng mặt (AC2).** `cargo tree -i <tên>` phải **không tìm thấy** cho `tauri-plugin-stronghold`, `tauri-plugin-keyring`, `tauri-wire`. Viết thành script chạy được, đừng làm bằng mắt.
  - [ ] **Kiểm 2 — không crash reporter, không analytics (AC5).** Quét **cả hai** cây: `cargo tree` và `npm ls --all`. Danh sách từ khoá và cách quét ở §Nghiệm thu AC5.
  - [ ] **Kiểm 3 — đọc ngoài scope bị từ chối (AC3).** Từ frontend, `convertFileSrc()` một đường dẫn ngoài `assetProtocol.scope` (`/etc/hosts` trên macOS, `C:\Windows\win.ini` trên Windows) rồi `fetch` nó → phải **thất bại**. Kèm một lượt nạp **trong** scope thành công (`resolveResource('fonts/SourceSans3[wght].ttf')` → `convertFileSrc` → `FontFace.load()`) — nếu không, một cấu hình chặn sạch mọi thứ vẫn "qua" phép kiểm.
  - [ ] Đặt ba phép kiểm ở chỗ **Story 1.3 gắn thẳng vào pipeline được** (AC của 1.3: *"gắn vào chính pipeline này, không dựng pipeline thứ hai"*). Gợi ý: `scripts/check-deps.sh` (hoặc `.mjs`) cho Kiểm 1+2; Kiểm 3 là test frontend.
  - [ ] ⚠️ **Kiểm 1 phải mở rộng thêm một dòng:** `tauri-plugin-fs` cũng phải **không có mặt** trong cây phụ thuộc (Ice chốt 2026-08-03). AC2 chỉ liệt kê ba tên, nhưng quyết định này thuộc cùng một hạng — và nếu không có phép kiểm, một story sau sẽ cài nó vào để "cho tiện" mà không ai biết.
  - [ ] Ghi vào Completion Notes **lệnh chính xác** để chạy lại cả ba — Story 1.3 sẽ chép chúng vào workflow.

- [ ] **Task 9 — Đặt bốn tệp font vào repo, đóng UX-DR4** *(Ice chốt 2026-08-03)* (AC: 3)
  - [ ] Đặt **bốn tệp** đã đo ở Story 1.1 vào `src-tauri/resources/fonts/`: `NotoSerifCJKtc-Regular.otf` · `SourceSerif4[opsz,wght].ttf` · `SourceSerif4-Italic[opsz,wght].ttf` · `SourceSans3[wght].ttf`.
  - [ ] **Đối chiếu SHA-256 từng tệp** với bảng ở `font-spike-results-2026-08-03.md §Phép đo 5` trước khi commit. Đây là chỗ duy nhất bắt được việc lấy nhầm `NotoSerifTC` (bản subset theo ngôn ngữ, 45 MB) thay `NotoSerifCJKtc` (biến thể vùng đầy đủ) — nhầm này **hỏng im lặng**: phần lớn ký tự vẫn hiện, chỉ tofu khi gặp văn bản khác hệ chữ.
  - [ ] Tổng bốn tệp phải ra **27.253.184 byte** (25,991 MiB). Lệch là lấy sai tệp.
  - [ ] Đặt kèm **ba tệp `LICENSE` gốc** (OFL 1.1 của `noto-cjk`, `sourceserif4`, `sourcesans3`) — điều kiện của FR38 và FR109, và Story 1.1 đã khuyến nghị mang theo để bịt luôn câu hỏi ở Story 10.4/10.5.
  - [ ] Khai `bundle.resources` trỏ `resources/fonts/` (và **chưa** khai cho `resources/dict/` — thư mục đó còn rỗng; xem bẫy dưới).
  - [ ] ⚠️ **Bẫy đóng gói:** `bundle.resources` trỏ vào glob **không khớp tệp nào** có thể làm `tauri build` gãy. Chỉ khai cho thư mục đã có tệp thật.
  - [ ] ⚠️ **`.gitignore` hiện có dòng `*.tmp` và `*.db` nhưng KHÔNG chặn `.otf`/`.ttf`** — kiểm lại `git status` sau khi thêm để chắc bốn tệp thật sự vào staging, đừng giả định.
  - [ ] Ghi vào Completion Notes rằng **UX-DR4 đóng ở story này** (nó không nằm trong Covers của story nào — Ice quyết gộp vào đây 2026-08-03), và cập nhật `epics.md` §UX Design Requirements nếu Ice chỉ đạo tường minh. ⚠️ Sửa `epics.md` là **ngoài phạm vi mặc định của `dev-story`** — hỏi trước.
  - [ ] ℹ️ Story này **không** dựng `@font-face` hay token typography — đó là Story 1.4. Ở đây chỉ cần một lượt nạp thử để Kiểm 3 chạy được.

- [ ] **Task 10 — Nghiệm thu chạy thật và quan sát mạng** (AC: 5, 6)
  - [ ] `npm run tauri dev` → cửa sổ mở, không lỗi console.
  - [ ] `npm run tauri build --bundles dmg` → ra `.dmg` chạy được. ⚠️ Đặt `CI=true` nếu `bundle_dmg.sh` chết ở bước AppleScript (bẫy #1 của Story 1.1).
  - [ ] **Quan sát mạng trọn một phiên** theo công thức ở §Nghiệm thu AC5 — mở app, dùng thử, đóng app; ghi lại kết quả quan sát thành số, không ghi *"không thấy gì"*.
  - [ ] Ghi **phiên bản toolchain** (Rust, Node, npm, `@tauri-apps/cli`, hệ điều hành) vào Completion Notes — cùng tiền lệ Giai đoạn 0 và Story 1.1, để số đo lặp lại được.

- [ ] **Task 11 — AC6: làm được phần nào ở đây, bàn giao phần nào cho 1.3** (AC: 6)
  - [ ] Chạy `cargo check --target x86_64-pc-windows-msvc` (target đã cài sẵn trên máy — xác minh 2026-08-03). Đây là bằng chứng **tầng biên dịch**: không có mã phụ thuộc nền tảng lọt vào.
  - [ ] ⚠️ **Không cố dựng `.msi` trên macOS.** Story 1.1 đã đâm vào đúng rào này: `tauri-cli` từ chối target `msi` vì WiX v3 là chương trình Windows. Rào ở **tầng đóng gói**, không ở tầng biên dịch.
  - [ ] **Bàn giao tường minh sang Story 1.3** — bản build Windows thật và phép so hành vi. `epics.md` Story 1.3 đã mang sẵn AC nhận bàn giao này (*"AC hai nền tảng của Story 1.2 … được cưỡng chế bằng CI"*). Ghi vào Completion Notes rằng AC6 **đóng một nửa ở đây, nửa còn lại ở 1.3** — đúng khuôn Story 1.1 đã bàn giao phép đo `.msi`. ⛔ **Không đánh dấu AC6 là đạt trọn nếu chưa có bản Windows chạy thật.**

---

## Dev Notes

### Ranh giới phạm vi — đọc trước khi gõ dòng đầu tiên

| Story này **có** làm | Story này **KHÔNG** làm |
|---|---|
| Dựng cây nguồn rỗng đúng hình dạng đã chốt | Cài đặt bất kỳ quy tắc nghiệp vụ nào |
| Khai capabilities, CSP, khoá phạm vi | Dựng CI — **đó là Story 1.3** |
| Ghim phiên bản, rà giấy phép phụ thuộc mới | Bộ token màu/chữ — **đó là Story 1.4** |
| Ba phép kiểm cưỡng chế chạy được bằng lệnh | Tài nguyên chuỗi `vi.json` có nội dung — **đó là Story 1.5** |
| Một cửa sổ `main` mở được, trống | `CommandRegistry` — **đó là Story 1.6** |
| Khung `dict-manifest.toml` rỗng có chú thích | Bốn panel, dockview — **đó là Story 1.14** |
| Cài **trọn** bảng Stack, ghim chính xác | Tầng ghi dữ liệu, `store::Writer` — **đó là Story 1.7** |
| Đặt bốn tệp font vào repo (UX-DR4) | Dùng bất kỳ crate nào vừa cài — chúng nằm đó chờ story sở hữu |

> **Cài trọn bảng Stack là quyết định của Ice ngày 2026-08-03.** Hệ quả cần biết trước: mười hai crate/gói được ghim và biên dịch ở commit này, phần lớn **chưa có mã nào gọi tới**. Đó là chủ ý — bảng Stack trở thành thứ **lock file xác nhận**, không phải một danh sách trong tài liệu mà mỗi story diễn giải lại. Cái giá là thời gian build đầu (`rusqlite` feature `bundled` biên dịch SQLite từ nguồn C) và một lần rà giấy phép dài hơn ở Task 7.

### Trạng thái repo hiện tại

Repo **chưa có một dòng mã nguồn nào**. Chỉ có `_bmad-output/` (tài liệu quy hoạch), `_bmad/` · `.claude/` · `.agent/` · `.agents/` (cấu hình công cụ, **đã gitignore**), `.github/agents/` (chỉ định nghĩa agent — **chưa có workflow CI nào**), `design-artifacts/`, `docs/` (rỗng).

Năm commit gần nhất **đều là commit tài liệu**. Không có mã cũ để giữ tương thích, cũng không có gì để tái dùng.

`.gitignore` hiện có đã bao `node_modules/`, `dist/`, `/target/`, `*.db`, `*.sqlite`. **Dòng `*.db` là cố ý và đúng** — nó giữ file từ điển ra ngoài git theo AD-25. Cần bổ sung `/src-tauri/target/` (dòng `/target/` hiện tại chỉ khớp gốc repo).

### Cây nguồn phải dựng — kèm AD ràng buộc từng module

Chép doc-comment từ cột phải vào `mod.rs` tương ứng.

```text
AuraTranslate/
  package.json · package-lock.json                    ← Task 6
  dict-manifest.toml                                  ← AD-25, khung rỗng có chú thích
  index.html · vite.config.ts · tsconfig.json
  src/                                                ← Vue 3 — CHỈ render + state UI (AD-1)
    modes/       .gitkeep + README                    Library · Workspace · ReadingMode · ReviewMode (AD-24) → Story 1.14
    panels/      .gitkeep + README                    Source · Lookup · AiTranslation · Editor        → Story 1.14
    layout/      .gitkeep + README                    dockview: dock/undock/tab/preset (FR17, FR18)   → Story 1.14
    commands/    .gitkeep + README                    CommandRegistry — MỌI thao tác đăng ký (AD-34)  → Story 1.6
    tokens/      .gitkeep + README                    token màu đã kiểm tương phản WCAG AA (AD-34)    → Story 1.4
    i18n/vi.json  = {}                                toàn bộ chuỗi giao diện (NFR16, AD-21)          → Story 1.5
  src-tauri/
    Cargo.toml · Cargo.lock · build.rs · tauri.conf.json
    capabilities/main.json                            scope tĩnh (AD-23)                              ← Task 4
    resources/dict/  .gitkeep + README                dict-core.db + mỗi lớp gỡ rời một .db (AD-10); tệp KHÔNG vào git (AD-25)
    resources/fonts/  4 tệp font + 3 LICENSE          $RESOURCE/fonts/** chỉ đọc (AD-23, UX-DR4)      ← Task 9
    src/
      main.rs                                         chỉ gọi lib::run()
      lib.rs                                          khai mod commands; mod core; mod ports;
      commands/mod.rs                                 bề mặt IPC — adapter, KHÔNG chứa quy tắc nghiệp vụ
      ports/mod.rs                                    ĐÚNG BA cổng: DictionarySource · TranslationProvider · ProjectStore (AD-2)
      core/mod.rs
      core/segment/mod.rs                             tách · gộp · tách đôi · về hưu (AD-3, AD-4, AD-5); pipeline nhập (AD-39)
      core/matching/mod.rs                            jieba + stemmer — DÙNG CHUNG cho FR40/FR51/FR61 (AD-17)
      core/glossary/mod.rs                            + bảng chờ ứng viên tách riêng (AD-20, AD-36)
      core/tm/mod.rs                                  khoá theo CẶP VĂN BẢN, không theo segment.id (AD-6)
      core/dict/mod.rs                                ba nhánh truy vấn (AD-26); KHÔNG hợp nhất nguồn (AD-19)
      core/library/mod.rs                             chỉ mục dẫn xuất + quét lại (AD-8)
      core/export/mod.rs                              docx/md/TMX + alignment + khối ghi nguồn (AD-38, AD-43)
      core/webimport/mod.rs                           Fetcher = ĐIỂM RA MẠNG THỨ BA, không phân tích nội dung;
                                                      Extractor không bao giờ chạm mạng (AD-40, AD-41)
      core/ai/mod.rs                                  C6, C7 — KHÔNG module nào khác được import (AD-13)
      core/scope/mod.rs                               ScopeResolver — phân giải hai tầng (AD-18)
      core/store/mod.rs                               Writer nối tiếp + Reader pool + checkpoint (AD-11, AD-12)
      core/i18n/mod.rs                                DANH MỤC message_key mà Rust được phép phát ra (AD-21).
                                                      KHÔNG chứa văn bản hiển thị — văn bản sống ở src/i18n/vi.json
  tools/dict-build/  README                           parser sống ở đây, KHÔNG vào bản phát hành (AD-25) → Story 1.9
```

> ⚠️ **Hai thư mục tên `commands/`, hai thứ hoàn toàn khác nhau.** Đây là chỗ dễ nhầm nhất trong cả cây nguồn, và nhầm ở story scaffold thì chín epic sau chép theo:
>
> | Đường dẫn | Là gì | AD | Story sở hữu |
> |---|---|---|---|
> | `src-tauri/src/commands/` | **Bề mặt IPC** — các hàm `#[tauri::command]` mà frontend gọi qua. Adapter thuần, không chứa quy tắc nghiệp vụ | AD-1 | rải theo từng epic |
> | `src/commands/` | **`CommandRegistry` của giao diện** — mọi thao tác người dùng đăng ký ở đây rồi mới bind vào chuột/phím. Command id dùng khoá chấm có tiền tố miền (`lookup.search_selection`) | AD-34, FR22 | **Story 1.6** |
>
> Hai thứ này **không** ánh xạ một-một và **không** được gộp. Ghi phân biệt này vào `README.md` của `src/commands/` và vào doc-comment của `src-tauri/src/commands/mod.rs`.

**Vì sao `mod.rs` chứ không `.gitkeep` cho phía Rust:** git không theo dõi thư mục rỗng, nên bằng cách nào đó vẫn phải đặt một tệp vào. Chọn `mod.rs` thì cây nguồn được **trình biên dịch xác nhận** mỗi lần `cargo check` — một thư mục bị đổi tên hay xoá nhầm ở Epic 5 sẽ **gãy build**, chứ không nằm im. Đó chính là hình dạng mà AD-13, AD-34, AD-21 đều đang dùng: *thứ cần giữ đúng thì phải để máy cưỡng chế, không để kỷ luật*.

### `core/i18n/` là gì — và không phải là gì

Ice chốt 2026-08-03: **theo bảng Consistency Conventions**, tạo `core/i18n/`. Bảng đó liệt kê mười hai module Rust; `epics.md` AC1 và Cây nguồn ở Structural Seed chỉ nêu mười một — bảng quy ước thắng.

**Nó KHÔNG chứa văn bản hiển thị.** AD-21 vẫn nguyên: *"Rust không bao giờ trả về văn bản hiển thị"*, *"không có chuỗi tiếng Việt nào trong mã Rust hay mã Vue"*. Văn bản sống ở `src/i18n/vi.json` và chỉ ở đó.

**Nó chứa danh mục `message_key` mà Rust được phép phát ra.** Hình dạng lỗi qua IPC là `{ code, message_key, params, retryable }` — `message_key` là một chuỗi khoá chấm (`lookup.empty_result`). Không có danh mục tập trung thì mỗi module tự gõ khoá của mình, và một khoá gõ sai chỉ lộ ra khi người dùng gặp đúng lỗi đó: frontend không phân giải được, hiện ra khoá trần hoặc chuỗi rỗng. **Đây đúng loại hỏng im lặng mà cả AD-21 lẫn AD-34 tồn tại để chặn** — cùng hình dạng với command id của `CommandRegistry`.

Story này **chỉ tạo `mod.rs` rỗng kèm doc-comment nêu đúng hai điều trên**. Hình dạng thật của danh mục (enum? hằng? sinh mã từ `vi.json`?) là quyết định của **Story 1.5** — chủ sở hữu NFR16 và AD-21. Ghi một dòng vào Completion Notes bàn giao sang đó.

### Bảng ghim phiên bản — cài trọn, kèm bẫy kênh phát hành

Kiểm chứng lại trên crates.io và npm ngày **2026-08-03**. **Cả mười hai hàng của bảng Stack đều tồn tại đúng số đã ghim** — không hàng nào phải đổi.

**Rust — `src-tauri/Cargo.toml`:**

| Crate | Ghim | Ghi chú |
|---|---|---|
| `tauri` | **2.11.5** | features `["protocol-asset"]` — bắt buộc khi bật `assetProtocol` |
| `tauri-build` | 2 | build-dependency |
| `rusqlite` | **0.40.1** | feature `bundled` — ⚠️ biên dịch SQLite từ nguồn C, build đầu chậm |
| `libsqlite3-sys` | **0.38.1** | phụ thuộc bắc cầu của `rusqlite`; khai tường minh để lock ghim đúng số bảng |
| `jieba-rs` | **0.10.3** | dành cho `core/matching/` (AD-17) |
| `tantivy-stemmers` | **0.4.0** | dành cho `core/matching/` — nhánh tiếng Anh |
| `docx-rs` | **0.4.22** | dành cho `core/export/` (AD-38) |
| `keyring` | **4.1.6** | **trực tiếp**, KHÔNG qua `tauri-plugin-keyring` (AD-29) |
| `reqwest` | *mới nhất* → **0.13.4** | bảng Stack ghi *"mới nhất lúc dựng"*; số thật hôm nay là 0.13.4 — **ghi ngược vào bảng** (Task 7) |
| `serde` / `serde_json` | mới nhất | phụ thuộc mới, chưa có trong bảng — thêm hàng ở Task 7 |
| `similar` **hoặc** `dissimilar` | ⛔ **không cài** | xem §Một hàng của bảng Stack chưa cài được |

**Frontend — `package.json`:**

| Gói | Ghim | Ghi chú |
|---|---|---|
| `vue` | **3.5.40** | latest hôm nay = đúng bảng Stack ✓ |
| `vite` | **8.2.0** | latest ✓. `engines.node = ^20.19.0 \|\| >=22.12.0`; máy có **v22.22.2** ✓ |
| `typescript` | **5.9.3** | ⚠️ **latest = 7.0.2.** Bảng Stack ghi *5.x* → phải ghim tường minh; `npm i -D typescript` là vi phạm AC2 |
| `dockview-vue` | **7.0.4** | latest ✓, peer `vue ^3.4.0` ✓. Cài ở đây, **dùng** ở Story 1.14 |
| `@tauri-apps/cli` | **2.11.4** | ⚠️ **2.11.5 KHÔNG tồn tại trên npm.** `dist-tags.latest = 2.11.4`. Crate và CLI đánh số riêng |
| `@tauri-apps/api` | 2.11.1 | latest hôm nay |
| `@vitejs/plugin-vue` | 6.0.8 | peer `vite: ^5 \|\| ^6 \|\| ^7 \|\| ^8` ✓ · `engines.node` giống Vite ✓ |
| `vue-tsc` | 3.3.9 | peer `typescript >= 5.0.0` ✓ hợp với ghim 5.9.3 |
| ~~`@tauri-apps/plugin-fs`~~ | ⛔ **không cài** | §Vì sao không có plugin `fs` |

**Toolchain trên máy, xác minh 2026-08-03:** `rustc` 1.97.1 (edition 2024 cần ≥ 1.85 ✓) · `cargo` 1.97.1 · Node v22.22.2 · npm 10.9.7 · target đã cài: `x86_64-apple-darwin`, **`x86_64-pc-windows-msvc`** (Task 11 dùng).

> **Bài học đắt nhất của bảng này:** hai hàng có bẫy (`@tauri-apps/cli`, `typescript`) đều là bẫy *"lệnh cài mặc định làm sai AC"*. Cả hai **không ném lỗi** — chúng chỉ cài một phiên bản khác. Ghim tường minh rồi **đọc lại từ lock** mà xác nhận.

#### Một hàng của bảng Stack chưa cài được — nói thẳng, không bỏ im lặng

Hàng `similar` **hoặc** `dissimilar` (cho Diff Viewer, FR93) **không cài ở story này**, và đây là ngoại lệ duy nhất của chỉ đạo *"cài trọn bảng Stack"*.

**Lý do:** bảng Deferred của `ARCHITECTURE-SPINE.md` để hàng này **mở có chủ ý** — *"cả hai tương thích GPLv3; đánh đổi (diff cấp grapheme vs semantic cleanup) chỉ phân xử được bằng dữ liệu thật"*, điều kiện mở lại là **Giai đoạn 5, thử cả hai trên bản review thật**. Cài một trong hai hôm nay là **âm thầm đóng một quyết định kiến trúc đang mở**; cài cả hai là ghim một crate chắc chắn sẽ bị gỡ.

**Cách xử lý:** ghi một dòng chú thích trong `Cargo.toml` ngay chỗ lẽ ra là hàng đó — *"`similar` vs `dissimilar`: hàng Deferred, chốt ở Giai đoạn 5 (Story 8.1)"* — và ghi vào Completion Notes. Số đã kiểm nếu Story 8.1 cần: `similar` 3.1.1 · `dissimilar` 1.0.11 (2026-08-03).

### Vì sao không có plugin `fs` — Ice chốt 2026-08-03

Báo cáo technical research (2026-08-02) từng giả định bộ plugin `sql` · `keyring` · `fs` · `dialog`. **Kiến trúc sau đó đã bác từng cái một**, và lý do luôn là cùng một câu:

| Plugin research đề xuất | Kiến trúc chốt | Ở đâu |
|---|---|---|
| `tauri-plugin-keyring` | ❌ — dùng crate `keyring` **trực tiếp trong Rust** | AD-29, `.memlog.md:48`, Stack §"Không dùng" |
| `tauri-plugin-sql` | ❌ — dùng `rusqlite` trực tiếp, writer nối tiếp trong Rust | AD-11, bảng Stack |
| `tauri-plugin-fs` | ❌ — **cùng lý do**, xem dưới | AD-1 + AD-29 |
| `tauri-plugin-stronghold` | ❌ — đã khai tử | Stack §"Không dùng" |

**Câu lý do, nguyên văn AD-29:** *"một Tauri plugin tồn tại để **phơi API ra JavaScript**, đúng thứ NFR11 cấm."*

Áp cho `fs`: plugin `fs` tồn tại để phơi **API hệ thống file** ra JavaScript. Nhưng AD-1 đã nói frontend **chỉ render và giữ state UI** — nó không có việc gì với hệ thống file. Cài plugin `fs` rồi thu hẹp scope là **tự tạo một bề mặt tấn công rồi rào lại**; không cài nó là **không có bề mặt để rào**.

> `tauri-plugin-fs` **không** nằm trong danh sách *"Không dùng, đã loại có lý do"* của bảng Stack — danh sách đó chỉ nêu ba cái đã bị bác tường minh. Nhưng nguyên tắc ở AD-1 + AD-29 phủ nó, và Ice xác nhận 2026-08-03. **Task 7 phải thêm nó vào danh sách đó** để lần sau không ai phải suy luận lại.

### Phạm vi tĩnh — AD-23 chia làm hai nửa, hai cơ chế khác nhau

AD-23 khai ba vùng. Chúng **không** cùng một cơ chế cưỡng chế, và trộn lẫn hai thứ này là chỗ dễ tin nhầm vào một hàng rào không tồn tại.

| Vùng AD-23 | Ai chạm tới | Cưỡng chế bằng | Nghiệm thu AC3 |
|---|---|---|---|
| `$RESOURCE/dict/**` chỉ đọc | Rust (mở `.db`) — và frontend **không** cần | `assetProtocol.scope` | ✅ test được từ frontend |
| `$RESOURCE/fonts/**` chỉ đọc | **frontend** — nạp `@font-face` qua asset protocol | `assetProtocol.scope` | ✅ test được từ frontend |
| `$APPDATA/**` đọc + ghi | **chỉ Rust** — `global.db`, `library-index.db` | kỷ luật mã Rust + AD-7, AD-11 | ⬜ nghiệm thu bằng **vắng mặt** |

**Khối `security` trong `tauri.conf.json`** *(khung để chép — CSP lấy nguyên từ cấu hình Story 1.1 đã kiểm chứng trên bản release)*:

```json
"security": {
  "csp": "default-src 'self'; script-src 'self'; style-src 'self' 'unsafe-inline'; font-src 'self' asset: http://asset.localhost; img-src 'self' asset: http://asset.localhost data:",
  "assetProtocol": {
    "enable": true,
    "scope": ["$RESOURCE/dict/**", "$RESOURCE/fonts/**"]
  }
}
```

**`src-tauri/capabilities/main.json`** — tối thiểu, không quyền plugin nào:

```json
{
  "$schema": "../gen/schemas/desktop-schema.json",
  "identifier": "main",
  "description": "Cửa sổ duy nhất. Không plugin filesystem — AD-1, AD-29.",
  "windows": ["main"],
  "permissions": ["core:default"]
}
```

**Ba điều phải hiểu đúng, nếu không sẽ báo cáo sai ở Completion Notes:**

1. **`assetProtocol.scope` là hàng rào thật, do framework cưỡng chế.** Đường dẫn ngoài scope bị webview **từ chối nạp** (*"asset protocol not configured to allow the path"*). Story 1.1 đã chạy đúng cơ chế này trên bản build release để nạp năm tệp font — nó **đã được kiểm chứng trên chính dự án này**, không phải suy từ tài liệu.
2. **Asset protocol chỉ đọc.** Không có đường ghi qua nó. Đó là lý do nó khớp trọn vẹn hai vùng `$RESOURCE/**` và **không** khớp `$APPDATA/**`.
3. **Nửa `$APPDATA/**` nghiệm thu bằng vắng mặt, không bằng khai báo.** Capabilities của Tauri canh **bề mặt IPC — tức webview**, không canh Rust: mã Rust gọi `std::fs` hay `rusqlite::Connection::open` không đi qua capabilities. Vì frontend **không có bất kỳ API filesystem nào** (không plugin `fs`, không `dialog`, không `sql`), phát biểu *"chỉ ba vùng này chạm tới"* đúng — nhưng nó đúng nhờ **vắng mặt bề mặt**, không nhờ một dòng khai báo.

> **Vì sao viết dài chỗ này thay vì chỉ đưa đoạn JSON.** AD-41 đã lập tiền lệ nói thẳng chỗ hàng rào yếu: *"capabilities của Tauri là khai báo tĩnh lúc build… AD-23 được framework cưỡng chế; AD-41 không."* Cùng tinh thần — **giấu chỗ yếu mới là chỗ nguy hiểm**.
>
> ⛔ **Đừng viết vào Completion Notes rằng "framework đã cưỡng chế mọi truy cập file".** Viết đúng ba dòng của bảng trên: hai vùng `$RESOURCE` do framework canh, vùng `$APPDATA` do kỷ luật Rust giữ và nghiệm thu bằng vắng mặt bề mặt.

### CSP — ba chỗ AC4 hỏng im lặng

Chuỗi CSP nằm trong khối `security` ở §Phạm vi tĩnh phía trên — **đó là bản Story 1.1 đã kiểm chứng trên bản build release**, chép nguyên.

`Cargo.toml`: `tauri = { version = "2.11.5", features = ["protocol-asset"] }` — feature này **bắt buộc** khi bật `assetProtocol`; `tauri-build` tự thêm và báo lỗi nếu thiếu (bẫy #4 của Story 1.1).

**Ba chỗ AC4 hỏng im lặng, ghi ra để không vấp:**

1. **`"csp": null` là TẮT CSP.** Nó không có nghĩa *"dùng mặc định"*. Khai chuỗi tường minh.
2. **`asset:` và `http://asset.localhost` không phải origin từ xa.** Chúng là giao thức cục bộ cho tài nguyên đã đóng gói. AD-15 cấm **CDN, font ngoài, ảnh ngoài** — cả ba đều là origin từ xa. Đặt chú thích ngay cạnh khối `security` để một giai đoạn sau không gỡ nhầm rồi làm hỏng đường nạp font.
3. **`http://asset.localhost` là dạng của Windows.** Bỏ nó đi thì macOS vẫn chạy và Windows hỏng — đúng loại khác biệt nền tảng mà Story 1.3 tồn tại để bắt.

#### Một quyết định phải cân, không được chép máy móc

`style-src 'unsafe-inline'` là thứ **nới nhất** trong khối trên. Nó **không** phải origin từ xa nên không phạm chữ của AD-15, nhưng nó nới một bậc mà AC4 (*"không có mã nào nới CSP"*) có quyền soi tới.

**Cách xử lý:** thử `style-src 'self'` trước trên bản build **release**. Vite tách style của SFC ra tệp CSS thật khi build, còn `:style` binding của Vue ghi qua CSSOM nên CSP không chặn — nhiều khả năng `'self'` đủ. Nếu bản release vẫn cần `'unsafe-inline'`, **giữ nó và ghi lý do cụ thể vào Completion Notes** (thứ gì đã bị chặn, ở đâu). Đừng chép nguyên vì Story 1.1 đã dùng — app thăm dò là app một tệp, không đại diện cho cây frontend thật.

### Nghiệm thu AC5 — công thức cụ thể, đừng kết luận bằng "không thấy gì"

**Quét cây phụ thuộc (phần "không có crash reporter hay analytics"):**

```bash
# Rust
cargo tree --prefix none --no-dedupe | sort -u > /tmp/deps-rust.txt
grep -Ei 'sentry|bugsnag|rollbar|crashlytics|datadog|newrelic|posthog|amplitude|mixpanel|segment-io|telemetry|analytics|opentelemetry' /tmp/deps-rust.txt

# npm
npm ls --all --parseable 2>/dev/null | sort -u > /tmp/deps-npm.txt
grep -Ei 'sentry|bugsnag|rollbar|crashlytics|datadog|newrelic|posthog|amplitude|mixpanel|analytics|telemetry|opentelemetry' /tmp/deps-npm.txt
```

⚠️ **`segment-io` khác `segment`.** Module Rust `core/segment/` của chính dự án tên là *segment* — đừng để phép quét tự báo động vào chính mình, và cũng đừng nới mẫu tới mức bỏ sót thư viện thật.

⚠️ **`reqwest` CÓ trong cây phụ thuộc từ story này, và đó không phải vi phạm AC5.** Cài trọn bảng Stack nghĩa là `reqwest` nằm đó nhưng **chưa có một dòng mã nào gọi tới**. AC5 nói *"không có **lời gọi** ra ngoài nào"* — một crate không được gọi thì không gọi đi đâu cả. Ba điểm ra mạng của AD-15 mở ở Story 4.x (`TranslationProvider`), 6.7 (`Fetcher`), 10.7 (kiểm tra phiên bản). **Ghi rõ điều này vào Completion Notes** để người rà soát không hiểu nhầm, và để phép quét ở Story 1.3 không đánh dấu sai.

**Quan sát mạng trọn một phiên (macOS):**

```bash
npm run tauri dev &                       # hoặc mở bản .dmg đã build
PID=$(pgrep -n AuraTranslate)
lsof -nP -p "$PID" -iTCP -iUDP            # chạy vài lần trong phiên
```

**Kết quả mong đợi:** không kết nối nào ra ngoài loopback. ⚠️ **Ở chế độ `dev` sẽ thấy kết nối tới `127.0.0.1:1420`** — đó là Vite dev server, cục bộ, **không phải** lời gọi ra ngoài. **Nghiệm thu AC5 phải chạy trên bản build release**, nơi frontend đã đóng gói và không còn dev server.

**Ghi lại thành số:** *"quan sát N lần trong M phút, thấy đúng K kết nối, tất cả tới 127.0.0.1"* — không ghi *"không thấy lời gọi nào"*. Tiền lệ Giai đoạn 0 và Story 1.1: số đo phải lặp lại được.

**NFR13 (không tài khoản, không đăng nhập, không đồng bộ đám mây) nghiệm thu ở đây bằng vắng mặt** — không màn hình đăng nhập, không SDK auth, không client đồng bộ trong cả hai cây. Ghi một dòng xác nhận vào Completion Notes; báo cáo mức sẵn sàng triển khai đã chỉ định story này làm chỗ đóng NFR13.

### AC6 — làm được gì hôm nay, và vì sao phần còn lại bàn giao

Máy dựng là macOS. Story 1.1 đã đâm vào rào này và ghi lại nguyên văn:

```
error: invalid value 'msi' for '--bundles [<BUNDLES>...]'
  [possible values: ios, app, dmg]
```

`.msi` dựng bằng WiX v3, `candle`/`light` là chương trình Windows — rào ở **tầng đóng gói**, không ở tầng biên dịch Rust.

| Việc | Ở story này | Ở Story 1.3 |
|---|---|---|
| `cargo check --target x86_64-pc-windows-msvc` | ✅ chạy được, target đã cài | — |
| Build `.dmg` chạy được trên macOS | ✅ | (lặp lại trên CI) |
| Build `.msi` chạy được trên Windows | ❌ công cụ từ chối trên macOS | ✅ runner Windows |
| So hành vi hai nền tảng | ❌ | ✅ — 1.3 đã mang sẵn AC nhận bàn giao |

**Bàn giao tường minh, không bỏ im lặng** — đúng khuôn Story 1.1 đã bàn giao hai phép đo `.msi`. Story 1.3 nhận thêm cả hai phép đo dung lượng `.msi` (có font / không font) và chế độ cài WebView2 (`downloadBootstrapper`; `embedBootstrapper` hay `offlineInstaller` **một mình nó** đủ làm `.msi` phình ~150 MB và vỡ NFR6 kể cả khi font bằng 0).

⛔ **Đừng đánh dấu AC6 đạt trọn.** Ghi *"đạt phần biên dịch; phần build và so hành vi bàn giao sang 1.3"*.

### Bẫy đã gặp thật ở Story 1.1 — năm cái, sẽ gặp lại

1. **`bundle_dmg.sh` chết ở bước AppleScript** khi không có phiên Finder tương tác: `Finder got an error: Can't set Finder window id … to 128. (-10006)` → `exit 64`. Bước đó chỉ trang trí vị trí icon. Đặt `CI=true` là Tauri truyền `--skip-jenkins` và bỏ hẳn bước này. **Story 1.3 sẽ gặp đúng lỗi này trên runner GitHub Actions.**
2. **Khai `[lib]` mà không có `src/lib.rs`** → `cargo metadata` gãy, Tauri CLI dừng trước cả khi biên dịch. Story này **có** `lib.rs` nên bẫy không phát sinh — nhưng tạo tệp **trước**, khai `[lib]` **sau**.
3. **Số crate `tauri` ≠ số CLI npm.** Đã đưa vào bảng ghim phiên bản ở trên.
4. **`assetProtocol` bật thì `Cargo.toml` bắt buộc có feature `tauri/protocol-asset`.** Đã đưa vào §CSP. Ở story này feature đó **không tuỳ chọn** — nó là cơ chế cưỡng chế duy nhất của AC3.
5. **`tauri build` xoá `.app` sau khi đóng gói `.dmg`.** Muốn giữ cả hai để soi bên trong thì build `--bundles app` riêng một lượt.

### Testing standards

Chưa có framework test nào trong repo, và **story này không phải chỗ chọn framework** — hãy chọn thứ rẻ nhất đủ dùng, để Story 1.3 gắn vào pipeline và các story sau mở rộng.

- **Phía Rust:** `cargo test` (built-in). Story này có thể chưa có `#[test]` nào có nghĩa — chấp nhận được, nhưng `cargo test` **phải chạy xanh** để 1.3 gắn vào CI được ngay.
- **Ba phép kiểm cưỡng chế của Task 8 quan trọng hơn unit test ở story này.** Chúng là thứ giữ AC2, AC3, AC5 khỏi thoái hoá thành kỷ luật — cùng hình dạng mà AD-13 và AD-41 đòi bộ test riêng.
- **Kiểm 1 và Kiểm 2 phải chạy được bằng một lệnh** và trả **mã thoát khác 0 khi thất bại**. Một script in ra cảnh báo rồi trả 0 là script không cưỡng chế được gì.
- **Kiểm 3 (đọc ngoài scope bị từ chối) phải có cả hai chiều** — một lượt đọc trong scope **thành công** và một lượt ngoài scope **bị từ chối**. Chỉ kiểm chiều từ chối thì một cấu hình chặn sạch mọi thứ vẫn "qua".
- **Ghi lại phiên bản toolchain và lệnh chính xác** — tiền lệ Giai đoạn 0 và Story 1.1. Đây là thứ thay cho test ở phần nghiệm thu bằng quan sát (AC5, AC6).
- **Đừng viết test vào `src-tauri/src/main.rs`.** Test tích hợp sống ở `src-tauri/tests/` và `use auratranslate_lib::…` — đó là lý do Task 1 bắt bố cục `lib.rs` + `main.rs`.

### Project Structure Notes

- **Gốc repo là gốc dự án Node.** `package.json`, `index.html`, `vite.config.ts`, `src/` nằm ở gốc; `src-tauri/` là thư mục con. Đây là hình dạng mà Structural Seed vẽ (`src/` và `src-tauri/` ngang hàng) — **đừng** lồng frontend vào một thư mục `frontend/` hay `ui/`.
- **`dict-manifest.toml` ở gốc repo**, không phải trong `src-tauri/` — Structural Seed đặt nó ngang hàng với `tools/`.
- **`tools/dict-build/` chưa là crate.** Chưa có `Cargo.toml`, chưa có workspace. Story 1.9 quyết hình dạng của nó. Dựng sẵn một workspace hôm nay là quyết định thay cho story chưa tới.
- **Quy ước đặt tên** (Consistency Conventions): Rust `snake_case`; Vue component `PascalCase.vue`; khoá chuỗi `vi.json` **phẳng theo khoá chấm** (`lookup.empty_result`). Ánh xạ tên thực thể đã cố định: Tác phẩm → `Work` (⛔ **cấm** `Project`, `Book`, `Novel`, `Document`) · Chương → `Chapter` · Panel Lookup → `LookupPanel` · Smart RAG Injector → `RagInjector` · Hán Việt → `HanViet`. Đuôi `.atproj` là ngoại lệ lịch sử, **không** kéo theo tên thực thể.
- **Không chuỗi tiếng Việt nào trong `.rs` hay `.vue`** (NFR16, AD-21) — áp **từ dòng code đầu tiên**, kể cả ở story scaffold này. Doc-comment và chú thích mã là ngoại lệ hiển nhiên (chúng không hiển thị ra giao diện); **văn bản hiển thị** thì không.
- Bốn tệp quy hoạch có thể phải sửa: `ARCHITECTURE-SPINE.md` (bảng Stack — Task 7) và `.memlog.md` của nó. **Đừng sửa `epics.md`** trừ khi Ice chỉ đạo tường minh — Story 1.1 đã lập tiền lệ: sửa `epics.md` là **ngoài phạm vi mặc định của `dev-story`**.

### References

- [Source: `_bmad-output/planning-artifacts/epics.md#Story 1.2`] — sáu AC nguyên văn, Covers FR104 · NFR12 · NFR14
- [Source: `_bmad-output/planning-artifacts/epics.md#Starter template & khung dự án`] — *"KHÔNG có starter template bên ngoài nào được chỉ định"*; cây nguồn bắt buộc; ba phụ thuộc đã loại có lý do
- [Source: `_bmad-output/planning-artifacts/epics.md#Story 1.3`] — story nhận bàn giao AC6 và hai phép đo `.msi`; *"gắn vào chính pipeline này, không dựng pipeline thứ hai"*
- [Source: `ARCHITECTURE-SPINE.md#Structural Seed`] — cây nguồn đầy đủ kèm chú thích AD từng module
- [Source: `ARCHITECTURE-SPINE.md#Stack`] — bảng ghim phiên bản; *"Không dùng, đã loại có lý do"*
- [Source: `ARCHITECTURE-SPINE.md#AD-23`] — scope tĩnh ba vùng: `$RESOURCE/dict/**` · `$RESOURCE/fonts/**` chỉ đọc, `$APPDATA/**` đọc+ghi; scope động chỉ cấp qua hộp thoại
- [Source: `ARCHITECTURE-SPINE.md#AD-15`] — đúng ba điểm ra mạng; CSP giữ nguyên không nới: không CDN, không font ngoài, không ảnh ngoài
- [Source: `ARCHITECTURE-SPINE.md#AD-41`] — tiền lệ **nói thẳng chỗ hàng rào yếu** thay vì giấu; capabilities là khai báo tĩnh lúc build
- [Source: `ARCHITECTURE-SPINE.md#AD-2`] — đúng ba cổng; cổng thứ tư phải là một AD mới
- [Source: `ARCHITECTURE-SPINE.md#AD-13`] — không module nào ngoài `ai/` được phụ thuộc `ai/`; test cưỡng chế ở Story 4.1
- [Source: `ARCHITECTURE-SPINE.md#AD-24`] — một cửa sổ OS, ba chế độ → cửa sổ label `main`
- [Source: `ARCHITECTURE-SPINE.md#AD-25`] — dữ liệu từ điển là artifact có phiên bản + checksum; `dict-manifest.toml`; parser chỉ ở build tool
- [Source: `ARCHITECTURE-SPINE.md#AD-21`, `#Consistency Conventions`] — không chuỗi tiếng Việt trong `.rs`/`.vue`; quy ước đặt tên; *"mỗi phụ thuộc mới phải rà tương thích GPLv3 trước khi thêm vào (NFR15) và ghi vào bảng Stack"*
- [Source: `prd.md:778`] — FR104 không telemetry
- [Source: `prd.md:838–851`] — NFR11 · NFR12 · NFR13 · NFR14 · NFR15 · NFR16
- [Source: `_bmad-output/implementation-artifacts/1-1-mui-tham-do-font-do-dung-luong-that-va-ra-giay-phep.md#Ranh giới phạm vi`] — *"Không dùng app thăm dò làm scaffold cho Story 1.2"*; trạng thái repo
- [Source: `research/font-spike-results-2026-08-03.md#Cấu hình Tauri đã kiểm chứng`] — khối `security` (CSP + `assetProtocol`) đã kiểm trên bản release, ghi rõ *"Story 1.2 chép lại đúng chỗ này"*
- [Source: `research/font-spike-results-2026-08-03.md#Bẫy gặp thật khi làm`] — bốn bẫy Tauri
- [Source: `research/font-spike-results-2026-08-03.md#Phép đo 5`] — bốn tệp font + SHA-256 (Task 9 dùng nếu Ice đồng ý)
- [Source: `DESIGN.md` frontmatter `families` / `fonts-bundled`] — bốn họ chữ và chính sách nhúng font (Task 9)
- [Source: `implementation-readiness-report-2026-08-03.md:631,913,919–920,1056`] — NFR13 đóng ở Story 1.2; xác nhận 1.2 đạt chuẩn *"không starter template"*; 1.2 sở hữu capabilities/CSP/scope; AC hai nền tảng là phép kiểm tay → 1.3
- [Source: `research/technical-auratranslate-tauri-rust-local-first-research-2026-08-02.md:343–352`] — bộ plugin từng đề xuất (`sql` · `keyring` · `fs` · `dialog`); mô hình ba tầng Capabilities/Permissions/Scopes; nguyên tắc default-deny
- [Web 2026-08-03] Tauri v2 §Asset protocol scope — `app.security.assetProtocol` nhận mảng glob hoặc object `{allow, deny, requireLiteralLeadingDot}`; đường dẫn ngoài scope bị webview **từ chối nạp** (*"asset protocol not configured to allow the path"*), áp cho mọi path qua `convertFileSrc()`
- [Web 2026-08-03] npm registry — `@tauri-apps/cli` latest **2.11.4** (không có 2.11.5) · `typescript` latest **7.0.2**, 5.x mới nhất **5.9.3** · `vue` 3.5.40 · `vite` 8.2.0 (`engines.node ^20.19.0 || >=22.12.0`) · `@vitejs/plugin-vue` 6.0.8 · `vue-tsc` 3.3.9 · `@tauri-apps/api` 2.11.1 · `dockview-vue` 7.0.4 (peer `vue ^3.4.0`)
- [Web 2026-08-03] crates.io — mọi hàng bảng Stack tồn tại đúng số ghim: `tauri` 2.11.5 · `rusqlite` 0.40.1 · `libsqlite3-sys` 0.38.1 · `jieba-rs` 0.10.3 · `tantivy-stemmers` 0.4.0 · `docx-rs` 0.4.22 · `keyring` 4.1.6 · `reqwest` **0.13.4** · `similar` 3.1.1 · `dissimilar` 1.0.11
- [Local 2026-08-03] `rustc`/`cargo` 1.97.1 · Node v22.22.2 · npm 10.9.7 · target đã cài `x86_64-apple-darwin`, `x86_64-pc-windows-msvc`

### Bốn quyết định của Ice — 2026-08-03, đã áp vào story

| # | Câu hỏi | Ice quyết | Áp ở đâu |
|---|---|---|---|
| 1 | Cài trọn bảng Stack hay cài theo story cần tới? | **Cài trọn** | Task 6 · §Bảng ghim phiên bản. Ngoại lệ duy nhất: hàng `similar`/`dissimilar` còn Deferred |
| 2 | Tệp font vào repo ở story này? | **Có** — UX-DR4 đóng ở đây | Task 9 · cây nguồn |
| 3 | `core/i18n/` — theo AC1 hay theo bảng quy ước? | **Theo bảng Consistency Conventions** → có tạo | Task 2 · §`core/i18n/` là gì |
| 4 | `tauri-plugin-fs`? | **Không dùng** — Ice nhớ đúng | Task 4 · Task 7 · Task 8 Kiểm 1 · §Vì sao không có plugin `fs` |

**Về #4, ghi lại cho rõ vì nó dễ bị mở lại:** `tauri-plugin-fs` **không** nằm trong danh sách *"Không dùng, đã loại có lý do"* của bảng Stack — danh sách đó chỉ nêu `tauri-plugin-stronghold`, `tauri-plugin-keyring`, `tauri-wire`. Chỗ duy nhất nó xuất hiện là báo cáo technical research 2026-08-02, trong bộ bốn plugin đề xuất `sql · keyring · fs · dialog` — **mà kiến trúc sau đó đã bác từng cái một** (AD-29 bác `keyring`, bảng Stack chọn `rusqlite` trực tiếp thay `sql`). Ice nhớ đúng cái mạch đó. Task 7 phải **ghi nó thành chữ vào bảng Stack**, để lần sau không ai phải suy luận lại.

---

## Dev Agent Record

### Agent Model Used

### Debug Log References

### Completion Notes List

### File List

## Change Log

| Ngày | Thay đổi |
|---|---|
| 2026-08-03 | Story được dựng bằng `bmad-create-story`. Phân tích `epics.md` §Story 1.2 + §Additional Requirements · `ARCHITECTURE-SPINE.md` (43 AD, Stack, Structural Seed, Consistency Conventions) · `prd.md` FR104/NFR11–NFR16 · Story 1.1 (bốn bẫy Tauri, tiền lệ bàn giao) · `font-spike-results-2026-08-03.md` (CSP + assetProtocol đã kiểm chứng) · `implementation-readiness-report` (NFR13 đóng ở đây). Kiểm chứng lại toàn bộ phiên bản trên npm/crates.io và toolchain cục bộ ngày 2026-08-03 — phát hiện hai bẫy phiên bản (`typescript` latest = 7.0.2; `@tauri-apps/cli` không có 2.11.5) |
| 2026-08-03 | **Ice quyết bốn điểm.** (1) Cài **trọn** bảng Stack — Task 6/7 và §Bảng ghim phiên bản viết lại thành hai bảng đầy đủ Rust + frontend; phát hiện hàng `similar`/`dissimilar` **không cài được** vì còn ở bảng Deferred, ghi thành mục riêng thay vì bỏ im lặng. (2) **Tệp font vào repo ở story này** — Task 9 đổi từ *"chờ xác nhận"* thành nhiệm vụ thật, kèm đối chiếu SHA-256 và ba tệp `LICENSE`; UX-DR4 đóng ở đây. (3) `core/i18n/` **có tạo**, theo bảng Consistency Conventions — thêm §`core/i18n/` là gì nêu rõ nó giữ **danh mục `message_key`**, không giữ văn bản hiển thị; hình dạng thật bàn giao sang Story 1.5. (4) **Bỏ `tauri-plugin-fs`** — soát lại tài liệu xác nhận Ice nhớ đúng: plugin này chỉ có trong báo cáo technical research (bộ `sql · keyring · fs · dialog`), và kiến trúc đã bác từng cái theo cùng một lý do (AD-1 + AD-29). Task 4 và §Phạm vi tĩnh viết lại quanh `assetProtocol.scope` — cơ chế Story 1.1 đã kiểm chứng thật trên bản release; AD-23 tách thành hai nửa với hai cơ chế cưỡng chế khác nhau, nói thẳng nửa `$APPDATA` nghiệm thu bằng vắng mặt |
