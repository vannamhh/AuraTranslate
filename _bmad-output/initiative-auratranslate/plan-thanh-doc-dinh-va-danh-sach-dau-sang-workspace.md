---
title: 'Thanh điều chỉnh Chế độ đọc dính ở trên cùng; danh sách chỗ đánh dấu ở lại khi sang Workspace'
type: 'feature'
ticket: ''
created: '2026-11-04'
status: 'done'
baseline_revision: 'e4e0b426c9ba8e32079e20dc981cc19a9094a70d'
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

**Problem:** Ở Chế độ đọc, thanh điều chỉnh (Song ngữ · Cỡ chữ · 1 2 3 · A · căn · Sáng · Mục lục · Chỗ đã đánh dấu) trôi đi khi cuộn xuống, nên muốn đổi gì thì phải cuộn ngược lên đầu. Bấm "Sửa →" hoặc "Sửa lần lượt cả N chỗ" trong danh sách "Chỗ đã đánh dấu khi đọc" thì chuyển sang Workspace và danh sách biến mất, nên sửa xong một chỗ không thấy các chỗ còn lại để chọn tiếp.

**Approach:** Ice chọn giữ hai việc trong một plan (2026-11-04).

Việc 1: chỉ thanh điều chỉnh `.toolbar` dùng `position: sticky` bên trong khung cuộn `.mode`; dòng tiến độ Chương vẫn cuộn đi bình thường (Ice chọn 3A).

Việc 2: danh sách đi theo sang Workspace thành panel dock mới `panel.reading_marks`, đứng cạnh Lưới, Tra cứu và Dịch AI (Ice chọn 1A). Panel tắt khi bấm "Đóng", khi mở Tác phẩm khác, hoặc khi quay về Chế độ đọc, vì ở đó đã có bảng modal riêng (Ice chọn 2B). Các nút giữ nguyên ý nghĩa, nhưng "Sửa →" trong panel nhảy tới câu ngay trong Workspace, không đổi mode.

## Boundaries & Constraints

**Always:** Mọi nút của danh sách trong Workspace là command đã đăng ký, có điểm chuột (AD-34, AD-62). Vùng danh sách khai điểm vào tiêu điểm, nên đóng nó không làm tiêu điểm rơi về `body`. Danh sách trong Workspace đọc cùng nguồn `readingState.ts` với Chế độ đọc, không có bản sao thứ hai. Không có câu nào thì nói rõ, không để trống im lặng. Chuỗi mới nằm trong `vi.json`. Các `z-index` thêm vào đều có `aura-allow-z-index` kèm lý do.

**Never:** "Sửa →" không tự bỏ dấu. Không đổi lệnh đánh dấu/bỏ dấu ở Rust. Danh sách trong Workspace không phải `aria-modal`, không bẫy Tab, không chặn gõ vào lưới. Không đổi cách cuộn tới câu (`scrollIntoView({block:'center'})`). Panel không được ghi vào bố cục đã lưu: danh sách chỉ sống trong bộ nhớ, nên nếu panel được khôi phục sau khi khởi động lại thì nó sẽ rỗng mà không báo gì. Không thêm công tắc cho panel này vào `TitlebarToolsMenu`.

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected Output / Behavior | Error Handling |
|----------|--------------|---------------------------|----------------|
| Cuộn | Chế độ đọc, cuộn xuống giữa Chương | Thanh điều chỉnh đứng ở mép trên khung cuộn, chữ và nút đánh dấu của câu không vẽ đè lên thanh | — |
| Sửa một chỗ | Danh sách mở, 2 chỗ, bấm "Sửa →" ở hàng 1 | Sang Workspace, câu 1 có con trỏ, danh sách vẫn hiện đủ 2 hàng, hàng 1 được tô là hàng hiện tại | — |
| Sửa chỗ kế | Workspace, danh sách hiện, bấm "Sửa →" ở hàng 2 (Chương khác) | Mở Chương đó, con trỏ ở câu 2, vẫn ở Workspace, danh sách vẫn hiện | Lỗi mở Chương đi qua `tError` như đường hiện có |
| Bỏ dấu | Workspace, bấm "Bỏ dấu" ở một hàng | Hàng biến khỏi danh sách; hết hàng thì hiện `marks_empty` | Lỗi ghi hiện ở `role=status` của vùng |
| Câu đã đổi | Hàng là câu đã về hưu | Giữ nhãn "Câu này đã đổi" như ở Chế độ đọc | — |
| Về Chế độ đọc | Panel đang hiện, chuyển sang Chế độ đọc | Panel bị gỡ khỏi Workspace; sang lại Workspace thì không còn panel | — |
| Đổi Tác phẩm | Panel đang hiện, mở Tác phẩm khác | Panel bị gỡ | — |
| Cửa sổ hẹp | Tầng `narrow`/`unsupported` (🔵 2026-11-04, Ice chọn: tầng `short` giữ panel, vì cửa sổ mặc định 1280×860 đã là `short`) | Panel bị gỡ trước Dịch AI và Tra cứu; Lưới không bao giờ bị gỡ | — |

