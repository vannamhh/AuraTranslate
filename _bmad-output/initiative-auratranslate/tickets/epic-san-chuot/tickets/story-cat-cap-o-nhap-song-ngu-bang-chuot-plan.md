---
title: 'Cắt cặp ở nhập song ngữ bằng chuột'
type: 'feature'
ticket: '7'
created: '2026-10-31'
status: 'built'
baseline_revision: 'e9742cc0ff8d0e758e86ee7eceaba6107b87d55e'
route: 'full'
route_source: 'auto'
risk: 'medium'
review: 'quick'
review_source: 'pinned'
lenses_ran: ['quick']
review_loop_iteration: 0
context:
  - '{project-root}/_bmad-output/initiative-auratranslate/tickets/epic-san-chuot/ux-san-chuot/EXPERIENCE.md'
  - '{project-root}/_bmad-output/initiative-auratranslate/tickets/epic-san-chuot/ux-san-chuot/DESIGN.md'
---

<frozen-after-approval reason="human-owned intent — do not modify unless human renegotiates">

## Intent

**Problem:** 5 id của bước cắt cặp song ngữ chỉ có phím: điểm cắt là một chuỗi có chú thích nên chuột không trỏ tới được, và chuyển cặp lệch không có nút.

**Approach:** Vẽ mỗi `candidate_positions` thành một đích bấm, một cú bấm là cắt hoặc bỏ cắt; thêm hai nút cặp lệch ở dòng tiêu đề; `caret_left`/`caret_right` khai `pointer: aim`, 3 id còn lại thành điểm chuột, rút 5 id khỏi `POINTER_DEBT`.

## Boundaries & Constraints

**Decision (repo chứng minh):** đích bấm là `<button type="button" tabindex="-1">`: một dòng có hàng chục đích nên không thành điểm dừng `Tab`, đường bàn phím vẫn là `← →` `Enter`.

**Decision (repo chứng minh):** cú nhấn trên đích và trên hai nút cặp lệch mang `@mousedown.left.prevent`, rồi đặt tiêu điểm vào `panel` bằng `focus({ preventScroll: true })`. `.bip-mismatch-focus` không dùng được vì `v-if` huỷ nó khi hàng cuối hết lệch và tiêu điểm rơi về `body` làm chết cả `Esc`; `.prevent` đơn thuần để tiêu điểm kẹt ở ô nhập mẫu Chương hoặc nút vừa bấm và `onScrimKeydown` bỏ qua phím.

**Decision (repo chứng minh):** `Cặp lệch trước`/`Cặp lệch sau` là giá trị mới của hai khoá `command.*` sẵn có, cùng một chuỗi cho nhãn nút, `title` và màn hình phím tắt; không test nào ghim chuỗi cũ. Không đổi `Chỗ lệch` ở `bilingual_mismatch_heading`.

**Decision (Ice chọn 2026-10-31):** hàng vừa khớp bằng cú bấm chuột ở lại trên màn: `‖`, cột mảnh và tiêu đề "đã khớp", bấm lại đưa hàng về lệch. `resolve` trả `Paired` ngay khi đủ số mảnh nên không có việc này thì `bấm lại` không tồn tại ở ca 2 câu / 1 câu. Webview giữ ảnh chụp hàng đó tới khi người dùng rời đi; Rust không đổi. Chỉ đường chuột: `Enter` vẫn tự sang hàng kế theo §Boundaries của epic.

**Decision:** hàng giữ không chặn xác nhận và không tính vào số cặp lệch: quy nhóm của nó đã hợp lệ, Rust vẫn là nơi phán quyết. `Cặp lệch sau`/`trước` rời hàng giữ, tới hàng lệch gần nhất theo `row_number`.

