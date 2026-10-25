# NFR18 đường flush — bàn giao sau lượt đo HEAD

Plan: `_bmad-output/initiative-auratranslate/plan-nfr18-duong-flush.md`. Lượt này chỉ đo; chưa sửa mã sản phẩm (Design Notes: đo trượt thì dừng, phần sửa giao agent mới).

## Điều kiện đo

- Build: release `npx tauri build --bundles app` trên HEAD `99f3fc1` (cây sạch), `wal_threshold_bytes` = 4 MiB (mặc định), không tiêm bench. Log: `build-head.log`.
- Chương: `chapters/s122.txt`, 122 câu (122 segment trong `project.db`), nguồn Tiếng Trung, một Chương.
- n=20 mẫu `VALID` mỗi nhịp, 22 lượt bắn (2 `RIG_FAIL`, đều do bàn đo). Hàng rào chiều âm `fence.sh` trước và sau: đứng.
- loadavg ngay trước kill: 40 phím/giây 2,6–13,8 (trung vị ~4,2); 10 phím/giây 3,3–5,3 (đầu lượt 16,7). Máy KHÔNG rảnh (Orca và các tiến trình khác chạy nền); phán quyết "đạt" cần lượt lại trên máy rảnh, nhưng 20/20 trượt ở 40 phím/giây thì khó là tải nền.
  🔵 2026-10-24 (Ice): câu "phán quyết đạt cần lượt lại trên máy rảnh" là sai. App phải đạt NFR18 và ngân sách mỗi phím dưới đúng tải nền này (Docker/VM, loadavg có lúc tới 80); tải máy không phải cớ để loại hay chạy lại phiên. Mỗi phiên vẫn ghi loadavg, A/B đo xen kẽ.

## Kết quả (`grid-table.sh`, `LABELS="h40 h10" PFX=`)

| Nhịp | n | min | trung vị | max | mẫu > 5 s |
|---|---|---|---|---|---|
| 40 phím/giây (`type-driver.sh`, cụm câu) | 20 | 6,98 | 12,24 | 19,40 | 20/20 |
| 10 phím/giây (`RATE=10hz`, một tiến trình) | 20 | 0,34 | 3,14 | 7,83 | 7/20 |

Chi tiết: `kill2-h40.tsv`, `kill2-h10.tsv`, `load-h*.tsv`, `typing2-h*-N.log`. Mốc 10 phím/giây lấy ngay trước khi gõ `]` của `[n]`, nên không cộng thời gian gõ cả câu như 40 phím/giây (cận trên, ~0,4 s theo 2.4).

Đối chiếu tháng 8 (lưới 2.4, `c097eb3`, loadavg 7–8,5): 7,7–24,4 s, trung vị 13,4–16,6 s. Chia khối 2-16 không đổi kết luận ở 40 phím/giây (trung vị 12,2 s).

## Đọc số

- NFR18 trượt ở cả hai nhịp trên HEAD. Giả thuyết "chủ yếu là phím xếp hàng ở lưới cũ" không đứng một mình: 40 phím/giây vẫn 7–19 s.
- Cửa sổ ở 10 phím/giây lớn nhất 7,8 s với cùng trần cứng 5 s ⇒ có độ trễ 0,7–2,8 s ngoài trần ở 7/20 mẫu; ở 40 phím/giây độ trễ ngoài trần 2–14 s, tăng theo tải phím ⇒ nghi phím xếp hàng ở webview, không phải một lượt ghi chậm.
- Chưa tách được "phím chờ" khỏi "phím đã tới → commit": bản release trần không có mốc phím đã tới. Cần mốc đó trước khi sửa.

## Nghi can (chưa kiểm, theo plan Code Map)

