---
ticket: 16
title: 'Epic 7 retro R-9 — no bare `origin` identifier, and a guard that keeps it so'
type: 'refactor'
created: '2026-10-05'
status: 'done'
baseline_revision: 'a000cbc68fc2da1b73b2a2bd6b7ed9277ecc33c0'
route: 'dispatch'
review_loop_iteration: 0
context: []
---

<frozen-after-approval reason="human-owned intent — do not modify unless human renegotiates">

## Intent

**Problem:** AGENTS.md §Conventions bans a bare `origin` identifier because "origin" names four disjoint things (spine §Consistency Conventions). Epic 7 added about 20 TM sites (`tm_list_pairs(origin, …)`, `OriginFilter`, `parse_filters(tier, origin)`, `config/tm.ts:217`), and no gate caught them because `naming_boundary.rs` does not check the word (retro F5, R-9). Older sites exist outside TM too: about 136 production lines in about 18 files, plus test lines.

**Approach:** Rename every bare `origin`/`origins`/`Origin` identifier in `src-tauri/src`, `src-tauri/tests` and `src/` to a name that carries its subject. Add a case to `naming_boundary.rs` that fails on such a token in code lines across the tree it already scans. No behaviour change, no schema change, no stored-value change.

**Decisions (Ice, 2026-10-05):**
- Q1 = C: rename everywhere, not only TM. This includes the two `origin.rs` module files and the chapter-origin wire key. The guard's only exemption is the dockview event field `e.origin` (`WorkspaceDock.vue:983`, third-party API).
- Q2 = X: the guard matches the whole token only (`origin`, `origins`, `Origin`). R-9's named TM compounds (`OriginFilter`, `TmManageOriginFilter`, `originFilterState`, `TmStoreError::UnknownOrigin`) are renamed by hand. Other compounds (`origin_overrides`, `extract_origin`, `baseline_origin`, `ORIGIN_OTHER`, …) stay.

## Boundaries & Constraints

**Always:** Subject names: TM pair / segment translation origin → `pair_origin` (the type is `PairOrigin`); source document (AD-43) → `chapter_origin` (the type is `ChapterOrigin`). Rust param and TS `invoke` key change together (`ipc_argument_contract`). The guard ignores comments, string literals, i18n keys and kebab-case CSS classes (`chapter-origin`, `'chapter.origin.heading'`, `tm.manage.origin_*`). The counter-check reverts one renamed site to a bare `origin` (a real edit) and the new case goes red for that reason.

**Never:** No change to stored values (`self`/`other`/`bilingual_import`), DB columns, `translation_origin`/`origin_*` keys, i18n keys, CSS classes, log tags or user-visible text. No `eslint-disable`, no new marker, no mass comment cleanup.

</frozen-after-approval>

## Code Map