**Always:** `@click` là đúng một `dispatch` literal. Hàm nhắm mới kiểm `candidate_positions` (AD-1, webview không tính luật). Mọi điểm dừng caret giữ chỗ cố định: caret đổi không làm chữ chảy lại, đích dưới con trỏ không dời giữa `mousedown` và `mouseup`, và nút giữ nguyên node qua mỗi lần render (`:key` theo vị trí). Đích và hai nút tắt khi `bilingualImportPreviewConfirming`: một `refresh` chen vào lúc xác nhận làm mất kết quả xác nhận qua `sequence`. Dáng theo `DESIGN.md` §Components, kích thước theo §Layout & Spacing. Nút `title` qua `commandTitle`; đích có `aria-label`, `title`, `aria-pressed`.

**Never:** Đổi `onMismatchKeydown`, ý nghĩa phím hay thứ tự caret; đổi Rust hoặc IPC; lọc `candidate_positions` ở webview. Sửa `POINTER_DEBT_BASELINE`. Ghi `tickets.toml`. Chép `focusOnPointerDown`. Vẽ đích ở hai đầu dòng.

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected Output / Behavior | Error Handling |
|----------|--------------|---------------------------|----------------|
| Nhắm | Cú nhấn trái trên đích vị trí p | Caret = p, `←` `→` đi tiếp từ p | Không |
| Chuột phải | Nút khác nút trái trên đích | Caret không đổi, 0 IPC | Không |
| Cắt, hàng vẫn lệch | Nguồn 3 câu, đích 1 câu, bấm một `¦` | `¦` thành `‖`, cột mảnh đổi, bấm lại thì bỏ cắt | Không |
| Cắt, hàng vừa khớp | Nguồn 2 câu, đích 1 câu, bấm một `¦` | Hàng ở lại: `‖`, hai mảnh, tiêu đề đã khớp; số cặp lệch giảm 1; nhập được nếu hết lệch | Không |
| Bỏ cắt hàng đang giữ | Bấm lại `‖` đó | Hàng về lệch, vẫn là hàng đang nhắm, caret giữ nguyên, số cặp lệch tăng lại | Không |
| Rời hàng giữ | Bấm `Cặp lệch sau` hoặc `↓` | Tới hàng lệch kế theo `row_number`, hàng giữ không quay lại | Không |
| Giữ khi hết lệch | Hàng cuối vừa khớp | Khối vẫn hiện, `Esc` và `← →` chạy, nút Nhập mở, không có hai nút cặp lệch | Không |
| Đổi nguồn khi đang giữ | Đổi cột, tiêu đề, mẫu Chương, bảng mã | Hết giữ, hiện theo preview mới | Không |
| Phím không đổi | `Enter` làm hàng khớp | Hàng biến mất, tự sang hàng kế, như cũ | Không |
| Bấm giữa hai lượt rebuild | Bấm đích khác khi `refresh` đang chạy | Lượt mới thắng theo `sequence`, không mất hàng | Không |
| Đang xác nhận | `bilingualImportPreviewConfirming` | Đích và hai nút tắt, 0 IPC | Không |
| Một cặp lệch | Không có hàng khác để tới, kể cả tính hàng giữ | Không có hai nút | Không |
| Chuyển cặp | Bấm `Cặp lệch sau` ở hàng cuối | Vòng về hàng đầu, caret 0, `←` `→` chạy tiếp | Không |
| Hàng không cắt được | Một phía trống | Không có dòng đích, không có đích | Không |
| Tiêu điểm ở ô nhập | Đang gõ ô mẫu Chương rồi bấm đích | Tiêu điểm về `panel`, `←` `→` chạy | Không |

</frozen-after-approval>

## Code Map

