---
title: 'Story 4.12: Narrow-window layout and threshold calibration'
type: 'feature' # feature | bugfix | refactor | chore
created: '2026-09-22'
status: 'done' # draft | ready-for-dev | in-progress | in-review | done
route: 'dispatch' # oneshot | dispatch
baseline_commit: 'bb902f9fe190b782f8221e2becb1f2e337a8b071'
review_loop_iteration: 0
context:
  - '{project-root}/AGENTS.md'
  - '{project-root}/src/AGENTS.md'
  - '{project-root}/tests/AGENTS.md'
---

<frozen-after-approval reason="human-owned intent — do not modify unless human renegotiates">

## Intent

**Problem:** Story 1.14 delivered the panel-sacrifice *mechanism* as pure functions and nothing
else: no code anywhere in `src/**` reads a window size, so on a small laptop all three panels stay
mounted and the Original|Translation grid is squeezed until translating is impractical. UX-DR15's
four thresholds (`[A11]`, Q9) are still mockup guesses — and at the shipped default window
1280×860 the work area is already 786 px tall, below UX-DR15's own 820 threshold, so the seeded
numbers cannot be trusted without measuring.

**Approach:** Add the missing half: a pure four-tier ladder keyed by layout preset in
`src/layout/workspaceLayout.ts`, one window-size observer in the dock shell that feeds it, and the
two surfaces the tiers need (Lookup's retreat, the unsupported notice). Then calibrate the numbers
on real hardware with all three panels carrying content, separately for Ⓑ-1 and Ⓑ-2, write the
result into `SPEC.md [A11]` and close Q9.

## Boundaries & Constraints

**Always:**

- The sacrifice ORDER is a decision, not a number. `SACRIFICE_ORDER` (AI yields first, Lookup
  second) and `NEVER_SACRIFICED` (`['panel.grid']`) keep their present contents; calibration may
  change only the four sizes. `check-layout.mjs` Kiểm A already enforces this against the live
  module — it must stay green untouched.
- 🔴 **An automatic tier change must never be persisted as the user's layout.** `WorkspaceDock.vue`
  writes `api.toJSON()` through the debounced schedule on every `onDidLayoutChange`; an auto-hide
  that reaches that path destroys the arrangement the user chose, silently and permanently.
  Restoring the window must restore the panels.
- The grid never yields at any size, including the unsupported tier. The frozen user story says a
  narrow window is an annoyance, not a stop — so the unsupported notice does not block the app.
- Lookup never disappears entirely — the sacrifice list is not a licence for it to vanish.
- The threshold module stays in the pure tier: no imports, no DOM, loadable by plain Node, because
  `check-layout.mjs` `import()`s it for real. All window reads live in the dock shell.
- Work-area height is window height minus the two shipped chrome tokens, **read from the tokens at
  runtime, not written as a literal** — they are `titlebar-height: 40px` and `status-height: 34px`
  today, so 74 px today, and UX-DR15's 38/32 is stale prose that no longer matches the product.
  Work-area width is the window width, unadjusted.
- Every new UI sentence goes through `vi.json` + `t()`; no numeral is formatted in the webview.

**Never:**

- Do not edit `DESIGN.md` or `epics.md` to match the code. `StatusBar.vue` §AC9 sets the precedent
  for exactly this divergence: the mismatch is recorded in `deferred-work.md` with Ice as owner and
  the dev does not touch the planning doc. `SPEC.md [A11]` is the one planning anchor this story
  does write, because closing it is the story's own acceptance criterion.
- Do not add a second OS window, a second store, or `localStorage` — `check-layout.mjs` Kiểm C
  bans all three by name.
- `src-tauri/**` is touched for exactly one line, `minWidth` (Decision 1). Nothing else there
  moves, and no Rust code learns about window size.
- Do not assert geometry in vitest. `happy-dom` computes no layout; its `ResizeObserver` shim is a
  no-op. Number-in / tier-out is a vitest claim; pixels on screen are a manual claim.

## I/O & Edge-Case Matrix

The ladder is total and evaluated top-down, so exactly one tier matches. Inputs are work-area
`(width, height)` in CSS px and the active `PresetId`.

| Scenario | Input / State | Expected Output / Behavior | Error Handling |
|---|---|---|---|
| Full layout | `w ≥ 1100`, `h ≥ 820` | Chosen preset, all three panels mounted | N/A |
| Short window | `w ≥ 1100`, `700 ≤ h < 820` | Lookup and AI Translation merge into one tabbed panel | N/A |
| Narrow or very short | `860 ≤ w < 1100`, or `h < 700` | Grid only; AI Translation hidden; Lookup leaves the panel grid and its entry point appears in the status bar | N/A |
| Unsupported | `w < 860` | Non-blocking notice; grid still rendered and usable | N/A |
| Lookup reached from the status bar | Any tier where Lookup has retreated | Activating the status-bar entry point slides the drawer open over the grid; closing it returns focus where it came from | N/A |
| Boundary exactly on a number | `w = 1100`, `h = 820` | Full layout — the bounds are inclusive | N/A |
| Preset switched at a fixed size | Same `(w, h)`, Ⓑ-1 ⇄ Ⓑ-2 | Tier is read from that preset's own numbers; the two sets are calibrated apart and may differ | N/A |
| Window grows back | Tier improves | Every panel returns to the spot it was removed from, and the user's own layout is intact | N/A |
| User hides a panel by hand, then resizes | Manual hide + tier change | The manual choice is not overwritten by the tier, and the tier is not persisted over it | N/A |
| Reaching the unsupported tier | `minWidth` lowered per Decision 1 | The tier is reachable by dragging, so the notice can be exercised by hand | N/A |

## Decisions

**Decision 1 — `minWidth` comes down so the fourth tier is real.** Ice ruled 2026-09-22 that
`src-tauri/tauri.conf.json:19` drops below 860 rather than shipping a screen nobody can reach.
Seed the value at `800`, far enough inside the tier that entering it is unambiguous, and let the
calibration pass move it if measurement says otherwise. `minHeight: 600` is untouched — it already
admits the `h < 700` tier. No gate pins these: `config_invariants.rs` reads `tauri.conf.json` for
CSP, security and capabilities only, never for window size.

**Decision 2 — Lookup retreats to one surface, not two.** The status bar carries the entry point;
activating it slides a drawer open over the grid. That satisfies both of UX-DR15's sentences —
*"rút về ngăn kéo"* and *"rút về thanh trạng thái, không bao giờ mất hẳn"* — with one construction
instead of two competing ones, and it keeps the 34 px bar to the one clause it fits. The drawer is
new: `src/` has twelve overlays and every one is a centred modal, which UX-DR16 forbids for routine
flows.

**Decision 3 — Ⓑ-2 is calibrated with parallel Hán-Việt gloss off, and the UX call stays with
Ice.** Story 2.5b measured a 388 px row in Ⓑ-2's 238,5 px column and handed the three-way choice
(keep · cap `<rt>` lines · parallel only in Ⓑ-1) to Ice, with this story owning only the threshold
side. So Ⓑ-2's numbers are measured in the default view, the dependency is recorded beside them,
and neither `<rt>` capping nor a per-preset gloss restriction is built here.

**Decision 4 — the spec ships at full length.** Ice ruled 2026-09-22, on a measured count of
**4 206** tokens against the 1 600 target, that the scope is one goal and that context rot is
already answered by `AGENTS.md`'s rule of ~4 phases, each a fresh agent. Writing these four
decisions in took it to **4 679** on the same slice — `tiktoken` `o200k_base`, everything above
§Implementation Notes, the identical cut taken for all eleven Epic 4 specs, where the range is
4 021 to 11 049 and this one is still the third smallest. The ruling stands; the number it was
made on does not, and that is why both are here.

</frozen-after-approval>

## Code Map

Anchors measured on `bb902f9`.

**Where the mechanism already is — reuse, do not rebuild**

- `src/layout/workspaceLayout.ts` — pure tier, no imports. `SACRIFICE_ORDER:191`,
  `NEVER_SACRIFICED:200`, `nextToSacrifice:214`, `nextToRestore:228`, `PANEL_IDS:54`,
  `PresetId:88`, `LAYOUT_PRESETS:158`, `DEFAULT_PRESET_ID:169`. The four numbers appear only in a
  comment at `:184` naming them as this story's job; `:22-30` and `:205-208` record the deliberate
  ban on window reads here. The new tier constants and ladder function belong in this file, keyed
  by `PresetId`; the DOM must stay out of it.
- `src/layout/WorkspaceDock.vue` — the only component importing that module (`:55-62`).
  `hidden: Map<PanelId, RememberedSpot>:163`, `rememberSpot:338` (it already has a `within`-group
  branch, which is what a tab-merge needs), `hidePanel`/`showPanel`/`togglePanel:357-469`,
  `restoreFocusIfLost:404-443` (guards AC4 of Story 1.6 — focus must not fall to `body` after a
  hide), `visiblePanelsInLayoutOrder:181-194` (drives the focus ring, already excludes hidden
  panels). ⚠️ `:489` is the `onDidLayoutChange` → persist path; the §Always rule about not
  persisting an automatic tier lives or dies here.
- `src/modes/WorkspaceMode.vue:10-11` carries the "no threshold, no `matchMedia`, no drawer —
  Story 4.12" banner, and `:74-78` `onPersist` writes the dock JSON to `ScopeKind::AppConfig`.
- `src/layout/dockController.ts` — the injected bridge from `CommandRegistry` to the live dock; its
  module cell `live` is a named exemption in `check-panel-refs.mjs`.
- `src/commands/focus.ts:172-193` — `enter(owner)` resolves lazily and checks `isConnected`. A
  retreated Lookup must either update its `declareFocus` resolver or call `releaseFocus`, or
  `enter('panel.lookup')` starts failing loudly.

**Where the new surfaces land**

- `src/StatusBar.vue` — `<footer role="status">` with a closed `v-else-if` priority stack at
  `:386-447`, each branch backed by a closed `Record` that fails `vue-tsc` when a state is missing.
  Height is `var(--space-status-height)` (`:471`); `:382` states the bar fits one clause only.
- No drawer or slide-over exists in `src/`. Every overlay there (`SettingsOverlay.vue` and
  eleven siblings) is a centred modal, and UX-DR16 forbids dialogs for routine flows.
- No reusable in-panel tab component exists. The two tab strips in the tree — `GridPanel.vue:1607`
  and `LookupPanel.vue` — are both bespoke, and the first was already hand-copied once. Panel-level
  tabbing is free from dockview: two panels in one group tab automatically via `PanelTab.vue`,
  registered as `tabComponents.aura` at `WorkspaceDock.vue:108`. Prefer the dockview group.
- `src/i18n/vi.json` — no key exists for a narrow window, a hidden panel, or an unsupported size;
  panel titles are at `:638-640`. All new strings are frontend-only, so `message_keys!` and
  `ipc_contract.rs` are not involved.

**Gates that will fire**

- `scripts/check-layout.mjs` — Kiểm A `:241-306` (`import()`s the real module; disjointness, union,
  exhaustive over every subset, order, `nextToRestore` is the exact reverse). Kiểm C `:417-492`
  `ALLOWED_GLOBAL_MEMBERS` is an allow-list: `window.addEventListener` and `document.documentElement`
  are already on it; `window.innerWidth`, `window.innerHeight` and `window.getComputedStyle` are
  **not** (measured — zero matches in `:417-492`), so adding them is a decision that must be written
  into the gate with its reason. Kiểm D `:570` self-tests that Kiểm C can go red. Note the gate
  scans `window.*`/`document.*` only, so a bare `ResizeObserver` would slip past unrecorded — which
  is the reason to use the window reads it can see.
- `check-i18n.mjs` A2 (no UI sentence outside `vi.json`), C (placeholder grammar), D (bans `bạn`).
- `check-commands.mjs` — only if a command id is added; floors `COMMAND_FLOOR = 52`,
  `CLICK_FLOOR = 27`, `DISPATCH_FLOOR = 40` are minimums, so additions cannot redden them.
- `check-tokens.mjs` `EXPECTED_SPACING:233` freezes `titlebar-height: 40px` and
  `status-height: 34px`. These are the numbers the work-area formula uses.

**Tests that move**

`scripts/check-layout.mjs` (a new Kiểm for the ladder, driven by numbers alone) ·
`tests/frontend/` — a new file for the pure ladder and for the tier-vs-manual-hide rule. No
frontend test imports `workspaceLayout.ts` today; there is nothing to extend, only to add.

🔵 2026-09-23 (orchestrator, after Phase 4a) — the prose above named a directory, so
`npm run test:story 4-12` reported the file Phase 4a created as THIẾU KHAI. Named here:
`tests/frontend/workspaceLayoutTier.test.ts` (the pure ladder and `FocusRegistry.release`).
Phase 4b adds its mounted-dock file to this list when it creates it.

🔵 2026-09-23 (Phase 4b) — same gap, same shape: `npm run test:story 4-12` reported
`tests/frontend/workspaceDockTier.test.ts` (mounted `WorkspaceDock`/`WorkspaceMode`, claims A–F)
as THIẾU KHAI. Named here so the next run's diff-vs-spec union already covers it.

🔵 2026-09-23 (Phase 4d) — same gap, same shape: `npm run test:story 4-12` reported
`tests/frontend/dockTree.test.ts` (new, pure — `dockTree.ts::findTreeSpot`, the orientation-alternation
and neighbour-direction rule) as THIẾU KHAI. Named here so the next run's diff-vs-spec union
already covers it.

## Tasks & Acceptance

**Execution:**

- [x] `src/layout/workspaceLayout.ts` -- add the four thresholds per `PresetId` as named constants
      seeded from UX-DR15, plus a pure total ladder `(workArea, presetId) -> tier` -- keep the file
      import-free and DOM-free or `check-layout.mjs` stops being able to `import()` it.
- [x] `scripts/check-layout.mjs` -- a new Kiểm that `import()`s the ladder and pins it on numbers:
      totality, the inclusive boundary at each threshold, top-down precedence, and that the grid is
      never yielded in any tier -- the existing Kiểm A must be left exactly as it is.
- [x] `src/layout/WorkspaceDock.vue` -- observe the window on `resize`, build the work area from
      `window.innerWidth`/`innerHeight` minus the two chrome tokens read through
      `window.getComputedStyle(document.documentElement)`, and drive the tier through the existing
      `hidePanel`/`showPanel` path so `rememberSpot` and `restoreFocusIfLost` keep working -- 🔴 the
      automatic change must not reach the `onDidLayoutChange` persist path at `:489`.
- [x] `scripts/check-layout.mjs` -- add `window.innerWidth`, `window.innerHeight` and
      `window.getComputedStyle` to `ALLOWED_GLOBAL_MEMBERS`, each with its reason in place -- this
      is the entry that ends Story 1.14's "no window read exists in `src/**`", so it is a decision
      that must be written down where the gate can be read.
- [x] `src/layout/WorkspaceDock.vue` -- merge Lookup and AI Translation into one dockview group for
      the short tier, using the `within` branch `rememberSpot` already has -- a bespoke in-panel tab
      strip would be the third hand-copy of the same markup.
- [x] `src/StatusBar.vue` + a new `src/layout/LookupDrawer.vue` (Decision 2) -- a status-bar entry
      point that opens a drawer over the grid, with `declareFocus('panel.lookup')` kept truthful in
      that state -- `enter()` checks `isConnected` and reports a loud failure against a resolver
      pointing at a removed panel, and the bar's priority stack is a closed `Record` that fails
      `vue-tsc` when a state is added without its key.
- [x] `src-tauri/tauri.conf.json` -- lower `minWidth` to 800 (Decision 1), leaving `minHeight` at
      600 -- this is the only line of `src-tauri/**` the story touches, and `cargo test --locked`
      still has to run because `config_invariants.rs` reads this file.
- [x] `src/modes/WorkspaceMode.vue` -- the unsupported-tier notice, non-blocking, with the grid
      still mounted and usable beneath it -- the frozen user story says a narrow window must not
      stop the user translating, so this is a notice and never a gate.
- [x] `src/i18n/vi.json` -- one key per new sentence, each declaring exactly the params it
      interpolates -- Check A2 goes red on any sentence living in a `.vue`.
- [x] `tests/frontend/` -- a new test file covering the ladder as pure numbers and the rule that an
      automatic tier never overwrites a manual hide -- assert the tier and the rendered string, not
      that a function was called.
- [x] `src/layout/WorkspaceDock.vue` + a pure tree helper -- read the direction a hidden panel
      returns in (`above`/`below`/`left`/`right`) from the serialized grid tree
      (`api.toJSON().grid`, orientation alternating per depth from `grid.orientation`), not from
      `boundingBox` -- Ice ruled 2026-09-23. `boundingBox` is live DOM geometry. It is all zeros
      before the dock is sized (`onReady`) and while Workspace is hidden under `KeepAlive`, so
      every direction came out `right`, and Lookup/AI Translation were rebuilt side by side
      instead of stacked.
- [x] `src/layout/WorkspaceDock.vue` + a pure tree transform -- while the `short` tier has
      Lookup and AI Translation merged, a genuine user change IS persisted, but in UN-MERGED form:
      `panel.ai_translation` is written back to its pre-merge spot (the merge's remembered
      anchor + direction). The `|| tierMerged.value` persist guard goes -- Ice ruled 2026-09-23
      ("giữ gộp, lưu dạng chưa gộp"), replacing the orchestrator's unsigned extension.
- [x] Calibrate on real hardware, **separately for Ⓑ-1 and Ⓑ-2**, with all three panels carrying
      content and parallel gloss off in Ⓑ-2 (Decision 3), and record each measurement's window
      size, work area, preset and verdict in §Implementation Notes -- Ⓑ-1 and Ⓑ-2 compress
      differently, so one set does not derive the other. 🔵 2026-09-23 — closed by Ice's ruling, not as written: one measurement (Ⓑ-2 only), then the seed numbers were ruled final for both presets. See §Implementation Notes → Phase 5 and §Spec Change Log.
- [x] `_bmad-output/specs/spec-AuraTranslate/SPEC.md` -- write the calibrated numbers into `[A11]`,
      mark `Q9` closed, and fix A11's pre-2026-08-14 wording (`giữ 2×2`, `gộp hàng dưới`) in place
      with 🔵 and today's date -- `epics.md:543` is the live wording; `SPEC.md:126` was missed in
      that pass.
- [x] `_bmad-output/implementation-artifacts/deferred-work.md` -- record, with a named owner and a
      §SECTION NAME citation: the undocumented `titlebar-height` 38 → 40 divergence between
      `DESIGN.md:132` and the frozen token (owner Ice, the same shape `StatusBar.vue` §AC9 used for
      `status-height`), and the Ⓑ-2 parallel-gloss UX choice Decision 3 leaves with Ice -- cite
      §SECTION NAME, never a line number.

**Acceptance Criteria:**

- Given the window is resized across a threshold and then back, when it returns to its original
  size, then every panel is at the spot it left and the layout stored in `AppConfig` is byte-for-byte
  what it was before the resize.
- Given a tier that hides a panel, when focus was inside that panel, then focus lands on a real
  element and never on `body`.
- Given the calibration pass, when it finishes, then Ⓑ-1 and Ⓑ-2 each carry their own four numbers,
  each traceable to a recorded measurement rather than to UX-DR15's seed.
- Given the default window 1280×860, when the calibrated numbers are applied, then the tier the app
  opens in is a measured decision that is written down — today's seed puts it below 820 and
  therefore not in the full layout.
- Given Lookup has retreated at a narrow tier, when the user activates its status-bar entry point,
  then the drawer opens with Lookup fully usable — at no window size is Lookup unreachable.
- Given `minWidth` at 800, when the window is dragged narrower than 860, then the unsupported
  notice appears and the user can still read the source and type a translation.
- Given `check-layout.mjs`, when it runs after this story, then Kiểm A passes with its assertions
  unchanged, and the new Kiểm goes red if any threshold constant is moved.
- Given the whole AI configuration removed, when the prior epics' features are exercised, then they
  still work in full — AD-13's boundary test still passes unchanged.

## Implementation Notes

### Phase 1 (2026-09-22) — the pure ladder and its gate (Tasks 1-2)

**What was added, in `src/layout/workspaceLayout.ts`** (append-only; one small 🔵-dated
pointer was also added to the existing comment above `SACRIFICE_ORDER` that used to say
the four thresholds "belong to Story 4.12" — it now points at the new section below
instead of being rewritten):

- `LayoutTier = 'full' | 'short' | 'narrow' | 'unsupported'`, `LAYOUT_TIERS` (the four
  values, for exhaustive iteration).
- `WorkArea = { readonly width: number; readonly height: number }` — CSS px, already
  chrome-subtracted; that subtraction is Phase 2's job in `WorkspaceDock.vue`, this
  module never reads a raw window size.
- `LayoutThresholds = { minFullWidth, minFullHeight, minShortHeight, minSupportedWidth }`
  — four named fields so one number can move without touching the function body.
- Two **separate** seed objects, `SEEDED_THRESHOLDS_B2`/`SEEDED_THRESHOLDS_B1`, both
  `{1100, 820, 700, 860}` today (seeded from the spec's I/O Matrix, per Phase 1's
  instructions to seed both presets identically) but deliberately **not one shared
  object** — Ⓑ-1 and Ⓑ-2 must be movable independently once Phase 5 calibrates them
  apart, and a shared reference would make "move Ⓑ-2 only" silently move Ⓑ-1 too.
- `LAYOUT_THRESHOLDS: Readonly<Record<PresetId, LayoutThresholds>>` keyed by the two real
  `PresetId`s.
- `layoutTierFor(workArea: WorkArea, presetId: PresetId): LayoutTier | null` — pure,
  total, top-down precedence: `width < minSupportedWidth` (unsupported) →
  `width < minFullWidth` (narrow) → `height < minShortHeight` (narrow, "very short") →
  `height < minFullHeight` (short) → `full`. An unrecognised `presetId` falls back to
  `LAYOUT_THRESHOLDS[DEFAULT_PRESET_ID]`, the same defensive shape as `presetById`.

**🔴 Decision recorded here because the coordinator's review made it, not me: the
non-finite contract.** My first draft returned a `LayoutTier` unconditionally, so
`layoutTierFor({width: NaN, height: NaN}, id)` fell through all four `if (x < threshold)`
guards (every comparison against `NaN` is `false`) and returned `'full'` — the single
worst answer, because it reports the roomiest layout on input that means "I could not
measure this." Phase 2 reads the two chrome tokens through
`getComputedStyle(...).getPropertyValue(...)` then `parseFloat`; an empty-string token
produces exactly this `NaN`, not a hypothetical. `AGENTS.md` names silent-emptiness as
this project's central failure class and states the rule directly: a value that can be
UNKNOWN gets an `Option`/`NULL`, never a silent default. The fix: `layoutTierFor` now
returns `LayoutTier | null`, guarding with `!Number.isFinite(width) || !Number.isFinite(height)`
at the top before touching any threshold — `Number.isFinite` rejects `NaN`, `Infinity`,
`-Infinity`, and any non-number value in one guard. **Contract Phase 2 must honour: on
`null`, keep the tier already applied and log a diagnostic naming the cause — never treat
`null` as `'full'` or as any other tier.**

**What was added, in `scripts/check-layout.mjs`** — a fifth check, **Kiểm E**, inserted
after Kiểm D, calling the real `layoutTierFor`/`LAYOUT_THRESHOLDS`/`LAYOUT_TIERS` through
the same `layoutMod` import Kiểm A already uses (no logic copied into the script). Four
propositions, each corrected once by the coordinator's review of the first draft:

1. **Totality, two halves.** 392 finite `(width, height, preset)` combinations (negative,
   zero, decimal, and large values included) always yield one of the four tier strings
   and never throw; a second grid of 24 combinations with `NaN`/`Infinity`/`-Infinity`/
   `undefined` in one or both dimensions always yields exactly `null`, never a tier
   string (added together with the `null` contract above).
2. **Boundary, both sides, keyed per preset — not one shared `SEED` looped over both.**
   The first draft pinned a single `SEED` object and iterated it over both `PresetId`s.
   That is correct only while both presets happen to share numbers, which the spec says
   is temporary: Phase 5 calibrates Ⓑ-1 and Ⓑ-2 **separately**, so the day one preset's
   number moves, a shared loop is the wrong shape — it would either miss the change or
   misattribute it to the wrong preset. Fixed to an `EXPECTED` table keyed by `PresetId`,
   the same shape `LAYOUT_THRESHOLDS` itself uses, so a future one-preset calibration
   reddens exactly that preset's rows and names them. All eight boundary assertions (4
   thresholds × 2 sides) are independently pinned literals in the gate script, not read
   back from `LAYOUT_THRESHOLDS` — moving a threshold without updating this table is the
   failure this check exists to catch. Verified by mutation: temporarily changed
   `SEEDED_THRESHOLDS_B2.minFullWidth` from `1100` to `1050`, ran `npm run check:layout`,
   confirmed Kiểm E goes red naming the exact constant and preset
   (`biên — minFullWidth kém 1 ⇒ narrow (layout.preset_grid, 1099×999) mong \`narrow\`, được \`full\``),
   then restored the file and reconfirmed green. Repeated the same mutation/restore cycle
   after the per-preset-keyed fix — same red, same named constant and preset.
