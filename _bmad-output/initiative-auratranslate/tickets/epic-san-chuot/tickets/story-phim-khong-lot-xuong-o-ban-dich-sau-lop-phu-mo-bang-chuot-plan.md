---
title: 'Phím không lọt xuống ô bản dịch sau lớp phủ mở bằng chuột'
type: 'bugfix'
ticket: '11'
created: '2026-11-01'
status: 'built'
baseline_revision: '80ac56bd40c32eef68d8acb69a5a8072de2217cc'
route: 'full'
route_source: 'auto'
risk: 'high'
review: 'quick'
review_source: 'pinned'
lenses_ran: ['quick']
review_loop_iteration: 1
context: []
---

<frozen-after-approval reason="human-owned intent — do not modify unless human renegotiates">

## Intent

**Problem:** WebKit giữ vùng chọn trong ô bản dịch khi cú bấm chuột mở lớp phủ, nên phím gõ lúc lớp phủ đang hiện chui xuống ô phía sau (đo `abc` thành `abcQ`), đi vào bộ đệm gõ rồi tự lưu, không dấu hiệu nào. Mới chặn cho mục Lịch sử của menu câu; 12 lớp phủ khác mở được từ Workspace bằng chuột chưa đo.

**Approach:** Đo mọi đường trước khi sửa. Khi lớp phủ mở, giữ điểm caret rồi xoá vùng chọn tại một chỗ chung mà cả 22 lớp phủ đã đi qua; khi tiêu điểm về lại đúng ô, trả caret. Bản riêng của Lịch sử gộp về chỗ đó. Bề mặt không phải lớp phủ chỉ sửa nếu phép đo cho thấy lọt.

**Decisions (Ice, 2026-11-01):** đóng lớp phủ trả caret về chỗ cũ ở cả đường chuột lẫn đường phím; đây là phần nới duy nhất của "mọi đường bàn phím không đổi".

**Decisions (Ice, 2026-11-01, sau bàn đo):** đang giữ caret thì đóng lớp phủ hay menu đều đưa tiêu điểm về ô, đúng điểm cũ, thay cho về nút mở; về nút mở theo UX-DR17 chỉ còn khi không giữ caret nào. Mục menu không mở lớp phủ (hộp thoại tệp gốc) cũng trả caret.

## Boundaries & Constraints

**Always:** Lớp phủ có ô nhập vẫn nhận phím ở ô đó. Caret trả về dạng một điểm, vì WebKit có thể đã tự bôi một từ khi chuột phải và gõ tiếp không được đè chữ. Lớp phủ lồng trong lớp phủ không làm mất điểm đã giữ. `src/commands/focus.ts` vẫn nạp được bằng Node thuần.

**Never:** Không chặn phím ở tầng `keydown` hay `beforeinput` của ô: đó là đường gõ của AD-35/AD-47. Không đổi Quick add, dải xác nhận Glossary, TM fuzzy: chúng tự lo caret. Không thêm `aria-modal` cho các lớp phủ còn thiếu. Không đổi hiệu năng gõ trong lưới.

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected Output / Behavior | Error Handling |
|----------|--------------|---------------------------|----------------|
| Mở bằng chuột | Caret giữa ô, bấm nút mở lớp phủ, gõ `Q` | Chữ ô và `target_text` không đổi | Không lỗi |
| Đóng rồi gõ | Như trên, `Esc`, gõ `zz` | `zz` chèn đúng điểm cũ | Không lỗi |
| Mở bằng phím | Hợp âm mở, `Esc`, gõ | Cùng kết quả đường chuột | Không lỗi |
| WebKit tự bôi | Chuột phải bôi một từ, mục Lịch sử, `Esc`, gõ | Chèn sau từ, không đè | Không lỗi |
| Lồng | Prompt Library mở Nhập Prompt, đóng cả hai | Caret về chỗ cũ | Không lỗi |
| Không có caret | Mở lớp phủ khi tiêu điểm ngoài lưới | Không giữ gì, hành vi cũ | Không lỗi |
| Ô đã rời DOM | Lớp phủ đổi Chương rồi đóng | Không trả caret, không ném | Bỏ điểm đã giữ |
| Không giữ caret | Mở lớp phủ khi không có caret trong ô, đóng | Tiêu điểm về nút mở theo UX-DR17 | Không lỗi |
| Đóng menu | Caret trong ô, mở `Công cụ ▾` hoặc menu nguồn Tra cứu, đóng không chọn, gõ `zz` | `zz` chèn đúng điểm cũ | Không lỗi |
| Mục không có lớp phủ | Mục Nhập bản góp ý mở hộp thoại tệp gốc, huỷ, gõ `zz` | `zz` chèn đúng điểm cũ | Không lỗi |

