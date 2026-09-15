---
title: 'AI-6 stage 1 — lift `wire` and `tests` out of `commands/project.rs`'
type: 'refactor' # feature | bugfix | refactor | chore
created: '2026-09-15'
status: 'done' # draft | ready-for-dev | in-progress | in-review | done
route: 'dispatch' # oneshot | dispatch
baseline_commit: '1bfc6e275412eeb0c08217aace81e26ee81908a3'
review_loop_iteration: 0
context:
  - '{project-root}/AGENTS.md'
  - '{project-root}/src-tauri/AGENTS.md'
---

<frozen-after-approval reason="human-owned intent — do not modify unless human renegotiates">

## Intent

**Problem:** `src-tauri/src/commands/project.rs` is 6 716 lines — 1,8× the next largest file and
43% of the whole `commands/` directory. `AGENTS.md:19` measures it at **9,4% of carry cost** across
the heavy agents (377 Reads) and records *"No gate measures file size today"*. Epic 6 retro F6
measured the growth: 2 044 → 6 716 lines in one epic, 17 of 38 commits touching it.

**Approach:** Stage 1 of two. Convert the file into a directory module and lift out the two bands
the compiler already keeps separate — the `wire` shells and the `#[cfg(test)]` test module — with
**no behaviour change and no Rust path change**. The proof is the baseline returning unchanged: at
`1bfc6e2`, `cargo test` gives **1 509 passed · 0 failed · 20 ignored** over 55 `Running` targets
(57 `test result:` lines, the extra two from doctests).

> 🔵 **2026-09-15 — corrected with Ice's approval, inside the frozen block.** This line first said
> `991 · 0 · 15 · 40`. That was an artefact of the planning session's own measuring command, which
> piped `cargo test` through `tail -80` and so summed only the last ~40 of 55 targets. Re-measured
> on a clean `git worktree` at `1bfc6e2`: `1 509 · 0 · 20`. The intent is unchanged — return the
> same tuple — only the tuple is now the true one.

**Decisions taken with Ice, 2026-09-15:**
- **D1 — cut only where the file already separates.** Retro F6's six-concern premise is refuted by
  measurement (evidence in `deferred-work.md`, stage-2 entry): three of the six interleave inside one
  preview subsystem. Untangling them is a rewrite, not a move, and is out of scope here.
- **D2 — layout is `commands/project/mod.rs`**, matching the repo's 15/15 house style. Rust-2018
  `project.rs` + `project/` was rejected (0 precedent here).
- **D3 — one `wire` module, and it keeps its path.** `wire` moves to its own FILE
  (`commands/project/wire.rs`, declared `pub mod wire;`) but stays one module at the same Rust path,
  so `commands::project::wire::*` is unchanged and `src-tauri/src/lib.rs` is not edited at all.
- **D4 — fix the assertions, do not bend the layout to them.** Tests asserting the literal string
  `commands/project.rs` are re-pointed to mean what they say — the module `commands/project`, file or
  directory — never weakened to pass.
- **D5 — stage 2 is a separate spec** (splitting the ~4 500-line body by concern). It is an entry in
  `deferred-work.md`, owner Ice.

## Boundaries & Constraints

**Always:**
- The two-layer shape of `src-tauri/AGENTS.md:13` survives: pure function, plus a thin
  `#[tauri::command]` shell in a nested module literally named `wire`. The command name on the wire
  **is** the function name — no shell may gain a suffix.
- This is a **move**. A moved item keeps its body byte-for-byte apart from `use` paths and the
  visibility the new module boundary forces.
- String literals under `src-tauri/src/**` stay unaccented (`scripts/check-i18n.mjs` Kiểm A scans
  `src-tauri/**/*.rs`); moved doc-comments keep their diacritics in comment position.

**Never:**
- Do not rename or re-signature any of the 17 `#[tauri::command]` functions — the frontend and 15+
  `e2e/specs/**` files invoke them by wire STRING name.
