# Story 1.5: Tài nguyên chuỗi giao diện và hình dạng lỗi qua IPC

Status: ready-for-dev

**Covers:** NFR16 · AD-21
**Epic:** 1 — Nền móng ứng dụng & Tra cứu ngoại tuyến tức thì
**Đóng mục Deferred:** `deferred-work.md:19` — *"NFR16 không có cơ chế cưỡng chế nào"*

<!-- Note: Validation is optional. Run validate-create-story for quality check before dev-story. -->

---

## Story

As a **người dựng**,
I want **không một chuỗi hiển thị nào nằm trong mã nguồn, kể cả chuỗi lỗi**,
so that **thêm một ngôn ngữ về sau không phải rà lại toàn bộ codebase**.

> **Vì sao story này đứng ở Epic 1 chứ không ở Epic 10.** NFR16 là một trong **hai yêu cầu cắt ngang áp từ Giai đoạn 1** (cùng NFR17). PRD nói thẳng lý do: *"tách chuỗi ra file riêng gần như không tốn gì nếu làm từ đầu và rất đắt nếu làm sau"*. Story 1.14 sẽ dựng bốn panel; Epic 3–9 thêm hàng trăm chuỗi. Cổng phải đứng **trước** chúng, không phải sau.

---

## Acceptance Criteria

### AC1 — Mọi chuỗi hiển thị phân giải từ `src/i18n/vi.json` theo khoá chấm

**Given** `src/i18n/vi.json`
**When** ứng dụng chạy
**Then** mọi chuỗi hiển thị phân giải từ file đó theo khoá chấm

> **Hình dạng file đã chốt, không phải lựa chọn của dev:** `vi.json` là **một object PHẲNG**, khoá là chuỗi có dấu chấm — `"lookup.empty_result": "…"`. **Không lồng object.** Nguồn: `ARCHITECTURE-SPINE.md#Consistency Conventions:540` — *"tài nguyên chuỗi `vi.json` **phẳng** theo khoá chấm (`lookup.empty_result`)"*.

### AC2 — Grep chuỗi tiếng Việt trong `.rs` và `.vue` không ra kết quả

**Given** mã `.rs` và mã `.vue`
**When** grep chuỗi tiếng Việt
**Then** không tìm thấy kết quả nào

> **Nghiệm thu bằng một lệnh có mã thoát, không bằng một lượt grep tay.** Đây chính là mục `deferred-work.md:19`: quy tắc này vi phạm chỉ cần gõ một nhãn button, mà thứ khó vi phạm hơn hẳn (lỡ cài `tauri-plugin-fs`) thì đã có cả script lẫn mã thoát. Phạm vi quét và các miễn trừ phải **viết ra tường minh** — xem §Ranh giới quét.

### AC3 — Lỗi qua IPC mang hình dạng bốn trường, không mang văn bản hiển thị

**Given** một lỗi phát sinh ở Rust
**When** lỗi đi qua ranh giới IPC
**Then** nó mang hình dạng `{ code, message_key, params, retryable }`
**And** không mang văn bản hiển thị

> **Bốn tên trường là hợp đồng nguyên văn**, viết đúng `snake_case` như trên dây. ⛔ `#[serde(rename_all = "camelCase")]` trên struct này là **phá hợp đồng** — nó biến `message_key` thành `messageKey` và mọi frontend đọc theo AD-21 sẽ nhận `undefined`. Nghiệm thu bằng một test so **đúng bốn khoá, đúng chính tả**.

### AC4 — `message_key` thiếu thì hiện khoá nguyên văn và ghi cảnh báo, không sập

**Given** một `message_key` không có trong `vi.json`
**When** frontend phân giải
**Then** hiển thị khoá đó nguyên văn và ghi cảnh báo, không sập

### AC5 — Chuỗi trạng thái theo năm quy tắc giọng văn UX-DR47

**Given** quy tắc giọng văn ở UX-DR47
**When** soạn chuỗi trạng thái
**Then** câu viết ở dạng vô nhân xưng, không xưng *"chúng tôi"*, không gọi người dùng là *"bạn"*
**And** thông báo lỗi nêu nguyên nhân thay vì đổ lỗi người dùng

> Hai vế đầu **máy kiểm được** (quét giá trị trong `vi.json`). Ba quy tắc còn lại của UX-DR47 — *nói việc không nói cảm xúc · nêu hệ quả · số liệu là số liệu* — là quyết định biên tập, nghiệm thu bằng mắt và ghi vào Completion Notes.

---

## Tasks / Subtasks

- [ ] **Task 1 — Chụp ảnh vi phạm hiện có TRƯỚC khi viết một dòng mã** (AC: 2)
  - [ ] Chạy phép đếm dưới đây trên `HEAD` và chép kết quả vào §Debug Log References. Đây là đường cơ sở; không có nó thì không chứng minh được cổng ở Task 6 thật sự bắt được gì
  - [ ] Số đã đo lúc dựng story (`HEAD = 0255163`) ở §Trạng thái repo hiện tại — đối chiếu, lệch thì dừng và đọc lại §🔴 Phát hiện chặn
  - [ ] ⛔ **Đừng "dọn" gì ở task này.** Task 1 chỉ đo

- [ ] **Task 2 — `src/i18n/resolve.ts`: hàm phân giải thuần, không import gì** (AC: 1, 4)
  - [ ] Tạo `src/i18n/resolve.ts` chứa `createResolver(catalog: Record<string, string>)` trả về `t(key, params?)`
  - [ ] ⛔ **Tệp này KHÔNG được `import` bất cứ thứ gì** — không Vue, không JSON, không `@tauri-apps/api`. Lý do ở §Vì sao `resolve.ts` phải thuần: nó là tệp duy nhất `check-i18n.mjs` nạp được để kiểm **hành vi** AC4, và Node chỉ bóc kiểu được cho TS "erasable-only"
  - [ ] ⛔ Không `enum`, không `namespace`, không parameter property — ba thứ Node từ chối bóc kiểu. Dùng union type chuỗi nếu cần
  - [ ] Hành vi khoá thiếu (AC4): trả **đúng khoá nguyên văn**, `console.warn` **một lần cho mỗi khoá** (dedupe bằng `Set`), **không ném**
  - [ ] ⚠️ Dedupe không phải tối ưu vặt: một khoá thiếu trong template Vue chạy lại mỗi lần render — không dedupe thì console ngập và cảnh báo thật chìm mất
  - [ ] Nội suy tham số: cú pháp `{ten_tham_so}`, tên khớp `[a-z_][a-z0-9_]*`. Tham số thiếu ⇒ **giữ nguyên placeholder** + `console.warn`, không ném, không thay bằng `undefined`
  - [ ] Tạo `src/i18n/index.ts`: `import catalog from './vi.json'` → `export const t = createResolver(catalog)`. `resolveJsonModule` đã bật sẵn ở `tsconfig.json:12`
  - [ ] Thêm `tError(err: IpcError): string` — nhận nguyên payload lỗi của AC3, trả chuỗi đã phân giải. Kiểu `IpcError` khai ở `src/i18n/index.ts` và **khớp từng chữ** bốn trường của AC3

