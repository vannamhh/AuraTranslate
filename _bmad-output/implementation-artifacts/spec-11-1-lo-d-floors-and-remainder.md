---
title: 'Story 11.1, lot D — population floors and the remaining gate debt items'
type: 'chore'
created: '2026-09-25'
status: 'done'
route: 'dispatch'
baseline_commit: '90d4428a39eee87dc1646b20f6189f203dd08b74'
review_loop_iteration: 0
context:
  - '{project-root}/scripts/AGENTS.md'
  - '{project-root}/src-tauri/AGENTS.md'
  - '{project-root}/_bmad-output/implementation-artifacts/epic-11-context.md'
---

<frozen-after-approval reason="human-owned intent — do not modify unless human renegotiates">

## Intent

**Problem:** Eighteen `deferred-work.md` items still end in `Chủ: Story 11.1` on HEAD `90d4428`. The largest group is the population floors. There are 48 floor constants (27 in `src-tauri/tests/**`, 21 in `scripts/**`). Only 3 still sit at 80% or more of the live population they guard, and the worst is at 5.7%. No mechanism stops a floor from drifting, so each one quietly stops guarding anything. The other thirteen items are smaller gaps in the gates, a CI verdict that nobody reads, and a WAL test that fails in CI right now.

**Approach:** Lot D is the last of four lots. The story goes `done` after it. Give each of the 18 items exactly one Epic 11 disposition after a HEAD re-read, following the decisions below. Build each guard inside the gate that already owns its surface.

## Boundaries & Constraints

**Always:** Every new Kiểm or guard carries a self-check that proves it can go red and does not go red wrongly. It is also counter-checked by really removing its seam. Measure each touched gate's verdict and scanned population before and after the change, and explain every count that changes. Ledger items keep their text: only `→` lines are appended.

**Never:** Add a dependency (NFR15). Make a visible UI change. Write speculative code for an item closed as `KHÔNG LÀM`. Lower a floor below `ceil(0.85 × live)`.

## I/O & Edge-Case Matrix

| Scenario | Input | Expected |
|---|---|---|
| Floor drifted | a floor below 80% of its gate's live count | that gate is red, and names the constant and `ceil(0.85 × live)` |
| Tree truncated | live count below the floor | red, as today |
| Floor bypass | a new `*_FLOOR` constant that never reaches the ratio helper | `check:gates` is red |
| Half closed | `→ ✅ ĐÓNG MỘT NỬA …`, or a `- ✅` line that says "Còn mở" | `check:debt-owner` scores it `half` |
| Reversed phrase | `→ ✅ ĐÃ ĐÓNG … CẢ HAI VẾ CÒN HỞ đã đóng` | stays `closed` |
| Phrase on a non-✅ line | a `→` note that mentions "một nửa" mid-sentence | status is unchanged |
| Shadow var | `--dv-floating-box-shadow: none` | red unless it carries `aura-allow-shadow: <reason>` |
| Empty exemption | `<!-- aura-allow-text: -->` | `check:i18n` is red |
| Bound string style | `:style="'color: red'"` | value `red` |
| Previous CI run red | `pre-push` while the last push's `CI` run has `conclusion == failure` | one warning line, push continues |
| Offline, or `gh` not authenticated | `pre-push` | silent skip, push continues |

## Decisions (Ice, 2026-09-25)

