---
title: 'Refactor sweep'
type: 'refactor'
ticket: '10'
created: '2026-10-31'
status: 'built'
baseline_revision: '7f1f1e497ec606770ea2317c01a0339435f020bf'
route: 'full'
route_source: 'auto'
risk: 'medium'
review: 'quick'
review_source: 'pinned'
lenses_ran: ['quick']
review_loop_iteration: 0
context: []
---

<frozen-after-approval reason="human-owned intent — do not modify unless human renegotiates">

## Intent

**Problem:** Chín story vá đã rút `POINTER_DEBT` về rỗng (`check:commands` đọc 297/33/0), nhưng epic chưa có bằng chứng theo từng id cho R7, và còn phát hiện review đã hoãn: menu câu khi rời cửa sổ (14.2, maybe-false), `Esc` đưa caret về đầu ô sau Lịch sử (14.4, Ice quyết hoãn). Ba ca e2e thêm sau review 14.2 chưa từng chạy.

**Approach:** Chốt bước kiểm `POINTER_DEBT` rỗng kèm bảng 44 id → ca R7, chạy ba ca e2e còn nợ, đo đúng phát hiện review đã hoãn rồi sửa theo mục nợ, và cho mỗi phát hiện hoãn của epic một dạng đóng hoặc một chủ.

**Decisions (Ice, 2026-10-31):**
- `Esc` đóng Lịch sử: sửa cả hai đường `Mod+H` và mục menu câu. Ice nới "không đổi đường phím" cho đúng một hành vi này: caret về chỗ cũ thay vì đầu ô.
- Ba nợ mã không phải phát hiện review (phím lọt sau lớp phủ, mở Chương từ Thư viện khi Review Mode mở, kéo cột số câu 116 ms): 14.10 không nhận, giữ `Chủ: Epic 14` để Sofia lập story riêng.
- Đẩy `master` cuối 14.10, đọc CI cả nửa Windows; lượt đêm kế tiếp để lại cho lượt sau.

## Boundaries & Constraints

**Always:** AD-62, Kiểm A, AD-34 §1, `unbound()` và màn hình phím tắt không đổi. Phát hiện `reject` có lý do giữ nguyên. Mục `Chủ: Ice`, `B7` giữ chủ. Lượt dùng thật giữ `Chủ: Epic 14`. Số đo ghi build, máy, quần thể. Đóng mục nợ bằng lời, không xoá.

**Never:** Nhận tính năng mới. Rà lại 258 điểm chuột cũ. Sửa `POINTER_DEBT_BASELINE`. Đổi hành vi `openChapterById`. Dọn comment ngoài dòng đang sửa. Tự lật `status` của plan 14.4–14.9. Ghi `tickets.toml`.

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected Output / Behavior | Error Handling |
|----------|--------------|---------------------------|----------------|
| Rời cửa sổ khi menu câu mở | Chuột phải trên câu, menu mở, Cmd-Tab đi rồi về | Bấm mục menu vẫn chạy đúng câu, hoặc menu đã đóng; không bao giờ mục menu im lặng không làm gì | `focusout` không bắn ⇒ không sửa, đóng mục nợ bằng số đo |
| Rời cửa sổ không có menu | Caret trong ô, Cmd-Tab đi rồi về | Như trước, gõ tiếp đúng chỗ | Không đổi |
| `Mod+H` rồi `Esc` | Caret giữa `abc`, sau `a` | Tiêu điểm về ô, gõ `zz` ra `azzbc`, không chữ nào bị đè | Không đổi |
| Mục menu Lịch sử rồi `Esc` | Chuột phải trong ô, WebKit tự bôi một từ | Caret thu về một điểm trong đúng ô, gõ tiếp không đè từ đó; phím trong lớp phủ vẫn không lọt xuống ô | Không có `Range` trong ô ⇒ đường đặt caret hiện có |
| Khôi phục phiên bản rồi đóng | Văn bản ô đã đổi, nút cũ rời DOM | Tiêu điểm về ô, không ném | `Range` hết `isConnected` ⇒ đường đặt caret hiện có |
| Câu không còn trong Chương | Chương đổi trong lúc lớp phủ mở | Hành vi đóng hiện có | Bỏ `Range` đã lưu |

