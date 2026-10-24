# Handoff Pha 1 -> Pha 2 (story 2-16)

Pha 1 xong. Bàn đo + số "trước" ở thư mục này (README có bảng số). Không đụng `src/`.

## Dùng lại bàn đo cho Pha 2 (đo bản thử chia khối)
- Dựng: `./build-216.sh <repo> <out>` (~11 phút, release). Chạy: `./run-move.sh <nhãn> <out> <122|9850>` (ngoài sandbox, màn hình mở khoá, KHÔNG chạm máy khi chạy). 3 phiên mỗi cỡ cho so sánh; `python3 analyse.py ket-qua-*.json`.
- Bàn này chỉ đo bố cục đầu, chuột thật và `segment.next`. Phiên gõ 30 s là bàn `../2-15-ban-do/` (`build-215.sh`, `run-session.sh`).
- 🔴 Hợp âm số (Ctrl+Alt+Shift+1..5) của bàn 2-15 đụng phím chụp màn hình của macshot (mã 19/20/21/23, đang chạy trên máy này): một lần phím 4 làm mất tiêu điểm cửa sổ app. Bàn 2-16 đã chuyển sang Ctrl+Alt+Shift+Q/W/E/R/T/U. Trước khi chạy `run-session.sh` 2-15 phải đổi hợp âm tương tự (trong `bench-215.js` và `run-session.sh`) hoặc tắt macshot; bản 2-15 chưa sửa.
- `segment.next` không có phím mặc định; `bench-216.js` gán `Mod+Alt+Y` qua `put_config` kind `shortcut` khi tạo Tác phẩm.
- Bộ chọn ô chuột: ba ô thứ 2, 4, 6 đang hiện (`.cell-tgt`, `data-segment-id`). Nếu khối đổi DOM, `.cell-tgt` vẫn phải tồn tại.

## Số "trước" (9.850 câu, baseline 5f25df6)
- Bố cục đầu 4,38–4,71 s (ô đích đầu có mặt ở 3,2–3,4 s).
- Chuột thật: handler->2 frame 114–142 ms, gửi->2 frame 233–306 ms.
- `segment.next`: handler->f 59–110 ms, gửi->f 117–183 ms.
- Cỡ 122 câu: bố cục 0,51–0,59 s, chuột 43–112 / 155–215 ms, phím 41–65 / 46–76 ms.
- Số gõ 10 phím/giây ở 9.850 câu: lấy từ bàn 2-15 (không đo lại ở Pha 1): ~0,5 s mỗi phím.

## Việc còn lại / rủi ro
- Ceiling cho 2.14 tiêu chí 2 ở thân story, không có trong plan này: so với số trên.
- `mouseup` của cliclick không ghi được; chỉ mousedown có số.
- Plan có `context: []` rỗng; Pha 2 đọc Code Map trong plan, không có tệp context để nạp.
- Phiên mỗi cỡ đo khi máy tải 5–10 (loadavg), ghi trong JSON.

# Handoff Pha 2 -> Pha 3 (story 2-16) — BLOCKED, cho Ice quyết

Kết luận: bản thử chia khối ĐẠT vế gõ, TRƯỢT vế dời con trỏ và làm bố cục đầu chậm gấp đôi ⇒ theo plan dừng, không commit `src/`. Pha 3 và 4 chưa nên chạy.

## Đã làm
- Bản thử (không commit, đã gỡ khỏi cây): `blocks-prototype.patch` (GridPanel.vue, proofreadHighlight.ts, một dòng test) + `gridBlocks.ts.txt`. Áp lại: `git apply blocks-prototype.patch` và chép `gridBlocks.ts.txt` thành `src/panels/gridBlocks.ts`.
  - `gridBlocksOf(total)` thuần; `.grid` bọc `.grid-block` (key theo thứ tự khối), năm cột subgrid mỗi khối, cột chung `--grid-columns`; số hàng/nhãn/v-memo dùng `block.start + j`; `.strip-track` chỉ dựng ở khối chứa neo (v-if), `gridRow` tính theo khối.
  - Host duy nhất của `colTgt` đổi sang `gridRoot` + `[data-col="tgt"][data-segment-id]` (restoreEditedText, repaintProofread, unwrapAll, onSelectionChange, watcher điều hướng, focusout). `colSrc/colTgt` chỉ gắn vào khối 0 (function ref) nên vùng chọn/Lookup ngoài khối 0 hỏng cho tới Pha 3 — đúng ý bản thử.
  - Hệ quả đã thấy: vue-tsc xanh; 5 ca vitest tìm `.grid` đỏ (gridPanelStripTrack ×4, gridPanelImages ×1), đúng danh sách Pha 4.
- Đổi hợp âm bàn 2-15 sang Ctrl+Alt+Shift+Q/W/E/U (tránh macshot): `2-15-ban-do/bench-215.js`, `run-session.sh` (đang sửa trong cây, chưa commit).

## Số đo bản thử (release, bản dựng từ cây làm việc trên 5f2e1a9 + patch, WKWebView macOS 15.8.2, 9.850 câu, loadavg 5–12, 3 phiên mỗi bên; baseline = 5f25df6/043f3bc đã có)
- Gõ 10 phím/giây 30 s (bàn 2-15, osascript): phím cuối 84 / 55 / 77 ms, không phím chờ, p50 44 ms, p95 135 ms, max ~300 ms (900 phím). Baseline cùng bàn: phím chờ ~260, phím cuối ~192 s, p50 ~96 s. ⇒ vế gõ ≤ 100 ms đạt ở phím cuối; p95 135 ms.
- Bố cục đầu: 7,2–9,2 s (baseline 4,4–4,7 s); ô đích đầu có mặt ở 5,4–6,8 s (baseline 3,2–3,4 s). TRỞ NÊN TỆ HƠN.
- Chuột thật, handler->2 frame: 234 / 599 / 303 / 593 / 763 ms; gửi->2 frame 361 / 891 / 441 / 864 / 7379 ms (baseline 114–142 / 233–306). Ba phiên chỉ có 0, 3, 2 lượt trúng ô (phiên 1 không trúng: bố cục chưa xong lúc bấm). Tiêu chí ≤ 50 ms: TRƯỢT, tệ hơn baseline.
- `segment.next` handler->f 38–115 ms (baseline 59–110), gửi->f 205–509 (baseline 117–183). Tiêu chí ≤ 50 ms: TRƯỢT.
- Giới hạn: loadavg bản thử 8–12 vs baseline 6–10; số chuột/phím có thể phồng một phần do tải, nhưng chênh 2–4 lần và bố cục đầu x2 vượt xa mức tải.

## Gợi ý cho Ice (chưa kiểm)
- Nghi nguyên nhân dời con trỏ chậm hơn: `gridBlockViews` tính lại 50 khối (slice mới) mỗi khi `stripAnchorIndex` đổi, nên mọi `.grid-block` re-render + diff; và 50 subgrid-khối vẫn bị WebKit tính lại vì thay đổi class `editing`/neo dải. Cần đo (profile) trước khi sửa; không tự chọn.
- Hai phương án để Ice chọn: (A) dừng 2-16, giữ bug 2.14/2.15 blocked; (B) thử lại bản thử với danh sách khối ổn định (slice theo khối được cache, chỉ khối chứa neo đổi) rồi đo lại. Chưa làm B.
- Số thô: `2-16-ban-do/ket-qua-blocks-9850-{1,2,3}.json` (chuột/phím/bố cục), `2-15-ban-do/ket-qua-blocks-9850-{1,2,3}.json` (gõ). Chưa ghi vào README bàn đo, chưa đụng plan/story/deferred-work.

# Handoff Pha 2 lượt 2 -> Ice / Pha 3 (story 2-16) — BLOCKED lần hai, cho Ice quyết

Phương án B (Ice 2026-10-23) đã làm đủ ba việc. Kết quả: danh sách khối ổn định KHÔNG cứu được vế dời con trỏ; không commit `src/`, cây `src/` sạch. Pha 3 và 4 chưa nên chạy.

