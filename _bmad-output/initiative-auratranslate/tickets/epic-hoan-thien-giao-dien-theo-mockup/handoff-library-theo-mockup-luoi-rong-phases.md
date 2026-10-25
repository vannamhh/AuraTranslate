# Handoff: Library theo mockup lưới rộng (plan `tickets/story-library-theo-mockup-luoi-rong-plan.md`)

## Pha 1 (webview) -- xong, chưa chạy test vỡ / e2e

Đã xanh: `check:tokens`, `check:commands`, `check:i18n`, `check:lint`, `vue-tsc`, `npm run build`. Chưa chạy: e2e, vitest toàn bộ, `check:scope`.

### Đã đổi
- Token: 12 `cover-{1..6}-{from,to}` mỗi theme trong `tokens.json` + bảng đóng băng; `colorsPerTheme` 36; `EMITTED_VAR_FLOOR` 147 -> 159 (nâng theo thông báo cổng, không hạ). `DESIGN.md` §Tông bìa sửa bằng 🔵 (24 + 12 = 36, bìa một màu).
- `libraryWorkCard.ts::workCoverTone` (FNV-1a theo `work_id`, 1..6). `libraryWorks.ts::aimLibraryWork(index)`.
- Lệnh `library.continue_translation` (`Shift+Enter`): `libraryChapters.ts::continueCurrentLibraryTranslation` (trả `false` ngoài Library, khi trang Tác phẩm đang mở, hoặc `openWorkById` hỏng; thành công thì `setMode('workspace')`). Nối ở `commands/index.ts` + `main.ts`.
- `LibraryMode.vue` viết lại phần template trên cùng: một hàng (ô tìm · Chính xác dấu|Không dấu · radio phạm vi chỉ khi ô tìm có chữ · bốn chip · số đếm `m / n` chỉ khi có lọc). Chip = component mới `src/modes/LibraryMenu.vue` (native `<details>`; Escape, bấm ngoài, `[data-menu-close]` đóng). Chip Trạng thái chứa 4 nút `data-lifecycle-filter` + `Bỏ lọc` (`data-lifecycle-action="clear_filter"`); chip Lĩnh vực/Ngôn ngữ/Sắp xếp là radio `@change` (`data-library-chip="genre|source_lang|sort"`, `data-library-chip-option`).
- Thẻ: `li.work-cell` tabindex lăn (0 cho thẻ chọn), `@mousedown`/`@focusin` -> `aimWork`, `@dblclick` -> `library.open_work`; `@keydown` của `ul` (`onGridKeydown`): Enter mở, mũi tên +-1 và +-số cột (đọc `grid-template-columns`), kẹp ở hai đầu bằng cách lặp `library.work_next/prev`, rồi focus thẻ mới. Bìa 112 px một màu `--color-cover-N-from`; nút `.work-go` (`data-library-continue-translation`, "Bắt đầu dịch →" khi `not_started`) + menu `⋯` (`data-library-work-more`: `data-library-open-work-folder`, `data-library-work-reviewer-import`) hiện ở thẻ chọn/hover/focus-within. Không còn: `.work-initial`, `[data-library-open-work]`, `[data-library-work-prev/next]`, `[data-lifecycle-action="list_works"]`, ba `<select>`, `[data-library-search-status]`, tiêu đề, dòng hướng dẫn. Tên `data-library-work-name`, tiến độ `data-library-work-progress`, huy hiệu `data-library-work-status` (ẩn khi `status = null`).
- `onActivated` focus thẻ đang chọn sau `enterFocus`; đóng trang Tác phẩm cũng trả focus về thẻ (`currentCard()`).
- `App.vue`: nút Nhập tài liệu + chord ở thanh tiêu đề (`v-if library`, `data-library-import-open` giữ nguyên hook). `StatusBar.vue`: nhánh `currentMode === 'library'` (số Tác phẩm, đang quét, lỗi); các mục Workspace nằm trong `v-else`.
- `check-commands.mjs` `HANDLER_TABLE`: thêm `LibraryMenu::onKeydown`, `LibraryMode::aimWork`, `LibraryMode::onGridKeydown`.
- i18n: thêm `command.library.continue_translation`, `mode.library.{continue_translation,start_translation,work_more_menu,filter_status_label,filter_status_many,filter_count,works_loading,works_progress_short,works_chapter_count}`, `status.library_{works,scanning}`; đổi chữ `search_placeholder`; xoá `list_works, work_prev, work_next, work_position, works_heading, open_work_button, works_not_loaded, works_result, works_progress_unknown`. Khoá `works_progress` còn tồn tại nhưng hết dùng ở Library.

