---
baseline_commit: 754f0f9a1a4f1da5b297cdbfa20bc9596a304139
---

# Story 1.2: Scaffold dự án và khoá phạm vi filesystem, phạm vi mạng

Status: review

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

- [x] **Task 1 — Dựng khung Tauri v2 bằng tay, không qua `create-tauri-app`** (AC: 1)
  - [x] ⛔ **Không chạy `create-tauri-app`, không chạy `npm create tauri-app`, không copy app thăm dò của Story 1.1.** AC1 mang mệnh đề nguyên văn *"không dùng bất kỳ starter template cộng đồng nào"*, và §Ranh giới phạm vi của Story 1.1 đã cấm tường minh việc dùng app thăm dò làm scaffold.
  - [x] Tạo `package.json` ở gốc repo (npm — máy đã có npm 10.9.7, Node v22.22.2, đủ cho Vite 8).
  - [x] Tạo `src-tauri/Cargo.toml`, `src-tauri/build.rs`, `src-tauri/tauri.conf.json`.
  - [x] **Bố cục crate: `src/lib.rs` (crate root, giữ `pub fn run()`) + `src/main.rs` (chỉ gọi `run()`).** Đây là quy ước của **chính framework Tauri v2**, không phải starter template cộng đồng — và nó là điều kiện để `tests/` có thể `use` được mã sản phẩm (Task 8 cần).
  - [x] Khai `[lib] name = "auratranslate_lib"`, `crate-type = ["staticlib", "cdylib", "rlib"]`. ⚠️ **Khai `[lib]` mà không có `src/lib.rs` là bẫy #2 của Story 1.1** — `cargo metadata` gãy trước cả khi biên dịch. Tạo tệp trước, khai sau.
  - [x] Cửa sổ duy nhất mang label `main` (AD-24 — một cửa sổ OS, ba chế độ). Label này bị `capabilities` tham chiếu ở Task 4.
  - [x] `productName` = `AuraTranslate`. Tên này quyết định tên tiến trình — công thức quan sát mạng ở §Nghiệm thu AC5 `pgrep` theo đúng chuỗi này.
  - [x] ⛔ **Không** để `identifier` mặc định kiểu `com.tauri.dev` — Tauri từ chối build. Dùng `com.auratranslate.desktop`. ⚠️ **Đừng kết thúc identifier bằng `.app`** — nó đụng phần mở rộng bundle của macOS.
  - [x] Khối `build` của `tauri.conf.json`: `beforeDevCommand: "npm run dev"` · `beforeBuildCommand: "npm run build"` · `devUrl: "http://localhost:1420"` · `frontendDist: "../dist"`. Bốn trường này sai một cái là `tauri dev` treo hoặc `tauri build` đóng gói một thư mục rỗng — **và bản rỗng vẫn build thành công**, hỏng im lặng.
  - [x] `vite.config.ts` cần bốn thiết lập cho Tauri: `server.port = 1420` + `server.strictPort = true` (nếu Vite tự nhảy cổng thì `devUrl` trỏ sai) · `server.host` để trống cho desktop · `clearScreen: false` (giữ lại lỗi Rust trên terminal) · `envPrefix: ['VITE_', 'TAURI_']`.

- [x] **Task 2 — Dựng đúng cây nguồn Rust, mỗi thư mục là module thật** (AC: 1)
  - [x] `src-tauri/src/commands/mod.rs` · `src-tauri/src/ports/mod.rs` · `src-tauri/src/core/mod.rs`.
  - [x] `src-tauri/src/core/<x>/mod.rs` cho **đúng mười hai** module: `segment` `matching` `glossary` `tm` `dict` `library` `export` `webimport` `ai` `scope` `store` **`i18n`**.
  - [x] **Dùng `mod.rs` rỗng, KHÔNG dùng `.gitkeep`.** Lý do: `mod.rs` khai trong `lib.rs` thì trình biên dịch đi qua chúng — cây nguồn trở thành thứ `cargo check` xác nhận, không phải thứ mắt người phải soát. `.gitkeep` không cưỡng chế được gì.
  - [x] Mỗi `mod.rs` mang **một dòng doc-comment** ghi module sở hữu khái niệm gì + AD ràng buộc nó (chép từ bảng Cây nguồn ở §Cây nguồn phải dựng). Đây là chỗ rẻ nhất để giữ ranh giới khỏi trôi.
  - [x] `ports/mod.rs` ghi chú **đúng ba cổng** `DictionarySource` · `TranslationProvider` · `ProjectStore` và mệnh đề *"cổng thứ tư phải là một AD mới"* (AD-2). Chưa khai trait nào ở story này.
  - [x] `core/ai/mod.rs` ghi chú AD-13: *không module nào ngoài `ai/` được import `ai/`*; test cưỡng chế thuộc Story 4.1.
  - [x] `core/i18n/mod.rs` — **có tạo** *(Ice chốt 2026-08-03: theo bảng Consistency Conventions)*. Doc-comment phải nói rõ nó **không** chứa văn bản hiển thị: xem §`core/i18n/` là gì.

- [x] **Task 3 — Dựng cây nguồn frontend và các tệp gốc còn lại** (AC: 1)
  - [x] `src/{modes,panels,layout,commands,tokens}` — mỗi thư mục một `.gitkeep` kèm `README.md` một dòng ghi **story nào sở hữu** nó (`modes`/`panels`/`layout` → 1.14 · `commands` → 1.6 · `tokens` → 1.4).
  - [x] `src/i18n/vi.json` — tệp thật với nội dung `{}`, không phải `.gitkeep`. Story 1.5 sở hữu nội dung; sự tồn tại của tệp là AC của story này.
  - [x] `src/main.ts`, `src/App.vue`, `index.html`, `vite.config.ts`, `tsconfig.json`, `tsconfig.node.json`, `env.d.ts`.
  - [x] `src-tauri/resources/dict/.gitkeep` + `README.md` ghi: *file `.db` không nằm trong git, tải theo `dict-manifest.toml` (AD-25); Story 1.9 và 10.1 sở hữu.*
  - [x] `src-tauri/resources/fonts/` — **thư mục này không có trong danh sách AC1 nhưng AC3 đòi scope `$RESOURCE/fonts/**`**, và AD-23 khai nó tường minh. Tạo thư mục; nội dung đặt ở Task 9.
  - [x] `tools/dict-build/` + `README.md` ghi: *parser sống ở đây, không vào bản phát hành (AD-25); Story 1.9 dựng nội dung.* **Chưa dựng thành crate Rust** — chưa có `Cargo.toml`, chưa vào workspace.
  - [x] `dict-manifest.toml` ở **gốc repo** — khung rỗng có chú thích mô tả ba trường bắt buộc mỗi tệp: `url`, `sha256`, `phiên bản nguồn thô` (AD-25). Không điền giá trị giả.
  - [x] Bổ sung `.gitignore`: `/src-tauri/target/`, `/src-tauri/gen/schemas/`, `/dist/`, `*.tsbuildinfo`. ⚠️ Dòng `*.db` **đã có sẵn** và đúng ý — nó giữ file từ điển ra ngoài git theo AD-25; **đừng gỡ nó**.
  - [x] ℹ️ `src-tauri/gen/schemas/` do `tauri-build` sinh ra ở lần build đầu. Trước lần build đầu, dòng `"$schema": "../gen/schemas/desktop-schema.json"` trong capabilities **chưa phân giải được** — đó là chuyện của editor tooling, **không phải lỗi**. Đừng gỡ dòng `$schema` để "sửa" nó.

