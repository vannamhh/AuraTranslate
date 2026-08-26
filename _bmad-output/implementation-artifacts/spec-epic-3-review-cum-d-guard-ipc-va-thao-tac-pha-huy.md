---
title: 'Cụm D — mười một chỗ hỏng ở frontend Glossary: dây không ai kiểm, cờ kẹt, và một phím xoá vĩnh viễn'
type: 'bugfix'
created: '2026-08-26'
status: 'done'
review_loop_iteration: 0
baseline_commit: '0f071845f01c02f48799108fae568844c639fc1a'
context:
  - '{project-root}/_bmad-output/implementation-artifacts/epic-3-context.md'
  - '{project-root}/AGENTS.md'
  - '{project-root}/src/AGENTS.md'
  - '{project-root}/tests/AGENTS.md'
---

<frozen-after-approval reason="human-owned intent — do not modify unless human renegotiates">

## Intent

**Problem:** Cụm D của vòng rà Epic 3 mang mười ba phát hiện ở frontend. Đo lại trên `0f07184`: **hai mục bị BÁC bằng phép đo** (vé `sequence` của Quick Add — dải không đóng/mở lại được trong lúc lưu; đua id Chương ở `glossaryMarksState` — dòng nhả `requestedForChapterId` chính là thứ MỞ đường thử lại, ngược điều sổ nợ khai), còn **mười một mục đúng nguyên**, trải ra **mười bốn chỗ vá**. Nặng nhất là hai lớp: ① `src/config/glossary.ts` khai chính sách *"dữ liệu IPC là một lời khai, không phải bảo đảm của trình biên dịch"* rồi **bỏ trống guard ở sáu đường**, trong đó `lookupGlossaryTerm` là đường duy nhất trong bảy adapter không kiểm cả một object lồng; ② `GlossaryManageOverlay` cho **Backspace/Delete xoá vĩnh viễn** hàng đang chọn — không xác nhận, không hoàn tác — và tiêu điểm mặc định ngay sau khi mở đúng là trạng thái đó, con trỏ đứng ở hàng đầu.

**Approach:** Kéo mọi dữ liệu qua dây của Glossary về đúng một khuôn `invoke<unknown>` + type guard, dùng lại `isGlossaryMark` làm mẫu. Cấp cho lượt xoá một **nhịp xác nhận nội tuyến** trong chính lớp phủ — cùng hình dạng preview-trước-khi-ghi mà đường Import bên cạnh đã có, không dựng component hộp thoại mới. Gộp hai cờ bận của Xuất/Nhập thành một cửa loại trừ. Bọc hai chỗ mã-ngoài-tầm-kiểm-soát chạy dưới listener bàn phím. Không thêm phụ thuộc, không đổi bề mặt IPC, không bước di trú.

## Boundaries & Constraints

**Always:**
- 🔴 **Mỗi mục vá kèm một ca test mà GỠ bản vá ra thì ca đó ĐỎ**, và ghi lại **tên ca** cùng **số ca đỏ thật**. Bộ test cũ vẫn xanh **không đủ** — Epic 3 dính đúng lớp lỗi này năm lần trong bảy ngày.
- 🔴 **Guard phải chạy THẬT trong ca test.** Năm tệp test hiện có (`glossaryImportPreview` · `glossaryManage` · `glossaryQueue` · `glossaryQuickAdd` · `glossaryQuickAddStrip`) thay TRỌN module `src/config/glossary` bằng bản giả, nên **không guard nào chạy ở đó**. Ca của nhóm ① phải mock đúng biên `@tauri-apps/api/core` — khuôn có sẵn ở ba tệp `glossaryConfirmStrip*`.
- 🔴 **Dữ liệu sai hình dạng đi vào ⇒ một `IpcError` ĐỌC ĐƯỢC đi ra, không phải một ngoại lệ và không phải `null` im lặng.** Adapter ở `src/config/*.ts` **không bao giờ ném** và giữ nguyên hình dạng ba trạng thái `{ <giá trị> | null, error: IpcError | null }`.
- 🔴 **Trường của struct TRẢ VỀ giữ `snake_case`.** Guard mới viết `work_tier_available`, `is_shadowed`, `row_count` — không camelCase hoá.
- 🔴 **Nhịp xác nhận xoá là hai nhịp trong CÙNG lớp phủ**, `Escape` huỷ nhịp một, và nhịp một **không** phát một lời gọi IPC nào. Không dựng component hộp thoại dùng chung (kho chưa có; bảy lớp phủ tự cài `role="dialog"` riêng) — dựng một cái ở đây là một quyết định kiến trúc, không phải một bản vá.
- 🔴 **`throw` cho một `CommandId` chưa đăng ký ở `registry.ts:203-211` GIỮ NGUYÊN** — nó là nửa cưỡng chế lúc chạy của AC1, có doc-comment riêng. Chỉ `spec.run()` được bọc.
- 🔴 **`registry.ts` không được `import` bất cứ thứ gì** — `check:commands` Kiểm C/D/E `import()` nó bằng Node trần. Chẩn đoán viết tay tại chỗ bằng `console.error`, không mượn một tiện ích log từ tệp khác.
- **Khoá `vi.json` mới đi cùng khuôn khoá chấm phẳng có tiền tố miền**, placeholder khớp `[a-z_][a-z0-9_]*`, không giá trị rỗng.
- **Màu và cỡ chữ chỉ từ token**; trạng thái "chờ xác nhận xoá" phân biệt bằng **chữ và ký hiệu**, không bằng `opacity` trung gian.
- **Mọi thao tác mới đăng ký ở `CommandRegistry` trước khi gắn vào phím hay chuột** (AD-34 §1); `@click` vẫn là đúng một lời gọi `dispatch('<id>')`.

