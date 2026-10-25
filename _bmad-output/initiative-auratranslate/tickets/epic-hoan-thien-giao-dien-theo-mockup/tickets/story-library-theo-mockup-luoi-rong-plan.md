---
title: 'Library theo mockup lưới rộng'
type: 'feature'
ticket: '24'
created: '2026-10-25'
status: 'done'
baseline_revision: 'bacde69da3564edb13ce0d5e4ecfce8a34e83814'
route: 'full'
route_source: 'auto'
risk: 'medium'
review: 'quick'
review_source: 'pinned'
lenses_ran: [quick]
review_loop_iteration: 0
context:
  - '{project-root}/_bmad-output/initiative-auratranslate/ux-auratranslate/library-nghiem-thu-thi-giac.md'
  - '{project-root}/_bmad-output/initiative-auratranslate/ux-auratranslate/mockups/library-luoi-rong.html'
---

<frozen-after-approval reason="human-owned intent — do not modify unless human renegotiates">

## Intent

**Problem:** Library lệch `mockups/library-luoi-rong.html` ở mười bốn dòng L1–L14 của `library-nghiem-thu-thi-giac.md`: bìa vuông 330 px, bốn tầng điều khiển, bấm thẻ không chọn được, không có đường vào thẳng phần dịch, lưới cố định 4/3/2 cột.

**Approach:** Sửa webview theo đúng bảng L1–L14 (cột "Cách sửa" là yêu cầu, không chép lại ở đây), thêm 12 token bìa mỗi theme, một lệnh `library.continue_translation`, và một spec e2e vừa nghiệm thu vừa dựng Library mẫu để chụp ảnh. Không đổi Rust.

- Decision (Ice, 2026-10-25): hợp âm Tiếp tục dịch là `Shift+↵`, không phải `Mod+↵` của phiếu — `Mod+↵` là `editor.confirm_segment` toàn cục và mọi biến thể `Mod`/`Alt`+`↵` đã gán. Lệnh gán lại được ở Cài đặt (FR22); port chỉ chạy khi `currentMode === 'library'`, nơi khác trả `false`.
- Decision (Ice, 2026-10-25): bìa một màu `cover-N-from`, không dải chuyển — giữ `DESIGN.md:436` và Kiểm F của `check-tokens.mjs`, không thêm miễn trừ. Vẫn thêm đủ 12 token mỗi theme theo §Tông bìa; `cover-N-to` chưa có chỗ dùng.
- Decision (Ice, 2026-10-25): giữ plan một khối (≈3.500 token), không tách L11/L12.

## Boundaries & Constraints

**Always:** Mọi thao tác chuột/phím đi qua `dispatch` một lệnh đã đăng ký (AD-34 ①); ngắm thẻ bằng `@mousedown`/`@focusin` theo khuôn `SegmentHistoryOverlay.vue:284-294`. Màu chỉ từ token; 12 token mới vào cả `tokens.json` lẫn bảng đóng băng của `check-tokens.mjs` cùng lượt. L3 giấu chữ "chưa biết" chỉ vì bug 23 đã ghi nguyên nhân (plan bug 23, Decision A); `NULL` vẫn là chưa biết, không thành `0`. `Tiếp tục dịch` mở hỏng (khoá, flush lỗi, IPC) thì ở lại Library và hiện lỗi, như `library.open_chapter`.

**Never:** Không lệnh Rust mới, không đổi `meta.json`/chỉ mục. Không viết lại thanh trạng thái Workspace, không sửa màn ngoài Library (phiếu sau). Không gán hợp âm trùng (`createKeymap` ném khi trùng). Không hạ `COMMAND_FLOOR` hay thêm miễn trừ để làm xanh cổng. Không `linear-gradient`.

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected Output / Behavior | Error Handling |
|---|---|---|---|
| Chọn thẻ | bấm một lần thẻ 3 | thẻ 3 viền `primary` 2 px, nút Tiếp tục dịch hiện trên bìa | — |
| Mở trang Tác phẩm | bấm đúp hoặc `↵` trên thẻ chọn | `library.open_work` → trang Tác phẩm | lỗi mở ⇒ ở lại lưới, thông báo hiện |
| Tiếp tục dịch | `Shift+↵` hoặc nút, Tác phẩm có Chương dở | Workspace, đúng Chương `last_chapter_id`, đúng câu `chapter_position` | khoá/flush/IPC ⇒ ở lại Library, lỗi hiện |
| Bắt đầu dịch | Tác phẩm `not_started` | nút ghi "Bắt đầu dịch →", cùng lệnh | như trên |
| Mũi tên | lưới có tiêu điểm, 5 cột | `→`/`←` ±1, `↓`/`↑` ±5, kẹp ở hai đầu | — |
| Quay lại Library | `⌘1` từ Workspace | tiêu điểm DOM về đúng thẻ đã chọn, không về `body` | — |
| Tiến độ chưa biết | `chapter_done_count = null` | không thanh, dòng ghi `N chương` | — |
| Trạng thái hỏng | `status` ngoài danh mục | vẫn hiện chữ `works_status_invalid` | — |

