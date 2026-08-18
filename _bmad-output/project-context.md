---
project_name: 'AuraTranslate'
user_name: 'Ice'
date: '2026-08-13'
sections_completed:
  [
    'technology_stack',
    'language_rules',
    'framework_rules',
    'testing_rules',
    'code_quality',
    'workflow_rules',
    'critical_rules',
  ]
status: 'complete'
rule_count: 131
optimized_for_llm: true
existing_patterns_found: 7
---

# Project Context for AI Agents

_Tệp này chứa các luật và khuôn mẫu bắt buộc mà agent phải theo khi viết mã trong dự án. Trọng tâm là những chi tiết KHÔNG hiển nhiên — thứ một agent dễ bỏ sót._

---

## Technology Stack & Versions

### Luật chép/trỏ của tệp này

**Chép cái không máy nào trả lời được; trỏ cái máy trả lời được.** Số phiên bản trần đọc
thẳng từ `src-tauri/Cargo.toml` · `package.json` · `dict-manifest.toml` — không chép sang
đây, một bản chép sẽ lệch trong im lặng. Thứ **ở lại đây** là ràng buộc mà không tệp
manifest nào nói ra. Bảng Stack đầy đủ kèm giấy phép: `ARCHITECTURE-SPINE.md` §Stack.

Rust edition 2024 · Vue 3 · TypeScript 5 · Vite · Tauri v2 · SQLite qua `libsqlite3-sys`
(feature `bundled`) · Node 22.

### Bảng phiên bản có HAI nửa — nửa dữ liệu thường bị bỏ quên

**Nửa mã nguồn** — `Cargo.toml` + `package.json`, đọc là ra.

**Nửa dữ liệu** — `dict-manifest.toml`, và nó mang một luật riêng:

- **Đổi lược đồ ⇒ dựng lại CẢ BỐN tệp `.db` bằng `--layer all` ⇒ bốn SHA-256 mới trong
  manifest ⇒ một release mới.** Đúng cả khi nguồn thô của một lớp KHÔNG đổi một byte —
  lược đồ đổi thì nhị phân đổi. *(Story 1.10c: thêm một cột NULL làm
  `dict-thieu-chuu.db` +8.192 byte. `SCHEMA_VERSION` nay là **v3**, `builder_version` 0.4.0.)*
- **`npm run check:dict-manifest` chỉ kiểm HÌNH DẠNG** — nó phải xanh trên một runner
  không có byte dữ liệu từ điển nào, nên nó **không bao giờ mở tệp `.db`**. Nó bắt được
  một lớp bị **rơi mất** (đòi đúng 3 `[[detachable]]`, đúng tên); nó **không** bắt được
  dữ liệu bị trộn giữa các tệp.
- Ba trường bắt buộc mỗi mục: `url` · `sha256` · `source_version` *(phiên bản NGUỒN THÔ,
  không phải phiên bản tệp `.db`)*. Không điền giá trị giả để "cho có".

### 🔴 Ràng buộc có ngòi nổ PHÁP LÝ, và không cổng nào canh nó

**`dict-tran-van-chanh.db` phải ở lại một tệp `.db` RIÊNG.** Trần Văn Chánh (1999) **còn
trong bản quyền — tác giả còn sống**; giấy CC0 của người số hoá không xoá được bản quyền
tác phẩm gốc. Lớp này được đóng gói rời **chính vì** rủi ro đó: FR112 thực thi được bằng
cách xoá đúng một tệp, không đụng ba tệp kia.

⇒ Không gộp lớp, không "hợp nhất cho gọn", không đưa dữ liệu của nó vào `dict-core.db`.
Vi phạm luật này **đi qua sạch cả mười một cổng** và không cho một lượt CI đỏ nào.

### Ràng buộc phiên bản KHÔNG đọc được từ tệp manifest

- **Crate ghim bằng `=`.** `"2.6.3"` trong Cargo NGHĨA LÀ `^2.6.3` — một dải rộng. Lock
  chỉ giữ số đúng tới lần `cargo update` đầu tiên; `=` giữ mãi.
- **npm ghim SỐ TRẦN, không caret**, và CI chạy **`npm ci`, KHÔNG `npm install`** —
  `ci.yml:101` gọi đó là *"hình dạng cưỡng chế được của NFR15"*. Nửa Rust là
  **`cargo test --locked`**.
- **HAI `rust-version`, cố ý lệch:** `src-tauri` khai **1.85**; `tools/dict-build` khai
  **1.97.1** (khớp toolchain CI thật). Hai workspace tách rời — đừng "đồng bộ" chúng.
- **CI ghim `dtolnay/rust-toolchain@1.97.1`, KHÔNG `@stable`** — toolchain trôi làm mọi
  số đo hiệu năng đã ghi hết so sánh được.
- **`@tauri-apps/cli` 2.11.4 ≠ crate `tauri` 2.11.5** — cố ý lệch, không phải quên đồng bộ.
- **`[profile.release]` đóng băng** (`codegen-units = 1` · `lto` · `opt-level = "s"` ·
  `panic = "abort"` · `strip`). Đổi là làm số đo NFR6 hết so sánh được — và `panic =
  "abort"` là tiền đề của luật "không `unwrap()` ở luồng writer".
- **`[features]` KHÔNG có `default = [...]`, đừng thêm.** Bộ mặc định rỗng là thứ giữ
  `axum` + `tauri-plugin-wdio-webdriver` khỏi `cargo tree`, tức khỏi bản phát hành
  (AD-45). Đo: cây mặc định 831 dòng · `--features wdio` 948.
- **Sàn SQLite:** FTS5 `trigram` (≥ 3.34) và `remove_diacritics 0` (≥ 3.27). Đến từ
  crate, không từ SQLite của hệ điều hành.
- **Node ≥ 22.18** — ba phép kiểm của `check-i18n.mjs` dựa vào việc Node bóc kiểu
  TypeScript mặc định.
- **Font: chỉ `Source Sans 3` khai Reserved Font Name `'Source'`** — subset riêng tệp đó
  thì BẮT BUỘC đổi tên font nội bộ; hai họ kia không khai nên subset thoải mái.
  `Source Serif 4` có **hai** tệp trên đĩa (roman + italic).

### Cửa bắt buộc trước khi thêm BẤT KỲ phụ thuộc nào (NFR15)

1. **Mở tệp giấy phép trong nguồn ĐÃ TẢI mà đọc** (`~/.cargo/registry/src/…`,
   `node_modules/…`) — không tin nhãn registry, không tin trường `license`. *(`vitest`
   khai `"MIT"` nhưng `LICENSE.md` dài 811 dòng, gộp giấy phép của 27 gói nó vendor.)*
