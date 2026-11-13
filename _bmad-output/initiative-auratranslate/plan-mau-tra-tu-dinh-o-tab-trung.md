---
title: 'Màu tra từ dính ở tab Trung: tô nhịp tra bằng thuộc tính trên mảnh nguyên văn thay cho Custom Highlight'
type: 'bugfix'
ticket: ''
created: '2026-11-13'
status: 'built'
baseline_revision: 'ffcf4073699d3de1d92b9c95d5fb8b3f416b28aa'
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

**Problem:** Ice báo (2026-11-13, ảnh ô nguyên văn tab Trung): bấm tra từ ở hai chỗ thì đôi khi cả hai chỗ cùng mang màu `lookup-active` (群體 và 追蹤). `paintLookupRanges` luôn gỡ highlight cũ trước khi đặt cái mới, nên sổ `CSS.highlights` đúng; màu cũ còn là do WebKit không vẽ lại vùng cũ sau `CSS.highlights.set`. Plan `plan-tra-cuu-bam-mot-lan-o-han-viet.md` đã đo lỗi này trên Safari 27.2 (2/6 lượt nạp giữ màu cũ) và chuyển hai kiểu Hán Việt sang thuộc tính; tab Trung được giữ vì lượt đo lúc đó ra 0/4.

**Approach:** Tab Trung bỏ Custom Highlight. Nhịp đang tra trở thành một trạng thái của lưới (`segmentId`, `start`, `end`); hai biên của nhịp được thêm vào tập biên chia `.src-piece`, và mảnh nằm trong nhịp mang thuộc tính `data-lookup-active` do Vue vẽ, kèm CSS màu `var(--color-lookup-active)`. Cả lưới chỉ còn một cơ chế tô là thuộc tính `data-lookup-active`.

## Boundaries & Constraints

**Always:**
- Toàn lưới chỉ có đúng một nhịp được tô. Tô nhịp mới thì gỡ nhịp cũ, dù nhịp cũ ở tab Trung hay ở một khối Hán Việt. Mọi chỗ đang gọi `clearPaintedLookup` (đổi Chương, đổi tab hoặc kiểu xem, đổi Glossary, `currentQuery` đổi, unmount) gỡ được cả hai loại dấu.
- Màu tô thắng `.src-piece.glossary-confirmed`. Dấu thuật ngữ (`glossary-pending`), nền Concordance (`concordance-source`) và dấu cắt `cut-mark` giữ nguyên.
- Lượt tra vẫn đi đúng một đường `offerPointLookup(text)`, truy vấn không đổi.
- Biên do nhịp tra sinh ra không phải điểm cắt: `isPendingCut` vẫn chỉ đúng với điểm cắt người dùng, và không vẽ `cut-mark` ở biên nhịp.

**Never:**
- Không giữ đường Custom Highlight hay đường dự phòng `selection` cho tab Trung.
- Không đổi đường tô của hai kiểu Hán Việt (`paintLookupBlocks`).
- Không sửa DOM ô nguyên văn ngoài Vue (không tự chẻ text node).

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected Output / Behavior | Error Handling |
|----------|--------------|---------------------------|----------------|
| Bấm A rồi B cùng câu | Hai từ khác nhau trong một ô | Chỉ các mảnh của B mang `data-lookup-active` | Không |
| Bấm A rồi B khác câu | Hai ô khác nhau | Ô A không còn mảnh nào mang dấu | Không |
| Bấm thuật ngữ Glossary đã xác nhận | Nhịp = cả thuật ngữ | Mảnh mang dấu, màu `lookup-active` thắng màu `primary` | Không |
| Từ nằm cạnh điểm cắt đang chờ | Ô có `cut-mark` | `cut-mark` giữ đúng chỗ, không thêm `cut-mark` ở biên nhịp | Không |
| Đổi sang tab Hán Việt sau khi bấm | Trung → Hán Việt | Không còn dấu nào | Không |

</frozen-after-approval>

## Implementation Notes

