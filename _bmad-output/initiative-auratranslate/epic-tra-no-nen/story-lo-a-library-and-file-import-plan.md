---
ticket: 6
title: 'Story 11.6, lot A — Library, the file-import pipeline and the N-Chapter Glossary scan keep what they promise'
type: 'chore'
created: '2026-09-28'
status: done
route: 'dispatch'
baseline_revision: '21a2521149921251adef1d4989844030a8a8ae45'
review_loop_iteration: 0
context:
  - '{project-root}/src-tauri/AGENTS.md'
  - '{project-root}/src/AGENTS.md'
  - '{project-root}/_bmad-output/initiative-auratranslate/archive-v6/epic-11-context.md'
---

<frozen-after-approval reason="human-owned intent — do not modify unless human renegotiates">

## Intent

**Problem:** On HEAD `21a2521`, 77 `deferred-work.md` items end in `Chủ: Story 11.6` (Task 0: `11-6-task0-2026-09-28.md`). This lot takes 38 of them: Library/index/search, the file/text import pipeline, and the three `spawn_import_scan` items. Eight signed Ice decisions (#2, #4, #6, #9, #28, #82, #83, #108) are not in the code. Chapters 2..N of a multi-Chapter import get no Glossary scan, with no error. `đ`/`Đ` never folds to `d` in Library search. Orphans only show after a manual Rescan.

**Approach:** Lot A of two (lot B, the URL/image/docx/APPEND/bilingual items, is `spec-11-6-lo-b-*`). The story goes `done` after lot B. Each item gets one Epic 11 disposition, listed below. Every fix gets a guard, and each guard is counter-checked by really removing its seam.

## Boundaries & Constraints

**Always:** Ledger items keep their text. Only `→` lines are appended, and a stale claim gets a 🔵 fix in place. A wire-shape change updates `ipc_contract.rs` and the direct `invoke()` calls in `e2e/`. A check only a person can make in the real app goes to `Chủ: Epic 11`. The `commands/project/mod.rs` split (#83) runs last, after every behaviour fix in that file, and changes no behaviour.

**Never:** Add a gate or a dependency. Add a migration other than `work.last_chapter_id` (#4). Build a user-visible capability nobody signed. Change AD-8's single derived write path of `Indexer::rebuild`. Change `sanitize_name`'s fallback. Write decision numbers, dates or provenance into code comments.

## Dispositions

Agent, 2026-09-28 (from Task 0; Ice may override any line at approval):
1. ✅ fix, Rust: L1916 (#2: script-ratio warning when the text's script does not match `source_lang`, at create and at append, non-blocking), L7733 (#4: `work.last_chapter_id`, written on Chapter switch; `open_work` falls back to the first Chapter when it is `NULL` or stale), L7807 + L7823 (#6: one `đ/Đ → d` fold on the index side and the query side, then `library_source_fts_nd` verified with that fold instead of `.contains()`), L8532 (`detect()`'s `Some(i)` arm falls back to the first decodable candidate, like the `None` arm), L10792 + L10818 (#82: the seven `sync_threadpool` comments state the real mechanism, without a count), L10946 + L10981 (#83: move the bilingual, URL-import and create-work/image blocks out of `commands/project/mod.rs`, then fix every stale `commands/project.rs` path).
2. ✅ fix, both layers: L8989 + L9226 + L11142 (Ice, 2026-09-28: one mechanism for all three paths; `spawn_import_scan` takes the N new Chapter ids for the pattern split, the URL list and APPEND; the scan time at N = 20, 50 and 2000 is measured before choosing background or inline, and a number over the NFR budget goes back to Ice), L7523 (a pure-read `library_list_orphans`, called from `LibraryMode.vue`'s `onActivated`).
3. ✅ fix, webview: L726 (the create button stays disabled while the name trims empty), L8640 (#9: explanatory copy when a `^`-anchored pattern yields one Chapter), L8018 (#108: `ReadingMode.vue`'s `onDeactivated` releases the segment array; the NFR5 re-measurement goes to `Chủ: Epic 11`).
4. ✅ test-only: L4527 (e2e: focus is not on `body` after a Chapter switch), L8557 (DOM tests for `ImportPreviewOverlay.vue`: Tab trap, confirm/cancel while `confirming`, radio `@change`), L8865 (`count_in_import` through the real multi-Chapter product path), L10843 (`thread::scope` cases for the four uncovered wires).
5. Self-closed with a pointer: L10860 (`c6e5e215326153a25befe3af41cdae54c17ba142`, `grid-image-row-alignment.e2e.mjs`).
6. `KHÔNG LÀM` with a reopen condition: L7579 (single `OpenWorkState`), L8951 (second in-memory copy; reopen at a real import of dozens of Chapters).
7. Reassign: L730 → `Chủ: Sally`, L7556 → `Chủ: Winston`, L8346 + L10874 → `Chủ: Ice`, L9106 + L9133 → `Chủ: Story 10.9`.

Ice, 2026-09-28 (spec kept whole at ~3 000 tokens):
8. L847 (option a): `create_work` and the APPEND path reject a `source_lang` outside `zh`/`en` with their own `message_key` and write nothing.
9. L7423, L7839, L7940, L8313 (option a each): `KHÔNG LÀM`. Reopen: L7423 when a nightly red slips past `pre-push` that a Rust command test would have caught; L7839 when Story 10.9 measures `rebuild` over its NFR3 budget; L7940 when a real per-Work `INSERT` failure is seen or can be injected; L8313 when the type must cross IPC or the decoder crate changes.
10. L7984 (option a): reading level, bilingual and the font-size/line-height overrides persist through an eighth `BootstrapConfig` field; the frozen field list in `ipc_contract.rs` changes in the same commit.
11. L8505 (option c): the 4 KiB window stays; the confidence copy no longer implies the whole file was checked. The decode risk stays open with a reopen condition.
12. L8692 (option a): on the URL path, the candidate and self-declared previews run extraction before normalising. The Rust phase times it at 5 candidates × N = 20 Chapters; a number over the preview budget goes back to Ice before landing.
13. L10724 (option a): `preview_import_encoding_from_text`, `rebuild_bilingual_import_preview` and `remove_url_import_item` become `(async)`; this reopens AI-4 D4's six-wire boundary and triggers the full suite.

Ice, 2026-09-28, during the Rust phase:
14. L7807 + L7823 (option b): a tolerant-mode snippet shows the original text with its diacritics, not the folded column. Each fold maps one character to one character, and the `‹›` marks are placed on the original text at the same character positions.

</frozen-after-approval>

## Code Map

- `src-tauri/src/commands/project/mod.rs` (5 376 lines) -- `create_work` INSERT ~:701 (L847, L1916), append ~:909-1022, `spawn_import_scan` :2049, `open_work` ~:5092 (L7733), `render`/`cleanup_and_chapters_preview_for` (L8865). #83 target: re-measure the block ranges before moving.
- `src-tauri/src/commands/project/wire.rs` -- the three `spawn_import_scan` call sites :511, :553, :844; the APPEND branch of `confirm_import_with_encoding` :769; sync wires :586, :1040, :1298; `(async)` sites; `reindex_library` :419.
- `src-tauri/src/core/library/indexer.rs` -- `rebuild` :220, `DELETE FROM library_segment` :379/:551, `harvest_work_text` :1428, `remove_diacritics` FTS options :764+. `meta.rs` :319 (live `updated_at`).
- `src-tauri/src/core/library/atproj.rs` -- `sanitize_name`; do not touch (L726 is a UI guard).
- `src-tauri/src/core/segment/encoding.rs` -- `EVIDENCE_WINDOW_BYTES` :64, `detect()` ~:182-217 (L8532), `render_candidates` ~:281-299 (L8692). `pipeline.rs` :269 (`PipelineInput.encoding`).
- `src-tauri/src/commands/library/wire.rs` -- `library_list_works`, the pure-read pattern for `library_list_orphans`.
- `src-tauri/tests/segment_contract.rs` :9338, `bilingual_import_contract.rs` :1232 -- the `thread::scope` template (L10843). `config_invariants.rs` `COMMAND_FILE_CENSUS` (L10818: point at it, no count).
- `src/modes/LibraryMode.vue` -- name trim :224, `onActivated` :158-186; `src/modes/libraryImport.ts` -- the existing `.trim() === ''` disabled guard to mirror.
- `src/ImportPreviewOverlay.vue` -- trapTab :521, `.ip-candidate` :1069 (L8557, L8640). `src/importPreviewState.ts` -- the `confirming` guards.
- `src/modes/ReadingMode.vue` -- `onDeactivated` :160-185, segment `v-for`. `src/modes/readingState.ts` :459-562 (Decision 10), with `src-tauri/src/commands/config.rs::BootstrapConfig` :57 and the frozen field list in `src-tauri/tests/ipc_contract.rs`.
- Decision 13: the `(async)` attribute lives on the wire in `wire.rs`; `lib.rs` :721/:736 registration lines do not change. `tests/config_invariants.rs` and `tests/ipc_contract.rs` assert the async set and must be updated with it.
- `src/panels/editorPanelState.ts` -- `switchChapter` (L4527 guard only; also where L7733 writes `last_chapter_id`).

## Tasks & Acceptance

**Execution:**
- [x] `src-tauri/**` -- Dispositions 1-2 and the Rust answers, then #83 last -- Rust phase.
- [x] `src/**`, `vi.json`, `tests/frontend/*`, `e2e/specs/*` -- Dispositions 2-4 and the webview answers -- Webview phase.
- [x] A real-removal counter-check for every new guard; the full suite once (migration and `lib.rs`) -- Tests phase.
- [x] `deferred-work.md` -- one `→` disposition for each of the 38 items -- Ledger phase.

**Acceptance Criteria:**
- Given lot A is done, when the lot-A item lines are read, then each of the 38 ends in one `→` disposition and `npm run check:debt-owner` is green.
- Given a 3-Chapter pattern-split import, a 3-link URL import and a 3-Chapter APPEND, when each finishes, then every new Chapter has been scanned, and the measured scan time at N = 20, 50 and 2000 is recorded in the spec.
- Given a Library entry with `Đà Nẵng`, when the query `da nang` runs in tolerant mode, then the Work is found, in the FTS half and in the verbatim half.
- Given a Work reopened after a switch to Chapter 3, when `open_work` runs, then Chapter 3 opens; given that Chapter was deleted, then the first Chapter opens.
- Given the Library root has an orphan, when Library mode is activated, then the orphan shows without a Rescan.
- Given Vietnamese text imported as `zh`, when the Work is created or appended to, then the warning is shown and the write still happens.
- Given `detect()` picks a candidate that cannot decode the bytes, when it returns, then the encoding decodes them.
- Given a create or APPEND call with `source_lang = "ja"`, when it runs, then it refuses with its own `message_key` and no row is written.
- Given reading level, bilingual and font-size set, when the app restarts, then Reading mode opens with them.
- Given a URL Chapter whose page chrome holds text the extractor drops, when its encoding preview renders, then the normalised counts equal those written on confirm.
- Given Reading mode is deactivated, when the KeepAlive cache holds it, then its segment array is released.
- Given each new guard, when its seam is really removed, then it goes red for that reason.

## Implementation Notes

- Glossary scan of N new Chapters (`bench-release`, Ice's Mac, real `resources/dict/` layers, a 48 640-char Chinese Chapter split by `split_source_text`): 91–151 ms per Chapter across two runs; N = 2000 ran for real at 302 s, plus 1.3 s for the enqueue writes. The scan runs on a detached thread after commit and holds `OpenWorkState` only to enqueue, so no IPC call or auto-save flush waits on it. No NFR caps completion latency; a 2000-Chapter import fills its candidates over about five minutes.
- One `ImportScanGeneration` per batch, not per Chapter. The baseline scanned only the first Chapter; calling the old single-Chapter shape N times would have made each call cancel the one before, leaving only the last Chapter scanned.
- `trigram remove_diacritics 1` did not fold Vietnamese marks (0 hits for `da nang` on `Đà Nẵng`), so `library_segment` gained `target_text_fold`/`source_text_fold` and `library-index.db` moved to schema 8 (derived store, rebuilt on mismatch). Tolerant snippets are rebuilt on the original text (Decision 14).
- Decision 12 cost: `webimport::extract`, 5 candidates × 20 Chapters, debug build, 420 ms total. The URL preview now decodes the whole body strictly before extracting; non-URL paths are unchanged.
- `source_lang` validation and the script-ratio warning sit inside `create_work`/`append_chapters_to_work`, so every product path gets them, not only the `_from_text`/`_from_file` wrappers.
- Wiring `library_list_orphans` on activation made `libraryScanHasLoaded` true without a rescan, and the UI showed "Đã lập chỉ mục 0". A separate `rescanResultHasLoaded` now gates the rescan result lines.
- Reading preferences go through `writeSchedule.ts` with the layout pair; the sliders fire on `@input`.
- #83: `commands/project/mod.rs` went from 5 549 to 3 196 lines, split into `work_creation.rs`, `bilingual.rs` and `url_import.rs`, re-exported with `pub use`. The preview system and the scan subsystem stay in `mod.rs`.
- Guards that cannot go red here: the L4527 e2e (the caret watcher already focuses a non-empty Chapter), and the concurrent create + reindex case (removing `rebuild_lock` stayed green 20 of 20 times; no delay can be injected).

## Spec Change Log

## Review Triage Log

- verification-gap: `open_chapter`'s `set_open_chapter` call has no test that reopens the Work; only `open_adjacent_chapter` is covered — medium, patch: a reopen test driven through `open_chapter`.
- blind: reading preferences have no last write on window close, unlike the layout's `beforeunload` in `WorkspaceDock.vue` — medium, patch: flush the pending write on `beforeunload`.
- blind + edge: `source_lang_mismatch` samples only `chapters.first()`, so a mismatch in Chapter 2..N of a create or APPEND never warns — medium, patch: warn when any new Chapter mismatches.
- edge: a literal `‹`/`›` in stored text ahead of a tolerant match shifts `first_marker_span_chars`, so the snippet marks the wrong span — low, patch: private-use marker characters for the internal `highlight()` call.
- edge: `fold_diacritics_case_preserving` (NFD, then drop U+0300..U+036F) is not one-to-one for characters whose marks fall outside that range (kana with dakuten, Hangul), breaking Decision 14's position mapping — medium, patch: fold per character, keeping exactly one output character each.
- edge: the scan measurements required by the AC reach only `eprintln!` — low, handled: recorded under Implementation Notes (not a code fix).
- edge: when `guarded_dict_layers` returns `None`, no Chapter of the batch gets `import_scan_failed` — maybe-false, rejected: the baseline returned silently the same way for its one Chapter; whether the UI waits on that event was not shown.
- blind: the bilingual submit button checks `name`, not `effectiveName` — false: `submitBilingualFilePath` always creates a new Work with `name.value` and has no destination picker.
- blind: the L4527 e2e case stays green with `enterFocus('panel.grid')` removed — low, rejected: disclosed in the case and in its ledger line; isolation needs an empty-Chapter fixture.
- blind: `create_work_from_file` has no dedicated `source_lang` rejection test — low, rejected: the check lives in the shared `create_work` body that the text-path test already removes-and-reds.
- blind: no test drives a short query through the new verbatim `_nd` branch — low, rejected: the branch sits under the same `if !short_query` as its exact sibling.
- blind: the `_nd` superset-of-exact property is asserted by measurement, not at runtime — low, rejected: design predates this lot; a runtime superset check would add a second query per search.
- blind: the `^` hint fires only on a literal leading `^` — false: Decision #9 names exactly a pattern that begins with `^`.
- blind: the orphan read racing an in-flight rescan in the other order is untested — low, rejected: both share one `sequence` counter; the tested order is the harmful one.
- blind: reading preferences reuse the layout debounce pair — false: `src/AGENTS.md` names exactly three pairs and forbids a fourth.

## Verification

**Commands:**
- `npm run build && cargo test --test <each changed contract test>` -- expected: green.
- `npx vitest run <changed test files>` -- expected: green.
- `npm run check:debt-owner && npm run check:i18n && npm run check:commands && npm run check:tokens` -- expected: green.

## Acceptance criteria from epics.md

Source: `epics.md` §Story 11.6 (v6, nay ở archive-v6).

As a chủ dự án,
I want nợ nhóm đường nhập và Library (Epic 5, Epic 6) được đóng hoặc quyết dứt điểm,
So that dữ liệu vào TM là dữ liệu đã nhập đúng.

Thừa kế AC chung của epic (xem Notes của `epic-tra-no-nen.md`).

**Acceptance Criteria:**

**Given** mọi mục mang `Chủ: Story 11.6` trong `deferred-work.md`
**When** story hoàn tất
**Then** thoả AC chung của Epic 11; AC riêng rút từ các mục lúc `create-story`
