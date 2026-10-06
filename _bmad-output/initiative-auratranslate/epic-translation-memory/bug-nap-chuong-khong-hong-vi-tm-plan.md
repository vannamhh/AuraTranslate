---
ticket: 12
title: 'Epic 7 retro R-2 — Chapter load never fails because of TM pre-fill'
type: 'bugfix'
created: '2026-10-05'
status: 'done'
route: 'dispatch' # oneshot | dispatch
baseline_revision: '310e806eb64445cd0d556c9938d0b4a8ccd8ba84'
review_loop_iteration: 0
context:
  - '{project-root}/src-tauri/AGENTS.md'
  - '{project-root}/src/AGENTS.md'
---

<frozen-after-approval reason="human-owned intent — do not modify unless human renegotiates">

## Intent

**Problem:** TM pre-fill (7-4) is a secondary feature that can break the primary path. If `global.db` is not managed, if any `tm_unit` row has an unknown origin, or if the fill fails in any other way, the whole Chapter load fails. The error is sometimes marked `retryable`, but retrying never succeeds (retro F3.1). Separately, the refresh that runs after a merge/split goes through the same load. If the user types into a segment while that refresh is in flight, the segment is re-marked as TM-filled and its snapshot origin/status is overwritten (F3.2).

**Approach:** Any failure inside the pre-fill degrades to "no pre-fill". The Chapter still loads, and the response says that pre-fill was skipped and why. The webview shows this as a non-fatal StatusBar notice. **Decision Q1 (Ice, 2026-10-05): option A.** The post-regroup refresh does not pre-fill: the load wire takes `prefill: bool` (the refresh passes `false`) and the TM adoption loop in `refreshChapterAssetsAfterRegroup` is removed. An empty segment created by merge/split stays empty until the Chapter is next opened, which reverses the regroup adoption added in 7-4. Ice also chose to keep the spec whole at 2226 tokens.

## Boundaries & Constraints

**Always:**
- Pre-fill stays all-or-nothing (one write transaction). A degraded load writes no TM text and returns `tm_filled_segment_ids` empty.
- The skip status is a closed wire type (Rust enum ⇄ TS union, checked by `isChapterSegments`). An unknown value is rejected and never treated as "ran". If no fill was attempted, the status says so; it never reads as "ran, 0 hits" (silent-emptiness rule).
- Each degraded load emits one `log`/`eprintln` line with the error code, following the precedent of the call sites.
- Counter-checks are real removals (AGENTS.md §Tests).

**Never:**
- Do not change the shared row mapping in `core/tm/mod.rs` (`query_pair_rows`, `pair_by_id`) or the callers of `tm_lookup_failed` in `tm.rs`/`aiprompt.rs`. Fuzzy, Concordance, RAG and TM management keep failing on a bad row; that becomes a debt item.
- No schema `CHECK` on `tm_unit.origin` and no migration.
- No retry button and no change to `editorLoadError`.
- Do not revisit 7-4 Q1 (a deliberately cleared segment is refilled when the Chapter is opened).

## I/O & Edge-Case Matrix

| Scenario | State | Expected |
|---|---|---|
| Ran | global managed, valid rows | as today; status `ran` |
| No global | `wired(.., false)` | segments returned, nothing written, status `skipped` + `store.open_failed`, notice shown |
| Bad origin row | one global or Work row with origin `x` | segments returned, nothing written, status `skipped` + `tm.lookup_failed`, notice shown |
| Fill write fails | write transaction errors | rolled back, segments returned, status `skipped` |
| Segment read fails | project read error | load fails as today (not a TM failure) |
| Typing during refresh | user types into segment S while the post-regroup refresh is in flight | no TM write happens; S is not added to the TM-filled set and its snapshot is not replaced |

</frozen-after-approval>

## Code Map

