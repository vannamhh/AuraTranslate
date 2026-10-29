---
title: 'Thanh tiêu đề 42px và ba nút cửa sổ macOS'
type: 'refactor'
ticket: '3'
created: '2026-10-29'
status: 'built'
baseline_revision: '968606dfcbd193616234f51b986aa30f27ecc672'
route: 'oneshot'
route_source: 'auto'
risk: 'medium'
review: 'quick'
review_source: 'pinned'
lenses_ran: ['quick']
review_loop_iteration: 0
context:
  - '{project-root}/e2e/AGENTS.md'
---

<frozen-after-approval reason="human-owned intent — do not modify unless human renegotiates">

## Intent

**Problem:** Thanh tiêu đề đã lên 42px (entry 1) nhưng `trafficLightPosition` vẫn `{x:16, y:13}` của thanh 40px. Đo Accessibility trên máy Ice (macOS 15.8, debug build `HEAD 968606df`): khung nút 16×16, đỉnh cách mép trên cửa sổ 7px, tâm ở 15px, trong khi giữa thanh là 21px; nút đóng cách mép trái 15px, bước 20px (AD-58 mục 6, R8).

**Approach:** Đổi `y` trong `src-tauri/tauri.conf.json` cho tâm nút về 21px ±0,5 theo số đo lại (tao đặt khung chứa nút cao `16 + y`, nên ước tính `y` = 19), đo lại để xác nhận; giữ `x` = 16 và `traffic-light-inset` 91px trừ khi số đo cho khoảng từ mép phải nút phóng to tới 91px không còn là bội 7 ≥ 14. Thêm một spec e2e đo `.titlebar` vẽ ra cao 42px, đỉnh 0, lề trái 91px trên macOS. Cập nhật mục nợ cũ ghi số `{x:16,y:13}`/88px bằng một dòng 🔵.

</frozen-after-approval>

## Tasks

- `src-tauri/tauri.conf.json:24` -- `trafficLightPosition.y` theo số đo; `x` chỉ đổi theo luật ở Approach.
- `e2e/specs/story-13-3-thanh-tieu-de-42px.e2e.mjs` (mới, khuôn của `story-13-1-luoi-7px.e2e.mjs`) -- một ca: `getBoundingClientRect()` của `.titlebar` có `top` 0 và `height` 42 ±0,5; trên `data-platform="mac"` `paddingLeft` là `91px`.
- `_bmad-output/initiative-auratranslate/deferred-work.md` -- sau mục "Chưa có ai nhìn thanh tiêu đề macOS trong app thật" (`Chủ: Epic 12`, dòng ~13472) nối một dòng `→ 🔵 2026-10-29 (Story 13-3)`: số mới của `trafficLightPosition` và lề 91px; phần nhìn ba nút nằm ở mục `Chủ: Epic 13`. Không đổi chủ, không đóng mục.
- Đo (script ở scratchpad, không commit): chạy `src-tauri/target/debug/auratranslate`, đọc `{position, size}` của cửa sổ và ba `buttons` qua `osascript`/System Events theo `unix id`, rồi tắt app. Đối chứng bằng một `screencapture -R` vùng góc trên trái nếu quyền Ghi màn hình cho phép (đo mép vòng tròn vẽ ra và nét viền dưới của thanh); bị chặn thì ghi lại, không đoán.

## Acceptance

- Given debug build sau khi đổi, when đọc khung ba nút qua Accessibility, then tâm theo trục dọc cách mép trên cửa sổ 21px ±0,5 và ba nút cùng hàng.
- Given spec mới chạy ngoài sandbox, when mở app, then pass; `titlebar-tools` và `cargo test --test config_invariants` vẫn xanh.
- Given Ice mở app thật, when nhìn góc trên trái, then ba nút nằm giữa thanh (mục nợ `Chủ: Epic 13`, không đánh dấu qua bằng suy luận).

## Implementation Notes

