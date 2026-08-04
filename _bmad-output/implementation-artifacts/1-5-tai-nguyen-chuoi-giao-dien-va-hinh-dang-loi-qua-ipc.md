---
baseline_commit: dd2df6680a895cf42e2fb557f21213f673dcc8e5
---

# Story 1.5: Tài nguyên chuỗi giao diện và hình dạng lỗi qua IPC

Status: done

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

- [x] **Task 1 — Chụp ảnh vi phạm hiện có TRƯỚC khi viết một dòng mã** (AC: 2)
  - [x] Chạy phép đếm dưới đây trên `HEAD` và chép kết quả vào §Debug Log References. Đây là đường cơ sở; không có nó thì không chứng minh được cổng ở Task 6 thật sự bắt được gì
  - [x] Số đã đo lúc dựng story (`HEAD = 0255163`) ở §Trạng thái repo hiện tại — đối chiếu, lệch thì dừng và đọc lại §🔴 Phát hiện chặn
  - [x] ⛔ **Đừng "dọn" gì ở task này.** Task 1 chỉ đo

- [x] **Task 2 — `src/i18n/resolve.ts`: hàm phân giải thuần, không import gì** (AC: 1, 4)
  - [x] Tạo `src/i18n/resolve.ts` chứa `createResolver(catalog: Record<string, string>)` trả về `t(key, params?)`
  - [x] ⛔ **Tệp này KHÔNG được `import` bất cứ thứ gì** — không Vue, không JSON, không `@tauri-apps/api`. Lý do ở §Vì sao `resolve.ts` phải thuần: nó là tệp duy nhất `check-i18n.mjs` nạp được để kiểm **hành vi** AC4, và Node chỉ bóc kiểu được cho TS "erasable-only"
  - [x] ⛔ Không `enum`, không `namespace`, không parameter property — ba thứ Node từ chối bóc kiểu. Dùng union type chuỗi nếu cần
  - [x] Hành vi khoá thiếu (AC4): trả **đúng khoá nguyên văn**, `console.warn` **một lần cho mỗi khoá** (dedupe bằng `Set`), **không ném**
  - [x] ⚠️ Dedupe không phải tối ưu vặt: một khoá thiếu trong template Vue chạy lại mỗi lần render — không dedupe thì console ngập và cảnh báo thật chìm mất
  - [x] Nội suy tham số: cú pháp `{ten_tham_so}`, tên khớp `[a-z_][a-z0-9_]*`. Tham số thiếu ⇒ **giữ nguyên placeholder** + `console.warn`, không ném, không thay bằng `undefined`
  - [x] Tạo `src/i18n/index.ts`: `import catalog from './vi.json'` → `export const t = createResolver(catalog)`. `resolveJsonModule` đã bật sẵn ở `tsconfig.json:12`
  - [x] Thêm `tError(err: IpcError): string` — nhận nguyên payload lỗi của AC3, trả chuỗi đã phân giải. Kiểu `IpcError` khai ở `src/i18n/index.ts` và **khớp từng chữ** bốn trường của AC3

- [x] **Task 3 — `vi.json`: hình dạng phẳng và bộ khoá mồi tối thiểu** (AC: 1, 5)
  - [x] `src/i18n/vi.json` hiện là `{}` (3 byte). Viết lại thành object **phẳng**, khoá chấm
  - [x] ⛔ **KHÔNG lồng object.** `{"lookup": {"empty_result": "…"}}` là sai hình dạng — xem AC1
  - [x] ⛔ **KHÔNG dựng sẵn một từ vựng khoá cho tính năng chưa tồn tại.** Story này sở hữu **cơ chế**, không sở hữu **từ vựng**. Mỗi story sau tự thêm khoá của nó. Một `vi.json` 200 khoá cho panel chưa ai dựng là 200 chuỗi không ai kiểm được, và chúng sẽ sai
  - [x] Bộ mồi tối thiểu — đủ để chứng minh cả bốn AC, không hơn:
    - `err.unknown` — khoá dự phòng cuối cùng của AD-21. Mọi lỗi Rust chưa phân loại được rơi vào đây thay vì rơi vào một chuỗi viết tay
    - `err.io.read_failed` với tham số `{path}` — chứng minh đường nội suy tham số chạy thật
  - [x] Soạn hai chuỗi theo UX-DR47: vô nhân xưng, nêu nguyên nhân, không đổ lỗi. Ví dụ hình dạng *(dev soạn bản cuối, đây không phải bản chép)*: `"Không đọc được tệp tại {path}."` — **không** `"Bạn đã chọn một tệp không đọc được."`
  - [x] Ghi bản cuối của cả hai chuỗi + lý do chọn chữ vào §Completion Notes (nghiệm thu ba quy tắc UX-DR47 mà máy không chấm được)

- [x] **Task 4 — Rust: danh mục `MessageKey` và kiểu `IpcError`** (AC: 3)
  - [x] `src-tauri/src/core/i18n/mod.rs` hiện **chỉ có doc-comment** và nói thẳng: *"Hình dạng thật của danh mục là quyết định của **Story 1.5**"*. Đây là chỗ trả lời
  - [x] Dùng `macro_rules! message_keys!` khai **một chỗ duy nhất** sinh ra cả `enum MessageKey`, `MessageKey::ALL` và `as_str()` — xem khung ở §Danh mục `MessageKey`. Lý do: `ALL` và `as_str()` viết tay sẽ trôi khỏi nhau, và test đồng bộ với `vi.json` chạy trên `ALL` nên `ALL` thiếu một biến thể là test xanh giả
  - [x] `Serialize` cho `MessageKey` = `serialize_str(self.as_str())`. ⛔ Không `#[derive(Serialize)]` trần trên enum — mặc định của serde cho unit variant là **tên biến thể** (`IoReadFailed`), không phải khoá chấm
  - [x] Khai `IpcError` với **đúng bốn trường, đúng chính tả**: `code` · `message_key` · `params` · `retryable`
  - [x] `params: BTreeMap<String, String>` — **BTree, không Hash**: thứ tự khoá ổn định thì test so JSON mới ổn định. **Giá trị là `String`**, kể cả số: định dạng số và ngày giờ chỉ ở frontend (`ARCHITECTURE-SPINE.md#Consistency Conventions` — *"Ngày giờ: lưu ISO-8601 UTC; định dạng hiển thị chỉ ở frontend"*, cùng nguyên tắc)
  - [x] ⛔ **`params` cũng không được mang văn bản hiển thị.** Một `params: {"reason": "Nhà cung cấp không phản hồi"}` là AD-21 bị thủng qua cửa sau. Tham số mang **dữ liệu** (đường dẫn, số đếm, tên nhà cung cấp), không mang **câu**
  - [x] `code`: định danh máy đọc, ổn định qua mọi lần sửa lời văn. ⚠️ `code` và `message_key` **được phép 1:1 hôm nay** nhưng **là hai trường, không phải một trường hai tên**: frontend rẽ nhánh trên `code`, hiển thị `message_key`. ⛔ `code` không bao giờ được đưa ra màn hình
  - [x] `retryable: bool` — chỉ là **quyền hiển thị một nút thử lại**. ⛔ Không mã nào được tự thử lại khi thấy `true`: AD-22 cấm auto-retry, và với BYOK nó là tính tiền hai lần
  - [x] Ba test trong `src-tauri/tests/` (xem §Testing standards để biết đặt ở đâu):
    - `ipc_error_wire_shape` — `serde_json::to_value(IpcError…)` có **đúng bốn khoá**, đúng chính tả `message_key` (không `messageKey`), `message_key` serialize thành **chuỗi khoá chấm**
    - `every_message_key_exists_in_vi_json` — đọc `../src/i18n/vi.json` (đường dẫn qua `CARGO_MANIFEST_DIR`, cùng khuôn `config_invariants.rs:11-19`), khẳng định **mọi** `MessageKey::ALL` có mặt. Chiều ngược lại **không** kiểm: `vi.json` có nhiều khoá chỉ frontend dùng, đó là bình thường
    - `vi_json_is_flat` — mọi giá trị là chuỗi, không object lồng *(hoặc để `check-i18n.mjs` gánh — chọn một, đừng làm cả hai)*

