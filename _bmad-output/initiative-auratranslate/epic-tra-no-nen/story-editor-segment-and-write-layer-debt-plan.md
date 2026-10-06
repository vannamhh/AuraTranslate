---
ticket: 5
title: 'Story 11.5 — the editor and the write layer keep what they promise about origin, order and errors'
type: 'chore'
created: '2026-09-27'
status: done
baseline_revision: '968529a1b957baf4d6dea45de8591184772a8cb3'
route: 'dispatch'
review_loop_iteration: 0
context:
  - '{project-root}/src-tauri/AGENTS.md'
  - '{project-root}/src/AGENTS.md'
  - '{project-root}/_bmad-output/initiative-auratranslate/archive-v6/epic-11-context.md'
---

<frozen-after-approval reason="human-owned intent — do not modify unless human renegotiates">

## Intent

**Problem:** On HEAD `968529a`, 43 `deferred-work.md` items end in `Chủ: Story 11.5` (29 open, 14 🟡). The Task 0 re-read is in `11-5-task0-2026-09-27.md`. Seven of Ice's signed decisions (#62, #64, #65, #67, #96, #97, #105) are not fully in the code. A second sign-back to the load-time text loses the origin it had at load. A `.atproj` with an unknown `translation_origin` opens silently. NFC and NFD forms of the same text count as an edit. A migration backup is copied non-atomically and never checked. Confirm and flush errors show a fixed string instead of their `message_key`. Focus falls to `body` on three of five editor reload paths.

**Approach:** One spec for all 43 items, no lots. Each item gets one Epic 11 disposition, as listed below. Every fix gets a guard, and each guard is counter-checked by really removing its seam.

## Boundaries & Constraints

**Always:** Ledger items keep their text; only `→` lines are appended, and a stale claim gets a 🔵 fix in place. The spine's Stack table gets the `unicode-normalization` row BEFORE `Cargo.toml` changes (NFR15; licence evidence in the Task 0 file, L3802). A wire-shape change updates `ipc_contract.rs` and the direct `invoke()` calls in `e2e/`. Every non-typing write to `target_text` sets the baseline and `translation_origin` in the same `UPDATE` (AD-47). A check only a person can make in the real app goes to `Chủ: Epic 11`; a Windows-only check goes to `B7`.