**Ask First:**
- Nếu bọc `spec.run()` làm **bất kỳ** phép kiểm nào của `check:commands` (Kiểm C/D/E) đỏ: **DỪNG và trình lỗi**. Đỏ ở đó nghĩa là một cổng đang chốt cứng "ngoại lệ của handler phải thoát ra" như một hành vi mong muốn — đó là câu hỏi cho Ice, không phải một cổng cần nới.
- Nếu đóng cửa loại trừ Xuất↔Nhập hoá ra đòi một ô nhớ dùng chung giữa `glossaryManageState.ts` và `glossaryImportState.ts` (hai module state độc lập hôm nay): trình **cả hai hình dạng** — một cờ thứ ba ở tầng trên, hay mỗi bên đọc `readonly` của bên kia — kèm chỗ nào sinh phụ thuộc vòng, đừng tự chọn.
- Nếu một guard mới làm ca hiện có đỏ vì **Rust thật sự đang gửi hình dạng khác** với điều `src/config/glossary.ts` khai: đó là một lệch hợp đồng dây, không phải một guard quá chặt — dừng và trình chỗ lệch.

**Never:**
- Không đụng hai mục **đã bị đo bác** (vé `sequence` của `saveGlossaryQuickAdd`; đua id Chương ở `glossaryMarksState`) — chúng đóng bằng `→ KHÔNG LÀM` kèm số đo, không bằng mã. Thêm một nhánh mà kiểu nói không bao giờ chạy là đúng thứ `tests/AGENTS.md` gọi là *"mã chết vĩnh viễn trong sản phẩm"*.
- Không đụng 20 phát hiện của cụm E và F, và không đụng mục nợ **C4**.
- Không đổi bề mặt IPC: không thêm/bớt `#[tauri::command]`, không đổi tên tham số trên dây, không đụng `src-tauri/**` trừ khi một guard phát hiện lệch hợp đồng (thì dừng theo §Ask First).
- Không thêm phụ thuộc npm nào (NFR15).
- Không thêm `?.` vào mã sản phẩm để một ca hết đỏ; khoảng thiếu của `happy-dom` vá ở `tests/frontend/support/setup.ts`.
- Không hạ ngưỡng, không `eslint-disable`, không chuyển một ca sang danh sách loại trừ để cổng hết đỏ.

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected Output / Behavior | Error Handling |
|----------|--------------|---------------------------|----------------|
| **①** `lookupGlossaryTerm` nhận hình dạng lạ | Rust trả `{ entry: {…}, work_tier_available: null }` | Guard từ chối; adapter trả `{ lookup: null, error: <IpcError đọc được> }` | 0 lượt ném, 0 truy cập trường trên dữ liệu chưa kiểm |
| **②** `addGlossaryTerm` / `approveGlossaryCandidate` trả không phải số nguyên | `invoke` trả `"12"` hoặc `12.5` | Cùng ca ① — id không đi tiếp vào state | `error` khác `null` |
| **③** `exportGlossaryTier` trả không phải chuỗi | `invoke` trả `42` | Cùng ca ①; **`null` vẫn là "đã huỷ hộp thoại"**, im lặng có chủ, KHÔNG phải lỗi | phân biệt được huỷ ↔ hỏng |
| **④** tóm tắt lượt nhập có số thực | `{ inserted: 1.5, updated: 2, identical: 0 }` | Guard từ chối cả bản ghi | `error` khác `null` |
| **⑤** xem-trước lượt nhập có số thực | `row_count: 3.7` | `isGlossaryImportPreview` trả `false` | như ④ |
| **⑥** mục Glossary khai `is_shadowed` sai tầng | `{ tier: 'work', is_shadowed: true }` | `isGlossaryEntry` trả `false` — bất biến chéo trường | như ④ |
| **⑦** mở lại xem-trước rồi HUỶ hộp thoại trong lúc một lượt xác nhận đang bay | `confirming = true`, `openGlossaryImportPreviewOverlay` chạy, outcome `'cancelled'` | `confirming` về `false` — nút xác nhận **dùng lại được** | không câu lỗi nào (huỷ là im lặng có chủ) |
| **⑧** lượt huỷ lô trượt ở tầng IPC | `cancelGlossaryImport()` trả `error` khác `null` | Lớp phủ vẫn đóng ngay, **và** một chẩn đoán nêu đích danh được ghi | không nuốt im lặng |
| **⑨** bảng chờ đang nạp | `queueStatus === 'unknown'` | Một ca **có tên riêng**, phân biệt được với "đã nạp và sạch" | N/A |
| **⑩** duyệt xong hàng cuối trong phiên | `status = 'loaded'`, mọi `row.outcome` khác `null` | Câu *"bảng chờ đã sạch"* hiện ra — nhánh phải **sống** | N/A |
| **⑪** Backspace ngay sau khi mở lớp phủ Quản lý | tiêu điểm ở `<section>` panel, con trỏ ở hàng đầu | Nhịp một: hàng vào trạng thái **chờ xác nhận xoá**, **0 lượt IPC** | N/A |
| **⑫** Backspace lần hai trên đúng hàng đó | đang chờ xác nhận | Xoá thật, rồi nạp lại trọn danh sách | lỗi hiện qua `manageActionError` |
| **⑬** đang chờ xác nhận rồi dời con trỏ / bấm `Escape` / sửa hàng | ArrowDown, `Escape`, hoặc vào chế độ sửa | Trạng thái chờ **tan**, không xoá gì | N/A |
| **⑭** bấm Xuất rồi bấm Nhập ngay | một lượt Xuất đang bay | Nút Nhập **không** khởi được lượt thứ hai — đúng một hộp thoại hệ điều hành bay | N/A |
| **⑮** `surface.resolve` của một panel ném | `resolve()` throw trong `currentSelectionText` **và** trong `currentSelectionTextForGlossaryQuickAdd` | Trả `''`; chẩn đoán nêu đích danh | ngoại lệ **không** thoát ra ngoài |
| **⑯** handler của một command ném | `spec.run()` throw, phát từ một hợp âm bàn phím | `dispatch` ghi chẩn đoán nêu đích danh id rồi trả về | ngoại lệ **không** thoát ra khỏi listener `keydown`; `throw` cho id **chưa đăng ký** vẫn ném |
| **⑰** Quick Add khi không có Tác phẩm mở | `quickAddWorkTierAvailable === false` | Radio tầng Tác phẩm `disabled` — 0 round-trip IPC vô ích | câu lý do vẫn hiện, không đổi |

