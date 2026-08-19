---
baseline_commit: d55d4aeb0ed107ba0ee17b8e5cfcc0c167b283f9
---
# Story 2.5c: Cắt bỏ câu khỏi bản dịch

Status: done

**Covers:** FR133 · bước di trú **8
Epic:** 2 — Biên tập theo segment, một vòng dịch tay hoàn chỉnh
**Story trước:** 2.5b (Lưới hai cột đối chiếu) — `done` 2026-08-15
**Story sau:** 2.5d (Ngắt đoạn của bản dịch, bước di trú **9**)

---

## Story

As a **người dịch**,
I want **bỏ hẳn một câu hoặc một dải câu khỏi bản dịch mà vẫn thấy mình đã bỏ gì**,
so that **quyết định "đoạn này không thuộc bản dịch" không biến thành một lỗ hổng im lặng**.

---

## Acceptance Criteria

Chép nguyên văn từ `epics.md:2341-2372`. Số hiệu AC1–AC7 là của story này, dùng để tham chiếu trong Tasks.

**AC1 — phạm vi do người dùng chọn từng lần
Given **một câu hoặc** một dải câu** đang chọn
**When** gọi thao tác cắt bỏ
**Then** cờ đặt trên **câu nguồn
And **phạm vi do người dùng chọn** từng lần**, không định trước

**AC2 — trục độc lập
Given** cờ cắt bỏ
**When** đọc
**Then** nó là một **trục độc lập** — câu **vẫn giữ** trạng thái riêng của nó trong bảng sáu giá trị
🔴 Đây **không** phải giá trị thứ bảy của bảng trạng thái. *"Cắt bỏ"* là quyết định về **thuộc hay không thuộc bản dịch**, không phải một mức độ hoàn thành — đúng khuôn `translate="no"` của XLIFF.

**AC3 — hiển thị trong lưới
Given** một câu đã cắt bỏ
**When** hiển thị trong lưới
**Then** hàng **vẫn nằm trong lưới**, **gạch ngang và mờ đi**

**AC4 — đảo ngược được
Given** một câu đã cắt bỏ
**When** người dùng bỏ cờ
**Then** câu quay về **đúng trạng thái cũ với nội dung cũ**

**AC5 — ẩn hoàn toàn ở đầu ra
Given **Chế độ đọc và** mọi** bản xuất
**When** render một câu đã cắt bỏ
**Then** **ẩn hoàn toàn** — không dấu vết, không `[…]`, không chỗ trống

**AC6 — điều hướng bỏ qua
Given** lệnh *"câu chưa dịch kế tiếp"
***When** gặp một câu đã cắt bỏ
**Then** **bỏ qua** nó

**AC7 — bước di trú 8
Given** lược đồ `project.db
`**When** thêm cột
**Then** một `ALTER TABLE` đánh số **8**, **không** sửa `SEGMENT_DDL` tại chỗ
🔴 Sửa `SEGMENT_DDL` cho ra **hai lược đồ khác nhau cho cùng một số phiên bản** — đúng vết sẹo số 4. Nguồn sự thật là `PROJECT_MIGRATIONS`, không phải một ghi chép ở nơi khác.

---

## 🔴 Quyết định mở — Ice chốt TRƯỚC khi viết dòng mã đầu tiên

Năm chỗ dưới đây có từ hai phương án hợp lệ và **đặc tả không chọn hộ**. Luật của dự án: *"Ice là người chốt các quyết định mở. Gặp một chỗ hai phương án đều hợp lệ: nêu cả hai kèm số đo, đừng tự chọn rồi đi tiếp — và cũng đừng loại một phương án chỉ vì nó đắt"* (`project-context.md:464-466`).

Dev agent **dừng ở đây** và trình năm quyết định. Không tự chọn.

### Quyết định #1 — "một dải câu" (AC1): năng lực chọn nhiều hàng **chưa tồn tại**

**Số đo:** `editorPanelState.ts:51` khai `caretSegmentId` là `Ref<number | null>` — **một số duy nhất, không mảng, không** `Set`. `setEditorCaret(id: number | null)` (`editorPanelState.ts:137-142`) nhận đúng một `id`. Mọi tương tác chuột trong `GridPanel.vue:384-457` đặt caret vào đúng một ô. Không có `selectedIds`/`Set<number>`/`{from,to}` ở bất kỳ đâu trong `GridPanel.vue` · `editorPanelState.ts` · `segmentNavigation.ts` · `selectionContract.ts`.

**Và đặc tả không mô tả cơ chế.** PRD (`prd.md:458`), `epics.md:130,2343`, `EXPERIENCE.md:132` đều chỉ nói *"phạm vi do người dùng chọn từng lần, không định trước"* — đó là mô tả **kết quả**, không phải **cơ chế**. Không tài liệu nào nói Shift+click, kéo chọn, hay Shift+mũi tên.

⚠️ **Và có một hàng rào từ story trước:** AC7 của Story 2.5b khai *"*`selectionContract.ts`* không sửa một dòng"*, vì `check-commands.mjs:1876-2113` Kiểm F đếm **tĩnh** số lời gọi `useSelectionSurface(...)` literal: sàn `SELECTION_SURFACE_FLOOR = 6` (`:1974`) và bảng theo tệp `SELECTION_PANEL_FILES` (`:1917-1921`, `GridPanel.vue: 2`). Lưu ý: `selectionContract.ts` phục vụ **chọn văn bản trong một cột** cho Auto-Lookup — nó **không phải** cơ chế chọn nhiều **hàng**. Hai khái niệm "vùng chọn" khác nhau trong cùng một lưới.

| Đường | Nội dung | Cái giá |
| --- | --- | --- |
| **(a)** | Dựng multi-select hàng trong story này: state mới (`anchorId`/`focusId` hoặc `Set<number>`) ở `editorPanelState.ts`, Shift+click + Shift+`↑`/`↓` | Một năng lực UI mới **không có đặc tả cơ chế**; phải tự thiết kế tương tác. Rủi ro va vào Kiểm F nếu cài nhầm qua `useSelectionSurface` |
| **(b)** | Story này làm **một câu** (câu đang có caret). "Dải câu" ghi nợ có chủ | AC1 đóng một nửa 🟡. Nhưng đóng **đúng** phần đã có nền, và không dựng một tương tác chưa ai đặc tả |
| **(c)** | "Dải" = hai lần gọi lệnh (đánh dấu đầu, rồi đánh dấu cuối) — không cần multi-select DOM | Rẻ, không state mới, không đụng Kiểm F. Nhưng là một tương tác lạ, chưa từng có tiền lệ trong app |

🔴 **Không đường nào là mặc định.** Nếu Ice chọn (b), phần còn hở ghi vào `deferred-work.md` kèm chủ — **không** sửa `epics.md` cho khớp mã (`project-context.md:456-458`).

### Quyết định #2 — AC5: cả hai bề mặt đầu ra **là khung rỗng**

**Số đo:**

- `src-tauri/src/core/export/mod.rs` — **7 dòng, toàn bộ là doc-comment, không một dòng mã**. `docx-rs = "=0.4.22"` khai ở `Cargo.toml:42` nhưng **chưa** `use` **ở đâu** (grep `docx_rs` trong `src-tauri/src/**/*.rs` chỉ trúng chính dòng comment).
- `src/modes/ReadingMode.vue` — 50 dòng, template chỉ có một `<p>` chở `t('mode.reading.status')`. Doc-comment tự ghi *"KHUNG RỖNG có chủ ý… toàn bộ thuộc Epic 5"*. `modeState.ts:30` xác nhận *"cả ba chế độ đều rỗng"*.

⚠️ **Và có một khoảng hở đặc tả thật, không phải chỗ mình đọc sót:** nghĩa vụ "ẩn hoàn toàn" chỉ được phát biểu **một chiều**, từ FR133 áp xuống. Các FR xuất bản (FR87 · FR88 · FR89 · FR121 · FR130 · FR131) và các Story của Epic 5 (5.11–5.13, Chế độ đọc) / Epic 8 (8.3 · 8.4 · 8.6, xuất bản) **không AC nào tham chiếu ngược lại FR133**. Nghĩa vụ tồn tại nhưng chưa có đường nghiệm thu ở phía tiêu thụ.

| Đường | Nội dung | Cái giá |
| --- | --- | --- |
| **(a)** | Chỉ lưu đúng cờ. Toàn bộ vế "ẩn" ghi nợ có chủ (Epic 5 · Epic 8) | Rẻ nhất. Nhưng nghĩa vụ đi vào sổ nợ và **phụ thuộc người sau đọc sổ** — đúng lớp lỗi mà khoảng hở đặc tả trên đã tạo ra một lần |
| **(b)** | Lưu cờ **và** dựng sẵn một **hàm thuần** ở Rust (`core/segment/`) trả về danh sách segment **đã lọc cờ**, kèm test hợp đồng khẳng định nó lọc. Hai bề mặt sau chỉ việc gọi | Thêm ~1 hàm + ~2 test. Đóng được vế "nghĩa vụ không có ai canh" ngay hôm nay, thay vì giao cho trí nhớ |

