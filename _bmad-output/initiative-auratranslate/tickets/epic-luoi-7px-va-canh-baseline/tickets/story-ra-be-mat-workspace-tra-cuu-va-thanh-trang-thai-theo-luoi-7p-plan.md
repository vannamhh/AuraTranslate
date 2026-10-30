---
title: 'Rà bề mặt Workspace, Tra cứu và thanh trạng thái theo lưới 7px'
type: 'refactor'
ticket: '6'
created: '2026-10-30'
status: 'built'
baseline_revision: '0183a30fd6affa7f9ac8b64e31d938c5e5bd90c6'
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

**Problem:** Phép chuyển máy móc của entry 1 đưa cả 4px (trong nhóm) lẫn 8px (giữa nhóm) về 7, nên các dải gợi ý và thẻ AI mất ranh giới nhóm; khối có viền 1px hay vạch trái cộng bề dày nét vào đệm, nên chiều cao và mép chữ lệch bội 7 (DESIGN.md §Lưới 7px — Trong nhóm 7, giữa nhóm 14 · Nét kẻ nằm trong ô lưới · Vạch + thụt 2 + 12).

**Approach:** Một spec e2e đo các khối của mockup `luoi-7px.html` mục 1 và 3 trên WKWebView trước, rồi sửa CSS của khung quanh lưới Workspace, panel Tra cứu, Đề xuất AI, ba dải gợi ý và thanh trạng thái cho tới khi spec xanh.

## Boundaries & Constraints

**Always:** Khối có viền 1px trừ 1px ở đệm cùng phía; khối có vạch trái 2px/3px trừ bề dày vạch ở đệm trái để vạch + thụt = 14. Cột xếp dọc mà thang cũ tách nhóm bằng 8px (con nối dòng bằng 4px) lấy khoảng 14; khe ngang giữa các nút giữ 7. Hàng điều khiển một dòng đứng tự do khai `height: 28px`. Mỗi giá trị lệch lưới vì bù nét mang `/* aura-allow-spacing: <reason> */` cùng dòng, như `controls.css:299`.

**Never:** Chạm thân lưới (`GridBlock.vue`, `SourceHanViet.vue`, token `--space-grid-*`). Đổi cỡ chữ, màu, chiều cao điều khiển hay ba thanh cố định. Chạm Library, trang Tác phẩm, Chế độ đọc, Cài đặt, lớp phủ (entry 7–8). Thêm token `--space-*` chỉ để lách Kiểm K.

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected Output / Behavior | Error Handling |
|----------|--------------|---------------------------|----------------|
| Hàng công cụ | hàng đầu của panel Đề xuất AI (`.ai-panel-head`), Tra cứu (`.lookup-tabrow`), lưới khi Chương `zh` (`.tabs`) | cao 28, đỉnh cách đỉnh vùng nội dung panel 14 (±0,5px) | — |
| Dải TM | `.tm-fuzzy-strip` hiện dưới câu có gợi ý mờ | cao ngoài là bội 7; mép chữ cách mép ngoài 14 | — |
| Đầu mục · dải nguồn | tra một từ có kết quả | `.lookup-head` 42, `.lookup-source-head` 35 | 0 nguồn từ điển ⇒ `console.warn` + `this.skip()` |
| Khối bản ghi | một `.lookup-source` có ≥ 2 cụm | cao ngoài là bội 7 | như trên |
| Thanh trạng thái | `.status` | cao 35, khe giữa các mục là bội 7 | — |
| Bẫy | trả `.tm-fuzzy-strip` về `padding: 14px` | ca dải TM đỏ | — |

</frozen-after-approval>

## Code Map

