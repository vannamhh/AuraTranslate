---
title: 'Esc trong ô bản dịch ẩn dải nội tuyến đang hiện'
type: 'bugfix'
ticket: ''
created: '2026-11-13'
status: 'built'
baseline_revision: 'c9858bfa904bf2e2d4694db070fa51014ecb65cd'
route: 'oneshot'
route_source: 'auto'
risk: 'low'
review: 'quick'
review_source: 'pinned'
lenses_ran: ['quick']
review_loop_iteration: 0
context: []
---

<frozen-after-approval reason="human-owned intent — do not modify unless human renegotiates">

## Intent

**Problem:** Các dải nội tuyến (đề xuất TM, Proofreader, xác nhận Glossary, thêm thuật ngữ) chỉ ẩn bằng `Esc` khi tiêu điểm nằm trong dải (`@keydown.esc` trên gốc dải). Khi con trỏ ở ô bản dịch, `Esc` đi vào `GridPanel.vue::onEditKeydown`, nơi chỉ dải quy ước được ẩn; Ice phải bấm vào dải rồi mới `Esc` được.

**Approach:** Ice báo 2026-11-13: `Esc` trong ô bản dịch phải ẩn dải đang hiện. `onEditKeydown` dispatch lệnh ẩn của dải đứng trên cùng (`topmostStrip(eligibleInlineStrips)`) — cùng lệnh mà `Esc` trong dải đó dispatch — thay cho nhánh riêng của dải quy ước; hai dispatch `Esc` còn lại giữ nguyên.

</frozen-after-approval>

## Tasks & Acceptance

**Execution:**
- [x] `src/panels/GridPanel.vue` -- nhánh `Escape` của `onEditKeydown`: `switch (topmostStrip(eligibleInlineStrips.value))` dispatch literal lệnh ẩn của dải tự động đứng trên cùng (`ai.proofread.hide` cho cả hai loại proofreader, `convention.hide`, `tm.fuzzy.hide`); hai dải Glossary không làm gì; `default: top satisfies never`.
- [x] `scripts/check-commands.mjs` -- `HANDLER_TABLE['src/panels/GridBlock.vue::onEditKeydown']` khai thêm `ai.proofread.hide`, `tm.fuzzy.hide`.
- [x] `tests/frontend/gridEscHidesTopmostStrip.test.ts` -- dải TM ở câu có con trỏ ⇒ chỉ `hideTmFuzzyStrip`; dải thêm thuật ngữ trên dải TM ⇒ không lệnh ẩn nào; không dải ⇒ không lệnh ẩn, vẫn xoá điểm cắt và vùng chọn. Spy cả năm lệnh ẩn.

**Acceptance Criteria:**
- Given dải đề xuất TM đang hiện và con trỏ ở ô bản dịch, when bấm `Esc`, then dải ẩn mà không phải bấm vào dải trước.
- Given hai dải cùng đủ điều kiện, when bấm `Esc` trong ô, then chỉ dải đang hiện (trên cùng) bị ẩn.
- Given không dải nào hiện, when bấm `Esc` trong ô, then hành vi như cũ (xoá điểm cắt, bỏ vùng chọn câu).

## Implementation Notes

- Oneshot: ~40 dòng, 3 tệp. `GridPanel.vue` đã import `inlineStripEligibility`, nên không thêm cạnh import mới.
- Bản đầu tra id qua một bảng `Record` rồi `dispatch(map[top])`; `check:commands` đỏ vì nó đọc tĩnh `dispatch` literal. Đổi sang `switch` literal: cổng thấy được, `satisfies never` bắt loại dải mới chưa có nhánh.
- Đối chứng: gỡ nhánh `tm_fuzzy` ⇒ vitest, `vue-tsc` và `check:commands` cùng đỏ; cho dải thêm thuật ngữ đóng từ ô ⇒ ca hai dải đỏ.
- Hai dải Glossary (thêm thuật ngữ, xác nhận) bị loại khỏi `Esc` trong ô: lệnh đóng của chúng trả tiêu điểm và vùng chọn đã lưu lúc mở, kéo con trỏ khỏi ô đang gõ; thông báo "đã chốt" không có lệnh ẩn. Chúng giữ `Esc` riêng bên trong dải.
- Sửa `scripts/check-*` ⇒ đã chạy toàn bộ vitest (275 tệp, 4416 ca) và 15 cổng của pre-push, đều xanh; `cargo test` không chạy vì lượt này không đụng Rust.

## Verification

**Commands:**
- `npx vitest run tests/frontend/gridEscHidesTopmostStrip.test.ts tests/frontend/editorClearSourceCuts.test.ts tests/frontend/inlineStripPriority.test.ts tests/frontend/editorTypingZone.test.ts` -- expected: xanh, mã thoát 0.
- `npx vue-tsc --noEmit -p tsconfig.json && npx eslint src/panels tests/frontend/gridEscHidesTopmostStrip.test.ts && npm run check:commands` -- expected: mã thoát 0.

**Manual checks (if no CLI):**
- Ice, app thật: con trỏ trong ô bản dịch, dải TM hiện, bấm `Esc` ⇒ dải ẩn.

## Review Triage Log

Lượt quick, 7 phát hiện: 2 medium, 5 low; 6 sửa, 1 ghi sổ nợ; một vế của phát hiện AC2 bị bác.
- medium, sửa: chỉ có thông báo "đã chốt" của dải xác nhận Glossary thì `glossary.confirm.defer` không làm gì (`current` rỗng). Đã loại dải xác nhận khỏi `Esc` trong ô.
- medium, sửa: đóng dải thêm thuật ngữ hoặc hoãn dải xác nhận từ ô trả tiêu điểm về chỗ đã lưu lúc mở, con trỏ nhảy khỏi ô. Loại cả hai; ca test hai dải canh, cho đóng lại thì đỏ.
- low, sửa: AC2 (chỉ dải trên cùng) không có ca. Thêm ca dải thêm thuật ngữ trên dải TM.
- low, sửa: ca "không dải" chỉ spy một lệnh và không khẳng định vế xoá điểm cắt. Nay spy năm lệnh ẩn cùng `clearSourceCuts` và `clearSegmentSelection`.
- low, sửa: ca TM không đi đường con trỏ. Nay đặt con trỏ ở câu 11 rồi quét theo con trỏ.
- bác (vế của phát hiện AC2): hoán đổi hai id literal vẫn xanh. Nay chỉ còn ba nhánh dispatch; nhánh TM có ca, `convention.hide` và `ai.proofread.hide` khác hẳn tên dải nên khó lẫn. Không đáng thêm hai bộ dựng state.
- low, sửa: sửa `scripts/check-*` cần chạy đủ bộ. Đã chạy đủ vitest và 15 cổng.
- low, hoãn: phép thử tay cần mục nợ. Đã ghi `deferred-work.md`, Chủ: Epic 2.
