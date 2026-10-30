---
title: 'Rà bề mặt Library, trang Tác phẩm và Chế độ đọc theo lưới 7px'
type: 'refactor'
ticket: '7'
created: '2026-10-30'
status: 'built'
baseline_revision: '5c776887335131e53c9e269b13b959442016835a'
route: 'full'
route_source: 'auto'
risk: 'medium'
review: 'quick'
review_source: 'pinned'
lenses_ran: ['quick']
review_loop_iteration: 0
context:
  - '{project-root}/src/AGENTS.md'
  - '{project-root}/e2e/AGENTS.md'
---

<frozen-after-approval reason="human-owned intent — do not modify unless human renegotiates">

## Intent

**Problem:** Phép chuyển máy móc của entry 1 để Library, trang Tác phẩm và Chế độ đọc lệch lưới: khối có viền hay vạch cộng bề dày nét vào đệm (mép chữ cách mép ngoài 15, 17), khe giữa nhóm còn 7, khoảng cách đoạn ở trang đọc là `1em` (17,5px ở mức Cân), và hằng ảo hoá của danh sách Chương còn 44/440 trong khi CSS đã 42/441 (DESIGN.md §Lưới 7px — Trong nhóm 7, giữa nhóm 14 · Nét kẻ nằm trong ô lưới).

**Approach:** Một spec e2e đo thẻ Library, hàng trang Tác phẩm và trang đọc trên WKWebView trước, rồi sửa CSS (và hai hằng ảo hoá) của ba bề mặt cho tới khi spec xanh. Spec cũng trả nợ của Story 13.5: đo hộp dòng của `.column` vẽ ra là bội 7.

**Decision (Ice 2026-10-30):** khoảng đoạn → đoạn ở Chế độ đọc là **một hộp dòng** (`1lh`: 28 · 35 · 42px ở `sm`/`md`/`lg`, như mockup mục 4), thay cho `1em`; tiêu đề Chương → đoạn đầu cũng một hộp dòng của chữ đọc (hôm nay 14). Ảnh (`.reading-figure`) và ghi chú (`.source-note`) dùng cùng nhịp với `.paragraph`. Phương án bị bác: `round(up, 1em, 7px)` (21px cả ba mức).

**Decision (Ice 2026-10-30, lúc triển khai):** tiêu đề Chương và dòng nguồn bản dịch `.chapter-origin` (luôn có, mockup không vẽ) là một nhóm: tiêu đề → dòng nguồn 7, dòng nguồn → đoạn đầu một hộp dòng chữ đọc. Phương án bị bác: hộp dòng nằm giữa tiêu đề và dòng nguồn (tiêu đề → đoạn đầu 70px ở mức Cân); giữ 14 giữa tiêu đề và dòng nguồn.

## Boundaries & Constraints

**Always:** Khối có viền 1px trừ 1px ở đệm cùng phía; vạch trái 2/3px trừ bề dày vạch ở đệm trái. Khe ngang giữa các nút và giữa nhãn với điều khiển của nó là 7; khoảng giữa hai nhóm (thông tin · tiến độ · hàng nút) là 14. Mỗi giá trị lệch lưới vì bù nét mang `/* aura-allow-spacing: <reason> */` cùng dòng. Hằng `ITEM_HEIGHT_PX`/`VIEWPORT_HEIGHT_PX` và `.wp-item`/`.chapters-list` bằng nhau như trước Story 13.1 (42 · 420, viền trên 1px nằm trong `max-height` như cũ).