- TM: `core/tm/mod.rs` (`UnknownOrigin` :112/:120; bindings :189-191, :447-449; `OriginFilter` :485-505; `rank_manage_listing` :570; `origins` :755-763; :795, :813), `core/tm/tmx.rs` (`unit.origin` :195, :293, :444), `commands/tm.rs` (:18; `parse_filters` :126-129 passes `"origin"` as the `filter` param of `tm.invalid_filter` :109; `TmListScan.origin` :162; :172-241; `wire::tm_list_pairs(app, origin, …)` :648-661).
- TM webview: `src/config/tm.ts` (`TmManageOriginFilter` :6, `isOrigin` :135-186, `tmListPairs` :216-225 with invoke key `origin`), `src/tmManageState.ts` (`originFilterState`, `tmManageOriginFilter`, `setTmManageOriginFilter`), `src/TmManageOverlay.vue` (`originLabel` :173, `onOriginChange`, `ORIGIN_OPTIONS`).
- Segment translation origin: `commands/segment.rs` (about 20 lines: :204, :212, :1250-1268, :2370, :2431, :2700-2757, :3007-3017, :3611, :3620); module `core/segment/origin.rs` → `translation_origin.rs` (`Arbitrated::Origin` → `Arbitrated::PairOrigin`), `core/segment/mod.rs:88`.
- Source document: module `core/webimport/origin.rs` → `chapter_origin.rs` (34 lines, mostly in-module tests), `core/webimport/mod.rs` :59-69, `core/segment/pipeline.rs` (`Flow.origins` and `origin`, 23 lines), `core/segment/import.rs:680`, `commands/project/work_creation.rs` :463/:855/:1795, `commands/project/mod.rs` :1104 (wire field `origin: ChapterOriginWire` → `chapter_origin`; there is no `rename_all`, so keys are snake_case), :1240-1241; `src/config/project.ts` :250/:459, `src/importPreviewState.ts:643`, `src/ChapterOrigin.vue:44` (emit tuple label).
- References to the old module paths: `commands/chapter.rs`, `core/segment/pipeline.rs`, `commands/segment.rs`, `core/segment/import.rs`, `src/importPreviewState.ts`, `tests/webimport_contract.rs`. Grep `segment/origin.rs`, `webimport/origin.rs`, `::origin::` before and after.
- Tests with bare tokens: `tm_contract.rs` (about 20 lines; positional `tm_list_pairs` callers :2112/:2254/:2257/:2778), `ai_prompt_contract.rs:190`, `tmx_contract.rs:277`, `segment_contract.rs` (:6088-6094, :6375-6380, :9887-9909, :9953 the `"origin"` key, :10655-10661), `chapter_origin_contract.rs:291`, `tmConfigGuards.test.ts:61-66` and :112, `tmManage.test.ts:40`, `importPreviewChapterOrigin.test.ts:59`, `importPreviewChapters.test.ts` (about 31 `origin:` keys).
- Guard: `tests/naming_boundary.rs`. Reuse `rust_sources` :196, `frontend_sources` :296, `strip_ts_comments` :238, the Rust `//` blanking :211 and the floors :123/:126. It does not strip strings, but `support/boundary_scan.rs::code_lines` :291 does. The exemption pattern to copy is `STORE_EXEMPT` :111 with `every_store_exemption_still_matches_something_real_in_the_repo` :925. The guard does not scan `src-tauri/tests`, `e2e/` or `scripts/`, and this story does not widen it (`e2e/support/pointer.mjs:52` is WebDriver's `origin`).
- Do not touch: `PairOrigin`, `TmPairOrigin`, `ChapterOrigin*` types, `glossary_boundary.rs` `NON_MANUAL_ORIGIN_TOKENS`, `commands/project/wire.rs` log tag `webimport[origin]`.

## Tasks & Acceptance

**Execution:**
- [x] TM Rust (`core/tm/mod.rs`, `core/tm/tmx.rs`, `commands/tm.rs`) -- `origin(s)` → `pair_origin(s)`; `OriginFilter` → `PairOriginFilter`; `UnknownOrigin` → `UnknownPairOrigin`; `invalid_filter("pair_origin", …)`; wire param `pair_origin`.
- [x] TM webview (`config/tm.ts`, `tmManageState.ts`, `TmManageOverlay.vue`) -- `pairOrigin`, `TmManagePairOriginFilter`, `isPairOrigin`, `pairOriginFilterState`, `pairOriginLabel`; invoke key `pairOrigin`.
- [x] Segment (`commands/segment.rs`, `git mv core/segment/origin.rs translation_origin.rs`, `core/segment/mod.rs`) -- rename bindings and the variant, and fix module paths.
- [x] Source document (`git mv core/webimport/origin.rs chapter_origin.rs`, `webimport/mod.rs`, `pipeline.rs`, `import.rs`, `commands/project/*`, `commands/chapter.rs`, `config/project.ts`, `importPreviewState.ts`, `ChapterOrigin.vue`) -- `chapter_origin(s)`; wire key `chapter_origin` on both sides.
- [x] Tests listed in the Code Map -- follow the renames, and rename their bare tokens.
- [x] `tests/naming_boundary.rs` -- new case over Rust and frontend code lines with the one named exemption, a check that the exemption still matches something real, and a positive control: a seeded bare `origin` line is caught, while a CSS class, an i18n key, a string and a `translation_origin` line are not.

**Acceptance Criteria:**
- Given the renamed tree, when the guard runs, then it reports zero bare `origin` tokens outside the dockview exemption.
- Given one renamed site reverted to a bare `origin`, when the guard runs, then it is red and names that file and line.
- Given TM Manage filtering and chapter-origin import preview/editing, when their existing tests run, then they pass unchanged in behaviour.

## Implementation Notes

- Subject names follow the frozen rule: segment and TM translation-origin values are `pair_origin` (type `PairOrigin`), source-document values are `chapter_origin`; `Arbitrated::Origin` became `Arbitrated::PairOrigin`. Wire changes: `tm_list_pairs` argument `pair_origin`/`pairOrigin`, `ChapterSplitPreviewEntryWire.chapter_origin`, and the `tm.invalid_filter` `filter` param value `pair_origin` (no i18n text reads it).
- The guard strips comments and string/char/template literals with its own `code_only` (Rust and TS/Vue modes), so CSS classes, i18n keys and `${...}` bodies are not scanned. A bare identifier used only inside an interpolation is still caught where it is declared.
- The one exemption, dockview's `e.origin` in `WorkspaceDock.vue`, carries its reason in the constant and must match exactly one code occurrence.
- Beyond the Code Map: `tests/frontend/workspaceDockTier.test.ts` had a focus-element binding named `origin`, renamed `focusBeforeDrawer`. The guard does not scan `src-tauri/tests`, `tests/frontend` or `e2e/` (scope unchanged); `e2e/` hits are WebDriver's `origin` key and comments.
- Counter-checks (real edits, then restored):
  - Reverting `PairOriginFilter::admits` to a bare `origin` turned `the_real_source_tree_has_no_bare_origin_identifier` red, naming `core/tm/mod.rs:501/504/505`.
  - Duplicating the `e.origin` line turned the exemption case red.
  - Sending the TS key `origin` instead of `pairOrigin` turned `ipc_argument_contract` red, naming `tm_list_pairs`.
- Verification ran under heavy machine load (Blender render plus a VM, load average 118–128 on 16 cores):
  - The full `cargo test` hung twice at `ipc_contract`, which passes alone (40/40, also via `cargo test --test ipc_contract`). The remaining binaries were run separately.
  - Vitest: one run green 1890/1890; two loaded runs had 3 and 11 failures in files this change does not touch.

## Spec Change Log

## Review Triage Log

- B1 `pair_origin` names a segment's origin, not a pair's — false: the frozen Boundaries name `pair_origin` for both TM pair and segment translation origin (type `PairOrigin`).
- B2 stale `Flow::origins` references in `import.rs`/`pipeline.rs` comments — low, patched (plain backtick text, not intra-doc links; updated with the other stale references below).
- B3 guard blind inside `${...}`/template literals — low, rejected: a bare identifier used there must be declared or typed somewhere in scanned code, and the declaration is caught; scanning interpolation bodies adds parser complexity for a gap not met in this tree.
- B4 `.vue` template apostrophes or regex quotes blank the rest of a line — low, rejected: same reasoning as B3, and template text is i18n-only (`check:i18n`).
- B5 `br"…"` byte raw strings not recognised — low, rejected: 2 occurrences in `src-tauri/src`, and the real-tree scan is green with no false positive.
- B6 guard does not scan `src-tauri/tests`, `tests/frontend`, `e2e`; e2e stubs unchecked; compounds `TmOriginCountWire`/`ORIGIN_PROP`/`UNTOUCHED_ORIGIN` pass — false: the frozen Intent scopes the guard to the tree it already scans and Q2=X to whole tokens; the e2e hits are comments and WebDriver's `origin`, not wire stubs.
- B7 exemption masks every `e.origin` in the file and its reason is not written down — low, patched: the reason is in the constant, and the case asserts exactly one occurrence (counter-checked red).
- B8 `bare_origin_columns` builds an unused `Vec`; the floor test duplicates `the_scanned_trees_are_both_large_enough_to_be_real` — low, patched: now `has_bare_origin -> bool`, and the duplicate test plus `walk_count` were deleted.
- B9 wire renames ship with no compatibility path — false: webview and Rust ship in one binary and these payloads are never persisted.
- B10 `tm[unknown_origin]`, `ORIGIN_PROP`, `tm.manage.origin_*`, `origin_for_all` left unchanged — false: the frozen Never excludes log tags and i18n keys, and Q2=X leaves compounds.
- B11 `chapter_origin` module and re-exports out of alphabetical order — low, patched: moved after `assets`.
- B12 spec, sprint status and the counter-check are not in the diff — false: the spec is the claims file, and counter-checks are recorded in Implementation Notes.
- B13 `Arbitrated::PairOrigin(PairOrigin)` reads circularly — low, rejected: the name is the one the spec's Code Map prescribes, and it is cosmetic.
- E1 `tm.invalid_filter` `filter` param changed from `origin` to `pair_origin` — false: the spec's Tasks prescribe it, and no i18n entry or webview code reads `invalid_filter` (grep of `src/` and `src/i18n/vi.json`).
- E2 doc on `wire::tm_list_pairs` (and on pure `tm_list_pairs`) still says `origin` — low, patched.
- E3 comments cite `chapter.origin` in `work_creation.rs:459-460` and `project/mod.rs:2176` — low, patched (and `pipeline.rs:1125, :1568`).
- E4 bare `origin` inside a string/interpolation/Vue binding passes — low, rejected: same as B3.
- E5 exemption is file-wide, not one occurrence — low, patched with B7.
- E6 `Origins`/`ORIGIN` casings not matched — false: Q2=X names the three tokens.
- E7 nested template literals and regex with quotes confuse `code_only` — low, rejected: same as B4.
- E8 test trees not scanned — false: same as B6.
- E9 nothing ties the TS key `pairOrigin` to the Rust param — false: `ipc_argument_contract` compares them; changing the TS key to `origin` turned it red, naming `tm_list_pairs`.
- E10 a cached payload with `origin` would be rejected — false: preview payloads are produced and consumed in one run, never stored.
- E11 "no behaviour change" claim vs the `filter` param — false: same as E1.
- E12 old module paths may be referenced elsewhere — false: grep of `src`, `src-tauri`, `tests`, `e2e`, `scripts`, `tools` shows no remaining reference to the old paths.
- V1 `tm_list_pairs` Rust/TS argument names not joined by any test — false: same evidence as E9.
- V-other `filter` param value unverified — false: same as E1.
- V-other chapter-origin key pinned independently on each side — false: the reviewer filed it as no gap; the TS guard rejects an entry without `chapter_origin` and the fixtures exercise it.
- V-other exemption is file-wide — low, patched with B7.

## Verification

**Commands:**
- `npm run build`, then in `src-tauri/`: `cargo test` (full: module renames, wire key and IPC-shape change touch shared wiring) -- green.
- `npx vitest run` -- green; `npm run check:i18n`, `npx vue-tsc --noEmit` (or the repo's typecheck script) -- green.
- `grep -rnwE 'origins?|Origin' src-tauri/src src` minus comments/strings -- only the dockview line.
