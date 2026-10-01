---
title: 'Story 7.2 — Origin on each TM pair'
type: 'feature'
created: '2026-10-01'
status: 'in-progress'
route: 'dispatch'
baseline_commit: '266c155f5ab91c00337cd065b50e81a668efee13'
review_loop_iteration: 0
context:
  - '{project-root}/AGENTS.md'
  - '{project-root}/src-tauri/AGENTS.md'
  - '{project-root}/_bmad-output/implementation-artifacts/epic-7-context.md'
---

<frozen-after-approval reason="human-owned intent — do not modify unless human renegotiates">

## Intent

**Problem:** Story 7.1 stores `tm_unit.translation_origin` as a free string copied from the confirm verdict, so nothing stops `''` (AD-47 ⑥: "no pair is written") or an unknown value from entering TM, and no code states the FR118 mine/others projection. Only the `self` and AI-`other` branches are guarded at the pair level; `bilingual_import` verbatim and "rewrite someone else's sentence ⇒ `self`" (Story 7.2 AC3) are not. The comparison baseline also lives only in the webview snapshot and is reset by every chapter load, so a rewrite that was auto-saved before a chapter switch or app restart is later confirmed as verbatim and enters TM as `other`/`bilingual_import`.

**Approach:** Give TM its own closed origin type with an exhaustive mine/others projection, make `insert_pair` accept only that type so `''` cannot reach `tm_unit`, and guard every FR117 branch at the pair level, including an edited 200-chapter-style bilingual Work (AC5: no skipping). Persist the comparison baseline in Rust so it survives chapter reload and restart (D1).

**Decisions (Ice, 2026-10-01):**
- D1 The baseline lives in Rust, set by every AD-47 ③ write, and `confirm_segment` compares against it instead of trusting the webview's load snapshot. This amends AD-47 ③ (chapter-load row) and Decision #2(b), so it needs a new AD drafted by Winston first (brief: `planning-artifacts/ad-brief-2026-10-01-moc-so-xuat-xu-luu-phia-rust.md`). Implementation of D1 waits for that AD; its shape (columns, migration 28, IPC) comes from the AD, not this spec.

## Boundaries & Constraints

**Always:**
- The value set stays exactly three plus `''` (AD-47 ⑥); `''` never reaches `tm_unit`.
- `self` projects to mine; `other` and `bilingual_import` project to others. The projection is an exhaustive `match`, so a new value fails to compile until it declares its side.
- Zero added user actions (FR117).
- Pairs are written only at the draft→confirmed transition (AD-31); existing rows are never updated (AD-6).
- `core/tm` never names `core::ai` or `crate::commands` (AD-13).

**Never:**
- No read API, sort, scope union, matching, UI, or RAG change (Stories 7.3–7.11).
- No change to the AD-47 ④ merge rule or the ⑤ restore exception.
- No bare `origin` identifier.

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected Output / Behavior | Error Handling |
|----------|--------------|---------------------------|----------------|
| Typed then confirmed | draft, origin `''`, user types `B` | pair (`A`,`B`,`self`) | N/A |
| Bilingual verbatim | imported `B`, origin `bilingual_import`, confirm unchanged | pair origin `bilingual_import` | N/A |
| Bilingual rewritten | imported `B`, user types `C`, confirm | pair origin `self` | N/A |
| Other verbatim / rewritten | AI-promoted `B` (`other`); confirm `B` / `C` | `other` / `self` | N/A |
| Edited Work, many chapters | bilingual Work, some sentences verbatim, some rewritten, all confirmed | one pair per confirm, none skipped, each with its own origin | N/A |
| Stale `''` echo | text unchanged, disk origin non-empty, `origin_at_load = ''` | nothing written | existing `ConfirmReject::UnknownOrigin`, transaction rolls back |
| Rewrite survives reload (D1) | bilingual `B`, user types `C`, flush, chapter reopened or app restarted, confirm `C` | pair origin `self` | N/A |

</frozen-after-approval>

## Code Map

- `src-tauri/src/core/tm/mod.rs` -- `insert_pair` (takes `translation_origin: &str`); `SimilarSegment` stays untouched.
- `src-tauri/src/commands/segment.rs` -- origin constants `:2045-2088`; `confirm_segment` `:2437`, verdict `:2558`, `insert_pair` call `:2577`; `ConfirmReject::UnknownOrigin` reused for the stale-`''` case.
- `src-tauri/src/commands/segment.rs:185-211` -- `insert_bilingual_segments` (draft, `bilingual_import`); caller `commands/project/work_creation.rs:496`. Reuse it in tests via the bilingual Work creation path used by `tests/bilingual_import_contract.rs`.
- `src-tauri/tests/tm_contract.rs` -- helpers `work`, `type_text`, `tm_rows`; `promoting_onto_a_confirmed_segment…` (`:286`) is the `other` model.

