---
title: 'Nightly e2e xanh lại'
type: 'bugfix'
ticket: ''
created: '2026-11-01'
status: 'built'
baseline_revision: 'dcec4937ffa8545b01ca4148c08b77531fa49307'
route: 'full'
route_source: 'auto'
risk: 'medium'
review: 'quick'
review_source: 'pinned'
lenses_ran: ['quick']
review_loop_iteration: 0
context: []
---

<frozen-after-approval reason="human-owned intent — do not modify unless human renegotiates">

## Intent

**Problem:** Nightly e2e đỏ y hệt ở run 41134334441 và 41241138850: 13 ca trong 10 spec, xanh cuối `97bcb587`. Nên mọi story trước `done` đều vướng một cổng đỏ, và lỗi thật lẫn trong lỗi spec cũ.

**Approach:** Sửa từng ca theo nguyên nhân đã truy: spec cũ theo mã mà Ice đã chốt, spec sai thì sửa dữ liệu, lỗi sản phẩm thì sửa sản phẩm. Không nới dung sai, không bỏ ca để lấy màu xanh.

## Boundaries & Constraints

**Decision (Ice chọn 2026-11-02):** `13-7` phát line-height bằng px thay vì tỉ số, không đo trên runner trước. Ca này chỉ đỏ trên macos-26, máy này và Playwright WebKit không tái hiện, nên bằng chứng cuối là nightly.

**Always:** Giữ ý định của từng ca; chỉ đổi giá trị đã lỗi thời. Spec đọc giá trị từ nguồn sản phẩm khi được (`AI_PROVIDERS.length` không đọc được từ spec thì ghi số 9 kèm lý do trong commit). Ca nào xanh chỉ khi đã chạy thật trên WKWebView.

**Never:** Hạ dung sai `TOLERANCE_PX` của `12-14`, `13-7`, `13-4`; `skip` hay xoá ca; `continue-on-error`, retry; đổi `::-webkit-scrollbar`; đổi quyết định đã chốt (popup TM, lưới 238, chevron hạ 3px).

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected Output / Behavior | Error Handling |
|----------|--------------|---------------------------|----------------|
| `12-14` gương | Thân prompt dài, hộp đã cuộn | `clientWidth` textarea và gương lệch ≤ 1, ngắt dòng trùng nhau | Lỗi sản phẩm, sửa CSS |
| `12-18` nhà cung cấp | Mở mục AI | 9 `[data-ai-provider]` | Không |
| `12-2` lưới | Mọi cỡ cửa sổ | Mỗi thẻ ≥ 238 px, tên ca sửa theo | Không |
| `12-34` gieo | 8 câu, tách 4 lần | `[1, 1, 1, 1, 4]`, cả 3 ca chạy | Không |
| `12-7` thanh trạng thái | Tra cứu chưa rút | Chữ `TM n · p%`, `title` là câu đầy đủ | Không |
| `12-8` nhãn nguồn | Nhãn không còn `▾` | Đọc được `Nguồn n/m`, các bước sau chạy tiếp | Ca sau lộ lệch khác thì truy tiếp |
| `12-9` dải TM | Dải là popup phủ | Sát dưới ô, phủ lên hàng kế, hàng kế không bị đẩy | Không |
| `13-4` chevron | Cạnh chữ | Đáy chevron thấp hơn baseline 3 px ± dung sai | Không |
| `5-11` tra cứu | Từ có trong fixture | Popup có kết quả thật ở cả fixture và từ điển thật | Không |

</frozen-after-approval>

## Code Map