- `src/BilingualImportPreviewOverlay.vue` -- `annotatedTargetLine()`:84 thành danh sách mảnh; `.bip-caret-line`:679; khối `.bip-mismatch-focus`:634, `v-if` của nó:635 phải nhận cả hàng đang giữ; tiêu đề `<p role="status">`:640 (nút đặt ngoài `role="status"`, như 14-6); `onMismatchKeydown`:220 không đổi; `panel`:314; CSS `.bip-mismatch-heading`:1105, `.bip-caret-line`:1149.
- `src/bilingualImportPreviewState.ts` -- thêm hàm nhắm cạnh `moveBilingualCaretRight`:536; `caretStops`:248; `toggleBilingualCutAtCaret`:547; `refresh`:279 đã giữ caret khi hàng còn; `activeMismatchRow`:84 đặt ở hàm nhắm để hàng vẫn được nhắm khi rơi về mặc định `list[0]`.
- ⚠️ `bilingualImportPreviewActiveMismatch`:196 chọn trong `mismatches`, nên hàng đang giữ cho `ActiveMismatchIndex`:203 ra `-1`. `moveToNextBilingualMismatch`:505 và `moveToPreviousBilingualMismatch`:516 hiểu `-1` là `from = 0`, sẽ bỏ sót hàng đầu: khi đang giữ phải đi theo `row_number`. `cutsForRow`:211 đọc được hàng giữ vì quy nhóm còn trong `regroupings`.
- ⚠️ `refresh` chạy từ mọi đường (đổi cột, tiêu đề, mẫu Chương, bảng mã), và `acceptAllBilingualProposals`:562, `skipActiveBilingualRow`:577 cùng dùng nó. Hàng giữ chỉ được đặt từ lượt cắt bằng chuột, mọi lượt khác xoá; `reset`, `open` cũng xoá.
- ⚠️ `word_boundaries` (`src-tauri/src/core/segment/bilingual.rs`:282) lấy cả hai phía dấu cách, nên mỗi dấu cách có hai đích; fixture `candidate_positions: [3]` trong test không phản ánh điều này. Dựng fixture đo bằng vị trí thật.
- ⚠️ `cutsForRow` có thể trả chỗ cắt ngoài `candidate_positions` khi quy nhóm cũ; dòng đích vẫn phải hiện `‖` ở đó, không bấm được.
- `src/commands/index.ts` -- khai `pointer: aim`, `at: 'src/BilingualImportPreviewOverlay.vue'` ở :1814 và :1825; khuôn `block_next`:1885.
- `src/ImportPreviewOverlay.vue` -- khuôn: `@mousedown.prevent`:1246, `commandTitle`:1244, `panel.focus` :463. `ui-btn` là Nút Phụ.
- `src/i18n/vi.json` -- `command.*`:791, `bilingual_caret_line_label`:1072 (thêm `¦`), khoá `aria-label` mới cho hai trạng thái đích.
- `scripts/check-commands.mjs` -- `POINTER_DEBT`:2905 chỉ xoá; `HANDLER_TABLE`:2465 xếp handler mới; sàn theo cổng in ra. ⚠️ Kiểm M `aim` chỉ đòi tệp có `@mousedown`, ca Vitest là chốt thật.
- `tests/frontend/importPreviewBilingual.test.ts` -- `freshOverlay`:109, ca phím:645 là khuôn mount; `tests/frontend/importPreviewBlocks.test.ts` là khuôn ca (i).
- `e2e/specs/story-12-6-import-preview.e2e.mjs`:176 -- đường nhập CSV thật; `e2e/specs/import-preview-mouse.e2e.mjs`, `e2e/support/pointer.mjs`.
- `_bmad-output/implementation-artifacts/14-6-ban-do/chuot-that.e2e.mjs` -- khuôn bàn đo, giữ nút 90 ms, chốt frontmost.

## Tasks & Acceptance

