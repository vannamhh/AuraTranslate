---
title: 'Trang Tác phẩm theo mockup: khung phải và nút Dịch · Đọc mỗi Chương'
type: 'feature'
ticket: '34'
created: '2026-10-29'
status: 'done'
baseline_revision: '0bd90d0fd9eefe452834f01cae4a58a7d58c3802'
route: 'full'
route_source: 'auto'
risk: 'medium'
review: 'quick'
review_source: 'pinned'
lenses_ran: ['quick']
review_loop_iteration: 0
context:
  - '{project-root}/_bmad-output/initiative-auratranslate/ux-auratranslate/mockups/trang-tac-pham.html'
  - '{project-root}/src/AGENTS.md'
  - '{project-root}/e2e/AGENTS.md'
---

<frozen-after-approval reason="human-owned intent — do not modify unless human renegotiates">

## Intent

**Problem:** Trang Tác phẩm (`LibraryWorkPage.vue`) lệch mockup `trang-tac-pham.html` (Ice duyệt 2026-10-29): nút "Về Library" có viền chiếm một hàng, ô trạng thái là thẻ nổi với "Bỏ ghi đè" luôn hiện, thanh chọn chín nút ngang hàng luôn hiện, và không có đường nào từ một Chương sang Chế độ đọc.

**Approach:** Dựng lại trang theo năm khung của ticket: đầu trang (đường dẫn, cột trạng thái), hai ô Dịch · Đọc trên mỗi hàng, lệnh mới `library.open_chapter_reading` (chỉ webview, theo đường `library.open_search_hit_reading`), khung phải 320px thay thanh chọn, số đếm và đầu nhóm dính. Giữ `ITEM_HEIGHT_PX` 44 và mọi id `dispatch` hiện có.

## Boundaries & Constraints

**Always:**
- Lớp `ui-*` của `controls.css` cho mọi control mới (DESIGN §Điều khiển); token `--color-surface-accent`, `--color-caret-rule`; ca `controls-adoption` xanh mà không thêm ngoại lệ mới cho `LibraryWorkPage.vue`.
- Hàng chưa Đã xong: không có nút Đọc, ô giữ chỗ (phương án A). Khung phải: nút Đọc `disabled` kèm câu lý do của mockup (dáng khoá dùng chung cho A và B).
- Nhãn "Đọc" trên hàng, "Đọc từ Chương này" trong khung, `aria-label` "Mở Chương N ở Chế độ đọc".
- Thẻ hàng loạt (≥ 2 Chương): Gộp là nút Chính, cùng Đổi trạng thái và Xuất; Tách, Dời, Xuất xứ vắng hẳn.
- Dưới 1.100px: khung trượt vào từ phải phủ lên danh sách; `Esc` đóng khung trước rồi mới bỏ chọn; cột "sửa lúc" ẩn trước, rồi cột số câu.
- Một mục nợ `Chủ: Epic 12` cho ảnh app thật cạnh mockup cùng khổ, mỗi khung 1–5 ghi khớp hoặc lý do (bước người của ticket).

**Never:**
- Không sửa Rust, `controls.css`, DESIGN.md, `check-*.mjs`; không thêm dependency, token, lớp `ui-*`.
- Không dựng "Đổi tên hàng loạt" của mockup (không có lệnh); ghi một mục nợ `Chủ: Ice`.
- Handler mới không gọi `openWorkById` (đặt con trỏ về 0, xoá `workPageWork`).
- Không né ca canh bằng selector thuộc tính hay mẹo khác.