</frozen-after-approval>

## Code Map

### Nhóm ① — guard lúc chạy trên dây (`src/config/glossary.ts`, 936 dòng)

- **Khuôn TỐT NHẤT để bắt chước: `isGlossaryMark` (`:279-309`)** — hàm `isX` **duy nhất** trong tệp kiểm đủ mọi trường + `Number.isInteger` + **hai** bất biến chéo trường (`is_confirmed ⇔ translation`, `han_viet_status ⇔ han_viet_suggestion`). Đọc nó trước khi viết guard mới.
- **Khuôn nối guard vào adapter** — bốn đường đã làm đúng: `glossaryMarksForChapter` (`:341-364`) · `pendingGlossaryCandidates` (`:436-461`) · `listGlossaryEntries` (`:654-673`) · `openGlossaryImportPreview` (`:834-858`). Tất cả dùng `invoke<unknown>` rồi gọi một `isX`. **Đây là hình dạng cần chép, không phát minh hình dạng mới.**
- `:113-130` **`lookupGlossaryTerm`** — `invoke<QuickAddLookupWire>` ở `:115` rồi truy `wire.entry` / `wire.work_tier_available` thẳng. **Không hàm `isQuickAddLookupWire` nào tồn tại trong tệp.** Đây là đường duy nhất trong bảy adapter thiếu guard cho một object lồng ⇒ mục nặng nhất của nhóm.
- `:141-168` **`addGlossaryTerm`** — `invoke<number>` ở `:149`, trả thẳng. · `:519-542` **`approveGlossaryCandidate`** — `invoke<number>` ở `:525`, trả thẳng.
- `:739-754` **`exportGlossaryTier`** — `invoke<string | null>` ở `:741`; chỉ so `path === null`. ⚠️ `null` **là** ca "đã huỷ hộp thoại", giữ nguyên nghĩa đó.
- `:877-894` **`confirmGlossaryImport`** — `invoke<GlossaryImportSummary>` ở `:881`, trả thẳng; hình dạng ba trường số không ai kiểm.
- `:793-809` **`isGlossaryImportPreview`** — bốn trường số (`row_count`, `recognized_column_count`, `new_count`, `identical_count`) chỉ `typeof === 'number'`. Là hàm `isX` **duy nhất** trong tệp thiếu `Number.isInteger`.
- `:613-628` **`isGlossaryEntry`** — đủ trường và có `Number.isInteger(id)`, **thiếu** bất biến chéo trường mà chính doc-comment của `GlossaryEntry` (`:608-610`) khai: `is_shadowed === true ⇒ tier === 'global'`.
- `:74-87` `isIpcError` · `:94-96` `hasIpcBridge()` (`'__TAURI_INTERNALS__' in window`) — gác nhánh `console.error` (lỗi thật) khỏi `console.info` (chạy ngoài Tauri). Guard mới trả lỗi phải đi qua đúng khuôn ba nhánh này. · `UNKNOWN_IPC_ERROR` là hằng lỗi có sẵn cho ca "trượt bằng thứ không phải `IpcError`".
- ⚠️ **Không hàm `isX` nào được `export`** — mọi phép kiểm chỉ chạm được gián tiếp qua hàm `async` công khai. Ca test vì thế phải đi qua biên `@tauri-apps/api/core`.

### Nhóm ② — cờ và nhánh trạng thái

- `src/glossaryImportState.ts:173-190` **`confirmGlossaryImportPreview`** — `if (mySequence !== sequence) return` ở `:181` cắt hàm **trước** `confirming.value = false` ở `:183`. Đường tới lỗ: `openGlossaryImportPreviewOverlay` (`:122`) tăng `sequence` ở `:126`, và nhánh `outcome === 'cancelled'` ở `:140` là nhánh **duy nhất** không chạy tới `:142` `confirming.value = false`. ⇒ cờ kẹt `true`, nút xác nhận khoá hẳn.
  - **Khuôn ĐÚNG:** `glossaryQueueState.ts::openGlossaryQueue` (`:124-136`) reset cờ bận **đồng bộ, trước `await`**, không phụ thuộc nhánh kết quả nào.
- `src/glossaryImportState.ts:196-204` **`cancelGlossaryImportPreview`** — `:201` `void cancelGlossaryImport().then(() => { if (mySequence !== sequence) return })`, thân callback là một no-op; `result.error` không hề được đọc. Hàm anh em ngay trên (`:184`) thì đọc.
  - ⚠️ **Hậu quả có phạm vi hẹp, đo rồi:** wire `glossary_cancel_import` (`src-tauri/src/commands/glossary.rs:1415-1422`) **luôn trả `Ok(())`**, nên `error` chỉ khác `null` khi chính cầu IPC trượt; lô treo bị ghi đè ở lượt mở kế tiếp (`:807`). ⇒ đây là một **chẩn đoán**, không phải một câu cho người dùng — nhưng nó không được nuốt.
