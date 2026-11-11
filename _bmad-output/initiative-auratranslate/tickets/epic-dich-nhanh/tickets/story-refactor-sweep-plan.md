---
title: 'Refactor sweep'
type: 'refactor'
ticket: '7'
created: '2026-11-11'
status: 'built'
baseline_revision: '3dfcf11babe41caf91ab8bede984a3fef66ec21e'
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

**Problem:** Review Triage Log của sáu plan 15.1–15.6 không có phát hiện nào route `defer`: mọi `patch` đã vá trong story, phần còn lại là `reject`/`false`. Còn hai việc sổ sách. (1) Mục sổ nợ "`check:panel-refs` còn 16 lỗi kế thừa" (`Chủ: Epic 15`) vẫn mở, dù `3dfcf11babe41caf91ab8bede984a3fef66ec21e` đã cho 16 ô miễn trừ có tên và `check:panel-refs` thoát 0 ở baseline. (2) Mục "Story 15.5 chưa có CI Windows/UTC" chỉ nói revision 15.5, còn revision 15.6 (`c292f35c4311d80343e78438bde3f7d48af335bd`) cũng chưa lên CI mà không có mục nào nhận.

**Approach:** Nối dòng `→` vào hai mục: đóng ✅ mục (1) trỏ commit; nối 🟡 vào mục (2) để nó nhận cả revision 15.6 và 15.7, `Chủ: Epic 15`. Chạy các test canh mà `test:story` chọn cho 15-1…15-6 một lượt để xác nhận epic vẫn xanh. Chỉ dọn: không chạm phát hiện đã vá hay đã bác, không thêm hành vi, sổ nợ chỉ NỐI dòng.

**Decision (Ice, 2026-11-11):** gộp mục sổ nợ `check:clippy` `ptr_arg` (`Chủ: Ice`, ghi từ 15.2) vào story này: tham số `path` của helper `preview` ở `src-tauri/tests/convention_exchange_contract.rs:115` đổi từ `&PathBuf` sang `&Path` cùng import, rồi đóng ✅ mục nợ. Mục này làm `pre-push` đỏ nên 22 commit của Epic 15 chưa push được.

</frozen-after-approval>

## Implementation Notes

- Oneshot: ba dòng nối vào `deferred-work.md`, một chữ ký helper trong test Rust, một hằng sàn cổng và một spec e2e; không đổi mã sản phẩm.
- Kết quả đo (macOS, Debug, rustc 1.99.1, sau mọi sửa): `npm run build` 0; `check:clippy` 0; 22 target Rust 607 qua / 0 đỏ / 8 bỏ qua; vitest 9 tệp 237/237; e2e bốn spec `story-15-*` xanh; `check:panel-refs` và `check:debt-owner` 0.
- Đã kiểm, KHÔNG làm: 15.1 handoff "plan-cau-dich 🔵 still open" đã có 🔵 2026-11-11 ở `plan-cau-dich-tham-khao-bang-cho-glossary.md:32`; mục nợ `lastRunId` đã ✅ ở 15.3; e2e 15-1 đã có. 15.3 "không có chỉ báo riêng khi `deferred > 0`" là khoảng trống tính năng, không phải dọn.
- `e2e/specs/glossary-context-reference.e2e.mjs` chưa ai chạy sau Epic 15 và đỏ 2/2 lượt (`disabled` là `null`): từ 15.2 readiness của cấu hình nhanh nạp bất đồng bộ lúc mở lớp phủ, nên spec đọc nút một lần trong lúc readiness còn `unknown`. Rust trả đúng `ready: false` (đo bằng spec tạm). Sửa spec: chờ `[data-gq-context-unconfigured]` rồi mới đọc `disabled`; nút bật trong cửa sổ `unknown` là hành vi có sẵn, vô hại (Rust trả `not_configured`), không đổi ở story chỉ-dọn.
- Đối chứng spec trên: 3/3 xanh; gỡ thật `contextReferenceUnconfigured ||` khỏi `:disabled` của `[data-gq-context-translate]` thì đỏ ở dòng 88 `Expected: not null`, đã khôi phục sạch.
- `check:debt-owner` đỏ sẵn ở baseline: sàn `PLANNED_TICKET_FLOOR` 210 dưới 80% của 263 plan có trạng thái (plan 15.6 đẩy số qua ngưỡng, 15.6 không chạy cổng này). Nâng lên 225 = ceil(0.85 × 264) theo công thức cổng tự in; chỉ cổng này đọc hằng đó và tự kiểm B của nó xanh.

## Review Triage Log

Lượt 1 (quick): 0 high · 0 medium · 3 low · 1 false · 0 maybe-false; 3 khẳng định đúng (chữ ký `&Path`, spec chờ cùng cờ với `:disabled`, assert `isExisting()` thừa).
- low · patch — ghi chú đầu tiên nói "chỉ hai dòng, không đổi mã" và trỏ tới Open Questions đã xoá; viết lại.
- low · patch — dòng 🟡 nối vào mục CI không có `Trỏ:` cho 15.6/15.7; thêm trỏ về plan này.
- low · patch — plan chưa ghi kết quả đo; thêm một dòng kết quả.
- low · báo Ice — nâng `PLANNED_TICKET_FLOOR` và sửa spec e2e nằm ngoài Intent đã khoá; cả hai là đỏ sẵn có của Epic 15 chặn chính phép kiểm của vé, ghi ở Implementation Notes, Ice duyệt khi nhận story.
- false — "clippy ✅ không có bằng chứng": `check:clippy` chạy sau phép sửa (log lượt kiểm, bước clippy thoát 0 lúc 21:52:23), `rustc --version` in 1.99.1.

## Verification

**Commands:**
- `npm run check:panel-refs && npm run check:debt-owner && npm run check:clippy` -- expected: exit 0
- `cargo test --manifest-path src-tauri/Cargo.toml` với 22 target hợp của `npm run test:story 15-1…15-6 -- --list` -- expected: xanh
- `npx vitest run` với các tệp frontend hợp của cùng lệnh, cộng tệp test frontend do commit 15-5/15-6 đổi -- expected: exit 0, không dòng `Errors`
- `npm run test:e2e -- --spec` bốn spec `story-15-*` (15-2 và 15-4 không có spec) và `glossary-context-reference.e2e.mjs`, ngoài sandbox, sau khi dựng lại binary `--features wdio` -- expected: xanh
