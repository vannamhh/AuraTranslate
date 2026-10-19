---
title: 'Dải gợi ý mọc thành track hàng dưới câu đang sửa'
type: 'feature'
ticket: '9'
created: '2026-10-14'
status: done
baseline_revision: 'de7c67e4fc4a7c25370549d0d02686c1b228239d'
route: 'full'
route_source: 'auto'
risk: 'high'
review: 'quick'
review_source: 'pinned'
lenses_ran: ['quick']
review_loop_iteration: 1
context:
  - '{project-root}/_bmad-output/initiative-auratranslate/ux-auratranslate/DESIGN.md'
---

<frozen-after-approval reason="human-owned intent — do not modify unless human renegotiates">

## Intent

**Problem:** Dải TM, chốt Glossary và Proofreader đang nằm chung một slot toàn bề ngang ngay trên thanh trạng thái (`App.vue:312-330`), xa câu chúng nói tới; mockup và EXPERIENCE.md §Component Patterns "Dải mọc dưới câu đang sửa" (🔵 D8) đặt chúng ngay dưới hàng đang sửa, đẩy văn bản xuống chứ không phủ lên (D8, đề xuất 5; dòng b:116, b:133, c:110, c:111).

**Approach:** Lưới `subgrid` của `GridPanel.vue` thêm một track hàng ngay sau hàng neo khi có dải để hiện; ba component dải chuyển vào track đó, giữ nguyên `topmostStrip`, `PRIORITY_ORDER` và hành vi bàn phím của từng dải. `GlossaryQuickAdd` ở lại chân cửa sổ. Đo thời gian frame khi dải mọc/thu trên Chương 2.000 câu và ghi số.

## Decisions (Ice, 2026-10-14)

1. Track chỉ chiếm cột Bản dịch (`grid-column: 4`), như `tm-fuzzy-match.html:50-52` và `proofreader.html:56-64`; ô nguyên văn bên trái track để trống.
2. Dải tóm tắt Proofreader ẩn khi không có câu đang sửa; số phát hiện theo loại vẫn ở thanh trạng thái (12.7).
3. Dải mọc dưới mép khung nhìn thì không cuộn; `focus()` của phím vào dải tự cuộn.
4. Giữ một plan dù quá 1.600 token: phép đo frame là Verify của chính entry.
5. Phần cấp lượt quét của dải Proofreader (đang quét, lỗi, chưa cấu hình, quét xong không câu nào; kèm nút Huỷ) không gắn câu nào nên luôn ở chân cửa sổ cạnh Quick Add, ở mọi chế độ, cùng lý do D8 để Quick Add ở chân. Khi phần này hiện thì không track nào mọc (một dải một lúc). Phát hiện của câu đang sửa, hoàn tác và lỗi nhận đề xuất, cùng dải tóm tắt vào track.
6. Ô bản dịch của câu neo trỏ `aria-details` tới dải; DOM giữ dải sau bốn cột.

## Boundaries & Constraints

**Always:**
- Một dải tại một thời điểm, chọn bằng `topmostStrip(eligibleInlineStrips)`; khi `glossary_quick_add` thắng thì không track nào mọc (hành vi ưu tiên hiện có giữ nguyên).
- Hàng neo: dải TM neo ở `tmFuzzySegmentId` (đi sau con trỏ 150 ms); chốt Glossary và phần câu của Proofreader neo ở `editorCaretSegmentId`; phần cấp lượt quét của Proofreader không neo (Quyết định 5).
- Tiêu điểm chuyển vào dải (`Mod+Alt+E`, `Mod+Alt+C`, Tab) không làm mất câu đang sửa: `onColumnFocusOut` không đặt con trỏ về `null` khi `relatedTarget` nằm trong dải.
- Dải nằm ngoài `colSrc`/`colTgt`: lưới vẫn đúng hai bề mặt chọn (`check:commands` ⑤), và đường ghi vẫn đọc `textContent` của ô không lẫn chữ dải.
- Chuyển động theo DESIGN.md §Motion: dải vào 90 ms, opacity 0.4→1, không translate/scale, không hiệu ứng thoát; `prefers-reduced-motion` thì không hiệu ứng. CSS thuần (không `matchMedia`, Kiểm C của `check:layout`), theo mẫu `LookupPanel.vue:975-1051`.
- Màu, cỡ chữ qua token; chữ mới qua `vi.json`.

