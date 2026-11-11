---
title: 'Refactor sweep'
type: 'refactor'
ticket: '7'
created: '2026-11-11'
status: 'built'
baseline_revision: '57810b33feb0112060b81909ace8f8de2be117f5'
route: 'full'
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

**Problem:** Hầu hết phát hiện review mà plan 16.1–16.6 để lại đã được vá trong chính story của chúng. Đã kiểm trên mã ở `57810b33`: 13/16 phát hiện `patch` của 16.2, 16.3, 16.5, 16.6 đều có mặt. Còn hai việc dọn. (1) Khối `<template v-else>` của `src/ConventionManageOverlay.vue` không thụt con của nó vào một bậc. (2) Mục sổ nợ "nút *Không phải lỗi* và thẻ chưa chạy trong WKWebView thật" vẫn mang `Chủ: Story 16.7` ở dòng cuối, dù vế còn hở (bàn phím) đã được 16.5 trỏ sang lượt dùng thật `Chủ: Epic 16`. Không giao lại thì `check:debt-owner` sẽ chặn 16.7 khép.

**Approach:** Thụt lại khối đó, chỉ đổi khoảng trắng. Nối một dòng `→` vào mục sổ nợ: đóng vế thụt dòng và giao vế bàn phím cho `Epic 16`.

## Boundaries & Constraints

**Always:** `git diff -w` trên tệp `.vue` phải rỗng. Mục sổ nợ chỉ được NỐI thêm dòng, không sửa hay xoá dòng cũ (AGENTS.md §Specs, handoffs, ledger).

**Never:** Tách thân thẻ ra component (16.5 đã bác vì lớn hơn một phép sửa trực tiếp). Chạm các phát hiện đã vá hay đã bác. Mở mục sổ nợ song song cho vế bàn phím. Thêm việc ngoài hai mục trên (vé: "chỉ dọn").

</frozen-after-approval>

## Code Map

- `src/ConventionManageOverlay.vue:278–526` — `<template v-else>` ở cột 6; con 279–525 cũng ở cột 6; thẻ đóng 526 nằm sát dưới `</template>` của 525. Không có `<pre>`, `textarea` hay `white-space: pre` trong khối (chỉ `nowrap` ở CSS :707, :744), nên khoảng trắng không đổi kết quả vẽ.
- `scripts/check-commands.mjs:2467–2470` và `tests/frontend/controls-adoption.test.ts:25` — trỏ tệp theo tên hàm/lớp, không theo số dòng, nên phép thụt không làm lệch.
- `_bmad-output/initiative-auratranslate/deferred-work.md` — mục `source_plan: …story-khong-phai-loi-nho-theo-tac-pham-plan.md`, summary "nút *Không phải lỗi*, thẻ **Không phải lỗi** của lớp phủ…" (khoảng dòng 14050–14054, `grep` bằng `chưa được chạy trong WKWebView thật`); mục `Chủ: Epic 16` lượt dùng thật ở khoảng :13963. Không đọc cả tệp (>1 MB).
- Đã kiểm, KHÔNG làm: A–O của review 16.2/16.3/16.5/16.6 đều có mặt. O chia làm hai: `close_open_work` được canh ở `ipc_contract.rs:2485`, không ở `project_contract.rs`, nhưng vẫn được canh. Hai mục `Chủ: Story 16.7` còn lại đã đóng ở 16.5 (`partition_point` KHÔNG LÀM, quét đè ✅).

## Tasks & Acceptance

**Execution:**
- [x] `src/ConventionManageOverlay.vue` — thụt 279–525 thêm hai dấu cách — để khối con nằm dưới `<template v-else>`.
- [x] `_bmad-output/initiative-auratranslate/deferred-work.md` — nối vào mục WKWebView: `→ 🟡 2026-11-11 (Story 16.7) — khối <template v-else> đã thụt lại (chỉ khoảng trắng). Vế bàn phím còn hở thuộc lượt dùng thật của mục Chủ: Epic 16 (tiêu điểm sau khi bấm nút trong dải bằng bàn phím, Space/Enter trên thẻ). Chủ: Epic 16` — để mục có chủ còn sống.

**Acceptance Criteria:**
- Given tệp `.vue` đã thụt lại, when chạy `git diff -w -- src/ConventionManageOverlay.vue`, then đầu ra rỗng, và mọi dòng 279–525 có thụt lớn hơn dòng 278.
- Given sổ nợ đã nối dòng, when chạy `npm run check:debt-owner`, then thoát 0, và không mục mở hay 🟡 nào của epic còn `Chủ:` cuối là `Story 16.7`.
- Given cả hai thay đổi, when chạy các test canh của lớp phủ, then xanh như ở baseline.

## Implementation Notes

- 13/16 phát hiện `patch` của review 16.2–16.6 đã có mặt trong mã ở baseline, nên story chỉ còn phép thụt lại khối và một dòng giao lại chủ trong sổ nợ.

## Plan Change Log

## Review Triage Log

Pass 1 (quick): 0 high · 0 medium · 0 low · 1 false · 0 maybe-false · 1 bác.
- Dòng sổ nợ trỏ vế bàn phím sang mục `Chủ: Epic 16` lượt dùng thật, nhưng mục đó không nhắc tới bàn phím — false: mục được giao lại là chính mục WKWebView, vẫn mở với `Chủ:` cuối là `Epic 16` và ghi rõ các phép kiểm bàn phím; `check:debt-owner` Kiểm C chặn Epic 16 khép khi mục này còn mở.
- Implementation Notes, Plan Change Log, Review Triage Log rỗng khi `in-review` — bác: phép sửa là sửa chính plan của lượt build.

## Verification

**Commands:**
- `git diff -w -- src/ConventionManageOverlay.vue` -- expected: rỗng
- `npx vitest run tests/frontend/conventionManage.test.ts tests/frontend/conventionImport.test.ts tests/frontend/conventionIgnoreCard.test.ts tests/frontend/controls-adoption.test.ts` -- expected: xanh, exit 0, không dòng `Errors`
- `npm run check:commands && npm run check:lint && npm run check:debt-owner` -- expected: exit 0
