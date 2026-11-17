---
title: 'Refactor sweep'
type: 'refactor'
ticket: '9'
created: '2026-11-17'
status: 'built'
baseline_revision: 'f01fe13127ed98179472d631e8a7ae78887358ad'
route: 'full'
route_source: 'auto'
risk: 'medium'
review: 'quick'
review_source: 'pinned'
lenses_ran: ['quick']
review_loop_iteration: 1
context:
  - '{project-root}/src/AGENTS.md'
  - '{project-root}/e2e/AGENTS.md'
---

<frozen-after-approval reason="human-owned intent — do not modify unless human renegotiates">

## Intent

**Problem:** Epic 13 chỉ đóng khi mọi việc 13.1–13.8 để lại đã được đóng hoặc ghi lý do. Phạm vi chốt ngày 2026-11-17 có bốn mục:
1. Hai mục nợ e2e là `deferred-work.md` §Story 13.6 (`12-8`, `12-9`) và §Story 13.4 (`12-34`). Cả hai đã đóng bằng lời: `c5d38600` sửa spec cũ, nightly 43995942157 xanh, lượt cục bộ chạy trên Vite mới.
2. 13.4 làm trái luật Always của chính nó. Hàng baseline đứng tự do, một dòng, chưa khai chiều cao bội 7: 146 hàng thì 4 hàng có khai.
3. Cũng của 13.4: nút chỉ có biểu tượng (`ui-icon-btn`, mục `ui-seg--icons`) canh icon giữa hộp 28px, nên đáy icon không nằm trên baseline như DESIGN.md §Lưới 7px — Canh baseline đòi.
4. `ReviewDock.vue:168` `.diff-status { min-height: var(--leading-ui-sm) }` là khai báo chết, vì token giãn dòng là số không đơn vị.

**Approach:** Chỉ sửa CSS. Hàng một dòng khai `height` 21 (hàng chữ), 28 (hàng điều khiển) hoặc 35 (hàng có điều khiển `--lg`). Icon trong nút chỉ có biểu tượng đặt đáy trên baseline của hộp dòng rỗng sẵn có, cả nhóm nằm giữa nút. Mỗi mục có một spec e2e trên WKWebView.

**Decision (Ice 2026-11-17):** (2) và (3) gộp vào 13.9, kể cả các tệp `*Overlay.vue` mà Epic 17 sắp dựng lại khung. Checkbox và radio giữ canh giữa dòng nhãn đầu, đúng như `.ui-check input` hiện có. Đó là ngoại lệ của luật "đáy biểu tượng trên baseline", ghi vào DESIGN.md.

**Decision (Ice 2026-11-17, sau khi đo):** đầu lớp phủ có tiêu đề `read-title` (hộp dòng 35) cao 35px, nút Đóng `align-self: center` thay vì theo baseline của tiêu đề. Cao tự nhiên của đầu là 36 vì baseline đẩy nút 28 xuống 8; khai 28 thì tiêu đề tràn 8. Đây là ngoại lệ thứ hai, ghi vào DESIGN.md cạnh ngoại lệ checkbox/radio.

## Boundaries & Constraints

**Always:** Hàng có `flex-wrap: wrap`, hoặc giữ chữ của người dùng hay dữ liệu có thể xuống dòng, thì không khai chiều cao. Một hàng chưa chắc thì đo trên WKWebView ở bề rộng tối thiểu: một dòng thì khai, không thì để nguyên và ghi vào Implementation Notes. Khai chiều cao không được làm con tràn khỏi hàng; nếu cao tự nhiên lớn hơn bậc (vd. h2 cạnh nút 28) thì sửa nguồn lệch, không hạ bậc. Chiều cao điều khiển chỉ sửa ở `controls.css` (`controls-adoption.test.ts`).

