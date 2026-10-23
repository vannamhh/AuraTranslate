---
title: 'Dời con trỏ chậm ở Chương lớn'
type: 'bugfix'
ticket: 14
created: '2026-10-22'
status: in-progress
baseline_revision: 'dab86d297681f86ef0022c54bab3f931ce295a19'
route: 'full'
route_source: 'auto'
risk: 'medium'
review: ''
review_source: ''
lenses_ran: []
review_loop_iteration: 0
context: []
blocked_at: "2026-10-23"
blocked_reason: "Ice 2026-10-23: ngừng sửa lưới, đo lại sau"
---

<frozen-after-approval reason="human-owned intent — do not modify unless human renegotiates">

## Intent

**Problem:** Ở 9.850 câu một lượt dời con trỏ mất ~720 ms. Đo tách (Implementation Notes) cho hai nguồn: ~390 ms là WebKit tính lại bố cục cả lưới `subgrid` vì `v-if` của vạch lề CHÈN một `div.rule` vào hàng nhận con trỏ (`GridPanel.vue:1810`); ~225 ms là `PanelFrame` dựng lại cả slot lưới vì template đọc giá trị theo con trỏ trên từng hàng. Đổi lớp, đổi `aria-label` và gỡ phần tử không tốn.

**Approach:** Dời con trỏ chỉ được đổi thuộc tính, không chèn phần tử vào lưới: phần tử vạch lề luôn có mặt ở mọi hàng, trạng thái đi bằng lớp. Làm lại hướng A (`v-memo` + tách vế con trỏ khỏi `ruleById`) để Vue chỉ dựng lại hai hàng. Đo `.strip-pad` (cùng kiểu chèn ở bốn cột khi dải mở) và sửa cùng cách nếu đắt. Nghiệm thu trên bản dựng thật.

**Decisions:**
- 2026-10-22, Ice giao quyền chọn hướng: chọn A kèm cổng số. A đã làm, đo, bỏ (không lợi ích đo được vì khối ~390 ms che).
- 2026-10-22, Ice giữ plan nguyên độ dài, không tách bàn đo ra plan riêng.
- 2026-10-22, Ice duyệt hướng mới sau sáu phép đo: (1) vạch lề luôn có mặt; (2) đo và sửa `.strip-pad` cùng khuôn nếu đắt; (3) làm lại A, có ca test riêng cho từng giá trị trong mảng `v-memo`, kể cả lỗi hàng, Glossary, `unlinked`; (4) đo trên bản dựng thật; còn > 50 ms thì dừng `blocked`, trình Ice số, không nới ngưỡng. Mốc sàn của phép đo "tới hết hai frame" trên bản sao không app ở bản debug là 54–60 ms.
- 2026-10-23, Ice chọn hình vạch mới thay bộ 2026-10-14: mọi vạch dày 3 px, chỉ khác kiểu nét và màu (`primary` liền, `confirmed` đôi, `draft` đứt, `tm-rule` chấm), ô không vạch giữ viền 3 px trong suốt; DESIGN.md mang 🔵 2026-10-23. Bị loại: gradient nền (phạm `src/AGENTS.md` và Kiểm F), vạch tách khỏi bố cục bằng `position: absolute` (chậm hơn). "Đúng vạch như trước" trong Matrix nghĩa là đúng trạng thái, hình theo DESIGN.md mới.

## Boundaries & Constraints

**Always:** Giữ mọi hàng trong DOM (e2e và `gridWait.mjs` đếm/đánh chỉ số mọi hàng; `grid-image-row-alignment` đòi các cột cùng số con). Trạng thái "không vạch" không hiện gì. Template vẫn dựng `s.target_text`, DOM giữ chữ đang gõ (`GridPanel.vue:1050-1072`). `focus()` của watcher đặt con trỏ vẫn là đường cuộn duy nhất, không `preventScroll`. Đúng 2 lời gọi `useSelectionSurface` (Check F). Mỗi số đo ghi bản dựng, phiên bản WebKit, số câu, tải máy.

