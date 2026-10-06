---
ticket: 7
title: 'Story 11.7, lot A — one Settings frame, ⌘, and the shared UI foundation keep what they promise'
type: 'chore'
created: '2026-09-29'
status: done
route: 'dispatch'
baseline_revision: '32d933cc9ab412aaa41ba5037a6a8c11f4209559'
review_loop_iteration: 0
context:
  - '{project-root}/src/AGENTS.md'
  - '{project-root}/_bmad-output/implementation-artifacts/epic-11-context.md'
---

<frozen-after-approval reason="human-owned intent — do not modify unless human renegotiates">

## Intent

**Problem:** On HEAD `32d933c`, 33 open `deferred-work.md` items end in `Chủ: Story 11.7` (Task 0: `11-7-task0-2026-09-29.md`; `L…` numbers below are Task 0's). Lot A takes the 18 in groups A and B. Signed decisions #91, #92, #93 and #102 are not in the code: the Settings frame lists eleven sections of which three have a body, the Glossary scan-threshold and the Shortcuts screens are separate overlays, `⌘,` opens Shortcuts, and three overlays (plus the quick-add form) have no accessible name.

**Approach:** Lot A of two; lot B (`spec-11-7-lo-b-ai-module.md`, groups C and D) follows and the story goes `done` after it. Each item gets one disposition below. Every fix gets a guard, counter-checked by really removing its seam.

## Boundaries & Constraints

**Always:** Ledger items keep their text; only `→` lines are appended, a stale claim gets a 🔵 fix in place. `SETTINGS_SECTIONS` keeps the mockup order (`settings.html:145-154`). The gathered screens keep their state modules (`glossarySettingsState.ts`, `config/shortcutsState.ts`). `check:commands` stays green with every default chord unique. A check only a person can make in the real app goes to `Chủ: Epic 11`. Touching command registration ⇒ one full suite by hand.

**Never:** Add a gate, a dependency or a migration. Change Rust. Build user-named layout presets (L651). Change any AD; an item that needs one goes to `Chủ: Winston`. Write decision numbers, dates or provenance into code comments.

## Dispositions

Agent, 2026-09-29 (from Task 0 and a re-read at HEAD; Ice may override any line at approval):
1. ✅ fix, webview, Settings frame (#91, #92, #102): L9475 + L9489 — the nav shows only sections with a body, in mockup order, `privacy` last (the mockup has no slot for it): `ai_and_model`, `prompt`, `glossary`, `shortcuts`, `privacy`. The empty-section owner label and `settings.nav.no_body_yet` go. The bodies of `GlossarySettingsOverlay.vue` and `ShortcutsOverlay.vue` move into Settings sections as child components; both overlays and their overlay-open predicates are deleted. `glossary.settings.open` and `shortcuts.open` stay as commands that open Settings at their section; their two titlebar buttons (`App.vue:272`, `:288`) are removed, the Settings button stays. Task 0 group A listed only `privacy` as having a body; `ai_and_model` and `prompt` also have one (`settingsState.ts:99`).
2. ✅ fix, webview, chords (#102): L7972 — `Mod+Comma` moves to `settings.open`, replacing `Mod+Alt+Comma`; `shortcuts.open` gets no default chord (reached through Settings). `reading.toggle_tuner` is unchanged.
3. ✅ fix, webview, a11y (#93): L12044 — `aria-labelledby` on `PromptLibraryOverlay`, `PromptImportOverlay`, `GlossaryImportOverlay`, and on the rebuilt Settings dialog, after the `AiPromptInspectorOverlay` pattern; L5435 — the same on the `GlossaryQuickAdd` form. One shared vitest.
4. ✅ fix, webview: L12459 — the dock remembers each group's size across a tier round trip and re-applies it under persist suppression; the real-build sash check becomes one `Chủ: Epic 11` item.
5. ✅ test-only: L1457 (a source-scan guard: `applyBindings(` is called only from `commitBindings`, the only `bindingsEpoch.value +=`), L1468 (a vitest pins "UI capture cannot enter Escape; `editor.clear_source_cuts` still defaults to Escape"; the ledger sentence gets a 🔵 fix).
6. Self-closed with a pointer: L682 (Story 4.1 moved `PanelFrame` `.status` to `ui-md-wrap`).
7. `KHÔNG LÀM` with a reopen condition: L673 (the product ships coloured tab groups), L689 (a `dockview-vue` bump whose `VueComponent` is no longer `any`-parameterised), L1480 (Sally wants the glyph in the copy).
8. Reassign: L651 → `Chủ: John` (needs an FR first); L1219 → `Chủ: Winston` (one cell in the spine's Capability Map C3 row).

Ice, 2026-09-29:
9. L308: a per-`(kind, key)` promise queue in `putConfig` (`config/bootstrap.ts`); a vitest with an out-of-order `invoke` mock asserts the last value lands last.
10. L2100: `KHÔNG LÀM`; reopen when a real focus loss to `body` is reported.
11. L4347: `KHÔNG LÀM`; the two commands keep 0 default chords, assignable in Settings → Shortcuts.
12. L7194 (shape 2): one `unhandledrejection` listener in `main.ts` logs the leaked rejection; Ice approves the new `check:layout` Kiểm C allow-list name it needs. A vitest dispatches a rejecting-after-`await` command and asserts the log.

</frozen-after-approval>

## Code Map

- `src/settingsState.ts` -- `SettingsSection`/`SETTINGS_SECTIONS` :38-63, `settingsSectionHasBody` :99, `settingsSectionOwnerLabel` :103+, provisional-order doc-comment :1-35 (rewrite), `openSettings`, `openSettingsToPrivacy`.
- `src/SettingsOverlay.vue` -- nav `v-for` ~:244, AI section :261, prompt :407, body-bearing branch :421, empty body :489; lot B adds the tier selector to the AI section.
- `src/GlossarySettingsOverlay.vue` (307 lines) + `src/glossarySettingsState.ts:91-138`; `src/ShortcutsOverlay.vue` (569 lines; Escape capture :167, `onEscape` :132) + `src/config/shortcutsState.ts` (`bindingsEpoch` :76, `commitBindings` :413-421).
- `src/App.vue` -- titlebar buttons :272 (shortcuts), :288 (glossary settings), :330 (settings); return-focus :168; overlay mounts :396-420.
- `src/main.ts` -- overlay-open predicate :1125-1129; CommandDeps `openShortcuts`/`openGlossarySettings`/`openSettings` :846, :868, :937.
- `src/commands/index.ts` -- `settings.open` :3426 (`Mod+Alt+Comma`, comment :3416-3424), `shortcuts.open` :3745-3747 (`Mod+Comma`), `glossary.settings.open` :3074, `reading.toggle_tuner` comment :2005, `editor.next_segment`/`prev_segment` :2750-2771, `applyBindings` :3939; `src/commands/registry.ts` `run` type :51, `dispatch` :200-245; `src/commands/focus.ts` `armBodyGuard` :160-172, `enter` :174; `src/commands/keys.ts` `createKeymap` duplicate throw ~:486.
- `src/config/bootstrap.ts:315-335` `putConfig`; `src/main.ts:1281-1287` mode watch.
- a11y: `src/PromptLibraryOverlay.vue:451`, `src/PromptImportOverlay.vue:112`, `src/GlossaryImportOverlay.vue:112`, `src/GlossaryQuickAdd.vue:112-123`; pattern `src/AiPromptInspectorOverlay.vue:152-154`, test pattern `tests/frontend/aiPromptInspector.test.ts` (B15).
- Dock: `src/layout/WorkspaceDock.vue` (`RememberedSpot` :180, `rememberSpot` :351, `showPanel` :381, tier merge :630-680, persist suppression :485/:807); `tests/frontend/workspaceDockTier.test.ts` (Case D).
- i18n `src/i18n/vi.json` (`command.settings.open` :350, `command.glossary.settings.open` :301, `settings.nav.*`, `shortcuts.gesture` :784); gate `scripts/check-commands.mjs` (`COMMAND_FLOOR` :237).
- Tests touching the moved overlays: `grep -ln "GlossarySettingsOverlay\|ShortcutsOverlay" tests/frontend e2e/specs` before moving.

## Tasks & Acceptance

**Execution:**
- [x] `src/**`, `vi.json`, `tests/frontend/*`, `e2e/specs/*` -- Dispositions 1-5 and Decisions 9 and 12 -- Webview phase.
- [x] Real-removal counter-check for every new guard; the full suite once (command registration changes) -- Tests phase.
- [x] `deferred-work.md` -- one `→` disposition for each of the 18 items; the new `Chủ: Epic 11` sash item -- Ledger phase.

**Acceptance Criteria:**
- Given lot A is done, when the 18 item lines are read, then each ends in one `→` disposition and `npm run check:debt-owner` is green.
- Given Settings opens, when its nav is read, then it lists exactly `ai_and_model`, `prompt`, `glossary`, `shortcuts`, `privacy` in that order, and each opens a body.
- Given any surface, when `⌘,` is pressed, then Settings opens; when `glossary.settings.open` or `shortcuts.open` is dispatched, then Settings opens on that section and no separate overlay mounts.
- Given the Shortcuts section, when a chord is reassigned, then `bindingsEpoch` bumps once and the new chord dispatches.
- Given each of the four dialogs and the quick-add form open, when `aria-labelledby` is resolved, then it names an element with the visible title.
- Given a sash dragged to a non-default ratio, when the layout goes narrow then wide, then each group's remembered size is re-applied and no persist fires in between.
- Given each new guard, when its seam is really removed, then it goes red for that reason.

## Implementation Notes

- The two gathered overlays became `SettingsGlossarySection.vue` and `SettingsShortcutsSection.vue`; `shortcuts.close` and `glossary.settings.close` were deleted with them (nothing dispatches them any more). Saving the Glossary threshold no longer closes anything: it shows `glossary.settings.saved`.
- `Escape` while a chord capture is armed cancels the capture before it can close Settings; the first guard for this stayed green under removal because the fallback path also cancelled (with a wrong notice), so the case now also asserts the notice is null.
- `check:commands` does not judge which command holds a chord, so moving `Mod+Comma` back to `shortcuts.open` stays green there; only `settingsFrame.test.ts` guards the chord.
- Decision 12 needed no new `check:layout` name: `window.addEventListener`/`removeEventListener` were already allowed.
- The phase's new files tipped `check:doc-refs`'s `judgeFloor` (369 < 80% of 464); `FILE_FLOOR` was raised to 395. The Webview agent first reported it red before the phase, but `git stash` had kept its untracked files; a clean baseline worktree is green.
- The sash fix is bookkeeping-tested only (D2 fakes the group size on the dockview prototype); dockview clamping on `setSize` and non-LIFO hide/show are the `Chủ: Epic 11` hand item, with the 920px Settings width for the shortcuts table.
- Under machine load the default 5000 ms vitest timeout failed the first case of three heavy-import files (two of them new); a quiet re-run was green. Debt item `Chủ: Murat`.

## Spec Change Log

## Review Triage Log

- VG1 Glossary section form wiring (`@input`, submit, `.gs-saved`) never mounted in a vitest — medium, patch: removing `@input`/`@submit` stays green.
- VG2 `undoMerge`'s `reapplySize` unexercised (D2 reaches only `showPanel`) — medium, patch.
- VG3 other `role="dialog"` surfaces unnamed; `dialogAccessibleName.test.ts` header claims "every" — low, patch the header; the adoption itself is pre-existing ⇒ defer (`Chủ: Sally`).
- VG4 `unhandledRejectionLog` wiring case scans 200 raw chars, a commented call passes — low, patch (comment-stripped scan).
- VG5 `Escape` on the Settings frame (no capture ⇒ `settings.close`; armed ⇒ cancel only) untested — medium, patch.
- VG6 / BH comments: `commands/index.ts` names `shortcutsState.ts` twice; `check-panel-refs.mjs` cites deleted `overlayOpen`; `WorkspaceDock.vue` docblock now sits above `groupSizeOf`; `putConfig` doc says Settings is a later story — low, patch (direct corrections).
- BH1 / ECH saved keymap: a user override holding `Mod+Comma` now clashes with the new default and the whole saved keymap falls back (not deleted, notice shown) — low for this lot: Ice's real `global.db` holds 0 shortcut rows; the whole-keymap fallback policy is pre-existing ⇒ defer (`Chủ: Ice`). Overrides naming the two deleted ids are ignored by `createKeymap` (iterates the registry) — false.
- BH2 `bootstrap.test.ts` second case asserts exact microtask order `['a','c','b']` — low, patch (assert only `b` after `a`, `c` not waiting on `a`).
- BH3 / ECH final `v-else` in `SettingsOverlay.vue` renders Privacy for any unhandled section — low, patch (`v-else-if` privacy); BH4 the nav test's `ai_and_model`/`privacy` markers are one selector and cannot catch it — low, patch (unique markers).
- BH5 / ECH closing or remounting the Glossary section during an in-flight save resets the input and clears the error — low, patch (`loadGlossarySettingsForm` returns while saving); an error lost when closing before the save completes stays — low, rejected (local write, same class as any overlay closed mid-write).
- BH6 invalid-threshold alert has no `role`, `role="status"` node created with its text, `saveError` survives an edit — false for this lot: all three are carried over unchanged from `GlossarySettingsOverlay.vue`.
- BH7 / ECH `putConfig` queue does not serialise `deleteConfig` — low, rejected: pre-existing path, and the race itself is unproven (sync command).
- BH8 / ECH remembered group size is absolute pixels, stale if the window changed size while hidden — maybe-false (medium if true), defer: added to the `Chủ: Epic 11` real-build check.
- BH9 / ECH `bindingsEpochWiring` misses `bindingsEpoch.value++` / `= … + 1` — low, patch (widen the regex); arrow-function attribution and double-quoted strings — low, rejected (needs an AST).
- BH10 unhandled-rejection listener not idempotent — false: `boot()` runs once per page load.
- BH11 ledger closures: "hàng 2026-09-29" has no pointer; e2e closure names no build — low, patch (cite the spec decision numbers and `cargo build --features wdio`, local macOS run 2026-09-29).
- BH12 e2e 3.5 dropped the `.gs-save` driver-click fallback — low, patch (retry via `dispatchPointerSequence` when `.gs-saved` is absent); its title's "Settings blocks global shortcuts" claim — false: `settingsOverlayIsOpen` is in `main.ts` `isBlocked`.
- BH13 `FILE_FLOOR` 369→395 unexplained — false: the gate prints the rule (`ceil(0.85 × live)`), and raising a floor tightens it.
- BH14 `App.vue` still describes `ShortcutsOverlay` — false: the comment now names `SettingsOverlay.vue`/`data-settings-open`.
- ECH1 `putConfig` lane poisoned by a rejected tail — false: `sendPut` is an `async` try/catch and never rejects.
- ECH2 showPanel on a user toggle persists the re-applied size — false: a user show is a user layout change and should persist.
- ECH3 fallback placement (anchor gone) drops the remembered size — low, rejected: no remembered anchor, nothing to restore against.
- ECH4 quick-add `<form>` needs `role` for its name — false: a named `<form>` maps to the `form` role.
- ECH5 static ids could duplicate with two instances — low, rejected: each surface mounts once.

## Verification

**Commands:**
- `npm run test:story 11-7 -- --list` then the selected vitest files -- green.
- `npm run check:commands && npm run check:i18n && npm run check:tokens && npm run check:debt-owner` -- green.

**Manual checks (if no CLI):**
- Real build: sash ratio survives narrow→wide (the `Chủ: Epic 11` item).

## Acceptance criteria from epics.md

Source: `epics.md` §Story 11.7 (v6, nay ở archive-v6).

As a chủ dự án,
I want nợ nhóm nền giao diện dùng chung (lệnh, tiêu điểm, phím tắt, a11y) và module AI (Epic 4) được đóng hoặc quyết dứt điểm,
So that các bề mặt mới của Epic 7–9 dựng trên nền lệnh và tiêu điểm đã vá.

Thừa kế AC chung của epic (xem Notes của `epic-tra-no-nen.md`).

**Acceptance Criteria:**

**Given** mọi mục mang `Chủ: Story 11.7` trong `deferred-work.md`
**When** story hoàn tất
**Then** thoả AC chung của Epic 11; AC riêng rút từ các mục lúc `create-story`