2. **Ghi vào bảng Stack của spine TRƯỚC khi thêm.** Ba lượt rà đầu của dự án đều là lượt
   "đuổi theo" — lỗi đã lặp ba lần, không phải một khả năng lý thuyết.
3. Chỉ giấy phép tương thích **GPLv3 theo chiều đi vào** (MIT · Apache-2.0 · BSD-2/3 ·
   ISC · OFL 1.1). Dự án là GPL-3.0-or-later.

### Sáu tên bị CẤM, cưỡng chế bằng `npm run check:deps`

`tauri-plugin-fs` · `tauri-plugin-dialog` · `tauri-plugin-sql` · `tauri-plugin-keyring` ·
`tauri-plugin-stronghold` · `tauri-wire`.

Lý do chung: một Tauri plugin tồn tại để **phơi API ra JavaScript**, mà frontend chỉ được
render và giữ state UI (AD-1). Dùng thay: `rusqlite` trực tiếp (AD-11), crate `keyring`
trực tiếp (AD-29). `tauri-plugin-wdio-webdriver` là ngoại lệ DUY NHẤT, đi qua hai lớp
chặn của AD-45.

### Hai crate CỐ Ý chưa cài

`similar` **hoặc** `dissimilar` cho Diff Viewer — `Cargo.toml:86-89` ghi sẵn cả hai số
(3.1.1 / 1.0.11) và **không cài cái nào**. Cài một trong hai hôm nay là âm thầm đóng một
quyết định kiến trúc đang mở (chốt ở Giai đoạn 5, sau khi thử cả hai trên bản review thật).

## Critical Implementation Rules

### Language-Specific Rules — Rust

- **Lỗi là GIÁ TRỊ, và `panic = "abort"` biến mọi `panic!` thành cái chết của tiến
  trình** — không unwind, không `Drop`, không cơ hội flush WAL; trên Windows release còn
  không in ra đâu. `catch_unwind` **vô dụng** ở đây. Mọi `unwrap()`/`expect()` trong
  `core/store/**` là một **lỗi thiết kế**, không phải lối tắt. Mutex khoá bằng
  `lock().unwrap_or_else(|e| e.into_inner())`; kênh phản hồi gửi bằng `let _ = tx.send(…)`.
- **Dựng lỗi IPC CHỈ qua `IpcError::new(code, message_key, params, retryable)`.** Bốn
  trường là riêng tư. Một struct literal đi vòng qua nó biên dịch sạch, qua mọi cổng —
  rồi đặt nguyên văn `{path}` lên màn hình người dùng. `new` là chỗ DUY NHẤT
  `message_key` gặp `params`.
- **`message_key` là danh mục ĐÓNG**, khai bằng `macro_rules! message_keys!` trong
  `core/i18n/`. Một khai báo sinh ra ba thứ (`enum` · `ALL` · `as_str`) **cộng bảng tham
  số bắt buộc**. Đừng viết tay một danh sách song song — test đồng bộ với `vi.json` chạy
  TRÊN `ALL`, nên một biến thể quên thêm vào `ALL` cho một test **xanh giả**.
- **Đừng đặt `#[serde(rename_all = "camelCase")]` lên `IpcError`** (thói quen viết Tauri).
  Bốn tên trường là **dây**, `tests/ipc_contract.rs` khoá lại.
- **Không văn bản hiển thị trong Rust — kể cả `impl Display`.** Các `Display` cho lỗi là
  **chẩn đoán cho log, viết KHÔNG DẤU**. `check:i18n` Kiểm A đỏ với chữ tiếng Việt có dấu
  ở **vị trí mã**. *(Comment tiếng Việt thì hoàn toàn được — xem §Code Quality.)*
- **Khuôn hai lớp cho mọi bề mặt IPC** (chốt ở Story 1.8): ① một **hàm thuần** nhận
  `Option<&Store>` — đây là thứ `tests/**` gọi được **không cần webview**; ② một
  `#[tauri::command]` **mỏng** trong module `wire`, chỉ lấy `State` qua **`try_state`**
  rồi gọi xuống lớp ①.
  🔴 `try_state`, **không** `state()`: mở kho có thể đã thất bại và `app.manage()` chưa
  từng chạy ⇒ `state()` panic ⇒ `panic = "abort"` giết cả tiến trình.
- **Tên command trên dây LÀ tên hàm** — nên vỏ sống trong một **module lồng** (`wire`),
  không mang hậu tố.
- Cùng khuôn macro cho bảng ngữ nghĩa hai tầng: `scope_kinds!` ở `core/scope/kinds.rs` —
  **không tồn tại cú pháp khai một loại mà không kèm ngữ nghĩa**.

### Language-Specific Rules — TypeScript / Vue

- 🔴 **`src/i18n/resolve.ts` KHÔNG được `import` bất cứ thứ gì, và không được dùng `enum`,
  `namespace`, hay parameter property (`constructor(private x)`).** Đây là **điều kiện kỹ
  thuật**, không phải gu kiến trúc: `check-i18n.mjs` Kiểm E `import()` thẳng tệp này bằng
  Node trần (bóc kiểu mặc định). Node **chỉ** bóc kiểu — nó không phân giải `./vi.json`
  theo luật bundler và không hiểu `.vue`; ba cấu trúc trên **sinh mã** nên Node từ chối.
  Một dòng `import` ở đây là Kiểm E chết. *(Luật "erasable-only" này áp cho cả
  `src/commands/registry.ts` · `focus.ts` · `index.ts` và `src/layout/writeSchedule.ts` —
  xem §Framework Rules.)*
- **`invoke()` gửi tham số dạng camelCase** dù hàm Rust nhận `snake_case`
  (`tauri-macros` `ArgumentCase::Camel`) ⇒ viết `sourceLang`, không `source_lang`.
  **Nhưng trường của struct TRẢ VỀ giữ nguyên `snake_case`** (`meta_schema_version`,
  `work_id`) — hai chiều khác nhau, đây là chỗ dễ sai nhất trên dây.
- **Adapter IPC ở `src/config/*.ts` KHÔNG BAO GIỜ ném.** Một `invoke`, một `try/catch`,
  trả về hình dạng **ba trạng thái** `{ <giá trị> | null, error: IpcError | null }`. Tầng
  UI hiển thị lỗi bằng `tError()`, không bằng `try/catch`.
- **Luôn kiểm kiểu LÚC CHẠY cho dữ liệu qua dây.** `IpcError` phía TS là một **lời khai**
  về dữ liệu đã đi qua IPC, không phải bảo đảm của trình biên dịch — Rust có thể trả
  `null` cho `params` sau một lượt đổi lược đồ, và type guard là chỗ duy nhất biết.