- Bàn đo Playwright WebKit 2370 không tái hiện được lỗi vẽ lại: headless ra 0/6 vì ảnh chụp vẽ lại cả trang, còn cửa sổ có giao diện thì bị cửa sổ khác che nên `screencapture` không chụp được. Nghiệm thu lỗi vẽ lại vẫn là việc của Ice, làm bằng mắt.
- Nhịp tra là một `shallowRef<LookupSpan>`. Mảnh được chẻ theo hai biên của nó, rồi Vue vẽ `data-lookup-active`. `paintLookup` và `paintLookupBlocksFor` đều gọi `clearPaintedLookup` trước, nên toàn lưới chỉ còn một dấu.
- Đối chứng đỏ đã làm thật: gỡ `lookupSpanOf(s.id)` khỏi `v-memo` làm đỏ 4/6 ca `gridLookupPaint`. Ngay lượt bấm đầu dấu đã không hiện, vì hàng không vẽ lại. Gỡ `lookupSpan.value = null` khỏi `clearPaintedLookup` thì ca `currentQuery` đỏ, còn ca A rồi B vẫn xanh vì `paintLookup` ghi đè trạng thái. Câu AC thứ nhất vì thế chỉ đúng với dòng `v-memo`.
- e2e `lookup-by-click` chạy cả spec thì đỏ 12/13 ở fixture ("Đã gọi 5 hàm reset mà lưới còn 1 hàng nguyên văn"), vì Vite của `tauri dev` đang mở có tem `?t=`. Chạy từng ca bằng `--mochaOpts.grep` thì 13/13 xanh trong WKWebView 605.1.15, kể cả ca mới "a click on a second word paints only that word".

## Review Triage Log

- Lượt 1 (quick): low 1, false 3.
  - false: "đối chứng đỏ không có trong diff". Đã gỡ thật cả hai dòng (xem Implementation Notes), và cả hai đều có ca canh.
  - false: "thiếu ca chuyển từ khối Hán Việt sang mảnh tab Trung". Hai cơ chế không cùng có mặt trong một kiểu xem, vì `showHanViet` là cờ toàn lưới, và đổi tab đã gỡ dấu qua `watch` (có ca riêng).
  - low, patch: kiểu `{ segmentId, start, end }` viết inline ba lần. Đã thay bằng `LookupSpan` xuất từ `gridBlockContext.ts`; vitest 59/59, vue-tsc, check:lint, check:tokens đều xanh.
  - false: "`paintLookupBlocksFor` gỡ dấu hai lần". Không có hậu quả xấu, và `paintLookupBlocks` tự gỡ là hợp đồng mà ca "tô A rồi B" của nó canh.

## Code Map

- `src/panels/GridPanel.vue:486-494` -- `sourcePieceStartsOf`: tập biên mảnh (điểm cắt + biên thuật ngữ). Thêm hai biên của nhịp tra khi nhịp thuộc `segmentId` này.
- `src/panels/GridPanel.vue:542-559` -- `sourcePieceInfoOf`, `isConcordanceHighlighted`: khuôn để thêm cờ `lookup` cho mảnh.
- `src/panels/GridPanel.vue:950-1013` -- `paintedLookup`, `clearPaintedLookup`, `paintLookup`, `lookupByPlainClick`, hai `watch` gỡ dấu. Nhánh `else paintLookup(ranges, text)` thay bằng đặt trạng thái nhịp. `span` (code point) đã có sẵn ở đây.
- `src/panels/GridBlock.vue:72-102` -- `concordanceHighlightOf` + `defineExpose` (vue-tsc không tính biểu thức `v-memo` là lượt đọc). Thêm hàm tương tự cho nhịp tra và đưa vào `v-memo` ở `:160-171`, nếu thiếu thì hàng không vẽ lại.
- `src/panels/GridBlock.vue:247-256` -- `.src-piece`; thêm `:data-lookup-active`. CSS ở `:608-616`.
- `src/panels/gridBlockContext.ts` -- `SourcePieceInfo` thêm `lookup: boolean`; context thêm hàm đọc nhịp theo hàng nếu GridBlock cần.
- `src/panels/lookupPaint.ts` -- bỏ `paintLookupRanges`, `highlightRegistry`, `LOOKUP_HIGHLIGHT`; giữ `paintLookupBlocks`, `clearLookupPaint`.
- `src/tokens/controls.css:766-768` -- bỏ `::highlight(lookup-active)`.
- `tests/frontend/lookupByClick.test.ts:409-450` -- khối `describe('lookupPaint')` kiểm `paintLookupRanges`; bỏ cùng hàm. `sourceRangesForSpan` vẫn dùng cho `lookupSpanAtPoint`, giữ ca của nó.
- `e2e/specs/lookup-by-click.e2e.mjs:21-36` -- `paintedText()` đọc `[data-lookup-active]` rồi `CSS.highlights`; bỏ vế `CSS.highlights`.
- Không đụng: `editorSegments.ts` (`lookupSpanAtPoint`, `sourceRangesForSpan`, `atomicBlocksForSpan`), `SourceHanViet.vue`, `selectionContract.ts`.

