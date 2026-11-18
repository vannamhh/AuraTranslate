---
title: 'Khung lớp phủ dùng chung, cổng giữ luật, và màn Xem trước lượt nhập bộ prompt'
type: 'feature'
ticket: '1'
created: '2026-11-18'
status: 'built'
baseline_revision: '99b5b0c49d365e215729cbb2485d87b22e38c373'
route: 'full'
route_source: 'auto'
risk: 'high'
review: 'quick'
review_source: 'pinned'
lenses_ran: ['quick']
review_loop_iteration: 0
context:
  - '{project-root}/src/AGENTS.md'
  - '{project-root}/scripts/AGENTS.md'
  - '{project-root}/e2e/AGENTS.md'
---

<frozen-after-approval reason="human-owned intent — do not modify unless human renegotiates">

## Intent

**Problem:** 19 lớp phủ nhóm Màn tự chép khung (scrim + `section` + viền), bảy bề rộng khác nhau, tám tệp nghi cuộn lồng; `PromptImportOverlay.vue` rộng 721px, cả khung cuộn và `.pi-body` lại cuộn riêng, nút Xác nhận nhập trôi theo nội dung.

**Approach:** Token `spacing.overlay-max-width` 1120px (CSS `--space-overlay-max-width`, vì `tokens/index.ts` sinh `--space-<tên>`), một khung dùng chung đầu/thân/chân với thân là vùng cuộn dọc duy nhất, một cổng nguồn canh R1 + R4 với danh sách nợ 18 tệp còn lại, và Xem trước lượt nhập bộ prompt chuyển sang khung (mockup mục 2, giữ `dl` như mã).

**Decision (Ice 2026-11-18, sau khi đọc số đo):** khung là lớp CSS `.ui-overlay-scrim` · `.ui-overlay` · `.ui-overlay__head` · `__body` · `__foot` trong `controls.css`, không phải component Vue; mỗi lớp phủ giữ lớp scoped riêng cạnh lớp khung. Bác component `OverlayFrame.vue` (phải truyền lớp riêng qua prop để giữ 223 tham chiếu test; Esc/Tab vẫn ở từng tệp).

**Decision (Ice 2026-11-18):** cổng là Kiểm L trong `check:tokens`, dùng `tokens-scan.mjs` và `exemptAt`; không thêm cổng riêng, không đụng ba danh sách.

**Decision (Ice 2026-11-18):** e2e mở màn thật qua móc env `AURATRANSLATE_E2E_PICK_FILE` sau hai lớp chặn AD-45 (`debug_assertions` + `feature = "wdio"`) trong `prompt_set_open_import_preview`; `wdio.conf.mjs` cấp đường dẫn theo worker. Bác đầu dò kiểu 13.8 vì chỉ đo CSS, không đo component.

**Decision (Ice 2026-11-18):** giữ trọn phạm vi ticket dù plan vượt 1.600 token.

## Boundaries & Constraints

**Always:** Giữ nguyên nội dung, nhánh trạng thái, Esc đóng, `trapTab`, tiêu điểm trả về nút mở, thứ tự Tab của màn. Khung co theo cửa sổ: `min(1120px, cửa sổ − 28px)`. Mỗi lớp phủ giữ lớp scoped riêng (`pi-panel`…) cạnh lớp khung để 223 tham chiếu trong test không đổi. Cổng có tự kiểm đỏ/xanh đi qua cùng bộ lọc với assert thật; đối chứng gỡ thật (AGENTS.md).

**Never:** Chuyển lớp phủ thứ hai; sửa 18 tệp nợ; rút `focusableWithin`/`trapTab` về một chỗ (AI-11, chỉ khi Ice nói); đổi chiều cao lớp phủ; chạm menu `Công cụ ▾`; sửa `e2e/specs/story-13-9-*` (13.9 đang chờ duyệt).

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected Output / Behavior | Error Handling |
|---|---|---|---|
| Cửa sổ rộng | 1280px, thân prompt 40 dòng | khung 1120, một phần tử cuộn dọc, chân dính thấy ở đỉnh và đáy | — |
| Cửa sổ hẹp | 800px | khung 772 | — |
| Xung đột tầng | `kind === 'conflict'` | hai thân `dd.pi-mono` hiện trọn, vẫn một vùng cuộn | — |
| Lỗi tải / `ipc_unavailable` | không có xem trước | đầu + thân báo lỗi, không chân | nút Huỷ ở đầu |
| Cổng: lồng | tệp đã dùng khung khai `overflow-y: auto` | ĐỎ, lý do "vùng cuộn lồng" | — |
| Cổng: nợ cũ | tệp trong danh sách nợ đã dùng khung | ĐỎ, đòi xoá dòng nợ | — |

