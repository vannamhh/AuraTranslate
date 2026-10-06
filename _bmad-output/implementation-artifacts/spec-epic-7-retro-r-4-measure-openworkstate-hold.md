---
title: 'Epic 7 retro R-4 — measure how long TM-reading paths hold OpenWorkState in the packaged release app'
type: 'chore'
created: '2026-10-05'
status: 'done'
route: 'dispatch'
baseline_commit: 'bc4b41d06fb36bb6b0275a4786ee762155982300'
review_loop_iteration: 0
context:
  - '{project-root}/src-tauri/AGENTS.md'
---

<frozen-after-approval reason="human-owned intent — do not modify unless human renegotiates">

## Intent

**Problem:** Six commands read the whole TM, or look it up once per sentence, while they hold `OpenWorkState` (retro Epic 7 F1.3–F1.5). A typing flush or a confirm can then wait behind them. The six are `ai_translate_batch`, `ai_translate_segment`, `ai_prompt_assemble` (Xem prompt), `read_open_chapter_segments` with the 7-4 prefill, `tm_concordance` and `tm_list_pairs`. None has a measured hold. The ledger items asking for one (`deferred-work.md` 12786, 12805, 12817) are open with `Chủ: Epic 7`, and Chapter load and the per-click TM reload have no item at all.

**Approach:** Measure in the packaged release app built with `nfr-bench`, following the `6-18-ban-do/` harness. A feature-gated observer thread polls `OpenWorkState.try_lock()` and records held windows. An injected probe opens a seeded Work and a Chapter, and invokes each real command 20 times. The numbers and a verdict go into the ledger.

## Boundaries & Constraints

**Always:**
- **Decision (Ice, 2026-10-05, Q1=B):** measure in the packaged app (`--features nfr-bench`), not in-process.
- **Decision (Ice, 2026-10-05, Q2=A):** the OS keychain read under the lock is not timed. It becomes its own structural ledger item with no number.
- **Decision (Ice, 2026-10-05, Q3=A):** under `nfr-bench` only, `core/aiconfig/keychain.rs` `read()` returns a fixed dummy key, so prepare reaches the TM work. Ice's real keychain is never read or written. The AI endpoint is `http://127.0.0.1:1/v1`, so the network half fails fast after the lock is released.
- Population: 100,000 `tm_unit` pairs in each tier (Global `global.db`, Work `.atproj`), deterministic, unique CJK sources of 30–60 chars. The prompt set body contains `{{tm_similar_segments}}`. Batch: 100 sentences. Chapter load: one Chapter of 300 draft segments with distinct sources, 150 of them with an exact TM hit. The run publishes the population and fails if it differs.
- Each number states commit, date, machine, build, machine load, run count and population. Report p50, p95 and max per command over 20 runs.
- Verdict rule (Story 11.8 F-R-3 precedent): p95 hold < 100 ms ⇒ `→ KHÔNG LÀM` with the numbers and a reopen condition. p95 ≥ 100 ms ⇒ `→ 🟡` with the numbers, keep `Chủ: Epic 7`, and bring the numbers to Ice. No fix in this spec.
- **Decision (Ice, 2026-10-06):** the six commands run on a Rust thread (`nfr_bench_e7_start`) through their real wire shells instead of being invoked by the injected probe over IPC, which stalled mid-run three times. The observed guard is the same.
- **Decision (Ice, 2026-10-06):** the numbers taken under machine load are accepted with the ledger's reading of the clean rounds. No idle-machine re-run.
- All new Rust sits behind `#[cfg(feature = "nfr-bench")]`, including every registration line (`config_invariants.rs:669`). The six measured commands are not edited.

**Never:**
- No change to guard scopes, no `Instant` hooks in the measured commands, no new dependency, no listening port.
- Never use a HOME outside the `bench_home` allowlist, and never commit `.db` files (AD-25).

</frozen-after-approval>

## Code Map

- `src-tauri/src/lib.rs:278-~700` `mod nfr_bench` -- add the lock watcher (start/stop commands, a thread polling `try_lock` every ~0.5 ms; `TryLockError::Poisoned` counts as unlocked) next to `nfr_bench_mark_and_wait_phase` (:591). Register at :990 behind the cfg. Markers go to `global.db` `config_value` under the `__nfr_bench_*__` allowlist (16 KB cap).
- `src-tauri/src/core/aiconfig/keychain.rs:38-42` `read()` -- add the cfg-gated dummy return.
- `src-tauri/tests/config_invariants.rs:669`, `tests/nfr_bench_wiring_contract.rs`, `tests/ipc_argument_contract.rs` -- extend for the new commands and the keychain stub (assert the stub is cfg-gated).
- `_bmad-output/implementation-artifacts/6-18-ban-do/{build.sh,run.sh,probe.js}` -- copy to a new `e7-r4-ban-do/`. `run.sh:54-62` sets up the bench HOME, `:119-216` seeds through ignored `bench-release` builder tests, `:281` `wait_marker`, `:316` launch. probe.js uses `window.__TAURI_INTERNALS__.invoke`.
- `src-tauri/tests/story_6_18_library.rs` -- builder pattern for writing `.atproj` into `$HOME/Documents/AuraTranslate`. `tests/tm_wire.rs:39` `seed_tm` -- TM seeding pattern.
- Commands driven by the probe:
  - Setup: `ai_config_save_field` (`commands/aiconfig.rs:286`), `prompt_set_create` (`commands/promptset.rs:632`), `open_chapter` (`commands/chapter.rs:1280`). The Work is opened by `nfr_bench` at `usable`.
  - Measured: `aitranslate.rs:896` `ai_translate_batch`, `:798` `ai_translate_segment`, `aiprompt.rs:576` `ai_prompt_assemble`, `segment.rs:4108` `read_open_chapter_segments`, `segment.rs:4446` `tm_concordance`, `tm.rs:648` `tm_list_pairs`.