## Đã làm
- `blocks-prototype-2.patch` (+ `gridBlocks.ts.txt` như cũ): bản thử đã sửa. `gridBlockViews` chỉ phụ thuộc `editorSegments` (slice theo khối, không đọc neo dải); `stripRowIn(block)` và `stripBlockIndex` đọc neo ngay trong template, nên chỉ khối chứa neo đổi `gridTemplateRows`/`.strip-track`. vue-tsc xanh. Áp: `git apply blocks-prototype-2.patch` (thay cho patch 1) và chép `gridBlocks.ts.txt` thành `src/panels/gridBlocks.ts`.
- Bàn đo (`bench-216.js`, `analyse.py`, đã sửa, chưa commit):
  - Bố cục đầu chỉ coi là xong khi 60 frame liên tiếp cách nhau < 40 ms (`quiet`), rồi mới focus ô đầu; `pickTargets` chỉ phát toạ độ khi vị trí 17 ô đích đứng yên 1 s (thử lại tới 8 lần). Thay cho `setTimeout 1500`.
  - Profile mỗi lượt chuột/phím: `bubble` (mousedown tới listener bubble ở `document`, tức sau handler Vue + flush microtask, trước hành động mặc định), `task` (setTimeout 0), `style`/`layout` ép tay, `raf1`, `f`, số đột biến DOM (MutationObserver gom qua callback) và số khối bị đụng.
- Số thô: `ket-qua-prof{Base,B,A,A2}-9850-*.json`. `Base` = cây 30049b3 (bố cục cũ), `B` = bản thử đã sửa, `A`/`A2` = bản thử cũ (neo dải đổi cả danh sách). Dựng release, ký ad-hoc, WKWebView macOS 15.8.2, 9.850 câu, cửa sổ 1200×900, cliclick + osascript. Dùng chung một bàn đo ⇒ so được với nhau, không so với số 4,4–4,7 s / 114–142 ms của Pha 1 (bàn cũ, máy nhẹ hơn).

## Kết quả (loadavg 1 phút lúc bắt đầu 12–20, 5 phút 21–32: máy bận hơn Pha 1 nhiều; cặp Base/B chạy xen kẽ nên cùng điều kiện)
| | Base (3 phiên) | B (3 phiên) |
|---|---|---|
| chuột: mousedown → bubble (JS handler + Vue flush) | 113·62·87 / 122·119·71 | 258·356·347 / 230·206·272 / 174·186·226 |
| chuột: handler → 2 frame (lượt trúng ô) | 235·240·174 (phiên 3); phiên 1 250·150·196 | 360·454·467 / 318·291·347 / 238·247·316 |
| phần sau flush (task → 2 frame) | ~80–127 | ~60–119 |
| phím `segment.next` handler → f | 87·132·112 / 102·99·90 / 144·156·104 | 67·92·177 / 83·86·87 / 45·88·162 |
| bố cục đầu (ms) | 5731 · 6066 · 6722 | 8102 · 5960 · 5877 |
- Chênh nằm ở JS giữa mousedown và listener bubble: khối làm phần Vue render + flush của một lượt `setEditorCaret` dày gấp 2–3 lần (63–122 → 174–356 ms). Phần style/layout ép tay đo 0–1 ms và `focus()`/`getBoundingClientRect` không bị gọi ở đường này: layout không phải chỗ tốn. Đột biến DOM chỉ 6 bản ghi, 1 khối ⇒ DOM vá nhỏ; cái tốn là hàm render/diff.
- Danh sách khối ổn định (B) không khác bản neo-đổi-cả-danh-sách (A2: 255–381 ms) ⇒ giả thuyết "mỗi lần neo đổi, 50 khối cắt lại" sai.
- Bố cục đầu dưới cùng tải: Base 5,7–6,7 s, B 5,9–8,1 s (hai phiên 5,9 s) — không còn "gấp đôi" như lượt 1, phần lớn lượt 1 là do tải; nhưng cũng không nhanh hơn.
- Vế gõ: KHÔNG đo lại ở bản B (lượt 1 đã đạt: phím cuối 55–84 ms). Danh sách khối ổn định không chạm đường gõ nhưng chưa có số.
- Tiêu chí dời con trỏ ≤ 50 ms: cả Base (kể cả 114–142 ms của Pha 1) lẫn B đều trượt; B tệ hơn Base ~1,3–2 lần từ handler tới frame.

## Giới hạn, rủi ro
- Tải máy 12–32: số tuyệt đối phồng; chỉ cặp xen kẽ Base/B có giá trị so sánh. Không có phiên nào ở máy nhẹ.
- Phiên Base 2 và A2 1 có lượt bấm không đổi tiêu điểm (task 0–1 ms, `hit=False`): bỏ khỏi so sánh. `mouseup` hầu như không ghi được (như Pha 1).
- Chưa rõ vì sao render gấp 2–3 lần: chưa kiểm hai giả thuyết (Fragment lồng thêm một tầng làm `v-memo` mất đường tắt `n1 === n2` ở cấp khối; 50 lần `patchKeyedChildren` trên mảng slice mới mỗi lần `gridBlockViews`/render tạo mảng mới thay vì cùng mảng). Cần profiler Vue/Safari timeline, không có trong bàn này.
- Chưa commit gì: bàn đo sửa + 9 `ket-qua-prof*.json` + `blocks-prototype-2.patch` còn là tệp chưa theo dõi/đã sửa trong cây. README bàn đo chưa có bảng số mới; plan/story/deferred-work chưa đụng.

## Gợi ý
- Ice chọn: (A) dừng 2-16, giữ 2.14/2.15 blocked; (B2) tiếp tục chỉ khi muốn điều tra gốc render 2–3 lần bằng Safari Web Inspector Timeline trên bản release (đo bằng tay, ngoài bàn này); (C) bỏ ý tưởng khối-subgrid và tìm hướng khác cho lệnh gõ (ví dụ tắt subgrid bằng hàng cao cố định) — chưa đo.

# Giao việc Pha 2 lượt 3 (story 2-16) — bản thử C2, Ice duyệt 2026-10-23

Gốc chi phí đã rõ (`d7262fc`; `../../initiative-auratranslate/research-grid-render-on-caret-move/research-grid-render-on-caret-move.md` §3, §Where the answers meet; `memo-probe.cjs`): ở bản thử 2, bốn vòng hàng có `v-memo` nằm trong `v-for` khối nên mọi khối dùng chung bốn ô `_cache` (7/9/13/15) của component; khối k nhận vnode của khối k−1, `_cached.key === s.id` trượt, mọi hàng dựng lại mỗi lượt dời con trỏ. Cả lưới là slot của `PanelFrame`, nên một lượt dời chạy lại một hiệu ứng render cho cả 50 khối.

## Việc (chỉ phần 1 và 2; KHÔNG dựng release, KHÔNG chạy bàn đo — phần 3 Ice hẹn giờ)
1. Bản thử 3 trên nền bản thử 2: mỗi khối 200 hàng là một component con (tên theo từ vựng cố định của AGENTS.md; không dùng `Project/Book/Novel/Document`). Mục tiêu đo được: một lượt dời con trỏ chỉ render lại khối con trỏ rời và khối con trỏ tới; các khối khác không chạy render.
   - Khối không được đọc trong render bất kỳ giá trị nào đổi theo con trỏ ở phạm vi cả Chương: `editorCaretSegmentId`, `stripAnchorSegmentId`, `stripAnchorIndex`, `stripBlockIndex`. Mỗi giá trị ấy vào khối qua prop đã thu về phạm vi khối: id con trỏ nếu nằm trong khối, ngược lại `null`; tương tự neo dải, hàng của dải trong khối (`stripRowIn`), có dựng dải hay không. `ruleOf`/`cellLabelOf`/lớp `editing` trong khối dùng prop này, không dùng ref toàn cục.
   - Giá trị không đổi khi dời con trỏ (`restRuleById`, `selectedRowClassById`, `rowErrorTextById`, `editorEditedShape`, `glossaryTermsOf`…) được phép đọc chung; ghi rõ trong handoff giá trị nào đọc chung.
   - ⚠️ Prop nào là object/mảng/hàm mới mỗi lượt render của cha (literal `{}`/`[]`, arrow inline, `.slice()` trong template) làm `shouldUpdateComponent` trả true cho mọi khối ⇒ quay về C1. `block` lấy từ `gridBlockViews` (giữ danh tính khi dời con trỏ). Handler sự kiện chuyển qua `provide/inject` một đối tượng hằng hoặc prop hàm ổn định.
   - Giữ hành vi bản thử 2: `colSrc/colTgt` chỉ gắn ở khối 0, dải ở khối chứa neo, `[data-col="tgt"][data-segment-id]` làm bộ chọn. Không sửa phần ngoài `src/panels/` trừ khi bắt buộc; nếu phải, ghi lý do.