1. Webview: `noteEditorEdit` chạy đồng bộ trên mỗi `input`, timer `setTimeout` bị trễ khi luồng chính bận; `flushEditorNow` chờ lô đang bay rồi mới chạy lô sau (`editorPanelState.ts` quanh `inFlight`). Khi người gõ tiếp trong lúc lô bay thì `onFlushed` không xoá `dirty` (văn bản đã đổi) nên `firstChangeAt` giữ nguyên, `due` ở quá khứ, lô kế chạy ngay: mỗi lượt ghi kém câu gõ trong lúc bay.
2. Rust: `save_segment_targets` là `pub fn` đồng bộ, Tauri v2 chạy trên luồng chính và giữ mutex `OpenWorkState` suốt lượt ghi; `flush_segment_targets` hai transaction nối tiếp; writer đơn tuyến `synchronous = FULL`.
3. Phím nóng xếp hàng (2.4 đo bộ đếm 3 → 13): nếu luồng chính bận vì lệnh đồng bộ thì phím cũng xếp hàng ⇒ nghi can 2 có thể là nguyên nhân gốc của 1.

## Việc cho agent kế

1. Thêm mốc phím đã tới: script trong trang ghi `event.timeStamp` và mốc `flushEditorNow` bắt đầu/xong (mẫu `../2-15-ban-do/bench-215.js`, `build-215.sh`: nối vào bundle vì CSP chặn script riêng), tách `window_s` thành phím chờ và tới→commit.
2. Sửa đúng phần trượt, kèm test đỏ khi gỡ bản sửa (`tests/frontend/editorFlush.test.ts`, `src-tauri/tests/flush_cadence_contract.rs` chỉ mô phỏng, không phải bằng chứng NFR18).
3. Đo lại cùng điều kiện, máy rảnh; vẫn trượt thì plan sang `blocked` kèm bảng số.

## Bàn đo: cái đã sửa so với `../2-4-ban-do/` (UI nhập đã đổi từ tháng 8)

- `setup-gui.sh`: luồng mới là `⌘N` → tên → Tab×4 → đường dẫn → Tab → Space (màn xem trước) → Page Down ×8 → bấm "Xác nhận và nhập" tại `(+102,+836)` so gốc cửa sổ. Một cú `cliclick c:` đơn lẻ đôi khi không ăn; mọi cú bấm nay là `m:` + `w:300` + `c:`.
- `focus-segment.sh`: ứng viên đầu `(437,167)` là cột đích hàng 1 trên lưới chia khối; ba ứng viên khác giữ làm đường lùi.
- `type-driver.sh`: `RATE=10hz` gọi `type-10hz-indexed.applescript` (một tiến trình, `delay 0.1`, ghi `n ts len` ngay trước `]`).
- `grid-table.sh`: nhận `LABELS` và `PFX`. `build-bench.sh` trỏ `bench.js` trong thư mục này.
- Chạy một nhịp: `RATE=10hz BENCH_SRC=$PWD/chapters/s122.txt ./kill-campaign-v2.sh 20 h10 20 55 40` (bỏ `RATE` cho 40 phím/giây). Tham chiếu app hard-code `src-tauri/target/release/bundle/macos/AuraTranslate.app`: dựng lại bản release sau mỗi bản sửa.
- `⌘2` không chuyển Workspace trong bàn đo (đã thử); bấm tab Workspace `(+101,+46)`.

## Pha chẩn đoán (agent thứ hai): Rust loại, nghi can là phím xếp hàng ở webview

Cách đo: tạm thêm một dòng `eprintln!` vào `save_segment_targets` (`wire.rs`, đã gỡ, cây sạch) ghi mốc nhận lệnh, mốc có khoá `OpenWorkState`, mốc xong và chỉ số `[n]` lớn nhất trong lô. Dựng release, chạy `kill-campaign-v2.sh 5 i40 20 55 8` (40 phím/giây, 122 câu, 5 mẫu VALID: 6,35–8,80 s, vẫn trượt). Log: `app2-i40-*.log` (dòng `NFR18 …`), `kill2-i40.tsv`.