</frozen-after-approval>

## Code Map

- `src/modes/ReadingMode.vue:605` `.toolbar` · CSS `:1143-1166` -- nằm trong `.mode` (`:1128`, `overflow:auto`). `.mode` có `padding-top` và thanh bù bằng `margin-top` âm, nên `top:0` khớp mép khung cuộn. `.column` (`position:relative`) và `.mark-actions` (`z-index:1`, `:1313`) đứng sau thanh trong DOM nên sẽ được vẽ đè lên thanh dính; cần `z-index: 2` có miễn trừ. Tiền lệ thanh dính không cần z-index: `LibraryWorkPage.vue:1361`.
- `src/modes/ReadingMode.vue:976-1030` -- bảng modal hiện tại (`.marks-overlay`, các hàng, `onMarksKeydown` cho ↑ ↓ ↵ ⌫). Tách thân danh sách ra component dùng chung; Chế độ đọc giữ vỏ modal và bẫy Tab, panel dùng thân không có vỏ.
- `src/modes/readingState.ts` -- refs ở `:280-299`. `openCurrentReadingMark` (`:444-456`) và `openReadingMark` của walk (`:388-399`) gọi `resetReading({preserveAnchor:true})`, hàm này luôn gọi `resetReadingMarks()` (`:458-469`) và xoá `marks`: đây là chỗ danh sách mất. Cần một ref "panel đánh dấu đang hiện trong Workspace", để đường mở hàng giữ `marks`, `marksCursor` và cờ này qua phép reset, và một nhánh mở hàng không gọi `setMode` khi đang ở Workspace. Có thể dùng `marksSequence` để chặn kết quả nạp đã cũ.
- `src/layout/workspaceLayout.ts:45-73` -- tập panel đóng (`PanelId`, `PANEL_IDS`, `PANEL_TITLE_KEYS`, `PANEL_COMPONENTS`). `SACRIFICE_ORDER` (`:196`): đặt `panel.reading_marks` lên đầu. Panel id không nằm trên đĩa (comment `:41`), nhưng `api.toJSON()` thì có: lọc panel này ra trước mỗi lần persist (`WorkspaceDock.vue:400` và `:845` là hai chỗ đang tuần tự hoá).
- `src/layout/WorkspaceDock.vue:114-118` (map component) · `showPanel`/`hidePanel`/`togglePanel` (`:514`) · `restoreFocusIfLost` · tiền lệ `addPanel(api, id, { referencePanel, direction })` (`:716`). Vị trí mặc định: bên phải `panel.grid`. 🔵 2026-11-04 (Ice, sau khi dùng thử): panel mở thành tab đang chọn trong nhóm của Tra cứu (không có Tra cứu thì nhóm của Đề xuất AI, không có cả hai thì bên phải lưới). Panel bọc trong `src/panels/PanelFrame.vue` để khai điểm vào tiêu điểm (AD-34). `src/panels/README.md` mô tả cách thêm panel.
- `src/commands/index.ts:2841-2846` · `src/main.ts:430,774` -- nơi đăng ký command và dep của reading marks. Command mới `reading.marks_panel_close` đặt cùng chỗ này. `src/commands/focus.ts` -- vòng xoay `focus.next_panel`.
- `src/App.vue:334-338` -- ba mode trong `<KeepAlive>`; gỡ panel khi rời Workspace sang Chế độ đọc (2B) theo sự kiện đổi mode, không theo `onDeactivated` của Workspace, vì sang Library thì không gỡ.
- `src/StatusBar.vue:714` -- nút "kế tiếp x/y" của walk; giữ nguyên, nhưng hàng hiện tại trong panel đi theo `marksWalk`.
- Test: `tests/frontend/readingMarks.test.ts:164-215` (mở hàng ⇒ workspace), `tests/frontend/readingToolbar.test.ts`, `tests/frontend/statusBarReadingWalk.test.ts`, `tests/frontend/dockTree.test.ts` và `tests/frontend/workspaceDockTier.test.ts` (tập panel, tầng), `e2e/specs/story-5-13-reading-marks.e2e.mjs:289-298`.