2. Kiểm bằng vitest (dev build, happy-dom): một tệp test mới đếm số lần mỗi khối render/updated khi `setEditorCaret` đi (a) trong một khối, (b) qua ranh giới khối, (c) khi neo dải đổi khối, ở ≥ 1.000 câu (≥ 5 khối). Mong đợi (a) ≤ 1 khối, (b)(c) ≤ 2 khối. Đếm qua hook công khai của Vue (`updated` trong một mixin/plugin của test, hoặc tương đương), không cài thiết bị đo vào mã sản phẩm. Phải đối chứng đỏ: tạm cho khối đọc thẳng `editorCaretSegmentId` (phép gỡ thật, giữ chữ ký) và thấy test đỏ đúng lý do, rồi trả lại.
   - Chạy thêm `vue-tsc` và các tệp vitest lưới: `gridCaretMoveCost`, `gridTypingCost`, `gridRuleStable`, `gridPanelStripTrack`, `gridPanelImages`, `gridPanelCellLabel`, `gridPanelRowErrorPriority`, `gridRestoreEditedText`, `editorTypingZone`, `segmentSelection`, `proofreadHighlight`. Đỏ đã biết từ bản thử 2: `gridPanelStripTrack` ×4 và `gridPanelImages` ×1 (đọc `style.gridTemplateRows` trên `.grid`). Đỏ khác ⇒ ghi lý do.

## Nơi làm và đầu ra
- Worktree tạm (đã có bản thử 2 commit ở `b2874aa`, `node_modules` là symlink): `/private/tmp/claude-501/-Users-hoangnam-LocalSites-addon-AuraTranslate/38ff5937-f6f4-47cd-9033-1ffaf37e4590/scratchpad/tree-b`. Không đụng cây chính ngoài hai tệp đầu ra dưới đây.
- Đầu ra, ghi vào `/Users/hoangnam/LocalSites/addon/AuraTranslate/_bmad-output/implementation-artifacts/2-16-ban-do/`:
  - `blocks-prototype-3.patch`: `git diff cc60020 -- src tests` trong worktree (dùng `git add -N` cho tệp mới), áp được lên `cc60020` bằng `git apply` và đứng một mình (không cần patch 2 hay `gridBlocks.ts.txt`).
  - Mục `# Handoff Pha 2 lượt 3 -> Ice / phần 3` nối vào cuối tệp này: đã làm gì, giá trị nào đọc chung, số khối render ở từng ca, kết quả đối chứng đỏ, danh sách test xanh/đỏ, `vue-tsc`, rủi ro. Ngắn, số kèm phạm vi đo.

# Handoff Pha 2 lượt 3 -> Ice / phần 3 (story 2-16) — bản thử C2

Chưa dựng release, chưa chạy bàn đo; mọi số dưới đây là vitest dev build trên happy-dom, 1.050 câu = 6 khối (5 × 200 + 50), đếm hook `updated` của Vue qua mixin của test.

## Đã làm
- Patch `blocks-prototype-3.patch` = `git diff cc60020 -- src tests`, `git apply --check` xanh trên cây sạch ở `cc60020`, đứng một mình. Cây thử: scratch `tree-b`, bản thử 3 CHƯA commit ở đó.
- `src/panels/GridBlock.vue` (mới): một khối 200 hàng = một component, mang nguyên bốn cột + dải cũ và toàn bộ CSS từ `.grid-block` trở xuống (scoped chuyển theo). `src/panels/gridBlockContext.ts` (mới): khoá `provide/inject` + kiểu. `GridPanel.vue` còn vòng `<GridBlock v-for>` và dựng một đối tượng ngữ cảnh HẰNG gọi `provide` một lần.
- Prop của khối đều nguyên thuỷ hoặc `block` từ `gridBlockViews`: `caretId` (id con trỏ nếu nằm trong khối, ngược lại `null`), `anchorId` (id neo dải nếu nằm trong khối), `stripRow` (hàng dải trong khối, -1 nếu không), `hasStripTrack` (boolean). Cha thêm computed `caretIndex` (một `findIndex` mỗi lượt dời) để cắt `caretId` theo khối.
- `ruleOf`, `ruleClassForRow`, `ruleSlotClass`, `cellLabelOf`, `STATE_LABEL_KEYS` chuyển sang khối; `ruleOf` dùng `props.caretId` + `restRuleById`. Lớp `editing` dùng `caretId`. v-memo hàng dùng `anchorId`. Khối KHÔNG đọc `editorCaretSegmentId`, `stripAnchorSegmentId`, `stripAnchorIndex`, `stripBlockIndex`.
- Đọc chung qua ngữ cảnh (không đổi khi dời con trỏ): `restRuleById`, `selectedRowClassById`, `unlinkedIds`, `rowErrorLabelById`, `rowErrorTextById`, `firstSegmentId`, `showHanViet`, `sourceTokenClass`, `cutOffsetsOf`/`cutCountOf`, `glossaryTermsOf`, `sourcePieceInfoOf`, `trailingImagesOf`/`leadingImagesOf`, bảng nhãn vai, các ref/handler (`setCol*Ref`, `setStripTrackRef`, sáu handler cột bản dịch, `onSourceCellMouseUp`, `onGlossaryPiece*`, `onStripFocusOut`). Đọc thẳng từ module trạng thái trong khối: `editorEditedShape`, `editorChapterAssets`, `editorAssetsDir`, `concordanceSourceHighlight`, `viewMode`.
- Ngoài `src/panels/`: không đụng. Test chuyển theo mã: `gridRuleStable.test.ts` đọc CSS từ `GridBlock.vue` (1 dòng).
- Hành vi bản thử 2 giữ nguyên: `colSrc/colTgt` chỉ ở khối 0, dải ở khối chứa neo, bộ chọn `[data-col="tgt"][data-segment-id]`.

## Số khối `updated` mỗi lượt (test mới `tests/frontend/gridBlockRenderCost.test.ts`, 4 ca, xanh)
- (a) dời con trỏ trong một khối (câu 250 -> 251): khối {1} = 1 khối.
- (b) qua ranh giới (399 -> 400): {1, 2} = 2 khối.
- (c) con trỏ và neo dải cùng sang khối khác (250 -> 650): {1, 3} = 2 khối; chỉ neo dải đổi (250 -> 850): {1, 4} = 2 khối.

## Đối chứng đỏ
Phép gỡ thật: `ruleOf` trong `GridBlock.vue` đọc `editorCaretSegmentId.value` thay `props.caretId` (giữ chữ ký, prop vẫn truyền). Kết quả: ca (a) nhận `[0,1,2,3,4,5]` thay `[1]`, ca (b) `[0..5]` thay `[1,2]`, ca con trỏ+neo `[0..5]` thay `[1,3]` — đỏ đúng lý do (cả 6 khối chạy render); ca chỉ-neo vẫn xanh vì nó không đụng con trỏ. Đã trả lại, bốn ca xanh.

## Test và kiểu
- `vue-tsc --noEmit -p tsconfig.json`: 0 lỗi.
- vitest, 12 tệp (11 tệp lưới theo giao việc + tệp mới): 85 xanh, 5 đỏ, đúng danh sách đã biết: `gridPanelStripTrack` ×4 và `gridPanelImages` ×1 (đều đọc `style.gridTemplateRows` trên `.grid`; nay thuộc tính ấy nằm trên `.grid-block`). Không đỏ nào khác.
- Không chạy: toàn bộ suite, gate, e2e, build release.

## Rủi ro
- Cha vẫn render lại mỗi lượt dời con trỏ (đọc `caretIndex` để cắt prop) và vẫn chạy lại hiệu ứng render của slot `PanelFrame`; chưa đo, nhưng nó chỉ so 50 bộ prop nguyên thuỷ. Phần 3 cần đo thật trên release.
- `defineExpose({ concordanceHighlightOf, chapterAssetsOf })` trong `GridBlock.vue` chỉ để `vue-tsc` 3.3.9 không báo TS6133 (nó không tính biểu thức `v-memo` là lượt đọc). Một dòng chú thích đã ghi; nếu bản thử vào `src/` thật nên đổi sang cách khác.
- 5 test đỏ ở trên cần sửa bộ chọn sang `.grid-block` nếu bản thử được nhận; tôi cố ý để nguyên vì giao việc ghi chúng là đỏ đã biết.
- `selectedRowClassById`, `rowErrorTextById`, `unlinkedIds` đọc chung: khi chúng đổi (vùng chọn, lỗi), mọi khối đọc `.get`/`.has` sẽ chạy lại; chưa có ca test cho vùng chọn đa-segment qua nhiều khối.
- Ca test không phủ `cutOffsetsOf` (điểm cắt đang chờ) khi dời con trỏ: chưa kiểm khối không-liên-quan có render lại khi `editorSourceCut` đổi cùng lượt dời con trỏ.

