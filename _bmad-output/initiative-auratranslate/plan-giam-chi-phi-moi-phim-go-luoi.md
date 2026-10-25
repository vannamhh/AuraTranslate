---
title: 'Giảm chi phí mỗi phím gõ trong lưới để NFR18 không trượt'
type: 'bugfix'
ticket: ''
created: '2026-10-24'
status: 'blocked'
baseline_revision: 'e33d55536f573c5940c5b7fe49c4fd85659f3ca4'
route: 'full'
route_source: 'auto'
risk: 'high'
review: ''
review_source: ''
lenses_ran: []
review_loop_iteration: 0
context:
  - '{project-root}/_bmad-output/initiative-auratranslate/research-grid-text-input-path/research-grid-text-input-path.md'
  - '{project-root}/_bmad-output/implementation-artifacts/nfr18-flush-ban-do/HANDOFF.md'
---

<frozen-after-approval reason="human-owned intent — do not modify unless human renegotiates">

## Intent

**Problem:** Ở HEAD, Chương 122 câu, cửa sổ mất dữ liệu sau `SIGKILL` là 7,0–19,4 s ở 40 phím/giây (20/20 mẫu trượt) và 0,3–7,8 s ở 10 phím/giây (7/20 trượt), so với trần NFR18 là 5 s. Rust ghi ≤ 1,1 ms và chặn cơn flush không rút cửa sổ (plan `plan-nfr18-duong-flush`, `blocked`). Chữ tới `noteEditorEdit` trễ dần trong phiên (1,3 s lên 5,5 s), tức luồng chính webview xử lý phím chậm hơn nhịp gõ.

**Approach:** Đo trước. Một bàn đo trong trang tách chi phí mỗi phím thành từng chặng (handler JS, lệnh sửa của WebKit giữa `beforeinput` và `input`, `selectionchange`, phần còn lại tới frame). Ice chọn hướng sửa theo số. Sau đó sửa chặng trội, khoá bằng một test đỏ khi gỡ bản sửa, rồi đo lại NFR18.

**Quyết định của Ice (2026-10-24):**
- Giữ một plan gộp: đo phân bổ, chọn hướng, sửa, đo lại. Ice chấp nhận plan dài khoảng 2.850 token.
- NFR18 phải đạt ở cả hai nhịp, 40 phím/giây (`type-driver.sh`) và 10 phím/giây (`RATE=10hz`), dù D5 thấy 40 phím/giây nhanh khoảng năm lần người gõ nhanh.
- Agent chạy lượt GUI trong phiên, Ice rời máy. Mỗi bước có trần thời gian và sentinel phát từ `trap … EXIT`; agent báo trước thời gian máy bị chiếm.
- Tải máy hiện có (Docker/VM chạy nền) là điều kiện nghiệm thu: không loại hay chạy lại phiên vì tải, không đòi máy rảnh.
- Sau lượt phân bổ (`HANDOFF.md` §Pha phân bổ chi phí mỗi phím): sửa bằng hướng A, `contain: layout style` trên `.cell-tgt`, rồi chạy kill campaign 20 mẫu ở cả hai nhịp. Ở 10 phím/giây phím không xếp hàng; nếu nhịp này còn trượt thì thêm mốc tách cửa sổ (chờ hẹn giờ flush, lô bay, commit) và tìm nguyên nhân trong cùng plan.

## Boundaries & Constraints

**Always:** Giữ nhịp flush AD-35 (`EDITOR_IDLE_MS` 2000, `EDITOR_HARD_CAP_MS` 5000). Giữ chốt `event.isComposing` ở đầu `onEditKeydown` và lý do không `normalize()` từ `beforeinput` (`proofreadHighlight.ts:67-71`). Mỗi số đo ghi bản build, cỡ Chương, nhịp, loadavg và n. Đo trên release dựng bằng `build-bench.sh`, màn hình mở khoá, hàng rào `fence.sh` trước và sau mỗi lượt.

