---
title: 'Story 11.1, lot B — pay down the Rust boundary-test debt items'
type: 'chore'
created: '2026-09-24'
status: 'done'
route: 'dispatch'
baseline_commit: 'b825a58836f2af221cc6c353daffbcbd3319bcf2'
review_loop_iteration: 1
context:
  - '{project-root}/src-tauri/AGENTS.md'
  - '{project-root}/_bmad-output/implementation-artifacts/epic-11-context.md'
---

<frozen-after-approval reason="human-owned intent — do not modify unless human renegotiates">

## Intent

**Problem:** Twelve `deferred-work.md` items owned by Story 11.1 concern `src-tauri/tests/*_boundary.rs`. Eighteen boundary files each carry their own copy of the tree-reading helpers. Only two of those copies use the safe directory check `rel == DIR || rel.starts_with("DIR/")`. None of them strips `/* */` comments. 🔵 2026-09-25: seven of them (not two) hide every line after a file's first `#[cfg(test)]`, which today includes about 370 product lines of `core/segment/role.rs`. `dict_boundary.rs::walk_any` panics on a `.DS_Store`. One item (`api_key`) never got the closure line that ballot #106 decided: the line was appended to the FR77 CI-anchor item right below it, and that item is now closed wrongly.

**Approach:** Lot B of four. Extract one shared scanning module, move every boundary file onto it, and fix the defects once, in that module. Give each of the twelve items exactly one Epic 11 disposition, after re-reading it against HEAD.

## Boundaries & Constraints

**Always:**
- Moving a file onto the shared module keeps its gate verdict and its scanned-file population identical on HEAD. Measure each file before and after; don't infer.
- Every fixed defect gets a case in the module's contract test, counter-checked by really removing the seam.
- Keep every item's existing text; append `→` lines only (AGENTS.md §Specs, handoffs, ledger).

**Never:**
- Change a `*_FLOOR` value (lot D).
- Touch `ipc_contract.rs` or `naming_boundary.rs`: they scan for other purposes.
- Add a dependency or a crate.
- Merge the two floors of `config_invariants.rs` and `glossary_boundary.rs`.
- Write speculative code for a "reopen when X" item whose condition has not happened.

## I/O & Edge-Case Matrix

| Scenario | Input | Expected |
|---|---|---|
| Sibling directory | `core/scope_legacy/x.rs` checked against `core/scope` | not inside |
| Block comment | a forbidden token between `/*` and `*/`, over several lines | not seen |
| `/*` inside a string | `"src/**/*.rs"; forbidden_call()` | the call is still seen |
| Raw string and char | `r#"/*"#`, `'"'` before real code | the code is still seen |
| Dotfile / non-UTF-8 | `src/.DS_Store` present | skipped with a named reason, no panic |
| Code after a test module | a forbidden token after the closing `}` of `#[cfg(test)] mod tests { … }` | seen |

## Decisions (Ice, 2026-09-24)

1. **L234 `KHÔNG LÀM`.** Test files open only temp databases; the `tests/**` exemption stays and its risk is accepted. The ledger line records the measurement: 17 files contain `rusqlite`, 7 of them only name the token.
2. **L542 `KHÔNG LÀM`.** Text scanning is a named limitation of the whole boundary-gate family. No partial guard, no AD.
3. **L9742: skip only the brace-matched `#[cfg(test)] mod … { }` block.** The shared helper replaces all seven `text_before_first_cfg_test_line` copies (🔵 2026-09-25, Ice: the spec first counted two; the other five are `cleanup`, `docx`, `segment_files`, `segment_pipeline` and `segment_chapterpattern`), so product code after a test module is scanned. If this turns any of the seven gates red on real code (e.g. `core/segment/role.rs` L108-480), fix that code in this lot. An unsupported shape (a `#[cfg(test)]` not followed by a `mod … {`, or unbalanced braces) is a FAIL, not a silent skip.
4. **The spec stays at ~2.1k tokens**, like lot A.

</frozen-after-approval>

## Code Map