- Phía Rust không phải nguyên nhân: lượt 3 có 140 lệnh, thời lượng trung vị 0,5 ms, lớn nhất 1,1 ms; chờ khoá tối đa 0,1 ms. Hai transaction, `synchronous = FULL` và mutex không đáng kể ⇒ nghi can 2 bị loại.
- Lô tới Rust dày (~4/giây, cách nhau trung vị 28 ms), nên timer/lô-chờ-lô không giữ chữ lại. Nhưng chỉ số `[n]` trong lô tới Rust trễ so với mốc bơm tăng dần trong phiên: 1,3 s → 2,5 → 4,3 → 5,5 s (lượt 3). Chữ chưa vào `noteEditorEdit` thì flush chưa có gì để gửi ⇒ cửa sổ mất dữ liệu là phím xếp hàng ở luồng chính webview (xử lý `input` chậm hơn 40 phím/giây), không phải đường flush.
- Chưa đo: mốc phím đã tới ở phía webview (nên tách xử lý `input` khỏi hàng đợi OS), và đường 10 phím/giây có cùng cơ chế không.

Việc kế: profile chi phí mỗi `input` trong `GridPanel.vue` (`onEditInput` → `noteEditorEdit` → render lưới/`triggerRef(editedShape)`/`datThongBao`), cùng lớp việc với Story 2-15; sửa ở đó, không ở `editorFlush`/`writeSchedule`. Đổi hành vi flush không rút được cửa sổ.

## Pha A/B (agent thứ ba): chặn cơn flush không rút cửa sổ

Hai bản release cùng mốc `99f3fc1`, cùng `eprintln!` tạm ở `save_segment_targets` (đã gỡ cùng bản B, cây sạch). A giữ nguyên hành vi; B: sau lô `saved` mà tập chờ còn sót thì không gọi lại ngay (bỏ đệ quy `flushEditorNow` và đặt sàn `now+5000` cho `armFlushTimer`). 40 phím/giây, 122 câu, 4 MiB, xen kẽ A/B, loadavg đầu lượt 4,7–12,8 (máy không rảnh), 6 mẫu VALID mỗi bản. Script `ab/run-ab.sh`, log `kill2-ab[AB]N.tsv`, `app2-ab*`, `typing2-ab*`. Số lượt flush đếm từ dòng `NFR18` của log app; "trễ [n]" là mốc nhận lô trừ mốc bơm của `[n]` lớn nhất trong lô (chỉ các `[n]` có mốc trong `typing2`).

| Bản | `window_s` mỗi mẫu | trung vị | lượt flush mỗi mẫu | trễ [n] lớn nhất mỗi mẫu (s) |
|---|---|---|---|---|
| A | 4,69 · 6,42 · 7,51 · 7,23 · 6,15 · 11,84 | 6,8 | 9 · 50 · 114 · 216 · 55 · 85 | 3,7 · 4,9 · 3,5 · 5,2 · 4,0 · 9,2 |
| B | 7,74 · 10,66 · 9,14 · 10,05 · 8,21 · 5,92 | 8,7 | 9 · 12 · 10 · 10 · 11 · 6 | 4,9 · 8,1 · 6,3 · 5,1 · 4,6 · 2,5 |

Đọc: B cắt số lượt flush từ 9–216 xuống 6–12 mà `window_s` không giảm (trung vị 6,8 -> 8,7; 5/6 mẫu B vẫn > 5 s, A 5/6 > 5 s). n=6, tải máy lệch, nên không kết luận B tệ hơn, chỉ kết luận B không tốt hơn. Cơn flush dồn dập tồn tại (A tới 216 lượt) nhưng không phải thứ giữ cửa sổ; khớp pha hai: phím xếp hàng ở webview. Hướng sửa vẫn là chi phí mỗi `input` ở `GridPanel.vue` (cùng lớp 2-15), chờ Ice chọn.

## Pha phân bổ chi phí mỗi phím (plan `plan-giam-chi-phi-moi-phim-go-luoi`)

Bàn đo: `bench.js` mục ⑪ (mốc `performance.now()` bằng listener capture/bubble ở `window`, không bọc hàm sản phẩm; Ctrl+Alt+Shift+8 đổ `__bench_keys__`; Ctrl+Alt+Shift+9/0/- bật biến thể WebKit tạm). Chạy: `keycost-session.sh <nhãn> <phiên> <giây>` (`RATE=10hz`, `VARIANT=n`, `CTRL=noop|attr|style|sleep`), lô `keycost-batch.sh`, `keycost-variants.sh`, `keycost-ctrl.sh`, `keycost-ctrl2.sh`; đọc bằng `keycost-summary.py` và `keycost-report.py`. Số liệu thô `keys-*.json`.

