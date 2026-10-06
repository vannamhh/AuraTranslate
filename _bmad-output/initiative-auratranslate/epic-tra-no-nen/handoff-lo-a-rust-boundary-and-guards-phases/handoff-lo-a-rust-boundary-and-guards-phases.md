---
type: handoff
title: "Story 11.8 lot A — phase handoff (working notes, not spec)"
status: done
created: 2026-09-30
skill: bmad-build
---

# Story 11.8 lot A — phase handoff (working notes, not spec)

## Rust phase

Done in `src-tauri/src/**` and `src/i18n/vi.json`; `cargo check` green. Nothing committed. No test file touched.

Changes, by Disposition:
- 1 (F-R-1): `commands/segment.rs` `confirm_segment` keep branch: `origin_at_load` outside `TRANSLATION_ORIGINS` now sets `ConfirmReject::UnknownOrigin` and returns `IpcError` code `segment.unknown_translation_origin`, key `MessageKey::SegmentUnknownTranslationOrigin` (`err.segment.unknown_translation_origin`, params `segment_id`, `value`), fn `segment_unknown_translation_origin`. New key in `core/i18n/mod.rs` (after `SegmentNothingToConfirm`) and `src/i18n/vi.json`. Branch order unchanged: text changed or empty disk origin => `self` (no check on the claim there, by design). Same single `UPDATE` for status + origin (AD-47).
- 2 (F-R-2): `commands/project/wire.rs` `confirm_import_with_encoding`: `AppendInProgressGuard` now built before the `already_open` split (after `destination_work_id` is Some), dropped explicitly after the `(created, new_chapter_ids)` if/else (it borrows `app`, which `spawn_import_scan(app, ..)` moves). Duplicate in the not-open branch deleted. Lock order re-read: guard only inserts into a HashSet under a short mutex; sweep holds that mutex but never takes `OpenWorkState`. NOT measured: whether an offline pending source emits `URL_IMPORT_IMAGE_PROGRESS_EVENT` on the append path (Disposition 2 says measure first; the fallback is "set empty afterwards", item stays 🟡).
- 3 (F-R-4): `commands/chapter.rs` merge: `open.chapter_id = a_id` -> `set_open_chapter(open, a_id)?` (same condition, still after commit, before `write_lifecycle_after_change`). Premise comment of `tests/project_contract.rs:2485-2516` (:2501-2503) is now false: Tests phase must fix it.
- 4 (F-D-9 + F-R-8): 4 src comment sites rewritten (`url_import.rs` ~:56, `mod.rs` ~:219 and ~:2726, `work_creation.rs` ~:949). `open_work` doc `# Lỗi` bullet in `mod.rs` now says `Store::open` may already have set WAL, backed up, migrated. 5 TEST sites still to rewrite in the Tests phase: `tests/chapter_origin_contract.rs:458-459`, `tests/ipc_contract.rs:974-975`, `:1038-1039`, `:1715-1716`, and the `project_contract.rs` sites (the spec's `:925` is only a pointer with no such text; the real "no MockRuntime" sentences are at `project_contract.rs:1083` and `:5617`). NOT touched by me (unnamed, so left): `mod.rs` ~:2996 "crate này không khai ... test-utils" in the `open_work` pure-fn doc.
- 5 (wire harness generics): generic over `R: tauri::Runtime` (`AppHandle<R>`): `wire::open_work`, `wire::confirm_import_with_encoding`, `replace_open_work`, `spawn_import_scan`, `run_one_chapter_import_scan`, `emit_import_scan_failed`, `default_library_root`, `resolve_library_root` (mod.rs); in wire.rs `resolve_cleanup_rules`, `resolve_cleanup_rules_for`, `resolve_cleanup_rules_for_destination`, `resolve_tier2_block_overrides`, `reset_tier2_block_overrides`, `resolve_chapter_origin_overrides`, `reset_chapter_origin_overrides`, `reindex_library`. `tests/project_wire.rs` NOT written.
- 6 (F-D-8): `wire::promote_ai_translation<R: tauri::Runtime>(app: tauri::AppHandle<R>, ..)` in `commands/segment.rs`.
- 8 (F-D-1 b): `core/dict/query.rs` new `pub fn exact_sql()`, `char_idx_one_sql()`, `char_idx_two_sql()` (`exact`/`char_idx` call them); re-exported from `core/dict/mod.rs` as `core::dict::{exact_sql, char_idx_one_sql, char_idx_two_sql}`. Params: exact `?1` query `?2` limit; one-char `?1 ?2`; two-char `?1 ?2 ?3` (ceiling). Test must first measure that EXPLAIN with `?` params matches the literal-plan (Task 0 warning). Grep `tests/dict_boundary.rs` for vocabulary rules on new pub items.
- 10 (F-R-5): `core/glossary/candidate_store.rs` approve: `(occurrence_count > 0).then_some(occurrence_count)`. (b)/(c) are ledger-only KHÔNG LÀM.
- 11 (F-R-6): `AND retired_at IS NULL` added to the `save_segment_targets` UPDATE and to the membership count in `unconfirm_edited_segments` (segment.rs). NOT added to the position-check count at ~:1642 (a different fn, not in scope).

Pins the Tests phase must update in the same commit (text pins, would go red now):
- `tests/config_invariants.rs:1115` -> `"pub fn confirm_import_with_encoding<R: tauri::Runtime>(\n        app: tauri::AppHandle<R>"`
- `tests/config_invariants.rs:1195` -> `"pub fn open_work<R: tauri::Runtime>(app: tauri::AppHandle<R>"`
- `tests/ipc_contract.rs:2165-2185` (`fn_param_list` for `promote_ai_translation`) and plain-shell census `config_invariants.rs:1522-1650`: check they still accept the generic shell. `:1079` (`merge_chapter_into_previous`) unchanged (not generic).
- `tests/project_contract.rs:5634` `.find("fn replace_open_work(app: &tauri::AppHandle")` -> `"fn replace_open_work<R: tauri::Runtime>(app: &tauri::AppHandle<R>"` (case `replace_open_work_clears_the_last_assembled_prompt_record_beside_its_two_siblings`, red now; measured). Its doc at :5617 also carries a false "no MockRuntime" sentence.
- `tests/config_invariants.rs:1819-1820` (+ fixtures :1892, :1913) find `"pub fn resolve_library_root("`: now `pub fn resolve_library_root<R: tauri::Runtime>(`. Not yet run; may be red.
- Measured on this tree: `segment_contract` 89 pass / 3 ignored, `glossary_commands_contract` 30 pass; `project_contract` 137 pass, 1 fail (the pin above). `dict_sources` and the pin targets (`config_invariants`, `ipc_contract`, `ipc_argument_contract`) not confirmed.
- Not verified by me: existing `cargo test` targets after these changes (see final report; `dist/` was missing so `npm run build` is needed first).

Tests-phase guards still to write (Disposition list): segment_contract keep-branch case + open_work still opens; project_contract 3-chapter merge case; segment_contract retired save/flush cases (2); glossary_commands_contract legacy 0 vs 37 twin; dict_sources EXPLAIN on builders; boundary_scan_contract (Disposition 7, no src change needed); project_wire.rs (sweep at open, asset scope revoke/regrant, reindex after APPEND into open Work, APPEND guard presence); promote shell cases (write, retired, `force`, no OpenWorkState); F-R-3 measurement (throwaway).

## Tests phase, part 1

Done in `src-tauri/tests/**` only. Nothing committed. Left for part 2: `tests/project_wire.rs` (Dispositions 5 and the 2 guard), F-R-3 measurement, the full suite, the `segment_wire`/promote shell cases (Disposition 6 shell cases NOT written here: the generic shell exists but no case drives it yet).

Pins and comments:
- `config_invariants.rs`: `confirm_import_with_encoding` and `open_work` pins take the generic form; `resolve_library_root_body` finds `pub fn resolve_library_root<R: tauri::Runtime>(`.
- `ipc_contract.rs`: `fn_param_list` and `wire_fn_body` accept `pub fn name<R: tauri::Runtime>(` as well as the plain form; expected params of `confirm_import_with_encoding` and `promote_ai_translation` now say `AppHandle<R>`. The plain-shell census in `config_invariants.rs` needed no change (green).
- `project_contract.rs`: `replace_open_work` pin generic; false "no MockRuntime" sentences at the `replace_open_work` case doc and the `url_import_items_state...` doc rewritten; the stale-fallback merge case now seeds `last_chapter_id` by SQL because the merge rewrites it itself.
- Rewritten stale comments in `chapter_origin_contract.rs` (~:458) and `ipc_contract.rs` (~:974, :1038, :1715). `project_contract.rs` :620 and :874 say only "no AppHandle needed" and were left.

Guards, each with a real removal in production code, red only for its own case:
- D1 `segment_contract`: keep-branch refusal + reopen via `open_work`, plus a positive twin (catalogue origin keeps disk origin). Removal: keep branch writes `origin_at_load` verbatim ⇒ only the refusal case red (`Ok(ConfirmOutcome)`).
- D11 `segment_contract`: save case and flush case with one retired id. Removing `AND retired_at IS NULL` from the `UPDATE` ⇒ only the save case red; removing it from the membership count ⇒ only the flush case red (live segment lowered to draft).
- D3 `project_contract`: 3-chapter merge (Chapter 3 open into 2) asserts `work.last_chapter_id` and reopen; plus a non-open merge case. Removal: `open.chapter_id = a_id;` ⇒ only the 3-chapter case red (disk 3, want 2).
- D10 `glossary_commands_contract`: legacy 0 ⇒ NULL, 37 ⇒ 37. Removal: `Some(occurrence_count)` ⇒ only the 0 case red.
- D7 `boundary_scan_contract`: `every_boundary_file_scans_through_the_shared_module` (19 files, `BOUNDARY_FILE_FLOOR = 17` = ceil(0.85 x 19)), plus two seeded-regression cases. Removal 1: `ai_boundary.rs:543` `is_inside(rel, AI_DIR)` => `rel.starts_with(AI_DIR)` ⇒ red naming `ai_boundary.rs:543`. Removal 2: `#[path]` line deleted from `cleanup_boundary.rs` ⇒ red naming that file. Both restored.
- D7 caveat: the "calls a population helper" clause matches `rust_sources(`/`any_sources(`/`paths_with_extensions(` unprefixed, because `naming_boundary.rs` feeds its floor from a LOCAL `rust_sources()` (it needs comment-blanked text). Weaker than "calls the shared helper"; a file with a same-named local fn passes.
- D8 `dict_sources::branch_one_and_two_never_scan_the_table` now runs `exact_sql()`, `char_idx_one_sql()`, `char_idx_two_sql()`. Measured first: `EXPLAIN QUERY PLAN` needs values bound (rusqlite errors `InvalidParameterCount` on unbound `?N`), and with bound values the plan text equals the literal-SQL plan for all three branches (release-free, debug test build, fixture with `ANALYZE`). Removal 1: `lower(e.headword)` in `exact_sql` ⇒ `SCAN e`, red. Removal 2: `OR lower(entry_id) < 0` in `char_idx_one_sql` ⇒ `SCAN char_idx`, red (this is the branch-2 removal; the 2-char builder was not separately broken). Side finding: `query.rs` :488 holds a SECOND hand copy of the branch-1 WHERE inside a different function; not in scope, not touched.

## Tests phase, part 2

Done in `src-tauri/tests/**` only (probe deleted). Nothing committed. Ledger untouched.

`tests/project_wire.rs` (MockRuntime, global Store with `library_root`, real `Indexer`; 5 pass, 1 ignored). Each guard has a real removal in production code, red only for its own case:
- sweep at open (L9974): remove `sweep_orphaned_asset_files` call in `replace_open_work` => only `opening_a_work_through_the_shell_sweeps_only_the_unreferenced_image_files` red.
- sweep skipped while APPEND registered: remove the `should_skip_orphan_sweep` block => only `..._skips_the_sweep_while_an_append_into_it_is_registered` red.
- asset scope revoke (L10502, revoke half): `forbid_directory` => `allow_directory` => only `the_asset_scope_follows_the_open_work_when_another_work_is_opened` red.
- reindex after APPEND into the open Work (L7443 seam): empty the `if already_open { reindex_library }` => only `an_append_into_the_open_work_moves_the_library_index_row_of_that_work` red.
- F-R-2 guard presence: measured first, an offline `.docx` with one embedded image (`fixtures_docx::image_png`, Local image) DOES emit `URL_IMPORT_IMAGE_PROGRESS_EVENT` on the append path, so the in-call observation variant is possible (no "set empty afterwards" fallback). The listener reads `AppendInProgressState` from inside the emit. Removing the guard creation => only `an_append_into_the_open_work_registers_the_append_guard_while_its_images_are_written` red. Item 2 can therefore close (not 🟡) on presence; real concurrency stays the L12593 hand check.
- 🔴 REAL PRODUCT BUG found: `reopening_a_work_after_leaving_it_grants_its_assets_again` (open A, B, A) is red on production code. Tauri `forbid_directory` has no inverse and outranks every allow (tauri-2.11.5 `scope/fs.rs`), so the revoke on A -> B blocks A's `assets/` for the rest of the session. Exactly the trap the original ledger item named ("chặn luôn lượt mở LẠI A"); Story 11.6's `→ 🟡` line claims "allow lại đúng thư mục khi Tác phẩm đó mở lại", which is false. The case is `#[ignore = "..."]` with a reason so the suite stays green; a fix needs an Ice decision (no unforbid API: rebuild scope is not possible either, `asset_protocol_scope()` shares one inner). Ledger phase: L10502 must NOT close on the re-grant half; new item `Chủ: Ice`/Story 11.8 lot B.

`tests/segment_wire.rs` (Disposition 6, 4 pass): write (text + origin `other`), retired (`segment.retired` for force false/true, row untouched), `force` forwarded (held then written), no `OpenWorkState` => `work.none_open`. Removals: `force` hardcoded `false` in the shell => only the force case red; `app.state::<OpenWorkState>()` in place of `try_state` => only the no-state case red.

Disposition 7 clause (population helper): prefix now required (`boundary_scan::rust_sources|any_sources|paths_with_extensions(`); `naming_boundary.rs` is a named exemption `POPULATION_HELPER_EXEMPT` with its reason (own comment-blanking walkers), and `the_population_helper_exemptions_are_still_needed` fails once that file calls a shared helper. Seeded case for the clause added. Real removal: stripped the `boundary_scan::` prefix in `cleanup_boundary.rs` => red naming that file. All 19 files pass with the required prefix.

Disposition 8, branch 2-char counter-check on its own: the `lower(ch)` edit on the second `char_idx` arm of `char_idx_two_sql` stays GREEN (the case only asserts no `SCAN e` plus some index in the plan; that edit leaves `dict_entry` searched). A real edit that does reach it (`e.id + 0 IN (` in `char_idx_two_sql`) => `SCAN e | LIST SUBQUERY`, red only for `branch_one_and_two_never_scan_the_table`, label "nhánh 2 (CharIdx, 2 ký tự)". The case does not guard the `char_idx` side plan; noted, not widened.

F-R-3 measurement (throwaway `tests/lock_hold_probe.rs`, deleted): release profile, ad-hoc signed binary, 20 runs per point, same machine load, `glossary_pending_candidates(Some(&open), ..)` = lock-hold, waiter thread taking `OpenWorkState.lock()` 1 ms after start = wait. Population: synthetic Chinese Work, C chapters x 4,000 chars, terms of 2 chars appearing ~1 in 6 tokens. `dict/` holds no `.db` in this tree, so `DictLayers::empty()`: Han-Viet lookup cost NOT included (a lower bound). Results, ms p50 / p95 of hold (wait tracks it within 2 ms):
- K=50, C=300: 317-326 / 377-398
- K=500, C=300: 1240-1292 / 1319-1344 (two runs)
- K=2000, C=300: 3548-3765 / 3681-4008
- K=500, C=100: 388-463 / 472-527
- K=500, C=20: 82 / 107
- APPEND non-network part, M=200 chapters x 4,000 chars, 0 images: 316 / 402
Decision per Disposition 9: p95 >= 100 ms at K=500, C=300 (13x the bar), and the APPEND non-network part is also >= 100 ms => STOP, no fix in this lot, numbers go to Ice. `KHÔNG LÀM` is NOT available. Cost scales with C x K (`candidate_chapter_span_counts` matches every term over every chapter text under the lock); a per-command fix (compute counts on a cloned read, drop the guard) is the candidate, not `split_chapter_into_segments`. Limits: synthetic text, no dictionary layers, unrelated to real-app stall (that is a sync `save_segment_targets` on the main thread).

Full suite (`npm run build` then all of `cargo test`, debug, this tree): 72 result lines, 1953 passed, 0 failed, 25 ignored (includes the one `#[ignore]` above). `check:i18n`, `check:debt-owner` and vitest not run in this phase.

## Ice decisions 15-16

Nothing committed. Ledger untouched.

Decision 15 (asset scope): `replace_open_work` no longer calls `forbid_directory`; the `assets_dir_to_forbid` helper and its `scope_revocation_tests` module are deleted. `tests/project_wire.rs`: the reopen case lost `#[ignore]` and now asserts A and B both stay allowed; the switch case is renamed `every_work_opened_in_the_session_keeps_its_assets_in_the_asset_scope` (B not allowed before it opens, A still allowed after A -> B). Counter-check: restoring the forbid in `replace_open_work` => both cases red (2 failed, 4 passed), restored => 6 pass. NOT changed: `lib.rs::close_open_work` still forbids the closed Work's `assets/` (same no-inverse trap if that Work is reopened in the same session after a close; Ice's decision named only `replace_open_work`). Ledger phase should record this as an owned item or ask Ice.

