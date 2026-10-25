---
title: 'NFR18 trượt 120/120 mẫu ở đường flush'
type: 'bugfix'
ticket: ''
created: '2026-10-24'
status: 'dropped'
blocked_reason: 'Đường flush đã bị loại: Rust ≤ 1,1 ms, chặn cơn flush không rút cửa sổ. Độ trễ nằm ở chi phí mỗi input của lưới. Ice chọn mở việc mới để giảm chi phí đó (cùng lớp 2-15), ngoài plan này.'
baseline_revision: '99f3fc1c0c46326e6743b6a60e63f96889c480c4'
route: 'full'
route_source: 'auto'
risk: 'high'
review: ''
review_source: ''
lenses_ran: []
review_loop_iteration: 0
context: []
---

<frozen-after-approval reason="human-owned intent — do not modify unless human renegotiates">

## Intent

**Problem:** Lưới sáu điểm của Story 2.4 (2026-08-18, bản `c097eb3`, loadavg 7–8,5) đo cửa sổ mất dữ liệu sau `SIGKILL` là 7,7–24,4 s, trung vị 13,4–16,6 s, so với trần NFR18 là 5 s. `wal_threshold_bytes` đã bị loại, còn lại đường flush. Phiếu #107 giao đường flush cho Story 10.9, nhưng 10.9 đã `done` mà không phủ NFR18, nên phần này hiện không có chủ.

**Approach:** Đo lại trên HEAD trước: lưới đã chia khối (2-16) và bỏ được hàng phím chờ cỡ ~15 s mà 2.4 ghi nhận, nên số tháng 8 có thể chủ yếu là phím xếp hàng chứ không phải flush. Chỉ khi số mới còn trượt mới tách cửa sổ thành hai phần (phím chờ và phím đã tới tới lúc commit) rồi sửa đúng phần trượt.

**Quyết định của Ice (2026-10-24):**
- Đo trước, sửa có điều kiện, và giữ một plan gộp. Ice chấp nhận plan dài 2.686 token.
- Đo ở hai nhịp gõ: cụm câu khoảng 40 phím/giây của 2.4 (`type-driver.sh`, để so với tháng 8) và 10 phím/giây của 2-15. Phán quyết NFR18 phải đạt ở cả hai nhịp.
- Agent chạy lượt GUI trong phiên, Ice rời máy. Mỗi bước có trần thời gian (`perl -e 'alarm …'`) và có sentinel phát từ `trap … EXIT`.
- `Chủ:` của mục #107 và của plan này là Story 2.4. Số đo mới giao về story đó.
- Sau lượt chẩn đoán hai: đo cơn flush trước, chưa sửa mã sản phẩm. Một bản đo tạm chặn cơn flush dồn dập rồi xem phím còn xếp hàng không. Hướng sửa chọn sau, theo số đo.
- Sau lượt A/B: mở việc mới để giảm chi phí mỗi `input` ở lưới (cùng lớp 2-15), ngoài plan này. Plan này dừng ở `blocked`.

## Boundaries & Constraints

**Always:** Nhịp `EDITOR_IDLE_MS = 2000` / `EDITOR_HARD_CAP_MS = 5000` giữ nguyên (FR100, AD-35). Mỗi số đo ghi bản build, cỡ Chương, nhịp gõ, loadavg và n. Đo trên bản release dựng theo `build-bench.sh`, máy rảnh, màn hình mở khoá. Hàng rào chiều âm của bàn đo (`fence.sh`) đứng trước và sau mỗi lượt.

**Never:** Không đổi một bất biến của spine (nhịp flush, `synchronous`, mô hình writer đơn tuyến). Nếu cần thì dừng và chuyển thành AD cho Winston. Không đóng hàng §Deferred WAL của spine (chủ: Winston). Không làm Task 3 NFR2 của Story 2.4. Không đánh dấu NFR18 đạt bằng suy luận hay bằng vitest.

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected Output / Behavior | Error Handling |
|----------|--------------|---------------------------|----------------|
| Gõ liên tục rồi sập | gõ 20–55 s, `SIGKILL` ngay khi ngừng | chỉ số lớn nhất còn trong `project.db` được bơm ≤ 5 s trước lúc kill | mẫu > 5 s ghi vào bảng, không bỏ |
| Sập lúc rảnh | đã ngừng gõ > 2 s | 0 s mất (lớp `IDLE_OK` của kill v2) | giữ trong mẫu |
| Lượt bàn đo hỏng | mất tiêu điểm > 5 %, kill trượt | `RIG_FAIL` hoặc `AMBIG`, bắn lại tới đủ mẫu hợp lệ | in số lượt đã bắn và số mẫu hợp lệ |
| Ghi lỗi giữa phiên | `save_segment_targets` trả lỗi | lượt thử lại ≥ 2 s, `dirty` giữ chữ | tính vào cửa sổ, không che |

</frozen-after-approval>

## Code Map

