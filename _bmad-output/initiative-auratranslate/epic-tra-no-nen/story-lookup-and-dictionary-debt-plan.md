---
ticket: 3
title: 'Story 11.3 — the lookup path Concordance builds on says why it is empty'
type: 'chore'
created: '2026-09-26'
status: done
baseline_revision: 'c6e5e215326153a25befe3af41cdae54c17ba142'
route: 'dispatch'
review_loop_iteration: 1
context:
  - '{project-root}/src-tauri/AGENTS.md'
  - '{project-root}/src/AGENTS.md'
  - '{project-root}/_bmad-output/initiative-auratranslate/archive-v6/epic-11-context.md'
---

<frozen-after-approval reason="human-owned intent — do not modify unless human renegotiates">

## Intent

**Problem:** On HEAD `c6e5e21`, 36 `deferred-work.md` items end in `Chủ: Story 11.3` (29 open, 7 🟡). The Task 0 re-read is in `11-3-task0-2026-09-26.md`. The lookup path that Story 7.7 builds on still has silent-emptiness holes: when every `.db` is unreadable, `layers_loaded` says "no layer attached" (two sites, the ledger named one), and the attribution list drops an unreadable layer without a trace. Six `SkipReason` variants have no behaviour case, and the branch-1/2 query plan is only checked by hand. On the webview side, parallel-view copy leaks readings on WKWebView, and `.hv-unit` still follows the 2026-08-17 signature that Ice reversed in phiếu quyết #89.

**Approach:** One spec for all 36 items, no lots. Each item gets one Epic 11 disposition, as listed below. Every fix gets a guard, and each guard is counter-checked by really removing its seam.

## Boundaries & Constraints

**Always:** Ledger items keep their text. Only `→` lines are appended, and a stale claim gets a 🔵 fix in place. A check that only a person can make in the real app goes to `Chủ: Epic 11`. A wire-shape change updates `ipc_contract.rs` and the `ipc_argument_contract.rs` scan stays green. Stored segment boundaries are never rewritten.

**Never:** Patch Unicode normalization or the branch length metric locally (L437/L7753/L789 go to Winston). Touch `tools/dict-build/src/schema.rs` or bump `SCHEMA_VERSION`. Add a gate or a dependency. Build a new capability (L1223).

## Dispositions