- `src/glossaryQueueState.ts:99-104` **`queueEmptyReasonFor`** — `GlossaryQueueStatus = 'unknown' | 'ipc_unavailable' | 'error' | 'no_work' | 'loaded'`; `'unknown'` rơi vào `return null`, **cùng giá trị** với `'error'` và `'loaded'`-có-hàng. Nhánh `'loaded' && rowCount === 0` là **mã chết cho mục đích nó khai**: `rows.value` chỉ bị gán lại ở `openGlossaryQueue` (`:133, :149-153, :170`) và `resetGlossaryQueue` (`:296`); `acceptGlossaryQueueCandidate`/`rejectGlossaryQueueCandidate` (`:223-281`) chỉ đổi `row.outcome`, **không bao giờ co mảng** ⇒ trong một phiên mở, `queueRows.length` là hằng số.
  - `:110-115` **`firstPendingIndexFrom`** đã lọc đúng `outcome === null` — **thước đo cần dùng đã có sẵn trong chính tệp.**
  - `src/GlossaryQueueOverlay.vue:192-197` — nơi gọi **duy nhất**; nó tự thêm `v-if="queueStatus === 'unknown'"` ngày 2026-08-24. ⇒ Vá ở hàm thuần phải **DỜI** mệnh đề đó vào hàm, không nhân đôi nó — hai chỗ cùng canh một mệnh đề là hai nguồn sự thật.
  - `src/i18n/vi.json:315` `glossary.queue.loading` và `:317` `glossary.queue.empty_all_reviewed` (*"Bảng chờ đã sạch — không còn ứng viên nào chờ duyệt."*) — **cả hai câu đã có**, không cần khoá mới cho nhóm này.

### Nhóm ③ — thao tác phá huỷ và cửa loại trừ

- `src/GlossaryManageOverlay.vue:220-227` — `case 'Backspace': case 'Delete':` → `preventDefault()` → `dispatch('glossary.manage.delete')`, một nhịp. `:203-208` `isFormField` (bốn kiểu, kể cả `HTMLButtonElement` từ bản vá 2026-08-24) là cửa duy nhất chặn trước.
- `src/GlossaryManageOverlay.vue:77-83` — watch mở lớp phủ đưa tiêu điểm vào `<section ref="panel" tabindex="-1">`, **không** phải một ô nhập ⇒ `isFormField` trả `false`. `manageCursor` khởi tạo `0` (`glossaryManageState.ts:250`) ⇒ hàng đầu. **Đây là trạng thái mặc định ngay sau khi mở.**
- `src/glossaryManageState.ts:407-430` **`deleteGlossaryManageEntry`** — `if (saving.value) return` · vé `mySequence` · `deleteGlossaryTerm` · `reloadGlossaryManageRowsAfterMutation`. Chốt tái nhập và vé đã đúng; **thiếu đúng một nhịp xác nhận phía trước**.
- **Khuôn preview-trước-khi-ghi cùng lớp phủ:** `glossaryImportState.ts:122-160` (mở xem-trước) → `:173-190` (chỉ khi xác nhận mới ghi). Đường Xoá thiếu hẳn bước một.
- `src/glossaryManageState.ts:388-399` `reloadGlossaryManageRowsAfterMutation` — nạp lại **trọn** danh sách sau một lượt xoá, kẹp con trỏ. Trạng thái chờ-xác-nhận phải tan ở đây.
- `src/GlossaryManageOverlay.vue:451-469` — hai nút: Xuất `:disabled="manageExportBusy"` (`:455`), Nhập `:disabled="importOpening"` (`:464`). **Hai cờ ở HAI module state độc lập**, không đọc chéo.
  - 🔵 **Vế "hai lượt Xuất song song" của mục nợ đã đóng từ trước khi mục nợ được viết:** `glossaryManageState.ts:486` `if (exportBusy.value) return` + `:497` vé `mySequence`, thêm ở Story 3.10b (`5e77e73`). Phía Nhập có cùng khuôn ở `glossaryImportState.ts:123` (ghi chú P9). ⇒ Lỗ thật **chỉ còn** là cửa chéo Xuất↔Nhập.
  - `main.ts:569-570` và `commands/index.ts:1919-1926` — `glossary.manage.export_csv` là đường gọi **duy nhất**; không có cửa sau nào ngoài hai nút.

### Nhóm ④ — `throw` thoát khỏi listener bàn phím

- `src/panels/selectionContract.ts:210` (`currentSelectionText`) **và** `:239` (`currentSelectionTextForGlossaryQuickAdd`) — **hai** chỗ gọi `surface.resolve(selection)` không bọc, không phải một như sổ nợ ghi. `resolve` là callback do panel tự cung cấp (`SourceHanViet.vue`), tức mã ngoài tầm kiểm soát của tệp này.
  - **Khuôn `try/catch` cùng tệp:** `modifySelection` (`:347-374`, khối `try` ở `:369-373`) và `focusSelectionSource` (`:391-412`) — bọc, `console.error` với tiền tố `[selection]`, rồi trả giá trị an toàn.
- `src/commands/registry.ts:200-215` **`dispatch`** — `spec.run()` ở `:214` không bọc. Đường thật tới listener: `keys.ts:593` `addEventListener('keydown', …)` → `handle()` → `keys.ts:523` `registry.dispatch(entry.binding.id)` → `:214`. Chỗ gọi thứ hai: `commands/index.ts:90`.
  - 🔴 `:203-211` — `throw` cho id **chưa đăng ký** là nửa cưỡng chế lúc chạy của AC1, có doc-comment riêng. **Giữ nguyên.**
  - 🔴 `registry.ts` mang banner *"TỆP NÀY KHÔNG ĐƯỢC IMPORT BẤT CỨ THỨ GÌ"* — `check:commands` Kiểm C/D/E `import()` nó bằng Node trần. `try/catch` + `console.error` là cú pháp và global, không phải import ⇒ hợp lệ; mượn một tiện ích log thì không.
  - **Khuôn "ghi chẩn đoán rồi trả false":** `src/commands/focus.ts:150-165` (`enter()`) — `console.error` nêu đích danh chủ thể rồi `return false`, kèm một `try/catch` quanh `resolve()`. Cùng khuôn: `focus.ts:306-350` · `glossaryConfirmStripState.ts:256` · `dockController.ts:69`.

### Nhóm ⑤ — một dòng UI

