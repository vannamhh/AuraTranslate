---
title: 'Cài đặt và Thư viện prompt theo khung chung'
type: 'refactor'
ticket: '2'
created: '2026-11-18'
status: built
baseline_revision: '82882eaa3270b3aee7de7c997886d833fd1a71ec'
route: 'full'
route_source: 'auto'
risk: 'medium'
review: 'quick'
review_source: 'pinned'
lenses_ran: ['quick']
review_loop_iteration: 0
context:
  - '{project-root}/src/AGENTS.md'
  - '{project-root}/e2e/AGENTS.md'
---

<frozen-after-approval reason="human-owned intent — do not modify unless human renegotiates">

## Intent

**Problem:** `SettingsOverlay.vue` (917px) và `PromptLibraryOverlay.vue` (959px) tự chép khung, cả khung lẫn cột cuộn (`.set-panel`/`.pl-panel` `overflow: auto` bọc `.pl-list` `overflow: auto` ⇒ cuộn lồng); ô chữ trong Cài đặt đo bằng `ch` (Mô hình, Người dịch, tên bố cục 24ch) dù luật lớp phủ đòi giãn.

**Approach:** Cả hai mang lớp khung `.ui-overlay-scrim`/`.ui-overlay`/`__head` của entry 1 cạnh lớp riêng. Cài đặt: đầu + thanh Phạm vi dính, lưới hai cột lấp phần còn lại, cột nội dung `.set-main` là `ui-overlay__body` (vùng cuộn duy nhất của khung). Thư viện prompt: đầu + tab loại + thanh phạm vi + khối chọn Proofreader dính, hai cột `.pl-list` và `.pl-ed` kề nhau mỗi cột tự cuộn. Ô chữ trong hai màn giãn hết cột; ô số giữ `ui-field--short`; ô chọn (`SelectMenu`) giữ bề rộng nhãn dài nhất. Xoá hai dòng nợ ở Kiểm L.

## Boundaries & Constraints

**Always:** Giữ nội dung, nhánh trạng thái, Esc, `trapTab`, tiêu điểm trả về nút mở, thứ tự Tab, z-index 10/11 ở scrim riêng. Giữ mọi lớp scoped hiện có (`set-panel`, `pl-panel`, `pl-input`…) vì test tham chiếu chúng. Cột kề nhau tự cuộn khai `overflow-y: auto` kèm miễn trừ một dòng `aura-allow-scroll: <lý do>` của Kiểm L. Thanh cuộn của cột phải sát viền phải khung.

**Never:** Sửa `controls.css`, `tokens.json`, `e2e/support/overlayFrame.mjs`, logic Kiểm L (chỉ xoá hai dòng nợ); chuyển lớp phủ khác; đổi chiều cao khung; đổi ô số sang giãn; sửa `e2e/specs/story-13-9-*`.

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected Output / Behavior | Error Handling |
|---|---|---|---|
| Cài đặt rộng | cửa sổ 1280, mục AI và mô hình, tầng Toàn cục | khung 1120 ±1; Endpoint, Mô hình, khoá API rộng = bề rộng nội dung `.set-main` (clientWidth − đệm trái/phải) ±1; ô Nhiệt độ (chip Tự nhập) = `round(up, 8ch, 7px)` ±0,5 | — |
| Cài đặt cuộn | mục dài (Phím tắt) | chỉ `.set-main` cuộn; đầu, thanh Phạm vi, cột điều hướng đứng yên; `nested` = 0 | — |
| Cửa sổ thấp | cột điều hướng dài hơn lưới | cột điều hướng tự cuộn, kề `.set-main`, không lồng | — |
| Thư viện prompt | cửa sổ 1280, kiểu Dịch và Proofreader | khung 1120 ±1; vùng cuộn đúng `.pl-list` + `.pl-ed`, `nested` = 0 | — |
| Thư viện lỗi tải | `promptSetLoadError` | đầu + tab + câu lỗi, không cột | — |