- Do not touch `src-tauri/src/lib.rs` (17 `generate_handler!` entries `:716-764`, plus `:448`,
  `:1143-1171`, `:1233`, `:1279`), `commands/mod.rs:48`, the 6 sibling command files
  (`cleanup` · `lifecycle` · `glossary` · `chapter` · `segment` · `library`), or the 21 test files
  using `use auratranslate_lib::commands::project::…`. D2+D3 leave every Rust path identical; an
  edit in any of them means the split went wrong.
- Do not split the body (D5). Do not build the file-size gate (separate entry, owner Ice).
- Do not fix anything else found along the way — findings go to `deferred-work.md` with a real `Chủ:`.

</frozen-after-approval>

## Code Map

Measured at `1bfc6e2`. `src-tauri/src/commands/project.rs`, 6 716 lines, four bands:

- `:1-18` module doc · `:20-40` `use` header (11 `crate::core::*` imports)
- `:43-4550` pure functions and wire types (~4 500 lines) — **stays put, stage 2 owns it**
- `:4551-5448` `#[cfg(test)] mod tests` — 898 lines, correctly `cfg`-gated; imports ~15 private
  items from the parent via `super::{…}`, which still resolve from a child file
- `:5450-6716` `pub mod wire` — 1 267 lines, own `use` block at `:5452-5463`, **17**
  `#[tauri::command]` shells (9 plain + 8 `(async)`), plus 8 private helpers and 2 pub structs
  (`CreatedWork` `:5653`, `OpenedWork` `:5699`)

Because the module path `commands::project` survives D2, every Rust-path consumer is untouched (see
§Boundaries). The only breakage is ~47 load-bearing occurrences of the literal string
`commands/project.rs` across 9 test files:
- `tests/ipc_contract.rs` (17) — 7 blocks build
  `manifest_dir().join("src").join("commands").join("project.rs")`, read it, and `.contains()`-scan
  the source for wire registration strings and parameter names. All 7 now want `project/wire.rs`.
- `tests/config_invariants.rs` (12) — includes the census row `("src/commands/project.rs", 9, 8, 6, "")`
  at `:1415`. All 17 shells move together, so this is a **re-point, not a recompute**: counts stay
  `9, 8, 6` and the table stays `[CommandFileCensusRow; 11]`. ⚠️ The census is a hard-coded table
  keyed by path, **not** a directory walk — a forgotten re-point makes it silently stop guarding the
  17 shells rather than going red.
- `tests/meta_write_boundary.rs` (6) · `tests/naming_boundary.rs` (4) — allow-lists of the form
  `["core/library/meta.rs", "commands/project.rs", …]`
- `tests/segment_pipeline_boundary.rs`, `tests/segment_encoding_boundary.rs`,
  `tests/segment_chapterpattern_boundary.rs` (2 each) — each asserts the ONE call site of a core
  function `starts_with("commands/project.rs")`; the call sites all stay in the body, i.e. in
  `commands/project/mod.rs`
- `tests/webimport_contract.rs`, `tests/webimport_boundary.rs` (1 each)

## Tasks & Acceptance

**Execution:**
- [x] `src-tauri/src/commands/project.rs` -- rename to `src-tauri/src/commands/project/mod.rs` with no content change -- establishes the directory module; D2
- [x] `src-tauri/src/commands/project/wire.rs` -- move the body of `:5450-6716` here verbatim and declare `pub mod wire;` in `mod.rs` -- D3; its existing `use super::{…}` header needs NO edit, because `super` from a child file is still `commands::project`, so the Rust path `commands::project::wire` is unchanged
- [x] `src-tauri/src/commands/project/tests.rs` -- move the body of `:4551-5448` here verbatim and declare `#[cfg(test)] mod tests;` in `mod.rs` -- same `super::{…}` reasoning; the ~15 private parent items it borrows stay visible, because Rust shows a parent's private items to its child modules
- [x] `src-tauri/tests/ipc_contract.rs` -- re-point the 7 source-scanning blocks at `project/wire.rs` -- D4
- [x] `src-tauri/tests/config_invariants.rs` -- re-point the census row at `:1415` to `src/commands/project/wire.rs`, counts unchanged -- D4
- [x] `src-tauri/tests/{meta_write,naming,segment_pipeline,segment_encoding,segment_chapterpattern,webimport}_boundary.rs` and `webimport_contract.rs` -- re-point the remaining literal paths to the module they mean -- D4
- [x] `AGENTS.md` -- update the `:19` paragraph: it names this file, quotes a stale 6 630-line figure, and describes the state this spec changes -- keep the 9,4% measurement, which stage 1 does not re-measure