- `src/GlossaryQuickAdd.vue:191-201` — radio tầng Work chỉ `:disabled="quickAddMode === 'edit'"`, không đọc `quickAddWorkTierAvailable`; nút Lưu (`:233-239`) cũng không.
- **Khuôn ĐÚNG ở component chị em:** `src/GlossaryManageOverlay.vue:443` `:disabled="!manageWorkTierAvailable"` trên đúng radio tương ứng. Chép sang.
- ⚠️ Hậu quả là **UX, không phải hỏng dữ liệu**: `saveGlossaryQuickAdd` (`glossaryQuickAddState.ts:289-317`) gửi `tier='work'`, Rust trả `GlossaryError::WorkTierUnavailable` ⇒ `vi.json:26` hiện câu lý do. 0 lượt ghi thật, nhưng một round-trip vô ích và một câu lỗi cho việc lẽ ra không bấm được.

### Hạ tầng test và cổng

- `tests/frontend/` — **39 tệp, 479 ca, xanh trọn, 11,28 s** (đo 2026-08-26 trên `0f07184`). Đây là con số **trước**.
- **Khuôn mock đúng biên** (giữ `config/glossary.ts` nguyên vẹn): `tests/frontend/glossaryConfirmStrip.test.ts` (21 ca) · `glossaryConfirmStripTemplate.test.ts` (9) · `glossaryConfirmStripSuggestion.test.ts` (5) — ba tệp duy nhất mock `@tauri-apps/api/core`. **Ca của nhóm ① đi theo khuôn này.**
- **Tệp mount component thật:** `glossaryManage.test.ts` (24 ca) · `glossaryQueue.test.ts` (22-26 ca) · `glossaryQuickAddStrip.test.ts` (15 ca, 27 lượt `mount`) — nhóm ③ và ⑤ vào đây.
- ⚠️ **`glossaryManage.test.ts:418-447` khoá hành vi xoá-một-nhịp hiện tại** (`trigger Backspace` ⇒ `expect(deleteHandler).toHaveBeenCalledTimes(1)`). Bản vá D10 **sẽ làm ca này đỏ** — đó là ca cần **sửa cho khớp hành vi mới**, không phải một tín hiệu dừng; ca sửa xong phải kiểm **cả hai nhịp**.
- `tests/frontend/glossarySelectionContract.test.ts` (3 ca) — chỗ của nhóm ④, vế `selectionContract`.
- `tests/frontend/support/setup.ts` (88 dòng) — ba mục vá `happy-dom`: `document.fonts` · `ResizeObserver` · `document.execCommand`. Thiếu gì thì thêm **ở đây**, không thêm `?.` vào `src/`.
- **Cổng có thể đỏ vì lượt vá này:** `check:commands` (Kiểm C/D/E `import()` `registry.ts` bằng Node trần và gọi `dispatch()` thật — mọi `import` mới ở đó giết ba phép kiểm cùng lúc) · `check:i18n` (khoá `vi.json` mới) · `check:tokens` (trạng thái chờ-xác-nhận nếu lỡ dùng màu viết thẳng hoặc `opacity`) · `check:panel-refs` (ô nhớ cấp module mới phải có đường `reset*()`).
- `src/i18n/vi.json` — **109 khoá** tiền tố `glossary.` hôm nay. **Không có khoá nào cho một câu xác nhận xoá**; `:350` `glossary.manage.delete` chỉ là nhãn nút.

## Tasks & Acceptance

**Execution:**
- [x] `src/config/glossary.ts` — dựng guard cho **`QuickAddLookupWire`** (và kiểu mục lồng của nó) rồi nối vào `lookupGlossaryTerm`; đây là đường duy nhất trong bảy adapter chưa kiểm một object lồng, và nó nuôi Quick Add — bề mặt người dùng chạm nhiều nhất của Epic 3.
- [x] `src/config/glossary.ts` — dựng guard cho **`GlossaryImportSummary`** rồi nối vào `confirmGlossaryImport`; ba trường số của nó chảy thẳng ra câu tóm tắt hiện cho người dùng.
- [x] `src/config/glossary.ts` — kiểm lúc chạy cho ba giá trị trần: id trả về của `addGlossaryTerm` và `approveGlossaryCandidate` (số nguyên), đường dẫn của `exportGlossaryTier` (chuỗi) — **giữ nguyên `null` là "đã huỷ hộp thoại"**, một id không phải số nguyên đi tiếp sẽ thành khoá tra cứu cho mọi lượt sửa/xoá về sau.
- [x] `src/config/glossary.ts` — `isGlossaryImportPreview` thêm `Number.isInteger` cho bốn trường số; `isGlossaryEntry` thêm bất biến chéo trường `is_shadowed ⇒ tier === 'global'` mà doc-comment của chính kiểu đó đã khai.
- [x] `src/glossaryImportState.ts` — cờ `confirming` phải hạ được ở **mọi** nhánh của lượt mở lại, gồm nhánh `'cancelled'`; và `cancelGlossaryImportPreview` phải **đọc** `result.error` rồi ghi một chẩn đoán nêu đích danh — hôm nay thân callback là một no-op.
- [x] `src/glossaryQueueState.ts` — `queueEmptyReasonFor` cấp một ca **có tên** cho `'unknown'`, và đo "đã duyệt hết" bằng **số hàng chưa xử lý** thay vì `rows.length`; đồng thời `GlossaryQueueOverlay.vue` **dời** mệnh đề `'unknown'` của nó vào hàm thay vì giữ hai chỗ cùng canh một mệnh đề.
- [x] `src/glossaryManageState.ts` + `src/GlossaryManageOverlay.vue` + `src/i18n/vi.json` — nhịp xác nhận xoá hai bước: nhịp một chỉ đổi trạng thái và nói ra (0 lượt IPC), nhịp hai mới ghi; dời con trỏ, `Escape`, vào chế độ sửa, hay một lượt nạp lại đều làm trạng thái chờ tan. Xoá vĩnh viễn không hoàn tác được thì một phím lỡ tay không được là toàn bộ khoảng cách tới nó.
- [x] `src/GlossaryManageOverlay.vue` (+ hai module state) — Xuất và Nhập loại trừ lẫn nhau; hai cửa hộp thoại hệ điều hành cùng bay là một trạng thái không ai định nghĩa hành vi.
- [x] `src/panels/selectionContract.ts` — bọc **cả hai** chỗ gọi `surface.resolve(selection)` theo đúng khuôn `modifySelection` cùng tệp; `resolve` là mã của panel, và cả hai hàm này chạy dưới một lượt `dispatch` từ bàn phím.
- [x] `src/commands/registry.ts` — bọc `spec.run()`, ghi chẩn đoán nêu đích danh `id`, **giữ nguyên** `throw` cho id chưa đăng ký; không thêm một dòng `import` nào.
- [x] `src/GlossaryQuickAdd.vue` — radio tầng Tác phẩm `disabled` khi `quickAddWorkTierAvailable === false`, chép khuôn `GlossaryManageOverlay.vue:443`.
- [x] `tests/frontend/**` — ca cho ①…⑰; nhóm ① mock đúng biên `@tauri-apps/api/core`, nhóm ③/⑤ mount component thật. Sửa `glossaryManage.test.ts:418-447` cho khớp hành vi hai nhịp, kiểm **cả hai** nhịp. Mỗi mục kèm một phép đối chứng gỡ-chỗ-nối **đã chạy thật**.
- [x] `_bmad-output/implementation-artifacts/deferred-work.md` — đóng mục **Cụm D** bằng chữ, và đóng mục **"hai lượt xuất song song"** bằng `→ ✅` kèm tên chỗ gác đã có sẵn; hai mục bị bác đóng bằng `→ KHÔNG LÀM 2026-08-26 — <lý do kèm số đo>`. Không xoá mục cũ, không làm tròn lên.