**Never:** Không đẩy việc cập nhật vạch lề ra sau frame (idle/`setTimeout`). Không đụng đường gõ ngoài việc đo nó (thuộc 2.4). Không nới ngưỡng hay sửa điều kiện spec e2e cho xanh. Không thêm bàn đo vào `package.json`. Không ảo hoá hàng.

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected Output / Behavior | Error Handling |
|----------|--------------|---------------------------|----------------|
| Bấm chuột | 9.850 câu, bấm ô bản dịch câu khác | ≤ 50 ms tới hết hai frame; vạch `primary` đổi cùng lượt vẽ; không phần tử nào được chèn vào lưới | — |
| Phím điều hướng | `Mod+Alt+ArrowDown` (`editor.next_untranslated`; `next_segment` không có phím mặc định) | ≤ 50 ms; ô đích nhận focus và cuộn vào | — |
| Câu đang chờ ghi | gõ ở câu A rồi dời sang B | chữ của A còn, vạch A theo chữ đã gõ | — |
| Vạch lề các trạng thái | câu rỗng/đã dịch/đã xác nhận/TM điền/có con trỏ | đúng vạch như trước; "không vạch" không hiện gì | — |
| Vùng chọn nhiều segment | chọn trọn Chương rồi dời con trỏ | lớp tô vùng chọn giữ nguyên | — |
| Dải Glossary/Proofreader mở | `stripAnchorSegmentId` theo con trỏ | dải đi theo câu mới; số đo ghi riêng | — |

</frozen-after-approval>

## Code Map

- `src/panels/GridPanel.vue:1810` -- `<div v-if="ruleClassById.get(s.id)" class="rule">`: chỗ chèn đắt; CSS `.rule` ở `:2259-2290`.
- `src/panels/GridPanel.vue:1812, 1828, 1932, 2006` -- `.strip-pad` `v-if` ở bốn cột theo `stripAnchorSegmentId`; CSS `:2174`. `stripAnchorIndex` (`:595`, `src/inlineStripEligibility.ts:61-73`) cũng đổi `:style` lưới (`:1797`).
- `src/panels/GridPanel.vue:279-298`, `:1696-1703`, `:1805-2007` -- `ruleById`/`ruleClassById`, `cellLabelOf`, bốn `v-for`. Lưới là slot của `PanelFrame` (`:1729`), nên mọi đọc phản ứng trong thân lưới bắt `PanelFrame` dựng lại cả slot.
- Diff của hướng A đã bỏ (đọc lại bằng chữ, không `git checkout`): `restRuleById`/`restRuleClassById`, `ruleOf`/`ruleClassForRow`, `v-memo` bốn cột, `NO_IMAGES` dùng chung (vì `?? []` mới mỗi lượt làm `v-memo` luôn trượt), concordance viết thẳng trong mảng memo (vue-tsc không tính `v-memo` là dùng hàm).
- `_bmad-output/implementation-artifacts/2-14-ban-do/` -- `do-doi-con-tro.e2e.mjs` (bấm, phím, gõ), `phat-lai-dot-bien.e2e.mjs` (ghi đột biến trong app + phát lại trên bản sao). Bản dựng thật: khuôn `../2-4-ban-do/build-bench.sh` (release nguyên vẹn + script đo tiêm vào `dist`; e2e không lái được release vì plugin WebDriver chỉ có ở debug).
- `tests/frontend/gridPanel*.test.ts`, `tests/frontend/support/segmentFixture.ts:42` -- 14 tệp mount GridPanel, fixture 1–10 câu.

## Tasks & Acceptance

