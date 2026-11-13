---
title: 'Shift+Enter gán lại cho editor.confirm_segment chạy được trong ô bản dịch'
type: 'bugfix'
ticket: ''
created: '2026-11-13'
status: 'built'
baseline_revision: '8917099f7bb00cdc57790d1ef22ebb0eeaf5df44'
route: 'oneshot'
route_source: 'auto'
risk: 'medium'
review: 'quick'
review_source: 'pinned'
lenses_ran: ['quick']
review_loop_iteration: 0
context: []
---

<frozen-after-approval reason="human-owned intent — do not modify unless human renegotiates">

## Intent

**Problem:** Ice gán `editor.confirm_segment` sang `Shift+Enter` ở màn phím tắt, màn nhận và lưu, nhưng trong ô bản dịch phím không xác nhận gì mà xuống dòng: `src/commands/keys.ts:531` cho mọi hợp âm thiếu `⌘`/`Ctrl` nhường vùng gõ (`isTypingZone`), và mọi ô bản dịch là `contenteditable`.

**Approach:** Ice quyết 2026-11-13: khi người dùng gán `Shift+Enter` cho lệnh xác nhận câu, `Shift+Enter` trong ô bản dịch phải xác nhận. Mặc định vẫn `Mod+Enter`; `Enter` trần vẫn là xuống dòng (FR134, EXPERIENCE.md §Phím: `Enter` trần không bao giờ xác nhận). Lệnh khai một cờ trên `CommandSpec`; chỉ hợp âm đúng `Shift+Enter` của lệnh có cờ được bắn trong vùng gõ, mọi hợp âm thiếu phím bổ trợ chính khác vẫn nhường như cũ.

</frozen-after-approval>

## Tasks & Acceptance

**Execution:**
- [ ] `src/commands/registry.ts` -- thêm `CommandSpec.shiftEnterWhileTyping?: boolean`, chuẩn hoá về `boolean` trong `frozen()` như `repeatable` -- cờ phải sống qua `register()`.
- [ ] `src/commands/keys.ts` -- lúc biên dịch tính cho mỗi entry `firesWhileTyping = spec.shiftEnterWhileTyping && code === 'Enter' && mods chỉ có shift`; dòng 531 bỏ qua phép nhường khi `firesWhileTyping` -- chốt IME (`isComposing`) đứng trước, không đổi.
- [ ] `src/commands/index.ts` -- `editor.confirm_segment` khai `shiftEnterWhileTyping: true`.
- [ ] `tests/frontend/keysShiftEnterConfirmInTypingZone.test.ts` -- ca: (1) qua `installCommands` thật với override `Shift+Enter` ⇒ trong ô contenteditable chạy `confirmSegment`, có `preventDefault`; (2) `Enter` trần gán cho lệnh có cờ vẫn nhường; (3) `Shift+Enter` gán cho lệnh không cờ vẫn nhường; (4) `Alt+Enter` cho lệnh có cờ vẫn nhường; (5) `isComposing` ⇒ không chạy.

**Acceptance Criteria:**
- Given Ice đã gán `Shift+Enter` cho "Xác nhận câu đang sửa", when con trỏ ở trong một ô bản dịch và bấm `Shift+Enter`, then câu được xác nhận và không có dòng mới nào chèn vào ô.
- Given mặc định chưa gán lại, when bấm `Shift+Enter` hoặc `Enter` trong ô bản dịch, then ô xuống dòng như trước.
- Given bộ gõ tiếng Việt đang dựng chữ, when bấm `Shift+Enter`, then không câu nào được xác nhận.

## Implementation Notes

- Oneshot: ~95 dòng, 4 tệp, một luật hẹp trong `keys.ts`.
- Chỉ đúng `Shift+Enter` (không `Alt+Enter`, không phím khác): đó là điều Ice quyết; mở rộng sau phải hỏi lại.
- Màn phím tắt vẫn nhận im lặng một hợp âm thiếu `⌘`/`Ctrl` khác cho lệnh chỉ chạy trong ô (vd. `Alt+Enter`) — chưa sửa ở đây.
- Lối tắt chỉ mở khi đích là một editing host (`isContentEditable`); ô input, textarea ngoài lưới vẫn nhường (review phát hiện 1).
- Tệp test mới đẩy quần thể `check:doc-refs` lên 942, `FILE_FLOOR` 753 trôi dưới 80%, nâng lên 801 theo lời cổng.

## Verification

**Commands:**
- `npx vitest run tests/frontend/keysShiftEnterConfirmInTypingZone.test.ts tests/frontend/keysModZYieldsToNativeEditor.test.ts tests/frontend/editorTypingZone.test.ts` -- expected: xanh, mã thoát 0.
- `npm run check:commands && npx vue-tsc --noEmit -p tsconfig.json && npx eslint src/commands tests/frontend/keysShiftEnterConfirmInTypingZone.test.ts` -- expected: mã thoát 0.

**Manual checks (if no CLI):**
- Ice, app thật (WKWebView): gán `Shift+Enter`, gõ trong ô bản dịch, bấm `Shift+Enter` ⇒ câu xác nhận, không xuống dòng.

## Review Triage Log

Lượt quick, 6 phát hiện: 3 medium, 3 low; 4 sửa, 1 hoãn, 1 bác.
- medium, patch: lối tắt vượt mọi vùng gõ, `Shift+Enter` trong ô thêm thuật ngữ (`GlossaryQuickAdd.vue`, không nằm trong cổng `main.ts`) sẽ ký câu. Đã giới hạn ở `isContentEditable`, có ca INPUT, gỡ điều kiện thì đỏ.
- low, patch: `!mods.alt` không ca nào chạm (Alt+Enter đã rơi ở `mods.shift`). Thêm ca `Alt+Shift+Enter`, gỡ vế thì đỏ. Bỏ vế `lacksPrimaryMod(mods)` thừa.
- medium, patch: AC2 không có ca qua bộ phím mặc định thật. Thêm ca `installCommands` mặc định, gắn cờ nhầm cho `library.continue_translation` thì đỏ.
- medium, defer: phép thử tay trong WKWebView chưa có chủ. Đã ghi `deferred-work.md`, Chủ: Epic 1.
- low, patch: EXPERIENCE.md §Phím hàng `Shift+Enter` không nói ngoại lệ mới. Thêm ghi chú 🔵 2026-11-13 tại chỗ.
- low, bác: doc-comment `GridPanel.vue` `onEditKeydown` ("hợp âm Enter duy nhất là `Mod+Enter`") đã sai từ trước vì `library.continue_translation` giữ `Shift+Enter`. Lượt này không sửa dòng đó, và AC11 (`Enter` trơn không ký) vẫn đứng nhờ `sameMods` cùng vế `mods.shift`.
