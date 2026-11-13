---
title: 'Bấm một lần để tra ở ô nguyên văn Hán Việt: tô bằng thuộc tính thay cho Custom Highlight, bấm trúng cả dòng âm đọc ở kiểu Song song'
type: 'bugfix'
ticket: ''
created: '2026-11-13'
status: 'built'
baseline_revision: '67a3445c5184c441d7292ed9e0433770812d2f35'
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

**Problem:** Ice báo (2026-11-13, ảnh lưới Workspace tab Hán Việt). (1) Ở kiểu Chuyển đổi, bấm sang từ khác thì từ cũ vẫn giữ màu `lookup-active`. (2) Ở kiểu Song song, chữ rất khó bấm trúng, và từ được tra không đổi màu. Đo trên Safari 27.2 (macOS 15.8.2, cùng WebKit với WKWebView) và Playwright WebKit 2370, bàn đo gọi đúng `lookupSpanAtPoint`/`sourceRangesForSpan`/`paintLookupRanges`:
- `CSS.highlights.set` thay highlight mà không vẽ lại vùng cũ ở 2/6 lượt nạp trang Safari (cả hai lượt giữ màu mọi từ đã bấm). Sổ highlight vẫn đúng, nên e2e đọc sổ không bắt được.
- WebKit không vẽ `::highlight` lên chữ gốc trong `<ruby>`: 0 điểm ảnh cam ở cả hai bản WebKit, trong khi Chromium vẽ 261–699 điểm ảnh.
- `.hv-unit` có `getClientRects()` chỉ phủ dòng chữ Hán, còn `<rt>` nằm dưới, nên `hits(anchor)` trượt ở mọi cú bấm vào âm đọc (WebKit 0/1000; chữ Hán 800/800).

**Approach:** Hai kiểu Hán Việt tô từ đang tra bằng một thuộc tính đặt lên các khối nguyên tử (`[data-src-atomic]`) mà nhịp tra phủ tới, kèm CSS `color` lấy từ token `lookup-active`. Bàn đo: 0/3 lượt giữ màu cũ, và ở kiểu Song song tô đúng chữ Hán, không tô âm đọc. Ở kiểu Song song, cú bấm trên `<rt>` của khối cũng tính là trúng khối đó. Tab Trung giữ Custom Highlight: Ice chọn phương án A ngày 2026-11-13, trong lượt đo tab Trung giữ màu cũ 0/4 lần. Ice cũng chọn giữ nguyên plan dù chưa đếm được token.

## Boundaries & Constraints

**Always:**
- Mỗi lúc chỉ có đúng một nhịp được tô trên toàn lưới. Tô nhịp mới thì gỡ nhịp cũ. Mọi chỗ đang gọi `clearPaintedLookup` (đổi Chương, đổi tab/kiểu xem, đổi Glossary, `currentQuery` đổi, unmount) gỡ được cả thuộc tính lẫn highlight.
- Màu chỉ lấy từ `var(--color-lookup-active)`; âm đọc `<rt>` giữ màu `on-surface-variant` của nó.
- Lượt tra vẫn đi đúng một đường: `offerPointLookup(text)`. Truy vấn và nhịp tra không đổi.

**Never:**
- Không đổi đường tô của tab Trung, của Concordance (`concordance-source`), và đường dự phòng `selection` khi engine thiếu API.
- Không thêm phần tử con vào `.hv-switch`/`.hv-parallel`: `resolveSwitch` ánh xạ theo `children[i]`.
- Không dùng tên lớp (`class`) cho dấu tô: Vue vá `class` theo vnode; thuộc tính ngoài vnode thì Vue không đụng tới.

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected Output / Behavior | Error Handling |
|----------|--------------|---------------------------|----------------|
| Chuyển đổi, bấm A rồi B | Hai khối `.hv-word` khác nhau | Chỉ B mang dấu tô; Panel Lookup hiện chữ Hán của B | Không |
| Song song, bấm âm đọc | Cú bấm giữa `<rt>` của một `.hv-unit` | Tra chữ Hán của khối đó; chữ Hán đổi màu, âm đọc không | Không |
| Song song, bấm chữ Hán | Cú bấm lên base `<ruby>` | Như trên | Không |
| Thuật ngữ Glossary phủ nhiều khối | Bấm một khối trong thuật ngữ | Mọi khối của thuật ngữ mang dấu tô | Không |
| Bấm dấu câu / chỗ trống | `.hv-text` hoặc khoảng ngoài khối | Giữ nguyên nhịp và dấu tô cũ | Không |
| Đổi tab hoặc kiểu xem sau khi bấm | Trung ↔ Hán Việt, Chuyển đổi ↔ Song song | Không còn dấu tô nào | Không |

