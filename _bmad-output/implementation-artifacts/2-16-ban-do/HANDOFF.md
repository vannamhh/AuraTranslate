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
