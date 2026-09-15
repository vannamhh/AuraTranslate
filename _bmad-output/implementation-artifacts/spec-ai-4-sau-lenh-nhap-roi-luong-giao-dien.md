---
title: 'AI-4 — six import commands leave the UI thread, and the gate that says so stops missing whole files'
type: 'bugfix'
created: '2026-09-15'
status: 'done'
route: 'dispatch'
baseline_commit: '29d10bfce7619ee7894e66fb7355a883984b2f4a'
review_loop_iteration: 1
context:
  - '{project-root}/AGENTS.md'
  - '{project-root}/src-tauri/AGENTS.md'
---

<frozen-after-approval reason="human-owned intent — do not modify unless human renegotiates">

## Intent

**Problem:** Six wires in `commands/project.rs` are declared plain `#[tauri::command]`, so tauri runs their bodies **on the main thread**, where they block the UI event loop. The heaviest is `confirm_import_with_encoding`: it reaches the network transitively through `create_work` → `prepare_chapter_images:975` → `fetch_and_write_one_asset:1402` → `webimport::fetch:1282`, a **sequential** loop in which each image waits up to `REQUEST_TIMEOUT` 20 s (`core/webimport/fetcher.rs:85`), so a URL import with N images on a dead host freezes the window for up to N × 20 s while holding the `PendingImportSourceState` lock. The same failure class was measured on Ice's real window on 2026-08-25 and is documented at `commands/glossary.rs:1295-1309`.

The gate meant to catch exactly this — `config_invariants.rs:987 the_blocking_wires_run_off_the_main_thread` — checks a hand-written 18-entry list naming four files. `project.rs`, the file with the most network calls in the epic, was never on it, and a sibling gate's hand-copied file list has already drifted (it omits `chapter.rs`).

**Approach:** Flip all six attributes to `#[tauri::command(async)]` — no body change, matching the 20 existing precedents. Then close the gate's structural blind spot so that a new command, a new command spelling, or a whole new file cannot go unwatched in silence, and make the two file lists one.

**The mechanism, stated correctly — this replaces a false claim from loopback 1.** `#[tauri::command(async)]` on a plain `fn` makes the macro emit `body_async` (`tauri-macros-2.6.3/src/command/wrapper.rs:361-396`), which wraps the synchronous call in `async move { … }` and hands it to `respond_async_serialized` (`tauri-2.11.5/src/ipc/mod.rs:343`) → `respond_async_serialized_inner` (`:371`) → `async_runtime::spawn` (`:375`) → **`tokio::spawn` on the multi-threaded runtime** (`async_runtime.rs:103-113`). The body therefore runs on a **tokio worker thread**, not on the UI thread — which is what removes the freeze. It is **not** moved to a blocking pool: `spawn_blocking` exists at `async_runtime.rs:290` and is not on this path, and the string `"sync_threadpool"` at `wrapper.rs:264` 🔵 *(2026-09-15: this said `:263`; `:263` is the `let kind = match` line, the string is on `:264`)* is consumed only by `tracing::debug_span!` at `:278`, so it is a log label and controls nothing.

## Decisions (Ice)

Round 1, 2026-09-15:

- **D1 — gate shape: file + count tripwire.** Keep the curated `cases` list and add to it, **plus** one declaration covering every command-bearing file with its measured plain/`(async)` counts, so a new command or a new file turns the gate red until a human classifies it. Rejected: a body-scanning gate — measured green on this very bug, see Design Notes — and a full per-command inversion, which would require dozens of sync-safety claims this story cannot measure.
- **D3 — spec length.** Above the 1,600-token guideline, kept in full deliberately: the bulk is the Code Map and the measurements that stop the implementing agent re-investigating. No exact figure is asserted here — writing a token count into the file changes it.

Round 2, 2026-09-15, after the loopback (see Spec Change Log):

- **D4 — correct the claim, keep the behaviour.** The fix stands as a flip; the mechanism paragraph above is the corrected account and every new doc-comment must match it. `spawn_blocking` was considered and rejected: it would mean editing bodies and diverging from 20 precedents. The new runtime property it leaves open — the 20 s-per-image loop now occupies a tokio worker, and the flipped wires can overlap each other where main-thread serialization previously forbade it — is **owned debt**, recorded, not designed away here.
- **D2 (revised) — scope is six commands**, chosen against a full census of all 59 plain wires rather than the two names the retro supplied: the original four, plus `preview_bilingual_import_from_file` (same 100 MB whole-file read) and `create_work_from_text` (whole-Library `reindex_library`, the immediate sibling of `create_work_from_file`). The other 8 flagged wires become owned debt.
- **D5 — the D1 reason column is a declaration, not a verdict.** A zero-`(async)` file's note records an **owned, unmeasured** claim pending measurement, never a safety finding. The `segment.rs` note is removed outright: that file's own 🔴 comment at `:448-451` contradicts it.

## Boundaries & Constraints

**Always:**
- **Order is load-bearing.** Flipping six commands changes `project.rs` from 15 plain / 2 async to 9 / 8 and the totals from 59/20 to 53/26, so the D1 count table must be written **after** the flip. Sequence: ① add the six `cases` rows while all six attributes are still plain → record **RED**, and the failure must name a *signature*, not a count; ② flip the six attributes → record **GREEN**; ③ only then write the count declaration from counts re-measured on the fixed tree.
- Each flipped wire's doc-comment states the mechanism per D4 and names **what actually blocks in that wire** — see the per-wire table in the Code Map. Do not copy one wire's reason onto another.
- The count declaration must classify every `#[tauri::command…]` line into a known spelling and **fail on an unknown one**, naming the file and line. Matching two exact strings and silently ignoring the rest is the defect found in loopback 1.
- A zero-`(async)` file's note is phrased as an owned unmeasured declaration (D5) and carries an owner.
- `the_blocking_wires_gate_reads_more_than_one_file` derives its file set from the shared `cases`, and pins **both** `chapter.rs` and `project.rs`.
- Every count that appears in prose is counted, not estimated. Loopback 1 shipped "eleven submodules" where there are ten and "twelve command-bearing files" where eleven bear one.
- Test function names stay assertive sentences (`src-tauri/AGENTS.md:18`); `tests/**` keeps its diacritics exemption.

**Never:**
- Do not rewrite any command body, do not make any of them `async fn`, do not introduce `spawn_blocking`, and do not touch the inner `MutexGuard` at `project.rs:3370` / `:3744`. With `(async)` on a plain `fn`, only the arguments must be `Send`; all six take `AppHandle` plus owned values.
- Do not build a gate that scans a command **body** for blocking markers. Measured: the body of `wire::confirm_import_with_encoding` contains zero such markers in code — the one lexical hit is inside a comment — so such a gate is green on this very bug, and `AGENTS.md:68` names the comment-matching half of the trap.
- Do not repair the same false mechanism claim where it already lives in product source. 🔵 *(2026-09-15 — this clause first named two sites, `glossary.rs:1301-1302` and `project.rs:6264-6266`. Counted on the tree: **six sites in three files** — `glossary.rs:1301` and `:1375`, `library.rs:597` and `:643`, `core/webimport/mod.rs:22` and `:43`. `project.rs` carries the string at none of them, so the original clause protected a site that does not exist and left four real ones unnamed.)* All six predate this change and are owned debt of Ice's, listed in `deferred-work.md`.
- Do not widen past the six named commands. This spec says nothing about the safety of the other 53.
- Do not touch `src/`, the frontend, or `core/**`.

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected Output / Behavior | Error Handling |
|----------|--------------|---------------------------|----------------|
| Gate, pre-fix | Six `cases` rows added; all six attrs still plain | FAILS naming an offending **signature**, not a count | N/A — this is the counter-check |
| Gate, post-fix | All six attrs `(async)`, count table written | PASSES | N/A |
| Gate, mutation | Revert exactly one of the six | FAILS naming that one signature | N/A |
| Gate, non-`cases` regression | Revert `(async)` on `reload_url_import_item`, which has no `cases` row | Count tripwire FAILS naming `project.rs` and both numbers | N/A |
| Gate, new command | A new plain `#[tauri::command]` in a classified file | FAILS naming the file and both numbers | N/A |
| Gate, new command, parameterized spelling | A new `#[tauri::command(rename_all = "snake_case")]` in a classified file | FAILS naming the file and the unrecognized attribute line | N/A |
| Gate, new file | A new `src/commands/<new>.rs` declaring a command | FAILS naming it as unclassified | N/A |
| Gate, feature build | Suite run with and without `--features nfr-bench` | Same verdict both ways | N/A |
| Signature drift | A parameter is renamed so the literal signature no longer matches | Panics with "khong tim thay chu ky" | Existing behaviour, keep |

