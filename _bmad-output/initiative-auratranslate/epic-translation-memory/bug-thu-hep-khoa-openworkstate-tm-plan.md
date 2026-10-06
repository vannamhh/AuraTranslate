---
ticket: 13
title: 'Epic 7 retro R-3 — TMX import/export stop holding OpenWorkState beyond the store access'
type: 'bugfix'
created: '2026-10-05'
status: 'done'
route: 'dispatch'
baseline_revision: '4aea97d663aee351de2fe8e542d0c9fe90bd1953'
review_loop_iteration: 1
context:
  - '{project-root}/src-tauri/AGENTS.md'
---

<frozen-after-approval reason="human-owned intent — do not modify unless human renegotiates">

## Intent

**Problem:** The three TMX wire shells (`tm_confirm_import`, `tm_open_import_preview`, `tm_export_tier`) lock `OpenWorkState` for their whole run even when the tier is Global. In that case no Work store is touched, yet a 20 s Global import blocks typing saves, confirm, Chapter load, and closing or switching Work (retro F1.1, F1.2, F1.6). No test in Epic 7 observes how long the lock is held (retro F1 "verification gap").

**Approach:** The Global tier never locks `OpenWorkState`. The Work tier locks it only while it reads from or writes to the Work store. Planning, diffing, TMX rendering and file I/O run unlocked. A pending Work plan records the `work_id` it was planned against, and confirm writes it only into that same Work. Add MockRuntime cases, modelled on `tests/glossary_wire.rs:84`, that observe lock holding at the wire level.

## Boundaries & Constraints

**Always:**
- Lock order stays `OpenWorkState` → `PendingTmxImportState`. No path locks `OpenWorkState` while it holds the pending lock.
- Confirm still holds `PendingTmxImportState` for the whole write, so cancel waits for it (`config_invariants.rs:1064-1069` stays true).
- **Decision (Ice, 2026-10-05, option A):** `clear_pending_tmx_import_for_work` takes the pending lock with `try_lock` and skips the clear when it is busy, so closing or switching Work never waits for a Global confirm. A Work plan that survives this way stays unconfirmable because of the `work_id` check.
- A Work-tier confirm whose plan `work_id` differs from the open Work, or that finds no Work open, writes nothing, drops the plan and returns `tm.no_pending_import`. This is the same outcome as when `close_open_work` clears the plan.
- The Work-tier confirm keeps holding `OpenWorkState` across its write transaction. That cost is measured by R-4, not removed here.
- Two-layer IPC (`src-tauri/AGENTS.md`). The pure functions keep taking `Option<&Store>`/`Option<&OpenWork>`, and every `tests/tmx_contract.rs` case keeps passing unchanged, except for constructing the new `PendingTmxImport` field.

**Never:**
- No `Arc`/clone of a Work `Store` to escape the lock. No change to `Store`, `OpenWork`, `close_open_work` or `replace_open_work`.
- No change to the wire shape (`TmxImportPreviewWire`, `TmxImportSummaryWire`) and no new error code.
- Not in scope: the AI, fuzzy, Concordance and Chapter-load locks (F1.3–F1.5 belong to R-4).

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected Output / Behavior | Error Handling |
|----------|--------------|---------------------------|----------------|
| Global confirm, Work open and locked elsewhere | another thread holds `OpenWorkState` | confirm finishes and writes to `global.db` | N/A |
| Global preview / export, `OpenWorkState` held elsewhere | parsed file / tier `global` | preview stored / TMX written | N/A |
| Work export of a large tier | many pairs | lock held only while `distinct_tier_pairs` reads | N/A |
| Work preview | parsed file | lock held only while `existing_pair_keys` reads; `work_id` recorded | N/A |
| Work plan, Work switched before confirm | plan for A, B open | nothing written to B; plan dropped | `tm.no_pending_import` |
| Close or switch Work during a Global confirm | Global write in progress | the clear returns at once; the Global plan is untouched | N/A |
| Plan tier changes between tier read and lock | Global plan replaced by Work plan | confirm releases the pending lock and runs the Work path once; a Work plan is never written without `OpenWorkState` held | existing codes |

