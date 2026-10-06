---
ticket: 2
title: 'Story 7.2 — Origin on each TM pair'
type: 'feature'
created: '2026-10-01'
status: done
route: 'dispatch'
baseline_revision: '266c155f5ab91c00337cd065b50e81a668efee13'
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
- D1 The baseline lives in Rust, set by every AD-47 ③ write, and `confirm_segment` compares against it instead of trusting the webview's load snapshot. This amends AD-47 ③ (chapter-load row) and Decision #2(b), so it needs a new AD drafted by Winston first (brief: `planning-artifacts/ad-brief-2026-10-01-moc-so-xuat-xu-luu-phia-rust.md`). Implementation of D1 waits for that AD; its shape (columns, migration 28, IPC) comes from the AD, not this spec. 🔵 2026-10-01: the AD is AD-50; D1 is implemented to it.

## Boundaries & Constraints

**Always:**
- The value set stays exactly three plus `''` (AD-47 ⑥); `''` never reaches `tm_unit`.
- `self` projects to mine; `other` and `bilingual_import` project to others. The projection is an exhaustive `match`, so a new value fails to compile until it declares its side.
- Zero added user actions (FR117).
- Pairs are written only at the draft→confirmed transition (AD-31); existing rows are never updated (AD-6).
- `core/tm` never names `core::ai` or `crate::commands` (AD-13).

**Never:**
- No read API, sort, scope union, matching, UI, or RAG change (Stories 7.3–7.11).
- No change to the AD-47 ④ agreement/disagreement rule. 🔵 2026-10-01 (Ice): its input changes to the AD-50 arbitration of each source segment, for merge and split; restore sets the baseline per AD-50 rule 2.
- No bare `origin` identifier.

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected Output / Behavior | Error Handling |
|----------|--------------|---------------------------|----------------|
| Typed then confirmed | draft, origin `''`, user types `B` | pair (`A`,`B`,`self`) | N/A |
| Bilingual verbatim | imported `B`, origin `bilingual_import`, confirm unchanged | pair origin `bilingual_import` | N/A |
| Bilingual rewritten | imported `B`, user types `C`, confirm | pair origin `self` | N/A |
| Other verbatim / rewritten | AI-promoted `B` (`other`); confirm `B` / `C` | `other` / `self` | N/A |
| Edited Work, many chapters | bilingual Work, some sentences verbatim, some rewritten, all confirmed | one pair per confirm, none skipped, each with its own origin | N/A |
| Out-of-set baseline origin | text unchanged, `baseline_translation_origin` outside the FR117 set | nothing written | existing `ConfirmReject::UnknownOrigin`, transaction rolls back |
| Rewrite survives reload (D1) | bilingual `B`, user types `C`, flush, chapter reopened or app restarted, confirm `C` | pair origin `self` | N/A |
| Rewritten draft merged (D1) | two bilingual segments, user rewrites both without confirming, merge, confirm unchanged | pair origin `self` | N/A |
| Restore sets others' baseline (D1) | baseline origin `self`, an older version holds AI text `A`, restore it, confirm unchanged | pair origin `other` | N/A |
| Migrated row (D1) | pre-step-28 Work, row typed and flushed, origin `''`, opened after step 28, confirm unchanged | pair origin `self` | N/A |

</frozen-after-approval>

## Code Map

- `src-tauri/src/core/tm/mod.rs` -- `insert_pair` (takes `translation_origin: &str`); `SimilarSegment` stays untouched.
- `src-tauri/src/commands/segment.rs` -- origin constants `:2045-2088`; `confirm_segment` `:2437`, verdict `:2558`, `insert_pair` call `:2577`; `ConfirmReject::UnknownOrigin` reused for the stale-`''` case.
- `src-tauri/src/commands/segment.rs:185-211` -- `insert_bilingual_segments` (draft, `bilingual_import`); caller `commands/project/work_creation.rs:496`. Reuse it in tests via the bilingual Work creation path used by `tests/bilingual_import_contract.rs`.
- `src-tauri/tests/tm_contract.rs` -- helpers `work`, `type_text`, `tm_rows`; `promoting_onto_a_confirmed_segment…` (`:286`) is the `other` model.
- D1 tests that move: `segment_contract.rs`, `ipc_contract.rs`, `ipc_argument_contract.rs`, `segment_baseline_guard.rs`, `ai_translate_contract.rs`, `chapter_origin_contract.rs`, `pinned_contract.rs`, `segment_role_contract.rs`; `tests/frontend/editorConfirmSegment.test.ts`, `gridPanelRowErrorPriority.test.ts`; e2e `segment-history-restore`, `grid-row-error-label`, `segment-merge-split`.

