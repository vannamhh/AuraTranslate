---
title: 'Bề rộng `ch` theo lưới 7px'
type: 'refactor'
ticket: '2'
created: '2026-10-29'
status: 'built'
baseline_revision: 'dd428222815032ca64992e737d8fa16d50048bf3'
route: 'full'
route_source: 'auto'
risk: 'medium'
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

**Problem:** Bề rộng đo bằng `ch` (ô nhập 8ch · 24ch, thước đọc 62/68/76ch, các `min-width`/`max-width` theo `ch`) ra số px lẻ, lệch lưới 7px; Kiểm K hiện cho `ch` qua không kiểm (DESIGN.md §Lưới 7px — Bề rộng).

**Approach:** Đo `round(up, Nch, 7px)` trên WKWebView trước. Hỗ trợ thì mọi bề rộng `ch` ở thuộc tính kích thước đổi sang `round(up, Nch, 7px)` và Kiểm K đỏ với `ch` trần; thước đọc chuyển phép làm tròn vào CSS để cổng thấy nó.

## Boundaries & Constraints

**Always:** Giữ nghĩa số ký tự: `ch` vẫn tính theo cỡ chữ của chính phần tử mang nó (thước đọc: `.column`). Token `read-measure-*` trong `tokens.json` giữ `62ch`/`68ch`/`76ch`. Bước đầu là spec e2e đo `round()`; nếu WKWebView không hỗ trợ (`CSS.supports` sai hoặc bề rộng đo được không phải bội 7) thì dừng, không sửa CSS, trình Ice hai đường thay thế kèm số đo. Ca e2e ghim số ký tự mỗi dòng là ca di chuyển.

**Never:** Đổi `padding-inline-start` theo `ch` của `LookupRecord.vue:352` (thụt treo canh chữ số, không phải bề rộng). Thêm thuộc tính mới vào `SIZE_PROPS` hay mở cổng sang biến tuỳ chỉnh. Đổi khoảng 7/14 hay chân lớp phủ 112px (entry 6–8). Kiểm WebView2 (mục `B7` đã có trong `deferred-work.md`).

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected Output / Behavior | Error Handling |
|----------|--------------|---------------------------|----------------|
| `ch` trần | `width: 24ch` · `min-width: 12ch` · `minmax(12ch, 1fr)` | đỏ | gợi ý `round(up, 24ch, 7px)` |
| Làm tròn đúng | `width: round(up, 24ch, 7px)` · `round(up, var(--reading-measure), 7px)` | xanh | — |
| Làm tròn sai | `round(24ch, 7px)` (mặc định gần nhất) · `round(down, 24ch, 7px)` · `round(up, 24ch, 4px)` | đỏ | — |
| Không phải `ch` | `50% 1fr 100vh 8.5em auto fit-content var(--x)` | xanh (như cũ) | — |

</frozen-after-approval>

## Code Map

- `scripts/check-tokens.mjs:2186` `offGridSizePx` / `:2214` `declOffGrid` -- thêm nhánh `ch` cho `SIZE_PROPS`: bỏ mọi đoạn `round(up, …, 7px)` (ngoặc cân bằng), rồi mọi số `ch` còn lại là lệch. Thông báo `fail` ở vòng quét (`:2234`) cần lời gợi ý riêng cho `ch`.
- `scripts/check-tokens.mjs:2281` `K_SIZE_CASES` -- ca `'% fr vh ch em …'` đang chứa `24ch` và mong xanh: bỏ `24ch` khỏi ca đó, thêm các ca của Matrix. Ca đi qua `declOffGrid`, không gọi thẳng hàm con.
- `src/tokens/controls.css:225` `8ch` · `:230` `24ch` · `:290` `min-width: 12ch`.
- `src/ImportPreviewOverlay.vue:2222,2250` `12ch` · `:2308,2455` `3ch` · `:2464` `20ch`; `src/BilingualImportPreviewOverlay.vue:1049` `3ch`; `src/StatusBar.vue:786` `max-width: 40ch`.
- `src/modes/ReadingMode.vue:749` -- `:style` đặt `width: readingStyle.measure` nội tuyến (cổng không quét nội tuyến). Đổi thành biến `--reading-measure` và `.column` (`:1106`) khai `width: round(up, var(--reading-measure), 7px)`; `var()` thay thế ở phần tử dùng nên `ch` vẫn tính trên `.column`. Sửa câu chú thích template `:740–742` (nói `width` đi thẳng).
- `src/modes/ReadingMode.vue:1077` `--source-note-inline: 30ch` -- biến tuỳ chỉnh, cổng không quét; vẫn đổi sang `round(up, 30ch, 7px)` vì là bề rộng (ghi chú song ngữ và rãnh trái dùng chung). `:1263` `max-width: 60ch`, `:1306` `72ch`.
- `src/modes/readingState.ts:733` `readingStyle.measure` -- không đổi; `tests/frontend/readingTypography.test.ts` (đòi `measure` kết thúc bằng `ch`) vẫn xanh.
- `e2e/specs/story-5-11-reading-mode.e2e.mjs:179–183` -- đòi `|bề rộng / 1ch − 68| < 0.5`; ở 17,5px cột 609,375 → 616px = 68,74ch nên đỏ. Ca di chuyển.
- `e2e/specs/story-13-1-luoi-7px.e2e.mjs` -- khuôn đo WKWebView của entry 1 (mở app, `browser.execute`, lọc phần tử cao > 0).
- `ui-field--medium`/`--short` dùng ở `SettingsTmSection.vue`, `SettingsExportSection.vue`, `SettingsGlossarySection.vue`, `SettingsLayoutSection.vue`, `PromptLibraryOverlay.vue`, `LibraryWorkPage.vue`.

