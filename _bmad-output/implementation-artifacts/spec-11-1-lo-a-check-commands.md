---
title: 'Story 11.1, lot A — pay down the nine check:commands debt items'
type: 'chore'
created: '2026-09-24'
status: 'done'
route: 'dispatch'
baseline_commit: '2abddd34c6440458d1deb1c9cd5fca58c1623ff0'
review_loop_iteration: 0
context:
  - '{project-root}/scripts/AGENTS.md'
  - '{project-root}/_bmad-output/implementation-artifacts/epic-11-context.md'
---

<frozen-after-approval reason="human-owned intent — do not modify unless human renegotiates">

## Intent

**Problem:** Nine `deferred-work.md` items whose last `Chủ:` is Story 11.1 concern `scripts/check-commands.mjs`. The gate is green on HEAD, but it only guards `@click`, while 15 `@keydown` handlers now dispatch commands and one grid `@mouseup` mutates state directly (`setEditorSourceCut`). The script also has no type-check and no automated test, and a copy of `clearSourceCuts` in a test can drift from `src/main.ts` without anything turning red.

**Approach:** Lot A of four for Story 11.1 (lots B–D are separate specs; the story goes `done` only after lot D). Close each of the nine items with exactly one Epic 11 disposition, based on a re-read against HEAD. Most items get a fix; the rest get `KHÔNG LÀM` or a reassignment.

## Boundaries & Constraints

**Always:** Every new or changed guard is counter-checked by really removing the seam (AGENTS.md §Tests). Literal ids stay readable: Kiểm A, B, E and F read literals from `p.masked`, so a string-blanked view is additive and never replaces `masked`. Each ledger item ends with `→ ✅ ĐÃ ĐÓNG` plus evidence, `→ KHÔNG LÀM 2026-09-24 (Story 11.1) — <reason>`, or `→ … Chủ: <concrete>`. Exemptions are named, carry a reason, and can die. The new Kiểm carries an in-script self-check that it can go red and does not go red wrongly (`scripts/AGENTS.md`).

**Never:** Raise the six stale floors (not a lot A item). Touch the 51 − 9 items of lots B–D. Add a dependency (NFR15). Hand-write a `.d.ts` that duplicates a script. Convert the 22 `@submit` direct calls into commands.

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected |
|---|---|---|
| New action handler | a `@keydown`/`@mouseup` handler added outside the new inventory | `check:commands` red, naming file and handler |
| Dispatch-set drift | `onEditKeydown` stops dispatching `editor.merge_segments` | red |
| Fake call in a string | `'useSelectionSurface(x, \'source\')'` inside a string literal | not counted as a registration |
| Copy drift | the `clearSourceCuts` guard is removed from its single definition | a vitest case goes red (today: 476 cases, 0 red) |

## Decisions (Ice, 2026-09-24)

1. **Handlers other than `@click`: exact two-way inventory.** A new Kiểm covers `@keydown`/`@keyup`/`@mouseup`/`@mousedown`/`@submit` (`@input`/`@change` stay out: data flow, AD-34 §1). A literal `dispatch('<id>')` passes as it does in Kiểm A. Every other handler must appear as `file::handler` in a frozen table (about 69 sites on HEAD), mapped to the exact set of ids its body dispatches or to `nonCommand: <reason>`. The 22 `@submit` direct calls and `setEditorSourceCut` are named exemptions. Unlisted handlers, stale entries and dispatch-set drift are all red. Items *Kiểm A chỉ canh `@click`*, *`@keydown` nay mang một thao tác thật* and *Cử chỉ chuột của lưới* close ✅.
2. **Conditional items: build the three cheap guards now.** A guard is not speculative feature code. The three guards: `SELECTION_PANEL_FILES` checked two-way against the panel imports of `src/layout/WorkspaceDock.vue`; ESLint `no-restricted-syntax` on `Literal[value='confirm_segment']` in `src/**` except `src/config/segment.ts`; Kiểm F red on `registerSelectionSurface(` outside `src/panels/selectionContract.ts`. All three close ✅.
3. **Spec size kept at ~2.2k tokens** (Ice accepted): lot A is already a slice of Story 11.1, and its parts share the extracted module.

