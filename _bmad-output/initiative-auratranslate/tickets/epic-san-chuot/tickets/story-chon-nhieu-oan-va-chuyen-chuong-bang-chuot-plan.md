---
title: 'Chọn nhiều đoạn và chuyển Chương bằng chuột'
type: 'feature'
ticket: '3'
created: '2026-10-30'
status: 'built'
baseline_revision: '71205e2fc5e825ac8614b483bc6450802cb5a716'
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

**Problem:** Vùng chọn nhiều câu chỉ đổi được bằng hai hợp âm gieo từ caret, và chuyển Chương chỉ có phím, nên người dùng chuột không chọn được lô câu để dịch và không sang được Chương kề.

**Approach:** Dựng kéo và `Shift`+bấm trên cột số câu, khai `pointer: aim` cho hai id `extend` với `at: src/panels/GridPanel.vue`; thêm hai nút ‹ › ở thanh trạng thái; rút 4 id khỏi `POINTER_DEBT`.

## Boundaries & Constraints

**Decision (repo chứng minh, không cần ký):** kết thúc kéo bắt `mouseup` ở `document` pha capture và `stopPropagation`. Không có bước này, thả chuột trên cột bản dịch chạy `onCellMouseUp` (`cell.focus()` và đặt caret), trái luật "caret còn" của `EXPERIENCE.md`.

**Decision:** hai nút ‹ › chặn mặc định của `mousedown` bằng handler có tên, không dùng `focusOnPointerDown` của các nút khác trong `StatusBar.vue`. Các nút đó lấy tiêu điểm để lớp phủ nhớ chỗ trả về; ‹ › không mở lớp phủ, và mất tiêu điểm thì `focusout` đặt caret `null`, `click` có thể không bắn (`EXPERIENCE.md` §Interaction Primitives).

**Decision:** trong lúc kéo, hàng dưới con trỏ đọc từ ô bất kỳ của câu, không chỉ ô số: chuột trôi ngang khỏi cột hẹp vẫn dời `focus`. Listener `mouseover`, `mouseup` chỉ gắn trong lúc kéo.

**Decision (Ice chọn 2026-10-30):** hai nút ẩn khi `reviewModeIsOpen`. Hợp âm `⌘⌥]` vẫn chạy trong Review Mode (`isBlocked` ở `src/main.ts:1142` không chặn), đổi Chương của Editor trong khi màn Review giữ `openChapterId` cũ; hợp âm không đổi ở story này, ghi mục nợ `Chủ: Story 14.4`. `EXPERIENCE.md` §State Patterns thêm dòng 🔵 thay cho "Không bao giờ tắt".

**Always:** Luật ở `EXPERIENCE.md` §Interaction Primitives *Kéo trên cột số câu*: `@mousedown` ủy quyền ở `.grid`, thoát ngay nếu không phải nút chính trên ô số; `preventDefault`; không ghi caret. Nhấn phụ thuộc về `onSentencePress`, phân biệt bằng `isSecondaryPress`. Dáng theo `DESIGN.md`: `ui-icon-btn`, cách dòng vị trí 7px; số câu rê chuột nền `surface-sunken`, con trỏ bấm được, câu đã chọn giữ dáng vùng chọn. Hai nút có `aria-label` và `title` kèm phím từ `shortcutRows`. Hiện theo `currentMode === 'workspace' && editorHasLoaded() && !reviewModeIsOpen`, không theo `positionText`.