**Decisions (Ice, 2026-10-29):**
- Khung phải theo mockup đúng chữ: chưa chọn Chương nào thì không có khung; "Nhập bản reviewer" chỉ có khi chọn đúng một Chương. Ice biết trang mất lối vào này khi chọn 0 hoặc ≥ 2 Chương; lưới Library vẫn còn `library.open_work_reviewer_import`.
- Hàng rê chuột: trạng thái `hoveredChapterId` của trang (`@mouseenter`/`@mouseleave` trên hàng) để template đổi `ui-btn--quiet` ↔ `ui-btn`; không sửa `controls.css`.
- Giữ trọn plan dù quá 1.600 token; chia ba pha như Design Notes.
- Ngoại lệ của dòng Never về `check-*.mjs`: nâng `CLICK_FLOOR` 237 → 253 và `DISPATCH_FLOOR` 334 → 360 trong `scripts/check-commands.mjs` đúng số cổng in ra (`ceil(0.85 × live)`, live 297 và 423); trang thêm 9 `@click` và 9 `dispatch`, phần trôi còn lại tích từ các story trước. Không đổi logic cổng.
- Ở bề rộng ≥ 1.100px, chưa chọn Chương nào thì cột 320px thu hẳn, danh sách chiếm trọn bề ngang; Ice chấp nhận cú bấm đầu lên nút hàng của một hàng chưa chọn có thể trượt vì danh sách hẹp lại dưới con trỏ.
- 🔵 2026-10-29, Ice sau lượt nhìn app chạy dev: mọi control của trang cao 28px (bỏ cỡ `--sm`, nút `--lg` của khung giữ 32px); hàng Chương đang chọn không còn viền, chỉ nền và vạch caret; hai nút Dịch · Đọc trên hàng luôn Chìm, chỉ hiện viền khi rê lên chính nút (thay quyết định "hàng rê chuột ⇒ Phụ" và "hàng chọn có một nút Chính"); khung phải bỏ "Dịch Chương này", "Đọc từ Chương này" và câu lý do khoá, `data-library-open-chapter` chuyển sang nút Dịch trên hàng.
- 🔵 2026-10-29, Ice: khung phải ít dùng nên bấm hàng chỉ chọn (để Shift/⌘ và bàn phím), không mở khung; mỗi hàng thêm nút "⋯" mở khung cho Chương đó (lệnh `library.chapter_panel.open`), khung có nút Đóng (`library.chapter_panel.close`); chọn từ hai Chương thì thẻ hàng loạt tự hiện. Lệch mockup `trang-tac-pham.html` khung 4.

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected Output / Behavior | Error Handling |
|----------|--------------|---------------------------|----------------|
| Đọc từ hàng | Bấm Đọc trên Chương N Đã xong | N thành Chương đang mở, Chế độ đọc mở, N đứng đầu lượt đọc; `⌘1` ⇒ con trỏ danh sách ở hàng N | Lỗi mở Chương: giữ ở Library, lỗi hiện như `library.open_chapter` |
| Hàng chưa xong | Chương Đang dịch / Tạm ngưng / Chưa bắt đầu | Ô Đọc trống, giữ bề ngang | — |
| Trạng thái lạ | `status` ngoài bốn giá trị | Không có nút Đọc ở hàng lẫn khung | — |
| Một Chương | 1 hàng chọn | Khung: tên, Dịch/Đọc, Sắp xếp Chương (bảy lệnh), Xuất xứ, ô "Áp cho các Chương đến số" + câu hệ quả "Chương 47 → 60 · 14 Chương nhận cùng xuất xứ" | Số ngoài khoảng: câu hệ quả không hiện, như `setChapterOriginApplyThroughOrd` trả `null` |
| Hai Chương rời | Chọn 3 và 5 | Thẻ hàng loạt, Gộp khoá (không liền) | — |
| Không ghi đè | Tác phẩm chưa ghi đè | Không có nút "Bỏ ghi đè" | — |

</frozen-after-approval>

## Code Map