- [ ] **Task 3 — `vi.json`: hình dạng phẳng và bộ khoá mồi tối thiểu** (AC: 1, 5)
  - [ ] `src/i18n/vi.json` hiện là `{}` (3 byte). Viết lại thành object **phẳng**, khoá chấm
  - [ ] ⛔ **KHÔNG lồng object.** `{"lookup": {"empty_result": "…"}}` là sai hình dạng — xem AC1
  - [ ] ⛔ **KHÔNG dựng sẵn một từ vựng khoá cho tính năng chưa tồn tại.** Story này sở hữu **cơ chế**, không sở hữu **từ vựng**. Mỗi story sau tự thêm khoá của nó. Một `vi.json` 200 khoá cho panel chưa ai dựng là 200 chuỗi không ai kiểm được, và chúng sẽ sai
  - [ ] Bộ mồi tối thiểu — đủ để chứng minh cả bốn AC, không hơn:
    - `err.unknown` — khoá dự phòng cuối cùng của AD-21. Mọi lỗi Rust chưa phân loại được rơi vào đây thay vì rơi vào một chuỗi viết tay
    - `err.io.read_failed` với tham số `{path}` — chứng minh đường nội suy tham số chạy thật
  - [ ] Soạn hai chuỗi theo UX-DR47: vô nhân xưng, nêu nguyên nhân, không đổ lỗi. Ví dụ hình dạng *(dev soạn bản cuối, đây không phải bản chép)*: `"Không đọc được tệp tại {path}."` — **không** `"Bạn đã chọn một tệp không đọc được."`
  - [ ] Ghi bản cuối của cả hai chuỗi + lý do chọn chữ vào §Completion Notes (nghiệm thu ba quy tắc UX-DR47 mà máy không chấm được)

- [ ] **Task 4 — Rust: danh mục `MessageKey` và kiểu `IpcError`** (AC: 3)
  - [ ] `src-tauri/src/core/i18n/mod.rs` hiện **chỉ có doc-comment** và nói thẳng: *"Hình dạng thật của danh mục là quyết định của **Story 1.5**"*. Đây là chỗ trả lời
  - [ ] Dùng `macro_rules! message_keys!` khai **một chỗ duy nhất** sinh ra cả `enum MessageKey`, `MessageKey::ALL` và `as_str()` — xem khung ở §Danh mục `MessageKey`. Lý do: `ALL` và `as_str()` viết tay sẽ trôi khỏi nhau, và test đồng bộ với `vi.json` chạy trên `ALL` nên `ALL` thiếu một biến thể là test xanh giả
  - [ ] `Serialize` cho `MessageKey` = `serialize_str(self.as_str())`. ⛔ Không `#[derive(Serialize)]` trần trên enum — mặc định của serde cho unit variant là **tên biến thể** (`IoReadFailed`), không phải khoá chấm
  - [ ] Khai `IpcError` với **đúng bốn trường, đúng chính tả**: `code` · `message_key` · `params` · `retryable`
  - [ ] `params: BTreeMap<String, String>` — **BTree, không Hash**: thứ tự khoá ổn định thì test so JSON mới ổn định. **Giá trị là `String`**, kể cả số: định dạng số và ngày giờ chỉ ở frontend (`ARCHITECTURE-SPINE.md#Consistency Conventions` — *"Ngày giờ: lưu ISO-8601 UTC; định dạng hiển thị chỉ ở frontend"*, cùng nguyên tắc)
  - [ ] ⛔ **`params` cũng không được mang văn bản hiển thị.** Một `params: {"reason": "Nhà cung cấp không phản hồi"}` là AD-21 bị thủng qua cửa sau. Tham số mang **dữ liệu** (đường dẫn, số đếm, tên nhà cung cấp), không mang **câu**
  - [ ] `code`: định danh máy đọc, ổn định qua mọi lần sửa lời văn. ⚠️ `code` và `message_key` **được phép 1:1 hôm nay** nhưng **là hai trường, không phải một trường hai tên**: frontend rẽ nhánh trên `code`, hiển thị `message_key`. ⛔ `code` không bao giờ được đưa ra màn hình
  - [ ] `retryable: bool` — chỉ là **quyền hiển thị một nút thử lại**. ⛔ Không mã nào được tự thử lại khi thấy `true`: AD-22 cấm auto-retry, và với BYOK nó là tính tiền hai lần
  - [ ] Ba test trong `src-tauri/tests/` (xem §Testing standards để biết đặt ở đâu):
    - `ipc_error_wire_shape` — `serde_json::to_value(IpcError…)` có **đúng bốn khoá**, đúng chính tả `message_key` (không `messageKey`), `message_key` serialize thành **chuỗi khoá chấm**
    - `every_message_key_exists_in_vi_json` — đọc `../src/i18n/vi.json` (đường dẫn qua `CARGO_MANIFEST_DIR`, cùng khuôn `config_invariants.rs:11-19`), khẳng định **mọi** `MessageKey::ALL` có mặt. Chiều ngược lại **không** kiểm: `vi.json` có nhiều khoá chỉ frontend dùng, đó là bình thường
    - `vi_json_is_flat` — mọi giá trị là chuỗi, không object lồng *(hoặc để `check-i18n.mjs` gánh — chọn một, đừng làm cả hai)*

- [ ] **Task 5 — Dời hai chuỗi chẩn đoán ra khỏi `App.vue`** (AC: 2)
  - [ ] `src/App.vue:38` và `:54` chứa **văn bản tiếng Việt trong template literal** — cổng Task 6 sẽ đỏ ở chính hai dòng này. Đây là phát hiện thật, không phải giả định: xem §🔴 Phát hiện chặn
  - [ ] Tạo `src/selftest/fallbackReport.ts` mang hai chuỗi đó, đúng khuôn tiền lệ `src/selftest/eventName.ts` — module bé, import tĩnh được vào bundle chính mà không kéo `scopeCheck.ts` theo
  - [ ] ⛔ **Đừng** đưa hai chuỗi này vào `vi.json`. Chúng là chẩn đoán cho log CI, không phải chuỗi giao diện; `vi.json` là tài nguyên **hiển thị**. Trộn hai thứ là làm hỏng chính ranh giới story này dựng
  - [ ] ⛔ **Đừng** import từ `./selftest/scopeCheck` — `App.vue:12-16` đã ghi rõ vì sao import tĩnh tệp đó là phá bất biến *"mã self-check không vào bundle release"*
  - [ ] Sau khi dời: `src/App.vue` phải **sạch tiếng Việt ở vị trí mã**, chỉ còn tiếng Việt trong comment

