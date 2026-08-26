---
title: 'Cụm E — ba lỗ hổng canh gác: cắt vệ đi rồi bộ test vẫn xanh trọn'
type: 'bugfix'
created: '2026-08-26'
status: 'done'
review_loop_iteration: 0
baseline_commit: '3be0f5f85cafb67fe98febdcc589467938ec816f'
context:
  - '{project-root}/_bmad-output/implementation-artifacts/epic-3-context.md'
  - '{project-root}/AGENTS.md'
  - '{project-root}/src/AGENTS.md'
  - '{project-root}/src-tauri/AGENTS.md'
  - '{project-root}/tests/AGENTS.md'
---

<frozen-after-approval reason="human-owned intent — do not modify unless human renegotiates">

## Intent

**Problem:** Cụm E của vòng rà Epic 3 là cụm DUY NHẤT trong 55 phát hiện mà bằng chứng là một lượt **chạy**, không một lượt đọc: ba vệ sản phẩm bị cắt bỏ hẳn mà bộ test vẫn xanh trọn. ① `zh_nested_padding` (`scan.rs`) lọc rác đuôi bằng `matches_child(drop_last) || matches_child(drop_first)` — cắt vế `drop_first` đi thì 137 ca Rust của bốn bộ Glossary vẫn xanh, vì cả hai ca hiện có đều dựng qua `drop_last` và một ca **cố ý né** `drop_first`; hệ quả là ứng viên rác **neo-ĐẦU** đi thẳng vào bảng chờ mà UI 3.2/3.8 cho phép duyệt vào Glossary. ② `hasIpcBridge()` (`src/config/glossary.ts`) gác nhánh `catch` của cả **15** adapter — ép nó trả `false` thì 238 ca / 16 tệp vitest vẫn xanh nguyên, và `__TAURI_INTERNALS__` xuất hiện **0 lần** trong 16 tệp `glossary*.test.ts`; nếu vệ này bị lật, MỌI lỗi IPC thật bị nuốt thành `error: null`, đúng lớp *"rỗng im lặng"* mà dự án tự ghi là bug trung tâm. ③ Đường đọc `glossary_scan_threshold` (`GlobalConfig` → `bootstrap_config`) không có ca đi-về nào: `ipc_contract.rs:177` chốt cứng giá trị mong đợi là **5**, trùng khít `DEFAULT_GLOSSARY_SCAN_THRESHOLD`, nên kể cả khi đường đọc gãy hoàn toàn (luôn trả mặc định) ca đó vẫn xanh.

**Approach:** Không đụng mã sản phẩm — ba mục này là lỗ hổng **nghiệm thu**, không phải khuyết tật hành vi. Mỗi mục nhận đúng phép kiểm mà chính nó còn thiếu, dựng theo khuôn đã có sẵn ngay cạnh: ① một ca `Zh` mà chuỗi dài bị loại **CHỈ** qua nhánh `drop_first` (drop_last lệch tần suất có chủ ý), cộng đối chứng chiều ngược; ② một tệp vitest mới mock `@tauri-apps/api/core` ở đúng biên IPC (khuôn `glossaryConfigGuards.test.ts`) và lái `window.__TAURI_INTERNALS__` cả hai chiều (khuôn `bootstrap.test.ts`) cho **cả 15** adapter; ③ một ca đi-về ghi→mở lại trong `scope_contract.rs`, chép khuôn `the_last_mode_survives_a_write_and_a_reopen` của khoá anh em `mode`.

## Boundaries & Constraints

**Always:**
- 🔴 **Mỗi mục kèm phép đối chứng GỠ-CHỖ-NỐI: cắt vệ sản phẩm ra thì ca mới phải ĐỎ**, và ghi lại **tên ca** cùng **số ca đỏ thật** vào §Completion Notes. Bộ test cũ xanh **không đủ** — đó chính là điều đã sinh ra cụm này.
- 🔴 **Mỗi mục kèm đối chứng CHIỀU NGƯỢC: vệ không được nói oan.** Một ca chứng minh chuỗi/giá trị HỢP LỆ vẫn đi qua.
- 🔴 **Cả 15 adapter của `src/config/glossary.ts` đều phải có mặt ở mục ②**, cả hai chiều của `hasIpcBridge()`. Vá một chỗ nối rồi tuyên bố bảng đã đóng là đúng lỗi mà `AGENTS.md` §Known pitfalls gọi tên.
- 🔴 **Mock ở biên `@tauri-apps/api/core`, KHÔNG mock trọn module `src/config/glossary`** — mock trọn module là một ca không chạy vệ nào.
- Ca Rust mới ở `*_contract.rs`, tên hàm là một **câu khẳng định**. Ca `scope_contract.rs` theo bốn luật đầu tệp: thư mục tạm riêng, `drop(store)` trước khi xoá, không `sleep`, không treo khi trượt.
- Chuỗi hiển thị: không có. Không khoá `vi.json` mới, không token mới.