3. **Top-down precedence.** `(800, 650)` → `unsupported` (width-unsupported outranks a
   height that would otherwise read "narrow, too short"); `(1500, 650)` → `narrow`
   (height-too-short outranks a comfortable width); `(900, 2000)` → `narrow`
   (narrow-width band outranks a comfortable height).
4. **`NEVER_SACRIFICED` untouched, two vocabularies disjoint — renamed from its original,
   overclaiming label.** The first draft named this proposition "LƯỚI KHÔNG BAO GIỜ BỊ
   NHƯỜNG" ("the grid is never yielded") in both the file header and the check body, but
   its body only ever checked that `NEVER_SACRIFICED` is untouched and that tier names
   don't collide with `PanelId` names — neither shows the grid stays mounted at any tier,
   because the pure ladder returns only a tier and has no notion of mounted panels; it
   structurally cannot prove that claim. The coordinator's review caught this — the same
   shape Story 4.11's review caught, a matrix row accepted on a test's NAME rather than
   its body. Renamed to state exactly what it proves (`NEVER_SACRIFICED` is exactly
   `['panel.grid']`, unmoved by Task 1/2; tier names and `PanelId` names are disjoint
   sets) and added an explicit note that the real "grid is never yielded at any tier"
   claim is proved in Phase 4, against `WorkspaceDock.vue`, not here.

**Commands run, counts measured myself:**

- `npm run check:layout` — green. 5 Kiểm (A-D byte-identical in behavior to the pre-Phase-1
  baseline; new Kiểm E green with the four propositions above).
- `npm run build` — green (`vue-tsc --noEmit` × 2 configs + `vite build`); the new
  exported types type-check cleanly against the rest of the tree.
- `npx vitest run` — green twice, same counts both times: **91 test files passed, 1327
  tests passed, 0 failed** (first run 143.99s, second run — after the three fixes above —
  172.73s; Phase 1 does not touch `tests/frontend/**`, so this is a regression check, not
  new coverage).