- [x] **Task 5 — Dời hai chuỗi chẩn đoán ra khỏi `App.vue`** (AC: 2)
  - [x] `src/App.vue:38` và `:54` chứa **văn bản tiếng Việt trong template literal** — cổng Task 6 sẽ đỏ ở chính hai dòng này. Đây là phát hiện thật, không phải giả định: xem §🔴 Phát hiện chặn
  - [x] Tạo `src/selftest/fallbackReport.ts` mang hai chuỗi đó, đúng khuôn tiền lệ `src/selftest/eventName.ts` — module bé, import tĩnh được vào bundle chính mà không kéo `scopeCheck.ts` theo
  - [x] ⛔ **Đừng** đưa hai chuỗi này vào `vi.json`. Chúng là chẩn đoán cho log CI, không phải chuỗi giao diện; `vi.json` là tài nguyên **hiển thị**. Trộn hai thứ là làm hỏng chính ranh giới story này dựng
  - [x] ⛔ **Đừng** import từ `./selftest/scopeCheck` — `App.vue:12-16` đã ghi rõ vì sao import tĩnh tệp đó là phá bất biến *"mã self-check không vào bundle release"*
  - [x] Sau khi dời: `src/App.vue` phải **sạch tiếng Việt ở vị trí mã**, chỉ còn tiếng Việt trong comment

- [x] **Task 6 — `scripts/check-i18n.mjs`: cổng có mã thoát** (AC: 1, 2, 5)
  - [x] Node thuần, `.mjs`, khuôn theo `scripts/check-deps.mjs` — kể cả `abort()` cho lỗi hạ tầng và **ngưỡng sàn** cho số tệp quét được
  - [x] ⚠️ **Node chứ không bash.** Ice đã chốt 2026-08-03 (`check-deps.mjs:22-24`): `npm run` trên Windows chạy qua `cmd.exe`, không có bash. Một cổng chỉ canh nửa số nền tảng thì không canh được NFR14
  - [x] Năm phép kiểm, xem §Khung `check-i18n.mjs` để biết chi tiết từng cái:
    - **Kiểm A** — không ký tự có dấu tiếng Việt ở **vị trí mã** của `src-tauri/src/**/*.rs` và `src/**/*.vue`
    - **Kiểm B** — `vi.json` phẳng; khoá khớp `^[a-z0-9]+(\.[a-z0-9_]+)+$`; không giá trị rỗng
    - **Kiểm C** — placeholder trong mọi giá trị khớp `\{[a-z_][a-z0-9_]*\}`; `{}` rỗng hoặc `{Ten}` hoa là FAIL
    - **Kiểm D** — giọng văn UX-DR47 phần máy chấm được: không `chúng tôi`, không `bạn` đứng thành tiếng riêng trong giá trị `vi.json`
    - **Kiểm E** — hành vi AC4: nạp `src/i18n/resolve.ts`, khẳng định ba đường — khoá có · khoá thiếu trả khoá nguyên văn không ném · tham số nội suy đúng
  - [x] ⛔ **NGƯỠNG SÀN, bắt buộc.** Quét được 0 tệp `.rs` hoặc 0 tệp `.vue` ⇒ `abort()`, **không** phải "đạt". Đây là bẫy số 2 mà `check-deps.mjs:15-17` đã đâm vào một lần: *"cây rỗng đọc thành sạch"*. Sàn hôm nay: **≥ 14 tệp `.rs`**, **≥ 1 tệp `.vue`**
  - [x] Danh sách miễn trừ viết **ngay trong script**, mỗi mục kèm **một câu lý do**. ⛔ Không miễn trừ im lặng bằng cách thu hẹp glob — xem §Ranh giới quét

- [x] **Task 7 — Chứng minh từng cổng bằng ĐỎ trước, XANH sau** (AC: 2, 4, 5)
  - [x] Với **mỗi** kiểm A–E: cố ý tạo một vi phạm → chạy → phải **đỏ**, và **đỏ đúng dòng đúng lý do** → gỡ vi phạm → phải **xanh**
  - [x] Vi phạm mẫu, mỗi kiểm một cái: A — thêm `const x = 'Đã lưu'` vào một `.vue`; B — lồng một object trong `vi.json`; C — đổi một placeholder thành `{Path}`; D — thêm `"Bạn hãy thử lại."`; E — sửa `resolve.ts` cho ném khi thiếu khoá
  - [x] Ghi bảng kết quả (kiểm · vi phạm · thông báo nhận được · mã thoát) vào §Debug Log References
  - [x] ⛔ **Một cổng chưa từng đỏ là một cổng chưa được chứng minh.** Story 1.3 §Task 11 và Story 1.4 §Task 3 đã đặt tiền lệ này; đừng phá

- [x] **Task 8 — Gắn MỘT bước vào pipeline đã có** (AC: 2)
  - [x] `package.json` → thêm `"check:i18n": "node scripts/check-i18n.mjs"`, đúng khuôn ba script đã có
  - [x] `.github/workflows/ci.yml` → thêm **một** bước `npm run check:i18n` trong job `check` đã có
  - [x] ⛔ **Không dựng workflow thứ hai.** AC4 của Story 1.3 cấm tường minh; khối *"CHỖ MÓC CHO EPIC SAU"* ở `ci.yml:420-435` là chỗ đã chừa sẵn
  - [x] Đặt bước **trước** `npm run build` (`ci.yml:100`), cạnh `check:deps` (`ci.yml:92`): nó chạy trong vài giây, không cần `dist/`, không cần cửa sổ đồ hoạ. Một chuỗi lọt vào nên đỏ **trước** khi tốn một lượt biên dịch Rust
  - [x] ⛔ **Đừng đặt nó xuống cụm cuối** nơi `check:scope` / `check:scope:bundled` đang đứng — hai bước đó cần webview, bước này thì không
  - [x] ⛔ **Đừng sắp xếp lại các bước đã có.** Thêm một bước, không mổ lại job
  - [x] ⚠️ **Story 1.4 cũng đang thêm một bước (`check:tokens`) vào đúng chỗ này và chưa dev xong.** Cái nào vào trước thì cái sau đặt bước của mình **kề bên**, không đụng bước kia. Nếu `check:tokens` chưa có mặt, **đừng thêm hộ**