**Never:** Đổi hợp âm, `extendSegmentSelectionDown/Up`, `segment.selection.clear`, hoặc đường gõ. Tự cuộn khi kéo (`EXPERIENCE.md` loại khỏi bản đầu). `POINTER_DEBT_BASELINE`. Ghi `tickets.toml`. Mở Story 14.4 trở đi.

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected Output / Behavior | Error Handling |
|----------|--------------|---------------------------|----------------|
| Kéo xuống | Nhấn số 3, kéo qua 4, thả ở 5 | Câu 3–5 tô, caret và tiêu điểm giữ nguyên | Không |
| Kéo ngược | Nhấn số 5, kéo lên 3 | Câu 3–5 tô, `anchor` 5 | Không |
| Bấm trơn | Nhấn và thả trên số 3, đang có vùng khác | Vùng chọn còn đúng câu 3 | Không |
| `Shift`+bấm có vùng | `anchor` 3, `Shift`+bấm số 8 | Câu 3–8 | Không |
| `Shift`+bấm chưa có vùng | Caret câu 2 | Câu 2–8 | Không |
| `Shift`+bấm không có gốc | Caret `null`, `anchor` đã chết sau đổi Chương | Vùng chọn còn đúng câu bấm | Không |
| Thả trên ô khác | Kéo rồi thả trên ô tgt hoặc src | Caret không đổi, không ghi điểm cắt | Không |
| Thả ngoài cửa sổ | `mouseover` kế tiếp có `buttons === 0` | Kéo kết thúc, không dời `focus` | Không |
| Nhấn phụ, `Ctrl`+bấm macOS | Trên số câu | Menu câu như cũ, vùng chọn không đổi | Không |
| Nhấn trên đệm cột số | `.strip-pad` | Không làm gì | Không |
| Bấm › từ ô đang gõ | Tiêu điểm trong ô tgt | Flush rồi sang Chương; caret không về `null` trước `click` | Flush lỗi: thông báo đã có |
| Biên Chương | Bấm › ở Chương cuối | Thông báo biên đã có | Không |
| Chương đang nạp | `editorHasLoaded()` sai | Hai nút ẩn cùng dòng vị trí | Không |
| Chế độ khác | Thư viện, Chế độ đọc | Không hiện nút | Không |
| Review Mode mở | `reviewModeIsOpen` đúng | Không hiện nút; hợp âm không đổi | Không |

</frozen-after-approval>

## Code Map

- `src/panels/segmentSelectionState.ts` -- `moveSegmentSelectionFocus`:113 giữ nguyên; luật gieo ba nhánh ở :120–140 là khuôn cho `Shift`+bấm (`anchor` hoặc `focus` đã chết thì gieo từ caret).
- `src/panels/GridPanel.vue` -- `.grid`:~1996 nhận `@mousedown`; `onSentencePress`:976 chứa phép đọc id từ `closest`, tách helper dùng chung; `PLATFORM`:735; `isSecondaryPress` đã import :145; `onCellMouseUp`:~1000, `onSourceCellMouseUp`:~915 là hai đường thả phải tránh; dọn listener theo khuôn `selectionchange`:1386.
- `src/panels/GridBlock.vue` -- ô số:140 có `data-num-segment-id`; `.cell-num` CSS:665; `.cell.row-selected`:810 cùng độ đặc hiệu với `:hover`.
- `src/commands/index.ts` -- vòng tuple :3326 thêm `pointer`, câu `reason` khác dòng "Bước con trỏ…" vì đây là mở rộng vùng; AD-62 mục 3 chỉ đòi `@mousedown` ở `at`.
- `src/StatusBar.vue` -- `.status-info`:567 nhánh `v-else` chạy cả Chế độ đọc, `align-items: baseline` khớp `ui-icon-btn::before`; `lookupChordText`:253 khuôn đọc phím; `GridSentenceMenu.vue::holdFocus`:36 khuôn chặn mặc định.
- `src/modes/LibraryMode.vue`:860–885 -- tiền lệ `ui-icon-btn` với ký tự ‹ › và `aria-label`.
- `src/panels/AiTranslationPanel.vue`:671, `src/i18n/vi.json`:1467 -- `batch_select_hint` thêm cách kéo.
- `scripts/check-commands.mjs` -- `POINTER_DEBT`:2898 (chỉ xoá ở đây), `HANDLER_TABLE` thêm `GridPanel.vue::<handler>` và `StatusBar.vue::holdFocus`, `HANDLER_ATTR_FLOOR`:2871 theo công thức cổng in ra; `R_MENU_HOLD_FOCUS`:2428 nói "floating menu", cần lý do riêng cho nút.
- `tests/frontend/gridSentenceMenu.test.ts`:38 -- khuôn `mountGrid`. `tests/frontend/statusBarWorkspace.test.ts` -- khuôn mount `StatusBar`. `tests/frontend/segmentSelection.test.ts` -- khuôn đổi Chương, nơi ca `anchor` đã chết sống.
- `e2e/specs/story-5-8-reorganise-chapters.e2e.mjs`:173 -- dựng hai Chương qua IPC. `_bmad-output/implementation-artifacts/14-2-ban-do/chuot-that.e2e.mjs` -- khuôn bàn đo `cliclick`.
- `_bmad-output/initiative-auratranslate/deferred-work.md` -- mục `Chủ: Story 14.3` ngay sau mục 14.2 (`grep`).

