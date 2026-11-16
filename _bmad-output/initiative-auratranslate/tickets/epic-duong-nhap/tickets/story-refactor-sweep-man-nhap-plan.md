---
title: 'Story 6.30: Refactor sweep (màn nhập)'
type: 'refactor'
ticket: '30'
created: '2026-11-16'
status: 'built'
baseline_revision: '33746ecc2a9ff2c8278f2056ead1ec00dd7e5b20'
route: 'full'
route_source: 'auto'
risk: 'medium'
review: 'quick'
review_source: 'pinned'
lenses_ran: ['quick']
review_loop_iteration: 0
context: []
---

<frozen-after-approval reason="human-owned intent — do not modify unless human renegotiates">

## Intent

**Problem:** Stories 6.25 to 6.29 left the import path with test-only wrappers, near-copies of the append core in Rust, the docx switch duplicated across the two preview overlays, five extension lists that must agree without anything making them agree, and comments naming removed symbols.

**Approach:** Remove dead code, merge each duplicate into one function, component or helper, and fix comments that state something false. Behaviour, IPC shapes and test selectors stay as they are, except for the two decisions below.

**Decisions (Ice, 2026-11-16):**
- A file drop while the import overlay is closed is ignored: the drop handler does nothing unless the overlay is open. This closes the `deferred-work.md` item owned by Story 6.30.
- `noticeKey` and its always-empty `<p class="notice" role="status">` in `LibraryImportOverlay.vue` are removed, accepting the ~28px upward shift of the footer area.
- The plan stays whole, despite running past 1600 tokens.

## Boundaries & Constraints

**Always:** Every change keeps behaviour, IPC payloads, i18n keys and the `data-*` hooks used by tests. A dedup that cannot be shown neutral is dropped and noted in Implementation Notes. The tests moved off deleted wrappers keep their asserts. Comment edits follow AGENTS.md §Code comments.

**Never:** Dropping `"docx"` from `BILINGUAL_SUPPORTED_EXTENSIONS` (it narrows what the wire accepts). Deriving `PendingImportSource.docx_sidecar` from `docx_source` (unverified; the 6.29 review rejected a related change). Renaming the machine-read anchor `// Đường APPEND (Story 6.7b)` (`ipc_contract.rs:2144`). Bulk edits to comments in lines these stories did not make stale. The three findings 6.25/6.29 rejected.

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected Output / Behavior | Error Handling |
|----------|--------------|---------------------------|----------------|
| Append text into existing Work | `.txt`, destination open / closed | Same Chapters, statuses, reindex as before | unchanged |
| Append bilingual into existing Work | `.csv` 2 pairs into 3-Chapter Work | Chapters 4 and 5, origin `bilingual_import` | unchanged |
| Classify | every entry ext + `.pdf` + mixed batch | same kind and verdict per path as today | unchanged |
| Lone `.docx` text, resolve-language fails | `auto`, no detectable lang | same error and same override-reset state as today | unchanged |
| Drop, overlay closed | `aura://file-dropped` with 2 `.txt`, overlay closed | Tệp list, tab and `lastError` unchanged; opening the overlay shows no dropped rows | none |
| Drop, overlay open | same payload, overlay open | files join the Tệp list as today | unchanged |

</frozen-after-approval>

## Code Map