</frozen-after-approval>

## Code Map

- `src/modes/LibraryMode.vue` -- thẻ `:553-669` (bìa `:563-570`, `.work-initial`, `coverInitial` `:223`, trạng thái `:584-598`, tiến độ `:606-631`, ba nút trong thẻ `:638-667`), `.grid-nav` `:531-551`, hàng điều khiển `.top-bar` `:289-483` (nhập `:357`, bốn nút lọc + Bỏ lọc + Tải danh sách `:375-434`, ba `<select>` `:444-482`), dòng hướng dẫn `:487`, tiêu đề + số đếm `:489-505`, `onActivated` `:146` (`enterFocus('mode.library')` về `<section>`), CSS `.works-grid` `:1232` + hai `@container` `:1330-1336`, `.work-cover` `:1267`.
- `src/modes/libraryWorks.ts` -- `workCursor`/`libraryWorkCursor` `:91`, `nextWork` `:214`, `prevWork` `:225`, `selectLibraryWorkById` `:219`, kẹp `:130`; nguồn số cho thanh trạng thái: `libraryWorksTotal`/`Matched` `:83`, `libraryWorksHaveLoaded` `:92`, `libraryWorksError` `:94`.
- `src/modes/libraryWorkCard.ts:34` -- `workCardMeta()` (nhãn ngôn ngữ, dòng phụ); thêm chọn tông theo `work_id` ở đây.
- `src/modes/libraryChapters.ts` -- `openWorkById` `:305` (flush, mở, `enterFocus('panel.grid')`, không đổi chế độ, trả `false` khi hỏng), `openCurrentChapter` `:462` (khuôn: `if (moved) setMode('workspace')`).
- `src/commands/index.ts` -- `registerAll` `:1346`, cổng `CommandDeps` `:171`, `library.work_prev/next` `:2143-2160` (chưa có phím), `library.import.open` `Mod+N` `:1577`; `src/main.ts:572` nối cổng; `src/i18n/vi.json` nhãn `command.library.continue_translation`.
- `scripts/check-commands.mjs` -- Kiểm K `HANDLER_TABLE` `:2414` (khuôn `LibraryWorkPage.vue::onListKeydown` `:2540`) cho `@keydown` của lưới; `COMMAND_FLOOR` `:237`.
- `src/tokens/tokens.json` + `scripts/check-tokens.mjs` -- bảng đóng băng `EXPECTED_COLORS_LIGHT/DARK` `:134-199`, `EXPECTED_COUNTS.colorsPerTheme: 24` `:290` (không phải 17: đã có `diff-*`, `word-*`) → 36; thông điệp "32 giá trị" `:752`. Không thêm token bìa vào `EXPECTED_ROLES`.
- `src/App.vue:229-267` -- nút Nhập tài liệu `v-if="currentMode === 'library'"` cạnh `<TitlebarToolsMenu />` `:266`, theo khuôn `:237`.
- `src/StatusBar.vue:526-611` -- ẩn mục Workspace khi `currentMode === 'library'`; số Tác phẩm, quét (`libraryRescan.ts` `:92-113`), lỗi.
- `src/TitlebarToolsMenu.vue` -- khuôn menu bàn phím cho chip `Trạng thái ▾` và nút `⋯`.
- Test vỡ vì gỡ phần tử (`data-lifecycle-action="list_works"`, `[data-library-work-prev/next]`, `.work-initial`, `[data-library-open-work]`, selects, `[data-library-search-status]`): e2e `story-5-4`, `5-5`, `5-6`, `5-7`, `5-8`, `5-9`, `5-10`, `12-2` (cột 4/3/2 + tỉ lệ bìa ≈1), `12-4`, `12-5`, `12-17`; vitest `tests/frontend/libraryWorks.test.ts` (`:717, 776-806, 916-950`), `librarySearch.test.ts`, `librarySearchScope.test.ts`, `libraryTopBar.test.ts`. `library-mode-switch-focus.e2e.mjs` đi qua `import()`, đọc lại nếu tiêu điểm trả về thẻ.
- e2e: `browser.keys(['Meta','Enter'])` mất `metaKey` (`editor-confirm-segment.e2e.mjs:20-35`); kiểm `Shift+Enter` có giữ `shiftKey` không, mất thì dùng `pressChordOnFocusedElement` (`:70`); đổi khổ theo `resizeToLogicalWidth` (`story-12-2…:55`); tạo Tác phẩm bằng `invoke('create_work_from_text')`, `set_chapter_status`; Tác phẩm mới có `chapter_done_count = Some(0)`, nên thẻ "chưa biết tiến độ" cần một `meta.json` bản 1 viết vào gốc Library của spec rồi quét lại.