**Execution:**
- [ ] `src/panels/GridPanel.vue` -- phần tử vạch lề luôn có mặt ở mọi hàng, trạng thái bằng lớp; "không vạch" không hiện -- Intent.
- [ ] `tests/frontend/gridRuleStable.test.ts` -- dời con trỏ giữa hai hàng thì cột vạch lề không có phần tử nào được thêm/gỡ (đếm bằng `MutationObserver` gom bản ghi qua callback), vạch `primary` đi theo con trỏ, mỗi trạng thái cho đúng lớp; đỏ trên `baseline_revision`.
- [ ] Đo `.strip-pad` (dải mở, dời con trỏ) bằng bàn đo; nếu > ~60 ms thì sửa cùng khuôn (phần tử luôn có mặt hoặc không chèn vào lưới) kèm ca test như trên; nếu không, ghi số.
- [ ] `src/panels/GridPanel.vue` + `tests/frontend/gridCaretMoveCost.test.ts` -- làm lại A theo Code Map; mỗi giá trị trong mảng `v-memo` một ca đổi nó và thấy ô đổi (kể cả lỗi hàng, Glossary, `unlinked`); ca đếm ở 20 và 200 câu đỏ trên `baseline_revision`.
- [ ] Đo bằng `do-doi-con-tro.e2e.mjs` (bản debug) trước và sau, cùng phiên máy; ghi số.
- [ ] Đo trên bản dựng thật theo khuôn `build-bench.sh`, script đo dời con trỏ tiêm vào `dist`; không dựng được thì ghi lý do và mở mục nợ `Chủ: Epic 2`.
- [ ] `_bmad-output/initiative-auratranslate/deferred-work.md` -- đóng hoặc 🟡 mục "TASK 8 ĐÃ ĐO" bằng chữ kèm số.

**Acceptance Criteria:**
- Given 9.850 câu trên bản dựng thật macOS, when dời con trỏ bằng chuột và bằng `Mod+Alt+ArrowDown`, n ≥ 3 mỗi đường, then mỗi lượt ≤ 50 ms tới hết hai frame; không đạt ⇒ `blocked` kèm số.
- Given bộ test, when `gridRuleStable.test.ts` và `gridCaretMoveCost.test.ts` chạy, then xanh, và gỡ từng bản sửa thì ca của nó đỏ.
- Given các spec e2e của lưới (`segment-navigation`, `grid-empty-cell`, `grid-image-row-alignment`, `grid-row-error-label`, `story-12-9-strip-row-track`, `hanviet-parallel-row-height`, `proofread-underline`), when chạy, then xanh không sửa điều kiện.

## Implementation Notes

