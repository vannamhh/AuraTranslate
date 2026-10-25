---
title: 'Tên Tác phẩm trên bìa, token on-cover'
type: 'feature'
ticket: ''
created: '2026-10-25'
status: 'built'
baseline_revision: '2eddaf35743a5df89005f7dee0fdd7677d9ba5cb'
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

**Problem:** Bìa thẻ Tác phẩm ở lưới Library là một khối màu trống; tên nằm ở phần dưới thẻ. App chưa có đường đặt ảnh bìa (FR3), nên bìa không cho biết gì.

**Approach:** Theo `ux-auratranslate/DESIGN.md` §Tông bìa Tác phẩm (lượt hai, Ice duyệt) và phiếu `library-nghiem-thu-thi-giac.md` L1/L2: tên in trên bìa, góc trên trái, serif 16 px, giãn dòng 1,35, cắt 3 dòng, màu `on-cover`; phần dưới thẻ không lặp tên; bìa cao 136 px. Token `on-cover = #fbfaf6` ở cả hai theme vào `src/tokens/tokens.json`, vào bảng đóng băng của `scripts/check-tokens.mjs` (`EXPECTED_COLORS_LIGHT`/`DARK`, `colorsPerTheme` 36 → 37), và 6 cặp `on-cover` × `cover-N-from` vào `contrast.pairs` (cặp kiểm ở cả hai theme). `on-cover` không vào `roles.text`: thêm vào đó sẽ đòi 6 tổ hợp với các nền chung, mà Ice không yêu cầu.

</frozen-after-approval>

## Implementation Notes

Route oneshot: khoảng 40 dòng ở 4 tệp (`tokens.json`, `check-tokens.mjs`, `LibraryMode.vue`, test). Đo trước: `#fbfaf6` trên sáu `cover-N-from` thấp nhất 7,05:1 (sáng) và 6,09:1 (tối), đều tông 4 — khớp số trong `DESIGN.md`.
- Giãn dòng: plan nói 1,35 (theo `DESIGN.md`), không token nào mang 1,35 và `check:tokens` cấm viết thẳng. Lần hỏi đầu tôi đưa hai lựa chọn đều dưới §Giãn dòng 1.66 là sàn cứng mà không nêu luật đó; rà soát bắt được, Ice chọn lại `read-sm` 16 px / 1,66 cắt 3 dòng. `DESIGN.md` sửa tại chỗ có 🔵.
- `aria-label` trên `span` không có tác dụng với trình đọc màn hình; tên đầy đủ nằm nguyên trong DOM (CSS cắt) và ở `title` cho người dùng chuột.
- Ca e2e mới trong `story-12-24-library-wide-grid` tạm thay tên bằng chuỗi dài để đo bố cục thật (màu, 1,66, 3 dòng, không đè nút đáy, bìa 136 px); đối chứng bằng cách trả giãn dòng về `read-title`: đỏ ở tỉ lệ 1,30.

## Review Triage Log

Quick, vòng 1: medium 1 · low 3 · false 2.

- medium · patch — giãn dòng mượn `read-title` (`wraps: false`, 1,3) cho tên xuống 3 dòng, dưới sàn cứng 1,66 và lọt qua cổng vì cổng chỉ xét token `wraps: true`; Ice chọn `read-sm` 1,66.
- low · patch — tên bị cắt mà không có chỗ nào cho thấy tên đầy đủ khi nhìn: thêm `title`.
- low · patch — không ca nào chạm màu `on-cover`, cắt 3 dòng, chiều cao bìa; thêm ca e2e đo trên WebKit.
- low · reject (sửa plan) — Intent ghi 1,35 trong khi mã dùng 1,66: quyết định của Ice lúc dựng, ghi ở Implementation Notes và `DESIGN.md`.
- false — "tên 3 dòng cộng nút có thể tràn bìa": đo trên app thật, đáy tên ≤ đỉnh nút và đáy nút ≤ đáy bìa với tên 3 dòng (ca e2e mới).
- false — "frontmatter cũ": `status` đặt ở bước kết thúc; `context` rỗng vì Intent đã chép đủ thông số từ `DESIGN.md`.