**Never:**
- Không đổi nội dung dải (Story 12.10), không đổi `PRIORITY_ORDER`, không thêm lệnh hay phím mới.
- Không thêm hàm cuộn vào luồng đặt con trỏ, không `preventScroll` (Story 2.10, `GridPanel.vue:1263-1300`).
- Không ảo hoá lưới trong story này (spine §Deferred); không dependency mới.
- Không sửa `ReadingMode.vue`.

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected Output / Behavior | Error Handling |
|----------|--------------|---------------------------|----------------|
| TM mọc | Con trỏ ở câu k có khớp mờ ≥ 65% | Track mới ngay sau hàng k chứa dải TM; hàng k+1 trở đi dời xuống một track | — |
| Thu | Esc trong dải, hoặc con trỏ sang câu không khớp | Track biến mất, lưới về N track; không hiệu ứng thoát | — |
| Ưu tiên | TM và chốt Glossary cùng đủ điều kiện ở câu k | Chỉ dải chốt Glossary mọc, cùng chỗ | — |
| Quick Add mở | `quickAddIsOpen` | Quick Add ở chân cửa sổ; không track nào trong lưới | — |
| Vào dải bằng phím | `Mod+Alt+E` khi dải TM đang hiện | Tiêu điểm vào dải; câu k vẫn là câu đang sửa; dải không biến mất | — |
| Đổi Chương / Tác phẩm | Dải đang mở | Track biến mất cùng lưới cũ | — |
| Tóm tắt Proofreader, không câu đang sửa | Lượt quét xong, con trỏ `null` | Không track nào; số phát hiện ở thanh trạng thái | — |
| Hàng neo ở mép dưới khung nhìn | Con trỏ ở hàng cuối nhìn thấy, TM khớp | Track mọc, khung nhìn không cuộn | — |
| Đang quét, có câu đang sửa | `scanning`, con trỏ ở câu k | Dải Proofreader (tiến độ, Huỷ) ở chân cửa sổ; không track nào trong lưới | — |
| Job không hàng neo | `error`/`not_configured`/`scanning`, con trỏ `null` hoặc đang ở Library/Chế độ đọc | Dải Proofreader ở chân cửa sổ, nhìn thấy, Huỷ bấm được | — |
| Phát hiện sau lượt quét | `done`, con trỏ ở câu k có phát hiện | Dải Proofreader trong track dưới câu k; chân cửa sổ không có dải | — |
| Trình đọc màn hình | Dải đang hiện dưới câu k | Ô bản dịch câu k có `aria-details` = id của dải; ô khác không có | — |

</frozen-after-approval>

## Code Map

