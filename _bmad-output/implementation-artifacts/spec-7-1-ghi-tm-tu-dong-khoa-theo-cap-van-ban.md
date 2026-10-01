---
title: 'Story 7.1 — Auto-write TM keyed by text pair'
type: 'feature'
created: '2026-10-01'
status: 'done'
route: 'dispatch'
baseline_commit: 'cdde5ccba6848adb669084f163842e2537cf5dc1'
review_loop_iteration: 0
context:
  - '{project-root}/AGENTS.md'
  - '{project-root}/src-tauri/AGENTS.md'
  - '{project-root}/_bmad-output/implementation-artifacts/epic-7-context.md'
---

<frozen-after-approval reason="human-owned intent — do not modify unless human renegotiates">

## Intent

**Problem:** Confirming a segment writes nothing to a Translation Memory: `core/tm/` holds only `SimilarSegment`, `project.db` has no TM table, and the `Ok(true)` branch of `confirm_segment` carries an "owner: Epic 7" placeholder. FR56 (and the TM half of FR44/FR129) is unmet. Two paths also break the pair's inputs: promoting an AI suggestion onto a confirmed segment keeps it confirmed with unsigned text (so that text never reaches TM), and segment merge/split silently drops `role`.

**Approach:** Add a Work-tier `tm_unit` table (project migration 27) holding `(source_text, target_text, translation_origin, created_at)` with no reference to `segment.id`, and insert one row inside the same `store.write` transaction as the draft→confirmed transition in `confirm_segment`, through a function owned by `core/tm/`.

**Decisions (Ice, 2026-10-01):**
- D1 `tm_unit.translation_origin` is stored now, the value `confirm_segment` writes to the segment in the same transaction. Story 7.2 only adds reading/projection.
- D2 Every transition appends a row, even when an identical (source, target, origin) row exists. Collapsing duplicates is a read concern (7.8/7.9).
- D3 No backfill: segments confirmed before migration 27 write no pair. TM starts empty.
- D4 `promote_ai_translation` that writes new text also sets `status='draft'`; the outcome carries the status and the webview mirrors it.
- D5 Merge/split is refused when any segment involved has a non-NULL `role`, with a new error key; nothing is written. Closes debt item ④ of spec 6.13.

## Boundaries & Constraints

**Always:**
- The pair is written exactly at the draft→confirmed transition (AD-31) and nowhere else. The already-confirmed no-op (`Ok(false)`) writes no pair.
- Existing `tm_unit` rows are never updated or deleted by this story (AD-6).
- Confirm writes only `project.db`, never `global.db` (AD-18).
- `alt`/`caption` segments write pairs through the same path as prose (AD-42, FR44, FR129).
- `core/tm` never names `core::ai` (AD-13).
- Use the origin naming convention: no bare `origin` identifier.

**Never:**
- No column, foreign key or index tying `tm_unit` to `segment.id`, `chapter_id` or `ord`.
- No UI, matching, read API, global-tier write or RAG change (Stories 7.2–7.11).
- No change to `restore_segment_version`, chapter merge/split, or import paths.
- No role-carrying logic in `write_regroup` (D5 refuses instead).

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected Output / Behavior | Error Handling |
|----------|--------------|---------------------------|----------------|
| First confirm | draft, source `A`, target `B` | one row (`A`,`B`, origin as written to segment) | N/A |
| Confirm again, no edit | already confirmed | no row, no version | N/A |
| Edit then re-confirm | confirmed `B` → `C` → confirm | rows (`A`,`B`), (`A`,`C`); first unchanged | N/A |
| Same pair twice | `B` → `C` → `B`, each confirmed | three rows, two of them (`A`,`B`) (D2) | N/A |
| Empty / whitespace target | `''` or `'  '` | no row | `segment.nothing_to_confirm`, as today |
| Retired / missing segment | — | no row | `segment.retired` / `segment.not_found`, as today |
| Prose merge/split after confirm | confirmed, then `merge_segments`/`split_segment` | earlier rows byte-identical | N/A |
| Role segment confirm | `role='alt'` or `'caption'` | one row, like prose | N/A |
| Role segment merge/split | either side of a merge, or the split segment, has a role | nothing written, no row retired | new `err.segment.*` key, `retryable=false` (D5) |
| Promote onto confirmed | confirmed, `promote_ai_translation` writes | `status='draft'`, origin `other`; next confirm writes a pair | N/A |
| Promote held back | `needs_confirmation=true` | nothing written, status unchanged | N/A |
| Insert fails | `tm_unit` insert errors | whole confirm rolls back | existing `IpcError` from `store.write` |

