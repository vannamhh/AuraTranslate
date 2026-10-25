# Handoff: thanh tiêu đề macOS (story 12-25)

## Pha 1 (Rust và cấu hình) — xong, chưa commit

- `tauri.conf.json`: cửa sổ `main` thêm `titleBarStyle: "Overlay"`, `hiddenTitle: true`, `trafficLightPosition {x:16,y:13}` (số bắt đầu, Ice chốt sau).
- `capabilities/main.json`: 5 quyền (thêm `core:window:allow-start-dragging`, `core:window:allow-internal-toggle-maximize`), `description` sửa.
- `config_invariants.rs`: `assert_eq!` năm chuỗi + thông điệp; test mới `main_window_block_matches_ad_58`. Đối chứng đỏ đã chạy: xoá `titleBarStyle` ⇒ đỏ; thêm `core:window:default` ⇒ đỏ. 50/50 xanh sau khi trả lại.
- `SECURITY-NOTES.md`, `scripts/check-deps.mjs:180`, `lib.rs` (1113, 1931, 1993, 2130): chữ ba ⇒ năm, bỏ con trỏ `:333`.
- `cargo fmt` đã chạy (rustfmt đứng riêng lỗi edition, dùng `cargo fmt`).

## Còn lại cho pha sau

- Pha Webview: `main.ts` dấu `data-platform="mac"`, `App.vue`, `tokens.json` (token `traffic-light-inset` + mục `deviations`), 3 menu capture, 17 lớp phủ.
- Pha Tests: `titlebarDragRegion.test.ts` mới, ba test menu, `deferred-work.md` hai mục nợ (Epic 12, B7).
- Chưa chạy: bộ đầy đủ, e2e, `check:tokens` sau pha 2.

## Pha 2 (Webview) — xong, chưa commit

- `main.ts`: `detectIsMac()` ⇒ `documentElement.dataset.platform = 'mac'` ngay trước `mount`. `App.vue`: `const titlebarDragRegion = detectIsMac() ? 'deep' : undefined`, gắn `:data-tauri-drag-region="titlebarDragRegion"` trên `header.titlebar`; `user-select: none`; `:root[data-platform='mac'] .titlebar { padding-left: var(--space-traffic-light-inset) }`.
- `tokens.json`: `spacing.traffic-light-inset` 88px + mục `deviations` (check:tokens xanh, không đụng EXPECTED_SPACING).
- Ba menu: add và remove `mousedown` đều thêm `true`.
- 17 lớp phủ: sau mỗi khối `.xx-scrim` thêm `:root[data-platform='mac'] .xx-scrim { padding-top: var(--space-titlebar-height); }` (aip, ti, gm, pl, ri, hist, gh, al, gi, cs, set, ex, li, gq, tm, cm, pi).
- Đã sửa regex `titlebarToolsMenu.test.ts:184` thành `<header class="titlebar"[^>]*>` (nếu không hai ca đỏ vì thuộc tính mới).
- Chạy: check:tokens xanh; vitest cả thư mục tests/frontend xanh; vue-tsc và eslint cho App.vue, main.ts sạch.
- Cho pha 3: `titlebarDragRegion.test.ts` (ca dấu nền tảng hai chiều có thể kiểm `detectIsMac` + `dataset`; vùng kéo đọc từ `App.vue` là biểu thức `titlebarDragRegion`, không phải chuỗi `deep` nằm trên thẻ — regex cần đọc cả `const titlebarDragRegion = detectIsMac() ? 'deep'`); ca bấm-header cho 3 menu; hai mục nợ Epic 12 + B7; e2e và bộ đầy đủ chưa chạy.

## Pha 3 (Tests) — xong, chưa commit

- Mới: `tests/frontend/titlebarDragRegion.test.ts` (thuộc tính duy nhất trên `header.titlebar` + biểu thức `titlebarDragRegion`, quét `src/` ba mẫu cấm, dấu `data-platform` trước mount, `detectIsMac` hai chiều, lề chỉ dưới dấu nền tảng) và `tests/frontend/support/dragRegionPress.ts` (giả `drag.js` + bấm header).
- Ba test menu thêm một ca bấm header kéo. Ca "dấu nền tảng hai chiều" chạy `detectIsMac` với `navigator.platform` giả, không chạy `boot()`; vị trí dấu trong `main.ts` canh bằng thứ tự chuỗi.
- Đối chứng gỡ thật (đã trả lại): dời thuộc tính lên `<main>` ⇒ ca App đỏ; bỏ `true` ở từng menu ⇒ đúng ca menu đó đỏ; xoá dòng `dataset.platform` ⇒ ca main đỏ.
- `deferred-work.md`: hai mục cuối tệp (Epic 12, B7).
- Xanh: 4 tệp vitest (30), `npm run build` + `config_invariants` (50), `check:tokens`, `check:debt-owner`. Chưa chạy bộ đầy đủ/e2e.