Điều kiện: release HEAD `e33d555` + bench (build-bench.sh), Chương `s122.txt` 122 câu, một cột đích, 30 s mỗi phiên, 40 phím/giây = `type-driver.sh`, loadavg trước gõ 7,8–16 (không loại phiên nào vì tải). Một phiên `focus` RIG_FAIL, chạy lại ở vòng sau. Bản bench có cộng thêm mã biến thể nên bản "gốc" ở dưới là bản bench, không phải release trần.

Chặng, p50/p95 ms, 40 phím/giây, n=2 phiên gốc (`base40`, ~850 phím mỗi phiên):

| Chặng | p50 | p95 |
|---|---|---|
| keydown (keys.ts + onEditKeydown) | 0 | 1 |
| beforeinput (onBeforeInput) | 0 | 0 |
| input (reportEdit → noteEditorEdit) | 0 | 1 |
| selectionchange (hai listener) | 0 | 0 |
| WebKit: beforeinput xong → input | 14–15 | 26–29 |
| input xong → frame kế (rAF) | 6–7 | 77–88 |
| tổng JS handler mỗi phím | 0 | 1 (max 5) |
| hàng đợi: `event.timeStamp` → keydown tới handler | 2,4 s (đầu) → 8–10 s (cuối) | 4,6 → 13 s |

Đọc số:
- Mã JS của sản phẩm trên đường phím tốn ≤ 1 ms p95 mỗi phím. Các nghi can JS (`unwrapProofread` ×2, `scheduleRepaintProofread`, `setEditorCaret`/position timer, `datThongBao`, `keys.ts handle`, `selectionchange`) không thể là chặng trội; gỡ thử chúng không thể rút quá 1 ms nên không dựng bản gỡ (số đo trực tiếp thay cho phép gỡ).
- Chặng trội nằm ở WebKit: lệnh sửa 14 ms p50 và đuôi layout/vẽ sau `input` (p95 ~80 ms). Hàng đợi tăng đều suốt phiên (độ trễ `[n]` tới `noteEditorEdit`: quý đầu 0,9–1,7 s, quý cuối 9–10,4 s, `base40-1/2`), khớp phân tích trước đó.
- 10 phím/giây (`base10`, n=3): hàng đợi 1–2 ms, `[n]` tới `noteEditorEdit` 0,02–0,05 s và không tăng, cùng đuôi frame p95 ~70 ms. ⇒ 7/20 mẫu trượt ở 10 phím/giây KHÔNG do phím xếp hàng ở webview; nguyên nhân khác (chưa giải thích, ngoài phạm vi các số trên).

Biến thể WebKit tạm (CSS/thuộc tính chèn vào trang trước khi gõ, 40 phím/giây, xen kẽ, n=3 mỗi nhóm, `input→rAF` p95 ms / hàng đợi p50 phần ba cuối s / độ trễ `[n]` quý cuối s):

| Nhóm | in→rAF p95 | hàng đợi cuối | độ trễ [n] cuối |
|---|---|---|---|
| gốc `dnone` | 72, 89, 71 (cộng `w0`/`cnone`: 52–74) | 10,3 · 13,0 · 13,2 | 10,7 · 14,0 · 13,6 |
| đối chứng chỉ đổi thuộc tính ô (`dattr`) | 97, 72, 64 | 12,7 · 11,3 · 9,8 | 12,9 · 12,7 · 10,9 |
| đối chứng thêm thẻ `<style>` rỗng (`dstyle`) | 82, 96, 69 | 13,1 · 13,2 · 9,9 | 14,3 · 13,3 · 10,6 |
| đối chứng chỉ bấm phím / chỉ chờ (`cnoop`,`csleep`) | 55–65 / 49–55 | 7,9–10,6 / 4,4–7,6 | gần gốc |
| `contain: layout style` trên `.cell-tgt` (`dv3`,`w3`) | 10, 9, 8 · 9, 9, 9 | 6,6 · 4,3 · 3,3 · 3,7 · 5,1 · 4,4 | 6,9 · 4,5 · 3,7 · 3,7 · 5,2 · 5,2 |
| `content-visibility:auto` trên `.cell` (`w2`) | 9, 8, 9 | 3,2 · 2,6 · 2,8 | 3,2 · 3,1 · 2,6 |
| `spellcheck=false` + `autocorrect=off` trên ô (`w1`) | 9, 9, 9 | 3,9 · 2,9 · 2,6 | 4,1 · 3,0 · 3,0 |

