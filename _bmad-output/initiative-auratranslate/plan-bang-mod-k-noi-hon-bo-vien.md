---
title: 'Bảng Mod+K nổi hơn, bỏ viền'
type: 'feature'
ticket: ''
created: '2026-11-14'
status: 'built'
baseline_revision: '847b5b3c7e5b649b5a95fe422c37fb4497ee7aec'
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

**Problem:** Ice xem app thật thấy khung Mod+K (`PaletteOverlay.vue`) chưa tách khỏi trang phía sau: bóng `--shadow-popover` nhẹ, scrim trong suốt, và viền `control-rule` 1px làm khung trông như một ô kẻ trên trang.

**Approach:** Bỏ viền ngoài của khung; ô gõ giữ viền `ui-field` như mọi ô nhập khác (Ice chọn 2026-11-14). Tách khung khỏi nền bằng cách làm tối phía sau: scrim `backdrop-filter: brightness(0.82)`, giữ bóng `--shadow-popover`, không token mới (Ice chọn phương án B 2026-11-14, sau khi xem bản dựng thử ba phương án: A bóng đậm hơn qua token mới, B nền tối đi, C nền nhoè + tối nhẹ). Hành vi scrim (bấm ra ngoài thì đóng, `@mousedown.self.prevent`, Esc, bẫy Tab) giữ nguyên.

</frozen-after-approval>

## Implementation Notes

- Oneshot: chỉ `src/PaletteOverlay.vue`, ba khai báo CSS. Phương án B không cần token mới vì `brightness()` không phải literal màu (`check:tokens` Kiểm B/F không chạm).
- `.pal-panel:focus-visible` từng đổi `border-color` sang `primary`; mất viền nên chỉ báo nay là `outline: 1px solid var(--color-primary)`.
- `tauri.conf.json` không khai `minimumSystemVersion` và `backdrop-filter` không tiền tố chỉ có từ Safari 18; `vite build` (Vite 8.2.0) tự phát cả `-webkit-backdrop-filter`, đã grep `dist/assets/*.css`.
- Playwright WebKit headless không vẽ `backdrop-filter` vào ảnh chụp (pixel nền 244,241,234 không đổi; Chromium 171,169,164 ở `brightness(0.7)`), nên bản dựng thử chụp bằng Chromium. Mắt người trong WKWebView thật vẫn là phép nghiệm thu.

## Review Triage Log

Lượt quick: 0 high · 0 medium · 2 low · 3 false.
- false — "chưa chạy `check:tokens`": đã chạy, exit 0.
- false — "macOS trước 15 mất lớp tối im lặng": `dist/assets/*.css` có cả `-webkit-backdrop-filter:brightness(.82)`, tiền tố có từ Safari 9.
- false — "ô gõ giữ viền chưa xác nhận": template không đổi, ô vẫn mang `ui-field` (viền `control-rule` ở `controls.css`).
- low, bỏ — vòng `outline` của `.pal-panel:focus-visible` nằm ngoài khung: `outline` không bị `overflow` của chính phần tử cắt, và khung chỉ nhận tiêu điểm khi bấm vào vùng không phải ô.
- low, bỏ — không có test cho "không viền / nền tối": thuần CSS, jsdom không dựng hình; quét nguồn CSS chỉ canh chính dòng đó. Nghiệm thu bằng mắt ở mục nợ Mod+K (Chủ: Ice).

## Verification

**Commands:**
- `npx vitest run tests/frontend/palette.test.ts` -- expected: xanh
- `npm run check:tokens` -- expected: xanh

**Manual checks (if no CLI):**
- Ice mở Mod+K trong app thật ở cả hai theme.