</frozen-after-approval>

## Code Map

- `src/SettingsOverlay.vue:335-352` scrim `.set-scrim` > `section.set-panel` > `header.set-head`; `:356` `.set-scope` (thanh Phạm vi); `:403` `.set-grid` > `nav.set-nav` + `.set-main`. CSS `:933-1002`: `.set-scrim` chép scrim chung; `.set-panel` max-width 917 + `overflow:auto`; `.set-grid` thiếu hàng `minmax(0, 1fr)` nên chưa cuộn được trong cột.
- `src/SettingsOverlay.vue:526-528` Endpoint `--full`, Mô hình `--medium` (cùng vòng `AI_TEXT_FIELDS`) ⇒ cả hai `--full`; `:597` ô chip Nhiệt độ/Số token `--short` (giữ, chỉ hiện sau khi bấm `[data-ai-chip="custom"]`, bấm không ghi gì); `:643` System prompt và `:689` khoá API đã `--full`.
- Ô chữ ở mục con: `SettingsExportSection.vue:35` Người dịch `--medium`; `SettingsLayoutSection.vue:98,116` tên bố cục `--medium` (hàng có nút) ; `SettingsGenreSection.vue:43,76` `.sg-input` `min-width: 280px` không lớp bề rộng; `SettingsShortcutsSection.vue:154` đã `flex: 1 1 auto`; `SettingsQuickAiSection.vue` lưới đã giãn. Ô số giữ: `SettingsGlossarySection.vue:38`, `SettingsTmSection.vue:38`, `SettingsDataSection.vue:137`.
- `src/PromptLibraryOverlay.vue:733-760` scrim `.pl-scrim` (+`.pl-scrim-above` z 11) > `section.pl-panel` > `header.pl-head`; `.pl-kind`, `.pl-bar`, các `p` báo, `section.pl-choice` (chỉ Proofreader), `.pl-cols` > `nav.pl-list` + `div.pl-ed` là con trực tiếp của khung (`<template v-else>` không sinh phần tử). CSS `:1115-1205`: `.pl-panel` max-width 959 + `overflow:auto` + padding 13; `.pl-list` `overflow:auto` (lồng). Ô tên `:974,1022` `--medium` ⇒ `--full`; `SelectMenu` `:824,840` giữ `--medium`.
- `scripts/check-tokens.mjs:2419-2421` `OVERLAY_DEBT` — xoá `SettingsOverlay`, `PromptLibraryOverlay`; `exemptAt(p, d.index, 'scroll')` là miễn trừ một dòng hiện có.
- `src/PromptImportOverlay.vue:108-134` mẫu đã chuyển: lớp riêng + lớp khung trên cùng phần tử, head scoped giữ chiều cao riêng.
- `e2e/support/overlayFrame.mjs` `resizeLogical`, `measureOverlayFrame(selector)` (bỏ qua `<textarea>`); `e2e/specs/story-17-1-khung-lop-phu.e2e.mjs` khuôn spec; `e2e/specs/story-12-18-settings.e2e.mjs:13-25` đường mở Cài đặt (`[data-settings-open]` → `[data-tools-item="settings.open"]`); `story-12-14-prompt-library.e2e.mjs` đường mở Thư viện (`[data-ai-prompt-inspector-open]` → `[data-prompt-library-open]`).
- `e2e/specs/story-13-2-be-rong-ch.e2e.mjs:50-56` đo `.set-panel .ui-field--medium` ở mục Xuất — sẽ hết phần tử; chuyển sang `.set-panel .ui-field--short` ở mục Glossary, giữ ý "ô `ch` thật là bội 7".

## Tasks & Acceptance