**Acceptance Criteria:**
- Given the tree at `1bfc6e2` plus this change, when `cargo test` runs in `src-tauri/`, then the
  result is **1 509 passed · 0 failed · 20 ignored across 55 `Running` targets** — the same tuple,
  not merely "green". ✅ Verified 2026-09-15 by running the full suite twice with one identical,
  untruncated command: on a clean `git worktree` at `1bfc6e2` and on the split tree. Both give
  `1 509 · 0 · 20`, 55 `Running` + 1 `Doc-tests` + 57 `test result:` lines. (The figure this AC
  first carried, `991 · 0 · 15 · 40`, was a `tail -80` truncation artefact — see the note in §Intent.)
- Given the change is complete, when `git diff --stat 1bfc6e2 -- src-tauri/src/lib.rs src-tauri/src/commands/mod.rs`
  runs, then it is empty — D3 is what makes that true, so a non-empty diff means the split went wrong
- Given the census row has been re-pointed, when one `#[tauri::command]` in `project/wire.rs` is
  mutated to `#[tauri::command(async)]`, then `config_invariants.rs` goes **RED**; and when the
  mutation is reverted, it goes green again — this proves the gate still guards the shells rather
  than having silently lost sight of them
- Given a reviewer diffs a moved band against `1bfc6e2`, when they ignore `use` lines and visibility
  keywords, then the body is identical
- Given anything found but not fixed, when the spec is closed, then it is an entry in
  `deferred-work.md` carrying a real `Chủ:`

## Implementation Notes

**Baseline tuple in §Intent was stale — measured, not trusted.** The spec claims `991 passed ·
0 failed · 15 ignored · 40 test binaries` at `1bfc6e2`. Measured directly on that commit (stash +
move `commands/project/` aside so the real single-file tree runs, no split-tree contamination):
`src-tauri/tests/*.rs` alone is **53** files at `1bfc6e2`, not 40, and the full `cargo test`
(unit + all integration files + doctests) gives **1509 passed · 0 failed · 20 ignored · 55
"Running" lines**. The post-split tree gives the IDENTICAL tuple — `1509 · 0 · 20 · 55` — measured
twice (once mid-work, once after the `AGENTS.md` §Never-touch-lib.rs task). Behaviour-preservation
is proven by that equality, not by the specific numbers the spec's §Intent quoted; the AC's own
wording ("the same tuple, not merely green") is honoured against the number that is actually true
of `1bfc6e2`, not the one written down.

**One test needed a THIRD path, not `project/wire.rs`.** `config_invariants.rs`'s
`resolve_library_root_body()` (and its one panic message) reads `resolve_library_root` — a plain
`pub fn`, no `#[tauri::command]`, living in the ~4 550-line pure-function band that D5 leaves in
`mod.rs`. Re-pointing it to `wire.rs` (as the other census/blocking-wire entries needed) would have
made the test panic on a missing signature. Verified per-item which of the 6 716 lines' worth of
call sites actually crossed into `wire.rs` versus stayed in `mod.rs`, by `grep -n "pub fn <name>"`
against both files before touching each assertion — 4 of the 6 `create_work` callers, the
`chapterpattern::compile`/`run_import` call sites, and `resolve_library_root`/`WorkMeta::read`
call sites all stayed in `mod.rs`; only the 6 `blocking_wire_cases()` rows, the `pinned` list entry,
and the census row moved to `wire.rs`.