## Tasks & Acceptance

**Execution:**
- [x] `src/tokens/tokens.json`, `scripts/check-tokens.mjs` -- 12 token `cover-{1..6}-{from,to}` mỗi theme theo `DESIGN.md` §Tông bìa; bảng đóng băng + đếm 36; bìa tô một màu `--color-cover-N-from` -- điều buộc (2) của ticket.
- [x] `src/modes/libraryWorkCard.ts` -- tông `1..6` ổn định theo `work_id` -- L7.
- [x] `src/modes/LibraryMode.vue` -- L1–L6, L8–L10, L13, L14: bìa 112 px, tên serif, giấu chữ chưa biết, ngắm thẻ, `@dblclick`/`↵` mở, mũi tên (lên/xuống theo số cột đọc từ `grid-template-columns` đã tính), một hàng chip, phạm vi tìm chỉ khi có chữ, bỏ nút Tải danh sách/tiêu đề/con trỏ, số đếm chỉ khi lọc, nút Tiếp tục dịch + `⋯`, lưới `auto-fill`; `onActivated` trả tiêu điểm về thẻ đang chọn.
- [x] `src/commands/index.ts`, `src/main.ts`, `src/i18n/vi.json`, `scripts/check-commands.mjs` -- `library.continue_translation` (port: `openWorkById` rồi `setMode('workspace')` khi `true`), `keys: ['Shift+Enter']`, `HANDLER_TABLE` cho `@keydown` lưới; xoá khoá i18n hết dùng.
- [x] `src/App.vue`, `src/StatusBar.vue` -- L11, L12.
- [x] `_bmad-output/initiative-auratranslate/ux-auratranslate/DESIGN.md` §Tông bìa -- sửa tại chỗ bằng 🔵: "17 + 12 = 29" thành số thật của `tokens.json` (24 + 12 = 36), và câu "dải chuyển `160deg`" thành bìa một màu `cover-N-from` theo Decision của Ice.
- [x] vitest + 11 spec e2e trên -- đi đường mới (bấm thẻ, phím, lệnh), không đường đã gỡ.
- [x] `e2e/specs/story-12-24-library-wide-grid.e2e.mjs` -- ca bàn phím của verify; ca 1150/2560 px; ca dựng Library mẫu (tên, lĩnh vực, tiến độ thật, đủ bốn loại thẻ) và lưu ảnh 1490×935 sáng/tối vào thư mục tạm của spec, in đường dẫn.

**Acceptance Criteria:**
- Given Library có ≥ 3 Tác phẩm, when `⌘1` → mũi tên tới thẻ thứ ba → `Shift+↵`, then Workspace mở đúng câu đang dở; `⌘1` về Library thì `document.activeElement` là thẻ thứ ba.
- Given cửa sổ 1150 px rồi 2560 px, when lưới vẽ lại, then số cột tăng và mọi thẻ rộng 240–305 px.
- Given 12 token mới, when `npm run check:tokens` và `npm run check:commands`, then xanh mà không hạ sàn nào.
- Given ảnh app thật 1490×935 dpr=2 sáng và tối trên Library mẫu, when đặt cạnh mockup cùng khổ, then mỗi dòng L1–L14 ghi khớp hoặc lý do; Ice chạy app một lượt (người, trước `done`).

## Implementation Notes