- `src/App.vue:312-330` -- slot chung hiện tại: `GlossaryQuickAdd`, `GlossaryConfirmStrip`, `ProofreaderStrip`, `TmFuzzyStrip` ngay trên `StatusBar`; import :44-55. Quick Add ở lại.
- `src/panels/inlineStripPriority.ts` -- `PRIORITY_ORDER`, `topmostStrip`; giữ nguyên. `src/inlineStripEligibility.ts` -- `eligibleInlineStrips`; chỗ đặt computed hàng neo mới.
- `src/TmFuzzyStrip.vue` (`tabindex=-1` root, `onKeydown`), `src/tmFuzzyStripState.ts` (`tmFuzzySegmentId` :35, `syncTmFuzzyStrip` :108 xoá dải khi con trỏ `null`, `focusTmFuzzyStrip` :144).
- `src/GlossaryConfirmStrip.vue` (tự lọc `eligible` :55-62, focus qua `confirmStripFocusRequest` :89-91), `src/glossaryConfirmStripState.ts` (`CurrentTarget` không có `segmentId`, suy từ con trỏ).
- `src/ProofreaderStrip.vue` (không watcher, chỉ render và `dispatch`, nên mount hai thể hiện an toàn; hiện khi top là `proofreader`/`proofreader_summary` :28; phần cấp lượt quét là các nhánh `scanning`/`error`/`not_configured`/`done` với `total === 0` của `hasJob` ở `inlineStripEligibility.ts:24-31`, phần câu là `proofreadCurrent`/`proofreadRevertible`/`proofreadActionError`; CSS `flex: none` + vạch trái :238-249 viết cho slot chân cửa sổ), `src/proofreadState.ts` (`proofreadCurrent` :315 theo con trỏ).
- `src/panels/GridPanel.vue` (2622 dòng, đọc theo offset) -- `.grid` :1783 `gridTemplateRows: repeat(N, auto)`; CSS cột :2111-2131 (bốn cột `3px 30px 1fr 1fr`, mỗi `.col` `grid-row: 1 / -1; subgrid`); bốn `v-for="s in editorSegments"` :1790/1807/1817/1925, ô tự xếp chỗ, không có phần tử hàng; không ảo hoá (mọi hàng render); `onColumnFocusOut` :1027-1031; luồng con trỏ/cuộn :1240-1300 không đụng.
- Cách chèn track: `repeat(N+1, auto)` khi có hàng neo; mỗi cột thêm một ô đệm `aria-hidden` ngay sau ô của hàng neo để ô sau tự dời một track; dải là con trực tiếp của `.grid`, `grid-row: k+2`. Không gán `grid-row` cho từng ô (4×N binding đổi mỗi lần di con trỏ).
- Gates: `scripts/check-commands.mjs` `HANDLER_TABLE` :2390 (handler mới trong `GridPanel.vue` cần dòng), ⑤ hai bề mặt chọn :1915; `scripts/check-tokens.mjs` B/D/F/H (`outline: none` chỉ ở root `tabindex=-1`); `scripts/check-panel-refs.mjs` (trạng thái module mới cần hàm reset).
- e2e: `openWorkspaceWithWork(name, text)` (`e2e/support/workspace.mjs:59`, `sourceLang:'zh'`, tách theo `。`); Chương lớn nhất hiện có 40 câu (`segment-navigation.e2e.mjs:61-63`, cảnh báo trần 120 s `wdio.conf.mjs:600` lúc tạo Work); chạy một spec `npm run test:e2e -- --spec e2e/specs/<tệp>`. Không có công cụ đo frame sẵn; số đo WKWebView cũ ở `deferred-work.md:2993-3006` (2.000 câu, di con trỏ 161–226 ms).
- Tests of this story: `tests/frontend/stripAnchorSegment.test.ts`, `tests/frontend/gridPanelStripTrack.test.ts`, `tests/frontend/proofreaderStripSlots.test.ts`, `tests/frontend/proofreadWiring.test.ts`, `e2e/specs/story-12-9-strip-row-track.e2e.mjs`.
- Tests that move: `tests/frontend/{proofreaderStripSlots,proofreaderStrip,tmFuzzyStrip,glossaryConfirmStripTemplate,glossaryQuickAddStrip,proofreadMemory,inlineStripPriority}.test.ts`; khoảng 20 tệp mount `GridPanel` (`gridPanelTmFilled`, `editorConfirmSegment`, `segmentSelection`, …) sẽ mount cả ba dải; e2e `glossary-quick-add`, `story-5-13-reading-marks`, `segment-navigation`, `hanviet-parallel-row-height`, `grid-image-row-alignment`.

## Tasks & Acceptance

**Execution:**
- [ ] `src/inlineStripEligibility.ts` -- computed chỗ của dải Proofreader (`foot` khi phần cấp lượt quét có mặt, `track` khi chỉ phần câu hoặc tóm tắt, theo `topmostStrip`); computed hàng neo: id câu theo kiểu dải thắng, `null` khi không dải nào, Quick Add thắng, Proofreader ở `foot`, hoặc tóm tắt thắng lúc con trỏ `null` -- Always 1–2, Quyết định 2, 5.
- [ ] `src/panels/GridPanel.vue` -- track N+1, ô đệm bốn cột, khối dải mount ba component, chặn `onColumnFocusOut` cho dải, CSS 90 ms + reduced-motion, `grid-column: 4` -- D8, Quyết định 1–3.
- [ ] `src/App.vue`, `src/{TmFuzzyStrip,GlossaryConfirmStrip,ProofreaderStrip}.vue` -- gỡ dải TM và chốt Glossary khỏi slot chân cửa sổ; `ProofreaderStrip` nhận prop chỗ đặt và chỉ render khi khớp computed chỗ, một thể hiện ở chân (`App.vue`, sau Quick Add) và một trong track; sửa CSS root viết cho slot chân -- b:116, b:133, c:110, Quyết định 5.
- [ ] `scripts/check-commands.mjs` -- thêm dòng `HANDLER_TABLE` nếu có handler mới.
- [ ] Tests (vitest) -- hàng neo theo bảng I/O (gồm Quick Add, TM neo `tmFuzzySegmentId`); lưới có đúng N+1 track và dải đứng sau ô hàng k ở cả bốn cột; vào dải giữ con trỏ (đối chứng đỏ: gỡ thật nhánh chặn trong `onColumnFocusOut`); chỗ dải Proofreader theo bốn hàng I/O mới (đối chứng đỏ: gỡ thật nhánh `foot`); `aria-details` chỉ trên ô câu neo.
- [ ] `e2e/specs/story-12-9-strip-row-track.e2e.mjs` -- Chương 2.000 câu, câu 1.500 có khớp TM với một câu đã xác nhận; dải nằm sát dưới hàng 1.500 (hình học thật); một ca đặt hàng neo ở mép dưới khung nhìn (`block: 'end'`) rồi so `scrollTop` của `.grid-scroll` trước và sau khi dải mọc; đo khoảng giữa các `requestAnimationFrame` khi mọc và khi thu, nhiều lượt, in median và max; ghi số vào Implementation Notes kèm bản build và số câu.

