# Analyzer digest — Q6 @ 99f3fc1

Short answer: no test counts how many `GridBlock`s re-render on a keystroke, and none measures wall-clock time for typing. The only GridBlock render counter (`gridBlockRenderCost`) drives caret moves, strip-anchor moves and confirm, not typing. The vitest typing-cost case counts stubbed `PanelFrame` renders and `segmentRuleInputOf` calls. In vitest every ordinary typed character is synthetic: set `textContent` (or `Text.data`/`appendData`) then a plain `new Event('input')`, no `keydown` or `beforeinput insertText` before it; `keydown`/`beforeinput` only for Enter, IME, paste. The e2e typing case at `editor-typing-flush.e2e.mjs:161-182` has no skip/fails marker; a comment says "red on purpose" and must not be skipped. Budget 15 reached.

## Config
- `vitest.config.ts:57 defineConfig @ 99f3fc1` happy-dom; only `tests/frontend/**/*.test.ts` (`:60`); header: happy-dom is not WebKit, real-engine claims belong to bench page or e2e (`:43-49`).
- `tests/frontend/support/setup.ts:83-84 @ 99f3fc1` patches `document.execCommand` (absent in happy-dom); comment `:71` returns `false`, no DOM touch (grep only).

## Render-cost counters
- `tests/frontend/gridTypingCost.test.ts @ 99f3fc1`: `type` helper (`:108-112`) sets `cell.textContent`, dispatches `new Event('input', {bubbles:true})`, awaits `$nextTick` — synthetic, input-only (LB). Probes: `segmentRuleInputOf` wrapped (`:14-23`); `PanelFrame` stub render increments `probe.frameRenders` (`:58-67`) — counts panel frame, not GridBlock (LB). `:115-128` `it.each([20, 200])` (rows `many(n)` `:76-87`): caret on 101, three strings; `frameRenders === 0`, `ruleInputCalls === 0`, `editorEditedText.get(101) === 'Dich 1.abc'`. 3-segment cases: `:130` first char removes `.empty`, `rule-draft` after caret leaves; `:144` delete to empty restores `.empty`; `:158` `goToNextUntranslated()` false after typing; `:168` `flushEditorNow` sends `[{id:12,target_text:'ab'}]`; `:179` `resetEditorPanel` empties map.
- `tests/frontend/gridCaretMoveCost.test.ts @ 99f3fc1`: `:111-123` `it.each([20, 200])` two caret moves `ruleInputCalls === 0`; `:125-305` each reactive value reaches row; typing cases synthetic (`:140-150`, `:152-162`, `:201-213` failed flush adds `.refused`). No GridBlock render count.
- `tests/frontend/gridBlockRenderCost.test.ts @ 99f3fc1` only GridBlock render counter: global mixin `updated()` pushes `$props.block.index` (`:74-82`); `settle` unique indices after `$nextTick` (`:92-97`); 1050 segments (`:7`); cases `:99-165`: caret inside block `[1]`, across boundary `[1,2]`, anchor+caret `[1,3]`, anchor only `[1,4]`, confirm across `[1,2]`, confirm inside `[1]`. None types (LB).
- `tests/frontend/gridConfirmWorkCost.test.ts:149-187 @ 99f3fc1`: confirm at 1050 vs 2100; patches `String.prototype[Symbol.iterator]`, `querySelector`/`querySelectorAll` (`:115-138`); equal passes, `gridQueries === 0`. No typing.
- `tests/frontend/gridRuleStable.test.ts @ 99f3fc1` (3 segments `:47`): `:43-69` MutationObserver on `.col-rule` across caret moves 0 nodes; `:79-90` CSS border width regex; `:92-100` synthetic `input` ⇒ `rule-draft`.
- `tests/frontend/gridBlocks.test.ts @ 99f3fc1`: `gridBlocksOf` (`:97-121`), structure at 20/450/6100 rows (`:123-217`), source selection/copy (`:235-313`). No typing, no render count.
- `tests/frontend/gridRestoreEditedText.test.ts @ 99f3fc1`: `typeInto` sets `firstChild.data` + `input` (`:32-36`); `:48-59` typed text survives `replaceEditorSegment`; `:61-74` 0 `[data-segment-id]` scans.