**Ask First:**
- 🔴 **Một ca mới ĐỎ ngay lượt chạy đầu ⇒ DỪNG và trình lỗi.** Cả ba mục kỳ vọng mã sản phẩm đang ĐÚNG; một ca đỏ nghĩa là vòng rà vừa tìm ra một khuyết tật hành vi thật, và đó là một quyết định về phạm vi cho Ice, không phải một dòng vá tiện tay.
- Nếu ca ① đòi sửa `zh_nested_padding` để dựng được một ca chỉ-`drop_first` (ví dụ hàm không phơi được đường đó qua `scan_candidates`): trình chỗ kẹt kèm fixture đã thử, đừng đổi chữ ký hàm sản phẩm.
- Nếu vế "đường đọc ngưỡng tới `scan_candidates_controlled` ở `commands/project.rs:574`" không nghiệm thu được ở tầng này (nó chạy trong một task sinh ra từ `AppHandle`): ghi một món nợ **có chủ**, đừng chấm đạt bằng suy luận.

**Never:**
- Không đổi mã sản phẩm ở `src-tauri/src/**` hay `src/**` — trừ khi §Ask First kích hoạt và Ice chốt.
- Không đụng cụm F, không đụng mục nợ **C4**, không đụng hai mục đã bị đo bác của cụm D.
- Không thêm phụ thuộc npm hay crate nào (NFR15). Không `tempfile` trong test Rust.
- Không `vi.useFakeTimers()`, không thêm `?.` vào mã sản phẩm cho một ca hết đỏ, không `eslint-disable`, không nới một cổng.
- Không dựng phép kiểm thứ hai cho một mệnh đề đã có chủ ở đường khác — `parse_glossary_scan_threshold` đã có Hàng 10 của `glossary_scan_contract.rs`; mục ③ canh **đường đọc**, không canh phép phân giải.

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected Output / Behavior | Error Handling |
|----------|--------------|---------------------------|----------------|
| **①a** rác neo-ĐẦU, chỉ khớp `drop_first` | `Zh`: N câu chứa `X+T` (T dài 2, X một ký tự đệm) ⇒ `freq(X+T) == freq(T)`; cộng k câu chứa `X+<khác>` ⇒ `freq(X+đầu2) != freq(X+T)` | `X+T` **vắng mặt** trong kết quả; `T` có mặt với đúng tần suất của nó | N/A |
| **①b** đối chứng ngược | cùng fixture, chuỗi lệch tần suất cả hai chiều con | chuỗi dài **được giữ** — vệ không loại oan | N/A |
| **②a** adapter trượt bằng một lỗi KHÔNG phải `IpcError`, **có** cầu Tauri | `window.__TAURI_INTERNALS__` có mặt; `invoke` reject `new Error(...)` | `error` là `UNKNOWN_IPC_ERROR` (hoặc `outcome: 'error'`) — lỗi thật **không bị nuốt** | chẩn đoán `console.error` nêu đích danh tên command |
| **②b** cùng lượt trượt, **không** cầu Tauri | `__TAURI_INTERNALS__` vắng mặt | `error: null` (hoặc `outcome: 'ipc_unavailable'`) — `npm run dev` ngoài Tauri vẫn chạy được | im lặng có chủ, `console.info` |
| **②c** quần thể | 15 adapter × 2 chiều | mỗi adapter có mặt ở **cả hai** chiều; danh sách đóng, thiếu một tên là thiếu một chỗ nối | N/A |
| **③a** ngưỡng chưa ai ghi | kho rỗng | `bootstrap_config(...).glossary_scan_threshold == DEFAULT_GLOSSARY_SCAN_THRESHOLD` (5) | N/A |
| **③b** ghi một giá trị KHÁC mặc định rồi mở lại kho | `put_config("app_config", "glossary_scan_threshold", "12")`, `drop(store)`, mở lại | trả **12** — giá trị nằm trên đĩa, không trong bộ nhớ lượt trước | lỗi ghi ⇒ ca đỏ, không nuốt |
| **③c** ghi đè lên chính khoá đó | ghi tiếp `"7"` | trả **7**, và đúng **một** hàng `config_value` cho cặp `(kind, key)` | N/A |
| **③d** giá trị hỏng trên đĩa, đi qua ĐƯỜNG ĐỌC | ghi `"abc"` rồi mở lại | rơi về **5**, phân biệt được với ③a nhờ hàng vẫn tồn tại | `eprintln!` chẩn đoán, KHÔNG ném |

