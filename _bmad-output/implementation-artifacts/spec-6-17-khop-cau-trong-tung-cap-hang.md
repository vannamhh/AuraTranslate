---
title: 'Story 6.17 — Manual sentence alignment inside each bilingual row'
type: 'feature'
created: '2026-09-12'
status: 'done' # draft | ready-for-dev | in-progress | in-review | done
route: 'dispatch' # oneshot | dispatch
baseline_commit: '66c76ccfda3a01bf1ceea69d956a3df2b36ae7ee'
review_loop_iteration: 0
context:
  - '{project-root}/AGENTS.md'
  - '{project-root}/src-tauri/AGENTS.md'
  - '{project-root}/src/AGENTS.md'
---

<frozen-after-approval reason="human-owned intent — do not modify unless human renegotiates">

## Intent

**Problem:** FR116 — Story 6.16 lists every row whose two cells split into different sentence
counts and locks confirm, but offers no way to fix the row inside the app: a file with one
translator-merged sentence cannot be imported at all.

**Approach:** Inside the bilingual preview, before any byte is written, the user regroups a
mismatched row's TARGET sentences — join adjacent ones, split one at a chosen point — until the
count equals the source count; that row then pairs in order like an equal row. Fully keyboard-
operable. Rust re-validates every regrouping at rebuild and at confirm.

### Decisions (Ice, 2026-09-12)

- **Target side only.** Regrouping never touches the source column; every segment keeps exactly
  one source sentence. The mockup's "gộp câu nguồn" proposal is not built.
- **Machine proposal plus one explicit bulk accept.** Each mismatched row whose two sides both
  have at least one sentence carries a proposed set of cuts; one command applies every proposal
  at once. A proposal is never applied without an explicit act.
- **"Bỏ qua hàng này" exists only where one side has 0 sentences.** The source sentences import
  untranslated (empty target, no `bilingual_import` origin); a 0-vs-n row drops its translation
  and preview says how many were dropped. A row with both sides non-empty must be regrouped.
- **Full spec kept** though it is over the 1600-token guide (estimated ~2 900).

## Boundaries & Constraints

**Always:**
- AD-5 §FR116: regrouping lives in preview only; 0 segments exist yet, so no retire + create.
- AD-4: boundaries are fixed once, at confirm; regrouping is input to that single computation.
- AD-37/AD-46 row flags unchanged: last segment of a row on (both sides), others off, last
  segment of a Chapter off. A cut never sets a flag.
- A regrouped row writes exactly like an equal row: `target_text` + `bilingual_import` in the
  same INSERT, status `draft`, Chapter `in_progress`.
- A regrouping applies only while the row's re-derived source sentences and target text equal
  the ones it was made against; otherwise the row returns to the mismatch list, visibly.
- Confirm stays refused in Rust while any row is unresolved (6.16 refusal kept, not only UI).
- Cut positions are Unicode-scalar offsets (never bytes); any number from the webview is
  tolerated without a panic point (`panic = "abort"`).
- A proposal is `source_count - 1` cuts on the canonical target line, taken from candidate
  boundaries — the machine sentence boundaries first, then word boundaries — nearest the positions
  the source sentence lengths imply. It is a starting point shown in preview, applied only by an
  explicit act, and it is the same rule for a join and for a split.
- A skipped row still starts a Chapter when its source cell matches the pattern, and its segments
  follow the same row flags as any other row.
- Equal-count rows, the prose import path and `split_source_text` output stay byte-identical.

