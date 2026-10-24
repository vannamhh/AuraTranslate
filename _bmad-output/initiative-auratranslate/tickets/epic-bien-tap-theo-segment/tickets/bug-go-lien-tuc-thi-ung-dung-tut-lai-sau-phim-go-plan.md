---
title: 'Gõ liên tục thì ứng dụng tụt lại sau phím gõ'
type: 'bugfix'
ticket: 15
created: '2026-10-23'
status: built
baseline_revision: '42f94abaa756c393d29ed141b5e8a7c176dca299'
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

**Problem:** Mỗi sự kiện `input` thay cả `Map` chữ đang sửa. Thân lưới đọc `Map` này ở từng hàng, mà thân lưới lại là slot của `PanelFrame`, nên mỗi phím chạy lại toàn bộ slot: 4 cột × n hàng, cộng `restRuleById` trên n câu. Ở 9.850 câu, việc này tốn hơn khoảng 100 ms giữa hai phím, nên phím xếp hàng.

**Approach:** Đo trước, ở nhịp 10 phím/giây, và tách việc theo sự kiện khỏi việc theo frame. Rồi để mỗi phím không còn kích hoạt lại lưới: lưới chỉ được báo khi tính rỗng của câu đổi, vì đó là thứ duy nhất lưới hiển thị theo chữ đang sửa (`resolveSegmentRule` chỉ xét `targetText !== ''`, cùng lớp `empty`). Mọi chỗ cần chữ chính xác vẫn đọc bản hiện hành. Đo lại trên bản dựng thật.

## Boundaries & Constraints

**Always:**
- DOM giữ chữ đang gõ: template chỉ dựng `s.target_text`, ô đọc `textContent`.
- Mỗi lượt flush, điều hướng (`danhSachDieuHuong`) và `restoreEditedText` đọc đúng chữ mới nhất.
- Nhịp flush 2.000/5.000 ms (Story 2.3, FR100) không đổi.
- Lượt gõ không đổi mốc so xuất xứ (AD-47).
- Khoá gõ lúc chuyển Chương giữ nguyên.
- `.margin-slot` luôn có mặt và viền dày cố định 3 px (`43729b3`).
- Mỗi số đo ghi bản dựng, phiên bản WebKit, số câu, cách gửi phím và tải máy.

**Never:**
- Không đẩy việc cập nhật vạch lề hay lớp `empty` ra sau frame (idle hay `setTimeout`).
- Không ảo hoá hàng.
- Không nới ngưỡng hay sửa điều kiện spec e2e cho xanh.
- Không thêm bàn đo vào `package.json`.
- Còn > 100 ms trên bản dựng thật thì dừng `blocked`, trình Ice số, không đổi cấu trúc lưới.

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected Output / Behavior | Error Handling |
|----------|--------------|---------------------------|----------------|
| Gõ giữa câu | câu đã có chữ, gõ thêm | slot lưới không render lại, không gọi `segmentRuleInputOf` | — |
| Gõ chữ đầu | câu rỗng, gõ 1 ký tự | lớp `empty` tắt ngay lượt vẽ đó; rời con trỏ thì vạch `draft` | — |
| Xoá hết chữ | xoá ký tự cuối | lớp `empty` bật; rời con trỏ thì không vạch | — |
| Điều hướng | gõ câu A rồi `Mod+Alt+ArrowDown` | A tính là đã dịch | — |
| Flush | gõ rồi chờ 2 s | lượt lưu mang chữ cuối cùng | — |
| Chuyển Chương | gõ dở rồi chuyển Chương | chữ được ghi, `Map` làm mới như trước | — |

</frozen-after-approval>

## Code Map