- **`verbatimModuleSyntax` bật** ⇒ `import type` phải tường minh. **Không `globals: true`
  ở vitest** ⇒ mỗi tệp test `import { describe, it, expect } from 'vitest'`. Cùng một lý
  do: một cái tên xuất hiện từ hư không là một cái tên `vue-tsc` phải được dạy riêng để thấy.
- 🔴 **`Ref` KHÔNG tự bóc trong khối `<script>`** — chỉ trong `template`. `if (someRef)`
  chạy trên **đối tượng** và **luôn đúng**; nó là TypeScript hợp lệ nên `vue-tsc` im.
  Đây là lỗi đã lọt qua **chín trên chín cổng** và là lý do cổng thứ mười (`check:lint`,
  `@typescript-eslint/no-unnecessary-condition` có kiểu) ra đời.
- **`@click` trong `.vue` phải là ĐÚNG MỘT lời gọi `dispatch('<id>')`** — không hàm khác,
  không mã nội tuyến (`check:commands` Kiểm A, AD-34).
- **Không quy tắc nghiệp vụ nào ở TypeScript** (AD-1). Ngoại lệ duy nhất, tường minh: văn
  bản đang gõ trong Editor là state cục bộ frontend.
- **Trong `e2e/**/*.mjs`: cấm `.click()` của driver**, dùng `realClick()` ở
  `e2e/support/pointer.mjs`. Driver bắn `click` **trước** `focusin` — ngược chuột thật —
  nên nó vừa cho ĐỎ sai nguyên nhân, vừa cho **XANH trên một sản phẩm đang hỏng**.
  Cưỡng chế bằng `no-restricted-syntax`.

### Framework-Specific Rules — Tauri v2

- 🔴 **`capabilities/` được phép chứa ĐÚNG MỘT tệp: `main.json`.** Tauri nạp **mọi** tệp
  trong thư mục đó bằng glob `{capabilities}/**/*` — mọi phần mở rộng, **có đệ quy**. Nên
  một `extra.json` với `"permissions": ["fs:default"]` cấp một bề mặt IPC mới mà test đọc
  `main.json` **vẫn xanh**. Cưỡng chế:
  `tests/config_invariants.rs::capabilities_directory_holds_exactly_the_one_reviewed_file`.
- **Tập quyền là TỐI THIỂU THẬT, không phải bundle `core:default`:** `core:path:default`
  (resolveResource) · `core:event:default` (Channel/emit — AD-22) · `core:resources:default`.
  Thêm một quyền là **quyết định kiến trúc**, không phải một dòng cấu hình.
- **CSP giữ nguyên, không nới** — không CDN, không font ngoài, **không ảnh ngoài**. Đây là
  lý do FR127 tải ảnh về `.atproj` thay vì giữ link. `assetProtocol.scope` hôm nay là
  `$RESOURCE/fonts/**` và chỉ thế.
- **Streaming AI đi qua Tauri Channel API**, không qua event rời, **không có client SSE
  tự kết nối lại** (AD-22). Auto-reconnect tạo một yêu cầu mới hoàn toàn ⇒ với BYOK,
  người dùng bị tính phí hai lần. Mọi lời gọi AI phải huỷ được giữa chừng.
- **Không mở cổng LẮNG NGHE nào trong bản phát hành** (AD-45). Công cụ cần máy chủ phải
  đi qua **hai** lớp cùng lúc: `optional = true` + feature ngoài `default`, **và**
  `#[cfg(debug_assertions)]` ở chỗ nối. Một `cfg` một mình **không đủ** — nó loại **mã**,
  không loại **phụ thuộc**.

### Framework-Specific Rules — Vue 3

- **THỨ TỰ KHỞI ĐỘNG trong `src/main.ts` là bắt buộc, cả ba mệnh đề:**
  1. `applyTheme()` **trước** `mount()` — nó ghi token thành CSS custom properties lên
     `documentElement`. Mount trước ⇒ mọi `var(--color-…)` rỗng ở lượt render đầu ⇒ một
     nháy trắng. Trên bản đã đóng gói nháy đó **ngắn hơn máy dev**, nên lỗi chỉ lộ ở máy
     người khác.
  2. `installCommands()` **trước** `mount()` — `dispatch` NÉM với id chưa đăng ký, và
     `App.vue` render tab chế độ với `@click="dispatch('mode.…')"`.
  3. `loadFonts()` khởi động **trước** `await loadBootstrapConfig()` — hai lời gọi không
     phụ thuộc nhau; xếp hàng chúng kéo dài đúng khoảng nháy chữ-hệ-thống.
- **Đăng ký command ở `main.ts`, KHÔNG trong `App.vue`** — một lượt HMR dựng lại
  component sẽ gọi `installCommands()` lần hai và `register()` ném vì id trùng.
- 🔴 **Luật "erasable-only" cho bốn tệp**: `src/commands/{index,registry,focus}.ts` và
  `src/layout/writeSchedule.ts` phải **nạp được bằng Node thuần** — cổng `check:commands`
  (Kiểm C/D/E) và `check:layout` (Kiểm B) `import()` chúng để chạy phép kiểm **HÀNH VI**
  trên chính mã sản phẩm. ⇒ Không `import` giá trị của `vue`/`dockview`; không `enum`,
  `namespace`, parameter property. **Một `import` giá trị ở đó giết ba phép kiểm cùng lúc.**
  Đây là lý do `src/layout/dockController.ts` tồn tại: `main.ts` tiêm hàm vào, không import.
- **Mỗi chế độ và mỗi panel khai điểm vào focus** ở `src/commands/focus.ts`; chuyển panel
  phải **dời focus DOM tường minh**, không để rơi về `body` (AD-34 §2).
- **Phím tắt và Auto-Lookup phát cùng một `dispatch(...)`, không gọi thẳng hàm.** Một lời
  gọi thẳng dựng một đường thứ hai mà `check:commands` **không nhìn thấy** (Kiểm A chỉ
  canh `@click`).
- **Hàm chạy từ một hợp âm bàn phím KHÔNG BAO GIỜ ném — nó KÊU.** Ghi chẩn đoán nêu đích
  danh rồi trả `false`. Và không "vá" bằng cách tự chuyển chế độ: đó là đoán ý người dùng.

### Framework-Specific Rules — dockview

- 🔴 **`onDidLayoutChange` bắn LIÊN TỤC trong lúc kéo sash.** Ghi một `putConfig` mỗi lần
  bắn ⇒ một cú kéo 3 giây là **hàng trăm** job xếp hàng qua `store::Writer` nối tiếp.
  **Không cổng nào đỏ vì chuyện đó**, và biểu hiện lộ ra ở Epic 2 dưới dạng *"gõ bị
  khựng"* mà không ai lần được về dòng nào. ⇒ Mọi nhịp ghi đi qua
  `src/layout/writeSchedule.ts` (idle + trần cứng không reset).
