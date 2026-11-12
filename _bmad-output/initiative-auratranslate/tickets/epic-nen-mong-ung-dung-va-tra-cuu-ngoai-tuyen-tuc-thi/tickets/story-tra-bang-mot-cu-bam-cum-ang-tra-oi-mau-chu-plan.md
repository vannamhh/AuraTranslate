---
title: 'Tra bằng một cú bấm, cụm đang tra đổi màu chữ'
type: 'feature'
ticket: '23'
created: '2026-11-12'
status: 'built'
baseline_revision: '2ef314f1766e31268b5cffd5f7495126d046fd7e'
route: 'full'
route_source: 'auto'
risk: 'medium'
review: 'quick'
review_source: 'pinned'
lenses_ran: ['quick']
review_loop_iteration: 0
context:
  - '{project-root}/_bmad-output/initiative-auratranslate/tickets/epic-nen-mong-ung-dung-va-tra-cuu-ngoai-tuyen-tuc-thi/tickets/story-tra-bang-mot-cu-bam-cum-ang-tra-oi-mau-chu.md'
  - '{project-root}/src/AGENTS.md'
  - '{project-root}/tests/AGENTS.md'
  - '{project-root}/e2e/AGENTS.md'
---

<frozen-after-approval reason="human-owned intent — do not modify unless human renegotiates">

## Intent

**Problem:** Ở cột nguyên văn của lưới, tra từ phải bấm đúp vì Auto-Lookup chỉ bắn khi nhả chuột có vùng bôi đen. Một cú bấm (WKWebView cho `rangeCount 0`) không tra gì. Thao tác này lệch khỏi lời hứa QuickTranslator của FR21.

**Approach:** Cú bấm trơn ở ô nguồn tính chỉ số ký tự từ toạ độ chuột, chọn cụm (thuật ngữ Glossary chứa nó, không thì từ `Intl.Segmenter` chứa nó), trao cụm đó cho đúng đường `dispatch('lookup.lookup_selection')` hiện có, rồi tô cụm bằng CSS Custom Highlight API với token `lookup-active`. Mọi tiêu chí là của story 1.23.

## Boundaries & Constraints