</frozen-after-approval>

## Code Map

### Mục ① — `drop_first` không ai canh

- `src-tauri/src/core/glossary/scan.rs:299-333` **`zh_nested_padding`** — vệ cần canh. Chạy từ độ dài DÀI xuống (`ZH_NGRAM_LENGTHS = [2,3,4]`, `:39`), bỏ qua `n <= 2`. `drop_last = chars[..len-1]`, `drop_first = chars[1..]`, `matches_child` khớp khi tần suất **bằng đúng** nhau. Chỉ ghi vào tập `dropped`, không đụng `freq`.
- `src-tauri/src/core/glossary/scan.rs:236-288` `count_zh_candidates` — sinh n-gram theo TỪNG segment (không bắc cầu), lọc `char::is_alphanumeric` trước khi đếm. Fixture phải là câu Hán liền mạch để n-gram sinh ra được.
- `src-tauri/src/core/glossary/scan.rs:206-233` `effective_threshold` — `Zh` + dài 2–3 ký tự + ký tự ĐẦU nằm trong `COMMON_SURNAMES` ⇒ ngưỡng `threshold - 1`. Ảnh hưởng tới việc chọn ký tự đệm; đừng để nó làm lệch phép so.
- `src-tauri/tests/glossary_scan_contract.rs:50-117` **Hàng 2, hai ca hiện có** — `a_nested_ngram_with_equal_frequency_to_its_substring_is_dropped_as_padding` (đi qua `drop_last`) và `a_nested_ngram_with_a_different_frequency_from_its_substring_is_kept_alongside_it`. Ca thứ hai mang một chú thích ⚠️ nói rõ nó **thêm câu riêng cho `炎的` để KHÔNG tình cờ khớp qua `drop_first`** — tức chiều `drop_first` bị né có chủ ý ngay tại chỗ. Ca mới đứng ngay dưới hai ca này, cùng Hàng 2.
- Khuôn fixture đã dùng trong tệp: `nothing_known` (mọi chuỗi đều `Missing`), `COMMON_SURNAMES`, `scan_candidates(&refs, MatchLang::Zh, 5, COMMON_SURNAMES, &mut is_known)`.

### Mục ② — `hasIpcBridge()` gác 15 nhánh `catch`, 0 ca chạm