## Typing behaviour
- `tests/frontend/editorTypingZone.test.ts @ 99f3fc1` (3 segments; caret via Range + synthetic `selectionchange` `:98-113`): `:121` contenteditable, 2×N anchors; `:140`, `:160` attribute stable, caret offset 9; `:200-250` Enter (`keydown` not prevented, `beforeinput insertParagraph` prevented, `insertLineBreak` not, anchor count same); `:263` `insertLineBreak` input keeps `\n`; `:285-297` ordinary sentence `textContent` + `new Event('input')` only (LB); `:299-343` paste `beforeinput insertFromPaste` prevented, `\r\n`→`\n`; `:345` `\n`-only stays `.empty`; `:383-467` IME sequence; `:469-505` `appendData` ×3 + `input` keeps same text node (reversed-typing guard); `:507` other row unchanged; `:535` survives unmount/remount, 0 saves.
- `tests/frontend/editorFlush.test.ts @ 99f3fc1` pure `createEditorFlush`, no DOM, no fake timers: constants (`:54-58`); 300 events / 100 ms ⇒ ≥6 flushes, gaps ≤5000 (`:61-78`); single event `[2000]` (`:81`); in-flight edits kept (`:88`); cap not reset `deadline() === 5000` (`:110-129`); only changed ids (`:131`).
- `tests/frontend/proofreadWiring.test.ts @ 99f3fc1`: `:132-160` mocked modules (`fresh()` `:24-52`), 0 `ai_proofread_*` invokes until dispatch; no DOM typing. `:174-196` source regex checks on `GridPanel.vue`/`GridBlock.vue`.

## e2e (WebdriverIO, real WKWebView)
- `e2e/specs/editor-typing-flush.e2e.mjs @ 99f3fc1`: header `:38-55` — `browser.keys()` fires only `keydown`, no text lands ⇒ types with `execCommand('insertText')` (`beforeinput` then `input`). Case `:92` plain `it(...)`, no `.skip/.only/xit/pending`: editable count, real-click focus (`:143-159`), `execCommand` (`:161-167`), `inserted.ok === true` (`:181`), comment `:168-180` why meant red; row count, flush, disk read back (`:198-210`), "Đã lưu N giây trước" (`:218`). Case `:234` `insertText`+`insertParagraph`+`insertText`: no child elements, `'Dong mot.\nDong hai.'`, 2 line rects, no row split, `\n` on disk, paragraph flag unchanged. No render count, no timing.
- `e2e/specs/grid-blocks.e2e.mjs:80-176 @ 99f3fc1` (1200 sentences): six blocks; jump to block 5 in view; block-5 selection reaches Lookup; `Mod+Shift+ArrowDown` across boundary paints 5 rows × 4 columns; chords synthetic `KeyboardEvent('keydown')` (`:26-31`). No typing.
- `e2e/specs/proofread-underline.e2e.mjs` not read (grep: synthetic `keydown`/`beforeinput` `:383-385`, `:439-449`, incl. `keydown 'a'` + `beforeinput insertCompositionText` `:448-449`).

## cargo
- `src-tauri/tests/flush_cadence_contract.rs:126-147 @ 99f3fc1` parses `EDITOR_IDLE_MS`, asserts `Tuning::default().idle_before_passive > EDITOR_IDLE_MS`; self-tests `:156-216`. No webview.
- `src-tauri/tests/segment_contract.rs:4566 typed_text_round_trips_through_the_flush_and_the_load_command @ 99f3fc1`: 5 segments, `save_segment_targets` with Vietnamese text + one empty string, `read_open_chapter_segments`; row count, ids, ords, paragraph flags, exact `target_text`, `source_text` unchanged (`:4577-4640`; not read past).

## Not found
- GridBlock re-render count on a keystroke (`mixins|updated(|renderTracked|renderTriggered|onRenderTriggered` in `tests/`, `e2e/`, `src-tauri/tests`).
- Wall-clock typing timing (`performance.now|Date.now`; only `story-12-9-strip-row-track.e2e.mjs:50`, a strip test).
- Ordinary character in vitest with `keydown` or `beforeinput` before `input`.
- Skip/fails marker on `editor-typing-flush` cases.
- Not read: `setup.ts` in full, `proofread-underline.e2e.mjs`, `segment_contract.rs` past `:4640`.
