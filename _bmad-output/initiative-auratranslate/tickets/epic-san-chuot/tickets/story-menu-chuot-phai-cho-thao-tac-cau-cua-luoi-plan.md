---
title: 'Menu chuột phải cho thao tác câu của Lưới'
type: 'feature'
ticket: '2'
created: '2026-10-30'
status: 'done'
baseline_revision: 'f9f89423dc8385c06bb83ba84b58089bd4f1d6e9'
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

**Problem:** 10 id thao tác câu của Lưới chỉ có phím, và hai đường từ chối (`editorOmitError`, `setCurrentSegmentParagraphEnd`) im lặng nên một cú bấm chuột sẽ không đổi gì trên màn hình.

**Approach:** Dựng menu câu mở bằng chuột phải qua command mới `editor.sentence_menu.open`, nhắm đúng câu dưới con trỏ, mỗi id một mục `dispatch` literal; đo hành vi thật trên WKWebView, đưa hai đường từ chối ra thanh trạng thái, rút 10 id khỏi `POINTER_DEBT`.

## Boundaries & Constraints

**Decision (repo chứng minh, không cần ký):** mục `end`/`join` chọn theo `is_target_paragraph_end`, không theo `is_paragraph_end` như `EXPERIENCE.md` §State Patterns ghi: lệnh đổi cờ bản dịch (AD-46), đọc cờ nguồn sẽ hiện sai mục. Sửa dòng đó bằng 🔵.

**Decision:** sau khi chọn một mục mà caret còn ở câu nhắm và tiêu điểm ở ô khác, tiêu điểm vào ô câu đó qua `editorCaretPlacement`; không thì câu đang viền và ô nhận chữ là hai câu khác nhau. Trước lúc chọn, không `focus()` ô bản dịch (giữ vùng bôi đen cho Story 14.5).

**Decision:** menu không nhận tiêu điểm nên không có điều hướng mũi tên (NFR17 ngoài epic). Mở menu chặn mọi hợp âm qua `isBlocked`. `Esc` đóng và bị nuốt; phím khác đóng rồi đi tiếp vào ô.

**Always:** Luật nhắm ở `EXPERIENCE.md` §Interaction Primitives. Nhãn, nhóm, thứ tự mục theo §Bảng chỗ đặt và §Component Patterns; dáng theo `DESIGN.md`. Nhóm 2 để Story 14.4, 14.5 thêm mục. Handler `mousedown` thoát ngay nếu không phải nhấn phụ: đường gõ không đổi. `Mod` qua `hasPrimaryModifier`/`formatChord`, không `metaKey` trần.

**Never:** `@mousedown` trong `GridPanel.vue` (Story 14.3 cần tệp này 0 thuộc tính). Thêm `data-segment-id` lên ô số câu (`editorTypingZone.test.ts`, `grid-empty-cell.e2e.mjs` đếm thuộc tính đó). Đổi hợp âm nào. Mở Story 14.3–14.5. Ghi `tickets.toml`.

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected Output / Behavior | Error Handling |
|----------|--------------|---------------------------|----------------|
| Nhắm ô bản dịch | Caret A, chuột phải ô tgt câu B, chọn Xác nhận | B được xác nhận, caret sang câu kế | Từ chối: câu đã có ở thanh trạng thái |
| Nhắm nguyên văn, số câu | Caret A, nhấn phụ ô src hoặc num câu B | `mousedown` bị chặn, mọi mục chạy trên B | Không |
| Đóng không chọn | `Esc`, bấm ngoài, cuộn Lưới, phím khác | Caret và vùng chọn về đúng lúc trước; từ WebKit bôi thu lại; phím gõ tiếp không đè chữ; `Esc` không xoá điểm cắt | Không |
| Caret `null` lúc mở | Đóng không chọn | Vùng chọn trong ô tgt thu về caret, caret theo vùng chọn | Không |
| Không có câu dưới con trỏ | Chuột phải dải TM/Glossary, đệm hàng | Không mở, không nhắm | Không |
| Không có cú nhấn | Phím `Menu`, hợp âm tự gán | Mở tại ô câu có caret; caret `null` thì không mở | Không |
| `Ctrl`+bấm trên macOS | `button 0`, `ctrlKey` | Như nhấn phụ | Không |
| Điểm cắt ở câu khác | Cắt ở câu 5, nhắm câu 20 | Tách câu tắt, `title` nói câu 5 | Không |
| Chưa có điểm cắt | Chọn Tách câu | `regroup_no_cut` nói `Mod`+bấm, phím theo nền tảng | Không |
| Câu đầu Chương | Nhắm câu đầu | Gộp lên tắt | Không |
| Từ chối cờ | `set_segment_omitted` hoặc `set_segment_paragraph_end` trả lỗi | Thanh trạng thái nói lý do; không có cầu IPC thì nói câu chung | Không ném |
| Sát mép cửa sổ | Mở gần mép phải, dưới | Menu 294px lật vào trong | Không |
| Đổi Chương khi mở | Chương mới nạp, panel tháo | Menu đóng | Không |