**Acceptance Criteria:**
- Given mỗi mục vá, when gỡ riêng bản vá đó ra và chạy bộ test, then **có ít nhất một ca đỏ trỏ đúng mục đó**; tên ca và số ca đỏ ghi lại thành số thật, không suy.
- Given sáu guard mới/sửa của nhóm ①, when một ca cấp dữ liệu **đúng hình dạng**, then adapter trả `error: null` như trước — bản vá thu hẹp cửa vào, không đổi hành vi đường thường của bảy adapter.
- Given mọi ca của nhóm ①, when đọc tệp test, then nó mock `@tauri-apps/api/core` chứ **không** mock module `src/config/glossary` — một ca mock trọn module là một ca không chạy guard nào.
- Given hai mục đã bị đo bác, when đọc diff, then **0 dòng** thay đổi ở `src/glossaryQuickAddState.ts` và `src/panels/glossaryMarksState.ts`.
- Given bộ test frontend, when chạy `npm run test`, then xanh trọn và số ca **tăng** so với **479** của lượt đo 2026-08-26; ghi con số trước/sau thật.
- Given toàn bộ bản vá, when chạy `.githooks/pre-push`, then exit 0 — kèm ghi nhận rằng nó chạy trên macOS và **không** nói gì về nửa Windows.

## Spec Change Log

### 2026-08-26 — §I/O Matrix thiếu một hàng: một lượt TỰ LẶP của bàn phím cũng là "lần hai"

**Phát hiện kích hoạt:** lăng kính blind-hunter, vòng rà ba lớp sau lượt thi hành. `onKeydown`
của `GlossaryManageOverlay.vue` không kiểm `event.repeat` (đo: **0** lần xuất hiện trong tệp),
và nhịp một của `deleteGlossaryManageEntry` trả về **đồng bộ, không đặt cờ bận nào**. Giữ phím
`Backspace` ⇒ hệ điều hành tự lặp `keydown` ⇒ nhịp một rồi nhịp hai chạy từ đúng **một** cú
nhấn mà người dùng cảm nhận là một lần bấm. Tức bản vá D10 **không đóng** được đúng kịch bản
nó tồn tại để chặn.

**Vì sao đây KHÔNG phải một chỗ mã lệch spec:** hàng **⑫** của §I/O Matrix viết *"Backspace lần
hai trên đúng hàng đó"* — một lượt tự lặp **đúng là** "lần hai" theo chữ. Mã khớp matrix; matrix
là thứ thiếu một hàng. Gốc rễ vì thế nằm **trong** khối `<frozen-after-approval>`, nơi chỉ Ice
sửa được.

**Ice chốt 2026-08-26: vá tại chỗ, KHÔNG quy hoạch lại.** Hình dạng mã đã đúng và mười bốn chỗ
vá kia đang xanh; hoàn nguyên trọn bộ để dựng lại vì một điều kiện thiếu là một cái giá không
tương xứng. Hàng còn thiếu ghi ở đây thay vì sửa chữ đã đóng băng:

> **⑫b** — giữ phím `Backspace`/`Delete` (một cú nhấn, hệ điều hành tự lặp `keydown`) ⇒ **0
> lượt xoá**. Nhịp hai chỉ được tính khi nó đến từ một sự kiện bàn phím **không** mang
> `repeat`. `ArrowUp`/`ArrowDown` **giữ nguyên** khả năng tự lặp — điều kiện đặt ở nhánh
> `Backspace`/`Delete`, không đặt ở đầu `onKeydown`.

**Trạng thái xấu mà mục này chặn:** một nhịp xác nhận trông như đã có nhưng một cú nhấn-giữ đi
xuyên qua được — tệ hơn không có xác nhận, vì nó khiến cả người dùng lẫn người rà tin rằng thao
tác phá huỷ đã được canh.