- Route oneshot: một dòng cấu hình, một spec e2e ~30 dòng, một dòng sổ nợ.
- Số đo ban đầu (Accessibility, trước khi đổi): cửa sổ (152,44) 1280×860; close (167,51), minimize (187,51), zoom (207,51), mỗi khung 16×16.
- `y` 13 → 19 đúng như ước tính: đỉnh khung nút = `y` − 6, tâm = `y` + 2 (khung 16, macOS 15.8). Đo lại (`x` 17): cửa sổ (2074,154); close (2090,167), minimize (2110,167), zoom (2130,167) ⇒ ba nút cùng hàng, đỉnh khung 13, tâm 21.
- Ảnh chụp `screencapture -R` (màn hình ngoài, dpr 1) cho số thật của vòng nút vẽ ra: vòng 12px nằm giữa khung 16px, hàng 15–26, viền dưới thanh ở hàng 41 ⇒ 15px trên, 15px dưới tới mép 42.
- `x` 16 → 17 theo luật ở Approach: với `x` = 16, vòng nút phóng to ở cột 57–68 (tính cả hai đầu), nội dung bắt đầu ở cột 91 ⇒ khoảng 22px (cột 69–90), lệch lưới nên không là phương án; `x` = 17 cho 21px (vòng ở cột 18–29, 38–49, 58–69).
- Khung Accessibility lệch 1px so với `x` cấu hình (`x` 17 ⇒ khung ở 16); số chốt theo ảnh chụp, không theo khung.
- Đối chứng đỏ: gỡ `height: var(--space-titlebar-height)` khỏi `.titlebar` trong `App.vue` ⇒ spec mới đỏ, cao lệch 12px (30 thay 42); trả lại thì xanh.
- `tokens.json` deviation `traffic-light-inset`: lý do nay nêu khoảng 21px với `x` = 17 thay câu "Ice đo lại"; `question` trỏ mục nợ `Chủ: Epic 13`.
- e2e ngoài sandbox sau khi đổi (build `--features wdio`): `story-13-3` 1/1 và `titlebar-tools` 7/7 xanh.
- Chạy đủ bộ vì `tauri.conf.json` là dây dùng chung: vitest 242 tệp/3660 ca xanh; `cargo test --no-fail-fast` 103 binary, 2784 qua, 1 đỏ (`naming_boundary`, `origin` trần ở `ReadingMode.vue:251` từ `02305186`, có trước diff này; ghi nợ `Chủ: Ice`); `config_invariants` 50/50 xanh.

## Plan Change Log

## Review Triage Log

Lượt 1 (quick): 7 phát hiện, low 7 · patch 6 · reject 1.

- low · patch — ghi chú `x` = 16 nói vòng nút phóng to "hết ở cột 69": ảnh đầu cho cột 57–68 tính cả hai đầu; khoảng 22px vẫn đúng.
- low · patch — phương án `x` = 16 ghi "để Ice so" mà lệch lưới theo luật ở Approach: bỏ, không phải phương án.
- low · patch — AC1 thiếu khung ba nút sau khi đổi: ghi số lượt Accessibility thứ hai (cả ba ở y 167).
- low · patch — AC2 chưa có bằng chứng: ghi lượt e2e; `config_invariants` nằm trong lượt chạy đủ bộ ở Verification.
- low · patch — `question` của deviation `traffic-light-inset` còn trỏ `Chủ: Epic 12`: trỏ sang `Chủ: Epic 13`.
- low · reject — spec ghim `platform === 'mac'` vô điều kiện: theo tiền lệ `titlebar-tools.e2e.mjs:87`, nửa Windows chưa từng chạy, và ghim như vậy bắt được bộ dò nền tảng hỏng trên macOS.
- low · patch — đầu spec dẫn AD-58 Rule 6 cho lề 91px, mà lề do Rule 5 định nghĩa: dẫn Rules 5–6.

## Verification

**Commands:**
- `cargo test --test config_invariants` (sau `npm run build`) -- expected: pass.
- `npx wdio run e2e/wdio.conf.mjs --spec e2e/specs/story-13-3-thanh-tieu-de-42px.e2e.mjs` rồi `titlebar-tools`, ngoài sandbox, build `--features wdio` -- expected: pass.

**Manual checks:**
- Tâm ba nút 21px ±0,5 theo số đo Accessibility sau khi đổi; ghi số vào commit message.