### Vitest đang đỏ (7, đều do gỡ phần tử cũ -> sửa ở pha 2)
`libraryTopBar.test.ts` (Nhập tài liệu giờ ở App.vue; thẻ không genre/ngày), `libraryWorks.test.ts` (ba nhánh tiến độ, `coverInitial` "?" x2, selects, nút reviewer chỉ ở thẻ chọn). Chưa chạy: `librarySearchScope.test.ts` xanh; e2e `story-5-4..5-10, 12-2, 12-4, 12-5, 12-17` chưa đụng (cần mở chip trước khi bấm lọc; `12-2` kiểm 4/3/2 cột nay là auto-fill 240 px).

### Rủi ro / việc cho pha sau
- `Shift+Enter` là chord toàn cục chỉ bị chặn trong vùng gõ; ngoài Library port trả `false` nhưng `preventDefault` vẫn chạy (không có chỗ nào khác trong sổ lệnh dùng `Shift+Enter` trần).
- Chưa mở app thật xem ảnh; vòng ngoài (outline 1px + border 1px) cho viền 2 px thẻ chọn, không dùng box-shadow (Kiểm F). Cần đối chiếu mắt ở pha 3.
- Tab trong lưới: chỉ thẻ chọn tabbable; các nút `.work-go` của thẻ khác `display: none` đến khi hover.
- Kiểm tra `e2e` `Shift+Enter` có giữ `shiftKey` không (plan Code Map); mất thì dùng `pressChordOnFocusedElement`.
- `library-mode-switch-focus.e2e.mjs`: tiêu điểm nay về thẻ chứ không về `<section>` khi có thẻ.

## Pha 2 (test vỡ) -- xong phần vitest; e2e xem dòng kết quả cuối mục này

### Vitest (xanh: `tests/frontend/library*`, `statusBar*`, `importPreview*`, `commands*` = 652 ca; `check:lint`, `vue-tsc` xanh)
- `libraryWorks.test.ts`: nhánh `null` nay là `"N chương"` và không có chữ "Chưa biết"; bỏ hai ca chữ cái đầu, thay bằng ca tông bìa 1..6; ca selects thành ca radio của chip (không còn `<select>`); thêm ca bấm thẻ (`mousedown` ngắm, `dblclick` -> `openCurrentLibraryWork`) và ca mũi tên/Enter/Shift+Enter; nút reviewer nay có ở MỌI thẻ (trong menu `⋯`). Hai ca mới gọi `vi.resetModules()` vì `installCommands` đăng ký một lần mỗi module.
- `libraryTopBar.test.ts`: nút Nhập tài liệu không còn trong Library; dòng phụ rỗng nay là `<span>` rỗng (không biến mất).
- `libraryImportOverlay.test.ts`: giả nút `[data-library-import-open]` trong DOM thay vì mount Library.
- `statusBar.test.ts`, `statusBarWorkspace.test.ts`: `setMode('workspace')` trong hàm dựng (mặc định `library` nay ẩn mục Workspace). Chưa có ca vitest riêng cho nhánh Library của thanh trạng thái (e2e/pha 3 phủ).

### e2e đã sửa (đường mới)
- Mới `e2e/support/libraryGrid.mjs`: `reloadWorks()` (bật rồi tắt chip Trạng thái -> `library_list_works`; đường tải lại duy nhất còn lại sau khi gỡ nút Tải danh sách), `aimWork(name)` (realClick tên thẻ rồi `focus()` JS lên thẻ: mousedown trên `<span>` KHÔNG đưa tiêu điểm vào `li` trong WKWebView), `openAimedWork()` (Enter), `openChip(chip)`, `workCellNames()`.
- Đã sửa: `story-5-4, 5-5, 5-6, 5-7, 5-8, 12-2, 12-4, 12-5, 12-17`. 5-6 chạy xanh một mình. Chip phải ĐÓNG sau khi chọn (menu mở phủ lên hàng thẻ đầu, bấm tên thẻ rơi vào menu).
- 12-2: ca 4/3/2 cột thành ca auto-fill (số cột không giảm khi cửa sổ rộng ra, mọi thẻ >= 239 px); bỏ kiểm tỉ lệ bìa.

