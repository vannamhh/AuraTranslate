---
title: 'Panel Đề xuất AI gọn: chọn bộ prompt vào màn Prompt, ghi cách chọn câu, có lối bỏ vùng chọn'
type: 'feature'
ticket: ''
created: '2026-10-26'
status: 'built'
baseline_revision: 'f6f66826c8984c8de13be85efc85527cfd412f9d'
route: 'full'
route_source: 'auto'
risk: 'medium'
review: 'quick'
review_source: 'pinned'
lenses_ran: ['quick']
review_loop_iteration: 0
context:
  - '{project-root}/src/AGENTS.md'
---

<frozen-after-approval reason="human-owned intent — do not modify unless human renegotiates">

## Intent

**Problem:** Panel Đề xuất AI có tiêu đề lặp lại tên tab dock, và để nhiều thứ chỉ dùng một lần lên trên các nút dịch: ô chọn bộ prompt, câu trạng thái bộ, Thư viện prompt, "Prompt tự lắp khi dịch.", tóm tắt Glossary. Nút "Dịch các câu đã chọn" luôn bị khoá mà không nói cách chọn câu (chỉ có ⌘⇧↓/⌘⇧↑), và không có cách nào bỏ vùng chọn: `segment.selection.clear` không có phím tắt lẫn nút, còn phím ngược chiều chỉ thu vùng chọn về một câu.

**Approach:** Panel chỉ còn dòng đầu (nhà cung cấp · mô hình · bộ đang dùng · nút Prompt) và hai hàng nút. Phần chọn bộ chuyển vào màn "Xem prompt", nay đổi tên thành "Prompt". Nút dịch theo vùng chọn ghi phím chọn câu ngay cạnh và có nút "Bỏ chọn". Esc trong ô dịch bỏ luôn vùng chọn.

**Đã chốt (Ice qua Sally, 2026-10-26):**
- Giữ tab dock và bỏ span `panel.ai_translation.title` bên trong panel. Ở tầng `short`, tab là cách duy nhất để chuyển giữa Tra cứu và Đề xuất AI.
- Dòng đầu: `provider · model · <tên bộ>` và nút "Prompt" (`ai.prompt_inspector.open`). Chưa chọn bộ: "Chưa chọn bộ prompt — Chọn bộ", nút "Chọn bộ" mở màn Prompt. Chưa có bộ nào: một dòng trỏ tới Thư viện prompt.
- Bỏ khỏi panel: SelectMenu bộ prompt, "Đang dùng: X" / "Chưa chọn bộ nào…", nút Thư viện prompt, "Prompt tự lắp khi dịch.", tóm tắt Glossary, đường kẻ trước nút lô. Lỗi lắp prompt và lỗi lưu lựa chọn bộ chỉ còn hiện trong màn Prompt.
- Hàng 1 giữ nguyên: Dịch câu này · Huỷ dịch · Đưa sang bản dịch (· Thử lại). Gợi ý "Đặt con trỏ vào ô dịch…" giữ nguyên.
- Hàng 2: "Dịch câu đã chọn". Chưa chọn câu nào: nút khoá, bên cạnh ghi "<phím> chọn câu", phím đọc từ bảng phím tắt thật của `segment.selection.extend_down`. Đã chọn: nhãn "Dịch N câu đã chọn", ẩn dòng ghi phím, hiện thêm nút "Bỏ chọn" (`segment.selection.clear`). Hàng này chừa chỗ cho nút "Dịch cả Chương" (`deferred-work.md`, 2026-10-26).
- Dòng tiến độ và các lỗi của lô luôn hiện. Danh sách hàng `.ai-batch-rows` cuộn bên trong panel.
- Esc trong ô dịch xoá điểm cắt câu nguồn VÀ bỏ vùng chọn câu.
- Màn Prompt: tiêu đề và nhãn nút mở là "Prompt". Đầu màn là chọn bộ (giữ cách nhớ theo Tác phẩm của 8ad0753) · Thư viện prompt, rồi tóm tắt Glossary, rồi phần prompt đã gửi như cũ. Màn vẫn chọn được bộ khi chưa có câu nào được chọn.
- Đổi bộ trong màn Prompt: chỉ tự lắp cho câu hiện tại khi màn CHƯA có bản ghi (ví dụ đang báo `no_set_selected`). Đã có bản ghi thì giữ nguyên bản đã gửi, hiện ghi chú "lắp bằng bộ khác" (`aiPromptRecord.prompt_set_name !== selectedPromptSetName`) và nút "Lắp prompt cho câu đang chọn" có sẵn. Cùng khuôn với bản ghi của câu khác.
- Giữ một plan dù vượt 1600 token (khoảng 1600 từ; tiktoken không có trên máy nên chưa đếm được).