</frozen-after-approval>

## Code Map

- `src/commands/focus.ts:413` `focusReturnTargetOnOpen` -- 22/22 `*Overlay.vue` gọi trong `watch` lúc mở, trước `nextTick(panel.focus())`: đây là chỗ chung. Chỉ import `registry.ts`; không được import giá trị từ `vue` hay `src/panels/*`.
- `src/panels/segmentHistoryState.ts:37-56`, `:146` `caretPointInTargetCell`, `takeHistoryCaretRange` -- bản riêng, chuyển về chỗ chung. `src/SegmentHistoryOverlay.vue:66,84,91` `placeCaret` -- gỡ.
- `src/panels/sentenceMenuState.ts:128` nhánh `'overlay'` -- GIỮ `removeAllRanges`. `@click` mục menu chạy `history.open`, cú bấm thật còn chạy microtask giữa hai listener, nên `openSegmentHistory` vẫn phải lưu đồng bộ trước nhánh này.
- `src/panels/GridPanel.vue:1234` `onColumnFocusOut` -- đặt `editorCaretSegmentId` `null` khi tiêu điểm rời cột; nên chỗ chung đọc DOM, không đọc state đó. `:1428` `selectionchange` đặt lại caret theo anchor.
- `src/panels/GridBlock.vue:283` -> `GridPanel.vue:1815,1802` -> `editorPanelState.ts:793,831` -- đường phím lọt xuống đĩa; auto-save 2 s, trần 5 s.
- `src/main.ts:1137` `isBlocked` -- chỉ `keydown`, không `preventDefault`, không phải cửa chặn.
- Đường mở Workspace bằng chuột: `TitlebarToolsMenu.vue:122,131,140,149,163,178,193,208,223`; `StatusBar.vue:740,749`; `panels/AiTranslationPanel.vue:421,433,442,473`; `panels/LookupPanel.vue:407`; `panels/GridSentenceMenu.vue:182`; `ProofreaderStrip.vue:211`; `modes/WorkspaceMode.vue:128`; `layout/ReviewDock.vue:123`.
- Ngoài lớp phủ, chưa đo: `layout/LookupDrawer.vue:61`, `TitlebarToolsMenu.vue:111`, `panels/LookupSourcesMenu.vue:101` lấy tiêu điểm mà không giữ Range; tab chế độ `App.vue:213-234`, `<KeepAlive>` + `v-if`, không gì xoá vùng chọn.
- `scripts/check-layout.mjs:441` -- `document.addEventListener` đã được phép.
- `_bmad-output/implementation-artifacts/14-10-ban-do/chuot-that.e2e.mjs` -- khuôn bàn đo. Con trỏ của driver không tin cậy và `browser.keys` không chèn vào ô, gõ thật chỉ có ở `osascript`. Kiểm frontmost; `cargo test` ghi đè binary `wdio`; tắt `tauri dev` hoặc dùng Vite dựng mới.

## Tasks & Acceptance

**Execution:**
- [x] `_bmad-output/implementation-artifacts/14-11-ban-do/` -- bàn đo trước khi sửa: 13 lớp phủ mở trực tiếp từ Workspace, ngăn kéo Tra cứu, menu `Công cụ ▾` đang mở, menu nguồn Tra cứu, hai tab chế độ; mỗi đường gõ `Q` khi đang hiện rồi đọc ô, `Esc`, gõ `zz`; thêm ba đường hợp âm -- lập bảng lọt/không lọt để biết phạm vi thật.
- [x] `src/commands/focus.ts` -- thêm giữ caret: `focusReturnTargetOnOpen` lưu điểm trong ô `[data-col="tgt"]` chứa vùng chọn rồi xoá vùng chọn; không có vùng chọn trong ô thì giữ nguyên điểm cũ; nghe `focusin` một lần để trả điểm khi đúng ô nhận tiêu điểm -- một chỗ, không sửa 22 tệp.
- [x] `src/panels/segmentHistoryState.ts`, `src/SegmentHistoryOverlay.vue` -- gộp về chỗ chung, gọi lưu đồng bộ ở `openSegmentHistory` -- một fact một chỗ.
- [x] Các bề mặt bảng đo báo lọt (`LookupDrawer.vue`, `TitlebarToolsMenu.vue`, `LookupSourcesMenu.vue`, tab chế độ) -- gọi cùng hàm giữ caret; không lọt thì ghi số đo, không sửa -- không sửa thứ chưa hỏng.
- [x] `tests/frontend/overlayCaretHold.test.ts` -- mỗi dòng Matrix một ca qua lời gọi thật, gồm mount ít nhất hai lớp phủ; một ca đọc dòng mã mọi `src/*Overlay.vue` có gọi `focusReturnTargetOnOpen` -- lớp phủ mới không quên được.
- [x] `e2e/specs/overlay-caret-hold.e2e.mjs` -- trên WKWebView, mở bằng hợp âm và bằng nút, `Esc`, chèn chữ, đọc ô -- khoá lượt trả caret.
- [x] `_bmad-output/initiative-auratranslate/deferred-work.md` -- đóng mục, bề mặt còn lại nếu có ghi chủ; `tests/frontend/segmentHistoryOverlay.test.ts` sửa theo chỗ chung.

