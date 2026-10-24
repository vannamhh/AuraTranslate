# NFR18 đường flush — bàn giao sau lượt đo HEAD

Plan: `_bmad-output/initiative-auratranslate/plan-nfr18-duong-flush.md`. Lượt này chỉ đo; chưa sửa mã sản phẩm (Design Notes: đo trượt thì dừng, phần sửa giao agent mới).

## Điều kiện đo

- Build: release `npx tauri build --bundles app` trên HEAD `99f3fc1` (cây sạch), `wal_threshold_bytes` = 4 MiB (mặc định), không tiêm bench. Log: `build-head.log`.
- Chương: `chapters/s122.txt`, 122 câu (122 segment trong `project.db`), nguồn Tiếng Trung, một Chương.
- n=20 mẫu `VALID` mỗi nhịp, 22 lượt bắn (2 `RIG_FAIL`, đều do bàn đo). Hàng rào chiều âm `fence.sh` trước và sau: đứng.
- loadavg ngay trước kill: 40 phím/giây 2,6–13,8 (trung vị ~4,2); 10 phím/giây 3,3–5,3 (đầu lượt 16,7). Máy KHÔNG rảnh (Orca và các tiến trình khác chạy nền); phán quyết "đạt" cần lượt lại trên máy rảnh, nhưng 20/20 trượt ở 40 phím/giây thì khó là tải nền.

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
