# Handoff phase 1 (Rust) -> phase 2 (Webview): story 5-18

Rust is done and uncommitted. Plan: `tickets/epic-library/tickets/story-at-trang-thai-chuong-ngay-trong-workspace-plan.md`.

## IPC wire shape of `chapter_started`

Always a plain `boolean` field, snake_case on the result (like every other result field). `true` only on the write that actually moved the Chapter `not_started` -> `in_progress`; `false` on every later write, on `needs_confirmation` holds and on no-op writes. Arguments of the existing commands are unchanged.

| Command (invoke name) | Result type | Where the flag is |
|---|---|---|
| `save_segment_targets` (flush) | `SaveOutcome` | `{ chapter_id, saved, chapter_started }` |
| `promote_ai_translation` (also batch AI acceptance) | `PromoteAiTranslationOutcome` | `{ segment_id, target_text, translation_origin, status, needs_confirmation, unsigned_draft, chapter_started }` |
| `accept_tm_fuzzy` | `PromoteAiTranslationOutcome` | same as above (goes through promote) |
| `accept_tm_exact` | `PromoteAiTranslationOutcome` | same as above |
| `review_accept_change` | `PromoteAiTranslationOutcome` | same as above |
| `restore_segment_version` | `RestoreOutcome` | `{ segment_id, status, restored, needs_confirmation, unsigned_draft, chapter_started }` |
| `ai_proofread_accept` | `ProofreadAcceptOutcome` (flattened) | `{ segment_id, ..., chapter_started, decision_id }` (flag at top level) |
| `ai_proofread_revert` | `PromoteAiTranslationOutcome` | same as above |

That is eight commands (the plan says "six paths"; accept_tm_fuzzy/exact, proofread accept/revert count as pairs). `src/config/segment.ts` types for these results need the new `chapter_started: boolean` field. Not touched by the flag: `confirm_segment`, `set_segment_omitted`, `merge_segments`, `split_segment`, `read_open_chapter_segments` (TM prefill never starts a Chapter).

## New async command

`refresh_after_chapter_started` (Tauri invoke name, `#[tauri::command(async)]`, in `commands::lifecycle::wire`). No arguments (`invoke('refresh_after_chapter_started')`). Returns the same `WorkLifecycle` as `set_chapter_status`: `{ status, status_is_override, derived_status }` (see `WorkLifecycle` in `src-tauri/src/commands/lifecycle.rs`). It rewrites `meta.json` from the store and reindexes the Library; it does not UPDATE any Chapter. Error cases: no Work open -> `work.none_open`; meta write failure -> `work.create_failed`.

Registered in `src-tauri/src/lib.rs` after `set_chapters_status`. No capability/permission file needed (checked: none list `set_chapters_status`).

Webview rule: when a result carries `chapter_started === true`, call `refresh_after_chapter_started`, then `loadChapters()` (the in-memory `ChapterRow.status` is stale until then). Library index is only correct after this call; if the app dies in between, `project.db` is right and Library catches up at the next status write.

## Other things the webview phase must know

- `lifecycle.set_chapter_done` (existing) uses `set_chapter_status`; unchanged. `loadChapters()` after it is still the webview's job.
- Migration 42 (`PROJECT_MIGRATIONS`, `CHAPTER_STARTED_BACKFILL_DDL`) moves old `not_started` Chapters with a non-blank translation to `in_progress`. `open_work` then rewrites `meta.json` if derived status/progress changed, and the `open_work` IPC shell reindexes the Library when the index row differs from the new meta. Nothing for the webview to do, but a Work opened right after upgrading can show Chapters that are now Dang dich.
- The Rust rule lives in `commands::lifecycle::mark_chapter_started` (+ `mark_segment_chapter_started`); the "non-blank" predicate is the `segment_has_translation_sql!` macro in `core/store/schema.rs` (same 25 White_Space code points as `str::trim`). The webview must not re-derive "chapter started" from text.
- New `Store::opened_from_version()` accessor (version found before migrating).
- Rust tests added: `tests/lifecycle_contract.rs` (flush matrix rows, promote, refresh command + index, migration 42), `tests/tm_contract.rs` (exact, fuzzy, prefill keeps not_started), `tests/alignment_contract.rs` (review accept), `tests/segment_contract.rs` (restore), `tests/proofread_decision_contract.rs` (accept + revert). Version-pinned tests bumped 41 -> 42; `config_invariants.rs` census row for `lifecycle.rs` 3 -> 4 async.
- `confirm_segment` does NOT report `chapter_started` (it writes no text); the "last segment confirmed" hint is purely webview state per the plan.

## Known red until the webview phase

`cargo test --test ipc_argument_contract` (`every_registered_commands_business_parameters_match_camel_case_across_the_wire`) fails with "registered but no `invoke()` found: refresh_after_chapter_started". It goes green as soon as `src/config/` (or wherever the six gates live) contains an `invoke('refresh_after_chapter_started')` call. Full `cargo test --no-fail-fast` otherwise: 101 targets green, this one red.

