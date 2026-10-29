---
title: 'Chế độ đọc: double-click nguyên văn chữ Hán để tra, kết quả hiện trong popup'
type: 'feature'
ticket: ''
created: '2026-10-29'
status: 'built'
baseline_revision: 'd875bc494374db5d2e024cb063a583bfecd7783d'
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

**Problem:** Ở Chế độ đọc, người đọc gặp một từ Hán chưa hiểu trong nguyên văn song ngữ thì không tra được. Chế độ đọc không có Panel Lookup, và bôi đen ở đó không chạy tra cứu vì trang đọc không đăng ký bề mặt chọn chữ nào.

**Approach:** Ice quyết (2026-10-29):
- Double-click một từ trong nguyên văn chữ Hán ở lề song ngữ thì tra từ đó. Kéo chuột bôi đen thì không tra. Double-click vào bản dịch tiếng Việt cũng không tra: từ điển chỉ có đầu mục chữ Hán và tiếng Anh, và 5/5 từ tiếng Việt đo trên bản dịch của Ice đều cho 0 kết quả.
- Kết quả hiện trong một popup bám ngay cạnh từ vừa chọn, không dùng ngăn kéo Lookup.
- Popup có: từ được tra, âm đọc, nghĩa theo từng từ điển, tra từng chữ khi không thấy cả từ, nút ghim mục từ, và dòng trỏ sang Concordance TM. Không có dải tab, menu chọn nguồn hay lịch sử.
- Bỏ quy tắc cấm lớp nổi. Bỏ luôn lệnh cấm bóng đổ: bóng đổ được dùng tự do ở nơi cần tạo chiều sâu, chỉ cần giá trị lấy từ token. Gradient vẫn bị cấm.

## Boundaries & Constraints