- `e2e/specs/story-12-14-prompt-library.e2e.mjs`:70 -- assert đúng. Lệch 5 ở máy này, 7 trên CI. `src/PromptLibraryOverlay.vue`:1188 gương `overflow: hidden` + `scrollbar-gutter: stable`, :1205 textarea; `::-webkit-scrollbar` 10px ở `src/tokens/controls.css`:729 (233ba4ec) có sau cùng lượt 3471d47e; chưa quy được 5 và 7, đo trước khi sửa.
- `e2e/specs/story-12-18-settings.e2e.mjs`:111 -- `toBe(5)`; `AI_PROVIDERS` có 9 mục (`src/aiConfigState.ts`:30, f88530bf).
- `e2e/specs/story-12-2-library-top-grid-overlay.e2e.mjs`:146 -- `>= 239`; `src/modes/LibraryMode.vue`:1137 `minmax(238px, 1fr)` (dd428222).
- `e2e/specs/story-12-34-work-page-read-chapter.e2e.mjs`:12 -- `SOURCE_TEXT` 5 câu, phải là 8 câu như `story-12-4-work-page-chapter-list.e2e.mjs`:14. Lỗi `pointerMove` là hệ quả.
- `e2e/specs/story-12-7-workspace-grid-status-bar.e2e.mjs`:131 -- `src/StatusBar.vue`:226 `status.tm_short`, câu đầy đủ ở `:title`.
- `e2e/specs/story-12-8-lookup-sources-popup.e2e.mjs`:24 -- bỏ ` ▾` khỏi regex (32d45b3b đổi sang SVG).
- `e2e/specs/story-12-9-strip-row-track.e2e.mjs`:128 -- `gapAboveNext` thuộc tiền đề track hàng, 7189c704 đổi thành popup phủ.
- `e2e/specs/story-13-4-canh-baseline.e2e.mjs`:52 -- `DESIGN.md`:455 🔵 2026-10-30 Ice hạ chevron 3px (e2331cd3).
- `e2e/specs/story-5-11-reading-mode.e2e.mjs`:34 -- `控制` không có trong `tools/dict-build/tests/fixtures/raw`; `皇子` có. Đổi `SOURCE_TEXT` và `LOOKUP_WORD`, không thêm fixture vì test `dict-build` đọc bộ đó.
- `src/modes/readingState.ts`:742, `src/modes/ReadingMode.vue`:762, :1124, :1142 -- đường line-height đọc.

## Tasks & Acceptance

**Pha (mỗi pha một agent mới):** Pha 1 là bảy spec cũ. Pha 2 là `12-14` và `13-7`. Pha 3 là chạy trọn e2e cục bộ.

**Execution:**
- [x] `e2e/specs/story-12-18-settings.e2e.mjs`, `story-12-2-library-top-grid-overlay.e2e.mjs`, `story-12-7-workspace-grid-status-bar.e2e.mjs`, `story-12-8-lookup-sources-popup.e2e.mjs`, `story-13-4-canh-baseline.e2e.mjs` -- đổi giá trị theo Matrix, sửa tên ca sai -- spec nói đúng sản phẩm.
- [x] `e2e/specs/story-12-34-work-page-read-chapter.e2e.mjs` -- văn bản 8 câu -- gieo đúng hình dạng.
- [x] `e2e/specs/story-12-9-strip-row-track.e2e.mjs` -- thay `gapAboveNext` bằng khẳng định dải phủ lên hàng kế, sửa tên ca và `describe` -- theo popup.
- [x] `e2e/specs/story-5-11-reading-mode.e2e.mjs` -- dùng từ có trong fixture -- ca chạy được ở CI.
- [x] `src/PromptLibraryOverlay.vue` -- đo `clientWidth`, `offsetWidth`, viền, đệm, gutter của cả hai; sửa CSS để hai hộp ngắt dòng như nhau.
- [x] `src/modes/readingState.ts`, `src/modes/ReadingMode.vue` -- `readingStyle.lineHeight` thành chuỗi px; `:1124` dùng thẳng biến thay vì `calc` nhân tỉ số; test `readingStyle` theo đó; tỉ số lưu, sàn 1.66 và bộ chỉnh không đổi.
- [x] `_bmad-output/initiative-auratranslate/deferred-work.md` -- ghi lý do `13-7` nếu chưa chứng minh được trên runner.