**Acceptance Criteria:**
- Given bàn đo chuột thật sau khi sửa, when chạy mọi đường đã liệt kê, then không đường nào đổi chữ của ô, `target_text` đọc lại từ đĩa không đổi, và `Esc` rồi gõ chèn đúng chỗ cũ; `ket-qua.md` ghi build, tải, số lần, số lượt mất cú bấm.
- Given gỡ thật lượt xoá vùng chọn khỏi chỗ chung, when chạy ca Vitest và bàn đo, then ca đỏ và bàn đo đọc lại `abcQ`.
- Given gỡ thật lượt trả điểm, when chạy e2e, then đúng các ca trả caret đỏ vì caret về đầu ô.
- Given một `*Overlay.vue` mới không gọi `focusReturnTargetOnOpen`, when chạy ca đọc mã, then ca đỏ nêu tên tệp.
- Given lớp phủ có ô nhập, when mở và gõ, then chữ vào ô nhập.

**Vòng 2 (bad_plan, xem Plan Change Log; mọi ô dưới đây chưa làm):**
- [x] `src/commands/focus.ts` -- gỡ `holdTargetCaretOnPointerDown`, cờ tạm, `KEEPS_FOCUS`, `returnFocusToHeldCell`; thêm `restoreHeldSelection()` theo Design Notes; ô đã rời DOM thì `focusReturnTargetOnOpen` bỏ điểm giữ; sửa docblock `LUẬT` của `focusReturnTargetOnOpen` cho đúng nhánh mới; doc comment mới tối đa hai dòng.
- [x] `src/StatusBar.vue`, `src/ProofreaderStrip.vue`, `src/TitlebarToolsMenu.vue`, `src/panels/LookupSourcesMenu.vue` -- trả về đúng baseline, không còn dòng giữ caret ở `mousedown`.
- [x] `src/titlebarToolsState.ts`, `src/panels/dictSourcesState.ts` -- gọi `restoreHeldSelection()` khi đóng.
- [x] `tests/frontend/overlayCaretHold.test.ts` -- bỏ `vi.resetModules()` hoặc import động để chỉ còn một thể hiện `focus.ts`; mount thật ít nhất hai lớp phủ, gồm `SegmentHistoryOverlay` đi hết đường đóng; ca riêng cho mục không mở lớp phủ; ca menu mở và đóng bằng hợp âm khi ô giữ tiêu điểm; ca menu sang nút khác; ca bấm vào nguồn đặt vùng chọn mới thì không trả; ca nút `Đã xong` không đổi vùng chọn; ca lớp phủ có ô nhập nhận chữ; ca ô rời DOM khẳng định đã bỏ điểm giữ.
- [x] `e2e/specs/overlay-caret-hold.e2e.mjs` -- bỏ `osascript`, `pgrep`, `execFileSync`, dùng `browser.keys`; ba ca nút dựng đúng trạng thái chuột thật trong `browser.execute`: tiêu điểm ở nút, `Range` trong ô, rồi mới bắn `click`, vì driver làm WebKit xoá vùng chọn.
- [x] `_bmad-output/implementation-artifacts/14-11-ban-do/` -- thêm dòng ngăn kéo Tra cứu (thu cửa sổ tới tầng có nút), dòng bấm `Đã xong` rồi gõ, dòng `Công cụ ▾` sang nút thanh trạng thái, dòng ô nhập trong lớp phủ, nút nhập ở thông báo Review Mode nếu dựng được (không dựng được: dòng `skipped` trong `ket-qua-sau2.json`, ghi ở `ket-qua.md`); chạy lại trọn bảng "sau" trên Vite mới; chạy thêm một lượt chỉ gỡ `removeAllRanges` và ghi số `Q` lọt; viết lại `ket-qua.md`, bỏ câu sai về `role="dialog"`.
- [x] `_bmad-output/initiative-auratranslate/deferred-work.md` -- bỏ dòng ✅ cũ; dùng `→ 🟡` nếu còn bề mặt chưa đo, nêu từng bề mặt và `Chủ: Epic 14`; đo đủ mới được ✅.

