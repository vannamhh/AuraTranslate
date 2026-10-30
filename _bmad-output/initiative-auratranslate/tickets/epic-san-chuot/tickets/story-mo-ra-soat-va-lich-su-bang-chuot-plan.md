---
title: 'Mở Rà soát và Lịch sử bằng chuột'
type: 'feature'
ticket: '4'
created: '2026-10-30'
status: 'built'
baseline_revision: 'bb2fa33ee9a237e2c803f01e4ad278dd35529a89'
route: 'full'
route_source: 'auto'
risk: 'medium'
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

**Problem:** Review Mode chỉ mở bằng phím, hai bước dời diff và lịch sử phiên bản của câu cũng vậy, nên người dùng chuột không vào được Rà soát; hợp âm chuyển Chương còn đổi Chương của Editor trong khi màn Review giữ Chương cũ.

**Approach:** Thêm ô Review Mode vào công tắc bố cục cùng `review.close` ở cùng chỗ, hai nút dời diff trên `.review-bar`, mục Lịch sử phiên bản ở nhóm 2 của menu câu, rút 4 id khỏi `POINTER_DEBT`, và đóng mục nợ hợp âm chuyển Chương.

## Boundaries & Constraints

**Decision (repo chứng minh, không cần ký):** mục Lịch sử bỏ qua placement. `closeSentenceMenu('chosen')` chạy sau `dispatch` và đặt tiêu điểm vào ô bản dịch trong khi lớp phủ cũng đang lấy tiêu điểm (`story-menu-chuot-phai-…-plan.md` §Design Notes). Thay vào đó là lý do đóng mới: chỉ thu vùng chọn, vì từ WebKit bôi ở ô bản dịch sống qua lớp phủ và phím gõ sau khi đóng sẽ đè nó.

**Decision:** hai nút diff chặn mặc định `mousedown` như ‹ › của Story 14.3: `showCursor` chỉ trả tiêu điểm về panel khi `activeElement` rỗng, mà trên WebView2 nút lấy tiêu điểm. Ô Review Mode không chặn: Workspace thành `inert` nên tiêu điểm rời ô soạn thảo dù thế nào, và `previousOwner` lo đường về.

**Decision:** trong Review Mode, hai ô preset không mang `aria-pressed="true"`, không thì công tắc có hai ô đang chọn cùng lúc.

**Decision (Ice chọn 2026-10-30):** khi Review Mode mở, `editor.next_chapter` và `editor.prev_chapter` không đổi Chương và `.diff-status` nói phải đóng Review Mode trước. Một phiên Rà soát gắn với một Chương.

**Always:** `@click` là đúng một `dispatch` literal; `review.open`, `review.close` là cặp `<button v-if>` ở cùng chỗ (`EXPERIENCE.md` §Ô Review Mode). Cả cặp tắt khi `attributionIsOpen`, như hai ô preset. Biểu tượng vẽ tay trong `AppIcon.vue`, ô Review Mode là hai cột cạnh nhau; nút diff dùng `ui-icon-btn`. `title` theo `EXPERIENCE.md` §Voice and Tone: nhãn command kèm phím từ `shortcutRows`. Thông báo biên diff dùng đúng các câu đã có.

