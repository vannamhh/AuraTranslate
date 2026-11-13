---
title: 'Bấm chữ gạch chân của Proofreader mở lại dải ở đúng lỗi đó'
type: 'feature'
ticket: ''
created: '2026-11-13'
status: 'built'
baseline_revision: 'c7093988466811a06e70c27f2d969aaa657a2a9c'
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

**Problem:** Sau khi `Esc` ẩn dải Proofreader ở một câu, chỉ có `F8`/`Shift+F8`, rời câu rồi quay lại, hoặc quét lại mới làm dải hiện lại; bấm chuột vào chữ gạch chân của Proofreader không làm gì, khác gạch chân của dải quy ước (`onConventionMarkClick`).

**Approach:** Ice đồng ý 2026-11-13: bấm (chuột trái, không bôi đen) vào chữ gạch chân của Proofreader trong ô bản dịch thì dải Proofreader hiện lại và trỏ đúng lỗi đó, cùng đường `land()` mà `F8` đi.

</frozen-after-approval>

## Tasks & Acceptance

**Execution:**
- [x] `src/panels/proofreadHighlight.ts` -- `wrapProofread` ghi `data-proofread-start` lên span của lỗi Proofreader (như `data-convention-start`).
- [x] `src/conventionState.ts` -- `openProofreadFindingAt(segmentId, start)`: tìm lỗi trong `proofreadMarkPositions()` theo câu và `start`, rồi `land({ family: 'proofread', … })`; không thấy thì không làm gì.
- [x] `src/panels/GridPanel.vue` -- trình nghe click của gạch chân quy ước cũng nhận span Proofreader và gọi `openProofreadFindingAt`.
- [x] Test: `proofreadHighlight.test.ts` (span mang `data-proofread-start`), `conventionState.test.ts` (mở đúng lỗi, dời con trỏ, đóng dải quy ước; `start` lạ thì không làm gì), một ca gắn GridPanel thật: bấm span ⇒ `openProofreadFindingAt` nhận đúng câu và `start`.

**Acceptance Criteria:**
- Given dải Proofreader đã bị `Esc` ẩn ở một câu, when bấm vào chữ gạch chân của Proofreader trong câu đó, then dải hiện lại và trỏ đúng lỗi được bấm.
- Given con trỏ ở câu khác, when bấm chữ gạch chân của Proofreader ở câu này, then con trỏ sang câu đó và dải trỏ lỗi đó.
- Given đang bôi đen chữ, when nhả chuột trên chữ gạch chân, then không mở gì.

## Implementation Notes

- Oneshot: ~50 dòng mã, 3 tệp mã, 4 tệp test.
- `onConventionMarkClick` đổi tên `onMarkClick` vì nay nhận cả hai họ gạch chân.
- Đối chứng gỡ thật: `data-proofread-start`, nhánh Proofreader trong bộ chọn, `.proofread-meaning`, lời gọi `land`, dòng bỏ ẩn ở `focusProofreadFinding`, đảo hai nhánh — mỗi chỗ đỏ đúng ca.

## Verification

**Commands:**
- `npx vitest run tests/frontend/proofreadHighlight.test.ts tests/frontend/conventionState.test.ts tests/frontend/gridProofreadMarkClick.test.ts` -- expected: xanh, mã thoát 0.
- `npx vue-tsc --noEmit -p tsconfig.json && npx eslint src tests/frontend/gridProofreadMarkClick.test.ts && npm run check:commands && npm run check:layout` -- expected: mã thoát 0.

## Review Triage Log

Lượt quick, 5 phát hiện: 2 medium, 2 low, 1 bác; 4 sửa (1 ghi sổ nợ).
- medium, sửa: dòng bỏ ẩn ở `focusProofreadFinding` (`proofreadState.ts`) không ca nào chạm. Thêm ca ở `proofreaderStripContent.test.ts`, gỡ dòng thì đỏ.
- low, sửa: `.proofread-meaning` trong bộ chọn không có ca. Ca bấm chạy cho cả hai lớp, gỡ thì đỏ.
- low, sửa: nhánh quy ước của trình nghe đổi tên không có ca. Thêm ca bấm `.convention-mark`, đảo nhánh thì đỏ.
- medium, hoãn: chưa có phép thử chuột thật. Ghi `deferred-work.md`, Chủ: Epic 9.
- bác: kế hoạch nằm ở gốc initiative chứ không trong cây vé. Bước 1 của bmad-build đặt kế hoạch không có vé ở đó; các kế hoạch trước cũng vậy.
