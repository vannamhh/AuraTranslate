---
title: 'Lưới đối chiếu chia khối 200 hàng'
type: 'feature'
ticket: 16
created: '2026-10-23'
status: 'in-progress'
baseline_revision: '5f25df6fac8287ff89d437c0c3dfe7a2b11443f0'
route: 'full'
route_source: 'auto'
risk: 'high'
review: ''
review_source: ''
lenses_ran: []
review_loop_iteration: 0
context: []
blocked_at: ""
blocked_reason: ""
---

<frozen-after-approval reason="human-owned intent — do not modify unless human renegotiates">

## Intent

**Problem:** Năm cột của lưới (`.col-rule/num/src/tgt` + `#strip-track`) là năm `subgrid` trải mọi hàng của Chương. WebKit vì thế tính lại bố cục cả lưới trong mỗi lệnh gõ: ~0,5 s mỗi phím ở 9.850 câu (bug 2.15, research D4), và có thể cả khi dời con trỏ (giả thuyết của 2.14).

**Approach:** Lấy số "trước" của lượt dời con trỏ trên bản release. Rồi làm bản thử chỉ chia phần bố cục: mỗi khối ≤ `GRID_BLOCK_ROWS` (200) hàng là một lưới riêng, có năm cột subgrid riêng, và mọi hàng vẫn ở trong DOM. Đo theo tiêu chí 1 của story: không đạt thì dừng và không commit phần chia khối; đạt thì làm xong vùng chọn, cổng, test, e2e, rồi chạy lượt đo nghiệm thu một lần.

## Boundaries & Constraints

**Always:** giữ mọi Boundaries của story. Thêm:
- Ngoài lưới chỉ đọc vùng chọn theo segment id.
- Không thêm `scrollIntoView` hay `preventScroll`: con trỏ được cuộn tới nhờ `focus()` (GridPanel :1277-1316).
- Mỗi số đo ghi bản dựng, phiên bản WebKit, số câu, cách gửi phím/chuột và tải máy.
- Quyết định 2026-10-23, Ice, về tiêu chí 2 (phương án A):
  - Kéo trong ô bản dịch giữ như nay: mỗi ô là một editing host, lượt kéo không ra khỏi ô.
  - Khoảng id của 4.9 (`Mod+Shift+↓/↑`) mở rộng qua ranh giới khối, tô đúng, và các lệnh dùng nó cho kết quả như trước.
  - Kéo chọn trong cột nguyên văn qua ranh giới khối chỉ tô cột nguyên văn.
  - `⌘A` giữ nguyên hành vi hiện có.
  - Kéo ô dịch thành khoảng id và `⌘A` chỉ chọn cột nguyên văn không thuộc story này.
- Quyết định 2026-10-23, Ice: giữ trọn plan dù 4.163 token, vượt hướng dẫn 1.600.

**Never:**
- Không ảo hoá hàng.
- Không dùng `contain` hay `content-visibility` trên khối: chúng tắt subgrid (research D1).
- Không hạ ngưỡng, không thêm miễn trừ để làm xanh một cổng.
- Không đổi điều kiện của spec e2e, chỉ đổi cách tìm hàng.

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected |
|---|---|---|
| Ít hơn một khối | Chương 20 câu | đúng 1 khối, 20 hàng |
| Khối cuối lẻ | 450 câu | 3 khối: 200 · 200 · 50 |
| Tách ở hàng 199 | khối 0 lên 201 hàng | chia lại: khối 0 còn 200 hàng; con trỏ và `⌘Z` của ô đang gõ còn nguyên |
| Gộp ở hàng đầu khối 1 | `Backspace` đầu ô 200 | gộp vào hàng 199 của khối 0, như ở giữa khối |
| Dải neo ở hàng cuối khối | neo = hàng 199 | `.strip-pad` và dải nằm trong khối 0, `gridRow` tính theo khối |
| Điều hướng tới khối khác | `segment.next_untranslated` từ khối 0 tới câu ở khối 30 | con trỏ ở đúng câu, câu đó được cuộn vào |
| Bôi đen nguyên văn | một cụm ở khối 5 | panel Lookup tra ngay |

