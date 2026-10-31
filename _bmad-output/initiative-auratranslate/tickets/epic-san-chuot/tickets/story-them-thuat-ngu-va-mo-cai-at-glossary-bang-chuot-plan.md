---
title: 'Thêm thuật ngữ và mở cài đặt Glossary bằng chuột'
type: 'feature'
ticket: '5'
created: '2026-10-31'
status: 'built'
baseline_revision: 'b7b877baae58d2bc30b023d89273a661c8347efd'
route: 'full'
route_source: 'auto'
risk: 'medium'
review: 'quick'
review_source: 'pinned'
lenses_ran: ['quick']
review_loop_iteration: 0
context:
  - '{project-root}/_bmad-output/initiative-auratranslate/tickets/epic-san-chuot/ux-san-chuot/EXPERIENCE.md'
---

<frozen-after-approval reason="human-owned intent — do not modify unless human renegotiates">

## Intent

**Problem:** `glossary.add_term` và `glossary.settings.open` chỉ có phím. Mục menu câu đọc vùng bôi đen lúc chạy, và `Range` dải đã lưu là thứ WebKit vừa bôi, nên đóng dải rồi gõ sẽ đè chữ hoặc mất phím.

**Approach:** Thêm mục Thêm thuật ngữ vào nhóm 2 của menu câu và mục Ngưỡng quét Glossary dưới Quản lý Glossary, rút 2 id khỏi `POINTER_DEBT`, chốt mục nợ `shortcuts.open`.

## Boundaries & Constraints

**Decision (repo chứng minh, không cần ký):** nhóm 2 về giữa nhóm 1 và nhóm 3 theo `EXPERIENCE.md` §Component Patterns. Story 14.4 đặt mục Lịch sử sau nhóm 3 và không có dòng 🔵 nào cho việc đó.

**Decision:** mục Thêm thuật ngữ `:disabled` khi `quickAddIsOpen`, `title` nói dải đang mở. `openGlossaryQuickAdd` bỏ qua lượt mở thứ hai, nên không tắt thì một cú bấm không làm gì (R9); điền lại từ mới sẽ đổi đường phím. Ghi 🔵 ở §State Patterns, chỗ bảng nói "Không bao giờ".

**Decision:** nhấn phụ trên ô số câu xoá vùng chọn sau khi đã ghi `previousRange`, để `dismissed` vẫn trả lại được. Phép đo cho thấy Chromium giữ từ bôi ở câu khác, nên trên WebView2 ô thuật ngữ sẽ không trống.

**Decision (Ice chọn 2026-10-31):** `shortcuts.open` giữ `via: settings.open`; mục nợ đóng bằng `KHÔNG LÀM`, nêu rõ tới mục Phím tắt mất 3 cú bấm và AD-62 mục 3 nêu đích danh id này. Spine không đổi, `Công cụ ▾` 9 mục.

**Decision (Ice chọn 2026-10-31):** đóng dải đã mở từ menu câu, bằng `Esc` hay sau Lưu, trả đúng trạng thái trước cú chuột phải như đóng menu không chọn: caret, vị trí trong chữ và vùng bôi của người dùng. Chuột phải ở câu B rồi Lưu thì caret về A.

**Always:** `@click` là đúng một `dispatch` literal. Mục mang thuộc tính `data-` riêng để `onDocumentClick` chọn lý do đóng mới; không dùng `data-opens-overlay`, vì lý do đó xoá vùng chọn còn dải cần đích trả về. `title` và phím theo khuôn các mục bên cạnh.

**Never:** Đổi `Mod+Alt+G`, `Mod+Alt+T`, chữ ký `openGlossaryQuickAdd`, hoặc đường đóng dải khi mở bằng phím. Sửa `POINTER_DEBT_BASELINE`. Ghi `tickets.toml`.

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected Output / Behavior | Error Handling |
|----------|--------------|---------------------------|----------------|
| Trong vùng bôi | Bôi từ ở nguyên văn hoặc bản dịch, chuột phải trong vùng bôi, chọn mục | Ô thuật ngữ điền đúng từ, tiêu điểm ở ô, gõ tiếp vào ô | Không |
| Ô số câu | Chuột phải ô số, kể cả khi có từ bôi ở câu khác | Ô thuật ngữ trống, nhận tiêu điểm | Không |
| Không bôi trước | macOS, chuột phải lên một từ | Điền từ WebKit tự bôi (§Interaction Primitives) | Không |
| Đóng dải | `Esc` hoặc Lưu, trước đó caret ở câu A giữa chữ, nhắm câu B ở từng cột | Caret về A đúng vị trí, gõ tiếp vào đó, chữ của B nguyên vẹn, không mất phím | Không |
| Dải đang mở | Chuột phải câu khác | Mục tắt, `title` nói lý do | Không |
| Chương đổi khi dải mở | Đóng dải | Không ném, caret không trỏ câu của Chương cũ | Không |
| Ngưỡng quét | Lần trước Cài đặt ở mục khác, chọn mục trong `Công cụ ▾` | Cài đặt mở đúng mục Glossary; `Esc` về `Công cụ ▾` | Không |