</frozen-after-approval>

## Code Map

- `src/PromptImportOverlay.vue:105-257,259-446` -- template scrim `.pi-scrim` > `section.pi-panel` (z 11, trên PromptLibrary 10); `.pi-panel` max-width 721 + `overflow:auto`; `.pi-body` max-height 196 + `overflow:auto` (lồng); `.pi-hint` + `.pi-actions` nằm trong thân ⇒ thành chân. Nhánh lỗi không có chân.
- `src/GlossaryQueueOverlay.vue` `.gq-actions` -- mẫu chân dính hiện có (`sticky; bottom:-13px`); khung mới thay mẹo này bằng flex cột.
- `src/tokens/tokens.json` `spacing` + `scripts/check-tokens.mjs:274` `EXPECTED_SPACING` (Kiểm A) -- thêm `overlay-max-width: 1120px` ở cả hai.
- `src/tokens/controls.css` -- nơi các lớp `.ui-*` dùng chung; chưa có lớp lớp phủ nào. `PanelFrame.vue` là component khung có slot duy nhất (17 nơi dùng).
- 18/19 màn có scrim giống hệt (`fixed; inset 0; flex; center; flex-start; padding panel-inline; background`) + quy tắc `:root[data-platform='mac'] … padding-top: titlebar-height`; khác: `ProofreadMemoryOverlay` (flex-end, 80vh). z-index: 13 tệp 10, 6 tệp 11 (Kiểm F đòi miễn trừ có tên, giữ ở từng tệp).
- `scripts/lib/tokens-scan.mjs` -- `maskCommentsAndStrings`, `parseCssBlocks`, `lineOf`: dùng lại cho cổng; `exemptAt` khuôn miễn trừ một dòng ở `check-tokens.mjs` (Kiểm D/F/H/K).
- `src-tauri/src/commands/promptset.rs:1113` `prompt_set_open_import_preview` -- mở hộp thoại HĐH bằng `blocking_pick_file`; WebDriver không lái được, `__TAURI_INTERNALS__.invoke` không ghi đè được (`defineProperty` chỉ `value`). Móc e2e kiểu AD-45: `src-tauri/src/lib.rs:76,100` + `e2e/wdio.conf.mjs:576-590` (env đặt vào `process.env` trước lúc spawn).
- `e2e/specs/story-13-9-chieu-cao-hang-va-icon.e2e.mjs:258` `resizeLogical` -- khuôn đổi cỡ cửa sổ theo dpr; chép ý vào helper mới, không sửa tệp này.
- `e2e/specs/story-12-14-prompt-library.e2e.mjs` -- đường mở Thư viện prompt (`[data-ai-prompt-inspector-open]` → `[data-prompt-library-open]`).
- `tests/frontend/promptImportOverlayRender.test.ts`, `tests/frontend/dialogAccessibleName.test.ts:140` -- giữ xanh, thêm assert khung.

## Tasks & Acceptance

