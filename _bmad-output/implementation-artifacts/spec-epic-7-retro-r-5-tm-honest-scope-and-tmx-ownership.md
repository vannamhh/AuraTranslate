---
title: 'Epic 7 retro R-5 — TM management tells what a filtered action leaves untouched; TMX import clamps future dates and asks who translated the file'
type: 'bugfix'
created: '2026-10-06'
status: 'done'
route: 'dispatch'
baseline_commit: '44dff171e9d1a66e895e623f41b401f5e280d2eb'
review_loop_iteration: 0
context:
  - '{project-root}/src-tauri/AGENTS.md'
  - '{project-root}/src/AGENTS.md'
---

<frozen-after-approval reason="human-owned intent — do not modify unless human renegotiates">

## Intent

**Problem:** (F2.3) Edit and delete in TM management act only on the copies the filters show (7-9 Q8), but the edit hint always says Global copies are overwritten, and a delete under a tier or origin filter leaves a hidden identical copy that 7-4 keeps filling in, with nothing on screen. (F2.5, F4.6) TMX import keeps any `created_at`, so a future date wins the AD-18 "newest" key forever, and it trusts `x-aura-origin=self` from anyone's file, putting someone else's text on the mine side with no way back.

**Approach:** Keep the 7-9 Q8 scope and make the screen say what it leaves untouched. TMX import clamps a future date to the import moment and counts it; the preview gets a "this file is my own translation" checkbox, off by default, which alone decides whether a pair may land as `self`.

**Decisions (Ice, 2026-10-06, retro Epic 7 Q-1…Q-5):**
- Q-1 ⇒ an edit keeps `created_at` (7-9 Q1 stands): an edit corrects text, it does not re-choose; re-confirming in the Editor is how a pair becomes newest.
- Q-2 ⇒ edit/delete keep the 7-9 Q8 scope (copies the filters show); the screen states how many identical copies stay.
- Q-3 ⇒ future dates are clamped, not rejected; ownership checkbox as below. Supersedes 7-10 Q1 for `self`-labelled pairs.
- Q-4 ⇒ restore then confirm stays `other` (AD-47 ⑤, AD-50). No code. Every version is born with a TM pair of the same text and origin, so an earlier `(X, self)` already sorts first.
- Q-5 ⇒ no per-row "mark as mine"; the Q-3 checkbox covers a user's own legacy TM at import.

## Boundaries & Constraints

**Always:**
- Hidden copies of a row = stored copies with the identical (source, target) in the loaded tiers (Work only when a Work is open), ignoring the tier, origin and search filters, minus the copies in the row. Computed in Rust, inside `rank_manage_listing`.
- Edit form: the base hint no longer mentions Global. A Global note appears only when the row has a Global copy, and a hidden-copies note appears only when the count is > 0. The delete confirm hint shows the same hidden-copies note.
- Ownership mapping happens at confirm. The plan keeps the label read from the file (`Option<PairOrigin>`, `None` when missing or unknown).
- Clamp inside the write transaction. One "now" is read once per transaction and used both as the default stamp and as the clamp bound; stored ISO strings compare lexicographically.
- The checkbox is unchecked every time a preview opens; it is not remembered.

**Never:** no migration, no new origin value (AD-47 ⑥), no relabelling of pairs already stored (the "already there" skip stays origin-blind), no change to edit/delete/push scope or to `created_at` on edit, no change to TMX export.

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected |
|---|---|---|
| Import, box off | file labels `self`, `other`, `bilingual_import`, none | `other`, `other`, `bilingual_import`, `other` |
| Import, box on | same file | `self`, `other`, `bilingual_import`, `self` |
| Future date | `x-aura-created-at` 2999 or `creationdate` 29990101T… | stored = import moment; summary `future_dated_count` = 1 |
| Past or missing date | 2015 date / none | kept / stamped now; not counted |
| Hidden by tier | Work + Global hold (S,X); filter `work` | row copies = 1, `hidden_copies` = 1; delete removes the Work copy only, and the note names 1 copy |
| Hidden by origin | (S,X,self) + (S,X,other); filter `self` | `hidden_copies` = 1 |
| Nothing hidden | filters `both`/`all` | `hidden_copies` = 0, no note |

</frozen-after-approval>

## Code Map