**Never:** Đổi cấu trúc markup của thẻ Library (bìa trên, nút Tiếp tục hiện khi rê chuột) cho giống sơ đồ `84px 1fr` của mockup mục 6; mockup ở đó chỉ minh hoạ 7/14. Đổi bìa 133px, cỡ chữ, màu, `readingStyle`/`readingState.ts`, giá trị lưu của thanh trượt. Chạm Workspace, Tra cứu, Cài đặt, lớp phủ (entry 6, 8). Thêm token `--space-*` chỉ để lách Kiểm K.

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected Output / Behavior | Error Handling |
|----------|--------------|---------------------------|----------------|
| Thẻ Library | một Tác phẩm trong lưới Library | cao ngoài `.work-cell` là bội 7; đỉnh và mép trái chữ đầu tiên của `.work-body` cách mép ngoài thẻ một bội 7; khe `.work-meta` → thanh tiến độ 14 | — |
| Hàng trang Tác phẩm | Tác phẩm ≥ 25 Chương, mở trang | `.wp-item` cao 42; mép chữ `.chapter-row` cách mép ngoài vạch 14; `scrollHeight` của `.chapters-list` = số mục × 42 (± 1 viền) | — |
| Trang đọc | Chương `done`, Chế độ đọc, ba mức `sm`/`md`/`lg` | hộp dòng vẽ ra của đoạn là bội 7 (±0,5); khoảng đoạn → đoạn và dòng nguồn → đoạn đầu bằng đúng một hộp dòng đó (28 · 35 · 42); tiêu đề → dòng nguồn 7 | — |
| Nợ 13.5 | cỡ 21px, tỉ số 2.34 · 2.37 · 2.4 qua thanh trượt | hộp dòng vẽ ra là 56 (không 55,99) | — |
| Bẫy | trả `ITEM_HEIGHT_PX` về 44 | ca hàng trang Tác phẩm đỏ | — |

</frozen-after-approval>

## Code Map

- `e2e/specs/story-13-6-ra-be-mat-workspace.e2e.mjs` -- khuôn: `onSeven` (lệch ≤ 0,5 so với bội 7), đo trong `browser.execute`. Mở Library/trang Tác phẩm: `e2e/support/libraryGrid.mjs` (`WORK_CELL`, `aimWork`, `openAimedWork`), `openWorkPage()` ở `story-12-4-work-page-chapter-list.e2e.mjs:125`. Mở Chế độ đọc: `story-5-11-reading-mode.e2e.mjs:80–95` (`openWorkspaceWithWork`, đặt Chương `done` qua IPC, `Meta+3`, chờ `.column`).
- `src/modes/LibraryMode.vue` -- `.work-cell:1144` viền 1, không đệm; `.work-cover:1163` 133px, đệm 14 (`:1170–1171`) ⇒ 13 ở cạnh có viền; `.work-body:1216–1217` đệm 14 ⇒ 13, `gap` 7 nối cả meta · tiến độ · số (meta → tiến độ phải 14, tiến độ → `.work-stat` giữ 7); `.work-meta:1234` `min-height: 1.4em` ⇒ một dòng `ui` 21px. `.orphan-row:1082` viền 1 + đệm 14; `.root-block:1028–1030`, `.works-block:1125–1127` đệm dưới 14 + viền 1 ⇒ 13. `.top-bar` lề 21 và `.works-grid` gap 21 đã đúng lưới, giữ.
- `src/modes/LibrarySearchResults.vue` -- `.search-chip:469/474` viền 1 + đệm 7/14 ⇒ 6/13; `.hitrow:539–541` đệm dưới 7 + viền 1 ⇒ 6.
- `src/modes/LibraryWorkPage.vue` -- `:68–69` `ITEM_HEIGHT_PX = 44`, `VIEWPORT_HEIGHT_PX = 440` (13.1 đổi CSS `:1331` 44→42 và `:1306` 440→441 mà quên hai hằng; đệm ảo hoá `:600`, `:712` và cuộn tới mục `:274–275` dùng 44). `.chapter-row:1380–1381` vạch 3 + đệm trái 14 ⇒ 11; `.wp-status:1143–1144` viền trái 1 + 14 ⇒ 13; `.wp-sect:1553–1554` viền trên 1 + 14 ⇒ 13; `.wp-head:1065–1066` đệm dưới 14 + viền ⇒ 13; `.wp-bar:1204–1205` đệm 7 + viền dưới ⇒ 6; `.wp-status-actions:1196` lề trên 7 ⇒ 14 (hàng nút là nhóm riêng). `.wp-title`, `.wp-sub`, `.wp-legend`, `.wp-crumb` đã đúng.
- `src/modes/ReadingMode.vue` -- `.mode:1039` đệm 14·14 (token panel), giữ. `.toolbar:1051` gap 14 ⇒ 7 (khe giữa nút); `.tuner:1059` cột các hàng thiết lập giữ 14, `.tuner-row:1066` (nhãn · thanh trượt · số) ⇒ 7; `.level-label:1372` `margin-inline-end`, `.kbd:1380/1388` `margin-inline-start` 14 ⇒ 7 (thuộc tính logic, Kiểm K không quét). `.marks-row:1343–1345`, `.mark-actions:1189`, `.toc-actions:1406` khe giữa nút ⇒ 7; đệm dưới `.marks-row` 14 + viền ⇒ 13. `.frontier:1232–1234`, `.progress-line:1396–1397` viền 1 + đệm 14 ⇒ 13. `.chapter-title:1127` lề dưới 14; `.paragraph:1135`, `.reading-figure:1144`, `.source-note:1104` lề dưới `1em` có miễn trừ; theo Decision đổi sang một hộp dòng. `1lh` của `.chapter-title` là hộp dòng của tiêu đề (35), không phải của chữ đọc: tiêu đề lấy `calc(var(--reading-line-height) * var(--reading-font-size))` hoặc tương đương. Kiểm `CSS.supports('margin-bottom', '1lh')` trên WKWebView trước; không hỗ trợ ⇒ dừng, trình Ice. Ô đánh dấu `:825–827` đặt `top/left` px bằng JS: vị trí, không phải khoảng, không đụng.
- `src/modes/readingState.ts:712–743` -- `--reading-line-height` là tỉ số `bộiBảy / cỡ`; chỉ đọc.
- `e2e/specs/story-12-24-library-wide-grid.e2e.mjs:322` ghim `coverHeight === 133`, `:276–278` bề rộng thẻ 239–306 -- phải còn xanh.

