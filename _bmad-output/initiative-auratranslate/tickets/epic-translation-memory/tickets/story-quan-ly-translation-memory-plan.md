---
ticket: 9
title: 'Story 7.9 — TM management: view, filter by origin and tier, edit, delete, push a Work pair up to Global'
type: 'feature'
created: '2026-10-03'
status: done
route: 'dispatch'
baseline_revision: '4f53b2a41e0ab69222fc5d6ad7489b82d2293a2b'
review_loop_iteration: 2
context:
  - '{project-root}/AGENTS.md'
  - '{project-root}/src/AGENTS.md'
  - '{project-root}/src-tauri/AGENTS.md'
  - '{project-root}/_bmad-output/initiative-auratranslate/archive-v6/epic-7-context.md'
---

<frozen-after-approval reason="human-owned intent — do not modify unless human renegotiates">

## Intent

**Problem:** TM fills itself on every confirm (7.1) and carries an origin per pair (7.2), but the translator cannot see the store, cannot clean out pairs that are not their style (R13, FR62), and the Global tier has no writer at all (spec 7.3: push-up is 7.9). `tm_unit` has no UPDATE/DELETE anywhere and no list-all command.

**Approach:** A keyboard-operable TM management overlay, modelled on the Glossary management overlay, over new commands that list both tiers with filters, edit and delete one pair, and push one Work pair to Global.

**Decisions (Ice, 2026-10-04):**
- Q1 ⇒ (A) an edit is an UPDATE in place: same id, same `created_at`, the old text is gone.
- Q2 ⇒ only `target_text` is editable (the source is the match key); an edited pair's origin becomes `self`.
- Q3 ⇒ push-up MOVES the pair (Work row deleted, origin kept), like `glossary_promote_term_to_global`; an identical (source, target) pair already in Global ⇒ refuse with a clear message, nothing written.
- Q5 ⇒ in scope from `tm-manage.html`: search over source and target, the TM health strip (count and % per origin), grouping of several targets under one source (FR63), bulk "delete every pair of others" (two presses).
- Q6 ⇒ the open command has no default binding (bindable in Settings). 🔵 2026-10-04 (review loop 2): the repo has no command palette, so Ice chose a title-bar button beside "Quản lý Glossary" (same pattern, `data-tm-manage-open` for focus return) as the way in; still no default chord.
- Q7 ⇒ Save in the edit form writes directly, no second press (Glossary precedent): the edit form is the explicit act. Delete, single or bulk, stays AD-49 class (iii).
- Q8 (review loop 1) ⇒ identical (source, target) pairs collapse into ONE list row regardless of tier and origin, the 7.8 rule; the row shows the tier and origin of its first copy in AD-18 order. Edit and delete act on every copy in the row (as the current filters show it); push moves one copy to Global and deletes every Work copy in the row. Ice kept the existing code and had it amended rather than reverted.
- Q4 ⇒ (A) after measuring both: Rust filters and returns at most 200 source groups plus totals (Design Notes); the overlay says "200 / N" and points to filter or search to narrow. No virtualized list.

## Boundaries & Constraints

**Always:**
- Each row shows source, target, origin (the three stored values, not only the mine/others side), tier and date (`historyTimeLabel`). Filters: origin, tier, search over source and target (FR62, epics §Story 7.9).
- Rows with the same `source_text` sit under one group header when the filtered list holds 2+ distinct targets for it (FR63). Within a group, copies with the same `target_text` are one row (Q8); collapsing is a read concern, stored rows are never merged.
- The health strip counts pairs per stored origin (with %) over the current tier filter, ignoring the origin filter and search.
- `tm_unit` joins AD-49 item 3 (user content), declared here. Delete, single or bulk, is class (iii): two presses with "không hoàn tác được", the Glossary pattern. Bulk delete removes every pair on the others side (`other` and `bilingual_import`) in the tiers the tier filter shows, and its confirm text states the count.
- Edit: `target_text` only, applied to every copy in the row; trimmed-empty text is refused; origin becomes `self`; ids and `created_at` unchanged; works in either tier.
- Push-up writes one copy to Global first, then deletes every Work copy in the row, so a crash between the two leaves a duplicate, never a loss. Refused (nothing written) when Global already holds the identical (source, target) pair, which includes a row that already has a Global copy.
- A deleted id is never reused (AUTOINCREMENT already guarantees it). No retire column, no migration.
- An edit or delete takes effect on the next exact, fuzzy or Concordance lookup (no TM cache exists in Rust; keep it that way).
- Writes go through each store's single writer; every write command re-reads the pair by tier and id and returns `tm.pair_not_found` when it is gone.
- With no Work open, the list shows Global only and push is unavailable.
- Search uses Concordance's substring rule (spec 7.7 R: NFC, English case-insensitive). Groups are ordered by their newest row, rows inside a group by AD-18.
- Every action has a keyboard path; the overlay traps Tab, Esc closes.
- Copy obeys `check:i18n` (no "bạn", no "chúng tôi"); the mockup text is rewritten.