- `src/config/glossary.ts:128-130` **`hasIpcBridge()`** — `typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window`.
- **15 adapter, danh sách ĐÓNG** (mỗi hàm một nhánh `catch` gọi `hasIpcBridge()`), kèm dòng gọi: `lookupGlossaryTerm` `:160` · `addGlossaryTerm` `:205` · `updateGlossaryTerm` `:233` · `glossaryMarksForChapter` `:399` · `pendingGlossaryCandidates` `:494` · `confirmPendingGlossaryTranslation` `:533` · `approveGlossaryCandidate` `:581` · `rejectGlossaryCandidate` `:619` · `listGlossaryEntries` `:721` · `deleteGlossaryTerm` `:744` · `exportGlossaryTier` `:812` · `openGlossaryImportPreview` `:918` · `confirmGlossaryImport` `:979` · `cancelGlossaryImport` `:1000` · `promoteGlossaryTermToGlobal` `:1017`.
- **BA hình dạng trả về khi không có cầu** — bảng ca phải đọc đúng trường: `{ found: 'unknown', error: null }` (`lookupGlossaryTerm`) · `{ outcome: 'ipc_unavailable' }` (`exportGlossaryTier`, `openGlossaryImportPreview`) · `{ …: null, error: null }` (12 hàm còn lại; tên trường giá trị lần lượt là `value`/`marks`/`candidates`/`entries`/`summary`).
- `src/config/glossary.ts:122-127` `UNKNOWN_IPC_ERROR` — `{ code: 'ipc.unknown', message_key: 'err.unknown', params: {}, retryable: false }`, là thứ chiều "có cầu" phải trả.
- `tests/frontend/glossaryConfigGuards.test.ts:16-24` **khuôn mock đúng biên** — `vi.mock('@tauri-apps/api/core', …)` + `freshAdapter()` (`vi.resetModules()` + `mockInvoke.mockReset()`). Chép nguyên khuôn này.
- `tests/frontend/bootstrap.test.ts:39-42,115,133` **khuôn lái cầu Tauri** — `Reflect.deleteProperty(window, '__TAURI_INTERNALS__')` trong `beforeEach`, `Object.defineProperty(window, '__TAURI_INTERNALS__', { configurable: true, value: {} })` cho chiều "có cầu". Đây là **tệp DUY NHẤT** trong `tests/frontend/**` đang lái biến này, và nó canh `src/config/bootstrap.ts`, không canh `glossary.ts`.

### Mục ③ — đường đọc ngưỡng không có ca đi-về

- `src-tauri/src/core/scope/store.rs:224-230` **`GlobalConfig::glossary_scan_threshold`** — vệ cần canh: `self.app.get(KEY_GLOSSARY_SCAN_THRESHOLD)` rồi qua `parse_glossary_scan_threshold`. Cắt nó thành `DEFAULT_GLOSSARY_SCAN_THRESHOLD` trần là phép đối chứng GỠ.
- `src-tauri/src/core/scope/store.rs:116` `KEY_GLOSSARY_SCAN_THRESHOLD = "glossary_scan_threshold"` · `:122` `DEFAULT_GLOSSARY_SCAN_THRESHOLD = 5` · `:145-164` `parse_glossary_scan_threshold` (**đã có chủ**: Hàng 10 của `glossary_scan_contract.rs:527-566`).
- `src-tauri/src/commands/config.rs:103,146` — trường thứ **bảy** trên dây, `bootstrap_config` đổ giá trị qua.
- `src-tauri/tests/ipc_contract.rs:150-205` — ca **hình dạng serialize**, dựng struct literal với `glossary_scan_threshold: 5`. Nó KHÔNG chạm kho, nên nó không nói gì về đường đọc; đừng sửa nó, chỉ đừng đọc nó thành một bằng chứng nó không phải.
- `src-tauri/tests/scope_contract.rs:665-716` **`the_last_mode_survives_a_write_and_a_reopen`** — khuôn cần chép nguyên: `put_config` → `drop(store)` → mở lại → `bootstrap_config` → `save_value` ghi đè → đếm `COUNT(*) FROM config_value` phải bằng 1. Cùng tệp, ca mới đứng ngay cạnh.
- `src-tauri/tests/scope_contract.rs:1-45` — bốn luật đầu tệp và danh sách `use` sẵn có (`bootstrap_config`, `put_config`, `save_value`, `Store`, `Transaction`).

### Baseline đã đo trên `3be0f5f` (2026-08-26)

`glossary_scan_contract` 25 · `glossary_commands_contract` 29 · `glossary_boundary` 11 · `glossary_contract` 72 · `scope_contract` 23 · `ipc_contract` 5 — tất cả xanh. `npx vitest run tests/frontend/glossary` = **16 tệp / 238 ca** xanh (sổ nợ ghi 14/186; con số đó có trước cụm D — 🔵 sửa tại chỗ khi đóng mục). `grep -c __TAURI_INTERNALS__ tests/frontend/glossary*.test.ts` = **0 trên cả 16 tệp**.

## Tasks & Acceptance