**Acceptance Criteria:**
- Given cây sau sửa, when chạy `npm run test:e2e` trên máy này, then mọi spec xanh, hoặc ca đỏ được ghi nguyên nhân.
- Given `12-14` trước sửa CSS, when gỡ bản sửa, then ca đỏ đúng vì lệch `clientWidth`.
- Given Prompt Library với dòng dài sát mép, when gõ tới chỗ ngắt dòng, then chữ tô màu và caret cùng hàng.
- Given nhánh đã push, when chạy `workflow_dispatch`, then job e2e xanh; đây là bằng chứng duy nhất cho `5-11` và `13-7`.

## Implementation Notes

- Trọn bộ e2e cục bộ hai lần: 67/69 spec, mọi ca của nightly đã xanh. Hai ca đỏ cố định khi không chạy đầu tiên: `panel-toggle-mouse` và `story-13-6`, đều không đỏ ở nightly; tái hiện bằng cặp spec, nguyên nhân ở sổ nợ.
- Gương `12-14`: `overflow: hidden` + `scrollbar-gutter: stable` giữ gutter gốc 15px, textarea dùng thanh cuộn 10px, lệch 5 px; `overflow: auto` với thumb trong suốt cho hai hộp 646. Spec thêm phép quét ký tự hẹp quanh điểm ngắt dòng; gỡ bản sửa thì lệch 21 px, tức đúng một dòng. Phép quét đầu bằng từ rộng 28 px không đỏ được, đã đổi.
- `12-9`: mép trái dải lệch ô 280 px đúng thiết kế popup rộng từ cột nguồn (7189c704), nên assert đổi thành dải phủ trọn ô.
- `--reading-line-height` có hai chỗ nhân tỉ số, `:1124` và `:1402`; plan chỉ nêu một, spec đỏ lộ chỗ thứ hai.
- Giả thuyết `13-7` do dpr bị bác: probe đọc dpr 1 trên chính máy này và `13-7` vẫn xanh, nên nghi WebKit của macOS 26. Bản sửa px chưa chứng minh được trên runner.

## Plan Change Log

## Review Triage Log

Lượt 1 (lens Quick): high 0 · medium 1 · low 3 · false 5 · maybe-false 0.

- medium, patch: AC ngắt dòng `12-14` chưa có ca canh; thêm phép quét vào spec, đối chứng đỏ 21 px.
- low, patch: mục nợ `13-7` thiếu `Cần:`; thêm, gộp `5-11`.
- low, patch: ca sàn 1.66 chép công thức; đổi thành `35px` như tên ca.
- low, reject: `::-webkit-scrollbar-thumb` riêng của gương; §Never nói về luật chung ở `controls.css`, luật này chỉ ẩn thumb của một hộp phụ.
- false: track lộ ra; `controls.css`:734 đã `transparent`.
- false: `::after` thêm dòng làm gương tràn riêng; phép quét đo `scrollHeight` hai hộp lệch ≤ 1.
- false: hai ca bảng mức đọc chép công thức là lỗi; đầu tệp quy định số lấy từ `tokens.json`, tính chất bội 7 có ca độc lập.
- false: comment `:1402`; vẫn đúng, nó giải thích vì sao không dùng `1lh`.
- false: `12-9` bỏ canh neo; assert mới vẫn đòi dải sát dưới ô và phủ trọn ô.

## Verification

**Commands:**
- `npx wdio run e2e/wdio.conf.mjs --spec <tệp>` từng spec khi đang vá -- xanh; ngoài sandbox, sau `cargo build --locked --features wdio`.
- `npm run test:e2e` một lần cuối -- HEAD đã đi trước lượt nightly gần nhất 22 commit nên có thể lộ ca mới.
- `npx vue-tsc --noEmit`, `npm run check:lint`, `npm run check:debt-owner` -- exit 0.
