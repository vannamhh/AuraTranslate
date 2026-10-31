---
title: 'Các lệnh lẻ có đường chuột'
type: 'feature'
ticket: '9'
created: '2026-10-31'
status: 'built'
baseline_revision: '0fd73da9f004dbcf0f5f6602568e92078b21ef90'
route: 'full'
route_source: 'auto'
risk: 'high'
review: 'quick'
review_source: 'pinned'
lenses_ran: ['quick']
review_loop_iteration: 0
context:
  - '{project-root}/_bmad-output/initiative-auratranslate/tickets/epic-san-chuot/ux-san-chuot/EXPERIENCE.md'
  - '{project-root}/_bmad-output/initiative-auratranslate/tickets/epic-san-chuot/ux-san-chuot/DESIGN.md'
---

<frozen-after-approval reason="human-owned intent — do not modify unless human renegotiates">

## Intent

**Problem:** Tám id cuối của `POINTER_DEBT` chưa có điểm chuột: bốn không có nút, bốn có nút nhưng đi qua hàm nên Kiểm M không tính. Viết lại thẳng thuộc tính thì mất dữ liệu ở hai chỗ: `prompt.library.close` bỏ lượt ghi, và hàng TM mất chỉ số nên Sửa hoặc Xoá rơi vào hàng đang nhắm thay vì hàng vừa bấm.

**Approach:** Thêm bốn nút theo bảng Story 14.9 ở `EXPERIENCE.md`; viết bốn `@submit` thành `dispatch` literal; hàng TM nhắm bằng `@mousedown` và `@focusin` theo khuôn `GlossaryHarvestOverlay.vue`; lượt ghi prompt chuyển vào `closePromptLibrary()`. `POINTER_DEBT` rỗng.

## Boundaries & Constraints

**Always:** `@click`/`@submit` là đúng `dispatch('<id>')` literal. Bốn nút mới có `:title="commandTitle('<id>')"`. Giữ `<form>` của bốn nút có sẵn (`.tm-row-acts form` là `display: contents`, test bấm `.pl-row-form`). `POINTER_DEBT` còn là mảng rỗng, không xoá hằng; `POINTER_DEBT_BASELINE` không đổi. `closePromptLibrary()` chỉ đóng khi lượt ghi trả `true`.

**Never:** Sửa `aimTmManageRow`, `hideProofreadStrip`, `updatePromptSetBody`. Chuyển bản nháp prompt ra khỏi `PromptLibraryOverlay.vue`. Đổi `keys` của command nào. Đổi `.wp-sel-hint` hay `proofread.strip.hint_hide`. Khai `pointer` thay cho điểm chuột ở 8 id này.

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected Output / Behavior | Error Handling |
|----------|--------------|---------------------------|----------------|
| Đóng prompt khi có sửa | Sửa thân, bấm Đóng ngay | Thân được ghi rồi lớp phủ đóng | Không lỗi |
| Ghi lỗi | `updatePromptSetBody` báo lỗi, qua nút Đóng hoặc Esc | Lớp phủ mở, nháp còn, lỗi hiện | Đúng `promptSetActionError` |
| Phím tự gán | `prompt.library.close` đi thẳng từ `dispatch` | Cùng đường ghi như nút | Như trên |
| Sửa hàng khác | Con trỏ ở A, bấm Sửa hàng B | Sửa B | Không lỗi |
| Xoá hai nhịp | Bấm Xoá hàng B hai lần | Lần một chờ xác nhận ở B, lần hai xoá B | Bấm hàng C giữa chừng huỷ nhịp chờ |
| Bàn phím tới nút | Tab tới Xoá hàng B rồi Enter | Nhắm B qua `@focusin`, xoá đúng B | Không lỗi |
| Đang sửa hoặc đang lưu | Bấm vào hàng khác | Con trỏ không đổi | `aimTmManageRow` từ chối |
| Hàng bị xén | Hàng sát mép dưới danh sách, bấm Sửa | Cú bấm tới, danh sách không cuộn dưới con trỏ | Không lỗi |
| Không ẩn được | `proofreaderStripPlace !== 'track'` | Nút Ẩn tắt (R9) | Không im lặng |
| Ẩn dải | Caret ở câu A, bấm Ẩn | Dải ẩn, gõ tiếp vào đúng chỗ caret | Không lỗi |
| Nút biến mất | Bấm Huỷ, hoặc Mục trước về mục đầu | Phím của lớp phủ còn chạy | Tiêu điểm về panel nếu rơi về `body` |

</frozen-after-approval>

## Code Map