**Acceptance Criteria:**
- Given Chương 2.000 câu và dải TM mở ở câu 1.500, when đo trên WKWebView, then dải hiện sát dưới câu đó và thời gian frame lúc mọc và lúc thu được ghi trong plan (không ngưỡng đạt).
- Given một lượt quét đang chạy, when người dùng sang Library rồi quay lại Workspace, then tiến độ và nút Huỷ nhìn thấy ở chân cửa sổ suốt lượt đó.

## Implementation Notes

- Anchor: `inlineStripAnchorSegmentId` (`inlineStripEligibility.ts`) returns `tmFuzzySegmentId` for TM, the caret for Glossary confirm, the caret for Proofreader only when `proofreaderStripPlace` is `track`, `null` otherwise. `proofreaderStripPlace` is `foot` whenever scan-level state is present (scanning, error, not_configured, done with total 0), `track` with a caret otherwise, `null` with no caret (summary hidden, Decision 2).
- `ProofreaderStrip` takes an optional `place` prop and renders only when it equals `proofreaderStripPlace`: one instance in `App.vue` after Quick Add (`foot`), one in `.strip-track` (`track`). Without the prop it shows by `topmostStrip` alone, so the older slot tests are unchanged. TM and Glossary confirm strips live only in the grid track.
- Grid: `.grid` gets `repeat(N+1, auto)` while anchored; each of the four columns renders a `.strip-pad` after the anchor cell; `.strip-track` is a direct child of `.grid` at `grid-column: 4; grid-row: index + 2`. The four `.col-*` carry explicit `grid-column` because the strip in column 4 would otherwise push `.col-tgt` to a fifth column.
- Strips stay mounted in `.strip-track` (`strip-track-off` hides it) because their sync watchers live in the components; `WorkspaceMode` sits in `<KeepAlive>`, so they keep syncing while Library is shown.
- Focus: `onColumnFocusOut` ignores moves into the track; `onStripFocusOut` clears the caret when focus leaves both the track and the translation column.
- DOM order: the track follows all four columns (inside `colTgt` it would route strip keys into the cell handlers and break the two-surface rule). `aria-details="strip-track"` on the anchor translation cell links them; the reading order itself still needs a screen-reader pass.
- Motion: `strip-track-in` 90 ms opacity 0.4 to 1 on the track; none under `prefers-reduced-motion`.
- Frame measurement (`story-12-9-strip-row-track.e2e.mjs`; WKWebView, debug build, Chapter of 2.000 sentences, anchor 1.500, 7 rounds, `cell.focus()` as trigger): grow first-frame gap median 193 ms (max 212); shrink median 222 ms (max 278); baseline caret move between two rows without a strip: window max median 123 ms (max 143). Grow/shrink report the first-frame gap and the baseline reports the window max, two different measures, so their difference is not the strip's cost; read them as recorded numbers for the virtualisation decision, not a pass threshold. Geometry asserted (within 2 px under the anchor cell, column 4); the bottom-edge case asserts `scrollTop` of `.grid-scroll` unchanged when the strip grows. The first e2e run timed out on `execute/async` with no code change; the rerun passed.
- Counter-check: removing the strip branch from `onColumnFocusOut` turns `gridPanelStripTrack.test.ts` red (caret case); removing the `foot` branch of `proofreaderStripPlace` turns four `stripAnchorSegment.test.ts` cases red. Removing the `place` check in `ProofreaderStrip.vue::isVisible` turns both `proofreaderStripSlots.test.ts` placement cases red; deleting `<ProofreaderStrip place="foot" />` from `App.vue` turns the new `proofreadWiring.test.ts` case red.

