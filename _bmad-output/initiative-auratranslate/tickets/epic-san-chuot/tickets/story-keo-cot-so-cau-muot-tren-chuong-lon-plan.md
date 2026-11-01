---
title: 'Kéo cột số câu mượt trên Chương lớn'
type: 'bugfix'
ticket: '13'
created: '2026-11-01'
status: 'built'
baseline_revision: '275748474daf8e6807b022ad596a1d487e08277a'
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

**Problem:** Kéo trên cột số câu đo được trung vị 116 ms mỗi hàng ở Chương 9.850 câu (debug, Vite dev), tức khoảng 8 khung hình mỗi giây, và chi phí theo cỡ Chương chứ không theo số hàng đổi. Chưa có số trên bản phát hành nên chưa biết người dùng thật có chịu mức đó không.

**Approach:** Đo lại trên bản phát hành trước, tách phần JS khỏi phần sau JS. Dưới 50 ms thì đóng nợ bằng số đo, không sửa mã. Còn trên 50 ms thì cắt đúng chỗ số đo chỉ ra, chủ yếu là chỉ render lại khối có hàng đổi theo khuôn prop `caretId`.

## Boundaries & Constraints

**Always:** Vòng đo giữ đúng hình dạng 14.3: `mouseover` trong trang, chờ một khung hình, 60 hàng liền nhau và 59 bước mỗi bước 167 hàng, Chương `'句子。'.repeat(9850)`. Mỗi số ghi bản dựng, quần thể, tải máy. So trước và sau thì xen kẽ, 3 phiên mỗi bên. Khối không có hàng đổi không render lại, dù vùng chọn trải nhiều khối.

**Never:** Nới ngưỡng 50 ms; gộp `mouseover` theo khung hình; đổi `pressSegmentSelection`, `dragSegmentSelectionFocus`, hợp âm mở rộng, đường gõ; đổi thứ tự hay nội dung `segmentSelectionIds` mà panel AI và phạm vi soát đọc; thêm token màu; dùng binary `--features wdio` làm số nghiệm thu.

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected Output / Behavior | Error Handling |
|----------|--------------|---------------------------|----------------|
| Bản phát hành đạt | Trung vị 60 hàng kề < 50 ms ở cả 3 phiên | Không sửa mã, đóng nợ bằng số đo | Không |
| Bản phát hành trượt | Trung vị ≥ 50 ms | Sửa theo chỗ tốn đo được, đo lại xen kẽ | Không |
| Sửa rồi vẫn trượt | Sau sửa ≥ 50 ms, phần còn lại ở sau JS | Không hạ ngưỡng; plan `blocked`, trình Ice số JS và sau JS kèm các phương án | Giữ mã đã giúp hay trả lại là việc Ice chọn |
| Kéo trong một khối | `focus` 10 → 11, cùng khối | Đúng một khối render | Không |
| Kéo qua ranh khối | `focus` 199 → 200 | Hai khối render, tô liền | Không |
| Vùng trải nhiều khối | `anchor` 10, `focus` 450 → 451 | Chỉ khối chứa 451 render | Không |
| Kéo ngược | `focus` lùi về trước `anchor` | Tô đúng, khối ra khỏi vùng mất tô | Không |
| Chọn trọn Chương | 9.850 hàng | Mọi hàng tô, panel AI đếm 9.850 | Không |
| Không có vùng chọn | `anchor` đã chết sau đổi Chương | Không hàng nào tô | Không |

</frozen-after-approval>

## Code Map

