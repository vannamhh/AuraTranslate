## Digest: `tests/` and `e2e/` at cc60020 (plus `tests/` at b2874aa)

Setup that applies to every test below: vitest runs under `happy-dom` (`vitest.config.ts:57 @ cc60020`). It includes `tests/frontend/**/*.test.ts`, with setup file `tests/frontend/support/setup.ts`. Per `tests/AGENTS.md:10 (Conventions) @ cc60020`, geometry, layout and real-engine claims belong to e2e (WebdriverIO, `e2e/wdio.conf.mjs`), not to vitest.

### Q6: how GridPanel rendering and caret movement are tested

**Render-cost tests (the only tests that measure re-render work):**

- `tests/frontend/gridCaretMoveCost.test.ts:110 (describe 'GridPanel caret move cost') @ cc60020`
  - `:111` `it.each([20, 200])` "moving the caret resolves no row rule again". It calls `state.setEditorCaret(100→101→102)` and asserts `probe.ruleInputCalls === 0`. That counter is a spy on `segmentRuleInputOf` (mock at `:13-22`).
  - It does not count GridPanel or row render calls. It has no `frameRenders` probe.
  - `:125-305` is one test per value a row reads. Each checks the DOM after a change:
    - caret: `editing` class, aria-label and `rule-primary` move to the new row (`:125`)
    - written text and `empty` class (`:140`, `:152`)
    - glossary marks (`:164`)
    - Han Viet tab (`:179`)
    - row error from confirm (`:188`) and from flush (`:201`)
    - unlinked (`:215`)
    - selection (`:230`)
    - source cut (`:245`)
    - concordance (`:257`)
    - strip anchor (`:270`). This one mocks `inlineStripAnchorSegmentId` with a ref via `vi.doMock('../../src/inlineStripEligibility')` and expects `.strip-pad` in every `.col-rule/.col-num/.col-src/.col-tgt` column.
  - Helper `mountGrid` at `:91` mounts GridPanel with a `PanelFrame` stub and `attachTo: document.body`.
- `tests/frontend/gridTypingCost.test.ts:114 (describe 'GridPanel typing cost') @ cc60020`
  - `:115` `it.each([20, 200])` "a keystroke inside a written row renders no slot and resolves no rule". It asserts `probe.frameRenders === 0` and `probe.ruleInputCalls === 0`.
  - `frameRenders` is counted by a `PanelFrame` stub whose render function increments it (`:58-67`, `STUBS`). This is the only proxy for "GridPanel re-rendered" in the test tree, and it is asserted only for typing, not for caret moves.
  - `:130-170` call `setEditorCaret` to check `empty` and rule classes after the caret leaves, and also cover navigation and flush.
- `tests/frontend/gridRuleStable.test.ts:42 (describe 'GridPanel margin rule column') @ cc60020`
  - `:43` A caret walk `setEditorCaret(13, 12, 11, null)` under a `MutationObserver` on `.col-rule` asserts no structural DOM mutations and that the same `.margin-slot` elements stay in place.
  - `:79` reads the `GridPanel.vue` CSS source with `readFileSync` to check that `border-left` widths match.

**Other vitest files that mount GridPanel** (`mount(GridPanel…)`, 19 files in total). These test the DOM and behaviour, not render counts:

- **Strip track and strip anchor:** `tests/frontend/gridPanelStripTrack.test.ts:66 (describe 'GridPanel strip track') @ cc60020`.
  - It covers the `gridTemplateRows` `repeat(N, auto)` setting, one `.strip-pad` per column, the track's `gridRow` and `aria-details`.
  - `:123` covers caret retention on focusout into the strip.
  - The anchor is mocked as a ref (`:57`).
- **Row error label:** `tests/frontend/gridPanelRowErrorPriority.test.ts:122` and `:194` (each sets the caret with `setEditorCaret`).
- **Cell label and margin rule:** `tests/frontend/gridPanelCellLabel.test.ts:40`.
- **Han Viet tab focus:** `tests/frontend/gridPanelHanVietTabFocus.test.ts:78`.
- **Images and four-column child counts:** `tests/frontend/gridPanelImages.test.ts:151`. Its `:152` checks that each column has N children and that `gridTemplateRows` is N.
- **TM-filled and unlinked rows:** `tests/frontend/gridPanelTmFilled.test.ts:47` and `:68`.
- **Typed text restored after a grid rebuild:** `tests/frontend/gridRestoreEditedText.test.ts:47`.
- **Caret moves through a DOM Selection (`putCaretIn`, `:98`):** `tests/frontend/editorTypingZone.test.ts:120`.
  - `:140` checks that `contenteditable` does not change when the caret moves.
  - `:160` checks that the caret keeps its offset across a move.