# Kết quả Pha 2 lượt 3, phần 3 (đo release, 2026-10-23 23:50 → 2026-10-24 00:12)

Bản dựng: release, ký ad-hoc, WKWebView macOS 15.8.2, 9.850 câu, cửa sổ 1200×900. Base = `0b619fb` (src = `cc60020`, bố cục cũ); P3 = `0b619fb` + `blocks-prototype-3.patch`. Xen kẽ base/P3, 3 phiên mỗi bên mỗi bàn; loadavg 1 phút lúc bắt đầu phiên 2,6–4,5 (Docker VM chạy nền). Số thô: `ket-qua-c2{base,p3}-9850-{1,2,3}.json` ở `2-16-ban-do/` (chuột/phím/bố cục) và `2-15-ban-do/` (gõ). Log điều phối giữ ngoài repo.

| | Base | P3 |
|---|---|---|
| Gõ 10 phím/giây 30 s: phím cuối hiện sau | 122,0–128,4 s (phím chờ 241–244) | 65–72 ms (phím chờ 1) |
| Gõ: trung vị / max độ trễ | 60–65 s / 122–128 s | 23–31 ms / 144–169 ms |
| Chuột: mousedown → listener bubble (JS + Vue flush) | 41–49 ms | 1–3 ms |
| Chuột: handler → 2 frame | 100–118 ms | 55–85 ms |
| Chuột: raf1 → f (phần sau JS) | 55–65 ms | 53–66 ms |
| Chuột: gửi → 2 frame | 202–230 ms | 159–207 ms (phiên 1: 385–388) |
| `segment.next`: handler → f | 59–82 ms | 58–76 ms |
| Bố cục đầu / ô đích đầu có mặt | 3,66–3,93 s / 2,80–2,94 s | 3,55–3,67 s / 3,42–3,54 s |

- Vế gõ ≤ 100 ms ở phím cuối: ĐẠT ở P3; base vẫn xếp hàng phím như cũ.
- Vế dời con trỏ ≤ 50 ms: TRƯỢT ở cả hai; P3 bỏ gần hết phần JS (41–49 → 1–3 ms) nhưng phần sau JS tới hai frame (~55–65 ms) không đổi, và `segment.next` không đổi. Phần còn lại chưa quy nguồn: lượt này không có số style/layout/paint của chính frame đó.
- Mỗi lượt chuột ở P3 đụng đúng 1 khối (`blocks=1`, 6 bản ghi DOM).
- P3 phiên 1: gửi → 2 frame 385–388 ms trong khi handler → 2 frame 69–79 ms ⇒ trễ nằm trước handler (giao sự kiện), không phải app.

# Giao việc Pha 3 (story 2-16) — Ice chọn hướng 1, 2026-10-24

Ice nhận C2 dù vế dời con trỏ ≤ 50 ms trượt (`fd13e19`; plan §Implementation Notes, bullet cuối). Pha 3 đưa bản thử 3 vào `src/` và làm phần vùng chọn của plan. Pha 4 (agent sau) làm test mới, e2e và đối chứng gỡ.

## Việc
1. Đưa vào cây chính: `git apply _bmad-output/implementation-artifacts/2-16-ban-do/blocks-prototype-3.patch` (đã kiểm: áp sạch lên HEAD; `src/` và `tests/` không đổi từ `cc60020`). Đọc mục "Handoff Pha 2 lượt 3 -> Ice / phần 3" ở trên trước: giá trị đọc chung, 5 đỏ đã biết, rủi ro.
2. Làm xanh vitest lưới: 5 đỏ đã biết (`gridPanelStripTrack` ×4, `gridPanelImages` ×1 đọc `style.gridTemplateRows` trên `.grid`) và các test plan §Code Map liệt kê là giả định một `.grid`/`.col-*`. Chỉ đổi cách tìm phần tử, không đổi điều kiện khẳng định (plan §Never). `gridTypingCost`, `gridCaretMoveCost` giữ nguyên điều kiện.
3. Plan §Tasks, Pha 3: bề mặt vùng chọn có phạm vi trong `selectionContract.ts` (tham số tuỳ chọn là bộ chọn cột; khoá mục thành (el, role); `surfaceFor` đòi anchor trong `closest(scope)`; `focusSelectionSource` đặt caret ở đầu phần tử khớp đầu tiên). Lưới gọi hai lần trên gốc lưới với `[data-col="src"]` / `[data-col="tgt"]`, thay cho `colSrc`/`colTgt` chỉ gắn ở khối 0 của bản thử. Kiểm F của `scripts/check-commands.mjs` vẫn đếm 2; sửa lý do ở chỗ ghi lý do và bắt buộc hai lời gọi của lưới có phạm vi. Mọi chỗ khác còn coi `colSrc`/`colTgt` là host duy nhất (plan §Code Map) đổi sang gốc lưới + bộ chọn cột; bản thử đã đổi một phần, kiểm hết.
4. Kéo chọn trong cột nguyên văn qua ranh giới khối chỉ tô cột nguyên văn (plan §Boundaries, quyết định tiêu chí 2; §Design Notes: hai cách, CSS `user-select: none` lên các cột khác khi lượt kéo bắt đầu trong `.col-src`, hoặc JS chặn đầu `focus` của vùng chọn trong `selectionchange`). Plan ghi ca e2e kéo chuột thật quyết định, và nếu cả hai đạt thì trình cả hai cho Ice. Vì vậy KHÔNG chọn: viết cả hai thành `keo-cot-nguyen-van-css.patch` và `keo-cot-nguyen-van-js.patch` trong `2-16-ban-do/`, mỗi patch áp riêng lên cây sau bước 3. Không đưa cách nào vào `src/`. Pha 4 viết ca e2e và chạy cả hai.
5. Kiểm: `npx vue-tsc --noEmit` (hoặc lệnh kiểu của `package.json`), các tệp vitest lưới và vùng chọn đã đổi hay liên quan (`segmentSelection`, `selectionContract*`, `tmConcordanceSurfaces`, `proofreadHighlight`, `gridBlockRenderCost` và danh sách ở mục giao việc Pha 2 lượt 3), `npm run test:story 2-16`, `npm run check:commands && npm run check:layout`. Không chạy cả bộ (AGENTS.md §Tests). Phép gỡ cho phạm vi bề mặt: gỡ thật kiểm `closest(scope)` trong `surfaceFor`, giữ chữ ký, thấy một ca đỏ đúng lý do, rồi trả lại; chưa có ca nào đỏ thì thêm ca.

## Ràng buộc
- Làm trong cây chính. KHÔNG commit, KHÔNG push; để thay đổi trong cây cho người giao việc đọc và commit.
- ⚠️ Không `git checkout`/`git stash`/`git restore` trên tệp có thay đổi chưa commit (mất việc). Lấy bản gốc bằng `git show HEAD:<path>` ra tệp tạm.
- ⚠️ `rm` là alias `rm -i` và treo im lặng: dùng `/bin/rm`. Không gọi `cat` không đối số.
- Comment theo AGENTS.md §Code comments; không ghi story id, số đo, lịch sử vào mã.
- Không thêm phụ thuộc. Không đụng `src-tauri/`.

## Đầu ra
Mục `# Handoff Pha 3 -> Pha 4 (story 2-16)` nối vào cuối tệp này, ngắn: đã làm gì (theo tệp), Kiểm F đổi gì, hai patch kéo chọn và cách áp, test xanh/đỏ kèm phạm vi, phép gỡ và kết quả, việc để lại cho Pha 4 (e2e nào phải đổi cách tìm hàng, e2e nào cần thêm).

# Handoff Pha 3 -> Pha 4 (story 2-16)

Chưa commit. Cây chính = HEAD + `blocks-prototype-3.patch` + phần dưới.