- [x] **Task 9 — Đóng sổ: doc-comment, mục Deferred, README** (AC: 1, 2, 3)
  - [x] Sửa `src-tauri/src/core/i18n/mod.rs`: thay câu *"Hình dạng thật … là quyết định của Story 1.5"* bằng **câu trả lời**, kèm lý do chọn macro
  - [x] Sửa `src-tauri/src/commands/mod.rs:7-8` — doc-comment ở đó đã trỏ tới `core::i18n`; cập nhật cho khớp tên kiểu thật (`IpcError`, `MessageKey`)
  - [x] `deferred-work.md:19` → đánh dấu **đã đóng**, ghi cơ chế đóng nó (`scripts/check-i18n.mjs` + bước CI). ⛔ Đừng xoá dòng cũ — khuôn của tệp đó là gạch ngang rồi ghi kết quả, không phải xoá
  - [x] Tạo `src/i18n/README.md` theo khuôn năm README đã có ở `src/{commands,layout,modes,panels,tokens}/`: hình dạng phẳng · cách thêm khoá · miễn trừ · lệnh chạy cổng

---

## Review Findings

*Lượt code review 2026-08-04 · ba lớp song song (Blind Hunter · Edge Case Hunter · Acceptance Auditor), mọi phát hiện dưới đây đã được kiểm chứng lại bằng cách chạy cổng thật với ca vi phạm rồi khôi phục cây.*

### Quyết định của Ice — 2026-08-04, đã chốt trong lượt review

- [x] **[Review][Patch] Cổng đo *dấu tiếng Việt*, không đo *chuỗi hiển thị* — `deferred-work.md:19` đã đánh ✅ ĐÃ ĐÓNG trên cơ sở đó** — `scripts/check-i18n.mjs:77-79` phát hiện 134 ký tự có dấu. `<button>Xem</button>`, `<button>Save</button>`, `Dong`, `Trang` đều **xanh**. AC2 nói nguyên văn *"grep chuỗi tiếng Việt"* nên cài đặt đúng phát biểu — nhưng NFR16 mà script tự trích ở `:870` là *"chuỗi hiển thị sống ở `vi.json` và chỉ ở đó"*, rộng hơn hẳn thứ cổng làm được.
  → ✅ **Ice chốt: giữ nguyên Kiểm A, sửa `deferred-work.md:19` từ ✅ ĐÃ ĐÓNG thành ĐÓNG MỘT PHẦN**, ghi giới hạn *"chỉ bắt chuỗi CÓ DẤU; nhãn không dấu (`Xem`, `Dong`) và nhãn tiếng Anh lọt"* nằm cạnh giới hạn `.ts` đã ghi. ⛔ Không mở rộng phạm vi cổng trong story này — một phép kiểm cấu trúc `.vue` (text node phải là `{{ t('…') }}`) là phạm vi mới và sẽ báo thừa trên `App.vue` hiện tại.
- [x] **[Review][Patch] Không phép kiểm nào nối `params` phía Rust với placeholder trong `vi.json`** — `src-tauri/src/core/i18n/mod.rs:127-150` khai `IpcError` là struct trường công khai, không constructor. Một chỗ gọi ở Story 1.6 viết `IpcError { message_key: MessageKey::IoReadFailed, params: BTreeMap::new(), .. }` sẽ **xanh cả ba cổng** (`ipc_contract.rs:146-171` chỉ kiểm khoá **có mặt**; Kiểm C `:667-683` chỉ kiểm **hình dạng** placeholder) và người dùng đọc được nguyên văn *"Không đọc được tệp tại {path}"*. Đây đúng lớp "hỏng im lặng" story tuyên bố đóng.
  → ✅ **Ice chốt: constructor `IpcError::new()` cưỡng chế.** Khai bảng tham số bắt buộc **ngay trong `message_keys!`** (một khai báo duy nhất, không trôi được — cùng lý lẽ đã chọn macro cho `ALL`/`as_str`), đóng trường struct lại để chỉ dựng được qua `new()`, và thêm một test duyệt `MessageKey::ALL` đối chiếu bảng ấy với placeholder bóc từ `vi.json`. Đóng lỗ hổng ở **tầng kiểu**, không chỉ ở tầng test.

### Cần vá

