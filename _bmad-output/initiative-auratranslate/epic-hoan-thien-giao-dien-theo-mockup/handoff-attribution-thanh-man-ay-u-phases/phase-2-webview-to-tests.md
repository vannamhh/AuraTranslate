# Webview -> Tests handoff (story attribution-thanh-man-ay-u, loop 2)

Uncommitted. Step 0 `git apply` of the loop-2 patch done (it also pre-modified `tests/frontend/attributionOverlayActions|Unreadable|titlebarToolsMenu.test.ts` and `e2e/specs/attribution-focus.e2e.mjs`; those are NOT yet reworked to the plan's test deltas). Webview deltas applied on top. Green: vue-tsc, eslint on touched src files, check:commands/i18n/tokens/panel-refs/layout (exit 0). `grep attr-scrim src` empty.

## Webview deltas done
- `App.vue`: `div.stage` (relative, flex:1, min-height:0, display:flex column) contains `.modeport` (flex:1, min-height:0, isolation kept), `GlossaryQuickAdd`, `ProofreaderStrip place="foot"`, `StatusBar`, then `<AttributionOverlay />` last. `LookupDrawer` and `pre.selftest` stay outside. `.modeport`/`.selftest` comments fixed (English line about `.attr-screen` sibling, no z-index).
- `commands/index.ts`: `mode.*` run calls `deps.closeAttribution?.()` before `setMode(mode)`. `modeState.ts`, `main.ts` untouched.
- `AttributionOverlay.vue`: removed `watch(currentMode)`, `skipFocusReturn`, `currentMode` import. Close branch of the open watch: after sync `back.focus()` a `nextTick` re-focus if `back.isConnected && activeElement !== back`. Foot copy error shown iff `attributionCopyFailedCode !== null && === attributionAimedCode`. CSS: `.attr-table td:first-child { border-left: 2px solid transparent }` for all rows; `tr.attr-aimed td:first-child { border-left-color: var(--color-primary) }`, no padding. `focusableWithin` doc replaced by one English line.
- `layout/LookupDrawer.vue`: `v-show="!attributionIsOpen"` on `div.ld-scrim` (next to `v-if`), import from `../panels/dictSourcesState`; comments English.
- `commands/keys.ts` comment now English, names `.attr-screen`. `GlossaryQuickAdd.vue`, `StatusBar.vue` comments updated to `.stage`.
- Unchanged from patch: `TitlebarToolsMenu.vue`, `dictSourcesState.ts`, `vi.json`.

## Current vitest state (3 files)
39 pass, 1 red: `attributionOverlayActions.test.ts` "changing mode while open closes the screen..." — it uses `setMode`, which no longer closes the screen. Rework per plan task (5): click other tab then active tab via `dispatch('mode.*')`; harness `installCommands` must inject real `closeAttribution` (the same closer `main.ts:924` injects).

## Tests still to do (plan Tasks, test section)
1. Decision 6 test: licence mock returns `LICENSE BODY <code>` per code; assert `attributionLicenseView.value?.code` and DOM text; no-licence row => viewer closed and `document.activeElement` is `tr.attr-aimed`.
2. Late clipboard reject: manually controlled promise; click copy, aim another row, then reject => no `.attr-error` in foot.
3. Placement as single-assert `it`s reading source minus comment lines: `.attr-screen` child of `.stage` after `.modeport`; CSS `position: absolute`; no `position: fixed`; no `--space-titlebar-height`; no `z-index`; `tr.attr-aimed` rule has no `padding`/`box-shadow`; `td:first-child` border-left same width in base and aimed rules.
4. App-mount harness: stub three strips as `<div data-stub="quick-add|proofreader|status">`; all inside `.stage` before `.attr-screen`; `LookupDrawer` outside `.stage`.
5. Mode tabs (other tab and the active Workspace tab) via `dispatch('mode.*')`.
6. Separate `expect`s for `.layout-switch-btn` and `[data-tools-trigger]` disabled.
7. LookupDrawer case (stub `LookupPanel`): drawer open + `attributionIsOpen` => `.ld-scrim` display none; close screen => visible and focus on the opener button.
8. e2e `attribution-focus.e2e.mjs`: remove the comment `Focus must be inside the screen, which no longer declares aria-modal.`; keep Escape comment to one English line; no new steps. Run it and `glossary-quick-add.e2e.mjs` outside sandbox (`.modeport` now flex:1 child of `.stage`, not measured in a real webview).
9. Counter-checks listed in the plan Verification (each a real removal; note the red `it`). Note `attribution-focus` e2e text ("no aria-modal") should be checked against current DOM.
