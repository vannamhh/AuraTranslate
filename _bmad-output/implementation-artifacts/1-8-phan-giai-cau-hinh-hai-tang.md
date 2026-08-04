---
baseline_commit: 0ff36a0f8d77cddeee09e32306f0e427438d2e35
---

# Story 1.8: Phân giải cấu hình hai tầng

Status: done

<!-- Note: Validation is optional. Run validate-create-story for quality check before dev-story. -->

**Covers:** FR103 · AD-18
**Epic:** 1 — Nền móng ứng dụng & Tra cứu ngoại tuyến tức thì

> 🔴 **Đây là story đầu tiên của cả dự án có một `#[tauri::command]` thật.** `src-tauri/src/commands/mod.rs` hôm nay là 21 dòng doc-comment và `lib.rs` **không có** `invoke_handler`. Từ hôm nay có một đường IPC sản phẩm, và ba món nợ đang chờ đúng đường đó mới trả được — xem §Nợ nhận lại.
>
> 🔴 **Và đây là chỗ AD-18 biến từ một bảng trong tài liệu thành một thứ trình biên dịch cưỡng chế.** Sáu loại dữ liệu trong bảng đó sống ở **sáu epic khác nhau** (Glossary → Epic 3, AI/Prompt → Epic 4, Luật làm sạch → Epic 6, TM → Epic 7). Không epic nào trong số đó sẽ đọc lại AD-18; chúng sẽ đọc **mã của story này**. Một ngữ nghĩa cài sai hôm nay không đỏ ở story sau — nó lộ ra ở Giai đoạn 4 dưới dạng *"TM toàn cục không bao giờ được dùng tới"*, đúng câu mà AD-18 tự khai là thứ nó tồn tại để chặn.

---

## Story

As a người dịch,
I want cấu hình riêng của một Tác phẩm đè lên cấu hình chung một cách nhất quán ở mọi nơi,
So that tôi không phải nhớ chỗ nào theo luật nào.

---

## Acceptance Criteria

### AC1 — Mọi phân giải hai tầng đi qua đúng một `ScopeResolver`

**Given** mọi phân giải hai tầng trong hệ thống
**When** xảy ra
**Then** đi qua đúng một `ScopeResolver`

*Đạt nghĩa là* **cả hai** cơ chế, theo đúng khuôn AC2 của Story 1.7 (§Bàn giao, mục 3):
1. **Kiểu** — `Semantics` và ba hàm phân giải chỉ tồn tại trong `core::scope`. Không module nào gọi được `resolve_override` cho một loại đã khai `Merge`: gọi sai trả `Err(ScopeError::WrongSemantics)`, ⛔ không im lặng làm theo ý người gọi.
2. **Test** — `src-tauri/tests/scope_boundary.rs` quét cây nguồn: một danh sách token cấm (§Task 8) chỉ được xuất hiện dưới `src/core/scope/**`. Có **sàn số tệp** để cây rỗng không đọc thành sạch.

⚠️ Hôm nay chưa có consumer nào — Glossary/TM/Prompt/AI/Luật làm sạch đều là module rỗng. Điều đó **không** làm AC1 thành mệnh đề vòng: cổng quét cấm token, nên nó đỏ ngay lần đầu một module Epic 3 tự viết một nhánh `if work.is_some()`. Đó chính là lượt đỏ mà AC này tồn tại để mua.

### AC2 — Bốn loại dữ liệu mang ngữ nghĩa **ghi đè**

**Given** loại dữ liệu Glossary, Prompt, Cấu hình AI, Tên người dịch
**When** phân giải
**Then** ngữ nghĩa là **ghi đè** — tầng Tác phẩm thắng

*Đạt nghĩa là* **ghi đè theo từng khoá, không theo cả tập** — xem §Quyết định #3. AD-18 viết rõ *"tầng Tác phẩm thắng **theo từng thuật ngữ**"*, và Story 3.4 phát biểu cùng một luật ở chiều ngược lại: *"áp **cả hai**, tầng Tác phẩm thắng khi trùng"*. Một cài đặt kiểu *"work rỗng thì dùng global, work không rỗng thì dùng work"* **đạt mọi test viết cẩu thả và sai hoàn toàn** — nó làm 412 mục Glossary toàn cục biến mất ngay khi người dùng thêm một mục riêng cho Tác phẩm.

*Và* kết quả phải mang **xuất xứ theo từng khoá**: giá trị thắng, tầng sinh ra nó, và giá trị **bị che** nếu có. Ba màn hình đã vẽ sẵn phụ thuộc vào điều này (§Yêu cầu UX).

### AC3 — Hai loại dữ liệu mang ngữ nghĩa **hợp nhất**

**Given** loại dữ liệu Translation Memory và Luật làm sạch khi nhập
**When** phân giải
**Then** ngữ nghĩa là **hợp nhất** — cả hai tầng cùng áp

*Đạt nghĩa là* **ba** mệnh đề, không phải một:
1. Kết quả chứa mục của **cả hai** tầng, ⛔ không khử trùng lặp *(AD-19 cùng triết lý: giữ nguyên bất đồng)*.
2. **Mỗi mục mang nhãn tầng** — không phải cả tập mang một nhãn. Story 6.5 đòi *"mỗi luật mang **nhãn tầng** — Toàn cục hoặc Tác phẩm"*, và màn quản lý Glossary đòi hiện mục toàn cục *"đang bị che"*.
3. **Tầng là khoá phụ, không bao giờ là khoá chính.** AD-18 khai tường minh: khoá chính là **xuất xứ** (FR118), khoá phụ là **tầng** (Tác phẩm trước Global). `core::scope` ⛔ **không biết** xuất xứ là gì — nó nhận một bộ so sánh chính từ chỗ gọi và **luôn** áp tầng làm khoá phụ. Xem §Quyết định #4.

### AC4 — Thêm loại dữ liệu mới phải khai ngữ nghĩa tường minh, không có mặc định ngầm

**Given** một loại dữ liệu mới được thêm vào
**When** đăng ký với `ScopeResolver`
**Then** phải khai ngữ nghĩa tường minh
**And** không có mặc định ngầm nào

*Đạt nghĩa là* **cưỡng chế bằng trình biên dịch, không bằng test và không bằng tài liệu**: một macro khai báo `scope_kinds!` sinh ra `enum ScopeKind` · `ALL` · `as_str()` · `semantics()` từ **một** chỗ khai, nên **không tồn tại cú pháp nào thêm được một biến thể mà không kèm ngữ nghĩa**. Đây là đúng khuôn `message_keys!` của Story 1.5 (`core/i18n/mod.rs:100`), và lý do giống hệt: *"một khai báo, ba thứ sinh ra, nên chúng không trôi khỏi nhau được."*

⛔ **Không có `impl Default for Semantics`. Không có nhánh `_ =>` trong bất kỳ `match` nào trên `ScopeKind`.** Hai thứ đó là cùng một lỗ mà AC này tồn tại để bịt.

> 🔴 **Lượt review kiến trúc đã bắt đúng lỗ này hai lần** *(`reviews/review-adversarial-2026-08-03b.md:49-59`, §F4)*: **Luật làm sạch (FR124)** và **Tên người dịch (FR131)** đều đã tới `ScopeResolver` mà **không có hàng trong bảng AD-18**. Bảng nay có sáu hàng vì hai hàng đó được vá vào. AC4 tồn tại để lần thứ ba không cần một lượt review mới bắt.

### AC5 — Tầng Global phân giải được khi chưa mở Tác phẩm nào

**Given** tầng Global
**When** ứng dụng chạy mà chưa mở Tác phẩm nào
**Then** phím tắt và preset bố cục phân giải được từ `global.db`

*Đạt nghĩa là* một vòng chạy **end-to-end thật**: `global.db` có bảng cấu hình *(bước di trú 2)* → `core::scope` đọc qua `Store::read` → `#[tauri::command]` trả về → `src/main.ts` áp trước `mount()`. Nghiệm thu bằng test ghi thẳng một hàng vào `global.db` rồi khẳng định đường phân giải trả đúng hàng đó.

⚠️ **`chưa mở Tác phẩm nào` là trạng thái duy nhất tồn tại hôm nay, không phải một ca biên.** `.atproj` và `project.db` là **Story 1.15**; `StoreKind::Project` chưa có `StoreSpec` nào. Tầng Tác phẩm hôm nay là `Option::None` ở mọi chữ ký — xem §Ranh giới phạm vi.

---

## Tasks / Subtasks

- [x] **Task 1 — Đường cơ sở: chạy tám lệnh trên cây sạch, ghi số vào §Debug Log References** (không AC)
  - [x] `npm run build` *(bắt buộc trước `cargo test` — `generate_context!` nhúng `dist/` lúc biên dịch)*
  - [x] `cargo test --manifest-path src-tauri/Cargo.toml` · `check:deps` · `check:tokens` · `check:i18n` · `check:commands` · `check:scope` · `check:scope:bundled`
  - [x] Ghi lại: số tệp `.rs` dưới `src-tauri/src/**` *(nay là **22**)*, số tệp trong quần thể của `check-i18n.mjs` *(nay là **23**, gồm `build.rs`, trừ `tests/**`)*, tổng số test Rust *(nay là **41**)*, số khoá `vi.json` *(nay là **16**)*
  - [x] ⛔ Không sửa gì ở task này. Tám lệnh phải exit 0 **trước** khi gõ dòng đầu tiên; một cái đỏ sẵn thì dừng và báo.

- [x] **Task 2 — `scope_kinds!` và bảng ngữ nghĩa** (AC2, AC3, AC4)
  - [x] `src-tauri/src/core/scope/kinds.rs` — `macro_rules! scope_kinds!` sinh `enum ScopeKind` · `ALL` · `as_str()` · `const fn semantics()`. Khuôn: `core/i18n/mod.rs:62-91`.
  - [x] Khai **chín** loại theo bảng ở §Quyết định #2. Sáu loại của AD-18 + ba loại Global-only của FR103/AC5.
  - [x] `pub enum Semantics { Override, Merge, GlobalOnly }` — ⛔ **không** `derive(Default)`, ⛔ không `impl Default`.
  - [x] `pub enum Tier { Global, Work }` — ⛔ tên là `Work`, **không** `Project`. Consistency Conventions cấm `Project` cho thực thể Tác phẩm; `StoreKind::Project` đặt tên cho **tệp** `project.db`, không cho tầng.
  - [x] Test đối chứng: `ALL.len()` so với một hằng số viết tay ⇒ thêm biến thể mà quên `ALL` thì đỏ.