- [x] [Review][Patch] `resolve.ts` chứa **byte NUL thô** → git phân loại là binary, toàn bộ nội dung tệp **vắng mặt khỏi mọi diff** (`Binary files /dev/null and b/src/i18n/resolve.ts differ`) và `grep`/`git grep` bỏ qua. Tệp mang ⛔ dày nhất của story lại là tệp không ai review được bằng diff. Sửa: dùng `\u0000` viết bằng escape (hoặc bất kỳ ký tự ASCII nào) làm dấu phân cách khoá dedupe — tương đương từng byte về hành vi [`src/i18n/resolve.ts:94`]
- [x] [Review][Patch] Kiểm A: một `char` literal Rust chứa `"` (ví dụ `matches!(c, '"')`) mở một string ma; sau đó `"/*"` trong một string thật bị đọc thành block comment không bao giờ đóng ⇒ **vi phạm AC2 thật ở phần còn lại của tệp lọt hoàn toàn**. Đã dựng lại: chèn `let q = '"'; let s = "/*"; let msg = "Đã lưu";` ⇒ cổng **exit 0**; đối chứng chỉ có `"Đã lưu"` ⇒ exit 1. Comment `:229-231` lập luận không cần theo dõi `'…'` vì char literal *"cùng một phán quyết"* với mã — đúng, nhưng chỉ khi máy **bỏ qua** nó, mà máy hiện đang để nó mở string [`scripts/check-i18n.mjs:229-231,269-273`]
- [x] [Review][Patch] Kiểm A: `scanScript` không có trạng thái regex literal ⇒ `/^https?:\/\//` mở một line comment giả và che nốt dòng. Đã dựng lại: `const _re = /^https?:\/\//; const _nhan = 'Đã lưu'` ⇒ **exit 0**. Đây đúng chiều báo sót mà header `:51-55` tuyên bố không thể xảy ra [`scripts/check-i18n.mjs:372-376`]
- [x] [Review][Patch] Kiểm A: `<!--` xuất hiện ở **bất kỳ đâu** trong template — kể cả trong giá trị attribute — làm mù phần còn lại của vùng; và `text.indexOf('-->', i + 4)` **không bị chặn bởi `to`** nên một `-->` trong `<script>` phía sau cũng nuốt được text node. Đã dựng lại: `<div title="a <!-- b"></div>` + `<button>Lưu</button>` ⇒ **exit 0**; đối chứng không có attribute ⇒ exit 1 [`scripts/check-i18n.mjs:520-523`]
- [x] [Review][Patch] Kiểm A và Kiểm D chỉ nhận dạng tiếng Việt **dựng sẵn (NFC)**; văn bản NFD (dán từ nguồn chuẩn hoá kiểu macOS) lọt cả hai. Đã dựng lại: `<button>Lưu</button>` với `ư` = `u`+U+031B ⇒ **exit 0**; bản NFC ⇒ exit 1. Sửa: `.normalize('NFC')` trên nội dung tệp trước khi quét và trên giá trị `vi.json` trước khi so ở Kiểm D [`scripts/check-i18n.mjs:77-79,698`]
- [x] [Review][Patch] Kiểm E dựng resolver trên một catalog **giả** tự viết ở `:810-814`; `vi.json` thật, `src/i18n/index.ts`, `t` và `tError` **không được nạp bởi bất cứ test hay cổng nào**. Thay `index.ts:42` bằng `export const t = (k) => k` thì mọi cổng vẫn xanh — nghĩa là AC1 (*"chuỗi phân giải từ `vi.json`"*) không có bằng chứng thực thi ở đâu cả, và `tError` — hàm được thêm riêng để chịu payload không tin được — có **không** assert nào. Sửa: Kiểm E nạp thêm `vi.json` thật và khẳng định hai khoá mồi phân giải đúng + `err.io.read_failed` nội suy `{path}` [`scripts/check-i18n.mjs:754,810-814` · `src/i18n/index.ts:42,57`]
- [x] [Review][Patch] `resolve.ts` nội suy `null` / `undefined` / số thẳng ra câu — vi phạm chính ⛔ doc-comment của nó ở `:61-63`. `has(params, name)` đúng nên nhánh "tham số thiếu" bị bỏ qua. Đã chạy thật: `t('err.io.read_failed', {path: null})` ⇒ *"Không đọc được tệp tại **null** — nội dung chưa được nạp."*; `{path: undefined}` ⇒ *"… tại **undefined** …"*. Sửa: kiểm `typeof params[name] === 'string'`, không phải chỉ `has()` [`src/i18n/resolve.ts:89,99`]
- [x] [Review][Patch] `t()` trả về **không phải chuỗi** khi `key` không phải chuỗi: `t(undefined)` ⇒ `undefined`, `t(1)` ⇒ số `1`, trong khi `Translate` khai `=> string`. `tError` tự phòng ở `:58` nhưng `t` xuất khẩu trực tiếp thì không [`src/i18n/resolve.ts:81-84`]
- [x] [Review][Patch] `tError` cảnh báo **mỗi lần gọi** khi `message_key` thiếu — không dedupe, đúng lũ log mà `resolve.ts:66-73` dựng hai `Set` để chặn. Một lỗi lặp qua mỗi lượt render sẽ ngập console [`src/i18n/index.ts:57-61`]
- [x] [Review][Patch] Kiểm C: phép đếm ngoặc cân bằng qua được ca đảo và ca kép. `"Xong } roi {"` ⇒ 1 `{` · 1 `}` cân bằng, `matchAll` không khớp gì ⇒ **xanh**, ngoặc thô ra màn hình. `"{{path}}"` ⇒ cân bằng, khớp `{path}` bên trong ⇒ xanh, rồi `resolve.ts` in ra `{/tmp/a.txt}` với ngoặc thừa [`scripts/check-i18n.mjs:669-682`]
- [x] [Review][Patch] Kiểm D so cụm bị cấm bằng `indexOf` chuỗi liền ⇒ mọi biến thể khoảng trắng lọt: `"Chúng  tôi không đọc được tệp."` (hai dấu cách, hoặc xuống dòng giữa hai tiếng) ⇒ **xanh**. Sửa: chuẩn hoá khoảng trắng trước khi so, hoặc dùng regex `chúng\s+tôi` [`scripts/check-i18n.mjs:698,707-717`]
- [x] [Review][Patch] Trùng khoá trong `vi.json` bị **nuốt im lặng ở cả hai phía**: `JSON.parse` giữ lần xuất hiện cuối, `serde_json` vào `BTreeMap` cũng vậy. `{"err.unknown":"A", …, "err.unknown":"B"}` ⇒ một chuỗi đã soạn và đã duyệt biến mất, hai cổng đều xanh. Phía Rust đã có `message_key_catalog_has_no_duplicate_keys`; phía `vi.json` thì không. `catalogRaw` đã sẵn ở `:612` để làm phép kiểm này [`scripts/check-i18n.mjs:609-613`]
- [x] [Review][Patch] Doc-comment `fallbackReport.ts` nêu **sai lý do** hai chuỗi được an toàn: nó nói chúng nằm sau *"miễn trừ CÓ TÊN"*. Thật ra Kiểm A chỉ quét `.rs` và `.vue`, `src/selftest/**` khớp **0 tệp** và chính cổng in ra con số đó mỗi lượt. Chúng an toàn vì là `.ts` — một lỗ phạm vi, không phải một miễn trừ đã duyệt. Comment này dạy người đọc sau một mô hình sai, đúng lúc repo vừa có một ví dụ mẫu về cách chuyển chuỗi từ `.vue` sang `.ts` để cổng xanh [`src/selftest/fallbackReport.ts:16-19`]
- [x] [Review][Patch] `sprint-status.yaml` `last_updated` bị đẩy **lùi 90 phút** (`2026-08-04T01:30:00` → `T00:00:00`) ở cả header comment lẫn khối dữ liệu — ghi đè một giá trị mới hơn do story trước viết. Task 9 không yêu cầu điều này [`_bmad-output/implementation-artifacts/sprint-status.yaml:2,44`]
- [x] [Review][Patch] `README.md` dẫn đầu bằng `import { t, tError } from '@/i18n'` nhưng **không có alias `@`** ở `vite.config.ts` hay `tsconfig.json` (đã grep: không `alias`, không `paths`). Cùng tệp, `:85` đặt tiêu đề *"Ba thứ sẽ hỏng im lặng nếu **bạn** không biết"* — chính đại từ Kiểm D cấm, trong tài liệu dạy quy tắc vô nhân xưng [`src/i18n/README.md:17,85`]
- [x] [Review][Patch] Kiểm B và test Rust cưỡng chế **hai văn phạm khoá khác nhau**: `^[a-z0-9]+(\.[a-z0-9_]+)+$` so với `contains('.')` + lọc ký tự. `err_io.read_failed` qua được phía Rust và đỏ ở Kiểm B — trong khi doc-comment Rust `:192-193` tuyên bố áp *"đúng luật mà Kiểm B áp"* [`src-tauri/tests/ipc_contract.rs:192-203` · `scripts/check-i18n.mjs:624`]
- [x] [Review][Patch] Năm chỗ tài liệu lệch với mã, gom một mục: (1) `deferred-work.md` — giới hạn `.ts` là một dòng `⚠️` **lồng dưới mục đã gạch ngang + ✅ ĐÃ ĐÓNG**, không phải "một mục mới" như Change Log tuyên bố; người rà mục còn mở sẽ bỏ qua cả khối. (2) Comment cạnh `RS_FLOOR` nói *"17 tệp `.rs` (sau miễn trừ)"*, số thật là **18** — và đó đúng là chỗ người sửa sàn sẽ đọc. (3) Change Log nói *"tám tệp sửa"*, File List liệt kê **chín**. (4) Completion Notes `:588` nói *"đã chọn Kiểm B… không viết test thứ ba trùng lặp"*, nhưng doc-comment `read_vi_json` tự viết *"cưỡng chế ở **cả hai phía**"* và ca R5 xác nhận `cargo test` cũng đỏ khi `vi.json` lồng — hai tài liệu do cùng một lượt viết ra mâu thuẫn nhau. (5) `tError(err, params?)` có tham số thứ hai mà Task 2 không nêu và §"Ba việc ngoài danh sách" không khai [`deferred-work.md:22-24` · `check-i18n.mjs:202` · story `:588,679` · `src/i18n/index.ts:59`]