- **Hàm nhịp ghi KHÔNG được tự đọc `Date.now()`** — mọi thời điểm đi vào qua tham số. Một
  hàm tự đọc đồng hồ buộc cổng phải `sleep` thật, tức một phép kiểm chậm và chập chờn.
- **Hai chỗ dùng, HAI cặp hằng, chỉ MỘT mang bảo đảm của AD-35:** bố cục dùng
  `IDLE_MS 500 / HARD_CAP_MS 5000` (**không** mang bảo đảm AD-35); Editor dùng
  `EDITOR_IDLE_MS 2000 / EDITOR_HARD_CAP_MS 5000` (**có**). Dùng chung *hình dạng*, không
  dùng chung *bảo đảm* — đừng gộp hai cặp hằng.
- **Không cửa sổ OS thứ hai** (AD-24): `addPopoutGroup` là đường **duy nhất** trong
  dockview gọi `window.open` — cấm. `check:layout` Kiểm C là một **danh sách CHO PHÉP**
  cho mọi thành viên của `window`/`document` mà `src/**` chạm tới; thêm một cái tên là
  một quyết định phải viết ra.

### Testing Rules

#### Bốn đường nghiệm thu, bốn vai KHÔNG chồng nhau (AC25)

| Đường | Vai | Chạy |
|---|---|---|
| **Cổng tĩnh** `scripts/check-*.mjs` | mệnh đề **khai báo trên TOÀN CÂY** (*"không màu viết thẳng ở bất kỳ đâu"*) | `npm run check:*` |
| **Test Rust** `src-tauri/tests/**` | hợp đồng · ranh giới module · bất biến cấu hình | `cargo test --locked` |
| **vitest** `tests/frontend/**` | hành vi của module thuần, mã đụng DOM, `.vue` | `npm run test` |
| **e2e WebdriverIO** `e2e/**` | hành vi trong **WKWebView/WebView2 THẬT** | `npm run test:e2e` (tay) |
| **Bàn đo chạy tay** | vế thị giác và vế **đo số** trên engine thật | tay |

**Chọn sai đường là dựng nguồn sự thật thứ hai.** Trước khi viết một phép kiểm mới, hỏi:
mệnh đề này đã có chủ chưa? `happy-dom` **không phải WebKit** — mọi mệnh đề về **hình
học**, **bố cục** hay **engine thật** thuộc bàn đo/e2e, không thuộc vitest.

#### Cây test frontend

- **`tests/frontend/**`, KHÔNG đồng vị trí trong `src/**`** — bốn cổng đếm quần thể
  `src/**` và một tệp test đổ vào đó **thổi phồng mẫu số** (cộng hai va chạm: Kiểm A của
  `check-i18n` đỏ với chữ tiếng Việt, Kiểm B của `check-tokens` đỏ với màu viết thẳng).
- **`tsconfig.json` PHẢI `include` cây test** — một cây test không được kiểm kiểu là một
  cây test sẽ mục: nó vẫn chạy xanh trong khi kiểu của thứ nó kiểm đã đổi dưới chân.
- **Mọi vá `happy-dom` sống ở `tests/frontend/support/setup.ts`, mỗi mục kèm một dòng nói
  nó thiếu gì và ai đọc nó.** Danh sách đó là một **món nợ đo được**.
  🔴 **Đường sai rất rẻ và phải chặn bằng tay:** thêm một `?.` vào **mã sản phẩm** cho hết
  đỏ. Đó là một nhánh mà **kiểu nói không bao giờ chạy** — mã chết vĩnh viễn trong sản
  phẩm để phục vụ một bản mô phỏng. Khoảng thiếu của bản mô phỏng vá **ở `setup.ts`**;
  khuyết tật sản phẩm vá **trong `src/`**.
- **Không `vi.useFakeTimers()` khi hàm đã nhận thời điểm qua tham số.** Hàm nhịp không đọc
  `Date.now()` ⇒ kiểm được **tất định và tức thời**; bọc đồng hồ giả là đổi một bảo đảm
  lấy một thói quen.

#### Test Rust

- Hai họ tên tệp, hai vai: **`*_contract.rs`** (hợp đồng: hình dạng dây, bảng đã khai,
  khoá đã đăng ký) và **`*_boundary.rs`** (ranh giới: module nào KHÔNG được mang từ vựng
  của module khác — ví dụ `scope_boundary.rs` cấm mọi module ngoài `core/scope/**` mang
  từ vựng phân giải hai tầng).
- **Tên hàm test là một CÂU khẳng định**, không phải `test_foo`:
  `the_semantics_table_matches_ad_18_row_by_row` · `every_message_key_exists_in_vi_json` ·
  `capabilities_directory_holds_exactly_the_one_reviewed_file`.
- Test gọi được **hàm thuần** của bề mặt IPC **không cần webview** — đó là lý do khuôn hai
  lớp tồn tại.
- **Thông báo `assert!` trong `src-tauri/tests/**` là miễn trừ CÓ TÊN** khỏi Kiểm A của
  `check:i18n` — miễn trừ được khai trong `EXEMPT`, không phải im lặng.

#### Luật của một CỔNG (áp cho mọi `scripts/check-*.mjs`)

- **Mã thoát là phán quyết.** Không có cổng nào ghi log rồi đi tiếp.
- **Mỗi cổng phải có phép TỰ KIỂM chứng minh nó ĐỎ ĐƯỢC — và không đỏ oan.** *(Kiểm D của
  `check-layout`, Kiểm C của `check-gates`.)* Một cổng chưa bao giờ đỏ là một cổng chưa ai
  biết nó có chạy không.
- 🔴 **Lỗi hạ tầng KHÔNG phải một phép kiểm đỏ.** Không đọc được tệp ⇒ `abort()` và thoát
  khác 0 kèm câu *"đây là lỗi hạ tầng, không phải đạt"*. Đừng bao giờ báo một kết quả
  không có thật.
- 🔴 **Không phán quyết nào được đọc tham số từ chính thứ nó đang kiểm.** Sàn WCAG, danh
  sách vai, danh sách loại trừ — đóng băng **trong script**. *(Đã đo: ba đường thoát đều
  cho exit 0 trong khi sản phẩm mang một cặp tương phản 4,245:1.)*
- **Sàn quần thể:** *"cây rỗng không phải cây sạch"*. Cổng đếm số tệp và `abort()` khi
  **dưới sàn**; sàn đặt ở ~80–85 % số thật. Thêm tệp vào `src/**` thì phải xét lại sàn —
  và nhớ sàn là **cận dưới**, nên tệp thừa không làm cổng đỏ, nó chỉ làm sàn vô nghĩa.
