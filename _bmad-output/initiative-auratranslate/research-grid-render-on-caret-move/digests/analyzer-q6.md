Digest for Q6: how GridPanel rendering and caret movement are tested (tree A = cc60020, tree B = b2874aa). 11 files read, within the 15-file budget. Nothing was run.

**Load-bearing claims (marked LB)**
- LB1: No test observes a row's `v-memo` hit or miss, a GridPanel render-function run, or a vnode patch count on a caret move. The only render-cost probes are described below.
- LB2: `gridCaretMoveCost.test.ts` and `gridTypingCost.test.ts` count calls to `segmentRuleInputOf` and `PanelFrame` renders. They are proxies, not patch or diff counts.
- LB3: Tree B breaks the `.grid` inline-style assertions in two vitest files and the `.grid` computed-columns assertion in one e2e spec. The first two go red under B and the e2e one is predicted to.
- LB4: No e2e spec measures a caret move through real mouse or key input, and none asserts a time threshold. The nearest are the 12-9 timing, which only logs, and the `.grid-scroll` scrollTop checks in `segment-navigation`.

**1. Environment**
- Vitest runs under happy-dom (locator `vitest.config.ts:57`; I did not re-read it). `tests/AGENTS.md:10` says geometry, layout and real-engine claims belong to e2e. I did not re-read that line either.
- happy-dom has no layout, so none of the vitest tests measures time or geometry. Every "cost" test below counts calls or DOM mutations.

**2. The three render-cost tests (tree A)**

(a) `tests/frontend/gridCaretMoveCost.test.ts:111-123` (`GridPanel caret move cost`, first `it.each`) @ cc60020
- Probe: `vi.mock` of `../../src/panels/editorSegments` wraps `segmentRuleInputOf` with a counter (`:13-22`, `probe.ruleInputCalls += 1`).
- Mechanism: mounts the real `GridPanel` with `PanelFrame` stubbed as a plain slot div (`:69`, `mountGrid` `:91-100`); sets the caret to 100 and ticks, then resets the counter (`:114-116`); moves the caret to 101 and to 102, with a `$nextTick` after each (`:117-120`); asserts `ruleInputCalls === 0` (`:121`).
- Row counts: `it.each([20, 200])`, built by `many(n)` (`:79-89`). Segment ids run 100..100+n-1, and every third row has an empty target.
- Fails when: any caret move makes some code resolve a row's margin rule again, in any row. It does not fail if the rule is resolved elsewhere, or by a function other than `segmentRuleInputOf`.
- Does not measure: whether a row's template re-ran; whether `GridPanel`'s render function ran; the number of vnodes patched; any time or layout cost.
- Same file, `:125-305`: one test per value a row reads (caret `:125`, written text `:140`, `:152`, glossary mark `:164`, Han Viet tab `:179`, row error `:188`, `:201`, unlinked `:215`, selection `:230`, source cut `:245`, concordance `:257`, strip anchor `:270`). These guard that a `v-memo` dependency list is not missing a value, through DOM output only.

(b) `tests/frontend/gridTypingCost.test.ts:114-128` @ cc60020
- Probe 1: `probe.frameRenders`, incremented inside a custom `PanelFrame` stub's render function (`:58-67`). Probe 2: the same `segmentRuleInputOf` call counter (mock block above line 40, not read).
- Mechanism: sets the caret to 101, resets both counters (`:118-121`); types three strings by setting `textContent` and dispatching `input`, tick after each (`type` `:108-112`); asserts `frameRenders === 0` and `ruleInputCalls === 0` (`:124-125`) and `state.editorEditedText.get(101)` (`:126`).
- Rows: 20 and 200. Fails when a keystroke re-renders the `PanelFrame` slot content, or re-resolves a rule. The stub's render reads `slots.default()`, so it re-runs only if the slot's reactive dependencies change.
- Does not measure row re-renders or patches; never asserts anything about caret movement. `:130-175` are behaviour checks.