## Tasks & Acceptance

**Execution:**
- [ ] `e2e/specs/story-13-7-ra-be-mat-library-doc.e2e.mjs` -- các ca của Matrix trước mọi sửa; ghi ca nào đỏ trên cây gốc -- R1, R6.
- [ ] `src/modes/LibraryWorkPage.vue` -- hai hằng ảo hoá 42 · 420, `max-height` 420; bù nét và khe nhóm theo Code Map -- R1.
- [ ] `src/modes/LibraryMode.vue`, `src/modes/LibrarySearchResults.vue` -- bù nét, khe meta → tiến độ 14, `min-height` 21px -- R1, R6.
- [ ] `src/modes/ReadingMode.vue` -- bù nét, khe trong nhóm 7, khoảng đoạn, ảnh, ghi chú và tiêu đề → đoạn đầu bằng một hộp dòng chữ đọc theo Decision -- R1, R6.

**Acceptance Criteria:**
- Given app chạy ngoài sandbox, when spec `story-13-7` chạy, then xanh; given phép gỡ ở hàng Bẫy, then đỏ đúng ca hàng trang Tác phẩm.
- Given cây sau story, when `npm run check:tokens`, then xanh, và mỗi miễn trừ `aura-allow-spacing` mới nằm trên một khai báo bù đúng bề dày nét cùng phía (số miễn trừ ghi ở Implementation Notes).
- Given spec `story-12-24`, `story-12-4`, `story-12-34`, `story-5-11`, `story-13-2`, `story-13-4`, when chạy lại, then xanh, hoặc ca đỏ được khoanh là đỏ cả trên cây gốc.
- Given ca nợ 13.5 xanh, then mục nợ Story 13.5 trong `deferred-work.md` được đóng bằng `→ ✅ ĐÃ ĐÓNG`.

## Implementation Notes

- Cây gốc đỏ cả năm ca bố cục (thẻ cao 222, chữ ở 148/15; hàng chương 17, `scrollHeight` 1202; khoảng đoạn 19 · 17,5 · 16 ở ba mức); ca nợ 13.5 xanh từ trước (56 ở 21px × 2.34 · 2.37 · 2.4). Hàng Bẫy (`ITEM_HEIGHT_PX` 44) đỏ đúng ca hàng trang Tác phẩm.
- Thẻ Library: viền 1 + đệm 13; bìa đệm `13px 13px 14px`; `.work-meta` có `margin-bottom: 7px` cộng `gap` 7 thành 14. Thanh tiến độ 3px không lọt lưới nên `margin-bottom: -3px` (thanh chiếm một dải 7px) để thẻ cao 224 = 32 × 7; với "tiến độ → số 7" nguyên văn thẻ cao 227.
- Tiêu đề Chương và `.chapter-origin` là một nhóm (Decision thứ hai): `.chapter-title` lề dưới 7, `.chapter-origin` lề dưới một hộp dòng chữ đọc bằng `calc(var(--reading-line-height) * var(--reading-font-size))` (`1lh` ở đó là dòng ui-sm 21).
- `.source-note` dùng `calc(var(--reading-line-height) * var(--reading-font-size))`, không `1lh` (cỡ chữ riêng); `.paragraph`, `.reading-figure` dùng `1lh`. `CSS.supports('margin-bottom','1lh')` đúng trên WKWebView.
- `.chapters-list` `max-height: 420px` (border-box, `clientHeight` 419) theo Boundaries; 20 dòng thêm mang `aura-allow-spacing` trong `src/modes/` (3 thay câu "theo em" cũ).
- `story-12-34` đỏ ở `seedWork` (chương cuối 1 câu thay vì 4) ở bước IPC, trước mọi bố cục; chưa chạy trên cây gốc.