## Tasks & Acceptance

**Execution:**
- [x] `src/panels/segmentSelectionState.ts` -- thêm `pressSegmentSelection(id, extend)` và `dragSegmentSelectionFocus(id)`; không ghi caret.
- [x] `src/panels/GridPanel.vue` -- handler `@mousedown` trên `.grid`, listener kéo gắn và gỡ theo lượt, dọn khi unmount.
- [x] `src/panels/GridBlock.vue` -- dáng rê chuột của `.cell-num`.
- [x] `src/commands/index.ts` -- `pointer: aim` cho hai id `extend`.
- [x] `src/StatusBar.vue` -- hai nút ‹ ›, `@click` literal, `@mousedown` có tên, `data-` để test chọn, CSS khoảng cách.
- [x] `src/panels/AiTranslationPanel.vue`, `src/i18n/vi.json` -- câu gợi ý.
- [x] `scripts/check-commands.mjs` -- gỡ 4 id, thêm mục `HANDLER_TABLE`, nâng sàn nếu cổng đòi.
- [x] `tests/frontend/gridSegmentSelectionDrag.test.ts` (mới) -- ca (i) cho hai id `extend`: bắn `mousedown`, `mouseover`, `mouseup` thật vào ô số, khẳng định `segmentSelectionIds`; mọi hàng Lưới của I/O Matrix.
- [x] `tests/frontend/statusBarWorkspace.test.ts` -- ca (i) cho hai id Chương: bấm nút, dep chạy; các hàng hiện/ẩn của I/O Matrix.
- [x] `e2e/specs/grid-segment-selection.e2e.mjs` (mới) -- kéo qua ba câu, `Shift`+bấm câu xa, ‹ › giữa hai Chương thật.
- [x] `_bmad-output/implementation-artifacts/14-3-ban-do/` -- bàn đo `cliclick`: kéo thật có `mouseover` đủ, thả trên ô tgt không dời caret, bấm › từ ô đang gõ thì `click` bắn và caret không `null`; thời gian một lượt dời `focus` trên Chương dài nhất có sẵn.
- [x] `deferred-work.md` -- đóng mục 14.3; thêm mục `Chủ: Epic 14` cho lượt dùng thật; thêm mục `Chủ: Story 14.4` cho hợp âm chuyển Chương trong Review Mode; bổ sung ‹ › vào mục `B7` về focus nút trên WebView2.
- [x] `tickets/epic-san-chuot/ux-san-chuot/EXPERIENCE.md` -- dòng 🔵 ở §State Patterns: ‹ › ẩn khi Review Mode mở.

**Acceptance Criteria:**
- Given cây sau story, when chạy `npm run check:commands`, then Kiểm M xanh với 271 điểm chuột, 29 khai `pointer`, 30 nợ.
- Given app thật, when kéo qua ba số câu rồi `Shift`+bấm một câu xa, then đúng các câu đó được tô và panel Đề xuất AI hiện đúng số câu.
- Given app thật có hai Chương, when bấm › rồi ‹, then Lưới sang Chương kề rồi về, thanh trạng thái đổi theo.
- Given ca (i) của id `extend`, when gỡ `@mousedown` khỏi `.grid`, then ca đó đỏ vì vùng chọn không đổi.
- Given ca (i) của id Chương, when gỡ `@click` khỏi nút, then chính ca đó đỏ.
- Given `at` trỏ `GridPanel.vue`, when gỡ `@mousedown` khỏi `.grid`, then `check:commands` đỏ đúng hai id `extend`.
- Given đang gõ trong ô, when so `gridTypingCost.test.ts` và `gridCaretMoveCost.test.ts`, then cả hai xanh không đổi ngưỡng.

## Implementation Notes