**Never:**
- No "import now, join later" / partial import (AD-5 §FR116, AC "before any segment is written").
- No editing of sentence TEXT — only where boundaries fall.
- No skip on a row whose two sides both have at least one sentence.
- No source-side regrouping, and no proposal that merges source sentences.
- No new crate, no migration (schema v22), no `tauri.conf.json`/`capabilities/**`/CSP change.
- No new override `Mutex` state: regroupings travel as a per-call param, like `chapter_pattern`.
- No identifier containing `Document`/`Project`/`Book`/`Novel`; no bare `origin`.
- Do not edit `epics.md`/`prd.md`/mockups to match the code.

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected Output / Behavior | Error Handling |
|---|---|---|---|
| Split target | Source 2, target 1; one cut at a word boundary | Row resolved; 2 segments, both texts, flags off/on | N/A |
| Join target | Source 2, target 3; boundary t1/t2 removed | 2 segments; first target `t1 t2` | N/A |
| Still unequal | Source 2, target pieces 3 after edits | Row still listed; confirm disabled; Rust confirm refused, 0 Work | Existing typed error |
| Bad cut on wire | Cut 0, = length, duplicate, past end, in multi-byte char | Row stays unresolved; no panic | Row counted as mismatch |
| Stale regrouping | Cuts made, then column swap / encoding change alters the row | Row listed again, cuts dropped | N/A |
| Unaffected regrouping | Cuts made, then header toggle not touching the row | Row stays resolved | N/A |
| Proposal shown | Source 2, target 1 | Row shows proposed cut; nothing pairs until accepted | N/A |
| Bulk accept | 3 rows with proposals, 1 row 1-vs-0 | One act resolves the 3; the 1-vs-0 row untouched | N/A |
| Skip blank target | Heading row 1-vs-0, skipped | 1 untranslated segment, empty target, no `bilingual_import`; row resolved | N/A |
| Skip blank source | 0-vs-2 row, skipped | Translation dropped, count shown in preview; 0 segments from the row | N/A |
| Skip refused | 2-vs-1 row, skip attempted from the wire | Row stays unresolved, nothing written | Typed error |
| Keyboard only | No pointer | Move between mismatches, move caret, toggle cut, accept all, skip, confirm | N/A |
| Cancel | Cuts made, then cancel | 0 Work; regrouping state cleared | N/A |
| Equal rows | No mismatch in file | Identical result to 6.16 | N/A |

</frozen-after-approval>

## Code Map

Line numbers drift — re-locate by symbol.

**Rust**
- `src-tauri/src/core/segment/pipeline.rs` `split_segments_step` bilingual branch (~`:1532-1576`) —
  mismatched rows `continue`; regrouping applies here, before row flags. `PipelineOutput::bilingual_mismatches` (~`:439`).
- `src-tauri/src/core/segment/bilingual.rs` `BilingualMismatch` (~`:143`, 4 fields), `BilingualSegment`.
- `src-tauri/src/core/segment/split.rs:225` `split_source_text` → `Vec<SplitSegment{text, is_paragraph_end}>`,
  no offsets — do not change it.
- `src-tauri/src/core/segment/regroup.rs:246` `split_at` — precedent only (post-persist): `chars()`
  not bytes, sort cuts, reject 0 / ≥ len / duplicates, `get` instead of indexing.
- `src-tauri/src/commands/project.rs` `BilingualMismatchWire` (~`:3424`); `preview_bilingual_import_from_file`
  (~`:5989`), `rebuild_bilingual_import_preview` (~`:6045`), `confirm_bilingual_import` (~`:6084`,
  core ~`:3636`); `create_work` refusal `BilingualMismatchedRows` (~`:470`). The 6.9/6.15 override
  Mutexes are index-based with no staleness check — not a model here.
- `src-tauri/src/lib.rs` registration; `tests/ipc_contract.rs:978` by-name guard (rebuild never reads the file).

**Webview**
- `src/BilingualImportPreviewOverlay.vue` mismatch section (~`:287-316`), Tab trap (~`:120-146`).
- `src/bilingualImportPreviewState.ts` `refresh` (~`:115`) + `sequence` guard; `bilingualImportPreviewCanConfirm`
  (~`:98`); `resetBilingualImportPreview` is the sole full reset (`check:panel-refs` Kiểm A).
- `src/config/project.ts:614-696` bilingual wire types + runtime guards.
- `src/commands/index.ts:1213-1254` bilingual commands + `CommandDeps`; `src/main.ts:472-485` wiring.
- Keyboard precedent `src/ImportPreviewOverlay.vue:636-767` `onScrimKeydown`: filters form fields, guards
  `event.repeat`, `⌥` via `event.code`, every key `dispatch`es a registered command. `check:commands`
  does not scan `@keydown` — the vitest suite is the guard.
- Mockup `ux-designs/ux-AuraTranslate-2026-08-02/mockups/bilingual-import.html` screen ② — layout
  "Chỗ lệch i / N · hàng r · Chương c", source sentences left, target right, `↑ ↓` between mismatches.

**Tests that move** — `bilingual_import_contract.rs:550-621` pins `BilingualMismatchWire` fields;
positional `confirm_bilingual_import` calls across that file; positional `create_work` calls in
`segment_contract.rs`. Precedents: `tests/frontend/importPreviewBlocks.test.ts:372-495` (keydown on
a mounted overlay), `tests/frontend/importPreviewBilingual.test.ts:21-75` (IPC fake).