- `src-tauri/src/commands/segment.rs` -- `load_open_chapter_segments` (:1165) has two `?`s: `global.ok_or_else(global_store_missing)?` at :1170 and `fill_exact_tm_matches(..)?` at :1171. `fill_exact_tm_matches` (:1180) does a candidate read, then `pairs_for_source`, then one `Store::write`. `tm_lookup_failed` (:1153) hard-codes `retryable: true` and is shared; leave it alone. `ChapterSegments` (:306-335) gets the status field. Wire at :4072-4085 (`app.try_state::<Store>()`). `write_non_user_target` (:2219). `merge_segments`/`split_segment` stay unchanged.
- `src-tauri/src/lib.rs:1056-1100` -- `open_global_store` never calls `manage` on failure. That is how "global missing" happens in the real app.
- Wire precedents for the status shape: `TmInjectionStatusWire` (`commands/aiprompt.rs:219-232`) and `LibraryRescan.text_skipped` (`commands/library.rs:145`).
- `src-tauri/tests/tm_contract.rs` -- `wired(tag, text, manage_global)` (:1039), `seed`/`seed_global` accept any origin (:782, :1063-1083). `a_load_without_a_managed_global_store_fails_and_writes_nothing` (:1242) must be inverted. `ChapterSegments` is built literally in `tests/ipc_contract.rs:808, :830`.
- `src/config/segment.ts` -- type at :190-210, guard `isChapterSegments` at :239-252, `readOpenChapterSegments` at :681.
- `src/panels/editorPanelState.ts` -- `ensureSegmentsLoaded` (:198, consumer :215) and `refreshChapterAssetsAfterRegroup` (:2677-2698, fired with `void` at :2792 after the flush at :2761). It replaces snapshot rows (:2688-2696) without checking `editedText` (:543) or `flush.pending()`. Also `tmFilledSegmentIds`/`dropTmFilled` (:143-148), notices via `datThongBao` (:2506) and `ghiRegroupNotice` (:2563).
- `src/StatusBar.vue:202-213` -- closed notice→i18n key tables; a new notice needs a key in `src/i18n/vi.json` (and any other locale file).
- Tests that move: `tm_contract.rs`, `ipc_contract.rs`, plus every test that calls the load wire if the new argument is not defaulted; `tests/frontend/editorRegroupAssetRefresh.test.ts` (hand-rolled deferreds `treoTuLuotThu`/`luotDangTreo`, and `dienSanLanHai`), `editorTmFilled.test.ts`, `configSegmentChapterSegmentsShape.test.ts`.

## Tasks & Acceptance

**Execution:**
- [x] `src-tauri/src/commands/segment.rs` -- add the closed status type to `ChapterSegments`; make `load_open_chapter_segments` turn a missing global or any `fill_exact_tm_matches` error into `skipped{code}` and still read the segments; add `prefill: bool` to the wire and skip `fill_exact_tm_matches` when it is false (status `not_asked`, not `ran`).
- [x] `src-tauri/tests/tm_contract.rs` (+ the literal at `ipc_contract.rs`) -- one wire case per matrix row, invert :1242.
- [x] `src/config/segment.ts`, `src/panels/editorPanelState.ts`, `src/StatusBar.vue`, `src/i18n/vi.json` -- mirror the type and guard, show the notice on a skipped load, pass `prefill: false` from `refreshChapterAssetsAfterRegroup` and remove its TM adoption loop.
- [x] `tests/frontend/*` -- guard-shape case, notice case, and a "type while refresh in flight" case using the existing deferreds.
- [x] `_bmad-output/implementation-artifacts/deferred-work.md` -- one item (≤5 lines): a bad-origin `tm_unit` row still fails Fuzzy/Concordance/RAG/TM management and TMX export (`core/tm/mod.rs:188-194`, `:426-428`). Chủ: Amelia.

**Acceptance Criteria:**
- Given the No-global and Bad-origin cases, when the degrade branch is really removed (the `?` restored), then both go red, and the Ran case stays green.
- Given the type-while-refresh vitest, when the refresh is really put back to `prefill: true` with the adoption loop restored, then it goes red because S is in the TM-filled set. The inverted case at `editorRegroupAssetRefresh.test.ts:183` asserts that no id is adopted.
- Given a `skipped` load, when the notice mapping is really removed, then the notice vitest goes red.

## Verification

**Commands:**
- `npm run build && cargo test --test tm_contract --test ipc_contract` (from `src-tauri/`) -- expected: green.
- `npx vitest run tests/frontend/editorRegroupAssetRefresh.test.ts tests/frontend/editorTmFilled.test.ts tests/frontend/configSegmentChapterSegmentsShape.test.ts` -- expected: green.
- `npm run check:i18n` -- expected: green.

## Implementation Notes