## Boundaries & Constraints

**Always:**
- `@click` là đúng một `dispatch('<id>')`. Đổi bộ đi qua `setSelectedPromptSetName` như hiện nay (`@change`).
- Mọi chuỗi đi qua `t()`/`tError()`. Không viết cứng ký hiệu phím: lấy từ `shortcutRows` như `StatusBar.vue::shortcutHintText`.
- Tên bộ trên dòng đầu đọc `selectedPromptSetName`, cùng nguồn với ô chọn trong màn Prompt.
- Khoá `vi.json` không còn ai đọc thì xoá.

**Never:**
- Không đổi đường dịch lô, hộp xác nhận chống mất bản nháp, ⌘⇧↓/⌘⇧↑, hay mô hình vùng chọn (`proofreadScope.ts:22` cũng đọc nó).
- Không gán `Escape` cho `segment.selection.clear` trong keymap: `createKeymap` ném khi trùng hợp âm với `editor.clear_source_cuts`.
- Không tự bỏ vùng chọn khi caret đổi hoặc rời lưới (bấm nút trên panel làm tiêu điểm rời lưới).
- Không làm "Dịch cả Chương" trong plan này. Không đổi Rust.

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected Output / Behavior | Error Handling |
|----------|--------------|---------------------------|----------------|
| Đã có bộ | Bộ "Tiên hiệp" đã chọn | Dòng đầu `ollama · gemma3 · Tiên hiệp`, không có ô chọn | — |
| Chưa chọn bộ | Có bộ nhưng `selectedPromptSetName === null` | "Chưa chọn bộ prompt — Chọn bộ"; bấm thì màn Prompt mở | — |
| Không có bộ nào | `promptSets` rỗng | Dòng trỏ Thư viện prompt; bấm thì Thư viện mở | — |
| Chưa chọn câu | Vùng chọn rỗng | Nút lô khoá, có ghi "⌘⇧↓ chọn câu", không có "Bỏ chọn" | — |
| Đã chọn 3 câu | ⌘⇧↓ hai lần từ một ô dịch | "Dịch 3 câu đã chọn" bật, có "Bỏ chọn", không có dòng ghi phím | — |
| Bỏ chọn | Bấm "Bỏ chọn" hoặc Esc trong ô dịch | Vùng chọn rỗng, lưới hết tô; Esc vẫn xoá điểm cắt | — |
| Phím gán lại | Ice gán phím khác cho `extend_down` | Dòng ghi phím hiện phím mới | Không có phím nào: ẩn dòng ghi phím |
| Màn Prompt, chưa có câu | Chưa đặt con trỏ, mở màn Prompt | Chọn bộ và mở Thư viện được; gợi ý `no_segment_hint` | — |
| Chọn bộ khi đang báo lỗi | Màn đang hiện `no_set_selected`, có câu hiện tại | Chọn bộ xong thì tự lắp và hiện prompt của câu đó | Lỗi lắp mới hiện qua `tError` |
| Đổi bộ khi đã có bản ghi | Bản ghi lắp bằng "Tiên hiệp", Ice chọn "Đô thị" | Bản ghi giữ nguyên, có ghi chú "lắp bằng bộ khác" và nút lắp; không tự lắp | — |

</frozen-after-approval>

## Code Map