- **Node thuần, không bash** — `npm run` trên Windows đi qua `cmd.exe`. Một cổng chỉ canh
  nửa số nền tảng thì không canh được NFR14.
- **Không thêm phụ thuộc npm cho một cổng.** Parser TOML/CSS trong `scripts/` là **tập con
  nghiêm ngặt tự viết**, và cú pháp ngoài tập con ⇒ FAIL, không bỏ qua.
- **Thêm một cổng = sửa BA danh sách** (`package.json` · `.github/workflows/ci.yml` ·
  `.githooks/pre-push`), và `check:gates` canh cả ba. Cổng không mang tiền tố `check:`
  (hôm nay: `test`) phải có mặt ở cả ba — Kiểm F.

#### Hai thứ CỐ Ý nằm ngoài `pre-push`

- **`check:scope` + `check:scope:bundled`** — dựng cửa sổ Tauri thật, cần **cổng 1420
  trống**; chúng trượt nếu đang mở `npm run tauri dev`. Chạy tay.
- **Bộ e2e** — mỗi spec mở một cửa sổ thật (~1,5 phút) **và nó ghi vào `global.db` cùng
  thư mục gốc Library THẬT của người chạy** nếu hai biến môi trường chuyển hướng không
  xuống được tiến trình con. `wdio.conf.mjs` có phép **tự kiểm dương tính** (`global.db`
  phải NẰM trong thư mục tạm) trước khi xoá bất cứ gì. Chạy tay.

#### Luật đo

- **Không đánh dấu đạt bằng suy luận.** Vế nào không nghiệm thu được ở tầng đang làm thì
  ghi vào `deferred-work.md` kèm chủ, không tự chấm đạt.
- **Số đo phải truy nguyên được**: ghi kèm phiên bản toolchain và ngày. *"Số đo không truy
  nguyên được thì không phải số đo."*

### Code Quality & Style Rules

#### Văn hoá chú thích — thứ khác nhất so với một kho thường

- **Chú thích viết bằng tiếng Việt, dày, và chở LÝ DO** — đây là quy ước có chủ ý, không
  phải nợ. Chú thích ở đây không kể mã làm gì; nó trả lời *vì sao hình dạng này chứ không
  phải hình dạng kia*, và **phương án bị loại đã bị loại bằng gì**.
- 🔴 **Một quyết định không hiển nhiên phải kèm một PHÉP ĐO, không một sở thích** — con
  số, ngày đo, và `tệp:dòng` làm bằng chứng. Khuôn có sẵn khắp kho:
  *"⚠️ Đo 2026-08-11: … Hệ quả đo được: …"*
- **Ghi thẳng chỗ YẾU thay vì giấu.** Mọi cổng, mọi module chở một mục *"GIỚI HẠN THẬT,
  ghi ra thay vì để người sau tự phát hiện"*. Một giới hạn không viết ra là một giới hạn
  người sau sẽ tưởng đã được xét.
- **Khi một mệnh đề hết đúng, SỬA TẠI CHỖ thay vì để nó lặng lẽ sai** — kèm ngày và lý do
  đổi. *(Ví dụ: "Mệnh đề 'kho có 0 plugin Tauri' hết đúng từ 2026-08-11 — sửa ở đây…")*
- **Ký hiệu dùng trong chú thích:** 🔴 = luật không được phá / chỗ quyết định · ⚠️ = bẫy,
  giới hạn, hoặc chỗ dễ đọc nhầm · ✅ = đã đóng · 🟡 = đóng một nửa · 🔵 = **cập nhật, một
  mệnh đề cũ đã hết đúng** · ⇒ = kết luận.
- 🔴 **Emoji `U+26D4` (biển cấm) bị CẤM trong toàn kho và trong cả câu trả lời cho Ice.**
  Viết `không` / `KHÔNG` thành chữ. Ice gỡ 8.298 ca ngày 2026-08-07 vì nó mang **ba nghĩa
  trộn lẫn** và ở dạng *"<ký hiệu> phải X"* nó **đảo ngược** nghĩa. *(Tệp này gọi tên nó
  bằng codepoint, có chủ ý.)* Gặp lại ở đâu là dấu hiệu nó đang bò ngược vào — gỡ ngay.
- 🔴 **Đừng bắt chước một ký hiệu chưa hiểu.** Thấy một quy ước lạ lặp lại nhiều: `grep`
  đếm số lần **và tìm định nghĩa** trước khi dùng lại. Không có định nghĩa ⇒ nêu với Ice
  kèm số đo, và trong lúc chờ thì viết chữ thường minh. *"Viết cho giống code xung quanh"*
  không phủ định *"đo trước khi tin"*.

#### Miễn trừ và cảnh báo

- 🔴 **Sửa KIỂU cho nó nói thật; đừng nhét một cảnh báo hay một miễn trừ để cổng hết đỏ.**
  Hạ ngưỡng, chuyển một cặp sang danh sách loại trừ, thêm một `eslint-disable` — cả ba đều
  cho exit 0 trên một sản phẩm đang hỏng.
- **Mọi miễn trừ phải CÓ TÊN, có lý do tại chỗ, và phải chết được.** `eslint.config.js`
  đặt `reportUnusedDisableDirectives: 'error'` vì một miễn trừ hết cần mà ở lại là đúng
  lớp nợ mà cổng tồn tại để chống.
  ⚠️ **Đo được:** lượt dựng 13 miễn trừ đầu tiên của kho **sai hình dạng** —
  `eslint-disable-next-line` đứng trước ba dòng chú thích tiếp nối nên nó trỏ vào một dòng
  **chú thích**, và 13 guard thật vẫn đỏ.

#### Đặt tên

- Rust `snake_case` · Vue `PascalCase.vue` · tài nguyên chuỗi `vi.json` **phẳng**, khoá
  chấm có tiền tố miền (`lookup.empty_result`).
- **Command id dùng CÙNG văn phạm khoá chấm** với khoá i18n (`review.accept_change`) — id
  trần sẽ bị hai giai đoạn cách nhau nhiều tháng đăng ký trùng và ghi đè nhau âm thầm.
- **Ánh xạ thuật ngữ CỐ ĐỊNH sang định danh tiếng Anh trong mã:** Tác phẩm → `Work` ·
  Chương → `Chapter` · Chế độ đọc → `ReadingMode` · Panel Lookup → `LookupPanel` · Smart
  RAG Injector → `RagInjector` · lớp nền/gỡ rời → `BaseLayer`/`DetachableLayer` · Hán Việt
  → `HanViet`. 🔴 **Cấm `Project`, `Book`, `Novel`, `Document` cho `Work`** *(đuôi
  `.atproj` là ngoại lệ lịch sử, không kéo theo tên thực thể)*.