</frozen-after-approval>

## Code Map

- `src/panels/GridPanel.vue` -- `.grid` (dòng 1954) nhận `@contextmenu.prevent`; `onCellMouseDown`:962, `onColumnFocusOut`:1166, `onCellMouseUp`:980 (focus + đặt caret), `onSelectionChange`:1268 (selection trong cột tgt kéo caret theo), `selectRange`:781, `targetCellById`:672, watcher `editorCaretPlacement`:~1380, `firstSegmentId`:346, `PLATFORM`:735; các handler đưa xuống `GridBlock` qua `ctx`:~1856.
- `src/panels/GridBlock.vue` -- gốc khối nhận `@mousedown.capture`; ô số câu:138 chưa có id, dùng thuộc tính riêng (`data-num-segment-id`).
- `src/panels/editorPanelState.ts` -- `setEditorCaret`:302, `editorSourceCut`:2448, `omitError`:1595, `setCurrentSegmentParagraphEnd`:1712 (đang vứt `error`), `datThongBao`:2714 (một cửa ghi, thêm ô mới ở đây).
- `src/commands/index.ts` -- thêm `CommandDeps.openSentenceMenu`, `register` sau vòng `merge/split`:3740, `keys: []`.
- `src/main.ts` -- nối dep; gate `isBlocked`:1143.
- `src/StatusBar.vue` -- `shortcutHintText`:200, `REGROUP_NOTICE_KEYS`:341, chuỗi `v-else-if`:583–631. `src/i18n/vi.json` -- `status.regroup_hint`:393, `panel.grid.regroup_no_cut`:1401.
- `src/TitlebarToolsMenu.vue` -- khuôn `chordOf`, `onDocumentMouseDown` capture; không chép `focusOnPointerDown`. `src/tokens/controls.css`:493 -- `.ui-menu`, `.ui-menu-item`, `.ui-menu-sep`.
- `scripts/check-commands.mjs` -- `POINTER_DEBT` (chỉ xoá ở đây, `POINTER_DEBT_BASELINE` giữ nguyên), `HANDLER_TABLE`:2613, `R_CURSOR_LOCAL`:2417.
- `tests/frontend/editorClearSourceCuts.test.ts` -- khuôn mount `GridPanel` + `installCommands`. `tests/frontend/statusBarWorkspace.test.ts`:204 -- ca gợi ý theo phím gán lại đổi theo câu mới.
- `e2e/support/pointer.mjs` -- `realClick`, `modifiedMouseDown`. ⚠️ Sự kiện driver `isTrusted: false`, không có hành vi mặc định, nên không đo được WebKit tự bôi đen.
- `_bmad-output/initiative-auratranslate/deferred-work.md`:13652 -- mục `Chủ: Story 14.2`.

## Tasks & Acceptance