### Hoãn — đã ghi vào `deferred-work.md`

- [x] [Review][Defer] Tệp nguồn tới qua symlink bị loại khỏi Kiểm A và **không tính vào sàn** — chỉ in ra như một dòng `detail`, không bao giờ là `fail` [`scripts/check-i18n.mjs:162-165,603`] — hoãn, chưa có symlink nào trong cây
- [x] [Review][Defer] Gốc quét cứng ở `src/` và `src-tauri/`; một `packages/`, `examples/` hay `e2e/` về sau vô hình với cổng mà sàn vẫn qua [`scripts/check-i18n.mjs:179-180`] — hoãn, chưa có thư mục nào ngoài hai gốc
- [x] [Review][Defer] `scanStyle` không có trạng thái `line_comment` ⇒ một comment `//` trong `<style lang="scss">` sẽ bị báo là vi phạm [`scripts/check-i18n.mjs:455-499`] — hoãn, chưa có `.scss` nào
- [x] [Review][Defer] Sàn đếm **tệp**, không đếm **nội dung**: một `App.vue` chỉ có khoảng trắng vẫn thoả `VUE_FLOOR = 1` và Kiểm A báo OK [`scripts/check-i18n.mjs:207-218`] — hoãn, mở lại nếu cây `.vue` phình lên
- [x] [Review][Defer] Assert *"không ký tự có dấu nào trên dây"* chạy trên chính fixture mà test tự dựng ở `:73-78`, nên nó chỉ đỏ khi ai đó sửa fixture — không quan sát đường sản phẩm nào (chưa có đường nào) [`src-tauri/tests/ipc_contract.rs:128-137`] — hoãn tới Story 1.6, khi command thật đầu tiên cho một đường thật để quan sát
- [x] [Review][Defer] `process.exit()` ngay sau `console.log` có thể cắt cụt phần chẩn đoán `file:dòng:cột` khi stdout là pipe trên Windows — mã thoát vẫn đúng, phần làm cổng dùng được thì mất [`scripts/check-i18n.mjs:873,876`] — hoãn tới lượt chạy runner thật của Story 1.3

### Đã loại (không phải phát hiện)

- `catalogRaw` bị gọi là "biến chết" — sai, nó được đọc ngay ở `:613` (`JSON.parse(catalogRaw)`).
- `tError` biến một hợp đồng gãy thành thông báo chung thay vì hỏng ồn ào — đúng thiết kế khoá dự phòng của AD-21, và doc-comment `index.ts:50-56` đã ghi rõ chủ ý.

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

Amelia (dev agent) · claude-opus-5 · 2026-08-04
`baseline_commit = dd2df66` (không phải `0255163` như lúc dựng story — xem §Đường cơ sở)

---

### Debug Log References

#### Task 1 — Đường cơ sở, đo trên `HEAD = dd2df66`

⚠️ **`HEAD` đã trôi hai commit so với lúc dựng story** (`0255163` → `614eb64` Story 1.3 → `dd2df66` Story 1.4). Đối chiếu với §Trạng thái repo hiện tại và ghi từng chỗ lệch thay vì dừng — cả ba chỗ lệch đều truy được về hai commit đó, và **không chỗ nào đổi phán quyết của §🔴 Phát hiện chặn**:

| Nhóm | Story ghi | Đo được | Phán quyết |
|---|---|---|---|
| `src-tauri/src/**/*.rs` | 21 dòng có dấu | **94 dòng / 17 tệp** | ✅ sạch — **0 dòng ngoài comment** (kiểm bằng lọc dòng không bắt đầu bằng `//`) |
| `src-tauri/tests/config_invariants.rs` | 183 | **183** ✅ khớp | ⚠️ miễn trừ có tên |
| `src/App.vue` | 26 | **28** | 🔴 vi phạm thật ở `:38` và `:54` |
| `src/selftest/scopeCheck.ts` | 124 | **124** ✅ khớp | ngoài phạm vi AC2 |
| Số tệp `.rs` trong `src-tauri/src/**` | 17 | **17** ✅ | — |
| Số tệp `.vue` trong `src/**` | 1 | **1** ✅ | — |

**Ba chỗ lệch, ba lý do:**

1. **`94` vs `21`** — `21` là số dòng của **riêng `lib.rs`**, không phải của cả cây (`lib.rs` đo lại hôm nay: đúng 21). Số của story là một phép đo hẹp hơn nhãn của nó, không phải một thay đổi trong cây nguồn. Mệnh đề quan trọng — *"chỉ trong comment, không một string literal nào"* — **đã kiểm lại và vẫn đúng**.
2. **App.vue `26` → `28`** — Story 1.4 thêm một comment hai dòng trong `<style>` (`:74-75`, giải thích `--face-ui-mono`). Comment, không phải vi phạm.
3. **Số tệp `.rs` cổng thật sự quét là 20, không phải 17** — cổng quét cả `src-tauri/**` (AC2 nói `.rs`, không nói *"`.rs` dưới `src/`"*): 17 tệp `src/` + `build.rs` + 2 tệp `tests/`. Sau miễn trừ `tests/**` còn **18**, trên sàn 14.

#### Task 7 — Nghiệm thu đỏ-rồi-xanh, cổng `check-i18n.mjs`

Harness tự khôi phục tệp trong `finally`, và chạy lại cổng sau toàn bộ để chứng minh cây sạch trở lại (**exit 0** ✅).

