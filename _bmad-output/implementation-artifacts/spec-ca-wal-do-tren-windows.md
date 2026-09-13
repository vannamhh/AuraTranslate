---
title: 'store_contract WAL ceiling drifts with schema size, not with writes'
type: 'bugfix'
created: '2026-09-13'
status: 'done'
route: 'dispatch'
review_loop_iteration: 1
baseline_commit: 'bd1566172a995476d971032c3cfb2607de68c181'
context: []
---

<frozen-after-approval reason="human-owned intent — do not modify unless human renegotiates">

## Intent

**Problem:** `store_contract::the_wal_stops_growing_once_it_crosses_the_threshold` has failed on `windows-2025` every CI run since 2026-09-03 — the only reason the whole `check` job is red. It is **not** a Windows problem and **not** a checkpoint regression. Measured 2026-09-13: the Windows peak is byte-identical across seven nightlies (984.712 B, 75,1%), a deterministic step from the 889.952 B (67,9%) the ceiling was calibrated on — and macOS stepped too, 94.792 B → **189.552 B**. In WAL frames (4.120 B each) both gained **exactly +23** (macOS 23→46, Windows 216→239): one common, platform-independent cause. The mechanism is healthy on both — `after_first == after_second` to the byte, growth 0 B, `threshold_triggered: 51`, `errors: 0`.

The defect is in the assertion, not the product. Proposition 2 bounds the peak as a fraction of **bytes written**, but what moved is the **schema's page count**; `written` is pinned at `2 * ROUNDS * BLOB`, so every migration that adds pages lifts the peak by a constant and the fraction drifts upward forever. Windows crossed first only because it starts higher. macOS is next: its headroom went from 7,2% to 14,5% against a 25% ceiling, so one more step of this size turns that half red too.

**Approach:** Make proposition 2 true-by-construction instead of re-calibrated. **Decision (Ice, 2026-09-13): self-calibrate inside the run** — write one round, capture the peak, write the remaining rounds, and assert the peak did not rise. That is immune to both `ROUNDS` and schema size, and it still catches a mechanism that reacts very late, because the first-round peak becomes the baseline rather than a hard-coded number. Confirm the +23 frames is schema growth before changing the assertion.

## Boundaries & Constraints

**Always:**
- Confirm the cause of +23 frames before touching the assertion — the expected cause is schema/migration page growth between 2026-08-11 and 2026-09-05, and it must be named with evidence, not assumed.
- Keep the diagnostic `println!` and keep both assertion messages pointing at the two numbers that separate *a Store regression* from *a calibration artefact*; they are why this diagnosis was possible at all.
- Any number that survives must state its measurement, its date, and its population size in a comment next to it.
- Record both 2026-09-13 measurements (macOS local, Windows CI × 7) wherever a number changes.
- 🔴 **A strict "the peak did not rise at all" may be red on `macos-26` CI, and today's data does not refute that.** `store_contract.rs:735` records that runner growing by **115.360 B** after `after_first`, while Ice's own macOS gave 0 B on the identical code. Today's three measurements all show growth 0, but their baseline is captured at the mechanism's first reaction, not after a completed round — a different point, so they do not settle the question. Choose the baseline capture point so the peak has demonstrably stabilised, and if a tolerance is needed, derive it from a measurement and name the runner it came from. Do not assume the `macos-26` number is stale just because it is older.

**Never:**
- Do not raise a ceiling just far enough to clear 984.712 B. That is the reflex the test's own message forbids, it repeats the n=1 mistake at n=2, and it leaves macOS to fail next.
- Do not make the assertion `#[cfg(windows)]`-only or delete it outright to reach green — Ice rejected that shape at the Story 1.21 review.
- Do not change `src-tauri/src/core/store` behaviour: the mechanism is measured healthy on both platforms. This is a test-contract change.
- Do not change `THRESHOLD`, `ROUNDS` or `BLOB` to move the percentage — that makes the number green without changing what is true.

Proposition 2 must keep catching the case proposition 1 alone misses: a mechanism that reacts **very late**, where `after_first` is already at the peak so growth is 0. A high but flat peak is allowed (Windows never rewinds the WAL); an unbounded one is not.