**Never:** Thân lưới Workspace. Đổi khoảng 7/14, cỡ chữ, màu hay chiều cao điều khiển. Khai `height` trên lớp điều khiển hay nhãn `ui-check`. Thêm miễn trừ cổng để khỏi sửa giá trị. `height: var(--leading-*)` (không đơn vị).

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected Output / Behavior | Error Handling |
|----------|--------------|---------------------------|----------------|
| Đầu lớp phủ | `.set-head`: h2 + nút Đóng `ui-btn--quiet` | cao 28 ±0,5; mọi con nằm trong hộp hàng | — |
| Hàng chữ có thể xuống dòng | `.work-stat` ở Library (thẻ 238px, có "(ghi đè thủ công)") | không khai chiều cao (🔵 2026-11-17, Ice: thay "cao 21" sau khi đo thấy hàng xuống dòng) | — |
| Hàng điều khiển panel | `.lookup-tools` | cao 28 ±0,5 | — |
| Nút chỉ biểu tượng | `ui-icon-btn` ở `StatusBar.vue:547` cạnh `ui-btn` | cùng đỉnh ±0,5; đáy svg = baseline chữ nút bên cạnh ±0,5 | — |
| Checkbox | nhãn `ui-check` | ô 14px canh giữa dòng nhãn đầu (không đổi) | — |
| Bẫy | gỡ `height` của một hàng có cao tự nhiên lệch bậc; trả `.ui-icon-btn` về `align-items: center` | spec đỏ đúng ca đó | — |

</frozen-after-approval>

## Code Map

- `src/tokens/controls.css` -- `:159` `.ui-row`; `:166` `::before` hộp dòng rỗng `\200b` dùng chung cho `.ui-icon-btn`, `.ui-seg--icons .ui-seg__item`, `.ui-check`; `:177` `.ui-icon-btn` (`--lg` 35 ở `:192`); `:667` `.ui-seg--icons .ui-seg__item`. Đã đo ở lượt 1: `flex-wrap: wrap; align-items: baseline; align-content: center` trên hai lớp này đặt đáy svg lên baseline và để nhóm hộp dòng 21 nằm giữa nút. Flex một dòng thì bỏ qua `align-content`. Trên cây gốc spec đỏ 5,5px (nút thanh trạng thái) và 3,5px (công tắc bố cục).
- `panels/LookupRecord.vue:117`, `:286` `.lookup-collapse--closed { transform: rotate(-90deg) }` -- đây là `ui-icon-btn` duy nhất bị xoay. Icon nay lệch tâm nút ~2,5px, nên xoay cả nút sẽ làm chevron nhảy ngang. Phải xoay svg quanh tâm của nó, không xoay nút.
- Nút chỉ có biểu tượng: `App.vue:254`, `StatusBar.vue:547,558`, `layout/ReviewDock.vue:94,105`, `ImportPreviewOverlay.vue:1705,1717`, `modes/LibrarySearchResults.vue:230,245`, `modes/LibraryMode.vue:874,885`, `modes/LibraryWorkPage.vue:727`, `modes/ReadingLookupPopup.vue:129`, `modes/ReadingMode.vue:653–708`, `panels/LookupRecord.vue:116,162`, `panels/LookupPanel.vue:413,462,705`.
- Đầu có tiêu đề `read-title` (hộp dòng 35), theo Decision thứ hai: `height: 35px` cộng `.X-head > :last-child { align-self: center }`. Nút nằm trần ở `cvi`, trong `.cvm-head-actions` ở `cvm`, trong `<form>` ở `pl`. Cao tự nhiên đo ở `.aip-head` và `.pl-head` là 36.
  - Có 19 đầu: `.aip-head`, `.al-head`, `.attr-head`, `.cm-head`, `.cvi-head`, `.cvm-head`, `.ex-head`, `.gh-head`, `.gi-head`, `.gm-head`, `.gq-head`, `.li-head`, `.pi-head`, `.pl-head`, `.ri-head`, `.hist-head`, `.ti-head`, `.tm-head` và `layout/LookupDrawer.vue` `.ld-head` (`.ld-title` là `read-title`).
  - `.ex-head` có đệm 14 + viền 1 (`--space-panel-block` 14), nên tổng border-box phải là bội 7; bù nét theo cách 13.8.
- Đầu có tiêu đề `ui-md`: `.set-head` content-box 28 (đệm 13 + viền, tổng 55, đã đo xanh), `.pfmem-head`, `.pfscope-head`.
- Ứng viên khác, chỉ khai khi đo đạt luật:
  - Hàng 28/35: `.aip-set-field`, `.pl-seg`, `.search-field`, `.wp-sort`, `.lookup-tools`, `.crv-group-head`, `.sc-search` (đo 35), `.pl-rename-form .pl-field` (đo 35).
  - Hàng 21: `.aip-tabs`, `.aip-brow`, `.ex-legend`, `.cm-row`, `.tm-legend-item`.
