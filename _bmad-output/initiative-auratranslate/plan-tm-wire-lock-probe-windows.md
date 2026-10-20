---
title: 'tm_wire: ca giữ OpenWorkState khi xuất Work không còn phụ thuộc mốc ngủ 5 ms'
type: 'bugfix'
ticket: ''
created: '2026-10-20'
status: 'built'
baseline_revision: '588360182698c096535519bb234934126b6881e1'
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

**Problem:** `the_work_export_shell_holds_open_work_state_only_while_it_reads_the_store` (`src-tauri/tests/tm_wire.rs`) đỏ trên `check (windows-2025)` của CI `39375446762` ở assert tiền đề "sau 5 ms OpenWorkState phai dang bi giu de do", trong khi cùng ca xanh trên Windows ở nightly `39331390830`; helper `lock_held_against_run` ngủ 5 ms rồi coi lần `try_lock` đầu thấy khoá rảnh là tiền đề sai, nên luồng lệnh chưa kịp lấy khoá (xếp lịch chậm khi 7 ca của tệp chạy song song) cũng làm ca đỏ.

**Approach:** Helper thăm dò liên tục từ lúc khởi chạy (không ngủ, nhường lượt), chờ đến khi thấy khoá bị giữ rồi mới tính lúc khoá rảnh lại; tiền đề đổi thành "phải thấy khoá bị giữ trước khi lượt xuất xong". Assert chính (`held < run / 2`, `run > 150 ms`) giữ nguyên nghĩa; không đổi mã sản phẩm.

</frozen-after-approval>

## Implementation Notes

- Oneshot: một helper test (`lock_held_against_run` trong `src-tauri/tests/tm_wire.rs`), không đổi mã sản phẩm.
- Đo trên macOS (bản debug, 3 lượt chạy riêng ca): khoá bị thấy giữ từ ~0,2 ms, giữ 389–407 ms, lượt xuất 1,52–1,56 s. Giả thuyết ban đầu "mốc ngủ ~15 ms trên Windows dài hơn thời gian giữ khoá" không khớp với cửa sổ này trên macOS; nguyên nhân Windows (luồng lệnh chưa lấy khoá ở lần thăm dò đầu) vẫn là suy luận, chưa đo trên Windows.
- Sau review: `held` và `run` tính từ lúc khoá được thấy giữ lần đầu (độ trễ khởi chạy không còn dồn vào assert chính); thăm dò ngủ 1 ms như bản cũ thay vì quay không nghỉ.
- Đối chứng đỏ: chuyển render + ghi tệp vào trong `with_tier_store` ở `export_tier_to` ⇒ ca đỏ ở `held < run / 2` (giữ 1,85 s / chạy 1,85 s; 1,67 s / 1,67 s), 2/2 lượt, không đỏ ở tiền đề; mã sản phẩm trả về nguyên trạng.

## Review Triage Log

- medium — `held`/`run` đo từ `started` nên độ trễ khởi chạy X biến assert thành `X + h < r`: thật, đã sửa (đo từ lúc thấy khoá giữ).
- medium — vòng quay không ngủ chiếm một lõi suốt ~1,5 s giữa 6 ca song song: thật, đã sửa (ngủ 1 ms; tiền đề chỉ cần thấy khoá trong cửa sổ ~390 ms).
- medium — Implementation Notes kết luận về Windows từ số đo macOS: thật, đã sửa câu chữ thành suy luận.
- low — Verification thiếu bước đọc nửa Windows của CI và kết quả đối chứng: thật, đã bổ sung.
- low — câu hai của doc comment kể thiết kế đã gỡ: thật, đã xoá.

## Verification

**Commands:**
- `cargo test --locked --manifest-path src-tauri/Cargo.toml --test tm_wire` -- expected: 7/7 xanh
- Đối chứng đỏ: gỡ thật phần nhả khoá sớm ở `export_tier_to` (render + ghi tệp chạy trong `with_tier_store`) -- expected: ca đỏ ở `held < run / 2`, không ở tiền đề
- `npm run check:clippy` và `npm run check:fmt` -- expected: xanh
- Sau push: `check (windows-2025)` của CI chạy trên commit này -- expected: `tm_wire` xanh; đỏ thì ghi lý do