- `src/panels/AiTranslationPanel.vue` -- template 443–821: `.ai-panel-head` (title, provider, `[data-ai-prompt-inspector-open]`), `.ai-prompt-bar` (SelectMenu, `promptSetChoiceError`, status, `[data-prompt-library-open]`), `.ai-inspector-bar` (`glossarySummaryText`, `aiPromptAssembleError`, `[data-ai-prompt-auto-note]`), `.ai-batch-bar`. Script: `promptSetOptions`/`onPromptSetSelectChange`, `glossarySummary*`, `isRecordStale` là phần chuyển đi. Giữ `watch(editorActiveSegmentId, clearAiPromptAssembleError)` và `loadPromptSets()` ở `onMounted`, vì panel luôn được dựng còn màn Prompt thì không.
- `src/AiPromptInspectorOverlay.vue` -- `.aip-head` chứa `ai.prompt_inspector.title`. Ô chọn bộ và tóm tắt Glossary đặt sau head, trước `aiPromptRecord === null`. `[data-ai-prompt-inspector-open]` là đích trả tiêu điểm khi đóng màn.
- `src/aiPromptInspectorState.ts:163` `openAiPromptInspector` -- tự lắp khi chưa có bản ghi. Khi chọn bộ lúc chưa có bản ghi thì dùng lại `ai.prompt.assemble` (`src/main.ts:1100`, đọc `selectedPromptSetName`). Nút `[data-aip-assemble-selected]` và ghi chú `isStale` của overlay hiện mới chỉ xét trường hợp câu khác; mở rộng thêm cho trường hợp bộ khác.
- `src/promptSetState.ts` -- `promptSets`, `selectedPromptSetName`, `setSelectedPromptSetName:215`, `promptSetChoiceError`.
- `src/StatusBar.vue:195` -- khuôn đọc phím từ `shortcutRows` (`src/config/shortcutsState.ts:138`).
- `src/panels/GridPanel.vue:1642` -- nhánh `Escape` của `onEditKeydown` (gắn ở `GridBlock.vue:280`). Thêm `dispatch('segment.selection.clear')`, giữ lời ghi không `preventDefault`.
- `src/commands/index.ts:3216` -- `segment.selection.clear` đã đăng ký, `keys: undefined`.
- `src/i18n/vi.json` -- `ai.prompt_inspector.title:227`, `command.ai.prompt_inspector.open:792`, `panel.ai_translation.*:1406–1416`.
- Test phải chuyển theo: `tests/frontend/aiPromptSetColumns.test.ts:35`, `promptLibraryOverlayRender.test.ts:763–830`, `aiPromptInspector.test.ts:1119–1207,1373`, `aiTranslateBatch.test.ts:231`, `aiPromptInspectorScreen.test.ts`. `e2e/specs/story-12-14-prompt-library.e2e.mjs:32` bấm `[data-prompt-library-open]` của panel, nay phải mở màn Prompt trước. `settingsFrame.test.ts:96` đọc nút của Cài đặt, không đổi.

## Tasks & Acceptance

**Execution:**
- [ ] `src/panels/AiTranslationPanel.vue` -- dựng lại dòng đầu và hai hàng nút theo Intent; bỏ các khối đã liệt kê; `.ai-batch-rows` có `max-height` theo token và `overflow-y: auto` -- panel thấp vẫn thấy đủ nút
- [ ] `src/AiPromptInspectorOverlay.vue` -- thêm ô chọn bộ, nút Thư viện prompt, `promptSetChoiceError`, tóm tắt Glossary (kèm dấu "của câu khác" khi bản ghi cũ) -- chọn bộ ngay tại chỗ xem prompt
- [ ] `src/panels/GridPanel.vue` -- Esc bỏ luôn vùng chọn -- lối bỏ chọn bằng bàn phím
- [ ] `src/i18n/vi.json` -- "Prompt", các nhãn mới, xoá khoá không còn ai đọc -- `check:i18n`
- [ ] tests (Code Map) + một test mới cho dòng đầu, ghi phím, Bỏ chọn, Esc theo I/O Matrix -- canh từng hàng của ma trận

**Acceptance Criteria:**
- Given panel ở tầng `full`, when nhìn từ trên xuống, then chữ "Đề xuất AI" chỉ xuất hiện một lần (trên tab dock).
- Given tầng `short`, when Ice bấm tab Đề xuất AI rồi tab Tra cứu, then hai panel vẫn chuyển qua lại được.
- Given màn Prompt đang mở, when Ice đổi bộ, then dòng đầu panel hiện tên bộ mới, và mở lại Tác phẩm vẫn còn bộ đó.