**Execution:**
- [ ] `src/panels/sentenceMenuState.ts` (mới) -- giữ cú nhấn phụ gần nhất (id hoặc "không câu", toạ độ, caret và `Range` trước đó), trạng thái mở, `openSentenceMenu`, `closeSentenceMenu('chosen' | 'dismissed')`; vị từ thuần `isSecondaryPress(event, platform)` -- một nguồn sự thật cho nhắm và đóng.
- [ ] `src/panels/GridBlock.vue`, `GridPanel.vue` -- handler `mousedown` capture: ghi cú nhấn; ô src, num thì `preventDefault` + `setEditorCaret`; `@contextmenu.prevent`; trả caret và `Range` khi `dismissed`, placement khi `chosen`; đóng khi cuộn `.grid-scroll` và khi `chapterId` đổi.
- [ ] `src/panels/GridSentenceMenu.vue` (mới) -- `<Teleport to="body">`, `ui-menu` 294px, tiêu đề `Câu N`, 11 nút `@click` literal, `omit`/`restore` và `end`/`join` là cặp `v-if`, lật vào trong, `mousedown` capture ngoài menu, `keydown` capture; bọc `@mousedown.prevent` (Kiểm K đỏ với giá trị rỗng thì dùng handler có tên, `nonCommand`).
- [ ] `src/commands/index.ts`, `src/main.ts`, `src/i18n/vi.json` -- command mới, dep, gate, khoá nhãn; `command.editor.sentence_menu.open`.
- [ ] `src/panels/editorPanelState.ts`, `src/StatusBar.vue` -- hai đường từ chối qua `datThongBao`, giữ `error` của lệnh cờ đoạn, gỡ comment "chưa ai đọc" đã sai; `regroup_no_cut` nhận `{mod_key}`; `status.regroup_hint` thành câu gợi ý chuột phải.
- [ ] `scripts/check-commands.mjs` -- gỡ 10 id khỏi `POINTER_DEBT`, thêm mục `HANDLER_TABLE`.
- [ ] `_bmad-output/implementation-artifacts/14-2-ban-do/` -- bàn đo chuột thật: `cliclick` vào cửa sổ app trong phiên wdio, `browser.execute` đọc lại; ghi thứ tự sự kiện, caret, tiêu điểm, vùng chọn ở ba cột và phím gõ sau khi đóng -- đóng hai điều vé giao đo. Số đo trái bảng UX thì sửa handler và ghi 🔵 ở `EXPERIENCE.md`.
- [ ] `tests/frontend/gridSentenceMenu.test.ts` (mới) -- ca (i) cho từng id: nhấn phụ + `contextmenu` thật, bấm mục, khẳng định dep chạy và caret là B; mọi hàng I/O Matrix kiểm được bằng happy-dom. `statusBarWorkspace.test.ts`, ca từ chối ở StatusBar.
- [ ] `e2e/specs/grid-sentence-menu.e2e.mjs` (mới) -- caret A, nhắm B ở ba cột, Xác nhận trúng B; đóng thì caret về A; điểm cắt ở câu khác thì Tách câu tắt.
- [ ] `deferred-work.md` -- đóng mục 14.2; thêm mục `Chủ: B7` cho phím `Menu` và khác biệt chuột phải trên Windows; thứ cần mắt người vào mục `Chủ: Epic 14`. `EXPERIENCE.md` -- dòng 🔵 về cờ đoạn.

**Acceptance Criteria:**
- Given cây sau story, when chạy `npm run check:commands`, then Kiểm M xanh với 269 điểm chuột, 27 khai `pointer`, 34 nợ.
- Given app thật với caret ở câu A, when chuột phải vào câu B ở từng cột và chọn Xác nhận, then đúng B được xác nhận; đóng không chọn thì caret về A và phím gõ tiếp không đè chữ.
- Given điểm cắt ở câu khác, when mở menu, then Tách câu tắt và `title` nói câu nào.
- Given ca (i) của một id, when gỡ `@click` khỏi mục đó, then chính ca đó đỏ; gỡ `@contextmenu.prevent` khỏi `.grid` thì mọi ca đỏ vì menu không mở.
- Given chặn tiêu điểm ở cột src, when gỡ `setEditorCaret` khỏi nhánh nhắm, then ca nhắm đỏ vì lệnh chạy trên câu A.
- Given đang gõ trong ô, when so `gridTypingCost.test.ts` và `gridCaretMoveCost.test.ts`, then cả hai xanh không đổi ngưỡng.

## Implementation Notes