- Không đổi Rust. `library.continue_translation` = `openWorkById` rồi `setMode('workspace')`; port trả `false` ngoài Library hoặc khi trang Tác phẩm đang mở, nhưng `Shift+↵` toàn cục vẫn `preventDefault` ở mọi chế độ (chỉ nhường vùng gõ).
- Chip lọc là `<details>` gốc (`src/modes/LibraryMenu.vue`); lĩnh vực/ngôn ngữ/sắp xếp là radio `@change` như ba `<select>` cũ, ngoài Kiểm A.
- Con trỏ lưới giữ theo `work_id` khi `loadWorks` đổi thứ tự (sắp `updated_desc` đẩy Tác phẩm vừa sửa lên đầu); trước đây giữ theo chỉ số nên tiêu điểm DOM và thẻ chọn tách nhau.
- WebKit không đưa tiêu điểm vào `li[tabindex]` khi bấm trúng `<span>` con: `aimWork` tự `focus()` thẻ trên `mousedown` (trừ khi bấm nút/`summary`). Helper e2e `aimWork` trước đó che lỗi này bằng `focus()` JS; đã gỡ.
- Mảng `ref` trong `v-for` theo thứ tự mount, không theo thứ tự danh sách: thẻ được tra bằng `querySelectorAll('[data-library-work-cell]')[index]`.
- `browser.keys(['Shift','Enter'])` mất `shiftKey` trong WKWebView: spec 12-24 gửi `KeyboardEvent` tổng hợp lên thẻ đang focus.
- `EMITTED_VAR_FLOOR` 147 → 159 theo `ceil(0.85 × live)` (nâng, không hạ).
- Lỗi mở Tác phẩm hiện cả ở thanh trạng thái Library, không chỉ ở `<p>` cuối trang.
- `12-5` chưa chạy được phần Library: before-all gõ chữ ở Workspace hết 30 s, `editor-typing-flush` đỏ y hệt trên worktree `HEAD` sạch ⇒ môi trường.

## Plan Change Log

## Review Triage Log

- Lượt 1 (quick): high 0 · medium 4 · low 7 · false 1 · maybe-false 0 (10 của lens, 2 của người điều phối); 9 patch, 2 bác, 0 defer.
- medium, patch — con trỏ theo chỉ số sau `loadWorks` đổi thứ tự ⇒ tiêu điểm ở thẻ khác thẻ chọn, `Shift+↵` mở nhầm: `loadWorks` giữ theo `work_id`; ca vitest đỏ khi gỡ dòng giữ.
- medium, patch — lỗi Tiếp tục dịch chỉ ở `<p>` cuối trang, khuất dưới lưới: thanh trạng thái Library hiện `libraryOpenWorkError`/notice; ca vitest đỏ khi gỡ.
- medium, patch — hai guard của port và đường `dispatch` chưa có ca: thêm 3 ca.
- low, patch — nhánh Library của `StatusBar.vue`, `LibraryMenu.vue` chưa có ca: thêm `statusBarLibrary.test.ts` (4), `libraryMenu.test.ts` (3).
- low, patch — `works_progress`, `sort_label` hết dùng: xoá khỏi `vi.json`.
- false — 2560 px vượt `availWidth`: macOS chỉ kẹp chiều cao, ca 1150/2560 xanh trên máy Ice (`e2e1224*.log`, 5 passing).
- low, bác — L1–L14 và ô task trống: sửa plan, không sửa mã; điền ở bước trình bày.
- low, patch — comment `AC7` tiếng Việt trong dòng viết lại và đầu spec `(ticket 24)`: xoá/rút còn hai dòng tiếng Anh.
- low, patch — `.work-status--unknown` chết: xoá.
- low, bác — mỗi `LibraryMenu` gắn 2 listener `document`: không sai, sửa cần thêm cơ chế.
- medium, patch (người điều phối) — bấm thẻ không đưa tiêu điểm vào thẻ trong WebKit nên mũi tên chết sau khi bấm; helper e2e che bằng `focus()` JS: `aimWork` focus thẻ, gỡ phần che.
- low, patch (người điều phối) — khoảng trống ~75 px trên lưới do hai `<p role=status>` rỗng giữ `min-height` + `margin`: `:empty` về 0.

## Design Notes

Ba pha, mỗi pha một agent mới, bàn giao qua `handoff-library-theo-mockup-luoi-rong-phases.md` cạnh epic: webview (token, thẻ, lệnh, thanh tiêu đề/trạng thái) · test vỡ · spec mới + ảnh. Không có pha Rust.

## Verification

**Commands:**
- `npm run check:tokens && npm run check:commands && npm run check:i18n` -- expected: xanh.
- `npm run test:story 12-24` -- expected: xanh (vitest Library).
- `npm run test:e2e -- --spec e2e/specs/story-12-24-library-wide-grid.e2e.mjs` rồi từng spec đã sửa -- expected: xanh, chạy ngoài sandbox.

**Manual checks (if no CLI):**
- Ảnh sáng/tối cạnh `library-luoi-rong.html`, bảng L1–L14 khớp/lý do trong Implementation Notes; Ice chạy app một lượt.