</frozen-after-approval>

## Code Map

- `scripts/check-commands.mjs` -- not import-safe (top-level `await loadTs`, `process.exit`). Kiểm A at ~:774 (`CLICK_ATTR_RE` :779, `DISPATCH_ONLY_RE` :778, loop over `attributesIn(p.masked…)` :806). `maskScript` :422 with string branch ~:500; `maskTemplate` :560. Kiểm F: `SELECTION_PANEL_FILES` ~:2026, `SURFACE_CALL_RE` :2088, `SURFACE_ANY_CALL_RE` :2098, non-literal warning :2205. Stale "Kiểm A chỉ canh `@click`" notes at :33-36, :851, :2436. Kiểm J is retired: use the next free letter for a new Kiểm.
- `scripts/check-debt-owner.mjs:104` -- only precedent of a pure exported helper plus an in-script self-test.
- `scripts/check-layout.mjs:232` -- precedent for `loadTs`-importing `src/layout/workspaceLayout.ts`.
- `src/layout/WorkspaceDock.vue:49-51,104-108` -- the only component → file map for workspace panels.
- `src/panels/GridPanel.vue` -- `onEditKeydown` :1450 (Escape → `editor.clear_source_cuts`, Backspace → `editor.merge_segments`); `onSourceCellMouseUp` :769 calls `setEditorSourceCut` directly at :808.
- `src/main.ts:803-806` -- the `clearSourceCuts` dep closes over module singletons only (`quickAddIsOpen`, `confirmStripIsOpen`, `clearEditorSourceCut`); `tests/frontend/editorClearSourceCuts.test.ts:81-84` holds the copy.
- `eslint.config.js:142-155` -- `no-restricted-syntax` precedent (`.click()` in `e2e/**`); `:62` ignores `scripts/**`.
- `tsconfig.node.json` -- includes only `vite.config.ts`. `npm run build` runs `vue-tsc -p` on it. Strict `checkJs` on `check-commands.mjs` measures 92 errors (75 are TS7006).
- `e2e/specs/segment-backspace-merge.e2e.mjs:19` -- cites stale lines `2348-2349`.

## Tasks & Acceptance

**Execution:**
- [x] `_bmad-output/implementation-artifacts/deferred-work.md` -- Task 0: re-read the nine items against HEAD; record any that self-closed.
- [x] `scripts/lib/commands-scan.mjs` (new) + `scripts/check-commands.mjs` -- extract `maskScript`, `maskTemplate`, `attributesIn`, a string-blanked `code` view and the handler scan into a pure module; the gate imports it; behaviour and output stay identical except for the new Kiểm.
- [x] `tsconfig.node.json` -- type-check the extracted module and `check-commands.mjs` under strict mode, fixing the 92 errors through JSDoc and types, not `any`.
- [x] `tests/frontend/checkCommandsScan.test.ts` (new) -- unit-test the module, covering every I/O row.
- [x] `scripts/check-commands.mjs` -- new Kiểm per Decision 1; call-head matches confirmed in the `code` view; update the three stale notes.
- [x] `src/editorClearSourceCuts.ts` (new, same layer as `main.ts`, which alone may import both Glossary strip states) + `src/main.ts` + `tests/frontend/editorClearSourceCuts.test.ts` -- a single exported `clearSourceCuts` dep; `main.ts` and the test import it.
- [x] `scripts/check-commands.mjs` (Kiểm F) + `eslint.config.js` -- the three guards of Decision 2.
- [x] `e2e/specs/segment-backspace-merge.e2e.mjs` -- cite the note by content, not by line.
- [x] `deferred-work.md` -- close all nine items. Add one new item (≤5 lines, `Chủ: Story 11.7`) for the `main.ts` dep copies in `aiTranslate.test.ts` and `aiTranslateBatch.test.ts`.

**Acceptance Criteria:**
- Given lot A is done, when `grep` runs over the nine items, then each one's last line is one of the three Epic 11 dispositions, and `npm run check:debt-owner` is green.
- Given each new guard, when its seam is really removed, then that guard's target goes red for the measured reason, and green again once the seam is restored.
- Given `npm run build`, when `check-commands.mjs` or the module gains a type error, then the build fails.

