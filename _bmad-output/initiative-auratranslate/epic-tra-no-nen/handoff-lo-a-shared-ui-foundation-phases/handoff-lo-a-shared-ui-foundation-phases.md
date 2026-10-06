---
type: handoff
title: "Story 11.7 lot A — Phase handoff notes"
status: done
created: 2026-09-29
skill: bmad-build
---

# Story 11.7 lot A — Phase handoff notes

Spec: `spec-11-7-lo-a-shared-ui-foundation.md`. Task 0: `11-7-task0-2026-09-29.md` (groups A and B). Baseline `32d933cc9ab412aaa41ba5037a6a8c11f4209559`.

Each phase appends one `## Phase N — <name>` section and ends it with a `### Remaining for the next phase` list. A later agent reads the spec, then only the latest section of each earlier phase.

## Phase 1 — Webview

Done: dispositions 1-4, decisions 9 and 12, and the guard tests for 1-5 (written, not yet counter-checked except the dock one). Baseline `32d933c`; nothing committed.

### What changed
- Settings frame: `SETTINGS_SECTIONS`/`SettingsSection` are now the five sections with a body (`ai_and_model`, `prompt`, `glossary`, `shortcuts`, `privacy`); `settingsSectionHasBody`, `settingsSectionOwnerLabel`, the empty-body branch and `settings.nav.no_body_yet` (plus six dead `settings.nav.*` keys) are gone. New `openSettingsToSection(section)` in `src/settingsState.ts`; `openSettingsToPrivacy` delegates to it. Nav buttons carry `data-settings-section` (used by e2e).
- `src/GlossarySettingsOverlay.vue` -> `src/SettingsGlossarySection.vue` and `src/ShortcutsOverlay.vue` -> `src/SettingsShortcutsSection.vue` (`git mv`, rewritten as single-root bodies; headings and intro stay in `SettingsOverlay.vue`). Both overlays, `glossarySettingsOverlayIsOpen`, `shortcutsOverlayIsOpen`, their mounts in `App.vue`, and the `main.ts` `isBlocked` term for Glossary settings are deleted. Shortcuts section calls `enterShortcutsScreen` on mount and `leaveShortcutsScreen` on unmount (was `openShortcuts`/`closeShortcuts`); Glossary section calls `loadGlossarySettingsForm` on mount; save no longer closes anything, it sets `glossarySettingsSaved` and shows `glossary.settings.saved` (new i18n key). `SettingsOverlay.onEscape` cancels a pending capture before `settings.close`.
- Commands: `Mod+Comma` -> `settings.open` (`Mod+Alt+Comma` dropped); `shortcuts.open` has no default chord; `shortcuts.close` and `glossary.settings.close` are deleted (deps `closeShortcuts`/`closeGlossarySettings` too). `main.ts` wires `openShortcuts`/`openGlossarySettings` to `openSettingsToSection`. Titlebar buttons for shortcuts and Glossary settings removed. `editorClearSourceCuts.ts` now guards on `settingsOverlayIsOpen`.
- a11y (#93): `aria-labelledby` + title `id` on `PromptLibraryOverlay` (`pl-title`), `PromptImportOverlay` (`pi-title`), `GlossaryImportOverlay` (`gi-title`), `SettingsOverlay` (`settings-title`), and the `GlossaryQuickAdd` form (`gqa-title`).
- Dock (L12459): `RememberedSpot` gained `size`; `rememberSpot` records the group's width/height when positive and finite; `showPanel` and `undoMerge` re-apply it via `reapplySize` right after `addPanel` (inside the existing persist suppression of `autoShow`/`undoMerge`).
- L308: `putConfig` in `src/config/bootstrap.ts` chains per `(kind, key)` (`putTails` map, self-clearing; EXEMPT entry added in `scripts/check-panel-refs.mjs`).
- L7194: `src/unhandledRejectionLog.ts` (`installUnhandledRejectionLog`, called first in `boot()`), uses `window.addEventListener('unhandledrejection', …)`. `window.addEventListener`/`removeEventListener` are already in `check:layout` Kiểm C's allow-list, so NO new allow-list name was needed and `scripts/check-layout.mjs` is untouched; Ice's approval item is moot.
- Gate tables: `scripts/check-commands.mjs` HANDLER_TABLE re-keyed to the new files (`SettingsOverlay.vue::onEscape`, `SettingsShortcutsSection.vue::aimRowFrom`/`onKeyCellKeydown`); `src/panels/README.md` two names updated.
- e2e (edited, NOT run): `shortcuts-focus`, `shortcuts-capture-mouse`, `story-3-5-review` now open Settings through `[data-settings-open]` and the `[data-settings-section=…]` nav button; `.sc-panel`/`.gs-panel` selectors became `.set-panel`; the Glossary spec waits for `.gs-saved` instead of a closing modal.

### New / changed tests (all green)
- New: `tests/frontend/settingsFrame.test.ts` (nav order, each entry has a body, `openSettingsToSection`, `Mod+Comma` chord + dispatch, `shortcuts.open` unbound, Escape default kept by `editor.clear_source_cuts`, reassign bumps once and new chord dispatches, Escape while capturing assigns nothing, leaving the section drops a capture), `dialogAccessibleName.test.ts`, `bindingsEpochWiring.test.ts` (source scan, comments and string literals stripped), `unhandledRejectionLog.test.ts`.
- Changed: `bootstrap.test.ts` (+2 putConfig ordering cases), `workspaceDockTier.test.ts` (+case D2, fakes group size on the dockview prototype), `glossarySettings.test.ts`, `settingsState.test.ts`, `editorClearSourceCuts.test.ts` (case 7b now opens Settings > Shortcuts).
- Verified: `vue-tsc`, `eslint src e2e tests`, `check:commands`, `check:i18n`, `check:tokens`, `check:panel-refs`, `check:layout` green; `vitest related` over the changed sources: 80 files / 1104 tests green. Only counter-check done: emptying `reapplySize` turned D2 red (`expected 0 to be 2`), then restored.
- `check:doc-refs` is red on HEAD too (`FILE_FLOOR` 369 vs 464 live files); not caused by this phase, not touched.

### Remaining for the next phase
- Tests phase: counter-check every guard by really removing its seam, one target each: swap two entries or re-add an empty section in `SETTINGS_SECTIONS` (nav-order case); put `Mod+Comma` back on `shortcuts.open` (`check:commands` + chord case); delete `aria-labelledby` from one overlay (only that `dialogAccessibleName` case red); move the `applyBindings(` call out of `commitBindings` (`bindingsEpochWiring`); make `putConfig` call `sendPut` directly (`bootstrap.test.ts` ordering case); remove the `unhandledrejection` listener or the `boot()` call; remove the `event.code === 'Escape'` branch in `SettingsShortcutsSection.vue::onKeyCellKeydown` (Escape-capture case); drop `settingsOverlayIsOpen.value ||` from `editorClearSourceCuts.ts` (case 7b).
- Then the one full suite by hand (command registration changed; `npm run build` first). Unknown until then: order-dependent failures outside the 80 related files.
- Ledger phase: dispositions per spec (L9475/L9489/L7972/L12044/L5435/L12459/L308/L7194 closed; L1457/L1468 closed by tests, L1468 sentence gets a 🔵 fix; L682 self-closed; L673/L689/L1480/L2100/L4347 KHÔNG LÀM; L651 -> John; L1219 -> Winston) plus the new `Chủ: Epic 11` item for the real-build sash check (narrow -> wide, `workspace_layout` unchanged). Also worth an `Epic 11` hand item: e2e specs edited here were never run, and real-app check that `⌘,` opens Settings and focus returns to the titlebar button.
- Risks to flag: dockview may clamp or rescale on `setSize` when siblings change (only measurable on a real build); non-LIFO hide/show orders restore a size taken at hide time; Settings panel max-width is 920px (the old shortcuts overlay was 1100px) so the shortcuts table is narrower.

## Phase 1b — orchestrator note

- Phase 1's claim "`check:doc-refs` was already red before this phase" is wrong: `git stash` kept the phase's untracked files, so the "before" run still counted them. A clean worktree at the baseline is green. The phase's new files tipped `judgeFloor` (369 < 80% of 464), so `FILE_FLOOR` in `scripts/check-doc-refs.mjs` was raised to 395 (`ceil(0.85 × live)`, the gate's own instruction). This touches `scripts/check-*`: the full suite is due once anyway.
- Phase 1 also deleted `shortcuts.close` and `glossary.settings.close` (their overlays are gone); the Tests phase confirms nothing still dispatches them, including `e2e/`.