**Never:** Đổi `Alt+ArrowDown/Up`, `Mod+Alt+3`, `Mod+H`. Đổi `openReviewMode`, `closeReviewMode`, `jumpDiff`, `openSegmentHistory`. `POINTER_DEBT_BASELINE`. Ghi `tickets.toml`. Thêm mục Thêm thuật ngữ (Story 14.5). Đổi hành vi `openChapterById`.

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected Output / Behavior | Error Handling |
|----------|--------------|---------------------------|----------------|
| Mở bằng ô | Workspace, Chương có bản Reviewer | Dải Rà soát hiện, ô `aria-pressed="true"`, ô dispatch `review.close` | Không |
| Đóng bằng ô | Review Mode mở | Về Workspace, tiêu điểm về panel cũ, caret về câu trước khi mở | Không |
| Không vào được | Chưa mở Chương, chưa nhập, bản cũ, lỗi đọc | Lời báo đã có hiện ra, ô vẫn dispatch `review.open` | Lời báo có nút đóng |
| Lớp phủ Ghi công mở | `attributionIsOpen` | Cả ba ô tắt | Không |
| Dời diff | 3 thay đổi, bấm xuống hai lần rồi lên | `Khác biệt 2/3`, cuộn tới đúng cặp, tiêu điểm không đổi | Không |
| Biên diff | Ở khác biệt cuối hoặc đầu | Câu `review.diff_last`, `review.diff_first` | Không |
| Không có thay đổi | 0 thay đổi | Câu `review.diff_none` | Không |
| Lịch sử từ menu | Caret A, chuột phải câu B ở ba cột, chọn Lịch sử phiên bản | Lớp phủ hiện lịch sử câu B, tiêu điểm ở lớp phủ | Câu `history.empty` đã có |
| Hợp âm Chương trong Review | Review Mode mở, `Mod+Alt+]` hoặc `Mod+Alt+[` | Chương của Editor không đổi, `.diff-status` nói lý do | Không |
| Đóng lớp phủ | `Esc` sau đường trên, chuột phải ở ô bản dịch | Tiêu điểm về chỗ cũ, không còn từ bị bôi, phím gõ tiếp không đè chữ | Không |

</frozen-after-approval>

## Code Map

- `src/App.vue` -- công tắc bố cục :249–275; `reviewModeIsOpen` đã import :98. `src/AppIcon.vue` -- `AppIconName`:2, nhánh `app-icon--wide`:8, `chevron`:34 chỉ trỏ xuống.
- `src/layout/ReviewDock.vue` -- `.review-bar`:82 `align-items: baseline`, `.ui-icon-btn::before` có sẵn hộp dòng rỗng; nút chèn giữa `.diff-stats` và `.diff-status`.
- `src/panels/GridSentenceMenu.vue` -- nét ngăn :157 là chỗ thêm nhóm 2 cùng nét ngăn thứ hai; `onDocumentClick`:76 chọn lý do đóng; `holdFocus`:36 là khuôn.
- `src/panels/sentenceMenuState.ts` -- `closeSentenceMenu`:112. `src/SegmentHistoryOverlay.vue`:64 -- `returnFocusTo` đọc `activeElement` sau cú bấm, không cần sửa.
- `src/StatusBar.vue` -- `chapterNavTitle`:247 đem lên thành `commandTitle(id)` dùng chung ở `src/config/shortcutsState.ts`; ba bề mặt cùng cần và Story 14.6–14.9 còn cần.
- `src/reviewModeCommandDeps.ts`, `src/main.ts`:831, `src/reviewModeState.ts`:57 -- `diffMessage` là bề mặt nói lý do khi Review Mode mở. Thanh trạng thái không phải chỗ đó.
- `src/panels/editorPanelState.ts`:2156 -- `goToNextChapter`.
- `scripts/check-commands.mjs` -- `POINTER_DEBT`:2902 (chỉ xoá ở đây), `R_BUTTON_HOLD_FOCUS`:2430 đổi câu khỏi "status bar", thêm `src/layout/ReviewDock.vue::holdFocus` vào `HANDLER_TABLE`, `HANDLER_ATTR_FLOOR`:2875 theo công thức cổng in ra.
- `tests/frontend/attributionOverlayActions.test.ts`:575 -- khuôn `mountApp`, đang đếm 2 ô. `tests/frontend/reviewMode.test.ts` -- mount `ReviewDock` thật. `tests/frontend/gridSentenceMenu.test.ts` -- bảng ca (i) theo mục.
- `e2e/specs/story-3-5-review.e2e.mjs`, `e2e/specs/segment-history-restore.e2e.mjs` -- dựng fixture. `_bmad-output/implementation-artifacts/14-3-ban-do/chuot-that.e2e.mjs` -- khuôn bàn đo `cliclick`.

## Tasks & Acceptance

