---
title: 'Story 11.1, lot C — pay down the webview-scanning gate debt items'
type: 'chore'
created: '2026-09-25'
status: 'done'
route: 'dispatch'
baseline_commit: 'cd72651484c56ae01e8f132d48486376adda97aa'
review_loop_iteration: 0
context:
  - '{project-root}/scripts/AGENTS.md'
  - '{project-root}/_bmad-output/implementation-artifacts/epic-11-context.md'
---

<frozen-after-approval reason="human-owned intent — do not modify unless human renegotiates">

## Intent

**Problem:** Sixteen `deferred-work.md` items whose last `Chủ:` is Story 11.1 concern the gates that scan the webview: `check-i18n.mjs`, `check-tokens.mjs` and `check-layout.mjs`. All sixteen still hold on HEAD `cd72651`. Real bugs have already got past every green gate this way: a `var(--space-inline-sm)` that no token defines rendered as `gap: 0` across three stories, and a dockview sash drew over an overlay. Beyond those, a panel-title key read through a non-literal `t()` call, the persistence wiring in `WorkspaceDock.vue` and the `PANEL_COMPONENTS` names have no guard, and 41 `epics.md:N` line citations across 24 code files drift silently.

**Approach:** Lot C of four (lot D takes the floor items, including L4626 "two floors dropped below the range"). Give each of the sixteen items exactly one Epic 11 disposition after a HEAD re-read. Build the cheap mechanical guards now, each one inside the gate that already owns that surface. Close with `KHÔNG LÀM` and a stated reopen condition the items whose condition has not happened.

## Boundaries & Constraints

**Always:**
- Every new Kiểm carries an in-script self-check that it can go red and does not go red wrongly (`scripts/AGENTS.md`), and is counter-checked by really removing its seam in production code.
- A source scan reads masked code: comments never trigger it, and a literal is never hidden from it.
- Each gate's verdict and scanned population on HEAD are measured before and after the change; a changed count is explained, not inferred.
- Ledger items keep their text; append `→` lines only.

**Never:**
- Change a `*_FLOOR` value or touch L286/L4626/L7168/L9417/L11100 (lot D).
- Add a dependency (NFR15), including a CSS or HTML parser.
- Make a visible UI change, except where the decision on L6724 below explicitly allows it.
- Write speculative code for a "reopen when X" item whose X has not happened (L51, L54, L635).

## I/O & Edge-Case Matrix

| Scenario | Input | Expected |
|---|---|---|
| Dead token | `gap: var(--space-inline-sm)` | `check:tokens` red, names file and var |
| Declared locally | `--row-h: 24px` declared in `src/**`, then `var(--row-h)` used | green |
| Dockview var | `var(--dv-…)` declared by `dockview.css` | green |
| Comment mentions `<script>` | `<!-- see <script setup> -->` in a `.vue` template | no fake region opens; real `<script>`/`<style>` still found |
| Typo'd panel title key | a `PANEL_TITLE_KEYS` value absent from `vi.json` | `check:i18n` red |
| Blank panel | a `PANEL_COMPONENTS` value missing from `WorkspaceDock.vue`'s `components` map | `check:layout` red |
| Schedule bypass | `onLayoutChange` persists without `schedule.onChange`/`onWrite` | `check:layout` red |
| Isolation dropped | `isolation: isolate` removed from the dockview host (`App.vue` ~:600) | `check:layout` red |
| Smooth scroll | `scroll-behavior: smooth` in any `src/**` style | `check:tokens` red; the comment mentions in `LookupPanel.vue` stay green |
| Off-grid spacing | `padding: 7px`, or `calc(var(--space-unit) * 2.75)` in `padding`/`margin`/`gap` | `check:tokens` red |
| Border compensation | `margin: -1px` with `aura-allow-spacing — <reason>` | green |

## Decisions (Ice, 2026-09-25)

1. **L117: mirror lot A in full.** Extract the pure functions into `scripts/lib/tokens-scan.mjs`, add `check-tokens.mjs` and the lib to `tsconfig.node.json`, and add `tests/frontend/checkTokensScan.test.ts`. The item closes 🟡: the Kiểm verdicts stay outside vitest because the script is not import-safe. The remaining gap keeps `Chủ: Story 11.1`, next to lot A's L187, for lot D to settle.
2. **L6724: a true 4px grid.** A new Kiểm in `check-tokens.mjs` requires every `padding`/`margin`/`gap` value (including the side variants) to be a spacing token or a whole multiple of `--space-unit`. Fractional N fails. All 53 off-grid values in the 10 files are fixed in this lot:
   - round to the nearest multiple of 4; a tie rounds up;
   - the "11px padding + 2px border" pattern becomes 12px in all six spots at once, including `.gs-alert` (this replaces the `2.75` that cụm F kept);
   - a ±1px/±2px value that only compensates a border width may keep its value, but only with a named `aura-allow-spacing — <reason>`.
   These are visible 1–3px shifts. Ice's real-use pass is a new debt item with `Chủ: Epic 11`.
