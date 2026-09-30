---
title: 'Story 11.8, lot A — Rust boundary faults between Epic 11 stories, and guards that reach the wiring'
type: 'bugfix'
created: '2026-09-30'
status: 'ready-for-dev'
route: 'dispatch'
review_loop_iteration: 0
context:
  - '{project-root}/src-tauri/AGENTS.md'
  - '{project-root}/_bmad-output/implementation-artifacts/epic-11-context.md'
---

<frozen-after-approval reason="human-owned intent — do not modify unless human renegotiates">

## Intent

**Problem:** On HEAD `8c801ef`, 11 of the 21 items ending `Chủ: Story 11.8` are Rust faults or guards left where two Epic 11 stories meet (Task 0: `11-8-task0-2026-09-30.md` §Group A). One bad `confirm_segment` call locks a whole Work. APPEND into the open Work races the orphan-image sweep. A merge forgets `last_chapter_id`. Flushes can touch retired segments. Several guards call the patched function, not the `#[tauri::command]` shell, or EXPLAIN hand-copied SQL.

**Approach:** Lot A goes first; lot B (webview, e2e, CI) follows and closes the story, because AC ③ needs the e2e run on the story's last commit. Every item below gets one disposition. Every new guard is counter-checked by really removing its seam.

## Boundaries & Constraints

**Always:** Ledger items keep their text; only `→` lines are appended, and a stale claim gets 🔵 in place. AD-47: the `confirm_segment` keep branch still sets status and origin in one `UPDATE`. A shell goes generic over `R: tauri::Runtime` only when a test must drive it. Its text pins (`config_invariants.rs`, `ipc_contract.rs`, `ipc_argument_contract.rs`) change in the same commit. A real-app check goes to `Chủ: Epic 11`, a Windows-only one to `Chủ: B7`.

**Never:** A migration, a new dependency, or a new gate. Changing `glossary_candidate` DDL. Changing `split_chapter_into_segments`'s locking. Writing `self` for an unknown origin. Rewriting stale comments beyond the sites named here.

## Dispositions

Agent, 2026-09-30 (Ice may override any line at approval):
1. F-R-1 fix: in the keep branch, an `origin_at_load` outside `TRANSLATION_ORIGINS` returns a new named error, writes nothing, and the Work still opens afterwards. This is fixed by the story's own AC ② ("từ chối và không ghi gì"), so the "keep disk origin" option is out. The key and its `vi.json` string are new.
2. F-R-2 fix: one `AppendInProgressGuard` is built before the `already_open` split and lives across both branches. Its presence is observed from inside the call through the image-progress event, IF an offline pending source emits one; measure that first. Otherwise the guard is the weaker "set empty after every path" case and the item stays 🟡.
3. F-R-4 fix: the merge calls `set_open_chapter`, guarded by a 3-chapter case (the 2-chapter case cannot see the bug). The premise comment of the `project_contract.rs:2485` case is corrected.
4. F-D-9 + F-R-8: the 9 comment sites are rewritten to what is true (the wire takes a concrete `AppHandle`), and the `open_work` doc at `mod.rs:3016` says `Store::open` may already have set WAL, backed up and migrated. No pre-open peek: whether an old DB has the column is unverified.
5. Wire harness: new `tests/project_wire.rs` on `mock_builder()`. `wire::open_work` (with `replace_open_work`) and `wire::confirm_import_with_encoding` go generic. Cases: orphan sweep at open (L9974), asset scope revoked and re-granted across a Work switch (L10502), reindex after APPEND into the open Work (L7443 seam), and item 2. Ledger: L9974 and L10502 close; L12593 shrinks to the true-concurrency hand check (🟡 `Chủ: Epic 11`); L7443 gets a 🔵 premise fix.
6. F-D-8: `wire::promote_ai_translation` goes generic, with four shell cases: write, retired, `force` forwarded, and no `OpenWorkState` managed.
7. F-D-1 (a): `boundary_scan_contract.rs` enumerates `tests/*_boundary.rs` (19, with a floor) on `code_lines` and asserts each declares the shared module, calls a population helper and `assert_population_floor`, and has no bare `starts_with(<CONST>)`. A seeded-regression case proves the predicate can fail.
8. F-D-1 (b): the three branch SQL strings of `core/dict/query.rs` become `pub` builders, and the EXPLAIN case runs them. Check first that the plan with bound parameters matches the literal plan.
9. F-R-3, measure first (Task 0 §F-R-3 recipe, release, signed binary, 20 runs). If p95 lock-hold and wait are < 100 ms at K=500, C=300 ⇒ `KHÔNG LÀM` with the numbers and a reopen condition. If ≥ 100 ms ⇒ stop and bring the numbers to Ice; no fix in this lot. The APPEND network hold is structural and is recorded only.
10. F-R-5 + F-A-4: `approve_candidate` maps a legacy 0 to `None` (twin case: 37 stays 37). `chapter_span_count` and `bilingual.rs` `column_count` ⇒ `KHÔNG LÀM` with Task 0's reopen conditions.
11. F-R-6: `AND retired_at IS NULL` in both the `unconfirm_edited_segments` membership count and the `save_segment_targets` `UPDATE`, so the batch is rejected whole as `segment.unknown_ids`.
12. F-A-3: `KHÔNG LÀM` for the `matching_close_brace` half: same name, one scans Rust and one scans TypeScript. L12692 gets a 🟡 line; lot B closes its F-W-9 half.