- Đã đo ở lượt 1, không khai:
  - `.review-bar`: ba `<p>` cao 35 (đệm 7) và có thể xuống dòng.
  - `.hist-row`, `.aip-bh`: con có viền cao 22/23.
  - `.ai-batch-row`: chữ 16px thừa kế tỉ số 1.615, cao ~25,8.
  - `.pl-kind`: cao 30 vì strut của `<form>`.
  - `.work-stat`: xuống dòng ở thẻ 238px khi có "(ghi đè thủ công)".
  - `.lookup-tabs`: khai 28 làm `.lookup-tabrow` thành 31 và 13-6 đỏ.
  - `.lookup-row`: dữ liệu. `.ip-head`, `.pl-vrow`, `.pt-item`, `.lookup-entry-head`: dữ liệu hoặc xuống dòng.
  - `.lookup-head-row`: 37, lệch bậc.
  - Không với tới được: `.bip-head`, `.bip-chapters-entry`, `.ip-chapters-entry`, `.rl-head`, `.lcp-head`.
- `layout/ReviewDock.vue:166` `.diff-status` -- border-box, đệm 7 trên và dưới. Muốn giữ chỗ một dòng khi rỗng thì `min-height` phải là 35.
- Đã có `height: 28px`, giữ nguyên: `.gm-foot`, `.orphan-actions`, `.wp-crumb`, `.tabs`.
- `/private/tmp/claude-501/-Users-hoangnam-LocalSites-addon-AuraTranslate/30bf66e4-7c37-4d44-9833-04aefcc63feb/scratchpad/13-9-loop1/` -- diff `src/` và spec của lượt 1 đã gỡ, chỉ để tham khảo. Không chép nguyên.
- `e2e/specs/story-13-4-canh-baseline.e2e.mjs` -- khuôn đầu dò baseline. `tests/frontend/controls-css.test.ts:72` đã canh ô checkbox.

## Tasks & Acceptance

**Execution:**
- [ ] `e2e/specs/story-13-9-chieu-cao-hang-va-icon.e2e.mjs`:
  - Một bảng `{mở màn, selector, bậc}` cho MỌI hàng được khai. Mỗi hàng đo hai lần trên WKWebView ở 800px: một lần với `style.height = 'auto'` đặt trong `browser.execute` (cao tự nhiên), một lần như khai. Đòi: cao khai bằng bậc ±0,5; cao tự nhiên ≤ bậc + 0,5; không con nào ra ngoài border-box ±0,5.
  - Thêm ca nút biểu tượng: thanh trạng thái, công tắc bố cục, `.lookup-collapse` đóng và mở. Đòi đáy svg = baseline ±0,5, nhóm hộp dòng nằm giữa nút ±0,5, chevron không dời tâm khi xoay.
  - Viết trước, chạy trên cây gốc, ghi các ca đỏ.
- [ ] `src/tokens/controls.css`, `panels/LookupRecord.vue` -- sửa icon và chuyển phép xoay sang svg -- mục 3.
- [ ] Các tệp có đầu `read-title` và hàng ứng viên -- chỉ khai những hàng mà bảng của spec đã đo đạt; hàng không đạt hoặc không với tới thì không khai -- mục 2.
- [ ] `src/layout/ReviewDock.vue` -- `.diff-status` `min-height: 35px` -- mục 4.
- [ ] `deferred-work.md` §Story 13.9 -- một mục, `Chủ: Ice`, liệt kê các hàng lệch bậc, không với tới hoặc chưa có cách khai: `.lookup-head-row`, `.review-bar`, `.ai-batch-row`, `.hist-row`, `.aip-bh`, `.pl-kind`, `.work-stat` và năm hàng không với tới.

**Acceptance Criteria:**
- Given cây sau story, when chạy `story-13-9` ngoài sandbox, then xanh. Given phép gỡ `height` thật của một hàng trong bảng có cao tự nhiên khác bậc (cách 21,5 do trộn font), hoặc trả `.ui-icon-btn` về `align-items: center`, then đỏ đúng ca đó vì lệch, không vì hết giờ. Nếu không hàng nào trong bảng lệch bậc, ghi số đo thay cho phép gỡ hàng.
- Given `npm run check:tokens`, then exit 0. `aura-allow-spacing` chỉ được thêm ở khai báo bù đúng bề dày nét.
- Given mọi `height` mới trong diff, then selector của nó có trong bảng của spec.
- Given `story-13-1`, `13-4`, `13-6`, `13-7`, `13-8`, `controls-css`, `controls-adoption`, then xanh. `titlebar-tools` đỏ từ trước 13.9, đã có mục nợ `Chủ: Story 17.6`.