- **Module Rust đặt theo KHÁI NIỆM MIỀN, không theo nhóm năng lực.** `C1`–`C10` là từ vựng
  sản phẩm và **không xuất hiện trong tên module**.
- **Ngày giờ:** lưu ISO-8601 UTC trong database; định dạng hiển thị **chỉ** ở frontend.

#### Chuỗi và token

- **`vi.json` PHẲNG, khoá chấm, không giá trị rỗng.** Placeholder đúng dải
  `{ten_tham_so}` khớp `[a-z_][a-z0-9_]*` — `{}`, `{Path}`, `{0}`, `{ path }` đều là FAIL
  ở cổng, không phải ở tay người dùng.
- **Tham số mang DỮ LIỆU, không mang CÂU.** Định dạng số và ngày là việc của nơi gọi.
- **Chuỗi literal trong `src-tauri/src/**` phải viết KHÔNG DẤU** (`khong`, không `không`)
  — Kiểm A của `check:i18n` cấm chữ có dấu ở vị trí mã. `tests/**` và `tools/dict-build/**`
  được miễn trừ nên giữ dấu.
- **Màu VÀ cỡ chữ chỉ đến từ token** (`check:tokens` Kiểm B + B2). Không bóng đổ, không
  gradient, không lớp nổi (Kiểm F). `opacity` trung gian phải có miễn trừ **có tên** (Kiểm D).

#### Tài liệu trong cây nguồn

- **Thư mục mang một khái niệm thì có `README.md`** — hôm nay: `src/{commands,i18n,layout,
  modes,panels,tokens}` · `src-tauri/resources/{dict,fonts}` · `tools/dict-build`. Thêm
  một khái niệm vào một trong số đó thì cập nhật README cùng lượt.

### Development Workflow Rules

#### Git

- 🔴 **Nhánh mặc định là `master`, KHÔNG phải `main`.** Viết cứng `branches: [main]` trong
  một workflow ⇒ CI **không bao giờ chạy** và **không lỗi nào được ném**.
- **`core.hooksPath = .githooks`.** `pre-push` chạy: chín cổng đọc-tệp → `npm run test`
  (vitest) → `npm run build` → `cargo test --locked`. Đỏ là **dừng**, không phải cảnh báo.
  *(Đo 2026-08-11: cổng 11s · build 5s · cargo test 34s.)*
- **Bỏ qua một lượt: `git push --no-verify` — và phải VIẾT LÝ DO vào commit message.**
- **Commit message: `type(scope): câu tiếng Việt`.** `type` ∈ `feat` · `fix` · `docs` ·
  `test` · `chore` · `ci`; `scope` là vùng thật (`story-2.4`, `e2e`, `tokens`, `gates`,
  `a11y`, `segment`, `repo`). Câu sau dấu hai chấm **nói ĐIỀU ĐÃ TÌM RA**, không chỉ điều
  đã sửa — khuôn có sẵn: *"số đầu tiên — NFR18 không đạt ở ngưỡng WAL mặc định, vì cái đuôi"*.
- **Cây bẩn TRƯỚC khi bắt đầu một story ⇒ commit riêng, trước, và HỎI Ice trước khi
  commit.** Diff của một story phải đọc được một mình.
- 🔴 **Không commit tệp `.db`.** Dòng `*.db` trong `.gitignore` là CỐ Ý (AD-25): dữ liệu từ
  điển đi qua GitHub Release + `dict-manifest.toml`, không qua git.
- **`_bmad/` · `.claude/` · `.agent/` · `.agents/` nằm ngoài index** — công cụ AI không
  phải nội dung kho. *(`_bmad-output/` thì CÓ được theo dõi: story và tài liệu quy hoạch
  là nội dung kho.)*

#### CI

- **Chạy mỗi push + PR + bấm tay**, repo công khai. Đo trên lượt xanh đầu: macOS **6m09s**
  · Windows **29m31s** (job Windows dựng ba biến thể `.msi`, mỗi biến thể một lượt biên
  dịch release riêng).
- **MỘT tệp workflow duy nhất** (`ci.yml`, AC4 của Story 1.3). Mọi luật cưỡng chế mới gắn
  vào chính tệp đó — **không dựng workflow thứ hai**. `.githooks/pre-push` là đường cưỡng
  chế thứ ba **được phép** vì `check:gates` Kiểm D/E buộc nó khớp hai danh sách kia.
- ⚠️ **Nửa Windows hôm nay KHÔNG có đường nghiệm thu tại chỗ** — `pre-push` chạy trên
  macOS của Ice. 263 ca xanh trên runner Windows là **một ảnh chụp**, không phải một trạng
  thái được canh (action item A5 của retro Epic 1).

#### Sổ nợ — `_bmad-output/implementation-artifacts/deferred-work.md`

- **Mọi thứ không nghiệm thu được ở story hiện tại đi vào đây, KÈM MỘT CHỦ** (story nào sẽ
  đóng). Không có mục nào mồ côi.
- **Không bao giờ XOÁ một mục đã đóng.** Đóng bằng cách nối tiếp `→ ✅ ĐÃ ĐÓNG <ngày>
  (Story x.y)` kèm cách đóng; mệnh đề đã hết đúng thì **gạch ngang**, không xoá. Lịch sử
  của một món nợ là bằng chứng cho quyết định kế tiếp.
- **Đóng MỘT NỬA thì ghi 🟡 và liệt kê phần CÒN HỞ**, không làm tròn lên thành ✅.

#### Story và spec

- 🔴 **Năng lực chưa dựng ≠ lệch spec.** Một AC mô tả đích đến **không sai** chỉ vì đường
  đi chưa tới đó. Đừng sửa `epics.md`/`prd.md` cho khớp mã đã viết — ghi một món nợ **có
  chủ** vào `deferred-work.md`.
- **Trạng thái story sống ở `sprint-status.yaml`**; nội dung sống trong tệp story
  (`§Dev Agent Record` · `§Completion Notes` · `§Debug Log References` · `§Review Findings`).
- 🔴 **Đổi một bất biến kiến trúc là một `AD` MỚI, không phải một dòng mã.** Cổng thứ tư
  (AD-2), cổng lắng nghe thứ hai (AD-45), một ngữ nghĩa phân giải mới (AD-18) — cả ba đi
  qua thủ tục viết ra, không qua một lượt "tiện tay".
- **Ice là người chốt các quyết định mở.** Gặp một chỗ hai phương án đều hợp lệ: nêu cả
  hai kèm **số đo**, đừng tự chọn rồi đi tiếp — và cũng đừng loại một phương án chỉ vì nó
  đắt.

### Critical Don't-Miss Rules

_Tiêu chí vào mục này: **vi phạm được mà không cổng nào đỏ**, và hậu quả là dữ liệu hỏng
hoặc một kết quả sai trông như bình thường._