**Execution:**
- [x] `src/bilingualImportPreviewState.ts`, `src/commands/index.ts` -- hàm nhắm caret, giữ hàng vừa khớp và bốn đường rời giữ, hai khai `pointer`.
- [x] `src/BilingualImportPreviewOverlay.vue`, `src/i18n/vi.json` -- dòng đích thành mảnh chữ xen đích bấm, hai nút cặp lệch, handler tiêu điểm, tiêu đề hàng đã khớp, CSS, chuỗi.
- [x] `scripts/check-commands.mjs` -- gỡ 5 id, xếp handler mới, sàn theo cổng.
- [x] `tests/frontend/importPreviewBilingual.test.ts` -- ca (i) cho ba id có nút qua bộ command thật; ca `mousedown` thật cho hai id `caret`; mọi hàng I/O Matrix kiểm được bằng happy-dom.
- [x] `e2e/specs/bilingual-cut-mouse.e2e.mjs` (mới) -- CSV thật, bấm `Cặp lệch sau`, bấm điểm cắt, xác nhận, đọc segment Tác phẩm đã nhập.
- [x] `_bmad-output/implementation-artifacts/14-7-ban-do/` -- bàn đo `cliclick`: bấm điểm cắt, bấm lại, `← →` sau cú bấm, tâm cách tâm trên dòng tiếng Việt dài.
- [x] `deferred-work.md` -- đóng mục 5 id.
- [x] `EXPERIENCE.md` -- dòng 🔵 ở §Bấm điểm cắt song ngữ (tiêu điểm về `panel`, hàng vừa khớp ở lại) và KF-6.

**Acceptance Criteria:**
- Given cây sau story, when chạy `npm run check:commands`, then Kiểm M xanh với 286 điểm chuột, 33 khai `pointer`, 11 nợ.
- Given app thật, when làm đúng phần Verify của vé, then cặp tách đúng chỗ và segment đã nhập khớp.
- Given ca (i) của một id, when gỡ `@click` khỏi nút đó, then chính ca đó đỏ.
- Given ca `mousedown`, when gỡ handler nhắm khỏi đích, then ca đỏ vì caret không đổi.
- Given khoảng cách tâm đo được dưới 24px, when áp `DESIGN.md` §Layout & Spacing, then đích rộng 21px.
- Given hàng đang giữ, when bấm xác nhận, then segment đã nhập theo đúng mảnh trên màn.

## Implementation Notes

- Hàng giữ là ảnh chụp `BilingualMismatchWire` ở `bilingualImportPreviewState.ts` (`heldRow`), chỉ đặt khi `toggleBilingualCutAtCaret` chạy sau `aimBilingualCaret` (cờ `holdArmed`, xoá bởi mọi phím qua `@keydown.capture` trên `.bip-screen` và khi chuyển hàng); mọi `refresh` khác xoá nó.
- `bilingualImportPreviewActiveMismatch` trả hàng giữ khi `activeMismatchRow` trỏ vào nó; `bilingualImportPreviewCanMoveBetweenMismatches` tính cả hàng giữ. Chuyển hàng khi đang giữ đi theo `row_number`.
- Dòng đích dựng bằng `v-text` (không text node, tránh khoảng trắng thừa dưới `pre-wrap`). Điểm dừng hai đầu là ô 7px không bấm được, giữ chỗ cho `▏`.
- Đích 21px: tâm cách tâm đo 23,6px trên bàn đo (`14-7-ban-do/ket-qua.md`).
- `controls-adoption.test.ts` nhận `.bip-cut-target` vào `NOT_A_CONTROL` (cùng tiền lệ liên kết trong câu); `POINTER_FLOOR` lên 244.
- Fixture cắt hai chỗ liền kề quanh một dấu cách cho mảnh rỗng, Rust từ chối; e2e cắt ở đích 0 và 2.
- Hàng giữ chỉ đặt khi `activeMismatchRow` còn trỏ vào nó: rời sang hàng khác lúc rebuild đang chạy thì kết quả về không kéo người dùng quay lại.
- `Áp mọi đề xuất` tắt khi đang giữ mà hết hàng lệch; trước story khối không hiện ở trạng thái đó nên nút không thể im lặng (R9).
- Cú nhấn mất `mouseup`: lượt 1 mất 1/4, hai lượt sau 0 mất trên 30 cú; không tái hiện, nguyên nhân chưa xác định, đã giao lượt dùng thật Epic 14 (`14-7-ban-do/ket-qua.md`).
- Ảnh bàn đo lộ hàng chọn cột tràn sang cột bảng mã; CSS có sẵn, đã ghi nợ `Chủ: Ice`, cùng mục tiêu điểm rơi về `body` của đường `Enter`.