</frozen-after-approval>

## Code Map

- `scripts/check-commands.mjs:2905` -- `POINTER_DEBT` rỗng; `:2908` baseline 44 id; `:2956` `POINTER_FLOOR = 253`. Không đổi.
- `src/panels/GridPanel.vue:1234` `onColumnFocusOut` -- `relatedTarget` null đặt caret `null`, nguồn của phát hiện 14.2.
- `src/panels/sentenceMenuState.ts:112` `closeSentenceMenu`, `:96` `restoreBeforeOpen` -- khuôn đóng và trả caret, dùng lại lý do `'dismissed'`.
- `src/panels/GridSentenceMenu.vue` -- bề mặt menu, nơi gắn nghe `blur` của cửa sổ nếu phép đo tái hiện.
- `src/panels/segmentHistoryState.ts:125`, `:151` `openSegmentHistory`, `closeSegmentHistory`; `src/SegmentHistoryOverlay.vue:62–85` `returnFocusTo`, `back.focus()` -- chỗ lưu và trả `Range`.
- `src/panels/sentenceMenuState.ts:57` `currentSelectionRange`; `:128` nhánh `'overlay'` gọi `removeAllRanges()` -- `@click` của mục chạy `history.open` TRƯỚC `onDocumentClick` đóng menu, nên `Range` đọc được lúc mở có thể là từ WebKit tự bôi. Đừng trả nguyên, và đừng bỏ `removeAllRanges`: đó là chốt chặn phím lọt.
- `tests/frontend/` -- ca R7 ghi id ngắn (`bilingual_toggle_cut`), nên grep id đầy đủ báo thiếu sai; phải đọc ca, không đếm chuỗi. Hai id `extend` khai `pointer: aim`, ca ở `gridSegmentSelectionDrag.test.ts`.
- `e2e/specs/grid-sentence-menu.e2e.mjs` -- ba ca chưa chạy; `browser.keys(['Meta', 'm'])` có thể không mang `metaKey`.
- `_bmad-output/implementation-artifacts/14-2-ban-do/` -- khuôn bàn đo chuột thật. Phải kiểm frontmost, và `cargo test` ghi đè binary `wdio`.

## Tasks & Acceptance

**Execution:**
- [x] `tickets/epic-san-chuot/tickets/story-refactor-sweep-plan.md` -- thêm bảng 44 id → tệp và tên ca R7 vào Verification; id thiếu ca thì viết ca -- Done when 2 của epic chưa có bằng chứng theo từng id.
- [x] `e2e/specs/grid-sentence-menu.e2e.mjs` -- dịch `--features wdio`, chạy riêng spec, sửa ca nếu hợp âm không mang `metaKey` -- đóng mục nợ ba ca.
- [x] `_bmad-output/implementation-artifacts/14-10-ban-do/` -- bàn đo Cmd-Tab thật khi menu câu mở: đọc `focusout`, caret, kết quả bấm mục -- trả lời phát hiện maybe-false.
- [x] `src/panels/GridSentenceMenu.vue` + `tests/frontend/` -- chỉ khi tái hiện: `blur` của cửa sổ đóng menu lý do `'dismissed'`, ca đỏ khi gỡ -- đúng lời giải mục nợ.
- [x] `src/panels/segmentHistoryState.ts`, `src/SegmentHistoryOverlay.vue`, `tests/frontend/segmentHistoryOverlay.test.ts` -- lưu `Range` nằm trong đúng ô lúc mở, trả dạng thu về một điểm sau `back.focus()`, các nhánh Matrix mỗi nhánh một ca -- đóng phát hiện 14.4.
- [x] `e2e/specs/` -- một ca chạy thật hai đường mở rồi `Esc`, gõ, đọc văn bản ô; bàn đo `14-10-ban-do/` đo đường menu bằng chuột thật -- WebKit tự bôi chỉ lộ ở chuột thật.
- [x] `_bmad-output/initiative-auratranslate/deferred-work.md` -- đóng ba ca e2e, mục rời cửa sổ và mục `Esc`; mỗi phát hiện hoãn còn lại có dạng đóng hoặc chủ -- `check:debt-owner` đọc được.