**KEEP — phải sống sót qua mọi lượt dựng lại:**
- Nhịp một **0 lượt IPC** và trạng thái chờ tan khi dời con trỏ · `Escape` · vào chế độ sửa ·
  nạp lại danh sách (hàng ⑪ ⑫ ⑬ đã đo và xanh).
- `Escape` huỷ nhịp một mà **không** đóng lớp phủ (`onEscape` hiện tại) — đây là hành vi §I/O ⑬
  cố ý đòi, **không** phải một chỗ hở; lăng kính edge-case nêu nó và nó đã bị **bác**.
- `promoteGlossaryManageEntry` đã dọn `deletePendingKey` (`glossaryManageState.ts:446`) — lăng
  kính edge-case nêu, cũng **bác**.
- Hai mục đã bị đo bác ở bước quy hoạch giữ nguyên **0 dòng đổi**.

### 2026-08-26 (muộn hơn) — một AC ghi "0 dòng" nay hết đúng theo CHỮ, còn đúng theo NGHĨA

**Phát hiện kích hoạt:** mục **#6** của vòng rà thứ hai (Quick Add vẫn gửi `tier: 'work'` khi
khả dụng lật giữa phiên) buộc phải chạm `src/glossaryQuickAddState.ts` — đúng một trong hai tệp
mà §Acceptance Criteria đòi **"0 dòng thay đổi"**.

**Đối chứng, không suy luận:** `git diff` của tệp đó là **+18 dòng, thuần một `computed`
`quickAddWorkTierBlocked`**. Nó **không** chạm `saveGlossaryQuickAdd`, **không** thêm vé
`sequence`, **không** đụng ba cửa chặn đồng bộ (`saving`/`isOpen`) là căn cứ để bác mục nợ.

