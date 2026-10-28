---
title: 'Thanh lọc Quản lý bộ nhớ dịch: bỏ ô trống thừa trong ô phân đoạn Tầng, mọi control cùng 28px và thẳng hàng'
type: 'bugfix'
ticket: ''
created: '2026-10-28'
status: 'built'
baseline_revision: '0275401a9cc78b158ef1f186bf7c7a706fcd2a86'
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

**Problem:** Hai lỗi trên thanh lọc của `TmManageOverlay.vue`, đo trong trình duyệt ở cổng 1420, dpr 2.
- **Ô trống thừa.** Khi chưa mở Tác phẩm, ô "Tác phẩm này" bị khoá và ô phân đoạn Tầng hiện thêm một ô trống ở mép trái. Nguyên nhân: `.ui-seg__item` khai `border: none`, nên độ dày viền giữ mặc định `medium` (3px); luật khoá đặt `border-left-style: dashed`, và ô đầu vẽ ra một viền đứt 3px.
- **Hàng lệch.** Fieldset Tầng và Xuất xứ bị flex kéo dãn rồi căn giữa nội dung, nên control cách nhãn 2px; ô Tìm và "Sắp theo" cách nhãn 4px. Thêm vào đó, chip và trigger "Ghi gần nhất" cao 24px trong khi ô Tìm và ô phân đoạn cao 28px.

**Approach:** Làm theo quyết định của Ice ở bản sửa Glossary: mọi control trên một hàng cao 28px.
- Ô phân đoạn đặt độ dày viền về 0. Bản sửa nằm ở `controls.css`, nên Glossary cũng hết lỗi này.
- Thanh lọc TM căn các cột theo mép trên. Legend có khoảng cách 4px như nhãn cột. Chip và trigger mang `ui-chip--md`/`size="md"`.

</frozen-after-approval>

## Implementation Notes

Tuyến oneshot: khoảng 15 dòng ở `src/tokens/controls.css` và `src/TmManageOverlay.vue`, cộng test.

- `border: 0` thay `border: none` trên `.ui-seg__item`. Đo: ô đầu bị khoá trước có `border-left-width` 3px, nay 0px; ô thứ hai bị khoá vẫn `1px dashed`. Luật nằm chung trong `controls.css`, nên ô Toàn cục của Glossary (bị khoá khi đang sửa) cũng hết lỗi.
- Legend `margin-bottom` 4px, và thanh lọc căn mép **dưới** (`flex-end`), lệch khỏi chữ "mép trên" của Approach. Ở 560px, nhãn ô Tìm xuống 2 dòng: căn mép trên đặt ô Tìm ở 140 và ô Tầng ở 122; căn mép dưới đặt cả hai ở 140. Ở 1440px, nhãn ở 100,25/166,25 và control ở 122,25/188,25 với cả hai cách căn; mọi control cao 28.
- Đo thêm ở 960 và 700px: "Sắp theo" xuống hàng riêng, gọn. Ở 560px, chip "Nhập song ngữ" xuống dòng thứ hai trong fieldset, không lệch.

## Review Triage Log

Lượt `quick` 1: low 2 · false 1.
- low, patch: assert `border-left-width` không thể đỏ. Đã bỏ; assert `border: 0` vẫn đỏ nếu trả về `none`.
- low, patch: ca cửa sổ hẹp chưa đo, và dòng 🟡 trong sổ nợ không nêu phạm vi. Đã đo 960/700/560px (ra lý do đổi sang `flex-end`) và ghi phạm vi vào dòng 🟡.
- false: frontmatter `status` vẫn `in-progress`. Lúc review đó là trạng thái đúng; nay đã `built`.

## Verification

**Commands:**
- `npx vitest run tests/frontend/tmManage.test.ts tests/frontend/controls-css.test.ts` -- expected: xanh, có ca canh viền ô phân đoạn và bậc md trên thanh lọc TM

**Manual checks (if no CLI):**
- Mở Quản lý bộ nhớ dịch ở `http://localhost:1420`. Ô Tìm và ô phân đoạn Tầng cùng `top`, cao 28. Chip Xuất xứ và trigger "Ghi gần nhất" cùng `top`, cao 28. Khoá `input` của "Tác phẩm này": `border-left-width` bằng 0.