**Acceptance Criteria:**
- Given cây thật, when `npm run check:commands`, then xanh và in `0 nợ`.
- Given bảng 44 id, when đọc từng dòng, then mỗi id trỏ tới một ca đã mở ra đọc, và lấy mẫu một id mỗi loại (`dispatch`, `aim`), gỡ thật thuộc tính hoặc handler làm đúng ca đó đỏ.
- Given spec `grid-sentence-menu`, when chạy ngoài sandbox, then tám ca xanh, hoặc ca đỏ được ghi nguyên nhân.
- Given phép đo Cmd-Tab, when đọc `ket-qua.md`, then ghi build, tải, số lần; sửa thì gỡ nghe `blur` làm đúng ca mới đỏ.
- Given bản sửa `Esc`, when gỡ thật lượt trả `Range`, then đúng các ca mới đỏ vì caret về đầu ô.
- Given `npm run check:debt-owner`, when chạy, then xanh và không còn phát hiện hoãn của 14.1–14.9 không chủ.
- Given commit của 14.10, when đẩy `master`, then `pre-push` xanh và CI đọc được cả nửa Windows, đỏ thì ghi nguyên nhân.

## Implementation Notes

- `check:commands` đọc `297 điểm chuột · 33 khai pointer · 0 nợ`. Bảng 44 id → ca R7 ở §Verification; các id `aim` lấy ca ở `importPreviewBlocks`, `importPreviewBilingual`, `gridSegmentSelectionDrag`. Đối chứng gỡ thật: bỏ `@click` của `ai.proofread.hide` làm đỏ đúng ca nút Ẩn; bỏ `@mousedown` nhắm khối ở `ImportPreviewOverlay.vue` làm đỏ 8 ca `importPreviewBlocks` (ca `aim` đứng đầu).
- Rời cửa sổ khi menu câu mở không tái hiện (3/3, Cmd-Tab thật): WebKit chỉ bắn `blur`, không `focusout`. Không thêm nghe `blur`; mục nợ ghi KHÔNG LÀM. Số đo ở `14-10-ban-do/ket-qua.md`.
- `Esc` ở Lịch sử: `openSegmentHistory` lưu điểm caret (thu về cuối vùng chọn) nếu Range nằm trong đúng ô bản dịch; `SegmentHistoryOverlay.vue` trả lại sau `back.focus()`. Lưu ở lúc mở, đồng bộ, vì mục menu chạy `history.open` TRƯỚC `removeAllRanges` của lý do `'overlay'`.
- Gỡ `placeCaret` làm đỏ 2 ca Vitest (Mod+H và mục menu) và ca e2e `⌘H` đọc `zzabc`; bàn đo chuột thật: `Ban zzdich 4.` và `Ban dichzz 5.` sau bản vá.
- e2e `grid-sentence-menu` 10/10 xanh trên Vite 1421 dựng mới (8 ca cũ, gồm ba ca nợ, và 2 ca mới). Trên Vite 1420 của `tauri dev` đỏ 9/10 vì tem `?t=`; ghi nợ `Chủ: Ice`. `browser.keys` không chèn chữ vào ô contenteditable, ca mới dùng `execCommand('insertText')`; gõ thật chỉ có ở bàn đo.
- Đường dựng e2e: `TAURI_CONFIG` đặt `devUrl` 1421 đổi binary debug `target/debug/auratranslate` sang bản trỏ 1421; `tauri dev` của Ice cần dựng lại.
- Đối chứng lượt vá, người điều phối chạy lại: gỡ `placeCaret(back, caret)` làm đúng hai ca e2e mới đỏ (`zzabc` ở `⌘H`, `startsWith('zz')` ở đường menu), tám ca cũ xanh; trả lại thì 10/10.
- Lượt kiểm lộ flake có từ trước ở hai ca 14.5 của `gridSentenceMenu.test.ts`: baseline đỏ 6/40, cây sau vá 4/32, tải 3–5. `press` xếp `reassert` một khung hình sau, ca đóng dải trước khi khung hình tới. Đường chỉ có ở test; thêm chờ khung hình, đỏ 0/40.