</frozen-after-approval>

## Code Map

- `src-tauri/tests/store_contract.rs:589` -- the test. `THRESHOLD` 64 KiB (:591), `ROUNDS` 20 (:592), `BLOB` 32 KiB (:593).
- `:691-695` -- proposition 2: `WAL_CEILING_NUM` = 3 on Windows, 1 elsewhere, over `WAL_CEILING_DEN` = 4; `written = 2 * ROUNDS * BLOB` = 1.310.720 B; ceiling = 983.040 B on Windows, 327.680 B on macOS. **This is the failing assertion (`:708-717`).**
- `:686-690` -- the comment that predicted this exact run ("Nếu một lượt CI sau vượt 75%, ĐỪNG nới tiếp theo phản xạ") and names debt A5 as the real close. Honour it.
- `:719-757` -- proposition 1, the growth bound (`GROWTH_NUM/DEN` = 1/4 of `round_bytes`). Passes with growth = 0 B on both platforms. `:727-731` records why the older self-referential form was wrong — do not reintroduce that shape.
- `:636` -- proposition 0: a checkpoint ran before the threshold was crossed. Green.
- `src-tauri/src/core/store` -- the mechanism under test. Measured healthy; out of scope per §Never.
- `PROJECT_MIGRATIONS` (search `src-tauri/src/core/store` for the migration list) -- prime suspect for the +23 pages; compare the list at `f5feca0` against 2026-08-11 to name the migrations Epic 5/6 added.
- `_bmad-output/implementation-artifacts/deferred-work.md` -- the owned entries for this test: the 2026-08-11 Ice decision ("AC5 nói CHỮNG LẠI, không nói có trần tuyệt đối"), the n=1 calibration warning, and debt A5. Update them rather than writing a fourth account.

## Tasks & Acceptance

**Execution:**
- [x] `src-tauri/src/core/store` (read-only) -- identify which migrations added pages between 2026-08-11 and 2026-09-05 and confirm they account for ~23 pages -- the assertion must not change on an unnamed cause
- [x] `src-tauri/tests/store_contract.rs` -- **(amended 2026-09-13 after review round 1 — see §Spec Change Log)** split proposition 2 onto two axes: **2a** `after_second <= after_first`, tolerance exactly 0, no external constant; **2b** `(after_first - before_writes) < written * NUM/DEN`, platform ceilings unchanged, which is the only clause that catches a very-late-reacting mechanism. Neither clause may carry a tolerance that grows with schema size except 2b's subtracted baseline, and that cost must be recorded as an owned debt. Keep a comment carrying the 2026-09-13 numbers and why the fraction-of-written form was retired
- [x] `src-tauri/tests/store_contract.rs` -- extend the assertion message so a future failure says which of the two causes it is, using the numbers already printed -- this is what made today's diagnosis possible
- [x] `_bmad-output/implementation-artifacts/deferred-work.md` -- close or amend the n=1 calibration entry and debt A5 with the 2026-09-13 measurements (Windows n=7 identical, macOS n=1 local) -- A5 asked for a Windows measurement and CI has been printing one nightly

**Acceptance Criteria:**
- Given a local macOS run of `cargo test --locked --test store_contract the_wal_stops_growing -- --nocapture`, when it completes, then it passes and prints the peak.
- Given the reshaped assertion applied to today's measured numbers (macOS 189.552 B, Windows 984.712 B, both with growth 0 B), when evaluated, then both platforms pass.
- Given a simulated "no mechanism" case (WAL grows with volume, growth ≈ `ROUND_BYTES`), when evaluated against the reshaped assertion, then it still fails — proven by a seeded check, not by argument.
- Given a simulated "reacts very late" case (`after_first` already at the peak, growth 0), when evaluated, then it still fails.
- **(added after review round 1)** Given proposition 2a forced to always pass, when the seeded checks run, then exactly the "no mechanism" case goes red; and given 2b forced to always pass, then exactly the "reacts very late" case goes red — proving the two clauses guard different axes and neither is redundant.
- Given `git diff --stat -- src-tauri/src`, when read, then the output is empty.