**Never:**
- TMX import/export (7.10), Smart RAG (7.11), the mockup's escape hatches ("chỉ dùng cặp của tôi", "xuất riêng"), a new origin value (AD-47 ⑥), a "Chương · câu" column (AD-6 cuts the link to segments).
- Changing the confirm path's append-only write (AD-6 rule text stays as is).

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected |
|---|---|---|
| Filter | Work: 2 `self`, 1 `other`; Global: 1 `bilingual_import`; origin = others, tier = both | 2 rows; strip counts 2/1/1 |
| Cap | 250 distinct sources, no filter | 200 groups, total 250 |
| Group | `(S,A)`, `(S,B)`, `(S,A)` | one group, two target rows; the `A` row carries both copies |
| Edit | `other` pair, target → "X" | same id and date, target "X", origin `self`; next `tm_fuzzy_matches` sees "X" |
| Edit empty | target → "  " | refused, nothing written |
| Delete | first press | nothing written, confirm shown; second press deletes |
| Bulk delete | tier = work, 3 others-side pairs | after two presses, the 3 gone, `self` rows and Global untouched |
| Push | Work `(S,A)` `other`, Global empty | Global `(S,A)` `other`, Work row gone |
| Collapsed row | Work `(S,A)` ×2, Global `(S,A)` ×1 | one row; delete (two presses) removes all three; edit changes all three |
| Push collapsed | Work `(S,A)` ×2, Global empty | Global holds one `(S,A)`, both Work copies gone |
| Push duplicate | Global already `(S,A)` | refused, both tiers unchanged |
| Stale | pair deleted elsewhere, then edit/delete/push | `tm.pair_not_found`, nothing written |

</frozen-after-approval>

## Code Map

- `src-tauri/src/core/store/schema.rs:891-903` -- `TM_UNIT_DDL` (AUTOINCREMENT, `CHECK (target_text <> '')`, index on `source_text`), same in both stores; read only.
- `src-tauri/src/core/tm/mod.rs` -- `PairOrigin` :37, `insert_pair` :70 (doc says "never updated", keep for the confirm path), `TmTier` :85, `TmPair` :91, `load_all_pair_rows` :156 (private, unpaged), `merge_tiers` :211, `concordance_key` :358 (reuse for search), `pair_by_id` :388. Add pure list/filter, update, delete, push functions here (AD-15: no I/O outside the store calls already used).
- `src-tauri/src/commands/segment.rs:2600-2660` -- `accept_tm_fuzzy`/`accept_tm_exact`: tier string to store mapping and `tm.pair_not_found` (:2608); reuse the mapping. New commands go in a new `commands/tm.rs` (async for the list: it reads whole tiers).
- `src-tauri/src/commands/glossary.rs` -- analog: `glossary_list_entries` :599, `glossary_update_term` :186, `glossary_delete_term` :619, `glossary_promote_term_to_global` :648 (two separate writes; same crash window if push is a move), wire shells :1271-1301.
- `src-tauri/src/lib.rs` -- command registration.
- `src/GlossaryManageOverlay.vue`, `src/glossaryManageState.ts` -- overlay, listbox, keyboard (`onKeydown` :306-351), two-press delete (`deletePendingKey`), inline edit; copy the shape, do not share state.
- `src/App.vue:57,367` -- overlay mount. `src/commands/index.ts` -- `glossary.manage.open` :3338-3353 is the open-command pattern (optional deps port :916, wired in `src/main.ts:882-898`); sub-actions register with `keys: undefined`.
- `src/config/segment.ts:1131-1160` -- `TmFuzzyTier`, `TmFuzzySide`; put the new wrappers and wire guards in a new `src/config/tm.ts`.
- `src/i18n/vi.json:887-918` -- `tm.*`; reuse `tm.fuzzy.tier_*`; new `tm.manage.*` keys including three origin labels. `src/panels/segmentHistoryTime.ts:100` `historyTimeLabel`.