## Implementation Notes

- Chỗ chung là `holdTargetCaret()` trong `src/commands/focus.ts`, gọi từ `focusReturnTargetOnOpen`; 22/22 `*Overlay.vue` đi qua, không sửa tệp nào của chúng. Khi đang giữ caret mà tiêu điểm chưa ở trong `[role="dialog"]`, `focusReturnTargetOnOpen` trả chính ô đã giữ (lớp phủ lồng vẫn trả lớp cha); không giữ caret thì về nút mở theo UX-DR17; ô đã rời DOM thì bỏ điểm giữ.
- Điểm trả ở một `focusin` vào đúng ô đã giữ; tiêu điểm sang ô khác thì bỏ. Điểm bỏ thêm ở `resetSegmentHistory()`. Không thấy vùng chọn trong ô thì giữ điểm cũ, để mục menu câu (xoá vùng chọn trước `watch`) và lớp phủ lồng không mất điểm.
- Hai menu không phải lớp phủ (`Công cụ ▾`, nguồn Tra cứu) giữ caret khi mở và gọi `restoreHeldSelection()` khi đóng: hoãn một macrotask, bỏ qua nếu có hộp thoại, bỏ điểm nếu người dùng đã đặt vùng chọn mới, còn lại `addRange`. Không đụng tiêu điểm, không giữ ở `mousedown`.
- Gốc của "mở bằng chuột, ô vẫn nhận phím và `Esc` không đóng" (đo, không đoán): vùng chọn còn trong contenteditable nên phím theo nó vào ô, cú gõ đưa `activeElement` về ô. `selectionchange` ở `GridPanel.vue` không giành lại tiêu điểm.
- Vòng 1 dựng `holdTargetCaretOnPointerDown` và `returnFocusToHeldCell` vì wdio làm mất vùng chọn ở `mousedown`; chuột thật không. Vòng 2 gỡ cả hai; e2e dựng lại trạng thái chuột thật (`pressWithSelection`), kể cả khi mở menu, không thì ca menu đỏ vì chưa giữ được gì.
- Bàn đo chuột thật vòng 2 (`ket-qua.md`): mọi đường đo được không lọt `Q`; `zz` đúng điểm cũ ở mọi dòng trừ hai tab chế độ (tiêu điểm ở tab) và Lịch sử qua chuột phải (chèn sau từ WebKit bôi, đúng Matrix). Gỡ `removeAllRanges` cho `Q` lọt 17/24 dòng.
- Đối chứng gỡ thật trên e2e: trả điểm ở `onHeldFocusIn` đỏ 4 ca; `addRange` ở `restoreHeldSelection` đỏ đúng ca menu; dòng trả ô trong `focusReturnTargetOnOpen` không đỏ ca e2e nào (`activeElement` đã là ô), chỉ Vitest khoá nó; e2e overlay xanh khi gỡ `removeAllRanges` vì `execCommand` theo tiêu điểm, nên AC 2 dựa vào bàn đo chuột thật.
- Chưa đo: liên kết bộ nhớ ở dải soát và hai nút Review Mode (cần dải soát/Review Mode trong quần thể bàn đo), chữ vào ô nhập của Glossary và Cài đặt, hai lớp phủ nhập cần hộp thoại tệp. Nợ `Chủ: Epic 14`. Dòng Matrix "Mục không có lớp phủ" do bàn đo `tools.export.reviewer_import.open` khoá, không có ca Vitest riêng.
- Menu nguồn Tra cứu giữ caret ở `LookupSourcesMenu.vue` (watch đồng bộ), nên hợp âm gán cho `lookup.toggle_sources_menu` khi panel không gắn không xoá vùng chọn.
- `--spec` của wdio không thay `specs` của cấu hình; chạy một tệp bằng cấu hình tạm ngoài cây import `config` rồi ghi đè `specs`.