## Implementation Notes

**Cause, measured — not inferred from a migration count (test-only, `src-tauri/src` untouched):**
`spec_with`/`StoreSpec::global` in `store_contract.rs` opens `global.db`, so the relevant list is
`GLOBAL_MIGRATIONS` (`src-tauri/src/core/store/schema.rs`), not `PROJECT_MIGRATIONS` as the Code
Map guessed. It went from **3 steps** at `0dae624` (2026-08-11, the commit that calibrated the old
ceiling) to **7 steps** at `HEAD` — that is **four** migrations added, not three:
`GLOSSARY_ENTRY_DDL` (v4, Story 3.1 — table + unique index + trigger, the heaviest of the four),
`GLOSSARY_ENTRY_ADD_FILE_IMPORT_ORIGIN_DDL` (v5, Story 3.10),
`LIBRARY_ORPHAN_DDL` (v6, Story 5.3), `IMPORT_CLEANUP_RULE_DDL` (v7, Story 6.5).

**The measurement (macOS, Ice's machine, 2026-09-13, n = 1 per configuration):** trimming
`GLOBAL_MIGRATIONS` back to the 3-step set of `0dae624` and re-running this test gives
`before_writes` = **53.592 B** and a peak of **94.792 B** — reproducing the 2026-08-11 macOS
number byte for byte. Restoring the 7-step set of `HEAD` gives `before_writes` = **148.352 B**.
Difference: **148.352 − 53.592 = 94.760 B = exactly 23 frames** (4.120 B/frame). So the whole
step lives in the schema baseline, paid once at `Store::open` before the test writes a byte —
which is why it is identical on both platforms. The measurement lumps all four steps together
and cannot split the 23 frames between them; it does not need to, because the fix subtracts the
whole baseline.

**Reshape — two propositions, two axes (Ice, 2026-09-13; see §Spec Change Log for why a single
clause could not carry both):**

- **2a `wal_peak_did_not_rise(after_first, after_second)`** — `after_second <= after_first`,
  tolerance exactly 0. It is self-referential inside one run, so it needs no external denominator
  and leaves nowhere for schema drift to lodge. The `1/4` constant of the 2026-08-19 version is
  retired. Evidence for zero tolerance: `windows-2025` CI gave `after_first == after_second` byte
  for byte on all seven nightlies, and macOS here gives the same at every capture point (n = 3).
- **2b `wal_ceiling_holds(before_writes, after_first, …)`** — `(after_first - before_writes)`
  against a fraction of `written`, platform ceilings unchanged. It reads **`after_first`**, not
  `after_second`: this is the clause that catches a mechanism reacting so late that `after_first`
  is already at the peak and 2a sees zero growth.

⚠️ **The cost, confined to 2b and signed off rather than hidden:** subtracting the baseline raises
2b's red threshold by exactly `before_writes` (148.352 B on macOS today), and that widening grows
with every future migration. The drift is not eliminated — it moves from a false red into a blind
range. 2a, in exchange, is completely drift-free. Recorded as an owned debt in `deferred-work.md`.

Both propositions are extracted to standalone functions so AC3/AC4 could be proven by a seeded
check: `wal_ceiling_still_catches_a_missing_checkpoint_mechanism` and
`wal_ceiling_still_catches_a_mechanism_that_reacts_very_late` feed hand-built byte values (no
real `Store`). The late-reacting case deliberately uses `written` (not `round_bytes`) as the
simulated peak — a bound anchored at `round_bytes` would pass the loosest platform ceiling
(Windows 3/4) even when the mechanism never ran once, so the seeded value has to exceed every
platform's ceiling to prove the claim on both `cfg(windows)` and not.

`before_writes`/`after_first`/`after_second` are now captured by `settled_wal_len` (polls
`checkpoint_stats().frames_checkpointed` until stable across three 15 ms reads, deadline
500 ms) instead of a blind `sleep(100ms)` — the spec's boundaries flagged that the old capture
point does not prove the peak had settled.

**Measured, macOS, Ice's machine, 2026-09-13 (n = 2 runs re-measured during verification,
byte-identical; the implementing agent reported 4 more, not independently re-counted):** `before_writes = after_first = after_second = 148.352 B` (36 frames), growth 0 B
both ways. That is lower than the 189.552 B the spec's Intent recorded for "today", and the two
numbers come from **different capture points, not the same run** — so the honest reading is a
timing shift, not a smaller high-water mark:

| migrations | old capture (`sleep(100ms)`) | new capture (`settled_wal_len`) |
| --- | --- | --- |
| 3 steps (`0dae624`) | 94.792 B (2026-08-11) | `before_writes` 53.592 B → peak **94.792 B** |
| 7 steps (`HEAD`) | 189.552 B | `before_writes` 148.352 B → peak **148.352 B** |

Under the 3-step schema the new capture reproduces the old number byte for byte (53.592 + 41.200).
Under the 7-step schema the same 41.200 B write-phase increment does **not** appear. A WAL file
cannot shrink without a TRUNCATE checkpoint, and none runs here — so this is not the same run
measured later; the extra settling time before the write rounds changes the checkpoint
interleaving enough that SQLite rewinds and reuses frames instead of appending. ⚠️ **Consequence
worth a reviewer's eye:** on macOS the reshaped proposition 2 now evaluates `grown = 0` against a
327.680 B ceiling — full headroom, so the integration run no longer exercises 2b at any realistic
distance. 2b is exercised by its seeded test instead. Ice was shown this trade with the arithmetic
and accepted it, confined to 2b (see §Spec Change Log); 2a carries no tolerance at all.

The Windows CI number (984.712 B, n = 7 nightlies, byte-identical) was measured against the **old**
code and says nothing about the reshaped test — CI has not run it yet.

**Not done / left to CI:** the "next `check (windows-2025)` must be green" verification step is
unverified until this change merges and CI runs — see §Verification below.

## Spec Change Log

### 2026-09-13 — review round 1, `intent_gap` resolved by Ice

**Triggering finding.** Triage rows 1–3: the delivered proposition 2 was baselined at a
pre-write schema snapshot, not at "the peak after round one" as the frozen Approach states, and
the subtraction widened the red threshold by exactly `before_writes` — a widening that grows with
every future migration. The drift the spec set out to kill was relocated, not removed.