## Tasks & Acceptance

**Execution:**
- [ ] `e2e/specs/story-13-2-be-rong-ch.e2e.mjs` -- trước mọi sửa CSS: `CSS.supports('width', 'round(up, 24ch, 7px)')` và một phần tử thử `width: round(up, 24ch, 7px)` ra bội 7 trên WKWebView -- điều kiện dừng của story.
- [ ] `scripts/check-tokens.mjs` -- nhánh `ch` của Kiểm K và ca tự kiểm của Matrix -- R6.
- [ ] `src/tokens/controls.css`, `ImportPreviewOverlay.vue`, `BilingualImportPreviewOverlay.vue`, `StatusBar.vue`, `ReadingMode.vue` -- mọi `Nch` ở thuộc tính kích thước thành `round(up, Nch, 7px)`; thước đọc qua `--reading-measure` -- R6.
- [ ] `e2e/specs/story-13-2-be-rong-ch.e2e.mjs` -- thêm ca: một ô `ui-field--medium` thật (Cài đặt) và `.column` ở ba mức đọc có bề rộng bội 7 (±0,01px), và `68 ≤ bề rộng / 1ch < 68 + 7 / 1ch` ở mức Cân.
- [ ] `e2e/specs/story-5-11-reading-mode.e2e.mjs` -- đổi phép ghim số ký tự sang khoảng nửa mở như trên, giữ phép so trước/sau khi đổi cỡ chữ với dung sai `7 / 1ch`.

**Acceptance Criteria:**
- Given cây sau story, when `npm run check:tokens`, then xanh và số ca tự kiểm Kiểm K tăng đúng số ca Matrix.
- Given đối chứng gỡ thật nhánh `ch` khỏi `declOffGrid`, when chạy lại `check:tokens`, then đỏ ở ca tự kiểm `width: 24ch`; given `controls.css` trả về `width: 24ch`, then đỏ ở đúng dòng đó.
- Given app chạy ngoài sandbox, when spec `story-13-2` và `story-5-11` chạy, then xanh.
- Given `src/**` trừ `selftest/`, when grep `[0-9]ch` ở thuộc tính kích thước, then mọi chỗ nằm trong `round(up, …, 7px)`.

## Implementation Notes