## Tasks & Acceptance

**Execution:**
- [x] `src-tauri/src/core/tm/mod.rs` -- add a closed pair-origin type (three variants, `from_stored`, `as_str`, exhaustive mine/others projection); `insert_pair` takes it -- `''` becomes unrepresentable.
- [x] `src-tauri/src/commands/segment.rs` -- parse the verdict into that type before any write; a `''` verdict rejects with `UnknownOrigin`.
- [x] `src-tauri/tests/tm_contract.rs` -- one case per matrix row, plus a projection case pinning all three values.
- [x] `src-tauri/src/core/store/schema.rs` -- step 28 per AD-50 rule 6 (two columns + backfill, one batch); bump the step-count fixture in `segment_contract.rs`.
- [x] `src-tauri/src/commands/project/mod.rs` -- `reject_unknown_translation_origin` also covers `baseline_translation_origin` (AD-50 rule 1).
- [x] `src-tauri/src/core/segment/` -- the pure arbitration function (AD-50 rule 4); `regroup.rs` `merge`/`split_at` take each source's arbitrated origin and set both baseline columns on new rows.
- [x] `src-tauri/src/commands/segment.rs` -- the single non-user `UPDATE` path (AD-50 rule 3) used by `promote_ai_translation` and `restore_segment_version`; `insert_segments`, `insert_bilingual_segments`, `write_regroup` write both baseline columns; `confirm_segment` and its wire drop `text_at_load`/`origin_at_load` and call the arbitration.
- [x] `src/config/segment.ts`, `src/panels/editorPanelState.ts` -- `confirmSegment(id)` only; drop the load-snapshot baseline and its comments.
- [x] Tests -- `ipc_contract.rs` pins `confirm_segment` args to `[segment_id]` and no `baseline_*` key on the segment DTO; a code-line guard (`src-tauri/tests/segment_baseline_guard.rs`) reds on any SQL writing `target_text` without `baseline_target_text`, flush the one named exemption; `tm_contract.rs` one case per new matrix row; update the existing `confirm_segment` callers in Rust tests, vitest and e2e.
- [x] Ledger -- close "Rust TIN mốc do webview khai" (`deferred-work.md:3693`) when the D1 cases are green.

**Acceptance Criteria:**
- Given `tm_contract.rs`, when the rewrite-to-`self` comparison is removed from the verdict, then the "rewritten" cases go red and the "verbatim" cases stay green.
- Given the out-of-set baseline origin case, when the type check is bypassed, then that case goes red because a pair with an unknown origin appears.
- Given the D1 reload and merge cases, when the comparison against `baseline_target_text` is really removed from the arbitration, then both go red and every verbatim case stays green.
- Given the guard, when `insert_bilingual_segments` really stops writing `baseline_target_text`, then the guard goes red.

## Implementation Notes

- `PairOrigin` lives in `core::tm` and spells the three strings itself (`core` cannot name `crate::commands`); the projection case pins them against `TRANSLATION_ORIGIN_*`, so the two lists cannot drift silently.
- Counter-check (real removal of `target_nfc != text_at_load_nfc` from the verdict): 4/21 `tm_contract` cases red (three new rewrite cases plus `the_same_pair_confirmed_twice_over_a_detour…`), every verbatim case green.
- AC2 has no runtime counter-check: once `insert_pair` takes `PairOrigin`, bypassing the type does not compile. The stale-`''` case pins the reject path and the absence of any row.
- The edited-Work case uses 60 sentences in one bilingual Work; AC5 is about not skipping, not chapter count.
- D1 and its matrix row stay open until the new AD lands; the story cannot reach `done` before then. 🔵 2026-10-01: AD-50 landed; D1 now waits only on its tasks.
- D1 counter-check (real removal of the baseline comparison in `core/segment/origin.rs::arbitrate`): 5/26 `tm_contract` red (rewritten bilingual, AI rewrite, edited Work, reload, merge-of-rewritten) and 2 `segment_contract` red; every verbatim, restore and migration case green. Restore written without the baseline: 1 red. `insert_bilingual_segments` without `baseline_target_text`: guard red naming that line. Step-28 backfill `UPDATE` removed: 1/27 `tm_contract` red (the backfill case).
- AC2 is checked at the arbitration, not at `insert_pair`: mapping an unknown baseline origin to `other` makes the out-of-set case red because a pair appears where none may; an unknown pair origin stays unrepresentable.
- A bad `baseline_translation_origin` also refuses the Work at open (AD-50 rule 1), so the `segment_contract` out-of-set case now asserts the open refusal.
- The migrated-row case drops the two columns and rewinds to step 27 on a real Work, then reopens so step 28 actually runs.