| Ca | Vi phạm | Thông báo nhận được | Exit |
|---|---|---|---|
| A1 | `const nhan = 'Đã lưu'` trong `<script>` của `.vue` | `src/App.vue:23:15 — chuỗi tiếng Việt ở vị trí mã (.vue)` | 1 ✅ |
| A2 | `<button>Lưu</button>` — **text node, không dấu nháy nào** | `src/App.vue:67:14 — …` | 1 ✅ |
| A3 | `pub const NHAN: &str = "Đã lưu";` | `…/core/i18n/mod.rs:134:25 — … (.rs)` | 1 ✅ |
| A4 | raw string `r#"Đã lưu"#` | `…/core/i18n/mod.rs:134:27 — … (.rs)` | 1 ✅ |
| **A5-âm** | comment có dấu + URL `"https://…"` có `//` trong string | *(không FAIL)* | **0** ✅ |
| **A6-âm** | block comment **LỒNG** `/* /* Đã */ vẫn trong: Đã */` | *(không FAIL)* | **0** ✅ |
| **A7-âm** | raw string chứa `//` và `"…"`, rồi comment có dấu sau đó | *(không FAIL)* | **0** ✅ |
| A8 | `content: "Đã lưu"` trong `<style>` của `.vue` | `src/App.vue:72:27 — …` | 1 ✅ |
| **A9-âm** | comment tiếng Việt ở **cả ba vùng** của `.vue` | *(không FAIL)* | **0** ✅ |
| A10 | `{{ 'Đã lưu' }}` trong template | `src/App.vue:67:15 — …` | 1 ✅ |
| B | object lồng trong `vi.json` | `` `lookup` có giá trị kiểu object — `vi.json` phải PHẲNG `` | 1 ✅ |
| C | `{path}` → `{Path}` | `` `err.io.read_failed` — placeholder `{Path}` ngoài dải `{ten_tham_so}` `` | 1 ✅ |
| D | thêm `"Bạn hãy thử lại."` | `` `err.retry` xưng hô: "bạn" — UX-DR47 đòi câu VÔ NHÂN XƯNG `` | 1 ✅ |
| E | `resolve.ts` **ném** khi thiếu khoá | `khoá thiếu làm `t()` NÉM — AC4 nói "không sập"` | 1 ✅ |
| E2 | `resolve.ts` trả `''` thay vì khoá | ``khoá thiếu trả về — nhận ``, phải là `khong.co` `` | 1 ✅ |
| E3 | bỏ dedupe cảnh báo | `khoá thiếu ghi cảnh báo LẶP LẠI` | 1 ✅ |

**16/16 đúng kỳ vọng.** Bốn ca `-âm` là đối chứng: chúng chứng minh cổng **không** bắt comment và **không** bị `//` trong chuỗi đánh lừa — nửa quan trọng bằng nửa kia, vì một cổng đỏ trên mọi comment sẽ bị gỡ trong tuần (`ci.yml:410-418`).

> 🔴 **Ca E tìm ra một khiếm khuyết thật của cổng, đã sửa trong lượt này.** Bản đầu bọc cả Kiểm E trong một `try` với `abort()` ở `catch`: đúng ca *"`resolve.ts` ném"* — tức vi phạm AC4 mà Kiểm E tồn tại để bắt — cho ra hai dòng FAIL đúng rồi lời gọi `t()` **kế tiếp** ném ra ngoài và bị báo thành *"Kiểm E KHÔNG chạy được"*. Mã thoát vẫn 1 nên nó không lọt, nhưng log nói **hạ tầng hỏng** trong khi thứ hỏng là **sản phẩm**. Nay mọi lời gọi `t()` đi qua một helper `call()` bắt ném thành FAIL có tên; `abort()` chỉ còn đúng một đường tới được: `createResolver` ném ngay lúc **dựng** resolver.

#### Task 7 (tiếp) — Nghiệm thu đỏ-rồi-xanh, ba test Rust

| Ca | Vi phạm | Test đỏ | Exit |
|---|---|---|---|
| R1 | `#[serde(rename_all = "camelCase")]` trên `IpcError` | `ipc_error_wire_shape` @ `:86` *(assert bốn khoá)* | 101 ✅ |
| R2 | `#[derive(Serialize)]` trần trên `MessageKey` | `ipc_error_wire_shape` @ `:94` *(assert khoá chấm)* | 101 ✅ |
| R3 | thêm biến thể `Missing => "err.chua.co"` | `every_message_key_exists_in_vi_json` | 101 ✅ |
| R4 | hai biến thể trỏ cùng một khoá chấm | `message_key_catalog_has_no_duplicate_keys` | 101 ✅ |
| R5 | `vi.json` lồng object | `every_message_key_exists_in_vi_json` @ `:58` *(panic có thông báo)* | 101 ✅ |

Khôi phục xong: **exit 0** ✅. R1 và R2 là hai bẫy hỏng-im-lặng #1 và #2 của story, và chúng đỏ ở **hai dòng assert khác nhau** — nghĩa là hai phép kiểm phân biệt được hai lỗi, không phải một phép kiểm bắt bừa.

#### Lệnh nghiệm thu đầy đủ — chạy lần cuối trên cây sạch

```
npm run check:i18n     → 5/5 kiểm ĐẠT · exit 0
npm run build          → vue-tsc ×2 sạch · vite build 215ms
cargo test             → 15 (config_invariants) + 3 (ipc_contract) = 18 ĐẠT · 0 warning
npm run check:deps     → 13/13 ĐẠT · exit 0 (không phụ thuộc mới nào)
npm run check:tokens   → hồi quy sau khi sửa App.vue · exit 0
```

---

### Completion Notes List

#### AC1 — chuỗi phân giải từ `vi.json` theo khoá chấm ✅

`src/i18n/index.ts` là **chỗ duy nhất** chạm `vi.json`; nơi tiêu thụ chỉ thấy `t` và `tError`. Hình dạng phẳng bị cưỡng chế ở **hai phía**: Kiểm B nói bằng thông báo cho người sửa, còn `read_vi_json()` phía Rust deserialize vào `BTreeMap<String, String>` nên một object lồng gãy ngay ở kiểu. Đây là chỗ story cho phép chọn một trong hai (*"hoặc để `check-i18n.mjs` gánh — chọn một, đừng làm cả hai"*): **đã chọn Kiểm B làm phép kiểm CÓ TÊN**, và không viết một test `vi_json_is_flat` riêng.

> ⚠️ **Sửa lại sau lượt code review 2026-08-04.** Câu trên từng viết là *"tính phẳng phía Rust là hệ quả của kiểu đích, không phải một test thứ ba trùng lặp"* — mà doc-comment của `read_vi_json` trong cùng lượt lại viết *"cưỡng chế ở **cả hai phía**"*, và ca R5 xác nhận `cargo test` cũng đỏ khi `vi.json` lồng. Hai tài liệu do cùng một lượt viết ra mâu thuẫn nhau. **Phát biểu đúng:** không có test `vi_json_is_flat` nào tồn tại, nhưng tính phẳng **vẫn** được cưỡng chế ở cả hai phía — Kiểm B bằng một phép kiểm có tên và có thông báo, phía Rust bằng kiểu đích `BTreeMap<String, String>`. ⛔ Của story (*"chọn một, đừng làm cả hai"*) nhắm vào việc viết **hai phép kiểm có tên** cho cùng một bất biến; một hệ quả của kiểu không phải một phép kiểm thứ hai, nhưng nó cũng không phải "không có gì", và bảo là không có gì thì sai.

#### AC2 — grep không ra kết quả ✅, cưỡng chế bằng mã thoát

Hai vi phạm thật duy nhất trong cây (`App.vue:38,54`) đã dời sang `src/selftest/fallbackReport.ts`. Chuỗi giữ **nguyên văn từng ký tự** — `scripts/check-scope.mjs` và `check-scope-bundled.mjs` đọc dòng `VERDICT:` trong đó, đổi khuôn là làm hai cổng của Story 1.2/1.3 mù.