### Remaining for the next phase

- Real-removal counter-check for each guard Phase 1 added or changed (Phase 1 ran only the dock one), each re-run on its own target.
- Run the three edited e2e specs (`shortcuts-focus`, `shortcuts-capture-mouse`, `story-3-5-review`) if the e2e runner can select them; otherwise say so.
- The full suite once, by hand (command registration and `scripts/check-*` changed).

## Phase 2 — Tests

Done: real-removal counter-check of every Phase 1 guard, e2e run, the full suite. One guard was blind and was fixed; no production code changed.

### What changed
- `tests/frontend/settingsFrame.test.ts`: the Escape-capture case now also asserts `shortcutNotice` is null. Before, removing the `event.code === 'Escape'` branch in `SettingsShortcutsSection.vue::onKeyCellKeydown` left it green (Escape fell through to `handleCaptureKey`, which also cancels and stops propagation, but shows a wrong `shortcuts.key_unknown` notice). Now red.
- `scripts/`, `src/`: untouched (every mutation was restored; no `.ccbak` left).

### Counter-checks (each a real removal, re-run on its own target, all red for the intended reason)
- Swap `glossary`/`shortcuts` in `SETTINGS_SECTIONS`: `settingsFrame` 2 red (order, per-entry body).
- `Mod+Comma` moved from `settings.open` to `shortcuts.open`: `settingsFrame` 2 red (incl. "pressing ⌘, dispatches settings.open"). `check:commands` stays GREEN — it does not judge which command holds the chord, so the chord guard is the vitest alone.
- `aria-labelledby` removed from each of the five surfaces in turn: `dialogAccessibleName` exactly 1 red each (the matching case).
- `applyBindings(` moved out of `commitBindings` behind a wrapper: `bindingsEpochWiring` 2 red. A second `bindingsEpoch.value +=` in `commitBindings`: 1 red.
- `putConfig` calling `sendPut` directly: `bootstrap.test.ts` 2 red (the two ordering cases).
- `unhandledrejection` listener removed: `unhandledRejectionLog` 1 red; `installUnhandledRejectionLog()` removed from `boot()`: 1 red.
- Escape branch removed: see above (was green, now red after the assert). `settingsOverlayIsOpen` dropped from `editorClearSourceCuts.ts`: `editorClearSourceCuts` case 7b red.
- The dock guard (D2) was counter-checked in Phase 1.
- Nothing dispatches `shortcuts.close` / `glossary.settings.close` (grep over `src`, `e2e`, `tests`, `scripts`: only comments name them).