3. **L1293: `KHÔNG LÀM`.** No deterministic scanner can compare a string's meaning with the behaviour it describes. The instance the item named is fixed (`vi.json:733,737`).
4. **The spec stays whole** (2,947 tokens, Ice accepted).
5. **L5458: a new gate `check:doc-refs` that only bans `epics.md:<digits>` in code files** (`src/`, `src-tauri/src`, `src-tauri/tests`, `scripts/`, `tests/`, `e2e/`). Each of the 41 citations (24 files; 35 in comments, 6 in strings) is rewritten as the FR/NFR/AD id it stands for, or deleted when it is only history. `§Story X.Y` is never used: AGENTS.md bans story ids in code. Register the gate in all three lists, guarded by `check:gates`. It touches `src-tauri`, so the full suite runs once.

6. **The 12 dead `var()` references Kiểm I found on HEAD are fixed in this lot** (a visible change, covered by Ice's real-use pass in Decision 2):
   - `var(--face-read)` → `var(--family-read)` in GlossaryManageOverlay :881, GlossaryQueueOverlay :428, PromptLibraryOverlay :784 and :915, and AiPromptInspectorOverlay :503;
   - the `--face/--font/--leading-read-body` trio → the `read-sm` trio in SegmentHistoryOverlay :409-411 and :465-467;
   - `.hist-aimed` background `var(--color-surface-variant)` → `var(--color-surface-accent)`, the same background as the grid's `.cell.row-selected`.
7. **Kiểm K found 12 declarations beyond the 53** (because it evaluates `calc()` and non-px units):
   - The seven fractional `calc(var(--space-unit) * 1.5 | 0.5)` values are rounded under Decision 2's rule (×1.5→×2, ×0.5→×1). They get no exemption.
   - Three kinds keep a named `aura-allow-spacing` exemption: `ReadingMode.vue`'s `1em` paragraph margins (they scale with the reading size), `dockview-theme.css`'s `calc(var(--panel-gap) / 2)` (a runtime value), and the sr-only `margin: -1px` (in 2 files).
</frozen-after-approval>

## Code Map

Full per-item findings (evidence, sizes, counter-check plans) for the implementer: `/private/tmp/claude-501/-Users-hoangnam-LocalSites-addon-AuraTranslate/24a8a135-0bf6-4768-9b05-fe8e40766895/scratchpad/lotc-{i18n,tokens,layout}.md`. If they are gone, rebuild them from the ledger items.
- `scripts/check-i18n.mjs` -- `vueRegions` :466-484 (bare regex, comment-blind); `scanTemplate` :729+ already tracks `<!-- -->` (reuse its technique); Kiểm A2 `ALLOWED_CALL_RE` :957; symlink skip :169-193 (L51); `scanStyle` :659-702 (L54).
- `src/layout/workspaceLayout.ts:62-66` `PANEL_TITLE_KEYS`, and `PANEL_COMPONENTS`; `src/panels/PanelTab.vue:80` is the non-literal `t()` call.
- `scripts/check-layout.mjs` -- `loadTs` import of `workspaceLayout.ts` in Kiểm A :221-312; Kiểm B :313-409 (only `writeSchedule.ts`); Kiểm C allow-list :421, storage deny list :530-538 (L635); next free letter F.
- `src/layout/WorkspaceDock.vue` -- `createWriteSchedule` :54/:761, `flush`/`onLayoutChange` :782-798, `onDidLayoutChange` :967, `components` map :49-51/:104-108. `scripts/lib/commands-scan.mjs` already parses that map (lot A guard, `check-commands.mjs:2133-2195`): reuse it, don't write a third parser.
- `src/App.vue:571-600` -- `isolation: isolate` on the dockview host; `node_modules/dockview-vue/dist/styles/dockview.css` max `z-index` 9999 (:2398).
- `scripts/check-tokens.mjs` -- `maskCommentsAndStrings` :459, `parseCssBlocks` :523, Kiểm B :849 (forward direction only); next free letter I. `src/tokens/index.ts` emits every token var (`:110` maps `DEFAULT`→`default`; `:123-130` computes the `--panel-*` vars).
- `src/main.ts:698,726,742` -- three comments wrongly claim that Kiểm A of `check:i18n` scans this `.ts` file; `:755-759` is the correct wording to copy.
- `tsconfig.node.json` -- lot A added `allowJs`/`checkJs`; `include` lists only `vite.config.ts` and the two commands files.

## Tasks & Acceptance

**Execution:**
- [x] `deferred-work.md` -- Task 0: re-read the 16 items (L51, 54, 117, 631, 635, 639, 664, 1233, 1293, 2187, 3374, 4235, 4323, 5458, 6724, 7070) against HEAD.
- [x] `scripts/check-i18n.mjs` -- make `vueRegions` comment-aware and add a self-check with a negative and a positive fixture (L7070). Resolve every `PANEL_TITLE_KEYS` value against `vi.json` through `loadTs` (L664).
- [x] `src/main.ts` -- reword the three comments (L3374), and drop the stale `:248 · :264 · :281` history sentence at :758-759.
- [x] `scripts/check-layout.mjs` -- Kiểm F: `WorkspaceDock.vue` persists only through the schedule (L631); every `PANEL_COMPONENTS` value is a key of the `components` map (L639); the element hosting dockview keeps `isolation: isolate` (L1233). Each with a self-check.
- [x] `scripts/check-tokens.mjs` -- Kiểm I: every `var(--x)` in `src/**` resolves to a var emitted by `tokens/index.ts`, declared in `src/**`, or declared by `dockview.css` (L4323). Kiểm J: no `scroll-behavior` value other than `auto` (L4235). Each with a self-check.
- [x] `scripts/lib/tokens-scan.mjs` (new), `tsconfig.node.json`, `tests/frontend/checkTokensScan.test.ts` (new) -- Decision 1 (L117).
- [x] `scripts/check-tokens.mjs` Kiểm K and the 10 `.vue` files -- Decision 2 (L6724); new debt item for Ice's real-use pass, `Chủ: Epic 11`.
- [x] Five overlay `.vue` files -- Decision 6: the 12 dead `var()` references, so that Kiểm I goes green.
- [x] `scripts/check-doc-refs.mjs` (new) plus `package.json`, `.github/workflows/ci.yml`, `.githooks/pre-push`, and the 24 citing files -- Decision 5 (L5458), with a self-check.
- [x] `deferred-work.md` -- dispositions: `KHÔNG LÀM` with the reopen condition for L51 (a `.vue`/`.rs` symlink appears), L54 (a `lang="scss"` block appears), L635 (a parser dependency is accepted) and L2187 (a standing directive, already followed); L1293 `KHÔNG LÀM` (Decision 3); `✅` with evidence for every item fixed.

**Acceptance Criteria:**
- Given lot C is done, when the 16 items are grepped, then each ends in an Epic 11 disposition, and `npm run check:debt-owner` is green.
- Given each new guard, when its seam in production code is really removed, then that gate goes red for the measured reason, and green again once the seam is restored.
- Given HEAD, when `check:i18n`, `check:layout` and `check:tokens` run, then each is green and its scanned-file counts match the before-measurement, or the difference is explained.

## Implementation Notes

- Pure CSS/scan helpers were extracted to `scripts/lib/tokens-scan.mjs`, like lot A did with `commands-scan.mjs`. `balancedBraceBody`/`splitTopLevel` moved into `commands-scan.mjs` so `check-layout` Kiểm F.2 reuses lot A's parser. Type-checking `check-tokens.mjs` surfaced ~70 strict-`checkJs` errors, fixed with JSDoc only.
- Kiểm I computes the emitted token vars from `tokens.json` instead of `import()`-ing `tokens/index.ts`, which fails under plain Node (`ERR_IMPORT_ATTRIBUTE_MISSING`). A self-check source-scans `index.ts` for drift in the naming convention.
- Kiểm I found 12 dead `var()` references on HEAD, and Kiểm K found 12 off-grid values beyond the 53 because it evaluates `calc()` and non-px units. Both went to Ice mid-lot, giving Decisions 6 and 7.
- The Kiểm K exemption self-check exposed a shared bug: `exemptAt` accepted `aura-allow-<kind>: */` with no reason, because `\S` matched the `*` of `*/`. The same bug sat in the separate `neverTextExemptAt` (found in review), so both are fixed, each with self-check cases. The same pattern in `check-i18n.mjs` (`aura-allow-text`) is deferred.
- The `epics.md:N` count on HEAD was 42 citations in 25 files. Where no FR/NFR/AD fits, a citation became its stable `UX-DR` id (for example UX-DR15, the panel sacrifice order).
- Phase 4 also fixed a literal NUL byte (committed long before this lot) in `src/ChapterImage.vue`'s `watch` key. It became `\u0000`, not a space, so the key still separates the `(assetsDir, fileName)` pair unambiguously.
- A ledger closure line had overwritten the heading `## Deferred from: lượt lập spec Story 3.4`; the orchestrator restored it before review.

## Spec Change Log

## Review Triage Log

- blind: `neverTextExemptAt` (`check-tokens.mjs:1253`) still exempted `aura-allow-never-text: <token> */` with no reason, so the Implementation Notes claim was false — medium, patch: added the `(?!\*\/)` lookahead plus 3 self-check cases; counter-checked by removing the lookahead (red on the no-reason case).
- blind+edge: `check-doc-refs.mjs` skipped symlinks silently, unlike the `skippedLinks` report of `check-i18n`/`check-tokens` — low, patch: collect them and print a `detail` line (verified with a temporary link).
- blind: the L2187 ledger line said `check:doc-refs` was built inside an existing gate — low, patch: reworded; it is a new static gate.
- blind: `inlineStyleBlocks` keeps a trailing `'` for a bound string `:style`, now pinned by a test and with no ledger line — pre-existing, defer.
- triage: `check-i18n.mjs:1095` accepts `<!-- aura-allow-text: -->` with no reason (`\S` matches `-` of `-->`) — pre-existing, defer.
- blind: `src/ChapterImage.vue` diff is opaque — false: the base blob holds a NUL byte, so git labels the file binary; `git diff --text` shows the single change to `\u0000`.
- blind: negative multiples of 4 (e.g. `margin: -8px`) pass Kiểm K with no exemption — false: Decision 2's rule is whole multiples of `--space-unit`; sign is not part of it.
- blind: Kiểm I re-implements `tokens/index.ts` naming instead of importing it — low, rejected: `import()` fails under plain Node (documented); a drift in the special-casing is rare, and the fix needs a TS loader.
- blind: the visible-change surface is larger than a "gate debt" framing suggests — false: not a defect; every change is under Decisions 2, 6 and 7 and the owned real-use debt item.
- blind: the new `ci.yml` step has no "cổng thứ N" ordinal — false: no named harm; `check:gates` checks the three lists mechanically.
- blind: `kinds.rs` cites "đoạn diễn giải FR103 trong `epics.md`" — false: it names FR103, a stable id, with no line anchor.
- blind: Decision 5 and the task list say 41 citations / 24 files while the notes say 42 / 25 — rejected: the fix is a spec edit.
- edge: Tasks say "the 10 `.vue` files" while spacing edits touch 12 — rejected: the fix is a spec edit, and Decision 7 accounts for the extra files.
- edge: `vueRegions` swallows the rest of a file after an unclosed `<!--` — false: an unclosed comment fails the Vue SFC compile in `npm run build`, so the tree cannot reach the gate that way.
- edge: `check-doc-refs` also scans `.md` under `scripts/`/`tests/`/`e2e/` — low, rejected: stricter than its header, green today, and `src/panels/README.md` is covered on purpose.
- edge: Kiểm F.3 reads only the first `.modeport { }` block — low, rejected: the tree has one `.modeport` rule; handling more adds branches for a shape that does not exist.
- edge: `calc(N * var(--space-unit))` is flagged off-grid — low, rejected: it fails loudly with a clear message, and the tree uses one ordering.
- verification-gap: no gaps found. Other: `compare()` gained null-handling — false: no path regresses. Other: `check-layout.mjs` is not type-checked — low, rejected: not a change here; Decision 1 covers `check-tokens` only.

## Design Notes

- The guards go into existing gates, so only `check:doc-refs` (L5458) touches the three gate lists. Each gate already owns its surface: i18n owns the key catalogue, layout owns the dock wiring, tokens owns CSS values.
- L4323 resolves against a union of real declarations rather than a frozen exclusion list. That way the computed `--panel-*` and `--dv-*` vars need no exemption, and a typo such as `var(--panel-gapp)` is still caught.
- L1233 guards the mechanism that fixed the bug, `isolation: isolate` on the dockview host. It does not compare z-index numbers across files.

## Verification

**Commands:**
- `npm run check:i18n && npm run check:layout && npm run check:tokens && npm run check:debt-owner` -- expected: all green.
- For each new guard: remove the seam, run that one gate, and see it go red for the named reason; restore the seam, and it goes green.