- `e2e/specs/story-13-1-luoi-7px.e2e.mjs` -- khuôn: `openWorkspaceWithWork`, `waitForGridRows`, đo trong `browser.execute`. Gieo dải TM: `story-12-9-strip-row-track.e2e.mjs` (`invoke`, `.tm-fuzzy-strip`). Panel Tra cứu có nguồn: `story-12-8-lookup-sources-popup.e2e.mjs` (`list_dict_sources`, `ensureFullLayoutTier`, skip khi 0 nguồn).
- `src/TmFuzzyStrip.vue:266` `.tm-fuzzy-strip` -- `border` 1 + `border-left` 3 + `padding` 14 ⇒ 13 13 13 11; `gap` 7 (cũ 8, con `.tmf-list` cũ 4) ⇒ 14. `:325` `.tmf-row` viền 1 trong suốt + đệm 7 ⇒ 6.
- `src/GlossaryConfirmStrip.vue:300` `.gcs-done`, `:323` `.glossary-confirm-strip` -- `border-top` 1 + đệm 14 ⇒ đệm trên 13; `.glossary-confirm-strip` `gap` và `:341` `.gcs-fieldset` `gap` 7 (cũ 8, con `.gcs-field`/`.gcs-options` cũ 4) ⇒ 14. `:407` `.gcs-option` viền 1 + đệm 7 ⇒ 6.
- `src/ProofreaderStrip.vue:334` `.proofreader-strip` -- `border-top` 1, `border-left` 3, đệm 14 ⇒ trên 13, trái 11. Dùng ở `GridBlock.vue:362` (track) và `App.vue:320` (foot); chỉ sửa tệp strip.
- `src/panels/AiTranslationPanel.vue` -- `:818` `.ai-panel-head` (hàng công cụ, chưa khai cao); `:924` `.ai-result-card` viền 1 + đệm 14 ⇒ 13, `gap` 7 (cũ 8) ⇒ 14; `:935` `.ai-translate-usage` 7 + nét 1 ⇒ 6; `:940` `.ai-group-label` 14 + nét 1 ⇒ 13; `:1036` `.ai-promote-confirm` viền 1 + đệm 14 ⇒ 13; `:1058` `.ai-promote-draft` vạch 2 + đệm trái 14 ⇒ 12.
- `src/panels/LookupPanel.vue` -- `.lookup-head` `:913` đã 42; `:1004` `.lookup-tabrow` (hàng công cụ); `:1060` `.lookup-section` đệm dưới 7 + nét 1 ⇒ 6; `:1102` `.lookup-row` và `:1207` `.concordance-hit` vạch 2 + đệm trái 14 ⇒ 12; `:1201` `.lookup-hint` đệm trên 14 + nét 1 ⇒ 13.
- `src/panels/LookupRecord.vue` -- `:237` `.lookup-source-head` đã 35 (border-box, `reset.css:22`); `:339` `.lookup-entry + .lookup-entry .lookup-entry-head` đệm trên 14 + nét 1 ⇒ 13; `:407` `.lookup-citation` vạch 2 + đệm 14 ⇒ 12. Khoảng trong cụm 7 và giữa nghĩa 14 đã khớp mockup.
- `src/panels/GridPanel.vue:1994` `.tabs` -- hàng công cụ của lưới, `align-items: baseline`, chưa khai cao.
- `src/modes/WorkspaceMode.vue:169` `.narrow-notice` -- đệm 14 + `border-bottom` 1 ⇒ đệm dưới 13.
- `src/StatusBar.vue:735` `.status` -- 35px, `gap` 14 đã như mockup; chỉ đo.
- `scripts/check-tokens.mjs:2255` `declOffGrid` -- mọi `px` không bội 7 ở padding/margin/gap đỏ trừ khi có `aura-allow-spacing` cùng dòng; `var(--space-*)` luôn qua.

## Tasks & Acceptance

**Execution:**
- [ ] `e2e/specs/story-13-6-ra-be-mat-workspace.e2e.mjs` -- các ca của Matrix trước mọi sửa CSS; ghi lại ca nào đỏ trên cây gốc -- R1, R6.
- [ ] `src/TmFuzzyStrip.vue`, `src/GlossaryConfirmStrip.vue`, `src/ProofreaderStrip.vue` -- bù viền/vạch và khoảng giữa nhóm 14 theo Code Map -- R1, R6.
- [ ] `src/panels/AiTranslationPanel.vue`, `src/panels/LookupPanel.vue`, `src/panels/LookupRecord.vue`, `src/modes/WorkspaceMode.vue` -- như trên -- R1, R6.
- [ ] `src/panels/GridPanel.vue`, `AiTranslationPanel.vue`, `LookupPanel.vue` -- `height: 28px` cho ba hàng công cụ khi spec đo được hàng một dòng khác 28 -- R1.