- `src/ProofreaderStrip.vue:192` -- `.pfs-head`. `.pfs-hint` giữ `margin-right: auto`, nên không có hint thì nút không sát phải. `proofreaderStripPlace` (`src/inlineStripEligibility.ts:54`) là `'track'` đúng khi `hideProofreadStrip` có tác dụng: `eligibleInlineStrips` bỏ qua cờ ẩn ở trạng thái quét, gồm cả `done` với `total === 0`, ca hàm không tự từ chối.
- `src/proofreadState.ts:281` -- chỉ trả tiêu điểm khi tiêu điểm đang trong dải, nên chuột không được kéo tiêu điểm khỏi ô.
- `src/modes/LibraryWorkPage.vue:715` -- `.wp-foot`, `selectedCount`.
- `src/modes/ReadingMode.vue:952` -- `.toc-actions`, `readingMarksClearPending`, ref `marksPanel` có `tabindex="-1"`; `onMarksKeydown` bỏ qua sự kiện từ phần tử con.
- `src/AlignmentOverlay.vue:267` -- `.al-actions`, `alignmentCursor`, `panel` có `tabindex="-1"`; `trapTab` và `onKeydown` nằm trên gốc lớp phủ nên `body` làm chúng chết. Nhãn dùng `command.export.alignment.prev` có sẵn.
- `src/TmManageOverlay.vue:299,676,722` -- `onRowEdit`, `onRowDelete`, `<li class="tm-row">`, hai `<form>`. `watch([tmManageCursor, tmManageFlatRows])` dòng 128 gọi `scrollIntoView` ở `nextTick`, tức giữa `mousedown` và `mouseup`.
- `src/tmManageState.ts:445` -- `aimTmManageRow` đã từ chối khi đang lưu, đang sửa, trùng hàng; `moveTo` huỷ nhịp chờ xoá.
- `src/GlossaryHarvestOverlay.vue:163` -- khuôn `@mousedown` + `@focusin` trên hàng.
- `src/PromptLibraryOverlay.vue:313-326,390,484,602,634,763` -- `flushBody`, `onOpenImport`, `onClose`, `onEscape`, hai `<form>`; comment dòng 17–18 sai vì nói Đóng là `@click`.
- `src/promptLibraryState.ts:38` -- `closePromptLibrary`, `resetPromptLibrary`; `src/main.ts:1088` nối vào deps. `resetPromptLibrary` chỉ test gọi. `let` cấp module phải có đường `reset*()` (`check:panel-refs`).
- `src/config/shortcutsState.ts:639` -- `commandTitle`.
- `scripts/check-commands.mjs:2516,2520,2600,2601,2908,2968` -- bốn hàng Kiểm K cần gỡ, `POINTER_DEBT`, `POINTER_FLOOR`; `R_CURSOR_LOCAL` là lý do của `aimRow`.
- `tests/frontend/promptLibraryOverlayRender.test.ts:717-729` -- ca đóng dùng `closeLibrary` mock nên sẽ xanh giả sau story; `dispatch` tới mock mà không ghi.

## Tasks & Acceptance