## Plan Change Log

## Review Triage Log

Lượt 1 (lens Quick cộng soát diff của người điều phối): high 0 · medium 3 · low 6 · false 7 · maybe-false 0.

- medium, patch: ca e2e đường menu xanh cả với `zzabc`, đúng lỗi đang sửa; thêm assert đỏ với `zzabc`.
- medium, patch: dòng `KHÔNG LÀM` khai đã gộp WebView2 vào mục `B7` nhưng không có dòng nào ở đó (người điều phối bắt); thêm dòng 🔵 vào mục `B7`, bỏ `→ Chủ: B7` lẻ ở mục `Esc` đã đóng.
- medium, patch: `KHÔNG LÀM` không nêu phạm vi đo; ghi `osascript`, về từ Finder, chưa đo Cmd-Tab người qua app thứ ba.
- low, patch: `!caret.startContainer.isConnected` là mã chết vì `Range` sống dời về cha của nút bị gỡ, `host.contains` đã đủ; xoá.
- low, patch: `activeElement` gán trên `document` không gỡ; gỡ ở `afterEach`, bỏ `void cell`.
- low, patch: ca `⌘H` dùng `KeyboardEvent` tổng hợp; đổi sang `browser.keys`, hoặc ghi lý do nếu không mở được lớp phủ.
- low, patch: comment `placeCaret` lặp lại hàm; rút còn hành vi nền tảng.
- low, patch: dòng 1 bàn đo mang story id; bỏ.
- low, patch: `ket-qua.md` trỏ `/tmp/e2e2.sh` không có trong repo; thay bằng các bước.
- false: hàng Matrix "phím không lọt" không có ca; `gridSentenceMenu.test.ts:550` khẳng định `rangeCount` 0 sau bấm mục Lịch sử, và `removeAllRanges` không đổi.
- false: hàng Matrix "Câu không còn trong Chương" không có ca; có ca "Chương đổi trong lúc mở".
- false: `caretRange` cũ khi mở và đóng cùng tick; `openSegmentHistory` luôn ghi đè nên không ai đọc giá trị cũ.
- false: tên Work `Story 14.10 — …` trái luật comment; đây là dữ liệu fixture, cùng khuôn tám tên có sẵn trong tệp, không phải comment.
- false: tải máy lệch giữa `ket-qua.md` và sổ nợ; hai con số thuộc hai lượt khác nhau, bàn đo và spec e2e.
- false: Task còn `[ ]`, Triage Log trống; sửa plan không phải phát hiện về mã.
- false: AC dựa vào ghi chú của tác giả; review chỉ đọc, người điều phối chạy lại cổng sau lượt vá.

## Verification

**Bảng 44 id → ca R7** (đã mở từng ca đọc; ca ở `tests/frontend/`; cột loại: `dispatch` = bấm bề mặt khẳng định lệnh chạy, `aim` = sự kiện chuột thật đổi con trỏ/vùng chọn)