**Acceptance Criteria:**
- Given app chạy ngoài sandbox, when spec `story-13-6` chạy, then xanh; given phép gỡ ở hàng Bẫy, then đỏ đúng ca dải TM.
- Given cây sau story, when `npm run check:tokens`, then xanh, và mỗi miễn trừ `aura-allow-spacing` mới nằm trên một khai báo bù đúng bề dày nét cùng phía (số miễn trừ mới ghi ở Implementation Notes).
- Given spec `story-13-1`, `story-13-4`, `story-12-9`, `story-12-8`, when chạy lại, then xanh.
- Given `git diff` của story, then không chạm `GridBlock.vue`, `SourceHanViet.vue` hay `tokens.json`.

## Implementation Notes

- Spec `story-13-6` on the unchanged tree: red only on the TM strip and the multi-entry `.lookup-source` block; toolbar rows (28, top 14), `.lookup-head` 42, `.lookup-source-head` 35 and `.status` 35 were already green, so no `height: 28px` was added (task 4 conditional not met).
- 18 new `aura-allow-spacing` exemptions, each on a declaration that subtracts a 1px border or 2px/3px rule from the 14px/7px inset; 8px/4px group gaps in TM strip, glossary strip and AI result card moved to 14.
- `story-12-9` (gapBelowCell 151 vs <= 2) and `story-12-8` (source label never reaches count; header 44 > 41) are red on the unchanged tree with the same values; not caused by this story.

## Plan Change Log

## Review Triage Log

Lượt 1 (quick): 7 phát hiện — medium 1 · false 6 · defer 1.

- medium · defer — AC chạy lại `story-12-9` và `story-12-8` không đạt: 12-9 `gapBelowCell` 151 (154 sau diff) > 2, 12-8 `.lookup-head` `scrollHeight` 44 > `clientHeight` 41 ở bề rộng 120px. Cả hai xanh ở nightly `97bcb587` (trước Epic 13) và đỏ trên cây gốc `0183a30f` theo lượt đo của agent triển khai; diff không chạm `.lookup-head` hay track hàng lưới. Ghi `deferred-work.md`, Chủ: Epic 13.
- false — các gap con `.tmf-list`, `.gcs-field`, `.gcs-options` "chưa đổi": Code Map ghi chúng là khoảng cũ 4 trong nhóm, nên giữ 7; chỉ cột cũ 8 tách nhóm lên 14, đúng như diff.
- false — Bẫy chưa đo: agent triển khai đã trả `.tm-fuzzy-strip` về `padding: 14px` và chỉ ca dải TM đỏ, rồi hoàn lại tệp.
- false — `.tmf-title` là đầu dò yếu: hàng Matrix đòi mép chữ của dải cách mép ngoài 14, và tiêu đề là chữ đầu tiên ở mép đó; chiều cao bội 7 của cả dải phủ phần `.tmf-row`.
- false — ca thanh trạng thái qua suông: `StatusBar.vue:536,538,565,647` có đủ `.status-line`, `.status-info`, `.status-end`.
- false — `.closest('.panel')` có thể null: `PanelFrame.vue:106` đặt `class="panel"`; spec chạy lại 7/7 xanh.
- false — không có vi phạm luật: 18 miễn trừ đều bù đúng bề dày nét cùng phía (người review tự kết luận).

## Design Notes

Bù nét theo mockup (`--edge: 1px`): `.tool { padding: 7px 14px 6px; border-bottom: 1px }` ⇒ 7 + 28 + 6 + 1 = 42. Ở app, panel đã có đệm 14 quanh nội dung, nên "hàng công cụ" là hàng đầu của thân panel: cao 28, cách mép 14, cách khối kế 14.

```css
.tm-fuzzy-strip {
  padding: 13px 13px 13px 11px; /* aura-allow-spacing: 1px border and 3px TM rule sit inside the 14px inset */
}
```

Miễn trừ thay vì token mới: thông báo của Kiểm K tự chỉ đường này cho phép bù nét, và một token `--space-*` 13px sẽ qua cổng ở mọi chỗ khác.

## Verification

**Commands:**
- `npm run check:tokens` -- expected: exit 0.
- `npm run test:e2e -- --spec e2e/specs/story-13-6-ra-be-mat-workspace.e2e.mjs`, rồi `story-13-1-luoi-7px`, `story-13-4-canh-baseline`, `story-12-9-strip-row-track`, `story-12-8-lookup-sources-popup`, ngoài sandbox -- expected: pass.
- `npx vitest run tests/frontend/controls-adoption.test.ts` -- expected: 0 failed, 0 Errors.

**Manual checks (if no CLI):**
- Không có; lượt nhìn app thật của Ice thuộc mục nợ `Chủ: Epic 13`.