- Số trước (`baseline_revision`, WKWebView 605.1.15, macOS, bản debug nạp Vite dev, 9.850 câu, máy rảnh, bỏ lượt đầu): bấm chuột (mousedown tới hết 2 frame) 688/661/733 ms (một lượt rỗng do chờ 400 ms ngắn hơn lượt đo, nâng lên 1500 ms); `Mod+Alt+ArrowDown` 677/735/777/709; một phím gõ 761/661/711/762.
- Số sau A (`v-memo` ở bốn `v-for` + vế con trỏ gấp vào lúc tra): bấm 663/634/645 (lượt thứ tư 1966 do máy), `Mod+Alt+ArrowDown` 1348/669/627/711, gõ 728/611/677/795. Cổng > 50 ms ⇒ dừng ở `blocked`, không chuyển sang ảo hoá.
- Sonde cùng phiên: `setPosition` vào ô bản dịch `contenteditable` ~600 ms, vào ô nguyên văn 27–34 ms, đổi class thuần 33 ms (nhịp rảnh), JS đồng bộ 0–14 ms, một lượt Vue cập nhật 1 ms; `spellcheck=false` không đổi (497–620 ms). Chi phí ở WebKit khi vùng chọn vào một trong 9.850 vùng sửa được, không ở Vue. Thử `contain`, `content-visibility`, `-webkit-user-modify` cộng dồn nên chưa kết luận được.
- Mã: `GridPanel.vue` đã sửa (`restRuleById`/`restRuleClassById` không phụ thuộc con trỏ, `ruleOf`/`ruleClassForRow`, `v-memo` ở bốn `v-for`, `NO_IMAGES` dùng chung vì `?? []` mới mỗi lượt làm `v-memo` luôn trượt). Lợi ích đo được: không. Giữ hay bỏ là việc của Ice.
- `tests/frontend/gridCaretMoveCost.test.ts`: đỏ trên mã cũ (200 so với 20 lượt phân giải), xanh sau sửa; gỡ riêng vế concordance khỏi `v-memo` thì ca concordance đỏ. Chưa có ca cho lỗi hàng, Glossary, `unlinked` qua cổng riêng.
- Gõ ở 9.850 câu cũng ~700 ms: mục nợ mới, `Chủ: Story 2.4`.
- Đối chứng gỡ vùng sửa (`doi-chung-vung-sua.e2e.mjs`, bản debug nạp Vite dev, WKWebView 605.1.15, tải 5,1/6,7/7,1 lúc 16:47): lượt 1 KHÔNG hợp lệ ngoài Control A `setPosition` 626/739/770/783 ms (trung vị 770) — các ca sau đỏ vì mocha hết 120 s, các ca chồng lên nhau trong cùng phiên, `cv` 5,7–7,1 s là lúc màn hình đã khoá. Số `go`/K100/K1000/K5000/B trong log lượt đó không dùng được.
- Lượt 2 và 3 (sau khi vá trần mocha và gộp mẫu `setPosition` vào một `execute`) đỏ ở `before` vì rAF không chạy: `CGSSessionScreenIsLocked = true`. Chưa có số nào cho gỡ `contenteditable`; cần màn hình mở khoá rồi chạy lại.
- Đối chứng gỡ vùng sửa (Ice chạy, 2026-10-22 14:37–14:45 UTC, màn hình mở, bản debug nạp Vite, WKWebView 605.1.15, 9.850 câu, cùng một phiên, nạp lại giữa các biến thể; tải máy không ghi): A `setPosition` 668/756/667/751 (trung vị 751), A bấm thật 661/768/630/736; gỡ `contenteditable` còn 3 ô `setPosition` 674/717/666/717, bấm 664/720/639/764; giữ K=100 710, K=1.000 750, K=5.000 683 (trung vị); chỉ `content-visibility: auto` 4.568–7.133 ms; B `setPosition` 699/761/621/701, bấm 668/714/728/687.
- Giả thuyết "chi phí theo số vùng sửa được" bị bác bằng phép gỡ thật: còn 3 ô sửa được vẫn ~700 ms, không đổi theo K. `content-visibility: auto` làm chậm thêm ~8 lần. Hướng "chỉ ô có con trỏ mới sửa được" vì thế không trúng.
- Bàn tách pha (`tach-pha.e2e.mjs`, 2026-10-22 15:00–15:04 UTC, màn hình mở, bản debug nạp Vite dev, WKWebView 605.1.15, 9.850 câu, DOM đầy đủ, n = 6 mỗi phép, load 4,59/4,77/4,52 trước và 4,24/4,52/4,48 sau; JSON `ket-qua-tach-pha-2026-10-22T15-04-05.101Z.json`).
- Trung vị ms sync/handlers(tSel)/toFrame1/frame1/total: ô bản dịch `focus`+`setPosition` 3/250/639/88/717 (đối chứng trôi cuối phiên 3/262/653/93/742); ô nguyên văn `setPosition` 0/0/0/22/22; bấm thật (t0 = mousedown bắt ở `window`) -/699/654/63/712 (tSel đến muộn, sau cả rAF thứ nhất ở nhiều mẫu, nên handlers không so được với hàng khác). `setTimeout(0)` đặt ở t1 chạy ở ~697 ms (setPosition), tức luồng chính bận tới ~700 ms; ở bấm thật ~264 ms vì mốc đó đặt ở mousedown, không cùng ý nghĩa.
- Kết luận: ~640 trong ~720 ms nằm trước rAF thứ nhất (đồng bộ chỉ 3 ms; selectionchange tới sau ~250 ms, rồi còn ~390 ms nữa trước frame), còn style/layout/paint của frame đầu chỉ ~90 ms; phép đo này chưa tách được phần ~640 ms giữa JS ứng dụng và công việc nội bộ WebKit của vùng chọn.
- Đo Vue dev tracing (`config.performance`, quan sát bằng `PerformanceObserver` vì Vue xoá measure ngay), 9.850 câu, WKWebView debug nạp Vite dev, load 3,98 → 3,19 (`do-vue.e2e.mjs`, `ket-qua-do-vue-2026-10-22T15-07-43.747Z.json`): mỗi lần dời con trỏ chỉ có `<PanelFrame>` render ×1 (trung vị 171–191 ms) + patch ×1 (54–55 ms) và `<StatusBar>` ×1 (0 ms); `<GridPanel>` không render lại; n = 6 (setPosition) và n = 3 (bấm thật).
- Tổng / hợp khoảng Vue / phần ngoài Vue (trung vị): setPosition 740 / 223 / 512 ms; bấm thật 707 / 246 / 461 ms. Chỉ MỘT cụm render, toàn bộ trước `selectionchange` (render ~25 ms, patch ~210 ms sau mốc đầu): giả thuyết `setEditorCaret` chạy hai lần kéo hai lượt re-render toàn lưới KHÔNG được số đo ủng hộ.
- Kết luận: Vue chiếm ~30% thời gian, còn ~60–70% nằm ngoài render/patch của component (chưa quy được: style/layout/paint của WebKit hoặc JS ngoài component). Thân lưới được dựng trong render của `PanelFrame` (slot), nên `PanelFrame` là nơi cần soi tiếp; nguyên nhân kích nó chưa được đo. Cờ performance tự thêm mark/measure nên số tuyệt đối có thể phồng nhẹ.
- Quy khoảng trống (`quy-khoang-trong.e2e.mjs`, 2026-10-22 15:11–15:13 UTC, màn hình mở, bản debug nạp Vite dev, WKWebView 605.1.15, 9.850 câu, n = 5 mỗi biến thể sau khi bỏ lượt đầu, tải 4,82 trước / 4,13 sau; JSON `ket-qua-quy-khoang-trong-2026-10-22T15-13-19.086Z.json`): gắn dụng cụ vào đọc layout (`getBoundingClientRect`, `getClientRects`, `getComputedStyle`, `getPropertyValue`, `scrollIntoView`, `focus`, getter `offset*`/`client*`/`scroll*`), `invoke` của Tauri, `setTimeout`/rAF/idle và handler `focus*`/`blur`/`mouse*`/`selectionchange`, kèm vòng ping `MessageChannel`. Trung vị `t0→rAF1`: setPosition 644 ms, bấm thật 639 ms; Vue render/patch 229 / 224, đọc layout 3 / 2 (chỉ `focus()`), callback timer 0, IPC 0 (không có lệnh `invoke` nào trong cửa sổ), handler sự kiện chồng lên Vue (selectionchange 3→256 ms), không có gì phủ 397 / 387 ms (~60%).
- Chủ thể đáng kể: `focus()` 3 ms (`GridPanel.vue:788`, 1 lần/lượt), callback timer ≤ 1 ms (`editorPanelState.ts:548`, `tmFuzzyStripState.ts:105`, `GridPanel.vue:788/919`, rAF `GridPanel.vue:897`, `onBlur` của dockview-vue), không phát hiện đọc layout nào > 1 ms. Một khối chặn liền mạch từ t0 tới sau rAF1 (mẫu giữa của setPosition: 0–691 và 732–752 ms; bấm thật 0–609): `mouseup` tới ở 573 ms thay vì ngay sau `mousedown`, `selectionchange` của setPosition xong ở 256 ms rồi ~300–390 ms không có JS nào của ứng dụng chạy tới frame 1. Số lượng trích từ stack của bản biên dịch Vite nên dòng của `.vue` là dòng sau biên dịch.
- Đối chứng dụng cụ: tổng setPosition gắn 742 ms so với gỡ 710 ms (r1 644 so với 615), chênh ~30 ms (~4%), hai biến thể chạy nối tiếp không xen kẽ; bấm thật chưa có đối chứng gỡ.
- Kết luận: khoảng trống KHÔNG nằm ở đọc layout đồng bộ, IPC hay timer của ứng dụng; nó là công việc luồng chính không do JS ứng dụng gọi (render/style/layout nội bộ WebKit, hoặc JS không có hook như microtask/Vue watcher), chưa được đặt tên bằng phép đo này; bước kế là profile gốc (Web Inspector Timelines) hoặc phép gỡ có đối chứng.
- Đối chứng môi trường (2026-10-22 22:16–22:20 giờ máy, Ice thoát OpenKey và Raycast; còn `OpenKeyHelper` và `RaycastAppIntents`; tải 5,09 trước, 3,44 sau; `tach-pha.e2e.mjs`, `ket-qua-tach-pha-2026-10-22T15-20-02.065Z.json`): ô bản dịch tổng 752 / 726 ms (trước đó có OpenKey: 717 / 742), ô nguyên văn 24, bấm thật 721. Giả thuyết "bộ gõ hay ứng dụng Accessibility gây quãng ~390 ms" bị bác cho OpenKey và Raycast.
- Bản sao tách app (`ban-sao-tach-app.e2e.mjs`, 2026-10-22 22:22–22:34 giờ máy, bản debug nạp Vite dev, WKWebView 605.1.15, 9.850 câu, n = 6 mỗi biến thể sau khi bỏ lượt đầu, tải 4,03 trước / 4,54 sau; `ket-qua-ban-sao-2026-10-22T15-34-20.775Z.json`): nhân bản `#app` (49.446 nút), `unmount()` Vue, thay bằng bản sao, giữ nguyên `<style>` ở `<head>`; `data-v-*` còn, `display` của `.grid` vẫn `grid`, 0 mutation khi đo. Đối chứng trong ứng dụng toFrame1 677 / total 748 ms; bản sao `focus()` + `setPosition` ở ô đích toFrame1 3 / total 69 ms (lặp cuối 70), ô nguồn total 19 ms.
- Bisect CSS trên bản sao (median total): grid→block 54, `.col` block 32, ẩn ba cột phụ 29, cuộn `visible` 64 / `hidden` 70, `user-select` chỉ cột đích 67, `white-space: normal` 67, bỏ viền ô 69, `.panel` overflow visible 69, lưới đứng riêng trong `<body>` 25 (kèm `.grid-scroll` 30). Không biến thể nào đáng kể so với bản sao nguyên (~69 ms); chỉ bỏ DOM phụ/cột (29, 25) hạ thêm.
- Kết luận: khối ~390 ms KHÔNG tái hiện khi gỡ Vue/ứng dụng khỏi cùng DOM + CSS (còn ~69 ms, trong đó ~66 ms ở frame đầu) nên không phải chi phí WebKit thuần của DOM này; thứ gây ra nằm ở trạng thái ứng dụng sống mà bản sao mất (nghi: listener, thuộc tính JS trên nút, ResizeObserver/IntersectionObserver, trạng thái Vue/dockview), bản sao chưa tách được cái nào.
- Bàn phát lại đột biến (`phat-lai-dot-bien.e2e.mjs`, 2026-10-22 22:44–22:49 giờ máy, màn hình mở, bản debug nạp Vite dev, WKWebView 605.1.15, 9.850 câu, n = 6 sau khi bỏ lượt đầu, tải 4,90 trước / 9,84 sau; JSON `ket-qua-phat-lai-2026-10-22T15-49-22.326Z.json`; lượt đầu cùng mã trừ canary: 0 mutation, tải 4,15 → 5,27, `ket-qua-phat-lai-2026-10-22T15-43-38.915Z.json`).
- Pha 1 (MutationObserver trên `document`, subtree + attributes + childList + characterData, 3 lần `focus()` + `setPosition` vào ô bản dịch, B-A-B): 0 mutation ở cả ba lần, mỗi lần chờ 1.200 ms rồi `takeRecords()`; canary (đặt rồi gỡ một thuộc tính) cho đúng 2 bản ghi nên observer chạy được. Tổng trong ứng dụng 737/695/812 ms. Vue patch ~210 ms mỗi lần dời con trỏ vì thế KHÔNG ghi gì vào DOM.
- Pha 2: vì không có mutation nào nên R1–R3 phát lại 0 thao tác (`n_ops` = 0) và R4.. (từng nhóm) không có nhóm nào để chạy; trung vị toFrame1/total: R0 5/60, R1 7/61 (cùng thao tác với R0), R2 (vùng chọn ở ô nguyên văn) 0/18, R3 (không vùng chọn) 0/20, R-last 7/59. R0 tương đương lượt bản sao trước (~69).
- Kết luận: giả thuyết "mutation DOM khi vùng chọn nằm trong host sửa được ép WebKit làm layout đồng bộ" bị bác ở tiền đề, vì Vue không tạo mutation nào ở lần dời con trỏ (đo 3 lần, có canary); khối ~390 ms phải nằm ở thứ khác bản sao mất, không phải đột biến DOM (nghi vẫn là listener/observer/trạng thái dockview gắn vào nút sống).
- 🔵 2026-10-22 — ba dòng trên (Pha 1 "0 mutation", Pha 2, Kết luận) không có giá trị: `MutationObserver` của spec mang callback rỗng nên bản ghi giao qua callback bị vứt, `takeRecords()` sau 1.200 ms chỉ còn bản ghi chưa giao; canary đọc đồng bộ nên vẫn ra 2. Spec đã sửa (gom bản ghi trong callback, canary đọc sau 50 ms) và chạy lại.
- Phát lại đột biến, lượt sửa (`phat-lai-dot-bien.e2e.mjs`, 2026-10-22 22:50–23:02 giờ máy, tải 5,69 trước, 3,86 sau, canary qua callback = 2, `ket-qua-phat-lai-2026-10-22T16-02-34.740Z.json`): mỗi lượt dời con trỏ Vue tạo 9 đột biến trên 3 hàng: `+editing`/`-editing` ở ô bản dịch, 2 `aria-label`, ở cột vạch lề gỡ một `div.rule.rule-primary` và chèn một `div.rule.rule-primary` (kèm cặp `#comment` của `v-if`), một `#text`.
- Trên bản sao (không app): không phát lại 60 ms; phát lại trọn 9 đột biến 493 ms (vùng chọn ở ô bản dịch), 428 (ở ô nguyên văn), 428 (không vùng chọn). Từng nhóm riêng, vùng chọn ở ô bản dịch: chỉ **chèn `div.rule.rule-primary` vào cột vạch lề** cho 447 ms; gỡ `div.rule` 58, `±editing` 58–63, `aria-label` 54, cặp `#comment` 55–62, `#text` 69; đo lại cuối phiên 54.
- Kết luận: khối ~390 ms là chi phí WebKit khi một phần tử mới được chèn vào một cột của lưới `subgrid` 9.850 hàng (`v-if` của vạch lề ở hàng nhận con trỏ); không cần vùng chọn trong ô sửa được. Đổi lớp không tốn. Phần còn lại ~225 ms là `PanelFrame` dựng lại slot. Mốc sàn của phép đo trên bản sao không app là 54–60 ms ở bản debug.
- Sau bản sửa (vạch lề luôn có mặt + `v-memo`), `do-doi-con-tro.e2e.mjs` 2026-10-22 23:12–23:15 giờ máy, tải 3,9–5,4, bản debug: bấm 607/607/732/741 ms, `Mod+Alt+ArrowDown` 638/661/695/727, gõ 577–760. Chưa cải thiện. Nghi vấn chưa đo: lớp `rule-primary` vẫn đổi bề dày `border-left` (3 px so với 2 px/không), tức vẫn đổi hình học trong lưới.
- Bàn biến thể CSS (`bien-the-css-vach.e2e.mjs`: đối chứng, viền đều 3 px, chỉ tô bằng `box-shadow: inset`) chưa có số: ba lượt 23:17–23:25 đỏ ở `before` (hai lượt hết trần mocha 120 s, lượt ba `execute/async` treo trong `langLai`, tức rAF không chạy). Ice đang dùng máy lúc đó; đo lại khi máy rảnh.
- Bàn biến thể CSS trên mã đã sửa (`bien-the-css-vach.e2e.mjs`, bản debug, 9.850 câu, bấm thật, n=5, tải 3,2–4,6). Lượt 23:47–23:56: đối chứng 692, viền đều 3 px chỉ đổi màu 170, `box-shadow: inset` 681, đối chứng lại 690. Lượt 23:58–00:08: đối chứng 696, viền 3 px giữ kiểu nét 180, **gradient nền giữ 2/3 px, chấm, gạch 162**, viền đều 3 px 155, đối chứng lại 635. Kết luận: đổi bề dày viền (và `box-shadow`) bắt WebKit tính lại bố cục cả lưới; đổi nền thì không. Còn ~100 ms trên mốc sàn ~60 ms chưa rõ nguồn. Chương đo toàn câu chưa dịch nên chỉ thử chuyển "không vạch" ↔ `primary`.
- Biến thể vạch tách khỏi bố cục (`::before` `position: absolute` trong ô vạch, ô `position: relative`, giữ 2/3 px), 2026-10-23 07:02–07:14 giờ máy, cùng điều kiện, `ket-qua-bien-the-css-c.txt`: đối chứng 699, tách bố cục 1.033, viền đều 3 px 164, tách bố cục lại 955, đối chứng lại 683. Chậm hơn mã hiện tại nên bị loại. Viền đều 3 px như đã đo cho `confirmed` và `primary` cùng nét liền 3 px, phạm WCAG 1.4.1 và Kiểm I của `check:commands`, nên `confirmed` thành nét đôi.