**Never:** Không đổi bất biến của spine. Nếu cần thì dừng và chuyển thành AD cho Winston. Không đổi `editorFlush`/`writeSchedule` (đã loại). Không đánh dấu NFR18 đạt bằng suy luận hay vitest. Không dựa vào Event Timing API: engine hệ thống 20621 của app không có nó.

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected Output / Behavior | Error Handling |
|----------|--------------|---------------------------|----------------|
| Gõ liên tục rồi sập | gõ 20–55 s, `SIGKILL` ngay khi ngừng | `[n]` lớn nhất trong `project.db` được bơm ≤ 5 s trước lúc kill | mẫu > 5 s ghi vào bảng, không bỏ |
| Có gạch chân soát lỗi | ô gõ đang mang span soát lỗi | gõ vẫn gỡ span như hiện nay, caret không nhảy | ca vitest giữ hành vi |
| Bộ gõ tiếng Việt | composition đang mở | không chặng nào ăn `keydown` hay ghi DOM lúc composing | ca IME trong `editorTypingZone.test.ts` xanh |

</frozen-after-approval>

## Code Map

- `src/commands/keys.ts:510-545` (`handle`), `:601-607` (`attachKeymap`) -- keydown capture trên `window`, duyệt mọi chord mỗi phím. `src/main.ts:1118-1152` là cổng `isBlocked`, OR khoảng 20 ref.
- `src/panels/GridPanel.vue:1608-1620` (`onEditKeydown`) -- mỗi keydown không composing gọi `unwrapProofread` và `scheduleRepaintProofread` (`setTimeout 0`). Không có span thì `unwrapProofread` thoát sau một `querySelectorAll` (`proofreadHighlight.ts:71-78`).
- `GridPanel.vue:1435-1440` (`onBeforeInput`) -- `unwrapProofread` lần hai. D5: handler trước lệnh sửa mà ghi DOM thì lượt layout đồng bộ đầu tiên của WebKit đắt hơn.
- `GridPanel.vue:1699-1715` (`reportEdit`, `onEditInput`) → `editorPanelState.ts:758-814` (`noteEditorEdit`): ghi `editedText` tại chỗ, `triggerRef(editedShape)` chỉ khi hình dạng đổi, `datThongBao({})`, đặt lại hẹn giờ flush.
- `GridPanel.vue:1234-1240`, `:1289-1323`, `:1325-1332` -- hai listener `selectionchange` trên `document`. `setEditorCaret` (`editorPanelState.ts:281-296`) đặt lại hẹn giờ lưu vị trí mỗi phím (`positionFlush.ts:35-42`).
- `src/panels/GridBlock.vue:308-346` -- ô đích `contenteditable`, `v-memo` theo cột. Ở 122 câu chỉ có một khối.
- `tests/frontend/gridTypingCost.test.ts` -- ca đếm việc mỗi phím (`segmentRuleInputOf`, `PanelFrame` render). Thêm ca khoá bản sửa ở đây.
- `_bmad-output/implementation-artifacts/nfr18-flush-ban-do/` -- bàn đo kill v2 (`kill-campaign-v2.sh`, `grid-table.sh`, `type-driver.sh` với `RATE=10hz`, `bench.js` nối vào bundle bằng `build-bench.sh`). `../2-15-ban-do/bench-215.js` ghi `event.timeStamp` tới frame đầu có chữ, dùng lại được.
- Số đã có: 2-15 đo ở 122 câu, 10 phím/giây, độ trễ phím cuối tới frame là 32–52 ms, không có phím chờ. Ngân sách ở 40 phím/giây là 25 ms mỗi phím.

## Tasks & Acceptance