⚠️ **Một quyết định mạnh hơn khung của story, ghi thẳng ra để lượt rà soát phân xử:** nội dung trong `{{ }}` của template **bị quét**, không được cho qua. §Khung liệt kê `{{ }}` cùng nhóm với *"khoảng trắng, chữ số, dấu câu ASCII"* — nhóm *không phải ký tự có dấu*, nên ở mọi ca lành cho qua hay không là như nhau. Ở ca hỏng thì khác hẳn: `{{ 'Đã lưu' }}` và `const x = 'Đã lưu'` là **cùng một vi phạm trong cùng một tệp**, và một cổng bắt cái sau mà tha cái trước là một lỗ hổng mời gọi. Ca A10 chứng minh nó đỏ.

#### AC3 — bốn trường, không văn bản hiển thị ✅

`message_key` có kiểu `MessageKey` chứ không phải `String`, nên **một khoá ngoài danh mục không biên dịch được**. `ipc_error_wire_shape` so `to_value(...).keys()` với đúng bốn chuỗi, cộng một assert *"không ký tự có dấu tiếng Việt nào trên dây"* — mệnh đề trung tâm của AD-21, kiểm được bằng máy vì chuỗi hiển thị của dự án là tiếng Việt có dấu.

> **Nghiệm thu AC3 khi chưa có `#[tauri::command]` nào — mệnh đề ghi vào story theo yêu cầu §Nghiệm thu AC3.** Tauri v2 đưa giá trị trả về của `#[tauri::command]` qua IPC bằng **chính `serde_json`**, không có tầng biến đổi nào chen giữa. Phiên bản đã kiểm: **`tauri = 2.11.5`** (ghim `=` ở `Cargo.toml`). `serde_json::to_value(IpcError…)` cho ra **đúng byte** frontend sẽ nhận, nên test serialize là bằng chứng về dây chứ không phải mô phỏng. **Không dựng `#[tauri::command]` giả** — nó là mã sản phẩm không ai gọi, chạy nó cần webview + một lượt biên dịch profile `dev` riêng (đắt nhất trên macOS ×10), và vòng chạy thật đến miễn phí ở Story 1.6.

#### AC4 — khoá thiếu hiện nguyên văn, ghi cảnh báo, không sập ✅

Nghiệm thu bằng **Kiểm E gọi hàm thật**, bảy mệnh đề, cả hai chiều. Dedupe dùng **hai `Set` riêng** (khoá thiếu · tham số thiếu): gộp một `Set` thì lỗi sau che lỗi trước. Tham số thiếu **giữ nguyên placeholder**, không thay `undefined` — `"Không đọc được tệp tại undefined."` là một câu hoàn chỉnh về ngữ pháp và sẽ đi thẳng ra màn hình, còn `{path}` còn nguyên thì ai nhìn cũng biết là lỗi lập trình.

`tError()` xử thêm một ca mà story không nêu nhưng payload từ bên kia ranh giới IPC bắt buộc phải chịu được: `message_key` **vắng mặt hoặc rỗng** ⇒ rơi về `err.unknown` + cảnh báo, **không ném**. Đó chính là việc mà khoá dự phòng cuối cùng của AD-21 tồn tại để làm.

#### AC5 — giọng văn UX-DR47 ✅

**Hai chuỗi mồi, bản cuối — nghiệm thu ba quy tắc "mắt" theo yêu cầu §Task 3:**

| Khoá | Chuỗi | Vì sao chọn chữ đó |
|---|---|---|
| `err.unknown` | *"Thao tác không hoàn tất vì một lỗi chưa được phân loại."* | **Nêu hệ quả trước, nguyên nhân sau** — người dịch cần biết *việc của họ có xong không* trước khi biết vì sao. **Vô nhân xưng**: không chủ ngữ người. **Không đổ lỗi**: *"chưa được phân loại"* đặt thiếu sót ở phía phần mềm, không ở phía người dùng. **Không cảm xúc**: không *"rất tiếc"*, không *"đã có lỗi xảy ra!"* |
| `err.io.read_failed` | *"Không đọc được tệp tại {path} — nội dung chưa được nạp."* | Story gợi hình dạng *"Không đọc được tệp tại {path}."*; **đã thêm vế hệ quả** vì quy tắc 2 đòi *nêu hệ quả, không chỉ nêu sự kiện* — vế đầu là sự kiện, vế sau nói cho người đọc biết trạng thái nào đang đúng. `{path}` là **dữ liệu**, không phải câu, và nó chứng minh đường nội suy chạy thật từ Rust qua dây tới `createResolver`. ⛔ Không viết *"Bạn đã chọn một tệp không đọc được."* |

Đúng **hai** khoá, và đó là một quyết định — story sở hữu **cơ chế**, không sở hữu **từ vựng**.

#### Ba câu hỏi cho Ice — đã đi theo mặc định của story, chưa có câu trả lời

1. **Miễn trừ `src-tauri/tests/**` khỏi Kiểm A** → **đã miễn trừ**, khai tường minh trong `EXEMPT` kèm một câu lý do, và cổng **in ra số tệp miễn trừ ở mỗi lượt chạy** (hôm nay: 2). Tiền lệ: quyết định #3 của Ice ở Story 1.3.
2. **`code` và `message_key` 1:1 ở v1** → **giữ 1:1**, hai trường riêng, và doc-comment của `IpcError` ghi rõ chúng được phép rời nhau về sau mà không phải đổi hợp đồng.
3. **Hai chuỗi mồi** → **đúng hai**.

#### Ba việc đã làm ngoài danh sách subtask — và vì sao

Cả ba đều là hệ quả trực tiếp của một lượt nghiệm thu chứ không phải phạm vi mới:

1. **`call()` helper trong Kiểm E** — sửa khiếm khuyết mà ca E phát hiện (xem §Debug Log).
2. **`message_key_catalog_has_no_duplicate_keys`** thay cho `vi_json_is_flat` trong bộ ba test Rust. Story cho phép để Kiểm B gánh tính phẳng; chỗ trống đó dùng cho một lỗ hổng mà **không** phép kiểm nào khác chạm tới: hai biến thể `MessageKey` trỏ cùng một khoá chấm. Hậu quả của nó đúng bằng hậu quả của một khoá thiếu — một trong hai lỗi sẽ hiện ra câu của lỗi kia — và cả `ALL`-vs-`vi.json` lẫn Kiểm B đều xanh với nó. Ca R4 chứng minh.
3. **`tError` có tham số thứ hai `params?`** mà Task 2 không nêu (*"Thêm `tError(err: IpcError): string`"*). Nó phục vụ chỗ gọi cần nội suy dữ liệu mà payload không mang, và vì `t` đã là hàm công khai nên nó không mở thêm quyền gì. *(Mục này thêm vào sau lượt code review 2026-08-04 — bản đầu để nó lặng lẽ.)*
4. **Một dòng bổ sung vào khối *"CHỖ MÓC CHO EPIC SAU"* của `ci.yml`** — khối đó liệt kê ba luật *đã biết lúc viết*; ghi rằng Story 1.5 gắn thêm một luật ngoài danh sách để sổ vẫn là sổ đầy đủ. ⛔ Không sắp xếp lại bước nào, không dựng workflow thứ hai.