</frozen-after-approval>

## Code Map

- `src-tauri/src/commands/tm.rs:331-342` -- `PendingTmxImport`: add the Work id (`Option<String>`, `Some` only for `TmTier::Work`).
- `tm.rs:414-436` -- `tm_render_tier` reads and renders under one borrow. Split it into a read step (pairs plus an owned tier label) and a render step, so the wire can drop the guard between them.
- `tm.rs:473-492` -- `tm_preview_parsed`: same split. Read `existing_pair_keys` under the lock, then plan, diff and store unlocked.
- `tm.rs:516-529` -- `tm_confirm_import`: check `work_id` against `open.meta.work_id` before writing.
- `tm.rs:670-745` -- the wire shells to narrow. The dialog makes `tm_export_tier`/`tm_open_import_preview` impossible to drive in MockRuntime, so move their post-dialog bodies into `pub fn` helpers in `wire` that take `&AppHandle<R>` plus the path. The commands call these helpers.
- `tm.rs:647-662` `tier_is_ready` -- already a short Work-only lock. Reuse it unchanged.
- `src/core/tm/tmx.rs` -- `render_tmx`, `plan_import`, `existing_pair_keys`, `distinct_tier_pairs`, `write_planned_pairs`. Reuse them and do not change them.
- `src/lib.rs:1375`, `src/commands/project/mod.rs:3390` -- they clear the pending plan before taking `OpenWorkState`. A preview can store a Work plan after that clear runs, which is why the `work_id` check exists. Leave the call sites as they are; change only `clear_pending_tmx_import_for_work` (`tm.rs:536-542`).
- KEEP: `/private/tmp/claude-501/-Users-hoangnam-LocalSites-addon-AuraTranslate/d5f037d3-badb-46c1-8fee-68b39d4e7173/scratchpad/r3-loop1-keep.patch` (`git apply` from the baseline) is the reviewed first-loop code with all of its counter-checks done. Start from it.
- `tests/glossary_wire.rs:73-114` -- the pattern to follow: a MockRuntime app, a `manage` call, the command on a thread, and the main thread measuring its own lock wait.
- `tests/tmx_contract.rs` -- the fixture and `confirm()` helpers. Add the Work-switch case here.

## Tasks & Acceptance