Ba biến thể thật đều cắt đuôi `input→rAF` p95 từ 50–97 xuống 8–10 ms và rút hàng đợi cuối còn 2,6–6,9 s; ba đối chứng (đổi thuộc tính, thêm `<style>`, bấm phím/chờ) giữ nguyên số gốc ⇒ hiệu ứng không do chính việc chạm DOM. Bốn biến thể đầu được đo trong lượt hai (`w*`, `var*` đã xoá sau khi sót nhãn sai do `seq 1 0`; số còn lại ở bảng và ở `keys-w*`). Còn khoảng cách: `webkit_edit` vẫn 13–15 ms p50 ở mọi nhóm và hàng đợi VẪN tăng dần trong phiên ở cả ba biến thể (cuối phiên 2,6–6,9 s so với ≤ 5 s cần cho NFR18 sau cộng thời gian gõ câu) ⇒ riêng CSS chưa chắc đưa 40 phím/giây về dưới 5 s; chưa đo kill NFR18 với biến thể nào. Chưa hiểu vì sao ba cơ chế khác nhau (đặc biệt `spellcheck`) cho cùng đuôi 9 ms; trước khi chọn nên đo thêm một biến thể cô lập.

Hàng rào `fence.sh` chiều âm trước/sau: đứng. Cây sản phẩm không đổi (chỉ `bench.js` và script bàn đo).

Hướng chờ Ice chọn (số ở trên): (A) `contain: layout style` cho `.cell-tgt`; (B) `content-visibility:auto` + `contain-intrinsic-size` cho `.cell`; (C) `spellcheck="false"` (đổi hành vi người dùng: mất gạch chân chính tả); (D) kết hợp rồi đo kill NFR18. Cả (A)–(D) ở CSS/thuộc tính ô, không ở handler JS.

## Kết quả hướng A (`contain: layout style` trên `.cell-tgt`)

Build: release `npx tauri build --bundles app` (không bench) trên HEAD `e33d555` + bản sửa CSS ở `GridBlock.vue`; 122 câu `s122.txt`, n=20 mẫu `VALID` mỗi nhịp (20 lượt bắn, không RIG_FAIL), hàng rào `fence.sh` trước/sau: đứng. loadavg trước kill: 40 phím/giây 4,95–14,62; 10 phím/giây 2,63–6,96 (máy không rảnh, không loại phiên). Số thô: `kill2-n40.tsv`, `kill2-n10.tsv`, `load-n*.tsv`, `run-n40.out`, `run-n10.out`.

| Nhịp | n | min | trung vị | max | mẫu > 5 s | trước (HEAD) |
|---|---|---|---|---|---|---|
| 40 phím/giây | 20 | 4,57 | 10,7 | 20,73 | 19/20 | 20/20, trung vị 12,24 |
| 10 phím/giây | 20 | 1,67 | 3,14 | 5,99 | 4/20 | 7/20, trung vị 3,14 |

⇒ AC1 KHÔNG đạt; plan sang `blocked`. Hướng A cắt đuôi layout/vẽ (xem bảng biến thể) nhưng hàng đợi 40 phím/giây vẫn tăng nên cửa sổ vẫn vượt trần. Chưa đo lại `[n]` tới `noteEditorEdit` bằng bench sau bản sửa (AC2 chưa kiểm). Hướng chờ Ice: B/C/D ở trên, hoặc giảm lệnh sửa WebKit 14 ms/phím.