- `_bmad-output/implementation-artifacts/2-4-ban-do/` -- bàn đo kill v2: `run-grid.sh` (một điểm: đặt nhãn và ngưỡng 4 MiB), `kill-campaign-v2.sh` (`window_s` = mốc kill − mốc bơm của chỉ số `[n]` lớn nhất còn trong kho, `:53`), `type-driver.sh` (cụm `osascript keystroke` cả câu, khoảng 40 phím/giây, nghỉ 0,6–1,4 s), `grid-table.sh` (bảng AC8), `setup-gui.sh` · `focus-segment.sh` · `front.sh` (cửa vào ô theo toạ độ khung nhìn, `⌘2` sang Workspace). Dựng và hiệu chuẩn trên mã tháng 8, chưa chạy trên DOM chia khối của 2-16.
- `_bmad-output/implementation-artifacts/2-15-ban-do/` -- bộ gửi 10 phím/giây trong một tiến trình, kèm script trong trang ghi `event.timeStamp` tới frame đầu có chữ. Dùng lại nếu cần tách phần phím chờ.
- `src/layout/writeSchedule.ts` (`createWriteSchedule`) -- `due = min(now+idle, firstChangeAt+cap)`. Trần không reset theo phím. Không sửa.
- `src/panels/editorFlush.ts:48,61,83` -- hằng 2000/5000/2000. `onFlushed :142-151` chỉ gọi `onWrite` khi `dirty` rỗng.
- `src/panels/editorPanelState.ts` -- `armFlushTimer :738-749` (một `setTimeout`, đặt lại mỗi phím), `noteEditorEdit :758-814` (đồng bộ, không đệm), `flushEditorNow :835-902`: chờ lô đang bay rồi mới chạy lô sau (`:845`), sau lỗi thì lùi ≥ 2 s (`:900`). Các nghi can ở phía webview là timer bị trễ khi luồng chính bận và lô sau phải chờ lô trước.
- `src/panels/GridPanel.vue:1699-1712` `reportEdit`/`onEditInput` -- nơi duy nhất gọi `noteEditorEdit`.
- `src-tauri/src/commands/segment/wire.rs:143-167` -- `save_segment_targets` là `pub fn` đồng bộ (Tauri v2 chạy lệnh đồng bộ trên luồng chính, chưa kiểm) và giữ mutex `OpenWorkState` suốt lượt ghi. `confirm.rs:515-523` `flush_segment_targets` chạy hai transaction nối tiếp. `core/store/writer.rs:246-308` là writer đơn tuyến, `synchronous = FULL` (`pragmas.rs:245`). Nghi can phía Rust.
- `tests/frontend/editorFlush.test.ts`, `src-tauri/tests/flush_cadence_contract.rs` -- chỉ mô phỏng không có độ trễ IPC, writer hay `SIGKILL`. Đừng tính chúng là bằng chứng cho NFR18.

## Tasks & Acceptance

**Execution:**
- [x] `2-4-ban-do/` (sao sang `_bmad-output/implementation-artifacts/nfr18-flush-ban-do/`) -- dựng bản release trên HEAD, chạy smoke một lượt kill và sửa cửa vào ô cho DOM chia khối nếu cần -- bàn đo phải còn sống thì số mới có nghĩa.
- [x] cùng thư mục -- thêm chế độ 10 phím/giây cho bộ lái gõ, theo mẫu một tiến trình của `2-15-ban-do/`, vẫn ghi mốc bơm cho từng `[n]`; rồi ở điểm 4 MiB chạy mỗi nhịp tới ≥ 20 mẫu hợp lệ và in bằng `grid-table.sh` -- ra phán quyết trên HEAD.
- [x] `src/panels/editorFlush.ts` / `editorPanelState.ts` (bản đo TẠM, gỡ sau lượt đo) -- dựng hai bản release cùng mốc `99f3fc1`, mỗi bản ghi số lượt flush, mốc nhận lô và `[n]` lớn nhất của lô (theo khuôn `eprintln!` ở `wire.rs` của lượt hai): bản A giữ nguyên hành vi; bản B chặn cơn flush (sau một lô `saved` mà tập chờ còn sót thì không gọi lại ngay, chờ tới trần kế). Chạy xen kẽ A/B, 40 phím/giây, 122 câu, ≥ 5 mẫu mỗi bản, cùng tải máy. So số lượt flush, độ trễ `[n]` tới Rust theo thời gian và `window_s`. Ghi bảng vào `HANDOFF.md`, gỡ bản đo, rồi dừng và báo lại -- chọn hướng sửa bằng số.
- [ ] (chờ Ice chọn hướng sau bước trên) nếu trượt: thêm mốc "phím đã tới" vào bàn đo để tách cửa sổ thành phím chờ và tới→commit; sửa nghi can trúng số (lệnh đồng bộ trên luồng chính, hai transaction, lô chờ lô, timer trễ) kèm test làm đỏ được khi gỡ bản sửa; đo lại cùng điều kiện -- sửa đúng phần đo thấy.
- [x] `deferred-work.md` -- nối dòng đóng hoặc 🟡 vào mục phiếu #107 (`grep -n "#107"`), kèm số và `→ Chủ: Story 2.4` -- mục đang mồ côi.