1. **One spec** for all 18 items. Ice accepts the size.
2. **Floors (L289, L4639, L7185, L9434, L11117) use option (b) with an 80% threshold.** One shared helper per language checks both `live ≥ floor` and `floor ≥ 0.8 × live`. Each gate passes in its own live count, so every constant keeps its own population root, including the shape counts (`CLICK_FLOOR` and the like) and the subdirectory floors. All 48 constants are raised to `min(live, ceil(0.85 × live))`. The measured cost: at the growth rate of the last two months, a floor set to 85% goes red again after about 4 days. That re-raise is the intended signal.
3. **L119/L190: `KHÔNG LÀM`.** The in-script self-checks already run on every `pre-push` and CI run. The ledger line names the class this misses: a Kiểm block skipped by a shared early return goes green by absence. It is reopened when that happens once.
4. **L653: classify custom properties by name suffix.** `-box-shadow`, `-text-shadow`, `-opacity` and `-z-index` fall under Kiểm D/F. Kiểm F gains a named `aura-allow-shadow: <reason>` exemption, with the same reason rule as `exemptAt`. The four `--dv-*` vars get correct markers.
5. **L4164: shared `:root` custom properties** for the cut-mark shape, declared in `src/tokens/reset.css` and read by both blocks.
6. **L5628: `KHÔNG LÀM`.** The item is reopened if the bench is used again. Today all 14 values match.
7. **L5794: `KHÔNG LÀM`** (Ice, renegotiated at review, 2026-09-25). Gates and tests check source code and app behaviour only, never task status or documents in `_bmad-output`, and the BMAD process is not touched. The Kiểm D built for it is removed, and `2-13`/`spec-5-14` keep their original status.
8. **L4785: reassign to `Chủ: Story 11.5`.** In the 14 sampled CI runs (2026-09-14 to 09-25), the WAL test was red 7 times, on both OSes, with `wal_checkpoint(PASSIVE)` reporting `busy=1`.
9. Already decided: L11951 follows ticket #42 (only a line that itself starts with ✅ is downgraded by a half-phrase; measured 4 flips, 0 false positives, 0 new reds). L4761 follows ticket #37 (warn, never block).

</frozen-after-approval>

## Code Map

The full measurements are in the scratchpad `/private/tmp/claude-501/-Users-hoangnam-LocalSites-addon-AuraTranslate/6d66edbc-a734-4107-b502-90eba240847d/scratchpad/lotd-{floors,debtgate,webview,infra}.md`: the 48-floor inventory with live counts, the candidate predicates, the pre-push script draft, and the CI evidence. If those files are gone, rebuild the facts from the ledger items.
- `src-tauri/tests/support/boundary_scan.rs` -- lot B's shared scanner. It has no assert today; add the floor-ratio helper here. `naming_boundary.rs` walks the tree locally and must call the helper too. `dict_boundary.rs::SRC_TAURI_RS_FLOOR` counts `src-tauri/{src,tests}` (161), not 98.
- `scripts/lib/` -- add a pure floor judge that each JS gate calls with its own count. `check-commands.mjs` shape floors: `CLICK_FLOOR`/`DISPATCH_FLOOR`/`COMMAND_FLOOR`/`SELECTION_SURFACE_FLOOR`/`HANDLER_ATTR_FLOOR`. `check-dict-build.mjs::RS_FILE_FLOOR` is at 24/24.
- `scripts/check-gates.mjs` -- home of the bypass Kiểm: every `*_FLOOR` constant in `scripts/**/*.mjs` and `src-tauri/tests/**/*.rs` must reach the helper. `CONTRAST_FLOORS` and `LINE_HEIGHT_FLOOR` (thresholds, not counts) get named exemptions.
- `scripts/check-debt-owner.mjs` -- `leadingStatus` :245, `continuationStatus` :271 (share one half-phrase regex); `SELFTEST_CASES` :359-434, `runSelftest` :464.
- `scripts/check-tokens.mjs` -- Kiểm D :1313-1368 and Kiểm F :1415-1455 match `d.prop` exactly; `exemptAt`. `src/**/dockview-theme.css` :87, :101, :129, :137 hold the four vars.
- `scripts/lib/tokens-scan.mjs::inlineStyleBlocks` :199-236; `tests/frontend/checkTokensScan.test.ts:132-140` pins the wrong value.
- `scripts/check-i18n.mjs:1095` -- the `aura-allow-text` regex; add `(?!-->)`, mirroring lot C.
- `src/commands/index.ts:1060` `PANEL_SUFFIXES` → derive it from `PANEL_IDS` (`src/layout/workspaceLayout.ts:54`). There is no import cycle.
- `GridPanel.vue:2155-2161`, `SourceHanViet.vue:952-959` -- the cut-mark blocks.
- `tests/frontend/editorClearSourceCuts.test.ts` -- precedent for `portMissing` and for real `KeyboardEvent` dispatch in happy-dom. All 132 registered commands are port-backed.
- `.githooks/pre-push` -- add a plain-shell step that no `check:` script names, so `check:gates` does not count it. It runs `gh run list --workflow ci.yml --branch master --event push`, matches on `git rev-parse @{u}`, and is wrapped in `perl -e 'alarm shift; exec @ARGV' 8`.

