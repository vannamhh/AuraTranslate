---
ticket: 4
title: 'Story 7.4 — Exact 100% TM match pre-fills an unconfirmed segment'
type: 'feature'
created: '2026-10-02'
status: done
route: 'dispatch'
baseline_revision: 'c751d1b898f07b5d72aa39f028410a31024892c3'
review_loop_iteration: 0
context:
  - '{project-root}/AGENTS.md'
  - '{project-root}/src-tauri/AGENTS.md'
  - '{project-root}/src/AGENTS.md'
  - '{project-root}/_bmad-output/implementation-artifacts/epic-7-context.md'
---

<frozen-after-approval reason="human-owned intent — do not modify unless human renegotiates">

## Intent

**Problem:** TM pairs are written (7.1–7.3) but never read back: a sentence identical to one already translated opens empty. The margin-bar branch `isTmFilled` is hard-coded `false`, and `tm_unit.source_text` has no index, so a per-segment lookup scans both tiers (measured: 100,000 pairs, 1,000 lookups, about 7.1 s without an index and 3.7 ms with one, Python `sqlite3` 3.53.4 in memory).

**Approach:** When a Chapter is loaded, for each eligible segment write the target of the first pair returned by `core::tm::pairs_for_source` through the single AD-50 writer `write_non_user_target`. The write is immediate, sets status `draft` and both origin columns to the pair's origin, and creates no `SegmentVersion`. The load result names the segments it filled; the webview shows `tm-rule` with the "needs confirmation" label for those, in this session only. Confirming without an edit keeps the pair's origin through the existing `arbitrate`.

## Boundaries & Constraints

**Decisions (Ice, 2026-10-02):**
- Q1 = fill on Chapter load: `read_open_chapter_segments` fills every eligible segment of the Chapter in one transaction before it returns. A repeat of a sentence confirmed later in the same Chapter fills on the next load.
- Q2 = session-only marker: `ChapterSegments` carries the ids filled by THIS load; the webview keeps them until the segment is edited, confirmed, or the Chapter/app is left. A filled, unconfirmed segment shows the plain `draft` bar after a Chapter switch or restart. No `is_tm_filled` derivation, no change to `origin.rs`.
- Spec kept whole at 2,138 tokens (tiktoken o200k).
- Review E2 = refill always: an empty draft is eligible on every load, including one the user emptied after a fill; the suggestion returns, unconfirmed and marked.

**Always:**
- Eligible: `status = 'draft'`, `target_text` empty after `trim`, not retired, not `is_omitted`. Text the user or another writer put there is never overwritten. Re-check eligibility inside the write transaction.
- Exact equality on the stored `source_text`, with no normalization. Both sides come from `segment.source_text`; TMX import (7.10) normalizes on its own insert.
- One index on `tm_unit(source_text)`: project step 29 and global step 12, forward-only (AD-25).
- Pair choice follows `pairs_for_source` order (AD-18): mine, then Work, then id.
- No comparison of `target_text` with the baseline outside `core/segment/origin.rs` (AD-50 rule 4).
- If the global `Store` is missing at the wire, that is an error, never a silent Work-only lookup.

**Never:**
- No fuzzy match, normalization layer, Concordance, multi-translation picker or management UI (7.5–7.9).
- No new `segment` column, no new colour token, no `baseline_*` key on the wire (AD-50 rule 5).
- No change to flush (AD-35) or to `confirm_segment` arbitration.

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected Output / Behavior | Error Handling |
|----------|--------------|---------------------------|----------------|
| Hit | empty draft, Work pair `self` for `S` | target = pair target, `draft`, origin `self`, baseline = (text, `self`), 0 versions | N/A |
| Origin beats tier | Work `other`, Global `self` | Global `self` text is filled | N/A |
| No pair | no row for `S` | segment unchanged | N/A |
| Has text | draft with typed text | unchanged | N/A |
| Confirmed / omitted / retired | any | unchanged | N/A |
| Confirm unedited | filled from `bilingual_import` pair | TM pair and segment origin `bilingual_import` | N/A |
| Confirm edited | filled, then one character changed | origin `self` | N/A |
| Marker lifetime | filled this load; then edited / confirmed / Chapter reloaded | `tm-rule` → `draft` / `confirmed` / `draft` | N/A |
| Global store missing | wire without a managed global `Store` | load fails, nothing written | error, not a Work-only lookup |
| Fresh and upgraded stores | project at 28, global at 11 | index exists, versions 29 and 12 | N/A |