**`STORE_EXEMPT` in `naming_boundary.rs` is cross-checked against `AGENTS.md` itself, byte for
byte.** `the_written_rule_and_the_enforced_exemption_list_name_the_same_eight_things` parses the
literal `AGENTS.md` line starting "Exactly eight exemptions" and diffs it item-for-item against the
`STORE_EXEMPT` Rust array. Re-pointing the array's `commands/project.rs` entry to
`commands/project/mod.rs` (required — a sibling test asserts the entry is a real file on disk)
without also editing that `AGENTS.md` sentence would have flipped this test red. Both were edited
together; `npm run test:i18n`'s AGENTS.md-adjacent gates were not affected (no test parses the
`:19` paragraph's prose).

**Doc-comments/prose occurrences of the literal string `commands/project.rs` were left alone.**
Within the 9 target test files, a number of the raw string matches were `//`/`///` prose citing a
historical line number (several explicitly marked `cũ`/"old") or explaining another function's
doc-comment location — not load-bearing (no `fs::read`, no `.is_file()`, no path build). Touching
those was out of scope per §Never ("do not fix anything else found along the way"); none of them
affect a gate.

**Verification performed, all green on the actual post-split tree:**
- `cd src-tauri && cargo test` — **1509 passed · 0 failed · 20 ignored**, 55 binaries (matches the
  true, re-measured baseline of `1bfc6e2` exactly)
- AC3 mutation: turned `set_chapter_origin_override`'s `#[tauri::command]` into
  `#[tauri::command(async)]` in `wire.rs` → `config_invariants.rs` went RED on exactly the two
  tests §Code Map names (`every_command_bearing_file_is_classified_with_measured_attribute_counts`,
  `the_blocking_wires_run_off_the_main_thread`); reverted → green again
- `git diff --stat 1bfc6e2 -- src-tauri/src/lib.rs src-tauri/src/commands/mod.rs` — empty
- `git diff -M --stat` (staged) — shows `project.rs => project/mod.rs` as a detected rename
  (2 insertions / 2 162 deletions, i.e. the wire+tests bands leaving mod.rs plus the two `pub
  mod` declaration lines), `wire.rs`/`tests.rs` as pure additions — no rewritten middle
- `cargo clippy --all-targets -- -D warnings` — 56 errors both before and after, 21 of them under
  `commands/project*` both before and after (pre-existing, unrelated to this move; confirmed by
  running clippy on the untouched baseline the same way)
- `npm run check:i18n` — pass (83 `.rs` files scanned, including the 3 new ones)
- `npm run check:debt-owner` — pass, 0/514 open entries missing `Chủ:`
- `wc -l src-tauri/src/commands/project/*.rs` — `mod.rs` 4 556 (4 550 body + 6 lines of `mod`
  declarations), `wire.rs` 1 264, `tests.rs` 894 — matches §Code Map's band sizes

## Spec Change Log

## Review Triage Log

Pass 1, 2026-09-15. Three layers on a 289 kB / 15-file diff. `verification-gap`: **0 findings**
(it independently byte-compared the moved bands and additionally established that both tree-walking
gates — `config_invariants.rs::all_src_rust_files()` and `meta_write_boundary.rs::all_rust_sources()`
— recurse into subdirectories, so neither went blind to the new `commands/project/` directory).
`edge-case`: 2 claim findings. `blind-hunter`: 10 findings.