(c) `tests/frontend/gridRuleStable.test.ts:42-101` @ cc60020
- Probe: a `MutationObserver` on `.col-rule` with `{childList: true, subtree: true}` (`:50-52`).
- Mechanism: 3-row fixture; caret walk 13, 12, 11, null asserting the `rule-*` class list each step (`:54-60`); waits 20 ms, asserts no node added/removed (`:61-67`) and identical slot nodes (`:66`).
- Does not observe attribute changes. `:79-90` reads `src/panels/GridPanel.vue` CSS text and checks `border-left` widths.

**3. Can any test observe `v-memo`, render runs or patch counts? (LB1)**
- `git grep` for `v-memo|vMemo|onRenderTriggered|renderTracked|onUpdated|patchFlag|renderCount` over `tests/`, `e2e/` and `src/panels/GridPanel.vue` at cc60020 hit only the four `v-memo` attributes (`src/panels/GridPanel.vue:1819`, `:1841`, `:1856`, `:2019`). Nothing in `tests/` or `e2e/`.
- A memo miss appears only as an extra call to `segmentRuleInputOf`, if the rule is part of the missed row's work. A memo hit is never asserted.
- Under B the call-count probes would pass or fail by the same rule; I read B's diff, not B's test results.

**4. Tests reading DOM structure that tree B's block wrapper changes (LB3)**
- `tests/frontend/gridPanelStripTrack.test.ts:66-110`: `:69-70`, `:79-80`, `:98`, `:107` read `.grid` `style.gridTemplateRows` and expect `repeat(3|4, auto)`. Under B `.grid` has no inline style: four assertions fail. `.strip-pad`, `.col-*` children and `.strip-track` assertions (`:71`, `:81-89`, `:109`) are unaffected for a one-block chapter.
- `tests/frontend/gridPanelImages.test.ts:151-164`: `:155-157` reads `.grid` `gridTemplateRows`, fails the same way under B; `:159-163` passes for one block.
- `e2e/specs/story-12-7-workspace-grid-status-bar.e2e.mjs:70-85`: expects `getComputedStyle(.grid).gridTemplateColumns` to split into 4; predicted to change under B (not run).
- Unaffected or low-risk: `gridCaretMoveCost.test.ts:296-298`, `gridRuleStable` (single block), `editorTypingZone.test.ts:121-135` (global counts). No test read mounts more than 200 rows.

**5. Other coverage near caret**
- `tests/frontend/editorTypingZone.test.ts:105-113` places a caret via `Range` + `getSelection()` + `selectionchange`, covering the DOM-selection route to `setEditorCaret`.
- `proofreadWiring.test.ts:174-217`, `proofreadHighlight.test.ts:117`: not read.

**6. e2e: does any spec measure caret-move time? (LB4)**
- `e2e/specs/story-12-9-strip-row-track.e2e.mjs:44-83` (`focusAndSample`): `cell.focus()` then samples `requestAnimationFrame` gaps until `.tm-fuzzy-strip` presence matches for 5 frames; `-1` after 15 s. `:86-175` runs rounds, prints medians/maxima (`:169-174`), asserts nothing about them; assertions are `grow.firstExpected >= 0`, a ≤2 px geometry check, and unchanged `.grid-scroll` scrollTop (`:111`, `:126-128`, `:147`). Focus-driven, no threshold. Runs on the debug build (`:170`).
- `e2e/specs/segment-navigation.e2e.mjs:189-304`: scrollTop checks after `⌘⌥↓`, not timing.
- `e2e/support/gridWait.mjs:72-82`: count/wait helpers, 30 s timeout (`:64`).

**Looked for and not found**: any render-hook/patch-count API in tests/e2e; any e2e numeric time threshold on caret movement. Not read: `vitest.config.ts`, `tests/AGENTS.md`, `proofreadWiring.test.ts`, `proofreadHighlight.test.ts`, `gridTypingCost.test.ts:1-39`.