## Tasks & Acceptance

**Execution:**
- [ ] `src/modes/ReadingMode.vue` -- `.toolbar` thành `position: sticky; top: 0`, kèm `z-index` có miễn trừ; tách thân danh sách ra `src/modes/ReadingMarksList.vue` -- việc 1, và phần dùng chung cho việc 2.
- [ ] `src/modes/readingState.ts` -- ref hiện-trong-Workspace; giữ danh sách qua phép reset khi mở hàng; mở hàng từ Workspace không đổi mode; gỡ khi đổi Tác phẩm hoặc sang Chế độ đọc -- nguồn duy nhất cho cả hai nơi.
- [ ] `src/layout/workspaceLayout.ts`, `src/layout/WorkspaceDock.vue`, `src/panels/ReadingMarksPanel.vue` (mới), `src/commands/focus.ts` -- panel dock không persist, gỡ đầu tiên ở tầng hẹp, hiện và ẩn theo ref ở trên -- 1A.
- [ ] `src/commands/index.ts`, `src/main.ts`, `src/i18n/vi.json` -- command đóng panel, tiêu đề panel -- AD-34, NFR16.
- [ ] `tests/frontend/readingMarks.test.ts`, `tests/frontend/readingToolbar.test.ts`, `tests/frontend/dockTree.test.ts` -- các ca trong ma trận. Đối chứng đỏ bằng phép gỡ thật: bỏ `sticky`, đặt lại `resetReadingMarks()` trên đường mở hàng, bỏ bộ lọc persist.
- [ ] `e2e/specs/story-5-13-reading-marks.e2e.mjs` -- sau "Sửa →", panel có trong Workspace với đủ hàng; bấm "Sửa →" hàng 2 thì vẫn ở Workspace. Sau khi cuộn, `top` của thanh bằng mép khung cuộn và `elementFromPoint` ở giữa thanh trúng thanh.

**Acceptance Criteria:**
- Given panel đang hiện, when bấm "Đóng", then panel biến mất và tiêu điểm về lưới.
- Given panel đang hiện, when gõ vào ô dịch, then gõ bình thường và Tab không bị giữ trong panel.
- Given panel đang hiện, when thoát app rồi mở lại, then Workspace không có panel đánh dấu nào.

## Implementation Notes

- `top: 0` trong Code Map sai: sticky lấy mốc từ hộp nội dung của `.mode`, nên e2e đo được thanh nằm thấp hơn mép khung cuộn 14px. Đúng là `top: calc(-1 * var(--space-panel-block))`.
- Thân danh sách nằm ở `src/modes/ReadingMarksList.vue`. Nút "Đóng" là slot, nên bảng modal đóng bằng `reading.marks_close`, còn panel đóng bằng `reading.marks_panel_close`.
- `ReadingMode.onDeactivated` cũng gọi `resetReading` và từng xoá danh sách ở bước sau "Sửa →". Vì vậy chỗ đó giữ `keepMarks` khi panel đang hiện.
- `PERSISTED_PANEL_IDS` thay `PANEL_IDS` ở mọi chỗ ghi/khôi phục bố cục, ở hậu tố lệnh `layout.toggle_*` và ở một mệnh đề của `check-layout.mjs`. `withoutPanels` (`dockTree.ts`) lọc panel khỏi JSON và trao bề ngang cho panel kề bên.
- Panel nhận tiêu điểm ở gốc `PanelFrame` rồi chuyển vào `.marks-body`, vì phím ↑ ↓ ↵ ⌫ chỉ chạy trên gốc của danh sách. `PanelFrame` có thêm `data-panel-owner` để test nhận ra panel nào đang giữ tiêu điểm.
- Đối chứng đỏ (gỡ thật, từng cái một): chốt panel cuối, gỡ ở `short`, lọc `floatingGroups`, chuyển tiêu điểm đều đỏ đúng ca. Gỡ `restoreFocusIfLost('panel.grid')` trong `syncReadingMarksPanel` thì ca "Đóng ⇒ lưới" vẫn xanh, nên tiêu điểm về lưới nhờ một đường khác của dock; ca đó canh kết quả, không canh dòng này.
- WebDriver nhúng chỉ gửi sự kiện chuột, không gửi `pointerdown`. Vì vậy e2e tự phát `pointerdown` lên hàng trước cú bấm. Với chuột thật, phần này chưa được đo.
- Đã có sẵn từ trước, không do thay đổi này: `controls-adoption.test.ts` đỏ 2 ca ở `.toc-row` của `ReadingMode.vue`, implementer đo cùng hai ca đỏ trên bản HEAD chưa sửa; `.toc-row` vào từ commit 48c99ff4.