### Cho pha 3 (spec 12-24 + ảnh)
- Dùng `support/libraryGrid.mjs`. `Shift+Enter` qua `browser.keys(['Shift','Enter'])`: CHƯA kiểm có giữ `shiftKey` không (`Meta+2` thì chạy trong 5-5); mất thì dùng `pressChordOnFocusedElement` (`editor-confirm-segment.e2e.mjs:70`).
- `data-reviewer-import-open` nay có ở mọi thẻ (trong `<details>` đóng) cạnh nút ở `TitlebarToolsMenu`/`LibraryWorkPage`; `ReviewerImportOverlay` trả tiêu điểm về phần tử đang active lúc mở (thẻ) nên chưa thấy vấn đề, nhưng `document.querySelector('[data-reviewer-import-open]')` đứng thứ tự DOM.
- Cột tới 4 ở 1440 px (293 px/thẻ); cửa sổ e2e mặc định cho 4 cột.

### e2e đã chạy (ngoài sandbox)
- Xanh: `5-4, 5-5, 5-6, 5-7, 5-8, 5-9, 5-10, 12-2, 12-4, 12-6, 12-17, titlebar-tools`. 12-2 đỏ ở bước trả tiêu điểm về nút Nhập tài liệu (nút nay ở thanh tiêu đề, ngoài `<section class="mode">`, nên `focusReturnTargetOnOpen` trả về section): sửa trong `src/LibraryImportOverlay.vue` (đích trả là section/body/null thì lấy `[data-library-import-open]`).
- CHƯA xác nhận: `12-5` (before-all gõ chữ ở Workspace hết 30 s; `editor-typing-flush` đỏ y hệt trên worktree HEAD sạch ⇒ môi trường, không phải thay đổi này). Phần Library của 12-5 (mở chip Trạng thái trước khi bấm lọc) chưa chạy được.
- `wdio ... --spec a --spec b` KHÔNG lọc: chạy cả 50 spec (20 đỏ, đa số Workspace gõ chữ). Chạy từng spec một.

## Pha 3 (spec 12-24 + ảnh) -- xong, 5/5 xanh (ngoài sandbox)

- Mới `e2e/specs/story-12-24-library-wide-grid.e2e.mjs`: dựng 8 Tác phẩm bằng IPC (tách Chương bằng `split_chapter_at_segment`, đổi trạng thái bằng `set_chapter_status`, nên tiến độ thật); một thẻ ghi đè `meta.json` thành v1 (tiến độ chưa biết) rồi `library_rescan`, một thẻ `genre = ''`. Ca: bốn loại thẻ không chữ "chưa biết"; `⌘2` rồi `⌘1` -> mũi tên tới thẻ đích -> `Shift+Enter` -> đúng Chương và `caret_segment_id` đã lưu -> `⌘1` tiêu điểm về đúng thẻ; kẹp hai đầu; 1150/2560 px (cột tăng, thẻ 239-306 px); ảnh sáng/tối.
- Lỗi sản phẩm tìm ra và sửa trong `LibraryMode.vue`: mảng `ref` trong `v-for` theo thứ tự mount, không theo thứ tự danh sách, nên sau khi tải lại đổi thứ tự thì mũi tên focus nhầm thẻ (con trỏ nhảy 0 -> 4 -> 3). Nay `cardAt(index)` truy vấn DOM theo `[data-library-work-cell]`; bỏ `workCellRefs`.
- `browser.keys(['Shift','Enter'])` mất `shiftKey` (chạy thành Enter, mở trang Tác phẩm): spec gửi `KeyboardEvent` tổng hợp lên phần tử đang focus.
- Cửa sổ: `screen.availHeight` = 935 nên chiều cao trong không đạt 935; spec chỉ ép chiều rộng. Ảnh 2980x1870 (1490x935 dpr 2) ra tại thư mục tạm `auratranslate-e2e-12-24-*` (đường dẫn in ra console); bản sao: scratchpad `library-light.png`, `library-dark.png`.
- Đối chiếu mắt: bốn loại thẻ, viền chọn 2 px, nút Tiếp tục dịch + `⋯` trên thẻ chọn, chip một hàng, số đếm chỉ khi lọc, Nhập tài liệu ở thanh tiêu đề, thanh trạng thái chỉ còn `Library · 8 Tác phẩm` đều khớp. Còn lại cho Ice: khoảng trống ~75 px giữa hàng chip và hàng thẻ đầu (chưa so với mockup từng điểm ảnh), nhãn thẻ không có tên lĩnh vực ghi `vừa sửa`.