**Execution:**
- [x] `src/tokens/tokens.json`, `scripts/check-tokens.mjs` (Kiểm A) -- thêm `overlay-max-width`.
- [x] `src/tokens/controls.css` -- lớp `.ui-overlay-scrim` (scrim chung + quy tắc mac `padding-top: titlebar-height`), `.ui-overlay` (`flex` cột, `max-width: var(--space-overlay-max-width)`, `max-height: 100%`, viền, nền, viền tiêu điểm theo Kiểm H), `__head`/`__foot` `flex: none`, `__body` `flex: 1; min-height: 0; overflow-y: auto`. z-index ở lại từng tệp với miễn trừ có tên.
- [x] `src/PromptImportOverlay.vue` -- thêm lớp khung cạnh `pi-scrim`/`pi-panel`/…; `.pi-body` bỏ `max-height`/`overflow`; `.pi-hint` + `.pi-actions` vào chân; xoá CSS khung riêng.
- [x] `scripts/check-tokens.mjs` -- Kiểm L: luật R1/R4 ở Design Notes, quần thể `src/*Overlay.vue` sàn 22, ba danh sách miễn trừ có tên, tự kiểm (một ca vùng cuộn lồng đỏ, một ca sạch xanh, một ca nợ cũ đỏ) đi qua cùng hàm với assert thật.
- [x] `src-tauri/src/commands/promptset.rs`, `src-tauri/src/lib.rs` -- hằng `E2E_PICK_FILE_ENV` + hàm đọc nó chỉ dưới `cfg(all(debug_assertions, feature = "wdio"))`, nhánh `not(...)` trả `None`; vỏ IPC dùng đường dẫn này thay hộp thoại khi có. Test Rust: nhánh phát hành không đọc env.
- [x] `e2e/wdio.conf.mjs` -- cấp `AURATRANSLATE_E2E_PICK_FILE` trong thư mục tạm của worker, cùng chỗ cặp env hiện có.
- [x] `e2e/support/overlayFrame.mjs` -- `resizeLogical`, `measureOverlayFrame(selector)` → `{ width, scrollers, nested }`, `isInViewport(el)`; entry 2–5 dùng lại.
- [x] `e2e/specs/story-17-1-khung-lop-phu.e2e.mjs` -- spec theo `verify` của ticket.
- [x] `tests/frontend/promptImportOverlayRender.test.ts` -- nút Xác nhận nhập nằm trong chân khung, không trong thân.

**Acceptance Criteria:**
- Given cửa sổ 1280px, when mở Xem trước lượt nhập bộ prompt với thân 40 dòng, then khung rộng 1120 ±1, đúng một phần tử cuộn dọc, nút Xác nhận nhập nằm trong khung nhìn khi thân cuộn ở đỉnh và ở đáy.
- Given cửa sổ 800px, when mở lại, then khung rộng 772 ±1.
- Given cổng chạy trên cây, when `PromptImportOverlay.vue` dùng khung, then xanh và danh sách nợ có đúng 18 tệp.
- Given đối chứng gỡ thật (`.pi-body` lấy lại `overflow: auto`), when chạy cổng và spec, then cả hai đỏ vì vùng cuộn lồng.

## Design Notes

Luật cổng (Notes của epic): tệp không trong danh sách nợ, không miễn trừ phải (R1) mang lớp `ui-overlay` trong template và không khai `width`/`max-width` trên các lớp riêng của phần tử khung; `.ui-overlay` trong `controls.css` chỉ lấy bề rộng từ `--space-overlay-max-width`. (R4) tệp đã dùng khung khai 0 vùng cuộn dọc riêng (`overflow`/`overflow-y` = `auto|scroll`), vì thân khung đã là vùng duy nhất; ô nhập nhiều dòng qua miễn trừ một dòng `aura-allow-scroll: <lý do>`. Popup (2) miễn cả hai luật; năm màn nhiều cột miễn R1, và miễn R4 ở cổng nguồn vì quan hệ lồng giữa các cột chỉ đo được trên DOM (helper e2e đo `nested`); ghi một mục nợ cho Attribution (`.attr-license-text`, nghi lồng) `Chủ: Epic 17`.

## Verification

**Commands:**
- `npm run check:tokens` -- exit 0, Kiểm L in tự kiểm và "nợ 18".
- `npm run build && cargo test --manifest-path src-tauri/Cargo.toml --lib promptset` -- xanh (nhánh phát hành không đọc env).
- `npx vitest run tests/frontend/promptImportOverlayRender.test.ts tests/frontend/dialogAccessibleName.test.ts` -- xanh.
- `npm run test:e2e -- --spec e2e/specs/story-17-1-khung-lop-phu.e2e.mjs` -- xanh, chạy ngoài sandbox, bản `--features wdio`.
- `npm run check:lint` và `npx vue-tsc --noEmit` -- xanh.

## Implementation Notes