**Execution:**
- [x] `_bmad-output/implementation-artifacts/nfr18-flush-ban-do/bench.js` -- thêm mốc mỗi phím bằng `performance.now()`: keydown capture vào/ra, `beforeinput`, `input` vào/ra, `selectionchange`, rAF kế tiếp, và độ trễ `now − event.timeStamp`. Đổ số qua `put_config`. -- tách chặng mà không sửa mã sản phẩm.
- [x] `_bmad-output/implementation-artifacts/nfr18-flush-ban-do/HANDOFF.md` -- lượt phân bổ trên release HEAD, 122 câu, hai nhịp, n ≥ 3 phiên: bảng p50/p95 mỗi chặng và tổng thời gian luồng chính mỗi phím. Mỗi chặng nghi can được gỡ thử bằng một bản tạm (gỡ thật rồi hoàn lại), đo xen kẽ. -- số cho lựa chọn.
- [x] CHECKPOINT -- trình Ice các hướng sửa kèm số đo; dừng chờ chọn.
- [x] mã sản phẩm theo hướng Ice chọn, trong các tệp ở Code Map -- sửa chặng trội.
- [x] `tests/frontend/gridTypingCost.test.ts` -- ca đếm việc của chặng đã sửa nếu chặng là JS; gỡ bản sửa thì đỏ đúng lý do. Nếu chặng nằm ở WebKit (CSS, cấu trúc ô) thì happy-dom không đo được, nên chọn ca e2e trong WKWebView cùng lúc với hướng sửa.
- [x] `HANDOFF.md` và plan -- `kill-campaign-v2.sh` n=20 mỗi nhịp trên release mới, bảng `grid-table.sh`; mục 🟡 của #107 trong `deferred-work.md` được đóng hoặc ghi phần còn hở.

**Acceptance Criteria:**
- Given release mới, 122 câu, cả hai nhịp, when chạy `kill-campaign-v2.sh` 20 mẫu `VALID`, then 20/20 mẫu ≤ 5 s; nếu không đạt thì plan sang `blocked` kèm bảng.
- Given cùng bàn đo phân bổ, when so trước và sau bản sửa, then độ trễ `[n]` tới `noteEditorEdit` không tăng dần trong phiên ở cả hai nhịp.
- Given vitest, when gỡ bản sửa, then ca mới đỏ vì đếm việc, không vì lỗi khác.

## Implementation Notes

- Phân bổ xong (HANDOFF §Pha phân bổ): JS sản phẩm ≤ 1 ms p95 mỗi phím; chặng trội ở WebKit (lệnh sửa 14 ms, đuôi layout/vẽ p95 ~80 ms). Ba biến thể CSS/thuộc tính cắt đuôi còn ~9 ms nhưng hàng đợi vẫn tăng; dừng ở CHECKPOINT chờ Ice chọn hướng.
- Hướng A (`contain: layout style` trên `.cell-tgt`, `GridBlock.vue`) đã áp; kill n=20: 40 phím/giây 19/20 > 5 s (max 20,7), 10 phím/giây 4/20 > 5 s (max 5,99). AC1 trượt ⇒ `blocked`. Khoá bằng ca e2e `grid-blocks` (computed `contain` + hàng thẳng); vitest không đo được CSS WebKit. Bảng: HANDOFF §Kết quả hướng A.

## Plan Change Log

## Review Triage Log

## Design Notes

Bàn đo trong trang chỉ ghi mốc, không bọc hàm sản phẩm. Khoảng `beforeinput` → `input` là phần WebKit tự làm (lệnh sửa và các lượt layout đồng bộ, D5 [22][23]). Nếu chặng này trội thì các hướng sửa nằm ở CSS và cấu trúc DOM của ô, không ở handler. Khi đó trình Ice từng phương án kèm số, như AGENTS.md yêu cầu.

## Verification

**Commands:**
- `npx vitest run tests/frontend/gridTypingCost.test.ts tests/frontend/editorTypingZone.test.ts` -- expected: xanh, và ca mới đỏ khi gỡ bản sửa.

**Manual checks (if no CLI):**
- Bảng `grid-table.sh` của lượt cuối ghi build, loadavg, n và số mẫu trượt.