## Tasks & Acceptance

**Execution:**
- [x] `deferred-work.md` -- Task 0: re-read the 18 items against HEAD.
- [x] `boundary_scan.rs`, the 20 Rust test files -- Rust floor helper, the Rust floors raised, and every Rust floor routed through the helper (Decision 2).
- [x] `scripts/lib/`, the 11 gate scripts, `check-gates.mjs` -- JS floor judge with its vitest file, the JS floors raised, and the bypass Kiểm (Decision 2).
- [x] `check-debt-owner.mjs` -- the L11951 predicate plus its self-check cases (Decision 7 was dropped: no Kiểm D).
- [x] `check-tokens.mjs`, `dockview-theme.css` -- suffix classification and `aura-allow-shadow` (Decision 4).
- [x] `tokens-scan.mjs`, `checkTokensScan.test.ts`, `check-i18n.mjs` -- L12241 and L12238, each with a case.
- [x] `src/commands/index.ts`, `reset.css`, both cut-mark blocks -- L674 and Decision 5.
- [x] `tests/frontend/` (new file) -- every registered command reports `portMissing` without its port, and `Mod+Alt+C` is dispatched through the real keymap (L5868).
- [x] `.githooks/pre-push` -- the CI-verdict warning (L4761).
- [x] `deferred-work.md` -- a disposition for all 18 items (`✅` with evidence, `KHÔNG LÀM` with a reopen condition, or L4785 → `Chủ: Story 11.5`).

**Acceptance Criteria:**
- Given lot D is done, when `npm run check:debt-owner` runs, then it is green, and no open or 🟡 item ends in `Chủ: Story 11.1`.
- Given each new guard, when its seam is really removed, then it goes red for the measured reason, and it goes green again once the seam is restored.
- Given a floor lowered in a scratch edit to below 80% of live, when its gate runs, then it goes red.
- Given the change touches `scripts/check-*` and `src-tauri/tests`, when the full suite runs once, then it is green, or every red is explained.

## Implementation Notes

- Both floor helpers check `live >= floor` and `floor >= 0.8 × live`; `check:gates` Kiểm G makes every `*_FLOOR` constant (48: 27 Rust, 21 JS) reach one of them. `CONTRAST_FLOORS` and `LINE_HEIGHT_FLOOR` are named exemptions.
- `check-deps.mjs::RUST_TREE_FLOOR` counts the raw `cargo tree --no-dedupe` lines (360), not the deduped names the investigation used (337).
- The first L11951 predicate (last half-phrase vs last "đã đóng") turned L139/L9170 from 🟡 back to open and missed L204. It was replaced by the measured candidate: 4 flips, 0 false positives, with the old "vẫn mở/còn mở on any `→` line" rule kept.
- Kiểm D (L5794) was built, then went red on this very story in review, because the workflow's transient states count as drift. Ice dropped it (Decision 7): gates check source code and app behaviour, never `_bmad-output` status.
- The CI-verdict step moved from inline shell + `perl` to `scripts/ci-previous-verdict.mjs` (`spawnSync` with a timeout), so vitest covers every branch on Windows too. A cold `gh` start can take about 14 s here, so the first run after boot is silent.
- L674 and Decision 5 are source fixes with no new gate: `PANEL_SUFFIXES` is derived from `PANEL_IDS`, and the cut-mark size lives in `:root`. Kiểm I catches a typo in the shared var.
- Floor history comments (dates, story ids, ratios) attached to the raised constants were deleted, because the ratio rule now lives in the helpers.

## Spec Change Log