- Khung không mang padding: đầu, thân, chân tự mang padding nên thanh cuộn của `__body` sát viền khung (DESIGN.md "sát mép khung"); đầu cao 35 nằm sát viền trên, không còn 13px phía trên như khung cũ. Vạch đầu/chân dùng `--color-outline` vì màu "faint" của mockup không phải token.
- Chân của Xem trước nhập là một hàng: dòng gợi ý `flex: 1` bên trái, hai nút bên phải; nhánh lỗi và `ipc_unavailable` không có chân, nút Huỷ ở đầu.
- Theo CSS Overflow 3, `overflow-x` khác `visible`/`clip` mà `overflow-y` để mặc định thì y tính thành `auto`; Kiểm L đếm trường hợp đó là vùng cuộn dọc, helper e2e đếm theo `getComputedStyle` và bỏ qua `<textarea>`.
- Ba danh sách của Kiểm L là Map tên → lý do; lý do nợ nêu entry sở hữu (17.2 · 17.3 · 17.4), lý do rỗng là đỏ.
- Móc `AURATRANSLATE_E2E_PICK_FILE` đi qua `absolute_dir_override_from_raw` (cùng luật đường dẫn tuyệt đối); `wdio.conf.mjs` ghi một prompt 40 dòng vào thư mục dữ liệu của worker. Không có ca Rust gọi `set_var`; `config_invariants` canh vị trí cfg của biến mới.
- Spec e2e mở màn ở cỡ cửa sổ mặc định rồi mới đổi cỡ: ở cao 720 nút mở Prompt inspector không tới được.
- Đối chứng gỡ thật (2026-11-18, sau bản vá review): trả `max-height: 196px; overflow: auto` cho `.pi-body` ⇒ Kiểm L đỏ "nested scroll region" và spec đỏ `scrollers` 2 thay vì 1; khôi phục rồi `touch`.
- Thông điệp lỗi và tên ca tự kiểm của Kiểm L viết tiếng Anh, khác các Kiểm khác của `check-tokens.mjs`.

## Plan Change Log

## Review Triage Log

Lượt 1 (quick) — high 0 · medium 6 · low 4 · false 1 · maybe-false 0.

- medium · patch — `controls.css` `.ui-overlay` mang padding 13 nên thanh cuộn của `__body` cách viền 13px, trái DESIGN.md "sát mép khung"; mockup mục 2 đặt padding ở đầu/thân/chân, đầu có vạch dưới, chân có vạch trên, nền và một hàng.
- low · patch — `.pi-foot { margin-top }` cộng với margin dưới của phần tử cuối trong thân thành 28px.
- medium · patch — Kiểm L R1 bỏ `ui-overlay` khỏi lớp cần soi, nên `.ui-overlay { max-width }` hay `section { max-width }` trong style scoped của tệp đã chuyển lọt qua.
- medium · patch — `overflow-x: auto` (y `visible`) tính ra `overflow-y: auto` theo CSS Overflow 3, nhưng cổng cho xanh (ca tự kiểm sai); helper e2e đếm cả `<textarea>` (UA `overflow: auto`) nên entry 2–4 đỏ oan.
- low · reject — `frameCssVerdict` không soi `overflow` trên `.ui-overlay`/scrim; entry 2–5 không sửa `controls.css` theo Notes của epic, sửa phải thêm nhánh.
- medium · patch — `OVERLAY_DEBT` và hai danh sách miễn trừ không kèm lý do (Notes của epic, AGENTS.md "carries its reason").
- medium · patch — `$('.ui-overlay__body')` không gắn vào `.pi-panel`; khi entry 2 chuyển Thư viện prompt (mở bên dưới) sang khung, `$()` có thể trúng thân của nó.
- low · patch — `resizeLogical` nhận lệch ±12px nhưng ca 800px so cứng 772 ±1; khung là `innerWidth − 28`.
- medium · patch — ca test `e2e_pick_file_release` gọi `unsafe set_var` trong binary test song song (edition 2024), đúng chỗ `lib.rs` doc của `absolute_dir_override_from_raw` tránh; ca chỉ kiểm một hằng `None`, `config_invariants` đã canh cfg.
- low · patch — chú thích mới trái luật chú thích (tiếng Việt, quá hai dòng, kể lại plan), khối doc thừa trên `parseSource`, doc `absolute_dir_override_from_raw` còn nói "hai móc".
- false — "hàng Xung đột tầng không có test": `promptImportOverlayRender.test.ts:143` vẽ cả hai thân; `.pi-mono` không `overflow`/`max-height`, Kiểm L xanh trên cây thật. Phần "Implementation Notes trống" là sửa plan, loại.