## Tasks & Acceptance

**Execution:**
- [ ] `tests/frontend/` -- viết trước, đỏ trước: mount lưới ở tab Trung (theo khuôn test lưới có sẵn), bấm A rồi B, rồi khẳng định chỉ các mảnh của B mang `data-lookup-active` và ghép chữ của chúng ra đúng B; thêm ca thuật ngữ đã xác nhận và ca `cut-mark`.
- [ ] `src/panels/GridPanel.vue` -- trạng thái nhịp tra; biên mảnh; cờ `lookup`; `clearPaintedLookup` gỡ cả trạng thái lẫn khối.
- [ ] `src/panels/GridBlock.vue`, `src/panels/gridBlockContext.ts` -- thuộc tính trên mảnh, `v-memo`, CSS.
- [ ] `src/panels/lookupPaint.ts`, `src/tokens/controls.css`, `tests/frontend/lookupByClick.test.ts` -- bỏ đường Custom Highlight và ca của nó.
- [ ] `e2e/specs/lookup-by-click.e2e.mjs` -- `paintedText()` chỉ đọc thuộc tính; ca tab Trung bấm A rồi B khẳng định chỉ B được tô.

**Acceptance Criteria:**
- Given ca vitest "A rồi B" xanh, when gỡ thật dòng đưa nhịp tra vào `v-memo` hoặc dòng gỡ trạng thái ở `clearPaintedLookup`, then ca đó đỏ vì A còn mang dấu.
- Given `grep -rn "CSS.highlights\|::highlight" src`, then không còn kết quả.

## Design Notes

Phương án bị loại: giữ Custom Highlight và ép WebKit vẽ lại (giữ một `Highlight` rồi `clear()`/`add()`, hoặc đặt lại ở `requestAnimationFrame`). Bàn đo Playwright WebKit 2370 headless với mã hiện tại ra 0/6 lượt nạp giữ màu cũ, vì ảnh chụp headless vẽ lại cả trang. Chụp màn hình thật cũng không dùng được vì cửa sổ WebKit bị trình duyệt của Ice che. Vậy cách ép vẽ lại không có số đo nào đỡ, trong khi cách đổi thuộc tính đã đo trên Safari: 0/3 lượt giữ màu cũ.

Thuộc tính do Vue vẽ (không như `paintLookupBlocks`, vốn đặt ngoài vnode) vì mảnh `.src-piece` do `v-for` sinh: chẻ mảnh làm Vue dựng lại text node, nên một thuộc tính đặt tay sẽ mất hoặc nằm sai mảnh.

## Verification

**Commands:**
- `npx vitest run tests/frontend/lookupByClick.test.ts` và tệp test mới -- expected: xanh
- `npm run check:tokens && npm run check:lint && npx vue-tsc --noEmit` -- expected: xanh
- `npm run test:e2e -- --spec e2e/specs/lookup-by-click.e2e.mjs` (ngoài sandbox, binary `--features wdio`, chạy từng ca bằng `--mochaOpts.grep` nếu Vite cũ có tem `?t=`) -- expected: mọi ca xanh

**Manual checks:**
- e2e đọc DOM chứ không đọc điểm ảnh, nên lỗi vẽ lại chỉ nghiệm thu được bằng mắt: Ice bấm liên tiếp nhiều từ ở tab Trung trong `npm run tauri dev`.