#### 🔴 Rỗng IM LẶNG bị cấm; rỗng CÓ LÝ DO thì không

Đây là lớp lỗi trung tâm của cả dự án. Một truy vấn trả 0 hàng trong 0,01 ms **không ném
lỗi nào** và biểu hiện thành *"tra từ không ra kết quả"* — không ai lần được nguyên nhân.

- Chuỗi con **1–2 ký tự tiếng Anh** khai là **KHÔNG HỖ TRỢ** và trả một trạng thái **phân
  biệt được** với *"không có kết quả"*. Không cho tràn qua nhánh tra chính xác *(nhánh trả
  về sẽ nói dối)*; không hạ ngưỡng trigram xuống 1 *(FTS5 `trigram` không lập chỉ mục
  token < 3 ký tự — đo: 0 hàng)*.
- **Vị từ điều phối zh/en là HÌNH DẠNG CHUỖI TRUY VẤN**, không phải ngôn ngữ của Tác phẩm.
  Bôi đen `API` trong một truyện tiếng Trung mà lọc `lang='zh'` ⇒ **0 hàng**, dù mục `API`
  có thật. Rỗng im lặng sinh ra bởi chính hàng rào chống rỗng im lặng.
- **`is_han` có ĐÚNG MỘT định nghĩa** — `tools/dict-build/src/char_idx.rs::is_han`. Hai
  workspace tách rời và **không có cổng kiểm chéo**: hai định nghĩa lệch nhau sẽ tra vào
  một `char_idx` chưa bao giờ lập chỉ mục ký tự đó ⇒ rỗng, không lỗi.
- **Không có sổ đăng ký *"tệp `.db` nào chứa ngôn ngữ nào"*.** Mọi tệp đang gắn đều được
  tra; `lang` lọc **trong SQL**. Một sổ như vậy sai im lặng đúng vào ngày một lớp gỡ rời
  được thêm hay gỡ (FR112).
- **Hạ chữ thường là THÊM một khoá, không THAY khoá gốc** — 1.635 đầu mục tiếng Anh mang
  chữ hoa có nghĩa (`API`, `Wikipedia`), 184 nhóm chỉ phân biệt nhau bằng chữ hoa. Dùng
  `headword IN (?1, ?2)` trong **một** truy vấn, không fallback dây chuyền. Phép hạ chữ
  thường phải **không phụ thuộc locale**.
- 🔴 **Một danh sách rỗng KHÔNG tự nói vì sao nó rỗng — hỏi vị từ `…HasLoaded` TRƯỚC khi kết
  luận.** Chưa nạp · đang chờ IPC · thật sự không có: chỉ ca thứ ba được phép nói *"không
  có"*; hai ca kia mà nói thế là màn hình khẳng định dứt khoát một điều nó chưa biết. Các vị
  từ ấy là hàm export nên **chỗ quên gọi vẫn biên dịch sạch, và không cổng nào canh** — đã
  hụt **hai** lần: `hanVietPending` (1.16) và lệnh điều hướng (2.10, `editorHasLoaded`).

#### 🔴 Dữ liệu người dùng — chỗ hỏng là VĨNH VIỄN

- **Ranh giới segment tính MỘT LẦN lúc nhập và lưu xuống.** Không đường mã nào tính lại
  lúc nạp. Một lần "cải thiện quy tắc tách câu" chạy lại trên Chương cũ làm lịch sử phiên
  bản, trạng thái xác nhận và ghi nhớ proofreader của mọi Chương **trỏ sai chỗ**.
- **Gộp/tách SEGMENT = về hưu + tạo mới** (trạng thái mới là *chưa xác nhận*, lịch sử
  rỗng). **Gộp/tách CHƯƠNG thì KHÔNG** — chỉ đổi `chapter_id` và `ord`, giữ nguyên
  `segment.id` và mọi dữ liệu gắn theo nó. Nhầm hai cái này phá sạch lịch sử của những
  Chương đã dịch xong.