Agent, 2026-09-26 (from Task 0; Ice may override any line at approval):
1. ✅ fix, Rust: L840 (both `layers_loaded` sites), L1162 (`list_source_attributions`/`list_dict_sources` also return skipped layers; `AttributionOverlay` shows a third state, "some layers unreadable", distinct from `empty`), L604 + L607 (a)(b)(c)(e)(f)(g) (new `dict_sources.rs` cases; (d) is already covered at `phase_two_never_mixes_entry_ids…`), L426 (an `EXPLAIN QUERY PLAN` test on the verbatim-DDL fixture: branch 1/2 use an index, never `SCAN`), L434 (enforce `char_idx`'s ≤2-char precondition by a type that only `pick_branch` can build, not by `debug_assert!` or a source scan), L445 (drop the now-false `epics.md` clause from the `matching_boundary.rs` doc and assert message, and keep the still-true spine-diagram clause).
2. ✅ fix, webview: L1005 (in `parallel` view, `onCopy` rebuilds the copied string through `resolveParallel`; `switch` view keeps copying the displayed reading, with `U+2060` turned into a space, as signed in Story 1.18b AC5 — amended by Ice 2026-09-26 after review), L4142 (phiếu quyết #89: `.hv-unit` carries `data-src-atomic="1"`; `hanVietCutAnchors.test.ts` flips its assertions; the `SRC_ATOMIC` doc drops the clause that is no longer true), L1042 ①② (a vitest: toggling a source redraws the chip into a non-opacity disabled state) and ④ (an e2e that Tabs from the chip strip to the Attribution rows; if the driver cannot move focus by Tab, ④ goes to `Chủ: Epic 11`). ③ is already covered by `attribution-focus.e2e.mjs`.
3. ✅ self-closed with a pointer: L590 (`MINIMUM_SCHEMA_VERSION`, `19ea24c`).
4. `KHÔNG LÀM` with a reopen condition: L388, L397, L780, L786, L792, L429, L1060, L4649, L1201, L1209, L1327, L1341, L1715, and L1223 (a new capability; reopen as a PM request).
5. Reassign: L437, L7753, L789 → `Chủ: Winston` (one AD for NFC/NFD and graphemes, next to Story 7.6's matcher sibling). L326, L584 → `Chủ: Ice` (condition: a raw HVTĐTD / Cổ Hán văn source exists). L774 → `Chủ: Ice` (legal decision before the first public release). L1216 → `Chủ: Epic 11` (the real-use pass judges the 200-row ceiling).

Ice, 2026-09-26:
6. L831 (option A): a phase-two `senses()` failure is reported in the response's skipped/truncated shape, not turned into an empty sense list. `LookupResponse` changes shape.
7. L816 (option c): `KHÔNG LÀM`. Interleaved text at that scale is adversarial, and no real chapter is known to hit it. Reopen when a real chapter goes over the ceiling in node count.
8. L352 (option a): `require_nonempty` also fails on a skip ratio above N. The Rust phase measures each source's current skip ratio on a real build and proposes N. Ice signs N before it lands.
9. L391 (option b): `KHÔNG LÀM`. The fixture-only test stays. Reopen when a real-data regression in the English layer is found.

</frozen-after-approval>

## Code Map

- `src-tauri/src/core/dict/mod.rs` -- `layers_loaded` at :935 (`GroupedLookup`) and :1218 (`HanVietLookup`). `list_source_attributions` is at ~:962. `layers.skipped()` is the accessor to reuse (used in `lookup_grouped`). Do not touch `pick_branch`'s length metric.
- `src-tauri/src/core/dict/query.rs` -- `char_idx` ~:232 (`debug_assert!`), and the branch-1/2 SQL.
- `src-tauri/src/core/dict/layer.rs` -- `SkipReason`, `DictLayers::open`. Leave the schema-version constants alone.
- `src-tauri/src/commands/dict.rs` -- `list_dict_sources`, and `lookup()` :254 `unwrap_or_default()` (Decision 6).
- `src-tauri/tests/dict_sources.rs` -- `build_layer`/`LayerSeed`/`temp_dir`, the fixture pattern in `a_broken_layer_is_skipped_by_name_and_the_rest_still_answer`, and `COPIED_DDL` + `fixture_ddl_is_verbatim_from_dict_build_schema`. A counter-check edits only a throwaway DDL copy, never `tools/dict-build/src/schema.rs`.
- `src-tauri/tests/matching_boundary.rs` -- doc :1-27 and message :248-267. Text only; the assertion logic stays.
- `src/AttributionOverlay.vue` ~:187, `src/panels/dictSourcesState.ts` -- the `load_failed`/`empty` states that L1162 extends.
- `src/panels/SourceHanViet.vue` -- `onCopy` ~:792, the `.hv-unit` span in the parallel branch ~:926. `src/panels/editorSegments.ts` `SRC_ATOMIC` doc ~:336 (the logic already reads the attribute).
- `tests/frontend/hanVietCutAnchors.test.ts` :97-110 and :162-175 -- these assert the old signature and must flip.
- `tools/dict-build/src/build.rs` -- `require_nonempty` :105, and the per-source `BuildReport` stats (`lines_read`/`lines_skipped`) (Decision 8). The call sites stay unchanged.
- `e2e/specs/attribution-focus.e2e.mjs` -- the pattern for L1042 ④. Its assertions stay.

## Tasks & Acceptance

**Execution:**
- [x] `core/dict/{mod,query,layer}.rs`, `commands/dict.rs`, `tests/{dict_sources,matching_boundary,ipc_contract}.rs` -- Disposition 1 + Decision 6 -- Rust phase.
- [x] `tools/dict-build/src/build.rs` + its tests -- Decision 8: measure the ratios, propose N, wait for Ice's signature, then land the check with a near-empty-decode fixture that must fail the build.
- [x] `SourceHanViet.vue`, `editorSegments.ts`, `AttributionOverlay.vue`, `dictSourcesState.ts`, i18n, new/changed `tests/frontend/*` -- Disposition 2, and the webview halves of L1162 and L831 -- Webview phase.
- [x] `e2e/specs/` (new, L1042 ④) and a counter-check for every new guard -- Tests phase.
- [x] `deferred-work.md` -- one `→` disposition for each of the 36 items. Story 2.9's spec gets a 🔵 fix, pointing to #89, where it records the 2026-08-17 per-character parallel cut.

**Acceptance Criteria:**
- Given the story is done, when `npm run check:debt-owner` runs, then it is green, and no open or 🟡 item ends in `Chủ: Story 11.3`.
- Given a layer whose phase-two `senses()` fails after phase one hit, when lookup runs, then the response names that layer as failed instead of returning an empty sense list.
- Given a Library directory where every `.db` is unreadable, when lookup, Hán Việt read and the attribution list run, then none of them reports "no layer attached". Each names the skipped layers.
- Given each new guard, when its seam is really removed, then it goes red for that reason. For L840, reverting either site reds its own case.
- Given a keyboard selection in parallel view, when it is copied, then the clipboard holds the source characters only.
- Given parallel view, when a cut is placed inside a Hán word, then it lands on the word boundary and is drawn.

## Implementation Notes

- Decision 8, signed by Ice 2026-09-26: option B. The ratio excludes every skip reason that contains `filtered, expected`, and N = 60 %. Measured on a real `--layer all` build of all 10 sources: the highest genuine ratio is en-wiktionary at 42,99 %, then en-wiktionary-vi at 13,00 %. The raw ratio would need N ≥ 99,7 %.
- Decision 8 is landed (`tools/dict-build/src/build.rs::genuine_skip_ratio`/`require_nonempty`), guarded by three unit tests built directly from `SourceStats`, no raw-fixture pipeline needed.
- L831/Decision 6 and L1162 both change wire shape: `LookupResponse` gained `senses_failed: Vec<String>`; `list_dict_sources` now returns `{ sources, skipped }` instead of a bare array. Both are reflected in `ipc_contract.rs` and consumed on the webview side.
- L437 (`core/dict::pick_branch`), L789 (`nom_guard`), L7753 (dict + Library FTS) are one NFC/NFD + grapheme-cluster family, reassigned whole to Winston for a single AD — patching any one of the three locally would have created a third independently-chosen normalization behavior next to Story 7.6's matcher fix.
- L786: Task 0's own Group A misread `split_readings`'s doc-comment — the 24,8 % figure is a documented, already-fixed Story 1.16 bug, not a live exposure. Disposition corrected to `KHÔNG LÀM` at the spec/ledger level; no Ice decision was needed.
- L607(g): Task 0 claimed both `FtsTrigram` and `CharIdx` 2-char branches were unguarded at the aggregation layer; only `FtsTrigram` actually was — `CharIdx` already had `source_counts_are_verified_not_candidate_counts`.
- L4142/phiếu quyết #89 landed: `.hv-unit` now carries `data-src-atomic="1"` in `parallel` view, reversing the 2026-08-17 signature. Story 2.9's AC9 (`chính xác từng chữ ở parallel`) got a 🔵 in-place correction pointing at #89, per "a claim that stops being true is fixed in place, not deleted."
- L1042 ④ measured, not built: on this machine's WKWebView e2e driver, `Tab` does not move focus between `<button>` elements at all (12 consecutive presses, same active element every time) — this contradicts Task 0's stated assumption that only keyboard *activation* was broken. Per the spec's own escape valve, the e2e spec was written, measured, and removed rather than kept permanently red; disposition reassigns to `Chủ: Epic 11`.
- The red `check:i18n` Phase 2 left (two Vietnamese `assert!` messages inside an inline `#[cfg(test)]` module under `src/core/dict/query.rs`) was resolved by translating to English, not by extending the named-exemption list — same reasoning the existing `tests/**` exemption already states, and it removes the actual violation instead of growing `EXEMPT`.

## Spec Change Log

- 2026-09-26, loop 1. Trigger: review found that `⌘C` in `switch` view copied source Han characters instead of the displayed reading. Frozen Disposition 2 named `resolveSwitch`, while L1005 only concerns `parallel`. Amended (Ice approved): L1005 rebuilds through `resolveParallel` only, and `switch` keeps the Story 1.18b AC5 copy. This avoids a clipboard that differs from what the user selected. Ice chose a targeted patch over a full revert. KEEP: every other change in the diff, the `parallel` rebuild and its two tests, and the `.hv-unit` atomic anchor.

## Review Triage Log

- edge+verification-gap+blind: in `switch` view, `onCopy` now copies source Han characters instead of the reading the user sees, because `resolveSwitch` returns `seg.chars[j]`; the `WORD_JOINER` `replaceAll` is now dead code — high, intent_gap: frozen Disposition 2 names `resolveSwitch` too, while L1005's own text scopes the fix to `parallel`, and Story 1.18b AC5 signed the switch-view copy.
- blind: `switch`-view copy has no test (`sourceHanVietCopy.test.ts` mounts only `parallel`) — high, same root cause as above.
- blind: new comments and test names in 17 files carry story ids, ledger lines, decision numbers, dates, measurements, 🔴/🔵 markers, `───` banners, and multi-line Vietnamese prose — medium, patch: AGENTS.md §Code comments bans each form. Rewrite to at most two English lines, or delete.
- blind: the phase 1 handoff says three counter-checks are missing but lists five — low, patch: correct the count in the handoff file.
- blind: the phase 3 handoff promises a full `cargo test` result and never appends it — low, patch: append the measured result (exit 0, 68 binaries, 1833 passed).
- blind: `SourceAttributions.skipped` merges never-opened layers with layers whose `dict_source` died later — false: each entry serialises its own `SkipReason` wire code (`OpenFailed`… vs `SourcesUnreadable`), so the wire keeps the distinction. The AC needs only the one banner.
- blind: `ShortQuery(..)` can be built directly inside `query.rs` — low, rejected: Rust privacy already stops every caller outside the module, and closing the gap means a new submodule.
- blind: the L604 ledger line claims all five counter-checks without saying they spanned two phases — false: the claim holds for the final tree, and the phases file records the split.
- blind: nothing guards `AttributionOverlay`'s banner-only combination — false: `attributionOverlayUnreadable.test.ts` asserts that no `attribution.empty` shows when every layer is skipped, and removing the guard reds it.
- blind: the Spec Change Log is empty although dispositions were corrected — rejected: the fix edits this build's spec.
- edge: no test pins a genuine skip ratio of exactly 60 % — low, rejected: `>` passes 60 % by design, and a real source landing on exactly 0.600 is unlikely.
- verification-gap: `lookupHistoryState.ts::targetsOf` snapshots `gloss: null` when the entry's layer is in `senses_failed`, so a pin keeps "no gloss" forever — medium, defer: pre-existing. Before this change, `unwrap_or_default()` gave `[]`, so the snapshot was the same `null`.

## Verification

**Commands:**
- `npm run build && cargo test --test dict_sources --test matching_boundary --test ipc_contract --test ipc_argument_contract` -- expected: green.
- `npx vitest run tests/frontend/hanVietCutAnchors.test.ts <new files>` -- expected: green.
- `npm run check:debt-owner && npm run check:i18n && npm run check:tokens` -- expected: green.

## Acceptance criteria from epics.md

Source: `epics.md` §Story 11.3 (v6, nay ở archive-v6).

As a chủ dự án,
I want nợ nhóm tra cứu và dữ liệu từ điển (Epic 1) được đóng hoặc quyết dứt điểm,
So that Concordance ở Story 7.7 dựng lên đường tra cứu đã vá.

Thừa kế AC chung của epic (xem Notes của `epic-tra-no-nen.md`).

**Acceptance Criteria:**

**Given** mọi mục mang `Chủ: Story 11.3` trong `deferred-work.md`
**When** story hoàn tất
**Then** thoả AC chung của Epic 11; AC riêng rút từ các mục lúc `create-story`