</frozen-after-approval>

## Code Map

- `src/panels/GridPanel.vue` -- template :1799-2069; CSS :2193-2225.
  - `.grid` đặt `gridTemplateRows` toàn cục (:1808). Các cột có `grid-row: 1/-1; subgrid` (:2209). Mọi v-for duyệt `editorSegments`, key `s.id`, có `v-memo`.
  - Số hàng `i+1` (:1846), `cellLabelOf(s.id,i)` (:1707) và mảng `v-memo` (:1841, :2021) đọc chỉ số `i` toàn cục ⇒ cần offset của khối.
  - Dải: `stripAnchorIndex` (:601) và `gridRow: idx+2` (:2063) là toàn cục; `.strip-pad` đứng ngay sau hàng neo trong mỗi cột (:1827, :1847, :1967, :2053).
  - Bảy listener nằm trên một `.col-tgt` (:1980-1987); để nguyên trên `.col-tgt` của từng khối.
  - Những chỗ coi `colTgt` là host duy nhất:
    - `onSelectionChange` (:1143)
    - focusout (:1044-1053)
    - `restoreEditedText` (:1080)
    - `repaintProofread`/`unwrapAllProofread` (:1097-1127)
    - watcher điều hướng `colTgt.value.querySelector` (:1255-1275)
  - Hai lời gọi `useSelectionSurface(colSrc|colTgt, …)` ở :651-652. Ref ở :596-598.
- `src/panels/selectionContract.ts`
  - `surfaces[]` mỗi mục một phần tử; đăng ký lại cùng phần tử thì thay mục cũ (:95). Khoá phải thành (el, role).
  - `surfaceFor` (:171) xét `el.contains(anchor)`.
  - `focusSelectionSource` (:455) chạy `selectNodeContents(source.el)` rồi collapse về đầu.
- `src/panels/proofreadHighlight.ts:104,124` -- `host.querySelector*`, nhận host từ GridPanel.
- `src/panels/segmentSelectionState.ts` -- vùng chọn nhiều câu của 4.9 là khoảng id (`Mod+Shift+↓/↑`), không phải `Range` DOM. Người dùng: `aiTranslateHandlers.ts:102`, `proofreadScope.ts:22`. Không đổi.
- `scripts/check-commands.mjs`
  - Kiểm F :1713-2018: `SELECTION_PANEL_FILES` GridPanel = 2 (:1754), regex :1814, quy tắc ① ⑤ ⑦, lý do ở :1732-1752.
  - Kiểm K :2543 là bảng handler chuột/phím.
- `scripts/check-layout.mjs` Kiểm C :410-500 -- danh sách API `window`/`document` được phép. Một API mới phải có mục ghi lý do.
- Test vitest giả định một `.grid` hoặc một `.col-*`:
  - `gridPanelImages.test.ts:152-162`
  - `gridPanelStripTrack.test.ts:69-107`
  - `gridCaretMoveCost.test.ts:297`
  - `gridRuleStable.test.ts:45`
  - `tmConcordanceSurfaces.test.ts:73,77`
- Test chi phí `gridTypingCost.test.ts` và `gridCaretMoveCost.test.ts` giữ nguyên điều kiện.
- e2e phụ thuộc cấu trúc:
  - `story-12-9-strip-row-track.e2e.mjs:116` (`~ .cell-tgt`)
  - `story-12-7-workspace-grid-status-bar.e2e.mjs:73-78` (một `.grid`)
  - `grid-image-row-alignment.e2e.mjs:60-75` (ghép `.col-* > *` theo chỉ số)
  - `hanviet-parallel-row-height.e2e.mjs:23` (ghi đè cột của `.grid`)
  - Helper dùng `[data-col][data-segment-id]` (`gridWait.mjs`, `panelReset.mjs`, `workspace.mjs`) nên an toàn.