- `src-tauri/tests/*_boundary.rs` (18 files) plus `config_invariants.rs:777` (`all_src_rust_files`). Copies of `src_root`/`rel_posix`/`all_rust_sources`/`code_lines`. Line table (phase handoff): `/private/tmp/claude-501/-Users-hoangnam-LocalSites-addon-AuraTranslate/d2034596-ade5-4c9b-8b18-29e6b5ab7d52/scratchpad/lot-b-investigation.md` §Helper-duplication. If it is gone, rebuild the table with `grep -n "fn src_root\|fn rel_posix\|fn all_rust_sources\|fn code_lines" src-tauri/tests/*.rs`.
- `segment_normalize_boundary.rs:100` and `webimport_boundary.rs:110` -- the two copies of `text_before_first_cfg_test_line` (8 call sites in webimport). `core/segment/role.rs` has test modules at L89 and L481, with product code between them.
- `ai_boundary.rs` (`is_inside_ai_module`) and `aiconfig_keychain_boundary.rs` -- the safe directory-check pattern to generalise.
- Bare `starts_with(DIR)` in 10 files, e.g. `scope_boundary.rs:175,222,251,289` and `store_boundary.rs:142,206,235`.
- `segment_boundary.rs:141` `is_comment` -- half-handles block comments; `store_boundary.rs:151,162,241,244` -- the comment filter is inlined.
- `dict_boundary.rs:957` test, `walk_any` at `:1013`, panicking read at `:981`. `:591` `mentions_a_dict_db_file`, `:860` `ordering_lacks_a_tiebreaker`.
- `tests/fixtures_docx.rs` + `segment_contract.rs:18-19` -- the `#[path]` sharing precedent, with `#[allow(dead_code)]` and a reason.
- Ballot `planning-artifacts/sprint-change-proposal-2026-09-24b-phieu-quyet.md:352` (#106) cites `b8f22f7:11211 (+ 11646)`. In `b8f22f7`, line 11646 is the tail of the `api_key` item. The closure line sits under the FR77 item instead (`deferred-work.md` ~:11873).
- HEAD has no `/*` on any non-comment line of `src-tauri/src`, so comment stripping changes no verdict today.

## Tasks & Acceptance

**Execution:**
- [x] `deferred-work.md` -- Task 0: re-read the 12 items (grep `Chủ: Story 11.1`, the lines above) and record any that self-closed.
- [x] `src-tauri/tests/support/boundary_scan.rs` (new; a subdirectory, so it is not its own test binary) -- `src_root`, `rel_posix`, `is_inside(rel, dir)`, `rust_sources(root)`, `code_lines` (lexes `"…"`, `r#"…"#` and char literals, strips `//` and nested `/* */`), `any_sources` (skips dotfiles, returns non-UTF-8 files as a named skip), and `without_test_modules` (Decision 3).
- [x] `src-tauri/tests/boundary_scan_contract.rs` (new) -- one case per I/O-matrix row.
- [x] The 18 `*_boundary.rs` files and `config_invariants.rs` -- include the module through `#[path]`, delete the local copies, and replace every bare `starts_with(DIR)` with `is_inside`.
- [x] The seven gates holding `text_before_first_cfg_test_line` -- switch to `without_test_modules`; fix any real violation this exposes.
- [x] `deferred-work.md` -- close all 12 items:
  - L302, L305, L6191, L7121, L7138, L9742 and L10367: `✅` with evidence.
  - L234 and L542: `KHÔNG LÀM` (Decisions 1 and 2).
  - L586 and L589: `KHÔNG LÀM`, with the reopen condition (multi-line SQL in `core/dict/**`; a `.db` name without `dict-`).
  - L11853: `→ KHÔNG LÀM 2026-09-24 (phiếu quyết #106)`.
  - The FR77 CI-anchor item: `→ 🟡` stating that the #106 line above belongs to the previous item, still `Chủ: Ice`.

**Acceptance Criteria:**
- Given lot B is done, when the 12 items are grepped, then each ends in an Epic 11 disposition, and `npm run check:debt-owner` is green.
- Given each migrated file, when it runs on HEAD, then its verdict and its scanned-file count equal the pre-migration measurement.
- Given each fixed defect, when its seam in `boundary_scan.rs` is really removed, then its contract case goes red for that reason, and green again once the seam is restored.
- Given an untracked `src/.DS_Store`, when `cargo test --test dict_boundary` runs, then it passes. Before the fix it panics. Delete the file afterwards.

## Implementation Notes

- `is_inside`/`rust_sources`/`code_lines` are thin re-exports at most call sites; where a file's own helper carried a NAME call sites depend on (`is_inside_ai_module`, `all_rust_sources`, `dict_sources`, …), that name stays as a one-line delegate to `boundary_scan::*` rather than touching every call site.
- `code_lines`'s item type moved from `&str` to owned `String` (block-comment masking can't stay a slice of the input). Most closures (`.filter(|(_, code)| …)`) needed no change (match-ergonomics + `&String`→`&str` deref coercion); `.any`/`.map`/`for` loops that bind the tuple by value needed a `&` added at the call site — mechanical, caught by `cargo test` one file at a time.
- `store_boundary.rs` and `segment_boundary.rs` had no separate `code_lines` fn (inline `code.starts_with("//")` loops) — folded into `boundary_scan::code_lines` too, since they're the same duplication class (L305/L7121) even though the ledger items don't name these two files directly.
- `without_test_modules` needed a third shape beyond "`mod NAME { … }`" (skip) and "malformed" (FAIL): `commands/project/mod.rs:5272` has `#[cfg(test)] mod tests;` (no body, defined in `tests.rs`). Both `segment_normalize_boundary.rs` and `webimport_boundary.rs` full-tree-scan every file including this one, so this shape had to be a third, valid, no-op case, not a FAIL — see `deferred-work.md` item L9742's closure for the measurement.
- Discovered mid-implementation: `text_before_first_cfg_test_line` has 7 copies, not 2 (`cleanup_boundary.rs`, `docx_boundary.rs`, `segment_chapterpattern_boundary.rs`, `segment_files_boundary.rs`, `segment_pipeline_boundary.rs` also carry one). 🔵 2026-09-25: Ice extended Decision 3 to all seven; the other five now switch to `without_test_modules` too, same as the first two — recorded in `deferred-work.md`'s L7138/L9742 closures.
- Two self-check tests per migrated file (`text_before_first_cfg_test_line_is_not_fooled_by…`/`…returns_the_whole_text_when…`) were deleted from all seven files carrying the old function — equivalent coverage now lives once in `boundary_scan_contract.rs`. This is why every migrated-from-`text_before_first_cfg_test_line` file's post-migration pass count is 2 lower than its own pre-migration baseline, while every other migrated file's count is unchanged.
- `dict_boundary.rs::dict_sources`/`ports` scan (`core/dict`, `ports/`) reuse `boundary_scan::rust_sources(&src_root())` filtered by `is_inside`, not a targeted-root walk — same file set, since a full-tree walk-then-filter and a subtree walk produce identical results.