</frozen-after-approval>

## Code Map

- `src-tauri/src/core/tm/mod.rs` -- `pairs_for_source(resolver, global, work, source)` (~:170), `TmPair`, `PairOrigin::as_str`. Reuse; do not add normalization.
- `src-tauri/src/commands/segment.rs` -- `write_non_user_target` (:2119, sole AD-50 writer), model caller `promote_ai_translation` (:2161, wire :3883). `read_open_chapter_segments` (:1063, wire :3590) and `ChapterSegment` (:284). Constants for status and origin at :2029-2082. The global `Store` is reached via `app.try_state::<Store>()` (precedent `commands/aitranslate.rs:818`); the Work resolver is `OpenWork.scope`.
- `src-tauri/src/core/segment/origin.rs` -- `arbitrate`; unchanged by this story, read-only reference for the confirm cases.
- `src-tauri/src/core/store/schema.rs` -- `TM_UNIT_DDL` (:888), `PROJECT_MIGRATIONS` (:2049, last `to_version` 28), `GLOBAL_MIGRATIONS` (:756, last 11). The step-count fixtures are in `tests/store_contract.rs`, `pinned_contract.rs` and `glossary_contract.rs` (see 7.3 notes).
- `src-tauri/tests/tm_contract.rs` -- helpers `work`, `bilingual_work`, `type_text`, `tm_rows`; wire test with `mock_builder` + `OpenWorkState`. Global seeding is direct SQL. Other load-wire users must manage a global `Store`: `segment_wire.rs`, `segment_contract.rs`, `ai_translate_*`, `segment_role_contract.rs`, `segment_image_contract.rs`, `ai_prompt_contract.rs`, `ipc_contract.rs`.
- Tests that move: `src-tauri/tests/tm_contract.rs`, `src-tauri/tests/ipc_contract.rs`, `src-tauri/tests/pinned_contract.rs`, `src-tauri/tests/store_contract.rs`, `src-tauri/tests/glossary_contract.rs`, `src-tauri/tests/segment_contract.rs`, `src-tauri/tests/segment_role_contract.rs`, `src-tauri/tests/chapter_origin_contract.rs`; `tests/frontend/aiTranslateBatchResetWiring.test.ts`, `tests/frontend/chapterPosition.test.ts`, `tests/frontend/configSegmentChapterSegmentsShape.test.ts`, `tests/frontend/editorRegroupAssetRefresh.test.ts`, `tests/frontend/editorRegroupGuards.test.ts`, `tests/frontend/editorSegmentRule.test.ts`, `tests/frontend/editorTmFilled.test.ts`, `tests/frontend/glossaryMarksRefresh.test.ts`, `tests/frontend/gridPanelTmFilled.test.ts`, `tests/frontend/gridPanelHanVietTabFocus.test.ts`, `tests/frontend/gridPanelImages.test.ts`, `tests/frontend/importPreviewDestination.test.ts`, `tests/frontend/libraryChapters.test.ts`, `tests/frontend/libraryImportBlocksResubmitWhilePreviewOpen.test.ts`, `tests/frontend/libraryImportDropMultiple.test.ts`, `tests/frontend/libraryImportRetryAfterFailedConfirm.test.ts`, `tests/frontend/librarySearch.test.ts`, `tests/frontend/segmentSelection.test.ts`.
- `src/config/segment.ts` -- `ChapterSegment` (:76-149) with its runtime guard (~:233), IPC adapters.
- `src/panels/editorSegments.ts` -- `segmentRuleInputOf` (:206) hard-codes `isTmFilled: false`; wire it here. `resolveSegmentRule` and the 5 `SEGMENT_RULE_VALUES` stay unchanged (`check:commands` Kiểm I).
- `src/panels/editorPanelState.ts` -- `editorSegments`, `replaceEditorSegment` (:268), `flushEditorBeforeDiscreteWrite` (:715).
- `src/panels/GridPanel.vue` -- `.rule-tm-rule` (:2152) and state label `panel.grid.state_tm` (:330, `vi.json:696`) already exist; reword the label to say it needs confirmation (UX-DR47 voice).