- `_bmad-output/implementation-artifacts/2-15-ban-do/` -- bàn release: dựng, tiêm `bench-215.js`, gõ bằng `osascript` 10 phím/giây. Bàn dời con trỏ hiện có (`2-14-ban-do/do-doi-con-tro.e2e.mjs`) chạy bản debug, nên không dùng được cho tiêu chí 1.

## Tasks & Acceptance

Mỗi pha một agent mới; bàn giao qua `_bmad-output/implementation-artifacts/2-16-ban-do/HANDOFF.md`.

**Execution:**
- [ ] Pha 1 · `_bmad-output/implementation-artifacts/2-16-ban-do/`
  - Chép khuôn bàn 2-15 và thêm hai phép: dời con trỏ bằng chuột thật (sự kiện hệ điều hành), và phím `segment.next` (Story 2.10). Tính từ lúc gửi tới hết hai frame; nội suy giờ gửi như README 2-15. Ghi cả thời gian bố cục đầu (mở Workspace → ô đích đầu có mặt).
  - Chạy trên `baseline_revision` ở 9.850 câu, 3 lượt mỗi đường ⇒ số "trước" cho 2.14 tiêu chí 2. Commit bàn đo và số riêng.
- [ ] Pha 2 · `src/panels/gridBlocks.ts` (mới) + `GridPanel.vue`
  - Thêm `GRID_BLOCK_ROWS = 200` và hàm thuần chia `editorSegments` thành khối `{start, rows}`.
  - Template: `.grid` bọc các `.grid-block`; mỗi khối có năm cột subgrid riêng, cột lấy chung từ một biến CSS. Đưa offset `start` vào số hàng, nhãn và `v-memo`. Dải và `.strip-pad` theo khối.
  - Đổi các chỗ dùng `colTgt` làm host duy nhất sang gốc lưới cộng `[data-col="tgt"]`.
  - Dựng release và đo một phiên gõ 30 s cộng 3 lượt chuột và 3 lượt phím. Không đạt ⇒ `blocked`, trình số cho Ice, không commit `src/`.
- [ ] Pha 3 · `selectionContract.ts` + `GridPanel.vue` + `check-commands.mjs`
  - Bề mặt có phạm vi: tham số tuỳ chọn là bộ chọn cột. `surfaceFor` đòi anchor nằm trong `closest(scope)`; `focusSelectionSource` đặt caret ở đầu phần tử khớp đầu tiên.
  - Lưới gọi hai lần trên gốc lưới với `[data-col="src"]` / `[data-col="tgt"]`.
  - Kiểm F vẫn đếm 2. Ghi lại lý do, và bắt buộc hai lời gọi của lưới có phạm vi.
  - Kéo chọn trong cột nguyên văn qua ranh giới chỉ tô cột nguyên văn (quyết định tiêu chí 2).
- [ ] Pha 4 · test và e2e
  - `tests/frontend/gridBlocks.test.ts` (mới): các ca của ma trận, cộng ca mount 450 câu ⇒ 3 `.grid-block`, mỗi khối ≤ hằng, không cột nào trải cả Chương. Ca này đỏ khi gộp về một khối.
  - Sửa cách tìm hàng trong 5 test vitest và 4 spec e2e ở Code Map.
  - Thêm ca e2e:
    - điều hướng qua ranh giới;
    - Lookup ở khối 5;
    - ≤ 1 px quanh một ranh giới: cố định ở `grid-image-row-alignment` với ≥ 201 câu, đối chiếu một lượt trên `baseline_revision`;
    - kéo chuột thật trong cột nguyên văn qua ranh giới, khẳng định không ô nào ngoài cột nguyên văn bị tô;
    - `Mod+Shift+↓` qua ranh giới, khẳng định đủ id và phần tô.
  - Gỡ thật phần chia khối rồi chạy lại để xác nhận ca hình dạng đỏ.