## Plan Change Log

- Lượt 1 → 2 (intent_gap + bad_plan, Ice trả lời 2026-10-14). Kích hoạt: phần job của dải Proofreader mất hẳn khi không có hàng neo; AC trình đọc màn hình không làm được ngoài `colTgt`. Sửa: Quyết định 5–6 và bốn hàng I/O; task `inlineStripEligibility.ts`, `ProofreaderStrip` hai thể hiện, e2e ca mép dưới; AC thứ hai thay bằng ca chuyển chế độ. Trạng thái xấu tránh được: lượt quét chạy mà không thấy tiến độ, lỗi hay nút Huỷ. KEEP (bản vá lượt 1: `/private/tmp/claude-501/-Users-hoangnam-LocalSites-addon-AuraTranslate/b113b027-2278-4617-931f-dfbbf8e8975e/scratchpad/keep-12-9-iter1.patch`, áp được bằng `git apply` trên baseline): ô đệm `.strip-pad` bốn cột, `grid-column` tường minh cho bốn `.col-*`, `.strip-track` ở `grid-column: 4` / `grid-row: index + 2` với `display: none` khi không neo, `onStripFocusOut`, keyframes 90 ms + reduced-motion, `stripAnchorSegment.test.ts`, `gridPanelStripTrack.test.ts` (bỏ ca "does not scroll": happy-dom không thể đỏ), spec e2e đo frame. Kèm sửa bốn patch của lượt 1: xoá chú thích Story 3.6 mồ côi trong `App.vue`; thụt lề phần tử con trong bốn `<template v-for>`; một dòng chú thích tại `.strip-track` rằng nó phải đứng sau các cột trong DOM để vẽ đè và nhận cú bấm; ca e2e mép dưới thay ca vitest.

## Review Triage Log

Lượt 1 (lens quick): high 1 · medium 1 · low 4 · false 3 · maybe-false 0 · defer 0. Có intent_gap ⇒ quay lại Ice, mã được hoàn về baseline.

- high · intent_gap — dải Proofreader dạng job (tiến độ, lỗi, nút Huỷ, chưa cấu hình) mất hẳn khi không có hàng neo: con trỏ `null` (`inlineStripAnchorSegmentId` trả `null`), ở Library/Chế độ đọc (`WorkspaceMode` trong `<KeepAlive>` nên không render), Chương rỗng (`.grid` trong `v-if="hasSegments"`); không chỗ nào khác hiện trạng thái job (`proofreadStateValue` chỉ có ở `ProofreaderStrip.vue` và `StatusBar.vue:193` lúc `done`). Quyết định 2 chỉ phủ dải tóm tắt.
- medium · bad_plan — AC thứ hai (trình đọc màn hình đọc dải ngay sau ô bản dịch của câu neo) không làm được trong ràng buộc "dải nằm ngoài `colTgt`": lưới không có phần tử hàng nên trong DOM dải chỉ đứng sau cả bốn cột.
- low · patch — `App.vue` còn chú thích Story 3.6 mồ côi phía trên `import SegmentHistoryOverlay` sau khi import nó tả bị xoá.
- low · patch — ca "does not scroll" của `gridPanelStripTrack.test.ts` không thể đỏ: happy-dom không có layout và dải bị stub; e2e căn giữa hàng neo nên `block: 'nearest'` cũng không cuộn. Cần ca e2e đặt hàng neo ở mép dưới và so `scrollTop`.
- low · patch — bốn `<template v-for>` trong `GridPanel.vue` không thụt lề phần tử con; chú thích `aura-allow-text` mới là bắt buộc (`check:i18n` đỏ khi thiếu, theo báo cáo triển khai) nên giữ.
- low · patch — dải vẽ đè lên vùng `.col-tgt` và nhận cú bấm chỉ vì đứng sau các cột trong DOM; bất biến này không test nào canh, cần một dòng chú thích tại `.strip-track`.
- false — thiếu dòng `HANDLER_TABLE` cho `onStripFocusOut`: `check:commands` thoát 0; bảng chỉ canh handler phím.
- false — các test slot cũ không được sửa: 10 tệp (7 tệp dải + 3 tệp mount `GridPanel`) chạy xanh 150 ca không cần đổi, vì chúng mount dải độc lập.
- false — (gộp vào dòng intent_gap) watcher của dải ngừng khi `GridPanel` không mount: `<KeepAlive>` giữ instance nên watcher vẫn chạy; hại thật chỉ là không render, đã ở dòng đầu.

