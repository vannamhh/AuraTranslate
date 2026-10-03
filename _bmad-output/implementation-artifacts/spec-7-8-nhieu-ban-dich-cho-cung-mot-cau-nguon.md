---
title: 'Story 7.8 — Multiple translations for one source sentence: keep all, show all with a date, the translator picks'
type: 'feature'
created: '2026-10-03'
status: 'done'
route: 'dispatch'
baseline_commit: '3a2e03c6cef2853a63db7fb7ed89c6fa445828e6'
review_loop_iteration: 0
context:
  - '{project-root}/AGENTS.md'
  - '{project-root}/src/AGENTS.md'
  - '{project-root}/src-tauri/AGENTS.md'
  - '{project-root}/_bmad-output/implementation-artifacts/epic-7-context.md'
---

<frozen-after-approval reason="human-owned intent — do not modify unless human renegotiates">

## Intent

**Problem:** The TM already keeps every confirmed pair (append-only `insert_pair`, spec 7.1 D2), but nothing shows the translator the several targets one source has: 7.4 silently pre-fills the head of AD-18, 7.5 hides its strip whenever an exact pair exists, and no TM DTO carries a date. AD-18's third key "then date" (FR63, epics §Story 7.8) is missing; today's tie-break is `id` ascending (spec 7.3:27 handed the date key to 7.8).

**Approach:** Add `created_at` as the third AD-18 key inside `core/tm`, ship it on the wire, and list every distinct target of an exact-source match, each with its date, in the 7.5 strip slot so the translator picks one.

**Decisions (Ice, 2026-10-03):**
- Q1 ⇒ (A) when the active segment's source has 2+ distinct exact targets, the 7.5 strip slot shows ALL of them (no cap of 3): target, date, side, tier, the row equal to the current text marked as in use; same keys as 7.5 (Mod+Alt+E, ↑/↓, Enter or 1–9, Esc). Concordance rows gain the date too.
- Q2 ⇒ (a) a picked row writes the pair's own origin, under the existing AD-47 ③ row "Điền sẵn từ TM khớp 100%"; no spine change.
- Q3 ⇒ (i) 7.4 still pre-fills the head; the list lets the translator switch.
- Q4 ⇒ newest first within the same side and tier (`tm-manage.html:174-186`), so 7.4 now pre-fills the latest translation instead of the oldest.

## Boundaries & Constraints

**Always:**
- Order = side (mine first) · tier (Work before Global) · `created_at` descending · `id` descending; one rule in `core/tm`, used by every consumer (`pairs_for_source`, `rank_fuzzy_candidates`, `rank_concordance`).
- Writing stays append-only (AD-6); the existing tests `editing_then_reconfirming_adds_a_pair_…` and `the_same_pair_confirmed_twice_…` pass unchanged.
- Rows with an identical `target_text` collapse into one list row (the first in order); collapsing is a read concern, stored rows are never merged.
- One distinct target ⇒ no list, 7.4/7.5 behaviour unchanged. The list and fuzzy matches never appear together.
- A pick re-reads the pair by tier and id, refuses a pair whose source is no longer the segment's source, and writes through `write_non_user_target` (baseline + origin together, `draft`, no SegmentVersion).
- A pick asks before overwriting only when the current text was typed by the user: non-empty, ≠ `baseline_target_text`, and no copy in `segment_version`. Replacing a pre-filled or previously picked text never asks.
- Dates render through `historyTimeLabel` (no `Intl.RelativeTimeFormat`, NFR16).

**Never:**
- A migration, an index change, a new dependency, or an UPDATE/DELETE on `tm_unit`.
- TM management grouping, edit or delete (Story 7.9), TMX (7.10).

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected |
|---|---|---|
| Two targets, same side and tier | Work mine `(S,A)` 2026-06-28, `(S,B)` 2026-08-03 | list B then A, each dated; 7.4 pre-fills B |
| Side beats date | Global mine `(S,A)` old, Work other `(S,B)` new | A first |
| Duplicate target | `(S,A)` ×3, `(S,B)` ×1 | two rows |
| One distinct target | `(S,A)` ×2 | no list, no fuzzy strip |
| Pick over pre-fill | draft = pre-filled B, pick A | A written with A's origin, no prompt |
| Pick over typed text | draft typed by user, never confirmed | `needs_confirmation`, nothing written until `force` |
| Pair gone / source differs | stale tier+id | `tm.pair_not_found`, nothing written |