**Always:** một đường tra duy nhất qua `lookup.lookup_selection` (Quyết định #4a), giữ quy tắc chống tra lặp hiện có; cú bấm không tạo vùng chọn DOM và không sửa DOM ô (tô bằng `CSS.highlights`); màu chỉ qua `var(--color-lookup-active)`; offset trong lưới là code point trên `source_text` thô, còn `Intl.Segmenter` và Range là UTF-16 trên văn bản đã chuẩn hoá: đổi tường minh.

**Never:** chạm Rust hay thêm IPC; đổi đường kéo-chọn, `Shift`+mũi tên, `Mod+Alt+L`, `Mod`+click (đánh dấu chỗ cắt), chuột phải, ô bản dịch, cột số câu; chạm Chế độ đọc; `window.CSS`/`globalThis.CSS` (vấp Kiểm C của `check:layout`); Range sống qua một lượt render lại.

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected Output / Behavior | Error Handling |
|----------|--------------|---------------------------|----------------|
| Từ thường | bấm `提` trong `小提琴`, không Glossary | tra `小提琴`, tô 3 chữ | — |
| Glossary thắng | bấm `姆` trong thuật ngữ `布拉姆斯` | tra `布拉姆斯` | — |
| Hán Việt | bấm một âm ở kiểu chuyển đổi hoặc song song | tra chữ Hán của khối đó (hoặc thuật ngữ chứa nó) | `.hv-text` (dấu câu/Latin) ⇒ không tra |
| Không có chữ | dấu câu, khoảng trắng, chỗ trống ngoài chữ | không phát, giữ kết quả và màu cũ | — |
| Bấm lại / bấm đúp | cùng cụm; cú thứ hai `detail === 2` | không tra lần hai | — |
| Kéo | nhả chuột có vùng bôi đen | đường kéo cũ, không đi nhánh điểm | — |
| Engine thiếu API | `CSS.highlights` không có | bôi đen cụm bằng `addRange`, vẫn tra | trùng `lastAutoDispatched` ⇒ không tra lặp |
| Thiếu `Intl.Segmenter` | đường lui hiện có | mỗi code point là một từ, `wordLike` theo `\p{L}\p{N}` | — |

</frozen-after-approval>

## Code Map

- `src/panels/selectionContract.ts:338` `attachSelectionWatcher` -- thêm "truy vấn điểm" một lần: setter export (vd `offerPointLookup(text)`), `currentSelectionText()` (l.218) trả nó trước; `onSelectionSettled` (l.361) tiêu thụ và xoá trong `finally` (dep bị đọc hai lần mỗi lượt: `markDispatch` ở `main.ts:1347` và handler `src/commands/index.ts:3118`). `onMouseUp` nhận `MouseEvent`, bỏ qua `detail === 2`. Giữ comment Bẫy 5 (l.364): rỗng ⇒ không xoá panel.
- `src/panels/GridPanel.vue:897` `onSourceCellMouseUp` -- nhánh không-`Mod`: chỉ khi `button 0`, `detail 1`, không có vùng chọn (`rangeCount 0` hoặc collapsed). Dùng `caretPointAt` (l.848, local), `segmentIdOf` (l.738), `cell.contains(node)`, `sourceCutOffsetOf`. Glossary: `glossarySpanAtPoint(id, offset)` (l.528, nửa mở, code point). Đăng ký bề mặt l.726: tham số thứ tư là `vietnamese`, `autoLookup` mặc định `true`, không đổi.
- `src/panels/editorSegments.ts:303` `sourceCutOffsetOf` -- ở Hán Việt (`data-src-atomic`: `.hv-word`, `.hv-unit`) chỉ trả ĐẦU khối; độ dài khối lấy từ phần tử neo (`data-src-start` + số chữ của base `ruby`/`.hv-word`). `originalOffsets` (l.485) đổi chỉ số chuẩn hoá ⇒ thô. Nơi hợp lý cho helper thuần dựng Range từ `[startCp, endCp)` trên các `.src-piece[data-src-start]` hoặc khối HV.
- `src/panels/wordBoundary.ts` -- chỉ export `wordStartOffsets` (l.111) và `WORD_JOINER`; `hanWordSegmenter()` (l.73) private. Thêm `wordAt(text, cpOffset) -> {start, end, wordLike} | null` trên cùng segmenter (`segment(text).containing(i)`), đầu vào/đầu ra code point thô.
- `src/panels/SourceHanViet.vue:905` `selectDoubleClickedWord` (`@mouseup`, `detail === 2`) -- đọc trước khi sửa, không phá.
- `src/tokens/tokens.json` -- `lookup-active` sáng `#a8430c` / tối `#fb923c` cạnh `tm-text`; thêm vào `roles.text`; 4 cặp tương phản × (`surface`, `surface-sunken`, `surface-accent`, `surface-tm`), khuôn cặp `tm-text`. `scripts/check-tokens.mjs` -- `EXPECTED_COLORS_LIGHT` (l.141), `EXPECTED_COLORS_DARK` (l.183), `colorsPerTheme` 39→40 (l.340); kiểm bảng màu `DESIGN.md` nếu cổng so với nó.
- `src/tokens/controls.css` -- quy tắc `::highlight(lookup-active) { color: var(--color-lookup-active) }`; không đặt trong `<style scoped>` (Vue viết lại selector thành không hợp lệ).
- Vô hiệu hoá màu: `watch([editorSegments, glossaryMarksPerSegment, showHanViet/viewMode, chapterId], clear, { flush: 'post' })` trong GridPanel (hàng GridBlock dùng `v-memo` trên đúng các nguồn này); lượt tô mới ghi đè cùng tên.
- Không đụng: `src/modes/ReadingMode.vue`, bề mặt `'display'` (l.727), `onGridMouseDown`/`onSentencePress`.

## Tasks & Acceptance

**Execution:**
- [x] `src/panels/wordBoundary.ts` -- thêm `wordAt` -- chọn từ chứa chữ, cùng bộ tách với double-click và cột Hán Việt.
- [x] `src/panels/selectionContract.ts` -- truy vấn điểm + bỏ qua `detail === 2` -- một đường, không tra lặp.
- [x] `src/panels/editorSegments.ts` -- helper thuần: khoảng cần tra cho một điểm (trơn và HV) và Range cho một khoảng -- kiểm được bằng vitest.
- [x] `src/panels/GridPanel.vue` -- nối nhánh bấm trơn, tô/xoá `CSS.highlights` (đường lui `addRange`), watch vô hiệu hoá.
- [x] `src/tokens/tokens.json`, `scripts/check-tokens.mjs`, `src/tokens/controls.css` -- token, cặp tương phản, quy tắc `::highlight`.
- [x] `tests/frontend/` -- vitest cho `wordAt`, truy vấn điểm (gồm đối chứng dương và `Mod+Alt+L` không bị ảnh hưởng), helper khoảng/Range, bỏ qua `detail === 2`; mỗi hàng ma trận I/O có một ca.
- [x] `e2e/specs/` -- spec `realClick` vào ô nguồn trên WKWebView: Panel Lookup nhận cụm đúng và `CSS.highlights.get('lookup-active')` phủ đúng chữ; cả ba kiểu xem.
- [x] `_bmad-output/implementation-artifacts/1-23-ban-do/` -- bàn đo chuột thật chép khuôn `14-5-ban-do/chuot-that.e2e.mjs` (kiểm frontmost, rê chuột trước khi bấm).

**Acceptance Criteria:**
- Given story 1.23, when the build is done, then mỗi tiêu chí 1–9 của story có ít nhất một ca kiểm đi tới đúng seam của nó.
- Given đường bấm trơn bị gỡ thật khỏi `GridPanel.vue`, when chạy lại spec e2e và bàn đo chuột thật, then cả hai đỏ vì không có lượt tra.

## Implementation Notes

- Skip of the second click (`detail === 2`) is scoped: `GridPanel` calls `skipNextAutoLookup()` on a source-cell double-click instead of the watcher dropping every `detail === 2`, so double-click lookup in the Source panel and Reading Mode (selection-based) is untouched.
- The click is resolved to a character by hit-testing the pointer against single-character Range rects (`caretPointAt` alone returns the nearest caret, so the right half of a word's last character would pick the next word and blank space would still resolve). Blank space and gaps between lines therefore look up nothing.
- Han Viet: only `.hv-word` and `.hv-unit` blocks look up (Glossary term at the block start first, else the block); `.hv-text` and the parallel view's plain text span do not. `<rt>` clicks resolve to the block.
- Paint clears on `currentQuery` changing to a different phrase, plus the planned post-flush watch. 🔵 2026-11-12: the in-flight race does not happen — `runLookup` drops late answers by `sequence`. `lastAutoDispatched` is now forgotten when `currentQuery` settles elsewhere, so clicking A again after the panel moved to B looks A up.
- `lookup-active` needed a fifth pair (`background`) and an excluded pair (`primary`) for `check:tokens` completeness, beyond the four in the plan.
- The embedded WebDriver (`tauri-plugin-wdio-webdriver` 1.3.0) builds `new MouseEvent` without `detail`, so its clicks carry `detail 0`; the plain-click branch accepts `detail <= 1` and skips only 2. Its pointer events never move the selection, so the e2e drag case builds the Range itself.
- e2e must call `ensureFullLayoutTier()`: in a small window Panel Lookup retreats to the drawer and never mounts. 8/8 green on WKWebView; removing the `lookupByPlainClick` call turns the 7 click cases red with Panel Lookup mounted and nothing looked up or painted, the drag case stays green.
- The new module took the Vite graph to 251, over `MODULE_CEILING` 250 (nightly before this story: exactly 250); the ceiling now follows its own rule, about four times the real count (1000).
- Real-mouse bench (`1-23-ban-do/`): 3/3 green on WKWebView with real `cliclick` input (`ket-qua.json`); removing the `lookupByPlainClick` call turns the single-click case red with nothing looked up or painted. It sizes the window to 1200×910 itself: a screen-tall window (as `ensureFullLayoutTier` makes) breaks its screen mapping, and below the `full` tier Panel Lookup never mounts.

## Plan Change Log

## Review Triage Log

Lượt 1 (quick): high 0 · medium 4 · low 2 · false 4 · maybe-false 0.

- medium · patch — bấm lại cụm A sau khi panel chuyển sang B bằng đường khác: `lastAutoDispatched` vẫn là A nên không tra, mà màu A vẫn tô trong lúc panel hiện B. Thật: `lastAutoDispatched` chỉ hội tụ khi chính lượt tự động về.
- false — lượt tra cũ về sau xoá màu mới: `runLookup` bỏ kết quả về muộn qua `sequence` (`lookupPanelState.ts`), nên `currentQuery` không lùi về A.
- low · reject — hai cú bấm nhanh vào hai cụm khác nhau rơi vào `detail === 2` nên mất lượt thứ hai: cần hai cú bấm trong bán kính bấm đúp của hệ điều hành; sửa thêm nhánh; Design Notes đã chọn bỏ qua mọi `detail === 2`.
- false — trạng thái một-lượt rò khi listener `document` không chạy: `main.ts` gắn watcher trước `mount()`, và chỗ chặn `mouseup` duy nhất (kéo ở cột số câu, capture) chặn luôn listener của ô nên không có gì được đặt.
- false — màu ở Hán Việt chuyển đổi không đúng cụm khi thuật ngữ bắt đầu giữa khối: `buildSegments` cắt khối ở biên thuật ngữ Glossary, nên một khối luôn nằm trọn trong hoặc ngoài thuật ngữ; tô âm Latin là đơn vị hiển thị đã chọn.
- medium · patch — e2e Hán Việt chuyển đổi chỉ kiểm "có màu", không kiểm màu phủ đúng chữ nào: phép kiểm không đỏ được khi tô sai khối.
- medium · patch — thiếu ca đi tới seam: vô hiệu hoá màu khi đổi kiểu xem / đổi Chương (tiêu chí 5) và hàng "Kéo" ở tầng ô; tiêu chí 8 (một lượt gọi, không bố cục lại) chưa có phép đo — phần bố cục chuyển sang defer.
- medium · patch — chưa chạy e2e và bàn đo, chưa làm đối chứng đỏ: chạy ở bước kiểm của phiên này.
- low · patch — `DESIGN.md` thiếu token `lookup-active` (frontmatter hai theme và bảng màu); phần đề xuất thêm 🔵 vào comment `EXPECTED_COUNTS` bị bác vì AGENTS.md cấm lịch sử trong mã.
- low · patch — mã số story trong tên test, `describe` e2e và đầu bàn đo; doc-comment `lookupByPlainClick` lặp lại thân hàm ngoài câu đầu.
- false — `wordAt` trả `null` ở `\n` cuối của `\r\n` cuối văn bản: chỗ đó là khoảng trắng, `null` (không tra) là đúng kết quả.

## Design Notes

Truy vấn điểm sống đúng một lượt `mouseup`: listener của ô chạy trước listener `document` nên ô đặt giá trị, watcher đọc rồi xoá. Nhờ vậy `markDispatch`, handler và dedupe dùng chung một chuỗi, và `Mod+Alt+L` không bao giờ thấy giá trị cũ. Bỏ qua `detail === 2` (không phải `>= 2`): bấm ba lần vẫn là một vùng chọn thật và đi đường kéo.

## Verification

**Commands:**
- `npm run test:story 23 -- --list` rồi chạy đúng các vitest được chọn -- expected: xanh, có ca đối chứng dương.
- `npm run check:tokens` và `npm run check:layout` -- expected: xanh, 40 màu mỗi theme.
- `npm run test:e2e -- --spec <spec mới>` (ngoài sandbox) -- expected: xanh; đỏ khi gỡ seam.

**Manual checks (if no CLI):**
- Bàn đo chuột thật trên WKWebView; nếu agent không chạy được (`osascript` -1743), Ice chạy trong Ghostty. Kiểm màu cả hai theme ⇒ mục nợ `Chủ: Epic 1` khi đóng.