- `cd src-tauri && cargo test --locked` — measured by the orchestrator on a still tree
  (my diff touches zero Rust files; I did not run this myself for the final confirmation,
  per explicit instruction not to start a third run while the orchestrator's own
  measurement was in flight).

**Left for Phase 2** — see the phase file's `### Phase 1` note under "What you must leave
for Phase 2."

### Phase 2 (2026-09-22) — wiring the real window to the ladder (Tasks 3-5)

**What was added, in `src/layout/WorkspaceDock.vue`** (new section "Tầng bố cục tự động theo
kích thước cửa sổ", between `togglePanel` and the persist section):

- `computeWorkArea(): WorkArea` — `width = window.innerWidth`; `height = window.innerHeight`
  minus two chrome tokens read at runtime via
  `window.getComputedStyle(document.documentElement).getPropertyValue('--space-titlebar-height'
  | '--space-status-height')` then `parseFloat`. No literal `40`/`34` anywhere — confirmed the
  two custom properties are the ones `tokens/index.ts::applyTheme` actually writes (traced from
  `tokens.json` → `applyTheme`'s `style.setProperty` loop; they have no static `:root` block in
  `src/**`, they only exist once `applyTheme()` has run, which `main.ts` already guarantees
  before `mount()`).
- `measureAndApplyTier()` — calls `layoutTierFor(computeWorkArea(), currentPresetId.value)`.
  On `null` (Phase 1's contract): logs a diagnostic, returns without touching `currentTier` —
  the tier already applied stays applied, never silently reread as `'full'`. On a tier equal to
  `currentTier.value`: no-op (cheap, so a dense `resize` stream costs only string comparisons
  until the tier actually crosses a threshold). Otherwise calls `applyTier(tier)`.
- `applyTier(tier)` — the dispatcher: `narrow`/`unsupported` → `undoMerge()` +
  `applyGridOnlySacrifice()` (only the grid remains — both other panels retreat, same panel
  set for both tiers per the pure module's own doc-comment); `short` →
  `restoreAllAutoHidden()` + `applyMerge()` (all three panels visible, AI Translation tabbed
  into Lookup's dockview group); `full` → `undoMerge()` + `restoreAllAutoHidden()`. Every
  branch reuses `hidePanel`/`showPanel` (via `autoHide`/`autoShow`) — no second hide mechanism.
  Ends by calling `restoreFocusIfLost` unconditionally (it already no-ops when focus was not
  actually lost, same discipline as `togglePanel`).
- `applyGridOnlySacrifice()` / `restoreAllAutoHidden()` — drive `nextToSacrifice`/a
  restore-priority scan of `SACRIFICE_ORDER` against the **live** `visiblePanelsInLayoutOrder()`
  set, so a panel the user already hid manually is transparently skipped on the sacrifice side.
- `applyMerge()` / `undoMerge()` — Task 5. Reuse the `within` branch `rememberSpot` already had:
  `applyMerge` calls `rememberSpot(api, aiPanel)` to save AI Translation's pre-merge position
  into `mergedSpot`, then `api.removePanel` + `addPanel(..., { referencePanel: 'panel.lookup',
  direction: 'within' })`. `undoMerge` reverses it from `mergedSpot`, falling back to "right of
  the first visible panel" (same fallback `showPanel` already uses) if the remembered anchor is
  gone. Both refuse silently if either panel is currently hidden by hand — merging would force
  a manual hide back open, which the spec forbids.
- **The anti-persist mechanism — two parts, not one:**
  1. `let suppressPersist = false`, and `onLayoutChange()`'s **first line** is now
     `if (suppressPersist) return`. Every automatic mutation (`autoHide`/`autoShow`/the two
     `api.removePanel`+`addPanel` pairs in `applyMerge`/`undoMerge`) brackets itself with
     `suppressPersist = true` / `= false` synchronously around the single dockview call —
     `dockview` fires `onDidLayoutChange` inside that same synchronous call, not on a
     microtask, so the bracket has to be tight and un-awaited. **This is the line Phase 4's
     counter-check 3 removes**, per the phase file.
  2. `applyTier(tier)`'s **first line** is `flush()`. This is a second, independent piece the
     phase file didn't anticipate and I want to flag explicitly: `suppressPersist` only stops a
     *new* dirty-mark; it does nothing about a write **already** pending from a real user
     action (e.g. `layout.preset_columns` dispatched a moment earlier, still inside its 500 ms
     idle window) when the tier then mutates the dock further. Without draining that pending
     write first, the eventual debounced `flush()` reads `api.toJSON()` **at flush time** —
     i.e. after the tier's own automatic mutation — and persists a layout the user never asked
     for, through a different door than the one `suppressPersist` guards. Calling `flush()` at
     the top of `applyTier` (it already no-ops when the schedule is clean, so this is free on
     the common path) closes that door: any real pending write lands *before* the tier touches
     anything. I verified this is still consistent with the counter-check as written: removing
     only the `onLayoutChange` guard (not this `flush()` call) still turns a plain resize that
     crosses a tier into a dirtied, eventually-persisted schedule, because the auto-mutations
     themselves start marking dirty again once the guard is gone — the `flush()` call only
     changes what a *pre-existing* pending write captures, it does not swallow new dirt once
     the primary guard is absent.
- `currentTier` / `currentPresetId` / `tierMerged` are `shallowRef`s (not plain `let`), each
  bound to a `data-*` attribute on `.dock-host` in the template — see "data-\* hooks" below.
- `autoHiddenIds: Set<PanelId>` — **the manual-vs-auto distinction Phase 4 needs**: a panel id
  is a member **iff** its most recent hide went through `autoHide()` (tier-driven), not through
  `hidePanel()` called directly from `togglePanel()` (a real `layout.toggle_*` dispatch).
  `reconcileAutoHidden()` (called first in every `applyTier`) drops any id from this set whose
  entry has disappeared from the real `hidden` map — i.e. the user showed it back manually in
  the meantime — so a stale record can never cause the tier to "restore" something it did not
  hide. Restoring only ever pulls from this set (`nextAutoRestoreCandidate`), never from
  `hidden` directly, which is the one-line reason a manual hide survives a resize round trip.
- `applyPreset()` — three additions: (a) clears `autoHiddenIds` / `tierMerged` / `mergedSpot`
  alongside the existing `hidden.clear()`, in **both** the success path and the mid-loop
  `catch` (a fresh preset is a from-scratch layout; the old tier bookkeeping describes nothing
  real once `api.clear()` has run); (b) on success, records `currentPresetId.value = preset.id`
  and resets `currentTier.value = null` **before** calling `measureAndApplyTier()` — reset to
  `null` rather than comparing tier names, because the dock was just rebuilt from scratch and
  the old tier string no longer describes anything even when it happens to read the same; (c)
  calls `measureAndApplyTier()` itself, so switching preset at a fixed window size immediately
  re-evaluates against the **new** preset's own thresholds (I/O Matrix row "Preset switched at
  a fixed size") rather than waiting for the next `resize` event.
- `onReady()` calls `measureAndApplyTier()` once at the very end (after `setDockController`),
  covering the `restore()` → `fromJSON` path (an arbitrary saved layout, which never calls
  `applyPreset` and therefore never self-measures) as well as the `restore()` →
  `applyPreset(DEFAULT_PRESET_ID)` path (where the call is a cheap no-op, since that path
  already measured once). `onMounted`/`onBeforeUnmount` add/remove a `window.addEventListener
  ('resize', onWindowResize)` pair alongside the existing `beforeunload` pair.

**data-\* hooks left for Phase 4 (and for Phase 3, which renders the surfaces these describe)**
— all three on `.dock-host`, the template root:

- `data-layout-tier` — `'full' | 'short' | 'narrow' | 'unsupported'`, absent before the first
  `measureAndApplyTier()` call resolves (i.e. absent only for a single synchronous instant
  during `onReady`, never in a mounted, idle app).
- `data-layout-preset` — the live `PresetId` the tier is currently reading thresholds from.
- `data-layout-merged` — the string `"true"` while `panel.ai_translation` is tabbed into
  `panel.lookup`'s group by the tier (`short`); attribute absent otherwise. Chosen as an
  explicit string rather than a bare boolean bind because Vue's attribute-omission rule for
  `false` is not obviously stable across attribute kinds, and I wanted the omit/present
  contract to be unambiguous for a test reading raw DOM.

**A known, deliberate limitation of `currentPresetId`:** it is updated **only** by a
successful `applyPreset()` call, never inferred from an arbitrary restored layout
(`restore()`'s `fromJSON` branch). A saved layout is not necessarily shaped like either preset
once the user has dragged panels around, and there is no reverse mapping from an arbitrary
dockview JSON tree back to a `PresetId`. Consequence: after restoring a heavily customized
layout from a previous session, `layoutTierFor` reads thresholds for `DEFAULT_PRESET_ID`
(Ⓑ-2) until the user next dispatches `layout.preset_*`, even if their customization visually
resembles Ⓑ-1. This is a reasonable approximation given the module's own fallback rule for an
unrecognised `presetId`, but it is a real, user-visible edge and I am naming it rather than
letting it pass silently.

**A design call for Ice/reviewer attention, not a bug:** the "manual choice survives the tier"
rule in the spec's I/O Matrix is written for the **hide** direction only ("User hides a panel
by hand, then resizes"). My implementation does not extend the same protection to a manual
**show** that contradicts the tier's own requirement: e.g. at `narrow` tier (grid-only
required) if the user manually re-shows Lookup via `layout.toggle_lookup` while `narrow` is
still active, it stays shown (the tier only reconciles on a tier *change*, not continuously) —
but if the tier then changes again to `unsupported` (still grid-only), `applyGridOnlySacrifice`
re-evaluates from the live visible set and **will** auto-hide Lookup again, overriding that
manual show. This is internally consistent (the tier enforces its own invariant on every
transition, and neither `narrow` nor `unsupported` ever permits more than the grid) and it is
documented in `applyGridOnlySacrifice`'s call sites, but the spec does not say which behaviour
it wants here, and I did not want to silently pick one without naming it.

**One fix outside Phase 2's own task list, in `src/layout/workspaceLayout.ts`
(Phase 1's file):** `npm run check:lint` was **red** before I touched anything else —
`layoutTierFor`'s `presetId: PresetId` parameter made `LAYOUT_THRESHOLDS[presetId] ??
LAYOUT_THRESHOLDS[DEFAULT_PRESET_ID]` an *unnecessary* conditional under
`@typescript-eslint/no-unnecessary-condition`: indexing a closed `Record<PresetId, …>` with a
value already typed `PresetId` is, to TypeScript, guaranteed never to miss — even though the
doc-comment directly above the line describes the fallback as handling "an id lạ" (a
stale/corrupted `PresetId` read back from disk). Verified this predates my diff: reverting only
this one file's `layoutTierFor` signature+body to Phase 1's original text (keeping every other
Phase 2 file as-is) reproduces the identical error at the identical line; `npm run check:lint`
was not part of Phase 1's own "Finish on" list, so it went unchecked. Fix: widened the
parameter to `presetId: string` and replaced the raw index with `presetById(presetId)` (the
validated lookup this same file already exposes) — `preset.id` is a *real* `PresetId` (not an
`as` cast), so the index stays type-safe, and the `??`/ternary now guards a genuine
`LayoutPreset | undefined`, satisfying the linter honestly rather than suppressing it. Pure
type-level change; `npm run check:layout` Kiểm E (392 finite + 24 non-finite combinations,
boundaries, priority) is unchanged and still green, confirming behaviour is identical. This
widens `layoutTierFor`'s public signature from Phase 1's stated contract (`presetId: PresetId`)
to `presetId: string` — every existing call site (Kiểm E's `PRESET_IDS` array of literal
strings, and my own `currentPresetId.value: PresetId`) is unaffected, since `PresetId` widens
to `string` trivially.

**Tasks intentionally not touched, per the phase file:** Task 6 (`StatusBar.vue` entry point +
`LookupDrawer.vue`), Task 7 (`tauri.conf.json` `minWidth`), Task 8 (`WorkspaceMode.vue`
unsupported notice), and the `vi.json` keys those three surfaces would render — all Phase 3.
`tests/frontend/**` — Phase 4's. Confirmed by inspection: at the tiers Phase 2 builds, Lookup
retreating at `narrow`/`unsupported` currently has **no discoverable UI entry point** (the
status-bar drawer is Phase 3's Task 6) — the only way back today is dispatching
`layout.toggle_lookup` directly (it exists, confirmed in `src/commands/index.ts:1162`, and is
deliberately unbound to a default key per `src/commands/index.ts`'s own §Quyết định #3 comment,
same as the other two `layout.toggle_*` commands). This satisfies "Lookup never disappears
entirely" at the command layer but not yet at the discoverable-UI layer — that gap is exactly
what Phase 3 closes.

**Commands run, counts measured myself:**

- `npm run check:layout` — green, 5 Kiểm (A-D byte-identical; new Kiểm E still green with the
  four propositions Phase 1 wrote). 20 `window`/`document` members now allowed (17 before this
  phase, +`window.innerWidth`/`window.innerHeight`/`window.getComputedStyle`), all with a
  reason written into `ALLOWED_GLOBAL_MEMBERS` per Task 4.
- `npm run check:i18n`, `check:commands`, `check:tokens`, `check:panel-refs`, `check:deps`,
  `check:dict`, `check:dict-manifest`, `check:gates`, `check:debt-owner` — all green
  individually (nine of the eleven `pre-push` gates; `check:layout` and `check:lint` counted
  separately above/below since they are the two this phase's diff could plausibly redden).
- `npm run check:lint` — **red** on first run (see the `workspaceLayout.ts` fix above), green
  after the fix, confirmed with a second full run.
- `npm run build` — green (`vue-tsc --noEmit` × 2 configs + `vite build`); confirmed green both
  before and after the `workspaceLayout.ts` lint fix.
- `npx vitest run` — green twice, same counts both times: **91 test files passed, 1327 tests
  passed, 0 failed** (first run 80.38s, before the `workspaceLayout.ts` fix; second run 78.89s,
  after it — Phase 2 does not touch `tests/frontend/**`, so both are regression checks against
  Phase 1's baseline, not new coverage. Byte-identical counts to Phase 1's own two runs).
- `cd src-tauri && cargo test --locked` — measured myself, one full run, no pipe:
  **1806 passed, 0 failed, 22 ignored, 65 `test result:` lines, exit 0.** Byte-identical to
  Phase 1's baseline, which is the expected answer: this phase's diff touches zero Rust files.
- `npm run test:story 4-12 --list` — reports 0/91 frontend targets touched (correct: Phase 2
  does not touch `tests/frontend/**`) and 2/61 Rust targets (`config_invariants`,
  `ipc_contract`) — **not** from my diff (`git diff --stat` confirms zero Rust files changed
  this phase); the script pattern-matches Rust test-binary names mentioned anywhere in the
  spec's prose, including a *negative* mention ("`ipc_contract.rs` are not involved") — a
  known, documented heuristic limitation of the script (it is a net for tests the story *will*
  touch, per its own description), not a signal about this phase's diff.

**Left for Phase 3** — the three `data-*` hooks above are ready for the status-bar entry point
and `LookupDrawer.vue` to attach to; `applyGridOnlySacrifice`/`applyMerge`/`undoMerge` do not
need to change when Phase 3 lands, since Lookup's retreat is already a real `hidePanel()` call
today, Phase 3 only adds a way back in besides `layout.toggle_lookup`. `currentTier`/
`currentPresetId`/`tierMerged` are exported as nothing (module-internal `shallowRef`s) — Phase
3 reads tier state through the `data-*` attributes or through its own status-bar logic, not by
importing anything new from `WorkspaceDock.vue`.

### Phase 3a (2026-09-23) — Lookup's retreat surface, and closing the boundary the first pass hit

Task 6 done. **The surface**: Decision 2's one construction, not two. `src/StatusBar.vue`
gains a fifth, always-clickable entry point (`.lookup-entry`, outside the five-branch priority
stack, pushed to the right edge) that dispatches `layout.lookup_drawer_open`; a new
`src/layout/LookupDrawer.vue` slides in from the right edge over the grid — same scrim/focus-trap
skeleton as the twelve existing overlays (copied from `AttributionOverlay.vue`, the simplest of
them), only the shape of `.panel` changes: anchored to one edge, full height, capped width
(`min(420px, 100%)`), not a centred box. Its body is `LookupPanel.vue` itself — the real
component, not a summary — so `PanelFrame.vue`'s existing `declareFocus('panel.lookup', …)` /
`releaseFocus('panel.lookup', …)` pair keeps `enter('panel.lookup')` pointed at a real, connected
element whether Lookup is currently living in the grid or in the drawer. State (`drawerOpen`,
`retreated`, the two command handlers) lives in a new `src/layout/lookupDrawerState.ts`, same
shape as `shortcutsState.ts`/`dictSourcesState.ts` — a module-level `ref`, injected into
`CommandDeps` at `src/main.ts` rather than imported into `src/commands/index.ts` (that file must
stay loadable by plain Node for Kiểm C/D/E of `check:commands`).

**A first pass at this task was stopped before writing notes, with two defects — both traced to
one cause: a boundary instruction (`4-12-phases-2026-09-22.md` §Phase 3a) that said the tier
machinery "already exists and is closed … do not change it," meant to freeze the tier LADDER
(the four thresholds, the precedence order, `onLayoutChange`'s first line), but read as "do not
touch `WorkspaceDock.vue` at all." Ice lifted the boundary 2026-09-23 to its intended scope; both
defects close at the root once `WorkspaceDock.vue::applyTier` is allowed to call out.**

- **Defect 1 — a 200ms `setInterval` polling the DOM.** The stopped pass had no way to learn the
  current tier except `document.querySelector('.dock-host').dataset.layoutTier`, read every
  200ms, forever, from `StatusBar.vue::onMounted` to `onBeforeUnmount`. Two things were wrong:
  `data-layout-tier` is a **test hook** (Phase 4's e2e reads it, per Phase 2's notes above), not
  a production data channel; and a permanent 5Hz timer buys a latency a direct call does not
  have. **Fix:** `WorkspaceDock.vue::applyTier` now calls `lookupDrawerState.ts::syncLayoutTier
  (tier)` directly, synchronously, right after its existing `flush()` line and before any branch
  that mutates the dock. The poll, `POLL_MS`, `pollTimer`, `startLookupRetreatWatch`/
  `stopLookupRetreatWatch`, `KNOWN_TIERS` and `readTierFromDom` are all deleted; `StatusBar.vue`
  no longer starts or stops anything for the drawer, it only reads `lookupHasRetreated` for its
  `v-if`. The `data-*` attributes on `.dock-host` are untouched — Phase 4 still reads them.
  A `.vue` importing a Vue-`ref`-using `.ts` sibling has no bearing on the "loadable by plain
  Node" constraint; that constraint binds `src/commands/index.ts` and, transitively,
  `src/layout/dockController.ts` (which is why *that* file may not hold a `ref` — `main.ts`
  injects live functions into it instead). `lookupDrawerState.ts` was never in that chain.

- **Defect 2 — `declareFocus('panel.lookup')` could throw "already declared".** When the tier
  jumped from `narrow`/`unsupported` straight to `short`/`full` in one step while the drawer was
  open, `applyTier` → `restoreAllAutoHidden` → `autoShow('panel.lookup')` → `addPanel` re-mounted
  `LookupPanel.vue` in the grid **synchronously** (dockview mounts panel components immediately,
  not on Vue's own microtask flush), while the drawer's copy still owned `panel.lookup` — Vue
  tears the drawer down **asynchronously**, on a later flush. **Fix, at the root:**
  `syncLayoutTier` closes the drawer AND relinquishes `panel.lookup` synchronously, in the same
  call, before `applyTier` reaches `restoreAllAutoHidden()`: `drawerOpen.value = false;
  releaseFocus('panel.lookup')`. That alone would create a second problem — when Vue eventually
  tears the drawer's old `PanelFrame` down and its `onBeforeUnmount` fires `releaseFocus
  ('panel.lookup')` again, an unconditional release would delete the grid's freshly-declared,
  valid entry. So `FocusRegistry.release` (`src/commands/focus.ts`) gained an optional second
  parameter, `expected: FocusEntry` — when given, `release` only deletes if the *current*
  registration is reference-equal to `expected`; a mismatch means someone else has legitimately
  taken over the owner name since, and it exits quietly (not an error — a late release after a
  handoff is not a bug). `PanelFrame.vue` now keeps one stable `resolve` closure per instance and
  passes it to both `declareFocus` and `releaseFocus`, so the comparison is meaningful; every
  other `release()` call site (three in `src/modes/*.vue`, calling it with a literal mode name)
  is unaffected — the parameter is optional and those sites never passed it. Considered and
  rejected: giving the drawer's copy a different owner id — `enter('panel.lookup')` must keep
  working while Lookup lives only in the drawer, and a second id breaks exactly that. `owner` is
  the single string `'panel.lookup'` in both places it lives, same as the stopped pass already
  had it.

**`vi.json` keys added, and which surface renders each:**

- `command.layout.lookup_drawer_open` ("Tra cứu") — the status-bar entry point button label,
  `StatusBar.vue`.
- `command.layout.lookup_drawer_close` ("Đóng ngăn kéo Tra cứu") — the drawer's close button
  label, `LookupDrawer.vue`.
- `panel.lookup.title` was **not** added this phase — it already existed (panel titles,
  Code Map line ~641) and is reused as-is for the drawer's `<h2>`.

**`data-*` hooks a test can select on** (three pre-existing from Phase 2, one new this phase):

- `.dock-host[data-layout-tier]`, `[data-layout-preset]`, `[data-layout-merged]` — unchanged
  from Phase 2, read directly by `syncLayoutTier`'s caller (`applyTier` already has `tier` as a
  parameter, so this phase adds no new *reader* of these three; they remain for e2e).
- `[data-lookup-drawer-open]` — a boolean-style attribute (present, no value) on the status-bar
  button, new this phase. `LookupDrawer.vue` also uses it as a CSS-independent selector to find
  the opener element for UX-DR17 focus-return (`focusReturnTargetOnOpen`/the `querySelector`
  fallback in its `watch(lookupDrawerIsOpen, …)`).

**Gates re-checked after Defect 2's fix:** `scripts/check-panel-refs.mjs` had four named
exemptions from the stopped pass (`KNOWN_TIERS`, `drawerOpen`, `retreated`, `pollTimer`).
`KNOWN_TIERS` and `pollTimer` no longer exist (deleted with the poll) — their exemptions are
removed, not left stale. `drawerOpen` and `retreated` are unchanged in kind (still session state,
still correctly exempt from Tác-phẩm reset) but `retreated`'s reason text described the 200ms
poll; reworded to describe the synchronous push. `check:panel-refs` now reports 33 named
exemptions (was 35) — confirmed by running the gate, not by counting the diff.

**Counts measured myself, this phase:** all eleven gates green individually
(`deps`/`tokens`/`i18n`/`commands`/`layout`/`panel-refs`/`dict`/`dict-manifest`/`lint`/`gates`/
`debt-owner`) · `npm run build` green (`vue-tsc` on both configs + `vite build`, confirming the
`FocusRegistry.release` signature change type-checks everywhere it's called) · `npx vitest run`
and `cd src-tauri && cargo test --locked` — see the phase-file's `## Phase notes` for the
measured counts, to avoid recording the same number twice in two files.

### Phase 3b (2026-09-23) — `minWidth: 800` and the unsupported-tier notice (Tasks 7, 8, part of 9)

**Changes:**

- `src-tauri/tauri.conf.json:19` — `minWidth` `960` → `800` (Decision 1). `minHeight` at `600`
  untouched. No other line of `src-tauri/**` moves.
- `src/layout/WorkspaceDock.vue` — added exactly one second typed emit beside `persist`:
  `(e: 'tier-change', tier: LayoutTier): void`, fired from `measureAndApplyTier()` right after
  `applyTier(tier)`, so it only ever fires with a measured, non-`null` tier (Phase 1's contract on
  `null` is unchanged). The thresholds, `layoutTierFor`, `applyTier`'s body, `onLayoutChange`'s
  `if (suppressPersist || autoHiddenIds.size > 0) return` line, and the persist path were not
  touched — only the one `emit(...)` call was added.
- `src/modes/WorkspaceMode.vue` — listens `@tier-change="onTierChange"`, holds the tier in a local
  `shallowRef<LayoutTier | null>`, and renders one in-flow `<p role="status" v-if="currentTier ===
  'unsupported'">` strip ABOVE `<WorkspaceDock>` in the same flex column — never an overlay, never
  a modal, no `z-index`, no intermediate `opacity`. `WorkspaceDock`'s root (`.dock-host`) already
  carries `flex: 1; min-height: 0`, so the notice's own height is taken out of the column and the
  grid simply gets a little shorter; it stays mounted and interactive under/behind nothing, because
  there is no "behind" — it is a sibling, not a layer. Does NOT read `data-layout-tier` or any
  other `data-*` attribute; its only data source is the `tier-change` payload.
- `src/i18n/vi.json` — one new key, `mode.workspace.narrow_notice`, no interpolation params (no
  numeral is formatted in the webview — the `860`/`800` cutoffs never reach the string).

**Does the notice affect the width the ladder reads?** No. `computeWorkArea()` (`WorkspaceDock.vue`
~line 568) derives the work area from `window.innerWidth`/`window.innerHeight` minus two *fixed*
CSS custom-property tokens (`--space-titlebar-height`, `--space-status-height`) — it never measures
`.dock-host`'s or any sibling's rendered bounding rect. The notice strip changes how much vertical
room the grid gets rendered into, but it changes neither `window.innerWidth`/`innerHeight` nor the
token values, so it cannot feed back into `layoutTierFor` and cannot oscillate the tier.

**Counts measured myself, this phase, each its own exit code, read only after its own completion
sentinel (not through a pipe):** all eleven gates green individually — `check:deps` exit 0 ·
`check:tokens` exit 0 · `check:i18n` exit 0 · `check:commands` exit 0 · `check:layout` exit 0 ·
`check:panel-refs` exit 0 · `check:dict` exit 0 · `check:dict-manifest` exit 0 · `check:lint`
exit 0 · `check:gates` exit 0 · `check:debt-owner` exit 0. `npm run build` exit 0 (`vue-tsc` both
configs + `vite build`, 190 modules). `npx vitest run` exit 0, **91 files / 1327 passed** — byte-
identical to Phase 3a's totals, expected: nothing under `tests/frontend/**` was touched this phase.
`cd src-tauri && cargo test --locked` exit 0, **1806 passed, 0 failed, 22 ignored, 65 `test
result:` lines** — also byte-identical to Phase 3a, and `tests/config_invariants.rs`'s 31 cases
are all green including the ones that read `tauri.conf.json`, none of which assert on window size
today (as this phase's own dispatch note predicted).

**Not done, left for whoever owns the rest of Task 9 and Task 9's test file:** this phase did not
touch `tests/frontend/**` (out of scope per the dispatch boundary) — the ladder-as-pure-numbers
test file and the manual-hide-survives-a-tier-change case are still open. `SPEC.md [A11]`
calibration and `deferred-work.md` entries (the remaining bullets of the task list) are also not
this phase's slice.

### Phase 4a (2026-09-23) — the ladder as pure numbers + `FocusRegistry.release` late-release, coverage only

New file `tests/frontend/workspaceLayoutTier.test.ts` (49 cases). No file under `src/**` or
`scripts/**` was left changed — every counter-check mutated a scratchpad-backed copy and was
restored, confirmed byte-identical by `shasum -a 256`, before the next step ran.

**Counter-checks, in order, each a removal at the seam:**

1. `src/layout/workspaceLayout.ts` — moved `SEEDED_THRESHOLDS_B1.minFullWidth` 1100 → 1150
   (`layout.preset_columns` only). `npm run check:layout` → exit 1, Kiểm E red line: `biên —
   minFullWidth đúng biên ⇒ không còn narrow-vì-rộng (layout.preset_columns, 1100×999) mong
   \`full\`, được \`narrow\`` — names exactly the mutated preset and threshold. Restored from
   the scratchpad copy, `shasum -a 256` match confirmed, re-run → exit 0.
2. `scripts/check-layout.mjs` — deleted the `'window.getComputedStyle'` entry (plus its comment)
   from `ALLOWED_GLOBAL_MEMBERS`. `npm run check:layout` → exit 1, Kiểm C red line:
   `src/layout/WorkspaceDock.vue:556 — \`window.getComputedStyle\` KHÔNG có trong danh sách cho
   phép` — names `WorkspaceDock.vue`, the file Phase 2 wired `computeWorkArea` into, exactly as
   expected. Restored, `shasum -a 256` match confirmed, re-run → exit 0.
3. (after the release case existed) `src/commands/focus.ts` — removed the two lines `const
   current = byOwner.get(owner)` / `if (expected !== undefined && current !== undefined &&
   current !== expected) return`. `npx vitest run tests/frontend/workspaceLayoutTier.test.ts` →
   exit 1, 1/49 red, all others stayed green: `vượt mặt — khai lại bằng owner khác (b) rồi gỡ
   MUỘN bằng resolver cũ (a) ⇒ b SỐNG SÓT, không bị gỡ nhầm` — `AssertionError: expected false to
   be true` on `registry.has('panel.lookup')`, i.e. the late release deleted `b`'s live
   declaration instead of skipping it — the right reason, not a signature mismatch. Restored,
   `shasum -a 256` match confirmed, re-run → 49/49 green.
4. Self-counter-check on my own ladder cases — flipped `if (width < t.minFullWidth)` to `if
   (width <= t.minFullWidth)` in `layoutTierFor`. `npx vitest run
   tests/frontend/workspaceLayoutTier.test.ts` → exit 1, 6/49 red, all three inclusive-boundary
   cases that touch `minFullWidth` (both presets' "Full layout — đúng biên" plus the two "Short
   window" boundary cases whose width sits at `minFullWidth`), each failing `expected 'narrow' to
   be 'full'`/`'short'` — the boundary assertions do catch the mutation. Restored, `shasum -a 256`
   match confirmed, re-run → 49/49 green.

**Row → case map (I/O & Edge-Case Matrix, lines 68-84):**

| Matrix row | Covered by |
|---|---|
| Full layout | both presets, exact-boundary + comfortably-above cases |
| Short window | exact `minFullWidth` + `minFullHeight-1`, and exact `minShortHeight` boundary |
| Narrow or very short (width branch) | exact `minSupportedWidth` boundary + `minFullWidth-1`, height held comfortable |
| Narrow or very short (height branch, "very short") | width held comfortable, exact `minShortHeight-1` |
| Unsupported | exact `minSupportedWidth-1`, and a case with width=1/height=100000 (width beats a generous height) |
| Boundary exactly on a number | folded into each tier's boundary case above, both inclusive directions |
| Preset switched at a fixed size | one case, see finding below |
| Reaching the unsupported tier (top-down precedence) | one case: width below `minSupportedWidth` AND height below `minShortHeight` together ⇒ `unsupported`, not `narrow` |
| Non-finite width/height/both (NaN/±Infinity/undefined) | full cross of 4 values × 2 presets × 3 shapes (width/height/both) = 24 cases, all assert `null` |
| `FocusRegistry.release` bare / matching-expected / late-with-override | 3 cases |

**Left for Phase 4b (needs `WorkspaceDock.vue` mounted):** "Lookup reached from the status bar",
"Window grows back", "User hides a panel by hand, then resizes", "Reaching the unsupported tier"
in its dragging sense (Decision 1's `minWidth` lowered so the tier is reachable by hand) — none of
these are pure-number claims, all four need a mounted component and are out of this phase's scope
per the brief.

**Finding — preset-switch row (§I/O Matrix "Preset switched at a fixed size"):** `LAYOUT_THRESHOLDS`
seeds `layout.preset_grid` (Ⓑ-2) and `layout.preset_columns` (Ⓑ-1) with the **same** four numbers
today (`workspaceLayout.ts`'s own doc-comment: *"HẠT GIỐNG — cả hai preset cùng bốn số này hôm
nay"*). The task brief for this row asked for a case pinning a place where the two presets'
current numbers give different tiers, with a fallback of reporting instead of inventing one if
they are identical — they are identical, so the case written asserts `layoutTierFor` returns the
**same** tier for both presets at a shared `(w, h)`, with a comment naming this as the current-seed
finding, not a fabricated divergence. This case must be rewritten with two independent
expectations once Task 11 (Phase 5, human-measured) calibrates Ⓑ-1 and Ⓑ-2 apart.

**Counts measured myself, this phase, each its own exit code, read only after its own completion
sentinel (not through a pipe):** `check:lint` exit 0 · `check:layout` exit 0 · `check:i18n` exit 0
(confirmed it does not scan `tests/**` — its own tally names only `.vue` files under `src/`).
`npm run build` exit 0 (vue-tsc both configs, including the test tree via `tsconfig.json`'s
`include`, + `vite build`, 190 modules). `npx vitest run` exit 0, **92 files / 1376 tests passed**
— Phase 3b's own baseline was 91/1327; the delta is exactly this phase's one new file, 49 new
cases, nothing else moved. `cd src-tauri && cargo test --locked` exit 0, **1806 passed, 0 failed**
across every binary — byte-identical to Phase 3b's total, expected: no Rust file was touched this
phase. `npm run test:story 4-12` exit 0, 15.8s, ran 2 Rust targets (`config_invariants`,
`ipc_contract`) + `tests/frontend/workspaceLayoutTier.test.ts` (49/49 green) — **and reported
THIẾU KHAI** naming exactly `tests/frontend/workspaceLayoutTier.test.ts`: §Code Map → "Tests that
move" (line ~178-180) still says only "a new file for the pure ladder and for the tier-vs-manual-
hide rule" without the concrete path. This is a spec gap, reported here rather than silently
patched into Code Map or hidden by picking a name the spec already used.

**Not done, left for Phase 4b:** the manual-hide-survives-a-tier-change case (needs a mounted
`WorkspaceDock.vue`, per the brief's explicit "Do NOT open `src/layout/WorkspaceDock.vue`" for this
phase) and the four matrix rows listed above it.

### Phase 4b (2026-09-23) — mounted `WorkspaceDock`/`WorkspaceMode`, claims A–F

**Feasibility (Step 0).** `dockview` mounts under `happy-dom` with no new `setup.ts` patch. A
probe (`mount(WorkspaceDock, { props: { savedLayout: '' } })`) rendered the real `dv-*` DOM tree on
the first try; `data-layout-tier` read `undefined` until `await nextTick()` + a zero-delay
`setTimeout`, because `@ready`'s `measureAndApplyTier()` and `restoreFocusIfLost`'s
`requestAnimationFrame` do not resolve synchronously with `mount()`. Child panels (`GridPanel` /
`LookupPanel` / `AiTranslationPanel`) mount for real and call `invoke()` through the `src/config/*`
adapters, which never throw (`src/AGENTS.md`) — running outside Tauri only prints a diagnostic
line, it never fails a case. No `setup.ts` patch was added; `ResizeObserver` and `document.fonts`
already covered what `dockview` and the panels touch.

**New file:** `tests/frontend/workspaceDockTier.test.ts` (one file — reads better as one, since all
six claims share the same mount/resize helpers). 6 real `it()` cases + 1 `it.fails()` (a documented
finding, see below) = 7 tests, all green, `npx vitest run tests/frontend/workspaceDockTier.test.ts`
twice in a row: exit 0 both times, 6 passed | 1 expected fail (7) both times.

**Claim → case map, and I/O Matrix rows covered:**

| Claim | Case(s) | I/O Matrix row | Mechanism exercised |
|---|---|---|---|
| A — direct persist door | `A` (green) + `A(finding)` (`it.fails`) | §Always "an automatic tier change must never be persisted" (not a numbered row) | `full → narrow` (autoHide, clean); `full → short` (`applyMerge`, leaks — see finding) |
| B — indirect persist door | `B` | same §Always clause, the `autoHiddenIds.size > 0` half | `full → narrow` then a genuine `layout.toggle_lookup` while `ai_translation` still tier-hidden, then `beforeunload` |
| C — manual hide vs. automatic tier | `C` | "User hides a panel by hand, then resizes" | manual hide survives `narrow → full`; tier-hidden panel returns; a panel manually re-shown mid-`full` is re-hidden at the *next* `narrow` application |
| D — window grows back | `D` | "Window grows back" | `full → narrow → full`; all three panels return; JSON panel-id sets compared before/after (equal); full JSON (incl. sash sizes) measured, not assumed |
| E — unsupported notice | `E` | "Unsupported" (the notice half; the numeric ladder is Phase 4a's) | mounts `WorkspaceMode`, asserts `[data-workspace-narrow-notice]` text + grid still mounted, both directions |
| F — lookup from status bar | `F` | "Lookup reached from the status bar" | mounts `WorkspaceDock` + `StatusBar` + `LookupDrawer` as siblings (App.vue's own shape) with `installCommands` wired through `dockController`/`lookupDrawerState`, clicks `[data-lookup-drawer-open]`, asserts the drawer renders `panel.lookup.title` |

**Row not reached, and why:** "Reaching the unsupported tier" in its *dragging* sense (Decision
1's lowered `minWidth`, "the tier is reachable by dragging, so the notice can be exercised by
hand") is a geometry/manual claim — `happy-dom` computes no layout, and the spec's own §Never says
"do not assert geometry in vitest." This row belongs to the manual bench or e2e, not to this file.

**Finding (not a defect in this test file — a real product gap, `it.fails`, ledger below):**
`full → short` (the `applyMerge` branch) emits ONE `persist` roughly `IDLE_MS` (500 ms) after
`applyTier` returns, even though `suppressPersist` wraps both the `removePanel`/`addPanel` calls of
the merge. Root cause, measured directly (temporary `console.log` inside `onLayoutChange`, removed
before the final restore): `dockview` does not fire `onDidLayoutChange` synchronously for every
mutation — when two panel mutations happen in the same synchronous tick (a merge's remove+add, or
a grid-only sacrifice's two sequential auto-hides), it coalesces them into ONE notification that
fires on a later microtask, by which point any same-tick flag (`suppressPersist`, reset to `false`
at the end of that same tick) has already gone false. The narrow/unsupported sacrifice path stays
clean only because `autoHiddenIds` has *already* reached its final non-zero size by the time that
deferred notification fires, so the OTHER half of the guard (`autoHiddenIds.size > 0`) still blocks
it — `applyMerge` has no equivalent second guard, so it leaks. Direct counter-proof: mounting
straight into `short` (no prior `full`) is clean — the leak needs the group-disposal that happens
when `ai_translation` is removed from a group it occupied alone, not the merge call itself. Not
fixed here (coverage-only phase); recorded in `deferred-work.md` with Ice as owner.

**Counter-check 3 (Step 2), after A and B existed — each done singly, source restored from a
scratchpad copy between mutations, `shasum -a 256` matched before AND after every restore:**

1. Drop `suppressPersist ||` alone (`onLayoutChange` becomes `if (autoHiddenIds.size > 0) return`).
   Ran the file: **all 7 cases stayed green**, including `A`. Read why (see Finding above): the
   narrow-tier sacrifice batches two auto-hides into one deferred notification that fires only
   after `autoHiddenIds` already holds both ids, so `autoHiddenIds.size > 0` alone is already
   sufficient for every case this file exercises — **`suppressPersist` guards nothing that this
   suite can observe.** This is the most important finding of the counter-check, stated plainly
   per the brief. Restored; `shasum -a 256` matched the pre-mutation copy; re-ran green (6 passed |
   1 expected fail).
2. Drop `|| autoHiddenIds.size > 0` alone (`onLayoutChange` becomes `if (suppressPersist) return`).
   Ran the file: **A, B, and D went red** — all three `AssertionError: expected 2 to be 1` (a
   `persist` emit count, not a crash), i.e. an extra flush actually landed. `C` (visibility-only,
   no persist-count assertion) and `E`/`F` (no persist-count assertion) stayed green, and
   `A(finding)` still failed-as-expected. Restored; `shasum -a 256` matched; re-ran green.
3. Drop `syncLayoutTier(tier)` from `applyTier` alone. Ran the file: **`F` went red**
   (`AssertionError: expected false to be true` — the status-bar entry never appears, because
   `lookupHasRetreated` never flips), **`E` stayed green** (it depends on `tier-change`, not
   `syncLayoutTier`). Restored; `shasum -a 256` matched; re-ran green.
4. Drop `emit('tier-change', tier)` from `measureAndApplyTier` alone. Ran the file: **`E` went red**
   (`AssertionError: expected false to be true` — `WorkspaceMode` never learns the tier, so the
   notice never renders), **`F` stayed green**. Restored; `shasum -a 256` matched; re-ran green.

Net verdict on counter-check 3: cases B, E, F are each independently guarded by the seam named for
them. Case A is guarded — but by `autoHiddenIds.size > 0` alone, not by `suppressPersist`; the
`suppressPersist` half of that same `if` is dead weight for every reachable path in this codebase
today (see Finding). This is reported, not silently fixed — `suppressPersist` is production code,
out of scope for a coverage-only phase.

**Measured counts, each command's own exit code, read after its own sentinel:** `check:deps` ·
`check:tokens` · `check:i18n` · `check:commands` · `check:layout` · `check:panel-refs` ·
`check:dict` · `check:dict-manifest` · `check:gates` · `check:debt-owner` · `check:lint` — all
exit 0. `npm run build` exit 0 (190 modules, unchanged). `npx vitest run` (full suite) exit 0,
**93 files / 1382 passed | 1 expected fail (1383)** — delta from Phase 4a's 92/1376 is exactly this
phase's one new file (6 real cases + 1 `it.fails`). `cd src-tauri && cargo test --locked` exit 0,
65 binaries, every `test result: ok`, 0 failed anywhere (no Rust file touched this phase, so this
re-confirms Phase 4a's byte-identical claim rather than measuring something new). `npm run
test:story 4-12` exit 0, 62.6s, ran 2 Rust targets (`config_invariants`, `ipc_contract`) +
`tests/frontend/workspaceDockTier.test.ts` + `tests/frontend/workspaceLayoutTier.test.ts`, 55
passed | 1 expected fail (56), **no THIẾU KHAI** (the new file is named in §Code Map, see the
🔵 2026-09-23 (Phase 4b) line added there).

**Setup.ts patches added:** none. The two existing entries (`document.fonts`, `ResizeObserver`)
already covered everything `dockview` and the three panels touch under `happy-dom`.

### Phase 4c (2026-09-23) — grow-back + merged-mutation cases, `A(finding)` closed, orchestrator's Fix 1/Fix 2 counter-checked

Orchestrator measured and fixed the leak Phase 4b's `A(finding)` pinned (`endSuppressPersist`
now resets `suppressPersist` via `queueMicrotask` — FIFO, so it runs AFTER dockview's own
`AsapEvent` microtask, not before it — Fix 1) and a second leak found while fixing it
(`onLayoutChange`'s guard grew an `|| tierMerged.value` clause — Fix 2). This phase converts the
finding into a real assertion, corrects a claim `A`'s own comment made about its own guard, adds
three new cases for the two fixes, and counter-checks all four disjuncts of the guard one at a
time.

**Cases changed:**
- `A(finding)` → `A2` (renamed `it.fails` → `it`): asserts NO persist after `full → short` over a
  full idle+hard-cap window (6000ms fake time). The comment block was rewritten to describe the
  guard now in place (Fix 1's microtask ordering) instead of a bug still open.
- `A`'s own inline comment (the "⚠️ Đây là lượt hy sinh HAI panel..." block) claimed the FIRST
  auto-hide's `onLayoutChange` fires while `autoHiddenIds` is still empty, so `A` depends on
  `suppressPersist` alone. Measured false: `onDidLayoutChange` is a microtask, and
  `applyGridOnlySacrifice`'s `for` loop synchronously hides BOTH panels (queuing both `autoHide`
  calls, and both `autoHiddenIds.add` calls) before the JS call stack unwinds and any queued
  microtask gets a turn — so by the time the FIRST `onLayoutChange` microtask fires,
  `autoHiddenIds` already holds both ids. Corrected in place (see counter-check `d` below for the
  direct confirmation: dropping `suppressPersist` from the guard entirely leaves `A` green).

**Cases added** (same file, same `describe` block, after `F`):
- `G` — grow back from narrow (`full → narrow → full`) persists nothing, checked across a full
  6000ms fake-time window on BOTH legs.
- `H` — grow back from short (`full → short → full`) persists nothing, same shape, also asserts
  `data-layout-merged` flips `true`→`undefined` across the round trip.
- `I` — a genuine user action (`togglePanel('panel.lookup')` hide+show) WHILE still merged at
  `short` persists nothing — the case Fix 2 (`tierMerged.value` guard) exists for. Waits past idle
  with REAL timers (~700ms) before recording the baseline, per instruction, rather than fake time.

All 10 cases in the file green against the fixed product (verified standalone:
`npx vitest run tests/frontend/workspaceDockTier.test.ts` → 10/10 passed, run before any
counter-check).

**Counter-checks — one disjunct of `onLayoutChange`'s
`if (suppressPersist || autoHiddenIds.size > 0 || tierMerged.value) return` removed at a time,
each restored from the `.fixed` backup + `shasum -a 256` match before the next, file run alone
each time:**

a. `endSuppressPersist` body replaced with a synchronous `suppressPersist = false` (Fix 1
   reverted, guard clauses left intact). **`G` and `H` went red** —
   `AssertionError: expected 2 to be 1` (persist-count mismatch on the grow-back leg, not a
   crash/timeout) both times. `A`, `A2`, `B`, `C`, `D`, `E`, `F`, `I` stayed green (8/10). `A2`
   staying green here is NOT evidence Fix 1 guards it — `tierMerged.value` (Fix 2, untouched by
   this removal) covers the entire `short`-tier window on its own, independent of `suppressPersist`
   timing; `A` stays green for the reason in the corrected comment above (`autoHiddenIds` already
   non-empty by the time either microtask ordering delivers the event). Restored; sha matched;
   re-ran 10/10 green before the next removal.
b. Drop `|| tierMerged.value` alone (`if (suppressPersist || autoHiddenIds.size > 0) return`).
   **Only `I` went red** — `AssertionError: expected 1 to be +0`, a real persist landing from the
   genuine toggle made while merged. All 9 other cases stayed green, including `A2` — the
   FIRST-ever `full → short` transition's own delayed "active view" `onDidLayoutChange` (the
   original finding) is caught by Fix 1's microtask ordering on its own; `tierMerged.value`'s
   unique, non-redundant job (per this removal) is guarding a SUBSEQUENT genuine user mutation
   while still merged, which is exactly what `I` tests. Restored; sha matched; re-ran green.
c. Drop `|| autoHiddenIds.size > 0` alone (`if (suppressPersist || tierMerged.value) return`).
   **Only `B` went red** — `AssertionError: expected 2 to be 1`, the genuine `panel.lookup`
   toggle-while-`panel.ai_translation`-still-tier-hidden case flushing a layout missing a panel.
   `A` stayed green here too (guarded redundantly by `suppressPersist`, since the FIRST
   `onLayoutChange` microtask still finds the flag `true` under FIFO ordering — this is the SAME
   event `A`'s corrected comment discusses, just from the other guard's side). Restored; sha
   matched; re-ran green.
d. Drop `suppressPersist ||` alone (`if (autoHiddenIds.size > 0 || tierMerged.value) return`).
   **`G` and `H` went red again**, same message shape as (a) — dropping the flag from the guard
   has the same observable effect on the grow-back cases as reverting its reset timing did, because
   for those two cases `autoHiddenIds` is empty and `tierMerged` is `false` by the time the leak's
   delayed event fires (both fixed-tier books are already closed). `A` and `A2` stayed green
   (covered redundantly by `autoHiddenIds`/`tierMerged` respectively, as in (a)). Restored; sha
   matched; re-ran green.

Every red message read was a `persist`-count `AssertionError`, never a crash or timeout — each
removal broke the exact invariant it should, nothing else. No case stayed green under all four
removals in a way that means it "guards nothing": `A` and `A2` each have genuine, independently-
sufficient double coverage (documented above, confirmed by which single-axis removal each one
survives and which combination of two would be needed to turn either red — not tested, out of
scope: the task specified single-disjunct removals only), while `B`, `G`, `H`, `I` each have
exactly one guard clause responsible for them and go red the moment that clause is missing.

**Counts, each command's own exit code:** the 11 gates (`check:deps`, `check:tokens`,
`check:i18n`, `check:commands`, `check:layout`, `check:panel-refs`, `check:dict`,
`check:dict-manifest`, `check:lint`, `check:gates`, `check:debt-owner`) all exit 0. `npm run
build` exit 0. `npx vitest run` (full suite) exit 0, **93 files / 1386 passed, 0 failed** — delta
from Phase 4b's 93/1382+1-expected-fail(1383) is +3 net (cases `G`, `H`, `I` added; `A(finding)`'s
expected-fail slot became `A2`'s real pass, net zero on that one). `cd src-tauri && cargo test
--locked` exit 0, **65 binaries, every `test result: ok`, 0 failed** (no Rust file touched this
phase — re-confirms, does not re-measure). `npm run test:story 4-12` exit 0, 13.3s, ran the same
2 Rust targets (`config_invariants`, `ipc_contract`) + the same 2 frontend files
(`workspaceDockTier.test.ts`, `workspaceLayoutTier.test.ts`), no THIẾU KHAI.

**`shasum -a 256 src/layout/WorkspaceDock.vue`** after the last restore matches the `.fixed`
backup (`38558de2bea5897f741b5a68a96b398e209ccfb558e2677459d948c03558ea4`…, see phase file) —
production code left byte-identical to what Phase 4b's orchestrator fixed; only the test file
changed this phase.

### Phase 4 — closed 2026-09-23 by the orchestrator: the persist guard was open, and is now fixed

Phase 4b's `A(finding)` was a real breach of §Always, and it was NARROWER than the truth. I
measured it again with a separate probe (fake timers, 6000 ms per leg), then deleted the probe:
- `full → short` persisted the merged group.
- `narrow → full` and `short → full` each persisted the layout the tier had just rebuilt.
- At `short`, a genuine user toggle followed by `beforeunload` persisted a group holding both
  `panel.ai_translation` and `panel.lookup`. The next session opens at full size with
  `tierMerged === false`, so nothing ever un-merges it.

Root cause, read in `dockview-core` 7.0.4: `onDidLayoutChange` is an `AsapEvent`
(`queueMicrotask`). The synchronous `suppressPersist = false` therefore ran before the event, and
the flag had never blocked anything. Phase 2's doc-comment said the opposite. It is corrected in
place with 🔵.

This was fixed inside the story, not deferred, because the rule it breaks is this story's own frozen
§Always rule:
1. `endSuppressPersist()` resets the flag in a microtask that is FIFO-after dockview's.
2. `onLayoutChange` gains `|| tierMerged.value`. That is Ice's 2026-09-22 ruling, applied to the
   merged state. ⚠️ **It is not a ruling Ice signed for that state.** The cost is the same as
   before: a sash dragged at `short` is not saved.

The `deferred-work.md` item Phase 4b opened is closed with ✅ and its true scope.

Phase 4c's counter-checks show that each disjunct has its own reddening case:
- `suppressPersist` → G, H
- `autoHiddenIds` → B
- `tierMerged` → I

A and A2 are each covered by two disjuncts, so a single removal leaves them green. That is
double coverage, not zero, and the per-disjunct cases above are the proof that each half guards
something.

Re-measured on a still tree (`git status --short | md5` identical before and after):
- all eleven gates 0
- `EXIT_BUILD=0`
- `EXIT_VITEST=0`: **93 files / 1386 passed**
- `EXIT_CARGO=0`: **1806 passed, 0 failed, 22 ignored, 65 `test result:` lines**
- `EXIT_STORY=0`: `test:story 4-12` ran 59 frontend cases and 2 Rust targets, no THIẾU KHAI

### Phase 4d (2026-09-23) — the return direction is read from the tree (written by the orchestrator)

The phase agent built and counter-checked this work, then had to stop before writing its notes.
These notes are written from the diff and from runs I read myself.

**What changed.** New pure, import-free file `src/layout/dockTree.ts`. It exports
`findTreeSpot(grid: SerializedGrid, id: string): RememberedTreeSpot | null` and `viewsIn`, plus the
types `GridOrientation`, `GridNode`, `SerializedGrid`, `PlacementDirection` and
`RememberedTreeSpot = { reference; direction }`. It is ONE walker: the old `siblingInTree`/`viewsIn`
left `WorkspaceDock.vue`. `rememberSpot` now calls `siblingSpotInTree(api, id)`, which is
`findTreeSpot(api.toJSON().grid, id)`, and `boundingBox` is gone from that path. Its doc-comment is
corrected in place with 🔵.

**The orientation rule, as the agent printed it from a real `api.toJSON()`.** `grid.orientation`
belongs to the root branch only; nested branches carry no orientation field and alternate by
depth. Ⓑ-2 has root `HORIZONTAL` `[grid, VERTICAL[lookup, ai]]`; Ⓑ-1 has root `VERTICAL`
`[grid, HORIZONTAL[lookup, ai]]`. dockview collapses a branch left with one child, so after
`ai_translation` is hidden, `lookup` sits directly under the root.

**Cases.** `tests/frontend/dockTree.test.ts` has 9 pure cases on fixtures copied from real
`toJSON()` output. In `workspaceDockTier.test.ts`, G and H now also assert that the rebuilt shape is
`(grid|(lookup/ai_translation))`, and a new case J asserts that Ⓑ-1 after
`full → narrow → full` is `(grid/(lookup|ai_translation))`.

**Counter-check, re-run by me at the seam.** I forced `siblingSpotInTree` to return
`direction: "right"` while keeping the anchor. `EXIT_CC=1`, and exactly G, H and J went red with
shape mismatches: `expected '(grid|lookup|ai_translation)' to be '(grid|(lookup/ai_translation))'`
and, for J, `expected '((grid|lookup|ai_translation))' to be '(grid/(lookup|ai_translation))'`.
This is the three-column layout Ice found on disk, reproduced on demand. The file was restored
from a scratchpad copy and the SHA matched.

**Counts.** The agent's background runs, read by me only after their sentinels:
- `EXIT_VITEST=0`, **94 files / 1396 passed**
- `EXIT_CARGO=0`, **1806 passed, 0 failed, 22 ignored, 65 `test result:` lines**

The agent reported the eleven gates, the build and `test:story 4-12` as exit 0. I re-run all of
these myself at Phase 4e's close. Agent context ~236k.

### Phase 4e (2026-09-23) — "giữ gộp, lưu dạng chưa gộp" (Ice's ruling)

**What changed.** `src/layout/dockTree.ts` gained `unmergeForPersist(dock, panelId, spot)` and the
type `SerializedDockJSON`. It is a NEW cây-dựng function, opposite of Phase 4d's `findTreeSpot`
(which reads a tree): given a full `api.toJSON()`-shaped object, a panel id sharing a leaf with
another panel, and a remembered spot, it returns a NEW object with that panel split into its own
leaf next to the reference leaf, in the remembered direction — never mutating the input.

Real `toJSON()` was printed from a mounted dock via a scratch test (`node:fs.appendFileSync`,
deleted after) to read the real shapes: every node (leaf AND branch) carries `size`; every leaf
carries `id`, a plain incrementing digit string dockview assigns and never reuses after a group is
removed (`"1"`, `"2"`, `"4"` — `"3"` already gone). `panels` is a dict keyed by panel id with no
group reference; `activeGroup` is the active leaf's `id`.

**Algorithm.** `locateLeaf` walks the tree like `findTreeSpot` but returns the actual node (to
mutate on a clone) plus `{ parent, index, axis }`, `axis` being the orientation the leaf's PARENT
uses to arrange its children. Two branches: if the reference leaf's parent already uses the axis
`spot.direction` needs, splice a new sibling leaf in (before/after), splitting `size` in half; else
wrap the reference leaf in a new perpendicular branch holding both (order by direction), replacing
it in its parent — or, if the reference leaf IS `grid.root` (no parent), the new branch becomes
`grid.root` and `grid.orientation` is updated to match, since the depth-alternation model needs the
root's own orientation to agree with what it actually contains. No-op (same reference returned)
when the panel isn't sharing a leaf with `spot.reference` — already dragged out by hand, or the
panel/reference no longer exist together.

**Bug caught by the pure tests before wiring.** `locateLeaf`'s first draft read the LEAF's own
recursive `orientation` parameter as its `axis` — that parameter is the axis the leaf would use for
its OWN children if it were a branch (i.e. already flipped one level too far), not the axis its
PARENT uses to place it among siblings. Every "should wrap" case instead did a flat sibling-insert
at the wrong depth. Fixed by carrying `parentAxis` explicitly through the recursion (mirroring how
`findTreeSpot` captures `orientation` at `path.push` time, before flipping for the recursive call).
Caught by `tests/frontend/dockTree.test.ts`'s Ⓑ-2/Ⓑ-1 shape assertions before the wiring step ran
at all.

**Wiring.** `WorkspaceDock.vue` gained `jsonForPersist(api)`, called from `flush()` in place of a
bare `api.toJSON()`: when `tierMerged.value && mergedSpot !== null && mergedSpot.direction !==
'within'`, it returns `unmergeForPersist(json, 'panel.ai_translation', { reference:
mergedSpot.reference, direction: mergedSpot.direction })`; otherwise the raw JSON, unchanged. The
`'within'` exclusion is new: `mergedSpot.direction` can in principle be `'within'` (if
`panel.ai_translation` was tabbed with some OTHER panel, not `panel.lookup`, right before the tier
merged it) — a shape `unmergeForPersist` isn't built to reconstruct (it only knows the four grid
directions). That case falls back to the raw JSON — the pre-Phase-4e behaviour, not a new
regression, and documented in place as an open edge rather than silently handled.

`onLayoutChange`'s guard dropped `|| tierMerged.value` — it is now just `suppressPersist ||
autoHiddenIds.size > 0`. The doc-comment block that used to read "🔴 VÀ VẾ `tierMerged.value`" is
now a "🔵 2026-09-23" note recording Ice's ruling, why the old absolute block was too strong (a real
sash-drag or panel toggle while merged used to be swallowed, same shape as the bug
`writeSchedule.ts` exists to prevent), and where the new un-merge-for-persist logic actually lives
(`jsonForPersist`, not the guard). `suppressPersist`/`endSuppressPersist` — the tier's OWN
merge/unmerge suppression — is untouched.

**Cases.** `tests/frontend/dockTree.test.ts`: 12 new cases for `unmergeForPersist` — Ⓑ-2 merged +
`{lookup, below}` and Ⓑ-1 merged + `{lookup, right}` (both wrap cases, fixtures built from the real
printed shape, since Story 4.12's real merge always collapses the shared leaf onto the root, making
the remembered nested-branch direction always mismatch the merged leaf's now-root-level axis); one
hand-built matching-axis case exercising the OTHER branch (sibling-insert, no wrap — not reachable
from a real Story 4.12 merge, since merge always promotes the shared leaf to be a direct child of
root); two no-op cases (already-separated panel, and a panel absent from the tree); input-not-
mutated; `activeView` fixed when it pointed at the removed panel; size split in half; new leaf id
uniqueness (`"3"` when `"1"`/`"2"` are taken). `tests/frontend/workspaceDockTier.test.ts` case **I**
was rewritten from "persists nothing" to "persists, in un-merged shape": full → short (merge) →
toggle `panel.lookup` off/on (a real user action through `layout.toggle_lookup`) → `beforeunload`,
then asserts `persist` count grew past baseline and the latest payload's shape is
`(grid|(lookup/ai_translation))` with no leaf holding both panel ids. Case A2 (tier merge itself
persists nothing) is untouched and still green.

**Counter-checks, all three restored + SHA-verified between (`shasum -a 256`, scratchpad backup at
`WorkspaceDock.vue.orig`).**

| # | Mutation | Result | Failure read |
|---|---|---|---|
| a | `jsonForPersist` returns raw `api.toJSON()` unconditionally (transform call removed) | RED | `expected '(grid\|ai_translation,lookup)' to be '(grid\|(lookup/ai_translation))'` — case I, a genuine shape assertion: the merged group got persisted as-is |
| b | `\|\| tierMerged.value` put back into `onLayoutChange`'s guard | RED | `expected 0 to be greater than 0` — case I, no persist fired at all |
| c | `endSuppressPersist`'s body replaced with a synchronous `suppressPersist = false` | RED (A2, G, H); GREEN (I) | A2/G/H: `expected 2 to be 1` (an extra persist fires because the flag clears before dockview's own `AsapEvent` microtask, same defect class Phase 4c fixed originally) — genuine count assertions, not crashes. I stayed green because it depends on `endSuppressPersist` only through `applyMerge`'s bracketing (which still runs, sync or not) and its own assertions never rely on the *count* staying at exactly the tier-merge baseline the way A2/G/H do — it only checks the LAST persisted payload's shape, and that payload is unaffected by whether one extra spurious persist fired earlier in the sequence. |

**Gates run so far.** `check:layout` EXIT=0 (all five checks OK, including Check B on write-cadence
and Check E on the four tier thresholds — neither touches this phase's surface but both re-verified
clean). `check:panel-refs` EXIT=0 (78 `.ts` files, 407 module-scope references, 33 named
exemptions, self-check green). Scoped `npx vitest run tests/frontend/dockTree.test.ts
tests/frontend/workspaceDockTier.test.ts`: **30 passed** (19 + 11).

**Pending at write time** (filled in after the background run's sentinel, never read early): full
`npx vitest run` file/test counts, `cargo test --locked` counts, remaining nine gates, `npm run
build`, `npm run test:story 4-12`.

🔵 Filled 2026-09-23 by the orchestrator, from the agent's `full_run.log` read after
`SENTINEL_DONE`, on the Phase 4e tree before the correction below:
- `EXIT_VITEST=0`, **94 files / 1406 passed**
- `EXIT_CARGO=0`, **1806 passed, 0 failed, 22 ignored, 65 lines**

### Phase 4e — corrected and closed 2026-09-23 by the orchestrator

Phase 4e's transform leaked the merged layout through two doors, both found by reading the diff
rather than the report:
1. It placed the split-out panel next to the SHARED leaf, and it did nothing unless that leaf also
   held `spot.reference`. The pre-merge anchor is the panel's sibling in the TREE, not
   necessarily its merge partner. In Ice's actual on-disk layout `grid | ai_translation | lookup`
   the anchor is `grid`, so the raw merged group was written.
2. `jsonForPersist` excluded `direction === 'within'` and wrote raw JSON, under a comment claiming
   "same behaviour as before Phase 4e". Before 4e the `tierMerged` half blocked the write entirely,
   so that claim was false.

**Fix.** `unmergeForPersist` now removes the panel from whichever leaf it shares, then places it
around the leaf that holds `spot.reference`. `within` joins that leaf's views. The new
`UnmergeSpot` type admits `within`, and `jsonForPersist` passes `mergedSpot` unfiltered. Both
doc-comments are corrected in place with 🔵.

**New cases.**
- Mounted case K loads Ice's real on-disk layout verbatim as `savedLayout`, goes full → short, makes
  a genuine toggle, then `beforeunload`. It must persist `(grid|ai_translation|lookup)`.
- Three pure cases: anchor `grid` / `right`; anchor `grid` / `within`; and a `within` no-op.
- Case I's two `not.toContain('lookup,ai_translation')` assertions ran on RAW JSON, where views
  read `"panel.lookup","panel.ai_translation"`, so they held on both branches. They are replaced
  by `expect(shapeOf(json)).not.toContain(',')`.

**Counter-check.** I restored both files to the agent's 4e version. `EXIT_CC=1`, 4/34 red,
including K with `expected '(grid|ai_translation,lookup)' to be '(grid|ai_translation|lookup)'`,
i.e. the merged group on disk. I then restored from scratchpad copies, the SHAs matched, and the
run was 34/34 green.

**Full bar, still tree** (`git status --short | md5` identical before/after):
- eleven gates 0
- `EXIT_BUILD=0`
- `EXIT_VITEST=0`: **94 files / 1410 passed**
- `EXIT_CARGO=0`: **1806 / 0 / 22 / 65**
- `EXIT_STORY=0`: `test:story 4-12` ran 83 cases, no THIẾU KHAI

Agent context ~255k.

### Phase 5 (2026-09-23) — calibration, closed by Ice's ruling rather than by measurement (Task 11)

Run by the orchestrator with Ice. No agent was dispatched.

**Conditions.**
- Build: DEBUG (`npm run tauri dev`, started in the orchestrator's session).
- Dictionary: `target/debug/dict` did not exist, so the dev build had no dictionary layers, because `bundle.resources` does not declare `dict`. I symlinked it to `../release/dict` (4 layers, 356 MB). The dev log then read `dict[layers] 4 layer(s) loaded`. The symlink lives under `target/` and is still there.
- Why not the existing `.app`: the release bundle was built 2026-09-15 00:31, before any Story 4.12 code, so it was unusable.
- AI: no provider was configured, so the AI Translation panel showed its "no provider" empty state.
- Ⓑ-2 was in the "Chuyển đổi" view (Decision 3).

**Instrument.**
- `osascript`/System Events sets the window size, and the size of the `AXScrollArea` (the web view) is read back after every set.
- The native title bar measured **28 pt** (window 1486×935 against web area 1486×907).
- Work area = web-area height − 74.
- Screen: 1536×960 pt with a 25 pt menu bar, so **the tallest reachable work area is 833**.
- To judge a tier below a threshold, I edited `SEEDED_THRESHOLDS_B1/_B2` in place so that every tier was forced to `full`. Before editing I backed the file up to scratchpad (sha256 `4a71de73…`), and afterwards I restored it from that backup, not from HEAD. The sha matched.
- `global.db` was backed up with `sqlite3 .backup` before anything was resized.

**The one measurement.**

| Preset | Forced tier | Work area | Verdict (Ice) |
|---|---|---|---|
| Ⓑ-2 | full | 1280×833 | usable |

After that one measurement Ice ended calibration and ruled the four seed numbers final for both presets. No size was judged unusable, so no threshold was located, and Ⓑ-1 was not measured. A second ruling followed: the default window 1280×860 (work area 1280×786) opens in `short`, which supersedes the earlier "opens in FULL" ruling (see §Spec Change Log). `workspaceLayout.ts` is unchanged, and so are `check-layout.mjs` Kiểm E and every test that pins these numbers.

**Hand checks on the running build**, restored thresholds, Ⓑ-2. Each screenshot is `screencapture -l <CGWindowID>` of the app window only.

| Work area | Tier on screen |
|---|---|
| 1280×786 | `short`: Lookup and Đề xuất AI in one tab group beside the grid |
| 1000×833 | `narrow`: grid only, with a "Tra cứu" entry in the status bar |
| 820×626 | `unsupported`: the notice sits above a rendered, readable grid, and "Tra cứu" is still in the status bar |

This confirms by hand the Matrix row "Reaching the unsupported tier" with `minWidth` 800.

Grow-back: I resized 1486×833 → 1280×786 → 1000×833 → 820×626 → 1486×833 and waited 7 s for the 5000 ms hard cap after each end. The `config_value` rows for `workspace_layout`, `updated_at` included, were byte-identical before and after (sha256 `2c149ca5…` both times).

⚠️ Not hand-checked: opening the Lookup drawer from the status bar, because I did not click inside the app. The only proof of that row is Phase 3a/4b's mounted vitest cases.

⚠️ A first screenshot taken by screen region captured another application that was covering the app window. I looked at it once, saw it was the wrong window, and deleted it. It proves nothing.

### Phase 6 (2026-09-23) — planning anchors and ledger (Tasks 12, 13), by the orchestrator

- `SPEC.md` [A11]: gets 🔵 with the four numbers, marked as a ruling and not a measurement. It also corrects the pre-2026-08-14 wording (`giữ 2×2`, `gộp hàng dưới`, `rút về ngăn kéo`) against UX-DR15, states the 74 px chrome, and records that the default window opens in `short`.
- `SPEC.md` Q9: gets 🔵 ĐÃ ĐÓNG by ruling, saying plainly that its own closing condition, a measurement, was not met.
- `SPEC.md` "Thứ tự hy sinh panel…" bullet (§Constraints): gets 🔵, since it still said the numbers "sẽ được đo lại".
- `deferred-work.md`, two new items:
  - the `titlebar-height` 38 → 40 divergence, citing §Bảng token khoảng cách và hình dạng and the precedent in §Deferred from: 2-3-hop-dong-flush-va-trang-thai-da-luu, owner Ice;
  - "None of the four narrow-window thresholds is a measured number", owner Ice.
- The existing Ⓑ-2 parallel-gloss item in §Lượt Correct Course 2026-08-14 gets `→ 🟡`: the threshold half is closed, and the UX choice stays with Ice.

### Step-03 verification (2026-09-23, orchestrator) — matrix audit found two unguarded clauses

I read the code diff since `baseline_commit`: 3 311 lines, untracked files included, `_bmad-output/` excluded. All 15 Execution tasks are present.

**Matrix audit, row by row:**

| Matrix row | Covering case(s) |
|---|---|
| Full layout · Short window · Narrow or very short · Unsupported | `workspaceLayoutTier.test.ts` ladder cases for each preset, plus Kiểm E. Mounted: A, A2, E |
| Boundary exactly on a number | `workspaceLayoutTier.test.ts` "đúng biên", plus Kiểm E §2 |
| Preset switched at a fixed size | `workspaceLayoutTier.test.ts` "Preset switched at a fixed size". Mounted: L |
| Window grows back | D, G, H, J |
| User hides a panel by hand, then resizes | C |
| Lookup reached from the status bar | F (opens and renders), **F2 (new)**: focus returns on close |
| Reaching the unsupported tier | By hand in Phase 5 |

**Two gaps, both now closed:**
- The row "Lookup reached from the status bar" says *"closing it returns focus where it came from"*, and AC2 says *"focus … never on `body`"*. Neither had a case: there were zero `activeElement` asserts across the three test files.
- I added **F2** and **M** to `tests/frontend/workspaceDockTier.test.ts`. Both mount with `attachTo: document.body`.
  - F2 starts focus in the grid, opens and closes through `dispatch`, and expects focus back on the grid. It does not accept the opener.
  - M focuses Lookup at `full` and resizes to `narrow`. It asserts that the Lookup root is disconnected, then that focus is on a connected element and not on `body`.

**Counter-checks, one real removal at each seam** (file backed up, then restored and sha-verified):
- Deleting the `restoreFocusIfLost(fallback)` call in `applyTier`: M goes red, `expected <body> not to be <body>`. F2 stays green. 15/16.
- Deleting the `back.focus()` branch in `LookupDrawer.vue`: F2 goes red, `expected <button …> to be <section …>`, meaning focus fell back to the opener. F stays green, which confirms F never guarded this clause. 15/16.

⚠️ Unrelated noise, not caused by this change: when run with `--reporter=verbose`, cases A–L each log `[focus] … ĐÃ THÁO khỏi DOM`, because they mount without `attachTo`. The default reporter prints none of it.

**Full bar after the two cases, still tree** (`git status --short | md5` identical before and after):
- all eleven gates exit 0
- `EXIT_BUILD=0`
- `EXIT_VITEST=0`: **94 files / 1414 passed**, which is the 1412 measured on the same tree before the change plus the two new cases
- `EXIT_CARGO=0`: **66 binaries, 1806 passed / 0 failed / 22 ignored**
- `EXIT_STORY=0`

## Spec Change Log

- 2026-09-23 (orchestrator) — §Code Map "Tests that move" now names `tests/frontend/workspaceLayoutTier.test.ts` and `tests/frontend/workspaceDockTier.test.ts`, which Phase 4a/4b created; the prose named only the directory, so `test:story` reported THIẾU KHAI. No frozen content changed.

- 2026-09-23 (orchestrator, Ice's rulings) — two Execution tasks added before calibration:
  tree-based return direction, and un-merged persistence while merged. Both follow from what Ice
  saw in the running app. The stored `workspace_layout` (09:23 today) was three columns
  `grid | ai_translation | lookup`, not Ⓑ-2, written through the now-fixed persist leak, and its
  shape comes from the `boundingBox` direction defect. Ice also ruled that the default window
  1280×860 (work area 786 high) opens in the FULL tier with all three panes, which is a
  constraint on the Phase 5 calibration, not a new number. No frozen content changed:
  §Always already forbids persisting an automatic change, and the Matrix row "Window grows back"
  already demands every panel return to the spot it left.

- 2026-09-23 (orchestrator, Ice's rulings at Phase 5) — **frozen intent renegotiated by Ice.** Two rulings, and I record the conflict between them instead of hiding it.
  - **Ruling 1: the seed thresholds are final.** After one measurement (Ⓑ-2, full tier forced, work area 1280×833, usable), Ice ruled the four UX-DR15 seed numbers (1100 · 820 · 700 · 860) final for both Ⓑ-1 and Ⓑ-2. This departs from frozen AC3 (*"each traceable to a recorded measurement rather than to UX-DR15's seed"*) and from the Intent's *"calibrate the numbers on real hardware"*. The numbers are owned by that ruling, and `deferred-work.md` carries an owned item saying they are unmeasured.
  - **Ruling 2: the default window opens in `short`.** This supersedes the ruling recorded in the entry above, that the default window 1280×860 opens in FULL. Under the seed `minFullHeight` 820 the two rulings cannot both hold, and I put that conflict to Ice. Ice chose `short`. The hand check confirms it: a 1280×786 work area renders Lookup and AI Translation as one tab group. AC4 (*"the tier the app opens in is a measured decision that is written down"*) is satisfied as a decision written down, not as a measurement.

## Review Triage Log

Review loop 1, 2026-09-23. Three layers ran: blind (B1–B11), verification-gap (V1–V3), edge-case (E1–E3).

| # | Finding | Verdict | Evidence | Route |
|---|---|---|---|---|
| B1 · V2 · E1 · E2 | `applyMerge`'s "already tabbed" branch sets `tierMerged = true` with no `mergedSpot` | medium | Dock drag-and-drop is not disabled, so a user can tab AI Translation into Lookup's group at `full`. On full → short → full, `undoMerge` fell back to "right of the first panel" and split the user's own group. New case O was red before the fix: `(grid\|ai_translation\|lookup)`. The un-merged-persist half (V2/E2) is **false** for this state: the group is the user's, so persisting it merged is correct. Ice's "lưu dạng chưa gộp" covers the tier's merge only. | patch: the branch no longer sets `tierMerged` |
| B2 · V3 | The "drawer open while tier jumps" fix is claimed closed but no case runs it | medium | Pre-verified gap. New case N covers it. Deleting the `syncLayoutTier(tier)` call in `applyTier` turns N red with `owner panel.lookup đã khai rồi` (`PanelFrame.vue:142`), so the race is real and N guards the release. ⚠️ Moving the call AFTER the branches leaves N green, because under happy-dom the grid copy declares a tick later. The "must run before `restoreAllAutoHidden`" ordering is therefore not proven. | patch: case N |
| V1 | `reconcileAutoHidden` has no case | medium | Pre-verified. Measured: deleting the call left all 16 cases green. New case P: deleting it now gives `dockview: panel with id panel.lookup already exists`. | patch: case P |
| E3 | Grow-back restores position but not sash ratio | medium | `rememberSpot` stores `{reference, direction}` only, and `WorkspaceDock.vue` never calls `setSize`. Case D's own comment says `addPanel` may re-split. The stored layout stays byte-identical (hand check, Phase 5), but the next user action persists the re-split ratio. | Ice ruled: owned debt, no revert (not a loopback) |
| B4 | `sprint-status.yaml` `last_updated` is stale | low | It reads `09-22-2026 21:25`, while the diff carries 2026-09-23 work. | patch at the step-05 status sync |
| B3 | Both presets share the same numbers, so a key swap is undetectable | false | With identical values a swap changes no output. Kiểm E pins `EXPECTED` per key independently, so the first divergent calibration would catch a swap. | reject |
| B5 | sprint-status says `in-progress`, not `review` | false | That was correct when the diff was taken. The workflow moves it at the step-05 sync. | reject |
| B6 | `ALLOWED_GLOBAL_MEMBERS` is repo-wide, not per call site | low | This is the pre-existing shape of Kiểm C, the same as `document.execCommand`. A per-file list is new gate machinery. | reject (low, fix adds complexity) |
| B7 | `command.layout.lookup_drawer_open` = "Tra cứu", identical to the panel title | low | The same key labels the 34 px status-bar button, which fits one clause. Disambiguating needs a second key. | reject (low, fix adds complexity) |
| B8 | No e2e for the new surface | false | The wdio harness cannot resize the window (spec §Verification), and the default window opens in `short`, where no drawer entry exists. | reject |
| B9 | `SIZE_UNSUPPORTED` width 700 is below `minWidth` 800 | false | The ladder is a pure function of a number, and 700 is a valid input. No outcome differs. The 800–860 band was confirmed by hand in Phase 5. | reject |
| B10 | Other text may still assume 70 px (38 + 32) | false | A grep of the diff finds 38/32 only inside the debt item that describes the stale numbers. `computeWorkArea` reads both tokens at runtime. | reject |
| B11 | Sibling-insert branch of `unmergeForPersist` does not assert `size` | low | The doc-comment calls the halved size an approximation, and dockview re-divides on `fromJSON`. No wrong outcome is shown. | reject (low) |

**Patches applied:**
- `src/layout/WorkspaceDock.vue::applyMerge`: the already-tabbed branch returns without setting `tierMerged`.
- `tests/frontend/workspaceDockTier.test.ts`: `USER_TABBED_LAYOUT` plus cases N, P and O.
- File result: 19/19 green.

**Full bar after the patches, still tree:**
- all eleven gates exit 0
- `EXIT_BUILD=0`
- `EXIT_VITEST=0`: **94 files / 1417 passed**, which is 1414 plus N, P and O
- `EXIT_CARGO=0`: **66 binaries, 1806 / 0 / 22**
- `EXIT_STORY=0`

⚠️ Nightly e2e: the latest `schedule` run (CI #35782683524, 2026-09-22, green) ran on `5a23410`, which is `origin/master`. Local `HEAD` is 5 commits ahead, and Story 4.12 is not committed yet. So that green run says nothing about this story, and a nightly run on pushed code has to be read before Ice signs.

## Design Notes

**Why the thresholds are keyed by preset and not global.** Ⓑ-2 gives the grid the left column at
full height with Lookup and AI stacked to its right; Ⓑ-1 gives the grid the full width across the
top with the other two in a row beneath. Vertical pressure therefore bites Ⓑ-1 first and horizontal
pressure bites Ⓑ-2 first. The 2026-08-14 amendment to UX-DR15 says this in one line — *"nay phải
hiệu chỉnh cho HAI bố cục, không phải một"* — and Story 2.5b's measurement is the evidence: Ⓑ-2's
source column is 238,5 px, a number Ⓑ-1 never produces.

**Why the window reads rather than a `ResizeObserver`.** Both work. `check-layout.mjs` Kiểm C scans
`window.*` and `document.*` members against an allow-list, so `window.innerWidth`/`innerHeight`
force this story's new surface to be written into that gate with a reason, while a bare
`ResizeObserver` constructor is invisible to it. Story 1.14 spent a whole AC establishing that no
window read exists in `src/**`; the honest way to end that ban is where the gate can see it.

**Why the automatic tier must not persist.** The dock writes `api.toJSON()` on every
`onDidLayoutChange` through a 500 ms debounce. If an auto-hide reaches that path, docking the
laptop to a small external display rewrites the user's saved arrangement, and undocking cannot
restore it — the information is gone. This is the one irreversible in the story, and it is silent.

## Verification

**Commands:**

- `npm run build` **before** any `cargo test` — without `dist/`, `cargo test` fails at compile time.
- `npx vitest run` — 0 red. `npm run test:story 4-12` for the scoped loop; it reports THIẾU KHAI for
  any test file the diff touched that this spec never named.
- The eleven `pre-push` gates individually: `check:deps` · `tokens` · `i18n` · `commands` ·
  `layout` · `panel-refs` · `dict` · `dict-manifest` · `lint` · `gates` · `debt-owner`. Read **why**
  each result is what it is, not its colour.
- `cd src-tauri && cargo test --locked` — the `minWidth` line puts `config_invariants.rs` in the
  blast radius even though no case reads window size today; measure the count, do not assume it.
- ⚠️ `pre-push` runs on Ice's macOS only. Read the CI run before concluding green, and read the
  latest nightly e2e `schedule` run before writing `done`.

**Manual checks:**

- ⚠️ No automated path can produce the calibration. `happy-dom` computes no geometry, the wdio
  harness sets `driverProvider: 'embedded'` with no window-resize capability and runs nightly only,
  and no Rust code listens for `WindowEvent::Resized`. The measurement is a human resizing a real
  build with a real Chapter open in all three panels — record the numbers, not a verdict.
- For each preset in turn: shrink the window one edge at a time, note the work area at which the
  grid stops being usable for translating, and compare it against the seeded number. That
  difference is what Q9 asks for.
- Resize across every threshold and back, then reopen the app: the layout must be the one the user
  arranged, not the one a tier left behind.