</frozen-after-approval>

## Code Map

**The six wires, with what actually blocks in each** (baseline line numbers; the attribute sits one line above the `pub fn`):

| attr | wire | what blocks | evidence |
|---|---|---|---|
| `:5792` | `create_work_from_text` | whole-Library reindex | → `create_work` → `reindex_library` → `Indexer::rebuild(root)` walks the entire Library root. **Not** network: the Blob shape leaves `blocks: None` (`pipeline.rs:702`), so image fetch is unreachable |
| `:5825` | `create_work_from_file` | 100 MB disk read + `.docx` unzip | → `import_file` → `std::fs::read` under `MAX_IMPORT_BYTES` (`core/segment/import.rs:82`, checked `:684`), then full pipeline and `reindex_library` |
| `:5930` | `preview_import_encoding_from_file` | 100 MB disk read + `.docx` unzip | → `import_file`, same ceiling |
| `:5983` | `confirm_import_with_encoding` | **network**, N × 20 s sequential | → `create_work` → `prepare_chapter_images:975` → `fetch_and_write_one_asset:1402` → `webimport::fetch:1282` |
| `:6080` | `preview_bilingual_import_from_file` | 100 MB disk read | → `import_bilingual_file` (`core/segment/import.rs:737`) → `std::fs::read` at `:763`, same ceiling |
| `:6185` | `confirm_bilingual_import` | full pipeline, bulk segment insert, disk writes | 🔵 **Not** network — loopback 1 recorded that falsely. The bilingual branch builds every `ImportedChapter` with `blocks: None` (`pipeline.rs:1011`, branch from `:986`) and `prepare_chapter_images` skips those (`project.rs:1011`, inside the fn that starts at `:975` — 🔵 the first draft of this row said `:636-637`, which is a different loop inside `create_work` gated on `weave_this_import`; the wrong number reached a doc-comment and a gate row before review caught it), so `webimport::fetch` is unreachable from here |

- `src-tauri/src/commands/project.rs:6264-6266` — the precedent comment for why `(async)`; `:6267` / `:6309` are the two already-converted wires. ⚠️ Its own text repeats the false `sync_threadpool` claim and an out-of-date "17 tiền lệ"; copy its *shape*, not its words.
- `src-tauri/tests/config_invariants.rs:987` — the gate. `cases: [(&str,&str,&str); 18]` at `:990-1087`; scan loop `:1089-1112`; per-file async-count asserts at `:1134`/`:1141`/`:1148` (glossary 7, library 4, chapter 5; `lifecycle.rs` has none). Row shape: `(relative path, literal signature prefix including exact newlines and indentation, why-it-blocks prose)`.
- `src-tauri/tests/config_invariants.rs:1124-1132` — `count_async_attrs`, line-based, skips `//`. One counter only: if the new table needs plain counts too, widen this into a free `fn` returning `(plain, async)` and have both call sites use it. Loopback 1 created a second, differently-shaped closure — do not repeat that.
- `src-tauri/tests/config_invariants.rs:1164-1171` — the sibling gate whose hand-copied list already drifted.
- `src-tauri/tests/config_invariants.rs:777` — `all_src_rust_files()`, an existing recursive walker. Reuse it; three siblings already exist elsewhere, do not add a fourth.
- **Baseline census, counted twice:** `chapter.rs` 4/5 · `cleanup.rs` 5/0 · `config.rs` 3/0 · `dict.rs` 3/0 · `glossary.rs` 8/7 · `library.rs` 1/4 · `lifecycle.rs` 1/2 · `mod.rs` 0/0 · `pinned.rs` 3/0 · `project.rs` 15/2 · `segment.rs` 14/0 · `lib.rs` 2/0 = **59 plain / 20 async**. After the flip: `project.rs` 9/8, totals **53/26**. `mod.rs` declares **ten** `pub mod` and bears **no** command; eleven of the twelve tabled files bear one.
- `src-tauri/src/lib.rs:590` — `nfr_bench_mark_and_wait_phase`, one of `lib.rs`'s two commands, exists only under `#[cfg(feature = "nfr-bench")]`. A source-text count counts it in every build; say so in place, by name rather than by line number.
- Not to change: `core/webimport/**`, `prepare_chapter_images`, the image loop. Its missing time budget, progress and cancel are separate owned debt (`deferred-work.md` §*Deferred from: spec-6-11…*, owner Ice) — this spec moves the work off the UI thread, it does not shorten the wait.

## Tasks & Acceptance

**Execution:**
- [x] `src-tauri/tests/config_invariants.rs` — add six `cases` rows for `project.rs`, each carrying that wire's own reason from the Code Map table, while the product is still unfixed; run the gate and record the RED output verbatim — the counter-check must exist before the fix (step ①).
- [x] `src-tauri/src/commands/project.rs` — flip `:5792`, `:5825`, `:5930`, `:5983`, `:6080`, `:6185` to `#[tauri::command(async)]`, each with a doc-comment stating the D4 mechanism and that wire's own blocking reason; no body change (step ②).
- [x] `src-tauri/tests/config_invariants.rs` — widen `count_async_attrs` into one shared counter returning `(plain, async)` that classifies every `#[tauri::command…]` line and **panics on an unrecognized spelling**, naming file and line; use it from both call sites so the four async numbers have a single source (step ③, part 1).
- [x] `src-tauri/tests/config_invariants.rs` — add the count declaration over every command-bearing file, counts re-measured post-flip, zero-`(async)` notes phrased per D5 with an owner and the `segment.rs` note dropped, plus an `all_src_rust_files()` sweep failing on an unclassified command-bearing file (step ③, part 2).
- [x] `src-tauri/tests/config_invariants.rs` — derive the sibling gate's file set from the shared `cases` and pin both `chapter.rs` and `project.rs`; keep the accumulated rationale doc-comment on the test it documents and give the extracted helper only the one-source-of-truth note, with the cross-reference retargeted.
- [x] `src-tauri/tests/config_invariants.rs` — run every matrix row as a seeded violation, including the parameterized spelling and the non-`cases` regression on `reload_url_import_item`; record each verbatim, and verify removal of any probe file with `git status`, not with a diff of tracked paths.
- [x] `_bmad-output/implementation-artifacts/deferred-work.md` — append owned entries for: the tokio-worker occupancy and wire overlap opened by D4; the 8 remaining flagged wires; the false `sync_threadpool` claim surviving in `glossary.rs:1301` and `project.rs:6264`; and the stale "17 tiền lệ" count.