</frozen-after-approval>

## Code Map

- `src/panels/GridPanel.vue:948-1013` -- `paintedLookup`, `clearPaintedLookup`, `paintLookup`, `pointerOver`, `lookupByPlainClick`, và hai `watch` gỡ dấu tô. Chỗ rẽ: dùng khối hay dùng range.
- `src/panels/editorSegments.ts:366-471` -- `SRC_ATOMIC`, `HAN_BLOCK_SELECTOR`, `anchorLength`, `sourceRangesForSpan` (giữ cho tab Trung), `lookupSpanAtPoint` (nhánh nguyên tử `:454-457` chỉ thử `ctx.hits(anchor)`).
- `src/panels/lookupPaint.ts` -- `clearLookupPaint`, `paintLookupRanges`. Thêm đường tô khối ở đây, cùng nơi gỡ.
- `src/panels/SourceHanViet.vue:842-940` (template), `:966-975` (`.glossary-confirmed` dùng `color: primary`; dấu tô phải thắng nó về độ ưu tiên selector), `rt { color }` ở cuối `<style scoped>`.
- `tests/frontend/lookupByClick.test.ts:272-389` -- `mountCell`, `ctx`, `textOf`; khối `lookupSpanAtPoint, Han Viet views`, `sourceRangesForSpan`, `lookupPaint`. Ca `parallel: a click on the reading…` (`:301`) xanh chỉ vì `hits` mặc định trả `true`.
- `e2e/specs/lookup-by-click.e2e.mjs` -- `paintedText()` đọc `CSS.highlights`; ca switch `:113`, parallel `:141`, đổi kiểu xem `:159`.
- Không đụng: `selectionContract.ts`, `hanVietSurfaces.ts`, `resolveSwitch`/`resolveParallel`.

## Tasks & Acceptance

**Execution:**
- [ ] `tests/frontend/lookupByClick.test.ts` -- viết trước, đỏ trước: (a) hàm lấy khối cho một nhịp trả đúng các `[data-src-atomic]` giao nhịp, cả `.hv-word`, `.hv-unit`, `.hv-text`; (b) tô khối A rồi khối B thì chỉ còn B mang thuộc tính, và `clearLookupPaint` gỡ hết; (c) Song song với `hits` chỉ trả `true` cho `<rt>` vẫn ra nhịp của khối -- khoá lỗi hình học WebKit ở mức hàm.
- [ ] `src/panels/editorSegments.ts` -- xuất hàm lấy khối nguyên tử theo nhịp (dùng lại `anchorLength`). Nhánh nguyên tử của `lookupSpanAtPoint` coi cú bấm là trúng khi con trỏ nằm trên khối hoặc trên `<rt>` của nó.
- [ ] `src/panels/lookupPaint.ts` -- thêm hàm tô khối bằng thuộc tính; mọi lượt tô (khối hay range) và `clearLookupPaint` đều gỡ dấu tô của lượt trước, ở cả hai cơ chế.
- [ ] `src/panels/GridPanel.vue` -- trong `lookupByPlainClick`: ô có khối nguyên tử thì tô khối; không có thì giữ đường range như cũ.
- [ ] `src/panels/SourceHanViet.vue` -- CSS cho thuộc tính tô, màu `var(--color-lookup-active)`, thắng được `.glossary-confirmed`.
- [ ] `e2e/specs/lookup-by-click.e2e.mjs` -- `paintedText()` đọc cả khối mang thuộc tính (chữ, bỏ `<rt>`); thêm hai ca: Chuyển đổi bấm A rồi B thì chỉ B được tô; Song song bấm vào giữa `<rt>` thì tra và tô chữ Hán. Ca hai đọc màu thật bằng `getComputedStyle` của base so với `<rt>`.