</frozen-after-approval>

## Code Map

- `src-tauri/src/commands/segment.rs`:
  - `confirm_segment` (~:2418) runs as one `open.store.write` transaction.
  - Its SELECT (~:2451) does not read `source_text` yet.
  - It returns `Ok(false)` early (~:2492).
  - It writes `UPDATE … status, translation_origin` (~:2546) and `INSERT segment_version` (~:2550).
  - The TM placeholder doc comment is at ~:2359-2367.
  - `promote_ai_translation` (~:2140) has its UPDATE after the unsigned-draft guard. `PromoteAiTranslationOutcome` is at ~:2102.
  - `RegroupReject` (~:2990) has `into_ipc`, `segment_has_no_previous` is the model for the new key, and `merge_segments` (~:3356) and `split_segment` (~:3462) call `write_regroup` (~:3146).
- `src-tauri/src/core/tm/mod.rs`: keep `SimilarSegment` and add the pair insert on a borrowed `&Transaction`.
- `src-tauri/src/core/store/schema.rs`: `PROJECT_MIGRATIONS` (~:2027) ends at step 26. Step 27 needs a doc-header "target" line, and the id uses `AUTOINCREMENT` per AD-3 (~:147).
- `src-tauri/src/core/i18n/mod.rs` (~:318 `SegmentNoPrevious`) and `src/i18n/vi.json` (~:54): add the new role-refusal key.
- Webview:
  - `src/config/segment.ts`: `promoteAiTranslation` (~:1119) and its outcome shape check.
  - `src/panels/editorPanelState.ts`: `promoteAiTranslationToEditor` (~:326) calls `replaceEditorSegment` (~:268), which already accepts `status`.
- Ladder pins that move to 27 (and the fake future step to 28):
  - `tests/segment_contract.rs`: ~:742, ~:876, ~:2115.
  - `tests/pinned_contract.rs`: ~:258.
  - `tests/chapter_origin_contract.rs`: ~:891.
  - `tests/segment_role_contract.rs`: ~:598.
  - `tests/project_contract.rs`: `NON_ENTITY_DETAIL_TABLES` (~:1744).
- Test setup to reuse:
  - `create_work_from_text`, `read_all_segment_rows`, `save_segment_targets`, `confirm_segment(Some(&opened), id, "", "")`, as in `segment_contract.rs` ~:3172.
  - Wire harness `Harness::with_open_work` in `tests/segment_wire.rs` ~:31-77.

## Tasks & Acceptance

**Execution (phases: Rust · Webview · Tests):**
- [x] `src-tauri/src/core/store/schema.rs`: add step 27 `tm_unit` (`target_text` CHECK non-empty). This is the Work-tier pair table.
- [x] `src-tauri/src/core/tm/mod.rs`: add the pair insert, so that `tm/` owns its table (AD-13 `tm → store`).
- [x] `src-tauri/src/commands/segment.rs` (FR56 at the AD-31 point, then D4 and D5):
  - Read `source_text` in the confirm SELECT.
  - Call the tm insert on the transition branch only, and replace the placeholder doc lines.
  - Make promote set `status='draft'` and return `status`.
  - Add the `RegroupReject` role variant, checked before any write.
- [x] i18n `mod.rs` and `vi.json`: add the role-refusal message key.
- [x] `src/config/segment.ts` and `src/panels/editorPanelState.ts`: accept and mirror `status` from the promote outcome (D4 on screen). Add the vitest case beside the existing promote tests.
- [x] Ladder tests listed in the Code Map: move the pins and add `tm_unit` to `NON_ENTITY_DETAIL_TABLES`.
- [x] `src-tauri/tests/tm_contract.rs` (new):
  - One case per I/O Matrix row, including a forced-failure rollback.
  - One wire case through `wire::confirm_segment`.
