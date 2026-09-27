---
title: 'Story 11.4 — Glossary tells the truth about what it counted, scanned and refused'
type: 'chore'
created: '2026-09-26'
status: 'done'
baseline_commit: '3dd4a5e863a796471e497baf91d227afd976cc8f'
route: 'dispatch'
review_loop_iteration: 0
context:
  - '{project-root}/src-tauri/AGENTS.md'
  - '{project-root}/src/AGENTS.md'
  - '{project-root}/_bmad-output/implementation-artifacts/epic-11-context.md'
---

<frozen-after-approval reason="human-owned intent — do not modify unless human renegotiates">

## Intent

**Problem:** On HEAD `3dd4a5e`, 41 `deferred-work.md` items end in `Chủ: Story 11.4` (32 open, 9 🟡). The Task 0 re-read is in `11-4-task0-2026-09-26.md`. Six of Ice's 2026-09-24 decisions (#53, #55, #56, #57, #59, #60) are signed but not in the code. The has-Work-tier invariant is guarded only by `debug_assert_eq!`, so a release build does not check it. An already-decided candidate surfaces as a generic write failure. A manual entry leaves its pending candidate stuck. Zero-width characters get into manual entries. The import scan fails with only `eprintln!`. The export-cancel branch cannot be tested.

**Approach:** One spec for all 41 items, no lots. Each item gets one Epic 11 disposition, as listed below. Every fix gets a guard, and each guard is counter-checked by really removing its seam.

## Boundaries & Constraints

**Always:** Ledger items keep their text. Only `→` lines are appended, and a stale claim gets a 🔵 fix in place. A new column that can be unknown is nullable. Migrations are twin steps (project 25 / global 10) sharing one const, one step per DB for the whole story. A wire-shape change updates `ipc_contract.rs`. A check only a person can make in the real app goes to `Chủ: Epic 11`, and a Windows-only check goes to `B7`.

**Never:** Code a fix for L7052 (cross-store snapshot; it goes to Winston). Edit `GLOSSARY_ENTRY_DDL` in place. Add a gate or a dependency. Build the four mockup capabilities that #56 rejected. Add a control whose only purpose is to avoid `check:commands`.

## Dispositions