- [ ] **Task 6 — `scripts/check-i18n.mjs`: cổng có mã thoát** (AC: 1, 2, 5)
  - [ ] Node thuần, `.mjs`, khuôn theo `scripts/check-deps.mjs` — kể cả `abort()` cho lỗi hạ tầng và **ngưỡng sàn** cho số tệp quét được
  - [ ] ⚠️ **Node chứ không bash.** Ice đã chốt 2026-08-03 (`check-deps.mjs:22-24`): `npm run` trên Windows chạy qua `cmd.exe`, không có bash. Một cổng chỉ canh nửa số nền tảng thì không canh được NFR14
  - [ ] Năm phép kiểm, xem §Khung `check-i18n.mjs` để biết chi tiết từng cái:
    - **Kiểm A** — không ký tự có dấu tiếng Việt ở **vị trí mã** của `src-tauri/src/**/*.rs` và `src/**/*.vue`
    - **Kiểm B** — `vi.json` phẳng; khoá khớp `^[a-z0-9]+(\.[a-z0-9_]+)+$`; không giá trị rỗng
    - **Kiểm C** — placeholder trong mọi giá trị khớp `\{[a-z_][a-z0-9_]*\}`; `{}` rỗng hoặc `{Ten}` hoa là FAIL
    - **Kiểm D** — giọng văn UX-DR47 phần máy chấm được: không `chúng tôi`, không `bạn` đứng thành tiếng riêng trong giá trị `vi.json`
    - **Kiểm E** — hành vi AC4: nạp `src/i18n/resolve.ts`, khẳng định ba đường — khoá có · khoá thiếu trả khoá nguyên văn không ném · tham số nội suy đúng
  - [ ] ⛔ **NGƯỠNG SÀN, bắt buộc.** Quét được 0 tệp `.rs` hoặc 0 tệp `.vue` ⇒ `abort()`, **không** phải "đạt". Đây là bẫy số 2 mà `check-deps.mjs:15-17` đã đâm vào một lần: *"cây rỗng đọc thành sạch"*. Sàn hôm nay: **≥ 14 tệp `.rs`**, **≥ 1 tệp `.vue`**
  - [ ] Danh sách miễn trừ viết **ngay trong script**, mỗi mục kèm **một câu lý do**. ⛔ Không miễn trừ im lặng bằng cách thu hẹp glob — xem §Ranh giới quét

- [ ] **Task 7 — Chứng minh từng cổng bằng ĐỎ trước, XANH sau** (AC: 2, 4, 5)
  - [ ] Với **mỗi** kiểm A–E: cố ý tạo một vi phạm → chạy → phải **đỏ**, và **đỏ đúng dòng đúng lý do** → gỡ vi phạm → phải **xanh**
  - [ ] Vi phạm mẫu, mỗi kiểm một cái: A — thêm `const x = 'Đã lưu'` vào một `.vue`; B — lồng một object trong `vi.json`; C — đổi một placeholder thành `{Path}`; D — thêm `"Bạn hãy thử lại."`; E — sửa `resolve.ts` cho ném khi thiếu khoá
  - [ ] Ghi bảng kết quả (kiểm · vi phạm · thông báo nhận được · mã thoát) vào §Debug Log References
  - [ ] ⛔ **Một cổng chưa từng đỏ là một cổng chưa được chứng minh.** Story 1.3 §Task 11 và Story 1.4 §Task 3 đã đặt tiền lệ này; đừng phá

- [ ] **Task 8 — Gắn MỘT bước vào pipeline đã có** (AC: 2)
  - [ ] `package.json` → thêm `"check:i18n": "node scripts/check-i18n.mjs"`, đúng khuôn ba script đã có
  - [ ] `.github/workflows/ci.yml` → thêm **một** bước `npm run check:i18n` trong job `check` đã có
  - [ ] ⛔ **Không dựng workflow thứ hai.** AC4 của Story 1.3 cấm tường minh; khối *"CHỖ MÓC CHO EPIC SAU"* ở `ci.yml:420-435` là chỗ đã chừa sẵn
  - [ ] Đặt bước **trước** `npm run build` (`ci.yml:100`), cạnh `check:deps` (`ci.yml:92`): nó chạy trong vài giây, không cần `dist/`, không cần cửa sổ đồ hoạ. Một chuỗi lọt vào nên đỏ **trước** khi tốn một lượt biên dịch Rust
  - [ ] ⛔ **Đừng đặt nó xuống cụm cuối** nơi `check:scope` / `check:scope:bundled` đang đứng — hai bước đó cần webview, bước này thì không
  - [ ] ⛔ **Đừng sắp xếp lại các bước đã có.** Thêm một bước, không mổ lại job
  - [ ] ⚠️ **Story 1.4 cũng đang thêm một bước (`check:tokens`) vào đúng chỗ này và chưa dev xong.** Cái nào vào trước thì cái sau đặt bước của mình **kề bên**, không đụng bước kia. Nếu `check:tokens` chưa có mặt, **đừng thêm hộ**

- [ ] **Task 9 — Đóng sổ: doc-comment, mục Deferred, README** (AC: 1, 2, 3)
  - [ ] Sửa `src-tauri/src/core/i18n/mod.rs`: thay câu *"Hình dạng thật … là quyết định của Story 1.5"* bằng **câu trả lời**, kèm lý do chọn macro
  - [ ] Sửa `src-tauri/src/commands/mod.rs:7-8` — doc-comment ở đó đã trỏ tới `core::i18n`; cập nhật cho khớp tên kiểu thật (`IpcError`, `MessageKey`)
  - [ ] `deferred-work.md:19` → đánh dấu **đã đóng**, ghi cơ chế đóng nó (`scripts/check-i18n.mjs` + bước CI). ⛔ Đừng xoá dòng cũ — khuôn của tệp đó là gạch ngang rồi ghi kết quả, không phải xoá
  - [ ] Tạo `src/i18n/README.md` theo khuôn năm README đã có ở `src/{commands,layout,modes,panels,tokens}/`: hình dạng phẳng · cách thêm khoá · miễn trừ · lệnh chạy cổng

---

## Dev Notes

### Ranh giới phạm vi — đọc trước khi gõ dòng đầu tiên

| Story này **có** làm | Story này **KHÔNG** làm |
|---|---|
| `src/i18n/**` — `vi.json`, `resolve.ts`, `index.ts`, `README.md` | Bất kỳ panel, mode hay layout nào — **Story 1.14** |
| `src-tauri/src/core/i18n/mod.rs` — danh mục `MessageKey` | Bộ token màu/chữ, lint màu — **Story 1.4** |
| `IpcError` + test hình dạng dây | `CommandRegistry`, id command, focus — **Story 1.6** |
| `scripts/check-i18n.mjs` + **một** bước trong `ci.yml` đã có | Một `#[tauri::command]` thật nào — chưa story nào cần |
| Dời hai chuỗi chẩn đoán khỏi `App.vue` (Task 5) | Đổi hành vi self-check của Story 1.2 |
| **Bộ khoá mồi hai cái**, đúng hai cái | Từ vựng khoá cho tính năng chưa dựng — mỗi story sau tự thêm |
| `deferred-work.md:19` | Bản tiếng Anh của giao diện — v1 **chỉ tiếng Việt** (NFR16) |

⛔ **Không đụng tới:** `src-tauri/tauri.conf.json` · `Cargo.toml` · `package.json` *(trừ đúng một dòng `scripts`)* · `src/selftest/scopeCheck.ts` · `_bmad-output/planning-artifacts/**`.

⛔ **Không thêm một phụ thuộc nào.** Xem §Vì sao không dùng `vue-i18n`.

---