## Đã làm (theo tệp)
- `src/panels/selectionContract.ts`: `Surface` có `scope`; `registerSelectionSurface`/`useSelectionSurface` nhận tham số cuối tuỳ chọn `scope` (bộ chọn cột); mục khoá theo (el, role); `surfaceFor` qua `ownsAnchor` đòi `closest(scope)` của anchor nằm trong `el`; `focusSelectionSource` đặt caret ở đầu phần tử khớp đầu tiên.
- `GridPanel.vue`: hai lời gọi trên `gridRoot` với `'[data-col="src"]'` / `'[data-col="tgt"]'`; bỏ `colSrc`/`colTgt`/`setColSrcRef`/`setColTgtRef`. `GridBlock.vue`, `gridBlockContext.ts`: bỏ hai ref đó.
- `tests/frontend/gridPanelStripTrack.test.ts`, `gridPanelImages.test.ts`: `.grid` -> `.grid-block` chỉ ở dòng đọc `gridTemplateRows`; điều kiện giữ nguyên. `tmConcordanceSurfaces.test.ts`: thêm ca "vùng chọn ở cột số không thuộc bề mặt nào".
- `src/panels/README.md`: hai ô bảng vai bề mặt.

## Cổng `scripts/check-commands.mjs`
- Kiểm F: vẫn đếm 2 (`SELECTION_PANEL_FILES`); ⑤ nay còn đòi lời gọi `source` có `'[data-col="src"]'` và `display` có `'[data-col="tgt"]'` trên `gridRoot` (regex `GRID_SCOPED_CALL_RE`, trường `scopeCol`); lý do viết lại.
- `VUE_FLOOR` 48 -> 52 (cổng tự báo `GridBlock.vue` làm sàn trôi, ceil(0.85 x 61)).
- Kiểm I: `EDITOR_PANEL_VUE` trỏ `GridBlock.vue` (khối CSS `.rule-*` đã chuyển sang đó).
- Kiểm K: bốn mục `GridPanel.vue::{onSourceCellMouseUp,onCellMouseDown,onCellMouseUp,onEditKeydown}` đổi sang `GridBlock.vue::`; thêm `HANDLER_PROVIDER` + tham số `provider` cho `judgeHandlerInventory` để thân hàm đọc ở `GridPanel.vue` (handler tới khối qua `inject`), vẫn đối chiếu bộ id `onEditKeydown` thật.

## Hai patch kéo chọn (chưa vào `src/`; mỗi cái áp riêng lên cây hiện tại, `git apply --check` xanh, `vue-tsc` xanh khi áp)
- `keo-cot-nguyen-van-css.patch`: mousedown/mouseup ở document (capture) đặt/gỡ `data-drag-src` trên `gridRoot` khi mousedown trong `[data-col="src"]`; `GridBlock.vue` thêm `.grid[data-drag-src] .col:not(.col-src) { user-select: none }` (kèm `-webkit-`).
- `keo-cot-nguyen-van-js.patch`: cờ `draggingFromSource` và `confineSourceDrag` trên `selectionchange`: focus ra ngoài cột nguyên văn thì `selection.extend` về cuối/đầu ô nguyên văn gần nhất theo hướng kéo. ⚠️ Giới hạn đoán trước: Range vẫn chứa DOM của cột bản dịch/số nằm giữa hai ô nguyên văn khác khối nên các cột đó có thể vẫn được tô; chỉ e2e kéo chuột thật mới kết luận. Chưa ca nào chạy thật cho cả hai.
- Áp: `git apply _bmad-output/implementation-artifacts/2-16-ban-do/keo-cot-nguyen-van-<css|js>.patch`; gỡ bằng `git apply -R`. Không áp cả hai cùng lúc (cả hai thêm `onDocumentMouseDown/Up` vào `GridPanel.vue`).

## Test
- Xanh: `vue-tsc --noEmit` 0 lỗi; `check:commands`, `check:layout`; `npm run test:story 2-16` (6 tệp, 36 ca); vitest 24 tệp lưới/vùng chọn/tm/glossary/proofread/editorAutoLookup lúc đầu: 166 xanh + đúng 5 đỏ đã biết, nay sửa xong (12 ca hai tệp xanh); 13 tệp (121 ca) gồm `checkCommandsScan`, `gridBlockRenderCost`, `gridTypingCost`, `gridCaretMoveCost`, `segmentSelection` xanh.
- Không chạy: toàn bộ suite, e2e, build.

## Phép gỡ
- Bỏ nhánh `scope === undefined` -> `return true` trong `ownsAnchor` (mất kiểm `closest(scope)`, chữ ký giữ): đỏ 2 ca ở `tmConcordanceSurfaces` (ca mới về cột số, và ca nguyên văn/bản dịch cũ), 11 tệp còn lại xanh. Đã trả lại.
- Bỏ bộ chọn `[data-col="tgt"]` khỏi lời gọi `display` trong `GridPanel.vue`: `check:commands` đỏ đúng một FAIL ⑤. Đã trả lại.

## Việc cho Pha 4
- Sửa cách tìm hàng 4 spec e2e ở Code Map (`story-12-9-strip-row-track:116`, `story-12-7-workspace-grid-status-bar:73-78` một `.grid`, `grid-image-row-alignment:60-75` ghép `.col-* > *` theo chỉ số, `hanviet-parallel-row-height:23`); các test vitest 5 ca đã sửa ở trên.
- `tests/frontend/gridBlocks.test.ts` (mới) theo plan; thêm e2e: điều hướng qua ranh giới, Lookup ở khối 5, <=1 px quanh ranh giới, kéo chuột thật cột nguyên văn qua ranh giới chạy cho CẢ HAI patch, `Mod+Shift+↓` qua ranh giới.
- Gỡ thật phần chia khối xác nhận ca hình dạng đỏ.
- Bề mặt nguyên văn nay là `[data-col="src"]` từng ô (không còn cả cột): vùng chọn nằm ở ảnh/dải trong cột nguyên văn mà không trong ô sẽ không phát lượt tra; e2e nên có một ca.
- Chú thích cũ nhắc `colSrc` còn ở `editorSegments.ts:466` và `tests/frontend/editorSourceCutGesture.test.ts:12` (không đụng).

# Giao việc Pha 4 (story 2-16) — test và e2e, 2026-10-24

Pha 3 đã commit ở `742096c`. Đọc mục "# Handoff Pha 3 -> Pha 4" ngay trên trước. Pha 4 làm plan §Tasks, Pha 4. KHÔNG làm mục "Nghiệm thu" (bàn release, tốn máy; Ice hẹn giờ riêng).

## Việc
1. `tests/frontend/gridBlocks.test.ts` (mới): các ca của plan §I/O & Edge-Case Matrix theo được ở vitest, cộng ca mount 450 câu ⇒ 3 `.grid-block`, mỗi khối ≤ `GRID_BLOCK_ROWS`, không cột nào trải cả Chương. Ca hình dạng phải đỏ khi gộp về một khối: gỡ thật phần chia khối trong `gridBlocks.ts`/template (giữ chữ ký), chạy lại đúng tệp đó, thấy đỏ đúng lý do, rồi trả lại.
2. Thêm ca vitest cho thay đổi hành vi Pha 3 nêu: bề mặt nguồn nay là từng ô `[data-col="src"]`. Ghi bằng test hành vi hiện có (vùng chọn trong ô nguồn ở khối ≥ 1 phát lượt tra; vùng chọn bắt đầu ngoài ô thì không). Không đổi mã sản phẩm để ca xanh; nếu thấy đó là hồi quy thật so với `cc60020`, dừng và ghi vào handoff cho Ice.
3. e2e: đổi cách tìm hàng ở 4 spec plan §Code Map liệt kê, không đổi điều kiện khẳng định (plan §Never). Thêm các ca e2e plan liệt kê (điều hướng qua ranh giới, Lookup ở khối 5, ≤ 1 px quanh ranh giới trong `grid-image-row-alignment` với ≥ 201 câu, `Mod+Shift+↓` qua ranh giới, kéo chuột thật trong cột nguyên văn qua ranh giới khẳng định không ô nào ngoài cột nguyên văn bị tô). Ca ≤ 1 px: plan đòi đối chiếu một lượt trên `baseline_revision`; làm bằng `git worktree add` vào scratchpad của bạn, đừng `checkout` trong cây chính.
4. Kéo chọn cột nguyên văn: chạy ca kéo chuột thật trên (a) cây hiện tại, để thấy ca đỏ khi chưa có cách nào, (b) cây + `keo-cot-nguyen-van-css.patch`, (c) cây + `keo-cot-nguyen-van-js.patch`. Áp patch bằng `git apply`, gỡ bằng `git apply -R` (KHÔNG `git checkout`). Ghi kết quả từng biến thể. KHÔNG chọn và KHÔNG đưa patch nào vào `src/`: Ice quyết (plan §Design Notes). Để cây ở trạng thái (a) khi xong.
5. Kiểm: `npx vue-tsc --noEmit`, `npm run test:story 2-16`, các tệp vitest đã đổi/thêm, `npm run check:commands && npm run check:layout`, `npx eslint` trên các tệp e2e đã đổi. e2e: chỉ chạy các spec đã đổi hoặc thêm, không chạy cả bộ.

