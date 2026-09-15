# AI-4 — loopback 1 history (2026-09-15)

Sidecar for `spec-ai-4-sau-lenh-nhap-roi-luong-giao-dien.md`. Moved out of the spec so the
actionable half stays readable. The code these notes describe was reverted at the loopback —
the diff is kept at `ai-4-implementation-2026-09-15.patch`, which is a **dead historical
artifact, not a restorable state** (see the note at the head of that file). The *conclusions*
that must survive re-derivation live in the spec's `## Spec Change Log`, not here.

🔵 **AMENDED 2026-09-15 (review round 3).** This header previously read "nothing here was
deleted or edited". That is an exemption `AGENTS.md:51` does not grant: *a claim that stops
being true gets FIXED IN PLACE with 🔵 and a date; don't delete it, and don't let it quietly
lie.* The rule has no carve-out for history files, and one sentence here was asserting — in
the present tense, as a verified manual check — the exact false premise this whole change
exists to refute. **Nothing is deleted from this file; refuted claims are struck through and
answered in place.** Preserving a record faithfully and letting it keep lying are not the
same thing.

---

## Implementation Notes

Order actually worked, matching D1's required sequence: ① added the four `cases` rows to `config_invariants.rs` (array `18 → 22`) while all four attributes were still plain → ran the gate → RED. ② Flipped the four attributes to `#[tauri::command(async)]` with doc comments modelled on `:6264-6266` → ran the gate → GREEN. ③ Re-measured plain/`(async)` counts on the fixed tree (`project.rs` 15/2 → 11/6, totals 55/24) and only then wrote the 12-file D1 count-table declaration (`the_blocking_wires_gate_count_table_covers_every_command_bearing_file`). Also refactored the `cases` array out into a shared `blocking_wire_cases()` fn so `the_blocking_wires_gate_reads_more_than_one_file` derives its file set from it instead of a hand-typed list (Task 4).

**Step ① — RED, verbatim** (`cargo test --locked --test config_invariants the_blocking_wires_run_off_the_main_thread`, four rows added, all four attributes still plain):

