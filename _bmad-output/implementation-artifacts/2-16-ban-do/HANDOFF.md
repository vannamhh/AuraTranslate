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