**Execution:**
- [x] `src/promptLibraryState.ts` -- thêm `let` giữ hàm kiểm rời, `setPromptLibraryLeaveCheck(check)`; `closePromptLibrary()` thành async, chờ hàm kiểm rồi mới đặt hai cờ; `resetPromptLibrary()` xoá hàm kiểm -- lượt ghi nằm trên đường của command, kể cả phím tự gán.
- [x] `src/main.ts` -- bọc `void closePromptLibrary()` trong deps nếu `check:lint` đòi -- `run` là `() => void`.
- [x] `src/PromptLibraryOverlay.vue` -- đăng ký `flushBody` ở `onMounted`, gỡ ở `onUnmounted`; xoá `onClose`, `onOpenImport`; `onEscape` gọi thẳng `dispatch('prompt.library.close')` ở cuối; hai `@submit.prevent` thành literal; sửa comment dòng 17–18.
- [x] `src/TmManageOverlay.vue` -- xoá `onRowEdit`, `onRowDelete`; hai `@submit.prevent` thành literal; `<li class="tm-row">` thêm `@mousedown="onRowMouseDown(i)"` và `@focusin="aimTmManageRow(i)"`; `onRowMouseDown` ghi hàng nhắm bằng chuột để `watch` dòng 128 bỏ qua `scrollIntoView` đúng cho hàng đó, cờ không được còn sót khi `aimTmManageRow` từ chối. `watch(tmManageFlatRows)` trả tiêu điểm về `list` khi còn hàng và tiêu điểm lạc -- bàn đo: hàng vừa xoá mang tiêu điểm đi theo.
- [x] `src/ProofreaderStrip.vue` -- thêm nút Ẩn cuối `.pfs-head`, `@mousedown.prevent`, `:disabled="proofreaderStripPlace !== 'track'"`; bỏ `margin-right: auto` của `.pfs-hint`, gom Huỷ, Bộ nhớ, Ẩn vào một cụm `margin-left: auto` -- nút sát phải ở mọi trạng thái.
- [x] `src/modes/LibraryWorkPage.vue` -- nút Bỏ chọn sau `.wp-sel-count`, `v-if="selectedCount > 0"`; `watch` `flush: 'post'`: số chọn về 0 và tiêu điểm ở `body` hoặc gốc `.wp` thì `listRef` nhận tiêu điểm -- bàn đo: WKWebView chuyển tiêu điểm lên gốc và `onListKeydown` chỉ nghe trên danh sách.
- [x] `src/modes/ReadingMode.vue` -- nút Huỷ sau nút bỏ hết, `v-if="readingMarksClearPending"`; `watch` `flush: 'post'`: pending về `false` và `activeElement === body` thì `marksPanel` nhận tiêu điểm.
- [x] `src/AlignmentOverlay.vue` -- nút Mục trước trước nút xem tiếp, `v-if="alignmentCursor > 0"`; `watch` tương tự khi cursor về 0, đích là `panel`.
- [x] `src/i18n/vi.json` -- `proofread.strip.hide` "Ẩn", `library.work_page.clear_selection` "Bỏ chọn", `mode.reading.marks_clear_cancel` "Huỷ".
- [x] `scripts/check-commands.mjs` -- gỡ `onOpenImport`, `onClose`, `onRowEdit`, `onRowDelete` khỏi bảng Kiểm K; thêm `src/TmManageOverlay.vue::onRowMouseDown` với `R_CURSOR_LOCAL`; `POINTER_DEBT` rỗng; `POINTER_FLOOR = 253` (ceil 0,85 × 297).
- [x] `tests/frontend/proofreaderStripContent.test.ts` -- ca (i); nút tắt ở `scanning`, `error`, `not_configured`, `done` với `total === 0`.
- [x] `tests/frontend/proofreaderStripSlots.test.ts` -- thêm `commandTitle` vào mock `shortcutsState` -- `ProofreaderStrip.vue` gọi nó lúc render.
- [x] `tests/frontend/libraryChapters.test.ts`, `tests/frontend/readingMarks.test.ts`, `tests/frontend/alignment.test.ts` -- ca (i), ca hiện/ẩn của nút; hai ca tiêu điểm: nút đang focus biến mất thì tiêu điểm về panel.
- [x] `tests/frontend/tmManage.test.ts` -- ca (i) cho `edit`, `delete` bằng `mousedown` rồi submit trên hàng khác hàng con trỏ; bốn hàng Xoá hai nhịp, Bàn phím, Đang sửa, Hàng bị xén của Matrix.
- [x] `tests/frontend/promptLibraryOverlayRender.test.ts` -- đổi ca 717–729 sang handler thật `closePromptLibrary`; thêm ca Phím tự gán và ca (i) cho `prompt.import.open`.
- [x] `_bmad-output/implementation-artifacts/14-9-ban-do/` -- bàn đo chuột thật theo khuôn 14-8: bấm đủ 8 nút; hàng TM xén sát mép; bấm Ẩn rồi gõ; `activeElement` sau ba nút biến mất.
- [x] `_bmad-output/initiative-auratranslate/deferred-work.md` -- đóng mục Story 14.9 ở dòng 13679; mọi lỗi bàn đo chưa sửa thành mục `Chủ: Epic 14`.

**Acceptance Criteria:**
- Given cây thật, when chạy `npm run check:commands`, then Kiểm M đọc 297 điểm chuột, 33 khai `pointer`, 0 nợ và xanh.
- Given gỡ thuộc tính khỏi từng nút, when chạy ca (i) của id đó, then ca đỏ vì `dispatch('<id>')` không chạy.
- Given gỡ lượt chờ hàm kiểm khỏi `closePromptLibrary()`, when chạy ca ghi lỗi, then ca đỏ vì lớp phủ đã đóng.
- Given gỡ `@mousedown` khỏi hàng TM, when chạy ca Sửa hàng khác, then ca đỏ vì sửa nhầm hàng con trỏ.
- Given app thật, when bấm từng nút bằng chuột thật, then mọi cú đủ `mousedown`, `mouseup`, `click`, và sửa, xoá đúng hàng vừa bấm.

## Implementation Notes