- `src/panels/editorPanelState.ts:683-685` -- `editedText = ref(new Map)` (reactive sâu), xuất `readonly`; `:755-805` `noteEditorEdit`: `:783-785` thay cả `Map` mỗi phím; `:789` `armFlushTimer`; `:992` làm mới `Map` khi chuyển Chương; `:1735` `danhSachDieuHuong` đọc lúc ra lệnh. Không `invoke` trên mỗi phím.
- `src/panels/GridPanel.vue:279-298` -- `restRuleById` lặp mọi câu, đọc chữ đang sửa; `ruleOf`/`ruleClassForRow`. `:1604-1620` `reportEdit`/`onEditInput`. `v-memo` cột đích `:2013-2021` đọc `editorEditedText.get(s.id)` và `ruleOf`; `:2030` lớp `empty`; `:2046` `{{ s.target_text }}`; `:1050-1088` `restoreEditedText`.
- `src/panels/PanelFrame.vue:173-175` -- `<slot />`: mọi đọc phản ứng trong thân lưới bị effect render của `PanelFrame` theo dõi.
- `src/panels/editorSegments.ts:162-175` -- `resolveSegmentRule`: chỉ xét tính rỗng của chữ.
- `tests/frontend/gridCaretMoveCost.test.ts` -- khuôn đếm: `vi.mock` bọc `segmentRuleInputOf` (13-22), `many(n)`, `PanelFrame` stub `<div><slot/></div>` (67), `it.each([20, 200])`, gõ bằng `textContent` + `Event('input')` (138-139). `tests/frontend/support/segmentFixture.ts` -- `recordSave`/`saveCalls`.
- `_bmad-output/implementation-artifacts/2-4-ban-do/` -- `build-bench.sh` (release, nối `bench.js` vào `dist/assets/index-*.js` vì CSP, xác nhận bằng mốc `__bench_alive__`); `bench.js:193` ghi giờ mỗi `input`; `type-driver.sh` gõ cả cụm bằng `osascript`, không có nhịp cố định; `chapters/ladder-m.txt` 122 câu.
- `_bmad-output/implementation-artifacts/2-14-ban-do/do-doi-con-tro.e2e.mjs:173-190` -- đo một phím rời (`InputEvent` tổng hợp, hai rAF), bản debug, 9.850 câu.

## Tasks & Acceptance

**Execution:**
- [ ] `_bmad-output/implementation-artifacts/2-15-ban-do/` -- bộ gửi phím 10 phím/giây trong MỘT tiến trình (AppleScript `repeat` + `delay 0.1`, chữ ASCII), rồi `Mod+Enter`. Script đo trong trang ghi với mọi phím `event.timeStamp` → frame đầu có chữ, ghi số phím chờ và thời gian từ `Mod+Enter` tới câu thành `confirmed`. Chạy trên bản release theo khuôn `build-bench.sh`, ở 122 và 9.850 câu, n ≥ 3. -- tiêu chí 2; đạt sẵn ⇒ tiêu chí 4, dừng.
- [ ] `tests/frontend/gridTypingCost.test.ts` -- ở 20 và 200 câu, một `input` giữa câu cho 0 lượt render slot (stub `PanelFrame` có bộ đếm render) và 0 lượt `segmentRuleInputOf`; đỏ trên `baseline_revision`. Một ca cho mỗi hàng của Matrix.
- [ ] `src/panels/editorPanelState.ts` -- `noteEditorEdit` giữ chữ hiện hành mà không kích hoạt người đọc phản ứng, trừ khi tính rỗng của câu đổi; xoá sạch ở `:992` vẫn kích hoạt.
- [ ] `src/panels/GridPanel.vue` -- người đọc trong thân lưới chỉ phụ thuộc vào tín hiệu tính rỗng.
- [ ] Đo lại bằng bàn đo của task 1, cùng phiên máy và cùng điều kiện; ghi số.
- [ ] `_bmad-output/initiative-auratranslate/deferred-work.md` -- đóng hoặc 🟡 hai mục nợ đường gõ (`Chủ: Story 2.15`), viết bằng chữ, kèm số.

**Acceptance Criteria:**
- Given bản release trên macOS ở 122 và 9.850 câu, when gõ 30 s ở 10 phím/giây rồi `Mod+Enter`, n ≥ 3, then chữ của phím cuối và câu `confirmed` đều hiện ≤ 100 ms, không còn phím chờ; không đạt ⇒ `blocked` kèm số.
- Given `gridTypingCost.test.ts`, when gỡ bản sửa ở `noteEditorEdit`, then đúng các ca đếm đỏ, ca Matrix vẫn xanh.
- Given các spec e2e của lưới (`segment-navigation`, `grid-empty-cell`, `story-12-7-workspace-grid-status-bar`), when chạy, then xanh mà không sửa điều kiện.

## Implementation Notes