- 🔴 **Mọi lượt ghi `target_text` mà văn bản KHÔNG đến từ bộ đệm gõ phải đặt CẢ HAI thứ trong cùng
  thao tác: mốc so sánh VÀ cột xuất xứ** (AD-47). Danh mục đóng: nhập song ngữ · chấp nhận thay đổi
  FR94 · điền sẵn từ TM FR58 · đề xuất AI · gộp/tách. **Ngoại lệ có tên duy nhất:** khôi phục FR101
  đặt mốc mà **không** đặt xuất xứ *(`segment_version` không mang xuất xứ — chữ ký #1(a))*.
  Quên vế xuất xứ ⇒ lượt xác nhận kế tiếp ghi **tôi dịch** cho chữ người dùng chưa gõ, cặp TM mang
  nhãn sai, `RagInjector` ưu tiên nó, và **không cổng nào đỏ**. Biểu hiện lộ ra sau hàng trăm câu
  dưới dạng *"AI dịch không còn giống giọng tôi"* — không lần ngược được. Flush AD-35 **không**
  thuộc danh mục này: nó chở đúng bộ đệm gõ.
- **`segment.id` không bao giờ tái dùng.** Mọi dữ liệu gắn theo segment tham chiếu `id`,
  **không bao giờ tham chiếu vị trí**.
- **Lược đồ có phiên bản, di trú CHỈ TIẾN.** Gặp phiên bản **mới hơn** ứng dụng ⇒ **từ
  chối mở** và báo rõ, **không bao giờ ghi vào**. Di trú chạy trong một giao dịch, sau khi
  đã sao lưu.
- **`library-index.db` là DẪN XUẤT** — chỉ `Indexer` ghi, và chỉ **sau khi** `.atproj` đã
  ghi xong. Xoá nó phải luôn là thao tác **an toàn**. `meta.json` cũng là cache dẫn xuất,
  ghi bởi chính `store::Writer` của Tác phẩm đó.
- **Mọi lệnh ghi đi qua `store::Writer` nối tiếp của kho tương ứng.** Không module nào tự
  mở kết nối ghi.
- **Hợp đồng flush Editor (AD-35):** idle **2 s** · **trần cứng 5 s KHÔNG reset bởi phím
  gõ** · xác nhận · rời segment · đóng Tác phẩm. Một debounce thuần **không bao giờ kích
  hoạt** khi người dùng gõ liên tục — mất không giới hạn công việc trong khi vẫn "đúng đặc
  tả auto-save". Một flush chỉ **xong** sau khi đã ghi vào **WAL**, không phải khi vào
  hàng đợi trong bộ nhớ.
- **Thao tác RỜI RẠC ghi NGAY, không qua bộ đệm gõ** — chấp nhận thay đổi từ Review Mode
  (FR94), điền sẵn từ TM khớp 100% (FR58). Định tuyến chúng qua bộ đệm khiến một thao tác
  người dùng **thấy đã xong** nằm chờ tới 5 giây rồi biến mất nếu app sập.

#### 🔴 Bảo mật

- **Khoá API không bao giờ đi qua IPC.** Crate `keyring` **trực tiếp trong Rust**.
  Frontend chỉ biết *"đã cấu hình / chưa cấu hình"*.
- **Nội dung từ ngoài KHÔNG BAO GIỜ render thành HTML.** Rust phân tích thành **mô hình dữ
  liệu có cấu trúc**; Vue render từ mô hình đó. Mô hình **không có nhánh nào mang HTML**.
  Không `v-html`, không tương đương.
- **Đúng BA điểm ra mạng, cả ba theo thao tác người dùng.** Không có điểm thứ tư. Không
  tải nền, không prefetch, không kiểm tra ngầm, **không tải lại ảnh đã có**.
- ⚠️ **AD-41 (phạm vi mạng) KHÔNG được framework cưỡng chế** — capabilities của Tauri là
  khai báo **tĩnh lúc build** nên không diễn đạt được *"chỉ các domain vừa dán lúc chạy"*.
  ⇒ **Nó bắt buộc phải có bộ test riêng**: từ chối host ngoài allowlist · từ chối **chuyển
  hướng** ra ngoài · từ chối **tài liệu** ở tầng 2 (tầng 2 chỉ được tải **ảnh**) · không
  lời gọi nào khi người dùng không bấm.

#### 🔴 Ranh giới module

- **Không module nào ngoài `ai/` được phụ thuộc `ai/`** (chiều ngược lại thì hợp lệ). Có
  test cưỡng chế. Vi phạm ⇒ FR77 (*chạy đầy đủ khi không cấu hình AI*) chết, và chỉ lộ ra
  khi một người dùng **không có API key** thử.
- **Không hợp nhất nguồn từ điển. Ở bất kỳ đâu.** Kết quả trả về **theo từng nguồn**, giữ
  nguyên bất đồng; cột `source` bắt buộc trên mọi bản ghi nghĩa. Cũng **không** hợp nhất
  `zh` với `en`.
- **Không cơ chế nào tự ghi vào Glossary.** Quét khi nhập và thu hoạch từ bản review ghi
  vào **bảng chờ riêng**; chỉ thao tác duyệt của người dùng mới chuyển sang Glossary.
  `glossary/` phơi ra **đúng một** truy vấn trả mục **đủ điều kiện chèn** vào prompt.
- **Mọi phân giải hai tầng đi qua `ScopeResolver`.** Ba loại *chỉ toàn cục* (phím tắt,
  preset bố cục, lựa chọn ứng dụng) phải **trả lỗi** khi có ai ghi ở tầng Tác phẩm — bỏ
  qua im lặng là cách một tầng bị cấm vẫn nằm trên đĩa rồi không bao giờ có tác dụng.

#### ⚠️ Hiệu năng

- **`LIKE` bị CẤM trên đường nóng tra cứu** — đo 20–50 ms.
- **NFR2: không frame nào vượt 50 ms** và **NFR18: mất ≤ 5 s công việc** đánh đổi lẫn
  nhau; chúng phải đo **cùng lúc trên cùng một Editor**.
- **`PRAGMA wal_autocheckpoint = 0`** — thời điểm checkpoint là quyết định của **ứng dụng**
  (luồng nền, kết nối riêng), cộng một **ngưỡng kích thước WAL buộc checkpoint** để
  `.db-wal` không phình vô hạn.
- **NFR1 đo TRÊN đường tiếng Anh, không suy ra từ số đo tiếng Trung.** Một con số mượn là
  một con số không ai đo.
- ⚠️ Số hiệu năng cũ của AD-26 *(0,02 / 0,15–4,5 / 0,13–0,19 ms)* **đã LỖI THỜI** — đo lại
  2026-08-05 trên `dict-core.db` sáu nguồn: nhánh 2 với truy vấn **một ký tự** là
  **7,324 ms**, vượt 1,6× cận trên cũ. Chi phí nằm ở **số hàng**, không ở chỉ mục, nên
  **không sửa được bằng một chỉ mục mới**.

#### ⚠️ Bẫy tài liệu — chú thích cũ hơn mã

Hai tệp còn mang một mệnh đề đã hết đúng: `src/commands/focus.ts:10-11` và
`src/panels/editorSegments.ts:25` vẫn viết *"dự án không có bộ chạy test frontend (và
không được thêm — NFR15)"*. Điều đó **hết đúng từ 2026-08-12** — kho **nay có** vitest +
`@vue/test-utils` + happy-dom, cây test ở `tests/frontend/**`.

⇒ **Tin cây nguồn hiện tại hơn một chú thích**, và khi chạm vào một tệp mang mệnh đề đã
sai thì **sửa tại chỗ kèm dấu 🔵 và ngày** — đúng cách ba tệp kia đã được sửa.
*(Cửa rà giấy phép NFR15 thì **vẫn đứng** cho gói tiếp theo — lượt lật đi QUA cửa đó,
không xoá nó.)*

---

## Usage Guidelines

**Cho AI agent:**

- Đọc tệp này **trước** khi viết dòng mã đầu tiên.
- Theo **đúng** luật đã ghi. Nghi ngờ thì chọn phương án **chặt hơn**.
- Tệp này **không thay** nguồn sự thật, nó trỏ về chúng:
  `ARCHITECTURE-SPINE.md` (47 `AD` + bảng Stack + Consistency Conventions) ·
  `deferred-work.md` (sổ nợ) · doc-comment của chính tệp đang sửa.
- Trước khi dựng một phép kiểm mới: hỏi **mệnh đề này đã có chủ ở đường nào chưa** (bốn
  đường ở §Testing Rules). Hai đường cùng canh một mệnh đề là hai nguồn sự thật.
- Gặp một luật ở đây mâu thuẫn với cây nguồn: **cây nguồn thắng**, và báo lại chỗ lệch —
  đừng im lặng theo một bên.

**Cho người duy trì:**

- Giữ tệp này gọn: nó chỉ chở thứ **không manifest nào và không cổng nào** trả lời được.
  Một luật đã có cổng canh thì cổng là chỗ của nó, không phải đây.
- Cập nhật khi: thêm/bỏ một cổng · đổi ngăn xếp hoặc thế hệ dữ liệu từ điển · một `AD`
  mới · một mệnh đề ở đây hết đúng.
- Luật đã trở nên hiển nhiên thì **gỡ** — độ dài của tệp này là chi phí thật ở mỗi lượt gọi.

Last Updated: 2026-08-18