## Tasks & Acceptance

**Execution:**
- [x] `src-tauri/src/core/store/schema.rs` -- index step 29 (project) and 12 (global); move the fixtures.
- [x] `src-tauri/src/commands/segment.rs` -- fill inside `read_open_chapter_segments` (wire gets the global `Store`), through `write_non_user_target` only; add the filled ids to `ChapterSegments`.
- [x] `src-tauri/tests/tm_contract.rs` -- one case per matrix row, wire level, with both stores managed.
- [x] `src/config/segment.ts` (type + runtime guard), `src/panels/editorPanelState.ts` (session set of filled ids, dropped on edit/confirm/reload), `src/panels/editorSegments.ts` (`isTmFilled` from that set), `src/i18n/vi.json` (label) -- Vitest in `tests/frontend/editorSegmentRule.test.ts` plus the state module's test.

**Acceptance Criteria:**
- Given the Hit case, when the `write_non_user_target` call is really removed, then Hit and Confirm-unedited go red and No-pair stays green.
- Given the Has-text case, when the eligibility re-check is really removed, then that case goes red.
- Given a filled segment, when the webview renders it, then the bar is `tm-rule` and the state label says it needs confirmation. With `isTmFilled` really back to `false`, the vitest goes red.
- Given the three ledger items owned by Story 7.4 (`grep -n 'Story 7.4' deferred-work.md`: the `isTmFilled` margin-bar branch, the FR58 row of the AD-47 ③ table, the missing `source_text` index), when this story closes, then each gets a `✅ ĐÃ ĐÓNG` line naming its guard.

## Implementation Notes

- The fill lives in `load_open_chapter_segments(global, open)`, called by the `read_open_chapter_segments` wire (now generic over the runtime for mock tests). The pure `read_open_chapter_segments(open)` keeps its signature and does not fill, so its ~81 test call sites and the AI callers (`aitranslate.rs`, `aiprompt.rs`) are unchanged.
- The candidate read filters `status = 'draft'` but not empty target; the in-transaction re-check is the only guard for the Has-text row. That keeps the guard able to fail, at the cost of one indexed lookup per draft with text.
- The webview's second load caller, `refreshChapterAssetsAfterRegroup`, also merges the ids it fills, because merge/split creates empty segments that the next load fills.
- The fill writes `target_text` without `Indexer::rebuild`, like flush, confirm and `promote_ai_translation`. Library search can lag behind pre-filled text until the next reindex. Not decided here; see review.
- Counter-checks (code really removed): without the `write_non_user_target` call, 4 `tm_contract` cases go red (Hit, Origin-beats-tier, Confirm-unedited, Marker lifetime) and No-pair stays green, re-run by the orchestrator. Re-check forced true: Has-text red. `isTmFilled` forced `false`: the `tm-rule` vitest red.
- Review patches (V1, V2, V3, B9, B10b) added tests only, plus the removed default in `segmentRuleInputOf`. Counter-checks: dropping the regroup adoption loop turns its new case red; `GridPanel.vue` passing an empty set turns `gridPanelTmFilled.test.ts` red.
- `check:debt-owner` Kiểm C is red on an epic-11 retro item owned by Story 11.8 (`done`); the diff does not touch that item.

## Spec Change Log

## Review Triage Log