## Tasks & Acceptance

**Execution:**
- [x] `src-tauri/src/core/segment/bilingual.rs` -- regrouping type (cuts or skip) + pure apply/validate fn + pure proposal fn, 0 panic points -- one rule for rebuild and confirm.
- [x] `src-tauri/src/core/segment/pipeline.rs` -- mismatch carries source sentences, canonical target line and its proposal; a valid regrouping pairs the row, a skip writes the source side untranslated -- AD-37 flags.
- [x] `src-tauri/src/commands/project.rs` + `src-tauri/src/lib.rs` -- regroupings param on rebuild and confirm; wire fields -- Rust stays the gate. (`lib.rs` needed no edit: both wires are registered by name, and the by-name guard `ipc_contract.rs::the_three_bilingual_import_wires_are_registered_...` passes unchanged.)
- [x] `src/config/project.ts` -- types, guards, adapters.
- [x] `src/bilingualImportPreviewState.ts` -- per-row cuts, caret, mismatch cursor; sent on rebuild/confirm; cleared only in the reset fn.
- [x] `src/BilingualImportPreviewOverlay.vue` -- focused-mismatch view per mockup ②, proposal shown, accept-all and skip surfaced only where allowed; keydown → `dispatch`.
- [x] `src/commands/index.ts` + `src/main.ts` + `src/i18n/vi.json` -- new `import.preview.bilingual_*` commands (navigate, caret, toggle cut, accept all, skip) and copy, impersonal voice.
- [x] `src-tauri/tests/bilingual_import_contract.rs` -- every Matrix row on the product path; move the pinned wire case.
- [x] `tests/frontend/importPreviewBilingual.test.ts` -- keyboard-only Matrix row, cancel, stale.

**Acceptance Criteria:**
- Given the cut validation removed (0 / ≥ length / duplicate accepted), when the new suite runs, then a named bad-cut case is RED.
- Given the staleness check removed, when the new suite runs, then the stale-regrouping case is RED.
- Given the Rust-side refusal for unresolved rows removed, when the suite runs, then it is RED.
- Given the proposal function replaced by a fixed midpoint cut, when the suite runs, then a named proposal case is RED.
- Given the guard that limits skip to rows with a 0-count side removed, when the suite runs, then a named case is RED.
- Given one keydown → `dispatch` line removed from the overlay, when vitest runs, then a keyboard case is RED.
- Given the eleven `check:*` gates + `npm run test` + `npm run build` + `cargo test --locked` in pre-push order, when run with no other cargo process, then 0 findings; `check:debt-owner` 0 orphans.

## Implementation Notes

**Implementation agent's own report (2026-09-12):** gates, `npm run test`, `npm run build` and a
full `cargo test --locked` run green; it stated plainly that it had run NONE of the red controls.
Everything below is the orchestrator's own measurement on the final tree, and where a measurement
contradicts an expectation, the measurement is what is recorded.

**Red controls — one mutation per seam, guarding suite = `cargo test --locked --lib --test
bilingual_import_contract` (M7: the one vitest file), each file restored and byte-compared
identical after the run (verified programmatically, all 7):**

| # | Mutation (product code) | Result |
|---|---|---|
| M1 | `apply_cuts`: both explicit cut guards (duplicate / 0 / ≥ length) deleted | **STILL GREEN** — see below |
| M2 | `apply_cuts`: empty-piece rejection deleted | RED — `apply_cuts_rejects_a_cut_that_leaves_an_empty_piece` |
| M3 | `resolve`: staleness snapshot comparison deleted | **STILL GREEN at first** — see below; RED after a new case |
| M4 | `create_work`: mismatch refusal disabled (`if false && …`) | RED — 3 cases, incl. `still_unequal_after_a_bad_cut_count_…` and `a_mismatched_row_is_listed_in_preview_…` |
| M5 | `propose_cuts`: proportional target replaced by a fixed midpoint | RED — `a_proposal_cuts_at_the_candidate_boundary_nearest_the_source_length_ratio` |
| M6 | `resolve`: skip-only-on-an-empty-side guard replaced by `Ok(Skipped)` | RED — `skip_is_refused_with_a_typed_error_when_both_sides_have_sentences` |
| M7 | overlay: the `Enter` → `bilingual_toggle_cut` dispatch deleted | RED — vitest, the seven-key case |