**Execution:**
- [x] `src-tauri/src/commands/tm.rs` -- add the `work_id` to the plan; split the read and render/plan steps; narrow the three wire shells (Global: no `OpenWorkState`; Work: lock only around store access; confirm reads the plan's tier briefly before choosing a path); make `clear_pending_tmx_import_for_work` skip on a busy lock -- F1.1, F1.2, F1.6.
- [x] `src-tauri/tests/tm_wire.rs` (new) -- MockRuntime cases: (1) a Global confirm, preview and export each complete while the test thread holds `OpenWorkState` (join with a timeout so a regression fails instead of hanging); (2) for a Work export and a Work preview, the time the test thread waits for the lock is below a quarter of the shell's run, with a precondition assert that the run lasts long enough to measure -- the retro's verification gap; (3) a Work plan confirmed through `wire::tm_confirm_import` lands in the open Work; (4) while a Global confirm is writing, `clear_pending_tmx_import_for_work` returns within a timeout and leaves the Global plan. The lock-window helper sums every window in which the lock is taken until the shell finishes, not just the first one.
- [x] `src-tauri/tests/tmx_contract.rs` -- a Work plan made against A, confirmed with B open, writes nothing to B, drops the plan and returns `tm.no_pending_import`; a Global preview stores `work_id: None`.

**Acceptance Criteria:**
- Given a Global import is confirming, when the user saves a segment or closes the Work, then that call does not wait for the import.
- Given each new guard, when its seam is removed in production code (re-lock `OpenWorkState` for Global, read under the lock through the render, drop the `work_id` check, make the clear wait on the lock again), then its own test target goes red for that reason (AGENTS.md §Tests).

## Implementation Notes

- Thresholds differ from Task 2's "wait below a quarter of the run". The Work export reads, sorts and dedupes the tier under the lock, which is about a third of the run, so its case polls `try_lock` and asserts held < run/2. With rendering moved back under the lock it measured 1.35 s held of a 1.44 s run. The Work preview case measures the real lock window (first `WouldBlock` to the next success) and asserts < run/8. The first design, wait-after-20 ms, stayed green with planning back under the lock, because the file parse runs before the lock is taken.
- Pure-layer split: `tm_read_tier_pairs`/`tm_render_tier_pairs` and `tm_read_existing_pairs`/`tm_plan_preview`. `tm_render_tier` and `tm_preview_parsed` keep their signatures as compositions. The post-dialog bodies are `wire::export_tier_to`/`wire::open_import_preview_from`. The one-line call from each dialog command into them is not driven by any test, because the dialog cannot run in MockRuntime.
- Confirm: `tm_pending_import_tier` peeks the tier, and `tm_confirm_global_import` returns `Ok(None)` when the plan is no longer Global, so the shell then takes the Work path. That fall-through cannot be raced deterministically; the pure half is covered in `tmx_contract.rs`.
- Counter-checks (real removals, each restored): re-locking `OpenWorkState` for Global times out the Global case; dropping the `work_id` check reds the Work-switch case; dropping the `tier != Global` guard reds `the_global_confirm_leaves_a_plan_that_turned_into_a_work_plan_untouched`; planning under the lock reds the preview case (23% held against a 12.5% limit); a Work confirm shell that does not lock `OpenWorkState` reds `a_work_plan_confirmed_through_the_shell_is_written_into_the_open_work` with `tm.no_pending_import`.
- A Work preview now reads existing pairs before it plans, so when both steps fail the read error surfaces first.
- Loop 2: `clear_pending_tmx_import_for_work` uses `try_lock`. Putting back the blocking `lock()` reds `closing_or_switching_work_does_not_wait_for_a_global_confirm_that_is_writing` on its 2 s timeout. `lock_window_against_run` now sums every taken window: a second `with_tier_store` around planning reds the preview case (468 ms held of a 1.24 s run), but nobody showed that the first-window-only helper would have stayed green. Storing a Global `work_id` reds `a_global_preview_stores_no_work_id`.

## Spec Change Log

- Loop 1 (intent_gap, review E1): a Global confirm held the pending mutex that close/switch Work lock to clear Work plans, so they waited out the import. Ice chose option A, recorded in the frozen block. Amended: Code Map (the clear function, KEEP patch), Tasks (try_lock; tm_wire cases 3–4; summed lock windows; Global `work_id: None`), AC counter-check list. Avoids: a narrowed lock whose close/switch still blocks for 20 s. KEEP: the whole first-loop patch, including the redesigned preview measurement and its recorded counter-checks.

## Review Triage Log

- [edge E1/E-claim1] medium, intent_gap: a Global confirm holds `PendingTmxImportState` for the whole write, and `close_open_work` (`lib.rs:1363`) and `replace_open_work` lock it through `clear_pending_tmx_import_for_work` (`tm.rs:627`), so closing or switching Work still waits for the import. The AC fails, and the frozen "Always: confirm holds pending for the whole write" causes it.
- [blind B1] false: no path nests pending → `OpenWorkState`. Preview releases `OpenWorkState` before `tm_plan_preview` locks pending; close/replace release pending before taking `OpenWorkState`.
- [blind B2] low, rejected: a Work→Global swap between peek and lock writes the Global plan correctly while needlessly holding `OpenWorkState`. It needs a concurrent preview during confirm, and the fix adds a branch.
- [blind B3 · VG2 · edge E3] defer (VG pre-verified): the wire-level Global→Work fall-through has no test, and the race cannot be forced without a hook.
- [blind B4 · edge E4 · VG other] low: the timing tests can flake under load, and the `run > 150 ms` premise can trip on a fast machine. Counter-checks exist (see Implementation Notes), so "no counter-check" is false.
- [blind B5] low: the label `match` is duplicated and `(Work, None)` → Global is reachable only if `tm_read_*` stopped erroring on a Work tier with no Work open. A typed label would make the type tell the truth.
- [blind B6a · edge E2] false: dropping a Work plan with `tm.no_pending_import` when no Work is open is the frozen Always rule.
- [blind B6b] low: no assert that a Global preview stores `work_id: None`.
- [blind small: doc comments] low, rejected.
- [blind small: unaccented test strings] false: `tests/glossary_wire.rs` and `tmx_contract.rs` use the same unaccented style.
- [blind small: wrappers] false: `tm_render_tier`/`tm_preview_parsed` are still called by `tm_export_tier` (`tm.rs:459`) and `tm_open_import_preview` (`tm.rs:573`).
- [VG1 · edge E6 · E-claim3] defer (VG pre-verified): the dialog shells `tm_export_tier`/`tm_open_import_preview` are not driven, so a lock added in their bodies would go unseen.
- [edge E5] medium: `lock_window_against_run` stops at the first release, so a shell that re-locks `OpenWorkState` for planning after a short read passes.
- [edge E-claim2] low: the thresholds deviate from the Task text and are recorded in Implementation Notes; the export ratio of 1/2 accepts a partial regression.
- [edge E-claim4] false: the spec allows the lock across the store read, and `distinct_tier_pairs` is that read.
- Loop 2 [VG1] patch (pre-verified): a Work plan confirmed with no Work open (the `is_none_or` arm) had no test.
- Loop 2 [VG2 · blind 1 · edge 4] low, defer (`deferred-work.md`, Chủ: Amelia): a Work plan can survive a close when the clear meets a busy lock. Stale keys cannot cause duplicates (`write_planned_pairs` rechecks existence in the transaction). Both of VG's demonstrations are false: a blocking `lock()` reds the clear case, and a clear that never runs reds `closing_the_work_clears_a_work_plan_and_keeps_a_global_one`.
- Loop 2 [edge 3] patch, medium: the clear case stopped at the first `WouldBlock`, which can be the confirm's tier peek, so the clear could run against a free lock and the case was vacuous on some runs.
- Loop 2 [edge 1 · edge 2 · blind 3] carried: low (thresholds and timing; see E-claim2 and B4).
- Loop 2 [edge 5] carried: low, rejected (B2).
- Loop 2 [blind 2 · VG other] carried: defer (B3 · VG2 · E3); the debt item is recorded with Chủ: Amelia.
- Loop 2 [blind 4] low, rejected: `lock_held_against_run` and `lock_window_against_run` measure different things on purpose (the export lock is taken at once; the preview lock is taken after the parse). Test-only.
- Loop 2 [blind 5] carried: low (B5).
- Loop 2 [blind 6] low, rejected: an unknown tier string takes the no-lock path and is then refused by `tmx_tier_from_wire`; no state is touched.
- Loop 2 [blind 7] false: without `tier_is_ready`, the helpers still return `work.none_open` from `tmx_tier_store`, the same code as the shell.
- Loop 2 [blind 8] carried: false (wrappers are used by the pure `tm_export_tier`/`tm_open_import_preview`).
- Loop 2 [blind 9] carried: false (B1). Spine AD-18/AD-6 is retro R-7, not this item.
- Loop 2 [blind 10a] low, rejected: confirming a plan for a reopened Work with the same id is correct by design.
- Loop 2 [blind 10b] false: `is_none_or` is already used in `split.rs:498`, `tmx.rs:408` and `tm/mod.rs:392`.
- Loop 2 [VG1-loop1 dialog shells] carried: defer; the debt item is recorded with Chủ: Epic 7.

## Verification

**Commands:**
- `npm run build && cd src-tauri && cargo test --test tm_wire --test tmx_contract --test config_invariants --test ipc_contract` -- expected: green.
- `cargo clippy` is not a gate (absent from `.githooks/pre-push`, `package.json`, `ci.yml`); its 70 errors at this baseline are all in files this change does not touch.