### Full run (by hand; build, then all)
- 12 gates green (`check:doc-refs` green with `FILE_FLOOR` 395). `npm run build` green.
- vitest: first run 3 failed / 1605 passed, all three `Test timed out in 5000ms` (`dialogAccessibleName` PromptLibraryOverlay, `settingsFrame` nav order, `glossaryMarksRefresh` 3.4b), cold-import timeouts while the machine was loaded (cargo was at 20% cpu). The three files alone: green twice. Second full run on a quiet machine: 116 files, 0 failed (exit 0).
- cargo: the run hit the 1800 s cap at `project_contract` on a loaded machine; `project_contract` alone: 138 passed in 50 s; the 22 targets after it alphabetically ran in one command: all green (`webimport_boundary` 41 passed, 5 ignored). No Rust file was changed by this story. Nothing was marked passed by inference, but the cargo result is stitched from three invocations, not one uninterrupted `cargo test --locked`.
- e2e (`cargo build --features wdio` then `wdio --spec` per file, macOS WKWebView): `shortcuts-focus` 1 passing, `shortcuts-capture-mouse` 2 passing, `story-3-5-review` 1 passing.

### Remaining for the next phase
- Ledger phase as listed in Phase 1 (dispositions for the 18 items; the `Chủ: Epic 11` items: real-build sash narrow -> wide; real-app `⌘,` opens Settings and focus returns to the titlebar button). The e2e note in Phase 1 is now moot (the three specs ran green).
- Flag for Ledger: the 5000 ms default timeout fails the first case of heavy-import test files under machine load; not a code fault, not fixed.

## Phase 3 — Ledger

Done: one `→` disposition appended to each of the 18 lot A items in `deferred-work.md`, plus 3 new items at the end. Only `deferred-work.md` changed; `npm run check:debt-owner` green (Kiểm A, B, C).

### What changed
- Closed `✅ ĐÃ ĐÓNG`: L308, L682 (self-closed, Story 4-1), L1457, L1468, L5435, L7194, L7972, L9475, L9489, L12044, L12459. Each names its guard and the counter-check result from Phase 1/2.
- `KHÔNG LÀM` with a reopen condition: L673, L689, L1480, L2100, L4347.
- Reassigned by a new last `Chủ:`: L651 -> John, L1219 -> Winston.
- In-place 🔵 fixes: L682's 2026-09-23 audit line (its "still true" claim was a misread doc-comment; the original line is replaced by itself plus the 🔵 sentence, the only removed diff line) and L1468's claim sentence (new 🔵 line before the two `→` lines).
- New items, all at file end: two `Chủ: Epic 11` hand checks (real-build sash narrow -> wide and non-LIFO hide/show; real-app `⌘,` + focus return + 920px shortcuts table), and one `Chủ: Murat` item for the 5000 ms vitest timeout under machine load (not fixed).
- L7972's closure records that `check:commands` stays green when `Mod+Comma` is on the wrong command, so the chord guard is the vitest alone.

### Remaining
- Lot B (`spec-11-7-lo-b-ai-module.md`) and its 15 remaining `Chủ: Story 11.7` items; the story goes `done` only after it. Nothing committed for lot A (Ice's dirty-tree rule: commit is Ice's call).
- Spec `## Implementation Notes` is still empty; the spec status is not changed by this phase.