**What was measured before asking.** Both readings of the frozen Approach's "one round" were
probed on Ice's machine (3 runs, byte-identical): `nền 148.352 · sau 1 blob 148.352 · sau đợt một
148.352 · sau đợt hai 148.352`. Reading the numbers together with the recorded Windows peak
(984.712 B, n = 7) showed **neither** reading satisfies the frozen block alone: a one-blob
baseline is red on Windows by ~832 KB (the WAL there grows with write volume through batch one —
"Windows never rewinds the WAL", as the frozen Intent itself says), while a batch-one baseline
collapses proposition 2 into proposition 1 and drops the very-late-reacting case the frozen block
requires it to catch.

**Ice's decision.** Two propositions on two axes: **2a** `after_second <= after_first` with zero
tolerance, and **2b** `(after_first - before_writes) < written * NUM/DEN` keeping the platform
ceilings. The blind range from subtracting the baseline is confined to 2b and accepted as an
owned debt; the "still growing" axis is left completely drift-free.

**Known-bad state avoided.** Re-deriving under either single reading — shipping a Windows-red
assertion, or silently dropping the late-reacting guarantee to reach green.

**KEEP instructions (survived the amendment and must survive any later one).**
- `settled_wal_len` and its documented reason for replacing the blind `sleep(100ms)`.
- The measured cause of the +23 frames (trim `GLOBAL_MIGRATIONS` to 3 steps → 53.592 B; restore
  7 steps → 148.352 B), and the correction that **four** migrations were added, not three.
- Both propositions extracted as pure functions so the seeded checks can drive them directly.
- Both assertion messages naming the numbers that separate a Store regression from a calibration
  artefact, now sourced from the functions' own `Err` strings rather than rebuilt at the call site.

## Review Triage Log

Review round 1 — three layers (blind-hunter, edge-case-hunter, verification-gap), 16 findings.

| # | Finding | Verdict | Evidence |
| --- | --- | --- | --- |
| 1 | Proposition 2 is baselined at a **pre-write schema snapshot**, not at "the peak after round one" as the frozen Approach and Task 2 both state (edge-case, `high` confidence). | `high` | Confirmed at `store_contract.rs:758` — `before_writes` is captured before `write_blobs` runs at all; `after_first` (the real post-round-one peak) is computed but never reaches `wal_ceiling_holds`. |
| 2 | The subtraction widens the bound by exactly `before_writes`, and that widening grows with every future migration (verification-gap, pre-verified). | `high` | Arithmetic re-derived here: old bound tripped at `after_second ≥ 327.680 B`; new bound trips at `≥ 327.680 + 148.352 = 476.032 B` on macOS today. The drift the spec set out to kill is not removed — it moves from a false red into a blind range that keeps growing. |
| 3 | The lost sensitivity never surfaces in the `deferred-work.md` closure entries (blind-hunter). | `medium` | Same root cause as #1/#2; the ledger reads as closed while the trade is still open. |
| 4 | `settled_wal_len` returns `file_len` on deadline expiry with no signal separating "settled" from "gave up" (all three layers). | `medium` | `store_contract.rs:567-581` — the `while` exits on `Instant::now() >= stop` and falls through to `file_len(wal)`. The frozen §Always required the capture point be chosen "so the peak has demonstrably stabilised"; a silent give-up does not demonstrate it. |
| 5 | `deferred-work.md` calls the seven Windows nightlies "kết quả xanh" while the same entry says the case was red on all seven (verification-gap, Other). | `medium` | Verified at `deferred-work.md:1689-1690` against the entry's own opening. Only the measured byte value was identical; the job failed every night. A false statement in the debt ledger. |
| 6 | Closure entries do not carry the spec's own caveat that the reshaped test has never run on `windows-2025` (blind-hunter). | `medium` | Verified: the seven nightlies measured the **old** code. A reader of the ledger alone would think the Windows half is confirmed for the new assertion. |
| 7 | The ceiling is computed twice — inline for the diagnostic message and again inside `wal_ceiling_holds` — so the printed number and the pass/fail decision have separate sources (blind-hunter). | `low` | Confirmed: the `assert!` discards the function's `Err` string via `.is_ok()` and rebuilds the message from the outer copy. A future edit to one formula desyncs the other. |
| 8 | "mười ngày" sits next to "n = 7" over an 11-calendar-day window with no reconciliation (blind-hunter). | `low` | Verified in the new `deferred-work.md` entry. |
| 9 | "*xem dưới*" in `deferred-work.md` points at the reshaped macOS number, which lives only in the spec file (blind-hunter). | `low` | Verified: nothing below it in that file supplies 148.352 B. |
| 10 | A checkpoint tick can mutate `.db-wal` between the last stable poll and the `file_len` read (edge-case). | `low` | Real but negligible at current margins (`grown = 0` against a 327.680 B ceiling); the fix adds a re-read loop, i.e. complexity for no demonstrated harm. **Rejected.** |
| 11 | `file_len` swallows a `metadata` error as `0` (edge-case). | `low` | Confirmed at `store_contract.rs:82-84` — but the helper is **untouched by this diff** and pre-dates it, and for `before_writes` a `0` makes the assertion *stricter* (a loud false red), not the silent skew the finding describes. **Deferred as pre-existing.** |
| 12 | No CI run IDs for the seven nightlies, breaking the file's own citation convention (blind-hunter). | `low` | Convention verified — prior CI observations in the same file cite `32212786258`, `31469843146`, `32438371572`. The run IDs were not available in this session. **Deferred.** |
| 13 | Code Map's "compare the list at `f5feca0`" sends a reader to an unrelated `docs(agents)` commit (blind-hunter). | `false` | `git diff f5feca0 bd15661 -- core/store/schema.rs` is empty: `f5feca0` is used as a "current tree" marker and the comparison reproduces correctly. |
| 14 | Code Map's `PROJECT_MIGRATIONS` guess is "permanently frozen" in the Intent (blind-hunter). | `false` | The frozen block closes at line 39; Code Map starts at line 41 — it is outside. The guess is real but already corrected in §Implementation Notes, and the remaining fix edits this build's spec. **Rejected.** |
| 15 | Intent says macOS "headroom went from 7,2% to 14,5%", which describes rising usage, not headroom (blind-hunter). | `low` | The reading is right — headroom against a 25% ceiling went 17,8% → 10,5%. But the sentence is inside `<frozen-after-approval>` and the fix edits this build's spec. **Rejected.** |
| 16 | Frontmatter `context: []` despite heavy cross-references (blind-hunter). | `low` | Fix edits this build's spec frontmatter. **Rejected.** |

**Grouping and routing.** Rows 1–3 share one root cause — the baseline capture point — and carry the
highest verdict, `high`. Root cause sits inside `<frozen-after-approval>`: the Approach names a
capture point ("write one round, capture the peak … assert the peak did not rise") that the
implementation did not use, while the §Always bullet conditions the strict form on a `macos-26`
measurement that cannot be taken locally and that today's data explicitly "does not refute".
Two readings survive, so intent cannot be inferred → **intent_gap**, which halts for the human.
Rows 4–9 route below it and are moot until the loopback resolves, since the code will be
re-derived. Rows 11–12 → **defer**. Rows 10, 13–16 → **rejected** on the evidence above.

## Verification

**Commands:**
- `cargo test --locked --manifest-path src-tauri/Cargo.toml --test store_contract -- --nocapture`
  — ran 2026-09-13: **19 passed, 0 failed** (17 original + 2 new seeded cases; the task list's
  "17 passed" no longer applies now that AC3/AC4 are separate `#[test]` functions). WAL line
  after the two-axis rebuild:
  `WAL: nền 148352 B (trước khi ghi) -> 148352 B sau đợt một -> 148352 B sau đợt hai · tổng đã
  ghi 1310720 B · [2b] đợt một lớn thêm 0 B kể từ nền (trần 327680 B = 1/4, 0.0%) · [2a] đợt hai
  đẩy đỉnh thêm 0 B (dung sai 0)`.