## Implementation Notes

- Hai mục nợ e2e đã đóng bằng lời (`c5d38600` sửa spec cũ; nightly 43995942157 82/82; chạy lại trên Vite mới do `wdio` dựng). Lượt đầu trên Vite của `tauri dev` có tem `?t=` thì không tính.
- `12-9` đỏ một lần ở vị trí 2 của lượt bốn spec (WebDriver 4445 khởi động lại, ba `execute/async` hết giờ); chạy riêng hai lần thì xanh. Đó là lỗi bộ chạy, không phải spec.
- `titlebar-tools` đỏ hai ca, giống nhau trên cây gốc `f01fe131` và trên cây có diff (menu có thêm `library.add_chapters`, `palette.open`); đã ghi nợ, `Chủ: Story 17.6`.
- Không làm, vì review story gốc đã kết luận: `.sr-value` lệch ~1px (13.8), `.work-progress-track` −3px (13.7).
- Bảng của `story-13-9` đo 19 đầu `read-title` + `.ex-head`, `.ld-head`, `.pfmem-head`, `.pfscope-head`, `.diff-status` trên probe mang thuộc tính scope của lớp phủ (đo CSS giao hàng, không đo dữ liệu); `.set-head`, `.pl-head`, `.lookup-tools`, `.search-field`, `.wp-sort`, `.sc-search`, `.pl-seg`, `.pl-field` đo ở màn thật. Mỗi hàng đòi thêm một luật CSS có `height`/`min-height` khớp nó, vì hàng nào cao tự nhiên đã đúng bậc thì gỡ `height` không làm cao đổi.
- Không hàng nào trong bảng có cao tự nhiên lệch bậc (đều đúng bậc), nên phép gỡ thật là gỡ `height` của `.lookup-tools` và `.pl-head` cùng trả `.ui-icon-btn` về `align-items: center`: đỏ đúng 5 ca (hai hàng, probe `.pl-head`, thanh trạng thái, `.lookup-collapse`).
- `.ex-head` đệm dọc 13 (`aura-allow-spacing`) để 35 + 26 + viền 1 + viền khung 1 = 63; `.work-stat` không khai (Always: có thể xuống dòng); Ice sửa hàng Matrix tương ứng.
- Chưa khai và ghi nợ `Chủ: Ice`: hàng lệch bậc, hàng cần dữ liệu e2e không dựng được (`.aip-tabs`, `.aip-brow`, `.aip-set-field`, `.ex-legend`, `.cm-row`, `.tm-legend-item`, `.crv-group-head`) và năm hàng không với tới.
- Cao tự nhiên (`height: auto`) đo ở cửa sổ 800px (Lookup: 1120x900, tầng `full`; probe ở bề rộng thật của panel): mọi hàng trong bảng đúng bậc, không hàng nào lệch: `.lookup-tools`, `.pl-seg`, `.search-field`, `.wp-sort`, `.set-head` (hộp nội dung) 28; `.pl-head`, `.pl-field`, `.sc-search`, 19 đầu `read-title` (`.ex-head`, `.ld-head` hộp nội dung) 35; `.pfmem-head`, `.pfscope-head` 28; `.diff-status` 35 và `.review-bar` 35 khi rỗng (cần hộp dòng `::before` `\200b`, thiếu thì 49).
- Chưa đọc CI của HEAD: 48 commit chưa đẩy, nửa Windows chỉ có ở CI. Phải đọc trước khi `done`.

## Plan Change Log