- `_bmad-output/implementation-artifacts/14-3-ban-do/chuot-that.e2e.mjs`:115 -- vòng đo gốc, chép hình dạng; `t0` nằm trong `rAF` trước nên số là khoảng giữa hai frame, gồm cả style và layout.
- `_bmad-output/implementation-artifacts/2-16-ban-do/` -- khuôn release: `build-216.sh` nối bench vào `dist/assets/index-*.js` (CSP cấm script riêng) rồi ký ad-hoc, ~11 phút; `run-move.sh` chạy HOME nháp, phím nóng, đổ số qua `put_config`; dùng `../2-4-ban-do/front.sh`. `2-15-ban-do/` là bàn gõ (`build-215.sh`, `run-session.sh`, `analyse.py`).
- `src/panels/segmentSelectionState.ts`:81 -- `segmentSelectionIds`: `.map` trên `editorSegments.value` đi qua proxy `readonly`, O(n) mỗi bước. Nghi phạm thứ hai, chưa đo. `toRaw` đã dùng ở `GridPanel.vue::caretIndex`.
- `src/panels/GridPanel.vue` -- `selectedSegmentIds`:310 và `selectedRowClassById`:321 chỉ phục vụ ngữ cảnh khối (:1913); `caretIdIn`:659 là khuôn; `<GridBlock>`:2031 nhận prop; `onGridMouseDown` giữ nguyên.
- `src/panels/GridBlock.vue` -- prop :17; tám chỗ đọc `selectedRowClassById.get(s.id)`: 116, 120, 138, 142, 156, 175, 318, 337, tức bốn `v-memo` và bốn `:class`.
- `src/panels/gridBlockContext.ts`:23 -- trường bỏ khi Map hết người đọc.
- `src/panels/editorSegments.ts`:251 -- `selectedRowClassOf` giữ, `segmentSelection.test.ts` đọc nó.
- `tests/frontend/gridBlockRenderCost.test.ts` -- khuôn đếm khối render, thêm ca vùng chọn.
- `tests/frontend/gridCaretMoveCost.test.ts`:230, `gridSegmentSelectionDrag.test.ts` -- phải xanh không đổi.
- Không chạm: `src/proofreadScope.ts`, `src/aiTranslateHandlers.ts`, `src/panels/AiTranslationPanel.vue`.

## Tasks & Acceptance

**Pha (AGENTS.md, mỗi pha một agent mới, giao qua `_bmad-output/implementation-artifacts/14-13-ban-do/HANDOFF.md`):** Pha 1 là hai mục đầu, kết thúc bằng số trước và kết luận đạt hay trượt, rồi dừng. Pha 2 là mã và test. Pha 3 là số sau, bàn gõ, chuột thật, sổ nợ. Đạt ở Pha 1 thì Pha 2 không chạy.

⚠️ `build-*.sh` nối bench vào `dist/` của cây đang dựng, nên sau mỗi bản dựng phải `npm run build` lại, nếu không `cargo test` nhúng bench vào binary. Lệnh dựng vượt 10 phút nên chạy nền, có trần `perl -e 'alarm shift; exec @ARGV'`.

**Execution:**
- [x] `_bmad-output/implementation-artifacts/14-13-ban-do/` -- `bench-1413.js`, `build-1413.sh`, `run-drag.sh`, `analyse.py`: dựng release ở `baseline_revision`, ba phiên, ghi trung vị, tối đa, số lượt trên 50 ms, thêm vế JS đọc sau một microtask (Vue flush xong) -- biết chi phí nằm đâu trước khi sửa.
- [x] `_bmad-output/implementation-artifacts/14-13-ban-do/ket-qua.md` -- số trước -- đạt thì nhảy thẳng tới bước sổ nợ.
- [x] `src/panels/segmentSelectionState.ts` -- computed khoảng chỉ số `{lo, hi}`, `segmentSelectionIds` suy từ đó, đọc qua `toRaw` -- một nguồn sự thật, bỏ O(n) qua proxy.
- [x] `src/panels/GridPanel.vue`, `src/panels/gridBlockContext.ts` -- bỏ Map, thêm `selectedFromIn(block)` và `selectedToIn(block)` trả số nguyên thuỷ -- khối ngoài vùng hoặc nằm trọn trong vùng giữ prop y nguyên.
- [x] `src/panels/GridBlock.vue` -- hai prop mới; tám chỗ đọc đổi sang chỉ số `j`; `v-memo` theo boolean đã chọn.
- [x] `tests/frontend/gridBlockRenderCost.test.ts` -- các hàng khối của I/O Matrix; đối chứng: cho khối đọc thẳng khoảng toàn cục, ca đỏ vì mọi khối render.
- [ ] `_bmad-output/implementation-artifacts/14-13-ban-do/ket-qua.md` -- số sau xen kẽ với số trước; thêm bàn gõ `2-15-ban-do` trước và sau, 3 phiên mỗi bên ở 9.850 câu.
- [x] `_bmad-output/initiative-auratranslate/deferred-work.md` -- đóng mục `Chủ: Story 14.13` bằng số đo, hoặc giữ mở nếu rơi vào hàng "Sửa rồi vẫn trượt".

**Acceptance Criteria:**
- Given bản phát hành ký ad-hoc, Chương 9.850 câu, when kéo 60 hàng liền nhau qua bàn đo, then trung vị mỗi hàng dưới 50 ms ở từng phiên trong 3 phiên, `ket-qua.md` ghi revision, WebKit, quần thể, tải máy.
- Given có sửa mã, when đọc `ket-qua.md`, then có số trước và sau cùng bàn, cùng phiên xen kẽ, cả hai hình dạng kéo.
- Given có sửa mã, when chạy bàn gõ trước và sau, then phím cuối và `Mod+Enter` không tăng quá dao động giữa các phiên của bản trước.
- Given bản phát hành, when kéo chuột thật qua ba số câu rồi `Shift`+bấm câu xa, then đúng các câu đó được tô như trước.