- `src-tauri/src/commands/project/mod.rs:2699` `stash_pending_import_source` · `:3021` `confirm_append_import_with_encoding` · `work_creation.rs:989` `append_chapters_to_work` -- test-only wrappers. Callers: `story_6_18_library.rs:313`, `cleanup_contract.rs` (587/735/1897/1990), `tm_contract.rs:498`, `bilingual_import_contract.rs:177`, `project_contract.rs` (7), `docx_contract.rs` (2). Doc links `wire.rs:598,623,1066`, `mod.rs:2694-2697`.
- `bilingual.rs:540-594` `confirm_bilingual_append_import_with_policy` ≈ `mod.rs:2969-3012` (resolve encoding, lock, clone shape, append, clear pending, `write_lifecycle_after_change`); head also repeats `bilingual.rs:489-500`. Text-path defaults `0, 1, false, &[], Refuse` at `mod.rs:2997-3001`, `work_creation.rs:1009-1013`.
- `wire.rs:986-1030` `append_into_destination` -- takes `AppHandle`/`PathBuf` by value (clones at `:953`, `:1352`); builds the same `OpenFailed` twice (`:1011-1018`, `:1023-1030`).
- `wire.rs:1274` `rebuild_bilingual_import_preview` calls `pending_destination(&app)` while holding `state`; read it in the lock block at `:1265-1273` (`current_pending_destination`).
- `wire.rs:732-783` lone-`.docx` text branch ≈ `docx_shape.rs:57-73`; resets order differs on the language-error path (file wire resets first). `ipc_contract.rs:1176` scans wires for literal `reset_*(&app);`.
- `core/segment/import.rs` -- ext lists `:67`, `:1044`, `:1048`, hard-coded match in `classify_import_paths` `:1139-1142`, `reject_batch_unsupported_extension` `:1038`; `wire.rs:1143` `is_docx_path` repeats the check. `import_bilingual_docx` `:1241` one-line wrapper; `detect_docx_shape` `:1288` can be private; `qualifying_table_rows` runs twice (`:1301`, `:1333`). Stale doc `:1046-1047`.
- Stale Rust comments: `wire.rs:22-24`, `:46-48`, `:829-833`; `mod.rs:866-868`; `work_creation.rs:690`.
- `src/ImportPreviewOverlay.vue:1132-1163` vs `src/BilingualImportPreviewOverlay.vue:208-215,431-446` -- docx switch; keep `data-ip-docx-shape`, `data-bip-docx-shape`, `data-docx-shape-option`. Carry types `importPreviewState.ts:2955-2983` vs `bilingualImportPreviewState.ts:804-832` (consumer `libraryImport.ts:840-862`).
- `src/modes/libraryImport.ts` -- path merge `:362-368` and `:975-976`; submittable verdict `:742` and `:886-890`; `importFileName`/`baseName` `:345-349` (also `bilingualImportPreviewState.ts:~143`); `noticeKey` `:463` (setters `:361,497,973`; import at `LibraryImportOverlay.vue:27`, `<p>` `:460` and its `.notice` CSS; test `libraryImportDropMultiple.test.ts:182`); drop handler `:942-978` (wired for LibraryMode's lifetime at `LibraryMode.vue:130`, no open check; overlay open flag `libraryImportOverlayIsOpen` in `libraryImportOverlayState.ts`).
- `src/modes/libraryImportFocus.ts` -- `openLibraryImportToAddChapters` repeats `openLibraryImportForm`; `tab` union retypes `ImportTab`.
- `src/LibraryImportOverlay.vue` -- unused CSS `.pick-row`/`.pick-name` `:568-580`; class `li-primary` `:479`.
- Stale webview comments: `libraryImport.ts:13-18,701-711,737-738,983`; `bilingualImportPreviewState.ts:385-386`; `importPreviewState.ts:172,1432,1612`; `LibraryMode.vue:138`; `commands/index.ts:211-216,239,249,370,516,536,617`; `main.ts:625`; `panels/sourcePanelState.ts:19`; `panels/editorPanelState.ts:2207`; `config/project.ts:129`; `e2e/support/importForm.mjs:1-26,104-106`; `tests/frontend/libraryImportDropMultiple.test.ts:14`.
- vitest -- `fakeClassification` mock factory in `importPreviewDestination:50`, `libraryImportRetryAfterFailedConfirm:32`, `libraryImportBlocksResubmitWhilePreviewOpen:38`, `libraryImportDropMultiple:36`, `libraryImportEntryMatrix:120`; `seedImportFiles` in the first three (`pick()` in EntryMatrix); move the factory body and seeding to `tests/frontend/support/importClassify.ts` (vi.mock hoists: share the body via async import).
- e2e -- `ready()`, `backToLibrary()`, open-overlay-and-wait copied across `story-6-25`…`6-29`; `importForm.mjs:31-32` already has `IMPORT_OPEN_BTN`/`IMPORT_PANEL`.

## Tasks & Acceptance

**Execution:** three phases, one fresh agent each (AGENTS.md §Policy): phase 1 Rust = the first four tasks; phase 2 Webview = the next two plus the ledger task; phase 3 Tests = vitest/e2e support consolidation and the Verification runs. Each phase appends its working notes (what changed, baseline case counts per target, anything left) to `/private/tmp/claude-501/-Users-hoangnam-LocalSites-addon-AuraTranslate/c18c6b4b-9769-4531-aa32-4c134b40ccb6/scratchpad/6-30-handoff.md` and reads the earlier phases' notes there first. Record baseline case counts from the baseline revision before changing a target.
- [ ] Rust tests -- move callers onto `_for`/`_with_progress` with neutral progress, then delete the three wrappers and fix their doc links.
- [ ] `bilingual.rs`, `mod.rs`, `work_creation.rs` -- one append core shared by text and bilingual confirms; a named neutral bilingual-params value.
- [ ] `wire.rs` -- borrow args and one error in `append_into_destination`; single lock in rebuild; dedup the lone-docx text branch only if the error-path reset order holds; fix stale comments.
- [ ] `import.rs` -- classifier reads the ext consts; one `is_docx` helper replaces `is_docx_path`; inline `import_bilingual_docx`; private `detect_docx_shape`; rows computed once; test that classifier kinds match the consts.
- [ ] `src/importPreview/DocxShapeSwitch.vue` (new) + both overlays + shared `DocxCarry` type.
- [ ] `libraryImport.ts`, `libraryImportFocus.ts`, `LibraryImportOverlay.vue` -- helpers above, dead CSS/class, stale comments; remove `noticeKey` and the notice `<p>`; drop handler returns early while the overlay is closed, with a vitest case for both matrix drop rows.
- [ ] `deferred-work.md` -- close the drop-while-closed item with `→ ✅ ĐÃ ĐÓNG <date> (Story 6.30)`.
- [ ] vitest support + e2e support -- consolidate helpers; specs keep their cases.

**Acceptance Criteria:**
- Given the baseline revision, when the touched Rust targets, the touched vitest files and e2e `story-6-23…6-29` and `story-12-6` run on the swept tree, then every case green before is green after, with the same case count per target plus the new drop and classifier cases.
- Given the open-check removed from the drop handler, when the new drop vitest case runs, then the closed-overlay case is red.
- Given the swept tree, when grepping `src-tauri/src` for the three deleted wrappers and `src`, `tests`, `e2e` for symbols named in the stale comments, then no hit remains.

## Implementation Notes

- Rust: three test-only wrappers deleted; text and bilingual confirms share `append_pending_import`; `BilingualParams::NEUTRAL` for text-path defaults; `is_docx` replaces `is_docx_path`.
- Dropped: lone-`.docx` text-branch dedup in `wire.rs` (reset order differs on the language-error path, not neutral).
- Webview: `DocxShapeSwitch.vue` + `docxCarry.ts` shared by both overlays; it must keep the `{prefix}-status {prefix}-error` classes on its error `<p>` (e2e 6-29 reads `.ip-error`, vitest does not).
- Drop handler ignores drops while the overlay is closed; `noticeKey` removed. e2e drops all happen with the overlay open.
- Design Notes counter-check done: removing the bilingual-segment insert in the append core turned 2 bilingual_import_contract cases red (Append in-progress, Unlinked); restored.
- Matrix row "lone `.docx`, language fails" had no test: `preview_import_encoding_from_file` took a concrete `AppHandle`, so it is now generic over `R` (the signature scans in `config_invariants.rs` and `ipc_contract.rs` follow) and `project_wire.rs` drives it with a digits-only `.docx`; removing the pending cancel on that branch turns it red at the pending assert.
- Test support: `seedImportFiles` moved to `tests/frontend/support/importClassify.ts`; e2e `waitLibraryReady`/`backToLibrary` moved to `e2e/support/importForm.mjs`.

## Plan Change Log

## Review Triage Log

Pass 1 (quick, loop 0): high 0 · medium 0 · low 5 · false 1 · maybe-false 0.

- low · patch — rewritten doc on `stash_pending_import_source_for` (`mod.rs`) keeps story ids "Story 6.12" and "spec 6.3"; ids dropped.
- false — doc on `confirm_append_import_with_encoding_with_progress` "describes a body that no longer exists": it states what the call does (clone pending, clear only on success, append + lifecycle write), and that still holds through `append_pending_import`; it never claims the calls are inline.
- low · patch — comments and one panic message in `project_contract.rs`, `docx_contract.rs`, `bilingual_import_contract.rs` still name the three deleted wrappers; renamed to the functions now called.
- low · patch — plan task left half done: the `vi.mock('../../src/config/library')` factory body is still copied in five vitest files and EntryMatrix's `pick()` builds its own mock; moved into `support/importClassify.ts`.
- low · reject — `ipc_contract` reindex counter accepts the borrowed form `reindex_library(app, root);`: the guard still requires exactly five call sites, so an extra borrowed call fails it; only a simultaneous removal plus addition would slip, unlikely in use.
- low · reject — `seedImportFiles` parameters typed loosely (`unknown`): a shape mismatch would still fail the cases at runtime; tightening adds generic mock types for no failure anyone meets.

## Design Notes

The shared append core is the one risky change: both confirms write Chapters, and a dropped `write_lifecycle_after_change` or `*guard = None` would lose state silently. The 6.28 counter-check (removing the `insert_bilingual_segments` branch turns Append/Unlinked red at `translation_origin`) must still turn red on the shared core.

## Verification

**Commands:**
- `cd src-tauri && cargo test --test bilingual_import_contract --test project_contract --test docx_contract --test docx_shape_contract --test ipc_contract --test cleanup_contract --test tm_contract --test story_6_18_library --test project_wire` and `cargo test --lib segment::import` -- green, case counts equal to baseline.
- `npx vitest run tests/frontend/libraryImport* tests/frontend/importPreview* tests/frontend/libraryAddChapters.test.ts tests/frontend/titlebarToolsMenu.test.ts` -- green, exit 0.
- `npm run check:fmt && npm run check:i18n && npm run check:commands` -- green.
- e2e specs above: `npm run test:e2e -- --spec <file>` (it builds `--features wdio`), outside the sandbox; any `cargo test` after that build overwrites the wdio binary, so rebuild before e2e -- green.
- `npm run build` before `cargo test` (without `dist/` it fails at compile time). Never the full suite: shared wiring (`lib.rs`, `Cargo.toml`, migrations) is not touched.