**Execution:**
- [ ] `src/SettingsOverlay.vue` -- thêm lớp khung; xoá CSS khung riêng (scrim trừ z-index, `.set-panel` trừ lớp, quy tắc mac); `.set-scope` `flex: none`; `.set-grid` `flex: 1; min-height: 0; grid-template-rows: minmax(0, 1fr)`; `.set-main` thêm `ui-overlay__body`; `.set-nav` `overflow-y: auto` + miễn trừ; Mô hình `--full` -- R1/R3/R4.
- [ ] `src/SettingsExportSection.vue`, `src/SettingsLayoutSection.vue`, `src/SettingsGenreSection.vue` -- ô chữ giãn hết hàng (`--full` khi đứng riêng, `flex: 1` khi chung hàng với nút); bỏ `min-width: 280px` -- R3.
- [ ] `src/PromptLibraryOverlay.vue` -- thêm lớp khung; xoá CSS khung riêng; con không phải đầu/cột của khung `flex: none` + đệm ngang 13px; `.pl-cols` `flex: 1; min-height: 0; grid-template-rows: minmax(0, 1fr)`; `.pl-list` và `.pl-ed` `overflow-y: auto` + miễn trừ, đệm trong cột, `.pl-ed` không đệm phải ngoài; ô tên `--full` -- R1/R3/R4.
- [ ] `scripts/check-tokens.mjs` -- xoá hai dòng nợ.
- [ ] `e2e/specs/story-17-2-cai-dat-va-thu-vien-prompt.e2e.mjs` -- spec theo I/O Matrix dùng helper entry 1; mở ở cỡ mặc định rồi `resizeLogical(1280, 720)`.
- [ ] `e2e/specs/story-13-2-be-rong-ch.e2e.mjs` -- ca 50-56 đo `--short` ở mục Glossary.
- [ ] `tests/frontend/settingsFrameDom.test.ts`, `tests/frontend/promptLibraryOverlayRender.test.ts` -- assert lớp khung có mặt, `.set-main` mang `ui-overlay__body`, Mô hình và ô tên mang `--full`.

**Acceptance Criteria:**
- Given cây sau thay đổi, when chạy `check:tokens`, then Kiểm L xanh với "nợ 16".
- Given đối chứng gỡ thật (`.set-panel` lấy lại `overflow: auto`), when chạy Kiểm L và spec 17-2, then cả hai đỏ vì vùng cuộn lồng; khôi phục rồi `touch`.
- Given 12-18, 12-14 và 13-2 e2e, when chạy, then xanh.

## Design Notes

Cột kề nhau tự cuộn là luật của DESIGN.md §Lớp phủ — Một vùng cuộn ("mỗi cột tự cuộn, vì các cột đứng cạnh nhau chứ không lồng"; mockup mục 3 ghi chú 2 cho cột điều hướng). Kiểm L không biết quan hệ cột ở nguồn, nên mỗi khai cuộn của cột mang miễn trừ một dòng có lý do; DOM e2e là chỗ đo `nested`. Không đưa Thư viện prompt vào danh sách nhiều cột: danh sách đó miễn cả R1, mà Thư viện thuộc nhóm Màn 1120px.

## Verification

**Commands:**
- `npm run check:tokens` -- exit 0, Kiểm L "nợ 16".
- `npx vitest run tests/frontend/settingsFrame.test.ts tests/frontend/settingsFrameDom.test.ts tests/frontend/settingsOverlayAiConfigKeyRender.test.ts tests/frontend/settingsOverlayAiConfigTierRender.test.ts tests/frontend/promptLibraryOverlayRender.test.ts tests/frontend/promptLibraryProofreadCard.test.ts tests/frontend/dialogAccessibleName.test.ts tests/frontend/glossarySettings.test.ts` -- xanh.
- `npm run test:e2e -- --spec e2e/specs/story-17-2-cai-dat-va-thu-vien-prompt.e2e.mjs`, rồi lần lượt `story-12-18-settings`, `story-12-14-prompt-library`, `story-13-2-be-rong-ch` -- xanh, ngoài sandbox, bản `--features wdio`.
- `npm run check:lint` và `npx vue-tsc --noEmit` -- xanh.