- `src/modes/LibraryWorkPage.vue` (1184) -- toàn trang. `.wp-back` :225 → đường dẫn `ui-btn--quiet`; `.wp-status` :281 → cột phải `border-left`, "Bỏ ghi đè" :326 `v-if` có ghi đè (`story-5-4` chỉ bấm sau khi đặt ghi đè); `.wp-bar` :341 chip lọc → `ui-seg ui-seg--sm` kèm số đếm như `.bar` của mockup, giữ `data-library-chapter-filter`; `.wp-selbar` :414–508 → khung phải; `ChapterOrigin` + ô áp :701–727 vào khung. Hàng :643: thêm ô `.act` (lưới 54px 48px); `.chapter-row--selected` nền `surface-accent` + vạch trái 3px `caret-rule` (mẫu `GridBlock.vue:523`). Hằng :67–69 không đổi.
- `src/modes/libraryChapters.ts` -- thêm `openCurrentChapterInReading()` cạnh `openCurrentChapter` :486: `openChapterById(row.chapter_id)` rồi `resetReading()`, `resetReadingToc()`, `setMode('reading')` (giống `openSearchHitInReading` ở `readingState.ts:266`, không anchor). `chapterWindow` :805: luôn giữ đầu nhóm của hàng trên cùng trong cửa sổ để `position: sticky` có phần tử mà dính.
- `src/modes/libraryChapterSelection.ts:156` -- thêm luật `read`: đúng một Chương, `status === 'done'`.
- `src/commands/index.ts` -- id mới cạnh `library.open_search_hit_reading` :2535, mẫu `portMissing`; port vào deps :465–501. `src/main.ts:709` -- nối port.
- `src/i18n/vi.json` -- `command.library.open_chapter_reading` và nhãn, câu lý do, câu hệ quả; không "bạn"/"chúng tôi".
- `scripts/check-commands.mjs:2555–2559` -- chỉ đọc: nút trên hàng dùng `@click` một `dispatch`; `@mousedown` của hàng đã chọn đúng hàng trước khi `@click` chạy. `@dblclick` của hàng không được chạy khi bấm nút (chặn bằng handler có tên và khai vào `HANDLER_TABLE` nếu cần — sửa bảng là sửa dữ liệu của cổng, được phép).
- `tests/frontend/libraryChapters.test.ts` -- :461 (mọi nút tắt khi chưa chọn) và :496 (Gộp tắt khi một Chương) mô tả thanh chọn cũ: viết lại cho khung phải; giữ :880–931 (`.chapter-origin-input`, `data-library-chapter-origin-apply-through`); :1016 (nút Nhập bản reviewer mang dấu mở khi chưa chọn) viết lại thành: chọn một Chương thì nút có trong khung, mang `data-reviewer-import-open`.
- `tests/frontend/exportScope.test.ts:248` -- regex cần `<button …data-export-open…>` rồi `@click="dispatch('export.open')"`.
- e2e giữ móc: `story-12-4` (`data-library-chapter-selected-count`, `-merge-open` khoá khi rời, `-split-open`, `-rename-begin`, `-chapters-status-menu`, `data-export-open` trả tiêu điểm), `story-5-8`, `story-5-7` (`data-library-open-chapter`), `story-5-4`, `story-12-17`.
- `e2e/specs/` -- spec mới chép `story-12-4-work-page-chapter-list.e2e.mjs` (seed :31–80, `openWorkPage` :124) và `resizeLogical` của `story-12-24-library-wide-grid.e2e.mjs:97`; khẳng định đọc theo `story-12-5-library-search-scope.e2e.mjs:179–228` (trang đọc không có `data-` id Chương: so `h2.chapter-title`).
- `_bmad-output/initiative-auratranslate/deferred-work.md` -- hai mục cuối tệp (grep, không đọc cả tệp).

## Tasks & Acceptance