## Review Triage Log

- edge+blind: Kiểm D is red on this very story (sprint `in-progress`, lot D spec `in-review`); the workflow's transient states (`draft`/`ready-for-dev` spec with sprint `backlog`, `in-review` with `in-progress`) are scored as drift, so any push during planning or review is blocked — high, intent_gap: Decision 7's pair set does not cover them. Resolved by Ice: L5794 becomes `KHÔNG LÀM` and Kiểm D is removed (Decision 7).
- verification-gap: `ci-previous-verdict.mjs` `--json headSha,conclusion,status` is not tied to the fields it reads; the tests ignore `args` — medium, patch: assert the `--json` field list in the test.
- verification-gap: Kiểm D picks the last file by lexical sort, so a key with a bare story file and an unrelated `spec-<key>-*.md` (today `3-5`, both `done`) compares against the `spec-` one — maybe-false (no wrong verdict today), defer. Moot: Kiểm D removed.
- verification-gap other: the L4761 closing line says a shell + `perl` step shipped; the audit fix moved it to `scripts/ci-previous-verdict.mjs` — low, patch: 🔵 correction on that line.
- edge: `inlineStyleBlocks` splits a bound string on `;` inside a quoted value — low, rejected: the static branch shares the shape, no `src/**` binding has it, and the fix adds a quote-aware scanner.
- edge: `STORY_KEY_RE` drops keys with a two-letter suffix — false: `parseSprintStatus` throws on such a key (`SPRINT_STORY_RE` has the same `[a-z]?`), so it cannot reach Kiểm D silently.
- edge: `printVerdict` could write a second label if it throws mid-way — low, rejected: `process.stdout.write`/`console.log` do not throw here.
- edge: `check-doc-refs.mjs::FILE_FLOOR = 366` while `ceil(0.85 × 433) = 369` (the lot's own new files) — low, patch: set 369.
- blind: `floor-judge.mjs` missing from `tsconfig.node.json` `include` — false: every including script imports it, so `checkJs` type-checks it through the import graph.
- blind: the full suite was never run — false: 12 gates, vitest 1470/1470, build and `cargo test --no-fail-fast` 1813/0 over 66 binaries were run on the phase 1–5 tree.
- blind: Rust floor doc-comments still state the old numbers (`AI_FLOOR` "Số thật hôm nay: 1", `DICT_FLOOR` "4/5 = 80,0%", `glossary_boundary.rs` 44/53 lines, and the other raised constants) — medium, patch: delete the measurement/history lines attached to each changed constant.
- blind: `timed_out`/`cancelled` previous runs print nothing — low, rejected: the matrix row is `conclusion == failure`, and the fix adds branches for rare outcomes.
- blind: `commandsRegistryPortMissing.test.ts` uses a literal `>= 150` count guard — low, rejected: a test sanity bound, not a gate floor; each command is still asserted one by one.
- blind: the L653 closing line says "cả ba biến" but a fourth var came under the rule — low, patch: 🔵 correction naming `--dv-tab-group-line-opacity`.
- blind: `F_SHADOW_EXEMPT_CASES` builds its fixture by hand instead of going through `parseCssBlocks`, so the self-check guards the functions, not the wiring — medium, patch: run the fixtures through `parseCssBlocks`.
- blind: the Phase 3 handoff note describes `hasUnreversedHalfPhrase`, which the shipped code no longer has — low, patch: correction note in the phases file.

## Design Notes

- Why a per-gate helper instead of one central Kiểm (L4639 proposed one in `check:gates`): only a gate knows its own live count. The shape floors have no directory root, so a central Kiểm would have to re-implement every gate's counter. That is exactly the "copied number drifts silently" failure. The central piece that remains is the bypass Kiểm, which checks that every floor constant reaches the helper.

## Verification

**Commands:**
- `npm run check:debt-owner && npm run check:gates && npm run check:tokens && npm run check:i18n` -- expected: green.
- `npm run build && cargo test --manifest-path src-tauri/Cargo.toml`, then vitest -- expected: green (full suite, once).