⇒ **Mệnh đề bị bác vẫn bị bác.** AC ấy sinh ra để chặn một việc cụ thể — dựng vé `sequence` cho
một kịch bản không tới được — và việc đó vẫn chưa ai làm. Cái hết đúng là **thước đo** ("0 dòng
ở cả tệp"), không phải **điều nó bảo vệ**. Thước đúng hôm nay: *0 dòng chạm
`saveGlossaryQuickAdd` và 0 vé `sequence` mới trong `glossaryQuickAddState.ts`*; còn
`src/panels/glossaryMarksState.ts` thì **vẫn đúng nguyên nghĩa đen — 0 dòng**, nó không có mặt
trong diff.

⚠️ Ghi ra đây thay vì sửa §Acceptance Criteria cho khớp mã: một AC đã dùng để phán quyết thì
lịch sử của nó là bằng chứng cho lượt sau, và sửa lặng lẽ một tiêu chí cho vừa kết quả là đúng
thứ lượt rà này tồn tại để bắt.

## Design Notes

**Vì sao hai mục bị bác, viết ra để không ai đo lại lần ba:**

- **Vé `sequence` cho `saveGlossaryQuickAdd`.** Kịch bản sổ nợ mô tả (*"một lượt lưu cũ trả về muộn đóng dải và gán nhầm lỗi cho thuật ngữ vừa gõ"*) đòi dải phải **đóng rồi mở lại** trong lúc một lượt lưu đang bay. Ba cửa chặn điều đó, cả ba đều đồng bộ: `saveGlossaryQuickAdd:298` `if (saving.value) return` · `closeGlossaryQuickAdd:277` `if (saving.value) return` · `openGlossaryQuickAdd:217` `if (isOpen.value) return`. Cửa thứ hai có doc-comment riêng nói thẳng *"không có ca nào cần `Esc` phải thắng một lượt ghi đang bay"*. Thêm một vé `sequence` hôm nay là thêm một nhánh mà kiểu nói **không bao giờ chạy**.
- **Đua id Chương ở `glossaryMarksState`.** Sổ nợ khai *"`requestedForChapterId` nằm lại `null` và KHÔNG đường nào thử lại"*. Đo được điều **ngược lại**: `:105` `if (loaded === null) requestedForChapterId = null` tồn tại từ commit tạo tệp (`53035e7`, Story 3.4b) và doc-comment ngay trên nó nói rõ mục đích — `ensureGlossaryMarksLoaded:118` `if (requestedForChapterId === chapterId) return` chỉ chặn khi hai giá trị **bằng nhau**, nên `null` là đúng thứ **mở** đường thử lại. Mọi lượt ghi `sequence`/`requestedForChapterId` đều nằm **trước** `await`, và JS đơn luồng ⇒ không có kẽ hở trong chính tệp. Hai chỗ gọi ở `editorPanelState.ts` (`:1578-1584`, `:2065-2071`) đã có guard khớp nhau.

**Vì sao nhánh `'unknown'` vẫn vá dù nơi tiêu thụ đã chặn:** phòng thủ thuộc về nơi **sở hữu dữ kiện**, không nơi tiêu thụ. `GlossaryQueueOverlay.vue:192-197` là nơi gọi duy nhất **hôm nay**; `queueEmptyReasonFor` là hàm `export` nên nơi gọi thứ hai không cần xin phép ai. Nhưng vá ở hàm mà **để nguyên** `v-if` ở template là dựng hai nguồn sự thật — nên mệnh đề phải **dời** vào hàm, không nhân đôi.

**Nhịp xác nhận xoá — hình dạng Ice chốt 2026-08-26.** Hai nhịp trong cùng lớp phủ, không component hộp thoại mới. Lý do loại ba phương án kia: bỏ hẳn hợp âm hạ sàn bàn phím của AD-34 §1 (một thao tác không còn gán được phím); thu hẹp tiêu điểm về listbox chỉ làm khó lỡ tay chứ không đóng được vế *"không hoàn tác"*; dựng một `ConfirmDialog` dùng chung là một quyết định kiến trúc — kho có **bảy** lớp phủ tự cài `role="dialog"` riêng và không cái nào chung gốc, nên cái đầu tiên phải đi qua thủ tục viết ra, không qua một lượt tiện tay.

## Verification

**Commands:**
- `npm run test` — expected: exit 0; số ca **tăng** so với **479** (đo 2026-08-26, `0f07184`). Ghi con số thật trước/sau.
- `npm run check:i18n` — expected: exit 0; số khoá `vi.json` tăng đúng bằng số câu mới của nhịp xác nhận, 0 miễn trừ mới.
- `npm run check:commands` — expected: exit 0. ⚠️ Chạy **ngay sau** khi chạm `registry.ts` và **trước** khi chạy tiếp — Kiểm C/D/E `import()` tệp đó bằng Node trần.
- `npm run check:tokens` · `npm run check:panel-refs` — expected: exit 0.
- `npm run build` — expected: exit 0. Chạy **TRƯỚC** `cargo test`.
- `.githooks/pre-push` — expected: exit 0. ⚠️ Chạy trên macOS của Ice; đọc lượt CI trước khi kết luận là xanh.

**Manual checks (if no CLI):**
- **Mười bốn phép đối chứng gỡ-chỗ-nối, chạy thật, một mục một lượt:** gỡ riêng từng bản vá, chạy bộ test, ghi **tên ca đỏ** và **số ca đỏ**, rồi khôi phục. Một mục mà bộ test vẫn xanh trọn nghĩa là ca của nó chưa chạm bề mặt — **sửa ca, không sửa kết luận**.
- **Đối chứng chiều ngược cho nhóm ①:** với mỗi guard mới, chạy một ca cấp dữ liệu **đúng** hình dạng và khẳng định `error === null`. Một guard chỉ biết nói "không" là một guard chưa ai chứng minh là không nói oan.
- **Đối chứng ca bị sửa:** chạy `glossaryManage.test.ts` **trước** khi sửa ca `:418-447` và ghi lại rằng nó đỏ vì đúng lý do (một nhịp không còn xoá), không vì một lý do khác.

## Suggested Review Order

**Thao tác phá huỷ — đọc trước hết, đây là chỗ một lỗi ở lại vĩnh viễn**

- Điểm vào: nhịp một chỉ đổi trạng thái, 0 lượt IPC; nhịp hai mới ghi.
  [`glossaryManageState.ts:471`](../../src/glossaryManageState.ts#L471)

- Lỗ vòng rà bắt được: giữ phím tự lặp từng đi xuyên qua cả hai nhịp.
  [`GlossaryManageOverlay.vue:246`](../../src/GlossaryManageOverlay.vue#L246)

- Câu hint phải gọi đúng tên nhãn nút đang hiển thị, không tên cũ.
  [`vi.json:353`](../../src/i18n/vi.json#L353)

**Cửa loại trừ Xuất↔Nhập — vì sao một module thứ ba, không phải import chéo**

- Ô nhớ đứng ngoài cả hai state; đồ thị import hai cạnh vào, không cạnh ra.
  [`glossaryExchangeGate.ts:1`](../../src/glossaryExchangeGate.ts#L1)

- Chiều Xuất: kiểm vé cũ TRƯỚC khi hạ cờ dùng chung.
  [`glossaryManageState.ts:561`](../../src/glossaryManageState.ts#L561)

- Nút bị khoá phải nói VÌ SAO, không chỉ xám đi.
  [`GlossaryManageOverlay.vue:532`](../../src/GlossaryManageOverlay.vue#L532)

**Guard lúc chạy trên dây — sáu đường, một khuôn**

- Đường nặng nhất: object lồng, trước đây 0 phép kiểm nào.
  [`glossary.ts:102`](../../src/config/glossary.ts#L102)

- Chỗ nối guard vào adapter; `invoke<unknown>` thay cho generic tin tưởng.
  [`glossary.ts:147`](../../src/config/glossary.ts#L147)

- Bất biến chéo trường mà doc-comment của kiểu đã khai từ trước.
  [`glossary.ts:677`](../../src/config/glossary.ts#L677)

- `null` vẫn là "đã huỷ hộp thoại"; chuỗi rỗng thì không phải thành công.
  [`glossary.ts:804`](../../src/config/glossary.ts#L804)

**Cờ và nhánh trạng thái — rỗng phải nói vì sao nó rỗng**

- Cờ hạ đồng bộ ngay khi lượt mở mới bắt đầu, không đợi biết `outcome`.
  [`glossaryImportState.ts:152`](../../src/glossaryImportState.ts#L152)

- Ca "đang nạp" có tên riêng; "đã duyệt hết" đo bằng số hàng chưa xử lý.
  [`glossaryQueueState.ts:122`](../../src/glossaryQueueState.ts#L122)

**Ngoại lệ dưới listener bàn phím — và giới hạn thật của bản vá**

- Bọc `spec.run()`; `throw` cho id chưa đăng ký giữ nguyên phía trên.
  [`registry.ts:239`](../../src/commands/registry.ts#L239)

- Hai chỗ gọi `resolve()` của panel, không một chỗ như sổ nợ ghi.
  [`selectionContract.ts:217`](../../src/panels/selectionContract.ts#L217)

**Một dòng UI**

- Chặn lượt gửi khi tầng Tác phẩm lật thành không khả dụng giữa phiên.
  [`GlossaryQuickAdd.vue:238`](../../src/GlossaryQuickAdd.vue#L238)

**Phần đỡ — test và sổ nợ**

- Ca nhóm ① mock đúng biên IPC nên guard sản phẩm chạy thật.
  [`glossaryConfigGuards.test.ts:46`](../../tests/frontend/glossaryConfigGuards.test.ts#L46)

- Ca đọc DOM đã mount, không chỉ đọc `ref` của tầng state.
  [`glossaryManage.test.ts:1`](../../tests/frontend/glossaryManage.test.ts#L1)

- Hai mục bị bác đóng bằng chữ kèm số đo, không bằng mã.
  [`deferred-work.md:7201`](./deferred-work.md#L7201)