Decision 16 (F-R-3): `commands/glossary.rs` `glossary_pending_candidates` split in three: `read_pending_snapshot(open)` (DB reads, the only part under the lock; owned `PendingSnapshot`), `build_pending_candidates(snapshot, layers, disabled)` (Han-Viet lookup + span count, no Work), and the pure `glossary_pending_candidates(open, ..)` composing both with unchanged signature. `core/glossary/candidate_store.rs`: new `chapter_source_texts(store)` and `chapter_span_counts_in(chapters, terms, lang)` (re-exported), `candidate_chapter_span_counts` composes them. The shell is now `glossary_pending_candidates<R: tauri::Runtime>(app: AppHandle<R>)` and takes the guard only around `read_pending_snapshot`; the two `config_invariants.rs` markers take the generic form. New `tests/glossary_wire.rs` (MockRuntime, 100 chapters x 4,000 chars, 200 terms, debug): a waiter takes `OpenWorkState.lock()` 20 ms after the shell starts and its wait must be < scan_time / 4. Counter-check: holding the guard across `build_pending_candidates` in the shell => red (wait 2.36 s of a 2.38 s scan), restored => green.

Re-measurement (throwaway probe, deleted; release profile, same recipe as part 2: K=500, C=300 x 4,000 chars, 20 runs, `DictLayers::empty()`, waiter takes the lock 1 ms after start, load average ~4 like part 2; binary is the cargo test binary, not ad-hoc re-signed, which does not affect in-process timing). Synthetic text uses a 60-character alphabet, so term density differs from part 2 and the old-path number is lower (0.97 s vs 1.3 s), only the same-population before/after inside the probe is comparable:
- old behaviour (pure fn called under the lock): hold p50 919 ms, p95 966 ms, max 1202 ms;
- shell now: waiter wait p50 5 ms, p95 6 ms, max 6 ms; shell total p50 919 ms, p95 952 ms (the scan still costs the same, it no longer blocks the lock).
Bar (< 100 ms p95): met at 6 ms. Limits: no dictionary layers (Han-Viet lookup cost not included, but it now runs unlocked too); wait is measured at one waiter, not real concurrent `save_segment_targets`. That true-concurrency check stays the L12593 hand check.