## Verification

**Commands:**
- `npm run check:commands && npm run check:lint && npm run check:debt-owner` -- expected: green.
- `npx vitest run tests/frontend/checkCommandsScan.test.ts tests/frontend/editorClearSourceCuts.test.ts` -- expected: green.
- Full suite once, by hand: `tsconfig.node.json` is shared wiring (AGENTS.md §Tests).

## Implementation Notes

- `scripts/lib/commands-scan.mjs` (new) holds `maskScript`/`maskTemplate`/`attributesIn`/`scanVueAttrs`/`functionBodyRange`, pure and vitest-covered; `check-commands.mjs` imports it. `maskScript`'s new `opts.blankLiterals` produces a second, additive `code` view (strings/template literals blanked, `${...}` kept) alongside the untouched `masked` view.
- Strict `checkJs` on `tsconfig.node.json` over both files found a real bug pre-dating this story: a stray `oBad += 1` sat before its own `let oBad = 0` (Kiểm E's FOCUS_OWNERS block) — a temporal-dead-zone `ReferenceError` waiting for the first `t()`/`title-key` call site missing from `vi.json`, never hit in practice. Deleted; `eFail(...)` already counted `eBad` on the line above.
- Kiểm F now finds a `useSelectionSurface`/`registerSelectionSurface` call-head in `p.code` (a string can't fake a call) but still reads the role literal from `p.masked` at that same offset — reading the role off `code` alone breaks the gate outright, since `code` blanks the `'source'`/`'display'` string too. Same masked-slice-at-offset shape as Kiểm B's `nonLiteralDispatchCalls`; reuse it, don't re-derive.
- Kiểm K (Decision 1) keys its frozen `HANDLER_TABLE` by `file::handler` (the value's leading identifier, not the call-site arguments) and resolves each entry's dispatched ids from the **handler's own function body** via `functionBodyRange` on the `code` view. Real population measured by running the scan, not estimated: 91 matched attributes, 63 distinct handlers (not the spec's "about 69"), 19 literal `dispatch(...)`, 12 handlers with real `ids`, 51 `nonCommand`. Floor set to 75 (~82% of 91).
- `LookupPanel.vue::moveTabFocus` is `nonCommand: R_DISPATCH_VIA_PARAM` by design: its body is `dispatch(commandId)`, a parameter, so Kiểm K correctly finds zero literal ids even though its two call sites pass real ids — those call-site ids are informational only and are never read by the checker (same honesty `nonLiteralDispatchCalls` already keeps for Kiểm B).
- Six handlers imported from a state module (`closeAttribution`, `closeSegmentHistory`, `closeLookupDrawer`, `aimRowFrom`, `aimDictSourceFrom`, `aimLookupEntryFrom`) can't have their body opened cross-file; `judgeHandlerInventory` now machine-verifies the import itself (`isImportedIdentifier` against `p.masked`, since `p.code` blanks the import path's quotes) before accepting `nonCommand` on a null body — only the dispatch *behaviour* of an imported function stays hand-verified belief.
- Decision 2's three guards (`SELECTION_PANEL_FILES` two-way vs. `WorkspaceDock.vue`'s panel map — parsed by brace-depth counting and a fail-closed `key: Ident` check, not a `[^}]*` regex that a nested `{}` or a shorthand entry could defeat; `registerSelectionSurface(` banned outside `selectionContract.ts`; ESLint `no-restricted-syntax` on the `confirm_segment` literal AND its bare-template-literal form) were each counter-checked by real seam removal, both directions where the guard has two.
- Review pass found four more real gaps, each patched and counter-checked: the dangling-`HANDLER_TABLE`-entry check had no self-check reaching it (extracted into `danglingHandlerKeys`, shared by the real run and a new self-check case); a handler value like `foo(); dispatch('x')` was silently keyed on `foo` (grammar now requires a bare identifier or one whole-value call, else FAIL); `errorMessage` returned a blank string for an `Error` with an empty message (falls back to `String(err)`).
- `src/editorClearSourceCuts.ts` (new) now holds the single `clearSourceCuts` definition; `main.ts` and `tests/frontend/editorClearSourceCuts.test.ts` import the same function instead of each keeping a copy. Counter-checked for real: removing the strip-open guard from that one definition turned four vitest cases red (⑥⑦⑨⑩), restored green after.
- Ledger item "`check-commands.mjs` không type-check, không test tự động" closes 🟡, not ✅: the type-check half is fully closed, but Kiểm K's own judgment (`judgeHandlerInventory`) and the rest of the script's Kiểms are only exercised by the in-script self-check plus hand seam-removal, never a vitest file — `check-commands.mjs` isn't import-safe (`process.exit`, top-level `await`).
- New debt item opened, `Chủ: Story 11.7`: `tests/frontend/aiTranslate.test.ts` and `aiTranslateBatch.test.ts` still carry verbatim copies of `main.ts::boot()`'s dep wiring, same drift class this story just closed for `clearSourceCuts`.