**Execution:**
- [x] `src-tauri/tests/glossary_scan_contract.rs` -- thêm vào Hàng 2 một ca `Zh` mà chuỗi dài bị loại **CHỈ** qua `drop_first` (①a) và một ca đối chứng ngược (①b); doc-comment nói rõ vì sao `drop_last` phải lệch tần suất -- hai ca hiện có đều đi qua `drop_last`, một ca né `drop_first` có chủ ý.
- [x] `tests/frontend/glossaryIpcBridge.test.ts` -- tệp MỚI: bảng 15 adapter × 2 chiều `hasIpcBridge()`, mock ở biên `@tauri-apps/api/core`, lái `window.__TAURI_INTERNALS__` -- vệ này gác 15 nhánh `catch` mà 0 ca nào chạm; danh sách adapter viết ĐÓNG để thêm hàm thứ 16 là một lượt sửa thấy được.
- [x] `src-tauri/tests/scope_contract.rs` -- thêm ca đi-về `glossary_scan_threshold` (③a–③d), chép khuôn `the_last_mode_survives_a_write_and_a_reopen` -- `ipc_contract.rs:177` chốt cứng đúng giá trị mặc định nên đường đọc gãy vẫn xanh.
- [x] `_bmad-output/implementation-artifacts/deferred-work.md` -- nối `→ ✅ ĐÃ ĐÓNG 2026-08-26` vào mục `Cụm E` kèm tên ba ca mới và số ca ĐỎ thật của ba phép GỠ; 🔵 sửa tại chỗ hai con số 14/186 đã hết đúng -- không xoá mục, không làm tròn lên.

**Acceptance Criteria:**
- Given `zh_nested_padding` bị cắt vế `|| matches_child(&drop_first)`, when chạy `cargo test --test glossary_scan_contract`, then ca ①a **ĐỎ** và số ca đỏ được ghi lại; khôi phục vế đó thì cả bộ xanh trở lại.
- Given `hasIpcBridge()` bị ép trả `false`, when chạy `npx vitest run tests/frontend/glossaryIpcBridge`, then **15** ca chiều "có cầu" ĐỎ; ép trả `true` thì **15** ca chiều "không cầu" ĐỎ.
- Given `GlobalConfig::glossary_scan_threshold` bị cắt thành `DEFAULT_GLOSSARY_SCAN_THRESHOLD` trần, when chạy `cargo test --test scope_contract`, then ca ③ **ĐỎ** ở nhánh ③b/③c.
- Given không cắt gì cả, when chạy trọn `pre-push`, then **0 ca đỏ** và không tệp nào trong `src-tauri/src/**` hay `src/**` xuất hiện trong `git diff --name-only`.

## Verification

**Commands:**
- `npx vitest run tests/frontend/glossary` -- expected: 17 tệp, ≥ 268 ca, 0 đỏ (238 + 30 ca mới).
- `cd src-tauri && cargo test --locked --test glossary_scan_contract --test scope_contract` -- expected: 27 + 24 ca (hoặc hơn), 0 đỏ.
- `npm run check:lint && npm run check:i18n` -- expected: exit 0. `check:i18n` Kiểm A cấm chữ có dấu ở vị trí mã trong `src-tauri/src/**`; `tests/**` được miễn trừ có tên nên chuỗi assert giữ dấu được.
- `npm run build && cd src-tauri && cargo test --locked` -- expected: 0 đỏ trên toàn bộ. Chạy `build` TRƯỚC `cargo test`, thiếu `dist/` thì gãy ở khâu biên dịch.

**Manual checks:**
- Ba phép **GỠ CHỖ NỐI**, chạy TỪNG cái một và khôi phục ngay sau: (1) xoá `|| matches_child(&drop_first)`; (2) đổi thân `hasIpcBridge()` thành `return false` rồi `return true`; (3) đổi thân `glossary_scan_threshold()` thành `DEFAULT_GLOSSARY_SCAN_THRESHOLD`. Ghi **số ca đỏ thật** và **tên ca** cho từng lượt vào §Completion Notes — một con số suy ra không phải một số đo.
- Đếm quần thể mục ②: số hàng trong bảng ca phải bằng số dòng `if (hasIpcBridge())` trong `src/config/glossary.ts` (`grep -c`). Hai số lệch nhau là một adapter không ai canh.

## Dev Agent Record

### Completion Notes