```
running 1 test
test the_blocking_wires_run_off_the_main_thread ... FAILED

---- the_blocking_wires_run_off_the_main_thread stdout ----
thread '...' panicked at tests/config_invariants.rs:1136:9:
vo tai `src/commands/project.rs` (chu ky `pub fn create_work_from_file(
        app: tauri::AppHandle`) KHONG mang `#[tauri::command(async)]` -- lenh dong bo chay tren
LUONG CHINH, va no CHAN o day: doc dia dong bo toi MAX_IMPORT_BYTES 100 MB
(core/segment/import.rs) roi giai nen `.docx` -- ca hai tren luong chinh, spec AI-4. ⇒ TREO UNG
DUNG (do 2026-08-25 tren cua so that, nhanh hop thoai). Dong doc duoc ngay truoc chu ky:
"#[tauri::command]"

test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 28 filtered out; finished in 0.01s
```

Names a signature (`create_work_from_file`), not a count mismatch — the loop's `assert!` panics on the first offending case in iteration order, so only one of the four is named here even though all four were still plain.

**Step ② — GREEN, verbatim** (same command, after flipping all four attributes):

```
running 1 test
test the_blocking_wires_run_off_the_main_thread ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 28 filtered out; finished in 0.01s
```

**Mutation check (Task 5), four separate runs** — each: `sed -i` one attribute line back to plain, run the gate, `cp` back a saved-good copy of `project.rs`, `diff` confirmed a clean restore:

- Revert `:5829` (`create_work_from_file`) → names `pub fn create_work_from_file(\n app: tauri::AppHandle`, `FAILED. 0 passed; 1 failed`.
- Revert `:5938` (`preview_import_encoding_from_file`) → names `pub fn preview_import_encoding_from_file(\n app: tauri::AppHandle`, `FAILED. 0 passed; 1 failed`.
- Revert `:5997` (`confirm_import_with_encoding`) → names `pub fn confirm_import_with_encoding(\n app: tauri::AppHandle`, `FAILED. 0 passed; 1 failed`.
- Revert `:6202` (`confirm_bilingual_import`) → names `pub fn confirm_bilingual_import(\n app: tauri::AppHandle`, `FAILED. 0 passed; 1 failed`.

Each of the four names exactly the one signature reverted, and `diff` against the saved-good copy was empty after every restore. Final re-run after all four restores: `the_blocking_wires_run_off_the_main_thread ... ok`.

**`cargo test --locked` totals, before/after** — measured with `git stash push` isolating the two changed files (`commands/project.rs`, `tests/config_invariants.rs`) on the same otherwise-clean tree, then `git stash pop`:

- BEFORE: `total passed: 1502`, `total failed: 0`, 0 lines matching `test result: FAILED`.
- AFTER: `total passed: 1503`, `total failed: 0`, 0 lines matching `test result: FAILED`.
- Net +1 passed (the new `the_blocking_wires_gate_count_table_covers_every_command_bearing_file` test), 0 regressions.

**Feature-build invariance (AC5)** — `cargo test --locked --features nfr-bench --test config_invariants the_blocking_wires_run_off_the_main_thread the_blocking_wires_gate_reads_more_than_one_file the_blocking_wires_gate_count_table_covers_every_command_bearing_file`: `3 passed; 0 failed` — same verdict as the default-feature run. Expected: the gate is a source-text scan, unaffected by which `cfg` the compiler evaluates.

**`npm run check:gates`**: green, unchanged (Kiểm A–F all OK).

**`.githooks/pre-push`**: run standalone (`sh .githooks/pre-push`), all eleven gates + `test` + `build` + `cargo test` green, 230s. 🔴 Per `AGENTS.md`, this is Ice's macOS only — it says nothing about the Windows half or about UTC; the CI run must still be read before this spec is declared done.

**Manual check re-verified**: `~/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/tauri-macros-2.6.3/src/command/wrapper.rs:264` reads `ExecutionContext::Async if function.sig.asyncness.is_none() => "sync_threadpool",` — ~~confirms a plain `fn` with `(async)` routes to `sync_threadpool`, the premise the whole fix rests on.~~

> 🔵 **FALSE — corrected in place 2026-09-15 (review round 3), per `AGENTS.md:51`.** Reading that string is not evidence about execution. `kind` is consumed at exactly one place, `wrapper.rs:278`, inside `tracing::debug_span!` — it is a **log label and controls nothing**. The executing path is `body_async` (`wrapper.rs:361-396`) → `respond_async_serialized` (`ipc/mod.rs:343`) → `async_runtime::spawn` (`:375`) → `tokio::spawn` on the **multi-threaded runtime** (`async_runtime.rs:103-113`). `spawn_blocking` (`async_runtime.rs:290`) is not on this path. The refutation is finding 1 of the pass-1 table below; this sentence is corrected here as well because a `grep` for `sync_threadpool` lands on **this line**, not on the table twenty lines down, and because `deferred-work.md`'s census of surviving copies is scoped to `src/` and therefore never counted this one. The fix and its whole point survive — the mechanism is a tokio worker thread, not a blocking pool.

**Matrix audit, run by the orchestrator (2026-09-15).** Two matrix rows — *Gate, new command* and *Gate, new file* — had no run behind them in the implementation report, and they are the two rows that justify D1 existing at all. A count table nobody has seen fire is decoration. Both were closed by seeding the actual violation, not by reading the code:

- **Gate, new command.** Appended a compiling `#[tauri::command] pub fn ai4_seeded_violation_probe()` to `commands/pinned.rs` (3 plain → 4). Result: `panicked at tests/config_invariants.rs:1298:9 … 'commands/pinned.rs': dem duoc 4 plain / 0 async, bang D1 khai 3 plain / 0 async`, `left: (4, 0) right: (3, 0)`. Restored from a saved copy; `git diff` on that file empty afterwards.
- **Gate, new file.** Created `src/commands/zz_ai4_probe.rs` carrying one `#[tauri::command]` and no `mod` declaration (so it is never compiled — the gate reads it off disk). Result: `panicked at tests/config_invariants.rs:1339:13 … 'zz_ai4_probe.rs' khai it nhat mot '#[tauri::command...]' nhung KHONG nam trong bang D1`. File deleted.
- **Extra measurement, not asked for by the spec, and the most informative one.** Reverted `(async)` on `reload_url_import_item` — a Story 6.7 precedent that is **not** in `blocking_wire_cases()`. Before this spec that regression had nothing watching it at all. Now **both** tripwires fire: `the_blocking_wires_run_off_the_main_thread` at `:1189` (`left: 5 right: 6`) and the D1 table at `:1298` (`left: (12, 5) right: (11, 6)`). This is the direct evidence that the structural blind spot is closed rather than merely documented.
- **Mutation spot-check.** Independently reverted the wire shell of `confirm_bilingual_import` (`:6202` — note the first `rg` hit, `:3724`, is the inner pure fn, not the shell): the gate named exactly `pub fn confirm_bilingual_import(\n        app: tauri::AppHandle`. Corroborates the four mutation runs reported above.
- **Final state after all seeding:** `cargo test --locked --test config_invariants` → `30 passed; 0 failed; 0 ignored; 0 filtered out`, and the tracked diff is byte-identical to the one captured before seeding began.
- ⚠️ **Process note worth keeping.** The `zz_ai4_probe.rs` deletion silently did not happen on the first attempt (`rm` prompted interactively and the file survived); the diff comparison said "identical" because it only covered tracked files, and only `git status` exposed the leftover. A seeded-violation probe is a file that will happily ride into a commit. Verify removal with `git status`, never with a diff of tracked paths.

## Review Triage Log

### Pass 1 — 2026-09-15, three layers (blind-hunter, edge-case-hunter, verification-gap)

Every verdict below was verified by the orchestrator at the cited location before being graded; probe-based claims were re-run or re-derived from source rather than taken on the reviewer's word.

**Routed to intent_gap (root cause inside `<frozen-after-approval>`):**

| # | Finding | Verdict | Evidence |
|---|---|---|---|
| 1 | The load-bearing mechanism claim is false: a plain `fn` with `(async)` does **not** reach a "sync threadpool" (edge-case, claim) | `high` | Verified in source. `wrapper.rs:263` `let kind = …"sync_threadpool"` is consumed **only** at `:278` inside `tracing::debug_span!`, and only under the `tracing` feature — it controls nothing. Execution goes `body_async` (`:361-396`) → `respond_async_serialized` (`tauri-2.11.5/src/ipc/mod.rs:343`) → `respond_async_serialized_inner` (`:371`) → `async_runtime::spawn` (`:375`) → `tokio::spawn` on the multi-thread runtime (`async_runtime.rs:103-113`). `spawn_blocking` exists (`async_runtime.rs:290`) and is **not** on this path. The sentence is in the frozen Intent, in Verification, and was copied into four new doc-comments. |
| 2 | A new runtime property this change introduces is observed by nothing: the sequential N × 20 s image loop now occupies a tokio **worker** thread, and the four wires can overlap each other where main-thread serialization previously made that impossible (edge-case) | `medium` | Follows from finding 1 once `tokio::spawn` replaces the assumed blocking pool. No test can invoke a wire at all — the crate has no `MockRuntime` (recorded at `project_contract.rs:925`, `ipc_contract.rs:906`). Not mentioned in `deferred-work.md`. |
| 3 | `wire::preview_bilingual_import_from_file` (`project.rs:6094`) is a **fifth** wire with the identical defect and was never put to Ice (verification-gap, edge-case) | `high` | Verified: still plain `#[tauri::command]`; calls `import_bilingual_file` (`core/segment/import.rs:737`) which does `std::fs::metadata` then `std::fs::read` of the whole file under the same `MAX_IMPORT_BYTES` 100 MB ceiling (`:759-765`) — the exact criterion used to justify flipping `preview_import_encoding_from_file`. On the product path (`src/config/project.ts:748`). Root cause is the orchestrator's investigation: it swept only the two names the retro supplied, so D2 was decided on an incomplete list. |
| 4 | The frozen Intent says both confirm commands reach the network transitively; `confirm_bilingual_import` cannot (edge-case, claim) | `medium` | Verified: the Bilingual branch of `pipeline.rs` (from `:986`) builds every `ImportedChapter` with `blocks: None` (`:1011`), and `prepare_chapter_images` matches `&chapter.blocks` and skips `None` (`project.rs:636-637`). So `webimport::fetch` is unreachable from that wire. The false reason is now recorded in three places: the frozen Intent, the new `cases` row's why-column, and the doc-comment at `project.rs:6196`. The wire still does a full pipeline plus disk writes synchronously — the decision to flip stands, only the recorded reason is wrong. |
| 5 | The D1 table records a named safety verdict for 28 plain commands, contradicting the spec's own Never clause (blind-hunter, edge-case) | `medium` | The frozen Never says "The other 55 plain commands are out of scope; this spec does not claim they are safe, it says nothing about them", and D1 rejected option (c) because it "would require 59 sync-safety claims this story cannot measure" — yet D1 as written mandates a reason for every zero-`(async)` file, producing exactly such claims for cleanup 5, config 3, dict 3, pinned 3, segment 14. The `segment.rs` reason is additionally contradicted by that file's own 🔴 comment at `:448-451` (CPU time inside a write closure blocks every other write). The contradiction is between two frozen clauses, so it cannot be resolved below the frozen block. |

**Routed to patch / bad_spec, moot this pass (cascading order — code will be re-derived):**

| # | Finding | Verdict | Evidence |
|---|---|---|---|
| 6 | `count_command_attrs` counts only two exact spellings, so a new command written `#[tauri::command(rename_all = …)]` is neither plain nor async and the table stays green (blind-hunter **and** verification-gap, independently probed in two different files) | `high` | Confirmed by reading the branch: `starts_with("#[tauri::command(async)]")` else `starts_with("#[tauri::command]")`. Both reviewers seeded a compiling parameterized command (in `dict.rs` and in `pinned.rs`) and got 3 passed; the plain spelling in the same file gives red. The gate's own doc-comment claim "một command MỚI … làm cổng đỏ" is therefore false for every parameterized form. The new-file sweep uses the looser prefix, so only the already-classified-file case is holed. |
| 7 | All four new `cases` rows could be deleted and every gate stays green (blind-hunter) | `medium` | The count gates cannot say *which* command is async; only a `cases` row names one. `the_blocking_wires_gate_reads_more_than_one_file` pins `chapter.rs` but not `project.rs`. |
| 8 | 22 `cases` rows against 24 measured `(async)` attributes — `start_url_import` and `reload_url_import_item` have no named row (blind-hunter) | `low` | Arithmetic confirmed (glossary 7 + chapter 5 + library 4 + lifecycle 2 + project 4 = 22). The orchestrator's own probe showed the count tripwire does catch a regression on them, but nothing names them. |
| 9 | Duplicate source of truth: `count_async_attrs` per-file asserts re-declare the same four async numbers the D1 table declares (blind-hunter) | `medium` | Exactly the dual-source drift the `blocking_wire_cases()` refactor was written to kill for the file list. |
| 10 | Two counters where the Code Map said "Reuse; do not write a second counter" (blind-hunter) | `low` | Both are local closures with different matching shapes; neither can call the other. Direct deviation from the spec's Code Map. |
| 11 | Uncounted numbers inside the justification prose: `mod.rs` reason says eleven submodules, and the doc-comment says twelve command-bearing files (blind-hunter, edge-case) | `medium` | Verified: `rg -c '^pub mod ' src/commands/mod.rs` = **10**. And only eleven of the twelve tabled files bear a command — `mod.rs` is in the table precisely because it bears none. `AGENTS.md` names this class ("A count states the SCOPE it measured"). |
| 12 | The accumulated rationale doc-comment moved off the test onto `blocking_wire_cases()`, leaving the test undocumented; the cross-reference at `:1355-1358` now dangles; heading still says "Năm vỏ CHẶN" over a 22-entry array (blind-hunter, edge-case ×2) | `low` | Verified by reading the diff. Also: the appended paragraph has no `///` separator, so rustdoc fuses it to the Story 5.3 text. |
| 13 | `assert!(want_async > 0 \|\| zero_reason.is_some())` reads only the hard-coded table literal, never the tree, so for the table as written it cannot fail (verification-gap, other) | `low` | Correct — it constrains future edits of the table, it does not measure anything. Same for the two totals asserts, which are strictly redundant with the twelve per-file asserts that fire first. |
| 14 | The `want_async > 0` escape means a file with one `(async)` wire never has to justify its plain ones — exempting 25 plain wires, the fifth wire among them (verification-gap) | `medium` | Confirmed by reading `:1305-1309`. This is the mechanism that let finding 3 stay invisible. |
| 15 | A stale `zero_reason` survives if a file later gains its first `(async)` wire (edge-case) | `low` | The assert is one-directional; `(want_async == 0) == zero_reason.is_some()` would close it. |
| 16 | A renamed or deleted tabled path dies with a raw read error instead of a classification message (edge-case) | `low` | Confirmed: `unwrap_or_else(|e| panic!("read {}: {e}"))`. |
| 17 | Stale number 60 lines from the new comments: the untouched precedent at `project.rs:6281` still claims "khuôn đã có **17** tiền lệ" where this change's own gate now measures 24 (blind-hunter) | `low` | Verified against the census. Pre-existing text, but this change makes it measurably wrong in the same file. |
| 18 | `lib.rs` prose cites `lib.rs:590` by line number twice where the symbol name is available (blind-hunter) | `low` | Line numbers drift; this session already hit a stale `lib.rs:479` citation in `deferred-work.md`. |

**Routed to defer (pre-existing, not caused by this change):**

| # | Finding | Verdict | Evidence |
|---|---|---|---|
| 19 | `all_src_rust_files()` skips symlinks silently, so a command-bearing file reached through one is never scanned (edge-case) | `low` | Pre-existing at `:783-789`, copied from `glossary_boundary.rs`. Real but not introduced here. |
| 20 | Neither counter strips `/* */` block comments or raw-string bodies before matching (edge-case) | `low` | Pre-existing shape; `AGENTS.md:68` already names the commented-line half of this trap. No occurrence exists in the tree today. |
| 21 | A new file using `use tauri::command;` then bare `#[command]` passes the new-file sweep (edge-case) | `low` | Verified there is **no** bare `#[command]` anywhere in `src-tauri/src` today — hypothetical, not a present defect. |
| 22 | No Rust test in this repo can invoke a wire; every assertion here is a source-text scan (verification-gap, other) | `medium` (unverified as harm) | Already recorded at four places in `tests/**`. Pre-existing and owned. The concurrency half of it is new and is captured as finding 2 instead. |
| 23 | The pre-existing `glossary.rs:1301-1302` doc-comment makes the same false `sync_threadpool` claim as finding 1, and so does `project.rs:6264-6266` | `medium` | Verified. Not caused by this change — the claim entered the repo with Story 3.10b — but finding 1 means the repo now has three more copies of it. |

**Rejected:**

| # | Finding | Verdict | Refutation |
|---|---|---|---|
| 24 | "The new D1 test has never been observed failing; no red counter-check exists for it" (blind-hunter) | `false` | Disproved by the orchestrator's matrix audit recorded in Implementation Notes above, run before this review: both the *new command* probe (`pinned.rs`, red at `:1298`, `left: (4, 0) right: (3, 0)`) and the *new file* probe (`zz_ai4_probe.rs`, red at `:1339`) were seeded and observed red. The reviewer read only the diff, which does not contain those notes. The variant-spelling half of the same finding is real and is logged separately as finding 6. |
| 25 | "Route the blocking body through `spawn_blocking`, or gate invocations with an in-flight `AtomicBool`" (edge-case, guard suggestion) | rejected as a fix | The underlying observation is real and kept as finding 2; the proposed remedy is a design change to the frozen approach, not a patch, so it cannot be applied at this layer. It becomes an option for Ice in the loopback. |