- `src-tauri/src/core/tm/mod.rs` -- `rank_manage_listing` (:566) builds `ManageRow { pair, copies }` (:531); add `hidden_copies` there. The snapshot already holds both tiers unfiltered.
- `src-tauri/src/core/tm/tmx.rs` -- `PlannedPair` (:375, `translation_origin` set at :444 with `.unwrap_or(Other)`), `created_at_for` (:395), `write_planned_pairs` (:487, INSERT with `COALESCE(?4, now)` at :505), `WriteOutcome` (:480).
- `src-tauri/src/commands/tm.rs` -- `TmGroupRowWire` (:61, built :204-213); `TmxImportSummaryWire` (:362); `tm_confirm_import` core (:578) and `tm_confirm_global_import` (:604), wire shell (:856). The shell gains `file_is_mine: bool` (camelCase `fileIsMine` on the wire).
- `src-tauri/tests/config_invariants.rs:1147` -- pins the shell prefix `pub fn tm_confirm_import<R: tauri::Runtime>(\n        app: tauri::AppHandle<R>`; keep `app` first.
- `src/config/tm.ts` -- `TmManageRow` (:12) with guards `isPairFields`/`isManageRow` (:149/:162); `TmxImportSummary` (:57), `isImportSummary` (:312), `tmConfirmImport` (:348).
- `src/tmImportState.ts` -- confirm call; it owns the checkbox ref (reset on open).
- `src/TmImportOverlay.vue` -- preview body (:100-140); replace `tm.import.origin_note` and add the checkbox above the counts.
- `src/TmManageOverlay.vue` -- `edit_hint` (:523), `deleteHintText` (:227), import-done line (:684-690).
- `src/i18n/vi.json` -- `tm.manage.edit_hint` (:959), `tm.import.origin_note`, `tm.exchange.import_done` (:1004). It is the only locale; `check:i18n` checks shape only.
- Tests: `src-tauri/tests/tm_contract.rs` `mod manage` (:2107), `tests/tmx_contract.rs` (e.g. :329 date stamp), `tests/tm_wire.rs`, `tests/ipc_argument_contract.rs` (reads `lib.rs`, no list to edit); vitest `tests/frontend/tmManage.test.ts`, `tmImport.test.ts`, `tmConfigGuards.test.ts`.

## Tasks & Acceptance

**Execution:**
- [x] `src-tauri/src/core/tm/mod.rs` -- add `hidden_copies: usize` to `ManageRow` and compute it -- Q-2 count.
- [x] `src-tauri/src/core/tm/tmx.rs` -- `PlannedPair` keeps the file label; add a pure `pair_origin_for(label, file_is_mine)`; `write_planned_pairs` takes `file_is_mine`, clamps, and returns `future_dated` in `WriteOutcome` -- Q-3.
- [x] `src-tauri/src/commands/tm.rs` -- add `hidden_copies` to the row wire, `future_dated_count` to the summary wire, and `file_is_mine` to both confirm paths and the shell -- wire.
- [x] `src/config/tm.ts` -- types, guards (non-negative integers), `tmConfirmImport(fileIsMine)` -- wire.
- [x] `src/tmImportState.ts`, `src/TmImportOverlay.vue` -- checkbox (keyboard reachable, labelled), passed on confirm -- Q-3.
- [x] `src/TmManageOverlay.vue`, `src/i18n/vi.json` -- conditional Global note in the edit form, hidden-copies note in edit and delete, future-date sentence after import when > 0, rewritten origin note -- honest screen.
- [x] Tests -- Rust: every matrix row through the wire commands; vitest: the checkbox value reaches the invoke, the notes show only when they apply, and the guards reject a missing new field.

**Acceptance Criteria:**
- Given a row with no Global copy, when the edit form opens, then no text mentions the Global tier.
- Given the checkbox was ticked and the preview is cancelled, when a new preview opens, then the checkbox is unchecked.
- Given a counter-check that maps `self` straight through in `pair_origin_for`, or removes the clamp, or returns `hidden_copies` = 0, then at least one named test goes red for that reason (AGENTS.md: a real removal).

## Implementation Notes

