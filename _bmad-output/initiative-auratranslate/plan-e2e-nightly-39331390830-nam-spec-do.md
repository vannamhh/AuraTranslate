---
title: 'e2e: năm spec đỏ ở nightly 39331390830 là lỗi của chính spec, không phải hồi quy sản phẩm'
type: 'bugfix'
ticket: ''
created: '2026-10-20'
status: 'built'
baseline_revision: '0501f3edfa75923ffe74219b8db6753f51dbfd60'
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

**Problem:** Nightly `39331390830` (`schedule`, `e2e (macos-26)`, tại `5e7d249efe04c3b817b5bc6ecb00c7c0b54f0b71`) đỏ 5/49 spec: `story-12-10-glossary-tm-suggestions`, `story-12-13-glossary-harvest`, `story-12-14-prompt-library`, `story-12-14-tm-only-mine`, `story-5-13-reading-marks`. Chạy lại năm spec đó trên máy Ice ở `0501f3edfa75923ffe74219b8db6753f51dbfd60` đỏ đúng năm chỗ, cùng thông báo, nên lỗi tất định chứ không do môi trường. Bốn spec 12-x chưa từng chạy xanh ở đâu (plan của chúng ghi chờ nightly). Spec 5-13 thì cũ đi vì `31d608716e5b2901a6700ea9514d3ee8c6eca6b4` (12-15) chuyển `display:none` từ `.mark-affordance` sang khung bọc `.mark-actions`.

**Approach:** Sửa từng spec cho khớp hành vi thật của sản phẩm, không đổi mã sản phẩm. Chạy lại năm spec trên máy đến khi xanh. Với mỗi spec, đối chứng bằng một phép gỡ thật trong mã sản phẩm để chắc nó vẫn đỏ được.

**Decision (Ice, 2026-10-20):** spec 5-13 dò chính nút `.mark-affordance` bằng `getClientRects().length` (nút có được vẽ hay không), không dò `display` của khung bọc `.mark-actions`.

</frozen-after-approval>

## Implementation Notes

- Đi đường oneshot vì chỉ chạm năm tệp `e2e/specs/*.e2e.mjs`, chừng 30 dòng; mã sản phẩm không đổi.
- 12-10: fixture `萧炎\n萧炎\n萧炎\n萧炎登场。` bị `NormalizeParagraphsAndWhitespace` (`core/segment/pipeline.rs`, phép 3 của `normalize.rs`) nối thành một đoạn, vì dòng không có dấu kết câu và không cách bằng dòng trống. Kết quả là 1 câu thay vì 4. Sửa: tách các dòng bằng dòng trống (`\n\n`).
- 12-13: `GlossaryManageOverlay.vue` chỉ dựng `<table class="gm-list">` khi Glossary có hàng; với Glossary rỗng nó hiện `.gm-empty`. Spec mở trình quản lý mà chưa thêm thuật ngữ nào, nên đếm được 0 cột. Sửa: thêm một thuật ngữ bằng `glossary_add_term` trước ca ba. Phương án "khẳng định trạng thái rỗng" bị loại vì bỏ mất điều ca này khẳng định (bảng bảy cột).
- 12-14 prompt-library: `.pl-row-form button*=NAME` không phải bộ chọn hợp lệ. Trong webdriverio 9.30.1, dạng `tag*=text` chỉ được dịch khi nó đứng một mình, không đi sau tổ hợp con cháu của CSS. Sửa: `$('.pl-panel').$('button*=NAME')`.
- 12-14 tm-only-mine: nhập song ngữ chỉ đặt `translation_origin = 'bilingual_import'` trên segment mà không ghi hàng nào vào `tm_unit`; cặp TM chỉ sinh ra khi segment được xác nhận. Sửa: gọi `confirm_segment` sau khi nhập, giống `story-12-10…:69`. Bỏ câu "Not yet run green…" ở đầu tệp và sửa câu nói sai nguồn của cặp.
- 5-13: hai lần dò `display` của `.mark-affordance` đổi sang `getClientRects().length`. Sau đó lộ chỗ cũ thứ hai, cũng từ `31d6087`: lớp phủ danh sách dấu nay là `.marks-overlay .marks-list`, nhãn hàng `Ch N`, và ba nút Trước/Sau/Mở được thay bằng ↑ ↓ trên `.marks-panel` cùng nút "Sửa →" trên từng hàng. Spec chuyển sang giao diện đó và giữ nguyên các điều nó khẳng định (hai hàng, đứng yên ở biên, mở dấu đã retire bằng neo sống).
- Máy Ice, `--features wdio`, cây đã sửa trên `0501f3e`: 5/5 tệp, 8/8 ca xanh. Đối chứng gỡ thật trong mã sản phẩm, mỗi spec đỏ đúng lý do: bỏ lượt nạp TM của dải (12-10:87), bỏ một `<th>` (12-13: 6 cột), bỏ đồng bộ `scrollTop` của mirror (12-14 prompt: lệch 120 px), bỏ bộ lọc `police` (tm-only-mine:90 `hidden_count` 0), bỏ `display:none` của `.mark-actions` (5-13:90). Lần gỡ `TmSidePolicy::from_work` đầu tiên đỏ ở :87, sai lý do, nên đã thay bằng lần gỡ ở `police`. Mã sản phẩm đã trả về nguyên bản.

## Review Triage Log

- low · patch: đầu tệp `story-12-14-prompt-library.e2e.mjs` còn câu "Not yet run green…" (lịch sử, sai từ nay); đã xoá.
- medium · patch: hai mục sổ nợ (12-14 `story-12-14-*` chưa chạy xanh; 12-13 chưa chạy cục bộ) không còn đúng; đã thêm dòng `→ 🔵 2026-10-20`, giữ `Chủ: Epic 12`, `check:debt-owner` đạt.
- low · reject: plan của story 12-14 (đã `done`) ghi "chờ nightly" ở :130/131/158; đó là trạng thái tại lúc viết, sổ nợ đã mang phần còn mở.
- low · patch: "Ice chọn b" trong Implementation Notes trỏ tới phương án không có trong plan; đã bỏ, Decision đã nói cách được chọn.

## Verification

**Commands:**
- `npm run build && cargo build --locked --features wdio --manifest-path src-tauri/Cargo.toml && npx wdio run e2e/wdio.conf.mjs --spec <năm spec>` (chạy ngoài sandbox) -- expected: 5/5 passed
- `npx eslint e2e/specs` -- expected: sạch