- `deferred-work.md` 12786-12788 (Concordance), 12805-12807 (`tm_list_pairs`), 12817-12819 (7-11 batch) -- find each by its `source_spec`/summary line, not by line number.

## Tasks & Acceptance

**Execution:**
- [x] `src-tauri/src/lib.rs`, `core/aiconfig/keychain.rs` -- add the lock watcher and the keychain stub, all cfg-gated.
- [x] `src-tauri/tests/config_invariants.rs` (+ the wiring contracts) -- guard that both stay out of a default build.
- [x] `src-tauri/tests/story_e7_r4_population.rs` -- ignored builder: one Work with the Chapter and 100k Work-tier pairs, plus 100k Global pairs. It prints and asserts the population.
- [x] `_bmad-output/implementation-artifacts/e7-r4-ban-do/` -- build.sh, run.sh, probe.js: set AI config and the prompt set, open the Chapter, run 20 rounds of each command inside watch start/stop, write results as markers.
- [x] `deferred-work.md` -- append verdict lines to the three items. Add items for Chapter load 7-4, the single-translate/Xem-prompt TM reload, and the keychain read under the lock.
- [x] `sprint-status.yaml` -- `epic-7-retro-item-83` → `done`.

**Acceptance Criteria:**
- Given a default build, when `config_invariants` runs, then the watcher commands and the keychain stub are absent.
- Given the seeded bench HOME, when run.sh finishes, then it prints the population and p50/p95/max for all six commands, and it exits non-zero if any command recorded zero held windows or a TM-less prompt.
- Given the watcher, when the probe holds no lock (a control round that invokes nothing), then the watcher reports no held window.
- Given the numbers, when the ledger is updated, then every touched item carries a verdict by the rule and `npm run check:debt-owner` passes.

## Implementation Notes

- The keychain stub reaches both AI paths: they call only `keychain::read()` (`aitranslate.rs:292`, `:410`).
- A round's hold is its longest single held window, which is what a waiting flush sees. The sum of windows is reported next to it.
- The first prefill `read_open_chapter_segments` writes the 150 hits, and later rounds only look up. Report round 1 (lookup + write) on its own line, separate from rounds 2–20 (lookup only).
- Phase 2: the six commands run on a Rust thread (`nfr_bench_e7_start`, through their real wire shells); `probe.js` only opens the Work and starts it. A webview-driven loop stalled mid-run (round 6 of `tm_concordance`, three times) with the webview, Rust and every tokio worker idle. The first complete run is `ALL_CHECKS_PASSED`.
- The lock watcher spins on `yield_now`: a 0.5 ms sleep gave poll gaps of 3 to 5 s under load, so a round's hold exceeded its own wall time. `summarize.mjs` now fails any round whose hold exceeds its call.
- Population has no near-match TM pair (`similar_segments` = 0), so the TM block of the prompt is empty; the scan runs, its injection is not measured.
- The final run's app binary matches `release_app_sha256` and was built after the last `lib.rs` edit, so it is the spin watcher. `run.sh` had hard-coded "0.5 ms sleep" into `environment.txt`; both now say `yield_now spin`. `working_diff_sha256` is the tree at run time, before the review-time edits to comments, labels and ledger text.
- Deviation from the frozen Approach: the commands are not invoked by the injected probe over IPC. They run on a Rust thread through the same wire shells, because the webview-driven loop stalled. The guard being observed is the same; IPC is outside the hold either way.
- Load bias: `ai_prompt_assemble` rounds 1–7 hold 588–663 ms (poll gap ≤ 6.7 ms). From round 8 the gap jumps to 38–88 ms and the hold to 1.08–4.65 s. So its p50/p95, and the later batch numbers, measure machine load. Every verdict is ≥ 100 ms either way.

## Review Triage Log