**🔴 Không một dòng mã sản phẩm nào đổi.** `git status --short src/ src-tauri/src/` rỗng sau lượt vá. Cửa §Ask First (*"một ca mới ĐỎ ngay lượt chạy đầu ⇒ DỪNG"*) **không kích hoạt**: cả ba ca xanh ngay lượt đầu, tức mã sản phẩm đang đúng và cụm E đúng là lỗ hổng nghiệm thu như spec đã đặt giả thiết.

#### Ba phép đối chứng GỠ-CHỖ-NỐI — số ca đỏ THẬT, chạy từng cái một rồi khôi phục

| Vệ bị cắt | Ca ĐỎ | Bộ test còn lại |
|---|---|---|
| ① xoá `\|\| matches_child(&drop_first)` (`scan.rs`) | **1** — `a_head_anchored_ngram_matching_only_its_drop_first_child_is_dropped_as_padding` | `glossary_boundary` 11/11 · `glossary_commands_contract` 29/29 · `glossary_contract` 72/72 **xanh** |
| ② `hasIpcBridge()` ⇒ `return false` | **15** — trọn nhóm `②a` | 16 tệp glossary kia **xanh** (254/269) |
| ② `hasIpcBridge()` ⇒ `return true` | **15** — trọn nhóm `②b` | 16 tệp glossary kia **xanh** (254/269) |
| ③ thân getter ⇒ `DEFAULT_GLOSSARY_SCAN_THRESHOLD` trần | **1** — `the_glossary_scan_threshold_survives_a_write_and_a_reopen` | `ipc_contract` 5/5 · `glossary_scan_contract` 27/27 **xanh** |
| ②c *(thêm ở vòng rà)* — một adapter **thứ 16** vào `src/config/glossary.ts` | **1** — ca quần thể `②c` | 30/31 còn lại **xanh** |

Ba lượt khôi phục đều đối chứng bằng `git status`, không bằng mắt: `src-tauri/src/core/glossary/scan.rs`, `src-tauri/src/core/scope/store.rs`, `src/config/glossary.ts` đều về **0 dòng thay đổi**.

⇒ Mệnh đề của sổ nợ đứng vững nguyên văn trên `3be0f5f`: ba vệ này trước lượt vá **không có ai canh**, và nay mỗi vệ có đúng một chủ.

#### Quần thể sau lượt vá

`glossary_scan_contract` 25 → **27** · `scope_contract` 23 → **24** · bộ glossary frontend 238 → **269** (16 → 17 tệp). Cổng: **11/11 xanh**. `npm run test` **43 tệp / 561 ca**. `cargo test --locked` **708 ca**. `npm run build` sạch (`vue-tsc` kiểm kiểu cả cây test vì `tsconfig` `include` nó). 0 đỏ ở mọi đường.

#### Hai vế KHÔNG nghiệm thu được ở tầng này — đã ghi nợ có chủ, không chấm đạt bằng suy luận

1. **Nửa `commands/project.rs` của mục ③.** Ca đi-về canh `Store → GlobalConfig → bootstrap_config`, tức nửa đi ra **webview**. Nửa bơm ngưỡng vào `scan_candidates_controlled` của lượt quét khi nhập chạy trong một task sinh từ `AppHandle` và không gọi được từ `tests/**`. ⚠️ Đừng đọc ca ③ thành *"ngưỡng cấu hình đã tới được lượt quét"*. Giới hạn này ghi tại chỗ trong doc-comment của chính ca đó.
2. **Bộ vitest Glossary đỏ ngẫu nhiên dưới tải CPU.** Bắt tình cờ: chạy song song với `cargo` cho 8 rồi 7 ca đỏ, **tập ca đỏ đổi giữa hai lượt**; `--no-file-parallelism` cho 238/238 xanh. Đã đo với tệp mới **gỡ hẳn khỏi cây** — vẫn đỏ, tức chập chờn có TRƯỚC lượt vá này. Mọi ca đỏ dừng ở mốc ~5.000 ms (timeout) nhưng in ra câu assert của chính nó, nên một lượt đỏ như vậy đọc lên giống một khuyết tật sản phẩm.

#### Debug Log References

- Đối chứng GỠ mục ①: `cargo test --locked --test glossary_scan_contract --test glossary_commands_contract --test glossary_boundary --test glossary_contract`
- Đối chứng GỠ mục ②: `npx vitest run tests/frontend/glossary --no-file-parallelism`
- Đối chứng GỠ mục ③: `cargo test --locked --test scope_contract --test ipc_contract --test glossary_scan_contract`