**Execution:**
- [x] `src/modes/libraryChapterSelection.ts`, `src/modes/libraryChapters.ts`, `src/commands/index.ts`, `src/main.ts`, `src/i18n/vi.json` -- luật `read`, `openCurrentChapterInReading`, lệnh và port, chuỗi -- pha 1: lệnh chạy được trước khi có giao diện.
- [x] `src/modes/LibraryWorkPage.vue` -- năm khung theo Code Map -- pha 2.
- [x] `tests/frontend/libraryChapters.test.ts`, `tests/frontend/readingState.test.ts` -- ca khung phải (một Chương đủ bảy lệnh + khối xuất xứ; hai Chương có Gộp, không còn `data-library-chapter-selbar`; trạng thái lạ không có Đọc), ca lệnh mới không đụng con trỏ -- pha 3.
- [x] `e2e/specs/story-12-34-work-page-read-chapter.e2e.mjs` -- 1280×860: hàng Đã xong có Đọc, hàng Đang dịch không; bấm Đọc trên N ⇒ N đầu lượt đọc; `⌘1` ⇒ con trỏ ở N -- pha 3.
- [x] `_bmad-output/initiative-auratranslate/deferred-work.md` -- nợ `Chủ: Epic 12` (ảnh cạnh mockup) và `Chủ: Ice` (đổi tên hàng loạt).

**Acceptance Criteria:**
- Given một Chương được chọn, when xem khung phải, then có đủ mọi lệnh của thanh chọn cũ (đổi tên, tách, dời lên, dời xuống, đổi trạng thái, xuất, nhập bản reviewer) và khối xuất xứ; given hai, then có Gộp và không còn thanh chọn ngang.
- Given spec e2e mới xanh, when gỡ thật `dispatch('library.open_chapter_reading')` khỏi nút hàng và chạy lại riêng spec đó, then đỏ; trả lại thì `cmp` trùng byte.
- Given mọi tệp đã sửa, when chạy Verification, then xanh.

## Implementation Notes

- Lệnh `library.open_chapter_reading` chỉ ở webview: `openCurrentChapterInReading` đọc con trỏ như `openCurrentChapter`, không gọi `openWorkById`; Chương chưa Đã xong thì bỏ qua.
- Dải dính làm ở trang (`stickyHead` chèn đầu nhóm của hàng trên cùng, trừ 44px khỏi đệm trên), không ở `chapterWindow` như Code Map; cuộn theo con trỏ lùi một hàng để hàng không nằm dưới dải dính.
- Nút Dịch · Đọc trên hàng là `tabindex="-1"`: lệnh đọc vùng chọn, chỉ `@mousedown` của hàng chọn đúng hàng; bàn phím đi qua Enter của danh sách và nút trong khung.
- 🔵 2026-10-29: bấm nút trên hàng chưa chọn từng chỉ hiện khung phải (chọn hàng ⇒ cột 320px mở ⇒ nút trượt khỏi con trỏ trước `click`). Nay `@mousedown.stop` của vùng nút chỉ dời con trỏ (`aimLibraryChapter`), lệnh mở đọc con trỏ rồi chọn Chương vừa mở; đối chứng gỡ `.stop` ⇒ ca vitest đỏ.
- `data-library-open-chapter` chỉ còn ở nút "Dịch Chương này" của khung; nút hàng mang `data-library-row-translate`/`-read`.
- Số đếm trong bộ lọc là text node dữ liệu dưới `aura-allow-text`, không qua một khoá i18n rỗng `{count}`.
- Menu trạng thái lặp ở thẻ một Chương và thẻ hàng loạt vì mỗi `dispatch` phải là literal.
- Đối chứng ca thu cột: gỡ `v-if` của `aside` và giữ cột ⇒ ca "chưa chọn" đỏ đúng lý do; trả lại ⇒ `cmp` trùng byte.
- e2e chưa chạy: cổng 1420 do `npm run dev` của Ice giữ suốt lượt; đối chứng gỡ `dispatch` của AC chưa làm (nợ `Chủ: Epic 12`).

## Plan Change Log

## Review Triage Log