## Implementation Notes

- Hàng "Kéo qua ranh khối" của Matrix ghi hai khối, nhưng một bước qua ranh chỉ đổi một hàng nên đúng một khối render, khớp §Always. Test khẳng định `[1]`; chờ Ice sửa câu trong khối frozen.
- `dragSegmentSelectionFocus` vẫn `.map` qua proxy và `includes` ở mỗi `mouseover`: §Never giữ nguyên hàm này, đây có thể là vế `js` 4–5 ms còn lại.
- AC gõ và AC chuột thật chưa đo: tiến trình agent lúc đó bị `-1743` (Automation), mục nợ `Chủ: Ice` giữ hai vế này; `app215-*`, `app-mouse` đã dựng sẵn, `run-mouse.sh` chưa chạy lần nào.
- Đối chứng gỡ thật: bỏ phép cắt theo khối ở `selectedFromIn`, `selectedToIn`, 5/7 ca vùng chọn đỏ vì cả 6 khối render; trả về byte y nguyên.
- `npm run build` đỏ từ `afdc4205` (đã ở sổ nợ, `Chủ: Ice`), nên bàn đo dùng `npx vite build`.

## Plan Change Log

## Review Triage Log

Lượt 1 (lens Quick): high 0 · medium 1 · low 5 · false 4 · maybe-false 0.

- medium, chờ Ice: AC gõ và AC chuột thật chưa đo; do quyền `-1743` của host, không vá được từ đây, plan giữ `in-review`.
- low, patch: ✅ ở mục nợ quá sớm khi hai AC còn mở; đổi thành 🟡 `Chủ: Ice`, gộp mục tách riêng vào đó.
- low, patch: đối chứng chưa ghi; chạy lại bằng phép gỡ thật, 5 ca đỏ, ghi ở Implementation Notes.
- low, patch: `2-15-ban-do/build-215.sh` bị sửa tại chỗ; trả về, `app215-*` đã dựng nên không cần nó.
- low, patch: `eslint-disable` không lý do ở hai bench; thêm lý do. Comment usage sai ở `build-1413m.sh`; sửa.
- low, patch: bản dựng lớn chỉ được canh bằng lời nhắc; thêm `.gitignore` của thư mục.
- reject (sửa khối frozen): hàng "Kéo qua ranh khối" ghi hai khối; một bước chỉ đổi một hàng, §Always đòi một khối, chờ Ice sửa câu.
- false: "trung vị mỗi lượt" khác AC 1; mỗi lượt là trung vị theo hàng, cùng thống kê.
- false: xoá comment token ở `GridPanel.vue` làm mất bất biến; `GridBlock.vue`:808 vẫn ghi.
- false: `segmentSelectionRange` thiếu doc comment; AGENTS.md mặc định không comment, tên và kiểu đã đủ.
- false: hàng "Sửa rồi vẫn trượt" áp dụng; trung vị dưới 50 ms ở cả 3 phiên cả hai hình dạng.

## Design Notes

Dải chọn luôn liền nhau (`ids.slice(lo, hi + 1)`), nên mỗi khối chỉ cần hai số. Khối ngoài vùng nhận `-1, -1`; khối trọn trong vùng nhận `0, rows - 1` nên kéo tiếp không làm nó đổi prop. Cha vẫn render mỗi bước như lượt dời caret, chỉ so ~50 bộ prop nguyên thuỷ.

Ở bàn 2-16, dời caret có JS 1–3 ms nhưng phần sau JS 53–66 ms, chưa quy nguồn (bug 2-14). Lượt tô này có thể đụng cùng trần, nên hàng "Sửa rồi vẫn trượt" là khả năng thật, và vế JS tách riêng để thấy điều đó ngay từ số trước.

## Verification

**Commands:**
- `npx vitest run tests/frontend/gridBlockRenderCost.test.ts tests/frontend/gridCaretMoveCost.test.ts tests/frontend/gridSegmentSelectionDrag.test.ts tests/frontend/segmentSelection.test.ts tests/frontend/gridTypingCost.test.ts` -- exit 0, không đổi ngưỡng.
- `npm run test:story 14-13`, `npx vue-tsc --noEmit`, `npm run check:lint`, `npm run check:debt-owner` -- exit 0.

**Manual checks (if no CLI):**
- Bàn đo release chạy ngoài sandbox, màn hình mở khoá, không chạm máy trong phiên.