### 🔴 Phát hiện chặn — repo hiện tại đã vi phạm chính luật story này sắp dựng

Cổng của Task 6 **sẽ đỏ ngay lượt đầu**, ở ba nhóm tệp khác nhau, và chỉ **một** trong ba là vi phạm thật. Dev phải phân biệt được ba nhóm này **trước** khi viết script, nếu không sẽ hoặc thu hẹp glob cho xanh (làm cổng thành đồ trang trí), hoặc đi dịch 20 thông báo `assert!` sang tiếng Anh (làm hỏng tài liệu nội tuyến mà không được gì).

Số đếm dòng có ký tự dấu tiếng Việt, đo trên `HEAD = 0255163`:

| Tệp | Dòng có dấu | Ở đâu | Phán quyết |
|---|---|---|---|
| `src-tauri/src/**/*.rs` *(mã sản phẩm)* | 21 | **chỉ trong comment** | ✅ **sạch** — không một string literal nào |
| `src-tauri/tests/config_invariants.rs` | 183 | comment **+ ~20 thông báo `assert!`** | ⚠️ **miễn trừ, phải viết ra** |
| `src/App.vue` | 26 | comment **+ 2 template literal** *(`:38`, `:54`)* | 🔴 **vi phạm thật — Task 5 dời đi** |
| `src/selftest/scopeCheck.ts` | 124 | comment + rất nhiều literal chẩn đoán | ⚠️ ngoài phạm vi AC2 (`.ts`), vẫn phải khai |

**Ba nhóm, ba phán quyết khác nhau:**

1. **Comment tiếng Việt ở mọi nơi — KHÔNG phải vi phạm.** Toàn bộ dự án này tự tài liệu hoá bằng tiếng Việt; đó là quy ước có chủ ý, đọc `lib.rs`, `check-deps.mjs`, `eventName.ts` là thấy. NFR16 nói về **chuỗi hiển thị**, không nói về comment. ⛔ Một cổng bắt comment sẽ đỏ vĩnh viễn ở mọi tệp và sẽ bị gỡ trong tuần — đúng cách hỏng đắt hơn hẳn việc không có cổng, mà `ci.yml:410-418` đã ghi thành bài học.
2. **Thông báo `assert!` trong `src-tauri/tests/**` — miễn trừ, nhưng phải KHAI.** Chúng không bao giờ vượt IPC, không bao giờ được render, và người đọc chúng là người đang sửa test. Dịch chúng sang tiếng Anh là mất giá trị tài liệu để đổi lấy con số không. **⛔ Nhưng miễn trừ phải là một dòng viết ra trong script kèm lý do, không phải một glob lặng lẽ hẹp lại.** Cùng luật mà Story 1.4 §Kiểm C áp cho cặp màu không dùng: *một danh sách kiểm tự rút gọn để cho xanh là đúng thứ cổng tồn tại để chặn.*
3. **`src/App.vue:38,54` — vi phạm thật, dời.** `.vue` nằm nguyên văn trong AC2. Hai chuỗi này là chẩn đoán cho log CI chứ không phải giao diện, nên chúng thuộc `src/selftest/**` (miễn trừ như nhóm 2) chứ không thuộc `vi.json`. Dời sang `src/selftest/fallbackReport.ts` giải cả hai vế: `App.vue` sạch, và miễn trừ gom về **một thư mục** thay vì rải rác.

> **Câu hỏi cho Ice ở §Câu hỏi cho Ice** về nhóm 2. **Nếu Ice chưa trả lời khi dev bắt đầu: miễn trừ `src-tauri/tests/**`, ghi rõ lý do trong script và trong Completion Notes.** Tiền lệ: quyết định #3 của Ice ở Story 1.3, và §Phát hiện chặn của Story 1.4.

---

### Trạng thái repo hiện tại — số, không phải mô tả

Đọc lúc dựng story, `HEAD = 0255163`:

| | |
|---|---|
| `src/i18n/vi.json` | tồn tại, nội dung là **`{}`** — 3 byte |
| `src-tauri/src/core/i18n/mod.rs` | tồn tại, **chỉ doc-comment**, không một dòng mã. Nó nói thẳng: *"quyết định của Story 1.5"* |
| `src-tauri/src/commands/mod.rs` | **chỉ doc-comment**, chưa một `#[tauri::command]` nào |
| Số tệp `.rs` trong `src-tauri/src/**` | **17** *(sàn Kiểm A đặt ở 14)* |
| Số tệp `.vue` trong `src/**` | **1** — `App.vue` |
| `src-tauri/tests/` | **một** tệp — `config_invariants.rs`, 514 dòng, 12+ test |
| Node trên máy Ice / CI | **v22.22.2** / `node-version: '22'` |
| Script cổng đã có | `check:deps` · `check:scope` · `check:scope:bundled` |
| Bước CI đã có trong job `check` | `check:deps` `:92` → `npm run build` `:100` → `cargo test` `:106` → build/đo → `check:scope:bundled` `:370` → `check:scope` `:389` |

**Ba lệnh kế thừa** *(Story 1.2 → 1.3, chép đúng, đừng phát minh lại)*:

```bash
npm run check:deps                                 # 13 phép kiểm — cây phụ thuộc
npm run check:scope                                # Kiểm 3 hai chiều — cần cửa sổ đồ hoạ
cargo test --manifest-path src-tauri/Cargo.toml    # bất biến cấu hình; CẦN `dist/` tồn tại
```

⚠️ `cargo test` **cần `dist/` tồn tại** — `generate_context!` nhúng frontend lúc biên dịch. Chạy `npm run build` trước, hoặc `cargo test` sẽ gãy vì một lý do không liên quan tới thay đổi của story này.

---

### Bốn thứ sẽ hỏng im lặng

Ba trong bốn cái **cho ra một lượt CI XANH** với kết quả vô nghĩa.

**1. 🔴 `#[serde(rename_all = "camelCase")]` — hợp đồng AD-21 gãy, không lỗi nào được ném.**
Thói quen viết Tauri là đặt `rename_all = "camelCase"` lên mọi struct qua IPC cho hợp phong cách JS. Ở đây nó biến `message_key` thành `messageKey`. Rust biên dịch sạch, `cargo test` xanh nếu test không so **chính tả khoá**, frontend nhận `undefined` và hiển thị chuỗi rỗng. AD-21 phát biểu bốn trường **nguyên văn** — chúng là dây, không phải sở thích. Test `ipc_error_wire_shape` phải so `to_value(...).as_object().keys()` **đúng bốn chuỗi đó**.

**2. 🔴 `#[derive(Serialize)]` trần trên `enum MessageKey` — khoá ra sai mà vẫn là chuỗi hợp lệ.**
Serde mặc định serialize unit variant thành **tên biến thể**: `MessageKey::IoReadFailed` → `"IoReadFailed"`. Đó là một chuỗi, JSON hợp lệ, không lỗi. Frontend tra `"IoReadFailed"` trong `vi.json`, không thấy, và theo đúng AC4 nó **hiển thị khoá nguyên văn rồi ghi cảnh báo** — nghĩa là hỏng đúng kiểu *"trông như đang chạy"*. Viết `Serialize` bằng tay: `serializer.serialize_str(self.as_str())`.