Full suite: `cargo test` (debug, dist present) 73 result lines, 1952 passed, 0 failed, 24 ignored (the reopen case is no longer ignored; 3 deleted unit tests, +1 `glossary_wire`). `check:i18n`, `check:debt-owner`, vitest not run.

## Ledger phase

Only `deferred-work.md` and this file touched; 17 `→` lines appended, no existing text changed. Nothing committed.
- 11 lot-A items in the 11.8 section: F-R-1, F-R-2, F-R-4, F-D-8, F-D-1 (a), F-D-1 (b), F-R-6, F-R-5+F-A-4 (a closed + one KHÔNG LÀM line for b/c) are closed with test file + case name; F-D-9+F-R-8 is 🟡 (two stale sentences left: `commands/project/mod.rs:2996`, `commands/project/tests.rs:704`, Chủ: Story 11.8); F-R-3 is 🟡 (glossary half fixed and measured; APPEND lock-hold across the image download recorded, `Chủ: Ice`); F-W-9+F-A-3 (L12692) is 🟡 for lot B with the `matching_close_brace` half KHÔNG LÀM.
- L7443: 🔵 premise fix (MockRuntime exists), only the APPEND-into-open-Work `reindex_library` seam covered. L9974: closed. L10502: 🔵 on the false "allow lại" claim, then closed by decision 15; `close_open_work` forbids on exit only (`lib.rs:1011`). L12593: 🟡 shrunk to two hand checks, `Chủ: Epic 11`.
- Not touched: L11262 (APPEND twin of L9974, still says the wire is unmeasured); the lot-B items; the Windows CI item. `SECURITY-NOTES.md` singular "thư mục .atproj ĐANG MỞ" wording vs decision 15 is flagged `Chủ: Ice` in the L10502 line, not edited.
- `check:debt-owner` and `check:i18n` green.