- Nhắm bằng `mousedown` capture ở gốc `GridBlock` (`onSentencePress`), định nghĩa trong `GridPanel.vue`; `openSentenceMenu` vẫn đặt lại caret về câu nhắm vì `onColumnFocusOut` có thể đặt `null` giữa hai sự kiện.
- Cú nhấn ghi lại chỉ có hiệu lực 1 giây và dùng một lần; không có cú nhấn thì menu mở tại ô có caret, nên phím `Menu` và hợp âm tự gán chạy cùng một đường.
- Menu nghe `keydown` ở `document` capture để chạy sau cổng `isBlocked` ở `window`: hợp âm bị chặn kể cả cú phím đóng menu. `Esc` bị nuốt ở đó nên không xoá điểm cắt.
- Ô số câu mang `data-num-segment-id` riêng; không thêm `data-segment-id` vì `editorTypingZone.test.ts` và `grid-empty-cell.e2e.mjs` đếm thuộc tính đó.
- Hai đường từ chối cờ đi qua một ô mới `editorFlagRefusal` trong `datThongBao`; `setCurrentSegmentParagraphEnd` giữ `error` của lệnh.
- Mục menu là 10 nút (không phải 11): Gộp lên tắt ở câu đầu Chương, Tách câu tắt khi điểm cắt ở câu khác, Bỏ điểm cắt chỉ hiện khi có điểm cắt. Nhóm 2 chưa có mục nào, Story 14.4 và 14.5 thêm cả nhóm lẫn nét ngăn.
- `HANDLER_ATTR_FLOOR` 157 lên 168 (`check:commands`) và `COMPONENT_FILE_FLOOR` 176 lên 189 (`check:tokens`) đúng công thức cổng in ra; cái sau đã đỏ sẵn ở HEAD (221 tệp, 80% = 176,8).
- Đo chuột thật bằng `cliclick` khớp bảng UX: macOS đi `mousedown → contextmenu → mouseup` và `mouseup` rơi vào menu nên `onCellMouseUp` không chạy; WebKit bôi đen từ dù đã chặn `mousedown`; đóng rồi gõ tiếp không đè chữ. Số đo ở `_bmad-output/implementation-artifacts/14-2-ban-do/ket-qua.md`; ba ca còn thiếu ghi nợ `Chủ: Epic 14`.
- Spec e2e 5/5 xanh trên WKWebView (`--features wdio`, Vite mới) trước lượt vá review. Ba ca thêm sau review (Xác nhận trên đĩa, chặn hợp âm khi menu mở, menu sát mép) chưa chạy vì `tauri dev` lại giữ cổng 1420; ghi nợ `Chủ: Epic 14`.
- Đối chứng AC5: gỡ một mình `setEditorCaret` ở nhánh nhắm thì lệnh vẫn chạy trên B vì `openSentenceMenu` nhắm lại; gỡ cả hai thì 16/35 ca đỏ, lệnh chạy trên câu A (`[11]` thay `[13]`).
- `controls-adoption.test.ts` đỏ từ `f9f89423` đã sửa cùng lượt: hai luật hình dạng ở `App.vue` chuyển thành biến thể `ui-seg--bare` trong `controls.css`, gỡ luôn miễn trừ `.layout-switch`, không thêm miễn trừ mới.

## Plan Change Log

## Review Triage Log

Lượt 1 (lens Quick): high 0 · medium 5 · low 10 · false 0 · maybe-false 1.