## Plan Change Log

## Review Triage Log

Lượt 1 (lens Quick): high 0 · medium 5 · low 7 · false 0 · maybe-false 0.

- medium, patch: `holdArmed` còn sau cú nhấn không thành `click` nên `Enter` kế tiếp giữ hàng, trái "Phím không đổi"; thay `@mouseleave` bằng `@keydown.capture` trên `.bip-screen`.
- medium, patch: nhấn, rê ra rồi về vẫn có `click` nhưng `mouseleave` đã gỡ nên hàng biến mất; cùng phép vá trên.
- medium, patch: `LONG_TARGET` có dấu phẩy không bọc nháy, ô đích dừng ở 14 từ, `ket-qua.md` khai 55 từ; bỏ dấu phẩy, đo lại, sửa quần thể.
- medium, patch: cú nhấn mất `mouseup` chưa rõ nguyên nhân mà chưa có mục sổ nợ, khác mục bấm lặp một điểm vì con trỏ đã rê; thêm dòng 🔵 vào mục `Chủ: Epic 14`.
- medium, patch: e2e `/Ba.*bon.*nam/` và `toHaveLength(5)` không bắt được tách sai; đổi sang `toEqual` trên cặp nguồn, đích.
- low, patch: `selectBilingualEncoding` rời hàng giữ mà để caret cũ, Matrix nêu `bảng mã` nhưng không ca nào phủ; đặt caret 0, thêm ca.
- low, patch: ca "hàng không cắt được" không thể đỏ vì fixture 0 điểm ứng viên; thêm điểm và khẳng định không có dòng đích.
- low, patch: thiếu khẳng định cho "Chuyển cặp", 0 IPC của "Đang xác nhận" và giữ nguyên node; thêm, đổi tên `keysBefore`.
- low, patch: `aura-allow-spacing` trên `border-bottom` không miễn trừ gì; xoá.
- low, patch: `aura-allow-text` trước `.bip-caret-line` không còn nội suy; xoá.
- low, patch: doc-comment của `ActiveMismatch` và `ActiveMismatchIndex` sai với hàng giữ; thay bằng tiếng Anh.
- low, patch: comment bàn đo mang story id và lịch sử; rút còn quy tắc.

## Design Notes

Hai đích quanh một dấu cách cho cùng kết quả vì mảnh được `trim`, nhưng lọc là luật nghiệp vụ và đổi ở Rust sẽ đổi điểm dừng của `← →`, nên giữ nguyên theo vé. Với 14px, tâm cách tâm quanh một dấu cách dự kiến dưới 24px nên nhiều khả năng lên 21px; dòng dài sẽ xuống hàng thêm.

## Verification

**Commands:**
- `npm run check:commands`, `npm run check:i18n`, `npm run check:tokens`, `npm run check:debt-owner` -- exit 0.
- `npm run test:story 14-7` -- exit 0.
- `npx vue-tsc --noEmit`, `npm run check:lint` -- exit 0.
- `npm run build`, rồi `cargo build --locked --features wdio --manifest-path src-tauri/Cargo.toml`, rồi `npx wdio run e2e/wdio.conf.mjs --spec e2e/specs/bilingual-cut-mouse.e2e.mjs` -- xanh, chạy ngoài sandbox.
- Chạy đủ bộ một lần: story chạm `scripts/check-*` (AGENTS.md §Tests).

**Manual checks (if no CLI):**
- Lượt dùng thật của Ice trước khi Epic 14 `done`: tới cặp lệch, cắt, bỏ cắt, nhập, đọc kết quả.