**Acceptance Criteria:**
- Given lưới ở tab Hán Việt, when gỡ phép thử `<rt>` khỏi `lookupSpanAtPoint`, then ca Song song bấm âm đọc (vitest và e2e) đỏ vì không có nhịp. Phép gỡ này là đối chứng đỏ thật.
- Given lưới ở tab Trung, when bấm một chữ, then hành vi giữ nguyên: các ca hiện có của `lookup-by-click.e2e.mjs` vẫn xanh.

## Design Notes

Dấu tô là thuộc tính ngoài vnode, nên khi Vue dựng lại khối thì dấu mất cùng khối. Mọi lần dựng lại đều đến từ những nguồn mà `watch` gỡ dấu tô đã theo dõi (`editorSegments`, Glossary, `showHanViet`, `viewMode`, Chương). Giữ danh sách phần tử đã tô trong `lookupPaint.ts` thay vì quét `document`, để không thêm thành viên `document` mới (`check:layout` Kiểm C).

## Verification

**Commands:**
- `npx vitest run tests/frontend/lookupByClick.test.ts` -- expected: xanh, kể cả ba ca mới
- `npm run check:tokens && npm run check:lint` -- expected: xanh
- `npm run test:e2e -- --spec e2e/specs/lookup-by-click.e2e.mjs` (ngoài sandbox, binary `--features wdio`) -- expected: mọi ca xanh

**Manual checks:**
- e2e đọc trạng thái DOM, không đọc điểm ảnh, nên lỗi vẽ lại chỉ nghiệm thu được bằng mắt: Ice bấm liên tiếp nhiều từ ở cả hai kiểu Hán Việt trong `npm run tauri dev`.

## Implementation Notes

- Bản đầu nhận cú bấm có caret trong `<rt>` mà không hỏi `hits`; `caretRangeFromPoint` bám chữ gần nhất nên chỗ trống cũng tra được. Đã đổi thành: con trỏ phải nằm trên khối hoặc trên `<rt>` của khối.
- Đối chứng đỏ đã làm thật: gỡ phép thử `<rt>` làm đỏ ca vitest "only the reading is hit" và ca e2e "middle of the reading" (headword rỗng). Gỡ `clearLookupPaint()` trong `paintLookupBlocks` làm đỏ ca tô A rồi B.
- e2e chạy cả spec đỏ 9/10 ở fixture ("Đã gọi 5 hàm reset mà lưới còn 1 hàng nguyên văn") vì Vite của `tauri dev` mở từ 09:41 phục vụ module có tem `?t=`. Chạy từng ca bằng `--mochaOpts.grep` thì 10/10 xanh trong WKWebView 605.1.15.
- e2e đọc DOM, không đọc điểm ảnh. Lỗi vẽ lại chỉ thấy bằng mắt; bàn đo Safari đã cho thấy cách tô bằng thuộc tính không giữ màu cũ (0/3 lượt).

## Review Triage Log

- Lượt 1 (quick): low 1, false 1.
  - low, patch: `timeoutMsg` của ca Song song mới được tính lúc gọi nên in headword cũ. Đã thay bằng chuỗi tĩnh; ca chạy lại xanh.
  - false: "hai khối đầu có thể nằm chung một thuật ngữ Glossary". Mỗi spec chạy với app và dữ liệu mới, không có Glossary; `他拉小提琴。` cho hai khối đầu là `他` và `拉`, và ca đã xanh.