## Spec Change Log

- 2026-10-01 -- AD-50 landed. Ice renegotiated two frozen items: the ④ Never now allows the input change, and the stale-`''` row/AC (unreachable once `origin_at_load` leaves the wire) became the out-of-set baseline origin row/AC. D1 expanded into tasks; three matrix rows added.

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

Pass 2, D1 (blind-hunter, edge-case-hunter, verification-gap):
- V1/B4b step-28 backfill `UPDATE` unpinned: the migration case types a row with origin `''`, so with or without the `UPDATE` it yields `self` — medium, patch: migrated non-`self` draft and retired row, assert both baseline columns.
- V2/B6 merge/split `RegroupReject::UnknownOrigin` path has no case — low, patch: one merge case with an out-of-set baseline, assert the code and no rows retired or created.
- E2/B5a guard accepts a write of `baseline_target_text` without `baseline_translation_origin` — medium, patch: require both columns; the writer floor rises to the three known writers.
- E3/B5b guard misses `format!`-built or >12-line SQL — low, rejected: no such writer exists, and detecting built SQL adds parsing complexity.
- E7 spec says the guard reds on "any" SQL — low, rejected: the fix edits this spec.
- E1/B4a step 28 cannot tell a pre-28 rewrite from imported text — false as a defect: AD-50 rule 6 records that loss; the original text was never stored.
- E4/B2 restore leaves `translation_origin` while the baseline origin becomes `other` — false: AD-47 ⑤ and AD-50 rule 2 keep the stored origin on restore.
- B3 restore labels the user's own older text `other` with no ledger item — false: AD-50 rule 2 records the cheap direction Ice chose; nothing left unaccepted.
- E5 confirm does not advance the baseline — false: AD-50 rule 2 (confirm and flush never touch the baseline) and Decision #96 require this.
- E6 empty target with an unknown baseline origin is copied into the new row — false: `arbitrate` returns `Unsigned` for empty text, so the new row takes the ④ result, never the bad value; open also refuses the Work.
- B1 Pass 1 rows cite deleted code — low, rejected: the fix edits this spec; Pass 2 supersedes them.
- B7 `write_non_user_target` arguments swap silently — false: `&str` and `Option<&str>` swapped do not compile.
- B8 changed doc comments mix Vietnamese and English (`NewSegment.translation_origin`, `ChapterSegment.translation_origin`) — low, patch: English only on those lines.
- B9 tests not rustfmt-formatted, positional tuple fields — low, rejected: the repo is not rustfmt-clean (`segment_contract.rs` 266 diffs) and has no fmt gate.
- B10 `pinned_contract` message says twenty-six steps — false: the diff updated message and value together.
- B11 open-time `UNION` scans `segment` twice — low, rejected: no measured cost; one extra scan on open.
- B12 `ChapterSegment.translation_origin` may be dead on the wire — false: `editorPanelState.ts:357` consumes it; AD-50 rule 5 keeps it for display.

## Verification

**Commands:**
- `cd src-tauri && cargo test --test tm_contract` -- expected: all green.
- `npm run test:story 7-2 -- --list` -- expected: lists the touched targets; run them.

## Acceptance criteria from epics.md

Source: `epics.md` §Story 7.2 (v6, nay ở archive-v6).

**Covers:** FR118

As a người dịch làm cả vai biên tập,
I want mỗi cặp trong kho biết nó là chữ của ai,
So that kho của tôi không âm thầm đầy lên bằng văn phong người khác.

**Acceptance Criteria:**

**Given** một cặp TM
**When** ghi
**Then** mang **xuất xứ kế thừa từ segment** sinh ra nó

**Given** ba giá trị xuất xứ
**When** mô hình hoá
**Then** *tôi dịch* · *người khác dịch* · *nhập từ tài liệu song ngữ*

**Given** người dùng biên tập một Tác phẩm do người khác dịch
**When** xác nhận từng câu
**Then** câu họ **viết lại** mang xuất xứ *tôi dịch*
**And** câu họ **duyệt nguyên văn** mang xuất xứ *người khác dịch*

**Given** việc phân biệt hai loại trên
**When** thực hiện
**Then** không tốn thêm một thao tác nào của người dùng

**Given** hệ thống gặp một Tác phẩm biên tập 200 chương
**When** ghi TM
**Then** **không bỏ không ghi** — làm vậy sẽ mất luôn những câu chính người dùng viết lại từ đầu