**Never:** Add a migration, a gate, or any dependency other than `unicode-normalization`. Build new user-visible capability (L1840, L3150, L3814). Touch `[profile.release]` (L2211). Add a `segment.current_version_id` (#95). Widen `isBlocked()` in `main.ts` (L6981). Write decision numbers, dates or provenance into code comments (L10053).

## Dispositions

Agent, 2026-09-27 (from Task 0; Ice may override any line at approval):
1. ✅ fix, Rust: L251 (backup copies to `<target>.tmp`, compares sizes, then `rename`; a failure removes the tmp and returns `StoreError::OpenFailed`), L4023 (`read_chapter_segment_texts` gets `ORDER BY ord, id`), L3754 (#96: `confirm_segment` takes `origin_at_load`; a text equal to the load-time text writes back the load-time origin), L3787 (#97: `open_work` refuses a `project.db` whose `segment.translation_origin` holds a value outside `TRANSLATION_ORIGINS`, with its own `message_key`), L3802 (#105: the FR117 comparison applies NFC to both sides, after `trim()`), L10053 (#67: the snap comment states the direction as settled).
2. ✅ test-only, Rust: L4748 (a split-born row gets the same 14-column assertion as the merge-born one), L4115 (restore after merging two empty unconfirmed segments).
3. ✅ fix, webview: L140 (#62: `enterFocus('panel.grid')` after the reload in `finishImportSubmission`, `openWorkById`, `mergeCurrentChapterUp`, with e2e), L1384 (tab-strip arrows move DOM focus), L2610 + L3510 (#64: the row label renders the real `message_key` of the confirm error, the flush error and the three restore refusals, on one surface, with e2e), L3859 (#65: remove `'ornament'` from `SEGMENT_RULE_VALUES` and its branch; keep the `--color-ornament` token), L3877 (a test calls `sourceCutOffsetOf` on a mounted Hán Việt cell), L4201 (the header comment states the real rule: no value imports), L6981 (`clearSourceCuts` ignores `Escape` while the history overlay or the shortcuts overlay is open).
4. Self-closed with a pointer: L2177 (`ca33072`, `grid-empty-cell.e2e.mjs`), L733 (`deferred-work.md:8564`, #8), L284 (with L251).
5. `KHÔNG LÀM` with a reopen condition: L254, L260, L264 (Ice 2026-08-04), L2211, L3568 + L3650 (#95, AD-47 ⑤), L3660 (#2(b)), L3954, L4015, L4166, L4436.
6. Reassign: L1840 → `Chủ: Ice` (new capability; its `SegmentVersion` blocker is gone since Story 2.6).

Ice, 2026-09-27 (spec kept whole at 2 947 tokens):
7. L3150 + L3814 (option B): reassign → `Chủ: Sally`, to specify how a range or a group is selected; a later story builds it. No selection code in 11.5.
8. L3552 (option A): each of the 11 trust-only adapters in `src/config/segment.ts` validates the payload shape the way `readSegmentHistory` does with `isSegmentVersionArray`, and a bad shape returns an error instead of passing through.
9. L10067 (option A): `move_chapter` and `merge_chapter_into_previous` check that the Chapter exists before `normalize_chapter_ord`, the order the Chapter split already uses; an unknown id writes nothing.
10. L4849 (option A): `KHÔNG LÀM`. PASSIVE-busy retry is by design and `ci-previous-verdict` warns on a red CI. Reopen when the flake rate rises above 7/14 or blocks a release.
11. L10545 (option B): `settled_wal_len` returns whether it settled or gave up at the deadline, and each call site decides what an unsettled read means for its assert.
12. L10561 (option B): `file_len` maps `NotFound` to `0` and panics with the path and error on any other error; the `!wal.exists()` assert uses `try_exists()`.
13. L12143 (option A): a failed promote shows `editorPromoteAiTranslationError` through a `tError()` surface, like `editorConfirmError`; the two narrow `vi.doMock` factories gain the export.
14. L12359 (option A): `promote_ai_translation` takes `force`. When the current text is non-empty and has no `segment_version` copy, it returns `needs_confirmation` with the draft and writes nothing; the caller asks, then calls again with `force = true`. Same shape as `restore_segment_version`. The native-undo loss in `replaceEditorSegment` stays out of scope.
15. L12364 (option A): inside a typing zone, `keys.ts::handle()` always yields `Mod+Z` and `Mod+Shift+Z` to the native editor, whatever is registered.
16. L2307 (option A, amended by Ice 2026-09-28): no automated caret-rect assert. `getClientRects()` of a collapsed range in an empty cell returned no rect in 3 of 3 WKWebView runs, which cannot tell an invisible caret from an API that does not measure one. L2307 goes to 🟡 with `Chủ: Epic 11`: Ice looks at the caret on an empty cell in the real-use pass.
17. L4364 (option A): while `confirmInFlight` is set, navigation commands are refused with a warning, like `regroupInFlight`.

</frozen-after-approval>

## Code Map

- `src-tauri/src/core/store/schema.rs` -- `backup_before_migration` ~:2491 (L251). `segment` DDL ~:1786 has no `CHECK` on `translation_origin`; leave it that way.
- `src-tauri/src/commands/segment.rs` -- `confirm_segment` origin branch ~:2426-2443 (L3754, L3802), `TRANSLATION_ORIGINS` ~:2065 (L3787), `promote_ai_translation` ~:2107 and the `restore_segment_version` pattern ~:807-838 (Decision 14), snap comment ~:3062 (L10053), `split_segment` ~:3389.
- `src-tauri/src/commands/project/mod.rs` -- `open_work` ~:5050 next to `SchemaTooNew` (L3787), `read_chapter_segment_texts` ~:1922 (L4023).
- `src-tauri/src/commands/chapter.rs` -- `move_chapter` ~:655, `merge_chapter_into_previous` ~:763; the split path ~:965-988 is the right order (Decision 9).
- `src-tauri/src/core/store/mod.rs` -- `StoreError`/`message_key`, `SchemaTooNew` shape to mirror.
- `src-tauri/tests/{segment_contract,store_contract,ipc_contract}.rs`, `commands/project/tests.rs` -- merge-column test ~:6920 to mirror; `file_len` :82, `settled_wal_len` :563.
- `src/panels/editorPanelState.ts` -- `confirmCurrentSegmentUnguarded` (thread `originAtLoad`), `confirmInFlight` ~:1134, `regroupInFlight` ~:2555, `editorPromoteAiTranslationError` ~:284. `src/config/segment.ts` -- the 12 adapters, `isSegmentVersionArray` :394 (Decision 8).
- `src/modes/{libraryImport,libraryChapters}.ts` -- the three reload paths (L140); copy the tail of `switchChapter`.
- `src/panels/GridPanel.vue` -- tab strip ~:1617, row label ~:1867 (uses a fixed key), `STATE_LABEL_KEYS` ~:321. `src/panels/editorSegments.ts` -- `SEGMENT_RULE_VALUES` :69, branch :162, header :26.
- `src/main.ts` -- the `clear_source_cuts` gate; `segmentHistoryState.ts::historyIsOpen`. `src/commands/keys.ts` :510 (Decision 15).
- `e2e/specs/{segment-history-restore,grid-empty-cell}.e2e.mjs` -- direct `invoke()` of `confirm_segment` at :77, :175.

## Tasks & Acceptance

**Execution:**
- [x] `ARCHITECTURE-SPINE.md` Stack row, then `src-tauri/**` -- Dispositions 1-2 and the Rust answers -- Rust phase.
- [x] `src/**`, `vi.json`, `tests/frontend/*`, `e2e/specs/*` -- Disposition 3 and the webview answers -- Webview phase.
- [x] A real-removal counter-check for every new guard; the full suite once (`Cargo.toml`/`Cargo.lock` change) -- Tests phase.
- [x] `deferred-work.md` -- one `→` disposition for each of the 43 items -- Ledger phase.

**Acceptance Criteria:**
- Given the story is done, when `npm run check:debt-owner` runs, then it is green and no open or 🟡 item ends in `Chủ: Story 11.5`.
- Given a segment loaded with `bilingual_import`, when it is signed, edited, signed, edited back to the load-time text and signed again, then its `translation_origin` is `bilingual_import`.
- Given a target that differs from the load-time text only by NFC/NFD form, when it is signed, then its origin is unchanged.
- Given a `project.db` with a `translation_origin` outside the catalogue, when `open_work` runs, then it refuses with its own `message_key` and writes nothing.
- Given a migration backup, when it finishes, then the backup's size equals the source's and no `.tmp` remains.
- Given a confirm, flush or restore refusal, when the row is rendered, then its label shows that error's `vi.json` text.
- Given a Work switch, an import or a Chapter merge, when the editor reloads, then focus is inside the grid, not on `body`.
- Given an unconfirmed draft with no `segment_version` copy, when an AI translation is promoted without `force`, then `needs_confirmation` returns the draft and `target_text` is unchanged.
- Given a Chapter id that does not exist, when `move_chapter` or `merge_chapter_into_previous` runs, then no `chapter.ord` changes.
- Given a segment adapter receiving a payload of the wrong shape, when it resolves, then it returns an error rather than the payload.
- Given a typing zone in focus, when `Mod+Z` is pressed, then no registered command runs and the native editor handles it.
- Given a confirm in flight, when a navigation command runs, then the caret stays where the confirm puts it and a warning is shown.
- Given each new guard, when its seam is really removed, then it goes red for that reason.

## Implementation Notes

- `ChapterSegment` had never carried `translation_origin`, so the first webview pass sent `originAtLoad = ''`; with the keep-branch now returning `origin_at_load`, re-signing an unedited `bilingual_import` segment would have written `''`. Phase 2b added the column to `select_chapter_segments`/`read_fresh_rows` and reads it from the load-time snapshot; AI promote now mirrors the origin into the snapshot too.
- L3787's refusal lives in `open_work` (`reject_unknown_translation_origin`), not `Store::open`: only the project store has the column. It names `params_from_iter` from `core::store` because `store_boundary.rs` bars `rusqlite` outside `core/store`.
- Decision 11: the baseline WAL read did not settle in 3 of 3 local runs (the `CREATE TABLE` just before it is itself a write), so `settled` is reported in the diagnostic output and no call site asserts on it.
- Guards that cannot go red: L4023 (`segment` has no index on `ord`, so ties already come back in `id` order; the symmetric test was deleted), L251 (a torn copy cannot be injected), Decision 12 (a non-`NotFound` stat error cannot be injected). L140's e2e passes with the seam removed: the Story 5.7 caret watcher already focuses the grid for a non-empty Chapter.
- Decision 8 reached two adapters the first pass left shallow: `isChapterSegments` and `isRegroupOutcome` now check every row (`isChapterSegmentRow`). The deeper guard exposed a `librarySearch.test.ts` fixture missing two fields.
- `Mod+Enter` does not reach `CommandRegistry` under WebDriver on WKWebView, so the #64 e2e calls `confirmCurrentSegment()` directly; the three L140 paths are driven by importing their functions, because every UI path to them runs while workspace mode is detached.
- Row-label priority when several errors hit one row (confirm, then restore, then flush) is an implementation choice, not an Ice decision.
- Local e2e: `segment-history-restore`, `segment-merge-split`, `library-mode-switch-focus`, `grid-row-error-label` green; `grid-empty-cell` back to its baseline asserts (Decision 16 amended).

## Spec Change Log

- 2026-09-28, Tests phase 3b. Trigger: the Decision 16 e2e assert read `caretRectHeight === null` in 3 of 3 local WKWebView runs; the number fits both "caret not drawn" and "no rect for a collapsed range in an empty element". Amended by Ice: the assert is removed and L2307 becomes a real-use check owned by Epic 11. KEEP: the existing selection-state asserts in `grid-empty-cell.e2e.mjs`.

## Review Triage Log

- blind: `promote_ai_translation`'s doc says the caller must flush first, but `main.ts` calls `promoteAiTranslationToEditor` without a flush, so an unsigned draft still in the AD-35 buffer is invisible to Decision 14's hold and gets overwritten — medium, patch: flush through `flushEditorBeforeDiscreteWrite` first, as `restoreVersion` does.
- blind + edge: `reject_unknown_translation_origin` hardcodes `?1..?4` while binding `TRANSLATION_ORIGINS` — low, patch: build the placeholder list from the array length.
- blind: `promote_ai_translation`'s `Arc<Mutex<Option<String>>>` duplicates the draft the write closure already returns — low, patch: derive `unsigned_draft` from the returned tuple.
- blind: the `Mod+Z` test covers only `metaKey` on bare `KeyZ`; the `ctrlKey` (Windows) branch and `Mod+Shift+Z` are untested — low, patch: two more cases in the same file.
- blind + edge: `promote_ai_translation` writes into a retired row; reachable because `aiTranslateRunSegmentId` can name a segment merged away after the AI run — medium, defer: the unconditional `UPDATE` predates this story.
- blind: `open_work`'s catalogue scan reads the whole `segment` table on every open, unmeasured — maybe-false (medium if true), defer: settle by timing `open_work` on the largest real Work.
- verification-gap + blind: `library-mode-switch-focus.e2e.mjs` stays green with the three `enterFocus('panel.grid')` lines removed (the caret watcher covers every non-empty Chapter) while the L140 ledger line reads ✅ — medium, defer: the behaviour is measured, the seam needs a fixture `openWorkspaceWithWork()` cannot build.
- blind: `editorConfirmInFlightBlocksNav.test.ts` drives one of three navigation commands — low, rejected: all three are one-line calls through the same `dieuHuongVaBao` guard (`editorPanelState.ts:1685-1695`), and the fix only adds tests.
- blind: `flushError` is not cleared when a flush fails with `error === null` — low, rejected: that branch means no IPC bridge (`npm run dev` in a browser), never the packaged app.
- blind: the two promote-confirm buttons look identical — low, rejected: the surface copies `SegmentHistoryOverlay.vue`'s shipped question; differentiating them is a design change.
- edge: `confirm_segment` writes any `origin_at_load` it is given, and an unknown value would make `open_work` refuse the Work — low, rejected: every caller takes the value from a row `open_work` already validated or from a catalogue literal (e2e), and the fix adds a guard plus a new error key for a state no path reaches.
- coordinator: the diff adds new `═══`/`───` banner blocks and section-title comments that root AGENTS.md §Code comments forbids — low, patch: remove the added banners, keep only allowed one-to-two-line comments.

## Verification

**Commands:**
- `npm run build && cargo test --test segment_contract --test store_contract --test ipc_contract` -- expected: green.
- `npx vitest run <changed test files>` -- expected: green.
- `npm run check:debt-owner && npm run check:i18n && npm run check:commands && npm run check:tokens` -- expected: green.

## Acceptance criteria from epics.md

Source: `epics.md` §Story 11.5 (v6, nay ở archive-v6).

As a chủ dự án,
I want nợ nhóm editor, segment và tầng ghi (Epic 2, `core/store`) được đóng hoặc quyết dứt điểm,
So that cặp TM ghi tại chuyển tiếp xác nhận (AD-31) đi qua một đường ghi đã vá.

Thừa kế AC chung của epic (xem Notes của `epic-tra-no-nen.md`).

**Acceptance Criteria:**

**Given** mọi mục mang `Chủ: Story 11.5` trong `deferred-work.md`
**When** story hoàn tất
**Then** thoả AC chung của Epic 11; AC riêng rút từ các mục lúc `create-story`
