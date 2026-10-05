---
title: 'Epic 7 retro R-6 — one row per (source, target) in the fuzzy strip and Concordance'
type: 'bugfix'
created: '2026-10-05'
status: 'done'
route: 'oneshot'
review_loop_iteration: 0
context: []
---

<frozen-after-approval reason="human-owned intent — do not modify unless human renegotiates">

## Intent

**Problem:** TM keeps duplicate (source, target) rows on disk on purpose (7-1 D2), but the fuzzy strip (7-5, cap 3) and Concordance (7-7, cap 50) list every stored row, so one pair can take 2–3 of the three fuzzy slots or several Concordance slots (retro F2.1). The TMX export re-implements the AD-18 order by hand instead of using `merge_tiers` (F2.8), and `fuzzy_pairs_in_candidates` turns a missing percent into 0 % with `.unwrap_or(0)` (F3.6).

**Approach:** Use the 7-8 rule in both lists: keep one row per (source, target), the first in AD-18 order (mine before others, Work before Global, newest, highest id), and never merge stored rows. Fuzzy dedupes before the cap of 3. Concordance dedupes before the cap of 50, and `total` counts distinct pairs so the list and its count agree. `distinct_tier_pairs` (TMX export) takes its order from `merge_tiers` and uses the same dedupe helper; its output order (by id) and signature stay unchanged. The percent travels with its row through `merge_tiers`, so no lookup can miss. No schema change, no IPC shape change, no frontend change. `rag.rs` keeps its own dedupe (out of scope).

</frozen-after-approval>

## Implementation Notes

- One helper `first_per_text_pair` in `core/tm/mod.rs` serves the fuzzy strip (`rank_fuzzy_candidates`), Concordance (`rank_concordance`) and TMX export (`distinct_tier_pairs`). `distinct_exact_targets` (7-8) is left as it is: within one source, its target-only key equals (source, target).
- Fuzzy dedupe sits in `rank_fuzzy_candidates`, not in `fuzzy_pairs_in_candidates`, so the RAG path (`rag.rs`, which dedupes in `select_tm_pairs`) keeps its input unchanged. Copies of one pair share a source, so they share a percent, and dedupe before or after the percent sort gives the same result.
- F3.6: `merge_tiers` now delegates to a generic `merge_tagged_tiers<T>`. The fuzzy scorer tags each row with its percent, which removes the `(tier, id) → percent` map and its `.unwrap_or(0)`.
- F2.8: `distinct_tier_pairs` feeds its single tier to `merge_tiers` through `ScopeResolver::global_only()`. `apply_merge` takes its tiers as data, not from `self.work`, and the slot passed in sets `TmTier`.
- New guards in `tests/tm_contract.rs`: `a_pair_stored_three_times_takes_one_fuzzy_slot_…` and `a_pair_stored_twice_is_one_concordance_hit_…`. Both go through the wire commands, and the kept copy is the AD-18 first one: a Global copy marked mine beats a Work copy marked other. Counter-check: removing the two `first_per_text_pair` calls turned both red (fuzzy listed `dup` twice and pushed out `third`; Concordance `total` was 3, not 2).

## Review Triage Log

- Dedupe absent from public `fuzzy_pairs_in_candidates` — false: deliberate, so the RAG input stays unchanged; its doc already says "every pair … uncut".
- No regression test for the TMX refactor — false: `tmx_contract` `exporting_the_work_tier_writes_one_tu_per_distinct_pair_with_its_first_copy…` runs the Work slot (`merge_tiers(Vec::new(), Some(rows))`) with a mine/others duplicate and passes; `resolve_merge` takes tiers as data and ignores `self.work`.
- No test for "percent travels with its row" — false: the percent is attached in the same closure that computes it, so no lookup is left that could miss; the dedupe seam was counter-checked red.
- Spec notes empty / status in-progress — false: a timing artefact; both were filled in at finalize.
- Two dedupe helpers (`distinct_exact_targets` and `first_per_text_pair`) — low, rejected: the choice is recorded in Implementation Notes, and folding them changes a 7-8 contract with no user effect.
- `()`-tag wrapper costs an extra pass; `T: Clone` where `Copy` would do — low, rejected: it is a move-only pass over rows `apply_merge` already clones, with no hot path measured as affected.
- Missing edge tests (cap after dedupe, `tm_empty` with only duplicates, same source with different targets) — low, rejected: `tm_empty` is computed before dedupe and is unchanged, and the fuzzy guard already keeps three different targets of one source as three rows.
- A Global copy marked mine hides a Work copy marked other, so the tier badge shows Global — false: this is the AD-18 rule the Intent names (side before tier), the same as 7-8.