- 2026-11-17, sau review lượt 1: oneshot → full. Ice gộp hai luật Always của 13.4 mà sweep bỏ sót, cùng ngoại lệ checkbox/radio. Bản kê 146 hàng baseline do một agent Explore làm lộ thêm `.diff-status`. GIỮ: hai dòng đóng sổ nợ và bằng chứng nightly.
- 2026-11-17, review lượt 2 (loop 1, bad_plan):
  - Phát hiện: Code Map xếp hàng vào nhóm "Hàng 28/21" từ bản kê tĩnh, không bắt đo. Hệ quả là nhiều hàng bị khai thấp hơn cao tự nhiên và con tràn (`.ld-head`, `.review-bar`, `.ai-batch-row`, `.hist-row`, `.aip-bh`, `.pl-kind`), một hàng có thể xuống dòng (`.work-stat`), `.ex-head` có tổng 64. Plan còn bảo khai `.diff-status` 21px, giá trị không giữ được chỗ dưới border-box. Hàng Bẫy khi đó đẩy người làm tới chỗ hạ bậc (`.pl-kind` 30 → 28).
  - Đã sửa: nhóm hàng thành ứng viên, chỉ khai khi bảng đo của spec đạt; chuẩn Bẫy là lệch do trộn font ≤ 0,5; thêm `.ld-head`, phép xoay `.lookup-collapse`, mục nợ cho hàng không khai, `.diff-status` 35, AC trừ `titlebar-tools`.
  - Trạng thái xấu cần tránh: khai `height` ở một hàng chưa đo, hoặc nhỏ hơn cao tự nhiên.
  - GIỮ: cách sửa icon ở `controls.css` (đã đo); 19 đầu `read-title` dùng 35 + `> :last-child { align-self: center }`; `.set-head` content-box 28; `.sc-search` và `.pl-field` 35; ca `.pl-head` có phép gỡ thật; dòng DESIGN.md và mọi dòng sổ nợ đã ghi.

## Review Triage Log

Lượt 1 (quick): 7 phát hiện — medium 3 · low 4 · patch 4 · HALT 3.

- medium · HALT — 13.4 để hàng một dòng đứng tự do không khai chiều cao bội 7, trái luật Always của chính plan 13.4 (13.6 vá `.tabs` cùng luật); review 13.4 không xét, sổ nợ không có, sweep không phủ.
- medium · HALT — nút chỉ có biểu tượng canh giữa hộp 28px (đáy cao hơn baseline 3,5px), trái luật Always "biểu tượng đặt đáy trên baseline" của 13.4; câu "đã kết luận ở review" sai, chỉ có ở Implementation Notes.
- medium · HALT — lượt e2e cục bộ chạy trên Vite của `tauri dev` có tem `?t=` (`vi.json?import&t=1794831439199`); chạy lại cần một Vite mới, tức Ice tắt `tauri dev`.
- low · patch — Intent ghi `panel-toggle-mouse` ✅; thật là 🟡 `Chủ: Ice`. Khối frozen không sửa, đính chính ở Implementation Notes.
- low · patch — dòng đóng §Story 13.6 thêm 🔵: ca đỏ của 12-9 là `gapAboveNext` 161, không phải `gapBelowCell`.
- low · patch — chưa đọc CI của HEAD (48 commit chưa đẩy); ghi ở Implementation Notes là việc còn lại trước `done`.
- low · patch — `12-18`/`12-14` của 13.8 dẫn bằng chứng; mục nợ lượt đêm `41134334441` (`Chủ: Epic 14`) nối dòng 🔵 nightly 82/82.

Lượt 2 (quick): 12 phát hiện — medium 7 · low 4 · false 1 · bad_plan 11 (loop 1).

- medium · bad_plan — `.ld-head` khai 28 nhưng `.ld-title` là `read-title` 35, tiêu đề tràn như 18 đầu lớp phủ.
- medium · bad_plan — `.review-bar` 28 nhưng ba `<p>` cao 35 (đệm 7, `ReviewDock.vue:158,168,178`) và có thể xuống dòng.
- low · bad_plan — `.diff-status` `min-height: 21px` không giữ chỗ dưới border-box có đệm 7; plan tự ghi sai con số.
- medium · bad_plan — `.ex-head` content-box 35 + đệm 14×2 + viền 1 = 64, lệch lưới.
- medium · bad_plan — `.lookup-collapse--closed` xoay cả nút, nay icon lệch tâm nút nên chevron nhảy khi thu/mở.
- medium · bad_plan — `.ai-batch-row` 21, chữ 16px thừa kế tỉ số ~25,8, bị hạ bậc.
- low · bad_plan — `.hist-row`/`.aip-bh` 21, con có viền cao 22/23 tràn.
- medium · bad_plan — `.pl-kind` 28, tự nhiên 30; Bẫy của plan đẩy tới chỗ hạ bậc mà Always cấm.
- medium · bad_plan — `.work-stat` 21 có thể xuống dòng ở thẻ 238px khi có "(ghi đè thủ công)".
- low · bad_plan — hàng lệch bậc/không với tới không có mục nợ có chủ.
- false — "13-1/4/6/7 và pre-push chưa chạy sau CSS cuối": lượt verify sau cùng chạy cả sáu spec 13-x xanh và `pre-push` xanh 2315 s; phần `titlebar-tools` đỏ là thật, AC sửa thành ngoại lệ có nợ (gộp vào dòng bad_plan cuối).
- low · bad_plan — ca phân đoạn chỉ đòi `topGap > 0`, không đòi nhóm nằm giữa; Implementation Notes quá 10 dòng, kể lịch sử pha và số trung gian (đã gọn lại). Ca checkbox: false, `controls-css.test.ts:72` đã canh.