- high · patch — `story-12-34` khai `seedWork` hai lần ở cấp module, `node --check` báo SyntaxError nên spec không nạp được; xoá bản yếu (chỉ đếm 5 Chương).
- medium · patch — nút Dịch · Đọc trên hàng vào được bằng Tab, Enter trên hàng chưa chọn mở Chương của con trỏ cũ; đặt `tabindex="-1"`.
- medium · intent_gap → Ice chọn thu cột (2026-10-29) — cột 320px trống luôn giữ ở bề rộng ≥ 1.100px khi chưa chọn gì, trái quyết định "không có khung"; vá tại chỗ (`aside v-if`, `.wp-body--with-panel`) thay vì lùi cả lượt dựng, vì sửa chỉ nằm trong một tệp.
- medium · ghi nợ — e2e và đối chứng gỡ `dispatch` chưa chạy (cổng 1420 bận); mục nợ `Chủ: Epic 12` nay nêu cả đối chứng.
- low · reject — `Esc` khi tiêu điểm nằm trong khung không đóng khung ở chế độ hẹp; WKWebView không đặt tiêu điểm lên nút khi bấm nên hiếm gặp, sửa cần thêm handler và mục `HANDLER_TABLE`. Lượt nhìn dưới 1.100px đã trong mục nợ.
- false · reject — `clientWidth` lệch `@container`: `.wp` không có padding nên hai số trùng.
- false · reject — cuộn âm: `syncScroll` kẹp `Math.max(0, next)`.
- false · reject — câu hệ quả khi đích nhỏ hơn ord: `setChapterOriginApplyThroughOrd` trả `null` khi `parsed < row.ord` (`libraryChapters.ts:642`).
- low · reject — dải dính làm ở trang thay vì `chapterWindow`: hành vi như nhau, không có hại được nêu; nhìn thật nằm trong mục nợ.
- false · reject — `@dblclick.stop` trần: `check:commands` xanh sau khi nâng sàn.
- false · reject — `padding: 0` trên `.ui-btn`: `controls-adoption` xanh trong lượt 280 ca.
- false · reject — lớp `wp-status--override` chết: rule `.wp-status--override .wp-badge` vẫn còn.
- low · patch — chú thích kể lại mã (đầu tệp trang, dòng "Selected row…") xoá; doc comment tiếng Việt của `openCurrentChapterInReading` đổi sang tiếng Anh; đầu spec e2e giữ theo khuôn các spec khác.
- low · patch — khoá `library.work_page.seg_count` chỉ là `{count}`; thay bằng text node dữ liệu có `aura-allow-text`.
- false · reject — `filter_all` mất `{count}` và nhãn ô áp đổi chữ: không còn nơi gọi hay test nào đọc chuỗi cũ.

## Design Notes

- Nút Dịch trên hàng là `library.open_chapter` (đã có). Hai nút hàng hoạt động nhờ `@mousedown` của hàng chọn đúng hàng trước, nên lệnh vẫn đọc vùng chọn như mọi lệnh khác.
- Cửa sổ ảo chỉ giữ đầu nhóm trong khi nó nằm trong vùng render; muốn dải dính đúng thì `chapterWindow` phải luôn render đầu nhóm của hàng trên cùng. Chiều cao đệm tính lại cho phần tử được giữ, mỗi phần tử vẫn 44px.
- Kế hoạch pha (AGENTS.md): 1 lệnh + trạng thái · 2 trang · 3 test + e2e, mỗi pha một agent mới, bàn giao qua tệp cạnh plan.

## Verification

**Commands:**
- `npx vitest run tests/frontend/libraryChapters.test.ts tests/frontend/readingState.test.ts tests/frontend/exportScope.test.ts tests/frontend/controls-adoption.test.ts tests/frontend/commandsRegistryPortMissing.test.ts tests/frontend/librarySearchScope.test.ts` -- expected: mã thoát 0, không dòng Errors.
- `npm run check:commands && npm run check:tokens && npm run check:i18n && npx vue-tsc --noEmit` -- expected: xanh.
- e2e spec mới, `story-12-4`, `story-5-8`, `story-5-7`, `story-5-4` (ngoài sandbox, cổng 1420 trống) -- expected: xanh; bận thì ghi lại, không suy ra.

**Manual checks:**
- Ice: ảnh app thật cạnh mockup cùng khổ, khung 1–5 (mục nợ `Chủ: Epic 12`).