**3. 🔴 Glob quét rỗng đọc thành "sạch".**
`check-deps.mjs:15-17` đã ghi lại đúng bẫy này một lần rồi: `npm ls` trên checkout chưa `npm ci` trả một mục và exit 0 → *"cây npm sạch (1 mục)"*. Ở đây tương đương: một glob viết sai (`src/**.vue` thay vì `src/**/*.vue`) khớp 0 tệp, script in *"không tìm thấy vi phạm"*, exit 0, và cổng chết im lặng ngay ngày nó ra đời. **Ngưỡng sàn là bắt buộc, không phải nice-to-have.**

**4. ⚠️ Bóc comment bằng `replace(/\/\/.*$/gm, '')` cắt nhầm giữa string literal.**
`"https://example.com"` có `//` bên trong dấu nháy. Một lượt bóc comment ngây thơ biến nó thành `"https:` và mọi thứ sau đó lệch. Rust còn có raw string `r#"…"#`, và `.vue` có ba vùng cú pháp khác nhau. **Phải quét từng ký tự, có trạng thái** — xem §Khung `check-i18n.mjs`. Hậu quả nếu làm ẩu là cả hai chiều: bỏ sót vi phạm thật, **và** báo động giả trên một URL vô hại.

---

### Danh mục `MessageKey` — hình dạng đã chốt

`core/i18n/mod.rs` để ngỏ ba đường: *enum? hằng? sinh mã từ `vi.json`?* Chốt: **enum, khai qua một `macro_rules!`, không sinh mã lúc build.**

Vì sao không sinh mã từ `vi.json` lúc build: nó thêm một `build.rs` và buộc `vi.json` thành đầu vào biên dịch Rust — nghĩa là sửa một dấu phẩy trong chuỗi giao diện làm biên dịch lại nửa cây Rust, mỗi lần, trên cả hai nền tảng CI. Giá đó trả cho một thứ mà **một test đọc file lúc chạy** đã bắt được y hệt.

Vì sao macro chứ không viết tay hai chỗ: `ALL` và `as_str()` phải khớp nhau, mà test đồng bộ với `vi.json` chạy trên `ALL`. Thêm một biến thể mà quên thêm vào `ALL` ⇒ test vẫn xanh, khoá vẫn thiếu trong `vi.json`, và nó lộ ra ở tay người dùng. Một khai báo, hai thứ sinh ra:

```rust
macro_rules! message_keys {
    ($($variant:ident => $key:literal),+ $(,)?) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        pub enum MessageKey { $($variant),+ }

        impl MessageKey {
            /// Mọi biến thể. Sinh từ CÙNG khai báo với `as_str` nên không trôi được.
            pub const ALL: &'static [MessageKey] = &[$(MessageKey::$variant),+];

            pub const fn as_str(self) -> &'static str {
                match self { $(MessageKey::$variant => $key),+ }
            }
        }
    };
}

message_keys! {
    Unknown      => "err.unknown",
    IoReadFailed => "err.io.read_failed",
}
```

`Serialize` viết tay ngay dưới, một hàm bốn dòng. ⛔ Đừng `#[derive(Serialize)]` — xem §Bốn thứ sẽ hỏng im lặng #2.

---

### Nghiệm thu AC3 khi chưa có một `#[tauri::command]` nào

`src-tauri/src/commands/mod.rs` hôm nay **chỉ có doc-comment** — không một hàm IPC nào tồn tại. AC3 nói *"một lỗi phát sinh ở Rust **khi đi qua ranh giới IPC**"*, nên câu hỏi hợp lý là: nghiệm thu bằng gì khi chưa có ranh giới nào để đi qua?

**Câu trả lời: bằng test serialize, và nó nghiệm thu đúng thứ AC3 nói.** Tauri v2 đưa giá trị trả về của `#[tauri::command]` qua IPC bằng **chính `serde_json`** — không có tầng biến đổi nào chen giữa. `serde_json::to_value(IpcError…)` cho ra **đúng byte** mà frontend sẽ nhận. Một test so bốn khoá và chính tả của chúng là bằng chứng về dây, không phải một phép mô phỏng.

⛔ **Đừng dựng một `#[tauri::command]` giả để "chứng minh cho thật".** Ba lý do:

1. Nó là mã sản phẩm không ai gọi — đúng thứ §Ranh giới phạm vi loại ra.
2. Chạy nó cần một webview, nghĩa là một bước CI cần phiên đồ hoạ. `ci.yml:360-368` đã ghi giá của loại bước đó: một lượt biên dịch profile `dev` **riêng**, đắt nhất trên macOS (hệ số ×10). Trả giá đó cho một hàm sẽ bị xoá ở story sau là sai chỗ.
3. Vòng chạy thật đến **miễn phí** ở Story 1.6 trở đi, khi command thật đầu tiên xuất hiện. Hợp đồng đã bị test khoá lại từ đây, nên nếu vòng đó lệch thì lệch ở phía command mới, không ở phía `IpcError`.

**Ghi mệnh đề này vào Completion Notes**, kèm phiên bản `tauri` đã kiểm (`=2.11.5`). Lượt code review sẽ hỏi đúng câu hỏi trên, và câu trả lời phải nằm trong story chứ không nằm trong trí nhớ ai đó.

---

### Vì sao `resolve.ts` phải thuần — và điều gì xảy ra nếu không

AC4 là một mệnh đề về **hành vi lúc chạy** (*"hiển thị khoá nguyên văn và ghi cảnh báo, không sập"*), không phải về hình dạng tệp. Nghiệm thu nó cần **gọi hàm thật**.

Dự án **không có bộ chạy test frontend**, và thêm một (`vitest`) là thêm một phụ thuộc — mà mọi phụ thuộc mới phải rà GPLv3 và vào bảng Stack **trước khi** thêm (NFR15, `ARCHITECTURE-SPINE.md#Consistency Conventions`). Đó là quyết định của Ice, không phải hệ quả phụ của story này.

Đường đi không tốn gì: **Node ≥ 22.18 bóc kiểu TypeScript mặc định**, nên `check-i18n.mjs` `import()` thẳng được `src/i18n/resolve.ts`. Máy Ice v22.22.2 ✅, CI `node-version: '22'` ✅.

Điều kiện, và cả ba đều là lý do `resolve.ts` không được import gì:

- Node **chỉ bóc kiểu**, không phân giải `./vi.json` theo luật bundler của Vite, không hiểu `.vue`.
- Cú pháp phải **"erasable-only"**: ⛔ không `enum`, ⛔ không `namespace`, ⛔ không parameter property (`constructor(private x)`). `type` / `interface` / annotation đều được.
- Nên: `resolve.ts` = hàm thuần + kiểu. `index.ts` = chỗ duy nhất chạm `vi.json` và Vue.

⚠️ **Nếu `import()` thất bại** (Node cũ, cờ tắt): `abort()` với thông báo nêu rõ *"Kiểm E KHÔNG chạy được"* và **exit 1**. ⛔ Không bỏ qua Kiểm E rồi exit 0 — `check-deps.mjs:60-66` đã đặt luật này thành chữ: *"Lỗi hạ tầng ≠ phép kiểm đỏ. Dừng ngay, đừng báo cáo một kết quả không có thật."*