</frozen-after-approval>

## Code Map

- `src/panels/GridSentenceMenu.vue` -- nét ngăn :158, :194 là biên nhóm; `onDocumentClick`:76 chọn lý do đóng; `cutElsewhereNumber`:19 là khuôn `title` khi tắt.
- `src/panels/sentenceMenuState.ts` -- `closeSentenceMenu`:112, `restoreBeforeOpen`:95 (thiếu kiểm câu còn tồn tại, dải sống qua đổi Chương).
- `src/glossaryQuickAddState.ts` -- `openGlossaryQuickAdd`:251 lưu `savedFocusEl`, `savedRange`; `restoreFocusAndSelection`:286 dùng chung cho `Esc` và Lưu.
- `src/panels/GridPanel.vue` -- `onSentencePress`:1017.
- `src/panels/selectionContract.ts` -- `surfaceFor`:185, `ownsAnchor`:74: bề mặt có `scope`, anchor ở ô số trả `''`. Không sửa.
- `src/TitlebarToolsMenu.vue` -- chèn sau mục `glossary.manage.open`:119. `src/settingsState.ts` -- `openSettingsToSection`, nối sẵn ở `src/main.ts`:971.
- `src/i18n/vi.json`:1420 -- thêm khoá menu.
- `scripts/check-commands.mjs` -- `POINTER_DEBT`:2902 chỉ xoá ở đây, `HANDLER_ATTR_FLOOR`:2876 theo công thức cổng in ra.
- `tests/frontend/gridSentenceMenu.test.ts` -- `CASES`:114, `mountGrid`:38. `tests/frontend/titlebarToolsMenu.test.ts`:36 -- đang đếm tám mục. `e2e/specs/titlebar-tools.e2e.mjs`:6 -- `ITEM_IDS`.
- `_bmad-output/implementation-artifacts/14-4-ban-do/chuot-that.e2e.mjs` -- khuôn bàn đo `cliclick`, `Esc` qua `osascript`.

## Tasks & Acceptance

**Execution:**
- [x] `src/panels/GridSentenceMenu.vue`, `src/i18n/vi.json` -- chuyển nhóm 2 về giữa, thêm mục đứng trước Lịch sử, `:disabled`, lý do đóng mới.
- [x] `src/panels/sentenceMenuState.ts`, `src/glossaryQuickAddState.ts` -- lý do đóng mới giao `restoreBeforeOpen` cho dải; dải trả `savedFocusEl` rồi chạy nó thay `savedRange`, đường phím giữ nguyên; `restoreBeforeOpen` kiểm câu còn tồn tại trước `setEditorCaret`.
- [x] `src/panels/GridPanel.vue` -- xoá vùng chọn ở nhánh ô số.
- [x] `src/TitlebarToolsMenu.vue` -- mục `glossary.settings.open`, `data-tools-item`.
- [x] `scripts/check-commands.mjs` -- gỡ 2 id.
- [x] `tests/frontend/gridSentenceMenu.test.ts`, `tests/frontend/titlebarToolsMenu.test.ts`, `tests/frontend/glossaryQuickAddStrip.test.ts` -- ca (i) cho từng id, mọi hàng I/O Matrix kiểm được bằng happy-dom, ca đường phím vẫn trả `savedRange`.
- [x] `e2e/specs/glossary-mouse.e2e.mjs` (mới), `e2e/specs/titlebar-tools.e2e.mjs` -- nhắm câu B, mục, ô thuật ngữ; `Công cụ ▾` tới mục Glossary từ mục khác.
- [x] `_bmad-output/implementation-artifacts/14-5-ban-do/` -- bàn đo `cliclick`: chuột phải trong vùng bôi và ô số, gõ vào ô thuật ngữ, `Esc` rồi gõ ở ba cột.
- [x] `deferred-work.md`, `EXPERIENCE.md` -- đóng mục 2 id, đóng mục `shortcuts.open` bằng `KHÔNG LÀM`, thêm lượt dùng thật vào `Chủ: Epic 14`, phần WebView2 vào `B7`; dòng 🔵 ở §State Patterns.

**Acceptance Criteria:**
- Given cây sau story, when chạy `npm run check:commands`, then Kiểm M xanh với 277 điểm chuột, 29 khai `pointer`, 24 nợ.
- Given app thật, when làm đúng phần Verify của ticket, then ô thuật ngữ điền sẵn từ bôi, ô số mở ô trống, `Công cụ ▾` tới đúng mục Glossary.
- Given ca (i) của một id, when gỡ `@click` khỏi mục đó, then chính ca đó đỏ.
- Given ca ô số có từ bôi ở câu khác, when gỡ dòng xoá vùng chọn, then ca đỏ vì ô thuật ngữ điền từ cũ.
- Given đang gõ trong ô, when so `gridTypingCost.test.ts` và `gridCaretMoveCost.test.ts`, then cả hai xanh không đổi ngưỡng.

## Implementation Notes