⚠️ Đường (b) **không** phải là "dựng Chế độ đọc" — nó chỉ dựng đúng cái chốt mà hai bề mặt kia sẽ cắm vào. AC5 vẫn **không** đóng được trọn ở story này dù chọn đường nào: không có bề mặt để nghiệm thu *"không dấu vết, không *`[…]`*, không chỗ trống"*. Phần đó là 🟡, ghi nợ có chủ. Đây là *"năng lực chưa dựng ≠ lệch spec"* (`project-context.md:456`), **không** phải cớ để tự chấm đạt.

### Quyết định #3 — cách GỌI thao tác: đặc tả **không nói**

Đã grep `EXPERIENCE.md` · `DESIGN.md` · `prd.md` · `epics.md` cho "chuột phải" · "context menu" · "right-click" · "phím tắt" — **không kết quả nào** mô tả cơ chế gọi lệnh cắt bỏ. AC1 chỉ nói *"When gọi thao tác cắt bỏ"*.

Ràng buộc đã cố định (không phải chỗ chọn): thao tác **bắt buộc** là một command đăng ký trong `CommandRegistry` (AD-34; `ARCHITECTURE-SPINE.md:689` — *"Không thao tác nào chỉ tồn tại trong một handler chuột"*), id theo văn phạm `^[a-z0-9]+(\.[a-z0-9_]+)+$` (`registry.ts:125`), tiền tố miền `editor.` đã chốt cho thao tác segment (`src/commands/index.ts:869-871`, Quyết định #4 Ice ký 2026-08-14).

| Đường | Nội dung |
| --- | --- |
| **(a)** | **Một** lệnh bập bênh `editor.toggle_omitted` — cắt bỏ và bỏ cờ cùng một phím |
| **(b)** | **Hai** lệnh `editor.omit_segment` + `editor.restore_segment` — hai phím, hai nhãn, trạng thái đọc được từ tên lệnh |

⚠️ Tiền lệ trong kho **không quyết hộ**: `editor.confirm_segment` là một chiều (bỏ xác nhận xảy ra **ngầm** khi sửa văn bản), nên nó không phải khuôn cho một cờ đảo ngược tường minh như AC4 đòi.
⚠️ Hợp âm phím phải chưa dùng — `src/commands/index.ts:880-883` kiểm trùng hợp âm **trên toàn registry**, không theo chế độ. Đã dùng: `Mod+Enter` (`editor.confirm_segment`), `Alt+ArrowDown` (`editor.next_untranslated`).

### Quyết định #4 — ô đã cắt bỏ có còn **gõ được** không?

AC không nói. Nhưng nó va vào một mệnh đề Story 2.5b vừa đạt được: `GridPanel.vue:981` đặt `contenteditable="true"` **tĩnh**, và chú thích `:962-964` ghi rõ *"Không một binding động nào: giá trị không đổi theo trạng thái, nên Vue không bao giờ vá thuộc tính này"*.

| Đường | Nội dung | Cái giá |
| --- | --- | --- |
| **(a)** | Vẫn gõ được. Cờ thuần thị giác + lọc đầu ra | Không đụng gì. Nhưng người dùng gõ vào một câu sẽ không bao giờ ra bản dịch — công bỏ đi im lặng |
| **(b)** | Chặn ở `onBeforeInput` (`GridPanel.vue:695-746`), giữ `contenteditable` **tĩnh** | Giữ nguyên mệnh đề của 2.5b. Caret vẫn đặt được (đọc/copy được), chỉ không sửa được |
| **(c)** | Gỡ `contenteditable` có điều kiện | **Phá** mệnh đề `:962-964` của 2.5b và mở lại đúng lớp lỗi WKWebView mà 2.5b tốn một bàn đo hai engine để đóng |

⚠️ Đường (c) là một quyết định 🔴, không phải một dòng mã — nếu chọn, phải nói rõ nó lật cái gì.

### Quyết định #5 — tên và kiểu cột

Từ vựng `omitted` đã có tiền lệ ở hai nơi: `DESIGN.md:148` khai token `grid-row-omitted`, và bản dựng thăm dò của 2.5b có `tr.row-omitted`. Dùng lại từ này, đừng đặt từ thứ hai.

| Đường | DDL | Tiền lệ trong kho |
| --- | --- | --- |
| **(a)** | `is_omitted INTEGER NOT NULL DEFAULT 0` | `is_paragraph_end INTEGER NOT NULL` (`schema.rs:341`) |
| **(b)** | `omitted_at TEXT` (NULL = không cắt bỏ) | `retired_at TEXT` (`schema.rs:342`) |

⚠️ AC4 đòi *"quay về đúng trạng thái cũ với nội dung cũ"* — cả hai đường đều thoả, vì cờ **không chạm** `status` lẫn `target_text`. Đường (b) chở thêm **khi nào**; đường (a) rẻ hơn một phép so sánh. Không đường nào sai.
🔴 Dù chọn gì: **không** `CHECK` trong DDL — giá trị hợp lệ cưỡng chế ở tầng Rust, đúng khuôn `status` và `chapter.status` (`schema.rs:400-402`).

---

## Tasks / Subtasks

> Task 0 chạy **trước** mọi task khác và **chặn** chúng.

- [x] **Task 0 — Trình năm quyết định mở cho Ice** (AC1, AC5, và ba chỗ hình dạng)
  - [x] 0.1 Trình Quyết định #1–#5 ở mục trên, kèm số đo đã ghi. Không tự chọn đường nào
  - [x] 0.2 Ghi chữ ký của Ice vào `§Dev Agent Record` kèm ngày
  - [x] 0.3 Nếu một chữ ký làm hẹp phạm vi (ví dụ #1 đường (b)): ghi phần còn hở vào `deferred-work.md` **kèm chủ**, ngay lúc ký. Không sửa `epics.md`
- [x] **Task 1 — Bước di trú 8** (AC7)
  - [x] 1.1 Thêm một hằng `SEGMENT_OMITTED_DDL` (tên theo Quyết định #5) trong `src-tauri/src/core/store/schema.rs`, cạnh `SEGMENT_STATUS_AND_VERSION_DDL`. **Một** câu `ALTER TABLE segment ADD COLUMN …` với `DEFAULT` hằng
  - [x] 1.2 Doc-comment theo đúng khuôn hai hằng trước: vì sao số **8**, vì sao `ALTER TABLE` chứ không sửa `SEGMENT_DDL`, giá trị backfill nghĩa là gì trên dữ liệu thật
  - [x] 1.3 Thêm `Migration { to_version: 8, sql: SEGMENT_OMITTED_DDL }` vào cuối `PROJECT_MIGRATIONS` (`schema.rs:524-555`)
  - [x] 1.4 Cập nhật test khai danh sách nguyên văn: `segment_contract.rs::the_project_migration_set_reaches_seven_through_six_steps` (`:492-500`) hiện khẳng định `vec![1,2,3,5,6,7]` — đổi cả **tên hàm** lẫn mảng. Tên hàm test là một **câu khẳng định**, phải nói đúng số mới
  - [x] 1.5 ⚠️ `a_project_database_newer_than_the_app_is_refused_and_never_written_to` (`:912-958`) dùng một fixture **giả** bước `to_version: 8` để mô phỏng "một bản app tương lai". Số 8 nay là số **thật** ⇒ fixture đó phải nâng lên 9, nếu không test mất ý nghĩa mà **vẫn xanh**
  - [x] 1.6 Rà hai hằng neo ở `pinned_contract.rs` (số bước · số phiên bản đích): 7/8 → 8/9
  - [x] 1.7 Thêm test backfill theo khuôn `a_project_database_at_version_six_migrates_to_seven_and_every_old_row_becomes_draft` (`:825`): db ở phiên bản 7 di trú lên 8, mọi hàng cũ nhận giá trị "không cắt bỏ", không hàng nào mất
  - [x] 1.8 *(ngoài kê hoạch, nhưng là hệ quả trực tiếp của 1.3 — ghi ra thay vì làm im lặng)* **Bốn** ca khác neo vào *"đích là 7"* đỏ ngay lượt chạy đầu; cộng `SegmentRow` 10 → **11** cột. Chi tiết ở `§Debug Log References`
- [x] **Task 2 — Cột mới đi qua dây IPC** (AC2, AC3, AC6)
  - [x] 2.1 Thêm trường vào struct `ChapterSegment` (`src-tauri/src/commands/segment.rs:154-163`). **Không** thêm `#[serde(rename_all)]` — tên trường trả về giữ `snake_case`
  - [x] 2.2 Thêm cột vào câu `SELECT` của `read_open_chapter_segments` (`segment.rs:316`) và phép đọc/ép kiểu (`:320-328`)
  - [x] 2.3 🔴 Thêm một test hợp đồng theo khuôn `the_load_command_carries_the_status_column_over_the_wire` (`segment_contract.rs:2005`). **Đây là cổng chống lại một lỗi ĐÃ XẢY RA:** bản đầu Story 2.5 quên `status` ở đúng hai chỗ 2.1/2.2 ⇒ `undefined` phía webview, **74/74 test frontend vẫn xanh** vì fixture chép tay có sẵn cột. Bắt được bởi e2e, không bởi vitest
  - [x] 2.4 Rà `the_raw_column_reader_sees_every_column_the_segment_table_actually_has` (`:1512`) — nó đọc thẳng cột SQL, phải biết cột mới *(làm ở Task 1.8: nó đỏ ngay lượt bước 8 vào, đúng vai)*
  - [x] 2.5 Thêm trường tương ứng vào type `ChapterSegment` phía TS (`src/config/segment.ts:66-89`), **snake\_case**, khớp đúng tên trên dây
  - [x] 2.6 *(hệ quả của 2.5)* **Ba** fixture chép tay phía frontend thiếu trường mới ⇒ `vue-tsc` đỏ. Đây là lần **kiểu** bắt được đúng lớp lỗi mà Story 2.5 phải nhờ e2e mới thấy — bắt được **chỉ vì** `tsconfig.json` include cây test
- [x] **Task 3 — Lệnh cắt bỏ / bỏ cờ** (AC1, AC4)
  - [x] 3.1 Hàm thuần Rust nhận `Option<&OpenWork>` + phạm vi segment, ghi cờ. Vỏ `#[tauri::command]` **mỏng** trong `pub mod wire`, lấy `State` qua `try_state`, không `state()`
  - [x] 3.2 🔴 Ghi **rời rạc**: một `open.store.write(|tx| …)`, một transaction — khuôn là `confirm_segment` (`segment.rs:692-757`). **Không** định tuyến qua bộ đệm gõ / `saveSegmentTargets`. Lý do ở `project-context.md:520-522`: một thao tác người dùng *thấy đã xong* nằm chờ tới 5 giây rồi biến mất nếu app sập
  - [x] 3.3 Từ chối có hình dạng: segment không tồn tại · segment đã về hưu. Dùng lại `SegmentNotFound` · `SegmentRetired` đang có (`core/i18n/mod.rs:209,220`). **Chỉ** thêm khoá `err.segment.*` mới nếu có một nhánh từ chối thật sự mới — mỗi khoá mới phải qua `message_keys!` (sinh đồng thời `enum` · `ALL` · `as_str` · bảng tham số) và có mặt trong `vi.json`
  - [x] 3.4 Đăng ký command trong `installCommands()` ở `src/commands/index.ts` — **chỉ ở đó**, không trong `App.vue` hay component (một lượt HMR sẽ gọi lần hai và `register()` ném vì id trùng)
  - [x] 3.5 Nhãn lệnh vào `vi.json` khoá `command.<id>`
  - [x] 3.6 Adapter IPC ở tầng TS **không bao giờ ném**: một `invoke`, một `try/catch`, trả hình dạng ba trạng thái `{ <giá trị> | null, error: IpcError | null }`
  - [x] 3.7 Cập nhật ảnh chụp hiển thị theo khuôn `confirmCurrentSegment` (`editorPanelState.ts:605-728`): dựng **mảng mới**, vì `shallowRef` không theo dõi sửa tại chỗ (`:704-709`)
- [x] **Task 4 — Hiển thị trong lưới** (AC3) — *mọi mục trừ 4.5, một lượt nhìn của Ice*
  - [x] 4.1 Thêm token `grid-row-omitted` vào `src/tokens/tokens.json` theo đúng `DESIGN.md:148`: `{ color: ornament, decoration: line-through }`
  - [x] 4.2 🔴 **"Mờ đi" = đổi màu chữ sang token** `ornament`**, KHÔNG phải** `opacity`**.** Hai nguồn độc lập nói cùng một điều: `DESIGN.md:230` (*"*`opacity`* ở trạng thái nghỉ chỉ áp cho nét và nền, không áp cho chữ"*) và cổng `check-tokens.mjs:1345-1396` Kiểm D (chỉ `0`/`1` đi tự do; giá trị trung gian FAIL trừ khi có miễn trừ có tên `aura-allow-opacity`). **Đừng nhét miễn trừ để cổng hết đỏ** — đường đúng đã có sẵn
  - [x] 4.3 🔴 **Không có phần tử "hàng" để gắn class.** `GridPanel.vue:889` là grid cha; năm **cột** là năm con, mỗi cột `grid-row: 1 / -1` + `grid-template-rows: subgrid` (`:1080-1085`). Một hàng chỉ là track thứ *i* mà năm cột cùng chia (`:5-19`). ⇒ Thêm một `:class` boolean vào **từng** `v-for` của các cột cần tô, đúng khuôn `.para-end` đang lặp ở cả năm (`:897,913,923,975,991`)
  - [x] 4.4 ⚠️ **Không** thêm giá trị vào `SEGMENT_RULE_VALUES` (`editorSegments.ts:69-76`) và **không** dựng khối `.rule-omitted`. Kiểm I (`check-commands.mjs:2116-2269`) đối chiếu **ba chiều** (hằng TS ↔ hằng viết tay `EXPECTED_RULE_VALUES:2147` ↔ khối CSS) **và chiều ngược lại** — một `.rule-<x>` lạ trong CSS làm cổng FAIL. Cộng thêm: `EXPERIENCE.md:99` gọi bảng vạch lề là *"tài nguyên đã tiêu hết"*. Đây chính là AC2 nói bằng ngôn ngữ của cổng
  - [ ] 4.5 🔴 **CÒN MỘT MÓN CHO ICE** — Kiểm bằng mắt: hàng đã cắt bỏ vẫn **thẳng hàng** với các hàng khác (subgrid), và khoảng thở `is_paragraph_end` không vỡ
- [x] **Task 5 — Điều hướng bỏ qua** (AC6)
  - [x] 5.1 Thêm trường mới vào `NavigationSegment` (`src/panels/segmentNavigation.ts:26-34`)
  - [x] 5.2 Thêm một `continue` trong vòng lặp của `nextUntranslatedId` (`:82-87`), **ngay cạnh** `if (s.retiredAt !== null) continue` (`:85`)
  - [x] 5.3 🔴 **Giữ** `isUntranslated` **(**`:56-58`**) nguyên, không đổi định nghĩa.** Tiền lệ đã chốt điều này: "về hưu" cũng không được nhét vào `isUntranslated`, nó lọc riêng ở vòng lặp. *"Cắt bỏ"* là mệnh đề khác *"chưa dịch"*, y hệt vậy
  - [x] 5.4 Map trường mới trong `navigationSegmentOf` (`:98-108`)
  - [x] 5.5 Test theo khuôn ca *"câu đã VỀ HƯU không bao giờ là đích, kể cả khi rỗng"* (`tests/frontend/segmentNavigation.test.ts:85-92`)
  - [x] 5.6 ⚠️ Giữ `segmentNavigation.ts` **thuần** — chỉ `import type`. Viết thẳng mọi hằng chuỗi thay vì `import` từ `config/segment.ts`, đúng khuôn `'draft'` viết thẳng ba lần (`:51-54`) có test canh đồng thuận (`segmentNavigation.test.ts:148-185`)
  - [x] 5.7 ⚠️ AC này **trùng chủ với Story 2.10** (`epics.md:2599-2602` có nguyên văn cùng mệnh đề). 2.5c chạy trước ⇒ **2.5c dựng**, 2.10 chỉ **nghiệm thu lại**. Ghi rõ vào `§Completion Notes` để 2.10 không dựng đường thứ hai
- [x] **Task 6 — Chốt lọc cho đầu ra** (AC5 — hình dạng theo Quyết định #2)
  - [x] 6.1 Nếu Ice chọn đường (b): một **hàm thuần** ở `src-tauri/src/core/segment/` trả danh sách segment đã lọc cờ, + test hợp đồng khẳng định câu đã cắt bỏ **không xuất hiện**
  - [x] 6.2 🔴 Logic lọc ở **Rust**, không phải một `v-if` rải rác ở Vue (AD-1: *"frontend chỉ render và giữ state UI"*)
  - [x] 6.3 Ghi nợ có chủ vào `deferred-work.md` cho phần không nghiệm thu được: vế Chế độ đọc → Epic 5 (Story 5.11–5.13), vế xuất bản → Epic 8 (Story 8.3 · 8.4 · 8.6). ⚠️ Ghi kèm phát hiện: **không AC nào của các story đó tham chiếu FR133** — nghĩa vụ hôm nay chỉ phát biểu một chiều
- [x] **Task 7 — Nghiệm thu**
  - [x] 7.1 `npm run check:*` — 11 cổng đọc-tệp, **cộng** `check:scope` + `check:scope:bundled` chạy tay (cần **cổng 1420 trống**)
  - [x] 7.2 `npm run test` (vitest) · `npm run build` · `cargo test --locked`
  - [x] 7.3 e2e chạy tay. ⚠️ Bảo đảm **cổng 1420 trống** trước — bug `wdio.conf.mjs::devServerIsUp()` chỉ hỏi `res.ok`, một Vite hấp hối vẫn trả 200 và làm **7/7 spec đỏ oan** (`deferred-work.md:3274-3330`, chưa vá). Nếu **4445** bị chiếm: đặt `TAURI_WEBDRIVER_PORT`, **không** giết tiến trình của người dùng
  - [x] 7.4 ⚠️ Nếu Task 4 đổi cấu trúc DOM của hàng: **đo lại** độ trễ dời con trỏ. 2.5b đã đo **706–770 ms** trên 9.850 câu — vượt trần NFR2 (50 ms/frame) **~15 lần**, còn hở, chủ là Story 2.4 (`deferred-work.md:3164-3194`). Thêm một `:class` vào năm `v-for` làm nặng thêm. **Ghi số, đừng suy luận**
  - [x] 7.5 Mỗi vế không nghiệm thu được ở tầng này ⇒ `deferred-work.md` kèm chủ. **Không tự chấm đạt**

---

## Dev Notes

### Đọc trước khi viết dòng đầu tiên

`_bmad-output/project-context.md` — 130 luật. Ba mục sát story này: §Critical Don't-Miss Rules (*"Rỗng IM LẶNG bị cấm"* · *"Dữ liệu người dùng — chỗ hỏng là VĨNH VIỄN"*), §Testing Rules (bốn đường nghiệm thu, bốn vai không chồng nhau), §Code Quality (văn hoá chú thích: **lý do**, kèm **phép đo**, không sở thích).

### 🔴 Story này KHÔNG thêm phụ thuộc nào

Mọi thứ cần đã có trong kho: `rusqlite` cho cột mới · `CommandRegistry` cho lệnh · token `ornament` đã tồn tại · `vitest` cho test frontend · khuôn `confirm_segment` cho đường ghi.

Nếu dev agent thấy mình muốn thêm một gói, **dừng lại** — đó là dấu hiệu đang đi sai đường. Cửa NFR15 vẫn đứng và có ba bước bắt buộc (`project-context.md:92-100`): ① mở tệp giấy phép trong nguồn **đã tải** mà đọc, không tin nhãn registry (`vitest` khai `"MIT"` nhưng `LICENSE.md` dài 811 dòng, gộp giấy phép của 27 gói nó vendor); ② ghi vào bảng Stack của spine **trước khi** thêm — ba lượt rà đầu của dự án đều là lượt "đuổi theo"; ③ chỉ giấy phép tương thích GPLv3 **theo chiều đi vào**.

⚠️ Và sáu tên bị **cấm** cưỡng chế bằng `npm run check:deps`: `tauri-plugin-fs` · `tauri-plugin-dialog` · `tauri-plugin-sql` · `tauri-plugin-keyring` · `tauri-plugin-stronghold` · `tauri-wire`.

### 🔴 Hai bảng "sáu giá trị", khác tầng, trùng ba cái tên

Đây là bẫy đọc lớn nhất của story này. AC2 nói *"bảng sáu giá trị"* — và có **hai** bảng khác nhau trong kho:

| Bảng | Ở đâu | Bao nhiêu giá trị | Là gì |
| --- | --- | --- | --- |
| **Vạch lề** | `src/panels/editorSegments.ts:69-76` — `SEGMENT_RULE_VALUES` | **sáu**: `confirmed` · `primary` · `tm-rule` · `draft` · `none` · `ornament` | Từ vựng **hiển thị** phía frontend |
| **Trạng thái CSDL** | `src-tauri/src/commands/segment.rs:555,557` | **hai**: `draft` · `confirmed` | Cột `segment.status` |

Doc-comment tại `segment.rs:549-554` ghi thẳng *"Hai giá trị hợp lệ… và đúng hai — Quyết định #5"*. Đi tìm "bảng sáu giá trị" trong `schema.rs` sẽ không thấy gì. AC2 nói về bảng **vạch lề**.

### Lược đồ và di trú

`PROJECT_MIGRATIONS` (`schema.rs:524-555`) hôm nay có sáu bước, đích **7**:

| `to_version` | hằng | story |
| --- | --- | --- |
| 1 | `SCHEMA_MIGRATION_LOG_DDL` | 1.15 |
| 2 | `WORK_DDL` | 1.15 |
| 3 | `CHAPTER_DDL` | 1.15 |
| **5** | `SEGMENT_DDL` | 2.1 |
| 6 | `SEGMENT_TARGET_TEXT_DDL` | 2.2 |
| 7 | `SEGMENT_STATUS_AND_VERSION_DDL` | 2.5 |

**Số 4 đã cháy** (`schema.rs:494-523`) — bản đầu Story 1.20 gắn `PINNED_ENTRY_DDL` vào bước 4, sau gỡ khi ghim chuyển sang `global.db`. `validate_strictly_increasing` (`:571-589`) chỉ đòi tăng dần nghiêm ngặt và **không** bắt được việc tái dùng số 4; cổng thật là `segment_contract.rs::the_project_migration_set_never_reuses_the_burned_number_four` (`:473-485`).

**Bảng** `segment` **hôm nay** (sau ba bước gộp): `id` · `chapter_id` · `ord` · `source_text` · `is_paragraph_end` · `retired_at` · `created_at` · `updated_at` · `target_text` · `status`. Một chỉ mục: `idx_segment_chapter_ord (chapter_id, ord)`. Không `FOREIGN KEY`, không `CHECK`.

**Cách di trú chạy** (`schema.rs:656-710`): lọc `m.to_version > from`, mỗi bước **một** `Transaction` **riêng**, ghi một dòng vào `schema_migration_log`, đặt `PRAGMA user_version`, commit. Backup (`:617-648`) chạy khi `found >= 1 && found < target`: `wal_checkpoint(TRUNCATE)` → xác nhận `busy == 0` → `fs::copy` ra `<tên>.db.bak-v<n>`.

⚠️ `DEFAULT` **phải là hằng.** SQLite đòi một `DEFAULT` không `NULL` cho mọi `ADD COLUMN … NOT NULL` trên bảng đã có dữ liệu, và **không** nhận biểu thức/hàm ở vị trí đó. Cả bước 6 (`DEFAULT ''`) lẫn bước 7 (`DEFAULT 'draft'`) đều ghi lại lý do này tại chỗ (`schema.rs:367-370`, `:409-414`). Dữ liệu thật: **10.477** hàng `segment` từ 21 Chương (đo 2026-08-12) sẽ nhận giá trị backfill — chọn giá trị nói **đúng sự thật** ("không cắt bỏ"), không phải một giá trị mồi.

### Dây IPC — và một lỗi đã xảy ra thật

`ChapterSegment` (`segment.rs:154-163`) đi trên dây **không** có `#[serde(rename_all)]`, nên trường trả về giữ `snake_case`. ⚠️ Chiều ngược lại khác: `invoke()` gửi **tham số** dạng camelCase dù hàm Rust nhận `snake_case`. Hai chiều khác nhau — đây là chỗ dễ sai nhất trên dây (`project-context.md:161-164`).

🔴 **Tiền lệ lỗi phải chặn:** bản đầu Story 2.5 thêm cột `status` vào CSDL nhưng quên thêm vào **struct** `ChapterSegment` và vào **câu** `SELECT`. Kết quả: `segment.status` luôn `undefined` phía webview. **74/74 test frontend vẫn xanh** — vì fixture chép tay đã có sẵn `status`. Chỉ e2e bắt được. Doc-comment tại `segment.rs:144-153` ghi lại nguyên vụ. Story này thêm đúng một cột nữa vào đúng đường đó ⇒ **cùng cái bẫy, cùng vị trí**.

### Lưới: hình dạng thật

`GridPanel.vue:889` là `<div class="grid">` cha khai `grid-template-rows` động. **Năm cột là năm con trực tiếp**, mỗi cột `grid-row: 1 / -1` + `grid-template-rows: subgrid` (`:1080-1085`). Thứ tự: vạch trạng thái (`.col-rule`) · số câu (`.col-num`) · nguyên văn (`.col-src`) · bản dịch (`.col-tgt`) · nhãn trạng thái (`.col-state`).

🔴 **Một hàng KHÔNG phải một phần tử DOM** (`:5-19`). Mọi kiểu dáng cấp hàng phải nhân ra từng ô — ghi rõ tại `:1087-1092` và `:1253-1261`. Khuôn có sẵn để chép: `.cell.para-end` (`:1110-1113`) áp qua `:class="{ 'para-end': s.is_paragraph_end }"` lặp ở **cả năm** `v-for`.

**Neo:** `data-segment-id` xuất hiện **hai lần** mỗi câu (ô nguyên văn + ô bản dịch), phân biệt bằng `data-col="src"|"tgt"`.

`contenteditable`**:** đặt **tĩnh** trong template (`:981`), không binding động (`:962-964`). Đường chuột sản phẩm: `setPosition` (không `addRange`) ở `mouseup` (không `mousedown`), cộng một lượt vá ở frame kế — vì `contenteditable` **trần không đủ** trên WKWebView (xem bàn đo dưới).

`onBeforeInput` (`:695-746`) hôm nay chặn: `insertParagraph`/`insertLineBreak` (AD-37) và `insertFromPaste`/`insertFromDrop`/`insertReplacementText` (tự chèn text thuần đã làm phẳng). `onEditKeydown` (`:764-767`) chặn `Enter` trần khi không composing.

### Token và cổng thị giác

🔵 **Đặc tả đã chốt hình dạng, đây không phải chỗ chọn:** `DESIGN.md:148` khai sẵn `grid-row-omitted: { color: ornament, decoration: line-through }`. Grep `tokens.json` · `GridPanel.vue` · `editorSegments.ts` · `check-tokens.mjs` cho `omitted` — **rỗng**. Token có trong đặc tả, chưa có trong mã.

`text-decoration: line-through` **không bị cổng nào cấm**: `text-decoration` nằm trong `COMPOSITE_COLOR_PROPS` (`check-tokens.mjs:900-912`) nên Kiểm B chỉ soi **phần màu** bên trong giá trị ghép; `line-through` không mang màu, đi qua tự do.

Kiểm B/B2 (`check-tokens.mjs:833-1064`): mọi màu phải là `var(--color-*)`, mọi cỡ/họ chữ phải là `var(--font-…)`/`var(--face-…)`/`var(--leading-…)`. Dùng token `ornament` đã có ⇒ **không cần token màu mới**, không cần cặp tương phản mới.

### Điều hướng

`isUntranslated` (`segmentNavigation.ts:56-58`) = `status === 'draft' && targetText === ''`. Hai vế, có lý do: vế `status` loại ca "đã xác nhận rồi bị xoá trắng"; vế `targetText` loại ca "đã gõ nhưng chưa ký".

`nextUntranslatedId(segments, fromId)` (`:75-89`) duyệt tuần tự từ `fromId+1`, **không quay vòng** (hết Chương trả `null`), và **bỏ qua** segment có `retiredAt !== null` ở **cả hai vai** — không phải đích, và không chặn đường (`:72-74, :85`). Đây là khuôn chính xác cho cờ cắt bỏ.

Command đã có: `editor.confirm_segment` (`Mod+Enter`, `index.ts:900-910`) · `editor.next_untranslated` (`Alt+ArrowDown`, `index.ts:938-952`).

### Luật "erasable-only" — một `import` sai giết ba phép kiểm

Tệp phải **nạp được bằng Node trần** (cổng `import()` chúng để chạy kiểm **hành vi** trên chính mã sản phẩm): `src/commands/{registry,focus,keys,index}.ts` · `src/panels/editorSegments.ts` · `src/panels/segmentNavigation.ts` · `src/layout/{workspaceLayout,writeSchedule}.ts`.

⇒ Không `import` **giá trị** của `vue`/`dockview`/`@tauri-apps/api`; không `enum`, `namespace`, parameter property. ⚠️ Nguy cơ thật của story này: vô tình `import` một hằng từ `config/segment.ts` vào `segmentNavigation.ts` hoặc `editorSegments.ts` ⇒ kéo theo `@tauri-apps/api` ⇒ Kiểm I `abort()` (không phải FAIL — `abort()` dừng hẳn CI, `check-commands.mjs:794-813`).

`editorPanelState.ts` và `editorFlush.ts` **không** chịu luật này — logic thao tác viết tự do ở đó.

### Bài học từ Story 2.5b

**① Chẩn đoán bị phép đo bác — bốn lần.** Cú bấm đầu tiên vào lưới mất caret. Bốn lượt vá (`contenteditable` trần → `cell.focus()` trong `mouseup` → `rAF` → `setTimeout(0)`) đều **bị phép đo bác**: cả bốn chạy **trước** lượt `enterFocus` thật sự gây lỗi. Nguyên nhân thật: `WorkspaceDock.vue:591-611` gọi `enterFocus(id)` trên **mọi** lượt đổi panel kể cả khi tiêu điểm đã ở trong đó; `focus.ts::enter()` chạy `el.focus()` vô điều kiện. Đã vá bằng điều kiện `el.contains(document.activeElement)`.
⇒ **Bài học cho story này: đo trước khi vá.** Một bản vá "hợp lý" chạy sai chỗ vẫn trông như một bản vá.

**②** `setAttribute` **đồng bộ trong** `mousedown` **là lời giải đã thắng — đừng chạm.** Nó là bản vá của một chẩn đoán **đã bị bác một lần** ở Story 2.3.

**③ Bàn đo hai engine, 5 mệnh đề** (`2-5b:775-799`) — WKWebView 605.1.15 vs Blink:

| Mệnh đề | WKWebView | Blink |
| --- | --- | --- |
| Chuột thật vào ô rỗng ⇒ caret | trần: **KHÔNG** · đường chuột sản phẩm: ĐẠT | ĐẠT |
| `Backspace` offset 0 ⇒ `beforeinput` | **KHÔNG** (0 sự kiện) | CÓ |
| `subgrid` giữ hàng thẳng | ĐẠT (lệch 0 px) | ĐẠT (0 px) |

⇒ `happy-dom` **không phải WebKit**. Mọi mệnh đề về hình học/engine thuộc bàn đo hoặc e2e, không thuộc vitest.

**④ Cổng 4445 suýt cho số của app khác.** Máy chủ WebDriver bám cổng cố định; trên máy Ice `gdrive-su` (PID 19811) đang giữ nó, phiên nối nhầm vào webview app khác và **vẫn trả số hợp lệ**. Lộ ra vì `activeElement = BUTTON.sidebar-folder-tree__chevron` — một class **0 kết quả** trong toàn kho.

**⑤ File List kê thừa là một món nợ.** 2.5b từng kê `editorFlush.ts` vào mục "Sửa" trong khi `git diff` trên nó **trống**. Kê thừa "mua thời gian đọc một tệp không đổi" và làm người đọc tin nhầm hợp đồng đã được xem lại.

### Project Structure Notes

- Test frontend ở **\`tests/frontend/**` phẳng**, không đồng vị trí trong `src/` — bốn cổng đếm quần thể `**src/**` và một tệp test đổ vào đó** thổi phồng mẫu số**, cộng hai va chạm (Kiểm A `check-i18n` đỏ với chữ có dấu; Kiểm B `check-tokens\` đỏ với màu viết thẳng).
- Chuỗi literal trong `src-tauri/src/**` viết** KHÔNG DẤU**; `tests/**` được miễn trừ nên giữ dấu.
- Tên hàm test là một **câu khẳng định**, không `test_foo`.
- Hai họ test Rust: `*_contract.rs` (hợp đồng) · `*_boundary.rs` (ranh giới module).
- `src/panels/README.md` cập nhật cùng lượt nếu thêm một khái niệm.
- Commit: `type(scope): câu tiếng Việt`, `scope = story-2.5c`. Câu sau dấu hai chấm **nói ĐIỀU ĐÃ TÌM RA**, không chỉ điều đã sửa. Mỗi lớp một commit sạch (bài học `2-5b:692`).

### Bẫy đã biết, ghi ra thay vì để phát hiện lại

- ⚠️ **Loại suy XLIFF chỉ đúng một nửa.** `epics.md` và `EXPERIENCE.md:126` đều viết *"đúng khuôn *`translate="no"`* của XLIFF"*. Trong XLIFF 2.0, `translate="no"` **khoá** một unit/segment và **giữ nguyên nội dung trong bản xuất** — nó là *"đừng dịch cái này"*, không phải *"bỏ cái này đi"*. AC5 của story này đòi **ẩn hoàn toàn**. Loại suy đúng ở vế *"trục độc lập, không phải một mức độ hoàn thành"*; **sai** ở vế hành vi đầu ra. Đừng chép ngữ nghĩa XLIFF vào phần xuất bản.
- ⚠️ **Số 8 đang được một test dùng làm số "tương lai".** `segment_contract.rs:912-958` dựng fixture `to_version: 8` để mô phỏng một db mới hơn app. Sau story này, 8 là số **thật** ⇒ fixture mất ý nghĩa mà **vẫn xanh**.
- ⚠️ `epics.md:2329` **mang một ước lượng đã bị đo sai.** Nó đoán chiều cao hàng Hán Việt song song ~330 px / 6–7 dòng; 2.5b đo được **388 px = 11,47 dòng**, và cột thật của Ⓑ-2 chỉ 238,5 px chứ không ~330. Nếu story này cần lý luận về chiều cao hàng: dùng số đã đo, không dùng ước lượng trong epics.
- ⚠️ **Fixture e2e không reset state panel giữa các spec** (`deferred-work.md:3093-3115`, chủ Story 1.22). Spec mới nên tự nạp lại webview sau khi tạo Tác phẩm, đừng dựa vào state sạch.
- ⚠️ **Command id nằm cứng trong spec e2e và không cổng nào canh** (`deferred-work.md:3117-3129`). `check:commands` **không đọc \`e2e/**`** ⇒ thêm/đổi id thì phải tự rà `e2e/\*\*\` bằng tay.

### References

- `_bmad-output/planning-artifacts/epics.md:2333-2373` — Story 2.5c, bảy AC
- `_bmad-output/planning-artifacts/epics.md:2596-2602` — Story 2.10 mang **cùng** AC "bỏ qua câu đã cắt bỏ"
- `_bmad-output/planning-artifacts/prds/prd-AuraTranslate-2026-08-02/prd.md:458` — FR133 nguyên văn
- `.../ux-designs/ux-AuraTranslate-2026-08-02/EXPERIENCE.md:126-132` — cắt bỏ là trục riêng
- `.../EXPERIENCE.md:95` — *"phân biệt giữ lại với loại bỏ bằng độ lùi, không bằng màu nhấn thứ hai"*
- `.../EXPERIENCE.md:99` — bảng vạch lề là *"tài nguyên đã tiêu hết"*
- `.../DESIGN.md:148` — token `grid-row-omitted`
- `.../DESIGN.md:230` — luật `opacity` không áp cho chữ
- `.../architecture/.../ARCHITECTURE-SPINE.md:89-93` (AD-3) · `:362-366` (AD-30) · `:368-392` (AD-31) · `:75-79` (AD-1) · `:406-417` (AD-34) · `:689,694` (Consistency Conventions)
- `_bmad-output/planning-artifacts/sprint-change-proposal-2026-08-14.md:333-344` — thứ tự thi công 2.5b → 2.5c → 2.5d
- `src-tauri/src/core/store/schema.rs:335-346` · `:374-375` · `:451-458` · `:494-523` · `:524-555` · `:617-710`
- `src-tauri/src/commands/segment.rs:144-163` · `:305-328` · `:549-579` · `:683-757` · `:983-1100`
- `src-tauri/tests/segment_contract.rs:473` · `:492` · `:912` · `:1512` · `:1609` · `:2005`
- `src/panels/GridPanel.vue:5-19` · `:889` · `:962-982` · `:1080-1092` · `:1110-1113`
- `src/panels/segmentNavigation.ts:26-34` · `:56-58` · `:75-108`
- `src/panels/editorSegments.ts:69-76` · `src/panels/editorPanelState.ts:51,137-142,605-728`
- `src/config/segment.ts:66-89`
- `scripts/check-commands.mjs:2116-2269` (Kiểm I) · `:1876-2113` (Kiểm F) · `:665-746` (Kiểm A)
- `scripts/check-tokens.mjs:833-1064` (Kiểm B/B2) · `:1345-1396` (Kiểm D)
- `_bmad-output/implementation-artifacts/2-5b-luoi-hai-cot-doi-chieu.md:696-710` · `:738-762` · `:775-814` · `:851-947`
- `_bmad-output/implementation-artifacts/deferred-work.md:3093-3129` · `:3164-3194` · `:3274-3330`
- XLIFF 2.0 `translate` — https://simplelocalize.io/docs/file-formats/xliff-2/
- SQLite `ALTER TABLE ADD COLUMN` — https://www.geeksforgeeks.org/sqlite/sqlite-alter-table/

---

## Testing

Bốn đường nghiệm thu, **bốn vai không chồng nhau**. Chọn sai đường là dựng nguồn sự thật thứ hai — trước khi viết một phép kiểm mới, hỏi: **mệnh đề này đã có chủ ở đường nào chưa?**

| Mệnh đề của story này | Đường đúng |
| --- | --- |
| Bước 8 tồn tại, danh sách di trú đúng, backfill đúng trên db thật | `cargo test` — `segment_contract.rs` |
| Cột mới đi qua dây IPC (không `undefined` phía webview) | `cargo test` — hợp đồng, theo khuôn `:2005` |
| Ghi cờ là một transaction, từ chối có hình dạng, không chạm `status`/`target_text` | `cargo test` |
| `nextUntranslatedId` bỏ qua câu đã cắt bỏ | `vitest` — `segmentNavigation.test.ts` |
| `isUntranslated` **không** đổi nghĩa | `vitest` |
| Hàng đã cắt bỏ có `line-through` + màu `ornament` | cổng `check:tokens` + `check:commands` |
| Hàng vẫn **thẳng hàng** trong subgrid, khoảng thở không vỡ | **bàn đo tay / e2e** — không phải vitest (`happy-dom` không phải WebKit) |
| Caret trong ô đã cắt bỏ (nếu Quyết định #4 chọn (b)) | **e2e** — WKWebView thật |

**Luật cổng:** mã thoát là phán quyết; lỗi hạ tầng **không phải** một phép kiểm đỏ (`abort()`, thoát khác 0, nói rõ *"đây là lỗi hạ tầng"*); không phán quyết nào đọc tham số từ chính thứ nó đang kiểm.

**Luật đo:** không đánh dấu đạt bằng suy luận. Số đo ghi kèm **phiên bản toolchain và ngày** — *"số đo không truy nguyên được thì không phải số đo"*.

**Thêm một cổng = sửa BA danh sách** (`package.json` · `.github/workflows/ci.yml` · `.githooks/pre-push`), `check:gates` canh cả ba. Story này nhiều khả năng **không** thêm cổng nào.

---

## Nợ dự kiến (ghi vào `deferred-work.md` kèm chủ, không tự chấm đạt)

| Món | Trạng thái dự kiến | Chủ |
| --- | --- | --- |
| AC5 vế Chế độ đọc — không có bề mặt để nghiệm thu *"không dấu vết, không *`[…]`*, không chỗ trống"* | 🟡 | Epic 5 (Story 5.11–5.13) |
| AC5 vế xuất bản — `core::export` là khung rỗng, `docx-rs` chưa `use` | 🟡 | Epic 8 (Story 8.3 · 8.4 · 8.6) |
| Nghĩa vụ FR133 chỉ phát biểu **một chiều** — không AC nào của Epic 5/8 tham chiếu ngược lại | 🔴 hở | Ice (quyết định: có thêm AC vào các story đó không) |
| AC1 vế "dải câu" — nếu Quyết định #1 chọn đường (b) | 🟡 | theo chữ ký của Ice |
| Đo lại độ trễ dời con trỏ nếu Task 4 đổi cấu trúc DOM hàng | phụ thuộc | Story 2.4 (chủ bộ đo NFR2/NFR18) |

**🔵 Đối chiếu sau khi thi hành (2026-08-15).** Năm món dự kiến: **năm** đã ghi vào
`deferred-work.md` kèm chủ, đúng như bảng — AC1(b) theo chữ ký #1, hai vế AC5 theo #2(b), vế
FR133-một-chiều giao Ice, và độ trễ giao Story 2.4 *(Task 4 không đổi cấu trúc DOM — vẫn
5 node/hàng — nhưng thêm bốn phép đọc thuộc tính mỗi hàng mỗi lượt vá, nên vẫn giao lại thay
vì tự chấm)*.

**Và HAI món ngoài dự kiến, cả hai lộ ra lúc thi hành:**

| Món |  | Chủ |
| --- | --- | --- |
| `DESIGN.md` §components khai màu **trái** với cổng `check:tokens`, và **không cổng nào canh khối đó** — nó trôi khỏi mã mỗi story, lần này là lần **thứ tư** | 🔴 hở | Ice *(một câu hỏi về quy trình, không một dòng chữ)* |
| AC3 vế **hình học** — hàng đã cắt bỏ vẫn thẳng hàng trong `subgrid`, và `line-through` **kế thừa** xuống `SourceHanViet` trong ô nguyên văn | 🟡 | Ice *(một lượt nhìn)* |

---

## Dev Agent Record

### Agent Model Used

`claude-opus-5` (Claude Code, dev-story workflow)

### Baseline đo trước khi chạm dòng đầu tiên

Commit `d55d4ae`, 2026-08-15 · toolchain: rustc 1.97.1 (CI ghim) · vitest 4.1.10 · Node 22.

| Phép đo | Số | Khớp ghi chép? |
| --- | --- | --- |
| `cargo test --locked` | **338 xanh / 0 đỏ / 5 ignored** | khớp `sprint-status.yaml` sau lượt ra mã 2.5b |
| `npm run test` (vitest) | **89/89**, 9 tệp | khớp (83 → 89 sau lượt ra mã 2.5b) |

⚠️ Cây bẩn lúc khởi hành: đúng **hai** tệp, và cả hai là sản phẩm của `create-story` cho chính story
này (tệp story + một khối comment trong `sprint-status.yaml`). Không có dấu vết của story khác ⇒
**không** cần một commit dọn riêng (`project-context.md:425-426`).

### Chữ ký của Ice cho năm quyết định mở

| \# | Nội dung | Đường Ice chọn | Ngày |
| --- | --- | --- | --- |
| 1 | Phạm vi "một dải câu" | **(b)** một câu (câu đang có caret); vế "dải" ghi nợ có chủ | 2026-08-15 |
| 2 | AC5 — chốt lọc cho đầu ra | **(b)** lưu cờ **và** dựng sẵn hàm thuần lọc ở Rust + test hợp đồng | 2026-08-15 |
| 3 | Cách gọi thao tác | **(b)** hai lệnh `editor.omit_segment` + `editor.restore_segment` | 2026-08-15 |
| 4 | Ô đã cắt bỏ có gõ được không | **(b)** chặn ở `onBeforeInput`, giữ `contenteditable` **tĩnh** | 2026-08-15 |
| 5 | Tên và kiểu cột | **(a)** `is_omitted INTEGER NOT NULL DEFAULT 0` | 2026-08-15 |

**Hệ quả đã ghi ngay lúc ký (Task 0.3):**

- #1(b) làm AC1 đóng **một nửa** 🟡 ⇒ vế "một dải câu" vào `deferred-work.md` kèm chủ. **Không** sửa

`epics.md` cho khớp mã (`project-context.md:456-458`).

- #2(b) **không** đóng trọn AC5 — nó dựng đúng cái chốt mà hai bề mặt tiêu thụ sẽ cắm vào. Hai vế

"ẩn" vẫn 🟡 và vào sổ nợ (Epic 5 · Epic 8), cộng phát hiện *"nghĩa vụ FR133 chỉ phát biểu một
chiều"* — chủ là Ice.

- #4(b) giữ nguyên mệnh đề `GridPanel.vue:962-964`; **không** một binding động nào được thêm.
- #5(a) chốt luôn từ vựng: cột `is_omitted`, token `grid-row-omitted`, lệnh `editor.omit_segment` —

**một** từ `omitted`, không đặt từ thứ hai.

### Debug Log References

#### 🔴 Ba tiền đề của story bị phép đo BÁC — ghi ra thay vì sửa lặng lẽ

**① Task 4.1 sai chỗ:** `grid-row-omitted` **KHÔNG thuộc** `tokens.json`**.
**Task 4.1 viết *"thêm token *`grid-row-omitted`* vào *`src/tokens/tokens.json`* theo đúng
*`DESIGN.md:148`*"*. Đo 2026-08-15: `tokens.json` có **0** khoá `grid-` — `grid-num-col`,
`grid-state-col`, `grid-empty-cell` cũng **không** ở đó. Khối `components:` của `DESIGN.md
`là một **đặc tả thành phần**, không phải tệp token; 2.5b đã cài `grid-empty-cell` bằng CSS
trong `GridPanel.vue` (`.cell-tgt.empty`). ⇒ Đi đường đã đo: một khối CSS `.cell.omitted`,
cùng khuôn. Thêm một khoá `grid-` vào `tokens.json` là dựng một hạng mục mới mà không cổng
nào biết đọc.

**② Task 4.2 chỉ vào một token BỊ CẤM.** Xem Quyết định #6 ở trên và mục tương ứng trong
`deferred-work.md`. Tự kiểm đã chạy: `ornament` ⇒ `check:tokens` **ĐỎ**, trả lại ⇒ **XANH**.

**③** `core/export/mod.rs` **có SÁU dòng, không bảy.** Story ghi bảy. Không đổi kết luận *(vẫn
toàn doc-comment, không một dòng mã)*, đính chính để số đo truy nguyên được.

#### Bốn ca neo vào "đích là 7" — đỏ ngay lượt chạy đầu sau bước 8

Bước di trú 8 làm **bốn** ca ngoài danh sách Task 1 đỏ, tất cả vì cùng một lý do *(chúng
khẳng định phiên bản đích)*, cộng `SegmentRow` 10 → 11 cột. Hai cái tên mang số hiệu đã được
**gỡ số ra khỏi tên**, vì chúng sẽ sai lại ở **mỗi** story thêm một bước:

| Ca | Sửa |
| --- | --- |
| `a_fresh_project_database_lands_at_version_seven_...` | → `..._lands_at_the_target_...`, assert 7 → **8** |
| `..._at_version_five_migrates_to_six_...` | → `..._migrates_up_...`, assert 7 → **8** |
| `..._at_version_six_migrates_to_seven_...` | → `..._migrates_up_...`, assert 7 → **8** |
| `..._stranded_at_the_burned_version_four_...` | assert 7 → **8** (tên không mang số) |

🔴 Số vẫn viết **thẳng** ở `assert_eq!`, **không** dẫn xuất từ `PROJECT_MIGRATIONS`: *"không
phán quyết nào được đọc tham số từ chính thứ nó đang kiểm"*. Một lượt thử dùng hằng dẫn xuất
đã bị bỏ ngay khi viết ra.

#### Phép tự kiểm đỏ-rồi-xanh đã CHẠY cho mọi ca mới

| Đột biến gieo vào mã sản phẩm | Ca đỏ |
| --- | --- |
| `DEFAULT 0` → `DEFAULT 1` ở bước 8 | 2 *(hình dạng cột · backfill)* |
| `UPDATE … SET is_omitted, status='draft'` | 2 *(AC2 trục độc lập · AC4 vòng đảo ngược)* |
| gỡ nhánh gác `retired_at` | 1 *(ba lối từ chối)* |
| `color: var(--color-ornament)` | cổng `check:tokens` |
| *(RED trước khi cài)* thiếu `set_segment_omitted` · thiếu trường `is_omitted` · thiếu `setCurrentSegmentOmitted` | biên dịch · 3 · 9 ca vitest |

### Completion Notes List

**Đã làm — bảy task, và cái gì đóng được thì đóng, cái gì không thì giao lại có chủ.**

- **AC7 ✅** — bước di trú **8** (`SEGMENT_OMITTED_DDL`), `PROJECT_MIGRATIONS` nay bảy bước

`1·2·3·5·6·7·8`. Số 4 vẫn cháy. Test backfill chạy trên fixture dựng từ **sáu bước THẬT**,
không chép tay DDL.

- **AC2 ✅** — trục độc lập, cưỡng chế ở **ba** tầng: câu `UPDATE` chạm **đúng một cột**

*(kể cả *`updated_at`* cũng không)*; ca hợp đồng so **mười cột kia từng byte**; ảnh chụp
webview dựng bằng **trải phần tử cũ**, không dựng lại từ `outcome`.

- **AC4 ✅** — và nó đúng **mà không một dòng mã khôi phục nào**: lượt cắt bỏ không xoá gì

thì lượt bỏ cờ không phải dựng lại gì. Ca vòng tròn ký → cắt bỏ → bỏ cờ so từng byte.

- **AC6 ✅** — `nextUntranslatedId` bỏ qua câu đã cắt bỏ ở **cả hai vai**, lọc **cạnh**

`retiredAt`. 🔴 `isUntranslated` **không đổi một chữ**, và có một ca riêng khẳng định điều
đó.

- **AC3 🟡** — gạch ngang + `on-surface-variant` trên **bốn** cột chữ. Cột vạch trạng thái

**không** nhận, vì AC2 nói câu *"vẫn giữ trạng thái riêng"* và tô vạch là xoá đúng thông
tin đó. Vế **hình học** *(hàng vẫn thẳng hàng trong subgrid)* chưa có phép đo — sổ nợ.

- **AC1 🟡** — **một câu**, theo chữ ký #1(b) của Ice. Vế *"một dải câu"* vào sổ nợ kèm chủ.
- **AC5 🟡** — chốt lọc `core::segment::omit` + 2 ca hợp đồng, theo chữ ký #2(b). **Không**

đóng trọn: hai bề mặt tiêu thụ là khung rỗng.

🔴 **AC6 TRÙNG CHỦ VỚI STORY 2.10 — đọc dòng này trước khi làm 2.10.** `epics.md:2599-2602
`mang **nguyên văn** cùng mệnh đề. **2.5c đã DỰNG nó
**(`segmentNavigation.ts::nextUntranslatedId`, ca vitest *"câu đã CẮT BỎ không bao giờ là
đích"*). Story 2.10 chỉ **nghiệm thu lại** — **đừng dựng đường thứ hai**; hai đường cùng canh
một mệnh đề là hai nguồn sự thật.

⚠️ **Một quyết định hình dạng chưa nằm trong năm quyết định của Task 0**, ghi ra để không ai
tưởng nó đã được xét: tầng dây có **một** vỏ `wire::set_segment_omitted(segmentId, omitted)
`cho **hai** command của `CommandRegistry`. Chữ ký #3(b) đòi hai **lệnh** — và chúng có thật,
hai id, hai nhãn, hai hợp âm (`Mod+Alt+X` · `Mod+Alt+R`). Một vỏ IPC thứ hai chỉ để chở một
boolean khác là một bề mặt nữa phải cấp quyền và canh, cho cùng một điều.

⚠️ `editorOmitError` **được export mà chưa component nào đọc** — đúng hình dạng và đúng món
nợ với `editorConfirmError` của Story 2.5. Hai khoá `err.segment.*` mà đường cắt bỏ dùng lại
dừng ở biên giới TypeScript, **không tới màn hình**. Ghi tại chỗ khai nó, không giấu.

### File List

⚠️ Kê từ `git diff --stat` trên `d55d4ae`, **không** kê từ trí nhớ. Bài học ⑤ của 2.5b: một
tệp kê thừa *"mua thời gian đọc một tệp không đổi"* và làm người đọc tin nhầm rằng một hợp
đồng đã được xem lại.

**Thêm (3)**

| Tệp |  |
| --- | --- |
| `src-tauri/src/core/segment/omit.rs` | chốt lọc cho mọi đầu ra (AC5, Quyết định #2(b)) |
| `tests/frontend/editorOmitSegment.test.ts` | 9 ca — ảnh chụp hiển thị, bốn nhóm |
| `_bmad-output/implementation-artifacts/2-5c-cat-bo-cau-khoi-ban-dich.md` | chính tệp story |

**Sửa (20)**

*Tầng Rust — lược đồ và lệnh*

| Tệp | Đổi gì |
| --- | --- |
| `src-tauri/src/core/store/schema.rs` | `SEGMENT_OMITTED_DDL` + bước 8; hai doc-comment sửa 🔵 |
| `src-tauri/src/core/store/mod.rs` | tái xuất hằng mới |
| `src-tauri/src/commands/segment.rs` | `ChapterSegment.is_omitted` · `SELECT` · `OmitOutcome` · `OmitReject` · `set_segment_omitted` + vỏ `wire` |
| `src-tauri/src/core/segment/mod.rs` | khai `pub mod omit` + doc-comment |
| `src-tauri/src/lib.rs` | đăng ký vỏ IPC |
| `src-tauri/tests/segment_contract.rs` | +9 ca; 4 ca neo đích 7→8; `SegmentRow` 10→11 cột |
| `src-tauri/tests/pinned_contract.rs` | hai hằng neo 6/7 → 7/8 |

*Tầng TypeScript*

| Tệp | Đổi gì |
| --- | --- |
| `src/config/segment.ts` | `is_omitted` trên kiểu dây · `OmitOutcome` · `setSegmentOmitted` |
| `src/panels/editorPanelState.ts` | `setCurrentSegmentOmitted` · `editorOmitError` · `OmitResult` |
| `src/panels/segmentNavigation.ts` | `NavigationSegment.isOmitted` · `continue` cạnh `retiredAt` · ánh xạ |
| `src/panels/GridPanel.vue` | `:class` `omitted` ở **bốn** cột + khối CSS `.cell.omitted` |
| `src/commands/index.ts` | cổng `setSegmentOmitted` + hai command |
| `src/main.ts` | nối `setCurrentSegmentOmitted` vào `installCommands` |
| `src/i18n/vi.json` | hai nhãn `command.editor.{omit,restore}_segment` |

*Cây test frontend — ba fixture chép tay, *`vue-tsc`* bắt cả ba*

| Tệp |  |
| --- | --- |
| `tests/frontend/support/segmentFixture.ts` | `is_omitted: false` ×3 + khối lý do |
| `tests/frontend/segmentNavigation.test.ts` | +3 ca (AC6, AC2, ánh xạ) |
| `tests/frontend/editorSegmentRule.test.ts` | một dòng dựng fixture |

*Tài liệu*

| Tệp |  |
| --- | --- |
| `_bmad-output/.../DESIGN.md` | `:148` theo Quyết định #6; **và** `:145`/`:146` — hai dòng cũ mâu thuẫn `:213` và mâu thuẫn mã từ 2.5b |
| `_bmad-output/implementation-artifacts/deferred-work.md` | 6 mục *(3 ghi lúc ký Task 0, 3 lúc nghiệm thu)* |
| `_bmad-output/implementation-artifacts/sprint-status.yaml` | trạng thái story |

🔴 **Không đụng một dòng nào:** `selectionContract.ts` · `editorSegments.ts` ·
`editorFlush.ts` · `SEGMENT_RULE_VALUES` · `focus.ts` · `check-*.mjs`. Không cổng mới, không
phụ thuộc mới, không `AD` mới.

### Change Log

| Ngày | Việc |
| --- | --- |
| 2026-08-15 | Task 0 — năm quyết định mở, Ice ký trọn gói: #1(b) · #2(b) · #3(b) · #4(b) · #5(a). Ghi nợ ba mục lúc ký |
| 2026-08-15 | Task 1 — bước di trú 8 (`is_omitted`), + 4 ca neo đích dời theo, `SegmentRow` 10→11 |
| 2026-08-15 | Task 2 — cột mới đi qua dây IPC; `vue-tsc` bắt ba fixture chép tay *(lớp lỗi Story 2.5 phải nhờ e2e mới thấy)* |
| 2026-08-15 | Task 3 — `set_segment_omitted` + hai command `Mod+Alt+X` / `Mod+Alt+R` |
| 2026-08-15 | Task 4 — 🔴 Quyết định **#6** phát sinh: `DESIGN.md:148` chỉ vào một token bị cổng cấm làm màu chữ. Ice ký đường (a) `on-surface-variant`; sửa `DESIGN.md` ba dòng |
| 2026-08-15 | Task 5 — điều hướng bỏ qua; `isUntranslated` **không đổi nghĩa** |
| 2026-08-15 | Task 6 — chốt lọc `core::segment::omit` cho AC5 |
| 2026-08-15 | Task 7 — nghiệm thu; ba vế không đo được ở tầng này đi vào sổ nợ kèm chủ |

## Nhật ký sprint-status

Gỡ nguyên văn từ `sprint-status.yaml` ngày 2026-08-19: tệp đó giữ TRẠNG THÁI, nội dung story thuộc về tệp này. Không sửa một ký tự.

```
  # 2.5c dung buoc di tru 8; 2.5d dung buoc 9. Doc PROJECT_MIGRATIONS (schema.rs), dung
  # doc mot ghi chep o noi khac — ke ca dong nay.
  # 🔵 2026-08-15 — create-story: chuyen sang ready-for-dev. Story mang NAM quyet dinh mo
  #   phai co chu ky cua Ice TRUOC khi viet dong ma dau tien (Task 0 chan moi task khac):
  #   #1 "mot dai cau" — nang luc chon nhieu HANG chua ton tai (`editorPanelState.ts:51` la
  #      mot `Ref<number|null>`), va dac ta khong mo ta co che chon. Ba duong, khong mac dinh.
  #   #2 AC5 "an hoan toan o Che do doc va moi ban xuat" — CA HAI be mat la KHUNG RONG:
  #      `core/export/mod.rs` co 7 dong toan doc-comment; `ReadingMode.vue` tu ghi "khung rong".
  #      Nghia vu FR133 con phat bieu MOT CHIEU — khong AC nao cua Epic 5/8 tham chieu nguoc lai.
  #   #3 cach GOI thao tac — dac ta khong noi (grep EXPERIENCE/DESIGN/prd/epics: 0 ket qua).
  #   #4 o da cat bo con go duoc khong — dung vao menh de "khong binding dong" 2.5b vua dat.
  #   #5 ten/kieu cot: `is_omitted INTEGER` hay `omitted_at TEXT`, ca hai deu co tien le.
  # ⚠️ Hai bay da ghi trong story: (a) loai suy `translate="no"` cua XLIFF chi dung MOT NUA —
  #   XLIFF KHOA va GIU nguyen van o ban xuat, AC5 doi AN HOAN TOAN; (b) so 8 dang bi
  #   `segment_contract.rs:912` dung lam so "tuong lai" trong mot fixture — sau story nay
  #   fixture do mat y nghia ma VAN XANH, phai nang len 9.
  # ⚠️ AC6 ("bo qua cau da cat bo") trung nguyen van voi mot AC cua Story 2.10 (epics.md:2599).
  #   2.5c chay truoc ⇒ 2.5c DUNG, 2.10 chi nghiem thu lai. Dung dung duong thu hai.
  # 🔵 2026-08-15 — 2.5c chuyen sang in-progress (dev-story). Baseline: commit d55d4ae
  #   (cargo 338/0/5 · vitest 89/89 — KHOP so ghi trong story).
  # ✅ 2026-08-15 — 2.5c XONG, chuyen sang `review`. Ice ky NAM quyet dinh cua Task 0
  #   (#1(b) mot cau · #2(b) chot loc o Rust · #3(b) hai lenh · #4(b) chan onBeforeInput ·
  #   #5(a) `is_omitted INTEGER`), CONG mot quyet dinh #6 phat sinh giua chung.
  #   Nghiem thu: 11 cong npm (gom check:scope + check:scope:bundled chay tay) · build ·
  #   vue-tsc · vitest 101/101 · cargo test 347/0/5 · e2e 7/7 spec, 9/9 ca (7m56).
  #   Buoc di tru 8 DA TIEU (`segment.is_omitted`). So ke tiep la **9** (Story 2.5d).
  #   Nguon su that van la `PROJECT_MIGRATIONS` (schema.rs), khong phai dong nay.
  # 🔴 QUYET DINH #6 — dac ta TU MAU THUAN, bat giua Task 4: `DESIGN.md:148` khai mau
  #   `ornament` cho hang da cat bo, ma `ornament` bi `check-tokens.mjs:1300-1334` CAM lam
  #   mau chu (do 2026-08-15: 2,44 sang / 2,64 toi — truot san AA 4,5). Tu kiem da chay:
  #   dat `ornament` => cong DO; tra lai => XANH. Ice ky duong (a) `on-surface-variant`
  #   (5,60 / 5,56). Day dung khuon Quyet dinh #9(a) cua 2.5b da giai mot lan.
  # 🔴 KHUON LAP LAI LAN THU TU: `DESIGN.md:145` va `:146` van khai `ornament` cho hai cot
  #   ma MA da dung `on-surface-variant` tu 2.5b, VA chung mau thuan voi chinh `DESIGN.md:213`.
  #   Da sua ca hai. Mon con lai la mot cau hoi QUY TRINH: khoi §components cua DESIGN.md
  #   khong co cong nao canh, nen no troi khoi ma moi story. Chu: Ice.
  # 🔴 CON MOT MON CHO ICE: Task 4.5 — nhin bang mat rang hang da cat bo van THANG HANG
  #   trong subgrid, va `line-through` KE THUA xuong SourceHanViet trong o nguyen van.
  #   Khong duong nghiem thu nao cua du an mo phong duoc ve hinh hoc (happy-dom khong phai WebKit).
  # 🟡 BA AC dong MOT NUA, ca ba ghi no co chu, KHONG tu cham dat:
  #   AC1 (ve "dai cau" — chu ky #1(b)) · AC3 (ve hinh hoc) · AC5 (hai be mat tieu thu la
  #   khung rong: `core/export/mod.rs` 6 dong toan doc-comment, `ReadingMode.vue` tu ghi rong).
  # 🔴 AC6 TRUNG CHU VOI STORY 2.10 (epics.md:2599-2602, nguyen van cung menh de):
  #   2.5c DA DUNG no. 2.10 chi NGHIEM THU LAI — dung dung duong thu hai.
  # 🔵 Sau mon no moi/da dong vao deferred-work.md — ba mon ghi LUC KY Task 0, ba mon luc nghiem thu.
```