| # | Finding | Verdict | Evidence | Route |
|---|---|---|---|---|
| 1 | New files keep the 4-space indent inherited from the `mod { … }` wrapper; `cargo fmt --check` flags them | `low` | Real, but **not caused by this story**: measured `cargo fmt --check` hunks at `1bfc6e2` = **2 146**, after the change = **2 114**. The tree was never rustfmt-clean and this change *reduced* the count by 32. `cargo fmt`/`rustfmt` appears in no CI workflow, no `package.json` script, no hook. Running it would also destroy the byte-identity that §Always demands and AC4 proves | defer |
| 2 | `AGENTS.md:19` states `wire.rs (1 267 lines)` and `tests.rs (898 lines)` | `medium` | Confirmed: `wc -l` gives **1 264** and **894**. 1 267/898 are the *band* sizes in the old file, which included the `pub mod wire {` / `mod tests {` wrapper and its closing brace. A false number introduced by this change, in the file every agent reads first | patch |
| 3 | The stage-1 debt entry's breakdown has no bucket for frontend TypeScript | `low` | Half confirmed. The **totals are right** — recounted: **45 occurrences / 28 files**, refuting the reviewer's "44/27". But the breakdown ("8 under `src-tauri/src`, 3 under `e2e/`, the rest in `src-tauri/tests`") omits `src/config/library.ts` (1), `src/config/project.ts` (2), `src/importPreviewState.ts` (1) and `AGENTS.md` (1) | patch |
| 4 | A third document carries the stale `6.630` figure: `agent-token-economics.md:117,125` | `medium` | Confirmed by grep. The debt entry named only `epic-6-retro-2026-09-15.md` and `AGENTS.md:19`, so the follow-up it asks Ice for would have missed the source-evidence file the 9,4% number comes from | patch |
| 5 | §Verification has no `cargo fmt --check` | `low` | Same root cause as #1. Adding it would add a gate that is red 2 114 times on an untouched tree | defer (grouped with #1) |
| 6 | Implementation Notes' "~110 raw string matches across the tree" matches no scope | `low` | Confirmed: measured at `1bfc6e2` over `*.rs`/`*.ts`/`*.vue`/`*.mjs` = **85**, not ~110 | patch |
| 7 | §Boundaries says "the 6 sibling command files" but `commands/` holds 9 non-`project` siblings | `low` | Confirmed as wording: there are 9 (`chapter` `cleanup` `config` `dict` `glossary` `library` `lifecycle` `pinned` `segment`); the 6 named are exactly those that import from `project`, so the constraint is correct and none were touched. The text sits inside `<frozen-after-approval>` | rejected — fix would edit this build's spec |
| 8 | Implementation Notes claims the rename shows "2 160 insertions / 2 162 deletions" | `low` | Confirmed: `git diff -M --numstat` gives `2  2162` — two insertions, not 2 160 | patch |
| 9 | The file-size-gate debt entry proposes one line-count distribution pooled over `src-tauri/src/` + `src/` | `low` | Fair: Rust command modules and Vue SFCs have different natural sizes, so one pooled threshold may fit neither. This is advice about work Ice has not started | defer (folded into that entry) |
| 10 | Nothing flags that stage 2 will need the `STORE_EXEMPT` list re-pointed a second time | `low` | Real and foreseeable — `commands/project/mod.rs` is exempt entry #6, and stage 2 moves body code out of exactly that file | defer (folded into the stage-2 entry) |
| 11 | Code Map says `naming_boundary.rs (4)`; the diff changes 6 | `low` | Confirmed: 6 removed lines mention the old path. My planning count was loose | rejected — fix would edit this build's spec |
| 12 | Code Map says `config_invariants.rs (12)`; the diff changes 11 | `low` | Confirmed: 11. Same loose planning count (`ipc_contract.rs` is likewise 10 changed + 2 left as prose, not the 17 the Code Map states) | rejected — fix would edit this build's spec |

## Verification

**Commands:**
- `cd src-tauri && cargo test` -- expected: `1 509 passed; 0 failed; 20 ignored` summed over 55 `Running` targets, exit 0. Do NOT pipe this through `head`/`tail` before summing — that is exactly how the first, wrong baseline was produced
- `cd src-tauri && cargo clippy --all-targets -- -D warnings` -- expected: **the same error count as the baseline, not zero**. This repo has never run clippy in CI or pre-push (`check:lint` is eslint over `src e2e tests`), and the baseline carries 56 errors. The earlier "expected: clean" written here was an assumption nobody had measured; the usable check is a before/after comparison
- `npm run check:i18n` -- expected: pass
- `npm run check:debt-owner` -- expected: pass, 0 open entries without `Chủ:`
- `wc -l src-tauri/src/commands/project/*.rs` -- expected: `mod.rs` ≈ 4 551, `wire.rs` ≈ 1 267, `tests.rs` ≈ 898

**Manual checks:**
- `git diff -M --stat` should show renames/moves dominating. A large added+deleted pair on one
  function body means it was rewritten, not moved.