- `TmPrefillStatus` (`ran` | `not_asked` | `skipped{code}`, serde-tagged by `kind`) lives on `ChapterSegments`. The pure `read_open_chapter_segments` reports `not_asked`; only `load_open_chapter_segments(global, open, prefill)` reports `ran` or `skipped`.
- The IPC wire takes `prefill: Option<bool>` and treats a missing value as `true`, not a bare `bool`, so existing callers that invoke with `{}` (e2e specs) keep the 7-4 behaviour. `readOpenChapterSegments` always sends the flag explicitly.
- The notice is the transient `NavNotice` value `'tm-prefill-skipped'` (key `panel.grid.nav_tm_prefill_skipped`), so the next action clears it. It is shown again on every load while the cause persists.
- The fill-write-failure case uses a `BEFORE UPDATE OF target_text` trigger that aborts the fill transaction.
- Counter-checks (code really removed or restored): with the `?` restored, the no-global, bad-global-row, bad-Work-row and fill-write cases go red and Ran stays green. With `prefill: true` and the adoption loop restored, both refresh vitests go red. With the StatusBar key mapping removed, the notice vitest goes red.
- The matrix row "segment read fails" is guarded by `a_failing_segment_read_still_fails_the_load_whether_the_prefill_ran_or_was_skipped`, added at the task check. It guards an unchanged path, so it has no counter-check.
- Not checked in the real app: the StatusBar notice, and the real `global.db` open failure in `lib.rs:1056-1100`. This is left for Ice's real-use pass under R-11.

## Spec Change Log

## Review Triage Log

- B1/E5 regrouped empties stay empty, no debt item — false: this is Ice's Q1-A decision, recorded in the frozen Intent; it is a decided behaviour, not a missing capability.
- B2a snapshot-replace pattern unguarded "in other paths" — false: no path is named; the only replace on the refresh path was removed.
- B2b/E8 type-while-refresh vitest guards the loop, not the flag — false: the flag is pinned separately (`tuyChonCacLuotGoi[1]` equals `{prefill:false}`), so both seams are guarded.
- B3/E6 `prefill: Option<bool>` default covered only implicitly — low, rejected: `w.load()` passes `None` and asserts `Ran`; `Some(true)` is a plain `unwrap_or`. The decision is in Implementation Notes.
- B4 Rust pins only the `not_asked` JSON shape — low, patch: a serde rename on `ran`/`skipped{code}` would make `isTmPrefillStatus` reject every load, with no Rust red.
- B5 guard accepts extra fields or an empty `code` — low, rejected: no harm; `kind` is the closed axis.
- B6 the notice does not name the cause; "Văn bản đang có không bị đổi" is untrue — false on the text: nothing is written on a skipped load; naming the cause is a design choice, and the code goes to the console.
- B7 no test for two consecutive skipped loads — low, rejected: same code path each load.
- B8 the `eprintln!` text is ASCII Vietnamese with no ids — false: it matches the 25 `eprintln!` precedents in `commands/` (e.g. `segment.rs` "tm lookup that bai").
- B9 the read-fails case is `cfg(unix)`; the trigger is not asserted as the cause; the combined test — low, rejected: same pattern as `segment_contract.rs`; without the trigger the same setup yields `Ran` (sibling case).
- B10 other `ChapterSegments` constructors and e2e not audited — false: the verification-gap layer searched the repo; no other constructor, and no e2e file reads the field.
- B11 spec bookkeeping and debt owner/line refs — false or low, rejected: the logs fill at triage; `Chủ: Amelia` is a valid persona owner.
- E1/E7 `splitChapterHere` clears the nav notice with `datThongBao({splitChapter:'split'})` after the reload — low, rejected: real but rare. Carrying `nav` over would hide the split confirmation (nav outranks split in `StatusBar.vue` `v-else-if`), and the skip notice reappears on the next load.
- E2 `datThongBao({nav})` in `ensureSegmentsLoaded` clears the other slots — low, rejected: every load path runs after `resetEditorPanel()` has cleared them, and the single-owner rule is intended.
- E3 `not_asked` not handled in `ensureSegmentsLoaded` — false: that path always sends `prefill: true`; Rust returns `not_asked` only for `false`.
- E4 fill commits, then the segment read fails — low, rejected and pre-existing: the write is a valid AD-50 non-user write; the read failure (non-UTF-8 dir) blocks every load anyway.
- V1 the `eprintln!` line is not asserted — low, rejected: a log line only.