**Acceptance Criteria:**
- Given all six attributes are plain and the six rows are in `cases`, when the gate runs, then it fails naming an offending signature — not a count mismatch.
- Given all six are `(async)` and the table is written, when the full suite runs, then the gate passes and the total test count is no lower than the pre-change baseline.
- Given exactly one of the six is reverted, when the gate runs, then it fails naming that one signature — checked for each of the six.
- Given `(async)` is removed from `reload_url_import_item`, which has no `cases` row, then the count tripwire fails naming `project.rs` and both numbers.
- Given a new plain command, a new parameterized-spelling command, or a new command-bearing file, when the gate runs, then it fails naming that file — and for the unknown spelling, the attribute line.
- Given the suite runs with and without `--features nfr-bench`, then the gate reaches the same verdict both ways.
- Given the sibling gate runs, then its file set is derived from `cases` and contains both `chapter.rs` and `project.rs`.
- Given the whole change, when `git diff` is read, then no file under `src-tauri/src/` other than `commands/project.rs` is touched, that diff is attribute lines plus doc comments only, and every number written in new prose matches a count that was actually run.

## Implementation Notes

Loopback 1's notes and the 25-row pass-1 triage log were moved to
`ai-4-loopback-1-history-2026-09-15.md` (same directory) so this file stays readable; the
conclusions that bind re-derivation are in `## Spec Change Log` below.

### Round 2 — orchestrator's own acceptance measurements (2026-09-15)

Run after the pass-2 patches landed, independently of the implementing agent's report.

- **The new concurrency test can actually go red — counter-checked at the seam.** Narrowed `confirm_bilingual_import`'s guard to read-clone-`drop`-then-`create_work`, with a re-lock to clear. Result: `two_concurrent_bilingual_confirms_…` FAILED with **two** `Ok`s — "Race A.atproj" and "Race B.atproj", two distinct `work_id`s from one pending source (`left: 2, right: 1`). Restored, green. This is a test that guards the wiring, not itself.
- **Citations fixed and verified:** `wrapper.rs:263` gone from the tree (0 hits); `:636-637` survives only inside the sentence explaining why it is the wrong loop.
- **Census re-counted by hand after the patches: 53 plain / 26 async**, matching `COMMAND_FILE_CENSUS`.
- **`fn walk(` under `tests/**` counted: exactly 19**, matching the corrected comment. The earlier "ba bản chép" was inherited from this spec's Code Map and was never counted.
- **Full suite: 1504 passed, 0 failed, 20 ignored** (1502 at baseline; +1 census test, +1 bilingual concurrency test). `check:gates` green. `.githooks/pre-push` green in 165 s.
- **A line number that moved three times inside one session.** The "17 tiền lệ" sentence was cited at `:6338` by review, measured at `:6339` by the implementing agent, and sits at `:6341` after that agent's own later patches. All three were honest measurements of a drifting target. Every `project.rs:NNNN` citation in that debt entry is now a **symbol name** instead.
- ⚠️ **Near-miss worth keeping, the same class the implementing agent reported.** My restore backup was never created: `cd src-tauri && cp …` failed at the `cd` (the shell was already there), and `&&` swallowed the `cp`. The mutation then had no backup, and `git checkout` was not an option because the whole AI-4 change is uncommitted — restoring from HEAD would have deleted it. Reverted by hand instead, then proved the revert by re-deriving the property rather than trusting it: the `project.rs` diff is once again attribute lines and `///` only, and the two remaining `drop(guard)` occurrences were shown absent from the diff's added lines, i.e. pre-existing. **Verify a probe harness's backup exists before mutating, and never restore uncommitted work from HEAD.**

### Round 2 — implementation, 2026-09-15

Re-derived from this spec on `29d10bf`; loopback 1's `.patch` was **not** applied or read for code.

**Step ① — RED, verbatim** (`cargo test --locked --test config_invariants`, six rows added to
`cases` (18 → 24), all six attributes still plain):