- [ ] Nghiệm thu · bàn 2-16, chạy một lần
  - Đúng tiêu chí 1 của 2.15: 3 phiên mỗi cỡ, 122 và 9.850 câu, kèm `Mod+Enter`, p50 và p95.
  - Đúng tiêu chí 1 của 2.14: 3 lượt mỗi đường ở 9.850 câu.
  - Ghi số vào Implementation Notes và README bàn đo.
  - Ghi nợ `Chủ: Epic 2` vào `deferred-work.md`: Ice dùng thật một Chương lớn.

**Acceptance Criteria:** sáu tiêu chí của story, theo đúng câu chữ trong story.

## Implementation Notes

- Pha 1 (`5f2e1a9`): số "trước" ở 9.850 câu là bố cục đầu 4,4–4,7 s; chuột từ handler tới 2 frame 114–142 ms; `segment.next` 59–110 ms. Bàn 2-16 dùng hợp âm chữ vì macshot giữ hợp âm số.
- Pha 2, lượt 1 (`30049b3`, bản thử chưa vào `src/`, ở `2-16-ban-do/blocks-prototype.patch`):
  - Gõ đạt: phím cuối 55–84 ms, không phím chờ.
  - Trượt hai vế: chuột từ handler 234–763 ms, bố cục đầu 7,2–9,2 s.
  - Số chuột chưa sạch: có lượt bấm rơi vào lúc bố cục đầu chưa xong.
- Quyết định 2026-10-23, Ice: phương án B, gồm ba việc:
  - Sửa bản thử: danh sách khối ổn định, không phụ thuộc neo dải; chỉ khối chứa neo đổi.
  - Trước khi sửa, profile một lượt dời con trỏ trên release để biết chi phí nằm ở đâu.
  - Sửa bàn đo để chỉ bấm khi bố cục đầu đã xong thật. Rồi đo lại một lượt: 1 phiên gõ, 3 lượt chuột, 3 lượt phím, và bố cục đầu.
  - Không đạt thì lại dừng, trình số cho Ice, không commit `src/`.
- Pha 2, lượt 2 (bản thử ở `2-16-ban-do/blocks-prototype-2.patch`):
  - Dời con trỏ vẫn trượt. Chuột, từ mousedown tới hết JS: 174–356 ms so với 62–122 ms của bố cục cũ, đo xen kẽ cùng tải (loadavg 12–20).
  - Profile: chi phí là render/diff Vue của cả template lưới trong `setEditorCaret`; style/layout 0–1 ms; DOM chỉ đổi 6 bản ghi.
  - Neo dải không phải nguyên nhân.