## Plan Change Log

- 2026-11-01, vòng 1 sang 2. Phát hiện: Quick review, các dòng high và medium ở Triage Log. Sửa: Design Notes đoạn menu, Tasks vòng 2. Trạng thái xấu cần tránh: `mousedown` xoá vùng chọn trên nút không mở gì; bộ đếm 0 ms giành tiêu điểm giữa cú bấm; mã sản phẩm dựng chỉ để driver qua e2e. Bị thay: ghi chú số 3 ở Implementation Notes (phần `returnFocusToHeldCell`, bốn nút `mousedown`).
  KEEP: `holdTargetCaret()` trong `focusReturnTargetOnOpen`; nhánh trả ô đang giữ khi tiêu điểm chưa ở `[role="dialog"]`; `focusin` chỉ trả đúng ô đã giữ, ô khác thì bỏ; không có vùng chọn thì giữ điểm cũ; gọi đồng bộ ở `openSegmentHistory`, `dropHeldCaret` ở `resetSegmentHistory`; gộp bỏ bản riêng của Lịch sử; bảng "trước" và `ket-qua-truoc.json`; khung `chuot-that.e2e.mjs`, ca Vitest còn đúng; kết luận gốc `Esc` không đóng lớp phủ.
  Lệch quy trình: `src/` không được gỡ trước vòng 2 vì lệnh gỡ hỏng ở shell; vòng 2 sửa tiếp trên mã vòng 1, các task vòng 2 viết dạng gỡ nên vẫn đúng.

## Review Triage Log

Vòng 1, quick: 4 high · 8 medium · 5 low · 0 false · 0 maybe-false.
- high, bad_plan: `StatusBar.vue` `focusOnPointerDown` dùng chung với `data-chapter-done`, `data-reading-walk-next`, hai nút không mở gì; `holdTargetCaret(true)` xoá vùng chọn ở `mousedown`, sau `click` bỏ điểm, caret mất. Đã đọc mã, đúng.
- high, bad_plan: `returnFocusToHeldCell` `KEEPS_FOCUS` thiếu `[tabindex]`; bộ đếm 0 ms chạy giữa `mousedown` và `mouseup`, kéo tiêu điểm vào ô khi bấm bôi nguồn. Đúng theo `TitlebarToolsMenu.vue:65` đóng ở `mousedown`.
- high, bad_plan: từ menu mở sang nút khác: bộ đếm thấy nút là `taken` nên bỏ điểm, hoặc giành tiêu điểm khỏi menu vừa mở. Cùng gốc với dòng trên.
- high, bad_plan: `holdTargetCaretOnPointerDown` là mã sản phẩm dựng cho driver; `ket-qua.md` đo chuột thật không mất vùng chọn ở `mousedown`. Người điều phối thêm vào lượt triage.
- medium, bad_plan: hợp âm gán cho `titlebar.tools.toggle`: `cell.focus()` trên ô đang giữ tiêu điểm không bắn `focusin`, không trả điểm. 237 lệnh `keys: undefined` gán được phím.
- medium, bad_plan: AC 1 chưa đạt: ngăn kéo Tra cứu, liên kết bộ nhớ, hai nút Review Mode chưa đo, "đi qua cùng hàm" là suy luận.
- medium, bad_plan: AC 2 thiếu nửa bàn đo khi chỉ gỡ `removeAllRanges`.
- medium, bad_plan: AC 5 không có bằng chứng.
- medium, bad_plan: mục nợ ghi ✅ mà còn "Chưa đo", không `Chủ:`.
- medium, bad_plan: e2e dùng `osascript`/`System Events`, runner không cấp quyền; `browser.keys` đã mang hợp âm này.
- medium, bad_plan: `vi.resetModules()` tạo hai thể hiện `focus.ts`, ca "mục mở lớp phủ" không chạm nhánh `held !== null`.
- medium, bad_plan: task đòi mount hai lớp phủ, tệp chỉ mount một.
- low, patch trong vòng 2: ca "mục không mở lớp phủ" trùng ca menu đầu.
- low, patch trong vòng 2: ô rời DOM chưa bỏ điểm giữ ở đường lớp phủ, không có hại thấy được.
- low, patch trong vòng 2: ba doc comment ba dòng, docblock `LUẬT` cũ.
- low, patch trong vòng 2: `ket-qua.md` dòng 54 nói sai phạm vi `role="dialog"`.
- low, patch trong vòng 2: AC 3 ghi bằng chứng từ các phép gỡ khác nhau; ghi lại đúng phép gỡ lượt trả điểm.