### Review Findings — vòng rà bước 4 (ba lăng kính, 2026-08-26)

**Không finding nào là `intent_gap` hay `bad_spec`** ⇒ không lật vòng, `review_loop_iteration` giữ `0`. Lăng kính verification-gap kết luận **không còn lỗ hổng nghiệm thu**; hai lăng kính kia cho 13 mục, sau khử trùng và thẩm định còn **6 bản vá**.

**Đã vá (`patch`):**

1. 🔴 **`②c` khai một bảo đảm mà nó không cấp** *(edge-case + blind-hunter, cùng một claim)*. Bản đầu chốt cứng `toHaveLength(15)` và chỉ kiểm MỘT chiều (mỗi tên trong bảng là một hàm thật), nên một adapter **thứ 16** thêm vào `src/config/glossary.ts` mà quên thêm hàng vẫn cho ca XANH — trong khi doc-comment ngay trên hứa nó sẽ đỏ. Nay ca so **BẰNG** tập hàm export thật với tập tên trong bảng. **Đối chứng đã chạy:** thêm một `pretendSixteenthAdapter` vào tệp nguồn ⇒ **ca `②c` ĐỎ** (1 đỏ / 31); hình dạng cũ sẽ xanh trên chính thí nghiệm đó. Đây đúng lớp lỗi mà cả cụm E tồn tại để đóng, và nó suýt tái sản xuất bên trong bản vá của chính nó.
2. 🔴 **Fixture của ①a không được ghim số học** *(blind-hunter)*. Nếu một lượt sửa sau vô tình đưa `在萧` về 40 thì vế `drop_last` cũng khớp, `在萧炎` vẫn bị loại, và ca xanh **vì một lý do khác hẳn**. Nay ca khẳng định `在萧` = **47** tại chỗ, không chỉ viết ở chú thích.
3. **Cụm mô tả sai cơ chế** *(verification-gap)*. Tôi viết đường quét khi nhập *"chạy trong một task sinh từ `AppHandle`"*; đo lại: `spawn_import_scan` dùng `std::thread::Builder::new().spawn(...)` — một **luồng OS** chỉ BẮT `AppHandle` vào closure, không `tauri::async_runtime` nào. Nội dung món nợ đứng nguyên (closure vẫn cần một app nên `tests/**` không gọi tới được; seam `keep_committed_import_when_scan_spawn_fails` chỉ phơi ca *spawn trượt*); chỉ tên cơ chế bị gọi sai, và nó sẽ đẩy người đọc sau đi tìm nhầm chỗ. Sửa ở **cả hai** nơi tôi đã chép nó.
4. **Header khai phạm vi rộng hơn thật** *(blind-hunter)*. `hasIpcBridge()` chỉ được hỏi ở **nửa sau** mỗi khối `catch` — nhánh `if (isIpcError(err))` chạy trước và tệp này không chạm nó. Đã ghi rõ phạm vi và trỏ chủ của nhánh kia.
5. **Mệnh đề sai về ca hiện có** *(tự bắt khi thẩm định blind-hunter #10)*. Tôi viết hai ca Hàng 2 *"đều đi qua vế TRÁI"*. Đo lại: ca thứ nhất khớp **CẢ HAI** vế (`萧炎` và `炎的` đều 40) — nên nó xanh dù cắt vế nào, và *"đi qua vế trái"* làm người đọc tưởng vế phải đã bị loại trừ ở đó. Sửa tại chỗ. ⇒ Cũng bác luôn blind-hunter #10 (*"ca cả-hai-vế chưa ai phủ"*): nó **đã** được phủ, bởi chính ca thứ nhất.
6. **`②b` họ `outcome` không chặn một `error` lạc** *(blind-hunter)*; nay khẳng định trường đó vắng mặt.

**Bác (`reject`), kèm lý do — không bác im lặng:**

- *"Ca Rust nên khẳng định `out.len()`"* — fixture sinh hàng chục n-gram phụ (`第0天`, `他在`…); ghim trọn tập là một ca giòn, và hai ca Hàng 2 có sẵn cũng dùng `.find()`. Mệnh đề đang kiểm là mệnh đề hẹp, giữ nó hẹp.
- *"Export `UNKNOWN_IPC_ERROR` thay vì chép"* — nới bề mặt export của mã sản phẩm để phục vụ một bàn test, và §Never cấm chạm mã sản phẩm. Bản chép lệch thì 15 ca `②a` đỏ kèm một diff `toEqual` đọc được — tín hiệu đã có.
- *"Dựng helper dùng chung cho `__TAURI_INTERNALS__`"* — hai dòng ở hai tệp; gom lại là một lượt refactor ngoài phạm vi.
- *"Nên thêm một ca hẹp quanh `commands/project.rs:574`"* — đã ghi nợ **có chủ**; làm được nó đòi tách một hàm thuần khỏi closure, tức sửa mã sản phẩm.
- *"`family` giả định trường `error` giữ tên"* — phòng xa cho một hình dạng chưa tồn tại.
- *"Diff đưa cho lăng kính thiếu sổ nợ và spec"* — lựa chọn đóng gói của tôi (chúng là tài liệu, không phải mã chịu rà), không phải khuyết tật của bản vá.

## Suggested Review Order

**Điểm vào — vệ nào bị lật, và tại sao 137 ca không thấy**

- Bảng ba phép GỠ kèm số ca đỏ thật; đọc đây trước, mọi thứ khác là bằng chứng.
  [`spec…cum-e.md` §Completion Notes](./spec-epic-3-review-cum-e-le-hong-canh-gac.md)

- Mục nợ gốc, nay đã đóng, kèm hai con số cũ được sửa tại chỗ.
  [`deferred-work.md:7290`](./deferred-work.md#L7290)

**① `drop_first` — nhánh chưa từng ai canh**

- Fixture cô lập đúng vế phải: `萧炎` bằng 40, `在萧` lệch thành 47.
  [`glossary_scan_contract.rs:158`](../../src-tauri/tests/glossary_scan_contract.rs#L158)

- Ghim số học trước khi kết luận — chống ca xanh vì một lý do khác.
  [`glossary_scan_contract.rs:182`](../../src-tauri/tests/glossary_scan_contract.rs#L182)

- Đối chứng ngược: cả hai chuỗi con lệch ⇒ chuỗi dài phải được GIỮ.
  [`glossary_scan_contract.rs:209`](../../src-tauri/tests/glossary_scan_contract.rs#L209)

**② `hasIpcBridge()` — vệ gác nửa sau của 15 khối `catch`**

- Ca quần thể: so BẰNG với tập hàm export, không chốt cứng số 15.
  [`glossaryIpcBridge.test.ts:273`](../../tests/frontend/glossaryIpcBridge.test.ts#L273)

- Bảng 15 adapter viết đóng; ba hình dạng trả về gộp thành hai họ.
  [`glossaryIpcBridge.test.ts:80`](../../tests/frontend/glossaryIpcBridge.test.ts#L80)

- Chiều "có cầu": lỗi thật không bị nuốt, chẩn đoán nêu đích danh command.
  [`glossaryIpcBridge.test.ts:217`](../../tests/frontend/glossaryIpcBridge.test.ts#L217)

- Chiều "không cầu": im lặng có chủ, và không một `error` lạc.
  [`glossaryIpcBridge.test.ts:234`](../../tests/frontend/glossaryIpcBridge.test.ts#L234)

**③ Đường đọc ngưỡng — `ipc_contract` không canh được vì nó chốt đúng giá trị mặc định**

- Ghi `12` rồi mở lại kho; giá trị KHÁC mặc định là điều kiện duy nhất.
  [`scope_contract.rs:758`](../../src-tauri/tests/scope_contract.rs#L758)

- Phạm vi ghi thẳng: nửa `commands/project.rs` vẫn hở, đã có chủ.
  [`scope_contract.rs:741`](../../src-tauri/tests/scope_contract.rs#L741)

**Ngoại vi — hai món nợ mới, cả hai có chủ**

- Nửa đường đọc ngưỡng không gọi được từ `tests/**` nếu không dựng app.
  [`deferred-work.md:7346`](./deferred-work.md#L7346)

- Bộ vitest Glossary đỏ ngẫu nhiên dưới tải CPU — có TRƯỚC lượt vá này.
  [`deferred-work.md:7371`](./deferred-work.md#L7371)