## Tasks & Acceptance

**Execution:**
- [x] `src-tauri/src/core/tm/mod.rs` -- list with origin and tier filters, edit, delete, push, all keyed by (tier, id).
- [x] `src-tauri/src/commands/tm.rs`, `src-tauri/src/lib.rs` -- wire shells, `tm.pair_not_found`.
- [x] `src-tauri/tests/tm_contract.rs` (or a new `tm_manage_contract.rs`) -- the I/O matrix through the wired commands, both tiers seeded with fixed dates; edit/delete seen by the next `tm_fuzzy_matches`/`tm_concordance`.
- [x] `src/config/tm.ts`, `src/tmManageState.ts`, `src/TmManageOverlay.vue`, `src/App.vue`, `src/commands/index.ts`, `src/main.ts`, `src/i18n/vi.json` -- overlay, filters, edit, two-press delete, push, open command.
- [x] `tests/frontend/tmManage.test.ts` -- keyboard paths, filters, two-press delete, push, stale pair.
- [x] Guards moving to live counts: `tests/ipc_argument_contract.rs` floor, `tests/config_invariants.rs` `COMMAND_FILE_CENSUS` (+1 row, length 16, `blocking_wire_cases` for the list), `tests/ipc_contract.rs` wire fields, `scripts/check-commands.mjs` `HANDLER_TABLE`, `tests/frontend/dialogAccessibleName.test.ts`.

**Acceptance Criteria:**
- Given the edit test, when the write keeps the old origin, then only the origin row goes red.
- Given the push test, when the Work delete is removed, then the push row goes red and the duplicate row stays green.
- Given the bulk-delete test, when it ignores the tier filter, then the Global-untouched assertion goes red.
- Given an edit or delete, when the next exact, fuzzy or Concordance lookup runs, then it reflects the change (asserted through the wired commands).
- Given the overlay, when only the keyboard is used, then filter, search, move, edit, save, delete (both presses), bulk delete and push all work.

## Implementation Notes