## Chạy e2e
- `e2e/AGENTS.md` trước. Dựng một lần: `cargo build --locked --features wdio --manifest-path src-tauri/Cargo.toml`, rồi `npx wdio run e2e/wdio.conf.mjs --spec <tệp>` cho từng spec. `npm run build` trước nếu `dist/` thiếu hay cũ (cargo nhúng `dist/`); mỗi lần đổi mã webview phải dựng lại `dist/` và binary trước khi chạy e2e.
- e2e phải chạy NGOÀI sandbox (trong sandbox webview đứng ở `tauri://localhost` và đỏ ở bước mount). Đặt trần mỗi lệnh dài: `perl -e 'alarm shift; exec @ARGV' <giây> <lệnh…>` (exit 142 = hết giờ).
- Màn hình khoá thì rAF không chạy và ca đỏ ở `before`: đỏ kiểu đó ghi là môi trường, không sửa mã.

## Ràng buộc
- KHÔNG commit, KHÔNG push. Không sửa plan hay story.
- ⚠️ Không `git checkout`/`git stash`/`git restore` trên tệp có thay đổi chưa commit. Bản gốc lấy bằng `git show <rev>:<path>`.
- ⚠️ `rm` là alias `rm -i`, treo im lặng: dùng `/bin/rm`. Không gọi `cat` không đối số.
- Comment theo AGENTS.md §Code comments. Không thêm phụ thuộc. Không đụng `src-tauri/`.

## Đầu ra
Mục `# Handoff Pha 4 -> Ice (story 2-16)` nối vào cuối tệp này, ngắn: tệp đã đổi/thêm, ca nào xanh/đỏ (kèm phạm vi chạy, bản dựng, lý do đỏ), phép gỡ, bảng ba biến thể kéo chọn, việc còn lại.

# Handoff Pha 4 -> Ice (story 2-16)

Chưa commit. Cây ở trạng thái (a): không patch kéo chọn nào trong `src/`. Không đụng `src/` hay `src-tauri/`.

## Đã đổi/thêm
- `tests/frontend/gridBlocks.test.ts` (mới, 13 ca): `gridBlocksOf` (20 câu, 450 ⇒ 200·200·50, đúng 200, 201), mount 20 câu ⇒ 1 khối, mount 450 ⇒ 3 `.grid-block` ≤ hằng và mỗi khối đúng 1 cột mỗi loại (không cột nào trải Chương), số hàng liên tục qua khối, `.strip-pad` neo hàng 199 nằm trong khối 0, tách ở hàng 199 (nạp lại có thêm 1 câu ⇒ 200·200·51, ô con trỏ vẫn là ô cũ), gộp bớt 1 hàng (200·200·49), `moveEditorCaretTo` tới khối 30 (6.100 câu), vùng chọn trong ô nguồn ở khối 5 phát lượt tra, vùng chọn bắt đầu ở `.col-src` ngoài ô thì không.
- `e2e/specs/grid-blocks.e2e.mjs` (mới, 1.200 câu, 1.100 câu đã dịch): chia 6 khối × 200; `Mod+Alt+↓` từ khối 0 tới câu 1.100 ở khối 5 (ô nhận focus, nằm trong khung cuộn); kéo chuột HỆ ĐIỀU HÀNH trong ô nguồn khối 5 làm `currentQuery` của Lookup đổi; `Mod+Shift+↓` ×4 từ hàng 198 tô đúng 5 id ở cả bốn cột; ca `DRAG_SOURCE_COLUMN` kéo qua ranh giới.
- `e2e/support/osMouse.mjs` (mới): kéo bằng `cliclick` + khung cửa sổ đặt/đọc qua System Events. ⚠️ Lý do: kéo bằng `browser.action('pointer')` KHÔNG tạo vùng chọn trên văn bản không sửa được (không có `selectstart`, đo được cả ô nguồn lẫn double-click); chỉ sự kiện hệ điều hành mới có. Hiệu chỉnh bằng một cú `cliclick c:` ở (400,300) trong khung nhìn. Cần màn hình mở khoá và cửa sổ lên trước.
- Cách tìm hàng: `hanviet-parallel-row-height` ghi đè `.grid-block` (không còn `.grid`); `story-12-7` đọc cột của `.grid-block` đầu; `story-12-9` lấy hàng kế bằng id `segments[MATCH_ORDINAL]` thay cho `~ .cell-tgt`; `grid-image-row-alignment` giữ nguyên ca cũ (ghép theo chỉ số vẫn đúng qua khối) và thêm 2 ca với 207 câu: hàng thẳng + hàng 198..201 không đè nhau (≤ 1 px), và chia 200 + 7 khối; `miniDocx.mjs` nhận tham số số đoạn thêm.

## Kết quả (bản dựng debug `--features wdio`, WebKit 605.1.15, e2e nạp từ Vite dev; từng spec một)
- Xanh: `vue-tsc`, `npm run test:story 2-16`, `gridBlocks.test.ts` 13 ca, `check:commands`, `check:layout`, eslint `e2e` + `gridBlocks.test.ts`.
- e2e xanh: `grid-image-row-alignment` 3/3, `story-12-9` 1/1, `story-12-7` 4/4, `hanviet-parallel-row-height` 4/4, `grid-blocks` 4/5 (đỏ đúng ca kéo, xem bảng).
- Không chạy: toàn bộ suite, các spec e2e khác, nghiệm thu.

## Phép gỡ
- Thay thân `gridBlocksOf` bằng một khối duy nhất (giữ chữ ký): `gridBlocks.test.ts` đỏ 8/13 (cả 2 ca hàm thuần, ca mount 450, tách 199, gộp, khối 30, hai ca vùng chọn khối 5), đúng lý do "một khối 450 hàng". Đã trả lại (`git diff` sạch).
- Đối chứng `baseline_revision` (5f25df6, worktree, nhị phân debug dùng chung): ca ≤ 1 px của `grid-image-row-alignment` XANH ở 207 câu (bố cục cũ), còn ca "chia 200 + 7 khối" đỏ (`[]`). Ca kéo `DRAG_SOURCE_COLUMN` (bỏ vế đếm khối) XANH trên baseline: ca có thể xanh, đỏ ở cây hiện tại là do chia khối. Worktree đã xoá.

## Bảng ba biến thể kéo chọn cột nguyên văn (kéo OS từ hàng 198 tới 202, qua ranh giới 200; `Selection.containsNode(cell, true)` sau mouseup)
| Biến thể | Ô nguồn | Ô dịch | Ô số | Ô vạch | Chữ bản dịch trong `toString()` | Kết quả |
|---|---|---|---|---|---|---|
| (a) cây hiện tại | >= 3 | 200 | 200 | 200 | có | ĐỎ |
| (b) + `keo-cot-nguyen-van-css.patch` | >= 3 | 200 | 200 | 200 | có | ĐỎ, y hệt (a) |
| (c) + `keo-cot-nguyen-van-js.patch` | >= 3 | 200 | 200 | 200 | có | ĐỎ, y hệt (a) |
- (b) hỏng vì `data-drag-src` bị gỡ ở mouseup: `user-select: none` chỉ che lúc kéo, Range vẫn chứa cột khác nên hiện lại khi thả. (c) hỏng đúng như ⚠️ Pha 3 đoán: focus vẫn ở cột nguyên văn của khối 1 nên `confineSourceDrag` không làm gì, Range vẫn chứa cột dịch/số/vạch nằm giữa. Con số 200 = cả cột dịch của khối 0 + cột số và vạch của khối 1.
- ⇒ Cả hai patch không đạt tiêu chí 2 ("kéo qua ranh giới chỉ tô cột nguyên văn"). Phép đo là trạng thái nghỉ sau thả (cái người dùng thấy); chưa đo ảnh chụp điểm ảnh. Tôi không chọn và không đưa patch nào vào `src/`.