- Quyết định 2026-10-23, Ice: chọn B2, tức profile tay bằng Safari Web Inspector trên hai bản release unminified (Base và `blocks-prototype-2.patch`, bật tính năng `devtools` của tauri chỉ trong worktree). Sau đó Ice tạm dừng để điều tra lại gốc chi phí render trong một phiên khác. Bản dựng dừng giữa chừng, chưa có bản ghi timeline nào.
- Gốc chi phí render (nghiên cứu mã `research-grid-render-on-caret-move/`, §3 và §Where the answers meet): bốn vòng hàng có `v-memo` nằm trong vòng khối nên 50 khối dùng chung bốn ô `_cache`; khối k nhận vnode của khối k−1, phép so khoá trượt, mọi hàng dựng lại mỗi lượt dời con trỏ. Mô hình Vue 3.5.40 prod trên happy-dom (`2-16-ban-do/memo-probe.cjs`, 9.850 câu): bản thử dựng lại 9.850 hàng mỗi lượt, bố cục cũ 2, mỗi khối một component nhận con trỏ qua prop 2 hàng.
- Quyết định 2026-10-23, Ice: bản thử thứ ba là C2, mỗi khối một component, các giá trị đổi theo con trỏ (con trỏ, neo dải, vùng chọn) vào khối qua prop riêng cho khối; đo lại trên release bằng bàn 2-16 và 2-15, xen kẽ với cây gốc.
- Bản thử 3 trên release (`0b619fb` + `blocks-prototype-3.patch`, số ở `93bc97b`, 9.850 câu, xen kẽ 3 phiên mỗi bên): gõ đạt, phím cuối 65–72 ms; dời con trỏ trượt ≤ 50 ms ở cả hai bố cục, vì phần JS còn 1–3 ms (base 41–49) nhưng phần sau JS tới hai frame 53–66 ms không đổi, `segment.next` 58–76 ms. Quyết định 2026-10-24, Ice: nhận C2 dù vế dời con trỏ của tiêu chí 1 trượt, làm tiếp Pha 3 và Pha 4 trên mã bản thử 3; phần sau JS thuộc bug 2-14, bug ấy vẫn mở với số này.
- Kéo chuột thật trong cột nguyên văn qua ranh giới khối (`7c9a3a4`): cả hai cách của §Design Notes đều trượt, vì vùng chọn là một khoảng liền theo DOM và cột bản dịch khối 0 nằm giữa hai cột nguyên văn. Quyết định 2026-10-24, Ice: cách thứ ba, chỉ tô và chỉ đọc cột nguyên văn (Range vẫn chứa ô khác); ca kéo dùng chuột hệ điều hành nằm ở bàn cục bộ `2-16-ban-do/`, không ở nightly.
- Nghiệm thu (release `24bb1e2`, 9.850 câu trừ khi ghi khác, loadavg 2,4–4,7; bảng ở `2-16-ban-do/HANDOFF.md` §Nghiệm thu): gõ ở 122 câu đạt mọi vế của 2.15 tiêu chí 1; ở 9.850 câu không còn phím chờ, phím cuối 72 · 77 · 187 ms, p95 44–54 ms, nhưng `Mod+Enter` tới xác nhận 164–194 ms (> 100), và đợt bản thử 3 đã là 147–164 ms. Dời con trỏ: chuột handler → 2 frame 59–88 ms, `segment.next` gửi → 2 frame 62–89 ms; bố cục đầu 3,70–3,82 s.
- Quyết định 2026-10-24, Ice: truy và sửa vế `Mod+Enter` ở 9.850 câu ngay trong story này, rồi nghiệm thu lại.
- Pha 5 (`f62012f`): một lượt `Mod+Enter` từ 6 khối render xuống 1–2; release 9.850 câu 183–216 → 95–146 ms. Profile (`d6bb53f`): chờ phím cuối 55–71 ms, rồi ~55–70 ms JS tăng theo số câu, frame chỉ 3–17 ms. Quyết định 2026-10-24, Ice: cắt tiếp phần JS đó trong story này.

## Plan Change Log

## Review Triage Log

## Design Notes

- Bề mặt có phạm vi thay vì N bề mặt.
  - Một bề mặt trên gốc lưới cho mỗi vai giữ hai lời gọi literal mà Kiểm F đếm, và vẫn đúng khi khối được thêm hay bớt.
  - N bề mặt sẽ phải đăng ký động trong v-for, mà quy tắc ①/⑤/⑦ cấm.
- Khối chia theo chỉ số, key theo số thứ tự khối.
  - Một lượt tách hay gộp đẩy một hàng qua mỗi ranh giới phía sau, và hàng ấy dựng lại: ~49 hàng ở 9.850 câu.
  - Hàng đang gõ không bao giờ là hàng bị đẩy, nên `⌘Z` của ô đang gõ còn nguyên. Ca ma trận "Tách ở hàng 199" giữ điều này.
- Giữ kéo cột nguyên văn trong cột: có hai cách.
  - CSS: khi lượt kéo bắt đầu trong `.col-src`, đặt `user-select: none` lên các cột khác.
  - JS: chặn đầu `focus` của vùng chọn trong `selectionchange`.
  - Ca e2e kéo chuột thật quyết định. Nếu cả hai cùng đạt, trình cả hai cho Ice kèm số đo; không tự chọn.

## Verification

**Commands:**
- `npm run test:story 2-16` -- xanh; cùng các tệp vitest đã sửa.
- `npm run check:commands && npm run check:layout` -- xanh.
- Pha 4: các spec e2e của lưới, chạy ngoài sandbox -- xanh.