- [x] **Task 3 — Ba hàm phân giải** (AC1, AC2, AC3)
  - [x] `src-tauri/src/core/scope/resolve.rs` — ba hàm **thuần**, không chạm đĩa, không chạm `Store`:
    - `resolve_override<K, V>(kind, global: &BTreeMap<K,V>, work: Option<&BTreeMap<K,V>>) -> Result<BTreeMap<K, Resolved<V>>, ScopeError>`
    - `resolve_merge<V>(kind, global: &[V], work: Option<&[V]>, primary: Option<&dyn Fn(&V,&V) -> Ordering>) -> Result<Vec<Tiered<V>>, ScopeError>`
    - `resolve_global_only<K, V>(kind, global: &BTreeMap<K,V>, work: Option<&BTreeMap<K,V>>) -> Result<BTreeMap<K, Resolved<V>>, ScopeError>`
  - [x] `pub struct Resolved<V> { value: V, tier: Tier, shadowed: Option<V> }` · `pub struct Tiered<V> { tier: Tier, value: V }`
  - [x] 🔴 `resolve_override` **hợp nhất khoá rồi mới cho Work thắng trên khoá trùng** (§Quyết định #3, §Bẫy 1). Khoá chỉ có ở Global **phải còn** trong kết quả, mang `tier: Global`.
  - [x] 🔴 `resolve_merge` áp `primary` trước, **tầng luôn là khoá phụ** (§Quyết định #4, §Bẫy 2). `primary = None` ⇒ chỉ còn tầng, Work trước Global — thứ tự ổn định (`sort_by` chứ không `sort_unstable_by`).
  - [x] `resolve_global_only` với `work = Some(..)` không rỗng ⇒ `Err(ScopeError::WorkTierForbidden)`.
  - [x] Gọi hàm sai ngữ nghĩa của `kind` ⇒ `Err(ScopeError::WrongSemantics { .. })` **ở cả debug lẫn release**. ⛔ Không `panic!`, không `unwrap()` — `panic = "abort"` ở `[profile.release]`.

- [x] **Task 4 — `ScopeResolver` là điểm vào duy nhất** (AC1)
  - [x] `src-tauri/src/core/scope/mod.rs` — giữ nguyên doc-comment 5 dòng đang có, viết tiếp bên dưới.
  - [x] `pub struct ScopeResolver { work: Option<WorkScope> }` — hôm nay `work` **luôn** là `None`; `WorkScope` là một struct rỗng đánh dấu chỗ Story 1.15 điền vào. `ScopeResolver::global_only()` là hàm dựng duy nhất tồn tại hôm nay.
  - [x] Ba hàm của Task 3 phơi ra làm **method** của `ScopeResolver`; `resolve.rs` giữ `pub(crate)`. Đó là vế *"đúng một"* của AC1 ở tầng kiểu.
  - [x] ⛔ `core::scope` **không `use tauri::…`** — cùng lý do Quyết định #1 của Story 1.7: test dựng được trên thư mục tạm mà không cần webview. Đường lấy `AppHandle`/`State` nằm ở `commands/`.
  - [x] ⛔ **Không khai trait nào.** AD-2 khoá số cổng ở **ba**, và AD-40 đã lập tiền lệ *"hai module Rust thường, không trait hoá"*. `ports/mod.rs` giữ nguyên 5 dòng.

- [x] **Task 5 — Bước di trú 2 của `global.db`** (AC5)
  - [x] `src-tauri/src/core/store/schema.rs` — thêm `pub const CONFIG_VALUE_DDL: &'static str` và `Migration { to_version: 2, sql: CONFIG_VALUE_DDL }` vào `GLOBAL_MIGRATIONS` (`schema.rs:76`).
  - [x] Lược đồ **một bảng** (§Quyết định #5):
    ```sql
    CREATE TABLE config_value (
      kind       TEXT NOT NULL,
      key        TEXT NOT NULL,
      value      TEXT NOT NULL,
      updated_at TEXT NOT NULL,
      PRIMARY KEY (kind, key)
    );
    ```
  - [x] `updated_at` dùng `strftime('%Y-%m-%dT%H:%M:%fZ','now')` của chính SQLite — ISO-8601 UTC theo Consistency Conventions. ⛔ Không thêm `chrono`/`time`.
  - [x] 🔴 **Sửa `src-tauri/tests/store_contract.rs` — đúng một test, đã soát cả tệp** (§Bẫy 3):
    - `a_fresh_database_migrates_up_to_target_and_logs_it` (`:752`) dùng `spec_with` ⇒ chạy trên `GLOBAL_MIGRATIONS` thật. Đổi `schema_version()` `1 → 2` (`:758`) và `COUNT(*) FROM schema_migration_log` `1 → 2` (`:766`), kèm câu khẳng định ở `:760`. Bổ sung khẳng định hàng `version = 2` có mặt.
    - ⛔ **Không đụng** `one_step_runs_and_a_backup_is_written_first` (`:797`), `a_failing_migration_rolls_back_and_leaves_the_version_alone` (`:854`) và ca kế nó — cả ba dùng `spec_with_migrations` với fixture cục bộ `TWO_STEP` / `BROKEN_STEP_TWO`, **không** phụ thuộc `GLOBAL_MIGRATIONS`.
    - ⛔ **Không đụng** `a_newer_schema_is_refused_without_touching_a_single_byte` (`:918`) — nó tính `target` **động** từ `GLOBAL_MIGRATIONS` (`:921`) nên tự đúng theo.
    - ⚠️ **Doc-comment ở `:714` thành SAI sau story này** — nó viết *"`GLOBAL_MIGRATIONS` hôm nay có **đúng một** bước, nên `target - 1 == 0`… Ca 10 vì thế **không thể** nghiệm thu trên bộ di trú thật."* Sau bước 2 thì nó **nghiệm thu được**. Cập nhật comment; ⛔ nhưng **giữ nguyên `TWO_STEP` và `spec_with_migrations`** — lý do chúng tồn tại (`StoreSpec.migrations` là trường, cho Story 1.15) không đổi.
  - [x] ⛔ `sql` là `&'static str`, ⛔ không `format!`. ⛔ Không sửa bước 1.

- [x] **Task 6 — Đường đọc/ghi tầng Global** (AC5)
  - [x] `src-tauri/src/core/scope/store.rs` — `load_kind(store: &Store, kind: ScopeKind) -> Result<BTreeMap<String,String>, StoreError>` và `save_value(store: &Store, kind, key, value) -> Result<(), StoreError>`.
  - [x] Dùng `Store::read` / `Store::write` với các kiểu **tái xuất** từ `core::store` (`Transaction`, `SqlResult`, `Row`, `ReadHandle`). 🔴 ⛔ **Token `rusqlite` không được xuất hiện ở đâu dưới `core/scope/**`, kể cả trong comment đuôi dòng** (§Bẫy 4).
  - [x] ⛔ Không gọi `Store::write` bên trong một job ghi — `writer.rs:104` trả `WriteFailed` chứ không xếp hàng, và đó là chốt chống deadlock chứ không phải lỗi.
  - [x] ⛔ `core::scope` **không** sở hữu kho của mọi loại. Glossary/TM/Prompt/Luật làm sạch sẽ mang **bảng riêng của chúng** ở epic của chúng; `config_value` chỉ phục vụ ba loại `GlobalOnly` (§Quyết định #5).

- [x] **Task 7 — Bề mặt IPC đầu tiên của dự án** (AC5, + ba món nợ)
  - [x] `src-tauri/src/commands/config.rs`:
    - `pub fn bootstrap_config(store: Option<&Store>) -> Result<BootstrapConfig, IpcError>` — **hàm thuần theo `Option<&Store>`, đây là thứ test gọi**.
    - `pub fn put_config(store: Option<&Store>, kind: &str, key: &str, value: &str) -> Result<(), IpcError>`
    - Hai `#[tauri::command]` mỏng bọc hai hàm trên, lấy `State<Store>` qua `try_state`.
  - [x] `store = None` ⇒ `IpcError` mang `code = "store.open_failed"`, `MessageKey::StoreOpenFailed`, `params = {"store": "global"}`, `retryable = false`. 🔴 **Đây là bề mặt mà `deferred-work.md:177` chờ** — một `$APPDATA` không ghi được nay **nói ra** thay vì chỉ ra `stderr`.
  - [x] `BootstrapConfig { theme: String, mode: String, shortcuts: BTreeMap<String,String>, layout_presets: BTreeMap<String,String> }` — ⛔ **không** `#[serde(rename_all = "camelCase")]`, khoá trên dây là `snake_case`.
  - [x] `src-tauri/src/lib.rs` — thêm `.invoke_handler(tauri::generate_handler![...])` vào builder (`lib.rs:37`). ⛔ Giữ nguyên `open_global_store` **không chặn khởi động** — nay nó đúng, vì đã có bề mặt để nói.
  - [x] 🔴 **Sửa `src-tauri/tests/ipc_contract.rs`**: `ipc_error_wire_shape` (`:128-137`) hôm nay quét chính fixture nó tự dựng ở `:73-78` — một mệnh đề vòng mà `deferred-work.md:49` giao đích danh story này. Đổi nó sang serialize **giá trị `bootstrap_config(None)` trả về**. ⛔ Không dựng command giả để "làm cho đúng lời hứa cũ" — hàm này là đường sản phẩm thật.
  - [x] ⛔ Không thêm khoá `MessageKey` nào và ⛔ không thêm chuỗi `vi.json` nào (§Quyết định #7). ⛔ Không đụng `capabilities/main.json` — `tests/config_invariants.rs:333` khoá đúng ba quyền, và command do ứng dụng khai **không cần** mục ACL trong Tauri v2.

- [x] **Task 8 — Hai tệp test mới** (AC1–AC5)
  - [x] `src-tauri/tests/scope_contract.rs` — hành vi. Tối thiểu: ghi đè theo khoá *(khoá chỉ-Global còn nguyên)*; `shadowed` mang giá trị bị che; hợp nhất giữ cả hai tầng, không khử trùng lặp; **tầng là khoá phụ** khi có `primary`; gọi sai ngữ nghĩa ⇒ `Err`; `resolve_global_only` với work ⇒ `Err`; `ALL` phủ mọi biến thể; bảng ngữ nghĩa khớp từng hàng AD-18; đọc lại được hàng ghi thẳng vào `global.db`.
  - [x] `src-tauri/tests/scope_boundary.rs` — ranh giới, khuôn `store_boundary.rs`:
    - `const SCOPE_DIR: &str = "core/scope";`
    - Token cấm ngoài `core/scope/**`: `"Semantics"`, `"resolve_override"`, `"resolve_merge"`, `"ScopeKind"` *(vế test của AC1)*.
    - `"rusqlite"` / `"Connection::open"` **vẫn cấm** trong `core/scope/**` — đây là quần thể mới, `store_boundary.rs` đã bao nhưng hãy khẳng định lại có chủ đích.
    - `core_scope_does_not_depend_on_tauri` — cùng khuôn `core_store_does_not_depend_on_tauri`.
    - **Sàn số tệp bắt buộc** + một **đối chứng dương** *(`core/scope/` thật sự có ≥ 3 tệp nhắc `ScopeKind`)*, để cây rỗng không đọc thành sạch.
  - [x] Mỗi ca có thư mục tạm **riêng** (pid + `AtomicU64`); **drop `Store` trước** khi `remove_dir_all` (Windows/NFR14); ⛔ không `sleep` dài; ⛔ không thêm `tempfile`.

- [x] **Task 9 — Frontend nạp cấu hình từ đĩa** (AC5)
  - [x] `src/config/bootstrap.ts` **(tệp mới)** — bọc `invoke('bootstrap_config')`, `try/catch`, trả `{ config | null, error: IpcError | null }`. ⛔ Không ném. Chạy trong `npm run dev` (không có Tauri) ⇒ nhánh lỗi, ứng dụng vẫn lên bằng mặc định.
  - [x] `src/main.ts` — `await` bootstrap **trước** `applyTheme()`, giữ nguyên hai khối *"THỨ TỰ BẮT BUỘC"*: `applyTheme(cfg?.theme ?? 'light')` → `loadFonts()` → `installCommands({ setMode, bindings })` → `setMode(cfg?.mode ?? 'library')` → `mount()`.
  - [x] `src/commands/index.ts` — `CommandDeps` nhận thêm `bindings?: Readonly<Record<string, readonly string[]>>`; `installCommands` dùng `bindings?.[id] ?? <mặc định hiện tại>`. ⛔ **Không** thêm `import` nào vào `src/commands/**` — ba phép kiểm của `check-commands.mjs` nạp thư mục này bằng Node thuần (§Bẫy 6).
  - [x] 🔴 **Xung đột hợp âm từ đĩa không được làm ứng dụng không khởi động nổi** (§Bẫy 5). `createKeymap` ném khi hai command trùng hợp âm; một `global.db` sửa tay là đủ gây ra. Bắt, ghi chẩn đoán, **rơi về hợp âm mặc định**. Màn giải quyết xung đột là **Story 1.21**.
  - [x] `src/main.ts` — `watch(currentMode, …)` gọi `put_config` để lưu chế độ cuối. 🔴 Đóng `deferred-work.md:140`.
  - [x] `src/App.vue` — dải báo lỗi không chặn, nội dung là `tError(err)`. ⛔ Chỉ dùng token màu đã có (`check:tokens`), ⛔ không chuỗi tiếng Việt trong `.vue`.

- [x] **Task 10 — Cập nhật sàn quần thể của các cổng** (không AC)
  - [x] `scripts/check-i18n.mjs:228` `RS_FLOOR` — đếm lại quần thể thật sau Task 2–7 rồi đặt sàn ở ~80 % *(hôm nay 18 trên quần thể 23; dự kiến quần thể mới 27)*.
  - [x] `src-tauri/tests/store_boundary.rs:44` `RS_FLOOR` — quần thể **khác** (`src-tauri/src/**`, nay 22, dự kiến 26). Đếm riêng, ⛔ đừng chép số của tệp kia.
  - [x] ⛔ **Sàn tồn tại để bắt một cây bị cắt mất, không phải để đếm tệp mới** — Story 1.7 §Completion Notes #10. Đặt sàn **bằng** số thật là tự tạo một cổng đỏ ở story sau.
  - [x] Chứng minh sàn cắn: đặt trên số thật ⇒ đỏ ⇒ trả lại.
  - [x] ⛔ **Không thêm bước CI mới.** `ci.yml` job `check` đã chạy đủ tám lệnh; story này không sinh loại kiểm mới (Story 1.7 §Quyết định #9, cùng lập luận).

- [x] **Task 11 — Nghiệm thu đỏ-rồi-xanh** (tất cả AC)
  - [x] Với **mỗi** cơ chế, gỡ đúng một thứ rồi trả lại, ghi bảng vào §Debug Log References. Tối thiểu **hai đối chứng âm mỗi cơ chế**. Tiền lệ: 1.5 (21 ca) · 1.6 (28 ca) · 1.7 (17 ca).
  - [x] Bắt buộc có trong bảng: đổi `resolve_override` sang *"work không rỗng thì trả work"* ⇒ ca khoá-chỉ-Global đỏ · đảo `primary`/tầng ⇒ ca khoá phụ đỏ · thêm nhánh `_ =>` vào `semantics()` ⇒ ca `ALL` đỏ · gõ `rusqlite` vào một comment đuôi dòng trong `core/scope/` ⇒ `scope_boundary` đỏ · quên sửa `store_contract.rs:753` ⇒ đỏ.
  - [x] Tám lệnh của Task 1 exit 0 lần cuối. `check:scope` và `check:scope:bundled` **phải chạy tay lại** vì `lib.rs` đã đổi.
  - [x] Ghi mọi thứ chưa đóng vào `deferred-work.md` dưới một mục `## Deferred from: 1-8-…`.

---

### Review Findings

Review chạy 2026-08-04 qua ba lớp song song (Blind Hunter, Edge Case Hunter, Acceptance Auditor đối chiếu §Acceptance Criteria + §Tám cái bẫy + §Quyết định thiết kế của chính story này). Acceptance Auditor không tìm thấy vi phạm AC nào — mọi AC, mọi bẫy, mọi quyết định khoá đều đúng như spec.

- [x] [Review][Patch] `scope_boundary.rs::only_core_scope_may_name_the_two_tier_vocabulary` sẽ tự chặn chính đường gọi hợp lệ mà nó khuyến nghị [`src-tauri/src/core/scope/mod.rs:201,218`, `src-tauri/tests/scope_boundary.rs:62-67`] — **Ice chọn phương án 2 (2026-08-04): đổi tên phương thức công khai của `ScopeResolver`.** `resolve_override`/`resolve_merge` đổi thành `apply_override`/`apply_merge`; `resolve.rs` giữ nguyên tên hàm nội bộ (`pub(crate)`, không đổi). Đã cập nhật mọi call site ở `scope_contract.rs` (10 chỗ) và câu hướng dẫn trong thông báo lỗi của `scope_boundary.rs`. Toàn bộ 62 test Rust + bốn cổng `.mjs` + `check:scope` chạy lại xanh sau sửa.
  ⚠️ **Sửa này chỉ xoá HAI trong BỐN token đụng độ.** `"ScopeKind"` vẫn nằm trong `FORBIDDEN_OUTSIDE_SCOPE`, và mọi lời gọi hợp lệ tương lai vẫn phải viết `ScopeKind::Glossary` (hay tương đương) để truyền tham số cho `apply_override`/`apply_merge` — nên cổng vẫn sẽ đỏ ở token đó ngay lần đầu Epic 3 chạm vào, dù đường gọi là đúng. Xoá triệt để đòi đổi chữ ký (nhận `&str` thay vì `ScopeKind`, giống khuôn `save_value`) — ngoài phạm vi quyết định hôm nay. Đã ghi vào `deferred-work.md`.

- [x] [Review][Patch] Dải báo lỗi kho `role="status"` khó được trình đọc màn hình thông báo đúng lúc nó quan trọng nhất [`src/App.vue:127`] — Đã sửa: thêm node `.sr-announcer` LUÔN có mặt trong DOM (không `v-if`), nội dung khởi tạo rỗng và chỉ được điền sau `nextTick()` trong `onMounted` — tạo một lượt ĐỔI nội dung thật mà trình đọc màn hình quan sát được, thay vì nội dung có sẵn từ lượt vẽ đầu. Dải hiển thị cho người dùng sáng mắt (`.config-error`) giữ nguyên `v-if`. `npm run build` + bốn cổng `.mjs` + `cargo test` (62 ca) chạy lại xanh.
- [x] [Review][Patch] `loadFonts()` không còn chạy song song từ đầu tiến trình khởi động [`src/main.ts:76`] — Đã sửa: đưa `void loadFonts()...` lên TRƯỚC `await loadBootstrapConfig()` trong `boot()`, khôi phục đúng hành vi "bắt đầu nạp font ngay lập tức, không chờ gì" như trước story này. `npm run build` + `cargo test` chạy lại xanh.

- [x] [Review][Defer] `resolve_one` trong `load_global_config` nuốt lỗi `WrongSemantics` bằng `debug_assert!` rồi `unwrap_or_default()` [`src-tauri/src/core/scope/store.rs:135-147`] — deferred, pre-existing pattern (lỗi lập trình được canh bằng `cargo test` bắt buộc trước khi merge, không phải bằng type system; rủi ro thật khi build release chỉ xảy ra nếu ai đó đổi ngữ nghĩa `AppConfig`/`Shortcut`/`LayoutPreset` mà quên sửa chỗ này, và CI sẽ đỏ trước khi điều đó lọt qua)
- [x] [Review][Defer] `watch(currentMode)` gọi `put_config` không có khoá thứ tự [`src/main.ts:178-184`] — deferred, pre-existing pattern (đổi chế độ liên tiếp nhanh có thể ghi đè bằng một giá trị cũ hơn xuống đĩa; xác suất thấp, tự phục hồi ở lượt chuyển chế độ kế tiếp)
- [x] [Review][Defer] Nhánh lỗi `store.read_failed` của `bootstrap_config` chưa có test ép đường đọc thật trượt [`src-tauri/tests/scope_contract.rs:700`] — deferred, pre-existing pattern (đường lan `?` và phép chuyển `From<StoreError>` đã được kiểm ở tầng `store` từ Story 1.7; thiếu một ca tích hợp trực tiếp qua `bootstrap_config`)
- [x] [Review][Defer] `save_value` không giới hạn độ dài `key`/`value` trước khi ghi [`src-tauri/src/core/scope/store.rs:203`] — deferred, pre-existing pattern (mọi lời gọi hôm nay đến từ frontend tin cậy của chính ứng dụng, không phải biên tin cậy với người dùng ngoài)
- [x] [Review][Defer] `rel.starts_with(SCOPE_DIR)` không có dấu `/` đuôi [`src-tauri/tests/scope_boundary.rs:40`] — deferred, pre-existing pattern (chép nguyên khuôn `store_boundary.rs` của Story 1.7; một thư mục anh em tên `core/scope_legacy` sẽ khớp nhầm, nhưng xác suất gần như bằng không)
- [x] [Review][Defer] `code_lines()` chỉ miễn trừ dòng bắt đầu bằng `//`, không bóc comment khối `/* … */` [`src-tauri/tests/scope_boundary.rs:133`] — deferred, pre-existing pattern (kế thừa đúng quy ước `store_boundary.rs`; codebase không dùng comment khối)

---

## Dev Notes

### Ranh giới phạm vi — đọc trước khi gõ dòng đầu tiên

| Thứ | Trong story này? |
|---|---|
| `enum ScopeKind` + bảng ngữ nghĩa + ba hàm phân giải | ✅ **Có** — hạt nhân |
| Bước di trú 2 của `global.db`, bảng `config_value` | ✅ **Có** |
| Hai `#[tauri::command]` + `invoke_handler` đầu tiên | ✅ **Có** |
| Frontend nạp theme + chế độ + hợp âm từ đĩa | ✅ **Có** |
| Bề mặt hiển thị lỗi mở kho | ✅ **Có** — `deferred-work.md:177` |
| Chữa mệnh đề vòng `ipc_error_wire_shape` | ✅ **Có** — `deferred-work.md:49` |
| `.atproj` · `project.db` · `StoreSpec::work()` | ❌ **Không** — **Story 1.15** |
| Dữ liệu Glossary / TM / Prompt / Cấu hình AI / Luật làm sạch | ❌ **Không** — Epic 3 · 7 · 4 · 4 · 6. Story này khai **ngữ nghĩa** của chúng, ⛔ không khai bảng của chúng |
| Màn hình Cài đặt, thanh chuyển phạm vi, `⌘⇧R` trả về kế thừa | ❌ **Không** — không story Epic 1 nào dựng màn Cài đặt |
| Màn gán phím, phát hiện & giải quyết xung đột phím | ❌ **Không** — **Story 1.21** |
| Nội dung preset bố cục, `dockview` | ❌ **Không** — **Story 1.14** |
| Đẩy một mục lên tầng Global (promote) | ❌ **Không** — Epic 3 / Epic 7, và nó chạm **hai** hàng đợi ghi |
| Trait trong `ports/` | ❌ **Không** — AD-2 khoá ở ba cổng; `ProjectStore` là Story 1.15 |
| Cache kết quả phân giải | ❌ **Không** — chưa có consumer đường nóng. Mở lại ở **Story 3.4** |
| Phụ thuộc mới bất kỳ | ❌ **Không** — NFR15 đòi rà GPLv3 + ghi vào bảng Stack trước |

### 🔴 Vì sao tầng Tác phẩm là `Option::None` ở khắp nơi hôm nay

`.atproj` là **Story 1.15**, bảy story nữa. `src-tauri/src/core/store/mod.rs:122-134` nói thẳng: *"⛔ Chỉ [`StoreKind::Global`] có mã khởi tạo hôm nay… Viết sẵn mã cho hai loại kia hôm nay là mã không ai gọi, và nó sẽ sai theo đúng cách mà không test nào bắt."*

Hệ quả cho story này, và nó là **thiết kế chứ không phải thiếu sót**:
- Mọi chữ ký nhận tầng Tác phẩm là `Option<&…>`, và đường sản phẩm hôm nay luôn truyền `None`.
- Nhánh `Some(..)` **vẫn phải đúng và vẫn phải có test** — test cấp dữ liệu tầng Work bằng tay. Đó là hợp đồng mà Story 1.15 cắm `project.db` vào.
- ⛔ Không dựng `StoreSpec::work()`, không mở kho thứ hai, không đoán hình dạng `meta.json`.

Nói cách khác: **story này giao xong hợp đồng hai tầng, và giao xong một tầng.**

### Trạng thái repo hiện tại — số, không phải mô tả

| Thứ | Số / trạng thái tại `HEAD = 0ff36a0` |
|---|---|
| `.rs` dưới `src-tauri/src/**` | **22** |
| Quần thể `check-i18n.mjs` | **23** *(gồm `build.rs`; `tests/**` miễn trừ)*, sàn **18** |
| Quần thể `store_boundary.rs` | **22**, sàn **18** (`:44`) |
| Test Rust | **41** — `config_invariants` 15 · `ipc_contract` 5 · `store_contract` 17 · `store_boundary` 4 |
| Khoá `vi.json` | **16** — 7 `err.*` · 4 `command.*` · 3 `mode.*.status` · 2 `panel.*.title` |
| Bảng trong `global.db` | **1** — `schema_migration_log`, `user_version = 1` |
| `#[tauri::command]` trong dự án | **0** |
| `invoke_handler` trong `lib.rs` | **không có** |
| `invoke()` ở frontend | **0** lời gọi |
| `src-tauri/src/core/scope/mod.rs` | **5 dòng doc-comment**, đã khai `pub mod scope;` ở `core/mod.rs` |
| `src-tauri/src/commands/mod.rs` | **21 dòng doc-comment**, 0 hàm |
| `src/layout/` | **rỗng** (README + `.gitkeep`) |
| Lưu trữ phím tắt | **không có** — hợp âm là literal ở `src/commands/index.ts:141` |
| Phụ thuộc mới cần thêm | **0** |

Doc-comment sẵn có ở `core/scope/mod.rs` — **giữ nguyên, viết tiếp bên dưới**:
```rust
//! `ScopeResolver` — phân giải hai tầng Global / Tác phẩm, ngữ nghĩa khai báo
//! tường minh (AD-18).
//!
//! Mọi tra cứu hai tầng đi qua đây; ghi đè hay hợp nhất là quyết định đã ghi ở AD-18,
//! không phải thứ mỗi nơi gọi tự chọn.
```

### 🔴 "Tầng" mang SÁU nghĩa khác nhau trong repo này — story sở hữu đúng một

Đây là bẫy đọc hiểu nghiêm trọng nhất của story. Trước khi sửa bất cứ thứ gì có chữ *scope* hay *tầng*, đối chiếu bảng này:

| Cách dùng | Nghĩa | Có phải story này không? |
|---|---|---|
| `core/scope/` · AD-18 · FR103 | **Cấu hình hai tầng Global / Tác phẩm** | ✅ **Đúng cái này** |
| `capabilities/` · AD-23 · `check:scope` · `check:scope:bundled` | Phạm vi **filesystem** do Tauri cưỡng chế | ❌ ⛔ Không đụng |
| AD-41 · NFR19 · `allowlist mạng hai tầng` | Phạm vi **mạng** lúc nhập từ URL | ❌ Epic 6 |
| Story 5.1 *"Library hai tầng"* | **Cấp bậc thực thể** Tác phẩm → Chương | ❌ Epic 5 |
| FR6 *"ghi đè thủ công được"* | Đè **trạng thái vòng đời** của một Tác phẩm | ❌ Epic 5 |
| FR32 / FR63 *"không hợp nhất"* / *"giữ lại tất cả"* | Không gộp **nguồn từ điển** / nhiều bản dịch | ❌ AD-19, Epic 7 |

Nói riêng: `npm run check:scope` **không liên quan gì** tới story này ngoài việc nó phải còn xanh sau khi `lib.rs` đổi.

### Tám cái bẫy — sáu trong tám cho ra một lượt CI XANH với hành vi sai

#### Bẫy 1 — Ghi đè theo *tập* thay vì theo *khoá* 🔴 nguy hiểm nhất

Cài đặt sai, và nó **trông rất hợp lý**:
```rust
if let Some(w) = work { if !w.is_empty() { return w.clone(); } }
global.clone()
```
Test viết cẩu thả *(global có `a`, work có `a` khác giá trị, khẳng định kết quả là giá trị của work)* **xanh**. Hành vi thật: người dùng có 412 mục Glossary toàn cục, thêm **một** mục riêng cho Tác phẩm, và **411 mục kia biến mất**.

AD-18 viết *"tầng Tác phẩm thắng **theo từng thuật ngữ**"*. Story 3.4 nói cùng luật rõ hơn: *"áp **cả hai**, tầng Tác phẩm thắng khi trùng"*. Test bắt buộc: **một khoá chỉ có ở Global phải còn trong kết quả, mang `tier: Global`.**

#### Bẫy 2 — Sắp xếp theo tầng trước rồi mới theo xuất xứ

AD-18 khai tường minh khoá **chính** là xuất xứ, khoá **phụ** là tầng, và giải thích vì sao: *"một cặp TM toàn cục do **chính người dùng** dịch vẫn giống văn phong của họ hơn một cặp Tác phẩm do người khác dịch."* Đảo hai khoá cho ra một danh sách **trông có thứ tự** và hỏng đúng mục đích của FR70. AD-18 còn nói trước hậu quả: *"Không khai thứ tự này thì Giai đoạn 4 và Giai đoạn 6 sẽ cài lệch nhau."*

⚠️ Và `sort_unstable_by` phá thứ tự trong nhóm bằng nhau — dùng `sort_by`.

#### Bẫy 3 — Quên sửa `store_contract.rs` — hoặc sửa quá tay 🔴

`store_contract.rs` có **năm** chỗ khẳng định `schema_version()` và **ba** chỗ đếm `schema_migration_log`. Chỉ **một** test chạy trên `GLOBAL_MIGRATIONS` thật (`a_fresh_database_migrates_up_to_target_and_logs_it`, `:752`, qua `spec_with`); ba test khác dùng fixture cục bộ qua `spec_with_migrations` và **phải giữ nguyên**; một test tính `target` động nên **tự đúng**.

Hai hướng hỏng, cả hai đều thật:
- **Sửa thiếu** ⇒ `cargo test` đỏ ngay, rẻ.
- **Sửa thừa** ⇒ đổi luôn các con số trong `TWO_STEP`/`BROKEN_STEP_TWO` cho *"nhất quán"*, và hai ca nghiệm thu **sao lưu trước khi di trú** cùng **rollback khi bước gãy** im lặng mất hiệu lực. Đây là hướng đắt: CI vẫn xanh.

Đây **không** phải test hỏng — nó đang làm đúng việc của nó: canh rằng số phiên bản đổi là một quyết định có người ký, không phải hiệu ứng phụ.

#### Bẫy 4 — Token `rusqlite` trong một comment đuôi dòng

`store_boundary.rs:54` cấm hai chuỗi `"rusqlite"` và `"Connection::open"` ngoài `core/store/**`. Bộ quét **miễn trừ dòng bắt đầu bằng `//`**, nhưng Story 1.7 đã ghi lại nguyên văn: *"Comment đuôi dòng vẫn bị bắt."* Dùng các kiểu **tái xuất** từ `core::store` (`Transaction` · `SqlError` · `SqlResult` · `Row` · `ReadHandle` tại `store/mod.rs:98-117`) và đừng gõ tên crate ở đâu cả.

#### Bẫy 5 — Hợp âm đọc từ đĩa gây xung đột ⇒ ứng dụng không mở được

`keys.ts:270` phát hiện hai command trùng hợp âm và `createKeymap` **ném**. `installCommands` chạy **trước `mount()`** (`main.ts`, khối *"THỨ TỰ BẮT BUỘC #2"*). Một `global.db` sửa tay là đủ để một cú ném ở đó cho ra **cửa sổ trắng**, đúng lớp lỗi mà `applyTheme()` đã có chốt để chặn ở Story 1.4. Bắt, ghi chẩn đoán, rơi về mặc định. Giải quyết xung đột cho người dùng là **Story 1.21**.

#### Bẫy 6 — Thêm một `import` vào `src/commands/**`

`check-commands.mjs` (Kiểm C/D/E) và `check-i18n.mjs` (Kiểm E) **nạp thẳng các tệp `.ts` đó bằng Node thuần** (type-stripping, Node ≥ 22.18). Một `import` giá trị của `vue`, của `.json`, hay của `@tauri-apps/api` giết **ba** phép kiểm hành vi cùng lúc. Đó chính là lý do `installCommands({ setMode })` nhận phụ thuộc bằng tiêm — Story 1.6 §Completion Notes. **`bindings` đi vào cùng cửa đó**, ⛔ không phải bằng một `invoke` trong `index.ts`.

#### Bẫy 7 — Ký tự có dấu tiếng Việt ở vị trí mã trong `.rs`

`check-i18n.mjs` Kiểm A quét `src-tauri/**/*.rs`. `src-tauri/tests/**` miễn trừ **có tên**; `src/core/**` thì **không**. Doc-comment và comment có dấu là hợp lệ; `Display`, thông báo chẩn đoán và **`debug_assert!`** thì không. Cổng này đã bắt `core/i18n/mod.rs` một lần rồi.

#### Bẫy 8 — Coi `State<Store>` là luôn có

`open_global_store` ghi chẩn đoán rồi **đi tiếp** khi mở kho thất bại (`lib.rs:84-116`), nên `app.manage(store)` **có thể chưa từng chạy**. Một `state::<Store>()` thẳng tay sẽ panic — và `panic = "abort"` giết tiến trình. Dùng `try_state`, và nhánh `None` chính là bề mặt lỗi mà story này nợ người dùng.

### Quyết định thiết kế — đã chốt, không phải lựa chọn của dev

#### #1 — `ScopeResolver` sở hữu **ngữ nghĩa**, không sở hữu **kho** của mọi loại

Cám dỗ tự nhiên: một bảng `config_value` cho tất cả, và `ScopeResolver` là bộ CRUD của nó. Sai, vì Glossary có phân loại/xuất xứ/vòng đời ba trạng thái (Story 3.1), TM có cặp văn bản + xuất xứ (AD-6), luật làm sạch có mẫu regex + cờ bật tắt (Story 6.5). Nhét chúng vào một cột `value TEXT` là dựng một lược đồ EAV mà bốn epic sau phải bóc ra.

**Chốt:** `core::scope` phơi ra ngữ nghĩa + ba hàm phân giải, nhận **dữ liệu đã nạp** từ chỗ gọi. Mỗi module miền tự sở hữu bảng của nó và tự nạp hai tầng, rồi đưa qua `ScopeResolver`. `config_value` phục vụ **riêng** ba loại `GlobalOnly` của story này.

#### #2 — Chín loại, ba ngữ nghĩa; ba loại cuối là **hàng mới**

| `ScopeKind` | Ngữ nghĩa | Nguồn | Chủ sở hữu dữ liệu |
|---|---|---|---|
| `Glossary` | `Override` | AD-18 · FR46 | Epic 3 |
| `Prompt` | `Override` | AD-18 · FR69 | Epic 4 |
| `AiConfig` | `Override` | AD-18 · FR68 | Epic 4 |
| `TranslatorName` | `Override` | AD-18 · FR131 | Epic 8 |
| `TranslationMemory` | `Merge` | AD-18 · FR57 | Epic 7 |
| `ImportCleanupRule` | `Merge` | AD-18 · FR124 | Epic 6 |
| `Shortcut` | **`GlobalOnly`** | FR103 · AC5 | Story 1.21 |
| `LayoutPreset` | **`GlobalOnly`** | FR103 · AC5 | Story 1.14 |
| `AppConfig` | **`GlobalOnly`** | AC5 *(theme, chế độ cuối)* | Story này |

Ba hàng cuối **không có trong bảng AD-18**, và đó chính là ca mà AC4 tồn tại để bắt. FR103 liệt kê phím tắt và preset bố cục ở tầng Global **và không có đối ứng ở tầng Tác phẩm**; mockup `settings.html:246` viết thẳng: *"Phím tắt chỉ tồn tại ở tầng Toàn cục — một thao tác không nên đổi phím theo từng Tác phẩm."*

Khai chúng là `Override` là **sai im lặng**: nó cho phép một tầng Tác phẩm mà UX đã cấm, và Story 1.14/1.21 sẽ dựng thanh chuyển phạm vi cho một thứ không nên có. `GlobalOnly` là hàng thứ ba, và nó là **mở rộng bảng AD-18** — xem §Câu hỏi cho Ice #1.

#### #3 — `Override` là hợp nhất khoá + Work thắng trên khoá trùng

```
kết quả = tất cả khoá của Global ∪ tất cả khoá của Work
với mỗi khoá:  có ở Work  ⇒ value = Work,   tier = Work,   shadowed = giá trị Global (nếu có)
               chỉ ở Global ⇒ value = Global, tier = Global, shadowed = None
```
`shadowed` **không phải trang trí**. `mockups/settings.html:172` vẽ *"Ghi đè Toàn cục — ở tầng Toàn cục đang là **Anthropic**"* ngay cạnh giá trị đang thắng; `mockups/glossary-manage.html:169` vẽ mục toàn cục *"đang bị che"*. Không mang `shadowed` từ hôm nay thì hai màn hình đó phải tự truy vấn lại tầng Global — tức đúng cái *"một truy vấn riêng"* mà Story 3.1 cấm.

#### #4 — Tầng **luôn** là khoá phụ; khoá chính do chỗ gọi cấp

`core::scope` ⛔ **không được biết** *xuất xứ* là gì — đó là dữ liệu trên bản ghi TM (FR118, Story 7.2). Kéo nó vào đây là kéo miền TM vào bộ phân giải.

**Chốt:** `resolve_merge` nhận `primary: Option<&dyn Fn(&V,&V) -> Ordering>` và **luôn** áp tầng làm khoá phụ *(Work trước Global)*, kể cả khi `primary` là `None`. Chỗ gọi không có cách nào tắt vế tầng — đó là vế của AD-18 mà story này cưỡng chế.

#### #5 — Một bảng `config_value`, không phải ba bảng

Story 1.7 khoá quy tắc: *"⛔ Không thêm bước cho một lược đồ chưa tồn tại. **Mỗi story sở hữu bước di trú của chính nó, cùng lúc với bảng mà nó cần.**"* Dựng `keybinding` + `layout_preset` + `app_config` hôm nay là dựng ba bảng cho hai tính năng chưa tồn tại (1.14, 1.21).

**Chốt:** một bảng khoá-giá trị có cột `kind`, phục vụ đúng ba loại `GlobalOnly`. Khi Story 1.14/1.21 có mô hình thật, chúng thêm bước di trú của **chúng**. AC5 vẫn nghiệm thu được thật: ghi một hàng `('shortcut', 'mode.library', 'Mod+1', …)` vào `global.db` rồi khẳng định đường phân giải trả đúng nó.

#### #6 — Hàm thuần theo `Option<&Store>`, `#[tauri::command]` chỉ là vỏ

`bootstrap_config(store: Option<&Store>)` là thứ **test gọi được** mà không cần webview, và nó là **đường sản phẩm thật** — không phải một fixture. Đó là điều kiện để chữa `ipc_error_wire_shape` cho tử tế: `deferred-work.md:49` cấm dựng command giả để đóng nợ, và một hàm thuần mà command thật bọc lại thì **không phải** command giả.

Đây cũng là AD-1 đọc đúng: `commands/` là adapter, quy tắc nằm dưới nó.

#### #7 — Không thêm khoá `MessageKey` nào, không thêm chuỗi `vi.json` nào

Mọi lỗi story này phát ra đều là lỗi kho: kho vắng mặt ⇒ `store.open_failed`; đọc hỏng ⇒ `store.read_failed`; ghi hỏng ⇒ `store.write_failed`. Cả năm khoá **đã có** từ Story 1.7 kèm `From<StoreError> for IpcError` và test `every_store_error_converts_to_a_complete_ipc_error`. Story 1.7 §Completion Notes #3 khoá quy tắc: *"⛔ Không khoá nào cho tính năng chưa tồn tại."*

`ScopeError` (`WrongSemantics`, `WorkTierForbidden`) là **lỗi lập trình, không phải lỗi người dùng** — nó ⛔ **không** `impl From<..> for IpcError` và ⛔ không bao giờ vượt ranh giới IPC. Viết một test khẳng định mọi nhánh lỗi của hai hàm command chỉ sinh ra `IpcError` dẫn xuất từ `StoreError`.

#### #8 — Trả về kế thừa = **XOÁ hàng tầng Work**, không phải chép giá trị Global xuống

Chưa cài hôm nay (không có tầng Work), nhưng **phải ghi vào doc-comment của `Resolved`** vì hai đường này không phân biệt được ở khoảnh khắc bấm nút và phân kỳ mãi mãi sau đó: chép giá trị xuống làm mục đó **đóng băng** ở giá trị Global cũ, và lần sau người dùng đổi cấu hình chung thì Tác phẩm này im lặng không theo. `settings.html:228` có nút *"Trả toàn bộ mục về kế thừa"*; ai cài nó sau này phải đọc được luật này ở đây.

#### #9 — Không cache

`ScopeResolver` chưa cache gì. Consumer đường nóng duy nhất là khớp Glossary khi gõ (**Story 3.4**, dưới trần NFR2 *không frame nào vượt 50 ms*), và hôm nay nó chưa tồn tại. Dựng cache bây giờ là dựng một cơ chế vô hiệu hoá mà không có gì để vô hiệu hoá. Ghi vào `deferred-work.md`, giao **Story 3.4**.

#### #10 — Bố cục panel: state sống ở frontend, **preset đã lưu** là dữ liệu của Rust

AD-1 liệt kê *"bố cục panel"* là state UI của frontend; FR103 nói preset bố cục sống ở `global.db`. Hai câu này không mâu thuẫn nếu tách đúng: **bố cục đang hiển thị** là của frontend; **preset đã đặt tên và lưu lại** là dữ liệu Rust, đọc qua `ScopeResolver`, ghi qua `store::Writer`. Ghi ra đây vì Story 1.14 sẽ phải chọn, và cách đọc kia dẫn tới `localStorage`.

### Bàn giao từ các story trước — thứ ảnh hưởng trực tiếp

- **`Store::write` / `Store::read`** (`store/mod.rs:556`, `:576`) — mỗi job ghi là **một giao dịch**; `Ok` ⇒ commit, `Err` ⇒ rollback. Đường đọc đặt `query_only = 1` nên `INSERT` qua nó **thất bại**, và đó là bằng chứng dương chứ không phải phiền nhiễu.
- **Gọi `Store::write` lồng nhau trả `WriteFailed` ngay** (`writer.rs:104`, chốt thêm trong lượt review 1.7) — ⛔ không xếp hàng, không deadlock. Đừng lồng.
- **`StoreSpec.migrations` là một TRƯỜNG**, không phải hằng tra theo `kind` (1.7 §Completion Notes #2) — Story 1.15 dùng đúng trường đó cho `project.db`.
- **`ReadHandle<'a>` là bí danh của `&rusqlite::Connection`**, ⛔ không phải kiểu bọc (`store/mod.rs:117`).
- **`IpcError::new` là hàm dựng duy nhất**; trường private; ⛔ không `#[serde(rename_all = "camelCase")]`; thiếu param ⇒ `debug_assert!` ở debug, rơi về `MessageKey::Unknown` ở release, ⛔ **không bao giờ panic**.
- **`params` mang DỮ LIỆU, không mang câu** — `detail` thô của SQLite ⛔ không đi vào `params` (1.7 §Completion Notes #5).
- **`vi.json` phẳng, khoá chấm** — lồng đối tượng làm hỏng cả `BTreeMap<String,String>` phía Rust lẫn Kiểm B.
- **`assetProtocol.scope` ⛔ không bao giờ chứa `$APPDATA`** (test `asset_protocol_scope_never_contains_appdata`) — webview **không đọc được** `global.db`. Mọi cấu hình tới frontend **chỉ** qua IPC.
- **Sáu con số `Tuning` chưa cái nào được đo** — chủ sở hữu **Story 2.4**. ⛔ Đừng "hiệu chỉnh" chúng ở đây.
- **`writer.rs:159`** — `Writer::shutdown()` không có trần cho `handle.join()`; Ice chấp nhận rủi ro 2026-08-04 **với điều kiện review tay mỗi khi một story mới ghi qua tầng này**. Story này là một story như vậy — nêu trong Completion Notes.
- **`pragmas.rs:249`** — lỗi checkpoint/sao lưu lúc chạy đều gắn nhãn `OpenFailed`. Story này là story đầu đưa chẩn đoán kho lên giao diện: ⛔ **đừng** phơi chẩn đoán checkpoint ra UI, nếu không một checkpoint lỗi sau nhiều giờ sẽ hiện nhầm *"Không mở được kho dữ liệu"*.
- **`schema.rs:137`** — sao lưu bằng `fs::copy` không nguyên tử và không xác minh lại. Story này là **lượt di trú thật đầu tiên trên một `global.db` đã có dữ liệu**, nên đây là lần đầu đường sao lưu chạy thật. Không sửa, nhưng ghi lại quan sát.
- **`tests/**` được miễn trừ khỏi phép quét ranh giới** (`deferred-work.md:179`) — nếu test mới của story này chạm `rusqlite` trực tiếp thì mục đó **mở lại**; nói ra trong Completion Notes.

### Yêu cầu UX — ba màn hình tương lai đang phụ thuộc vào hình dạng API hôm nay

Story này ⛔ **không dựng giao diện nào**. Nhưng ba mockup đã kiểm toán quy định hình dạng dữ liệu, và bỏ sót chúng hôm nay là buộc ba story sau tự truy vấn vòng qua `ScopeResolver`:

1. **`mockups/settings.html`** — *"thanh scope — trái tim của FR103"* (`:26`). Trong **cùng một** cấu hình AI, có trường `ghi đè` và có trường `kế thừa` **cùng lúc** (`:172`, `:188`, `:200`). ⇒ ghi đè là **theo trường**, không theo cả struct. Còn đòi đếm được *"2 mục đang ghi đè"* (`:140`) và hiện giá trị Global bên cạnh giá trị đang thắng (`:172`).
2. **`mockups/glossary-manage.html:169`** — mục tầng Global hiện inline với nhãn *"đang bị che"*. ⇒ `shadowed` phải có mặt trong kết quả `Override`.
3. **`EXPERIENCE.md:216`** *(UX-DR40)* — mỗi luật làm sạch mang **nhãn tầng**, và cả hai tầng cùng áp. ⇒ nhãn tầng nằm trên **từng mục** của kết quả `Merge`.

`EXPERIENCE.md:23` khoá chỗ đứng của phần mã này: *"Tách câu, khớp ngôn ngữ, **phân giải scope** đều nằm ở Rust."*

### Testing standards

- Test Rust ở `src-tauri/tests/` *(integration, `use auratranslate_lib::…`)*; **khai phạm vi ở dòng 1**; một tệp một mối quan tâm. Story này thêm **hai**: `scope_contract.rs` *(hành vi)* và `scope_boundary.rs` *(ranh giới cây nguồn)*.
- **Nghiệm thu đỏ-rồi-xanh bắt buộc**, ≥ 2 đối chứng âm mỗi cơ chế.
- Tên test là câu tiếng Anh đầy đủ — tiền lệ `a_newer_schema_is_refused_without_touching_a_single_byte`, `only_core_store_may_name_rusqlite`.
- **Mọi test quét cây nguồn phải có sàn quần thể** + **đối chứng dương**. *"Cây rỗng đọc thành sạch"* là cách một cổng chết im lặng ngay ngày ra đời.
- `cargo test` chạy song song trong một tiến trình ⇒ mỗi ca một thư mục tạm riêng; **drop `Store` trước** khi xoá (Windows/NFR14); ⛔ không `sleep` dài; ⛔ không `tempfile`.
- **Tám lệnh phải exit 0 trước khi báo xong**; `check:scope` + `check:scope:bundled` chạy **lại bằng tay** vì `lib.rs` đổi *(chúng chạy binary với trần thời gian và đọc dòng `VERDICT:` — một `Store::close` chậm làm chúng đỏ vì lý do không liên quan)*.
- Frontend **không có test runner** và ⛔ story này không được thêm một cái (NFR15 đòi rà giấy phép trước). Hành vi frontend nghiệm thu bằng các cổng `.mjs` sẵn có.

### Thông tin kỹ thuật — không có phụ thuộc mới

Story này thêm **0** phụ thuộc. Mọi thứ cần đã ghim trong `src-tauri/Cargo.toml`: `tauri = "=2.11.5"` *(feature `protocol-asset`)*, `serde = "=1.0.229"` *(feature `derive`)*, `serde_json = "=1.0.151"`, `rusqlite = "=0.40.1"` *(feature `bundled`, chỉ dùng gián tiếp qua `core::store`)*. Frontend: `@tauri-apps/api 2.11.1` đã có sẵn — `invoke` lấy từ `@tauri-apps/api/core`.

⚠️ **Điều này là ràng buộc, không phải quan sát.** Consistency Conventions: *"Mỗi phụ thuộc mới phải rà tương thích GPLv3 **trước khi** thêm vào (NFR15) và ghi vào bảng Stack."* Và sáu tên bị cấm — trong đó **`tauri-plugin-sql`** *(vì AD-11)* và **`tauri-plugin-fs`** *(vì AD-1 + AD-29)* — được cưỡng chế bằng `npm run check:deps` chạy `cargo tree -i`, không bằng kỷ luật.

`rusqlite` 0.40.1 tắt feature `backup` và `hooks`; SQLite `bundled` đo được ở Story 1.7 là **3.53.2**. `edition = "2024"`, `rust-version = "1.85"`, toolchain CI ghim `1.97.1`.

### Nợ nhận lại — ba mục `deferred-work.md` đóng ở story này

| Mục | Nội dung | Đóng bằng |
|---|---|---|
| `:49` | `ipc_error_wire_shape` là **mệnh đề vòng** — quét chính fixture nó tự dựng, không quan sát đường sản phẩm nào | Task 7 — serialize giá trị `bootstrap_config(None)` trả về |
| `:140` | Chế độ mặc định lúc khởi động là `library` và **không phép kiểm nào canh**; 1.8 nạp lựa chọn từ đĩa sẽ chạm chỗ này | Task 9 — nạp + lưu chế độ cuối |
| `:177` | Lỗi mở kho **chỉ ra `stderr`**, không tới người dùng; *"Story 1.8 chỉ phải nối dây"* | Task 7 + Task 9 — nhánh `None` ⇒ `IpcError` ⇒ dải báo lỗi |

⛔ Mục `:128` *(xung đột `⌘1`/`⌘2` giữa preset bố cục và chế độ)* là **Story 1.14**, không phải story này. ⛔ Mục `:160-171` *(sáu con số `Tuning`)* là **Story 2.4**.

### Project Structure Notes

Cây mới, khớp `ARCHITECTURE-SPINE.md#Cây nguồn` *(dòng `scope/ # ScopeResolver (AD-18)`)*:

```text
src-tauri/src/core/scope/
  mod.rs        # ScopeResolver · Tier · ScopeError · điểm vào duy nhất   (AD-18)
  kinds.rs      # scope_kinds! · ScopeKind · Semantics · bảng ngữ nghĩa    (AC4)
  resolve.rs    # ba hàm phân giải, THUẦN, không chạm đĩa             (AC2, AC3)
  store.rs      # nạp/ghi tầng Global qua Store, không gõ tên rusqlite     (AC5)
src-tauri/src/commands/
  config.rs     # hai #[tauri::command] đầu tiên của dự án               (AC5)
src-tauri/tests/
  scope_contract.rs   # hành vi
  scope_boundary.rs   # AC1 vế "cưỡng chế bằng test"
src/config/
  bootstrap.ts  # bọc invoke, có nhánh không-có-Tauri
```

Hình dạng nhiều tệp trong một thư mục module là khuôn spine đã dùng cho `webimport/` và Story 1.7 đã dùng cho `store/`. Rust `snake_case` theo Consistency Conventions.

**Không có chỗ lệch nào so với cây nguồn đã khai.** ⛔ Không thêm thư mục thứ mười ba vào `core/`; ⛔ không khai trait nào trong `ports/`; ⛔ không đụng `src/layout/` *(Story 1.14)*.

⚠️ `src/config/` **không** có trong cây nguồn của spine — spine chỉ liệt kê `modes/ panels/ layout/ commands/ tokens/ i18n/`. Đây là một thư mục frontend mới. Lý do chấp nhận: nó không phải một khái niệm miền mới mà là **adapter IPC phía webview**, và đặt nó vào `src/commands/` sẽ phá Bẫy 6. Nêu trong Completion Notes để lượt review phân xử.

### References

- `_bmad-output/planning-artifacts/epics.md:1298-1328` — năm AC nguyên văn của Story 1.8
- `epics.md:1254-1296` *(Story 1.7)* · `:1533-1578` *(1.14)* · `:1580-1623` *(1.15)* · `:1835-1868` *(1.21)* — bốn ranh giới liền kề
- `epics.md:421` — dòng bất biến AD-18 của Epic 1 · `:409-416` — bốn dòng bất biến lưu trữ/ghi/di trú
- `epics.md:296` FR103 · `:282-298` FR96–FR104 · `:723` bảng truy vết FR103 → `ScopeResolver`
- `epics.md:2352-2391` *(3.1)* · `:2501-2503` *(3.4)* · `:2703` *(3.9)* · `:2820-2855` *(4.2)* · `:2899-2933` *(4.4)* · `:4039-4080` *(6.5)* · `:4765-4802` *(7.3)* · `:5378-5381` *(8.7)* — **tám** consumer tương lai
- `epics.md:383` — *"Đúng ba cổng, không hơn"* · `:381` — *"Ranh giới kiến trúc phải được cưỡng chế bằng test, không bằng kỷ luật"*
- `ARCHITECTURE-SPINE.md#AD-18` *(:236-255)* — bảng sáu loại, lý do luật làm sạch là hợp nhất, **thứ tự TM hai khoá**, luật ghi TM một chiều
- `ARCHITECTURE-SPINE.md#AD-2` — ba cổng, `ScopeResolver` **không phải** cổng · `#AD-40` — tiền lệ *"không trait hoá"*
- `ARCHITECTURE-SPINE.md#AD-7` — năm loại kho · `#AD-8` — ⛔ cấu hình không nằm ở `library-index.db` · `#AD-9` — `.atproj` là thư mục
- `ARCHITECTURE-SPINE.md#AD-11` · `#AD-12` · `#AD-30` — writer nối tiếp, PRAGMA, di trú chỉ tiến
- `ARCHITECTURE-SPINE.md#AD-21` — hình dạng lỗi bốn trường · `#AD-1` — bố cục panel là state UI · `#AD-34` — `CommandRegistry` · `#AD-43` — tên người dịch *"qua `ScopeResolver` theo AD-18"*
- `ARCHITECTURE-SPINE.md#Consistency Conventions:538` — cấm `Project` cho `Work` · `:554` — *"Phân giải hai tầng luôn qua `ScopeResolver`"* · `:557` — rà GPLv3 trước khi thêm phụ thuộc
- `ARCHITECTURE-SPINE.md#Structural Seed:699` — `scope/ # ScopeResolver (AD-18)` · `#Capability Map` — `core/scope/` thuộc **C4, C5, C9**
- `ARCHITECTURE-SPINE.md#Stack:609-611` — sáu tên bị cấm, cưỡng chế bằng `check:deps`
- `reviews/review-adversarial-2026-08-03b.md:49-59` §F4 — hai loại dữ liệu từng tới `ScopeResolver` mà **không có hàng ngữ nghĩa**
- `prds/prd-AuraTranslate-2026-08-02/prd.md:765-774` — FR103 dạng **bảng**, có mệnh đề *"ngôn ngữ nguồn (cố định, đặt lúc tạo)"* mà `epics.md:296` làm rơi mất · `:212` — định nghĩa *Scope* trong Glossary
- `prd.md:814-818` NFR1/NFR2/NFR4/NFR5 · `:829-830` NFR9/NFR10 · `:853` NFR16 · `:863` NFR17
- `ux-designs/…/EXPERIENCE.md:23` — *"phân giải scope nằm ở Rust"* · `:216` UX-DR40 nhãn tầng · `:340` — màn Cài đặt ánh xạ FR103
- `mockups/settings.html:26,123-142,172-228,243-247` — thanh scope, ghi đè/kế thừa theo trường, *"Phím tắt chỉ tồn tại ở tầng Toàn cục"*
- `mockups/glossary-manage.html:121,169,207,214` — bộ lọc ba chiều, mục *"đang bị che"*, thao tác đẩy lên tầng Global
- `src-tauri/src/core/scope/mod.rs` — doc-comment giao việc, **giữ nguyên và viết tiếp**
- `src-tauri/src/core/store/mod.rs:98-117` *(tái xuất)* · `:238` `StoreSpec::global` · `:433-456` `From<StoreError> for IpcError` · `:556`/`:576` `write`/`read` · `:122-134` ⛔ chỉ `Global` có mã khởi tạo
- `src-tauri/src/core/store/schema.rs:38-45` — *"cấu hình là Story 1.8"* · `:52` DDL · `:65` `Migration` · `:76` `GLOBAL_MIGRATIONS` · `:85` `target_version`
- `src-tauri/src/core/i18n/mod.rs:62-91` `message_keys!` *(khuôn cho `scope_kinds!`)* · `:100-134` danh mục khoá · `:228` `IpcError::new`
- `src-tauri/src/lib.rs:30` `GLOBAL_DB_FILE` · `:37-50` builder *(chỗ cắm `invoke_handler`)* · `:74-77,:111` — *"Bề mặt hiển thị thuộc Story 1.8"* · `:84-116` `open_global_store`
- `src-tauri/src/commands/mod.rs` — quy tắc adapter, ⛔ đừng nhầm với `src/commands/`
- `src-tauri/tests/store_boundary.rs:37,44,54,106-214` — khuôn cổng quét, sàn, đối chứng dương/âm
- `src-tauri/tests/store_contract.rs:752-770` 🔴 — test **duy nhất** chạy trên `GLOBAL_MIGRATIONS` thật; `:714` doc-comment thành sai; `:718-741` `TWO_STEP`/`BROKEN_STEP_TWO` ⛔ giữ nguyên; `:921` tự đúng theo
- `src-tauri/tests/ipc_contract.rs:73-78,128-137` 🔴 — mệnh đề vòng phải chữa
- `src-tauri/tests/config_invariants.rs:333,368` — ⛔ `capabilities/main.json` bị khoá đúng ba quyền
- `src/main.ts:18` — *"Giao diện chọn theme và việc lưu lựa chọn xuống đĩa thuộc Story 1.8"* · `src/tokens/index.ts:64` · `src/modes/modeState.ts:39` — ba lời hứa *"nạp từ đĩa"* ghi sẵn trong mã
- `src/commands/index.ts:129-186` — `installCommands(deps)`, hợp âm literal ở `:141` · `src/commands/keys.ts:253,270` — `createKeymap`, phát hiện trùng hợp âm
- `src/i18n/index.ts:72` `tError` · `src/i18n/vi.json` — 16 khoá
- `scripts/check-i18n.mjs:228` `RS_FLOOR` · `scripts/check-commands.mjs:174-175` sàn frontend
- `deferred-work.md:49,140,177` — ba mục đóng ở story này · `:128` *(1.14)* · `:160-190` *(2.4 và các mục review 1.7)*
- `.github/workflows/ci.yml:466-499` — khối *"CHỖ MÓC CHO EPIC SAU"*; ⛔ không thêm workflow thứ hai
- `1-7-…md:435-444` §Testing standards · `:636-667` §Completion Notes · `:190-214` §Review Findings

---

### Câu hỏi cho Ice — đã có mặc định, không chặn

1. **`Semantics::GlobalOnly` — có phải là hàng thứ ba hợp lệ của AD-18 không?**
   FR103 đặt phím tắt và preset bố cục ở tầng Global **và không cho chúng đối ứng ở tầng Tác phẩm**; `settings.html:246` nói thẳng *"một thao tác không nên đổi phím theo từng Tác phẩm"*. Bảng AD-18 chỉ có hai ngữ nghĩa, nên khai chúng bằng một trong hai đều sai: `Override` mở một tầng Tác phẩm mà UX đã cấm, `Merge` thì vô nghĩa.
   → **Mặc định: thêm `GlobalOnly` và ghi ba hàng mới vào bảng AD-18.** Đây là **sửa tầng kiến trúc**, cùng loại với lượt vá F4 đã thêm hai hàng cho FR124/FR131 — nên nó cần Ice ký, không phải dev tự quyết. ⛔ Nếu Ice bác, đường thay thế **duy nhất** là không đăng ký ba loại đó với `ScopeResolver`, và khi đó AC5 phải phân giải bằng một đường riêng — tức đúng thứ AC1 cấm.

2. **`ngôn ngữ nguồn` có phải một loại của `ScopeResolver` không?**
   FR103 liệt kê nó ở tầng Tác phẩm, nhưng Story 5.1 định nghĩa nó là trường **bất biến** trong `meta.json`, đặt lúc tạo và *"không đổi được"*, và nó **không có đối ứng ở tầng Global** — nên không có gì để ghi đè. Bản PRD còn ghi rõ *"(cố định, đặt lúc tạo)"*, mệnh đề mà `epics.md:296` làm rơi mất.
   → **Mặc định: KHÔNG đăng ký.** Nó là thuộc tính của `Work`, không phải cấu hình hai tầng. Ghi lý do vào doc-comment của `scope_kinds!` để Story 1.15 không thêm vào.

3. **Ghi đè `AiConfig` theo từng trường hay theo cả struct?**
   AD-18 chỉ ghi *"Cấu hình AI | ghi đè"*. Story 4.2 cũng chỉ nói *"ghi đè được theo Tác phẩm đó"*. Chỉ `settings.html` lộ ra rằng trong **cùng một** cấu hình có trường `ghi đè` và trường `kế thừa` cùng lúc (`:172`, `:188`, `:200`) — tức **theo từng trường**.
   → **Mặc định: theo từng trường**, tức `AiConfig` phân giải như một map khoá→giá trị y hệt Glossary, không phải một struct nguyên khối. Đây đúng là hình dạng mơ hồ mà F4 đã cắn một lần; chốt bằng chữ ở đây rẻ hơn chốt ở Epic 4.

4. **`src/config/` — thư mục frontend mới, có chấp nhận không?**
   Cây nguồn của spine không có nó. Đặt lời gọi `invoke` vào `src/commands/` sẽ giết ba phép kiểm hành vi (§Bẫy 6); đặt vào `src/modes/` thì sai khái niệm.
   → **Mặc định: tạo `src/config/`**, một tệp, khai rõ trong doc-comment rằng nó là adapter IPC chứ không phải một khái niệm miền mới.

---

## Dev Agent Record

### Agent Model Used

`claude-opus-5` (Claude Code) — 2026-08-04.

### Debug Log References

#### Đường cơ sở (Task 1) — tám lệnh trên cây sạch tại `HEAD = 0ff36a0`

| Thứ | Story dự đoán | **Đo được** |
|---|---|---|
| `npm run build` | exit 0 | ✅ exit 0 |
| `cargo test` | exit 0 | ✅ exit 0 |
| `check:deps` · `check:tokens` · `check:i18n` · `check:commands` · `check:scope` · `check:scope:bundled` | exit 0 | ✅ cả sáu exit 0 |
| `.rs` dưới `src-tauri/src/**` | 22 | **22** ✅ |
| Quần thể `check-i18n.mjs` *(sau miễn trừ)* | 23 | **23** ✅ |
| Khoá `vi.json` | 16 | **16** ✅ |
| Tổng test Rust | 41 | 🔴 **40** — xem ghi chú dưới |

🔴 **Một sai lệch trong story file, đã kiểm lại:** §Trạng thái repo ghi *"Test Rust — **41** — `config_invariants` 15 · `ipc_contract` 5 · `store_contract` **17** · `store_boundary` 4"*. Số thật là **40**: `store_contract` có **16** ca, không phải 17. Tổng của chính bốn số đó cũng là 40 — con số 41 là lỗi cộng ở story file, ⛔ không phải một ca test bị mất. ⛔ Không sửa gì để "đạt" con số cũ.

#### Sau story (Task 11) — tám lệnh, lần cuối

| Lệnh | Kết quả |
|---|---|
| `npm run build` | ✅ exit 0 · 44 module *(43 trước)* |
| `cargo test` | ✅ exit 0 · **62 test** *(40 trước, **+22**)* |
| `check:deps` · `check:tokens` · `check:i18n` · `check:commands` | ✅ exit 0 |
| `check:scope` | ✅ exit 0 · `VERDICT: PASS` — **chạy tay lại vì `lib.rs` đã đổi** |
| `check:scope:bundled` | ✅ exit 0 · `VERDICT: PASS` — **chạy tay lại** |

Phân bố 62 test: `config_invariants` 15 · `ipc_contract` 5 · **`scope_boundary` 5** · **`scope_contract` 17** · `store_boundary` 4 · `store_contract` 16.

Quần thể sau story: `.rs` dưới `src-tauri/src/**` = **26** *(dự đoán 26 ✅)* · quần thể `check-i18n.mjs` = **27** *(dự đoán 27 ✅)* · khoá `vi.json` = **16** *(⛔ không thêm khoá nào, §Quyết định #7)*.

#### 🔴 Nghiệm thu đỏ-rồi-xanh — **17 đối chứng âm, 17 đỏ**

Mỗi lượt: gỡ đúng một thứ → chạy đúng một ca → trả lại. Tiền lệ: 1.5 (21 ca) · 1.6 (28 ca) · 1.7 (17 ca).

| # | Cơ chế | Gỡ cái gì | Ca phải đỏ | KQ |
|---|---|---|---|---|
| N1 | AC2 ghi đè | `merge_by_key` → *"work không rỗng thì trả work"* | `an_override_keeps_keys_that_only_exist_in_the_global_tier` | 🔴 |
| N2 | AC2 xuất xứ | `shadowed` luôn `None` | `an_override_carries_the_shadowed_value` | 🔴 |
| N3 | AC3 khoá phụ | đảo `primary` ↔ tầng trong `sort_by` | `the_tier_is_always_the_secondary_sort_key` | 🔴 |
| N4 | AC3 hợp nhất | tầng Global biến mất khi có tầng Work | `a_merge_keeps_both_tiers_without_deduplicating` | 🔴 |
| N5 | AC3 thứ tự tầng | `Tier::rank` đảo Work ↔ Global | `a_merge_without_a_primary_key_puts_work_before_global_and_stays_stable` | 🔴 |
| N6 | AC4 danh mục | thêm biến thể thứ mười vào `scope_kinds!` | `the_kind_table_has_every_variant` | 🔴 |
| **N7** | **AC4 trình biên dịch** | khai một biến thể **KHÔNG kèm ngữ nghĩa** | `cargo build` | 🔴 **không biên dịch được** |
| N8 | AC4 bảng AD-18 | `Shortcut: GlobalOnly` → `Override` | `the_semantics_table_matches_ad_18_row_by_row` | 🔴 |
| N9 | AC5 tầng bị cấm | bỏ chốt `WorkTierForbidden` | `a_global_only_kind_refuses_a_work_tier` | 🔴 |
| N10 | AC1 cổng ngữ nghĩa | `require()` luôn `Ok` | `calling_the_wrong_resolver_for_a_kind_is_refused` | 🔴 |
| N11 | Bẫy 4 | gõ `rusqlite` vào một **comment đuôi dòng** trong `core/scope/` | `core_scope_never_names_the_sqlite_layer` | 🔴 |
| N12 | AC1 vế test | `use …ScopeKind` trong `commands/config.rs` | `only_core_scope_may_name_the_two_tier_vocabulary` | 🔴 |
| N13 | Bẫy 3 | trả `store_contract` về `schema_version() == 1` | `a_fresh_database_migrates_up_to_target_and_logs_it` | 🔴 |
| N14 | Nợ `:49` | `bootstrap_config(None)` trả `Ok(<mặc định>)` | `ipc_error_wire_shape` | 🔴 |
| N15 | §Quyết định #1 | bỏ chốt `GlobalOnly` trong `save_value` | `every_command_error_comes_from_the_store_vocabulary` | 🔴 |
| N16 | Sàn quần thể | `store_boundary::RS_FLOOR` 20 → 27 *(trên số thật 26)* | `the_scanned_tree_is_large_enough_to_be_real` | 🔴 |
| N17 | Sàn quần thể | `check-i18n::RS_FLOOR` 21 → 28 *(trên số thật 27)* | `npm run check:i18n` | 🔴 `abort()` |

⚠️ **Một đối chứng đã viết SAI ở lượt đầu, và đó là dữ liệu chứ không phải một sự cố.** Bản N4 đầu tiên chèn `out.dedup_by(|_, _| false)` — một predicate **không bao giờ đúng**, nên nó không khử gì và ca vẫn xanh. Cổng không yếu; **mutation** yếu. Viết lại thành *"tầng Global biến mất khi có tầng Work"* — đúng dạng Bẫy 1 áp cho `Merge` — thì đỏ ngay. Ghi ra vì một bảng đối chứng âm chỉ đáng tin nếu nó cũng ghi lại lượt mà chính nó sai.

⚠️ **N7 là đối chứng quan trọng nhất của AC4, và nó không phải một test.** AC4 đòi *"cưỡng chế bằng trình biên dịch, không bằng test và không bằng tài liệu"* — nên bằng chứng đúng là **mã không biên dịch được**, không phải một ca đỏ. ⚠️ Và ⛔ đối chứng *"thêm một nhánh `_ =>` vào `semantics()`"* mà story đề xuất **không cho ra một ca đỏ**: trên một `match` đã phủ hết biến thể, `_` là một nhánh **không tới được** — `rustc` cảnh báo, ⛔ không lỗi, và không test nào đổi kết quả. Phép bảo vệ thật nằm ở chỗ khác và mạnh hơn: **không tồn tại cú pháp nào khai được một biến thể mà không kèm ngữ nghĩa** (N7), nên không bao giờ có một biến thể thiếu nhánh để một `_ =>` che đi. Ghi ra để lượt review không đi tìm một ca đỏ không tồn tại.

### Completion Notes List

1. **Bốn câu hỏi cho Ice đã được ký 2026-08-04, tất cả theo mặc định của story.** (#1) `Semantics::GlobalOnly` là hàng thứ ba hợp lệ của AD-18 — **bảng AD-18 trong `ARCHITECTURE-SPINE.md:242` đã được cập nhật** với ba hàng mới cộng một khối lý do; (#2) `ngôn ngữ nguồn` ⛔ **không** đăng ký, lý do ghi vào doc-comment của `kinds.rs` **và** vào AD-18 để Story 1.15 không thêm lại; (#3) `AiConfig` ghi đè **theo từng trường** — cũng đã ghi vào AD-18, vì chốt bằng chữ ở đây rẻ hơn chốt ở Epic 4; (#4) tạo `src/config/`, lý do ghi ở `deferred-work.md` cho lượt review phân xử.

2. **Chín loại, ba ngữ nghĩa, sinh từ một khai báo.** `scope_kinds!` sinh `enum ScopeKind` · `ALL` · `as_str()` · `semantics()` · `from_wire()`. ⛔ Không `impl Default for Semantics`, ⛔ không `#[derive(Default)]`, ⛔ không nhánh `_ =>` trong bất kỳ `match` nào **trên `ScopeKind`**. ⚠️ `from_wire` **có** một nhánh `_ => None` và nó ⛔ không vi phạm luật đó: nó `match` trên `&str` — một tập vô hạn, không tin được, đến từ dây IPC và từ cột `kind` trên đĩa — nên nhánh cuối là bắt buộc, và nó trả `None` chứ ⛔ không đoán. Lý do ghi tại chỗ.

3. **`Tier`, ⛔ không `Project`.** Consistency Conventions `:538` cấm `Project` cho thực thể Tác phẩm. `Tier::rank()` là một **hàm** chứ không phải `#[derive(Ord)]` trên thứ tự khai báo, có chủ ý: thứ tự khai báo đọc tự nhiên là *Global rồi Work* (tầng dưới trước), còn thứ tự **sắp xếp** thì ngược lại — buộc hai thứ vào một `derive` nghĩa là đảo một cái sẽ im lặng đảo cái kia.

4. **`Resolved` và `Tiered` có trường RIÊNG TƯ**, dựng chỉ qua `pub(crate) new`. Không phải để giấu — bốn accessor phơi hết. Lý do là một bất biến: `tier == Global` thì `shadowed` **luôn** `None`, vì Global là tầng dưới cùng. `debug_assert!` canh nó ở profile debug *(và `cargo test` chạy ở debug)*.

5. **🔴 `merge_by_key` ⛔ không có đường nào trả về sớm với chỉ một tầng.** Đó là hình dạng mã, không phải một lời hứa: tầng Global vào trước **nguyên vẹn**, tầng Work `insert` đè lên **chỉ trên khoá nó thật sự có**. Bản đầu dùng `BTreeSet` hợp khoá rồi `.expect("key came from the union")` — đã **viết lại** để bỏ hẳn đường panic đó *(§Task 3: ⛔ không `panic!`, không `unwrap()`; `panic = "abort"` ở release)*. Bản hiện tại không có nhánh panic nào.

6. **⛔ Không thêm khoá `MessageKey` nào, ⛔ không thêm chuỗi `vi.json` nào** — `vi.json` vẫn đúng **16** khoá. `ScopeError` ⛔ **không** `impl From<..> for IpcError` và ⛔ không bao giờ vượt ranh giới IPC. Một `kind` lạ hoặc một loại không phải `GlobalOnly` đi vào `save_value` trả `StoreError::WriteFailed` — và đó **không phải một cách nói tránh**: không byte nào được ghi, `store.write_failed` nghĩa đen là *"thay đổi vừa rồi chưa được lưu"*, và `every_command_error_comes_from_the_store_vocabulary` khẳng định cả bốn nhánh lỗi **cộng** mệnh đề *"bảng vẫn rỗng sau hai lượt bị từ chối"*.

7. **⚠️ Tên command trên dây là tên hàm — nên vỏ `#[tauri::command]` sống trong một module `wire` lồng.** Frontend gọi `invoke('bootstrap_config')`, và cái tên đó đã thuộc về **hàm thuần**. Một hậu tố `_command` sẽ đổi tên trên dây và `invoke` không tìm thấy gì. ⛔ Không đảo hướng: hàm thuần là đường sản phẩm, vỏ là thứ bỏ đi được trong test. Khuôn này đã ghi vào `commands/mod.rs` cho mọi story sau.

8. **⛔ `capabilities/main.json` không đụng một dòng.** Trong Tauri v2, command do **chính ứng dụng** khai ⛔ không cần mục ACL — ACL canh command của **plugin**. `config_invariants.rs:333` vẫn khoá đúng ba quyền và vẫn xanh.

9. **§Bẫy 5 giải bằng một registry NHÁP, ⛔ không bằng cách nới một phép cưỡng chế đang đúng.** `createKeymap` ném khi hai command trùng hợp âm, và `installCommands` chạy **trước `mount()`** — một `global.db` sửa tay là đủ cho một cửa sổ trắng. Chốt: dựng thử keymap trên `createRegistry()` nháp; xung đột ⇒ ghi chẩn đoán rồi rơi về hợp âm mặc định. ⛔ Không thử trên registry thật rồi dọn, vì `register()` ném với id trùng (AC2 của Story 1.6) và đó là hành vi **đúng**. Màn giải quyết xung đột vẫn là **Story 1.21**; ở đây chỉ có *"đừng chết"*.

10. **⛔ Không một `import` nào thêm vào `src/commands/**`** (§Bẫy 6). `bindings` đi vào bằng **tiêm**, đúng cửa mà `setMode` đã đi từ Story 1.6. `check:commands` và `check:i18n` Kiểm E vẫn nạp được bộ command của sản phẩm bằng Node thuần — cả hai xanh.

11. **⚠️ `setMode` lúc khởi động chỉ gọi khi giá trị KHÁC chế độ hiện tại**, và đó không phải một phép tối ưu: `setMode(x)` với `x` đang là chế độ hiện tại đi vào nhánh *"bấm ⌘1 khi đang ở chính chế độ đó"* và gọi `enterFocus` — mà lúc đó chưa `mount()`, nên chưa chế độ nào khai điểm vào focus, và mỗi lần khởi động sẽ ghi một `console.error` vô nghĩa. Tương tự, `watch(currentMode)` đăng ký **sau** lượt đặt chế độ ban đầu, nếu không mỗi lần khởi động lại ghi đè đúng giá trị vừa đọc lên.

12. **Mặc định tồn tại ở HAI tầng, có chủ ý.** Rust trả `light`/`library` khi kho rỗng; frontend có `?? 'light'` cho trường hợp **không có cầu IPC**. ⛔ Không bỏ tầng Rust và trả chuỗi rỗng: `cfg?.theme ?? 'light'` chỉ bắt `null`/`undefined`, còn `''` là một giá trị và nó đi thẳng vào `applyTheme('')`. Ngược lại, ⛔ **không** thêm chốt hợp lệ cho `mode` ở Rust — `modeState.ts:39` đã có, và hai danh sách chép tay ở hai tầng là hai danh sách sẽ trôi khỏi nhau.

13. **⚠️ Dải báo lỗi chỉ hiện khi Rust TRẢ LỜI.** Một phiên `npm run dev` không có cầu IPC cho `configError = null` **có chủ ý** — dựng một `IpcError` giả ở đó làm mọi lần chạy dev mọc một dải *"Không mở được kho dữ liệu"*, một câu sai, và một câu sẽ dạy người đọc bỏ qua đúng dải này. Hệ quả ⛔ ghi ra thay vì đánh dấu đạt: **đường hiển thị chưa từng chạy trong một webview thật** — nghiệm thu cần một `$APPDATA` chỉ-đọc, tức một bảng nghiệm thu tay. Giao **Story 1.15**.

14. **`pragmas.rs:249` — ⛔ đã KHÔNG phơi chẩn đoán checkpoint ra UI.** Story cảnh báo đúng: lỗi checkpoint/sao lưu lúc chạy đều gắn nhãn `OpenFailed`, nên nối chúng lên giao diện sẽ làm một checkpoint lỗi sau nhiều giờ hiện nhầm *"Không mở được kho dữ liệu"*. Bề mặt lỗi của story này **chỉ** nhận `IpcError` từ `bootstrap_config`/`put_config`; `Store::diagnostics()` ⛔ không được nối vào đâu cả.

15. **`writer.rs:159` — đã review tay theo đúng điều kiện Ice đặt 2026-08-04.** Story này ghi qua tầng `store::Writer`, nên nó kích hoạt điều kiện *"review tay mỗi khi một story mới ghi qua tầng này"*. Đã soát: `save_value` chạy **đúng một** `tx.execute` với SQL hằng, ⛔ không I/O, ⛔ không gọi ra ngoài, ⛔ không `Store::write` lồng nhau. Giả định *"job ghi không chặn"* vẫn đúng.

16. **`schema.rs:137` — quan sát, ⛔ không sửa.** Đây là **lượt di trú thật đầu tiên trên một `global.db` đã có dữ liệu**, tức lần đầu đường sao lưu `fs::copy` chạy thật trên máy người dùng. Mục *"không nguyên tử, không xác minh lại"* từ hôm nay không còn là lý thuyết: mọi người dùng đã chạy một bản `user_version = 1` sẽ đi qua nó đúng một lần. Ngoài phạm vi story; đã ghi vào `deferred-work.md`.

17. **✅ Miễn trừ `tests/**` khỏi phép quét ranh giới (`deferred-work.md:179`) KHÔNG phải mở lại.** Hai tệp test mới ⛔ không gõ tên crate SQLite: `Store::write` nhận closure lấy `&Transaction` — kiểu **tái xuất** — nên ca ghi thẳng một hàng vào `global.db` viết được mà không chạm `rusqlite`. Số tệp test chạm crate đó vẫn đúng bằng `store_contract.rs`.

18. **Sàn quần thể đặt DƯỚI số thật, và hai sàn đo hai cây KHÁC nhau.** `check-i18n.mjs` 18 → **21** *(thật 27)*; `store_boundary.rs` 18 → **20** *(thật 26)*; `scope_boundary.rs` mới đặt **20**. Story 1.7 §Completion Notes #10: *"sàn tồn tại để bắt một cây bị cắt mất, không phải để đếm tệp mới"* — đặt bằng số thật là tự tạo một cổng đỏ ở story sau. ⚠️ Hai quần thể là `src-tauri/src/**` (26) và `src-tauri/**` sau miễn trừ `tests/**` (27, gồm `build.rs`); chép số của tệp này sang tệp kia là đặt một cái sàn cho một cây khác. Đã ghi vào doc-comment của cả hai. **N16/N17 chứng minh cả hai sàn cắn.**

19. **⛔ Không thêm bước CI mới, ⛔ không thêm phụ thuộc nào, ⛔ không khai trait nào.** `ci.yml` job `check` đã chạy đủ tám lệnh và story này ⛔ không sinh loại kiểm mới. `Cargo.toml` và `package.json` không đổi một dòng. `ports/mod.rs` giữ nguyên 5 dòng — AD-2 khoá số cổng ở ba, và AD-40 đã lập tiền lệ *"hai module Rust thường, không trait hoá"*.

20. **⚠️ Một lỗi cộng ở story file, ⛔ không sửa mã để khớp.** §Trạng thái repo ghi 41 test Rust; số thật tại `HEAD = 0ff36a0` là **40** *(`store_contract` có 16 ca, không phải 17 — và tổng của chính bốn số đã liệt cũng ra 40)*. Xem §Debug Log References.

### File List

**Mới — Rust (4)**
- `src-tauri/src/core/scope/kinds.rs`
- `src-tauri/src/core/scope/resolve.rs`
- `src-tauri/src/core/scope/store.rs`
- `src-tauri/src/commands/config.rs`

**Mới — test Rust (2)**
- `src-tauri/tests/scope_contract.rs`
- `src-tauri/tests/scope_boundary.rs`

**Mới — frontend (1)**
- `src/config/bootstrap.ts`

**Sửa — Rust (5)**
- `src-tauri/src/core/scope/mod.rs` — giữ nguyên 5 dòng doc-comment cũ, viết tiếp bên dưới: `ScopeResolver` · `Tier` · `ScopeError` · `WorkScope`
- `src-tauri/src/core/store/schema.rs` — `CONFIG_VALUE_DDL`, bước di trú 2 vào `GLOBAL_MIGRATIONS`
- `src-tauri/src/core/store/mod.rs` — tái xuất `CONFIG_VALUE_DDL`
- `src-tauri/src/commands/mod.rs` — khai `pub mod config;` + khuôn *"hàm thuần trước, `#[tauri::command]` là vỏ"*
- `src-tauri/src/lib.rs` — `invoke_handler` đầu tiên của dự án; cập nhật doc-comment của `open_global_store`

**Sửa — test Rust (3)**
- `src-tauri/tests/store_contract.rs` — **đúng một** ca (`a_fresh_database_migrates_up_to_target_and_logs_it`) + doc-comment `:714` đã thành sai; ⛔ `TWO_STEP`/`BROKEN_STEP_TWO` giữ nguyên
- `src-tauri/tests/ipc_contract.rs` — chữa mệnh đề vòng `ipc_error_wire_shape`
- `src-tauri/tests/store_boundary.rs` — `RS_FLOOR` 18 → 20

**Sửa — frontend (3)**
- `src/main.ts` — `boot()` async; nạp cấu hình trước `applyTheme`; tiêm `bindings`; `watch(currentMode)` → `put_config`
- `src/commands/index.ts` — `CommandDeps.bindings`; `registerAll` / `bindingsAreUsable` / `chordsFor`; ⛔ không `import` mới
- `src/App.vue` — dải báo lỗi không chặn qua `tError`

**Sửa — cổng (1)**
- `scripts/check-i18n.mjs` — `RS_FLOOR` 18 → 21

**Sửa — tài liệu (3)**
- `_bmad-output/planning-artifacts/architecture/architecture-AuraTranslate-2026-08-02/ARCHITECTURE-SPINE.md` — **bảng AD-18: ba hàng `chỉ toàn cục` + làm rõ `Cấu hình AI` theo từng trường + lý do ⛔ loại `ngôn ngữ nguồn`** *(Ice ký 2026-08-04)*
- `_bmad-output/implementation-artifacts/deferred-work.md` — đóng `:49` · `:140` · `:177`; mở §*Deferred from: 1-8-…* (8 mục)
- `_bmad-output/implementation-artifacts/sprint-status.yaml` — `1-8-…: ready-for-dev → in-progress → review`

⛔ **Không đụng:** `Cargo.toml` · `package.json` · `capabilities/main.json` · `ports/mod.rs` · `src/layout/` · `.github/workflows/ci.yml` · `src/i18n/vi.json`.

## Change Log

| Ngày | Thay đổi |
|---|---|
| 2026-08-04 | Story 1.8 hoàn tất. `core::scope` với `scope_kinds!` (9 loại · 3 ngữ nghĩa) và ba hàm phân giải thuần; bước di trú 2 của `global.db` (`config_value`); **hai `#[tauri::command]` và `invoke_handler` đầu tiên của dự án**; frontend nạp theme + chế độ + hợp âm từ đĩa và lưu chế độ cuối; dải báo lỗi kho không chặn. 22 test Rust mới (40 → 62), 17 đối chứng âm đều đỏ, tám cổng exit 0. |
| 2026-08-04 | **Sửa tầng kiến trúc — Ice ký:** bảng AD-18 nhận ngữ nghĩa thứ ba `chỉ toàn cục` cùng ba hàng (Phím tắt · Preset bố cục · Lựa chọn ứng dụng); `Cấu hình AI` làm rõ là ghi đè **theo từng trường**; ghi tường minh vì sao `ngôn ngữ nguồn` ⛔ **không** thuộc bảng. |
| 2026-08-04 | Đóng ba mục `deferred-work.md`: `:49` *(mệnh đề vòng `ipc_error_wire_shape`)* · `:140` *(chế độ khởi động không ai canh)* · `:177` *(lỗi mở kho chỉ ra `stderr`)*. |