- [x] `deferred-work.md`:
  - Close spec 6.13 ④ (D5) and ⑤.
  - Close the "AC5 … đóng bằng CẤU TRÚC … Chủ: Epic 7" item.

**Acceptance Criteria:**
- Given confirmed pairs, when their segment row is retired by any path, then `tm_unit` row count and content are unchanged.
- Given `tm_contract.rs`, when the tm insert call is removed from `confirm_segment`, then the first-confirm, re-confirm and role-confirm cases go red and the no-op case stays green.
- Given the promote and role-refusal cases, when the D4 status write or the D5 check is removed, then each one's own case goes red.

## Implementation Notes

- `insert_pair` takes `core::store::{Transaction, SqlResult}`, not `rusqlite` types: `store_boundary` forbids naming `rusqlite` outside `core::store`.
- `wire::confirm_segment` became generic over `tauri::Runtime` so `tm_contract` can drive it on `MockRuntime`; `ipc_contract` pins the new signature.
- A confirmed segment returns to draft only through `flush_segment_targets` (`unconfirm_edited_segments`), not `save_segment_targets`; the edit/re-confirm cases use the flush.
- Two `segment_role_contract` tests that expected a role-less row after merge/split were wrong under D5 and became refusal tests.
- Counter-check (real removal of the `insert_pair` call): 8/14 `tm_contract` cases red, the no-op case green; D4 removal reds only the promote case; D5 removal reds the three refusal cases.
- Full `cargo test` after review patches: 1966 passed, 0 failed, 24 ignored, 74 targets (local Mac).

## Spec Change Log

## Review Triage Log

Pass 1 (blind-hunter, edge-case-hunter, verification-gap; verification-gap filed no gap):
- E1 promote of identical text demotes a confirmed segment — low, rejected: needs an AI suggestion byte-equal to the confirmed text; fix is a new branch.
- E2/B7 strict `status` check in `isPromoteAiTranslationOutcome` — false: Rust writes only `SEGMENT_STATUS_DRAFT`/`_CONFIRMED`.
- E3/E10 other `write_regroup` callers bypass D5 — false: only `merge_segments` and `split_segment` call it.
- E4 merge with two role rows names one id — low, rejected: cosmetic.
- E5 empty `source_text` stored — false: import creates no empty segment and split refuses an empty piece.
- E6 `''` origin reaches `tm_unit` — false: an empty loaded origin takes the `self` branch.
- E7 role-refusal case lacks confirmed pairs — low, rejected: refusal rolls back before any write; prose case covers pair identity.
- E8 wire case uses empty `text_at_load` only — low, rejected: shell passes arguments through; contract cases cover the origin branches.
- E9 guards not independent — false: D4 removal reds only the promote case (phase measurement).
- B1 bare `origin` local passed to `insert_pair` — low, patched: renamed `confirmed_origin`.
- B2 story ids in new test comments — low, patched in `segment_role_contract.rs`, `project_contract.rs`; ladder-step anchors kept per file convention.
- B3 no CHECK for the origin set or whitespace target — low, rejected: `confirm_segment` is the only writer and validates both.
- B4 other `confirmed` writers unaudited — false: only `confirm_segment` writes `'confirmed'` (grep, verification-gap).
- B5 role segments can never be merged or split — false as a defect: D5 is Ice's decision.
- B6 role as upper side of merge untested — false: `merge_segments(ids[2])` has the role row as `tren`.
- B8 spec line numbers and unmeasured ledger claims — rejected (spec edit); the `insert_pair` removal was re-measured by the orchestrator: 8/14 red.
- B9 stale example list in the migration doc — false: it illustrates a gapped increasing list, not a count.
- B10 no `source_text` index and no backfill notice — rejected: reads arrive in 7.4+, and D3 is Ice's decision.

## Verification

**Commands:**
- `npm run build`: `dist/` must exist before the cargo tests.
- In `src-tauri/`: `cargo test --test tm_contract --test segment_contract --test segment_wire --test pinned_contract --test chapter_origin_contract --test segment_role_contract --test project_contract --test ai_boundary`. Expected: green.
- The vitest file(s) touched for promote. Expected: green.
- Full `cargo test` once, by hand: a migration is shared wiring (AGENTS.md).