- Built in two phases (Rust, then webview); working notes in [epic-7-retro-r-5-phases-2026-10-06.md](epic-7-retro-r-5-phases-2026-10-06.md).
- `pair_origin_for` is the single place a file label becomes a stored origin; `PlannedPair.translation_origin` now holds the raw label (`Option`), resolved at confirm, so the preview counts are unchanged.
- `write_planned_pairs` reads "now" once per transaction; the old `COALESCE(?4, now)` in SQL is gone because the clamp needs the same value in Rust.
- `hidden_copies` is counted only for the at most 200 groups shipped, from the unfiltered snapshot; `keep` now borrows the snapshot and clones the rows it keeps.
- `tm_confirm_import` takes a required `fileIsMine`; `config_invariants` still matches because `app` stays first.
- `tm.import.origin_note` was reworded after phase 2: the first wording said every pair lands as "Người khác dịch" with the box off, which is false for `bilingual_import`.
- Counter-checks by real removal: `self` passthrough, clamp, `hidden_copies = 0`, each shell call site forwarding `false`, the confirm sending `false`, the edit-form hidden note; each turned its own test red.
- Real-app check is a ledger item `Chủ: Epic 7` (`deferred-work.md`, source_spec this file).

## Spec Change Log

## Review Triage Log

- VG1/BH4 no test passes `true` through `wire::tm_confirm_import`, so a shell forwarding `false` stays green -- medium; patch (two `tm_wire` shell tests, Global and Work, counter-checked).
- VG2 checkbox `:disabled` while confirming is untested -- low: the value is read once at call time; rejected.
- VG3 `resetTmImport` reset of the box untested -- low: every preview open resets it first; rejected.
- VG-o1/EC3/BH3c lexicographic date compare breaks on non-24-char dates -- false: `created_at_for` only yields `parse_iso_millis` (exact 24-char) or `iso_from_tmx_date` (`.000Z`) values.
- VG-o2 vitest guards never counter-checked -- low: two seams (confirm arg, edit hidden note) were removed for real and each turned its own test red; rejected for the rest.
- EC1 hidden count keyed by raw text while groups use `concordance_key` -- false: groups key on exact `source_text`; `concordance_key` only filters search.
- EC2 `saturating_sub` masks copies > stored -- false: copies come from the same snapshot rows, so stored >= copies always.
- EC4/BH2 bulk delete lacks a hidden-copies note -- false: bulk deletes by origin side in the tiers shown and its confirm already names the tier.
- EC5 box toggled between click and invoke -- false: the value is read synchronously in the call expression.
- EC6 required `fileIsMine` breaks a stale webview -- false: webview and Rust ship in one bundle.
- EC7 re-import with the box on keeps already-stored `other` pairs -- low: the frozen Never forbids relabelling stored pairs (Q-5); rejected.
- BH1 own backup restored with the box off lands as `other`, and the preview shows no `self` count -- low: Ice took this cost in Q-3 (cheap direction, recoverable); rejected.
- BH3 future-date test bounds are loose -- false: `future_dated_count == 2` separates clamped from dropped; an equal date keeps the same value.
- BH5 handoff contradicts itself; manual check has no ledger owner -- low; patch (handoff fixed; debt item `Chủ: Epic 7` added).
- BH6 `.cloned()` of filtered rows and a re-keyed map in `rank_manage_listing` -- low: a few MB per listing at 100k pairs beside a 208–597 ms measured listing; rejected.
- BH7 `editHasGlobal` duplicates `hasGlobal`; writable exported ref -- low: no caller diverges; `v-model` needs a writable ref; rejected.
- BH8 tests assert Vietnamese literals -- low: negative cases pair with positive ones in the same test; rejected.
- BH9 note does not name the hiding filter; import-done sentence spacing -- low: wording choice; Vue condenses the whitespace to one space; rejected.
- BH10 field doc comments restate names -- false: they state what counts (hidden by filters, replaced by the import moment), which the type does not.

## Verification

**Commands:**
- `npm run build` then `cargo test --test tm_contract --test tmx_contract --test tm_wire --test config_invariants --test ipc_argument_contract` (from `src-tauri/`) -- expected: green.
- `npx vitest run tests/frontend/tmManage.test.ts tests/frontend/tmImport.test.ts tests/frontend/tmConfigGuards.test.ts` -- expected: green.
- `npm run check:i18n && npm run check:tokens` -- expected: green.

**Manual checks:**
- In the real app, import a TMX holding a `self` pair with the box off, then filter TM management by `other`: the pair is there. Owner: Epic 7 real-use pass (R-11).