- `mouseover` kiểm `buttons === 0` để kết thúc kéo khi thả ngoài cửa sổ; WKWebView báo `buttons=1` suốt lượt kéo chuột thật nên điều kiện này không tự cắt kéo.
- Sau khi thả, WebKit bắn `click` lên `DIV` chung của hai ô; không handler nào đọc.
- Câu đầu tệp `segmentSelectionState.ts` nói caret được đọc "đúng một lần" đã sai sau `Shift`+bấm, đã sửa tại chỗ.
- `.status-info` có `gap` 14px nên `.chapter-nav` dùng lề âm một `--space-unit` để đạt khoảng cách 7px của `DESIGN.md`.
- Ca "Mod giữ không ghi điểm cắt" phải stub `caretRangeFromPoint`: happy-dom không có API này nên handler thoát sớm và ca không thể đỏ.
- Hai ca e2e xanh trên WKWebView khi chạy riêng (`--mochaOpts.grep`). Chạy chung, ca sau đỏ ở fixture vì Vite của `tauri dev` có 36 import mang tem `?t=`, `import()` của fixture ra module khác; không phải lỗi sản phẩm.
- Không gộp `mouseover` theo khung hình: 9.850 câu đo 116 ms mỗi lượt và chi phí theo cỡ Chương, gộp không giảm giá mỗi khung. Mục nợ `Chủ: Epic 14`, số đo ở `_bmad-output/implementation-artifacts/14-3-ban-do/ket-qua.md`.
- Đối chứng gỡ thật, mỗi phép đỏ đúng lý do: `@mousedown` khỏi `.grid`, `up.stopPropagation()`, hai `@click`, `@mousedown="holdFocus"`, `!reviewModeIsOpen`.

## Plan Change Log

## Review Triage Log

Lượt 1 (lens Quick): high 0 · medium 0 · low 3 · false 1 · maybe-false 0.

- low, patch: comment đầu và tên `describe` của bàn đo mang id story, trái AGENTS.md §Code comments; đã bỏ.
- low, patch: `ket-qua.md` trỏ tới `ket-qua.json` không được commit, thư mục 14-2 cũng chỉ giữ `.mjs` và `.md`; đã xoá câu đó và tệp sinh ra.
- false: bấm trơn trên số câu tạo vùng chọn một câu là đúng ý định, `EXPERIENCE.md` §Interaction Primitives ghi `mousedown` đặt `anchor` và `focus` vào câu đó, hàng "Bấm trơn" của Matrix cũng vậy.
- low, reject: mock `reviewModeIsOpen` trong `statusBarWorkspace.test.ts` khó đọc và không có ca mount sẵn Review Mode; ca hiện có đỏ khi gỡ `!reviewModeIsOpen`, `computed` phản ứng như nhau ở hai thời điểm, sửa cần thêm cấu trúc test.

## Design Notes

`editorHasLoaded()` sai suốt lượt nạp (`resetEditorPanel` rồi `ensureSegmentsLoaded`), nên hai nút ẩn trong lúc đó và cú bấm › liên tiếp có thể rơi vào khoảng trống. Giữ theo vé vì dòng vị trí và bộ đếm cũng ẩn cùng lúc, không có xê dịch mới; `dangChuyenChuong` chặn lượt trùng.

Mỗi lượt dời `focus` tính lại `selectedRowClassById`, nhưng `mouseover` chỉ bắn khi qua ranh ô nên tần suất bị chặn theo số hàng đi qua. Bàn đo trả lời có cần gộp theo khung hình hay không; không gộp trước khi có số.

Sự kiện driver `isTrusted: false` và có thể không sinh `mouseover`, nên e2e canh dây nối còn bàn đo `cliclick` canh chuột thật.

## Verification

**Commands:**
- `npm run check:commands` -- exit 0, in 271/29/30.
- `npm run check:i18n`, `npm run check:tokens`, `npm run check:debt-owner` -- exit 0.
- `npm run test:story 14-3` -- exit 0.
- `npx vue-tsc --noEmit`, `npm run check:lint` -- exit 0.
- `cargo build --locked --features wdio --manifest-path src-tauri/Cargo.toml`, rồi `npx wdio run e2e/wdio.conf.mjs --spec e2e/specs/grid-segment-selection.e2e.mjs` -- xanh, chạy ngoài sandbox.
- Chạy đủ bộ một lần: story chạm `scripts/check-*` (AGENTS.md §Tests).

**Manual checks (if no CLI):**
- Lượt dùng thật của Ice trước khi Epic 14 `done`: kéo trên cột số, `Shift`+bấm, ‹ › từ ô đang gõ.