- [x] **Task 4 — Phạm vi filesystem tĩnh theo AD-23, KHÔNG qua plugin `fs`** (AC: 3)
  - [x] ⛔ **Không cài `tauri-plugin-fs`, `tauri-plugin-sql`, `tauri-plugin-dialog`, `tauri-plugin-store`.** Lý do đầy đủ ở §Vì sao không có plugin `fs` — đọc trước khi làm task này.
  - [x] Tạo `src-tauri/capabilities/main.json` khai `"windows": ["main"]`, permissions **tối thiểu**: `core:default` (cần cho `resolveResource`/`convertFileSrc`) và không gì khác. Không thêm quyền plugin nào.
  - [x] Khai phạm vi tĩnh trong `app.security.assetProtocol.scope` của `tauri.conf.json` — **đúng hai mục, không hơn**: `$RESOURCE/dict/**` và `$RESOURCE/fonts/**`, cả hai **chỉ đọc theo bản chất giao thức**. Khung JSON ở §Phạm vi tĩnh.
  - [x] ⛔ **Không đưa `$APPDATA` vào `assetProtocol.scope`.** Frontend không có việc gì với `global.db` hay `library-index.db` (AD-1, AD-11). Nửa `$APPDATA/**` của AD-23 là phạm vi của **mã Rust**, xem §Phạm vi tĩnh.
  - [x] Bật feature `protocol-asset` trong `Cargo.toml` — **bắt buộc** khi bật `assetProtocol`; `tauri-build` báo lỗi nếu thiếu (bẫy #4 của Story 1.1).
  - [x] Phía Rust: mọi đường dẫn `$APPDATA` lấy qua `app.path().app_data_dir()`, **không viết cứng** `~/Library/Application Support/…`. Đường dẫn viết cứng là chỗ NFR14 (hành vi tương đương hai nền tảng) hỏng đầu tiên.

- [x] **Task 5 — CSP và ba hàng rào mạng** (AC: 4, 5)
  - [x] Chép nguyên văn khối `security` mà Story 1.1 đã kiểm chứng trên bản build release (§Cấu hình Tauri đã kiểm chứng của báo cáo mũi thăm dò). Khung ở §CSP dưới.
  - [x] Khai CSP **tường minh trong `tauri.conf.json`**, không để `null`. `csp: null` là **tắt CSP**, không phải "dùng mặc định" — đây là chỗ AC4 hỏng im lặng dễ nhất.
  - [x] Ghi một chú thích ngay cạnh khối `security` nêu: *`font-src asset:` và `img-src asset:` KHÔNG phải nới CSP theo nghĩa AD-15 cấm — AD-15 cấm origin **từ xa**; asset protocol là tài nguyên cục bộ đã nằm trong bản cài.* Chú thích này tồn tại để một giai đoạn sau không gỡ nhầm.
  - [x] Thử hạ `style-src` xuống `'self'` trước; chỉ giữ `'unsafe-inline'` nếu bản build **release** thật sự cần, và **ghi lý do vào Completion Notes**. Xem §Một quyết định phải cân, không được chép máy móc.
  - [x] ⛔ Không thêm bất kỳ `devCsp` nào nới ra ngoài `'self'` + kênh HMR cục bộ.
  - [x] ⛔ Không thêm `http`/`https` client nào ở story này. Ba điểm ra mạng của AD-15 thuộc Story 4.x, 6.7, 10.7 — **không có điểm thứ tư**.

- [x] **Task 6 — Cài TRỌN bảng Stack, ghim chính xác** *(Ice chốt 2026-08-03)* (AC: 2)
  - [x] Cài **toàn bộ** các hàng của bảng Stack ngay ở commit này, không đợi story cần tới. Bảng đầy đủ kèm bẫy kênh phát hành ở §Bảng ghim phiên bản.
  - [x] Ghim **chính xác**, không dùng dải rộng (`^`, `~`, `*`). Cả mười hai hàng đều tồn tại đúng số đã ghim — xác minh lại crates.io/npm ngày 2026-08-03.
  - [x] ⚠️ **`typescript` phải ghim `5.9.3`.** `npm i -D typescript` hôm nay kéo về **7.0.2** — bảng Stack ghi *TypeScript 5.x*, nên cài mặc định là **vi phạm AC2** ngay ở lệnh đầu tiên.
  - [x] ⚠️ **`@tauri-apps/cli` ghim `2.11.4`, không phải `2.11.5`.** Crate `tauri` và CLI npm đánh số riêng; `2.11.5` **không tồn tại** trên npm (bẫy #3 của Story 1.1, xác minh lại 2026-08-03: `dist-tags.latest = 2.11.4`).
  - [x] ⚠️ **`similar` / `dissimilar` là hàng DUY NHẤT không cài được** — xem §Một hàng của bảng Stack chưa cài được. Ghi tường minh vào Completion Notes, đừng bỏ im lặng.
  - [x] ⚠️ **`rusqlite` feature `bundled` biên dịch SQLite từ nguồn C** — lần build đầu chậm hơn hẳn (vài phút). Đó là bình thường, không phải treo. `libsqlite3-sys 0.38.1` là phụ thuộc bắc cầu của `rusqlite 0.40.1`; khai tường minh để lock ghim đúng số bảng Stack.
  - [x] Crate cài mà chưa dùng **không** sinh cảnh báo của `cargo` — nên đừng chờ trình biên dịch nhắc. Ghi vào doc-comment của module sở hữu (`core/dict/` cho `jieba-rs`, `core/store/` cho `rusqlite`…) rằng crate nào dành cho nó, để story sau không cài trùng bằng tên khác.
  - [x] Commit **cả** `Cargo.lock` **và** `package-lock.json`. Không có lock thì "ghim phiên bản" chỉ đúng trên máy người dựng đầu tiên, và AC6 (*cùng một commit → hai nền tảng*) mất nghĩa.
  - [x] Ghi vào Completion Notes **phiên bản đã giải quyết thật** của từng phụ thuộc (đọc từ lock), không chép lại con số trong Dev Notes. `reqwest` đặc biệt cần ghi — bảng Stack chỉ ghi *"mới nhất lúc dựng"*, nên **số thật phải quay ngược vào bảng**.

- [x] **Task 7 — Rà giấy phép và cập nhật bảng Stack** (AC: 2)
  - [x] NFR15: **mỗi** phụ thuộc phải rà tương thích GPL v3 **trước khi** thêm, và ghi vào bảng Stack (Consistency Conventions).
  - [x] Rà **bằng cách đọc tệp `LICENSE` trong nguồn đã tải** (`~/.cargo/registry/src/…`, `node_modules/…`), đúng tiền lệ Story 1.1 — nhãn của registry là dẫn xuất, không phải nguồn sự thật. Story 1.1 đã bắt được `source-han-serif` bị GitHub gắn `NOASSERTION` trong khi văn bản nói rõ OFL 1.1.
  - [x] Mười hai hàng đã có sẵn cột giấy phép (kiểm chứng 2026-08-02) → việc ở đây là **xác minh lại bằng tệp**, không phải tra lại từ đầu. Đánh dấu hàng nào đã tự tay mở tệp mà đọc.
  - [x] Thêm hàng **mới** vào bảng Stack cho các phụ thuộc chưa có: `tauri-build` · `serde` · `serde_json` · `@vitejs/plugin-vue` · `vue-tsc` · `@tauri-apps/api` · `@tauri-apps/cli` — theo đúng khuôn ba cột `Name` · `Version` · `Giấy phép`.
  - [x] Điền **số thật** của `reqwest` vào bảng, thay chuỗi *"mới nhất lúc dựng"*.
  - [x] Thêm **`tauri-plugin-fs`** vào danh sách *"Không dùng, đã loại có lý do"* của bảng Stack, kèm lý do một dòng (AD-1 + AD-29 — plugin tồn tại để phơi API ra JS; webview mỏng không có việc gì với filesystem). Ice chốt 2026-08-03. Xem §Vì sao không có plugin `fs`.
  - [x] Thêm một dòng `(decision)` vào `.memlog.md` của architecture ghi quyết định này — đúng khuôn dòng `:48` đã ghi cho `tauri-plugin-keyring`.

- [x] **Task 8 — Ba phép kiểm cưỡng chế, chạy được bằng lệnh** (AC: 2, 3, 5)
  - [x] **Kiểm 1 — ba phụ thuộc đã loại vắng mặt (AC2).** `cargo tree -i <tên>` phải **không tìm thấy** cho `tauri-plugin-stronghold`, `tauri-plugin-keyring`, `tauri-wire`. Viết thành script chạy được, đừng làm bằng mắt.
  - [x] **Kiểm 2 — không crash reporter, không analytics (AC5).** Quét **cả hai** cây: `cargo tree` và `npm ls --all`. Danh sách từ khoá và cách quét ở §Nghiệm thu AC5.
  - [x] **Kiểm 3 — đọc ngoài scope bị từ chối (AC3).** Từ frontend, `convertFileSrc()` một đường dẫn ngoài `assetProtocol.scope` (`/etc/hosts` trên macOS, `C:\Windows\win.ini` trên Windows) rồi `fetch` nó → phải **thất bại**. Kèm một lượt nạp **trong** scope thành công (`resolveResource('fonts/SourceSans3[wght].ttf')` → `convertFileSrc` → `FontFace.load()`) — nếu không, một cấu hình chặn sạch mọi thứ vẫn "qua" phép kiểm.
  - [x] Đặt ba phép kiểm ở chỗ **Story 1.3 gắn thẳng vào pipeline được** (AC của 1.3: *"gắn vào chính pipeline này, không dựng pipeline thứ hai"*). Gợi ý: `scripts/check-deps.sh` (hoặc `.mjs`) cho Kiểm 1+2; Kiểm 3 là test frontend.
  - [x] ⚠️ **Kiểm 1 phải mở rộng thêm một dòng:** `tauri-plugin-fs` cũng phải **không có mặt** trong cây phụ thuộc (Ice chốt 2026-08-03). AC2 chỉ liệt kê ba tên, nhưng quyết định này thuộc cùng một hạng — và nếu không có phép kiểm, một story sau sẽ cài nó vào để "cho tiện" mà không ai biết.
  - [x] Ghi vào Completion Notes **lệnh chính xác** để chạy lại cả ba — Story 1.3 sẽ chép chúng vào workflow.

- [x] **Task 9 — Đặt bốn tệp font vào repo, đóng UX-DR4** *(Ice chốt 2026-08-03)* (AC: 3)
  - [x] Đặt **bốn tệp** đã đo ở Story 1.1 vào `src-tauri/resources/fonts/`: `NotoSerifCJKtc-Regular.otf` · `SourceSerif4[opsz,wght].ttf` · `SourceSerif4-Italic[opsz,wght].ttf` · `SourceSans3[wght].ttf`.
  - [x] **Đối chiếu SHA-256 từng tệp** với bảng ở `font-spike-results-2026-08-03.md §Phép đo 5` trước khi commit. Đây là chỗ duy nhất bắt được việc lấy nhầm `NotoSerifTC` (bản subset theo ngôn ngữ, 45 MB) thay `NotoSerifCJKtc` (biến thể vùng đầy đủ) — nhầm này **hỏng im lặng**: phần lớn ký tự vẫn hiện, chỉ tofu khi gặp văn bản khác hệ chữ.
  - [x] Tổng bốn tệp phải ra **27.253.184 byte** (25,991 MiB). Lệch là lấy sai tệp.
  - [x] Đặt kèm **ba tệp `LICENSE` gốc** (OFL 1.1 của `noto-cjk`, `sourceserif4`, `sourcesans3`) — điều kiện của FR38 và FR109, và Story 1.1 đã khuyến nghị mang theo để bịt luôn câu hỏi ở Story 10.4/10.5.
  - [x] Khai `bundle.resources` trỏ `resources/fonts/` (và **chưa** khai cho `resources/dict/` — thư mục đó còn rỗng; xem bẫy dưới).
  - [x] ⚠️ **Bẫy đóng gói:** `bundle.resources` trỏ vào glob **không khớp tệp nào** có thể làm `tauri build` gãy. Chỉ khai cho thư mục đã có tệp thật.
  - [x] ⚠️ **`.gitignore` hiện có dòng `*.tmp` và `*.db` nhưng KHÔNG chặn `.otf`/`.ttf`** — kiểm lại `git status` sau khi thêm để chắc bốn tệp thật sự vào staging, đừng giả định.
  - [x] Ghi vào Completion Notes rằng **UX-DR4 đóng ở story này** (nó không nằm trong Covers của story nào — Ice quyết gộp vào đây 2026-08-03), và cập nhật `epics.md` §UX Design Requirements nếu Ice chỉ đạo tường minh. ⚠️ Sửa `epics.md` là **ngoài phạm vi mặc định của `dev-story`** — hỏi trước.
  - [x] ℹ️ Story này **không** dựng `@font-face` hay token typography — đó là Story 1.4. Ở đây chỉ cần một lượt nạp thử để Kiểm 3 chạy được.

- [x] **Task 10 — Nghiệm thu chạy thật và quan sát mạng** (AC: 5, 6)
  - [x] `npm run tauri dev` → cửa sổ mở, không lỗi console.
  - [x] `npm run tauri build --bundles dmg` → ra `.dmg` chạy được. ⚠️ Đặt `CI=true` nếu `bundle_dmg.sh` chết ở bước AppleScript (bẫy #1 của Story 1.1).
  - [x] **Quan sát mạng trọn một phiên** theo công thức ở §Nghiệm thu AC5 — mở app, dùng thử, đóng app; ghi lại kết quả quan sát thành số, không ghi *"không thấy gì"*.
  - [x] Ghi **phiên bản toolchain** (Rust, Node, npm, `@tauri-apps/cli`, hệ điều hành) vào Completion Notes — cùng tiền lệ Giai đoạn 0 và Story 1.1, để số đo lặp lại được.

- [x] **Task 11 — AC6: làm được phần nào ở đây, bàn giao phần nào cho 1.3** (AC: 6) *(phạm vi thu hẹp — Ice chốt 2026-08-03, xem subtask đầu)*
  - [ ] ~~Chạy `cargo check --target x86_64-pc-windows-msvc` (target đã cài sẵn trên máy — xác minh 2026-08-03). Đây là bằng chứng **tầng biên dịch**: không có mã phụ thuộc nền tảng lọt vào.~~
    → ⛔ **KHÔNG LÀM ĐƯỢC TRÊN MÁY NÀY. Ô cố ý để trống, không tick bừa.** Lệnh gãy ở ba crate build **native C** — `zstd-sys` (qua `jieba-rs` → `include-flate`), `libsqlite3-sys` (qua `rusqlite` feature `bundled`), `aws-lc-sys` (qua `reqwest` → `rustls`) — với lỗi `cc-rs: command did not execute successfully … --target=x86_64-pc-windows-msvc`. Máy không có `cl.exe`/`clang-cl`/`lld-link`. **Tiền đề của subtask này sai:** *target Rust đã cài ≠ cross-compile được*; rào nằm ở **tầng biên dịch C**, cùng hình dạng rào WiX/`.msi` mà Story 1.1 đâm phải ở tầng đóng gói.
    → **Bằng chứng tầng biên dịch lấy được thay thế, không phải bằng 0:** `cargo metadata --filter-platform x86_64-pc-windows-msvc` **OK**, cây phụ thuộc phân giải trọn vẹn cho Windows (**346 crate**, gồm `webview2-com` 0.38.2 · `windows-core` 0.61.2 · `windows-sys` 0.59.0/0.61.2) → không phụ thuộc nào là macOS-only; và `grep` xác nhận mã nguồn dự án **không có** `cfg(target_os)`/`cfg(windows)`/`cfg(unix)` lẫn đường dẫn viết cứng.
    → **Ice chốt 2026-08-03: bàn giao sang Story 1.3**, nơi lệnh này chỉ là `cargo check` bình thường trên runner Windows. **Không** thêm `cargo-xwin` — đó là phụ thuộc ngoài bảng Stack và vẫn không thay được bản Windows chạy thật mà AC6 đòi.
  - [x] ⚠️ **Không cố dựng `.msi` trên macOS.** Story 1.1 đã đâm vào đúng rào này: `tauri-cli` từ chối target `msi` vì WiX v3 là chương trình Windows. Rào ở **tầng đóng gói**, không ở tầng biên dịch.
  - [x] **Bàn giao tường minh sang Story 1.3** — bản build Windows thật và phép so hành vi. `epics.md` Story 1.3 đã mang sẵn AC nhận bàn giao này (*"AC hai nền tảng của Story 1.2 … được cưỡng chế bằng CI"*). Ghi vào Completion Notes rằng AC6 **đóng một nửa ở đây, nửa còn lại ở 1.3** — đúng khuôn Story 1.1 đã bàn giao phép đo `.msi`. ⛔ **Không đánh dấu AC6 là đạt trọn nếu chưa có bản Windows chạy thật.**

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

`claude-opus-5` (Claude Code, workflow `bmad-dev-story`), 2026-08-03.

### Debug Log References

**Phiên bản toolchain — số đo dưới đây chỉ lặp lại được trên đúng bộ này:**

| | |
|---|---|
| Hệ điều hành | macOS 15.7.7 (build 24G720), Darwin 24.6.0, x86_64 |
| `rustc` / `cargo` | 1.97.1 (edition 2024 cần ≥ 1.85 ✓) |
| Node / npm | v22.22.2 / 10.9.7 |
| `@tauri-apps/cli` | 2.11.4 |
| Target Rust đã cài | `x86_64-apple-darwin`, `x86_64-pc-windows-msvc` |
| `Cargo.lock` | 579 gói · `package-lock.json` 59 gói |

**Lệnh chạy lại ba phép kiểm — Story 1.3 chép đúng ba dòng này vào workflow:**

```bash
npm run check:deps     # Kiểm 1 + Kiểm 2 — scripts/check-deps.sh   (13 phép kiểm)
npm run check:scope    # Kiểm 3          — scripts/check-scope.sh  (2 chiều)
cargo test --manifest-path src-tauri/Cargo.toml   # 9 test bất biến cấu hình
```

Cả ba trả **mã thoát khác 0 khi thất bại** — đã kiểm chứng bằng cách phá cấu hình rồi khôi phục, không phải giả định:

| Phép kiểm | Cách phá để thử | Kết quả đỏ | Kết quả xanh |
|---|---|---|---|
| `cargo test` | `csp` → chuỗi có `https://cdn.example.com` + `'unsafe-inline'`; scope thêm `$APPDATA/**` | **4/9 FAILED** đúng bốn test tương ứng | 9/9 ok |
| `check-deps.sh` | tạo `node_modules/@tauri-apps/plugin-fs/` | `FAIL`, **exit 1** | exit 0 |
| `check-scope.sh` | `assetProtocol.scope` thêm `/etc/**` | `LEAK — read 4135 bytes`, **exit 1** | exit 0 |

**Build:** `CI=true npx tauri build --bundles dmg` → `Finished release profile in 5m 01s`, ra `AuraTranslate_0.1.0_x64.dmg` = **22.944.022 byte (22,94 MB)**. Bẫy #1 của Story 1.1 (`bundle_dmg.sh` chết ở AppleScript) **không phát sinh** nhờ `CI=true`.

### Completion Notes List

#### Bốn thứ trong Dev Notes hoá ra sai khi chạm vào thật

Ghi trước, vì đây là phần một người rà soát cần nhất — và ba trong bốn cái đều **hỏng im lặng**.

1. **⛔ Công thức nghiệm thu AC5 trong §Nghiệm thu AC5 SAI HAI CHỖ, và cả hai đều cho ra số vô nghĩa mà trông như số thật.**
   - `pgrep -n AuraTranslate` trả **rỗng**. Tên tiến trình lấy từ `package.name` của Cargo (**`auratranslate`**, chữ thường), không từ `productName`; `CFBundleExecutable` = `auratranslate`. `pgrep` phân biệt hoa thường.
   - Nặng hơn: **`lsof -nP -p "$PID" -iTCP -iUDP` là phép HOẶC, không phải VÀ.** lsof trả *"file của PID"* **hợp** *"mọi file mạng của mọi tiến trình"*. Kể cả khi PID đúng, lượt đo đầu cho ra **274 socket, 234 'ra ngoài'** — của Lark, AnyDesk, `ssh`, Affinity… **Phải có cờ `-a`.**
   - Công thức đúng: `PID=$(pgrep -x auratranslate | tail -n1)` rồi `lsof -nP -a -p "$PID" -iTCP -iUDP`. **Story 1.3 phải dùng bản đã sửa** — bản trong story sẽ cho CI đỏ vĩnh viễn vì báo động giả.
   - Script đã thêm chốt: không tìm được PID thì **dừng ngay**, không đo tiếp.

2. **`tauri.conf.json` KHÔNG mang được chú thích.** Task 5 yêu cầu *"ghi một chú thích ngay cạnh khối `security`"*. Thử field `_comment_security` → `tauri-build` gãy: *"unknown field `_comment_security`"* — Tauri từ chối mọi field lạ. Chú thích chuyển sang **`src-tauri/SECURITY-NOTES.md`, nằm ngay cạnh `tauri.conf.json`**, và được **neo bằng test** `src-tauri/tests/config_invariants.rs`: sửa `security` mà quên đọc thì test đỏ. Chú thích không cưỡng chế được gì; test thì có.

3. **`cargo check --target x86_64-pc-windows-msvc` KHÔNG chạy được trên máy này** — xem mục AC6 riêng bên dưới. Target Rust đã cài, nhưng ba crate build native C thì không.

4. **Ghi chú `keyring` trong bản nháp `Cargo.toml` của tôi sai và đã sửa tại chỗ.** Tôi viết *"mặc định là kho giả"*. Đọc `[features]` của chính crate: `default = ["v1"]`, và `v1` đã kéo sẵn `apple-native-keyring-store/keychain` + `windows-native-keyring-store` + `zbus-secret-service-keyring-store`. Không cần khai feature tay, không có nguy cơ rơi vào kho giả. Story 4.3 dùng được ngay.

#### AC1 — cây nguồn

25/25 đường dẫn bắt buộc tồn tại; **0 thiếu**. **15 tệp `mod.rs`** (12 module `core/` gồm `i18n`, cộng `core/` `commands/` `ports/`), tất cả khai trong `lib.rs`/`core/mod.rs` → cây nguồn là thứ **`cargo check` xác nhận**, không phải thứ mắt người soát. Không dùng `.gitkeep` phía Rust.

**Không starter template nào được dùng.** Không chạy `create-tauri-app`, không copy app thăm dò của Story 1.1. Mọi tệp gõ tay. *(Ngoại lệ duy nhất: bộ icon sinh bằng `npx tauri icon` từ một PNG tôi tự dựng — xem mục icon bên dưới.)*

#### AC2 — phiên bản, và số THẬT đọc từ lock

Đọc từ `Cargo.lock` / `package-lock.json` sau khi cài, **không chép lại số trong Dev Notes**:

| Rust | Ghim | Lock | | npm | Ghim | Lock |
|---|---|---|---|---|---|---|
| `tauri` | 2.11.5 | **2.11.5** | | `vue` | 3.5.40 | **3.5.40** |
| `tauri-build` | 2 | **2.6.3** | | `vite` | 8.2.0 | **8.2.0** |
| `serde` | 1 | **1.0.229** | | `typescript` | 5.9.3 | **5.9.3** |
| `serde_json` | 1 | **1.0.151** | | `dockview-vue` | 7.0.4 | **7.0.4** |
| `rusqlite` | 0.40.1 | **0.40.1** | | `@tauri-apps/cli` | 2.11.4 | **2.11.4** |
| `libsqlite3-sys` | 0.38.1 | **0.38.1** | | `@tauri-apps/api` | 2.11.1 | **2.11.1** |
| `jieba-rs` | 0.10.3 | **0.10.3** | | `@vitejs/plugin-vue` | 6.0.8 | **6.0.8** |
| `tantivy-stemmers` | 0.4.0 | **0.4.0** | | `vue-tsc` | 3.3.9 | **3.3.9** |
| `docx-rs` | 0.4.22 | **0.4.22** | | | | |
| `keyring` | 4.1.6 | **4.1.6** | | | | |
| `reqwest` | *"mới nhất"* | **0.13.4** ← đã ghi ngược vào bảng Stack | | | | |

Cả hai bẫy phiên bản mà Dev Notes cảnh báo đều **thật và đã tránh được**: `npm i -D typescript` kéo 7.0.2; `@tauri-apps/cli@2.11.5` không tồn tại. Cả `Cargo.lock` lẫn `package-lock.json` đều commit.

**⚠️ `similar` / `dissimilar` — hàng DUY NHẤT của bảng Stack không cài**, đúng như Dev Notes dặn. Nó còn ở bảng Deferred, điều kiện mở lại là Giai đoạn 5 (Story 8.1) sau khi thử cả hai trên bản review thật. Cài một trong hai hôm nay là âm thầm đóng một quyết định kiến trúc đang mở. Đã ghi chú tường minh trong `Cargo.toml` ngay chỗ lẽ ra là hàng đó. Số đã kiểm nếu 8.1 cần: `similar` 3.1.1 · `dissimilar` 1.0.11.

**Kiểm 1 mở rộng thành 10 tên** thay vì 4: ba tên nguyên văn AC2 + `tauri-plugin-fs` (Ice chốt) + `tauri-plugin-sql` + `tauri-plugin-dialog` (cùng hạng lý do AD-1/AD-29/AD-11) + bốn gói npm tương ứng. Tất cả **vắng mặt**.

#### AC3 — phạm vi filesystem

⛔ **Không viết rằng "framework đã cưỡng chế mọi truy cập file".** Sự thật chia làm hai:

| Vùng AD-23 | Cưỡng chế bằng | Bằng chứng |
|---|---|---|
| `$RESOURCE/dict/**` · `$RESOURCE/fonts/**` | `assetProtocol.scope` — **framework cưỡng chế thật** | Đo được: trong scope nạp OK, `/etc/hosts` trả **HTTP 403** |
| `$APPDATA/**` | kỷ luật mã Rust (AD-7, AD-11) | **Nghiệm thu bằng VẮNG MẶT bề mặt** — không plugin `fs`/`dialog`/`sql`, nên webview không có API filesystem nào để rào |

Capabilities canh **bề mặt IPC tức webview**, không canh Rust: `std::fs` và `rusqlite::Connection::open` không đi qua capabilities. Phát biểu *"chỉ ba vùng này chạm tới"* đúng, nhưng đúng nhờ **vắng mặt bề mặt**, không nhờ một dòng khai báo. Toàn bộ lời văn ở `src-tauri/SECURITY-NOTES.md`, neo bằng test.

`capabilities/main.json` khai đúng `["core:default"]` — không quyền plugin nào. Test `main_capability_grants_the_minimum_and_no_plugin_permission` đỏ nếu ai thêm.

#### AC4 — CSP, và một quyết định đã cân chứ không chép

**`style-src` hạ từ `'self' 'unsafe-inline'` (bản Story 1.1) xuống `'self'`, và kiểm chứng trên bản build RELEASE — không có gì bị chặn.** Đúng như Dev Notes dự đoán: Vite tách `<style scoped>` của SFC ra tệp CSS thật (`dist/assets/index-*.css`, 0,16 kB) và `:style` binding ghi qua CSSOM. App thăm dò một tệp của Story 1.1 không đại diện cho cây frontend thật — nên **không chép nguyên**.

Chuỗi CSP cuối cùng:

```
default-src 'self'; script-src 'self'; style-src 'self';
font-src 'self' asset: http://asset.localhost;
img-src 'self' asset: http://asset.localhost data:
```

`asset:` và `http://asset.localhost` giữ nguyên và **không phải nới CSP theo nghĩa AD-15 cấm** — AD-15 cấm origin **từ xa**. `http://asset.localhost` là dạng của Windows; bỏ nó thì macOS vẫn chạy và Windows hỏng. Không có `devCsp`. Không `csp: null`.

#### AC5 — không một lời gọi ra ngoài nào

**§Quan sát mạng — số, không phải "không thấy gì".**

Chạy trên bản release lấy ra từ `.dmg` (không phải bản dev — bản dev có Vite dev server ở `127.0.0.1:1420`, cục bộ nhưng gây nhiễu). Trọn một phiên: mở → chạm phím vào cửa sổ ở giữa phiên → đóng.

```
PID=15284  tiến trình='auratranslate'
lsof -nP -a -p $PID -iTCP -iUDP   × 18 mẫu, cách nhau 10 s

quan sát 18 lần trong 3 phút
tổng số socket của tiến trình : 0
  loopback                    : 0
  ra ngoài                    : 0
```

**18/18 mẫu cho `socket=0`.** Mạnh hơn mức Dev Notes mong đợi: Dev Notes dự tính *"K kết nối, tất cả tới 127.0.0.1"*, nhưng bản release **không mở socket nào cả** — kể cả loopback, vì frontend đã đóng gói và không còn dev server.

**⚠️ `reqwest` CÓ trong cây phụ thuộc, và đó KHÔNG phải vi phạm AC5.** Cài trọn bảng Stack nghĩa là nó nằm đó nhưng **chưa một dòng mã nào gọi tới**. AC5 nói *"không có **lời gọi** ra ngoài nào"*. Ba điểm ra mạng của AD-15 mở ở Story 4.x (`TranslationProvider`), 6.7 (`Fetcher`), 10.7 (kiểm tra phiên bản) — **không có điểm thứ tư**. Ghi chú này in ra ở cuối mỗi lượt `check-deps.sh` để người rà soát sau không hiểu nhầm.

Quét crash reporter/analytics trên **cả hai cây**: Rust **343 mục**, npm **59 mục**, **0 hit**. Mẫu quét phân biệt `segment-io` với `segment` (module `core/segment/` của chính dự án).

**NFR13 (không tài khoản, không đăng nhập, không đồng bộ đám mây) đóng ở đây, nghiệm thu bằng VẮNG MẶT**: không màn hình đăng nhập, không SDK auth, không client đồng bộ. Đã thêm hẳn một phép kiểm riêng vào `check-deps.sh` quét `auth0|okta|firebase-auth|supabase|clerk|cognito|dropbox|googleapis|onedrive|icloud` trên cả hai cây → **0 hit**.

#### AC6 — ⛔ KHÔNG đạt trọn, và ít hơn cả mức story dự tính

Story dự tính *"đóng một nửa ở đây (tầng biên dịch), nửa còn lại ở 1.3"*. **Nửa tầng biên dịch cũng không đóng được.**

`cargo check --target x86_64-pc-windows-msvc` **gãy**, và **không phải vì mã của dự án**. Ba crate trong cây phải biên dịch mã C cho MSVC:

| Crate | Ai kéo vào |
|---|---|
| `zstd-sys` 2.0.16 | `jieba-rs` → `include-flate` → `zstd` |
| `libsqlite3-sys` 0.38.1 | `rusqlite` feature `bundled` |
| `aws-lc-sys` 0.43.0 | `reqwest` → `rustls` |

Lỗi là `cc-rs: command did not execute successfully … "cc" … --target=x86_64-pc-windows-msvc`. Máy **không có** `cl.exe`, `clang-cl` hay `lld-link`. Đây là **rào ở tầng biên dịch C**, cùng hình dạng với rào WiX/`.msi` mà Story 1.1 đâm phải ở tầng đóng gói — *target Rust đã cài không có nghĩa là cross-compile được*. Dev Notes viết *"✅ chạy được, target đã cài"* là **suy từ `rustup target list`, không phải từ một lượt chạy thật**.

**Bằng chứng cho Windows thật sự lấy được, ghi ra để không ai tưởng là bằng 0:**

- `cargo metadata --filter-platform x86_64-pc-windows-msvc` **OK** — cây phụ thuộc phân giải trọn vẹn cho Windows: **346 crate** (macOS 343), có mặt `webview2-com` 0.38.2, `webview2-com-sys`, `windows-core` 0.61.2, `windows-sys` 0.59.0/0.61.2. Không phụ thuộc nào là macOS-only.
- Mã nguồn của dự án **không có** `cfg(target_os)`, `cfg(windows)`, `cfg(unix)`, không đường dẫn viết cứng (`/Users/`, `C:\`, `Application Support`) — kiểm bằng `grep` trên `src-tauri/src` và `src-tauri/tests`.
- `bundle.windows.webviewInstallMode = downloadBootstrapper` đã khai (Story 1.1 cảnh báo `embedBootstrapper`/`offlineInstaller` một mình đủ làm `.msi` phình ~150 MB và vỡ NFR6).

**Bàn giao sang Story 1.3 — nay là BA việc, không phải hai:**

1. Bản build Windows thật (`.msi`) và phép so hành vi hai nền tảng.
2. Hai phép đo dung lượng `.msi` (có font / không font) + chế độ cài WebView2 *(đã bàn giao từ Story 1.1)*.
3. **MỚI: chính `cargo check` cho Windows.** Trên runner Windows nó là `cargo check` bình thường. **Ice chốt 2026-08-03: bàn giao, KHÔNG thêm `cargo-xwin`** — công cụ đó nằm ngoài bảng Stack, kéo theo MSVC SDK của Microsoft, và vẫn không thay được bản Windows chạy thật mà AC6 đòi.

**Trạng thái AC6:** ⛔ **không đạt trọn, và đạt ít hơn mức story dự tính.** Ghi đúng như vậy, không ghi "đạt phần biên dịch".

#### Task 9 — font, và UX-DR4

Bốn tệp vào `src-tauri/resources/fonts/`, **SHA-256 đối chiếu từng tệp với `font-spike-results-2026-08-03.md §Phép đo 5` trước khi commit: 4/4 `OK`**. Tổng **27.253.184 byte**, khớp đúng con số bắt buộc → không lấy nhầm `NotoSerifTC` (bản subset 45 MB) thay `NotoSerifCJKtc`.

Ba tệp `LICENSE` gốc đi kèm (`LICENSE-notoserifcjk.txt`, `OFL-sourceserif4.txt`, `OFL-sourcesans3.txt`) — FR38, FR109. **Đã xác minh cả bảy tệp có mặt trong `.app` đã đóng gói** (`Contents/Resources/fonts/`), không chỉ trong repo.

`.gitignore` không chặn `.otf`/`.ttf` — kiểm bằng `git check-ignore` (rỗng) và `git add -n` (8/8 tệp vào staging). Không giả định.

`bundle.resources` khai **dạng map** `{"resources/fonts/*": "fonts/"}`, không phải dạng mảng: dạng mảng giữ nguyên cấu trúc đường dẫn nên `resolveResource('fonts/…')` sẽ trượt. **Chưa** khai cho `resources/dict/` — thư mục đó còn rỗng, và glob không khớp tệp nào có thể làm `tauri build` gãy.

**UX-DR4 đóng ở story này.** Ice chốt 2026-08-03 **không sửa `epics.md`** — giữ đúng tiền lệ Story 1.1 (sửa `epics.md` là ngoài phạm vi mặc định của `dev-story`). Ghi nhận nằm ở story file này và bảng Stack. Story này **không** dựng `@font-face` hay token typography — đó là Story 1.4.

#### Task 7 — rà giấy phép: bốn hàng phải phân xử bằng mắt

19 hàng phần mềm rà bằng cách **mở tệp trong nguồn đã tải**. **15 khớp nhãn.** Bốn hàng còn lại, và bài học lặp lại tiền lệ `source-han-serif` của Story 1.1:

- **`tantivy-stemmers` 0.4.0 — suýt chấm SAI.** Tệp `LICENSE` dùng gạch đầu dòng thay vì điều khoản đánh số, nên bộ nhận dạng tự động của tôi đọc thành BSD-2. Đọc bằng mắt: mệnh đề *"Neither the name … may be used to endorse"* **có mặt** → đúng **BSD-3-Clause**, nhãn đúng. *(Tệp còn sót placeholder `{{ project }}` chưa thay — lỗi hình thức của thượng nguồn.)* **Bài học mới: lần này chính bộ nhận dạng tự động cũng là dẫn xuất.**
- **`jieba-rs` 0.10.3** — bản `.crate` không kèm `LICENSE` (`license.workspace = true`, tệp thật ở gốc workspace không được đóng gói). `README.md` **trong nguồn đã tải** ghi nguyên văn *"This work is released under the MIT license."* → MIT.
- **`docx-rs` 0.4.22 — bằng chứng YẾU NHẤT cả bảng.** Không `LICENSE`, README không có mục giấy phép. Chỉ có `license = "MIT"` trong `Cargo.toml.orig` + một dòng header trong `src/xml_json/mod.rs`. Nếu Giai đoạn 5 đổi sang `docx-reader`/`rdocx` thì rà lại từ đầu.
- **`dockview-vue` 7.0.4** — gói npm không kèm tệp; `dockview-core` 7.0.4 mang banner `@license MIT` **nhúng trong bundle đã phát hành** → MIT.

Bảy hàng mới đã thêm vào bảng Stack (`tauri-build` · `serde` · `serde_json` · `@vitejs/plugin-vue` · `vue-tsc` · `@tauri-apps/api` · `@tauri-apps/cli`), cột `Giấy phép` nay mang dấu **✓ / ⚠️** để không ai tưởng cả bảng cùng một độ chắc. `tauri-plugin-fs` (+ `sql`, `dialog`) đã vào danh sách *"Không dùng, đã loại có lý do"*. Bốn dòng ghi vào `.memlog.md` của architecture.

#### Ba việc phải làm mà story không lường trước

1. **Icon.** `tauri build` cần bộ icon, story không nhắc. Tôi dựng một PNG 1024px **tạm** (nền mực đậm + chữ `A` dựng bằng chính `Source Serif 4` của dự án) rồi `npx tauri icon`. Đã xoá thư mục `android/`/`ios/` mà lệnh sinh ra — dự án là desktop hai nền tảng (AD-24). **Đây là icon tạm, không phải nhận diện thương hiệu; chưa story nào sở hữu việc đó.**
2. **`tsconfig` project reference gãy.** `tsconfig.node.json` có `composite: true` + `noEmit: true` → `TS6310: Referenced project may not disable emit` với TypeScript 5.9.3. Bỏ `references`/`composite`, kiểm hai config bằng hai lượt `vue-tsc` trong script `build`.
3. **`tauri dev` NUỐT mã thoát của ứng dụng.** Kiểm 3 ban đầu để Rust gọi `app.exit(1)`; app thoát 1 nhưng `npm run check:scope` vẫn trả **0**. Đó đúng là *"script in ra cảnh báo rồi trả 0"* mà §Testing standards cấm. Nên `scripts/check-scope.sh` đọc dòng `VERDICT:` từ log và **tự quyết mã thoát** — kèm nhánh *"không tìm thấy VERDICT ⇒ exit 1"*, để một lượt chạy chết giữa chừng không đọc thành "đạt".

#### Bàn giao tường minh

| Nhận | Story | Nội dung |
|---|---|---|
| **1.3** | CI | Ba lệnh ở §Debug Log · công thức AC5 **đã sửa** (`pgrep -x auratranslate` + `lsof -a`) · build Windows thật · hai phép đo `.msi` · `cargo check` cho Windows |
| **1.4** | token | `@font-face` từ `$RESOURCE/fonts/**`; `Source Sans 3` phải dựng ở 400/600/700 rồi mới coi token `ui-label` là đã kiểm; khai `FontFace` với `{ weight: "200 900" }` |
| **1.5** | i18n | Hình dạng thật của danh mục `message_key` trong `core::i18n` (enum? hằng? sinh mã từ `vi.json`?) |
| **1.7** | store | `rusqlite` + `libsqlite3-sys` đã ghim, chưa dùng |
| **1.9 / 10.1** | dict | `dict-manifest.toml` khung rỗng; `tools/dict-build/` chưa là crate |
| **4.1** | ai | Test cưỡng chế ranh giới AD-13 |
| **4.3** | keyring | `keyring` 4.1.6 đã kéo sẵn backend gốc ba nền tảng qua feature `v1` |
| **8.1** | diff | Chốt `similar` vs `dissimilar` |

#### Một ghi chú về NFR16 để người rà soát phủ quyết được rẻ

Không có chuỗi tiếng Việt nào là **văn bản hiển thị** trong `.rs` hay `.vue`. `App.vue` không hiển thị chữ nào; self-check in tiếng Anh. **Nhưng** thông điệp assert trong `src-tauri/tests/config_invariants.rs` viết tiếng Việt. Tôi xếp chúng cùng hạng với chú thích mã — chẩn đoán cho người dựng, không bao giờ vượt ranh giới IPC, không bao giờ render ra giao diện. Nếu Ice đọc AD-21 chặt hơn thì đây là chỗ sửa, và sửa rẻ.

### File List

**Mới — mã sản phẩm:**

- `package.json` · `package-lock.json`
- `index.html` · `vite.config.ts` · `tsconfig.json` · `tsconfig.node.json` · `env.d.ts`
- `dict-manifest.toml`
- `src/main.ts` · `src/App.vue`
- `src/i18n/vi.json`
- `src/selftest/scopeCheck.ts`
- `src/modes/{.gitkeep,README.md}` · `src/panels/{.gitkeep,README.md}` · `src/layout/{.gitkeep,README.md}` · `src/commands/{.gitkeep,README.md}` · `src/tokens/{.gitkeep,README.md}`
- `src-tauri/Cargo.toml` · `src-tauri/Cargo.lock` · `src-tauri/build.rs` · `src-tauri/tauri.conf.json`
- `src-tauri/SECURITY-NOTES.md`
- `src-tauri/capabilities/main.json`
- `src-tauri/src/main.rs` · `src-tauri/src/lib.rs`
- `src-tauri/src/commands/mod.rs` · `src-tauri/src/ports/mod.rs` · `src-tauri/src/core/mod.rs`
- `src-tauri/src/core/{segment,matching,glossary,tm,dict,library,export,webimport,ai,scope,store,i18n}/mod.rs` *(12 tệp)*
- `src-tauri/tests/config_invariants.rs`
- `src-tauri/resources/dict/{.gitkeep,README.md}`
- `src-tauri/resources/fonts/README.md`
- `src-tauri/resources/fonts/NotoSerifCJKtc-Regular.otf` · `SourceSerif4[opsz,wght].ttf` · `SourceSerif4-Italic[opsz,wght].ttf` · `SourceSans3[wght].ttf`
- `src-tauri/resources/fonts/LICENSE-notoserifcjk.txt` · `OFL-sourceserif4.txt` · `OFL-sourcesans3.txt`
- `src-tauri/icons/*` *(icon tạm sinh bằng `npx tauri icon`; `source-icon.png` là bản gốc 1024px)*
- `scripts/check-deps.sh` · `scripts/check-scope.sh`
- `tools/dict-build/README.md`

**Sửa:**

- `.gitignore` — thêm `/src-tauri/target/`, `/src-tauri/gen/schemas/`, `*.tsbuildinfo`; ghi chú giữ dòng `*.db` (AD-25)
- `_bmad-output/planning-artifacts/architecture/architecture-AuraTranslate-2026-08-02/ARCHITECTURE-SPINE.md` — bảng Stack: 7 hàng mới, `reqwest` → 0.13.4, TypeScript → 5.9.3, cột giấy phép mang dấu ✓/⚠️; §rà NFR15 lượt hai; `tauri-plugin-fs`/`sql`/`dialog` vào danh sách "Không dùng"
- `_bmad-output/planning-artifacts/architecture/architecture-AuraTranslate-2026-08-02/.memlog.md` — 4 dòng
- `_bmad-output/implementation-artifacts/1-2-scaffold-du-an-va-khoa-pham-vi-filesystem-pham-vi-mang.md` — story này
- `_bmad-output/implementation-artifacts/sprint-status.yaml` — `ready-for-dev` → `in-progress` → `review`

**KHÔNG sửa (có chủ ý):** `_bmad-output/planning-artifacts/epics.md` — Ice chốt 2026-08-03, giữ tiền lệ Story 1.1.

**Không vào repo** — nằm trong scratchpad của phiên: `observe-network.sh` · `license_audit.py` · `make_icon.py` · `build.log` · `network.log` · `scope*.log`.

## Change Log

| Ngày | Thay đổi |
|---|---|
| 2026-08-03 | Story được dựng bằng `bmad-create-story`. Phân tích `epics.md` §Story 1.2 + §Additional Requirements · `ARCHITECTURE-SPINE.md` (43 AD, Stack, Structural Seed, Consistency Conventions) · `prd.md` FR104/NFR11–NFR16 · Story 1.1 (bốn bẫy Tauri, tiền lệ bàn giao) · `font-spike-results-2026-08-03.md` (CSP + assetProtocol đã kiểm chứng) · `implementation-readiness-report` (NFR13 đóng ở đây). Kiểm chứng lại toàn bộ phiên bản trên npm/crates.io và toolchain cục bộ ngày 2026-08-03 — phát hiện hai bẫy phiên bản (`typescript` latest = 7.0.2; `@tauri-apps/cli` không có 2.11.5) |
| 2026-08-03 | **Commit mã nguồn đầu tiên của dự án.** Cây nguồn 25/25 đường dẫn, 15 `mod.rs`, không starter template. Cài trọn bảng Stack: `Cargo.lock` 579 gói · `package-lock.json` 59 gói, cả 19 hàng khớp đúng số ghim. Ba hàng rào lên đúng chỗ: `assetProtocol.scope` hai mục (đo được: `/etc/hosts` → HTTP 403), CSP tường minh với **`style-src` hạ xuống `'self'`** và kiểm chứng trên bản release, capabilities đúng `["core:default"]`. Ba phép kiểm cưỡng chế chạy được bằng lệnh và **đã kiểm chứng đỏ/xanh**: 9 test bất biến cấu hình · `check-deps.sh` 13 phép kiểm · `check-scope.sh` hai chiều. `.dmg` 22.944.022 byte. Quan sát mạng bản release: **18/18 mẫu, 0 socket**. Bốn tệp font + ba `LICENSE` vào repo, SHA-256 4/4 khớp, tổng 27.253.184 byte đúng con số bắt buộc — UX-DR4 đóng. Rà NFR15 19 hàng bằng cách mở tệp: 15 khớp, 4 phải phân xử bằng mắt |
| 2026-08-03 | **Bốn thứ trong Dev Notes sai khi chạm vào thật, đã ghi thành mục riêng thay vì sửa lặng.** (1) Công thức nghiệm thu AC5 sai **hai chỗ**: `pgrep -n AuraTranslate` trả rỗng vì tên tiến trình là `auratranslate` (từ `package.name`, không từ `productName`); và `lsof -p PID -iTCP -iUDP` là phép **HOẶC** — thiếu cờ `-a` nên lượt đo đầu cho ra 274 socket / 234 "ra ngoài" của Lark, AnyDesk, `ssh`. Story 1.3 phải dùng bản đã sửa. (2) `tauri.conf.json` **không mang được chú thích** — `tauri-build` từ chối field lạ; chú thích chuyển sang `SECURITY-NOTES.md` cạnh nó và **neo bằng test**. (3) `cargo check --target x86_64-pc-windows-msvc` **không chạy được**: ba crate build native C (`zstd-sys` qua `jieba-rs`, `libsqlite3-sys`, `aws-lc-sys`) cần toolchain C của MSVC mà máy không có — *target Rust đã cài ≠ cross-compile được*, cùng hình dạng rào WiX của Story 1.1. (4) Ghi chú `keyring` của bản nháp sai: feature mặc định `v1` **đã** kéo backend gốc cả ba nền tảng. Thêm ba việc story không lường: icon, `tsconfig` `TS6310`, và **`tauri dev` nuốt mã thoát** khiến Kiểm 3 phải tự quyết mã thoát từ dòng `VERDICT:` |
| 2026-08-03 | **Ice quyết bốn điểm.** (1) Cài **trọn** bảng Stack — Task 6/7 và §Bảng ghim phiên bản viết lại thành hai bảng đầy đủ Rust + frontend; phát hiện hàng `similar`/`dissimilar` **không cài được** vì còn ở bảng Deferred, ghi thành mục riêng thay vì bỏ im lặng. (2) **Tệp font vào repo ở story này** — Task 9 đổi từ *"chờ xác nhận"* thành nhiệm vụ thật, kèm đối chiếu SHA-256 và ba tệp `LICENSE`; UX-DR4 đóng ở đây. (3) `core/i18n/` **có tạo**, theo bảng Consistency Conventions — thêm §`core/i18n/` là gì nêu rõ nó giữ **danh mục `message_key`**, không giữ văn bản hiển thị; hình dạng thật bàn giao sang Story 1.5. (4) **Bỏ `tauri-plugin-fs`** — soát lại tài liệu xác nhận Ice nhớ đúng: plugin này chỉ có trong báo cáo technical research (bộ `sql · keyring · fs · dialog`), và kiến trúc đã bác từng cái theo cùng một lý do (AD-1 + AD-29). Task 4 và §Phạm vi tĩnh viết lại quanh `assetProtocol.scope` — cơ chế Story 1.1 đã kiểm chứng thật trên bản release; AD-23 tách thành hai nửa với hai cơ chế cưỡng chế khác nhau, nói thẳng nửa `$APPDATA` nghiệm thu bằng vắng mặt |