- Commands live in a new `commands/tm.rs`: `tm_list_pairs` and `tm_delete_others` are `(async)`; `tm_update_pair_target`, `tm_delete_pair`, `tm_push_pair_to_global` take the row's `copies` plus `sourceText`/`expectedTarget`, re-read each copy and act only on copies whose source and target still match. New codes: `tm.target_empty`, `tm.global_pair_exists`, `tm.invalid_filter`.
- Q8 collapse is a read concern in `rank_manage_listing`: one row per (source, target) inside a group, carrying every copy in AD-18 order; health and `total_pairs` still count stored rows.
- Push writes one Global copy (the first AD-18 copy's origin and `created_at`) in one Global transaction with the duplicate check, then deletes the Work copies.
- An edit whose text equals every copy's target writes nothing and keeps the origin (B1).
- Webview: search re-lists 150 ms after the last keystroke; an edit pins its row against re-lists; every write failure re-lists; handlers do nothing while the overlay is closed; a title-bar button is the way in (Q6 corrected, no chord); handlers come from `tmManageCommandDeps()`.
- `src/config/tm.ts::malformed` takes the command name second: `ipc_argument_contract` reads any call with a `CMD_*` constant first as an `invoke` and panicked on `malformed(CMD_…, value)`.
- Guards moved to live counts: `REGISTERED_COMMAND_FLOOR` 105 to 110, `COMMAND_FILE_CENSUS` gains `tm.rs` (3 plain, 2 async; tree 63/43 to 66/45), `check-commands` floors (`HANDLER_ATTR_FLOOR` 78 to 84 among them) and `HANDLER_TABLE`, `check-i18n` `VUE_FLOOR`.
- Counter-checks were real removals, restored; each turned only its own case red (listed per loop in `7-9-phases-2026-10-04.md`). Two pairs guard each other and go red only together: the source/target re-read in `live_copies` with the `target_text` condition on `DELETE`, and the pinned row in save with the pin branch in `loadRows`.
- Ledger: four new items (real-app pass, `main.ts` wiring, bindable overlay sub-commands, `tm_list_pairs` lock release), all Chủ: Epic 7.

## Spec Change Log

- Loop 1 (VG other finding: the Group matrix row said "two target rows" while the code showed every stored row). Ice resolved the intent gap as Q8 (collapse identical (source, target) across tiers; edit/delete act on every copy in the row; push moves one copy and deletes every Work copy) and chose to amend the existing code instead of reverting it. Amended: frozen Decisions (Q8), Always (group, edit, push lines), matrix rows Group, Collapsed row, Push collapsed. Known-bad state avoided: a management row that edits or deletes one hidden copy while identical copies stay. KEEP: the snapshot-then-filter list path and its lock release, the 200-group cap, the write-Global-first push order, per-origin health counts over the tier filter, the two-press delete state machine, `malformed(value, command)` argument order.

## Review Triage Log

Loop 3:
- L3-BH1 the nine `tm.manage.*` sub-commands are bindable in Settings but a binding never fires (blocked while the overlay is open, no-op while closed) -- medium, pre-existing: every overlay's sub-commands share it (Settings lists the whole registry by design, `isBlocked` blocks while open); defer (ledger).
- L3-BH2 `tm_delete_others` holds `OpenWorkState` while deleting from Global -- low: a rare explicit action; rejected.
- L3-BH3 single-row delete and edit say nothing about a Global copy shared by every Work, and the edit hint does not say every copy is rewritten -- low: direct copy correction; patch.
- L3-BH4 story ids in the `tmManageState.ts` and `tmManage.test.ts` headers; the new `App.vue` comment restates the button -- low: AGENTS.md comment rules; patch.
- L3-BH5 `total_pairs` is sent but not shown -- low: harmless field; rejected.
- L3-BH6 the wire-key list is asserted in both `tm_contract.rs` and `ipc_contract.rs` -- low: duplication only; rejected.
- L3-BH7 options in a multi-target group lose the source text for screen readers (header is `role="presentation"`) -- medium: each option is read as its target alone; patch.
- L3-BH8 stale-push cases send empty source/target, so they cannot fail for the claimed reason -- medium: no case pushes listed copies deleted elsewhere; patch.
- L3-BH9 collapsed-push fixture cannot tell "first AD-18 copy" from "newest" or "any mine copy" and does not assert the kept `created_at` -- medium; patch.
- L3-BH10 push core takes `Option<&Store>` it never gets; Global-only row with no Work answers `work.none_open` -- low: the button is disabled on Global rows; rejected.
- L3-BH11 blank-target check runs after store resolution -- low: only the error code order differs; rejected.
- L3-BH12 `editPin` stays set after a stale save -- low: direct correction; patch.
- L3-BH13 group-order tie compares ids from two databases -- low: same `created_at` to the millisecond across tiers; rejected.
- L3-BH14 relative dates go stale while open; pending-delete border invisible on the last row -- carried 7.8 E6 / low cosmetic; rejected.
- L3-ECH1 the pinned row's copies can change during an edit -- low: re-lists during an edit are now cancelled or pinned and the modal blocks confirms; rejected.
- L3-ECH2/ECH3 `saving` clears before the follow-up re-list; push notice can read a superseded re-list -- low: a stale write answers `tm.pair_not_found` and re-lists; rejected.
- L3-ECH4/ECH5 partial bulk or push failure -- carried L2-ECH2 / B10 (rejected beyond the re-list).
- L3-ECH6 trim/NFC of the edited target -- carried B12.
- L3-ECH7 edited row leaving the filter -- carried E9.
- L3-ECH8 an unchanged save keeps the others-side origin despite the hint -- false: B1 decided an unchanged save is not an edit.
- L3-VG1 keys on a focused action button are untested against the row shortcuts -- medium: removing the button exemption stays green; patch.
- L3-VG2 the no-Work rendering (push disabled, Work option disabled, note) is unasserted -- medium; patch.
- L3-VG3 the Tab trap has no behavioural case -- medium; patch.
- L3-VG4 `tm_list_pairs` releasing the lock before scoring is unguarded -- medium: needs a concurrency probe, same gap as `tm_concordance`; defer (ledger).

Loop 2:
- L2-BH1/VG no way to open the overlay: no palette exists, no opener in `App.vue` -- intent_gap: Q6 rested on a false premise; resolved by Ice (title-bar button, Q6 updated).
- L2-BH2 sub-commands are bindable in Settings and run while the overlay is closed; `tm.manage.delete_others` pressed twice deletes with no confirm on screen -- high: `shortcutsState.ts:139` lists every command, the state handlers never check `overlayOpen`, `closeTmManage` keeps the listing; patch (handlers no-op while closed).
- L2-ECH1 a re-list (debounced search or refilter) resolving after edit began moves the cursor, so Save writes the typed text onto another pair -- high: the expected target is re-read from the new row, so the server check passes; patch (edit pins its row key, begin-edit cancels the pending search).
- L2-ECH4/BH3/VG3 push with tier = work: the moved pair is not listed, the cursor lands on another row and the notice says it sits on the moved pair -- medium: patch (separate notice when the moved pair is not listed, plus a case).
- L2-ECH5/BH3b edited row leaving the filter vanishes silently -- carried E9 (rejected).
- L2-ECH6/BH4 delete hint names only Backspace (Delete also confirms, a focused button ignores both), one-copy rows read "1 bản lưu … cả 1 bản"; bulk-done names "tầng Tác phẩm này" when tier = global or no Work is open -- low: direct copy corrections; patch.
- L2-ECH7/BH7a listbox out of the Tab cycle -- carried B5 (rejected).
- L2-BH7b rows have no click target -- low: Prev/Next and the keyboard reach every row, Glossary rows behave the same; rejected.
- L2-BH7c the current row is never scrolled into view in the 50vh list -- medium: arrow navigation past the fold hides the cursor row; patch (`scrollIntoView({ block: 'nearest' })`).
- L2-ECH2/ECH8/ECH9/BH5 after a write fails with any code but `tm.pair_not_found`, the list stays stale even when part of the write landed -- low: direct correction (re-list after every write failure); patch.
- L2-BH6 row writes use one transaction per copy, not per store -- low: needs a store failure mid-row; rejected.
- L2-ECH3 pushing a mixed-origin row keeps only the first copy's origin -- false: Q8 moves one copy with the first copy's origin by decision.
- L2-BH8 row shows every copy's tier but only the first origin -- false: the first copy's tier and origin are shown as Q8 says; the extra tier badges explain why push is refused.
- L2-ECH10 a write with no IPC bridge shows nothing -- low: only outside Tauri; rejected.
- L2-ECH11 the open Work changes while the overlay holds its copy ids -- low: the modal scrim and `isBlocked` keep Work switching out of reach while it is open; rejected.
- L2-ECH12 others-side count changes between the two bulk presses -- low: needs a concurrent write in the same second; rejected.
- L2-BH re-list shows no busy state while the old rows stay actionable -- low: the highlighted row is the one acted on; rejected.
- L2-BH9a Implementation Notes say the tree census moved to 67/44, the code asserts 66/45 -- low: corrected in Implementation Notes.
- L2-BH9b `config_invariants` assert message keeps the 63/43 parenthetical -- low: direct correction; patch.
- L2-BH9c `ipc_contract.rs` TM case builds structs by hand -- false: that is the file's pattern (`serde_json::to_value` of a built struct pins the wire keys); a renamed field turns it red.
- L2-BH9d "Story 7.9:" in that test's doc comment -- low: AGENTS.md bans story ids in code; patch.
- L2-BH10 `focusInitialTarget` and `focusSettledTarget` are identical -- low: direct deletion; patch.
- L2-BH11 per-row `String` clone in the group sort -- carried B16 (rejected).
- L2-BH12 unknown tier or empty copy list answers `tm.pair_not_found` -- carried B7 (rejected).
- L2-VG1 update/delete of a row with a Work copy and no Work open has no case -- medium: removing `work_store_for` stays green; patch.
- L2-VG2 `main.ts` deps spread and `isBlocked` line unverified -- medium, pre-existing harness gap shared by every overlay: defer (ledger).

Loop 1:

- B1/E7 saving an unchanged target relabels an `other`/`bilingual_import` pair as `self` -- medium: `update_pair_target` always writes `self`; Enter then Enter changes the origin with no text change; patch (unchanged text is a no-op).
- B2/E6 search re-lists the whole TM on every keystroke -- medium: `setTmManageSearch` calls `loadRows` with no debounce, each call reads both tiers under `OpenWorkState`; patch (debounce like the 7.5 strip).
- B3 listing reads Global under the Work lock and reads Work when tier = global -- low: `tm_empty` needs both tiers; same lock pattern as `tm_concordance`, whose cost is already a ledger item (Chủ: Epic 7); rejected.
- B4/E1/E2/E3 keyboard edit: focus stays on the list, Backspace twice deletes the row being edited, Enter resets the draft, arrows discard it, Esc closes the overlay with the edit open, focus falls to body after save -- high: `beginTmManageEdit` moves no focus and `deleteTmManagePair`/`moveTo` never check `editing`; patch.
- B5 the listbox is out of the Tab cycle -- low: Prev/Next/Edit/Delete buttons are Tab stops, so every action keeps a keyboard path; `tabindex="-1"` on the listbox is Ice's Glossary decision (`GlossaryManageOverlay.vue` cụm F 7); rejected, left to the real-app ledger item.
- B6 tier = work with no Work open lists nothing silently -- low: the select disables Work after load, but the wire answers an empty list where `tm_delete_others` answers `work.none_open`; patch (same error).
- B7 unknown tier string answers `tm.pair_not_found` in write commands -- low: only a caller bug reaches it, the webview types restrict tier; rejected.
- B8 `tm_delete_others` is a plain (main-thread) command and its census comment is wrong -- medium: it can delete ~200k rows in two stores; patch (`(async)`, census and comment).
- B9 bulk-delete copy does not name the tiers or that Global is shared by every Work -- medium: with tier = both it removes Global others-side pairs under "ở tầng đang lọc"; patch (name the tiers).
- B10/E8 push half-done when the Work delete fails after the Global commit; retry says "chưa ghi gì" -- low: needs a Work store write failure; the spec chose duplicate over loss; rejected.
- B11 after a push the cursor points at a neighbour and no notice shows -- low: direct correction (follow the moved pair's key, show a notice); patch.
- B12 edited target and duplicate check use raw text (no NFC/trim) -- low: rare, normalisation adds complexity; rejected (same as 7.8 E1).
- B13a/VG1 health ignoring search is unasserted, and the bulk confirm count reads it -- medium: no test lists with a search and reads `health`; patch.
- B13b header loss when a filter narrows a group to one target -- low: untested branch of an asserted rule; rejected.
- B13c cap keeps the newest groups is unasserted -- low: counts asserted, order rule asserted elsewhere; rejected.
- B13d/E10 exact lookup after edit or delete is unasserted -- medium: AC names exact, fuzzy and Concordance; only fuzzy and Concordance are asserted; patch.
- B13e lookup after push untested -- low: tier move asserted directly; rejected.
- B14/E11 task line names `tests/ipc_contract.rs` but the diff never touches it -- low: the wire-shape test sits in `tm_contract.rs`; patch (add the TM wire structs to `ipc_contract.rs`).
- B15a "Của tôi" and "Tự dịch" origin options return the same rows -- low: direct deletion of the redundant `mine` option; patch.
- B15b health percentages round to 99% -- low: cosmetic; rejected.
- B15c `tm.manage.pair_gone` duplicates `tm.fuzzy.pair_gone` -- low: direct correction; patch.
- B16 group sort clones strings per row; real latency unmeasured -- low: covered by the real-app ledger item; rejected.
- E4 a failed re-list keeps the old listing, so actions hit invisible rows -- low: direct correction (clear the listing on failure); patch.
- E5 search box accepts typing while a write is saving but the filter ignores it -- low: direct correction (disable while saving); patch.
- E9 an edited pair leaving the current filter vanishes without notice -- low: the filters' intended effect; rejected.
- E12 keyboard-only test drives edit by `setValue` and bulk/push by click -- medium: same root as B4; patch (with B4).
- VG2 group-order test passes under a first-row or oldest-row key -- medium: tie broken by `source_text`; patch (fixture where alphabetical and newest order disagree).
- VG3 `tm_delete_others("both")` with no Work open has no case -- medium: the default filter path when only Global exists; patch.
- VG4 `config/tm.ts` wire guards never run -- medium: both frontend tests mock the module; patch (guard test on the `glossaryConfigGuards` pattern).
- VG5 focus (open, empty fallback, return, edit field) untested -- medium: no `activeElement` assertion; patch (with B4).
- VG6 held Backspace (`event.repeat`) unguarded -- medium: removing the line leaves every case green; patch.
- VG7 Esc inside the edit form untested -- medium: dropping `.stop` stays green; patch (with B4).
- VG8 date column unasserted despite the test name -- low: direct correction; patch.
- VG9 TM handlers inlined in `main.ts`, tests wire their own copy -- medium: production wiring is unguarded; patch (`tmManageCommandDeps()` on the 7.5/7.7 pattern).
- VG other: Group matrix row vs code -- intent_gap: resolved by Ice as Q8 (Spec Change Log, loop 1).
- VG other: focus lost after saving an edit -- high: same root as B4; patch.

## Design Notes

Measured 2026-10-04 (release, commit `4f53b2a41e0ab69222fc5d6ad7489b82d2293a2b`, 100,000 pairs per tier, load 5.3 to 3.8, 5 alternating rounds): A, both tiers, no search: 493–597 ms, ~95 KB; search ~1%: 208–243 ms; tier = work through SQL: 236–320 ms (SQL beats load-then-filter when the set narrows; the both-tier `COUNT(DISTINCT)` in SQL took 746–882 ms). The probe's sort-and-truncate stage (175–197 ms) cloned whole rows into ~110k groups and was untuned. B was 58 MB per open and was rejected.

## Verification

**Commands:**
- `cargo test --test tm_contract` -- expected: green.
- `npm run test:story 7-9 -- --list`, then the listed vitest files -- expected: green.
- Full suite once (`lib.rs` registration changes).

## Acceptance criteria from epics.md

Source: `epics.md` §Story 7.9 (v6, nay ở archive-v6).

**Covers:** FR62

As a người dịch,
I want rà lại kho của mình và dọn phần không phải văn phong của tôi,
So that tôi kiểm soát được thứ AI đang học từ tôi.

**Acceptance Criteria:**

**Given** màn hình quản lý TM
**When** mở
**Then** xem được từng mục TM

**Given** một mục TM
**When** người dùng sửa
**Then** thay đổi lưu và có hiệu lực ở lần khớp kế tiếp

**Given** một mục TM
**When** người dùng xoá
**Then** mục biến khỏi kho

**Given** danh sách TM
**When** hiển thị
**Then** hiện **xuất xứ** của từng cặp

**Given** danh sách TM
**When** lọc
**Then** **lọc được theo xuất xứ** — để rà lại hoặc dọn sạch phần không phải văn phong của mình

**Given** danh sách TM
**When** lọc
**Then** lọc được theo tầng — TM Tác phẩm hay TM toàn cục

**Given** một cặp ở tầng Tác phẩm
**When** người dùng muốn đẩy lên tầng toàn cục
**Then** làm được bằng một thao tác chủ động

**Given** mọi thao tác trên màn hình này
**When** thực hiện
**Then** làm được bằng bàn phím