- Kiểm M đọc 297/33/0: `POINTER_DEBT` rỗng, đạt Done when 1 của epic.
- `proofreaderStripPlace === 'track'` khớp đúng các ca `hideProofreadStrip` có tác dụng, kể cả `done` với `total === 0`, nơi hàm không tự từ chối mà dải vẫn còn.
- Bàn đo bắt lỗi plan chưa nêu: bấm Bỏ chọn đưa tiêu điểm lên gốc `.wp` nên phím của danh sách Chương chết; `watch` nay trả tiêu điểm về `listRef`, nhận cả `body` cho ca nút giữ tiêu điểm rồi biến mất.
- Review tìm lỗi Xoá TM làm mọi phím của lớp phủ chết, có từ trước story; bàn đo đọc `BODY` và con trỏ 2 → 2 trước bản vá, `UL.tm-list` và 2 → 3 sau bản vá.
- AC 5 "mọi cú" không đạt ở mọi lượt: lượt chốt 22/22, ba lượt xanh trước có 0/22, 1/23, 3/25 cú mất, nguyên nhân chưa xác định, đã nối vào mục `Chủ: Epic 14`.
- Danh sách Vitest của plan bỏ sót `proofreaderStripSlots.test.ts`: mock thiếu `commandTitle` đỏ 9 ca và 51 lỗi, chỉ lượt đủ bộ lộ ra.
- Đối chứng đỏ đã chạy: gỡ thuộc tính khỏi từng nút trong 8 id, mỗi phép đỏ 1–3 ca của tệp đó; gỡ `@mousedown` hàng TM (3 ca), lượt chờ ở `closePromptLibrary()` (3 ca), điều kiện tiêu điểm Bỏ chọn (2 ca), nhánh tiêu điểm sau Xoá (1 ca).
- `cargo test` ghi đè binary `wdio`: bàn đo sau đó báo `Embedded WebDriver server did not become ready on port 4445`, dịch lại `--features wdio` là hết.
- Lượt đêm `41134334441` đỏ trên `5e488139`, cũ hơn diff này 14 commit; đã ghi mục `Chủ: Epic 14`.

## Plan Change Log

## Review Triage Log

Lượt 1 (lens Quick): high 0 · medium 4 · low 2 · false 0 · maybe-false 0.

- low, patch: `proofreaderStripSlots.test.ts:41` thiếu `commandTitle` trong diff đã đưa soát; đã sửa sau lượt đủ bộ đầu, cùng vào commit.
- medium, patch: AC 5 và cú mất chưa có mục nợ, bảng `ket-qua.md` ghi Ẩn "3 sự kiện"; đã sửa bảng, nối vào mục `Chủ: Epic 14`.
- medium, patch: dòng 🟡 ở sổ nợ nói bàn đo chưa chạy là sai; đã thay bằng dòng ✅.
- medium, patch: phần chưa đo không có chủ; đã thêm dòng `Chủ: Epic 14` và dòng `Chủ: B7`.
- low, patch: comment sửa ở `PromptLibraryOverlay.vue:17-19` bằng tiếng Việt và lặp template; đã xoá câu đó.
- medium, patch: Xoá TM làm tiêu điểm về `body`; có từ trước nhưng story biến nút thành đường chuột chính thức, bàn đo đọc `BODY`, đã sửa kèm ca Vitest.

## Design Notes

Hàm kiểm ở mô-đun chứ không chuyển bản nháp ra ngoài: `bodyDraft`, `selectedRow`, `createOpen`, `flushChain` là state màn hình và comment đầu tệp giữ ranh giới đó.

```ts
export async function closePromptLibrary(): Promise<void> {
  if (leaveCheck !== null && !(await leaveCheck())) return
  overlayOpen.value = false
  overInspector.value = false
}
```

`@focusin` là bắt buộc, không phải trang trí: `onRowDelete(i)` hôm nay nhắm hàng cả khi Enter trên nút, và Xoá TM không hoàn tác được. Tab qua các hàng giờ dời con trỏ và huỷ nhịp chờ, giống `GlossaryHarvestOverlay.vue`.

`@mousedown.prevent` trên nút Ẩn thay vì `focusOnPointerDown`: giữ tiêu điểm trong ô là chắc hơn trả lại nó sau.

## Verification

**Commands:**
- `npm run check:commands`, `npm run check:i18n`, `npm run check:tokens`, `npm run check:panel-refs`, `npm run check:debt-owner` -- exit 0.
- `npx vitest run tests/frontend/proofreaderStripContent.test.ts tests/frontend/proofreaderStripSlots.test.ts tests/frontend/libraryChapters.test.ts tests/frontend/readingMarks.test.ts tests/frontend/alignment.test.ts tests/frontend/tmManage.test.ts tests/frontend/promptLibraryOverlayRender.test.ts`, `npx vue-tsc --noEmit`, `npm run check:lint` -- exit 0.
- `npm run build`, `cargo build --locked --features wdio --manifest-path src-tauri/Cargo.toml`, bàn đo `14-9-ban-do` -- chạy ngoài sandbox.
- Chạy đủ bộ một lần: story chạm `scripts/check-*` (AGENTS.md §Tests).

**Manual checks (if no CLI):**
- Lượt dùng thật của Ice trước khi Epic 14 `done`: tám nút trên bản chạy thật, nhất là sửa và xoá hàng TM.