**Execution:**
- [x] `src/AppIcon.vue` -- thêm biểu tượng Review Mode và `chevron-up`.
- [x] `src/config/shortcutsState.ts`, `src/StatusBar.vue` -- `commandTitle(id)`, StatusBar dùng lại.
- [x] `src/App.vue` -- cặp nút `review.open`, `review.close`, sửa `aria-pressed` hai ô preset.
- [x] `src/layout/ReviewDock.vue` -- hai nút diff, `@mousedown="holdFocus"`, `data-` để test chọn.
- [x] `src/panels/sentenceMenuState.ts`, `src/panels/GridSentenceMenu.vue` -- lý do đóng mới, nhóm 2 với mục `history.open`, thuộc tính đánh dấu mục mở lớp phủ để Story 14.5 dùng lại.
- [x] `src/reviewModeState.ts`, `src/reviewModeCommandDeps.ts`, `src/main.ts`, `src/i18n/vi.json` -- hai dep chuyển Chương đi qua `reviewModeCommandDeps`, từ chối khi `status` là `open` và ghi `diffMessage`.
- [x] `scripts/check-commands.mjs` -- gỡ 4 id, cập nhật `HANDLER_TABLE`, sàn.
- [x] `tests/frontend/attributionOverlayActions.test.ts`, `tests/frontend/reviewMode.test.ts`, `tests/frontend/gridSentenceMenu.test.ts` -- ca (i) cho từng id, mọi hàng I/O Matrix, ca hợp âm chuyển Chương.
- [x] `e2e/specs/review-mouse.e2e.mjs` (mới) -- vào Review, dời diff, ra bằng cùng ô; chuột phải câu B, mở Lịch sử, thấy lịch sử B.
- [x] `_bmad-output/implementation-artifacts/14-4-ban-do/` -- bàn đo `cliclick`: bấm ô từ ô đang gõ rồi bấm lại, caret về đúng câu; chuột phải ô bản dịch, Lịch sử, `Esc`, gõ, không đè chữ.
- [x] `deferred-work.md` -- đóng mục 4 id và mục hợp âm; thêm lượt dùng thật vào mục `Chủ: Epic 14`; thêm mục `Chủ: Epic 14`: mở Chương khác cùng Tác phẩm từ Thư viện khi Review Mode mở có cùng lệch Chương; `EXPERIENCE.md` dòng 🔵 ở §State Patterns về hợp âm.

**Acceptance Criteria:**
- Given cây sau story, when chạy `npm run check:commands`, then Kiểm M xanh với 275 điểm chuột, 29 khai `pointer`, 26 nợ.
- Given app thật, when bấm ô Review Mode, bấm xuống và lên, rồi bấm lại chính ô đó, then dải Rà soát hiện, diff dời đúng, và Workspace trở về với caret ở câu cũ.
- Given app thật, when chuột phải vào một câu và chọn Lịch sử phiên bản, then lớp phủ hiện lịch sử đúng câu đó.
- Given ca (i) của một id, when gỡ `@click` khỏi nút của id đó, then chính ca đó đỏ.
- Given ca mục Lịch sử, when bỏ lý do đóng mới, then ca đỏ vì placement được yêu cầu.
- Given đang gõ trong ô, when so `gridTypingCost.test.ts` và `gridCaretMoveCost.test.ts`, then cả hai xanh không đổi ngưỡng.

## Implementation Notes

- `closeSentenceMenu` nhận lý do `overlay`; mục mang `data-opens-overlay` chọn lý do đó, Story 14.5 dùng lại thuộc tính này.
- `commandTitle(id)` ở `shortcutsState.ts` thay `chapterNavTitle`; ô Review Mode, hai nút diff và ‹ › cùng dùng.
- Hai dep chuyển Chương nằm trong `reviewModeCommandDeps` qua `refuseChapterMoveInReview`; `main.ts` không còn khai chúng.
- `titlebarToolsMenu.test.ts` cấm `review.open` trong header; đổi thành: chỉ nằm trong `.layout-switch`, đúng hai lời gọi.
- Đo chuột thật: chỉ thu vùng chọn thì phím gõ khi lớp phủ mở vẫn lọt xuống ô, vì WebKit giữ vùng chọn khi cú bấm đổi tiêu điểm; lý do `overlay` nên xoá hẳn vùng chọn. Đổi lại `Esc` đưa caret về đầu ô, đúng như `Mod+H`. Số đo ở `_bmad-output/implementation-artifacts/14-4-ban-do/ket-qua.md`.
- Sau khi đóng Review, tiêu điểm ở vỏ panel và viền caret chưa hiện; `returnToLastCaret` ở `reviewModeCommandDeps.ts` đưa vào ô và đặt lại caret, không đổi `closeReviewMode`. Vỏ panel khác giữ nguyên tiêu điểm.
- e2e và bàn đo trả lời `alignment_open`, `review_diff` ở `window.fetch` vì `invoke` của Tauri không ghi đè được và đường nhập bản Reviewer đi qua hộp thoại tệp gốc. `tauri dev` đang chạy không chặn e2e: `wdio.conf.mjs` dùng lại Vite của nó.
- `.review-bar` để nút canh baseline: thanh giữ 35px.