```
thread 'the_blocking_wires_run_off_the_main_thread' (144029315) panicked at tests/config_invariants.rs:1152:9:
vo tai `src/commands/project.rs` (chu ky `pub fn create_work_from_text(
        app: tauri::AppHandle`) KHONG mang `#[tauri::command(async)]` -- lenh dong bo chay tren LUONG CHINH, va no CHAN o day: quet TOAN BO goc Library: `reindex_library` -> `Indexer::rebuild(root)` duyet het thu muc goc sau moi luot tao. KHONG phai mang -- hinh Blob de `blocks` rong (`core/segment/pipeline.rs:702`) nen luot tai anh khong voi toi duoc. ⇒ TREO UNG DUNG (do 2026-08-25 tren cua so that, nhanh hop thoai). Dong doc duoc ngay truoc chu ky: "#[tauri::command]"

test result: FAILED. 28 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.04s
```

Names a **signature**, not a count — the `assert!` fires on the first offending row in iteration
order, so one of the six is named even though all six were plain.

**Step ② — GREEN** (same command, after the six flips): `test result: ok. 29 passed; 0 failed`.

**Step ③ — counts re-measured on the FIXED tree, then written.** `project.rs` 15/2 → **9/8**;
totals 59/20 → **53/26** over **eleven** command-bearing files (`commands/mod.rs` declares ten
`pub mod` and bears none). Gate after step ③: `30 passed; 0 failed`.

**Every matrix row run as a seeded violation**, each restored and re-verified afterwards. The
verbatim capture files lived in a session scratchpad and do **not** survive the session; what is
reproducible is the seeding recipe in each row below plus the line the failure fired on.

| Matrix row | Seeded how | Verdict |
|---|---|---|
| Gate, pre-fix | six rows added, six attrs plain | RED at `:1152`, names `create_work_from_text`'s signature |
| Gate, post-fix | — | GREEN, 30 passed |
| Gate, mutation (×6) | revert `(async)` on each of the six, one at a time | RED each time at `:1173`-region, each naming **its own** signature; restored byte-for-byte between runs |
| Gate, non-`cases` regression | revert `(async)` on `reload_url_import_item` | **both** tripwires fire: census at `:1420` `left: (10, 7) right: (9, 8)`, and the gate's negative check at `:1218` `left: 7 right: 8` |
| Gate, new command | compiling `#[tauri::command] pub fn ai4_probe_new_command()` in `project.rs` | RED at `:1420`, `dem duoc 10 plain / 8 (async) … khai 9 plain / 8 (async)` |
| Gate, parameterized spelling | `#[tauri::command(rename_all = "snake_case")]` in `project.rs` | RED at `:1299`, names `src/commands/project.rs:6342` **and the attribute line** — fires in *both* tests, because both use the shared counter |
| Gate, new file | `src/commands/ai4_probe.rs`, one command, no `mod` decl | RED at `:1469`, `["src/commands/ai4_probe.rs (1 plain / 0 async)"]` |
| Gate, feature build | with and without `--features nfr-bench` | identical both ways: `30 passed` green, and `exit=101` naming the same signature when seeded |
| Signature drift | rename the wire's `app` param (plus a body shim so it still compiles) | panics `khong tim thay chu ky …` at `:1173` — existing behaviour, unchanged |
| *(added: finding 7 + the new pin)* | delete all six `project.rs` rows from `blocking_wire_cases()` — a removal at the seam, the source attributes untouched | RED twice: the sibling gate's pin at `:1213` (`KHONG con hang nao … duoc GHIM dich danh`) and the census's rows column at `:1390` (`co 0 hang …, khai 6`). Before this change that deletion was green everywhere |

`git status` after the new-file probe listed only the two intended modifications — no leftover
`ai4_probe.rs`. ⚠️ **The probe harness itself produced the one real scare of this session:** its
first restore step used `git checkout -- src-tauri/src`, which silently reverted the six
*uncommitted* flips it was supposed to be proving. A restore-from-HEAD is not a restore when the
thing under test is not in HEAD. Fixed by restoring from a byte-for-byte backup copy, with an
assert on the expected `(async)` attribute count before every probe run.

**Deviation from the task list, stated rather than buried — one fix, one counted number.**
① The task list says "use [the shared counter] from both call sites so the four async numbers
have a single source". Keeping three literal per-file numbers in the gate *and* the same numbers
in the census table is finding 9 (duplicate counts) reproduced, so the literals now live **only**
in `COMMAND_FILE_CENSUS`; the gate's negative-direction check survives but reads its expected
number from that table. Both tests call `count_command_attrs`.
② The census carries a fourth column — the number of `blocking_wire_cases()` rows for that file —
because finding 7 (a `cases` row is deletable with every gate still green) has no other cure:
deleting a row changes no attribute in the source. For `project.rs` the two columns deliberately
disagree (6 rows / 8 `(async)`), which is exactly the gap the `async` column covers.
③ `tests/config_invariants.rs`'s **own** doc-comment repeated the false `sync_threadpool` claim.
It is not one of the two pre-existing doc-comments the frozen Never protects (both of those are
under `src/`), and `AGENTS.md` requires an expired claim to be fixed in place with 🔵 and a date,
so it was corrected there. The two protected copies under `src/` were left untouched — **and the
count in the spec's own task line is low: the claim survives at FIVE sites in THREE files**
(`glossary.rs:1301` and `:1375`, `library.rs:597` and `:643`, `project.rs:6339`), all five now
recorded as one owned debt entry.

> 🔵 **CORRECTED IN PLACE 2026-09-15 (review round 3), per `AGENTS.md:51` — the paragraph above
> is itself miscounted, in the very sentence complaining that a count was low.** Re-measured with
> `grep -rn sync_threadpool src-tauri/src`: the surviving false-claim sites are **six, in three
> files** — `glossary.rs:1301`, `:1375`, `library.rs:597`, `:643`, **`core/webimport/mod.rs:22`,
> `:43`**. The two `webimport/mod.rs` copies were never listed. And **`project.rs:6339` does not
> exist as a site**: the only `sync_threadpool` in `project.rs` is at `:5802`, and it is the
> *corrected* sentence written by this very change. The frozen Never and the `deferred-work.md`
> entry both already say "six sites in three files" and are right; this paragraph alone was wrong,
> so a reader working from it would have edited a correct comment and left two real ones standing.

**Not done, and why.** The spec's Task line asks for a debt entry naming "the 8 remaining flagged
wires". Neither this spec nor the sidecar names them — the only surviving census is per-*file*,
not per-*wire* — and re-deriving the list would mean judging 53 commands this spec explicitly
says nothing about. The debt entry therefore records the count from frozen D2, records that the
names exist nowhere, and hands the re-run to its owner.

### Round 2 — review fixes, 2026-09-15

Fifteen findings applied. Three are worth keeping beyond the diff:

- **Two of the census/gate claims were measurably false and are now measured.** ① The census
  doc-comment's proof ("remove `(async)` from `reload_url_import_item` ⇒ the other gate stays
  green") does **not** separate the two tests: `project.rs` has `cases` rows, so the first gate's
  negative-direction loop reads that file's number from the census and goes red too. Replaced
  with a proof that does separate them, and run: add one plain command to `commands/cleanup.rs`
  (0 `cases` rows) ⇒ `the_blocking_wires_run_off_the_main_thread ... ok`, census `FAILED` with
  `cleanup.rs dem duoc 6 plain / 0 (async) … khai 5 plain / 0 (async)`. ② The negative-direction
  comment claimed `(async)` must sit on exactly the expected wires; it checks a per-file **count**,
  so moving `(async)` between two `project.rs` wires passes it. Comment now says which of the two
  loops guards identity (the `cases` loop) and which guards quantity.
- **The `(53, 26)` totals assert was circular** — it summed the same per-file numbers asserted
  five lines above, so it could not fail on its own. It now counts independently over
  `all_src_rust_files()`, and that walk gained the `RS_FLOOR_FOR_DIALOG_CHECK` population floor
  its sibling at `:817` already had: without it, a walk returning nothing made the
  unclassified-file check vacuously green — the repo's central "silent emptiness" class.
- **One review finding did not reproduce, and is not applied.** The review placed the
  "17 tiền lệ" sentence at `project.rs:6338` with `:6339` holding the `library.rs:640`
  pointer. Measured on this tree: `grep -n` puts the sentence on **`:6339`** and the pointer on
  `:6340`; the debt entry's original `:6339` was right and stands. The other half of that finding
  did reproduce and is applied — `library.rs:640` is correct (it opens `library_choose_root`'s
  🔴 `(async)` block), so the entry no longer leaves it as an open question.

**Counts corrected to measured values, per AGENTS.md.** "ba bản chép" of the tree walker → **19**
`fn walk(` definitions under `src-tauri/tests/**`. "Năm vỏ CHẶN" → no count (the array has 24 rows
and has been renumbered 2 → 5 → 7 → 18 → 24). "TÁM vỏ còn lại" → **53**, with the old eight kept
in place and re-scoped to `commands/glossary.rs`, which is all it ever measured. `chapter.rs`'s
five `cases` rows → four from Story 5.8 plus `update_chapter_origin` from 6.15.

**New test.** `bilingual_import_contract.rs::two_concurrent_bilingual_confirms_on_the_same_pending_source_produce_exactly_one_work_not_two`
— the twin of the prose sibling at `segment_contract.rs:9012`, same `std::thread::scope` shape.
It exists *because of* this change: before AI-4 the bilingual confirm wire was plain, so tauri
serialized it on the UI thread and two invokes could not overlap; `(async)` makes them overlap for
real, and the `MutexGuard` held from `project.rs:3744` across `create_work` to `*guard = None` at
`:3772` becomes the only thing between them. **Counter-check run:** narrowing that guard to
read-clone-`drop`-then-`create_work` turns it RED with two `Ok`s — two Works from one pending
source — and restoring the guard turns it GREEN.

**Also recorded, from re-measuring the overlap hazard for the debt entry:** three wires that stay
plain still `.lock()` the same `PendingImportSourceState` the `(async)` confirm wires hold across
`create_work` — `preview_import_encoding_from_text` (`:5908`),
`rebuild_bilingual_import_preview` (`:6203`), `remove_url_import_item` (`:6446`). They run on the
UI thread, so a `.lock()` there can now stall for the full N × 20 s. Not reachable through the
product UI: `src/importPreviewState.ts` latches all six call sites on `confirming.value` (`:853`,
`:876`, `:931`, `:974`, `:1023`, `:1383`) — only a raw `internals.invoke` gets past, which the
existing comment at `project.rs:3372-3375` already names. `tier2_block_confirm_range` was checked
and excluded: it locks `UrlImportItemsState` and `Tier2BlockOverridesState`, not this one.

**Verification numbers (all on Ice's macOS, 2026-09-15).**
- `cargo test --locked`: BEFORE **1502 passed / 0 failed / 20 ignored** over 57 result lines,
  exit 0 — measured on a still tree before any edit. AFTER ~~**1503 / 0 / 20**~~, 57 lines, exit 0.
  ~~Net **+1**, the one new test~~; 0 regressions.
  🔵 **CORRECTED 2026-09-15 (review round 3): AFTER is 1504 / 0 / 20, net +2.** Re-measured
  independently: `1504 passed / 0 failed / 20 ignored`, exit 0. The change adds **two** `#[test]`
  functions — `every_command_bearing_file_is_classified_with_measured_attribute_counts` and
  `two_concurrent_bilingual_confirms_…` — so "+1, the one new test" was wrong in both halves.
  §*Round 2 — orchestrator's own acceptance measurements* already said 1504; this line was the
  stale one, and a future regression check starting from 1503 would have begun from a false floor.
  Round 3's patches add asserts to existing tests, not new tests, so the total is still 1504.
- `cargo test --locked --test config_invariants`: 30 passed, and identical under
  `--features nfr-bench`.
- `npm run check:gates`: green, unchanged (Kiểm A–F).
- `.githooks/pre-push`: green, eleven gates + vitest + build + `cargo test`, 161 s.
  🔵 **2026-09-15 (review round 3) — three durations now exist for this hook and they are three
  different runs, not a contradiction: 165 s** (§*Round 2 orchestrator*, post-patch run), **161 s**
  (this line, and again on the round-3 re-run), **178 s** (the push of `f09d79d`). None is a
  baseline for the next story on its own; the hook's cost is machine- and cache-dependent.
  🔴 Ice's macOS only — per `AGENTS.md` it says nothing about Windows or UTC.
  ~~**The CI run has not been read: nothing has been pushed. This spec is not done until it is.**~~
  🔵 **CLOSED 2026-09-15 (review round 3).** `574c869..f09d79d` pushed with Ice's approval; CI run
  **`34959441963` is green on both platforms** — `check (windows-2025)` 30m13s ✓,
  `check (macos-26)` 10m04s ✓. ⚠️ `e2e (macos-26)` ran **0 s**: `.github/workflows/ci.yml:51-54`
  runs that job only on `schedule`/`workflow_dispatch`, never on push — so a green CI here does
  **not** cover the e2e suite. That gap was closed separately by running `npm run test:e2e`
  locally (24/24 spec files, exit 0, 2m27s); see Review Triage Log pass 3, row 1.
  ⚠️ Note for whoever reads `master`'s nightly runs: the four scheduled runs before this one were
  red on `the_wal_stops_growing_once_it_crosses_the_threshold` (`store_contract.rs:890`, macOS).
  That ca did **not** reproduce on either platform at `f09d79d` — it is runner-environment flake
  on a pre-existing story, not a state of `master` and not caused by AI-4.
- `git diff` scope: three files. Under `src-tauri/src/` only `commands/project.rs`, and its diff
  is exactly six `#[tauri::command]` → `#[tauri::command(async)]` lines plus added `///` lines —
  verified by filtering the diff, zero other added or removed lines.
- D4 mechanism re-checked against the downloaded crate sources before any doc-comment was
  written: `wrapper.rs:249` dispatches `ExecutionContext::Async` to `body_async`; `:264`'s
  `"sync_threadpool"` is bound to `kind`, whose sole use is `tracing::debug_span!` at `:278`;
  `body_async` (`:361-396`) calls `respond_async_serialized` (`ipc/mod.rs:343`) →
  `respond_async_serialized_inner` (`:371`) → `async_runtime::spawn` (`:375`) → `tokio::spawn`
  (`async_runtime.rs:103-113`); `default_runtime` (`:222`) builds `TokioRuntime::new()`, the
  multi-threaded runtime. `spawn_blocking` at `:290` is on no part of this path.

## Spec Change Log

### Loopback 1 — intent_gap, 2026-09-15

**Triggering findings:** Review Triage Log pass 1, findings 1, 3, 4, 5 — all rooted inside `<frozen-after-approval>`, so no amendment below the frozen block could fix them.

**What was wrong, in one line each:**
- The frozen Intent's mechanism claim was false. `(async)` on a plain `fn` routes to `tokio::spawn` on the multi-thread runtime, not to a "sync threadpool"; `wrapper.rs:263`'s `"sync_threadpool"` string is consumed only by `tracing::debug_span!` at `:278`. The orchestrator read `body_async` early in the session, saw `respond_async_serialized(async move { … })`, and still wrote the wrong mechanism into the frozen block, from where it was copied into four new doc-comments.
- Scope was decided on an incomplete list. A fifth wire with the identical 100 MB criterion (`preview_bilingual_import_from_file`) was never surfaced, because investigation swept only the two names the retro supplied instead of the whole command surface.
- The frozen Intent claimed both confirm commands reach the network. `confirm_bilingual_import` cannot: the bilingual branch sets `blocks: None` and `prepare_chapter_images` skips those chapters.
- D1 and the frozen Never contradicted each other: Never says the other plain commands are not claimed safe, while D1 mandated a written safety reason for every zero-`(async)` file, producing 28 unmeasured safety verdicts.

**Ice's resolutions (2026-09-15, second round):**
1. Correct the claim, keep the behaviour — the fix still takes the work off the UI thread, which is what the glossary precedent measured on a real window. The new runtime property (a 20 s-per-image loop now occupying a tokio worker; the flipped wires can now overlap) becomes owned debt, not a design change. `spawn_blocking` was considered and rejected: it would require editing bodies and would diverge from the 20 existing precedents.
2. Scope becomes **six** commands — the original four plus `preview_bilingual_import_from_file` (same 100 MB ceiling) and `create_work_from_text` (FULLSCAN through `reindex_library`, the immediate sibling of an already-flipped wire). The other 8 flagged wires become owned debt.
3. The D1 reason column is demoted from a safety verdict to an **owned, unmeasured declaration**; the `segment.rs` reason is removed outright, since that file's own 🔴 comment at `:448-451` contradicts it.
4. Not chosen: repairing the same false claim in the two pre-existing doc-comments (`glossary.rs:1301`, `project.rs:6264`). They entered with Story 3.10b and stay as owned debt.

**Known-bad state this avoids:** shipping three fresh copies of a false mechanism claim into doc-comments that later readers would cite as authority — the repo already has two such copies being cited that way — plus a gate whose own advertised guarantee ("a new command turns it red") is false for any parameterized attribute spelling.

**KEEP — these survived review and must survive re-derivation:**
- The three-step order (rows red → flip → count table) and the evidence discipline around it. The RED run naming a signature rather than a count is exactly right; do not let the count assertions fire first.
- `blocking_wire_cases()` as one source of truth, with `the_blocking_wires_gate_reads_more_than_one_file` deriving its file set from it. This killed a drift that had already happened once.
- The D1 count table's *mechanism* — it demonstrably catches a regression on `reload_url_import_item`, a wire that `cases` does not name and that nothing watched before.
- The seeded-violation probes as the acceptance evidence for the table's two blind-spot rows; reuse them, and add one for the parameterized spelling.
- Reusing `all_src_rust_files()` rather than adding a fourth walker.

**Carried forward as required fixes, not re-litigated:** findings 6 (parameterized-attribute hole in `count_command_attrs`), 7 (`cases` rows deletable with gates still green), 9-11 (duplicate counts, two counters, the `mod.rs` "eleven" that is ten and "twelve command-bearing files" that is eleven), 12 (doc-comment reparenting and the dangling cross-reference), 14 (`want_async > 0` exempting a file's plain wires — the mechanism that hid the fifth wire).

## Review Triage Log

Pass 1 (25 findings, 2026-09-15) is in `ai-4-loopback-1-history-2026-09-15.md`.

### Pass 2 — 2026-09-15, three layers on the re-derived tree

No `intent_gap` and no `bad_spec` this pass, so no loopback: `review_loop_iteration` stays 1. Every verdict was verified by the orchestrator at the cited location. **Two of the confirmed defects originated in this spec's own Code Map**, were copied faithfully into code, and are marked 🔴 below — a faithful copy of a wrong premise is still wrong.

| # | Finding | Verdict | Route | Evidence |
|---|---|---|---|---|
| 1 | 🔴 `project.rs:636-637` cited as where `prepare_chapter_images` skips `blocks: None` — in the doc-comment **and** the `cases` row (blind-hunter, edge-case) | `medium` | patch | Verified: `prepare_chapter_images` starts at `:975`, its skip is `:1011`. `:636-637` is inside `create_work` (from `:354`), a different loop gated on `weave_this_import`. **The wrong number came from this spec's Code Map**, written by the orchestrator from an `rg` hit taken without checking which function it fell in. |
| 2 | 🔴 `wrapper.rs:263` cited as the line holding `"sync_threadpool"`, in three places written by this diff (verification-gap) | `low` | patch | Verified: `:263` is `let kind = match attrs.execution_context {`; the string is `:264`. **Also this spec's error** — and the pre-existing sentence being corrected already had `:264` right. |
| 3 | The census test's own stated counter-proof no longer reproduces (all three layers; verification-gap ran it) | `medium` | patch | Both gates now go red on the `reload_url_import_item` mutation, because the new negative-direction block derives per-file counts from the census and `project.rs` is in `case_files`. A proof that does reproduce was supplied and re-run: duplicating a `#[tauri::command]` line in `cleanup.rs` leaves the first gate green and reds the census test. The gate is sound; the sentence justifying it is not. |
| 4 | The retained anti-scatter comment is no longer enforced (blind-hunter) | `medium` | patch | The old check bound a file's `(async)` count to its number of `cases` rows; the new one binds it to a census number that for `project.rs` is deliberately 8 against 6 rows, so moving `(async)` onto a different `project.rs` wire passes both gates. |
| 5 | Stale counts survive in the moved doc-comment: "Năm vỏ CHẶN", "TÁM vỏ còn lại" (blind-hunter) | `low` | patch | The array it documents has 24 rows; 53 plain wires remain. The addendum updated the history but not the heading. |
| 6 | "ba bản chép của cùng cây quét" is an uncounted number (blind-hunter) | `low` | patch | A count finds at least six (`docx_boundary.rs:34`, `cleanup_boundary.rs:43`, `ai_boundary.rs:104`, `glossary_boundary.rs:271`, `dict_boundary.rs:104`, `:998`). Inherited from this spec's Code Map — the same class the change exists to stop. |
| 7 | The `all_src_rust_files()` sweep has no population floor (blind-hunter) | `medium` | patch | If the walk returns nothing, the unclassified-file check is vacuously green and the per-file asserts would not notice, since they read paths directly. The sibling call site at `:815` guards with `RS_FLOOR_FOR_DIALOG_CHECK`; `glossary_boundary.rs` runs a tree-size check first. |
| 8 | The `(53, 26)` totals assert cannot detect what its message claims (blind-hunter, edge-case) | `low` | patch | It sums the very per-file numbers already asserted above it. A missing file is caught by the sweep, not here. |
| 9 | The `segment.rs` census row carries a prose `why` while the doc-comment says it is deliberately empty and the spec task said to drop it (blind-hunter) | `low` | patch | `""` now encodes two different meanings across rows. |
| 10 | The counting-loop comment claims an unknown spelling panics in "cả hai" gates (edge-case) | `low` | patch | The first gate scans only the 5 files with `cases` rows; a strange spelling in the other six panics only in the census test. |
| 11 | `chapter.rs`'s five `cases` rows attributed wholly to Story 5.8 (edge-case) | `low` | patch | Four came from 5.8; `update_chapter_origin` came from 6.15 — the attribution that was carried by the assert message this diff deleted. |
| 12 | Debt entry 4 cites `:6339` for the "17 tiền lệ" sentence, which is on `:6338`, and flags `library.rs:640` unverified when a grep settles it (blind-hunter) | `low` | patch | Verified both. |
| 13 | No test would fail if `confirm_bilingual_import`'s guard were narrowed, though `(async)` makes overlap newly reachable (verification-gap, pre-verified) | `medium` | patch | The prose sibling has exactly that test (`segment_contract.rs:9012`, two real threads); the ~15 bilingual tests each call the wire once on one thread. Narrowing the guard to read-clone-then-`create_work` leaves all of them green and yields two Works from one pending source. |
| 14 | Three wires that stay plain still `.lock()` the `PendingImportSourceState` the `(async)` confirms hold across `create_work`, so a main-thread `.lock()` can stall N × 20 s (edge-case) | `medium` | defer | Verified the three: `preview_import_encoding_from_text`, `rebuild_bilingual_import_preview`, `remove_url_import_item`. Also verified the reachability limit rather than assuming it — `src/importPreviewState.ts` latches on `confirming` at six call sites (`:853`, `:876`, `:931`, `:974`, `:1023`, `:1383`), so no product-UI path reaches it; only a raw `internals.invoke`, which `project.rs:3372-3375` already says the latch does not stop. Not intent_gap: the intent holds for every path the product can take. Extends debt entry 1 rather than opening a new one. |
| 15 | The negative-direction loop is logically redundant with the census test (blind-hunter) | `low` | rejected | True as stated, but its fix is deleting a second, differently-worded failure message on a gate whose whole history is people missing the first one. The duplication is cheap and the message is not. Rejected under the low-finding rule: no user or developer meets a defect here. |
| 16 | Two unrelated "eights" now coexist — the gate doc-comment's "TÁM vỏ còn lại" and debt entry 2's "8 flagged wires" (blind-hunter) | `low` | rejected | Real but cosmetic; finding 5 already rewrites the doc-comment's eight out of existence, which removes the collision without a cross-reference. |
| 17 | Overlapping preview/confirm can stash a different file than the one on screen; `OpenWorkState` replace-by-completion-order (edge-case ×4) | `medium` (unverified) | defer | All four are the same root: overlap is newly possible and ordering is unspecified. Already owned in this diff's debt entry 1; finding 14 sharpens it. What would settle them: a concurrent-invoke harness, which the repo cannot build today (no `MockRuntime`). |
| 18 | Block comments, raw strings, `cfg_attr`, and symlinked files are still unhandled by the counters and the walk (edge-case ×2) | `low` | defer | Pre-existing shapes, no occurrence in the tree today, and `AGENTS.md:68` already names the comment half. |
| 19 | Deleting the three per-file `assert_eq!` messages lost the named wire lists behind each count (edge-case) | `low` | defer | Real information loss, but the names live in `blocking_wire_cases()` one screen above; not worth re-encoding twice. |
| 20 | `Indexer::rebuild` overlap between the two `create_work_*` wires (edge-case, withdrawn by its own author) | `false` | — | The reviewer re-checked and found `rebuild_lock` already serializes it, and removed the finding before filing. Recorded so the same question is not re-opened. |

### Pass 3 — 2026-09-15, three layers on the shipped tree (`29d10bf..f09d79d`)

Resumed review: the spec was left `in-review` after pass 2, so step-01 routed back here and the
three layers ran again on the **patched** tree — the pass-2 patches had never themselves been
reviewed. 29 findings. No `intent_gap` and no `bad_spec`, so no loopback: `review_loop_iteration`
stays **1**.

Five findings restate rows already in the pass-2 table; they are marked `carried`, keep pass 2's
verdict and route, and are **not** patched or deferred a second time.

🔴 **Two things this pass established by running something, not by reading it.** First, the
counter-proof recorded in the census gate's own doc-comment does not reproduce as written — it
reds through a *different* hole than the one it claims to demonstrate, so hole ① had no valid
demonstration behind it until this pass ran one. Second, the defect that finding 2 exists to
prevent **did not reproduce in 10 runs** on this machine; the patch is kept as structural
insurance and says so, rather than claiming a measurement it does not have.

| # | Finding | Verdict | Route | Evidence |
|---|---|---|---|---|
| 1 | The executed verification contains zero executions of the six changed wires (verification-gap, pre-verified) | `medium` | patch | `pre-push` excludes e2e (`.githooks/pre-push:27-29`); `.github/workflows/ci.yml:51-54` runs the `e2e` job only on `schedule`/`workflow_dispatch`, **not on push** — confirmed live: run `34959441963` shows `e2e (macos-26) in 0s`. Closed by running `npm run test:e2e`: **24/24 spec files, exit 0, 2m27s**. Measured dispatch coverage: `create_work_from_text` (10 raw `invoke` sites in `e2e/`) and `confirm_import_with_encoding` (2 specs through the real product path — `importForm.mjs` → `.ip-act-primary` → `confirmImportPreview()` → `confirmImportWithEncoding` → the wire), i.e. **2 of 6**, including the heaviest one the Intent names. The other four need a file dialog or have no caller. |
| 2 | The new concurrency test's red direction is unforced, so the mutation it guards can pass (verification-gap, pre-verified) | `medium` | patch | Claim filed as: with the guard narrowed, thread 1 can finish and clear the slot before thread 2 reads it, yielding the exact pair the test asserts. **Measured on the mutated tree (guard narrowed to read-clone-`drop`-then-`create_work`): with `Barrier` 10/10 red; without `Barrier` 10/10 red.** The green-on-a-broken-tree scenario did **not** reproduce on Ice's macOS. `Barrier::new(2)` kept anyway — it removes the scheduling dependence structurally, and 10/10 on an idle macOS says nothing about a loaded Windows runner. The test's comment now carries both numbers and forbids citing it as evidence the test ever went green falsely. |
| 3 | The census gate's recorded counter-proof cannot reproduce as written (edge-case) | `medium` | patch | **Seeded both spellings and ran them.** One-line `#[tauri::command] pub fn ai4_probe_cleanup() {}` (the form written in the doc-comment) ⇒ red via hole ②: `` `src/commands/cleanup.rs:265` mang mot cach viet thuoc tinh KHONG nam trong hai cach da biet ``. Two-line form ⇒ red via hole ① with exactly the recorded text: `cleanup.rs dem duoc 6 plain / 0 (async) … khai 5 plain / 0 (async)`, and `the_blocking_wires_run_off_the_main_thread ... ok`. The gate is sound; the recorded proof was not. Corrected to the two-line form with a 🔵 note recording that hole ① had no valid demonstration until now. |
| 4 | `RS_FLOOR_FOR_DIALOG_CHECK = 44` is 55% of today's tree, violating its own stated 80–85% rule (blind-hunter) | `medium` | patch | Its doc-comment derives 44 from **55** `.rs` files measured 2026-08-25. Measured 2026-09-15: `find src-tauri/src -name '*.rs' \| wc -l` = **80**. The new `all_src_rust_files()` sweep added by pass-2 finding 7 reuses this constant, so the sweep can lose 45% of the tree and stay green. Re-measured to **65** (80 × 0.82) with the population and the re-measure command written next to it. |
| 5 | The census `why` column is destructured to `_`, so D5's owned-declaration rule is enforced by nothing (edge-case) | `medium` | patch | `for &(rel, want_plain, want_async, want_rows, _) in &COMMAND_FILE_CENSUS` — the column the doc-comment calls a LỜI KHAI CÓ CHỦ is dead data, and the claim that `segment.rs` is *deliberately* empty is unverifiable. Added an assert binding `why.is_empty()` to `rel == "src/commands/segment.rs"` for every 0-`(async)` row. **Counter-checked in both directions:** blanking `cleanup.rs`'s note ⇒ red; giving `segment.rs` a note ⇒ red. An assert green on both branches would have guarded neither. |
| 6 | The sidecar preserves the exact false premise this change exists to refute, with no in-place correction (blind-hunter) | `medium` | patch | `ai-4-loopback-1-history-2026-09-15.md:65` asserts, in the present tense as a *"Manual check re-verified"*, that `(async)` on a plain `fn` "routes to `sync_threadpool`". The refutation sits 20 lines below, but `AGENTS.md:51` requires the claim itself be fixed in place with 🔵 and a date, and the file's header granted itself a blanket exemption ("nothing here was deleted or edited"). A `grep` for `sync_threadpool` lands on the false line; `deferred-work.md`'s census of survivors is scoped to `src/` and never counted it. Struck through and answered in place; the header amended to say refuted claims are answered, not preserved intact. |
| 7 | The attribute classifier reports a legal one-line Rust spelling as an unknown *Tauri* spelling (blind-hunter, edge-case) | `low` | patch | `count_command_attrs_in` exact-matches the whole trimmed line, so `#[tauri::command] pub fn f()` — a *known* attribute with a legal tail — panics with "một cách viết thuộc tính KHÔNG nằm trong hai cách đã biết", sending the reader after a Tauri attribute form that is not the problem. Demonstrated by the finding-3 probe. Split into its own arm naming the real condition (attribute not alone on its line). |
| 8 | A dead 317-line `.patch` artifact ships in the commit carrying superseded literals (blind-hunter) | `low` | patch | `ai-4-implementation-2026-09-15.patch` is loopback-1 code that was reverted and re-derived. It contains `[…; 22]` (shipped: 24 rows), `count_async_attrs("src/commands/project.rs") == 6` (shipped: 9/8 via the shared counter) and a twelve-file 55/24 table (shipped: eleven files, 53/26) — and holds 6 hits for `COMMAND_FILE_CENSUS`/`blocking_wire_cases`, so a grep for either lands in dead code. Header added naming it a dead artifact and pointing at the live definitions. |
| 9 | The de-numbered heading re-numbers itself three words later, and miscounts its own drift (blind-hunter) | `low` | patch | *"Câu mới cố ý KHÔNG mang số"* is written immediately after *"mảng nay có **24** hàng"* — rebuilding the second source of truth it says it is removing, while `COMMAND_FILE_CENSUS` already holds and asserts that count. And "đã cũ **ba** lần" over `2 → 5 → 7 → 18 → 24` is four transitions; the doc-comment below calls this change *MỞ LẦN TƯ*. Number removed from the prose, three corrected to four. |
| 10 | `pipeline.rs:702` is cited as the Blob-specific reason `blocks` is empty, in two places this diff wrote (verification-gap, Other) | `low` | patch | `:702` is `blocks: vec![None; n]` inside the common `Flow` initializer, reached identically by `Blob`, `Chapters` **and** `Bilingual` (`match` at `:680-692`) — it cannot distinguish Blob from the URL path. The real reason is that the step filling `blocks` runs only on the HTML/URL path (`pipeline.rs:1554-1560`). Conclusion unchanged, pointer corrected in `project.rs` and in the `cases` row. Same class as pass-2 findings 1 and 2 — a wrong line inherited from this spec's Code Map. |
| 11 | "Sáu chỗ gọi" in `importPreviewState.ts` is a count with no stated scope, and the whole reachability argument rests on it (blind-hunter) | `low` | patch | `grep -n 'if (confirming.value' src/importPreviewState.ts` returns **eleven** guards: the six named plus `:1424`, `:1441`, `:1466`, `:1501`, `:1624`. The six may well be the correct six for the three still-plain wires, but no criterion is written for which guard fronts which wire, and "not reachable through the product UI" is only true if the enumeration is complete. Scope note added to the `deferred-work.md` entry. |
| 12 | `(async)` moved between two non-`cases` `project.rs` wires passes both gates (edge-case) | `medium` | patch | `carried` — pass-2 finding 4, same location and same claim; the code still reads as that row describes. Not patched again. |
| 13 | Overlapping preview/confirm can stash a different source than the one on screen (edge-case) | `medium` (unverified) | defer | `carried` — pass-2 finding 17. |
| 14 | Deleting the per-file assert messages lost the named wire lists (edge-case) | `low` | defer | `carried` — pass-2 finding 19. |
| 15 | `cfg_attr`, bare `#[command]`, and multi-line attribute forms are counted by neither column (edge-case) | `low` | defer | `carried` — pass-2 finding 18. Re-measured anyway: `use tauri::command`, bare `#[command]` and `cfg_attr(…tauri::command` all return **zero** hits under `src/`. |
| 16 | Four of the six flipped wires have no overlap test (verification-gap, filed for completeness) | `medium` | defer | `carried` — pass-2 findings 14/17, and the layer filed it as `defer` itself. |
| 17 | The stated blocker for deferring the overlap tests — "the repo cannot build a concurrent harness (no `MockRuntime`)" — is false (blind-hunter) | `medium` | defer | **New, not carried**: this refutes the *reason* on a carried row rather than restating its claim. The test added by this very diff drives the pure fn from two real threads via `std::thread::scope` with no tauri runtime at all, twinning `segment_contract.rs:9012`. The real blocker is scope (four more tests, four different invariants), not capability. New `deferred-work.md` entry carries the corrected reason and warns against re-copying the false one. |
| 18 | `wire::create_work_from_file` has no caller anywhere in the repo (measured during finding 1) | `medium` | defer | **New.** `grep -rn create_work_from_file src/ e2e/` = **3** hits, all comments (`project.ts:119`, `:126`, `GridPanel.vue:196`). Zero `invoke` sites in `e2e/`, against 10 for its twin `create_work_from_text`. ⚠️ The verification-gap layer filed that *both* wires have e2e callers; the count shows one does. Deleting an IPC wire is a product decision, not a review patch — filed with an owner. |
| 19 | The census counter dies with a raw I/O error when a tabled file is renamed or deleted (blind-hunter, edge-case) | `low` | rejected | Real: `count_command_attrs` does `unwrap_or_else(\|e\| panic!("read {}: {e}"))`. Rejected under the low-finding rule — renaming a command file is not everyday work, the resulting message (`read src/commands/foo.rs: No such file`) is not actually misleading, and the proposed fix adds a guard branch. |
| 20 | The race test leaks its temp dir and open store on the assertion-failure path (edge-case) | `low` | rejected | Verified: `cleanup(&root)` and `drop(opened.store)` sit after the asserts. But **32 other tests in the same file end with the identical `cleanup(&root)`** — this is the file's established convention, not something this story introduced, and the root is uniquely named per run. Fixing one of 33 identical sites buys inconsistency, not safety. |
| 21 | The pin's stated reason is falsified by the census's fourth column (blind-hunter) | `false` | rejected | The sentence at `config_invariants.rs:1259` — *"bảng đó biết `project.rs` có 8 vỏ `(async)`, nó không biết vỏ NÀO"* — is **accurate**. The fourth column pins a *row count* (`want_rows`), and a count is not a name; nothing in `COMMAND_FILE_CENSUS` names a wire. The finding's own premise does not disprove the sentence. |
| 22 | The pass-1 triage ledger does not say which pass-1 `patch` findings were made moot by the D5 redesign (blind-hunter) | `low` | rejected | Fix is to edit this build's spec. Reported to the human instead. |
| 23 | Two contradictory full-suite totals in one file: 1504 vs 1503 (blind-hunter, edge-case) | `medium` | rejected | Real and confirmed — the diff adds two `#[test]` fns, so **1504** is right and §*Verification numbers* is stale. Fix is to edit this build's spec, which step-04 forbids, so it routed `rejected`. **Escalated to Ice, who authorised an in-place 🔵 fix**; §*Verification numbers* now carries it, re-measured independently at `1504 passed / 0 failed / 20 ignored`. |
| 24 | Two contradictory pre-push durations for the same run: 165 s vs 161 s (blind-hunter) | `low` | rejected | Same rule, same routing. **Escalated to Ice, who authorised an in-place 🔵 fix.** Resolved not as a contradiction but as three *different runs* — 165 s, 161 s, 178 s — now written down as such, with the note that none is a baseline on its own. |
| 25 | §*Round 2 — implementation* ③ says the false `sync_threadpool` claim survives at "FIVE sites in THREE files", listing `project.rs:6339` (blind-hunter, edge-case) | `medium` | rejected | Confirmed by grep: the survivors are `glossary.rs:1301`, `:1375`, `library.rs:597`, `:643`, `core/webimport/mod.rs:22`, `:43` — **six sites in three files**, exactly as the frozen Never and the `deferred-work.md` entry already say. `project.rs:5802` holds only the *corrected* sentence. So ③ alone is wrong, and ③ is in this build's spec. **Escalated to Ice, who authorised an in-place 🔵 fix**, now appended to ③. Worth naming: the miscount sits in the very sentence complaining that another count was low. |
| 26 | The spec's own done-condition — "The CI run has not been read: nothing has been pushed" — shipped already false (blind-hunter) | `low` | rejected | Fix edits this build's spec. **Escalated to Ice, who authorised an in-place 🔵 fix.** Now closed on the facts: `574c869..f09d79d` pushed with Ice's approval; run `34959441963` green on Windows (30m13s) and macOS (10m04s) — with the caveat, written into the spec, that the `e2e` job ran 0 s because it is schedule-only. |
| 27 | The per-file assert messages that named each count's wires were deleted (edge-case) | `low` | rejected | Duplicate of row 14 as filed by a second layer; recorded here so no finding is dropped. |
| 28 | A renamed/moved census path should fail with a classification message, not an I/O one (edge-case) | `low` | rejected | Duplicate of row 19 as filed by a second layer. |
| 29 | The negative-direction loop is redundant with the census test (blind-hunter, restated) | `low` | rejected | `carried` refusal — pass-2 finding 15 rejected this on the same grounds: the duplication is cheap and the second failure message is not. |

## Design Notes

**Why the counter-check runs before the fix.** The gate is a source-text scan, so the only way to show it discriminates is to make it red on real product code. `AGENTS.md:67-68` requires a removal at the seam and warns that commenting a line out is not a removal for a text-scanning gate. Adding a `cases` row while the attribute is genuinely plain *is* the removal — the product is what is in the wrong state, not the test.

**Why not scan bodies.** One line of evidence, measured on the baseline tree:

```
$ sed -n '5983,6063p' src/commands/project.rs | rg 'blocking|reqwest|fetch|fs::|read_to_string|File::'
30:        // `create_work` (mỗi lời gọi `fetch` một lần, ngay khi hoàn tất) — KỂ CẢ khi
```

One hit, and it is a comment. The real blocking is four call-hops away. A per-function body scan is green on the very defect this spec fixes.

**Why the count table earns its place, and what it does not prove.** It is not a safety argument — it is a tripwire. Its value was demonstrated in loopback 1: removing `(async)` from `reload_url_import_item`, a wire that `cases` does not name and that nothing previously watched, turned it red. What it cannot do is say *which* command is async; only a `cases` row names one. That is why both structures exist and why the sibling gate must pin `project.rs` as well as `chapter.rs`.

**The trap that produced loopback 1, stated so it is not walked into twice.** Reading `wrapper.rs:264` and seeing the literal `"sync_threadpool"` is not evidence about execution. Follow the variable: `kind` is used once, at `:278`, inside `tracing::debug_span!`. The executing path is `body_async` → `respond_async_serialized` → `async_runtime::spawn` → `tokio::spawn`. A string that names a thing is not the thing.

## Verification

**Commands:**
- `cargo test --locked --test config_invariants` (from `src-tauri/`) — expected: RED at step ① naming a signature; GREEN after step ②; GREEN again after step ③.
- `cargo test --locked` (from `src-tauri/`) — expected: no regression against the pre-change baseline count. Record both numbers; do not write "all green" without them.
- `cargo test --locked --features nfr-bench --test config_invariants` — expected: same verdict as the default build.
- `npm run check:gates` — expected: unchanged, still green.
- `.githooks/pre-push` — expected: green. 🔴 A green pre-push here is Ice's macOS only; per `AGENTS.md` it says nothing about Windows or UTC. Read the CI run before this spec is declared done.
- `git status` after any seeded-violation probe — expected: no leftover probe file. A diff of tracked paths will not show one.

**Manual checks:**
- Confirm the D4 mechanism paragraph against source before writing any doc-comment that repeats it: `wrapper.rs:264` and `:278`, then `ipc/mod.rs:343`/`:371`/`:375`, then `async_runtime.rs:103-113`. Do not cite `wrapper.rs:264` as proof of a blocking pool — it is where the `"sync_threadpool"` *string* is written, and reading a string is not evidence about execution; `:278` is its only consumer. 🔵 **SỬA 2026-09-15 (review round 2)** — this line said `:263`, which is `let kind = match attrs.execution_context {`. The frozen Intent (§`:24`) and the loopback-1 record carry the same `:263` and are left as written; the three places this change itself wrote it (`project.rs`, `config_invariants.rs`, `deferred-work.md`) are corrected to `:264`.