- **Row selection class:** `tests/frontend/segmentSelection.test.ts:374`.
- Also mounting GridPanel: `glossaryHoverSelection.test.ts:129`, `glossaryMarksRefresh.test.ts:521`, `editorClearSourceCuts.test.ts:124`, `tmConcordanceHighlight.test.ts:71`, `tmConcordanceSurfaces.test.ts:63`, `reviewMode.test.ts:355` (GridPanel instance and scrollTop kept). All are under `tests/frontend/ @ cc60020`.

**Tests that read GridPanel source as text:**

- `tests/frontend/proofreadWiring.test.ts:174-217 @ cc60020`. Regex checks on the `GridPanel.vue` script and template: the proofread repaint calls, `@compositionstart`, and the `strip-track` div containing `<ProofreaderStrip place="track" />`.
- `tests/frontend/gridRuleStable.test.ts:80`, the CSS check above.

**Wider `setEditorCaret` use:** 30 files under `tests/frontend/` call it (for example `segmentHistory.test.ts`, `tmFuzzyStrip.test.ts`, `editorNavNotice.test.ts`). Most of them drive state without asserting anything about the grid render.

**e2e (real WKWebView):**

- `e2e/specs/segment-navigation.e2e.mjs:189 @ cc60020` moves the caret between cells with ⌘⌥↓. `:230` and `:285` check the scroll.
- `e2e/specs/story-12-9-strip-row-track.e2e.mjs:86 @ cc60020` focuses `[data-col="tgt"]` cells and samples `requestAnimationFrame` gaps with `performance.now()` (`focusAndSample`, `:44`). This is the only frame-timing measurement in the tree.
- `e2e/specs/story-12-7-workspace-grid-status-bar.e2e.mjs:44 @ cc60020` sets the caret through `state.setEditorCaret` and checks margin-rule computed styles (`:59`) and the four columns (`:70`).
- Shared helpers: `e2e/support/gridWait.mjs:72 (countGridCells)` and `:82 (waitForGridRows)`.
- 19 spec files under `e2e/specs/` mention the caret. One example is `e2e/specs/segment-backspace-merge.e2e.mjs:88 (caretDauO)`.

**Tree B:** it changes one test file. `tests/frontend/proofreadHighlight.test.ts:117 (describe 'repaintProofreadHost after a typed edit in one segment') @ b2874aa` adds `cell.setAttribute('data-col', 'tgt')` to its fixture cells. This matches B's change at `src/panels/proofreadHighlight.ts:124 (repaintProofreadHost) @ b2874aa`, where the selector becomes `[data-col="tgt"][data-segment-id=…]`. B changes no grid render-cost test, so `gridCaretMoveCost`, `gridTypingCost` and `gridRuleStable` are identical in A and B.

### Q4, test side

- No test references `stripAnchorIndex`: `git grep stripAnchorIndex cc60020 -- tests e2e` found 0 hits.
- Tests reach the strip anchor only by mocking `inlineStripAnchorSegmentId` (`gridPanelStripTrack.test.ts:57`, `gridCaretMoveCost.test.ts:273`).
- The anchor function itself is tested in `tests/frontend/stripAnchorSegment.test.ts:44 (describe 'inlineStripAnchorSegmentId') @ cc60020`.

### Q2, Q5, test side

- No test mentions `v-memo` or memo dependencies: `git grep -E "v-memo|vMemo|memo" cc60020 -- tests e2e` found nothing apart from unrelated "proofreadMemory" names.
- The row structure is asserted only through the DOM: one `.col-rule/.col-num/.col-src/.col-tgt` column each, N children per column, `.strip-pad` placement and `gridTemplateRows`. This is in `gridPanelStripTrack.test.ts:75` and `gridPanelImages.test.ts:152`.

### Searched, found nothing (cc60020, `tests/` and `e2e/`)

- `renderCount|onRenderTriggered|onRenderTracked|renderTriggered|onUpdated|patchCount`: no hits.
- `v-memo|vMemo`: no hits.
- `stripAnchorIndex`: no hits.

No test counts GridPanel render-function calls or row re-renders on a caret move. The closest checks are `ruleInputCalls` (caret move) and `frameRenders` (typing only).