---

### Khung `check-i18n.mjs` — hình dạng, không phải bản chép

Khuôn chung lấy nguyên từ `scripts/check-deps.mjs`: `pass()` / `fail()` đếm lỗi, `abort()` cho lỗi hạ tầng, in tiêu đề từng kiểm, exit 1 nếu `failures !== 0`.

**Bộ dấu tiếng Việt** — 134 ký tự (67 chữ thường + 67 hoa), đủ để phân biệt tiếng Việt với mọi ngôn ngữ Latin khác:

```
àáảãạăằắẳẵặâầấẩẫậèéẻẽẹêềếểễệìíỉĩịòóỏõọôồốổỗộơờớởỡợùúủũụưừứửữựỳýỷỹỵđ
```

**Kiểm A — quét có trạng thái, không `replace` ngây thơ.**
Duyệt từng ký tự, giữ một trong các trạng thái: `code` · `line_comment` · `block_comment` · `string` *(mang theo ký tự đóng và cờ raw)* · **`vue_template_text`**.

- Trong `line_comment` / `block_comment` ⇒ **bỏ qua** (nhóm 1 của §Phát hiện chặn).
- Trong `string` hoặc `code` ⇒ có dấu là **FAIL**, in **đường dẫn:dòng:cột** và trích 60 ký tự quanh chỗ đó.
- Rust: `//` `///` `//!` `/* */` *(lồng nhau được)*, `"…"` với escape `\"`, `r"…"` và `r#"…"#` *(không escape)*.
- Vue: `<!-- -->` trong template; `//` và `/* */` trong `<script>`; `/* */` trong `<style>`.
- 🔴 **`vue_template_text` là vế mà một lượt cài đặt vội sẽ bỏ sót.** `<button>Lưu</button>` **không có dấu nháy nào** — nó là vi phạm nặng nhất của AC2 và một script chỉ soi string literal sẽ không thấy. Text node giữa hai thẻ **phải bị quét như mã**. Cho qua: khoảng trắng, chữ số, dấu câu ASCII, và nội dung trong `{{ }}`.

**Kiểm B — hình dạng `vi.json`.** Object phẳng; mọi giá trị là `string`; khoá khớp `^[a-z0-9]+(\.[a-z0-9_]+)+$` *(bắt buộc ≥ 1 dấu chấm — khoá phải có tiền tố miền)*; không giá trị rỗng. Bắt luôn: `JSON.parse` gãy ⇒ `abort()`, không phải `fail()`.

**Kiểm C — placeholder.** Mọi `{…}` trong giá trị phải khớp `\{[a-z_][a-z0-9_]*\}`. Bắt `{}`, `{Path}`, `{0}`, `{ path }`.

**Kiểm D — giọng văn, phần máy chấm được.** Quét giá trị `vi.json` tìm `chúng tôi` và `bạn` **đứng thành tiếng riêng** *(biên tiếng, không phải substring)*, không phân biệt hoa thường. Danh sách ngoại lệ khai tường minh trong script, mỗi mục một dòng lý do — mặc định **rỗng**.

**Kiểm E — hành vi AC4.** `await import('../src/i18n/resolve.ts')`, dựng resolver trên một catalog giả ba khoá, khẳng định:
`t('co.mat')` trả giá trị · `t('khong.co')` trả **đúng `'khong.co'`** và **không ném** · `t('co.tham_so', {path: '/x'})` nội suy đúng · `t('co.tham_so')` **giữ nguyên** `{path}` và không ném.

**Ngưỡng sàn.** `rsFiles.length < 14` hoặc `vueFiles.length < 1` ⇒ `abort()`. Số thật hôm nay: 17 và 1.

**Miễn trừ — viết ra, mỗi mục một câu lý do:**

```js
const EXEMPT = [
  ['src-tauri/tests/**',  'thông báo assert! — không vượt IPC, không render; người đọc là người sửa test'],
  ['src/selftest/**',     'chẩn đoán cho log CI; debug-only, import động, không vào bundle release'],
]
```

⛔ Miễn trừ **không** được cài bằng cách thu hẹp glob quét. Glob quét cả cây; miễn trừ là một bước lọc **có tên và có lý do**, và script **in ra** số tệp đã miễn trừ ở mỗi lượt chạy để nó không lặng lẽ phình lên.

---

### Vì sao không dùng `vue-i18n` — và điều kiện để đổi ý

`vue-i18n` **11.4.8, MIT** *(kiểm chứng npm 2026-08-03)* — giấy phép tương thích GPL v3, không có gì sai về mặt pháp lý. Vẫn **không dùng**, ba lý do xếp theo sức nặng:

1. **NFR15 là một thủ tục, không phải một lượt kiểm.** Mọi phụ thuộc mới phải rà tương thích GPLv3 **trước khi** thêm **và ghi vào bảng Stack** — bảng Stack có 19 hàng đã ghim và `check:deps` cưỡng chế nó. Thêm hàng thứ 20 là quyết định của Ice.
2. **Phần dự án này cần chỉ là một hàm.** v1 **chỉ tiếng Việt** (NFR16) — không chuyển ngôn ngữ lúc chạy, không số nhiều, không định dạng theo vùng, không lazy-load bundle ngôn ngữ. Còn lại: tra một khoá phẳng, nội suy vài placeholder, xử lý khoá thiếu. Khoảng 40 dòng.
3. **Một thư viện làm Kiểm E đắt hơn hẳn.** `resolve.ts` thuần thì Node nạp thẳng. Một plugin Vue thì phải dựng app instance để kiểm — và điều đó kéo theo đúng bộ chạy test frontend mà lý do #1 vừa hoãn.

**Điều kiện mở lại:** khi có ngôn ngữ thứ hai thật *(ngoài phạm vi v1)*, hoặc khi nhu cầu số nhiều/định dạng vùng xuất hiện. Lúc đó: rà GPLv3 → thêm vào bảng Stack → thay `createResolver` **sau** ranh giới `t()`. Ranh giới đó chính là thứ story này dựng để lần đổi ý sau rẻ.

---

### Giọng văn — năm quy tắc, và cái nào máy chấm được

`EXPERIENCE.md#Voice and Tone:51-61` — viết cho **một người dịch chuyên nghiệp**, không phải cho người dùng phổ thông cần dỗ dành:

| Quy tắc | Ví dụ đúng | Ví dụ sai | Máy chấm? |
|---|---|---|---|
| Nói việc, không nói cảm xúc | *"Đã gộp hai câu."* | *"Tuyệt vời! Đã gộp xong 🎉"* | ✗ *(mắt)* |
| Nêu hệ quả, không chỉ nêu sự kiện | *"Câu mới chưa xác nhận — lịch sử của hai câu cũ vẫn tra lại được."* | *"Đã gộp."* | ✗ *(mắt)* |
| Không đổ lỗi người dùng | *"Nhà cung cấp không phản hồi"* | *"Bạn đã nhập sai khoá"* | ✗ *(mắt)* |
| Số liệu là số liệu | *"412 token · ước tính ~0,004 USD"* | *"một chút chi phí"* | ✗ *(mắt)* |
| Vô nhân xưng | *"Không đọc được tệp tại {path}."* | *"Chúng tôi không đọc được tệp bạn chọn."* | ✅ **Kiểm D** |