Vòng 2, quick: 0 high · 7 medium · 7 low · 0 false · 0 maybe-false; 1 low bác, 1 medium hoãn, còn lại patch.
- low, bác: `addRange` kéo tiêu điểm vào ô giữa cú bấm từ menu sang nút khác. Có thật ở WKWebView, nhưng chỉ khi đang giữ caret, cú bấm vẫn tới nút và phím về ô là chỗ người dùng đang gõ; sửa cần thêm nhánh.
- medium, patch: hợp âm gán cho menu nguồn khi `LookupPanel` không gắn: `toggleSourcesMenu` xoá vùng chọn mà không có menu nào hiện, gõ tắt im lặng.
- medium, patch: `focusReturnTargetOnOpen` hỏi `activeElement` trong hộp thoại; tiêu điểm rơi `BODY` thì lớp phủ lồng trả về ô phía sau lớp phủ ngoài.
- medium, defer đã có chủ: AC 1, ba đường `strip.memory`, `review.notice_import`, `review.align_link` chưa đo; mục nợ `→ 🟡`, `Chủ: Epic 14`.
- medium, patch: AC 5 chưa có bằng chứng, ca Vitest gán rồi đọc lại `input.value`.
- medium, patch: `pressWithSelection` không dựng được trạng thái nó nói, vì `addRange` đưa tiêu điểm về ô; tên ca và comment nói sai, nhánh trả ô do Vitest và bàn đo canh.
- medium, patch: ca `Đã xong` bắn sự kiện vào nút trần, không canh được lỗi vòng 1.
- medium, patch: ca đọc mã AC 4 tính cả comment một dòng và comment HTML.
- medium, patch: chưa có dòng WebView2 ở `B7`.
- low, patch: AC 3 đếm trên năm ca chứ không sáu, thiếu lý do đỏ.
- low, patch: ca `ô đã rời DOM` không khẳng định đã bỏ điểm giữ.
- low, patch: ca `mục không mở lớp phủ` vẫn trùng ca menu đầu.
- low, patch: `cell.isConnected` trong `heldCell` là mã chết vì `Range` sống dời về nút cha.
- low, patch: doc comment ba dòng; dòng `Ngoại lệ` tiếng Việt.

## Design Notes

Trả caret ở một `focusin` thay vì sửa `back.focus()` của 22 tệp. `focus()` đặt caret về đầu ô rồi bắn `focusin` đồng bộ, nên listener trả điểm ngay sau đó. Bấm chuột vào ô: `focusin` tới trước khi trình duyệt đặt caret theo cú bấm, nên điểm bấm thắng. Listener chỉ gắn khi đang giữ một điểm.

Menu đóng chỉ trả vùng chọn, không đụng tiêu điểm: `restoreHeldSelection()` hoãn một macrotask để lớp phủ kịp dựng; bỏ qua nếu có `[role="dialog"]`, bỏ điểm nếu `rangeCount > 0` vì người dùng vừa đặt vùng chọn mới, còn lại `addRange` rồi bỏ điểm. Baseline đã đo: vùng chọn trong ô thì phím theo nó vào ô dù tiêu điểm ở nút. Nên không còn giành tiêu điểm giữa cú bấm, và ô đang giữ tiêu điểm cũng nhận lại caret.

Không giữ caret ở `mousedown`: chuột thật còn nguyên vùng chọn tới `click`.

Điểm giữ thu về cuối vùng chọn trong mọi trường hợp. Đường chuột hôm nay giữ nguyên vùng chọn thật qua lớp phủ; thu lại đổi điều đó, nhưng gõ tiếp lên một vùng chọn được trả về sẽ đè chữ, đúng lỗi 14.10 tránh.

## Verification

**Commands:**
- `npx vitest run tests/frontend/overlayCaretHold.test.ts tests/frontend/segmentHistoryOverlay.test.ts` -- expected: xanh, kiểm mã thoát.
- `npm run test:story 14-11` -- expected: chọn đúng test các tệp đổi.
- `node -e "import('./src/commands/focus.ts')"` qua `npm run check:commands` -- expected: `focus.ts` vẫn nạp bằng Node.
- `npm run check:layout && npm run check:debt-owner` -- expected: xanh.

**Manual checks (if no CLI):**
- Bàn đo `14-11-ban-do/` chạy ngoài sandbox, app frontmost, `ket-qua.md` đủ bảng trước và sau.