## Plan Change Log

- 2026-11-04, Ice dùng thử: mọi việc ổn, nhưng muốn panel đứng chung nhóm tab với Tra cứu và Đề xuất AI. `syncReadingMarksPanel` thêm panel `within` Tra cứu, hoặc Đề xuất AI nếu thiếu Tra cứu. Ca mới đo ở cả tầng `full` lẫn cửa sổ mặc định (`short`). Đối chứng: trả về `right` của lưới ⇒ đỏ. Lọc panel tạm khỏi neo của `rememberSpot` đã thử rồi bỏ, vì gỡ nó thì ca "Tra cứu về đúng chỗ" vẫn xanh (dock dựng lại từ cây đã chụp và đã lọc). GIỮ: panel tạm, không persist, gỡ ở `narrow`/`unsupported`.

## Review Triage Log

Lượt 1 (quick): high 0 · medium 5 · low 7 · false 1 · maybe-false 0.
- medium · patch — thanh dính `z-index: 2` nằm cùng ngữ cảnh xếp lớp với hai overlay `fixed` (z auto), nên vẽ đè lên 42px đầu của Mục lục và bảng đánh dấu. Sửa: overlay có z-index cao hơn.
- medium · patch — `withoutPanels` không lọc `floatingGroups`; panel thả nổi để lại một view trỏ tới panel đã xoá, lần khôi phục kế ném lỗi và rơi về preset mặc định.
- medium · patch — chốt "panel cuối cùng" của `hidePanel` đếm panel tạm làm hàng xóm, nên `layout.toggle_grid` gỡ được lưới.
- medium · patch — hàng "Cửa sổ hẹp" ghi rõ `short`; mã giữ panel ở `short`. Sửa theo hàng đã khoá, bỏ ghi chú đọc lại hàng. 🔵 Sau bản vá e2e đỏ: cửa sổ mặc định 1280×860 (vùng làm việc khoảng 786px, dưới 820px) là `short`, nên panel không bao giờ hiện. Ice chọn giữ panel ở `short`; hàng đã sửa theo quyết định đó.
- medium · patch — vào panel qua `enterFocus`/`focus.next_panel` thì ↑ ↓ ↵ ⌫ không chạy (`target === currentTarget`); câu nhắc "Mở Workspace…" hiện trong chính Workspace.
- low · patch — `pointer.at` của `reading.marks_next/prev` còn trỏ `ReadingMode.vue`.
- low · patch — ca "Đóng" gọi thẳng hàm state và chỉ kiểm tiêu điểm khác `body`, không canh dây lệnh và AC "về lưới".
- low · patch — ca "mở Tác phẩm khác" gọi thẳng `resetReading()` thay vì `openWorkById`.
- false — ca khởi động lại không thể đỏ khi gỡ bộ lọc persist: đúng, nhưng bộ lọc đã được ca đầu của `readingMarksDock` canh (`currentLayoutJson()` không chứa `reading_marks`), và implementer đã gỡ bộ lọc ⇒ ca đó đỏ. Không mất lưới.
- low · patch — vitest quét CSS cho thanh dính trùng với e2e, trái `tests/AGENTS.md`. Xoá ca vitest.
- low · patch — phần kiểm bằng chuột thật chưa có mục nợ. Đã ghi vào `deferred-work.md`, Chủ: Ice.
- low · patch — bốn comment nói "ba panel" thành sai sau thay đổi.
- low · patch — doc comment trên hai hàm riêng (`resetAfterMarkOpen`, `syncReadingMarksPanel`).

## Verification

**Commands:**
- `npx vitest run tests/frontend/readingMarks.test.ts tests/frontend/readingToolbar.test.ts tests/frontend/statusBarReadingWalk.test.ts` -- expected: xanh, thoát 0, không có dòng Errors.
- `npx wdio run e2e/wdio.conf.mjs --spec e2e/specs/story-5-13-reading-marks.e2e.mjs` (sau `cargo build --locked --features wdio`, ngoài sandbox) -- expected: xanh.