#### Giới hạn ghi thẳng, KHÔNG đánh dấu đạt

⚠️ **Phạm vi Kiểm A là `.rs` và `.vue` — đúng phát biểu AC2 và Story 10.9, và không hơn.** Tệp `.ts` không bị cổng nào canh, mà `src/tokens/fonts.ts` (59 dòng có dấu), `src/selftest/scopeCheck.ts` (124) và cả `src/i18n/resolve.ts` (thông báo `console.warn`) đều mang chuỗi tiếng Việt ở vị trí mã. Hôm nay tất cả đều là **chẩn đoán/log**, không phải chuỗi hiển thị — nhưng mệnh đề đó do **người đọc** giữ, không do máy giữ. Vế thật sự rủi ro là component, nằm ở `.vue`, và đã được canh. Đã ghi thành một mục ở `deferred-work.md` với điều kiện mở lại: **ngày đầu tiên một chuỗi hiển thị xuất hiện trong `.ts`**.

⚠️ **Miễn trừ `src/selftest/**` hôm nay khớp 0 tệp** (thư mục chỉ có `.ts`) và con số đó được **in ra có chủ ý** thay vì gỡ mục đi — nó khai trước cho ngày một `.vue` chẩn đoán xuất hiện ở đó, và giữ lý do nằm cạnh chỗ cưỡng chế.

⚠️ **Chưa chạy trên runner nào.** Mọi số ở trên đo trên macOS 26 / Node v22.22.2. Kiểm E dựa vào Node ≥ 22.18 bóc kiểu TypeScript mặc định; CI khai `node-version: '22'`, nên đường này *phải* chạy — nhưng *phải* chưa phải *đã*. ⛔ Nếu runner có Node 22.x < 22.18 thì Kiểm E `abort()` với exit 1 kèm thông báo nêu đích danh phiên bản đang chạy — **không** im lặng bỏ qua. Story 1.3 vẫn `in-progress` chờ một lượt runner thật; bước này sẽ được xác nhận trong cùng lượt đó.

---

### File List

**Mới (6):**

| Tệp | Vai |
|---|---|
| `src/i18n/resolve.ts` | Hàm phân giải thuần — `createResolver` → `t(key, params?)`. ⛔ Không `import` gì |
| `src/i18n/index.ts` | Chỗ duy nhất chạm `vi.json`; export `t`, `tError`, kiểu `IpcError` |
| `src/i18n/README.md` | Khuôn theo năm README đã có ở `src/*/` |
| `src/selftest/fallbackReport.ts` | Hai chuỗi chẩn đoán dời khỏi `App.vue` (Task 5) |
| `scripts/check-i18n.mjs` | Cổng năm phép kiểm, mã thoát là phán quyết |
| `src-tauri/tests/ipc_contract.rs` | Ba test hợp đồng dây AD-21 |

**Sửa (9):**

| Tệp | Sửa gì |
|---|---|
| `src/i18n/vi.json` | `{}` → object phẳng, hai khoá mồi |
| `src/App.vue` | Dời hai chuỗi chẩn đoán ra `fallbackReport.ts`; thêm một `import` tĩnh |
| `src-tauri/src/core/i18n/mod.rs` | Doc-comment giao việc → **câu trả lời**; `message_keys!`, `MessageKey`, `Serialize` viết tay, `IpcError` |
| `src-tauri/src/commands/mod.rs` | Doc-comment `:7-8` khớp tên kiểu thật (`IpcError`, `MessageKey`) |
| `package.json` | Thêm đúng một dòng `scripts`: `check:i18n` |
| `.github/workflows/ci.yml` | Thêm **một** bước trong job `check` đã có + một dòng vào sổ *"CHỖ MÓC CHO EPIC SAU"* |
| `_bmad-output/implementation-artifacts/deferred-work.md` | `:19` đánh dấu **đã đóng**, ghi cơ chế đóng và giới hạn còn lại |
| `_bmad-output/implementation-artifacts/sprint-status.yaml` | `1-5-…` → `in-progress` → `review` |
| `_bmad-output/implementation-artifacts/1-5-…-qua-ipc.md` | Chính tệp này — frontmatter `baseline_commit`, 60 checkbox, Dev Agent Record, File List, Change Log, Status |

⛔ **Không đụng tới** (đúng §Ranh giới phạm vi): `src-tauri/tauri.conf.json` · `Cargo.toml` · `src/selftest/scopeCheck.ts` · `_bmad-output/planning-artifacts/**`. **Không thêm một phụ thuộc nào** — `check:deps` xanh sau khi xong, bảng Stack 19 hàng không đổi.

---

## Change Log

| Ngày | Ai | Gì |
|---|---|---|
| 2026-08-03 | Bob (SM) | Dựng story từ `epics.md#Story 1.5`, ARCHITECTURE-SPINE (AD-21, Consistency Conventions, Cây nguồn), EXPERIENCE.md (UX-DR47), trạng thái repo `HEAD = 0255163`, và bàn giao từ Story 1.2/1.3/1.4. Ghi §🔴 Phát hiện chặn — ba nhóm vi phạm hiện có, ba phán quyết khác nhau. |
| 2026-08-04 | Code review (3 lớp song song) | **19 patch đã áp, 2 quyết định của Ice đã chốt, 6 mục hoãn.** Bốn lỗ thủng của Kiểm A đóng lại — char literal Rust chứa `"` · regex literal JS · `<!--` trong attribute · NFD; cả bốn đã dựng lại được ở dạng **exit 0** trước khi sửa. Thêm `IpcError::new()` cưỡng chế bảng tham số ở tầng kiểu (trường struct thành riêng tư), Kiểm E nay chạy trên `vi.json` THẬT, `resolve.ts` chặn tham số/khoá sai kiểu, Kiểm B bắt khoá trùng, Kiểm C bắt ngoặc thừa, Kiểm D so theo `\s+`. Gỡ một byte NUL khỏi `resolve.ts` (git đang phân loại tệp là *binary*). `deferred-work.md:19` hạ từ ✅ ĐÃ ĐÓNG xuống 🟡 ĐÓNG MỘT PHẦN kèm hai giới hạn. Nghiệm thu **23 ca đỏ-rồi-xanh** (13 hỏng + 10 đối chứng âm); `check:i18n` · `build` · `cargo test` **20 ĐẠT** · `check:deps` · `check:tokens` đều exit 0. Status → `done`. |
| 2026-08-04 | Amelia (dev) | Cài đặt trọn chín task. Sáu tệp mới, **chín** tệp sửa (khớp §File List). Cổng `check-i18n.mjs` (5 kiểm) + 3 test Rust, gắn **một** bước vào `ci.yml`. Nghiệm thu đỏ-rồi-xanh **16/16 ca cổng** (gồm 4 đối chứng âm) và **5/5 ca test Rust**. Đóng `deferred-work.md:19`, ghi giới hạn `.ts` chưa được canh thành mục mới. Status → `review`. |