## Plan Change Log

- 2026-10-22 — sáu phép đo bác giả thuyết "Vue dựng lại toàn lưới là toàn bộ chi phí": ~390 ms là WebKit dựng lại bố cục vì `v-if` của vạch lề chèn phần tử vào lưới `subgrid`, ~225 ms là `PanelFrame` dựng lại slot. Ice duyệt viết lại Intent/Approach/Boundaries/Matrix; Code Map và Tasks dựng lại theo đó. GIỮ: diff của A (Code Map) và bàn đo `2-14-ban-do/`; tránh: `MutationObserver` callback rỗng (vứt bản ghi), canary đọc đồng bộ.
- 2026-10-23 — đổi bề dày viền trong lưới cũng ép tính lại bố cục cả lưới như chèn phần tử; Ice chọn hình vạch cùng 3 px (Decisions). Thêm ca `gridRuleStable` giữ bề dày mọi khối `.rule-*` bằng ô trống; e2e 12.7 đổi câu kiểm nét `confirmed`/`draft` theo hình mới.

## Review Triage Log

## Verification

**Commands:**
- `npx vitest run tests/frontend/gridRuleStable.test.ts tests/frontend/gridCaretMoveCost.test.ts` -- expected: xanh; gỡ từng bản sửa thì ca của nó đỏ.
- `npx vitest run tests/frontend/gridPanel*.test.ts tests/frontend/gridRestoreEditedText.test.ts tests/frontend/segmentSelection.test.ts` -- expected: xanh.
- `TAURI_WEBDRIVER_PORT=4467 perl -e 'alarm shift; exec @ARGV' 2400 npm run test:e2e -- --spec _bmad-output/implementation-artifacts/2-14-ban-do/do-doi-con-tro.e2e.mjs` -- ngoài sandbox, màn hình mở; ghi số.
- `npm run test:e2e -- --spec e2e/specs/<spec lưới>.e2e.mjs` cho từng spec trong AC thứ ba -- expected: xanh.