- Mục mang `data-opens-glossary-strip`; lý do đóng `glossary` giao `restoreBeforeOpen` cho dải qua `adoptQuickAddRestore`. Dải không mở được thì trả ngay, như đóng không chọn.
- `restoreOnClose` chạy sau khi trả `savedFocusEl`, thay `savedRange`; đường phím không đổi.
- Trạng thái tắt của mục đọc một lần lúc menu mở (`stripWasOpen`): bàn đo chuột thật lộ rằng `:disabled` sống làm `onDocumentClick` thoát sớm, menu không đóng và phím đầu vào ô mất. Vitest và e2e đều không thấy vì `.click()` không chạy microtask giữa hai listener. Ca khoá lỗi: dải mở trong lúc menu mở thì mục vẫn bật.
- Đo chuột thật khớp I/O Matrix ở cả ba cột: `_bmad-output/implementation-artifacts/14-5-ban-do/ket-qua.md`. Đường Lưu chỉ có ca Vitest.
- Cmd-Tab thật làm app đo nhận cú bấm đơn sau 1,3–1,4 s; rê chuột trước khi bấm. Hai điều kiện này và chốt frontmost nằm trong bàn đo.
- Cổng buộc nâng `CLICK_FLOOR` 253 lên 271 và `POINTER_FLOOR` 220 lên 236 theo công thức `ceil(0.85 × live)`; `HANDLER_ATTR_FLOOR` không đổi.
- Chạy đủ một lần: 14 cổng, `vue-tsc`, build xanh; vitest 3772/3772 với `--testTimeout=20000` ở dòng lệnh vì Blender và máy ảo đẩy load lên 37–90 làm ca đầu mỗi tệp (4,2–4,4 s lúc load 28) vượt trần 5 s; `cargo test` bị cắt ở trần 21 phút, không ca đỏ, story không đổi Rust.
- e2e chỉ chạy được từng ca một khi `tauri dev` đang giữ cổng 1420 (tem `?t=`). `afterEach` đóng dải để nightly chạy cả spec; chưa chạy cả spec trong một phiên.

## Plan Change Log

## Review Triage Log

Lượt 1 (lens Quick): high 0 · medium 0 · low 2 · false 4 · maybe-false 0.

- false: dải mở bằng phím khi menu đang mở rồi bấm mục ghi đè `restoreOnClose`; không tới được, `isBlocked` ở `main.ts`:1136 chặn hợp âm khi menu mở, chỗ gọi `openGlossaryQuickAdd` còn lại đi qua lớp phủ Quản lý.
- low, reject: `restoreOnClose` ném thì vùng chọn không trả; đã có `console.warn` như nhánh `savedRange`, và rơi về `savedRange` chính là trạng thái mất phím mà story bỏ.
- low, reject: `before.range === null` không trả tiêu điểm khi nhắm ô bản dịch; cần caret khác `null` mà không có `Range`, ít gặp, và `onSelectionChange` kéo caret theo ô đang có tiêu điểm nên phím vẫn vào ô.
- false: `SEL` trùng `SEL2.tgt` và dòng trống thừa trong bàn đo; `typeInto` đang dùng `SEL`, không có hại gọi tên được ở một tệp đo.
- false: Task list của plan còn `[ ]`; sửa plan không phải phát hiện về mã, đánh dấu lúc trình bày.
- false: mục `shortcuts.open` không phản ánh ở spine; chính đúng quyết định "spine không đổi", reviewer cũng ghi không vi phạm.

## Design Notes

Đo trên Playwright WebKit và Chromium bằng chuột thật, chưa phải WKWebView; bàn đo chốt các điều này trên app thật. Với cú bấm tin cậy, microtask chạy giữa hai listener, nên ô thuật ngữ đã có tiêu điểm trước khi `onDocumentClick` chạy; `removeAllRanges` lúc đó không làm hỏng ô, gõ vẫn vào `alphaXY`. Tiêu điểm ở ô bản dịch A với `Range` ở nguyên văn B thì phím `Q` mất hẳn trên cả hai engine: đó là trạng thái mà `restoreFocusAndSelection` tạo ra sau khi chuột phải ở nguyên văn, vì `mousedown` đã chặn nên tiêu điểm vẫn ở A.

Chuột phải ô số khi có từ bôi ở câu khác: WebKit thay vùng chọn, Chromium giữ từ cũ.

## Verification

**Commands:**
- `npm run check:commands` -- exit 0, in 277/29/24.
- `npm run check:i18n`, `npm run check:tokens`, `npm run check:debt-owner` -- exit 0.
- `npm run test:story 14-5` -- exit 0.
- `npx vue-tsc --noEmit`, `npm run check:lint` -- exit 0.
- `cargo build --locked --features wdio --manifest-path src-tauri/Cargo.toml`, rồi `npx wdio run e2e/wdio.conf.mjs --spec e2e/specs/glossary-mouse.e2e.mjs` -- xanh, chạy ngoài sandbox.
- Chạy đủ bộ một lần: story chạm `scripts/check-*` (AGENTS.md §Tests).

**Manual checks (if no CLI):**
- Lượt dùng thật của Ice trước khi Epic 14 `done`: chuột phải trong vùng bôi và ô số, đóng dải rồi gõ tiếp, `Công cụ ▾` tới mục Glossary.