Ba quy tắc "mắt" nghiệm thu bằng cách chép hai chuỗi mồi vào §Completion Notes kèm một câu vì sao chọn chữ đó. Với **hai** chuỗi thì đó là việc năm phút; đây cũng là lý do §Task 3 cấm dựng sẵn một từ vựng lớn — 200 chuỗi thì không ai nghiệm thu nổi và quy tắc thoái hoá thành trang trí.

---

### Project Structure Notes

Vị trí tệp bám đúng Structural Seed (`ARCHITECTURE-SPINE.md#Cây nguồn:710`), không phát minh thư mục mới:

```text
src/i18n/
  vi.json          # ĐÃ CÓ (rỗng) — toàn bộ chuỗi giao diện (NFR16, AD-21)
  resolve.ts       # MỚI — hàm thuần, không import gì (xem §Vì sao resolve.ts phải thuần)
  index.ts         # MỚI — chỗ DUY NHẤT chạm vi.json; export `t`, `tError`, kiểu `IpcError`
  README.md        # MỚI — khuôn theo năm README đã có ở src/*/
src/selftest/
  fallbackReport.ts # MỚI (Task 5) — hai chuỗi chẩn đoán dời khỏi App.vue
src-tauri/src/core/i18n/
  mod.rs           # ĐÃ CÓ (chỉ doc-comment) — thay bằng danh mục MessageKey + IpcError
src-tauri/tests/
  …                # ba test mới — xem §Testing standards
scripts/
  check-i18n.mjs   # MỚI — khuôn theo check-deps.mjs
```

**Một biến thể so với Cây nguồn, có chủ ý:** spine viết `src/i18n/vi.json` như thể thư mục chỉ có một tệp. Ba tệp `.ts` thêm vào **không** phải chuỗi giao diện — chúng là cơ chế phân giải, và không có chỗ nào khác đúng hơn để đặt. Ghi ở đây để lượt rà soát sau không đọc nhầm thành trôi khỏi Structural Seed.

**Quy ước đặt tên** *(`ARCHITECTURE-SPINE.md#Consistency Conventions:540`)*: Rust `snake_case` · Vue component `PascalCase.vue` · tài nguyên chuỗi **phẳng theo khoá chấm**.

⚠️ **Khoá `vi.json` và id `CommandRegistry` dùng CÙNG một hình dạng nhưng KHÔNG cùng một không gian tên.** Story 1.6 AC2 nói *"dùng khoá chấm có tiền tố miền, **cùng hình dạng khoá `vi.json`**"* — cùng hình dạng, hai danh mục. ⛔ Đừng dựng một bảng tra dùng chung; một command và nhãn của nó là hai thứ đổi độc lập với nhau.

---

### Testing standards — thừa kế nguyên từ Story 1.2 và 1.3

- **Mã thoát là phán quyết.** Một script in cảnh báo rồi trả 0 là một phép kiểm không cưỡng chế được gì.
- **Cây rỗng không phải cây sạch.** Ngưỡng sàn bắt buộc; đừng gỡ.
- **Phép kiểm phải có cả hai chiều.** Kiểm E phải kiểm cả khoá có lẫn khoá thiếu — chỉ kiểm một chiều thì một resolver luôn trả `''` cũng "qua".
- **Nghiệm thu bằng đỏ trước, xanh sau.** Task 7 là bắt buộc.
- **Lỗi hạ tầng ≠ phép kiểm đỏ.** `abort()` và exit 1, đừng báo một kết quả không có thật.

**Đặt test Rust ở đâu:** thêm vào `src-tauri/tests/` như một tệp mới *(`ipc_contract.rs` là tên gợi ý)*, **không** nhét vào `config_invariants.rs` — tệp đó có phạm vi đã khai ở doc-comment dòng 1 (*"bất biến cấu hình của Story 1.2"*) và trộn vào là làm hỏng thứ khiến nó đọc được. `tests/` `use auratranslate_lib::…` được — bố cục `lib.rs` + `main.rs` tồn tại chính vì lý do này (`lib.rs:3-4`).

**Đọc `vi.json` từ test Rust:** `PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("..").join("src/i18n/vi.json")` — cùng khuôn `config_invariants.rs:11-19`. ⚠️ `CARGO_MANIFEST_DIR` trỏ `src-tauri/`, nên phải lùi một cấp; và lỗi đọc file phải `panic!` kèm đường dẫn, đừng `unwrap()` trần.

**Lệnh nghiệm thu đầy đủ của story này:**

```bash
npm run check:i18n                                 # 5 kiểm — cổng NFR16/AD-21
npm run build                                      # dựng dist/ (cargo test cần)
cargo test --manifest-path src-tauri/Cargo.toml    # bất biến cấu hình + 3 test mới
npm run check:deps                                 # hồi quy: không phụ thuộc mới nào lọt vào
```

---

### Git intelligence — ba commit gần nhất nói gì về cách repo này làm việc

| Commit | Bài học áp thẳng vào story này |
|---|---|
| `0255163` | Chỉ đụng `_bmad-output/**`. Tài liệu quy hoạch và mã đi hai commit khác nhau — giữ nếp đó |
| `a2a5612` *(bash → Node)* | Cổng viết bằng **Node**, không bash. Ice đã đổi một script rồi vì Windows không có bash; ⛔ đừng viết `check-i18n.sh` |
| `a89b5ca` | Scaffold để lại **doc-comment thay cho mã** ở mọi module chưa tới lượt. `core/i18n/mod.rs` là một trong số đó, và nó **chỉ định đích danh Story 1.5**. Thay doc-comment bằng câu trả lời là một phần của định nghĩa "xong" |

Hai commit gần nhất còn cho thấy một nếp viết mã của repo: **mọi quyết định không hiển nhiên đều có một khối comment giải thích *vì sao*, kèm cả cái bẫy đã đâm phải.** Đọc `check-deps.mjs:9-28` hay `eventName.ts:1-21` để lấy đúng giọng. `check-i18n.mjs` và `core/i18n/mod.rs` phải viết cùng giọng đó — đặc biệt là ghi lại vì sao miễn trừ `tests/**` và vì sao quét có trạng thái.

---

### Latest tech — kiểm chứng 2026-08-03

- **`vue-i18n` 11.4.8, MIT** — tương thích GPLv3, **vẫn không dùng**. Lý do đầy đủ ở §Vì sao không dùng `vue-i18n`.
- **Node ≥ 22.18 bóc kiểu TypeScript mặc định** *(không cần cờ)*. Máy Ice v22.22.2, CI `node-version: '22'`. Đây là điều kiện của Kiểm E — và §Vì sao `resolve.ts` phải thuần ghi cả đường lui nếu nó vắng mặt.
- **Không phụ thuộc mới nào** trong story này. Bảng Stack 19 hàng không đổi, `check:deps` phải vẫn xanh sau khi xong.