- E2 (edge) refill after the user empties a filled segment — medium, intent_gap: a flush of `''` leaves an eligible empty draft, so every load refills it; the frozen eligibility rule does not say whether a user-emptied cell counts. Ice chose A (keep refilling), recorded in Decisions; no code change.
- V1 (gap) regroup-refresh adoption loop untested — medium, patch: every mock in `editorRegroupAssetRefresh.test.ts` returns `tm_filled_segment_ids: []`.
- V2 (gap) GridPanel wiring of the filled set untested — medium, patch: tests call `segmentRuleInputOf` directly; reverting `GridPanel.vue` stays green.
- V3 (gap) duplicate sources in one Chapter untested — medium, patch: no fixture repeats a source, so the `by_source` cache is unexercised; whitespace-only sources are dropped at split (part rejected).
- V-o1 / B1 / E10 a `read_`-named wire now writes — low, rejected: Q1 decision; both webview callers handle the ids; renaming is new IPC surface.
- V-o2 unaccented Vietnamese `eprintln!` — false: house convention (`lifecycle.rs:227`).
- V-o3 label test asserts `vi.json` text — low, folded into V2 (the mount test asserts the rendered label).
- B2 candidate SELECT has no empty-target filter — low, rejected: deliberate so the in-transaction re-check is the only, failable guard; indexed lookup costs ≈3.7 ms per 1,000.
- B3 / E1 a missing global store or unknown-origin pair fails the load — false: the missing store is a frozen decision; `tm_unit` has one writer (`insert_pair`, typed `PairOrigin`), the Global tier has none.
- B4 non-`Store` TM errors map to `MessageKey::Unknown` — low, rejected: the only such variant is the unknown origin, unreachable (B3).
- B5 / E5 regroup refresh can fill a cell with unflushed typing — low, rejected: regroup flushes first; the window is one IPC after the merge, and flush then wins in the DB, so `arbitrate` yields `self`.
- B6 refresh never removes ids of retired segments — false: retired rows are not loaded, so a stale id renders nothing.
- B7 / E6 marker gone after reload — false: Q2 decision (session-only).
- B8 `noteEditorEdit` drops the marker on non-changing events — false: `reportEdit` runs only on `input` and paste.
- B9 defaulted `tmFilledSegmentIds = new Set()` hides a missing argument — low, patch: one production caller; delete the default.
- B10a no normalization tests — false: frozen "no normalization". B10b index use not proven — low, patch: assert `EXPLAIN QUERY PLAN` uses `tm_unit_source_text`. B10c closures while `in-progress` — false: ledger convention.
- B11 missing tests: duplicates (= V3); Global-only fill — false (Origin-beats-tier fills from Global); mid-fill rollback and concurrency — low, rejected (one transaction under the existing lock); role segments — false (E3); language pair — false (target is always Vietnamese); seam counter-check — false (Implementation Notes, re-run by the orchestrator).
- E3 alt/caption filled, whitespace pair target — false: role segments join TM like any segment; confirm refuses whitespace targets.
- E4 `OpenWorkState` lock held across the fill — low, rejected: indexed lookups, same lock the read already held.
- E7 caret on a filled row hides `tm-rule` — false: caret precedence is the `resolveSegmentRule` contract.
- E8 payload without the new field is rejected — false: Rust and webview ship together; the guard is the house pattern.
- E9 the Has-text case cannot reach the re-check — false: the SELECT does not filter empty targets; forcing the re-check true turned it red.
- Orchestrator: fill writes `target_text` without `Indexer::rebuild` — medium, defer: pre-existing for flush, confirm and AI promote.

## Verification

**Commands:**
- `npm run test:story 7-4 -- --list`, then the listed targets -- expected: green.
- Full suite once by hand: migrations change (AGENTS.md).

## Acceptance criteria from epics.md

Source: `epics.md` §Story 7.4 (v6, nay ở archive-v6).

**Covers:** FR58

As a người dịch,
I want câu y hệt tôi từng dịch được điền sẵn nhưng vẫn chờ tôi gật đầu,
So that công cụ không bao giờ tự coi một câu là xong thay tôi.

**Acceptance Criteria:**

**Given** một segment có văn bản nguồn y hệt một cặp đã có trong TM
**When** mở
**Then** bản dịch cũ được **điền sẵn** vào Editor

**Given** một segment được điền sẵn từ khớp 100%
**When** hiển thị
**Then** vạch lề là `tm-rule` — **gợi ý cần xác nhận**

**Given** một segment được điền sẵn
**When** kiểm trạng thái
**Then** **chưa xác nhận**
**And** hệ thống **không** tự coi nó là đã hoàn thành

**Given** một segment được điền sẵn
**When** ghi xuống
**Then** ghi **ngay**, không đi qua bộ đệm gõ — đây là một hành động dứt khoát, không phải văn bản đang soạn
**And** **không** tạo `SegmentVersion`

**Given** người dùng xác nhận một segment được điền sẵn mà không sửa gì
**When** ghi xuất xứ
**Then** giữ nguyên xuất xứ của cặp TM nguồn

**Given** nhiều cặp TM cùng khớp 100%
**When** chọn cặp để điền sẵn
**Then** theo đúng thứ tự hai khoá của Story 7.3