Ice, 2026-09-30:
13. Push run `36702088358` on HEAD `8c801ef` is red on `check (windows-2025)` only (`bindingsEpochWiring.test.ts`, two cases, from `7dbcf23`). It becomes a new ledger item `Chủ: Story 11.8`, fixed in lot B. Until then, lot A's CI reads cite that run id as the reason for the red Windows half.
14. The spec stays whole at ~2.5k tokens; per-item detail lives in Task 0.

</frozen-after-approval>

## Code Map

- Task 0 per item (file:line at HEAD, guard design, counter-check, risks): `11-8-task0-2026-09-30.md` §Group A. The phase agents read their item there, not here.
- Pattern to copy: `src-tauri/tests/ai_translate_wire.rs` (`Harness`, `mock_builder` :120); generic shells `commands/aitranslate.rs:796-797`.
- Shells to make generic: `commands/project/wire.rs` `confirm_import_with_encoding` :785 (APPEND split :892-943), `open_work` :1208; `commands/project/mod.rs` `replace_open_work` :3314 (sweep :3358-3379, scope :3346-3352 / :3409-3415); `commands/segment.rs` shell `promote_ai_translation` :3865.
- Signature pins: `tests/config_invariants.rs` :1079, :1115, :1195, plain-shell census :1522-1650; `tests/ipc_contract.rs` :2165-2185; `tests/ipc_argument_contract.rs` :151-153.
- Fix sites: `commands/segment.rs` :2519-2530 (keep branch), :1972-1976 and the membership count in `unconfirm_edited_segments` (~:2881); `commands/chapter.rs` :951-955; `core/glossary/candidate_store.rs` :345; `core/dict/query.rs` :200-207, :265-273, :288-299; `core/i18n/mod.rs` `message_keys!` (neighbour `StoreUnknownTranslationOrigin` :144); `src/i18n/vi.json`.
- Tests: `segment_contract.rs` (:5902 misses the keep branch; saves from :2720), `project_contract.rs` :2485, `glossary_commands_contract.rs`, `dict_sources.rs` :5298-5366 (read with offset/limit), `boundary_scan_contract.rs`.

## Tasks & Acceptance

**Execution:**
- [ ] `src-tauri/src/**`, `src/i18n/vi.json` -- Dispositions 1-6, 8, 10, 11: the generic shells first (5, 6), then the fixes -- Rust phase.
- [ ] `src-tauri/tests/**` -- the guards of Dispositions 1-8, 10, 11; one real removal per guard; the F-R-3 measurement (throwaway probe, not committed); the full suite once, because shell signatures and the pins change -- Tests phase.
- [ ] `deferred-work.md` -- one `→` line for each of the 11 items and for L7443, L9974, L10502, L12593 -- Ledger phase.

**Acceptance Criteria:**
- Given a draft segment whose on-disk origin is set and whose text is unchanged, when `confirm_segment` receives an origin outside the FR117 catalogue, then it errors, the row is unchanged, and `open_work` still opens the Work.
- Given a Work with 3 Chapters and Chapter 3 open, when Chapter 3 is merged into Chapter 2 and the Work reopened, then Chapter 2 opens.
- Given a batch holding one live and one retired segment, when it is flushed, then it is rejected whole, and the live segment keeps its text and `confirmed` status.
- Given each new guard, when its seam is really removed in production code, then only that guard's target goes red, for that reason.
- Given lot A is done, when the 11 item lines are read, then each ends in one `→` disposition (L12692 🟡 for lot B) and `npm run check:debt-owner` is green.

## Implementation Notes

## Spec Change Log

## Review Triage Log

## Verification

**Commands:**
- `npm run build`, then one `cargo test --test <target>` per changed target (`segment_contract`, `project_contract`, `project_wire`, `ai_translate_wire`, `glossary_commands_contract`, `dict_sources`, `boundary_scan_contract`, `config_invariants`, `ipc_contract`, `ipc_argument_contract`) -- green.
- The full suite once in the Tests phase; `npm run check:i18n`, `npm run check:debt-owner` -- green.