**M1 stayed green, and that is correct, not a hole.** The two `if`s are redundant: a cut of 0, a
duplicate cut, or a cut at the end all produce an empty piece, which the empty-piece rejection
(M2) refuses, and a cut past the end makes `chars.get(start..end)?` return `None`. The effective
guard is M2; the two `if`s are defence in depth and were left in place.

**M3 exposed a real hole and it was closed.** With the snapshot comparison deleted, every existing
"stale" case still passed — they go red for a different reason (the old cuts no longer produce as
many pieces as the row has source sentences), so none of them guarded the snapshot itself. A stale
regrouping whose piece count happens to fit would have been applied silently to changed text. Added
`bilingual.rs::a_stale_regrouping_whose_cut_count_still_fits_is_refused_by_the_snapshot_alone`
(2 source sentences before and after, old cut still yields 2 non-empty pieces — only the snapshot
can refuse it); re-measured with the same mutation: RED on exactly that case.

**M5 was never measured green.** Reading the existing proposal cases showed they only assert that
the proposal resolves the row — a midpoint cut also resolves a 2-vs-1 row — so the case
`a_proposal_cuts_at_the_candidate_boundary_nearest_the_source_length_ratio` was written first
(short source sentence + long one ⇒ the cut must land on the word boundary after `aa`, not at the
line's midpoint), and the mutation then went red on it.

**Matrix test audit.** Every row had a covering case that ran and passed, except **"Unaffected
regrouping"**: the Rust unit case carrying that name calls `resolve` twice with identical cells and
never toggles a header, so nothing covered the product path. Added
`bilingual_import_contract.rs::a_regrouping_survives_a_header_toggle_that_does_not_touch_its_row`
— cuts made with the header off, `has_header` then turned on (row 1 dropped, the edited row keeps
its file row number and its two cells), row still pairs, and the confirm writes the two segments.

**Verification on the final tree (2026-09-12, no other cargo process running):**
- `npm run build`: exit 0. `npm run test`: 1 008/1 008 on 74 files.
- Every `check:*` script in `package.json` (13, the eleven pre-push gates plus `check:scope`
  and `check:scope:bundled`): each exit 0. `check-debt-owner.mjs --report`: 0 open items without `Chủ:`.
- `cargo test --locked`: exit 0 — 54 `test result: ok` lines, 0 failures;
  `bilingual_import_contract` 32/32, lib unit tests 201/201.
- `git diff -- src-tauri/tauri.conf.json src-tauri/capabilities src-tauri/Cargo.toml
  src-tauri/Cargo.lock package.json package-lock.json`: empty.

**Review pass 1 — patches applied and re-verified (2026-09-12).** Six patch rows went to the
implementation agent. EC-4's first fix summed the dropped-sentence total over rows still in the
mismatch list, so it fell back to 0 the moment the rebuild resolved the skipped row — the number
was visible for exactly one IPC round-trip, and the new vitest case asserted that (2 before the
await, 0 after). Sent back: the total is now accumulated in Rust over the rows `split_segments_step`
actually resolved as `Skipped`, carried on `PipelineOutput` and on the candidate wire, and only
read in TypeScript.

**Red controls re-measured on the FINAL tree** (the patches changed every file that carries these
seams, so the first measurement was of a different tree). One mutation per seam, guarding suite
`cargo test --locked --lib --test bilingual_import_contract` (M7: the one vitest file), each file
restored and byte-compared identical:

| # | Mutation | Result |
|---|---|---|
| M1 | `apply_cuts`: both explicit cut guards deleted | STILL GREEN — redundant, see above |
| M2 | `apply_cuts`: empty-piece rejection deleted | RED — `apply_cuts_rejects_a_cut_that_leaves_an_empty_piece` |
| M3 | `resolve`: staleness snapshot comparison deleted | RED — `a_stale_regrouping_whose_cut_count_still_fits_…` |
| M4 | `create_work`: mismatch refusal disabled | RED — 3 cases |
| M5 | `propose_cuts`: midpoint instead of the proportional target | RED — `a_proposal_cuts_at_the_candidate_boundary_…` |
| M6 | `resolve`: skip-only-on-an-empty-side guard removed | RED — `skip_is_refused_with_a_typed_error_…` |
| M7 | overlay: `Enter` → toggle-cut dispatch deleted | RED — vitest, the seven-key case |
| M8 | `split_segments_step`: skipped-sentence total never accumulated | RED — `skip_on_a_blank_source_drops_the_translation_and_yields_zero_segments` |

**Final verification (2026-09-12, tree standing still, no other cargo process):**
- `npm run build`: exit 0. `npm run test`: 1 009/1 009 on 74 files.
- All 13 `check:*` scripts: exit 0. `check:debt-owner` re-run after the new debt entry: 0 open
  items without `Chủ:`.
- `cargo test --locked`: exit 0, 54 `test result: ok` lines. Baseline for the red controls:
  lib 202/202, `bilingual_import_contract` 33/33.
- Pinned-config diff (`tauri.conf.json`, `capabilities`, `Cargo.toml`, `Cargo.lock`,
  `package.json`, `package-lock.json`): empty.

**Two measurements that were thrown away, and why.** (1) One verification run was taken while the
implementation agent was still editing — `check:scope:bundled` went red on a half-written type in
the vitest helper. That number measured a moving tree, so it was discarded and re-run on a still
one. (2) `check:scope` timed out at its own 300 s self-check on the next run: the agent had run
`cargo clean`, so the gate was still compiling 496 of 504 crates when its clock ran out. Re-run on
a warm cache: pass, both directions.

**One environment failure, filed as debt, not caused by this story.** `cargo test --locked` exited
101 with `dict_boundary::the_webview_and_the_string_catalog_hardcode_no_source_identity` panicking
on `đọc .DS_Store: stream did not contain valid UTF-8`. That test reads every file under `src/` as
UTF-8; a Finder `.DS_Store` written the same day made the whole suite red, and because `.DS_Store`
is git-ignored it never shows in a diff. The file was deleted and the suite went green; the fragile
walk is recorded in `deferred-work.md` with an owner.

## Spec Change Log

## Review Triage Log

### Review pass 1 — 2026-09-12 (blind-hunter · edge-case-hunter · verification-gap)

| # | Finding (layer) | Verdict | Evidence | Route |
|---|---|---|---|---|
| EC-4 | The frozen Matrix row "Skip blank source" says the preview shows how many target sentences a skipped 0-source row drops; nothing computes or shows it (edge-case) | medium | Verified: `BilingualMismatchWire` carries no sentence count any more, the overlay shows only `bilingual_row_count`/`bilingual_chapter_count`/`bilingual_pair_count` (`BilingualImportPreviewOverlay.vue:409-413`), and `grep` finds no dropped-count string anywhere. The contract case `skip_on_a_blank_source_drops_the_translation_and_yields_zero_segments` asserts 0 segments only — my own matrix audit checked that half and missed this one | patch |
| BH-2 | No test deserializes `BilingualRegroupingWire`, so the serde renames `"cuts"`/`"skip"` that must match the TypeScript literals are unguarded (blind) | medium | Verified: the wire types appear only in `commands/project.rs`; no `serde_json::from_*` anywhere in `bilingual_import_contract.rs`; `ipc_contract.rs` does a textual source scan, not a decode. A rename typo would surface only in the running app | patch |
| VG-1 | The skip-with-blank-target fan-out (one untranslated segment per source sentence) is only ever exercised with ONE source sentence (verification-gap, pre-verified) | low | Filed with evidence; re-checked: all three skip fixtures are single-sentence (`bilingual.rs:625`, contract `"Chuong Mot,\n"` and `"Heading Day.,\n"`). Joining the sentences into one pair would keep every case green | patch |
| BH-1 | `piecesFor`'s doc comment says it does not trim; the body trims every piece (blind) | low | Verified at `BilingualImportPreviewOverlay.vue`: comment says "không trim ở đây", body calls `.trim()` on each slice. Direct correction | patch |
| BH-8 | The keyboard hint presents `S` as always available, but skip only applies to a row with an empty side (blind) | low | Verified: `mode.library.preview.bilingual_keyboard_hint` says "S bỏ qua hàng" with no condition, while `bilingualImportPreviewCanSkipActiveRow` gates it. Direct text correction | patch |
| VG-2 | `refresh()` keeps the caret when the focused row is still listed even though a rebuild can change that row's `target_line`/`candidate_positions` (verification-gap, pre-verified, filed as defer) | low | Filed with a full walkthrough (column swap keeps `row_number` but changes both). No data-integrity effect — Rust's snapshot check still refuses a stale cut — but caret stepping jumps to an end. Fix is one condition, so it goes in now rather than to the debt ledger | patch |
| EC-1 | `resolve` would return `SkipNotAllowed` for a 0-vs-0 row, whose message claims both sides have sentences (edge-case) | false | A 0-vs-0 row has equal counts, so `split_segments_step` pairs it and `continue`s before any regrouping is consulted — `resolve` is never called for it | reject |
| EC-2 | Two regroupings with the same `row_number`: `find` takes the first, the second is dropped silently (edge-case) | low | Real for a hand-built call, unreachable from the product path — the webview keys regroupings by `row_number` in a `Map`. Fix would add a dedupe/refusal branch for a state never demonstrated | reject |
| EC-3 | Toggling a cut on a row with 0 source sentences stores `Cuts` that Rust always discards; the UI shows pieces that never pair (edge-case) | low | Reachable (such a row has candidate positions) but harmless: the row stays listed, skip still resolves it, nothing is written. Fix adds a guard branch | reject |
| BH-3 | Per-row linear scan over regroupings on every rebuild (blind) | low | Real; the sets are one entry per mismatched row, so the work is negligible at the sizes this screen handles. Fix swaps in a map for no measured gain | reject |
| BH-4 | Only the focused mismatch is shown; no way to jump to a specific row (blind) | false | This is the approved design, not a defect: mockup ② is exactly "Chỗ lệch i / N" with `↑ ↓` stepping | reject |
| BH-5 | The form-field filter treats `<button>` like a text field, so the seven keys do nothing while a row-action button has focus (blind) | low | Verified in `onMismatchKeydown`. Filtering buttons is deliberate — `Enter` on a focused button already activates it, and letting it also toggle a cut is worse. Separating arrows from `Enter`/`A`/`S` adds branches | reject |
| BH-6 | An invalid skip short-circuits on the first offending row, so only one row number is ever reported (blind) | low | Real, but the state is a webview contract violation the UI cannot produce (skip is offered only where a side is empty). Aggregating would add surface for an unreachable case | reject |
| BH-7 | `propose_cuts` can return fewer cuts than needed and "accept all" gives no per-row feedback when it does (blind) | low | Real per its own doc comment; needs a Vietnamese target line with no word boundary at all. The row simply stays listed. A fix adds new UI surface | reject |
| BH-9 | The template `v-if` carries a redundant `activeMismatch !== null` clause (blind) | false | Not redundant: the template dereferences `.row_number`/`.source_sentences` on it, and `vue-tsc` narrows through that clause. Deleting it breaks the type-check | reject |
| BH-10 | No test covers a typed refusal raised by a NON-selected encoding candidate inside the preview loop (blind) | low | Real gap, but the behaviour there is to swallow the error deliberately (`is_selected` branch) — a candidate that cannot parse must not break the strip. The selected-candidate path is covered | reject |
| BH-11 | The "empty `target_text` ⇒ no `bilingual_import` origin" inference has no assertion at the insert boundary (blind) | low | The invariant is enforced upstream: `apply_cuts` rejects any empty piece, and red control M2 proves that rejection is guarded. A `debug_assert` at the INSERT cannot see the pairing that would break it | reject |
| BH-12 | Three template helpers are plain functions, not `computed()` (blind) | low | Style inconsistency on short strings, no named harm | reject |

## Design Notes

**Representation.** Canonical target line = the row's machine target sentences joined by one
U+0020. A regrouping is `{row_number, source_sentences, target_line, cuts | skip}` — cuts are
scalar offsets into `target_line`; the machine boundaries are the initial cuts, so a join removes
a cut and a split adds one, and the proposal is just another set of cuts. Pieces are trimmed; an
empty piece invalidates the row. Echoing the two texts is the staleness check: Rust applies a
regrouping only when the re-derived row is identical, so a column swap or an encoding change can
never pair a stale cut silently.

**Keys.** `↑ ↓` previous/next mismatch, `← →` caret between word boundaries of the target line,
one command toggles the cut at the caret, one accepts every proposal, one skips a 0-side row. No
bare `Space`; no `Esc` rebinding — `Esc` already cancels the overlay
(`BilingualImportPreviewOverlay.vue:153`).

## Verification

**Commands:**
- `npm run build && (cd src-tauri && cargo test --locked)` -- pre-push order, exit 0.
- `npm run test` -- green.
- Each `check:*` gate -- 0 violations; `node scripts/check-debt-owner.mjs --report` -- 0 orphans.
- Red controls in §Acceptance -- counts and case names recorded in §Implementation Notes.
- `git diff -- src-tauri/tauri.conf.json src-tauri/capabilities src-tauri/Cargo.toml` -- empty.

**Manual checks (if no CLI):**
- CSV with one 2-vs-1 row: keyboard only, split the translation, confirm; Chapter shows two paired segments, unconfirmed.