## Review Triage Log

- blind: other events (`@dblclick`/`@focusin`/`@scroll`/`@copy`…) outside Kiểm A and K — low, rejected: the hole predates this story, the 10 such handlers in `src` today are focus, aim or scroll state with no dispatch, and widening needs a new inventory.
- blind: `@[dyn]`/`v-on="{…}"` invisible to Kiểm K — false: `OPAQUE_CLICK_RES` is tested against every attribute in the Kiểm A loop (`check-commands.mjs:649`), not only click, so both forms already fail.
- blind+edge: a `nonCommand` entry whose body `functionBodyRange` cannot open (arrow function, any cause) passes like a verified one — medium, patch.
- blind: `SURFACE_CALL_RE` closure line claims ✅ but the comma-argument and case forms are untouched — low, patch: the line must state that those forms reach only the yellow non-literal warning.
- blind: 🟡 item owned by `Story 11.1` goes stale at `done` — false: `check:debt-owner` Kiểm C turns red at that transition and forces a reassignment.
- blind: `Chủ: Story 11.7` may not exist — false: `11-7-trả-nợ-nền-giao-diện-dùng-chung-và-ai` is in `sprint-status.yaml`.
- blind: `allowJs` in `tsconfig.json` loosens `src` — low, rejected: `include` lists only `*.ts`/`*.vue`, so only explicitly imported JS enters, and `src` has no `.js`.
- blind: `HANDLER_ATTR_FLOOR = 75` unexplained — false: 75/91 = 82%, inside the 80–85% rule of `scripts/AGENTS.md`.
- blind+edge: comment says 22 `@submit` exemptions, table holds 21 `R_SUBMIT_DIRECT` entries — low, patch: code comments carry no counts (AGENTS.md §Code comments); delete it.
- blind+verification-gap: the dangling "mục TREO" loop has no self-check; setting its condition to `false` leaves the gate green — medium, patch.
- blind+edge: `COMPONENTS_OBJ_RE` truncates at a nested `}` and `ENTRY_RE` skips a shorthand `{ NewPanel }`, silently dropping panels from the two-way check — medium, patch (`scripts/AGENTS.md`: unknown syntax ⇒ FAIL).
- blind+edge: ESLint ban misses `` `confirm_segment` `` as a template literal without expressions — low, patch.
- edge: `errorMessage` prints a blank line for an `Error` with an empty message — low, patch (direct correction).
- edge: a handler value like `foo(); dispatch('x')` is keyed by `foo` and the second call is never checked — medium, patch: the value grammar must be a bare identifier or a single call, else FAIL.
- verification-gap (other): the removed `oBad += 1` is undocumented — false: Implementation Notes bullet 2 records it.
- orchestrator: comments added or changed in this diff break AGENTS.md §Code comments (Vietnamese, story ids, "Quyết định N", dates, counts, history) in `check-commands.mjs`, `commands-scan.mjs`, `eslint.config.js`, `editorClearSourceCuts.ts`, `main.ts`, `LookupPanel.vue`, `LibraryMode.vue`, `commands/index.ts`, both test files and the e2e spec — medium, patch.