## Việc còn lại
- Ice quyết cách kéo cột nguyên văn: cần cách thứ ba (ví dụ chặn `mousedown` trong `.col-src` rồi tự dựng vùng chọn theo từng ô nguồn, hoặc ranh giới khối không cho Range đi qua cột khác) hoặc nới tiêu chí 2; ca `DRAG_SOURCE_COLUMN` là tiêu chuẩn đo. Lệnh: `npx wdio run e2e/wdio.conf.mjs --spec e2e/specs/grid-blocks.e2e.mjs`.
- `grid-blocks.e2e.mjs` đang ĐỎ 1/5 (ca kéo) cho tới khi có cách; đừng đẩy lên nightly như vậy. `osMouse.mjs` chưa chạy trên runner CI (Linux/Windows không có `cliclick`; spec này là macOS-only như nightly).
- e2e chưa chạy: nghiệm thu bàn release (ngoài phạm vi Pha 4). Hàng tiêu đề e2e `before` tốn ~40 s cho 1.200 câu.

## Sau Pha 4 (người giao việc, 2026-10-24)
- Ice chọn: cách thứ ba cho kéo chọn (chỉ tô và chỉ đọc cột nguyên văn), và ca chuột hệ điều hành ở bàn cục bộ.
- Hai ca dùng `cliclick` (Lookup bằng kéo thật ở khối 5, `DRAG_SOURCE_COLUMN`) chuyển sang `keo-cot-nguyen-van.e2e.mjs` cùng `osMouse.mjs` trong thư mục này; nightly không có `cliclick` hay quyền Trợ năng nên `before` của cả spec sẽ đỏ.
- `e2e/specs/grid-blocks.e2e.mjs` còn 4 ca; ca Lookup ở khối 5 nay đặt vùng chọn bằng `Range` trên nút chữ rồi bắn `mouseup`: 4/4 xanh (bản debug `--features wdio`, Vite dev, ngoài sandbox). ⚠️ Cùng ca đó với `selectNodeContents(.src-piece)` (anchor là phần tử, không phải nút chữ) thì không có lượt tra; chưa kiểm hành vi này ở `cc60020`.
- `npm run build` đỏ từ `742096c` ở `vue-tsc -p tsconfig.node.json` (hai bảng tra mới của `check-commands.mjs` thiếu kiểu); đã sửa riêng.

# Giao việc Pha 4b (story 2-16) — kéo chọn cột nguyên văn, cách thứ ba, 2026-10-24

Đã có ở `7c9a3a4`: ca `DRAG_SOURCE_COLUMN` trong `keo-cot-nguyen-van.e2e.mjs` đỏ ở cây hiện tại và ở hai patch `keo-cot-nguyen-van-{css,js}.patch` (bảng ở mục "Handoff Pha 4 -> Ice"). Vùng chọn của trình duyệt là một khoảng liền theo DOM; khi kéo từ cột nguyên văn khối 0 sang khối 1, ô bản dịch, ô số, ô vạch của khối 0 nằm trong khoảng ấy và không bỏ ra được. Ice chọn cách thứ ba: Range vẫn chứa các ô đó, nhưng người dùng chỉ THẤY tô ở cột nguyên văn và mọi thứ ĐỌC vùng chọn chỉ nhận chữ của cột nguyên văn.

## Việc
1. Mã sản phẩm (vào `src/`, không thành patch):
   - Trạng thái "lượt chọn bắt đầu trong cột nguyên văn" trên gốc lưới. Nó phải sống qua lúc thả chuột (patch CSS hỏng chính vì gỡ cờ ở `mouseup`), và hết khi vùng chọn thu về caret, khi anchor rời cột nguyên văn, hoặc khi một lượt chọn mới bắt đầu ở cột khác.
   - Khi trạng thái bật: `::selection` trong suốt ở mọi phần tử ngoài `[data-col="src"]` trong lưới (cột vạch, số, bản dịch, dải, ảnh nếu có). Không dùng `user-select: none`, vì nó đổi chính khoảng chọn.
   - Đọc: `resolveSourceSelection` (`GridPanel.vue`) chỉ trả chữ nằm trong `[data-col="src"]` của khoảng chọn, theo thứ tự hàng, giữ đúng cách ghép giữa các ô như ở bố cục cũ khi kéo nhiều hàng (đọc `cc60020` bằng `git show` để so). Kiểm `copy` (`⌘C`): bố cục cũ chép chữ nguồn; nếu nay chép cả chữ bản dịch thì chặn `copy` khi trạng thái bật và đặt `clipboardData` bằng cùng chữ. Lượt tra tự động, Glossary quick add (`currentSelectionTextForGlossaryQuickAdd`) và mọi chỗ khác đọc vùng chọn từ lưới: liệt kê và kiểm từng chỗ.
   - API `window`/`document` mới (nếu có) phải có mục ghi lý do ở `check-layout.mjs` Kiểm C. Không hạ ngưỡng, không thêm miễn trừ.
2. Đổi khẳng định của `DRAG_SOURCE_COLUMN` cho đúng tiêu chí 2 ("chỉ tô cột nguyên văn"): đo phần ĐƯỢC TÔ bằng điểm ảnh (`browser.takeScreenshot()` hoặc `takeElementScreenshot` ở vài ô bản dịch, số, vạch giữa hai ranh giới, so với cùng ô khi không có vùng chọn), cộng chữ mà lượt tra và `⌘C` nhận không chứa chữ bản dịch. Bỏ khẳng định `containsNode`, vì cách thứ ba giữ các ô đó trong Range. Ghi hiệu chỉnh dpr nếu đo điểm ảnh (máy Ice dpr=2).
   - Chạy ca trên cây chưa có thay đổi của bước 1 (phải đỏ ở vế tô) và trên cây đã có (phải xanh). Đó là phép gỡ thật của bước 1.
   - Ca Lookup bằng kéo thật ở khối 5 trong cùng tệp vẫn phải xanh.
3. vitest cho bộ đọc: một tệp hoặc ca mới cho `resolveSourceSelection` với một Range giả từ ô nguồn khối 0 sang ô nguồn khối 1 (happy-dom có `Range`): kết quả chỉ chứa chữ nguồn. Gỡ thật phần lọc thì ca đỏ đúng lý do.
4. Kiểm: `npm run build` (gồm `vue-tsc` cả hai tsconfig; ⚠️ Pha 3 làm đỏ nó vì chỉ chạy cấu hình mặc định), `npm run test:story 2-16`, các tệp vitest lưới/vùng chọn/glossary đã đổi hay liên quan, `npm run check:commands && npm run check:layout`, `npx eslint e2e`, e2e `grid-blocks.e2e.mjs` (nightly, 4 ca) và bàn `keo-cot-nguyen-van.e2e.mjs`.

## Chạy e2e
- Như mục "Giao việc Pha 4" ở trên: dựng `cargo build --locked --features wdio --manifest-path src-tauri/Cargo.toml`; e2e nạp Vite dev nên đổi `src/` không cần dựng lại binary. Bàn cục bộ: `TAURI_WEBDRIVER_PORT=4467 npm run test:e2e -- --spec _bmad-output/implementation-artifacts/2-16-ban-do/keo-cot-nguyen-van.e2e.mjs` hoặc `npx wdio run e2e/wdio.conf.mjs --spec <tệp>`. NGOÀI sandbox, trần mỗi lệnh bằng `perl -e 'alarm shift; exec @ARGV' <giây> …`.
- Bàn cục bộ dùng `cliclick` và cần màn hình mở khoá, cửa sổ lên trước. `[BÀN ĐO HỎNG]` là môi trường, không sửa mã.

## Ràng buộc
- KHÔNG commit, KHÔNG push. Không sửa plan hay story. Không xoá hai patch cũ.
- ⚠️ Không `git checkout`/`git stash`/`git restore` trên tệp có thay đổi chưa commit; bản gốc lấy bằng `git show <rev>:<path>`. Để đo cây "chưa có bước 1", tạm lưu diff bước 1 ra patch và `git apply -R`, rồi `git apply` lại.
- ⚠️ `rm` là alias `rm -i`: dùng `/bin/rm`. Không gọi `cat` không đối số.
- Comment theo AGENTS.md §Code comments. Không thêm phụ thuộc. Không đụng `src-tauri/`.