## Design Notes

Kiểu chữ "Chọn bộ" và "Bỏ chọn" giống `.ai-translate-cancel` (nút chữ). Dòng ghi phím dùng `.ai-translate-hint`.

## Verification

**Commands:**
- `npm run test:story -- --list` rồi chạy đúng các tệp vitest ở Code Map -- expected: qua, và dòng `Errors` trống
- `npm run check:i18n && npm run check:commands && npm run check:lint` -- expected: xanh
- `npm run test:e2e -- --spec e2e/specs/story-12-14-prompt-library.e2e.mjs` (ngoài sandbox) -- expected: qua

**Manual checks (if no CLI):**
- Ice mở app: dòng đầu panel, ⌘⇧↓ rồi Bỏ chọn và Esc, đổi bộ trong màn Prompt.

## Implementation Notes

- Thư viện prompt mở từ màn Prompt nằm DƯỚI màn đó (cùng z-index, màn Prompt dựng sau), nên `promptLibraryState` nhớ `promptLibraryOverInspector` và scrim thư viện lên z-index 11 khi mở chồng.
- Nút "Chọn bộ" và "Prompt" trên dòng đầu cùng dispatch `ai.prompt_inspector.open`; `panel.ai_translation.title` giữ lại vì `workspaceLayout.ts` dùng làm nhãn tab dock.
- Hàng chú thích "Đặt con trỏ…" thêm `data-ai-no-segment-hint` để tách khỏi dòng ghi phím cũng mang `.ai-translate-hint`.
- Chưa chạy e2e `story-12-14` (cần ngoài sandbox); spec đã sửa để mở màn Prompt trước khi bấm Thư viện prompt.
  🔵 2026-10-26: đã chạy ngoài sandbox, 1 passing (bản trước năm bản vá review; các bản vá không đụng đường spec này).

## Plan Change Log

## Review Triage Log

Lượt 1 (quick): high 0 · medium 0 · low 5 · false 5 · maybe-false 1.
- low · patch — `isOtherSet` đúng khi `selectedPromptSetName === null` (chọn "— chưa chọn —"): ghi chú hiện `“”` và nút lắp chỉ ra lỗi `no_set_selected`; thêm điều kiện bộ đang chọn khác `null`.
- low · patch — `command.segment.selection.clear_short` là khoá `command.*` không có command nào (không tiền lệ `_short` trong `vi.json`); đổi sang khoá `panel.ai_translation.*`.
- low · patch — `src/App.vue:84` còn ghi nút "Xem prompt".
- low · patch — `src/aiPromptInspectorState.ts:187` còn trỏ `.ai-inspector-alert` của panel, class đã xoá.
- low · patch — lời ghi trên nhánh `Escape` (`GridPanel.vue`) ghi "một command" trong khi nhánh nay phát hai.
- false — `commands/index.ts:1142,4682` ghi panel gọi `prompt.library.open`: vẫn đúng, nhánh `promptSets` rỗng của dòng đầu gọi nó.
- false — "lưới hết tô" chưa có assert: tô màu đọc `segmentSelectionIds` (`GridPanel.vue:308`), test đã kiểm nó rỗng.
- false — thiếu test cuộn `.ai-batch-rows`: không có hành vi sai nào được nêu; là CSS trong diff.
- false — thiếu test AC tầng `short`: diff không đụng tab dock hay `workspaceLayout.ts`.
- maybe-false · reject — chọn bộ trong lúc lượt lắp của `openAiPromptInspector` chưa về có thể chạy hai lượt lắp; cần đo thời gian `ai_prompt_assemble` so với một cú chọn bằng chuột; nếu thật thì chỉ low.
- false — spec e2e `story-12-14` chưa chạy: đã chạy ngoài sandbox, 1 passing — thư viện mở được từ màn Prompt và luồng của spec đi hết.
- defer — `promptSets` rỗng lúc chưa nạp xong hoặc nạp lỗi vẫn hiện "Chưa có bộ prompt nào"; có từ trước (panel cũ cùng điều kiện), ghi sổ nợ.