# Phase 2 (Webview) -> phase 3 (Tests)

Webview done, uncommitted. Nothing in `src-tauri/` touched.

## Wiring
- Helper lives in `src/config/lifecycle.ts` (no new file: `check-*` floors count `src/**/*.ts`). `noteChapterStarted(outcome)` is called, un-awaited, from the adapters of the eight commands (`config/segment.ts` x5, `alignment.ts`, `proofread.ts` accept+revert). Flag true -> `refresh_after_chapter_started` -> registered reload (`libraryChapters.loadChapters`, registered at module load). Result types and runtime guards now REQUIRE `chapter_started: boolean`; fixtures that mock the wire without it become `ipc.unknown`.
- `lifecycle.set_chapter_done` handler `setOpenChapterStatus` now: error into new `openChapterDoneError` (not `openWorkLifecycleError`), on success dismisses the suggestion and reloads the chapter list.
- New command `lifecycle.dismiss_chapter_done_suggestion` (deps `dismissChapterDoneSuggestion`, wired in `main.ts`).

## Selectors
- Workspace Done button (StatusBar, workspace mode, open Chapter): `[data-chapter-done]` (disabled + "Chương đã xong" when the open Chapter is done or lifecycle busy). Error: `[data-status-chapter-done-error]` (role=alert).
- Suggestion strip (WorkspaceMode): `[data-chapter-done-suggestion]`, actions `[data-chapter-done-suggestion-accept]` and `[data-chapter-done-suggestion-dismiss]`. Visible = `libraryChapterDoneSuggestionVisible` (suggested and open Chapter not done).

## Removed from Library
- Button `[data-lifecycle-action="set_chapter_done"]` in `LibraryWorkPage.vue` and key `mode.library.set_chapter_done`. The command stays.

## e2e to change (phase 3)
- `e2e/specs/story-5-5-progress.e2e.mjs`: replace `setChapterStatusViaIpc(chapterId, 'done')` (~:169/:206) with clicking `[data-chapter-done]` in Workspace (the Chapter must be the open one).
- `e2e/specs/story-5-12-reading-frontier.e2e.mjs`: keep the IPC stand-in (Chapter 1 is not the open Chapter); fix header comment lines ~21-23 that say Story 5.7 owns the missing button.

## Status
- vue-tsc 0; check:commands 0, check:tokens 0, check:panel-refs 0; `ipc_argument_contract` green.
- `check:i18n` red with ONE failure not from this phase: `src/panels/AiTranslationPanel.vue:655 {{chapterRunLabel}}` (file unmodified, from story 4-15).
- Existing fixtures updated with `chapter_started: false`: aiBatchPanelRows, aiTranslatePromoteConfirmFlow, proofreadDecisionRatio, tmFuzzyAdapters. `test:story 5-18` still lists 8 test files the plan's Code Map does not name (4 Rust + these 4 frontend): add to "Tests that move".

# Phase 3 (Tests)

e2e changed: `story-5-5-progress.e2e.mjs` now clicks `[data-chapter-done]` with `realClick` and waits for it to become disabled (IPC stand-in and `readOpenChapterIdViaIpc` removed, header updated). `story-5-12-reading-frontier.e2e.mjs`: stand-in kept, header limit 3 corrected (Chapter 1 is not the open Chapter). No e2e spec or helper fakes results of the eight commands (specs call the real ones), so nothing to add there.

Commands (exit): `npm test` (full vitest) 0, 239 files / 3414 tests, no Errors/Unhandled lines · 14 pre-push gates: all 0 except `check:i18n` 1 (only the known `AiTranslationPanel.vue:655 {{chapterRunLabel}}`) · `npm run build` 0 · `cargo test --test ipc_argument_contract --test lifecycle_contract` 0 (1 + 27 passed) · eslint on the two specs 0 · e2e 5-5 + 5-12 (outside sandbox, `TAURI_WEBDRIVER_PORT=4545`) 0, 2 specs passed.

First e2e run (default port) was red for environment: port 4445 is held by Google Drive Suite (`gdrive-suite-manager`, pid 60318), so the app could not bind its embedded server. Override via `TAURI_WEBDRIVER_PORT` fixes it; no code change.

## Review fix: confirm counts

- `confirm_segment` (Rust) now carries `chapter_started` in `ConfirmOutcome`; `mark_segment_chapter_started` runs in the write transaction only on the real transition (already-confirmed no-op returns `false`). Test: `confirming_a_prefilled_sentence_starts_the_chapter_once` in `lifecycle_contract.rs`.
- Webview: `ConfirmOutcome.chapter_started` is required by the guard and `confirmSegment` calls `void noteChapterStarted(outcome)`; `confirm_segment` added to the CASES table in `libraryWorks.test.ts`; confirm fixtures in eight vitest files gained `chapter_started: false`.