## Plan Change Log

## Review Triage Log

Lượt 1 (quick): 7 dòng — low 4 (patch 1 · reject 3) · medium 1 (defer) · false 2.

- false — "`.work-meta` `margin-bottom: 7px` làm meta → `.work-stat` 14 khi không có thanh tiến độ (`chapter_done_count` null)": khi không có thanh, `.work-stat` mở đầu nhóm tiến độ nên cách meta 14 là đúng luật giữa nhóm; mọi phần của thẻ vẫn là bội 7 nên thẻ vẫn cao bội 7.
- low · reject — spec không đo `.source-note` và `.reading-figure`: đúng, gỡ hai khai báo đó suite vẫn xanh; gieo ảnh (6.14) hay Tác phẩm song ngữ là việc thêm fixture, không phải sửa trực tiếp, và `calc(tỉ số × cỡ cột)` của ghi chú là cách đúng vì `1lh` ở đó là dòng `read-lg` của chính ghi chú.
- medium · defer — AC chạy lại `story-12-34` không đạt: đỏ ở bước gieo IPC, chưa chạy trên cây gốc. Cùng triệu chứng với mục nợ của Story 13.4 (Chủ: Epic 12); diff không chạm `src-tauri` hay lệnh gieo. Đã nối một dòng `→` vào mục đó, không thêm mục trùng.
- false — "Tasks còn `[ ]`, chưa ghi kết quả năm spec còn lại": trạng thái nằm ở frontmatter; AGENTS.md cấm ghi số test trung gian vào Implementation Notes.
- low · patch — ca 21px dispatch `input` mà không kiểm giá trị đã áp; ba tỉ số đều ra 56 nên tỉ số không áp vẫn xanh. Thêm assert cỡ chữ cột 21px và `value` thanh trượt bằng tỉ số vừa đặt.
- low · reject — thẻ không có thanh tiến độ không được đo: cùng gốc với dòng false đầu; mọi phần còn lại là bội 7, thêm ca cần gieo Tác phẩm không đếm được tiến độ.
- low · reject (người điều phối tự ghi) — `.work-progress-track` `margin-bottom: -3px` để thanh 3px chiếm dải 7, nên thanh → số còn 4px thay vì 7 như Code Map; giữ 7 thì thẻ cao 227. Matrix (meta → thanh 14, thẻ bội 7) đạt; mắt nhìn thuộc lượt dùng thật của Ice (`Chủ: Epic 13`).

## Design Notes

Đo từ mép ngoài khối có viền, không từ hộp nội dung: viền 1 + đệm 13 = 14. Bìa 133 = 19 × 7 nằm dưới viền trên 1px, nên đệm trên `.work-body` là 13 để chữ đầu ở 147 = 21 × 7.

```css
.chapter-row {
  padding-left: 11px; /* aura-allow-spacing: 3px status rule sits inside the 14px inset */
}
```

Khoảng đoạn `1lh` phải qua Kiểm K bằng một miễn trừ có lý do (hộp dòng đã là bội 7 do `readingStyle` dựng), vì cổng chỉ nhận px bội 7 hay `var(--space-*)`; câu miễn trừ cũ "theo em" được thay, không giữ.

## Verification

**Commands:**
- `npm run check:tokens` -- expected: exit 0.
- `npm run test:e2e -- --spec e2e/specs/story-13-7-ra-be-mat-library-doc.e2e.mjs`, rồi các spec ở AC thứ ba, ngoài sandbox -- expected: pass.
- vitest của các tệp đổi (`npx vitest run tests/frontend/<tên>` cho mọi tệp test import `LibraryWorkPage`, `LibraryMode`, `ReadingMode`) -- expected: 0 failed, 0 Errors.

**Manual checks (if no CLI):**
- Không có; lượt nhìn app thật của Ice thuộc mục nợ `Chủ: Epic 13`.