## Spec Change Log

- 🔵 2026-09-24 — Code Map said "the two copies of `text_before_first_cfg_test_line`" (`segment_normalize_boundary.rs`, `webimport_boundary.rs`). Measured during implementation: 7 copies exist. 🔵 2026-09-25: the next sentence of this entry ("Decision 3's scope … still stands and was followed") is no longer true — see the 2026-09-25 entry below; Decision 3 now covers all seven, and all seven were migrated.

- 2026-09-25 — Review loop 1 (intent_gap): five more gates kept `text_before_first_cfg_test_line`, so L9742/L7138 closed ✅ over a live blind spot. Ice extended Decision 3 to all seven copies and chose to patch forward on the existing code rather than revert and re-derive. Amended: Problem sentence, Decision 3, the switch task. Known-bad state avoided: a ✅ on a blind spot still present in three `segment_*` gates. KEEP: `boundary_scan.rs`'s lexer and `without_test_modules` shapes, the 14 contract cases, all ledger dispositions except the L7138/L9742 wording.

## Review Triage Log

- orchestrator: five more gates (`cleanup`, `docx`, `segment_files`, `segment_pipeline`, `segment_chapterpattern`) keep their own `text_before_first_cfg_test_line`, so the post-test-module blind spot is still real (`core/segment/role.rs` L108-480 is invisible to the three `segment_*` gates), yet L9742 and L7138 close ✅. The frozen Problem said "two of them" and Decision 3 names two copies — intent_gap.
- blind: `code_lines` now masks trailing `//` comments, narrowing what gates see — false: only comment text is masked, no Rust gate reads a marker from a comment (grep over `*_boundary.rs`: 0), and strings/code stay verbatim.
- blind: `without_test_modules` panics on a bare `#[cfg(test)] fn` anywhere in the tree — false: a loud FAIL on an unsupported shape is Decision 3; `src-tauri/src` has 0 such lines today.
- blind: `all_rust_sources` reads every file and then `code_lines(file)` reads it again — low, rejected: ~70 small files, negligible cost, and the fix adds a path-only walker.
- blind: `dict_boundary` now reads the whole `src/` tree to filter `core/dict` — low, rejected: same reason.
- blind: `any_sources`'s `_skipped` is discarded in `the_webview_and_the_string_catalog_hardcode_no_source_identity`, so a non-UTF-8 source file drops out of the scan silently — medium, patch: `src/` holds only `.gitkeep` dotfiles besides text sources, so any `NonUtf8` skip there is an anomaly and must fail.
- blind+edge: `any_sources` labels every read error `NonUtf8` — low, patch (direct correction): only `ErrorKind::InvalidData` is `NonUtf8`, any other error panics.
- blind: the two `catch_unwind` contract cases print the expected panic and its backtrace on every run — low, patch: use `#[should_panic(expected = …)]`.
- blind: the rationale for filtering blank lines was deleted with a per-file comment — false: the behaviour is kept and stated in `code_lines`'s doc; review-round history is banned in comments (AGENTS.md §Code comments).
- blind: `mod tests` with `{` on the next line panics — false: loud FAIL on a shape the tree never uses (rustfmt style), per Decision 3.
- blind: no full `cargo test` run is recorded — false: the implementer ran it (66 binaries, 0 failed), and command logs don't go in the spec; it is re-run after the patches anyway.
- edge: a `#[cfg(test)]` line inside a string or block comment is taken as a real anchor — low, rejected: 0 in the tree, the old copies share it, and the fix needs an anchor search through the lexer.
- edge: the deleted "no `#[cfg(test)]` ⇒ text unchanged" self-check has no replacement, so a passthrough regression would let `webimport_boundary` scan nothing and stay green — medium, patch: add the identity case to `boundary_scan_contract.rs`.
- verification-gap: `ai_boundary.rs:1008-1018` still says `code_lines` keeps trailing `//` and `/* */` — low, patch: the premise is now false; fix the comment.
- verification-gap: `naming_boundary.rs:83` justifies its own stripper by "the other `*_boundary.rs` only drop `//` lines" — low, patch: the premise is now false; fix that one sentence only.
- orchestrator: comments added or changed in the diff break AGENTS.md §Code comments (Vietnamese, story/lot ids, "Quyết định 3", history, counts such as "37/38", "18 tệp", "2/18") in `boundary_scan.rs`, `boundary_scan_contract.rs` and the migrated files — medium, patch.
- orchestrator: `segment_boundary.rs` keeps its own multi-extension `walk` for the webview tree — low, patch: the task says delete local copies.
- r2 edge: `matching_boundary.rs:236,246` still use bare `rel.starts_with("core/dict")`, so a sibling `core/dict_legacy` satisfies the `dict_files >= 1` floor even if `core/dict` is empty — medium, patch: use `is_inside`.
- r2 blind: no contract case asserts the 1-based line number `code_lines` reports, while every migrated violation message changed its index arithmetic — low, patch: one case pinning line numbers.
- r2 blind: `#[allow(dead_code)]` reasons name only some of the unused helpers — low, patch: one generic reason (the module is shared, not every helper is used here).
- r2 blind: `naming_boundary.rs` is edited despite the Never list — low, rejected: the Never guards its scan logic; the one-sentence edit keeps its comment true.
- r2 blind: the Spec Change Log says 14 contract cases, there are 15 — rejected: the fix edits this spec.
- r2 blind: `#[cfg(all(test, …))]` is not an anchor — low, rejected: 0 in the tree, and the result is a louder scan (a possible false red), not a silent green.
- r2 blind: `#[cfg(test)]` followed by another attribute before `mod` panics — carried: loud FAIL on an unsupported shape (Decision 3).
- r2 blind: hidden directories are walked — low, rejected: none under the scanned roots, and the fix adds a branch.
- r2 blind: Verification omits the CI/nightly read — rejected: the fix edits this spec; the read happens before `done`.
- r2 blind: one-line delegate wrappers keep the old private names — low, rejected: recorded in Implementation Notes, and removing them touches every call site.
- r2 blind: `dict_boundary` walks the whole tree to filter a subtree — carried: low, rejected in loop 1.
- r2 edge: a one-line `#[cfg(test)] mod tests {` is not stripped — low, rejected: 0 in the tree, and the result is a louder scan, not a silent green.
- r2 edge: a `#[cfg(test)]` line inside a string is taken as an anchor — carried: low, rejected in loop 1.
- r2 edge: a dotfile other than `.gitkeep` with source text is skipped silently — low, rejected: only `.gitkeep` exists under `src/`, and the fix adds an allowlist guard.
- r2 edge: `webimport_boundary` panics on an unrelated file's attribute shape — carried: false in loop 1 (Decision 3).
- r2 edge: the Problem statement says no copy stripped `/* */`, yet three stripped `/*`, `* `, `*/` line prefixes — rejected: the fix edits this spec.
- r2 verification-gap: no gaps found.

## Verification

**Commands:**
- `npm run build` first, then `cargo test --test boundary_scan_contract` plus each touched `--test <name>` -- expected: green.
- The full `cargo test` once, by hand: 19 test targets change together, so a failure could depend on cross-module order (AGENTS.md §Tests).
- `npm run check:debt-owner` -- expected: green.