## Implementation Notes

- Dòng trạng thái `.ai-field-status` nằm trong `.ai-field-form` chung hàng với ô, nên Endpoint/Mô hình hụt 91px so với cột; nay dòng đó xuống hàng riêng (`flex: 1 0 100%`). Ngoài danh sách tệp của plan, đổi bố cục chứ không đổi nội dung.
- Thư viện prompt: con trực tiếp của khung (trừ đầu và cột) lùi vào bằng `margin-inline: 13px`, không bằng đệm, để `.pl-bar` giữ đệm 14px riêng và dải nền cùng vạch `.pl-kind` nằm trong lề như trước.
- Ô chữ chung hàng với nút (Người dịch, tên bố cục, đổi tên thể loại, đổi tên bộ) giãn bằng `flex: 1` thay vì `--full`.
- `measureOverlayFrame` đếm `.menu-panel` của `SelectMenu` đang đóng là vùng cuộn (kiểu Proofreader ra 4); spec lọc nhãn `menu-panel`, mục nợ `Chủ: Story 17.3` đòi helper bỏ phần tử không có hộp trước lô 3–4.
- Hàng "Cửa sổ thấp" chỉ đo được cột điều hướng là vùng cuộn kề, không lồng: ở cửa sổ tối thiểu 800×600 danh sách 11 mục vẫn vừa lưới, nên không ép được tràn thật.
- Spec 17-2 mở Thư viện trước Cài đặt: sau khi đổi cỡ cửa sổ ở các ca Cài đặt, nút `[data-ai-prompt-inspector-open]` biến khỏi DOM.
- Đối chứng gỡ thật: trả `overflow: auto` cho `.set-panel` ⇒ Kiểm L đỏ "nested scroll region", ca "Settings scrolls only in .set-main" đỏ `nested` 2; khôi phục rồi `touch`.

## Plan Change Log

## Review Triage Log

Lượt 1 (quick) — high 0 · medium 0 · low 5 · false 3 · maybe-false 0.

- low · patch — `.pl-panel > :not(.pl-head, .pl-cols) { padding-inline }` đè đệm 14px của `.pl-bar` và kéo dải nền `.pl-bar` cùng vạch `.pl-kind` ra hết bề ngang khung, đổi hình không ai yêu cầu; chuyển sang `margin-inline`.
- low · patch — hàng "Cửa sổ thấp" chỉ kiểm `set-nav` có khai cuộn, không có tràn thật; spec ép khung thấp, đo cột điều hướng tràn và cuộn riêng.
- false — "ca Cài đặt không ở `settingsFrameDom.test.ts`": ca khung nằm ở `settingsOverlayAiConfigTierRender.test.ts` (tệp có mock `aiConfigGet`), đã chạy trong 139/139; các hàng Cài đặt của bảng I/O là hàng e2e.
- low · reject — ô đổi tên bộ mang cả `--full` lẫn `flex: 1`; `flex: 1` quyết định, `width: 100%` thừa nhưng vô hại, không ai gặp.
- low · reject — `.ai-field-form > .ai-field-status` chạm cả form System prompt (ô chip và form khoá không chứa dòng trạng thái, phần đó của phát hiện sai); dòng trạng thái xuống hàng dưới ô là nhất quán, không mất chữ.
- low · reject — các ca e2e 17-2 nối trạng thái nhau như `story-12-18`; wdio chạy `it` theo thứ tự, ca 3 chờ `[data-ai-chip-input]` tồn tại trước khi đo.
- false — "đối chứng và 'nợ 16' chỉ có trong lời": đối chứng gỡ thật không nằm trong diff theo AGENTS.md; `npm run check:tokens` in "nợ 16" ở lượt kiểm của bước 3.
- false — "13-2 có thể thiếu ô `--short` ở mục Glossary": `SettingsGlossarySection.vue:38` không có `v-if`, và 13-2 đã chạy xanh.