Lượt 3 (quick): 10 phát hiện — medium 3 · low 5 · maybe-false 1 · intent_gap 1 (Ice đã trả lời) · patch 7 · defer 1 · reject 1.

- medium · patch — `.diff-status` `min-height: 35px` trong hàng `align-items: baseline`: `<p>` rỗng không có hộp dòng nên baseline tổng hợp ở đáy, `.review-bar` thành ~48,5. Cho `<p>` rỗng một hộp dòng và đo `.review-bar` khi rỗng.
- medium · patch — hàng trên màn thật đo ở cửa sổ 10.000px (`ensureFullLayoutTier`), không ở 800; probe đo ở cột 800 thay vì bề rộng panel thật. `.wp-sort` có nhãn có thể xuống dòng.
- low · patch — phép gỡ chỉ đỏ nhờ `hasRule` (không xét giá trị); chưa ghi số cao tự nhiên thay cho phép gỡ như AC đòi.
- medium · patch — thiếu assert "nhóm hộp dòng nằm giữa nút ±0,5" ở ba ca nút biểu tượng; ca thanh trạng thái so baseline của chính hàng chứa nút.
- low · patch — ca `.lookup-collapse` `skip` khi không có từ điển (CI), chỉ chạy cục bộ; ghi vào mục nợ 13.9.
- low · patch — đầu probe chỉ đòi cao ≤ 35 và không tràn; `align-self: flex-end` vẫn qua. Đòi con cuối canh giữa ±0,5.
- low · patch — `btnTop` đọc mà không assert (spec :518).
- maybe-false (low nếu thật) · defer — chevron trong nút chỉ biểu tượng nay đáy trên baseline, nét giữa ô nên trông cao; luật hạ 3px của DESIGN chỉ nói chevron "đứng cạnh chữ". Lượt dùng thật của Ice phân xử; nối vào mục nợ 13.9.
- maybe-false (low nếu thật) · reject — nút chứa ký tự `‹ › ⋯`: ký tự là chữ cùng phông, cùng hộp dòng với `::before`, nên canh baseline cho cùng vị trí với canh giữa cũ; đo một nút `‹` sẽ phân xử.
- intent_gap — Matrix "`.work-stat` cao 21" trái luật Always (hàng xuống dòng). Ice 2026-11-17: sửa Matrix, giữ mã.
- Ngoài review: `pre-push` đỏ ở vitest với 288/288 tệp, 4653/4653 ca qua, 1 `EnvironmentTeardownError` (`onUserConsoleLog` pending, khi chạy `editorConfirmSegment.test.ts`). Diff không chạm tệp vitest đọc ngoài `controls.css`; chạy lại sau khi vá.

## Verification

**Commands:**
- `npm run check:tokens` -- expected: exit 0.
- `npx vitest run tests/frontend/controls-css.test.ts tests/frontend/controls-adoption.test.ts` -- expected: 0 failed, 0 Errors.
- `npm run test:e2e -- --spec e2e/specs/story-13-9-chieu-cao-hang-va-icon.e2e.mjs` rồi `story-13-1`, `13-4`, `13-6`, `13-7`, `13-8` (ngoài sandbox, không có `tauri dev` đang chạy) -- expected: pass.
- `.githooks/pre-push` với ref `master` so `origin/master`, trần `alarm` 3600 s -- expected: mọi bước xanh (`verify` của ticket).
- `npm run check:debt-owner` -- expected: exit 0.

**Manual checks:**
- Lượt dùng thật của Ice thuộc mục nợ `Chủ: Epic 13`, không thuộc story này.