- medium, patch: hợp âm tự gán mở rồi đóng menu trong cùng một `keydown`, xác nhận ở `keys.ts` (`preventDefault`, không `stopPropagation`); đã thêm `if (event.defaultPrevented) return` và ca đỏ khi gỡ.
- medium, patch: `chosen` chỉ hỏi tiêu điểm nên từ WebKit bôi đen ở ô bản dịch còn nguyên và phím sau đè nó, khớp số đo ở `ket-qua.md`; đã thu vùng chọn hoặc placement, mỗi nhánh một ca.
- medium, patch: ca chặn hợp âm dùng `isBlocked` giả đọc DOM; đã dùng `sentenceMenuIsOpen` và thêm ca e2e, ca e2e chưa chạy và `browser.keys` có thể không mang `metaKey`.
- medium, patch: ca (i) không kiểm tham số nên đổi `omit`/`restore` hoặc `end`/`join` vẫn xanh; đã kiểm tham số boolean.
- medium, patch: spec chỉ dùng Cắt bỏ câu trong khi AC nói Xác nhận; đã thêm ca e2e ký trên đĩa, chưa chạy.
- maybe-false, defer: `onColumnFocusOut` đặt caret `null` khi rời cửa sổ thì mục menu không làm gì; WebKit có bắn `focusout` khi đổi app hay không chưa đo, cần Cmd-Tab khi menu mở trên app thật.
- low, reject: AC5 không đỏ đúng lý do vì `openSentenceMenu` cũng nhắm lại; sửa là sửa AC của plan, đối chứng thật ghi ở Implementation Notes.
- low, patch: `modKeyLabel` viết tay ký hiệu; đã suy từ `formatChord`.
- low, patch: hai comment tiếng Việt ở `main.ts` trái luật; đã xoá.
- low, patch: mục nợ cờ đích bị từ chối còn mở dù đã sửa; đã đóng.
- low, patch: `EXPERIENCE.md` còn ghi chưa đo; đã thêm dòng 🔵.
- low, reject: caret `null` lúc mở thì `lastCaretSegmentId` và vị trí đã lưu ở lại câu vừa chuột phải; ít gặp, sửa cần thêm một đường ghi trạng thái.
- low, reject: nhấn giữ quá 1 giây trên Windows rơi về đường không có cú nhấn; ít gặp, thuộc lượt kiểm `B7`.
- low, patch: `placeMenu` không kẹp vào cửa sổ; đã kẹp, có ca đơn vị và ca e2e chưa chạy.
- low, patch: `MENU_WIDTH` không ai dùng; đã xoá.
- low, patch: e2e dùng `metaKey` trần; đã thêm `ctrlKey`.

## Design Notes

Hai thứ tự sự kiện, nên `open` tự đặt lại caret về câu nhắm thay vì tin caret lúc đó: macOS đi `mousedown → contextmenu → mouseup`, Windows đi `mousedown → mouseup → contextmenu`. Giữa `mousedown` và `contextmenu`, `onColumnFocusOut` có thể đặt caret `null`, vì WKWebView trao tiêu điểm cho `section` của panel.

Handler đặt `capture` ở gốc khối để đọc caret trước khi `onCellMouseDown` ghi đè, và một thuộc tính phủ ba cột.

Trên macOS chuột phải vào chữ làm WebKit bôi đen từ dù đã chặn `mousedown`, nên có vùng bôi đen ở nguyên văn gần như mọi lần. Vì thế luật "không `focus()`" chỉ áp dụng trước lúc chọn; `chosen` chạy sau `dispatch`, và `add_term` đọc `getSelection()` ngay trong `run`. Mục mở lớp phủ của Story 14.4, 14.5 phải bỏ qua placement, ghi vào plan của hai story đó.

Gõ khi caret lệch không mất dữ liệu: `onEditInput` theo ô DOM và `selectionchange` kéo caret về; đó là lý do placement là việc của hiển thị, không phải an toàn dữ liệu.

Phép đo cần sự kiện tin cậy nên không dùng driver. Bàn đo cần quyền Accessibility cho `cliclick`; thiếu quyền thì hai điều đo vào mục `Chủ: Epic 14`, plan ghi rõ chưa đo, không suy.

## Verification

**Commands:**
- `npm run check:commands` -- exit 0, in 269/27/34.
- `npm run check:i18n`, `npm run check:tokens`, `npm run check:debt-owner` -- exit 0.
- `npm run test:story 14-2` -- exit 0.
- `npx vue-tsc --noEmit`, `npx vue-tsc --noEmit -p tsconfig.node.json`, `npm run check:lint` -- exit 0.
- `cargo build --locked --features wdio --manifest-path src-tauri/Cargo.toml`, rồi `npx wdio run e2e/wdio.conf.mjs --spec e2e/specs/grid-sentence-menu.e2e.mjs` -- xanh, chạy ngoài sandbox.
- Chạy đủ bộ một lần: story chạm `scripts/check-*` (AGENTS.md §Tests).

**Manual checks (if no CLI):**
- Lượt dùng thật của Ice trước khi Epic 14 `done`: chuột phải ở ba cột, Tách câu với điểm cắt ở câu khác, đóng rồi gõ tiếp.