| id | tệp | ca | loại |
|---|---|---|---|
| `editor.confirm_segment` | `gridSentenceMenu.test.ts` | `it.each(CASES)` `<id>: nhấn phụ + contextmenu thật ở câu 13, bấm mục, lệnh chạy khi caret là 13` | dispatch |
| `editor.merge_segments` | `gridSentenceMenu.test.ts` | `it.each(CASES)` `<id>: nhấn phụ + contextmenu thật ở câu 13, bấm mục, lệnh chạy khi caret là 13` | dispatch |
| `editor.split_segment` | `gridSentenceMenu.test.ts` | `it.each(CASES)` `<id>: nhấn phụ + contextmenu thật ở câu 13, bấm mục, lệnh chạy khi caret là 13` | dispatch |
| `editor.omit_segment` | `gridSentenceMenu.test.ts` | `it.each(CASES)` `<id>: nhấn phụ + contextmenu thật ở câu 13, bấm mục, lệnh chạy khi caret là 13` | dispatch |
| `editor.restore_segment` | `gridSentenceMenu.test.ts` | `it.each(CASES)` `<id>: nhấn phụ + contextmenu thật ở câu 13, bấm mục, lệnh chạy khi caret là 13` | dispatch |
| `editor.end_target_paragraph` | `gridSentenceMenu.test.ts` | `it.each(CASES)` `<id>: nhấn phụ + contextmenu thật ở câu 13, bấm mục, lệnh chạy khi caret là 13` | dispatch |
| `editor.join_target_paragraph` | `gridSentenceMenu.test.ts` | `it.each(CASES)` `<id>: nhấn phụ + contextmenu thật ở câu 13, bấm mục, lệnh chạy khi caret là 13` | dispatch |
| `editor.next_untranslated` | `gridSentenceMenu.test.ts` | `it.each(CASES)` `<id>: nhấn phụ + contextmenu thật ở câu 13, bấm mục, lệnh chạy khi caret là 13` | dispatch |
| `editor.clear_source_cuts` | `gridSentenceMenu.test.ts` | `it.each(CASES)` `<id>: nhấn phụ + contextmenu thật ở câu 13, bấm mục, lệnh chạy khi caret là 13` | dispatch |
| `editor.split_chapter` | `gridSentenceMenu.test.ts` | `it.each(CASES)` `<id>: nhấn phụ + contextmenu thật ở câu 13, bấm mục, lệnh chạy khi caret là 13` | dispatch |
| `history.open` | `gridSentenceMenu.test.ts` | `it.each(CASES)` `<id>: nhấn phụ + contextmenu thật ở câu 13, bấm mục, lệnh chạy khi caret là 13` | dispatch |
| `glossary.add_term` | `gridSentenceMenu.test.ts` | `it.each(CASES)` `<id>: nhấn phụ + contextmenu thật ở câu 13, bấm mục, lệnh chạy khi caret là 13` | dispatch |
| `segment.selection.extend_down` | `gridSegmentSelectionDrag.test.ts` | `kéo xuống: nhấn 11, qua 12, thả ở 13` · `Shift+bấm có vùng` | aim |
| `segment.selection.extend_up` | `gridSegmentSelectionDrag.test.ts` | `kéo ngược: nhấn 13, lên 11` · `Shift+bấm chưa có vùng` | aim |
| `editor.next_chapter` | `statusBarWorkspace.test.ts` | `bấm %s chạy đúng dep %s` (`data-chapter-next`) | dispatch |
| `editor.prev_chapter` | `statusBarWorkspace.test.ts` | `bấm %s chạy đúng dep %s` (`data-chapter-prev`) | dispatch |
| `review.open` | `attributionOverlayActions.test.ts` | `review.open: the third cell dispatches it` | dispatch |
| `review.diff_next` | `reviewMode.test.ts` | `review.diff_next button: down twice then up walks 2/3 then 1/3` | dispatch |
| `review.diff_prev` | `reviewMode.test.ts` | `review.diff_prev button: says the first and last boundary sentences` | dispatch |
| `glossary.settings.open` | `titlebarToolsMenu.test.ts` | `the item sits under Quản lý Glossary and dispatches glossary.settings.open` | dispatch |
| `import.preview.block_next` | `importPreviewBlocks.test.ts` | `mousedown thật trên khối 3 nhắm khối 3 ... J tới khối 4` | aim |
| `import.preview.block_prev` | `importPreviewBlocks.test.ts` | `mousedown thật trên khối 3 nhắm khối 3 ... K về khối 2` | aim |
| `import.preview.block_toggle_kept` | `importPreviewBlocks.test.ts` | `(i) Loại/Giữ khối này bắn block_toggle_kept` | dispatch |
| `import.preview.block_mark_range_start` | `importPreviewBlocks.test.ts` | `(i) Đầu vùng giữ bắn block_mark_range_start` | dispatch |
| `import.preview.block_confirm_range` | `importPreviewBlocks.test.ts` | `(i) Cuối vùng giữ ... bắn block_confirm_range` | dispatch |
| `import.preview.jump_to_cleanup_rules` | `importPreviewBlocks.test.ts` | `(i) Tới luật làm sạch bắn jump_to_cleanup_rules` | dispatch |
| `import.preview.chapter_next` | `importPreviewBlocks.test.ts` | `(i) ‹ › bắn chapter_prev / chapter_next, tắt ở hai đầu` | dispatch |
| `import.preview.chapter_prev` | `importPreviewBlocks.test.ts` | `(i) ‹ › bắn chapter_prev / chapter_next, tắt ở hai đầu` | dispatch |
| `import.preview.bilingual_caret_left` | `importPreviewBilingual.test.ts` | `mousedown trái trên đích nhắm caret, ← → đi tiếp từ đó, 0 IPC` | aim |
| `import.preview.bilingual_caret_right` | `importPreviewBilingual.test.ts` | `mousedown trái trên đích nhắm caret, ← → đi tiếp từ đó, 0 IPC` | aim |
| `import.preview.bilingual_toggle_cut` | `importPreviewBilingual.test.ts` | `(i) bấm đích bắn bilingual_toggle_cut` | dispatch |
| `import.preview.bilingual_next_mismatch` | `importPreviewBilingual.test.ts` | `(i) Cặp lệch trước/sau bắn hai lệnh` | dispatch |
| `import.preview.bilingual_previous_mismatch` | `importPreviewBilingual.test.ts` | `(i) Cặp lệch trước/sau bắn hai lệnh` | dispatch |
| `layout.toggle_ai_translation` | `titlebarToolsMenu.test.ts` | `the layout.toggle_ai_translation switch reports the dock and toggles panel.ai_translation` | dispatch |
| `layout.toggle_grid` | `titlebarToolsMenu.test.ts` | `the layout.toggle_grid switch ...` (cùng `for PANEL_SWITCHES`) | dispatch |
| `layout.toggle_lookup` | `titlebarToolsMenu.test.ts` | `the layout.toggle_lookup switch ...` (cùng `for PANEL_SWITCHES`) | dispatch |
| `ai.proofread.hide` | `proofreaderStripContent.test.ts` | `the hide button dispatches ai.proofread.hide without taking focus from the cell` | dispatch |
| `library.chapters.clear_selection` | `libraryChapters.test.ts` | `nút Bỏ chọn chỉ hiện khi có chọn và dispatch library.chapters.clear_selection` | dispatch |
| `reading.marks_clear_cancel` | `readingMarks.test.ts` | `the Huỷ button shows only while clearing is armed and dispatches reading.marks_clear_cancel` | dispatch |
| `export.alignment.prev` | `alignment.test.ts` | `the Mục trước button hides at the first item and dispatches export.alignment.prev from the second` | dispatch |
| `prompt.import.open` | `promptLibraryOverlayRender.test.ts` | `nút Nhập từ file dispatch prompt.import.open` | dispatch |
| `prompt.library.close` | `promptLibraryOverlayRender.test.ts` | `it.each(closeVia)` `nút Đóng` · `phím tự gán: dispatch("prompt.library.close")` | dispatch |
| `tm.manage.edit` | `tmManage.test.ts` | `Sửa on another row edits that row, not the row under the cursor` | dispatch |
| `tm.manage.delete` | `tmManage.test.ts` | `Xoá needs two presses on the same row ...` | dispatch |

**Commands:**
- `npm run check:commands` -- expected: thoát 0, dòng `297 điểm chuột · 33 khai pointer · 0 nợ` (số đổi theo phạm vi thì ghi lại).
- `npm run test:story 14-2`, tương tự `14-3` tới `14-9` -- expected: xanh; đây là "test của các story vá vẫn xanh".
- `npm run check:debt-owner` -- expected: thoát 0.

- Agent triển khai không commit, không push. Người điều phối đẩy `master` sau review: `pre-push` xanh, rồi `gh run list` đọc lượt `push` của commit cuối.

**Manual checks (if no CLI):**
- `ket-qua.md` của bàn đo: frontmost trước mỗi cú bấm.