Lượt 2 (lens quick): high 0 · medium 2 · low 2 · false 2 · maybe-false 1 · defer 1.

- medium · patch — không test nào canh `App.vue` mount `<ProofreaderStrip place="foot" />` ngoài `<KeepAlive>` và `GridPanel.vue` mount `place="track"`: các ca "placement" mount component với `place` truyền tay, xoá dòng ở `App.vue` vẫn xanh. Thêm ca đọc dòng mã (`codeLines`) theo mẫu `proofreadWiring.test.ts:197`.
- medium · patch — vòng đo baseline của e2e bắt đầu khi dải TM còn mở (vòng trước kết thúc bằng `focusAndSample(matchId, true)`), nên vòng 0 là một lần thu dải; max 213 ms và hiệu "+46/+85 ms" dựa vào nó. Thu dải một lượt không ghi trước vòng baseline, đo lại.
- low · patch — chú thích đầu spec e2e mang id story (luật chú thích của AGENTS.md); vế "90 ms" trong `aura-allow-opacity` là false (giá trị thiết kế của chính hiệu ứng, không phải số đo); vế `aura-allow-text` thừa là false (`check:i18n` thoát 0 và đỏ khi thiếu ở lượt 1).
- false — `ProofreaderStrip.vue` còn `flex: none` và vạch trái: thể hiện ở chân cần `flex: none` trong cột flex của `App.vue`; trong `.strip-track` (khối thường) thuộc tính đó không tác dụng; vạch trái trùng `.band` của `proofreader.html:56-64`.
- false — dải chỉ tồn tại khi `GridPanel` mount: watcher của `TmFuzzyStrip.vue:26` và `GlossaryConfirmStrip.vue:48` đều `immediate: true` nên mount lại là đồng bộ lại; Chương rỗng và Chế độ đọc không có câu đang sửa để neo.
- maybe-false · defer — `onStripFocusOut` có thể đặt con trỏ về `null` khi phần tử đang giữ tiêu điểm trong dải bị gỡ (Esc, nhận khớp) mà tiêu điểm không trả về ô; cần đo trên WKWebView. Nếu đúng: medium. Trước 12.9, vào dải đã làm con trỏ về `null`.
- low · patch — `scrollBeforeAfter` của e2e gom `cellBottom`, `rect` mà không assert; vế `preventScroll` là false: Quyết định 3 nói riêng việc dải mọc không cuộn.

## Design Notes

Lưới không có phần tử hàng nên track mới phải được mọi cột nhường chỗ; ô đệm sau hàng neo giữ chi phí Vue ở bốn phần tử mỗi lần dải đổi chỗ, còn chi phí layout lại 4×2.000 ô của trình duyệt là thứ story đo chứ không né được khi chưa ảo hoá. Thứ tự DOM của dải (sau bốn cột) khác thứ tự nhìn, nên AC thứ hai đòi kiểm thứ tự trình đọc màn hình. Nếu số đo xấu, đó là đầu vào cho quyết định ảo hoá, không phải cớ để tự hạ hành vi.

## Verification

**Commands:**
- `npm run test:story 12-9` -- expected: các test vitest của dải và lưới xanh.
- `npm run test:e2e -- --spec e2e/specs/story-12-9-strip-row-track.e2e.mjs` -- expected: xanh, in số frame.
- Một spec e2e khác chạm dải (`glossary-quick-add`) -- expected: xanh (vòng import mới giữa `GridPanel` và các dải).

**Manual checks (if no CLI):**
- Bản đóng gói macOS: mở dải ở câu cuối khung nhìn, sáng và tối (nợ `Chủ: Epic 12`).