- `cargo test --locked --manifest-path src-tauri/Cargo.toml` — ran 2026-09-13: whole Rust suite
  green (unit + `tests/*` + doctests), exit code 0.
- `bash .githooks/pre-push` re-run after the two-axis rebuild — 14/14 gates green in 186 s.
- `git diff --stat -- src-tauri/src` — ran 2026-09-13: empty, confirmed.
- `bash .githooks/pre-push` — ran 2026-09-13: all fourteen gates green in 219 s (eleven
  `check:*` gates, vitest, `npm run build`, `cargo test --locked`).
- **Red-side counter-check, per proposition, run independently (not the implementing agent's own
  report)** — this is what proves the two clauses are not redundant:
  - forcing **2a** (`wal_peak_did_not_rise`) to always return `Ok` → **only**
    `wal_ceiling_still_catches_a_missing_checkpoint_mechanism` fails (1 passed, 1 failed);
  - forcing **2b** (`wal_ceiling_holds`) to always return `Ok` → **only**
    `wal_ceiling_still_catches_a_mechanism_that_reacts_very_late` fails (1 passed, 1 failed);
  - reverting either mutation restores 19/19. Each seeded test guards exactly one axis.
- **Cause re-measured independently:** trimming `GLOBAL_MIGRATIONS` to the 3-step set of `0dae624`
  and restoring it — see §Implementation Notes for the two baselines and the 94.760 B difference.
  `src-tauri/src` was restored with `git checkout` and re-confirmed empty afterwards.
- `cargo fmt --check` fails across ~100 files including ones this change never touched — that is
  the repo's pre-existing state, and neither CI (`.github/workflows/ci.yml`) nor `.githooks/pre-push`
  runs a formatting gate. Not introduced here, not in scope.
- Windows is verified only by CI. This change has **not** run on `windows-2025` yet — the next
  `check (windows-2025)` after this merges must be read before calling the Windows half fixed;
  until then say it is unverified rather than implying the 7-run history already covers it (that
  history is against the *old* test, not the reshaped one).