**Acceptance Criteria:**
- Given bản release HEAD ở 122 câu, máy rảnh, when chạy kill v2 tại 4 MiB tới ≥ 20 mẫu hợp lệ ở mỗi nhịp gõ, then mọi mẫu `VALID` có `window_s` ≤ 5 s ở cả hai nhịp. Nếu vẫn trượt sau bản sửa thì plan sang `blocked` kèm bảng số.
- Given một bản sửa ở đường flush, when gỡ nó khỏi mã sản phẩm và chạy lại test canh nó, then test đỏ đúng vì lý do đang đo.
- Given mục phiếu #107 trong sổ nợ, when đọc dòng cuối, then nó ghi số đo trên HEAD, bản build, loadavg và một `Chủ:` mà `check:debt-owner` nhận.

## Implementation Notes

- Lượt đo HEAD (release `99f3fc1`, 122 câu, 4 MiB, n=20 hợp lệ mỗi nhịp, loadavg 3–14, máy không rảnh): 40 phím/giây 6,98–19,40 s, 20/20 mẫu > 5 s; 10 phím/giây 0,34–7,83 s, 7/20 mẫu > 5 s. Trung vị theo `grid-table.sh` là trung vị dưới. NFR18 trượt ở cả hai nhịp, nên lưới chia khối 2-16 không đủ để đạt.
- Bàn đo đã chép và sửa cho UI nhập mới; chi tiết và lệnh chạy ở `nfr18-flush-ban-do/HANDOFF.md`. Pha sửa bắt đầu từ §Việc cho agent kế của tệp đó: mốc phím đã tới trước, sửa sau.

- Lượt hai: 140 lượt ghi trong 57 s, khoảng cách trung vị 28 ms (p10 17, p90 76), trong khi nhịp 2 s/5 s chỉ cho khoảng 12–20 lượt. Lý do: gõ liên tục trong một ô thì tập chờ không bao giờ rỗng, nên `onFlushed` (`editorFlush.ts:142-151`) không gọi `onWrite` và `firstChangeAt` đứng yên ở mốc đầu. Hết 5 s đầu, `due` nằm trong quá khứ, nên mỗi lượt đặt lại timer đều ghi ngay; thêm vào đó `flushEditorNow` tự gọi lại (`editorPanelState.ts:845`). Chưa đo được cơn ghi này có làm webview chậm hay không.
- Chẩn đoán lượt hai (release `99f3fc1` + `eprintln!` tạm đã gỡ, 40 phím/giây, n=5, 6,35–8,80 s): Rust xử lý lô trong ≤ 1,1 ms, chờ khoá ≤ 0,1 ms, lô tới dày ~4/giây; chỉ số `[n]` tới Rust trễ 1,3 → 5,5 s so với mốc bơm. Cửa sổ là phím xếp hàng ở webview, không phải đường flush; chưa có bản sửa nên Task 3 chưa làm và plan `blocked` chờ Ice chọn hướng (profile `input` ở lưới, gần 2-15). Chi tiết: `nfr18-flush-ban-do/HANDOFF.md`.
- Lượt ba (A/B, 40 phím/giây, n=6 mỗi bản, loadavg 4,7–12,8): chặn cơn flush cắt số lượt flush từ 9–216 xuống 6–12 nhưng `window_s` không giảm (trung vị 6,8 -> 8,7 s; 5/6 mẫu mỗi bản > 5 s). Đường flush không phải đòn bẩy; bản đo đã gỡ. Bảng ở `nfr18-flush-ban-do/HANDOFF.md` §Pha A/B. Task 4 chờ Ice chọn hướng (chi phí `input` ở lưới).

## Plan Change Log

## Review Triage Log

## Design Notes

Bằng chứng cho hướng đo trước: ở lượt kill tháng 8 (`kill2-g4mib.tsv`), `max_n` 21 so với `last_n` 32, tức khoảng 11 câu chưa tới kho. Cùng lúc đó 2.4 ghi app tụt khoảng 15 s sau 30 s gõ (bộ đếm phím 3 → 13). Mã scheduler không đổi hành vi kể từ `6a4e6b8`. Vì vậy chỉ một lượt đo mới phân biệt được "phím xếp hàng ở lưới cũ" với "flush chậm".

Điều kiện chạy: lượt GUI cần `osascript`/`cliclick` và một app thật, nên chạy ngoài sandbox. Ice đã rời máy cho lượt này. Nếu sau khi đo trên HEAD mà vẫn trượt, ghi bảng số và nghi can trúng số vào `nfr18-flush-ban-do/HANDOFF.md`, rồi dừng và báo lại. Phần sửa giao cho một agent mới, làm việc từ plan và tệp bàn giao đó.

## Verification

**Commands:**
- `npx vitest run tests/frontend/editorFlush.test.ts` -- xanh nếu có chạm webview.
- `cargo test --test flush_cadence_contract` (sau `npm run build`) -- xanh nếu có chạm Rust.

**Manual checks (if no CLI):**
- Bảng `grid-table.sh` của lượt HEAD, kèm loadavg và hàng rào trước/sau.