Agent, 2026-09-26 (from Task 0; Ice may override any line at approval):
1. ✅ fix, Rust: L4966 + L5315 (one type built only from `OpenWork` replaces the `(resolver, work)` pair at all four `debug_assert_eq!` sites, so the pair cannot disagree in release), L5134 (assert `GLOBAL_MIGRATIONS[2].sql == PINNED_ENTRY_DDL`), L5265 (a `StoreError` variant for a business-rule rejection with its own `MessageKey`/code; already-decided candidates map to it), L5336 (a manual add resolves the open Work's pending candidate with the same `source_term`, in the same write), L6423 (`insert_manual_entry` strips the five `Cf` code points through `strip_zero_width`; the twin migration adds triggers that reject them), L6641 (pull the scan threshold out of the thread closure into a pure fn), L6247 (split `glossary_export_tier` into the pure and wire layers; a test for cancel = `Ok(None)` and no file written), L6039 (#56: chapter-span count on the pending queue; the other four capabilities `KHÔNG LÀM`), L7003 (#60: drop `{size}`).
2. ✅ fix, webview: L5366 (#53: quick-add defaults to Work once, on the first lookup that reports a Work tier; later lookups never switch tier), L6003 + L6020 (#55, same commit: a source toggle refreshes marks from `editorPanelState.ts`; the confirm strip keeps typed text when only the suggestion changed), L6055 chips (real radios, `GlossaryQuickAdd.vue` precedent), L6070 (an already-decided row sets `actionError`; one static line says a category is not kept across close), L6134 (#59: search and filters are disabled while editing), L12082 (assert against literal strings from `vi.json`).
3. ✅ self-closed with a pointer: L6118 (`3be0f5f`, two-beat confirm with AD-49 wording), L6462 C1/C3/C5.
4. `KHÔNG LÀM` with a reopen condition: L5145, L5165, L5236, L5252, L5888, L6845, L6690 (a)(b).
5. Reassign: L470, L5641, L5657, L6410, L6894, L7136, L6690 e2e trio → `Chủ: Epic 11`. L6207 → `Chủ: B7` (its cancel half is L6247). L7052, L6462 C4, L6690 (c) → `Chủ: Winston` (one AD for cross-store reads).

Ice, 2026-09-26:
6. L5352 (option B): `KHÔNG LÀM`. Reopen when a per-keystroke caller goes through the Glossary write shells.
7. L5810 (option A): the six infrastructure-failure branches of `spawn_import_scan` emit a `scan_failed` outcome on `GLOSSARY_IMPORT_SCAN_EVENT`, and no UI listens to it. L5833: `KHÔNG LÀM`. Reopen when a user finds candidates missing after a Work switch, or when `src/` listens to the scan event.
8. L6103 + L5920 (option A): a nullable `occurrence_count` on `glossary_entry`, added in the twin step. `approve_candidate` seeds it from the candidate, and after that it never changes. Manual, imported and Global rows are `NULL`, and promotion to Global writes `NULL`. `NULL` shows as "—". The manage list gets a "Dùng" column and a sort by frequency with `NULL` last. The confirm strip shows the count when it is not `NULL` (`GlossaryMark` gains the field).
9. L6055 rows (option A): row selection is data (AD-34 §1). A row binds the cursor with `v-model` as listbox selection, and the keyboard and the mouse call the same setter.
10. L6087 (option A): `KHÔNG LÀM`. Delete and re-add is the path. L6462 C2: `KHÔNG LÀM`. Reopen both when a story adds `UPDATE … SET source_term`.
11. L6690 ⑥ (option A): 12px is re-signed. The 2026-08-25 decision text gets a 🔵 pointing to `a9464f4`, and the drift trap is self-closed.
12. L6982 (option C, amended by Ice 2026-09-27): `KHÔNG LÀM`. Reopen when the main thread is measured stalling on `OpenWorkState` behind a Glossary command. The guard cannot be released during the core call: `OpenWork` owns a non-`Clone` `Store` (`commands/project/mod.rs:56`), and an `Arc<Store>` would change when the old store closes on a Work switch. The count at HEAD: 7 `(async)` wrappers of 15 commands, and only `glossary_confirm_import` writes.

</frozen-after-approval>

## Code Map

- `src-tauri/src/core/glossary/store.rs` -- `debug_assert_eq!` at :767, :835, :976, :1341. `insert_manual_entry` ~:141/:180. `add_manual_term` ~:874. `update_manual_term` ~:910. `promote_to_global` ~:1076. Leave `load_tier`'s two-read shape alone (L7052).
- `src-tauri/src/core/glossary/candidate_store.rs` -- `approve_candidate` ~:284, `already_decided_error` ~:351. Keep `decode_*`'s `FromSqlConversionFailure`: that one really is corruption.
- `src-tauri/src/core/store/mod.rs` -- `StoreError` ~:363, `message_key` ~:446, `code` ~:462, `IpcError` conversion ~:533. Find the rusqlite→`StoreError` mapping in the writer.
- `src-tauri/src/core/store/schema.rs` -- `PROJECT_MIGRATIONS` ends at 24 (~:2066), `GLOBAL_MIGRATIONS` at 9 (~:708). Copy the `char(N)` style from :309-323.
- `src-tauri/src/core/glossary/exchange.rs` -- `strip_zero_width` :509 (widen its visibility).
- `src-tauri/src/commands/project/mod.rs` -- `spawn_import_scan` ~:2005. The threshold is in the closure ~:2060. The event consts are at ~:1775.
- `src-tauri/src/commands/glossary.rs` -- `glossary_export_tier` ~:1266-1345. `glossary_confirm_import` keeps `pending.lock()` across its write (~:848): that lock is what keeps cancel from racing the commit.
- `src-tauri/src/core/glossary/entry.rs` -- `GlossaryEntry` ~:209, `GlossaryMark` ~:265 gain `occurrence_count: Option<i64>` (Decision 8). `candidate.rs:126` is the source value.
- `src-tauri/src/core/i18n/mod.rs:192`, `src/i18n/vi.json:17` -- L7003.
- `src-tauri/tests/{pinned_contract,glossary_import_dialog_contract,glossary_exchange_contract,scope_contract,ipc_contract}.rs` -- fixture helpers `temp_dir`/`write_file`/`open_global`.
- `src/glossaryQuickAddState.ts` -- `'global'` at :249/:381. `quickAddWorkTierBlocked` and its no-autoswitch doc stay.
- `src/panels/{editorPanelState,glossaryMarksState,dictSourcesState}.ts` -- the call-site doc in `glossaryMarksState.ts` ~:17-24 gets call site #4. `src/glossaryConfirmStripState.ts` -- `targetsEqual` ~:58, `applyTarget` ~:175.
- `src/GlossaryQueueOverlay.vue` `.gq-chip` ~:273, `src/glossaryQueueState.ts` accept/reject early returns. `src/GlossaryManageOverlay.vue` search `<input>` :376.
- `tests/frontend/glossaryQuickAddStrip.test.ts:114,156`, `glossaryConfirmStripTemplate.test.ts:243` -- literal pattern from `aiTranslate.test.ts`.

## Tasks & Acceptance

**Execution:**
- [x] `core/glossary/*`, `core/store/{mod,schema}.rs`, `commands/{glossary,project/mod}.rs`, `core/i18n`, `src-tauri/tests/*` -- Disposition 1 and Decisions 7, 8 (schema, seed, wire) -- Rust phase.
- [x] `src/glossary*State.ts`, `src/panels/*`, `Glossary*Overlay.vue`, `vi.json`, `tests/frontend/*` -- Disposition 2 and Decisions 8 (column, sort, strip) and 9 -- Webview phase.
- [x] A real-removal counter-check for every new guard; full suite once (migrations are shared wiring) -- Tests phase.
- [x] `deferred-work.md` -- one `→` disposition for each of the 41 items, plus the 🔵 on the 2026-08-25 11px decision text (Decision 11).

**Acceptance Criteria:**
- Given the story is done, when `npm run check:debt-owner` runs, then it is green and no open or 🟡 item ends in `Chủ: Story 11.4`.
- Given a release-profile build, when a Glossary call is given a resolver and a Work from different sources, then it cannot compile or it fails loudly. It never returns entries silently.
- Given a pending candidate, when the same `source_term` is added manually, then the queue no longer lists it.
- Given a manual term whose only content is zero-width characters, when it is saved through the command or through raw SQL, then it is rejected.
- Given an open Work, when quick-add opens and its first lookup reports a Work tier, then the tier is Work, and a later lookup does not change it.
- Given text typed in the confirm strip, when a dictionary source is toggled and the target is unchanged, then the typed text survives and the marks redraw.
- Given an entry approved from a candidate and one added manually, when the manage list is sorted by frequency, then the approved one shows its count, the manual one shows "—", and `NULL` sorts last.
- Given each new guard, when its seam is really removed, then it goes red for that reason.

## Implementation Notes

- Decision 12 could not be built as first signed: `OpenWork` owns a non-`Clone` `Store`, so the `OpenWorkState` guard cannot be dropped while a `&Store` is in use. Ice amended it to option C (see Spec Change Log).
- `WorkContext::new` returns `Result`; a resolver without a Work tier paired with a store fails with `GlossaryError::WorkContextMismatch` (reuses the `glossary.scope_error` key). `marks_for_source_text`/`confirmed_terms_for_injection` keep their loose pair because `ai_boundary.rs` locks the five names, and they check `has_work_tier() == work.is_some()` themselves. `Store` carries no Work id, so a cross-Work pair is not detectable; it is unreachable because `OpenWorkState` holds one `OpenWork`.
- L5265 routes by type: `already_decided_error` boxes a `BusinessRuleConflict` inside `FromSqlConversionFailure`, and `WriteTicket::wait` downcasts it to `StoreError::Conflict`, leaving the corruption decoders on `WriteFailed`.
- `add_manual_term`: at the Work tier the insert and the candidate resolve share one transaction; at the Global tier the open Work's candidate is resolved in a second write, because the rows live in two databases.
- One twin migration step (project 25 / global 10) adds nullable `glossary_entry.occurrence_count` and two zero-width triggers; SQLite cannot alter a `CHECK`, and `GLOSSARY_ENTRY_DDL` stays untouched.
- `chapter_span_count` is computed at read time through the shared matcher over the whole Work; its cost is unmeasured and deferred to `Chủ: Epic 11`.
- Decision 7's six emit sites need a live `AppHandle`; they are guarded by a source scan over code lines only (comments excluded), plus a pure test of the event shape.
- The first full suite caught two misses from the Rust phase: `store_contract.rs` still expected 9 global steps, and a literal `。` in a new fixture broke `segment_boundary.rs`.
- The first Tests pass found 13 fixes whose seam could be removed without any test going red; each now has a test through its real caller with a recorded counter-check. That pass also found `setGlossaryManageSearch` missing its edit guard.
- Ledger: three Story 11.2 items (the `workflow_dispatch` evidence runs) were reassigned to `Chủ: Epic 11`. `3dd4a5e` marked 11-2 `done` while they were open, and `check:debt-owner` would otherwise stay red.

## Spec Change Log

- 2026-09-27, Rust phase. Trigger: Decision 12 (option A) could not be built without an `Arc<Store>` ownership change, and the planning count (9 of 21) had included doc-comment lines. Amended by Ice: option C, `KHÔNG LÀM` with a reopen condition, and the lock-contention AC was dropped. KEEP: every other Rust-phase change.

## Review Triage Log

- verification-gap: `glossary_pending_candidates`'s keyed `chapter_span_count` join has no test through the real command with ≥2 candidates — medium, patch: filed with evidence; a positional-zip regression would attach counts to the wrong row unseen.
- verification-gap (other): `update_manual_term` only `trim()`s, so an edited translation/note keeps embedded zero-width characters that `insert_manual_entry` now strips — low, patch: direct correction, same helper.
- blind: `## Implementation Notes` is empty — rejected: the fix edits this build's spec.
- blind: the L6690 ledger line says no new Ice signature records the 12px reversal — low, patch: Ice re-signed 12px in Decision 11 (2026-09-26), so the ledger sentence is now false.
- blind: a Global-tier manual add while a Work is open leaves the Work's same-term pending candidate stuck — medium, patch: `add_manual_term` gates the resolve on `tier == Work` (store.rs:948), and Disposition 1 names no tier.
- blind: toggling sort-by-frequency while not editing resets the cursor to row 0 — low, patch: the function already locates the current row for the editing branch; apply it in both.
- blind: `applyTarget`'s "unedited" test cannot tell a user who retyped exactly the old suggestion — rejected: low, and telling them apart needs a dirty flag; the text overwritten equals the old suggestion.
- blind: `candidate_chapter_span_counts` scans every Chapter's text on each queue open, unmeasured — maybe-false (medium if true), defer: settle by timing the command on the largest real Work.
- blind: the `debug_assert_eq!` on `chapter_span_counts.len()` reintroduces a debug-only guard, so a release mismatch degrades to `0` — false: `candidate_chapter_span_counts` builds one entry per input term (`seen` is sized from `terms`), so the lengths cannot differ.
- edge: in release a length mismatch would zip chapter-span counts to the wrong `source_term` — false: same refutation, the vector is sized from `terms` by construction.
- blind: the 30-code-point list is repeated five times in the trigger DDL — false: a shipped migration constant is never edited; changing the set is a new migration.
- blind: an added `emit_import_scan_failed` line in the filter/enqueue failure arm is mis-indented — low, patch: re-indent the two lines.
- blind: three Story 11.2 items were reassigned to `Chủ: Epic 11` without a spec line — rejected: the fix edits this build's spec; reported to Ice at presentation.
- blind: promoting a counted Work entry shows "—" in Global with no explanation — rejected: low; Decision 8 signs `NULL` for Global, and the column title already says the dash means unknown.
- edge: in `add_manual_term` (Work tier) the candidate `UPDATE` is a second write, so its failure returns `Err` for an entry already saved — medium, patch: the entry and the candidate live in the same `project.db`, so one transaction does it.
- edge: a manual tier pick made before the first quick-add lookup resolves is overwritten by the Work default — low, patch: `onTierChange` writes `quickAddTierChoice` without setting `workTierDefaultApplied`.
- edge: `WorkContext::new` does not compare the resolver's `work_id` with the store's Work — false: `OpenWorkState` holds one `OpenWork` at a time and every production construction takes both halves from it, so no path can mix two Works; `Store` carries no Work id to compare.

## Verification

**Commands:**
- `npm run build && cargo test --test pinned_contract --test glossary_import_dialog_contract --test glossary_exchange_contract --test scope_contract --test ipc_contract` -- expected: green.
- `npx vitest run tests/frontend/glossaryQuickAddStrip.test.ts tests/frontend/glossaryConfirmStripTemplate.test.ts <new files>` -- expected: green.
- `npm run check:debt-owner && npm run check:i18n && npm run check:commands && npm run check:tokens` -- expected: green.