- VG-1 lock_watch_start/stop and lock_watch_* markers never run — low: nothing in the final flow calls them (measure_round spawns the watcher directly) → patch: deleted with their allowlist arm, registrations, guard entries; census 6→4, tree 70→68.
- VG-2 HANDOFF-phase-1.md stale (census 5/69, webview probe) — low: true → patch: file deleted, spec pointer removed.
- VG-3 "53 plain" doc comments in config_invariants.rs — low: already present at baseline `bc4b41d` → defer (ledger item, Chủ: Epic 8).
- VG-4 TM-less check only reads `kind == searched`, injection unmeasured — false as a defect: the AC asks for a TM search, and the ledger lines state `similar_segments` = 0.
- VG-5 working_diff_sha256 predates review edits — low: recorded in Implementation Notes; the hash names the measured tree, which is its purpose → reject.
- BH-1 handoff counts stale — duplicate of VG-2, patched there.
- BH-2 spec claims wiring contracts were extended — false as a defect: `ipc_argument_contract` skips cfg-gated registrations and `nfr_bench_wiring_contract` checks only the `usable` arm; fix would be a spec edit → reject.
- BH-3 guard does not assert `bench_home()?` in each new command — low: code calls it in all three; regression needs a deliberate edit and only bench builds run it; fix adds guard logic → reject.
- BH-4 dead lock_watch commands — duplicate of VG-1.
- BH-5 run.sh progress query reads unwritten keys, wrong substr offset — low: true (offset 19 lands in `atch_`) → patch: query line removed with VG-1.
- BH-6 summarize.mjs never checks outcomes; VERDICT prints `p95_ms=` for the worst value — medium: an early rejection would still yield a window and a verdict → patch: outcome must be `ok` or `provider_unreachable` for the AI pair; label renamed `verdict_ms`.
- BH-7 panic in probe thread hangs until the 1800 s cap — low: run.sh dies with "probe did not finish within the time cap"; fix adds catch_unwind → reject.
- BH-8 numbers taken under load (7.9→10.2), biased prompt/batch figures — medium: `ai_prompt_assemble` rounds 8–20 and the whole batch ran with poll gaps of 21 ms–1.76 s → Ice accepted the numbers with the in-ledger reading (Decision 2026-10-06); verdicts hold either way.
- BH-9 watcher perturbs the lock; holds shorter than a gap are invisible — maybe-false, low at most: `try_lock` success holds for nanoseconds and the thresholds are ≥ 100 ms; settle by a run with the sleep watcher on an idle machine → reject.
- BH-10 narrow population (4-char query, list filters, no near-match) — low: the hold covers reading every pair; filters run after release; the frozen population is what was measured → reject.
- BH-11 ledger condition block repeated five times — low: the frozen constraint says each number states its conditions → reject.
- BH-12 ledger cites latest-run.log, which `*.log` ignores — medium: the pointer dangles after commit (precedent 5-14/6-18 do not track logs either) → patch: pointers now name `raw-result.json`, `environment.txt` and `summarize.mjs`.
- BH-13 incident history in probe.js/run.sh comments; per-phase notes — false for the comments (each states a platform workaround and why); notes are a spec edit → reject.
- BH-14 keychain.rs doc block now documents the stub — low: true → patch: stub got a one-line doc, the block sits on the real `read()` again.
- BH-15 sprint item done while ledger items stay 🟡 — false: R-4's action is to measure and record; the 🟡 lines are its output.
- ECH-1 panic leaves spinning watcher and no marker — duplicate of BH-7.
- ECH-2 done marker over 16 KB swallowed — false: 20 fixed rounds give 9.6 KB, digits are the only growth.
- ECH-3 early rejection passes as a verdict — duplicate of BH-6, patched.
- ECH-4 open_at_stop flag unchecked — low: direct one-line check → patch.
- ECH-5 empty rounds crash summarize — false: Rust pushes exactly 20 rounds or writes `e7_probe_error`, and the length check precedes it.
- ECH-6 details totals unchecked — low: the builder and the probe population checks already fail first → reject.
- ECH-7 adjacent or foreign holds merge into one window — false for the cited case: `ai_prompt_assemble` wall time, timed independently around the call, equals its hold (3.4 s), and no other lock taker runs during the probe.
- ECH-8 pgrep could catch a user's app from the same bundle path — low: the path is `src-tauri/target/release/bundle`; fix adds a guard → reject.
- ECH-9 VERDICT label mislabels p95 — duplicate of BH-6.
- ECH-10 wiring contracts claim — duplicate of BH-2.
- ECH-11 census comment says five while six counted — false: six = `confirm_exit_flush` + five; now four = one + three after VG-1.
- ECH-12 hash cannot reproduce committed tree — duplicate of VG-5.
- ECH-13 spinner contends with the measured command — duplicate of BH-9.

## Verification

**Commands:**
- `bash _bmad-output/implementation-artifacts/e7-r4-ban-do/build.sh && bash _bmad-output/implementation-artifacts/e7-r4-ban-do/run.sh` -- expected: population line plus six result lines.
- Full suite once: the change touches command registration in `lib.rs` (AGENTS.md).
- `npm run check:debt-owner` -- expected: pass.