---

### References

- [Source: `_bmad-output/planning-artifacts/epics.md#Story 1.5` — năm AC nguyên văn, `:1179-1210`]
- [Source: `_bmad-output/planning-artifacts/epics.md#NonFunctional Requirements` — NFR16, `:362`]
- [Source: `_bmad-output/planning-artifacts/epics.md#UX Design Requirements` — UX-DR47 năm quy tắc giọng văn, `:605`]
- [Source: `_bmad-output/planning-artifacts/epics.md#Additional Requirements` — AD-21 phát biểu rút gọn, `:443`]
- [Source: `_bmad-output/planning-artifacts/epics.md#Epic 1` — *"hai yêu cầu cắt ngang áp từ Giai đoạn 1, không được để lại sau"*, `:479`]
- [Source: `_bmad-output/planning-artifacts/epics.md#Story 1.6` — id command *"cùng hình dạng khoá `vi.json`"*, `:1230`]
- [Source: `_bmad-output/planning-artifacts/epics.md#Story 1.14` — panel trống *"nêu rõ trạng thái bằng chuỗi trong `vi.json`"*, `:1576`]
- [Source: `_bmad-output/planning-artifacts/epics.md#Story 4.10` — lỗi AI dùng lại đúng hình dạng bốn trường, `:3178-3180`]
- [Source: `_bmad-output/planning-artifacts/epics.md#Story 10.9` — nghiệm thu cuối: *"không chuỗi tiếng Việt nào trong `.rs` hay `.vue`"*, `:6354-6355`]
- [Source: `.../ARCHITECTURE-SPINE.md#AD-21` — Binds *tất cả*; Prevents *"NFR16 bị thủng ở tầng lỗi — chỗ dễ quên nhất và đắt nhất để sửa sau"*, `:269-273`]
- [Source: `.../ARCHITECTURE-SPINE.md#AD-22` — *"không bao giờ tự thử lại"*, ràng buộc lên `retryable`, `:275-279`]
- [Source: `.../ARCHITECTURE-SPINE.md#Consistency Conventions` — `vi.json` **phẳng** theo khoá chấm `:540`; hình dạng lỗi `:550`; chuỗi giao diện `:551`; NFR15 rà giấy phép trước khi thêm `:557`]
- [Source: `.../ARCHITECTURE-SPINE.md#Cây nguồn` — `i18n/vi.json` *"toàn bộ chuỗi giao diện (NFR16, AD-21)"*, `:710`]
- [Source: `.../ux-designs/.../EXPERIENCE.md#Voice and Tone` — năm quy tắc + *"Rust trả `{ code, message_key, params, retryable }`"*, `:51-61`]
- [Source: `.../prds/.../prd.md` — NFR16 và giải thích *"rẻ nếu làm từ đầu, rất đắt nếu làm sau"*, `:853-855`]
- [Source: `_bmad-output/implementation-artifacts/deferred-work.md:19` — mục story này đóng]
- [Source: `_bmad-output/implementation-artifacts/1-3-…-moi-lan-push.md:481` — *"NFR16 không có cơ chế cưỡng chế → chủ sở hữu là Story 1.5"*]
- [Source: `_bmad-output/implementation-artifacts/1-4-…-tu-dong.md:147` — 1.4 giao lại: *"chuỗi giao diện, `vi.json`, hình dạng lỗi IPC — Story 1.5"*]
- [Source: `src-tauri/src/core/i18n/mod.rs:1-14` — doc-comment giao việc, *"quyết định của Story 1.5"*]
- [Source: `src-tauri/src/commands/mod.rs:7-8` — *"`message_key` lấy từ danh mục ở `core::i18n`"*]
- [Source: `src-tauri/src/lib.rs:3-4` — vì sao `tests/` `use` được mã sản phẩm]
- [Source: `src-tauri/tests/config_invariants.rs:11-19` — khuôn đọc JSON từ `CARGO_MANIFEST_DIR`]
- [Source: `scripts/check-deps.mjs:9-28,47-51,60-66` — mã thoát là phán quyết · ngưỡng sàn · `abort()` · vì sao Node chứ không bash]
- [Source: `src/App.vue:12-16,38,54` — vì sao không import tĩnh `scopeCheck`; hai chuỗi Task 5 phải dời]
- [Source: `src/selftest/eventName.ts:1-21` — tiền lệ module bé tách riêng để import tĩnh an toàn]
- [Source: `.github/workflows/ci.yml:11-13,92,100,420-435` — một pipeline duy nhất; vị trí `check:deps`; khối *"CHỖ MÓC CHO EPIC SAU"*]
- [Source: `tsconfig.json:12,17-18` — `resolveJsonModule` đã bật; `noUnusedLocals`/`noUnusedParameters` bật]
- [Web 2026-08-03] npm — `vue-i18n` **11.4.8**, MIT
- [Web 2026-08-03] Node.js — bóc kiểu TypeScript bật mặc định từ **22.18**; cú pháp phải "erasable-only" *(không `enum`, `namespace`, parameter property)*

---

## Câu hỏi cho Ice

*(Không chặn — dev có đường mặc định cho cả ba. Trả lời sau cũng được.)*

1. **Miễn trừ `src-tauri/tests/**` khỏi Kiểm A — đồng ý không?** ~20 thông báo `assert!` trong `config_invariants.rs` viết bằng tiếng Việt. Chúng không vượt IPC và không được render; người đọc chúng là người đang sửa test. Dịch sang tiếng Anh là mất giá trị tài liệu để đổi lấy con số không.
   → **Mặc định nếu chưa trả lời: miễn trừ, ghi lý do trong script và Completion Notes.**

2. **`code` và `message_key` có nên 1:1 ở v1 không?** Hai trường phục vụ hai việc (rẽ nhánh máy · hiển thị người), nhưng hôm nay chưa có nhánh nào cần rẽ. Giữ 1:1 thì đơn giản; tách sớm thì đắt vô ích.
   → **Mặc định: 1:1 hôm nay, giữ nguyên hai trường, ghi rõ trong doc-comment rằng chúng được phép rời nhau về sau.**

3. **Hai chuỗi mồi có đủ không, hay muốn thêm một bộ khoá lỗi chung ngay từ đây?** Story cố ý giữ ở hai — §Task 3 giải thích vì sao một từ vựng dựng sẵn cho tính năng chưa tồn tại sẽ sai.
   → **Mặc định: đúng hai chuỗi.**

---

## Dev Agent Record

### Agent Model Used

### Debug Log References

### Completion Notes List

### File List

---

## Change Log

| Ngày | Ai | Gì |
|---|---|---|
| 2026-08-03 | Bob (SM) | Dựng story từ `epics.md#Story 1.5`, ARCHITECTURE-SPINE (AD-21, Consistency Conventions, Cây nguồn), EXPERIENCE.md (UX-DR47), trạng thái repo `HEAD = 0255163`, và bàn giao từ Story 1.2/1.3/1.4. Ghi §🔴 Phát hiện chặn — ba nhóm vi phạm hiện có, ba phán quyết khác nhau. |