## Tasks & Acceptance

**Execution:**
- [x] `src-tauri/src/core/tm/mod.rs` -- add a closed pair-origin type (three variants, `from_stored`, `as_str`, exhaustive mine/others projection); `insert_pair` takes it -- `''` becomes unrepresentable.
- [x] `src-tauri/src/commands/segment.rs` -- parse the verdict into that type before any write; a `''` verdict rejects with `UnknownOrigin`.
- [x] `src-tauri/tests/tm_contract.rs` -- one case per matrix row, plus a projection case pinning all three values.
- [ ] D1 -- blocked on the new AD; expand into per-file tasks when it lands. Closes ledger item "Rust TIN mốc do webview khai" (`deferred-work.md:3693`).

**Acceptance Criteria:**
- Given `tm_contract.rs`, when the rewrite-to-`self` comparison is removed from the verdict, then the "rewritten" cases go red and the "verbatim" cases stay green.
- Given the stale-`''` case, when the type check is bypassed, then that case goes red because a row with origin `''` appears.

## Implementation Notes

- `PairOrigin` lives in `core::tm` and spells the three strings itself (`core` cannot name `crate::commands`); the projection case pins them against `TRANSLATION_ORIGIN_*`, so the two lists cannot drift silently.
- Counter-check (real removal of `target_nfc != text_at_load_nfc` from the verdict): 4/21 `tm_contract` cases red (three new rewrite cases plus `the_same_pair_confirmed_twice_over_a_detour…`), every verbatim case green.
- AC2 has no runtime counter-check: once `insert_pair` takes `PairOrigin`, bypassing the type does not compile. The stale-`''` case pins the reject path and the absence of any row.
- The edited-Work case uses 60 sentences in one bilingual Work; AC5 is about not skipping, not chapter count.
- D1 and its matrix row stay open until the new AD lands; the story cannot reach `done` before then.

## Spec Change Log

## Review Triage Log

Pass 1 (blind-hunter, edge-case-hunter, verification-gap; verification-gap filed no gap):
- B1/E4 two origin vocabularies can drift — low, rejected: `TRANSLATION_ORIGINS` is still read by `reject_unknown_translation_origin`, `segment_contract::the_translation_origin_catalogue_matches_ad_47_row_by_row` reds on a fifth value, and the projection case pins `PairOrigin` to the constants.
- B2/E5 `PairSide`/`side()` has no production caller — false as a defect: the spec's Always requires the exhaustive projection; readers arrive in 7.3/7.9/7.11.
- B3a `None` from `from_stored` may skip silently — false: it rejects with `UnknownOrigin` and rolls back (`segment.rs:2563`).
- B3b/E1 a valid but wrong `origin_at_load` echo mislabels the pair — medium, defer: pre-existing (Decision #2(b)), tracked by the open D1 task and `deferred-work.md:3693`; no new ledger entry.
- B3c verbatim `self` through `from_stored` untested end to end — low, rejected: the projection case pins `from_stored("self")`.
- B4a duplicated `BILINGUAL` constant, `"self"` literals, split `use` — low, patched.
- B4b positional arguments to `confirm_bilingual_import` break silently — false: a signature change fails to compile.
- B4c 60-row case relies on unordered `tm_rows` — false: `tm_rows` uses `ORDER BY id`.
- B5a no typed-then-reverted or trim/NFC case — false: `segment_contract` has `re_signing_after_an_edit_and_an_undo…` and `text_that_differs_only_by_unicode_normalization…`.
- B5b index panic before length check, temp dirs leak on panic — low, rejected: cosmetic, file-wide pattern.
- E2 whitespace/NFC-only edits count as verbatim — false as a defect: Decision #105 (Story 11.5) chose that comparison.
- E3 stored `''` with unchanged text gives `self` — false: documented sentinel rule (`segment.rs` doc above `confirm_segment`): text with no origin came from the typing buffer.
- E6 60 rows in one chapter, not 200 chapters — low, rejected: AC5 is about not skipping; noted in Implementation Notes.
- E7 Intent reads as if the reload gap were closed — false: D1 states it waits for the AD and its task is open.

## Verification

**Commands:**
- `cd src-tauri && cargo test --test tm_contract` -- expected: all green.
- `npm run test:story 7-2 -- --list` -- expected: lists the touched targets; run them.