- Điều kiện dừng không kích hoạt: WKWebView (webkit 605.1.15, macOS) trả `CSS.supports('width','round(up, 24ch, 7px)')` = true; ở `ui-md` `24ch` = 155,0625px làm tròn ra 161px.
- Kiểm K: `offGridSizeCh` gỡ mọi đoạn `round(up, …, 7px)` (ngoặc cân bằng) rồi coi mọi số `ch` còn lại là lệch; `declOffGrid` gộp nó với `offGridSizePx`. Tự kiểm Kiểm K 26 → 34 ca. `round(up, 24ch, 4px)` đỏ nhờ nhánh `px` có sẵn (4px lệch lưới), không nhờ nhánh `ch`.
- Đối chứng gỡ thật: bỏ `offGridSizeCh` khỏi `declOffGrid` ⇒ 5 ca tự kiểm đỏ; trả `controls.css` về `width: 24ch` ⇒ đỏ đúng dòng đó.
- Thước đọc: `:style` đặt `--reading-measure` thay `width`; `.column` khai `width: round(up, var(--reading-measure), 7px)`. Token `read-measure-*` và `readingStyle.measure` không đổi.
- `--source-note-inline` (biến tuỳ chỉnh, cổng không quét) cũng đổi sang `round(up, 30ch, 7px)`.
- `story-5-11` và spec mới đòi số ký tự mỗi dòng trong [N, N + 7/1ch) ở mỗi mức (62/68/76), thay cho phép ghim ±0,5.
- Kiểm chứng: `check:tokens` exit 0; vitest đủ bộ 242 tệp/3660 ca exit 0, không Errors (trước lượt sửa review, lượt sửa chỉ đổi chú thích và e2e); ba tệp vitest của plan 63 ca xanh sau lượt sửa; hai spec e2e xanh ngoài sandbox sau lượt sửa. Chưa chạy `cargo test` (không đụng Rust).

## Plan Change Log

## Review Triage Log

Lượt 1 (quick): 6 phát hiện — medium 1 · low 4 · false 1 · patch 5.

- medium · patch — `story-13-2-be-rong-ch.e2e.mjs:85-95` ca ba mức đọc chỉ đòi bội 7, mà mức Cân là mặc định: phím mức không ăn thì cả ba lượt đo cùng một cột vẫn xanh; ca phải đòi số ký tự của từng mức trong [N, N + 7/1ch) với N = 62/68/76.
- low · patch — `story-13-2-be-rong-ch.e2e.mjs:17-20` `isMultipleOf7(0)` đúng, nên phần tử ẩn hoặc cột sụp vẫn xanh; thêm điều kiện bề rộng > 0.
- low · patch — `story-5-11-reading-mode.e2e.mjs:163-178` chú thích còn khai cột 609,375px và "bề rộng / 1ch giữ nguyên", nay sai (616px, lệch tới 7/1ch); thay chú thích, không thêm câu dưới câu cũ.
- low · patch — chú thích mới hoặc viết lại bằng tiếng Việt, kể lịch sử hoặc nhắc lại dòng kế (`story-5-11` dòng mới, `ReadingMode.vue:740-742`, JSDoc `offGridSizeCh` và `SIZE_PROPS`), trái AGENTS.md §Code comments.
- low · patch — `ReadingMode.vue:1112-1113` vẫn ghi "hai biến CSS đặt ở `:style`", nay là ba (`--reading-measure`).
- false — ca Cài đặt có thể đo ô `ui-field--medium` của mục AI (`SettingsOverlay.vue:524`): ô đó cũng là một `ui-field--medium` thật trong Cài đặt, đúng thứ AC đòi; phát hiện thứ hai (bề rộng > 0) chặn trường hợp ô đã ẩn.

## Design Notes

`round(up, A, B)` của CSS Values 4 làm tròn lên bội của B; WebKit có từ Safari 15.4, nên khả năng cao là hỗ trợ, nhưng story đo trước. Vì làm tròn lên, số ký tự mỗi dòng chỉ tăng, tối đa thêm `7px / 1ch` (dưới một ký tự ở 17,5px).

Ví dụ thước đọc:

```css
.column { width: round(up, var(--reading-measure), 7px); }
```

```js
:style="{ '--reading-measure': readingStyle.measure, ... }"
```

## Verification

**Commands:**
- `npm run check:tokens` -- expected: exit 0.
- `npx vitest run tests/frontend/readingTypography.test.ts tests/frontend/checkTokensScan.test.ts tests/frontend/controls-css.test.ts` trong vòng làm; một lượt `npx vitest run` đủ bộ trước khi báo xong vì story đổi `scripts/check-tokens.mjs` (AGENTS.md) -- expected: 0 failed, 0 Errors.
- `npm run test:e2e -- --spec e2e/specs/story-13-2-be-rong-ch.e2e.mjs` và `--spec e2e/specs/story-5-11-reading-mode.e2e.mjs` ngoài sandbox -- expected: pass.