## Đầu ra
Mục `# Handoff Pha 4b -> Ice (story 2-16)` nối vào cuối tệp này, ngắn: tệp đã đổi, cơ chế trạng thái (khi bật, khi tắt), những chỗ đọc vùng chọn đã kiểm và kết quả, cách đo điểm ảnh, bảng đỏ/xanh của `DRAG_SOURCE_COLUMN` trước và sau bước 1, test xanh/đỏ kèm phạm vi, rủi ro.

# Handoff Pha 4b -> Ice (story 2-16)

Chưa commit. `src/` có thay đổi (cách thứ ba, không patch); hai patch cũ còn nguyên trong thư mục này.

## Đã đổi/thêm
- `src/panels/gridSourceSelection.ts` (mới): `isSourceColumnSelecting(selection, root)` (vùng chọn không thu gọn, anchor nằm trong `[data-col="src"]` của lưới) và `sourceColumnText(selection, root)` (chữ của từng ô nguồn mà Range chạm tới, cắt theo Range, nối bằng `\n` theo thứ tự hàng).
- `src/panels/GridPanel.vue`: `sourceSelecting` (ref) cập nhật ở đầu `onSourceSelectionChange` (sự kiện `selectionchange` đã có, 0 API `window`/`document` mới); gốc lưới mang `data-sel-src` khi bật; CSS scoped `.grid[data-sel-src] :deep(.col:not(.col-src)::selection)` (và con cháu) `background: transparent`; `resolveSourceSelection` dùng `sourceColumnText` khi bật (sau nhánh Hán Việt, không đổi); `@copy` trên gốc lưới chặn bản sao gốc và đặt `text/plain` bằng cùng chữ.
- Bật: anchor trong ô nguồn và vùng chọn chưa thu gọn. Tắt: ngay lượt `selectionchange` kế khi vùng chọn thu về caret, anchor rời cột nguyên văn, hoặc lượt chọn mới bắt đầu ở cột khác. Sống qua `mouseup` vì không gắn với chuột.
- `tests/frontend/gridSourceSelection.test.ts` (mới, 7 ca): Range ô nguồn khối 0 sang khối 1 ⇒ `"BB\nCC"` dù `selection.toString()` chứa chữ dịch; mọi hàng; trong một ô; `isSourceColumnSelecting` đúng/sai (thu gọn, anchor ở cột dịch, ngoài lưới). Gỡ thật phần lọc (trả `selection.toString()`): đỏ 2/7 đúng lý do (`'BBtr-Atr-B12CC'`); đã trả lại.
- `2-16-ban-do/keo-cot-nguyen-van.e2e.mjs`: `DRAG_SOURCE_COLUMN` đổi khẳng định (bỏ `containsNode`); `2-16-ban-do/pngRegion.mjs` (mới): giải PNG bằng `node:zlib` và đếm điểm ảnh khác trong một hình chữ nhật.

## Chỗ đọc vùng chọn đã kiểm
- Lượt tra (`currentSelectionText`), Glossary quick add (`currentSelectionTextForGlossaryQuickAdd`), Concordance (`currentSelectionTextForConcordance`): cả ba gọi `surface.resolve` của bề mặt nguồn = `resolveSourceSelection` ⇒ đã lọc; e2e đo `currentSelectionText()`. Hai cái sau không có ca e2e riêng (cùng đường `resolve`).
- `⌘C`: đã chặn và thay; e2e đo bằng `execCommand('copy')` + listener ở `document`.
- Không đổi: `currentSelectionSegmentIdForConcordance` (đọc anchor), `glossaryQuickAddState`/`glossaryConfirmStripState` (chỉ lưu/trả Range).
- Chưa kiểm: kéo thả chữ đã chọn (drag-and-drop) và menu ngữ cảnh "Copy" của hệ điều hành vẫn dùng Range gốc, có thể mang chữ bản dịch.

## Đo điểm ảnh
Hai ảnh `browser.takeScreenshot()` toàn cửa sổ (trước/sau kéo OS từ hàng 198 tới 202), đếm điểm ảnh khác trong rect (CSS px × png.width/innerWidth) của các ô thăm dò; rect trước = rect sau được khẳng định. ⚠️ `takeElementScreenshot` trên driver này trả cả cửa sổ (360.800 byte, giống ảnh full) và treo ở trang 1.200 câu, nên không dùng. Máy Ice dpr=2 (đo ra `scale` = 2); phép so là trước/sau cùng máy nên dpr không vào kết quả.

## Bảng `DRAG_SOURCE_COLUMN` (bản debug `--features wdio`, WebKit 605.1.15, Vite dev, ngoài sandbox, dpr 2)
| Điểm ảnh khác | tgt h198 | tgt h199 | số h200 | số h201 | số/vạch h198-199, vạch h200-201 | nguồn h198 | nguồn h199 | Lookup có chữ dịch | copy |
|---|---|---|---|---|---|---|---|---|---|
| trước bước 1 (đảo `GridPanel.vue`) | 39.736 | 39.736 | 2.445 | 2.452 | 0 | 32.306 | 40.151 | có | không bị chặn |
| sau bước 1 | 0 | 0 | 0 | 0 | 0 | 32.306 | 40.151 | không | chặn, chữ nguồn, không chữ dịch |
- Trước: ĐỎ ở vế tô và vế đọc. Sau: XANH, ca Lookup bằng kéo thật ở khối 5 cùng tệp cũng xanh. Range vẫn chứa ô dịch (khẳng định `rangeHasTranslation`), đúng thiết kế.

## Test (phạm vi)
- Xanh: `npm run build` (gồm `vue-tsc` hai tsconfig), `npm run test:story 2-16` (20 ca), 20 tệp vitest tên chứa grid/selection/glossaryQuick (166 ca), `check:commands`, `check:layout`, eslint `e2e` + các tệp đã đổi; e2e `grid-blocks.e2e.mjs` 4/4, bàn `keo-cot-nguyen-van.e2e.mjs` 2/2.
- ĐỎ có từ trước, không do Pha này: `npm run check:tokens` dừng ở "sàn `COMPONENT_FILE_FLOOR` = 164 đã trôi dưới 80% số thật 207" (`scripts/check-tokens.mjs`, không đổi; Pha này không thêm `.vue`) nên Kiểm B không chạy trên `background: transparent`; giá trị đó nằm trong `ALLOWED_COLOR_KEYWORDS`. Cần nâng sàn lên 176 riêng.
- Không chạy: toàn bộ suite, spec e2e khác, nghiệm thu release.

## Rủi ro
- Tô bật sau `selectionchange` (bất đồng bộ): có thể một khung hình chữ dịch hiện tô khi kéo; chưa đo ảnh giữa lúc kéo, chỉ trạng thái nghỉ sau thả.
- `sourceColumnText` quét mọi ô nguồn của lưới mỗi lượt đọc (9.850 câu ⇒ cỡ vạn `intersectsNode`) chỉ khi bật và chỉ lúc tra/chép, chưa đo thời gian.
- Chữ ghép giữa các ô là `\n`; `toString()` cũ của vùng chọn trong một khối có thể ngắt dòng khác chút (khoảng trắng); chưa so với `cc60020` bằng chuột thật.

## Sau Pha 4b (người giao việc, 2026-10-24)
- `sourceColumnText` gọi `intersectsNode` trên mọi ô nguồn: ca Lookup khối 5 của `gridBlocks.test.ts` tốn 11 s trên happy-dom và hết trần 30 s khi chạy đủ bộ. Nay chỉ duyệt các ô nguồn từ ô chứa đầu khoảng chọn tới ô chứa cuối (khoảng chọn liền theo DOM), về cả danh sách khi một đầu không ở ô nguồn: 1,15 s. Chưa chạy lại bàn `keo-cot-nguyen-van.e2e.mjs` sau thay đổi này (bàn điều khiển chuột thật của máy).
- `check:tokens` báo đỏ "có từ trước" là sai: ở `fd13e19` nó xanh, chính các tệp mới của story làm sàn trôi. `check:doc-refs` cũng vậy. `check:i18n` đỏ thật từ `043f3bc`. Cả ba đã sửa riêng; 14 cổng `pre-push` và vitest đủ bộ xanh.