- Bàn đo: `_bmad-output/implementation-artifacts/2-15-ban-do/`. Bản release (ký ad-hoc) macOS, WebKit 605.1.15, `bench-215.js` nối vào bundle; phím thật qua một `osascript` 300 ký tự ASCII, ~9,9 phím/giây, rồi `Cmd+Return`, con trỏ còn trong ô. Trước = worktree ở `baseline_revision`, sau = cây làm việc chưa commit; xen kẽ, cùng phiên máy, tải máy 4–17 (máy có tiến trình khác chạy nền).
- `event.timeStamp` của WebKit bằng giờ chạy handler, không phải giờ phím tới; nên giờ gửi nội suy giữa hai mốc `osascript` ghi (đầu và phím 300), sai số cỡ chục ms.
- 122 câu, n=3 mỗi bên: phím cuối trước 37–40 ms, sau 39–55 ms; `Mod+Enter` tới `rule-confirmed` trước 49–53 ms, sau 40–52 ms; không phím chờ. Đã đạt ≤ 100 ms từ TRƯỚC khi sửa (tiêu chí 4 ở cỡ này).
- 9.850 câu, n=3 mỗi bên: trước phím cuối hiện sau 188–193 s (một phiên rơi 10/300 phím), `Mod+Enter` 187–193 s, còn 251–260 phím chờ lúc phím cuối được gửi; sau 157–163 s và 157–163 s, còn 253–254 phím chờ. Mỗi phím ~0,5–0,65 s ở cả hai bản: bản sửa giảm ~20%, tiêu chí 1 KHÔNG đạt ⇒ `blocked`, trình Ice; không đổi cấu trúc lưới (Boundaries).
- Chưa làm: lượt debug đếm render `PanelFrame` (Ice cắt phạm vi); nguồn ~0,5 s mỗi phím còn lại chưa tách. e2e `grid-empty-cell`, `story-12-7-workspace-grid-status-bar`, `segment-navigation` xanh (1, 4, 4 ca) trên cây làm việc, ngoài sandbox, không sửa điều kiện.
- 🔵 2026-10-24 — đóng bằng story 2-16 (lưới chia khối 200 hàng, mỗi khối một component; `8260a92`), không bằng mã của vé này. Bàn đo của vé, release `8260a92`, loadavg 2,9–5,4, 3 phiên mỗi cỡ: 122 câu phím cuối 32–52 ms, `Mod+Enter` 41 ms; 9.850 câu phím cuối 56–77 ms, không phím chờ, p50 20–38 / p95 43–75 ms, `Mod+Enter` 68–97 ms (đo xen kẽ bản trước: 102–140). Tiêu chí 1 đạt; bảng đủ ở `2-16-ban-do/HANDOFF.md` §Nghiệm thu lần ba. e2e `segment-navigation` 4, `grid-empty-cell` 1, `story-12-7-workspace-grid-status-bar` 4 xanh trên `fa9250a`, bản debug, ngoài sandbox. Hai mục nợ đường gõ: phần xếp hàng ✅, mệnh đề một phím ≤ 50 ms của NFR2 🟡 `Chủ: Story 2.4`.

## Plan Change Log

- 2026-10-23 — Ice bớt phạm vi đo: bỏ lượt bản debug có `config.performance` ở task 1 và spec `grid-row-error-label` ở AC thứ ba. GIỮ: n ≥ 3 phiên mỗi cỡ, số trước và sau.

## Review Triage Log

## Design Notes

Đã loại:
- `shallowReactive` `Map` theo khoá: slot vẫn là một effect duy nhất, nên một khoá đổi vẫn dựng lại cả slot.
- Component theo hàng: lưới xếp theo cột (`subgrid`), nên phải có 4n component.
- Đưa thân lưới ra khỏi slot: chi phí O(n) chỉ chuyển sang `GridPanel`.

Bản sửa không chạm vào việc theo frame, tức bố cục `subgrid` khi chữ đổi chiều cao hàng. Nếu task 1 cho thấy bố cục một frame ở 9.850 câu đã > 100 ms, cổng của tiêu chí 1 sẽ dừng ở `blocked`.

## Verification

**Commands:**
- `npx vitest run tests/frontend/gridTypingCost.test.ts tests/frontend/gridCaretMoveCost.test.ts tests/frontend/gridRuleStable.test.ts tests/frontend/gridPanel*.test.ts tests/frontend/gridRestoreEditedText.test.ts` -- expected: xanh.
- `npm run test:e2e -- --spec e2e/specs/<spec>.e2e.mjs` cho từng spec trong AC thứ ba, ngoài sandbox, màn hình mở -- expected: xanh.