**Always:**
- Lượt tra đi qua `dispatch('lookup.lookup_selection')` (Quyết định #4a ở `main.ts`), để lịch sử, ghim và các nguồn đang bật dùng chung một đường với Workspace. Mọi nút trong popup là `@click` gọi đúng một `dispatch` (`check:commands` Kiểm A).
- Nguyên văn là bề mặt chọn chữ vai `source` nhưng tắt Auto-Lookup: `mouseup` sau khi bôi đen không bao giờ phát lượt tra.
- Popup dùng nền, viền và bóng lấy từ token; `z-index` mang miễn trừ `aura-allow-z-index` có lý do.
- Đóng popup bằng Esc, bằng cú bấm ra ngoài (listener `mousedown` trên `document` ở pha capture, theo luật 4 của AD-58), khi tắt song ngữ, và khi rời Chế độ đọc. Một lượt double-click mới thay nội dung và vị trí popup.
- Dòng Concordance chuyển sang Workspace rồi mở tab Concordance cho từ đang tra.
- Thêm token là việc của AD-34: token bóng mới vào sổ `deviations` của `tokens.json`, ghi rằng Ice quyết ngày 2026-10-29.

**Never:**
- Không tra bản dịch tiếng Việt, không mở `LookupDrawer`, không gắn panel vào Chế độ đọc (FR11).
- Không chép lại logic tính kết quả của `LookupPanel.vue`: phần popup cần thì tách ra để panel và popup dùng chung.
- Không đổi `runLookup`. Không nới lệnh cấm gradient, và không cho bóng đổ mang giá trị viết tay.

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected Output / Behavior | Error Handling |
|----------|--------------|---------------------------|----------------|
| Tra trúng | Double-click `控制` trong nguyên văn | Popup cạnh từ: `控制`, âm đọc, nghĩa theo từng từ điển | Không |
| Không thấy cả từ | WebKit chọn một cụm không có trong từ điển | Popup báo không thấy và hiện từng chữ để bấm tra riêng | Không |
| Bôi đen | Kéo chọn một cụm trong nguyên văn rồi nhả chuột | Không tra, không popup, lịch sử không đổi | Không |
| Bản dịch | Double-click một từ tiếng Việt | Chỉ bôi đen như hiện nay, không popup | Không |
| Lỗi IPC | `lookup_dictionary` lỗi | Popup hiện câu lỗi của Panel Lookup (`panel.lookup.lookup_failed`) | Không popup trắng |
| Đóng | Esc, bấm ra ngoài, `B`, rời mode | Popup đóng, focus không nhảy khỏi trang đọc | Không |

</frozen-after-approval>

## Code Map

- `src/modes/ReadingMode.vue` -- `.source-note` (một `<p>` mỗi đoạn, trong `.reading-paragraph`); thêm `@dblclick` và chỗ gắn popup. Cuộn nằm ở `section.mode` (`root`).
- `src/panels/LookupPanel.vue:160-195` -- `groups`, `headReading`, `headPinTarget`/`headPinned`, `notFoundChars`, `sensesFor`: cục bộ, cần tách ra để dùng chung. Đầu mục: `:470-500`; tra từng chữ và dòng Concordance: `:585-605`; `LookupRecord` theo nhóm: `:606-613`.
- `src/panels/LookupRecord.vue` -- vẽ một nhóm nguồn kèm nghĩa; dùng lại nguyên vẹn.
- `src/panels/lookupPanelState.ts` -- `runLookup` (`:323`), `currentQuery`, `lookupDisplayable`, `layerSensesFailed`, `aimLookupCharFrom`.
- `src/selectionContract.ts:132,215-236,329-400` -- `useSelectionSurface`, `currentSelectionText()` (chỉ trả chữ trong bề mặt vai `source`), và watcher Auto-Lookup bắn khi `mouseup`. Cần một cách cho bề mặt tắt Auto-Lookup.
- `src/main.ts:1274-1287` -- Quyết định #4a và chỗ gắn watcher.
- `src/commands/index.ts` -- `lookup.lookup_selection` (`:2851`), `lookup.toggle_pin` (`:3079`), `lookup.lookup_char`, `tm.concordance` (`:3773`, dựa vào `tmConcordanceCommandDeps.ts:51`); lệnh mới đăng ký ở đây, nhãn ở `vi.json`.
- `scripts/check-commands.mjs` -- Kiểm F ghim số lần gọi `useSelectionSurface` theo tệp; Kiểm K liệt kê handler của `ReadingMode.vue` (`:2582`).
- `scripts/check-tokens.mjs:1438-1470` (Kiểm F) -- `BANNED_PROPS` chặn hẳn `box-shadow`/`text-shadow`; `BANNED_VALUE_RE` chặn gradient và `drop-shadow`.
- `src/tokens/tokens.json` + `src/tokens/index.ts` -- token và cách sinh biến CSS; `deviations`.
- `src/AGENTS.md:19`, `ux-auratranslate/DESIGN.md:158,376,527` -- câu cấm lớp nổi và bóng đổ.

## Implementation Notes

- Môi trường máy Ice: e2e phải chạy ngoài sandbox, dùng lại Vite của `tauri dev` đang mở ở cổng 1420; `tauri dev` có thể dựng lại `src-tauri/target/debug/auratranslate` mà thiếu feature `wdio`, khi đó WebDriver nhúng không lên ở cổng 4445 ⇒ chạy lại `cargo build --locked --features wdio`.
- Popup sống suốt lượt mở (chỉ gác theo neo + song ngữ); sau một double-click mới nó ẩn (`visibility: hidden`) tới khi lượt tra của chính lần mở đó trả lời, còn lượt tra từng chữ trong popup giữ nguyên popup và focus.
- Neo lưu bằng toạ độ nội dung của `section.mode` chứa nguyên văn ngay lúc mở (popup nằm trong `section.mode`, nay `position: relative`, cuộn theo trang); lật lên khi chỗ trên rộng hơn chỗ dưới, `max-height` bằng chỗ còn thấy được. Chữ dưới popup đổi dòng (`readingStyle`, `readingRun` đổi, cửa sổ đổi cỡ) thì popup đóng.
- Thác "hiện gì" của bản ghi (lỗi → … → kết quả) là `lookupBody` ở `lookupView.ts`; Panel Lookup và popup cùng rẽ nhánh theo nó.
- Dòng Concordance luôn hiện khi có từ đang tra (không chờ phép đếm `probeConcordanceCount`, vì watcher của Panel Lookup ở Workspace đặt lại phép đếm đó khi lượt tra trúng).
- Ô nhớ `anchor` đi qua `resetReadingLookup()` (luật của `check:panel-refs`); cổng lệnh `closeReadingLookup` trỏ vào nó.
- Ba sàn `judgeFloor` trôi vì thêm tệp/bề mặt và được nâng theo chính câu cổng in ra: `TS_FLOOR` và `FILE_FLOOR` 121→131 (153 tệp `.ts`), `SELECTION_SURFACE_FLOOR` 19→21 (24 lời gọi).
- AD-58 luật 4 nay có chỗ thứ tư (`ReadingLookupPopup.vue`), ghi 🔵 trong spine.
- e2e chưa xanh (2026-10-29): ca có sẵn của `story-5-11` đỏ ở bước chờ flush trước khi tới Chế độ đọc, y hệt cả khi trả fixture về `Cau mot。Cau hai。`; `flushWait.mjs` đọc state qua `import()` trong khi Vite của `tauri dev` đang mở đã nhận nhiều lượt sửa tệp (bẫy tem HMR) ⇒ cần khởi động lại `tauri dev` rồi chạy lại spec. 🔵 2026-10-29: đã xác nhận bằng `curl` (app nạp `editorPanelState.ts?t=…`, spec `import()` bản không tem); với Vite sạch, 5-11 (2/2), 5-12, 5-13 xanh, và gỡ `@dblclick` ⇒ 5-11 đỏ ở "popup tra cuu khong hien ket qua that sau double-click". Sau lượt vá review cả ba vẫn xanh.

## Tasks & Acceptance

**Execution:**
- [x] `src/panels/lookupView.ts` (mới) + `LookupPanel.vue` -- chuyển năm computed ở trên sang module dùng chung; panel import lại, không đổi hành vi.
- [x] `src/selectionContract.ts` -- bề mặt vai `source` có thể tắt Auto-Lookup; `currentSelectionText()` vẫn trả chữ của nó cho lệnh phát tay.
- [x] `src/modes/ReadingLookupPopup.vue` (mới) -- đầu mục (từ, âm đọc, nút ghim), `LookupRecord` theo nhóm, tra từng chữ, dòng Concordance, câu lỗi, câu không thấy; định vị theo hình chữ nhật của vùng chọn, lật lên trên khi thiếu chỗ bên dưới, không tràn khỏi `section.mode`; có bóng từ token mới.
- [x] `src/modes/ReadingMode.vue` -- đăng ký các `.source-note` (Auto-Lookup tắt); `@dblclick` gọi lệnh mở popup; đóng theo bảng trên.
- [x] `src/commands/index.ts` + `src/main.ts` + `src/i18n/vi.json` -- `reading.lookup_open` (mở popup rồi `lookup.lookup_selection`), `reading.lookup_close`, `reading.lookup_concordance`; cập nhật Kiểm F/K của `check:commands` nếu cổng đòi.
- [x] `src/tokens/tokens.json` + `scripts/check-tokens.mjs` -- token bóng (sáng/tối) kèm mục `deviations`; Kiểm F cho `box-shadow`/`text-shadow` khi giá trị là đúng một `var(--shadow-…)`, vẫn chặn gradient và giá trị viết tay.
- [x] `src/AGENTS.md`, `DESIGN.md`, `EXPERIENCE.md` -- bỏ câu cấm lớp nổi và bóng đổ (DESIGN, EXPERIENCE ghi 🔵 2026-10-29); ghi tra cứu bằng popup ở Chế độ đọc.
- [x] `tests/frontend/readingLookupPopup.test.ts` + test của `selectionContract` -- mở bằng double-click trên nguyên văn; bôi đen không tra; không mở trên bản dịch; các đường đóng; nút gọi đúng lệnh; câu lỗi.
- [x] `e2e/specs/story-5-11-reading-mode.e2e.mjs` -- bật song ngữ, chọn một từ trong nguyên văn rồi phát `dblclick` (sự kiện chuột của driver có `isTrusted: false`, không tự chọn chữ), popup nằm cạnh vùng chọn và có kết quả thật; Esc đóng.

**Acceptance Criteria:**
- Given song ngữ đang bật, when double-click một từ chữ Hán có trong từ điển, then popup hiện sát từ đó với nghĩa của mọi từ điển đang bật, và lượt tra có trong Lịch sử của Panel Lookup.
- Given popup đang mở, when bấm ghim, then mục từ có trong danh sách Đã ghim của Panel Lookup.
- Given popup đang mở, when bấm dòng Concordance, then app ở Workspace với tab Concordance của từ đó.
- Given một `box-shadow` viết tay hoặc một gradient trong `src/`, when chạy `check:tokens`, then cổng đỏ; `box-shadow: var(--shadow-…)` thì xanh.
- Given Panel Lookup ở Workspace, when tra như trước (kể cả Auto-Lookup ở cột nguồn), then mọi ca vitest và e2e cũ vẫn xanh.

## Verification

**Commands:**
- `npx vitest run tests/frontend/readingLookupPopup.test.ts` và các tệp `tests/frontend/{lookup,selection}*.test.ts` -- expected: xanh, thoát 0.
- Bộ đầy đủ một lượt (thay đổi chạm `scripts/check-*`): `npm run build`, mọi cổng trong `.githooks/pre-push`, toàn bộ vitest -- expected: xanh.
- `npx wdio run e2e/wdio.conf.mjs --spec e2e/specs/story-5-11-reading-mode.e2e.mjs` (sau `cargo build --locked --features wdio`) -- expected: xanh.

**Manual checks (if no CLI):**
- Trong `tauri dev`: bật `B`, double-click một từ ở lề nguyên văn; popup mở cạnh từ, có bóng nhẹ; Esc đóng; kéo bôi đen không mở popup.

## Review Triage Log

Lượt 1 (quick): 13 phát hiện — medium 3, low 7, false 2, rejected-low 3 (tính cả 2 false).
- medium · patch — popup đổi chỗ, chớp tắt và đẩy focus về `section.mode` khi tra từng chữ sau khi đã cuộn: neo chỉ quy ra toạ độ nội dung lúc mount, mà popup bị tháo trong lúc `lookupPending`. Đã vá: neo lưu theo toạ độ nội dung ngay lúc double-click, popup sống suốt lượt mở, chỉ ẩn (`visibility`) cho tới câu trả lời đầu của một lượt double-click mới.
- medium · patch — đổi mức chữ, tinh chỉnh, thay `readingRun` hay đổi cỡ cửa sổ làm popup lơ lửng ở chỗ cũ. Đã vá: đóng khi `readingStyle`/`readingRun` đổi và khi `window` resize; host đổi cỡ mà cửa sổ không đổi thì chưa bắt (cần `ResizeObserver`).
- medium · patch — chuỗi điều kiện "hiện gì" chép nguyên từ `LookupPanel.vue`, trái Never của plan. Đã vá: `lookupBody` trong `lookupView.ts`, hai template cùng rẽ theo nó.
- low · patch — `openReadingLookup` nhận vùng chọn neo ở Element, khác vị từ của `surfaceFor`, nên popup có thể hiện truy vấn cũ. Đã vá bằng cùng vị từ nút văn bản; gỡ dòng kiểm đó không làm ca nào đỏ vì neo Element đã bị loại ở `closest` (dòng thừa, vô hại).
- low · patch — chú thích Kiểm F mang ngày, 🔵 và tiếng Việt. Đã viết lại hai dòng tiếng Anh.
- low · patch — vòng `shadows` trong `tokens/index.ts` chen giữa chú thích và mã panel-separator. Đã dời, có tiêu đề riêng.
- low · patch — Kiểm F đọc danh sách tên bóng từ chính `tokens.json`. Đã đóng băng `['popover']` trong script; khai thêm tên ⇒ cổng đỏ.
- low · patch — `DESIGN.md:376` vẫn nêu luật cũ, 🔵 nằm ở đoạn riêng. Đã sửa tại chỗ.
- low · patch — kiểm tay trong app thật chưa có mục nợ có chủ. Đã ghi `deferred-work.md` (Chủ: Ice).
- false — "e2e chưa từng xanh": nguyên nhân là tem HMR (đã chứng minh); với Vite sạch ca mới và ca cũ đều xanh.
- false — gọi thẳng `openLookupDrawer()` trái Never "không mở `LookupDrawer`": Never nói về chỗ hiện kết quả ở Chế độ đọc; ngăn kéo chỉ mở sau khi đã sang Workspace ở bố cục hẹp để thấy Concordance, và port gọi hàm state là cách các port khác vẫn làm.
- low · reject — fixture ca 5-11 cũ đổi sang chữ Hán thật và ca mới phụ thuộc ca trước: mọi mệnh đề của ca cũ vẫn đo như trước và xanh; tách ca cần thêm một lượt dựng Tác phẩm.
- low · reject — dòng lịch sử tra gắn `openChapter` của Editor chứ không phải Chương đang đọc: chỉ lệch ở phạm vi lịch sử "Chương"; sửa cần đổi `runLookup`/`recordLookup`, thứ plan khoá.
- low · reject — `Mod+Alt+L` trên vùng chọn nguyên văn chạy một lượt tra không hiện gì: trước đây phím này cũng không hiện gì ở Chế độ đọc; chặn cần thêm một nhánh theo chế độ.