## Plan Change Log

## Review Triage Log

Lượt 1 (lens Quick): high 0 · medium 1 · low 5 · false 4 · maybe-false 0.

- medium, patch: `returnToLastCaret` với tiêu điểm ở `body` tìm thấy ô ở bất kỳ đâu và cướp tiêu điểm khi đóng ngoài Workspace; đã chặn `body`, thêm ca.
- low, Ice quyết: `Esc` sau mục Lịch sử đưa caret về đầu ô; bảng I/O không đòi giữ offset và `Mod+H` đã vậy, nhưng Decision đóng băng ghi "chỉ thu vùng chọn" trong khi số đo buộc xoá hẳn.
- low, patch: e2e chưa khoá "không còn từ bị bôi" sau `Esc`; đã thêm assert.
- low, patch: Implementation Notes 11 gạch đầu dòng và có thuật lại lệnh; đã cắt còn 8.
- low, patch: mục nợ và dòng 🔵 ghi 2026-10-30 trong khi đóng ngày 2026-10-31; đã sửa.
- low, reject: hợp âm trong lúc `status` là `loading` vẫn đổi Chương; cửa sổ một lượt IPC, chặn thêm phải có bề mặt báo riêng vì dải Rà soát chưa dựng.
- false: wiring `returnToLastCaret` không có gì canh; ca e2e "chính ô đó trả về đúng câu" đỏ trước khi có đường này.
- false: `history.open` không chạy trên câu B; ca (i) kiểm caret là câu nhắm và e2e đọc đúng bản của B.
- false: ca `holdFocus` xanh dù gỡ handler; ca đó kiểm `defaultPrevented` nên gỡ là đỏ.
- false: doc comment của `returnToLastCaret` chỉ tả lại hành vi; nó nói vì sao hàm tồn tại, thứ chữ ký không cho thấy.

## Design Notes

Bấm ô từ ô đang gõ làm `focusout` đặt caret `null`, nhưng `lastCaretSegmentId` còn nên `enterFocus` đưa về đúng câu lúc đóng. Vì suy từ mã nên bàn đo phải xác nhận trên WKWebView; đỏ thì sửa và ghi vào Implementation Notes.

Mở Lịch sử từ ô nguyên văn câu B khi caret ở câu A: tiêu điểm thật vẫn ở ô A, nên đóng lớp phủ trả về A. Đúng UX-DR17 (về chỗ cũ), và khớp "Đóng không chọn" của menu.

## Verification

**Commands:**
- `npm run check:commands` -- exit 0, in 275/29/26.
- `npm run check:i18n`, `npm run check:tokens`, `npm run check:debt-owner` -- exit 0.
- `npm run test:story 14-4` -- exit 0.
- `npx vue-tsc --noEmit`, `npm run check:lint` -- exit 0.
- `cargo build --locked --features wdio --manifest-path src-tauri/Cargo.toml`, rồi `npx wdio run e2e/wdio.conf.mjs --spec e2e/specs/review-mouse.e2e.mjs` -- xanh, chạy ngoài sandbox.
- Chạy đủ bộ một lần: story chạm `scripts/check-*` (AGENTS.md §Tests).

**Manual checks (if no CLI):**
- Lượt dùng thật của Ice trước khi Epic 14 `done`: ô Review Mode, hai nút diff, Lịch sử phiên bản từ menu câu.