</frozen-after-approval>

## Code Map

- `src-tauri/src/core/tm/mod.rs` -- `insert_pair` :70 (keep); `TmPair` :90 and `RawPair` lack `created_at`; SELECTs :147 :156 :376 load `ORDER BY id`; `pairs_for_source` :193; `merge_tiers` :204 → `ScopeResolver::apply_merge` (`core/scope/resolve.rs:215`, stable sort on side then tier — loading each tier newest-first gives the date key without touching `core/scope`); `rank_fuzzy_candidates` :298 (identical sources excluded :284); `rank_concordance` :362; `pair_by_id`.
- `src-tauri/src/core/store/schema.rs:891` -- `created_at` is ISO-8601 UTC with ms (lexicographic = chronological); read only.
- `src-tauri/src/commands/segment.rs` -- `fill_exact_tm_matches` :1180 (`.next()`, keep); `write_non_user_target` :2219; `promote_ai_translation` :2261 (unsigned-draft guard :2301, copy its shape, add the baseline exemption for the new pick only); `TmFuzzyMatch` :2353; `prepare_tm_fuzzy` :2391 returns empty on exact pairs (:2419) — return the distinct exact targets there instead when 2+; `TmConcordanceHit` :2486; `accept_tm_fuzzy` :2574 (writes `other`; the exact pick is a sibling `accept_tm_exact` with the pair's origin); `lib.rs` registration.
- `src/config/segment.ts:1137`, `:1239` -- hand-written TM types, guards, `invoke` wrappers.
- `src/TmFuzzyStrip.vue:107-135`, `src/tmFuzzyStripState.ts`, `src/tmFuzzyCommandDeps.ts` -- extend for the exact list; do not fork a second strip.
- `src/panels/LookupPanel.vue:673-682` -- Concordance rows. `src/panels/segmentHistoryTime.ts:100` `historyTimeLabel`.
- `src/i18n/vi.json` -- reuse `tm.fuzzy.side_*`/`tier_*`; new keys for the list heading and "in use".
- `src-tauri/tests/tm_contract.rs` -- `same_side_and_tier_keeps_id_order` :845 becomes the date-key test; seeds set `created_at` directly (:789, :1495).
- Guards that move with a new command/field: `tests/ipc_contract.rs`, `tests/ipc_argument_contract.rs` floor, `tests/config_invariants.rs` segment.rs census, `scripts/check-*` file floors.
- Tests that move: `src-tauri/tests/tm_contract.rs`, `config_invariants.rs`, `ipc_argument_contract.rs`, `ipc_contract.rs`; `tests/frontend/tmFuzzyStrip.test.ts`, `tests/frontend/tmFuzzyAdapters.test.ts`, `tests/frontend/tmConcordance.test.ts`.

## Tasks & Acceptance

**Execution:**
- [x] `src-tauri/src/core/tm/mod.rs` -- `created_at` on `TmPair`/`RawPair`, newest-first order inside `merge_tiers`, pure `distinct_exact_targets` (collapse by target).
- [x] `src-tauri/src/commands/segment.rs`, `src-tauri/src/lib.rs` -- exact list in the `tm_fuzzy_matches` response, `created_at` on the exact-list and concordance DTOs (not on fuzzy rows), `accept_tm_exact`; move the guards above to live counts.
- [x] `src-tauri/tests/tm_contract.rs` -- the I/O matrix through the wired commands, both tiers seeded with fixed dates.
- [x] `src/config/segment.ts`, `src/tmFuzzyStripState.ts`, `src/TmFuzzyStrip.vue`, `src/tmFuzzyCommandDeps.ts`, `src/panels/LookupPanel.vue`, `src/i18n/vi.json` -- list rendering, in-use mark, pick, dates.
- [x] `tests/frontend/` -- list shows all rows with dates, in-use mark, pick by key, prompt only on typed text, Concordance date.

**Acceptance Criteria:**
- Given the date-key test, when `merge_tiers` stops sorting by date, then the same-side-same-tier row goes red and the side/tier rows stay green.
- Given the pick test, when `accept_tm_exact` writes `other` instead of the pair's origin, then only the origin rows go red.
- Given the pick-over-pre-fill test, when the baseline exemption is removed, then that case goes red and the typed-text case stays green.
- Given the collapse test, when collapsing is removed, then the duplicate-target row goes red.
- Given a source with 2+ distinct targets, when 7.4 pre-fills, then the strip still lists every target (the pre-fill never hides the choice).

## Implementation Notes

- The date key lives in `merge_tiers`: each tier's already-filtered rows are sorted `created_at` desc then `id` desc before the stable side/tier merge; loads stay `ORDER BY id`. Sorting in SQL (`ORDER BY created_at DESC, id DESC` on the full-table load) cost +150–240 ms per fuzzy scan (release, 100k pairs per tier, 4 alternating runs: 513–565 ms vs 719–774 ms); in `merge_tiers` it is 506–568 ms.
- The exact list rides on `tm_fuzzy_matches` (`exact` field, filled only with 2+ distinct targets, then `matches` is empty); no second scan command. `accept_tm_exact` is the sibling of `accept_tm_fuzzy`, with its own write closure rather than `promote_ai_translation`, because it needs the pair's origin and the baseline exemption.
- `fill_exact_tm_matches` is unchanged; it now pre-fills the newest pair of the head side/tier (Q4).
- One strip component: `TmFuzzyStrip.vue` renders either list; the pending overwrite carries `kind` so the forced re-call goes to the same command.
- Guards moved to live counts: `REGISTERED_COMMAND_FLOOR` 104 to 105 (live count read with the floor at 9999), `config_invariants` segment.rs census 16 to 17 plain, tree 62 to 63.
- Counter-checks (real changes, restored): the two `sort_by` calls in `merge_tiers` removed made the 4 date-order cases red with side/tier green; the pick writing `other` turned only the origin case red; the baseline exemption removed turned the 3 no-prompt cases red with typed-text green; collapse removed turned 3 red; the webview exact routing removed turned the 4 exact-pick vitest cases red.
- Ledger: one new item (real-app pass of the list, Chủ: Epic 7).

## Spec Change Log

## Review Triage Log

- VG1 date key unasserted through `rank_concordance`/`rank_fuzzy_candidates` — medium: every date-order case goes through `pairs_for_source`; patch (two wired cases).
- VG2 digit beyond row count in fuzzy mode untested — low: guard `row >= tmFuzzyRowCount()` exists, no case; patch (one vitest case).
- VG3 exact-only eligibility untested — false: reverting `tmFuzzyIsEligible` reddens every exact component case (reviewer withdrew it).
- E1 collapse key is raw `target_text` (NFC/whitespace variants) — low: rare, normalisation adds complexity; rejected.
- E2/B2 `created_at` compared as strings — low: today every writer is `insert_pair`'s `strftime` format; TMX import could break it; defer (Chủ: Story 7.10).
- E3 work-tier pick needs the global store — false: the app always manages it; same precondition as `accept_tm_fuzzy`.
- E4 pair read outside the write transaction — false: `tm_unit` is append-only (spec Never); 7.9 owns its mutations.
- E5 overwrite wording from `isExactList` not pending `kind` — low: direct correction; patch.
- E6/B4 relative date labels not refreshed while the strip stays open — low: needs a timer; rejected.
- E7/B6 rows past 9 have no digit, list uncapped — low: Ice chose no cap (Q1); layout is in the real-app ledger item; rejected.
- E8 `config_invariants` message says 62/43 — low: direct correction; patch.
- B1 pick over a confirmed segment demotes to draft without asking — false: the confirmed text has a `segment_version` copy, same rule as `promote_ai_translation`; AD-31 edit-of-confirmed is unconfirmed.
- B3 untested error branches of `accept_tm_exact` — low: they mirror `accept_tm_fuzzy`; rejected.
- B5 in-use mark compares saved text, not the typing buffer — low: cosmetic until flush; rejected.
- B7 inline tier/side match beside new helpers, `tmFuzzyRowCount` wrapper — low: direct correction; patch.
- B8 story ids in test describe names — false: established naming, 52 frontend test files do it; the AGENTS.md ban sits under Code comments.
- B9 handoff file still says SQL date sort and red `ipc_argument_contract` — low: fixed in the handoff file.
- B10 no counter-check for in-use mark, concordance date, `isTmExactTarget` — low: rejected; concordance date case asserts the shipped field.
- B11 strip on every visit, newest pre-fill, latency not re-measured — false: Q1/Q3/Q4 are Ice's decisions; latency measured (Implementation Notes).

## Verification

**Commands:**
- `cargo test --test tm_contract` -- expected: green, including the new date-key and pick rows.
- `npm run test:story 7-8 -- --list`, then the listed vitest files -- expected: green.
